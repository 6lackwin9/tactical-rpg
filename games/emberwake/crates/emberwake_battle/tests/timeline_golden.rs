//! RULESET1-X (docs/specs/RULESET1_TASKS.md 4.2): the timeline golden traces and the inert-extensions check.
//!
//! Golden: the five `tl_` encounters (RULESET1-A's timeline test fights) for seeds 1337, 7 and 42, each fought by
//! `sim::fight` with its authored party, board and round limit under the `timeline` ruleset row, on a frozen copy of
//! the catalog (`tests/fixtures/timeline_kit.json`), compared step for step with `tests/fixtures/timeline_golden.json`
//! (start, schedule, tells, every step, then outcome, rounds, log, metrics and report). The golden is the engine-only
//! reference of the timeline ruleset (section 5): it was recorded once when RULESET1-X landed and is re-recorded only for
//! an intended rule change, named in the commit and a docs/HISTORY.md row; never edited by hand. Re-record (the frozen
//! catalog is kept; delete `timeline_kit.json` first to re-freeze it from the shipped data):
//!
//! `RECORD=1 tools/fc.sh test --test timeline_golden`
//!
//! Inert extensions (4.2, RULESET1 3.2 "an extension is inert without data"): every Ashlands encounter played under
//! classic and under classic with every extension of the register switched on (options stay off) gives identical
//! traces, outcomes, events, logs, metrics, reports and RNG draw counts.
//!
//! The `tl_` fights are game data, so the test lives in the game's façade crate (the kit names no game).

use aegis_core::json;
use emberwake_battle::metrics::BattleMetrics;
use emberwake_battle::session::{EncounterDef, Phase, Session};
use emberwake_battle::sim::{fight, json_close, metrics_trace, FightSpec};
use emberwake_data::{Catalog, UnitDef};
use emberwake_rules::ruleset::{Ruleset, RuleKind, REGISTER, TIMELINE};
use emberwake_world::battle::WorldRoster;
use emberwake_world::Data;
use indexmap::IndexMap;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The timeline test encounters (RULESET1_TASKS 2.7) and the golden's seeds (4.2).
const ENCOUNTERS: [&str; 5] = ["tl_duel_windup", "tl_duel_reach", "tl_duel_armour", "tl_flank", "tl_skirmish"];
const SEEDS: [i64; 3] = [1337, 7, 42];
/// The party AI preset of the golden (the sim's default).
const PRESET: &str = "balanced";
/// The encounter fields a fight keeps (as the sim's `feel` mode does); party, enemies, round limit and seed are set apart.
const BOARD_KEYS: [&str; 11] = ["zone", "arena", "exits", "rows", "total_blocks", "unit_ap", "ap_regen", "deployment", "objective", "phases", "guests"];

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn data_dir() -> PathBuf {
    Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")
}

fn recording() -> bool {
    std::env::var("RECORD").is_ok_and(|v| v == "1")
}

/// The frozen catalog: read from the fixture; when recording without one, frozen first from the shipped data's loader
/// output (rows in file order, so the file is the same on every machine).
fn frozen_catalog() -> Value {
    let path = fixtures().join("timeline_kit.json");
    if recording() && !path.is_file() {
        let src = emberwake_content::source::load(&data_dir());
        assert!(src.errors.is_empty(), "the shipped data has load errors: {:?}", src.errors);
        std::fs::create_dir_all(fixtures()).unwrap();
        std::fs::write(&path, serde_json::to_string_pretty(&src.catalog).unwrap() + "\n").unwrap();
        eprintln!("froze the catalog to {}", path.display());
    }
    let text = std::fs::read_to_string(&path).expect("tests/fixtures/timeline_kit.json (frozen when RULESET1-X landed)");
    serde_json::from_str(&text).unwrap()
}

fn label(id: &str, seed: i64) -> String {
    format!("{id} seed {seed}")
}

