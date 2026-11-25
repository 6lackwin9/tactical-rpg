//! REGIONGROUND1-K (docs/specs/REGIONGROUND1_KIT.md 2.3): `rows` alone decide walking. On a fixture with ground over every
//! cell, the map rules give what they give without it, and no world or battle source reads the new keys. (The region
//! screen's `path_to` lives in the game's screen, over `is_open`; its own checks are slice L's.)

use aegis_core::json::Dict;
use rpgkit_world::explore;
use serde_json::{json, Value};
use std::collections::{HashSet, VecDeque};
use std::path::Path;

const ROWS: [&str; 6] = ["##########", "#....#...#", "#.##.#.#.#", "#.~~...#.#", "#....#...#", "##########"];

fn map(extra: Value) -> Dict {
    let mut t = json!({"id": "walk_map", "name": "W", "rows": ROWS, "zones": ["ssssssssss", "ssssssssss", "ssssssssss", "ssssssssss", "ssssssssss", "ssssssssss"],
        "zone_legend": {"s": {"kind": "safe"}}, "anchors": [{"id": "spawn", "x": 1, "y": 1}], "paths": {}, "props": [], "tags": []});
    for (k, v) in extra.as_object().unwrap() {
        t[k] = v.clone();
    }
    t.as_object().cloned().unwrap()
}

/// The cells a walk from (1, 1) reaches, by the map's own rule.
fn reach(t: &Dict) -> HashSet<(i64, i64)> {
    let mut seen = HashSet::from([(1, 1)]);
    let mut queue = VecDeque::from([(1, 1)]);
    while let Some((x, y)) = queue.pop_front() {
        for next in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
            if explore::is_open(t, next.0, next.1) && seen.insert(next) {
                queue.push_back(next);
            }
        }
    }
    seen
}

#[test]
fn ground_over_every_cell_changes_no_walk() {
    let plain = map(json!({}));
    // every cell one letter, so a reader of `ground` would see a wall or water nowhere
    let drawn = map(json!({"ground": vec!["aaaaaaaaaa"; 6], "ground_legend": {"a": "dust"}, "ground_patches": [{"id": "stain", "x": 3, "y": 1, "w": 1, "h": 1, "strength": 0.2}], "wall_storeys": [{"x": 2, "y": 2, "w": 2, "h": 1, "storeys": 3}]}));
    assert_eq!(explore::size_of(&plain), explore::size_of(&drawn));
    for y in -1..=6 {
        for x in -1..=10 {
            assert_eq!(explore::is_open(&plain, x, y), explore::is_open(&drawn, x, y), "is_open ({x}, {y})");
            assert_eq!(explore::tile(&plain, x, y), explore::tile(&drawn, x, y), "tile ({x}, {y})");
            assert_eq!(explore::zone_at(&plain, x, y), explore::zone_at(&drawn, x, y), "zone ({x}, {y})");
        }
    }
    assert_eq!(reach(&plain), reach(&drawn));
    assert!(!explore::is_open(&drawn, 2, 3), "`~` stays closed whatever its ground says");
}

fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn no_world_or_battle_source_reads_the_new_keys() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&here.join("src"), &mut files);
    rust_files(&here.join("../rpgkit_battle/src"), &mut files);
    assert!(!files.is_empty());
    for f in files {
        let text = std::fs::read_to_string(&f).unwrap();
        // not the bare word `ground`: a data zone's fallback terrain and the battle's hazards use it for other things
        for key in ["ground_legend", "ground_patches", "wall_storeys"] {
            assert!(!text.contains(key), "{} reads '{key}'", f.display());
        }
    }
}
