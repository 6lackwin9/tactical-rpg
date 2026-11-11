//! PR5-31 (`LAYER-UP-PARTY`) on scratch terms and a scratch world (no shipped data): the party screen's edit rules in
//! `party_ops` and `roster::toggle_active`. The deck terms (RULESET1 9 D1-D6): a bucket takes its own kind up to the
//! ruleset's slots, a first actives edit pins the automatic deck while a passive edit keeps the actives automatic, an
//! active leaving the deck takes its `channeled` mark, order and follow-up places with it, a follow-up joins only an
//! opener sharing one of its rows, the last active stays. The classic slots (SKILLUI1): class slots take class path
//! families only, `MAX_CLASS_SLOTS` + `MAX_EXTRA_SLOTS`, a full class group spills to a free extra slot, ranks follow the
//! slotted families. The gear moves: a two-handed main hand stows an off-hand item it can't hold and refuses one after,
//! a two-handed default coming back clears the off hand. The pouch: classic at most `POUCH_MAX` items; under
//! `pouch_stacks` a stack slot with its packed target, replacing and taking the dropped place. The squad: one to
//! `max_active` active members.

use indexmap::IndexMap;
use rpgkit_data::Deck;
use rpgkit_rules::ruleset::DeckSlots;
use rpgkit_rules::stats::{MAX_CLASS_SLOTS, MAX_EXTRA_SLOTS};
use rpgkit_world::party_ops::{self, can_assign_to_slot, drop_move, equip, pouch_put, take_off, DeckTerms, SkillSlots, Stash, DECK_KINDS};
use rpgkit_world::state::{off_hand_ok, stack_limit, PartyState, UnitState, POUCH_MAX};
use rpgkit_world::{roster, Data};
use serde_json::json;

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

// ---- the deck terms ------------------------------------------------------------------------------------------------

/// Scratch terms (test-only ids): actives over two intents, two passives, a keystone, one basic per intent; the auto
/// deck slots `cleave` and `wall`, an empty active row plays as `lunge`.
fn terms(slots: DeckSlots) -> DeckTerms {
    DeckTerms {
        slots,
        intents: strs(&["assault", "guard"]),
        basics: strs(&["strike", "hold"]),
        actives: strs(&["cleave", "lunge", "wall", "rush"]),
        passives: strs(&["p1", "p2"]),
        keystones: strs(&["k1"]),
        intent_of: [("cleave", "assault"), ("lunge", "assault"), ("rush", "assault"), ("wall", "guard"), ("strike", "assault"), ("hold", "guard")].iter().map(|(a, b)| (a.to_string(), b.to_string())).collect(),
        rows_of: IndexMap::new(),
        auto: Deck { active: strs(&["cleave", "wall"]), ..Default::default() },
        auto_actives: strs(&["lunge"]),
        intent_names: IndexMap::new(),
    }
}

#[test]
fn a_deck_bucket_takes_its_own_kind_up_to_the_rulesets_slots() {
    let t = terms(DeckSlots { active: 3, passive: 1, keystone: 1 });
    for (kind, _, _) in DECK_KINDS {
        let cap = t.cap(kind);
        let mut d = Deck::default();
        // fill the bucket from what is owned (an actives edit starts from the auto deck's actives)
        for id in t.owned(kind).to_vec() {
            if DeckTerms::list(&t.shown(&d), kind).contains(&id) {
                continue;
            }
            match t.slot(&d, kind, &id) {
                Ok(next) => d = next,
                Err(e) => {
                    assert_eq!(DeckTerms::list(&t.shown(&d), kind).len(), cap, "{kind}: refused only when full ({e})");
                    assert!(e.contains("are full"), "{kind}: {e}");
                }
            }
        }
        assert!(DeckTerms::list(&d, kind).len() <= cap, "{kind}: never above its slots");
    }
    // a bucket takes its own kind only; an unowned id is refused
    assert!(t.after_drop(&Deck::default(), "deck.own.passive.p1", "deck.active.0").unwrap_err().contains("active skills only"));
    assert!(t.slot(&Deck::default(), "active", "nobody").unwrap_err().contains("not owned"));
}

