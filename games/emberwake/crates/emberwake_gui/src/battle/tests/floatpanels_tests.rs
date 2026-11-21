//! FLOATPANELS1 (`PR7-33`; `preferences/ui.md` "Floating panels", `P-UI-03`): the battle log opens as a floating
//! panel over the board (`aegis_ui::float_panel`): at the board's top left, locked; unlocked it drags over the board
//! and its place and lock are kept in the settings; minimize shrinks it in place to a "Combat log" button that opens
//! it again where it was; Close and L (the screen's `show_log`) still hide it.

use crate::battle::draw::LOG_AREA;
use crate::battle::{Action, BattleScreen};
use crate::screens::{Cx, Screen};
use aegis_ui::canvas::Rect;
use aegis_ui::screen::Settings;
use aegis_ui::ui::Gfx;
use emberwake_rules::ruleset::TIMELINE;
use serde_json::json;

fn fight() -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    let mut b = BattleScreen::arena_with(&cx, Some("sunk_ward".into()), "enc_evt_01", Some(TIMELINE.to_string()), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    let mut g = crate::hud::hud_tests::gfx();
    b.tick(&mut cx);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    (cx, g, b)
}

fn at(b: &BattleScreen, id: &str) -> Option<Rect> {
    b.w.iter().find(|w| w.id == id).map(|w| w.rect)
}

fn redraw(cx: &mut Cx, g: &mut Gfx, b: &mut BattleScreen) {
    b.draw(cx, g, (-500, -500), true);
}

#[test]
fn the_battle_log_floats_minimizes_in_place_and_keeps_its_lock() {
    let (mut cx, mut g, mut b) = fight();
    assert!(at(&b, "log.panel").is_none(), "closed until Expand");
    b.show_log = true;
    redraw(&mut cx, &mut g, &mut b);
    let r = at(&b, "log.panel").expect("the floating log");
    assert_eq!((r.x, r.y), (LOG_AREA.x + 4, LOG_AREA.y + 4), "the board's top left");
    assert!(r.w < LOG_AREA.w, "narrower than the board: the board shows beside it");
    assert!(at(&b, "log.lines").is_some() && at(&b, "log.panel.lock").is_some() && at(&b, "log.panel.min").is_some());
    // unlock: kept in the settings
    b.run(&mut cx, Action::LogLock);
    redraw(&mut cx, &mut g, &mut b);
    assert!(b.w.iter().any(|w| w.id == "log.panel" && w.drag.as_deref() == Some("battle.log")), "unlocked: it drags");
    redraw(&mut cx, &mut g, &mut b);
    let saved = Settings::load(cx.shot_dir.join("settings.json"), crate::screens::default_settings());
    assert_eq!(saved.v[crate::floats::BATTLE_LOG_KEY], json!({"pos": [r.x, r.y], "locked": false}));
    // minimize toggles in place
    let min = at(&b, "log.panel.min").unwrap();
    b.run(&mut cx, Action::LogMin);
    redraw(&mut cx, &mut g, &mut b);
    assert!(at(&b, "log.panel").is_none() && at(&b, "log.lines").is_none(), "minimized");
    let chip = b.w.iter().find(|w| w.id == "log.panel.open").expect("the Combat log button");
    assert_eq!(chip.text, "Combat log");
    assert_eq!((chip.rect.x + chip.rect.w, chip.rect.y), (min.x + min.w, min.y - 1), "where the minimize button was");
    b.run(&mut cx, Action::LogOpen);
    redraw(&mut cx, &mut g, &mut b);
    assert_eq!(at(&b, "log.panel.min"), Some(min), "restored in place");
    // Close hides it
    b.show_log = false;
    redraw(&mut cx, &mut g, &mut b);
    assert!(at(&b, "log.panel").is_none() && at(&b, "log.panel.open").is_none());
}
