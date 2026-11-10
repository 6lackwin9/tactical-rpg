//! The asset checks A1-A6 (ASSETORG2 section 6) over the asset layers of the checked data folder's game
//! (`View::assets`): bodies that are not there or do not load, item icons not in their folder, unique portraits and
//! common portraits missing, palettes that do not fit their body; PROPS1 3.3: the map prop checks A7-A10 (art missing
//! along the world's chain, a PNG of the wrong size, an explicit variant past the files, art hiding an anchor or path);
//! PORTRAITFX1 5.5: the portrait file checks A11 (face sidecars) and A12 (expression frames) of each layer;
//! REGIONGROUND1 KT 1.5: the ground checks A16-A19 (art missing along the chain, a wrong size, a mark's file, a marker
//! behind a wall) and A20 (a location's boundary walls with no art).

use super::View;
use crate::art;
use aegis_core::assets::{self, Clips};
use aegis_core::json::{self, Dict};
use aegis_core::vfs;
use serde_json::Value;

/// A PNG's size from its header (None: not a PNG).
fn png_size(path: &std::path::Path) -> Option<(u32, u32)> {
    let bytes = vfs::read(path).ok()?;
    let head = bytes.get(..24)?;
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
                let sets = art::region_sets(&lineage, "").join(",");
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

// ---- LOOK1 5.4: the tile set binding checks (A13 id and files, A14 transition pieces, A15 sizes) --------------------

/// The pixel sizes of a region tile and of a stage tile (LOOK1 section 3: 16 square or tilted at 2x; a stage cell 64 x 48).
const REGION_TILE_PX: [(u32, u32); 2] = [(32, 32), (32, 24)];
const STAGE_TILE_PX: [(u32, u32); 1] = [(64, 48)];
/// The wall face (LOOK1 5.1) has its own sizes: region 32 x 16 (square) or 32 x 12 (tilted), or the tall face 128 x 48
/// (REGIONGROUND1_SCENE 1.6: one storey, 4 cells wide), stage 64 x 24.
const REGION_FACE_PX: [(u32, u32); 3] = [(32, 16), (32, 12), (128, 48)];
const STAGE_FACE_PX: [(u32, u32); 1] = [(64, 24)];

/// The world's `stage_set` and the `tileset` of each of its locations, as (label, set id) with empty ids left out.
fn bound_sets(v: &View, w: &str) -> (Option<String>, Vec<(String, String)>) {
    let stage = Some(json::s(v.worlds.manifest(w), "stage_set")).filter(|s| !s.is_empty());
    let mut regions: Vec<(String, String)> = Vec::new();
    for e in v.worlds.merged_entries(w, "locations") {
        let set = json::s(e, "tileset");
        if !set.is_empty() {
            regions.push((json::s(e, "id"), set));
        }
    }
    (stage, regions)
}

/// A13 (id rule) and A15 (wrong size) of world `w`: a set id that is not `[a-z0-9_]{2,32}`, and a PNG of a bound set
/// (`tilesets/<set>/**`) that is not a tile of the set's role; a location's `boundary` id by the same rule, and its
/// files by the same sizes. A set's `ground/` folder has its own check (A17). Errors; the size check is skipped without
/// asset layers.
pub fn tileset_errors(v: &View, w: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let (stage, regions) = bound_sets(v, w);
    if let Some(set) = &stage {
        if !art::set_id_ok(set) {
            out.push(format!("Stage set '{set}' must be 2-32 characters of a-z, 0-9 and _."));
        }
    }
    for (loc, set) in &regions {
        if !art::set_id_ok(set) {
            out.push(format!("Location '{loc}' tile set '{set}' must be 2-32 characters of a-z, 0-9 and _."));
        }
    }
    for e in v.worlds.merged_entries(w, "locations") {
        let boundary = json::s(e, "boundary");
        if !boundary.is_empty() && !art::set_id_ok(&boundary) {
            out.push(format!("Location '{}' boundary '{boundary}' must be 2-32 characters of a-z, 0-9 and _.", json::s(e, "id")));
        }
    }
    type Role<'a> = (String, &'a [(u32, u32)], &'a [(u32, u32)], &'a str);
    let mut roles: Vec<Role> = Vec::new();
    for (_, set) in &regions {
        roles.push((set.clone(), &REGION_TILE_PX, &REGION_FACE_PX, "region"));
    }
    if let Some(set) = stage {
        roles.push((set, &STAGE_TILE_PX, &STAGE_FACE_PX, "stage"));
    }
    let mut seen: Vec<(String, &str)> = Vec::new();
    for (set, tile_px, face_px, role) in roles {
        if !art::set_id_ok(&set) || seen.iter().any(|(s, r)| *s == set && *r == role) {
            continue;
        }
        seen.push((set.clone(), role));
        for root in &v.assets.roots {
            let dir = root.join(format!("tilesets/{set}"));
            let ground = format!("tilesets/{set}/ground/");
            for rel in files_under(root, &dir).into_iter().filter(|r| r.ends_with(".png") && !r.starts_with(&ground)) {
                if let Some(size) = png_size(&root.join(&rel)) {
                    let in_set = rel.strip_prefix(&format!("tilesets/{set}/")).unwrap_or(&rel);
                    let tile = in_set.strip_prefix("boundary/").and_then(|b| b.split_once('/')).map_or(in_set, |(_, rest)| rest);
                    let face = tile.starts_with("wall_face/");
                    let (sizes, what) = if face { (face_px, format!("{role} wall face")) } else { (tile_px, format!("{role} tile")) };
                    if !sizes.contains(&size) {
                        let wants = sizes.iter().map(|(a, b)| format!("{a}x{b}")).collect::<Vec<_>>().join(" or ");
                        out.push(format!("Tile art '{rel}' is {}x{} px; a {what} wants {wants}.", size.0, size.1));
                    }
                }
            }
        }
    }
    out
}

