//! Balance sim runner (tools/sim/sim_runner.gd, BAL2_SIM.md sections 1-3): sandbox units with variants, one fight
//! with the round cap / preset / start distance / enemy block, the binary and HP-margin scores, and the side-swapped
//! matchup average. One BattleMetrics follows every fight (the conservation checks run in every mode).

use crate::args::SimArgs;
use crate::roles;
use crate::variants;
use aegis_core::json;
use aegis_core::rng::GodotRng;
use emberwake_battle::encounter::Phases;
use emberwake_battle::grid::NO_CELL;
use emberwake_battle::metrics::{BattleMetrics, EndResult};
use emberwake_battle::session::{EncounterDef, Fate, Phase, Roster, Session};
use emberwake_battle::sim::{binary_score, hp_score, jitter_start, plan_skip, shuffled, side_hp_frac, JITTER_SALT, MAX_STEPS_PER_ROUND};
use emberwake_data::{Catalog, UnitDef};
use emberwake_world::Data;
use indexmap::IndexMap;
use serde_json::{json, Map, Value};

/// A declared sandbox unit (SimRunner.define).
#[derive(Debug, Clone, PartialEq)]
pub struct Def {
    pub class_id: String,
    pub level: i32,
    pub tier: String,
    pub party: bool,
    /// An explicit family list (wins over the loadout argument).
    pub loadout: Vec<String>,
    /// A party reference unit: class spend, default loadout.
    pub plain: bool,
    /// A catalog unit copied with `level` (its class and tier).
    pub copy_of: String,
    /// The story character (world, ref) the unit plays: it spawns with that character's extras.
    pub character: Option<(String, String)>,
}

impl Def {
    pub fn new(class_id: &str, level: i32, tier: &str, party: bool) -> Self {
        Self { class_id: class_id.into(), level, tier: tier.into(), party, loadout: Vec::new(), plain: false, copy_of: String::new(), character: None }
    }
}

/// REWARD1-S: how one fight left its enemies and what the defeated ones are worth (rewards per defeated enemy).
#[derive(Debug, Clone, Default)]
pub struct RewardRec {
    pub roster: i32,
    pub slain: i32,
    pub surrendered: i32,
    pub fled: i32,
    /// Kill XP per hero at the sim's party level, over the defeated enemies.
    pub xp: i64,
    /// Kill weight of the defeated enemies and of the whole roster: the purse share is weight / total.
    pub weight: i64,
    pub total: i64,
    /// Expected item value of the defeated enemies' loot rolls, in gold x100.
    pub drop_value_x100: i64,
    /// (sim unit id, fate) per enemy, spawn order.
    pub fates: Vec<(String, Fate)>,
}

impl RewardRec {
    pub fn defeated(&self) -> i32 {
        self.slain + self.surrendered + self.fled
    }
}

/// What one fight gives back (SimRunner.fight's record).
#[derive(Debug, Clone, Default)]
pub struct FightRes {
    /// False: a unit was refused, or the fight was skipped.
    pub ok: bool,
    pub skipped: bool,
    pub l: f64,
    pub r: f64,
    pub bin: f64,
    pub hp: f64,
    pub rounds: i32,
    pub cleared: bool,
    pub won: bool,
    pub survivors: i32,
    pub party_size: i32,
    pub plan_fail: bool,
    /// Party unit name -> HP and max HP at the end.
    pub party_hp: IndexMap<String, i32>,
    pub party_max: IndexMap<String, i32>,
    pub reward: RewardRec,
}

/// Mean scores of a matchup: bin, hp, n (fights counted).
#[derive(Debug, Clone, Copy, Default)]
pub struct Cell {
    pub bin: f64,
    pub hp: f64,
    pub n: usize,
}

/// The sandbox roster: written sim units, catalog units, story characters with their extras.
struct SimRoster<'a> {
    d: &'a Data,
    units: &'a IndexMap<String, UnitDef>,
    characters: IndexMap<String, (String, String)>,
}

