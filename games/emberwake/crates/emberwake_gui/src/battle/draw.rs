//! One frame of the battle screen (`draw`: the strip, party cards and hero panel, the board's call, hostile cards and intel, the bottom bar and log, the timeline view and talk), the panel portraits and the intel fit.

use aegis_ui::theme::th;
use super::board::{cell_rect, draw_flat, draw_stage, hp_bar, note_drawn, with_alpha, Board};
use super::input::{engage_positions, kind_name, rest_text, target_name, RowAct};
use super::marks::{arena_badge, arrows_of, broken_spans, draw_hold_pick, draw_tracers, focus_set, held_guard, hit_forecasts, meter_row, meter_text, meters_of, party_foresight_off, poise_bar, poise_shown, push_forecasts, push_text, seen_intents, HitForecast, HoldMarks, HookMarks, HELD, METER};
use super::overlays::{overlay_of, reach_text, Layers};
use super::playback::PlaySpeed;
use super::{Action, BattleScreen, ALLY, AP_PIPS, GRID_X, GRID_Y, PANEL, PANELS, PANEL_INSET, PANEL_PORTRAIT, PANEL_SURFACE, PANEL_TEXT_X, PLAYBACK_SPEEDS, TACTIC_LABELS, TALK_VEIL, TILE, TIMELINE_SURFACE, TV_PAD};
use super::tips::{ap_blocks, ap_gauge_of, class_names, committed, committed_icon_chains, derived_line, derived_rows, draw_badge_row, draw_block_label, inspect_tip, intent_glyph, kind_color, kind_icon, next_action, short_name, stance_name, unit_tip, ApAnim, BlockLook};
use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use crate::intent_text;
use crate::portraits::{Faces, Who};
use crate::talk;
use crate::timeline_view;
use crate::stage::{Geom, View};
use crate::screens::Cx;
use emberwake_battle::intent::Intent;
use aegis_ui::painter::Painter;
use aegis_ui::ui::{Gfx, Tip, Ui, BODY, H, RADIUS, SMALL, TITLE, W};
use emberwake_battle::grid::{GRID_H, GRID_W};
use emberwake_battle::round_intent::{self as ri, RoundIntent};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::{Phase, Session};
use aegis_core::json::s;
use emberwake_battle::skill::Kind;
use emberwake_battle::world::World;
use serde_json::{json, Value};
use std::time::Instant;

