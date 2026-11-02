//! Region map grounds (REGIONGROUND1, docs/specs/REGIONGROUND1_KIT.md 1.1-1.3; engine-only, no Godot original): the
//! checks of one row of the kind `grounds` and across rows (unique ids and orders); the template checks G1-G8 of a map's
//! `ground`, `ground_legend` and `ground_patches`; G9 of its `wall_storeys`; G10 of a world's `material_family`
//! (REGIONGROUND1_SCENE, decisions "Wall storeys and walls beside props", "Two kinds of wall"). Presentation only: none
//! of these keys reaches walking, a rule, a save or a trace. The numbers' ranges live in these checks (the schema rows
//! carry no `min` / `max`, so normalizing never clamps): a value out of range is refused with its reason.

use super::map::{grid, prop_name_ok, PROP_ART_MAX, PROP_ID_MAX};
use super::View;
use crate::source::{fields, GROUND_PATCH_KEYS};
use aegis_core::assets::hex_colour;
use aegis_core::json::{self, gstr, Dict};
use serde_json::Value;

/// The catalog kind (file data/catalog/grounds.json; a missing file is an empty table).
pub const KIND: &str = rpgkit_data::GROUNDS_KIND;
/// KT 1.1: the id's length range, the name's and the paint order's limits.
pub const ID_MIN: usize = 2;
pub const ID_MAX: usize = 24;
pub const NAME_MAX: usize = 32;
pub const ORDER_MAX: i64 = 999;
/// KT 1.1: the number ranges, all from 0 (key, label, largest).
pub const NUMBER_MAX: [(&str, &str, f64); 3] = [("wear", "Wear", 0.3), ("tone", "Tone", 0.2), ("patches", "Patches", 0.3)];
pub const MARKS_MAX: usize = 4;
pub const MARKS_PER_100_MAX: i64 = 10;
/// G6, G8: the legend's and the placed patches' list limits.
pub const GROUND_LEGEND_MAX: usize = 12;
pub const GROUND_PATCHES_MAX: usize = 16;
/// G8: a placed patch's strength.
pub const PATCH_STRENGTH: (f64, f64) = (0.02, 0.25);
/// G9 (the scene's wall storeys; a wall is 1 storey unless a rect says more): the largest height and the rects per map.
pub const WALL_STOREYS_MAX: i64 = 4;
pub const WALL_STOREY_RECTS_MAX: usize = 64;
/// G10 (the scene's material family): "a short palette and material list".
pub const FAMILY_LIST_MAX: usize = 8;
pub const FAMILY_MATERIAL_MAX: usize = 32;
/// The tile that decides nothing about a ground: a wall takes any (G7) and blocks patches (G8).
const WALL: char = '#';
const WATER_CELL: char = '~';
const FLOOR_CELL: char = '.';

fn chars(s: &str) -> usize {
    s.chars().count()
}

/// A stored float is snapped to 4 places (normalize), which may land a hair past a range's end.
pub(crate) fn within(x: f64, lo: f64, hi: f64) -> bool {
    x >= lo - 1e-9 && x <= hi + 1e-9
}

/// `[a-z0-9_]`, `ID_MIN`-`ID_MAX` characters.
fn id_ok(id: &str) -> bool {
    json::id_ok(id) && (ID_MIN..=ID_MAX).contains(&chars(id))
}

/// Whether a rect leaves a `mw` x `mh` map; `w` and `h` are at least 1. No sum is taken, so no number overflows it.
pub(crate) fn leaves_map(x: i64, y: i64, w: i64, h: i64, mw: i64, mh: i64) -> bool {
    x < 0 || y < 0 || w > mw || h > mh || x > mw - w || y > mh - h
}

fn is_water(row: &Dict) -> bool {
    json::s(row, "kind") == rpgkit_data::GROUND_WATER
}

// ---- The rows (KT 1.1) -------------------------------------------------------------------------------------------

/// The problems of each row (same order as the rows), on its own and against the others (unique ids and orders).
pub fn check_rows(rows: &[Dict]) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = rows.iter().map(row_errors).collect();
    for (i, found) in out.iter_mut().enumerate() {
        let id = json::s(&rows[i], "id");
        if !id.is_empty() && rows[..i].iter().any(|r| json::s(r, "id") == id) {
            found.push(format!("ID '{id}' is already used by another ground."));
        }
        let order = json::i(&rows[i], "order", -1);
        if let Some(other) = rows[..i].iter().find(|r| json::i(r, "order", -1) == order && (0..=ORDER_MAX).contains(&order)) {
            found.push(format!("Order {order} is already used by ground '{}'.", json::s(other, "id")));
        }
    }
    out
}

