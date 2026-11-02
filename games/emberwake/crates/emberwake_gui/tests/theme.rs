//! LOOK1-G1: the shipped Tallow and Iron theme (`games/emberwake/assets/ui/theme.json` and `ui/frames/`, spec
//! LOOK1 sections 2-3 and LOOK1_TASKS G1). It loads with no error through the game's asset layers (game, kit,
//! engine); every LOOK1 8.2 contrast pair meets its minimum; every frame role resolves to the game's own file, at
//! the G1 table's size and slice, in the art bible's hues; the faces are the picked fonts.

use aegis_core::assets::Layers;
use aegis_ui::painter::Typeface;
use aegis_ui::theme::{self, contrast_pairs, parse_colour, Theme, FRAME_ROLES};
use std::path::{Path, PathBuf};

fn game_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn shipped() -> Theme {
    let layers = Layers::for_game(&game_dir());
    assert!(layers.notices.is_empty(), "layer notices: {:?}", layers.notices);
    match theme::load(&layers) {
        Ok(t) => t,
        Err(errs) => panic!("the game theme has errors:\n{}", errs.join("\n")),
    }
}

/// The art bible's 16 hues (games/emberwake/docs/lore/specs/art_and_visual_style.md section 3) and the two inset
/// darks the G1 frame table names.
const PALETTE: [&str; 18] = [
    "#0a0b10", "#14171f", "#1e232d", "#2c221e", "#4b5563", "#9ca3af", "#d9cbb4", "#6b7280", "#92400e", "#d97706", "#fbbf24", "#e11d48", "#991b1b", "#9333ea", "#c084fc", "#06b6d4", "#050608", "#0e1016",
];

#[test]
fn the_game_theme_loads_clean_from_the_game_layers() {
    let t = shipped();
    assert!(t.sources.iter().any(|s| s == "ui/theme.json (game)"), "sources: {:?}", t.sources);
    assert!(t.frames_on);
    assert_eq!(t.layout.radius, 0.0);
    assert_eq!(t.layout.frame_radius, 0.0);
    assert_eq!(t.layout.focus_glow, 1);
    // The focus ring follows the focus colour; foe and good keep today's (LOOK1 Q10).
    assert_eq!(t.ring, t.focus);
    let d = Theme::default();
    assert_eq!((t.foe, t.good), (d.foe, d.good));
}

#[test]
fn every_contrast_pair_meets_its_minimum() {
    let t = shipped();
    let mut low = Vec::new();
    for (fg, ground, least) in contrast_pairs() {
        let r = t.contrast(t.role(fg).unwrap(), t.role(ground).unwrap());
        println!("{fg} on {ground}: {r:.2} (least {least})");
        if r < least {
            low.push(format!("{fg} on {ground}: {r:.2} < {least}"));
        }
    }
    assert!(low.is_empty(), "{low:?}");
}

#[test]
fn every_frame_role_resolves_to_a_game_frame_in_the_palette() {
    let t = shipped();
    let palette: Vec<[u8; 3]> = PALETTE.iter().map(|h| {
        let c = parse_colour(h).unwrap();
        [c[0], c[1], c[2]]
    }).collect();
    for (role, _) in FRAME_ROLES {
        let f = t.frame(role).unwrap_or_else(|| panic!("frame role {role} draws the vector panel"));
        assert_eq!(f.source, format!("ui/frames/{role}.png (game)"), "{role} resolves to another role's file");
        let (size, slice) = match role {
            "panel" => (24, 8),
            "ornate" => (f.nine.slice[0] * 2 + 32, f.nine.slice[0]),
            "ornate_full" => (f.nine.slice[0] * 2 + 64, f.nine.slice[0]),
            _ => (16, 4),
        };
        assert_eq!((f.img.w, f.img.h), (size, size), "{role} size");
        assert_eq!(f.nine.slice, [slice; 4], "{role} slice");
        let high = role.starts_with("ornate");
        assert!(if high { f.nine.scale >= 8 } else { f.nine.scale == 2 }, "{role} scale");
        assert_eq!(f.nine.reduce, high, "{role}: only the high-resolution frames average");
        let ink = if role == "parchment" { "ink" } else { "text" };
        assert_eq!(f.nine.ink, ink, "{role} ink");
        for (i, px) in f.img.px.chunks(4).enumerate() {
            // The ornate corners are the picked kit's art (decision "Expression sheets: scale only and aligned cells": no palette pass): the rule holds for
            // its edges and centre, where a straddling frame's bands are see-through past the line.
            let (x, y) = (i as i32 % size, i as i32 / size);
            if role == "ornate" && (x < slice || x >= size - slice) && (y < slice || y >= size - slice) {
                continue;
            }
            if role == "ornate" && px[3] == 0 {
                continue;
            }
            // Its edges are the kit's art too.
            if role == "ornate_full" && !((slice..size - slice).contains(&x) && (slice..size - slice).contains(&y)) {
                continue;
            }
            assert!(palette.contains(&[px[0], px[1], px[2]]), "{role}: colour {:02x}{:02x}{:02x} is not in the palette", px[0], px[1], px[2]);
            assert!([255, 0xf0, 0xe0].contains(&px[3]), "{role}: alpha {}", px[3]);
        }
    }
}

