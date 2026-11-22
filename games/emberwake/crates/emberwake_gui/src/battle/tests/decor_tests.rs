//! DECOR1 (PR7-81, PR7-79 follow-up): cover decor draws on the shipped data's boards through both arena paths: the
//! launch path (`screen=arena`, `scene=<world>:<id>`: `BattleScreen::sandbox`) and `arena_with`. Each opens the world's
//! encounter with the world's stage set, so every cover decor resolves to its set's cover art and stands in the feet
//! pass, standing in place of an inner block's own cover. The encounters are picked by rule (every world encounter
//! with a cover decor in a world naming a stage set), never by id.

use crate::battle::BattleScreen;
use crate::stage::{self, Item};
use aegis_core::json;
use aegis_ui::art::Art;
use emberwake_content::art::{decor_art, DecorArt};
use emberwake_world::dialogue;

#[test]
fn cover_decor_draws_through_both_arena_paths_on_the_shipped_boards() {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    let mut picked: Vec<(String, String)> = Vec::new();
    for w in cx.data.catalogs.world_ids() {
        if dialogue::stage_set(&cx.data, &w).is_empty() {
            continue;
        }
        for e in cx.data.catalogs.merged_entries(&w, "encounters") {
            let covers = json::arr(json::get(e, "decor")).iter().any(|d| matches!(decor_art(&json::s(json::dict(d), "art")), Some(DecorArt::Cover(_))));
            if covers {
                picked.push((w.clone(), json::s(e, "id")));
            }
        }
    }
    assert!(!picked.is_empty(), "the shipped boards hold cover decor (PR7-79)");
    for (w, e) in &picked {
        cx.app.encounter = format!("{w}:{e}");
        for (path, b) in [("launch", BattleScreen::sandbox(&cx)), ("arena_with", BattleScreen::arena_with(&cx, Some(w.clone()), e, None, "", false))] {
            let set = b.stage_set(&cx);
            assert_eq!(set, dialogue::stage_set(&cx.data, w), "{path} {w}:{e}: the world's stage set");
            let s = b.host.session.as_ref().unwrap_or_else(|| panic!("{path} {w}:{e}: {}", b.message));
            let look = stage::zone_look(&mut cx.art, &set, &s.def.zone);
            let items = stage::decor_items(&mut cx.art, &s.w.grid, (&set, &look), &s.def.decor);
            for (k, d) in s.def.decor.iter().enumerate().filter(|(_, d)| matches!(decor_art(&d.art), Some(DecorArt::Cover(_)))) {
                assert!(items.iter().any(|p| p.item == Item::Decor(k)), "{path} {w}:{e}: {d:?} draws");
                if stage::shown_cover(&mut cx.art, &s.w.grid, (&set, &look), d.at).is_some() {
                    assert!(!stage::block_shown(&mut cx.art, &s.w.grid, (&set, &look), &s.def.decor, d.at), "{path} {w}:{e}: {d:?} stands in place of the block's cover");
                }
            }
        }
    }
}
