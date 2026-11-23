//! PR7-92 (INPUT-REACH2, FB-489 call 4): the battle board is one Tab stop. Tab passes it once, on to the strip, the
//! panels and End turn; on it the arrows walk every tile and token, and Enter presses one as a click.

use crate::battle::{BattleScreen, BOARD_GROUP};
use crate::screens::Screen;
use aegis_ui::focus::{self, Move};
use aegis_ui::reach::{self, Device};
use serde_json::json;

#[test]
fn the_board_is_one_tab_stop_and_the_arrows_walk_it() {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, None, "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    let mut g = crate::hud::hud_tests::gfx();
    crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
    let infos = b.widgets();
    let group = b.focus_group();
    assert_eq!(group, BOARD_GROUP);
    let plain = focus::ring(&infos, false);
    let board = plain.iter().filter(|w| focus::in_group(w, group)).count();
    assert!(board > 20, "a board of tiles and tokens: {board}");
    let ring = focus::ring_grouped(&infos, false, group, None, None);
    assert_eq!(ring.iter().filter(|w| focus::in_group(w, group)).count(), 1, "the board is one stop");
    assert_eq!(ring.len(), plain.len() - board + 1);
    // Tab from the board leaves it at once
    let stop = ring.iter().find(|w| focus::in_group(w, group)).and_then(|w| w["id"].as_str()).unwrap().to_string();
    let next = focus::step_grouped(&infos, Some(&stop), Move::Next, false, group, None).unwrap();
    assert!(!next.starts_with("tile.") && !next.starts_with("unit."), "Tab goes on past the board: {next}");
    // the arrows walk the board from the stop and stay on it
    for d in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
        let to = focus::step_grouped(&infos, Some(&stop), Move::Dir(d.0, d.1), false, group, None).unwrap();
        assert!(to.starts_with("tile.") || to.starts_with("unit."), "{d:?}: {to}");
    }
    let keys = reach::device_reach_grouped(&infos, Device::keys(), b.arrow_focus(), None, group);
    let miss: Vec<String> = reach::unreached(&infos, &keys).into_iter().map(|u| u.0).collect();
    assert!(miss.is_empty(), "the keys still reach every tile and token: {miss:?}");
    // Enter on a tile presses it as a click would
    let tile = plain.iter().find(|w| w["id"].as_str().is_some_and(|i| i.starts_with("tile."))).and_then(|w| w["id"].as_str()).unwrap().to_string();
    assert!(!b.press_focus(&mut cx, &tile, aegis_ui::screen::Key::Enter), "Enter stays a click on the board");
    assert!(b.click_id(&mut cx, &tile).is_ok());
}
