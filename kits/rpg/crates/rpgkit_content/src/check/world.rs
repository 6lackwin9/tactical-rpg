//! The world checks (WorldCatalog.validate_entry / validate_world / validate_all / world_warnings / warnings_all): every
//! problem of a world entry (id, name, enums, references across the lineage and connections, overrides, the rules of
//! its kind) and of a world manifest, and the soft warnings over a world.

use super::{board, campaign, dialogue, encounter, map, quest, shop, story, travel, View};
use crate::source::{content, fields};
use aegis_content::world::split_qid;
use aegis_core::json::{self, gstr, Dict};
use serde_json::Value;

/// WorldSchema.KINDS, then CAMPAIGN1's and STORYUI1's kinds (the loader's list).
pub const KINDS: [&str; 19] = crate::source::WORLD_KINDS;
const TUNING_KEYS: [&str; 5] = ["hp_pct", "power", "armor", "speed", "ap"];
const ITEM_TUNING_KEYS: [&str; 3] = ["power", "mitigation", "burden"];
const STANCES: [&str; 4] = ["ally", "rival", "enemy", "neutral"];
const PARTY_MAX: i64 = 4;
const SUPPLIES_MAX: i64 = 99;
const GONE_FLAGS_NOT_LIST: &str = "Gone flags must be a list of {\"key\", \"value\"} conditions.";

pub fn qid(w: &str, id: &str) -> String {
    format!("{w}:{id}")
}

fn singular(kind: &str) -> &str {
    kind.strip_suffix('s').unwrap_or(kind)
}

/// WorldSchema.normalize.
fn normalize(kind: &str, entry: &Dict) -> Dict {
    aegis_content::schema::normalize(fields("world", kind), entry, true)
}

fn field(kind: &str, key: &str) -> &'static Dict {
    fields("world", kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).unwrap_or_else(|| json::empty())
}

/// WorldCatalog.ref_errors: the problems of `r` as a `ref_kind` ref written in world `w`.
pub fn ref_errors(v: &View, w: &str, label: &str, ref_kind: &str, r: &str) -> Vec<String> {
    check_ref(v, w, &json::obj([("label", Value::from(label))]), ref_kind, r)
}

/// WorldCatalog._check_ref: a base catalog ref ("base:<kind>"), a world id, or a world ref (local, inherited or a
/// qualified id of an ancestor or connected world) that resolves and passes the field's filter.
fn check_ref(v: &View, w: &str, f: &Dict, ref_kind: &str, r: &str) -> Vec<String> {
    let label = json::s(f, "label");
    if let Some(base_kind) = ref_kind.strip_prefix("base:") {
        let target = v.get(base_kind, r);
        return if target.is_empty() {
            vec![format!("{label}: unknown base {} '{r}'.", singular(base_kind))]
        } else if !super::passes_filter(f, target) {
            vec![format!("{label}: '{r}' can't be used here.")]
        } else {
            Vec::new()
        };
    }
    if ref_kind == "worlds" {
        return if v.worlds.has_world(r) { Vec::new() } else { vec![format!("{label}: unknown world '{r}'.")] };
    }
    if r.contains(':') {
        match split_qid(r) {
            Some((rw, _)) if v.worlds.has_world(&rw) => {
                if !v.worlds.can_reach_world(w, &rw) {
                    return vec![format!("{label}: world '{rw}' is neither an ancestor nor connected (add it to Extends or Connects).")];
                }
            }
            _ => return vec![format!("{label}: unknown world in '{r}'.")],
        }
    }
    let hit = v.worlds.resolve(w, ref_kind, r);
    if hit.is_empty() {
        vec![format!("{label}: unknown {} '{r}'.", singular(ref_kind))]
    } else if !super::passes_filter(f, hit) {
        vec![format!("{label}: '{r}' can't be used here.")]
    } else {
        Vec::new()
    }
}

/// WorldCatalog.validate_entry: every problem of `entry` as a `kind` entry of world `w` (`current_id`: the id it
/// replaces).
pub fn validate_entry(v: &View, w: &str, kind: &str, entry: &Dict, current_id: &str) -> Vec<String> {
    let e = normalize(kind, &crate::source::folded(kind, entry));
    let mut errors = Vec::new();
    let id = json::s(&e, "id");
    if id.is_empty() {
        errors.push("ID is required.".into());
    } else if !aegis_content::schema::is_id(&id) {
        errors.push(format!("ID '{id}' must be lowercase snake_case (a-z, 0-9, _)."));
    } else if id != current_id && v.worlds.own_entries(w, kind).iter().any(|x| json::s(x, "id") == id) {
        errors.push(format!("ID '{id}' is already used by another {} entry in this world.", singular(kind)));
    }
    if json::s(&e, "name").trim().is_empty() {
        errors.push("Name is required.".into());
    }
    for f in fields("world", kind).iter().map(json::dict).filter(|f| json::s(f, "type") == "enum" && super::enum_checked(f)) {
        let value = json::s(&e, &json::s(f, "key"));
        let options = super::options_of(v, f);
        if !options.contains(&value) {
            errors.push(format!("{}: '{value}' is not one of {}.", json::s(f, "label"), options.join(", ")));
        }
    }
    errors.extend(entry_ref_errors(v, w, kind, &e));
    errors.extend(override_errors(v, w, kind, &e));
    match kind {
        "regions" => errors.extend(region_errors(v, w, &e, &id)),
        "locations" => errors.extend(location_errors(v, w, &e)),
        "factions" => errors.extend(faction_errors(v, w, &e)),
        "characters" => errors.extend(character_errors(v, w, &e)),
        "items" => errors.extend(item_errors(v, &e)),
        "quests" => {
            errors.extend(quest_errors(v, w, &e));
            // STORYUI1 4.5: the board fields
            errors.extend(board::quest_board_errors(v, w, &e));
        }
        "encounters" => errors.extend(encounter_errors(v, w, &e)),
        "loot" => errors.extend(quest::loot_entry_errors(v, &e)),
        "dialogues" => errors.extend(dialogue::tree_errors(v, w, &e)),
        "shops" => errors.extend(shop::shop_errors(v, w, &e)),
        "routes" => errors.extend(travel::route_errors(v, w, &e)),
        "meters" => errors.extend(campaign::meter_errors(v, w, &e)),
        "checkpoints" => errors.extend(campaign::checkpoint_errors(v, w, &e)),
        "triggers" => errors.extend(campaign::trigger_errors(v, w, &e)),
        "vignettes" => errors.extend(story::vignette_errors(v, w, &e)),
        "sequences" => errors.extend(story::sequence_errors(v, w, &e)),
        "boards" => errors.extend(board::board_errors(&e)),
        _ => {}
    }
    // CAMPAIGN1 11.4: reserved flag keys in the entry's conditions and flag lists
    errors.extend(campaign::read_key_errors(v, w, &e));
    errors
}