/// The own problems of one (normalized) row.
pub fn row_errors(e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let known: Vec<String> = fields("content", KIND).iter().map(|f| json::s(json::dict(f), "key")).collect();
    for k in e.keys().filter(|k| !known.contains(k)) {
        out.push(format!("Unknown field '{k}'."));
    }
    let id = json::s(e, "id");
    if id.is_empty() {
        out.push("ID is required.".into());
    } else if !id_ok(&id) {
        out.push(format!("ID '{id}' must be {ID_MIN}-{ID_MAX} characters of a-z, 0-9 and _."));
    }
    let name = json::s(e, "name");
    if name.trim().is_empty() {
        out.push("Name is required.".into());
    } else if chars(&name) > NAME_MAX {
        out.push(format!("Name is {} characters (max {NAME_MAX}).", chars(&name)));
    }
    let order = json::get(e, "order");
    if !json::is_int(order) || !(0..=ORDER_MAX).contains(&json::to_int(order)) {
        out.push(format!("Order must be a whole number 0-{ORDER_MAX}."));
    }
    let kind = json::s(e, "kind");
    let kinds: Vec<String> = fields("content", KIND).iter().map(json::dict).find(|f| json::s(f, "key") == "kind").map(|f| json::aget(f, "options").iter().map(gstr).collect()).unwrap_or_default();
    if !kinds.contains(&kind) {
        out.push(format!("Kind: '{kind}' is not one of {}.", kinds.join(", ")));
    }
    let colour = json::s(e, "colour");
    if hex_colour(&colour).is_none() {
        out.push(format!("Colour '{colour}' must be #rrggbb."));
    }
    for (key, label, hi) in NUMBER_MAX {
        let x = json::f(e, key, 0.0);
        if !within(x, 0.0, hi) {
            out.push(format!("{label} must be 0-{hi}."));
        }
    }
    let marks = json::sget(e, "marks");
    if marks.len() > MARKS_MAX {
        out.push(format!("At most {MARKS_MAX} marks (got {}).", marks.len()));
    }
    for m in &marks {
        if !prop_name_ok(Some(&Value::from(m.as_str())), PROP_ART_MAX) {
            out.push(format!("Mark '{m}' must be a prop art id: lowercase [a-z0-9_], 2-{PROP_ART_MAX} characters."));
        }
    }
    let per = json::i(e, "marks_per_100", 0);
    if !(0..=MARKS_PER_100_MAX).contains(&per) {
        out.push(format!("Marks per 100 cells must be a whole number 0-{MARKS_PER_100_MAX}."));
    }
    let origin = json::s(e, "origin");
    let origins: Vec<String> = fields("content", KIND).iter().map(json::dict).find(|f| json::s(f, "key") == "origin").map(|f| json::aget(f, "options").iter().map(gstr).collect()).unwrap_or_default();
    if !origins.contains(&origin) {
        out.push(format!("Origin: '{origin}' is not one of {}.", origins.join(", ")));
    }
    out
}

/// The Forge's check of one entry (normalized) as entry `current_id` ("" = a new entry): its own problems and those
/// against the loaded rows (its old row taken out); the entry is checked last.
pub fn entry_errors(v: &View, e: &Dict, current_id: &str) -> Vec<String> {
    let mut rows: Vec<Dict> = v.entries(KIND).iter().filter(|r| current_id.is_empty() || json::s(r, "id") != current_id).cloned().collect();
    rows.push(e.clone());
    check_rows(&rows).pop().unwrap_or_default()
}

// ---- The template (KT 1.2, 1.3) ----------------------------------------------------------------------------------

/// The `ground` rows as a grid when it is one string per row of one width (None: G2 names it).
fn ground_grid(list: &[Value]) -> Option<Vec<Vec<char>>> {
    list.iter().all(Value::is_string).then(|| list.iter().map(|r| gstr(r).chars().collect()).collect())
}

/// G2: the ground's size as a message, when it is not the map's. A row of another width gives the width named (a row
/// that is not text counts as 0), so the two sizes in the message always differ.
fn size_error(list: &[Value], map: &[Vec<char>]) -> Option<String> {
    let (w2, h2) = (map[0].len(), map.len());
    let widths: Vec<usize> = list.iter().map(|r| r.as_str().map_or(0, chars)).collect();
    let w = widths.iter().copied().find(|w| *w != w2).unwrap_or(w2);
    (w != w2 || list.len() != h2 || !list.iter().all(Value::is_string)).then(|| format!("Ground has {w} x {} cells; the map has {w2} x {h2}.", list.len()))
}

