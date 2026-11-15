//! GEAR-REWORK1 (PR5-51b, `docs/sessions/U3_GEAR_REWORK.md` 2-3): every shipped loadout a strict class wears is
//! legal under the strict gear rule: no hard equip lock (`gear::wear_lock`: proficiency 0 in a non-accessory's gear type,
//! or a raw `required_stats` entry above the wearer's unscaled stat plus the proficiency discount) and the hand rules
//! (`gear::item_fits_slot` for the slot, `gear::off_hand_ok` for the off hand beside the main hand). Covered: party
//! heroes and start parties as a new game makes them, world characters (their drill passives included), class default
//! loadouts, the Dojo presets on a Dojo hero, and every catalog unit of a strict class. Expectations come from the rules,
//! not from shipped ids or numbers.
//!
//! Catalog units carry no passives, so a catalog row a world character fights from may rely on that character's drill
//! for a piece it wears; built bare such a row shows the lock. Those rows are listed (not failed) when the world
//! characters built on them wear the same pieces legally (asserted).
//!
//! FB-267 (planner decision): heroes, start parties and class defaults are hard asserts; the enemy / NPC side (world
//! characters, catalog units) and the Dojo presets no Dojo hero can wear keep pinned lists, so a new violation fails
//! and a fixed one must leave its list.

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
/// Returns (violations as (character "world:id", line), catalog rows the characters stand on with the pieces they wear
/// legally, characters seen).
fn world_characters(d: &Data) -> (Vec<(String, String)>, Vec<(String, String)>, usize) {
    let mut out = Vec::new();
    let mut legal_rows = Vec::new();
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
            if bad.is_empty() {
                legal_rows.push((json::s(c, "base_unit"), format!("{w}:{r}")));
            }
            out.extend(bad.into_iter().map(|b| (format!("{w}:{r}"), b)));
            seen += 1;
        }
    }
    (out, legal_rows, seen)
}

/// The violation lines of `found` ((key, line)) as a sorted set, asserted equal to the pinned list `pinned` (full lines:
/// who, piece, slot and reason, so a second violation on an already pinned unit or preset fails too); the message
/// lists every line found so a new violation (or one fixed, which must leave the pin) is named.
fn assert_pinned(what: &str, found: &[(String, String)], pinned: &[&str]) {
    let lines: BTreeSet<&str> = found.iter().map(|(_, l)| l.as_str()).collect();
    let want: BTreeSet<&str> = pinned.iter().copied().collect();
    let all: Vec<&str> = lines.iter().copied().collect();
    assert_eq!(lines, want, "{what}: the violations differ from the pinned list (a new one fails; a fixed one leaves the pin):\n{}", all.join("\n"));
}

/// FB-267 (PR5-51b planner decision): the row's hard requirement is the starter / founder loadouts (heroes, start
/// parties, class defaults: hard asserts below). Enemy and NPC world characters keep the soft-penalty fallback in battle
/// (the build does not refuse their gear), so their known violations are pinned here for Design: warriors wearing a
/// DEX 10 `requirement` piece (buckler, medium body) at DEX 9 with the proficiency discount. Any new violation fails;
/// fixing one forces the list to shrink.
const WORLD_CHARACTERS_PINNED: &[&str] = &[
    r#"character ashlands:brask_toller (warrior): shd_buckler in off_hand: Unfit { stat: "DEX", need: 10, have: 9 }"#,
    r#"character ashlands:hest_the_quartermaster (warrior): shd_buckler in off_hand: Unfit { stat: "DEX", need: 10, have: 9 }"#,
    r#"character ashlands:mara_voss (warrior): arm_leather in body: Unfit { stat: "DEX", need: 10, have: 9 }"#,
    r#"character iron_reaches:orsa_tallow (warrior): shd_buckler in off_hand: Unfit { stat: "DEX", need: 10, have: 9 }"#,
    r#"character tidewrack:ilse_marrow (warrior): arm_leather in body: Unfit { stat: "DEX", need: 10, have: 9 }"#,
];

