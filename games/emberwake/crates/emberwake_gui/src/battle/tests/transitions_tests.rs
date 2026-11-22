//! TRANSITIONS1 (`PR7-14`, games/emberwake/docs/specs/TRANSITIONS1.md) on a scratch world built here (no shipped data):
//! a story whose sequence is a vignette, a fight (`t_fight`: Ash against one Grunt of 1 HP, so autoplay wins) and either
//! the `enter` step (`to_hub`) or another vignette (`go_on`). Expectations come from the spec: T1, a story fight opens
//! with the battle beat (the ash cover, the banner naming the fight, its objective and enemy count, the party walking
//! in), which holds the battle until control starts and which any press skips without acting; Reduce motion makes it a
//! plain fade with no walk-in; a fight met on the map has none. T2, a won fight plays the victory beat (the FX clock in
//! slow motion, the Victory banner in the outcome panel's place), then the results over the dimmed battle; Continue fades
//! out and closes both, and the hub opens on the arrival card, unless the story goes on to a vignette. The beats' texts
//! pass the text audit at every text size.

use crate::battle::{Action, BattleScreen};
use crate::hud::hud_tests::{frame, game_cx, gfx, rect};
use crate::screens::{Cx, GameCx, Key, Nav, Screen};
use crate::story::StoryStage;
use crate::transitions::{self as tr, Kind, ASH};
use aegis_core::clock::{Duration, Instant};
use aegis_core::json::{self, s};
use aegis_ui::screen::TEXT_SIZES;
use aegis_ui::ui::{Gfx, H, W};
use emberwake_world::Data;
use serde_json::{json, Value};
use std::sync::Arc;

const WORLD: &str = "beatw";

fn with_ids(rows: Value) -> Value {
    Value::Array(
        json::arr(&rows)
            .iter()
            .map(|r| {
                let mut r = json::dict(r).clone();
                let id = s(&r, "id");
                r.insert("_world".into(), WORLD.into());
                r.insert("_qid".into(), format!("{WORLD}:{id}").into());
                Value::Object(r)
            })
            .collect(),
    )
}

