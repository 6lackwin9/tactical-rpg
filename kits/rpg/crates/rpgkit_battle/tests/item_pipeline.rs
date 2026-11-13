//! A1.2-K3 (`docs/sessions/A1_VALUE.md` section 1 and acceptance 1, FB-55, INTENT2 IR11, IR12, IR17, section 6) on a
//! scratch catalog built here (no shipped data): consumables through the rule engine. A damage item enters the damage
//! pipeline with its flat damage: without item mitigation (the default, classic) the value is exactly the old flat
//! damage, with it (`item_mitigation` on, a plain tunable) guard, cover and armour apply (`damage::item_damage`); a heal item runs the heal
//! pipeline (capped at the missing HP); a buff item grants `empower` now (`status::grant_now`, cap 50, so 50 % and
//! 30 % are not clipped) and changes no stat or max HP. IR12: a foe use whose foe left reach at impact spends its copy
//! under `thrown_miss_spends` (off: kept); a friend use never does; a hidden foe is not found (GD5, `evaded`) whatever
//! the spend switch, and an area item's Assault row whose target left reach spends too. Items derive their rows under `field_intent`
//! (heal Support + Stand, buff Stand, area bomb Field + Assault, single bomb Assault); an area item's Field row lands
//! its shape on a tile at full power (reach + 1, all units but the user), its Assault row is target-locked with
//! `splash_mult` off the target, foes only and `assault_shape_blocks` more wind-up; without `field_intent` an area item
//! hits its one target. AI (IR15, FB-121): under `field_intent` a hurt unit drinks its own heal in a Stand round.

use indexmap::IndexMap;
use rpgkit_battle::command::{self, CmdKind, Command};
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::round_intent::{self, RoundIntent, RoundTarget};
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::{Catalog, ItemDef};
use rpgkit_rules::damage::{self, ItemHit};
use rpgkit_rules::ruleset::{Ruleset, FIELD_INTENT, ROUND_INTENT, TIMELINE};
use rpgkit_rules::stats::roundi;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HP: i32 = 500;
const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const FOE_B: UnitId = 3;
const HERO_AT: V2 = V2 { x: 4, y: 4 };
const FOE_AT: V2 = V2 { x: 6, y: 4 };

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
    ];
    let unit = |id: &str, side: &str, speed: i32, extra: Value| {
        let mut row = json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": HP, "speed": speed, "attack_range": 1, "skill_cap": 5});
        row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        row
    };
    Catalog::from_value(&json!({
        "skills": skills,
        "terrain": [{"id": "rubble", "name": "Rubble", "cover_guard": 0.4}],
        "units": [unit("k_hero", "party", 14, json!({})), unit("k_ally", "party", 12, json!({})), unit("k_foe", "enemy", 10, json!({"armor": 25})), unit("k_foe_b", "enemy", 8, json!({}))],
    }))
}

fn item(extra: Value) -> ItemDef {
    let mut row = json!({"id": "k_item", "name": "Item", "item_type": "CONSUMABLE", "use_blocks": 1, "max_stack": 2});
    row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    ItemDef::from_obj(row.as_object().unwrap())
}

fn bomb(area: &str) -> ItemDef {
    item(json!({"id": format!("k_bomb_{}", area.to_lowercase()), "name": "Bomb", "consumable_type": "THROWABLE_BOMB", "damage": 40, "use_reach": 3, "area": area}))
}

fn potion() -> ItemDef {
    item(json!({"id": "k_potion", "name": "Potion", "consumable_type": "HEALTH_POTION", "heal": 90, "use_reach": 1}))
}

fn elixir(pct: f64) -> ItemDef {
    item(json!({"id": format!("k_elixir_{}", roundi(pct * 100.0)), "name": "Elixir", "consumable_type": "FOCUS_INCENSE", "buff_pct": pct, "duration": 2, "use_reach": 0}))
}

fn ruleset(rules: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules}).as_object().unwrap())
}

/// `ruleset` with the three item switches on (as the shipped timeline row sets them).
fn with_items(rules: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules, "item_mitigation": "on", "thrown_miss_spends": "on", "pouch_stacks": "on"}).as_object().unwrap())
}

