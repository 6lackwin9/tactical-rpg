//! INPUT1-PB (INPUT1.md 12, PB1-PB5): the playback aids of a timeline battle (the first catalog encounter played by the
//! timeline ruleset): the impact tracers, the strip's block cursor, the playback speed, the log by round and block and
//! the timeline view pausing the playback. Every expectation comes from the rules and the kit (the popped commands,
//! `Session::log_groups`, the tuning's own pacing), never a shipped number; classic is unchanged.

use crate::battle::draw::{log_groups_json, log_part_name, LOG_INDENT};
use crate::battle::marks::{tracer_alpha, traces_of};
use crate::battle::{Action, BattleScreen, PLAYBACK_SPEED, PLAYBACK_SPEEDS, TRACER_ALPHA, TRACER_MS};
use aegis_ui::canvas::Rect;
use crate::fx;
use crate::screens::{Cx, Key, Screen};
use aegis_ui::theme::th;
use aegis_ui::ui::{Gfx, Widget, H, W};
use emberwake_battle::command::CmdKind;
use emberwake_battle::grid::V2;
use emberwake_battle::session::{Phase, Session};
use aegis_ui::art::Art;
use emberwake_battle::skill::Kind;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use emberwake_rules::ruleset::TIMELINE;

fn arena(ruleset: Option<&str>) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    b.tick(&mut cx);
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn widget<'a>(b: &'a BattleScreen, id: &str) -> Option<&'a Widget<Action>> {
    b.w.iter().find(|x| x.id == id)
}

/// One playback tick as soon as it may go: a pending impact fired, the step pacing and the impact gap over.
fn tick_now(b: &mut BattleScreen, cx: &mut Cx) {
    b.flush(cx);
    b.next_at = Instant::now();
    b.gap_until = None;
    b.tick(cx);
}

/// The tile of unit `u` as the tracers use it.
fn tile(b: &BattleScreen, u: usize) -> Value {
    let c = s(b).w.grid.pos(u).or(b.tokens.anims.get(u).and_then(|a| a.seen.pos)).expect("a tile");
    json!([c.x, c.y])
}

/// Plays on to the next round (planned with the party tactic) once a round is over; false when the battle is.
fn next_round(b: &mut BattleScreen, cx: &mut Cx) -> bool {
    if s(b).combat_over || !s(b).can_advance_round() {
        return false;
    }
    b.run(cx, Action::NextRound);
    b.act(cx, Action::Confirm);
    s(b).plan_confirmed
}

/// PB1: a step whose attack landed puts one tracer from the actor's tile to each unit it hit or staggered (solid,
/// alpha 200 at its impact, the actor's side colour); a self-only command (Defend, Focus, Rest) puts none. Skip
/// animations: the tracers draw as the step's result shows.
#[test]
fn a_landed_hit_puts_one_tracer_per_unit_and_a_self_command_none() {
    let (mut cx, _, mut b) = arena(Some(TIMELINE));
    cx.settings.v["skip_anim"] = json!(true);
    b.act(&mut cx, Action::Confirm);
    let (mut hits, mut selfs) = (0, 0);
    for _ in 0..600 {
        if s(&b).phase == Phase::Replay && ((hits > 0 && selfs > 0) || !next_round(&mut b, &mut cx)) {
            break;
        }
        b.tracers.clear();
        let before = s(&b).last_popped;
        b.send(&cx, json!({"cmd": "step"}));
        let sess = s(&b);
        let Some((_, _, idx)) = sess.last_popped.filter(|p| Some(*p) != before) else { continue };
        let a = sess.resolver.scheduled[idx].clone();
        let tr = b.state(&cx)["overlay"]["tracers"].as_array().cloned().expect("overlay.tracers");
        match a.command.kind {
            CmdKind::Defend { .. } | CmdKind::Focus { .. } | CmdKind::Rest { .. } => {
                assert!(tr.is_empty(), "a self command: {tr:?}");
                selfs += 1;
            }
            CmdKind::Attack { .. } if !a.cancelled && a.command.reactions.is_empty() && !a.command.hits.is_empty() => {
                let mut landed: Vec<usize> = a.command.hits.iter().map(|h| h.0).chain(a.command.staggers.iter().map(|st| st.target)).filter(|&u| u != a.unit).collect();
                landed.sort_unstable();
                landed.dedup();
                assert_eq!(tr.len(), landed.len(), "one tracer per unit it landed on: {tr:?}");
                for u in landed {
                    assert!(tr.contains(&json!({"from": tile(&b, a.unit), "to": tile(&b, u), "alpha": TRACER_ALPHA, "dashed": false})), "{u}: {tr:?}");
                }
                let col = if sess.w.allies.contains(&a.unit) { th().gold } else { th().foe };
                assert!(b.tracers.iter().all(|t| t.0.col == col && !t.0.dashed));
                hits += 1;
            }
            _ => {}
        }
    }
    assert!(hits > 0 && selfs > 0, "an attack landed ({hits}) and a self command went ({selfs})");
}

