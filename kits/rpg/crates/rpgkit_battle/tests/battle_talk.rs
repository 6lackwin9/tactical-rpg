//! ENCOUNTER2-C1 on scratch catalogs built here (no shipped data): the phase `talk` effect (docs/specs/ENCOUNTER2_C.md
//! DT1-DT3) and the event triggers (PT1-PT4). Expectations come from the rules: a talk only appends its cue
//! {phase, dialogue, speaker, round, block} to `Session::talks` (HP, plans, draws, cells and outcome are those of the
//! same battle without it); a `round` trigger with `block` 0 fires at its round start (before planning), with `block` n
//! at the start of block n; `downed {unit, count}` and `broken {unit}` are due at the end of the block in which the
//! count-th unit of the name is down (or one of it breaks, rule poise only) and fire at the next block start, or the
//! next round start after the last block; a phase due when the battle ends never fires.

use indexmap::IndexMap;
use rpgkit_battle::encounter::TalkCue;
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::PlanStep;
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, POISE, TIMELINE};
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;

/// The roster order: party (Ash, Bryn), then the enemies (a brute, two guards, the boss).
const ASH: UnitId = 0;
const BRUTE: UnitId = 2;
const GUARD: UnitId = 3;
const GUARD_B: UnitId = 4;

fn skills() -> Vec<Value> {
    vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "p_jab", "name": "p_jab", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.2, "target_mode": "ENEMY", "stagger": 12}),
    ]
}

/// Units with `hp` each (a poise bar of 40 for the poise battles).
fn catalog(hp: i32) -> Arc<Catalog> {
    catalog_hp(hp, hp)
}

/// Heroes with `hero_hp`, enemies with `foe_hp`.
fn catalog_hp(hero_hp: i32, foe_hp: i32) -> Arc<Catalog> {
    let unit = |id: &str, name: &str, side: &str| {
        let hp = if side == "party" { hero_hp } else { foe_hp };
        json!({"id": id, "name": name, "side": side, "stat_mode": "fixed", "hp": hp, "speed": 12, "attack_range": 1, "skill_cap": 5, "poise": 40})
    };
    Arc::new(Catalog::from_value(&json!({
        "skills": skills(),
        "units": [unit("k_ash", "Ash", "party"), unit("k_bryn", "Bryn", "party"), unit("k_brute", "Brute", "enemy"), unit("k_guard", "Guard", "enemy"), unit("k_boss", "Boss", "enemy")],
    })))
}

/// A rout on an open floor with `phases` and `extra` merged over the row.
fn row(phases: Value, extra: Value) -> Value {
    let mut row = json!({"id": "talky", "name": "Talky", "rows": vec![".".repeat(12); 9], "party": ["k_ash", "k_bryn"],
        "enemies": ["k_brute", "k_guard", "k_guard", "k_boss"], "phases": phases,
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 6, "rng_seed": 1337});
    for (k, v) in extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    row
}

fn session_in(cat: Arc<Catalog>, row: Value, ruleset: Ruleset) -> Session {
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    Session::start(cat, def, &IndexMap::new())
}

fn session(row: Value) -> Session {
    session_in(catalog(5000), row, Ruleset::classic())
}

fn talk(dialogue: &str, speaker: &str) -> Value {
    json!({"type": "talk", "dialogue": dialogue, "speaker": speaker})
}

fn phase(id: &str, trigger: Value, effects: Value) -> Value {
    json!({"id": id, "name": id, "trigger": trigger, "effects": effects})
}

fn cue(phase: &str, dialogue: &str, speaker: &str, round: i32, block: i32) -> TalkCue {
    TalkCue { phase: phase.into(), dialogue: dialogue.into(), speaker: speaker.into(), round, block }
}

