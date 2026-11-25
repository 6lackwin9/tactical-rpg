//! W2-5 I1.2-S2 (docs/sessions/I_AI.md section 1 item 5, FB-52b) on a scratch catalog built here (no shipped data): the
//! classic AI's generic rating-modifier scorer behind the ruleset switch `ai_rating_scorer` (a plain tunable, default
//! off). Off: only the Armor and Move Rate debuffs and the Armor buff have target orders (I1.2-S1, 1-to-1), so `weaken`,
//! `empower` and `haste` take the generic orders. On: a `weaken` (lowers Attack Power) goes first to the foes that
//! threaten a friend this round, hardest hitter first; an `empower` (raises Attack Power) to the friends that can reach
//! a foe this round, hardest hitter first; a `haste` (raises Move Rate) to the friends with no foe in reach, nearest to
//! a foe first. Expectations are derived from the geometry and the unit stats set here.

use indexmap::IndexMap;
use rpgkit_battle::ai;
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::Plan;
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{self, Ruleset, CLASSIC};
use rpgkit_rules::status::{self, Rating};
use serde_json::json;
use std::sync::Arc;

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const ALLY2: UnitId = 2;
const WEAK: UnitId = 3;
const STRONG: UnitId = 4;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "t_weaken", "name": "Weaken", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.5, "status_effect": "weaken", "status_potency": 100, "status_rounds": 2}),
        json!({"id": "t_expose", "name": "Expose", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.5, "status_effect": "expose", "status_potency": 100, "status_rounds": 2}),
        json!({"id": "t_empower", "name": "Empower", "kind": "SUPPORT", "level": 1, "blocks": 1, "grant_status": "empower", "status_rounds": 2, "target_mode": "ALLY_OR_SELF"}),
        json!({"id": "t_haste", "name": "Haste", "kind": "SUPPORT", "level": 1, "blocks": 1, "grant_status": "haste", "status_rounds": 2, "target_mode": "ALLY_OR_SELF"}),
    ];
    let unit = |id: &str, side: &str, power: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 500, "power": power, "armor": 0, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("p_hero", "party", 20), unit("p_ally", "party", 50), unit("p_ally2", "party", 30), unit("e_weak", "enemy", 10), unit("e_strong", "enemy", 40)]
    }))
}

fn rules(scorer: Option<&str>) -> Ruleset {
    let mut row = json!({"id": CLASSIC, "name": "Classic", "rules": []});
    if let Some(v) = scorer {
        row["ai_rating_scorer"] = json!(v);
    }
    Ruleset::from_row(row.as_object().unwrap())
}