/// WorldCatalog.blocking_errors: the problems that refuse a world write (id, name, enums, overrides, the shape of the
/// json fields). Unresolved references and incomplete entries do not block: worlds are full of mutual references.
pub fn blocking_errors(v: &View, w: &str, kind: &str, entry: &Dict, current_id: &str) -> Vec<String> {
    let e = normalize(kind, &crate::source::folded(kind, entry));
    let mut errors = Vec::new();
    let id = json::s(&e, "id");
    if id.is_empty() {
        errors.push("ID is required.".into());
    } else if !aegis_content::schema::is_id(&id) {
        errors.push(format!("ID '{id}' must be lowercase snake_case (a-z, 0-9, _)."));
    } else if id != current_id && v.worlds.own_entries(w, kind).iter().any(|x| json::s(x, "id") == id) {
        errors.push(format!("ID '{id}' is already used by another {} entry in this world.", singular(kind)));
    }
    if json::s(&e, "name").trim().is_empty() {
        errors.push("Name is required.".into());
    }
    for f in fields("world", kind).iter().map(json::dict).filter(|f| json::s(f, "type") == "enum" && super::enum_checked(f)) {
        let value = json::s(&e, &json::s(f, "key"));
        let options = super::options_of(v, f);
        if !options.contains(&value) {
            errors.push(format!("{}: '{value}' is not one of {}.", json::s(f, "label"), options.join(", ")));
        }
    }
    errors.extend(override_errors(v, w, kind, &e));
    errors.extend(shape_errors(kind, &e));
    errors
}

/// WorldCatalog._shape_errors: the json fields hold the shape their readers expect.
fn shape_errors(kind: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let label = |key: &str| json::s(field(kind, key), "label");
    let list_errors = |errors: &mut Vec<String>, keys: &[&str]| {
        for key in keys.iter().filter(|k| !json::get(e, k).is_array()) {
            errors.push(format!("{} must be a list.", label(key)));
        }
    };
    match kind {
        "characters" => {
            errors.extend(tuning_errors(json::get(e, "tuning"), &TUNING_KEYS, "Character"));
            if !json::get(e, "gone_flags").is_array() {
                errors.push(GONE_FLAGS_NOT_LIST.into());
            }
        }
        "items" => errors.extend(tuning_errors(json::get(e, "tuning"), &ITEM_TUNING_KEYS, "Item")),
        "dialogues" if !json::get(e, "nodes").is_array() => errors.push("Nodes must be a list of node objects.".into()),
        "shops" => list_errors(&mut errors, &["stock", "loot", "buys", "requires_flags"]),
        "locations" if !json::get(e, "exits").is_array() => errors.push("Exits must be a list of {\"to\", \"label\"} objects.".into()),
        "routes" => {
            for key in ["encounters", "elites", "scripted", "requires_flags", "unless_flags"] {
                if !e.get(key).unwrap_or(&Value::Array(Vec::new())).is_array() {
                    errors.push(format!("{} must be a list.", label(key)));
                }
            }
        }
        "factions" => match json::get(e, "relations").as_object() {
            None => errors.push("Relations must be an object {faction: stance}.".into()),
            Some(rel) => {
                for stance in rel.values().filter(|s| !STANCES.contains(&gstr(s).as_str())) {
                    errors.push(format!("Relation stance '{}' must be ally, rival, enemy or neutral.", gstr(stance)));
                }
            }
        },
        "quests" => {
            if !json::get(e, "steps").is_array() {
                errors.push("Steps must be a list.".into());
            }
            list_errors(&mut errors, &["requires_flags", "complete_flags", "variants", "loot_pool"]);
            if !json::get(e, "links").is_object() {
                errors.push("Links must be an object.".into());
            }
            if !json::get(e, "reputation").is_object() {
                errors.push("Reputation must be an object {faction: delta}.".into());
            }
        }
        "loot" if !json::get(e, "requires_flags").is_array() => errors.push("Requires flags must be a list.".into()),
        // CAMPAIGN1 11.2: the json fields of the campaign kinds
        "meters" => {
            list_errors(&mut errors, &["bands", "rules", "inputs", "modifiers", "floors"]);
            if !json::get(e, "hud").is_object() {
                errors.push("HUD must be an object {\"short\", \"icon\", \"order\"}.".into());
            }
        }
        "checkpoints" => list_errors(&mut errors, &["met_when", "warn_days", "effects"]),
        "triggers" => list_errors(&mut errors, &["when", "effects", "then"]),
        // STORYUI1 4.1-4.4: the json fields of the story kinds
        "vignettes" => list_errors(&mut errors, &["cards"]),
        "sequences" => list_errors(&mut errors, &["steps"]),
        "boards" => list_errors(&mut errors, &["requires_flags"]),
        "encounters" => {
            if json::aget(e, "party").len() as i64 > PARTY_MAX {
                errors.push("The party holds at most 4 members.".into());
            }
            match json::get(e, "exits").as_array() {
                None => errors.push("Exit tiles must be a list of [x, y] pairs.".into()),
                Some(list) => {
                    let off = |x: &Value| {
                        let pair = json::arr(x);
                        let (w, h) = (rpgkit_battle::grid::GRID_W as i64, rpgkit_battle::grid::GRID_H as i64);
                        pair.len() != 2 || json::to_int(&pair[0]) < 0 || json::to_int(&pair[1]) < 0 || json::to_int(&pair[0]) >= w || json::to_int(&pair[1]) >= h
                    };
                    if list.iter().any(off) {
                        errors.push("Exit tiles are [x, y] pairs on the 12x9 grid.".into());
                    }
                }
            }
        }
        _ => {}
    }
    errors
}

