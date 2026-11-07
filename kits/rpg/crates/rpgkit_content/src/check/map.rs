//! Map rules of the world checks (MapRules): a location's map position, zone bindings and markers against its map
//! template, the [encounter, weight] pair lists (also used by routes) and the soft location warnings. PROPS1: the
//! map template's own prop checks P1-P12 (`prop_errors`, engine-only: Godot's MapCatalog had no props).

use super::quest::flag_list_errors;
use super::world::{qid, ref_errors};
use super::View;
use crate::source::{prop_full, PROP_KEYS};
use aegis_core::json::{self, gstr, Dict};
use serde_json::Value;
use std::collections::HashMap;

const SPAWN: &str = "spawn";
const WORLD_MAP: &str = "world_map";
const MAX_MARKERS: usize = 64;
const MAX_RADIUS: i64 = 3;
const MAX_BIND_LIST: usize = 8;
const WEIGHT_MIN: i64 = 1;
const WEIGHT_MAX: i64 = 100;
const STEPS_LO: i64 = 1;
const STEPS_HI: i64 = 99;
const MAX_CHEST_KEYS: usize = 8;
const CHEST_COUNT_MAX: i64 = 99;
const CHEST_GOLD_MAX: i64 = 9999;
const CHEST_SUPPLIES_MAX: i64 = 99;
const BINDING_KEYS: [&str; 5] = ["encounters", "elites", "elite_pct", "steps_min", "steps_max"];
const MARKER_COMMON: [&str; 8] = ["id", "kind", "anchor", "requires_flags", "unless_flags", "sets_flags", "label", "note"];
const LABEL_MAX: usize = 32;
const NOTE_MAX: usize = 160;
const FLAG_KEYS: [&str; 3] = ["requires_flags", "unless_flags", "sets_flags"];
const TRIGGERS: [&str; 2] = ["touch", "proximity"];
/// NPCFACE1: the ways a person's marker may face on the map (`down`, the default, is never stored).
const FACES: [&str; 4] = ["down", "up", "left", "right"];
/// PROPS1 1: props per map, the longest footprint side, the highest explicit variant.
pub const PROPS_MAX: usize = 128;
pub const PROP_SIDE_MAX: i64 = 16;
pub const PROP_VARIANT_MAX: i64 = 15;
const PROP_LAYERS: [&str; 2] = ["sort", "ground"];
pub(crate) const PROP_ID_MAX: usize = 24;
pub(crate) const PROP_ART_MAX: usize = 32;

/// MapRules.MARKER_KINDS: kind -> its own keys with defaults.
fn marker_kind(kind: &str) -> Option<Dict> {
    let d = |pairs: &[(&str, Value)]| pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect::<Dict>();
    Some(match kind {
        "exit" => d(&[("to", Value::from("")), ("to_anchor", Value::from(SPAWN)), ("locked_text", Value::from("The way is shut."))]),
        "encounter" => d(&[("encounter", Value::from("")), ("trigger", Value::from("touch")), ("radius", json::int(1)), ("once", Value::Bool(true))]),
        "roamer" => d(&[("encounter", Value::from("")), ("path", Value::from("")), ("radius", json::int(0)), ("once", Value::Bool(true))]),
        "npc" => d(&[("character", Value::from("")), ("face", Value::from("down"))]),
        "chest" => d(&[("items", Value::Object(Dict::new())), ("loot", Value::from("")), ("gold", json::int(0)), ("supplies", json::int(0))]),
        "lore" => d(&[("lore", Value::from(""))]),
        "point" => Dict::new(),
        _ => return None,
    })
}
const MARKER_KIND_NAMES: [&str; 7] = ["exit", "encounter", "roamer", "npc", "chest", "lore", "point"];

fn int_in(v: &Value, lo: i64, hi: i64) -> bool {
    json::is_int(v) && (lo..=hi).contains(&json::to_int(v))
}

