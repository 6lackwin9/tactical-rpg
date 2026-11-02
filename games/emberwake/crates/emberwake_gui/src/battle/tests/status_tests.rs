//! STATUSFX1-L, the acceptance of STATUSFX1.md section 4 on the screen (the pure rules are tested in `fx.rs`): the
//! battles are the data's own `goblin_ambush`; a status an action puts on is injected into the session before that
//! action's step (the slice's status skills are not in the shipped data), and every expectation comes from a rule.

use crate::battle::tests::{ambush, step_until};
use crate::battle::board::{draw_status_loops, StatusOn};
use crate::battle::{Action, BattleScreen};
use crate::fx;
use crate::screens::{Cx, Screen};
use crate::stage::{Geom, View};
use crate::tuning::Tuning;
use aegis_ui::fx::{Layer, Prim};
use aegis_ui::ui::Ui;
use emberwake_battle::session::Phase;
use serde_json::json;

/// Plays the ambush as `step_until` does, calling `before` with the step's number just before each step; stops after
/// the step `stop` accepts and returns its number (None: the battle ended first).
fn play(cx: &mut Cx, b: &mut BattleScreen, mut before: impl FnMut(&mut BattleScreen, usize), stop: impl Fn(&BattleScreen, usize) -> bool) -> Option<usize> {
    let mut k = 0;
    for _ in 0..600 {
        let (over, phase, planning, can) = {
            let s = b.host.session.as_ref().unwrap();
            (s.combat_over, s.phase, s.is_planning(), s.can_advance_round())
        };
        if over || (phase == Phase::Replay && !can) {
            return None;
        }
        if phase == Phase::Replay {
            b.send(cx, json!({"cmd": "advance"}));
            continue;
        }
        if planning {
            b.send(cx, json!({"cmd": "preset", "preset": "aggressive"}));
        }
        b.flush(cx);
        b.floats.clear();
        b.flashes.clear();
        before(b, k);
        b.send(cx, json!({"cmd": "step"}));
        if stop(b, k) {
            return Some(k);
        }
        k += 1;
    }
    None
}

/// The first step whose action lands on another unit that still stands, and that unit.
fn hit_step() -> (usize, usize) {
    let (mut cx, _g, mut b) = ambush(json!("stage"));
    let lands = |b: &BattleScreen| b.impacts.last().and_then(|i| i.target.filter(|&u| u != i.actor && i.after.get(u).is_some_and(|a| a.alive)));
    let k = play(&mut cx, &mut b, |_, _| {}, |b, _| lands(b).is_some()).expect("the ambush has an action landing on a unit");
    (k, lands(&b).unwrap())
}

fn poison_on(u: usize, at: usize) -> impl FnMut(&mut BattleScreen, usize) {
    move |b, k| {
        if k == at {
            b.host.session.as_mut().unwrap().w.units[u].st.pending.insert("poison".into());
        }
    }
}

fn burst(e: &aegis_ui::fx::Effect) -> bool {
    matches!(e.prim, Prim::Ring(r) if r.col == [fx::VIOLET[0], fx::VIOLET[1], fx::VIOLET[2], 200] && r.ms == fx::BURST_MS)
}

/// 4 (S4, S5): a status an action puts on bursts at that action's impact, not at its pop, and shows (pending) from
/// it; none under skip animations, where it shows at once.
#[test]
fn a_status_an_action_puts_on_bursts_at_its_impact_not_at_the_pop() {
    let (k, u) = hit_step();
    let (mut cx, _g, mut b) = ambush(json!("stage"));
    assert_eq!(play(&mut cx, &mut b, poison_on(u, k), |_, j| j == k), Some(k));
    let pop = b.clock.now();
    let i = b.impacts.last().cloned().expect("the same step lands an impact");
    assert_eq!(i.target, Some(u));
    assert!(i.marks[u].contains(&("poison", true)), "the step's statuses wait in the impact");
    assert!(!b.tokens.anims[u].marks.iter().any(|m| m.0 == "poison"), "not shown before the impact");
    assert!(!b.fx.live.iter().any(burst), "no burst at the pop");
    b.resolve_impacts(&mut cx);
    let at = b.impacts[0].at.expect("resolved");
    assert!(at > pop, "the impact comes after the pop");
    b.clock.ms = at as f64;
    b.fire_due(&mut cx);
    let bursts: Vec<_> = b.fx.live.iter().filter(|e| burst(e)).collect();
    assert_eq!(bursts.len(), 1, "one burst");
    assert_eq!((bursts[0].start, bursts[0].layer), (at, Layer::Air), "at the impact, in the air");
    let geo = Geom::new(View::Stage, &cx.tuning.value);
    let (fx_, fy_) = geo.foot(b.host.session.as_ref().unwrap().w.grid.pos(u).unwrap());
    assert!(matches!(bursts[0].prim, Prim::Ring(r) if r.at == fx::slot_anchor(&geo, (fx_ as f32, fy_ as f32), fx::Slot::Body)), "at its chest");
    assert!(b.tokens.anims[u].marks.contains(&("poison", true, at)), "shown pending from the impact, its loop from the burst");

    let (mut cx, _g, mut b) = ambush(json!("stage"));
    cx.settings.v["skip_anim"] = json!(true);
    assert_eq!(play(&mut cx, &mut b, poison_on(u, k), |_, j| j == k), Some(k));
    assert!(b.impacts.is_empty(), "skip: no impact");
    assert!(b.tokens.anims[u].marks.iter().any(|m| m.0 == "poison"), "skip: shown at once");
    assert!(!b.fx.live.iter().any(burst), "skip: no burst");
}

