//! GEAR-REWORK1 (PR5-51b, `docs/sessions/U3_GEAR_REWORK.md` 2-3): every shipped loadout a strict class wears is
//! legal under the strict gear rule: no hard equip lock (`gear::wear_lock`: proficiency 0 in a non-accessory's gear type,
//! or a raw `required_stats` entry above the wearer's unscaled stat plus the proficiency discount) and the hand rules
//! (`gear::item_fits_slot` for the slot, `gear::off_hand_ok` for the off hand beside the main hand). Covered: party
//! heroes and start parties as a new game makes them, world characters (their drill passives included), class default
//! loadouts, the Dojo presets on a Dojo hero, and every catalog unit of a strict class. Expectations come from the rules,
//! not from shipped ids or numbers.
//!
//! PR5-57 (GEAR-REWORK1 follow-up): all pinned lists are retired. World characters, catalog units (with their own
//! unit-level drill `passives`), and all shipped Dojo presets on the Dojo heroes pass with zero violations.

use aegis_core::json;
use emberwake_battle::dojo::DojoSession;
use emberwake_battle::session::Roster;
use emberwake_data::{Catalog, ClassDef, ItemDef, UnitDef, LOADOUT_SLOTS, NO_ITEM};
use emberwake_rules::gear::{item_fits_slot, off_hand_ok, wear_lock};
use emberwake_rules::stats::{CharacterStats, Proficiency};
use emberwake_world::battle::WorldRoster;
use emberwake_world::{new_game, Data};
use std::collections::BTreeSet;
use std::path::Path;

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("data").0
}

/// The strict rule's stats of a `class` wearer at `level` with `spend` and its always-on `passives` (drills).
fn stats_of(cat: &Catalog, class: &ClassDef, level: i32, spend: &indexmap::IndexMap<String, i32>, passives: &[String]) -> CharacterStats {
    let mut s = CharacterStats::new(class, level.max(1), spend);
    s.proficiency = Proficiency::from_class(class).with_drills(cat, passives);
    s
}

/// The violations of loadout `gear` ((slot key, item)) on a wearer, each "who: item in slot: why". `lock` reads the
/// hard lock of one item.
fn violations(who: &str, gear: &[(String, ItemDef)], lock: impl Fn(&ItemDef) -> Option<String>) -> Vec<String> {
    let mut out = Vec::new();
    let main = gear.iter().find(|(s, _)| s == "main_hand").map(|(_, i)| i);
    for (slot, item) in gear {
        if !item_fits_slot(item, slot) {
            out.push(format!("{who}: {} in {slot}: does not fit the slot (slot_rule '{}')", item.id, item.slot_rule));
        }
        if slot == "off_hand" && !off_hand_ok(main, item) {
            out.push(format!("{who}: {} in off_hand: not allowed beside two-handed {}", item.id, main.map(|m| m.id.as_str()).unwrap_or("")));
        }
        if let Some(why) = lock(item) {
            out.push(format!("{who}: {} in {slot}: {why}", item.id));
        }
    }
    out
}

/// The loadout catalog row `e` is built with (its own slot items or its class's defaults), as (slot, item). A class
/// default off-hand piece the main hand does not allow (`off_hand_ok`: a default shield under a two-handed weapon) is
/// not held, as the build drops it; an off-hand piece the row names itself stays (and is checked).
fn row_gear(cat: &Catalog, e: &UnitDef) -> Vec<(String, ItemDef)> {
    let mut gear: Vec<(String, ItemDef)> = LOADOUT_SLOTS
        .iter()
        .filter_map(|slot| emberwake_rules::unit::row_slot_item(cat, e, slot).and_then(|id| cat.items.get(&id)).filter(|i| i.is_equipment()).map(|i| (slot.to_string(), i.clone())))
        .collect();
    let own_off = e.slots.get("off_hand").is_some_and(|v| !v.is_empty());
    drop_default_off_hand(&mut gear, own_off);
    gear
}

