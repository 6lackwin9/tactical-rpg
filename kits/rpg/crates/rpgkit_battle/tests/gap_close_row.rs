//! CLASS-BASICS (READY R2-W2b; FB-130 Option A; docs/sessions/I_AI.md section 1, INTENT2 matrix "AI rows") on a
//! scratch catalog built here (no shipped data): the round intent AI's role-agnostic Maneuver gap-close row under the
//! rule `field_intent`. A unit of any role holding a gap-close attack (a Maneuver attack with `dash_tiles`, like lunge)
//! picks a Maneuver round opened by it when no foe is in its normal reach, a foe is within the dash, and the landing
//! tile is safe (no hazard there, the round's likely incoming damage below its HP). Not when the landing is unsafe, a
//! foe is already in reach, the foe is beyond the dash, nor without the rule.

use indexmap::IndexMap;
use rpgkit_battle::ai::{self, IntentPick};
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::Plan;
use rpgkit_battle::round_intent::{self, ManeuverKind, Opener, RoundIntent, RoundTarget};
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::world::{AiProfile, Role, Tactic, UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, FIELD_INTENT, ROUND_INTENT, TIMELINE};
use serde_json::json;
use std::sync::Arc;

const HERO: UnitId = 0;
const FOE: UnitId = 1;
const FOE2: UnitId = 2;
const HERO_AT: V2 = V2::new(2, 4);
/// Two tiles east of the hero: just out of melee reach; its flank tiles (MR1: the transition reads `dash_tiles` as
/// `flank`) are 3 tiles from the hero, inside the dash.
const NEAR_AT: V2 = V2::new(4, 4);
const FAR_AT: V2 = V2::new(11, 0);
/// The lunge's dash.
const DASH: i32 = 3;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        // a gap-close attack (A4_EFFECTS section 2: an ATTACK with dash_tiles derives Maneuver under field_intent)
        json!({"id": "g_lunge", "name": "Lunge", "kind": "ATTACK", "level": 1, "blocks": 2, "power_mult": 1.0, "dash_tiles": DASH}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 5000, "power": 20, "armor": 0, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("p_hero", "party"), unit("p_foe", "enemy"), unit("p_foe2", "enemy")]
    }))
}

fn rules(list: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": list}).as_object().unwrap())
}

fn field_on() -> Ruleset {
    rules(&[ROUND_INTENT, FIELD_INTENT])
}

/// The hero at `HERO_AT`, FOE at `foe_at`, FOE2 far away.
fn session(ruleset: Ruleset, foe_at: V2) -> World {
    let row = json!({"id": "r_fight", "name": "Fight", "party": ["p_hero"], "enemies": ["p_foe", "p_foe2"], "total_blocks": 6, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, HERO_AT), (FOE, foe_at), (FOE2, FAR_AT)] {
        s.w.grid.positions[c] = Some(cell);
    }
    s.w.clone()
}

fn lunge(w: &World) -> usize {
    w.book().find("g_lunge").expect("g_lunge")
}

/// A front-line (not Skirmisher) profile that never flees nor guards on HP.
fn profile(w: &World) -> AiProfile {
    let mut p = w.profile_of(HERO);
    p.role = Role::Aggressor;
    p.flee_ratio = 0.0;
    p.defend_below = 0.0;
    p
}

/// The hero's AI round (the table, Balanced): the row, the pick and the planned skills in order.
fn ai_round(w: &mut World, profile: &AiProfile) -> (&'static str, Option<IntentPick>, Vec<usize>) {
    let mut plan = Plan::new();
    let round = ai::plan_round_intent(w, &mut plan, HERO, profile, Tactic::Balanced);
    let skills = plan.entries_for(w, HERO).into_iter().filter(|e| e.step_index >= 0).map(|e| e.skill).collect();
    (round.row, round.pick, skills)
}

/// A foe just out of melee reach and inside the dash: an Aggressor takes the gap-close row, a
/// Maneuver Engage round on the foe opened by the lunge, which is planned first.
#[test]
fn a_non_skirmisher_dashes_at_a_foe_out_of_reach_with_a_safe_landing() {
    let foe_at = NEAR_AT;
    let mut w = session(field_on(), foe_at);
    let g = lunge(&w);
    assert_eq!(round_intent::skill_rows(&w, &w.cat.skills[g]), ["maneuver"], "a gap-close attack derives Maneuver");
    let p = profile(&w);
    assert_ne!(p.role, Role::Skirmisher);
    let (row, pick, skills) = ai_round(&mut w, &p);
    assert_eq!(row, ai::ROW_GAP_CLOSE);
    let pick = pick.expect("a pick");
    assert_eq!((pick.intent, pick.target, pick.kind, pick.opener), (RoundIntent::Maneuver, RoundTarget::Unit(FOE), Some(ManeuverKind::Engage), Some(Opener::Skill(g))));
    assert_eq!(skills.first(), Some(&g), "the lunge opens");
    // the dash lands next to the foe
    let mut plan = Plan::new();
    ai::plan_round_intent(&mut w, &mut plan, HERO, &p, Tactic::Balanced);
    let e = plan.entries_for(&w, HERO).into_iter().find(|e| e.skill == g).expect("the lunge entry");
    assert_eq!(e.pos_after.dist(foe_at), 1, "lands next to the foe ({:?})", e.pos_after);
}

/// The same foe, but the landing tile is not safe (the round's likely incoming damage there reaches the hero's HP): not
/// the gap-close row.
#[test]
fn an_unsafe_landing_skips_the_row() {
    let mut w = session(field_on(), NEAR_AT);
    w.units[HERO].hp = 1;
    let p = profile(&w);
    let (row, _, _) = ai_round(&mut w, &p);
    assert_ne!(row, ai::ROW_GAP_CLOSE);
}

/// A foe already in melee reach (next to the hero): the Assault row fights it, not the gap-close row; a foe beyond the
/// dash: not the gap-close row either.
#[test]
fn a_foe_in_reach_or_beyond_the_dash_skips_the_row() {
    let mut w = session(field_on(), NEAR_AT);
    w.grid.positions[FOE2] = Some(HERO_AT + V2::new(0, 1));
    let p = profile(&w);
    let (row, _, _) = ai_round(&mut w, &p);
    assert_ne!(row, ai::ROW_GAP_CLOSE);
    let mut w = session(field_on(), HERO_AT + V2::new(DASH + 3, 0));
    let (row, _, _) = ai_round(&mut w, &p);
    assert_ne!(row, ai::ROW_GAP_CLOSE);
}

/// Without `field_intent` the row does not apply.
#[test]
fn without_the_rule_no_gap_close_row() {
    let mut w = session(rules(&[ROUND_INTENT]), NEAR_AT);
    let p = profile(&w);
    let (row, _, _) = ai_round(&mut w, &p);
    assert_ne!(row, ai::ROW_GAP_CLOSE);
}