/// PB1 by the rules on a played command: a protected flank (MR8 c) or a lost target (GD5) is one dashed tracer to
/// the target; a cancelled action none; a support on another unit draws in the heal colour, on itself none.
#[test]
fn a_miss_is_one_dashed_tracer_and_a_cancelled_action_none() {
    let (mut cx, _, mut b) = arena(Some(TIMELINE));
    cx.settings.v["skip_anim"] = json!(true);
    b.act(&mut cx, Action::Confirm);
    while s(&b).phase != Phase::Replay {
        b.send(&cx, json!({"cmd": "step"}));
    }
    let heal = cx.tuning.value.heal_color;
    let sess = b.host.session.as_mut().unwrap();
    let idx = sess.resolver.scheduled.iter().position(|a| matches!(a.command.kind, CmdKind::Attack { target: Some(_), .. })).expect("an attack");
    let CmdKind::Attack { target: Some(t), .. } = sess.resolver.scheduled[idx].command.kind else { unreachable!() };
    let actor = sess.resolver.scheduled[idx].unit;
    let at = |u: usize| Some(V2::new(u as i32, 0));
    let c = &mut sess.resolver.scheduled[idx].command;
    (c.missed, c.evaded) = (true, None);
    c.hits.clear();
    c.staggers.clear();
    c.reactions.clear();
    let col = if sess.w.allies.contains(&actor) { th().gold } else { th().foe };
    let want = vec![fx::Trace { from: V2::new(actor as i32, 0), to: V2::new(t as i32, 0), col, dashed: true }];
    assert_eq!(traces_of(sess, idx, &at, heal), want, "MR8 c: a protected flank");
    let c = &mut sess.resolver.scheduled[idx].command;
    (c.missed, c.evaded) = (false, Some(t));
    assert_eq!(traces_of(sess, idx, &at, heal), want, "GD5: a lost target");
    sess.resolver.scheduled[idx].cancelled = true;
    assert!(traces_of(sess, idx, &at, heal).is_empty(), "a cancelled action");
    sess.resolver.scheduled[idx].cancelled = false;
    let other = (actor + 1) % sess.w.units.len();
    let skill = sess.w.cat.skills.iter().position(|k| emberwake_battle::skill::kind(k) == Kind::Support).unwrap_or(0);
    for (target, want) in [(other, vec![fx::Trace { from: V2::new(actor as i32, 0), to: V2::new(other as i32, 0), col: heal, dashed: false }]), (actor, Vec::new())] {
        let c = &mut sess.resolver.scheduled[idx].command;
        (c.evaded, c.failed) = (None, false);
        c.kind = CmdKind::Support { target, skill, last_heal: 5, last_status: String::new(), last_cleansed: Vec::new() };
        assert_eq!(traces_of(sess, idx, &at, heal), want, "a support on {target}");
    }
}

/// PB1: with the group `tracers` off the agent link lists none (and the board draws none); the tracer fades from
/// alpha 200 to 0 over 400 ms on the FX clock, and under reduce motion holds 200 for 400 ms, then is gone.
#[test]
fn the_group_off_lists_no_tracer_and_a_tracer_fades_or_holds() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
    let now = b.clock.now();
    b.tracers.push((fx::Trace { from: V2::new(1, 1), to: V2::new(3, 1), col: th().gold, dashed: false }, now));
    assert_eq!(b.state(&cx)["overlay"]["tracers"].as_array().map(Vec::len), Some(1));
    cx.settings.v["battle_overlays"]["tracers"] = json!(false);
    assert_eq!(b.state(&cx)["overlay"]["tracers"], json!([]), "the group off");
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!((tracer_alpha(0, false), tracer_alpha(TRACER_MS / 2, false), tracer_alpha(TRACER_MS, false)), (TRACER_ALPHA, TRACER_ALPHA / 2, 0));
    assert_eq!((tracer_alpha(0, true), tracer_alpha(TRACER_MS - 1, true), tracer_alpha(TRACER_MS, true)), (TRACER_ALPHA, TRACER_ALPHA, 0), "reduce motion: no fade");
    // classic: no tracer, no playback state
    let (mut cx, _, mut b) = arena(None);
    cx.settings.v["skip_anim"] = json!(true);
    b.act(&mut cx, Action::Confirm);
    for _ in 0..40 {
        b.send(&cx, json!({"cmd": "step"}));
    }
    let st = b.state(&cx);
    assert!(b.tracers.is_empty() && st.get("playback").is_none() && st.get("log_groups").is_none(), "classic is unchanged");
}

