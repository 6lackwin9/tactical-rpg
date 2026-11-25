//! W2-5 I1.2-S3 (docs/sessions/I_AI.md section 1 item 5 slice 3; INTENT2 matrix "AI rows", INTENT2-AI) on a scratch
//! catalog built here (no shipped data): the round intent AI's ring row and full row Z under the rule `field_intent`.
//! - Ring row (before row 4): two or more foes inside a usable RING of the unit's Stand order give a Stand round on
//!   itself opened by the ring; a reckless unit takes it too and keeps no Defend skill in it ("reckless drops Defend
//!   skills, not the Stand intent").
//! - Row Z (before row 5; K1b's minimal row plus the matrix's conditions): an elite or boss takes a Field round only on
//!   a source that also holds its first ranked foe; a unit whose first ranked foe is a taunter only on a source that
//!   covers the taunter; with `flush_cover` a foe hit in cover counts twice in the source's score.
//!
//! Without the rule neither row applies.

use indexmap::IndexMap;
use rpgkit_battle::ai::{self, IntentPick};
use rpgkit_battle::command::cover_for;
use rpgkit_battle::grid::{V2, GRID_W, TERRAIN_IDS};
use rpgkit_battle::plan::Plan;
use rpgkit_battle::round_intent::{self, Opener, RoundIntent, RoundTarget};
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::skill::{self, Kind};
use rpgkit_battle::world::{AiProfile, Tactic, UnitId, UnitTactic, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, AI_TACTICS, FIELD_INTENT, ROUND_INTENT, TIMELINE};
use serde_json::{json, Value};
use std::sync::Arc;

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const FOE2: UnitId = 3;
const FOE3: UnitId = 4;
const HERO_AT: V2 = V2::new(2, 4);
/// FOE and FOE2 make a cluster inside one BURST source (band 2-3 from the hero); FOE3 and the ally are away.
const FOE_AT: V2 = V2::new(5, 4);
const FOE2_AT: V2 = V2::new(5, 5);
const FOE3_AT: V2 = V2::new(9, 8);
const ALLY_AT: V2 = V2::new(0, 0);

fn catalog() -> Catalog {
    let row = |id: &str, extra: Value| {
        let mut r = json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.5});
        r.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        r
    };
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        // a thrown BURST (IR21 Field, Assault)
        row("f_burst", json!({"blocks": 2, "reach_min": 2, "reach_max": 3, "area": "BURST"})),
        // a RING (IR21: Stand only), drawn around the user
        row("f_ring", json!({"blocks": 2, "area": "RING"})),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 5000, "power": 20, "armor": 0, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("p_hero", "party"), unit("p_ally", "party"), unit("p_foe", "enemy"), unit("p_foe2", "enemy"), unit("p_foe3", "enemy")]
    }))
}

fn rules(list: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": list}).as_object().unwrap())
}

fn field_on() -> Ruleset {
    rules(&[ROUND_INTENT, FIELD_INTENT])
}

fn session(ruleset: Ruleset) -> World {
    let row = json!({"id": "r_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe", "p_foe2", "p_foe3"], "total_blocks": 6, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, HERO_AT), (ALLY, ALLY_AT), (FOE, FOE_AT), (FOE2, FOE2_AT), (FOE3, FOE3_AT)] {
        s.w.grid.positions[c] = Some(cell);
    }
    s.w.clone()
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

/// The hero's AI round (the table, Balanced) with `profile`: the row, the pick and the planned skills in order.
fn ai_round(w: &mut World, profile: &AiProfile) -> (&'static str, Option<IntentPick>, Vec<usize>) {
    let mut plan = Plan::new();
    let round = ai::plan_round_intent(w, &mut plan, HERO, profile, Tactic::Balanced);
    let skills = plan.entries_for(w, HERO).into_iter().filter(|e| e.step_index >= 0).map(|e| e.skill).collect();
    (round.row, round.pick, skills)
}

fn profile(w: &World) -> AiProfile {
    let mut p = w.profile_of(HERO);
    p.flee_ratio = 0.0;
    p
}

/// The tiles a Field pick's opener marks from the hero's tile.
fn marked(w: &World, pick: &IntentPick) -> Vec<V2> {
    let (Some(Opener::Skill(s)), RoundTarget::Cell(aim)) = (pick.opener.clone(), pick.target) else { panic!("a Field pick with a skill opener: {pick:?}") };
    skill::field_cells(&w.cat.skills[s], HERO_AT, aim)
}

