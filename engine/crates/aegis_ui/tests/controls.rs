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

// ---- CONTROLS-UI1 (PR7-44): rebinding -------------------------------------------------------------------------------

use aegis_ui::controls::{capturable, key_label, Action, Input, Outcome, Rebind, Step};

/// A game's list: three actions, two on letters and one on Enter.
const ACTIONS: [Action; 3] = [
    Action { id: "use", label: "Use", key: Key::Char('e') },
    Action { id: "undo", label: "Undo", key: Key::Char('z') },
    Action { id: "ok", label: "Confirm", key: Key::Enter },
];

/// Every key of a set gives one distinct key: the map stays a permutation of them.
fn is_permutation(c: &Controls, keys: &[Key]) -> bool {
    let mut out: Vec<String> = keys.iter().map(|k| c.key(*k).name()).collect();
    out.sort();
    let mut want: Vec<String> = keys.iter().map(|k| k.name()).collect();
    want.sort();
    out == want
}

/// `bind_key` swaps: the new key gives the action, the action's old key gives what the new one gave; any chain of binds
/// keeps every key giving exactly one thing, and `key_for` finds each action's key.
#[test]
fn binding_a_key_swaps_it_with_the_old_one() {
    let mut c = Controls::default();
    c.bind_key(Key::Char('e'), Key::Char('r'));
    assert_eq!((c.key(Key::Char('r')), c.key(Key::Char('e'))), (Key::Char('e'), Key::Char('r')));
    assert_eq!(c.key_for(Key::Char('e')), Some(Key::Char('r')));
    let keys = [Key::Char('e'), Key::Char('r'), Key::Char('z'), Key::Enter, Key::Up];
    c.bind_key(Key::Char('z'), Key::Char('r'));
    c.bind_key(Key::Enter, Key::Up);
    c.bind_key(Key::Char('e'), Key::Char('e'));
    assert!(is_permutation(&c, &keys), "{:?}", c.to_json());
    for k in keys {
        assert_eq!(c.key(c.key_for(k).unwrap()), k, "{k:?}");
    }
    // binding a key back to itself is a no-op past the swap: the map is written only where it differs
    let mut d = Controls::default();
    d.bind_key(Key::Char('e'), Key::Char('r'));
    d.bind_key(Key::Char('e'), Key::Char('e'));
    assert_eq!(d, Controls::default());
    // what the settings write reads back the same
    assert_eq!(Controls::from_settings(&json!({"controls": c.to_json()})), (c, Vec::new()));
}

/// `bind_pad` swaps the buttons the same way; `pad_for` is the first button in `PADS` order.
#[test]
fn binding_a_pad_button_swaps_it() {
    let mut c = Controls::default();
    assert_eq!(c.pad_for(Key::Esc), Some(Pad::East), "East before Start");
    c.bind_pad(Key::Enter, Pad::East);
    assert_eq!((c.pad(Pad::East), c.pad(Pad::South), c.pad(Pad::Start)), (Key::Enter, Key::Esc, Key::Esc));
    c.bind_pad(Key::Char('z'), Pad::West);
    assert_eq!(c.pad(Pad::West), Key::Char('z'), "an action with no button takes one");
    assert_eq!(c.pad_for(Key::Space), None, "and the one it held is gone");
}

/// The words the tab shows, and the keys a rebind refuses.
#[test]
fn key_and_button_labels() {
    assert_eq!([Key::Enter, Key::BackTab, Key::PageUp, Key::Char('e'), Key::Char('?'), Key::Fn(5)].map(key_label), ["Enter", "Shift+Tab", "Page Up", "E", "?", "F5"]);
    assert_eq!([Pad::South, Pad::LeftBumper, Pad::DUp].map(Pad::label), ["Pad A", "LB", "D-pad Up"]);
    assert!(PADS.iter().all(|(p, _)| !p.label().is_empty()));
    assert!(capturable(Key::Char('r')) && capturable(Key::Enter));
    assert!(!capturable(Key::Esc) && !capturable(Key::CtrlEnter) && !capturable(Key::Fn(9)) && !capturable(Key::ShiftDigit(1)));
}

