//! ENGINE-38 (LORE-RENAME1) on a fixture game written here: a unit referenced by an encounter, a quest, a seed and a
//! portrait path is renamed everywhere (its asset file moved); a dry run writes nothing; refs lists every place; prose
//! and other ids that only contain the id are left alone; a rename that adds a problem is put back.

use aegis_content::rename::{self, Ref};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("aegis_rename_{}_{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let w = |rel: &str, text: &str| {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    w("data/catalog/units.json", "{\n\t\"kind\": \"units\",\n\t\"entries\": [\n\t\t{\n\t\t\t\"id\": \"wren\",\n\t\t\t\"name\": \"Wren the wren\",\n\t\t\t\"art\": \"rogue/wren\",\n\t\t\t\"portrait\": \"portraits/wren.png\"\n\t\t},\n\t\t{\n\t\t\t\"id\": \"wren_elder\",\n\t\t\t\"name\": \"Elder\"\n\t\t}\n\t]\n}\n");
    w("data/worlds/w1/encounters.json", "{\"kind\": \"encounters\", \"entries\": [{\"id\": \"e1\", \"enemies\": [\"wren\", \"wren_elder\"], \"spawn\": {\"wren\": [1, 2]}}]}\n");
    w("data/worlds/w1/quests.json", "{\"kind\": \"quests\", \"entries\": [{\"id\": \"q1\", \"target\": \"w1:wren\", \"text\": \"Find wren by the river.\"}]}\n");
    w("data/seeds/s1.json", "{\n  \"steps\": [\n    {\"op\": \"upsert\", \"kind\": \"units\", \"entry\": {\"id\": \"wren\"}}\n  ]\n}\n");
    w("assets/portraits/wren.png", "png");
    w("assets/portraits/wren_elder.png", "png");
    w("assets/rogue/wren/idle.png", "png");
    root
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

#[test]
fn a_part_or_stem_matches_and_prose_does_not() {
    assert_eq!(rename::rename_in("wren", "wren", "kite").as_deref(), Some("kite"));
    assert_eq!(rename::rename_in("rogue/wren", "wren", "kite").as_deref(), Some("rogue/kite"));
    assert_eq!(rename::rename_in("portraits/wren.png", "wren", "kite").as_deref(), Some("portraits/kite.png"));
    assert_eq!(rename::rename_in("w1:wren", "wren", "kite").as_deref(), Some("w1:kite"));
    assert_eq!(rename::rename_in("wren_elder", "wren", "kite"), None, "another id that starts with it");
    assert_eq!(rename::rename_in("Find wren by the river.", "wren", "kite"), None, "prose");
    assert_eq!(rename::rename_in("a/b", "wren", "kite"), None);
}

#[test]
fn refs_list_every_place_and_a_dry_run_writes_nothing() {
    let root = fixture("refs");
    let (data, assets) = (root.join("data"), vec![root.join("assets")]);
    let refs: Vec<Ref> = rename::refs(&data, &assets, "wren");
    let texts: Vec<&str> = refs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(texts, ["wren", "rogue/wren", "portraits/wren.png", "wren", "wren", "wren", "w1:wren", "wren.png", "wren"], "{refs:?}");
    assert_eq!(refs[0].line, 5, "the unit's id line");
    let before: Vec<String> = ["data/catalog/units.json", "data/worlds/w1/encounters.json", "data/seeds/s1.json"].iter().map(|r| read(&root, r)).collect();
    let plan = rename::plan(&data, &assets, "wren", "kite");
    assert_eq!(plan.files.len(), 4);
    assert_eq!(plan.moves.len(), 2, "the portrait and the art folder: {:?}", plan.moves);
    let after: Vec<String> = ["data/catalog/units.json", "data/worlds/w1/encounters.json", "data/seeds/s1.json"].iter().map(|r| read(&root, r)).collect();
    assert_eq!(before, after, "planning writes nothing");
    assert!(root.join("assets/portraits/wren.png").is_file());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_rename_changes_only_the_matching_strings_and_moves_the_art() {
    let root = fixture("apply");
    let (data, assets) = (root.join("data"), vec![root.join("assets")]);
    let units = read(&root, "data/catalog/units.json");
    let plan = rename::plan(&data, &assets, "wren", "kite");
    rename::apply(&plan, "wren", "kite", &[], Vec::new).unwrap();
    assert_eq!(read(&root, "data/catalog/units.json"), units.replace("\"wren\"", "\"kite\"").replace("rogue/wren", "rogue/kite").replace("portraits/wren.png", "portraits/kite.png"), "byte for byte otherwise (tabs kept, prose kept)");
    assert!(read(&root, "data/catalog/units.json").contains("Wren the wren"));
    assert_eq!(read(&root, "data/worlds/w1/encounters.json"), "{\"kind\": \"encounters\", \"entries\": [{\"id\": \"e1\", \"enemies\": [\"kite\", \"wren_elder\"], \"spawn\": {\"kite\": [1, 2]}}]}\n");
    assert!(read(&root, "data/worlds/w1/quests.json").contains("\"w1:kite\"") && read(&root, "data/worlds/w1/quests.json").contains("Find wren"));
    assert!(read(&root, "data/seeds/s1.json").contains("{\"id\": \"kite\"}"));
    assert!(root.join("assets/portraits/kite.png").is_file() && !root.join("assets/portraits/wren.png").exists());
    assert!(root.join("assets/rogue/kite/idle.png").is_file());
    assert!(root.join("assets/portraits/wren_elder.png").is_file(), "another id's art stays");
    assert!(rename::refs(&data, &assets, "wren").is_empty());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_rename_that_adds_a_problem_is_put_back() {
    let root = fixture("undo");
    let (data, assets) = (root.join("data"), vec![root.join("assets")]);
    let units = read(&root, "data/catalog/units.json");
    let plan = rename::plan(&data, &assets, "wren", "kite");
    let err = rename::apply(&plan, "wren", "kite", &[], || vec!["broken reference".to_string()]).unwrap_err();
    assert_eq!(err, ["broken reference"]);
    assert_eq!(read(&root, "data/catalog/units.json"), units);
    assert!(root.join("assets/portraits/wren.png").is_file() && !root.join("assets/portraits/kite.png").exists());
    let _ = std::fs::remove_dir_all(&root);
}
