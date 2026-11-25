//! CLASSKEY1 (`PR6-14`, `docs/sessions/U2_CLASS.md` "Class keystones") on a scratch copy of the shipped data with the
//! row's seed (`data/seeds/pr6_14.json`) written through the content writer (a no-op once the seed is applied): each
//! class of the spec table names its keystone at level 6; a recruit of the class one level below has no keystone and at
//! the keystone level slots its class keystone (the world's deck rule); a Founder whose deck names a keystone keeps its
//! own at any level; Master Alchemist covers the three mage bombs on top of Elyse's three (+20% each); and the folder
//! check reports nothing new.

use aegis_core::json;
use emberwake_content::store;
use emberwake_world::state::UnitState;
use emberwake_world::{new_game, progression, Data};
use std::path::{Path, PathBuf};

/// The spec table: class -> its keystone passive.
const SPEC: [(&str, &str); 6] = [
    ("warrior", "unbroken_phalanx"),
    ("rogue", "ghost_of_the_undercroft"),
    ("priest", "pure_hippocratic_oath"),
    ("ranger", "dead_eye_discipline"),
    ("brawler", "unbroken_iron"),
    ("mage", "master_alchemist"),
];
/// The spec's keystone level ("learned at level 6").
const KEY_LEVEL: i32 = 6;
/// Master Alchemist's families: the three added, then Elyse's three kept, +0.2 potency each.
const ALCHEMIST: &str = "master_alchemist";
const ALCHEMIST_FAMILIES: [&str; 6] = ["firebomb", "glacial_burst", "choke_cloud", "phosphorus_pot", "magnesium_flash_bomb", "choke_gas"];
const ALCHEMIST_POTENCY: f64 = 0.2;

fn game_data() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let p = e.unwrap().path();
        let name = p.file_name().unwrap().to_owned();
        if p.is_dir() {
            copy_dir(&p, &to.join(name));
        } else {
            std::fs::copy(&p, to.join(name)).unwrap();
        }
    }
}

/// The scratch data with the row's seed written through the content writer.
fn seeded(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("emberwake_class_keystone_{tag}"));
    let dir = t.join("data");
    copy_dir(&game_data(), &dir);
    let seed: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(game_data().join("seeds/pr6_14.json")).expect("the PR6-14 seed")).unwrap();
    for step in seed["steps"].as_array().expect("seed steps") {
        assert_eq!(step["op"], "upsert", "the seed only upserts");
        let kind = step["kind"].as_str().unwrap();
        // PASSIVE-ICON1 (ENGINE-24): the later seed `passive_icon1.json` owns a passive's `icon`; the replay keeps it
        let mut entry = json::dict(&step["entry"]).clone();
        if kind == "passives" && !entry.contains_key("icon") {
            let shipped: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("catalog/passives.json")).unwrap()).unwrap();
            let id = json::s(&entry, "id");
            if let Some(row) = json::aget(json::dict(&shipped), "entries").iter().map(json::dict).find(|r| json::s(r, "id") == id) {
                entry.insert("icon".into(), row.get("icon").cloned().unwrap_or_default());
            }
        }
        if let Err(e) = store::upsert(&dir, None, kind, &entry, None) {
            panic!("seed step {kind}: {e:?}");
        }
    }
    (t, dir)
}

fn open(dir: &Path) -> Data {
    Data::open(dir).expect("data").0
}

/// The first world whose battles run the deck rule.
fn deck_world(d: &Data) -> (String, emberwake_rules::DeckRule) {
    d.world_ids().into_iter().find_map(|w| progression::deck_rule(d, &w).map(|r| (w, r))).expect("a world with the deck rule")
}

/// A recruit's base unit of `class`, by rule: the first derived party catalog unit of the class whose deck names no
/// keystone.
fn recruit_row(d: &Data, class: &str) -> String {
    d.cat.units.values().find(|u| u.side == "party" && u.class_id == class && u.stat_mode == "derived" && u.deck.keystone.is_empty()).map(|u| u.id.clone()).unwrap_or_else(|| panic!("a recruit row of {class}"))
}

