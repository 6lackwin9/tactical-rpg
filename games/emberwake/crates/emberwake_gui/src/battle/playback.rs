//! Playing a step on the screen: impacts and their timing, floats, shakes and sounds, portrait reactions, the animations' sync, and the playback aids of a timeline battle (speed, block cursor, tracers).

use super::board::{dashed, foot_point, foot_points, place};
use super::flow::sound_of;
use super::marks::{party_foresight_off, tracer_alpha, traces_of};
use super::overlays::Layers;
use super::{BattleScreen, DASH_MS, PANEL_SURFACE, PLAYBACK_SPEED, TIMELINE_ENEMY_EVENTS, TIMELINE_SURFACE};
use super::tokens::{action_clips, body_keys, Float, PoseSet};
use aegis_ui::canvas::Rgba;
use crate::fx::{self, Impact, Shown};
use crate::portraits::Who;
use aegis_ui::fx::Clock;
use crate::stage::{Geom, View};
use crate::screens::Cx;
use aegis_ui::audio::Bus;
use emberwake_battle::command::CmdKind;
use emberwake_battle::grid::V2;
use emberwake_battle::session::Phase;
use aegis_core::json::Dict;
use serde_json::{json, Value};
use std::time::{Duration, Instant};

/// INPUT1 PB3: the playback speed setting (`playback_speed`: "1x", "0.5x", "block"; anything else reads 1x).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PlaySpeed {
    Full,
    Half,
    Block,
}

impl PlaySpeed {
    pub(super) fn of(settings: &Value) -> PlaySpeed {
        match settings[PLAYBACK_SPEED].as_str() {
            Some("0.5x") => PlaySpeed::Half,
            Some("block") => PlaySpeed::Block,
            _ => PlaySpeed::Full,
        }
    }

    /// Its index in `PLAYBACK_SPEEDS`.
    pub(super) fn index(self) -> usize {
        match self {
            PlaySpeed::Full => 0,
            PlaySpeed::Half => 1,
            PlaySpeed::Block => 2,
        }
    }
}

/// VISCLAR1-D: the round banner's fade in, hold and fade out (ms of the FX clock).
pub(super) const ROUND_BANNER_MS: (u64, u64, u64) = (140, 760, 300);

/// VISCLAR1-D: the round banner's alpha `age` ms after its round started (0: gone); `still` (skip animations, reduce
/// motion) shows it whole for as long, with no fade.
pub(super) fn round_banner_alpha(age: u64, still: bool) -> u8 {
    let (fade_in, hold, fade_out) = ROUND_BANNER_MS;
    let total = fade_in + hold + fade_out;
    if age >= total {
        0
    } else if still || (age >= fade_in && age < fade_in + hold) {
        255
    } else if age < fade_in {
        (255 * age / fade_in) as u8
    } else {
        (255 * (total - age) / fade_out) as u8
    }
}

