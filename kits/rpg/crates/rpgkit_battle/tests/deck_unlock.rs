//! HOOKS1-U on a scratch catalog built here (no shipped data): the encounter phase effect `deck` (HOOKS1 10.1 DU1)
//! under the rule `deck`. A fired effect adds its families to each active unit's deck actives and replaces the named
//! intents' order lists from that unit's next planning (a phase fired mid-round: the next round; at a round start: that
//! round), as the `ai` effect's tactics; without the rule nothing changes. Expectations come from the rules and the
//! fixture's own rows. The content checks (DU2: an unowned `add`, actives over `deck_slots`, one effect per unit) are
//! covered in `rpgkit_content/tests/hooks_schema.rs`.

use indexmap::IndexMap;
use rpgkit_battle::round_intent::RoundIntent;
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::skill::SkillId;
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, DECK, ROUND_INTENT, TIMELINE};
use serde_json::{json, Value};
use std::sync::Arc;

/// The class path, in path order (every family unlocks at level 1).
const PATH: [&str; 5] = ["d_one", "d_two", "d_three", "d_four", "d_ward"];
/// The test ruleset's basics, in their order.
const BASICS: [&str; 3] = ["move", "rest", "strike"];
/// The active slots of the test ruleset.
const ACTIVE_SLOTS: i32 = 3;
/// The boss's authored deck actives (within the slots, one short of them).
const BOSS_ACTIVES: [&str; 2] = ["d_one", "d_two"];
/// The family the boss's phase unlocks (on its class path, not in its deck).
const UNLOCKED: &str = "d_three";
/// The boss's assault order after the phase.
const NEW_ORDER: [&str; 3] = ["d_three", "d_one", "strike"];
/// The auto-deck enemy's assault order from its round-2 phase.
const AUTO_ORDER: [&str; 2] = ["d_two", "strike"];
/// The boss's phase threshold (percent of max HP).
const PCT: i32 = 50;

const BOSS: UnitId = 1;
const AUTO: UnitId = 2;

fn catalog() -> Catalog {
    let atk = |id: &str| json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.1, "target_mode": "ENEMY"});
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        atk("d_one"),
        atk("d_two"),
        atk("d_three"),
        atk("d_four"),
        json!({"id": "d_ward", "name": "d_ward", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.3}),
    ];
    let path: serde_json::Map<String, Value> = PATH.iter().map(|f| (f.to_string(), json!(1))).collect();
    let unit = |id: &str, side: &str, deck: Value| json!({"id": id, "name": id, "side": side, "class_id": "drill", "level": 1, "stat_mode": "derived", "deck": deck});
    Catalog::from_value(&json!({
        "skills": skills,
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 8, "DEX": 10, "INT": 6, "CONS": 30, "PERC": 10, "KNOW": 10}, "skill_path": path}],
        "units": [
            unit("u_hero", "party", json!({})),
            unit("u_boss", "enemy", json!({"active": BOSS_ACTIVES})),
            unit("u_auto", "enemy", json!({}))
        ]
    }))
}

fn ruleset(rules: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules, "basics": BASICS, "deck_slots": {"active": ACTIVE_SLOTS, "passive": 1, "keystone": 1}}).as_object().unwrap())
}

/// The boss unlocks `UNLOCKED` and a new assault order below `PCT` HP; the auto-deck enemy gets a new assault order at
/// the start of round 2.
fn phases() -> Value {
    json!([
        {"id": "crush", "name": "Crush", "trigger": {"kind": "hp_below", "unit": "u_boss", "pct": PCT},
         "effects": [{"type": "line", "text": "The boss lifts the great hammer."}, {"type": "deck", "unit": "u_boss", "add": [UNLOCKED], "order": {"assault": NEW_ORDER}}]},
        {"id": "drill", "name": "Drill", "trigger": {"kind": "round", "round": 2},
         "effects": [{"type": "deck", "unit": "u_auto", "order": {"assault": AUTO_ORDER}}]}
    ])
}

fn session(rules: Ruleset) -> Session {
    session_with(rules, phases())
}

