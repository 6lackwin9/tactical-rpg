//! One frame of the battle screen (`draw`: the strip, party cards and hero panel, the board's call, hostile cards and intel, the bottom bar and log, the timeline view and talk), the panel portraits and the intel fit.

use aegis_ui::theme::th;
use super::board::{cell_rect, draw_flat, draw_stage, note_drawn, with_alpha, Board};
use super::input::{engage_positions, kind_name, pick_text, rest_text, skill_text, target_name, RowAct};
use super::marks::{arena_badge, arrows_of, broken_spans, draw_hold_chip, draw_hold_pick, draw_tracers, focus_set, held_guard, hit_forecasts, meter_row, meter_text, meters_of, party_foresight_off, pip_view, poise_shown, push_forecasts, push_text, seen_intents, HitForecast, HoldMarks, HookMarks, HELD, METER};
use super::overlays::{overlay_of, reach_text, Layers};
use super::playback::PlaySpeed;
use super::{Action, BattleScreen, ally, AP_PIPS, GRID_X, GRID_Y, PANEL, PANELS, PLAYBACK_SPEEDS, TACTIC_LABELS, TALK_VEIL, TILE, TIMELINE_SURFACE, TV_PAD};
use super::skill_menu::CHANNEL_TINT;
use super::tips::{action_channel_note, ap_blocks, ap_gauge_of, ap_held_text, channel_action, channel_block, channel_fill, downed_banner, planned_channel_note, class_names, committed, committed_icon_chains, derived_line, derived_rows, draw_block_label, dulled_line, dulled_weapons, inspect_tip, intent_glyph, kind_color, kind_icon, kind_line_color, next_action, short_name, stance_name, unit_tip, ApAnim, ApPips, BlockLook, DULLED_BADGE, REST_BLOCK};
use aegis_ui::canvas::{rgba, Rect, Rgba};
use crate::intent_text;
use crate::portraits::{Faces, Who};
use crate::talk;
use crate::timeline_view;
use crate::stage::{Geom, View};
use crate::screens::Cx;
use emberwake_battle::intent::Intent;
use aegis_ui::painter::Painter;
use aegis_ui::ui::{Gfx, Tip, Ui, BODY, H, RADIUS, SMALL, TITLE, W};
use emberwake_battle::grid::{V2, GRID_H, GRID_W};
use emberwake_battle::round_intent::{self as ri, RoundIntent};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::{Phase, Session};
use aegis_core::json::s;
use emberwake_battle::skill::Kind;
use emberwake_battle::world::World;
use serde_json::{json, Value};
use aegis_core::clock::Instant;

/// BATTLE-UI2 contract 2 (PR7-40 part 2): a tactic button's width (44 px), and Auto's.
pub(super) const TACTIC_W: i32 = 22;

/// Contract 2: a tactic button in `r`: the portraits of `faces` (the party's first two, or the selected hero), a gold
/// pennant at the top right and a chevron at the bottom right; its widget is the old dropdown's (`label` the tactic).
#[allow(clippy::too_many_arguments)]
fn tactic_button(ui: &mut Ui<Action>, cx: &mut Cx, small: &mut Faces, looks: &super::plates::Looks, id: &str, r: Rect, faces: &[usize], label: &str, action: Action, enabled: bool, tip: Tip) {
    let hover = enabled && ui.hover(r);
    ui.p.panel(r, if hover { th().card_hot } else { th().field }, if hover { th().focus } else { th().edge_live }, RADIUS);
    let side = r.h - 4;
    for (k, u) in faces.iter().enumerate().rev() {
        if let Some(l) = looks.get(*u) {
            small.draw(&mut ui.p, cx, &l.0, &l.1, super::TIMELINE_SURFACE, Rect::new(r.x + 2 + k as i32 * 4, r.y + 2, side, side));
        }
    }
    // the pennant: a mast and a gold flag
    let (px, py) = (r.x + r.w - 7, r.y + 2);
    ui.p.fill(Rect::new(px, py, 1, 8), th().text);
    for k in 0..3 {
        ui.p.fill(Rect::new(px + 1, py + k, 5 - k * 2 + 1, 1), th().gold);
        ui.p.fill(Rect::new(px + 1, py + 4 - k, 5 - k * 2 + 1, 1), th().gold);
    }
    let col = if enabled { th().text } else { th().dim };
    let (cxm, cym) = ((r.x + r.w - 4) as f32, (r.y + r.h - 3) as f32);
    ui.p.line(cxm - 2.0, cym - 1.0, cxm, cym + 1.0, col, 0.0);
    ui.p.line(cxm, cym + 1.0, cxm + 2.0, cym - 1.0, col, 0.0);
    if !enabled {
        ui.p.fill(r.inset(1), rgba(10, 11, 16, 110));
    }
    ui.push(id, "dropdown", label, r, enabled, false, Some(action), tip);
}

/// Contract 2: Auto as a play button in `r` (a triangle and an "AUTO" tag under it); lit while autoplay runs.
fn auto_button(ui: &mut Ui<Action>, r: Rect, enabled: bool, on: bool, tip: Tip) {
    let hover = enabled && ui.hover(r);
    let fill = if on { th().selected } else if hover { th().card_hot } else { th().card };
    ui.p.panel(r, fill, if on || hover { th().focus } else if enabled { th().edge_live } else { th().edge }, RADIUS);
    let col = if !enabled { th().dim } else if on { th().gold } else { th().text };
    let (tx, ty) = (r.x + r.w / 2 - 3, r.y + 2);
    for k in 0..4 {
        ui.p.fill(Rect::new(tx + k, ty + k, 1, 8 - 2 * k), col);
    }
    let lw = ui.p.text_w("AUTO", SMALL);
    ui.p.text(r.x as f32 + (r.w as f32 - lw) / 2.0, (r.y + r.h) as f32 - ui.p.line_h(SMALL) - 0.5, "AUTO", SMALL, if enabled { crate::tokens::planned() } else { th().dim });
    ui.push("strip.auto", "button", "Auto", r, enabled, on, Some(Action::Autoplay), tip);
}

/// FLOATPANELS1: the battle log's panel: the area it floats in (the board), its size (half the board's width, its
/// height), its name on the minimized button and its drag payload.
pub(super) const LOG_AREA: Rect = Rect::new(GRID_X, GRID_Y, GRID_W * TILE, GRID_H * TILE);
const LOG_SIZE: (i32, i32) = (GRID_W * TILE / 2 + 32, GRID_H * TILE - 8);
const LOG_NAME: &str = "Combat log";
const LOG_DRAG: &str = "battle.log";

