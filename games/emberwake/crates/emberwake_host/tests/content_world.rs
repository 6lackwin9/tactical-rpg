//! `aegis content world` and the seed step {"op": "world"} (WORLDW1) through the game's binary, on a scratch copy of
//! data/ (the shipped data is only read): a resave of every shipped world's manifest without changes is byte-identical,
//! a seed creates a world and adds an entry to it, and a refused step leaves the folder as it was.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let p = e.path();
        if p.is_dir() {
            copy_dir(&p, &to.join(e.file_name()));
        } else if p.extension().is_some_and(|x| x == "json") {
            fs::copy(&p, to.join(e.file_name())).unwrap();
        }
    }
}

/// A scratch copy of the shipped data folder (json files only) at `<tmp>/<tag>/data`.
fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("emberwake_content_world_{tag}"));
    let data = t.join("data");
    copy_dir(&data_dir(), &data);
    (t, data)
}

fn aegis(data: &Path, args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_aegis"));
    cmd.arg("content").args(args).arg(format!("data={}", data.display()));
    cmd.output().unwrap()
}

fn text(out: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))
}

fn worlds(data: &Path) -> Vec<String> {
    let mut ids: Vec<String> = fs::read_dir(data.join("worlds")).unwrap().filter_map(|e| e.ok()).filter(|e| e.path().join("world.json").is_file()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    ids.sort();
    ids
}

#[test]
fn an_untouched_resave_of_every_shipped_world_manifest_is_byte_identical() {
    let (_scratch, data) = scratch("resave");
    let ids = worlds(&data);
    assert!(!ids.is_empty(), "the shipped data has worlds");
    for id in &ids {
        let path = aegis_content::world::manifest_path(&data, id);
        let before = fs::read(&path).unwrap();
        let out = aegis(&data, &["world", id, "{}"]);
        assert!(out.status.success(), "{id}: {}", text(&out));
        assert!(text(&out).contains(&format!("updated world '{id}'")), "{}", text(&out));
        assert_eq!(fs::read(&path).unwrap(), before, "{id}: world.json byte-identical after a resave");
        let shown = aegis(&data, &["world", id]);
        assert_eq!(shown.stdout, before, "{id}: `world <id>` prints the manifest");
    }
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

#[test]
fn a_seed_creates_a_world_and_fills_it() {
    let (_scratch, data) = scratch("seed");
    let id = "test_seed_world";
    let seed = data.parent().unwrap().join("seed.json");
    fs::write(
        &seed,
        serde_json::json!({"steps": [
            {"op": "world", "id": id, "fields": {"name": "Test Seed World", "tagline": "Made by a seed."}},
            {"op": "upsert", "world": id, "kind": "lore", "entry": {"id": "test_note", "name": "Test Note", "category": "history", "text": "A line."}}
        ]})
        .to_string(),
    )
    .unwrap();
    let out = aegis(&data, &["apply", &seed.display().to_string()]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains(&format!("1/2: created world '{id}'")), "{}", text(&out));
    let d = emberwake_world::Data::open(&data).unwrap().0;
    assert_eq!(aegis_core::json::s(d.resolve(id, "lore", "test_note"), "text"), "A line.");
    let m: serde_json::Value = serde_json::from_str(&fs::read_to_string(aegis_content::world::manifest_path(&data, id)).unwrap()).unwrap();
    assert_eq!(m["name"], "Test Seed World");

    // a refused step: the bad value is named, nothing is written
    let before = fs::read(aegis_content::world::manifest_path(&data, id)).unwrap();
    fs::write(&seed, serde_json::json!([{"op": "world", "id": id, "fields": {"ruleset": "no_such_ruleset"}}]).to_string()).unwrap();
    let out = aegis(&data, &["apply", &seed.display().to_string()]);
    assert!(!out.status.success());
    assert!(text(&out).contains("no_such_ruleset"), "{}", text(&out));
    assert_eq!(fs::read(aegis_content::world::manifest_path(&data, id)).unwrap(), before);
    let _ = fs::remove_dir_all(data.parent().unwrap());
}
