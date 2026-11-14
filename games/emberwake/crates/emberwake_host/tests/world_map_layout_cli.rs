//! PR5-48 (MAP-LAYOUT-ED), clause 2 through the command line: `aegis content world_map_layout <name> [<edits>]` (the
//! kit's sidecar writer behind the engine's `Hooks::sidecar` op) run as the `aegis` binary on a scratch copy of the
//! game (its data, game.json and world map sidecars, the backgrounds as stand-in files), never the shipped files: no
//! edits re-save the shipped Act 1 sidecar byte for byte, an edit lands in the canonical text, a refused edit writes
//! nothing, a pre-staged act starts its sidecar, and a seed step takes the op too. The shipped sidecar is already in the
//! writer's canonical form.

use aegis_core::scratch::Scratch;
use emberwake_content::world_map;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn game_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const ACT1: &str = "sunk_ward_quarter";

fn sidecar(game: &Path, name: &str) -> PathBuf {
    game.join("assets").join(emberwake_content::art::world_map_layout(name))
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let p = e.unwrap().path();
        let to = to.join(p.file_name().unwrap());
        if p.is_dir() {
            copy_dir(&p, &to);
        } else if p.extension().is_some_and(|x| x == "json") {
            std::fs::copy(&p, to).unwrap();
        }
    }
}

/// A scratch game: the shipped data and game.json, the world map sidecars as they are, each background as a stand-in.
fn scratch_game(tag: &str) -> (Scratch, PathBuf) {
    let s = Scratch::new(tag);
    let game = s.path().join("game");
    copy_dir(&game_dir().join("data"), &game.join("data"));
    std::fs::copy(game_dir().join("game.json"), game.join("game.json")).unwrap();
    let from = game_dir().join("assets").join(world_map::DIR);
    let to = game.join("assets").join(world_map::DIR);
    std::fs::create_dir_all(&to).unwrap();
    for e in std::fs::read_dir(&from).unwrap() {
        let p = e.unwrap().path();
        match p.extension().and_then(|x| x.to_str()) {
            Some("json") => {
                std::fs::copy(&p, to.join(p.file_name().unwrap())).unwrap();
            }
            Some("png") => std::fs::write(to.join(p.file_name().unwrap()), "png").unwrap(),
            _ => {}
        }
    }
    (s, game)
}

fn aegis(game: &Path, args: &[&str]) -> (bool, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_aegis")).args(args).arg(format!("data={}", game.join("data").display())).output().expect("the aegis binary runs");
    (out.status.success(), String::from_utf8_lossy(&out.stdout).to_string(), String::from_utf8_lossy(&out.stderr).to_string())
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap()
}

#[test]
fn the_shipped_act_one_sidecar_is_in_the_canonical_form() {
    let text = read(&sidecar(&game_dir(), ACT1));
    assert_eq!(world_map::text(&serde_json::from_str(&text).unwrap()), text);
}

#[test]
fn the_op_resaves_the_shipped_sidecar_byte_for_byte_and_shows_it() {
    let (_s, game) = scratch_game("wml_cli_resave");
    let shipped = read(&sidecar(&game_dir(), ACT1));
    let (ok, out, err) = aegis(&game, &["content", world_map::OP, ACT1, "[]"]);
    assert!(ok, "{out}{err}");
    assert_eq!(out.trim(), format!("unchanged {} (0 edit(s))", sidecar(&game, ACT1).display()));
    assert_eq!(read(&sidecar(&game, ACT1)), shipped, "byte for byte");
    let (ok, out, _) = aegis(&game, &["content", world_map::OP, ACT1]);
    assert!(ok);
    assert_eq!(out, shipped, "no edits given: the file as it is");
}

#[test]
fn an_edit_lands_in_the_canonical_text_and_a_refused_one_writes_nothing() {
    let (_s, game) = scratch_game("wml_cli_edit");
    let path = sidecar(&game, ACT1);
    let before: Value = serde_json::from_str(&read(&path)).unwrap();
    // the file's first landmark and first route (picked by rule, not by name)
    let (mark, was) = before["landmarks"].as_object().unwrap().iter().next().map(|(k, v)| (k.clone(), v.clone())).unwrap();
    let route = before["routes"].as_object().unwrap().keys().next().unwrap().clone();
    let edits = json!([{"id": mark, "pos": [0.4321, 0.6789], "scale": 1.25}, {"route": route, "points": [[0.5, 0.5], [0.55, 0.45]]}]).to_string();
    let (ok, out, err) = aegis(&game, &["content", world_map::OP, ACT1, &edits]);
    assert!(ok, "{out}{err}");
    assert!(out.starts_with("wrote ") && out.trim_end().ends_with("(2 edit(s))"), "{out}");
    let after: Value = serde_json::from_str(&read(&path)).unwrap();
    assert_eq!(after["landmarks"][&mark], json!({"pos": [0.4321, 0.6789], "scale": 1.25}));
    assert_ne!(after["landmarks"][&mark], was);
    assert_eq!(after["routes"][&route], json!([[0.5, 0.5], [0.55, 0.45]]));
    assert_eq!(read(&path), world_map::text(&after), "the canonical text");
    let kept = read(&path);
    for bad in [json!([{"id": "no_such_place", "pos": [0.5, 0.5]}]), json!([{"id": mark, "pos": [3.0, 0.5]}]), json!([{"id": mark, "scale": 0.1}]), json!([{"route": "no_such_road", "points": []}])] {
        let (ok, out, err) = aegis(&game, &["content", world_map::OP, ACT1, &bad.to_string()]);
        assert!(!ok, "{bad}: refused ({out})");
        assert!(err.contains("error: backgrounds/world_map/sunk_ward_quarter.json: ") && err.contains("nothing was written"), "{err}");
        assert_eq!(read(&path), kept, "{bad}: nothing written");
    }
}

#[test]
fn a_prestaged_act_starts_its_sidecar_and_a_seed_step_takes_the_op() {
    let (s, game) = scratch_game("wml_cli_staged");
    let staged = ["high_bastion", "mantle_fissure"].into_iter().find(|n| game.join("assets").join(emberwake_content::art::world_map_background(n)).exists() && !sidecar(&game, n).exists());
    let Some(name) = staged else { return };
    let (ok, out, err) = aegis(&game, &["content", world_map::OP, name, r#"[{"id": "spire_gate", "pos": [0.498, 0.158], "scale": 1.35}]"#]);
    assert!(ok, "{out}{err}");
    let v: Value = serde_json::from_str(&read(&sidecar(&game, name))).unwrap();
    assert_eq!(v, json!({"landmarks": {"spire_gate": {"pos": [0.498, 0.158], "scale": 1.35}}, "routes": {}}));
    // a seed: {"op": "world_map_layout", "name", "edits"}
    let seed = s.path().join("seed.json");
    std::fs::write(&seed, json!({"steps": [{"op": world_map::OP, "name": name, "edits": [{"route": "climb", "points": [[0.5, 0.3]]}]}]}).to_string()).unwrap();
    let (ok, out, err) = aegis(&game, &["content", "apply", seed.to_str().unwrap()]);
    assert!(ok, "{out}{err}");
    assert!(out.starts_with("1/1: wrote "), "{out}");
    let v: Value = serde_json::from_str(&read(&sidecar(&game, name))).unwrap();
    assert_eq!(v["routes"]["climb"], json!([[0.5, 0.3]]));
    // a name with no data and no background is refused
    let (ok, _, err) = aegis(&game, &["content", world_map::OP, "nowhere_at_all", "[]"]);
    assert!(!ok && err.contains("nothing to lay out"), "{err}");
}
