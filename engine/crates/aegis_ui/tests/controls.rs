//! INPUT-MAP1 (ENGINE-7): the control map read from settings, its JSON, the held-direction repeat and the stick.

use aegis_core::clock::{Duration, Instant};
use aegis_ui::controls::{self, Controls, Pad, Repeat, PADS};
use aegis_ui::screen::Key;
use serde_json::json;

#[test]
fn the_defaults_leave_keys_alone_and_give_the_pad_its_usual_meanings() {
    let c = Controls::default();
    for k in [Key::Up, Key::Enter, Key::Char('w'), Key::Esc] {
        assert_eq!(c.key(k), k);
    }
    assert_eq!((c.pad(Pad::South), c.pad(Pad::East), c.pad(Pad::Start)), (Key::Enter, Key::Esc, Key::Esc));
    assert_eq!((c.pad(Pad::DLeft), c.pad(Pad::LeftBumper)), (Key::Left, Key::PageUp));
    assert_eq!(Controls::from_settings(&json!({})), (Controls::default(), Vec::new()));
    for (p, n) in PADS {
        assert_eq!(Pad::parse(n), Some(p));
        assert_eq!(p.name(), n);
    }
}

#[test]
fn settings_rebind_keys_and_buttons_and_name_each_problem() {
    let (c, problems) = Controls::from_settings(&json!({"controls": {"keys": {"i": "up", "j": "left", "w": "w", "x": "ctrl+enter", "q": "nonsense"}, "pad": {"south": "space", "moon": "up", "east": 3}, "mouse": {}}}));
    assert_eq!((c.key(Key::Char('i')), c.key(Key::Char('j')), c.key(Key::Char('w'))), (Key::Up, Key::Left, Key::Char('w')));
    assert_eq!(c.key(Key::Char('x')), Key::Char('x'), "the shell's own Ctrl+Enter cannot be bound");
    assert_eq!((c.pad(Pad::South), c.pad(Pad::East)), (Key::Space, Key::Esc), "a bad entry keeps its default");
    let all = problems.join(" | ");
    for want in ["'ctrl+enter' is not a key name", "'nonsense' is not a key name", "'moon' is not a button", "pad 'east'", "unknown key 'mouse'"] {
        assert!(all.contains(want), "{want} in {all}");
    }
    // What it writes reads back the same.
    let (again, none) = Controls::from_settings(&json!({"controls": c.to_json()}));
    assert_eq!((again, none), (c.clone(), Vec::<String>::new()));
    assert_eq!(c.to_json()["keys"], json!({"i": "up", "j": "left"}), "only rebound keys are written");
    assert!(Controls::from_settings(&json!({"controls": [1]})).1[0].contains("an object"));
}

#[test]
fn a_held_direction_repeats_after_a_pause_and_stops_when_let_go() {
    let t0 = Instant::now();
    let mut r = Repeat::default();
    assert_eq!(r.set(Some(Key::Left), t0), Some(Key::Left), "the first press sends at once");
    assert_eq!(r.set(Some(Key::Left), t0), None, "the same direction does not send again");
    assert_eq!(r.poll(t0 + Duration::from_millis(100)), None);
    assert_eq!(r.poll(t0 + controls::REPEAT_FIRST), Some(Key::Left));
    assert_eq!(r.poll(t0 + controls::REPEAT_FIRST + Duration::from_millis(50)), None);
    assert_eq!(r.poll(t0 + controls::REPEAT_FIRST + controls::REPEAT_EVERY), Some(Key::Left));
    assert_eq!(r.held(), Some(Key::Left));
    assert_eq!(r.set(Some(Key::Up), t0), Some(Key::Up), "a new direction sends at once");
    assert_eq!(r.set(None, t0), None);
    assert_eq!((r.held(), r.poll(t0 + Duration::from_secs(5)), r.due()), (None, None, None));
}

#[test]
fn the_stick_counts_past_its_dead_zone_along_its_larger_axis() {
    assert_eq!(controls::stick_dir(0.2, -0.3), None);
    assert_eq!(controls::stick_dir(0.9, 0.4), Some(Key::Right));
    assert_eq!(controls::stick_dir(-0.6, 0.0), Some(Key::Left));
    assert_eq!(controls::stick_dir(0.1, 0.8), Some(Key::Up), "pads report up as positive");
    assert_eq!(controls::stick_dir(0.3, -0.7), Some(Key::Down));
}