/// The zone legend's object entries.
fn legend(t: &Dict) -> Vec<&Dict> {
    json::dget(t, "zone_legend").values().filter_map(Value::as_object).collect()
}

fn zone_kind(t: &Dict, zone: &str) -> String {
    legend(t).into_iter().find(|z| json::s(z, "id") == zone).map(|z| json::s(z, "kind")).unwrap_or_default()
}

fn danger_zone_ids(t: &Dict) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for z in legend(t) {
        let id = json::s(z, "id");
        if !id.is_empty() && !out.contains(&id) && zone_kind(t, &id) == "danger" {
            out.push(id);
        }
    }
    out
}

fn has_anchor(t: &Dict, anchor: &str) -> bool {
    !anchor.is_empty() && json::aget(t, "anchors").iter().any(|a| json::s(json::dict(a), "id") == anchor)
}

fn has_path(t: &Dict, path: &str) -> bool {
    !path.is_empty() && json::dget(t, "paths").contains_key(path)
}

fn exit_targets(e: &Dict) -> Vec<String> {
    json::aget(e, "exits").iter().map(|x| json::s(json::dict(x), "to")).filter(|to| !to.is_empty()).collect()
}

/// MapRules.location_errors.
pub fn location_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut errors = map_pos_errors(e.get("map_pos").unwrap_or(&Value::Array(Vec::new())));
    let map_id = json::s(e, "map");
    let bindings_raw = e.get("zone_bindings").cloned().unwrap_or(Value::Object(Dict::new()));
    let markers_raw = e.get("markers").cloned().unwrap_or(Value::Array(Vec::new()));
    let mut types_ok = true;
    if !bindings_raw.is_object() {
        errors.push("Zone bindings must be an object {zone id: binding}.".into());
        types_ok = false;
    }
    if !markers_raw.is_array() {
        errors.push("Markers must be a list.".into());
        types_ok = false;
    }
    if map_id.is_empty() {
        if !json::dict(&bindings_raw).is_empty() || !json::arr(&markers_raw).is_empty() {
            errors.push("Zone bindings and markers need a map.".into());
        }
        return errors;
    }
    let t = v.get("maps", &map_id);
    if t.is_empty() {
        errors.push(format!("Unknown map template '{map_id}'."));
        return errors;
    }
    if !types_ok {
        return errors;
    }
    for (zone, b) in json::dict(&bindings_raw) {
        errors.extend(binding_errors(v, w, t, &map_id, zone, b));
    }
    errors.extend(markers_errors(v, w, e, t, json::arr(&markers_raw)));
    errors.extend(world_map_exit_errors(v, w, e));
    errors
}

/// `map_pos` is [] (automatic layout) or [x, y] in 0..1.
fn map_pos_errors(raw: &Value) -> Vec<String> {
    let message = || vec!["Map position must be empty or two numbers [x, y] between 0 and 1.".to_string()];
    let Some(list) = raw.as_array() else { return message() };
    if list.is_empty() {
        return Vec::new();
    }
    let ok = list.len() == 2 && list.iter().all(|x| x.as_f64().is_some_and(|f| (0.0..=1.0).contains(&f)));
    if ok {
        Vec::new()
    } else {
        message()
    }
}

/// A location with a world_map exit marker must be a node of its world's route graph.
fn world_map_exit_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let id = json::s(e, "id");
    let has_exit = json::aget(e, "markers").iter().map(json::dict).any(|m| json::s(m, "kind") == "exit" && json::s(m, "to") == WORLD_MAP);
    if has_exit && !super::travel::nodes(v, w).contains(&qid(w, &id)) {
        return vec![format!("'{id}' has a world map exit but no route.")];
    }
    Vec::new()
}