fn session_with(rules: Ruleset, phases: Value) -> Session {
    let row = json!({"id": "d_fight", "name": "Fight", "party": ["u_hero"], "enemies": ["u_boss", "u_auto"], "total_blocks": 6, "unit_ap": 10, "max_rounds": 8, "phases": phases});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = rules;
    Session::start_with(Arc::new(catalog()), def, &IndexMap::new(), Vec::new())
}

fn strings(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

fn skill_ids(w: &World, fams: &[&str]) -> Vec<SkillId> {
    fams.iter().map(|f| w.book().find(f).unwrap_or_else(|| panic!("skill {f}"))).collect()
}

fn assault(s: &Session, c: UnitId) -> Vec<SkillId> {
    rip::order(&s.w, c, RoundIntent::Assault)
}

/// Steps the session until round `round` reaches block `block` (or the round's replay).
fn step_to(s: &mut Session, round: i32, block: i32) {
    while !(s.current_round == round && s.current_block >= block) && !(s.current_round == round && s.phase == Phase::Replay) {
        if s.phase == Phase::Replay {
            assert!(s.can_advance_round(), "the battle goes on to round {round}");
            s.advance_round();
        } else {
            s.step_next();
        }
    }
}

/// Steps through the rest of the round and starts the next one.
fn next_round(s: &mut Session) {
    let r = s.current_round;
    while s.current_round == r && s.phase != Phase::Replay {
        s.step_next();
    }
    assert!(s.can_advance_round(), "round {r} ends with the battle going on");
    s.advance_round();
}

/// DU1: a phase fired at block 4 of round 2 changes nothing in round 2; round 3 plans with the unlocked family in the
/// actives and the new order (the authored actives first, the added one after them, within the slots).
#[test]
fn a_deck_phase_mid_round_applies_from_the_next_round() {
    let mut s = session(ruleset(&[ROUND_INTENT, DECK]));
    let start_actives = s.w.units[BOSS].active_families();
    assert_eq!(start_actives, strings(&BOSS_ACTIVES), "the authored deck fills the actives");
    let before = assault(&s, BOSS);
    let unlocked = skill_ids(&s.w, &[UNLOCKED])[0];
    assert!(!before.contains(&unlocked), "the unlocked family is not in the boss's order before the phase");

    step_to(&mut s, 2, 3);
    assert!(s.phases.fired.get("crush").is_none(), "the boss is above {PCT}% so far");
    let max = s.w.units[BOSS].max_hp;
    s.w.units[BOSS].hp = max * PCT / 100;
    step_to(&mut s, 2, 4);
    while s.phases.fired.get("crush").is_none() && s.current_round == 2 && s.phase != Phase::Replay {
        s.step_next();
    }
    assert_eq!(s.phases.fired.get("crush"), Some(&(2, 4)), "the phase fires at the start of block 4 after block 3 ends below {PCT}%");
    assert!(s.events.iter().any(|e| e.starts_with("Crush:") && e.contains("new skills from the next round")), "{:?}", s.events);
    assert_eq!(s.w.units[BOSS].active_families(), start_actives, "round 2 keeps the actives");
    assert_eq!(assault(&s, BOSS), before, "round 2 keeps the order");

    next_round(&mut s);
    assert_eq!(s.current_round, 3);
    let mut actives = strings(&BOSS_ACTIVES);
    actives.push(UNLOCKED.to_string());
    assert_eq!(s.w.units[BOSS].active_families(), actives, "round 3: the added family takes the next slot");
    assert_eq!(s.w.units[BOSS].deck.active, actives);
    assert_eq!(assault(&s, BOSS), skill_ids(&s.w, &NEW_ORDER), "round 3 plans with the new order");
    assert!(s.w.units[BOSS].can_use(&s.w.cat, s.w.cat.skill(UNLOCKED).unwrap()), "the unlocked family is usable");

    // applied once: a later round keeps it as it is
    next_round(&mut s);
    assert_eq!(s.w.units[BOSS].active_families(), actives);
    assert_eq!(assault(&s, BOSS), skill_ids(&s.w, &NEW_ORDER));
}

/// DU1: a phase fired at a round start applies to that round's planning; an order-only effect leaves an auto deck's
/// actives automatic (its slots as built) and replaces only the named intent's list.
#[test]
fn a_round_start_deck_phase_applies_that_round_and_an_order_alone_keeps_the_auto_actives() {
    let mut s = session(ruleset(&[ROUND_INTENT, DECK]));
    let start_actives = s.w.units[AUTO].active_families();
    assert_eq!(start_actives.len(), ACTIVE_SLOTS as usize, "the auto deck fills every active slot from the class path");
    let before = assault(&s, AUTO);
    assert_ne!(before, skill_ids(&s.w, &AUTO_ORDER));
    next_round(&mut s);
    assert_eq!(s.phases.fired.get("drill"), Some(&(2, 0)));
    assert!(s.events.iter().any(|e| e.starts_with("Drill:") && e.ends_with("(new skills)")), "{:?}", s.events);
    assert_eq!(assault(&s, AUTO), skill_ids(&s.w, &AUTO_ORDER), "round 2 plans with the new order");
    assert!(s.w.units[AUTO].deck.active.is_empty(), "the actives stay automatic");
    assert_eq!(s.w.units[AUTO].active_families(), start_actives);
}

/// Without the rule `deck` the phases fire but no deck, order or log part changes.
#[test]
fn without_the_deck_rule_nothing_changes() {
    let mut s = session(ruleset(&[ROUND_INTENT]));
    let actives: Vec<Vec<String>> = [BOSS, AUTO].iter().map(|&c| s.w.units[c].active_families()).collect();
    let orders: Vec<Vec<SkillId>> = [BOSS, AUTO].iter().map(|&c| assault(&s, c)).collect();
    let decks = [s.w.units[BOSS].deck.clone(), s.w.units[AUTO].deck.clone()];
    let max = s.w.units[BOSS].max_hp;
    s.w.units[BOSS].hp = max * PCT / 100;
    next_round(&mut s);
    next_round(&mut s);
    assert!(s.phases.fired.contains_key("crush") && s.phases.fired.contains_key("drill"), "both phases fire");
    assert!(!s.events.iter().any(|e| e.contains("new skills")), "{:?}", s.events);
    assert_eq!([BOSS, AUTO].iter().map(|&c| s.w.units[c].active_families()).collect::<Vec<_>>(), actives);
    assert_eq!([BOSS, AUTO].iter().map(|&c| assault(&s, c)).collect::<Vec<_>>(), orders);
    assert_eq!([s.w.units[BOSS].deck.clone(), s.w.units[AUTO].deck.clone()], decks);
}

/// DU1 as built (user 2026-10-29 (b)): an added family always lands. On an automatic deck whose class path fills every
/// active slot the unlocked family pushes out the last automatic one: active from the next planning, the slot count
/// unchanged; an authored deck's actives are never pushed out.
#[test]
fn an_add_on_a_full_automatic_deck_pushes_out_the_last_automatic_family() {
    let extra = PATH[ACTIVE_SLOTS as usize];
    let fx = json!([{"id": "unlock", "name": "Unlock", "trigger": {"kind": "round", "round": 2},
        "effects": [{"type": "deck", "unit": "u_auto", "add": [extra]}, {"type": "deck", "unit": "u_boss", "add": [UNLOCKED]}]}]);
    let mut s = session_with(ruleset(&[ROUND_INTENT, DECK]), fx);
    let start = s.w.units[AUTO].active_families();
    assert_eq!(start, strings(&PATH[..ACTIVE_SLOTS as usize]), "the class path fills the automatic deck's slots");
    assert!(!start.contains(&extra.to_string()));
    let unlocked = skill_ids(&s.w, &[extra])[0];
    assert!(!assault(&s, AUTO).contains(&unlocked));
    next_round(&mut s);
    let mut want = start.clone();
    let pushed = want.pop().unwrap();
    want.push(extra.to_string());
    let now = s.w.units[AUTO].active_families();
    assert_eq!(now, want, "the added family is active, the last automatic one ({pushed}) is gone");
    assert_eq!(now.len(), ACTIVE_SLOTS as usize, "the slot count is unchanged");
    assert!(assault(&s, AUTO).contains(&unlocked), "round 2 plans with the unlocked family");
    // the authored deck (one short of its slots) keeps both authored actives and takes the added family after them
    let mut boss = strings(&BOSS_ACTIVES);
    boss.push(UNLOCKED.to_string());
    assert_eq!(s.w.units[BOSS].active_families(), boss);
}
