//! STORYUI1-E: the vignette player (docs/specs/STORYUI1.md 5.1, tests TE1-TE10 of STORYUI1_TASKS.md 2). The model is
//! driven without a clock (`advance`, `press`, `skip`); expectations come from the rules: the phase order, the text
//! speeds' characters a second, the fade formulas, skip and cue rules, the music wish, the wrap-once typewriter and the
//! layout at every text size (the text audit).

use aegis_ui::art::Art;
use aegis_ui::audio::Audio;
use aegis_ui::canvas::Canvas;
use aegis_ui::painter::{Fonts, Painter, View};
use aegis_ui::playtest::Playtests;
use aegis_ui::screen::{Context, Cx, Nav, Screen, Settings, TEXT_SIZES, TEXT_SPEEDS};
use aegis_ui::text_audit;
use aegis_ui::ui::{Gfx, Kit, Ui, H, W};
use aegis_ui::vignette::{self, Card, Ended, Palette, Phase, Player, Reel, Style, TextSpeed, VignetteScreen, CARD_BODY, SKIP_TAG};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::rc::Rc;

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
        settings: Settings::load(std::env::temp_dir().join("aegis_vignette_no_settings.json"), json!({})),
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

/// A text of `n` characters made of words of mixed lengths (no word longer than a line).
fn words(n: usize) -> String {
    let pool = ["the", "lantern", "a", "river", "of", "glass", "under", "seven", "bells", "and", "nobody", "answered", "quietly"];
    let mut s = String::new();
    let mut i = 0;
    while s.chars().count() < n {
        if !s.is_empty() {
            s.push(' ');
        }
        s.push_str(pool[i % pool.len()]);
        i += 1;
    }
    let mut t: String = s.chars().take(n).collect();
    if t.ends_with(' ') {
        t.pop();
        t.push('.');
    }
    t
}

fn card(text: &str) -> Card {
    Card { style: Style::Image, text: text.into(), ..Card::default() }
}

fn reel(cards: Vec<Card>) -> Reel {
    Reel { id: "test".into(), cards, ..Reel::default() }
}

/// Presses until the player reaches `phase` on its current card (at most a few presses).
fn to_phase(p: &mut Player, phase: Phase) {
    for _ in 0..4 {
        if p.phase() == phase {
            return;
        }
        p.press();
    }
    assert_eq!(p.phase(), phase);
}

#[test]
fn te1_each_press_moves_exactly_one_phase_through_two_cards() {
    let mut p = Player::new(reel(vec![card("First card."), card("Second card.")]), TextSpeed::Normal);
    let walk = [(0, Phase::FadeIn), (0, Phase::Typing), (0, Phase::Hold), (0, Phase::FadeOut), (1, Phase::FadeIn), (1, Phase::Typing), (1, Phase::Hold), (1, Phase::FadeOut)];
    for (i, (card, phase)) in walk.iter().enumerate() {
        assert_eq!((p.card_index(), p.phase()), (*card, *phase), "after {i} presses");
        p.press();
    }
    assert!(p.done(), "the press that ends the last fade out ends the reel");
    assert!(!p.skipped());
    p.press();
    assert!(p.done(), "a press after the end does nothing");
    // The same walk by time: each phase lasts its own length.
    let c = Card::default();
    let text = "Twelve chars";
    let mut p = Player::new(reel(vec![card(text), card(text)]), TextSpeed::Normal);
    p.advance(c.fade_in_ms - 1);
    assert_eq!(p.phase(), Phase::FadeIn);
    p.advance(1);
    assert_eq!(p.phase(), Phase::Typing);
    let typing = (text.chars().count() as u32 * 1000).div_ceil(60);
    p.advance(typing);
    assert_eq!(p.phase(), Phase::Hold);
    p.press();
    p.advance(c.fade_out_ms);
    assert_eq!((p.card_index(), p.phase()), (1, Phase::FadeIn));
}