/// Classic: no rule at all.
fn classic() -> Ruleset {
    ruleset(&[])
}

fn intent_only() -> Ruleset {
    with_items(&[ROUND_INTENT])
}

fn field_on() -> Ruleset {
    with_items(&[ROUND_INTENT, FIELD_INTENT])
}

/// Hero (4, 4), foe (6, 4); the ally at (7, 4) beside the foe, foe_b at (6, 5) below it.
fn session(rules: Ruleset) -> World {
    let row = json!({"id": "k_fight", "name": "Fight", "party": ["k_hero", "k_ally"], "enemies": ["k_foe", "k_foe_b"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = rules;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, HERO_AT), (ALLY, V2::new(7, 4)), (FOE, FOE_AT), (FOE_B, V2::new(6, 5))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s.w
}

fn give(w: &mut World, c: UnitId, it: &ItemDef, copies: usize) {
    for _ in 0..copies {
        w.units[c].inventory.pouch.push(it.clone());
    }
    w.add_item_skills();
}

fn use_on(c: UnitId, t: UnitId, it: &ItemDef) -> Command {
    let mut cmd = Command::new(c, 0, 1, CmdKind::Item { target: t, item: Box::new(it.clone()), used: false, last_heal: 0, last_damage: 0, last_boost: 0.0, cell: None, struck_units: Vec::new() });
    cmd.name = it.name.clone();
    cmd
}

fn run_one(w: &mut World, c: UnitId, cmd: Command) -> Command {
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(c, cmd, 1), "booked");
    for block in 1..=BLOCKS {
        for i in res.prepare_block(w, block) {
            res.execute_single(w, i);
        }
    }
    res.scheduled.pop().unwrap().command
}

fn used(cmd: &Command) -> bool {
    matches!(cmd.kind, CmdKind::Item { used: true, .. })
}

/// Plans `step` for `c` (its intent and target set first) and builds the item's command.
fn planned(w: &mut World, c: UnitId, intent: RoundIntent, target: RoundTarget, step: Step) -> Command {
    round_intent::set_intent(w, c, intent).unwrap();
    round_intent::set_target(w, c, target).unwrap();
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, c, step, true), "", "the item step plans");
    plan.entries_for(w, c).into_iter().find(|e| e.item.is_some()).expect("the item's entry").build_command(w, c)
}

/// Acceptance 1 (exact port): classic, no item mitigation: a bomb on an armoured, guarding foe deals its flat damage,
/// as `w.damage(t, damage, true)` did.
#[test]
fn classic_item_damage_is_the_flat_damage() {
    let mut w = session(classic());
    let b = bomb("NONE");
    give(&mut w, HERO, &b, 1);
    w.units[FOE].guard = 0.3;
    let before = w.units[FOE].hp;
    let cmd = run_one(&mut w, HERO, use_on(HERO, FOE, &b));
    assert!(used(&cmd), "{}", cmd.log);
    assert_eq!(before - w.units[FOE].hp, b.damage, "{}", cmd.log);
    assert_eq!(damage::item_damage(b.damage, ItemHit::default()), b.damage);
}

/// Acceptance 1 (mitigation): with `item_mitigation` on, the foe's guard and armour apply in the mitigation stage,
/// rounded once with a floor of 1; a row that lists `round_intent` but not the switch keeps the flat damage.
#[test]
fn item_damage_takes_guard_and_armour_under_the_rule() {
    let mut w = session(intent_only());
    assert!(w.ruleset.numbers.item_mitigation, "on as the row sets it");
    assert!(!ruleset(&[ROUND_INTENT, FIELD_INTENT]).numbers.item_mitigation, "off by default, whatever the rules");
    let b = bomb("NONE");
    give(&mut w, HERO, &b, 1);
    w.units[FOE].guard = 0.3;
    let rating = damage::armor_rating(&w.units[FOE]);
    assert!(rating > 0, "the foe is armoured");
    let want = roundi(b.damage as f64 * (1.0 - 0.3) * (1.0 - damage::armor_reduction(rating))).max(1);
    let before = w.units[FOE].hp;
    let cmd = run_one(&mut w, HERO, use_on(HERO, FOE, &b));
    assert_eq!(before - w.units[FOE].hp, want, "{}", cmd.log);
    assert!(want < b.damage);
    // the floor: a 1-damage item still deals 1 through a 90 % guard
    assert_eq!(damage::item_damage(1, ItemHit { splash: 1.0, mitigation: Some((0.9, 200)), armor_piercing: false }), 1);
}