fn binding_errors(v: &View, w: &str, t: &Dict, map_id: &str, zone: &str, raw: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    match zone_kind(t, zone).as_str() {
        "" => errors.push(format!("Zone '{zone}' is not in map '{map_id}'.")),
        "safe" => errors.push(format!("Zone '{zone}' is safe: safe zones roll nothing.")),
        _ => {}
    }
    let Some(raw) = raw.as_object() else {
        errors.push(format!("Zone binding '{zone}' must be an object."));
        return errors;
    };
    for key in raw.keys().filter(|k| !BINDING_KEYS.contains(&k.as_str())) {
        errors.push(format!("Zone binding '{zone}': unknown key '{key}'."));
    }
    let mut b: Dict = [("encounters", Value::Array(Vec::new())), ("elites", Value::Array(Vec::new())), ("elite_pct", json::int(10)), ("steps_min", json::int(12)), ("steps_max", json::int(20))].into_iter().map(|(k, x)| (k.to_string(), x)).collect();
    for (k, x) in json::dict(&json::normalize(&Value::Object(raw.clone()))) {
        b.insert(k.clone(), x.clone());
    }
    let label = format!("Zone binding '{zone}'");
    for key in ["encounters", "elites"] {
        errors.extend(pair_list_errors(v, w, &label, key, &b[key]));
    }
    if !int_in(&b["elite_pct"], 0, 100) {
        errors.push(format!("Zone binding '{zone}': elite_pct must be 0..100."));
    }
    let mut steps_ok = true;
    for key in ["steps_min", "steps_max"] {
        if !int_in(&b[key], STEPS_LO, STEPS_HI) {
            errors.push(format!("Zone binding '{zone}': {key} must be {STEPS_LO}..{STEPS_HI}."));
            steps_ok = false;
        }
    }
    if steps_ok && json::to_int(&b["steps_min"]) > json::to_int(&b["steps_max"]) {
        errors.push(format!("Zone binding '{zone}': steps_min is above steps_max."));
    }
    if json::arr(&b["encounters"]).is_empty() && json::arr(&b["elites"]).is_empty() {
        errors.push(format!("Zone binding '{zone}' binds no encounter."));
    }
    errors
}

/// MapRules.pair_list_errors: an [[encounter ref, weight 1..100]...] list (zone bindings and routes).
pub fn pair_list_errors(v: &View, w: &str, label: &str, key: &str, raw: &Value) -> Vec<String> {
    let Some(list) = raw.as_array() else {
        return vec![format!("{label}: {key} must be a list of [encounter, weight] pairs.")];
    };
    let mut errors = Vec::new();
    if list.len() > MAX_BIND_LIST {
        errors.push(format!("{label}: at most {MAX_BIND_LIST} {key}."));
    }
    let mut seen: Vec<String> = Vec::new();
    for x in list {
        let pair = json::arr(x);
        if pair.len() != 2 || !pair[0].is_string() {
            errors.push(format!("{label}: {key} entries are [encounter, weight] pairs."));
            continue;
        }
        let r = gstr(&pair[0]);
        if seen.contains(&r) {
            errors.push(format!("{label}: {key} lists '{r}' twice."));
        }
        seen.push(r.clone());
        errors.extend(ref_errors(v, w, &format!("{label} {key}"), "encounters", &r));
        if !int_in(&pair[1], WEIGHT_MIN, WEIGHT_MAX) {
            errors.push(format!("{label}: {key} weight of '{r}' must be {WEIGHT_MIN}..{WEIGHT_MAX}."));
        }
    }
    errors
}

/// MapRules.marker_full: a marker with the common and kind defaults filled.
fn marker_full(m: &Dict) -> Dict {
    let kind = json::s(m, "kind");
    let mut out: Dict = [("id", Value::from("")), ("kind", Value::from("")), ("requires_flags", Value::Array(Vec::new())), ("unless_flags", Value::Array(Vec::new())), ("sets_flags", Value::Array(Vec::new())), ("label", Value::from("")), ("note", Value::from(""))]
        .into_iter()
        .map(|(k, x)| (k.to_string(), x))
        .collect();
    if kind != "roamer" {
        out.insert("anchor".into(), Value::from(""));
    }
    for (k, x) in marker_kind(&kind).unwrap_or_default() {
        out.insert(k, x);
    }
    for (k, x) in json::dict(&json::normalize(&Value::Object(m.clone()))) {
        out.insert(k.clone(), x.clone());
    }
    out
}

