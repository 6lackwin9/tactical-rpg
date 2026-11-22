//! PR7-41 (PT-16, PT-22) on a scratch catalog built here (no shipped data), under the round intent rules: the auto plan
//! the party drafts always confirms (a drafted round a later change broke is drafted again at Confirm; a round the
//! player set stays refused, and the refusal names the hero and the step); a guard that only covers friends opens
//! only with a friend near; Focus Weakest takes the weakest foe it lands an attack on this round over a weaker one it
//! can only close in on; a Guard round grants a status once, not every block; a hero whose target falls mid-round
//! turns on a foe it reaches instead of resting. Expectations come from the rules, not from printed numbers. The last
//! two rules wait for a golden ruling (FB-334) behind the test-only switches `regrant` and `retarget`
//! (`aegis_core::switches`, read once per process); their tests skip with the switch off and run with
//! `AEGIS_SWITCHES=regrant,retarget`.

use aegis_core::switches;
use indexmap::IndexMap;
use rpgkit_battle::ai;
use rpgkit_battle::grid::{GRID_W, TERRAIN_IDS, V2};
use rpgkit_battle::plan::Plan;
use rpgkit_battle::round_intent::RoundTarget;
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::world::{Tactic, UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::status;
use rpgkit_rules::ruleset::{Ruleset, AP_COST, ENEMY_MOVES, ROUND_INTENT, TIMELINE};
use serde_json::json;
use std::sync::Arc;

const BLOCKS: i32 = 6;
const BASICS: [&str; 4] = ["move", "rest", "strike", "active_guard"];

const HERO: UnitId = 0;
const MATE: UnitId = 1;
const FOE: UnitId = 2;
const FOE2: UnitId = 3;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "round_intent": "assault", "level": 1, "blocks": 1, "ap_cost": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "active_guard", "name": "Active Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 1, "mitigation": 0.35}),
        json!({"id": "t_fade", "name": "Fade", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 1, "ap_cost": 2, "mitigation": 0.2,
            "grant_status": "stealth", "status_rounds": 2}),
        json!({"id": "t_hook", "name": "Hook Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 1, "mitigation": 0.3,
            "triggers": [{"on": "ally_hit", "when": {"range": 1, "melee": true}, "effects": [{"type": "redirect", "pct": 100}], "limit": "round"}]}),
    ];
    let unit = |id: &str, side: &str, hp: i32, order: serde_json::Value| {
        json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": hp, "armor": 0, "speed": 12, "attack_range": 1, "skill_cap": 5,
            "ai_role": "AGGRESSOR", "flee_ratio": 0.0, "deck": {"order": order}})
    };
    let units = vec![
        unit("p_hero", "party", 300, json!({"assault": ["strike"], "guard": ["t_fade", "active_guard"]})),
        unit("p_mate", "party", 300, json!({"assault": ["strike"], "guard": ["active_guard"]})),
        unit("e_foe", "enemy", 300, json!({"assault": ["strike"], "guard": ["active_guard"]})),
        unit("e_foe2", "enemy", 300, json!({"assault": ["strike"], "guard": ["active_guard"]})),
    ];
    Catalog::from_value(&json!({"skills": skills, "units": units}))
}

fn session() -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["p_hero", "p_mate"], "enemies": ["e_foe", "e_foe2"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, ENEMY_MOVES, AP_COST], "basics": BASICS}).as_object().unwrap());
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for c in 0..s.w.units.len() {
        s.w.units[c].known_skills = ["strike", "active_guard", "t_fade", "t_hook"].into_iter().map(String::from).collect();
        s.w.remove(c);
    }
    s
}

fn place(w: &mut World, c: UnitId, x: i32, y: i32) {
    w.grid.positions[c] = Some(V2::new(x, y));
}

fn skill_id(w: &World, id: &str) -> usize {
    w.cat.skills.iter().position(|s| s.id == id).unwrap()
}

/// The skill ids of `c`'s compiled round, in order.
fn round_skills(w: &World, plan: &Plan, c: UnitId) -> Vec<String> {
    plan.compile(w, c).entries.iter().map(|e| w.cat.skills[e.skill].id.clone()).collect()
}

fn guard_round(w: &mut World, c: UnitId) -> Plan {
    let mut plan = Plan::new();
    let p = w.profile_of(c);
    ai::plan_round_intent(w, &mut plan, c, &p, Tactic::GuardCounter);
    plan
}