// ---- Ring row ---------------------------------------------------------------------------------------------------

/// Two foes inside the ring: the ring row, a Stand round on itself opened by the ring; one foe: not the ring row;
/// without `field_intent`: not the ring row.
#[test]
fn two_foes_in_a_ring_give_a_stand_round_opened_by_it() {
    let mut w = session(field_on());
    w.grid.positions[FOE] = Some(HERO_AT + V2::new(1, 1));
    w.grid.positions[FOE2] = Some(HERO_AT + V2::new(-1, 0));
    let ring = sid(&w, "f_ring");
    assert!(round_intent_ring_rows(&w, ring));
    let p = profile(&w);
    let (row, pick, skills) = ai_round(&mut w, &p);
    assert_eq!(row, ai::ROW_RING);
    let pick = pick.expect("a pick");
    assert_eq!((pick.intent, pick.target, pick.opener), (RoundIntent::Guard, RoundTarget::Myself, Some(Opener::Skill(ring))));
    assert_eq!(skills.first(), Some(&ring), "the ring opens");
    // one foe in the ring: below RING_MIN_FOES
    assert_eq!(ai::RING_MIN_FOES, 2);
    let mut w = session(field_on());
    w.grid.positions[FOE] = Some(HERO_AT + V2::new(1, 1));
    let (row, _, _) = ai_round(&mut w, &p);
    assert_ne!(row, ai::ROW_RING);
    // without the rule
    let mut w = session(rules(&[ROUND_INTENT]));
    w.grid.positions[FOE] = Some(HERO_AT + V2::new(1, 1));
    w.grid.positions[FOE2] = Some(HERO_AT + V2::new(-1, 0));
    let (row, _, _) = ai_round(&mut w, &p);
    assert_ne!(row, ai::ROW_RING);
}

/// IR21: a RING derives Stand only under the rule.
fn round_intent_ring_rows(w: &World, ring: usize) -> bool {
    round_intent::skill_rows(w, &w.cat.skills[ring]) == ["guard"]
}

/// "Reckless drops Defend skills, not the Stand intent": a reckless unit takes the ring row (a reckless boss flails)
/// and its round keeps no Defend skill; the same round for a unit that is not reckless keeps the Defend the Stand
/// order puts after the ring.
#[test]
fn a_reckless_unit_takes_the_ring_row_without_defend_skills() {
    let play = |reckless: bool| {
        let mut w = session(field_on());
        w.grid.positions[FOE] = Some(HERO_AT + V2::new(1, 1));
        w.grid.positions[FOE2] = Some(HERO_AT + V2::new(-1, 0));
        // the Stand order: the Defend first, then the ring (the ring row opens with the ring, the fill tries Guard first)
        w.units[HERO].deck.order.insert("guard".into(), vec!["guard".into(), "f_ring".into()]);
        let mut p = profile(&w);
        p.reckless = reckless;
        let (row, pick, skills) = ai_round(&mut w, &p);
        let kinds: Vec<Kind> = skills.iter().map(|&s| skill::kind(&w.cat.skills[s])).collect();
        (row, pick.map(|p| p.drop_defend), kinds, skills.first().copied() == Some(sid(&w, "f_ring")))
    };
    let (row, drop, kinds, ring_first) = play(false);
    assert_eq!((row, drop, ring_first), (ai::ROW_RING, Some(false), true));
    assert!(kinds.contains(&Kind::Defend), "not reckless: the fill guards after the ring ({kinds:?})");
    let (row, drop, kinds, ring_first) = play(true);
    assert_eq!((row, drop, ring_first), (ai::ROW_RING, Some(true), true), "reckless: still the ring row");
    assert!(!kinds.contains(&Kind::Defend), "reckless: no Defend skill ({kinds:?})");
    assert!(kinds.contains(&Kind::Attack));
}

// ---- Row Z ------------------------------------------------------------------------------------------------------

