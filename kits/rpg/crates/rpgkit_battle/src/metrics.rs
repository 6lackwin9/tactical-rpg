//! Battle metrics (BattleMetrics, scripts/combat/session/battle_metrics.gd; BREPORT1-L 1.2 and the counting part of the
//! balance sim collector): one record per unit per fight, built from the battle's HP journal (`World::hp_journal`),
//! its action log (`log::Entry`), its combos and its round-end steps. HP changes are buffered per unit; each log entry
//! credits the buffer to that entry's actor. Changes still buffered at a block's end have no entry (they fail the
//! checks); HP lost after the last block of a round is round-end damage (dot_taken, and dot_dealt of the status's
//! source); HP a unit with regen gains there is a heal. Statuses are diffed per entry and credited to their recorded
//! source (STATCRED1-L); positions are sampled per block. `detail` (status and combo counts, regen, best hit / finish
//! / save) feeds the battle report.
//!
//! Self-checks record "METRIC FAIL <unit> <field>": the log's damage (capped at the victim's HP) must equal the HP the
//! victims lost, a heal entry's amount the HP gained, an area entry's hit count the units it touched; a gain outside a
//! heal entry or a loss without an attack entry is unexplained.
//!
//! The session feeds it (`Session::attach_metrics`): the sim driver calls `Session::metrics_step` / `metrics_round` /
//! `metrics_end` like SimRunner's observer hooks; a live battle lets the session close each round itself.

use crate::grid::V2;
use crate::log::Entry;
use crate::resolver::Combo;
use crate::session::Session;
use crate::skill;
use crate::world::UnitId;
use rpgkit_rules::status;
use indexmap::{IndexMap, IndexSet};
use serde_json::{json, Map, Value};

/// near_death_saves: an ally under this HP share, then healed.
pub const NEAR_DEATH_FRAC: f64 = 0.25;
/// BAL5 K3: down_early = downed before this round.
pub const EARLY_DOWN_ROUND: i32 = 3;
pub const COMBO_KINDS: [&str; 3] = ["exploit", "break", "flow"];
/// A unit off the grid that was never seen on it (BattleMetrics._tile's fallback).
const NOWHERE: V2 = V2 { x: -99, y: -99 };

/// One unit's numbers for one fight (FIELDS, summed per unit per fight).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UnitRecord {
    pub dmg_dealt: i32,
    pub dmg_dealt_ranged: i32,
    pub dmg_taken: i32,
    pub dot_taken: i32,
    pub ff_dealt: i32,
    pub ff_taken: i32,
    pub attacks_received: i32,
    pub heal_done: i32,
    pub overheal: i32,
    pub heal_received: i32,
    pub cleanses: i32,
    pub buffs_applied: i32,
    pub buff_rounds: i32,
    pub debuffs_applied: i32,
    pub debuff_rounds: i32,
    pub area_casts: i32,
    pub area_foes_hit: i32,
    pub area_allies_hit: i32,
    pub kills: i32,
    pub blocks_alive: i32,
    pub survived: i32,
    pub hp_end_frac: f64,
    pub min_hp_frac: f64,
    pub near_death_saves: i32,
    pub dist_sum: i32,
    pub dist_samples: i32,
    pub blocks_adjacent: i32,
    pub hold_blocks: i32,
    pub taunt_blocks: i32,
    pub priority_dmg: i32,
    pub hp_start: i32,
    pub hp_end: i32,
    pub max_hp: i32,
    pub other_loss: i32,
    pub other_gain: i32,
    pub overheal_unknown: i32,
    pub regen_received: i32,
    pub dot_dealt: i32,
    pub heal_casts: i32,
    pub downed: i32,
    pub down_round: i32,
    pub down_early: i32,
    pub combos: i32,
}

