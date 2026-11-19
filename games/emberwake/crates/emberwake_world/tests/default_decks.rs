//! PR7-41 (PT-22): the default deck of every party unit a Sunk Ward character fights as (its `base_unit`) fills the
//! active slots of the world's ruleset from the skill families it owns (its class path at its level and its extra
//! skills, basics aside): a slot is left empty only when every owned family is already slotted. Read from the shipped
//! data; nothing is written.

use aegis_core::json;
use emberwake_world::Data;
use std::path::Path;

#[test]
fn party_default_decks_fill_every_active_slot() {
    let d = Data::open(&Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")).unwrap().0;
    let rules = d.ruleset("sunk_ward");
    let slots = usize::try_from(rules.numbers.deck_slots.active).unwrap_or(0);
    assert!(slots > 0, "the fixture: the world's ruleset has active deck slots");
    let mut checked = 0;
    let bases: Vec<String> = d.merged_entries("sunk_ward", "characters").into_iter().map(|c| json::s(c, "base_unit")).filter(|b| !b.is_empty()).collect();
    for u in d.cat.units.values().filter(|u| u.side == "party" && !u.deck.active.is_empty() && bases.contains(&u.id)) {
        let mut owned: Vec<String> = Vec::new();
        let path = d.cat.classes.get(&u.class_id).map(|c| c.path_steps()).unwrap_or_default();
        for id in path.into_iter().filter(|(l, _)| *l <= u.level).map(|(_, s)| s).chain(u.extra_skills.iter().cloned()) {
            let fam = d.cat.family_of(&id).to_string();
            let basic = d.cat.is_basic_family(&fam) || rules.numbers.basics.contains(&fam) || d.cat.skill(&id).is_some_and(|s| s.basic || s.system);
            if !basic && !owned.contains(&fam) {
                owned.push(fam);
            }
        }
        let want = slots.min(owned.len());
        assert!(
            u.deck.active.len() >= want,
            "{}: {} of {slots} active slots filled though it owns {} families ({:?}; active {:?})",
            u.id,
            u.deck.active.len(),
            owned.len(),
            owned,
            u.deck.active
        );
        checked += 1;
    }
    assert!(checked > 0, "some party unit has a default deck");
}
