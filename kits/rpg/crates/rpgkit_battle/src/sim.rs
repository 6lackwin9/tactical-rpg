//! The balance-sim fight (tools/sim/sim_runner.gd `fight`, sim_variants.gd jitter): one AI-vs-AI battle with the
//! seeded start jitter, scored like the reference. Optionally records a step trace in the Godot exporter's shape, with
//! what a BattleMetrics following the fight counted, the battle report and the action log (PORT-BREPORT).

use crate::grid::{Grid, V2};
use crate::metrics::{BattleMetrics, EndResult};
use crate::report::Report;
use crate::session::{EncounterDef, Phase, Session};
use crate::world::{UnitId, World};
use aegis_core::rng::GodotRng;
use rpgkit_data::{Catalog, UnitDef};
use indexmap::IndexMap;
use serde_json::{json, Value};
use std::sync::Arc;

pub const MAX_STEPS_PER_ROUND: usize = 600;
pub const JITTER_SALT: i64 = 7919;

#[derive(Debug, Clone)]
pub struct FightSpec {
    pub left: Vec<String>,
    pub right: Vec<String>,
    pub seed: i64,
    pub preset: String,
    pub cap: i32,
    pub jitter: bool,
    /// Encounter fields that replace the sim defaults (zone, arena, exits, blocks, AP, waves...); None = the stone arena.
    pub base: Option<serde_json::Map<String, Value>>,
    /// SimArgs `pouch=`: every party unit's battle pouch becomes these catalog consumables and its AI may use items.
    pub pouch: Vec<String>,
    /// RULESET1: the rules of the fight (SimArgs `ruleset=`, `rules=`; classic by default).
    pub ruleset: rpgkit_rules::ruleset::Ruleset,
}

#[derive(Debug, Clone, Default)]
pub struct FightResult {
    pub ok: bool,
    pub plan_fail: bool,
    /// Jitter fight whose every plan refusal was a reach error (SimRunner._plan_skip): not counted.
    pub skipped: bool,
    /// HP fractions of each side at the end.
    pub l: f64,
    pub r: f64,
    pub bin: f64,
    pub hp: f64,
    pub rounds: i32,
    /// `Session::final_outcome` (ENCOUNTER2 S2: the round limit gives `timeout_outcome`).
    pub outcome: String,
    /// The step trace (when requested): {"start": [...], "rounds": [{"round", "actions", "intents", "steps"}],
    /// "metrics", "report", "log"}.
    pub trace: Option<Value>,
    /// The fight's metrics (when a trace was requested).
    pub metrics: Option<Box<BattleMetrics>>,
}

fn units_state(w: &World) -> Value {
    let mut out = Vec::new();
    for &c in w.allies.iter().chain(w.enemies.iter()) {
        let u = &w.units[c];
        let p = w.grid.position(c);
        let mut active: Vec<&String> = u.st.active.iter().collect();
        let mut pending: Vec<&String> = u.st.pending.iter().collect();
        active.sort();
        pending.sort();
        out.push(json!([u.name, u.hp, u.ap, p.x, p.y, u.stance as i32, active, pending, u.is_withdrawn]));
    }
    Value::Array(out)
}

/// SimVariants.shuffled: a seeded Fisher-Yates copy.
pub fn shuffled(ids: &[String], rng: &mut GodotRng) -> Vec<String> {
    let mut out = ids.to_vec();
    let mut i = out.len();
    while i > 1 {
        i -= 1;
        let j = rng.randi_range(0, i as i32) as usize;
        out.swap(i, j);
    }
    out
}

