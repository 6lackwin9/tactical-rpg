//! DIORAMA1B (D5, contract 8.1): painted body sheets. A `clips.json` may name the frame row the feet stand on (`foot`)
//! and mark the sheet `painted`; `Clips::parse` reads and checks both, a painted sheet draws without the palette swap
//! (`Art` loads it as painted even when a palette is asked for), and a painted sheet with only `idle` plays `idle` for
//! every missing clip (SPRITE1-L). Expectations follow from the clips.json rules and the fixture's own colours.

use aegis_core::assets::{Clips, Layers};
use aegis_core::scratch::Scratch;
use aegis_ui::art::Art;
use serde_json::json;
use std::path::Path;

const W: u32 = 12;
const H: u32 = 10;
/// The fixture's body colour (the palette's `main` slot) and the palette file's swap for it.
const MAIN: [u8; 3] = [10, 10, 200];
const SWAP: [u8; 3] = [200, 30, 30];

fn write_png(path: &Path, w: u32, h: u32, px: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let file = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(px).unwrap();
}

/// A scratch root with body `units/hero/<variant>`: one `idle` row of 2 frames, each filled with `MAIN` over rows
/// 2..=`H - 3` (the feet on row `H - 3`), a palette `main` and a palette file `red` swapping it; `painted` as given.
fn painted_root(name: &str, painted: bool) -> Scratch {
    let root = Scratch::vacant(&format!("ui_painted_{name}"));
    let (w, h) = (2 * W, H);
    let mut px = vec![0u8; (w * h * 4) as usize];
    for y in 2..=H - 3 {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            px[i..i + 4].copy_from_slice(&[MAIN[0], MAIN[1], MAIN[2], 255]);
        }
    }
    let dir = root.join("units/hero/base");
    write_png(&dir.join("sheet.png"), w, h, &px);
    let clips = json!({"frame": [W, H], "foot": H - 3, "painted": painted, "palette": {"main": "#0a0ac8"}, "clips": [{"id": "idle", "frames": 2, "ms": [100, 100]}]});
    std::fs::write(dir.join("clips.json"), clips.to_string()).unwrap();
    std::fs::create_dir_all(dir.join("palettes")).unwrap();
    std::fs::write(dir.join("palettes/red.json"), json!({"slots": {"main": "#c81e1e"}}).to_string()).unwrap();
    root
}

fn centre(art: &mut Art, palette: &str) -> [u8; 3] {
    let img = art.body_frame("hero/base", palette, "idle", 0).expect("the frame loads");
    let i = (((H as i32 / 2) * img.w + img.w / 2) * 4) as usize;
    [img.px[i], img.px[i + 1], img.px[i + 2]]
}

#[test]
fn clips_json_reads_its_foot_row_and_painted_flag() {
    let c = Clips::parse(&json!({"frame": [112, 96], "foot": 93, "painted": true, "clips": [{"id": "idle", "frames": 1, "ms": [100]}]})).unwrap();
    assert_eq!((c.frame, c.foot, c.painted), ((112, 96), Some(93), true));
    let plain = Clips::parse(&json!({"frame": [48, 48], "clips": [{"id": "idle", "frames": 1, "ms": [100]}]})).unwrap();
    assert_eq!((plain.foot, plain.painted), (None, false), "an old sheet: no foot row, not painted");
    let past = Clips::parse(&json!({"frame": [48, 48], "foot": 48, "clips": [{"id": "idle", "frames": 1, "ms": [100]}]})).unwrap_err();
    assert!(past.iter().any(|p| p.contains("foot must be a row of the frame (0 to 47)")), "{past:?}");
    let flag = Clips::parse(&json!({"frame": [48, 48], "painted": "yes", "clips": [{"id": "idle", "frames": 1, "ms": [100]}]})).unwrap_err();
    assert!(flag.iter().any(|p| p.contains("painted must be true or false")), "{flag:?}");
}

#[test]
fn a_painted_sheet_draws_without_the_palette_swap() {
    let root = painted_root("on", true);
    let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
    assert_eq!(centre(&mut art, ""), MAIN);
    assert_eq!(centre(&mut art, "red"), MAIN, "painted: the palette file is not applied");
    // The same sheet not painted takes the swap (the rule the flag turns off).
    let root = painted_root("off", false);
    let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
    assert_eq!(centre(&mut art, "red"), SWAP, "not painted: the palette file swaps main");
}

#[test]
fn a_painted_sheet_plays_idle_for_every_missing_clip() {
    let root = painted_root("idle", true);
    let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
    let clips = art.clips("hero/base").expect("the body loads").clone();
    for clip in ["walk", "attack", "hurt", "death"] {
        let f = clips.frame_at(clip, 150).expect("a frame");
        assert_eq!((f.clip.as_str(), f.row), ("idle", 0), "{clip} falls back to idle");
        assert!(!art.has_clip("hero/base", clip), "{clip} is not its own clip");
    }
}