/// PT-22: a Guard round opened by a skill that grants its user a status (stealth) does not grant it again in the fill:
/// the later blocks take the next guard of the order. A hero already holding the status opens with the other guard.
#[test]
fn a_guard_round_grants_its_status_once() {
    if !switches::on("regrant") {
        eprintln!("skipped: switch `regrant` off (AEGIS_SWITCHES=regrant runs it)");
        return;
    }
    let mut s = session();
    let w = &mut s.w;
    place(w, HERO, 3, 4);
    place(w, FOE, 9, 4);
    let plan = guard_round(w, HERO);
    let skills = round_skills(w, &plan, HERO);
    assert_eq!(skills.first().map(String::as_str), Some("t_fade"), "the order's first guard opens: {skills:?}");
    assert_eq!(skills.iter().filter(|s| *s == "t_fade").count(), 1, "the status is granted once: {skills:?}");
    assert!(skills.iter().any(|s| s == "active_guard"), "the fill guards with the next skill: {skills:?}");
    assert!(plan.compile(w, HERO).ap_left > 0, "the round does not spend every AP on re-grants: {skills:?}");

    status::grant_now(&mut w.units[HERO], "stealth", 2, 0, "test");
    let plan = guard_round(w, HERO);
    let skills = round_skills(w, &plan, HERO);
    assert!(!skills.iter().any(|s| s == "t_fade"), "a held status is not re-granted: {skills:?}");
    assert_eq!(skills.first().map(String::as_str), Some("active_guard"));
}

/// PT-22: a guard whose every trigger is an `ally_hit` (it only covers friends) opens a Guard round only with an active
/// friend within its trigger range; alone, the next guard of the order opens and the fill leaves it out.
#[test]
fn a_cover_guard_needs_a_friend_to_cover() {
    let mut s = session();
    let w = &mut s.w;
    let hook = skill_id(w, "t_hook");
    w.units[HERO].known_skills.retain(|k| k != "t_fade");
    // the order: the cover guard first
    w.units[HERO].deck.order.insert("guard".into(), vec!["t_hook".into(), "active_guard".into()]);
    assert_eq!(rpgkit_battle::round_intent_plan::order(w, HERO, rpgkit_battle::round_intent::RoundIntent::Guard).first(), Some(&hook), "the fixture: the hook opens the order");
    place(w, HERO, 3, 4);
    place(w, FOE, 9, 4);
    place(w, MATE, 3, 8);
    let plan = guard_round(w, HERO);
    let skills = round_skills(w, &plan, HERO);
    assert!(!skills.iter().any(|s| s == "t_hook"), "no friend within 1: no cover guard: {skills:?}");
    assert!(skills.iter().any(|s| s == "active_guard"), "{skills:?}");

    place(w, MATE, 3, 5);
    let plan = guard_round(w, HERO);
    let skills = round_skills(w, &plan, HERO);
    assert_eq!(skills.first().map(String::as_str), Some("t_hook"), "a friend beside: the cover guard opens: {skills:?}");
}

/// PT-22: Focus Weakest takes the weakest foe the round lands an attack on; a weaker foe it can only close in on this
/// round comes after (before the fix it spent the round walking to it).
#[test]
fn focus_weakest_takes_a_foe_it_reaches_this_round() {
    let mut s = session();
    let w = &mut s.w;
    place(w, HERO, 2, 4);
    place(w, FOE, 3, 4);
    place(w, FOE2, 13, 4);
    w.units[FOE2].hp = 40;
    assert!(w.units[FOE2].hp < w.units[FOE].hp, "the fixture: the far foe is the weakest");
    let mut plan = Plan::new();
    let p = w.profile_of(HERO);
    let r = ai::plan_round_intent(w, &mut plan, HERO, &p, Tactic::FocusWeakest);
    let target = r.pick.as_ref().map(|p| p.target);
    assert_eq!(target, Some(rpgkit_battle::round_intent::RoundTarget::Unit(FOE)), "the foe in reach: {:?}", round_skills(w, &plan, HERO));
    assert!(plan.compile(w, HERO).entries.iter().any(|e| e.target == Some(FOE) && w.cat.skills[e.skill].id == "strike"));
}