/// G1-G8 and G9 of a map template, in that order: the shapes, the legend, the cells, the patches, the wall storeys.
/// The cell checks and the patches' ground need G2 passed; a map without a usable `rows` grid skips them.
pub fn map_errors(v: &View, t: &Dict) -> Vec<String> {
    let mut errors = ground_errors(v, t);
    errors.extend(wall_storey_errors(t));
    errors
}

fn ground_errors(v: &View, t: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let ground = json::get(t, "ground");
    let legend_raw = json::get(t, "ground_legend");
    let patches_raw = json::get(t, "ground_patches");
    let ground_list: &[Value] = ground.as_array().map_or(&[], Vec::as_slice);
    if !ground.is_null() && !ground.is_array() {
        errors.push("Ground must be a list of rows.".into());
    }
    if !legend_raw.is_null() && !legend_raw.is_object() {
        errors.push("Ground legend must be an object.".into());
    }
    let patch_list: &[Value] = patches_raw.as_array().map_or(&[], Vec::as_slice);
    if (!patches_raw.is_null() && !patches_raw.is_array()) || !patch_list.iter().all(Value::is_object) {
        errors.push("Ground patches must be a list of objects.".into());
    }
    let legend = json::dget(t, "ground_legend");
    if ground_list.is_empty() {
        // G1
        if !legend.is_empty() || !patch_list.is_empty() {
            errors.push("Ground legend and patches need ground rows.".into());
        }
        return errors;
    }
    let map = grid(t);
    // G2
    let sized = match &map {
        Some(m) => match size_error(ground_list, m) {
            Some(problem) => {
                errors.push(problem);
                false
            }
            None => true,
        },
        None => false,
    };
    // G3, G5, G6
    for (k, id) in legend {
        let mut key = k.chars();
        let one = key.next().is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) && key.next().is_none();
        if !one {
            errors.push(format!("Ground legend key '{k}' must be one lowercase letter or digit."));
        }
        if !v.has(KIND, &gstr(id)) {
            errors.push(format!("Ground legend '{k}': no ground '{}'.", gstr(id)));
        }
    }
    if legend.len() > GROUND_LEGEND_MAX {
        errors.push(format!("At most {GROUND_LEGEND_MAX} grounds per map (got {}).", legend.len()));
    }
    if patch_list.len() > GROUND_PATCHES_MAX {
        errors.push(format!("At most {GROUND_PATCHES_MAX} ground patches per map (got {}).", patch_list.len()));
    }
    let rows = map.as_ref().filter(|_| sized);
    let cells = rows.and_then(|_| ground_grid(ground_list));
    if let (Some(rows), Some(cells)) = (rows, &cells) {
        cell_errors(v, legend, rows, cells, &mut errors);
    }
    let usable = rows.zip(cells.as_ref());
    patch_errors(v, legend, patch_list, usable, &mut errors);
    errors
}

/// G4 and G7, one line per wrong character and per wrong (ground, tile) pair at its first cell, so a map of one wrong
/// letter gives one line and not a thousand.
fn cell_errors(v: &View, legend: &Dict, rows: &[Vec<char>], cells: &[Vec<char>], errors: &mut Vec<String>) {
    let mut named: Vec<char> = Vec::new();
    let mut pairs: Vec<(String, char)> = Vec::new();
    for (y, row) in cells.iter().enumerate() {
        for (x, c) in row.iter().enumerate() {
            let Some(id) = legend.get(&c.to_string()).map(gstr) else {
                if !named.contains(c) {
                    named.push(*c);
                    errors.push(format!("Ground ({x}, {y}): '{c}' is not in the legend."));
                }
                continue;
            };
            let tile = rows[y][x];
            if !v.has(KIND, &id) || (tile != FLOOR_CELL && tile != WATER_CELL) {
                continue;
            }
            let water = is_water(v.get(KIND, &id));
            if ((water && tile == FLOOR_CELL) || (!water && tile == WATER_CELL)) && !pairs.iter().any(|(i, t)| *i == id && *t == tile) {
                pairs.push((id.clone(), tile));
                errors.push(format!("Ground ({x}, {y}): '{id}' on a '{tile}' cell."));
            }
        }
    }
}