/// Drops a defaulted (not the wearer's own) off-hand piece the main hand does not allow, as the build does.
fn drop_default_off_hand(gear: &mut Vec<(String, ItemDef)>, own_off: bool) {
    let main = gear.iter().find(|(s, _)| s == "main_hand").map(|(_, i)| i.clone());
    if !own_off {
        gear.retain(|(s, i)| s != "off_hand" || off_hand_ok(main.as_ref(), i));
    }
}

fn strict(class: &ClassDef) -> bool {
    Proficiency::from_class(class).is_strict()
}

/// Every world character's gear: its row (raised to the world's stage) with its named items on top (the named item's
/// base item: names and tuning do not touch the gear rules), worn with its class and its drill passives.
/// Returns (violations as (character "world:id", line), characters seen).
fn world_characters(d: &Data) -> (Vec<(String, String)>, usize) {
    let mut out = Vec::new();
    let mut seen = 0;
    for w in d.world_ids() {
        let cat = d.cat_in(&w);
        let roster = WorldRoster::new(d, w.clone());
        for c in d.own_entries(&w, "characters") {
            let r = json::s(c, "id");
            let Some(e) = roster.entry(cat, &r) else { continue };
            let Some(class) = cat.classes.get(&e.class_id).filter(|c| strict(c)) else { continue };
            let mut gear = row_gear(cat, &e);
            for n in json::aget(c, "named_items") {
                let entry = d.resolve(&w, "items", &json::gstr(n));
                let Some(item) = d.cat.items.get(&json::s(entry, "base_item")).filter(|i| i.is_equipment()) else { continue };
                let slot = item.slot.to_lowercase();
                gear.retain(|(s, _)| *s != slot);
                gear.push((slot, item.clone()));
            }
            let stats = stats_of(cat, class, e.level, &e.spend, &d.character_passives(c, &w));
            let bad = violations(&format!("character {w}:{r} ({})", class.id), &gear, |i| wear_lock(&stats, i).map(|l| format!("{l:?}")));
            out.extend(bad.into_iter().map(|b| (format!("{w}:{r}"), b)));
            seen += 1;
        }
    }
    (out, seen)
}

#[test]
fn world_character_gear_is_legal() {
    let d = data();
    let (bad, seen) = world_characters(&d);
    assert!(seen > 0, "the shipped worlds have characters of a strict class");
    let lines: Vec<&str> = bad.iter().map(|(_, l)| l.as_str()).collect();
    assert!(lines.is_empty(), "illegal world character gear:\n{}", lines.join("\n"));
}

#[test]
fn every_party_hero_and_start_party_wears_its_gear_legally() {
    let d = data();
    let mut bad = Vec::new();
    let mut seen = 0;
    for w in d.world_ids() {
        let mut ids: BTreeSet<String> = d.own_entries(&w, "characters").iter().map(|c| json::s(c, "id")).collect();
        let start = d.start_party(&w);
        ids.extend(start.iter().cloned());
        for id in &ids {
            // a start-party catalog unit joins as itself (`new_game::create`)
            let hero = new_game::hero_from_character(&d, &w, id).or_else(|| d.cat.units.get(id).filter(|_| start.contains(id)).map(|u| emberwake_world::state::UnitState::at_level(&d, id, i64::from(u.level), id)));
            let Some(h) = hero else {
                assert!(!start.contains(id), "start party {w}:{id} joins");
                continue;
            };
            let cat = d.cat_in(&w);
            let Some(class) = cat.classes.get(&h.class_id(&d)) else { continue };
            if !strict(class) {
                continue;
            }
            let gear: Vec<(String, ItemDef)> = LOADOUT_SLOTS
                .iter()
                .filter_map(|slot| {
                    let id = match h.equipment.get(*slot) {
                        Some(i) if !i.is_empty() => Some(i.clone()),
                        _ => h.default_item(&d, &w, slot),
                    }?;
                    (id != NO_ITEM).then(|| cat.items.get(&id).filter(|i| i.is_equipment()).map(|i| (slot.to_string(), i.clone()))).flatten()
                })
                .collect();
            let own_off = h.equipment.get("off_hand").is_some_and(|i| !i.is_empty());
            let mut gear = gear;
            drop_default_off_hand(&mut gear, own_off);
            bad.extend(violations(&format!("hero {w}:{id} ({})", class.id), &gear, |i| h.wear_lock(&d, i).map(|l| format!("{l:?}"))));
            seen += 1;
        }
    }
    assert!(seen > 0, "the shipped worlds have party heroes of a strict class");
    assert!(bad.is_empty(), "illegal hero / start party gear:\n{}", bad.join("\n"));
}

