//! PR5-48 (MAP-LAYOUT-ED): the world map layout sidecars, `backgrounds/world_map/<name>.json` (`art::world_map_layout`):
//! `{landmarks: {location id: {pos: [x, y], scale}}, routes: {route id: [[x, y], ...]}}`, read by the world map over the
//! locations' `map_pos`. Their check (`check`, run by `aegis content check` for every sidecar whose name is a world or a
//! region in the data, `check_all`) and their writer (`write`, the `aegis content world_map_layout` op the layout
//! editor's Save calls: edits applied, checked, written in the canonical form `text`, or refused with nothing written).
//!
//! A sidecar's name is a world (`Owner::World`) or a region of a world (`Owner::Region`): its landmark ids name locations
//! and its route ids name routes of that world; a name with a background but no world or region yet is pre-staged
//! (`Owner::PreStaged`: positions and scales checked, ids not); any other name is not a sidecar (`Owner::Unknown`,
//! ignored by the check, refused by the writer).

use crate::art;
use aegis_content::world::Catalogs;
use aegis_core::assets::Layers;
use aegis_core::json::{self, Dict};
use aegis_core::vfs;
use serde_json::{Map, Value};
use std::path::PathBuf;

/// The folder the sidecars and the backgrounds sit in (under an asset layer).
pub const DIR: &str = "backgrounds/world_map";
/// The range a landmark's position and a route point's coordinates must lie in (map fractions; past 0..1 for an edge
/// anchor).
pub const POS_RANGE: (f64, f64) = (-0.25, 1.25);
/// The range a landmark's map scale must lie in.
pub const SCALE_RANGE: (f64, f64) = (0.25, 4.0);
/// The `aegis content` op that writes a sidecar (`store::RpgContent::sidecar`).
pub const OP: &str = "world_map_layout";
const TOP_KEYS: [&str; 2] = ["landmarks", "routes"];
const LANDMARK_KEYS: [&str; 2] = ["pos", "scale"];

/// What a sidecar's name is in the data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Owner {
    /// A world's id.
    World(String),
    /// A region of world `world`.
    Region { world: String },
    /// No world or region yet, but a background `<name>.png` in a layer (an act whose data has not landed).
    PreStaged,
    /// Neither.
    Unknown,
}

impl Owner {
    /// The world the ids resolve in (None: they are not checked).
    pub fn world(&self) -> Option<&str> {
        match self {
            Owner::World(w) | Owner::Region { world: w } => Some(w),
            _ => None,
        }
    }
}

/// What `name` is: a region of a world (worlds in id order, the first holding it), else a world, else pre-staged when a
/// layer has its background, else unknown.
pub fn owner(worlds: &Catalogs, layers: &Layers, name: &str) -> Owner {
    let ids = worlds.world_ids();
    if let Some(w) = ids.iter().find(|w| worlds.own_entries(w, "regions").iter().any(|r| json::s(r, "id") == name)) {
        return Owner::Region { world: w.clone() };
    }
    if worlds.has_world(name) {
        return Owner::World(name.to_string());
    }
    if layers.has(&art::world_map_background(name)) {
        return Owner::PreStaged;
    }
    Owner::Unknown
}

/// The names of the files with extension `ext` in `DIR` of every layer, unique, sorted (an id-named file only).
fn names_with(layers: &Layers, ext: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for root in &layers.roots {
        let Ok(files) = vfs::list_dir(root.join(DIR)) else { continue };
        for f in files {
            let Some(file) = f.file_name().map(|n| n.to_string_lossy().to_string()) else { continue };
            let Some(stem) = file.strip_suffix(ext) else { continue };
            if aegis_content::schema::is_id(stem) && vfs::is_file(&f) && !out.iter().any(|n| n == stem) {
                out.push(stem.to_string());
            }
        }
    }
    out.sort();
    out
}

/// Every sidecar name in the layers (`DIR/<name>.json`).
pub fn sidecar_names(layers: &Layers) -> Vec<String> {
    names_with(layers, ".json")
}

/// Every background name in the layers (`DIR/<name>.png`).
pub fn background_names(layers: &Layers) -> Vec<String> {
    names_with(layers, ".png")
}

/// A sidecar's problems: errors (they block) and warnings (they do not).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Findings {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

fn in_pos_range(v: f64) -> bool {
    v.is_finite() && (POS_RANGE.0..=POS_RANGE.1).contains(&v)
}