/// WorldCatalog._ref_errors: every ref field of the kind (lists without dupes unless allowed).
fn entry_ref_errors(v: &View, w: &str, kind: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    // PHASEFOLD 1.3: the arrival's units are checked as the retired wave_units field (after enemies) was.
    let arrival = if kind == "encounters" { encounter::arrival_units(e) } else { None };
    for f in fields("world", kind).iter().map(json::dict) {
        errors.extend(field_ref_errors(v, w, f, json::get(e, &json::s(f, "key"))));
        if let (Some(units), "enemies") = (&arrival, json::s(f, "key").as_str()) {
            errors.extend(field_ref_errors(v, w, encounter::wave_field(true), units));
        }
    }
    errors
}

/// The reference problems of one ref field holding `value`: listed twice, then each ref (`check_ref`).
fn field_ref_errors(v: &View, w: &str, f: &Dict, value: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    let ref_kind = json::s(f, "ref");
    if ref_kind.is_empty() {
        return errors;
    }
    let mut values: Vec<String> = Vec::new();
    match value {
        Value::Array(list) => {
            values = list.iter().map(gstr).collect();
            if !json::b(f, "dupes", false) {
                let mut seen: Vec<&String> = Vec::new();
                for x in &values {
                    if seen.contains(&x) {
                        errors.push(format!("{} lists '{x}' twice.", json::s(f, "label")));
                    }
                    seen.push(x);
                }
            }
        }
        raw if !gstr(raw).is_empty() && !raw.is_null() => values.push(gstr(raw)),
        _ => {}
    }
    for x in values {
        errors.extend(check_ref(v, w, f, &ref_kind, &x));
    }
    errors
}

/// WorldCatalog._override_errors: `override` names an entry of an ancestor world by its qualified id.
fn override_errors(v: &View, w: &str, kind: &str, e: &Dict) -> Vec<String> {
    let ov = json::s(e, "override");
    if ov.is_empty() {
        return Vec::new();
    }
    let error = match split_qid(&ov) {
        None => "Overrides must be a qualified id (world:id).".to_string(),
        Some((ow, _)) if ow == w => "An entry can only override an entry of an ancestor world.".into(),
        Some((ow, _)) if !v.worlds.lineage(w).contains(&ow) => format!("Overrides: '{ow}' is not an ancestor world."),
        Some((ow, local)) if v.worlds.own_by_id(&ow, kind, &local).is_none() => format!("Overrides: no {} '{local}' in world '{ow}'.", singular(kind)),
        Some(_) => return Vec::new(),
    };
    vec![error]
}

fn region_errors(v: &View, w: &str, e: &Dict, id: &str) -> Vec<String> {
    let mut seen = vec![id.to_string()];
    let mut cursor = json::s(e, "parent");
    while !cursor.is_empty() {
        let hit = v.worlds.resolve(w, "regions", &cursor);
        if hit.is_empty() {
            break;
        }
        let hit_id = json::s(hit, "id");
        if seen.contains(&hit_id) {
            return vec!["The region hierarchy is circular.".into()];
        }
        seen.push(hit_id);
        cursor = json::s(hit, "parent");
    }
    Vec::new()
}

fn location_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let Some(exits) = json::get(e, "exits").as_array() else {
        return vec!["Exits must be a list of {\"to\", \"label\"} objects.".into()];
    };
    let mut errors = Vec::new();
    for x in exits {
        let to = json::s(json::dict(x), "to");
        if to.is_empty() {
            errors.push("Every exit needs a \"to\" location.".into());
            continue;
        }
        if let Some((tw, _)) = split_qid(&to).filter(|_| to.contains(':')) {
            if v.worlds.has_world(&tw) && !v.worlds.can_reach_world(w, &tw) {
                errors.push(format!("Exit '{to}': world '{tw}' is neither an ancestor nor connected."));
                continue;
            }
        }
        if v.worlds.resolve(w, "locations", &to).is_empty() {
            errors.push(format!("Exit leads to unknown location '{to}'."));
        }
    }
    errors.extend(map::location_errors(v, w, e));
    errors
}

fn faction_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let Some(rel) = json::get(e, "relations").as_object() else {
        return vec!["Relations must be an object {faction: stance}.".into()];
    };
    let mut errors = Vec::new();
    for (other, stance) in rel {
        if v.worlds.resolve(w, "factions", other).is_empty() {
            errors.push(format!("Relation to unknown faction '{other}'."));
        }
        if !STANCES.contains(&gstr(stance).as_str()) {
            errors.push(format!("Relation stance '{}' must be ally, rival, enemy or neutral.", gstr(stance)));
        }
    }
    errors
}

