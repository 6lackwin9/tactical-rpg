//! BattleBridge (scripts/exploration/battle_bridge.gd): the shared way out of a battle into the game state.
//! won: XP to every active member, the quest hook, the encounter purse; lost / fled change nothing here (the
//! runners apply the defeat and flee rules). Building the battle itself (GameEncounter) lives with the session.

use crate::data::Data;
use crate::json::{self, Dict};
use crate::state::GameState;
use crate::{progression, purse, quest};
use serde_json::Value;

pub const OUTCOMES: [&str; 3] = ["won", "lost", "fled"];

/// WorldEncounter's roster: refs are world characters resolved in `world`; each spawns as its base unit wearing the
/// character (WorldFactory.unit_entry) and then gets its named items, passives and tuning (apply_character).
pub struct WorldRoster<'a> {
    pub d: &'a Data,
    pub world: String,
}

impl aegis_battle::session::Roster for WorldRoster<'_> {
    fn entry(&self, _cat: &aegis_data::Catalog, r: &str) -> Option<aegis_data::UnitDef> {
        let e = self.d.character_unit_entry(self.d.resolve(&self.world, "characters", r));
        (!e.is_empty()).then(|| aegis_data::UnitDef::from_obj(&e))
    }

    fn customize(&self, cat: &aegis_data::Catalog, r: &str, u: &mut aegis_rules::Unit) {
        apply_character(self.d, cat, self.d.resolve(&self.world, "characters", r), &self.world, u);
    }

    fn tier(&self, _cat: &aegis_data::Catalog, r: &str) -> String {
        let e = self.d.character_unit_entry(self.d.resolve(&self.world, "characters", r));
        json::s_or(&e, "tier", "standard")
    }
}

/// WorldFactory.named_item: the base item with the named item's name, story, tuning deltas and extra stat bonuses.
pub fn named_item(d: &Data, item_entry: &Dict) -> Option<aegis_data::ItemDef> {
    let mut base = d.entry("items", &json::s(item_entry, "base_item")).clone();
    if base.is_empty() {
        return None;
    }
    let tuning = json::dget(item_entry, "tuning");
    let name = json::s_or(item_entry, "name", &json::s(&base, "name"));
    base.insert("name".into(), Value::from(name));
    let story = json::s(item_entry, "description");
    if !story.is_empty() {
        base.insert("description".into(), Value::from(story));
    }
    for (field, key) in [("base_power", "power"), ("mitigation", "mitigation"), ("burden", "burden")] {
        let v = (json::i(&base, field, 0) + json::i(tuning, key, 0)).max(0);
        base.insert(field.into(), Value::from(v));
    }
    let mut bonuses = json::dget(&base, "stat_bonuses").clone();
    for (stat, v) in json::dget(item_entry, "stat_bonuses") {
        let sum = bonuses.get(stat).map(json::to_int).unwrap_or(0) + json::to_int(v);
        bonuses.insert(stat.clone(), Value::from(sum));
    }
    base.insert("stat_bonuses".into(), Value::Object(bonuses));
    let qid = json::s_or(item_entry, "_qid", &json::s(item_entry, "id")).replace(':', "_");
    let id = format!("{}__{}", json::s_or(&base, "id", "item"), qid);
    base.insert("id".into(), Value::from(id));
    Some(aegis_data::ItemDef::from_obj(&base))
}