impl BattleScreen {
    /// The board catches up with the battle after a request: the HP changes become floats and hit flashes, new log
    /// lines their sounds, the tokens follow (`sync_anims`). FX1-L1: a pending impact fires first (rule F); a single
    /// `step` that popped an action with a delivery (4.3) holds the step's board feedback until its impact (rule I,
    /// `Impact`), everything else (and everything under skip animations) shows at once. The HUD updates now.
    pub fn after_change(&mut self, cx: &Cx) {
        self.flush(cx);
        self.rev += 1;
        let skip = cx.settings.v["skip_anim"].as_bool().unwrap_or(false);
        let t = &cx.tuning.value;
        let mut floats = Vec::new();
        let mut struck = Vec::new();
        let mut sounds = Vec::new();
        let mut fell = Vec::new();
        let hps: Vec<i32> = self.session().w.units.iter().map(|u| u.hp).collect();
        if hps.len() == self.last_hp.len() {
            for (i, (&old, &new)) in self.last_hp.iter().zip(hps.iter()).enumerate() {
                if new < old {
                    floats.push((i, format!("-{}", old - new), t.damage_color));
                    struck.push(i);
                    if new <= 0 {
                        sounds.push("sfx/defeat.wav".to_string());
                        fell.push(i);
                    }
                } else if new > old {
                    floats.push((i, format!("+{}", new - old), t.heal_color));
                }
            }
        }
        let hp_before = std::mem::replace(&mut self.last_hp, hps);
        let log = &self.session().resolver.log;
        let lines: Vec<String> = log.iter().skip(self.log_seen).cloned().collect();
        // PORTRAITFX1-P (5.3): the step's portrait events, fired with its feedback (at the impact, rule I)
        let mut portrait = if hp_before.len() == self.last_hp.len() { emberwake_battle::portrait::session_step_events(self.session(), &hp_before, &lines) } else { Vec::new() };
        for line in &lines {
            if let Some(key) = sound_of(line) {
                sounds.push(format!("sfx/{key}.wav"));
            }
        }
        self.log_seen = log.len();
        // PORTRAITFX2 5.3: the step that wins the battle shows `victory` with its feedback
        let outcome_now = self.session().outcome.clone();
        let outcome_before = std::mem::replace(&mut self.portrait_outcome, outcome_now);
        portrait.extend(emberwake_battle::portrait::victory_events(self.session(), &outcome_before));
        let (n, last) = { let s = self.session(); (s.events.len(), s.events.last().cloned()) };
        if n > self.events_seen {
            self.message = last.unwrap_or_default();
            self.events_seen = n;
        }
        self.phase_shakes(cx);
        // INPUT1 PB1: the popped action's impact tracers (timeline battles): with its impact, else at once
        let traces = self.new_traces(cx);
        // PB3, PB5: a step that finished its block may hold the playback at the block's end
        self.note_block_end();
        let pending = if skip || self.req_cmd != "step" { None } else { self.popped_impact(t.walk_ms) };
        let after = self.sync_anims_deferred(cx, pending.is_some());
        match pending {
            Some(mut i) => {
                i.traces = traces;
                i.mods = fx::Mods::of(&lines, !struck.is_empty());
                i.broke = fx::broke(&lines);
                (i.after, i.floats, i.struck, i.sounds) = (after, floats, struck, sounds);
                i.marks = self.session().w.units.iter().map(|u| fx::marks_of(&u.st)).collect();
                i.portrait = portrait;
                self.impacts.push(i);
            }
            None => {
                // 4.5 row 2: a break whose step has no action to land shakes at once (not under skip animations, 4.6)
                if !skip && fx::broke(&lines) {
                    let now = self.clock.now();
                    let seed = { let s = self.session(); fx::shake_seed(s.current_round, s.current_block, fx::BREAK_KEY) };
                    self.shake(cx, fx::HEAVY, now, seed);
                }
                // decision "FX1-L2 answers": a unit that falls with no action to land (a status tick) raises its dust now
                if !skip {
                    self.fall_dust(cx, &fell);
                }
                // PB1: skip animations (or no delivery): the tracers draw as the step's result shows
                let now = self.clock.now();
                self.tracers.extend(traces.into_iter().map(|tr| (tr, now)));
                let shown: Vec<usize> = struck.iter().copied().chain(floats.iter().map(|f| f.0)).collect();
                self.react_portraits_shown(cx, &portrait, now, &shown);
                self.show(cx, floats, struck, sounds)
            }
        }
        self.react_battle_portraits(cx);
        self.push_bursts(cx);
    }

    /// PORTRAITFX2 5.3: the portrait events a change fires at once: `round_start` when the round counter rose
    /// (classic; none under the round intent rule) and `skill` at a popped skill's wind-up (its level against the rows'
    /// `min_level`). Presentation only.
    fn react_battle_portraits(&mut self, cx: &Cx) {
        let Some(s) = self.host.session.as_ref() else { return };
        let rounds = emberwake_battle::portrait::round_events(s, self.portrait_round);
        self.portrait_round = s.current_round;
        let skill = if s.last_popped != self.portrait_popped { emberwake_battle::portrait::skill_event(s) } else { None };
        self.portrait_popped = s.last_popped;
        let now = self.clock.now();
        if !rounds.is_empty() {
            let all: Vec<usize> = rounds.iter().map(|e| e.0).collect();
            self.react_portraits_shown(cx, &rounds, now, &all);
        }
        if let Some((u, level)) = skill {
            self.react_portrait_skill(cx, u, level, now);
        }
    }

    /// PORTRAITFX2 5.3: unit `u`'s `skill` event at FX clock ms `at` for a skill of level `level`: its panel portrait,
    /// and its timeline row while the view is open (a party row; an enemy's row reacts to blows only).
    fn react_portrait_skill(&mut self, cx: &Cx, u: usize, level: i32, at: u64) {
        let Some(s) = self.host.session.as_ref().filter(|s| u < s.w.units.len()) else { return };
        let (ruleset, party) = (s.w.ruleset.id.clone(), s.w.allies.contains(&u));
        let key = self.unit_portrait(cx, u).0.key;
        let rows = self.timeline_view_open() && party;
        self.panel_faces.clock = Clock { ms: at as f64, hold: 0.0 };
        self.timeline_faces.clock = self.panel_faces.clock;
        let event = [emberwake_battle::portrait::SKILL];
        self.panel_faces.react_at(cx, &key, &event, level, PANEL_SURFACE, &ruleset);
        if rows {
            self.timeline_faces.react_at(cx, &key, &event, level, TIMELINE_SURFACE, &ruleset);
        }
        self.panel_faces.clock = self.clock;
        self.timeline_faces.clock = self.clock;
    }

