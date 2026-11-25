//! PR6-9 (`docs/sessions/PK4_PK5_CAMPAIGN_HUB.md` 5, B6) on a scratch world built here (no shipped data): the
//! `sieges` world kind and its runner. The effect `start_siege` opens the siege at its first room (`SiegeState`, saved
//! as `siege` only while one runs); a won room that is not the last carries the party (exact HP, no rally; a hero left
//! at 0 HP stays downed and sits the next room out until triaged; the pouches are not topped up); no rest while it runs;
//! one re-stock between rooms; a lost room ends the siege, lowers its facility one level (never below level 1) and runs
//! the loss effects; the last room won runs the win effects. Expectations come from the row's own constants and the
//! kit's HP rules (`progression::max_hp_of`, `hp_after`).

use aegis_core::json::{self, Dict};
use rpgkit_battle::encounter::PartyFate;
use rpgkit_data::facility::LEVEL_MIN;
use rpgkit_world::battle::{self, Tally};
use rpgkit_world::roster::HeroEnd;
use rpgkit_world::siege::{self, SiegeState};
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{campaign, explore, progression, Data, GameState};
use serde_json::{json, Value};

const WON_FLAG: &str = "w.siege_won";
const LOST_FLAG: &str = "w.siege_lost";
const RESTOCKS: i64 = 1;
const SALTS_HEAL: i64 = 15;
const LEAD_END_HP: i64 = 10;

fn with_ids(w: &str, rows: Value) -> Value {
    Value::Array(
        json::arr(&rows)
            .iter()
            .map(|r| {
                let mut r = json::dict(r).clone();
                let id = json::s(&r, "id");
                r.insert("_world".into(), w.into());
                r.insert("_qid".into(), format!("{w}:{id}").into());
                Value::Object(r)
            })
            .collect(),
    )
}

/// World `w` (a campaign; rest at the hub): the siege `hold` (three rooms; the second guards the facility `vault`),
/// two party characters and a rat.
fn data() -> Data {
    let worlds = json!({"w": {"manifest": {"id": "w", "start_location": "hub", "start_party": ["lead"], "campaign": {"start_act": 1},
        "rest": {"days": 1, "cost": 0, "where": "rest_places", "on_enter": false, "needs_upkeep": false}}, "own": {
        "locations": with_ids("w", json!([{"id": "hub", "name": "Hub", "rest": true}])),
        "encounters": with_ids("w", json!([
            {"id": "grate", "name": "Grate", "enemies": ["rat"]},
            {"id": "lock", "name": "Lock", "enemies": ["rat"]},
            {"id": "hearth", "name": "Hearth", "enemies": ["rat"]}
        ])),
        "characters": with_ids("w", json!([
            {"id": "lead", "name": "Lead", "role": "party", "base_unit": "hero"},
            {"id": "second", "name": "Second", "role": "party", "base_unit": "hero"},
            {"id": "rat", "name": "Rat", "role": "npc", "base_unit": "rat"},
            {"id": "keeper", "name": "Keeper", "role": "npc"}
        ])),
        "facilities": with_ids("w", json!([{"id": "vault", "name": "Vault", "keeper_character": "keeper", "levels": [
            {"level": 1, "name": "Bare", "scrap_cost": 0}, {"level": 2, "name": "Fitted", "scrap_cost": 0}, {"level": 3, "name": "Grand", "scrap_cost": 0}]}])),
        "sieges": with_ids("w", json!([{"id": "hold", "name": "The Hold", "restocks": RESTOCKS, "rooms": [
            {"encounter": "grate", "name": "Grate"}, {"encounter": "lock", "name": "Lock", "facility": "vault"}, {"encounter": "hearth", "name": "Hearth"}],
            "win_effects": [{"effect": "set_flag", "key": WON_FLAG, "value": true}],
            "loss_effects": [{"effect": "set_flag", "key": LOST_FLAG, "value": true}]}]))
    }}});
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}
        ],
        "units": [
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 3},
            {"id": "rat", "name": "Rat", "side": "enemy", "class_id": "drill", "level": 1}
        ],
        "items": [
            {"id": "salts", "name": "Salts", "item_type": "CONSUMABLE", "value": 5, "revive": true, "heal": SALTS_HEAL},
            {"id": "bread", "name": "Bread", "item_type": "CONSUMABLE", "value": 1, "heal": 5}
        ]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// Lead and Second fielded at the hub.