pub(super) fn draw(g: &mut BattleScreen, cx: &mut Cx, gfx: &mut Gfx, mouse: (i32, i32), live: bool) {
    let now = Instant::now();
    g.keys = crate::controls_view::hints(&cx.settings.v);
    let tuning = cx.tuning.value.clone();
    gfx.canvas.clear(th().bg);
    // ENCOUNTER2-C DT7: the open talk's game state and view; a new node starts its line and replies at the top
    let talk_gs = g.talk_state_of(cx);
    let talk_view = talk_gs.as_ref().and_then(|gs| g.talk_view(cx, gs));
    let node = match (g.talk.as_ref(), talk_view.as_ref()) {
        (Some(t), Some(v)) => format!("{}/{}", t.dialogue, s(v, "node")),
        _ => String::new(),
    };
    talk::follow_node(gfx, &mut g.talk_node, node);
    g.faces.begin();
    // PORTRAITFX1-P: the panel portraits run on the FX clock (4.5)
    g.panel_faces.begin();
    g.panel_faces.clock = g.clock;
    g.timeline_faces.begin();
    g.timeline_faces.clock = g.clock;
    for f in [&mut g.card_faces, &mut g.mini_faces] {
        f.begin();
        f.clock = g.clock;
    }
    // DT8: the speaker's token wears the selection ring
    let talker = match (talk_gs.as_ref(), talk_view.as_ref()) {
        (Some(gs), Some(v)) => g.speaker_unit(cx, gs, &s(v, "speaker")),
        _ => None,
    };
    let mut ui: Ui<Action> = Ui::new(gfx, mouse, live);
    g.menu_row = ((ui.p.line_h(BODY) + 2.5).ceil() as i32).max(11);
    let menu_row = g.menu_row;
    if let Some(pop) = g.popup.as_mut() {
        // VISCLAR1-D: the skill menu beside its tile, in this frame's view of the board
        let geo = Geom::new(View::of_setting(&cx.settings.v["battle_view"]), &tuning);
        // PR7-59: on the board as the last frame shifted it under the boss bar
        let lift = g.board_lift;
        let lifted = |c: V2| {
            let r = geo.cell_rect(c);
            Rect::new(r.x, r.y + lift, r.w, r.h)
        };
        let tile = pop.anchor.map(lifted);
        // PR7-54 (FB-364 (1), contract 7): clear of the docked card or boss bar and the foe panel (last frame's
        // panels) and the menu's target tile
        let mut keep: Vec<Rect> = g.panels.iter().filter(|p| p.0 == "intel" || p.0 == "hostiles").map(|p| p.1).collect();
        if let (Some(s), Some((_, emberwake_battle::round_intent::RoundTarget::Unit(t)))) = (g.host.session.as_ref(), pop.menu) {
            keep.extend(s.w.grid.pos(t).filter(|c| Some(*c) != pop.anchor).map(lifted));
        }
        pop.fit_grid(&mut ui.p, menu_row, tile, Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4), &keep, MENU_LEFT);
    }
    let Some(s) = g.host.session.as_ref() else {
        ui.p.text(8.0, 8.0, &g.message, BODY, th().foe);
        g.w = ui.widgets;
        return;
    };
    let w = &s.w;
    let planning = s.phase == Phase::Planning && !s.plan_confirmed;
    let sel = g.selected.filter(|u| w.allies.contains(u));
    // INPUT1 PB1-PB5: the playback aids of a timeline battle; PB2: the block the playback resolves
    let aids = g.playback_aids();
    let cursor = g.cursor_block(now);
    // INPUT1 IS1: the inspected unit (the intel panel is pinned on it; its revealed moves show, AR3)
    let pinned = g.inspected();
    // INTENT1: the enemy tells while planning; their chips and arrows on the grid follow the Show enemy tells setting (I).
    // VISCLAR1 VC4: with the debug Party foresight off a foresight reveal reads unrevealed everywhere on the screen
    let foresight_off = party_foresight_off(&cx.settings.v, cfg!(debug_assertions));
    let tells: Vec<Intent> = seen_intents(w, s.enemy_intents(), foresight_off);
    let show_tells = cx.settings.v["show_tells"].as_bool().unwrap_or(true);
    // RULESET1-U slice 4 (UI5, V5): every unit's round intent record, both sides, planning and playback (empty without
    // the rule plan_badges: classic)
    let rounds: Vec<Intent> = seen_intents(w, s.round_intents(), foresight_off);
    // RULESET1-U slice 5: each overlay group behind its flag (settings `battle_overlays`; all on by default)
    let layers = Layers::of(&cx.settings.v["battle_overlays"]);
    // HOOKS1-UI slice 1: the ground hazards and the arena telegraph (empty in classic)
    let marks = HookMarks::of(s, &layers, cx.held.alt);
    // ENCOUNTER2-U: a hold escape's tiles and gate (H7; None without a hold escape)
    let hold = HoldMarks::of(s, &layers);
    // HOOKS1-UI slice 2 (CN2): each unit's class name in the battle's world, for the hover cards
    let classes = class_names(&cx.data, &g.class_world, w);
    let round_of = |u: usize| rounds.iter().find(|r| r.unit == u).and_then(|r| r.round_intent).filter(|_| layers.glyphs);

    // -- Top strip: the selected hero's six blocks and the round controls.
    g.panels = vec![("strip", Rect::new(2, 2, W - 4, 24)), ("grid", Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4)), ("bar", Rect::new(2, 322, W - 4, 36))];
    crate::hud::hud_panel(&mut ui, Rect::new(2, 2, W - 4, 24), th().game("battle_panel", PANEL), th().edge, RADIUS);
    // INPUT1 IF5: a hovered skill-menu row previews the round as if picked (copies of the world and the plan, kept for
    // the battle's revision); GH2 a: a member whose ghost waits for Confirm shows it from copies too
    let timeline = ri::on(w);
    let hovered_row = g.popup.as_ref().filter(|p| p.menu.is_some() && live).and_then(|p| p.row_at(g.menu_row, mouse));
    // VISCLAR1-D: a cell may act on another target than the menu's own
    let hovered_pick = g.popup.as_ref().and_then(|p| {
        let row = p.rows.get(hovered_row?)?.clone();
        let (h, at) = p.menu?;
        Some(((h, row.at.unwrap_or(at)), row))
    });
    if let Some(((h, at), row)) = hovered_pick.clone().filter(|_| planning) {
        let k = hovered_row.unwrap_or(0);
        if g.preview.as_ref().is_none_or(|p| (p.0, p.1) != (g.rev, k)) {
            let view = match &row.act {
                RowAct::Pick(picks) if row.valid => rip::preview_plan(w, &s.plan, h, at, picks).ok(),
                _ => None,
            };
            g.preview = Some((g.rev, k, view));
        }
    } else {
        g.preview = None;
    }
    let deferred_req = sel.and_then(|u| g.deferred.iter().find(|d| d.0 == u)).map(|d| (d.0, d.1.clone()));
    match (&deferred_req, planning) {
        (Some((m, change)), true) => {
            if g.deferred_view.as_ref().is_none_or(|v| (v.0, v.1) != (g.rev, *m)) {
                g.deferred_view = Some((g.rev, *m, change.view(s, *m)));
            }
        }
        _ => g.deferred_view = None,
    }
    let shown: Option<(&World, &emberwake_battle::plan::Plan)> = match (&g.preview, &g.deferred_view) {
        (Some((_, _, Some((pw, pp)))), _) => Some((pw, pp)),
        (_, Some((_, _, Some((pw, pp))))) => Some((pw, pp)),
        _ => None,
    };
    let (pw, pplan) = shown.unwrap_or((w, &s.plan));
    let previewing = shown.is_some();
    // INPUT1-AP (AP5, AP10): the selected party member's AP gauge, under the round intent only; while planning each
    // member's AP is kept for the playback's hollow pips
    if planning {
        g.round_ap = (s.current_round, w.allies.iter().map(|&a| (a, w.units[a].ap)).collect());
    }
    // VISCLAR1-C: the selected hero's round, compiled once a frame for the strip, the costs and the panel
    let compiled = sel.map(|u| pplan.compile(pw, u));
    // VISCLAR1-B: with each planned block its icon chain (a skill's family, then its kind)
    let (entries, icons): (Vec<(i32, i32, String, String, Option<String>, i32, i32)>, Vec<Vec<String>>) = match sel {
        Some(u) if s.plan_confirmed => committed(s, u).into_iter().map(|c| (c.0, c.1, c.2, c.3, c.4, 0, 0)).zip(committed_icon_chains(s, u)).unzip(),
        Some(u) if w.units[u].is_active() => compiled
            .iter()
            .flat_map(|st| st.entries.iter())
            .map(|e| {
                let sk = &w.cat.skills[e.skill];
                let chain = if e.item.is_some() { vec![kind_icon("ITEM")] } else { emberwake_content::art::skill_icon_chain(sk.family_id(), &sk.kind) };
                // PK1-S2c (PK1 4.3 item 6): a marrow version's name with the marrow mark
                let name = if e.item.is_some() { sk.name.clone() } else { skill_text(pw, u, e.skill) };
                ((e.start_block, e.end_block(pw), name, sk.kind.clone(), e.target.filter(|t| *t != u).map(|t| w.units[t].name.clone()), e.total_ap(), pplan.forecast_entry(pw, &s.resolver.scheduled, u, e)), chain)
            })
            .unzip(),
        _ => (Vec::new(), Vec::new()),
    };
    // PK1-S2c (PK1 4.3 item 6, user 2026-11-10): which spans run in the Channel round's mode (the placeholder tint)
    let channel_spans: Vec<bool> = match sel {
        Some(u) if s.plan_confirmed => s.resolver.scheduled.iter().filter(|a| a.unit == u).map(|a| channel_action(w, a)).collect(),
        Some(u) if w.units[u].is_active() => compiled.iter().flat_map(|st| st.entries.iter()).map(|e| e.item.is_none() && channel_block(pw, u, &w.cat.skills[e.skill])).collect(),
        _ => Vec::new(),
    };
    // BATTLE-UI2 contract 2 (PR7-40 part 2): each planned span's effect chips: a heal's moss heart, then each status
    // the move applies or grants (planning only)
    let effects: Vec<(bool, Vec<String>)> = match sel {
        Some(u) if timeline && planning && w.units[u].is_active() => compiled
            .iter()
            .flat_map(|st| st.entries.iter())
            .map(|e| {
                let sk = &w.cat.skills[e.skill];
                let statuses: Vec<String> = [sk.status_effect.clone(), sk.grant_status.clone()].into_iter().filter(|x| !x.is_empty()).fold(Vec::new(), |mut v, x| {
                    if !v.contains(&x) {
                        v.push(x);
                    }
                    v
                });
                (e.item.is_none() && emberwake_battle::skill::heals(sk), if e.item.is_none() { statuses } else { Vec::new() })
            })
            .collect(),
        _ => Vec::new(),
    };
    // PR5-26 (audit B8): each span's Channel line (the Channel Rest, a marrow version's terms) for its hover card
    let channel_notes: Vec<Option<String>> = match sel {
        Some(u) if s.plan_confirmed => s.resolver.scheduled.iter().filter(|a| a.unit == u).map(|a| action_channel_note(w, a)).collect(),
        Some(u) if w.units[u].is_active() => compiled.iter().flat_map(|st| st.entries.iter()).map(|e| e.item.is_none().then(|| planned_channel_note(pw, u, e.skill)).flatten()).collect(),
        _ => Vec::new(),
    };
    // RULESET1-U (intent UI2, PL6): under the round intent the planned blocks carry the planner's reason tags and the
    // Rest blocks the round's Rest gain; INPUT1 GH1: which are ghost (the planner's, after the picks)
    let (reasons, rest, ghosts) = match sel {
        // VISCLAR1-C (decision "Battle view answers, part 2"): the ghost shows in planning, not otherwise
        Some(u) if timeline && s.is_planning() && w.units[u].is_active() => (rip::reasons(pw, pplan, u), Some(rip::rest_preview(pw, pplan, u)).filter(|_| w.ruleset.rest_at_round_end()), rip::ghost_entries(pw, pplan, u)),
        _ => (Vec::new(), None, Vec::new()),
    };
    let gauge = match sel {
        Some(u) if timeline && w.units[u].is_active() => {
            let f = planning.then(|| rip::ap_forecast(pw, pplan, u));
            // INPUT2: one block a planned entry that costs AP, in its kind's colour (ghost entries as ghost blocks)
            let blocks = if planning { ap_blocks(&entries.iter().enumerate().map(|(k, e)| (e.5, e.3.clone(), ghosts.get(k).copied().unwrap_or(false))).collect::<Vec<_>>()) } else { Vec::new() };
            // AP3: a hovered row the hero cannot pay: the kit's shortfall for it (IF6 d)
            let short = hovered_pick.as_ref().filter(|((h, _), row)| planning && *h == u && !row.valid).and_then(|(_, row)| row.ap_short);
            let start = g.round_ap.1.iter().find(|r| r.0 == u && g.round_ap.0 == s.current_round).map_or(w.units[u].ap, |r| r.1);
            let mut gauge = ap_gauge_of(w, u, f.as_ref(), &blocks, short, start);
            // AP10: paid pips go hollow and regained ones fill over AP_ANIM_MS during playback and into the next
            // round's planning; changes while planning (picks, hovers) and the switch to playback show at once
            let target = (gauge.spent, gauge.edge);
            let instant = cx.settings.v["skip_anim"].as_bool().unwrap_or(false) || cx.settings.reduce_motion();
            let anim = match g.ap_anim {
                Some(a) if a.unit == u && a.to == target && a.planning == planning => a,
                Some(a) if a.unit == u && !instant && ((!planning && !a.planning) || (planning && a.round != s.current_round)) => ApAnim { unit: u, round: s.current_round, planning, from: a.level(now), to: target, at: now },
                _ => ApAnim { unit: u, round: s.current_round, planning, from: target, to: target, at: now },
            };
            g.ap_anim = Some(anim);
            (gauge.spent, gauge.edge) = anim.level(now);
            Some(gauge)
        }
        _ => None,
    };
    // RULESET1-U slice 5: each span's break, the planned hits' forecasts (the battle's own plan: a preview has none)
    let broken: Vec<bool> = match sel {
        Some(u) if timeline && s.plan_confirmed => broken_spans(s, u, &entries),
        _ => Vec::new(),
    };
    let hits: Vec<Option<HitForecast>> = match sel {
        Some(u) if timeline && layers.forecasts && !s.plan_confirmed && w.units[u].is_active() && !previewing => hit_forecasts(s, u),
        _ => Vec::new(),
    };
    // HOOKS1-UI slice 3 review: each span's HP cost from its own skill (the plan entry's) or, once confirmed, its command
    let costs: Vec<i32> = match sel {
        Some(u) if s.plan_confirmed => s.resolver.scheduled.iter().filter(|a| a.unit == u).map(|a| a.command.hp_cost).collect(),
        Some(u) if w.units[u].is_active() => compiled.iter().flat_map(|st| st.entries.iter()).map(|e| emberwake_battle::command::hp_cost_for(pw, u, &w.cat.skills[e.skill])).collect(),
        _ => Vec::new(),
    };
    // HOOKS1-UI slice 4: each planned entry's push and pull forecasts (rule forced_move; planning, layer `pushes`)
    let pushes: Vec<Vec<emberwake_battle::forced::Forecast>> = match sel {
        Some(u) if planning && layers.pushes && w.allies.contains(&u) && w.units[u].is_active() && !previewing => push_forecasts(s, u),
        _ => Vec::new(),
    };
    // BATTLE-UI2 (PR7-40): the view-model of the plates, the foe panel and the docked foe card (contract 2-6), and every
    // unit's portrait once a frame; the card's foe: the inspected one, else the hovered one (last frame's plates), else
    // the selected hero's target (the model), else the first foe on the field
    let looks: super::plates::Looks = (0..w.units.len()).map(|u| g.unit_portrait(cx, u)).collect();
    let hovered_plate = if live && g.popup.is_none() { g.w.iter().filter(|x| x.rect.contains(mouse.0, mouse.1)).find_map(|x| x.id.strip_prefix("foe.").and_then(|n| n.parse::<usize>().ok())) } else { None };
    let sel_target = sel.and_then(|h| w.round_states.get(&h)).and_then(|r| r.target).is_some_and(|t| matches!(t, ri::RoundTarget::Unit(u) if w.enemies.contains(&u)));
    let first_foe = w.enemies.iter().copied().find(|e| w.units[*e].is_active() && w.grid.pos(*e).is_some());
    // contract 5 (PR7-40 part 2): while a hero is planning, the card shows its target and stays there (a hovered plate or
    // a pin does not move it); else the pin, the hover, the first foe
    let card_foe = if planning && sel_target { None } else { pinned.filter(|p| w.enemies.contains(p)).or(hovered_plate).or(if sel_target { None } else { first_foe }) };
    let round_ap: Vec<(usize, i32)> = if g.round_ap.0 == s.current_round { g.round_ap.1.clone() } else { Vec::new() };
    let data = cx.data.clone();
    let world_id = g.class_world.clone();
    let plate_name = |u: usize| -> String {
        let unit = &w.units[u];
        if w.allies.contains(&u) {
            return unit.name.clone();
        }
        let short = w.cat.units.get(&unit.catalog_id).map_or("", |d| d.short_name.as_str());
        emberwake_data::plate_name(short, &unit.name).to_string()
    };
    let full_name = |u: usize| -> (String, String) {
        let unit = &w.units[u];
        let ch = if world_id.is_empty() { None } else { data.merged_entries(&world_id, "characters").into_iter().find(|c| aegis_core::json::s(c, "base_unit") == unit.catalog_id && aegis_core::json::s(c, "role") == "enemy") };
        match ch {
            Some(c) => (aegis_core::json::s_or(c, "name", &unit.name), aegis_core::json::s(c, "title")),
            None => (unit.name.clone(), String::new()),
        }
    };
    let intent_of = |u: usize| round_of(u).filter(|_| w.allies.contains(&u) || show_tells);
    let passive_about = |id: &str| -> String { aegis_core::json::s(data.entry("passives", id), "description") };
    let model = crate::battle_view::model(s, &crate::battle_view::Inputs { sel, card_foe, round_ap: &round_ap, plate_name: &plate_name, full_name: &full_name, intent_of: &intent_of, open_groups: &g.open_groups, pinned, passive_about: &passive_about });
    // BATTLE-UI2 contract 2: the selected hero's portrait, then "Round 1 of 8" in the display face's gold, the goal and
    // the protect line (gold) under it; UI5: the hero's intent glyph at the title's end
    let mut tx = 8;
    if let Some(l) = sel.and_then(|u| looks.get(u)) {
        let pr = Rect::new(5, 5, STRIP_PORTRAIT, STRIP_PORTRAIT);
        g.card_faces.draw(&mut ui.p, cx, &l.0, &l.1, super::PANEL_SURFACE, pr);
        ui.push("strip.portrait", "portrait", l.0.name.clone(), pr, true, false, None, Vec::new());
        tx = pr.x + pr.w + 3;
    }
    let title_w = match sel.and_then(round_of) {
        Some(i) => {
            intent_glyph(&mut ui.p, &w.cat, i, Rect::new(TITLE_END - 10, 4, 10, 10));
            (TITLE_END - 12 - tx) as f32
        }
        None => (TITLE_END - tx) as f32,
    };
    let round_text = ui.p.fit(&model.top.round, BODY, title_w);
    ui.p.text_face(aegis_ui::painter::Typeface::Display, tx as f32, 3.0, &round_text, BODY, th().gold);
    // PR7-63: a title cut to its room hovers to the whole of it
    let title_tip: Tip = if round_text != model.top.round { vec![(model.top.round.clone(), th().gold)] } else { Vec::new() };
    ui.push("strip.title", "label", model.top.round.clone(), Rect::new(tx, 3, title_w as i32, ui.p.line_h(BODY).ceil() as i32), true, false, None, title_tip);
    let line_room = (TITLE_END - tx) as f32;
    // PR7-63 (XL-BATTLE1): the goal and protect lines stack under the title by the text size's line heights (at Normal
    // the old 11 and 6 px), so at Extra large they no longer run into the title or each other
    let (goal_y, line_step) = strip_lines(ui.p.line_h(BODY), ui.p.line_h(SMALL));
    for (k, (id, text, colour)) in [("strip.goal", &model.top.goal, th().text), ("strip.protect", &model.top.protect, crate::tokens::boss_gold())].into_iter().enumerate() {
        if text.is_empty() {
            continue;
        }
        let ly = goal_y + line_step * k as i32;
        ui.p.text_fit(tx as f32, ly as f32, text, SMALL, line_room, colour);
        // OBJUI1 (PR7-31; OBJ1 rule 9, MORALE1 3): the goal hovers to the full objective line and the morale line
        let tip: Tip = if id == "strip.goal" { [(&model.top.objective, th().text), (&model.top.morale, th().dim)].into_iter().filter(|(t, _)| !t.is_empty()).map(|(t, c)| (t.clone(), c)).collect() } else { Vec::new() };
        ui.push(id, "label", text.clone(), Rect::new(tx, ly, line_room as i32, ui.p.line_h(SMALL).ceil() as i32), true, false, None, tip);
    }
    // HOOKS1-UI slice 3 (UM7): the selected hero's meters on a thin row at the strip's foot
    if let Some(u) = sel {
        let ms = meters_of(w, u, &layers);
        if !ms.is_empty() {
            let mr = Rect::new(tx, 23, TITLE_END - tx, 2);
            meter_row(&mut ui.p, mr, &ms);
            let tip: Tip = ms.iter().map(|m| (meter_text(m), METER)).collect();
            ui.push("strip.meters", "meters", tip.iter().map(|t| t.0.clone()).collect::<Vec<_>>().join("; "), Rect::new(mr.x, mr.y - 1, mr.w, mr.h + 2), true, false, None, tip);
        }
    }
    // TEXTFIT1: the controls right of the blocks take their text's width; the blocks give up width first (34 down to
    // 26 each), then the two dropdowns, the wider first (their text cuts).
    let party_label = format!("Party: {}", TACTIC_LABELS[g.party_tactic]);
    let hero_label = format!("Hero: {}", TACTIC_LABELS[sel.map_or(g.hero_tactic, |u| if timeline { g.tactic_of(u) } else { g.hero_tactic })]);
    // ENCOUNTER2-C4: a locked round still to play: Skip in Confirm's place (the rest of the round at once)
    let skip = g.can_skip();
    // PR7-41 (PT-32): while a talk is up (the intro's too) only its box takes clicks, so the strip reads disabled
    let open = !g.talk_due();
    let (label, action, enabled) = if planning {
        // ENCOUNTER2 H1: planning cannot be confirmed before the holder is picked
        ("Confirm round", Action::Confirm, !s.needs_stay_pick())
    } else if skip {
        ("Skip", Action::Skip, true)
    } else if s.phase == Phase::Replay && s.can_advance_round() {
        // VISCLAR1-D: under the round intent the round-end pause can be cut short (`run` ends the wait)
        ("Next round", Action::NextRound, timeline || !g.pending_next_round)
    } else {
        (if s.combat_over { "Battle over" } else if s.phase == Phase::Replay { "Round limit" } else { "Resolving" }, Action::NextRound, false)
    };
    let mut fit_w = |text: &str, pad: f32, min: i32| ((ui.p.text_w(text, BODY) + pad).ceil() as i32).max(min);
    // BATTLE-UI2 contract 2 (PR7-40 part 2): under the round intent the tactics are portrait-and-pennant buttons and
    // Auto a play button with its "AUTO" tag (classic keeps the text menus)
    // during playback the speed chips take the tactics' place at their old widths
    let buttons = timeline && !(aids && !planning);
    let (mut party_w, mut hero_w) = if buttons { (TACTIC_W, TACTIC_W) } else { (fit_w(&party_label, 18.0, 82), fit_w(&hero_label, 18.0, 80)) };
    let (auto_w, clear_w, confirm_w) = (if timeline { TACTIC_W } else { fit_w("Auto", 10.0, 34) }, fit_w("Clear", 10.0, 34), fit_w(label, 12.0, 60));
    let bx = 92;
    let blocks = s.def.total_blocks.max(1);
    // INPUT1-AP (AP6): under the round intent the blocks leave the AP gauge its 12 pips and a label ("-<n>") at their
    // end, kept whether or not a hero is selected, so the strip does not shift
    let gauge_w = if timeline { rpgkit_ui::pips::pips_w(AP_PIPS) + rpgkit_ui::pips::LABEL_GAP + rpgkit_ui::pips::label_w(&mut ui, "-00") } else { 0 };
    let room = W - 6 - bx - 6 - 8 - auto_w - clear_w - confirm_w - if timeline { gauge_w + 2 } else { 0 };
    // AP6: with the gauge the blocks give up width down to 20 each (their span labels cut, TEXTFIT1) before the menus
    let bw = ((room - party_w - hero_w) / blocks).clamp(if timeline { 20 } else { 26 }, 34);
    let mut over = blocks * bw + party_w + hero_w - room;
    while over > 0 && !buttons && (party_w > 82 || hero_w > 80) {
        let (big, floor) = if party_w - 82 >= hero_w - 80 { (&mut party_w, 82) } else { (&mut hero_w, 80) };
        let cut = over.min(*big - floor);
        *big -= cut;
        over -= cut;
    }
    // INPUT1 PB2: a timeline battle's strip follows the playback's resolving block (classic: the current block)
    let live_block = if aids { cursor.unwrap_or(0) } else if !planning && s.phase == Phase::Resolving { s.current_block } else { 0 };
    // STATUSVIS1 SV5, SV15 (PR7-57): the selected hero's locked blocks, by run (stun one tile over blocks 1-2, stagger
    // a tile on 1 and one on 4): a grey tile with the status icon in place of the lock's entries, no plan there
    let locks: Vec<emberwake_battle::status_view::LockRun> = match sel {
        Some(u) if timeline && w.units[u].is_active() => emberwake_battle::status_view::lock_runs(w, u),
        _ => Vec::new(),
    };
    let locked = |b: i32| emberwake_battle::status_view::run_at(&locks, b);
    for b in 1..=s.def.total_blocks {
        let r = Rect::new(bx + (b - 1) * bw, 6, bw - 1, 16);
        let at = entries.iter().position(|e| b >= e.0 && b <= e.1);
        let entry = at.map(|i| &entries[i]);
        let mut tip: Tip = Vec::new();
        let ghost = at.and_then(|i| ghosts.get(i)).copied().unwrap_or(false);
        let why = at.and_then(|i| reasons.get(i)).filter(|r| !r.is_empty());
        match entry {
            Some(e) => {
                // INPUT1 GH8: a ghost block's tip starts "Auto-filled" with its reason tag
                if ghost {
                    tip.push((why.map_or_else(|| "Auto-filled".to_string(), |r| format!("Auto-filled: {r}")), th().dim));
                }
                tip.push((e.2.clone(), th().gold));
                tip.push((format!("Blocks {}-{}", e.0, e.1), th().text));
                if let Some(t) = &e.4 {
                    tip.push((format!("Target: {t}"), th().foe));
                }
                if e.5 > 0 {
                    tip.push((format!("{} AP", e.5), ally()));
                }
                if e.6 > 0 {
                    tip.push((format!("Expected damage about {}", e.6), th().gold));
                }
                // HOOKS1-UI slice 3 (HC1): the step's HP cost
                let cost = at.and_then(|i| costs.get(i)).copied().unwrap_or(0);
                if cost > 0 {
                    tip.push((intent_text::hp_cost_text(cost), th().foe));
                }
                if let Some(why) = why.filter(|_| !ghost) {
                    tip.push((format!("Why: {why}"), th().dim));
                }
                if let Some(note) = at.and_then(|i| channel_notes.get(i)).and_then(|n| n.as_ref()) {
                    tip.push((note.clone(), CHANNEL_TINT));
                }
                if let Some(gain) = rest.filter(|r| e.3 == "REST" && r.blocks > 0) {
                    tip.push((format!("{} ({} Rest block{}; the AP comes at round end)", rest_text(gain), gain.blocks, if gain.blocks == 1 { "" } else { "s" }), th().good));
                }
                for f in at.and_then(|i| pushes.get(i)).into_iter().flatten() {
                    tip.push(push_text(w, f));
                }
                if let Some(f) = at.and_then(|i| hits.get(i)).and_then(|f| f.as_ref()) {
                    tip.extend(f.lines(w));
                }
                if at.and_then(|i| broken.get(i)).copied().unwrap_or(false) {
                    tip.push(("BREAK: cancelled by a poise break".into(), th().foe));
                }
            }
            None => {
                ui.p.panel(r, th().game("strip_cell", rgba(14, 20, 34, 255)), if live_block == b { th().gold } else { th().edge }, RADIUS);
                ui.p.text(r.x as f32 + 3.0, r.y as f32 + 4.0, &format!("{b} Idle"), SMALL, th().dim);
                tip.push((format!("Block {b}: nothing planned"), th().dim));
            }
        }
        // RULESET1-U slice 6 (TEXTFIT1): a span's first block holds its whole span, so its label (drawn across the span)
        // sits in its own widget; the later blocks, pushed after it, keep their own hover cards
        let wr = match entry {
            Some(e) if e.0 == b => Rect::new(r.x, r.y, (e.1 - e.0 + 1) * bw - 1, r.h),
            _ => r,
        };
        // HOOKS1-UI slice 1 (AH3): the arena hazards that strike at the block's end
        tip.extend(marks.block_tip(b));
        // INPUT2: a block showing a ghost entry of a party member (planning, no row previewed) accepts the ghost up to it
        let accept = at.filter(|_| ghost && planning && hovered_pick.is_none() && sel.is_some_and(|u| w.allies.contains(&u)));
        if accept.is_some() {
            tip.push((format!("Click: accept the auto-fill up to here ({}: all of it)", g.keys.key_hint(aegis_ui::screen::Key::Char('f'))), th().dim));
        }
        // BATTLE-UI2 (battle contract 7, FB-358): an empty or a picked block opens the skill menu under it; a ghost block
        // opens it from a right click or the menu key (Tab, pad north)
        let menu = timeline && planning && accept.is_none() && open && sel.is_some_and(|u| w.allies.contains(&u) && w.units[u].is_active()) && locked(b).is_none();
        if menu {
            tip.push(("Click: the skill menu for this block".into(), th().dim));
        } else if accept.is_some() {
            tip.push((format!("Right click or {}: the skill menu", g.keys.hint(aegis_ui::screen::Key::Tab)), th().dim));
        }
        let action = accept.map(Action::GhostSlot).or(menu.then_some(Action::StripMenu(b)));
        // SV5: a locked block's hover names the lock and its blocks; the run's first block holds the whole run
        let (wr, tip) = match locked(b) {
            Some(run) => (if run.start == b { Rect::new(r.x, r.y, (run.end - run.start + 1) * bw - 1, r.h) } else { r }, super::statusvis::lock_tip(run)),
            None => (wr, tip),
        };
        ui.push(format!("strip.block{b}"), "block", entry.map(|e| e.2.clone()).unwrap_or_else(|| "empty".into()), wr, true, live_block == b, action, tip);
    }
    // Each action is one continuous span over its blocks, coloured by kind (COMBAT-UI3).
    let mut strip_labels = Vec::new();
    for (k, e) in entries.iter().enumerate() {
        let r = Rect::new(bx + (e.0 - 1) * bw, 6, (e.1 - e.0 + 1) * bw - 1, 16);
        if locked(e.0).is_some() {
            continue;
        }
        let live = live_block >= e.0 && live_block <= e.1;
        // VISCLAR1-D (mock `rest_colour`): under the round intent a Rest block is light slate (the planner fills every spare
        // block with them, so it has to stand out as time not used)
        let rest = timeline && e.3 == "REST";
        // INPUT1 GH8: a ghost span (the planner's, after the picks) in the engine's ghost look
        let ghost = ghosts.get(k).copied().unwrap_or(false) && !rest;
        // VISCLAR1-C (decision "Battle view answers, part 2"): one solid colour a block, no wind-up or impact shading (the
        // full timeline view keeps its parts); an auto-filled block fades, with no outline
        // PK1-S2c (placeholder, user 2026-11-10): a span in the Channel round's mode is tinted violet with a violet edge
        let channel = channel_spans.get(k).copied().unwrap_or(false);
        let base = if channel { channel_fill(kind_color(&e.3)) } else { kind_color(&e.3) };
        let fill = if rest { REST_BLOCK } else if ghost { aegis_ui::timeline::ghost_fill(base) } else { base };
        let edge = if live { th().gold } else if channel { CHANNEL_TINT } else if ghost { rgba(0, 0, 0, 0) } else if timeline { fill } else { rgba(255, 255, 255, 50) };
        ui.p.panel(r, fill, edge, RADIUS);
        if timeline {
            // VISCLAR1-B (mock C): the skill's icon and name centred, the icon alone when the name does not fit
            let look = if rest { BlockLook::Rest } else if ghost { BlockLook::Ghost } else { BlockLook::Chosen };
            let mut label = draw_block_label(&mut ui, &mut cx.art, r, &e.2, icons.get(k).map_or(&[], Vec::as_slice), look, kind_line_color(&e.3));
            if channel {
                label["channel"] = json!(true);
            }
            strip_labels.push(label);
        } else {
            ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 4.0, &e.2, SMALL, r.w as f32 - 6.0, th().text);
        }
        if broken.get(k).copied().unwrap_or(false) && layers.breaks {
            let mark = aegis_ui::timeline::Mark { key: "break".into(), block: e.0, glyph: "BREAK".into(), colour: th().foe, tip: vec![("BREAK".into(), th().foe), (format!("{}: cancelled by a poise break (AP paid stays spent)", e.2), th().text)] };
            let mr = Rect::new(r.x + 1, 7, (bw - 3).min(r.w - 2), 14);
            aegis_ui::timeline::draw_mark(&mut ui, &format!("strip.mark.{k}"), &mark, mr);
        }
    }
    for run in &locks {
        let r = Rect::new(bx + (run.start - 1) * bw, 6, (run.end - run.start + 1) * bw - 1, 16);
        super::statusvis::lock_tile(&mut ui, cx, r, &run.status);
        if live_block >= run.start && live_block <= run.end {
            ui.p.panel(r, rgba(0, 0, 0, 0), th().gold, 0.0);
        }
        strip_labels.push(json!({"locked": run.status, "blocks": [run.start, run.end]}));
    }
    g.strip_labels = strip_labels;
    // VISCLAR1-D: a Move that replaced a larger block says so, under the blocks it left as Rest
    // (drawn after the panels, as it lies over the board's top edge)
    // and while the Move waits for its tile, what the tile under the mouse takes
    let aimed = aegis_ui::ui::hit(&g.w, mouse).and_then(|a| if let Action::TileClick(c) = a { g.move_tiles.iter().find(|t| t.cell == c).cloned() } else { None }).filter(|_| g.move_aim.is_some());
    let aim_note = aimed.map(|t| {
        let drops = if t.drops.is_empty() { String::new() } else { format!(", drops {}", t.drops) };
        (format!("Move here: {} block{}, {} AP{drops}", t.blocks, if t.blocks == 1 { "" } else { "s" }, t.ap), bx)
    });
    let move_note = aim_note.or(sel.filter(|_| timeline && planning).and_then(|u| rip::move_note(pplan, u)).map(|note| (note, entries.iter().find(|e| e.3 == "REST").map_or(bx, |e| bx + (e.0 - 1) * bw))));
    // HOOKS1-UI slice 1 (AH3): a badge on each block at whose end an arena hazard strikes, over the spans
    for b in (1..=s.def.total_blocks).filter(|b| marks.show_arena && marks.arena.iter().any(|t| t.blocks.contains(b))) {
        arena_badge(&mut ui.p, Rect::new(bx + (b - 1) * bw, 6, bw - 1, 16));
    }
    // BATTLE-UI2 contract 2: each planned move's AP pips under its icon (dim on a ghost), and the block ruler under the
    // spans, red on each block a foe's blow lands on the selected hero (only the ruler, not the tile; take 17)
    if timeline {
        for (k, e) in entries.iter().enumerate().filter(|(_, e)| e.5 > 0) {
            let r = Rect::new(bx + (e.0 - 1) * bw, 6, (e.1 - e.0 + 1) * bw - 1, 16);
            let ghost = ghosts.get(k).copied().unwrap_or(false);
            let n = e.5.min((r.w - 2) / 3).max(1);
            let x0 = r.x + (r.w - (n * 3 - 1)) / 2;
            let mut pip = crate::tokens::planned();
            if ghost {
                pip[3] = 110;
            }
            for i in 0..n {
                ui.p.fill(Rect::new(x0 + i * 3, r.y + r.h - 3, 2, 2), pip);
            }
        }
        ui.p.fill(Rect::new(bx, 23, blocks * bw - 1, 1), rgba(255, 255, 255, 40));
        for b in model.top.impacts.iter().filter(|b| (1..=blocks).contains(*b)) {
            ui.p.fill(Rect::new(bx + (b - 1) * bw, 22, bw - 1, 2), th().foe);
        }
        // PR7-40 part 2: the selected tile's 2 px gold rim: the hero's latest pick (the one Undo takes back)
        let picked = if planning { entries.iter().enumerate().rev().find(|(k, e)| !ghosts.get(*k).copied().unwrap_or(false) && e.3 != "REST" && locked(e.0).is_none()).map(|(k, _)| k) } else { None };
        if let Some(e) = picked.and_then(|k| entries.get(k)) {
            let r = Rect::new(bx + (e.0 - 1) * bw, 6, (e.1 - e.0 + 1) * bw - 1, 16);
            for edge in [Rect::new(r.x, r.y, r.w, 1), Rect::new(r.x, r.y + r.h - 1, r.w, 1), Rect::new(r.x, r.y, 1, r.h), Rect::new(r.x + r.w - 1, r.y, 1, r.h)] {
                ui.p.fill(edge, th().gold);
            }
        }
        // contract 2: the effect chips on each span's top edge, centred, half above it (a heal's moss heart, then a chip
        // per status the move applies)
        for (k, e) in entries.iter().enumerate() {
            let Some((heal, statuses)) = effects.get(k) else { continue };
            let n = *heal as i32 + statuses.len() as i32;
            if n == 0 {
                continue;
            }
            let r = Rect::new(bx + (e.0 - 1) * bw, 6, (e.1 - e.0 + 1) * bw - 1, 16);
            const CHIP: i32 = 6;
            let shown = n.min(((r.w + 1) / (CHIP + 1)).max(1));
            let mut cx0 = r.x + (r.w - (shown * (CHIP + 1) - 1)) / 2;
            let cy = r.y - CHIP / 2;
            let mut tip: Tip = vec![(e.2.clone(), th().gold)];
            let mut left = shown;
            if *heal {
                let cr = Rect::new(cx0, cy, CHIP, CHIP);
                ui.p.fill(cr, rgba(10, 12, 16, 235));
                super::plates::heal_heart(&mut ui, cx, cr.inset(1));
                tip.push(("Heals".into(), crate::tokens::moss()));
                cx0 += CHIP + 1;
                left -= 1;
            }
            for st in statuses.iter().take(left.max(0) as usize) {
                let cr = Rect::new(cx0, cy, CHIP, CHIP);
                let rim = if emberwake_rules::status::is_buff(st) { th().gold } else { crate::tokens::oxblood() };
                ui.p.panel(cr, rgba(10, 12, 16, 235), rim, 0.0);
                if !super::plates::icon(&mut ui, cx, &emberwake_content::art::status_icon(st), cr.inset(1), 255) {
                    ui.p.fill(cr.inset(2), rim);
                }
                cx0 += CHIP + 1;
            }
            tip.extend(statuses.iter().map(|st| (format!("Applies {}", emberwake_rules::status::label_with_magnitude(st, 0)), th().text)));
            ui.push(format!("strip.effects.{k}"), "badge", tip.iter().skip(1).map(|t| t.0.clone()).collect::<Vec<_>>().join("; "), Rect::new(r.x + (r.w - (shown * (CHIP + 1) - 1)) / 2, cy, shown * (CHIP + 1) - 1, CHIP), true, false, None, tip);
        }
    }
    // INPUT1 PB2: a 1 px GOLD frame around the resolving block's column, over the spans
    if let Some(b) = cursor.filter(|b| (1..=blocks).contains(b)) {
        let c = Rect::new(bx + (b - 1) * bw - 1, 5, bw + 1, 18);
        for edge in [Rect::new(c.x, c.y, c.w, 1), Rect::new(c.x, c.y + c.h - 1, c.w, 1), Rect::new(c.x, c.y, 1, c.h), Rect::new(c.x + c.w - 1, c.y, 1, c.h)] {
            ui.p.fill(edge, th().gold);
        }
    }
    let mut x = bx + blocks * bw + 6;
    // AP6: the gauge (`strip.ap`) after the last block, before the tactic menus, centred on the strip lane
    if timeline {
        let gx = bx + blocks * bw + 4;
        g.ap_gauge = gauge.map(|ga| {
            ga.draw(&mut ui, "strip.ap", gx, 14);
            // BATTLE-UI2 contract 2: the AP line under the pips ("AP 10/12 -6") and the ghost's cost in grey
            if sel.is_some() {
                let line = model.top.ap_text();
                let lw = ui.p.text_w(&line, SMALL).min(gauge_w as f32);
                ui.p.text_fit(gx as f32, 19.0, &line, SMALL, gauge_w as f32, th().text);
                ui.push("strip.ap_line", "label", line, Rect::new(gx, 19, lw.ceil() as i32 + 1, ui.p.line_h(SMALL).ceil() as i32), true, false, None, Vec::new());
                let ghost = model.top.ghost_text();
                if !ghost.is_empty() && lw + 2.0 + ui.p.text_w(&ghost, SMALL) <= gauge_w as f32 {
                    ui.p.text(gx as f32 + lw + 2.0, 19.0, &ghost, SMALL, th().dim);
                }
            }
            let label = ga.before.clone().map(|l| (l, "before")).or(ga.after.clone().map(|l| (l, "after")));
            json!({"unit": sel, "pips": ga.reading(), "red": ga.alert, "label": label.as_ref().map(|l| l.0.clone()), "label_side": label.map(|l| l.1), "per_pip": ga.unit, "slots": ga.slots})
        });
        x = gx + gauge_w + 4;
    } else {
        g.ap_gauge = None;
    }
    if aids && !planning {
        // INPUT1 PB3: during playback the speed chips take the tactic menus' place (1x, 0.5x, Block; Next block in
        // Block mode)
        let end = x + party_w + 2 + hero_w;
        let speed = PlaySpeed::of(&cx.settings.v);
        let tips = [
            "Playback at full speed (P cycles the speeds).",
            "Half speed: every walk, clip, float, tracer and pause takes twice as long (P cycles).",
            "Block by block: the playback waits after each block; N or Next block plays the next one (P cycles).",
        ];
        for (k, (_, label, id)) in PLAYBACK_SPEEDS.iter().enumerate() {
            let cw = ((ui.p.text_w(label, BODY) + 10.0).ceil() as i32).max(18);
            ui.button_full(id, Rect::new(x, 6, cw, 16), label, Action::PlaybackSpeed(k), true, speed.index() == k, BODY, vec![(format!("Playback speed: {label}"), th().gold), (tips[k].into(), th().text)]);
            x += cw + 2;
        }
        if speed == PlaySpeed::Block {
            ui.button_full("playback.next", Rect::new(x, 6, (end - x).max(10), 16), "Next block", Action::NextBlock, g.waiting.is_some(), false, BODY, vec![(format!("Next block ({})", g.keys.hint(aegis_ui::screen::Key::Char('n'))), th().gold), ("Plays the next block, then waits again.".into(), th().text)]);
        }
        x = end + 2;
    } else {
        let party_tip: Tip = vec![("Party tactic".into(), th().gold), (party_label.clone(), th().text), ("AI-drafts every hero with a tactic (you can edit the steps after).".into(), th().text)];
        let hero_tip: Tip = vec![("Hero tactic".into(), th().gold), (hero_label.clone(), th().text), ("AI-drafts the selected hero only.".into(), th().text)];
        if timeline {
            let party_faces: Vec<usize> = model.party.iter().map(|p| p.unit).take(2).collect();
            tactic_button(&mut ui, cx, &mut g.mini_faces, &looks, "strip.party", Rect::new(x, 6, party_w, 16), &party_faces, &party_label, Action::PartyMenu, planning && open, party_tip);
            x += party_w + 2;
            tactic_button(&mut ui, cx, &mut g.mini_faces, &looks, "strip.hero", Rect::new(x, 6, hero_w, 16), &sel.into_iter().collect::<Vec<_>>(), &hero_label, Action::HeroMenu, planning && sel.is_some() && open, hero_tip);
        } else {
            ui.dropdown("strip.party", Rect::new(x, 6, party_w, 16), &party_label, Action::PartyMenu, planning && open, party_tip);
            x += party_w + 2;
            ui.dropdown("strip.hero", Rect::new(x, 6, hero_w, 16), &hero_label, Action::HeroMenu, planning && sel.is_some() && open, hero_tip);
        }
        x += hero_w + 2;
    }
    let auto_tip: Tip = vec![("Autoplay".into(), th().gold), ("Drafts with the party tactic, confirms and plays rounds until the battle ends. Click again to stop.".into(), th().text)];
    if timeline {
        auto_button(&mut ui, Rect::new(x, 6, auto_w, 16), !s.combat_over && open, g.auto, auto_tip);
    } else {
        ui.button_full("strip.auto", Rect::new(x, 6, auto_w, 16), "Auto", Action::Autoplay, !s.combat_over && open, g.auto, BODY, auto_tip);
    }
    x += auto_w + 2;
    ui.button_full("strip.clear", Rect::new(x, 6, clear_w, 16), "Clear", Action::Clear, planning && sel.is_some() && open, false, BODY, vec![(if timeline { "Clears the selected hero's picks (the planner fills the round again); with no picks, empties its auto-filled round (the hero rests)." } else { "Clears the selected hero's steps." }.into(), th().text)]);
    x += clear_w + 2;
    if skip {
        ui.button_full("battle.skip", Rect::new(x, 6, W - 6 - x, 16), label, action, enabled && open, true, BODY, vec![("Skip".into(), th().gold), ("Resolves the rest of this scripted round at once.".into(), th().text)]);
    } else {
        ui.button_full("strip.confirm", Rect::new(x, 6, W - 6 - x, 16), label, action, enabled && open, planning, BODY, vec![(label.into(), th().gold)]);
        // BATTLE-UI2 contract 2: Confirm is the one amber button
        if planning && enabled && open {
            ui.p.panel(Rect::new(x, 6, W - 6 - x, 16), rgba(0, 0, 0, 0), crate::tokens::planned(), RADIUS);
        }
    }

    // -- Left: the party plates, the guests and the selected hero (BATTLE-UI2 contract 3: 184 px wide, 176 px plates).
    crate::hud::hud_panel(&mut ui, Rect::new(2, 28, SIDE_W, 290), th().game("battle_panel", PANEL), th().edge, RADIUS);
    // PR7-37 (PT-02): the heroes still standing: on their feet, on the field or gone out by an exit (not downed)
    let alive = w.allies.iter().filter(|a| w.units[**a].is_alive()).count();
    ui.p.text(8.0, 32.0, &format!("Party {alive}/{}", w.allies.len()), TITLE, ally());
    let mut y = 42;
    // UI5: the party member the mouse is over (its card or its token) fills the threat intel panel
    let mut hovered_ally = None;
    // VISCLAR1-D: the cards' AP pips run on the FX clock (the agent link's `party_cards`)
    let (fx_now, pips_still) = (g.clock.now(), cx.settings.v["skip_anim"].as_bool().unwrap_or(false) || cx.settings.reduce_motion());
    let mut cards_shown = Vec::with_capacity(w.allies.len());
    let px = 2 + (SIDE_W - super::plates::PLATE_W) / 2;
    for (k, plate) in model.party.iter().chain(model.guests.iter()).enumerate() {
        let a = plate.unit;
        let u = &w.units[a];
        let guest = k >= model.party.len();
        if guest && k == model.party.len() {
            // contract 3: the guests under a teal divider
            ui.p.fill(Rect::new(px, y + 1, super::plates::PLATE_W, 1), crate::tokens::guest());
            ui.p.text(px as f32, y as f32 + 2.0, "Guests", SMALL, crate::tokens::guest());
            y += 10;
        }
        let r = Rect::new(px, y, super::plates::PLATE_W, if guest { super::plates::COMPACT_H } else { super::plates::PLATE_H });
        if ui.hover(r) {
            hovered_ally = Some(a);
        }
        let look = &looks[a];
        let first_widget = ui.widgets.len();
        if guest {
            super::plates::guest_plate(&mut ui, cx, &mut g.panel_faces, w, look, plate, r);
        } else {
            super::plates::party_plate(&mut ui, cx, &mut g.panel_faces, w, look, plate, r);
        }
        // PR7-3 (review N3): a dulled weapon's badge on the plate's top right, its tip the penalty
        let dulled = dulled_weapons(u);
        if !dulled.is_empty() {
            let bw = (ui.p.text_w(DULLED_BADGE, SMALL) + 6.0).ceil() as i32;
            let br = Rect::new(r.x + 2, r.y + r.h - 12, bw, 9);
            ui.p.panel(br, with_alpha(th().foe, 60), th().foe, 2.0);
            ui.p.text(br.x as f32 + 3.0, br.y as f32 + (br.h as f32 - ui.p.line_h(SMALL)) / 2.0, DULLED_BADGE, SMALL, th().foe);
            let tip: Vec<(String, Rgba)> = std::iter::once(("Weapon dulled".to_string(), th().foe)).chain(dulled.iter().map(|x| (dulled_line(x), th().text))).collect();
            ui.push(format!("party.{a}.dulled"), "badge", dulled.iter().map(|x| dulled_line(x)).collect::<Vec<_>>().join("; "), br, true, false, None, tip);
        }
        if let Some((banner, bcol)) = downed_banner(w, a) {
            ui.p.text_fit(r.x as f32 + 28.0, r.y as f32 + 25.0, &banner, SMALL, (r.w - 44) as f32, bcol);
            cards_shown.push(json!({"unit": a, "ap": u.ap, "ap_shown": 0.0, "poise_pips": Option::<i32>::None, "poise": Option::<i32>::None}));
        } else {
            let poise = poise_shown(w, a, &layers);
            let k = g.card_ap.iter().position(|c| c.0 == a).unwrap_or_else(|| {
                g.card_ap.push((a, ApPips::still(u.ap as f32, fx_now)));
                g.card_ap.len() - 1
            });
            g.card_ap[k].1 = g.card_ap[k].1.follow(u.ap as f32, fx_now, pips_still);
            let level = g.card_ap[k].1.level(fx_now);
            cards_shown.push(json!({"unit": a, "ap": u.ap, "ap_shown": level, "ap_left": plate.ap.map(|x| x.left()), "poise_pips": poise.as_ref().map(|st| pip_view(&w.ruleset.numbers, st).lit), "poise": poise.as_ref().map(|st| st.value)}));
        }
        meter_row(&mut ui.p, Rect::new(r.x + 3, r.y + r.h - 7, r.w - 18, 2), &meters_of(w, a, &layers));
        if layers.guard && held_guard(s, a).is_some() {
            ui.p.panel(r.inset(1), rgba(0, 0, 0, 0), HELD, RADIUS);
        }
        let tip = unit_tip(s, a, sel, &tells, &rounds, &classes);
        ui.push(format!("party.{a}"), "card", format!("{} HP {}/{} {}", u.name, u.hp, u.max_hp, crate::intent_text::ap_text(u.ap, u.max_ap, "/")), r, true, plate.selected, Some(Action::Select(a)), tip);
        super::statusvis::lift_plate_info(&mut ui, first_widget, &format!("party.{a}"));
        y += r.h + super::plates::PLATE_GAP;
    }
    g.cards_shown = cards_shown;
    g.panels.push(("party", Rect::new(2, 28, SIDE_W, y - 26)));
    // BATTLE-UI2: the selected hero's panel keeps the room under the plates (its Self and Ally buttons, its plan lines)
    // (a short room keeps only the Self and Ally buttons, under the round intent)
    let hero_room = 318 - y - 6;
    if let Some(h) = sel.filter(|&h| timeline && w.units.get(h).is_some_and(|u| u.is_active()) && (HERO_BUTTONS_H..HERO_MIN_H).contains(&hero_room)) {
        let r = Rect::new(px, y + 2, super::plates::PLATE_W, hero_room);
        g.panels.push(("hero", r));
        ui.p.panel(r, th().card, th().edge, RADIUS);
        self_ally_buttons(&mut ui, &g.keys, w, h, r.x + 4, r.y + 2, planning);
    }
    if let Some(h) = sel.filter(|&h| w.units.get(h).is_some_and(|u| u.is_active()) && 318 - y - 6 >= HERO_MIN_H) {
        let u = &w.units[h];
        let r = Rect::new(px, y + 2, super::plates::PLATE_W, 318 - y - 6);
        g.panels.push(("hero", r));
        ui.p.panel(r, th().card, th().edge, RADIUS);
        // BATTLE-UI2: the hero's portrait is on its plate (contract 3); the panel's lines take its whole width
        let face = Rect::new(r.x + 4, r.y + 2, 0, 0);
        let (tx, tw) = (r.x + 4, (r.w - 8) as f32);
        ui.p.text_fit(tx as f32, r.y as f32 + 2.0, &u.name, BODY, tw, th().gold);
        // RULESET1-U: under the round intent the corner names the hero's intent (the classic stance is inert: IN3);
        // PORTRAITFX1-P: on the line under the name
        let corner = if timeline { ri::intent_of(pw, h).map_or_else(|| "No intent".to_string(), |i| i.label(&pw.cat)) } else { stance_name(u).to_string() };
        let row2 = r.y + 2 + (ui.p.line_h(BODY).ceil() as i32).max(10);
        ui.p.text_fit(tx as f32, row2 as f32, &corner, SMALL, tw, th().dim);
        let row3 = row2 + (ui.p.line_h(SMALL).ceil() as i32).max(8) + 1;
        // INPUT1 IF8, SL4: under the round intent the panel's one control is the Self button (key S)
        let top = if timeline {
            self_ally_buttons(&mut ui, &g.keys, w, h, tx, row3, planning)
        } else {
            row3
        };
        let st = compiled.as_ref().expect("a hero selected");
        let weapon = u.inventory.main_weapon().map(|e| e.item.name.clone()).unwrap_or_else(|| "Bare hands".into());
        let mut lines: Vec<(String, Rgba)> = Vec::new();
        // DERIVE1 4.4: the derived-stat line hovers to each row's formula and breakdown
        let derived = derived_rows(s, h);
        let mut derived_at = None;
        let mut stats = |lines: &mut Vec<(String, Rgba)>| {
            lines.push((format!("Initiative {}  Reach {}", u.initiative, reach_text(w, h)), th().text));
            if let Some(line) = derived_line(&derived) {
                derived_at = Some(lines.len());
                lines.push((line, th().text));
            }
        };
        let statuses = |lines: &mut Vec<(String, Rgba)>| {
            if !u.st.active.is_empty() {
                lines.push((format!("Now: {}", u.st.active.iter().map(|x| emberwake_rules::status::display_name(x)).collect::<Vec<_>>().join(", ")), th().foe));
            }
            if !u.st.pending.is_empty() {
                lines.push((format!("Next: {}", u.st.pending.iter().map(|x| emberwake_rules::status::display_name(x)).collect::<Vec<_>>().join(", ")), th().dim));
            }
        };
        let ap_line = (format!("AP left {}/{}", if s.plan_confirmed { u.ap } else { st.ap_left }, u.max_ap), ally());
        // PK1-S2c (FB-174, PK1-S1 `ap` set): why the hero's AP is held this round (the kit's `State::ap_held`)
        let ap_held = s.resolver.triggers.ap_held(h).map(|(ap, src)| (ap_held_text(ap, src), th().foe));
        if timeline {
            // INPUT1 IF8: the round first: "Intent: <Intent> (<kind>) from <first pick | tactic>" (or carried over), a
            // ghost's intent change that waits for Confirm (IN6), the target, the picks, the planned commands without
            // the Rest blocks (the ghost dim), then the Rest gain; the stats after.
            let intent = ri::intent_of(pw, h);
            let picks = rip::picks_of(w, h);
            let from = match picks.iter().find(|p| p.holds_target()) {
                Some(p) => format!(" from {}", pick_text(w, h, p)),
                None if picks.is_empty() && !g.player_target.contains(&h) => format!(" from {}", TACTIC_LABELS[g.tactic_of(h)]),
                None => " (carried over)".into(),
            };
            match intent {
                Some(i) => {
                    let kind = if i == RoundIntent::Maneuver { pw.round_states.get(&h).and_then(|r| r.maneuver).map(|k| format!(" ({})", kind_name(k))).unwrap_or_default() } else { String::new() };
                    lines.push((format!("Intent: {}{kind}{from}", i.label(&pw.cat)), th().text));
                }
                None => lines.push(("Intent: none yet (click a target)".into(), th().gold)),
            }
            if deferred_req.is_some() && planning {
                let drops = intent.map(|i| ri::would_drop(w, h, i)).unwrap_or_default();
                if !drops.is_empty() {
                    lines.push((format!("Changing loses {}.", drops.join(", ")), th().foe));
                }
            }
            match pw.round_states.get(&h).and_then(|r| r.target) {
                Some(t) => lines.push((format!("Target: {}", target_name(pw, t)), th().text)),
                None => lines.push(("Target: click the board".into(), th().dim)),
            }
            // PK1-S2c (PK1 4.3 items 1-2): a Channel round says so (placeholder words and tint)
            if emberwake_battle::channel::is_channel(pw, h) {
                lines.push((format!("{} round: {} versions only", RoundIntent::Channel.label(&pw.cat), super::tips::channeled_word(pw)), CHANNEL_TINT));
            }
            if !picks.is_empty() && !s.plan_confirmed {
                lines.push((format!("Picks: {}", picks.iter().map(|p| pick_text(w, h, p)).collect::<Vec<_>>().join(", ")), th().gold));
            }
            if s.plan_confirmed {
                for (start, _, name, _, tgt) in committed(s, h) {
                    let tgt = tgt.map(|t| format!(" > {}", short_name(&t))).unwrap_or_default();
                    lines.push((format!("B{start} {name}{tgt}"), th().text));
                }
            } else {
                for (k, e) in st.entries.iter().enumerate().filter(|(_, e)| e.step_index >= 0 && emberwake_battle::skill::kind(&w.cat.skills[e.skill]) != Kind::Rest) {
                    let name = if e.item.is_some() { w.cat.skills[e.skill].name.clone() } else { skill_text(pw, h, e.skill) };
                    let tgt = e.target.filter(|t| *t != h).map(|t| format!(" > {}", short_name(&w.units[t].name))).unwrap_or_default();
                    lines.push((format!("B{} {name}{tgt}", e.start_block), if ghosts.get(k).copied().unwrap_or(false) { th().dim } else { th().text }));
                }
                if let Some(gain) = rest.filter(|r| r.blocks > 0) {
                    lines.push((rest_text(gain), th().good));
                }
                let empty: Vec<String> = (1..=s.def.total_blocks).filter(|b| !st.entries.iter().any(|e| *b >= e.start_block && *b <= e.end_block(pw))).map(|b| b.to_string()).collect();
                if !empty.is_empty() && intent.is_some() && u.is_active() {
                    lines.push((format!("Empty: B{}", empty.join(", B")), th().gold));
                }
                if st.error_index >= 0 {
                    lines.push((st.error_text.clone(), th().foe));
                }
            }
            // ENCOUNTER2 H3: why a Withdraw waits (the gate is shut)
            let note = s.gate_note();
            if !note.is_empty() && !s.plan_confirmed {
                lines.push((note, th().foe));
            }
            lines.push(ap_line);
            lines.extend(ap_held);
            statuses(&mut lines);
            stats(&mut lines);
            lines.push((format!("Gear: {weapon}"), th().gold));
        } else {
            stats(&mut lines);
            lines.push(ap_line);
            lines.extend(ap_held);
            lines.push((format!("Gear: {weapon}"), th().gold));
            statuses(&mut lines);
            if s.plan_confirmed {
                lines.push(("This round".into(), th().text));
                for (start, _, name, _, tgt) in committed(s, h) {
                    let tgt = tgt.map(|t| format!(" > {}", short_name(&t))).unwrap_or_default();
                    lines.push((format!("B{start} {name}{tgt}"), th().text));
                }
            } else {
                lines.push(("Plan".into(), th().text));
                // ENCOUNTER2 H3: why a Withdraw waits (the gate is shut)
                let note = s.gate_note();
                if !note.is_empty() {
                    lines.push((note, th().foe));
                }
                if st.entries.iter().all(|e| e.step_index < 0) {
                    lines.push(("No plan yet".into(), th().gold));
                }
                for e in st.entries.iter().filter(|e| e.step_index >= 0) {
                    let name = if e.item.is_some() { w.cat.skills[e.skill].name.clone() } else { skill_text(pw, h, e.skill) };
                    let tgt = e.target.filter(|t| *t != h).map(|t| format!(" > {}", short_name(&w.units[t].name))).unwrap_or_default();
                    lines.push((format!("B{} {name}{tgt}", e.start_block), th().text));
                }
                let empty: Vec<String> = (1..=s.def.total_blocks).filter(|b| !st.entries.iter().any(|e| *b >= e.start_block && *b <= e.end_block(w))).map(|b| b.to_string()).collect();
                if !empty.is_empty() && u.is_active() {
                    lines.push((format!("Empty: B{}", empty.join(", B")), th().gold));
                }
                if st.error_index >= 0 {
                    lines.push((st.error_text.clone(), th().foe));
                }
            }
        }
        let mut ly = top as f32;
        // RULESET1-U slice 6 (TEXTFIT1): the line pitch grows with the text size (8 at Small and Normal, as before)
        let pitch = ui.p.line_h(SMALL).max(8.0);
        for (n, (text, col)) in lines.into_iter().enumerate() {
            // PORTRAITFX1-P: a line whose first row fits beside the portrait starts 40 px in (and wraps to the room
            // left); the first that does not starts under the portrait
            let below = (face.y + face.h + 1) as f32;
            let beside = ly + pitch <= below;
            if !beside && ly < below {
                ly = below;
            }
            let top = ly;
            let (lx, lw) = if beside { (tx as f32, tw) } else { (r.x as f32 + 4.0, (r.w - 8) as f32) };
            for part in ui.p.wrap(&text, SMALL, lw) {
                if ly > (r.y + r.h) as f32 - pitch - 1.0 {
                    break;
                }
                ui.p.text(lx, ly, &part, SMALL, col);
                ly += pitch;
            }
            if Some(n) == derived_at && !derived.is_empty() && ly > top {
                let tip: Tip = derived.iter().flat_map(|d| [(format!("{} {}", d.label, d.shown), th().gold), (d.desc.clone(), th().text)]).collect();
                ui.push("hero.derived", "row", text, Rect::new(r.x + 2, top as i32 - 1, r.w - 4, (ly - top).ceil() as i32), true, false, None, tip);
            }
        }
        ui.push("hero.panel", "panel", u.name.clone(), r, true, false, None, Vec::new());
    }

    // BATTLE-UI2 (contract 3): the board's main panel between the side panels (888 px at 1280 x 720), the grid inside it
    crate::hud::hud_panel(&mut ui, BOARD, th().game("battle_panel", PANEL), th().edge, RADIUS);
    // -- Grid (pixel layer): STAGE1 Stage (the tilted board) or Flat (the board seen from above, as before) by the
    // Battle view setting; both use the same tokens, actions, widget names and HUD.
    let view = View::of_setting(&cx.settings.v["battle_view"]);
    // 3.6 / 3.9: a unit hovered through its party or hostile card (the cards of the last frame) is hovered on the board.
    let card_hover = if live && g.popup.is_none() {
        g.w.iter().filter(|x| x.rect.contains(mouse.0, mouse.1)).find_map(|x| x.id.strip_prefix("party.").or_else(|| x.id.strip_prefix("foe.")).and_then(|n| n.parse::<usize>().ok()))
    } else {
        None
    };
    // FX1 4.1: the camera's shake shifts the board's passes (inside its clip); the HUD and every hit rect stay put.
    let shift = g.camera.offset_at(g.clock.now());
    // RULESET1-U slice 3: the round intent's board overlays (None in classic)
    let mut overlay = overlay_of(s, sel, planning);
    // INPUT1 MR13: the hovered Engage row marks its target's open position tiles
    let marked = match &hovered_pick {
        Some(((h, at), row)) if planning => match &row.act {
            RowAct::Pick(picks) => engage_positions(s, *h, *at, picks),
            _ => Vec::new(),
        },
        // VISCLAR1-D: the tiles the Move can go to while it waits for one
        _ if planning && g.move_aim.is_some() => g.move_tiles.iter().map(|t| (t.cell, true)).collect(),
        _ => Vec::new(),
    };
    // A4.2-K1b: the hovered area row previews its impact tiles (Field marks or the Assault lock)
    let previewed = match &hovered_pick {
        Some(((h, at), row)) if planning => match &row.act {
            RowAct::Pick(picks) => super::input::area_preview(s, *h, row.at.unwrap_or(*at), picks),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };
    if let Some(o) = overlay.as_mut() {
        if !previewed.is_empty() {
            o.areas = previewed;
        }
        o.positions = marked.clone();
        // VISCLAR1 AR1-AR3: every unit's target arrow; the board draws the selected and inspected unit's (Alt: all)
        o.arrows = arrows_of(s, &tells, &rounds, &layers, show_tells);
        g.arrows_shown = o.shown(&layers, sel, pinned, cx.held.alt).arrows;
    } else {
        g.arrows_shown = Vec::new();
    }
    g.positions_shown = marked;
    let push_list: Vec<emberwake_battle::forced::Forecast> = pushes.iter().flatten().cloned().collect();
    // VC2: the focus set but the unit under the mouse (each view adds its own hover)
    let focus = focus_set(w, sel, None, pinned);
    let stage_set = g.stage_set(cx);
    // PK1-S2c (PK1 4.3 item 6, placeholder): the units whose round is a Channel round wear the tint: a party member's
    // own flag, an enemy's through its round record while the tells show
    let channel_units: Vec<usize> = (0..w.units.len()).filter(|&i| if w.allies.contains(&i) { emberwake_battle::channel::is_channel(w, i) } else { show_tells && rounds.iter().any(|r| r.unit == i && r.channel) }).collect();
    g.channel_units = channel_units.clone();
    // BOSSUI1 BU11, BU12 (PR7-59): the Stage board shifts down until the boss bar covers none of its units and
    // targetable tiles; its passes draw shifted and its widgets (tiles, units) are moved with them
    let lift = if model.boss.is_some() && !g.timeline_view_open() { super::boss_bar::board_lift(w, &Geom::new(view, &tuning)) } else { 0 };
    g.board_lift = lift;
    let bar_bottom = model.boss.as_ref().filter(|_| !g.timeline_view_open()).map(|bb| BOARD.y + super::boss_bar::INSET + super::boss_bar::bar_h(bb));
    let widgets_from = ui.widgets.len();
    ui.mouse.1 -= lift;
    let board = Board { s, tokens: &mut g.tokens, floats: &g.floats, flashes: &g.flashes, popup: g.popup.is_some() || talk_view.is_some(), anim_ms: g.clock.ms, sel, talker, planning, tells: &tells, show_tells, card_hover, inspect: pinned, focus: &focus, fx: &g.fx, shift: (shift.0, shift.1 + lift), sidesteps: &g.sidesteps, overlay: overlay.as_ref(), rounds: &rounds, layers, alt: cx.held.alt, marks: &marks, classes: &classes, pushes: &push_list, hold: hold.as_ref(), channel: &channel_units, bar_bottom };
    let (board_hover, flat_hover) = match view {
        View::Stage => (draw_stage(board, &stage_set, &mut ui, cx, &tuning, now), None),
        View::Flat => (None, draw_flat(board, &mut ui, cx, &tuning, now)),
    };
    ui.mouse.1 += lift;
    for wd in &mut ui.widgets[widgets_from..] {
        wd.rect.y += lift;
    }
    g.focus_shown = focus_set(w, sel, board_hover.or(flat_hover).or(card_hover), pinned);
    // INPUT1 PB1: the impact tracers above the tokens (group `tracers`), shaken with the board, inside its panel
    if aids && layers.tracers && !g.tracers.is_empty() {
        let geo = Geom::new(view, &tuning);
        ui.p.clip(Some(Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4)));
        ui.p.shift(shift.0, shift.1 + lift);
        draw_tracers(&mut ui.p, &g.tracers, &geo, g.clock.now(), cx.settings.reduce_motion());
        ui.p.shift(-shift.0, -shift.1 - lift);
        ui.p.clip(None);
    }
    // ENCOUNTER2 H1: the hold pick over the board while round 1 waits for it
    // ESCAPE-PLAN1 (3): until round 1 is confirmed a chip names the holder and opens the pick again
    if planning && (s.needs_stay_pick() || (g.hold_repick && s.current_round == 1)) {
        draw_hold_pick(&mut ui, s);
    } else if planning && s.current_round == 1 && s.w.hold.is_some() {
        draw_hold_chip(&mut ui, s);
    }
    // WINDUP1 6.2: the clip each token drew (the agent link's `tokens`)
    note_drawn(&mut g.tokens, s, &mut cx.art, g.clock.ms as u64);

    // -- Right: the foe panel (BATTLE-UI2 contract 4: "Hostiles N", the reinforcement tag, full plates up to 6, compact
    // from 7, groups from 11), then the docked foe card at the board's top right (contract 5).
    let right = Rect::new(W - 2 - SIDE_W, 28, SIDE_W, 290);
    crate::hud::hud_panel(&mut ui, right, th().game("battle_panel", PANEL), th().edge, RADIUS);
    ui.p.text(right.x as f32 + 6.0, 32.0, &model.hostiles, TITLE, th().foe);
    // PR7-63: the count and the tag keep their own cells; a tag that would run into the count takes its own row
    let mut tag_row = 0;
    if !model.reinforce.is_empty() {
        let tw = ui.p.text_w(&model.reinforce, SMALL).ceil() as i32 + 6;
        let count_end = right.x + 6 + ui.p.text_w(&model.hostiles, TITLE).ceil() as i32;
        let lh = ui.p.line_h(SMALL);
        // as tall as its line (9 at Normal), its text centred when the line is taller
        let tag_h = (lh.ceil() as i32 + 1).max(9);
        let mut ty = 31;
        if right.x + right.w - 4 - tw < count_end + 3 {
            ty = 32 + ui.p.line_h(TITLE).ceil() as i32 - 1;
            tag_row = ty + tag_h + 2 - 42;
        }
        let tr = Rect::new(right.x + right.w - 4 - tw, ty, tw, tag_h);
        ui.p.panel(tr, rgba(0, 0, 0, 0), crate::tokens::planned(), 1.5);
        let text_y = if tag_h > 9 { tr.y as f32 + (tag_h as f32 - lh) / 2.0 } else { tr.y as f32 + 1.0 };
        ui.p.text(tr.x as f32 + 3.0, text_y, &model.reinforce, SMALL, crate::tokens::planned());
        ui.push("foe.reinforce", "badge", model.reinforce.clone(), tr, true, false, None, vec![("Reinforcements".into(), crate::tokens::planned()), (model.reinforce.clone(), th().text)]);
    }
    let mut y = 42 + tag_row;
    let mut hovered_foe = None;
    let fx = right.x + (SIDE_W - super::plates::PLATE_W) / 2;
    for (k, entry) in model.foes.iter().enumerate() {
        let h = super::plates::entry_h(entry);
        if y + h > 318 {
            break;
        }
        let r = Rect::new(fx, y, super::plates::PLATE_W, h);
        let on_board = entry.units().into_iter().any(|e| match view {
            View::Stage => board_hover == Some(e),
            View::Flat => w.cells_of(e).into_iter().any(|p| ui.hover(cell_rect(p))),
        });
        let hover = ui.hover(r) || on_board;
        match entry {
            crate::battle_view::FoeEntry::Full(p) | crate::battle_view::FoeEntry::Compact(p) => {
                let e = p.unit;
                if hover {
                    hovered_foe = Some(e);
                }
                let hot = hover || model.card.as_ref().is_some_and(|c| c.foe == e);
                let first_widget = ui.widgets.len();
                if matches!(entry, crate::battle_view::FoeEntry::Full(_)) {
                    super::plates::foe_plate(&mut ui, cx, &mut g.panel_faces, &mut g.mini_faces, w, &looks, p, r, hot);
                } else {
                    super::plates::compact_plate(&mut ui, cx, p, r, hot);
                }
                let u = &w.units[e];
                if layers.guard && held_guard(s, e).is_some() {
                    ui.p.panel(r.inset(1), rgba(0, 0, 0, 0), HELD, RADIUS);
                }
                let tell = tells.iter().find(|t| t.unit == e);
                let next = tell.map(|t| intent_text::round_card_line(w, t)).unwrap_or_else(|| format!("Next: {}", next_action(s, e)));
                // RULESET1 V1 (rule plan_badges): the impacts of the round on the "when it lands" row's hover
                if let Some(t) = tell.filter(|_| u.is_active() && matches!(entry, crate::battle_view::FoeEntry::Full(_)) && emberwake_battle::intent::badges_on(w) && layers.badges) {
                    let tip = impact_tip(t, s.def.total_blocks.max(1));
                    ui.push(format!("foe.{e}.badges"), "badges", tip.iter().skip(1).map(|x| x.0.clone()).collect::<Vec<_>>().join("; "), super::plates::lands_rect(r), true, false, None, tip);
                }
                let tip = unit_tip(s, e, sel, &tells, &rounds, &classes);
                ui.push(format!("foe.{e}"), "card", format!("{} {}/{} {}", u.name, u.hp, u.max_hp, next), r, true, hover, Some(Action::UnitClick(e)), tip);
                super::statusvis::lift_plate_info(&mut ui, first_widget, &format!("foe.{e}"));
            }
            crate::battle_view::FoeEntry::Group { key, name, plates, open } => {
                // contract 4 (PR7-40 part 2): a click opens the group into compact plates; the open group's header closes it
                let mut tip: Tip = std::iter::once((name.clone(), th().foe)).chain(plates.iter().map(|p| (format!("{} {}/{}", p.name, p.hp, p.max_hp), th().text))).collect();
                if *open {
                    super::plates::group_header(&mut ui, name, r);
                    tip.push(("Click: close the group".into(), th().dim));
                } else {
                    super::plates::group_row(&mut ui, cx, &mut g.mini_faces, &looks, name, plates, r);
                    tip.push(("Click: open the group".into(), th().dim));
                }
                ui.push(format!("foe.group.{k}"), "card", name.clone(), r, true, hover, Some(Action::FoeGroup(key.clone())), tip);
            }
        }
        y += h + super::plates::PLATE_GAP;
    }
    g.panels.push(("hostiles", Rect::new(right.x, 28, right.w, y - 26)));
    debug_assert!(g.panels.iter().all(|p| PANELS.contains(&p.0)), "a panel recorded under a name PANELS lacks");
    let _ = (hovered_ally, hovered_foe);
    // contract 5: the docked foe card at the board's top right (the panel `intel`); the full timeline view covers the
    // board, the card with it
    // BOSSUI1 (PR7-54, contract 9.1): the boss bar across the board's top (the panel `intel`); the card docks under it
    let mut card_top = BOARD.y + 3;
    if let Some(bb) = model.boss.as_ref().filter(|_| !g.timeline_view_open()) {
        let inset = super::boss_bar::INSET;
        let r = Rect::new(BOARD.x + inset, BOARD.y + inset, BOARD.w - 2 * inset, super::boss_bar::bar_h(bb));
        g.panels.push(("intel", r));
        let tip = if pinned == Some(bb.unit) { inspect_tip(s, bb.unit, sel, &tells, &rounds, &classes, show_tells, &g.keys) } else { Vec::new() };
        if let Some(hr) = super::boss_bar::draw_bar(&mut ui, cx, (&mut g.card_faces, &mut g.mini_faces), &looks, bb, r, tip) {
            g.panels.push(("intel", hr));
        }
        card_top = r.y + r.h + 2;
    }
    if let Some(c) = model.card.as_ref().filter(|_| !g.timeline_view_open()) {
        let h = super::card::card_h(c);
        // contract 9.2 (BU12): the card docks at the board's top right unless it would cover the selected hero or its
        // target, then top left
        let mut r = Rect::new(BOARD.x + BOARD.w - 3 - super::card::CARD_W, card_top, super::card::CARD_W, h.min(BOARD.h - 6));
        let geo = Geom::new(View::of_setting(&cx.settings.v["battle_view"]), &tuning);
        let keep: Vec<Rect> = card_keep_clear(w, sel).into_iter().map(|c| geo.cell_rect(c)).map(|r| Rect::new(r.x, r.y + lift, r.w, r.h)).collect();
        // ESCAPE-PLAN1 (5): and the objective tiles (a hold's tiles, the exits), on the side covering fewer of them
        let goals: Vec<Rect> = objective_cells(w).into_iter().map(|c| geo.cell_rect(c)).map(|r| Rect::new(r.x, r.y + lift, r.w, r.h)).collect();
        // DIORAMA4 S4: in a boss fight (the bar shown) the card covers no party cell: away from the party, else compact
        let boss = model.boss.is_some();
        let party: Vec<Rect> = if boss { w.allies.iter().filter(|a| w.units[**a].is_alive()).flat_map(|a| w.cells_of(*a)).map(|c| geo.cell_rect(c)).map(|r| Rect::new(r.x, r.y + lift, r.w, r.h)).collect() } else { Vec::new() };
        let xs: Vec<i32> = w.allies.iter().filter_map(|a| w.grid.pos(*a)).map(|c| c.x).collect();
        let party_left = !xs.is_empty() && 2 * xs.iter().sum::<i32>() < xs.len() as i32 * (emberwake_battle::grid::GRID_W - 1);
        let hits = |list: &[Rect], r: Rect| list.iter().filter(|k| super::skill_menu::overlaps(r, **k)).count();
        r = super::card::dock(r, BOARD.x + 3, boss, party_left, |r| (hits(&party, r), hits(&keep, r), hits(&goals, r)));
        g.panels.push(("intel", r));
        // INPUT1 IS1: a pinned foe's card names it and holds its inspect lines on its hover; the agent link's
        // `intel.unit` is the card under the round intent or a pin (classic: `card.unit`), a pinned member's plate
        // when a member is inspected (IS3)
        let (label, tip) = if pinned == Some(c.foe) { (super::tips::inspecting(&w.units[c.foe].name, &g.keys), inspect_tip(s, c.foe, sel, &tells, &rounds, &classes, show_tells, &g.keys)) } else { (w.units[c.foe].name.clone(), Vec::new()) };
        let id = if (ri::on(w) || pinned.is_some()) && !pinned.is_some_and(|p| w.allies.contains(&p)) { "intel.unit" } else { "card.unit" };
        super::card::draw_card(&mut ui, cx, (&mut g.card_faces, &mut g.mini_faces), &looks, c, r, id, &label, tip);
    }
    if let Some(m) = pinned.filter(|p| w.allies.contains(p)) {
        if let Some(r) = ui.widgets.iter().find(|x| x.id == format!("party.{m}")).map(|x| x.rect) {
            ui.push("intel.unit", "panel", super::tips::inspecting(&w.units[m].name, &g.keys), r, true, false, None, inspect_tip(s, m, sel, &tells, &rounds, &classes, show_tells, &g.keys));
        }
    }
    g.view_json = crate::battle_view::view(&model).to_json();

    // -- Bottom bar: playback, round and status, the last log line.
    crate::hud::hud_panel(&mut ui, Rect::new(2, 322, W - 4, 36), th().game("battle_panel", PANEL), th().edge, RADIUS);
    // PR7-41 (PT-32): Restart and Step show only when they work (a world battle never restarts; Step once a round runs)
    if !g.world_mode() {
        ui.button_full("bar.restart", Rect::new(6, 325, 40, 14), "Restart", Action::Restart, true, false, BODY, vec![(format!("Restart the battle ({})", g.keys.key_hint(aegis_ui::screen::Key::Char('r'))), th().text)]);
    }
    if !planning {
        ui.button_full("bar.step", Rect::new(48, 325, 34, 14), "Step", Action::Step, true, false, BODY, vec![(format!("Resolve one action ({})", g.keys.hint(aegis_ui::screen::Key::Space)), th().text)]);
    }
    // Test Arena only: leave at any time (Esc does the same); a world battle is left only through its end, and its Menu
    // (PR7-26; Esc with nothing open does the same) opens the in-game Settings popup in the same place.
    // PR7-47 (FB-344 question 4): Menu / Exit is as wide as its label at the text size (30 at Normal), and the round
    // read-out and the message after it move right by as much
    let bar_label = if g.world_mode() { "Menu" } else { "Exit" };
    let bar_w = 30.max((ui.p.text_w(bar_label, BODY) + ui.m().uf(6.0)).ceil() as i32);
    if g.world_mode() {
        ui.button_full("bar.menu", Rect::new(84, 325, bar_w, 14), "Menu", Action::Settings, true, false, BODY, vec![(format!("Menu ({})", g.keys.hint(aegis_ui::screen::Key::Esc)), th().gold), ("Settings, Retreat where the fight allows it, and Return to title once the fight is over.".into(), th().text)]);
    } else {
        ui.button_full("bar.exit", Rect::new(84, 325, bar_w, 14), "Exit", Action::Leave, true, false, BODY, vec![(format!("Back to the screen you came from ({})", g.keys.hint(aegis_ui::screen::Key::Esc)), th().text)]);
    }
    let ox = 32 + (bar_w - 30);
    let phase = match s.phase {
        Phase::Planning => "Planning",
        Phase::Resolving => "Resolving",
        Phase::Replay => "Round over",
    };
    ui.p.text((88 + ox) as f32, 326.0, &format!("Round {}/{}  {phase}", s.current_round, s.def.max_rounds), BODY, th().gold);
    // RULESET1-U (user 2026-10-29): an Arena fight beyond classic shows the Scene Runner card's ruleset chip at the
    // bar's right end; the message gives up the width
    let chip = g.arena_chip();
    let chip_w = chip.as_ref().map_or(0, |c| ((ui.p.text_w(c, SMALL) + 10.0).ceil() as i32).min(200) + 4);
    // VISCLAR1-D: the last round's summary before the next round's prompt, as much of it as fits; the log row keeps it whole
    let room = (430 - ox - chip_w) as f32;
    let shown_message = match g.round_note.as_ref().filter(|n| n.0 == g.message) {
        Some((prompt, summary)) => summary_and_prompt(summary, prompt, room, |t| ui.p.text_w(t, BODY)),
        None => g.message.clone(),
    };
    // PR7-41 (PT-34): the phase word leads the bar already ("Round 2/8  Round over"), so the message does not repeat it
    let shown_message = without_phase(&shown_message, phase);
    ui.p.text_fit((200 + ox) as f32, 326.0, &shown_message, BODY, room, th().text);
    // PR7-63: a message cut to the bar hovers to the whole of it
    let status_tip: Tip = if ui.p.text_w(&shown_message, BODY) > room { vec![(shown_message.clone(), th().text)] } else { Vec::new() };
    ui.push("bar.status", "label", format!("Round {} {phase}: {shown_message}", s.current_round), Rect::new(88 + ox, 325, 540 - ox - chip_w, 12), true, false, None, status_tip);
    if let Some(chip) = chip {
        let cr = Rect::new(W - 6 - (chip_w - 4), 325, chip_w - 4, 14);
        ui.p.panel(cr, th().game("chip", rgba(30, 44, 70, 255)), th().edge, 2.0);
        ui.p.text_fit(cr.x as f32 + 5.0, cr.y as f32 + 3.0, &chip, SMALL, (cr.w - 10) as f32, th().gold);
        let on = format!("Rules on: {}.", s.w.ruleset.rules.join(", "));
        ui.push("bar.chip", "label", chip.clone(), cr, true, false, None, vec![(format!("Ruleset: {chip}"), th().gold), (on, th().text)]);
    }
    let last = s.resolver.log.last().cloned().unwrap_or_else(|| "No actions yet".into());
    // INPUT1 PB4: a timeline battle's log in its parts (round and block); during playback the compact log shows the
    // resolving block's part (its last line; the card lists the part)
    let groups = if aids { s.log_groups() } else { Vec::new() };
    let (last, mut card): (String, Tip) = match cursor {
        Some(b) => {
            let lines = groups.iter().find(|p| (p.round, p.block) == (s.current_round, b)).map(|p| p.lines.clone()).unwrap_or_default();
            let head = (format!("Round {} block {b}", s.current_round), th().gold);
            let shown = lines.last().map_or_else(|| format!("Block {b}: nothing logged yet"), |l| format!("Block {b}: {l}"));
            let start = lines.len().saturating_sub(LOG_CARD_LINES);
            (shown, std::iter::once(head).chain(lines[start..].iter().map(|l| (l.clone(), th().text))).collect())
        }
        None => (last.clone(), vec![(last, th().text)]),
    };
    card.push(("Click to open the full log.".into(), th().dim));
    let log_r = Rect::new(6, 341, W - 64, 14);
    ui.p.panel(log_r, th().card, th().edge, RADIUS);
    ui.p.text_fit(10.0, 342.5, &last, SMALL, (W - 72) as f32, th().dim);
    ui.push("bar.last", "label", last, log_r, true, false, Some(Action::ToggleLog), card);
    ui.button_full("bar.log", Rect::new(W - 54, 341, 48, 14), if g.show_log { "Close" } else { "Expand" }, Action::ToggleLog, true, g.show_log, BODY, vec![(format!("Full combat log ({}); the wheel scrolls", g.keys.hint(aegis_ui::screen::Key::Char('l'))), th().text)]);

    let outcome = g.outcome();
    // TRANSITIONS1 T2: the victory beat's banner stands in for the outcome panel while it runs
    if !outcome.is_empty() && g.beat.is_none_or(|b| b.kind != crate::transitions::Kind::Victory) {
        let r = Rect::new(GRID_X + 92, GRID_Y + 110, 200, 56);
        ui.p.panel(r, th().game("outcome_fill", rgba(10, 14, 22, 240)), th().gold, 4.0);
        let (title, col) = match outcome.as_str() {
            "won" => ("Victory", th().good),
            // ENRAGE2 EN9: a random fight's foes slipped away at its round cap
            emberwake_world::battle::ESCAPED => (crate::hub::SLIPPED_AWAY, th().gold),
            // PR7-37 (PT-02): the round limit ran out (nobody fled)
            _ if g.timed_out() => (crate::hub::OUT_OF_TIME, th().gold),
            "lost" => ("Defeat", th().foe),
            "fled" => ("Fled", th().gold),
            _ => ("Battle over", th().gold),
        };
        ui.p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + 8.0, title, 11.0, col);
        if g.world_mode() {
            ui.button_full("banner.continue", Rect::new(r.x + 64, r.y + 32, 72, 16), "Continue", Action::Continue, true, true, BODY, Vec::new());
        } else {
            ui.button_full("banner.restart", Rect::new(r.x + 24, r.y + 32, 72, 16), "Play again", Action::Restart, true, false, BODY, Vec::new());
            ui.button_full("banner.leave", Rect::new(r.x + 104, r.y + 32, 72, 16), "Leave", Action::Leave, true, false, BODY, Vec::new());
        }
    }

    // FLOATPANELS1: the open log floats over the board (a free corner: the board's top left, locked); unlocked it drags
    // anywhere over the board, and its minimize button shrinks it in place to a "Combat log" button
    let log_open = g.show_log && {
        let size = LOG_SIZE;
        let chip = aegis_ui::float_panel::chip_size(&mut ui, LOG_NAME);
        let held = ui.dragging() == Some(LOG_DRAG);
        crate::floats::frame(cx, &mut g.log_panel, &mut g.log_kept, crate::floats::BATTLE_LOG_KEY, LOG_AREA, (LOG_AREA.x + 4, LOG_AREA.y + 4), size, chip, mouse, held);
        g.log_size = size;
        let acts = aegis_ui::float_panel::Acts { body: Action::Step, lock: Action::LogLock, min: Action::LogMin, open: Action::LogOpen };
        let hud = |ui: &mut aegis_ui::ui::Ui<Action>, p: Rect| crate::hud::hud_panel(ui, p, rgba(0, 0, 0, 0), th().edge, 3.0);
        aegis_ui::float_panel::draw(&mut ui, &g.log_panel, "log.panel", LOG_NAME, LOG_DRAG, size, chip, &acts, th().game("log_fill", rgba(10, 14, 24, 246)), hud).is_some()
    };
    if log_open {
        let r = g.log_panel.rect(LOG_SIZE);
        // ENCOUNTER2-U (decision "RULESET1-A3 loose ends"): the resolver log with the encounter's phase and kill lines
        let log = s.log_lines(0, 0);
        let head = Rect::new(r.x + 6, r.y + 3, r.w - 12 - 2 * aegis_ui::float_panel::PANEL_BTN - 8, 10);
        ui.p.text_fit(head.x as f32, head.y as f32, &format!("Combat log ({} entries)", log.len()), TITLE, head.w as f32, th().gold);
        ui.push("log.lines", "label", format!("{} lines", log.len()), head, true, false, None, Vec::new());
        // the rows: text, colour, x offset, a rule after it; INPUT1 PB4: a timeline battle's lines under their round's
        // "Round <r>" and their part's DIM "Block <b>" ("Round start", "Round end") with a 1 px rule, 6 px in
        let mut wrapped: Vec<(String, Rgba, f32, bool)> = Vec::new();
        let wrap_line = |p: &mut Painter, out: &mut Vec<(String, Rgba, f32, bool)>, l: &str, indent: f32| {
            let col = if l.starts_with("[Phase]") { th().gold } else if l.starts_with("[Kill]") { th().foe } else if l.starts_with("[Attack]") { th().text } else if l.starts_with("[Support]") { th().good } else if l.contains("Failed") || l.contains("Out of Range") { th().foe } else { th().dim };
            for (k, part) in p.wrap(&crate::intent_text::plain(l), SMALL, (r.w - 22) as f32 - indent).into_iter().enumerate() {
                out.push((if k == 0 { part } else { format!("   {part}") }, col, indent, false));
            }
        };
        if aids {
            let mut round = 0;
            for part in &groups {
                if part.round != round {
                    round = part.round;
                    wrapped.push((format!("Round {round}"), th().gold, 0.0, false));
                }
                wrapped.push((log_part_name(part.block), th().dim, 0.0, true));
                for l in &part.lines {
                    wrap_line(&mut ui.p, &mut wrapped, l, LOG_INDENT);
                }
            }
        } else {
            for l in &log {
                wrap_line(&mut ui.p, &mut wrapped, l, 0.0);
            }
        }
        let visible = ((r.h - 18) as f32 / 8.0) as i32;
        let max_scroll = (wrapped.len() as i32 - visible).max(0);
        g.log_scroll = g.log_scroll.min(max_scroll);
        let start = (wrapped.len() as i32 - visible - g.log_scroll).max(0) as usize;
        for (k, (t, col, indent, rule)) in wrapped.iter().skip(start).take(visible as usize).enumerate() {
            let (x, y) = (r.x as f32 + 6.0 + indent, r.y as f32 + 15.0 + k as f32 * 8.0);
            ui.p.text(x, y, t, SMALL, *col);
            if *rule {
                let x0 = (x + ui.p.text_w(t, SMALL) + 3.0).ceil() as i32;
                ui.p.fill(Rect::new(x0, y as i32 + 4, (r.x + r.w - 12 - x0).max(0), 1), with_alpha(th().dim, 110));
            }
        }
        if max_scroll > 0 {
            let track = Rect::new(r.x + r.w - 8, r.y + 15, 4, r.h - 19);
            ui.p.panel(track, th().game("battle_track", rgba(30, 40, 60, 255)), th().game("battle_track", rgba(30, 40, 60, 255)), 1.5);
            let thumb_h = (track.h * visible / wrapped.len() as i32).max(8);
            let pos = (track.h - thumb_h) * (max_scroll - g.log_scroll) / max_scroll;
            ui.p.panel(Rect::new(track.x, track.y + pos, 4, thumb_h), th().gold, th().gold, 1.5);
        }
    }

    // ENCOUNTER2-C4: a scripted round's banner (locked or guided) above the board, centred at its top edge
    if let Some(text) = s.round_script().map(|r| r.banner.clone()).filter(|b| !b.is_empty()) {
        let bw = ((ui.p.text_w(&text, BODY) + 16.0).ceil() as i32).min(GRID_W * TILE - 8);
        let r = Rect::new(GRID_X + (GRID_W * TILE - bw) / 2, GRID_Y + 2, bw, 16);
        ui.p.panel(r, th().card, th().gold, RADIUS);
        ui.p.text_fit(r.x as f32 + 8.0, r.y as f32 + 3.0, &text, BODY, r.w as f32 - 16.0, th().gold);
        ui.push("battle.banner", "label", text, r, true, false, None, Vec::new());
    }

    // VISCLAR1-D: the round banner, over the board's upper part (under a scripted round's banner); it takes no input
    if let Some((round, alpha)) = g.round_banner_now(cx) {
        draw_round_banner(&mut ui.p, Rect::new(GRID_X, GRID_Y, GRID_W * TILE, GRID_H * TILE), round, alpha);
    }

    // INPUT1 TV1-TV6: the full timeline view over the grid panel's rect (key T; T or Esc closes it): the engine widget
    // on the shared builder (`timeline_view`), 4 px inside the panel; its backdrop keeps the board's clicks and hover
    // cards from showing through
    if g.timeline_view_open() {
        let r = Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4);
        let opts = timeline_view::Opts { viewed: s.current_round, reveal_all: false, foresight_off, show_tells, layers, selected: sel, block: None, shown: sel.zip(shown).map(|(u, (pw, pp))| (u, pw, pp)), playhead: cursor };
        let mut t = timeline_view::build(s, &opts);
        // PORTRAITFX1-TL (6.2 row 4): every unit's track leads with its portrait in the icon square (minimal tier,
        // `timeline_faces`), the panels' `unit_portrait` with the board's HP and life
        let mut icons: std::collections::BTreeMap<String, (Who, emberwake_battle::portrait::Look)> = std::collections::BTreeMap::new();
        for track in t.sections.iter_mut().flat_map(|sec| sec.tracks.iter_mut()) {
            if let Some(u) = timeline_view::unit_of(&track.key).filter(|&u| u < s.w.units.len()) {
                track.icon = true;
                let (who, mut look) = g.unit_portrait(cx, u);
                if timeline_row_plain(s, u, track.masked.is_some()) {
                    look = plain_look(&look);
                }
                icons.insert(track.key.clone(), (who, look));
            }
        }
        ui.push("timeline.backdrop", "backdrop", "", r, true, false, Some(Action::TimelinePick(None)), Vec::new());
        crate::hud::hud_panel(&mut ui, r, th().game("timeline_fill", rgba(10, 14, 24, 250)), th().edge, RADIUS);
        let on_track = |k: &str| Action::TimelinePick(timeline_view::unit_of(k));
        // STATUSVIS1-B (SV14, SV15): each row's one target in its aside square, each span's skill (or lock) icon
        let aims = timeline_view::row_aims(s, &opts);
        let faces = &mut g.timeline_faces;
        let small = &mut g.mini_faces;
        let slot = |ui: &mut Ui<Action>, slot: aegis_ui::timeline::Slot<'_>, sq: Rect| match slot {
            aegis_ui::timeline::Slot::Icon(track) => {
                if let Some((who, look)) = icons.get(&track.key) {
                    timeline_portrait(ui, cx, faces, who, look, sq);
                }
            }
            aegis_ui::timeline::Slot::Aside(track) => {
                if let Some(aim) = aims.get(&track.key) {
                    super::statusvis::row_target(ui, cx, small, &looks, &s.w, &track.key, *aim, sq);
                }
            }
            aegis_ui::timeline::Slot::Segment(_, seg) => {
                let chain = seg.icon.as_deref().map(timeline_view::icon_chain).unwrap_or_default();
                let px = ui.icon_px(sq).max(1);
                if let Some(img) = cx.art.first(&chain, px, px).cloned() {
                    ui.p.image_alpha(&img, sq, if seg.ghost { 150 } else { 255 });
                }
            }
        };
        t.draw_slots(&mut ui, "timeline", Rect::new(r.x + TV_PAD, r.y + TV_PAD, r.w - 2 * TV_PAD, r.h - 2 * TV_PAD), |_| Action::TimelinePick(None), on_track, |k, _| on_track(k), slot);
        g.tv_shown = Some(t);
    } else {
        g.tv_shown = None;
    }

    if let Some((note, left)) = move_note {
        let (tw, th_) = (ui.p.text_w(&note, SMALL).ceil() as i32 + 8, ui.p.line_h(SMALL).ceil() as i32 + 2);
        let plate = Rect::new(left.min(W - tw - 2), 24, tw, th_);
        ui.p.panel(plate, th().game("popup_fill", rgba(14, 20, 34, 252)), REST_BLOCK, 2.0);
        ui.p.text(plate.x as f32 + 4.0, plate.y as f32 + 1.0, &note, SMALL, th().text);
        ui.push("strip.move_note", "note", note.clone(), plate, true, false, None, vec![(note, th().text)]);
    }

    if let Some(grid) = g.popup.as_ref().and_then(|p| p.grid.clone()) {
        // VISCLAR1-D: the skill menu as icons
        let pop = g.popup.clone().expect("the grid's popup");
        ui.push("popup.backdrop", "backdrop", "", Rect::new(0, 0, W, H), true, false, Some(Action::ClosePopup), Vec::new());
        super::skill_menu::draw_menu(&mut ui, cx, &pop, &grid, g.menu_row);
    } else if let Some(pop) = g.popup.clone() {
        let r = pop.rect(g.menu_row);
        ui.push("popup.backdrop", "backdrop", "", Rect::new(0, 0, W, H), true, false, Some(Action::ClosePopup), Vec::new());
        ui.p.panel(r, th().game("popup_fill", rgba(14, 20, 34, 252)), th().gold, 3.0);
        ui.p.text_fit(r.x as f32 + 5.0, r.y as f32 + 2.0, &pop.title, BODY, r.w as f32 - 10.0, th().gold);
        for (k, row) in pop.rows.iter().enumerate() {
            let rr = pop.row(g.menu_row, k);
            let hover = ui.hover(rr);
            if hover && row.valid {
                ui.p.panel(rr, th().card_hot, th().edge_hot, 1.5);
            }
            let col = if row.valid { th().text } else { th().dim };
            // the label column: 78 px in a 170 px menu, wider in the skill menu (INPUT1 IF3)
            let label_w = (pop.w - 92) as f32;
            ui.p.text_fit(rr.x as f32 + 4.0, rr.y as f32 + 1.5, &row.label, BODY, label_w, col);
            ui.p.text_fit(rr.x as f32 + label_w + 6.0, rr.y as f32 + 2.0, &row.detail, SMALL, rr.w as f32 - label_w - 8.0, if row.valid { th().gold } else { th().dim });
            // IF3, IN6: a pick that changes the intent and drops intent-bound buffs: a red corner
            if row.warn {
                ui.p.fill(Rect::new(rr.x + rr.w - 4, rr.y + 1, 3, 3), th().foe);
            }
            ui.push(format!("popup.{k}"), "option", format!("{} {}", row.label, row.detail), rr, row.valid, hover, Some(Action::Choose(k)), row.tip.clone());
        }
    }

    // ENCOUNTER2-C DT7: the talk over a 35 % veil, centred at the bottom; its backdrop keeps the board's hover cards
    // from showing through the box
    if let (Some(gs), Some(view)) = (talk_gs.as_ref(), talk_view.as_ref()) {
        ui.p.fill(Rect::new(0, 0, W, H), th().game("talk_veil", TALK_VEIL));
        let b = talk::talk_layout(&mut ui.p, view);
        ui.push("talk.backdrop", "backdrop", "", b.r, true, false, None, Vec::new());
        talk::draw_talk(&mut ui, cx, gs, &mut g.faces, view, &b, false, Action::Talk);
    }

    // TRANSITIONS1: the running beat over everything, its full-screen widget last (a click skips it)
    g.draw_beat(&mut ui, cx, now);
    g.w = ui.widgets;
}