impl Roster for SimRoster<'_> {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        self.units.get(r).cloned().or_else(|| cat.units.get(r).cloned())
    }

    fn customize(&self, cat: &Catalog, r: &str, u: &mut emberwake_rules::Unit) {
        if let Some((w, c)) = self.characters.get(r) {
            emberwake_world::battle::apply_character(self.d, cat, self.d.resolve(w, "characters", c), w, u);
        }
    }
}

/// The AP of a unit built for the stats table (ContentFactory.unit's default; the table shows no AP).
const STATS_AP: i32 = 12;

pub struct Runner<'a> {
    pub d: &'a Data,
    pub args: &'a SimArgs,
    pub defs: IndexMap<String, Def>,
    /// Unit id -> the entry written for the spend variant in `written`.
    units: IndexMap<String, UnitDef>,
    written: IndexMap<String, String>,
    pub spend_variant: String,
    pub preset: String,
    pub metrics: BattleMetrics,
    pub not_moved_total: i32,
    pub fights: i32,
    pub plan_fails: i32,
    pub plan_skips: i32,
    /// Columns x rows of the clumped enemy block ((0, 0) = none).
    pub enemy_block: (i32, i32),
    pub block_misses: i32,
    /// Campaign: party unit name -> the HP the fight starts with (absent = full).
    pub start_hp: IndexMap<String, i32>,
    /// > 0 replaces the `cap` round limit.
    pub rounds_override: i32,
    /// An objective win (hold, escape, slay) counts as cleared.
    pub objective_wins: bool,
    /// REWARD1-S: the sim ids of the encounter's key loot kills (one per key loot row), for the key loot count.
    pub key_kills: Vec<String>,
    /// The printed lines (the "§" lines of the Godot sim, without the mark).
    pub out: Vec<String>,
}

impl<'a> Runner<'a> {
    pub fn new(d: &'a Data, args: &'a SimArgs) -> Self {
        Self {
            d,
            args,
            defs: IndexMap::new(),
            units: IndexMap::new(),
            written: IndexMap::new(),
            spend_variant: "class".into(),
            preset: "balanced".into(),
            metrics: BattleMetrics::new(),
            not_moved_total: 0,
            fights: 0,
            plan_fails: 0,
            plan_skips: 0,
            enemy_block: (0, 0),
            block_misses: 0,
            start_hp: IndexMap::new(),
            rounds_override: 0,
            objective_wins: false,
            key_kills: Vec::new(),
            out: Vec::new(),
        }
    }

    pub fn print(&mut self, line: impl Into<String>) {
        self.out.push(line.into());
    }

    /// SimRunner.define: declares sandbox unit `id` (an unchanged declaration keeps the entry already written).
    pub fn define(&mut self, id: &str, def: Def) {
        if self.defs.get(id) == Some(&def) {
            return;
        }
        self.defs.insert(id.into(), def);
        self.written.shift_remove(id);
    }

    /// SimRunner._write: the unit entry of `id` for the current spend variant. Undeclared ids are catalog units.
    fn write(&mut self, id: &str) -> bool {
        let Some(def) = self.defs.get(id).cloned() else {
            if self.d.cat.units.contains_key(id) {
                return true;
            }
            self.print(format!("SIM unknown unit {id}"));
            return false;
        };
        let variant = if def.party && !def.plain { self.spend_variant.clone() } else { "class".to_string() };
        if self.written.get(id) == Some(&variant) {
            return true;
        }
        let mut entry: Map<String, Value> = if def.copy_of.is_empty() {
            json!({"id": id, "name": id, "side": if def.party { "party" } else { "enemy" }, "class_id": def.class_id, "level": def.level, "tier": def.tier, "flee_ratio": 0.0})
                .as_object()
                .unwrap()
                .clone()
        } else {
            let mut e = self.d.entry("units", &def.copy_of).clone();
            e.insert("id".into(), json!(id));
            e.insert("level".into(), json!(def.level));
            e
        };
        if !def.loadout.is_empty() {
            entry.insert("loadout".into(), json!(def.loadout));
        }
        if def.party && !def.plain {
            entry.insert("spend".into(), Value::Object(variants::spend_for(&self.d.cat, &variant, &def.class_id, def.level)));
            if def.loadout.is_empty() && self.args.loadout == "newest" {
                entry.insert("loadout".into(), json!(variants::newest_loadout(&self.d.cat, &def.class_id, def.level)));
            } else if def.loadout.is_empty() && self.args.loadout == "role" {
                entry.insert("loadout".into(), json!(roles::role_loadout(&self.d.cat, &def.class_id, roles::primary_role(&def.class_id), def.level)));
            }
        }
        self.units.insert(id.into(), UnitDef::from_obj(&entry));
        self.written.insert(id.into(), variant);
        true
    }