fn state(d: &Data) -> GameState {
    let members = vec![UnitState::create("lead", "hero"), UnitState::create("second", "hero")];
    let mut gs = GameState::create(d, "w", PartyState { members, active_ids: vec!["lead".into(), "second".into()], ..PartyState::default() });
    explore::enter(d, &mut gs, "hub", explore::SPAWN);
    gs
}

/// `gs` with the siege opened through the shared effect body.
fn besieged(d: &Data) -> GameState {
    let mut gs = state(d);
    let mut events = Vec::new();
    let notes = campaign::apply_effects(d, &mut gs, &[effect("hold")], &mut events);
    assert!(notes.is_empty(), "{notes:?}");
    gs
}

fn effect(siege: &str) -> Dict {
    json::dict(&json!({"effect": "start_siege", "siege": siege})).clone()
}

fn max_hp(d: &Data, gs: &GameState, id: &str) -> i64 {
    progression::max_hp_of(d, gs.party.member(id).unwrap(), "w")
}

/// A fielded member's end: alive at `hp`, or downed at 0.
fn end(member: &str, hp: i64, max: i64) -> HeroEnd {
    HeroEnd {
        member: member.into(),
        hp,
        max_hp: max,
        alive: hp > 0,
        pouch: Vec::new(),
        unloaded: Vec::new(),
        fate: Some(if hp > 0 { PartyFate::Standing } else { PartyFate::Downed }),
        marks: Vec::new(),
        hostile: String::new(),
        attacked: Vec::new(),
        worn: Vec::new(),
        strain_peak: 0,
        strain_end: 0,
        channel_actions: 0,
    }
}

/// The room's fight settled with `outcome` and the party ends `party`.
fn settle_room(d: &Data, gs: &mut GameState, outcome: &str, party: Vec<HeroEnd>) -> Dict {
    let fight = siege::fight(d, gs).expect("a room is up");
    let tally = Tally { party, world: "w".into(), ..Tally::default() };
    battle::finish_world(d, gs, siege::FIGHT_SOURCE, &fight, outcome, Some(&tally))
}

#[test]
fn start_siege_opens_the_first_room_once() {
    let d = data();
    let mut gs = state(&d);
    let mut events = Vec::new();
    let notes = campaign::apply_effects(&d, &mut gs, &[effect("hold")], &mut events);
    assert!(notes.is_empty(), "{notes:?}");
    assert_eq!(gs.siege, Some(SiegeState { siege: "w:hold".into(), room: 0, restocks: RESTOCKS, downed: Vec::new() }));
    assert!(events.iter().any(|e| json::s(json::dict(e), "event") == "siege_started"));
    let fight = siege::fight(&d, &gs).unwrap();
    assert_eq!((json::s(&fight, "source"), json::s(&fight, "encounter")), (siege::FIGHT_SOURCE.to_string(), "w:grate".to_string()));
    // a second start and an unknown siege are refused with a note (the siege under way is kept)
    assert_eq!(campaign::apply_effects(&d, &mut gs, &[effect("hold")], &mut events), [siege::RUNNING_TEXT.to_string()]);
    let mut fresh = state(&d);
    assert_eq!(campaign::apply_effects(&d, &mut fresh, &[effect("nope")], &mut events), ["Unknown siege 'nope'.".to_string()]);
    assert_eq!(fresh.siege, None);
}

#[test]
fn the_siege_is_saved_only_while_it_runs() {
    let d = data();
    let gs = state(&d);
    assert!(!json::dict(&gs.to_value()).contains_key("siege"), "no siege: no key (older saves byte-identical)");
    let mut gs = besieged(&d);
    gs.siege.as_mut().unwrap().downed.push("second".into());
    let saved = gs.to_value();
    assert_eq!(json::dget(json::dict(&saved), "siege").get("room"), Some(&json!(0)));
    let back = GameState::from_dict(&d, json::dict(&saved));
    assert_eq!(back.siege, gs.siege);
    assert!(back.load_problems.is_empty(), "{:?}", back.load_problems);
}