#[test]
fn each_class_names_its_spec_keystone_at_level_six() {
    let (_t, dir) = seeded("table");
    let d = open(&dir);
    for (class, key) in SPEC {
        let k = d.cat.classes[class].keystone.clone().unwrap_or_else(|| panic!("{class} has a keystone"));
        assert_eq!((k.id.as_str(), k.level), (key, KEY_LEVEL), "{class}");
        assert_eq!(d.cat.passives[key].slot, "keystone", "{key}");
    }
}

#[test]
fn a_recruit_slots_its_class_keystone_from_the_keystone_level() {
    let (_t, dir) = seeded("recruits");
    let d = open(&dir);
    let (world, rule) = deck_world(&d);
    for (class, key) in SPEC {
        let row = recruit_row(&d, class);
        let at = |level: i32| {
            let s = UnitState::at_level(&d, &row, i64::from(level), &row);
            progression::build_as(&d, &s, &world, true, Some(&rule)).unwrap_or_else(|| panic!("{row} builds")).unit
        };
        let below = at(KEY_LEVEL - 1);
        assert!(below.deck.keystone.is_empty() && !below.passives.iter().any(|p| p == key), "{class} at {}: {:?}", KEY_LEVEL - 1, below.passives);
        let keyed = at(KEY_LEVEL);
        assert_eq!(keyed.deck.keystone, vec![key.to_string()], "{class} at {KEY_LEVEL}");
        assert!(keyed.passives.iter().any(|p| p == key), "{class} at {KEY_LEVEL}: {:?}", keyed.passives);
    }
}

#[test]
fn a_founder_keeps_its_own_keystone() {
    let (_t, dir) = seeded("founders");
    let d = open(&dir);
    let (world, rule) = deck_world(&d);
    let mut founders = 0;
    for w in d.world_ids() {
        for c in d.own_entries(&w, "characters") {
            let Some(mut h) = new_game::hero_from_character(&d, &w, &json::s(c, "id")) else { continue };
            let own = if h.deck.keystone.is_empty() { d.cat.units[&h.base_unit].deck.keystone.clone() } else { h.deck.keystone.clone() };
            if own.is_empty() {
                continue;
            }
            for level in [1, KEY_LEVEL, KEY_LEVEL + 1] {
                h.total_xp = progression::xp_for_level(&d, i64::from(level));
                let u = progression::build_as(&d, &h, &world, true, Some(&rule)).unwrap_or_else(|| panic!("{} builds", h.id)).unit;
                assert_eq!(u.deck.keystone, own, "{} at {level}", h.id);
            }
            founders += 1;
        }
    }
    assert!(founders > 0, "the shipped worlds have a Founder whose deck names a keystone");
}

#[test]
fn master_alchemist_covers_the_mage_bombs_and_elyses() {
    let (_t, dir) = seeded("alchemist");
    let d = open(&dir);
    let fx: Vec<(String, f64)> = d.cat.passives[ALCHEMIST]
        .effects
        .iter()
        .filter(|e| e["type"] == "skill_potency")
        .map(|e| (e["family"].as_str().unwrap_or_default().to_string(), e["value"].as_f64().unwrap_or(0.0)))
        .collect();
    for fam in ALCHEMIST_FAMILIES {
        assert!(d.cat.skills.iter().any(|s| s.id == fam), "{fam} is a skill");
        assert!(fx.iter().any(|(f, v)| f == fam && (v - ALCHEMIST_POTENCY).abs() < 1e-9), "{fam}: {fx:?}");
    }
    // a mage recruit at the keystone level holds the +20% on each of the six
    let (world, rule) = deck_world(&d);
    let row = recruit_row(&d, "mage");
    let build = |level: i32| progression::build_as(&d, &UnitState::at_level(&d, &row, i64::from(level), &row), &world, true, Some(&rule)).unwrap().unit;
    let (below, keyed) = (build(KEY_LEVEL - 1), build(KEY_LEVEL));
    for fam in ALCHEMIST_FAMILIES {
        assert!((keyed.potency_for(fam) - below.potency_for(fam) - ALCHEMIST_POTENCY).abs() < 1e-9, "{fam}");
    }
}

/// Both copies lie outside the game folder (no asset layers): the seed adds no problem to what the copy reports.
#[test]
fn the_seeded_folder_check_reports_nothing_new() {
    let t = aegis_core::scratch::Scratch::vacant("emberwake_class_keystone_base");
    let base = t.join("data");
    copy_dir(&game_data(), &base);
    let (_s, dir) = seeded("check");
    assert_eq!(store::check(&dir), store::check(&base));
}

