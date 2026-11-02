//! CLN-38: the RNG and hash golden. Every seeded golden of every game (battle traces, world fights, rolls derived
//! from hashes) depends on `rng::GodotRng` and `ghash::hash` drawing exactly the same numbers, so their outputs are
//! frozen in `tests/fixtures/rng_hash.json`: seeds with a fixed draw sequence (`randi`, `randf`, `randi_range`,
//! `randf_range` with their bounds), hash inputs (strings, ints and the `[seed, stream, n]` roll key) with their
//! values and the first draw of an RNG seeded by the key's hash.
//!
//! The values were recorded once from the engine (CLN-38, 2026-11-01; the inputs follow the retired Godot export's
//! rng.json and hash.json). Re-record only for an intended change of the engine's numeric semantics, named in the
//! commit and a docs/HISTORY.md row; never edit the fixture by hand:
//!
//! `RECORD=1 tools/fc.sh test --test rng_hash`

use aegis_core::ghash::{hash, GVar};
use aegis_core::rng::GodotRng;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::OnceLock;

/// The seeds of the draw sequences (zero, small, the battle seeds, a large and a negative one).
const SEEDS: [i64; 6] = [0, 1, 1337, 42, 987654321, -5];
/// Rounds of the five-draw pattern per seed.
const ROUNDS: usize = 6;
/// The roll-key seeds and streams (a GameRolls-style key `[seed, stream, n]`).
const KEY_SEEDS: [i64; 5] = [0, 1, 1337, 2147483647, 99];
const KEY_STREAMS: [&str; 4] = ["explore", "travel", "purse", ""];
const KEY_NS: [i64; 3] = [0, 1, 7];
const STRINGS: [&str; 4] = ["", "a", "explore", "region:keep|3|0"];
const INTS: [i64; 5] = [0, 1, -1, 123456789, 2147483647];
/// Float tolerance (the fixture's floats are written in shortest form and parsed back).
const EPS: f64 = 1e-12;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rng_hash.json")
}

fn recording() -> bool {
    std::env::var("RECORD").is_ok_and(|v| v == "1")
}

/// One round of the draw pattern on `r`: `[kind, args..., value]` rows.
fn draw_round(r: &mut GodotRng) -> Vec<Value> {
    vec![
        json!(["randi", r.randi()]),
        json!(["randf", r.randf()]),
        json!(["randi_range", -1, 1, r.randi_range(-1, 1)]),
        json!(["randi_range", 0, 99, r.randi_range(0, 99)]),
        json!(["randf_range", 0.5, 2.0, r.randf_range(0.5, 2.0)]),
    ]
}

fn roll_key(seed: i64, stream: &str, n: i64) -> GVar {
    GVar::Arr(vec![GVar::Int(seed), GVar::Str(stream.into()), GVar::Int(n)])
}

/// The golden from the current engine: `{rng: [{seed, draws}], hash: [...]}`, one row per line.
fn record() -> String {
    let mut rng_rows = Vec::new();
    for seed in SEEDS {
        let mut r = GodotRng::from_i64(seed);
        let draws: Vec<Value> = (0..ROUNDS).flat_map(|_| draw_round(&mut r)).collect();
        let lines: Vec<String> = draws.iter().map(|d| format!("      {d}")).collect();
        rng_rows.push(format!("    {{\"seed\": {seed}, \"draws\": [\n{}\n    ]}}", lines.join(",\n")));
    }
    let mut hash_rows: Vec<Value> = Vec::new();
    for seed in KEY_SEEDS {
        for stream in KEY_STREAMS {
            for n in KEY_NS {
                let h = hash(&roll_key(seed, stream, n));
                hash_rows.push(json!({"seed": seed, "stream": stream, "n": n, "hash": h, "roll": GodotRng::new(h as u64).randi()}));
            }
        }
    }
    hash_rows.extend(STRINGS.iter().map(|s| json!({"string": s, "hash": hash(&GVar::Str((*s).into()))})));
    hash_rows.extend(INTS.iter().map(|i| json!({"int": i, "hash": hash(&GVar::Int(*i))})));
    let hash_lines: Vec<String> = hash_rows.iter().map(|h| format!("    {h}")).collect();
    format!("{{\n  \"rng\": [\n{}\n  ],\n  \"hash\": [\n{}\n  ]\n}}\n", rng_rows.join(",\n"), hash_lines.join(",\n"))
}

