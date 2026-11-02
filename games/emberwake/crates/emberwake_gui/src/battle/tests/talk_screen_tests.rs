//! ENCOUNTER2-C2 (docs/specs/ENCOUNTER2_C.md 3.3; section 6 "battle.rs tests" and the text audit) on a scratch world
//! built here (no shipped data): encounter `t_talk` raises a talk at round 1's start (Ash speaks) and one at round 1
//! block 2 (the Grunt speaks); `t_plain` is the same fight without phases. Expectations come from the rules: while a
//! talk is open the screen sends no request and autoplay waits (DT5); its replies run in the Arena's scratch state or
//! the game in play (DT6, DT9); closing resumes the playback after `impact_gap_ms` (DT10); the agent link reads it
//! (DT11); the speaker's token is the one whose hero key plays the speaker (DT8).

use crate::battle::flow::qid_in;
use crate::battle::{Action, BattleScreen, GRID_X, GRID_Y};
use crate::talk::{self, TalkAct};
use crate::screens::{Cx, Key, Screen};
use aegis_ui::ui::{Gfx, H, W};
use aegis_core::json::{self, s};
use serde_json::{json, Value};
use std::time::Instant;
use crate::hud::hud_tests::{frame, game_cx, gfx, rect};
use crate::screens::Game;
use aegis_ui::screen::TEXT_SIZES;
use emberwake_world::Data;
use std::sync::Arc;

const WORLD: &str = "talkw";
const LONG_LINE: &str = "The riders took the north road at dusk, and the forge has been cold since; if we hold this alley until the bell, the wardens can close the gate behind us and nobody else has to bleed for it tonight.";

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