/// G8 of each placed patch.
fn patch_errors(v: &View, legend: &Dict, list: &[Value], usable: Option<(&Vec<Vec<char>>, &Vec<Vec<char>>)>, errors: &mut Vec<String>) {
    let mut ids: Vec<String> = Vec::new();
    for raw in list.iter().filter(|p| p.is_object()) {
        let p = crate::source::ground_patch_full(json::dict(raw));
        let id = json::s(&p, "id");
        let mut push = |reason: String| errors.push(format!("Ground patch '{id}': {reason}."));
        for k in p.keys().filter(|k| !GROUND_PATCH_KEYS.contains(&k.as_str())) {
            push(format!("unknown field '{k}'"));
        }
        if !prop_name_ok(p.get("id"), PROP_ID_MAX) {
            push(format!("its id must be lowercase [a-z0-9_], 2-{PROP_ID_MAX} characters"));
        } else if ids.contains(&id) {
            push("its id is used twice".into());
        }
        ids.push(id.clone());
        let strength = json::get(&p, "strength");
        if !strength.is_number() || !within(json::f(&p, "strength", 0.0), PATCH_STRENGTH.0, PATCH_STRENGTH.1) {
            push(format!("strength must be {}-{}", PATCH_STRENGTH.0, PATCH_STRENGTH.1));
        }
        let whole = |k: &str| p.get(k).is_some_and(json::is_int);
        if !(whole("x") && whole("y") && whole("w") && whole("h")) || json::i(&p, "w", 0) < 1 || json::i(&p, "h", 0) < 1 {
            push("x, y, w and h must be whole numbers, w and h at least 1".into());
            continue;
        }
        let Some((rows, cells)) = usable else { continue };
        let (x, y, w, h) = (json::i(&p, "x", 0), json::i(&p, "y", 0), json::i(&p, "w", 1), json::i(&p, "h", 1));
        let (mw, mh) = (rows[0].len() as i64, rows.len() as i64);
        if leaves_map(x, y, w, h, mw, mh) {
            push(format!("it leaves the map ({mw} x {mh})"));
            continue;
        }
        // the rect and a ring of one cell (cut off at the map's edge): one land ground, no wall
        let mut first: Option<String> = None;
        'ring: for ty in (y - 1).max(0)..(y + h + 1).min(mh) {
            for tx in (x - 1).max(0)..(x + w + 1).min(mw) {
                if rows[ty as usize][tx as usize] == WALL {
                    push(format!("({tx}, {ty}) is a wall"));
                    break 'ring;
                }
                let Some(ground) = legend.get(&cells[ty as usize][tx as usize].to_string()).map(gstr) else { continue };
                if is_water(v.get(KIND, &ground)) {
                    push(format!("({tx}, {ty}) is water"));
                    break 'ring;
                }
                match &first {
                    None => first = Some(ground),
                    Some(one) if *one != ground => {
                        push(format!("its rect and a ring of one cell hold both '{one}' and '{ground}'"));
                        break 'ring;
                    }
                    Some(_) => {}
                }
            }
        }
    }
}

/// KT 1.3: the legend entries no cell uses, one warning each (never blocking). Skipped when the rows are no grid.
pub fn legend_warnings(t: &Dict) -> Vec<String> {
    let list = json::aget(t, "ground");
    let Some(cells) = (!list.is_empty()).then(|| ground_grid(list)).flatten() else { return Vec::new() };
    json::dget(t, "ground_legend")
        .iter()
        .filter(|(k, _)| !cells.iter().flatten().any(|c| c.to_string() == **k))
        .map(|(k, id)| format!("Ground legend '{k}': no cell uses '{}'.", gstr(id)))
        .collect()
}

// ---- Wall storeys (G9; the scene's "Storeys and how walls sit with the props") ---------------------------------------

/// A `wall_storeys` rect G9 would take as far as its numbers go: x, y, w, h and storeys whole, w and h at least 1,
/// storeys in range.
fn storey_rect_ok(r: &Dict) -> bool {
    let whole = |k: &str| r.get(k).is_some_and(json::is_int);
    whole("x") && whole("y") && whole("w") && whole("h") && whole("storeys") && json::i(r, "w", 0) >= 1 && json::i(r, "h", 0) >= 1 && (2..=WALL_STOREYS_MAX).contains(&json::i(r, "storeys", 0))
}

/// The storeys of the wall cell (x, y) by the template's `wall_storeys` rects; 1 when none holds it. A rect G9 refuses
/// for its numbers counts for nothing, so the answer is always 1-`WALL_STOREYS_MAX`, whatever the row says.
pub fn storeys_at(t: &Dict, x: i64, y: i64) -> i64 {
    json::aget(t, "wall_storeys")
        .iter()
        .map(json::dict)
        .filter(|r| storey_rect_ok(r))
        .find(|r| {
            let (rx, ry) = (json::i(r, "x", 0), json::i(r, "y", 0));
            x >= rx && y >= ry && x < rx.saturating_add(json::i(r, "w", 1)) && y < ry.saturating_add(json::i(r, "h", 1))
        })
        .map_or(1, |r| json::i(r, "storeys", 1))
}

