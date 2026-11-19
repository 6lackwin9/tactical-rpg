//! ENGINE-2 (PERF-STEP1): the painter's benchmarks (`aegis_ui::perf::bench`): a whole-canvas fill, 64 panels and 40 lines
//! of text, each timed `runs` times on a canvas of each size, with the calibration workload before and after. One JSON
//! line per size. Run it in a release build: `tools/fc.sh bench [size=WxH,...] [runs=N]`.
//!
//!   ui_bench [sizes=1280x720,1920x1080] [runs=50]

use aegis_ui::perf;
use serde_json::json;

fn main() -> anyhow::Result<()> {
    let args: Vec<(String, String)> = std::env::args().skip(1).filter_map(|a| a.split_once('=').map(|(k, v)| (k.to_string(), v.to_string()))).collect();
    let arg = |k: &str, d: &str| args.iter().find(|(a, _)| a == k).map_or(d.to_string(), |(_, v)| v.clone());
    let runs: usize = arg("runs", "50").parse().ok().filter(|n| (1..=10_000).contains(n)).ok_or_else(|| anyhow::anyhow!("runs is a whole number from 1 to 10000"))?;
    let font = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf");
    let ttf = std::fs::read(&font)?;
    for size in arg("sizes", "1280x720,1920x1080").split(',') {
        let (w, h) = size.split_once('x').and_then(|(w, h)| Some((w.parse::<i32>().ok()?, h.parse::<i32>().ok()?))).filter(|(w, h)| (64..=7680).contains(w) && (36..=4320).contains(h)).ok_or_else(|| anyhow::anyhow!("size '{size}' is WxH, 64x36 to 7680x4320"))?;
        let before = perf::calib();
        let benches = perf::bench(&ttf, w, h, runs);
        let after = perf::calib();
        let mut line = json!({"size": size, "runs": runs, "build": if cfg!(debug_assertions) { "dev" } else { "release" }, "calib_before": (before * 1000.0).round() / 1000.0, "calib_after": (after * 1000.0).round() / 1000.0, "steady": perf::steady(before, after)});
        for b in &benches {
            line[b.name] = b.stats.to_json();
        }
        println!("{line}");
    }
    Ok(())
}