fn data() -> Data {
    let skills = json!([
        {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
        {"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3},
        {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
        {"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true},
        {"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
        {"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
        {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}
    ]);
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 4000, "speed": 12, "attack_range": 1, "skill_cap": 5});
    // a party character's base unit is class-derived (the world's new game checks it)
    let hero = json!({"id": "t_hero", "name": "t_hero", "side": "party", "class_id": "drill", "level": 1});
    let class = json!({"id": "drill", "name": "Drill", "base_stats": {"STR": 8, "DEX": 10, "INT": 6, "CONS": 30, "PERC": 10, "KNOW": 10}});
    let catalog = json!({"skills": skills, "classes": [class], "units": [hero, unit("t_foe", "enemy")]});
    let encounter = |id: &str, phases: Value| {
        json!({"id": id, "name": id, "rows": vec![".".repeat(12); 9], "party": ["ash"], "enemies": ["grunt"], "total_blocks": 6, "unit_ap": 10, "max_rounds": 3, "phases": phases})
    };
    let phases = json!([
        {"id": "opening", "name": "Opening", "trigger": {"kind": "round", "round": 1}, "effects": [{"type": "talk", "dialogue": "dlg_open", "speaker": "ash"}]},
        {"id": "mid", "name": "Mid", "trigger": {"kind": "round", "round": 1, "block": 2}, "effects": [{"type": "talk", "dialogue": "dlg_mid", "speaker": "grunt"}]}
    ]);
    let dialogues = json!([
        {"id": "dlg_open", "start": "a", "services": false, "speaker": "ash", "nodes": [
            {"id": "a", "text": "Hold the line.", "next": "b"},
            {"id": "b", "text": "Ready?", "choices": [{"text": "Ready.", "effects": [{"effect": "set_flag", "key": "ready", "value": true}]}]}]},
        {"id": "dlg_mid", "start": "a", "services": false, "speaker": "grunt", "nodes": [{"id": "a", "text": "Again.", "end": true}]},
        {"id": "dlg_long", "start": "a", "services": false, "speaker": "ash", "nodes": [{"id": "a", "text": LONG_LINE, "choices": [
            {"text": "We hold the alley until the bell, whatever comes down the north road.", "effects": [{"effect": "set_flag", "key": "hold", "value": true}]},
            {"text": "Fall back to the gate.", "effects": [{"effect": "add_flag", "key": "fallback", "delta": 1}]},
            {"text": "Say nothing."}]}]}
    ]);
    let worlds = json!({"talkw": {"manifest": {"id": WORLD, "name": "Talk", "start_location": "hub", "start_party": ["ash"]}, "own": {
        "locations": with_ids(json!([{"id": "hub", "name": "Hub"}])),
        "characters": with_ids(json!([{"id": "ash", "name": "Ash", "role": "party", "base_unit": "t_hero"}, {"id": "grunt", "name": "Grunt", "role": "enemy", "base_unit": "t_foe"}])),
        "dialogues": with_ids(dialogues),
        "encounters": with_ids(json!([encounter("t_talk", phases), encounter("t_plain", json!([]))]))
    }}});
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// A debug game of the scratch world (skip animations on, so no impact holds a step).
fn talk_cx() -> Cx {
    let mut cx = game_cx();
    let d = Arc::new(data());
    let gs = emberwake_world::new_game::create_for_world(&d, WORLD, 1).expect("the scratch world starts a game");
    cx.data = d;
    cx.game = Some(Game::new(gs, -1, true));
    cx.settings.v["battle_view"] = json!("flat");
    cx.settings.v["skip_anim"] = json!(true);
    cx
}

fn arena(cx: &Cx, encounter: &str) -> BattleScreen {
    let b = BattleScreen::arena_with(cx, Some(WORLD.into()), encounter, None, "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    b
}

/// One tick with the step pause and the impact gap over.
fn tick_ready(b: &mut BattleScreen, cx: &mut Cx) {
    b.next_at = Instant::now();
    b.gap_until = None;
    b.tick(cx);
}

fn widget(b: &BattleScreen, id: &str) -> Option<Value> {
    b.widgets().into_iter().find(|w| w["id"] == id)
}

/// The unit whose hero key plays world character `id`.
fn unit_of(b: &BattleScreen, cx: &Cx, id: &str) -> usize {
    let qid = qid_in(&cx.data, WORLD, id);
    let s = b.session();
    (0..s.w.units.len()).find(|&c| s.hero_key(c) == qid).expect("the character fights")
}

/// DT5, DT6, DT8, DT9, DT11: the round-1 talk opens at the first tick, before planning; while it is open no action,
/// key, Esc or right click sends a request and autoplay's plan waits; its Continue and reply run in the Arena's
/// scratch state; once it closes autoplay's plan goes out.
#[test]
fn a_round_one_talk_opens_before_planning_and_holds_every_request() {
    let (mut cx, mut g) = (talk_cx(), gfx());
    let mut b = arena(&cx, "t_talk");
    assert_eq!(b.session().talks().len(), 1, "round 1's talk is cued at the start");
    assert!(b.planning() && b.state(&cx).get("talk").is_none(), "nothing open before the first tick");
    b.tick(&mut cx);
    let st = b.state(&cx);
    let qid = qid_in(&cx.data, WORLD, "ash");
    assert!(!qid.is_empty());
    let read = (st["talk"]["dialogue"].clone(), st["talk"]["node"].clone(), st["talk"]["speaker"].clone(), st["talk"]["text"].clone());
    assert_eq!(read, (json!("dlg_open"), json!("a"), json!(qid), json!("Hold the line.")));
    assert_eq!(st["talk"]["replies"], json!([{"id": "dialogue.continue", "text": "Continue", "locked": false}]));
    assert_eq!(st["talk"]["speaker_unit"], json!(unit_of(&b, &cx, "ash")), "Ash's token wears the ring");
    let sent = b.host.history.len();
    for a in [Action::Confirm, Action::Step, Action::Autoplay, Action::Undo, Action::ToggleLog, Action::Leave] {
        b.act(&mut cx, a);
    }
    for k in [Key::Char('a'), Key::Char('z'), Key::Char('l'), Key::Char('r')] {
        b.key(&mut cx, k);
    }
    b.escape(&mut cx);
    b.back(&mut cx);
    assert_eq!(b.host.history.len(), sent, "no request while the talk is open");
    assert!(!b.auto && !b.show_log && b.planning() && cx.nav.is_empty(), "every battle action and key is swallowed");
    frame(&mut b, &mut cx, &mut g, 1.0);
    for id in ["dialogue", "talk.backdrop", "dialogue.continue"] {
        assert!(widget(&b, id).is_some(), "{id} drawn");
    }
    // autoplay switched on (as the A key does outside a talk) waits for the talk
    b.run(&mut cx, Action::Autoplay);
    assert!(b.auto && b.host.history.len() == sent && b.planning(), "autoplay's plan waits");
    b.click_id(&mut cx, "dialogue.continue").expect("Continue clicks");
    assert_eq!(b.state(&cx)["talk"]["node"], json!("b"));
    b.key(&mut cx, Key::Char('1'));
    assert!(b.talk.is_none() && b.state(&cx).get("talk").is_none(), "the reply ends the talk");
    assert_eq!(b.talk_gs.as_ref().and_then(|gs| gs.flags.get("ready")), Some(&json!(true)), "the flag lands in the Arena's scratch state");
    assert!(!cx.gs().flags.contains_key("ready"), "the game in play is not the Arena's");
    b.tick(&mut cx);
    assert!(b.host.history.len() > sent && b.session().plan_confirmed, "autoplay's plan goes out once the talk closed");
}

/// DT5, DT10: the block-2 talk opens during the playback and holds it (no step, the block stays); Continue closes it
/// and the playback steps on once `impact_gap_ms` is over. DT8: the Grunt's token is the speaker's.
#[test]
fn the_playback_waits_for_a_block_talk_and_resumes_after_the_gap() {
    let mut cx = talk_cx();
    let mut b = arena(&cx, "t_talk");
    b.tick(&mut cx);
    b.run(&mut cx, Action::Talk(TalkAct::Advance));
    b.run(&mut cx, Action::Talk(TalkAct::Choose(0)));
    assert!(b.talk.is_none());
    b.act(&mut cx, Action::Confirm);
    assert!(b.playing, "{}", b.message);
    for _ in 0..40 {
        tick_ready(&mut b, &mut cx);
        if b.talk.is_some() {
            break;
        }
    }
    assert!(b.talk.is_some(), "the block talk opens during the playback");
    let cue = b.session().talks()[1].clone();
    assert_eq!((cue.dialogue.as_str(), cue.round, cue.block), ("dlg_mid", 1, 2));
    assert_eq!(b.state(&cx)["talk"]["speaker_unit"], json!(unit_of(&b, &cx, "grunt")), "an enemy character speaks from its token");
    let at = |b: &BattleScreen| (b.host.history.len(), b.session().current_round, b.session().current_block);
    let held = at(&b);
    for _ in 0..5 {
        tick_ready(&mut b, &mut cx);
    }
    assert_eq!(at(&b), held, "no step while it is open");
    assert!(b.playing, "the playback is held, not stopped");
    b.key(&mut cx, Key::Enter);
    assert!(b.talk.is_none(), "Continue on an end node closes it");
    if cx.tuning.value.impact_gap_ms > 0 {
        assert!(b.gap_until.is_some_and(|g| g > Instant::now()), "the gap starts at the close");
        b.next_at = Instant::now();
        b.tick(&mut cx);
        assert_eq!(b.host.history.len(), held.0, "not before the gap is over");
    }
    tick_ready(&mut b, &mut cx);
    assert_eq!(b.host.history.len(), held.0 + 1, "the playback steps on");
}

/// DT6, DT8, DT9: a world fight's talk runs in the game in play: the reply's flag is the game's; the member who plays
/// the speaker wears the ring.
#[test]
fn a_world_fight_talk_runs_in_the_game_in_play() {
    let mut cx = talk_cx();
    let member = cx.gs().party.active_members().first().map(|m| m.id.clone()).expect("a party member");
    let fight = json::dict(&json!({"encounter": "t_talk"})).clone();
    let mut b = BattleScreen::world(&mut cx, &fight, "map").expect("the world fight starts");
    b.tick(&mut cx);
    let st = b.state(&cx);
    assert_eq!(st["talk"]["dialogue"], json!("dlg_open"));
    let s = b.session();
    let hero = (0..s.w.units.len()).find(|&c| s.hero_key(c) == member).expect("the member fights");
    assert_eq!(st["talk"]["speaker_unit"], json!(hero), "the member who plays Ash wears the ring");
    b.run(&mut cx, Action::Talk(TalkAct::Advance));
    b.run(&mut cx, Action::Talk(TalkAct::Choose(0)));
    assert!(b.talk.is_none() && b.talk_gs.is_none());
    assert_eq!(cx.gs().flags.get("ready"), Some(&json!(true)), "the flag is the game's");
}

/// Every battle without a talk: no `talk` key, no veil or box, and autoplay plans at once.
#[test]
fn without_a_talk_nothing_changes() {
    let (mut cx, mut g) = (talk_cx(), gfx());
    let mut b = arena(&cx, "t_plain");
    b.tick(&mut cx);
    frame(&mut b, &mut cx, &mut g, 1.0);
    assert!(b.talk.is_none() && b.state(&cx).get("talk").is_none() && widget(&b, "talk.backdrop").is_none() && widget(&b, "dialogue").is_none());
    let sent = b.host.history.len();
    b.act(&mut cx, Action::Autoplay);
    assert!(b.host.history.len() > sent && b.playing, "autoplay plans and confirms at once");
}

/// DT7 and the text audit (section 6): the box is the region map's (400 wide times the text size, centred, 6 above
/// the bottom) over a veil that darkens the board, and its texts read clean (no cut, edge, spill or overlap) at the
/// four text sizes, with a long line and three replies.
#[test]
fn the_battle_talk_box_reads_clean_at_every_text_size() {
    let (mut cx, mut g) = (talk_cx(), gfx());
    let mut b = arena(&cx, "t_plain");
    assert!(b.preview_talk(&mut cx, WORLD, "dlg_long", ""), "{}", b.message);
    let probe = |g: &Gfx| {
        let i = (((GRID_Y + 40) * g.canvas.w + GRID_X + 40) * 4) as usize;
        [g.canvas.px[i] as u32, g.canvas.px[i + 1] as u32, g.canvas.px[i + 2] as u32]
    };
    for (size, scale) in TEXT_SIZES {
        // the frame without the talk first: the talk's texts are the ones drawn after it
        let open = b.talk.take();
        let base = frame(&mut b, &mut cx, &mut g, scale as f32).len();
        let bare = probe(&g);
        b.talk = open;
        let recs = frame(&mut b, &mut cx, &mut g, scale as f32);
        let veiled = probe(&g);
        assert!(veiled.iter().zip(bare.iter()).all(|(v, o)| v <= o) && veiled != bare, "{size}: the veil darkens the board ({bare:?} to {veiled:?})");
        let (x, y, w, h) = rect(&widget(&b, "dialogue").expect("the box"));
        assert_eq!(w, ((W - 40) as i64).min((talk::TALK_W as f64 * scale).round() as i64), "{size}: width");
        assert!((x - (W as i64 - x - w)).abs() <= 1 && H as i64 - (y + h) == talk::TALK_GAP as i64, "{size}: centred at the bottom");
        let talk_recs: Vec<aegis_ui::painter::TextRec> = recs[base..].to_vec();
        assert!(talk_recs.iter().any(|r| !r.text.is_empty() && LONG_LINE.starts_with(r.text.as_str())), "{size}: the line is drawn");
        let boxes: Vec<(String, f32, f32, f32, f32)> = b
            .widgets()
            .iter()
            .filter(|wd| {
                let id = wd["id"].as_str().unwrap_or("");
                id == "dialogue" || id.starts_with("dialogue.") || id.starts_with("choice.")
            })
            .map(|wd| {
                let (bx, by, bw, bh) = rect(wd);
                (s(json::dict(wd), "id"), bx as f32, by as f32, bw as f32, bh as f32)
            })
            .collect();
        let v = aegis_ui::text_audit::issues(&talk_recs, W as f32, H as f32, &boxes);
        let counts: Vec<i64> = ["cut", "edge", "spill", "overlap"].iter().map(|k| v["counts"][*k].as_i64().unwrap_or(-1)).collect();
        assert_eq!(counts, [0, 0, 0, 0], "{size}: {v}");
    }
    // the replies run in the scratch state: add_flag counts up
    b.run(&mut cx, Action::Talk(TalkAct::Choose(1)));
    assert_eq!(b.talk_gs.as_ref().map(|gs| json::i(&gs.flags, "fallback", 0)), Some(1));
}
