//! U2.2-S2 / U4.2-S2 on the shipped data (read only): every hero, world character, catalog unit and fixed unit built
//! through the one builder (`emberwake_rules::build_from`) is the unit the builds before it made, field for field (the
//! old hero and NPC paths are kept here as the oracle); a fixed unit's explicit class gives the stats the classless
//! fallback gave; a member's re-gathered passives are its new-game list; and one fight fields a hero, a world
//! character, a catalog unit and a fixed unit, each built by the same path.

use aegis_core::json::{self, Dict};
use emberwake_battle::session::{EncounterDef, Phase, Prebuilt, Roster, Session};
use emberwake_battle::world::AiProfile;
use emberwake_data::{Catalog, UnitDef, STATS};
use emberwake_rules::gear::{off_hand_ok, Equipment, Slot};
use emberwake_rules::{build_from, build_unit, build_unit_in, DeckRule, Unit, UnitBuild};
use emberwake_world::battle::{named_item, WorldRoster};
use emberwake_world::state::UnitState;
use emberwake_world::{new_game, progression, Data};
use std::path::Path;

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("data").0
}

fn same(a: &Unit, b: &Unit) -> bool {
    format!("{a:?}") == format!("{b:?}")
}

/// U4.2-S3 (FB-29): a save leaves its character's temperament out (`new_game::hero_from_character`) and every build
/// reads it from the character, so the saved passive snapshot the old path applied is the snapshot with the
/// temperament back in its gathered place (`Data::character_passives` order; the save's other ids after, in order).
fn saved_with_temperament(d: &Data, state: &UnitState) -> Vec<String> {
    let character = progression::character_of(d, state);
    let temperament = json::s(character, "temperament");
    let world = aegis_content::split_qid(&state.lore_ref).map(|(w, _)| w).unwrap_or_default();
    let mut out: Vec<String> = d.character_passives(character, &world).into_iter().filter(|p| state.passives.contains(p) || (!temperament.is_empty() && *p == temperament)).collect();
    out.extend(state.passives.iter().filter(|p| !out.contains(p)).cloned().collect::<Vec<_>>());
    out
}

/// The member build before U2.2-S2 (`progression::build_as`): the row alone, then the saved passive snapshot, skills,
/// gear, pouch, vitals, loadout, deck and ranks.
fn old_member(d: &Data, state: &UnitState, world: &str, deck: Option<&DeckRule>) -> Option<Unit> {
    if !state.validate(d).is_empty() {
        return None;
    }
    let cat = d.cat_in(world);
    let mut work = state.clone();
    progression::auto_for(d, &mut work);
    let mut def = cat.units.get(&work.base_unit)?.clone();
    def.level = work.level(d) as i32;
    def.spend = work.spend.iter().map(|(k, v)| (k.clone(), *v as i32)).collect();
    if def.spend.is_empty() {
        def.spend.insert(STATS[0].to_string(), 0);
    }
    def.name = progression::display_name(d, &work);
    let mut u = build_unit(cat, &def, 0);
    if let Some(rule) = deck {
        if !work.deck.is_empty() {
            u.deck = work.deck.clone();
        }
        u.deck_rule = Some(rule.clone());
    }
    for id in &saved_with_temperament(d, &work) {
        if let Some(p) = cat.passives.get(id) {
            u.apply_passive(cat, p);
        }
    }
    for sk in &work.learned_skills {
        u.learn_skill_checked(cat, sk, false);
    }
    for (slot_name, item_id) in &work.equipment {
        let (Some(slot), Some(item)) = (Slot::parse(slot_name), cat.items.get(item_id)) else { continue };
        if item.is_equipment() {
            u.inventory.unequip(slot);
            u.inventory.equip(Equipment::new(item), slot);
        }
    }
    if let (Some(main), Some(off)) = (u.inventory.get(Slot::MainHand), u.inventory.get(Slot::OffHand)) {
        if !off_hand_ok(Some(&main.item), &off.item) {
            u.inventory.unequip(Slot::OffHand);
        }
    }
    u.inventory.pouch.clear();
    // A1.2-K3 (INTENT2-K3): the battle pouch takes stacks under the world's `pouch_stacks` (the one intended difference)
    u.inventory.pouch_stacks = d.ruleset(world).numbers.pouch_stacks;
    for id in &work.pouch {
        if let Some(item) = cat.items.get(id).filter(|i| i.item_type == "CONSUMABLE") {
            u.inventory.add_to_pouch(item);
        }
    }
    u.sync_from_stats_and_gear(cat);
    u.hp = u.max_hp;
    if !work.loadout_class.is_empty() || !work.loadout_extra.is_empty() {
        let _ = u.set_loadout(cat, work.loadout_class.clone(), work.loadout_extra.clone());
    }
    u.apply_deck(cat);
    u.rank_choice = Some(work.loadout_ranks.iter().map(|(k, v)| (k.clone(), *v as i32)).collect());
    Some(u)
}

