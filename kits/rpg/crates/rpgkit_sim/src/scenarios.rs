//! Balance sim encounters (tools/sim/sim_scenarios.gd, BAL2_SIM.md sections 5 and 6.4): the generated party
//! encounters, catalog encounters with the party replaced, the five role limit tests, and the fight-set runner that
//! feeds the metrics. Every unit is a sandbox unit from class + level + tier; catalog encounters are only read.

use crate::kpi::Acc;
use crate::roles;
use crate::runner::{Def, Runner};
use crate::SimClasses;
use aegis_core::json;
use rpgkit_battle::encounter;
use rpgkit_battle::metrics::accumulate;
use rpgkit_rules::stats::MAX_LEVEL;
use indexmap::IndexMap;
use serde_json::{json, Map, Value};

pub const WAVE_ROUND: i32 = 2;
pub const WAVE_BLOCK: i32 = 1;
/// Striker band (8.1): the priority target down by this round.
pub const PRIORITY_ROUND: i32 = 5;

/// [class, tier] per unit.
pub type Roster = &'static [(&'static str, &'static str)];

/// 5: a generated encounter's roster (KITIDS1: the game's, SimClasses.generated): [class, tier] per unit; "@elite" /
/// "@boss" follow elite= / boss=; the wave arrives at WAVE_ROUND / WAVE_BLOCK.
#[derive(Debug, Clone, Copy)]
pub struct Generated {
    pub id: &'static str,
    pub enemies: Roster,
    pub wave: Roster,
}

/// The roster and wave of generated encounter `id` (None: the game gives it none).
pub fn generated(c: &SimClasses, id: &str) -> Option<(Roster, Roster)> {
    c.generated.iter().find(|g| g.id == id).map(|g| (g.enemies, g.wave))
}

/// 6.4: one limit test (KITIDS1: the game's, SimClasses.scenarios). `allies`: reference classes after the test unit;
/// `block`: the clumped enemy start; `tier_until`: up to that level the enemy at the slot uses the tier instead.
#[derive(Debug, Clone, Copy)]
pub struct Scenario {
    pub id: &'static str,
    pub role: &'static str,
    pub allies: &'static [&'static str],
    pub enemies: Roster,
    pub priority: bool,
    pub block: (i32, i32),
    pub tier_until: Option<(i32, usize, &'static str)>,
}

/// The limit test `id` (None: the game gives it none; `class_problem` refuses such a role line before any fight).
pub fn scenario_def(c: &SimClasses, id: &str) -> Option<Scenario> {
    c.scenarios.iter().find(|s| s.id == id).copied()
}

/// The scenario of each role.
pub fn role_scenario(role: &str) -> &'static str {
    match role {
        "tank" => "tank_hold",
        "ranged" => "ranged_dive",
        "aoe" => "aoe_swarm",
        "support" => "support_attrition",
        _ => "striker_priority",
    }
}

/// BAL5 S2: the scenario whose foes `s2=on` swaps for SimClasses.s2_enemies.
const S2_SCENARIO: &str = "support_attrition";

/// BAL5 B2: the generated boss's phase (BOSS1 Crown of Nails effects) on unit `boss`.
pub fn boss_phase_for(boss: &str) -> Value {
    json!({"id": "sim_boss_phase", "name": "Sim boss phase", "pause_autoplay": false,
        "trigger": {"kind": "hp_below", "unit": boss, "pct": 50},
        "effects": [
            {"type": "status", "unit": boss, "status": "fortify", "rounds": 3, "magnitude": 5},
            {"type": "status", "unit": boss, "status": "regen", "rounds": 3, "magnitude": 4},
            {"type": "ai", "unit": boss, "reckless": true, "target_pref": "WEAKEST"},
        ]})
}

/// One encounter of a fight set: enemy ids, encounter fields, label; max_rounds (world encounters; 0 = the cap).
#[derive(Debug, Clone, Default)]
pub struct Enc {
    /// REWARD1-S (world encounters): the purse range and the sim ids of the key loot kills.
    pub purse: Option<(i64, i64)>,
    pub key_kills: Vec<String>,
    pub enemies: Vec<String>,
    pub base: Map<String, Value>,
    pub label: String,
    pub max_rounds: i32,
    /// Campaign: a filler fight before a story battle.
    pub extra: bool,
    /// RULESET1: the ruleset of the world that resolves a world encounter (None: catalog or generated).
    pub ruleset: Option<rpgkit_rules::ruleset::Ruleset>,
}

