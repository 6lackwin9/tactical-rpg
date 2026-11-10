//! RULESET1-K (docs/specs/RULESET1.md 8, K1-K6) on a scratch catalog built here (no shipped data): under the rule
//! `reach` a melee unit whose main weapon's family has the engine rule `reach` attacks in the band (1, 2); its
//! `attack_range` stays 1; off-hand jabs, LINE and RING stay (1, 1) and a skill's own `reach_max` wins (K2); distance is
//! Manhattan with nothing in the way counted (K3); approaches, dashes and the chase stop at the band's far edge (K4);
//! riposte answers only at distance 1 (K5); a reach hit is a melee hit (no cover term, the same damage as from 1 tile,
//! K6); the band bounds "engaged" (RULESET1_INTENT MV2) and the threat measure (PL4). Classic, and the rule switched off,
//! keep today's bands exactly (`reach_by` = `reach_for`).

use indexmap::IndexMap;
use rpgkit_battle::command::cover_between;
use rpgkit_battle::grid::{V2, GRID_W};
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::round_intent::{self, RoundIntent, RoundTarget};
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::skill::{self, Kind};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::{Catalog, ItemDef};
use rpgkit_rules::gear::Equipment;
use rpgkit_rules::ruleset::{Ruleset, REACH, ROUND_INTENT, TIMELINE};
use rpgkit_rules::status;
use rpgkit_rules::unit::REACH_WEAPON_REACH;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;

const LANCER: UnitId = 0;
const SWORD: UnitId = 1;
const FOE: UnitId = 2;
const FOE_B: UnitId = 3;

/// A weapon row as the loader resolves it (`reach_rule` only for a reach family).
fn weapon(id: &str, family: &str, base_range: i32, reach: bool) -> Value {
    let mut row = json!({"id": id, "name": id, "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": family, "base_power": 20, "base_range": base_range, "two_handed": true});
    if reach {
        row[rpgkit_data::ITEM_REACH_RULE] = json!(true);
    }
    row
}

/// Engine skills, a plain cut, an off-hand jab, LINE and RING areas, a lunge with its own reach (1-3) and a dash;
/// a lancer with a reach weapon, a swordsman with a plain one, two foes.
fn catalog() -> Catalog {
    catalog_with(None)
}

/// `catalog` with the lancer's pike resolved against a family `range_band` of `reach_max` (PR5-28; None: no band).
fn catalog_with(reach_max: Option<i32>) -> Catalog {
    let mut pike = weapon("t_pike", "t_pole", 1, true);
    if let Some(m) = reach_max {
        pike[rpgkit_data::ITEM_FAMILY_REACH_MAX] = json!(m);
    }
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "t_cut", "name": "Cut", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0}),
        json!({"id": "t_jab", "name": "Jab", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.5, "weapon_hand": "off"}),
        json!({"id": "t_line", "name": "Pierce Line", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.8, "area": "LINE"}),
        json!({"id": "t_ring", "name": "Ring", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.8, "area": "RING"}),
        json!({"id": "t_lunge", "name": "Lunge", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0, "reach_min": 1, "reach_max": 3}),
        json!({"id": "t_rush", "name": "Rush", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0, "dash_tiles": 3}),
    ];
    let unit = |id: &str, side: &str, main: &str| {
        json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": 1, "skill_cap": 5, "main_hand": main})
    };
    Catalog::from_value(&json!({
        "skills": skills,
        "items": [pike, weapon("t_sabre", "t_blade", 1, false)],
        "units": [unit("t_lancer", "party", "t_pike"), unit("t_swordsman", "party", "t_sabre"), unit("t_foe", "enemy", ""), unit("t_foe_b", "enemy", "")]
    }))
}

fn ruleset(rules: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules}).as_object().unwrap())
}

/// The rule `reach` alone (an extension works without the round intent).
fn reach_on() -> Ruleset {
    ruleset(&[REACH])
}

/// Lancer, swordsman, foe, foe_b (unit ids 0..4), deployed, then moved to the corners (tests place them).
fn session(rules: Ruleset) -> Session {
    session_from(rules, catalog())
}

