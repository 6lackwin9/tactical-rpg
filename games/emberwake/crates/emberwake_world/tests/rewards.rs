//! REWARD1: rewards per defeated enemy (games/emberwake/docs/specs/REWARD1.md section 11). Scratch copies of the shipped
//! data get the purses, loot tables, key loot and objective rewards a case needs (written through the content store).

use aegis_core::json::{self, Dict};
use emberwake_content::store;
use emberwake_world::{battle, save, Data, GameState};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn data_dir() -> PathBuf {
    Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let p = e.path();
        if p.is_dir() {
            copy_dir(&p, &to.join(e.file_name()));
        } else if p.extension().is_some_and(|x| x == "json") {
            std::fs::copy(&p, to.join(e.file_name())).unwrap();
        }
    }
}

/// A scratch copy of the shipped data; `edits` are (world or None, kind, id, {field: value}) merged into entries.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn scratch(name: &str, edits: &[(Option<&str>, &str, &str, Value)]) -> (Scratch, Data) {
    let tmp = std::env::temp_dir().join(format!("aegis_rewards_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    for sub in ["catalog", "worlds", "maps", "quests", "dialogue", "progression", "assets"] {
        copy_dir(&data_dir().join(sub), &tmp.join(sub));
    }
    for (world, kind, id, fields) in edits {
        let d = Data::open(&tmp).unwrap().0;
        let mut e: Dict = match world {
            Some(w) => {
                let hit = d.resolve(w, kind, id);
                hit.iter().filter(|(k, _)| !k.starts_with('_')).map(|(k, v)| (k.clone(), v.clone())).collect()
            }
            None => d.entry(kind, id).clone(),
        };
        if e.is_empty() {
            e.insert("id".into(), json!(id));
        }
        for (k, v) in fields.as_object().unwrap() {
            e.insert(k.clone(), v.clone());
        }
        store::upsert(&tmp, *world, kind, &e, None).unwrap_or_else(|err| panic!("{kind} {id}: {err:?}"));
    }
    let d = Data::open(&tmp).unwrap().0;
    (Scratch(tmp), d)
}

fn envelope(gs: &GameState, version: i64) -> Dict {
    json::obj([("format", json!(save::FORMAT)), ("version", json!(version)), ("saved_at", json!(0)), ("game_state", gs.to_value())])
}

#[test]
fn the_roster_keys_each_ref_by_its_earlier_entries_and_weighs_kill_xp() {
    let (_s, d) = scratch("roster", &[(Some("ashlands"), "encounters", "ash_raiders", json!({"wave_units": ["ash_brawler"]}))]);
    let r = battle::roster(&d, "ashlands", "ashlands:ash_raiders");
    let keys: Vec<&str> = r.iter().map(|f| f.key.as_str()).collect();
    assert_eq!(keys, ["ash_brawler#0", "ash_skirmisher#0", "ash_shaman#0", "ash_brawler#1"]);
    assert!(r.iter().all(|f| f.tier == "minion" && f.level == 3 && f.weight == 1250), "minion L3 weighs 1250: {r:?}");
    let patrol = battle::roster(&d, "ashlands", "ashlands:ash_patrol");
    assert_eq!(patrol.iter().map(|f| f.weight).collect::<Vec<_>>(), [3000, 1250], "the warchief (standard L4) and a minion");
}

#[test]
fn version_11_saves_pay_no_once_purse_twice() {
    let (_s, d) = scratch("v11", &[(Some("ashlands"), "encounters", "ash_patrol", json!({"purse": {"gold_min": 5, "gold_max": 9, "once": true}})), (Some("ashlands"), "encounters", "ash_raiders", json!({"purse": {"gold_min": 5, "gold_max": 9, "once": false}}))]);
    let mut gs = emberwake_world::new_game::create_for_world(&d, "ashlands", 7).expect("a new game");
    gs.purses = json::obj([("ashlands:ash_patrol", json!(1)), ("ashlands:ash_raiders", json!(3)), ("ashlands:ash_scouts", json!(1))]);
    let (out, warnings) = save::migrate(&d, &envelope(&gs, 11)).expect("a version 11 save migrates");
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(out["version"], json!(save::VERSION));
    let back = GameState::from_dict(&d, out["game_state"].as_object().unwrap());
    assert_eq!(Value::Object(back.reward_paid.clone()), json!({"ashlands:ash_patrol": ["ash_warchief#0", "ash_berserker#0"]}), "only the once purse already won");
    assert!(back.validate(&d).is_empty(), "{:?}", back.validate(&d));

    gs.purses = Dict::new();
    let (out, _) = save::migrate(&d, &envelope(&gs, 11)).unwrap();
    assert_eq!(out["game_state"]["reward_paid"], json!({}), "no purse won: an empty record");
}

#[test]
fn reward_paid_round_trips_and_a_bad_one_is_a_load_problem() {
    let d = Data::open(&data_dir()).unwrap().0;
    let mut gs = emberwake_world::new_game::create_for_world(&d, "ashlands", 3).unwrap();
    gs.reward_paid = json::obj([("ashlands:ash_patrol", json!(["ash_warchief#0", "objective"]))]);
    let back = GameState::from_dict(&d, gs.to_value().as_object().unwrap());
    assert_eq!(back.reward_paid, gs.reward_paid);
    let mut bad = gs.to_value();
    bad["reward_paid"] = json!({"ashlands:ash_patrol": [3]});
    let loaded = GameState::from_dict(&d, bad.as_object().unwrap());
    assert!(loaded.validate(&d).contains(&"reward_paid must map encounter ids to lists of keys.".to_string()), "{:?}", loaded.validate(&d));
    bad["reward_paid"] = json!([]);
    let loaded = GameState::from_dict(&d, bad.as_object().unwrap());
    assert!(loaded.validate(&d).contains(&"reward_paid must map encounter ids to lists of keys.".to_string()));
}