/// Steps (an empty party plan, the next round when one ends) until the battle stands at the start of `block` of
/// `round`, resolving, nothing of the block played yet (its start events still due). A step that ends a block moves to
/// the next one and stops, so the first step that shows block n > 1 leaves it unplayed; block 1 starts within the
/// confirming step (the target is then reached or passed).
fn step_to(s: &mut Session, round: i32, block: i32) {
    for _ in 0..2000 {
        if s.phase == Phase::Resolving && (s.current_round, s.current_block) >= (round, block) {
            assert_eq!(s.current_round, round, "passed round {round}");
            return;
        }
        assert!(!s.combat_over, "the battle ended before round {round} block {block}");
        if s.phase == Phase::Replay {
            s.advance_round();
        } else {
            s.step_next();
        }
    }
    panic!("never reached round {round} block {block}");
}

/// Plays on to the end of round `round` (or the battle).
fn finish_round(s: &mut Session, round: i32) {
    while !(s.phase == Phase::Replay && s.current_round == round) && !s.combat_over {
        if s.phase == Phase::Replay {
            s.advance_round();
        } else {
            s.step_next();
        }
    }
}

#[test]
fn a_round_one_talk_is_cued_before_round_one_is_planned() {
    let s = session(row(json!([phase("opening", json!({"kind": "round", "round": 1}), json!([talk("dlg_open", "ash")]))]), json!({})));
    assert!(s.is_planning(), "round 1 is still to plan");
    assert_eq!(s.talks(), [cue("opening", "dlg_open", "ash", 1, 0)]);
    assert_eq!(s.phases.fired.get("opening"), Some(&(1, 0)), "fired at the round start");
    assert!(s.phases.has_talk());
    // a battle without a talk effect lists none
    let plain = session(row(json!([phase("opening", json!({"kind": "round", "round": 1}), json!([{"type": "line", "text": "Hold."}]))]), json!({})));
    assert!(plain.talks().is_empty() && !plain.phases.has_talk());
}

#[test]
fn a_round_trigger_with_a_block_fires_at_that_block_start() {
    let mut s = session(row(json!([phase("mid", json!({"kind": "round", "round": 2, "block": 3}), json!([talk("dlg_mid", "")]))]), json!({})));
    step_to(&mut s, 2, 1);
    assert!(s.talks().is_empty(), "nothing at round 2's start");
    step_to(&mut s, 2, 3);
    assert!(s.phases.fired.get("mid").is_none(), "not before block 3 starts");
    s.step_next();
    assert_eq!(s.phases.fired.get("mid"), Some(&(2, 3)), "the phase takes block 3's first step");
    assert_eq!(s.talks(), [cue("mid", "dlg_mid", "", 2, 3)]);
    // block 0 is the round start, as a trigger without `block`
    let mut z = session(row(json!([phase("start", json!({"kind": "round", "round": 2, "block": 0}), json!([talk("dlg_start", "")]))]), json!({})));
    finish_round(&mut z, 1);
    assert!(z.talks().is_empty());
    z.advance_round();
    assert_eq!(z.talks(), [cue("start", "dlg_start", "", 2, 0)]);
}

/// A full battle with fights and deaths: the party auto-plans, both sides strike until it ends or round 6 closes.
fn play_out(row: Value) -> (Session, Vec<(UnitId, i32)>) {
    let mut s = session_in(catalog_hp(200, 40), row, Ruleset::classic());
    s.w.hp_journal = Some(Vec::new());
    for _ in 0..6 {
        if !s.plan_confirmed {
            s.auto_draft("balanced");
            assert!(s.confirm_plan());
        }
        s.resolve_round();
        if !s.can_advance_round() {
            break;
        }
        s.advance_round();
    }
    let journal = s.w.hp_journal.take().unwrap_or_default();
    (s, journal)
}