/// A point that is two finite numbers in `POS_RANGE`.
fn point_ok(v: &Value) -> bool {
    match v.as_array().map(Vec::as_slice) {
        Some([x, y]) => x.as_f64().is_some_and(in_pos_range) && y.as_f64().is_some_and(in_pos_range),
        _ => false,
    }
}

const POINT_RULE: &str = "must be [x, y], two numbers in -0.25..=1.25";

/// The landmark `id`'s location as seen from `world` ({} when none).
fn location<'a>(worlds: &'a Catalogs, world: &str, id: &str) -> &'a Dict {
    worlds.resolve(world, "locations", id)
}

/// Checks layout `v` of sidecar `name`: its shape (an object of `landmarks` and `routes`, no other key), every landmark's
/// `pos` and every route point (`POS_RANGE`) and `scale` (`SCALE_RANGE`, optional); with a world or region owner also
/// that each landmark names a location and each route a route of that world, and warns when a location has no landmark
/// image (`art::landmark`: the map draws its kind pin, `art::pin`). A pre-staged name skips the id checks and the
/// warnings.
pub fn check(worlds: &Catalogs, layers: &Layers, name: &str, v: &Value) -> Findings {
    let mut f = Findings::default();
    let owner = owner(worlds, layers, name);
    let world = owner.world();
    let Some(root) = v.as_object() else {
        f.errors.push("A layout is an object {landmarks, routes}.".into());
        return f;
    };
    for k in root.keys().filter(|k| !TOP_KEYS.contains(&k.as_str())) {
        f.errors.push(format!("Unknown key '{k}' (landmarks, routes)."));
    }
    match root.get("landmarks") {
        None => {}
        Some(Value::Object(marks)) => {
            for (id, e) in marks {
                let Some(e) = e.as_object() else {
                    f.errors.push(format!("Landmark '{id}' is an object {{pos, scale}}."));
                    continue;
                };
                for k in e.keys().filter(|k| !LANDMARK_KEYS.contains(&k.as_str())) {
                    f.errors.push(format!("Landmark '{id}': unknown key '{k}' (pos, scale)."));
                }
                match e.get("pos") {
                    Some(p) if point_ok(p) => {}
                    Some(p) => f.errors.push(format!("Landmark '{id}': pos {POINT_RULE} (got {p}).")),
                    None => f.errors.push(format!("Landmark '{id}' has no pos.")),
                }
                if let Some(s) = e.get("scale") {
                    if !s.as_f64().is_some_and(|s| s.is_finite() && (SCALE_RANGE.0..=SCALE_RANGE.1).contains(&s)) {
                        f.errors.push(format!("Landmark '{id}': scale must be a number in 0.25..=4.0 (got {s})."));
                    }
                }
                let Some(w) = world else { continue };
                let loc = location(worlds, w, id);
                if loc.is_empty() {
                    f.errors.push(format!("Landmark '{id}' isn't a location of world '{w}'."));
                } else if !layers.roots.is_empty() && !layers.has(&art::landmark(id)) {
                    let kind = json::s(loc, "kind");
                    let pin = if kind.is_empty() { "its kind pin".to_string() } else { format!("its pin {}", art::pin(&kind)) };
                    f.warnings.push(format!("Landmark '{id}' has no {}: the map draws {pin}.", art::landmark(id)));
                }
            }
        }
        Some(_) => f.errors.push("'landmarks' is an object {location id: {pos, scale}}.".into()),
    }
    match root.get("routes") {
        None => {}
        Some(Value::Object(routes)) => {
            for (id, pts) in routes {
                match pts.as_array() {
                    Some(list) => {
                        for (k, p) in list.iter().enumerate().filter(|(_, p)| !point_ok(p)) {
                            f.errors.push(format!("Route '{id}' point {k}: {POINT_RULE} (got {p})."));
                        }
                    }
                    None => f.errors.push(format!("Route '{id}' is a list of [x, y] points.")),
                }
                if let Some(w) = world {
                    if worlds.resolve(w, "routes", id).is_empty() {
                        f.errors.push(format!("Route '{id}' isn't a route of world '{w}'."));
                    }
                }
            }
        }
        Some(_) => f.errors.push("'routes' is an object {route id: [[x, y], ...]}.".into()),
    }
    f
}

/// The sidecar file text of `name` as the layers resolve it (the first layer holding it).
pub fn read(layers: &Layers, name: &str) -> Option<(PathBuf, String)> {
    let path = layers.find(&art::world_map_layout(name))?;
    let text = vfs::read_to_string(&path).ok()?;
    Some((path, text))
}