/// `ornate`'s corners are the picked kit's art, not a few flat hues: `gen_art.py` must not fall back to drawing them. Its
/// tips are in the spikes' light: the brightest tenth of a tip is as bright as the brightest tenth of the outer spikes, within
/// a tenth (the kit's own tip, unlit, is far duller).
#[test]
fn the_ornate_corners_are_the_picked_art() {
    let t = shipped();
    let f = t.frame("ornate").expect("the ornate frame");
    let slice = f.nine.slice[0];
    let mut colours = std::collections::HashSet::new();
    let px = |x: i32, y: i32| {
        let i = ((y * f.img.w + x) * 4) as usize;
        [f.img.px[i], f.img.px[i + 1], f.img.px[i + 2], f.img.px[i + 3]]
    };
    for y in 0..slice {
        for x in 0..slice {
            colours.insert(px(x, y));
        }
    }
    assert!(colours.len() > 1000, "only {} colours in the corner", colours.len());
    assert!(f.nine.straddle, "half in, half out");
    let bright = |cells: Vec<(i32, i32)>| {
        let mut lum: Vec<f32> = cells.into_iter().map(|(x, y)| px(x, y)).filter(|p| p[3] > 128).map(|p| 0.3 * p[0] as f32 + 0.59 * p[1] as f32 + 0.11 * p[2] as f32).collect();
        assert!(lum.len() > 50, "too few pixels: {}", lum.len());
        lum.sort_by(f32::total_cmp);
        let tenth = lum.len() / 10;
        lum[lum.len() - tenth..].iter().sum::<f32>() / tenth as f32
    };
    let [ax, ay] = f.nine.corner_anchor.expect("the anchor");
    let spike = bright((0..slice).flat_map(|x| (0..ay - 3 * f.nine.scale).map(move |y| (x, y))).collect());
    // Past the stub's cut along the top arm: the head's own art, up to the slice's end; the same down the left arm.
    let cut = 10 * f.nine.scale;
    let tip_x = bright((ax + cut..slice).flat_map(|x| (ay - 2 * f.nine.scale..ay + 2 * f.nine.scale).map(move |y| (x, y))).collect());
    let tip_y = bright((ay + cut..slice).flat_map(|y| (ax - 2 * f.nine.scale..ax + 2 * f.nine.scale).map(move |x| (x, y))).collect());
    println!("the brightest tenth: spikes {spike:.0}, tips {tip_x:.0} and {tip_y:.0}");
    for (name, tip) in [("across", tip_x), ("down", tip_y)] {
        assert!((tip - spike).abs() <= 0.1 * spike, "{name}: the tip {tip:.0} against the spikes {spike:.0}");
    }
}