#[test]
fn te2_typing_reveals_characters_at_the_text_speed() {
    let text = words(300);
    for speed in TextSpeed::ALL {
        let mut p = Player::new(reel(vec![card(&text)]), speed);
        match speed.cps() {
            Some(cps) => {
                p.press(); // end the fade in
                assert_eq!((p.phase(), p.shown()), (Phase::Typing, 0), "{speed:?}: typing starts with nothing shown");
                p.advance(1000);
                assert_eq!(p.shown(), cps as usize, "{speed:?}: one second shows its characters a second");
            }
            // instant fades are 0, so the card is past its fade in and its typing at once
            None => assert_eq!((p.phase(), p.shown()), (Phase::Hold, p.total()), "instant shows it all as typing starts"),
        }
    }
    let speeds: Vec<(Option<u32>, &str)> = TextSpeed::ALL.iter().map(|s| (s.cps(), s.name())).collect();
    assert_eq!(speeds, vec![(Some(30), "slow"), (Some(60), "normal"), (Some(120), "fast"), (None, "instant")]);
    assert_eq!(TEXT_SPEEDS.to_vec(), TextSpeed::ALL.iter().map(|s| s.name()).collect::<Vec<_>>(), "the settings' choices are the speeds");
    // A press while typing shows the whole text.
    let mut p = Player::new(reel(vec![card(&text)]), TextSpeed::Slow);
    p.press();
    p.advance(500);
    p.press();
    assert_eq!((p.phase(), p.shown()), (Phase::Hold, p.total()));
}

#[test]
fn te2_the_text_speed_and_reduce_motion_are_engine_settings() {
    let d = Settings::engine_defaults();
    assert_eq!((d["text_speed"].clone(), d["reduce_motion"].clone()), (json!("normal"), json!(false)));
    let mut s = Settings::load(std::env::temp_dir().join("aegis_vignette_no_settings.json"), json!({}));
    assert_eq!((TextSpeed::of(&s), vignette::reduce_motion(&s)), (TextSpeed::Normal, false));
    s.v["text_speed"] = json!("fast");
    s.v["reduce_motion"] = json!(true);
    assert_eq!((TextSpeed::of(&s), vignette::reduce_motion(&s)), (TextSpeed::Fast, true));
    s.v["text_speed"] = json!("warp");
    assert_eq!(TextSpeed::of(&s), TextSpeed::Normal, "an unknown value reads as the default");
}

#[test]
fn te3_a_timed_hold_moves_on_by_itself_and_a_zero_hold_waits() {
    let mut c = card("Held.");
    c.hold_ms = 500;
    let mut p = Player::new(reel(vec![c.clone(), card("Next.")]), TextSpeed::Normal);
    to_phase(&mut p, Phase::Hold);
    p.advance(c.hold_ms - 1);
    assert_eq!(p.phase(), Phase::Hold, "not before the hold is over");
    assert!(!p.waiting(), "a timed hold shows no hint");
    p.advance(1);
    assert_eq!(p.phase(), Phase::FadeOut, "the fade out starts hold_ms after the text is out");
    let mut p = Player::new(reel(vec![card("Waits.")]), TextSpeed::Normal);
    to_phase(&mut p, Phase::Hold);
    p.advance(10_000_000);
    assert_eq!(p.phase(), Phase::Hold, "a hold of 0 waits for a press");
    assert!(p.waiting());
}

#[test]
fn te4_skip_ends_a_skippable_reel_and_a_fixed_one_plays_on() {
    let cards: Vec<Card> = (0..3).map(|i| Card { cue: format!("sfx/c{i}.wav"), ..card("Cue card.") }).collect();
    let mut p = Player::new(reel(cards.clone()), TextSpeed::Normal);
    assert_eq!(p.take_cues(), vec!["sfx/c0.wav".to_string()]);
    assert!(p.skip());
    assert!(p.done() && p.skipped());
    p.advance(100_000);
    p.press();
    assert!(p.take_cues().is_empty(), "no cue after a skip");
    let mut fixed = reel(cards);
    fixed.skippable = false;
    let mut p = Player::new(fixed, TextSpeed::Normal);
    p.press();
    assert!(!p.skip(), "a reel that is not skippable ignores skip");
    assert_eq!((p.card_index(), p.phase(), p.done()), (0, Phase::Typing, false));
    while !p.done() {
        p.press();
    }
    assert!(!p.skipped());
}

