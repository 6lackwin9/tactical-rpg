//! VIEWKEYS1: debug hotkeys to judge the view without opening Settings (user 2026-11-03). F6 cycles the map zoom on the
//! region map, F9 cycles the window size on the screens that call `window_key` (the region and world maps, the battle,
//! the game menu, the main menu). Each press saves like a normal setting and shows the new value; the shell applies a
//! window size when it sees the setting change. A release build ignores both keys. The Settings screen does not call
//! them: its draft would overwrite the key's value on Apply. VIEWKEYS2: F11 cycles the walk step on the region map.

use crate::mapview::{zoom_of, WalkStep, WALK_STEP_KEY};
use crate::screens::{show_errors, Cx, Key};
use serde_json::{json, Value};

/// F6's steps, in the quarters of `mapview::ZOOM_QUARTERS`: 1x, 1.5x, 2x.
pub const ZOOM_CYCLE: [i32; 3] = [4, 6, 8];
/// F9's steps, as `window_size` values.
pub const SIZE_CYCLE: [&str; 3] = ["1280x720", "1600x900", "1920x1080"];

/// F11's steps, as `walk_step_ms` values.
pub const WALK_CYCLE: [f64; 5] = [90.0, 110.0, 130.0, 160.0, 200.0];

/// Where a screen shows the key's note: the region map's log ticker, or a notice on a screen with no log in view.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shown {
    Log,
    Notice,
}

/// The first step above `now` (a step or any other value); past the last step, the first.
fn next_above<T: Copy + PartialOrd>(steps: &[T], now: T) -> T {
    steps.iter().copied().find(|s| *s > now).unwrap_or(steps[0])
}

/// The zoom setting F6 goes to from `now` (read as the map reads it, so it steps from the zoom on screen).
pub fn next_zoom(now: &Value) -> f64 {
    next_above(&ZOOM_CYCLE, zoom_of(now)) as f64 / 4.0
}

/// The step F11 goes to from `now`: the first step above it; past the last step, or from no number, the first (a value
/// out of `WALK_STEP_MS` also lands on the first: below it the first step is above, over it the steps are past).
pub fn next_walk(now: &Value) -> f64 {
    now.as_f64().map_or(WALK_CYCLE[0], |b| next_above(&WALK_CYCLE, b))
}

/// A `window_size` value's pixel count (None: not a size).
fn pixels(size: &str) -> Option<u64> {
    let (w, h) = size.split_once('x')?;
    Some(u64::from(w.parse::<u32>().ok()?) * u64::from(h.parse::<u32>().ok()?))
}

/// The size F9 goes to from `now`: the first step with more pixels; past the last step, or from a value that is no
/// size, the first.
pub fn next_size(now: &Value) -> &'static str {
    let Some(have) = now.as_str().and_then(pixels) else { return SIZE_CYCLE[0] };
    SIZE_CYCLE.into_iter().find(|s| pixels(s).is_some_and(|p| p > have)).unwrap_or(SIZE_CYCLE[0])
}

/// Why F9 may not size the window in this `window_mode` (the modes the shell does not size); None when it may.
pub fn size_refusal(mode: &Value) -> Option<String> {
    let mode = mode.as_str().filter(|m| matches!(*m, "Borderless" | "Fullscreen"))?;
    Some(format!("F9: the window size only applies in Windowed mode (now {mode})."))
}

fn show(cx: &mut Cx, line: String, shown: Shown) {
    match shown {
        Shown::Log => cx.log(line),
        Shown::Notice => cx.notice.push(line),
    }
}

fn save(cx: &mut Cx) {
    if let Err(e) = cx.settings.save() {
        show_errors(cx, vec![format!("Settings could not be saved: {e}")]);
    }
}

/// F6 on the region map: whether `k` was the key (a release build takes it and does nothing).
pub fn zoom_key(cx: &mut Cx, k: Key) -> bool {
    press_zoom(cx, k, cfg!(debug_assertions))
}

/// F9 on a screen: whether `k` was the key (a release build takes it and does nothing).
pub fn window_key(cx: &mut Cx, k: Key, shown: Shown) -> bool {
    press_window(cx, k, shown, cfg!(debug_assertions))
}

/// F11 on the region map: whether `k` was the key (a release build takes it and does nothing).
pub fn walk_key(cx: &mut Cx, k: Key) -> bool {
    press_walk(cx, k, cfg!(debug_assertions))
}

