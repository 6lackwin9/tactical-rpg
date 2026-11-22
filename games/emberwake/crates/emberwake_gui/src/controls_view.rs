//! CONTROLS-UI1 (PR7-44): the Settings Controls tab, born split (`docs/specs/UISPLIT1.md` US1, US4, US8). Its view
//! `settings_controls` lists each action the game lets the player rebind with its key and pad button, the swap question
//! and the inputs that stay fixed; its actions start a rebind, answer the question and reset the map. The rebinding
//! itself (raw input, clash, swap, the settings key `controls`) is the engine's (`aegis_ui::controls::Rebind`); this file
//! names the game's actions and words. Drawn through the default layout until Art/UI writes
//! `ui/layouts/settings_controls.json`; its fixture is `ui/views/settings_controls.view.json` (`tests::the_fixture_has_the_view_shape`).

use aegis_ui::controls::{key_label, Action, Controls, Fixed, Input, Outcome, Rebind, Step};
use aegis_ui::screen::Key;
use aegis_ui::screen_layout::{Act, Field, RowView, View};
use serde_json::{json, Value};

/// The view's screen name (its layout and fixture files).
pub const SCREEN: &str = "settings_controls";
/// The view's actions: a row's rebind (its arg the action id), the question's Swap and Cancel, Reset to defaults.
pub const REBIND: &str = "bindings";
pub const SWAP: &str = "swap";
pub const CANCEL: &str = "cancel";
pub const RESET: &str = "reset";

/// The actions the player can rebind (`[PROVISIONAL]`, FB-327): each a key the game's screens read, labelled with its
/// uses (the map is one for every screen, so a key used on two screens is one action).
pub const ACTIONS: [Action; 26] = [
    Action { id: "confirm", label: "Confirm / confirm the round", key: Key::Enter },
    Action { id: "press", label: "Press / step, next round", key: Key::Space },
    Action { id: "back", label: "Back / game menu", key: Key::Esc },
    Action { id: "focus_next", label: "Next control", key: Key::Tab },
    Action { id: "focus_prev", label: "Previous control", key: Key::BackTab },
    Action { id: "up", label: "Up / move up", key: Key::Up },
    Action { id: "down", label: "Down / move down", key: Key::Down },
    Action { id: "left", label: "Left / move left", key: Key::Left },
    Action { id: "right", label: "Right / move right", key: Key::Right },
    Action { id: "page_up", label: "Page back / log up", key: Key::PageUp },
    Action { id: "page_down", label: "Page on / log down", key: Key::PageDown },
    Action { id: "walk_up", label: "Walk up", key: Key::Char('w') },
    Action { id: "walk_left", label: "Walk left / autoplay (battle)", key: Key::Char('a') },
    Action { id: "walk_down", label: "Walk down / target self (battle)", key: Key::Char('s') },
    Action { id: "walk_right", label: "Walk right", key: Key::Char('d') },
    Action { id: "interact", label: "Interact with the marker you face", key: Key::Char('e') },
    Action { id: "lead", label: "Next leader on the map", key: Key::Char('q') },
    Action { id: "party", label: "Party / playback speed (battle)", key: Key::Char('p') },
    Action { id: "journal", label: "Journal", key: Key::Char('j') },
    Action { id: "stash", label: "Stash", key: Key::Char('i') },
    Action { id: "log", label: "Event log / combat log", key: Key::Char('l') },
    Action { id: "help", label: "Help", key: Key::Char('?') },
    Action { id: "undo", label: "Undo the hero's last step", key: Key::Char('z') },
    Action { id: "timeline", label: "Full timeline view", key: Key::Char('t') },
    Action { id: "next_block", label: "Next block (timeline)", key: Key::Char('n') },
    Action { id: "auto_fill", label: "Accept the auto-fill (timeline)", key: Key::Char('f') },
];

