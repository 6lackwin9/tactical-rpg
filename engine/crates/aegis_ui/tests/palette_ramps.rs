//! PR7-25 (`STYLE-ENGINE1` 1, S2 sprite ramps): a palette file may name a slot's `_lo` and `_hi` shades besides its
//! base, and the body's swap (`aegis_core::assets::palette_pairs` into `Image::swap_colours`, via `Art`) swaps them
//! too; a palette that names only bases swaps only bases, as before. The shade suffixes do not count toward the
//! 16-slot cap. Expectations follow from the rule on a scratch body; the review shot (only with `AEGIS_REVIEW_DIR`)
//! recolours a kit body that names its shades.

use aegis_core::assets::{self, palette_pairs, parse_palette, shade_base, Clips, Layers, PALETTE_SLOTS_MAX};
use aegis_core::scratch::Scratch;
use aegis_ui::art::Art;
use aegis_ui::canvas::Image;
use serde_json::{json, Value};
use std::path::Path;

const F: u32 = 8;
const BASE: [u8; 3] = [0x90, 0x30, 0x30];
const LO: [u8; 3] = [0x60, 0x20, 0x20];
const HI: [u8; 3] = [0xc0, 0x50, 0x50];
const OTHER: [u8; 3] = [0x20, 0x40, 0x80];
const NEW_BASE: [u8; 3] = [0x4b, 0x55, 0x63];
const NEW_LO: [u8; 3] = [0x2e, 0x34, 0x3e];
const NEW_HI: [u8; 3] = [0x6a, 0x72, 0x7c];

