//! PK2-S5a (`docs/sessions/PK2_GRAFTS.md` 6, FB-167 Q7 (a)) on a scratch world built here (no shipped data): weapon
//! instances. They are active only from the manifest's `durability.from_act`, under a world ruleset with the rule
//! `durability`; before that (or without either) nothing changes and a save is the same as before (no instance keys).
//! At the act line every owned weapon (item type `WEAPON`: worn in a slot or in the stash; never a shield, armour, a
//! consumable or a class default) becomes an instance at its full durability (`ItemDef::max_durability`: the item's
//! own, else its family's); a shop buy and loot add fresh full instances; a sale takes the worst stashed instance;
//! gear moved between a slot and the stash keeps its instance; a departed member's instances go to the stash; the
//! stash and the slots refer to them by uid (`weapons::wield`); a masterwork main hand opens the weapon's own technique
//! in the build. Expectations derive from the rows below and the kit's rules, not from printed numbers.

use aegis_core::json;
use rpgkit_world::weapons::{self, Weapon, MEMBER_KEY, STASH_KEY, UID_PREFIX};
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{battle, campaign, new_game, progression, roster, save, shop, Data, GameState};
use serde_json::{json, Value};
use std::collections::HashSet;

/// The act the scratch manifest starts instances in.
const FROM_ACT: i64 = 2;
/// The sword family's full durability and the heirloom's own (over it).
const FAMILY_DURABILITY: i64 = 30;
const OWN_DURABILITY: i64 = 45;
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

/// A class training sword, axe, tome and shield; a blade (the sword family's durability, its own technique), an
/// heirloom sword (its own durability), an axe (no durability anywhere), a tome (off-hand focus, item type WEAPON), a
/// buckler (SHIELD), a cap and a potion; a ruleset row `steel` with the rule `durability` and one `plain` without; a
/// campaign world from act 1 with a shop selling blades and an encounter whose objective reward loots one.
fn data_with(ruleset: &str) -> Data {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10});
    let sword = |id: &str, extra: Value| {
        let mut row = json!({"id": id, "name": id, "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10, "value": 10,
            "family_durability": FAMILY_DURABILITY});
        row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        row
    };
    let catalog = json!({
        "classes": [{"id": "blade", "name": "Blade", "skill_path": {}, "base_stats": stats, "proficiency": {
            "sword": {"level": 2}, "axe": {"level": 2}, "tome": {"level": 2}, "shield": {"level": 2}, "cloth": {"level": 2}}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
            {"id": OWN_TECHNIQUE, "name": "Own", "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY"}
        ],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "blade", "level": 1}],
        "items": [
            sword("blade", json!({"technique": {"skill": OWN_TECHNIQUE}})),
            sword("heirloom", json!({"durability": OWN_DURABILITY})),
            {"id": "axe", "name": "Axe", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "axe", "base_power": 10, "value": 10},
            {"id": "tome", "name": "Tome", "item_type": "WEAPON", "slot": "OFF_HAND", "weapon_type": "tome", "off_hand_category": "focus", "value": 10,
                "family_durability": FAMILY_DURABILITY},
            {"id": "buckler", "name": "Buckler", "item_type": "SHIELD", "slot": "OFF_HAND", "weapon_type": "shield", "value": 10},
            {"id": "cap", "name": "Cap", "item_type": "ARMOR", "slot": "HEAD", "weapon_type": "cloth", "value": 10},
            {"id": "potion", "name": "Potion", "item_type": "CONSUMABLE", "value": 5, "max_stack": 9}
        ],
        "rulesets": [{"id": "steel", "name": "Steel", "rules": ["durability"]}, {"id": "plain", "name": "Plain", "rules": []}]
    });
    let families = json!([
        {"id": "sword", "item_type": "WEAPON", "hands": ["main_1h"]},
        {"id": "axe", "item_type": "WEAPON", "hands": ["main_1h"]},
        {"id": "tome", "item_type": "WEAPON", "hands": ["off"]},
        {"id": "shield", "item_type": "SHIELD", "hands": ["off"]}
    ]);
    let worlds = json!({"w": {"manifest": {"id": "w", "ruleset": ruleset, "campaign": {"start_act": 1}}, "own": {
        "characters": with_ids("w", json!([{"id": "named", "name": "Named", "role": "party", "base_unit": "hero"}, {"id": "other", "name": "Other", "role": "party", "base_unit": "hero"}])),
        "shops": with_ids("w", json!([{"id": "store", "name": "Store", "stock": [{"item": "blade"}], "buys": ["WEAPON"]}])),
        "loot": with_ids("w", json!([{"id": "l_blade", "ref": "blade", "count": 1}])),
        "encounters": with_ids("w", json!([{"id": "e_fight", "name": "Fight", "enemies": [], "objective_reward": {"loot": "l_blade"}}]))
    }}});
    let mut d = Data::from_values(&catalog, &worlds, &json!({"item_families": families}));
    d.from_act = FROM_ACT;
    d
}