/// `session` over catalog `cat`.
fn session_from(rules: Ruleset, cat: Catalog) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["t_lancer", "t_swordsman"], "enemies": ["t_foe", "t_foe_b"], "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = rules;
    let mut s = Session::start(Arc::new(cat), def, &IndexMap::new());
    for (c, cell) in [(LANCER, V2::new(0, 0)), (SWORD, V2::new(0, 8)), (FOE, V2::new(11, 0)), (FOE_B, V2::new(11, 8))] {
        place(&mut s.w, c, cell);
    }
    s
}

fn place(w: &mut World, c: UnitId, cell: V2) {
    w.grid.positions[c] = Some(cell);
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn run(w: &mut World, res: &mut Resolver, from: i32, to: i32) {
    for b in from..=to {
        for i in res.prepare_block(w, b) {
            res.execute_single(w, i);
        }
    }
}

fn commit(w: &World, plan: &Plan) -> Resolver {
    let mut res = Resolver::new(BLOCKS, 7);
    plan.commit(w, &mut res);
    res
}

/// K1, K2: under the rule a reach weapon gives a plain melee attack the band (1, 2); `attack_range` stays 1 (melee:
/// no kiting, riposte and cover as melee); off-hand jabs, LINE and RING stay (1, 1); a skill's own `reach_max` wins;
/// a plain weapon, or a ranged unit holding the reach weapon, keeps today's band. Classic and the rule switched off:
/// `reach_by` is `reach_for` for every unit and skill.
#[test]
fn a_reach_weapon_widens_the_melee_band_under_the_rule() {
    let s = session(reach_on());
    let w = &s.w;
    let lancer = &w.units[LANCER];
    assert!(lancer.has_reach_weapon() && !w.units[SWORD].has_reach_weapon(), "the gear fact");
    assert_eq!((lancer.attack_range, lancer.min_attack_range()), (1, 1), "K2: attack_range stays 1");
    let band = |id: &str, c: UnitId| skill::reach_by(&w.cat.skills[sid(w, id)], &w.units[c], &w.ruleset);
    assert_eq!(band("t_cut", LANCER), (1, REACH_WEAPON_REACH), "K1: melee reach 2");
    assert_eq!(band("strike", LANCER), (1, REACH_WEAPON_REACH));
    assert_eq!(band("t_rush", LANCER), (1, REACH_WEAPON_REACH), "a dash attack strikes from the band too");
    for id in ["t_jab", "t_line", "t_ring"] {
        assert_eq!(band(id, LANCER), (1, 1), "K2: {id} stays (1, 1)");
    }
    assert_eq!(band("t_lunge", LANCER), (1, 3), "the skill's own reach");
    assert_eq!(band("t_cut", SWORD), (1, 1), "no reach weapon");
    assert_eq!((skill::melee_reach(lancer, &w.ruleset), skill::melee_reach(&w.units[SWORD], &w.ruleset)), (REACH_WEAPON_REACH, 1));
    // a ranged unit with the same weapon is not melee: its ranged band, no melee reach
    let mut archer = lancer.clone();
    archer.attack_range = 3;
    let cut = &w.cat.skills[sid(w, "t_cut")];
    assert!(!archer.has_reach_weapon());
    assert_eq!(skill::reach_by(cut, &archer, &w.ruleset), skill::reach_for(cut, &archer));
    // the family rule only counts on a melee weapon (a reach row with base range 2 is a ranged weapon)
    let ranged_pole = ItemDef::from_obj(weapon("t_long", "t_pole", 2, true).as_object().unwrap());
    assert!(ranged_pole.reach_rule && !Equipment::new(&ranged_pole).has_reach());
    assert!(Equipment::new(&w.cat.items["t_pike"]).has_reach() && !Equipment::new(&w.cat.items["t_sabre"]).has_reach());
    // classic and the rule off: today's bands exactly
    for rules in [Ruleset::classic(), ruleset(&[ROUND_INTENT])] {
        for c in [LANCER, SWORD, FOE] {
            for s in &w.cat.skills {
                assert_eq!(skill::reach_by(s, &w.units[c], &rules), skill::reach_for(s, &w.units[c]), "{} by {c} in {:?}", s.id, rules.rules);
            }
            assert_eq!(skill::melee_reach(&w.units[c], &rules), 1);
            assert_eq!(skill::threat_reach(&w.units[c], &rules), w.units[c].attack_range.max(1));
        }
    }
    let mut off = reach_on();
    off.adjust("-reach").unwrap();
    assert_eq!(skill::reach_by(cut, lancer, &off), (1, 1), "switched off by the Arena or the sim");
    // K6: the unit's plain attack band (the battle screen's reach band, "Reach 1-2") is a plain skill's `reach_by`
    let strike = &w.cat.skills[sid(w, "strike")];
    for rules in [reach_on(), Ruleset::classic(), off.clone()] {
        for u in [lancer, &w.units[SWORD], &archer] {
            assert_eq!(skill::attack_band(u, &rules), skill::reach_by(strike, u, &rules), "{} in {:?}", u.name, rules.rules);
        }
    }
    assert_eq!(skill::attack_band(lancer, &w.ruleset), (1, REACH_WEAPON_REACH), "the hover card's Reach 1-2");
    assert_eq!(skill::attack_band(&archer, &w.ruleset), (2, 3.max(archer.attack_range)), "a ranged band");
}

/// K2, K3: from 2 tiles the lancer plans the strike in place and it lands, over a unit or a wall in between (no line
/// of sight); classic plans an approach first and the same strike from 2 tiles would be out of range.
#[test]
fn a_reach_strike_lands_from_two_tiles_over_what_stands_between() {
    for between in ["unit", "wall"] {
        for (rules, on) in [(reach_on(), true), (Ruleset::classic(), false)] {
            let mut s = session(rules);
            let w = &mut s.w;
            place(w, LANCER, V2::new(3, 4));
            place(w, FOE, V2::new(5, 4));
            let mid = V2::new(4, 4);
            if between == "unit" {
                place(w, SWORD, mid);
            } else {
                w.grid.terrain[(mid.y * GRID_W + mid.x) as usize] = rpgkit_battle::grid::TERRAIN_IDS.iter().position(|t| *t == "wall").unwrap();
                assert!(!w.grid.is_walkable(mid), "a wall");
            }
            let mut plan = Plan::new();
            let p = plan.add_step(w, LANCER, PlanStep::engage(sid(w, "t_cut"), FOE), false);
            let entries = plan.entries_for(w, LANCER);
            let moves = entries.iter().filter(|e| skill::kind(&w.cat.skills[e.skill]) == Kind::Move).count();
            if on {
                assert_eq!((p.as_str(), moves, entries.len()), ("", 0, 1), "{between}: the strike in place");
                let hp = w.units[FOE].hp;
                let mut res = commit(w, &plan);
                run(w, &mut res, 1, 1);
                assert!(w.units[FOE].hp < hp, "{between}: it lands over what stands between");
                assert_eq!(w.pos(LANCER), V2::new(3, 4), "no step taken");
            } else {
                assert!(!p.is_empty() || moves > 0, "{between}: classic does not strike from 2 tiles ({p})");
            }
        }
    }
}

/// K4: an approach and a dash end on the band's far edge (2 tiles from the foe under the rule, 1 in classic); a dash
/// already in band is refused as today ("Already in reach").
#[test]
fn approaches_and_dashes_stop_at_the_band_edge() {
    for (rules, on) in [(reach_on(), true), (Ruleset::classic(), false)] {
        let edge = if on { REACH_WEAPON_REACH } else { 1 };
        // the approach: foe 5 tiles away, the approach covers d - edge tiles
        let mut s = session(rules.clone());
        let w = &mut s.w;
        let foe_at = V2::new(5, 4);
        place(w, LANCER, V2::new(0, 4));
        place(w, FOE, foe_at);
        let mut plan = Plan::new();
        assert_eq!(plan.add_step(w, LANCER, PlanStep::engage(sid(w, "t_cut"), FOE), false), "");
        let entries = plan.entries_for(w, LANCER);
        let tiles: i32 = entries.iter().filter(|e| skill::kind(&w.cat.skills[e.skill]) == Kind::Move).map(|e| e.tiles).sum();
        assert_eq!(tiles, V2::new(0, 4).dist(foe_at) - edge, "on {on}: approach to the band edge");
        assert_eq!(entries.last().unwrap().pos_after.dist(foe_at), edge);
        // the dash: foe 4 tiles away, the dash ends on the band edge
        let mut s = session(rules.clone());
        let w = &mut s.w;
        place(w, LANCER, V2::new(1, 4));
        place(w, FOE, foe_at);
        let mut plan = Plan::new();
        assert_eq!(plan.add_step(w, LANCER, PlanStep::engage(sid(w, "t_rush"), FOE), false), "");
        let e = plan.entries_for(w, LANCER).pop().unwrap();
        assert_eq!((e.pos_after.dist(foe_at), e.path.len() as i32), (edge, V2::new(1, 4).dist(foe_at) - edge), "on {on}: dash to the band edge");
        let mut res = commit(w, &plan);
        run(w, &mut res, 1, 1);
        assert_eq!(w.pos(LANCER).dist(foe_at), edge, "on {on}: the run dash too");
    }
    // in band from 2 tiles: the dash is refused, as at 1 tile today
    let mut s = session(reach_on());
    let w = &mut s.w;
    place(w, LANCER, V2::new(3, 4));
    place(w, FOE, V2::new(5, 4));
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, LANCER, PlanStep::engage(sid(w, "t_rush"), FOE), false), "Already in reach.");
}