/// Party units simp_<class>_<i> (i = slot, 1-based) with the loadout / spend arguments.
pub fn define_party(r: &mut Runner, comp: &[String], level: i32) -> Vec<String> {
    comp.iter()
        .enumerate()
        .map(|(i, c)| {
            let id = format!("simp_{c}_{}", i + 1);
            r.define(&id, Def::new(c, level, "hero", true));
            id
        })
        .collect()
}

/// Enemy units sime_<class>_<tier>_<i> (i continues across lists so names stay unique).
pub fn define_enemies(r: &mut Runner, list: &[(&str, &str)], level: i32, first: usize, elite: &str, boss: &str) -> Vec<String> {
    list.iter()
        .enumerate()
        .map(|(k, (class, tier))| {
            let class = match *class {
                "@elite" => elite,
                "@boss" => boss,
                c => c,
            };
            let id = format!("sime_{class}_{tier}_{}", first + k);
            r.define(&id, Def::new(class, level, tier, false));
            id
        })
        .collect()
}

/// Encounter `id` for a party at `level`: generated ids use level + elevel; catalog ids keep authored levels unless
/// elevel was given. boss_phase: the generated boss carries its sim phase.
#[allow(clippy::too_many_arguments)]
pub fn encounter(r: &mut Runner, id: &str, level: i32, elevel: i32, elevel_set: bool, elite: &str, boss: &str, boss_phase: bool) -> Enc {
    let foe_level = (level + elevel).clamp(1, MAX_LEVEL);
    if let Some((enemies, wave)) = generated(&r.args.game.classes, id) {
        let foes = define_enemies(r, enemies, foe_level, 1, elite, boss);
        let mut base = Map::new();
        let mut phases: Vec<Value> = Vec::new();
        if !wave.is_empty() {
            let wave_ids = define_enemies(r, wave, foe_level, foes.len() + 1, elite, boss);
            phases.push(encounter::arrival_phase(&wave_ids, WAVE_ROUND, WAVE_BLOCK, encounter::ARRIVAL_LABEL));
        }
        if boss_phase && id == "boss" {
            phases.push(boss_phase_for(&foes[0]));
        }
        if !phases.is_empty() {
            base.insert("phases".into(), Value::Array(phases));
        }
        return Enc { enemies: foes, base, label: id.into(), ..Enc::default() };
    }
    let entry = r.d.entry("encounters", id).clone();
    let mut base = Map::new();
    for key in ["zone", "arena", "exits", "total_blocks", "unit_ap", "start_ap", "ap_regen"] {
        if let Some(v) = entry.get(key) {
            base.insert(key.into(), v.clone());
        }
    }
    let enemies = catalog_ids(r, &json::sget(&entry, "enemies"), foe_level, elevel_set);
    // PHASEFOLD rule 7: the arrival phase only (the other phases stay out), its units as sandbox ids.
    if let Some(mut arrival) = arrival_of(&entry) {
        for fx in arrival.get_mut("effects").and_then(Value::as_array_mut).into_iter().flatten().filter_map(Value::as_object_mut).filter(|fx| fx.contains_key("units")) {
            let units = catalog_ids(r, &json::sget(fx, "units"), foe_level, elevel_set);
            fx.insert("units".into(), json!(units));
        }
        base.insert("phases".into(), json!([arrival]));
    }
    Enc { enemies, base, label: id.into(), ..Enc::default() }
}

/// PHASEFOLD: the encounter's arrival phase (the reinforcement wave; first in a valid list), if it has one.
pub fn arrival_of(entry: &Map<String, Value>) -> Option<Value> {
    json::aget(entry, "phases").iter().find(|p| json::s(json::dget(json::dict(p), "trigger"), "kind") == encounter::ARRIVAL).cloned()
}

/// Catalog roster ids, or level-shifted sandbox copies simc_<id> when elevel was given.
pub fn catalog_ids(r: &mut Runner, ids: &[String], level: i32, shifted: bool) -> Vec<String> {
    ids.iter()
        .map(|id| {
            if !shifted {
                return id.clone();
            }
            let e = r.d.entry("units", id).clone();
            let copy = format!("simc_{id}");
            // PR4-1 part 2 (AUDIT-07, FB-148): the shift goes through the world's raise, so an explicit `spend`
            // gets the raised levels' points
            let def = Def { copy_of: id.clone(), raise: true, ..Def::new(&json::s(&e, "class_id"), level, &rpgkit_data::unit_tier(&e), false) };
            r.define(&copy, def);
            copy
        })
        .collect()
}