/// Every fight of the golden, in order: (label, record).
fn golden_fights(raw: &Value) -> Vec<(String, Value)> {
    let cat = Arc::new(Catalog::from_value(raw));
    let ruleset = Ruleset::of(&cat, TIMELINE).expect("the frozen catalog's timeline row");
    let rows = json::aget(json::dict(raw), "encounters");
    let extras: IndexMap<String, UnitDef> = IndexMap::new();
    let mut out = Vec::new();
    for id in ENCOUNTERS {
        let entry = rows.iter().filter_map(Value::as_object).find(|e| json::s(e, "id") == id).unwrap_or_else(|| panic!("encounter {id} is missing from the frozen catalog"));
        let base: Map<String, Value> = BOARD_KEYS.iter().filter_map(|k| entry.get(*k).map(|v| (k.to_string(), v.clone()))).collect();
        for seed in SEEDS {
            let spec = FightSpec {
                left: json::sget(entry, "party"),
                right: json::sget(entry, "enemies"),
                seed,
                preset: PRESET.into(),
                cap: json::i(entry, "max_rounds", 8) as i32,
                jitter: false,
                base: Some(base.clone()),
                pouch: Vec::new(),
                ruleset: ruleset.clone(),
            };
            let r = fight(cat.clone(), &extras, &spec, true);
            assert!(r.ok, "{id} seed {seed}: the fight did not start");
            let mut rec = r.trace.unwrap_or(Value::Null);
            if let Some(o) = rec.as_object_mut() {
                o.insert("outcome".into(), json!(r.outcome));
                o.insert("rounds_played".into(), json!(r.rounds));
                o.insert("plan_fail".into(), json!(r.plan_fail));
            }
            out.push((label(id, seed), rec));
        }
    }
    out
}

/// The first difference between two records of a fight, as a readable line (`what` names the reference side).
fn first_divergence(what: &str, g: &Value, r: &Value) -> Option<String> {
    if g["start"] != r["start"] {
        return Some(format!("start:\n  {what} {}\n  now    {}", g["start"], r["start"]));
    }
    let empty = Vec::new();
    let (gr, rr) = (g["rounds"].as_array().unwrap_or(&empty), r["rounds"].as_array().unwrap_or(&empty));
    for (i, (a, b)) in gr.iter().zip(rr.iter()).enumerate() {
        for key in ["actions", "intents"] {
            if a[key] != b[key] {
                return Some(format!("round {} {key}:\n  {what} {}\n  now    {}", i + 1, a[key], b[key]));
            }
        }
        let (gs, rs) = (a["steps"].as_array().unwrap_or(&empty), b["steps"].as_array().unwrap_or(&empty));
        for (j, (x, y)) in gs.iter().zip(rs.iter()).enumerate() {
            if x != y {
                return Some(format!("round {} step {}:\n  {what} {}\n  now    {}", i + 1, j + 1, x, y));
            }
        }
        if gs.len() != rs.len() {
            return Some(format!("round {} step count: {what} {} now {}", i + 1, gs.len(), rs.len()));
        }
    }
    if gr.len() != rr.len() {
        return Some(format!("round count: {what} {} now {}", gr.len(), rr.len()));
    }
    // Numbers within 1e-9: serde_json reads some recorded decimals one unit in the last place off.
    for key in ["outcome", "rounds_played", "plan_fail", "events", "draws", "log", "metrics", "report"] {
        if !json_close(&g[key], &r[key]) {
            let (at, a, b) = deepest_difference(key.to_string(), &g[key], &r[key]);
            return Some(format!("{at}:\n  {what} {a}\n  now    {b}"));
        }
    }
    None
}

/// The path and the two values of the first difference inside `a` and `b` (objects by key, arrays by index), so a
/// large metrics or log value fails on one readable line.
fn deepest_difference(at: String, a: &Value, b: &Value) -> (String, Value, Value) {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            for k in x.keys().chain(y.keys().filter(|k| !x.contains_key(*k))) {
                let (u, v) = (x.get(k).unwrap_or(&Value::Null), y.get(k).unwrap_or(&Value::Null));
                if !json_close(u, v) {
                    return deepest_difference(format!("{at}.{k}"), u, v);
                }
            }
        }
        (Value::Array(x), Value::Array(y)) if x.len() == y.len() => {
            for (i, (u, v)) in x.iter().zip(y.iter()).enumerate() {
                if !json_close(u, v) {
                    return deepest_difference(format!("{at}[{i}]"), u, v);
                }
            }
        }
        _ => {}
    }
    (at, a.clone(), b.clone())
}

#[test]
fn timeline_fights_match_the_golden() {
    let raw = frozen_catalog();
    let now = golden_fights(&raw);
    let path = fixtures().join("timeline_golden.json");
    if recording() {
        let list: Vec<Value> = now.iter().map(|(l, t)| json!({"fight": l, "trace": t})).collect();
        // pretty-printed (keys in order) so a re-record shows a readable line diff
        let text = serde_json::to_string_pretty(&json!({"spec": "RULESET1_TASKS 4.2 (RULESET1-X)", "ruleset": TIMELINE, "preset": PRESET, "fights": list})).unwrap();
        std::fs::write(&path, text + "\n").unwrap();
        eprintln!("recorded {} fights to {}", now.len(), path.display());
        return;
    }
    let text = std::fs::read_to_string(&path).expect("tests/fixtures/timeline_golden.json (recorded when RULESET1-X landed)");
    let golden: Value = serde_json::from_str(&text).unwrap();
    let recorded = golden["fights"].as_array().unwrap();
    assert_eq!(recorded.len(), now.len(), "fight count");
    let mut failures = Vec::new();
    for (g, (l, r)) in recorded.iter().zip(now.iter()) {
        assert_eq!(g["fight"].as_str(), Some(l.as_str()), "fight order");
        if let Some(diff) = first_divergence("golden", &g["trace"], r) {
            failures.push(format!("{l}: {diff}"));
        }
    }
    eprintln!("timeline golden fights checked: {}, diverged: {}", now.len(), failures.len());
    assert!(failures.is_empty(), "{} of {} diverge from the timeline golden; first:\n{}", failures.len(), now.len(), failures.iter().take(3).cloned().collect::<Vec<_>>().join("\n"));
}