/// An elite or boss takes a Field round only on a source that also holds its first ranked foe: with the nearest foe
/// (first ranked for an Aggressor) apart from the cluster, the plain unit still takes the cluster, the elite does not;
/// with the first ranked foe in the cluster the elite takes it too.
#[test]
fn an_elite_takes_a_field_round_only_on_its_first_ranked_foe() {
    let apart = |tier: &str| {
        let mut w = session(field_on());
        w.grid.positions[FOE3] = Some(HERO_AT + V2::new(-1, 0));
        w.units[HERO].tier = tier.into();
        let p = profile(&w);
        ai_round(&mut w, &p).0
    };
    assert_eq!(apart("standard"), ai::ROW_FIELD, "a plain unit: the cluster");
    assert_ne!(apart("elite"), ai::ROW_FIELD, "an elite: its first ranked foe is in no source with the cluster");
    assert_ne!(apart("boss"), ai::ROW_FIELD);
    let mut w = session(field_on());
    w.units[HERO].tier = "elite".into();
    let p = profile(&w);
    let (row, pick, _) = ai_round(&mut w, &p);
    assert_eq!(row, ai::ROW_FIELD);
    assert!(marked(&w, &pick.unwrap()).contains(&FOE_AT), "the source holds the first ranked foe (the nearest)");
}

/// A unit whose first ranked foe is a taunter picks only a source covering it: a taunter apart from the cluster
/// leaves no Field round; a taunter in the cluster is in the source.
#[test]
fn a_taunted_unit_picks_only_a_source_covering_the_taunter() {
    let mut w = session(field_on());
    w.grid.positions[FOE3] = Some(V2::new(2, 6));
    rpgkit_rules::status::grant_now(&mut w.units[FOE3], "taunt", 2, 0, "test");
    assert!(ai::pulls_targeting("taunt"));
    let p = profile(&w);
    assert_ne!(ai_round(&mut w, &p).0, ai::ROW_FIELD, "the taunter is in no source with another foe");
    let mut w = session(field_on());
    rpgkit_rules::status::grant_now(&mut w.units[FOE2], "taunt", 2, 0, "test");
    let (row, pick, _) = ai_round(&mut w, &p);
    assert_eq!(row, ai::ROW_FIELD);
    assert!(marked(&w, &pick.unwrap()).contains(&FOE2_AT), "the source covers the taunter");
}

/// Row Z extends `flush_pick`: with `flush_cover` a foe hit in cover against the unit counts twice, so of two
/// two-foe sources the one with a covered foe wins; without the tactic the order and cell index decide.
#[test]
fn flush_cover_prefers_the_field_source_with_a_covered_foe() {
    // FOE (5, 2) and FOE2 (5, 4) share the sources on row 3, FOE2 and FOE3 (5, 6) those on row 5; a wall beside FOE3
    let play = |flush: bool| {
        let mut w = session(rules(&[ROUND_INTENT, FIELD_INTENT, AI_TACTICS]));
        for (c, cell) in [(FOE, V2::new(5, 2)), (FOE2, V2::new(5, 4)), (FOE3, V2::new(5, 6))] {
            w.grid.positions[c] = Some(cell);
        }
        let wall = TERRAIN_IDS.iter().position(|t| *t == "wall").unwrap();
        // a scratch wall that gives cover (the scratch catalog's terrain rows give none)
        w.grid.table.rows[wall].cover_guard = 0.5;
        let at = V2::new(4, 6);
        w.grid.terrain[(at.y * GRID_W + at.x) as usize] = wall;
        assert!(cover_for(&w.grid, HERO_AT, V2::new(5, 6), true) > 0.0, "FOE3 in cover against the hero");
        assert_eq!(cover_for(&w.grid, HERO_AT, V2::new(5, 2), true), 0.0, "FOE not in cover");
        if flush {
            w.profiles.get_mut(&HERO).expect("profile").tactics.push(UnitTactic::FlushCover);
        }
        let p = profile(&w);
        let (row, pick, _) = ai_round(&mut w, &p);
        assert_eq!(row, ai::ROW_FIELD);
        marked(&w, &pick.unwrap())
    };
    let plain = play(false);
    assert!(plain.contains(&V2::new(5, 2)) && !plain.contains(&V2::new(5, 6)), "without the tactic: the lower cell index ({plain:?})");
    let flushed = play(true);
    assert!(flushed.contains(&V2::new(5, 6)), "flush_cover: the covered foe counts twice ({flushed:?})");
}