/// The inputs that stay as they are (mouse, held keys, chords, number rows and the keys of `FIXED_KEYS`).
pub const FIXED: [(&str, &str); 10] = [
    ("Run", "Hold Shift"),
    ("Back / inspect a unit", "Right click"),
    ("Choose a dialogue line", "1-9"),
    ("Select party member", "1-4"),
    ("Yes to a question", "Y"),
    ("Restart the battle", "R"),
    ("Skills of the selected hero", "Shift+1-9"),
    ("Support an ally (timeline)", "Ctrl+click"),
    ("Outline every unit, every mark", "Hold Alt"),
    ("Screenshot", "F12"),
];

/// CONTROLS-UI1B (PR7-49, FB-352 (4)): the keys screens read that `ACTIONS` does not list; a rebind onto one is
/// refused with a line naming the screen that reads it (`[PROVISIONAL]`).
pub const FIXED_KEYS: [Fixed; 11] = [
    Fixed { key: Key::Char('y'), label: "Yes to a question", screen: "every question box" },
    Fixed { key: Key::Char('r'), label: "Restart the battle", screen: "the battle screen" },
    Fixed { key: Key::Char('1'), label: "Choose a dialogue line / select party member", screen: "conversations and the battle screen" },
    Fixed { key: Key::Char('2'), label: "Choose a dialogue line / select party member", screen: "conversations and the battle screen" },
    Fixed { key: Key::Char('3'), label: "Choose a dialogue line / select party member", screen: "conversations and the battle screen" },
    Fixed { key: Key::Char('4'), label: "Choose a dialogue line / select party member", screen: "conversations and the battle screen" },
    Fixed { key: Key::Char('5'), label: "Choose a dialogue line", screen: "conversations" },
    Fixed { key: Key::Char('6'), label: "Choose a dialogue line", screen: "conversations" },
    Fixed { key: Key::Char('7'), label: "Choose a dialogue line", screen: "conversations" },
    Fixed { key: Key::Char('8'), label: "Choose a dialogue line", screen: "conversations" },
    Fixed { key: Key::Char('9'), label: "Choose a dialogue line", screen: "conversations" },
];

/// The view's words for an input: a key's name or a pad button's.
fn input_label(i: Input) -> String {
    match i {
        Input::Key(k) => key_label(k),
        Input::Pad(p) => p.label().to_string(),
    }
}

/// An action's key and pad button ("E · Pad A"), for tests; its row shows them in two columns (`key_text`, `pad_text`).
#[cfg(test)]
pub fn binding_text(c: &Controls, a: &Action) -> String {
    format!("{} · {}", key_text(c, a), pad_text(c, a))
}

/// CONTROLS-UI1B (FB-351): an action's key, its row's `detail` column.
pub fn key_text(c: &Controls, a: &Action) -> String {
    c.key_hint(a.key)
}

/// CONTROLS-UI1B (FB-351): an action's pad button, its row's `detail2` column; "no pad button" for a buttonless action
/// (Journal, Undo; FB-352 (3): none for now).
pub fn pad_text(c: &Controls, a: &Action) -> String {
    c.pad_for(a.key).map_or("no pad button", |p| p.label()).to_string()
}

/// CONTROLS-UI1B (PR7-49 (1)): the control map in force for `settings` (the whole object), for a screen's key hints
/// (`Controls::hint`, `Controls::key_hint`): every hint reads the bound key, never a fixed letter.
pub fn hints(settings: &Value) -> Controls {
    Controls::from_settings(settings).0
}

