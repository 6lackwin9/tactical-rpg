//! VIEWKEYS1: debug hotkeys to judge the view without opening Settings (user 2026-11-03). F6 cycles the map zoom on the
//! region map, F9 cycles the window size on the screens that call `window_key` (the region and world maps, the battle,
//! the game menu, the main menu). Each press saves like a normal setting and shows the new value; the shell applies a
//! window size when it sees the setting change. A release build ignores both keys. The Settings screen does not call
//! them: its draft would overwrite the key's value on Apply. VIEWKEYS2: F11 cycles the walk step on the region map.
//! TURNFEEL1: F10 cycles the walk's beat and equal speed on the region map, F5 its turn corner.

use crate::mapview::{zoom_of, TurnCorner, View, WalkFeel, WalkStep, EQUAL_SPEED_KEY, QUICK_FINISH_MS, TURN_BEAT_KEY, TURN_BEAT_MS, TURN_CORNERS, TURN_CORNER_KEY, WALK_STEP_KEY};
use crate::screens::{show_errors, Cx, Key};
use aegis_ui::screen::WindowShow;
use serde_json::{json, Value};

/// F6's steps, in the quarters of `mapview::ZOOM_QUARTERS`: 1x, 1.5x, 2x.
pub const ZOOM_CYCLE: [i32; 3] = [4, 6, 8];
/// F9's steps, as `window_size` values.
pub const SIZE_CYCLE: [&str; 3] = ["1280x720", "1600x900", "1920x1080"];

/// F11's steps, as `walk_step_ms` values.
pub const WALK_CYCLE: [f64; 5] = [90.0, 110.0, 130.0, 160.0, 200.0];

/// F10's steps: off, the beat, equal speed, both.
pub const FEEL_CYCLE: [WalkFeel; 4] = [
    WalkFeel { beat: false, equal: false, corner: TurnCorner::Cut },
    WalkFeel { beat: true, equal: false, corner: TurnCorner::Cut },
    WalkFeel { beat: false, equal: true, corner: TurnCorner::Cut },
    WalkFeel { beat: true, equal: true, corner: TurnCorner::Cut },
];

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

/// The options F10 goes to from `now`: the next beat and equal speed, the corner kept.
pub fn next_feel(now: WalkFeel) -> WalkFeel {
    let at = FEEL_CYCLE.iter().position(|f| (f.beat, f.equal) == (now.beat, now.equal)).unwrap_or(0);
    WalkFeel { corner: now.corner, ..FEEL_CYCLE[(at + 1) % FEEL_CYCLE.len()] }
}

/// The corner F5 goes to from `now`.
pub fn next_corner(now: TurnCorner) -> TurnCorner {
    let at = TURN_CORNERS.iter().position(|c| *c == now).unwrap_or(0);
    TURN_CORNERS[(at + 1) % TURN_CORNERS.len()]
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

/// Why F9 may not size the window under these settings (Fullscreen covers the monitor, as the greyed-out Resolution
/// row shows; a Borderless window sizes like a framed one); None when it may.
pub fn size_refusal(settings: &Value) -> Option<String> {
    (WindowShow::of(settings) == WindowShow::Fullscreen).then(|| "F9: Resolution does not apply in Fullscreen.".to_string())
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
    press_zoom(cx, k, crate::DEV_KEYS)
}

/// F9 on a screen: whether `k` was the key (a release build takes it and does nothing).
pub fn window_key(cx: &mut Cx, k: Key, shown: Shown) -> bool {
    press_window(cx, k, shown, crate::DEV_KEYS)
}

/// F11 on the region map: whether `k` was the key (a release build takes it and does nothing).
pub fn walk_key(cx: &mut Cx, k: Key) -> bool {
    press_walk(cx, k, crate::DEV_KEYS)
}

/// F10 on the region map: whether `k` was the key (a release build takes it and does nothing).
pub fn feel_key(cx: &mut Cx, k: Key) -> bool {
    press_feel(cx, k, crate::DEV_KEYS)
}

/// F5 on the region map: whether `k` was the key (a release build takes it and does nothing).
pub fn corner_key(cx: &mut Cx, k: Key) -> bool {
    press_corner(cx, k, crate::DEV_KEYS)
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
        let line = format!("Map zoom {to}x, walk step {} (F6).", walk_words(WalkStep::of(&cx.settings.v, false)));
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
        let line = format!("Walk step {} (F11).", walk_words(WalkStep::of(&cx.settings.v, false)));
        cx.log(line);
    }
    true
}