/// PR7-16 (`FB-303`): a recruit whose base unit has an empty `deck.keystone` cannot see or save its class keystone
/// below the keystone level; at the keystone level `DeckTerms::for_member` lists it, slotting succeeds, and both
/// `validate()` and `deck_errors()` return empty.
#[test]
fn a_recruit_deck_editor_exposes_and_validates_the_class_keystone_only_from_its_level() {
    use emberwake_world::party_ops::deck::DeckTerms;
    let (_t, dir) = seeded("deck_editor");
    let d = open(&dir);
    let (world, _) = deck_world(&d);
    let rs = d.ruleset(&world);
    for (class, key) in SPEC {
        let row = recruit_row(&d, class);
        assert!(d.cat.units[&row].deck.keystone.is_empty(), "{row} has empty base.deck.keystone");

        // Below keystone level: omitted from DeckTerms::for_member, slot refused, saving rejected by validate() & deck_errors().
        let mut below = UnitState::at_level(&d, &row, i64::from(KEY_LEVEL - 1), &row);
        let below_terms = DeckTerms::for_member(&d, &below, &world, &rs).expect("deck terms");
        assert!(!below_terms.keystones.contains(&key.to_string()), "{class} below level {KEY_LEVEL} omits {key}");
        assert!(below_terms.slot(&below.deck, "keystone", key).is_err(), "{class} below level {KEY_LEVEL} cannot slot {key}");
        below.deck.keystone = vec![key.to_string()];
        assert!(!below.validate(&d).is_empty(), "{class} below level {KEY_LEVEL}: validate() rejects {key}");
        assert!(!below.deck_errors(&d, &row).is_empty(), "{class} below level {KEY_LEVEL}: deck_errors() rejects {key}");

        // At keystone level: listed by DeckTerms::for_member, auto-slotted, slotting succeeds, and validate()/deck_errors() are empty.
        let mut at_key = UnitState::at_level(&d, &row, i64::from(KEY_LEVEL), &row);
        let terms = DeckTerms::for_member(&d, &at_key, &world, &rs).expect("deck terms");
        assert!(terms.keystones.contains(&key.to_string()), "{class} at level {KEY_LEVEL} lists {key}");
        assert_eq!(terms.shown(&at_key.deck).keystone, vec![key.to_string()], "{class} at level {KEY_LEVEL} auto-slots {key}");

        // Verify explicit slotting into a free keystone slot via DeckTerms::slot succeeds (with another keystone already slotted in a 2-slot ruleset)
        // and replacing an alternate keystone via after_drop succeeds under the shipped 1-slot ruleset.
        let other = d.cat.passives.values().find(|p| p.slot == "keystone" && p.id != key).expect("another keystone").id.clone();
        let mut with_other = at_key.clone();
        with_other.passives.push(other.clone());
        with_other.deck.keystone = vec![other.clone()];
        let terms_other = DeckTerms::for_member(&d, &with_other, &world, &rs).expect("deck terms");
        let swapped = terms_other.after_drop(&with_other.deck, &format!("deck.own.keystone.{key}"), "deck.keystone.0").expect("replacing into keystone slot succeeds");
        assert_eq!(swapped.keystone, vec![key.to_string()]);
        let mut rs_two = rs.clone();
        rs_two.numbers.deck_slots.keystone = 2;
        let terms_two = DeckTerms::for_member(&d, &with_other, &world, &rs_two).expect("deck terms with 2 keystone slots");
        let slotted = terms_two.slot(&with_other.deck, "keystone", key).expect("slot() succeeds for owned class keystone");
        assert_eq!(slotted.keystone, vec![other, key.to_string()]);

        at_key.deck = swapped;
        assert!(at_key.validate(&d).is_empty(), "{class} at level {KEY_LEVEL}: validate() clean: {:?}", at_key.validate(&d));
        assert!(at_key.deck_errors(&d, &row).is_empty(), "{class} at level {KEY_LEVEL}: deck_errors() clean: {:?}", at_key.deck_errors(&d, &row));
    }
}
