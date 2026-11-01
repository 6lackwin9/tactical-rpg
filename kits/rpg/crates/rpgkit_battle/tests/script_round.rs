//! ENCOUNTER2-C3 on a scratch catalog built here (no shipped data): scripted rounds (docs/specs/ENCOUNTER2_C.md 4).
//! Expectations come from the rules: a scripted round is planned at its start by the same picks a player makes (a
//! twin battle without the script, given the same picks by hand, schedules and plays the same round: SR2, SR4); a
//! locked round is confirmed at once and the auto driver leaves it alone (SR5, SR7); a guided round fixes the enemies'
//! entries and hands the party's to the player as picks (SR6); `fill` decides the blocks after the picks and the
//! Auto-fill setting is not read (SR3); an entry that cannot be planned gives one `Script` cell and its unit plays its
//! own round, which is the round the twin plays without that entry (SR11); classic ignores the script with one cell
//! (SR8); unit names `ref#n`, `party:<id>`, `party#n` (SN1); the metrics count the rounds and fallbacks (SR9).
//!
//! Every battle plays round 1 unscripted (the auto driver), then the test places the units and starts round 2, the
//! scripted one, so the script plans on a board the test knows.

use indexmap::IndexMap;
use rpgkit_battle::grid::V2;
use rpgkit_battle::metrics::{BattleMetrics, EndResult};
use rpgkit_battle::round_intent::RoundTarget;
use rpgkit_battle::round_intent_plan::{self as rip, Pick};
use rpgkit_battle::script::{self, CLASSIC_LINE};
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, AP_COST, ROUND_INTENT, TIMELINE};
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const BASICS: [&str; 4] = ["move", "rest", "strike", "active_guard"];

/// Roster order: the party (Vanguard, Archer), then the enemies (two footmen, a brute).
const VANGUARD: UnitId = 0;
const ARCHER: UnitId = 1;
const FOOTMAN: UnitId = 2;
const FOOTMAN_B: UnitId = 3;
const BRUTE: UnitId = 4;

fn catalog() -> Arc<Catalog> {
    let atk = |id: &str, intent: &str, blocks: i32, ap: i32, level: i32| {
        json!({"id": id, "name": id, "kind": "ATTACK", "round_intent": intent, "level": level, "blocks": blocks, "recovery_blocks": 1, "ap_cost": ap, "power_mult": 0.5, "target_mode": "ENEMY"})
    };
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "active_guard", "name": "Active Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 1, "mitigation": 0.5}),
        atk("tl_heavy_cleave", "assault", 3, 3, 1),
        atk("tl_shoulder_check", "assault", 1, 1, 1),
        // more AP than any unit has
        atk("tl_costly", "assault", 1, 30, 1),
        // above every unit's skill cap
        atk("tl_master", "assault", 1, 1, 6),
    ];
    let unit = |id: &str, side: &str, range: i32| {
        json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 900, "speed": 12, "attack_range": range, "skill_cap": 5,
            "deck": {"order": {"assault": ["tl_heavy_cleave", "tl_shoulder_check", "strike"], "guard": ["active_guard"]}}})
    };
    let units = vec![unit("tl_vanguard", "party", 1), unit("tl_archer", "party", 3), unit("tl_footman", "enemy", 1), unit("tl_brute", "enemy", 1)];
    Arc::new(Catalog::from_value(&json!({"skills": skills, "units": units})))
}

fn timeline() -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, AP_COST], "basics": BASICS}).as_object().unwrap())
}