/// Every frame's thin accent line is the theme's `frame_line`: `ornate` has one, on the rows `gen_art.py` draws it, and
/// no other frame carries that shade, which is decorative but must stay visible on the panel.
#[test]
fn the_frame_line_is_the_one_role() {
    let t = shipped();
    let line = t.role("frame_line").unwrap();
    let f = t.frame("ornate").unwrap();
    let mid = (f.img.w / 2 * 4) as usize;
    let row = (f.nine.slice[1] / 2) * f.img.w;
    let at = row as usize * 4 + mid;
    assert_eq!(f.img.px[at..at + 4], line, "the top edge's line");
    for (role, _) in FRAME_ROLES {
        let other = t.frame(role).unwrap();
        if role != "ornate" {
            assert!(!other.img.px.chunks(4).any(|p| p[..3] == line[..3]), "{role} has the line's colour but is not listed");
        }
    }
    let ratio = t.contrast(line, t.role("panel").unwrap());
    println!("frame_line on panel: {ratio:.2}");
    assert!(ratio >= 2.0, "frame_line on panel: {ratio:.2}");
}

/// The shipped frame at every window size: its corners keep one size on any panel and follow the window, centred on the
/// boundary across the arm; the line is exactly `frame_line`, one logical px wide, across the boundary between the corners'
/// tips, with nothing else on the edge; the fill stops at the boundary.
#[test]
fn the_ornate_frame_straddles_at_one_corner_size_at_every_window_size() {
    use aegis_ui::canvas::{Canvas, Rect};
    use aegis_ui::painter::{Fonts, Painter, View};
    let t = shipped();
    let f = t.frame("ornate").unwrap();
    let line = t.role("frame_line").unwrap();
    let font = std::fs::read(game_dir().join("../../engine/assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
    let mut fonts = Fonts::new(&font);
    let slice = f.nine.slice[0];
    let ground = [0, 255, 0, 255];
    let scale = f.nine.scale as f32;
    for (ww, wh, whole) in [(960, 540, false), (1280, 720, true), (1920, 1080, true), (3840, 2160, true)] {
        let reach_of = |fonts: &mut Fonts, r: Rect| {
            let mut c = Canvas::new(ww, wh);
            c.clear(ground);
            let mut p = Painter { c: &mut c, v: View::fit(ww as u32, wh as u32), fonts };
            p.nine(&f.img, &f.nine, r);
            let pr = p.v.rect(r);
            // Along a row of the panel's top arm, off the line: above it, where only the corners and tips draw.
            let at = pr.y - (p.v.s * 2.0).ceil() as i32;
            let drawn = |x: i32| p.c.px[((at * p.c.w + x) * 4 + 1) as usize] != 255;
            let far = (pr.x..pr.x + pr.w / 2).rfind(|x| drawn(*x)).unwrap();
            (far - pr.x, c)
        };
        let r = Rect::new(200, 120, 160, 64);
        let view = View::fit(ww as u32, wh as u32);
        let pr = view.rect(r);
        let (one, c) = reach_of(&mut fonts, r);
        for other in [Rect::new(200, 120, 280, 170), Rect::new(200, 120, 40, 160), Rect::new(30, 120, 580, 60)] {
            assert_eq!(reach_of(&mut fonts, other).0, one, "{ww}: the corner on {other:?}");
        }
        let s = view.s;
        assert!((one as f32) < (slice - f.nine.corner_anchor.unwrap()[0]) as f32 * s / scale + 1.0, "{ww}: past the art: {one}");
        let at = |x: i32, y: i32| [c.px[((y * c.w + x) * 4) as usize], c.px[((y * c.w + x) * 4 + 1) as usize], c.px[((y * c.w + x) * 4 + 2) as usize]];
        // The line, at the top edge's middle: rows of the role's colour only, then ground above and the fill below.
        let mid = pr.x + pr.w / 2;
        let rows: Vec<i32> = (pr.y - (4.0 * s) as i32..pr.y + (4.0 * s) as i32).filter(|y| at(mid, *y) == [line[0], line[1], line[2]]).collect();
        if whole {
            assert_eq!(rows.len() as f32, s, "{ww}: the line's rows {rows:?}");
            assert!(rows[0] < pr.y && *rows.last().unwrap() >= pr.y, "{ww}: across the boundary: {rows:?} at {}", pr.y);
            assert_eq!(at(mid, rows[0] - 1), [0, 255, 0], "{ww}: nothing outside the line");
        }
        assert_ne!(at(mid, pr.y + (2.0 * s) as i32)[1], 255, "{ww}: the fill reaches the line");
        // All four sides carry the same line, between the tips.
        let (midy, right, bottom) = (pr.y + pr.h / 2, pr.x + pr.w - 1, pr.y + pr.h - 1);
        if whole {
            for (name, p) in [("left", at(pr.x - 1, midy)), ("right", at(right + 1, midy)), ("bottom", at(mid, bottom + 1))] {
                assert_eq!(p, [line[0], line[1], line[2]], "{ww}: the {name} line");
            }
        }
        // Across the top arm, by the stub: the art's section about the boundary, within a logical px each way.
        let stub = pr.x + (6.0 * s) as i32;
        let drawn: Vec<i32> = (pr.y - (5.0 * s) as i32..pr.y + (5.0 * s) as i32).filter(|y| at(stub, *y) != [0, 255, 0] && at(stub, *y)[1] > 40).map(|y| y - pr.y).collect();
        let off = (drawn[0] + drawn.last().unwrap() + 1) as f32 / 2.0;
        assert!(off.abs() <= s, "{ww}: the arm's section {drawn:?} is {off} px off the boundary");
    }
}

/// The kit's edge as a whole frame, at every window size and any panel length: each side keeps one section centred on the
/// boundary, repeats at one period whatever the length instead of stretching, and joins its corners without a step.
#[test]
fn the_full_frames_edge_is_continuous_at_every_window_size_and_length() {
    use aegis_ui::canvas::{Canvas, Rect};
    use aegis_ui::painter::{Fonts, Painter, View};
    let t = shipped();
    let f = t.frame("ornate_full").expect("the full frame");
    assert_eq!(f.nine.edges, aegis_ui::theme::NineFill::Tile);
    let font = std::fs::read(game_dir().join("../../engine/assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
    let mut fonts = Fonts::new(&font);
    let (slice, scale) = (f.nine.slice[0], f.nine.scale as f32);
    let seg = f.img.w - 2 * slice;
    let [ax, ay] = f.nine.corner_anchor.unwrap();
    let ground = [0, 255, 0, 255];
    // Printed at 6-14 across the window sizes; a soft row drawn twice under the corner printed 44-63.
    const SEAM: u8 = 16;
    for (ww, wh) in [(960, 540), (1280, 720), (1920, 1080), (3840, 2160)] {
        let view = View::fit(ww as u32, wh as u32);
        let s = view.s;
        let period = (seg as f32 * s / scale).round() as i32;
        let mut first: Option<Vec<[u8; 3]>> = None;
        let mut seam = 0;
        for (w, h) in [(80, 60), (97, 61), (160, 64), (333, 150), (580, 170)] {
            let r = Rect::new(20, 40, w, h);
            let pr = view.rect(r);
            let mut c = Canvas::new(ww, wh);
            c.clear(ground);
            let mut p = Painter { c: &mut c, v: view, fonts: &mut fonts };
            p.nine(&f.img, &f.nine, r);
            let at = |x: i32, y: i32| {
                let i = ((y * c.w + x) * 4) as usize;
                [c.px[i], c.px[i + 1], c.px[i + 2]]
            };
            // Where the corners' art ends and only the band draws; the band's rows against the ground, above the boundary.
            let (near, far) = (pr.x + ((slice - ax) as f32 * s / scale) as i32 + 1, pr.x + pr.w - ((slice - ax) as f32 * s / scale) as i32 - 2);
            let top_of = |x: i32| (pr.y - (6.0 * s) as i32..pr.y).find(|y| at(x, *y) != [0, 255, 0]);
            let tops: Vec<Option<i32>> = (near - (4.0 * s) as i32..far + (4.0 * s) as i32).map(top_of).collect();
            let row = tops[0].expect("the band");
            assert!(tops.iter().all(|y| y.is_some_and(|y| (y - row).abs() <= 1)), "{ww} {w} x {h}: the outer rows step across the join: {row} and {tops:?}");
            // The section centred on the boundary, within a px of rounding, against the fill inside: its outer half's size.
            let half = pr.y - row;
            assert!((half as f32 - 10.0 * s / scale).abs() <= 1.5, "{ww} {w} x {h}: {half} px outside the boundary at view {s}");
            // Along the spine, across the corner's join too: no step in brightness larger than the art's own.
            let spine: Vec<[u8; 3]> = (near - (4.0 * s) as i32..far + (4.0 * s) as i32).map(|x| at(x, pr.y)).collect();
            let worst = spine.windows(2).map(|q| q[0].iter().zip(q[1]).map(|(a, b)| a.abs_diff(b)).max().unwrap()).max().unwrap();
            assert!(worst <= 48, "{ww} {w} x {h}: a step of {worst} along the spine");
            // At the corner cell's end, on every row of the section on both near sides: the art on either side of the join reads
            // alike, so no soft row is drawn twice.
            let end = ((slice - ax) as f32 * s / scale).round() as i32;
            let end_y = ((slice - ay) as f32 * s / scale).round() as i32;
            let cross = (2.0 * s) as i32;
            let step = |p: [u8; 3], q: [u8; 3]| p.iter().zip(q).map(|(x, y)| x.abs_diff(y)).max().unwrap();
            for d in -cross..cross {
                // The near corners' ends, and the far corners', whose phase depends on the panel's length.
                seam = seam.max(step(at(pr.x + end - 1, pr.y + d), at(pr.x + end, pr.y + d)));
                seam = seam.max(step(at(pr.x + d, pr.y + end_y - 1), at(pr.x + d, pr.y + end_y)));
                let (right, bottom) = (pr.x + pr.w, pr.y + pr.h);
                seam = seam.max(step(at(right - end - 1, pr.y + d), at(right - end, pr.y + d)));
                seam = seam.max(step(at(pr.x + d, bottom - end_y - 1), at(pr.x + d, bottom - end_y)));
            }
            // Repeats: the same row of the band reads the same one period on, on every length; the same art from its start.
            let row_px: Vec<[u8; 3]> = (near..far).map(|x| at(x, pr.y - 1)).collect();
            if w > 200 {
                for i in 0..row_px.len().saturating_sub(period as usize) {
                    assert_eq!(row_px[i], row_px[i + period as usize], "{ww} {w} x {h}: at {i} of period {period}");
                }
            }
            let phase: Vec<[u8; 3]> = (pr.x + 2 * period..pr.x + 3 * period).map(|x| at(x, pr.y - 1)).collect();
            if pr.x + 3 * period < far {
                assert_eq!(first.get_or_insert_with(|| phase.clone()), &phase, "{ww} {w} x {h}: one period reads alike on any length");
            }
        }
        println!("{ww}: the largest step at the join {seam}");
        assert!(seam <= SEAM, "{ww}: a step of {seam} at the corner's end");
    }
}

#[test]
fn the_faces_are_the_picked_fonts() {
    let t = shipped();
    for (face, file) in [(Typeface::Body, "fonts/OpenSans-SemiBold.ttf"), (Typeface::Display, "fonts/AlegreyaSC-Bold.ttf"), (Typeface::Mono, "fonts/JetBrainsMono-Bold.ttf")] {
        let f = t.face(face);
        assert_eq!(f.file, file);
        assert!(f.bytes.is_some(), "{file} was not read");
    }
    // Every shipped font has its licence beside it.
    let fonts = game_dir().join("assets/fonts");
    for e in std::fs::read_dir(&fonts).unwrap() {
        let name = e.unwrap().file_name().to_string_lossy().into_owned();
        if let Some(stem) = name.strip_suffix(".ttf") {
            let family = stem.split('-').next().unwrap();
            assert!(fonts.join(format!("{family}-OFL.txt")).is_file(), "{name} has no {family}-OFL.txt");
        }
    }
}

// ---- LOOK1-G2: emberwake_gui reads the theme --------------------------------------------------------------------------

/// The crate's sources by file, the path relative to `src`.
fn sources() -> Vec<(String, String)> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    walk(&root, &mut files);
    files.sort();
    files.into_iter().map(|p| (p.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/"), std::fs::read_to_string(&p).unwrap())).collect()
}

/// `src` with comments gone and each string's contents gone (the quotes stay); a char literal goes too.
fn code_only(src: &str) -> String {
    let b = src.as_bytes();
    let (mut out, mut i) = (Vec::with_capacity(b.len()), 0);
    while i < b.len() {
        if b[i] == b'/' && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
        } else if b[i] == b'"' {
            out.push(b'"');
            i += 1;
            while i < b.len() && b[i] != b'"' {
                i += if b[i] == b'\\' { 2 } else { 1 };
            }
            out.push(b'"');
            i += 1;
        } else if b[i] == b'\'' && (b.get(i + 2) == Some(&b'\'') || (b.get(i + 1) == Some(&b'\\') && b.get(i + 3) == Some(&b'\''))) {
            i += if b[i + 1] == b'\\' { 4 } else { 3 };
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The identifiers of `code`, each with the character before it (`:` for a path).
fn idents(code: &str) -> Vec<(char, String)> {
    let (mut out, mut cur, mut before, mut last) = (Vec::new(), String::new(), ' ', ' ');
    for c in code.chars() {
        if c.is_alphanumeric() || c == '_' {
            if cur.is_empty() {
                before = last;
            }
            cur.push(c);
        } else if !cur.is_empty() {
            out.push((before, std::mem::take(&mut cur)));
        }
        last = c;
    }
    out
}

/// The old colour constants of `aegis_ui::ui`: a screen draws through `th()`. A bare name is legal only in a file that
/// declares its own constant of that name (`fx::GOLD`, an effect's art hue; `timeline_view::FOE`, a block colour: art by
/// design, LOOK1_TASKS G2 point 3); a path to `ui::` never is.
#[test]
fn no_ui_colour_constant_is_left_in_the_screens() {
    const OLD: [&str; 13] = ["TEXT", "DIM", "GOLD", "INFO", "FOE", "GOOD", "BORDER", "CARD", "PANEL_HI", "MODAL", "FRAME", "BG", "SELECTED"];
    let mut found = Vec::new();
    for (file, src) in sources() {
        let code = code_only(&src);
        let tokens = idents(&code);
        for name in OLD {
            let own = code.contains(&format!("const {name}: Rgba"));
            let bare = tokens.iter().any(|(before, t)| t == name && *before != ':');
            if (bare && !own) || code.contains(&format!("ui::{name}")) {
                found.push(format!("{file}: {name}"));
            }
        }
    }
    assert!(found.is_empty(), "{found:?}");
}

/// Each `.game("name", fallback)` of the sources: (name, fallback text, file).
fn game_colour_uses() -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for (file, src) in sources() {
        let mut rest = src.as_str();
        while let Some(at) = rest.find(".game(\"") {
            rest = &rest[at + 7..];
            let end = rest.find('"').unwrap();
            let name = rest[..end].to_string();
            let args = rest[end + 1..].trim_start_matches([',', ' ']);
            let mut depth = 0;
            let stop = args.char_indices().find(|(_, c)| {
                match c {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
                depth < 0
            });
            out.push((name, args[..stop.map_or(0, |s| s.0)].to_string(), file.clone()));
        }
    }
    out
}

/// The theme file's `game_colors` and the code's agree both ways, and one name has one fallback (so the default theme
/// draws each as it did).
#[test]
fn every_game_colour_is_set_used_and_has_one_fallback() {
    let uses = game_colour_uses();
    assert!(!uses.is_empty());
    let t = shipped();
    let mut fallback = std::collections::BTreeMap::new();
    for (name, lit, file) in &uses {
        assert!(t.game_colors.contains_key(name), "{file}: '{name}' is not in theme.json");
        let first = fallback.entry(name.clone()).or_insert_with(|| lit.clone());
        assert_eq!(first, lit, "{file}: '{name}' has two fallbacks");
    }
    for name in t.game_colors.keys() {
        assert!(fallback.contains_key(name), "theme.json sets '{name}' and no code reads it");
    }
}

/// The game colours filled behind text, with the theme roles drawn on them (`dim` at 3:1, the rest at 4.5:1). A
/// translucent fill is taken over `bg`, as `contrast` does.
const FILLS_BEHIND_TEXT: [(&str, &[&str]); 28] = [
    ("battle_panel", &["text", "dim", "gold", "foe", "good", "info"]),
    ("chip", &["text", "gold", "dim"]),
    ("column", &["text", "gold", "dim", "foe", "good"]),
    ("defeat_card", &["text", "foe"]),
    ("editor_pane", &["text", "dim", "gold"]),
    ("foe_hot", &["text", "foe", "dim"]),
    ("hold_pick_fill", &["gold", "text"]),
    ("hud_bar", &["text", "gold", "dim"]),
    ("legend_fill", &["text", "dim"]),
    ("log_fill", &["text", "dim", "gold", "foe"]),
    ("map_pane", &["text", "gold", "dim", "good", "info"]),
    ("menu_row", &["text", "gold", "dim"]),
    ("outcome_fill", &["gold", "good", "foe", "text"]),
    ("party_box", &["text", "gold", "dim"]),
    ("popup_fill", &["text", "gold", "dim"]),
    ("roster_hot", &["text", "gold", "dim"]),
    ("settings_body", &["text", "dim", "gold"]),
    ("slot_filled", &["text", "gold", "dim"]),
    ("slot_open", &["text", "gold", "dim"]),
    ("strip_cell", &["dim"]),
    ("talk_box", &["text", "gold", "dim"]),
    ("dojo_pick", &["text", "gold"]),
    ("dojo_idle", &["text", "gold", "dim"]),
    ("foe_chip", &["foe", "text"]),
    ("slot_empty", &["text", "dim"]),
    ("result_veil", &["text", "gold", "dim"]),
    ("party_area", &["text", "dim"]),
    ("timeline_fill", &["dim"]),
];

#[test]
fn every_fill_behind_text_reads() {
    let t = shipped();
    let mut low = Vec::new();
    for (fill, roles) in FILLS_BEHIND_TEXT {
        let ground = *t.game_colors.get(fill).unwrap_or_else(|| panic!("{fill} is not set"));
        for role in roles {
            let least = if *role == "dim" { 3.0 } else { 4.5 };
            let r = t.contrast(t.role(role).unwrap(), ground);
            println!("{role} on {fill}: {r:.2}");
            if r < least {
                low.push(format!("{role} on {fill}: {r:.2} < {least}"));
            }
        }
    }
    let dim = t.contrast(t.game("story_dim", [0; 4]), t.stage);
    println!("story_dim on stage: {dim:.2}");
    assert!(dim >= 4.5, "the story's dim on its stage: {dim:.2}");
    assert!(low.is_empty(), "{low:?}");
}

/// The theme sets every role but the two the spec leaves at today's (LOOK1 Q10), so no screen keeps a navy default.
#[test]
fn the_game_theme_sets_every_role_but_foe_and_good() {
    let file: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(game_dir().join("assets/ui/theme.json")).unwrap()).unwrap();
    let colors = file["colors"].as_object().unwrap();
    for role in theme::ROLES {
        assert_eq!(colors.contains_key(*role), !["foe", "good"].contains(role), "{role}");
    }
}

/// The two things LOOK1-E1B left open: hovering the desk's grip shows, and the ruler's playing label reads on its fill.
#[test]
fn the_grip_shows_its_hover_and_the_playing_label_reads() {
    let t = shipped();
    assert_ne!(t.grip, t.focus, "the grip's hover colour is `focus`");
    assert!(contrast_pairs().contains(&("focus", "playing", 4.5)));
    assert!(t.contrast(t.focus, t.playing) >= 4.5);
    // The block being played stands apart from the ruler it sits on, by its edge.
    assert!(t.contrast(t.playing_edge, t.ruler) >= 3.0);
}

/// LAYOUT720-E3: the kit's widgets in this game's frames and faces, drawn on the native grid with their numbers doubled, are
/// their old picture: to the bit where no division rounds on the grid. `AEGIS_SHOT_DIR=<folder>` writes the pictures.
#[test]
fn the_shipped_frames_and_faces_draw_the_kit_the_same_on_the_native_grid() {
    use aegis_ui::canvas::{Canvas, Image, Rect};
    use aegis_ui::painter::{Fonts, Grid, View};
    use aegis_ui::shot_diff::shot_diff;
    use aegis_ui::ui::{draw_tip, tip, Gfx, Kit, Ui};
    theme::set_theme(shipped());
    let font = std::fs::read(game_dir().join("../../engine/assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
    let shots = std::env::var("AEGIS_SHOT_DIR").ok();
    for (pw, ph) in [(1280, 720), (1920, 1080), (1366, 768)] {
        let picture = |grid: Grid, hovered: bool| {
            let mut g = Gfx { canvas: Canvas::new(pw, ph), view: View::fit(pw as u32, ph as u32).for_grid(grid), fonts: Fonts::new(&font), kit: Kit::default() };
            let u = grid.g();
            let r = |x: i32, y: i32, w: i32, h: i32| Rect::new(x * u, y * u, w * u, h * u);
            g.canvas.clear(aegis_ui::theme::th().bg);
            {
                let mouse = if hovered { (30 * u, 100 * u) } else { (-1, -1) };
                let mut ui: Ui<()> = Ui::new(&mut g, mouse, true);
                let c = ui.frame(r(5, 5, 300, 70), "A frame's title", false);
                ui.p.fill(Rect::new(15 * u, c, 30 * u, 3 * u), ui.t.text);
                ui.frame_body("ornate", r(320, 15, 300, 60));
                ui.frame_body("ornate_full", r(320, 95, 300, 60));
                ui.button("a", r(10, 92, 80, 16), "Hovered", (), true);
                ui.button("b", r(100, 92, 80, 16), "Idle", (), true);
                ui.button("c", r(190, 92, 80, 16), "Disabled", (), false);
                ui.tab("d", r(10, 114, 60, 18), "Open", (), true);
                ui.tab("e", r(74, 114, 60, 18), "Shut", (), false);
                ui.card("f", r(10, 140, 140, 34), &[("A card".into(), ui.t.text), ("Its second line".into(), ui.t.dim)], Some(()), false, true, Vec::new());
                ui.card("g", r(160, 140, 140, 34), &[("Selected".into(), ui.t.text)], Some(()), true, true, Vec::new());
                let i = ui.inset(r(10, 180, 140, 40), false);
                ui.para(i.x, i.y, i.w, "Inset text", ui.m().small, ui.t.text);
                let i = ui.inset(r(160, 180, 140, 40), true);
                ui.para(i.x, i.y, i.w, "Parchment text", ui.m().small, ui.t.ink);
                ui.header(10 * u, 230 * u, "A header");
                ui.badge(r(150, 230, 24, 12), "42", "card");
                ui.push("w", "row", "", r(320, 200, 100, 40), true, false, None, tip("A card's title", "Its body, long enough to wrap onto a line or two."));
                let w = std::mem::take(&mut ui.widgets);
                draw_tip(&mut ui.p, &w, (330 * u, 210 * u));
            }
            Image { w: g.canvas.w, h: g.canvas.h, px: g.canvas.px }
        };
        let view = View::fit(pw as u32, ph as u32);
        for hovered in [false, true] {
            let (old, native) = (picture(Grid::Old, hovered), picture(Grid::Native, hovered));
            if let Some(dir) = &shots {
                for (name, img) in [("old", &old), ("native", &native)] {
                    let path = Path::new(dir).join(format!("game_{pw}_{hovered}_{name}.png"));
                    let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(path).unwrap()), img.w as u32, img.h as u32);
                    enc.set_color(png::ColorType::Rgba);
                    enc.set_depth(png::BitDepth::Eight);
                    enc.write_header().unwrap().write_image_data(&img.px).unwrap();
                }
            }
            // The card's measured width rounds up in units, so the tip's far edge may land one away.
            let diff = shot_diff(&old, &native, view.for_grid(Grid::Native).s).unwrap();
            assert!(diff.passed(), "{pw}x{ph} hovered {hovered}: {diff}");
        }
    }
    theme::set_theme(Theme::default());
}

/// `mono` is the static Bold the user picked: fontdue draws a variable font at its default weight, so a file with an
/// `fvar` table or a weight class other than 700 would draw the wrong face.
#[test]
fn mono_is_a_static_bold() {
    let t = shipped();
    let bytes = t.face(Typeface::Mono).bytes.clone().expect("the mono file");
    let tables = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
    let table = |tag: &[u8; 4]| (0..tables).map(|i| 12 + 16 * i).find(|o| &bytes[*o..*o + 4] == tag).map(|o| u32::from_be_bytes([bytes[o + 8], bytes[o + 9], bytes[o + 10], bytes[o + 11]]) as usize);
    assert!(table(b"fvar").is_none(), "a variable font draws at its default weight");
    let os2 = table(b"OS/2").expect("an OS/2 table");
    assert_eq!(u16::from_be_bytes([bytes[os2 + 4], bytes[os2 + 5]]), 700);
}