#[test]
fn te5_the_cover_is_half_at_a_fades_midpoint() {
    let c = Card { fade_in_ms: 600, fade_out_ms: 400, ..card("Fades.") };
    for speed in [TextSpeed::Normal, TextSpeed::Fast] {
        let fade_in = speed.fade(c.fade_in_ms);
        let fade_out = speed.fade(c.fade_out_ms);
        assert_eq!((fade_in, fade_out), if speed == TextSpeed::Fast { (c.fade_in_ms / 2, c.fade_out_ms / 2) } else { (c.fade_in_ms, c.fade_out_ms) }, "fast halves the fades");
        let mut p = Player::new(reel(vec![c.clone(), card("After.")]), speed);
        assert_eq!(p.cover_alpha(), 255, "{speed:?}: the fade in starts covered");
        p.advance(fade_in / 2);
        assert!(p.cover_alpha().abs_diff(128) <= 1, "{speed:?}: fade in midpoint {}", p.cover_alpha());
        p.advance(fade_in - fade_in / 2);
        assert_eq!((p.phase(), p.cover_alpha()), (Phase::Typing, 0), "{speed:?}: the fade is over exactly at its length");
        to_phase(&mut p, Phase::FadeOut);
        assert_eq!(p.cover_alpha(), 0, "{speed:?}: the fade out starts clear");
        p.advance(fade_out / 2);
        assert!(p.cover_alpha().abs_diff(128) <= 1, "{speed:?}: fade out midpoint {}", p.cover_alpha());
        p.advance(fade_out - fade_out / 2);
        assert_eq!(p.card_index(), 1, "{speed:?}: the next card at the fade out's end");
    }
    let p = Player::new(reel(vec![c.clone()]), TextSpeed::Instant);
    assert_eq!((TextSpeed::Instant.fade(c.fade_in_ms), p.phase(), p.cover_alpha()), (0, Phase::Hold, 0), "instant has no fades");
}

#[test]
fn te6_a_cue_fires_once_per_card_and_none_after_a_skip() {
    let cards: Vec<Card> = (0..3).map(|i| Card { cue: format!("sfx/c{i}.wav"), hold_ms: 200, ..card("Cue card.") }).collect();
    let mut p = Player::new(reel(cards.clone()), TextSpeed::Normal);
    let mut heard = p.take_cues();
    for _ in 0..200 {
        p.advance(50);
        heard.extend(p.take_cues());
    }
    assert!(p.done(), "timed holds play the reel through");
    assert_eq!(heard, vec!["sfx/c0.wav", "sfx/c1.wav", "sfx/c2.wav"], "each card's cue once, in order");
    let mut p = Player::new(reel(cards), TextSpeed::Normal);
    let mut heard = p.take_cues();
    while p.card_index() < 1 {
        p.press();
    }
    heard.extend(p.take_cues());
    p.skip();
    p.advance(100_000);
    heard.extend(p.take_cues());
    assert_eq!(heard, vec!["sfx/c0.wav", "sfx/c1.wav"], "no cue for the cards after the skip");
    // A card without a cue plays none.
    let mut p = Player::new(reel(vec![card("Silent.")]), TextSpeed::Normal);
    assert!(p.take_cues().is_empty());
}

#[test]
fn te7_the_music_wish() {
    let wish = |m: &str| Reel { music: m.into(), ..Reel::default() }.music_wish();
    assert_eq!(wish(""), None, "\"\" leaves the music to the screen below");
    assert_eq!(wish("none"), Some(String::new()), "none is silence");
    assert_eq!(wish("music/theme.ogg"), Some("music/theme.ogg".into()));
    let cx = cx();
    let s: VignetteScreen<App> = VignetteScreen::new(Reel { music: "none".into(), ..reel(vec![card("x")]) }, TextSpeed::Normal, |_, _| {});
    assert_eq!(s.music(&cx), Some(String::new()), "the screen wishes the reel's music");
}