/// SimVariants.jitter_start: each deployed unit (left side first) moves to a seeded pick of its own tile and its free
/// neighbours on the same half.
pub fn jitter_start(w: &mut World, rng: &mut GodotRng) {
    let units: Vec<UnitId> = w.allies.iter().chain(w.enemies.iter()).copied().collect();
    for c in units {
        if !w.has(c) {
            continue;
        }
        let here = w.pos(c);
        let mut options = vec![here];
        for off in [V2::new(0, -1), V2::new(-1, 0), V2::new(1, 0), V2::new(0, 1)] {
            let tile = here + off;
            if !w.grid.is_walkable(tile) || w.grid.is_exit(tile) || w.is_occupied(tile) {
                continue;
            }
            if Grid::is_party_half(tile) != Grid::is_party_half(here) {
                continue;
            }
            options.push(tile);
        }
        let pick = options[rng.randi_range(0, options.len() as i32 - 1) as usize];
        if pick != here {
            w.remove(c);
            w.place(c, pick);
        }
    }
}

/// SimRunner.side_hp_frac: remaining HP over max HP of a side (inactive units count 0).
pub fn side_hp_frac(w: &World, units: &[UnitId]) -> f64 {
    let hp: f64 = units.iter().map(|&c| if w.units[c].is_active() { w.units[c].hp as f64 } else { 0.0 }).sum();
    let max: f64 = units.iter().map(|&c| w.units[c].max_hp as f64).sum();
    hp / max.max(1.0)
}

pub fn binary_score(l: f64, r: f64) -> f64 {
    if r <= 0.0 && l > 0.0 {
        return 1.0;
    }
    if l <= 0.0 && r > 0.0 {
        return 0.0;
    }
    if l > r + 0.02 {
        1.0
    } else if r > l + 0.02 {
        0.0
    } else {
        0.5
    }
}

pub fn hp_score(l: f64, r: f64) -> f64 {
    if r <= 0.0 && l > 0.0 {
        return 1.0;
    }
    if l <= 0.0 && r > 0.0 {
        return 0.0;
    }
    if l <= 0.0 && r <= 0.0 {
        return 0.5;
    }
    (0.5 + (l - r) / 2.0).clamp(0.0, 1.0)
}

/// One sim battle (SimRunner.fight on the sim's stone arena).
pub fn fight(cat: Arc<Catalog>, units: &IndexMap<String, UnitDef>, spec: &FightSpec, record: bool) -> FightResult {
    let mut jitter_rng = spec.jitter.then(|| GodotRng::from_i64(spec.seed.wrapping_mul(JITTER_SALT).wrapping_add(1)));
    let party = match jitter_rng.as_mut() {
        Some(r) => shuffled(&spec.left, r),
        None => spec.left.clone(),
    };
    let mut entry = json!({"id": "sim_fight", "name": "Sim", "zone": "stone_arena", "arena": "CLOSED", "total_blocks": 6, "unit_ap": 12, "ap_regen": 4});
    let obj = entry.as_object_mut().unwrap();
    if let Some(base) = &spec.base {
        for (k, v) in base {
            obj.insert(k.clone(), v.clone());
        }
    }
    obj.insert("party".into(), json!(party));
    obj.insert("enemies".into(), json!(spec.right));
    obj.insert("max_rounds".into(), json!(spec.cap));
    obj.insert("rng_seed".into(), json!(spec.seed));
    let def = EncounterDef::from_obj_with(obj, spec.ruleset.clone());
    let mut s = Session::start(cat, def, units);
    if let Some(r) = jitter_rng.as_mut() {
        jitter_start(&mut s.w, r);
        s.replan_enemies();
    }
    if !spec.pouch.is_empty() {
        apply_pouch(&mut s.w, &spec.pouch);
    }
    let mut trace_rounds: Vec<Value> = Vec::new();
    let start = record.then(|| units_state(&s.w));
    if record {
        s.attach_metrics(BattleMetrics::new(), false);
    }
    let mut rounds = 0;
    let mut plan_fail = false;
    for round in 0..spec.cap {
        rounds = round + 1;
        // The tells while planning (INTENT1), before the party's plan is confirmed.
        let intents = record.then(|| crate::intent::trace(&s.w, &s.enemy_intents()));
        // RULESET1-G2: the auto driver's draft (a hold escape's holder by `pick_default`, then the preset)
        s.auto_draft(&spec.preset);
        if !s.confirm_plan() {
            if spec.jitter && plan_skip(&s) {
                return FightResult { skipped: true, ..FightResult::default() };
            }
            plan_fail = true;
            break;
        }
        let mut steps = Vec::new();
        let actions: Vec<Value> = if record {
            s.resolver.scheduled.iter().map(|a| json!([s.w.units[a.unit].name, a.command.name, a.start, a.end, a.command.ap_cost, a.command.stance_switch_cost])).collect()
        } else {
            Vec::new()
        };
        for _ in 0..MAX_STEPS_PER_ROUND {
            if s.phase == Phase::Replay {
                break;
            }
            s.step_next();
            s.metrics_step();
            if record {
                steps.push(json!({"block": s.current_block, "phase": s.phase as i32, "units": units_state(&s.w)}));
            }
        }
        if record {
            trace_rounds.push(json!({"round": s.current_round, "actions": actions, "intents": intents, "steps": steps}));
        }
        if s.combat_over || !s.can_advance_round() {
            break;
        }
        s.advance_round();
        s.metrics_round();
    }
    let l = side_hp_frac(&s.w, &s.w.allies);
    let r = side_hp_frac(&s.w, &s.w.enemies);
    let won = s.outcome == "won";
    s.metrics_end(EndResult { cleared: r <= 0.0, rounds, l, r, hp: if won { 1.0 } else { hp_score(l, r) } });
    let mut trace = record.then(|| json!({"start": start, "rounds": trace_rounds}));
    if let Some(t) = trace.as_mut() {
        merge(t, metrics_trace(&s));
    }
    FightResult {
        ok: true,
        plan_fail,
        skipped: false,
        l,
        r,
        bin: if won { 1.0 } else { binary_score(l, r) },
        hp: if won { 1.0 } else { hp_score(l, r) },
        rounds,
        outcome: s.final_outcome(),
        trace,
        metrics: s.metrics.take(),
    }
}

