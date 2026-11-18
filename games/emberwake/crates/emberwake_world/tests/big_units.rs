//! BIGUNIT1 section 5 (World): with the seed `data/seeds/pr7_12.json` applied (the Amalgam at `footprint: 2`) on a
//! scratch copy of the data (never the shipped folder), the Interlude's fight (`sunk_ward` `enc_evt_05`) deploys the
//! Amalgam on a 2 x 2 square that fits, raises no footprint problem, and plays to its end under every preset the
//! traces use, headless as world_traces plays it, with no two units ever sharing a cell.

use emberwake_battle::session::{EncounterDef, Phase, Session};
use emberwake_content::store;
use emberwake_world::battle::WorldRoster;
use emberwake_world::Data;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const WORLD: &str = "sunk_ward";
const FIGHT: &str = "enc_evt_05";
const BIG: &str = "calcified_amalgam";

fn data_dir() -> PathBuf {
    Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let p = e.path();
        if p.is_dir() {
            copy_dir(&p, &to.join(e.file_name()));
        } else {
            fs::copy(&p, to.join(e.file_name())).unwrap();
        }
    }
}

/// A scratch copy of the shipped data with the PR7-12 seed applied through the writer.
fn seeded() -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant("emberwake_big_units");
    let dir = t.join("data");
    copy_dir(&data_dir(), &dir);
    let seed: Value = serde_json::from_str(&fs::read_to_string(data_dir().join("seeds/pr7_12.json")).unwrap()).unwrap();
    for step in seed["steps"].as_array().unwrap() {
        assert_eq!(step["op"], "upsert");
        let kind = step["kind"].as_str().unwrap();
        store::upsert(&dir, None, kind, step["entry"].as_object().unwrap(), None).unwrap_or_else(|e| panic!("seed step: {e:?}"));
    }
    (t, dir)
}

/// No two active units share a cell; every big unit's square is on walkable, non-exit tiles.
fn assert_no_overlap(s: &Session, when: &str) {
    let w = &s.w;
    let mut seen = Vec::new();
    for u in (0..w.units.len()).filter(|&u| w.has(u)) {
        for cell in w.cells_of(u) {
            assert!(!seen.contains(&cell), "{when}: two units on {cell}");
            seen.push(cell);
        }
        if w.size_of(u) > 1 {
            assert!(w.grid.square_fits(w.pos(u), w.size_of(u)), "{when}: {}'s square at {} does not fit", w.units[u].name, w.pos(u));
        }
    }
}

#[test]
fn the_interlude_plays_with_a_two_by_two_amalgam() {
    let (_t, dir) = seeded();
    let footprint = store::check(&dir).into_iter().filter(|p| p.contains("2 x 2")).collect::<Vec<_>>();
    assert!(footprint.is_empty(), "the seeded data places the Amalgam: {footprint:?}");
    let d = Data::open(&dir).expect("seeded data").0;
    assert_eq!(d.cat.units.get(BIG).map(|u| u.footprint), Some(2), "the seed reached the typed row");
    let entry = d.resolve(WORLD, "encounters", FIGHT).clone();
    assert!(!entry.is_empty(), "{WORLD}:{FIGHT}");
    for preset in ["balanced", "aggressive", "defensive", "withdraw"] {
        for seed in 1..=3 {
            let mut def = EncounterDef::from_obj(&entry);
            def.id = format!("{WORLD}:{FIGHT}");
            def.rng_seed = seed;
            let mut s = Session::start(d.cat.clone(), def, &WorldRoster::new(&d, WORLD));
            let when = format!("{preset} seed {seed}");
            let big: Vec<usize> = (0..s.w.units.len()).filter(|&u| s.w.size_of(u) == 2).collect();
            assert_eq!(big.len(), 1, "{when}: one 2 x 2 unit");
            assert!(s.w.has(big[0]), "{when}: the Amalgam is deployed");
            assert_no_overlap(&s, &when);
            let mut ended = false;
            for _ in 0..s.def.max_rounds {
                s.plan_apply_preset_all(preset);
                if !s.confirm_plan() {
                    break;
                }
                for _ in 0..600 {
                    if s.phase == Phase::Replay {
                        break;
                    }
                    s.step_next();
                    assert_no_overlap(&s, &format!("{when} round {}", s.current_round));
                }
                assert_eq!(s.phase, Phase::Replay, "{when}: the round resolves");
                if s.combat_over || !s.can_advance_round() {
                    ended = true;
                    break;
                }
                s.advance_round();
            }
            assert!(ended || s.current_round >= s.def.max_rounds, "{when}: the battle reaches its end");
        }
    }
}
