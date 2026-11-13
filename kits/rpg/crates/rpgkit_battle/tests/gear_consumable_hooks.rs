//! ACT1-HOOKS2 item 3 (`FB-244` (4-6)) on a scratch catalog built here (no shipped data). Expectations come from the
//! rules:
//!
//! - `skill_block_cost` {skill, delta} on a worn item: the skill takes `blocks + sum(deltas)` blocks for its wearer,
//!   at least 1 (`Unit::skill_blocks`); several items' deltas on one skill add up before the clamp; the unit's planner
//!   entry, the built command, the resolver's impact block and the skill text all read the same number; another unit
//!   and the item taken off read the row's own blocks;
//! - a consumable's `meter_delta` {meter, delta} moves its target's meter by the delta, clamped to the gauge's floor
//!   (rule unit_meters), logged as a `[Meter]` line; a target without that meter (or a battle without the rule) is left
//!   as it is and the copy is still used;
//! - a consumable's `armor_piercing` leaves the armour step out of the item damage pipeline (guard and cover stay), in
//!   the battle and in the planner's estimate; without `item_mitigation` both items deal their flat damage.

use rpgkit_battle::command::{self, CmdKind, Command};
use rpgkit_battle::grid::V2;
use rpgkit_battle::meter;
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::session::{EncounterDef, Roster, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::{Catalog, ItemDef, PassiveDef, UnitDef};
use rpgkit_rules::damage::{self, ItemHit};
use rpgkit_rules::gear::{Equipment, Slot};
use rpgkit_rules::ruleset::{Ruleset, TIMELINE, TRIGGERS, UNIT_METERS};
use rpgkit_rules::Overrides;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const HP: i32 = 500;
/// The test meter's start (the hero holds it through passive `t_trait`); its floor is 0.
const START: i32 = 30;
const HEAVY: &str = "t_heavy";

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 1, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": HEAVY, "name": "Heavy", "kind": "ATTACK", "level": 1, "blocks": 4, "power_mult": 1.0, "target_mode": "ENEMY"}),
    ];
    let meters = vec![json!({"id": "t_gauge", "name": "Gauge", "short": "Ga", "min": 0, "max": 100, "start": START, "show": "bar"})];
    let unit = |id: &str, side: &str, speed: i32, armor: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": HP, "power": 100, "armor": armor, "speed": speed, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({
        "skills": skills,
        "unit_meters": meters,
        "units": [unit("t_hero", "party", 14, 0), unit("t_ally", "party", 12, 0), unit("t_foe", "enemy", 10, 40)],
    }))
}

/// The hero holds the test meter (passive `t_trait`); the ally and the foe hold none.
struct Holders;

impl Roster for Holders {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        cat.units.get(r).cloned()
    }

    fn overrides(&self, _cat: &Catalog, r: &str) -> Overrides {
        let passives = if r == "t_hero" { vec!["t_trait".to_string()] } else { Vec::new() };
        Overrides { passives, ..Overrides::default() }
    }
}

fn ruleset(rules: &[&str], item_mitigation: bool) -> Ruleset {
    let mut row = json!({"id": TIMELINE, "name": "Timeline", "rules": rules});
    if item_mitigation {
        row["item_mitigation"] = json!("on");
    }
    Ruleset::from_row(row.as_object().unwrap())
}

