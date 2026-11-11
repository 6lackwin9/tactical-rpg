//! U1.2-S3b (docs/sessions/U1_STATS.md "status potency from KNOW, against resist"; S1_STATUS.md section 2) on a scratch
//! catalog built here (no shipped data): a skill's status roll adds the attacker's term `know_potency x KNOW` (the
//! ruleset number, default 0 = "potency off") to its potency before INT and status resist:
//! `chance = clamp(round(potency + know_potency x KNOW - INT x 2.5 - resist_for(status)), 0, 100)`. The actor paths
//! (a single-target skill status, an area skill status, the Dojo dummy) read it, shown and rolled alike; grounds,
//! terrain hazards and the trigger `inflict` effect keep their plain potency; a status that is not `resistable` still
//! lands without a roll. Expectations come from that formula.

use aegis_core::rng::GodotRng;
use indexmap::IndexMap;
use rpgkit_battle::command::{CmdKind, Command};
use rpgkit_battle::dojo::DojoSession;
use rpgkit_battle::ground::Placed;
use rpgkit_battle::grid::{GRID_W, NO_CELL, V2};
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, GROUND_HAZARDS, TIMELINE, TRIGGERS};
use rpgkit_rules::stats::roundi;
use rpgkit_rules::{status, Unit};
use serde_json::{json, Value};
use std::sync::Arc;

/// LAYER-DOWN-2: the Dojo's cast is the units the catalog marks (`dojo`); the fixture's ids.
const HEROES: [&str; 2] = ["dojo_warrior", "dojo_ranger"];

/// Unit row `u` with its Dojo part (`dojo`: hero or dummy).
fn in_dojo(mut u: serde_json::Value, part: &str) -> serde_json::Value {
    u["dojo"] = serde_json::json!(part);
    u
}

const BLOCKS: i32 = 6;
const HP: i32 = 5000;
const HERO: UnitId = 0;
const FOE: UnitId = 1;
/// The built-in terrain index the scratch `mud` row overrides (as in `ground_hazards.rs`).
const MUD: usize = 8;
/// The foe's INT: resist INT x 2.5 = 100, so a potency of 100 alone has chance 0.
const FOE_INT: i32 = 40;
const POTENCY: i32 = 100;
/// The hero's KNOW for the "a weight lifts the chance to 100" checks (3 x 40 = 120 over a resist of 100).
const KEEN_KNOW: i32 = 40;

fn catalog() -> Catalog {
    let attack = |id: &str, extra: Value| {
        let mut row = json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.2, "reach_min": 1, "reach_max": 4, "target_mode": "ENEMY"});
        row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        row
    };
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        attack("k_hex", json!({"power_mult": 0.0, "status_effect": "weaken", "status_potency": POTENCY, "status_magnitude": 10})),
        attack("k_blast", json!({"area": "BURST", "status_effect": "weaken", "status_potency": POTENCY, "status_magnitude": 10})),
        attack("k_plain", json!({})),
    ];
    let unit = |id: &str, side: &str, speed: i32| {
        json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": HP, "power": 20, "armor": 0, "speed": speed, "attack_range": 1, "skill_cap": 5})
    };
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("p_hero", "party", 14), unit("p_foe", "enemy", 10), in_dojo(unit(HEROES[0], "party", 12), "hero"), in_dojo(unit(HEROES[1], "party", 12), "hero"), in_dojo(unit("training_dummy", "enemy", 10), "dummy")],
        "hazards": [{"id": "k_hex_ground", "name": "Hex Ground", "on": "enter", "status": "weaken", "potency": POTENCY, "magnitude": 10, "status_rounds": 1, "affects": "foes", "duration": 3}],
        "terrain": [{"id": "mud", "name": "Mud", "hazard_status": "weaken", "hazard_potency": POTENCY, "hazard_on": "enter"}]
    }))
}

fn set_stat(u: &mut Unit, stat: &str, v: i32) {
    u.stats.core.insert(stat.to_string(), v);
}