#[test]
fn a_talk_changes_no_hp_draw_cell_or_outcome() {
    let phases = |with_talk: bool| {
        let fx = |line: &str, d: &str| if with_talk { json!([{"type": "line", "text": line}, talk(d, "ash")]) } else { json!([{"type": "line", "text": line}]) };
        json!([
            phase("opening", json!({"kind": "round", "round": 1}), fx("Here they come.", "dlg_open")),
            phase("mid", json!({"kind": "round", "round": 2, "block": 2}), fx("Press on.", "dlg_mid")),
            phase("first_down", json!({"kind": "downed", "unit": "k_guard", "count": 1}), fx("One down.", "dlg_down")),
            phase("hero_down", json!({"kind": "downed", "unit": "party"}), fx("Get up!", "dlg_hero")),
        ])
    };
    let (with, journal_with) = play_out(row(phases(true), json!({})));
    let (without, journal_without) = play_out(row(phases(false), json!({})));
    // a round-start, a mid-block and a downed phase fired (a guard falls in this fight)
    for id in ["opening", "mid", "first_down"] {
        assert!(with.phases.fired.contains_key(id), "{id} fired: {:?}", with.phases.fired);
    }
    assert_eq!(with.talks().len(), with.phases.fired.len(), "one cue per fired phase");
    assert!(without.talks().is_empty());
    assert!(!journal_with.is_empty(), "the battle dealt damage");
    assert_eq!(journal_with, journal_without, "the HP journal");
    assert_eq!(with.log_lines(0, 0), without.log_lines(0, 0), "the combat log");
    assert_eq!(format!("{:?}", with.log), format!("{:?}", without.log), "the log cells");
    assert_eq!(with.events, without.events, "the event lines");
    assert_eq!(with.phases.fired, without.phases.fired, "the same phases fired at the same points");
    assert_eq!((with.outcome.clone(), with.current_round, with.combat_over), (without.outcome.clone(), without.current_round, without.combat_over));
    // each cue names the point its phase fired at
    for c in with.talks() {
        assert_eq!(with.phases.fired.get(&c.phase), Some(&(c.round, c.block)), "{c:?}");
    }
}

#[test]
fn downed_fires_after_the_count_th_fall() {
    let phases = json!([
        phase("one", json!({"kind": "downed", "unit": "k_guard", "count": 1}), json!([talk("dlg_one", "")])),
        phase("two", json!({"kind": "downed", "unit": "k_guard", "count": 2}), json!([talk("dlg_two", "")])),
        phase("brute", json!({"kind": "downed", "unit": "k_brute"}), json!([talk("dlg_brute", "")])),
    ]);
    let mut s = session(row(phases, json!({})));
    // the first guard falls in block 2: due at its end, fired at block 3's start
    step_to(&mut s, 1, 2);
    s.w.kill(GUARD, true);
    step_to(&mut s, 1, 3);
    assert!(s.phases.fired.get("one").is_none(), "not before block 3 starts");
    s.step_next();
    assert_eq!(s.phases.fired.get("one"), Some(&(1, 3)));
    assert!(s.phases.fired.get("two").is_none(), "one guard of two is down");
    // the second in block 4: count 2 at block 5's start
    step_to(&mut s, 1, 4);
    s.w.kill(GUARD_B, true);
    step_to(&mut s, 1, 5);
    s.step_next();
    assert_eq!(s.phases.fired.get("two"), Some(&(1, 5)));
    // the brute in the last block: at the next round start
    step_to(&mut s, 1, BLOCKS);
    s.w.kill(BRUTE, true);
    finish_round(&mut s, 1);
    assert!(s.phases.fired.get("brute").is_none(), "nothing at the round end");
    s.advance_round();
    assert_eq!(s.phases.fired.get("brute"), Some(&(2, 0)));
    let order: Vec<(&str, i32, i32)> = s.talks().iter().map(|t| (t.dialogue.as_str(), t.round, t.block)).collect();
    assert_eq!(order, [("dlg_one", 1, 3), ("dlg_two", 1, 5), ("dlg_brute", 2, 0)]);
}

