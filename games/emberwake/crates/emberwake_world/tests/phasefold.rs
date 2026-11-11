//! PHASEFOLD (games/emberwake/docs/specs/PHASEFOLD.md section 4): the single reinforcement wave plays exactly as it did
//! before it became an arrival phase. The golden `fixtures/phasefold_golden.json` was recorded once with the engine
//! before the change (`RECORD=1 cargo test -p emberwake_world --test phasefold`); re-recorded only for an intended
//! data change, named in docs/HISTORY.md (2026-10-27, WAR2H1: Grakka's battle axe changed ashmaw_stand; 2026-11-10,
//! PR5-18 WEAPONROSTER1: the class lineup and default loadouts changed 60 of the 72 fights): the shipped fights
//! with a wave (goblin_ambush in the catalog, pass_ambush and ashmaw_stand in ashlands), morale off and on, four presets,
//! three seeds. Each fight is replayed and compared step for step (unit states, schedule, intents), then the outcome,
//! the log, metrics and report, the events, the fates and the tally keys. Then the
//! arrival checks (1.2, 1.3) and the win and morale rules (2).

use aegis_core::json::{self, Dict};
use emberwake_battle::encounter::{arrival_phase, ARRIVAL_LABEL};
use emberwake_battle::metrics::{BattleMetrics, EndResult};
use emberwake_battle::session::{EncounterDef, Phase, Session};
use emberwake_battle::sim::{json_close, metrics_trace};
use emberwake_content::{check, store};
use emberwake_data::UnitDef;
use emberwake_world::battle::{self, Tally, WorldRoster};
use emberwake_world::Data;
use indexmap::IndexMap;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const FIGHTS: [(&str, &str); 3] = [("", "goblin_ambush"), ("ashlands", "pass_ambush"), ("ashlands", "ashmaw_stand")];
const PRESETS: [&str; 4] = ["balanced", "aggressive", "defensive", "withdraw"];

fn data_dir() -> PathBuf {
    Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")
}

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/phasefold_golden.json")
}

fn overrides() -> [Value; 2] {
    [json!({}), json!({"morale": true, "morale_break": 70})]
}

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

fn phase_int(p: Phase) -> i64 {
    match p {
        Phase::Planning => 0,
        Phase::Resolving => 1,
        Phase::Replay => 2,
    }
}

/// One fight played as world_traces plays it, plus what PHASEFOLD must keep (events, fates, tally keys, roster keys).
fn run(d: &Data, world: &str, id: &str, over: &Value, preset: &str, seed: i64) -> Value {
    let mut entry = if world.is_empty() { d.cat.encounters.get(id).cloned().unwrap_or_default() } else { d.resolve(world, "encounters", id).clone() };
    assert!(!entry.is_empty(), "encounter {world}:{id} is missing from the shipped data");
    for (k, v) in json::dict(over) {
        entry.insert(k.clone(), v.clone());
    }
    let mut def = EncounterDef::from_obj(&entry);
    if !world.is_empty() {
        def.id = format!("{world}:{id}");
    }
    def.rng_seed = seed;
    let extras: IndexMap<String, UnitDef> = IndexMap::new();
    let roster = WorldRoster::new(d, world);
    let mut s = if world.is_empty() { Session::start(d.cat.clone(), def, &extras) } else { Session::start(d.cat.clone(), def, &roster) };
    let start = units_state(&s);
    s.attach_metrics(BattleMetrics::new(), false);
    let mut rounds = Vec::new();
    for _ in 0..s.def.max_rounds {
        let intents = emberwake_battle::intent::trace(&s.w, &s.enemy_intents());
        s.plan_apply_preset_all(preset);
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
            steps.push(json!({"block": s.current_block, "phase": phase_int(s.phase), "units": units_state(&s)}));
        }
        rounds.push(json!({"round": s.current_round, "actions": actions, "intents": intents, "steps": steps}));
        if s.combat_over || !s.can_advance_round() {
            break;
        }
        s.advance_round();
        s.metrics_round();
    }
    s.metrics_end(EndResult { rounds: s.current_round, ..EndResult::default() });
    let fates: Vec<Value> = s.fates().into_iter().map(|(r, f)| json!([r, f.name()])).collect();
    let tally: Vec<Value> = Tally::from_session(&s).foes.into_iter().map(|f| json!([f.key, f.fate.name()])).collect();
    let roster_keys: Vec<String> = if world.is_empty() { Vec::new() } else { battle::roster(d, world, id).into_iter().map(|f| f.key).collect() };
    let mut out = json!({"start": start, "rounds": rounds, "outcome": s.outcome, "events": s.events, "fates": fates, "tally": tally, "roster": roster_keys});
    if let (Some(o), Value::Object(m)) = (out.as_object_mut(), metrics_trace(&s)) {
        o.extend(m);
    }
    out
}