impl UnitRecord {
    /// Every field in FIELDS order, as numbers (the KPI tables and the traces).
    pub fn fields(&self) -> [(&'static str, f64); 43] {
        let i = |v: i32| v as f64;
        [
            ("dmg_dealt", i(self.dmg_dealt)),
            ("dmg_dealt_ranged", i(self.dmg_dealt_ranged)),
            ("dmg_taken", i(self.dmg_taken)),
            ("dot_taken", i(self.dot_taken)),
            ("ff_dealt", i(self.ff_dealt)),
            ("ff_taken", i(self.ff_taken)),
            ("attacks_received", i(self.attacks_received)),
            ("heal_done", i(self.heal_done)),
            ("overheal", i(self.overheal)),
            ("heal_received", i(self.heal_received)),
            ("cleanses", i(self.cleanses)),
            ("buffs_applied", i(self.buffs_applied)),
            ("buff_rounds", i(self.buff_rounds)),
            ("debuffs_applied", i(self.debuffs_applied)),
            ("debuff_rounds", i(self.debuff_rounds)),
            ("area_casts", i(self.area_casts)),
            ("area_foes_hit", i(self.area_foes_hit)),
            ("area_allies_hit", i(self.area_allies_hit)),
            ("kills", i(self.kills)),
            ("blocks_alive", i(self.blocks_alive)),
            ("survived", i(self.survived)),
            ("hp_end_frac", self.hp_end_frac),
            ("min_hp_frac", self.min_hp_frac),
            ("near_death_saves", i(self.near_death_saves)),
            ("dist_sum", i(self.dist_sum)),
            ("dist_samples", i(self.dist_samples)),
            ("blocks_adjacent", i(self.blocks_adjacent)),
            ("hold_blocks", i(self.hold_blocks)),
            ("taunt_blocks", i(self.taunt_blocks)),
            ("priority_dmg", i(self.priority_dmg)),
            ("hp_start", i(self.hp_start)),
            ("hp_end", i(self.hp_end)),
            ("max_hp", i(self.max_hp)),
            ("other_loss", i(self.other_loss)),
            ("other_gain", i(self.other_gain)),
            ("overheal_unknown", i(self.overheal_unknown)),
            ("regen_received", i(self.regen_received)),
            ("dot_dealt", i(self.dot_dealt)),
            ("heal_casts", i(self.heal_casts)),
            ("downed", i(self.downed)),
            ("down_round", i(self.down_round)),
            ("down_early", i(self.down_early)),
            ("combos", i(self.combos)),
        ]
    }

    pub fn to_json(&self) -> Value {
        let mut m = Map::new();
        for (k, v) in self.fields() {
            m.insert(k.into(), if k.ends_with("_frac") { json!(v) } else { json!(v as i64) });
        }
        Value::Object(m)
    }
}

/// The biggest hit a unit dealt.
#[derive(Debug, Clone, PartialEq)]
pub struct BestHit {
    pub amount: i32,
    pub skill: String,
    pub target: String,
    pub round: i32,
    /// Index of the log entry it was credited at.
    pub order: i32,
}

/// The first combo kill (exploit or break) on a foe.
#[derive(Debug, Clone, PartialEq)]
pub struct BestFinish {
    pub skill: String,
    pub target: String,
    pub combo: String,
    pub status: String,
    pub round: i32,
    pub order: i32,
}

/// A heal on an ally under NEAR_DEATH_FRAC (`hp` = its HP before the heal).
#[derive(Debug, Clone, PartialEq)]
pub struct Save {
    pub ally: String,
    pub hp: i32,
    pub round: i32,
    pub order: i32,
}

/// Report detail of one unit (BREPORT1 1.2).
#[derive(Debug, Clone, PartialEq)]
pub struct Detail {
    pub status_counts: IndexMap<String, i32>,
    pub combo_counts: IndexMap<String, i32>,
    pub regen_done: i32,
    pub best_hit: Option<BestHit>,
    pub best_finish: Option<BestFinish>,
    /// Chosen from `saves` when the fight ends.
    pub best_save: Option<Save>,
    pub saves: Vec<Save>,
}

impl Default for Detail {
    fn default() -> Self {
        Self {
            status_counts: IndexMap::new(),
            combo_counts: COMBO_KINDS.iter().map(|k| (k.to_string(), 0)).collect(),
            regen_done: 0,
            best_hit: None,
            best_finish: None,
            best_save: None,
            saves: Vec::new(),
        }
    }
}

fn save_json(s: &Option<Save>) -> Value {
    s.as_ref().map_or(json!({}), |s| json!({"ally": s.ally, "hp": s.hp, "round": s.round, "order": s.order}))
}

impl Detail {
    pub fn to_json(&self) -> Value {
        json!({
            "status_counts": counts_json(&self.status_counts),
            "combo_counts": counts_json(&self.combo_counts),
            "regen_done": self.regen_done,
            "best_hit": self.best_hit.as_ref().map_or(json!({}), |h| json!({"amount": h.amount, "skill": h.skill, "target": h.target, "round": h.round, "order": h.order})),
            "best_finish": self.best_finish.as_ref().map_or(json!({}), |f| json!({"skill": f.skill, "target": f.target, "combo": f.combo, "status": f.status, "round": f.round, "order": f.order})),
            "best_save": save_json(&self.best_save),
            "saves": self.saves.iter().map(|s| save_json(&Some(s.clone()))).collect::<Vec<_>>(),
        })
    }
}

/// RULESET1 P9 (rule poise; engine-only, timeline keys): one unit's stagger and break counts for one fight. The hits
/// it dealt: `stagger_dealt` (poise taken), `windup_hits` (hits into a windup), `breaks` (breaks it dealt), of them
/// `windup_breaks` (the target was in windup), `chance_breaks` (by the chance roll) and `sure_breaks` (by `sure_break`),
/// and `chances` (break chance rolls); what it took: `broken` (times it broke).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PoiseRecord {
    pub stagger_dealt: i32,
    pub windup_hits: i32,
    pub breaks: i32,
    pub windup_breaks: i32,
    pub chances: i32,
    pub chance_breaks: i32,
    pub sure_breaks: i32,
    pub broken: i32,
}

impl PoiseRecord {
    pub fn to_json(&self) -> Value {
        json!({"stagger_dealt": self.stagger_dealt, "windup_hits": self.windup_hits, "breaks": self.breaks, "windup_breaks": self.windup_breaks,
            "chances": self.chances, "chance_breaks": self.chance_breaks, "sure_breaks": self.sure_breaks, "broken": self.broken})
    }
}

/// HOOKS1 4.2 (rule forced_move; engine-only, timeline keys): one unit's forced-move counts for one fight, kept only
/// for units a forced move involved. What it drove: `pushes` and `pulls` (moves that were not held), `tiles` (tiles it
/// moved others), `collisions` (its moves that hit something), `collision_damage` (HP its collisions took, blockers
/// included), `collision_breaks` (units its collisions broke), `stops` (Walks its moves cancelled); what it took:
/// `moved` (times it was moved) and `held` (times it held firm).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ForcedRecord {
    pub pushes: i32,
    pub pulls: i32,
    pub tiles: i32,
    pub collisions: i32,
    pub collision_damage: i32,
    pub collision_breaks: i32,
    pub stops: i32,
    pub moved: i32,
    pub held: i32,
}

impl ForcedRecord {
    pub fn to_json(&self) -> Value {
        json!({"pushes": self.pushes, "pulls": self.pulls, "tiles": self.tiles, "collisions": self.collisions, "collision_damage": self.collision_damage,
            "collision_breaks": self.collision_breaks, "stops": self.stops, "moved": self.moved, "held": self.held})
    }
}