#[test]
fn the_golden_fights_are_timeline_fights_and_repeat() {
    if recording() {
        return; // the golden test freezes and records; this one reads the fixtures it writes
    }
    let raw = frozen_catalog();
    let cat = Catalog::from_value(&raw);
    let ruleset = Ruleset::of(&cat, TIMELINE).expect("timeline row");
    assert!(!ruleset.is_classic() && ruleset.has(emberwake_rules::ruleset::ROUND_INTENT), "the golden plays the timeline ruleset");
    // determinism (PL1): two runs of the same inputs are identical
    let (a, b) = (golden_fights(&raw), golden_fights(&raw));
    for ((l, x), (_, y)) in a.iter().zip(b.iter()) {
        assert_eq!(first_divergence("first", x, y), None, "{l} repeats");
    }
}

// ---- Inert extensions ---------------------------------------------------------------------------------------------

const INERT_WORLD: &str = "ashlands";
const INERT_SEEDS: [i64; 2] = [1, 2];

fn units_state(s: &Session) -> Value {
    let w = &s.w;
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

/// One world fight as world_traces plays it, under `ruleset`, with the RNG draws after every step.
fn world_fight(d: &Data, id: &str, seed: i64, ruleset: &Ruleset) -> Value {
    let entry = d.resolve(INERT_WORLD, "encounters", id).clone();
    let mut def = EncounterDef::from_obj(&entry);
    def.id = format!("{INERT_WORLD}:{id}");
    def.rng_seed = seed;
    def.ruleset = ruleset.clone();
    let roster = WorldRoster { d, world: INERT_WORLD.to_string() };
    let mut s = Session::start(d.cat.clone(), def, &roster);
    let start = units_state(&s);
    s.attach_metrics(BattleMetrics::new(), false);
    let mut rounds = Vec::new();
    for _ in 0..s.def.max_rounds {
        let intents = emberwake_battle::intent::trace(&s.w, &s.enemy_intents());
        s.plan_apply_preset_all(PRESET);
        if !s.confirm_plan() {
            break;
        }
        let actions: Vec<Value> = s.resolver.scheduled.iter().map(|a| json!([s.w.units[a.unit].name, a.command.name, a.start, a.end, a.command.ap_cost, a.command.stance_switch_cost])).collect();
        let mut steps = Vec::new();
        for _ in 0..600 {
            if s.phase == Phase::Replay {
                break;
            }
            s.step_next();
            s.metrics_step();
            steps.push(json!({"block": s.current_block, "phase": s.phase as i32, "units": units_state(&s), "draws": s.resolver.rng.draws}));
        }
        rounds.push(json!({"round": s.current_round, "actions": actions, "intents": intents, "steps": steps}));
        if s.combat_over || !s.can_advance_round() {
            break;
        }
        s.advance_round();
        s.metrics_round();
    }
    s.metrics_end(emberwake_battle::metrics::EndResult { rounds: s.current_round, ..Default::default() });
    let mut out = json!({"start": start, "rounds": rounds, "outcome": s.outcome, "events": s.events, "draws": s.resolver.rng.draws});
    if let (Some(o), Value::Object(m)) = (out.as_object_mut(), metrics_trace(&s)) {
        o.extend(m);
    }
    out
}

/// `plan_badges` is the one extension that needs no data (RULESET1 10, RULESET1-V): it appends columns to each tell
/// (the command's impact, the round intent, the impact badges). The classic columns must stay as they were, so each
/// tell of `extended` is cut to the length of the same tell under classic (its command too) before the comparison;
/// a tell the classic run does not have is left as it is (and so differs).
fn strip_badge_columns(classic: &Value, extended: &mut Value) {
    let empty = Vec::new();
    let rounds = classic["rounds"].as_array().unwrap_or(&empty);
    let Some(ext_rounds) = extended["rounds"].as_array_mut() else { return };
    for (c, e) in rounds.iter().zip(ext_rounds.iter_mut()) {
        let tells = c["intents"].as_array().unwrap_or(&empty);
        let Some(ext_tells) = e["intents"].as_array_mut() else { continue };
        for (ct, et) in tells.iter().zip(ext_tells.iter_mut()) {
            let (Some(ct), Some(et)) = (ct.as_array(), et.as_array_mut()) else { continue };
            et.truncate(ct.len());
            if let (Some(Value::Array(cc)), Some(Value::Array(ec))) = (ct.get(1), et.get_mut(1)) {
                ec.truncate(cc.len());
            }
        }
    }
}

/// Whether a readout says nothing happened: every number 0, every string empty, every list and object quiet.
fn quiet(v: &Value) -> bool {
    match v {
        Value::Null | Value::Bool(false) => true,
        Value::Number(n) => n.as_f64() == Some(0.0),
        Value::String(s) => s.is_empty(),
        Value::Array(a) => a.iter().all(quiet),
        Value::Object(o) => o.values().all(quiet),
        Value::Bool(true) => false,
    }
}

/// An extension switched on may add its own readout (the `poise` metrics of RULESET1-P), but without data it must
/// count nothing: the object keys of `extended` absent under classic are removed when quiet and listed as loud
/// otherwise (`at` names them); the rest is compared as it is.
fn strip_quiet_readouts(at: &str, classic: &Value, extended: &mut Value, loud: &mut Vec<String>) {
    let (Value::Object(c), Value::Object(e)) = (classic, extended) else { return };
    let added: Vec<String> = e.keys().filter(|k| !c.contains_key(*k)).cloned().collect();
    for k in added {
        if quiet(&e[&k]) {
            e.shift_remove(&k);
        } else {
            loud.push(format!("{at}.{k} = {}", e[&k]));
        }
    }
    for (k, v) in e.iter_mut() {
        if let Some(cv) = c.get(k) {
            strip_quiet_readouts(&format!("{at}.{k}"), cv, v, loud);
        }
    }
}

#[test]
fn every_extension_is_inert_on_the_ashlands_fights() {
    let d = Data::open(&data_dir()).expect("shipped data").0;
    assert!(d.ruleset(INERT_WORLD).is_classic(), "the Ashlands play classic");
    let extensions: Vec<&str> = REGISTER.iter().filter(|r| r.kind == RuleKind::Extension).map(|r| r.id).collect();
    let mut all_on = Ruleset::classic();
    all_on.adjust(&extensions.iter().map(|id| format!("+{id}")).collect::<Vec<_>>().join(",")).expect("extensions switch on over classic");
    assert_eq!(all_on.rules, extensions, "every extension on, no option");
    let ids: Vec<String> = d.own_entries(INERT_WORLD, "encounters").iter().map(|e| json::s(e, "id")).collect();
    assert!(!ids.is_empty(), "the Ashlands have encounters");
    let mut failures = Vec::new();
    let mut fights = 0;
    for id in &ids {
        for seed in INERT_SEEDS {
            let classic = world_fight(&d, id, seed, &Ruleset::classic());
            let mut extended = world_fight(&d, id, seed, &all_on);
            strip_badge_columns(&classic, &mut extended);
            let mut loud = Vec::new();
            for key in ["metrics", "report"] {
                strip_quiet_readouts(key, &classic[key], &mut extended[key], &mut loud);
            }
            if !loud.is_empty() {
                failures.push(format!("{INERT_WORLD}:{id} seed {seed}: an extension readout counts something: {}", loud.join("; ")));
            }
            fights += 1;
            if let Some(diff) = first_divergence("classic", &classic, &extended) {
                failures.push(format!("{INERT_WORLD}:{id} seed {seed}: {diff}"));
            }
        }
    }
    // the comparison sees a rule change: the same fight under the timeline row (its options on) differs
    let timeline = Ruleset::of(&d.cat, TIMELINE).expect("the shipped timeline row");
    let (classic, changed) = (world_fight(&d, &ids[0], INERT_SEEDS[0], &Ruleset::classic()), world_fight(&d, &ids[0], INERT_SEEDS[0], &timeline));
    assert!(first_divergence("classic", &classic, &changed).is_some(), "{INERT_WORLD}:{} under timeline differs from classic", ids[0]);
    eprintln!("inert extensions: {fights} Ashlands fights checked with [{}], diverged: {}", extensions.join(", "), failures.len());
    assert!(failures.is_empty(), "{} of {fights} change with every extension on; first:\n{}", failures.len(), failures.iter().take(3).cloned().collect::<Vec<_>>().join("\n"));
}