/// 3 (S3): a status only pending shows pending; at the round start, when it turns active, it shows full, its loop
/// running on (no new burst: no action).
#[test]
fn a_pending_status_turns_full_when_it_turns_active() {
    let (cx, _g, mut b) = ambush(json!("stage"));
    b.sync_anims(&cx);
    let v = (0..b.host.session.as_ref().unwrap().w.units.len()).find(|&i| b.host.session.as_ref().unwrap().w.units[i].is_alive()).unwrap();
    b.host.session.as_mut().unwrap().w.units[v].st.pending.insert("poison".into());
    b.clock.ms = 500.0;
    b.sync_anims(&cx);
    assert_eq!(b.tokens.anims[v].marks, [("poison", true, 500)]);
    let st = &mut b.host.session.as_mut().unwrap().w.units[v].st;
    st.pending.shift_remove("poison");
    st.active.insert("poison".into());
    b.clock.ms = 900.0;
    b.sync_anims(&cx);
    assert_eq!(b.tokens.anims[v].marks, [("poison", false, 500)], "full alpha, the same loop");
    assert!(b.fx.live.is_empty(), "no burst without an action");
}

/// 6 (S1, rule I): a unit that falls draws its looks until its fall at the impact, none after; a stealthed unit draws
/// its sprite and outline at 55 % alpha (`status_fx` names it).
#[test]
fn a_fallen_unit_draws_no_looks_and_a_stealthed_one_is_faded() {
    let (mut cx, mut g, mut b) = ambush(json!("stage"));
    assert!(step_until(&mut cx, &mut b, |b, i| i.struck.iter().any(|&u| !b.host.session.as_ref().unwrap().w.units[u].is_alive())), "the ambush has a fall");
    let dead = *b.impacts[0].struck.iter().find(|&&u| !b.host.session.as_ref().unwrap().w.units[u].is_alive()).unwrap();
    b.tokens.anims[dead].marks = vec![("burn", false, 0)];
    b.impacts[0].marks[dead] = vec![("burn", false)];
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!(b.tokens.anims[dead].status_fx, ["burn"], "burning until its fall");
    b.resolve_impacts(&mut cx);
    let at = b.impacts[0].at.unwrap();
    b.clock.ms = at as f64;
    b.fire_due(&mut cx);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(b.tokens.anims[dead].marks.iter().any(|m| m.0 == "burn") && b.tokens.anims[dead].status_fx.is_empty(), "none from its fall");

    for view in ["stage", "flat"] {
        let (mut cx, mut g, mut b) = ambush(json!(view));
        b.sync_anims(&cx);
        let s = b.host.session.as_ref().unwrap();
        let v = (0..s.w.units.len()).find(|&i| s.w.units[i].is_alive() && s.w.grid.pos(i).is_some()).unwrap();
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let plain = g.canvas.px.clone();
        b.tokens.anims[v].marks = vec![("stealth", false, 0)];
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_eq!(b.tokens.anims[v].status_fx, ["stealth"], "{view}");
        assert_ne!(plain, g.canvas.px, "{view}: the stealthed token is faded");
        assert_eq!(fx::token_alpha(&[fx::Drawn { look: fx::status_look("stealth").unwrap(), pending: false, start: 0 }], &cx.tuning.value), fx::STEALTH_ALPHA);
    }
}