/// A13 (files) and A14 of world `w`, warnings: a bound set no layer holds (a region set with neither
/// `tilesets/<set>/floor/v0.png` nor a `ground/` folder, a stage set without a tile), a location's map using a tile its
/// set lacks (a map with ground draws `.` and `~` from its grounds, so only `#` counts there), and a
/// transition pair with some of its 14 pieces (`m01`-`m14`) missing. Skipped without asset layers.
pub fn tileset_warnings(v: &View, w: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if v.assets.roots.is_empty() {
        return out;
    }
    let (stage, regions) = bound_sets(v, w);
    let mut push = |line: String| {
        if !out.contains(&line) {
            out.push(line);
        }
    };
    let mut sets: Vec<String> = Vec::new();
    for (loc, set) in &regions {
        if !art::set_id_ok(set) {
            continue;
        }
        if !v.assets.has(&format!("tilesets/{set}/floor/v0.png")) && v.assets.dirs(&format!("tilesets/{set}/ground")).is_empty() {
            push(format!("Location '{loc}' tile set '{set}': no file tilesets/{set}/floor/v0.png in any layer; it draws today's sets."));
        } else {
            let map_id = v.worlds.merged_entries(w, "locations").into_iter().find(|e| json::s(e, "id") == *loc).map(|e| json::s(e, "map")).unwrap_or_default();
            let t = v.get("maps", &map_id);
            let used: Vec<char> = json::aget(t, "rows").iter().flat_map(|r| json::gstr(r).chars().collect::<Vec<_>>()).collect();
            let has_ground = !json::aget(t, "ground").is_empty();
            for ch in ['.', '#', '~'] {
                let tile = art::region_tile(ch).unwrap_or("floor");
                if used.contains(&ch) && !(has_ground && ch != '#') && v.assets.variants(&format!("tilesets/{set}/{tile}")) == 0 {
                    push(format!("Location '{loc}' tile set '{set}': its map uses '{ch}' but tilesets/{set}/{tile}/v0.png is missing; that tile draws today's set."));
                }
            }
        }
        sets.push(set.clone());
    }
    if let Some(set) = stage {
        if art::set_id_ok(&set) {
            let has_tile = v.assets.dirs(&format!("tilesets/{set}")).iter().any(|d| v.assets.has(&format!("tilesets/{set}/{d}/v0.png")));
            if !has_tile {
                push(format!("Stage set '{set}': no tilesets/{set}/<terrain>/v0.png in any layer; the battle draws common_stage."));
            }
            sets.push(set);
        }
    }
    sets.sort();
    sets.dedup();
    for set in sets {
        let base = format!("tilesets/{set}");
        for upper in v.assets.dirs(&base) {
            for pair in v.assets.dirs(&format!("{base}/{upper}")).into_iter().filter(|p| p.starts_with("over_")) {
                let dir = format!("{base}/{upper}/{pair}");
                let missing: Vec<String> = (1..=14u8).map(|n| format!("m{n:02}")).filter(|m| !v.assets.has(&format!("{dir}/{m}.png"))).collect();
                if !missing.is_empty() && missing.len() < 14 {
                    push(format!("Tile set '{set}' pair {upper} {pair}: {} of 14 pieces missing ({}); the pair draws no transition.", missing.len(), missing.join(", ")));
                }
            }
        }
    }
    out
}