fn tuning_errors(raw: &Value, allowed: &[&str], what: &str) -> Vec<String> {
    let Some(tuning) = raw.as_object() else {
        return vec![format!("{what} tuning must be an object.")];
    };
    let mut errors = Vec::new();
    for (key, x) in tuning {
        if !allowed.contains(&key.as_str()) {
            errors.push(format!("Unknown {} tuning '{key}' (allowed: {}).", what.to_lowercase(), allowed.join(", ")));
        } else if !x.is_number() {
            errors.push(format!("Tuning '{key}' must be a number."));
        }
    }
    errors
}

fn character_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    if json::s(e, "base_unit").is_empty() {
        errors.push("Choose the base unit this character plays as.".into());
    }
    errors.extend(tuning_errors(json::get(e, "tuning"), &TUNING_KEYS, "Character"));
    errors.extend(super::assets::body_errors(v, &json::s(e, "art")));
    match e.get("gone_flags").unwrap_or(&Value::Array(Vec::new())) {
        gone @ Value::Array(_) => errors.extend(quest::condition_errors(gone, "Gone flags")),
        _ => errors.push(GONE_FLAGS_NOT_LIST.into()),
    }
    let mut slots: Vec<(String, String)> = Vec::new();
    for r in json::sget(e, "named_items") {
        let named = v.worlds.resolve(w, "items", &r);
        if named.is_empty() {
            continue;
        }
        let base = v.get("items", &json::s(named, "base_item"));
        if base.is_empty() {
            continue;
        }
        if !["WEAPON", "SHIELD", "ARMOR", "ACCESSORY"].contains(&json::s(base, "item_type").as_str()) {
            errors.push(format!("Named item '{r}' isn't equipment, so it can't be worn."));
            continue;
        }
        let slot = json::s(base, "slot");
        if let Some((_, other)) = slots.iter().find(|(s, _)| *s == slot) {
            errors.push(format!("'{other}' and '{r}' compete for the {} slot.", slot.to_lowercase().replace('_', " ")));
        }
        slots.retain(|(s, _)| *s != slot);
        slots.push((slot, r));
    }
    // ROSTER1 KS2, 11.3: a party character's keepsake; RK1: a member id is never a roster read word
    let keepsake = json::s(e, "keepsake");
    if !keepsake.is_empty() {
        if json::s(e, "role") != "party" {
            errors.push("Keepsake: only a party character carries one.".into());
        }
        if v.has("items", &keepsake) {
            errors.extend(keepsake_item_errors(v, &keepsake, "Keepsake"));
        }
    }
    let id = json::s(e, "id");
    if rpgkit_data::ROSTER_WORDS.contains(&id.as_str()) {
        errors.push(format!("Id '{id}' is a roster read word ({}); pick another.", rpgkit_data::ROSTER_WORDS.join(", ")));
    }
    errors
}

fn item_errors(v: &View, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let base_id = json::s(e, "base_item");
    if base_id.is_empty() {
        errors.push("Choose the base item this is a named version of.".into());
    } else if super::item::inherit(v) {
        let base = v.get("items", &base_id);
        if !base.is_empty() && super::item::family_of(v, base).is_empty() {
            errors.push(format!("Base item '{base_id}' does not resolve: its item family '{}' is unknown.", json::s(base, "family")));
        }
    }
    errors.extend(tuning_errors(json::get(e, "tuning"), &ITEM_TUNING_KEYS, "Item"));
    errors
}

fn quest_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let Some(steps) = json::get(e, "steps").as_array() else {
        return vec!["Steps must be a list.".into()];
    };
    let mut errors = Vec::new();
    let templated = !json::s(e, "template").is_empty();
    for (i, step) in steps.iter().enumerate() {
        let step = json::dict(step);
        if json::s(step, "text").trim().is_empty() {
            errors.push(format!("Quest step {} needs a \"text\".", i + 1));
        }
        if templated {
            continue;
        }
        let loc = json::s(step, "location");
        if !loc.is_empty() && v.worlds.resolve(w, "locations", &loc).is_empty() {
            errors.push(format!("Quest step {}: unknown location '{loc}'.", i + 1));
        }
        let enc = json::s(step, "encounter");
        if !enc.is_empty() && v.worlds.resolve(w, "encounters", &enc).is_empty() {
            errors.push(format!("Quest step {}: unknown encounter '{enc}'.", i + 1));
        }
    }
    errors.extend(quest::world_quest_errors(v, w, e));
    errors
}

/// WorldCatalog.guest_ok: false only for an npc character whose base unit is not on the party side.
fn guest_ok(v: &View, w: &str, r: &str) -> bool {
    let c = v.worlds.resolve(w, "characters", r);
    if c.is_empty() || json::s(c, "role") != "npc" {
        return true;
    }
    json::s(v.get("units", &json::s(c, "base_unit")), "side") == "party"
}

