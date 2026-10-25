//! The asset checks A1-A6 (ASSETORG2 section 6) over the asset layers of the checked data folder's game
//! (`View::assets`): bodies that are not there or do not load, item icons not in their folder, unique portraits and
//! common portraits missing, palettes that do not fit their body; PROPS1 3.3: the map prop checks A7-A10 (art missing
//! along the world's chain, a PNG of the wrong size, an explicit variant past the files, art hiding an anchor or path).

use super::View;
use crate::art;
use aegis_core::assets::{self, Clips};
use aegis_core::json::{self, Dict};
use serde_json::Value;

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

/// A prop's footprint (x, y, w, h) when its numbers are well formed (P7), else None (the errors name it).
fn prop_footprint(p: &Dict) -> Option<(i64, i64, i64, i64)> {
    let int = |k: &str| p.get(k).filter(|v| json::is_int(v)).map(json::to_int);
    let (x, y, w, h) = (int("x")?, int("y")?, int("w")?, int("h")?);
    ((1..=super::map::PROP_SIDE_MAX).contains(&w) && (1..=super::map::PROP_SIDE_MAX).contains(&h)).then_some((x, y, w, h))
}

/// The wanted pixel size of a w x h prop on `layer` as text ("64-96 x 64-192 px", "64 x 64 px").
fn range_text(w: i64, h: i64, layer: &str) -> String {
    let ((w0, w1), (h0, h1)) = art::prop_png_range(w, h, layer);
    let side = |a: i64, b: i64| if a == b { a.to_string() } else { format!("{a}-{b}") };
    format!("{} x {} px", side(w0, w1), side(h0, h1))
}

/// Whether the drawn rect of a standing prop (PROPS1 4.3: `pw` x `ph` art px at 32 per tile, bottom on the
/// footprint's bottom edge, centred across it) covers the centre of tile (tx, ty). In half art px, so the centring
/// stays exact.
fn covers(fp: (i64, i64, i64, i64), size: (i64, i64), tx: i64, ty: i64) -> bool {
    let ((x, y, w, h), (pw, ph)) = (fp, size);
    let (left, bottom) = (64 * x + 32 * w - pw, 64 * (y + h));
    let (cx, cy) = (64 * tx + 32, 64 * ty + 32);
    cx >= left && cx < left + 2 * pw && cy >= bottom - 2 * ph && cy < bottom
}

/// A7-A10 (PROPS1 3.3) of world `w`: the props of every map its locations use (its own and inherited ones), looked up
/// along its chain (`art::prop_dirs`), one line per finding. Warnings, never blocking; skipped without asset layers.
pub fn prop_warnings(v: &View, w: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if v.assets.roots.is_empty() {
        return out;
    }
    let mut lineage = v.worlds.lineage(w);
    lineage.reverse();
    let mut maps: Vec<String> = Vec::new();
    for e in v.worlds.merged_entries(w, "locations") {
        let m = json::s(e, "map");
        if !m.is_empty() && !maps.contains(&m) {
            maps.push(m);
        }
    }
    let mut push = |line: String| {
        if !out.contains(&line) {
            out.push(line);
        }
    };
    for map_id in maps {
        let t = v.get("maps", &map_id);
        let Some(Value::Array(props)) = t.get("props") else { continue };
        for p in props.iter().filter_map(Value::as_object) {
            let (id, art_name, layer) = (json::s(p, "id"), json::s(p, "art"), json::s(p, "layer"));
            let Some(fp) = prop_footprint(p) else { continue };
            if art_name.is_empty() {
                continue;
            }
            let dirs = art::prop_dirs(&lineage, &art_name);
            let Some((dir, n)) = dirs.iter().map(|d| (d.clone(), v.assets.variants(d) as i64)).find(|(_, n)| *n > 0) else {
                // A7; A10 with the footprint alone never fires (a standing footprint lies on walls, P9)
                let sets = art::region_sets(&lineage).join(",");
                push(format!("Map '{map_id}' prop art '{art_name}': no file props/{{{sets}}}/{art_name}/v0.png for world '{w}'; it draws its shape."));
                continue;
            };
            let size_of = |k: i64| v.assets.find(&format!("{dir}/v{k}.png")).and_then(|path| png_size(&path)).map(|(a, b)| (a as i64, b as i64));
            // A8: every file of the run
            let ((w0, w1), (h0, h1)) = art::prop_png_range(fp.2, fp.3, &layer);
            for k in 0..n {
                if let Some((pw, ph)) = size_of(k) {
                    if pw < w0 || pw > w1 || ph < h0 || ph > h1 {
                        push(format!("Prop art '{dir}/v{k}.png' is {pw}x{ph} px; a {} x {} {layer} prop wants {}.", fp.2, fp.3, range_text(fp.2, fp.3, &layer)));
                    }
                }
            }
            // A9
            let variant = json::i(p, "variant", -1);
            if variant >= n {
                push(format!("Map '{map_id}' prop '{id}': variant {variant}, but '{art_name}' has {n} files; it shows v0."));
            }
            // A10: the file it shows (3.2), its size or the footprint's
            if layer != "sort" {
                continue;
            }
            let pick = match variant {
                v if v >= n => 0,
                v if v >= 0 => v,
                _ => (fp.0 * 7 + fp.1 * 13).rem_euclid(n),
            };
            let size = size_of(pick).unwrap_or((32 * fp.2, 32 * fp.3));
            for a in json::aget(t, "anchors").iter().map(json::dict) {
                let (ax, ay) = (json::i(a, "x", -1), json::i(a, "y", -1));
                if covers(fp, size, ax, ay) {
                    push(format!("Map '{map_id}' prop '{id}' hides '{}' at ({ax}, {ay}) behind its art.", json::s(a, "id")));
                }
            }
            for (path, points) in json::dget(t, "paths") {
                let hit = json::arr(points).iter().map(json::arr).find(|xy| xy.len() == 2 && covers(fp, size, json::to_int(&xy[0]), json::to_int(&xy[1])));
                if let Some(xy) = hit {
                    push(format!("Map '{map_id}' prop '{id}' hides '{path}' at ({}, {}) behind its art.", json::to_int(&xy[0]), json::to_int(&xy[1])));
                }
            }
        }
    }
    out
}