fn merge(t: &mut Value, more: Value) {
    if let (Some(o), Value::Object(m)) = (t.as_object_mut(), more) {
        o.extend(m);
    }
}

/// The trace's {"metrics", "report", "log"} of a finished fight whose metrics followed it.
pub fn metrics_trace(s: &Session) -> Value {
    let Some(m) = s.metrics.as_deref() else { return json!({}) };
    json!({"metrics": m.to_json(), "report": Report::build(m, s, &s.outcome).to_json(), "log": s.log.iter().map(|e| e.trace()).collect::<Vec<_>>()})
}

/// Equal as data: numbers within 1e-9, objects regardless of key order.
pub fn json_close(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => (x.as_f64().unwrap_or(0.0) - y.as_f64().unwrap_or(0.0)).abs() <= 1e-9,
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| json_close(p, q)),
        (Value::Object(x), Value::Object(y)) => x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| json_close(v, w))),
        _ => a == b,
    }
}

/// SimRunner.apply_pouch (ITEMUSE1 rule 9): every party unit's battle pouch becomes `ids` (catalog consumables,
/// unknown ids skipped) and its AI profile may use items. Enemies keep use_items off.
pub fn apply_pouch(w: &mut World, ids: &[String]) {
    for a in w.allies.clone() {
        w.units[a].inventory.pouch.clear();
        for id in ids {
            if let Some(item) = w.cat.items.get(id).filter(|i| i.is_consumable()).cloned() {
                w.units[a].inventory.add_to_pouch(&item);
            }
        }
        let mut profile = w.profile_of(a);
        profile.use_items = true;
        w.profiles.insert(a, profile);
    }
    w.add_item_skills();
    w.ignore_hazards_when_off();
}

/// SimRunner.is_reach_error / _plan_skip: every invalid ally plan failed because its target is out of reach.
pub fn plan_skip(s: &Session) -> bool {
    let mut any = false;
    for &a in &s.w.allies {
        if !s.w.units[a].is_active() {
            continue;
        }
        let st = s.plan.compile(&s.w, a);
        if st.error_index < 0 {
            continue;
        }
        let t = &st.error_text;
        if !(t.starts_with("Too far") || t.starts_with("No path") || t.contains(" is too far for ")) {
            return false;
        }
        any = true;
    }
    any
}
