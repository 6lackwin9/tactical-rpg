//! LOOK1-K tile set binding (LOOK1 5.4): the set id rule, the lookups' order (`art::region_sets`,
//! `art::battle_stage_dirs`, `art::transition_dirs`), the schema fields and the asset checks (`check::assets::
//! tileset_errors` / `tileset_warnings`) on a test-only world and scratch asset layer. Expectations follow from the
//! spec's rules, never from shipped content.

use aegis_core::assets::Layers;
use rpgkit_content::check::{assets, View};
use rpgkit_content::art;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn png(path: &Path, w: u32, h: u32) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut b = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR".to_vec();
    b.extend(w.to_be_bytes());
    b.extend(h.to_be_bytes());
    std::fs::write(path, b).unwrap();
}

fn scratch(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rpgkit_look1_sets_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    root
}

/// A world with a stage set and two locations (one bound to `tileset`, one not) on one map using floor, wall and water.
fn view(root: &Path, tileset: &str, stage_set: &str) -> View {
    let worlds = json!({"test_world": {"manifest": {"id": "test_world", "stage_set": stage_set}, "own": {"locations": [
        {"id": "bound", "map": "test_map", "tileset": tileset}, {"id": "plain", "map": "test_map"}]}}});
    let map = json!({"id": "test_map", "name": "T", "rows": ["#####", "#.~.#", "#####"], "zones": [], "zone_legend": {}, "anchors": [], "paths": {}, "props": [], "tags": []});
    View::new(&json!({}), &worlds, &json!({"maps": [map]}), Layers::of(vec![root.to_path_buf()]))
}

#[test]
fn the_id_rule() {
    for ok in ["ab", "sunk_ward", "set_2", &"a".repeat(32)] {
        assert!(art::set_id_ok(ok), "{ok}");
    }
    for bad in ["", "a", "Sunk", "sunk-ward", "sunk ward", "../x", &"a".repeat(33)] {
        assert!(!art::set_id_ok(bad), "{bad}");
    }
    let root = scratch("ids");
    let v = view(&root, "Bad-Set", "x");
    let errors = assets::tileset_errors(&v, "test_world");
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(errors[0].starts_with("Stage set 'x'") && errors[1].starts_with("Location 'bound' tile set 'Bad-Set'"));
    assert!(assets::tileset_errors(&view(&root, "", ""), "test_world").is_empty(), "empty fields name nothing");
}

#[test]
fn lookups_put_a_named_set_first_and_empty_gives_todays_chains() {
    let lineage = vec!["child".to_string(), "parent".to_string()];
    assert_eq!(art::region_sets(&lineage, ""), ["child", "parent", "common"]);
    assert_eq!(art::region_sets(&lineage, "ward"), ["ward", "child", "parent", "common"]);
    assert_eq!(art::region_sets(&lineage, "parent"), ["parent", "child", "common"], "a set once");
    assert_eq!(art::battle_stage_dirs("grass", ""), ["tilesets/common_stage/grass"]);
    assert_eq!(art::battle_stage_dirs("grass", "ward_stage"), ["tilesets/ward_stage/grass", "tilesets/common_stage/grass"]);
    assert_eq!(art::battle_stage_dirs("wall_face", "common_stage"), ["tilesets/common_stage/wall_face"]);
    assert_eq!(art::transition_dirs(&art::region_sets(&lineage, "ward"), "grass", "dirt")[0], "tilesets/ward/grass/over_dirt");
    assert_eq!(art::transition_piece("ward", "grass", "dirt", 5), "tilesets/ward/grass/over_dirt/m05.png");
    assert_eq!(art::prop_dirs(&lineage, "tent"), ["props/child/tent", "props/parent/tent", "props/common/tent"], "props ignore the tileset");
}