/// A1 section 1: a heal item runs the heal pipeline, capped at the missing HP.
#[test]
fn a_heal_item_heals_up_to_the_missing_hp() {
    let mut w = session(intent_only());
    let p = potion();
    give(&mut w, HERO, &p, 1);
    w.units[ALLY].hp -= 40;
    w.grid.positions[ALLY] = Some(HERO_AT + V2::new(1, 0));
    let cmd = run_one(&mut w, HERO, use_on(HERO, ALLY, &p));
    assert_eq!(w.units[ALLY].hp, w.units[ALLY].max_hp, "{}", cmd.log);
    assert_eq!(damage::item_heal(p.heal), p.heal);
}

/// Acceptance 1 (buff), FB-55(1), C07: a buff item grants `empower` now at its percent (50 and 30 unclipped by the
/// raised cap); max HP, stats and the Focus boost are untouched.
#[test]
fn a_buff_item_grants_empower_now_and_keeps_max_hp() {
    for pct in [0.5, 0.3] {
        let mut w = session(classic());
        let e = elixir(pct);
        give(&mut w, HERO, &e, 1);
        let (max_hp, boost) = (w.units[HERO].max_hp, w.units[HERO].attack_boost);
        let cmd = run_one(&mut w, HERO, use_on(HERO, HERO, &e));
        let u = &w.units[HERO];
        assert!(u.has_status("empower"), "active now: {}", cmd.log);
        assert_eq!(u.active_magnitude("empower"), roundi(pct * 100.0));
        assert!((u.empower_factor() - (1.0 + pct)).abs() < 1e-9, "not clipped");
        assert_eq!((u.max_hp, u.attack_boost), (max_hp, boost), "no pool or Focus change");
        assert_eq!(command::item_grant(&e), ("empower", roundi(pct * 100.0)));
    }
}

/// IR12: a foe out of reach at impact: under `thrown_miss_spends` the throw misses and spends its copy (no `evaded`:
/// only a hidden foe evades); off (the default) keeps it ("not used"). A friend out of reach never spends.
#[test]
fn a_thrown_miss_spends_its_copy_under_the_rule() {
    for (rules, spends) in [(intent_only(), true), (classic(), false)] {
        let mut w = session(rules);
        let b = bomb("NONE");
        give(&mut w, HERO, &b, 2);
        w.grid.positions[FOE] = Some(V2::new(11, 4));
        let before = w.units[FOE].hp;
        let cmd = run_one(&mut w, HERO, use_on(HERO, FOE, &b));
        assert_eq!(w.units[FOE].hp, before, "no hit: {}", cmd.log);
        assert_eq!(used(&cmd), spends, "{}", cmd.log);
        assert_eq!(cmd.evaded, None, "out of reach is no evade");
        assert_eq!(command::pouch_count(&w, HERO, &b.id), if spends { 1 } else { 2 }, "{}", cmd.log);
    }
    let mut w = session(intent_only());
    let p = potion();
    give(&mut w, HERO, &p, 1);
    w.grid.positions[ALLY] = Some(V2::new(0, 8));
    let cmd = run_one(&mut w, HERO, use_on(HERO, ALLY, &p));
    assert!(!used(&cmd) && command::pouch_count(&w, HERO, &p.id) == 1, "a friend use keeps it: {}", cmd.log);
}