#[test]
fn a_first_actives_edit_pins_the_automatic_deck_and_a_passive_edit_keeps_it_automatic() {
    let t = terms(DeckSlots { active: 3, passive: 1, keystone: 1 });
    let empty = Deck::default();
    assert_eq!(t.shown(&empty), t.auto, "an empty deck plays as the auto deck");
    let p = t.slot(&empty, "passive", "p1").unwrap();
    assert!(p.active.is_empty(), "a passive edit keeps the actives automatic");
    assert_eq!(t.shown(&p).active, t.auto_actives, "an empty active row plays as the auto actives");
    let a = t.slot(&empty, "active", "rush").unwrap();
    let mut want = t.auto.active.clone();
    want.push("rush".into());
    assert_eq!(a.active, want, "an actives edit pins the auto deck, then slots");
}

#[test]
fn an_active_leaving_the_deck_takes_its_mark_order_and_follow_ups() {
    let t = terms(DeckSlots { active: 3, passive: 1, keystone: 1 });
    let mut d = Deck { active: strs(&["cleave", "lunge", "wall"]), channeled: strs(&["lunge"]), ..Default::default() };
    d.order.insert("assault".into(), strs(&["lunge", "cleave"]));
    d.follow_ups.insert("lunge".into(), strs(&["cleave"]));
    d.follow_ups.insert("cleave".into(), strs(&["lunge"]));
    let out = t.unslot(&d, "active", 1).unwrap();
    assert_eq!(out.active, strs(&["cleave", "wall"]));
    assert!(out.channeled.is_empty(), "the mark goes with it");
    assert_eq!(out.order.get("assault"), Some(&strs(&["cleave"])), "out of every order");
    assert!(!out.follow_ups.contains_key("lunge") && !out.follow_ups.contains_key("cleave"), "its own list and every emptied list go");
    // a replaced active goes the same way; a kept one keeps its mark
    let r = t.after_drop(&d, "deck.own.active.rush", "deck.active.0").unwrap();
    assert_eq!((r.active.clone(), r.channeled.clone()), (strs(&["rush", "lunge", "wall"]), strs(&["lunge"])));
    assert_eq!(r.order.get("assault"), Some(&strs(&["lunge", "rush"])), "a new active joins the end of its intent's written order");
    // the last active stays (an empty row is the automatic pick)
    let one = Deck { active: strs(&["wall"]), ..Default::default() };
    assert!(t.unslot(&one, "active", 0).unwrap_err().contains("at least one"));
}

#[test]
fn a_follow_up_joins_only_an_opener_sharing_one_of_its_rows() {
    let mut t = terms(DeckSlots { active: 3, passive: 1, keystone: 1 });
    let d = t.auto.clone();
    let op = "cleave";
    assert_eq!(t.follow_candidates(&d, op), t.order(&d, "assault"), "the opener's intent order, nothing listed yet");
    let added = t.follow_add(&d, op, "strike", None).unwrap();
    assert_eq!(t.follow_list(&added, op), strs(&["strike"]));
    let refusal = t.follow_add(&d, op, "wall", None).unwrap_err();
    assert!(refusal.contains("not Assault"), "{refusal}");
    assert!(t.follow_add(&d, "rush", "strike", None).unwrap_err().contains("not an opener"), "rush is not in the deck");
    // with a second row (A4.2-K1b) the guard skill shares one and may join
    t.rows_of.insert("wall".into(), strs(&["guard", "assault"]));
    assert_eq!(t.follow_list(&t.follow_add(&d, op, "wall", None).unwrap(), op), strs(&["wall"]));
    assert_eq!(t.follow_clear(&added, op).unwrap().follow_ups.get(op), None, "cleared: the intent's order again");
    assert!(t.order_auto(&Deck { order: [("assault".to_string(), strs(&["strike", "cleave"]))].into_iter().collect(), ..t.auto.clone() }).order.is_empty());
}

