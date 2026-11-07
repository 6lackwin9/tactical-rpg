//! A3.2-K2 (`docs/sessions/A3_SHAPES.md` section 1, INTENT2 IR9) on a scratch catalog built here (no shipped data):
//! the skill field `affects` (all, enemies, allies) picks which units an area hits; omitted, the row decides: a skill
//! that leaves a hazard hits all (its tiles are marked), an Assault or Stand (`Guard`) row enemies, a Support row
//! allies, any other row all; a battle without round intents (classic) has no rows and hits all, as before K2. The user
//! is never hit. IR9's test: an ally beside the target is unhurt by the Assault row and hit when the area hits all.

use indexmap::IndexMap;
use rpgkit_battle::command::Command;
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::round_intent::{self, RoundIntent};
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::skill;
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::{Affects, Catalog, SkillDef};
use rpgkit_rules::ruleset::{Ruleset, GROUND_HAZARDS, ROUND_INTENT, TIMELINE};
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HP: i32 = 5000;
const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
/// The BURST aim: the foe's tile; the ally stands beside it, inside the 3 x 3.
const FOE_AT: V2 = V2::new(5, 4);
const ALLY_AT: V2 = V2::new(5, 5);
const HERO_AT: V2 = V2::new(4, 4);

/// A BURST blast (reach 1-2) with no `affects`, the same with `all`, `allies`, and one leaving a fire hazard.
fn catalog() -> Catalog {
    let blast = |id: &str, extra: Value| {
        let mut row = json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.5, "reach_min": 1, "reach_max": 2, "area": "BURST"});
        row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        row
    };
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        blast("a_blast", json!({})),
        blast("a_blast_all", json!({"affects": "all"})),
        blast("a_blast_allies", json!({"affects": "allies"})),
        blast("a_fire_blast", json!({"hazard": {"id": "a_fire", "shape": "SKILL"}})),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": HP, "power": 20, "armor": 0, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("p_hero", "party"), unit("p_ally", "party"), unit("p_foe", "enemy")],
        "hazards": [{"id": "a_fire", "name": "Fire", "on": "round_end", "damage": 1, "duration": 1}]
    }))
}

fn timeline(rules: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules}).as_object().unwrap())
}

/// Hero (4, 4), its ally (5, 5) beside the foe (5, 4).
fn session(ruleset: Ruleset) -> Session {
    let row = json!({"id": "a_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe"], "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, HERO_AT), (ALLY, ALLY_AT), (FOE, FOE_AT)] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn def(w: &World, id: &str) -> SkillDef {
    w.cat.skills[sid(w, id)].clone()
}

fn blast(w: &World, skill: &str) -> Command {
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, HERO, PlanStep::area_at(sid(w, skill), FOE_AT), true), "", "{skill}");
    let id = sid(w, skill);
    plan.entries_for(w, HERO).into_iter().find(|e| e.skill == id).expect("the skill's entry").build_command(w, HERO)
}

/// The blast resolved in block 1: (the ally hurt, the foe hurt, the hero hurt).
fn hurt(mut w: World, skill: &str) -> (bool, bool, bool) {
    let cmd = blast(&w, skill);
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(HERO, cmd, 1), "booked");
    for i in res.prepare_block(&mut w, 1) {
        res.execute_single(&mut w, i);
    }
    (w.units[ALLY].hp < HP, w.units[FOE].hp < HP, w.units[HERO].hp < HP)
}

fn with_intent(ruleset: Ruleset, intent: RoundIntent) -> World {
    let mut w = session(ruleset).w.clone();
    round_intent::set_intent(&mut w, HERO, intent).expect("intent");
    w
}

#[test]
fn the_field_reads_its_three_values_and_blank_is_the_row_default() {
    let w = session(Ruleset::classic()).w.clone();
    assert_eq!(def(&w, "a_blast").affects, None, "omitted: the row's default");
    assert_eq!(def(&w, "a_blast_all").affects, Some(Affects::All));
    assert_eq!(def(&w, "a_blast_allies").affects, Some(Affects::Allies));
    for id in Affects::IDS {
        assert_eq!(Affects::parse(id).map(Affects::id), Some(id), "{id} round trips");
    }
    assert_eq!(Affects::parse("foes"), None, "only the three ids");
}

