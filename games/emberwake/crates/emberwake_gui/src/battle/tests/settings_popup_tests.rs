//! PR7-26 (2) (`PLAYTEST-UI1`): a world battle opens the decluttered in-game Settings popup (`SettingsScreen::in_game`)
//! from Esc once nothing else is open, and from its bottom bar's Menu button; Esc still closes what is open first; the
//! popup's Return to title keeps the hub's busy check (a fight is open: a notice, no reset). The Test Arena's Esc still
//! leaves. On the scratch world of `transitions_tests` (no shipped data).

use super::transitions_tests::{battle, beat_cx};
use crate::battle::{Action, BattleScreen};
use crate::hud::hud_tests::{frame, gfx};
use crate::screens::{Cx, GameCx, Nav, Screen};
use serde_json::json;

/// The screen pushed last, if `cx.nav` holds a push.
fn pushed(cx: &mut Cx) -> Option<Box<dyn Screen<GameCx>>> {
    match cx.nav.pop() {
        Some(Nav::Push(s)) => Some(s),
        _ => None,
    }
}

fn map_battle(cx: &mut Cx) -> BattleScreen {
    let mut b = battle(cx, "map");
    b.tick(cx);
    assert!(b.beat.is_none(), "a fight met on the map has no beat");
    b
}

#[test]
fn esc_with_nothing_open_opens_the_in_game_settings_popup() {
    let mut cx = beat_cx(false, false);
    let mut g = gfx();
    let mut b = map_battle(&mut cx);
    frame(&mut b, &mut cx, &mut g, 1.0);
    // an open panel closes first, with no popup
    b.show_log = true;
    b.escape(&mut cx);
    assert!(!b.show_log && cx.nav.is_empty(), "Esc closes the log first");
    // then the selection (timeline rules), then the popup
    let mut presses = 0;
    while cx.nav.is_empty() && presses < 4 {
        b.escape(&mut cx);
        presses += 1;
    }
    let s = pushed(&mut cx).expect("Esc pushed a screen");
    assert_eq!(s.id(), "settings");
    assert_eq!(s.state(&cx)["popup"], json!(true), "the decluttered in-game popup");
    assert!(s.overlay(), "drawn over the battle");
}

#[test]
fn the_world_bar_menu_button_opens_the_popup_and_the_arena_keeps_exit() {
    let mut cx = beat_cx(false, false);
    let mut g = gfx();
    let mut b = map_battle(&mut cx);
    frame(&mut b, &mut cx, &mut g, 1.0);
    let ids: Vec<String> = b.widgets().iter().map(|w| w["id"].as_str().unwrap_or_default().to_string()).collect();
    assert!(ids.iter().any(|i| i == "bar.menu") && !ids.iter().any(|i| i == "bar.exit"), "a world battle has Menu, no Exit: {ids:?}");
    b.click_id(&mut cx, "bar.menu").expect("the Menu button");
    let s = pushed(&mut cx).expect("Menu pushed a screen");
    assert_eq!((s.id(), s.state(&cx)["popup"].clone()), ("settings", json!(true)));
    b.act(&mut cx, Action::Settings);
    assert!(matches!(pushed(&mut cx), Some(s) if s.id() == "settings"));
}

#[test]
fn return_to_title_from_a_battle_is_refused_while_the_fight_runs() {
    let mut cx = beat_cx(false, false);
    let mut g = gfx();
    let mut b = map_battle(&mut cx);
    frame(&mut b, &mut cx, &mut g, 1.0);
    assert!(cx.busy(), "a world fight marks the game busy");
    b.act(&mut cx, Action::Settings);
    let mut s = pushed(&mut cx).expect("the popup");
    s.draw(&mut cx, &mut g, (-1, -1), true);
    s.click_id(&mut cx, "settings.title").expect("Return to title");
    assert!(cx.has_game(), "the game stays");
    assert!(!cx.nav.iter().any(|n| matches!(n, Nav::Reset(_))), "no reset to the title");
    assert!(cx.nav.iter().any(|n| matches!(n, Nav::Push(x) if x.id() == "notice")), "a notice says why");
}

/// The proof shot (only with `AEGIS_REVIEW_DIR` set): a world battle at 1280 x 720 with Esc's Settings popup over it,
/// as `battle_settings_popup.png`, and the battle alone (its bottom bar's Menu button) as `battle_menu_button.png`.
#[test]
fn battle_settings_popup_shot() {
    let Ok(out) = std::env::var("AEGIS_REVIEW_DIR") else { return };
    let mut cx = beat_cx(false, false);
    let mut b = map_battle(&mut cx);
    let mut g = gfx();
    g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
    g.view = aegis_ui::painter::View::fit(1280, 720);
    let write = |g: &aegis_ui::ui::Gfx, name: &str| {
        let file = std::fs::File::create(std::path::Path::new(&out).join(name)).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), 1280, 720);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&g.canvas.px).unwrap();
    };
    b.draw(&mut cx, &mut g, (-500, -500), true);
    write(&g, "battle_menu_button.png");
    b.act(&mut cx, Action::Settings);
    let mut s = pushed(&mut cx).expect("the popup");
    s.draw(&mut cx, &mut g, (-500, -500), true);
    write(&g, "battle_settings_popup.png");
}