fn data() -> Data {
    data_with("steel")
}

/// A game in act `act`: the hero wearing a blade, a tome and a cap; a second member with the class default only; the
/// stash with two blades, an heirloom, an axe, a buckler and potions; gold for a few blades.
fn game(d: &Data, act: i64) -> GameState {
    let mut m = new_game::hero_from_character(d, "w", "named").expect("a party character");
    for (slot, item) in [("main_hand", "blade"), ("off_hand", "tome"), ("head", "cap")] {
        m.equipment.insert(slot.into(), item.into());
    }
    let other = new_game::hero_from_character(d, "w", "other").expect("a second party character");
    let ids = vec![m.id.clone(), other.id.clone()];
    let mut gs = GameState::create(d, "w", PartyState { active_ids: ids, members: vec![m, other], ..PartyState::default() });
    gs.campaign.as_mut().expect("a campaign world").act = act;
    for (item, n) in [("blade", 2), ("heirloom", 1), ("axe", 1), ("buckler", 1), ("potion", 3)] {
        gs.party.add_to_stash(item, n);
    }
    gs.party.gold = 100;
    gs
}

fn hero(gs: &GameState) -> &UnitState {
    &gs.party.members[0]
}

/// The stashed instances' items, in order.
fn stashed_items(gs: &GameState) -> Vec<&str> {
    gs.party.stash_weapons.iter().map(|w| w.item.as_str()).collect()
}

fn count(gs: &GameState, item: &str) -> usize {
    gs.party.stash_weapons.iter().filter(|w| w.item == item).count()
}

fn set_act(d: &Data, gs: &mut GameState, act: i64) {
    let mut events = Vec::new();
    assert!(campaign::apply_effect(d, gs, json!({"effect": "set_act", "act": act}).as_object().unwrap(), &mut events));
}

/// The same instance set and fields, every uid unique and given.
fn uids_unique(gs: &GameState) {
    let all = weapons::all(&gs.party);
    let set: HashSet<&str> = all.iter().map(|w| w.uid.as_str()).collect();
    assert_eq!(set.len(), all.len(), "every uid once: {all:?}");
    assert!(all.iter().all(|w| w.uid.starts_with(UID_PREFIX)), "every instance named");
}

/// A save envelope of `state` as `save::decode` reads it.
fn envelope(state: &Value) -> String {
    json!({"format": save::FORMAT, "version": save::VERSION, "saved_at": 1, "game_state": state}).to_string()
}

#[test]
fn nothing_is_an_instance_before_the_act_without_the_manifest_key_or_without_the_rule() {
    let d = data();
    let mut early = game(&d, FROM_ACT - 1);
    let before = early.clone();
    assert!(!weapons::active(&d, &early) && !weapons::sync(&d, &mut early), "act {} is before the line", FROM_ACT - 1);
    assert_eq!(early, before, "nothing changes");
    let mut none = data();
    none.from_act = 0;
    let mut gs = game(&none, FROM_ACT + 1);
    assert!(!weapons::active(&none, &gs) && !weapons::sync(&none, &mut gs), "no durability.from_act: never");
    let plain = data_with("plain");
    let mut gs = game(&plain, FROM_ACT + 1);
    assert!(!weapons::active(&plain, &gs) && !weapons::sync(&plain, &mut gs), "a ruleset without the rule: never");
    let v = gs.to_value();
    assert!(json::dget(json::dict(&v), "party").get(STASH_KEY).is_none(), "no stash instance key");
    assert!(v["party"]["members"].as_array().unwrap().iter().all(|m| m.get(MEMBER_KEY).is_none()), "no member instance key");
}