/// PB1: with animations on a step's tracers wait in its impact and draw from the impact (rule I's moment).
#[test]
fn the_tracers_wait_for_the_impact() {
    let (mut cx, _, mut b) = arena(Some(TIMELINE));
    b.act(&mut cx, Action::Confirm);
    let mut seen = false;
    for _ in 0..300 {
        if s(&b).phase == Phase::Replay && !next_round(&mut b, &mut cx) {
            break;
        }
        b.flush(&cx);
        b.tracers.clear();
        b.send(&cx, json!({"cmd": "step"}));
        let Some(i) = b.impacts.first().filter(|i| !i.traces.is_empty()) else { continue };
        let traces = i.traces.clone();
        assert!(b.tracers.is_empty(), "nothing before the impact");
        b.flush(&cx);
        assert_eq!(b.tracers.iter().map(|t| t.0).collect::<Vec<_>>(), traces, "the impact's tracers");
        seen = true;
        break;
    }
    assert!(seen, "an action with tracers had an impact");
}

/// PB2: the strip frames (its block widget active) and `playback.block` names the block the last step resolved,
/// from its first step until the next block's; none at the round's start (Confirm, no step yet) and once the round
/// is over and its last impact has played.
#[test]
fn the_strip_cursor_follows_the_resolving_block() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
    cx.settings.v["skip_anim"] = json!(true);
    b.act(&mut cx, Action::Confirm);
    assert_eq!(b.state(&cx)["playback"]["block"], Value::Null, "the round's start");
    let mut seen = Vec::new();
    while s(&b).phase != Phase::Replay {
        let resolving = if s(&b).phase == Phase::Planning { 1 } else { s(&b).current_block };
        b.send(&cx, json!({"cmd": "step"}));
        if s(&b).phase == Phase::Replay {
            break;
        }
        assert_eq!(b.state(&cx)["playback"]["block"], json!(resolving));
        b.draw(&mut cx, &mut g, (-500, -500), true);
        for blk in 1..=s(&b).def.total_blocks {
            assert_eq!(widget(&b, &format!("strip.block{blk}")).unwrap().active, blk == resolving, "block {blk} while {resolving} resolves");
        }
        seen.push(resolving);
    }
    assert!(seen.windows(2).all(|w| w[0] <= w[1]) && seen.first() == Some(&1), "{seen:?}");
    b.gap_until = None;
    assert_eq!(b.state(&cx)["playback"]["block"], Value::Null, "the round's end");
}

/// PB3: P cycles 1x, 0.5x, Block and the setting is kept; during playback the chips take the tactic menus' place
/// (Next block in Block mode only), while planning the menus are back; classic: no chip and P changes nothing.
#[test]
fn p_cycles_the_speeds_and_the_chips_show_during_playback() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
    assert_eq!(cx.settings.v[PLAYBACK_SPEED], json!("1x"), "the default");
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(widget(&b, "strip.party").is_some() && widget(&b, "playback.speed.1").is_none(), "planning: the menus");
    let mut seen = Vec::new();
    for _ in 0..3 {
        b.key(&mut cx, Key::Char('p'));
        seen.push(cx.settings.v[PLAYBACK_SPEED].as_str().unwrap().to_string());
    }
    assert_eq!(seen, ["0.5x", "block", "1x"]);
    let saved: Value = serde_json::from_str(&std::fs::read_to_string(cx.shot_dir.join("settings.json")).expect("saved")).unwrap();
    assert_eq!(saved[PLAYBACK_SPEED], json!("1x"), "remembered");
    b.act(&mut cx, Action::Confirm);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(widget(&b, "strip.party").is_none() && widget(&b, "strip.hero").is_none());
    for (k, (v, _, id)) in PLAYBACK_SPEEDS.iter().enumerate() {
        b.run(&mut cx, Action::PlaybackSpeed(k));
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(PLAYBACK_SPEEDS.iter().all(|c| widget(&b, c.2).is_some_and(|w| w.active == (c.2 == *id))), "{v} on");
        assert_eq!(widget(&b, "playback.next").is_some(), *v == "block", "{v}: Next block");
        assert_eq!(b.state(&cx)["playback"]["speed"], json!(v));
    }
    let (mut cx, mut g, mut b) = arena(None);
    b.key(&mut cx, Key::Char('p'));
    b.act(&mut cx, Action::Confirm);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!(cx.settings.v[PLAYBACK_SPEED], json!("1x"));
    assert!(widget(&b, "playback.speed.1").is_none() && widget(&b, "strip.party").is_some(), "classic keeps its menus");
}