/// The flow: Idle ignores input; a free key binds at once; another action's key asks, Swap (or Enter, or the pad
/// button that gives Enter) trades them, Esc cancels with nothing written; Reset drops the map.
#[test]
fn the_rebind_flow_asks_before_a_swap() {
    let (mut r, mut s) = (Rebind::default(), json!({"volume": 3}));
    assert_eq!(r.input(&ACTIONS, &[], &mut s, Input::Key(Key::Char('q'))), Outcome::Ignored);
    r.start(0);
    assert_eq!(r.input(&ACTIONS, &[], &mut s, Input::Key(Key::Char('q'))), Outcome::Bound);
    assert_eq!(s["controls"]["keys"]["q"], json!("e"));
    r.start(0);
    assert_eq!(r.input(&ACTIONS, &[], &mut s, Input::Key(Key::Char('z'))), Outcome::Asked);
    assert_eq!(r.step, Step::Clash { at: 0, input: Input::Key(Key::Char('z')), with: 1 });
    let before = s.clone();
    assert_eq!(r.input(&ACTIONS, &[], &mut s, Input::Key(Key::Esc)), Outcome::Cancelled);
    assert_eq!(s, before);
    r.start(0);
    r.input(&ACTIONS, &[], &mut s, Input::Key(Key::Char('z')));
    assert_eq!(r.input(&ACTIONS, &[], &mut s, Input::Pad(Pad::South)), Outcome::Bound, "the pad's confirm answers Swap");
    let c = Controls::from_settings(&s).0;
    assert_eq!((c.key_for(Key::Char('e')), c.key_for(Key::Char('z'))), (Some(Key::Char('z')), Some(Key::Char('q'))));
    r.start(2);
    assert_eq!(r.input(&ACTIONS, &[], &mut s, Input::Pad(Pad::West)), Outcome::Bound, "Space is no listed action");
    r.reset(&mut s);
    assert_eq!((s, r.capturing()), (json!({"volume": 3}), false));
}

/// A pad button for an action with no button is refused when another listed action holds it (a swap would leave that
/// one with none): the rebind keeps waiting and nothing is written. Two actions that both have buttons ask and swap.
#[test]
fn a_pad_swap_needs_a_button_to_trade() {
    let (mut r, mut s) = (Rebind::default(), json!({}));
    r.start(1);
    assert_eq!(r.input(&ACTIONS, &[], &mut s, Input::Pad(Pad::South)), Outcome::Refused, "Undo has no button; Pad A is Confirm's");
    assert_eq!((r.step, r.refused), (Step::Waiting(1), Some(2)));
    assert_eq!(s, json!({}));
    assert_eq!(r.input(&ACTIONS, &[], &mut s, Input::Pad(Pad::West)), Outcome::Bound, "Pad X gives no listed action: taken");
    assert_eq!(Controls::from_settings(&s).0.pad(Pad::West), Key::Char('z'));
    // Undo now has Pad X: Pad A is a swap with Confirm, asked first
    r.start(1);
    assert_eq!(r.input(&ACTIONS, &[], &mut s, Input::Pad(Pad::South)), Outcome::Asked);
    assert_eq!(r.step, Step::Clash { at: 1, input: Input::Pad(Pad::South), with: 2 });
    assert_eq!(r.swap(&ACTIONS, &mut s), Outcome::Bound);
    let c = Controls::from_settings(&s).0;
    assert_eq!((c.pad(Pad::South), c.pad(Pad::West)), (Key::Char('z'), Key::Enter));
}