/// The last fight's outcome numbers.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FightRecord {
    pub cleared: bool,
    pub rounds: i32,
    pub party_hp_frac: f64,
    pub enemy_hp_frac: f64,
    pub hp_score: f64,
    pub survivors: i32,
    pub priority_down_round: i32,
}

impl FightRecord {
    pub fn to_json(&self) -> Value {
        json!({"cleared": self.cleared, "rounds": self.rounds, "party_hp_frac": self.party_hp_frac, "enemy_hp_frac": self.enemy_hp_frac,
            "hp_score": self.hp_score, "survivors": self.survivors, "priority_down_round": self.priority_down_round})
    }
}

/// What the fight driver knows at the end (SimRunner's result; a live battle passes the default).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EndResult {
    pub cleared: bool,
    pub rounds: i32,
    pub l: f64,
    pub r: f64,
    pub hp: f64,
}

/// HP changes of one unit since the last entry.
#[derive(Debug, Clone, Copy, Default)]
struct Buf {
    start: i32,
    loss: i32,
    gain: i32,
}

type Rounds = IndexMap<String, i32>;

#[derive(Debug, Clone, Default)]
pub struct BattleMetrics {
    /// Display name of the scenario's priority target ("" = none).
    pub priority_name: String,
    /// Display name -> record of the last fight (allies, then enemies).
    pub units: IndexMap<String, UnitRecord>,
    pub detail: IndexMap<String, Detail>,
    /// RULESET1 P9: display name -> stagger and break counts of the last fight (empty without the rule poise).
    pub poise: IndexMap<String, PoiseRecord>,
    /// HOOKS1 4.2: display name -> forced-move counts of the last fight (only units a forced move involved; empty
    /// without the rule forced_move).
    pub forced: IndexMap<String, ForcedRecord>,
    /// Set when the fight ends.
    pub fight_record: Option<FightRecord>,
    pub fail_lines: Vec<String>,
    /// Self-checks evaluated (all fights).
    pub checks: i32,
    /// Sum every fight per unit name (`totals`).
    pub keep_totals: bool,
    pub totals: IndexMap<String, IndexMap<&'static str, f64>>,
    /// finish() ran: the records no longer change.
    pub frozen: bool,
    /// The session closes rounds itself (a live battle) instead of a driver calling on_step / on_round.
    pub live: bool,
    following: bool,
    by_name: IndexMap<String, UnitId>,
    /// Allies, then enemies: unit -> party side.
    ally: IndexMap<UnitId, bool>,
    names: IndexMap<UnitId, String>,
    last_hp: IndexMap<UnitId, i32>,
    buf: IndexMap<UnitId, Buf>,
    /// Attacker -> (start, loss): its own loss in its action, for the riposte entries.
    hold: IndexMap<UnitId, (i32, i32)>,
    pending: IndexMap<UnitId, Rounds>,
    active: IndexMap<UnitId, Rounds>,
    pos: IndexMap<UnitId, V2>,
    start_tile: IndexMap<UnitId, V2>,
    below: IndexSet<UnitId>,
    saves: IndexMap<UnitId, IndexSet<UnitId>>,
    round_end: bool,
    steps_seen: u32,
    priority_down: i32,
}