pub(super) fn draw(g: &mut BattleScreen, cx: &mut Cx, gfx: &mut Gfx, mouse: (i32, i32), live: bool) {
    let now = Instant::now();
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
    // DT8: the speaker's token wears the selection ring
    let talker = match (talk_gs.as_ref(), talk_view.as_ref()) {
        (Some(gs), Some(v)) => g.speaker_unit(cx, gs, &s(v, "speaker")),
        _ => None,
    };
    let mut ui: Ui<Action> = Ui::new(gfx, mouse, live);
    g.menu_row = ((ui.p.line_h(BODY) + 2.5).ceil() as i32).max(11);
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
    ui.p.panel(Rect::new(2, 2, W - 4, 24), th().game("battle_panel", PANEL), th().edge, RADIUS);
    let hero_name = sel.map(|u| w.units[u].name.clone()).unwrap_or_else(|| "No hero".into());
    // UI5: the strip row leads with the hero's intent glyph (the board and the cards share it)
    let title_w = match sel.and_then(round_of) {
        Some(i) => {
            intent_glyph(&mut ui.p, &w.cat, i, Rect::new(78, 8, 10, 10));
            68.0
        }
        None => 80.0,
    };
    ui.p.text_fit(8.0, 9.0, &format!("{hero_name} plan"), TITLE, title_w, th().gold);
    ui.push("strip.title", "label", format!("{hero_name} plan"), Rect::new(8, 8, title_w as i32, ui.p.line_h(TITLE).ceil() as i32), true, false, None, Vec::new());
    // HOOKS1-UI slice 3 (UM7): the selected hero's meters on a thin row under the title
    if let Some(u) = sel {
        let ms = meters_of(w, u, &layers);
        if !ms.is_empty() {
            let mr = Rect::new(8, 21, 76, 3);
            meter_row(&mut ui.p, mr, &ms);
            let tip: Tip = ms.iter().map(|m| (meter_text(m), METER)).collect();
            ui.push("strip.meters", "meters", tip.iter().map(|t| t.0.clone()).collect::<Vec<_>>().join("; "), Rect::new(mr.x, mr.y - 1, mr.w, mr.h + 2), true, false, None, tip);
        }
    }
    // INPUT1 IF5: a hovered skill-menu row previews the round as if picked (copies of the world and the plan, kept for
    // the battle's revision); GH2 a: a member whose ghost waits for Confirm shows it from copies too
    let timeline = ri::on(w);
    let hovered_row = g.popup.as_ref().filter(|p| p.menu.is_some() && live).and_then(|p| (0..p.rows.len()).find(|&k| p.row(g.menu_row, k).contains(mouse.0, mouse.1)));
    let hovered_pick = g.popup.as_ref().and_then(|p| Some((p.menu?, p.rows.get(hovered_row?)?.clone())));
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
                ((e.start_block, e.end_block(pw), sk.name.clone(), sk.kind.clone(), e.target.filter(|t| *t != u).map(|t| w.units[t].name.clone()), e.total_ap(), e.est_damage), chain)
            })
            .unzip(),
        _ => (Vec::new(), Vec::new()),
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
        Some(u) if w.units[u].is_active() => compiled.iter().flat_map(|st| st.entries.iter()).map(|e| emberwake_battle::command::hp_cost_of(pw, &w.cat.skills[e.skill])).collect(),
        _ => Vec::new(),
    };
    // HOOKS1-UI slice 4: each planned entry's push and pull forecasts (rule forced_move; planning, layer `pushes`)
    let pushes: Vec<Vec<emberwake_battle::forced::Forecast>> = match sel {
        Some(u) if planning && layers.pushes && w.allies.contains(&u) && w.units[u].is_active() && !previewing => push_forecasts(s, u),
        _ => Vec::new(),
    };
    // TEXTFIT1: the controls right of the blocks take their text's width; the blocks give up width first (34 down to
    // 26 each), then the two dropdowns, the wider first (their text cuts).
    let party_label = format!("Party: {}", TACTIC_LABELS[g.party_tactic]);
    let hero_label = format!("Hero: {}", TACTIC_LABELS[sel.map_or(g.hero_tactic, |u| if timeline { g.tactic_of(u) } else { g.hero_tactic })]);
    // ENCOUNTER2-C4: a locked round still to play: Skip in Confirm's place (the rest of the round at once)
    let skip = g.can_skip();
    let (label, action, enabled) = if planning {
        // ENCOUNTER2 H1: planning cannot be confirmed before the holder is picked
        ("Confirm round", Action::Confirm, !s.needs_stay_pick())
    } else if skip {
        ("Skip", Action::Skip, true)
    } else if s.phase == Phase::Replay && s.can_advance_round() {
        ("Next round", Action::NextRound, !g.pending_next_round)
    } else {
        (if s.combat_over { "Battle over" } else if s.phase == Phase::Replay { "Round limit" } else { "Resolving" }, Action::NextRound, false)
    };
    let mut fit_w = |text: &str, pad: f32, min: i32| ((ui.p.text_w(text, BODY) + pad).ceil() as i32).max(min);
    let (mut party_w, mut hero_w) = (fit_w(&party_label, 18.0, 82), fit_w(&hero_label, 18.0, 80));
    let (auto_w, clear_w, confirm_w) = (fit_w("Auto", 10.0, 34), fit_w("Clear", 10.0, 34), fit_w(label, 12.0, 60));
    let bx = 92;
    let blocks = s.def.total_blocks.max(1);
    // INPUT1-AP (AP6): under the round intent the blocks leave the AP gauge its 12 pips and a label ("-<n>") at their
    // end, kept whether or not a hero is selected, so the strip does not shift
    let gauge_w = if timeline { aegis_ui::pips::pips_w(AP_PIPS) + aegis_ui::pips::LABEL_GAP + aegis_ui::pips::label_w(&mut ui, "-00") } else { 0 };
    let room = W - 6 - bx - 6 - 8 - auto_w - clear_w - confirm_w - if timeline { gauge_w + 2 } else { 0 };
    // AP6: with the gauge the blocks give up width down to 20 each (their span labels cut, TEXTFIT1) before the menus
    let bw = ((room - party_w - hero_w) / blocks).clamp(if timeline { 20 } else { 26 }, 34);
    let mut over = blocks * bw + party_w + hero_w - room;
    while over > 0 && (party_w > 82 || hero_w > 80) {
        let (big, floor) = if party_w - 82 >= hero_w - 80 { (&mut party_w, 82) } else { (&mut hero_w, 80) };
        let cut = over.min(*big - floor);
        *big -= cut;
        over -= cut;
    }
    // INPUT1 PB2: a timeline battle's strip follows the playback's resolving block (classic: the current block)
    let live_block = if aids { cursor.unwrap_or(0) } else if !planning && s.phase == Phase::Resolving { s.current_block } else { 0 };
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
                    tip.push((format!("{} AP", e.5), ALLY));
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
            tip.push(("Click: accept the auto-fill up to here (F: all of it)".into(), th().dim));
        }
        ui.push(format!("strip.block{b}"), "block", entry.map(|e| e.2.clone()).unwrap_or_else(|| "empty".into()), wr, true, live_block == b, accept.map(Action::GhostSlot), tip);
    }
    // Each action is one continuous span over its blocks, coloured by kind (COMBAT-UI3).
    let mut strip_labels = Vec::new();
    for (k, e) in entries.iter().enumerate() {
        let r = Rect::new(bx + (e.0 - 1) * bw, 6, (e.1 - e.0 + 1) * bw - 1, 16);
        let live = live_block >= e.0 && live_block <= e.1;
        // RULESET1-U (UI2): under the round intent Rest blocks are dim (the planner fills every spare block with them)
        let dim = timeline && e.3 == "REST";
        // INPUT1 GH8: a ghost span (the planner's, after the picks) in the engine's ghost look; a ghost Rest stays dim
        let ghost = ghosts.get(k).copied().unwrap_or(false) && !dim;
        // VISCLAR1-C (decision "Battle view answers, part 2"): one solid colour a block, no wind-up or impact shading (the
        // full timeline view keeps its parts); an auto-filled block fades, with no outline
        let fill = if dim { rgb(34, 40, 54) } else if ghost { aegis_ui::timeline::ghost_fill(kind_color(&e.3)) } else { kind_color(&e.3) };
        let edge = if live { th().gold } else if ghost { rgba(0, 0, 0, 0) } else if dim { rgba(255, 255, 255, 24) } else if timeline { fill } else { rgba(255, 255, 255, 50) };
        ui.p.panel(r, fill, edge, RADIUS);
        if timeline {
            // VISCLAR1-B (mock C): the skill's icon and name centred, the icon alone when the name does not fit
            let look = if dim { BlockLook::Dim } else if ghost { BlockLook::Ghost } else { BlockLook::Chosen };
            strip_labels.push(draw_block_label(&mut ui, &mut cx.art, r, &e.2, icons.get(k).map_or(&[], Vec::as_slice), look));
        } else {
            ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 4.0, &e.2, SMALL, r.w as f32 - 6.0, th().text);
        }
        if broken.get(k).copied().unwrap_or(false) && layers.breaks {
            let mark = aegis_ui::timeline::Mark { key: "break".into(), block: e.0, glyph: "BREAK".into(), colour: th().foe, tip: vec![("BREAK".into(), th().foe), (format!("{}: cancelled by a poise break (AP paid stays spent)", e.2), th().text)] };
            let mr = Rect::new(r.x + 1, 7, (bw - 3).min(r.w - 2), 14);
            aegis_ui::timeline::draw_mark(&mut ui, &format!("strip.mark.{k}"), &mark, mr);
        }
    }
    g.strip_labels = strip_labels;
    // HOOKS1-UI slice 1 (AH3): a badge on each block at whose end an arena hazard strikes, over the spans
    for b in (1..=s.def.total_blocks).filter(|b| marks.show_arena && marks.arena.iter().any(|t| t.blocks.contains(b))) {
        arena_badge(&mut ui.p, Rect::new(bx + (b - 1) * bw, 6, bw - 1, 16));
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
            ui.button_full("playback.next", Rect::new(x, 6, (end - x).max(10), 16), "Next block", Action::NextBlock, g.waiting.is_some(), false, BODY, vec![("Next block (N)".into(), th().gold), ("Plays the next block, then waits again.".into(), th().text)]);
        }
        x = end + 2;
    } else {
        ui.dropdown("strip.party", Rect::new(x, 6, party_w, 16), &party_label, Action::PartyMenu, planning, vec![("Party tactic".into(), th().gold), ("AI-drafts every hero with a tactic (you can edit the steps after).".into(), th().text)]);
        x += party_w + 2;
        ui.dropdown("strip.hero", Rect::new(x, 6, hero_w, 16), &hero_label, Action::HeroMenu, planning && sel.is_some(), vec![("Hero tactic".into(), th().gold), ("AI-drafts the selected hero only.".into(), th().text)]);
        x += hero_w + 2;
    }
    ui.button_full("strip.auto", Rect::new(x, 6, auto_w, 16), "Auto", Action::Autoplay, !s.combat_over, g.auto, BODY, vec![("Autoplay".into(), th().gold), ("Drafts with the party tactic, confirms and plays rounds until the battle ends. Click again to stop.".into(), th().text)]);
    x += auto_w + 2;
    ui.button_full("strip.clear", Rect::new(x, 6, clear_w, 16), "Clear", Action::Clear, planning && sel.is_some(), false, BODY, vec![(if timeline { "Clears the selected hero's picks (the planner fills the round again)." } else { "Clears the selected hero's steps." }.into(), th().text)]);
    x += clear_w + 2;
    if skip {
        ui.button_full("battle.skip", Rect::new(x, 6, W - 6 - x, 16), label, action, enabled, true, BODY, vec![("Skip".into(), th().gold), ("Resolves the rest of this scripted round at once.".into(), th().text)]);
    } else {
        ui.button_full("strip.confirm", Rect::new(x, 6, W - 6 - x, 16), label, action, enabled, planning, BODY, vec![(label.into(), th().gold)]);
    }

    // -- Left: party cards and the selected hero.
    ui.p.panel(Rect::new(2, 28, 124, 290), th().game("battle_panel", PANEL), th().edge, RADIUS);
    let alive = w.allies.iter().filter(|a| w.units[**a].is_active()).count();
    ui.p.text(8.0, 32.0, &format!("Party {alive}/{}", w.allies.len()), TITLE, ALLY);
    let mut y = 44;
    // UI5: the party member the mouse is over (its card or its token) fills the threat intel panel
    let mut hovered_ally = None;
    for &a in &w.allies {
        let u = &w.units[a];
        // HOOKS1-UI slice 3: a meter row grows the card by 4 (UM7)
        let ms = meters_of(w, a, &layers);
        let r = Rect::new(6, y, 116, if ms.is_empty() { 30 } else { 34 });
        let is_sel = Some(a) == sel;
        let hover = ui.hover(r);
        if hover {
            hovered_ally = Some(a);
        }
        ui.p.panel(r, if is_sel { th().card_hot } else if hover { th().game("roster_hot", rgba(28, 40, 66, 255)) } else { th().card }, if is_sel { th().gold } else { th().edge }, RADIUS);
        // UI5: the member's intent glyph at the card's top right (the name gives way)
        let glyph = round_of(a).filter(|_| u.is_active());
        if let Some(i) = glyph {
            intent_glyph(&mut ui.p, &w.cat, i, Rect::new(r.x + r.w - 13, r.y + 2, 10, 10));
        }
        ui.p.text_fit(r.x as f32 + 4.0, r.y as f32 + 1.5, &u.name, BODY, if glyph.is_some() { 94.0 } else { 108.0 }, if u.is_active() { ALLY } else { th().dim });
        hp_bar(&mut ui.p, Rect::new(r.x + 4, r.y + 12, 108, 4), u.hp, u.max_hp, th().good);
        // RULESET1-U slice 5: the member's poise bar under its HP bar (rule poise)
        if let Some(st) = poise_shown(w, a, &layers) {
            poise_bar(&mut ui.p, Rect::new(r.x + 4, r.y + 16, 108, 2), &st);
        }
        ui.p.text(r.x as f32 + 4.0, r.y as f32 + 18.0, &format!("HP {}/{}", u.hp, u.max_hp), SMALL, th().text);
        ui.p.text_right(r.x as f32 + r.w as f32 - 4.0, r.y as f32 + 18.0, &format!("AP {}/{}", u.ap, u.max_ap), SMALL, ALLY);
        meter_row(&mut ui.p, Rect::new(r.x + 4, r.y + 29, 108, 3), &ms);
        if layers.guard && held_guard(s, a).is_some() {
            ui.p.panel(r.inset(1), rgba(0, 0, 0, 0), HELD, RADIUS);
        }
        let tip = unit_tip(s, a, sel, &tells, &rounds, &classes);
        ui.push(format!("party.{a}"), "card", format!("{} HP {}/{} AP {}/{}", u.name, u.hp, u.max_hp, u.ap, u.max_ap), r, true, is_sel, Some(Action::Select(a)), tip);
        y += r.h + 3;
    }
    g.panels.push(("party", Rect::new(2, 28, 124, y - 26)));
    if let Some(h) = sel {
        let u = &w.units[h];
        let r = Rect::new(6, y + 2, 116, 318 - y - 6);
        g.panels.push(("hero", r));
        ui.p.panel(r, th().card, th().edge, RADIUS);
        // PORTRAITFX1-P (6.2 row 3): the hero's portrait at the panel's top-left; the name, the corner and the Self
        // button beside it, 40 px in
        let (who, look) = g.unit_portrait(cx, h);
        let face = panel_portrait(&mut ui, cx, &mut g.panel_faces, &who, &look, r, "hero.portrait");
        let (tx, tw) = (r.x + PANEL_TEXT_X, (r.w - PANEL_TEXT_X - 4) as f32);
        ui.p.text_fit(tx as f32, r.y as f32 + 2.0, &u.name, BODY, tw, th().gold);
        // RULESET1-U: under the round intent the corner names the hero's intent (the classic stance is inert: IN3);
        // PORTRAITFX1-P: on the line under the name
        let corner = if timeline { ri::intent_of(pw, h).map_or_else(|| "No intent".to_string(), |i| i.label(&pw.cat)) } else { stance_name(u).to_string() };
        let row2 = r.y + 2 + (ui.p.line_h(BODY).ceil() as i32).max(10);
        ui.p.text_fit(tx as f32, row2 as f32, &corner, SMALL, tw, th().dim);
        let row3 = row2 + (ui.p.line_h(SMALL).ceil() as i32).max(8) + 1;
        // INPUT1 IF8, SL4: under the round intent the panel's one control is the Self button (key S)
        let top = if timeline {
            let tip = vec![("Self (S)".into(), th().gold), (format!("{} targets itself: its Guard and self skills.", u.name), th().text)];
            let bw = ((ui.p.text_w("Self", SMALL) + 10.0).ceil() as i32).max(30);
            ui.button_full("hero.self", Rect::new(tx, row3, bw, 11), "Self", Action::SelfTarget, planning, false, SMALL, tip);
            row3 + 13
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
            lines.push((format!("Speed {}  Reach {}", u.speed, reach_text(w, h)), th().text));
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
        let ap_line = (format!("AP left {}/{}", if s.plan_confirmed { u.ap } else { st.ap_left }, u.max_ap), ALLY);
        if timeline {
            // INPUT1 IF8: the round first: "Intent: <Intent> (<kind>) from <first pick | tactic>" (or carried over), a
            // ghost's intent change that waits for Confirm (IN6), the target, the picks, the planned commands without
            // the Rest blocks (the ghost dim), then the Rest gain; the stats after.
            let intent = ri::intent_of(pw, h);
            let picks = rip::picks_of(w, h);
            let from = match picks.iter().find(|p| p.holds_target()) {
                Some(p) => format!(" from {}", rip::pick_name(w, p)),
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
            if !picks.is_empty() && !s.plan_confirmed {
                lines.push((format!("Picks: {}", picks.iter().map(|p| rip::pick_name(w, p)).collect::<Vec<_>>().join(", ")), th().gold));
            }
            if s.plan_confirmed {
                for (start, _, name, _, tgt) in committed(s, h) {
                    let tgt = tgt.map(|t| format!(" > {}", short_name(&t))).unwrap_or_default();
                    lines.push((format!("B{start} {name}{tgt}"), th().text));
                }
            } else {
                for (k, e) in st.entries.iter().enumerate().filter(|(_, e)| e.step_index >= 0 && emberwake_battle::skill::kind(&w.cat.skills[e.skill]) != Kind::Rest) {
                    let name = &w.cat.skills[e.skill].name;
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
            statuses(&mut lines);
            stats(&mut lines);
            lines.push((format!("Gear: {weapon}"), th().gold));
        } else {
            stats(&mut lines);
            lines.push(ap_line);
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
                    let name = &w.cat.skills[e.skill].name;
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
            let (lx, lw) = if beside { (tx as f32, tw) } else { (r.x as f32 + 4.0, 108.0) };
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
        _ => Vec::new(),
    };
    if let Some(o) = overlay.as_mut() {
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
    let board = Board { s, tokens: &mut g.tokens, floats: &g.floats, flashes: &g.flashes, popup: g.popup.is_some() || talk_view.is_some(), anim_ms: g.clock.ms, sel, talker, planning, tells: &tells, show_tells, card_hover, inspect: pinned, focus: &focus, fx: &g.fx, shift, sidesteps: &g.sidesteps, overlay: overlay.as_ref(), rounds: &rounds, layers, alt: cx.held.alt, marks: &marks, classes: &classes, pushes: &push_list, hold: hold.as_ref() };
    let (board_hover, flat_hover) = match view {
        View::Stage => (draw_stage(board, &stage_set, &mut ui, cx, &tuning, now), None),
        View::Flat => (None, draw_flat(board, &mut ui, cx, &tuning, now)),
    };
    g.focus_shown = focus_set(w, sel, board_hover.or(flat_hover).or(card_hover), pinned);
    // INPUT1 PB1: the impact tracers above the tokens (group `tracers`), shaken with the board, inside its panel
    if aids && layers.tracers && !g.tracers.is_empty() {
        let geo = Geom::new(view, &tuning);
        ui.p.clip(Some(Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4)));
        ui.p.shift(shift.0, shift.1);
        draw_tracers(&mut ui.p, &g.tracers, &geo, g.clock.now(), cx.settings.reduce_motion());
        ui.p.shift(-shift.0, -shift.1);
        ui.p.clip(None);
    }
    // ENCOUNTER2 H1: the hold pick over the board while round 1 waits for it
    if planning && s.needs_stay_pick() {
        draw_hold_pick(&mut ui, s);
    }
    // WINDUP1 6.2: the clip each token drew (the agent link's `tokens`)
    note_drawn(&mut g.tokens, s, &mut cx.art, g.clock.ms as u64);

    // -- Right: hostile cards and the hovered (or first) foe's detail card.
    let right = Rect::new(W - 126, 28, 124, 290);
    ui.p.panel(right, th().game("battle_panel", PANEL), th().edge, RADIUS);
    let foes_active = w.enemies.iter().filter(|e| w.units[**e].is_active()).count();
    ui.p.text(right.x as f32 + 6.0, 32.0, &format!("Hostiles {foes_active}"), TITLE, th().foe);
    let mut y = 44;
    let mut hovered_foe = None;
    for &e in &w.enemies {
        let u = &w.units[e];
        if w.grid.pos(e).is_none() && u.is_alive() && !u.is_withdrawn {
            continue; // reinforcements not on the field yet
        }
        if y > 206 {
            break;
        }
        // TEXTFIT1: the Next line wraps to a second line when it does not fit (the card grows).
        let tell = tells.iter().find(|t| t.unit == e);
        // RULESET1-U slice 4 (UI5): under the round intent the line leads with the intent ("Assault: Wolf; Cut B1")
        let next = tell.map(|t| intent_text::round_card_line(w, t)).unwrap_or_else(|| format!("Next: {}", next_action(s, e)));
        let next_w = if tell.is_some_and(|t| t.revealed) { 98.0 } else { 110.0 };
        let next_lines = if u.is_active() { ui.p.wrap(&next, SMALL, next_w) } else { Vec::new() };
        let two = next_lines.len() > 1;
        // V1: the impact badges on a row of the round's blocks under the line (rule plan_badges, while planning)
        let badges = tell.filter(|_| u.is_active() && emberwake_battle::intent::badges_on(w) && layers.badges);
        let row_h = if badges.is_some() { 6 } else { 0 };
        // HOOKS1-UI slice 3: a meter row grows the card by 5 (UM7)
        let ms = meters_of(w, e, &layers);
        let meter_h = if ms.is_empty() { 0 } else { 5 };
        let r = Rect::new(right.x + 4, y, 116, if two { 22 + ui.p.line_h(SMALL).ceil() as i32 } else { 22 } + row_h + meter_h);
        let on_board = match view {
            View::Stage => board_hover == Some(e),
            View::Flat => w.grid.pos(e).is_some_and(|p| ui.hover(cell_rect(p))),
        };
        let hover = ui.hover(r) || on_board;
        if hover {
            hovered_foe = Some(e);
        }
        ui.p.panel(r, if hover { th().game("foe_hot", rgba(48, 28, 38, 255)) } else { th().card }, if hover { th().foe } else { th().edge }, RADIUS);
        let col = if u.is_active() { th().foe } else { th().dim };
        // UI5: the enemy's intent glyph before its name (a tell, so only while the tells show)
        // slice 5: from the round records, as the tokens draw it, so the two agree during the playback too
        let glyph = round_of(e).filter(|_| show_tells && u.is_active());
        if let Some(i) = glyph {
            intent_glyph(&mut ui.p, &w.cat, i, Rect::new(r.x + 3, r.y + 2, 10, 10));
        }
        let gx = if glyph.is_some() { 12.0 } else { 0.0 };
        ui.p.text_fit(r.x as f32 + 3.0 + gx, r.y as f32 + 1.5, &short_name(&u.name), BODY, 70.0 - gx, col);
        if let Some(t) = badges {
            let row = Rect::new(r.x + 3, r.y + r.h - 7, r.w - 6, 4);
            let tip = draw_badge_row(&mut ui.p, t, row, s.def.total_blocks.max(1));
            ui.push(format!("foe.{e}.badges"), "badges", tip.iter().skip(1).map(|x| x.0.clone()).collect::<Vec<_>>().join("; "), row, true, false, None, tip);
        }
        ui.p.text_right(r.x as f32 + r.w as f32 - 3.0, r.y as f32 + 1.5, &format!("{}/{}", u.hp, u.max_hp), BODY, if u.is_active() { th().good } else { th().dim });
        if u.is_active() {
            if two {
                ui.p.text(r.x as f32 + 3.0, r.y as f32 + 11.5, &next_lines[0], SMALL, th().gold);
                // the rest of the line as written (a word `wrap` broke mid-word stays whole)
                let rest = next.get(next_lines[0].len()..).unwrap_or("").trim_start();
                ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 11.5 + ui.p.line_h(SMALL), rest, SMALL, next_w, th().gold);
            } else {
                ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 11.5, &next, SMALL, next_w, th().gold);
            }
            if tell.is_some_and(|t| t.revealed) {
                if let Some(img) = cx.art.image(&emberwake_content::art::badge("foresight"), 9, 9) {
                    ui.p.image(img, Rect::new(r.x + r.w - 11, r.y + 11, 9, 9));
                }
            }
        } else {
            ui.p.text(r.x as f32 + 3.0, r.y as f32 + 11.5, if u.is_withdrawn { "Fled" } else { "Down" }, SMALL, th().dim);
        }
        meter_row(&mut ui.p, Rect::new(r.x + 3, r.y + r.h - row_h - 5, r.w - 6, 3), &ms);
        if layers.guard && held_guard(s, e).is_some() {
            ui.p.panel(r.inset(1), rgba(0, 0, 0, 0), HELD, RADIUS);
        }
        let tip = unit_tip(s, e, sel, &tells, &rounds, &classes);
        ui.push(format!("foe.{e}"), "card", format!("{} {}/{} {}", u.name, u.hp, u.max_hp, next), r, true, hover, Some(Action::UnitClick(e)), tip);
        y += r.h + 2;
    }
    g.panels.push(("hostiles", Rect::new(right.x, 28, right.w, y - 26)));
    debug_assert!(g.panels.iter().all(|p| PANELS.contains(&p.0)), "a panel recorded under a name PANELS lacks");
    // RULESET1-U slice 4 (UI5): under plan_badges the intel panel shows the hovered unit of either side (its card or
    // its token); classic keeps the hovered (or first) foe
    let ally_hover = if rounds.is_empty() {
        None
    } else {
        hovered_ally.or(match view {
            View::Stage => board_hover.filter(|u| w.allies.contains(u)),
            View::Flat => w.allies.iter().copied().find(|&a| w.units[a].is_active() && w.grid.pos(a).is_some_and(|p| ui.hover(cell_rect(p)))),
        })
    };
    // INPUT1 IS1: an inspected unit pins the panel (until another inspect, Esc or the round's end)
    let detail_foe = pinned.or(hovered_foe).or(ally_hover).or_else(|| w.enemies.iter().copied().find(|e| w.units[*e].is_active() && w.grid.pos(*e).is_some()));
    if let Some(e) = detail_foe {
        let r = Rect::new(right.x + 4, y + 2, 116, 318 - y - 6);
        g.panels.push(("intel", r));
        ui.p.panel(r, th().card, if w.allies.contains(&e) { ALLY } else { th().foe }, RADIUS);
        let mut ly = r.y as f32 + 3.0;
        let mut lines = if pinned.is_some() { inspect_tip(s, e, sel, &tells, &rounds, &classes, show_tells) } else { unit_tip(s, e, sel, &tells, &rounds, &classes) };
        // RULESET1-U slice 5: the selected hero's planned hits on this unit, forecast (rule poise)
        if let Some(h) = sel.filter(|h| layers.forecasts && planning && ri::on(w) && w.units[*h].is_active()) {
            // VISCLAR1-C: the strip's list when it has one (a preview leaves that empty)
            let own;
            let list = if previewing {
                own = hit_forecasts(s, h);
                &own
            } else {
                &hits
            };
            for f in list.iter().flatten().filter(|f| f.target == e) {
                lines.push(f.short(w));
            }
        }
        // the agent link names the panel's unit (UI5: either side; classic adds no widget)
        if !rounds.is_empty() || pinned.is_some() {
            ui.push("intel.unit", "panel", lines.first().map(|l| l.0.clone()).unwrap_or_default(), r, true, false, None, Vec::new());
        }
        // PORTRAITFX1-P (6.2 row 3): the unit's portrait at the panel's top-left, the lines beside it 40 px in
        let (who, look) = g.unit_portrait(cx, e);
        let face = panel_portrait(&mut ui, cx, &mut g.panel_faces, &who, &look, r, "intel.portrait");
        let beside = (face.y + face.h + 1) as f32 - ly;
        // HOOKS1-UI slice 5: the lines that fit (the Level/Speed line gives way first, then "More on its hover card")
        for (part, col, head, narrow) in intel_parts_at(&mut ui.p, &lines, (r.h - 12) as f32, beside) {
            let size = if head { BODY } else { SMALL };
            let x = if narrow { r.x + PANEL_TEXT_X } else { r.x + 4 };
            ui.p.text(x as f32, ly, &part, size, col);
            ly += if head { INTEL_HEAD } else { INTEL_LINE };
        }
    }

    // -- Bottom bar: playback, round and status, the last log line.
    ui.p.panel(Rect::new(2, 322, W - 4, 36), th().game("battle_panel", PANEL), th().edge, RADIUS);
    ui.button_full("bar.restart", Rect::new(6, 325, 40, 14), "Restart", Action::Restart, !g.world_mode(), false, BODY, vec![("Restart the battle (R)".into(), th().text)]);
    ui.button_full("bar.step", Rect::new(48, 325, 34, 14), "Step", Action::Step, !planning || s.plan_confirmed, false, BODY, vec![("Resolve one action (Space)".into(), th().text)]);
    // Test Arena only: leave at any time (Esc does the same); a world battle is left only through its end.
    let ox = if g.world_mode() {
        0
    } else {
        ui.button_full("bar.exit", Rect::new(84, 325, 30, 14), "Exit", Action::Leave, true, false, BODY, vec![("Back to the screen you came from (Esc)".into(), th().text)]);
        32
    };
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
    ui.p.text_fit((200 + ox) as f32, 326.0, &g.message, BODY, (430 - ox - chip_w) as f32, th().text);
    ui.push("bar.status", "label", format!("Round {} {phase}: {}", s.current_round, g.message), Rect::new(88 + ox, 325, 540 - ox - chip_w, 12), true, false, None, Vec::new());
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
    ui.button_full("bar.log", Rect::new(W - 54, 341, 48, 14), if g.show_log { "Close" } else { "Expand" }, Action::ToggleLog, true, g.show_log, BODY, vec![("Full combat log (L); the wheel scrolls".into(), th().text)]);

    let outcome = g.outcome();
    if !outcome.is_empty() {
        let r = Rect::new(GRID_X + 92, GRID_Y + 110, 200, 56);
        ui.p.panel(r, th().game("outcome_fill", rgba(10, 14, 22, 240)), th().gold, 4.0);
        let (title, col) = match outcome.as_str() {
            "won" => ("Victory", th().good),
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

    if g.show_log {
        let r = Rect::new(GRID_X, GRID_Y + 4, GRID_W * TILE, GRID_H * TILE - 8);
        // ENCOUNTER2-U (decision "RULESET1-A3 loose ends"): the resolver log with the encounter's phase and kill lines
        let log = s.log_lines(0, 0);
        ui.push("log.panel", "panel", format!("{} lines", log.len()), r, true, true, Some(Action::Step), Vec::new());
        ui.p.panel(r, th().game("log_fill", rgba(10, 14, 24, 246)), th().edge, 3.0);
        ui.p.text(r.x as f32 + 6.0, r.y as f32 + 3.0, &format!("Combat log ({} entries)", log.len()), TITLE, th().gold);
        // the rows: text, colour, x offset, a rule after it; INPUT1 PB4: a timeline battle's lines under their round's
        // "Round <r>" and their part's DIM "Block <b>" ("Round start", "Round end") with a 1 px rule, 6 px in
        let mut wrapped: Vec<(String, Rgba, f32, bool)> = Vec::new();
        let wrap_line = |p: &mut Painter, out: &mut Vec<(String, Rgba, f32, bool)>, l: &str, indent: f32| {
            let col = if l.starts_with("[Phase]") { th().gold } else if l.starts_with("[Kill]") { th().foe } else if l.starts_with("[Attack]") { th().text } else if l.starts_with("[Support]") { th().good } else if l.contains("Failed") || l.contains("Out of Range") { th().foe } else { th().dim };
            for (k, part) in p.wrap(l, SMALL, (r.w - 22) as f32 - indent).into_iter().enumerate() {
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
        ui.p.panel(r, th().game("timeline_fill", rgba(10, 14, 24, 250)), th().edge, RADIUS);
        let on_track = |k: &str| Action::TimelinePick(timeline_view::unit_of(k));
        let faces = &mut g.timeline_faces;
        let icon = |ui: &mut Ui<Action>, track: &aegis_ui::timeline::Track, sq: Rect| {
            if let Some((who, look)) = icons.get(&track.key) {
                timeline_portrait(ui, cx, faces, who, look, sq);
            }
        };
        t.draw_icons(&mut ui, "timeline", Rect::new(r.x + TV_PAD, r.y + TV_PAD, r.w - 2 * TV_PAD, r.h - 2 * TV_PAD), |_| Action::TimelinePick(None), on_track, |k, _| on_track(k), icon);
        g.tv_shown = Some(t);
    } else {
        g.tv_shown = None;
    }

    if let Some(pop) = g.popup.clone() {
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

    g.w = ui.widgets;
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

/// PORTRAITFX1-P: `who`'s portrait at panel `r`'s top-left (`PANEL_PORTRAIT` px, `PANEL_INSET` in) on the talk box's
/// dark panel, with `look`'s state rows on surface `panel` (the initial letter without a file); its widget `id` (kind
/// `portrait`, no action). The portrait's rect.
#[allow(clippy::too_many_arguments)]
fn panel_portrait(ui: &mut Ui<Action>, cx: &mut Cx, faces: &mut Faces, who: &Who, look: &emberwake_battle::portrait::Look, r: Rect, id: &str) -> Rect {
    let pr = Rect::new(r.x + PANEL_INSET, r.y + PANEL_INSET, PANEL_PORTRAIT, PANEL_PORTRAIT);
    ui.p.panel(pr, th().game("portrait_well", rgb(22, 30, 48)), th().edge, 2.0);
    faces.draw(&mut ui.p, cx, who, look, PANEL_SURFACE, pr);
    ui.push(id, "portrait", who.name.clone(), pr, true, false, None, Vec::new());
    pr
}

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
    ui.p.panel(sq, th().game("portrait_well", rgb(22, 30, 48)), th().edge, 2.0);
    faces.draw(&mut ui.p, cx, who, look, TIMELINE_SURFACE, Rect::new(sq.x + 1, sq.y + 1, sq.w - 2, sq.h - 2));
}

// ---- HOOKS1-UI slice 5: the intel panel fits its lines ------------------------------------------------------------

/// The intel panel's line steps: its head (BODY) and the rest (SMALL), and its text width.
pub(super) const INTEL_HEAD: f32 = 10.0;
pub(super) const INTEL_LINE: f32 = 8.0;
const INTEL_W: f32 = 108.0;
/// The panel's last line when its lines do not all fit (the unit's hover card has them all).
pub(super) const INTEL_MORE: &str = "More on its hover card";

/// HOOKS1-UI slice 5 (decision "RULESET1-B2 tuning and HOOKS1-UI slice 3 answers"): the intel panel's wrapped lines
/// that fit in `room` px ((text, colour, head)): when they run past it the "Level ..." line (level, class, speed, reach)
/// gives way first; when they still do not fit the panel keeps what fits and ends with `INTEL_MORE`.
#[cfg(test)]
pub(super) fn intel_parts(p: &mut Painter, lines: &[(String, Rgba)], room: f32) -> Vec<(String, Rgba, bool)> {
    intel_parts_at(p, lines, room, 0.0).into_iter().map(|(t, c, h, _)| (t, c, h)).collect()
}

/// `intel_parts` beside the panel's portrait (PORTRAITFX1-P): a line that starts within the first `beside` px of the
/// text wraps to the room right of the portrait and is marked narrow (drawn `PANEL_TEXT_X` in).
fn intel_parts_at(p: &mut Painter, lines: &[(String, Rgba)], room: f32, beside: f32) -> Vec<(String, Rgba, bool, bool)> {
    let narrow_w = INTEL_W - (PANEL_TEXT_X - 4) as f32;
    let wrap = |p: &mut Painter, lines: &[(String, Rgba)]| {
        let mut out = Vec::new();
        let mut at = 0.0;
        for (i, (t, col)) in lines.iter().enumerate() {
            let narrow = at < beside;
            for part in p.wrap(t, if i == 0 { BODY } else { SMALL }, if narrow { narrow_w } else { INTEL_W }) {
                at += if i == 0 { INTEL_HEAD } else { INTEL_LINE };
                out.push((part, *col, i == 0, narrow));
            }
        }
        out
    };
    let height = |parts: &[(String, Rgba, bool, bool)]| parts.iter().map(|x| if x.2 { INTEL_HEAD } else { INTEL_LINE }).sum::<f32>();
    let mut parts = wrap(p, lines);
    if height(&parts) > room {
        let kept: Vec<(String, Rgba)> = lines.iter().enumerate().filter(|(i, l)| *i == 0 || !l.0.starts_with("Level ")).map(|(_, l)| l.clone()).collect();
        parts = wrap(p, &kept);
    }
    if height(&parts) > room {
        while !parts.is_empty() && height(&parts) + INTEL_LINE > room {
            parts.pop();
        }
        let narrow = height(&parts) < beside;
        parts.push((INTEL_MORE.to_string(), th().dim, false, narrow));
    }
    parts
}
