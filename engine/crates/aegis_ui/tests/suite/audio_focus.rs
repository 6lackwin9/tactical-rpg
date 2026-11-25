//! PR7-26 (`PLAYTEST-UI1` 5): the background mute. With the engine setting `mute_in_background` on (its default) a
//! window that loses the focus plays at master 0 and the level comes back with the focus; off, the focus changes
//! nothing; the settings' volumes are never written. On an offline `Audio` (no device: the levels still follow).

use aegis_ui::audio::{Audio, Bus, MUTE_IN_BACKGROUND};
use aegis_ui::screen::Settings;
use serde_json::json;

fn settings(on: Option<bool>) -> serde_json::Value {
    let mut v = Settings::engine_defaults();
    if let Some(on) = on {
        v[MUTE_IN_BACKGROUND] = json!(on);
    }
    v
}

#[test]
fn the_default_is_on() {
    assert_eq!(Settings::engine_defaults()[MUTE_IN_BACKGROUND], json!(true));
}

#[test]
fn a_lost_focus_mutes_when_on_and_the_focus_back_restores() {
    for (on, label) in [(None, "default"), (Some(true), "on")] {
        let v = settings(on);
        let a = Audio::offline(None);
        a.set_levels(&v);
        let full: Vec<f32> = [Bus::Music, Bus::Sfx, Bus::Ui].iter().map(|b| a.level(*b)).collect();
        assert!(full.iter().all(|l| *l > 0.0), "{label}: audible while focused");
        a.set_focused(false, &v);
        assert!(!a.focused());
        for b in [Bus::Music, Bus::Sfx, Bus::Ui] {
            assert_eq!(a.level(b), 0.0, "{label}: {b:?} silent in the background");
        }
        a.set_levels(&v);
        assert_eq!(a.level(Bus::Music), 0.0, "{label}: the frame's set_levels keeps it silent");
        a.set_focused(true, &v);
        let back: Vec<f32> = [Bus::Music, Bus::Sfx, Bus::Ui].iter().map(|b| a.level(*b)).collect();
        assert_eq!(back, full, "{label}: restored with the focus");
        assert_eq!(v, settings(on), "{label}: the settings are untouched");
    }
}

#[test]
fn off_the_focus_changes_nothing() {
    let v = settings(Some(false));
    let a = Audio::offline(None);
    a.set_levels(&v);
    let full = a.level(Bus::Music);
    a.set_focused(false, &v);
    assert_eq!(a.level(Bus::Music), full, "off: still audible in the background");
    a.set_focused(true, &v);
    assert_eq!(a.level(Bus::Music), full);
}

#[test]
fn mute_all_stays_silent_with_the_focus_back() {
    let mut v = settings(None);
    v["mute"] = json!(true);
    let a = Audio::offline(None);
    a.set_focused(false, &v);
    a.set_focused(true, &v);
    assert_eq!(a.level(Bus::Music), 0.0, "Mute all is its own switch");
}