#[test]
fn te8_a_half_typed_caption_breaks_where_the_full_one_does() {
    let mut g = gfx();
    let text = words(360);
    let total = text.chars().count();
    let width = 300.0;
    let mut p = Painter { c: &mut g.canvas, v: g.view, fonts: &mut g.fonts };
    let full = p.wrap(&text, CARD_BODY, width);
    assert!(full.len() >= 3, "the sample wraps over several lines");
    for shown in (0..=total).step_by(7).chain([total]) {
        let typed = vignette::typed_lines(&mut p, &text, shown, CARD_BODY, width);
        assert_eq!(typed.iter().map(|t| t.0.clone()).collect::<Vec<_>>(), full, "shown {shown}: the same line breaks");
        for k in 0..typed.len() {
            assert!(typed[k].1 <= typed[k].0.chars().count());
            if k + 1 < typed.len() && typed[k + 1].1 > 0 {
                assert_eq!(typed[k].1, typed[k].0.chars().count(), "shown {shown}: a line is typed out before the next starts");
            }
        }
        // the typed characters and the spaces at the breaks they passed add up to `shown`
        let typed_chars: usize = typed.iter().map(|t| t.1).sum();
        let breaks = typed.iter().skip(1).filter(|t| t.1 > 0).count();
        assert!(typed_chars + breaks <= shown && shown <= typed_chars + breaks + 1, "shown {shown}: {typed_chars} + {breaks}");
    }
    // Wrapping the typed prefix alone would break some line elsewhere: the reason for wrapping once.
    let differs = (1..total).any(|n| {
        let prefix: String = text.chars().take(n).collect();
        let alone = p.wrap(&prefix, CARD_BODY, width);
        alone.iter().zip(&full).any(|(a, f)| !f.starts_with(a.as_str()))
    });
    assert!(differs, "some half-typed prefix wraps differently on its own");
}

/// A screen on `c` drawn in its hold (every text out) at text size `scale`, with the audit on: the audit's counts.
fn audited(c: Card, scale: f32) -> Value {
    let (mut cx, mut g) = (cx(), gfx());
    let mut s: VignetteScreen<App> = VignetteScreen::new(reel(vec![c]), TextSpeed::Normal, |_, _| {});
    to_phase(&mut s.player, Phase::Hold);
    g.fonts.scale = scale;
    g.fonts.audit = Some(Vec::new());
    s.draw(&mut cx, &mut g, (-1, -1), true);
    let recs = g.fonts.audit.take().unwrap();
    let boxes: Vec<(String, f32, f32, f32, f32)> = s.widgets().iter().map(|w| (w["id"].as_str().unwrap().to_string(), w["rect"][0].as_f64().unwrap() as f32, w["rect"][1].as_f64().unwrap() as f32, w["rect"][2].as_f64().unwrap() as f32, w["rect"][3].as_f64().unwrap() as f32)).collect();
    let v = text_audit::issues(&recs, W as f32, H as f32, &boxes);
    assert!(v["texts"].as_u64().unwrap() > 0, "the card drew its text");
    v
}

