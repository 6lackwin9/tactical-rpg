//! A1.2-K3 (INTENT2-K3 "Stacks & Auto-Fill", INTENT2-U data half) on scratch worlds: a pouch slot's packed target
//! (`pouch_packed`) defaults to `min(5, max_stack)` when an item is assigned (even with a short stash), is adjustable
//! `1..min(5, max_stack)`, a second slot of the same item and a fourth slot are refused, and it saves only when set (a
//! member without it saves as before; missing reads the current pouch). Auto-fill (`roster::auto_fill`) runs only
//! under the world's `pouch_stacks` (a plain tunable the row sets; off: nothing): it tops pouches up from
//! the stash one copy at a time in round-robin passes, the higher packed target first (ties by party slot order), and
//! names each slot left short (`Stash short: <name> <cur> of <packed> <item>`). At combat start a slot without a target
//! takes its current count. The member validation accepts both pouch forms.

use rpgkit_battle::encounter::PartyFate;
use rpgkit_world::roster::{self, FightEnd, HeroEnd};
use rpgkit_world::state::{UnitState, POUCH_PACKED_KEY};
use rpgkit_world::{Data, GameState};
use serde_json::json;

/// Items: a potion (max_stack 3), a bomb (2), a salve with a large `max_stack` (capped at 5), a sword (not a consumable).
/// World `t` plays a round_intent ruleset, `c` classic.
fn data() -> Data {
    let item = |id: &str, max_stack: i64| json!({"id": id, "name": format!("N_{id}"), "item_type": "CONSUMABLE", "consumable_type": "HEALTH_POTION", "heal": 10, "max_stack": max_stack});
    let catalog = json!({
        "items": [item("p_potion", 3), item("p_bomb", 2), item("p_salve", 99), {"id": "p_sword", "name": "Sword", "item_type": "WEAPON"}],
        "rulesets": [{"id": "timeline", "name": "Timeline", "rules": ["round_intent"], "pouch_stacks": "on"}],
    });
    let worlds = json!({
        "t": {"manifest": {"id": "t", "ruleset": "timeline"}, "own": {}},
        "c": {"manifest": {"id": "c"}, "own": {}}
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn member(id: &str, pouch: &[&str]) -> UnitState {
    UnitState { id: id.into(), pouch: pouch.iter().map(|s| s.to_string()).collect(), ..UnitState::default() }
}

fn game(members: Vec<UnitState>, stash: &[(&str, i64)]) -> GameState {
    let mut gs = GameState::default();
    gs.party.members = members;
    for (id, n) in stash {
        gs.party.add_to_stash(id, *n);
    }
    gs
}

fn ids(gs: &GameState) -> Vec<String> {
    gs.party.members.iter().map(|m| m.id.clone()).collect()
}

#[test]
fn assigning_defaults_the_packed_target_and_it_adjusts_within_the_stack() {
    let d = data();
    let mut m = member("m1", &[]);
    m.assign_pouch(&d, "p_potion").unwrap();
    assert_eq!(m.packed("p_potion"), 3, "min(5, max_stack) with an empty stash");
    m.assign_pouch(&d, "p_salve").unwrap();
    assert_eq!(m.packed("p_salve"), 5, "capped at 5");
    assert!(m.assign_pouch(&d, "p_potion").is_err(), "no second slot of the same item");
    assert!(m.assign_pouch(&d, "p_sword").is_err(), "not a consumable");
    m.assign_pouch(&d, "p_bomb").unwrap();
    assert_eq!(m.pouch_slots(), ["p_potion", "p_salve", "p_bomb"]);
    assert!(member("m9", &["p_potion", "p_bomb", "p_salve"]).assign_pouch(&d, "p_potion").is_err(), "already slotted");
    assert!(m.set_packed(&d, "p_potion", 1).is_ok() && m.packed("p_potion") == 1);
    assert!(m.set_packed(&d, "p_potion", 0).is_err() && m.set_packed(&d, "p_potion", 4).is_err(), "1..min(5, max_stack)");
    assert!(m.set_packed(&d, "p_sword", 1).is_err(), "no slot");
    assert_eq!(member("m2", &["p_bomb", "p_bomb"]).packed("p_bomb"), 2, "missing reads the current pouch");
}

#[test]
fn pouch_packed_saves_only_when_set() {
    let d = data();
    let plain = member("m1", &["p_potion"]);
    assert!(plain.to_value().get(POUCH_PACKED_KEY).is_none(), "a member without targets saves as before");
    let mut m = plain.clone();
    m.assign_pouch(&d, "p_bomb").unwrap();
    let saved = m.to_value();
    assert_eq!(saved.get(POUCH_PACKED_KEY), Some(&json!({"p_bomb": 2})));
    let back = UnitState::from_dict(saved.as_object().unwrap());
    assert_eq!(back.pouch_packed, m.pouch_packed);
}

#[test]
fn the_member_check_takes_both_pouch_forms() {
    let d = data();
    let errs = |m: &UnitState| m.pouch_errors(&d, "m");
    assert!(errs(&member("m", &["p_potion", "p_potion", "p_bomb"])).is_empty(), "classic: 3 copies");
    assert!(errs(&member("m", &["p_potion", "p_potion", "p_potion", "p_bomb", "p_bomb"])).is_empty(), "stacks within their limits");
    assert_eq!(errs(&member("m", &["p_bomb", "p_bomb", "p_bomb", "p_potion"])).len(), 1, "a bomb stack above 2");
    let mut m = member("m", &[]);
    m.pouch_packed.insert("p_bomb".into(), 3);
    assert_eq!(errs(&m).len(), 1, "a packed target above the stack");
}

#[test]
fn auto_fill_round_robins_the_higher_target_first() {
    let d = data();
    let mut a = member("a", &[]);
    let mut b = member("b", &[]);
    let mut c = member("c", &[]);
    a.pouch_packed.insert("p_potion".into(), 1);
    b.pouch_packed.insert("p_potion".into(), 3);
    c.pouch_packed.insert("p_potion".into(), 3);
    let mut gs = game(vec![a, b, c], &[("p_potion", 4)]);
    let members = ids(&gs);
    let lines = roster::auto_fill(&d, &mut gs, "t", &members);
    let counts: Vec<i64> = gs.party.members.iter().map(|m| m.pouch_count("p_potion")).collect();
    // pass 1: b, c (3), a (1) -> 1 each; pass 2: b, c -> stash empty after b
    assert_eq!(counts, [1, 2, 1], "{lines:?}");
    assert_eq!(gs.party.stash.get("p_potion").copied().unwrap_or(0), 0);
    assert_eq!(lines, ["Stash short: b 2 of 3 N_p_potion", "Stash short: c 1 of 3 N_p_potion"]);
    let mut gs = game(vec![member("a", &["p_potion"])], &[("p_potion", 9)]);
    gs.party.members[0].pouch_packed.insert("p_potion".into(), 3);
    let members = ids(&gs);
    assert!(roster::auto_fill(&d, &mut gs, "t", &members).is_empty(), "full: no line");
    assert_eq!((gs.party.members[0].pouch_count("p_potion"), gs.party.stash["p_potion"]), (3, 7));
}

#[test]
fn classic_does_not_auto_fill_and_start_fixes_missing_targets() {
    let d = data();
    let mut a = member("a", &["p_bomb"]);
    a.pouch_packed.insert("p_bomb".into(), 2);
    let mut gs = game(vec![a], &[("p_bomb", 5)]);
    let members = ids(&gs);
    assert!(roster::auto_fill(&d, &mut gs, "c", &members).is_empty());
    assert_eq!(gs.party.members[0].pouch_count("p_bomb"), 1, "classic: nothing moves");
    let mut gs = game(vec![member("a", &["p_bomb"])], &[("p_bomb", 5)]);
    assert!(roster::auto_fill_at_start(&d, &mut gs, "t").is_empty());
    assert_eq!(gs.party.members[0].pouch_packed.get("p_bomb"), Some(&1), "the current count becomes the target");
    gs.party.members[0].pouch.clear();
    let members = ids(&gs);
    roster::auto_fill(&d, &mut gs, "t", &members);
    assert_eq!(gs.party.members[0].pouch_count("p_bomb"), 1, "a spent slot is refilled from the stash");
}

fn end(member: &str, pouch: &[&str], fate: PartyFate) -> HeroEnd {
    HeroEnd { member: member.into(), hp: 10, max_hp: 10, alive: true, pouch: pouch.iter().map(|s| s.to_string()).collect(), unloaded: Vec::new(), fate: Some(fate), marks: Vec::new() }
}

/// INTENT2-K3 at battle end: `settle_party` auto-fills only the members who came home, after every departure: a stayer
/// captured by `stay_behind.then` takes nothing (its pouch goes to the stash) and is not named short; the survivor's
/// short slot is the one `Stash short` line (the `stash_short` result).
#[test]
fn settle_party_fills_only_the_survivors() {
    let d = data();
    let mut home = member("home", &[]);
    home.pouch_packed.insert("p_potion".into(), 3);
    let mut gone = member("gone", &["p_potion"]);
    gone.pouch_packed.insert("p_potion".into(), 3);
    let mut gs = game(vec![home, gone], &[("p_potion", 1)]);
    gs.world_id = "t".into();
    let f = FightEnd { outcome: "won".into(), encounter: "t:fight".into(), world: "t".into(), escape: false, then: "capture".into() };
    let ends = [end("home", &[], PartyFate::Standing), end("gone", &["p_potion"], PartyFate::Stayed)];
    let settled = roster::settle_party(&d, &mut gs, &f, &ends);
    assert_eq!(settled.departed, [("gone".to_string(), "captured".to_string())]);
    assert!(gs.party.member("gone").is_none());
    // the stash held 1 and got the stayer's 1: the survivor takes both, 2 of 3
    assert_eq!(gs.party.member("home").unwrap().pouch_count("p_potion"), 2);
    assert_eq!(settled.stash_short, ["Stash short: home 2 of 3 N_p_potion"]);
}

/// The kit's world fight start (`battle::begin_fight`) runs the combat-start auto-fill (its lines returned) after the
/// party_min check; a refused fight moves nothing.
#[test]
fn the_world_fight_start_auto_fills() {
    let d = data();
    let mut m = member("m", &[]);
    m.pouch_packed.insert("p_bomb".into(), 2);
    let mut gs = game(vec![m], &[("p_bomb", 1)]);
    gs.world_id = "t".into();
    let (world, lines) = rpgkit_world::battle::begin_fight(&d, &mut gs, "nowhere").unwrap();
    assert_eq!(world, "t");
    assert_eq!(gs.party.members[0].pouch_count("p_bomb"), 1);
    assert_eq!(lines, ["Stash short: m 1 of 2 N_p_bomb"]);
}

/// INTENT2-U: the slot order is the packed targets' order, then the slots without one; moving a slot keeps its place
/// (every slot then has a target), taking a slot gives its copies back, and `pack_pouch` gives back the copies above the
/// new target.
#[test]
fn slots_keep_their_order_and_the_slot_edits_return_copies() {
    let d = data();
    let mut m = member("m1", &["p_bomb", "p_bomb"]);
    m.assign_pouch(&d, "p_potion").unwrap();
    assert_eq!(m.pouch_slots(), ["p_potion", "p_bomb"], "a packed slot first, then the pouch ids without a target");
    m.move_pouch_slot(&d, 1, 0).unwrap();
    assert_eq!(m.pouch_slots(), ["p_bomb", "p_potion"], "the moved slot keeps its place");
    assert_eq!((m.packed("p_bomb"), m.pouch_count("p_bomb")), (2, 2), "a slot without a target takes its count");
    assert!(m.move_pouch_slot(&d, 1, 1).is_err() && m.move_pouch_slot(&d, 5, 0).is_err(), "nothing to move");
    m.assign_pouch(&d, "p_salve").unwrap();
    m.move_pouch_slot(&d, 2, 0).unwrap();
    assert_eq!(m.pouch_slots(), ["p_salve", "p_bomb", "p_potion"]);
    let mut stash = indexmap::IndexMap::new();
    m.pack_pouch(&d, "p_bomb", 1, &mut stash).unwrap();
    assert_eq!((m.pouch_count("p_bomb"), m.packed("p_bomb"), stash["p_bomb"]), (1, 1, 1), "the copy above 1 back");
    assert!(m.pack_pouch(&d, "p_bomb", 3, &mut stash).is_err(), "above the stack limit");
    assert_eq!(m.take_pouch_slot(1, &mut stash).unwrap(), "p_bomb");
    assert_eq!((m.pouch_count("p_bomb"), stash["p_bomb"]), (0, 2), "its copies back");
    assert_eq!(m.pouch_slots(), ["p_salve", "p_potion"], "and its target gone");
    assert!(m.take_pouch_slot(2, &mut stash).is_err(), "no such slot");
}