/// Timeline with grounds and triggers on, `know_potency` set; hero (4, 4), foe (8, 4) with INT 40, hero KNOW `know`.
fn session(know_potency: f64, know: i32) -> Session {
    let mut ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [GROUND_HAZARDS, TRIGGERS]}).as_object().unwrap());
    ruleset.numbers.know_potency = know_potency;
    let row = json!({"id": "k_fight", "name": "Fight", "party": ["p_hero"], "enemies": ["p_foe"], "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(4, 4)), (FOE, V2::new(8, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    set_stat(&mut s.w.units[HERO], "KNOW", know);
    set_stat(&mut s.w.units[FOE], "INT", FOE_INT);
    s
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn command(w: &World, c: UnitId, step: Step, skill: &str) -> Command {
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, c, step, true), "", "{skill}");
    let id = sid(w, skill);
    plan.entries_for(w, c).into_iter().find(|e| e.skill == id).expect("the skill's entry").build_command(w, c)
}

fn walk(c: UnitId, dirs: &[V2]) -> Command {
    let mut cmd = Command::new(c, 0, 1, CmdKind::Walk { path: dirs.to_vec(), expected_start: NO_CELL, goal: NO_CELL, approach: None, moved_tiles: 0, cut_short: false, blocked_by: String::new() });
    cmd.name = "Move".into();
    cmd
}

fn run_one(w: &mut World, c: UnitId, cmd: Command) -> Resolver {
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(c, cmd, 1), "booked");
    for i in res.prepare_block(w, 1) {
        res.execute_single(w, i);
    }
    res
}

fn weakened(u: &Unit) -> bool {
    u.st.has_any("weaken")
}

/// The formula's chance (unclamped part) for the scratch numbers.
fn expected(know_potency: f64, know: i32, resist: f64) -> i32 {
    roundi(POTENCY as f64 + know_potency * know as f64 - resist).clamp(0, 100)
}

#[test]
fn the_term_is_weight_times_know_and_zero_changes_nothing() {
    let s = session(0.0, 10);
    let (hero, foe) = (&s.w.units[HERO], &s.w.units[FOE]);
    let resist = foe.debuff_resistance() + foe.resist_for("weaken") as f64;
    assert_eq!(resist, FOE_INT as f64 * 2.5);
    for potency in [0, 40, 100, 160] {
        assert_eq!(status::apply_chance_with(foe, "weaken", potency, status::know_bonus(hero, 0.0)), status::apply_chance(foe, "weaken", potency), "weight 0 = potency off");
    }
    for (weight, know) in [(0.5, 10), (1.0, 13), (1.5, 10), (2.0, 21), (3.0, 7)] {
        let mut h = hero.clone();
        set_stat(&mut h, "KNOW", know);
        let bonus = status::know_bonus(&h, weight);
        assert_eq!(bonus, weight * know as f64, "weight {weight} x KNOW {know}");
        let got = status::apply_chance_with(foe, "weaken", POTENCY, bonus);
        assert_eq!(got, expected(weight, know, resist), "weight {weight}, KNOW {know}");
        assert_eq!(got - status::apply_chance(foe, "weaken", POTENCY), roundi(weight * know as f64), "the chance rises by weight x KNOW");
    }
}

#[test]
fn a_status_that_is_not_resistable_still_skips_the_roll() {
    let s = session(3.0, KEEN_KNOW);
    let (hero, mut foe) = (s.w.units[HERO].clone(), s.w.units[FOE].clone());
    assert!(!status::resistable("taunt"));
    let bonus = status::know_bonus(&hero, 3.0);
    assert_eq!(status::apply_chance_with(&foe, "taunt", 0, bonus), 100);
    let mut rng = GodotRng::new(7);
    let mut probe = rng.clone();
    assert!(status::try_inflict_with(&mut foe, "taunt", 0, bonus, Some(&mut rng), 1, 0, "p_hero"));
    assert_eq!(rng.randi(), probe.randi(), "no draw");
}

#[test]
fn a_single_target_skill_status_reads_the_actor_term() {
    // weight 0: chance 0, resisted; weight 3 x KNOW 40: chance 100, lands (whatever the draw)
    for (weight, lands) in [(0.0, false), (3.0, true)] {
        assert_eq!(expected(weight, KEEN_KNOW, FOE_INT as f64 * 2.5), if lands { 100 } else { 0 });
        let s = session(weight, KEEN_KNOW);
        let mut w = s.w.clone();
        let cmd = command(&w, HERO, PlanStep::engage(sid(&w, "k_hex"), FOE), "k_hex");
        let res = run_one(&mut w, HERO, cmd);
        assert_eq!(weakened(&w.units[FOE]), lands, "weight {weight}: {:?}", res.log);
    }
}