fn markers_errors(v: &View, w: &str, e: &Dict, t: &Dict, list: &[Value]) -> Vec<String> {
    let mut errors = Vec::new();
    if list.len() > MAX_MARKERS {
        errors.push(format!("At most {MAX_MARKERS} markers."));
    }
    let mut ids: Vec<String> = Vec::new();
    let mut taken: Vec<(String, String)> = Vec::new();
    for (i, raw) in list.iter().enumerate() {
        if !raw.is_object() {
            errors.push(format!("Marker {i} must be an object."));
            continue;
        }
        let raw = json::normalize(raw);
        let raw = json::dict(&raw);
        let mid = json::s(raw, "id");
        if !raw.get("id").is_some_and(Value::is_string) || !json::id_ok(&mid) {
            errors.push(format!("Marker {i}: id '{mid}' must be lowercase [a-z0-9_]."));
            continue;
        }
        if ids.contains(&mid) {
            errors.push(format!("Marker id '{mid}' is used twice."));
        }
        ids.push(mid.clone());
        let kind = json::s(raw, "kind");
        let Some(own) = marker_kind(&kind) else {
            errors.push(format!("Marker '{mid}': unknown kind '{kind}' (use {}).", MARKER_KIND_NAMES.join(", ")));
            continue;
        };
        for k in raw.keys() {
            if k == "anchor" && kind == "roamer" {
                errors.push(format!("Marker '{mid}': a roamer takes no anchor (it walks its path)."));
            } else if !MARKER_COMMON.contains(&k.as_str()) && !own.contains_key(k) {
                errors.push(format!("Marker '{mid}' ({kind}): unknown key '{k}'."));
            }
        }
        let m = marker_full(raw);
        if kind != "roamer" {
            let anchor = json::s(&m, "anchor");
            if !has_anchor(t, &anchor) {
                errors.push(format!("Marker '{mid}': unknown anchor '{anchor}'."));
            } else if anchor == SPAWN {
                errors.push(format!("Marker '{mid}': the '{SPAWN}' anchor holds no marker."));
            } else if let Some((_, holder)) = taken.iter().find(|(a, _)| *a == anchor) {
                errors.push(format!("Marker '{mid}': anchor '{anchor}' already holds marker '{holder}'."));
            } else {
                taken.push((anchor, mid.clone()));
            }
        }
        for fk in FLAG_KEYS {
            errors.extend(flag_list_errors(&m[fk], &format!("Marker '{mid}' {fk}"), fk));
        }
        errors.extend(marker_text_errors(&mid, &kind, &m));
        errors.extend(marker_kind_errors(v, w, e, t, &mid, &kind, &m));
    }
    errors
}

fn marker_text_errors(mid: &str, kind: &str, m: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    match m["label"].as_str() {
        None => errors.push(format!("Marker '{mid}': label must be text.")),
        Some(l) if l.chars().count() > LABEL_MAX => errors.push(format!("Marker '{mid}': label is at most {LABEL_MAX} characters.")),
        Some(_) => {}
    }
    match m["note"].as_str() {
        None => errors.push(format!("Marker '{mid}': note must be text.")),
        Some("") => {}
        Some(_) if kind != "point" => errors.push(format!("Marker '{mid}': only points take a note.")),
        Some(n) if n.chars().count() > NOTE_MAX => errors.push(format!("Marker '{mid}': note is at most {NOTE_MAX} characters.")),
        Some(_) => {}
    }
    errors
}