fn encounter_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let party = json::aget(e, "party");
    if party.is_empty() {
        errors.push("Add at least one party member.".into());
    }
    if party.len() as i64 > PARTY_MAX {
        errors.push("The party holds at most 4 members.".into());
    }
    if json::aget(e, "enemies").is_empty() {
        errors.push("Add at least one enemy.".into());
    }
    errors.extend(encounter::arrival_errors(e));
    errors.extend(encounter::phase_problems(e, &|r: &str| check_ref(v, w, encounter::wave_field(true), "characters", r).is_empty()));
    errors.extend(encounter::objective_problems(v, e, &|r: &str| guest_ok(v, w, r)));
    errors.extend(encounter::morale_problems(e));
    errors.extend(encounter::start_status_problems(e.get("start_status").unwrap_or(&Value::Array(Vec::new()))));
    errors.extend(encounter::purse_problems(e));
    errors.extend(super::reward::key_loot_problems(v, w, e));
    errors.extend(super::reward::objective_reward_problems(v, w, e));
    match json::get(e, "exits") {
        Value::Array(list) => {
            for x in list {
                let pair = json::arr(x);
                if pair.len() != 2 {
                    errors.push("Exit tiles are [x, y] pairs.".into());
                    break;
                }
                let (px, py) = (json::to_int(&pair[0]), json::to_int(&pair[1]));
                if px < 0 || py < 0 || px >= rpgkit_battle::grid::GRID_W as i64 || py >= rpgkit_battle::grid::GRID_H as i64 {
                    errors.push(format!("Exit {} is off the 12x9 grid.", gstr(x)));
                }
            }
        }
        _ => errors.push("Exit tiles must be a list of [x, y] pairs.".into()),
    }
    errors.extend(encounter::rows_problems(e));
    // ENCOUNTER2 S1 and section 6: unkillable refs and the stay-behind block (flag and then are a world encounter's)
    errors.extend(encounter::unkillable_problems(e));
    errors.extend(encounter::stay_behind_problems(v, e, true));
    // HOOKS1 AH1 and DU2: the arena hazards and the deck phase effects (a ref plays as its character's base unit)
    errors.extend(super::hooks::arena_hazards_errors(e));
    let unit_of = |r: &str| v.get("units", &json::s(v.worlds.resolve(w, "characters", r), "base_unit")).clone();
    errors.extend(super::hooks::deck_effect_errors(v, e, Some(w), &unit_of));
    // ROSTER1 BK6: the fewest heroes the fight takes, at most the world's party max
    let party_min = e.get("party_min").cloned().unwrap_or(json::int(0));
    let top = rpgkit_data::PARTY_MIN_MAX.min(party_max(v, w));
    if !campaign::int_in(&party_min, 0, top) {
        let why = if top < rpgkit_data::PARTY_MIN_MAX { " (the world's party max)" } else { "" };
        errors.push(format!("Party min must be a whole number 0..{top}{why}."));
    }
    errors
}

/// WorldCatalog.upsert's store step (the blocking checks are the caller's): `entry` normalized into world `w`'s own
/// `kind` list, in place of the entry with its id, else appended.
pub fn put_entry(v: &mut View, w: &str, kind: &str, entry: &Dict) {
    let mut clean = normalize(kind, entry);
    let id = json::s(&clean, "id");
    clean.insert("_world".into(), Value::from(w));
    clean.insert("_qid".into(), Value::from(qid(w, &id)));
    let list = v.worlds.worlds.entry(w.to_string()).or_default().own.entry(kind.to_string()).or_default();
    match list.iter_mut().find(|e| json::s(e, "id") == id) {
        Some(slot) => *slot = clean,
        None => list.push(clean),
    }
}

// ---- manifests ----

/// WorldCatalog.has_cycle: the world's `extends` chain leads back to it.
fn has_cycle(v: &View, w: &str, path: &mut Vec<String>) -> bool {
    if path.iter().any(|p| p == w) {
        return true;
    }
    if !v.worlds.has_world(w) {
        return false;
    }
    path.push(w.to_string());
    let cycle = json::sget(v.worlds.manifest(w), "extends").iter().any(|parent| has_cycle(v, parent, path));
    path.pop();
    cycle
}

/// WorldCatalog.party_character_errors: what keeps character `id` from joining the party.
pub fn party_character_errors(v: &View, w: &str, id: &str) -> Vec<String> {
    let c = v.worlds.resolve(w, "characters", id);
    if c.is_empty() {
        return vec![format!("'{id}' is not a known character.")];
    }
    let mut errors = Vec::new();
    let role = json::s_or(c, "role", "npc");
    if role != "party" {
        errors.push(format!("Party hero '{id}' needs role party (has {role})."));
    }
    let base_id = json::s(c, "base_unit");
    let base = v.get("units", &base_id);
    if base.is_empty() || json::s_or(base, "side", "enemy") != "party" {
        errors.push(format!("Party hero '{id}' must play as a party unit (base unit '{base_id}')."));
    }
    if !json::sget(c, "named_items").is_empty() || !json::sget(c, "disciplines").is_empty() || !json::dget(c, "tuning").is_empty() {
        errors.push(format!("Party hero '{id}' may not carry named items, disciplines or tuning yet (RECRUIT1)."));
    }
    errors
}

fn start_party_errors(v: &View, w: &str, raw: &Value) -> Vec<String> {
    let ids = json::strings(raw);
    let mut errors = Vec::new();
    let mut seen: Vec<&String> = Vec::new();
    for id in &ids {
        if seen.contains(&id) {
            errors.push(format!("Start party lists '{id}' twice."));
            continue;
        }
        seen.push(id);
        let unit = v.get("units", id);
        let character = v.worlds.resolve(w, "characters", id);
        if !unit.is_empty() && !character.is_empty() {
            errors.push(format!("'{id}' is both a unit and a character."));
        } else if !character.is_empty() {
            errors.extend(party_character_errors(v, w, id));
        } else if unit.is_empty() {
            errors.push(format!("Start party unit '{id}' is not a known unit."));
        } else if json::s_or(unit, "side", "enemy") != "party" {
            errors.push(format!("Start party unit '{id}' is not a party unit."));
        }
    }
    if ids.len() as i64 > PARTY_MAX {
        errors.push("A start party holds at most 4 units.".into());
    }
    errors
}

/// WorldCatalog.party_max: the nearest manifest in the lineage with party_max >= 1, else PARTY_MAX.
fn party_max(v: &View, w: &str) -> i64 {
    v.worlds.lineage(w).iter().rev().map(|lw| json::get(v.worlds.manifest(lw), "party_max")).find(|cap| json::is_int(cap) && json::to_int(cap) >= 1).map_or(PARTY_MAX, |cap| json::to_int(cap).min(PARTY_MAX))
}