/// IR11, IR21 item mapping (FB-19): the derived rows under `field_intent`, the one legacy intent without it.
#[test]
fn items_derive_their_rows_under_field_intent() {
    let w = session(field_on());
    assert_eq!(round_intent::item_rows(&w, &potion()), ["support", "guard"]);
    assert_eq!(round_intent::item_rows(&w, &elixir(0.5)), ["guard"]);
    assert_eq!(round_intent::item_rows(&w, &bomb("PLUS")), ["field", "assault"]);
    assert_eq!(round_intent::item_rows(&w, &bomb("NONE")), ["assault"]);
    let short = item(json!({"damage": 30, "use_reach": 1, "area": "BURST"}));
    assert_eq!(round_intent::item_rows(&w, &short), ["field"], "an area within reach 1 is Field only");
    let legacy = session(intent_only());
    assert_eq!(round_intent::item_rows(&legacy, &bomb("PLUS")), ["maneuver"]);
    assert_eq!(round_intent::item_rows(&legacy, &potion()), ["support"]);
    assert!(round_intent::target_problem(&legacy, HERO, RoundIntent::Support, RoundTarget::Myself).is_empty(), "Support on yourself stays without the rule");
}

/// INTENT2 section 6, IR17 (Field row): an area item aimed at a tile lands its shape there at full power on everyone
/// but its user (the ally beside the foe included), reaching one tile further than the item (the `reach` bonus).
#[test]
fn an_area_item_field_row_hits_every_unit_on_its_tiles() {
    let mut w = session(field_on());
    let b = bomb("PLUS");
    give(&mut w, HERO, &b, 2);
    let cmd = planned(&mut w, HERO, RoundIntent::Field, RoundTarget::Cell(FOE_AT), PlanStep::use_item_at(&b.id, FOE_AT));
    assert!(matches!(cmd.kind, CmdKind::Item { cell: Some(c), .. } if c == FOE_AT));
    let hp: Vec<i32> = w.units.iter().map(|u| u.hp).collect();
    let cmd = run_one(&mut w, HERO, cmd);
    for t in [FOE, FOE_B, ALLY] {
        let want = damage::item_damage(b.damage, command::item_hit(&w, HERO, t, 1.0));
        assert_eq!(hp[t] - w.units[t].hp, want, "unit {t} at full power: {}", cmd.log);
    }
    assert_eq!(w.units[HERO].hp, hp[HERO], "never the user");
    assert_eq!(command::pouch_count(&w, HERO, &b.id), 1);
    // the reach bonus: the item's reach + 1 plans, + 2 does not
    assert_eq!(command::item_field_reach(&b), b.battle_reach() + 1);
    let mut w = session(field_on());
    give(&mut w, HERO, &b, 2);
    round_intent::set_intent(&mut w, HERO, RoundIntent::Field).unwrap();
    let far = HERO_AT + V2::new(b.battle_reach() + 2, 0);
    round_intent::set_target(&mut w, HERO, RoundTarget::Cell(far)).unwrap();
    let mut plan = Plan::new();
    assert!(!plan.add_step(&w, HERO, PlanStep::use_item_at(&b.id, far), true).is_empty(), "out of the Field reach");
}

/// IR17 (Assault row): an area item aimed at an enemy is target-locked: full damage on it, `splash_mult` on the other
/// foes in the shape, the ally beside it unhurt (foes only), one more wind-up block (`assault_shape_blocks`).
#[test]
fn an_area_item_assault_row_splashes_foes_only() {
    let mut w = session(field_on());
    let b = bomb("PLUS");
    give(&mut w, HERO, &b, 2);
    let cmd = planned(&mut w, HERO, RoundIntent::Assault, RoundTarget::Unit(FOE), PlanStep::use_item(&b.id, FOE));
    assert_eq!(cmd.block_cost, b.use_blocks + w.ruleset.numbers.assault_shape_blocks, "the longer wind-up");
    let hp: Vec<i32> = w.units.iter().map(|u| u.hp).collect();
    let cmd = run_one(&mut w, HERO, cmd);
    assert_eq!(hp[FOE] - w.units[FOE].hp, damage::item_damage(b.damage, command::item_hit(&w, HERO, FOE, 1.0)), "{}", cmd.log);
    let splash = w.ruleset.numbers.splash_mult;
    assert_eq!(hp[FOE_B] - w.units[FOE_B].hp, damage::item_damage(b.damage, command::item_hit(&w, HERO, FOE_B, splash)), "{}", cmd.log);
    assert_eq!(w.units[ALLY].hp, hp[ALLY], "foes only");
}

