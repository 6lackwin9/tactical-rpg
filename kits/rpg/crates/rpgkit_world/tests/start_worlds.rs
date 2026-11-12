//! ACT1-HOOKS1 (FB-241 c): New Game's default world is the first of the game manifest's `start_worlds` that is playable
//! (`new_game::default_world`), read beside the data folder (`source::load` -> `Data::start_worlds`); with no such list,
//! or none of it playable, it falls back to the first playable world, as before.

use aegis_core::json;
use rpgkit_world::{new_game, Data};
use serde_json::{json, Value};

fn with_ids(w: &str, rows: Value) -> Value {
    Value::Array(
        json::arr(&rows)
            .iter()
            .map(|r| {
                let mut r = json::dict(r).clone();
                let id = json::s(&r, "id");
                r.insert("_world".into(), w.into());
                r.insert("_qid".into(), format!("{w}:{id}").into());
                Value::Object(r)
            })
            .collect(),
    )
}

/// Worlds `alpha`, `beta` (both playable) and `husk` (no start party: not playable).
fn data() -> Data {
    let own = |w: &str| json!({"locations": with_ids(w, json!([{"id": "hub", "name": "Hub"}]))});
    let worlds = json!({
        "alpha": {"manifest": {"id": "alpha", "start_location": "hub", "start_party": ["hero"]}, "own": own("alpha")},
        "beta": {"manifest": {"id": "beta", "start_location": "hub", "start_party": ["hero"]}, "own": own("beta")},
        "husk": {"manifest": {"id": "husk", "start_location": "hub"}, "own": own("husk")}
    });
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

#[test]
fn the_first_playable_start_world_is_the_default_else_the_first_playable_world() {
    let mut d = data();
    let playable = new_game::playable_worlds(&d);
    assert!(playable.len() >= 2 && !playable.contains(&"husk".to_string()), "{playable:?}");
    let (first, other) = (playable[0].clone(), playable[1].clone());
    assert!(d.start_worlds.is_empty(), "from values: no manifest list");
    assert_eq!(new_game::default_world(&d), first, "no list: the first playable world, as before");
    d.start_worlds = vec![other.clone()];
    assert_eq!(new_game::default_world(&d), other, "the manifest's start world");
    d.start_worlds = vec!["husk".into(), "gone".into(), other.clone()];
    assert_eq!(new_game::default_world(&d), other, "unplayable and unknown start worlds are passed over");
    d.start_worlds = vec!["husk".into()];
    assert_eq!(new_game::default_world(&d), first, "none of the list playable: the first playable world");
}

#[test]
fn the_loader_reads_start_worlds_from_the_manifest_beside_the_data_folder() {
    let scratch = aegis_core::scratch::Scratch::new("start_worlds");
    let game = scratch.path().to_path_buf();
    let data_dir = game.join("data");
    for dir in ["catalog", "worlds"] {
        std::fs::create_dir_all(data_dir.join(dir)).expect("folders");
    }
    let src = rpgkit_content::source::load(&data_dir);
    assert!(src.start_worlds.is_empty(), "no manifest: no start worlds");
    std::fs::write(game.join(aegis_content::manifest::FILE), json!({"id": "demo", "title": "Demo", "worlds": ["beta", "alpha"], "start_worlds": ["beta"]}).to_string()).expect("manifest");
    assert_eq!(rpgkit_content::source::load(&data_dir).start_worlds, ["beta"]);
    let (d, _) = Data::open(&data_dir).expect("the data folder opens");
    assert_eq!(d.start_worlds, ["beta"], "Data carries the manifest's list");
}