/// Every sidecar of the layers whose name is a world, a region or pre-staged, with its findings (a file that is not
/// JSON is one error); names neither are passed over. In name order.
pub fn check_all(worlds: &Catalogs, layers: &Layers) -> Vec<(String, Findings)> {
    let mut out = Vec::new();
    for name in sidecar_names(layers) {
        if owner(worlds, layers, &name) == Owner::Unknown {
            continue;
        }
        let Some((_, text)) = read(layers, &name) else { continue };
        let f = match serde_json::from_str::<Value>(&text) {
            Ok(v) => check(worlds, layers, &name, &v),
            Err(e) => Findings { errors: vec![format!("Not JSON: {e}.")], warnings: Vec::new() },
        };
        out.push((name, f));
    }
    out
}

/// A position kept to 4 decimals, a scale to 3 (the layout editor's precision); -0 is 0.
fn round_to(v: f64, places: i32) -> f64 {
    let k = 10f64.powi(places);
    let r = (v * k).round() / k;
    if r == 0.0 {
        0.0
    } else {
        r
    }
}

fn num_text(v: f64, places: i32) -> String {
    serde_json::Number::from_f64(round_to(v, places)).map_or_else(|| "0.0".into(), |n| n.to_string())
}

fn point_text(p: &Value) -> String {
    let xy = p.as_array().map(Vec::as_slice).unwrap_or(&[]);
    let at = |i: usize| xy.get(i).and_then(Value::as_f64).unwrap_or(0.0);
    format!("[{}, {}]", num_text(at(0), 4), num_text(at(1), 4))
}

fn key_text(k: &str) -> String {
    Value::String(k.to_string()).to_string()
}

/// The canonical text of a checked layout: two-space indent, `landmarks` then `routes`, one line per landmark
/// (`{"pos": [x, y], "scale": s}`, scale 1 when it has none) and per route, entries in the layout's own order, positions
/// to 4 decimals and scales to 3, a newline at the end.
pub fn text(v: &Value) -> String {
    let empty = Map::new();
    let section = |k: &str| v.get(k).and_then(Value::as_object).unwrap_or(&empty);
    let block = |lines: Vec<String>| if lines.is_empty() { "{}".to_string() } else { format!("{{\n{}\n  }}", lines.join(",\n")) };
    let marks: Vec<String> = section("landmarks")
        .iter()
        .map(|(id, e)| {
            let scale = e.get("scale").and_then(Value::as_f64).unwrap_or(1.0);
            format!("    {}: {{\"pos\": {}, \"scale\": {}}}", key_text(id), point_text(e.get("pos").unwrap_or(&Value::Null)), num_text(scale, 3))
        })
        .collect();
    let routes: Vec<String> = section("routes")
        .iter()
        .map(|(id, pts)| {
            let pts: Vec<String> = pts.as_array().map(|l| l.iter().map(point_text).collect()).unwrap_or_default();
            format!("    {}: [{}]", key_text(id), pts.join(", "))
        })
        .collect();
    format!("{{\n  \"landmarks\": {},\n  \"routes\": {}\n}}\n", block(marks), block(routes))
}