    /// The unit of declared `id` for the current variant (stats mode).
    pub fn build(&mut self, id: &str) -> Option<emberwake_rules::Unit> {
        if !self.write(id) {
            return None;
        }
        let def = self.units.get(id)?;
        Some(emberwake_rules::build_unit(&self.d.cat, def, STATS_AP))
    }

    /// One battle of `left` (party) vs `right` (enemies); `base` replaces encounter defaults (zone, arena, exits,
    /// blocks, AP, waves, phases, objective...); party, enemies, max_rounds and rng_seed are the sim's.
    pub fn fight(&mut self, left: &[String], right: &[String], seed: i64, base: &Map<String, Value>) -> FightRes {
        let mut waves: Vec<String> = json::aget(base, "wave_units").iter().chain(json::aget(base, "guests")).map(json::gstr).collect();
        let phases: Vec<Value> = json::aget(base, "phases").to_vec();
        for p in &Phases::create(&phases).phases {
            waves.extend(p.wave_refs());
        }
        for id in left.iter().chain(right).chain(&waves) {
            if !self.write(id) {
                return FightRes::default();
            }
        }
        let mut entry = json!({"id": "sim_fight", "name": "Sim", "zone": "stone_arena", "arena": "CLOSED", "total_blocks": 6, "unit_ap": 12, "ap_regen": 4}).as_object().unwrap().clone();
        for (k, v) in base {
            entry.insert(k.clone(), v.clone());
        }
        let mut jitter = self.args.jitter.then(|| GodotRng::from_i64(seed.wrapping_mul(JITTER_SALT).wrapping_add(1)));
        let party_ids = match jitter.as_mut() {
            Some(r) => shuffled(left, r),
            None => left.to_vec(),
        };
        let round_limit = if self.rounds_override > 0 { self.rounds_override } else { self.args.cap };
        entry.insert("party".into(), json!(party_ids));
        entry.insert("enemies".into(), json!(right));
        entry.insert("max_rounds".into(), json!(round_limit));
        entry.insert("rng_seed".into(), json!(seed));
        let characters: IndexMap<String, (String, String)> =
            left.iter().chain(right).chain(&waves).filter_map(|id| self.defs.get(id).and_then(|d| d.character.clone()).map(|c| (id.clone(), c))).collect();
        let roster = SimRoster { d: self.d, units: &self.units, characters };
        let mut s = Session::start(self.d.cat.clone(), EncounterDef::from_obj(&entry), &roster);
        self.prepare(&mut s, jitter.as_mut());
        for &a in &s.w.allies.clone() {
            if let Some(&hp) = self.start_hp.get(&s.w.units[a].name) {
                let u = &mut s.w.units[a];
                u.hp = hp.clamp(0, u.max_hp);
            }
        }
        s.attach_metrics(std::mem::take(&mut self.metrics), false);
        let mut rounds = 0;
        let mut plan_failed = false;
        for round in 0..round_limit {
            rounds = round + 1;
            s.plan_apply_preset_all(&self.preset);
            if !s.confirm_plan() {
                if self.args.jitter && self.plan_skip(&s, round + 1) {
                    self.metrics = *s.metrics.take().unwrap_or_default();
                    return FightRes { skipped: true, ..FightRes::default() };
                }
                self.report_plan_fail(&s, round + 1);
                plan_failed = true;
                break;
            }
            for _ in 0..MAX_STEPS_PER_ROUND {
                if s.phase == Phase::Replay {
                    break;
                }
                s.step_next();
                s.metrics_step();
            }
            if s.combat_over || !s.can_advance_round() {
                break;
            }
            s.advance_round();
            s.metrics_round();
        }
        let l = side_hp_frac(&s.w, &s.w.allies);
        let r = side_hp_frac(&s.w, &s.w.enemies);
        let survivors = s.w.allies.iter().filter(|&&c| s.w.units[c].is_active()).count() as i32;
        let won = s.outcome == "won";
        let res = FightRes {
            ok: true,
            skipped: false,
            l,
            r,
            bin: if won { 1.0 } else { binary_score(l, r) },
            hp: if won { 1.0 } else { hp_score(l, r) },
            rounds,
            cleared: r <= 0.0 || (self.objective_wins && won),
            won: won || r <= 0.0,
            survivors,
            party_size: s.w.allies.len() as i32,
            plan_fail: plan_failed,
            party_hp: s.w.allies.iter().map(|&c| (s.w.units[c].name.clone(), s.w.units[c].hp)).collect(),
            party_max: s.w.allies.iter().map(|&c| (s.w.units[c].name.clone(), s.w.units[c].max_hp)).collect(),
            reward: self.reward_of(&s),
        };
        s.metrics_end(EndResult { cleared: res.cleared, rounds, l, r, hp: res.hp });
        self.metrics = *s.metrics.take().unwrap_or_default();
        self.fights += 1;
        res
    }