// ---- REGIONGROUND1 KT 1.5: the ground checks A16-A19, and A20 ---------------------------------------------------------

/// A world's region-map locations: (map id, the tile sets over its own `tileset` and the lineage).
fn map_locations(v: &View, w: &str) -> Vec<(String, Vec<String>)> {
    let mut lineage = v.worlds.lineage(w);
    lineage.reverse();
    let mut out = Vec::new();
    for e in v.worlds.merged_entries(w, "locations") {
        let map = json::s(e, "map");
        if !map.is_empty() {
            let tileset = json::s(e, "tileset");
            let set = if art::set_id_ok(&tileset) { tileset } else { String::new() };
            out.push((map, art::region_sets(&lineage, &set)));
        }
    }
    out
}

/// The ground ids the cells of template `t` use, in legend order.
fn used_grounds(t: &Dict) -> Vec<String> {
    let cells: Vec<char> = json::aget(t, "ground").iter().flat_map(|r| json::gstr(r).chars().collect::<Vec<_>>()).collect();
    let mut out: Vec<String> = Vec::new();
    for (k, id) in json::dget(t, "ground_legend") {
        let id = json::gstr(id);
        if cells.iter().any(|c| c.to_string() == *k) && !out.contains(&id) {
            out.push(id);
        }
    }
    out
}

/// A17 of world `w` (an error, as A15): a file of a ground along any of its locations' chains that is not
/// `art::ground_px` square, `tilesets/<set>/ground/<ground>/<file>.png`. A deeper file is not a ground's texture. Skipped
/// without asset layers.
pub fn ground_errors(v: &View, w: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut sets: Vec<String> = map_locations(v, w).into_iter().flat_map(|(_, s)| s).collect();
    sets.sort();
    sets.dedup();
    let px = art::ground_px();
    for root in &v.assets.roots {
        for set in &sets {
            let base = format!("tilesets/{set}/ground/");
            for rel in files_under(root, &root.join(&base)).into_iter().filter(|r| r.ends_with(".png") && r[base.len()..].matches('/').count() == 1) {
                match png_size(&root.join(&rel)) {
                    Some((a, b)) if (a, b) != (px, px) => out.push(format!("Ground art '{rel}' is {a}x{b} px; it must be {px} x {px}.")),
                    _ => {}
                }
            }
        }
    }
    out
}

/// The largest side of a ground mark's file in art px (A18): 32 x 32 at the job's density, from a 32-per-cell file
/// (REGIONGROUND1_ENGINE 3.4).
const GROUND_MARK_PX: u32 = 64;
/// A wall storey covers this many tilted rows north of its cell (REGIONGROUND1_SCENE 1.3: a unit is 24 tall, a row 12).
const ROWS_PER_STOREY: i64 = 2;

/// A16, A18, A19 and A20 of world `w`, warnings, one line per finding; skipped without asset layers.
pub fn ground_warnings(v: &View, w: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if v.assets.roots.is_empty() {
        return out;
    }
    let mut lineage = v.worlds.lineage(w);
    lineage.reverse();
    let mut push = |line: String| {
        if !out.contains(&line) {
            out.push(line);
        }
    };
    for (map_id, sets) in map_locations(v, w) {
        let t = v.get("maps", &map_id);
        for id in used_grounds(t) {
            // A16
            if !art::ground_dirs(&sets, &id).iter().any(|d| v.assets.has(&format!("{d}/v0.png"))) {
                push(format!("Map '{map_id}' ground '{id}': no art for world '{w}'; it draws its flat colour."));
            }
            // A18
            for mark in json::sget(v.get(super::ground::KIND, &id), "marks") {
                let dirs = art::prop_dirs(&lineage, &mark);
                let Some((dir, n)) = dirs.iter().map(|d| (d.clone(), v.assets.variants(d))).find(|(_, n)| *n > 0) else {
                    let names = art::region_sets(&lineage, "").join(",");
                    push(format!("Ground '{id}' mark '{mark}': no file props/{{{names}}}/{mark}/v0.png for world '{w}'."));
                    continue;
                };
                for k in 0..n {
                    let size = v.assets.find(&format!("{dir}/v{k}.png")).and_then(|path| png_size(&path));
                    if let Some((a, b)) = size.filter(|(a, b)| *a > GROUND_MARK_PX || *b > GROUND_MARK_PX) {
                        push(format!("Ground '{id}' mark '{mark}': {dir}/v{k}.png is {a}x{b} px, larger than {GROUND_MARK_PX} x {GROUND_MARK_PX}."));
                    }
                }
            }
        }
    }
    // A19 and A20, per location: a marker's anchor and the boundary's art
    for e in v.worlds.merged_entries(w, "locations") {
        let map_id = json::s(e, "map");
        let t = v.get("maps", &map_id);
        for m in json::aget(e, "markers").iter().map(json::dict) {
            let anchor = json::s(m, "anchor");
            let Some(a) = json::aget(t, "anchors").iter().map(json::dict).find(|a| json::s(a, "id") == anchor) else { continue };
            if behind_a_wall(v, &lineage, t, json::i(a, "x", -1), json::i(a, "y", -1)) {
                push(format!("Map '{map_id}' marker anchor '{anchor}' stands behind a wall; it draws its outline."));
            }
        }
        let boundary = json::s(e, "boundary");
        if art::set_id_ok(&boundary) {
            let tileset = json::s(e, "tileset");
            let sets = art::region_sets(&lineage, if art::set_id_ok(&tileset) { &tileset } else { "" });
            let found = ["wall", "wall_face"].iter().any(|tile| art::boundary_dirs(&sets, &boundary, tile).iter().any(|d| v.assets.has(&format!("{d}/v0.png"))));
            if !found {
                push(format!("Location '{}' boundary '{boundary}': no art along its tile sets; it draws the set's own walls.", json::s(e, "id")));
            }
        }
    }
    out
}