/// The walk's step time for the F6 and F11 notes, so it reads while comparing zoom levels: the base and, when the Animation speed
/// changes it, the time it makes.
fn walk_words(step: WalkStep) -> String {
    if step.wall == step.base {
        format!("{:.0} ms", step.base)
    } else {
        format!("{:.0} ms ({:.0} ms at this Animation speed)", step.base, step.wall)
    }
}

fn press_zoom(cx: &mut Cx, k: Key, debug: bool) -> bool {
    if k != Key::Fn(6) {
        return false;
    }
    if debug {
        let to = next_zoom(&cx.settings.v["map_zoom"]);
        cx.settings.v["map_zoom"] = json!(to);
        save(cx);
        let line = format!("Map zoom {to}x, walk step {} (F6).", walk_words(WalkStep::of(&cx.settings.v)));
        cx.log(line);
    }
    true
}

fn press_walk(cx: &mut Cx, k: Key, debug: bool) -> bool {
    if k != Key::Fn(11) {
        return false;
    }
    if debug {
        let to = next_walk(&cx.settings.v[WALK_STEP_KEY]);
        cx.settings.v[WALK_STEP_KEY] = json!(to);
        save(cx);
        let line = format!("Walk step {} (F11).", walk_words(WalkStep::of(&cx.settings.v)));
        cx.log(line);
    }
    true
}