#[test]
fn an_act_1_save_loads_unchanged() {
    let d = data();
    let gs = game(&d, FROM_ACT - 1);
    let saved = gs.to_value();
    let loaded = save::decode(&d, &envelope(&saved)).expect("loads");
    assert_eq!(loaded.state.to_value(), saved, "the same state, no instance keys");
    assert!(weapons::all(&loaded.state.party).is_empty());
    // and a game without the manifest key loads unchanged in any act
    let mut none = data();
    none.from_act = 0;
    let late = game(&none, FROM_ACT + 1).to_value();
    assert_eq!(save::decode(&none, &envelope(&late)).expect("loads").state.to_value(), late);
}

#[test]
fn the_act_line_makes_every_owned_weapon_an_instance_at_full_durability() {
    let d = data();
    let mut gs = game(&d, FROM_ACT - 1);
    set_act(&d, &mut gs, FROM_ACT);
    let h = hero(&gs);
    let worn: Vec<(&str, &str)> = h.weapons.iter().map(|(slot, w)| (slot.as_str(), w.item.as_str())).collect();
    assert_eq!(worn, [("main_hand", "blade"), ("off_hand", "tome")], "the worn weapons (the off-hand focus too), not the cap");
    assert!(gs.party.members[1].weapons.is_empty(), "a class default is no owned weapon");
    // one instance per stashed weapon copy, stash order; the buckler (a shield) and the potions stay plain ids
    let mut want = Vec::new();
    for (item, n) in &gs.party.stash {
        if d.cat.items[item].item_type == "WEAPON" {
            want.extend(std::iter::repeat_n(item.as_str(), *n as usize));
        }
    }
    assert_eq!(stashed_items(&gs), want);
    assert_eq!(want, ["blade", "blade", "heirloom", "axe"]);
    for w in weapons::all(&gs.party) {
        assert_eq!(w.durability, i64::from(d.cat.items[&w.item].max_durability()), "{} starts full", w.item);
        assert!(!w.masterwork);
    }
    let full = |item: &str| gs.party.stash_weapons.iter().find(|w| w.item == item).unwrap().durability;
    assert_eq!((full("blade"), full("heirloom"), full("axe")), (FAMILY_DURABILITY, OWN_DURABILITY, 0), "the family's, the item's own over it, none");
    uids_unique(&gs);
    let first: Vec<String> = weapons::all(&gs.party).iter().map(|w| w.uid.clone()).collect();
    assert_eq!(first[0], format!("{UID_PREFIX}1"), "numbered from 1, the members' first");
    // the line is crossed once: a later sync changes nothing, a later act keeps the same instances
    assert!(!weapons::sync(&d, &mut gs), "a second sync changes nothing");
    set_act(&d, &mut gs, FROM_ACT + 1);
    assert_eq!(weapons::all(&gs.party).iter().map(|w| w.uid.clone()).collect::<Vec<_>>(), first);
}

#[test]
fn the_instances_save_and_load_by_uid() {
    let d = data();
    let mut gs = game(&d, FROM_ACT);
    weapons::sync(&d, &mut gs);
    gs.party.stash_weapons[0].durability -= 1;
    gs.party.members[0].weapons.get_mut("main_hand").unwrap().masterwork = true;
    let v = gs.to_value();
    let member = &v["party"]["members"][0][MEMBER_KEY];
    assert_eq!(member["main_hand"].as_object().unwrap().keys().collect::<Vec<_>>(), weapons::KEYS.iter().collect::<Vec<_>>(), "{{uid, item, durability, masterwork}}");
    assert_eq!(v["party"][STASH_KEY].as_array().unwrap().len(), gs.party.stash_weapons.len());
    let back = save::decode(&d, &envelope(&v)).expect("loads").state;
    assert_eq!(back.to_value(), v, "the same instances back");
    // a damaged instance is refused, a uid twice too
    let mut bad = v.clone();
    bad["party"][STASH_KEY][0]["durability"] = json!(-1);
    assert!(save::decode(&d, &envelope(&bad)).is_err(), "negative durability");
    let mut twice = v.clone();
    twice["party"][STASH_KEY][1]["uid"] = twice["party"][STASH_KEY][0]["uid"].clone();
    let renamed = save::decode(&d, &envelope(&twice)).expect("a duplicate uid is renamed while active").state;
    uids_unique(&renamed);
    let mut off = data();
    off.from_act = 0;
    let errors = save::decode(&off, &envelope(&twice)).err().expect("refused while inactive (nothing renames it)");
    assert!(errors.iter().any(|e| e.contains("used twice")), "{errors:?}");
}