/// PR7-54 (`FB-364` (1)): the skill menu's preferred side beside a tile (false: the right). `ui/layouts/battle.json`
/// names no side yet; code keeps the menu clear either way (`skill_menu::menu_place_clear`).
const MENU_LEFT: bool = false;

/// BOSSUI1 BU12: the board cells the docked card and the menus keep clear of: the selected hero's and its round target's.
pub(super) fn card_keep_clear(w: &World, sel: Option<usize>) -> Vec<emberwake_battle::grid::V2> {
    let Some(h) = sel else { return Vec::new() };
    std::iter::once(h).chain(round_target(w, h)).filter_map(|u| w.grid.pos(u)).collect()
}

/// Hero `h`'s round target unit (none: no round state, or a cell target).
pub(super) fn round_target(w: &World, h: usize) -> Option<usize> {
    w.round_states.get(&h).and_then(|r| r.target).and_then(|t| match t {
        ri::RoundTarget::Unit(u) => Some(u),
        _ => None,
    })
}

/// ESCAPE-PLAN1 (5): the objective tiles the docked card keeps clear of: a hold escape's tiles and the board's exits.
pub(super) fn objective_cells(w: &World) -> Vec<emberwake_battle::grid::V2> {
    w.hold.iter().flat_map(|g| g.tiles.iter().copied()).chain(w.grid.exits.iter().copied()).collect()
}

