//! The plan of a region map's ground picture (REGIONGROUND1, docs/specs/REGIONGROUND1_KIT.md 1.4; engine-only, no Godot
//! original): from a map template and the ground rows to what the engine's ground composer needs, as plain values
//! (the game's screen makes the composer's job from it, with the art `art::ground_dirs` finds). Pure: it reads data, and
//! no rule, save, trace or walk reads it.

use crate::check::ground::{leaves_map, within, PATCH_STRENGTH};
use crate::source::GROUND_PATCH_STRENGTH;
use aegis_core::json::{self, gstr, Dict};
use indexmap::IndexMap;
use rpgkit_data::GroundDef;
use serde_json::Value;

/// A map's large patch on purpose (KT 1.2 `ground_patches`), in cells.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedGroundPatch {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub strength: f32,
}

/// What one map's ground draws: `layers` paint low `order` first, `cells` are indices into them row by row, `blocked`
/// the `#` cells a patch or mark may not reach, `seed` one per map however it is reached.
#[derive(Debug, Clone, PartialEq)]
pub struct GroundPlan {
    pub cols: i32,
    pub rows: i32,
    pub layers: Vec<GroundDef>,
    pub cells: Vec<u8>,
    pub blocked: Vec<bool>,
    pub placed: Vec<PlacedGroundPatch>,
    pub seed: u64,
}

/// FNV-1a 64 of `text`: the seed of a map's picture (KT 1.4), so it never touches a game RNG stream.
pub fn seed_of(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3))
}

/// One placed patch as G8 takes its numbers (a missing strength is the default); None for anything else, so a bad row
/// can never draw as a phantom patch or wrap into another one.
fn placed_patch(raw: &Value, cols: i64, rows: i64) -> Option<PlacedGroundPatch> {
    let p = raw.as_object()?;
    let whole = |k: &str| p.get(k).filter(|v| json::is_int(v)).map(json::to_int);
    let (x, y, w, h) = (whole("x")?, whole("y")?, whole("w")?, whole("h")?);
    if w < 1 || h < 1 || leaves_map(x, y, w, h, cols, rows) {
        return None;
    }
    let strength = match p.get("strength") {
        None => GROUND_PATCH_STRENGTH,
        Some(v) => v.as_f64().filter(|s| within(*s, PATCH_STRENGTH.0, PATCH_STRENGTH.1))?,
    };
    Some(PlacedGroundPatch { x: x as i32, y: y as i32, w: w as i32, h: h as i32, strength: strength as f32 })
}

/// KT 1.4: None when `ground` is empty (the map draws as it always did), or when the rows cannot be planned; else the
/// grounds its cells use, sorted by `order` (a legend entry no cell uses adds no layer, so it never changes the
/// picture), the cells as indices into them, `blocked` from `#` in `rows`, the placed patches and the seed of the
/// template's id. It plans only what the checks pass: a wrong size, a character or id with no row, or a patch G8 refuses
/// for its shape or numbers gives None, the map drawing as if it had no ground. (G8's ring rule stays with the check.)
pub fn plan(template: &Dict, grounds: &IndexMap<String, GroundDef>) -> Option<GroundPlan> {
    let list = json::aget(template, "ground");
    if list.is_empty() || !list.iter().all(Value::is_string) {
        return None;
    }
    let legend = json::dget(template, "ground_legend");
    let cells_raw: Vec<Vec<char>> = list.iter().map(|r| gstr(r).chars().collect()).collect();
    let rows_raw = crate::check::map::grid(template)?;
    let (cols, rows) = (rows_raw[0].len(), rows_raw.len());
    if cells_raw.len() != rows || cells_raw.iter().any(|r| r.len() != cols) {
        return None;
    }
    let id_of = |c: &char| legend.get(&c.to_string()).map(gstr);
    let mut layers: Vec<GroundDef> = Vec::new();
    for c in cells_raw.iter().flatten() {
        let id = id_of(c)?;
        if !layers.iter().any(|l| l.id == id) {
            layers.push(grounds.get(&id)?.clone());
        }
    }
    layers.sort_by(|a, b| a.order.cmp(&b.order).then_with(|| a.id.cmp(&b.id)));
    if layers.len() > u8::MAX as usize {
        return None;
    }
    let index = |c: &char| id_of(c).and_then(|id| layers.iter().position(|l| l.id == id)).map(|i| i as u8);
    let cells: Option<Vec<u8>> = cells_raw.iter().flatten().map(index).collect();
    let blocked = rows_raw.iter().flatten().map(|c| *c == '#').collect();
    let placed: Vec<PlacedGroundPatch> = match template.get("ground_patches") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(list)) => list.iter().map(|p| placed_patch(p, cols as i64, rows as i64)).collect::<Option<_>>()?,
        Some(_) => return None,
    };
    Some(GroundPlan { cols: cols as i32, rows: rows as i32, layers, cells: cells?, blocked, placed, seed: seed_of(&json::s(template, "id")) })
}