/// K4: the chase (`follow_tiles`, one tile) steps only while the target is out of band. Planned from distance `d`,
/// the foe then slips one tile away or not: under the rule a foe at 2 (planned there, or slipped there from 1) is hit
/// without a step and one that slipped to 3 is followed one tile back to 2; classic follows a foe that slipped from 1
/// to 2. Every case ends in a hit from the band.
#[test]
fn the_chase_steps_only_out_of_band() {
    for (d, slip, rules, on) in [(2, false, reach_on(), true), (1, true, reach_on(), true), (2, true, reach_on(), true), (1, true, Ruleset::classic(), false)] {
        let mut s = session(rules);
        let w = &mut s.w;
        let edge = if on { REACH_WEAPON_REACH } else { 1 };
        let start = V2::new(3, 4);
        let foe_at = start + V2::new(d, 0);
        place(w, LANCER, start);
        place(w, FOE, foe_at);
        let mut plan = Plan::new();
        assert_eq!(plan.add_step(w, LANCER, PlanStep::engage(sid(w, "t_cut"), FOE), false), "");
        assert!(plan.entries_for(w, LANCER)[0].chase, "an unmoved strike chases");
        let mut res = commit(w, &plan);
        if slip {
            place(w, FOE, foe_at + V2::new(1, 0));
        }
        let before = w.pos(LANCER).dist(w.pos(FOE));
        let hp = w.units[FOE].hp;
        run(w, &mut res, 1, 1);
        let stepped = start.dist(w.pos(LANCER));
        assert_eq!(stepped, (before - edge).max(0), "d {d}, slip {slip}, rule {on}: a step only out of band");
        assert_eq!(w.pos(LANCER).dist(w.pos(FOE)), before.min(edge));
        assert!(w.units[FOE].hp < hp, "d {d}, slip {slip}, rule {on}: the hit lands");
    }
}