/// The world character build before U2.2-S2: the session's row build, `WorldRoster::customize` (the world's class
/// path) and `apply_character` (named items, passives, vitals when geared, tuning).
fn old_npc(d: &Data, cat: &Catalog, world: &str, character: &Dict, e: &UnitDef, ap: i32, deck: Option<&DeckRule>) -> Unit {
    let mut u = build_unit_in(cat, e, ap, deck);
    if json::get(d.class_overlay(world, &e.class_id), "skill_path").is_object() {
        let (max_ap, cur_ap) = (u.max_ap, u.ap);
        let rule = u.deck_rule.clone();
        u = build_unit_in(d.cat_in(world), e, 0, rule.as_ref());
        (u.max_ap, u.ap) = (max_ap, cur_ap);
    }
    let mut geared = false;
    for r in json::aget(character, "named_items") {
        let entry = d.resolve(world, "items", &json::gstr(r));
        if entry.is_empty() {
            continue;
        }
        if let Some(item) = named_item(d, entry).filter(|i| i.is_equipment()) {
            u.inventory.equip(Equipment::new(&item), Slot::parse(&item.slot).unwrap_or(Slot::MainHand));
            geared = true;
        }
    }
    for id in d.character_passives(character, world) {
        if let Some(p) = cat.passives.get(&id) {
            u.apply_passive(cat, p);
        }
    }
    if geared && json::s_or(d.entry("units", &json::s(character, "base_unit")), "stat_mode", "fixed") == "derived" {
        u.sync_from_stats_and_gear(cat);
        u.hp = u.max_hp;
    }
    let tuning = json::dget(character, "tuning");
    let key = |k: &str| json::i(tuning, k, 0) as i32;
    if key("hp_pct") != 0 {
        u.max_hp = (json::roundi(u.max_hp as f64 * (1.0 + key("hp_pct") as f64 / 100.0)) as i32).max(1);
        u.hp = u.max_hp;
    }
    if key("power") != 0 {
        u.power_override = (emberwake_rules::damage::power_of(&u, u.attack_range > 1, false) + key("power")).max(1);
    }
    if key("armor") != 0 {
        u.armor_override = (emberwake_rules::damage::armor_rating(&u) + key("armor")).max(0);
    }
    if key("speed") != 0 {
        u.initiative = (u.initiative + key("speed")).max(1);
    }
    if key("ap") != 0 {
        u.max_ap = (u.max_ap + key("ap")).max(1);
        u.ap = u.max_ap;
    }
    u
}

/// Every party hero of every world (the characters that may join), as a new game makes it.
fn heroes(d: &Data) -> Vec<(String, UnitState)> {
    let mut out = Vec::new();
    for w in d.world_ids() {
        for c in d.own_entries(&w, "characters") {
            if let Some(h) = new_game::hero_from_character(d, &w, &json::s(c, "id")) {
                out.push((w.clone(), h));
            }
        }
    }
    out
}

