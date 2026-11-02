//! The story stage (STORYUI1-U1, docs/specs/STORYUI1.md 5.2-5.4; engine-only, no Godot original): the screen a running
//! story sequence shows (`StoryStage`), playing the kit's steps (`emberwake_world::story`): a `vignette` step through
//! the engine's vignette player (`aegis_ui::vignette::VignetteScreen`), a `fight` step as a world fight with runner
//! `story`, the `enter` and `end` steps by leaving for the region map or the main menu; queued vignettes play first
//! (Q9). On the maps, `map_tick` hands over to the stage while a sequence runs and plays the vignette queue (W2). The
//! card text takes the theme's colours (`palette`). Story-free: it names no place, person or flag (KIT2-UI moves the
//! screens to the kit later, KIT1 KQ4).

use crate::screens::{show_errors, Cx, GameCx, Nav, Screen};
use aegis_core::json::{self, s, s_or, Dict};
use aegis_ui::canvas::{rgb, Rect};
use aegis_ui::theme::th;
use aegis_ui::ui::{Gfx, Ui, Widget, BODY, H, W};
use aegis_ui::vignette::{self, Card, Ended, Palette, Reel, VignetteScreen};
use aegis_ui::widget_plumbing;
use emberwake_data::Step;
use emberwake_world::{story, Data};
use serde_json::{json, Value};

/// The card colours (STORYUI1 3, LOOK1-G2: from the theme): `text`, `gold` as the accent, and a dim of the card's own
/// (Polished Steel, hue 06: a card's dim lines sit on art, whatever the theme's `dim` is).
pub fn palette() -> Palette {
    let t = th();
    Palette { text: t.text, accent: t.gold, dim: t.game("story_dim", rgb(0x9c, 0xa3, 0xaf)) }
}

/// A card of world `world` as the engine plays it (W5): the card keys through `Card::from_json`; a `line` card's speaker
/// shows the character's name (`speaker_name` when the kit resolved it) and its portrait chain for the card's
/// `expression` (PORTRAITFX1-X: the kit's `story::card_portrait`, `face_chain`; with no frames today's chain).
pub fn card_of(d: &Data, world: &str, raw: &Dict) -> Card {
    let mut c = Card::from_json(&Value::Object(raw.clone()));
    let speaker = s(raw, "speaker");
    if !speaker.is_empty() {
        let who = d.resolve(world, "characters", &speaker);
        c.speaker = s_or(raw, "speaker_name", &s_or(who, "name", &speaker));
        c.portrait = story::card_portrait(d, world, raw);
        // PORTRAITFX1-D: the portrait's effects are seeded by the character id, as the talk box's
        c.portrait_key = s(who, "id");
    }
    c
}

/// A scene (`story::scene`: the vignette as it plays now, cards resolved) as the engine's reel.
pub fn reel_of(d: &Data, world: &str, scene: &Dict) -> Reel {
    let mut reel = Reel::from_json(&json!({"id": s(scene, "vignette"), "music": s(scene, "music"), "frame": s(scene, "frame"), "skippable": json::b(scene, "skippable", true)}));
    reel.cards = json::aget(scene, "cards").iter().map(|c| card_of(d, world, json::dict(c))).collect();
    reel
}

/// A world's vignette entry as written (the UI Gallery, no game: every card shown, tokens as written).
pub fn entry_reel(d: &Data, world: &str, entry: &Dict) -> Reel {
    let mut reel = Reel::from_json(&Value::Object(entry.clone()));
    reel.id = s_or(entry, "_qid", &s(entry, "id"));
    reel.cards = json::aget(entry, "cards").iter().map(|c| card_of(d, world, json::dict(c))).collect();
    reel
}

/// A vignette screen over `reel` as the game plays it: the player's text speed, the game's palette, its tuned shake
/// presets and limits for the cards' `shake` (FX1 5.2; `tuning.json` `shakes`, `shake_max_px`, `shake_max_ms`), and
/// the line cards' portraits lit by the game's rim key (PORTRAITFX1-D, 6.4: Tallow Amber), an expression frame without
/// its own sidecar taking its base portrait's (PORTRAITFX1-X: the kit's `face_sidecars`) and the data's expression effects
/// drawn while a card's `expression` holds (PORTRAITFX1-EX).
pub fn vignette_screen(cx: &Cx, reel: Reel, on_done: impl FnOnce(&mut Cx, Ended) + 'static) -> VignetteScreen<GameCx> {
    let t = &cx.tuning.value;
    VignetteScreen::with_settings(&cx.settings, reel, on_done).palette(palette()).with_shakes(t.shakes.clone(), t.shake_limits()).portrait_rim(Some(crate::portraits::RIM))
        .portrait_sidecars(emberwake_content::art::face_sidecars)
        .portrait_expressions(crate::portraits::expression_fx_of(&cx.data.cat.portrait_expressions))
}