    /// PORTRAITFX2 5.3: `battle_start` (and `ambush`) on the battle screen's first frame, before the first step (a
    /// session that has popped an action or ended fires none).
    pub(super) fn react_start_portraits(&mut self, cx: &Cx) {
        if !std::mem::take(&mut self.portrait_start) {
            return;
        }
        let Some(s) = self.host.session.as_ref().filter(|s| s.last_popped.is_none() && s.outcome.is_empty()) else { return };
        let events = emberwake_battle::portrait::start_events(s);
        let all: Vec<usize> = events.iter().map(|e| e.0).collect();
        let now = self.clock.now();
        self.react_portraits_shown(cx, &events, now, &all);
    }

    /// FX1-L2 4.4.4 for a fall shown at once (decision "FX1-L2 answers": a unit felled by a status tick): each of
    /// `fell`'s dust at its foot point now, seeded by the round, block and unit.
    fn fall_dust(&mut self, cx: &Cx, fell: &[usize]) {
        let Some(s) = self.host.session.as_ref() else { return };
        let t = &cx.tuning.value;
        let geo = Geom::new(View::of_setting(&cx.settings.v["battle_view"]), t);
        let squash = if geo.view == View::Stage { 0.75 } else { 1.0 };
        let now = self.clock.now();
        for &u in fell {
            if let Some(c) = s.w.grid.pos(u).or(self.tokens.anims.get(u).and_then(|a| a.seen.pos)) {
                for e in fx::fall_effects(foot_point(&geo, c), squash, now, fx::seed_of(s.current_round, s.current_block, 0xFA11, u), t) {
                    self.fx.push(e);
                }
            }
        }
    }

    /// STATUSFX1 S4: the apply bursts the tokens queued go on the live list at their unit's chest in the current view
    /// (none under skip animations, S7).
    fn push_bursts(&mut self, cx: &Cx) {
        let bursts = std::mem::take(&mut self.tokens.bursts);
        if cx.settings.v["skip_anim"].as_bool().unwrap_or(false) {
            return;
        }
        let Some(s) = self.host.session.as_ref() else { return };
        let t = &cx.tuning.value;
        let geo = Geom::new(View::of_setting(&cx.settings.v["battle_view"]), t);
        for (u, id, at) in bursts {
            let Some(c) = s.w.grid.pos(u).or(self.tokens.anims.get(u).and_then(|a| a.seen.pos)) else { continue };
            if let Some(e) = fx::status_look(id).and_then(|l| fx::apply_burst(l, &geo, t, foot_point(&geo, c), at)) {
                self.fx.push(e);
            }
        }
    }

    /// FX1 4.5: starts the preset `name` of the game's table at clock `at` (seeded by `seed`; the art bible's limits;
    /// none under reduce motion; a weaker shake never replaces a stronger running one).
    fn shake(&mut self, cx: &Cx, name: &str, at: u64, seed: u64) {
        let t = &cx.tuning.value;
        if let Some(s) = t.shakes.get(name) {
            self.camera.start(s, at, seed, t.shake_limits(), cx.settings.reduce_motion());
        }
    }

    /// FX1 4.5 row 3: the phases fired since the last look whose effects hold a `shake` start the strongest of their
    /// presets at once (also under skip animations; not under reduce motion, 4.6).
    fn phase_shakes(&mut self, cx: &Cx) {
        let Some(s) = self.host.session.as_ref() else { return };
        let fired = &s.phases.fired;
        let presets: Vec<String> = fired
            .keys()
            .skip(self.phases_seen)
            .filter_map(|id| s.phases.phase(id))
            .flat_map(|p| p.effects_of(emberwake_data::SHAKE_EFFECT))
            .filter_map(|e| e.get("preset").and_then(Value::as_str).map(str::to_string))
            .collect();
        // 4.2 step 8: seeded by the round, block and the first new phase's index (never the FX clock)
        let seed = fx::shake_seed(s.current_round, s.current_block, self.phases_seen as u64);
        self.phases_seen = fired.len();
        let t = &cx.tuning.value;
        if let Some(shake) = fx::strongest(presets.iter().map(String::as_str), &t.shakes) {
            let now = self.clock.now();
            self.camera.start(shake, now, seed, t.shake_limits(), cx.settings.reduce_motion());
        }
    }

    /// The step feedback shown now: floats, hit flashes (wall clock) and sounds.
    pub(super) fn show(&mut self, cx: &Cx, floats: Vec<(usize, String, Rgba)>, struck: Vec<usize>, sounds: Vec<String>) {
        let now = Instant::now();
        // INPUT1 PB3: a float shows twice as long at 0.5x
        let ms = cx.tuning.value.float_ms * self.pace(cx);
        self.floats.extend(floats.into_iter().map(|(unit, text, color)| Float { unit, text, color, born: now, ms }));
        self.flashes.extend(struck.into_iter().map(|i| (i, now)));
        for s in sounds {
            cx.audio.play(Bus::Sfx, &s);
        }
    }