fn feel_words(feel: WalkFeel, view: View, key: &str) -> String {
    let beat = if feel.beat { format!("{TURN_BEAT_MS:.0} ms") } else { "off".to_string() };
    let equal = match (feel.equal, view) {
        (false, _) => "off",
        (true, View::Tilted) => "on",
        (true, View::Square) => "on (changes nothing on square cells)",
    };
    let corner = match feel.corner {
        TurnCorner::Cut => "cut".to_string(),
        TurnCorner::Finish => "finish the leg".to_string(),
        TurnCorner::Quick => format!("quick finish {QUICK_FINISH_MS:.0} ms"),
    };
    format!("Turn beat {beat}, equal speed {equal}, corner {corner} ({key}).")
}

fn press_corner(cx: &mut Cx, k: Key, debug: bool) -> bool {
    if k != Key::Fn(5) {
        return false;
    }
    if debug {
        let now = WalkFeel::read(&cx.settings.v, true);
        let to = WalkFeel { corner: next_corner(now.corner), ..now };
        cx.settings.v[TURN_CORNER_KEY] = json!(to.corner.name());
        save(cx);
        let line = feel_words(to, View::of_setting(&cx.settings.v["region_view"]), "F5");
        cx.log(line);
    }
    true
}

fn press_feel(cx: &mut Cx, k: Key, debug: bool) -> bool {
    if k != Key::Fn(10) {
        return false;
    }
    if debug {
        let to = next_feel(WalkFeel::read(&cx.settings.v, true));
        cx.settings.v[TURN_BEAT_KEY] = json!(to.beat);
        cx.settings.v[EQUAL_SPEED_KEY] = json!(to.equal);
        save(cx);
        let line = feel_words(to, View::of_setting(&cx.settings.v["region_view"]), "F10");
        cx.log(line);
    }
    true
}