/// Hero (4,4), the ally below it (4,5), the foe beside it (5,4).
fn session(rules: Ruleset) -> World {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["t_hero", "t_ally"], "enemies": ["t_foe"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = rules;
    let mut cat = catalog();
    cat.passives.insert("t_trait".into(), PassiveDef { id: "t_trait".into(), name: "Trait".into(), effects: vec![json!({"type": "meter", "meter": "t_gauge"})], slot: String::new() });
    let mut s = Session::start(Arc::new(cat), def, &Holders);
    for (c, cell) in [(HERO, V2::new(4, 4)), (ALLY, V2::new(4, 5)), (FOE, V2::new(5, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s.w
}

fn metered() -> World {
    session(ruleset(&[TRIGGERS, UNIT_METERS], false))
}

fn def(row: Value) -> ItemDef {
    ItemDef::from_obj(row.as_object().unwrap())
}

/// A worn accessory whose only effect is `skill_block_cost` {HEAVY, delta}.
fn band(id: &str, delta: i32) -> ItemDef {
    def(json!({"id": id, "name": id, "item_type": "ACCESSORY", "slot": "ACCESSORY_1", "effects": [{"type": "skill_block_cost", "skill": HEAVY, "delta": delta}]}))
}

fn wear(w: &mut World, c: UnitId, items: &[(&ItemDef, Slot)]) {
    let cat = w.cat.clone();
    for (it, slot) in items {
        w.units[c].inventory.equip(Equipment::new(it), *slot);
    }
    w.units[c].sync_from_stats_and_gear(&cat);
}

fn take_off(w: &mut World, c: UnitId, slots: &[Slot]) {
    let cat = w.cat.clone();
    for slot in slots {
        w.units[c].inventory.unequip(*slot);
    }
    w.units[c].sync_from_stats_and_gear(&cat);
}

fn heavy(w: &World) -> rpgkit_data::SkillDef {
    w.cat.skill(HEAVY).expect("the heavy skill").clone()
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

/// `c` plans the heavy skill on the foe: its entry's blocks, and the command built from it.
fn planned_heavy(w: &World, c: UnitId) -> (i32, Command) {
    let mut plan = Plan::new();
    let step: Step = PlanStep::engage(sid(w, HEAVY), FOE);
    assert_eq!(plan.add_step(w, c, step, true), "", "the heavy step plans");
    let id = sid(w, HEAVY);
    let e = plan.entries_for(w, c).into_iter().find(|e| e.skill == id).expect("the heavy entry");
    (e.blocks(w), e.build_command(w, c))
}

/// The blocks every reader sees for the hero's heavy skill: the world's, the planner entry's, the built command's, the
/// resolver's (impact - start + 1) and the skill text's; asserts they agree and returns the number.
fn read_blocks(w: &World) -> i32 {
    let own = w.blocks_of(HERO, &heavy(w));
    let (entry, cmd) = planned_heavy(w, HERO);
    assert_eq!(entry, own, "the planner entry reads the unit's blocks");
    assert_eq!(cmd.block_cost, own, "the command reads them");
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(HERO, cmd, 1), "booked");
    let a = res.scheduled.last().unwrap();
    assert_eq!(a.impact - a.start + 1, own, "the resolver lands it after that many blocks");
    assert_eq!(w.units[HERO].skill_blocks(&heavy(w)), own);
    own
}

#[test]
fn a_worn_block_cost_shortens_the_skill_for_every_reader() {
    let mut w = metered();
    let base = heavy(&w).blocks;
    assert_eq!(read_blocks(&w), base, "unworn: the row's blocks");
    let ring = band("t_ring", -1);
    wear(&mut w, HERO, &[(&ring, Slot::Accessory1)]);
    assert_eq!(read_blocks(&w), base - 1, "one block faster while worn");
    assert_eq!(w.blocks_of(FOE, &heavy(&w)), base, "another unit reads the row's blocks");
    take_off(&mut w, HERO, &[Slot::Accessory1]);
    assert_eq!(read_blocks(&w), base, "gone with the item");
}

#[test]
fn deltas_on_one_skill_stack_then_clamp_at_one_block() {
    let mut w = metered();
    let base = heavy(&w).blocks;
    let (a, b, c) = (band("t_ring_a", -1), band("t_ring_b", -1), band("t_ring_c", -3));
    wear(&mut w, HERO, &[(&a, Slot::Accessory1), (&b, Slot::Accessory2)]);
    assert_eq!(read_blocks(&w), (base - 2).max(1), "two -1 items add up");
    take_off(&mut w, HERO, &[Slot::Accessory1]);
    wear(&mut w, HERO, &[(&c, Slot::Accessory1)]);
    assert!(base - 3 - 1 < 1, "the fixture's sum goes below one block");
    assert_eq!(read_blocks(&w), 1, "summed, then clamped to 1 block");
    let slower = band("t_weight", 2);
    take_off(&mut w, HERO, &[Slot::Accessory1, Slot::Accessory2]);
    wear(&mut w, HERO, &[(&slower, Slot::Accessory1)]);
    assert_eq!(read_blocks(&w), base + 2, "a positive delta slows it");
}

#[test]
fn a_passive_block_cost_counts_too() {
    let mut w = metered();
    let base = heavy(&w).blocks;
    let cat = w.cat.clone();
    let fx = json!({"type": "skill_block_cost", "skill": HEAVY, "delta": -1});
    w.units[HERO].apply_effects(&cat, &[fx], "passive:t_quick", false);
    let ring = band("t_ring", -1);
    wear(&mut w, HERO, &[(&ring, Slot::Accessory1)]);
    assert_eq!(read_blocks(&w), (base - 2).max(1), "the passive's and the item's deltas add up");
}

fn vial() -> ItemDef {
    def(json!({"id": "t_vial", "name": "Vial", "item_type": "CONSUMABLE", "consumable_type": "FOCUS_INCENSE", "use_blocks": 1, "use_reach": 0, "meter_delta": {"meter": "t_gauge", "delta": -25}}))
}

/// `c` uses `it` on `t` (one copy given first), alone in a fresh resolver; the command and the round's log.
fn use_item(w: &mut World, c: UnitId, t: UnitId, it: &ItemDef) -> (Command, Vec<String>) {
    w.units[c].inventory.pouch.push(it.clone());
    w.add_item_skills();
    let mut cmd = Command::new(c, 0, it.use_blocks.max(1), CmdKind::Item { target: t, item: Box::new(it.clone()), used: false, last_heal: 0, last_damage: 0, last_boost: 0.0, cell: None, struck_units: Vec::new() });
    cmd.name = it.name.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(c, cmd, 1), "booked");
    for b in 1..=BLOCKS {
        for i in res.prepare_block(w, b) {
            res.execute_single(w, i);
        }
    }
    (res.scheduled.pop().unwrap().command, res.log)
}

fn used(cmd: &Command) -> bool {
    matches!(cmd.kind, CmdKind::Item { used: true, .. })
}

#[test]
fn a_meter_delta_moves_the_users_meter_within_its_floor() {
    let mut w = metered();
    let v = vial();
    let d = v.meter_delta.clone().expect("the vial's meter change").delta;
    assert_eq!(meter::value(&w, HERO, "t_gauge"), Some(START));
    let (cmd, log) = use_item(&mut w, HERO, HERO, &v);
    assert!(used(&cmd), "{}", cmd.log);
    let once = (START + d).max(0);
    assert_eq!(meter::value(&w, HERO, "t_gauge"), Some(once), "{log:?}");
    assert!(log.iter().any(|l| l.starts_with("[Meter]") && l.contains(&format!("({START} -> {once})"))), "the meter line: {log:?}");
    let (_, log) = use_item(&mut w, HERO, HERO, &v);
    let twice = (once + d).max(0);
    assert!(once + d < 0, "the fixture's second use goes below the floor");
    assert_eq!(meter::value(&w, HERO, "t_gauge"), Some(twice), "clamped to the floor: {log:?}");
}

#[test]
fn a_meter_delta_on_a_unit_without_the_meter_does_nothing() {
    let mut w = metered();
    assert_eq!(meter::value(&w, ALLY, "t_gauge"), None, "the ally holds no meter");
    let hero_before = meter::value(&w, HERO, "t_gauge");
    let hp = w.units[ALLY].hp;
    let (cmd, log) = use_item(&mut w, ALLY, ALLY, &vial());
    assert!(used(&cmd), "the copy is used: {}", cmd.log);
    assert!(command::pouch_item(&w, ALLY, "t_vial").is_none(), "and spent");
    assert_eq!(meter::value(&w, ALLY, "t_gauge"), None);
    assert_eq!(meter::value(&w, HERO, "t_gauge"), hero_before, "nobody else's meter moves");
    assert_eq!(w.units[ALLY].hp, hp);
    assert!(!log.iter().any(|l| l.starts_with("[Meter]")), "no meter line: {log:?}");
    // without the rule the holder's use is a no-op too
    let mut w = session(ruleset(&[TRIGGERS], false));
    let (cmd, _) = use_item(&mut w, HERO, HERO, &vial());
    assert!(used(&cmd) && meter::value(&w, HERO, "t_gauge").is_none(), "{}", cmd.log);
}

fn dart(piercing: bool) -> ItemDef {
    def(json!({"id": if piercing { "t_dart_ap" } else { "t_dart" }, "name": "Dart", "item_type": "CONSUMABLE", "consumable_type": "THROWABLE_BOMB",
        "damage": 32, "use_blocks": 2, "use_reach": 3, "armor_piercing": piercing}))
}

/// The foe's HP lost to one `it` thrown by the hero, and the planner's estimate of it.
fn thrown(rules: Ruleset, it: &ItemDef) -> (i32, i32) {
    let mut w = session(rules);
    w.units[HERO].inventory.pouch.push(it.clone());
    w.add_item_skills();
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&w, HERO, PlanStep::use_item(&it.id, FOE), true), "", "the throw plans");
    let est = plan.entries_for(&w, HERO).into_iter().find(|e| e.item.is_some()).expect("the item entry").est_damage;
    w.units[HERO].inventory.pouch.clear();
    let before = w.units[FOE].hp;
    let (cmd, _) = use_item(&mut w, HERO, FOE, it);
    assert!(used(&cmd), "{}", cmd.log);
    (before - w.units[FOE].hp, est)
}

#[test]
fn an_armor_piercing_item_skips_the_armour_step() {
    let rules = || ruleset(&[], true);
    let w = session(rules());
    let plain = command::item_hit(&w, HERO, FOE, 1.0);
    let (guard, rating) = plain.mitigation.expect("item mitigation is on");
    assert!(rating > 0, "the foe is armoured");
    let (ap, plain_dart) = (dart(true), dart(false));
    assert_eq!(command::item_hit_of(&w, &ap, HERO, FOE, 1.0), ItemHit { armor_piercing: true, ..plain });
    let want_ap = damage::item_damage(ap.damage, ItemHit { armor_piercing: true, ..plain });
    let want_plain = damage::item_damage(plain_dart.damage, plain);
    assert_eq!(want_ap, damage::item_damage(ap.damage, ItemHit { splash: 1.0, mitigation: Some((guard, 0)), armor_piercing: false }), "as if unarmoured");
    assert!(want_ap > want_plain, "armour lowers the plain dart only");
    assert_eq!(thrown(rules(), &ap), (want_ap, want_ap), "battle and estimate");
    assert_eq!(thrown(rules(), &plain_dart), (want_plain, want_plain), "battle and estimate");
    let (_, trace) = damage::item_damage_traced(ap.damage, ItemHit { armor_piercing: true, ..plain });
    assert!(!format!("{trace:?}").contains("armour"), "no armour step: {trace:?}");
    let (_, trace) = damage::item_damage_traced(plain_dart.damage, plain);
    assert!(format!("{trace:?}").contains("armour"), "the plain dart's armour step: {trace:?}");
}

#[test]
fn without_item_mitigation_piercing_changes_nothing() {
    let (ap, plain) = (dart(true), dart(false));
    assert_eq!(thrown(ruleset(&[], false), &ap).0, ap.damage, "the flat damage");
    assert_eq!(thrown(ruleset(&[], false), &plain).0, plain.damage, "the flat damage");
}