fn label(world: &str, id: &str, over: &Value, preset: &str, seed: i64) -> String {
    format!("{}:{id} {over} {preset} seed {seed}", if world.is_empty() { "catalog" } else { world })
}

/// Every fight of the golden, in order: (label, trace).
fn fights(d: &Data) -> Vec<(String, Value)> {
    let mut out = Vec::new();
    for (world, id) in FIGHTS {
        for over in overrides() {
            for preset in PRESETS {
                for seed in 1..=3 {
                    out.push((label(world, id, &over, preset, seed), run(d, world, id, &over, preset, seed)));
                }
            }
        }
    }
    out
}

/// The first difference between the golden fight and the replay, as a readable line.
fn first_divergence(g: &Value, r: &Value) -> Option<String> {
    if g["start"] != r["start"] {
        return Some(format!("start:\n  golden {}\n  now    {}", g["start"], r["start"]));
    }
    let (gr, rr) = (g["rounds"].as_array().unwrap(), r["rounds"].as_array().unwrap());
    for (i, (a, b)) in gr.iter().zip(rr.iter()).enumerate() {
        for key in ["actions", "intents"] {
            if a[key] != b[key] {
                return Some(format!("round {} {key}:\n  golden {}\n  now    {}", i + 1, a[key], b[key]));
            }
        }
        let (gs, rs) = (a["steps"].as_array().unwrap(), b["steps"].as_array().unwrap());
        for (j, (x, y)) in gs.iter().zip(rs.iter()).enumerate() {
            if x != y {
                return Some(format!("round {} step {}:\n  golden {}\n  now    {}", i + 1, j + 1, x, y));
            }
        }
        if gs.len() != rs.len() {
            return Some(format!("round {} step count: golden {} now {}", i + 1, gs.len(), rs.len()));
        }
    }
    if gr.len() != rr.len() {
        return Some(format!("round count: golden {} now {}", gr.len(), rr.len()));
    }
    // Numbers within 1e-9: serde_json reads some recorded decimals one unit in the last place off.
    for key in ["outcome", "log", "events", "fates", "tally", "roster", "metrics", "report"] {
        if !json_close(&g[key], &r[key]) {
            return Some(format!("{key}:\n  golden {}\n  now    {}", g[key], r[key]));
        }
    }
    None
}