#[test]
fn every_hero_and_world_character_builds_as_before() {
    let d = data();
    let heroes = heroes(&d);
    assert!(!heroes.is_empty(), "the shipped worlds have party heroes");
    let catalog_members = d.cat.units.values().filter(|u| u.side == "party").map(|u| ("".to_string(), UnitState::at_level(&d, &u.id, u.level as i64, &u.id)));
    for (w, h) in heroes.iter().cloned().chain(catalog_members) {
        let rule = progression::deck_rule(&d, &w);
        for (world, deck) in [("", None), (w.as_str(), rule.as_ref())] {
            let new = progression::build_as(&d, &h, world, true, deck).map(|b| b.unit);
            let old = old_member(&d, &h, world, deck);
            assert_eq!(new.is_some(), old.is_some(), "member {} in '{world}'", h.id);
            if let (Some(n), Some(o)) = (new, old) {
                assert!(same(&n, &o), "member {} in '{world}' (deck rule {}) differs", h.id, deck.is_some());
            }
        }
    }
    let mut npcs = 0;
    for w in d.world_ids() {
        let roster = WorldRoster::new(&d, w.clone());
        let rule = progression::deck_rule(&d, &w);
        for c in d.own_entries(&w, "characters") {
            let r = json::s(c, "id");
            let Some(e) = roster.entry(&d.cat, &r) else { continue };
            for (ap, deck) in [(0, None), (12, rule.as_ref())] {
                let new = build_from(&d.cat, UnitBuild { row: &e, default_ap: ap, deck, wield: d.ruleset(&w).numbers.wield(), over: roster.overrides(&d.cat, &r), growth: None }).unit;
                let old = old_npc(&d, &d.cat, &w, d.resolve(&w, "characters", &r), &e, ap, deck);
                assert!(same(&new, &old), "character {w}:{r} (deck rule {}) differs", deck.is_some());
            }
            npcs += 1;
        }
    }
    assert!(npcs > 0, "the shipped worlds have characters");
}

#[test]
fn a_fixed_unit_with_its_class_has_the_stats_the_classless_fallback_gave() {
    let d = data();
    let fixed: Vec<&UnitDef> = d.cat.units.values().filter(|u| u.stat_mode == "fixed").collect();
    assert!(!fixed.is_empty(), "the shipped catalog has a fixed unit");
    for e in fixed {
        assert!(d.cat.classes.contains_key(&e.class_id), "fixed unit {} names a class", e.id);
        let mut classless = e.clone();
        classless.class_id.clear();
        classless.slots.values_mut().for_each(|v| v.clear());
        classless.pouch.clear();
        for rule in [None, progression::deck_rule(&d, &d.world_ids()[0])] {
            let now = build_from(&d.cat, UnitBuild::row(e, 0, rule.as_ref())).unit;
            let mut before = build_unit_in(&d.cat, &classless, 0, rule.as_ref());
            // U3.2-S3 (FB-33): a classless build wears its gear unrestricted; with no gear the flag is all that differs
            assert!(before.stats.proficiency.unrestricted, "a classless build is unrestricted");
            before.stats.proficiency.unrestricted = false;
            assert!(same(&now, &before), "fixed unit {} differs from its classless build", e.id);
        }
    }
}

#[test]
fn every_shipped_hero_regathers_its_new_game_passives() {
    let d = data();
    for (w, h) in heroes(&d) {
        // U4.2-S3: the new-game list leaves the temperament out; the build gathers it back from the character
        assert_eq!(progression::member_passives(&d, &h), saved_with_temperament(&d, &h), "hero {w}:{} (a save made now builds as before)", h.id);
        let temperament = json::s(progression::character_of(&d, &h), "temperament");
        assert!(temperament.is_empty() || !h.passives.contains(&temperament), "hero {w}:{}: the save leaves the temperament out", h.id);
    }
}