fn marker_kind_errors(v: &View, w: &str, e: &Dict, t: &Dict, mid: &str, kind: &str, m: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    match kind {
        "exit" => {
            let to = json::s(m, "to");
            if to.is_empty() {
                errors.push(format!("Marker '{mid}': an exit needs \"to\"."));
            } else if to != WORLD_MAP && !exit_targets(e).contains(&to) {
                errors.push(format!("Exit '{to}' is not in this location's exits."));
            }
            let to_anchor = json::s(m, "to_anchor");
            let mut target_map = json::empty();
            if !to.is_empty() && to != WORLD_MAP {
                let map_id = json::s(v.worlds.resolve(w, "locations", &to), "map");
                if !map_id.is_empty() {
                    target_map = v.get("maps", &map_id);
                }
            }
            if !target_map.is_empty() {
                if !has_anchor(target_map, &to_anchor) {
                    errors.push(format!("Marker '{mid}': to_anchor '{to_anchor}' is not an anchor of map '{}'.", json::s(target_map, "id")));
                }
            } else if to == WORLD_MAP {
                if to_anchor != SPAWN {
                    errors.push(format!("Marker '{mid}': a world_map exit always arrives at '{SPAWN}' (to_anchor must be '{SPAWN}')."));
                }
            } else if to_anchor != SPAWN {
                errors.push(format!("Marker '{mid}': '{to}' has no map, so to_anchor must be '{SPAWN}'."));
            }
            if !m["locked_text"].is_string() {
                errors.push(format!("Marker '{mid}': locked_text must be text."));
            }
        }
        "encounter" => {
            errors.extend(ref_key_errors(v, w, mid, "encounter", "encounters", m));
            if !TRIGGERS.contains(&json::s(m, "trigger").as_str()) {
                errors.push(format!("Marker '{mid}': trigger must be touch or proximity."));
            }
            errors.extend(radius_once_errors(mid, m));
        }
        "roamer" => {
            errors.extend(ref_key_errors(v, w, mid, "encounter", "encounters", m));
            if !has_path(t, &json::s(m, "path")) {
                errors.push(format!("Marker '{mid}': Unknown path '{}'.", json::s(m, "path")));
            }
            errors.extend(radius_once_errors(mid, m));
        }
        "npc" => {
            errors.extend(ref_key_errors(v, w, mid, "character", "characters", m));
            if !m["face"].as_str().is_some_and(|f| FACES.contains(&f)) {
                errors.push(format!("Marker '{mid}': face must be one of {}.", FACES.join(", ")));
            }
        }
        "lore" => errors.extend(ref_key_errors(v, w, mid, "lore", "lore", m)),
        "chest" => errors.extend(chest_errors(v, w, mid, m)),
        _ => {}
    }
    errors
}

fn ref_key_errors(v: &View, w: &str, mid: &str, key: &str, ref_kind: &str, m: &Dict) -> Vec<String> {
    match m[key].as_str().filter(|s| !s.is_empty()) {
        None => vec![format!("Marker '{mid}': choose a {key}.")],
        Some(r) => ref_errors(v, w, &format!("Marker '{mid}' {key}"), ref_kind, r),
    }
}

fn radius_once_errors(mid: &str, m: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    if !int_in(&m["radius"], 0, MAX_RADIUS) {
        errors.push(format!("Marker '{mid}': radius must be 0..{MAX_RADIUS}."));
    }
    if !m["once"].is_boolean() {
        errors.push(format!("Marker '{mid}': once must be true or false."));
    }
    errors
}

