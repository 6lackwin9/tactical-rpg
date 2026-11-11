//! PK2-FOLLOWUP (PR5-36, `docs/sessions/PK2_GRAFTS.md` 5, 7; FB-211, FB-214 (d), FB-216, FB-218 (e)) on a scratch world
//! built here (no shipped data): the recipe effects on weapon instances and the forge rite. A `weapon` recipe runs on a
//! picked instance (by uid) while instances are active; `repair` brings it back to full durability for its share of the
//! effect's price (`repair_share`: the full-repair price times the missing durability's fraction, rounded up): purse
//! alone for a basic weapon, plus salvage of its tier (`tiers` by rarity) or, for a themed weapon, of its theme
//! (`themes`); `reforge` brings it back to full and, behind the recipe's `requires`, makes it masterwork (its own
//! technique opens in the build); `forge_rite` gives a member with mastery 3 its forge rite (the family keystone opens).
//! A refusal changes nothing. Expectations derive from the rows below and the kit's rules, not from printed numbers.

use aegis_core::json;
use rpgkit_data::recipe::repair_share;
use rpgkit_world::recipe::{self, Pick};
use rpgkit_world::state::PartyState;
use rpgkit_world::{mastery, new_game, progression, weapons, Data, GameState};
use serde_json::{json, Value};

const SMITH: &str = "w:smith";
/// The act instances start in, every weapon's full durability, the party's purse.
const FROM_ACT: i64 = 2;
const MAX: i64 = 40;
const GOLD: i64 = 100;
/// The repair's full-repair price: purse and salvage count.
const REPAIR_GOLD: i64 = 20;
const REPAIR_COUNT: i64 = 2;
const OWN_TECHNIQUE: &str = "t_own";

fn with_ids(w: &str, rows: Value) -> Value {
    let rows = rows
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let mut r = r.as_object().unwrap().clone();
            let id = r["id"].as_str().unwrap().to_string();
            r.insert("_world".into(), w.into());
            r.insert("_qid".into(), format!("{w}:{id}").into());
            Value::Object(r)
        })
        .collect();
    Value::Array(rows)
}

fn recipe(id: &str, target: &str, effects: Value) -> Value {
    json!({"id": id, "name": id, "npc": "smith", "requires": [], "cost": {}, "target": target, "effects": effects})
}

/// A sword class (proficiency 3); a common blade (with its own technique), a rare blade and a rare themed blade, each
/// wearing `MAX`; salvage items; the smith's repair, plain reforge, masterwork reforge (behind a flag) and forge rite.
fn data() -> Data {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10});
    let sword = |id: &str, extra: Value| {
        let mut row = json!({"id": id, "name": id, "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10, "value": 10, "durability": MAX});
        row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        row
    };
    let mut masterwork = recipe("masterwork", "weapon", json!([{"effect": "reforge", "masterwork": true}]));
    masterwork["requires"] = json!([{"kind": "flag", "key": "forge_open"}]);
    masterwork["cost"] = json!({"items": {"ingot": 1}});
    let catalog = json!({
        "classes": [{"id": "blade", "name": "Blade", "skill_path": {}, "base_stats": stats, "proficiency": {"sword": {"level": 3, "role": "main"}}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
            {"id": OWN_TECHNIQUE, "name": "Own", "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY"}
        ],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "blade", "level": 1}],
        "items": [
            sword("plain", json!({"rarity": "COMMON", "technique": {"skill": OWN_TECHNIQUE}})),
            sword("fine_blade", json!({"rarity": "RARE"})),
            sword("order_blade", json!({"rarity": "RARE", "theme": "order"})),
            {"id": "fine", "name": "Fine Salvage", "item_type": "MATERIAL", "value": 0},
            {"id": "order_steel", "name": "Order Steel", "item_type": "MATERIAL", "value": 0},
            {"id": "ingot", "name": "Ingot", "item_type": "MATERIAL", "value": 0}
        ],
        "rulesets": [{"id": "steel", "name": "Steel", "rules": ["durability"]}],
        "recipes": [
            recipe("repair", "weapon", json!([{"effect": "repair", "gold": REPAIR_GOLD, "count": REPAIR_COUNT, "tiers": {"RARE": "fine"}, "themes": {"order": "order_steel"}}])),
            recipe("reforge", "weapon", json!([{"effect": "reforge"}])),
            masterwork,
            recipe("rite", "member", json!([{"effect": "forge_rite"}]))
        ]
    });
    let families = json!([{"id": "sword", "item_type": "WEAPON", "hands": ["main_1h"]}]);
    let worlds = json!({"w": {"manifest": {"id": "w", "ruleset": "steel", "campaign": {"start_act": 1}}, "own": {
        "characters": with_ids("w", json!([{"id": "named", "name": "Named", "role": "party", "base_unit": "hero"}, {"id": "smith", "name": "Smith"}]))
    }}});
    let mut d = Data::from_values(&catalog, &worlds, &json!({"item_families": families}));
    d.from_act = FROM_ACT;
    d
}

