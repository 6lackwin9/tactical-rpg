//! PR7-42 (PACE1, PT-17) on the shipped data (read only): a quest's recommended level (`rec_level`) shows on its
//! board row and detail, and each board section lists its rated quests by that level, lowest first, before the
//! unrated ones (file order among equals). On the first Act 1 board the easier first job is offered above the harder
//! one. No quest, board or level is named: they are read from the data.

use aegis_core::json::{self, Dict};
use emberwake_world::{board, new_game, quest, Data};
use std::path::Path;

const WORLD: &str = "sunk_ward";

fn data() -> Data {
    Data::open(&Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")).unwrap().0
}

fn rows(view: &Dict) -> Vec<(String, Vec<Dict>)> {
    json::aget(view, "sections").iter().map(json::dict).map(|s| (json::s(s, "id"), json::aget(s, "rows").iter().map(|r| json::dict(r).clone()).collect())).collect()
}

#[test]
fn board_rows_show_and_follow_the_recommended_level() {
    let d = data();
    let gs = new_game::create_for_world(&d, WORLD, 1).expect("a new game");
    let mut rated_boards = 0;
    for b in board::boards(&d, &gs) {
        let qid = json::s(b, "_qid");
        let order = board::quests(&d, &gs, &qid);
        let view = board::view(&d, &gs, &qid);
        for (section, list) in rows(&view) {
            let levels: Vec<i64> = list.iter().map(|r| json::i(r, "rec_level", 0)).collect();
            for (r, lv) in list.iter().zip(&levels) {
                let q = json::s(r, "quest");
                assert_eq!(*lv, board::rec_level(&d, &gs, &q), "{q}: the row's level is the quest's");
                assert_eq!(json::i(&board::detail(&d, &gs, &q), "rec_level", 0), *lv, "{q}: the detail's level is the row's");
            }
            let rated: Vec<i64> = levels.iter().copied().take_while(|l| *l > 0).collect();
            assert!(levels[rated.len()..].iter().all(|l| *l == 0), "{qid} {section}: rated rows first: {levels:?}");
            assert!(rated.windows(2).all(|w| w[0] <= w[1]), "{qid} {section}: lowest level first: {levels:?}");
            for w in list.windows(2) {
                let (a, b) = (json::s(&w[0], "quest"), json::s(&w[1], "quest"));
                let (la, lb) = (json::i(&w[0], "rec_level", 0), json::i(&w[1], "rec_level", 0));
                if la == lb {
                    let pos = |q: &str| order.iter().position(|x| x == q).unwrap();
                    assert!(pos(&a) < pos(&b), "{qid} {section}: file order among equals ({a}, {b})");
                }
            }
            if rated.len() >= 2 {
                rated_boards += 1;
            }
        }
    }
    assert!(rated_boards > 0, "a {WORLD} board offers two rated jobs side by side");
}

#[test]
fn the_level_reads_as_words() {
    assert_eq!(board::rec_level_text(0), "");
    assert_eq!(board::rec_level_text(5), "Recommended level 5");
}

/// PR7-78 (ACT1-DATA1, FB-388) and PR7-84 (ACT1-DATA2, PT-77): every job the board offers that sends the party to set
/// a flag (a `flag_job` template) and every main raid (a `main_raid` template) shows a recommended level.
#[test]
fn every_flag_job_on_a_board_has_a_level() {
    let d = data();
    let gs = new_game::create_for_world(&d, WORLD, 1).expect("a new game");
    let mut jobs = 0;
    for b in board::boards(&d, &gs) {
        for q in board::quests(&d, &gs, &json::s(b, "_qid")) {
            let t = json::s(&quest::resolved(&d, &gs, &q), "template");
            if t.contains("flag_job") || t.contains("main_raid") {
                jobs += 1;
                assert!(board::rec_level(&d, &gs, &q) > 0, "{q}: a flag job or main raid shows a recommended level");
            }
        }
    }
    assert!(jobs > 0, "a {WORLD} board offers a flag job");
}