/// INTENT2 section 6: without `field_intent` (a `round_intent`-only row, classic) an area item hits its one target.
#[test]
fn without_field_intent_an_area_item_hits_one_target() {
    let mut w = session(intent_only());
    let b = bomb("PLUS");
    give(&mut w, HERO, &b, 1);
    assert!(!command::item_area_on(&w, &b));
    let hp_b = w.units[FOE_B].hp;
    let cmd = run_one(&mut w, HERO, use_on(HERO, FOE, &b));
    assert!(used(&cmd), "{}", cmd.log);
    assert_eq!(w.units[FOE_B].hp, hp_b, "no splash: {}", cmd.log);
}

/// The terrain index of `rubble` in `grid::TERRAIN_IDS`.
/// The terrain index of `rubble` (cover), looked up rather than hard-coded.
fn rubble() -> usize {
    rpgkit_battle::grid::TERRAIN_IDS.iter().position(|t| *t == "rubble").expect("rubble terrain")
}

/// A1 section 1 (Situational Guard / Cover): with `item_mitigation` on, the cover on the target's tile joins its guard
/// in the mitigation stage (a throw counts as ranged).
#[test]
fn item_mitigation_counts_cover() {
    let mut w = session(intent_only());
    let b = bomb("NONE");
    give(&mut w, HERO, &b, 1);
    let width = rpgkit_battle::grid::GRID_W;
    w.grid.terrain[(FOE_AT.y * width + FOE_AT.x) as usize] = rubble();
    let cover = command::cover_between(&w, HERO, FOE, true);
    assert!(cover > 0.0, "the foe stands in rubble");
    let rating = damage::armor_rating(&w.units[FOE]);
    let want = damage::item_damage(b.damage, ItemHit { splash: 1.0, mitigation: Some((cover, rating)), armor_piercing: false });
    assert_eq!(command::item_hit(&w, HERO, FOE, 1.0).mitigation, Some((cover, rating)));
    let before = w.units[FOE].hp;
    let cmd = run_one(&mut w, HERO, use_on(HERO, FOE, &b));
    assert_eq!(before - w.units[FOE].hp, want, "{}", cmd.log);
    assert!(want < damage::item_damage(b.damage, ItemHit { splash: 1.0, mitigation: Some((0.0, rating)), armor_piercing: false }), "cover lowers it");
}

/// IR12 with GD5: a hidden foe (stealth, moved this round) is not found: `evaded`, no damage; the copy is spent under
/// `thrown_miss_spends` and kept without it (the hidden check holds whatever the spend switch).
#[test]
fn a_hidden_foe_is_not_found() {
    for (rules, spends) in [(intent_only(), true), (ruleset(&[ROUND_INTENT]), false)] {
        let mut w = session(rules);
        let b = bomb("NONE");
        give(&mut w, HERO, &b, 2);
        rpgkit_rules::status::grant_now(&mut w.units[FOE], "stealth", 2, 0, "test");
        round_intent::note_moved(&mut w, FOE);
        assert!(round_intent::hidden(&w, FOE));
        let before = w.units[FOE].hp;
        let cmd = run_one(&mut w, HERO, use_on(HERO, FOE, &b));
        assert_eq!(w.units[FOE].hp, before, "{}", cmd.log);
        assert_eq!(cmd.evaded, Some(FOE), "{}", cmd.log);
        assert_eq!(used(&cmd), spends, "{}", cmd.log);
        assert_eq!(command::pouch_count(&w, HERO, &b.id), if spends { 1 } else { 2 });
    }
}

