//! PR5-45 on a scratch world built here (no shipped data): the `stash` recipe target and its `craft` effect. A stash
//! recipe needs no pick; running it pays its cost as any recipe does and adds `count` of its item to the party stash
//! (stacking with what is there). Before the act weapon instances start, a crafted weapon is a plain stash id; from it
//! (under a ruleset with `durability`) each crafted copy gets a fresh instance at full durability, the instances held
//! before kept as they were. A refusal (an unpaid cost, a failing `requires`) changes nothing. Expectations derive from
//! the rows below and the kit's rules, not from printed numbers.

use aegis_core::json;
use rpgkit_world::recipe::{self, Pick, NOT_NOW_TEXT};
use rpgkit_world::state::PartyState;
use rpgkit_world::{new_game, weapons, Data, GameState};
use serde_json::{json, Value};

const SMITH: &str = "w:smith";
/// The act instances start in, the blade's full durability, the party's purse.
const FROM_ACT: i64 = 2;
const MAX: i64 = 30;
const GOLD: i64 = 100;
/// Each recipe's purse price, the ore it takes, the darts one run makes and the blades the double run makes.
const PRICE: i64 = 15;
const ORE: i64 = 1;
const DARTS: i64 = 3;
const PAIR: i64 = 2;
/// The flag the gated recipe waits on.
const GATE: &str = "craft_open";

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

/// A stash recipe of the smith crafting `count` of `item` for `PRICE` gold and `ORE` ore.
fn craft(id: &str, item: &str, count: Option<i64>) -> Value {
    let mut fx = json!({"effect": "craft", "item": item});
    if let Some(n) = count {
        fx["count"] = json!(n);
    }
    json!({"id": id, "name": id, "npc": "smith", "requires": [], "cost": {"gold": PRICE, "items": {"ore": ORE}}, "target": "stash", "effects": [fx]})
}

/// A sword class; a wearing blade, a stackable dart and ore; the smith's blade (count omitted: one), darts, a pair of
/// blades and a gated blade.
fn data() -> Data {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10});
    let mut gated = craft("make_gated", "blade", None);
    gated["requires"] = json!([{"kind": "flag", "key": GATE, "value": true}]);
    let catalog = json!({
        "classes": [{"id": "fencer", "name": "Fencer", "skill_path": {}, "base_stats": stats, "proficiency": {"sword": {"level": 3, "role": "main"}}}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fencer", "level": 1}],
        "items": [
            {"id": "blade", "name": "Blade", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10, "value": 10, "durability": MAX},
            {"id": "dart", "name": "Dart", "item_type": "CONSUMABLE", "value": 1},
            {"id": "ore", "name": "Ore", "item_type": "MATERIAL", "value": 0}
        ],
        "rulesets": [{"id": "steel", "name": "Steel", "rules": ["durability"]}],
        "recipes": [craft("make_blade", "blade", None), craft("make_darts", "dart", Some(DARTS)), craft("make_pair", "blade", Some(PAIR)), gated]
    });
    let families = json!([{"id": "sword", "item_type": "WEAPON", "hands": ["main_1h"]}]);
    let worlds = json!({"w": {"manifest": {"id": "w", "ruleset": "steel", "campaign": {"start_act": 1}}, "own": {
        "characters": with_ids("w", json!([{"id": "named", "name": "Named", "role": "party", "base_unit": "hero"}, {"id": "smith", "name": "Smith"}]))
    }}});
    let mut d = Data::from_values(&catalog, &worlds, &json!({"item_families": families}));
    d.from_act = FROM_ACT;
    d
}

/// A game in act `act` with the purse and `ores` ore in the stash.
fn game(d: &Data, act: i64, ores: i64) -> GameState {
    let m = new_game::hero_from_character(d, "w", "named").expect("a party character");
    let mut gs = GameState::create(d, "w", PartyState { active_ids: vec![m.id.clone()], members: vec![m], ..PartyState::default() });
    gs.campaign.as_mut().expect("a campaign world").act = act;
    gs.party.gold = GOLD;
    gs.party.add_to_stash("ore", ores);
    weapons::sync(d, &mut gs);
    gs
}

fn ok(d: &Data, gs: &mut GameState, id: &str) -> Vec<String> {
    let res = recipe::run(d, gs, SMITH, id, &Pick::default());
    assert!(json::b(&res, "ok", false), "{id}: {res:?}");
    json::sget(&res, "notes")
}

/// A refused run: the refusal, and the state is unchanged.
fn refused(d: &Data, gs: &mut GameState, id: &str) -> String {
    let before = gs.to_value();
    let res = recipe::run(d, gs, SMITH, id, &Pick::default());
    assert!(!json::b(&res, "ok", true), "{id} should be refused");
    assert_eq!(gs.to_value(), before, "{id}: a refusal changes nothing");
    json::sget(&res, "errors").join(" ")
}

fn stash(gs: &GameState, item: &str) -> i64 {
    gs.party.stash.get(item).copied().unwrap_or(0)
}

fn instances(gs: &GameState, item: &str) -> Vec<weapons::Weapon> {
    weapons::all(&gs.party).into_iter().filter(|w| w.item == item).cloned().collect()
}