/// G9: each rect `{x, y, w, h, storeys}` in the map, on wall cells only, off the south row and the side columns (those
/// stay short so nothing hides behind them), with no overlap and 2-`WALL_STOREYS_MAX` storeys (a wall is 1 by default).
fn wall_storey_errors(t: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let raw = json::get(t, "wall_storeys");
    if raw.is_null() {
        return errors;
    }
    let Some(list) = raw.as_array().filter(|l| l.iter().all(Value::is_object)) else {
        errors.push("Wall storeys must be a list of objects.".into());
        return errors;
    };
    if list.len() > WALL_STOREY_RECTS_MAX {
        errors.push(format!("At most {WALL_STOREY_RECTS_MAX} wall storey rects per map (got {}).", list.len()));
    }
    let map = grid(t);
    let mut taken: Vec<(i64, i64)> = Vec::new();
    for (i, item) in list.iter().enumerate() {
        let r = json::dict(item);
        let mut push = |reason: String| errors.push(format!("Wall storeys {i}: {reason}."));
        for k in r.keys().filter(|k| !["x", "y", "w", "h", "storeys"].contains(&k.as_str())) {
            push(format!("unknown field '{k}'"));
        }
        let whole = |k: &str| r.get(k).is_some_and(json::is_int);
        if !whole("storeys") || !(2..=WALL_STOREYS_MAX).contains(&json::i(r, "storeys", 0)) {
            push(format!("storeys must be a whole number 2-{WALL_STOREYS_MAX} (a wall is 1 by default)"));
        }
        if !(whole("x") && whole("y") && whole("w") && whole("h")) || json::i(r, "w", 0) < 1 || json::i(r, "h", 0) < 1 {
            push("x, y, w and h must be whole numbers, w and h at least 1".into());
            continue;
        }
        let Some(rows) = &map else { continue };
        let (x, y, w, h) = (json::i(r, "x", 0), json::i(r, "y", 0), json::i(r, "w", 1), json::i(r, "h", 1));
        let (mw, mh) = (rows[0].len() as i64, rows.len() as i64);
        if leaves_map(x, y, w, h, mw, mh) {
            push(format!("it leaves the map ({mw} x {mh})"));
            continue;
        }
        if x == 0 || x + w == mw || y + h == mh {
            push("the south row and the side columns stay short".into());
        }
        for ty in y..y + h {
            for tx in x..x + w {
                if rows[ty as usize][tx as usize] != WALL {
                    push(format!("({tx}, {ty}) is not a wall"));
                } else if taken.contains(&(tx, ty)) {
                    push(format!("({tx}, {ty}) is in another rect"));
                }
                taken.push((tx, ty));
            }
        }
    }
    errors
}

// ---- Material family (G10; the scene's "Two kinds of wall") -----------------------------------------------------------

/// G10: a world manifest's `material_family`: `{}` (not set), or `palette` and `materials`, each 1-`FAMILY_LIST_MAX`
/// entries: `#rrggbb` colours and names of at most `FAMILY_MATERIAL_MAX` characters.
pub fn family_errors(m: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let family = match m.get("material_family") {
        None => return errors,
        Some(Value::Object(o)) => o,
        Some(_) => {
            errors.push("Material family must be an object.".into());
            return errors;
        }
    };
    if family.is_empty() {
        return errors;
    }
    for k in family.keys().filter(|k| !["palette", "materials"].contains(&k.as_str())) {
        errors.push(format!("Material family: unknown field '{k}'."));
    }
    let list = |key: &str| family.get(key).and_then(Value::as_array).filter(|l| (1..=FAMILY_LIST_MAX).contains(&l.len()) && l.iter().all(Value::is_string));
    match list("palette") {
        None => errors.push(format!("Material family: palette must list 1-{FAMILY_LIST_MAX} colours.")),
        Some(colours) => {
            for c in colours.iter().map(gstr).filter(|c| hex_colour(c).is_none()) {
                errors.push(format!("Material family: colour '{c}' must be #rrggbb."));
            }
        }
    }
    match list("materials") {
        None => errors.push(format!("Material family: materials must list 1-{FAMILY_LIST_MAX} names.")),
        Some(names) => {
            for n in names.iter().map(gstr).filter(|n| n.trim().is_empty() || chars(n) > FAMILY_MATERIAL_MAX) {
                errors.push(format!("Material family: material '{n}' must be 1-{FAMILY_MATERIAL_MAX} characters."));
            }
        }
    }
    errors
}