fn chest_errors(v: &View, w: &str, mid: &str, m: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let items = json::dict(&m["items"]);
    if !m["items"].is_object() {
        errors.push(format!("Marker '{mid}': items must be an object {{base item id: count}}."));
    } else if items.len() > MAX_CHEST_KEYS {
        errors.push(format!("Marker '{mid}': at most {MAX_CHEST_KEYS} item kinds."));
    }
    for (key, count) in items {
        if !v.has("items", key) {
            errors.push(format!("Marker '{mid}': unknown base item '{key}'."));
        }
        if !int_in(count, 1, CHEST_COUNT_MAX) {
            errors.push(format!("Marker '{mid}': count of '{key}' must be 1..{CHEST_COUNT_MAX}."));
        }
    }
    let loot = json::s(m, "loot");
    if !m["loot"].is_string() {
        errors.push(format!("Marker '{mid}': loot must be a loot ref."));
    } else if !loot.is_empty() {
        let found = ref_errors(v, w, &format!("Marker '{mid}' loot"), "loot", &loot);
        let ok = found.is_empty();
        errors.extend(found);
        if ok && json::s(v.worlds.resolve(w, "loot", &loot), "reward_kind") != "item" {
            errors.push("Chest loot must be an item entry.".into());
        }
    }
    if !int_in(&m["gold"], 0, CHEST_GOLD_MAX) {
        errors.push(format!("Marker '{mid}': gold must be a whole number 0..{CHEST_GOLD_MAX}."));
    }
    let supplies = m.get("supplies").cloned().unwrap_or(json::int(0));
    if !int_in(&supplies, 0, CHEST_SUPPLIES_MAX) {
        errors.push(format!("Marker '{mid}': supplies must be a whole number 0..{CHEST_SUPPLIES_MAX}."));
    }
    if items.is_empty() && loot.is_empty() && json::to_int(&m["gold"]) <= 0 && json::to_int(&supplies) <= 0 {
        errors.push(format!("Chest '{mid}' holds nothing."));
    }
    errors
}

/// MapRules.warnings: danger zones without a binding and exits without a marker, over a world's own locations.
pub fn warnings(v: &View, w: &str) -> Vec<String> {
    let mut out = Vec::new();
    for e in v.worlds.own_entries(w, "locations") {
        let t = v.get("maps", &json::s(e, "map"));
        if t.is_empty() {
            continue;
        }
        let q = json::s(e, "_qid");
        let bindings = json::dget(e, "zone_bindings");
        for zid in danger_zone_ids(t).iter().filter(|z| !bindings.contains_key(*z)) {
            out.push(format!("{q}: Danger zone '{zid}' has no binding (it never rolls)."));
        }
        let marked: Vec<String> = json::aget(e, "markers").iter().map(json::dict).filter(|m| json::s(m, "kind") == "exit").map(|m| json::s(m, "to")).collect();
        for to in exit_targets(e).iter().filter(|to| !marked.contains(to)) {
            out.push(format!("{q}: Exit to '{to}' has no marker on the map."));
        }
    }
    out
}

/// A prop's id or art name: lowercase [a-z0-9_], 2 to `max` characters.
pub(crate) fn prop_name_ok(v: Option<&Value>, max: usize) -> bool {
    v.and_then(Value::as_str).is_some_and(|s| json::id_ok(s) && (2..=max).contains(&s.chars().count()))
}

/// The template's rows as a grid of tiles when they are text rows of one non-zero width (None: no usable grid).
pub(crate) fn grid(t: &Dict) -> Option<Vec<Vec<char>>> {
    let rows = json::aget(t, "rows");
    if rows.is_empty() || !rows.iter().all(Value::is_string) {
        return None;
    }
    let g: Vec<Vec<char>> = rows.iter().map(|r| gstr(r).chars().collect()).collect();
    let w = g[0].len();
    (w > 0 && g.iter().all(|r| r.len() == w)).then_some(g)
}