// ---- VISCLAR1-D: the round banner --------------------------------------------------------

/// The round's `summary` before the next round's `prompt` in `room`: the prompt stays whole; the summary gives up its last
/// clauses to fit, all of it if it must (`width` measures a line).
pub(super) fn summary_and_prompt(summary: &str, prompt: &str, room: f32, mut width: impl FnMut(&str) -> f32) -> String {
    let mut kept = summary.trim_end_matches('.');
    loop {
        let line = format!("{kept}. {prompt}");
        if width(&line) <= room {
            return line;
        }
        match kept.rfind(", ") {
            Some(k) => kept = &kept[..k],
            None => return prompt.to_string(),
        }
    }
}

/// PR7-41 (PT-34): `message` without a lead that repeats the bar's `phase` word ("Round over: 1 break" -> "1 break").
pub(super) fn without_phase(message: &str, phase: &str) -> String {
    match message.strip_prefix(phase) {
        Some(rest) if rest.starts_with(':') || rest.starts_with('.') => rest[1..].trim_start().to_string(),
        _ => message.to_string(),
    }
}

/// The new round's name on a ribbon, centred in the upper part of `board`, at `alpha` (`round_banner_alpha`). Transient for
/// the text audit: it passes over the board by design.
pub(super) fn draw_round_banner(p: &mut Painter, board: Rect, round: i32, alpha: u8) {
    const SIZE: f32 = TITLE * 1.6;
    let text = format!("Round {round}");
    let (tw, lh) = (p.text_w(&text, SIZE), p.line_h(SIZE));
    let r = Rect::new(board.x + (board.w as f32 - tw - 32.0) as i32 / 2, board.y + board.h / 5, (tw + 32.0) as i32, (lh + 6.0) as i32);
    let scale = |a: u8| (a as u32 * alpha as u32 / 255) as u8;
    p.panel(r, with_alpha(th().card, scale(235)), with_alpha(th().gold, scale(255)), RADIUS);
    p.transient(|p| {
        p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + 3.0, &text, SIZE, with_alpha(th().gold, scale(255)));
    });
}