/// K5, K6: a riposting foe answers the lancer's strike from 1 tile, never from 2; the strike deals the same damage from
/// either (a melee hit: no ranged or cover term), and the cover between is 0.
#[test]
fn riposte_answers_only_at_one_tile_and_a_reach_hit_is_a_melee_hit() {
    let mut dealt = Vec::new();
    for d in [1, 2] {
        let mut s = session(reach_on());
        let w = &mut s.w;
        place(w, LANCER, V2::new(5 - d, 4));
        place(w, FOE, V2::new(5, 4));
        status::grant_now(&mut w.units[FOE], "riposte", 2, 0, "test");
        assert_eq!(cover_between(w, LANCER, FOE, w.units[LANCER].attack_range > 1), 0.0, "K6: a melee hit takes no cover");
        let mut plan = Plan::new();
        assert_eq!(plan.add_step(w, LANCER, PlanStep::engage(sid(w, "t_cut"), FOE), false), "");
        let mut res = commit(w, &plan);
        let hp = w.units[FOE].hp;
        run(w, &mut res, 1, 1);
        let cmd = &res.scheduled[0].command;
        assert_eq!(cmd.reactions.is_empty(), d == 2, "K5: riposte at distance {d}");
        dealt.push(hp - w.units[FOE].hp);
    }
    assert!(dealt[0] > 0);
    assert_eq!(dealt[0], dealt[1], "K6: the same hit from 1 and 2 tiles");
}