/// PB3: during playback the speed chips (and Next block) sit in the tactic menus' place, left of Auto, and their
/// labels fit at the four text sizes (no cut, spilled, overlapping or off-canvas label).
#[test]
fn the_speed_chips_fit_at_every_text_size() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
    cx.settings.v[PLAYBACK_SPEED] = json!("block");
    b.act(&mut cx, Action::Confirm);
    g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
    g.view = aegis_ui::painter::View::fit(1280, 720);
    let labels = ["1x", "0.5x", "Block", "Next block"];
    for scale in [0.85f32, 1.0, 1.15, 1.3] {
        let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale);
        let auto = widget(&b, "strip.auto").expect("Auto").rect;
        let chips: Vec<Rect> = ["playback.speed.1", "playback.speed.half", "playback.speed.block", "playback.next"].iter().map(|id| widget(&b, id).unwrap_or_else(|| panic!("{id}")).rect).collect();
        assert!(chips.windows(2).all(|c| c[0].x + c[0].w <= c[1].x) && chips.last().is_some_and(|c| c.x + c.w < auto.x), "scale {scale}: {chips:?} left of Auto {auto:?}");
        let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
        let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
        let named = |e: &Value| e["widget"].as_str().is_some_and(|w| w.starts_with("playback.")) || ["text", "shown", "a", "b"].iter().any(|k| e[*k].as_str().is_some_and(|x| labels.contains(&x)));
        for kind in ["cut", "edge", "overlap", "spill"] {
            let hits: Vec<&Value> = v[kind].as_array().unwrap().iter().filter(|e| named(e)).collect();
            assert!(hits.is_empty(), "scale {scale}: {kind} {hits:?}");
        }
    }
}

/// PB3 (timed): at 0.5x a block's playback takes twice as long: the wait after a step, the floats, and the FX clock
/// (walks, clips, tracers) runs at half the animation speed; while planning and under skip animations it plays as
/// 1x.
#[test]
fn half_speed_doubles_the_playbacks_waits_and_clock() {
    let measure = |speed: &str| {
        let (mut cx, _, mut b) = arena(Some(TIMELINE));
        cx.settings.v[PLAYBACK_SPEED] = json!(speed);
        let planning_pace = b.pace(&cx);
        b.act(&mut cx, Action::Confirm);
        // the FX clock over 80 wall ms (no step goes: the next one is far off)
        b.next_at = Instant::now() + Duration::from_secs(60);
        let ms0 = b.clock.ms;
        let t0 = Instant::now();
        b.clock_at = t0 - Duration::from_millis(80);
        b.tick(&mut cx);
        let clock = (b.clock.ms - ms0, t0.elapsed().as_secs_f64() * 1000.0);
        // the wait after a step
        b.gap_until = None;
        b.impacts.clear();
        let t1 = Instant::now();
        b.next_at = t1;
        b.tick(&mut cx);
        let wait = (b.next_at.duration_since(t1).as_millis() as u64, t1.elapsed().as_millis() as u64);
        b.show(&cx, vec![(0, "-1".into(), th().foe)], Vec::new(), Vec::new());
        let float = b.floats.last().unwrap().ms;
        cx.settings.v["skip_anim"] = json!(true);
        (planning_pace, clock, wait, float, b.pace(&cx), cx.tuning.value.clone())
    };
    let (p1, c1, w1, f1, skip1, t) = measure("1x");
    let (p2, c2, w2, f2, skip2, _) = measure("0.5x");
    assert_eq!((p1, p2, skip1, skip2), (1, 1, 1, 1), "planning and skip animations play as 1x");
    assert!(c1.0 >= 79.5 && c1.0 <= 80.0 + c1.1 + 0.5, "1x clock {c1:?}");
    assert!(c2.0 >= 39.5 && c2.0 <= (80.0 + c2.1) / 2.0 + 0.5, "0.5x clock {c2:?}");
    assert!(w1.0 + w1.1 >= t.step_ms && w1.0 <= t.step_ms, "1x step wait {w1:?}");
    assert!(w2.0 + w2.1 >= 2 * t.step_ms && w2.0 <= 2 * t.step_ms, "0.5x step wait {w2:?}");
    assert_eq!((f1, f2), (t.float_ms, 2 * t.float_ms), "the floats");
}