fn hex(c: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

fn write_png(path: &Path, w: u32, h: u32, px: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let file = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(px).unwrap();
}

/// The fixture frame: two rows each of base, shadow, light and an unrelated colour.
fn rows() -> [[u8; 3]; 4] {
    [BASE, LO, HI, OTHER]
}

/// A scratch root with body `hero/base` (one idle frame of `rows()`); `shades`: its clips.json names `main_lo` and
/// `main_hi` besides `main`. Palette files: `ramp` (base and both shades) and `base_only` (the base).
fn body_root(tag: &str, shades: bool) -> Scratch {
    let root = Scratch::vacant(&format!("ui_palette_ramps_{tag}"));
    let mut px = Vec::new();
    for c in rows() {
        for _ in 0..2 * F {
            px.extend_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
    let dir = root.join("units/hero/base");
    write_png(&dir.join("sheet.png"), F, F, &px);
    let mut palette = json!({"main": hex(BASE), "cloth": hex(OTHER)});
    if shades {
        palette["main_lo"] = json!(hex(LO));
        palette["main_hi"] = json!(hex(HI));
    }
    let clips = json!({"frame": [F, F], "palette": palette, "clips": [{"id": "idle", "frames": 1, "ms": [100]}]});
    std::fs::write(dir.join("clips.json"), clips.to_string()).unwrap();
    std::fs::create_dir_all(dir.join("palettes")).unwrap();
    let files = [
        ("ramp", json!({"slots": {"main": hex(NEW_BASE), "main_lo": hex(NEW_LO), "main_hi": hex(NEW_HI)}})),
        ("base_only", json!({"slots": {"main": hex(NEW_BASE)}})),
    ];
    for (id, v) in files {
        std::fs::write(dir.join(format!("palettes/{id}.json")), v.to_string()).unwrap();
    }
    root
}

/// The colour of each fixture row in `img` (its first pixel).
fn row_colours(img: &Image) -> Vec<[u8; 3]> {
    (0..4).map(|r| {
        let i = ((r * 2 * img.w) * 4) as usize;
        [img.px[i], img.px[i + 1], img.px[i + 2]]
    })
    .collect()
}

fn frame(root: &Scratch, palette: &str) -> Image {
    let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
    art.body_frame("hero/base", palette, "idle", 0).expect("the fixture body loads")
}

#[test]
fn shade_names_belong_to_their_base() {
    assert_eq!(shade_base("main_lo"), Some("main"));
    assert_eq!(shade_base("main_hi"), Some("main"));
    assert_eq!(shade_base("main"), None);
    assert_eq!(shade_base("_lo"), None, "a bare suffix names no base");
    assert_eq!(shade_base("main_low"), None);
}

#[test]
fn a_ramp_palette_swaps_the_base_and_both_shades() {
    let root = body_root("ramp", true);
    assert_eq!(row_colours(&frame(&root, "")), rows(), "no palette: the sheet as drawn");
    assert_eq!(row_colours(&frame(&root, "ramp")), [NEW_BASE, NEW_LO, NEW_HI, OTHER], "base, shadow and light take the file's ramp; other slots stay");
}

#[test]
fn a_base_only_palette_is_unchanged_by_ramps() {
    let root = body_root("base_only", true);
    let got = frame(&root, "base_only");
    assert_eq!(row_colours(&got), [NEW_BASE, LO, HI, OTHER], "only the base swaps; the body's shades stay as drawn");
    // The same as the pre-ramp swap: only the base pair.
    let mut before = frame(&root, "");
    before.swap_colours(&[(BASE, NEW_BASE)]);
    assert_eq!(got, before);
}

#[test]
fn a_shade_the_body_does_not_name_swaps_nothing() {
    let root = body_root("no_shades", false);
    assert_eq!(row_colours(&frame(&root, "ramp")), [NEW_BASE, LO, HI, OTHER], "the body names no shades: only its base swaps");
}

#[test]
fn palette_pairs_follow_the_file_and_the_body() {
    let body = Clips::parse(&json!({"frame": [F, F], "palette": {"main": hex(BASE), "main_lo": hex(LO), "main_hi": hex(HI)}, "clips": [{"id": "idle", "frames": 1, "ms": [100]}]})).unwrap().palette;
    let file = parse_palette(&json!({"slots": {"main_hi": hex(NEW_HI), "main": hex(NEW_BASE), "trim": "#000000"}})).unwrap();
    assert_eq!(palette_pairs(&body, &file), [(HI, NEW_HI), (BASE, NEW_BASE)], "in the file's order; a slot the body lacks is skipped");
}

#[test]
fn shades_do_not_count_toward_the_slot_cap() {
    let mut full = serde_json::Map::new();
    for i in 0..PALETTE_SLOTS_MAX {
        for suffix in ["", "_lo", "_hi"] {
            full.insert(format!("s{i}{suffix}"), json!("#102030"));
        }
    }
    assert!(parse_palette(&json!({"slots": Value::Object(full.clone())})).is_ok(), "{PALETTE_SLOTS_MAX} bases, each with both shades");
    full.insert("one_more".into(), json!("#102030"));
    let err = parse_palette(&json!({"slots": Value::Object(full)})).unwrap_err();
    assert!(err.iter().any(|p| p.contains(&format!("at most {PALETTE_SLOTS_MAX} slots"))), "{err:?}");
}

/// Review shot (not a check; only with `AEGIS_REVIEW_DIR`): the first kit body whose clips.json names a base and its
/// `_lo` shade, at 6x: as drawn, recoloured with a base-only palette and with the full ramp (iron, bible 3 shadow and
/// light), side by side in `sprite_ramp_before_after.png`.
#[test]
fn review_shot_sprite_ramp() {
    let Ok(out) = std::env::var("AEGIS_REVIEW_DIR") else { return };
    let kit = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../kits/rpg/assets");
    let layers = Layers::of(vec![kit.clone()]);
    let body = layers
        .bodies()
        .into_iter()
        .find(|b| {
            assets::read_json(&layers, &assets::body_file(b, "clips.json")).and_then(|v| Clips::parse(&v).ok()).is_some_and(|c| c.palette.keys().any(|k| shade_base(k).is_some_and(|base| c.palette.contains_key(base))))
        })
        .expect("a kit body naming a shade of one of its slots");
    let clips = Clips::parse(&assets::read_json(&layers, &assets::body_file(&body, "clips.json")).unwrap()).unwrap();
    let base = clips.palette.keys().find(|k| shade_base(k).is_some_and(|b| clips.palette.contains_key(b))).map(|k| shade_base(k).unwrap().to_string()).unwrap();
    let scratch = Scratch::vacant("ui_palette_ramps_shot");
    let dir = scratch.join(assets::body_file(&body, "palettes"));
    std::fs::create_dir_all(&dir).unwrap();
    let mut ramp = json!({base.clone(): hex(NEW_BASE)});
    std::fs::write(dir.join("base_only.json"), json!({"slots": ramp.clone()}).to_string()).unwrap();
    for (suffix, c) in [("_lo", NEW_LO), ("_hi", NEW_HI)] {
        if clips.palette.contains_key(&format!("{base}{suffix}")) {
            ramp[format!("{base}{suffix}")] = json!(hex(c));
        }
    }
    std::fs::write(dir.join("ramp.json"), json!({"slots": ramp}).to_string()).unwrap();
    let mut art = Art::layered(Layers::of(vec![scratch.to_path_buf(), kit]));
    let shots: Vec<Image> = ["", "base_only", "ramp"].iter().map(|p| art.body_frame(&body, p, "idle", 0).expect("the body's idle frame")).collect();
    let (s, w, h) = (6, shots[0].w, shots[0].h);
    let (cw, ch) = (w * s * 3, h * s);
    let mut px = vec![0u8; (cw * ch * 4) as usize];
    for (n, img) in shots.iter().enumerate() {
        for y in 0..ch {
            for x in 0..w * s {
                let si = (((y / s) * w + x / s) * 4) as usize;
                let di = ((y * cw + n as i32 * w * s + x) * 4) as usize;
                let a = img.px[si + 3] as u32;
                for k in 0..3 {
                    px[di + k] = ((img.px[si + k] as u32 * a + 0x1a * (255 - a)) / 255) as u8;
                }
                px[di + 3] = 255;
            }
        }
    }
    std::fs::create_dir_all(&out).unwrap();
    write_png(&Path::new(&out).join("sprite_ramp_before_after.png"), cw as u32, ch as u32, &px);
    println!("ramp shot: body {body}, slot {base}");
}