    /// FX1-L1 rule I: the impact of the action this step popped, when it has a delivery (4.3) and its actor stands:
    /// the delivery, `t0` (now, or the end of the actor's slide when it moved in this step, `walk_ms`) and the cells
    /// it goes between; when it lands is resolved with the actor's clip (`resolve_impacts`). Call before the tokens
    /// follow the step.
    fn popped_impact(&self, walk_ms: u64) -> Option<Impact> {
        let s = self.host.session.as_ref()?;
        let t = &self.tokens;
        if t.anims.len() != s.w.units.len() || s.log.len() < t.log_len_seen {
            return None;
        }
        let (_, _, idx) = s.last_popped.filter(|p| Some(*p) != t.popped_seen && p.0 == s.current_round)?;
        let act = s.resolver.scheduled.get(idx)?;
        let u = &s.w.units[act.unit];
        if act.cancelled || !u.is_alive() {
            return None;
        }
        let from = s.w.grid.pos(act.unit)?;
        let at = |i: usize| s.w.grid.pos(i).or(t.anims[i].seen.pos);
        let (mut target, mut cells) = (None, Vec::new());
        let aim = match &act.command.kind {
            CmdKind::Attack { target: tg, .. } => {
                target = *tg;
                tg.and_then(at)
            }
            CmdKind::Area { aim, cells: c, .. } => {
                cells = c.iter().copied().filter(|c| s.w.grid.is_valid(*c)).collect();
                Some(*aim)
            }
            CmdKind::Support { target, .. } => at(*target),
            CmdKind::Item { target, .. } => at(*target),
            _ => None,
        };
        let to = aim.filter(|c| *c != emberwake_battle::grid::NO_CELL).unwrap_or(from);
        let def = s.w.cat.skills.iter().find(|k| k.name == act.command.name);
        let skill = def.map_or("", |k| k.id.as_str());
        // PR4-2 (AUDIT-10): the same `Act` the FX tests build (`fx::Act::popped`)
        let fx_act = fx::Act::popped(&act.command.kind, act.unit, skill, u.inventory.main_weapon(), from.dist(to));
        let (kind, family) = (fx_act.kind, fx_act.family);
        let style = fx::style(fx_act);
        let radius = cells.iter().map(|c| c.dist(to)).max().unwrap_or(0);
        let delivery = style.delivery;
        if delivery == fx::Delivery::None {
            return None;
        }
        let (round, block, _) = s.last_popped?;
        let now = self.clock.now();
        let moved = t.anims[act.unit].seen.pos.is_some_and(|p| p != from);
        // INPUT1 MR12: after a dash's shorter slide
        let slide = if dashed(s, act.unit) { DASH_MS } else { walk_ms };
        let t0 = if moved && walk_ms > 0 { now + slide } else { now };
        let clips = if kind == "item" { Vec::new() } else { action_clips(skill, kind, family) };
        Some(Impact {
            at: None,
            actor: act.unit,
            delivery,
            t0,
            clips,
            // WINDUP1 W4: it strikes from the wind-up its actor showed (the tokens have not followed the step yet)
            strike: t.anims[act.unit].windup.is_some_and(|w| w.round == s.current_round && w.idx == idx),
            from,
            to,
            after: Vec::new(),
            floats: Vec::new(),
            struck: Vec::new(),
            sounds: Vec::new(),
            style,
            area: kind == "area",
            seed: fx::seed_of(round, block, idx, act.unit),
            blocks: def.map_or(0, |k| k.blocks),
            mods: fx::Mods::default(),
            broke: false,
            launched: false,
            cells,
            radius,
            target,
            marks: Vec::new(),
            traces: Vec::new(),
            portrait: Vec::new(),
        })
    }

    /// FX1-L1: when each pending impact lands, from its actor's clip (the first candidate its body has; its `impact`
    /// frame, decision "Attack timing on the board" A) and the board's anchors in the current view (a shot's path).
    pub(super) fn resolve_impacts(&mut self, cx: &mut Cx) {
        let Some(s) = self.host.session.as_ref() else { return };
        let t = cx.tuning.value.clone();
        let geo = Geom::new(View::of_setting(&cx.settings.v["battle_view"]), &t);
        for i in self.impacts.iter_mut().filter(|i| i.at.is_none()) {
            // WINDUP1 W4: a strike from a held wind-up starts at the clip's `strike` frame, so it lands that much sooner
            let clip_ms = s.w.units.get(i.actor).and_then(|u| cx.art.first_body(&body_keys(&s.art[i.actor], u))).and_then(|b| {
                let name = i.clips.iter().copied().find(|c| cx.art.has_clip(&b, c))?;
                let c = cx.art.clips(&b)?.clip(name)?;
                let skipped = if i.strike { c.ms_before(c.strike) } else { 0 };
                c.impact_ms().map(|ms| ms.saturating_sub(skipped))
            });
            // FX1-L2: the shot's own flight (a falling orb's from above), a volley's or a shockwave's spread
            let pl = place(&geo, i.from, i.to, i.area);
            let cells = foot_points(&geo, &i.cells);
            let at = fx::impact_at(i.delivery, i.t0, clip_ms, fx::flight_px(&i.style, pl), &t).map(|a| a + fx::spread_ms(&i.style, &cells, pl.to));
            i.at = Some(at.unwrap_or(i.t0));
        }
        // Slice 2: the delivery (the trail or the projectile) goes on the live list once its impact time is known.
        let mut out = Vec::new();
        for i in self.impacts.iter_mut().filter(|i| !i.launched) {
            if let Some(at) = i.at {
                i.launched = true;
                out.extend(fx::delivery_effects(&i.style, place(&geo, i.from, i.to, i.area), &foot_points(&geo, &i.cells), i.mods, i.t0, at, i.seed, &t));
            }
        }
        for e in out {
            self.fx.push(e);
        }
    }