/// The view of the tab for the settings `settings` (the whole object) and the rebind in progress.
pub fn view(settings: &Value, rebind: &Rebind) -> View {
    let c = Controls::from_settings(settings).0;
    let label = |i: usize| ACTIONS.get(i).map_or("", |a| a.label);
    let prompt = match rebind.step {
        Step::Idle => "Click an action, then press its new key or pad button. Esc cancels.".to_string(),
        Step::Waiting(at) => match (rebind.refused, rebind.refused_fixed.and_then(|f| FIXED_KEYS.get(f))) {
            (_, Some(f)) => format!("{} stays fixed: {} uses it ({}). Press another key, or Esc to cancel.", key_label(f.key), f.screen, f.label),
            (Some(with), None) => format!("{} has no pad button to trade; rebind {} first. Press a key, or Esc to cancel.", label(at), label(with)),
            (None, None) => format!("Press the new key or pad button for {}. Its own button, Esc or {} s with no input cancels.", label(at), aegis_ui::controls::WAIT.as_secs()),
        },
        Step::Clash { at, input, with } => format!("{} is already {}. Swap it to {}? {} swaps, Esc cancels.", input_label(input), label(with), label(at), c.hint(Key::Enter)),
    };
    let rows = ACTIONS
        .iter()
        .enumerate()
        .map(|(i, a)| {
            // FB-351: the key in `detail`, the pad button in `detail2` (two columns where the layout gives them)
            let (detail, detail2) = match rebind.step {
                Step::Waiting(at) if at == i => ("Press a key or button…".to_string(), String::new()),
                // the question on the row too, for a row far down the scrolled list
                Step::Clash { at, input, .. } if at == i => (format!("{} taken", input_label(input)), format!("{} swaps", c.key_hint(Key::Enter))),
                _ => (key_text(&c, a), pad_text(&c, a)),
            };
            RowView { id: a.id.to_string(), label: a.label.to_string(), detail, detail2, sub: String::new(), picture: None, enabled: true, ..Default::default() }
        })
        .collect();
    let asking = matches!(rebind.step, Step::Clash { .. });
    View::new(SCREEN)
        .field("prompt", Field::Text(prompt))
        .field(REBIND, Field::Rows(rows))
        .field("fixed", Field::Stats(FIXED.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()))
        .action(REBIND, "Rebind", true)
        .action(SWAP, "Swap", asking)
        .action(CANCEL, "Cancel", rebind.capturing())
        .action(RESET, "Reset to defaults", settings.get(aegis_ui::controls::SETTINGS_KEY).is_some())
}

/// Runs a view action. True when the settings changed (the screen saves them).
pub fn act(rebind: &mut Rebind, settings: &mut Value, a: &Act) -> bool {
    match a.action.as_str() {
        REBIND => {
            // a row's click; the footer's bar (no row) does nothing
            if let Some(i) = a.arg.as_deref().and_then(|id| ACTIONS.iter().position(|x| x.id == id)) {
                rebind.start(i);
            }
            false
        }
        SWAP => rebind.swap(&ACTIONS, settings) == Outcome::Bound,
        CANCEL => {
            rebind.cancel();
            false
        }
        RESET => {
            let had = settings.get(aegis_ui::controls::SETTINGS_KEY).is_some();
            rebind.reset(settings);
            had
        }
        _ => false,
    }
}