#[test]
fn every_class_default_loadout_is_legal_at_level_one() {
    let d = data();
    let mut bad = Vec::new();
    let mut seen = 0;
    for class in d.cat.classes.values().filter(|c| strict(c)) {
        let gear: Vec<(String, ItemDef)> = class
            .loadout
            .iter()
            .filter(|(_, id)| !id.is_empty() && id.as_str() != NO_ITEM)
            .filter_map(|(slot, id)| d.cat.items.get(id).filter(|i| i.is_equipment()).map(|i| (slot.to_lowercase(), i.clone())))
            .collect();
        let stats = stats_of(&d.cat, class, 1, &indexmap::IndexMap::new(), &[]);
        bad.extend(violations(&format!("class {} default", class.id), &gear, |i| wear_lock(&stats, i).map(|l| format!("{l:?}"))));
        seen += 1;
    }
    assert!(seen > 0, "the catalog has a strict class");
    assert!(bad.is_empty(), "illegal class default loadouts:\n{}", bad.join("\n"));
}

/// Each preset is worn whole by a Dojo hero (one that may wear every piece).
#[test]
fn every_dojo_preset_is_legal_on_its_hero() {
    let d = data();
    let s = DojoSession::new(d.cat.clone());
    assert!(!s.presets().is_empty(), "the game ships its presets");
    let mut bad = Vec::new();
    for p in s.presets() {
        let gear: Vec<(String, ItemDef)> = p.gear.iter().filter_map(|(slot, id)| s.cat.items.get(id).map(|i| (slot.clone(), i.clone()))).collect();
        assert_eq!(gear.len(), p.gear.len(), "preset {} names catalog items", p.id);
        let tries: Vec<(String, Vec<String>)> = s
            .heroes
            .iter()
            .map(|h| {
                assert!(h.stats.proficiency.is_strict(), "Dojo hero {} is of a strict class", h.name);
                (h.name.clone(), violations(&format!("preset {} on {}", p.id, h.name), &gear, |i| wear_lock(&h.stats, i).map(|l| format!("{l:?}"))))
            })
            .collect();
        if tries.iter().all(|(_, v)| !v.is_empty()) {
            bad.extend(tries.into_iter().flat_map(|(_, v)| v));
        }
    }
    assert!(bad.is_empty(), "Dojo presets no hero may wear whole:\n{}", bad.join("\n"));
}

/// Catalog units of a strict class built with their own passives (unit-level drills) are legal.
#[test]
fn strict_catalog_units_are_legal() {
    let d = data();
    let mut bad = Vec::new();
    let mut seen = 0;
    for e in d.cat.units.values() {
        let Some(class) = d.cat.classes.get(&e.class_id).filter(|c| strict(c)) else { continue };
        let stats = stats_of(&d.cat, class, e.level, &e.spend, &e.passives);
        let v = violations(&format!("unit {} ({})", e.id, class.id), &row_gear(&d.cat, e), |i| wear_lock(&stats, i).map(|l| format!("{l:?}")));
        seen += 1;
        bad.extend(v);
    }
    assert!(seen > 0, "the catalog has units of a strict class");
    assert!(bad.is_empty(), "illegal catalog unit gear:\n{}", bad.join("\n"));
}
