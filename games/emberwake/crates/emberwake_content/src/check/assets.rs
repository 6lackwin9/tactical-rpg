//! The asset checks A1-A6 (ASSETORG2 section 6) over the asset layers of the checked data folder's game
//! (`View::assets`): bodies that are not there or do not load, item icons not in their folder, unique portraits and
//! common portraits missing, palettes that do not fit their body.

use super::View;
use crate::art;
use aegis_core::assets::{self, Clips};
use aegis_core::json::{self, Dict};

/// A PNG's size from its header (None: not a PNG).
fn png_size(path: &std::path::Path) -> Option<(u32, u32)> {
    let mut head = [0u8; 24];
    use std::io::Read;
    std::fs::File::open(path).ok()?.read_exact(&mut head).ok()?;
    (head[1..4] == *b"PNG").then(|| (u32::from_be_bytes([head[16], head[17], head[18], head[19]]), u32::from_be_bytes([head[20], head[21], head[22], head[23]])))
}

/// Body `key`'s clips, or why it does not load (A6); None when it is not a body at all (A1).
fn body_state(v: &View, key: &str) -> Option<Result<Clips, String>> {
    let raw = assets::read_json(&v.assets, &assets::body_file(key, "clips.json"))?;
    let clips = match Clips::parse(&raw) {
        Ok(c) => c,
        Err(p) => return Some(Err(p.join(" "))),
    };
    let Some(sheet) = v.assets.find(&assets::body_file(key, "sheet.png")) else { return Some(Err("no sheet.png.".into())) };
    let Some((w, h)) = png_size(&sheet) else { return Some(Err("sheet.png is not a PNG.".into())) };
    let problem = clips.sheet_problem(w, h);
    Some(if problem.is_empty() { Ok(clips) } else { Err(problem) })
}

/// A1 and A6 for an `art` value ("" passes: it inherits).
pub fn body_errors(v: &View, art: &str) -> Vec<String> {
    if art.is_empty() || v.assets.roots.is_empty() {
        return Vec::new();
    }
    match body_state(v, art) {
        None => vec![format!("Art '{art}' isn't a body (units/<class>/<variant>/ in the game, kit or engine).")],
        Some(Err(problem)) => vec![format!("Body '{art}': {problem}")],
        Some(Ok(_)) => Vec::new(),
    }
}

/// A2: an item's `icon_id` must be a file in its family's folder.
pub fn icon_errors(v: &View, item: &Dict, family: &Dict) -> Vec<String> {
    let icon = json::s(item, "icon_id");
    if icon.is_empty() || v.assets.roots.is_empty() {
        return Vec::new();
    }
    let folder = json::s_or(family, "art", &json::s(item, "family"));
    let folder = if folder.is_empty() { json::s(item, "family") } else { folder };
    if v.assets.has(&format!("items/{folder}/{icon}.png")) {
        Vec::new()
    } else {
        vec![format!("Icon '{icon}' isn't in items/{folder}/ (game, kit or engine).")]
    }
}

/// A3, A4 and A5 of a unit (`character` empty) or a world character on base unit `unit`.
pub fn portrait_and_palette_warnings(v: &View, character: &Dict, unit: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    if v.assets.roots.is_empty() {
        return out;
    }
    let class = v.get("classes", &json::s(unit, "class_id"));
    let body = art::body_of(character, unit, class);
    let who = if character.is_empty() { unit } else { character };
    let (id, name) = (json::s(who, "id"), json::s_or(who, "name", &json::s(who, "id")));
    let tier = art::portrait_tier(character, unit);
    let common = art::common_portrait(&body);
    let (class_id, variant) = art::split_body(&body);
    // A3: a unique tier without its file
    if let Some(folder) = art::tier_folder(&tier) {
        let own = format!("portraits/{folder}/{id}.png");
        if !v.assets.has(&own) {
            out.push(format!("{name} ({tier}) has no portrait at {own}; it shows the common portrait {class_id}_{variant}."));
        }
    } else if !v.assets.has(&common) && body_state(v, &body).is_some() {
        // A5: a body worn by a common-tier unit or character without its common portrait
        out.push(format!("Body '{body}' has no common portrait; {name} shows {class_id}_base."));
    }
    // A4: the palette file and its slots
    let palette = art::palette_of(character, unit);
    if !palette.is_empty() {
        let rel = assets::body_file(&body, &format!("palettes/{palette}.json"));
        let problem = match assets::read_json(&v.assets, &rel) {
            None => format!("no file {rel}"),
            Some(raw) => match (assets::parse_palette(&raw), body_state(v, &body)) {
                (Err(p), _) => p.join(" "),
                (Ok(slots), Some(Ok(clips))) => {
                    let missing: Vec<&String> = slots.keys().filter(|s| !clips.palette.contains_key(*s)).collect();
                    if missing.is_empty() {
                        String::new()
                    } else {
                        format!("slot '{}' is not in the body's palette", missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("', '"))
                    }
                }
                _ => String::new(),
            },
        };
        if !problem.is_empty() {
            out.push(format!("Palette '{palette}' of {name}: {}.", problem.trim_end_matches('.')));
        }
    }
    out
}