/// The rebind for the agent link: `step` (idle, waiting, clash), the action asked for and the one it clashes with.
pub fn state(rebind: &Rebind) -> Value {
    let id = |i: usize| ACTIONS.get(i).map_or("", |a| a.id);
    match rebind.step {
        Step::Idle => json!({"step": "idle"}),
        Step::Waiting(at) => match (rebind.refused, rebind.refused_fixed.and_then(|f| FIXED_KEYS.get(f))) {
            (_, Some(f)) => json!({"step": "waiting", "action": id(at), "fixed": f.key.name()}),
            (Some(with), None) => json!({"step": "waiting", "action": id(at), "refused": id(with)}),
            (None, None) => json!({"step": "waiting", "action": id(at)}),
        },
        Step::Clash { at, input, with } => json!({"step": "clash", "action": id(at), "with": id(with), "input": input_label(input)}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_ui::controls::{Pad, PADS};

    fn row<'a>(v: &'a View, id: &str) -> &'a RowView {
        match v.get(REBIND) {
            Some(Field::Rows(r)) => r.iter().find(|r| r.id == id).expect("the row"),
            _ => panic!("no rows"),
        }
    }

    fn click(rebind: &mut Rebind, settings: &mut Value, action: &str, arg: Option<&str>) -> bool {
        act(rebind, settings, &Act { action: action.into(), arg: arg.map(str::to_string) })
    }

    fn at(id: &str) -> usize {
        ACTIONS.iter().position(|a| a.id == id).unwrap()
    }

    /// Every listed action names its own key and id; each has a key by default, and the defaults' pad buttons each give
    /// a listed action (so every button shows on some row).
    #[test]
    fn the_actions_are_distinct_and_every_default_button_shows() {
        for (i, a) in ACTIONS.iter().enumerate() {
            assert!(ACTIONS[i + 1..].iter().all(|b| b.key != a.key && b.id != a.id), "{} is listed once", a.id);
        }
        let c = Controls::default();
        for a in &ACTIONS {
            assert_eq!(c.key_for(a.key), Some(a.key), "{}: its own key by default", a.id);
        }
        for (p, _) in PADS {
            assert!(ACTIONS.iter().any(|a| a.key == c.pad(p)), "{p:?} gives a listed action");
        }
    }

    /// The rows show each action's key and button; a click waits for the input (its row says so), a key binds it and the
    /// settings hold the map; a pad button binds the pad; Esc cancels with nothing changed.
    #[test]
    fn a_click_then_a_key_rebinds_and_esc_cancels() {
        let (mut r, mut s) = (Rebind::default(), json!({}));
        let c = Controls::default();
        let interact = &ACTIONS[at("interact")];
        assert_eq!((row(&view(&s, &r), "interact").detail.as_str(), row(&view(&s, &r), "interact").detail2.as_str()), ("E", "no pad button"));
        assert_eq!(binding_text(&c, interact), "E · no pad button");
        assert!(binding_text(&c, &ACTIONS[at("confirm")]).contains(Pad::South.label()));
        assert!(!click(&mut r, &mut s, REBIND, Some("interact")));
        assert_eq!(state(&r), json!({"step": "waiting", "action": "interact"}));
        assert_eq!(row(&view(&s, &r), "interact").detail, "Press a key or button…");
        assert_eq!(r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Key(Key::Char('k'))), Outcome::Bound);
        let c = Controls::from_settings(&s).0;
        assert_eq!((c.key(Key::Char('k')), c.key_for(Key::Char('e'))), (Key::Char('e'), Some(Key::Char('k'))));
        assert_eq!(state(&r), json!({"step": "idle"}));
        // Esc cancels; a key a rebind may not take is held and the rebind stays open
        let before = s.clone();
        click(&mut r, &mut s, REBIND, Some("interact"));
        assert_eq!(r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Key(Key::Fn(5))), Outcome::Held);
        assert!(r.capturing());
        assert_eq!(r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Key(Key::Esc)), Outcome::Cancelled);
        assert_eq!((s.clone(), r.capturing()), (before, false));
        // a pad button for Interact (no button to trade): Pad Y is Next control's, so it is refused and the rebind waits
        click(&mut r, &mut s, REBIND, Some("interact"));
        let before = s.clone();
        assert_eq!(r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Pad(Pad::North)), Outcome::Refused, "Pad Y gives Tab (Next control) by default");
        assert_eq!(state(&r), json!({"step": "waiting", "action": "interact", "refused": "focus_next"}));
        assert!(matches!(view(&s, &r).get("prompt"), Some(Field::Text(t)) if t.contains("no pad button to trade") && t.contains("Next control")));
        assert_eq!(s, before, "nothing written");
        assert_eq!(r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Key(Key::Esc)), Outcome::Cancelled);
    }

    /// An input another action holds asks first; Swap moves it and the other action takes the old one; Cancel leaves the
    /// map; Enter answers Swap too.
    #[test]
    fn a_clash_asks_and_swap_trades_the_two() {
        let (mut r, mut s) = (Rebind::default(), json!({}));
        click(&mut r, &mut s, REBIND, Some("interact"));
        assert_eq!(r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Key(Key::Char('z'))), Outcome::Asked);
        let v = view(&s, &r);
        assert!(v.act(SWAP).unwrap().enabled && v.act(CANCEL).unwrap().enabled);
        assert!(matches!(v.get("prompt"), Some(Field::Text(t)) if t.contains("Undo") && t.contains("Swap")));
        assert_eq!(s, json!({}), "nothing written while it asks");
        assert!(!click(&mut r, &mut s, CANCEL, None));
        assert_eq!(s, json!({}));
        click(&mut r, &mut s, REBIND, Some("interact"));
        r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Key(Key::Char('z')));
        assert!(click(&mut r, &mut s, SWAP, None));
        let c = Controls::from_settings(&s).0;
        assert_eq!((c.key_for(Key::Char('e')), c.key_for(Key::Char('z'))), (Some(Key::Char('z')), Some(Key::Char('e'))));
        // Enter answers the question as Swap: the pad's Pad B (Back) to Confirm, Confirm's Pad A to Back
        click(&mut r, &mut s, REBIND, Some("confirm"));
        assert_eq!(r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Pad(Pad::East)), Outcome::Asked);
        assert_eq!(r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Key(Key::Char('x'))), Outcome::Held, "a key the question does not take");
        assert_eq!(r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Key(Key::Enter)), Outcome::Bound);
        let c = Controls::from_settings(&s).0;
        assert_eq!((c.pad(Pad::East), c.pad(Pad::South)), (Key::Enter, Key::Esc));
    }

    /// Reset to defaults drops the map (on only while there is one); the rows show the defaults again.
    #[test]
    fn reset_puts_the_defaults_back() {
        let (mut r, mut s) = (Rebind::default(), json!({"music": 40}));
        assert!(!view(&s, &r).act(RESET).unwrap().enabled);
        click(&mut r, &mut s, REBIND, Some("journal"));
        r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Key(Key::Char('k')));
        assert!(view(&s, &r).act(RESET).unwrap().enabled);
        assert!(click(&mut r, &mut s, RESET, None));
        assert_eq!(s, json!({"music": 40}));
        assert_eq!(row(&view(&s, &r), "journal").detail, key_text(&Controls::default(), &ACTIONS[at("journal")]));
    }

    /// CONTROLS-UI1B (PR7-49 (2)): a waiting rebind ends with nothing written on the button the action already has
    /// (a pad-only player's way out: Pad B for Back, Pad A for Confirm) or after `WAIT` with no input; any input
    /// restarts the wait; every pad button stays bindable (a button is never only a way out).
    #[test]
    fn a_waiting_rebind_ends_on_its_own_button_or_after_the_wait() {
        use aegis_ui::controls::WAIT;
        let (mut r, mut s) = (Rebind::default(), json!({}));
        let c = Controls::default();
        for a in ACTIONS.iter().filter(|a| c.pad_for(a.key).is_some()) {
            click(&mut r, &mut s, REBIND, Some(a.id));
            let own = c.pad_for(a.key).unwrap();
            assert_eq!(r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Pad(own)), Outcome::Cancelled, "{}: its own {own:?}", a.id);
            assert_eq!((s.clone(), r.capturing()), (json!({}), false), "{}: nothing written", a.id);
            click(&mut r, &mut s, REBIND, Some(a.id));
            assert_eq!(r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Key(a.key)), Outcome::Cancelled, "{}: its own key", a.id);
        }
        // the wait: one frame before `until` it still waits, at `until` it ends; a refused input restarts it
        click(&mut r, &mut s, REBIND, Some("interact"));
        let until = r.until.expect("a waiting rebind has an end");
        assert!(!r.expire(until - aegis_core::clock::Duration::from_nanos(1)) && r.capturing());
        assert_eq!(r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Key(Key::Char('y'))), Outcome::Refused);
        let later = r.until.unwrap();
        assert!(later >= until, "an input restarts the wait");
        assert!(!r.expire(later - aegis_core::clock::Duration::from_nanos(1)) && r.capturing());
        assert!(r.expire(later) && !r.capturing() && r.until.is_none());
        assert!(later - until <= WAIT);
        assert_eq!(s, json!({}));
        // every button stays bindable: a button no listed action holds (none by default) or a swap between two
        for (p, _) in aegis_ui::controls::PADS {
            let holder = ACTIONS.iter().find(|a| a.key == c.pad(p)).expect("every default button gives a listed action");
            let other = ACTIONS.iter().find(|a| a.id != holder.id && c.pad_for(a.key).is_some()).unwrap();
            let mut s = json!({});
            click(&mut r, &mut s, REBIND, Some(other.id));
            let o = r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Pad(p));
            assert!(matches!(o, Outcome::Asked | Outcome::Cancelled), "{p:?} for {}: {o:?}", other.id);
            if o == Outcome::Asked {
                assert!(click(&mut r, &mut s, SWAP, None), "{p:?} swaps onto {}", other.id);
                assert_eq!(Controls::from_settings(&s).0.pad(p), other.key);
            }
        }
    }

    /// CONTROLS-UI1B (PR7-49 (3)): the keys screens read that the table does not list (`y`, `r`, the digits) are
    /// fixed: a rebind onto one is refused with a line naming the screen, nothing is written and the rebind waits.
    /// They are listed under the fixed inputs, and no action has one by default.
    #[test]
    fn a_fixed_key_is_refused_with_its_screen() {
        let (mut r, mut s) = (Rebind::default(), json!({}));
        let c = Controls::default();
        for f in &FIXED_KEYS {
            assert!(ACTIONS.iter().all(|a| a.key != f.key && c.key_for(a.key) != Some(f.key)), "{:?} is no action's", f.key);
            click(&mut r, &mut s, REBIND, Some("interact"));
            assert_eq!(r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Key(f.key)), Outcome::Refused, "{:?}", f.key);
            assert_eq!(state(&r), json!({"step": "waiting", "action": "interact", "fixed": f.key.name()}));
            assert!(matches!(view(&s, &r).get("prompt"), Some(Field::Text(t)) if t.contains("stays fixed") && t.contains(f.screen) && t.contains(&key_label(f.key))));
            assert_eq!(s, json!({}), "nothing written");
            assert_eq!(r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Key(Key::Esc)), Outcome::Cancelled);
        }
        for k in ["Y", "R", "1-9", "1-4"] {
            assert!(FIXED.iter().any(|(_, b)| *b == k), "{k} is listed fixed");
        }
        // Journal and Undo stay buttonless (FB-352 (3))
        for id in ["journal", "undo"] {
            assert_eq!(pad_text(&c, &ACTIONS[at(id)]), "no pad button");
        }
    }

    /// CONTROLS-UI1B (FB-351): each row gives its key in `detail` and its pad button in `detail2`, and a rebind shows
    /// in the column it changed.
    #[test]
    fn the_rows_split_key_and_pad_into_two_columns() {
        let (mut r, mut s) = (Rebind::default(), json!({}));
        let c = Controls::default();
        for a in &ACTIONS {
            let v = view(&s, &r);
            let rw = row(&v, a.id);
            assert_eq!((rw.detail.clone(), rw.detail2.clone()), (key_text(&c, a), pad_text(&c, a)), "{}", a.id);
        }
        assert_eq!((row(&view(&s, &r), "confirm").detail.as_str(), row(&view(&s, &r), "confirm").detail2.as_str()), ("Enter", "Pad A"));
        click(&mut r, &mut s, REBIND, Some("back"));
        r.input(&ACTIONS, &FIXED_KEYS, &mut s, Input::Pad(Pad::South));
        assert!(click(&mut r, &mut s, SWAP, None));
        let v = view(&s, &r);
        assert_eq!((row(&v, "back").detail.as_str(), row(&v, "back").detail2.as_str()), ("Esc", "Pad A"));
        assert_eq!(row(&v, "confirm").detail2, "Pad B");
    }

    /// US6: the fixture has the view's shape (field names and kinds, action ids), so a layout drawn against it fits the
    /// running tab.
    #[test]
    fn the_fixture_has_the_view_shape() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let layers = aegis_core::assets::Layers::of(vec![root]);
        let fixture = aegis_ui::screen_layout::load_fixture(&layers, SCREEN).expect("the fixture file").expect("it parses");
        let shape = |v: &View| (v.screen.clone(), v.fields.iter().map(|(n, f)| (n.clone(), f.kind())).collect::<Vec<_>>(), v.actions.iter().map(|a| a.id.clone()).collect::<Vec<_>>());
        assert_eq!(shape(&fixture), shape(&view(&json!({}), &Rebind::default())));
    }
}