/// In the swap question a raw Esc cancels even when the map makes Esc something else; Reset keeps the other settings'
/// order.
#[test]
fn raw_esc_cancels_the_question_and_reset_keeps_the_order() {
    let mut s = json!({"a": 1, "controls": {"keys": {"esc": "q"}}, "b": 2, "c": 3});
    let mut r = Rebind::default();
    r.start(0);
    assert_eq!(r.input(&ACTIONS, &[], &mut s, Input::Key(Key::Char('z'))), Outcome::Asked);
    assert_eq!(r.input(&ACTIONS, &[], &mut s, Input::Key(Key::Esc)), Outcome::Cancelled);
    r.reset(&mut s);
    assert_eq!(s.as_object().unwrap().keys().collect::<Vec<_>>(), ["a", "b", "c"]);
}

// ---- CONTROLS-UI1B: hints, the way out of a waiting rebind, fixed keys -----------------------------------------------

use aegis_ui::controls::{Fixed, WAIT};

/// A hint reads the bound key (and the pad button when the action has one), so a rebind shows on every screen.
#[test]
fn a_hint_follows_the_binding() {
    let mut c = Controls::default();
    assert_eq!((c.key_hint(Key::Char('p')), c.hint(Key::Char('p'))), ("P".to_string(), "P".to_string()));
    assert_eq!((c.key_hint(Key::Esc), c.hint(Key::Esc)), ("Esc".to_string(), "Esc / Pad B".to_string()));
    c.bind_key(Key::Char('p'), Key::Char('k'));
    c.bind_pad(Key::Esc, Pad::North);
    assert_eq!((c.hint(Key::Char('p')), c.hint(Key::Char('k'))), ("K".to_string(), "P".to_string()));
    assert_eq!(c.hint(Key::Esc), "Esc / Pad Y");
    assert_eq!(c.hint(Key::Tab), "Tab / Pad B", "the swapped action takes the old button");
}

/// A waiting rebind ends with nothing written on the input the action already has (its key or pad button), and after
/// `WAIT` with no input (frame-exact on the given instant); an input restarts the wait; the swap question ends too.
#[test]
fn a_waiting_rebind_ends_on_its_own_input_or_after_the_wait() {
    let (mut r, mut s) = (Rebind::default(), json!({}));
    assert!(!r.expire(Instant::now() + WAIT * 2), "idle: nothing to end");
    r.start(2);
    assert_eq!(r.input(&ACTIONS, &[], &mut s, Input::Pad(Pad::South)), Outcome::Cancelled, "Pad A is Confirm's own");
    assert_eq!((s.clone(), r.capturing()), (json!({}), false));
    r.start(0);
    assert_eq!(r.input(&ACTIONS, &[], &mut s, Input::Key(Key::Char('e'))), Outcome::Cancelled, "E is Use's own");
    assert_eq!(s, json!({}));
    r.start(0);
    let until = r.until.expect("an end");
    assert!(!r.expire(until - Duration::from_nanos(1)) && r.capturing());
    assert!(r.expire(until) && !r.capturing() && r.until.is_none());
    r.start(0);
    assert_eq!(r.input(&ACTIONS, &[], &mut s, Input::Key(Key::Char('z'))), Outcome::Asked);
    let until = r.until.unwrap();
    assert!(r.expire(until) && r.step == Step::Idle);
    assert_eq!(s, json!({}));
}

/// A fixed key is refused (its index in `refused_fixed`), the rebind waits and nothing is written; the next input
/// clears it.
#[test]
fn a_fixed_key_is_refused() {
    const FIXED: [Fixed; 1] = [Fixed { key: Key::Char('y'), label: "Yes", screen: "questions" }];
    let (mut r, mut s) = (Rebind::default(), json!({}));
    r.start(0);
    assert_eq!(r.input(&ACTIONS, &FIXED, &mut s, Input::Key(Key::Char('y'))), Outcome::Refused);
    assert_eq!((r.step, r.refused_fixed, s.clone()), (Step::Waiting(0), Some(0), json!({})));
    assert_eq!(r.input(&ACTIONS, &FIXED, &mut s, Input::Key(Key::Char('k'))), Outcome::Bound);
    assert_eq!(r.refused_fixed, None);
    assert_eq!(Controls::from_settings(&s).0.key_for(Key::Char('e')), Some(Key::Char('k')));
}