/// The fight with `script` and `extra` merged over the row.
fn row(script: Value, extra: Value) -> Value {
    let mut row = json!({"id": "t_scripted", "name": "Scripted", "rows": vec![".".repeat(12); 9], "party": ["tl_vanguard", "tl_archer"],
        "enemies": ["tl_footman", "tl_footman", "tl_brute"], "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 4, "max_rounds": 4, "rng_seed": 1337, "script": script});
    for (k, v) in extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    row
}

fn start(row: &Value, ruleset: Ruleset) -> Session {
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    Session::start(catalog(), def, &IndexMap::new())
}

fn place(w: &mut World, c: UnitId, cell: V2) {
    w.grid.positions[c] = Some(cell);
}

/// Round 1 by the auto driver (as autoplay and the sims play it), then every unit on its round-2 tile.
fn after_round_one(s: &mut Session) {
    s.auto_draft("balanced");
    assert!(s.confirm_plan());
    s.resolve_round();
    assert_eq!(s.phase, Phase::Replay);
    let tiles = [V2::new(3, 4), V2::new(1, 2), V2::new(5, 4), V2::new(6, 6), V2::new(9, 1)];
    for (c, cell) in tiles.into_iter().enumerate() {
        place(&mut s.w, c, cell);
    }
}

/// Starts the battle and plays to the start of round 2 (where the scripts of these tests are).
fn at_round_two(row: &Value, ruleset: Ruleset) -> Session {
    let mut s = start(row, ruleset);
    after_round_one(&mut s);
    s.advance_round();
    assert_eq!(s.current_round, 2);
    s
}

/// The round's schedule as (unit, command, start, end).
fn schedule(s: &Session) -> Vec<(UnitId, String, i32, i32)> {
    s.resolver.scheduled.iter().map(|a| (a.unit, a.command.name.clone(), a.start, a.end)).collect()
}

fn of(s: &Session, c: UnitId) -> Vec<String> {
    s.resolver.scheduled.iter().filter(|a| a.unit == c).map(|a| a.command.name.clone()).collect()
}

/// The `Script` cells' lines.
fn script_cells(s: &Session) -> Vec<String> {
    s.log.iter().filter(|e| e.category == "EVENT" && e.subtype == rpgkit_battle::log::SCRIPT).map(|e| e.detail.clone()).collect()
}

fn hp(s: &Session) -> Vec<i32> {
    s.w.units.iter().map(|u| u.hp).collect()
}

fn entry(unit: &str, target: Value, picks: Value, fill: &str) -> Value {
    json!({"unit": unit, "target": target, "picks": picks, "fill": fill})
}

fn round_two(lock: bool, others: &str, entries: Value) -> Value {
    json!([{"round": 2, "lock": lock, "others": others, "banner": "Watch", "entries": entries}])
}

fn sid(s: &Session, id: &str) -> usize {
    s.w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

/// SR2, SR4, SR5: a locked round is planned by its party entries (the same pick a player makes) and the others' auto
/// plan, then confirmed at its start; its schedule and the round it plays are the twin's, where the same pick is made
/// by hand and the others drafted by the auto driver.
#[test]
fn a_locked_round_plays_the_picks_a_player_would_make() {
    let script = round_two(true, "auto", json!([entry("party:tl_vanguard", json!("tl_footman"), json!([{"skill": "tl_shoulder_check"}]), "auto")]));
    let mut s = at_round_two(&row(script, json!({})), timeline());
    assert!(s.round_locked() && s.plan_confirmed, "confirmed at the round start");
    assert_eq!(s.round_script().map(|r| (r.round, r.lock, r.banner.as_str())), Some((2, true, "Watch")));
    assert_eq!(rip::picks_of(&s.w, VANGUARD), vec![Pick::Skill { skill: sid(&s, "tl_shoulder_check"), kind: None }]);
    assert!(script_cells(&s).is_empty(), "{:?}", script_cells(&s));

    let mut twin = at_round_two(&row(json!([]), json!({})), timeline());
    assert!(!twin.round_locked() && !twin.plan_confirmed);
    let shoulder = sid(&twin, "tl_shoulder_check");
    assert_eq!(twin.pick(VANGUARD, RoundTarget::Unit(FOOTMAN), &[Pick::Skill { skill: shoulder, kind: None }]).0, Ok(()));
    twin.auto_draft("balanced");
    assert!(twin.confirm_plan());
    assert_eq!(schedule(&s), schedule(&twin), "the same schedule");
    assert!(of(&s, VANGUARD).iter().any(|n| n == "tl_shoulder_check"), "{:?}", of(&s, VANGUARD));

    // the auto driver (autoplay, the sims) leaves a locked round alone: its draft and confirm change nothing
    let before = schedule(&s);
    assert_eq!(s.auto_draft("aggressive"), None);
    assert!(s.confirm_plan());
    assert_eq!(schedule(&s), before);
    s.resolve_round();
    twin.resolve_round();
    assert_eq!(s.log_lines(0, 0), twin.log_lines(0, 0), "the same round");
    assert_eq!(hp(&s), hp(&twin));
    assert_eq!((s.scripted_rounds(), s.script_fallbacks()), (1, 0));
    // the next round is not scripted
    s.advance_round();
    assert!(!s.round_locked() && !s.plan_confirmed && s.round_script().is_none());
}

/// SR6: a guided round fixes the enemies' entries and hands the party's to the player as first picks (kept, changed or
/// undone; the player confirms).
#[test]
fn a_guided_round_fixes_the_enemies_and_hands_the_party_its_picks() {
    let script = round_two(
        false,
        "auto",
        json!([entry("tl_footman#2", json!("self"), json!([{"skill": "active_guard"}]), "rest"), entry("party:tl_vanguard", json!("tl_footman"), json!([{"skill": "tl_shoulder_check"}]), "auto")]),
    );
    let mut s = at_round_two(&row(script, json!({})), timeline());
    assert!(!s.round_locked() && !s.plan_confirmed && s.is_planning(), "the player plans");
    // the enemy's entry, then Rest on every block after it (fill rest)
    let footman_b = of(&s, FOOTMAN_B);
    assert_eq!(footman_b.first().map(String::as_str), Some("Active Guard"), "{footman_b:?}");
    assert!(footman_b.len() > 1 && footman_b[1..].iter().all(|n| n == "Rest"), "{footman_b:?}");
    assert_eq!(rip::picks_of(&s.w, VANGUARD).len(), 1);
    let (undone, _) = rip::unpick(&mut s.w, &mut s.plan, VANGUARD);
    assert_eq!(undone, Ok(()));
    assert!(rip::picks_of(&s.w, VANGUARD).is_empty(), "the player may undo it");
    assert!(s.confirm_plan(), "and confirms as usual");
    assert_eq!(of(&s, FOOTMAN_B).first().map(String::as_str), Some("Active Guard"), "the enemy's entry stays");
    assert_eq!(s.scripted_rounds(), 1);
}

/// SR3: `fill` rest leaves only Rest after the picks; `fill` auto is the planner's fill; the player's Auto-fill
/// setting changes neither.
#[test]
fn fill_decides_the_blocks_after_the_picks_and_auto_fill_is_not_read() {
    let entries = |fill: &str| json!([entry("party:tl_vanguard", json!("tl_footman"), json!([{"skill": "tl_shoulder_check"}]), fill), entry("party:tl_archer", json!("self"), json!([{"skill": "active_guard"}]), "auto")]);
    let rested = at_round_two(&row(round_two(true, "auto", entries("rest")), json!({})), timeline());
    let names = of(&rested, VANGUARD);
    let at = names.iter().position(|n| n == "tl_shoulder_check").expect("the pick");
    assert!(names.len() > at + 1 && names[at + 1..].iter().all(|n| n == "Rest"), "{names:?}");
    let filled = at_round_two(&row(round_two(true, "auto", entries("auto")), json!({})), timeline());
    assert!(of(&filled, VANGUARD).iter().skip(at + 1).any(|n| n != "Rest"), "the planner fills: {:?}", of(&filled, VANGUARD));
    // Auto-fill off (set after round 1) changes neither
    for fill in ["rest", "auto"] {
        let mut s = start(&row(round_two(true, "auto", entries(fill)), json!({})), timeline());
        after_round_one(&mut s);
        rip::set_autofill(&mut s.w, &mut s.plan, false);
        s.advance_round();
        let on = if fill == "rest" { &rested } else { &filled };
        assert_eq!(schedule(&s), schedule(on), "fill {fill}, Auto-fill off");
    }
}

/// SR11: an entry whose target is down, whose family is not the unit's, or whose pick it cannot pay gives one
/// `Script` cell and its unit plays its own round (the twin's round without the script); the round stays locked.
#[test]
fn a_failed_entry_logs_one_cell_and_its_unit_plays_its_own_round() {
    // the brute falls at round 2's start (a phase kill, before planning)
    let kill = json!({"phases": [{"id": "fall", "name": "Fall", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "kill", "units": ["tl_brute"]}]}]});
    let script = round_two(
        true,
        "auto",
        json!([
            entry("party:tl_vanguard", json!("tl_brute"), json!([{"skill": "tl_shoulder_check"}]), "auto"),
            entry("party:tl_archer", json!("tl_footman"), json!([{"skill": "tl_master"}]), "auto"),
            entry("tl_footman", json!("party:tl_vanguard"), json!([{"skill": "tl_costly"}]), "auto")
        ]),
    );
    let mut s = at_round_two(&row(script, kill.clone()), timeline());
    assert!(!s.w.units[BRUTE].is_alive());
    let cells = script_cells(&s);
    assert_eq!(cells.len(), 3, "{cells:?}");
    assert!(cells[0].starts_with("Script: tl_footman cannot plan its picks: ") && cells[0].ends_with("; it plays its own round."), "{}", cells[0]);
    assert_eq!(cells[1], "Script: tl_vanguard has no target: tl_brute is not on the field; it plays its own round.");
    assert_eq!(cells[2], "Script: tl_archer has no skill 'tl_master'; it plays its own round.");
    assert!(s.round_locked() && s.plan_confirmed, "the round stays locked");
    assert_eq!(s.script_fallbacks(), 3);
    // the twin: the same battle without the script plays each of those units' own round
    let mut twin = at_round_two(&row(json!([]), kill), timeline());
    twin.auto_draft("balanced");
    assert!(twin.confirm_plan());
    assert_eq!(schedule(&s), schedule(&twin));
    s.resolve_round();
    twin.resolve_round();
    assert_eq!(s.resolver.log, twin.resolver.log);
    let script_lines: Vec<String> = s.log_lines(0, 0).into_iter().filter(|l| l.starts_with("[Script] ")).collect();
    assert_eq!(script_lines.len(), 3, "each cell has its combat log line");
}

/// SR8: under classic the script is ignored: one cell at the battle start, and the battle is the one without it.
#[test]
fn classic_ignores_the_script_with_one_cell() {
    let script = json!([{"round": 1, "entries": [entry("party:tl_vanguard", json!("tl_footman"), json!([{"skill": "tl_shoulder_check"}]), "auto")]}]);
    let mut s = start(&row(script, json!({})), Ruleset::classic());
    let mut plain = start(&row(json!([]), json!({})), Ruleset::classic());
    assert_eq!(script_cells(&s), vec![CLASSIC_LINE.to_string()]);
    assert!(script_cells(&plain).is_empty());
    assert!(s.round_script().is_none() && !s.round_locked() && !s.plan_confirmed);
    for b in [&mut s, &mut plain] {
        b.auto_draft("balanced");
        assert!(b.confirm_plan());
        b.resolve_round();
    }
    assert_eq!(s.resolver.log, plain.resolver.log);
    assert_eq!(hp(&s), hp(&plain));
    assert_eq!((s.scripted_rounds(), s.script_fallbacks()), (0, 0));
}

/// SN1: `party#n` (the n-th hero), `party:<id>` (a catalog fight: the party unit ref) and `ref#n` (the n-th unit built
/// from the ref) as entry units and targets.
#[test]
fn unit_names_cover_ref_n_party_id_and_party_n() {
    let script = round_two(
        true,
        "auto",
        json!([
            entry("party#1", json!("tl_footman#2"), json!([{"skill": "tl_shoulder_check"}]), "auto"),
            entry("party:tl_archer", json!("self"), json!(["rest"]), "rest"),
            entry("tl_footman#2", json!("self"), json!([{"skill": "active_guard"}]), "auto")
        ]),
    );
    let s = at_round_two(&row(script, json!({})), timeline());
    assert!(script_cells(&s).is_empty(), "{:?}", script_cells(&s));
    assert_eq!(s.w.round_states[&VANGUARD].target, Some(RoundTarget::Unit(FOOTMAN_B)));
    assert_eq!(rip::picks_of(&s.w, ARCHER), vec![Pick::Rest]);
    assert!(of(&s, ARCHER).iter().all(|n| n == "Rest"), "{:?}", of(&s, ARCHER));
    assert_eq!(of(&s, FOOTMAN_B).first().map(String::as_str), Some("Active Guard"));
    assert_eq!(script::unit_of(&s, "tl_footman"), Some(FOOTMAN));
    assert_eq!(script::unit_of(&s, "tl_footman#2"), Some(FOOTMAN_B));
    assert_eq!(script::unit_of(&s, "tl_footman#3"), None);
    assert_eq!(script::unit_of(&s, "party#2"), Some(ARCHER));
    assert_eq!(script::unit_of(&s, "party:tl_vanguard"), Some(VANGUARD));
}

/// SR2 `others: rest`: the members with no entry Rest the whole round.
#[test]
fn others_rest_rests_the_members_with_no_entry() {
    let script = round_two(true, "rest", json!([entry("party:tl_vanguard", json!("tl_footman"), json!([{"skill": "tl_shoulder_check"}]), "auto")]));
    let s = at_round_two(&row(script, json!({})), timeline());
    assert_eq!(rip::picks_of(&s.w, ARCHER), vec![Pick::Rest]);
    let archer = of(&s, ARCHER);
    assert!(!archer.is_empty() && archer.iter().all(|n| n == "Rest"), "{archer:?}");
}

/// SR7, SR9: the auto driver plays a battle with a locked round through (the round is the script's), and the metrics
/// count the scripted round and its fallbacks (in the JSON only then).
#[test]
fn the_auto_driver_plays_the_script_and_the_metrics_count_it() {
    let script = round_two(true, "auto", json!([entry("party:tl_vanguard", json!("tl_footman"), json!([{"skill": "tl_shoulder_check"}]), "auto"), entry("party:tl_archer", json!("self"), json!([{"skill": "tl_master"}]), "auto")]));
    let play = |row: &Value| {
        let mut s = start(row, timeline());
        s.attach_metrics(BattleMetrics::new(), true);
        while !s.combat_over {
            if s.phase == Phase::Replay {
                if !s.can_advance_round() {
                    break;
                }
                s.advance_round();
            }
            s.auto_draft("balanced");
            assert!(s.confirm_plan());
            s.resolve_round();
        }
        s.metrics_end(EndResult::default());
        s
    };
    let s = play(&row(script, json!({})));
    let m = s.metrics.as_deref().unwrap();
    assert_eq!((m.scripted_rounds, m.script_fallbacks), (1, 1));
    assert_eq!(m.to_json()["script"], json!({"scripted_rounds": 1, "script_fallbacks": 1}));
    let plain = play(&row(json!([]), json!({})));
    assert!(plain.metrics.as_deref().unwrap().to_json().get("script").is_none(), "other fights keep the shape");
}

/// Decision "ENCOUNTER2-C3 answers" (2): a locked round whose plan cannot be confirmed at its start (here a hold
/// escape's round 1, which waits for its holder; the content check refuses such a row, the session is built without
/// it) logs one `Script` cell, counts a fallback and plays guided: the player picks the holder and confirms.
#[test]
fn a_locked_round_that_cannot_be_confirmed_falls_back_to_a_guided_round() {
    let script = json!([{"round": 1, "lock": true, "entries": [entry("party:tl_vanguard", json!("self"), json!([{"skill": "active_guard"}]), "auto")]}]);
    let hold = json!({"objective": "escape", "stay_behind": {"mode": "hold", "tiles": [[6, 4]], "tiles_label": "the gap"}});
    let mut s = start(&row(script, hold), timeline());
    assert!(s.needs_stay_pick());
    assert!(!s.round_locked() && !s.plan_confirmed && s.is_planning(), "it plays guided");
    let line = script::lock_fallback_line(1, "a hold escape waits for its holder");
    assert_eq!(line, "Script: round 1 cannot be locked: a hold escape waits for its holder; it plays as a guided round.");
    assert_eq!(script_cells(&s), vec![line]);
    assert_eq!(s.log_lines(0, 0).iter().filter(|l| l.starts_with("[Script] ")).count(), 1, "its combat log line");
    assert_eq!((s.scripted_rounds(), s.script_fallbacks()), (1, 1));
    assert_eq!(rip::picks_of(&s.w, VANGUARD).len(), 1, "the entry stays as a first pick");
    assert_eq!(s.stay_pick(ARCHER), Ok(()));
    assert!(s.confirm_plan(), "the player confirms");
    s.resolve_round();
    assert_eq!(s.phase, Phase::Replay);
    assert_eq!(script_cells(&s).len(), 1, "logged once");
}