#[test]
fn an_area_skill_status_shows_and_rolls_the_actor_term() {
    for (weight, know) in [(0.0, 10), (1.5, 10), (3.0, KEEN_KNOW)] {
        let chance = expected(weight, know, FOE_INT as f64 * 2.5);
        let s = session(weight, know);
        let mut w = s.w.clone();
        let cmd = command(&w, HERO, PlanStep::area_at(sid(&w, "k_blast"), V2::new(8, 4)), "k_blast");
        let res = run_one(&mut w, HERO, cmd);
        let line = res.log.iter().find(|l| l.to_lowercase().contains("weaken")).unwrap_or_else(|| panic!("the status note: {:?}", res.log));
        assert!(line.contains(&format!("({chance}%)")), "weight {weight}, KNOW {know}: shows {chance}%: {line}");
        if chance == 0 || chance == 100 {
            assert_eq!(weakened(&w.units[FOE]), chance == 100, "{line}");
        }
    }
}

#[test]
fn the_dojo_dummy_shows_and_rolls_the_actor_term() {
    for (weight, know) in [(0.0, 10), (1.5, 10), (3.0, KEEN_KNOW)] {
        let mut d = DojoSession::new(Arc::new(catalog()));
        d.ruleset.numbers.know_potency = weight;
        let active = d.active;
        set_stat(&mut d.heroes[active], "KNOW", know);
        set_stat(&mut d.dummy, "INT", FOE_INT);
        let resist = d.dummy.debuff_resistance() + d.dummy.resist_for("weaken") as f64;
        let chance = roundi(POTENCY as f64 + d.hero().status_potency_bonus() as f64 + weight * know as f64 - resist).clamp(0, 100);
        d.use_skill(Some("k_hex"));
        let e = d.log.iter().rev().find(|e| e.title == "k_hex").expect("the hit's entry");
        assert!(e.detail.contains(&format!("({chance}%)")), "weight {weight}, KNOW {know}: shows {chance}%: {}", e.detail);
        if chance == 0 || chance == 100 {
            assert_eq!(weakened(&d.dummy), chance == 100, "{}", e.detail);
        }
    }
}

#[test]
fn grounds_terrain_and_trigger_inflicts_keep_the_plain_potency() {
    // weight 3 x KNOW 40 would lift any of these to 100; with the plain potency (100 against resist 100) none lands
    let s = session(3.0, KEEN_KNOW);

    // a ground the keen hero laid, then mud: the foe walks (8, 4) -> (7, 4) ground -> (6, 4) mud
    let mut w = s.w.clone();
    let def = w.cat.hazards["k_hex_ground"].clone();
    w.grid.ground.insert(V2::new(7, 4), Placed { def, by: HERO, round: 1, last_round: 3 });
    w.grid.terrain[(4 * GRID_W + 6) as usize] = MUD;
    let res = run_one(&mut w, FOE, walk(FOE, &[V2::new(-1, 0), V2::new(-1, 0)]));
    assert_eq!(w.pos(FOE), V2::new(6, 4), "{:?}", res.log);
    assert!(!weakened(&w.units[FOE]), "ground and terrain: plain potency: {:?}", res.log);

    // the keen hero's trigger `inflict` on a hit dealt
    let mut w = s.w.clone();
    w.grid.positions[FOE] = Some(V2::new(5, 4));
    let cat = w.cat.clone();
    let e = json!({"type": "trigger", "on": "hit_dealt", "when": {}, "effects": [{"type": "inflict", "status": "weaken", "potency": POTENCY, "magnitude": 10, "to": "other"}], "limit": ""});
    w.units[HERO].apply_effects(&cat, &[e], "passive:k_curse", false);
    let cmd = command(&w, HERO, PlanStep::engage(sid(&w, "k_plain"), FOE), "k_plain");
    let res = run_one(&mut w, HERO, cmd);
    assert!(res.triggers.fired.iter().any(|f| f.holder == HERO && f.on == "hit_dealt"), "the trigger fired: {:?}", res.log);
    assert!(!weakened(&w.units[FOE]), "trigger inflict: plain potency: {:?}", res.log);
}
