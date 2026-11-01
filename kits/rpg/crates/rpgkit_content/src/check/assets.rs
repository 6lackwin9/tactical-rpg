//! The asset checks A1-A6 (ASSETORG2 section 6) over the asset layers of the checked data folder's game
//! (`View::assets`): bodies that are not there or do not load, item icons not in their folder, unique portraits and
//! common portraits missing, palettes that do not fit their body; PROPS1 3.3: the map prop checks A7-A10 (art missing
//! along the world's chain, a PNG of the wrong size, an explicit variant past the files, art hiding an anchor or path);
//! PORTRAITFX1 5.5: the portrait file checks A11 (face sidecars) and A12 (expression frames) of each layer.

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

// ---- PORTRAITFX1 5.5: the portrait file checks A11 and A12 ----------------------------------------------------------

/// A layer's name in a report key: the folder holding its `assets/` (the game, the kit, `engine`).
pub fn layer_name(root: &std::path::Path) -> String {
    root.parent().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| root.to_string_lossy().replace('\\', "/"))
}

/// Every file under `dir`, as paths relative to `root` with forward slashes, sorted.
fn files_under(root: &std::path::Path, dir: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(read) = std::fs::read_dir(&d) else { continue };
        for e in read.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if let Ok(rel) = p.strip_prefix(root) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    out.sort();
    out
}

/// A11 of one sidecar file (`rel` under `root`), without the path: its PNG missing beside it, then why it does not
/// parse, or its range and anchor problems against the PNG's width (`face_problems`; width 0 = not checked).
pub fn sidecar_problems(root: &std::path::Path, rel: &str) -> Vec<String> {
    let mut out = Vec::new();
    let png_rel = format!("{}.png", rel.strip_suffix(".face.json").unwrap_or(rel));
    let png = root.join(&png_rel);
    let width = if png.is_file() {
        png_size(&png).map_or(0, |(w, _)| w as i32)
    } else {
        out.push(format!("no portrait {} beside the sidecar.", png_rel.rsplit('/').next().unwrap_or(&png_rel)));
        0
    };
    let raw = match std::fs::read_to_string(root.join(rel)).map_err(|e| e.to_string()).and_then(|t| serde_json::from_str::<Value>(&t).map_err(|e| e.to_string())) {
        Ok(v) => v,
        Err(e) => {
            out.push(format!("the sidecar is not JSON ({e})."));
            return out;
        }
    };
    match assets::parse_face(&raw) {
        Err(p) => out.extend(p),
        Ok(face) => out.extend(assets::face_problems(&face, width)),
    }
    out
}

/// A11 and A12 over the `portraits/` of asset layer `root` (warnings, one `<path>: <problem>` line each, paths
/// relative to the root, in path order; A11 first): every `*.face.json` parses and fits its PNG (`sidecar_problems`);
/// under `portraits/<folder>/faces/<id>/` (unique tiers only) each file is `<expression>.png` or
/// `<expression>.face.json` with an expression of the game's faces `names` (PORTRAITFX2 FL4: `View::faces`), `<id>` has its
/// unique portrait or a `neutral.png` (in any of `layers`), and a frame without its own sidecar under a base sidecar
/// with eyes uses the base anchors.
pub fn portrait_file_warnings(root: &std::path::Path, layers: &assets::Layers, names: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let portraits = root.join("portraits");
    let files = files_under(root, &portraits);
    for rel in files.iter().filter(|f| f.ends_with(".face.json")) {
        out.extend(sidecar_problems(root, rel).into_iter().map(|m| format!("{rel}: {m}")));
    }
    let Ok(read) = std::fs::read_dir(&portraits) else { return out };
    let mut folders: Vec<String> = read.flatten().filter(|e| e.path().join("faces").is_dir()).map(|e| e.file_name().to_string_lossy().to_string()).collect();
    folders.sort();
    let unique: Vec<&str> = ["hero", "npc", "elite", "boss"].iter().filter_map(|t| art::tier_folder(t)).collect();
    for folder in folders {
        let faces = format!("portraits/{folder}/faces");
        if !unique.contains(&folder.as_str()) {
            out.push(format!("{faces}/: expression frames are for the unique tiers only ({}).", unique.join(", ")));
            continue;
        }
        let mut ids: Vec<String> = Vec::new();
        for rel in files.iter().filter_map(|f| f.strip_prefix(&format!("{faces}/"))) {
            match rel.split_once('/') {
                None => out.push(format!("{faces}/{rel}: a frame belongs in a character folder ({faces}/<id>/).")),
                Some((id, _)) if !ids.iter().any(|x| x == id) => ids.push(id.to_string()),
                Some(_) => {}
            }
        }
        for id in ids {
            let dir = format!("{faces}/{id}");
            let base = format!("portraits/{folder}/{id}.png");
            if !layers.has(&base) && !layers.has(&format!("{dir}/neutral.png")) {
                out.push(format!("{dir}/: no portrait {base} and no neutral.png."));
            }
            let base_eyes = layers.roots.iter().find(|r| r.join(&base).is_file()).and_then(|r| std::fs::read_to_string(r.join(assets::face_file(&base))).ok()).and_then(|t| serde_json::from_str::<Value>(&t).ok()).and_then(|v| assets::parse_face(&v).ok()).is_some_and(|f| !f.eyes.is_empty());
            for rel in files.iter().filter(|f| f.strip_prefix(&format!("{dir}/")).is_some()) {
                let name = &rel[dir.len() + 1..];
                let (stem, png) = match (name.strip_suffix(".face.json"), name.strip_suffix(".png")) {
                    (Some(s), _) => (s, false),
                    (None, Some(s)) => (s, true),
                    _ => {
                        out.push(format!("{rel}: a frame is <expression>.png or <expression>.face.json."));
                        continue;
                    }
                };
                if !names.iter().any(|f| f == stem) {
                    out.push(format!("{rel}: '{stem}' is not one of {}.", names.join(", ")));
                } else if png && base_eyes && !root.join(assets::face_file(rel)).is_file() {
                    out.push(format!("{rel}: no {stem}.face.json; it uses the base anchors ({}).", assets::face_file(&base)));
                }
            }
        }
    }
    out
}
