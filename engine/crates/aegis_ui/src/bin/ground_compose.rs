//! The ground composer for tools (`docs/specs/REGIONGROUND1_ENGINE.md` section 6), so a review board's picture is the
//! game's: the one implementation, `aegis_ui::ground`, a JSON job to a PNG. The job's fields: `ground::compose_file`.
//!
//!   cargo run -q -p aegis_ui --bin ground_compose -- JOB.json OUT.png
//!   cargo run -q --release -p aegis_ui --bin ground_compose -- --timing
//!
//! `--timing` prints what the spec's budgets ask, for a 48 x 30, 8-layer job in both cell forms: compose, the physical
//! copy at 720p, 1080p and 4K, a frame's blit and the bytes the cache holds.

use aegis_ui::canvas::{Canvas, Image, Rect};
use aegis_ui::ground::{self, GroundJob, GroundKey, GroundLayer, GroundMark, PlacedPatch, MARKS_PER_100_MAX, TEX_CELLS, TILTED_CELL};
use aegis_ui::painter::{Fonts, Painter, View};
use std::path::Path;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let run = match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["--timing"] => {
            timing();
            Ok(())
        }
        [job, out] => ground::compose_file(Path::new(job), Path::new(out)).map(|c| println!("{} x {} art px, composed in {:.1} ms", c.w, c.h, c.compose_ms)),
        _ => {
            eprintln!("usage: ground_compose JOB.json OUT.png | ground_compose --timing");
            std::process::exit(2);
        }
    };
    if let Err(e) = run {
        eprintln!("ground_compose: {e:#}");
        std::process::exit(1);
    }
}

/// The largest shipped map's size, every layer used and many borders: a coarse hash picks each 3 x 3 block's ground.
fn budget_cells(cols: i32, rows: i32, layers: u64) -> Vec<u8> {
    (0..rows).flat_map(|y| (0..cols).map(move |x| (ground::splitmix64(((y / 3) as u64) << 16 | (x / 3) as u64) % layers) as u8)).collect()
}

fn timing() {
    const LAYERS: u64 = 8;
    let (cols, rows) = (48, 30);
    for (name, cell) in [("square", (ground::CELL_PX, ground::CELL_PX)), ("tilted", TILTED_CELL)] {
        let tex: Vec<Image> = (0..LAYERS).map(|i| ground::two_tone(TEX_CELLS * cell.0, TEX_CELLS * cell.1, [90 + i as u8 * 8, 80, 70, 255], [84 + i as u8 * 8, 75, 66, 255], i)).collect();
        let layers: Vec<GroundLayer> = tex.iter().enumerate().map(|(i, t)| GroundLayer { tex: Some(t), tone: 0.13, wear: 0.1, strays: true, sunken: i == 6, ..GroundLayer::flat(i as i32, [0, 0, 0, 255]) }).collect();
        let cells = budget_cells(cols, rows, LAYERS);
        let blocked = vec![false; cells.len()];
        let job = GroundJob { seed: 7, cols, rows, cell, cells: &cells, blocked: &blocked, layers: &layers, placed: &[] };
        let t0 = Instant::now();
        let key = GroundKey::of("timing", &job);
        let key_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let (mut best, mut worst) = (f64::MAX, 0.0f64);
        let mut art = ground::compose(&job);
        for _ in 0..10 {
            let t0 = Instant::now();
            art = ground::compose(&job);
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            (best, worst) = (best.min(ms), worst.max(ms));
        }
        println!("{name}: {cols} x {rows}, {LAYERS} layers, art {} x {} ({:.1} MB): compose {best:.1} to {worst:.1} ms (10 runs), key {key_ms:.2} ms", art.w, art.h, art.px.len() as f64 / 1e6);
        // The same with patches, marks and placed patches on every layer: many borders, then one open ground (its whole
        // area holds candidates and tries, the most a patch pass can read).
        let mark = ground::soft_ellipse(5, 3, [30, 26, 22, 255], 255);
        let dressed: Vec<GroundLayer> = layers.iter().map(|l| GroundLayer { patches: 0.2, marks: vec![GroundMark { img: &mark }], marks_per_100: MARKS_PER_100_MAX, ..l.clone() }).collect();
        let open = vec![0u8; cells.len()];
        let placed: Vec<PlacedPatch> = (0..16).map(|i| PlacedPatch { x: 1 + i * 3 % (cols - 8), y: 1 + i * 2 % (rows - 8), w: 6, h: 6, strength: 0.12 }).collect();
        for (what, cells, placed) in [("many borders", &cells, &[][..]), ("one open ground, 16 placed", &open, &placed[..])] {
            let job = GroundJob { seed: 7, cols, rows, cell, cells, blocked: &blocked, layers: &dressed, placed };
            let (mut best, mut worst) = (f64::MAX, 0.0f64);
            for _ in 0..10 {
                let t0 = Instant::now();
                std::hint::black_box(ground::compose(&job));
                let ms = t0.elapsed().as_secs_f64() * 1000.0;
                (best, worst) = (best.min(ms), worst.max(ms));
            }
            let groups = ground::patch_groups(&job);
            println!("  with patches and marks, {what}: compose {best:.1} to {worst:.1} ms (10 runs), {} of {} patches kept", groups.iter().filter(|g| g.kept).count(), groups.len());
        }
        for (win, (pw, ph)) in [("1280 x 720", (1280u32, 720u32)), ("1920 x 1080", (1920, 1080)), ("3840 x 2160", (3840, 2160))] {
            let v = View::fit(pw, ph);
            let r = Rect::new(0, 0, cols * 16, rows * cell.1 * 16 / cell.0);
            let pr = v.rect(r);
            let (mut copy, mut copy_worst) = (f64::MAX, 0.0f64);
            let mut bytes = 0;
            for _ in 0..10 {
                let t0 = Instant::now();
                let px = ground::expand(&art, pr.w, pr.h);
                let ms = t0.elapsed().as_secs_f64() * 1000.0;
                (copy, copy_worst) = (copy.min(ms), copy_worst.max(ms));
                bytes = px.len();
            }
            // A whole frame's blit through `draw`, the window covered by the map.
            let mut c = Canvas::new(pw as i32, ph as i32);
            let mut fonts = Fonts::new(&std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).expect("the engine font"));
            let mut p = Painter { c: &mut c, v, fonts: &mut fonts };
            ground::draw(&mut p, key.clone(), r, || art.clone());
            let (mut blit, mut blit_worst) = (f64::MAX, 0.0f64);
            for _ in 0..20 {
                let t0 = Instant::now();
                ground::draw(&mut p, key.clone(), r, || art.clone());
                let ms = t0.elapsed().as_secs_f64() * 1000.0;
                (blit, blit_worst) = (blit.min(ms), blit_worst.max(ms));
            }
            println!("  {win}: copy {} x {} ({:.1} MB) {copy:.1} to {copy_worst:.1} ms, frame blit {blit:.2} to {blit_worst:.2} ms, cache holds {:.1} MB", pr.w, pr.h, bytes as f64 / 1e6, ground::ground_held_bytes() as f64 / 1e6);
        }
    }
}