/// The player for `scene` in the game's palette; Skip asks first for a scene not seen yet (W6).
fn scene_screen(cx: &Cx, scene: &Dict, on_done: impl FnOnce(&mut Cx, Ended) + 'static) -> VignetteScreen<GameCx> {
    let reel = reel_of(&cx.data, &cx.gs().world_id, scene);
    vignette_screen(cx, reel, on_done).ask_first(!json::b(scene, "seen", false))
}

/// W2: plays the queue's oldest vignette (pushed over the screen on top); its end or skip takes it off the queue and
/// marks it seen. The pushed reel's music wish, or None when nothing was queued.
fn play_queued(cx: &mut Cx) -> Option<Option<String>> {
    let d = cx.data.clone();
    let scene = story::next_scene(&d, cx.gs_mut())?;
    let qid = s(&scene, "vignette");
    let screen = scene_screen(cx, &scene, move |cx: &mut Cx, _| {
        story::vignette_done(cx.gs_mut(), &qid);
        cx.nav.push(Nav::Pop);
    });
    let wish = screen.player.reel.music_wish();
    cx.nav.push(Nav::Push(Box::new(screen)));
    Some(wish)
}

/// A map screen's tick with nothing of its own open (no conversation, shop, board or page; its popups are screens over
/// it, so it does not tick under them): a running sequence takes over (the story stage), else the oldest queued
/// vignette plays (W2). CAMPAIGN1 T2: the campaign queue's leading vignettes join the story queue first
/// (`story::pull_pending`); once the vignettes played, a queued battle starts (`runover::play_pending_fight`; a queued
/// dialogue is the region map's). True when the map handed over.
pub fn map_tick(cx: &mut Cx) -> bool {
    if !cx.has_game() || cx.busy() || !cx.nav.is_empty() {
        return false;
    }
    if story::current(&cx.data, cx.gs()).is_some() {
        let stage = StoryStage::new(cx);
        cx.nav.push(Nav::Reset(Box::new(stage)));
        return true;
    }
    let d = cx.data.clone();
    story::pull_pending(&d, cx.gs_mut());
    play_queued(cx).is_some() || crate::runover::play_pending_fight(cx)
}

/// Logs the notes of a story result (effects, "Try again.") and takes its campaign events (CAMPAIGN1 W1).
fn log_notes(cx: &mut Cx, res: &Dict) {
    for n in json::sget(res, "notes") {
        cx.log(n);
    }
    crate::runover::take(cx, res);
}

#[derive(Debug, Clone, Copy)]
pub enum StAct {
    MainMenu,
}

/// The story stage (5.3, 5.4): plays the running sequence's steps one by one. Whatever it starts (a vignette, a fight)
/// is a screen over it; when that screen goes the stage goes on with the next step (on its next tick, so a results
/// popup or a level-up screen opened after a fight comes first).
pub struct StoryStage {
    /// A screen the stage pushed is over it (cleared when the stage is the top again).
    waiting: bool,
    /// The music under the cards: the menu theme until a vignette names a track or silence (V8: "" keeps it playing).
    music: Option<String>,
    /// Why a step could not run (shown with a way back to the main menu).
    error: Vec<String>,
    w: Vec<Widget<StAct>>,
}

impl StoryStage {
    pub fn new(cx: &Cx) -> Self {
        Self { waiting: false, music: crate::music::game_track(cx, crate::music::MENU), error: Vec::new(), w: Vec::new() }
    }