/// MV2 and PL4 read the band: under the round intent a lancer whose target stands 2 tiles away is engaged with it
/// only with the rule `reach`; a tile at the lancer's 2-block move plus 2 counts it as a threat only with the rule.
#[test]
fn engaged_and_threat_use_the_melee_band() {
    for (rules, on) in [(ruleset(&[ROUND_INTENT, REACH]), true), (ruleset(&[ROUND_INTENT]), false)] {
        let mut s = session(rules);
        let w = &mut s.w;
        place(w, LANCER, V2::new(3, 4));
        place(w, FOE, V2::new(5, 4));
        round_intent::set_intent(w, LANCER, RoundIntent::Assault).expect("intent");
        round_intent::set_target(w, LANCER, RoundTarget::Unit(FOE)).expect("target");
        assert_eq!(round_intent::melee_reach(w, LANCER), if on { REACH_WEAPON_REACH } else { 1 });
        assert_eq!(round_intent::engaged_with(w, LANCER), on.then_some(FOE), "MV2 at distance 2, rule {on}");
        // the threat: lancer at (0,0), swordsman out of the way; a tile at move x 2 + melee reach 2
        place(w, LANCER, V2::new(0, 0));
        place(w, SWORD, V2::new(11, 8));
        let far = w.units[LANCER].move_per_block() * 2 + REACH_WEAPON_REACH;
        let cell = V2::new(far, 0);
        assert!(cell.dist(w.pos(SWORD)) > w.units[SWORD].move_per_block() * 2 + 1, "the swordsman does not reach it");
        assert_eq!(round_intent::threat(w, FOE, cell), on as usize, "PL4 threat, rule {on}");
    }
}

/// PR5-28 (U3 FB-36c): the reach band's far edge is the weapon family's `range_band.max` (the resolved row's
/// `ITEM_FAMILY_REACH_MAX`), `REACH_WEAPON_REACH` without one; it moves every reader of the band (a plain attack's band,
/// the melee reach, the threat measure, the hover card) and only under the rule; a skill's own `reach_max` still wins.
#[test]
fn the_reach_band_edge_is_the_family_range_band_max() {
    let fallback = session(reach_on());
    assert_eq!(fallback.w.units[LANCER].weapon_reach(), REACH_WEAPON_REACH, "no range_band: the fallback");
    for edge in [2, 3, 4] {
        let s = session_from(reach_on(), catalog_with(Some(edge)));
        let w = &s.w;
        let lancer = &w.units[LANCER];
        assert!(lancer.has_reach_weapon());
        assert_eq!(lancer.weapon_reach(), edge);
        let band = |id: &str, c: UnitId| skill::reach_by(&w.cat.skills[sid(w, id)], &w.units[c], &w.ruleset);
        assert_eq!(band("t_cut", LANCER), (1, edge), "a plain attack strikes 1 to the family's max");
        assert_eq!(band("strike", LANCER), (1, edge));
        assert_eq!(band("t_jab", LANCER), (1, 1), "an off-hand jab stays (1, 1)");
        assert_eq!(band("t_lunge", LANCER), (1, 3), "the skill's own reach wins");
        assert_eq!(band("t_cut", SWORD), (1, 1), "no reach weapon");
        assert_eq!(skill::melee_reach(lancer, &w.ruleset), edge);
        assert_eq!(skill::attack_band(lancer, &w.ruleset), (1, edge), "the hover card's band");
        assert_eq!(skill::threat_reach(lancer, &w.ruleset), edge.max(1));
        assert_eq!(round_intent::melee_reach(w, LANCER), edge);
        for rules in [Ruleset::classic(), ruleset(&[ROUND_INTENT])] {
            assert_eq!(skill::melee_reach(lancer, &rules), 1, "the band edge is read only under the rule");
            assert_eq!(skill::attack_band(lancer, &rules), skill::reach_for(&w.cat.skills[sid(w, "strike")], lancer));
        }
    }
}