/// A role scenario's units for test class `class_id`.
pub struct ScenarioUnits {
    pub party: Vec<String>,
    pub enemies: Vec<String>,
    /// The priority target's id ("" = none).
    pub priority: String,
    pub test: String,
    pub refs: Vec<String>,
}

/// A role scenario: the test unit simt_<class> (role loadout, variants apply), plain references simr_<class>_<i>,
/// enemies at the same level.
pub fn scenario(r: &mut Runner, id: &str, class_id: &str, level: i32, test_plain: bool, s2: bool) -> ScenarioUnits {
    let c = r.args.game.classes;
    let s = scenario_def(&c, id).expect("class_problem checked the role's limit test");
    let test = format!("simt_{class_id}{}", if test_plain { "_ref" } else { "" });
    if test_plain {
        r.define(&test, Def { plain: true, ..Def::new(class_id, level, "hero", true) });
    } else {
        let loadout = roles::role_loadout(&r.d.cat, &c, class_id, s.role, level);
        r.define(&test, Def { loadout, ..Def::new(class_id, level, "hero", true) });
    }
    let refs: Vec<String> = s
        .allies
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let rid = format!("simr_{c}_{}", i + 2);
            r.define(&rid, Def { plain: true, ..Def::new(c, level, "hero", true) });
            rid
        })
        .collect();
    let list = scenario_enemies(&c, id, level, s2);
    let enemies = define_enemies(r, &list, level, 1, c.elite, c.boss);
    let mut party = vec![test.clone()];
    party.extend(refs.iter().cloned());
    let priority = if s.priority { enemies[0].clone() } else { String::new() };
    ScenarioUnits { party, enemies, priority, test, refs }
}

/// The [class, tier] enemy list of a scenario at `level` (tier_until applied).
pub fn scenario_enemies(c: &SimClasses, id: &str, level: i32, s2: bool) -> Vec<(&'static str, &'static str)> {
    let Some(s) = scenario_def(c, id) else { return Vec::new() };
    let mut list: Vec<(&'static str, &'static str)> = if s2 && id == S2_SCENARIO && !c.s2_enemies.is_empty() { c.s2_enemies.to_vec() } else { s.enemies.to_vec() };
    if let Some((lvl, slot, tier)) = s.tier_until {
        if level <= lvl {
            list[slot] = (list[slot].0, tier);
        }
    }
    list
}

/// A set of fights and their sums (SimScenarios.empty_set), plus the role mode's derived side totals.
#[derive(Debug, Clone, Default)]
pub struct FightSet {
    pub fights: f64,
    pub cleared: f64,
    pub rounds_sum: f64,
    pub rounds_cleared_sum: f64,
    pub survivors_sum: f64,
    pub party_size_sum: f64,
    pub party_hp_sum: f64,
    pub hp_sum: f64,
    pub prio_by5: f64,
    pub prio_round_sum: f64,
    /// Unit name -> its summed records (all fights / cleared fights).
    pub units: IndexMap<String, Acc>,
    pub units_won: IndexMap<String, Acc>,
    pub skipped: f64,
    pub party_ids: Vec<String>,
    pub test: Acc,
    pub party: Acc,
    pub refs: Acc,
    pub refs_won: Acc,
    pub cap: i32,
    pub baseline_survivors: f64,
    /// REWARD1-S: the fights' reward sums and the encounter's purse mean (0 = none).
    pub rewards: RewardSum,
    pub purse_mean: f64,
}

/// REWARD1-S: what a set of fights' defeated enemies paid, summed (means are over `fights`).
#[derive(Debug, Clone, Default)]
pub struct RewardSum {
    pub fights: f64,
    pub roster: f64,
    pub slain: f64,
    pub surrendered: f64,
    pub fled: f64,
    pub xp: f64,
    /// Sum of the purse share fractions (defeated weight / roster weight).
    pub share: f64,
    pub drop_value: f64,
    pub keys: f64,
}