/// A game in act `act`: the hero wielding a plain blade; the stash a rare and a themed blade, salvage and an ingot.
fn game(d: &Data, act: i64) -> GameState {
    let mut m = new_game::hero_from_character(d, "w", "named").expect("a party character");
    m.equipment.insert("main_hand".into(), "plain".into());
    let mut gs = GameState::create(d, "w", PartyState { active_ids: vec![m.id.clone()], members: vec![m], ..PartyState::default() });
    gs.campaign.as_mut().expect("a campaign world").act = act;
    for (item, n) in [("fine_blade", 1), ("order_blade", 1), ("fine", 3), ("order_steel", 3), ("ingot", 1)] {
        gs.party.add_to_stash(item, n);
    }
    gs.party.gold = GOLD;
    weapons::sync(d, &mut gs);
    gs
}

/// The uid of the (first) instance of `item`.
fn uid(gs: &GameState, item: &str) -> String {
    weapons::all(&gs.party).into_iter().find(|w| w.item == item).map(|w| w.uid.clone()).expect("an instance")
}

fn durability(gs: &GameState, uid: &str) -> i64 {
    weapons::find(&gs.party, uid).expect("the instance").durability
}

/// Wears instance `uid` down to `left`.
fn wear_to(gs: &mut GameState, uid: &str, left: i64) {
    weapons::find_mut(&mut gs.party, uid).expect("the instance").durability = left;
}

fn on(uid: &str) -> Pick {
    Pick { weapon: uid.into(), ..Pick::default() }
}

fn ok(d: &Data, gs: &mut GameState, id: &str, pick: &Pick) -> Vec<String> {
    let res = recipe::run(d, gs, SMITH, id, pick);
    assert!(json::b(&res, "ok", false), "{id}: {res:?}");
    json::sget(&res, "notes")
}

/// A refused run: the refusal, and the state is unchanged.
fn refused(d: &Data, gs: &mut GameState, id: &str, pick: &Pick) -> String {
    let before = gs.to_value();
    let res = recipe::run(d, gs, SMITH, id, pick);
    assert!(!json::b(&res, "ok", true), "{id} should be refused");
    assert_eq!(gs.to_value(), before, "{id}: a refusal changes nothing");
    json::sget(&res, "errors").join(" ")
}

fn stash(gs: &GameState, item: &str) -> i64 {
    gs.party.stash.get(item).copied().unwrap_or(0)
}

#[test]
fn the_share_is_the_missing_fraction_rounded_up() {
    assert_eq!(repair_share(REPAIR_GOLD, MAX, MAX), REPAIR_GOLD, "a full repair pays the full price");
    assert_eq!(repair_share(REPAIR_GOLD, MAX / 2, MAX), REPAIR_GOLD / 2);
    assert_eq!(repair_share(REPAIR_GOLD, 1, MAX), 1, "any repair of a priced part costs at least 1");
    assert_eq!((repair_share(REPAIR_GOLD, 0, MAX), repair_share(0, MAX, MAX), repair_share(REPAIR_GOLD, MAX, 0)), (0, 0, 0));
    assert_eq!(repair_share(REPAIR_COUNT, MAX * 2, MAX), REPAIR_COUNT, "never past the full price");
}

#[test]
fn a_basic_weapon_repairs_for_purse_alone() {
    let d = data();
    let mut gs = game(&d, FROM_ACT);
    let blade = uid(&gs, "plain");
    let rc = &d.cat.recipes["repair"];
    assert_eq!(recipe::targets(&d, &gs, rc), weapons::all(&gs.party).iter().map(|w| w.uid.clone()).collect::<Vec<_>>(), "every instance, members first");
    assert_eq!(refused(&d, &mut gs, "repair", &on(&blade)), "plain needs no repair.", "a whole weapon");
    let missing = MAX / 4;
    wear_to(&mut gs, &blade, MAX - missing);
    let cost = repair_share(REPAIR_GOLD, missing, MAX);
    let view = recipe::view(&d, &gs, SMITH, &on(&blade));
    let row = view.iter().map(json::dict).find(|r| json::s(r, "id") == "repair").unwrap();
    assert_eq!(json::sget(row, "cost"), [format!("{cost} gold")], "the pane shows the scaled share, no salvage for a common blade");
    ok(&d, &mut gs, "repair", &on(&blade));
    assert_eq!((durability(&gs, &blade), gs.party.gold, stash(&gs, "fine")), (MAX, GOLD - cost, 3), "back to full; purse alone");
    // the cost lines read the instances as the run sees them (synced): a worn weapon sold since the last sync shows no
    // share, as the run refuses it
    let sold = uid(&gs, "fine_blade");
    wear_to(&mut gs, &sold, 0);
    assert!(!recipe::pick_cost_lines(&d, &gs, rc, &on(&sold)).is_empty(), "a worn instance shows its share");
    gs.party.remove_from_stash("fine_blade", 1);
    assert!(weapons::find(&gs.party, &sold).is_some(), "not synced yet");
    assert_eq!(recipe::pick_cost_lines(&d, &gs, rc, &on(&sold)), Vec::<String>::new(), "gone after a sync: no share");
    assert_eq!(refused(&d, &mut gs, "repair", &on(&sold)), "Pick a weapon.");
    wear_to(&mut gs, &blade, 0);
    gs.party.gold = REPAIR_GOLD - 1;
    assert_eq!(refused(&d, &mut gs, "repair", &on(&blade)), "Not enough gold.");
    assert_eq!(refused(&d, &mut gs, "repair", &on("w999")), "Pick a weapon.");
}