#[test]
fn a_won_room_carries_exact_hp_and_the_downed() {
    let d = data();
    let mut gs = besieged(&d);
    let (lead_max, second_max) = (max_hp(&d, &gs, "lead"), max_hp(&d, &gs, "second"));
    assert!(progression::hp_after(LEAD_END_HP, lead_max, "won") > LEAD_END_HP, "outside a siege a win rallies");
    let res = settle_room(&d, &mut gs, "won", vec![end("lead", LEAD_END_HP, lead_max), end("second", 0, second_max)]);
    assert!(json::b(&res, "ok", false), "{res:?}");
    let st = gs.siege.clone().expect("the siege goes on");
    assert_eq!((st.room, st.downed.clone()), (1, vec!["second".to_string()]));
    assert_eq!(gs.party.member("lead").unwrap().hp_lost, lead_max - LEAD_END_HP, "exact HP, no rally");
    assert_eq!(gs.party.member("second").unwrap().hp_lost, second_max, "downed stays down");
    assert_eq!(json::s(json::dget(&res, "siege"), "state"), "next");
    // the next room fields only the members still standing
    let fight = siege::fight(&d, &gs).unwrap();
    assert_eq!(json::s(&fight, "encounter"), "w:lock");
    let wf = battle::begin_world(&d, &mut gs, &fight).expect("the room starts");
    assert_eq!(wf.members.iter().map(|(m, _)| m.as_str()).collect::<Vec<_>>(), ["lead"]);
}

#[test]
fn no_rest_while_the_siege_runs_and_the_pouch_is_held_between_rooms() {
    let d = data();
    let mut gs = state(&d);
    assert_ne!(campaign::purse::rest_refusal(&d, &gs), siege::NO_CAMP_TEXT, "no siege refusal before the siege");
    gs = besieged(&d);
    assert_eq!(campaign::purse::rest_refusal(&d, &gs), siege::NO_CAMP_TEXT);
    assert_eq!(progression::can_camp(&d, &gs), siege::NO_CAMP_TEXT);
    // room 1 of 3: no top-up after it, the first start fills as usual
    assert!(siege::holds_pouch(&d, &gs, true) && !siege::holds_pouch(&d, &gs, false));
    gs.siege.as_mut().unwrap().room = 2;
    assert!(!siege::holds_pouch(&d, &gs, true), "the last room settles as usual");
    assert!(siege::holds_pouch(&d, &gs, false), "a later room starts with the pouches as they were");
}

#[test]
fn one_restock_between_rooms_and_triage_revives_with_a_stash_item() {
    let d = data();
    let mut gs = besieged(&d);
    assert_eq!(json::sget(&siege::restock(&d, &mut gs), "errors"), [siege::FIRST_ROOM_TEXT]);
    let second_max = max_hp(&d, &gs, "second");
    let lead_max = max_hp(&d, &gs, "lead");
    settle_room(&d, &mut gs, "won", vec![end("lead", lead_max, lead_max), end("second", 0, second_max)]);
    let res = siege::restock(&d, &mut gs);
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(gs.siege.as_ref().unwrap().restocks, RESTOCKS - 1);
    assert_eq!(json::sget(&siege::restock(&d, &mut gs), "errors"), [siege::NO_RESTOCK_TEXT]);
    // triage: not with an item that cannot revive, nor without one in the stash
    assert_eq!(json::sget(&siege::triage(&d, &mut gs, "second", "bread"), "errors"), [siege::NO_REVIVE_TEXT]);
    assert_eq!(json::sget(&siege::triage(&d, &mut gs, "second", "salts"), "errors"), ["Not enough Salts.".to_string()]);
    gs.party.stash.insert("salts".into(), 2);
    assert_eq!(json::sget(&siege::triage(&d, &mut gs, "lead", "salts"), "errors"), [siege::NOT_DOWNED_TEXT]);
    let res = siege::triage(&d, &mut gs, "second", "salts");
    assert_eq!(json::i(&res, "hp", 0), SALTS_HEAL.min(second_max));
    assert_eq!(gs.party.member("second").unwrap().hp_lost, second_max - SALTS_HEAL.min(second_max));
    assert!(gs.siege.as_ref().unwrap().downed.is_empty());
    assert_eq!(gs.party.stash.get("salts").copied(), Some(1));
}

