//! The balance sim against Godot's (games/emberwake/parity/sim.sh): every recorded command line runs on the same
//! exported catalog and worlds, and must print the same lines. PARITY_SIM=<n,n> runs only those recordings.

use emberwake_world::Data;
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../parity/out")
}

#[test]
fn sim_prints_what_godot_printed() {
    let dir = out_dir().join("sim");
    let Ok(read) = std::fs::read_dir(&dir) else {
        eprintln!("no parity/out/sim: run games/emberwake/parity/sim.sh");
        return;
    };
    let only: Option<Vec<String>> = std::env::var("PARITY_SIM").ok().map(|v| v.split(',').map(|s| format!("{}.txt", s.trim())).collect());
    let mut files: Vec<String> = read.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).filter(|n| n.ends_with(".txt")).collect();
    files.sort_by_key(|n| n.trim_end_matches(".txt").parse::<i32>().unwrap_or(0));
    let d = Data::load(&out_dir()).expect("parity exports (games/emberwake/parity/export.sh)");
    let mut failures = Vec::new();
    for name in files {
        if only.as_ref().is_some_and(|o| !o.contains(&name)) {
            continue;
        }
        let text = std::fs::read_to_string(dir.join(&name)).unwrap();
        let mut lines = text.lines();
        let cmd = lines.next().unwrap_or("").to_string();
        let want: Vec<&str> = lines.collect();
        let argv: Vec<String> = cmd.split_whitespace().map(String::from).collect();
        let got = emberwake_sim::run(&d, &argv);
        let first = want.iter().zip(&got).position(|(w, g)| *w != g.as_str());
        match first {
            Some(i) => failures.push(format!("{name} ({cmd}) line {}:\n  godot {}\n  rust  {}", i + 1, want[i], got[i])),
            None if want.len() != got.len() => failures.push(format!("{name} ({cmd}): godot {} lines, rust {}; first extra: {:?}", want.len(), got.len(), got.get(want.len()).or(want.get(got.len()).map(|s| s.to_string()).as_ref()))),
            None => eprintln!("{name} ({cmd}): {} lines match", want.len()),
        }
    }
    assert!(failures.is_empty(), "{} of the recordings differ:\n{}", failures.len(), failures.join("\n"));
}