#[test]
fn a_higher_tier_weapon_needs_its_tier_salvage_and_a_themed_one_its_theme() {
    let d = data();
    let mut gs = game(&d, FROM_ACT);
    let (fine, order) = (uid(&gs, "fine_blade"), uid(&gs, "order_blade"));
    let missing = MAX / 2;
    wear_to(&mut gs, &fine, MAX - missing);
    wear_to(&mut gs, &order, MAX - missing);
    let (cost, count) = (repair_share(REPAIR_GOLD, missing, MAX), repair_share(REPAIR_COUNT, missing, MAX));
    ok(&d, &mut gs, "repair", &on(&fine));
    assert_eq!((durability(&gs, &fine), gs.party.gold, stash(&gs, "fine"), stash(&gs, "order_steel")), (MAX, GOLD - cost, 3 - count, 3), "rarity RARE: fine salvage");
    ok(&d, &mut gs, "repair", &on(&order));
    assert_eq!((durability(&gs, &order), gs.party.gold, stash(&gs, "fine"), stash(&gs, "order_steel")), (MAX, GOLD - 2 * cost, 3 - count, 3 - count), "theme order: its own salvage, not the tier's");
    wear_to(&mut gs, &order, 0);
    gs.party.remove_from_stash("order_steel", stash(&gs, "order_steel"));
    assert_eq!(refused(&d, &mut gs, "repair", &on(&order)), "Not enough Order Steel.");
}

#[test]
fn a_weapon_recipe_waits_for_the_act_line() {
    let d = data();
    let mut gs = game(&d, FROM_ACT - 1);
    assert!(weapons::all(&gs.party).is_empty() && recipe::targets(&d, &gs, &d.cat.recipes["repair"]).is_empty());
    assert_eq!(refused(&d, &mut gs, "repair", &on("w1")), recipe::NO_WEAPONS_TEXT);
}

#[test]
fn reforge_restores_and_masterwork_waits_on_its_requires() {
    let d = data();
    let mut gs = game(&d, FROM_ACT);
    let blade = uid(&gs, "plain");
    assert_eq!(refused(&d, &mut gs, "reforge", &on(&blade)), "plain needs no reforging.");
    wear_to(&mut gs, &blade, 1);
    ok(&d, &mut gs, "reforge", &on(&blade));
    assert_eq!((durability(&gs, &blade), gs.party.gold), (MAX, GOLD), "a free reforge back to full");
    assert_eq!(refused(&d, &mut gs, "masterwork", &on(&blade)), recipe::NOT_NOW_TEXT, "masterwork forging is not open");
    assert!(progression::build(&d, &gs.party.members[0]).expect("a build").unit.techniques.is_empty(), "no technique before");
    gs.set_flag("forge_open", Value::Bool(true));
    ok(&d, &mut gs, "masterwork", &on(&blade));
    let w = weapons::find(&gs.party, &blade).unwrap();
    assert!(w.masterwork && stash(&gs, "ingot") == 0, "masterwork, the ingot paid");
    let built = progression::build(&d, &gs.party.members[0]).expect("a build");
    assert!(built.unit.techniques.iter().any(|t| t.skill == OWN_TECHNIQUE), "the masterwork main hand opens its own technique");
    gs.party.add_to_stash("ingot", 1);
    assert_eq!(refused(&d, &mut gs, "masterwork", &on(&blade)), "plain needs no reforging.", "masterwork and whole already");
}

#[test]
fn the_forge_rite_needs_mastery_3_and_is_taken_once() {
    let d = data();
    let mut gs = game(&d, FROM_ACT);
    let hero = Pick { member: gs.party.members[0].id.clone(), ..Pick::default() };
    assert!(refused(&d, &mut gs, "rite", &hero).contains("needs mastery 3"));
    gs.party.members[0].mastery_xp.insert("sword".into(), mastery::xp_cap());
    ok(&d, &mut gs, "rite", &hero);
    assert!(gs.party.members[0].forge_rite);
    assert!(gs.party.members[0].to_value().get(rpgkit_world::state::FORGE_RITE_KEY).is_some(), "saved");
    assert!(refused(&d, &mut gs, "rite", &hero).contains("already"));
}
