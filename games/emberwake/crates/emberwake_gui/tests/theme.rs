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
            "ornate" => (48, 16),
            _ => (16, 4),
        };
        assert_eq!((f.img.w, f.img.h), (size, size), "{role} size");
        assert_eq!(f.nine.slice, [slice; 4], "{role} slice");
        assert_eq!(f.nine.scale, 2, "{role} scale");
        let ink = if role == "parchment" { "ink" } else { "text" };
        assert_eq!(f.nine.ink, ink, "{role} ink");
        for px in f.img.px.chunks(4) {
            assert!(palette.contains(&[px[0], px[1], px[2]]), "{role}: colour {:02x}{:02x}{:02x} is not in the palette", px[0], px[1], px[2]);
            assert!([255, 0xf0, 0xe0].contains(&px[3]), "{role}: alpha {}", px[3]);
        }
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
