//! PR7-47 (POLISH-FIX4 item 4, `FB-344`) on the shipped data (read only): every character of a new Sunk Ward game who is
//! asked "What do you sell?" answers it, either with a shop or with a short line of their own (Widow Varka's
//! closed-shop line, PR7-37), and the vendor-like keepers with no shop (the pawnbroker, the lodging keeper and Skerrit
//! before he is vouched for) are among them. No content numbers: the answer is read through the dialogue runner.

use aegis_core::json;
use emberwake_world::dialogue::DialogueRunner;
use emberwake_world::{new_game, Data, GameState};
use std::path::Path;

const WORLD: &str = "sunk_ward";
const ASK: &str = "What do you sell?";
/// The keepers the playtest found without an answer (`FB-344` question 1).
const KEEPERS: [&str; 3] = ["cit_pawnbroker", "cit_lodging", "skerrit_bone_wright"];
/// A no-trade line stays short.
const LINE_MAX: usize = 120;

fn data() -> Data {
    Data::open(&Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")).unwrap().0
}

/// The answer of character `id` to `ASK` in `gs`: (shop opened, line said), None when not offered.
fn answer(d: &Data, gs: &GameState, id: &str) -> Option<(String, String)> {
    let mut gs = gs.clone();
    let mut r = DialogueRunner::open(d, &mut gs, id);
    let view = r.view(d, &gs);
    let at = json::aget(&view, "choices").iter().map(json::dict).position(|c| json::s(c, "text") == ASK && !json::b(c, "locked", false))?;
    let res = r.choose(d, &mut gs, at as i64);
    assert!(json::b(&res, "ok", false), "{id}: {:?}", json::sget(&res, "errors"));
    let after = r.view(d, &gs);
    Some((json::s(&after, "shop"), json::s(&after, "text")))
}

#[test]
fn whoever_is_asked_what_they_sell_answers() {
    let d = data();
    let gs = new_game::create_for_world(&d, WORLD, 1).expect("a new game");
    let mut asked = Vec::new();
    for c in d.catalogs.own_entries(WORLD, "characters") {
        let id = json::s(c, "id");
        let Some((shop, line)) = answer(&d, &gs, &id) else { continue };
        assert!(!shop.is_empty() || !line.trim().is_empty(), "{id} answers with a shop or a line");
        asked.push(id);
    }
    for k in KEEPERS {
        assert!(asked.iter().any(|a| a == k), "{k} is asked what they sell: {asked:?}");
        let (shop, line) = answer(&d, &gs, k).expect("offered");
        assert!(shop.is_empty(), "{k} keeps no shop");
        assert!(!line.is_empty() && line.len() <= LINE_MAX, "{k}'s no-trade line is short: {line:?}");
    }
}