/// A one-tile corridor west to east on row 4, (3, 4) to (5, 4), walled above and below: the hero starts at its west
/// end, FOE stands in a pocket at its east end (only (5, 4) reaches it), FOE2 in a pocket under (4, 4); the mate
/// stands above (4, 4) with one opening, (4, 3). The hero's round to FOE walks through (4, 4); the mate's round to FOE2
/// ends on (4, 4).
fn corridor() -> Session {
    let mut s = session();
    let w = &mut s.w;
    let wall = TERRAIN_IDS.iter().position(|t| *t == "wall").unwrap();
    for x in 1..=8 {
        for y in 1..=7 {
            let open = (y == 4 && (3..=6).contains(&x)) || (x == 4 && (2..=5).contains(&y));
            if !open {
                w.grid.terrain[(y * GRID_W + x) as usize] = wall;
            }
        }
    }
    place(w, HERO, 3, 4);
    place(w, FOE, 6, 4);
    place(w, FOE2, 4, 5);
    place(w, MATE, 4, 2);
    // the hero goes for FOE (the weakest), the mate for the nearest foe (FOE2)
    w.units[FOE].hp = 40;
    s
}

/// The corridor's drafts in party order, the way the battle screen drafts its ghosts; the mate's draft is the planner's
/// own (`plan_party_intents`), so nothing redrafts the hero yet.
fn corridor_drafts(s: &mut Session) -> Option<UnitId> {
    s.plan_apply_preset(HERO, "focus_weakest");
    assert!(s.plan_is_valid(), "the hero's draft is valid: {:?}", round_skills(&s.w, &s.plan, HERO));
    let target = s.plan.compile(&s.w, HERO).entries.iter().find_map(|e| e.target);
    ai::plan_party_intents(&mut s.w, &mut s.plan, &[(MATE, "aggressive".to_string())]);
    target
}

/// PT-16: a drafted round a later member's draft broke (the mate's round ends on the hero's path) is drafted again at
/// Confirm, so the auto plan confirms; the screen's per-member preset heals it at once.
#[test]
fn a_draft_a_later_draft_broke_confirms() {
    let mut s = corridor();
    assert_eq!(corridor_drafts(&mut s), Some(FOE), "the fixture: the hero goes for FOE");
    let mate_end = s.plan.planned_position(&s.w, MATE);
    assert_eq!(mate_end, V2::new(4, 4), "the fixture: the mate's round ends on the hero's path");
    assert!(!s.plan_is_valid(), "the mate's draft broke the hero's round: {:?}", s.plan_refusal());
    assert!(s.confirm_plan(), "Confirm drafts the broken round again: {:?}", s.plan_refusal());
    assert!(s.plan_confirmed);

    // the battle screen's way: the mate's ghost through `plan_apply_preset` redrafts the hero at once
    let mut s = corridor();
    s.plan_apply_preset(HERO, "focus_weakest");
    s.plan_apply_preset(MATE, "aggressive");
    assert_eq!(s.plan.planned_position(&s.w, MATE), V2::new(4, 4));
    assert!(s.plan_is_valid(), "{:?}", s.plan_refusal());
}

/// PT-16: a round edited by hand is never replaced at Confirm: the hero's drafted round, its target then set by hand,
/// broken by the mate's draft, is refused, and the refusal names the hero, the step and its skill.
#[test]
fn a_hand_edited_round_is_refused_not_redrafted() {
    let mut s = corridor();
    s.plan_apply_preset(HERO, "focus_weakest");
    let steps = s.plan.steps_of(HERO);
    s.set_round_target(HERO, RoundTarget::Unit(FOE)).expect("the target by hand");
    assert_eq!(s.plan.steps_of(HERO).len(), steps.len(), "the steps stay");
    s.plan_apply_preset(MATE, "aggressive");
    assert!(!s.plan_is_valid(), "the mate's draft broke the hand-set round, and nothing redrafted it");
    let before = s.plan.steps_of(HERO).len();
    assert!(!s.confirm_plan(), "a round set by hand stays refused");
    assert_eq!(s.plan.steps_of(HERO).len(), before, "and is not replaced");
    let why = s.plan_refusal().expect("a refusal");
    let st = s.plan.compile(&s.w, HERO);
    let step = usize::try_from(st.error_index).expect("a broken step");
    let skill = s.plan.steps_of(HERO)[step].skill.map_or("Move".to_string(), |k| s.w.cat.skills[k].name.clone());
    assert!(why.starts_with(&format!("p_hero, step {} ({skill}): ", step + 1)), "the hero, the step and the skill: {why}");
    assert!(!why.contains("plan_view"), "no developer words: {why}");

    // the tactic drafted again over the hand edit makes it the auto draft again
    let mut s = corridor();
    s.plan_apply_preset(HERO, "focus_weakest");
    s.set_round_target(HERO, RoundTarget::Unit(FOE)).unwrap();
    s.plan_apply_preset(HERO, "focus_weakest");
    s.plan_apply_preset(MATE, "aggressive");
    assert!(s.plan_is_valid(), "a draft after the hand edit is the auto draft again: {:?}", s.plan_refusal());
}