/// A ruleset with the deck rule gives a member its terms, its slots the ruleset's; classic gives none.
#[test]
fn a_members_terms_read_the_rulesets_deck_slots() {
    let d = world();
    let m = UnitState::create("hero", "plain");
    assert!(DeckTerms::for_member(&d, &m, "w", &d.ruleset("w")).is_none(), "classic: no deck terms");
    let rs = d.ruleset("tw");
    let t = DeckTerms::for_member(&d, &m, "tw", &rs).expect("the deck rule");
    for (kind, _, _) in DECK_KINDS {
        let want = match kind {
            "active" => rs_slots(&d).0,
            "passive" => rs_slots(&d).1,
            _ => rs_slots(&d).2,
        };
        assert_eq!(t.cap(kind), want as usize, "{kind}");
    }
    let path = party_ops::path_families(&d, &m, "tw");
    assert!(!t.actives.is_empty() && t.actives.iter().all(|f| path.contains(f)), "the owned actives: the class path's {:?}", t.actives);
    assert!(t.auto.active.len() <= t.cap("active") && t.auto_actives.len() <= t.cap("active"));
    assert!(t.basics.iter().all(|b| !t.actives.contains(b)), "a basic takes no slot");
}

// ---- the classic slots -----------------------------------------------------------------------------------------------

/// A class path one family longer than the class slots, and families off the path to fill the extra slots.
fn path() -> Vec<String> {
    (0..=MAX_CLASS_SLOTS).map(|i| format!("f_{i}")).collect()
}

fn off_path() -> Vec<String> {
    (0..=MAX_EXTRA_SLOTS).map(|i| format!("x_{i}")).collect()
}

#[test]
fn class_slots_take_the_class_path_only_and_a_full_class_group_spills_to_the_extra_slots() {
    let (path, off) = (path(), off_path());
    assert!(can_assign_to_slot(&path, &path[0], true) && !can_assign_to_slot(&path, &off[0], true));
    assert!(can_assign_to_slot(&path, &off[0], false) && can_assign_to_slot(&path, &path[0], false), "any family takes an extra slot");
    let rank = |_: &str| Some(1);
    let mut s = SkillSlots::default();
    for f in &path[..MAX_CLASS_SLOTS] {
        s = s.slot(&path, f, &rank).unwrap().expect("a free class slot");
    }
    assert_eq!(s.class.len(), MAX_CLASS_SLOTS);
    assert_eq!(s.slot(&path, &path[0], &rank).unwrap(), None, "already slotted: nothing changes");
    let spilled = s.slot(&path, &path[MAX_CLASS_SLOTS], &rank).unwrap().unwrap();
    assert_eq!(spilled.extra, vec![path[MAX_CLASS_SLOTS].clone()], "a full class group spills to a free extra slot");
    let mut full = s.clone();
    for f in &off[..MAX_EXTRA_SLOTS] {
        full = full.slot(&path, f, &rank).unwrap().unwrap();
    }
    assert_eq!(full.extra.len(), MAX_EXTRA_SLOTS);
    assert!(full.slot(&path, &off[MAX_EXTRA_SLOTS], &rank).unwrap_err().starts_with("Extra slots are full"));
    assert!(full.slot(&path, &path[MAX_CLASS_SLOTS], &rank).unwrap_err().starts_with("Class slots are full"));
    assert_eq!(full.ranks.len(), MAX_CLASS_SLOTS + MAX_EXTRA_SLOTS, "every slotted family stores its rank");
}

#[test]
fn a_classic_drop_replaces_swaps_or_moves_and_ranks_follow_the_slotted_families() {
    let (path, off) = (path(), off_path());
    let top = |_: &str| Some(2);
    let s = SkillSlots { class: vec![path[0].clone(), path[1].clone()], extra: vec![off[0].clone()], ranks: [(path[0].clone(), 1), (path[1].clone(), 1), (off[0].clone(), 1)].into_iter().collect() };
    let r = s.after_drop(&path, &format!("skill.{}", path[2]), "slot.class.0", &top).unwrap();
    assert_eq!(r.class[0], path[2], "replaces");
    assert!(!r.ranks.contains_key(&path[0]) && r.ranks.get(&path[2]) == Some(&2), "the replaced rank goes, the new one is its top");
    assert!(s.after_drop(&path, &format!("skill.{}", off[1]), "slot.class.0", &top).unwrap_err().contains("class path skills only"));
    assert!(s.after_drop(&path, "slot.extra.0", "slot.class.0", &top).is_err(), "an off-path family can't swap into a class slot");
    let swapped = s.after_drop(&path, "slot.class.0", "slot.class.1", &top).unwrap();
    assert_eq!(swapped.class, vec![path[1].clone(), path[0].clone()]);
    let moved = s.after_drop(&path, "slot.class.1", "slot.extra.1", &top).unwrap();
    assert_eq!((moved.class.len(), moved.extra.len()), (1, 2), "a class path family moves to a free extra slot");
    let out = s.after_drop(&path, "slot.extra.0", "list", &top).unwrap();
    assert!(out.extra.is_empty() && !out.ranks.contains_key(&off[0]), "back to the list: its rank goes");
    assert_eq!(s.after_drop(&path, "slot.class.0", "slot.class.0", &top).unwrap_err(), "Same slot.");
    let mut kept = SkillSlots { ranks: [(path[3].clone(), 1)].into_iter().collect(), ..s.clone() };
    kept.keep_slotted_ranks();
    assert!(!kept.ranks.contains_key(&path[3]));
    kept.unslot(true, 0);
    assert!(!kept.class.contains(&path[0]) && !kept.ranks.contains_key(&path[0]));
}