/// IR12, IR17: an area item's Assault row whose target left reach before impact finds no tiles, hits no one and spends
/// its copy under `thrown_miss_spends`.
#[test]
fn an_area_assault_miss_spends() {
    let mut w = session(field_on());
    let b = bomb("PLUS");
    give(&mut w, HERO, &b, 2);
    let cmd = planned(&mut w, HERO, RoundIntent::Assault, RoundTarget::Unit(FOE), PlanStep::use_item(&b.id, FOE));
    w.grid.positions[FOE] = Some(V2::new(11, 8));
    let hp: Vec<i32> = w.units.iter().map(|u| u.hp).collect();
    let cmd = run_one(&mut w, HERO, cmd);
    assert_eq!(w.units.iter().map(|u| u.hp).collect::<Vec<_>>(), hp, "no one is hit: {}", cmd.log);
    assert!(used(&cmd), "{}", cmd.log);
    assert_eq!(command::pouch_count(&w, HERO, &b.id), 1);
    assert_eq!(cmd.evaded, None, "out of reach is no evade");
}

/// IR15, FB-121 (review H2): under `field_intent` a hurt AI unit with a heal item and no heal skill drinks it in a Stand
/// round on itself (Support on yourself is refused).
#[test]
fn the_ai_drinks_its_own_heal_in_a_stand_round() {
    let mut w = session(field_on());
    let p = potion();
    give(&mut w, FOE_B, &p, 1);
    let mut profile = w.profile_of(FOE_B);
    profile.use_items = true;
    profile.flee_ratio = 0.0;
    w.profiles.insert(FOE_B, profile);
    w.units[FOE_B].hp = 1;
    let mut plan = Plan::new();
    rpgkit_battle::ai::plan_enemy_intents(&mut w, &mut plan);
    let intent = round_intent::intent_of(&w, FOE_B);
    let drinks = plan.entries_for(&w, FOE_B).iter().any(|e| e.item.as_ref().is_some_and(|i| i.id == p.id));
    assert!(drinks, "the potion is planned (intent {intent:?})");
    assert_eq!(intent, Some(RoundIntent::Guard), "a Stand round on itself");
}

/// INTENT2-U (section 6 "Menu"): under `field_intent` the menu has a cell for an item in each listed row it allows (an
/// area bomb in Field and Assault, a heal in Support and Stand), each live only on the target its row takes; without the
/// rule one cell, in the row its target gives.
#[test]
fn the_menu_lists_an_item_in_each_row_it_allows() {
    use rpgkit_battle::round_intent_plan::{menu_options, Pick, PickOption};
    let cells = |w: &World, at: RoundTarget, id: &str| -> Vec<PickOption> {
        menu_options(w, &Plan::new(), HERO, at).into_iter().filter(|o| matches!(&o.picks[..], [Pick::Item(x)] if x == id)).collect()
    };
    let rows = |c: &[PickOption]| c.iter().map(|o| (o.intent, o.problem.is_none())).collect::<Vec<_>>();
    let mut w = session(field_on());
    let (b, p) = (bomb("BURST"), potion());
    give(&mut w, HERO, &b, 1);
    give(&mut w, HERO, &p, 1);
    assert_eq!(rows(&cells(&w, RoundTarget::Unit(FOE), &b.id)), [(Some(RoundIntent::Field), true), (Some(RoundIntent::Assault), true)], "a bomb on an enemy: both rows live");
    assert_eq!(rows(&cells(&w, RoundTarget::Cell(V2::new(5, 6)), &b.id)), [(Some(RoundIntent::Field), true), (Some(RoundIntent::Assault), false)], "on a tile: Field only");
    let on_self = cells(&w, RoundTarget::Myself, &p.id);
    assert_eq!(rows(&on_self), [(Some(RoundIntent::Support), false), (Some(RoundIntent::Guard), true)], "a heal on yourself: Stand");
    assert_eq!(on_self[0].problem.as_deref(), Some("Target an ally to use this"));
    assert_eq!(rows(&cells(&w, RoundTarget::Unit(ALLY), &p.id)), [(Some(RoundIntent::Support), true), (Some(RoundIntent::Guard), false)], "on an ally: Support");
    let mut w = session(intent_only());
    give(&mut w, HERO, &b, 1);
    assert_eq!(cells(&w, RoundTarget::Unit(FOE), &b.id).len(), 1, "without field_intent: one cell");
}