/// PT-22: a hero whose target falls before its next attack starts turns on the foe it reaches ("[Retarget]") instead
/// of resting the rest of the round (TG4's Rest stays for a hero with no foe in reach).
#[test]
fn a_hero_whose_target_falls_retargets() {
    if !switches::on("retarget") {
        eprintln!("skipped: switch `retarget` off (AEGIS_SWITCHES=retarget runs it)");
        return;
    }
    let mut s = session();
    place(&mut s.w, HERO, 4, 4);
    place(&mut s.w, FOE, 5, 4);
    place(&mut s.w, FOE2, 4, 5);
    place(&mut s.w, MATE, 0, 9);
    let mut plan_ok = false;
    s.plan_apply_preset_all("aggressive");
    let first = s.plan.compile(&s.w, HERO).entries.iter().find_map(|e| e.target).expect("the hero attacks");
    let other = if first == FOE { FOE2 } else { FOE };
    let attacks = s.plan.compile(&s.w, HERO).entries.iter().filter(|e| e.target == Some(first)).count();
    assert!(attacks >= 2, "the fixture: more than one attack on the target");
    // the target dies to the first hit
    s.w.units[first].hp = 1;
    if s.confirm_plan() {
        plan_ok = true;
    }
    assert!(plan_ok);
    let hp_other = s.w.units[other].hp;
    s.resolve_round();
    let lines: Vec<String> = s.resolver.log.clone();
    assert!(!s.w.units[first].is_active(), "the target fell: {lines:#?}");
    assert!(lines.iter().any(|l| l.starts_with("[Retarget] p_hero's target")), "the hero turns on the other foe: {lines:#?}");
    assert!(!lines.iter().any(|l| l.starts_with("[Rest] p_hero has lost")), "no TG4 rest: {lines:#?}");
    assert!(s.w.units[other].hp < hp_other, "the other foe is hit: {lines:#?}");
}

/// PLAN-FIX2 (PT-91, FB-406): the fill after hand picks does not repeat them: after "Move here, then Fade" the fill
/// grants the stealth no second time (whatever the `regrant` switch) and guards with the next skill; a picked guard
/// that is not an attack is not filled again either.
#[test]
fn the_fill_after_hand_picks_does_not_repeat_them() {
    use rpgkit_battle::round_intent::Pick;
    use rpgkit_battle::round_intent_plan as rip;
    let mut s = session();
    let w = &mut s.w;
    w.autofill = true;
    place(w, HERO, 3, 4);
    place(w, FOE, 9, 4);
    let mut plan = Plan::new();
    let (fade, guard) = (skill_id(w, "t_fade"), skill_id(w, "active_guard"));
    let tile = V2::new(4, 4);
    let picks = [Pick::MoveHere(tile), Pick::Skill { skill: fade, kind: None }];
    let (done, _) = rip::pick(w, &mut plan, HERO, RoundTarget::Cell(tile), &picks);
    assert_eq!(done, Ok(()));
    let skills = round_skills(w, &plan, HERO);
    assert_eq!(skills.iter().filter(|s| *s == "t_fade").count(), 1, "the picked stealth is granted once: {skills:?}");
    assert!(skills.iter().any(|s| s == "active_guard"), "the fill guards with the next skill: {skills:?}");

    let mut plan = Plan::new();
    let picks = [Pick::Skill { skill: guard, kind: None }];
    let (done, _) = rip::pick(w, &mut plan, HERO, RoundTarget::Myself, &picks);
    assert_eq!(done, Ok(()));
    let skills = round_skills(w, &plan, HERO);
    assert_eq!(skills.iter().filter(|s| *s == "active_guard").count(), 1, "a picked guard is not filled again: {skills:?}");
}