// ---- INPUT1-PB: the combat log by round and block (INPUT1 PB4) ---------------------------------------------------

/// PB4: how many of the resolving block's lines the compact log's card lists (its last ones), and how far the expanded
/// log's lines sit in under their part's header (px).
const LOG_CARD_LINES: usize = 12;
pub(super) const LOG_INDENT: f32 = 6.0;

/// PB4: a log part's header: "Round start", "Block <b>", "Round end".
pub(super) fn log_part_name(block: i32) -> String {
    match block {
        emberwake_battle::session::LOG_ROUND_START => "Round start".into(),
        emberwake_battle::session::LOG_ROUND_END => "Round end".into(),
        b => format!("Block {b}"),
    }
}

/// PB4: the agent link's `log_groups`: [{round, block (a number, "start" or "end"), lines}].
pub(super) fn log_groups_json(groups: &[emberwake_battle::session::LogGroup]) -> Value {
    Value::Array(groups.iter().map(|g| {
        let block = match g.block {
            emberwake_battle::session::LOG_ROUND_START => json!("start"),
            emberwake_battle::session::LOG_ROUND_END => json!("end"),
            b => json!(b),
        };
        json!({"round": g.round, "block": block, "lines": g.lines})
    }).collect())
}

// ---- PORTRAITFX1-P: the panel portraits (6.2 row 3) --------------------------------------------------------------