    /// The next beat: a queued vignette first (Q9), then the step at `running.step`.
    fn run(&mut self, cx: &mut Cx) {
        if self.waiting || !self.error.is_empty() || !cx.has_game() || !cx.nav.is_empty() {
            return;
        }
        // CAMPAIGN1 W1, F3: campaign notices between the beats; the run-over screen once the run is over
        if crate::runover::map_tick(cx) {
            return;
        }
        if let Some(wish) = play_queued(cx) {
            self.wait_on(wish);
            return;
        }
        let d = cx.data.clone();
        match story::current(&d, cx.gs()) {
            // nothing runs (an aborted fight ended it): the map
            None if !story::is_running(cx.gs()) => {
                let map = crate::region::RegionMap::new(cx);
                cx.nav.push(Nav::Reset(Box::new(map)));
            }
            None => self.error = vec![format!("The story cannot play step {} of '{}'.", cx.gs().story.running.as_ref().map_or(0, |r| r.step) + 1, cx.gs().story.running.as_ref().map(|r| r.sequence.clone()).unwrap_or_default())],
            Some(Step::Vignette { vignette }) => {
                let scene = story::scene(&d, cx.gs(), &vignette);
                if json::aget(&scene, "cards").is_empty() {
                    // every card left out by `when` (or an unknown ref): the step is done at once (W4)
                    let res = story::step_done(&d, cx.gs_mut());
                    log_notes(cx, &res);
                    return;
                }
                let screen = scene_screen(cx, &scene, |cx: &mut Cx, _| {
                    let d = cx.data.clone();
                    let res = story::step_done(&d, cx.gs_mut());
                    if !json::b(&res, "ok", false) {
                        show_errors(cx, json::sget(&res, "errors"));
                    }
                    log_notes(cx, &res);
                    cx.nav.push(Nav::Pop);
                });
                let wish = screen.player.reel.music_wish();
                cx.nav.push(Nav::Push(Box::new(screen)));
                self.wait_on(wish);
            }
            Some(Step::Fight { .. }) => {
                let Some(fight) = story::fight(&d, cx.gs()) else { return };
                let name = s_or(d.resolve(&cx.gs().world_id, "encounters", &s(&fight, "encounter")), "name", "Enemies");
                match crate::region::try_fight(cx, &fight, story::FIGHT_SOURCE) {
                    Ok(()) => {
                        cx.log(format!("Fight: {name}!"));
                        self.waiting = true;
                    }
                    Err(e) => self.error = vec![e],
                }
            }
            Some(Step::Enter { .. }) | Some(Step::End { .. }) => self.finish(cx),
            // Q2: `current` never gives an effects step
            Some(Step::Effects { .. }) => self.error = vec!["The story stopped at an effects step.".into()],
        }
    }

    /// Something was pushed: wait for it; a vignette's music wish becomes the stage's (V8).
    fn wait_on(&mut self, wish: Option<String>) {
        self.waiting = true;
        if wish.is_some() {
            self.music = wish;
        }
    }

    /// Q6, Q7: the `enter` or `end` step: the region map (an `enter` autosaves through its arrival's `autosave` event),
    /// or the main menu after the autosave with the game in play ended.
    fn finish(&mut self, cx: &mut Cx) {
        let d = cx.data.clone();
        let res = story::finish(&d, cx.gs_mut());
        if !json::b(&res, "ok", false) {
            self.error = json::sget(&res, "errors");
            return;
        }
        crate::region::log_events(cx, json::aget(&res, "events"));
        crate::runover::take(cx, &res);
        if s(&res, "to") == "menu" {
            cx.autosave();
            cx.end_game();
            cx.nav.push(Nav::Reset(Box::new(crate::menus::MainMenu::default())));
        } else {
            let map = crate::region::RegionMap::new(cx);
            cx.nav.push(Nav::Reset(Box::new(map)));
        }
    }

    fn act(&mut self, cx: &mut Cx, a: StAct) {
        match a {
            StAct::MainMenu => {
                cx.end_game();
                cx.nav.push(Nav::Reset(Box::new(crate::menus::MainMenu::default())));
            }
        }
    }
}

impl Screen<GameCx> for StoryStage {
    fn id(&self) -> &'static str {
        "story_stage"
    }

    fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<StAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().stage);
        if !self.error.is_empty() {
            let r = Rect::new(120, 100, 400, 160);
            ui.frame(r, "The story cannot go on", false);
            let mut y = r.y + 30;
            for e in &self.error {
                y = ui.para(r.x + 10, y, r.w - 20, e, BODY, th().text) + 4;
            }
            let bw = (ui.p.text_w("Main menu", BODY) + 16.0).ceil() as i32;
            ui.button("story.main_menu", Rect::new(r.x + r.w - bw - 10, r.y + r.h - 26, bw, 16), "Main menu", StAct::MainMenu, true);
        }
        self.w = ui.widgets;
    }

    /// No game menu while a sequence runs (Q8); Esc and right click do nothing here (a card's Skip is its own).
    fn back(&mut self, _cx: &mut Cx) {}

    fn tick(&mut self, cx: &mut Cx) {
        self.run(cx);
    }

    fn resume(&mut self, _cx: &mut Cx) {
        self.waiting = false;
    }

    fn state(&self, cx: &Cx) -> Value {
        let running = cx.game.as_ref().and_then(|g| g.gs.story.running.clone());
        let step = cx.game.as_ref().and_then(|g| story::current(&cx.data, &g.gs)).map(|st| match st {
            Step::Vignette { .. } => "vignette",
            Step::Fight { .. } => "fight",
            Step::Effects { .. } => "effects",
            Step::Enter { .. } => "enter",
            Step::End { .. } => "end",
        });
        json!({"sequence": running.as_ref().map(|r| r.sequence.clone()), "step": running.as_ref().map(|r| r.step), "kind": step, "waiting": self.waiting, "error": self.error})
    }

    fn music(&self, _cx: &Cx) -> Option<String> {
        self.music.clone()
    }

    widget_plumbing!();
}