/// Applies `edits` to layout `v` (an object; `landmarks` and `routes` made when missing): each edit is `{id, pos?,
/// scale?}` (a landmark's values; a new landmark needs `pos` and takes scale 1) or `{route, points}` (a route's points,
/// replaced). `edits` is one edit or a list (the layout editor's `LayoutEdit::to_json` each). The values are checked
/// after (`check`); here only the edits' own form.
pub fn apply(v: &mut Value, edits: &Value) -> Result<usize, Vec<String>> {
    let list: Vec<&Value> = match edits {
        Value::Array(a) => a.iter().collect(),
        Value::Object(_) => vec![edits],
        _ => return Err(vec!["The edits are a list of {id, pos, scale} and {route, points}.".into()]),
    };
    let Some(root) = v.as_object_mut() else { return Err(vec!["A layout is an object {landmarks, routes}.".into()]) };
    for k in TOP_KEYS {
        if !root.contains_key(k) {
            root.insert(k.into(), Value::Object(Map::new()));
        }
    }
    let mut errors = Vec::new();
    for (n, e) in list.iter().enumerate() {
        let n = n + 1;
        let Some(e) = e.as_object() else {
            errors.push(format!("Edit {n} is an object {{id, pos, scale}} or {{route, points}}."));
            continue;
        };
        let id = e.get("id").and_then(Value::as_str);
        let route = e.get("route").and_then(Value::as_str);
        let allowed: &[&str] = if route.is_some() { &["route", "points"] } else { &["id", "pos", "scale"] };
        if let Some(k) = e.keys().find(|k| !allowed.contains(&k.as_str())) {
            errors.push(format!("Edit {n}: unknown key '{k}' ({}).", allowed.join(", ")));
            continue;
        }
        match (id, route) {
            (Some(id), None) => {
                let Some(Value::Object(marks)) = root.get_mut("landmarks") else {
                    errors.push("'landmarks' is an object {location id: {pos, scale}}.".into());
                    continue;
                };
                let (pos, scale) = (e.get("pos").filter(|p| !p.is_null()), e.get("scale").filter(|s| !s.is_null()));
                if pos.is_none() && scale.is_none() {
                    errors.push(format!("Edit {n}: landmark '{id}' needs pos and / or scale."));
                    continue;
                }
                let entry = match marks.get_mut(id) {
                    Some(Value::Object(o)) => o,
                    Some(_) => {
                        errors.push(format!("Landmark '{id}' is an object {{pos, scale}}."));
                        continue;
                    }
                    None => {
                        if pos.is_none() {
                            errors.push(format!("Edit {n}: the new landmark '{id}' needs a pos."));
                            continue;
                        }
                        marks.insert(id.into(), serde_json::json!({"pos": Value::Null, "scale": 1.0}));
                        marks.get_mut(id).and_then(Value::as_object_mut).expect("just inserted")
                    }
                };
                if let Some(p) = pos {
                    entry.insert("pos".into(), p.clone());
                }
                if let Some(s) = scale {
                    entry.insert("scale".into(), s.clone());
                }
            }
            (None, Some(route)) => {
                let Some(Value::Object(routes)) = root.get_mut("routes") else {
                    errors.push("'routes' is an object {route id: [[x, y], ...]}.".into());
                    continue;
                };
                let Some(points) = e.get("points") else {
                    errors.push(format!("Edit {n}: route '{route}' needs points [[x, y], ...]."));
                    continue;
                };
                routes.insert(route.into(), points.clone());
            }
            _ => errors.push(format!("Edit {n} needs id (a landmark) or route (a route), not both.")),
        }
    }
    if errors.is_empty() {
        Ok(list.len())
    } else {
        Err(errors)
    }
}

/// What a write did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Written {
    pub path: PathBuf,
    pub edits: usize,
    /// The text differs from the file's before.
    pub changed: bool,
}

/// The `aegis content world_map_layout` writer: applies `edits` (`apply`) to sidecar `name` as the layers resolve it
/// (an empty layout when none has it), checks the result (`check`), and writes its canonical text (`text`) into the first
/// layer (the game's own assets). Refused with nothing written when the name is no world, region or pre-staged
/// background, the file there is not JSON, an edit is malformed, or the result has an error.
pub fn write(worlds: &Catalogs, layers: &Layers, name: &str, edits: &Value) -> Result<Written, Vec<String>> {
    if !aegis_content::schema::is_id(name) {
        return Err(vec![format!("'{name}' isn't a sidecar name (lowercase snake_case).")]);
    }
    let Some(root) = layers.roots.first() else { return Err(vec!["No asset layer to write into.".into()]) };
    if owner(worlds, layers, name) == Owner::Unknown {
        return Err(vec![format!("No world or region '{name}' in the data and no {}: nothing to lay out.", art::world_map_background(name))]);
    }
    let rel = art::world_map_layout(name);
    let before = read(layers, name);
    let mut v = match &before {
        Some((path, text)) => serde_json::from_str::<Value>(text).map_err(|e| vec![format!("{}: not JSON: {e}.", path.display())])?,
        None => serde_json::json!({}),
    };
    let edits = apply(&mut v, edits)?;
    let found = check(worlds, layers, name, &v);
    if !found.errors.is_empty() {
        return Err(found.errors.into_iter().map(|e| format!("{rel}: {e}")).collect());
    }
    let out = text(&v);
    let path = root.join(&rel);
    let own_before = vfs::read_to_string(&path).ok();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| vec![format!("{}: {e}", dir.display())])?;
    }
    std::fs::write(&path, &out).map_err(|e| vec![format!("{}: {e}", path.display())])?;
    Ok(Written { changed: own_before.as_deref() != Some(out.as_str()), path, edits })
}