    /// FX1-L1: fires the impacts due on the clock (rule I): the units react, the floats, flashes and sounds show, the
    /// hit-stop holds the clock when any HP fell, and autoplay waits `impact_gap_ms` from now (rule P).
    pub(super) fn fire_due(&mut self, cx: &mut Cx) {
        self.resolve_impacts(cx);
        let now = self.clock.now();
        while self.impacts.first().is_some_and(|i| i.at.is_some_and(|at| at <= now)) {
            let i = self.impacts.remove(0);
            let at = i.at.unwrap_or(now);
            self.fire(cx, i, at);
        }
    }

    /// FX1-L1 rule F: every pending impact fires now, in order (before any later catch-up).
    pub(super) fn flush(&mut self, cx: &Cx) {
        let now = self.clock.now();
        for i in std::mem::take(&mut self.impacts) {
            let at = i.at.map_or(now, |a| a.min(now));
            self.fire(cx, i, at);
        }
    }

    pub(super) fn fire(&mut self, cx: &Cx, i: Impact, at: u64) {
        let t = &cx.tuning.value;
        let stop = fx::hit_stop(&i, t);
        if stop > 0.0 {
            self.clock.hold(stop);
        }
        self.tokens.react(&i.after, &i.marks, at, false);
        self.push_bursts(cx);
        // PORTRAITFX1-P: the panel portraits flinch as the action lands
        let shown: Vec<usize> = i.struck.iter().copied().chain(i.floats.iter().map(|f| f.0)).collect();
        self.react_portraits_shown(cx, &i.portrait, at, &shown);
        // INPUT1 PB1: the action's tracers from its impact; PB3: the gap after it doubles at 0.5x
        self.tracers.extend(i.traces.iter().map(|tr| (*tr, at)));
        self.gap_until = Some(Instant::now() + Duration::from_millis(t.impact_gap_ms * self.pace(cx)));
        // Slice 2: the impact effects (4.4) on the target, or on each unit an area struck, and the heavy shake of a
        // Heavy hit or a break (4.5 rows 1-2); neither once skip animations is on (4.6). FX1-L2: an area's aim ring
        // (and a BURST's flash), the effects on the actor for `on_actor` rows, the dust of each unit that falls, all
        // within the impact budget (sparks first); a miss's target steps aside (not under reduce motion).
        let skip = cx.settings.v["skip_anim"].as_bool().unwrap_or(false);
        if let Some(s) = self.host.session.as_ref().filter(|_| !skip) {
            let geo = Geom::new(View::of_setting(&cx.settings.v["battle_view"]), t);
            let cell = |u: usize| s.w.grid.pos(u).or(self.tokens.anims.get(u).and_then(|a| a.seen.pos));
            let spot = |c: V2| fx::Spot { chest: fx::anchor(&geo, c), foot: foot_point(&geo, c) };
            let struck: Vec<V2> = i.struck.iter().filter_map(|&u| cell(u)).collect();
            let cells: Vec<V2> = if i.style.on_actor {
                vec![i.from]
            } else if i.area {
                struck.clone()
            } else {
                vec![i.to]
            };
            let pl = place(&geo, i.from, i.to, i.area);
            let mut out = Vec::new();
            if i.area || i.style.aim.is_some() {
                out.extend(fx::aim_effects(&i.style, i.area, foot_point(&geo, i.to), i.radius, struck.contains(&i.to), pl.squash, at, i.seed));
            }
            let hits: Vec<fx::Spot> = cells.iter().map(|&c| spot(c)).collect();
            out.extend(fx::impact_effects(&i.style, i.mods, pl, &hits, at, i.seed, t));
            for (k, &u) in i.struck.iter().enumerate() {
                if i.after.get(u).is_some_and(|a| !a.alive) {
                    if let Some(c) = cell(u) {
                        out.extend(fx::fall_effects(foot_point(&geo, c), pl.squash, at, i.seed ^ k as u64, t));
                    }
                }
            }
            for e in fx::budget(out, fx::IMPACT_BUDGET) {
                self.fx.push(e);
            }
            let standing = |u: usize| i.after.get(u).is_some_and(|a| a.alive) && s.w.grid.pos(u).is_some();
            if let Some(u) = i.target.filter(|&u| i.mods.miss && !i.area && u != i.actor && standing(u) && !cx.settings.reduce_motion()) {
                let away = fx::unit_dir(fx::anchor(&geo, i.from), fx::anchor(&geo, s.w.grid.pos(u).unwrap_or(i.to)));
                self.sidesteps.push((u, at, away));
            }
        }
        if !skip && fx::heavy_shake(i.blocks, i.hit(), i.broke) {
            self.shake(cx, fx::HEAVY, at, i.seed);
        }
        self.show(cx, i.floats, i.struck, i.sounds);
    }