/// The scratch world; `go_on`: the story goes on to a vignette after the fight (else it enters the hub).
fn data(go_on: bool) -> Data {
    let skills = json!([
        {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
        {"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3},
        {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
        {"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true},
        {"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
        {"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
        {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}
    ]);
    let hero = json!({"id": "t_hero", "name": "t_hero", "side": "party", "class_id": "drill", "level": 1});
    let foe = json!({"id": "t_foe", "name": "t_foe", "side": "enemy", "stat_mode": "fixed", "hp": 1, "speed": 1, "attack_range": 1, "skill_cap": 5});
    let class = json!({"id": "drill", "name": "Drill", "base_stats": {"STR": 30, "DEX": 30, "INT": 6, "CONS": 30, "PERC": 30, "KNOW": 10}});
    let ruleset = json!({"id": "timeline", "name": "Timeline", "rules": ["round_intent", "ap_cost"], "basics": ["move", "rest", "strike", "guard"]});
    let catalog = json!({"skills": skills, "classes": [class], "units": [hero, foe], "rulesets": [ruleset]});
    let fight = json!({"id": "t_fight", "name": "The Grate", "rows": vec![".".repeat(12); 9], "party": ["ash"], "enemies": ["grunt"], "total_blocks": 6, "unit_ap": 10, "max_rounds": 30});
    let after = if go_on { json!({"kind": "vignette", "vignette": "v_after"}) } else { json!({"kind": "enter", "location": "yard", "anchor": ""}) };
    let mut steps = vec![json!({"kind": "vignette", "vignette": "v_before"}), json!({"kind": "fight", "encounter": "t_fight", "on_lost": "retry"}), after];
    if go_on {
        steps.push(json!({"kind": "enter", "location": "yard", "anchor": ""}));
    }
    let worlds = json!({WORLD: {"manifest": {"id": WORLD, "name": "Beat", "start_location": "yard", "start_party": ["ash"], "start_sequence": "story"}, "own": {
        "locations": with_ids(json!([{"id": "yard", "name": "The Yard"}])),
        "characters": with_ids(json!([{"id": "ash", "name": "Ash", "role": "party", "base_unit": "t_hero"}, {"id": "grunt", "name": "Grunt", "role": "enemy", "base_unit": "t_foe"}])),
        "encounters": with_ids(json!([fight])),
        "vignettes": with_ids(json!([
            {"id": "v_before", "name": "Before", "cards": [{"style": "text", "text": "The grate gives."}]},
            {"id": "v_after", "name": "After", "cards": [{"style": "text", "text": "Quiet again."}]}
        ])),
        "sequences": with_ids(json!([{"id": "story", "name": "Story", "steps": steps}]))
    }}});
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// A game of the scratch world, not a debug one (its fights open the battle screen), on the flat board.
pub(super) fn beat_cx(go_on: bool, reduce: bool) -> Cx {
    let mut cx = game_cx();
    let d = Arc::new(data(go_on));
    let gs = emberwake_world::new_game::create_for_world(&d, WORLD, 1).expect("the scratch world starts a game");
    cx.data = d;
    cx.game = Some(crate::screens::Game::new(gs, -1, true));
    cx.debug = false;
    cx.settings.v["battle_view"] = json!("flat");
    cx.settings.v["reduce_motion"] = json!(reduce);
    cx
}

fn fight() -> json::Dict {
    json::dict(&json!({"encounter": "t_fight"})).clone()
}

/// The world fight of runner `runner`.
pub(super) fn battle(cx: &mut Cx, runner: &str) -> BattleScreen {
    let b = BattleScreen::world(cx, &fight(), runner).expect("the world fight starts");
    cx.nav.clear();
    b
}

/// The beat started `ms` ago.
fn at_ms(b: &mut BattleScreen, ms: u64) {
    if let Some(beat) = b.beat.as_mut() {
        beat.started = Instant::now() - Duration::from_millis(ms);
    }
}

fn widget(screen: &dyn Screen<GameCx>, id: &str) -> Option<Value> {
    screen.widgets().into_iter().find(|w| w["id"] == id)
}

/// The battle wins at once (the host's autoplay), as the agent link would.
fn win(b: &mut BattleScreen, cx: &mut Cx) {
    let reply = b.command(cx, &json!({"cmd": "autoplay"})).expect("the battle answers");
    assert_eq!(reply["ok"], json!(true), "{reply}");
    assert_eq!(b.outcome(), "won", "Ash beats a Grunt of 1 HP");
}

/// The screen the last navigation pushed.
fn pushed(cx: &mut Cx) -> Box<dyn Screen<GameCx>> {
    match cx.nav.pop() {
        Some(Nav::Push(s)) | Some(Nav::Reset(s)) => s,
        _ => panic!("nothing was pushed"),
    }
}

fn pixel(g: &Gfx, x: i32, y: i32) -> [u8; 4] {
    let i = ((y * g.canvas.w + x) * 4) as usize;
    [g.canvas.px[i], g.canvas.px[i + 1], g.canvas.px[i + 2], g.canvas.px[i + 3]]
}

/// T1: a story fight opens with the battle beat: its banner reads the fight (the encounter's name, the objective line,
/// the enemies on the field), the ash covers the stage at its start and a full-screen widget lies over everything; the
/// party's walk-in waits off its cells; the beat holds the battle until it ends by itself.
#[test]
fn a_story_fight_opens_with_the_battle_beat_until_control_starts() {
    let (mut cx, mut g) = (beat_cx(false, false), gfx());
    let mut b = battle(&mut cx, emberwake_world::story::FIGHT_SOURCE);
    let beat = b.beat.expect("the battle beat runs");
    assert_eq!((beat.kind, beat.plain, beat.length()), (Kind::Intro, false, tr::INTRO_MS));
    let foes = b.session().w.units.len() - b.session().w.allies.len();
    let st = b.state(&cx);
    assert_eq!(st["beat"]["banner_text"], json!({"name": "The Grate", "objective": b.intro_card.objective, "foes": if foes == 1 { "1 enemy".to_string() } else { format!("{foes} enemies") }}));
    assert!(!b.intro_card.objective.is_empty(), "the objective line");
    assert!(!b.at_rest(), "a beat keeps the frames coming");
    // the first tick: the tokens catch up and every living ally walks in from three cells toward the nearer side edge
    b.tick(&mut cx);
    for &i in &b.session().w.allies {
        let to = b.session().w.grid.pos(i).expect("placed");
        let (from, t0) = b.tokens.anims[i].slide.expect("the ally walks in");
        assert_eq!((from.y, (from.x - to.x).abs()), (to.y, tr::WALK_CELLS), "from the edge side, same row");
        assert!(t0 >= b.clock.now(), "it waits off its cell until the wipe has opened the stage");
        assert_eq!(b.tokens.anims[i].slide_ms(cx.tuning.value.walk_ms), tr::WALK_MS);
    }
    // its first frame: the stage under the ash, the click widget last and over all of the screen
    at_ms(&mut b, 0);
    frame(&mut b, &mut cx, &mut g, 1.0);
    let ash = aegis_ui::theme::th().game("ash_cover", ASH);
    assert_eq!(pixel(&g, W / 2, H / 2)[..3], ash[..3], "the ash covers the stage");
    let last = b.widgets().last().cloned().expect("widgets");
    assert_eq!((last["id"].clone(), rect(&last)), (json!("battle.beat"), (0, 0, W as i64, H as i64)));
    // the banner in place, then the end: control starts, the widget goes
    at_ms(&mut b, tr::BANNER_IN_MS + tr::BANNER_SLIDE_MS);
    assert_eq!(b.state(&cx)["beat"]["banner"], json!(1.0));
    let sent = b.host.history.len();
    b.tick(&mut cx);
    assert!(b.beat.is_some() && b.host.history.len() == sent, "the battle waits");
    at_ms(&mut b, tr::INTRO_MS);
    b.tick(&mut cx);
    assert!(b.beat.is_none() && b.state(&cx).get("beat").is_none());
    frame(&mut b, &mut cx, &mut g, 1.0);
    assert!(widget(&b, "battle.beat").is_none());
}

/// T1: a press during the beat skips it (the walk-in lands at once) and does nothing else: Confirm sends no request;
/// a key, Esc, right click and the click widget skip it too.
#[test]
fn a_press_skips_the_battle_beat_without_acting() {
    let mut cx = beat_cx(false, false);
    let mut g = gfx();
    type Press = fn(&mut BattleScreen, &mut Cx);
    let presses: [(&str, Press); 5] = [
        ("confirm", |b, cx| b.act(cx, Action::Confirm)),
        ("key", |b, cx| b.key(cx, Key::Char('a'))),
        ("esc", |b, cx| b.escape(cx)),
        ("right click", |b, cx| b.back(cx)),
        ("click", |b, cx| b.click_id(cx, "battle.beat").expect("the beat's widget")),
    ];
    for (what, press) in presses {
        let mut b = battle(&mut cx, emberwake_world::story::FIGHT_SOURCE);
        b.tick(&mut cx);
        frame(&mut b, &mut cx, &mut g, 1.0);
        let sent = b.host.history.len();
        press(&mut b, &mut cx);
        assert!(b.beat.is_none(), "{what}: skipped");
        assert_eq!(b.host.history.len(), sent, "{what}: no request");
        assert!(!b.auto, "{what}: autoplay not started");
        assert!(b.tokens.anims.iter().all(|a| a.slide.is_none()), "{what}: the walk-in lands");
        cx.set_busy(false);
    }
}

/// T1 with Reduce motion: a plain `PLAIN_MS` fade from black, no ash, no banner, no walk-in; and a fight met on the
/// map has no beat.
#[test]
fn reduce_motion_makes_the_battle_beat_a_plain_fade_and_a_map_fight_has_none() {
    let (mut cx, mut g) = (beat_cx(false, true), gfx());
    let mut b = battle(&mut cx, emberwake_world::story::FIGHT_SOURCE);
    let beat = b.beat.expect("the plain beat");
    assert_eq!((beat.plain, beat.length()), (true, tr::PLAIN_MS));
    b.tick(&mut cx);
    assert!(b.tokens.anims.iter().all(|a| a.slide.is_none()), "no walk-in");
    at_ms(&mut b, 0);
    frame(&mut b, &mut cx, &mut g, 1.0);
    assert_eq!(pixel(&g, W / 2, H / 2)[..3], [0, 0, 0], "black at its start");
    at_ms(&mut b, tr::PLAIN_MS / 2);
    assert_eq!((b.state(&cx)["beat"]["cover"].clone(), b.state(&cx)["beat"]["banner"].clone()), (json!(0.0), Value::Null));
    at_ms(&mut b, tr::PLAIN_MS);
    b.tick(&mut cx);
    assert!(b.beat.is_none(), "control after the fade");
    cx.set_busy(false);
    let b = battle(&mut cx, "map");
    assert!(b.beat.is_none(), "a map fight starts as before");
}

/// The beats' texts read clean at every text size: T1's banner (name, objective, count) and T2's Victory banner.
#[test]
fn the_beat_banners_pass_the_text_audit() {
    let (mut cx, mut g) = (beat_cx(false, false), gfx());
    let mut b = battle(&mut cx, emberwake_world::story::FIGHT_SOURCE);
    b.tick(&mut cx);
    win(&mut b, &mut cx);
    for kind in [Kind::Intro, Kind::Victory] {
        for (size, scale) in TEXT_SIZES {
            b.beat = Some(tr::Beat::new(kind, false));
            at_ms(&mut b, if kind == Kind::Intro { tr::BANNER_IN_MS + tr::BANNER_SLIDE_MS } else { tr::VICTORY_BANNER_AT_MS + tr::BANNER_SLIDE_MS });
            let recs = frame(&mut b, &mut cx, &mut g, scale as f32);
            let card = &b.intro_card;
            let ours = |t: &str| t == card.name || t == card.count_line() || t == "Victory" || (!t.is_empty() && card.objective.starts_with(t));
            let beat_recs: Vec<aegis_ui::painter::TextRec> = recs.into_iter().filter(|r| ours(&r.text)).collect();
            let words: Vec<&str> = beat_recs.iter().map(|r| r.text.as_str()).collect();
            match kind {
                Kind::Intro => assert!(words.contains(&"The Grate") && words.iter().any(|w| w.contains("enem")), "{size}: {words:?}"),
                Kind::Victory => assert!(words.contains(&"Victory"), "{size}: {words:?}"),
            }
            let v = aegis_ui::text_audit::issues(&beat_recs, W as f32, H as f32, &[]);
            let counts: Vec<i64> = ["cut", "edge", "spill", "overlap"].iter().map(|k| v["counts"][*k].as_i64().unwrap_or(-1)).collect();
            assert_eq!(counts, [0, 0, 0, 0], "{kind:?} {size}: {v}");
        }
    }
    b.beat = None;
}

/// T2: a won world fight plays the victory beat (the FX clock slowed for `SLOWMO_MS`, the Victory banner in the outcome
/// panel's place), then the results open over the battle (the battle is not closed under them); a press skips to them.
#[test]
fn a_won_fight_plays_the_victory_beat_then_the_results_over_the_field() {
    let (mut cx, mut g) = (beat_cx(false, false), gfx());
    let mut b = battle(&mut cx, "map");
    win(&mut b, &mut cx);
    b.tick(&mut cx);
    let beat = b.beat.expect("the victory beat");
    assert_eq!((beat.kind, beat.plain), (Kind::Victory, false));
    assert_eq!(beat.clock_rate(Instant::now()), tr::SLOWMO_RATE, "the last hit in slow motion");
    let before = b.clock.now();
    std::thread::sleep(std::time::Duration::from_millis(40));
    b.tick(&mut cx);
    let slowed = b.clock.now() - before;
    assert!(slowed < 40, "the FX clock ran slow: {slowed} ms in 40+ wall ms");
    at_ms(&mut b, tr::VICTORY_BANNER_AT_MS + tr::BANNER_SLIDE_MS);
    frame(&mut b, &mut cx, &mut g, 1.0);
    assert!(widget(&b, "banner.continue").is_none(), "the Victory banner stands in for the outcome panel");
    assert!(widget(&b, "battle.beat").is_some());
    assert!(cx.nav.is_empty(), "not settled yet");
    at_ms(&mut b, tr::VICTORY_MS);
    b.tick(&mut cx);
    assert!(b.beat.is_none());
    assert_eq!(cx.nav.len(), 1, "only the results are pushed: the battle stays under them");
    let results = pushed(&mut cx);
    assert_eq!(results.id(), "battle_results");
    assert_eq!(results.state(&cx)["over_battle"], json!({"plain": false, "arrives": true, "leaving": null}));
    // a press skips the beat: the results at once
    let mut cx = beat_cx(false, false);
    let mut b = battle(&mut cx, "map");
    win(&mut b, &mut cx);
    b.tick(&mut cx);
    assert!(b.beat.is_some());
    b.key(&mut cx, Key::Space);
    assert!(b.beat.is_none() && pushed(&mut cx).id() == "battle_results", "skipped to the results");
    // Reduce motion: no slow motion, the banner at once
    let mut cx = beat_cx(false, true);
    let mut b = battle(&mut cx, "map");
    win(&mut b, &mut cx);
    b.tick(&mut cx);
    let beat = b.beat.expect("the victory beat");
    assert_eq!((beat.plain, beat.clock_rate(Instant::now()), beat.frame(Instant::now()).banner), (true, 1.0, Some(1.0)));
    // a fight met on the road leaves no arrival
    let mut cx = beat_cx(false, false);
    let mut b = battle(&mut cx, crate::battle::TRAVEL_RUNNER);
    win(&mut b, &mut cx);
    b.tick(&mut cx);
    b.key(&mut cx, Key::Space);
    assert_eq!(pushed(&mut cx).state(&cx)["over_battle"]["arrives"], json!(false));
}

/// The vignette, then a story fight's battle beat, the victory beat, the results over the field, Continue's fade and
/// the hub's arrival card ("Back at <place> · Day <n>"); with the story going on to a vignette there is no card.
#[test]
fn a_story_goes_from_the_vignette_through_both_beats_to_the_hub() {
    for go_on in [false, true] {
        let (mut cx, mut g) = (beat_cx(go_on, false), gfx());
        let mut stage = StoryStage::new(&cx);
        stage.tick(&mut cx);
        let mut v = pushed(&mut cx);
        assert_eq!(v.id(), "vignette");
        for _ in 0..200 {
            if cx.nav.iter().any(|n| matches!(n, Nav::Pop)) {
                break;
            }
            v.key(&mut cx, Key::Space);
        }
        cx.nav.clear();
        stage.resume(&mut cx);
        stage.tick(&mut cx);
        let mut b = pushed(&mut cx);
        assert_eq!((b.id(), b.state(&cx)["beat"]["kind"].clone()), ("battle", json!("intro")), "T1 after the vignette");
        b.tick(&mut cx);
        frame(b.as_mut(), &mut cx, &mut g, 1.0);
        b.click_id(&mut cx, "battle.beat").expect("a click skips the beat");
        assert!(b.state(&cx).get("beat").is_none());
        assert_eq!(b.command(&cx, &json!({"cmd": "autoplay"})).expect("the battle answers")["ok"], json!(true));
        b.tick(&mut cx);
        assert_eq!(b.state(&cx)["beat"]["kind"], json!("victory"), "T2 on the win");
        b.key(&mut cx, Key::Enter);
        let mut results = pushed(&mut cx);
        assert!(cx.nav.is_empty(), "the battle is not closed under the results");
        frame(results.as_mut(), &mut cx, &mut g, 1.0);
        results.click_id(&mut cx, "results.continue").expect("Continue");
        assert!(cx.nav.is_empty() && results.state(&cx)["over_battle"]["leaving"].is_u64(), "Continue fades out first");
        frame(results.as_mut(), &mut cx, &mut g, 1.0);
        results.key(&mut cx, Key::Enter);
        assert!(cx.nav.len() == 2 && cx.nav.iter().all(|n| matches!(n, Nav::Pop)), "the results and the battle close");
        cx.nav.clear();
        assert!(cx.game.as_ref().is_some_and(|g| g.arrival.is_some()), "the win leaves the hub its arrival");
        stage.resume(&mut cx);
        stage.tick(&mut cx);
        if go_on {
            let v = pushed(&mut cx);
            assert_eq!(v.id(), "vignette", "the story goes on to its vignette");
            assert!(cx.game.as_ref().is_some_and(|g| g.arrival.is_none()), "no arrival card then");
            continue;
        }
        let mut map = pushed(&mut cx);
        assert_eq!(map.id(), "region_map", "the enter step: the hub");
        map.tick(&mut cx);
        assert_eq!(map.state(&cx)["arrival"]["card"], json!(format!("Back at The Yard \u{b7} Day {}", cx.gs().time + 1)));
        let recs = frame(map.as_mut(), &mut cx, &mut g, 1.0);
        assert!(recs.iter().any(|r| r.text.starts_with("Back at The Yard")), "the card is drawn");
    }
}

/// T2 step 4: the arrival names what changed since the fight started: a new bunk talk, new shop stock (none here: the
/// lines are empty), and every visible running countdown on the card.
#[test]
fn the_arrival_card_reads_the_place_the_day_and_the_running_countdowns() {
    let mut cx = beat_cx(false, false);
    cx.gs_mut().location_id = format!("{WORLD}:yard");
    let before = tr::snapshot(&cx.data, cx.gs());
    let a = tr::arrival(&cx.data, cx.gs(), &before);
    assert_eq!(a, tr::Arrival { card: format!("Back at The Yard \u{b7} Day {}", cx.gs().time + 1), changes: Vec::new() });
}

/// One 1280 x 720 frame of `screens` (the first drawn first, as the shell draws the screens below an overlay) saved as
/// `<out>/<name>.png`.
fn save_shot(screens: &mut [&mut dyn Screen<GameCx>], cx: &mut Cx, out: &str, name: &str) {
    let mut g = gfx();
    g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
    g.view = aegis_ui::painter::View::fit(1280, 720);
    let n = screens.len();
    for (i, screen) in screens.iter_mut().enumerate() {
        screen.draw(cx, &mut g, (-500, -500), i + 1 == n);
    }
    let file = std::fs::File::create(std::path::Path::new(out).join(format!("{name}.png"))).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), 1280, 720);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&g.canvas.px).unwrap();
}

/// Review shots of the key frames with the game's art and theme (only with `AEGIS_REVIEW_DIR` set): the ash cover and
/// its wipe, the banner with the party walking in and leaving, Reduce motion's fade, the Victory banner, the results
/// over the field and the hub's arrival card.
#[test]
fn review_shots_of_the_beats() {
    let Ok(out) = std::env::var("AEGIS_REVIEW_DIR") else { return };
    std::fs::create_dir_all(&out).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let art = || aegis_ui::art::Art::new(Some(root.to_path_buf()));
    let mut cx = beat_cx(false, false);
    cx.art = art();
    aegis_ui::theme::set_theme(aegis_ui::theme::load(cx.art.layers()).expect("the game's theme"));
    let mut b = battle(&mut cx, emberwake_world::story::FIGHT_SOURCE);
    b.tick(&mut cx);
    for (ms, name) in [(0, "t1_0_cover"), (550, "t1_1_wipe"), (1300, "t1_2_banner_walk_in"), (1950, "t1_3_banner_leaves")] {
        at_ms(&mut b, ms);
        b.clock.ms = ms as f64;
        save_shot(&mut [&mut b], &mut cx, &out, name);
    }
    b.skip_beat(&mut cx);
    win(&mut b, &mut cx);
    b.tick(&mut cx);
    at_ms(&mut b, tr::VICTORY_BANNER_AT_MS + tr::BANNER_SLIDE_MS);
    save_shot(&mut [&mut b], &mut cx, &out, "t2_0_victory");
    b.skip_beat(&mut cx);
    let mut results = pushed(&mut cx);
    std::thread::sleep(std::time::Duration::from_millis(tr::RESULTS_SLIDE_MS + 20));
    save_shot(&mut [&mut b, results.as_mut()], &mut cx, &out, "t2_1_results_over_field");
    results.click_id(&mut cx, "results.continue").expect("Continue");
    std::thread::sleep(std::time::Duration::from_millis(tr::OUT_MS / 2));
    save_shot(&mut [&mut b, results.as_mut()], &mut cx, &out, "t2_2_continue_fade");
    cx.nav.clear();
    let mut map = crate::region::RegionMap::new(&mut cx);
    let card = tr::card(&cx.data, cx.gs());
    if let Some(gm) = cx.game.as_mut() {
        gm.arrival = Some(tr::Arrival { card, changes: vec!["New talk at the bunks: Ash and Bryn.".into()] });
    }
    map.tick(&mut cx);
    save_shot(&mut [&mut map], &mut cx, &out, "t2_3_hub_arrival");
    let mut cx = beat_cx(false, true);
    cx.art = art();
    let mut b = battle(&mut cx, emberwake_world::story::FIGHT_SOURCE);
    b.tick(&mut cx);
    at_ms(&mut b, tr::PLAIN_MS / 2);
    save_shot(&mut [&mut b], &mut cx, &out, "t1_reduce_motion_fade");
}

/// ACT1-FIX4 (PT-99): once the battle is over and the outcome panel shows, the board's hover cards close: a mouse
/// resting on any board widget with a card shows none (a backdrop with no card covers the board under the banner),
/// while the banner's Continue still takes the click.
#[test]
fn hover_cards_close_when_the_battle_ends() {
    let (mut cx, mut g) = (beat_cx(false, false), gfx());
    let mut b = battle(&mut cx, "map");
    frame(&mut b, &mut cx, &mut g, 1.0);
    let board = crate::battle::draw::BOARD;
    let carded: Vec<(i32, i32)> = b.w.iter().filter(|w| !w.tip.is_empty() && board.contains(w.rect.x + w.rect.w / 2, w.rect.y + w.rect.h / 2)).map(|w| (w.rect.x + w.rect.w / 2, w.rect.y + w.rect.h / 2)).collect();
    assert!(!carded.is_empty(), "the live board has hover cards");
    win(&mut b, &mut cx);
    b.tick(&mut cx);
    b.skip_beat(&mut cx);
    cx.nav.clear();
    assert!(b.beat.is_none());
    frame(&mut b, &mut cx, &mut g, 1.0);
    assert!(widget(&b, "banner.continue").is_some(), "the outcome panel shows");
    for at in carded {
        assert!(aegis_ui::ui::tip_widget(&b.w, at).is_none_or(|w| w.id.starts_with("banner.")), "a card at {at:?} after the end");
    }
    let c = b.w.iter().find(|w| w.id == "banner.continue").expect("Continue").rect;
    assert!(aegis_ui::ui::hit(&b.w, (c.x + c.w / 2, c.y + c.h / 2)).is_some(), "Continue still clicks");
}