#[test]
fn a_lost_room_ends_the_siege_and_damages_its_facility() {
    let d = data();
    let mut gs = besieged(&d);
    gs.facilities.insert("w:vault".into(), 3);
    let lead_max = max_hp(&d, &gs, "lead");
    settle_room(&d, &mut gs, "won", vec![end("lead", lead_max, lead_max), end("second", lead_max, lead_max)]);
    let res = settle_room(&d, &mut gs, "lost", vec![end("lead", 0, lead_max), end("second", 0, lead_max)]);
    assert_eq!(gs.siege, None, "the siege is over");
    assert_eq!(gs.facilities.get("w:vault").copied(), Some(2), "the room's facility drops one level");
    assert!(gs.flag_true(LOST_FLAG) && !gs.flag_true(WON_FLAG));
    let sg = json::dget(&res, "siege");
    assert_eq!(json::s(sg, "state"), "lost");
    assert!(json::sget(sg, "lines").contains(&"Vault is damaged.".to_string()));
    assert_ne!(campaign::purse::rest_refusal(&d, &gs), siege::NO_CAMP_TEXT, "no siege refusal once it is over");
    // a facility at level 2 drops to the first level (no saved key); at the first level it stays
    let mut gs = besieged(&d);
    gs.facilities.insert("w:vault".into(), 2);
    gs.siege.as_mut().unwrap().room = 1;
    settle_room(&d, &mut gs, "fled", vec![end("lead", 5, lead_max)]);
    assert!(!gs.facilities.contains_key("w:vault"));
    assert_eq!(rpgkit_world::facility::level(&gs, "w:vault"), LEVEL_MIN);
}

#[test]
fn the_last_room_won_runs_the_win_effects() {
    let d = data();
    let mut gs = besieged(&d);
    gs.siege.as_mut().unwrap().room = 2;
    let lead_max = max_hp(&d, &gs, "lead");
    let res = settle_room(&d, &mut gs, "won", vec![end("lead", 1, lead_max)]);
    assert_eq!(gs.siege, None);
    assert!(gs.flag_true(WON_FLAG) && !gs.flag_true(LOST_FLAG));
    assert_eq!(json::s(json::dget(&res, "siege"), "state"), "won");
    // the last room settles as any fight: the win rallies
    assert_eq!(gs.party.member("lead").unwrap().hp_lost, lead_max - progression::hp_after(1, lead_max, "won"));
}

#[test]
fn with_every_fighter_down_the_room_is_yielded() {
    let d = data();
    let mut gs = besieged(&d);
    gs.siege.as_mut().unwrap().downed = vec!["lead".into(), "second".into()];
    assert_eq!(siege::fight_refusal(&gs), siege::ALL_DOWN_TEXT);
    let fight = siege::fight(&d, &gs).unwrap();
    assert_eq!(battle::begin_world(&d, &mut gs, &fight).err().as_deref(), Some(siege::ALL_DOWN_TEXT));
    assert_eq!(battle::start_refusal(&d, &gs, "grate"), siege::ALL_DOWN_TEXT);
    let res = siege::forfeit(&d, &mut gs);
    assert_eq!(json::s(json::dget(&res, "siege"), "state"), "lost");
    assert!(gs.siege.is_none() && gs.flag_true(LOST_FLAG));
}

#[test]
fn the_view_reads_rooms_downed_and_items() {
    let d = data();
    let mut gs = besieged(&d);
    gs.siege.as_mut().unwrap().room = 1;
    gs.siege.as_mut().unwrap().downed = vec!["second".into()];
    gs.party.stash.insert("salts".into(), 1);
    let v = siege::view(&d, &gs).unwrap();
    let v = json::dict(&v);
    let states: Vec<String> = json::aget(v, "rooms").iter().map(|r| json::s(json::dict(r), "state")).collect();
    assert_eq!(states, ["cleared", "next", "ahead"]);
    assert_eq!(json::s(v, "name"), "The Hold");
    assert_eq!(json::aget(v, "downed").len(), 1);
    assert_eq!(json::s(json::dict(&json::aget(v, "items")[0]), "id"), "salts");
    assert!(json::b(v, "can_restock", false) && json::b(v, "no_camp", false));
    assert!(siege::view(&d, &state(&d)).is_none());
}

#[test]
fn a_withdrawn_foe_is_counted_apart_from_the_slain() {
    use rpgkit_battle::session::Fate;
    use rpgkit_world::battle::Foe;
    let d = data();
    let defeated = |fate: Fate| {
        let mut gs = state(&d);
        let tally = Tally { foes: vec![Foe { key: "rat#0".into(), unit_ref: "rat".into(), fate }], ..Tally::default() };
        let res = battle::settle(&d, &mut gs, "w:grate", "won", Some(&tally));
        json::dget(json::dget(&res, "rewards"), "defeated").clone()
    };
    let w = defeated(Fate::Withdrawn);
    assert_eq!((json::i(&w, "withdrawn", 0), json::i(&w, "slain", 0), json::i(&w, "fled", 0)), (1, 0, 0));
    assert!(!defeated(Fate::Killed).contains_key("withdrawn"), "the key only when a foe withdrew (results unchanged otherwise)");
}