/// PROPS1 2, checks P1-P12 of a map template's `props` in the order they run: the list, its size, then per prop the
/// keys, id, art, footprint numbers, footprint in the map, standing props on walls and not sharing a tile, layer,
/// variant and flip. P8-P10 need a usable grid and P7 passed; P9 and P10 only for props inside the map.
pub fn prop_errors(t: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let list = match t.get("props") {
        None => return errors,
        Some(Value::Array(l)) if l.iter().all(Value::is_object) => l,
        Some(_) => {
            errors.push("Props must be a list of objects.".into());
            return errors;
        }
    };
    if list.len() > PROPS_MAX {
        errors.push(format!("At most {PROPS_MAX} props per map (got {}).", list.len()));
    }
    let grid = grid(t);
    let mut ids: Vec<String> = Vec::new();
    // tile -> the standing prop on it
    let mut standing: HashMap<(i64, i64), String> = HashMap::new();
    for (i, raw) in list.iter().enumerate() {
        let p = prop_full(json::dict(raw));
        let id = json::s(&p, "id");
        for k in p.keys().filter(|k| !PROP_KEYS.contains(&k.as_str())) {
            errors.push(format!("Prop '{id}': unknown field '{k}'."));
        }
        if !prop_name_ok(p.get("id"), PROP_ID_MAX) {
            errors.push(format!("Prop {i}: id '{id}' must be lowercase [a-z0-9_], 2-{PROP_ID_MAX} characters."));
            continue;
        }
        if ids.contains(&id) {
            errors.push(format!("Prop id '{id}' is used twice."));
        }
        ids.push(id.clone());
        if !prop_name_ok(p.get("art"), PROP_ART_MAX) {
            errors.push(format!("Prop '{id}': art '{}' must be lowercase [a-z0-9_], 2-{PROP_ART_MAX} characters.", json::s(&p, "art")));
        }
        let whole = |k: &str| p.get(k).is_some_and(json::is_int);
        let side = |k: &str| p.get(k).is_some_and(|v| json::is_int(v) && (1..=PROP_SIDE_MAX).contains(&json::to_int(v)));
        let shape_ok = whole("x") && whole("y") && side("w") && side("h");
        if !shape_ok {
            errors.push(format!("Prop '{id}': x and y are whole numbers; w and h are 1-{PROP_SIDE_MAX}."));
        }
        let layer = json::get(&p, "layer").as_str().unwrap_or_default();
        if let (true, Some(g)) = (shape_ok, &grid) {
            let (x, y, w, h) = (json::i(&p, "x", 0), json::i(&p, "y", 0), json::i(&p, "w", 1), json::i(&p, "h", 1));
            let (mw, mh) = (g[0].len() as i64, g.len() as i64);
            if x < 0 || y < 0 || x + w > mw || y + h > mh {
                errors.push(format!("Prop '{id}': its footprint ({x}..{}, {y}..{}) leaves the map ({mw} x {mh}).", x + w - 1, y + h - 1));
            } else if layer == "sort" {
                let tiles: Vec<(i64, i64)> = (y..y + h).flat_map(|ty| (x..x + w).map(move |tx| (tx, ty))).collect();
                if let Some(&(tx, ty)) = tiles.iter().find(|(tx, ty)| g[*ty as usize][*tx as usize] != '#') {
                    errors.push(format!("Prop '{id}': a standing prop needs wall tiles under its whole footprint; ({tx}, {ty}) is '{}'.", g[ty as usize][tx as usize]));
                }
                let mut met: Vec<String> = Vec::new();
                for &(tx, ty) in &tiles {
                    match standing.get(&(tx, ty)) {
                        Some(other) if !met.contains(other) => {
                            errors.push(format!("Props '{other}' and '{id}' stand on the same tile ({tx}, {ty})."));
                            met.push(other.clone());
                        }
                        Some(_) => {}
                        None => {
                            standing.insert((tx, ty), id.clone());
                        }
                    }
                }
            }
        }
        if !PROP_LAYERS.contains(&layer) {
            errors.push(format!("Prop '{id}': layer must be sort or ground."));
        }
        let variant_ok = p.get("variant").is_some_and(|v| json::is_int(v) && (-1..=PROP_VARIANT_MAX).contains(&json::to_int(v)));
        if !variant_ok || !p.get("flip").is_some_and(Value::is_boolean) {
            errors.push(format!("Prop '{id}': variant is -1 (by position) or 0-{PROP_VARIANT_MAX}; flip is true or false."));
        }
    }
    errors
}