/// PORTRAITFX1-TL (user 2026-11-01): an enemy's timeline row holds no state cue (no fade, no wound tint, no other
/// hold) while the view masks its track (an unrevealed plan, TV4) or it is stealthed (the board's stealth veil, GD5);
/// party rows always show theirs.
fn timeline_row_plain(s: &Session, u: usize, masked: bool) -> bool {
    !s.w.allies.contains(&u) && (masked || s.w.units[u].has_status("stealth"))
}

/// `look` with no state a cue row can hold on but down (the board shows a fall): full HP, no statuses, conditions or
/// derived stats.
fn plain_look(look: &emberwake_battle::portrait::Look) -> emberwake_battle::portrait::Look {
    emberwake_battle::portrait::Look { hp: look.max_hp, max_hp: look.max_hp, down: look.down, statuses: Vec::new(), conditions: Vec::new(), derived: Vec::new(), ruleset: look.ruleset.clone(), unspent: 0 }
}

/// PORTRAITFX1-TL (6.2 row 4): `who`'s portrait in a timeline track's icon square `sq` (the widget's
/// `timeline.icon.<track>`), on the panels' dark backing, with `look`'s state rows on surface `timeline` through
/// `faces` (capped at the minimal tier); the initial letter without a file.
fn timeline_portrait(ui: &mut Ui<Action>, cx: &mut Cx, faces: &mut Faces, who: &Who, look: &emberwake_battle::portrait::Look, sq: Rect) {
    faces.draw(&mut ui.p, cx, who, look, TIMELINE_SURFACE, sq);
}