    /// PORTRAITFX1-P (5.3, 6.2 row 3): the panel portraits react to a step's events at FX clock ms `at` (its impact, or
    /// now when the step has none to wait for): each unit's portrait fires the kit's `panel` event rows (none under skip
    /// animations, `Faces::react`). Presentation only: nothing here reads or moves the battle. Every blow counts as shown
    /// on the board (the tests' entry; the screen passes the step's own, `react_portraits_shown`).
    #[cfg(test)]
    pub(super) fn react_portraits(&mut self, cx: &Cx, events: &[(usize, &'static str)], at: u64) {
        let shown: Vec<usize> = events.iter().map(|e| e.0).collect();
        self.react_portraits_shown(cx, events, at, &shown);
    }

    /// `react_portraits` with `shown`, the units the board shows the step's blow on (its floats and hit flashes).
    /// PORTRAITFX1-TL (user 2026-11-01): the timeline view's rows react too, only while the view is open; an enemy's
    /// row only to a blow (`hit`, `crit`, `guard_hit`) the board shows on it (party rows to every event, as the panels).
    pub(super) fn react_portraits_shown(&mut self, cx: &Cx, events: &[(usize, &'static str)], at: u64, shown: &[usize]) {
        let Some(s) = self.host.session.as_ref().filter(|_| !events.is_empty()) else { return };
        let ruleset = s.w.ruleset.id.clone();
        let row_reacts = |u: usize, e: &str| s.w.allies.contains(&u) || (TIMELINE_ENEMY_EVENTS.contains(&e) && shown.contains(&u));
        let fired: Vec<(String, &str, bool)> = events.iter().filter(|(u, _)| *u < s.w.units.len()).map(|&(u, e)| (self.unit_portrait(cx, u).0.key, e, row_reacts(u, e))).collect();
        let rows = self.timeline_view_open();
        self.panel_faces.clock = Clock { ms: at as f64, hold: 0.0 };
        self.timeline_faces.clock = self.panel_faces.clock;
        for (key, event, row) in fired {
            self.panel_faces.react(cx, &key, &[event], PANEL_SURFACE, &ruleset);
            if rows && row {
                self.timeline_faces.react(cx, &key, &[event], TIMELINE_SURFACE, &ruleset);
            }
        }
        self.panel_faces.clock = self.clock;
        self.timeline_faces.clock = self.clock;
    }

    /// PORTRAITFX1-P: battle unit `c`'s panel portrait and its look. A world fight's party unit is its member
    /// (`hub::member_portrait`'s key, its ROSTER1 conditions); a unit whose hero key names a character of the battle's
    /// world draws as it; any other unit as itself (`Who::of_unit`). The look's HP and life are the board's (rule I:
    /// the state before a step until its impact).
    pub(super) fn unit_portrait(&self, cx: &Cx, c: usize) -> (Who, emberwake_battle::portrait::Look) {
        let s = self.session();
        let d = &cx.data;
        let key = s.hero_key(c);
        let member = if self.world_mode() && !key.is_empty() { cx.gs().party.members.iter().find(|m| m.id == key) } else { None };
        let (who, conditions) = match member {
            Some(m) => (Who { key: format!("member:{}", m.id), ..Who::of_member(d, m) }, m.conditions.clone()),
            None => {
                let world = if self.world_mode() { self.class_world.clone() } else { self.arena_world.clone().unwrap_or_default() };
                let empty = Dict::new();
                let character = if key.is_empty() { &empty } else { d.resolve(&world, "characters", &key) };
                (Who::of_unit(d, s, c, character), Vec::new())
            }
        };
        let mut look = crate::portraits::look_of_unit(s, c, &conditions);
        let u = &s.w.units[c];
        let (hp, alive) = self.tokens.shown(c, u, s.w.units.len());
        look.hp = hp;
        look.down = !alive && !u.is_withdrawn;
        (who, look)
    }

    /// Rule P: autoplay may send the next step (no impact pending and its gap over).
    pub(super) fn impacts_done(&self, now: Instant) -> bool {
        self.impacts.is_empty() && self.gap_until.is_none_or(|g| now >= g)
    }

    /// FX1 4.8, the agent link's `state.fx`: the effects not done on the FX clock (`live`, their `kinds`), the impacts
    /// waiting to land (`pending`) and the camera's offset (`shake`), so a test or an agent can wait for them to end.
    pub(super) fn fx_state(&self) -> Value {
        let now = self.clock.now();
        let kinds = self.fx.kinds(now);
        let (dx, dy) = self.camera.offset_at(now);
        json!({"live": kinds.len(), "pending": self.impacts.len(), "shake": [dx, dy], "kinds": kinds})
    }

    /// SPRITE1-U: the tokens catch up with the battle (`Tokens::catch_up`; the skip animations setting applies). While
    /// an impact is pending they keep the state before its step (rule I).
    pub(super) fn sync_anims(&mut self, cx: &Cx) {
        let defer = !self.impacts.is_empty();
        self.sync_anims_deferred(cx, defer);
        self.push_bursts(cx);
    }

    fn sync_anims_deferred(&mut self, cx: &Cx, defer: bool) -> Vec<Shown> {
        let instant = cx.settings.v["skip_anim"].as_bool().unwrap_or(false);
        self.tokens.still = cx.settings.reduce_motion();
        self.tokens.foresight_off = party_foresight_off(&cx.settings.v, cfg!(debug_assertions));
        match self.host.session.as_ref() {
            Some(s) => {
                let tells = cx.settings.v["show_tells"].as_bool().unwrap_or(true);
                self.tokens.follow(s, self.clock.now(), instant, cx.tuning.value.walk_ms, tells, defer)
            }
            None => Vec::new(),
        }
    }

    /// WINDUP1: looks up the pose clips of each unit's body (`PoseSet`) once a battle's units are known (the catch-ups
    /// run where the art cannot be read); true when they were new, so the tokens catch up again to show the poses.
    pub(super) fn learn_poses(&mut self, cx: &mut Cx) -> bool {
        let Some(s) = self.host.session.as_ref() else { return false };
        if self.tokens.poses.len() == s.w.units.len() {
            return false;
        }
        self.tokens.poses = s.w.units.iter().enumerate().map(|(i, u)| cx.art.first_body(&body_keys(&s.art[i], u)).and_then(|b| cx.art.clips(&b).map(PoseSet::of)).unwrap_or_default()).collect();
        true
    }

    /// VISCLAR1-D: the banner is still on its time (FX clock).
    pub(super) fn round_banner_alive(&self) -> bool {
        let (fade_in, hold, fade_out) = ROUND_BANNER_MS;
        self.banner.is_some_and(|b| self.clock.now().saturating_sub(b.1) < fade_in + hold + fade_out)
    }

    /// VISCLAR1-D: a party card's pips have not yet landed on its unit's AP (FX clock).
    pub(super) fn pips_moving(&self) -> bool {
        let Some(s) = self.host.session.as_ref() else { return false };
        self.card_ap.iter().any(|(a, p)| s.w.units.get(*a).is_some_and(|u| p.level(self.clock.now()) != u.ap as f32))
    }

    /// VISCLAR1-D: a new round's banner starts, once no talk holds the round's start (timeline battles).
    pub(super) fn follow_round_banner(&mut self) {
        let Some(round) = self.host.session.as_ref().filter(|s| emberwake_battle::round_intent::on(&s.w)).map(|s| s.current_round) else { return };
        if self.banner.is_none_or(|b| b.0 != round) && !self.talk_due() {
            self.banner = Some((round, self.clock.now()));
        }
    }

    /// VISCLAR1-D: the round the banner names and its alpha now (None: no banner showing).
    pub(super) fn round_banner_now(&self, cx: &Cx) -> Option<(i32, u8)> {
        let (round, at) = self.banner?;
        let still = cx.settings.v["skip_anim"].as_bool().unwrap_or(false) || cx.settings.reduce_motion();
        Some((round, round_banner_alpha(self.clock.now().saturating_sub(at), still))).filter(|b| b.1 > 0)
    }

    /// INPUT1 PB1-PB5: the playback aids are on: a timeline battle (the round intent or any ruleset rule, as TV1).
    pub(super) fn playback_aids(&self) -> bool {
        self.timeline_view_on()
    }

    /// PB3: a timeline battle's round is playing: from Confirm until the round is over and its last impact has played.
    fn in_playback(&self) -> bool {
        let Some(s) = self.host.session.as_ref() else { return false };
        self.playback_aids() && !(s.phase == Phase::Planning && !s.plan_confirmed) && (s.phase != Phase::Replay || !self.impacts_done(Instant::now()))
    }

    /// PB3: the playback's pacing factor: 2 at 0.5x while a round plays (every walk, clip, float, tracer and pause
    /// between steps takes twice as long, on top of the animation speed), else 1; skip animations plays 0.5x as 1x.
    pub(super) fn pace(&self, cx: &Cx) -> u64 {
        let half = PlaySpeed::of(&cx.settings.v) == PlaySpeed::Half && !cx.settings.v["skip_anim"].as_bool().unwrap_or(false);
        u64::from(half && self.in_playback()) + 1
    }

    /// PB2: a step request is about to go: the block it resolves (block 1 from planning) is the cursor's from now on.
    pub(super) fn note_step(&mut self) {
        let Some(s) = self.host.session.as_ref().filter(|_| self.req_cmd == "step") else { return };
        let block = match s.phase {
            Phase::Planning => 1,
            Phase::Resolving => s.current_block,
            Phase::Replay => return,
        };
        self.play_block = Some((s.current_round, block));
    }

    /// PB3, PB5: after a request: the block a step finished (the round's last one too, which holds autoplay before
    /// the next round); none from the party's planning on.
    fn note_block_end(&mut self) {
        let Some(s) = self.host.session.as_ref() else { return };
        if s.phase == Phase::Planning && !s.plan_confirmed {
            (self.block_ended, self.waiting) = (None, None);
            return;
        }
        if self.req_cmd != "step" {
            return;
        }
        let Some((_, b)) = self.play_block.filter(|p| p.0 == s.current_round) else { return };
        let ended = match s.phase {
            Phase::Resolving => s.current_block > b,
            Phase::Replay => !s.combat_over,
            Phase::Planning => false,
        };
        self.block_ended = ended.then_some(b);
    }

    /// PB3, PB5: holds the playback after the block the last step finished while Block mode is on or the timeline view
    /// is open (the message bar says how to go on); true while it holds.
    pub(super) fn hold_at_block_end(&mut self, cx: &Cx) -> bool {
        let block_mode = PlaySpeed::of(&cx.settings.v) == PlaySpeed::Block;
        let Some(b) = self.block_ended.filter(|_| self.playback_aids() && (block_mode || self.timeline_view_open())) else {
            self.waiting = None;
            return false;
        };
        if self.waiting != Some((b, block_mode)) {
            self.waiting = Some((b, block_mode));
            let n = self.session().def.total_blocks;
            self.message = if block_mode {
                format!("Block {b} of {n}: N for the next")
            } else {
                format!("Block {b} of {n}: paused while the timeline view is open (T closes it, N plays the next block)")
            };
        }
        true
    }

    /// PB3, PB5: the held playback goes on with the next block (and holds again at its end while it should).
    pub(super) fn release_block(&mut self) {
        (self.block_ended, self.waiting) = (None, None);
        self.next_at = Instant::now();
        if self.host.session.as_ref().is_some_and(|s| s.phase == Phase::Resolving) {
            self.message = "Resolving the round...".into();
        }
    }

    /// PB2: the block the strip frames and the timeline view's playhead marks: the one the playback resolves, from its
    /// first step until the next block's; none while the party plans, from Confirm to the first step (the round's
    /// start) and once the round is over and its last impact has played (its end).
    pub(super) fn cursor_block(&self, now: Instant) -> Option<i32> {
        let s = self.host.session.as_ref()?;
        if !self.playback_aids() || s.phase == Phase::Planning {
            return None;
        }
        let (_, b) = self.play_block.filter(|p| p.0 == s.current_round)?;
        (s.phase != Phase::Replay || !self.impacts_done(now) || self.waiting.is_some()).then_some(b)
    }

    /// PB1: the tracers of the action the last step popped (once each; none outside timeline battles or for any other
    /// request).
    fn new_traces(&mut self, cx: &Cx) -> Vec<fx::Trace> {
        let Some(s) = self.host.session.as_ref() else { return Vec::new() };
        let popped = s.last_popped;
        let fresh = popped.filter(|p| self.req_cmd == "step" && Some(*p) != self.traced && p.0 == s.current_round);
        self.traced = popped;
        let Some((_, _, idx)) = fresh.filter(|_| self.playback_aids()) else { return Vec::new() };
        let tokens = &self.tokens;
        let at = |u: usize| s.w.grid.pos(u).or(tokens.anims.get(u).and_then(|a| a.seen.pos));
        traces_of(s, idx, &at, cx.tuning.value.heal_color)
    }

    /// PB1: the live tracers as the agent link reads them (`overlay.tracers`: from, to, alpha now, dashed); none with
    /// the group `tracers` off.
    pub(super) fn tracers_json(&self, cx: &Cx) -> Value {
        let on = Layers::of(&cx.settings.v["battle_overlays"]).tracers;
        let (now, still) = (self.clock.now(), cx.settings.reduce_motion());
        let cell = |c: V2| json!([c.x, c.y]);
        Value::Array(self.tracers.iter().filter(|_| on).filter_map(|(t, at)| {
            let a = tracer_alpha(now.saturating_sub(*at), still);
            (a > 0).then(|| json!({"from": cell(t.from), "to": cell(t.to), "alpha": a, "dashed": t.dashed}))
        }).collect())
    }
}