/// WorldCatalog.start_party: the world's own non-empty start party, else the nearest ancestor's.
fn start_party(v: &View, w: &str) -> Vec<String> {
    v.worlds.lineage(w).iter().rev().map(|lw| json::sget(v.worlds.manifest(lw), "start_party")).find(|ids| !ids.is_empty()).unwrap_or_default()
}

/// WorldCatalog.manifest_errors.
pub fn manifest_errors(v: &View, w: &str) -> Vec<String> {
    let m = v.worlds.manifest(w);
    if m.is_empty() {
        return vec![format!("No world '{w}'.")];
    }
    let mut errors = Vec::new();
    if json::s(m, "name").trim().is_empty() {
        errors.push("The world needs a name.".into());
    }
    if has_cycle(v, w, &mut Vec::new()) {
        errors.push("The inheritance is circular.".into());
    }
    for (key, label) in [("extends", "Extends"), ("connects", "Connects to")] {
        for other in json::sget(m, key) {
            if other == w {
                errors.push(format!("{label} itself."));
            } else if !v.worlds.has_world(&other) {
                errors.push(format!("{label} unknown world '{other}'."));
            }
        }
    }
    let start = json::s(m, "start_location");
    if !start.is_empty() && v.worlds.resolve(w, "locations", &start).is_empty() {
        errors.push(format!("Start location '{start}' is unknown."));
    }
    errors.extend(start_party_errors(v, w, m.get("start_party").unwrap_or(&Value::Array(Vec::new()))));
    let cap = m.get("party_max").cloned().unwrap_or(json::int(-1));
    if !json::is_int(&cap) || (json::to_int(&cap) != -1 && !(1..=PARTY_MAX).contains(&json::to_int(&cap))) {
        errors.push(format!("Party max must be -1 (inherited) or 1..{PARTY_MAX}."));
    }
    let resolved = start_party(v, w).len() as i64;
    if resolved <= PARTY_MAX && resolved > party_max(v, w) {
        errors.push(format!("Start party has {resolved} heroes; party max is {}.", party_max(v, w)));
    }
    let gold = m.get("start_gold").cloned().unwrap_or(json::int(-1));
    if !json::is_int(&gold) || !(-1..=99999).contains(&json::to_int(&gold)) {
        errors.push("Start gold must be -1 (inherited) or 0..99999.".into());
    }
    let supplies = m.get("start_supplies").cloned().unwrap_or(json::int(-1));
    if !json::is_int(&supplies) || !(-1..=SUPPLIES_MAX).contains(&json::to_int(&supplies)) {
        errors.push("Start supplies must be -1 (inherited) or 0..99.".into());
    }
    errors.extend(dialogue::service_lines_errors(m.get("service_lines").unwrap_or(&Value::Object(Dict::new())), "Service lines"));
    errors.extend(ruleset_errors(v, m));
    errors.extend(campaign::manifest_errors(v, w, m));
    // ROSTER1 11.1: the roster block
    errors.extend(roster_errors(v, w, m));
    errors.extend(story::manifest_errors(v, w, m));
    // HOOKS1 CN1: the class names and paths of the world
    errors.extend(super::hooks::classes_errors(v, m));
    errors
}

/// RULESET1-R (engine-only): the manifest's `ruleset` ("" inherits) is a ruleset id whose row exists (classic has
/// none), and its `plan_reveal_flag` is empty or a flag key.
fn ruleset_errors(v: &View, m: &Dict) -> Vec<String> {
    use rpgkit_rules::ruleset::{CLASSIC, RULESETS};
    let mut errors = Vec::new();
    let r = json::s(m, "ruleset");
    if !r.is_empty() && !RULESETS.contains(&r.as_str()) {
        errors.push(format!("Ruleset: '{r}' is not one of {} (or empty: inherited).", RULESETS.join(", ")));
    } else if !r.is_empty() && r != CLASSIC && !v.has(super::ruleset::KIND, &r) {
        errors.push(format!("Ruleset '{r}' needs its row in {} (data/catalog/{}.json).", super::ruleset::KIND, super::ruleset::KIND));
    }
    let flag = json::s(m, "plan_reveal_flag");
    if !flag.is_empty() && !quest::flag_key_ok(&flag) {
        errors.push(format!("Plan reveal flag '{flag}' must be a flag key (a-z, 0-9, _ and .)."));
    }
    errors
}

// ---- ROSTER1 11.1-11.3 (kit hook; engine-only, no Godot original) ----

const ROSTER_KEYS: [&str; 7] = ["bunks", "bunks_flag", "bunks_max", "crypts", "open_flag", "rules", "keepsakes"];
const ROSTER_RULE_KEYS: [&str; 9] = ["acts", "bleed_out_rounds", "hp_after", "downed_won", "downed_left", "downed_lost", "standing_lost", "wounded_max_hp_pct", "keepsakes_on"];
const KEEPSAKES_KEYS: [&str; 3] = ["on", "by_class", "name"];

/// A roster flag key (`bunks_flag`, `open_flag`): a flag key, not a reserved one ("" = fine).
fn roster_flag_error(m: &Dict, key: &str) -> Option<String> {
    let flag = json::s(m, key);
    if flag.is_empty() {
        return None;
    }
    if !quest::flag_key_ok(&flag) {
        return Some(format!("Roster: {key} '{flag}' must be a flag key (a-z, 0-9, _ and .)."));
    }
    let why = campaign::write_key_error(&flag);
    (!why.is_empty()).then(|| format!("Roster: {key}: {why}"))
}