#[test]
fn world_character_gear_is_legal_but_for_the_pinned_list() {
    let d = data();
    let (bad, _, seen) = world_characters(&d);
    assert!(seen > 0, "the shipped worlds have characters of a strict class");
    assert_pinned("world character gear", &bad, WORLD_CHARACTERS_PINNED);
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

/// FB-267: presets no Dojo hero (warrior, ranger) can wear whole: they target a class the Dojo has no hero for, or
/// stress a stat deficit that the strict rule now refuses; their pieces are skipped under the strict rule (a design
/// question for Design). Pinned: a new one fails, a fixed one must leave the list.
const PRESETS_PINNED: &[&str] = &[
    r#"preset arcanist on Ranger: wpn_tome in off_hand: Untrained { gear: "tome" }"#,
    r#"preset arcanist on Ranger: wpn_wand in main_hand: Untrained { gear: "sprayer" }"#,
    r#"preset arcanist on Warrior: wpn_tome in off_hand: Untrained { gear: "tome" }"#,
    r#"preset arcanist on Warrior: wpn_wand in main_hand: Untrained { gear: "sprayer" }"#,
    r#"preset dual_wield on Ranger: wpn_dagger in main_hand: Untrained { gear: "dagger" }"#,
    r#"preset dual_wield on Ranger: wpn_stiletto in off_hand: Untrained { gear: "dagger" }"#,
    r#"preset dual_wield on Warrior: arm_leather in body: Unfit { stat: "DEX", need: 10, have: 9 }"#,
    r#"preset dual_wield on Warrior: wpn_dagger in main_hand: Untrained { gear: "dagger" }"#,
    r#"preset dual_wield on Warrior: wpn_stiletto in off_hand: Untrained { gear: "dagger" }"#,
    r#"preset duelist on Ranger: wpn_dagger in off_hand: Untrained { gear: "dagger" }"#,
    r#"preset duelist on Warrior: arm_leather in body: Unfit { stat: "DEX", need: 10, have: 9 }"#,
    r#"preset duelist on Warrior: wpn_dagger in off_hand: Untrained { gear: "dagger" }"#,
    r#"preset titan_deficit on Ranger: wpn_warhammer in main_hand: Untrained { gear: "hammer" }"#,
    r#"preset titan_deficit on Warrior: wpn_warhammer in main_hand: Unfit { stat: "STR", need: 22, have: 19 }"#,
];

/// Each preset is worn whole by a Dojo hero (its hero: one that may wear every piece), but for the pinned list.
#[test]
fn every_dojo_preset_is_legal_on_its_hero_but_for_the_pinned_list() {
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
            bad.extend(tries.into_iter().flat_map(|(_, v)| v).map(|l| (p.id.clone(), l)));
        }
    }
    assert_pinned("Dojo presets no hero may wear whole", &bad, PRESETS_PINNED);
}

/// Catalog rows whose pieces only a world character's drill covers (catalog units carry no passives): proficiency 0
/// built bare, legal as the world character on the row (asserted). FB-267: pinned.
const DRILL_COVERED_PINNED: &[&str] = &[
    r#"unit calcified_husk (warrior): bone_club in main_hand: Untrained { gear: "censer" } (legal as sunk_ward:calcified_husk)"#,
    r#"unit guild_master_orlov (brawler): flensing_hooks in main_hand: Untrained { gear: "bone_saw" } (legal as sunk_ward:guild_master_orlov)"#,
    r#"unit guild_master_orlov (brawler): skinners_apron in body: Untrained { gear: "medium" } (legal as sunk_ward:guild_master_orlov)"#,
    r#"unit mortuary_flagellant (rogue): penitent_scourge in main_hand: Untrained { gear: "scourge" } (legal as sunk_ward:mortuary_flagellant)"#,
    r#"unit overseer_grendel (brawler): skinners_apron in body: Untrained { gear: "medium" } (legal as sunk_ward:overseer_grendel)"#,
    r#"unit proctor_quell (warrior): quells_bone_saws in main_hand: Untrained { gear: "bone_saw" } (legal as sunk_ward:proctor_quell)"#,
];

/// FB-267: catalog units with another violation (enemy / test rows keep the soft-penalty fallback): a warrior row
/// wearing a DEX 10 `requirement` medium body at DEX 9. Pinned.
const CATALOG_PINNED: &[&str] = &[r#"unit aberrant_test (warrior): arm_leather in body: Unfit { stat: "DEX", need: 10, have: 9 }"#];

/// Catalog units of a strict class built bare (no passives): legal, but for the drill-covered and the pinned rows.
#[test]
fn strict_catalog_units_are_legal_or_pinned() {
    let d = data();
    let (_, legal_rows, _) = world_characters(&d);
    let mut bad = Vec::new();
    let mut drill_covered = Vec::new();
    let mut seen = 0;
    for e in d.cat.units.values() {
        let Some(class) = d.cat.classes.get(&e.class_id).filter(|c| strict(c)) else { continue };
        let stats = stats_of(&d.cat, class, e.level, &e.spend, &[]);
        let v = violations(&format!("unit {} ({})", e.id, class.id), &row_gear(&d.cat, e), |i| wear_lock(&stats, i).map(|l| format!("{l:?}")));
        seen += 1;
        if v.is_empty() {
            continue;
        }
        // only proficiency locks can be a drill's; a world character standing on the row must wear it legally
        let chars: Vec<&String> = legal_rows.iter().filter(|(row, _)| *row == e.id).map(|(_, c)| c).collect();
        if !chars.is_empty() && v.iter().all(|x| x.contains("Untrained")) {
            let legal_as = chars.iter().map(|c| c.as_str()).collect::<Vec<_>>().join(", ");
            drill_covered.extend(v.into_iter().map(|l| (e.id.clone(), format!("{l} (legal as {legal_as})"))));
        } else {
            bad.extend(v.into_iter().map(|l| (e.id.clone(), l)));
        }
    }
    assert!(seen > 0, "the catalog has units of a strict class");
    assert_pinned("catalog rows covered only by the world character's drill", &drill_covered, DRILL_COVERED_PINNED);
    assert_pinned("catalog unit gear", &bad, CATALOG_PINNED);
}