/// A world's characters, else its catalog units: one roster for every kind of unit, no per-unit code.
struct AnyUnit<'a>(WorldRoster<'a>);

impl Roster for AnyUnit<'_> {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        self.0.entry(cat, r).or_else(|| cat.units.get(r).cloned())
    }

    fn overrides(&self, cat: &Catalog, r: &str) -> emberwake_rules::Overrides {
        self.0.overrides(cat, r)
    }
}

#[test]
fn any_unit_npc_included_fights_through_the_one_builder() {
    let d = data();
    let (w, hero) = heroes(&d).into_iter().next().expect("a party hero");
    let roster = AnyUnit(WorldRoster::new(&d, w.clone()));
    // the world character with the most overrides, a derived enemy template, a fixed unit
    let sources = |c: &Dict| d.character_passives(c, &w).len() + json::aget(c, "named_items").len() + json::dget(c, "tuning").len();
    let npc = d.own_entries(&w, "characters").iter().filter(|c| json::s(c, "role") != "party" && roster.0.entry(&d.cat, &json::s(c, "id")).is_some()).max_by_key(|c| sources(c)).map(|c| json::s(c, "id")).expect("a world character");
    let template = d.cat.units.values().find(|u| u.side == "enemy" && u.stat_mode == "derived").map(|u| u.id.clone()).expect("an enemy template");
    let fixed = d.cat.units.values().find(|u| u.stat_mode == "fixed").map(|u| u.id.clone()).expect("a fixed unit");
    let built = progression::build_played(&d, &hero, &w).expect("the hero builds");
    let party = vec![Prebuilt { profile: AiProfile::for_unit(&d.cat, &built.def), default_intent: built.def.default_intent.clone(), unit: built.unit, art: String::new(), palette: String::new(), key: hero.id.clone() }];
    let mut row = d.own_entries(&w, "encounters").first().expect("an encounter").clone();
    for k in ["phases", "guests", "protect", "waves", "objective", "stay_behind", "party_fates"] {
        row.shift_remove(k);
    }
    row.insert("party".into(), serde_json::json!([]));
    row.insert("enemies".into(), serde_json::json!([npc, template, fixed]));
    row.insert("max_rounds".into(), serde_json::json!(4));
    let def = EncounterDef::from_obj(&row);
    let mut s = Session::start_with(d.cat.clone(), def, &roster, party);
    assert_eq!(s.w.units.len(), 4, "the hero and the three enemies take the field");
    for (i, r) in [&npc, &template, &fixed].into_iter().enumerate() {
        let e = roster.entry(&d.cat, r).expect("its row");
        let want = build_from(&d.cat, UnitBuild { row: &e, default_ap: s.def.unit_ap, deck: s.w.units[s.w.enemies[i]].deck_rule.as_ref(), wield: s.def.ruleset.numbers.wield(), over: roster.overrides(&d.cat, r), growth: None }).unit;
        let got = &s.w.units[s.w.enemies[i]];
        assert_eq!((got.max_hp, got.initiative, &got.passives, &got.catalog_id), (want.max_hp, want.initiative, &want.passives, &want.catalog_id), "enemy {r} is the builder's unit");
    }
    let mut rounds = 0;
    while !s.combat_over && rounds < 4 {
        s.plan_apply_preset_all("balanced");
        assert!(s.confirm_plan(), "round {} confirms", rounds + 1);
        for _ in 0..10_000 {
            if s.phase == Phase::Replay {
                break;
            }
            s.step_next();
        }
        assert_eq!(s.phase, Phase::Replay, "round {} resolves", rounds + 1);
        rounds += 1;
        if !s.combat_over && s.can_advance_round() {
            s.advance_round();
        }
    }
    let acted: Vec<usize> = s.history.iter().flat_map(|h| h.actions.iter().map(|a| a.unit)).collect();
    for u in 0..s.w.units.len() {
        assert!(acted.contains(&u), "unit {} ({}) acted in the fight", u, s.w.units[u].name);
    }
}