fn press_window(cx: &mut Cx, k: Key, shown: Shown, debug: bool) -> bool {
    if k != Key::Fn(9) {
        return false;
    }
    if debug {
        match size_refusal(&cx.settings.v["window_mode"]) {
            Some(why) => show(cx, why, shown),
            None => {
                let to = next_size(&cx.settings.v["window_size"]);
                cx.settings.v["window_size"] = json!(to);
                save(cx);
                show(cx, format!("Window {to} (F9)."), shown);
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::hud_tests::game_cx;
    use crate::mapview::{zoom_steps, WALK_STEP_MS, ZOOM_QUARTERS};
    use crate::screens::Screen;
    use aegis_ui::screen::{Settings, WINDOW_MODES, WINDOW_SIZES};

    /// `game_cx`'s settings file, in its own folder.
    fn file_of(cx: &Cx) -> std::path::PathBuf {
        cx.shot_dir.join("settings.json")
    }

    fn saved(cx: &Cx, key: &str) -> Value {
        let text = std::fs::read_to_string(file_of(cx)).expect("the settings file");
        serde_json::from_str::<Value>(&text).expect("json")[key].clone()
    }

    /// Every cycle step is a value the Settings rows offer, so a key never saves what they cannot show.
    #[test]
    fn the_steps_are_settings_values() {
        for q in ZOOM_CYCLE {
            assert!((ZOOM_QUARTERS.0..=ZOOM_QUARTERS.1).contains(&q), "{q}");
            assert!(zoom_steps().any(|z| z == q as f64 / 4.0), "{q}");
        }
        for s in SIZE_CYCLE {
            assert!(WINDOW_SIZES.contains(&s), "{s}");
        }
    }

    /// F6 walks the steps in order and comes back to the first; the steps are 1x, 1.5x, 2x.
    #[test]
    fn f6_cycles_in_order() {
        assert_eq!(next_zoom(&json!(1.0)), 1.5);
        assert_eq!(next_zoom(&json!(1.5)), 2.0);
        assert_eq!(next_zoom(&json!(2.0)), 1.0);
    }

    /// Every other zoom the Settings row offers goes to the next step above; past the last, the first.
    #[test]
    fn f6_from_an_off_cycle_value_goes_to_the_next_step_above() {
        let top = *ZOOM_CYCLE.last().unwrap() as f64 / 4.0;
        for z in zoom_steps().filter(|z| !ZOOM_CYCLE.contains(&((z * 4.0) as i32))) {
            let want = ZOOM_CYCLE.iter().map(|q| *q as f64 / 4.0).find(|s| *s > z).unwrap_or(1.0);
            assert_eq!(next_zoom(&json!(z)), want, "from {z}");
            assert_eq!(z > top, want == 1.0, "only past the last wraps: {z}");
        }
        assert_eq!(next_zoom(&Value::Null), 1.5, "missing reads as the default 1x");
    }

    /// F9 walks the steps in order and comes back to the first.
    #[test]
    fn f9_cycles_in_order() {
        assert_eq!(next_size(&json!("1280x720")), "1600x900");
        assert_eq!(next_size(&json!("1600x900")), "1920x1080");
        assert_eq!(next_size(&json!("1920x1080")), "1280x720");
    }

    /// Every other size the Settings row offers goes to the next step above; past the last, or no size at all, the first.
    #[test]
    fn f9_from_an_off_cycle_value_goes_to_the_next_step_above() {
        let area = |s: &str| pixels(s).expect("a size");
        for s in WINDOW_SIZES.into_iter().filter(|s| !SIZE_CYCLE.contains(s)) {
            let want = SIZE_CYCLE.into_iter().find(|c| area(c) > area(s)).unwrap_or(SIZE_CYCLE[0]);
            assert_eq!(next_size(&json!(s)), want, "from {s}");
        }
        assert_eq!(next_size(&json!("960x540")), "1280x720", "below the first");
        assert_eq!(next_size(&json!("2560x1440")), "1280x720", "above the last");
        for bad in [json!("big"), json!("x"), json!("99999999999x1"), json!(7), Value::Null] {
            assert_eq!(next_size(&bad), "1280x720", "{bad}");
        }
    }

    /// F6 saves the new zoom, reads the setting the map draws from and notes the value; other keys pass.
    #[test]
    fn f6_saves_and_notes_the_value() {
        let mut cx = game_cx();
        assert!(!press_zoom(&mut cx, Key::Fn(9), true), "not its key");
        assert!(press_zoom(&mut cx, Key::Fn(6), true));
        assert_eq!(cx.settings.v["map_zoom"], json!(1.5));
        assert_eq!(saved(&cx, "map_zoom"), json!(1.5), "the file");
        let base = WalkStep::of(&cx.settings.v).base;
        assert_eq!(cx.log.last().cloned(), Some(format!("Map zoom 1.5x, walk step {base:.0} ms (F6).")));
        cx.settings.v["anim_speed"] = json!(2.0);
        press_zoom(&mut cx, Key::Fn(6), true);
        assert_eq!(cx.log.last().cloned(), Some(format!("Map zoom 2x, walk step {base:.0} ms ({:.0} ms at this Animation speed) (F6).", base / 2.0)));
        let reload = Settings::load(file_of(&cx), crate::screens::default_settings());
        assert_eq!(reload.v["map_zoom"], json!(2.0), "a new run reads it");
    }

    /// In Windowed mode F9 saves the size and shows it where the screen says; other keys pass.
    #[test]
    fn f9_in_windowed_mode_saves_and_shows_the_size() {
        let mut cx = game_cx();
        assert_eq!(cx.settings.v["window_mode"], json!("Windowed"), "the default");
        assert!(!press_window(&mut cx, Key::Fn(6), Shown::Log, true), "not its key");
        assert!(press_window(&mut cx, Key::Fn(9), Shown::Log, true));
        assert_eq!(cx.settings.v["window_size"], json!("1600x900"));
        assert_eq!(saved(&cx, "window_size"), json!("1600x900"), "the file");
        assert_eq!(cx.log.last().map(String::as_str), Some("Window 1600x900 (F9)."));
        assert!(cx.notice.is_empty(), "the log shows it");
        press_window(&mut cx, Key::Fn(9), Shown::Notice, true);
        assert_eq!(cx.notice, vec!["Window 1920x1080 (F9).".to_string()]);
    }

    /// Every mode but Windowed leaves the size and its file alone and says why, naming the mode.
    #[test]
    fn f9_outside_windowed_mode_only_says_why() {
        let others: Vec<&str> = WINDOW_MODES.into_iter().filter(|m| *m != "Windowed").collect();
        assert_eq!(others, ["Borderless", "Fullscreen"], "the modes the shell does not size");
        for mode in others {
            let mut cx = game_cx();
            cx.settings.v["window_mode"] = json!(mode);
            let was = cx.settings.v["window_size"].clone();
            assert!(press_window(&mut cx, Key::Fn(9), Shown::Notice, true));
            assert_eq!(cx.settings.v["window_size"], was, "{mode}");
            assert!(!file_of(&cx).exists(), "{mode}: nothing to save");
            assert!(cx.notice[0].contains(mode) && cx.notice[0].contains("Windowed"), "{:?}", cx.notice);
        }
    }

    /// F11 walks its steps in order and comes back to the first; each is a value the Walk step slider offers.
    #[test]
    fn f11_cycles_in_order() {
        let (least, most, default) = WALK_STEP_MS;
        assert!(WALK_CYCLE.contains(&default), "the default is a step");
        for (i, step) in WALK_CYCLE.into_iter().enumerate() {
            assert!((least..=most).contains(&step) && (step - least) % 10.0 == 0.0, "{step}");
            assert_eq!(next_walk(&json!(step)), WALK_CYCLE[(i + 1) % WALK_CYCLE.len()], "from {step}");
        }
        assert_eq!(WALK_CYCLE, [90.0, 110.0, 130.0, 160.0, 200.0]);
    }

    /// An off-cycle value goes to the next step above; past the last, or from a bad value, to the first.
    #[test]
    fn f11_from_an_off_cycle_or_bad_value() {
        for (now, want) in [(60.0, 90.0), (100.0, 110.0), (120.0, 130.0), (140.0, 160.0), (199.0, 200.0), (210.0, 90.0), (240.0, 90.0)] {
            assert_eq!(next_walk(&json!(now)), want, "from {now}");
        }
        for bad in [json!("slow"), Value::Null, json!(true), json!(59), json!(241), json!(-5), json!(1e300)] {
            assert_eq!(next_walk(&bad), WALK_CYCLE[0], "{bad}");
        }
    }

    /// F11 saves the step and the note gives it, plus the effective time off 1x; other keys pass; with the debug gate
    /// shut it is taken and changes nothing.
    #[test]
    fn f11_saves_and_notes_the_value_in_a_debug_build_only() {
        let mut cx = game_cx();
        let before = cx.settings.v.clone();
        assert!(press_walk(&mut cx, Key::Fn(11), false));
        assert_eq!(cx.settings.v, before);
        assert!(!file_of(&cx).exists() && cx.log.is_empty(), "the gate shut: nothing");
        assert!(!press_walk(&mut cx, Key::Fn(6), true), "not its key");
        assert!(press_walk(&mut cx, Key::Fn(11), true));
        assert_eq!(cx.settings.v[WALK_STEP_KEY], json!(160.0), "from the default");
        assert_eq!(saved(&cx, WALK_STEP_KEY), json!(160.0), "the file");
        if cfg!(debug_assertions) {
            assert_eq!(cx.log.last().map(String::as_str), Some("Walk step 160 ms (F11)."));
            cx.settings.v["anim_speed"] = json!(2.0);
            press_walk(&mut cx, Key::Fn(11), true);
            assert_eq!(cx.log.last().map(String::as_str), Some("Walk step 200 ms (100 ms at this Animation speed) (F11)."));
        }
        let was = cx.settings.v[WALK_STEP_KEY].clone();
        assert!(walk_key(&mut cx, Key::Fn(11)), "the public key takes it");
        assert_eq!(cx.settings.v[WALK_STEP_KEY] != was, cfg!(debug_assertions), "and follows the build");
    }

    /// With the debug gate shut the keys are taken and change, save and show nothing.
    #[test]
    fn a_release_build_ignores_the_keys() {
        let mut cx = game_cx();
        let before = cx.settings.v.clone();
        assert!(press_zoom(&mut cx, Key::Fn(6), false));
        assert!(press_window(&mut cx, Key::Fn(9), Shown::Log, false));
        assert!(press_window(&mut cx, Key::Fn(9), Shown::Notice, false));
        assert_eq!(cx.settings.v, before);
        assert!(!file_of(&cx).exists() && cx.log.is_empty() && cx.notice.is_empty());
    }

    /// The public keys follow the build's gate (a debug test run acts, a release test run does not).
    #[test]
    fn the_public_keys_follow_the_build() {
        let mut cx = game_cx();
        assert!(zoom_key(&mut cx, Key::Fn(6)) && window_key(&mut cx, Key::Fn(9), Shown::Log));
        let (zoom, size) = if cfg!(debug_assertions) { (json!(1.5), json!("1600x900")) } else { (json!(1.0), json!("1280x720")) };
        assert_eq!(cx.settings.v["map_zoom"], zoom);
        assert_eq!(cx.settings.v["window_size"], size);
    }

    /// VIEWKEYS1: the Settings debug row's Walk step: only a debug build draws it, in range at every step, beside the
    /// foresight box and not on it; the pick is a draft until Apply, which saves it and the map walks by it.
    #[cfg(debug_assertions)]
    #[test]
    fn the_walk_step_slider_tunes_the_step_on_apply() {
        use crate::hud::hud_tests::gfx;
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut s = crate::menus::SettingsScreen::new(&cx);
        s.draw(&mut cx, &mut g, (-500, -500), true);
        s.click_id(&mut cx, "settings.tab.battle").expect("the tab");
        s.draw(&mut cx, &mut g, (-500, -500), true);
        let widgets = s.widgets();
        let of = |id: &str| widgets.iter().find(|w| w["id"] == id).unwrap_or_else(|| panic!("{id}"));
        let n = |w: &Value, i: usize| w["rect"][i].as_i64().expect("a rect");
        let (row, boxed) = (of("settings.debug.walk_step"), of("settings.debug.party_foresight_off"));
        assert_eq!(row["text"], json!(format!("{:.0} ms", WALK_STEP_MS.2)), "the default");
        assert!(n(row, 0) + n(row, 2) <= n(boxed, 0), "beside the box: {row} {boxed}");
        let steps: Vec<&Value> = widgets.iter().filter(|w| w["id"].as_str().is_some_and(|i| i.starts_with("settings.debug.walk_step.step."))).collect();
        assert_eq!(steps.len(), ((WALK_STEP_MS.1 - WALK_STEP_MS.0) / 10.0) as usize + 1);
        let at = ((200.0 - WALK_STEP_MS.0) / 10.0) as usize;
        s.click_id(&mut cx, &format!("settings.debug.walk_step.step.{at}")).expect("the step");
        assert_eq!(cx.settings.v[WALK_STEP_KEY], json!(WALK_STEP_MS.2), "only a draft so far");
        s.click_id(&mut cx, "settings.apply").expect("apply");
        assert_eq!(WalkStep::of(&cx.settings.v).base, 200.0, "not the volumes' 0-100 clamp");
        assert_eq!(saved(&cx, WALK_STEP_KEY).as_f64(), Some(200.0), "the file");
        // the slider's ends are the range's
        for (i, want) in [(0, WALK_STEP_MS.0), (steps.len() - 1, WALK_STEP_MS.1)] {
            let mut s = crate::menus::SettingsScreen::new(&cx);
            s.draw(&mut cx, &mut g, (-500, -500), true);
            s.click_id(&mut cx, "settings.tab.battle").expect("the tab");
            s.draw(&mut cx, &mut g, (-500, -500), true);
            s.click_id(&mut cx, &format!("settings.debug.walk_step.step.{i}")).expect("the end");
            s.click_id(&mut cx, "settings.apply").expect("apply");
            assert_eq!(WalkStep::of(&cx.settings.v).base, want);
        }
    }

    /// The screens that call `window_key` act on F9 and show it in a notice; the agent link knows the names.
    #[test]
    fn f9_reaches_the_wired_screens_and_the_link_has_the_names() {
        let mut screens: Vec<Box<dyn Screen<crate::screens::GameCx>>> = Vec::new();
        screens.push(Box::new(crate::worldmap::WorldMap::new()));
        screens.push(Box::new(crate::hub::Hub::default()));
        screens.push(Box::new(crate::menus::MainMenu::default()));
        let mut cx = game_cx();
        screens.push(Box::new(crate::battle::BattleScreen::sandbox(&cx)));
        for mut s in screens {
            let id = s.id();
            cx.settings.v["window_size"] = json!("1280x720");
            cx.notice.clear();
            s.key(&mut cx, Key::Fn(9));
            let want = if cfg!(debug_assertions) { "1600x900" } else { "1280x720" };
            assert_eq!(cx.settings.v["window_size"], json!(want), "{id}");
            assert_eq!(cx.notice.len(), cfg!(debug_assertions) as usize, "{id}");
            assert!(cx.nav.is_empty(), "{id}: no screen change");
        }
        for (name, k) in [("f6", Key::Fn(6)), ("f9", Key::Fn(9)), ("f11", Key::Fn(11))] {
            assert_eq!(Key::parse(name), Some(k));
            assert_eq!(k.name(), name);
        }
    }
}