#[test]
fn the_schema_holds_both_fields_with_empty_defaults() {
    let schema: Value = serde_json::from_str(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../schema.json")).unwrap()).unwrap();
    for (table, key) in [("locations", "tileset"), ("world", "stage_set")] {
        let f = schema["world"][table].as_array().unwrap().iter().find(|f| f["key"] == key).unwrap_or_else(|| panic!("{table}.{key}"));
        assert_eq!(f["default"], "");
        assert!(!f["help"].as_str().unwrap().is_empty());
    }
}

#[test]
fn warnings_name_missing_files_tiles_and_pieces() {
    let root = scratch("warn");
    // nothing there: both named sets are missing, the plain location says nothing
    let v = view(&root, "ward", "ward_stage");
    assert_eq!(
        assets::tileset_warnings(&v, "test_world"),
        ["Location 'bound' tile set 'ward': no file tilesets/ward/floor/v0.png in any layer; it draws today's sets.".to_string(),
         "Stage set 'ward_stage': no tilesets/ward_stage/<terrain>/v0.png in any layer; the battle draws common_stage.".to_string()]
    );
    // floor and wall but no shallow_water, and a pair with 3 of 14 pieces
    png(&root.join("tilesets/ward/floor/v0.png"), 32, 32);
    png(&root.join("tilesets/ward/wall/v0.png"), 32, 32);
    for m in [1, 2, 3] {
        png(&root.join(format!("tilesets/ward/floor/over_wall/m{m:02}.png")), 32, 32);
    }
    png(&root.join("tilesets/ward_stage/grass/v0.png"), 64, 48);
    let found = assets::tileset_warnings(&view(&root, "ward", "ward_stage"), "test_world");
    assert_eq!(found.len(), 2, "{found:?}");
    assert_eq!(found[0], "Location 'bound' tile set 'ward': its map uses '~' but tilesets/ward/shallow_water/v0.png is missing; that tile draws today's set.");
    assert_eq!(found[1], "Tile set 'ward' pair floor over_wall: 11 of 14 pieces missing (m04, m05, m06, m07, m08, m09, m10, m11, m12, m13, m14); the pair draws no transition.");
    // all 14 or none: no pair line
    for m in 4..=14 {
        png(&root.join(format!("tilesets/ward/floor/over_wall/m{m:02}.png")), 32, 32);
    }
    png(&root.join("tilesets/ward/shallow_water/v0.png"), 32, 32);
    assert!(assets::tileset_warnings(&view(&root, "ward", "ward_stage"), "test_world").is_empty());
    let mut none = view(&root, "ward", "ward_stage");
    none.assets = Layers::default();
    assert!(assets::tileset_warnings(&none, "test_world").is_empty(), "skipped without asset layers");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn tiles_of_the_wrong_size_are_errors() {
    let root = scratch("size");
    png(&root.join("tilesets/ward/floor/v0.png"), 32, 32);
    png(&root.join("tilesets/ward/floor/v1.png"), 32, 24);
    png(&root.join("tilesets/ward/wall/v0.png"), 16, 16);
    png(&root.join("tilesets/ward_stage/grass/v0.png"), 64, 48);
    png(&root.join("tilesets/ward_stage/wall_face/v0.png"), 64, 24);
    png(&root.join("tilesets/ward_stage/wall_face/v1.png"), 64, 48);
    png(&root.join("tilesets/ward/wall_face/v0.png"), 32, 16);
    png(&root.join("tilesets/ward/wall_face/v1.png"), 32, 12);
    png(&root.join("tilesets/ward/wall_face/v2.png"), 32, 32);
    let errors = assets::tileset_errors(&view(&root, "ward", "ward_stage"), "test_world");
    assert_eq!(
        errors,
        ["Tile art 'tilesets/ward/wall/v0.png' is 16x16 px; a region tile wants 32x32 or 32x24.".to_string(),
         "Tile art 'tilesets/ward/wall_face/v2.png' is 32x32 px; a region wall face wants 32x16 or 32x12 or 128x48.".to_string(),
         "Tile art 'tilesets/ward_stage/wall_face/v1.png' is 64x48 px; a stage wall face wants 64x24.".to_string()]
    );
    // one id as both the region set and the stage set is checked in both roles
    let both = assets::tileset_errors(&view(&root, "ward_stage", "ward_stage"), "test_world");
    assert!(both.iter().any(|e| e.contains("region wall face")) && both.iter().any(|e| e.contains("stage wall face")), "{both:?}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_writer_leaves_empty_set_fields_out() {
    let entry = |extra: Value| {
        let mut e = json!({"id": "spot", "name": "Spot", "map": "m"});
        for (k, v) in extra.as_object().unwrap() {
            e[k] = v.clone();
        }
        e.as_object().cloned().unwrap()
    };
    let plain = rpgkit_content::store::world_storage("locations", &entry(json!({})));
    assert!(!plain.contains_key("tileset"), "an empty tileset is not written");
    assert!(!rpgkit_content::store::world_storage("locations", &entry(json!({"tileset": ""}))).contains_key("tileset"));
    let bound = rpgkit_content::store::world_storage("locations", &entry(json!({"tileset": "ward"})));
    assert_eq!(bound["tileset"], "ward");
    let m = |extra: Value| rpgkit_content::store::world_storage("world", &extra.as_object().cloned().unwrap());
    assert!(!m(json!({"id": "w", "name": "W"})).contains_key("stage_set"));
    assert_eq!(m(json!({"id": "w", "name": "W", "stage_set": "ward_stage"}))["stage_set"], "ward_stage");
}