#[test]
fn a_rank_step_walks_the_selectable_ranks_and_stops_at_either_end() {
    let ranks = [1, 2, 3];
    assert_eq!(party_ops::draft_rank(&ranks, None), 3, "no entry: the highest");
    assert_eq!(party_ops::draft_rank(&ranks, Some(2)), 2);
    assert_eq!(party_ops::draft_rank(&[2, 3], Some(1)), 2, "below every rank: the lowest");
    assert_eq!(party_ops::draft_rank(&[], None), 0);
    assert_eq!(party_ops::rank_step(&ranks, Some(2), -1), Some(1));
    assert_eq!(party_ops::rank_step(&ranks, Some(2), 1), Some(3));
    assert_eq!(party_ops::rank_step(&ranks, None, 1), None, "past the top");
    assert_eq!(party_ops::rank_step(&ranks, Some(1), -1), None, "past the bottom");
}

// ---- the gear moves --------------------------------------------------------------------------------------------------

/// A class with a two-family path, a plain unit and one whose main hand defaults to a two-hander; a one-handed sword
/// and dagger, a two-handed maul, a shield, a focus (family rule `focus`), consumables (a potion stacking 3, a bomb 2,
/// a salve 99); world `w` classic, `tw` a ruleset with the deck rule and `pouch_stacks`.
fn world() -> Data {
    let atk = |id: &str| json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY"});
    let consumable = |id: &str, max_stack: i64| json!({"id": id, "name": format!("N_{id}"), "item_type": "CONSUMABLE", "consumable_type": "HEALTH_POTION", "heal": 10, "max_stack": max_stack});
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "skill_path": {"g_one": 1, "g_two": 1}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            atk("g_one"), atk("g_two")
        ],
        "items": [
            {"id": "sword", "name": "Sword", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword"},
            {"id": "dagger", "name": "Dagger", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "dagger"},
            {"id": "maul", "name": "Maul", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "hammer", "two_handed": true},
            {"id": "shield", "name": "Shield", "item_type": "SHIELD", "slot": "OFF_HAND", "weapon_type": "shield"},
            {"id": "focus", "name": "Focus", "item_type": "WEAPON", "slot": "OFF_HAND", "weapon_type": "orb", "family_rules": ["focus"]},
            consumable("potion", 3), consumable("bomb", 2), consumable("salve", 99)
        ],
        "units": [
            {"id": "plain", "name": "Plain", "side": "party", "class_id": "drill", "level": 1},
            {"id": "heavy", "name": "Heavy", "side": "party", "class_id": "drill", "level": 1, "main_hand": "maul"}
        ],
        "rulesets": [{"id": "timeline", "name": "Timeline", "rules": ["round_intent", "deck"], "basics": ["move", "strike"], "pouch_stacks": "on",
            "deck_slots": {"active": 2, "passive": 1, "keystone": 1}}]
    });
    let worlds = json!({
        "w": {"manifest": {"id": "w", "start_party": ["plain"]}, "own": {}},
        "tw": {"manifest": {"id": "tw", "start_party": ["plain"], "ruleset": "timeline"}, "own": {}}
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// The scratch ruleset's deck slots (active, passive, keystone), as the catalog row gives them.
fn rs_slots(d: &Data) -> (i32, i32, i32) {
    let s = d.ruleset("tw").numbers.deck_slots;
    (s.active, s.passive, s.keystone)
}

fn stash(items: &[&str]) -> Stash {
    items.iter().map(|i| (i.to_string(), 2)).collect()
}

fn worn(m: &UnitState, slot: &str) -> Option<String> {
    m.equipment.get(slot).cloned()
}

#[test]
fn a_two_handed_main_hand_stows_what_the_off_hand_cant_hold_and_refuses_it_after() {
    let d = world();
    let (sword, maul, shield) = (&d.cat.items["sword"], &d.cat.items["maul"], &d.cat.items["shield"]);
    assert!(!sword.two_handed && maul.two_handed && !off_hand_ok(Some(maul), shield), "the fixture: the rule's cases");
    let mut m = UnitState::create("hero", "plain");
    let mut st = stash(&["sword", "maul", "shield", "dagger", "focus"]);
    equip(&d, "w", &mut m, &mut st, "sword", Some("main_hand")).unwrap();
    equip(&d, "w", &mut m, &mut st, "shield", Some("off_hand")).unwrap();
    assert_eq!((st["sword"], st["shield"]), (1, 1), "one of each worn");
    equip(&d, "w", &mut m, &mut st, "maul", Some("main_hand")).unwrap();
    assert_eq!(worn(&m, "off_hand"), None, "the shield is stowed");
    assert_eq!((st["sword"], st["shield"]), (2, 2), "the sword and the shield back in the stash");
    assert!(drop_move(&d, "w", &mut m, &mut st, "stash.shield", "slot.off_hand", false).unwrap_err().contains("needs both hands"));
    assert!(equip(&d, "w", &mut m, &mut st, "focus", Some("off_hand")).unwrap_err().contains("needs both hands"), "a focus too");
    // a one-handed weapon may stay with a two-hander (off_hand_ok)
    assert!(off_hand_ok(Some(maul), &d.cat.items["dagger"]));
    equip(&d, "w", &mut m, &mut st, "dagger", Some("off_hand")).unwrap();
    assert_eq!(worn(&m, "off_hand").as_deref(), Some("dagger"));
    assert!(equip(&d, "w", &mut m, &mut st, "maul", Some("off_hand")).unwrap_err().contains("doesn't go in the off hand slot"), "a two-hander is no off-hand item");
}

#[test]
fn a_two_handed_default_coming_back_clears_the_off_hand() {
    let d = world();
    let mut m = UnitState::create("hero", "heavy");
    let mut st = stash(&["sword", "shield"]);
    equip(&d, "w", &mut m, &mut st, "sword", Some("main_hand")).unwrap();
    equip(&d, "w", &mut m, &mut st, "shield", Some("off_hand")).unwrap();
    assert_eq!(party_ops::main_weapon(&d, "w", &m).map(|i| i.id), Some("sword".to_string()));
    take_off(&d, "w", &mut m, &mut st, "main_hand").unwrap();
    assert_eq!(party_ops::main_weapon(&d, "w", &m).map(|i| i.two_handed), Some(true), "the two-handed default is back");
    assert_eq!((worn(&m, "main_hand"), worn(&m, "off_hand")), (None, None), "and the shield went with the sword");
    assert_eq!((st["sword"], st["shield"]), (2, 2));
    assert!(take_off(&d, "w", &mut m, &mut st, "main_hand").unwrap_err().contains("class default"), "nothing worn to take off");
    // a worn one-hander moves to the off hand under a one-handed main
    let mut p = UnitState::create("hero", "plain");
    let mut st = stash(&["sword", "dagger"]);
    equip(&d, "w", &mut p, &mut st, "dagger", Some("main_hand")).unwrap();
    drop_move(&d, "w", &mut p, &mut st, "equip.main_hand", "slot.off_hand", false).unwrap();
    assert_eq!((worn(&p, "main_hand"), worn(&p, "off_hand").as_deref()), (None, Some("dagger")));
}

#[test]
fn the_classic_pouch_holds_its_size_and_a_dropped_item_replaces_its_place() {
    let d = world();
    let mut m = UnitState::create("hero", "plain");
    let mut st: Stash = [("potion".to_string(), POUCH_MAX as i64 + 1), ("bomb".to_string(), 1), ("sword".to_string(), 1)].into_iter().collect();
    assert!(pouch_put(&d, &mut m, &mut st, "sword", None, false).unwrap_err().contains("battle pouch"), "gear is no consumable");
    for _ in 0..POUCH_MAX {
        pouch_put(&d, &mut m, &mut st, "potion", None, false).unwrap();
    }
    assert!(pouch_put(&d, &mut m, &mut st, "potion", None, false).unwrap_err().contains(&format!("at most {POUCH_MAX}")));
    drop_move(&d, "w", &mut m, &mut st, "stash.bomb", "pouch.0", false).unwrap();
    assert_eq!(m.pouch[0], "bomb");
    assert_eq!((st.get("bomb"), st["potion"]), (None, 2), "the replaced potion is back, the bomb's last copy gone");
}

#[test]
fn a_stack_slot_takes_its_packed_target_replaces_a_slot_and_takes_the_dropped_place() {
    let d = world();
    let mut m = UnitState::create("hero", "plain");
    let mut st = stash(&["potion", "bomb", "salve"]);
    drop_move(&d, "tw", &mut m, &mut st, "stash.potion", "pouch.0", true).unwrap();
    drop_move(&d, "tw", &mut m, &mut st, "stash.bomb", "pouch.1", true).unwrap();
    assert_eq!(m.pouch_slots(), strs(&["potion", "bomb"]));
    assert_eq!(m.packed("potion"), stack_limit(&d, "potion"), "packed to min(5, max_stack)");
    let before = m.clone();
    drop_move(&d, "tw", &mut m, &mut st, "stash.potion", "pouch.0", true).unwrap();
    assert_eq!(m, before, "dropped on its own slot: nothing changes");
    let again = drop_move(&d, "tw", &mut m, &mut st, "stash.potion", "pouch.1", true).unwrap_err();
    assert!(again.contains("N_potion") && !again.contains("'potion'"), "a second slot of one item is refused, by its name: {again}");
    drop_move(&d, "tw", &mut m, &mut st, "stash.salve", "pouch.0", true).unwrap();
    assert_eq!(m.pouch_slots(), strs(&["salve", "bomb"]), "the salve replaces the potion and takes its place");
    assert_eq!(m.packed("salve"), stack_limit(&d, "salve"));
    drop_move(&d, "tw", &mut m, &mut st, "pouch.0", "stash", true).unwrap();
    assert_eq!(m.pouch_slots(), strs(&["bomb"]));
    party_ops::pack(&d, &mut m, &mut st, "bomb", 1).unwrap();
    assert_eq!(m.packed("bomb"), 1);
    assert!(party_ops::pack(&d, &mut m, &mut st, "bomb", stack_limit(&d, "bomb") + 1).unwrap_err().contains("N_bomb"));
}

// ---- the squad -------------------------------------------------------------------------------------------------------

#[test]
fn the_active_squad_holds_one_to_max_active_members() {
    let mut party = PartyState::default();
    let cap = 2;
    party.max_active = cap;
    party.members = (0..=cap).map(|i| UnitState::create(&format!("m{i}"), "plain")).collect();
    let id = |i: i64| format!("m{i}");
    let first: Vec<String> = (0..cap).map(id).collect();
    let refused = roster::toggle_active(&mut party, &id(cap)).unwrap_err();
    assert_eq!(refused, format!("The active party holds at most {cap}."));
    assert_eq!(party.active_ids, first, "an empty list is written out as the first max_active members");
    for i in 0..cap - 1 {
        roster::toggle_active(&mut party, &id(i)).unwrap();
    }
    assert_eq!(party.active_ids, vec![id(cap - 1)]);
    assert_eq!(roster::toggle_active(&mut party, &id(cap - 1)).unwrap_err(), "Keep at least one active member.");
    roster::toggle_active(&mut party, &id(cap)).unwrap();
    assert_eq!(party.active_ids, vec![id(cap - 1), id(cap)]);
    assert_eq!(party.active_indexes().len() as i64, cap);
}
