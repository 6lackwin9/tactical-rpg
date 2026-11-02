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
            _ => (16, 4),
        };
        assert_eq!((f.img.w, f.img.h), (size, size), "{role} size");
        assert_eq!(f.nine.slice, [slice; 4], "{role} slice");
        assert!(if role == "ornate" { f.nine.scale >= 8 } else { f.nine.scale == 2 }, "{role} scale");
        assert_eq!(f.nine.reduce, role == "ornate", "{role}: only the high-resolution frame averages");
        let ink = if role == "parchment" { "ink" } else { "text" };
        assert_eq!(f.nine.ink, ink, "{role} ink");
        for (i, px) in f.img.px.chunks(4).enumerate() {
            // The ornate corners are the picked art (decision "Expression sheets: scale only and aligned cells": no palette pass): the rule holds for its
            // edges and centre, where a straddling frame's bands are see-through past their rows.
            let (x, y) = (i as i32 % size, i as i32 / size);
            if role == "ornate" && (x < slice || x >= size - slice) && (y < slice || y >= size - slice) {
                continue;
            }
            if role == "ornate" && px[3] == 0 {
                continue;
            }
            assert!(palette.contains(&[px[0], px[1], px[2]]), "{role}: colour {:02x}{:02x}{:02x} is not in the palette", px[0], px[1], px[2]);
            assert!([255, 0xf0, 0xe0].contains(&px[3]), "{role}: alpha {}", px[3]);
        }
    }
}

