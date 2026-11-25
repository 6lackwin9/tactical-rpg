//! PR7-41 (PT-28) on the shipped data (read only): under the deck rule a party member owns its base unit's deck
//! passives from level 1 and, from its class keystone's level (CLASSKEY1), its keystone (`progression::deck_grants`,
//! `owned_passives`); the Skills tab offers them (`DeckTerms::for_member`) and the battle unit applies them while
//! slotted. Classic builds keep `member_passives` alone. On the agent playtest's save `after_vats`
//! (`games/emberwake/scenarios/saves/`, 2026-11-18) every hero at or past the keystone level shows a keystone. Which
//! units, passives and levels are involved is read from the data, not hard-coded.

use emberwake_world::party_ops::deck::DeckTerms;
use emberwake_world::state::UnitState;
use emberwake_world::{progression, save, Data};
use std::path::{Path, PathBuf};

fn data() -> Data {
    Data::open(&Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")).unwrap().0
}

/// The world with the deck rule (the first shipped world whose ruleset has it).
fn deck_world(d: &Data) -> String {
    d.world_ids().into_iter().find(|w| progression::deck_rule(d, w).is_some()).expect("a shipped world plays the deck rule")
}

/// Every catalog party unit whose deck names a passive or keystone, with its class keystone level.
fn decked(d: &Data) -> Vec<(String, i32)> {
    d.cat
        .units
        .values()
        .filter(|u| u.side == "party" && (!u.deck.passive.is_empty() || !u.deck.keystone.is_empty()))
        .map(|u| (u.id.clone(), d.cat.classes.get(&u.class_id).and_then(|c| c.keystone.as_ref()).map_or(1, |k| k.level.max(1))))
        .collect()
}

#[test]
fn a_member_owns_its_base_deck_passives_at_once_and_its_keystone_from_the_keystone_level() {
    let d = data();
    let w = deck_world(&d);
    let rs = d.ruleset(&w);
    let units = decked(&d);
    assert!(!units.is_empty(), "the shipped catalog has party units with deck passives");
    for (id, key_level) in units {
        let base = d.cat.units[&id].clone();
        for level in [1, key_level - 1, key_level] {
            if level < 1 {
                continue;
            }
            let m = UnitState::at_level(&d, &id, level as i64, &id);
            let owned = progression::owned_passives(&d, &m);
            let keystone_open = level >= key_level;
            for p in &base.deck.passive {
                assert!(owned.contains(p), "{id} level {level}: owns its deck passive {p}");
            }
            for k in &base.deck.keystone {
                assert_eq!(owned.contains(k), keystone_open, "{id} level {level}: its deck keystone {k} only from level {key_level}");
            }
            assert_eq!(progression::keystone_level(&d, &m), key_level, "{id}: the keystone level is its class's");
            // the Skills tab offers what it owns and plays it in the automatic deck
            let t = DeckTerms::for_member(&d, &m, &w, &rs).expect("deck terms under the deck rule");
            for p in base.deck.passive.iter().filter(|p| d.cat.passives.get(*p).is_some_and(|x| x.slot == "passive")) {
                assert!(t.passives.contains(p) && t.auto.passive.contains(p), "{id} level {level}: {p} offered and auto-slotted");
            }
            assert_eq!(!t.auto.keystone.is_empty(), keystone_open && !t.keystones.is_empty(), "{id} level {level}: a keystone shows only from level {key_level}");
            // the battle unit applies the slotted ones under the deck rule, and classic builds stay as they were
            let u = progression::build_with(&d, &m, &w, true).expect("the member builds").unit;
            for p in t.auto.passive.iter().chain(&t.auto.keystone) {
                assert!(u.passives.contains(p), "{id} level {level}: the slotted {p} applies in battle");
            }
            let classic = progression::member_passives(&d, &m);
            for p in base.deck.passive.iter().filter(|p| !classic.contains(p)) {
                assert!(!progression::build_as(&d, &m, "", true, None).unwrap().unit.passives.contains(p), "{id}: a classic build does not take {p}");
            }
        }
    }
}

#[test]
fn a_slotted_other_keystone_keeps_the_slot() {
    let d = data();
    let w = deck_world(&d);
    let (id, key_level) = decked(&d).into_iter().find(|(id, _)| !d.cat.units[id].deck.keystone.is_empty()).expect("a unit with a deck keystone");
    let mine = d.cat.units[&id].deck.keystone[0].clone();
    // another keystone the member owns (a scratch grant of a shipped keystone row other than its own)
    let other = d.cat.passives.values().filter(|p| p.slot == "keystone" && p.id != mine).map(|p| p.id.clone()).min().expect("a second keystone row");
    let mut m = UnitState::at_level(&d, &id, key_level as i64, &id);
    m.passives.push(other.clone());
    m.deck.keystone = vec![other.clone()];
    assert!(m.validate(&d).is_empty(), "the deck naming the other keystone is valid: {:?}", m.validate(&d));
    let u = progression::build_with(&d, &m, &w, true).unwrap().unit;
    assert!(u.passives.contains(&other) && !u.passives.contains(&mine), "one keystone slot: the slotted one applies, not {mine}");
    // and naming its own keystone explicitly applies it
    m.deck.keystone = vec![mine.clone()];
    assert!(m.validate(&d).is_empty(), "its own keystone is owned at level {key_level}");
    assert!(progression::build_with(&d, &m, &w, true).unwrap().unit.passives.contains(&mine));
}

#[test]
fn every_hero_past_the_keystone_level_on_the_playtest_save_shows_a_keystone() {
    let d = data();
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/saves/after_vats/slot_0.json");
    let gs = save::decode(&d, &std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))).expect("the save loads").state;
    let rs = d.ruleset(&gs.world_id);
    let mut checked = 0;
    for m in gs.party.members.iter().filter(|m| m.level(&d) as i32 >= progression::keystone_level(&d, m)) {
        let t = DeckTerms::for_member(&d, m, &gs.world_id, &rs).expect("deck terms");
        let shown = t.shown(&m.deck);
        assert!(!shown.keystone.is_empty(), "{} (level {}): the keystone slot is filled", m.id, m.level(&d));
        let u = progression::build_with(&d, m, &gs.world_id, true).unwrap().unit;
        assert!(shown.keystone.iter().all(|k| u.passives.contains(k)), "{}: the shown keystone applies", m.id);
        checked += 1;
    }
    assert!(checked > 0, "the save has heroes past the keystone level");
}