/// A19 (REGIONGROUND1_SCENE 1.3): whether cell (x, y) lies on the rows a wall south of it covers (2 per storey), that
/// wall being off the south row and the side columns (those stay short) and not under a standing prop with art (the art
/// is the height, and it draws no block).
fn behind_a_wall(v: &View, lineage: &[String], t: &Dict, x: i64, y: i64) -> bool {
    let Some(rows) = super::map::grid(t) else { return false };
    let (mw, mh) = (rows[0].len() as i64, rows.len() as i64);
    if x <= 0 || x >= mw - 1 || y < 0 || y >= mh {
        return false;
    }
    let tallest = ROWS_PER_STOREY * super::ground::WALL_STOREYS_MAX;
    (y + 1..=(y + tallest).min(mh - 2)).any(|wy| rows[wy as usize][x as usize] == '#' && wy - y <= ROWS_PER_STOREY * super::ground::storeys_at(t, x, wy) && !under_standing_art(v, lineage, t, x, wy))
}

/// Whether wall cell (x, y) lies under a standing prop whose art is on hand along the chain.
fn under_standing_art(v: &View, lineage: &[String], t: &Dict, x: i64, y: i64) -> bool {
    json::aget(t, "props").iter().map(json::dict).filter(|p| json::s(p, "layer") == "sort").any(|p| prop_footprint(p).is_some_and(|(px, py, pw, ph)| x >= px && y >= py && x < px.saturating_add(pw) && y < py.saturating_add(ph)) && art::prop_dirs(lineage, &json::s(p, "art")).iter().any(|d| v.assets.variants(d) > 0))
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
        let Ok(read) = vfs::list_dir(&d) else { continue };
        for p in read {
            if vfs::is_dir(&p) {
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
    let width = if vfs::is_file(&png) {
        png_size(&png).map_or(0, |(w, _)| w as i32)
    } else {
        out.push(format!("no portrait {} beside the sidecar.", png_rel.rsplit('/').next().unwrap_or(&png_rel)));
        0
    };
    let raw = match vfs::read_to_string(root.join(rel)).map_err(|e| e.to_string()).and_then(|t| serde_json::from_str::<Value>(&t).map_err(|e| e.to_string())) {
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
    let Ok(read) = vfs::list_dir(&portraits) else { return out };
    let mut folders: Vec<String> = read.into_iter().filter(|p| vfs::is_dir(p.join("faces"))).map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string()).collect();
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
            let base_eyes = layers.roots.iter().find(|r| vfs::is_file(r.join(&base))).and_then(|r| vfs::read_to_string(r.join(assets::face_file(&base))).ok()).and_then(|t| serde_json::from_str::<Value>(&t).ok()).and_then(|v| assets::parse_face(&v).ok()).is_some_and(|f| !f.eyes.is_empty());
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
                } else if png && base_eyes && !vfs::is_file(root.join(assets::face_file(rel))) {
                    out.push(format!("{rel}: no {stem}.face.json; it uses the base anchors ({}).", assets::face_file(&base)));
                }
            }
        }
    }
    out
}