#[test]
fn te9_a_360_character_caption_fits_at_every_text_size() {
    let text = words(360);
    assert_eq!(text.chars().count(), 360);
    let cards = [
        Card { style: Style::Image, text: text.clone(), ..Card::default() },
        Card { style: Style::Text, text: text.clone(), ..Card::default() },
        Card { style: Style::Line, speaker: "A speaker with a long name".into(), text: text.clone(), ..Card::default() },
        Card { style: Style::Title, title: words(48), subtitle: words(80), ..Card::default() },
    ];
    for (label, scale) in TEXT_SIZES {
        for c in &cards {
            let v = audited(c.clone(), scale as f32);
            assert_eq!(v["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "{label}, {}: {v}", c.style.name());
        }
    }
}

#[test]
fn te10_the_model_never_reads_a_clock() {
    let mut cards: Vec<Card> = (0..3).map(|i| Card { ash: true, hold_ms: 300 * i, cue: format!("sfx/{i}.wav"), ..card(&words(90)) }).collect();
    cards[1].style = Style::Text;
    let calls: Vec<Option<u32>> = vec![Some(16), Some(300), None, Some(33), Some(1000), None, Some(5), Some(700), None, Some(2000), Some(16), None, Some(4000)];
    let run = || {
        let mut p = Player::new(reel(cards.clone()), TextSpeed::Normal);
        let mut g = gfx();
        let mut art = Art::new(None);
        let mut frames = Vec::new();
        for c in &calls {
            match c {
                Some(ms) => p.advance(*ms),
                None => p.press(),
            }
            let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
            vignette::draw(&mut ui, &mut art, &p, &Palette::default(), false);
            drop(ui);
            frames.push((p.state(), p.take_cues(), g.canvas.px.clone()));
        }
        frames
    };
    let (a, b) = (run(), run());
    assert_eq!(a.len(), b.len());
    for (i, (x, y)) in a.iter().zip(&b).enumerate() {
        assert_eq!((&x.0, &x.1), (&y.0, &y.1), "call {i}: the same state and cues");
        assert!(x.2 == y.2, "call {i}: the same frame");
    }
    // Reading and drawing leave the model where it is.
    let p = Player::new(reel(cards.clone()), TextSpeed::Normal);
    let before = p.state();
    let (mut g, mut art) = (gfx(), Art::new(None));
    let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
    vignette::draw(&mut ui, &mut art, &p, &Palette::default(), false);
    assert_eq!(p.state(), before);
}

#[test]
fn reduce_motion_hides_the_ash() {
    let frame = |ash: bool, still: bool| {
        let mut p = Player::new(reel(vec![Card { style: Style::Text, ash, top: [40, 40, 60, 255], bottom: [10, 10, 20, 255], ..card("Ash.") }]), TextSpeed::Normal);
        p.press();
        p.press();
        p.advance(2500);
        let mut g = gfx();
        let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
        vignette::draw(&mut ui, &mut Art::new(None), &p, &Palette::default(), still);
        drop(ui);
        g.canvas.px
    };
    assert!(frame(true, false) != frame(false, false), "ash draws specks");
    assert!(frame(true, true) == frame(false, false), "reduce motion draws none");
}

#[test]
fn the_agent_view_and_the_widgets() {
    let (mut cx, mut g) = (cx(), gfx());
    let mut s: VignetteScreen<App> = VignetteScreen::new(reel(vec![card("Hello."), card("Again.")]), TextSpeed::Normal, |_, _| {});
    s.draw(&mut cx, &mut g, (-1, -1), true);
    let st = s.state(&cx);
    for k in ["reel", "card", "cards", "phase", "shown", "total", "done"] {
        assert!(st.get(k).is_some(), "state has {k}: {st}");
    }
    assert_eq!((st["reel"].clone(), st["cards"].clone(), st["phase"].clone(), st["total"].clone()), (json!("test"), json!(2), json!("fade_in"), json!(6)));
    let ids: Vec<String> = s.widgets().iter().map(|w| w["id"].as_str().unwrap().to_string()).collect();
    assert_eq!(ids, vec!["vignette.next", "vignette.skip"]);
    s.click_id(&mut cx, "vignette.next").unwrap();
    assert_eq!(s.state(&cx)["phase"], json!("typing"), "the stage is a press");
    s.key(&mut cx, aegis_ui::Key::Space);
    assert_eq!(s.state(&cx)["phase"], json!("hold"), "Space is a press");
    s.key(&mut cx, aegis_ui::Key::Enter);
    assert_eq!(s.state(&cx)["phase"], json!("fade_out"), "Enter is a press");
    let mut fixed: VignetteScreen<App> = VignetteScreen::new(Reel { skippable: false, ..reel(vec![card("x")]) }, TextSpeed::Normal, |_, _| {});
    fixed.draw(&mut cx, &mut g, (-1, -1), true);
    assert!(fixed.widgets().iter().all(|w| w["id"] != "vignette.skip"), "no Skip on a reel that is not skippable");
    fixed.back(&mut cx);
    fixed.escape(&mut cx);
    assert!(!fixed.player.done() && cx.nav.is_empty(), "right click and Esc are refused");
}

/// A screen whose `on_done` records how it ended (and pops, as a caller would).
fn recording(r: Reel, ask: bool) -> (VignetteScreen<App>, Rc<RefCell<Vec<Ended>>>) {
    let ends = Rc::new(RefCell::new(Vec::new()));
    let e = ends.clone();
    let s = VignetteScreen::new(r, TextSpeed::Normal, move |cx: &mut Cx<App>, how| {
        e.borrow_mut().push(how);
        cx.nav.push(Nav::Pop);
    })
    .ask_first(ask);
    (s, ends)
}

#[test]
fn on_done_runs_once_when_the_reel_ends_or_is_skipped() {
    let mut cx = cx();
    let (mut s, ends) = recording(reel(vec![Card { hold_ms: 100, ..card("One.") }]), false);
    for _ in 0..100 {
        s.advance(&mut cx, 50);
    }
    assert_eq!(*ends.borrow(), vec![Ended::Played], "once, when played through");
    assert!(matches!(cx.nav.as_slice(), [Nav::Pop]));
    cx.nav.clear();
    let (mut s, ends) = recording(reel(vec![card("One."), card("Two.")]), false);
    s.back(&mut cx);
    s.advance(&mut cx, 1000);
    s.back(&mut cx);
    assert_eq!(*ends.borrow(), vec![Ended::Skipped], "a right click skips at once without ask_first");
    // An empty reel is over at its first frame.
    let (mut s, ends) = recording(reel(Vec::new()), false);
    s.advance(&mut cx, 0);
    assert_eq!(*ends.borrow(), vec![Ended::Played]);
}

#[test]
fn ask_first_asks_before_skipping_with_yes_the_default() {
    let mut cx = cx();
    let (mut s, ends) = recording(reel(vec![card("Unseen.")]), true);
    s.back(&mut cx);
    let Some(Nav::Push(mut ask)) = cx.nav.pop() else { panic!("the question is pushed") };
    assert_eq!(ask.state(&cx), json!({"question": "Skip this scene?", "default": "yes"}));
    assert!(!s.player.done(), "nothing skipped while the question is open");
    // No: back on the scene, still playing.
    ask.escape(&mut cx);
    cx.nav.clear();
    s.resume(&mut cx);
    assert!(!s.player.done() && ends.borrow().is_empty());
    // Yes (Enter, the default): the scene is skipped on its return.
    s.escape(&mut cx);
    let Some(Nav::Push(mut ask)) = cx.nav.pop() else { panic!("asked again") };
    let mut g = gfx();
    ask.draw(&mut cx, &mut g, (-1, -1), true);
    ask.key(&mut cx, aegis_ui::Key::Enter);
    cx.nav.clear();
    assert!(cx.answers.iter().any(|a| a == SKIP_TAG));
    s.resume(&mut cx);
    assert_eq!(*ends.borrow(), vec![Ended::Skipped]);
}

#[test]
fn a_reel_reads_from_its_json_form() {
    let r = Reel::from_json(&json!({"id": "v", "music": "none", "frame": "#102030", "cards": [
        {"style": "title", "title": "T", "top": "#010203"},
        {"text": "Caption", "top": "#0a0b0c", "bottom": "#ffffff", "hold_ms": 99999, "fade_in_ms": -5, "cue": "sfx/x.wav", "ash": true},
        {"style": "nonsense", "top": "red"}
    ]}));
    assert_eq!((r.id.as_str(), r.music.as_str(), r.frame, r.skippable, r.cards.len()), ("v", "none", [16, 32, 48, 255], true, 3));
    let d = Card::default();
    assert_eq!((r.cards[0].style, r.cards[0].top, r.cards[0].bottom), (Style::Title, [1, 2, 3, 255], [1, 2, 3, 255]), "bottom defaults to top");
    assert_eq!((r.cards[1].style, r.cards[1].hold_ms, r.cards[1].fade_in_ms, r.cards[1].fade_out_ms, r.cards[1].ash), (Style::Image, 20000, 0, d.fade_out_ms, true), "timings are clamped to their ranges");
    assert_eq!((r.cards[2].style, r.cards[2].top), (Style::Image, d.top), "bad values read as the defaults");
    assert_eq!((vignette::hex("#ff8000"), vignette::hex("ff8000"), vignette::hex("#ff80")), (Some([255, 128, 0, 255]), None, None));
    let sample = vignette::sample_reel();
    let mut styles: Vec<&str> = sample.cards.iter().map(|c| c.style.name()).collect();
    styles.sort();
    styles.dedup();
    assert_eq!(styles.len(), 4, "the sample reel shows every style");
}