    /// REWARD1-S: the fates of the session's enemies, their kill XP for a hero at the sim level, their kill weights and
    /// the expected value of their loot rolls (the base unit's table: chance x item value x count).
    fn reward_of(&self, s: &Session) -> RewardRec {
        let d = self.d;
        let mut rec = RewardRec { fates: s.fates(), ..RewardRec::default() };
        let fates = rec.fates.clone();
        for (id, fate) in &fates {
            // a sim def (a level copy, a world character), else the catalog unit itself
            let def = self.defs.get(id);
            let entry = d.entry("units", id);
            let level = def.map_or_else(|| json::i(entry, "level", 1), |x| x.level as i64);
            let tier = def.map_or_else(|| json::s_or(entry, "tier", "standard"), |x| x.tier.clone());
            let weight = emberwake_world::progression::kill_weight(d, level, &tier);
            rec.roster += 1;
            rec.total += weight;
            match fate {
                Fate::Killed => rec.slain += 1,
                Fate::Surrendered => rec.surrendered += 1,
                Fate::Fled => rec.fled += 1,
                Fate::Standing => continue,
            }
            rec.weight += weight;
            rec.xp += emberwake_world::progression::kill_xp(d, level, &tier, self.args.level as i64);
            let base = def.map(|x| x.copy_of.clone()).filter(|c| !c.is_empty()).unwrap_or_else(|| id.clone());
            for row in json::aget(d.entry("units", &base), "loot").iter().filter_map(serde_json::Value::as_object) {
                let value = json::i(d.entry("items", &json::s(row, "item")), "value", 0);
                rec.drop_value_x100 += json::i(row, "chance", 0) * value * json::i(row, "count", 1);
            }
        }
        rec
    }

    /// SimRunner._prepare (after start): enemy presets, the seeded jitter, the start distance and the enemy block,
    /// the party pouch; then the round-1 enemy plan is redrafted.
    fn prepare(&mut self, s: &mut Session, jitter: Option<&mut GodotRng>) {
        let mut changed = false;
        if self.preset != "balanced" {
            for &e in &s.w.enemies.clone() {
                if let Some(p) = s.w.profiles.get(&e).cloned() {
                    s.w.profiles.insert(e, p.with_preset(&self.preset));
                }
            }
            changed = true;
        }
        let mut jitter = jitter;
        if let Some(r) = jitter.as_deref_mut() {
            jitter_start(&mut s.w, r);
            changed = true;
        }
        let (allies, enemies) = (s.w.allies.clone(), s.w.enemies.clone());
        if self.args.distance > 0 {
            self.not_moved_total += variants::apply_distance(&mut s.w, &allies, &enemies, self.args.distance);
            changed = true;
        }
        if self.enemy_block != (0, 0) {
            let shift = jitter.as_deref_mut().map_or(0, |r| r.randi_range(-1, 1));
            if !variants::apply_block(&mut s.w, &allies, &enemies, self.enemy_block, self.args.distance, shift) {
                self.block_misses += 1;
            }
            changed = true;
        }
        if !self.args.pouch.is_empty() {
            emberwake_battle::sim::apply_pouch(&mut s.w, &self.args.pouch);
        }
        if changed {
            s.replan_enemies();
        }
    }