#[test]
fn a_stash_recipe_needs_no_pick_and_its_craft_stacks() {
    let d = data();
    let mut gs = game(&d, 1, 2 * ORE);
    let rc = &d.cat.recipes["make_darts"];
    assert!(recipe::targets(&d, &gs, rc).is_empty() && recipe::families(&d, &gs, rc, &Pick::default()).is_empty(), "nothing to pick");
    let view = recipe::view(&d, &gs, SMITH, &Pick::default());
    let row = view.iter().map(json::dict).find(|r| json::s(r, "id") == "make_darts").expect("offered");
    assert!(json::b(row, "ready", false), "ready with no pick: {row:?}");
    let notes = ok(&d, &mut gs, "make_darts");
    assert_eq!(notes, [format!("Done: {}.", rc.name), format!("Dart x{DARTS} added to stash.")]);
    assert_eq!((stash(&gs, "dart"), stash(&gs, "ore"), gs.party.gold), (DARTS, ORE, GOLD - PRICE), "crafted and paid");
    ok(&d, &mut gs, "make_darts");
    assert_eq!((stash(&gs, "dart"), stash(&gs, "ore"), gs.party.gold), (2 * DARTS, 0, GOLD - 2 * PRICE), "the craft stacks");
}

#[test]
fn before_the_act_line_a_crafted_weapon_is_a_plain_stash_id() {
    let d = data();
    let mut gs = game(&d, FROM_ACT - 1, ORE);
    assert!(!weapons::active(&d, &gs));
    let notes = ok(&d, &mut gs, "make_blade");
    assert_eq!(notes.last().map(String::as_str), Some("Blade added to stash."), "count omitted: one");
    assert_eq!((stash(&gs, "blade"), gs.party.gold), (1, GOLD - PRICE));
    assert!(weapons::all(&gs.party).is_empty(), "no instances before the act line");
}

#[test]
fn from_the_act_line_each_crafted_weapon_is_a_fresh_full_instance() {
    let d = data();
    let mut gs = game(&d, FROM_ACT, 2 * ORE);
    assert!(weapons::active(&d, &gs));
    // a worn blade already in the stash keeps its instance and its wear
    gs.party.add_to_stash("blade", 1);
    weapons::sync(&d, &mut gs);
    let old = instances(&gs, "blade");
    assert_eq!(old.len(), 1);
    let worn = MAX / 3;
    weapons::find_mut(&mut gs.party, &old[0].uid).expect("the instance").durability = worn;
    ok(&d, &mut gs, "make_blade");
    let now = instances(&gs, "blade");
    assert_eq!((stash(&gs, "blade"), now.len()), (2, 2), "one instance per stashed copy");
    assert_eq!(weapons::find(&gs.party, &old[0].uid).map(|w| w.durability), Some(worn), "the old one keeps its wear");
    let fresh: Vec<&weapons::Weapon> = now.iter().filter(|w| w.uid != old[0].uid).collect();
    assert_eq!(fresh.iter().map(|w| (w.durability, w.masterwork)).collect::<Vec<_>>(), [(weapons::full(&d, "blade"), false)], "the crafted one at full");
    assert_eq!(weapons::full(&d, "blade"), MAX);
    let notes = ok(&d, &mut gs, "make_pair");
    assert_eq!(notes.last().cloned(), Some(format!("Blade x{PAIR} added to stash.")));
    let after = instances(&gs, "blade");
    assert_eq!((stash(&gs, "blade"), after.len()), (2 + PAIR, (2 + PAIR) as usize), "each crafted copy an instance");
    assert_eq!(after.iter().filter(|w| w.durability == MAX).count(), (1 + PAIR) as usize, "every crafted copy full, the worn one as it was");
    let mut uids: Vec<&str> = after.iter().map(|w| w.uid.as_str()).collect();
    uids.sort_unstable();
    uids.dedup();
    assert_eq!(uids.len(), after.len(), "distinct uids");
}

#[test]
fn an_unpaid_cost_or_a_failing_requires_refuses_with_nothing_paid() {
    let d = data();
    let mut gs = game(&d, FROM_ACT, 0);
    assert_eq!(refused(&d, &mut gs, "make_blade"), "Not enough Ore.");
    gs.party.add_to_stash("ore", ORE);
    gs.party.gold = PRICE - 1;
    assert_eq!(refused(&d, &mut gs, "make_blade"), "Not enough gold.");
    gs.party.gold = GOLD;
    assert_eq!(refused(&d, &mut gs, "make_gated"), NOT_NOW_TEXT, "the requires fail");
    assert!(recipe::offered(&d, &gs, SMITH).iter().all(|rc| rc.id != "make_gated"), "and it is not offered");
    gs.set_flag(GATE, Value::Bool(true));
    ok(&d, &mut gs, "make_gated");
    assert_eq!((stash(&gs, "blade"), instances(&gs, "blade").len(), gs.party.gold, stash(&gs, "ore")), (1, 1, GOLD - PRICE, 0), "open: crafted and paid");
}