/// `d[key]`, when present, is a list of lives that pay a keepsake (KEEPSAKE_LIVES).
fn lives_key(d: &Dict, key: &str, label: &str) -> Option<String> {
    use rpgkit_data::KEEPSAKE_LIVES;
    let x = d.get(key)?;
    (!x.as_array().is_some_and(|a| a.iter().all(|l| l.as_str().is_some_and(|s| KEEPSAKE_LIVES.contains(&s))))).then(|| format!("{label}: {key} must be a list of {}.", KEEPSAKE_LIVES.join(", ")))
}

/// KS2, 11.3: the problems of catalog item `id` as a keepsake: an accessory, `value` 0 (never sold), `special`.
pub fn keepsake_item_errors(v: &View, id: &str, label: &str) -> Vec<String> {
    let item = v.get("items", id);
    if item.is_empty() {
        return vec![format!("{label}: unknown item '{id}'.")];
    }
    let mut errors = Vec::new();
    if json::s(item, "item_type") != "ACCESSORY" {
        errors.push(format!("{label}: '{id}' must be an accessory."));
    }
    if json::i(item, "value", 10) != 0 {
        errors.push(format!("{label}: '{id}' must have value 0 (a keepsake is never sold)."));
    }
    if !json::b(item, "special", false) {
        errors.push(format!("{label}: '{id}' must be a special item."));
    }
    errors
}

/// 11.1, 11.2: the problems of a world manifest's `roster` block (absent or {} = today's rules).
pub fn roster_errors(v: &View, w: &str, m: &Dict) -> Vec<String> {
    use campaign::{acts_key, enum_key, int_key, int_in, list, object, text_key, unknown_keys};
    use rpgkit_data::{BLEED_OUT_ROUNDS_MAX, BUNKS_MAX, HP_AFTER, KEEPSAKE_NAME_MAX, KEEPSAKE_NAME_TOKENS, ROSTER_RULES_MAX, SETTLE_HURT, SETTLE_LOSS, WOUNDED_PCT_MAX, WOUNDED_PCT_MIN};
    let mut errors = Vec::new();
    let Some(r) = m.get("roster").and_then(|raw| object(raw, "Roster", &mut errors)) else { return errors };
    let top = BUNKS_MAX as i64;
    errors.extend(unknown_keys(r, &ROSTER_KEYS, "Roster"));
    let bunks = r.get("bunks").cloned().unwrap_or(json::int(0));
    if !int_in(&bunks, 0, top) {
        errors.push(format!("Bunks must be 0 (party max) or 1..{top}."));
    }
    // bunks_max is checked against a valid bunks only (0 = the party max)
    let base = if int_in(&bunks, 1, top) { json::to_int(&bunks) } else { party_max(v, w) };
    errors.extend(int_key(r, "bunks_max", base, top, "Roster"));
    errors.extend(int_key(r, "crypts", 0, top, "Roster"));
    errors.extend(roster_flag_error(r, "bunks_flag"));
    errors.extend(roster_flag_error(r, "open_flag"));
    if let Some(rows) = r.get("rules").and_then(|raw| list(raw, "Roster rules", &mut errors)) {
        if rows.len() > ROSTER_RULES_MAX {
            errors.push(format!("Roster: rules hold at most {ROSTER_RULES_MAX} rows (has {}).", rows.len()));
        }
        let mut open_ended_early = false;
        for (i, raw) in rows.iter().enumerate() {
            let label = format!("Roster rule {}", i + 1);
            let Some(row) = object(raw, &label, &mut errors) else { continue };
            errors.extend(unknown_keys(row, &ROSTER_RULE_KEYS, &label));
            errors.extend(acts_key(row, "acts", &label));
            if json::aget(row, "acts").is_empty() && i + 1 < rows.len() {
                open_ended_early = true;
            }
            errors.extend(int_key(row, "bleed_out_rounds", 0, BLEED_OUT_ROUNDS_MAX as i64, &label));
            errors.extend(enum_key(row, "hp_after", &HP_AFTER, &label));
            errors.extend(enum_key(row, "downed_won", &SETTLE_HURT, &label));
            errors.extend(enum_key(row, "standing_lost", &SETTLE_HURT, &label));
            errors.extend(enum_key(row, "downed_left", &SETTLE_LOSS, &label));
            errors.extend(enum_key(row, "downed_lost", &SETTLE_LOSS, &label));
            errors.extend(int_key(row, "wounded_max_hp_pct", WOUNDED_PCT_MIN as i64, WOUNDED_PCT_MAX as i64, &label));
            errors.extend(lives_key(row, "keepsakes_on", &label));
        }
        if open_ended_early {
            errors.push("Roster rules: only the last row may have no acts (it covers every other act).".into());
        }
    }
    if let Some(ks) = r.get("keepsakes").and_then(|raw| object(raw, "Roster keepsakes", &mut errors)) {
        let label = "Roster keepsakes";
        errors.extend(unknown_keys(ks, &KEEPSAKES_KEYS, label));
        errors.extend(lives_key(ks, "on", label));
        if let Some(by_class) = ks.get("by_class").and_then(|raw| object(raw, "Roster keepsakes by_class", &mut errors)) {
            for (class, item) in by_class {
                if !v.has("classes", class) {
                    errors.push(format!("{label} by_class: unknown class '{class}'."));
                }
                match item.as_str() {
                    Some(id) => errors.extend(keepsake_item_errors(v, id, &format!("{label} by_class {class}"))),
                    None => errors.push(format!("{label} by_class {class}: the item must be an item id.")),
                }
            }
        }
        errors.extend(text_key(ks, "name", KEEPSAKE_NAME_MAX, true, label));
        let name = json::s(ks, "name");
        let mut rest = name.clone();
        for t in KEEPSAKE_NAME_TOKENS {
            rest = rest.replace(t, "");
        }
        if rest.contains('{') || rest.contains('}') {
            errors.push(format!("{label}: name takes only the tokens {}.", KEEPSAKE_NAME_TOKENS.join(" and ")));
        }
    }
    errors
}