#[test]
fn wave_fights_match_the_golden_recorded_before_the_fold() {
    let d = Data::open(&data_dir()).expect("shipped data").0;
    let now = fights(&d);
    if std::env::var("RECORD").is_ok_and(|v| v == "1") {
        let list: Vec<Value> = now.iter().map(|(l, t)| json!({"fight": l, "trace": t})).collect();
        let text = serde_json::to_string(&json!({"spec": "PHASEFOLD section 4", "fights": list})).unwrap();
        std::fs::create_dir_all(golden_path().parent().unwrap()).unwrap();
        std::fs::write(golden_path(), text + "\n").unwrap();
        eprintln!("recorded {} fights to {}", now.len(), golden_path().display());
        return;
    }
    let text = std::fs::read_to_string(golden_path()).expect("tests/fixtures/phasefold_golden.json (recorded once before PHASEFOLD)");
    let golden: Value = serde_json::from_str(&text).unwrap();
    let recorded = golden["fights"].as_array().unwrap();
    assert_eq!(recorded.len(), now.len(), "fight count");
    let mut failures = Vec::new();
    for (g, (l, r)) in recorded.iter().zip(now.iter()) {
        assert_eq!(g["fight"].as_str(), Some(l.as_str()), "fight order");
        if let Some(diff) = first_divergence(&g["trace"], r) {
            failures.push(format!("{l}: {diff}"));
        }
    }
    eprintln!("phasefold golden fights checked: {}, diverged: {}", now.len(), failures.len());
    assert!(failures.is_empty(), "{} of {} diverge; first:\n{}", failures.len(), now.len(), failures.iter().take(3).cloned().collect::<Vec<_>>().join("\n"));
}

// ---- 1.4 the fold -------------------------------------------------------------------------------------------------