// ---- BATTLE-UI2: the screen's panels -----------------------------------------------------------------------------

/// BATTLE-UI2 (contract 3): both side panels' width (184 px) and the least height the hero panel under the plates takes.
pub(super) const SIDE_W: i32 = 92;
const HERO_MIN_H: i32 = 40;
/// BATTLE-UI2 (contract 3): the board's panel between the side panels (888 px at 1280 x 720).
pub(super) const BOARD: Rect = Rect::new(98, 28, 444, 290);

/// RULESET1 V1 on a foe plate's "when it lands" row: its hover, a title then a line per impact badge at its block.
fn impact_tip(t: &Intent, blocks: i32) -> Tip {
    use emberwake_battle::intent::HIT;
    let mut tip: Tip = vec![("Impacts this round".into(), th().gold)];
    for &(b, k) in t.impacts.iter().filter(|(b, _)| (1..=blocks).contains(b)) {
        tip.push((format!("B{b}: {}", if k == HIT { "a hit lands" } else { "an action lands" }), if k == HIT { th().foe } else { th().gold }));
    }
    if tip.len() == 1 {
        tip.push(("Nothing lands this round".into(), th().dim));
    }
    tip
}

/// BATTLE-UI2 contract 2: the strip's hero portrait and the right end of its title column (the blocks start at 92).
const STRIP_PORTRAIT: i32 = 18;