/// The golden, read once per run (recorded first when `RECORD=1`, once even with the tests running in parallel).
fn golden() -> &'static Value {
    static GOLDEN: OnceLock<Value> = OnceLock::new();
    GOLDEN.get_or_init(|| {
        let path = fixture();
        if recording() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, record()).unwrap();
            eprintln!("recorded {}", path.display());
        }
        let text = std::fs::read_to_string(&path).expect("tests/fixtures/rng_hash.json (recorded by CLN-38)");
        serde_json::from_str(&text).unwrap()
    })
}

#[test]
fn rng_draws_match_the_golden() {
    let g = golden();
    let rows = g["rng"].as_array().unwrap();
    assert!(!rows.is_empty(), "the golden has no seeds");
    let mut n = 0;
    for row in rows {
        let seed = row["seed"].as_i64().unwrap();
        let mut r = GodotRng::from_i64(seed);
        for (i, d) in row["draws"].as_array().unwrap().iter().enumerate() {
            let d = d.as_array().unwrap();
            let want = d.last().unwrap();
            let at = format!("seed {seed} draw {i} {}", d[0]);
            match d[0].as_str().unwrap() {
                "randi" => assert_eq!(r.randi(), want.as_i64().unwrap(), "{at}"),
                "randf" => {
                    let got = r.randf();
                    assert!((got - want.as_f64().unwrap()).abs() < EPS, "{at}: got {got} want {want}");
                }
                "randi_range" => {
                    let got = r.randi_range(d[1].as_i64().unwrap() as i32, d[2].as_i64().unwrap() as i32);
                    assert_eq!(got as i64, want.as_i64().unwrap(), "{at}");
                }
                "randf_range" => {
                    let got = r.randf_range(d[1].as_f64().unwrap(), d[2].as_f64().unwrap());
                    assert!((got - want.as_f64().unwrap()).abs() < EPS, "{at}: got {got} want {want}");
                }
                other => panic!("{at}: unknown draw kind {other}"),
            }
            n += 1;
        }
    }
    eprintln!("rng: {n} draws match the golden");
}

#[test]
fn hashes_match_the_golden() {
    let g = golden();
    let rows = g["hash"].as_array().unwrap();
    assert!(!rows.is_empty(), "the golden has no hashes");
    for r in rows {
        let want = r["hash"].as_u64().unwrap() as u32;
        let got = if let Some(s) = r.get("string") {
            hash(&GVar::Str(s.as_str().unwrap().into()))
        } else if let Some(i) = r.get("int") {
            hash(&GVar::Int(i.as_i64().unwrap()))
        } else {
            let h = hash(&roll_key(r["seed"].as_i64().unwrap(), r["stream"].as_str().unwrap(), r["n"].as_i64().unwrap()));
            assert_eq!(GodotRng::new(h as u64).randi(), r["roll"].as_i64().unwrap(), "the roll seeded by {r}");
            h
        };
        assert_eq!(got, want, "hash of {r}");
    }
    eprintln!("hash: {} samples match the golden", rows.len());
}

/// The fixture holds exactly the inputs this file records (so a re-record cannot silently drop a case).
#[test]
fn the_golden_covers_every_input() {
    let g = golden();
    let seeds: Vec<i64> = g["rng"].as_array().unwrap().iter().map(|r| r["seed"].as_i64().unwrap()).collect();
    assert_eq!(seeds, SEEDS.to_vec());
    for r in g["rng"].as_array().unwrap() {
        assert_eq!(r["draws"].as_array().unwrap().len(), ROUNDS * 5, "draws of seed {}", r["seed"]);
    }
    let keys = KEY_SEEDS.len() * KEY_STREAMS.len() * KEY_NS.len();
    assert_eq!(g["hash"].as_array().unwrap().len(), keys + STRINGS.len() + INTS.len());
}
