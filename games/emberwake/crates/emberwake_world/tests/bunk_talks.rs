//! BUNKS1 (PR7-13) on the shipped data, read only: the `sunk_ward` bunk talks (seed `seeds/pr7_13.json`) and its hub.
//! The talks hold one written talk for each pair of the world's first four party characters (the starter founders)
//! and a template talk for every shipped `temper_*` temperament (the seed's fifth picks by an origin); they check
//! clean; a founder pair's talk waits once the two fought its `min_together` battles side by side and plays once; a
//! generated recruit's template talk pairs it with the first founder; the hub map has a bunk prop for each bunk.
//! Expectations come from the rows and the rules, not from counted content.

use aegis_core::json;
use emberwake_world::state::{PartyState, UnitState};
use emberwake_world::{bunks, explore, new_game, roster, talks, Data, GameState};
use emberwake_data::talk::{pair_key, TalkRow, TALKS_KIND};
use std::path::{Path, PathBuf};

const WORLD: &str = "sunk_ward";
/// BUNKS1 row PR7-13: written talks for this many founder pairs and this many template talks.
const FOUNDER_PAIRS: usize = 6;
const TEMPLATES: usize = 5;

fn dir() -> PathBuf {
    Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")
}

fn data() -> Data {
    Data::open(&dir()).expect("data").0
}

fn rows(d: &Data) -> Vec<TalkRow> {
    d.catalogs.merged_entries(WORLD, TALKS_KIND).into_iter().map(TalkRow::from_obj).collect()
}

/// The founders the written talks pair (every character a pair names, first-named order).
fn founders(d: &Data) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for name in rows(d).into_iter().filter(|r| !r.template).flat_map(|r| r.pair) {
        if !out.contains(&name) {
            out.push(name);
        }
    }
    out
}

/// A game whose party is characters `ids` (all active).
fn game(d: &Data, ids: &[&str]) -> GameState {
    let members: Vec<UnitState> = ids.iter().map(|c| new_game::hero_from_character(d, WORLD, c).expect("a party character")).collect();
    GameState::create(d, WORLD, PartyState { active_ids: members.iter().map(|m| m.id.clone()).collect(), members, ..PartyState::default() })
}

#[test]
fn the_talks_cover_the_founder_pairs_and_every_temperament() {
    let d = data();
    let rows = rows(&d);
    let founders = founders(&d);
    let mut pairs: Vec<(String, String)> = rows.iter().filter(|r| !r.template).map(|r| (r.pair[0].clone().min(r.pair[1].clone()), r.pair[0].clone().max(r.pair[1].clone()))).collect();
    pairs.sort();
    pairs.dedup();
    assert_eq!(pairs.len(), FOUNDER_PAIRS, "one written talk per founder pair: {pairs:?}");
    // the pairs are every pair of the founders they name (4 founders make 6 pairs), all party characters
    assert_eq!(founders.len() * (founders.len() - 1) / 2, FOUNDER_PAIRS, "every pair of {founders:?}");
    for f in &founders {
        assert_eq!(json::s(d.catalogs.resolve(WORLD, "characters", f), "role"), "party", "{f} is a party character");
    }
    let templates: Vec<&TalkRow> = rows.iter().filter(|r| r.template).collect();
    assert_eq!(templates.len(), TEMPLATES);
    let tempers: Vec<String> = d.cat.passives.keys().filter(|k| k.starts_with("temper_")).cloned().collect();
    assert!(!tempers.is_empty());
    for t in &tempers {
        assert_eq!(templates.iter().filter(|r| &r.temperament == t).count(), 1, "one template talk for {t}");
    }
    assert!(templates.iter().all(|r| !r.temperament.is_empty() || !r.origin.is_empty()));
}

#[test]
fn the_talks_and_their_dialogues_check_clean() {
    let d = data();
    let dialogues: Vec<String> = rows(&d).iter().map(|r| format!("{WORLD}/dialogues:{}", r.dialogue)).collect();
    let problems: Vec<String> = emberwake_content::store::check(&dir())
        .into_iter()
        .filter(|p| p.starts_with(&format!("{WORLD}/{TALKS_KIND}")) || dialogues.iter().any(|x| p.starts_with(x.as_str())))
        .collect();
    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn a_founder_pair_talk_waits_after_battles_together_and_plays_once() {
    let d = data();
    let rows = rows(&d);
    let row = rows.iter().find(|r| !r.template).expect("a written talk");
    let mut gs = game(&d, &[&row.pair[0], &row.pair[1]]);
    let (a, b) = (gs.party.members[0].id.clone(), gs.party.members[1].id.clone());
    for _ in 0..row.min_together {
        assert!(talks::waiting(&d, &gs).iter().all(|w| w.id != row.id), "waits on its counter");
        let ends = roster::default_ends(&d, &gs, WORLD, "won");
        roster::settle_party(&d, &mut gs, &roster::FightEnd::new("won", WORLD), &ends);
    }
    assert_eq!(talks::together(&gs, &a, &b), row.min_together);
    let w = talks::waiting_for(&d, &gs, &a).expect("the pair's talk waits");
    assert_eq!(w.id, row.id);
    let (r, _) = talks::play(&d, &mut gs, &w);
    assert!(!r.closed, "{:?}", r.errors);
    assert!(!json::s(&r.view(&d, &gs), "text").is_empty());
    assert!(gs.flag_true(&w.seen));
    assert!(talks::waiting(&d, &gs).iter().all(|x| x.id != row.id), "played once");
}

#[test]
fn a_generated_recruit_gets_its_temperament_talk_with_the_first_founder() {
    let d = data();
    let rows = rows(&d);
    let row = rows.iter().find(|r| r.template && !r.temperament.is_empty()).expect("a temperament template");
    let founder = founders(&d)[0].clone();
    let mut gs = game(&d, &[&founder]);
    let mut m = UnitState::at_level(&d, &gs.party.members[0].base_unit.clone(), 1, "recruit_1");
    m.name = "Recruit".into();
    m.template = format!("{WORLD}:tpl");
    m.temperament = row.temperament.clone();
    gs.party.members.push(m);
    gs.party.together.insert(pair_key(&founder, "recruit_1"), row.min_together);
    let w = talks::waiting_for(&d, &gs, "recruit_1").expect("the template talk waits");
    assert_eq!((w.id.as_str(), w.a.as_str()), (row.id.as_str(), founder.as_str()));
    let (r, _) = talks::play(&d, &mut gs, &w);
    let text = json::s(&r.view(&d, &gs), "text");
    assert!(!r.closed && !text.contains("{a}") && !text.contains("{b}"), "{text}");
}

#[test]
fn the_hub_map_has_a_bunk_prop_for_each_bunk() {
    let d = data();
    let first = d.catalogs.own_entries(WORLD, "characters").iter().find(|c| json::s(c, "role") == "party").map(|c| json::s(c, "id")).expect("a party character");
    let gs = game(&d, &[&first]);
    let hub = json::s(json::dget(d.manifest(WORLD), "defeat"), "to");
    let loc = explore::resolve_location(&d, &gs, &hub);
    let t = explore::template_of(&d, loc);
    assert!(bunks::is_hub(&gs, t), "the retreat hub {hub} is a hub");
    assert_eq!(bunks::bunk_props(t).len() as i64, roster::bunk_cap(&gs), "as many bunk props as the bunk cap");
    assert!(!bunks::table_spots(loc, t).is_empty(), "the hub has table spots");
}