/// PB3: Block mode waits after each block (the message bar "Block <b> of <n>: N for the next", nothing sent while it
/// waits), N plays the next one; skip animations still waits; autoplay waits after the round's last block too and
/// N goes on to the next round.
#[test]
fn block_mode_waits_after_each_block_and_n_plays_the_next() {
    let (mut cx, _, mut b) = arena(Some(TIMELINE));
    cx.settings.v["skip_anim"] = json!(true);
    cx.settings.v[PLAYBACK_SPEED] = json!("block");
    b.act(&mut cx, Action::Autoplay);
    assert!(b.auto && s(&b).plan_confirmed, "{}", b.message);
    let n = s(&b).def.total_blocks;
    let mut waits = Vec::new();
    for _ in 0..800 {
        if s(&b).current_round > 1 || s(&b).combat_over {
            break;
        }
        tick_now(&mut b, &mut cx);
        let st = b.state(&cx)["playback"].clone();
        if st["waiting"] != json!(true) {
            continue;
        }
        assert_eq!(st["speed"], json!("block"));
        let blk = st["block"].as_i64().expect("the block") as i32;
        assert_eq!(b.message, format!("Block {blk} of {n}: N for the next"));
        let sent = b.host.history.len();
        tick_now(&mut b, &mut cx);
        assert_eq!(b.host.history.len(), sent, "nothing goes while it waits");
        waits.push(blk);
        b.key(&mut cx, Key::Char('n'));
        assert_eq!(b.state(&cx)["playback"]["waiting"], json!(false), "N lets it go on");
    }
    assert!(!waits.is_empty(), "it waited");
    assert_eq!(waits, (1..=waits.len() as i32).collect::<Vec<_>>(), "each block in turn");
    if !s(&b).combat_over {
        assert_eq!(waits.len() as i32, n, "the round's last block waits too under autoplay");
        assert_eq!(s(&b).current_round, 2, "N went on to the next round");
    }
}

/// PB5: T during playback pauses it at the end of the resolving block (`playback.waiting`, nothing sent); closing the
/// view clears it and the playback goes on; in Block mode N steps the next block while the view stays open, its
/// playhead following.
#[test]
fn t_pauses_the_playback_at_the_block_end_and_closing_resumes() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
    cx.settings.v["skip_anim"] = json!(true);
    b.act(&mut cx, Action::Confirm);
    b.key(&mut cx, Key::Char('t'));
    assert!(b.tv_open);
    let mut paused = false;
    for _ in 0..200 {
        tick_now(&mut b, &mut cx);
        if b.state(&cx)["playback"]["waiting"] == json!(true) {
            paused = true;
            break;
        }
    }
    assert!(paused && s(&b).phase == Phase::Resolving, "paused at a block's end: {}", b.message);
    assert!(b.message.contains("timeline view"), "{}", b.message);
    let sent = b.host.history.len();
    tick_now(&mut b, &mut cx);
    assert_eq!(b.host.history.len(), sent, "held while open");
    b.key(&mut cx, Key::Char('t'));
    assert!(!b.tv_open && b.state(&cx)["playback"]["waiting"] == json!(false), "closing clears it");
    tick_now(&mut b, &mut cx);
    assert!(b.host.history.len() > sent, "and the playback goes on");
    // Block mode: N plays the next block with the view open
    cx.settings.v[PLAYBACK_SPEED] = json!("block");
    b.key(&mut cx, Key::Char('t'));
    let mut steps = 0;
    for _ in 0..200 {
        if s(&b).phase == Phase::Replay {
            break;
        }
        tick_now(&mut b, &mut cx);
        if b.state(&cx)["playback"]["waiting"] == json!(true) {
            let blk = b.state(&cx)["playback"]["block"].clone();
            b.draw(&mut cx, &mut g, (-500, -500), true);
            assert_eq!(json!(b.tv_shown.as_ref().expect("open").playhead), blk, "the playhead follows");
            b.key(&mut cx, Key::Char('n'));
            assert!(b.tv_open, "the view stays open");
            steps += 1;
        }
    }
    assert!(steps > 0, "N stepped with the view open");
}