#[test]
fn shops_and_loot_add_fresh_instances_and_a_sale_takes_the_worst() {
    let d = data();
    let mut gs = game(&d, FROM_ACT);
    weapons::sync(&d, &mut gs);
    let blades = count(&gs, "blade");
    let r = shop::buy(&d, &mut gs, "store", "item", "blade", 1);
    assert!(json::b(&r, "ok", false), "{r:?}");
    assert_eq!(count(&gs, "blade"), blades + 1, "a bought blade is an instance");
    let newest = gs.party.stash_weapons.iter().filter(|w| w.item == "blade").last().unwrap().clone();
    assert_eq!((newest.durability, newest.masterwork), (FAMILY_DURABILITY, false), "at full durability");
    uids_unique(&gs);
    // wear two blades down differently, make one masterwork: a sale keeps the masterwork and the most durable
    let at: Vec<usize> = gs.party.stash_weapons.iter().enumerate().filter(|(_, w)| w.item == "blade").map(|(i, _)| i).collect();
    gs.party.stash_weapons[at[0]].masterwork = true;
    gs.party.stash_weapons[at[0]].durability = 0;
    gs.party.stash_weapons[at[1]].durability = 1;
    let worst = gs.party.stash_weapons[at[1]].uid.clone();
    let r = shop::sell(&d, &mut gs, "store", "blade", 1);
    assert!(json::b(&r, "ok", false), "{r:?}");
    assert_eq!(count(&gs, "blade"), blades, "one blade instance fewer");
    assert!(weapons::find(&gs.party, &worst).is_none(), "the worst (not masterwork, least durability) went");
    assert!(gs.party.stash_weapons.iter().any(|w| w.masterwork), "the masterwork stays");
    // the trade path too
    let r = shop::trade(&d, &mut gs, "store", &[("item".into(), "blade".into(), 2)], &[]);
    assert!(json::b(&r, "ok", false), "{r:?}");
    assert_eq!(count(&gs, "blade"), blades + 2);
    // loot: the encounter's objective reward
    let before = count(&gs, "blade");
    battle::settle(&d, &mut gs, "w:e_fight", "won", None);
    assert_eq!(gs.party.stash.get("blade").copied().unwrap_or(0) as usize, before + 1, "the loot reached the stash");
    assert_eq!(count(&gs, "blade"), before + 1, "loot is a fresh instance");
    assert_eq!(gs.party.stash_weapons.iter().filter(|w| w.item == "blade").last().unwrap().durability, FAMILY_DURABILITY, "at full durability");
    uids_unique(&gs);
}