fn press_window(cx: &mut Cx, k: Key, shown: Shown, debug: bool) -> bool {
    if k != Key::Fn(9) {
        return false;
    }
    if debug {
        match size_refusal(&cx.settings.v) {
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
    use crate::screens::WINDOW_SIZES;
    use aegis_ui::screen::Settings;

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
        for s in aegis_ui::screen::WINDOW_SIZES.into_iter().filter(|s| !SIZE_CYCLE.contains(s)) {
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
        let base = WalkStep::of(&cx.settings.v, false).base;
        assert_eq!(cx.log.last().cloned(), Some(format!("Map zoom 1.5x, walk step {base:.0} ms (F6).")));
        cx.settings.v["anim_speed"] = json!(2.0);
        press_zoom(&mut cx, Key::Fn(6), true);
        assert_eq!(cx.log.last().cloned(), Some(format!("Map zoom 2x, walk step {base:.0} ms ({:.0} ms at this Animation speed) (F6).", base / 2.0)));
        let reload = Settings::load(file_of(&cx), crate::screens::default_settings());
        assert_eq!(reload.v["map_zoom"], json!(2.0), "a new run reads it");
    }

    /// A Windowed and a Borderless window both take F9: it saves the size and shows it where the screen says; other
    /// keys pass.
    #[test]
    fn f9_saves_and_shows_the_size_unless_fullscreen() {
        for borderless in [false, true] {
            let mut cx = game_cx();
            assert_eq!(WindowShow::of(&cx.settings.v), WindowShow::Windowed, "the default");
            cx.settings.v["borderless"] = json!(borderless);
            assert!(!press_window(&mut cx, Key::Fn(6), Shown::Log, true), "not its key");
            assert!(press_window(&mut cx, Key::Fn(9), Shown::Log, true));
            assert_eq!(cx.settings.v["window_size"], json!("1600x900"));
            assert_eq!(saved(&cx, "window_size"), json!("1600x900"), "the file");
            assert_eq!(cx.log.last().map(String::as_str), Some("Window 1600x900 (F9)."));
            assert!(cx.notice.is_empty(), "the log shows it");
            press_window(&mut cx, Key::Fn(9), Shown::Notice, true);
            assert_eq!(cx.notice, vec!["Window 1920x1080 (F9).".to_string()]);
        }
    }

    /// Fullscreen, with or without Borderless, leaves the size and its file alone and says why.
    #[test]
    fn f9_in_fullscreen_only_says_why() {
        for borderless in [false, true] {
            let mut cx = game_cx();
            cx.settings.v["fullscreen"] = json!(true);
            cx.settings.v["borderless"] = json!(borderless);
            let was = cx.settings.v["window_size"].clone();
            assert!(press_window(&mut cx, Key::Fn(9), Shown::Notice, true));
            assert_eq!(cx.settings.v["window_size"], was);
            assert!(!file_of(&cx).exists(), "nothing to save");
            assert!(cx.notice[0].contains("Fullscreen") && cx.notice[0].contains("F9"), "{:?}", cx.notice);
        }
    }

    /// The game's list starts at the decision's 720p: the engine's default list without what is under it, the first
    /// is the default size, and a saved smaller size opens at the first.
    #[test]
    fn the_game_offers_720p_and_up() {
        let area = |s: &str| pixels(s).expect("a size");
        let engine: Vec<&str> = aegis_ui::screen::WINDOW_SIZES.into_iter().filter(|s| area(s) >= area("1280x720")).collect();
        assert_eq!(WINDOW_SIZES.to_vec(), engine);
        let mut cx = game_cx();
        assert_eq!(cx.settings.v["window_size"], json!(WINDOW_SIZES[0]), "the default");
        for small in aegis_ui::screen::WINDOW_SIZES.into_iter().filter(|s| area(s) < area("1280x720")) {
            cx.settings.v["window_size"] = json!(small);
            cx.settings.keep_window_size(&WINDOW_SIZES);
            assert_eq!(cx.settings.v["window_size"], json!(WINDOW_SIZES[0]), "{small}");
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
        cx.settings.v[WALK_STEP_KEY] = json!(130.0);
        assert!(press_walk(&mut cx, Key::Fn(11), true));
        assert_eq!(cx.settings.v[WALK_STEP_KEY], json!(160.0), "the next step");
        assert_eq!(saved(&cx, WALK_STEP_KEY), json!(160.0), "the file");
        if cfg!(debug_assertions) {
            assert_eq!(cx.log.last().map(String::as_str), Some("Walk step 160 ms (F11)."));
            cx.settings.v["anim_speed"] = json!(2.0);
            press_walk(&mut cx, Key::Fn(11), true);
            assert_eq!(cx.log.last().map(String::as_str), Some("Walk step 200 ms (100 ms at this Animation speed) (F11)."));
        }
        let was = cx.settings.v[WALK_STEP_KEY].clone();
        assert!(walk_key(&mut cx, Key::Fn(11)), "the public key takes it");
        assert_eq!(cx.settings.v[WALK_STEP_KEY] != was, crate::DEV_KEYS, "and follows the build");
    }

    /// TURNFEEL1: F10 walks the four combinations in order and back; a value that is no bool reads as off; each press saves
    /// and notes what is on, and says when the view makes equal speed do nothing; other keys pass; with the debug gate
    /// shut it is taken and changes nothing.
    #[test]
    fn f10_cycles_the_four_feels_and_saves_them() {
        let mut cx = game_cx();
        let before = cx.settings.v.clone();
        assert!(press_feel(&mut cx, Key::Fn(10), false));
        assert_eq!(cx.settings.v, before);
        assert!(!file_of(&cx).exists() && cx.log.is_empty(), "the gate shut: nothing");
        assert!(!press_feel(&mut cx, Key::Fn(11), true), "not its key");
        assert_eq!(WalkFeel::read(&cx.settings.v, true), FEEL_CYCLE[0], "both off by default");
        for i in 1..=FEEL_CYCLE.len() {
            let want = FEEL_CYCLE[i % FEEL_CYCLE.len()];
            assert!(press_feel(&mut cx, Key::Fn(10), true));
            assert_eq!(WalkFeel::read(&cx.settings.v, true), want);
            assert_eq!((saved(&cx, TURN_BEAT_KEY), saved(&cx, EQUAL_SPEED_KEY)), (json!(want.beat), json!(want.equal)), "the file");
            assert_eq!(cx.log.last().cloned(), Some(feel_words(want, View::Square, "F10")));
        }
        let distinct: std::collections::HashSet<(bool, bool)> = FEEL_CYCLE.iter().map(|f| (f.beat, f.equal)).collect();
        assert_eq!(distinct.len(), 4, "every combination");
        cx.settings.v[TURN_BEAT_KEY] = json!("yes");
        cx.settings.v[EQUAL_SPEED_KEY] = json!(1);
        assert_eq!(WalkFeel::read(&cx.settings.v, true), FEEL_CYCLE[0], "no bool is off");
        assert_eq!(feel_words(FEEL_CYCLE[3], View::Tilted, "F10"), "Turn beat 60 ms, equal speed on, corner cut (F10).");
        assert_eq!(feel_words(FEEL_CYCLE[2], View::Square, "F10"), "Turn beat off, equal speed on (changes nothing on square cells), corner cut (F10).");
        assert_eq!(feel_words(FEEL_CYCLE[0], View::Tilted, "F10"), "Turn beat off, equal speed off, corner cut (F10).");
    }

    /// F5 walks the three corners in order and back, leaves the beat and equal speed alone, and F10 leaves the corner
    /// alone; each saves and notes it; with the debug gate shut it is taken and changes nothing.
    #[test]
    fn f5_cycles_the_corner_and_saves_it() {
        let mut cx = game_cx();
        let before = cx.settings.v.clone();
        assert!(press_corner(&mut cx, Key::Fn(5), false));
        assert_eq!(cx.settings.v, before);
        assert!(!file_of(&cx).exists() && cx.log.is_empty(), "the gate shut: nothing");
        assert!(!press_corner(&mut cx, Key::Fn(10), true), "not its key");
        press_feel(&mut cx, Key::Fn(10), true);
        let was = WalkFeel::read(&cx.settings.v, true);
        for want in [TurnCorner::Finish, TurnCorner::Quick, TurnCorner::Cut] {
            assert!(press_corner(&mut cx, Key::Fn(5), true));
            let now = WalkFeel::read(&cx.settings.v, true);
            assert_eq!(now, WalkFeel { corner: want, ..was }, "only the corner moves");
            assert_eq!(saved(&cx, TURN_CORNER_KEY), json!(want.name()), "the file");
            assert_eq!(cx.log.last().cloned(), Some(feel_words(now, View::Square, "F5")));
        }
        assert_eq!(WalkFeel::read(&crate::screens::default_settings(), true).corner, TurnCorner::Cut, "the default");
        cx.settings.v[TURN_CORNER_KEY] = json!("quick");
        press_feel(&mut cx, Key::Fn(10), true);
        assert_eq!(WalkFeel::read(&cx.settings.v, true).corner, TurnCorner::Quick, "F10 keeps the corner");
        assert_eq!(feel_words(WalkFeel { beat: false, equal: false, corner: TurnCorner::Quick }, View::Tilted, "F5"), "Turn beat off, equal speed off, corner quick finish 60 ms (F5).");
        assert_eq!(feel_words(WalkFeel { beat: false, equal: false, corner: TurnCorner::Finish }, View::Tilted, "F5"), "Turn beat off, equal speed off, corner finish the leg (F5).");
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
        let (zoom, size) = if crate::DEV_KEYS { (json!(1.5), json!("1600x900")) } else { (json!(1.0), json!("1280x720")) };
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
        let at = ((180.0 - WALK_STEP_MS.0) / 10.0) as usize;
        s.click_id(&mut cx, &format!("settings.debug.walk_step.step.{at}")).expect("the step");
        assert_eq!(cx.settings.v[WALK_STEP_KEY], json!(WALK_STEP_MS.2), "only a draft so far");
        s.click_id(&mut cx, "settings.apply").expect("apply");
        assert_eq!(WalkStep::of(&cx.settings.v, false).base, 180.0, "not the volumes' 0-100 clamp");
        assert_eq!(saved(&cx, WALK_STEP_KEY).as_f64(), Some(180.0), "the file");
        // the slider's ends are the range's
        for (i, want) in [(0, WALK_STEP_MS.0), (steps.len() - 1, WALK_STEP_MS.1)] {
            let mut s = crate::menus::SettingsScreen::new(&cx);
            s.draw(&mut cx, &mut g, (-500, -500), true);
            s.click_id(&mut cx, "settings.tab.battle").expect("the tab");
            s.draw(&mut cx, &mut g, (-500, -500), true);
            s.click_id(&mut cx, &format!("settings.debug.walk_step.step.{i}")).expect("the end");
            s.click_id(&mut cx, "settings.apply").expect("apply");
            assert_eq!(WalkStep::of(&cx.settings.v, false).base, want);
        }
    }

    /// DISPLAY2: the Display tab has the two toggles and its Resolution row greys out only in Fullscreen; a toggle is a
    /// draft until Apply, which saves it.
    #[test]
    fn the_display_tab_greys_resolution_only_in_fullscreen() {
        use crate::hud::hud_tests::gfx;
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut s = crate::menus::SettingsScreen::new(&cx);
        s.draw(&mut cx, &mut g, (-500, -500), true);
        s.click_id(&mut cx, "settings.tab.display").expect("the tab");
        let mut enabled = |s: &mut crate::menus::SettingsScreen, cx: &mut Cx| {
            s.draw(cx, &mut g, (-500, -500), true);
            s.widgets().iter().find(|w| w["id"] == "settings.window_size").expect("the row")["enabled"].clone()
        };
        assert_eq!(enabled(&mut s, &mut cx), json!(true), "Windowed");
        s.click_id(&mut cx, "settings.borderless").expect("the toggle");
        assert_eq!(enabled(&mut s, &mut cx), json!(true), "Borderless");
        s.click_id(&mut cx, "settings.fullscreen").expect("the toggle");
        assert_eq!(enabled(&mut s, &mut cx), json!(false), "Fullscreen");
        assert_eq!(WindowShow::of(&cx.settings.v), WindowShow::Windowed, "only a draft so far");
        s.click_id(&mut cx, "settings.apply").expect("apply");
        assert_eq!(WindowShow::of(&cx.settings.v), WindowShow::Fullscreen);
        assert_eq!((saved(&cx, "fullscreen"), saved(&cx, "borderless")), (json!(true), json!(true)), "the file");
        s.click_id(&mut cx, "settings.fullscreen").expect("the toggle");
        assert_eq!(enabled(&mut s, &mut cx), json!(true), "Borderless again");
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
            let want = if crate::DEV_KEYS { "1600x900" } else { "1280x720" };
            assert_eq!(cx.settings.v["window_size"], json!(want), "{id}");
            assert_eq!(cx.notice.len(), crate::DEV_KEYS as usize, "{id}");
            assert!(cx.nav.is_empty(), "{id}: no screen change");
        }
        for (name, k) in [("f5", Key::Fn(5)), ("f6", Key::Fn(6)), ("f9", Key::Fn(9)), ("f10", Key::Fn(10)), ("f11", Key::Fn(11))] {
            assert_eq!(Key::parse(name), Some(k));
            assert_eq!(k.name(), name);
        }
    }
}