#[test]
fn downed_counts_phase_kills_and_names_heroes() {
    let phases = json!([
        phase("cull", json!({"kind": "round", "round": 1, "block": 2}), json!([{"type": "kill", "units": ["k_brute"]}])),
        phase("after_cull", json!({"kind": "downed", "unit": "k_brute"}), json!([talk("dlg_cull", "")])),
        phase("ash_down", json!({"kind": "downed", "unit": "party:k_ash"}), json!([talk("dlg_ash", "")])),
        phase("any_hero", json!({"kind": "downed", "unit": "party", "count": 2}), json!([talk("dlg_all", "")])),
    ]);
    let mut s = session(row(phases, json!({})));
    step_to(&mut s, 1, 3);
    s.step_next();
    assert_eq!(s.phases.fired.get("cull"), Some(&(1, 2)), "the kill at block 2's start");
    assert_eq!(s.phases.fired.get("after_cull"), Some(&(1, 3)), "the kill's fall counts");
    assert!(s.phases.fired.get("ash_down").is_none());
    // `party:<id>` is the hero built from that party ref; `party` counts every hero
    assert_eq!(s.units_named("party:k_ash"), [ASH]);
    assert_eq!(s.units_named("party"), [0, 1]);
    assert_eq!(s.units_named("k_guard"), [GUARD, GUARD_B]);
    s.w.kill(ASH, true);
    step_to(&mut s, 1, 4);
    s.step_next();
    assert_eq!(s.phases.fired.get("ash_down"), Some(&(1, 4)));
    assert!(s.phases.fired.get("any_hero").is_none(), "one hero of two is down");
}

#[test]
fn a_phase_due_when_the_battle_ends_never_fires() {
    let phases = json!([phase("last", json!({"kind": "downed", "unit": "k_brute"}), json!([talk("dlg_last", "")]))]);
    let mut s = session(row(phases, json!({"enemies": ["k_brute"]})));
    step_to(&mut s, 1, 2);
    s.w.kill(BRUTE, true);
    s.step_next();
    while s.phase != Phase::Replay {
        s.step_next();
    }
    assert!(s.combat_over && s.outcome == "won", "{}", s.outcome);
    assert!(s.phases.fired.get("last").is_none() && s.talks().is_empty());
}

// ---- broken (rule poise) ----

const HERO: UnitId = 0;
const FOE: UnitId = 2;

fn poise_rules() -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [POISE]}).as_object().unwrap())
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

/// The hero jabs the brute (bar at 1) in block 1; returns the battle at block 2's start (its events still due).
fn jab_battle(ruleset: Ruleset) -> Session {
    let phases = json!([phase("snap", json!({"kind": "broken", "unit": "k_brute"}), json!([talk("dlg_snap", "")]))]);
    let mut s = session_in(catalog(5000), row(phases, json!({})), ruleset);
    for (c, cell) in [(0, V2::new(4, 4)), (1, V2::new(0, 8)), (FOE, V2::new(5, 4)), (3, V2::new(11, 0)), (4, V2::new(11, 8)), (5, V2::new(11, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s.replan_enemies();
    if let Some(p) = s.w.poise.get_mut(&FOE) {
        p.value = 1;
    }
    assert_eq!(s.plan.add_step(&s.w, HERO, PlanStep::engage(sid(&s.w, "p_jab"), FOE), true), "");
    step_to(&mut s, 1, 2);
    s
}

#[test]
fn broken_fires_after_a_break_under_poise_and_never_without() {
    let mut s = jab_battle(poise_rules());
    assert!(s.log.iter().any(|e| e.outcome == "BREAK"), "the jab broke the brute");
    assert!(s.phases.fired.get("snap").is_none(), "due at block 1's end, not fired yet");
    s.step_next();
    assert_eq!(s.phases.fired.get("snap"), Some(&(1, 2)));
    assert_eq!(s.talks(), [cue("snap", "dlg_snap", "", 1, 2)]);

    let mut c = jab_battle(Ruleset::classic());
    assert!(c.log.iter().all(|e| e.outcome != "BREAK"));
    finish_round(&mut c, 1);
    c.advance_round();
    assert!(c.phases.fired.get("snap").is_none() && c.talks().is_empty(), "classic has no breaks");
}