fn obj(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

fn strs(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

// ---- 1.2, 1.3 the checks ------------------------------------------------------------------------------------------

/// The shipped catalog's first party-side unit and first two enemy-side units (picked by rule, read only).
fn units_by_side(d: &Data) -> (String, String, String) {
    let side = |s: &str| d.cat.units.values().filter(|u| u.side == s).map(|u| u.id.clone()).collect::<Vec<_>>();
    let (party, enemies) = (side("party"), side("enemy"));
    assert!(!party.is_empty() && enemies.len() >= 2, "the catalog has party and enemy units");
    (party[0].clone(), enemies[0].clone(), enemies[1].clone())
}

fn arrival_with(units: &[String], trigger: Value) -> Value {
    let mut a = arrival_phase(units, 1, 4, ARRIVAL_LABEL);
    a["trigger"] = trigger;
    a
}

fn round_phase(id: &str, round: i64) -> Value {
    json!({"id": id, "name": id, "trigger": {"kind": "round", "round": round}, "effects": [{"type": "line", "text": "Now."}]})
}

#[test]
fn arrival_checks_keep_the_wave_messages() {
    let d = Data::open(&data_dir()).unwrap().0;
    let view = store::view(&data_dir());
    let (p, e1, e2) = units_by_side(&d);
    let base = obj(json!({"id": "test_arrival", "name": "Test Arrival", "party": [p], "enemies": [e1], "max_rounds": 6, "total_blocks": 6}));
    let errors = |phases: Value| {
        let mut e = base.clone();
        e.insert("phases".into(), phases);
        check::validate_entry(&view, "encounters", &e, "")
    };
    assert_eq!(errors(json!([])), Vec::<String>::new(), "the base row is clean");
    let ok = arrival_phase(std::slice::from_ref(&e2), 2, 1, "Here!");
    assert_eq!(errors(json!([ok])), Vec::<String>::new());

    // 1.2: past the last round or block, the old field messages; below 1, the phase messages
    assert_eq!(errors(json!([arrival_with(std::slice::from_ref(&e2), json!({"kind": "arrival", "round": 7, "block": 1}))])), ["The wave round is after the last round."]);
    assert_eq!(errors(json!([arrival_with(std::slice::from_ref(&e2), json!({"kind": "arrival", "round": 1, "block": 7}))])), ["The wave block is past the end of the round (6 blocks)."]);
    assert_eq!(errors(json!([arrival_with(std::slice::from_ref(&e2), json!({"kind": "arrival", "round": 0, "block": 0}))])), ["Phase 'reinforcements': round must be 1..6.", "Phase 'reinforcements': block must be 1..6."]);
    assert_eq!(errors(json!([arrival_with(std::slice::from_ref(&e2), json!({"kind": "arrival", "pct": 50}))])), ["Phase 'reinforcements': unknown trigger key 'pct'."]);
    // 1.3: the shape of an arrival
    let mut two = arrival_phase(std::slice::from_ref(&e2), 1, 4, "x");
    two["effects"].as_array_mut().unwrap().push(json!({"type": "line", "text": "Hi."}));
    assert_eq!(errors(json!([two])), ["Phase 'reinforcements': an arrival holds one wave and nothing else."]);
    let mut second = arrival_phase(std::slice::from_ref(&e2), 1, 4, "x");
    second["id"] = json!("more");
    assert_eq!(errors(json!([ok, second])), ["Phase 'more': the arrival comes first in the phase list.", "An encounter holds at most one arrival."]);
    assert_eq!(errors(json!([round_phase("first", 2), ok])), ["Phase 'reinforcements': the arrival comes first in the phase list."]);
    let mut pausing = ok.clone();
    pausing["pause_autoplay"] = json!(true);
    assert_eq!(errors(json!([pausing])), ["Phase 'reinforcements': an arrival never pauses autoplay."]);
    let mut label = ok.clone();
    label["effects"][0]["label"] = json!(3);
    assert_eq!(errors(json!([label])), ["Phase 'reinforcements': the wave label must be text."]);
    // the refs: the retired field's messages at its place, never "wave unit ... is not an enemy"
    assert_eq!(errors(json!([arrival_phase(&strs(&["zz_missing"]), 1, 4, "x")])), ["Reinforcements: unknown unit 'zz_missing'."]);
    assert_eq!(errors(json!([arrival_phase(std::slice::from_ref(&p), 1, 4, "x")])), [format!("Reinforcements: '{p}' can't be used here.")]);
    // four boss phases and the arrival: clean (the arrival is not counted); a fifth boss phase is one too many
    let four: Vec<Value> = std::iter::once(ok.clone()).chain((2..=5).map(|r| round_phase(&format!("p{r}"), r))).collect();
    assert_eq!(errors(Value::Array(four.clone())), Vec::<String>::new());
    let mut five = four;
    five.push(round_phase("p6", 6));
    assert_eq!(errors(Value::Array(five)), ["An encounter holds at most 4 phases."]);
    // the unit cap counts the arrival with the enemies
    let crowd = vec![e2.clone(); 12];
    assert_eq!(errors(json!([arrival_phase(&crowd, 1, 4, "x")])), ["Encounter enemies, waves and phase waves hold at most 12 units."]);
}

#[test]
fn world_arrival_refs_read_as_the_wave_field() {
    let view = store::view(&data_dir());
    let clean = view.worlds.own_entries("ashlands", "encounters").iter().find(|e| json::aget(e, "phases").is_empty() && check::world::validate_entry(&view, "ashlands", "encounters", e, &json::s(e, "id")).is_empty()).cloned().expect("a clean ashlands encounter without phases");
    let mut e: Dict = clean.iter().filter(|(k, _)| !k.starts_with('_')).map(|(k, v)| (k.clone(), v.clone())).collect();
    e.insert("phases".into(), json!([arrival_phase(&strs(&["zz_missing"]), 1, 4, "x")]));
    let errors = check::world::validate_entry(&view, "ashlands", "encounters", &e, &json::s(&e, "id"));
    assert_eq!(errors, ["Reinforcements: unknown character 'zz_missing'."]);
}

// ---- 2 the battle -------------------------------------------------------------------------------------------------

/// A catalog fight: `party` vs `enemy`, with `phases`; the deployed enemy is down before the first step.
fn fight_with_the_enemy_down(d: &Data, p: &str, enemy: &str, phases: Value, morale: bool) -> Session {
    let entry = obj(json!({"id": "test_arrival", "name": "Test Arrival", "party": [p], "enemies": [enemy], "phases": phases, "morale": morale}));
    let mut s = Session::start(d.cat.clone(), EncounterDef::from_obj(&entry), &IndexMap::<String, UnitDef>::new());
    let first = s.w.enemies[0];
    s.w.units[first].hp = 0;
    s.replan_enemies();
    s
}

fn play_round(s: &mut Session) {
    s.plan_apply_preset_all("defensive");
    assert!(s.confirm_plan());
    for _ in 0..600 {
        if s.phase == Phase::Replay {
            break;
        }
        s.step_next();
    }
}

#[test]
fn an_arrival_counts_as_a_foe_before_it_comes_and_a_phase_wave_does_not() {
    let d = Data::open(&data_dir()).unwrap().0;
    let (p, e1, e2) = units_by_side(&d);
    // W8: every deployed enemy is down in round 1, the arrival enters at round 2 block 1: no win yet
    let mut s = fight_with_the_enemy_down(&d, &p, &e1, json!([arrival_phase(std::slice::from_ref(&e2), 2, 1, "Here!")]), false);
    let arrival = s.phases.units_of("reinforcements");
    assert_eq!(arrival.len(), 1);
    assert!(!s.is_reserved(arrival[0]), "an arrival unit is never reserved");
    play_round(&mut s);
    assert_eq!(s.outcome, "", "not won while the arrival is still to come");
    assert!(s.w.grid.positions[arrival[0]].is_none(), "still off the field");
    // round 2, block 1: the arrival enters with the wave's cell and line, then the fight goes on until it is down
    s.advance_round();
    s.plan_apply_preset_all("defensive");
    assert!(s.confirm_plan());
    s.step_next();
    assert!(s.w.grid.positions[arrival[0]].is_some(), "on the field");
    let cell = s.log.last().unwrap();
    assert_eq!((cell.round, cell.block, cell.actor.as_str(), cell.skill.as_str(), cell.subtype.as_str(), cell.outcome.as_str()), (2, 1, "Reinforcements", "Here!", "Wave", "ARRIVED"));
    let name = s.w.units[arrival[0]].name.clone();
    assert_eq!(s.events.last().unwrap(), &format!("Here!: {name} join the battle."));
    assert_eq!(s.phases.fired.get("reinforcements"), Some(&(2, 1)));
    s.w.units[arrival[0]].hp = 0;
    for _ in 0..600 {
        if s.phase == Phase::Replay {
            break;
        }
        s.step_next();
    }
    assert_eq!(s.outcome, "won");

    // BOSS1 rule 4 unchanged: the same wave as a round phase is held back, so the fight is won in round 1
    let late = json!({"id": "late", "name": "Late", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "wave", "units": [e2]}]});
    let mut s = fight_with_the_enemy_down(&d, &p, &e1, json!([late]), false);
    assert!(s.is_reserved(s.phases.units_of("late")[0]));
    play_round(&mut s);
    assert_eq!(s.outcome, "won");
}

#[test]
fn an_arriving_wave_forms_the_morale_group_wave() {
    let d = Data::open(&data_dir()).unwrap().0;
    let (p, e1, e2) = units_by_side(&d);
    let entry = obj(json!({"id": "test_arrival", "name": "Test Arrival", "party": [p], "enemies": [e1], "phases": [arrival_phase(&[e2.clone(), e2.clone()], 1, 1, "Here!")], "morale": true}));
    let mut s = Session::start(d.cat.clone(), EncounterDef::from_obj(&entry), &IndexMap::<String, UnitDef>::new());
    assert!(!s.morale.has_group("wave"));
    s.plan_apply_preset_all("defensive");
    assert!(s.confirm_plan());
    s.step_next();
    let placed = s.phases.units_of("reinforcements").iter().filter(|&&c| s.w.grid.positions[c].is_some()).count();
    assert_eq!(placed, 2);
    let group = s.morale.groups.iter().find(|g| g.source == "wave").expect("a wave group");
    assert_eq!(group.size as usize, placed);
    assert_eq!(group.members, [format!("{e2}#0"), format!("{e2}#1")]);
}