/// PB4: the log by round and block: the agent link's `log_groups` is the kit's parts (their lines read in order are
/// the log); the expanded log draws "Round <r>", then each part's header ("Round start", "Block <b>", "Round end")
/// above its lines, 6 px in; during playback the compact log shows the resolving block's part; every header and
/// line passes the text audit at the four text sizes.
#[test]
fn the_log_groups_lines_under_their_round_and_block() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
    cx.settings.v["skip_anim"] = json!(true);
    b.act(&mut cx, Action::Confirm);
    let mut compact = 0;
    while s(&b).phase != Phase::Replay {
        b.send(&cx, json!({"cmd": "step"}));
        if let Some(blk) = b.cursor_block(Instant::now()) {
            b.draw(&mut cx, &mut g, (-500, -500), true);
            let w = widget(&b, "bar.last").expect("the compact log");
            assert!(w.text.starts_with(&format!("Block {blk}: ")), "{}", w.text);
            assert_eq!(w.tip[0].0, format!("Round {} block {blk}", s(&b).current_round));
            compact += 1;
        }
    }
    assert!(compact > 0);
    let groups = s(&b).log_groups();
    assert_eq!(b.state(&cx)["log_groups"], log_groups_json(&groups));
    assert_eq!(groups.iter().flat_map(|p| p.lines.clone()).collect::<Vec<_>>(), s(&b).log_lines(0, 0), "every line once, in order");
    assert!(groups.iter().any(|p| p.block >= 1), "a block's part");
    g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
    g.view = aegis_ui::painter::View::fit(1280, 720);
    b.show_log = true;
    for scale in [0.85f32, 1.0, 1.15, 1.3] {
        let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale);
        let panel = widget(&b, "log.panel").expect("the log").rect;
        let rows: Vec<&aegis_ui::painter::TextRec> = recs.iter().filter(|r| r.y > panel.y as f32 + 12.0 && r.y < (panel.y + panel.h) as f32 && r.x >= panel.x as f32 && r.x < (panel.x + panel.w) as f32).collect();
        assert!(!rows.is_empty(), "scale {scale}: rows drawn");
        // each line drawn sits below the header of the part it belongs to, 6 px further in
        let mut part: Option<&emberwake_battle::session::LogGroup> = None;
        let mut header_x = 0.0;
        for r in &rows {
            if let Some(p) = groups.iter().find(|p| log_part_name(p.block) == r.text) {
                part = Some(p);
                header_x = r.x;
            } else if r.text.starts_with("Round ") {
                part = None;
            } else if let Some(p) = part {
                let t = r.text.trim_start();
                assert!(p.lines.iter().any(|l| l.contains(t)), "scale {scale}: {:?} under {}", r.text, log_part_name(p.block));
                assert!(r.x >= header_x + LOG_INDENT - 0.6, "scale {scale}: {:?} 6 px in", r.text);
            }
        }
        let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
        let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
        let texts: Vec<&str> = rows.iter().map(|r| r.text.as_str()).collect();
        let named = |e: &Value| ["text", "shown", "a", "b"].iter().any(|k| e[*k].as_str().is_some_and(|x| texts.contains(&x)));
        // (a spill is only the log's own: the panel lies over the board's tile widgets)
        let ours = |e: &Value| e["widget"].as_str().is_none_or(|w| w.starts_with("log.") || w.starts_with("bar."));
        for kind in ["cut", "edge", "overlap", "spill"] {
            let hits: Vec<&Value> = v[kind].as_array().unwrap().iter().filter(|e| named(e) && (kind != "spill" || ours(e))).collect();
            assert!(hits.is_empty(), "scale {scale}: {kind} {hits:?}");
        }
    }
}