#[test]
fn gear_moves_keep_their_instance_and_the_stash_is_addressed_by_uid() {
    let d = data();
    let mut gs = game(&d, FROM_ACT);
    weapons::sync(&d, &mut gs);
    let id = hero(&gs).id.clone();
    let wielded = hero(&gs).weapons["main_hand"].clone();
    // equipment by uid: swap in a stashed blade of the same item
    let spare = gs.party.stash_weapons.iter().find(|w| w.item == "blade").unwrap().uid.clone();
    let stash_before = gs.party.stash.clone();
    weapons::wield(&d, &mut gs, &id, "main_hand", &spare).expect("same item");
    assert_eq!(hero(&gs).weapons["main_hand"].uid, spare);
    assert!(gs.party.stash_weapons.iter().any(|w| w.uid == wielded.uid), "the old one is stashed");
    assert_eq!(gs.party.stash, stash_before, "the counts do not change");
    let axe = gs.party.stash_weapons.iter().find(|w| w.item == "axe").unwrap().uid.clone();
    assert!(weapons::wield(&d, &mut gs, &id, "main_hand", &axe).is_err(), "another item is refused");
    assert!(weapons::wield(&d, &mut gs, &id, "main_hand", "nope").is_err());
    // unequip to the stash (a GUI move of ids), then wear it again: the same instance comes back
    gs.party.members[0].weapons.get_mut("main_hand").unwrap().masterwork = true;
    let kept = hero(&gs).weapons["main_hand"].clone();
    let item = gs.party.members[0].equipment.shift_remove("main_hand").unwrap();
    gs.party.add_to_stash(&item, 1);
    weapons::sync(&d, &mut gs);
    assert!(gs.party.stash_weapons.contains(&kept), "unequipped: the instance is stashed");
    gs.party.remove_from_stash(&item, 1);
    gs.party.members[0].equipment.insert("main_hand".into(), item);
    weapons::sync(&d, &mut gs);
    assert_eq!(hero(&gs).weapons["main_hand"], kept, "worn again: the best of the item (the masterwork) comes back");
    uids_unique(&gs);
    // a departed member's instances go to the stash with its gear
    let held: Vec<Weapon> = hero(&gs).weapons.values().cloned().collect();
    roster::depart(&d, &mut gs, &id, "retired", "story", "").expect("departs");
    assert!(held.iter().all(|w| gs.party.stash_weapons.contains(w)), "stashed");
    assert!(gs.party.departed[0].member.weapons.is_empty(), "the record keeps none");
    assert!(!weapons::sync(&d, &mut gs), "already in agreement");
}

#[test]
fn a_masterwork_main_hand_opens_its_own_technique() {
    let d = data();
    let mut gs = game(&d, FROM_ACT);
    weapons::sync(&d, &mut gs);
    let open = |gs: &GameState| progression::build(&d, hero(gs)).expect("builds").unit.techniques.iter().any(|t| t.skill == OWN_TECHNIQUE);
    assert!(!open(&gs), "a plain instance opens nothing");
    gs.party.members[0].weapons.get_mut("main_hand").unwrap().masterwork = true;
    assert!(weapons::masterwork_main(hero(&gs)));
    let u = progression::build(&d, hero(&gs)).expect("builds").unit;
    assert!(u.masterwork && open(&gs), "the masterwork blade's own technique opens");
    // an instance of another item than the slot holds is not wielded
    gs.party.members[0].equipment.insert("main_hand".into(), "heirloom".into());
    assert!(!weapons::masterwork_main(hero(&gs)) && !open(&gs));
}

#[test]
fn a_save_writes_the_instances_in_agreement() {
    let d = data();
    let mut gs = game(&d, FROM_ACT);
    // a save of a state never synced (an Act 2 save made before the data named the act line) converts on load
    let raw = gs.to_value();
    assert!(raw["party"].get(STASH_KEY).is_none());
    let loaded = save::decode(&d, &envelope(&raw)).expect("loads").state;
    weapons::sync(&d, &mut gs);
    assert_eq!(loaded.to_value(), gs.to_value(), "the load converts as the act line does");
    let dir = aegis_core::scratch::Scratch::new("rpgkit_weapon_instances_save");
    let mut stale = gs.clone();
    stale.party.add_to_stash("blade", 1);
    assert!(save::write(&d, dir.path(), &stale, 1, 5).is_empty());
    let back = save::read(&d, dir.path(), 1).expect("reads back").state;
    assert_eq!(count(&back, "blade"), count(&gs, "blade") + 1, "the save made the new blade an instance");
    uids_unique(&back);
}

#[test]
fn a_huge_uid_number_never_overflows_the_next_uid() {
    let d = data();
    let mut gs = game(&d, FROM_ACT);
    weapons::sync(&d, &mut gs);
    let huge = format!("{}{}", weapons::UID_PREFIX, u64::MAX);
    gs.party.stash_weapons[0].uid = huge.clone();
    let r = shop::buy(&d, &mut gs, "store", "item", "blade", 1);
    assert!(json::b(&r, "ok", false), "{r:?}");
    assert!(weapons::find(&gs.party, &huge).is_some(), "a long uid stays valid");
    uids_unique(&gs);
}