#[test]
fn the_default_follows_the_row() {
    let w = session(Ruleset::classic()).w.clone();
    let plain = def(&w, "a_blast");
    assert_eq!(skill::affects_by(&plain, None), Affects::All, "no rows (classic): all, as before");
    assert_eq!(skill::affects_by(&plain, Some(RoundIntent::Assault)), Affects::Enemies);
    assert_eq!(skill::affects_by(&plain, Some(RoundIntent::Guard)), Affects::Enemies, "Stand");
    assert_eq!(skill::affects_by(&plain, Some(RoundIntent::Support)), Affects::Allies);
    assert_eq!(skill::affects_by(&plain, Some(RoundIntent::Maneuver)), Affects::All, "a row IR9 does not name");
    for row in [None, Some(RoundIntent::Assault), Some(RoundIntent::Support)] {
        assert_eq!(skill::affects_by(&def(&w, "a_blast_all"), row), Affects::All, "the field wins in {row:?}");
        assert_eq!(skill::affects_by(&def(&w, "a_blast_allies"), row), Affects::Allies, "the field wins in {row:?}");
    }
    // a hazard is carried only with the rule on (the catalog strips it otherwise): its marked tiles hit all
    let g = session(timeline(&[GROUND_HAZARDS, ROUND_INTENT])).w.clone();
    assert_eq!(skill::affects_by(&def(&g, "a_fire_blast"), Some(RoundIntent::Assault)), Affects::All, "a ground hazard hits all");
    assert_eq!(skill::affects_by(&def(&w, "a_fire_blast"), Some(RoundIntent::Assault)), Affects::Enemies, "rule off: no hazard, the row decides");
}

#[test]
fn affects_unit_is_the_side_test() {
    for friend in [true, false] {
        assert!(skill::affects_unit(Affects::All, friend));
        assert_eq!(skill::affects_unit(Affects::Enemies, friend), !friend);
        assert_eq!(skill::affects_unit(Affects::Allies, friend), friend);
    }
}

/// IR9's test: the Assault row spares the ally beside the target; an area that hits all (classic, or `affects` all)
/// hurts it. The user inside its own burst is never hit.
#[test]
fn an_ally_beside_the_target_is_spared_by_the_assault_row_only() {
    assert_eq!(hurt(session(Ruleset::classic()).w.clone(), "a_blast"), (true, true, false), "classic: hits allies too");
    let on = || timeline(&[ROUND_INTENT]);
    assert_eq!(hurt(with_intent(on(), RoundIntent::Assault), "a_blast"), (false, true, false), "Assault row: foes only");
    assert_eq!(hurt(with_intent(on(), RoundIntent::Assault), "a_blast_all"), (true, true, false), "affects all: the ally in the marks is hit");
    assert_eq!(hurt(with_intent(on(), RoundIntent::Assault), "a_blast_allies"), (true, false, false), "affects allies: friends only");
}

/// The planner's estimate follows the same filter: the Assault row books no friend hit and the foe's damage; an
/// `allies` area books no foe damage.
#[test]
fn the_plan_estimate_reads_affects() {
    let entry = |w: &World, skill: &str| {
        let mut plan = Plan::new();
        assert_eq!(plan.add_step(w, HERO, PlanStep::area_at(sid(w, skill), FOE_AT), true), "");
        let id = sid(w, skill);
        let e = plan.entries_for(w, HERO).into_iter().find(|e| e.skill == id).expect("entry");
        (e.hits_allies, e.est_damage > 0)
    };
    assert_eq!(entry(&session(Ruleset::classic()).w, "a_blast"), (1, true), "classic: the ally counted");
    let w = with_intent(timeline(&[ROUND_INTENT]), RoundIntent::Assault);
    assert_eq!(entry(&w, "a_blast"), (0, true), "Assault: no friend hit");
    assert_eq!(entry(&w, "a_blast_allies"), (1, false), "allies: no foe damage booked");
}
