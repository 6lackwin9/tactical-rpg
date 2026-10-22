//! ASKNO1: the kit's yes / no question (`Cx::ask_tag`, `Cx::ask_tag_no`). The default question answers Yes on Enter
//! and opens without a visible focus; the destructive form answers No on Enter and opens with the focus on No. Both
//! answer Yes on `y` and No on `n`, Esc and right click, with the same widget ids.

use aegis_ui::art::Art;
use aegis_ui::audio::Audio;
use aegis_ui::canvas::Canvas;
use aegis_ui::focus;
use aegis_ui::painter::{Fonts, View};
use aegis_ui::playtest::Playtests;
use aegis_ui::screen::{Context, Cx, Key, Nav, Screen, Settings};
use aegis_ui::ui::{Gfx, Kit, H, W};
use serde_json::{json, Value};

struct App;

impl Context for App {
    const MORE_COMMANDS: &'static str = "none";
    fn note(&mut self, _line: String) {}
    fn agent_summary(&self) -> Value {
        Value::Null
    }
    fn agent_game(&self) -> Value {
        Value::Null
    }
}

fn cx() -> Cx<App> {
    Cx {
        settings: Settings::load(std::env::temp_dir().join("aegis_ask_no_settings.json"), json!({})),
        art: Art::new(None),
        audio: Audio::offline(None),
        nav: Vec::new(),
        notice: Vec::new(),
        shot_dir: std::env::temp_dir(),
        quit: false,
        answers: Vec::new(),
        playtest: Playtests::off(),
        app: App,
    }
}

fn gfx() -> Gfx {
    let font = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
    Gfx { canvas: Canvas::new(W, H), view: View::fit(W as u32, H as u32), fonts: Fonts::new(&font), kit: Kit::default() }
}

const TAG: &str = "test.discard";

/// The question the call pushed, drawn once (its widgets known); `cx.nav` left empty.
fn asked(cx: &mut Cx<App>, g: &mut Gfx, no_default: bool) -> Box<dyn Screen<App>> {
    if no_default {
        cx.ask_tag_no("Discard this?", TAG);
    } else {
        cx.ask_tag("Discard this?", TAG);
    }
    let Some(Nav::Push(mut s)) = cx.nav.pop() else { panic!("the question is pushed") };
    assert!(cx.nav.is_empty());
    s.draw(cx, g, (-1, -1), true);
    s
}

/// One key on a fresh question: whether it answered Yes and whether it closed.
fn answer(no_default: bool, k: Option<Key>) -> (bool, bool) {
    let (mut cx, mut g) = (cx(), gfx());
    let mut s = asked(&mut cx, &mut g, no_default);
    match k {
        Some(k) => s.key(&mut cx, k),
        None => s.escape(&mut cx),
    }
    (cx.take_answer(TAG), matches!(cx.nav.as_slice(), [Nav::Pop]))
}

fn ids(s: &dyn Screen<App>) -> Vec<String> {
    s.widgets().iter().filter_map(|w| w["id"].as_str().map(String::from)).collect()
}

#[test]
fn the_default_question_answers_yes_on_enter() {
    assert_eq!(answer(false, Some(Key::Enter)), (true, true), "Enter = Yes");
    assert_eq!(answer(false, Some(Key::Char('y'))), (true, true), "y = Yes");
    assert_eq!(answer(false, Some(Key::Char('n'))), (false, true), "n = No");
    assert_eq!(answer(false, None), (false, true), "Esc = No");
    assert_eq!(answer(false, Some(Key::Char('x'))), (false, false), "another key does nothing");
    let (mut cx, mut g) = (cx(), gfx());
    let s = asked(&mut cx, &mut g, false);
    assert!(!s.auto_focus(), "no focus shown as it opens");
    assert_eq!(s.state(&cx)["default"], "yes");
}

#[test]
fn the_destructive_question_answers_no_on_enter() {
    assert_eq!(answer(true, Some(Key::Enter)), (false, true), "Enter = No");
    assert_eq!(answer(true, Some(Key::Char('y'))), (true, true), "y = Yes");
    assert_eq!(answer(true, Some(Key::Char('n'))), (false, true), "n = No");
    assert_eq!(answer(true, None), (false, true), "Esc = No");
    assert_eq!(answer(true, Some(Key::Char('x'))), (false, false), "another key does nothing");
    let (mut cx, mut g) = (cx(), gfx());
    let mut s = asked(&mut cx, &mut g, true);
    assert!(s.auto_focus(), "the focus shows as it opens");
    assert_eq!(focus::step(&s.widgets(), None, focus::Move::Next, false).as_deref(), Some("ask.no"), "the opening focus is No");
    assert_eq!(s.state(&cx)["default"], "no");
    // right click is No too; the buttons still answer what they say
    s.back(&mut cx);
    assert!(!cx.take_answer(TAG) && matches!(cx.nav.as_slice(), [Nav::Pop]));
    let (mut cx, mut g) = (self::cx(), gfx());
    let mut s = asked(&mut cx, &mut g, true);
    s.click_id(&mut cx, "ask.yes").expect("the Yes button");
    assert!(cx.take_answer(TAG), "the Yes button is Yes");
}

#[test]
fn both_forms_keep_the_same_widget_ids() {
    let (mut cx, mut g) = (cx(), gfx());
    let old = asked(&mut cx, &mut g, false);
    let new = asked(&mut cx, &mut g, true);
    assert_eq!(ids(old.as_ref()), ids(new.as_ref()));
    assert!(ids(new.as_ref()).iter().any(|i| i == "ask.no") && ids(new.as_ref()).iter().any(|i| i == "ask.yes"));
    assert_eq!(old.id(), new.id());
}
