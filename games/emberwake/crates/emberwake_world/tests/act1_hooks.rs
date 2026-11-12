//! ACT1-HOOKS1 (FB-241 c) on a scratch copy of the shipped data: the Sunk Ward encounter whose guests are
//! `tithe_captive` characters (picked by rule: the row names the captive, M2's cages) takes a `survivors` block through
//! the writer (`aegis content`'s upsert; the data folder still loads and checks clean), and a world fight of it settled
//! after one captive falls records the captives that lived in the flag; a settle without a battle counts every one.
//! Counts are read from the encounter's own guests; nothing shipped is written.

use aegis_core::json::{self, Dict};
use aegis_core::scratch::Scratch;
use emberwake_content::store;
use emberwake_world::campaign::trigger;
use emberwake_world::{battle, new_game, Data};
use serde_json::json;
use std::path::{Path, PathBuf};

const WORLD: &str = "sunk_ward";
const CAPTIVE: &str = "tithe_captive";
const FLAG: &str = "sunk_ward.saved_youths";

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

/// The world's own encounter whose guests include a character on the captive base unit, and those guests' refs.
fn captive_fight(d: &Data) -> (Dict, Vec<String>) {
    d.merged_entries(WORLD, "encounters")
        .into_iter()
        .find_map(|e| {
            let captives: Vec<String> = json::sget(e, "guests").into_iter().filter(|r| json::s(d.resolve(WORLD, "characters", r), "base_unit") == CAPTIVE).collect();
            (!captives.is_empty()).then(|| (e.clone(), captives))
        })
        .expect("a Sunk Ward encounter with captive guests")
}

/// A scratch copy of the shipped data with the captive fight's `survivors` block set through the writer.
fn scratch() -> (Scratch, Data, String, Vec<String>) {
    let tmp = Scratch::vacant("act1_hooks_survivors");
    for sub in ["catalog", "worlds", "maps", "quests", "dialogue", "progression"] {
        copy_dir(&data_dir().join(sub), &tmp.join(sub));
    }
    let (enc, captives) = captive_fight(&Data::open(&tmp).unwrap().0);
    let mut row: Dict = enc.iter().filter(|(k, _)| !k.starts_with('_')).map(|(k, v)| (k.clone(), v.clone())).collect();
    row.insert("survivors".into(), json!({"flag": FLAG, "unit": CAPTIVE}));
    store::upsert(&tmp, Some(WORLD), "encounters", &row, None).unwrap_or_else(|e| panic!("the writer takes the block: {e:?}"));
    let d = Data::open(&tmp).unwrap().0;
    assert_eq!(d.resolve(WORLD, "encounters", &json::s(&enc, "id")).get("survivors"), row.get("survivors"), "stored and read back");
    (tmp, d, json::s(&enc, "_qid"), captives)
}

fn fight(qid: &str) -> Dict {
    json::dict(&json!({"source": trigger::FIGHT_SOURCE, "encounter": qid})).clone()
}

#[test]
fn the_m2_settle_records_the_captives_that_lived() {
    let (_tmp, d, qid, captives) = scratch();
    let mut gs = new_game::create_for_world(&d, WORLD, 1).expect("a new game");
    let mut f = battle::begin_world(&d, &mut gs, &fight(&qid)).unwrap_or_else(|e| panic!("{qid} starts: {e}"));
    let fallen = &captives[0];
    let spawned: usize = captives.iter().map(|r| f.session.units_of_ref(r).len()).sum();
    assert!(spawned >= 1, "the captives fight on the party's side");
    for c in f.session.units_of_ref(fallen) {
        f.session.w.units[c].hp = 0;
    }
    let lived = spawned - f.session.units_of_ref(fallen).len();
    let tally = f.tally();
    let res = battle::finish_world(&d, &mut gs, trigger::FIGHT_SOURCE, &fight(&qid), "won", Some(&tally));
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(gs.flag(FLAG), Some(&json::int(lived as i64)), "the captives that lived after {fallen} fell");
}

#[test]
fn a_settle_without_a_battle_counts_every_captive() {
    let (_tmp, d, qid, captives) = scratch();
    let mut gs = new_game::create_for_world(&d, WORLD, 1).expect("a new game");
    let res = battle::finish_world(&d, &mut gs, trigger::FIGHT_SOURCE, &fight(&qid), "won", None);
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(gs.flag(FLAG), Some(&json::int(captives.len() as i64)));
}