/// 8 (S9): each `state.tokens` entry's `status_fx` names the looks drawn, slot order head, body, feet, tint, alpha.
#[test]
fn the_agent_link_tokens_name_the_status_looks_drawn() {
    for view in ["stage", "flat"] {
        let (mut cx, mut g, mut b) = ambush(json!(view));
        b.sync_anims(&cx);
        let standing: Vec<usize> = {
            let s = b.host.session.as_ref().unwrap();
            (0..s.w.units.len()).filter(|&i| s.w.units[i].is_alive() && s.w.grid.pos(i).is_some()).collect()
        };
        assert!(standing.len() >= 3);
        let (a, c, d) = (standing[0], standing[1], standing[2]);
        {
            let w = &mut b.host.session.as_mut().unwrap().w;
            for id in ["burn", "stun"] {
                w.units[a].st.active.insert(id.into());
            }
            w.units[c].st.pending.insert("bleed".into());
            for id in ["stealth", "fortify"] {
                w.units[d].st.active.insert(id.into());
            }
        }
        b.sync_anims(&cx);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let st = b.state(&cx)["tokens"].clone();
        let list = st.as_array().expect("tokens");
        for (i, e) in list.iter().enumerate() {
            let want: &[&str] = match i {
                _ if i == a => &["stun", "burn"],
                _ if i == c => &["bleed"],
                _ if i == d => &["fortify", "stealth"],
                _ => &[],
            };
            assert_eq!(e["status_fx"], json!(want), "{view}: unit {i}");
            assert_eq!(e["status_fx"], json!(b.tokens.anims[i].status_fx), "{view}: unit {i} as drawn");
        }
    }
}

/// 5 (S7): under reduce motion the loops draw still (the same at any clock); otherwise they move.
#[test]
fn reduce_motion_draws_the_loops_still() {
    let t = Tuning::default();
    let geo = Geom::new(View::Stage, &t);
    let looks = ["stun", "burn"].iter().map(|id| fx::Drawn { look: fx::status_look(id).unwrap(), pending: false, start: 0 }).collect();
    let on = [StatusOn { unit: 0, foot: (300.0, 200.0), left: false, looks }];
    let render = |now: u64, still: bool, on: &[StatusOn]| {
        let mut g = crate::hud::hud_tests::gfx();
        {
            let mut ui: Ui<Action> = Ui::new(&mut g, (-1, -1), true);
            draw_status_loops(&mut ui.p, on, Layer::Air, &geo, &t, now, still);
        }
        g.canvas.px
    };
    let blank = render(100, true, &[]);
    assert_ne!(render(100, true, &on), blank, "still looks show");
    assert_eq!(render(100, true, &on), render(700, true, &on), "the same at any clock");
    assert_ne!(render(100, false, &on), render(350, false, &on), "they move otherwise");
}

/// Decision "FX1-L2 answers": a unit felled with no action to land (a status tick) raises FX1-L2's fall dust at once
/// at its foot point on the ground layer; none under skip animations.
#[test]
fn a_unit_felled_without_an_impact_raises_its_dust_at_once() {
    for skip in [false, true] {
        let (mut cx, _g, mut b) = ambush(json!("stage"));
        cx.settings.v["skip_anim"] = json!(skip);
        b.sync_anims(&cx);
        let u = {
            let s = b.host.session.as_ref().unwrap();
            (0..s.w.units.len()).find(|&i| s.w.enemies.contains(&i) && s.w.units[i].is_alive() && s.w.grid.pos(i).is_some()).unwrap()
        };
        b.host.session.as_mut().unwrap().w.units[u].hp = 0;
        b.clock.ms = 2000.0;
        b.after_change(&cx);
        let geo = Geom::new(View::Stage, &cx.tuning.value);
        let (fx_, fy_) = geo.foot(b.host.session.as_ref().unwrap().w.grid.pos(u).unwrap());
        let dust = b.fx.live.iter().filter(|e| e.layer == Layer::Ground && e.start == 2000 && matches!(e.prim, Prim::Ring(r) if r.at == (fx_ as f32, fy_ as f32) && r.r1 == 10.0)).count();
        assert_eq!((b.impacts.len(), dust), (0, usize::from(!skip)), "skip {skip}: dust at once, none under skip");
    }
}