/// WorldFactory.apply_character: named items worn, lore passives applied, vitals re-derived when geared (derived
/// units only), then the tuning deltas.
pub fn apply_character(d: &Data, cat: &aegis_data::Catalog, character: &Dict, world: &str, u: &mut aegis_rules::Unit) {
    let mut geared = false;
    for r in json::aget(character, "named_items") {
        let item_entry = d.resolve(world, "items", &json::gstr(r));
        if item_entry.is_empty() {
            continue;
        }
        if let Some(item) = named_item(d, item_entry).filter(|i| i.is_equipment()) {
            let slot = aegis_rules::gear::Slot::parse(&item.slot).unwrap_or(aegis_rules::gear::Slot::MainHand);
            u.inventory.equip(aegis_rules::gear::Equipment::new(&item), slot);
            geared = true;
        }
    }
    for id in d.character_passives(character, world) {
        if let Some(p) = cat.passives.get(&id) {
            u.apply_passive(cat, p);
        }
    }
    let base_unit = d.entry("units", &json::s(character, "base_unit"));
    if geared && json::s_or(base_unit, "stat_mode", "fixed") == "derived" {
        u.sync_from_stats_and_gear(cat);
        u.hp = u.max_hp;
    }
    let tuning = json::dget(character, "tuning");
    let hp_pct = json::i(tuning, "hp_pct", 0);
    if hp_pct != 0 {
        u.max_hp = (json::roundi(u.max_hp as f64 * (1.0 + hp_pct as f64 / 100.0)) as i32).max(1);
        u.hp = u.max_hp;
    }
    let power = json::i(tuning, "power", 0) as i32;
    if power != 0 {
        u.power_override = (aegis_rules::damage::power_of(u, u.attack_range > 1, false) + power).max(1);
    }
    let armor = json::i(tuning, "armor", 0) as i32;
    if armor != 0 {
        u.armor_override = (aegis_rules::damage::armor_rating(u) + armor).max(0);
    }
    let speed = json::i(tuning, "speed", 0) as i32;
    if speed != 0 {
        u.speed = (u.speed + speed).max(1);
    }
    let ap = json::i(tuning, "ap", 0) as i32;
    if ap != 0 {
        u.max_ap = (u.max_ap + ap).max(1);
        u.ap = u.max_ap;
    }
}

/// (level, tier) of the encounter's enemies, reinforcements and phase waves, seen from `world_id`.
pub fn enemy_levels(d: &Data, world_id: &str, encounter_qid: &str) -> Vec<(i64, String)> {
    let enc = d.resolve(world_id, "encounters", encounter_qid);
    let mut refs = json::sget(enc, "enemies");
    refs.extend(json::sget(enc, "wave_units"));
    for phase in json::aget(enc, "phases") {
        for fx in json::aget(json::dict(phase), "effects") {
            let fx = json::dict(fx);
            if json::s(fx, "type") == "wave" {
                refs.extend(json::sget(fx, "units"));
            }
        }
    }
    let mut out = Vec::new();
    for r in refs {
        let entry = d.character_unit_entry(d.resolve(world_id, "characters", &r));
        if entry.is_empty() {
            continue;
        }
        out.push((json::i(&entry, "level", 1), json::s_or(&entry, "tier", "standard")));
    }
    out
}

/// {ok, errors, xp: member id -> award, advanced, purse?}.
pub fn apply(d: &Data, gs: &mut GameState, encounter_qid: &str, outcome: &str) -> Dict {
    let done = |ok: bool, errors: &[String], xp: Dict, advanced: &[String]| {
        let mut out = json::ok_result(ok, errors);
        out.insert("xp".into(), Value::Object(xp));
        out.insert("advanced".into(), json::strs(advanced));
        out
    };
    if !OUTCOMES.contains(&outcome) {
        return done(false, &[format!("Unknown battle outcome '{outcome}' (use won, lost or fled).")], Dict::new(), &[]);
    }
    if outcome != "won" {
        return done(true, &[], Dict::new(), &[]);
    }
    let enemies = enemy_levels(d, &gs.world_id, encounter_qid);
    let xp = progression::award_encounter(d, &mut gs.party, &enemies);
    let advanced = quest::on_encounter_won(d, gs, encounter_qid);
    let mut out = done(true, &[], xp, &advanced);
    let paid = purse::pay(d, gs, encounter_qid);
    if !paid.is_empty() {
        out.insert("purse".into(), Value::Object(paid));
    }
    out
}