impl BattleMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    // ---- The fight's hooks -------------------------------------------------------------------------------------

    /// The fight starts (its first round is being planned): a fresh record per unit.
    pub fn on_start(&mut self, s: &Session) {
        self.following = true;
        self.frozen = false;
        self.units.clear();
        self.detail.clear();
        self.poise.clear();
        self.forced.clear();
        self.fight_record = None;
        self.by_name.clear();
        self.ally.clear();
        self.names.clear();
        self.last_hp.clear();
        self.buf.clear();
        self.hold.clear();
        self.pos.clear();
        self.start_tile.clear();
        self.below.clear();
        self.saves.clear();
        self.round_end = false;
        self.priority_down = 0;
        self.steps_seen = s.resolver.round_end_calls;
        let w = &s.w;
        for &c in w.allies.iter().chain(&w.enemies) {
            let u = &w.units[c];
            self.by_name.insert(u.name.clone(), c);
            self.ally.insert(c, w.allies.contains(&c));
            self.names.insert(c, u.name.clone());
            let rec = UnitRecord { hp_start: u.hp, max_hp: u.max_hp, min_hp_frac: u.hp as f64 / (u.max_hp as f64).max(1.0), ..UnitRecord::default() };
            self.units.insert(u.name.clone(), rec);
            self.detail.insert(u.name.clone(), Detail::default());
            if crate::poise::on(w) {
                self.poise.insert(u.name.clone(), PoiseRecord::default());
            }
            self.last_hp.insert(c, u.hp);
            self.remember_statuses(s, c);
            self.track_tile(s, c);
        }
    }

    /// After every resolution step (sim driver).
    pub fn on_step(&mut self, s: &Session) {
        self.credit_round_end(s);
        self.round_end = false;
        self.check_priority(s);
    }

    /// A new round began (sim driver): the roll-over changed statuses outside any entry; new baseline.
    pub fn on_round(&mut self, s: &Session) {
        for c in self.ally.keys().copied().collect::<Vec<_>>() {
            self.remember_statuses(s, c);
        }
    }

    /// The fight is over: survival, saves, the fight record and the conservation checks.
    pub fn on_end(&mut self, s: &Session, result: EndResult) {
        self.credit_round_end(s);
        self.flush_hold();
        self.flush_unexplained();
        self.check_priority(s);
        let cap = s.def.max_rounds;
        let w = &s.w;
        for c in self.ally.keys().copied().collect::<Vec<_>>() {
            let u = &w.units[c];
            let saved = self.saves.get(&c).map_or(0, |set| set.iter().filter(|a| w.units[**a].is_active()).count() as i32);
            let rec = self.rec(c);
            rec.survived = u.is_active() as i32;
            rec.hp_end = u.hp;
            rec.hp_end_frac = (if u.is_active() { u.hp } else { 0 }) as f64 / (u.max_hp as f64).max(1.0);
            rec.near_death_saves = saved;
            self.pick_save(s, c);
        }
        let survivors = w.allies.iter().filter(|&&c| w.units[c].is_active()).count() as i32;
        self.fight_record = Some(FightRecord {
            cleared: result.cleared,
            rounds: result.rounds,
            party_hp_frac: result.l,
            enemy_hp_frac: result.r,
            hp_score: result.hp,
            survivors,
            priority_down_round: if self.priority_down > 0 { self.priority_down } else { cap + 1 },
        });
        self.check_conservation();
        if self.keep_totals {
            for (n, rec) in &self.units {
                accumulate(self.totals.entry(n.clone()).or_default(), rec);
            }
        }
        self.following = false;
    }

    /// The live fight is over: closes the records as on_end does, then freezes them.
    pub fn finish(&mut self, s: &Session) {
        if !self.frozen {
            self.on_end(s, EndResult::default());
        }
        self.frozen = true;
    }

    /// Live: a round started (its roll-over is done).
    pub fn round_started(&mut self, s: &Session) {
        self.credit_round_end(s);
        self.round_end = false;
        self.check_priority(s);
        self.on_round(s);
    }

    /// Live: a round ended (after round_closed).
    pub fn round_concluded(&mut self, s: &Session) {
        self.credit_round_end(s);
        self.round_end = false;
        self.check_priority(s);
    }

    /// Whether the session should feed this collector.
    pub fn is_following(&self) -> bool {
        self.following && !self.frozen
    }

    // ---- What the session reports --------------------------------------------------------------------------------

    /// Combatant.health_changed: unit `c` now has `new_hp`.
    pub fn on_health(&mut self, s: &Session, c: UnitId, new_hp: i32) {
        let Some(&last) = self.last_hp.get(&c) else { return };
        let d = new_hp - last;
        self.last_hp.insert(c, new_hp);
        let u = &s.w.units[c];
        let max_hp = u.max_hp;
        let round = s.current_round;
        let rec = self.rec(c);
        rec.min_hp_frac = rec.min_hp_frac.min(new_hp as f64 / (max_hp as f64).max(1.0));
        let first_down = new_hp <= 0 && rec.downed == 0;
        if first_down {
            rec.downed = 1;
            rec.down_round = round;
            rec.down_early = (round < EARLY_DOWN_ROUND) as i32;
        }
        if new_hp > 0 && (new_hp as f64) < NEAR_DEATH_FRAC * max_hp as f64 {
            self.below.insert(c);
        }
        if self.round_end {
            if d < 0 {
                self.rec(c).dot_taken -= d;
            } else if d > 0 && u.has_status("regen") {
                let rec = self.rec(c);
                rec.heal_received += d;
                rec.regen_received += d;
                let source = u.st.sources.get("regen").cloned().unwrap_or_default();
                if let Some(healer) = self.unit(&source) {
                    self.rec(healer).heal_done += d;
                    self.det(healer).regen_done += d;
                }
            } else if d > 0 {
                self.rec(c).other_gain += d;
            }
            return;
        }
        let b = self.buf.entry(c).or_insert(Buf { start: new_hp - d, loss: 0, gain: 0 });
        if d < 0 {
            b.loss -= d;
        } else if d > 0 {
            b.gain += d;
        }
    }

    /// session.log_appended: the entry at `idx` of the session's log was just added.
    pub fn on_log(&mut self, s: &Session, idx: usize) {
        let e = &s.log[idx];
        let actor = self.unit(&e.actor);
        if e.category == "ATTACK" && e.skill == "Riposte" && e.subtype == "Counter" {
            let target = self.unit(&e.target);
            self.riposte(s, actor, target, e);
            return;
        }
        self.flush_hold();
        let changed = std::mem::take(&mut self.buf);
        let Some(actor) = actor else {
            self.unexplained(&changed);
            return;
        };
        match e.category {
            "ATTACK" if e.subtype == crate::log::COLLISION => self.collision(s, actor, e, &changed),
            "ATTACK" => self.attack(s, actor, e, &changed),
            "BUFF" => self.support(s, actor, e, &changed),
            _ => self.unexplained(&changed),
        }
        for c in self.ally.keys().copied().collect::<Vec<_>>() {
            self.status_diff(s, actor, e, c);
            self.remember_statuses(s, c);
            self.track_tile(s, c);
        }
        self.check_priority(s);
    }

    /// CombatResolver.combo_resolved (before the action's log entry, so `order` is that entry's index).
    pub fn on_combo(&mut self, s: &Session, k: &Combo) {
        let Some(actor_name) = self.names.get(&k.actor).cloned() else { return };
        if !self.units.contains_key(&actor_name) {
            return;
        }
        self.rec(k.actor).combos += 1;
        let foes = self.foes(k.actor, k.target);
        let target = s.w.units[k.target].name.clone();
        let order = s.log.len() as i32;
        let d = self.det(k.actor);
        *d.combo_counts.entry(k.kind.to_string()).or_insert(0) += 1;
        if k.killed && k.kind != "flow" && foes && d.best_finish.is_none() {
            d.best_finish = Some(BestFinish { skill: k.skill.clone(), target, combo: k.kind.into(), status: k.status.clone(), round: s.current_round, order });
        }
    }

    /// RULESET1 P9: the stagger records of the action at `idx` of the round (its unit dealt them).
    pub fn on_staggers(&mut self, s: &Session, idx: usize) {
        let Some(a) = s.resolver.scheduled.get(idx) else { return };
        let Some(actor) = self.names.get(&a.unit).cloned() else { return };
        for st in &a.command.staggers {
            self.stagger_hit(s, &actor, st);
        }
    }

    /// RULESET1 P9: one stagger record `actor` dealt.
    fn stagger_hit(&mut self, s: &Session, actor: &str, st: &crate::poise::StaggerHit) {
        let target = s.w.units[st.target].name.clone();
        if let Some(r) = self.poise.get_mut(actor) {
            r.stagger_dealt += st.before - st.after.max(0);
            r.windup_hits += st.windup as i32;
            r.chances += st.chance.is_some() as i32;
            if let Some(b) = &st.broke {
                r.breaks += 1;
                r.windup_breaks += st.windup as i32;
                r.chance_breaks += (b.cause == crate::poise::BreakCause::Chance) as i32;
                r.sure_breaks += (b.cause == crate::poise::BreakCause::Sure) as i32;
            }
        }
        if st.broke.is_some() {
            if let Some(r) = self.poise.get_mut(&target) {
                r.broken += 1;
            }
        }
    }

    /// HOOKS1 4.2: the forced moves of the action at `idx` of the round (its unit drove them): the forced-move counts,
    /// and the collisions' stagger and breaks as its poise record's (rule poise).
    pub fn on_forced(&mut self, s: &Session, idx: usize) {
        let Some(a) = s.resolver.scheduled.get(idx) else { return };
        for m in &a.command.pushes {
            let driver = m.by.and_then(|b| self.names.get(&b).cloned());
            if let Some(name) = &driver {
                let r = self.forced.entry(name.clone()).or_default();
                if !m.held {
                    if m.pull {
                        r.pulls += 1;
                    } else {
                        r.pushes += 1;
                    }
                    r.tiles += m.tiles;
                    r.collisions += !m.blocked_by.is_empty() as i32;
                    r.collision_damage += m.hits.iter().map(|h| h.damage).sum::<i32>();
                    r.collision_breaks += m.hits.iter().filter(|h| h.stagger.as_ref().is_some_and(|st| st.broke.is_some())).count() as i32;
                    r.stops += m.stopped;
                }
            }
            if let Some(name) = self.names.get(&m.unit).cloned() {
                let r = self.forced.entry(name).or_default();
                if m.held {
                    r.held += 1;
                } else {
                    r.moved += 1;
                }
            }
            if let Some(name) = &driver {
                for st in m.hits.iter().filter_map(|h| h.stagger.as_ref()) {
                    self.stagger_hit(s, name, st);
                }
            }
        }
    }

    /// resolver.block_completed: unexplained changes, a position sample, and the round end begins after the last block.
    pub fn on_block_completed(&mut self, s: &Session, block: i32) {
        self.flush_hold();
        self.flush_unexplained();
        self.sample_block(s);
        if block >= s.def.total_blocks {
            self.round_end = true;
        }
    }

    /// session.round_concluded (before the next round's roll-over resets the status baseline): statuses applied after
    /// the round's last entry are credited by source only.
    pub fn on_round_closed(&mut self, s: &Session) {
        for c in self.ally.keys().copied().collect::<Vec<_>>() {
            self.credit_statuses(s, None, c);
            self.remember_statuses(s, c);
        }
    }

    // ---- HP changes and their entries ----------------------------------------------------------------------------

    /// A single-target or area attack: every other unit's loss is a hit by `actor` (exact per victim).
    fn attack(&mut self, s: &Session, actor: UnitId, e: &Entry, changed: &IndexMap<UnitId, Buf>) {
        let area = e.subtype == "Area Attack";
        let mut target = self.unit(&e.target);
        let (mut lost, mut capped_log, mut touched) = (0, 0, 0);
        for (&v, b) in changed {
            if b.gain > 0 {
                self.rec(v).other_gain += b.gain;
            }
            if v == actor {
                if b.loss > 0 {
                    self.hold.insert(actor, (b.start, b.loss));
                }
                continue;
            }
            touched += 1;
            if target.is_none() && !area {
                target = Some(v);
            }
            lost += b.loss;
            capped_log += b.start.min(e.damage);
            self.hit(s, actor, v, b.loss, b.start, e);
            if area {
                if self.foes(actor, v) {
                    self.rec(actor).area_foes_hit += 1;
                    self.rec(v).attacks_received += 1;
                } else {
                    self.rec(actor).area_allies_hit += 1;
                }
            }
        }
        self.checks += 1;
        let name = self.names[&actor].clone();
        if area {
            self.rec(actor).area_casts += 1;
            let hits = if e.outcome != "MISS" { e.outcome.split(' ').next().and_then(|n| n.parse().ok()).unwrap_or(0) } else { 0 };
            if touched != hits || lost > e.damage {
                self.fail(&name, &format!("log_area {}: {hits} hits / {} damage logged, {touched} units / {lost} HP lost", e.skill, e.damage));
            }
            return;
        }
        if let Some(t) = target.filter(|t| self.foes(actor, *t)) {
            self.rec(t).attacks_received += 1;
        }
        let expected = if touched > 0 { e.damage.min(capped_log) } else { e.damage };
        if touched > 1 || lost != expected {
            self.fail(&name, &format!("log_damage {}: {} logged (capped {expected}), {lost} HP lost by {touched} units", e.skill, e.damage));
        }
    }

    /// HOOKS1 FM4: a collision cell: the one unit it names lost exactly the logged HP, credited to the driver as a hit
    /// (friendly fire on its own side or on itself); it is not an attack received.
    fn collision(&mut self, s: &Session, actor: UnitId, e: &Entry, changed: &IndexMap<UnitId, Buf>) {
        let target = self.unit(&e.target);
        let (mut lost, mut touched) = (0, 0);
        for (&v, b) in changed {
            if b.gain > 0 {
                self.rec(v).other_gain += b.gain;
            }
            if b.loss <= 0 {
                continue;
            }
            touched += 1;
            lost += b.loss;
            if Some(v) != target {
                self.rec(v).other_loss += b.loss;
                continue;
            }
            self.hit(s, actor, v, b.loss, b.start, e);
        }
        self.checks += 1;
        if touched > 1 || lost != e.damage {
            let name = self.names[&actor].clone();
            self.fail(&name, &format!("log_collision {}: {} logged, {lost} HP lost by {touched} units", e.skill, e.damage));
        }
    }

    /// A riposte: the defender's counter explains part of the attacker's held loss.
    fn riposte(&mut self, s: &Session, defender: Option<UnitId>, attacker: Option<UnitId>, e: &Entry) {
        self.checks += 1;
        let (start, left) = attacker.and_then(|a| self.hold.get(&a).copied()).unwrap_or((0, 0));
        let mut take = e.damage.min(start);
        if defender.is_none() || attacker.is_none() || take > left {
            self.fail(&e.actor, &format!("log_riposte: {} logged (capped {take}), {left} HP held", e.damage));
            take = take.min(left);
        }
        let (Some(defender), Some(attacker)) = (defender, attacker) else { return };
        self.hit(s, defender, attacker, take, start, e);
        self.hold.insert(attacker, (start - take, left - take));
    }

    /// A support entry: gains are heals by `actor` (the target's must equal the logged heal). An item entry (a pouch
    /// potion) balances the ledger as heal_received on its side only.
    fn support(&mut self, s: &Session, actor: UnitId, e: &Entry, changed: &IndexMap<UnitId, Buf>) {
        let target = self.unit(&e.target);
        let mut target_gain = 0;
        let heal_entry = e.subtype == "Support";
        let item_entry = e.subtype == "Item";
        for (&c, b) in changed {
            if b.loss > 0 {
                self.rec(c).other_loss += b.loss;
            }
            if b.gain <= 0 {
                continue;
            }
            if item_entry && !self.foes(actor, c) {
                self.rec(c).heal_received += b.gain;
                if Some(c) == target {
                    target_gain = b.gain;
                }
                continue;
            }
            if !heal_entry || self.foes(actor, c) {
                self.rec(c).other_gain += b.gain;
                continue;
            }
            self.rec(c).heal_received += b.gain;
            self.rec(actor).heal_done += b.gain;
            if Some(c) == target {
                target_gain = b.gain;
            }
            if c != actor && self.below.contains(&c) {
                self.saves.entry(actor).or_default().insert(c);
            }
            let max_hp = s.w.units[c].max_hp;
            if c != actor && b.start > 0 && (b.start as f64) < NEAR_DEATH_FRAC * max_hp as f64 {
                let save = Save { ally: self.names[&c].clone(), hp: b.start, round: s.current_round, order: s.log.len() as i32 - 1 };
                self.det(actor).saves.push(save);
            }
        }
        let name = self.names[&actor].clone();
        if item_entry {
            self.checks += 1;
            if target_gain != e.value {
                self.fail(&name, &format!("log_item_heal {}: {} logged, {target_gain} HP gained", e.skill, e.value));
            }
        }
        if heal_entry {
            self.rec(actor).heal_casts += 1;
            self.checks += 1;
            if target_gain != e.value {
                self.fail(&name, &format!("log_heal {}: {} logged, {target_gain} HP gained", e.skill, e.value));
            }
            self.overheal(s, actor, e, target_gain);
        }
    }

    /// Buffered changes no entry explains (they fail the conservation check).
    fn unexplained(&mut self, changed: &IndexMap<UnitId, Buf>) {
        for (&c, b) in changed {
            let rec = self.rec(c);
            rec.other_loss += b.loss;
            rec.other_gain += b.gain;
        }
    }

    fn flush_unexplained(&mut self) {
        let changed = std::mem::take(&mut self.buf);
        self.unexplained(&changed);
    }

    /// An attacker's own loss that no riposte entry explained.
    fn flush_hold(&mut self) {
        for (c, (_, left)) in std::mem::take(&mut self.hold) {
            if left > 0 {
                self.rec(c).other_loss += left;
            }
        }
    }

    fn hit(&mut self, s: &Session, actor: UnitId, victim: UnitId, loss: i32, start: i32, e: &Entry) {
        if loss <= 0 {
            return;
        }
        if !self.foes(actor, victim) {
            self.rec(victim).ff_taken += loss;
            self.rec(actor).ff_dealt += loss;
            return;
        }
        self.rec(actor).dmg_dealt += loss;
        self.rec(victim).dmg_taken += loss;
        let order = s.log.len() as i32 - 1;
        let target = self.names[&victim].clone();
        let d = self.det(actor);
        if loss > d.best_hit.as_ref().map_or(0, |h| h.amount) {
            d.best_hit = Some(BestHit { amount: loss, skill: e.skill.clone(), target: target.clone(), round: s.current_round, order });
        }
        if self.tile(s, actor).dist(self.tile(s, victim)) > 1 {
            self.rec(actor).dmg_dealt_ranged += loss;
        }
        if !self.priority_name.is_empty() && target == self.priority_name {
            self.rec(actor).priority_dmg += loss;
        }
        if start > 0 && start - loss <= 0 {
            self.rec(actor).kills += 1;
        }
    }

    fn check_priority(&mut self, s: &Session) {
        if self.priority_name.is_empty() || self.priority_down > 0 || !self.following {
            return;
        }
        if let Some(p) = self.unit(&self.priority_name.clone()) {
            if !s.w.units[p].is_active() {
                self.priority_down = s.current_round;
            }
        }
    }

    /// Overheal of a heal cast: the skill's heal amount (SupportRules.heal_amount) minus the HP gained.
    fn overheal(&mut self, s: &Session, healer: UnitId, e: &Entry, gained: i32) {
        if e.outcome == "FAILED" {
            return;
        }
        let w = &s.w;
        let mut amount = -1;
        for (id, sk) in w.cat.skills.iter().enumerate() {
            if sk.name == e.skill && skill::heals(sk) && w.units[healer].can_use(&w.cat, sk) {
                amount = amount.max(crate::command::heal_amount(w, healer, id));
            }
        }
        let rec = self.rec(healer);
        if amount < 0 {
            if gained > 0 {
                rec.overheal_unknown += 1;
            }
            return;
        }
        rec.overheal += (amount - gained).max(0);
    }

    /// Statuses `actor`'s entry granted or inflicted (credit_statuses), and debuffs it removed from its side with a
    /// BUFF entry (cleanses).
    fn status_diff(&mut self, s: &Session, actor: UnitId, e: &Entry, c: UnitId) {
        self.credit_statuses(s, Some(actor), c);
        if e.category != "BUFF" || self.foes(actor, c) || !s.w.units[c].is_alive() {
            return;
        }
        let (new_pending, new_active) = status_rounds(s, c);
        let mut had: Vec<String> = self.pending.get(&c).map(|m| m.keys().cloned().collect()).unwrap_or_default();
        for k in self.active.get(&c).map(|m| m.keys().cloned().collect::<Vec<_>>()).unwrap_or_default() {
            if !had.contains(&k) {
                had.push(k);
            }
        }
        let debuffs = status::debuffs();
        for st in had {
            if debuffs.contains(&st.as_str()) && !new_pending.contains_key(&st) && !new_active.contains_key(&st) {
                self.rec(actor).cleanses += 1;
            }
        }
    }

    /// Statuses that appeared on `c` since the last baseline, credited to their applier: the unit named as the status's
    /// source, else `actor`; a boss phase counts for an enemy recipient, a start buff for no one. Buffs count on the
    /// applier's side, debuffs on its foes. Re-applying a queued status at the same length is not visible.
    fn credit_statuses(&mut self, s: &Session, actor: Option<UnitId>, c: UnitId) {
        let st = &s.w.units[c].st;
        let (new_pending, new_active) = status_rounds(s, c);
        let old_pending = self.pending.get(&c).cloned().unwrap_or_default();
        let old_active = self.active.get(&c).cloned().unwrap_or_default();
        let (buffs, debuffs) = (status::buffs(), status::debuffs());
        for (now, before, sources) in [(&new_pending, &old_pending, &st.pending_sources), (&new_active, &old_active, &st.sources)] {
            for (name, &rounds) in now {
                if before.get(name).is_some_and(|&b| b >= rounds) {
                    continue;
                }
                let source = sources.get(name).cloned().unwrap_or_default();
                let Some(applier) = self.applier(actor, &source, c) else { continue };
                let counted = if buffs.contains(&name.as_str()) && !self.foes(applier, c) {
                    let rec = self.rec(applier);
                    rec.buffs_applied += 1;
                    rec.buff_rounds += rounds;
                    true
                } else if debuffs.contains(&name.as_str()) && self.foes(applier, c) {
                    let rec = self.rec(applier);
                    rec.debuffs_applied += 1;
                    rec.debuff_rounds += rounds;
                    true
                } else {
                    false
                };
                if counted {
                    *self.det(applier).status_counts.entry(name.clone()).or_insert(0) += 1;
                }
            }
        }
    }

    /// The unit credited with a status on `recipient` whose recorded source is `source`.
    fn applier(&self, actor: Option<UnitId>, source: &str, recipient: UnitId) -> Option<UnitId> {
        if source.is_empty() {
            return actor;
        }
        if source.starts_with("phase:") {
            return (self.ally.get(&recipient) == Some(&false)).then_some(recipient);
        }
        if source.starts_with("start:") {
            return None;
        }
        self.unit(source).or(actor)
    }

    fn remember_statuses(&mut self, s: &Session, c: UnitId) {
        let (p, a) = status_rounds(s, c);
        self.pending.insert(c, p);
        self.active.insert(c, a);
    }

    /// Credits the round end's damage steps (bleed, poison, burn) to the status's source as dot_dealt, once per
    /// apply_round_end_effects call.
    fn credit_round_end(&mut self, s: &Session) {
        if s.resolver.round_end_calls == self.steps_seen {
            return;
        }
        self.steps_seen = s.resolver.round_end_calls;
        for row in &s.resolver.round_end_steps {
            if row.status == "regen" {
                continue;
            }
            if let Some(source) = self.unit(&row.source) {
                self.rec(source).dot_dealt += row.amount;
            }
        }
    }

    // ---- Per-block sampling --------------------------------------------------------------------------------------

    fn sample_block(&mut self, s: &Session) {
        let w = &s.w;
        let order: Vec<UnitId> = self.ally.keys().copied().collect();
        for &c in &order {
            self.track_tile(s, c);
            if !w.has(c) {
                continue;
            }
            let here = w.pos(c);
            self.rec(c).blocks_alive += 1;
            let nearest = order.iter().filter(|&&f| self.foes(c, f) && w.has(f)).map(|&f| here.dist(w.pos(f))).min();
            if let Some(d) = nearest {
                let rec = self.rec(c);
                rec.dist_sum += d;
                rec.dist_samples += 1;
                if d <= 1 {
                    rec.blocks_adjacent += 1;
                }
            }
            let start = self.start_tile.get(&c).copied().unwrap_or(here);
            if here.dist(start) <= 1 {
                self.rec(c).hold_blocks += 1;
            }
            if w.units[c].st.active.contains("taunt") {
                self.rec(c).taunt_blocks += 1;
            }
        }
    }

    fn track_tile(&mut self, s: &Session, c: UnitId) {
        if s.w.has(c) {
            let t = s.w.pos(c);
            self.pos.insert(c, t);
            self.start_tile.entry(c).or_insert(t);
        }
    }

    fn tile(&self, s: &Session, c: UnitId) -> V2 {
        if s.w.has(c) {
            return s.w.pos(c);
        }
        self.pos.get(&c).copied().unwrap_or(NOWHERE)
    }

    // ---- Checks and helpers --------------------------------------------------------------------------------------

    /// Per unit: hp_start - dmg_taken - ff_taken - dot_taken + heal_received == hp_end; unexplained changes are
    /// reported on their own. Per side: party dmg_dealt == enemy dmg_taken and the reverse.
    fn check_conservation(&mut self) {
        let (mut dealt, mut taken) = ([0, 0], [0, 0]);
        for c in self.ally.keys().copied().collect::<Vec<_>>() {
            let side = if self.ally[&c] { 0 } else { 1 };
            let name = self.names[&c].clone();
            let rec = self.units[&name].clone();
            dealt[side] += rec.dmg_dealt;
            taken[side] += rec.dmg_taken;
            self.checks += 1;
            let expect = rec.hp_start - rec.dmg_taken - rec.ff_taken - rec.dot_taken + rec.heal_received;
            if expect != rec.hp_end {
                self.fail(&name, &format!("hp ({expect} expected, {} at the end)", rec.hp_end));
            }
            for (field, v) in [("other_loss", rec.other_loss), ("other_gain", rec.other_gain)] {
                if v != 0 {
                    self.fail(&name, &format!("{field} {v}"));
                }
            }
        }
        self.checks += 2;
        if dealt[0] != taken[1] {
            self.fail("party", &format!("dmg_dealt {} != enemy dmg_taken {}", dealt[0], taken[1]));
        }
        if dealt[1] != taken[0] {
            self.fail("enemy", &format!("dmg_dealt {} != party dmg_taken {}", dealt[1], taken[0]));
        }
    }

    fn fail(&mut self, unit: &str, field: &str) {
        self.fail_lines.push(format!("METRIC FAIL {unit} {field}"));
    }

    fn unit(&self, name: &str) -> Option<UnitId> {
        self.by_name.get(name).copied()
    }

    fn foes(&self, a: UnitId, b: UnitId) -> bool {
        self.ally.get(&a).copied().unwrap_or(false) != self.ally.get(&b).copied().unwrap_or(false)
    }

    fn rec(&mut self, c: UnitId) -> &mut UnitRecord {
        let name = &self.names[&c];
        self.units.get_mut(name).expect("a record per unit")
    }

    fn det(&mut self, c: UnitId) -> &mut Detail {
        let name = self.names[&c].clone();
        self.detail.entry(name).or_default()
    }

    /// The best save of `healer`: of its heals on an ally under NEAR_DEATH_FRAC who is still alive, the lowest HP
    /// before the heal, then the earliest.
    fn pick_save(&mut self, s: &Session, healer: UnitId) {
        let by_name = self.by_name.clone();
        let d = self.det(healer);
        let mut best: Option<Save> = None;
        for sv in &d.saves {
            let alive = by_name.get(&sv.ally).is_some_and(|&a| s.w.units[a].is_alive());
            if alive && best.as_ref().is_none_or(|b| sv.hp < b.hp) {
                best = Some(sv.clone());
            }
        }
        d.best_save = best;
    }

    /// The parity traces' shape: {units: {name: record}, detail: {name: detail}, fight, checks, fails}; RULESET1 P9 adds
    /// `poise: {name: record}` only under the rule poise (classic keeps its shape).
    pub fn to_json(&self) -> Value {
        let units: Map<String, Value> = self.units.iter().map(|(n, r)| (n.clone(), r.to_json())).collect();
        let detail: Map<String, Value> = self.detail.iter().map(|(n, d)| (n.clone(), d.to_json())).collect();
        let mut out = json!({"units": units, "detail": detail, "fight": self.fight_record.as_ref().map_or(json!({}), |f| f.to_json()),
            "checks": self.checks, "fails": self.fail_lines});
        if !self.poise.is_empty() {
            out["poise"] = Value::Object(self.poise.iter().map(|(n, r)| (n.clone(), r.to_json())).collect());
        }
        // HOOKS1 4.2: only when a forced move happened (the rule on without one keeps the shape)
        if !self.forced.is_empty() {
            out["forced"] = Value::Object(self.forced.iter().map(|(n, r)| (n.clone(), r.to_json())).collect());
        }
        out
    }
}

/// A count map as a JSON object (in its order).
pub fn counts_json(m: &IndexMap<String, i32>) -> Value {
    Value::Object(m.iter().map(|(k, v)| (k.clone(), json!(v))).collect())
}

/// {status: rounds} of `c`'s queued and active statuses (a status without a round count has 1).
fn status_rounds(s: &Session, c: UnitId) -> (Rounds, Rounds) {
    let st = &s.w.units[c].st;
    let pending = st.pending.iter().map(|k| (k.clone(), st.pending_rounds.get(k).copied().unwrap_or(1))).collect();
    let active = st.active.iter().map(|k| (k.clone(), st.rounds_left.get(k).copied().unwrap_or(1))).collect();
    (pending, active)
}

/// Adds `rec` into `acc` (field sums plus "fights").
pub fn accumulate(acc: &mut IndexMap<&'static str, f64>, rec: &UnitRecord) {
    for (k, v) in rec.fields() {
        *acc.entry(k).or_insert(0.0) += v;
    }
    *acc.entry("fights").or_insert(0.0) += 1.0;
}