/// The gallery's reels: the engine's sample, then every world's vignettes as written (V15).
pub fn gallery_reels(d: &Data) -> Vec<(String, Reel)> {
    let mut out = vec![("Sample reel".to_string(), vignette::sample_reel())];
    for w in d.world_ids() {
        let world = s_or(d.manifest(&w), "name", &w);
        for e in d.own_entries(&w, "vignettes") {
            out.push((format!("{world}: {}", s_or(e, "name", &s(e, "id"))), entry_reel(d, &w, e)));
        }
    }
    out
}

/// STORYUI1-U1 tests (STORYUI1_TASKS 2: TU1, TU2; TU3 is the gallery's) on a scratch world `tale` (a start sequence
/// with vignettes, a story fight that retries on a loss and an `enter` at the end; a person whose choices start a
/// sequence, queue a vignette and open a board) and world `classic` (none of it). A debug game, so fights settle
/// through the debug fight popup with the battle screen's runner (`battle::after_fight`). Expectations follow from the
/// rules and the fixture's own data.
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::hud::hud_tests::{frame, gfx, rect};
    use crate::screens::{default_settings, enter_game, Key};
    use aegis_ui::screen::{Settings, TEXT_SIZES};
    use emberwake_world::{new_game, save};
    use std::sync::Arc;

    fn with_ids(w: &str, rows: Value) -> Value {
        let rows = json::arr(&rows)
            .iter()
            .map(|r| {
                let mut r = json::dict(r).clone();
                let id = s(&r, "id");
                r.insert("_world".into(), w.into());
                r.insert("_qid".into(), format!("{w}:{id}").into());
                Value::Object(r)
            })
            .collect();
        Value::Array(rows)
    }

    fn text_card(text: &str) -> Value {
        json!({"style": "text", "text": text})
    }

    pub(crate) fn data() -> Data {
        let vignettes = json!([
            {"id": "v_intro", "name": "Intro", "cards": [text_card("The bells."), {"style": "title", "title": "Act 1", "subtitle": "The start"}]},
            {"id": "v_line", "name": "Line", "music": "none", "cards": [{"style": "line", "speaker": "clerk", "text": "{flag:began} began."}]},
            {"id": "v_news", "name": "News", "cards": [text_card("News.")]}
        ]);
        let sequences = json!([
            {"id": "prologue", "name": "Prologue", "steps": [
                {"kind": "effects", "effects": [{"effect": "set_flag", "key": "began", "value": "it"}]},
                {"kind": "vignette", "vignette": "v_intro"},
                {"kind": "fight", "encounter": "fight_a", "on_lost": "retry"},
                {"kind": "vignette", "vignette": "v_line"},
                {"kind": "enter", "location": "alley", "anchor": ""}
            ]},
            {"id": "side_story", "name": "Side", "steps": [{"kind": "vignette", "vignette": "v_news"}, {"kind": "end", "to": "menu"}]}
        ]);
        let dialogues = json!([{"id": "talk", "start": "a", "services": false, "nodes": [
            {"id": "a", "text": "Well?", "choices": [
                {"text": "Tell me a story.", "effects": [{"effect": "sequence", "sequence": "side_story"}]},
                {"text": "Show me.", "to": "a", "effects": [{"effect": "vignette", "vignette": "v_news"}]},
                {"text": "The board.", "to": "a", "effects": [{"effect": "open_board", "board": "b"}]}
            ]}
        ]}]);
        let worlds = json!({
            "tale": {"manifest": {"id": "tale", "name": "Tale", "start_location": "hub", "start_party": ["hero"], "start_sequence": "prologue"}, "own": {
                "locations": with_ids("tale", json!([{"id": "hub", "name": "Hub"}, {"id": "alley", "name": "Alley"}])),
                "characters": with_ids("tale", json!([{"id": "clerk", "name": "Clerk", "role": "npc", "dialogue": "talk"}])),
                "dialogues": with_ids("tale", dialogues),
                "encounters": with_ids("tale", json!([{"id": "fight_a", "name": "Fight A", "enemies": []}])),
                "boards": with_ids("tale", json!([{"id": "b", "name": "Board", "keeper": "clerk"}])),
                "vignettes": with_ids("tale", vignettes),
                "sequences": with_ids("tale", sequences)
            }},
            "classic": {"manifest": {"id": "classic", "name": "Classic", "start_location": "camp", "start_party": ["hero"]}, "own": {
                "locations": with_ids("classic", json!([{"id": "camp", "name": "Camp"}]))
            }}
        });
        let catalog = json!({
            "classes": [{"id": "fighter", "name": "Fighter"}],
            "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
        });
        Data::from_values(&catalog, &worlds, &json!({}))
    }

    /// A debug context over `d` with no game yet (nothing is autosaved; settings and saves in a scratch folder).
    pub(crate) fn story_cx(d: Data) -> Cx {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let tmp = std::env::temp_dir().join(format!("emberwake_story_{}_{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let app = GameCx {
            data: Arc::new(d),
            manifest: Default::default(),
            game: None,
            save_dir: tmp.join("saves"),
            tuning: aegis_ui::tuning::TuningFile::new(None),
            log: Vec::new(),
            encounter: String::new(),
            seed: None,
            debug: true,
            data_dir: tmp.clone(),
        };
        Cx {
            settings: Settings::load(tmp.join("settings.json"), default_settings()),
            art: aegis_ui::art::Art::new(None),
            audio: aegis_ui::audio::Audio::offline(None),
            nav: Vec::new(),
            notice: Vec::new(),
            shot_dir: tmp,
            quit: false,
            answers: Vec::new(),
            playtest: aegis_ui::playtest::Playtests::off(),
            held: Default::default(),
            app,
        }
    }

    /// The screen the last navigation opened (Push or Reset), and how.
    fn take(cx: &mut Cx) -> (&'static str, Box<dyn Screen<GameCx>>) {
        match cx.nav.pop() {
            Some(Nav::Push(s)) => ("push", s),
            Some(Nav::Reset(s)) => ("reset", s),
            Some(Nav::Pop) => panic!("a pop, not a screen"),
            None => panic!("nothing was opened"),
        }
    }

    fn step_of(cx: &Cx) -> Option<i64> {
        cx.gs().story.running.as_ref().map(|r| r.step)
    }

    /// Presses Space on a vignette screen until its reel ends (each press moves one phase, V3); the reel's `on_done`
    /// queues the pop, which is taken off.
    fn play_out(cx: &mut Cx, v: &mut Box<dyn Screen<GameCx>>) {
        for _ in 0..200 {
            if cx.nav.iter().any(|n| matches!(n, Nav::Pop)) {
                cx.nav.retain(|n| !matches!(n, Nav::Pop));
                return;
            }
            v.key(cx, Key::Space);
        }
        panic!("the reel did not end");
    }

    /// TU1 (5.4, Q2-Q7): a new game in a world with a start sequence opens on the story stage; the stage plays the
    /// vignette step, then starts the story fight (runner `story`); a loss with `retry` keeps the step and logs
    /// "Try again."; a win moves on; the `enter` step leaves for the region map at the location with `running` cleared.
    #[test]
    fn a_new_game_opens_on_the_story_stage_which_plays_the_steps() {
        let mut cx = story_cx(data());
        let mut g = gfx();
        let gs = new_game::create_for_world(&cx.data.clone(), "tale", 1).expect("a new game");
        assert_eq!(gs.story.running.as_ref().map(|r| r.step), Some(1), "the leading effects step ran at the start (Q1, Q3)");
        enter_game(&mut cx, gs, -1, Vec::new());
        let (how, mut stage) = take(&mut cx);
        assert_eq!((how, stage.id()), ("reset", "story_stage"));
        assert!(cx.nav.is_empty());

        // the vignette step: the player over the stage, Skip asks first for an unseen scene (W6)
        stage.tick(&mut cx);
        let (how, mut v) = take(&mut cx);
        assert_eq!((how, v.id()), ("push", "vignette"));
        let st = v.state(&cx);
        assert_eq!((st["reel"].clone(), st["cards"].clone(), st["ask_first"].clone()), (json!("tale:v_intro"), json!(2), json!(true)));
        stage.tick(&mut cx);
        assert!(cx.nav.is_empty(), "the stage waits while the vignette plays");
        play_out(&mut cx, &mut v);
        assert_eq!(step_of(&cx), Some(2), "Q4: the vignette's end moves the story to the fight");
        assert!(cx.gs().story.seen.contains(&"tale:v_intro".to_string()));

        // the fight step: a world fight with runner `story` (the debug popup in a debug game)
        stage.resume(&mut cx);
        stage.tick(&mut cx);
        let (_, mut fight) = take(&mut cx);
        assert_eq!(fight.id(), "debug_fight");
        assert_eq!((fight.state(&cx)["source"].clone(), fight.state(&cx)["encounter"].clone()), (json!(story::FIGHT_SOURCE), json!("tale:fight_a")));
        fight.draw(&mut cx, &mut g, (-500, -500), true);
        fight.click_id(&mut cx, "debug_fight.lost").expect("Lost");
        assert_eq!(step_of(&cx), Some(2), "Q5: `retry` keeps the step");
        assert!(cx.log.iter().any(|l| l == story::RETRY_TEXT), "the log says {:?}", story::RETRY_TEXT);
        let (_, results) = take(&mut cx);
        assert_eq!(results.id(), "battle_results");
        cx.nav.clear();
        stage.resume(&mut cx);
        stage.tick(&mut cx);
        let (_, mut fight) = take(&mut cx);
        assert_eq!(fight.id(), "debug_fight", "the fight is tried again");
        fight.draw(&mut cx, &mut g, (-500, -500), true);
        fight.click_id(&mut cx, "debug_fight.won").expect("Skip");
        assert_eq!(step_of(&cx), Some(3), "Q5: a won story fight moves on");
        cx.nav.clear();

        // the line card, then the `enter` step: the region map at the location, the story over
        stage.resume(&mut cx);
        stage.tick(&mut cx);
        let (_, mut v) = take(&mut cx);
        assert_eq!(v.state(&cx)["reel"], json!("tale:v_line"));
        assert_eq!(stage.music(&cx), Some(String::new()), "V8: the vignette's `none` is silence, kept under the stage");
        play_out(&mut cx, &mut v);
        stage.resume(&mut cx);
        stage.tick(&mut cx);
        let (how, map) = take(&mut cx);
        assert_eq!((how, map.id()), ("reset", "region_map"));
        assert!(!story::is_running(cx.gs()), "Q6: `running` cleared");
        assert_eq!(cx.gs().location_id, "tale:alley");
    }

    /// 5.4, Q8: a world without a start sequence enters the region map as before; a save made during a sequence loads
    /// onto the stage.
    #[test]
    fn a_classic_world_enters_the_map_and_a_story_save_the_stage() {
        let mut cx = story_cx(data());
        let d = cx.data.clone();
        let gs = new_game::create_for_world(&d, "classic", 1).expect("a new game");
        enter_game(&mut cx, gs, -1, Vec::new());
        assert_eq!(take(&mut cx).1.id(), "region_map");
        let gs = new_game::create_for_world(&d, "tale", 1).expect("a new game");
        assert!(save::write(&d, &cx.save_dir, &gs, 2, 0).is_empty());
        let loaded = save::read(&d, &cx.save_dir, 2).expect("the save loads");
        assert!(story::is_running(&loaded.state));
        enter_game(&mut cx, loaded.state, 2, loaded.warnings);
        assert_eq!(take(&mut cx).1.id(), "story_stage");
        let _ = std::fs::remove_dir_all(&cx.shot_dir);
    }

    /// TU2 (W2): a queued vignette plays when the region map ticks with nothing open, not while a conversation is
    /// open; its end takes it off the queue and marks it seen.
    #[test]
    fn a_queued_vignette_plays_on_the_map_with_nothing_open() {
        let mut cx = story_cx(data());
        let mut gs = new_game::create_for_world(&cx.data.clone(), "tale", 1).expect("a new game");
        gs.story = Default::default();
        enter_game(&mut cx, gs, -1, Vec::new());
        let (_, mut map) = take(&mut cx);
        map.tick(&mut cx);
        assert!(cx.nav.is_empty(), "nothing queued, nothing plays");
        let d = cx.data.clone();
        assert!(story::queue(&d, cx.gs_mut(), "v_news"));
        let mut talk = crate::region::RegionMap::new(&mut cx);
        talk.talk(&mut cx, "clerk");
        assert!(!talk.state(&cx)["dialogue"].is_null(), "a conversation is open");
        talk.tick(&mut cx);
        assert!(cx.nav.is_empty(), "W2: not while a conversation is open");
        talk.back(&mut cx);
        assert!(talk.state(&cx)["dialogue"].is_null(), "the conversation closed");
        talk.tick(&mut cx);
        let (how, mut v) = take(&mut cx);
        assert_eq!((how, v.id(), v.state(&cx)["reel"].clone()), ("push", "vignette", json!("tale:v_news")));
        play_out(&mut cx, &mut v);
        assert!(cx.gs().story.queue.is_empty() && cx.gs().story.seen == ["tale:v_news"], "W2: off the queue, seen");
        talk.tick(&mut cx);
        assert!(cx.nav.is_empty());
    }

    /// STORYUI1-W gaps: a `board` choice holds the replies (no "Close the board first." dead end; STORYUI1-U2: its pane
    /// stands in the dialogue box's place) until Esc closes the
    /// board back to the conversation; a `sequence` choice ends the talk and the map hands over to the stage, which
    /// shows its scenes and, at `end` `menu`, ends the game in play (Q7).
    #[test]
    fn an_open_board_closes_back_to_the_talk_and_a_sequence_choice_shows_its_scenes() {
        let mut cx = story_cx(data());
        let mut g = gfx();
        let mut gs = new_game::create_for_world(&cx.data.clone(), "tale", 1).expect("a new game");
        gs.story = Default::default();
        enter_game(&mut cx, gs, -1, Vec::new());
        cx.nav.clear();
        let mut talk = crate::region::RegionMap::new(&mut cx);
        talk.talk(&mut cx, "clerk");
        frame(&mut talk, &mut cx, &mut g, 1.0);
        talk.click_id(&mut cx, "choice.2").expect("the board reply");
        assert_eq!(talk.state(&cx)["board"], json!("tale:b"));
        frame(&mut talk, &mut cx, &mut g, 1.0);
        // STORYUI1-U2: the board pane takes the dialogue box's place (the shop's rule), so no reply is there to click
        let replies: Vec<Value> = talk.widgets().into_iter().filter(|w| w["kind"] == "choice" && w["enabled"] == json!(true)).collect();
        assert!(replies.is_empty() && talk.widgets().iter().any(|w| w["id"] == "board"), "the replies wait while the board pane is open");
        talk.back(&mut cx);
        let st = talk.state(&cx);
        assert!(st["board"].is_null() && !st["dialogue"].is_null(), "Esc closes the board back to the conversation");
        frame(&mut talk, &mut cx, &mut g, 1.0);
        assert!(talk.widgets().iter().any(|w| w["kind"] == "choice" && w["enabled"] == json!(true)));

        talk.click_id(&mut cx, "choice.0").expect("the story reply");
        assert!(talk.state(&cx)["dialogue"].is_null(), "the conversation closed");
        assert_eq!(cx.gs().story.running.as_ref().map(|r| r.sequence.clone()), Some("tale:side_story".to_string()));
        assert!(!cx.busy());
        talk.tick(&mut cx);
        let (how, mut stage) = take(&mut cx);
        assert_eq!((how, stage.id()), ("reset", "story_stage"));
        stage.tick(&mut cx);
        let (_, mut v) = take(&mut cx);
        assert_eq!(v.state(&cx)["reel"], json!("tale:v_news"));
        play_out(&mut cx, &mut v);
        stage.resume(&mut cx);
        stage.tick(&mut cx);
        let (how, menu) = take(&mut cx);
        assert_eq!((how, menu.id()), ("reset", "main_menu"), "Q7: `end` `menu` ends the game in play");
        assert!(!cx.has_game());
    }

    /// W3, W5, V8: a scene's cards play as the kit resolved them (tokens filled, a line card's speaker named) with the
    /// reel's music; the gallery's entry reel shows a card as written.
    #[test]
    fn a_scene_becomes_the_engines_reel() {
        let d = data();
        let gs = new_game::create_for_world(&d, "tale", 1).expect("a new game");
        let sc = story::scene(&d, &gs, "v_line");
        let reel = reel_of(&d, &gs.world_id, &sc);
        assert_eq!((reel.id.as_str(), reel.music_wish(), reel.cards.len()), ("tale:v_line", Some(String::new()), 1));
        let c = &reel.cards[0];
        assert_eq!((c.style, c.speaker.as_str(), c.text.as_str()), (vignette::Style::Line, "Clerk", "it began."));
        let raw = d.own_entries("tale", "vignettes").iter().find(|e| s(e, "id") == "v_line").expect("v_line");
        assert_eq!(entry_reel(&d, "tale", raw).cards[0].text, "{flag:began} began.");
        let names: Vec<String> = gallery_reels(&d).into_iter().map(|r| r.0).collect();
        assert_eq!(names, ["Sample reel", "Tale: Intro", "Tale: Line", "Tale: News"], "V15: the sample, then every world's vignettes");
    }

    /// PORTRAITFX1-X X3: a `line` card's portrait chain is the kit's card resolution for its `expression`
    /// (`story::card_portrait`: the speaker's frame first, its neutral frame, then today's chain), the card keeps the
    /// expression for the engine, and the game's vignette screens take an expression frame's anchors from its base
    /// portrait's sidecar when the frame has none (the kit's `face_sidecars`).
    #[test]
    fn a_line_cards_expression_resolves_its_portrait_chain() {
        let d = data();
        let raw = json::dict(&json!({"style": "line", "speaker": "clerk", "expression": "angry", "text": "Out!"})).clone();
        let c = card_of(&d, "tale", &raw);
        assert_eq!(c.expression, "angry");
        assert_eq!(c.portrait, story::card_portrait(&d, "tale", &raw));
        let clerk = d.resolve("tale", "characters", "clerk");
        let today = crate::screens::character_art(&d, clerk).2;
        assert_eq!(c.portrait[..2], ["portraits/npcs/faces/clerk/angry.png", "portraits/npcs/faces/clerk/neutral.png"]);
        assert_eq!(c.portrait[2..], today[..], "then today's chain: a missing frame falls back to the single portrait");
        let plain = card_of(&d, "tale", &json::dict(&json!({"style": "line", "speaker": "clerk", "text": "Hi."})).clone());
        assert_eq!((plain.expression.as_str(), plain.portrait[0].as_str()), ("", "portraits/npcs/faces/clerk/neutral.png"));
        let cx = story_cx(data());
        let s = vignette_screen(&cx, Reel::default(), |_, _| {});
        let rule = s.portrait.sidecars.expect("the game's sidecar rule");
        assert_eq!(rule("portraits/npcs/faces/clerk/angry.png"), emberwake_content::art::face_sidecars("portraits/npcs/faces/clerk/angry.png"));
    }

    /// 4.8: the settings' gameplay tab sets the text speed and reduce motion (the engine's keys), inside the tab body at
    /// every text size.
    #[test]
    fn the_gameplay_tab_sets_text_speed_and_reduce_motion() {
        let mut cx = story_cx(data());
        let mut g = gfx();
        let mut s = crate::menus::SettingsScreen::new(&cx);
        s.draw(&mut cx, &mut g, (-500, -500), true);
        s.click_id(&mut cx, "settings.tab.gameplay").expect("the gameplay tab");
        for &(_, scale) in TEXT_SIZES.iter() {
            frame(&mut s, &mut cx, &mut g, scale as f32);
            let ws = s.widgets();
            for id in ["settings.text_speed", "settings.reduce_motion"] {
                let w = ws.iter().find(|w| w["id"] == id).unwrap_or_else(|| panic!("{id}"));
                let (_, y, _, h) = rect(w);
                assert!(y + h <= 25 + 55 + 222, "{id} inside the tab body at text size {scale}");
            }
        }
        assert_eq!(s.state(&cx)["draft"][vignette::TEXT_SPEED], json!("normal"), "the default speed");
        s.click_id(&mut cx, "settings.reduce_motion").expect("reduce motion");
        assert_eq!(s.state(&cx)["draft"][vignette::REDUCE_MOTION], json!(true));
        s.click_id(&mut cx, "settings.apply").expect("apply");
        assert!(cx.settings.reduce_motion());
        let _ = std::fs::remove_dir_all(&cx.shot_dir);
    }

    /// FX1-V V4 (FX1 5.2): the game's vignette screens play the cards' shakes with its tuning table and limits: a tuned
    /// `toll` of 1 px moves the card by at most 1 px (and does move it: at its start the envelope is full).
    #[test]
    fn vignette_screens_shake_with_the_tuning_table() {
        use aegis_ui::tuning::Tuned;
        let mut cx = story_cx(data());
        cx.tuning.value = crate::tuning::Tuning::from_json(&json!({"shakes": {"toll": {"px": 1}}}));
        let reel = Reel { id: "t".into(), cards: vec![Card { shake: "toll".into(), ..Card::default() }], ..Reel::default() };
        let mut s = vignette_screen(&cx, reel, |_, _| {});
        assert_eq!((&s.player.shakes, s.player.shake_limits), (&cx.tuning.value.shakes, cx.tuning.value.shake_limits()), "the tuning's table and limits");
        let toll = s.player.shakes.get("toll").expect("toll");
        assert_eq!(toll.px, 1.0);
        let mut seen = Vec::new();
        while s.player.card_ms() <= toll.ms as u64 {
            seen.push(s.player.shake_offset(false));
            s.player.advance(10);
        }
        assert!(seen.iter().all(|o| o.0.abs() <= 1 && o.1 == 0), "at most 1 px, along x: {seen:?}");
        assert!(seen.iter().any(|o| *o != (0, 0)), "the card shakes");
        let _ = std::fs::remove_dir_all(&cx.shot_dir);
    }
}