impl RewardSum {
    /// Adds one fight; `key_kills` = the key loot rows' kill ids (a row pays on a win with all its units killed).
    pub fn add(&mut self, rec: &crate::runner::RewardRec, won: bool, key_kills: &[String]) {
        self.fights += 1.0;
        self.roster += rec.roster as f64;
        self.slain += rec.slain as f64;
        self.surrendered += rec.surrendered as f64;
        self.fled += rec.fled as f64;
        self.xp += rec.xp as f64;
        self.share += if rec.total > 0 { rec.weight as f64 / rec.total as f64 } else { 0.0 };
        self.drop_value += rec.drop_value_x100 as f64 / 100.0;
        for kill in key_kills {
            let of: Vec<&rpgkit_battle::session::Fate> = rec.fates.iter().filter(|f| &f.0 == kill).map(|f| &f.1).collect();
            if won && !of.is_empty() && of.iter().all(|f| **f == rpgkit_battle::session::Fate::Killed) {
                self.keys += 1.0;
            }
        }
    }

    /// "defeated 3.0/4.0 (slain 2.5, surr 0.5, fled 0.0), xp/hero 27.0, gold 12.4, drops 1.85, key 0.40" (means over the
    /// fights; gold = the mean share fraction x the purse mean, no floor or once rule: an expected value).
    pub fn line(&self, purse_mean: f64) -> String {
        let n = self.fights.max(1.0);
        let m = |v: f64| aegis_core::fmt::fixed(v / n, 1);
        format!(
            "defeated {}/{} (slain {}, surr {}, fled {}), xp/hero {}, gold {}, drops {}, key {}",
            m(self.slain + self.surrendered + self.fled),
            m(self.roster),
            m(self.slain),
            m(self.surrendered),
            m(self.fled),
            m(self.xp),
            aegis_core::fmt::fixed(self.share / n * purse_mean, 1),
            aegis_core::fmt::fixed(self.drop_value / n, 2),
            aegis_core::fmt::fixed(self.keys / n, 2)
        )
    }
}

/// Runs `left` vs `enemies` for every seed with the runner's spend variant and preset; adds fight and per-unit sums
/// into `acc` (cleared fights also into units_won). Refused fights are skipped; jitter skips are counted.
pub fn run_seeds(r: &mut Runner, left: &[String], enemies: &[String], base: &Map<String, Value>, seeds: &[i64], priority: &str, acc: &mut FightSet) {
    r.metrics.priority_name = priority.to_string();
    for &s in seeds {
        let res = r.fight(left, enemies, s, base);
        if res.skipped {
            acc.skipped += 1.0;
        }
        if !res.ok {
            continue;
        }
        let f = r.metrics.fight_record.clone().unwrap_or_default();
        let kills = r.key_kills.clone();
        acc.rewards.add(&res.reward, res.won, &kills);
        acc.fights += 1.0;
        if f.cleared {
            acc.cleared += 1.0;
            acc.rounds_cleared_sum += f.rounds as f64;
        }
        acc.rounds_sum += f.rounds as f64;
        acc.survivors_sum += f.survivors as f64;
        acc.party_size_sum += left.len() as f64;
        acc.party_hp_sum += f.party_hp_frac;
        acc.hp_sum += f.hp_score;
        if f.priority_down_round <= PRIORITY_ROUND {
            acc.prio_by5 += 1.0;
        }
        acc.prio_round_sum += f.priority_down_round as f64;
        for (n, rec) in &r.metrics.units {
            accumulate(acc.units.entry(n.clone()).or_default(), rec);
            if f.cleared {
                accumulate(acc.units_won.entry(n.clone()).or_default(), rec);
            }
        }
    }
}

pub fn clear_pct(acc: &FightSet) -> f64 {
    acc.cleared / acc.fights.max(1.0) * 100.0
}

pub fn mean(acc: &FightSet, v: f64) -> f64 {
    v / acc.fights.max(1.0)
}

/// Field sums over the named units of a set (won = cleared fights only).
pub fn sum_units(acc: &FightSet, names: &[String], won: bool) -> Acc {
    let mut out = Acc::new();
    let per_unit = if won { &acc.units_won } else { &acc.units };
    for n in names {
        if let Some(u) = per_unit.get(n) {
            for (f, v) in u {
                *out.entry(f).or_insert(0.0) += v;
            }
        }
    }
    out
}