    /// K6: every refusal is a reach error: the fight is skipped (one PLAN SKIP line).
    fn plan_skip(&mut self, s: &Session, round: i32) -> bool {
        if !plan_skip(s) {
            return false;
        }
        let reasons: Vec<String> = s
            .w
            .allies
            .iter()
            .filter(|&&a| s.w.units[a].is_active())
            .filter_map(|&a| {
                let st = s.plan.compile(&s.w, a);
                (st.error_index >= 0).then(|| format!("{} {}", s.w.units[a].name, st.error_text))
            })
            .collect();
        self.plan_skips += 1;
        self.print(format!("PLAN SKIP round {round}, preset {} (jitter, unreachable target; fight skipped): {}", self.preset, reasons.join("; ")));
        true
    }

    /// A refused plan ends the fight early: one PLAN FAIL line per invalid ally.
    fn report_plan_fail(&mut self, s: &Session, round: i32) {
        self.plan_fails += 1;
        let w = &s.w;
        for &a in &w.allies {
            if !w.units[a].is_active() {
                continue;
            }
            let st = s.plan.compile(w, a);
            if st.error_index < 0 {
                continue;
            }
            let all = s.plan.steps_of(a);
            let steps: Vec<String> = all.iter().map(|p| format!("{}{}", p.title(w), if p.cell.get() != NO_CELL { format!(" @{}", p.cell.get()) } else { String::new() })).collect();
            let bad = all[st.error_index as usize].cell.get();
            let mut holders = Vec::new();
            for &o in w.allies.iter().chain(&w.enemies) {
                if o != a && w.units[o].is_active() && w.has(o) && bad != NO_CELL && (w.pos(o) == bad || s.plan.planned_position(w, o) == bad) {
                    holders.push(format!("{} {}->{}", w.units[o].name, w.pos(o), s.plan.planned_position(w, o)));
                }
            }
            let line = format!(
                "PLAN FAIL {} {} (round {round}, preset {}, at {}, step {} of [{}]; tile held by [{}])",
                w.units[a].name,
                st.error_text,
                self.preset,
                w.pos(a),
                st.error_index + 1,
                steps.join(", "),
                holders.join(", ")
            );
            self.out.push(line);
        }
    }

    /// Mean scores of `a` vs `b` over spend variants x presets x seeds, sides swapped.
    pub fn matchup(&mut self, a: &[String], b: &[String]) -> Cell {
        let (mut bin, mut hp, mut n) = (0.0, 0.0, 0usize);
        let none = Map::new();
        for sv in self.args.spend_variants() {
            self.spend_variant = sv;
            for pv in self.args.preset_variants() {
                self.preset = pv;
                for s in self.args.seeds() {
                    let ab = self.fight(a, b, s, &none);
                    let ba = self.fight(b, a, s, &none);
                    if !ab.ok || !ba.ok {
                        continue;
                    }
                    bin += ab.bin + 1.0 - ba.bin;
                    hp += ab.hp + 1.0 - ba.hp;
                    n += 2;
                }
            }
        }
        self.spend_variant = "class".into();
        self.preset = "balanced".into();
        let d = (n as f64).max(1.0);
        Cell { bin: bin / d, hp: hp / d, n }
    }

    /// A score cell: "bin/hp" in percent (or one of them per `score`); "n/a" when nothing ran.
    pub fn cell(&self, c: Cell) -> String {
        if c.n == 0 {
            return "n/a".into();
        }
        let (b, h) = ((c.bin * 100.0).round() as i64, (c.hp * 100.0).round() as i64);
        if !self.args.show_hp() {
            return format!("{b}%");
        }
        if !self.args.show_binary() {
            return format!("{h}");
        }
        format!("{b}/{h}")
    }

    /// The distance note under a table.
    pub fn distance_note(&mut self) {
        if self.args.distance > 0 {
            let line = format!("distance {}: {} units not moved (summed over {} fights)", self.args.distance, self.not_moved_total, self.fights);
            self.print(line);
        }
    }
}