/// RK1: the problem of reading `roster.` key `key` in world `w` ("" = fine): a roster word, `departed.<life>`, or a
/// member id (a character of the world, a party unit or a procedural `recruit_<n>`).
pub fn roster_read_key_error(v: &View, w: &str, key: &str) -> String {
    use rpgkit_data::{roster_read_key, RosterRead, DEPART_LIVES};
    match roster_read_key(key) {
        None => format!("'{key}' is not a roster read key (roster.living, roster.bunks, roster.bunks_free, roster.pool, roster.open, roster.departed.<{}>, roster.<member>).", DEPART_LIVES.join(" | ")),
        Some(RosterRead::Member(id)) => {
            let procedural = id.strip_prefix("recruit_").is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));
            let unit = v.get("units", id);
            if procedural || !v.worlds.resolve(w, "characters", id).is_empty() || json::s(unit, "side") == "party" {
                String::new()
            } else {
                format!("'{key}' names no member of this world (a character, a party unit or recruit_<n>).")
            }
        }
        Some(_) => String::new(),
    }
}

// ---- whole worlds ----

/// WorldCatalog.validate_world: {"<world>/world" or "<world>/<kind>:<id>": [errors]} over the world's own entries.
pub fn validate_world(v: &View, w: &str) -> super::Report {
    let mut problems = super::Report::new();
    let m_errors = manifest_errors(v, w);
    if !m_errors.is_empty() {
        problems.insert(format!("{w}/world"), m_errors);
    }
    for kind in KINDS {
        for e in v.worlds.own_entries(w, kind) {
            let id = json::s(e, "id");
            let errors = validate_entry(v, w, kind, e, &id);
            if !errors.is_empty() {
                problems.insert(format!("{w}/{kind}:{id}"), errors);
            }
        }
    }
    problems
}

/// WorldCatalog.validate_all: every world's problems (the world load problems are in the catalog report's "load", the
/// loader reads both in one pass).
pub fn validate_all(v: &View) -> super::Report {
    let mut problems = super::Report::new();
    for w in v.worlds.world_ids() {
        problems.extend(validate_world(v, &w));
    }
    problems
}

/// WorldCatalog.encounter_warnings: a boss fight without an enrage phase, the purse warnings and the REWARD1 key loot
/// and objective reward warnings, per own encounter.
fn encounter_warnings(v: &View, w: &str) -> Vec<String> {
    let tier_of = |r: &str| {
        let c = v.worlds.resolve(w, "characters", r);
        json::s(&content("units", v.get("units", &json::s(c, "base_unit"))), "tier")
    };
    let mut out = Vec::new();
    for e in v.worlds.own_entries(w, "encounters") {
        let id = json::s(e, "id");
        let reward = super::reward::key_loot_warnings(v, w, e).into_iter().chain(super::reward::objective_reward_warnings(e));
        // HOOKS1 AH1: arena hazards strike only under the rule arena_hazards of the world's ruleset
        let off = !json::aget(e, "arena_hazards").is_empty() && !super::hooks::lists_rule(super::hooks::world_ruleset_row(v, w), rpgkit_rules::ruleset::ARENA_HAZARDS);
        let arena = off.then(|| "Arena hazards never strike here: the world's ruleset does not list the rule arena_hazards.".to_string());
        // HOOKS1 DU1 (user 2026-10-29 (b)): an add that pushes out an automatic deck's families
        let unit_of = |r: &str| v.get("units", &json::s(v.worlds.resolve(w, "characters", r), "base_unit")).clone();
        let deck = super::hooks::deck_effect_warnings(v, e, Some(w), &unit_of);
        for warning in encounter::pace_warnings(e, &tier_of).into_iter().chain(encounter::purse_warnings(e)).chain(reward).chain(arena).chain(deck) {
            out.push(format!("{w}:{id}: {warning}"));
        }
    }
    out
}

/// WorldCatalog.world_warnings: the map, route, dialogue, gone-NPC, quest link and reward, reputation key and
/// encounter lints of one world; the character asset checks and the map prop asset checks (PROPS1 A7-A10).
pub fn world_warnings(v: &View, w: &str) -> Vec<String> {
    let mut out = map::warnings(v, w);
    out.extend(travel::warnings(v, w));
    out.extend(dialogue::dialogue_warnings(v, w));
    out.extend(dialogue::quest_warnings(v, w));
    out.extend(quest::link_warnings(v, w));
    out.extend(quest::reward_warnings(v, w));
    out.extend(quest::rep_key_warnings(v, w));
    out.extend(encounter_warnings(v, w));
    out.extend(character_asset_warnings(v, w));
    out.extend(super::assets::prop_warnings(v, w));
    out.extend(campaign::warnings(v, w));
    out.extend(story::warnings(v, w));
    out
}

/// ASSETORG2 A3-A5 for the world's own characters: unique portraits, common portraits, palettes.
fn character_asset_warnings(v: &View, w: &str) -> Vec<String> {
    let mut out = Vec::new();
    for c in v.worlds.own_entries(w, "characters") {
        let unit = v.get("units", &json::s(c, "base_unit"));
        for warning in super::assets::portrait_and_palette_warnings(v, c, unit) {
            out.push(format!("{w}:{}: {warning}", json::s(c, "id")));
        }
    }
    out
}

/// WorldCatalog.warnings_all: {world: [warnings]} for every world with one.
pub fn warnings_all(v: &View) -> super::Report {
    let mut out = super::Report::new();
    for w in v.worlds.world_ids() {
        let warnings = world_warnings(v, &w);
        if !warnings.is_empty() {
            out.insert(w, warnings);
        }
    }
    out
}
