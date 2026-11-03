//! DISPLAY2, the window's settings: how `fullscreen` and `borderless` read, what an old `window_mode` becomes, the cap to
//! the usable area, where a window goes, and holding a saved size to a game's list. The shell's calls to the OS are
//! `size_and_place` in `shell.rs`, over these rules.

use aegis_ui::screen::{window_capped, window_centred, window_kept_in, window_size_of, ScreenRect, Settings, WindowShow, WINDOW_SIDE_MAX, WINDOW_SIZES};
use serde_json::{json, Value};
use std::path::PathBuf;

/// A settings file holding `text`, in its own folder.
fn file_with(name: &str, text: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("aegis_window_settings_{}_{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("settings.json");
    std::fs::write(&path, text).unwrap();
    path
}

fn load(name: &str, text: &str) -> Settings {
    Settings::load(file_with(name, text), json!({}))
}

#[test]
fn the_defaults_are_a_framed_window() {
    let v = Settings::defaults(json!({}));
    assert_eq!(WindowShow::of(&v), WindowShow::Windowed);
    assert!(v.get("window_mode").is_none(), "the old key is gone");
}

/// Fullscreen beats Borderless; anything but `true` is off.
#[test]
fn the_toggles_read_as_one_show() {
    let show = |v: Value| WindowShow::of(&v);
    assert_eq!(show(json!({"fullscreen": true, "borderless": false})), WindowShow::Fullscreen);
    assert_eq!(show(json!({"fullscreen": true, "borderless": true})), WindowShow::Fullscreen);
    assert_eq!(show(json!({"fullscreen": false, "borderless": true})), WindowShow::Borderless);
    assert_eq!(show(json!({"fullscreen": false, "borderless": false})), WindowShow::Windowed);
    for bad in [Value::Null, json!("true"), json!(1), json!([])] {
        assert_eq!(show(json!({"fullscreen": bad, "borderless": bad})), WindowShow::Windowed, "{bad}");
    }
    assert_eq!(show(json!({})), WindowShow::Windowed, "missing");
}

/// Both old modes that covered the monitor become Fullscreen on; Windowed, an unknown value or a wrong type both off.
#[test]
fn an_old_window_mode_carries_over() {
    for (i, (old, on)) in [(json!("Borderless"), true), (json!("Fullscreen"), true), (json!("Windowed"), false), (json!("Exclusive"), false), (json!(""), false), (json!(3), false), (Value::Null, false)].into_iter().enumerate() {
        let s = load(&format!("old{i}"), &json!({"window_mode": old, "window_size": "1600x900"}).to_string());
        assert_eq!(s.v["fullscreen"], json!(on), "{old}");
        assert_eq!(s.v["borderless"], json!(false), "{old}");
        assert_eq!(s.v["window_size"], json!("1600x900"), "{old}: the size stays");
        assert!(s.v.get("window_mode").is_none(), "{old}: the old key goes");
    }
}

/// The next save writes the new keys only, and a new run reads them back.
#[test]
fn a_carried_over_file_saves_in_the_new_form() {
    let path = file_with("resave", r#"{"window_mode": "Borderless"}"#);
    Settings::load(path.clone(), json!({})).save().unwrap();
    let text: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert!(text.get("window_mode").is_none());
    assert_eq!(text["fullscreen"], json!(true));
    assert_eq!(WindowShow::of(&Settings::load(path, json!({})).v), WindowShow::Fullscreen);
}

/// Dropping the old key leaves the order of the other keys alone, through the next save.
#[test]
fn the_carry_over_keeps_the_key_order() {
    let path = file_with("order", r#"{"zeta": 1, "window_mode": "Windowed", "alpha": 2, "mid": 3, "beta": 4}"#);
    Settings::load(path.clone(), json!({})).save().unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let at = |k: &str| text.find(&format!("\"{k}\"")).unwrap_or_else(|| panic!("{k}"));
    assert!(at("zeta") < at("alpha") && at("alpha") < at("mid") && at("mid") < at("beta"), "{text}");
}

/// A file with the new keys keeps them whatever an old key beside them says.
#[test]
fn the_new_keys_win_over_a_stale_old_one() {
    let s = load("both", r#"{"window_mode": "Fullscreen", "borderless": true}"#);
    assert_eq!(WindowShow::of(&s.v), WindowShow::Borderless);
    assert!(s.v.get("window_mode").is_none());
}

#[test]
fn a_broken_file_gives_the_defaults() {
    for (i, text) in ["", "{", "[1]", "7"].into_iter().enumerate() {
        assert_eq!(WindowShow::of(&load(&format!("broken{i}"), text).v), WindowShow::Windowed, "{text:?}");
    }
}

#[test]
fn a_size_value_is_bounded() {
    assert_eq!(window_size_of(&json!("1280x720")), Some((1280, 720)));
    let top = format!("{WINDOW_SIDE_MAX}x{WINDOW_SIDE_MAX}");
    assert_eq!(window_size_of(&json!(top)), Some((WINDOW_SIDE_MAX, WINDOW_SIDE_MAX)));
    let past = format!("{}x720", WINDOW_SIDE_MAX + 1);
    for bad in [json!(past), json!("0x720"), json!("1280x0"), json!("big"), json!("x"), json!("-1x720"), json!("1280x720x2"), json!("99999999999x1"), json!(1280), Value::Null] {
        assert_eq!(window_size_of(&bad), None, "{bad}");
    }
}

/// Sizes are real pixels: the value is the client size with no scale in the sum, so at 200 % (frame and work area as
/// that monitor reports them, all in px) 1280 x 720 and 1920 x 1080 both fit and stay exact. The shell's request is
/// `size_and_place`, over these two.
#[test]
fn a_size_means_real_pixels_at_any_scale() {
    let (frame, usable) = ((26, 71), (2880, 1704));
    for (s, want) in [("1280x720", (1280, 720)), ("1920x1080", (1920, 1080))] {
        assert_eq!(window_size_of(&json!(s)), Some(want));
        assert_eq!(window_capped(want, frame, usable), want, "{s}");
    }
}

/// A size that fits, frame and all, is left alone, to the pixel.
#[test]
fn the_cap_leaves_a_size_that_fits() {
    for s in WINDOW_SIZES {
        let want = window_size_of(&json!(s)).unwrap();
        assert_eq!(window_capped(want, (16, 39), (want.0 + 16, want.1 + 39)), want, "{s}");
        assert_eq!(window_capped(want, (0, 0), want), want, "{s}: no frame");
    }
}

/// Whatever the size and frame, the capped window fits the area, keeps its shape to a pixel and is as large as the rule
/// lets it be: the side that sets the scale uses all of its room.
#[test]
fn the_cap_fits_and_keeps_the_shape() {
    let usables = [(1920, 1032), (1920, 1080), (1366, 728), (2560, 1392), (1080, 1872), (3440, 1392), (800, 560)];
    for usable in usables {
        for frame in [(0, 0), (16, 39), (32, 78)] {
            for s in WINDOW_SIZES {
                let want = window_size_of(&json!(s)).unwrap();
                let got = window_capped(want, frame, usable);
                let room = (usable.0 - frame.0, usable.1 - frame.1);
                assert!(got.0 <= room.0 && got.1 <= room.1, "{s} {usable:?} {frame:?}: {got:?}");
                if want.0 > room.0 || want.1 > room.1 {
                    assert!(got.0 == room.0 || got.1 == room.1, "{s} {usable:?} {frame:?}: {got:?} wastes room");
                    let cross = (i64::from(got.0) * i64::from(want.1) - i64::from(got.1) * i64::from(want.0)).abs();
                    assert!(cross <= i64::from(want.0.max(want.1)), "{s} {usable:?} {frame:?}: {got:?} changed shape");
                }
            }
        }
    }
}

#[test]
fn the_cap_survives_extremes() {
    assert_eq!(window_capped((WINDOW_SIDE_MAX, WINDOW_SIDE_MAX), (0, 0), (0, 0)), (1, 1), "no room at all");
    assert_eq!(window_capped((1280, 720), (2000, 2000), (1920, 1080)), (1, 1), "a frame past the area");
    assert_eq!(window_capped((u32::MAX, u32::MAX), (0, 0), (1920, 1080)), (1080, 1080), "no overflow");
}

const MONITOR: ScreenRect = ScreenRect { x: 0, y: 0, w: 1920, h: 1080 };
const WORK: ScreenRect = ScreenRect { x: 0, y: 0, w: 1920, h: 1032 };

#[test]
fn a_borderless_window_is_centred_on_the_monitor() {
    assert_eq!(window_centred(MONITOR, WORK, (1280, 720)), (320, 180));
    let (w, h) = (1600u32, 900u32);
    let (x, y) = window_centred(MONITOR, WORK, (w, h));
    assert_eq!(x, (MONITOR.w as i32 - w as i32) / 2);
    assert!((y + h as i32) <= WORK.h as i32, "above the taskbar");
    // another monitor, left of and above the main one, at a negative origin
    let second = ScreenRect { x: -2560, y: -300, w: 2560, h: 1440 };
    assert_eq!(window_centred(second, second, (1280, 720)), (-2560 + 640, -300 + 360));
}

/// A window as tall as the usable area sits on it: the taskbar's share does not push it under.
#[test]
fn a_borderless_window_that_fills_the_usable_area_stays_on_it() {
    let (w, h) = window_capped((1920, 1080), (0, 0), (WORK.w, WORK.h));
    let (x, y) = window_centred(MONITOR, WORK, (w, h));
    assert!(x >= 0 && y == 0 && x + w as i32 <= WORK.w as i32 && y + h as i32 <= WORK.h as i32, "{x} {y} {w} {h}");
    // a taskbar on top: the usable area starts below it
    let top = ScreenRect { x: 0, y: 48, w: 1920, h: 1032 };
    assert_eq!(window_centred(MONITOR, top, (1600, 1032)).1, 48);
}

/// A framed window keeps its place when it fits there, and moves only as far as it must.
#[test]
fn a_framed_window_moves_only_to_stay_on_the_usable_area() {
    assert_eq!(window_kept_in(WORK, (100, 100), (1296, 759)), (100, 100));
    assert_eq!(window_kept_in(WORK, (900, 500), (1296, 759)), (1920 - 1296, 1032 - 759));
    assert_eq!(window_kept_in(WORK, (-50, -50), (1296, 759)), (0, 0));
    assert_eq!(window_kept_in(WORK, (10, 10), (4000, 4000)), (0, 0), "too big: the top left");
}

/// A listed size stays; one off the list goes to the nearest in pixels; anything else to the first; an empty list
/// changes nothing. The engine reads no list of its own.
#[test]
fn a_saved_size_is_held_to_the_games_list() {
    let offered = ["1280x720", "1600x900", "2560x1440"];
    let held = |saved: Value| {
        let mut s = load("held", &json!({"window_size": saved}).to_string());
        s.keep_window_size(&offered);
        s.v["window_size"].clone()
    };
    for s in offered {
        assert_eq!(held(json!(s)), json!(s));
    }
    for (saved, want) in [("960x540", "1280x720"), ("1366x768", "1280x720"), ("1920x1080", "1600x900"), ("3840x2160", "2560x1440")] {
        assert_eq!(held(json!(saved)), json!(want), "{saved}");
    }
    for bad in [json!("big"), json!("0x0"), json!(5), Value::Null] {
        assert_eq!(held(bad.clone()), json!("1280x720"), "{bad}");
    }
    let mut s = load("empty", r#"{"window_size": "960x540"}"#);
    s.keep_window_size(&[]);
    assert_eq!(s.v["window_size"], json!("960x540"));
}