/// PR7-63 (XL-BATTLE1): the strip's goal line's top and the step to the protect line, for the title's and the small
/// line's heights: the goal starts where the title's glyphs end (a line's box has room above and below its letters,
/// about a third of the small line), never above Normal's 11; the step is two thirds of a small line, never under 6.
pub(super) fn strip_lines(title_h: f32, small_h: f32) -> (i32, i32) {
    let goal = (3.0 + title_h - small_h / 3.0).ceil() as i32;
    (goal.max(11), ((small_h * 2.0 / 3.0).ceil() as i32).max(6))
}
const TITLE_END: i32 = 88;
/// The hero panel's room for its Self and Ally buttons alone.
const HERO_BUTTONS_H: i32 = 15;

/// INPUT1 IF8, SL4: the hero panel's Self button (key S) and PR7-41 (PT-23) the Ally button beside it (the members the
/// hero can use a skill or pouch item on), their row at `y`; the y under the row.
fn self_ally_buttons(ui: &mut Ui<Action>, keys: &aegis_ui::controls::Controls, w: &World, h: usize, tx: i32, y: i32, planning: bool) -> i32 {
    let u = &w.units[h];
    let tip = vec![(format!("Self ({})", keys.hint(aegis_ui::screen::Key::Char('s'))), th().gold), (format!("{} targets itself: its Guard and self skills.", u.name), th().text)];
    let bw = ((ui.p.text_w("Self", SMALL) + 10.0).ceil() as i32).max(30);
    ui.button_full("hero.self", Rect::new(tx, y, bw, 11), "Self", Action::SelfTarget, planning, false, SMALL, tip);
    let others = w.allies.iter().any(|&a| a != h && ((w.units[a].is_active() && w.has(a)) || w.can_revive(a)));
    let tip = vec![("Ally".into(), th().gold), (format!("{} uses a skill or pouch item on another member: pick the member, then the skill (Ctrl+click on a member does the same).", u.name), th().text)];
    let aw = ((ui.p.text_w("Ally", SMALL) + 10.0).ceil() as i32).max(30);
    ui.button_full("hero.ally", Rect::new(tx + bw + 3, y, aw, 11), "Ally", Action::AllyTarget, planning && others, false, SMALL, tip);
    y + 13
}