/// `ornate`'s corners are the picked art, not a few flat hues: `gen_art.py` must not fall back to drawing them.
#[test]
fn the_ornate_corners_are_the_picked_art() {
    let t = shipped();
    let f = t.frame("ornate").expect("the ornate frame");
    let slice = f.nine.slice[0];
    let mut colours = std::collections::HashSet::new();
    for y in 0..slice {
        for x in 0..slice {
            let i = ((y * f.img.w + x) * 4) as usize;
            colours.insert([f.img.px[i], f.img.px[i + 1], f.img.px[i + 2]]);
        }
    }
    assert!(colours.len() > 1000, "only {} colours in the corner", colours.len());
    assert!(f.nine.corner_arm.is_some_and(|m| (0.2..=0.25).contains(&m)), "each arm 20-25 % of its own edge");
    assert!(f.nine.straddle, "half in, half out");
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

/// The shipped frame on a panel at every window size, drawn alone: its arms reach no further than a quarter of each edge
/// (the art's natural end may stop them short), centred across their whole section on the boundary, ending on their
/// finish near the line, and the line's rows fall on both sides of the boundary.
#[test]
fn the_ornate_frame_straddles_with_finished_arms_at_every_window_size() {
    use aegis_ui::canvas::{Canvas, Rect};
    use aegis_ui::painter::{Fonts, Painter, View};
    let t = shipped();
    let f = t.frame("ornate").unwrap();
    let font = std::fs::read(game_dir().join("../../engine/assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
    let mut fonts = Fonts::new(&font);
    let slice = f.nine.slice[0];
    // The corners alone: everything else in the file see-through.
    let mut only = (*f.img).clone();
    let n = only.w;
    for (i, px) in only.px.chunks_mut(4).enumerate() {
        let (x, y) = (i as i32 % n, i as i32 / n);
        if !((x < slice || x >= n - slice) && (y < slice || y >= n - slice)) {
            px.copy_from_slice(&[0, 0, 0, 0]);
        }
    }
    let ground = [0, 255, 0, 255];
    let r = Rect::new(200, 120, 180, 84);
    for (ww, wh) in [(960, 540), (1280, 720), (1920, 1080), (3840, 2160)] {
        let mut c = Canvas::new(ww, wh);
        c.clear(ground);
        let mut p = Painter { c: &mut c, v: View::fit(ww as u32, wh as u32), fonts: &mut fonts };
        let s = p.v.s;
        let pr = p.v.rect(r);
        p.nine(&only, &f.nine, r);
        let drawn = |c: &Canvas, x: i32, y: i32| c.px[((y * c.w + x) * 4 + 1) as usize] != 255;
        // How far each arm reaches from the corner point (the left half and top half of the panel hold the near corners).
        let cross = (14.0 * s) as i32;
        let along_x = |x: i32| (pr.y - cross..pr.y + cross).filter(|y| drawn(p.c, x, *y)).count() as i32;
        let along_y = |y: i32| (pr.x - cross..pr.x + cross).filter(|x| drawn(p.c, *x, y)).count() as i32;
        let end_x = (pr.x..pr.x + pr.w / 2).rfind(|x| along_x(*x) > 0).unwrap();
        let end_y = (pr.y..pr.y + pr.h / 2).rfind(|y| along_y(*y) > 0).unwrap();
        let (share_x, share_y) = ((end_x - pr.x) as f32 / pr.w as f32, (end_y - pr.y) as f32 / pr.h as f32);
        println!("{ww}: arms cover {share_x:.3} across, {share_y:.3} down");
        assert!((0.12..=0.25).contains(&share_x), "{ww}: the horizontal arm covers {share_x:.3} of its edge");
        assert!((0.12..=0.25).contains(&share_y), "{ww}: the vertical arm covers {share_y:.3} of its edge");
        // Centred across the section, past the elbow: the middle of the 10th and 90th percentiles of the drawn pixels.
        let centre = |counts: Vec<i32>| {
            let total: i32 = counts.iter().sum();
            let at = |share: f32| {
                let mut sum = 0;
                counts.iter().position(|c| { sum += c; sum as f32 >= share * total as f32 }).unwrap() as f32
            };
            (at(0.1) + at(0.9)) / 2.0 - cross as f32
        };
        let top: Vec<i32> = (-cross..cross).map(|d| (pr.x + (14.0 * s) as i32..end_x).filter(|x| drawn(p.c, *x, pr.y + d)).count() as i32).collect();
        let left: Vec<i32> = (-cross..cross).map(|d| (pr.y + (8.0 * s) as i32..end_y - (4.0 * s) as i32).filter(|y| drawn(p.c, pr.x + d, *y)).count() as i32).collect();
        for (name, counts) in [("top", top), ("left", left)] {
            let off = centre(counts) / s;
            assert!(off.abs() <= 2.0, "{ww}: the {name} arm's centre is {off:.2} logical px off the boundary");
        }
        // The ends lie near the line: the last drawn pixels of the top arm within 3 logical px of it, and the foot's ends
        // narrower than the leg.
        let tip = (pr.y - cross..pr.y + cross).filter(|y| drawn(p.c, end_x, *y)).map(|y| (y - pr.y).abs()).min().unwrap();
        assert!(tip as f32 <= 3.0 * s, "{ww}: the top arm ends {tip} px from the line");
        let (leg, foot) = (along_y(pr.y + (6.0 * s) as i32), along_y(end_y));
        assert!(foot < leg, "{ww}: the left arm's end {foot} of its leg {leg}");
        // On black, so a row the line half covers still reads red.
        let mut c = Canvas::new(ww, wh);
        c.clear([0, 0, 0, 255]);
        let mut p = Painter { c: &mut c, v: View::fit(ww as u32, wh as u32), fonts: &mut fonts };
        p.nine(&f.img, &f.nine, r);
        let mid = pr.x + pr.w / 2;
        let reddish: Vec<i32> = (pr.y - 8..pr.y + 8).filter(|y| {
            let i = ((y * ww + mid) * 4) as usize;
            let q = &p.c.px[i..i + 3];
            q[0] as i32 > q[1] as i32 + 25
        }).collect();
        assert!(reddish.first().is_some_and(|y| *y < pr.y) && reddish.last().is_some_and(|y| *y >= pr.y), "{ww}: the line's rows {reddish:?} about {}", pr.y);
    }
}

#[test]
fn the_faces_are_the_picked_fonts() {
    let t = shipped();
    for (face, file) in [(Typeface::Body, "fonts/OpenSans-SemiBold.ttf"), (Typeface::Display, "fonts/AlegreyaSC-Bold.ttf"), (Typeface::Mono, "fonts/JetBrainsMono-wght.ttf")] {
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