/// HERO (0, 2), ALLY (0, 8), ALLY2 (5, 4); WEAK (6, 4) beside ALLY2, STRONG (11, 0) far from every friend.
fn session(scorer: Option<&str>) -> World {
    let row = json!({"id": "s_fight", "name": "Fight", "party": ["p_hero", "p_ally", "p_ally2"], "enemies": ["e_weak", "e_strong"], "total_blocks": 6, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = rules(scorer);
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(0, 2)), (ALLY, V2::new(0, 8)), (ALLY2, V2::new(5, 4)), (WEAK, V2::new(6, 4)), (STRONG, V2::new(11, 0))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s.w.clone()
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

/// The tiles a unit covers this round: its round's move tiles plus its basic reach (attack range 1 here).
fn round_reach(w: &World, plan: &Plan, c: UnitId) -> i32 {
    plan.max_move_tiles(w, c) + w.units[c].attack_range
}

fn targets(scorer: Option<&str>, skill: &str) -> Vec<UnitId> {
    let w = session(scorer);
    let plan = Plan::new();
    ai::status_targets(&w, &plan, HERO, sid(&w, skill))
}

/// The switch: a row-only plain tunable, off unless the row says `on`, listed in the field rules with on / off.
#[test]
fn the_switch_is_a_plain_tunable_default_off() {
    assert!(!rules(None).numbers.ai_rating_scorer);
    assert!(!rules(Some("off")).numbers.ai_rating_scorer);
    assert!(rules(Some("on")).numbers.ai_rating_scorer);
    assert_eq!(rules(None).numbers.field("ai_rating_scorer").as_deref(), Some("off"));
    let rule = ruleset::field_rule_of("ai_rating_scorer").expect("a field rule");
    assert!(rule.values.contains(&"on") && rule.values.contains(&"off"));
    // timeline rules do not switch it on by themselves (no rule-driven default)
    let tl = Ruleset::from_row(json!({"id": "timeline", "rules": ["round_intent", "field_intent", "ai_tactics"]}).as_object().unwrap());
    assert!(!tl.numbers.ai_rating_scorer);
    let w = session(Some("on"));
    assert!(ai::rating_scorer(&w));
    assert!(!ai::rating_scorer(&session(None)));
}

/// The statuses the scorer opens up are the table's Attack Power and Move Rate modifier rows.
#[test]
fn the_new_reads_are_table_predicates() {
    assert!(status::lowers_rating("weaken", Rating::AttackPower));
    assert!(status::raises_rating("empower", Rating::AttackPower));
    assert!(status::raises_rating("haste", Rating::MoveRate));
}

/// `weaken`: off, the foes by power (STRONG before WEAK); on, the foe threatening a friend first.
#[test]
fn weaken_goes_to_the_threatening_foe_first_only_when_on() {
    let w = session(None);
    let plan = Plan::new();
    // STRONG hits harder; only WEAK reaches a friend this round
    assert!(rpgkit_rules::damage::power_of(&w.units[STRONG], false, false) > rpgkit_rules::damage::power_of(&w.units[WEAK], false, false));
    let friends = [HERO, ALLY, ALLY2];
    assert!(friends.iter().any(|&f| w.pos(WEAK).dist(w.pos(f)) <= round_reach(&w, &plan, WEAK)));
    assert!(friends.iter().all(|&f| w.pos(STRONG).dist(w.pos(f)) > round_reach(&w, &plan, STRONG)));
    assert_eq!(targets(None, "t_weaken"), [STRONG, WEAK], "off: the generic order (by power)");
    assert_eq!(targets(Some("on"), "t_weaken"), [WEAK, STRONG], "on: the threat first");
    // the Armor debuff keeps its own order either way (I1.2-S1)
    assert_eq!(targets(None, "t_expose"), targets(Some("on"), "t_expose"));
}

/// `empower`: off, the generic buff order (friends with no foe near, farthest first); on, the friends that reach a
/// foe this round, hardest hitter first.
#[test]
fn empower_goes_to_the_friend_in_the_fight_only_when_on() {
    let w = session(None);
    let plan = Plan::new();
    let reaches = |f: UnitId| [WEAK, STRONG].iter().any(|&e| w.pos(e).dist(w.pos(f)) <= round_reach(&w, &plan, f));
    assert!(reaches(ALLY2) && !reaches(HERO) && !reaches(ALLY), "only ALLY2 reaches a foe this round");
    assert_eq!(targets(Some("on"), "t_empower"), [ALLY2]);
    let off = targets(None, "t_empower");
    assert!(!off.contains(&ALLY2) && off.contains(&HERO) && off.contains(&ALLY), "off: the generic order skips the friend beside a foe ({off:?})");
}

/// `haste`: on, the friends with no foe in reach, the one nearest a foe first (HERO 8 tiles, ALLY 10); off, the
/// generic order puts the farthest first.
#[test]
fn haste_goes_to_the_friend_nearest_the_fight_when_on() {
    let w = session(None);
    let near = |f: UnitId| [WEAK, STRONG].iter().map(|&e| w.pos(e).dist(w.pos(f))).min().unwrap();
    assert!(near(HERO) < near(ALLY) && near(ALLY2) <= 1);
    assert_eq!(targets(Some("on"), "t_haste"), [HERO, ALLY]);
    assert_eq!(targets(None, "t_haste"), [ALLY, HERO]);
}
