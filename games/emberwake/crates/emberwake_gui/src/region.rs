//! The region map (scripts/ui/overworld/region_map_view.gd + dialogue_panel.gd + shop_panel.gd): walking a
//! location's map over the game state (ExploreRunner) under the HUD bar, the floating "Here" legend and the event log
//! ticker (PLAYUI1-M: the full-width map, a camera centred on the party with walls drawn past the edge), the dialogue
//! box at the bottom centre (DialogueRunner; PLAYUI1-D: the camera keeps both speakers above it), the shop's trade over
//! a dimmed map (`shop.rs`, PLAYUI1-P), the turn-in picker and lore pages. Fights start the battle screen; a world-map exit opens
//! the world map. Map-less locations show their exits as buttons. PLAYUI1-R (spec PLAYUI1 2.4): the party leader and the
//! people stand on their tiles as body sprites (`walk` while moving, else `idle`; side view, mirrored by the last
//! left or right step; a person in the conversation faces the party), with today's tokens as the fallback.
//! STORYUI1-U1: with nothing open the map hands over to the story stage while a sequence runs and plays queued
//! vignettes (`story::map_tick`); a `sequence` choice ends the talk and the stage takes over. STORYUI1-U2: an open
//! board shows its pane (`board.rs`) in the shop's place until Close, Esc or a right click closes it back to the
//! conversation; the keeper of a board with work wears the quest badge ("New work on the board"). CAMPAIGN1-U: a queued `{dialogue}` scene
//! opens as the conversation when nothing is open (T2); a reply whose `rest` would end the run asks first (W3); the
//! log lines name the purse (P1) and a retreat (R4). MAPNAV1-U: edge arrows toward the legend's markers out of view, a
//! legend row or an arrow pans the camera there (a second row click walks), a tile click walks or says it can't, and
//! Q or a click on the leader passes the lead (`GameState::explore_leader`, saved as `explore.leader`). EXPLOREVIEW1:
//! the debug setting `region_view` draws the map square (as above) or tilted (`mapview.rs`: squashed ground, water lips,
//! walls as blocks in one feet-sorted pass, click areas in draw order); F7 flips it in a debug build.
//! REGIONGROUND1-L: a map with a `ground` layer draws its composed picture (`ground.rs`) in place of the `.` and `~` tiles
//! (and the tilted water lips); everything else draws over it as before, and `rows` alone still decide walking.
//! REGIONVIEW1: the party's drawn position glides between tiles and the camera follows it in whole physical px, stopped
//! by the map's edges with a small margin; pans and the talk camera ease (`mapview.rs`). The map's outer wall ring and what
//! lies past it are the border: flat, under everything, in the location's boundary art when it has one.
//! SERVICES1 (SILOS1 T2): the conversation, the shop, the board, the turn-in picker and the rest ask are `talk::Services`,
//! held here and drawn over the map; the map only stops its walk when one of them starts a fight.

use aegis_core::assets::{ClipPlayer, Facing, View};
use aegis_ui::theme::th;
use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use crate::hud::{hud_bar, hud_key, hud_open, purse_word, Hud, PurseWord, HUD_H};
use crate::runover::days as day_count;
use crate::music;
use crate::mapview::{self, MapGeom};
#[cfg(test)]
use crate::mapview::{CELL_W, FACE_H, SPRITE};
use crate::props::{Block, Item, PropDraw, ART_TILE, BOX_EDGE, BOX_FACE, BOX_FACE_H, BOX_FILL};
use crate::ground::MapGround;
#[cfg(test)]
use crate::portraits::Faces;
use crate::talk::{talk_layout, Services, SvcAct};
use crate::screens::{show_errors, Cx, GameCx, Key, Nav, Screen};
use aegis_ui::painter::View as PView;
use aegis_ui::ui::{draw_glyph, tip, Gfx, Glyph, Ui, Widget, BODY, H, SMALL, W};
use aegis_ui::widget_plumbing;
use aegis_core::json::{self, s, s_or, Dict};
use emberwake_world::explore::{self, Pos};
use emberwake_world::quest;
use serde_json::{json, Value};
use std::collections::VecDeque;
#[cfg(test)]
use std::time::Duration;
use std::time::Instant;

/// The leader keeps walking this long (animation ms) past a step's time, so key steps and walked paths play `walk` without
/// a stop between tiles at any step time (`mapview::WalkStep`).
const WALK_LINGER_MS: f64 = 40.0;

#[derive(Debug, Clone)]
pub enum RAct {
    MapClick,
    Marker(String),
    Exit(String),
    Menu,
    /// A HUD bar button (PLAYUI1-H).
    Hud(Hud),
    /// SERVICES1: a click on the conversation, its shop, its board or the turn-in picker (`talk::Services`).
    Svc(SvcAct),
    CloseLore,
    /// Fold or open the legend (`map_legend_open`).
    Legend,
    /// Show the last events in the ticker, or only the latest (L).
    Log,
    /// MAPNAV1-U: a legend row or an edge arrow: the camera looks at the marker (a second click on the row while it
    /// looks there walks there, as a marker click).
    Pan(String),
    /// MAPNAV1-U: the next standing active member leads (Q, or a click on the leader).
    Lead,
}

impl From<SvcAct> for RAct {
    fn from(a: SvcAct) -> Self {
        RAct::Svc(a)
    }
}

pub struct RegionMap {
    w: Vec<Widget<RAct>>,
    mouse: (i32, i32),
    facing: Pos,
    /// SERVICES1 (SILOS1 T2): the conversation, its shop, its board and the turn-in picker.
    pub(crate) svc: Services,
    pub(crate) lore: Option<(String, String)>,
    walk: Vec<Pos>,
    walk_target: Option<String>,
    next_walk: Instant,
    /// WALK2, VIEWKEYS1: the time between steps, read from the settings each frame.
    step: mapview::WalkStep,
    /// WALK2: a movement key pressed on this map is held: the draw steps every `step` until it is let go.
    hold: bool,
    /// Map geometry of the last frame.
    geo: MapGeom,
    /// The ticker shows the last `LOG_OPEN` events (else only the latest).
    log_open: bool,
    /// PLAYUI1-R: the sprites' animation clock (ms, at the animation speed setting) and when it last advanced.
    anim_ms: f64,
    anim_at: Instant,
    /// The leader's clips (`walk` / `idle`), the clock time of the last step that moved the party, and the facing
    /// (side view, left after a step to the left until a step to the right; DIR4-M adds up and down).
    leader: ClipPlayer,
    moved_at: Option<(f64, f64)>,
    leader_face: Facing,
    /// The sprites drawn last frame (agent link `state.sprites`, tests).
    sprites: Vec<Value>,
    /// MAPNAV1-U: the tile the camera looks at instead of the party (a legend row or an edge arrow); cleared by the
    /// party's first step (a key step or a walked path's), an arrow key, Esc, an exit or a change of location.
    pan: Option<Pos>,
    /// The location `pan` was set in (a pan never carries into another location).
    pan_loc: String,
    /// MAPNAV1-U: the edge arrows drawn last frame (agent link `state.arrows`, tests).
    arrows: Vec<Value>,
    /// REGIONGROUND1-L: the map's composed ground, kept while its job stands.
    ground: MapGround,
    /// REGIONVIEW1: this frame's wall time (set by the tick; None before the first: no time has passed, all at rest).
    frame_at: Option<Instant>,
    /// The scheduled time of a step taken since the last frame.
    step_at: Option<Instant>,
    /// That step's times (TURNFEEL1).
    leg: mapview::Leg,
    /// When the last step was taken (`state.walk_feel`, under `camera_trail` as it is a clock reading: what a render
    /// measures a turn from).
    last_step: Option<Instant>,
    /// The party's drawn position and the camera over frames.
    glide: mapview::Glide,
    follow: mapview::Follow,
    /// The last frame's camera (physical px of cell (0, 0)), the party's physical px and scale (agent link `state.camera`);
    /// a trail of them only under the unlisted setting `camera_trail` (what a render measures).
    cam: CamShot,
    cam_trail: VecDeque<(f64, CamShot)>,
    opened_at: Instant,
}

/// One frame's camera as the agent link reports it.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct CamShot {
    px: (i32, i32),
    party: (i32, i32),
    scale: f32,
}

/// The frames `state.camera.trail` keeps.
const CAM_TRAIL: usize = 240;

/// Starts a world fight: the battle screen, or the Skip / Fled / Lost popup in a debug game.
pub fn begin_fight(cx: &mut Cx, fight: &Dict, runner: &str) {
    if let Err(e) = try_fight(cx, fight, runner) {
        show_errors(cx, vec![e]);
    }
}

/// `begin_fight` that hands the reason back when the battle cannot start (the story stage shows it itself).
pub fn try_fight(cx: &mut Cx, fight: &Dict, runner: &str) -> Result<(), String> {
    if cx.debug {
        cx.nav.push(Nav::Push(Box::new(crate::runner::DebugFight::new(fight, runner))));
        return Ok(());
    }
    let b = crate::battle::BattleScreen::world(cx, fight, runner)?;
    cx.nav.push(Nav::Push(Box::new(b)));
    Ok(())
}

/// Logs "Fight: <name>!" and starts world fight `fight` (`begin_fight`).
pub fn fight_now(cx: &mut Cx, fight: &Dict, runner: &str) {
    let name = s_or(cx.data.resolve(&cx.gs().world_id, "encounters", &s(fight, "encounter")), "name", "Enemies");
    cx.log(format!("Fight: {name}!"));
    begin_fight(cx, fight, runner);
}

/// Event lines for the log; `autosave` writes the autosave.
pub fn log_events(cx: &mut Cx, events: &[Value]) {
    for e in events {
        let e = json::dict(e);
        match s(e, "kind").as_str() {
            "entered" => {
                let loc = s(e, "location");
                let name = cx.location_name(&cx.gs().world_id.clone(), &loc);
                cx.log(format!("Entered {name}."));
                for q in json::sget(e, "advanced") {
                    let qn = s_or(&quest::resolved(&cx.data, cx.gs(), &q), "name", &q);
                    cx.log(format!("Quest updated: {qn}."));
                }
            }
            "rested" => cx.log(format!("The party rests (+{} HP).", json::i(e, "restored", 0))),
            "locked" => cx.log(format!("Locked: {}", s(e, "text"))),
            // CAMPAIGN1 P1: the purse's names (the Ashlands lines are today's); R2: a rest's days
            "camped" => {
                let days = json::i(e, "days", 0);
                let took = if days > 0 { format!(", {}", day_count(days)) } else { String::new() };
                let upkeep = purse_word(cx, PurseWord::Upkeep);
                cx.log(format!("The party makes camp (-{} {upkeep}, +{} HP{took}).", json::i(e, "spent", 0), json::i(e, "restored", 0)));
            }
            "defeat" => {
                let name = cx.location_name(&cx.gs().world_id.clone(), &s(e, "location"));
                let upkeep = purse_word(cx, PurseWord::Upkeep);
                // R4: a retreat falls back to the hub, its days and upkeep spent
                if s(e, "mode") == "retreat" {
                    let days = json::i(e, "days", 0);
                    let cost = if days > 0 { format!(" ({}, -{} {upkeep})", day_count(days), json::i(e, "spent", 0)) } else { String::new() };
                    cx.log(format!("Defeated. The party falls back to {name}{cost}."));
                    continue;
                }
                let how = match s(e, "mode").as_str() {
                    "rest_place" => "rests there for free".to_string(),
                    "forced_rest" => format!("rests ({} {upkeep})", json::i(e, "spent", 0)),
                    _ => "limps back to town".to_string(),
                };
                cx.log(format!("Defeated. The party wakes at {name} and {how}."));
            }
            "retreat_failed" => cx.log(s(e, "text")),
            "exit" => log_events(cx, json::aget(e, "events")),
            "autosave" => cx.autosave(),
            _ => {}
        }
    }
}

impl RegionMap {
    pub fn new(cx: &mut Cx) -> Self {
        let d = cx.data.clone();
        let res = explore::resume(&d, cx.gs_mut());
        if !json::b(&res, "ok", false) {
            cx.notice = json::sget(&res, "errors");
        }
        log_events(cx, json::aget(&res, "events"));
        crate::runover::take(cx, &res);
        Self {
            w: Vec::new(),
            mouse: (0, 0),
            facing: (0, 1),
            svc: Services::default(),
            lore: None,
            walk: Vec::new(),
            walk_target: None,
            next_walk: Instant::now(),
            step: mapview::WalkStep::of(&cx.settings.v, cx.held.shift),
            hold: false,
            geo: MapGeom::default(),
            log_open: false,
            anim_ms: 0.0,
            anim_at: Instant::now(),
            leader: ClipPlayer::default(),
            moved_at: None,
            leader_face: Facing::default(),
            sprites: Vec::new(),
            pan: None,
            pan_loc: String::new(),
            arrows: Vec::new(),
            ground: MapGround::default(),
            frame_at: None,
            step_at: None,
            leg: mapview::Leg::plain(0.0),
            last_step: None,
            glide: mapview::Glide::default(),
            follow: mapview::Follow::default(),
            cam: CamShot::default(),
            cam_trail: VecDeque::new(),
            opened_at: Instant::now(),
        }
    }

    /// The scheduled time of a step due at `now`: `next_walk` while on schedule, so the steps average `step` whatever
    /// the frame rate and the glide stays even; `now` after a stop or a long frame.
    fn step_time(&self, now: Instant) -> Instant {
        if now.saturating_duration_since(self.next_walk) < self.step.gap() {
            self.next_walk.min(now)
        } else {
            now
        }
    }

    /// A step in `dir` is taken at `at`: its times, and when the next one comes (TURNFEEL1: it waits out this one's beat
    /// and glide; a key press does not wait, so a turn answers at once and only the drawn glide starts later). Before the
    /// step: it turns the party, and the beat asks which way it faced.
    fn plan_step(&mut self, cx: &Cx, at: Instant, dir: Pos) {
        let view = mapview::View::of_setting(&cx.settings.v["region_view"]);
        self.leg = mapview::WalkFeel::of(&cx.settings.v).leg(self.step, view, self.facing, dir);
        self.step_at = Some(at);
        self.last_step = Some(at);
        self.next_walk = at + self.leg.gap();
    }

    fn modal(&self) -> bool {
        self.svc.talking() || self.lore.is_some()
    }

    /// One step; handles fights, world-map exits and the log. False when the walk should stop.
    fn step(&mut self, cx: &mut Cx, dir: Pos) -> bool {
        self.turn(dir);
        let d = cx.data.clone();
        let from = cx.gs().position;
        let res = explore::step(&d, cx.gs_mut(), dir);
        if cx.gs().position != from {
            // TURNFEEL1: a clean corner plays the rest of the leg first; the next step and the walk clip wait for it
            let to = cx.gs().position;
            let corner = self.step_at.and_then(|at| self.glide.corner_step(&cx.gs().location_id, from, to, at, self.leg));
            if let Some(wait) = corner {
                self.next_walk += std::time::Duration::from_secs_f64(wait / 1000.0);
                self.leg.anim += wait * self.step.base / self.step.wall;
            }
            self.moved_at = Some((self.anim_ms, self.leg.anim));
            self.pan = None;
        }
        if !json::b(&res, "ok", false) {
            for e in json::sget(&res, "errors") {
                cx.log(e);
            }
            return false;
        }
        let events = json::aget(&res, "events").to_vec();
        log_events(cx, &events);
        crate::runover::take(cx, &res);
        let mut go_on = true;
        for e in &events {
            let e = json::dict(e);
            match s(e, "kind").as_str() {
                // CAMPAIGN1 W3: a marker fight that could end the run asks first (No: the party steps back); a fight
                // that cannot be refused gets the notice
                "fight" => {
                    go_on = false;
                    let enc = s(e, "encounter");
                    match (s(e, "source") == "marker", crate::runover::fight_ask(cx, &enc)) {
                        (true, Some(q)) => {
                            self.walk.clear();
                            explore::decline_fight(&d, cx.gs_mut(), e);
                            let f = e.clone();
                            cx.ask_no(q, move |cx| fight_now(cx, &f, "explore"));
                        }
                        (false, Some(_)) => {
                            self.start_fight(cx, e, "explore");
                            crate::runover::warn_fight(cx, &enc);
                        }
                        _ => self.start_fight(cx, e, "explore"),
                    }
                }
                "exit" if s(e, "to") == explore::WORLD_MAP => {
                    cx.nav.push(Nav::Push(Box::new(crate::worldmap::WorldMap::new())));
                    go_on = false;
                }
                "exit" | "blocked" | "locked" => go_on = false,
                _ => {}
            }
        }
        go_on
    }

    pub fn start_fight(&mut self, cx: &mut Cx, fight: &Dict, runner: &str) {
        self.walk.clear();
        fight_now(cx, fight, runner);
    }

    /// Faces `dir` (a step or a use): the sprite turns four ways (DIR4-M, `Facing::toward`): up or down on an up or
    /// down one (keeping its last left/right for side rows), left or right on a horizontal one.
    fn turn(&mut self, dir: Pos) {
        self.facing = dir;
        self.leader_face = self.leader_face.toward(dir.0 as i32, dir.1 as i32);
    }

    /// A frame's tick at wall time `now` (the frame's clock: the glide and the camera read it in `draw`).
    fn tick_at(&mut self, cx: &mut Cx, now: Instant) {
        self.frame_at = Some(now);
        // PLAYUI1-R: the sprites play on their own clock at the animation speed setting (as the battle tokens)
        let speed = mapview::anim_speed(&cx.settings.v);
        // WALKRUN1: Shift held runs; the pace of a step is set as it is taken, so a release counts from the next step
        self.step = mapview::WalkStep::of(&cx.settings.v, cx.held.shift);
        self.anim_ms += now.duration_since(self.anim_at).as_secs_f64() * 1000.0 * speed;
        self.anim_at = now;
        // PORTRAITFX1 4.5: the talk box's portrait clock moves; CAMPAIGN1 W3: a Yes to the ask before a resting reply
        // picks it (`Services::tick`)
        if let Some(fought) = self.svc.tick(cx) {
            if fought {
                self.walk.clear();
            }
            return;
        }
        // STORYUI1 W2, 5.3: with nothing open, a running sequence takes over and queued vignettes play; CAMPAIGN1 W1,
        // F3: campaign notices first, then the run-over screen once the run is over; T2: the campaign queue's head
        // plays (vignettes and battles through `story::map_tick`, a dialogue scene here)
        if !self.modal() && (crate::runover::map_tick(cx) || crate::story::map_tick(cx) || self.svc.open_pending(cx)) {
            self.walk.clear();
            self.walk_target = None;
            return;
        }
        // WALK2: a held movement key keeps stepping at the step rate (the press itself stepped at once; the OS key
        // repeat is dropped by the shell, `hold_walk`), so the steps chain with the walk clip running.
        if cx.held.dir.is_none() {
            self.hold = false;
        }
        if self.hold && self.walk.is_empty() && !self.modal() {
            if let (Some(dir), true) = (cx.held.dir, now >= self.next_walk) {
                let at = self.step_time(now);
                self.plan_step(cx, at, dir);
                let t = explore::template_of(&cx.data, explore::location(&cx.data, cx.gs()));
                if t.is_empty() || !self.step(cx, dir) {
                    self.hold = false;
                }
            }
            return;
        }
        if self.walk.is_empty() || self.modal() || now < self.next_walk {
            return;
        }
        let at = self.step_time(now);
        let dir = self.walk.remove(0);
        self.plan_step(cx, at, dir);
        if !self.step(cx, dir) {
            self.walk.clear();
            self.walk_target = None;
            return;
        }
        if self.walk.is_empty() {
            if let Some(id) = self.walk_target.take() {
                self.walk_to_marker(cx, &id);
            }
        }
    }

    /// Whether the leader walks at the clock's now: on a walked path, or within a step and `WALK_LINGER_MS` of the last step
    /// (that step's own time: Shift pressed during it must not end it early, TURNFEEL1).
    fn moving(&self) -> bool {
        !self.walk.is_empty() || self.moved_at.is_some_and(|(t, step)| self.anim_ms - t < step + WALK_LINGER_MS)
    }

    fn interact(&mut self, cx: &mut Cx, dir: Pos) {
        self.turn(dir);
        let d = cx.data.clone();
        let res = explore::interact(&d, cx.gs_mut(), dir);
        if !json::b(&res, "ok", false) {
            show_errors(cx, json::sget(&res, "errors"));
            return;
        }
        match s(&res, "kind").as_str() {
            "npc" => self.talk(cx, &s(&res, "character")),
            "lore" => self.lore = Some((s(&res, "name"), s(&res, "text"))),
            "chest" => {
                if json::b(&res, "empty", false) {
                    cx.log("The chest is empty.");
                } else {
                    let mut got: Vec<String> = json::ints(json::get(&res, "items")).iter().map(|(id, n)| format!("{} x{n}", cx.data.item_name(id))).collect();
                    if json::i(&res, "gold", 0) > 0 {
                        got.push(format!("{} {}", json::i(&res, "gold", 0), purse_word(cx, PurseWord::Currency)));
                    }
                    if json::i(&res, "supplies", 0) > 0 {
                        got.push(format!("{} {}", json::i(&res, "supplies", 0), purse_word(cx, PurseWord::Upkeep)));
                    }
                    cx.log(if got.is_empty() { "The chest holds nothing useful.".to_string() } else { format!("Found: {}.", got.join(", ")) });
                    for n in json::sget(&res, "notes") {
                        cx.log(n);
                    }
                }
            }
            _ => cx.log("Nothing there."),
        }
    }

    /// Opens the conversation with `character` (`Services::talk`).
    pub fn talk(&mut self, cx: &mut Cx, character: &str) {
        self.svc.talk(cx, character);
    }

    /// BFS over open, unblocked tiles to `target` (or next to it); the step directions.
    fn path_to(&self, cx: &Cx, target: Pos, adjacent: bool) -> Option<Vec<Pos>> {
        let d = &cx.data;
        let gs = cx.gs();
        let loc = explore::location(d, gs);
        let t = explore::template_of(d, loc);
        let loc_qid = s(loc, "_qid");
        let steps = gs.explore_int("loc_steps");
        let mut blocked = Vec::new();
        for m in explore::markers_of(loc) {
            let st = explore::presence(d, gs, &loc_qid, &m);
            if st == "locked" || (st == "present" && ["npc", "chest", "lore"].contains(&s(&m, "kind").as_str())) {
                blocked.push(explore::marker_pos(t, &m, steps));
            }
        }
        let start = gs.position;
        let mut prev: std::collections::HashMap<Pos, (Pos, Pos)> = std::collections::HashMap::new();
        let mut queue = std::collections::VecDeque::from([start]);
        let mut seen = vec![start];
        while let Some(cur) = queue.pop_front() {
            let reached = if adjacent { (cur.0 - target.0).abs() + (cur.1 - target.1).abs() == 1 } else { cur == target };
            if reached {
                let mut dirs = Vec::new();
                let mut c = cur;
                while c != start {
                    let (p, dir) = prev[&c];
                    dirs.push(dir);
                    c = p;
                }
                dirs.reverse();
                return Some(dirs);
            }
            for dir in explore::DIRS {
                let n = (cur.0 + dir.0, cur.1 + dir.1);
                if seen.contains(&n) || !explore::is_open(t, n.0, n.1) || (blocked.contains(&n) && (adjacent || n != target)) {
                    continue;
                }
                seen.push(n);
                prev.insert(n, (cur, dir));
                queue.push_back(n);
            }
        }
        None
    }

    fn walk_to_marker(&mut self, cx: &mut Cx, id: &str) {
        let d = cx.data.clone();
        let gs = cx.gs();
        let loc = explore::location(&d, gs);
        let t = explore::template_of(&d, loc);
        let Some(m) = explore::markers_of(loc).into_iter().find(|m| s(m, "id") == id) else { return };
        let at = explore::marker_pos(t, &m, gs.explore_int("loc_steps"));
        let solid = ["npc", "chest", "lore"].contains(&s(&m, "kind").as_str()) || explore::presence(&d, gs, &s(loc, "_qid"), &m) == "locked";
        if solid {
            let dir = (at.0 - gs.position.0, at.1 - gs.position.1);
            if explore::DIRS.contains(&dir) {
                self.interact(cx, dir);
                return;
            }
        }
        match self.path_to(cx, at, solid) {
            Some(path) if !path.is_empty() => {
                self.walk = path;
                self.walk_target = if solid { Some(id.to_string()) } else { None };
            }
            _ => cx.log("You can't get there."),
        }
    }

    fn act(&mut self, cx: &mut Cx, a: RAct) {
        let d = cx.data.clone();
        match a {
            RAct::MapClick => {
                let (tx, ty) = self.geo.cell_at(self.mouse.0, self.mouse.1);
                // MAPNAV1-U: a tile no open path reaches (a wall, water, a tile walled off) says so
                match self.path_to(cx, (tx, ty), false) {
                    Some(path) => {
                        self.walk = path;
                        self.walk_target = None;
                    }
                    None => cx.log("You can't get there."),
                }
            }
            RAct::Marker(id) => self.walk_to_marker(cx, &id),
            RAct::Pan(id) => {
                let Some(at) = marker_tile(cx, &id) else { return };
                if self.pan == Some(at) {
                    self.walk_to_marker(cx, &id);
                } else {
                    self.pan = Some(at);
                    self.pan_loc = cx.gs().location_id.clone();
                }
            }
            RAct::Lead => {
                let d = cx.data.clone();
                let world = cx.gs().world_id.clone();
                let before = leader_id(cx);
                let Some(id) = cx.gs_mut().cycle_explore_leader(1, standing(&d, &world)) else { return };
                if id != before {
                    let name = cx.gs().party.member(&id).map(|m| crate::hub::member_name(cx, m)).unwrap_or_default();
                    cx.log(format!("{name} leads the party."));
                    self.leader = ClipPlayer::default();
                }
            }
            RAct::Exit(to) => {
                self.pan = None;
                let res = explore::take_exit(&d, cx.gs_mut(), &to, explore::SPAWN);
                if !json::b(&res, "ok", false) {
                    show_errors(cx, json::sget(&res, "errors"));
                    return;
                }
                let events = json::aget(&res, "events").to_vec();
                log_events(cx, &events);
                crate::runover::take(cx, &res);
                if to == explore::WORLD_MAP {
                    cx.nav.push(Nav::Push(Box::new(crate::worldmap::WorldMap::new())));
                }
            }
            RAct::Menu => cx.nav.push(Nav::Push(Box::new(crate::hub::Hub::default()))),
            RAct::Hud(h) => {
                self.walk.clear();
                hud_open(cx, h, &REGION_HELP);
            }
            RAct::Svc(a) => {
                if self.svc.act(cx, a) {
                    self.walk.clear();
                }
            }
            RAct::CloseLore => self.lore = None,
            RAct::Legend => {
                let open = cx.settings.v["map_legend_open"].as_bool().unwrap_or(true);
                cx.settings.v["map_legend_open"] = json!(!open);
                if let Err(e) = cx.settings.save() {
                    cx.log(format!("Settings could not be saved: {e}"));
                }
            }
            RAct::Log => self.log_open = !self.log_open,
        }
    }

    fn marker_style(kind: &str, state: &str) -> (Rgba, &'static str) {
        match (kind, state) {
            ("exit", "locked") => (rgb(180, 70, 60), ">"),
            ("exit", _) => (rgb(70, 190, 100), ">"),
            ("chest", "spent") => (rgb(90, 74, 50), "c"),
            ("chest", _) => (rgb(200, 150, 60), "c"),
            ("lore", _) => (rgb(150, 110, 220), "L"),
            ("encounter", _) => (rgb(220, 70, 60), "!"),
            ("roamer", _) => (rgb(230, 110, 60), "R"),
            ("npc", _) => (rgb(230, 190, 80), ""),
            _ => (rgb(230, 220, 120), "?"),
        }
    }
}

impl Screen<GameCx> for RegionMap {
    /// The arrow keys walk the party.
    fn arrow_focus(&self) -> bool {
        false
    }
    fn hold_walk(&self) -> bool {
        true
    }
    fn id(&self) -> &'static str {
        "region_map"
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        self.mouse = mouse;
        self.arrows.clear();
        // MAPNAV1-U: a pan set in another location (an exit, the world map, a fight's wake) is dropped
        if self.pan.is_some() && self.pan_loc != cx.gs().location_id {
            self.pan = None;
        }
        // Yes to "Discard this trade?" (PLAYUI1-P): back to the conversation
        if self.svc.take_discard(cx) {
            self.walk.clear();
        }
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        // the open conversation; a new node starts its line and its replies scrolled to the top; PORTRAITFX1-D: a
        // portrait left out of the last frame (the conversation ended or hid) is dropped
        let talk_view = self.svc.begin_frame(g, &d, &gs);
        let mut ui: Ui<RAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().game("map_ground", rgb(14, 18, 26)));
        let loc = explore::location(&d, &gs);
        let t = explore::template_of(&d, loc);
        let talking = self.svc.talking();
        let modal = self.modal();
        // measured before the map: the camera keeps both speakers above the box (also while the shop hides it, so the
        // map does not jump when the shop opens or closes)
        let talk_box = talk_view.as_ref().map(|v| talk_layout(&mut ui.p, v));
        // -- Map (PLAYUI1-M, spec PLAYUI1 3.2; REGIONVIEW1): the full width under the HUD bar; the camera follows the party's
        // drawn position (while talking, the midpoint of the party and the speaker, inside the map area above the dialogue
        // box), stopping at the map's edges with a small margin
        let map_area = Rect::new(0, HUD_H, W, H - HUD_H);
        let focus_area = talk_box.as_ref().map_or(map_area, |b| Rect::new(0, HUD_H, W, b.r.y - HUD_H));
        let names_always = cx.settings.v["map_names"].as_str() == Some("always");
        // MAPNAV1-U: (id, name, tile) of the legend's markers out of view (their edge arrows follow the legend)
        let mut off_view: Vec<(String, String, Rect)> = Vec::new();
        // where the names drew: an edge arrow keeps off them (near an edge they meet the arrows' band)
        let mut name_boxes: Vec<Rect> = Vec::new();
        if !t.is_empty() {
            // EXPLOREVIEW1: the view this frame (a switch shows on the next frame)
            let view = mapview::View::of_setting(&cx.settings.v["region_view"]);
            let tilted = view == mapview::View::Tilted;
            // the zoom sizes the whole scene (the cell, and all that takes its size from it)
            let q = mapview::zoom_of(&cx.settings.v["map_zoom"]);
            let cell_size = mapview::cell_of(view, q);
            let ts = cell_size.0;
            // REGIONVIEW1: the party's drawn position, and what the camera looks at (MAPNAV1-U: outside a talk the panned
            // tile, a legend row or an edge arrow, when set); the party's glide always counts, the pan and talk ease
            let scale = ui.p.v.s;
            let step_at = self.step_at.take();
            let leg = if step_at.is_some() { self.leg } else { mapview::Leg::plain(self.step.wall) };
            let (drawn, jumped) = self.glide.update(&gs.location_id, gs.position, step_at, self.frame_at, leg);
            let tile_at = |p: Pos| (p.0 as f32, p.1 as f32);
            let (look, at) = match (talk_view.as_ref(), self.pan) {
                (Some(v), _) => {
                    let who = self.speaker_tile(cx, v).unwrap_or(gs.position);
                    (mapview::Look::Talk(who, focus_area), ((drawn.0 + who.0 as f32) / 2.0, (drawn.1 + who.1 as f32) / 2.0))
                }
                (None, Some(p)) => (mapview::Look::Pan(p), tile_at(p)),
                (None, None) => (mapview::Look::Party, drawn),
            };
            let (map_w, map_h) = explore::size_of(t);
            let target = mapview::camera_px(cell_size, focus_area, (map_w as i32, map_h as i32), at, scale);
            let camera = self.follow.update(look, (view, q), target, scale, self.frame_at, jumped || cx.settings.reduce_motion());
            let (geo, shift) = mapview::placed(view, q, camera, scale);
            self.geo = geo;
            // ASSETORG2 4.5, LOOK1 5.4: the location's `tileset`, the world lineage's tile sets child first, then common
            let mut lineage = d.lineage(&gs.world_id);
            lineage.reverse();
            let sets = emberwake_content::art::region_sets(&lineage, &s(loc, "tileset"));
            let set_refs: Vec<&str> = sets.iter().map(String::as_str).collect();
            ui.push_clip(map_area);
            // the map draws under the camera's rest (physical px); rects, clicks and hit areas keep `geo`'s whole px
            let base_view = ui.p.v;
            let map_view = PView { ox: base_view.ox + shift.0, oy: base_view.oy + shift.1, ..base_view };
            ui.p.v = map_view;
            // REGIONGROUND1-L: the composed ground under everything; it stands in for the `.` and `~` tiles below
            self.ground.sync(cx, t, &sets, view, q);
            let has_ground = self.ground.draw(&mut ui.p, &geo);
            // the cells to draw: those meeting the area, by a logical px more (the camera's rest can bring one in)
            let paint_area = Rect::new(map_area.x - 1, map_area.y - 1, map_area.w + 2, map_area.h + 2);
            let (x0, x1) = geo.cols(paint_area);
            let (y0, y1) = geo.rows(paint_area);
            // the art's size: the tile at the cell (square), or the 32 x 32 art the painter squashes by nearest
            // sampling into the 16 x 12 cell (tilted, EXPLOREVIEW1 3.2 until LOOK1 makes 4:3 region art)
            let art_size = if tilted { ART_TILE * q / 4 } else { ts };
            // render only: off the map is the wall (`#`), nothing walkable
            let ch_at = |x: i32, y: i32| explore::tile(t, x as i64, y as i64).unwrap_or('#');
            let border_at = |x: i32, y: i32| on_border(t, (map_w, map_h), x as i64, y as i64);
            let walls = WallArt::of(&sets, &s(loc, "boundary"));
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let r = geo.cell(x, y);
                    let ch = ch_at(x, y);
                    // the border draws after this pass; tilted, a wall is a block in the sorted pass below
                    if border_at(x, y) || (tilted && ch == '#') {
                        continue;
                    }
                    if has_ground && matches!(ch, '.' | '~') {
                        continue;
                    }
                    let tile = emberwake_content::art::region_tile(ch);
                    match tile.and_then(|t| cx.art.tile_in(&set_refs, t, x, y, art_size)) {
                        Some(img) => ui.p.image(img, r),
                        None => ui.p.fill(r, match ch {
                            '.' => rgb(70, 90, 64),
                            '#' => WALL_FILL,
                            '~' => rgb(40, 90, 100),
                            _ => rgb(20, 20, 24),
                        }),
                    }
                    if tilted && ch == '~' {
                        draw_lips(&mut ui, r, ch_at(x, y - 1) != '~', ch_at(x, y + 1) != '~');
                    }
                }
            }
            // REGIONVIEW1: the border, flat so that it ends on the map's edge: tilted, a wall on an open cell keeps its
            // face and the shade it drops, inside its own cell and the cell south (the row above the area can shade the
            // first one)
            for y in (if tilted { y0 - 1 } else { y0 })..=y1 {
                for x in x0..=x1 {
                    if border_at(x, y) {
                        draw_border(&mut ui, cx, &walls, x, y, geo.cell(x, y), art_size, tilted && ch_at(x, y + 1) != '#');
                    }
                }
            }
            if !modal {
                ui.push("map", "map", s_or(loc, "name", ""), map_area, true, false, Some(RAct::MapClick), Vec::new());
            }
            // PROPS1 4.2-4.6: the ground props over the tiles; the standing props go into the sorted pass below. Props
            // are art only: no widget, and walking still comes from `rows`
            let props = crate::props::layout(t, &lineage, (geo.ox, geo.oy), geo.cell_size(), paint_area, |dirs, variant, x, y| {
                let rel = cx.art.pick(dirs, variant, x, y)?;
                let size = cx.art.size_of(&rel)?;
                Some((rel, size))
            });
            // REGIONVIEW1: past the map is boundary art only: a prop's art that overhangs it is cut there (a footprint
            // cannot leave the map: check P8)
            let map_rect = Rect::new(geo.ox, geo.oy, map_w as i32 * geo.cell_size().0, map_h as i32 * geo.cell_size().1);
            for p in props.iter().filter(|p| p.ground) {
                draw_prop(&mut ui, cx, p, map_rect, q);
            }
            let steps = gs.explore_int("loc_steps");
            // PLAYUI1-R: the people in the conversation turn to the party; the clock the sprites play at
            let talk_tiles: Vec<Pos> = talk_view.as_ref().map(|v| self.talk_tiles(cx, v)).unwrap_or_default();
            let now = self.anim_ms as u64;
            let ahead = (gs.position.0 + self.facing.0, gs.position.1 + self.facing.1);
            let legend_hover = self.legend_hover(live);
            // (marker, name, the name sits over a badge)
            let mut labels: Vec<(Rect, String, bool)> = Vec::new();
            // the markers to draw in the sorted pass (their click areas and names are taken here)
            let mut marks: Vec<MarkDraw> = Vec::new();
            let mut hits: Vec<MarkHit> = Vec::new();
            for raw in json::aget(loc, "markers") {
                let m = explore::marker_full(json::dict(raw));
                let view = explore::marker_view(&d, &gs, loc, &m);
                let state = s(&view, "state");
                if state == "absent" {
                    continue;
                }
                let at = explore::marker_pos(t, &m, steps);
                if at.0 < 0 {
                    continue;
                }
                let r = geo.cell_of(at);
                let kind = s(&m, "kind");
                let id = s(&m, "id");
                let label = s(&view, "label");
                let seen = r.intersect(map_area);
                if seen.w <= 0 || seen.h <= 0 {
                    // MAPNAV1-U: a legend entry out of view gets an edge arrow
                    if kind != "point" {
                        off_view.push((id, label, r));
                    }
                    continue;
                }
                let mut services = Vec::new();
                let mut npc = None;
                // the drawn top: the icon standing on the cell (the cell itself in the square view), else the sprite's
                let mut top = mapview::icon_rect(r);
                if kind == "npc" {
                    let who = d.resolve(&gs.world_id, "characters", &s(&m, "character"));
                    services = explore::npc_services(&d, &gs, &s(who, "_qid"));
                    let (body, palette, _) = crate::screens::character_art(&d, who);
                    // 2.4: the person's body, else its class body (the chain), else today's token
                    let sprite = cx.art.first_body(&emberwake_content::art::body_chain(&body));
                    if sprite.is_some() {
                        top = mapview::sprite_rect(r);
                    }
                    // facing the party while in the conversation (four ways, DIR4-M: up or down when the party is
                    // more above or below, keeping their left or right); else the row's `face`, down by default
                    // (NPCFACE1; a body with no down row keeps its side view)
                    let rest = explore::marker_facing(&m);
                    let face = if talk_tiles.contains(&at) { rest.toward((gs.position.0 - at.0) as i32, (gs.position.1 - at.1) as i32) } else { rest };
                    npc = Some(NpcDraw { body, sprite, palette, initial: s(who, "name").chars().next().unwrap_or('?').to_string(), face, offset: idle_offset(&s(who, "id")) });
                }
                // 2.2: the first service is the badge (turn_in and offer are today's `ready` and `quest`)
                marks.push(MarkDraw { id: id.clone(), r, top: top.y, kind: kind.clone(), state: state.clone(), npc, badge: services.first().copied() });
                // EXPLOREVIEW1 3.5: a person's click area is their cell, so the cell north of them still walks (a tall
                // sprite's head takes no click); tilted, an icon's is the cell and the icon standing on it
                let hit = if tilted && kind != "npc" { mapview::union(r, top) } else { r };
                let name_rect = Rect::new(r.x, top.y, r.w, r.h);
                // 2.3: names always (people and exits, MAPREAD1), or on hover: the marker or its legend row under the
                // pointer, or the marker the party faces (tilted: the hovered one is the front-most, below)
                let named = if names_always { kind == "npc" || kind == "exit" } else { (!modal && !tilted && ui.hover(r)) || legend_hover.as_deref() == Some(id.as_str()) || at == ahead };
                if named {
                    labels.push((name_rect, label.clone(), !services.is_empty()));
                }
                let mut t = tip(&label, &s(&view, "verb"));
                if !services.is_empty() {
                    t.push((services_text(&services), th().info));
                }
                hits.push(MarkHit { id, label, hit, foot: r.y + r.h, tip: t, named, name_rect, badge: !services.is_empty() });
            }
            // tilted: the front-most click area under the pointer names its marker (click areas overlap)
            if tilted && !names_always && !modal {
                if let Some(h) = front_hit(&hits, |r| ui.hover(r)).filter(|h| !h.named) {
                    labels.push((h.name_rect, h.label.clone(), h.badge));
                }
            }
            // EXPLOREVIEW1 3.3: tilted, the walls inside the map (not its border) are blocks in the sorted pass; the row
            // above the area can still shade the first row
            let mut blocks: Vec<Block> = Vec::new();
            if tilted {
                for y in (y0 - 1)..=y1 {
                    for x in x0..=x1 {
                        if ch_at(x, y) != '#' || border_at(x, y) {
                            continue;
                        }
                        let cell = geo.cell(x, y);
                        let face = ch_at(x, y + 1) != '#';
                        let rect = mapview::block_rect(cell, face);
                        let seen = Rect::new(rect.x, rect.y, rect.w, rect.h + if face { mapview::shade_of(cell.h) } else { 0 }).intersect(paint_area);
                        if seen.w > 0 && seen.h > 0 {
                            blocks.push(Block { x, y, cell, rect, face });
                        }
                    }
                }
            }
            // the sorted pass (PROPS1 4.4): standing props, markers and the party back to front by their feet (the
            // sprites stand on their tiles, so lower rows cover upper ones, spec PLAYUI1 2.4)
            // REGIONVIEW1: the party's cell where it is drawn, and the view that puts it on its own physical px
            let (pr, party_shift) = mapview::party_px(view, q, camera, drawn, scale);
            let party_view = PView { ox: base_view.ox + party_shift.0, oy: base_view.oy + party_shift.1, ..base_view };
            let shot = CamShot { px: camera, party: (party_view.x(pr.x as f32), party_view.y(pr.y as f32)), scale };
            self.cam = shot;
            if let Some(now) = self.frame_at.filter(|_| cx.settings.v["camera_trail"] == json!(true)) {
                self.cam_trail.push_back((now.saturating_duration_since(self.opened_at).as_secs_f64(), shot));
                if self.cam_trail.len() > CAM_TRAIL {
                    self.cam_trail.pop_front();
                }
            }
            let (pcx, pcy) = (pr.x as f32 + pr.w as f32 / 2.0, pr.y as f32 + pr.h as f32 / 2.0);
            // 2.4: the leader (the first active member): `walk` while moving, else `idle`
            let (leader_body, leader_palette) = leader_art(cx);
            let leader_sprite = cx.art.first_body(&emberwake_content::art::body_chain(&leader_body));
            let base = if self.moving() { "walk" } else { "idle" };
            // TURNFEEL1: while a turn finishes its leg the sprite faces the way it still goes
            let leader_shown = self.frame_at.and_then(|at| self.glide.heading(at)).map_or(self.leader_face, |d| self.leader_face.toward(d.0, d.1));
            self.leader.set_base(base, now);
            let drawn_party = if leader_sprite.is_some() { mapview::sprite_rect(pr) } else { pr };
            let order = crate::props::sorted(&props, &blocks, &marks.iter().map(|m| m.r).collect::<Vec<_>>(), drawn_party);
            let idle = ClipPlayer::default();
            let mut sprites = Vec::new();
            let mut party_frame = None;
            for item in &order {
                match *item {
                    Item::Prop(i) => draw_prop(&mut ui, cx, &props[i], map_rect, q),
                    Item::Block(i) => draw_block(&mut ui, cx, &blocks[i], &walls, &ch_at, &border_at),
                    Item::Marker(i) => {
                        if let Some(sp) = draw_marker(&mut ui, cx, &marks[i], ts, &idle, now) {
                            sprites.push(sp.info(&format!("marker.{}", marks[i].id)));
                        }
                    }
                    Item::Party => {
                        ui.p.v = party_view;
                        let sprite = leader_sprite.as_deref().and_then(|b| draw_sprite(&mut ui, cx, b, &leader_palette, &self.leader, now, leader_shown, pr, crate::props::PARTY));
                        match sprite {
                            Some(sp) => {
                                sprites.push(sp.info("party"));
                                party_frame = Some(sp);
                            }
                            None => {
                                ui.p.circle(pcx, pcy, ts as f32 * 0.3, crate::props::PARTY);
                                ui.p.line(pcx, pcy, pcx + self.facing.0 as f32 * ts as f32 * 0.32, pcy + self.facing.1 as f32 * ts as f32 * 0.32, rgb(40, 30, 10), 0.0);
                            }
                        }
                        ui.p.v = map_view;
                    }
                }
            }
            // 4.5: a standing prop or a wall block drawn over the party: its outline over them (the sprite's silhouette
            // outline)
            if crate::props::party_hidden(&props, &blocks, &order, drawn_party) {
                ui.p.v = party_view;
                match party_frame.as_ref().and_then(|sp| cx.art.body_outline(&sp.body, &leader_palette, &sp.clip, sp.frame, sp.px, crate::props::PARTY, sp.flip)) {
                    Some(o) => ui.p.image(o, Rect::new(drawn_party.x - 1, drawn_party.y - 1, drawn_party.w + 2, drawn_party.h + 2)),
                    None => ui.p.circle(pcx, pcy, ts as f32 * 0.3, crate::props::PARTY),
                }
                ui.p.v = map_view;
            }
            self.sprites = sprites;
            // overlays, never covered by a prop: the service badges (over a sprite's head), then the names
            for m in &marks {
                let Some(first) = m.badge else { continue };
                let (side, over) = (9 * q / 4, 5 * q / 4);
                let br = Rect::new(m.r.x + ts - over, m.top - over, side, side);
                let px = ui.icon_px(Rect::new(0, 0, br.w + 4, br.h + 4));
                match cx.art.image(&service_icon(first), px, px) {
                    Some(img) => ui.p.image(img, br),
                    None => service_glyph(&mut ui, first, br),
                }
            }
            // MAPNAV1-U: a click on the leader (or Q) passes the lead to the next standing active member
            let lead = leader_id(cx);
            let lead_name = gs.party.member(&lead).map(|m| crate::hub::member_name(cx, m)).unwrap_or_default();
            let can_pass = gs.leader_choices(standing(&d, &gs.world_id)).len() > 1;
            let lead_tip = tip(&format!("{lead_name} leads"), if can_pass { "Q or click: the next member leads." } else { "No one else can lead." });
            // EXPLOREVIEW1 3.5: the click areas in feet order, front-most last (square: the markers in list order, then
            // the party, on their cells); a click takes the last one under the pointer
            let party_foot = pr.y + pr.h;
            if tilted {
                sort_hits(&mut hits);
            }
            let (behind, front): (Vec<MarkHit>, Vec<MarkHit>) = hits.into_iter().partition(|h| !tilted || h.foot <= party_foot);
            if !modal {
                push_hits(&mut ui, behind);
            }
            ui.push("party", "party", format!("Party at {}, {}", gs.position.0, gs.position.1), pr, true, false, (!modal && can_pass).then_some(RAct::Lead), lead_tip);
            if !modal {
                push_hits(&mut ui, front);
            }
            // the names over everything on the map, kept inside the map area
            for (r, label, badge) in labels {
                let w = ui.p.text_w(&label, SMALL);
                let lx = (r.x as f32 + ts as f32 / 2.0 - w / 2.0).min((map_area.x + map_area.w) as f32 - 2.0 - w).max(map_area.x as f32 + 2.0);
                let ly = (r.y as f32 - if badge { (5 * q / 4) as f32 } else { 1.0 } - ui.p.line_h(SMALL)).max(map_area.y as f32 + 1.0);
                ui.p.text(lx + 0.5, ly + 0.5, &label, SMALL, rgba(0, 0, 0, 200));
                ui.p.text(lx, ly, &label, SMALL, th().text);
                name_boxes.push(Rect::new(lx.floor() as i32, ly.floor() as i32, w.ceil() as i32 + 1, ui.p.line_h(SMALL).ceil() as i32));
            }
            ui.p.v = base_view;
            ui.pop_clip();
        } else {
            ui.p.text(20.0, 40.0, &s_or(loc, "name", "Nowhere"), 11.0, th().gold);
            let y = ui.para(20, 58, 440, &s(loc, "description"), BODY, th().text);
            for (i, ex) in explore::menu(&d, &gs).iter().enumerate() {
                let to = s(ex, "to");
                let label = s(ex, "label");
                ui.button(&format!("exit.{to}"), Rect::new(20, y + 10 + i as i32 * 22, 200, 18), &label, RAct::Exit(to.clone()), !modal);
            }
        }
        // -- HUD bar (PLAYUI1-H): the location name; gold, supplies and the icon buttons at the right
        let name = s_or(loc, "name", "").to_string();
        let why = if modal { "Finish the conversation, shop or page first." } else { "" };
        hud_bar(&mut ui, cx, 4, &name, ("region.menu", "Esc"), why, &REGION_HELP, RAct::Hud);
        // -- The legend (2.1 A) at the top right and the event log ticker (MAPLOG1) at the bottom left; off in a talk
        let mut panes = Vec::new();
        if !t.is_empty() && !talking {
            panes.push(self.draw_legend(&mut ui, cx, modal));
        }
        if !talking {
            panes.extend(self.draw_ticker(&mut ui, cx, modal));
        }
        panes.extend(name_boxes);
        // -- MAPNAV1-U (DR2 6 A): an arrow at the map's edge toward each legend entry out of view (off in a talk, as
        // the legend), moved along the edge off the legend and the ticker; a click looks there
        if !talking {
            ui.push_clip(map_area);
            // REGIONVIEW1: the arrows placed so far, so two toward the same side never stack
            let mut placed: Vec<(f32, f32)> = Vec::new();
            for (id, label, r) in off_view {
                let target = (r.x as f32 + r.w as f32 / 2.0, r.y as f32 + r.h as f32 / 2.0);
                let Some((ax, ay)) = edge_arrow(map_area, target).map(|(x, y, _, _)| beside_arrows(map_area, clear_of(map_area, (x, y), &panes), &placed, &panes)) else { continue };
                placed.push((ax, ay));
                let (dx, dy) = (target.0 - ax, target.1 - ay);
                let len = (dx * dx + dy * dy).sqrt().max(0.5);
                let (ux, uy) = (dx / len, dy / len);
                let hr = Rect::new(ax.round() as i32 - ARROW / 2, ay.round() as i32 - ARROW / 2, ARROW, ARROW);
                let hover = !modal && ui.hover(hr);
                ui.p.circle(ax, ay, ARROW as f32 / 2.0, th().game("map_arrow", rgba(12, 18, 28, 210)));
                let half = ARROW as f32 * 0.3;
                ui.p.arrow(ax - ux * half, ay - uy * half, ax + ux * half, ay + uy * half, if hover { th().text } else { th().gold }, 0.0, half * 0.9);
                self.arrows.push(json!({"id": id, "at": [ax.round() as i64, ay.round() as i64], "dir": [(ux * 100.0).round() / 100.0, (uy * 100.0).round() / 100.0]}));
                if !modal {
                    ui.push(format!("arrow.{id}"), "arrow", label.clone(), hr, true, false, Some(RAct::Pan(id)), tip(&label, "Out of view: click to look there."));
                }
            }
            ui.pop_clip();
        }
        // -- The dialogue box (3.1); while the shop is open (3.3) the box hides and the shop sits centred over the dimmed
        // map; an open board (STORYUI1 B11) takes the shop's place; the turn-in picker over them (`Services::draw`)
        if let (Some(view), Some(b)) = (talk_view.as_ref(), talk_box.as_ref()) {
            self.svc.draw(&mut ui, cx, view, b, map_area);
        }
        if let Some((name, text)) = self.lore.clone() {
            ui.p.fill(Rect::new(0, 0, W, H), th().scrim);
            let fr = Rect::new(150, 60, 340, 220);
            ui.frame(fr, &name, false);
            ui.para(fr.x + 10, fr.y + 30, fr.w - 20, &text, BODY, th().text);
            ui.button("lore.close", Rect::new(fr.x + fr.w - 60, fr.y + fr.h - 24, 50, 16), "Close", RAct::CloseLore, true);
        }
        self.w = ui.widgets;
    }

    fn tick(&mut self, cx: &mut Cx) {
        self.tick_at(cx, Instant::now());
    }

    fn key(&mut self, cx: &mut Cx, k: Key) {
        // VIEWKEYS1, VIEWKEYS2, TURNFEEL1: a debug build's F6 (map zoom), F11 (walk step), F10 (walk feel), F5 (turn corner) and
        // F9 (window size), whatever is open
        if crate::viewkeys::zoom_key(cx, k) || crate::viewkeys::walk_key(cx, k) || crate::viewkeys::feel_key(cx, k) || crate::viewkeys::corner_key(cx, k) || crate::viewkeys::window_key(cx, k, crate::viewkeys::Shown::Log) {
            return;
        }
        // EXPLOREVIEW1: a debug build's F7 flips the region view live, whatever is open
        if k == Key::Fn(7) {
            if cfg!(debug_assertions) {
                flip_view(cx);
            }
            return;
        }
        // an open conversation takes the keys (`Services::key`)
        if let Some(fought) = self.svc.key(cx, k) {
            if fought {
                self.walk.clear();
            }
            return;
        }
        if self.lore.is_some() {
            if matches!(k, Key::Enter | Key::Space) {
                self.lore = None;
            }
            return;
        }
        if let Some(dir) = k.dir() {
            self.walk.clear();
            self.pan = None;
            let t = explore::template_of(&cx.data, explore::location(&cx.data, cx.gs()));
            if !t.is_empty() {
                self.step = mapview::WalkStep::of(&cx.settings.v, cx.held.shift);
                // the frame's clock: the press moves from this frame on, and the steps chain from it
                let at = self.frame_at.unwrap_or_else(Instant::now);
                self.plan_step(cx, at, dir);
                self.hold = self.step(cx, dir);
            }
        } else if k == Key::Char('e') || k == Key::Enter {
            let dir = self.facing;
            self.interact(cx, dir);
        } else if matches!(k, Key::Char(c) if c.eq_ignore_ascii_case(&'l')) {
            self.act(cx, RAct::Log);
        } else if matches!(k, Key::Char(c) if c.eq_ignore_ascii_case(&'q')) {
            self.act(cx, RAct::Lead);
        } else if let Some(h) = hud_key(k) {
            self.act(cx, RAct::Hud(h));
        }
    }

    fn back(&mut self, cx: &mut Cx) {
        if self.lore.is_some() {
            self.lore = None;
        } else if let Some(fought) = self.svc.back(cx) {
            // the turn-in picker, the shop, the board, then the conversation (`Services::back`)
            if fought {
                self.walk.clear();
            }
        } else if self.pan.is_some() {
            // MAPNAV1-U: Esc first brings the camera back to the party
            self.pan = None;
        } else {
            self.walk.clear();
            self.act(cx, RAct::Menu);
        }
    }

    fn state(&self, cx: &Cx) -> Value {
        let gs = cx.gs();
        let feel = mapview::WalkFeel::of(&cx.settings.v);
        let svc = self.svc.state(cx);
        json!({"location": gs.location_id, "position": [gs.position.0, gs.position.1], "facing": [self.facing.0, self.facing.1],
            // TURNFEEL1: the feel options and the last step's times (wall ms)
            "walk_feel": {"turn_beat": feel.beat, "equal_speed": feel.equal, "corner": feel.corner.name(), "step_at_ms": self.last_step.filter(|_| cx.settings.v["camera_trail"] == json!(true)).map(|t| t.saturating_duration_since(self.opened_at).as_secs_f64() * 1000.0), "glide_ms": self.leg.glide, "delay_ms": self.leg.delay},
            "walking": !self.walk.is_empty(), "log": cx.log.iter().rev().take(8).rev().collect::<Vec<_>>(),
            "dialogue": svc["dialogue"], "turn_in": svc["turn_in"],
            "lore": self.lore.as_ref().map(|l| l.0.clone()), "legend_open": cx.settings.v["map_legend_open"].as_bool().unwrap_or(true), "log_open": self.log_open,
            "shop": svc["shop"], "board": svc["board"], "board_pane": svc["board_pane"], "sprites": self.sprites,
            // MAPNAV1-U: who leads, the edge arrows of the last frame and the tile the camera looks at (null: the party)
            // REGIONVIEW1: the camera (physical px of cell (0, 0)), the party's physical px, and the last frames of both
            "camera": {"px": [self.cam.px.0, self.cam.px.1], "party": [self.cam.party.0, self.cam.party.1], "scale": self.cam.scale, "zoom": self.geo.q as f64 / 4.0,
                "trail": (!self.cam_trail.is_empty()).then(|| self.cam_trail.iter().map(|(t, c)| json!([(t * 100000.0).round() / 100.0, c.px.0, c.px.1, c.party.0, c.party.1])).collect::<Vec<_>>())},
            "view": mapview::View::of_setting(&cx.settings.v["region_view"]).name(), "ground": self.ground.state(), "leader": leader_id(cx), "arrows": self.arrows, "pan": self.pan.filter(|_| self.pan_loc == gs.location_id).map(|p| json!([p.0, p.1])),
            // PORTRAITFX1 4.11: the speaker's portrait while the talk box shows (the shop or the board hides it)
            "portraits": svc["portraits"]})
    }

    /// MUSIC2: shop while the shop is open, else the location's kind.
    fn music(&self, cx: &Cx) -> Option<String> {
        let shop = !self.svc.shop_qid().is_empty();
        cx.game.as_ref().and_then(|g| music::game_track(cx, music::region_context(&cx.data, &g.gs, shop)))
    }

    widget_plumbing!();
}

impl RegionMap {
    /// The tile of the person speaking (the node's speaker, else the one talked to): their marker here that is not
    /// absent, the nearest to the party when several show them; None when they have no marker on this map.
    fn speaker_tile(&self, cx: &Cx, view: &Dict) -> Option<Pos> {
        self.talk_tiles(cx, view).first().copied()
    }

    /// The tiles of the people in the conversation (PLAYUI1-R: they face the party): the node's speaker's marker, then
    /// the one talked to (when someone else), each the nearest to the party of their markers here that are not absent.
    fn talk_tiles(&self, cx: &Cx, view: &Dict) -> Vec<Pos> {
        let d = &cx.data;
        let gs = cx.gs();
        let loc = explore::location(d, gs);
        let t = explore::template_of(d, loc);
        let loc_qid = s(loc, "_qid");
        let steps = gs.explore_int("loc_steps");
        let talked = self.svc.talked();
        let mut out = Vec::new();
        for who in [s(view, "speaker"), talked] {
            if who.is_empty() {
                continue;
            }
            let near = explore::markers_of(loc)
                .into_iter()
                .filter(|m| s(m, "kind") == "npc" && emberwake_world::dialogue::character_qid(d, gs, &s(m, "character")) == who && explore::presence(d, gs, &loc_qid, m) != "absent")
                .map(|m| explore::marker_pos(t, &m, steps))
                .filter(|at| at.0 >= 0)
                .min_by_key(|at| (at.0 - gs.position.0).abs() + (at.1 - gs.position.1).abs());
            if let Some(at) = near.filter(|at| !out.contains(at)) {
                out.push(at);
            }
        }
        out
    }
}

impl RegionMap {
    /// The marker whose legend row the pointer was over in the last frame (2.3: its marker shows its name).
    fn legend_hover(&self, live: bool) -> Option<String> {
        if !live {
            return None;
        }
        self.w.iter().find(|w| w.kind == "row" && w.rect.contains(self.mouse.0, self.mouse.1)).and_then(|w| w.id.strip_prefix("here.")).map(str::to_string)
    }

    /// The legend (spec PLAYUI1 2.1 A, MAPNAV1-U's collapse): a pane floating at the map's top right under the HUD bar.
    /// Its header "Here (N)" with a chevron collapses it to the header (kept in the settings as `map_legend_open`); a
    /// row per marker shows the marker's map icon, its name and (people) their service icons at the right; spent
    /// markers dim. A row click looks there (MAPNAV1-U: the camera pans to the marker); a second click while it looks
    /// there walks there as before. Width from the texts, at most `LEGEND_MAX` (times the text size). Its rect (the edge
    /// arrows keep off it).
    fn draw_legend(&mut self, ui: &mut Ui<RAct>, cx: &mut Cx, modal: bool) -> Rect {
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        let loc = explore::location(&d, &gs);
        // (id, kind, state, label, verb, services)
        let mut rows: Vec<(String, String, String, String, String, Vec<&'static str>)> = Vec::new();
        for m in explore::markers_of(loc) {
            let view = explore::marker_view(&d, &gs, loc, &m);
            let (kind, state) = (s(&m, "kind"), s(&view, "state"));
            if state == "absent" || kind == "point" {
                continue;
            }
            let services = if kind == "npc" { explore::npc_services(&d, &gs, &s(d.resolve(&gs.world_id, "characters", &s(&m, "character")), "_qid")) } else { Vec::new() };
            rows.push((s(&m, "id"), kind, state, s(&view, "label"), s(&view, "verb"), services));
        }
        let open = cx.settings.v["map_legend_open"].as_bool().unwrap_or(true);
        let rh = (ui.p.line_h(BODY).ceil() as i32 + 4).max(14);
        let title = format!("Here ({})", rows.len());
        let mut w = 14 + ui.p.text_w(&title, BODY).ceil() as i32 + 8;
        for (_, _, _, label, _, services) in &rows {
            w = w.max(4 + ICON + 4 + ui.p.text_w(label, BODY).ceil() as i32 + 6 + services.len() as i32 * (SERVICE + 1) + 4);
        }
        let w = w.min((LEGEND_MAX as f32 * ui.p.fonts.scale).round() as i32);
        let x = W - 4 - w;
        let head = Rect::new(x, HUD_H + 4, w, rh);
        ui.p.fill(head, th().game("map_pane", PANE));
        let t = tip(&title, if open { "Click to fold the legend to this line." } else { "Click to open the legend: who and what is here." });
        ui.section("legend", head, &title, "", open, RAct::Legend, t);
        if !open || rows.is_empty() {
            return head;
        }
        let top = head.y + head.h + 1;
        let content = rows.len() as i32 * rh;
        let pane = Rect::new(x, top, w, content.min(H - 4 - top));
        let whole = Rect::new(x, head.y, w, pane.y + pane.h - head.y);
        ui.p.fill(pane, th().game("map_pane", PANE));
        ui.p.panel(pane, rgba(0, 0, 0, 0), th().edge, 3.0);
        let off = ui.scroll_begin("legend.rows", pane, content);
        let px = ui.icon_px(Rect::new(0, 0, ICON + 4, ICON + 4));
        let spx = ui.icon_px(Rect::new(0, 0, SERVICE + 4, SERVICE + 4));
        for (i, (id, kind, state, label, verb, services)) in rows.into_iter().enumerate() {
            let r = Rect::new(x, top - off + i as i32 * rh, w, rh);
            let hover = !modal && ui.hover(r);
            if hover {
                ui.p.panel(r, th().card_hot, th().edge_hot, 3.0);
            }
            let ir = Rect::new(r.x + 4, r.y + (rh - ICON) / 2, ICON, ICON);
            match cx.art.image(&emberwake_content::art::marker(marker_icon(&kind, &state)), px, px) {
                Some(img) => ui.p.image(img, ir),
                None => ui.p.fill(Rect::new(ir.x + 2, ir.y + 2, ICON - 4, ICON - 4), Self::marker_style(&kind, &state).0),
            }
            let sw = services.len() as i32 * (SERVICE + 1);
            let tx = ir.x + ICON + 4;
            ui.p.text_fit(tx as f32, r.y as f32 + (rh as f32 - ui.p.line_h(BODY)) / 2.0, &label, BODY, (r.x + r.w - 4 - sw - tx) as f32, if state == "spent" { th().dim } else { th().text });
            let mut sx = r.x + r.w - 4 - sw;
            for sv in &services {
                let sr = Rect::new(sx, r.y + (rh - SERVICE) / 2, SERVICE, SERVICE);
                match cx.art.image(&service_icon(sv), spx, spx) {
                    Some(img) => ui.p.image(img, sr),
                    None => service_glyph(ui, sv, sr),
                }
                sx += SERVICE + 1;
            }
            let mut t = tip(&label, &verb);
            if !services.is_empty() {
                t.push((services_text(&services), th().info));
            }
            t.push(("Click to look there; click again to walk there.".to_string(), th().dim));
            ui.push(format!("here.{id}"), "row", label.clone(), r, true, false, (!modal).then(|| RAct::Pan(id.clone())), t);
        }
        ui.scroll_end();
        whole
    }

    /// The event log ticker (MAPLOG1-U, spec PLAYUI1 2.1 A): the latest event on one line at the map's bottom left; L
    /// or a click opens the last `LOG_OPEN` events (newest at the bottom), and again shows only the latest. Its rect
    /// (None with an empty log).
    fn draw_ticker(&mut self, ui: &mut Ui<RAct>, cx: &Cx, modal: bool) -> Option<Rect> {
        let Some(latest) = cx.log.last().cloned() else { return None };
        let lh = ui.p.line_h(SMALL).ceil() as i32 + 1;
        let max_w = ((TICKER_MAX as f32 * ui.p.fonts.scale).round() as i32).min(W - 8);
        let inner = max_w - 15 - 4;
        let lines: Vec<String> = if self.log_open {
            let room = ((H - HUD_H - 16) / lh).max(1) as usize;
            let mut out = Vec::new();
            for l in cx.log.iter().rev().take(LOG_OPEN) {
                let mut wrapped = ui.p.wrap(l, SMALL, inner as f32);
                wrapped.reverse();
                out.extend(wrapped);
            }
            out.truncate(room);
            out.reverse();
            out
        } else {
            vec![latest.clone()]
        };
        let text_w = lines.iter().map(|l| ui.p.text_w(l, SMALL).ceil() as i32).max().unwrap_or(0);
        let w = (15 + text_w + 4).min(max_w);
        let h = lines.len() as i32 * lh + 5;
        let r = Rect::new(4, H - 4 - h, w, h);
        let hover = !modal && ui.hover(r);
        ui.p.fill(r, th().game("map_pane", PANE));
        ui.p.panel(r, rgba(0, 0, 0, 0), if hover { th().gold } else { th().edge }, 3.0);
        let last_y = r.y + 3 + (lines.len() as i32 - 1) * lh;
        draw_glyph(&mut ui.p, if self.log_open { Glyph::ChevronDown } else { Glyph::ChevronRight }, Rect::new(r.x + 4, last_y + (lh - 8) / 2, 8, 8), if hover { th().gold } else { th().dim });
        for (i, l) in lines.iter().enumerate() {
            let ly = (r.y + 3 + i as i32 * lh) as f32;
            ui.p.text_fit((r.x + 15) as f32, ly, l, SMALL, inner as f32, if i + 1 == lines.len() { th().text } else { th().dim });
        }
        let mut t = tip("Event log (L)", if self.log_open { "Click or press L to show only the latest event." } else { "Click or press L for the last 12 events." });
        if !self.log_open && ui.p.text_w(&latest, SMALL) > inner as f32 {
            t.push((latest.clone(), th().text));
        }
        ui.push("log", "button", latest, r, !modal, self.log_open, Some(RAct::Log), t);
        Some(r)
    }
}

/// A marker for the sorted pass (PROPS1 4.4): its id, tile, the top of what is drawn there (a sprite's head, else the
/// tile; badges and names sit over it), kind and state; a person's art; the first service's badge (drawn with the
/// overlays).
struct MarkDraw {
    id: String,
    r: Rect,
    top: i32,
    kind: String,
    state: String,
    npc: Option<NpcDraw>,
    badge: Option<&'static str>,
}

/// A marker's click area and name, taken before the sorted pass (EXPLOREVIEW1 3.5): its id, label, the area, its feet,
/// the tip, whether a name already shows, where the name sits and whether it sits over a badge.
struct MarkHit {
    id: String,
    label: String,
    hit: Rect,
    foot: i32,
    tip: Vec<(String, Rgba)>,
    named: bool,
    name_rect: Rect,
    badge: bool,
}

/// The tilted view's click order, back to front: the draw order (`props::sorted`) by feet, then left edge.
fn sort_hits(hits: &mut [MarkHit]) {
    hits.sort_by_key(|h| (h.foot, h.hit.x));
}

/// The click area drawn in front among those `over` accepts (the last of `sort_hits`' order).
fn front_hit(hits: &[MarkHit], over: impl Fn(Rect) -> bool) -> Option<&MarkHit> {
    hits.iter().filter(|h| over(h.hit)).max_by_key(|h| (h.foot, h.hit.x))
}

/// The markers' click areas, in the order given.
fn push_hits(ui: &mut Ui<RAct>, hits: Vec<MarkHit>) {
    for h in hits {
        ui.push(format!("marker.{}", h.id), "marker", h.label, h.hit, true, false, Some(RAct::Marker(h.id)), h.tip);
    }
}

/// The other region view, saved to the settings like the Scene Runner's select (EXPLOREVIEW1 Q2, the debug build's F7).
fn flip_view(cx: &mut Cx) {
    let to = mapview::View::of_setting(&cx.settings.v["region_view"]).flipped();
    cx.settings.v["region_view"] = json!(to.name());
    if let Err(e) = cx.settings.save() {
        show_errors(cx, vec![format!("Settings could not be saved: {e}")]);
    }
}

/// A person's art on the map (PLAYUI1-R): the body key, the first of its chain that loads (None: today's token), the
/// palette, the initial of the token stand-in, the facing and the person's idle offset (`idle_offset`).
struct NpcDraw {
    body: String,
    sprite: Option<String>,
    palette: String,
    initial: String,
    face: Facing,
    offset: u64,
}

/// A person's stable idle offset (ms, under `IDLE_SPREAD_MS`): FNV-1a of their character id, so the people on a map do
/// not idle in step and each always idles the same way (no random draw, no clock).
fn idle_offset(id: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in id.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h % IDLE_SPREAD_MS
}

/// The range of the idle offsets (longer than an idle loop, so any phase is possible).
const IDLE_SPREAD_MS: u64 = 4096;

/// A sprite as drawn (PLAYUI1-R): the body, the clip and frame shown, mirrored or not, the drawn rect and the pixel
/// size the frame was fitted to; the facing asked for and the view of the row shown (DIR4-M: a side row when the body
/// has no up or down row for the clip).
struct SpriteDrawn {
    body: String,
    clip: String,
    frame: u32,
    flip: bool,
    rect: Rect,
    px: i32,
    face: Facing,
    view: View,
}

impl SpriteDrawn {
    /// The agent link's line for it (`state.sprites`): `facing` the view asked for, `view` the row drawn (`side`,
    /// `down`, `up`).
    fn info(&self, id: &str) -> Value {
        json!({"id": id, "body": self.body, "clip": self.clip, "frame": self.frame, "left": self.flip, "facing": view_name(self.face.view), "view": view_name(self.view), "rect": [self.rect.x, self.rect.y, self.rect.w, self.rect.h]})
    }
}

/// A view's name on the agent link (`side`, `down`, `up`).
fn view_name(v: View) -> &'static str {
    match v {
        View::Side => "side",
        other => other.name(),
    }
}

/// MAPNAV1-U: an edge arrow's hit box and drawn disc (logical px).
const ARROW: i32 = 14;

/// MAPNAV1-U: where the edge arrow toward `target` (a point out of `area`) sits and the way it points: on the ray from
/// the area's centre to the target, where it meets the area inset by `ARROW` (x, y, unit dx, unit dy). None when the
/// target is the centre.
fn edge_arrow(area: Rect, target: (f32, f32)) -> Option<(f32, f32, f32, f32)> {
    let (cx, cy) = (area.x as f32 + area.w as f32 / 2.0, area.y as f32 + area.h as f32 / 2.0);
    let (dx, dy) = (target.0 - cx, target.1 - cy);
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.5 {
        return None;
    }
    let (hw, hh) = (area.w as f32 / 2.0 - ARROW as f32, area.h as f32 / 2.0 - ARROW as f32);
    let sx = if dx.abs() > 1e-3 { hw / dx.abs() } else { f32::INFINITY };
    let sy = if dy.abs() > 1e-3 { hh / dy.abs() } else { f32::INFINITY };
    let k = sx.min(sy).min(1.0);
    Some((cx + dx * k, cy + dy * k, dx / len, dy / len))
}

/// Whether an arrow at `p` meets the pane `r` (with 2 px to spare).
fn arrow_meets(p: (f32, f32), r: &Rect) -> bool {
    let half = ARROW as f32 / 2.0;
    p.0 + half > r.x as f32 - 2.0 && p.0 - half < (r.x + r.w) as f32 + 2.0 && p.1 + half > r.y as f32 - 2.0 && p.1 - half < (r.y + r.h) as f32 + 2.0
}

/// Whether an arrow at `p` is in the arrows' band of `area`.
fn arrow_inside(area: Rect, p: (f32, f32)) -> bool {
    p.0 >= (area.x + ARROW) as f32 - 0.5 && p.0 <= (area.x + area.w - ARROW) as f32 + 0.5 && p.1 >= (area.y + ARROW) as f32 - 0.5 && p.1 <= (area.y + area.h - ARROW) as f32 + 0.5
}

/// REGIONVIEW1: an edge arrow at `at` moved along the edge it sits on, to the nearest place clear of the arrows already
/// `placed` and of the panes (else where it was).
fn beside_arrows(area: Rect, at: (f32, f32), placed: &[(f32, f32)], panes: &[Rect]) -> (f32, f32) {
    let gap = ARROW as f32 + 2.0;
    let free = |p: (f32, f32)| placed.iter().all(|o| (p.0 - o.0).abs() >= gap || (p.1 - o.1).abs() >= gap) && !panes.iter().any(|r| arrow_meets(p, r));
    if placed.iter().all(|o| (at.0 - o.0).abs() >= gap || (at.1 - o.1).abs() >= gap) {
        return at;
    }
    let on_side = at.0 <= (area.x + ARROW) as f32 + 0.5 || at.0 >= (area.x + area.w - ARROW) as f32 - 0.5;
    (1..=(placed.len() as i32 + 1))
        .flat_map(|k| [k, -k])
        .map(|k| if on_side { (at.0, at.1 + k as f32 * gap) } else { (at.0 + k as f32 * gap, at.1) })
        .find(|p| arrow_inside(area, *p) && free(*p))
        .unwrap_or(at)
}

/// MAPNAV1-U: an edge arrow at `at` moved off the floating panes (the legend, the ticker): to the nearest place beside
/// the pane it meets that stays in the arrows' band of `area` and clear of every pane (else where it was).
fn clear_of(area: Rect, at: (f32, f32), panes: &[Rect]) -> (f32, f32) {
    let half = ARROW as f32 / 2.0;
    let pad = half + 2.0;
    let meets = |p: (f32, f32), r: &Rect| arrow_meets(p, r);
    let inside = |p: (f32, f32)| arrow_inside(area, p);
    let Some(r) = panes.iter().find(|r| meets(at, r)) else { return at };
    let options = [(r.x as f32 - pad, at.1), ((r.x + r.w) as f32 + pad, at.1), (at.0, r.y as f32 - pad), (at.0, (r.y + r.h) as f32 + pad)];
    let dist = |p: &(f32, f32)| (p.0 - at.0).abs() + (p.1 - at.1).abs();
    options.into_iter().filter(|p| inside(*p) && !panes.iter().any(|r| meets(*p, r))).min_by(|a, b| dist(a).total_cmp(&dist(b))).unwrap_or(at)
}

/// MAPNAV1-U: whether a member can lead on the maps in world `world`: not downed (HP lost below the member's max HP;
/// benched and departed members are not active, so `leader_choices` never sees them).
pub(crate) fn standing<'a>(d: &'a emberwake_world::Data, world: &str) -> impl Fn(&emberwake_world::state::UnitState) -> bool + 'a {
    let world = world.to_string();
    move |m| m.hp_lost <= 0 || m.hp_lost < emberwake_world::progression::max_hp_of(d, m, &world)
}

/// MAPNAV1-U: the tile of marker `id` of the location (None when it has none here or no place on the map).
fn marker_tile(cx: &Cx, id: &str) -> Option<Pos> {
    let gs = cx.gs();
    let loc = explore::location(&cx.data, gs);
    let t = explore::template_of(&cx.data, loc);
    let m = explore::markers_of(loc).into_iter().find(|m| s(m, "id") == id)?;
    Some(explore::marker_pos(t, &m, gs.explore_int("loc_steps"))).filter(|at| at.0 >= 0)
}

/// MAPNAV1-U: the id of the member who leads on the maps (`GameState::explore_leader`; "" without a party).
pub(crate) fn leader_id(cx: &Cx) -> String {
    let gs = cx.gs();
    gs.explore_leader(standing(&cx.data, &gs.world_id)).map(|m| m.id.clone()).unwrap_or_default()
}

/// The party leader's body key and palette (spec PLAYUI1 2.4): the leader's (MAPNAV1-U, `leader_id`) character art
/// (its base unit's, its class's when it has no character), as the battle draws the member.
fn leader_art(cx: &Cx) -> (String, String) {
    let d = &cx.data;
    let gs = cx.gs();
    let Some(m) = gs.party.member(&leader_id(cx)) else { return (String::new(), String::new()) };
    let character = d.resolve(&gs.world_id, "characters", &m.lore_ref);
    if character.is_empty() {
        let unit = d.entry("units", &m.base_unit);
        let class = d.entry("classes", &s(unit, "class_id"));
        (emberwake_content::art::body_of(&Default::default(), unit, class), emberwake_content::art::palette_of(&Default::default(), unit))
    } else {
        let (body, palette, _) = crate::screens::character_art(d, character);
        (body, palette)
    }
}

/// Body `body` standing on tile `r` (spec PLAYUI1 2.4): a foot shadow in `foot`, then the frame `player` plays at
/// `now` in `face` (`Art::played_facing`, DIR4-M: the up or down row when the body has one, else the side row,
/// mirrored facing left), filling its rect at the zoomed size (SPRITEFIT1's exact fit). None when it does not play (the
/// caller draws today's token).
#[allow(clippy::too_many_arguments)]
fn draw_sprite(ui: &mut Ui<RAct>, cx: &mut Cx, body: &str, palette: &str, player: &ClipPlayer, now: u64, face: Facing, r: Rect, foot: Rgba) -> Option<SpriteDrawn> {
    let sr = mapview::sprite_rect(r);
    let px = (sr.w as f32 * ui.p.v.s).round().max(1.0) as i32;
    let (f, img) = cx.art.played_facing(body, palette, player, now, px, face)?;
    let flip = face.mirrors(&f);
    let feet = (r.y + r.h) as f32 - r.w as f32 / 8.0;
    ui.p.ellipse(r.x as f32 + r.w as f32 / 2.0, feet, r.w as f32 * 0.4, r.h as f32 * 0.14, rgba(foot[0], foot[1], foot[2], FOOT_ALPHA));
    ui.p.image(img, sr);
    let view = if f.side { View::Side } else { face.view };
    Some(SpriteDrawn { body: body.to_string(), clip: f.clip, frame: f.frame, flip, rect: sr, px, face, view })
}

/// A marker on its tile: a person's sprite (PLAYUI1-R; their ring and token, or initial, without a body), else the
/// marker icon (or its drawn stand-in), standing on the cell (EXPLOREVIEW1 3.5). The person's sprite as drawn.
fn draw_marker(ui: &mut Ui<RAct>, cx: &mut Cx, m: &MarkDraw, ts: i32, idle: &ClipPlayer, now: u64) -> Option<SpriteDrawn> {
    let r = mapview::icon_rect(m.r);
    let (cx_, cy_) = (r.x as f32 + ts as f32 / 2.0, r.y as f32 + ts as f32 / 2.0);
    if let Some(n) = &m.npc {
        let (col, _) = RegionMap::marker_style(&m.kind, &m.state);
        if let Some(sp) = n.sprite.as_deref().and_then(|b| draw_sprite(ui, cx, b, &n.palette, idle, now + n.offset, n.face, m.r, col)) {
            return Some(sp);
        }
        ui.p.circle(cx_, cy_, ts as f32 * 0.42, col);
        let tok = Rect::new(r.x + ts / 8, r.y + ts / 8, ts - ts / 4, ts - ts / 4);
        match cx.art.first_token(&emberwake_content::art::body_chain(&n.body), &n.palette, tok.w) {
            Some(img) => ui.p.image(img, tok),
            None => ui.p.text_center(cx_, cy_ - 4.0, &n.initial, BODY, rgb(20, 20, 20)),
        }
        return None;
    }
    let px = ui.icon_px(Rect::new(0, 0, ts + 4, ts + 4));
    match cx.art.image(&emberwake_content::art::marker(marker_icon(&m.kind, &m.state)), px, px) {
        Some(img) => ui.p.image(img, r),
        None => {
            let (col, glyph) = RegionMap::marker_style(&m.kind, &m.state);
            ui.p.panel(r.inset(ts / 5), col, rgba(255, 255, 255, 90), 2.0);
            ui.p.text_center(cx_, cy_ - 4.5, glyph, BODY, rgb(16, 16, 20));
        }
    }
    None
}

/// The foot shadow's alpha under a sprite.
const FOOT_ALPHA: u8 = 150;

/// A prop, cut at the map's rect (REGIONVIEW1: only boundary art past the map). The clip is physical px under the
/// frame's view, so the cut moves with the map.
fn draw_prop(ui: &mut Ui<RAct>, cx: &mut Cx, p: &PropDraw, map: Rect, q: i32) {
    let leaves = p.rect.intersect(map) != p.rect;
    let kept = ui.p.c.clip;
    if leaves {
        let inside = ui.p.v.rect(map);
        ui.p.c.clip = Some(kept.map_or(inside, |c| c.intersect(inside)));
    }
    draw_prop_art(ui, cx, p, q);
    ui.p.c.clip = kept;
}

/// A prop (PROPS1 4.3, 4.6): its art in its drawn rect (mirrored when flipped), one source pixel per screen pixel;
/// without art a standing prop is a plain block over its footprint (fill, a darker south face, a 1 px edge; tilted,
/// raised over the footprint with a face its lift tall, EXPLOREVIEW1 3.4) and a ground prop is nothing. `q`: the zoom.
fn draw_prop_art(ui: &mut Ui<RAct>, cx: &mut Cx, p: &PropDraw, q: i32) {
    let (w, h) = ((p.rect.w as f32 * ui.p.v.s).round().max(1.0) as i32, (p.rect.h as f32 * ui.p.v.s).round().max(1.0) as i32);
    let img = match &p.file {
        Some(f) if p.flip => cx.art.image_mirrored(f, w, h),
        Some(f) => cx.art.image(f, w, h),
        None => None,
    };
    if let Some(img) = img {
        ui.p.image(img, p.rect);
        return;
    }
    if p.ground {
        return;
    }
    let r = p.rect;
    ui.p.fill(r, BOX_FILL);
    let face = if p.file.is_none() && r.y < p.foot.y { p.foot.y - r.y } else { (r.h / 4).clamp(1, BOX_FACE_H * q / 4) };
    ui.p.fill(Rect::new(r.x, r.y + r.h - face, r.w, face), BOX_FACE);
    let e = (q / 4).max(1);
    for edge in [Rect::new(r.x, r.y, r.w, e), Rect::new(r.x, r.y + r.h - e, r.w, e), Rect::new(r.x, r.y, e, r.h), Rect::new(r.x + r.w - e, r.y, e, r.h)] {
        ui.p.fill(edge, BOX_EDGE);
    }
}

/// A wall without art.
const WALL_FILL: Rgba = rgb(40, 60, 50);

/// EXPLOREVIEW1 3.2: a water cell sunken by `LIP_DEPTH`: a band over its top when the cell north is not water, its
/// bottom row shaded when the cell south is not.
fn draw_lips(ui: &mut Ui<RAct>, r: Rect, north: bool, south: bool) {
    let (line, lip) = (mapview::line_of(r.h), mapview::lip_of(r.h));
    if north {
        ui.p.fill(Rect::new(r.x, r.y, r.w, line), rgb(118, 116, 62));
        ui.p.fill(Rect::new(r.x, r.y + line, r.w, lip), rgb(66, 54, 30));
        ui.p.fill(Rect::new(r.x, r.y + line + lip, r.w, line), rgba(60, 40, 26, 110));
    }
    if south {
        ui.p.fill(Rect::new(r.x, r.y + r.h - line, r.w, line), rgba(48, 34, 22, 120));
    }
}

/// REGIONVIEW1: whether (x, y) is the map's border: past the map, or a wall on its outer ring (shipped maps ring themselves
/// with `#`, which reads as one with what lies past it). Display only: `rows` still decide walking.
fn on_border(t: &Dict, (w, h): Pos, x: i64, y: i64) -> bool {
    match explore::tile(t, x, y) {
        None => true,
        Some(c) => c == '#' && (x == 0 || y == 0 || x == w - 1 || y == h - 1),
    }
}

/// The folders a frame's walls draw from (REGIONGROUND1_SCENE: the location's `boundary` art for its border, else the
/// walls' own): the region sets, and the boundary's `wall` and `wall_face` folders over them.
struct WallArt<'a> {
    names: &'a [String],
    sets: Vec<&'a str>,
    boundary_wall: Vec<String>,
    boundary_face: Vec<String>,
}

impl<'a> WallArt<'a> {
    fn of(names: &'a [String], boundary: &str) -> Self {
        let dirs = |tile: &str| if boundary.is_empty() { Vec::new() } else { emberwake_content::art::boundary_dirs(names, boundary, tile) };
        WallArt { names, sets: names.iter().map(String::as_str).collect(), boundary_wall: dirs("wall"), boundary_face: dirs("wall_face") }
    }

    /// The face's folders: the boundary's first on the border, the region sets', then the battle's stage set.
    fn face_dirs(&self, border: bool) -> Vec<String> {
        let own = self.names.iter().map(|set| format!("tilesets/{set}/wall_face"));
        let first = self.boundary_face.iter().filter(|_| border).cloned();
        first.chain(own).chain(emberwake_content::art::battle_stage_dirs("wall_face", "")).collect()
    }
}

/// A wall's south face on its cell (EXPLOREVIEW1 3.2): `wall_face` along `dirs`, else the cap's top rows at 75 %, with its
/// top line and the shade on the cell south.
fn draw_face(ui: &mut Ui<RAct>, cx: &mut Cx, b: &Block, sets: &[&str], dirs: &[String]) {
    let face = mapview::face_rect(b.cell);
    let wall = emberwake_content::art::region_tile('#').unwrap_or("wall");
    let dirs: Vec<&str> = dirs.iter().map(String::as_str).collect();
    match cx.art.varied(&dirs, None, b.x, b.y, face.w, face.h) {
        Some(img) => ui.p.image(img, face),
        None => match darkened_cap(cx, sets, wall, b) {
            Some(img) => ui.p.image(&img, face),
            None => ui.p.fill(face, rgb(30, 45, 38)),
        },
    }
    ui.p.fill(Rect::new(face.x, face.y, face.w, mapview::line_of(b.cell.h)), rgb(24, 24, 30));
    ui.p.fill(Rect::new(b.cell.x, b.cell.y + b.cell.h, b.cell.w, mapview::shade_of(b.cell.h)), rgba(0, 0, 0, 70));
}

/// REGIONVIEW1: a cell of the border, flat on its cell (no lift, so it stays out of the map): the boundary's `wall` art, else
/// the wall tile, else a plain fill; tilted, its south face when the cell south is open.
#[allow(clippy::too_many_arguments)]
fn draw_border(ui: &mut Ui<RAct>, cx: &mut Cx, walls: &WallArt, x: i32, y: i32, r: Rect, size: i32, face: bool) {
    let dirs: Vec<&str> = walls.boundary_wall.iter().map(String::as_str).collect();
    let wall = emberwake_content::art::region_tile('#').unwrap_or("wall");
    let own = cx.art.pick(&dirs, None, x, y);
    let img = match own {
        Some(rel) => cx.art.image(&rel, size, size),
        None => cx.art.tile_in(&walls.sets, wall, x, y, size),
    };
    match img {
        Some(img) => ui.p.image(img, r),
        None => ui.p.fill(r, WALL_FILL),
    }
    if face {
        let b = Block { x, y, cell: r, rect: r, face };
        draw_face(ui, cx, &b, &walls.sets, &walls.face_dirs(true));
    }
}

/// EXPLOREVIEW1 3.2: a wall block: its cap (the wall tile, squashed and lifted), its south face when the cell south is
/// open with its top line and shade (`draw_face`), the cap's highlight and side lines where the neighbour is not a wall.
/// REGIONVIEW1: over a flat border south, the cell's own rows that the lifted cap leaves open are the wall too.
fn draw_block(ui: &mut Ui<RAct>, cx: &mut Cx, b: &Block, walls: &WallArt, ch_at: &impl Fn(i32, i32) -> char, border_at: &impl Fn(i32, i32) -> bool) {
    let cap = mapview::cap_rect(b.cell);
    let wall = emberwake_content::art::region_tile('#').unwrap_or("wall");
    let sets = walls.sets.as_slice();
    let (size, line) = (ART_TILE * b.cell.w / mapview::CELL_W, mapview::line_of(b.cell.h));
    if border_at(b.x, b.y + 1) {
        match cx.art.tile_in(sets, wall, b.x, b.y, size) {
            Some(img) => ui.p.image(img, b.cell),
            None => ui.p.fill(b.cell, WALL_FILL),
        }
    }
    match cx.art.tile_in(sets, wall, b.x, b.y, size) {
        Some(img) => ui.p.image(img, cap),
        None => ui.p.fill(cap, WALL_FILL),
    }
    if b.face {
        draw_face(ui, cx, b, sets, &walls.face_dirs(false));
    }
    if ch_at(b.x, b.y - 1) != '#' {
        ui.p.fill(Rect::new(cap.x, cap.y, cap.w, line), rgb(170, 172, 186));
    }
    let side = rgb(30, 30, 38);
    if ch_at(b.x - 1, b.y) != '#' {
        ui.p.fill(Rect::new(cap.x, cap.y, line, b.rect.h), side);
    }
    if ch_at(b.x + 1, b.y) != '#' {
        ui.p.fill(Rect::new(cap.x + cap.w - line, cap.y, line, b.rect.h), side);
    }
}

/// The wall face's fallback: the cap's top rows, as many as the face is tall at this zoom, at 75 % brightness (None: no wall
/// art).
fn darkened_cap(cx: &mut Cx, sets: &[&str], wall: &str, b: &Block) -> Option<aegis_ui::canvas::Image> {
    let dirs: Vec<String> = sets.iter().map(|set| format!("tilesets/{set}/{wall}")).collect();
    let dirs: Vec<&str> = dirs.iter().map(String::as_str).collect();
    let img = cx.art.varied(&dirs, None, b.x, b.y, b.cell.w, b.cell.h)?;
    let mut top = img.crop(0, 0, img.w, mapview::lift_of(b.cell.h));
    for (k, v) in top.px.iter_mut().enumerate() {
        if k % 4 != 3 {
            *v = (*v as f32 * 0.75).round() as u8;
        }
    }
    Some(top)
}

/// The legend pane's and the ticker's fill.
const PANE: Rgba = rgba(12, 18, 28, 235);
/// The legend's widest at text size Normal (spec PLAYUI1 2.1 A), and the ticker's.
const LEGEND_MAX: i32 = 170;
const TICKER_MAX: i32 = 360;
/// The events the opened ticker shows (MAPLOG1-U).
pub const LOG_OPEN: usize = 12;
/// A legend row's marker icon and a service icon (logical px).
const ICON: i32 = 12;
const SERVICE: i32 = 10;

/// The marker's icon in `map/markers/` (`emberwake_content::art::marker`; the MARKERS1 idea may move it to `ui/markers/`).
fn marker_icon(kind: &str, state: &str) -> &'static str {
    match (kind, state) {
        ("exit", "locked") => "exit_locked",
        ("exit", _) => "exit",
        ("chest", "spent") => "chest_open",
        ("chest", _) => "chest",
        ("lore", _) => "lore",
        ("encounter", _) => "encounter",
        ("roamer", _) => "roamer",
        ("npc", _) => "npc",
        ("point", _) => "poi",
        _ => "objective",
    }
}

/// A service's icon (spec PLAYUI1 section 6): `turn_in` and `offer` are the quest badges `ready` and `quest`; a board
/// with work (STORYUI1 B4) is the quest badge too.
pub(crate) fn service_icon(service: &str) -> String {
    match service {
        "turn_in" => emberwake_content::art::badge("ready"),
        "offer" | "board" => emberwake_content::art::badge("quest"),
        _ => format!("ui/services/{service}.png"),
    }
}

/// "Quest to turn in, Shop" for a hover card.
pub(crate) fn services_text(services: &[&str]) -> String {
    services
        .iter()
        .map(|sv| match *sv {
            "turn_in" => "Quest to turn in",
            "offer" => "New quest",
            "board" => emberwake_world::board::BADGE_TEXT,
            "shop" => "Shop",
            "recruit" => "Recruits",
            _ => "",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The drawn stand-in of a service icon: today's `?` and `!` badges (a board with work is the `!`, STORYUI1 B4), a coin
/// for a shop, a plus for recruits.
pub(crate) fn service_glyph<A>(ui: &mut Ui<A>, service: &str, r: Rect) {
    let (cx_, cy_, sz) = (r.x as f32 + r.w as f32 / 2.0, r.y as f32 + r.h as f32 / 2.0, r.w as f32);
    match service {
        "turn_in" | "offer" | "board" => {
            let (text, col) = if service == "turn_in" { ("?", th().good) } else { ("!", th().gold) };
            let size = SMALL.max(r.h as f32 * 0.9 / 1.28);
            ui.p.text_center(cx_, cy_ - ui.p.line_h(size) / 2.0, text, size, col);
        }
        "shop" => {
            ui.p.circle(cx_, cy_, sz * 0.45, rgb(150, 110, 30));
            ui.p.circle(cx_, cy_, sz * 0.33, th().gold);
        }
        _ => {
            ui.p.line(cx_ - sz * 0.4, cy_, cx_ + sz * 0.4, cy_, th().good, 0.0);
            ui.p.line(cx_, cy_ - sz * 0.4, cx_, cy_ + sz * 0.4, th().good, 0.0);
        }
    }
}

/// The talk box over the map as `Services::draw` draws it (the tests draw fixture views).
#[cfg(test)]
fn draw_talk(ui: &mut Ui<RAct>, cx: &mut Cx, faces: &mut Faces, view: &Dict, b: &crate::talk::TalkBox, busy: bool) {
    let gs = cx.gs().clone();
    crate::talk::draw_talk(ui, cx, &gs, faces, view, b, busy, crate::talk::svc_talk_act::<RAct>);
}

/// The region map's controls (the Help button's hover card and popup).
pub const REGION_HELP: [&str; 9] = [
    "Arrows or WASD: move",
    "E or Enter: use what the party faces",
    "Click: walk there, or on a marker to use it",
    "Legend row or edge arrow: look there (again: walk there)",
    "Q or a click on the leader: the next member leads",
    "1-9: pick a reply in a conversation",
    "P: party   J: journal   I: stash",
    "L: the last events, or only the latest",
    "Esc: back, or the game menu",
];

#[cfg(test)]
mod tests {
    use aegis_core::assets::Layers;
    use std::path::Path;

    /// ASSETORG2 section 3: every file under each asset root (the game's, its kit's, the engine's) matches a pattern of
    /// the layered tree (README.md files aside), and the region map's tiles of the Ashlands are there.
    #[test]
    fn the_assets_follow_the_layered_tree() {
        let game = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let layers = Layers::for_game(&game);
        assert_eq!(layers.roots.len(), 3, "game, kit and engine roots: {:?}", layers.notices);
        let mut stray = Vec::new();
        for root in &layers.roots {
            stray.extend(strays(root));
        }
        assert!(stray.is_empty(), "files outside the layered tree (docs/specs/ASSETORG2.md section 3): {stray:?}");
        for t in ["floor", "wall", "shallow_water"] {
            assert_eq!(layers.variants(&format!("tilesets/ashlands/{t}")), 3, "{t}");
        }
    }

    fn strays(root: &Path) -> Vec<String> {
        let mut stray = Vec::new();
        let mut walk = vec![root.to_path_buf()];
        while let Some(dir) = walk.pop() {
            for e in std::fs::read_dir(&dir).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk.push(p);
                    continue;
                }
                let rel = p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
                if !layout_ok(&rel) {
                    stray.push(format!("{}: {rel}", root.display()));
                }
            }
        }
        stray
    }

    /// REGIONGROUND1: a ground's one texture and a boundary's tiles are in the tree; a ground's other file is a stray.
    #[test]
    fn ground_and_boundary_art_follow_their_patterns() {
        for ok in ["tilesets/ashlands/ground/mud/v0.png", "tilesets/common/boundary/keep/wall/v0.png", "tilesets/common/boundary/keep/wall_face/v2.png"] {
            assert!(layout_ok(ok), "{ok}");
        }
        for stray in ["tilesets/ashlands/ground/mud/v1.png", "tilesets/ashlands/ground/mud.png", "tilesets/ashlands/ground/mud/over_ash/m01.png", "tilesets/common/boundary/keep/roof/v0.png", "tilesets/common/boundary/keep/wall/face.png"] {
            assert!(!layout_ok(stray), "{stray}");
        }
    }

    /// PROPS1 3.1: map prop art is `props/<set>/<art>/v<n>.png` (a variant run, as tile sets); anything else under
    /// `props/` is a stray.
    #[test]
    fn prop_art_follows_the_variant_pattern() {
        for ok in ["props/common/tent/v0.png", "props/ashlands/keep_hall/v12.png"] {
            assert!(layout_ok(ok), "{ok}");
        }
        for stray in ["props/common/tent/tent.png", "props/common/tent/v0.jpg", "props/tent/v0.png", "props/common/tent/v0/a.png", "props/common/tent/vx.png"] {
            assert!(!layout_ok(stray), "{stray}");
        }
    }

    /// PORTRAITFX1 4.1 and 5.4: a portrait's face sidecar `<id>.face.json` sits beside it (any portrait folder);
    /// expression frames `faces/<id>/<expression>.png` (and their sidecars) only under the unique folders, named by
    /// the game's faces (PORTRAITFX2 FL4: `Catalog::faces`).
    #[test]
    fn portrait_sidecars_and_expression_frames_follow_the_layout() {
        for ok in ["portraits/npcs/tess.png", "portraits/npcs/tess.face.json", "portraits/common/beast_render_beast.face.json", "portraits/heroes/faces/kaelen/hurt.png", "portraits/bosses/faces/proctor_quell/neutral.face.json"] {
            assert!(layout_ok(ok), "{ok}");
        }
        for stray in ["portraits/npcs/tess.json", "portraits/heroes/faces/kaelen/smirk.png", "portraits/common/faces/beast/hurt.png", "portraits/npcs/faces/tess.png", "portraits/npcs/faces/tess/hurt/a.png", "portraits/npcs/faces/tess/hurt.jpg"] {
            assert!(!layout_ok(stray), "{stray}");
        }
    }

    /// The shipped data's faces (PORTRAITFX2 FL1), read once.
    fn shipped_faces() -> &'static [String] {
        static FACES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
        FACES.get_or_init(|| {
            let dir = emberwake_world::Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
            emberwake_world::Data::open(&dir).expect("data").0.cat.faces()
        })
    }

    /// One path of the section 3 tree (game, kit or engine root), plus the PLAYUI1 section 6 icons: toolkit glyphs
    /// `ui/<id>.png` and the `ui/hud`, `ui/markers` and `ui/services` folders, PROPS1's `props/<set>/<art>/`, the
    /// story scene pictures `backgrounds/story/<id>.png` (STORYART1, named by a vignette card's `image`) and the
    /// portrait sidecars and expression frames (PORTRAITFX1).
    fn layout_ok(rel: &str) -> bool {
        let p: Vec<&str> = rel.split('/').collect();
        let png = |s: &str| s.ends_with(".png");
        let face = |s: &str| png(s) || s.ends_with(".face.json");
        let expression = |s: &str| shipped_faces().iter().any(|e| s == format!("{e}.png") || s == format!("{e}.face.json"));
        let v = |s: &str| s.starts_with('v') && s.trim_start_matches('v').trim_end_matches(".png").parse::<u32>().is_ok() && png(s);
        if p.last() == Some(&"README.md") || matches!(p.as_slice(), ["fonts", ..]) {
            return true;
        }
        match p.as_slice() {
            ["units", _, _, "sheet.png" | "clips.json"] => true,
            ["units", _, _, "palettes", f] => f.ends_with(".json"),
            ["portraits", "common" | "heroes" | "npcs" | "elites" | "bosses", f] => face(f),
            ["portraits", "heroes" | "npcs" | "elites" | "bosses", "faces", _, f] => expression(f),
            // FB-140 (ASSETORG2 section 3): full-body story stage sprites
            ["portraits", "full_body", f] => png(f),
            ["items", _, f] => png(f),
            ["skills", "kinds", f] | ["skills", f] | ["statuses", f] | ["map", "markers", f] => png(f),
            // LOOK1-G1: the theme file and its nine-slice frames (`<role>.png` with a `<role>.json` sidecar)
            ["ui", "theme.json"] => true,
            ["ui", "frames", f] => png(f) || f.ends_with(".json"),
            ["ui", "pins" | "badges" | "chips" | "intents" | "hud" | "markers" | "services", f] | ["ui", f] => png(f),
            ["tilesets", _, _, f] | ["props", _, _, f] => v(f),
            // REGIONGROUND1: a ground's one texture (only `v0` is read), and a boundary's tiles
            ["tilesets", _, "ground", _, "v0.png"] => true,
            ["tilesets", _, "boundary", _, "wall" | "wall_face", f] => v(f),
            ["backgrounds", "menu" | "title" | "world_map" | "location" | "battle" | "story", f] => png(f),
            ["audio", "sfx", f] => f.ends_with(".wav"),
            ["audio", "music", f] => f.ends_with(".ogg"),
            _ => false,
        }
    }
}

/// PLAYUI1-M (spec PLAYUI1 2.1-2.3, 3.2): the full-width region map, its legend, the event log ticker, names on hover
/// and the centred camera, drawn headless over a debug game of the first playable world (the art layers are off, so
/// tiles are flat fills and icons their drawn stand-ins).
#[cfg(test)]
mod map_tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx, rect};
    use aegis_ui::painter::TextRec;
    use aegis_ui::screen::{Settings, TEXT_SIZES};

    /// One frame with the mouse at `mouse` (text size Normal); the texts drawn.
    fn frame_at(map: &mut RegionMap, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32)) -> Vec<TextRec> {
        g.fonts.scale = 1.0;
        g.fonts.audit = Some(Vec::new());
        map.draw(cx, g, mouse, true);
        g.fonts.audit.take().unwrap_or_default()
    }

    fn widget(map: &RegionMap, id: &str) -> Option<Value> {
        map.widgets().into_iter().find(|w| w["id"] == id)
    }

    fn ids_with(map: &RegionMap, prefix: &str) -> Vec<String> {
        map.widgets().iter().map(|w| s(json::dict(w), "id")).filter(|id| id.starts_with(prefix)).collect()
    }

    fn area() -> Rect {
        Rect::new(0, HUD_H, W, H - HUD_H)
    }

    /// The markers the legend lists, in order: present (or spent or locked) and not a bare point.
    fn listed(cx: &Cx) -> Vec<Dict> {
        let gs = cx.gs();
        let loc = explore::location(&cx.data, gs);
        explore::markers_of(loc)
            .into_iter()
            .filter(|m| s(&explore::marker_view(&cx.data, gs, loc, m), "state") != "absent" && s(m, "kind") != "point")
            .collect()
    }

    /// A text drawn as `marker`'s name: `label` just above the marker's tile (shifted sideways at most to stay inside).
    fn named_above(recs: &[TextRec], label: &str, marker: (i64, i64, i64, i64)) -> bool {
        let (mx, my, mw, _) = marker;
        recs.iter().any(|r| r.text == label && r.y + r.h <= my as f32 + 2.0 && r.y >= my as f32 - 24.0 && (r.x + r.w / 2.0 - (mx + mw / 2) as f32).abs() <= r.w / 2.0 + CELL_W as f32)
    }

    #[test]
    fn the_map_takes_the_full_width_under_the_bar() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let m = widget(&map, "map").expect("the map");
        assert_eq!(rect(&m), (0, HUD_H as i64, W as i64, (H - HUD_H) as i64));
        // a click on an open tile next to the party walks there (the click maps through the centred camera)
        let gs = cx.gs().clone();
        let t = explore::template_of(&cx.data, explore::location(&cx.data, &gs));
        let open = explore::DIRS.into_iter().map(|d| (gs.position.0 + d.0, gs.position.1 + d.1)).find(|p| explore::is_open(t, p.0, p.1));
        if let Some(p) = open {
            let r = map.geo.cell_of(p);
            let at = (r.x + r.w / 2, r.y + r.h / 2);
            frame_at(&mut map, &mut cx, &mut g, at);
            assert!(map.click(&mut cx, at), "the map takes the click");
            assert_eq!(map.state(&cx)["walking"], json!(true), "a click next to the party walks there");
        }
    }

    /// REGIONVIEW1 (replacing 3.2 A's no clamp): with the party on each corner tile of every map of the world, the party is
    /// drawn on its cell in the map area, the camera stops with `MARGIN_CELLS` of margin (a map that fits centres), and past the
    /// map's edge the wall is drawn (render only).
    #[test]
    fn the_camera_stops_at_the_map_and_walls_fill_past_the_edge() {
        let (mut cx, mut g) = (game_cx(), gfx());
        cx.settings.v["map_legend_open"] = json!(false);
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        let mut maps = 0;
        for loc in d.merged_entries(&world, "locations") {
            let t = explore::template_of(&d, loc);
            if t.is_empty() {
                continue;
            }
            maps += 1;
            let (w, h) = explore::size_of(t);
            for (x, y) in [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)] {
                cx.gs_mut().location_id = s(loc, "id");
                let mut map = RegionMap::new(&mut cx);
                // after `resume` (which puts the party back on a safe tile): the corner itself, wall or not
                cx.gs_mut().position = (x, y);
                cx.log.clear();
                frame(&mut map, &mut cx, &mut g, 1.0);
                let a = area();
                let (px, py, pw, ph) = rect(&widget(&map, "party").expect("the party"));
                assert!(a.contains((px + pw / 2) as i32, (py + ph / 2) as i32), "{} at ({x}, {y}): the party is in view", s(loc, "id"));
                let (cw, ch) = map.geo.cell_size();
                let (mw, mh) = (w as i32 * cw, h as i32 * ch);
                let want = |start: i32, len: i32, map: i32, cell: i32, near: bool| {
                    if map <= len {
                        start + (len - map) / 2
                    } else if near {
                        start + mapview::MARGIN_CELLS * cell
                    } else {
                        start + len - map - mapview::MARGIN_CELLS * cell
                    }
                };
                assert_eq!((map.geo.ox, map.geo.oy), (want(a.x, a.w, mw, cw, x == 0), want(a.y, a.h, mh, ch, y == 0)), "{} at ({x}, {y})", s(loc, "id"));
                // the canvas pixels just past the map on the sides the party is near, along the seams (an edge arrow or
                // the folded legend may sit on one of them, never on all)
                let ox = if x == 0 { map.geo.ox - 2 } else { map.geo.ox + mw + 1 };
                let oy = if y == 0 { map.geo.oy - 2 } else { map.geo.oy + mh + 1 };
                let wall = |px: i32, py: i32, g: &Gfx| {
                    let i = ((py * g.canvas.w + px) * 4) as usize;
                    g.canvas.px[i..i + 3] == [40, 60, 50]
                };
                assert!((1..4).any(|k| wall(ox, a.y + a.h * k / 4, &g)), "{} at ({x}, {y}): the wall past the side at x {ox}", s(loc, "id"));
                assert!((1..4).any(|k| wall(a.x + a.w * k / 4, oy, &g)), "{} at ({x}, {y}): the wall past the end at y {oy}", s(loc, "id"));
            }
        }
        assert!(maps > 0, "the world has a location with a map");
    }

    #[test]
    fn the_legend_lists_the_markers_and_folds_to_its_header() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let head = widget(&map, "legend").expect("the legend header");
        assert_eq!(head["kind"], json!("section"));
        let (hx, hy, hw, _) = rect(&head);
        assert_eq!((hx + hw, hy), ((W - 4) as i64, (HUD_H + 4) as i64), "top right under the bar");
        assert!(hw <= LEGEND_MAX as i64, "at most {LEGEND_MAX} wide at Normal: {hw}");
        let markers = listed(&cx);
        assert!(head["text"].as_str().is_some_and(|t| t == format!("Here ({})", markers.len())), "{head}");
        let want: Vec<String> = markers.iter().map(|m| format!("here.{}", s(m, "id"))).collect();
        let rows = ids_with(&map, "here.");
        assert_eq!(rows, want);
        // a person's row names their services on its hover card
        let gs = cx.gs().clone();
        for m in markers.iter().filter(|m| s(m, "kind") == "npc") {
            let who = cx.data.resolve(&gs.world_id, "characters", &s(m, "character"));
            let services = explore::npc_services(&cx.data, &gs, &s(who, "_qid"));
            let card = widget(&map, &format!("here.{}", s(m, "id"))).map(|w| w["tip"].to_string()).unwrap_or_default();
            assert_eq!(card.contains(&services_text(&services)) && !services.is_empty(), !services.is_empty(), "{card}");
        }
        // a click folds it to the header, kept in the settings file; another opens it again
        map.click_id(&mut cx, "legend").expect("fold");
        assert_eq!(map.state(&cx)["legend_open"], json!(false));
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert!(widget(&map, "legend").is_some() && ids_with(&map, "here.").is_empty(), "folded to its header");
        let saved = Settings::load(cx.shot_dir.join("settings.json"), crate::screens::default_settings());
        assert_eq!(saved.v["map_legend_open"], json!(false), "the fold is saved");
        map.click_id(&mut cx, "legend").expect("open");
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(ids_with(&map, "here."), want);
    }

    #[test]
    fn the_ticker_shows_the_latest_event_and_l_shows_the_last_12() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        cx.log = (0..20).map(|i| format!("Event {i}.")).collect();
        let shown = |map: &RegionMap, recs: &[TextRec]| -> Vec<String> {
            let (x, y, w, h) = rect(&widget(map, "log").expect("the ticker"));
            recs.iter().filter(|r| r.x >= x as f32 && r.x < (x + w) as f32 && r.y >= y as f32 && r.y < (y + h) as f32).map(|r| r.text.clone()).collect()
        };
        let recs = frame(&mut map, &mut cx, &mut g, 1.0);
        let (x, y, _, h) = rect(&widget(&map, "log").expect("the ticker"));
        assert!(x < 10 && y + h >= (H - 10) as i64, "bottom left: {x}, {y}");
        assert_eq!(shown(&map, &recs), ["Event 19."]);
        map.key(&mut cx, Key::Char('l'));
        let recs = frame(&mut map, &mut cx, &mut g, 1.0);
        let want: Vec<String> = (20 - LOG_OPEN..20).map(|i| format!("Event {i}.")).collect();
        assert_eq!(shown(&map, &recs), want, "the last {LOG_OPEN}, newest at the bottom");
        map.click_id(&mut cx, "log").expect("the ticker");
        let recs = frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(shown(&map, &recs), ["Event 19."]);
        // hidden while talking (and the legend with it)
        let world = cx.gs().world_id.clone();
        let people: Vec<String> = cx.data.merged_entries(&world, "characters").iter().map(|c| s(c, "id")).collect();
        for p in people {
            map.talk(&mut cx, &p);
            if map.svc.talking() {
                break;
            }
        }
        assert!(map.svc.talking(), "someone in the world talks");
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert!(widget(&map, "log").is_none() && widget(&map, "legend").is_none(), "no ticker or legend in a conversation");
    }

    /// 2.3 A: names on hover by default (a marker or its legend row under the pointer, or the marker ahead), or always.
    #[test]
    fn names_show_on_hover_by_default_or_always() {
        let (mut cx, mut g) = (game_cx(), gfx());
        assert_eq!(cx.settings.v["map_names"], json!("hover"), "the default");
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let gs = cx.gs().clone();
        let ahead = (gs.position.0 + map.facing.0, gs.position.1 + map.facing.1);
        let t = explore::template_of(&cx.data, explore::location(&cx.data, &gs)).clone();
        let a = area();
        // (id, kind, label, rect, ahead) of the markers wholly inside the map area
        let markers: Vec<(String, String, String, (i64, i64, i64, i64), bool)> = map
            .widgets()
            .iter()
            .filter(|w| w["kind"] == "marker")
            .filter_map(|w| {
                let id = s(json::dict(w), "id").trim_start_matches("marker.").to_string();
                let m = explore::markers_of(explore::location(&cx.data, &gs)).into_iter().find(|m| s(m, "id") == id)?;
                let r = rect(w);
                let inside = r.0 >= 0 && r.1 >= a.y as i64 + 24 && r.0 + r.2 <= W as i64 && r.1 + r.3 <= H as i64;
                inside.then(|| (id, s(&m, "kind"), s(json::dict(w), "text"), r, explore::marker_pos(&t, &m, gs.explore_int("loc_steps")) == ahead))
            })
            .collect();
        assert!(!markers.is_empty(), "the start shows a marker");
        let recs = frame(&mut map, &mut cx, &mut g, 1.0);
        for (id, _, label, r, faced) in &markers {
            assert_eq!(named_above(&recs, label, *r), *faced, "{id}: named only when the party faces it");
        }
        // the pointer on a marker names it; on its legend row too
        let (id, _, label, r, _) = &markers[0];
        let centre = ((r.0 + r.2 / 2) as i32, (r.1 + r.3 / 2) as i32);
        let recs = frame_at(&mut map, &mut cx, &mut g, centre);
        assert!(named_above(&recs, label, *r), "{id}: hovered, named");
        if let Some(row) = widget(&map, &format!("here.{id}")) {
            let (x, y, w, h) = rect(&row);
            let on_row = ((x + w / 2) as i32, (y + h / 2) as i32);
            frame_at(&mut map, &mut cx, &mut g, on_row);
            let recs = frame_at(&mut map, &mut cx, &mut g, on_row);
            assert!(named_above(&recs, label, *r), "{id}: its legend row hovered, named");
        }
        // always: every person and exit
        cx.settings.v["map_names"] = json!("always");
        let recs = frame(&mut map, &mut cx, &mut g, 1.0);
        for (id, kind, label, r, _) in &markers {
            assert_eq!(named_above(&recs, label, *r), kind == "npc" || kind == "exit", "{id} ({kind}) with names always");
        }
    }

    /// TEXTFIT1: the map's own texts (legend open and folded, ticker short and opened, names on hover and always) fit
    /// at every text size: nothing cut, past the canvas, overlapping or out of its control.
    #[test]
    fn the_region_map_reads_clean_at_every_text_size() {
        // EXPLOREVIEW1: under both region views
        mapview::VIEWS.iter().for_each(|v| the_region_map_reads_clean_at_every_text_size_in(v.0));
    }

    fn the_region_map_reads_clean_at_every_text_size_in(view: &str) {
        let (mut cx, mut g) = (game_cx(), gfx());
        reads_clean(&mut cx, &mut g, view);
        // REGIONVIEW1: the zoom moves names and arrows, and the panes keep their size
        cx.settings.v["map_zoom"] = json!(2.0);
        reads_clean(&mut cx, &mut g, view);
    }

    /// The map on `cx` in `view` under every names, legend and log state at the four text sizes.
    pub(super) fn reads_clean(cx: &mut Cx, g: &mut Gfx, view: &str) {
        cx.settings.v["region_view"] = json!(view);
        let mut map = RegionMap::new(cx);
        let place = cx.location_name(&cx.gs().world_id.clone(), &cx.gs().location_id.clone());
        for line in ["The party rests (+40 HP).".to_string(), format!("Entered {place}."), "The party makes camp (-3 supplies, +40 HP).".into(), "Found: Healing Draught x2, 15 gold.".into()] {
            cx.log(line);
        }
        for names in ["hover", "always"] {
            cx.settings.v["map_names"] = json!(names);
            for legend in [true, false] {
                cx.settings.v["map_legend_open"] = json!(legend);
                for log_open in [false, true] {
                    map.log_open = log_open;
                    for (size, scale) in TEXT_SIZES {
                        let recs = frame(&mut map, cx, g, scale as f32);
                        let boxes: Vec<(String, f32, f32, f32, f32)> = map
                            .widgets()
                            .iter()
                            .map(|w| {
                                let (x, y, ww, hh) = rect(w);
                                (s(json::dict(w), "id"), x as f32, y as f32, ww as f32, hh as f32)
                            })
                            .collect();
                        let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
                        let counts = &v["counts"];
                        let label = format!("names {names}, legend {legend}, log {log_open} at {size}: {v}");
                        assert_eq!((counts["cut"].clone(), counts["edge"].clone(), counts["spill"].clone()), (json!(0), json!(0), json!(0)), "{label}");
                        // names may meet each other where people stand side by side (content); nothing else may overlap
                        if names == "hover" {
                            assert_eq!(counts["overlap"], json!(0), "{label}");
                        }
                    }
                }
            }
        }
    }
}

/// PLAYUI1-D (spec PLAYUI1 3.1-3.3): the dialogue box, the camera on both speakers, the shop over the dimmed map, drawn
/// headless over a debug game of the first playable world (art layers off: portraits are initials, tiles flat fills).
#[cfg(test)]
mod talk_tests {
    use super::*;
    use crate::talk::{PORTRAIT, TALK_GAP, TALK_LEFT, TALK_MAX_PCT, TALK_MIN_H, TALK_PAD, TALK_W};
    use crate::hud::hud_tests::{frame, game_cx, gfx, rect};
    use aegis_ui::painter::TextRec;
    use aegis_ui::screen::TEXT_SIZES;

    fn widget(map: &RegionMap, id: &str) -> Option<Value> {
        map.widgets().into_iter().find(|w| w["id"] == id)
    }

    fn replies(widgets: &[Value]) -> Vec<Value> {
        widgets.iter().filter(|w| w["kind"] == "choice").cloned().collect()
    }

    /// TEXTFIT1 counts (cut, edge, spill, overlap) of a frame, and the report.
    fn audit(widgets: &[Value], recs: &[TextRec]) -> (Vec<i64>, Value) {
        let boxes: Vec<(String, f32, f32, f32, f32)> = widgets
            .iter()
            .map(|w| {
                let (x, y, ww, hh) = rect(w);
                (s(json::dict(w), "id"), x as f32, y as f32, ww as f32, hh as f32)
            })
            .collect();
        let v = aegis_ui::text_audit::issues(recs, W as f32, H as f32, &boxes);
        let c = &v["counts"];
        (["cut", "edge", "spill", "overlap"].iter().map(|k| c[*k].as_i64().unwrap_or(-1)).collect(), v)
    }

    /// The box's width at text size `scale` (3.1: 400 times the text size, at most the canvas less 40).
    fn box_w(scale: f64) -> i64 {
        ((W - 40) as i64).min((TALK_W as f64 * scale).round() as i64)
    }

    /// Every conversation opened the way a player opens it: for each person on a map of the world, from each open tile
    /// next to them, facing them and pressing E. `each` gets the map (talking), the party's tile and the person's, and
    /// says whether to go on; the game state is put back after each. The number of conversations opened.
    pub(super) fn each_talk(cx: &mut Cx, mut each: impl FnMut(&mut RegionMap, &mut Cx, Pos, Pos) -> bool) -> usize {
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        let start = cx.gs().clone();
        let mut opened = 0;
        for loc in d.merged_entries(&world, "locations") {
            let t = explore::template_of(&d, loc).clone();
            if t.is_empty() {
                continue;
            }
            cx.gs_mut().location_id = s(loc, "id");
            let _ = RegionMap::new(cx);
            let here = cx.gs().clone();
            let steps = here.explore_int("loc_steps");
            for m in explore::markers_of(loc) {
                if s(&m, "kind") != "npc" || explore::presence(&d, &here, &s(loc, "_qid"), &m) == "absent" {
                    continue;
                }
                let at = explore::marker_pos(&t, &m, steps);
                for dir in explore::DIRS {
                    let p = (at.0 - dir.0, at.1 - dir.1);
                    if !explore::is_open(&t, p.0, p.1) {
                        continue;
                    }
                    *cx.gs_mut() = here.clone();
                    let mut map = RegionMap::new(cx);
                    cx.gs_mut().position = p;
                    map.interact(cx, dir);
                    if map.svc.talking() {
                        opened += 1;
                        let go_on = each(&mut map, cx, p, at);
                        map.svc.end(cx);
                        if !go_on {
                            *cx.gs_mut() = start;
                            return opened;
                        }
                    }
                }
            }
        }
        *cx.gs_mut() = start;
        opened
    }

    /// 3.1: 400 wide (times the text size), centred, 6 above the bottom, 104 to 45 % of the canvas tall; the name under
    /// the portrait; the line above the replies; no key hint line; the legend and the log ticker stay hidden.
    #[test]
    fn the_dialogue_box_is_a_small_centred_bottom_box() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let n = each_talk(&mut cx, |map, cx, _, _| {
            let name = s(json::dict(&map.state(cx)["dialogue"]), "speaker_name");
            for (size, scale) in TEXT_SIZES {
                let recs = frame(map, cx, &mut g, scale as f32);
                let (x, y, w, h) = rect(&widget(map, "dialogue").expect("the dialogue box"));
                assert_eq!(w, box_w(scale), "{size}: width");
                assert!((x - (W as i64 - x - w)).abs() <= 1, "{size}: centred ({x}, {w})");
                assert_eq!(H as i64 - (y + h), TALK_GAP as i64, "{size}: {TALK_GAP} above the bottom");
                assert!(h >= TALK_MIN_H as i64 && h <= (H * TALK_MAX_PCT / 100) as i64, "{size}: height {h}");
                let left = (TALK_LEFT as f64).max((TALK_LEFT as f64 * scale).round()) as f32;
                let under = (y + (TALK_PAD + PORTRAIT) as i64) as f32;
                let named = recs.iter().any(|r| !r.text.is_empty() && name.starts_with(r.text.as_str()) && r.y >= under && r.x >= x as f32 && r.x + r.w <= x as f32 + left);
                assert!(named, "{size}: {name} under the portrait");
                let (_, ty, _, th) = rect(&widget(map, "dialogue.text").expect("the line"));
                let rows = replies(&map.widgets());
                assert!(!rows.is_empty(), "{size}: a reply or Continue");
                for r in &rows {
                    let (rx, ry, rw, rh) = rect(r);
                    assert!(ry >= ty + th && rx >= x && ry + rh <= y + h && rx + rw <= x + w, "{size}: {r} below the line, inside the box");
                }
                assert!(!recs.iter().any(|r| r.text.contains("1-9")), "{size}: no key hint line");
                assert!(widget(map, "legend").is_none() && widget(map, "log").is_none(), "{size}: no legend or ticker while talking");
            }
            false
        });
        assert!(n > 0, "someone on a map of the world talks");
    }

    /// 3.2 acceptance: for every person on the world's maps, a conversation opened from any open tile next to them keeps
    /// both the party's and their tile inside the map above the box, at all four text sizes; every box reads clean.
    #[test]
    fn both_speakers_stay_in_view_above_the_box() {
        // EXPLOREVIEW1: under both region views
        mapview::VIEWS.iter().for_each(|v| both_speakers_stay_in_view_above_the_box_in(v.0));
    }

    fn both_speakers_stay_in_view_above_the_box_in(view: &str) {
        let (mut cx, mut g) = (game_cx(), gfx());
        cx.settings.v["region_view"] = json!(view);
        let n = each_talk(&mut cx, |map, cx, party, npc| {
            for (size, scale) in TEXT_SIZES {
                let recs = frame(map, cx, &mut g, scale as f32);
                let (_, by, _, _) = rect(&widget(map, "dialogue").expect("the dialogue box"));
                let view = Rect::new(0, HUD_H, W, by as i32 - HUD_H);
                for (who, at) in [("party", party), ("speaker", npc)] {
                    let r = map.geo.cell_of(at);
                    let inside = r.x >= view.x && r.y >= view.y && r.x + r.w <= view.x + view.w && r.y + r.h <= view.y + view.h;
                    assert!(inside, "{} at {party:?} talking to {npc:?}, {size}: the {who}'s tile {r:?} is not inside {view:?}", cx.gs().location_id);
                }
                // the box's texts (the map's own are `map_tests`'; a drawn badge stand-in may sit on the party's tile
                // when the party stands just above a quest giver)
                let in_box: Vec<TextRec> = recs.into_iter().filter(|r| r.y >= by as f32).collect();
                let (counts, v) = audit(&map.widgets(), &in_box);
                assert_eq!(counts, [0, 0, 0, 0], "{} at {party:?}, {size}: {v}", cx.gs().location_id);
            }
            true
        });
        eprintln!("{n} conversations checked");
        assert!(n > 0, "someone on a map of the world talks");
    }

    /// 3.1: the number keys still choose (the farewell reply ends the conversation).
    #[test]
    fn a_number_key_chooses_a_reply() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut checked = false;
        each_talk(&mut cx, |map, cx, _, _| {
            frame(map, cx, &mut g, 1.0);
            let choices = json::aget(json::dict(&map.state(cx)["dialogue"]), "choices").to_vec();
            let Some(i) = choices.iter().position(|c| s(json::dict(c), "service") == "farewell").filter(|i| *i < 9) else { return true };
            map.key(cx, Key::Char(char::from(b'1' + i as u8)));
            assert!(!map.svc.talking(), "key {} (farewell) ends the conversation", i + 1);
            checked = true;
            false
        });
        assert!(checked, "a conversation with a farewell reply");
    }

    /// 3.1 text audit with a long line and many replies: the box stays at most 45 % of the canvas, the line and the
    /// replies scroll inside it, nothing is cut, past the canvas, spilled or overlapping at any text size.
    #[test]
    fn a_long_line_and_many_replies_scroll_inside_the_box() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let world = cx.gs().world_id.clone();
        let speaker = cx.data.merged_entries(&world, "characters").first().map(|c| s(c, "id")).unwrap_or_default();
        let line = "The ash has not stopped falling since the forge went cold, and every road out of the valley is watched by the Hollow riders. ".repeat(4);
        let mut choices: Vec<Value> = (0..12)
            .map(|i| json!({"text": format!("Reply {i}: tell me more about the riders, the forge and the road north before I decide."), "locked": false, "reason": "", "service": ""}))
            .collect();
        choices[2] = json!({"text": "[Might 8] Force the gate open.", "locked": true, "reason": "Needs Might 8 (best 5)", "service": ""});
        let view = json::dict(&json!({"speaker": speaker, "speaker_name": "Warden Ilsa Varn", "node": "long", "text": line.trim_end(), "choices": choices,
            "end": false, "next": false, "faction": "f", "faction_name": "Ember Wardens", "band": "neutral"}))
        .clone();
        for (size, scale) in TEXT_SIZES {
            g.fonts.scale = scale as f32;
            g.fonts.audit = Some(Vec::new());
            let mut ui: Ui<RAct> = Ui::new(&mut g, (-500, -500), true);
            let b = talk_layout(&mut ui.p, &view);
            draw_talk(&mut ui, &mut cx, &mut Faces::default(), &view, &b, false);
            let widgets: Vec<Value> = ui.widgets.iter().map(|w| w.info()).collect();
            drop(ui);
            let recs = g.fonts.audit.take().unwrap_or_default();
            assert!(b.r.h <= H * TALK_MAX_PCT / 100 && b.r.y + b.r.h == H - TALK_GAP, "{size}: {:?}", b.r);
            let max = |id: &str| widgets.iter().find(|w| w["id"] == id).and_then(|w| w["text"].as_str()?.parse::<i32>().ok()).unwrap_or(0);
            assert!(max("dialogue.text") > 0 && max("dialogue.choices") > 0, "{size}: the line and the replies scroll");
            let shown: Vec<Value> = replies(&widgets).into_iter().filter(|w| rect(w).2 > 0 && rect(w).3 > 0).collect();
            assert!(shown.first().is_some_and(|w| w["id"] == "choice.0") && shown.len() < 12, "{size}: the first replies show, the rest scroll");
            for w in &shown {
                let (x, y, ww, hh) = rect(w);
                let inside = x >= b.r.x as i64 && y >= b.r.y as i64 && x + ww <= (b.r.x + b.r.w) as i64 && y + hh <= (b.r.y + b.r.h) as i64;
                assert!(inside, "{size}: {w} inside the box");
            }
            assert!(shown.iter().any(|w| w["id"] == "choice.2" && w["enabled"] == json!(false)), "{size}: the locked reply shows, off");
            let (counts, v) = audit(&widgets, &recs);
            assert_eq!(counts, [0, 0, 0, 0], "{size}: {v}");
        }
    }

    /// 3.3: the shop hides the box and sits centred under the HUD bar over the dimmed map; Esc brings the box back.
    #[test]
    fn the_shop_hides_the_box_over_a_dimmed_map() {
        let (mut cx, mut g) = (game_cx(), gfx());
        // the map's brightness left of the box (the legend and the ticker are hidden while talking)
        let light = |g: &Gfx| -> u64 {
            let mut sum = 0;
            for y in (HUD_H + 2)..(H - 10) {
                for x in 0..100 {
                    let i = ((y * g.canvas.w + x) * 4) as usize;
                    sum += g.canvas.px[i..i + 3].iter().map(|v| *v as u64).sum::<u64>();
                }
            }
            sum
        };
        let mut checked = false;
        each_talk(&mut cx, |map, cx, _, _| {
            let choices = json::aget(json::dict(&map.state(cx)["dialogue"]), "choices").to_vec();
            let Some(k) = choices.iter().position(|c| s(json::dict(c), "service") == "shop") else { return true };
            frame(map, cx, &mut g, 1.0);
            let before = light(&g);
            map.click_id(cx, &format!("choice.{k}")).expect("the shop reply");
            frame(map, cx, &mut g, 1.0);
            assert!(widget(map, "dialogue").is_none() && replies(&map.widgets()).is_empty(), "the box hides while the shop is open");
            let (x, y, w, h) = rect(&widget(map, "shop").expect("the shop"));
            assert!((x - (W as i64 - x - w)).abs() <= 1 && (y - HUD_H as i64 - (H as i64 - y - h)).abs() <= 1, "centred under the HUD bar: {x}, {y}, {w}, {h}");
            assert!(light(&g) * 10 < before * 7, "the map is dimmed: {} vs {before}", light(&g));
            map.back(cx);
            frame(map, cx, &mut g, 1.0);
            assert!(map.svc.talking() && widget(map, "dialogue").is_some() && widget(map, "shop").is_none(), "Esc closes the shop, the box comes back");
            checked = true;
            false
        });
        if !checked {
            eprintln!("no shop keeper on the world's maps: nothing to check");
        }
    }
}

/// PORTRAITFX1-D (docs/specs/PORTRAITFX1.md 6.2 row 1; PORTRAITFX1_TASKS 1.5 D1, D2, D4, D5): the talk box's portrait
/// lives, drawn headless over a debug game of the first playable world with a scratch portrait layer (a sidecar with
/// eyes on a Tallow Amber fill, so the rim and the blink show) and cue rows set by each test, not the shipped ones.
#[cfg(test)]
mod portrait_tests {
    use super::talk_tests::each_talk;
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx, rect};
    use crate::portraits::tests::{layer, row};
    use crate::portraits::RIM;
    use aegis_core::assets::Layers;
    use aegis_ui::art::Art;
    use aegis_ui::fx::Clock;
    use aegis_ui::portrait_fx::{blink_phase, blink_starts, seed_of, tq, BLINK_MS, EASE_MS};
    use aegis_ui::screen::TEXT_SIZES;
    use emberwake_data::PortraitCue;

    /// A game whose cue rows are `rows` (none of the shipped data's).
    fn cx_with(rows: Vec<PortraitCue>) -> Cx {
        let mut cx = game_cx();
        std::sync::Arc::get_mut(&mut cx.app.data).expect("the test owns the data").edit_cat(|c| c.portrait_cues = rows);
        cx
    }

    /// A scratch layer holding `character`'s portrait (the first file of its chain) with eyes, a dark lid and the
    /// game's rim key ("").
    fn portrait_art(cx: &mut Cx, name: &str, character: &Dict) {
        let rel = crate::screens::character_art(&cx.data, character).2.first().cloned().expect("a portrait chain");
        let face = json!({"size": 96, "eyes": [[30, 40, 10, 6], [56, 40, 10, 6]], "lid": "#203040", "rim": ""});
        cx.art = Art::layered(Layers::of(vec![layer(name, &rel, 96, [RIM[0], RIM[1], RIM[2]], Some(face))]));
    }

    /// The speaker of the open conversation on `map`.
    fn speaker(map: &RegionMap, cx: &Cx) -> Dict {
        let view = map.state(cx)["dialogue"].clone();
        cx.data.resolve(&cx.gs().world_id, "characters", &s(json::dict(&view), "speaker")).clone()
    }

    /// The talk box portrait composed at clock `t` (one frame of the map).
    fn composed_at(map: &mut RegionMap, cx: &mut Cx, g: &mut Gfx, key: &str, t: u64) -> Vec<u8> {
        map.svc.faces.clock = Clock { ms: t as f64, hold: 0.0 };
        frame(map, cx, g, 1.0);
        map.svc.faces.get(key).and_then(|f| f.composed()).map(|i| i.px.clone()).expect("the speaker's portrait composed")
    }

    /// D1 and D5: the talk box portrait changes pixels over 4 s by default (the rim flickers; breath at the full tier);
    /// under reduce motion the frames differ only where a blink is closed or half closed (4.7). While the conversation
    /// is open the map's `state` lists the speaker's portrait; with none open, no portrait.
    #[test]
    fn d1_d5_the_talk_box_portrait_lives_and_shows_in_the_state() {
        let mut cx = cx_with(Vec::new());
        let mut g = gfx();
        let quiet = RegionMap::new(&mut cx);
        assert_eq!(quiet.state(&cx)["portraits"], json!([]), "no conversation: no portrait");
        let mut checked = false;
        each_talk(&mut cx, |map, cx, _, _| {
            let character = speaker(map, cx);
            let key = s(&character, "id");
            if key.is_empty() {
                return true;
            }
            portrait_art(cx, "talk_d1", &character);
            let seed = seed_of(&key);
            // every 100 ms over 4 s, and the middle of the first blink (wherever it falls)
            let first = blink_starts(seed, 10_000).first().copied().expect("a blink within 10 s");
            let mut times: Vec<u64> = (0..=40).map(|k| k * 100).collect();
            times.push(first + BLINK_MS / 2);
            let open = |t: u64| blink_phase(seed, tq(t)) == 0;
            for reduce in [false, true] {
                cx.settings.v["reduce_motion"] = json!(reduce);
                let frames: Vec<(u64, Vec<u8>)> = times.iter().map(|&t| (t, composed_at(map, cx, &mut g, &key, t))).collect();
                let still: Vec<&(u64, Vec<u8>)> = frames.iter().filter(|f| open(f.0)).collect();
                let blinks: Vec<&(u64, Vec<u8>)> = frames.iter().filter(|f| !open(f.0)).collect();
                assert!(!blinks.is_empty(), "a sample inside a blink");
                if reduce {
                    assert!(still.iter().all(|f| f.1 == still[0].1), "reduce motion: the open-eyed frames are equal");
                    assert!(blinks.iter().all(|f| f.1 != still[0].1), "reduce motion: a blink still shows");
                } else {
                    assert!(still.iter().any(|f| f.1 != still[0].1), "by default the open-eyed frames change over 4 s");
                }
            }
            let st = map.state(cx);
            let listed = st["portraits"].as_array().cloned().unwrap_or_default();
            assert!(listed.iter().any(|p| p["key"] == json!(key)), "the speaker's portrait is in the state: {st}");
            assert!(listed.iter().all(|p| p["key"] == json!(key)), "only the speaker's");
            map.svc.end(cx);
            frame(map, cx, &mut g, 1.0);
            assert_eq!(map.state(cx)["portraits"], json!([]), "the conversation ended: no portrait");
            checked = true;
            false
        });
        assert!(checked, "a conversation with a speaker");
    }

    /// A view of a conversation with `character` (as `DialogueRunner::view` shapes it).
    fn view_of(character: &Dict) -> Dict {
        json::dict(&json!({"speaker": s(character, "_qid"), "speaker_name": s(character, "name"), "node": "n", "text": "A line.", "choices": [{"text": "Farewell.", "locked": false, "reason": "", "service": "farewell"}],
            "end": false, "next": false, "faction": "", "faction_name": "", "band": "neutral"}))
        .clone()
    }

    /// Draws the talk box for `view` at clock `t` into `faces`, at text size `scale`: its widgets and the texts drawn.
    fn talk_at(cx: &mut Cx, g: &mut Gfx, faces: &mut Faces, view: &Dict, t: u64, scale: f32) -> (Vec<Value>, Vec<aegis_ui::painter::TextRec>) {
        faces.clock = Clock { ms: t as f64, hold: 0.0 };
        g.fonts.scale = scale;
        g.fonts.audit = Some(Vec::new());
        let mut ui: Ui<RAct> = Ui::new(g, (-500, -500), true);
        faces.begin();
        let b = talk_layout(&mut ui.p, view);
        draw_talk(&mut ui, cx, faces, view, &b, false);
        let widgets = ui.widgets.iter().map(|w| w.info()).collect();
        drop(ui);
        (widgets, g.fonts.audit.take().unwrap_or_default())
    }

    /// D2 and D4: a speaker played by a wounded party member draws the `wounded` row's desaturate 0.35 (with the rows of
    /// PORTRAITFX1_TASKS 2.3); a world character no member plays matches no `hp` row (it has no HP); the talk box with a
    /// drawn portrait reads clean (TEXTFIT1) at every text size.
    #[test]
    fn d2_d4_a_wounded_member_speaker_is_desaturated_and_the_box_reads_clean() {
        let rows = vec![row(json!({"id": "wounded", "trigger": "condition", "condition": "wounded", "effect": "desaturate", "intensity": 0.35})), row(json!({"id": "any_hp", "trigger": "hp", "below": 100, "effect": "tint", "intensity": 0.5}))];
        let mut cx = cx_with(rows);
        let mut g = gfx();
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        let at = cx.gs().party.members.iter().position(|m| !emberwake_world::progression::character_of(&d, m).is_empty()).expect("a member who plays a character");
        cx.gs_mut().party.members[at].conditions = vec!["wounded".to_string()];
        let character = emberwake_world::progression::character_of(&d, &cx.gs().party.members[at]).clone();
        portrait_art(&mut cx, "talk_d2", &character);
        let view = view_of(&character);
        let key = s(&character, "id");
        let mut faces = Faces::default();
        for t in [1000, 1000 + EASE_MS] {
            talk_at(&mut cx, &mut g, &mut faces, &view, t, 1.0);
        }
        let fx = faces.get(&key).expect("the member's portrait");
        let now = faces.now();
        assert!(fx.level("desaturate", now).is_some_and(|l| (l.i - 0.35).abs() < 1e-3), "wounded: desaturate 0.35");
        assert!(fx.level("tint", now).is_none(), "the member's HP is full: the hp row does not match");
        assert!(fx.live(now).contains(&"desaturate"));
        // a world character no member plays: no hp row, whatever its HP would be
        let played: Vec<String> = cx.gs().party.members.iter().map(|m| s(emberwake_world::progression::character_of(&d, m), "_qid")).collect();
        let other = d.merged_entries(&world, "characters").into_iter().find(|c| !played.contains(&s(c, "_qid"))).expect("a character no member plays").clone();
        portrait_art(&mut cx, "talk_d2_other", &other);
        let mut faces = Faces::default();
        let view = view_of(&other);
        for t in [1000, 1000 + EASE_MS] {
            talk_at(&mut cx, &mut g, &mut faces, &view, t, 1.0);
        }
        let fx = faces.get(&s(&other, "id")).expect("the character's portrait");
        assert!(fx.level("tint", faces.now()).is_none() && fx.level("desaturate", faces.now()).is_none(), "a world character matches no hp or condition row");
        // D4: the box reads clean with the portrait drawn, at every text size
        for (size, scale) in TEXT_SIZES {
            let (widgets, recs) = talk_at(&mut cx, &mut g, &mut faces, &view, 2000, scale as f32);
            assert!(faces.get(&s(&other, "id")).and_then(|f| f.composed()).is_some(), "{size}: the portrait drew");
            let boxes: Vec<(String, f32, f32, f32, f32)> = widgets
                .iter()
                .map(|w| {
                    let (x, y, ww, hh) = rect(w);
                    (s(json::dict(w), "id"), x as f32, y as f32, ww as f32, hh as f32)
                })
                .collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            assert_eq!(v["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "{size}: {v}");
        }
    }
}

/// PLAYUI1-R (spec PLAYUI1 2.4): the leader and the people as body sprites, drawn headless over a debug game of the
/// first playable world with the game's art layers (the game's, its kit's and the engine's assets).
#[cfg(test)]
mod sprite_tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx};

    /// A debug game with the game's art.
    fn art_cx() -> Cx {
        let mut cx = game_cx();
        cx.art = aegis_ui::art::Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        cx
    }

    fn sprite(map: &RegionMap, cx: &Cx, id: &str) -> Option<Value> {
        map.state(cx)["sprites"].as_array()?.iter().find(|s| s["id"] == id).cloned()
    }

    fn rect_of(v: &Value) -> Rect {
        let n = |i: usize| v["rect"][i].as_i64().unwrap_or(0) as i32;
        Rect::new(n(0), n(1), n(2), n(3))
    }

    /// The map tile at (`x`, `y`) in the last frame.
    fn tile_rect(map: &RegionMap, at: Pos) -> Rect {
        map.geo.cell_of(at)
    }

    /// The tile a sprite stands on: under the middle of its bottom edge.
    fn stands_on(map: &RegionMap, r: Rect) -> Pos {
        map.geo.cell_at(r.x + r.w / 2, r.y + r.h - 1)
    }

    /// The first active member's body as the art resolves it: their character's body, else the class body.
    fn leader_body(cx: &mut Cx) -> Option<String> {
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        let m = gs.party.active_members().first().copied().cloned()?;
        let ch = d.resolve(&gs.world_id, "characters", &m.lore_ref);
        let (body, _, _) = crate::screens::character_art(&d, ch);
        cx.art.first_body(&emberwake_content::art::body_chain(&body))
    }

    /// 2.4: the leader stands on the party's tile, 1.5 tiles tall and bottom-centred, idle and facing right at the start.
    #[test]
    fn the_leader_stands_on_the_party_tile() {
        // EXPLOREVIEW1: under both region views
        mapview::VIEWS.iter().for_each(|v| the_leader_stands_on_the_party_tile_in(v.0));
    }

    fn the_leader_stands_on_the_party_tile_in(view: &str) {
        let (mut cx, mut g) = (art_cx(), gfx());
        cx.settings.v["region_view"] = json!(view);
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let want = leader_body(&mut cx).expect("the leader has a body");
        let sp = sprite(&map, &cx, "party").expect("the leader is a sprite");
        assert_eq!(sp["body"], json!(want));
        assert_eq!((sp["clip"].clone(), sp["left"].clone()), (json!("idle"), json!(false)));
        let (r, t) = (rect_of(&sp), tile_rect(&map, cx.gs().position));
        assert_eq!((r.w, r.h), (SPRITE, SPRITE), "1.5 tiles tall");
        assert_eq!((r.y + r.h, r.x + r.w / 2), (t.y + t.h, t.x + t.w / 2), "bottom-centred on the tile");
    }

    /// The view a body draws `clip` in for a `facing` (`side`, `down`, `up`): the facing's view when the body has a
    /// row of that view for the clip, else the side row (DIR4 3.2).
    fn drawn_view(cx: &mut Cx, body: &str, clip: &str, facing: &str) -> &'static str {
        let has = cx.art.clips(body).is_some_and(|c| c.clips.iter().any(|c| c.of == clip && view_name(c.view) == facing));
        match facing {
            "down" if has => "down",
            "up" if has => "up",
            _ => "side",
        }
    }

    /// 2.4 and DIR4-M: a step plays `walk` and a step left mirrors the sprite; a down or up step faces down or up
    /// (`Facing::toward`, keeping the last left/right); a step right turns it back; standing still it plays `idle`
    /// again. The row drawn is the facing's when the body has one, else the side row.
    #[test]
    fn steps_walk_and_turn_the_leader() {
        let (mut cx, mut g) = (art_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        let d = cx.data.clone();
        let start = cx.gs().clone();
        let t = explore::template_of(&d, explore::location(&d, &start)).clone();
        let (w, h) = explore::size_of(&t);
        let marked: Vec<Pos> = explore::markers_of(explore::location(&d, &start)).iter().map(|m| explore::marker_pos(&t, m, start.explore_int("loc_steps"))).collect();
        let free = |p: Pos| explore::is_open(&t, p.0, p.1) && !marked.contains(&p);
        // a free tile with free tiles left of it and below that one
        let at = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).find(|&(x, y)| free((x, y)) && free((x - 1, y)) && free((x - 1, y + 1)) && free((x, y + 1))).expect("room for three steps");
        cx.gs_mut().position = at;
        let mut checks = 0;
        // (step, facing, left/right kept): left, down (keeps left), right, up (keeps right), back to the start
        for (dir, facing, left) in [((-1, 0), "side", true), ((0, 1), "down", true), ((1, 0), "side", false), ((0, -1), "up", false)] {
            let from = cx.gs().position;
            map.step(&mut cx, dir);
            assert_eq!(cx.gs().position, (from.0 + dir.0, from.1 + dir.1), "the step {dir:?} from {from:?}: {:?}", cx.log);
            let want = Facing { view: View::parse(facing).unwrap_or(View::Side), left };
            assert_eq!(map.leader_face, want, "the facing after the step {dir:?}");
            frame(&mut map, &mut cx, &mut g, 1.0);
            let sp = sprite(&map, &cx, "party").expect("the leader");
            let body = s(json::dict(&sp), "body");
            let view = drawn_view(&mut cx, &body, "walk", facing);
            assert_eq!((sp["clip"].as_str().map(|c| c.starts_with("walk")), sp["facing"].clone(), sp["view"].clone()), (Some(true), json!(facing), json!(view)), "after the step {dir:?}");
            // a side row mirrors facing left; an up or down row never does
            assert_eq!(sp["left"], json!(view == "side" && left), "mirrored after the step {dir:?}");
            checks += 1;
        }
        assert_eq!(checks, 4);
        map.anim_ms += map.step.base + WALK_LINGER_MS + 1.0;
        frame(&mut map, &mut cx, &mut g, 1.0);
        // standing still after the last step (up): idle in the view it faces, the side row when the body has no up row
        let sp = sprite(&map, &cx, "party").expect("the leader");
        let view = drawn_view(&mut cx, &s(json::dict(&sp), "body"), "idle", "up");
        let want = if view == "side" { "idle".to_string() } else { format!("idle_{view}") };
        assert_eq!((sp["clip"].clone(), sp["view"].clone()), (json!(want), json!(view)), "standing still");
    }

    /// WALK2: a held movement key steps at once on the press (no key-repeat delay), chains a step every `step`
    /// with the leader on one unbroken `walk` clip, turns at once to a newly held direction, and releasing stops on
    /// the tile and settles to idle once the linger has run out.
    #[test]
    fn a_held_key_walks_at_once_and_chains_steps() {
        let (mut cx, mut g) = (art_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        let d = cx.data.clone();
        let start = cx.gs().clone();
        let t = explore::template_of(&d, explore::location(&d, &start)).clone();
        let (w, h) = explore::size_of(&t);
        let marked: Vec<Pos> = explore::markers_of(explore::location(&d, &start)).iter().map(|m| explore::marker_pos(&t, m, start.explore_int("loc_steps"))).collect();
        let free = |p: Pos| explore::is_open(&t, p.0, p.1) && !marked.contains(&p);
        // a free tile with a free row of three to its right and one free tile below the last of them
        let at = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).find(|&(x, y)| (0..4).all(|i| free((x + i, y))) && free((x + 3, y + 1))).expect("room for four steps");
        cx.gs_mut().position = at;
        // the press steps on the same frame, and the key stays armed
        map.key(&mut cx, Key::Right);
        cx.held.dir = Some((1, 0));
        assert_eq!(cx.gs().position, (at.0 + 1, at.1), "the press steps at once");
        assert!(map.hold, "held");
        // a frame before the step interval is up does not step again
        map.tick(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(cx.gs().position, (at.0 + 1, at.1), "no second step inside the interval");
        assert_eq!(map.leader.base(), "walk");
        // each interval that elapses steps once and the clip never leaves `walk`
        for n in 2..=3 {
            map.next_walk = Instant::now() - Duration::from_millis(1);
            map.tick(&mut cx);
            frame(&mut map, &mut cx, &mut g, 1.0);
            assert_eq!(cx.gs().position, (at.0 + n, at.1), "step {n} of the hold");
            assert!(map.moving());
            assert_eq!(map.leader.base(), "walk", "no idle frame between steps");
        }
        // the key is read from the held state: a new direction held turns the leader on its next step
        map.key(&mut cx, Key::Down);
        cx.held.dir = Some((0, 1));
        assert_eq!(cx.gs().position, (at.0 + 3, at.1 + 1), "the turn steps at once");
        assert_eq!(map.leader.base(), "walk");
        // release: the next frame steps no more, the leader keeps the walk only until the linger is out, then idles
        cx.held.dir = None;
        map.next_walk = Instant::now() - Duration::from_millis(1);
        map.tick(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(cx.gs().position, (at.0 + 3, at.1 + 1), "released: stopped on the tile");
        assert!(!map.hold);
        map.anim_ms += map.step.base + WALK_LINGER_MS + 1.0;
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(map.leader.base(), "idle");
    }

    /// 2.4 and DIR4-M: every person with a body is a sprite on their tile (idle, facing right); in a conversation
    /// opened from any side the person turns to the party four ways (`Facing::toward`: left when the party is to their
    /// left, down or up when it stands straight below or above, keeping the sheets' right), and the leader faces them
    /// the same way. The row drawn is the facing's when the body has one, else the side row.
    #[test]
    fn people_are_sprites_and_face_the_party_while_talking() {
        // EXPLOREVIEW1: under both region views
        mapview::VIEWS.iter().for_each(|v| people_are_sprites_and_face_the_party_while_talking_in(v.0));
    }

    fn people_are_sprites_and_face_the_party_while_talking_in(view: &str) {
        let (mut cx, mut g) = (art_cx(), gfx());
        cx.settings.v["region_view"] = json!(view);
        // (left, right, down or up) conversations checked
        let mut sides = (0, 0, 0);
        let n = super::talk_tests::each_talk(&mut cx, |map, cx, party, npc| {
            frame(map, cx, &mut g, 1.0);
            let sprites = map.state(cx)["sprites"].as_array().cloned().unwrap_or_default();
            let them = sprites.iter().find(|s| s["id"] != "party" && stands_on(map, rect_of(s)) == npc).cloned();
            let Some(them) = them else {
                // no body anywhere in their chain: today's token
                return true;
            };
            let (dx, dy) = ((party.0 - npc.0) as i32, (party.1 - npc.1) as i32);
            let want = Facing::default().toward(dx, dy);
            let facing = view_name(want.view);
            let view = drawn_view(cx, &s(json::dict(&them), "body"), "idle", facing);
            assert_eq!(them["clip"].as_str().map(|c| c.starts_with("idle")), Some(true), "{npc:?}");
            assert_eq!((them["facing"].clone(), them["view"].clone()), (json!(facing), json!(view)), "{} at {npc:?} talking with the party at {party:?}", them["id"]);
            assert_eq!(them["left"], json!(view == "side" && want.left), "{} at {npc:?} talking with the party at {party:?}", them["id"]);
            // the leader faces them (from a fresh map: the sheets' right kept on an up or down turn)
            let lead = sprites.iter().find(|s| s["id"] == "party").expect("the leader");
            let lead_want = Facing::default().toward(-dx, -dy);
            assert_eq!(map.leader_face, lead_want, "the leader faces {npc:?} from {party:?}");
            assert_eq!(lead["facing"], json!(view_name(lead_want.view)), "the leader faces {npc:?} from {party:?}");
            if dy.abs() > dx.abs() {
                sides.2 += 1;
            } else if want.left {
                sides.0 += 1;
            } else {
                sides.1 += 1;
            }
            map.svc.end(cx);
            frame(map, cx, &mut g, 1.0);
            let after = sprite(map, cx, them["id"].as_str().unwrap_or("")).expect("still drawn");
            // NPCFACE1: out of the conversation back to the row's facing (down by default)
            let loc = cx.data.resolve(&cx.gs().world_id, "locations", &cx.gs().location_id);
            let t = explore::template_of(&cx.data, loc);
            let steps = cx.gs().explore_int("loc_steps");
            let rest = explore::markers_of(loc).iter().find(|m| explore::marker_pos(t, m, steps) == npc).map(explore::marker_facing).expect("their marker");
            assert_eq!((after["left"].clone(), after["facing"].clone()), (json!(rest.left), json!(view_name(rest.view))), "out of the conversation they rest as their row says");
            true
        });
        println!("conversations checked (left, right, down or up): {sides:?}");
        assert!(n > 0, "someone on a map of the world talks");
        assert!(sides.0 > 0 && sides.1 > 0 && sides.2 > 0, "talked to from both sides and from above or below: {sides:?}");
    }

    /// NPCFACE1 under both region views: a person with no `face` draws down, a row's `face` draws that way, a bad value
    /// (a `content check` problem) still draws down, a body with no down row (beast, crawler, dummy) keeps its side
    /// view, and the party is untouched.
    #[test]
    fn people_rest_facing_down_unless_their_row_says() {
        use crate::screens::Game;
        use emberwake_world::{new_game, Data};
        // (marker id, its art, its face, the drawn `facing`, the drawn `view`, mirrored)
        let cases: [(&str, &str, Option<Value>, &str, &str, bool); 8] = [
            ("plain", "warrior", None, "down", "down", false),
            ("up", "warrior", Some(json!("up")), "up", "up", false),
            ("left", "warrior", Some(json!("left")), "side", "side", true),
            ("right", "warrior", Some(json!("right")), "side", "side", false),
            ("bad", "warrior", Some(json!("sideways")), "down", "down", false),
            ("beast", "beast", None, "down", "side", false),
            ("crawler", "crawler", None, "down", "side", false),
            ("dummy", "dummy", None, "down", "side", false),
        ];
        let at = |i: usize| (2 + 2 * i as i64, 2);
        let anchors: Vec<Value> = cases.iter().enumerate().map(|(i, c)| json!({"id": c.0, "x": at(i).0, "y": at(i).1})).chain([json!({"id": "spawn", "x": 2, "y": 4})]).collect();
        let markers: Vec<Value> = cases.iter().map(|c| {
            let mut m = json!({"id": c.0, "kind": "npc", "anchor": c.0, "character": c.0});
            if let Some(f) = &c.2 {
                m["face"] = f.clone();
            }
            m
        }).collect();
        let owned = |id: &str, mut r: Value| {
            r["_world"] = json!("yard");
            r["_qid"] = json!(format!("yard:{id}"));
            r
        };
        let people: Vec<Value> = cases.iter().map(|c| owned(c.0, json!({"id": c.0, "name": c.0, "role": "npc", "art": c.1}))).collect();
        let worlds = json!({"yard": {"manifest": {"id": "yard", "name": "Yard", "start_location": "hub", "start_party": ["hero"]}, "own": {
            "locations": [owned("hub", json!({"id": "hub", "name": "Hub", "map": "yard_map", "markers": markers}))], "characters": people}}});
        let catalog = json!({"classes": [{"id": "fighter", "name": "Fighter", "art": "warrior"}], "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]});
        let rows = vec![".".repeat(24); 8];
        let data = std::sync::Arc::new(Data::from_values(&catalog, &worlds, &json!({"maps": [{"id": "yard_map", "rows": rows, "anchors": anchors}]})));
        for (view, _) in mapview::VIEWS {
            let (mut cx, mut g) = (art_cx(), gfx());
            let mut gs = new_game::create_for_world(&data, "yard", 1).expect("a new game");
            explore::enter(&data, &mut gs, "hub", explore::SPAWN);
            cx.app.data = data.clone();
            cx.app.game = Some(Game::new(gs, -1, true));
            cx.settings.v["region_view"] = json!(view);
            cx.settings.v["map_legend_open"] = json!(false);
            let mut map = RegionMap::new(&mut cx);
            frame(&mut map, &mut cx, &mut g, 1.0);
            let sprites = map.state(&cx)["sprites"].as_array().cloned().unwrap_or_default();
            for (i, c) in cases.iter().enumerate() {
                let sp = sprites.iter().find(|s| s["id"] != "party" && stands_on(&map, rect_of(s)) == at(i)).unwrap_or_else(|| panic!("{} is drawn in {view}: {sprites:?}", c.0));
                assert_eq!((sp["facing"].clone(), sp["view"].clone(), sp["left"].clone()), (json!(c.3), json!(c.4), json!(c.5)), "{} in {view}", c.0);
            }
            let party = sprites.iter().find(|s| s["id"] == "party").expect("the leader");
            assert_eq!(party["facing"], json!("side"), "the party keeps its own facing in {view}");
        }
    }

    /// 2.4: on every map of the world each person whose body chain loads is a sprite, and the sprites draw in row
    /// order (lower rows cover upper ones); the names sit over the sprites' heads.
    #[test]
    fn sprites_draw_in_row_order_with_names_above_their_heads() {
        let (mut cx, mut g) = (art_cx(), gfx());
        cx.settings.v["map_names"] = json!("always");
        cx.settings.v["map_legend_open"] = json!(false);
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        let mut people = 0;
        for loc in d.merged_entries(&world, "locations") {
            let t = explore::template_of(&d, loc).clone();
            if t.is_empty() {
                continue;
            }
            cx.gs_mut().location_id = s(loc, "id");
            let _ = RegionMap::new(&mut cx);
            let here = cx.gs().clone();
            // the party on an open tile next to each person in turn (the camera follows it)
            let spots: Vec<Pos> = explore::markers_of(loc)
                .iter()
                .filter(|m| s(m, "kind") == "npc")
                .map(|m| explore::marker_pos(&t, m, here.explore_int("loc_steps")))
                .filter_map(|at| explore::DIRS.iter().map(|d| (at.0 + d.0, at.1 + d.1)).find(|p| explore::is_open(&t, p.0, p.1)))
                .collect();
            for spot in spots {
            *cx.gs_mut() = here.clone();
            let mut map = RegionMap::new(&mut cx);
            cx.gs_mut().position = spot;
            let recs = frame(&mut map, &mut cx, &mut g, 1.0);
            let sprites = map.state(&cx)["sprites"].as_array().cloned().unwrap_or_default();
            let feet: Vec<i32> = sprites.iter().map(|s| rect_of(s).y + rect_of(s).h).collect();
            assert!(feet.windows(2).all(|w| w[0] <= w[1]), "{}: sprites back to front by their feet: {feet:?}", s(loc, "id"));
            let gs = cx.gs().clone();
            for wd in map.widgets().iter().filter(|w| w["kind"] == "marker") {
                let id = s(json::dict(wd), "id");
                let Some(m) = explore::markers_of(loc).into_iter().find(|m| format!("marker.{}", s(m, "id")) == id) else { continue };
                if s(&m, "kind") != "npc" {
                    continue;
                }
                let who = d.resolve(&world, "characters", &s(&m, "character"));
                let (body, _, _) = crate::screens::character_art(&d, who);
                let has = cx.art.first_body(&emberwake_content::art::body_chain(&body)).is_some();
                let sp = sprite(&map, &cx, &id);
                assert_eq!(sp.is_some(), has, "{id}: a sprite when the body chain of '{body}' loads");
                let Some(sp) = sp else { continue };
                people += 1;
                let r = rect_of(&sp);
                let tr = tile_rect(&map, explore::marker_pos(&t, &m, gs.explore_int("loc_steps")));
                assert_eq!((r.y + r.h, r.x + r.w / 2), (tr.y + tr.h, tr.x + tr.w / 2), "{id}: bottom-centred on its tile");
                let label = s(json::dict(wd), "text");
                let named = recs.iter().any(|t| t.text == label && t.y + t.h <= r.y as f32 + 1.0 && t.x <= (r.x + r.w) as f32 && t.x + t.w >= r.x as f32);
                let inside = r.y > HUD_H + 14;
                assert!(named || !inside, "{id}: '{label}' above the sprite's head {r:?} at {:?}", gs.position);
            }
            }
        }
        assert!(people > 0, "a person with a body on a map of the world");
    }

    /// The idle offset is a pure function of the id (the same person always idles the same way) and spreads the ids.
    #[test]
    fn idle_offsets_are_stable_per_person() {
        let ids = ["mara", "tev", "hest", "brask_toller", "orsa_tallow", "wenn_tallow"];
        for id in ids {
            assert_eq!(idle_offset(id), idle_offset(id), "{id}");
            assert!(idle_offset(id) < IDLE_SPREAD_MS, "{id}");
        }
        let mut seen: Vec<u64> = ids.iter().map(|id| idle_offset(id)).collect();
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), ids.len(), "distinct ids, distinct offsets: {seen:?}");
    }

    /// Each person's idle plays at the clock plus their offset: on every map of the world (the party next to each
    /// person) every person's sprite shows the idle frame at that time, and somewhere two people with different ids
    /// show different frames at the same clock time.
    #[test]
    fn people_idle_out_of_step() {
        let (mut cx, mut g) = (art_cx(), gfx());
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        let mut apart = false;
        let mut checked = 0;
        for loc in d.merged_entries(&world, "locations") {
            let t = explore::template_of(&d, loc).clone();
            if t.is_empty() {
                continue;
            }
            cx.gs_mut().location_id = s(loc, "id");
            let _ = RegionMap::new(&mut cx);
            let here = cx.gs().clone();
            let steps = here.explore_int("loc_steps");
            let spots: Vec<Pos> = explore::markers_of(loc)
                .iter()
                .filter(|m| s(m, "kind") == "npc")
                .map(|m| explore::marker_pos(&t, m, steps))
                .filter_map(|at| explore::DIRS.iter().map(|d| (at.0 + d.0, at.1 + d.1)).find(|p| explore::is_open(&t, p.0, p.1)))
                .collect();
            for spot in spots {
                *cx.gs_mut() = here.clone();
                let mut map = RegionMap::new(&mut cx);
                cx.gs_mut().position = spot;
                map.anim_ms = 1000.0;
                frame(&mut map, &mut cx, &mut g, 1.0);
                // (character id, frame shown) of each person drawn
                let mut shown: Vec<(String, i64)> = Vec::new();
                for m in explore::markers_of(loc).into_iter().filter(|m| s(m, "kind") == "npc") {
                    let Some(sp) = sprite(&map, &cx, &format!("marker.{}", s(&m, "id"))) else { continue };
                    let who = d.resolve(&world, "characters", &s(&m, "character"));
                    let id = s(who, "id");
                    let body = s(json::dict(&sp), "body");
                    let want = cx.art.clips(&body).and_then(|c| c.frame_at("idle", 1000 + idle_offset(&id))).expect("an idle frame");
                    assert_eq!(sp["frame"], json!(want.frame), "{id} at clock 1000 with offset {}", idle_offset(&id));
                    checked += 1;
                    shown.push((id, sp["frame"].as_i64().unwrap_or(-1)));
                }
                apart |= shown.iter().any(|a| shown.iter().any(|b| a.0 != b.0 && a.1 != b.1));
            }
        }
        assert!(checked > 1, "people drawn: {checked}");
        assert!(apart, "two people with different ids at different idle frames");
    }

    /// DIR4-M: a scratch body `key` (its `body_dir`) under `root` with clips `ids` (one 8 x 8 frame each, 100 ms, looping),
    /// each `<of>_<view>` id whose side clip is listed a view row of it; every row a lopsided bar (so a mirror shows)
    /// in its own colour (so the rows differ).
    fn scratch_body(root: &std::path::Path, key: &str, ids: &[&str]) {
        const F: u32 = 8;
        let dir = root.join(aegis_core::assets::body_dir(key));
        std::fs::create_dir_all(&dir).unwrap();
        let mut px = vec![0u8; (F * F * ids.len() as u32 * 4) as usize];
        for row in 0..ids.len() as u32 {
            for x in [1u32, 2, 3, 6] {
                let i = (((row * F + 4) * F + x) * 4) as usize;
                px[i..i + 4].copy_from_slice(&[40 + 30 * row as u8, 140, 60, 255]);
            }
        }
        let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(dir.join("sheet.png")).unwrap()), F, F * ids.len() as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&px).unwrap();
        let clips: Vec<Value> = ids
            .iter()
            .map(|id| {
                let mut c = json!({"id": id, "frames": 1, "ms": [100], "loop": true});
                for v in ["up", "down"] {
                    if let Some(of) = id.strip_suffix(&format!("_{v}")).filter(|of| ids.contains(of)) {
                        c["of"] = json!(of);
                        c["view"] = json!(v);
                    }
                }
                c
            })
            .collect();
        std::fs::write(dir.join("clips.json"), json!({"frame": [F, F], "clips": clips}).to_string()).unwrap();
    }

    /// The canvas after a frame with the leader in `face`.
    fn canvas_facing(map: &mut RegionMap, cx: &mut Cx, g: &mut Gfx, face: Facing) -> Vec<u8> {
        map.leader_face = face;
        frame(map, cx, g, 1.0);
        g.canvas.px.clone()
    }

    /// Every facing: side, down and up, each right and left.
    fn facings() -> Vec<Facing> {
        [View::Side, View::Down, View::Up].into_iter().flat_map(|view| [false, true].map(|left| Facing { view, left })).collect()
    }

    /// DIR4-M (as DIR4-G check 5): a body without up or down rows (a scratch side body as the leader's, and the
    /// shipped leader's body when it has none) draws the map in any facing pixel for pixel as the side facing with
    /// the same left/right (PLAYUI1-R); a body with the rows draws them facing down or up (never mirrored), so the map
    /// differs from the side facing there.
    #[test]
    fn a_body_without_view_rows_draws_the_side_row_exactly_as_before() {
        let (mut cx, mut g) = (art_cx(), gfx());
        let chain = emberwake_content::art::body_chain(&leader_art(&cx).0);
        let key = chain.first().cloned().expect("the leader has a body chain");
        let same_as_side = |cx: &mut Cx, g: &mut Gfx, what: &str| {
            let mut map = RegionMap::new(cx);
            for f in facings() {
                let want = canvas_facing(&mut map, cx, g, Facing::side(f.left));
                let got = canvas_facing(&mut map, cx, g, f);
                assert!(got == want, "{what}: the map in {f:?} draws as the side facing");
                let sp = sprite(&map, cx, "party").expect("the leader is a sprite");
                assert_eq!((sp["view"].clone(), sp["left"].clone()), (json!("side"), json!(f.left)), "{what} in {f:?}");
            }
        };
        // the shipped leader's body, when it has no view rows yet
        let shipped = leader_body(&mut cx).expect("the leader has a body");
        if cx.art.clips(&shipped).is_some_and(|c| c.clips.iter().all(|c| c.view == View::Side)) {
            same_as_side(&mut cx, &mut g, &shipped);
        } else {
            println!("the shipped leader body {shipped} has view rows: checked by the scratch bodies only");
        }
        // scratch bodies as the leader's: side clips only, then with front and back rows
        let root = aegis_core::scratch::Scratch::vacant("emberwake_dir4m");
        let side = root.join("side");
        scratch_body(&side, &key, &["idle", "walk"]);
        cx.art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![side.clone()]));
        assert_eq!(cx.art.body_problems(&key), Vec::<String>::new(), "the scratch side body loads");
        same_as_side(&mut cx, &mut g, "the scratch side body");
        let views = root.join("views");
        scratch_body(&views, &key, &["idle", "walk", "idle_down", "idle_up", "walk_down", "walk_up"]);
        cx.art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![views.clone()]));
        assert_eq!(cx.art.body_problems(&key), Vec::<String>::new(), "the scratch view body loads");
        let mut map = RegionMap::new(&mut cx);
        for f in facings().into_iter().filter(|f| f.view != View::Side) {
            let side_px = canvas_facing(&mut map, &mut cx, &mut g, Facing::side(f.left));
            let got = canvas_facing(&mut map, &mut cx, &mut g, f);
            let sp = sprite(&map, &cx, "party").expect("the leader is a sprite");
            let name = view_name(f.view);
            assert_eq!((sp["clip"].clone(), sp["view"].clone(), sp["left"].clone()), (json!(format!("idle_{name}")), json!(name), json!(false)), "in {f:?}");
            assert!(got != side_px, "the {name} row differs from the side row on the map ({f:?})");
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// 2.4 fallback: with no body (the art layers off) the party is today's token, and no sprite is drawn.
    #[test]
    fn without_a_body_the_party_is_todays_token() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(map.state(&cx)["sprites"], json!([]));
        let t = tile_rect(&map, cx.gs().position);
        // the token's ring in the party colour on the party's tile
        let near = |px: &[u8]| px.iter().zip(crate::props::PARTY.iter()).all(|(a, b)| (*a as i32 - *b as i32).abs() <= 24);
        let ring = (t.y..t.y + t.h).flat_map(|y| (t.x..t.x + t.w).map(move |x| (x, y))).filter(|&(x, y)| near(&g.canvas.px[((y * g.canvas.w + x) * 4) as usize..][..3])).count();
        assert!(ring >= 8, "the token's ring on the party's tile: {ring} pixels");
    }
}

/// MAPNAV1-U (spec DESIGN_REVIEW2 6 A, user 2026-11-02): edge arrows toward the legend's markers out of view, a legend
/// row or an arrow pans the camera there, a tile click walks there or says it can't, and Q or a click on the leader
/// passes the lead along the standing active members (saved as `explore.leader`). Drawn headless over a debug game of
/// the first playable world; expectations come from the map, its markers and the camera rule, never shipped numbers.
#[cfg(test)]
mod nav_tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx};

    fn area() -> Rect {
        Rect::new(0, HUD_H, W, H - HUD_H)
    }

    /// The tile `at`'s rect through the last frame's camera.
    fn tile_rect(map: &RegionMap, at: Pos) -> Rect {
        map.geo.cell_of(at)
    }

    /// The legend's markers (present, spent or locked, not a bare point) with a map place: (id, tile).
    fn listed(cx: &Cx) -> Vec<(String, Pos)> {
        let gs = cx.gs();
        let loc = explore::location(&cx.data, gs);
        let t = explore::template_of(&cx.data, loc);
        explore::markers_of(loc)
            .into_iter()
            .filter(|m| s(&explore::marker_view(&cx.data, gs, loc, m), "state") != "absent" && s(m, "kind") != "point")
            .map(|m| (s(&m, "id"), explore::marker_pos(t, &m, gs.explore_int("loc_steps"))))
            .filter(|(_, at)| at.0 >= 0)
            .collect()
    }

    fn arrow_ids(map: &RegionMap, cx: &Cx) -> Vec<String> {
        let mut v: Vec<String> = map.state(cx)["arrows"].as_array().cloned().unwrap_or_default().iter().map(|a| s(json::dict(a), "id")).collect();
        v.sort();
        v
    }

    /// Every map location of the world, the party on two opposite corners: exactly the legend's markers whose tile is
    /// out of the map area have an arrow; each sits inside the area near its edge and points at its marker.
    #[test]
    fn an_off_screen_marker_gets_an_edge_arrow_and_an_on_screen_one_none() {
        // EXPLOREVIEW1: under both region views
        mapview::VIEWS.iter().for_each(|v| an_off_screen_marker_gets_an_edge_arrow_and_an_on_screen_one_none_in(v.0));
    }

    fn an_off_screen_marker_gets_an_edge_arrow_and_an_on_screen_one_none_in(view: &str) {
        let (mut cx, mut g) = (game_cx(), gfx());
        cx.settings.v["region_view"] = json!(view);
        cx.settings.v["map_legend_open"] = json!(false);
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        let (mut arrows, mut in_view) = (0, 0);
        for loc in d.merged_entries(&world, "locations") {
            let t = explore::template_of(&d, loc);
            if t.is_empty() {
                continue;
            }
            let (w, h) = explore::size_of(t);
            for corner in [(0, 0), (w - 1, h - 1)] {
                cx.gs_mut().location_id = s(loc, "id");
                let mut map = RegionMap::new(&mut cx);
                cx.gs_mut().position = corner;
                frame(&mut map, &mut cx, &mut g, 1.0);
                let a = area();
                let mut want: Vec<String> = Vec::new();
                for (id, at) in listed(&cx) {
                    let seen = tile_rect(&map, at).intersect(a);
                    if seen.w <= 0 || seen.h <= 0 {
                        want.push(id);
                    } else {
                        in_view += 1;
                    }
                }
                want.sort();
                assert_eq!(arrow_ids(&map, &cx), want, "{} at {corner:?}", s(loc, "id"));
                let state = map.state(&cx);
                let spots: Vec<(i64, i64)> = state["arrows"].as_array().cloned().unwrap_or_default().iter().map(|v| (v["at"][0].as_i64().unwrap(), v["at"][1].as_i64().unwrap())).collect();
                for (i, p) in spots.iter().enumerate() {
                    assert!(spots[..i].iter().all(|o| (o.0 - p.0).abs() >= ARROW as i64 || (o.1 - p.1).abs() >= ARROW as i64), "{} at {corner:?}: arrows stacked: {spots:?}", s(loc, "id"));
                }
                for arrow in state["arrows"].as_array().cloned().unwrap_or_default() {
                    let id = s(json::dict(&arrow), "id");
                    let (x, y) = (arrow["at"][0].as_i64().unwrap() as i32, arrow["at"][1].as_i64().unwrap() as i32);
                    assert!(a.contains(x, y), "{id}'s arrow inside the map area: ({x}, {y})");
                    let edge = (x - a.x).min(a.x + a.w - x).min(y - a.y).min(a.y + a.h - y);
                    assert!(edge <= ARROW + 1, "{id}'s arrow at the edge: {edge} px in");
                    let at = listed(&cx).into_iter().find(|m| m.0 == id).unwrap().1;
                    let r = tile_rect(&map, at);
                    let (tx, ty) = (r.x as f64 + r.w as f64 / 2.0 - x as f64, r.y as f64 + r.h as f64 / 2.0 - y as f64);
                    let (ux, uy) = (arrow["dir"][0].as_f64().unwrap(), arrow["dir"][1].as_f64().unwrap());
                    let cos = (tx * ux + ty * uy) / (tx * tx + ty * ty).sqrt();
                    assert!(cos > 0.95, "{id}'s arrow points at its marker: cos {cos}");
                    let wd = map.widgets().into_iter().find(|w| w["id"] == format!("arrow.{id}")).expect("the arrow's widget");
                    assert_eq!(wd["kind"], json!("arrow"));
                    arrows += 1;
                }
            }
        }
        assert!(arrows > 0 && in_view > 0, "the world's maps show both cases ({arrows} arrows, {in_view} markers in view)");
    }

    /// A legend row pans the camera to its marker (the marker's tile at the map area's centre) and its arrow goes; an
    /// arrow pans the same way; Esc first brings the camera back to the party; a second row click walks there.
    #[test]
    fn a_legend_row_or_an_arrow_pans_the_camera_to_its_marker() {
        // EXPLOREVIEW1: under both region views
        mapview::VIEWS.iter().for_each(|v| a_legend_row_or_an_arrow_pans_the_camera_to_its_marker_in(v.0));
    }

    fn a_legend_row_or_an_arrow_pans_the_camera_to_its_marker_in(view: &str) {
        let (mut cx, mut g) = (game_cx(), gfx());
        cx.settings.v["region_view"] = json!(view);
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let pos = cx.gs().position;
        let dist = |at: &Pos| (at.0 - pos.0).abs() + (at.1 - pos.1).abs();
        let (id, at) = listed(&cx).into_iter().filter(|(_, at)| dist(at) > 1).max_by_key(|(_, at)| dist(at)).expect("a marker away from the party");
        map.click_id(&mut cx, &format!("here.{id}")).expect("the row");
        assert_eq!(map.state(&cx)["pan"], json!([at.0, at.1]));
        assert_eq!(map.state(&cx)["walking"], json!(false), "the first click only looks");
        frame(&mut map, &mut cx, &mut g, 1.0);
        map.widgets().into_iter().find(|w| w["id"] == format!("marker.{id}")).expect("the marker in view");
        let a = area();
        let (cols, rows) = explore::size_of(explore::template_of(&cx.data, explore::location(&cx.data, cx.gs())));
        let (cw, ch) = map.geo.cell_size();
        let want = mapview::camera_px(map.geo.cell_size(), a, (cols as i32, rows as i32), (at.0 as f32, at.1 as f32), 1.0);
        assert_eq!((map.geo.ox, map.geo.oy), want, "the camera looks there, stopped by the map's edges");
        // on the area's centre, unless an edge stopped it: then the camera stands on its margin
        let r = tile_rect(&map, at);
        let centred = (r.x + r.w / 2, r.y + r.h / 2) == (a.x + a.w / 2, a.y + a.h / 2);
        let m = mapview::MARGIN_CELLS;
        let on_margin = [map.geo.ox == a.x + m * cw, map.geo.ox + cols as i32 * cw == a.x + a.w - m * cw, map.geo.oy == a.y + m * ch, map.geo.oy + rows as i32 * ch == a.y + a.h - m * ch].contains(&true);
        assert!(centred || on_margin, "the marker at the centre or the camera on a margin");
        assert!(!arrow_ids(&map, &cx).contains(&id), "no arrow for a marker in view");
        // a pan never carries into another location
        let here = cx.gs().location_id.clone();
        cx.gs_mut().location_id = format!("{here}_elsewhere");
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert!(map.pan.is_none(), "another location drops the pan");
        cx.gs_mut().location_id = here;
        frame(&mut map, &mut cx, &mut g, 1.0);
        map.click_id(&mut cx, &format!("here.{id}")).expect("the row");
        frame(&mut map, &mut cx, &mut g, 1.0);
        // Esc recentres before the game menu
        map.back(&mut cx);
        assert!(cx.nav.is_empty() && map.state(&cx)["pan"].is_null(), "Esc recentres, no menu");
        frame(&mut map, &mut cx, &mut g, 1.0);
        // an arrow pans the same way (when the party's view has one)
        if let Some(aid) = arrow_ids(&map, &cx).first().cloned() {
            map.click_id(&mut cx, &format!("arrow.{aid}")).expect("the arrow");
            let want = listed(&cx).into_iter().find(|m| m.0 == aid).unwrap().1;
            assert_eq!(map.state(&cx)["pan"], json!([want.0, want.1]));
            frame(&mut map, &mut cx, &mut g, 1.0);
            assert!(!arrow_ids(&map, &cx).contains(&aid), "its marker is in view now");
        }
        // the row again while the camera looks there walks (when a path is open)
        map.click_id(&mut cx, &format!("here.{id}")).expect("the row");
        frame(&mut map, &mut cx, &mut g, 1.0);
        let m = explore::markers_of(explore::location(&cx.data, cx.gs())).into_iter().find(|m| s(m, "id") == id).unwrap();
        let solid = ["npc", "chest", "lore"].contains(&s(&m, "kind").as_str());
        if map.path_to(&cx, at, solid).is_some() {
            map.click_id(&mut cx, &format!("here.{id}")).expect("the row again");
            assert_eq!(map.state(&cx)["walking"], json!(true), "the second click walks to {id}");
        }
    }

    /// Near the party: (a free tile two or more steps away by an open path and the path's length, a closed tile).
    fn targets(map: &RegionMap, cx: &Cx) -> (Option<(Pos, usize)>, Option<Pos>) {
        let gs = cx.gs();
        let loc = explore::location(&cx.data, gs);
        let t = explore::template_of(&cx.data, loc);
        let marked: Vec<Pos> = explore::markers_of(loc).iter().map(|m| explore::marker_pos(t, m, gs.explore_int("loc_steps"))).collect();
        let near: Vec<Pos> = (-4..=4).flat_map(|dy| (-4..=4).map(move |dx| (dx, dy))).map(|(dx, dy)| (gs.position.0 + dx, gs.position.1 + dy)).filter(|p| !marked.contains(p) && *p != gs.position).collect();
        let open = near.iter().filter(|p| explore::is_open(t, p.0, p.1)).find_map(|p| map.path_to(cx, *p, false).filter(|path| path.len() >= 2).map(|path| (*p, path.len())));
        let closed = near.iter().copied().find(|p| !explore::is_open(t, p.0, p.1));
        (open, closed)
    }

    fn click_tile(map: &mut RegionMap, cx: &mut Cx, g: &mut Gfx, at: Pos) {
        let r = tile_rect(map, at);
        let pt = (r.x + r.w / 2, r.y + r.h / 2);
        map.draw(cx, g, pt, true);
        assert!(map.click(cx, pt), "the map takes the click on {at:?}");
    }

    /// A click on a free tile walks the open path there (the leader walks while it plays); a click on a closed tile
    /// walks nowhere and says so.
    #[test]
    fn a_tile_click_walks_to_a_reachable_tile_and_refuses_a_blocked_one() {
        // EXPLOREVIEW1: under both region views
        mapview::VIEWS.iter().for_each(|v| a_tile_click_walks_to_a_reachable_tile_and_refuses_a_blocked_one_in(v.0));
    }

    fn a_tile_click_walks_to_a_reachable_tile_and_refuses_a_blocked_one_in(view: &str) {
        let (mut cx, mut g) = (game_cx(), gfx());
        cx.settings.v["region_view"] = json!(view);
        cx.settings.v["map_legend_open"] = json!(false);
        let mut map = RegionMap::new(&mut cx);
        // no random fight in the few steps
        cx.gs_mut().explore.insert("next_at".into(), json!(99));
        cx.gs_mut().explore.insert("steps".into(), json!(0));
        frame(&mut map, &mut cx, &mut g, 1.0);
        let (open, closed) = targets(&map, &cx);
        let (to, steps) = open.expect("a free tile two steps or more away");
        click_tile(&mut map, &mut cx, &mut g, to);
        assert_eq!(map.walk.len(), steps, "the walk is the open path");
        assert!(map.moving(), "the leader walks while the path plays");
        for _ in 0..steps {
            map.next_walk = Instant::now();
            map.tick(&mut cx);
        }
        assert_eq!(cx.gs().position, to, "the walk reaches {to:?}: {:?}", cx.log);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let wall = closed.or_else(|| targets(&map, &cx).1).expect("a closed tile near the party");
        let at = cx.gs().position;
        click_tile(&mut map, &mut cx, &mut g, wall);
        assert!(map.walk.is_empty() && cx.gs().position == at, "no walk onto {wall:?}");
        assert_eq!(cx.log.last().map(String::as_str), Some("You can't get there."));
    }

    /// A party of three (test-only members beside the first; the middle one downed): Q and a click on the leader pass
    /// the lead to the next standing member, wrapping and skipping the downed one; the lead is in the state, the Party
    /// button opens on it, and it survives a save and a load.
    #[test]
    fn the_lead_cycles_the_standing_active_members_and_survives_save_and_load() {
        // EXPLOREVIEW1: under both region views
        mapview::VIEWS.iter().for_each(|v| the_lead_cycles_the_standing_active_members_and_survives_save_and_load_in(v.0));
    }

    fn the_lead_cycles_the_standing_active_members_and_survives_save_and_load_in(view: &str) {
        let (mut cx, mut g) = (game_cx(), gfx());
        cx.settings.v["region_view"] = json!(view);
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        {
            let gs = cx.gs_mut();
            gs.party.members.truncate(1);
            gs.party.active_ids.clear();
            let first = gs.party.members[0].clone();
            for id in ["test_down", "test_up"] {
                let mut m = first.clone();
                m.id = id.into();
                gs.party.members.push(m);
            }
        }
        // after `resume` (a rest place heals on arrival): the middle member is down
        let mut map = RegionMap::new(&mut cx);
        let max = emberwake_world::progression::max_hp_of(&d, cx.gs().party.member("test_down").unwrap(), &world);
        assert!(max > 0);
        cx.gs_mut().party.member_mut("test_down").unwrap().hp_lost = max;
        let active: Vec<String> = cx.gs().party.active_members().iter().map(|m| m.id.clone()).collect();
        let up: Vec<String> = active.iter().filter(|i| *i != "test_down").cloned().collect();
        assert!(up.len() >= 2 && active.contains(&"test_down".to_string()), "the cap keeps three active: {active:?}");
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(map.state(&cx)["leader"], json!(up[0]), "no lead chosen: the first active member");
        let mut seen = Vec::new();
        for i in 0..up.len() * 2 {
            if i % 2 == 0 {
                map.key(&mut cx, Key::Char('q'));
            } else {
                frame(&mut map, &mut cx, &mut g, 1.0);
                map.click_id(&mut cx, "party").expect("a click on the leader");
            }
            seen.push(s(json::dict(&map.state(&cx)), "leader"));
        }
        let want: Vec<String> = (1..=up.len() * 2).map(|i| up[i % up.len()].clone()).collect();
        assert_eq!(seen, want, "the lead passes along the standing members and wraps");
        assert!(cx.log.last().is_some_and(|l| l.ends_with("leads the party.")), "{:?}", cx.log.last());
        map.key(&mut cx, Key::Char('q'));
        let lead = s(json::dict(&map.state(&cx)), "leader");
        assert_ne!(lead, up[0]);
        // the Party button opens on the leader
        frame(&mut map, &mut cx, &mut g, 1.0);
        map.click_id(&mut cx, "hud.party").expect("party");
        let opened = match cx.nav.last() {
            Some(Nav::Push(s)) => s.state(&cx),
            _ => Value::Null,
        };
        cx.nav.clear();
        assert_eq!(opened["member"], json!(lead));
        // saved with the game
        let dir = cx.shot_dir.join("lead_saves");
        let gs = cx.gs().clone();
        assert!(emberwake_world::save::write(&d, &dir, &gs, 1, 7).is_empty());
        let back = emberwake_world::save::read(&d, &dir, 1).expect("reads back");
        assert_eq!(back.state.explore_leader(standing(&d, &world)).map(|m| m.id.clone()), Some(lead.clone()));
        assert_eq!(back.state.explore[emberwake_world::state::LEADER_KEY], json!(lead));
        // one standing member left: no one else can lead (the leader takes no click)
        for id in up.iter().filter(|i| **i != up[0]) {
            cx.gs_mut().party.member_mut(id).unwrap().hp_lost = max;
        }
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(map.state(&cx)["leader"], json!(up[0]));
        assert!(map.click_id(&mut cx, "party").is_err(), "no lead to pass");
    }
}

/// EXPLOREVIEW1 section 4 on the drawn map (the geometry's own checks are `mapview.rs`'; MAPNAV1-U, the talk camera, the
/// sprites and the text audit run under both views above). Sites are found by rule on the world's maps; drawn colours
/// are the no-art stand-ins or scratch art.
#[cfg(test)]
mod view_tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx, rect};
    use std::path::Path;

    const WALL_FACE: [u8; 3] = [30, 45, 38];
    const WALL_CAP: [u8; 3] = [40, 60, 50];

    fn px(g: &Gfx, x: i32, y: i32) -> [u8; 3] {
        let i = ((y * g.canvas.w + x) * 4) as usize;
        [g.canvas.px[i], g.canvas.px[i + 1], g.canvas.px[i + 2]]
    }

    /// Nothing but the map's own tiles stands within two cells of (x, y): no marker, no prop.
    fn clear(t: &Dict, loc: &Dict, x: i64, y: i64) -> bool {
        let markers = explore::markers_of(loc).iter().all(|m| {
            let at = explore::marker_pos(t, m, 0);
            (at.0 - x).abs() > 2 || (at.1 - y).abs() > 2
        });
        let props = json::aget(t, "props").iter().all(|raw| {
            let p = json::dict(raw);
            let (px, py) = (json::i(p, "x", 0), json::i(p, "y", 0));
            let (w, h) = (json::i(p, "w", 1), json::i(p, "h", 1));
            x + 2 < px || x - 2 >= px + w || y + 2 < py || y - 2 >= py + h
        });
        markers && props
    }

    /// The first (location id, cell) of the world's maps that passes `want(template, x, y)` and stands clear.
    fn site(cx: &Cx, want: impl Fn(&Dict, i64, i64) -> bool) -> (String, Pos) {
        let d = cx.data.clone();
        for loc in d.merged_entries(&cx.gs().world_id, "locations") {
            let t = explore::template_of(&d, loc);
            if t.is_empty() {
                continue;
            }
            let (w, h) = explore::size_of(t);
            for y in 0..h {
                for x in 0..w {
                    if want(t, x, y) && clear(t, loc, x, y) {
                        return (s(loc, "id"), (x, y));
                    }
                }
            }
        }
        panic!("no such place on the world's maps");
    }

    fn ch(t: &Dict, x: i64, y: i64) -> char {
        explore::tile(t, x, y).unwrap_or('#')
    }

    /// A frame of location `loc` with the camera on `at` and the party on an open cell four or more cells off it.
    fn look_at(cx: &mut Cx, g: &mut Gfx, loc: &str, at: Pos) -> RegionMap {
        cx.gs_mut().location_id = loc.to_string();
        let mut map = RegionMap::new(cx);
        let d = cx.data.clone();
        let t = explore::template_of(&d, explore::location(&d, cx.gs())).clone();
        let (w, h) = explore::size_of(&t);
        let far = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).find(|p| explore::is_open(&t, p.0, p.1) && ((p.0 - at.0).abs() >= 4 || (p.1 - at.1).abs() >= 4)).expect("an open cell away");
        cx.gs_mut().position = far;
        map.pan = Some(at);
        map.pan_loc = loc.to_string();
        cx.settings.v["map_legend_open"] = json!(false);
        frame(&mut map, cx, g, 1.0);
        map
    }

    /// Check 1: with `region_view` missing the frame is the square one, to the pixel and the widget.
    #[test]
    fn a_missing_setting_draws_the_square_view() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        assert_eq!(cx.settings.v["region_view"], json!("square"), "the default");
        frame(&mut map, &mut cx, &mut g, 1.0);
        let (pixels, widgets) = (g.canvas.px.clone(), map.widgets());
        assert_eq!(map.state(&cx)["view"], json!("square"));
        if let Some(o) = cx.settings.v.as_object_mut() {
            o.remove("region_view");
        }
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert!(g.canvas.px == pixels, "the same pixels");
        assert_eq!(map.widgets(), widgets);
        assert_eq!(map.state(&cx)["view"], json!("square"));
        cx.settings.v["region_view"] = json!("tilted");
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert!(g.canvas.px != pixels, "the switch shows on the next frame");
        assert_eq!(map.state(&cx)["view"], json!("tilted"));
    }

    /// Check 4: a wall with an open cell south shows its face (darker than its cap), with its top line and the shade on
    /// the cell south; a wall with a wall south shows the next cap there. Square, the same cell is one flat tile.
    #[test]
    fn a_wall_shows_a_face_only_over_an_open_cell() {
        let (mut cx, mut g) = (game_cx(), gfx());
        cx.settings.v["region_view"] = json!("tilted");
        let (loc, at) = site(&cx, |t, x, y| ch(t, x, y) == '#' && ch(t, x, y + 1) == '.' && ch(t, x, y + 2) == '.' && ch(t, x - 1, y) == '#' && ch(t, x + 1, y) == '#');
        let map = look_at(&mut cx, &mut g, &loc, at);
        let c = map.geo.cell_of(at);
        let mid = c.x + c.w / 2;
        assert_eq!(px(&g, mid, c.y + 9), WALL_FACE, "the face");
        assert_eq!(px(&g, mid, c.y), WALL_CAP, "the cap");
        assert!(px(&g, mid, c.y + 9).iter().sum::<u8>() < px(&g, mid, c.y).iter().sum::<u8>(), "the face is darker than the cap");
        assert_eq!(px(&g, mid, c.y + mapview::FACE_H), [24, 24, 30], "the line on the face's top edge");
        let floor = px(&g, mid, c.y + c.h + mapview::SHADE_H + 1);
        let shaded = px(&g, mid, c.y + c.h);
        assert!(shaded.iter().map(|v| *v as i32).sum::<i32>() < floor.iter().map(|v| *v as i32).sum::<i32>(), "the shade on the cell south: {shaded:?} over {floor:?}");
        let (loc2, at2) = site(&cx, |t, x, y| ch(t, x, y) == '#' && ch(t, x, y + 1) == '#' && ch(t, x, y + 2) == '#' && ch(t, x, y - 1) == '#');
        let map = look_at(&mut cx, &mut g, &loc2, at2);
        let c2 = map.geo.cell_of(at2);
        assert_eq!(px(&g, c2.x + c2.w / 2, c2.y + 9), WALL_CAP, "no face: the south wall's cap");
        cx.settings.v["region_view"] = json!("square");
        let map = look_at(&mut cx, &mut g, &loc, at);
        let c = map.geo.cell_of(at);
        assert_eq!(px(&g, c.x + c.w / 2, c.y + 9), WALL_CAP, "square: no face");
    }

    /// Scratch art: `rel` in one colour.
    fn solid(root: &Path, rel: &str, w: u32, h: u32, c: [u8; 3]) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(path).unwrap()), w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let row: Vec<u8> = (0..w * h).flat_map(|_| [c[0], c[1], c[2], 255]).collect();
        enc.write_header().unwrap().write_image_data(&row).unwrap();
    }

    fn scratch(tag: &str) -> aegis_core::scratch::Scratch {
        let root = aegis_core::scratch::Scratch::new(&format!("emberwake_mapview_{tag}"));
        root
    }

    /// 3.2: the face comes from the region sets, then the stage set, else the cap's top rows at 75 %; water gets its lip
    /// under an open north cell.
    #[test]
    fn the_face_art_falls_back_along_its_chain() {
        let (mut cx, mut g) = (game_cx(), gfx());
        cx.settings.v["region_view"] = json!("tilted");
        let world = cx.gs().world_id.clone();
        let (loc, at) = site(&cx, |t, x, y| ch(t, x, y) == '#' && ch(t, x, y + 1) == '.' && ch(t, x - 1, y) == '#' && ch(t, x + 1, y) == '#');
        let root = scratch("face");
        let wall = [200, 100, 40];
        solid(&root, "tilesets/common/wall/v0.png", 32, 32, wall);
        let mut probe = |cx: &mut Cx, root: &Path| {
            cx.art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![root.to_path_buf()]));
            let map = look_at(cx, &mut g, &loc, at);
            let c = map.geo.cell_of(at);
            (px(&g, c.x + c.w / 2, c.y + 9), px(&g, c.x + c.w / 2, c.y))
        };
        assert_eq!(probe(&mut cx, &root), ([150, 75, 30], wall), "no face art: the cap's rows at 75 %");
        solid(&root, "tilesets/common_stage/wall_face/v0.png", 128, 48, [10, 20, 30]);
        assert_eq!(probe(&mut cx, &root).0, [10, 20, 30], "the stage set's face");
        solid(&root, &format!("tilesets/{world}/wall_face/v0.png"), 128, 48, [60, 70, 80]);
        assert_eq!(probe(&mut cx, &root).0, [60, 70, 80], "the region set's face first");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Check 4: the leader north of a wall is behind its cap and gets the outline; on open ground neither.
    #[test]
    fn the_leader_north_of_a_wall_is_behind_its_cap_with_an_outline() {
        let mut cx = game_cx();
        cx.art = aegis_ui::art::Art::new(Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        let mut g = gfx();
        cx.settings.v["region_view"] = json!("tilted");
        let open = |t: &Dict, x: i64, y: i64| ch(t, x, y) == '.';
        let (loc, behind) = site(&cx, |t, x, y| open(t, x, y) && ch(t, x, y + 1) == '#' && !on_border(t, explore::size_of(t), x, y + 1));
        let (loc3, by_ring) = site(&cx, |t, x, y| open(t, x, y) && ch(t, x, y + 1) == '#' && on_border(t, explore::size_of(t), x, y + 1));
        let (loc2, free) = site(&cx, |t, x, y| (-2..=2).all(|dy| (-2..=2).all(|dx| open(t, x + dx, y + dy))));
        let mut outline = |cx: &mut Cx, loc: &str, at: Pos| {
            cx.gs_mut().location_id = loc.to_string();
            let mut map = RegionMap::new(cx);
            cx.gs_mut().position = at;
            frame(&mut map, cx, &mut g, 1.0);
            let c = map.geo.cell_of(at);
            let sp = map.state(cx)["sprites"].as_array().and_then(|l| l.iter().find(|v| v["id"] == "party").cloned()).expect("the leader's sprite");
            let n = |i: usize| sp["rect"][i].as_i64().unwrap_or(0) as i32;
            assert_eq!((n(1) + n(3), n(0) + n(2) / 2), (c.y + c.h, c.x + c.w / 2), "the sprite stands on the party's cell");
            let party = [crate::props::PARTY[0], crate::props::PARTY[1], crate::props::PARTY[2]];
            (n(1) - 1..n(1) + n(3) + 1).flat_map(|y| (n(0) - 1..n(0) + n(2) + 1).map(move |x| (x, y))).filter(|p| px(&g, p.0, p.1) == party).count()
        };
        assert!(outline(&mut cx, &loc, behind) > 0, "the outline over the cap");
        assert_eq!(outline(&mut cx, &loc2, free), 0, "open ground: no outline");
        assert_eq!(outline(&mut cx, &loc3, by_ring), 0, "REGIONVIEW1: the map's outer wall hides nobody");
    }

    /// Check 6: a tilted prop without art is the raised box; the square one stays on its footprint with today's face,
    /// and so does a prop whose art will not load.
    #[test]
    fn a_prop_without_art_draws_the_raised_box() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let foot = Rect::new(100, 100, 160, 72);
        let p = |rect: Rect| PropDraw { id: "x".into(), rect, foot, file: None, flip: false, ground: false, foot_y: foot.y + foot.h };
        let mut ui: Ui<RAct> = Ui::new(&mut g, (0, 0), true);
        draw_prop_art(&mut ui, &mut cx, &p(Rect::new(foot.x, foot.y - mapview::BLOCK_LIFT, foot.w, foot.h + mapview::BLOCK_LIFT)), 4);
        drop(ui);
        let mid = foot.x + foot.w / 2;
        let colour = |c: Rgba| [c[0], c[1], c[2]];
        assert_eq!(px(&g, mid, foot.y - 6), colour(BOX_EDGE), "the raised top edge");
        assert_eq!(px(&g, mid, foot.y - 3), colour(BOX_FILL), "the fill over the footprint");
        assert_eq!(px(&g, mid, foot.y + foot.h - 4), colour(BOX_FACE), "the south face");
        assert_eq!(px(&g, mid, foot.y + foot.h - FACE_H), colour(BOX_FACE), "6 px tall");
        assert_eq!(px(&g, mid, foot.y + foot.h - FACE_H - 1), colour(BOX_FILL));
        // the square box: on its footprint, a quarter of its height at most `BOX_FACE_H` (a raised rect with a file that
        // fails to load draws this face too)
        let plain = (foot.h / 4).clamp(1, BOX_FACE_H);
        assert!(plain < FACE_H, "the two faces differ");
        let unloadable = PropDraw { file: Some("no/such/file.png".into()), ..p(Rect::new(foot.x, foot.y - mapview::BLOCK_LIFT, foot.w, foot.h + mapview::BLOCK_LIFT)) };
        for (drawn, label) in [(p(foot), "square"), (unloadable, "art that will not load")] {
            let mut g = gfx();
            let mut ui: Ui<RAct> = Ui::new(&mut g, (0, 0), true);
            draw_prop_art(&mut ui, &mut cx, &drawn, 4);
            drop(ui);
            assert_eq!(px(&g, mid, drawn.rect.y), colour(BOX_EDGE), "{label}: the top edge");
            assert_eq!(px(&g, mid, foot.y + foot.h - 2), colour(BOX_FACE), "{label}: the south face");
            assert_eq!(px(&g, mid, foot.y + foot.h - plain), colour(BOX_FACE), "{label}: its first row");
            assert_eq!(px(&g, mid, foot.y + foot.h - plain - 1), colour(BOX_FILL), "{label}: the fill above it");
        }
    }

    /// The square guard: every marker's click area is its cell (cut to the map) and they run in the marker list's order,
    /// then the party on its cell.
    #[test]
    fn square_click_areas_stay_on_their_cells_in_list_order() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        let area = Rect::new(0, HUD_H, W, H - HUD_H);
        let mut seen = 0;
        for loc in d.merged_entries(&world, "locations") {
            let t = explore::template_of(&d, loc);
            if t.is_empty() {
                continue;
            }
            cx.gs_mut().location_id = s(loc, "id");
            let mut map = RegionMap::new(&mut cx);
            frame(&mut map, &mut cx, &mut g, 1.0);
            let gs = cx.gs().clone();
            let steps = gs.explore_int("loc_steps");
            let mut want: Vec<(String, (i64, i64, i64, i64))> = Vec::new();
            for raw in json::aget(loc, "markers") {
                let m = explore::marker_full(json::dict(raw));
                let at = explore::marker_pos(t, &m, steps);
                if s(&explore::marker_view(&d, &gs, loc, &m), "state") == "absent" || at.0 < 0 {
                    continue;
                }
                let r = Rect::new(map.geo.ox + at.0 as i32 * 16, map.geo.oy + at.1 as i32 * 16, 16, 16).intersect(area);
                if r.w > 0 && r.h > 0 {
                    want.push((format!("marker.{}", s(&m, "id")), (r.x as i64, r.y as i64, r.w as i64, r.h as i64)));
                }
            }
            seen += want.len();
            let p = Rect::new(map.geo.ox + gs.position.0 as i32 * 16, map.geo.oy + gs.position.1 as i32 * 16, 16, 16);
            want.push(("party".to_string(), (p.x as i64, p.y as i64, p.w as i64, p.h as i64)));
            let got: Vec<(String, (i64, i64, i64, i64))> = map.widgets().iter().filter(|w| w["kind"] == "marker" || w["id"] == "party").map(|w| (s(json::dict(w), "id"), rect(w))).collect();
            assert_eq!(got, want, "{}", s(loc, "id"));
        }
        assert!(seen > 0, "a marker in view on a map of the world");
    }

    /// 3.5: a person's and the party's click areas are their cells in both views: a person's own cell takes the click, a
    /// tall sprite's head and the cell north of them do not; tilted, the areas run in draw order.
    #[test]
    fn click_areas_stay_on_the_cell_in_draw_order() {
        let mut cx = game_cx();
        cx.art = aegis_ui::art::Art::new(Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        let mut g = gfx();
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        let mut checked = 0;
        for loc in d.merged_entries(&world, "locations") {
            let t = explore::template_of(&d, loc);
            if t.is_empty() {
                continue;
            }
            for view in ["square", "tilted"] {
                cx.settings.v["region_view"] = json!(view);
                cx.gs_mut().location_id = s(loc, "id");
                let mut map = RegionMap::new(&mut cx);
                frame(&mut map, &mut cx, &mut g, 1.0);
                let party = rect(&map.widgets().into_iter().find(|w| w["id"] == "party").expect("the party"));
                let cell = map.geo.cell_of(cx.gs().position);
                assert_eq!(party, (cell.x as i64, cell.y as i64, cell.w as i64, cell.h as i64), "{view}: the party's cell");
                if view == "tilted" {
                    // those cut by the map's edges aside, the areas run by feet (the party last at equal feet)
                    let kept: Vec<(i64, i64, bool)> = map.widgets().iter().filter(|w| w["kind"] == "marker" || w["id"] == "party").filter_map(|w| {
                        let (x, y, ww, h) = rect(w);
                        (x > 0 && x + ww < W as i64 && y + h < (H - 1) as i64).then_some((y + h, x, w["id"] == "party"))
                    }).collect();
                    assert!(kept.windows(2).all(|p| p[0].0 < p[1].0 || (p[0].0 == p[1].0 && (p[1].2 || p[0].1 <= p[1].1))), "{}: draw order {kept:?}", s(loc, "id"));
                }
                // the camera on each person in turn
                let people: Vec<Pos> = explore::markers_of(loc).iter().filter(|m| s(m, "kind") == "npc").map(|m| explore::marker_pos(t, m, 0)).filter(|at| at.0 >= 0).collect();
                for at in people {
                    map.pan = Some(at);
                    map.pan_loc = s(loc, "id");
                    frame(&mut map, &mut cx, &mut g, 1.0);
                    let stands_here = |v: &Value| {
                        let n = |i: usize| v["rect"][i].as_i64().unwrap_or(0) as i32;
                        map.geo.cell_at(n(0) + n(2) / 2, n(1) + n(3) - 1) == at && v["id"] != "party"
                    };
                    let Some(raw) = map.state(&cx)["sprites"].as_array().and_then(|l| l.iter().find(|v| stands_here(v)).cloned()) else { continue };
                    let id = s(json::dict(&raw), "id");
                    let marker = id.strip_prefix("marker.").expect("a marker's sprite");
                    let own = map.geo.cell_of(at);
                    let takes = |p: (i32, i32)| matches!(aegis_ui::ui::hit(&map.w, p), Some(RAct::Marker(m)) if m == marker);
                    assert!(takes((own.x + own.w / 2, own.y + own.h / 2)), "{view}: {id}'s own cell takes the click");
                    let north = map.geo.cell_of((at.0, at.1 - 1));
                    assert!(!takes((north.x + north.w / 2, north.y + north.h / 2)), "{view}: the cell north of {id} is not {id}'s");
                    let n = |i: usize| raw["rect"][i].as_i64().unwrap_or(0) as i32;
                    assert!(!takes((n(0) + n(2) / 2, n(1) + 2)), "{view}: {id}'s head takes no click");
                    checked += 1;
                }
            }
        }
        assert!(checked > 0, "someone stands on a map of the world");
    }

    /// Review 2026-11-02 (MAPNAV1-U unchanged): with two standing members, a click anywhere in a cell next to the leader
    /// walks there and never passes the lead; a click on the leader's own cell does.
    #[test]
    fn a_click_beside_the_leader_walks_and_does_not_pass_the_lead() {
        for (view, _) in mapview::VIEWS {
            let mut cx = game_cx();
            cx.art = aegis_ui::art::Art::new(Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
            let mut g = gfx();
            cx.settings.v["region_view"] = json!(view);
            cx.settings.v["map_legend_open"] = json!(false);
            let (loc, at) = site(&cx, |t, x, y| (-2..=2).all(|dy| (-2..=2).all(|dx| ch(t, x + dx, y + dy) == '.')));
            cx.gs_mut().location_id = loc;
            let mut map = RegionMap::new(&mut cx);
            {
                let gs = cx.gs_mut();
                gs.party.members.truncate(1);
                gs.party.active_ids.clear();
                let mut second = gs.party.members[0].clone();
                second.id = "test_second".into();
                gs.party.members.push(second);
                gs.position = at;
            }
            frame(&mut map, &mut cx, &mut g, 1.0);
            let gs = cx.gs().clone();
            assert!(gs.leader_choices(standing(&cx.data, &gs.world_id)).len() >= 2, "two can lead");
            let own = map.geo.cell_of(at);
            assert!(matches!(aegis_ui::ui::hit(&map.w, (own.x + own.w / 2, own.y + own.h / 2)), Some(RAct::Lead)), "{view}: the leader's own cell passes the lead");
            for dir in explore::DIRS {
                let to = (at.0 + dir.0, at.1 + dir.1);
                let c = map.geo.cell_of(to);
                for pt in [(c.x + c.w / 2, c.y + c.h / 2), (c.x, c.y), (c.x + c.w - 1, c.y), (c.x, c.y + c.h - 1), (c.x + c.w - 1, c.y + c.h - 1)] {
                    map.walk.clear();
                    map.draw(&mut cx, &mut g, pt, true);
                    assert!(matches!(aegis_ui::ui::hit(&map.w, pt), Some(RAct::MapClick)), "{view}: {pt:?} in the cell at {dir:?} is the map's");
                    assert!(map.click(&mut cx, pt), "{view}: the map takes the click at {pt:?}");
                    assert_eq!(map.walk, vec![dir], "{view}: one step {dir:?}, the click at {pt:?}");
                    assert_eq!(leader_id(&cx), gs.party.members[0].id, "{view}: the lead stays");
                }
            }
        }
    }

    /// L3: the click order and the hovered name follow the draw order, by feet then left edge, so what is drawn in front
    /// takes the click.
    #[test]
    fn the_person_drawn_in_front_takes_the_click_and_the_name() {
        let mark = |id: &str, x: i32, foot: i32| MarkHit { id: id.into(), label: id.into(), hit: Rect::new(x, foot - 20, 30, 20), foot, tip: Vec::new(), named: false, name_rect: Rect::new(x, foot - 20, 30, 20), badge: false };
        // all overlap at (35, 95); pushed out of order
        let mut hits = vec![mark("right", 20, 100), mark("low", 15, 110), mark("left", 10, 100)];
        let party = Rect::new(500, 500, 16, 12);
        let order = crate::props::sorted(&[], &[], &hits.iter().map(|h| Rect::new(h.hit.x, h.foot - 12, 16, 12)).collect::<Vec<_>>(), party);
        let drawn: Vec<String> = order.iter().filter_map(|i| if let Item::Marker(n) = i { Some(hits[*n].id.clone()) } else { None }).collect();
        let front = front_hit(&hits, |r| r.contains(35, 95)).map(|h| h.id.clone());
        assert_eq!(front.as_ref(), drawn.last(), "the hovered name is the one drawn last");
        sort_hits(&mut hits);
        assert_eq!(hits.iter().map(|h| h.id.clone()).collect::<Vec<_>>(), drawn, "the click order is the draw order");
        let mut g = gfx();
        let mut ui: Ui<RAct> = Ui::new(&mut g, (0, 0), true);
        push_hits(&mut ui, hits);
        assert!(matches!(aegis_ui::ui::hit(&ui.widgets, (35, 95)), Some(RAct::Marker(m)) if Some(&m) == drawn.last()), "the one in front takes the click");
        // at equal feet, the one further right is in front
        assert_eq!(drawn, ["left", "right", "low"]);
    }

    /// 3.2: water's lip: a 3 px band and its soft row under a dry north cell, a shaded bottom row over a dry south cell;
    /// none between two water cells.
    #[test]
    fn water_gets_its_lips_only_beside_dry_cells() {
        let mut g = gfx();
        let water = [40, 90, 100];
        let r = Rect::new(100, 100, 16, 12);
        let paint = |g: &mut Gfx, north: bool, south: bool| {
            let mut ui: Ui<RAct> = Ui::new(g, (0, 0), true);
            ui.p.fill(r, rgb(water[0], water[1], water[2]));
            draw_lips(&mut ui, r, north, south);
        };
        let mid = r.x + r.w / 2;
        paint(&mut g, true, true);
        assert_eq!(px(&g, mid, r.y), [118, 116, 62], "the top row");
        assert_eq!((px(&g, mid, r.y + 1), px(&g, mid, r.y + mapview::LIP_DEPTH)), ([66, 54, 30], [66, 54, 30]), "the rest of the band");
        assert_ne!(px(&g, mid, r.y + 3), water, "the soft row under it");
        assert_eq!(px(&g, mid, r.y + 4), water, "then the water");
        assert_ne!(px(&g, mid, r.y + r.h - 1), water, "the bottom row");
        paint(&mut g, false, false);
        for y in r.y..r.y + r.h {
            assert_eq!(px(&g, mid, y), water, "between water cells, row {y}");
        }
    }

    /// Q2: a debug build's F7 flips the view live and saves it, also in a talk; a release build ignores it.
    #[test]
    fn f7_flips_the_view_in_a_debug_build() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let start = cx.gs().position;
        map.key(&mut cx, Key::Fn(7));
        let want = if cfg!(debug_assertions) { "tilted" } else { "square" };
        assert_eq!(cx.settings.v["region_view"], json!(want));
        assert_eq!(cx.gs().position, start, "no step");
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(map.state(&cx)["view"], json!(want));
        assert_eq!(map.geo.view.name(), want);
        map.key(&mut cx, Key::Fn(7));
        assert_eq!(cx.settings.v["region_view"], json!("square"));
        assert_eq!(Key::parse("f7"), Some(Key::Fn(7)), "the agent link's name");
        assert_eq!(Key::Fn(7).name(), "f7");
    }

    /// WALKRUN1: Shift held runs a held-key walk and a path walk at `RUN_STEP_MS`, in any build; let go, the step under way
    /// keeps its time and the next is a walk's; it is still one step per due tick.
    #[test]
    fn shift_runs_and_letting_go_walks_from_the_next_step() {
        let (run, walk) = (mapview::RUN_STEP_MS, mapview::WALK_STEP_MS.2);
        let ms = |v: f64| Duration::from_secs_f64(v / 1000.0);
        for speed in [0.5, 1.0, 2.0] {
            let (mut cx, mut g) = (game_cx(), gfx());
            cx.settings.v["anim_speed"] = json!(speed);
            let mut map = RegionMap::new(&mut cx);
            let d = cx.data.clone();
            let start = cx.gs().clone();
            let t = explore::template_of(&d, explore::location(&d, &start)).clone();
            let (w, h) = explore::size_of(&t);
            let marked: Vec<Pos> = explore::markers_of(explore::location(&d, &start)).iter().map(|m| explore::marker_pos(&t, m, start.explore_int("loc_steps"))).collect();
            let free = |p: Pos| explore::is_open(&t, p.0, p.1) && !marked.contains(&p);
            let at = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).find(|&(x, y)| (0..5).all(|i| free((x + i, y)))).expect("a free row");
            cx.gs_mut().position = at;
            frame(&mut map, &mut cx, &mut g, 1.0);
            // a path walk with Shift held
            cx.held.shift = true;
            map.walk = vec![(1, 0); 3];
            let begin = Instant::now() + Duration::from_secs(1);
            map.next_walk = begin;
            map.tick_at(&mut cx, begin);
            assert_eq!(cx.gs().position, (at.0 + 1, at.1));
            assert_eq!(map.next_walk, begin + ms(run / speed), "the run's time at {speed}x");
            // let go: the step under way keeps its time, none comes early, and the one after it is a walk's
            cx.held.shift = false;
            let second = map.next_walk;
            map.tick_at(&mut cx, second - Duration::from_millis(1));
            assert_eq!(cx.gs().position, (at.0 + 1, at.1), "none before its time");
            map.tick_at(&mut cx, second);
            assert_eq!(cx.gs().position, (at.0 + 2, at.1), "the next step comes on the run's schedule");
            assert_eq!(map.next_walk, second + ms(walk / speed), "and walks from there at {speed}x");
            // a held key with Shift held: the press takes the run's time at once
            map.walk.clear();
            cx.held.shift = true;
            map.key(&mut cx, Key::Right);
            let pressed = map.frame_at.expect("the frame's clock");
            assert_eq!(cx.gs().position, (at.0 + 3, at.1));
            assert_eq!(map.next_walk, pressed + ms(run / speed), "the held key's run at {speed}x");
        }
    }

    /// VIEWKEYS1: a walked path keeps one step per due tick at any step time, the leader never idles between its steps
    /// and the game state it ends in is the same: the step time is presentation, journeys are not touched.
    #[test]
    fn the_walk_step_only_changes_the_time_between_steps() {
        let mut ended = Vec::new();
        for (speed, base) in [(0.5, 240.0), (1.0, 130.0), (2.0, 60.0)] {
            let (mut cx, mut g) = (game_cx(), gfx());
            cx.settings.v["anim_speed"] = json!(speed);
            cx.settings.v[mapview::WALK_STEP_KEY] = json!(base);
            let mut map = RegionMap::new(&mut cx);
            let d = cx.data.clone();
            let start = cx.gs().clone();
            let t = explore::template_of(&d, explore::location(&d, &start)).clone();
            let (w, h) = explore::size_of(&t);
            let marked: Vec<Pos> = explore::markers_of(explore::location(&d, &start)).iter().map(|m| explore::marker_pos(&t, m, start.explore_int("loc_steps"))).collect();
            let free = |p: Pos| explore::is_open(&t, p.0, p.1) && !marked.contains(&p);
            let at = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).find(|&(x, y)| (0..4).all(|i| free((x + i, y)))).expect("a free row");
            cx.gs_mut().position = at;
            frame(&mut map, &mut cx, &mut g, 1.0);
            map.walk = vec![(1, 0); 3];
            let begin = Instant::now() + Duration::from_secs(1);
            map.next_walk = begin;
            let want = if cfg!(debug_assertions) { base / speed } else { mapview::WALK_STEP_MS.2 / speed };
            assert_eq!(map.step.wall, want);
            let gap = map.step.gap();
            for n in 1..=3 {
                let due = begin + gap * (n - 1);
                map.tick_at(&mut cx, due);
                map.tick_at(&mut cx, due);
                assert_eq!(cx.gs().position, (at.0 + n as i64, at.1), "one step per due tick: {base} at {speed}x");
                map.tick_at(&mut cx, due + gap - Duration::from_millis(1));
                assert_eq!(cx.gs().position, (at.0 + n as i64, at.1), "none before its time");
                if n < 3 {
                    assert!(map.moving(), "no idle frame between steps: {base} at {speed}x");
                }
            }
            assert!(map.moving(), "the last step still walks to its end: {base} at {speed}x");
            let gs = cx.gs();
            ended.push((gs.position.0 - at.0, gs.position.1 - at.1, gs.world_id.clone(), gs.location_id.clone()));
        }
        assert!(ended.windows(2).all(|p| p[0] == p[1]), "the same end at every step time: {ended:?}");
    }

    /// Past the run's base and linger, the frames where the old `moving` read idle at the run's pace.
    const RUN_LATE_MS: f64 = mapview::RUN_STEP_MS + WALK_LINGER_MS + 1.0;

    /// TURNFEEL1 (and WALKRUN1's carried fix): Shift pressed during a walk's step, on a keyed step and on a path, ends
    /// neither the walk clip early nor the step's schedule: the step keeps its own time until the next one is taken.
    #[test]
    fn a_change_of_pace_shows_no_idle_frame() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        let at = free_row(&mut cx, 4);
        cx.gs_mut().position = at;
        frame(&mut map, &mut cx, &mut g, 1.0);
        let begin = Instant::now() + Duration::from_secs(1);
        map.walk = vec![(1, 0)];
        map.next_walk = begin;
        map.tick_at(&mut cx, begin);
        assert_eq!(cx.gs().position, (at.0 + 1, at.1));
        assert!(map.walk.is_empty(), "only the last step's time can hold the clip");
        let walk = map.step.base;
        // 16 ms frames to the next step's time, with Shift pressed just past where the run's base and linger would end
        cx.held.shift = true;
        let mut t = RUN_LATE_MS;
        while t < walk {
            map.tick_at(&mut cx, begin + Duration::from_secs_f64(t / 1000.0));
            assert_eq!(map.step.base, mapview::RUN_STEP_MS, "the pace changed");
            assert!(map.moving(), "no idle frame at {t} ms of a {walk} ms step");
            t += 16.0;
        }
    }

    /// TURNFEEL1, end to end on the map: with the beat on a turn moves the party's tile at once, faces it at once, holds
    /// the drawn position through the beat and takes the next step late by the beat; the same walk ends on the same tile.
    #[test]
    fn a_turn_beat_delays_the_glide_and_the_next_step_not_the_tile() {
        let mut ended = Vec::new();
        for beat in [false, true] {
            let (mut cx, mut g) = (game_cx(), gfx());
            cx.settings.v[mapview::TURN_BEAT_KEY] = json!(beat);
            let mut map = RegionMap::new(&mut cx);
            let at = free_row(&mut cx, 4);
            cx.gs_mut().position = at;
            map.facing = (0, 1);
            frame(&mut map, &mut cx, &mut g, 1.0);
            let begin = Instant::now() + Duration::from_secs(1);
            map.walk = vec![(1, 0), (1, 0), (1, 0)];
            map.next_walk = begin;
            map.tick_at(&mut cx, begin);
            assert_eq!(cx.gs().position, (at.0 + 1, at.1), "the tile changes on schedule");
            assert_eq!(map.facing, (1, 0), "and the party faces it");
            let on = beat && cfg!(debug_assertions);
            let wall = map.step.wall;
            let delay = if on { mapview::TURN_BEAT_MS } else { 0.0 };
            assert_eq!(map.next_walk, begin + Duration::from_secs_f64((delay + wall) / 1000.0), "the next step waits");
            assert!(map.moving());
            map.tick_at(&mut cx, map.next_walk);
            assert_eq!(cx.gs().position, (at.0 + 2, at.1));
            assert_eq!(map.leg.delay, 0.0, "straight on: no beat");
            map.tick_at(&mut cx, map.next_walk);
            ended.push(cx.gs().position.0 - at.0);
        }
        assert_eq!(ended[0], ended[1], "the same walk");
    }

    /// TURNFEEL1: a press answers at once with the beat on: the tile moves on the press, whatever the schedule says.
    #[test]
    fn a_key_press_during_a_beat_still_steps_at_once() {
        let (mut cx, mut g) = (game_cx(), gfx());
        cx.settings.v[mapview::TURN_BEAT_KEY] = json!(true);
        let mut map = RegionMap::new(&mut cx);
        let at = free_row(&mut cx, 4);
        cx.gs_mut().position = at;
        map.facing = (0, 1);
        frame(&mut map, &mut cx, &mut g, 1.0);
        map.key(&mut cx, Key::Right);
        map.key(&mut cx, Key::Right);
        assert_eq!(cx.gs().position, (at.0 + 2, at.1), "two presses, two tiles, with no wait");
    }

    /// A free tile with a free tile to its right and one below that: a walk right, then a turn down.
    fn free_corner(cx: &mut Cx) -> Pos {
        let d = cx.data.clone();
        let start = cx.gs().clone();
        let t = explore::template_of(&d, explore::location(&d, &start)).clone();
        let (w, h) = explore::size_of(&t);
        let marked: Vec<Pos> = explore::markers_of(explore::location(&d, &start)).iter().map(|m| explore::marker_pos(&t, m, start.explore_int("loc_steps"))).collect();
        let free = |p: Pos| explore::is_open(&t, p.0, p.1) && !marked.contains(&p);
        (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).find(|&(x, y)| free((x, y)) && free((x + 1, y)) && free((x + 2, y)) && free((x + 1, y + 1)) && free((x + 1, y + 2))).expect("a free corner")
    }

    /// TURNFEEL1, on the map: a press mid-leg moves the tile and the facing `E` uses at once, and a tap released during the
    /// finish still turns: the drawn path finishes the leg, then goes to the new tile, in the time the setting says; the
    /// next held step waits for it; the sprite's clip never idles on the way.
    #[cfg(debug_assertions)]
    #[test]
    fn a_tap_released_during_a_finish_still_turns() {
        for (name, wait) in [("finish", 150.0), ("quick", 60.0)] {
            let (mut cx, mut g) = (game_cx(), gfx());
            cx.settings.v[mapview::TURN_CORNER_KEY] = json!(name);
            let mut map = RegionMap::new(&mut cx);
            let at = free_corner(&mut cx);
            cx.gs_mut().position = at;
            map.facing = (1, 0);
            frame(&mut map, &mut cx, &mut g, 1.0);
            let t0 = Instant::now() + Duration::from_secs(1);
            let ms = |m: f64| t0 + Duration::from_secs_f64(m / 1000.0);
            map.tick_at(&mut cx, t0);
            cx.held.dir = None;
            let loc = cx.gs().location_id.clone();
            let drawn = |map: &mut RegionMap, cx: &mut Cx, now: f64| {
                let (leg, step_at) = (map.leg, map.step_at.take());
                map.glide.update(&loc, cx.gs().position, step_at, Some(ms(now)), leg).0
            };
            // a step right from rest
            map.key(&mut cx, Key::Right);
            let (x, y) = (at.0 as f32, at.1 as f32);
            assert_eq!(drawn(&mut map, &mut cx, 0.0), (x, y));
            assert_eq!(drawn(&mut map, &mut cx, 50.0), (x + 0.25, y));
            // a tap down 50 ms in, let go at once: the frame's clock reads the press
            map.tick_at(&mut cx, ms(50.0));
            cx.held.dir = Some((0, 1));
            map.key(&mut cx, Key::Down);
            cx.held.dir = None;
            assert_eq!(cx.gs().position, (at.0 + 1, at.1 + 1), "{name}: the tile moves on the press");
            assert_eq!(map.facing, (0, 1), "{name}: E faces it at once");
            let walk = map.step.wall;
            assert_eq!(map.next_walk, ms(50.0 + wait + walk), "{name}: the next step waits");
            let mut t = 50.0;
            while t < 50.0 + wait + walk {
                let p = drawn(&mut map, &mut cx, t);
                assert!(p.1 == y || p.0 == x + 1.0, "{name}: an L at {t} ms: {p:?}");
                map.tick_at(&mut cx, ms(t));
                assert!(map.moving(), "{name}: no idle frame at {t} ms");
                t += 8.0;
            }
            assert_eq!(drawn(&mut map, &mut cx, 50.0 + wait + walk), (x + 1.0, y + 1.0), "{name}: the turn played");
            assert_eq!(cx.gs().position, (at.0 + 1, at.1 + 1), "{name}: one step only");
        }
    }

    /// TURNFEEL1: the corner setting changes no tile: the same keys end on the same tile, with the same log, under each;
    /// a reversal on a leg with no turn pending has the same schedule.
    #[test]
    fn the_corner_setting_changes_no_tile() {
        let mut ended = Vec::new();
        for name in ["cut", "finish", "quick"] {
            let (mut cx, mut g) = (game_cx(), gfx());
            cx.settings.v[mapview::TURN_CORNER_KEY] = json!(name);
            let mut map = RegionMap::new(&mut cx);
            let at = free_corner(&mut cx);
            cx.gs_mut().position = at;
            map.facing = (1, 0);
            frame(&mut map, &mut cx, &mut g, 1.0);
            map.tick(&mut cx);
            map.key(&mut cx, Key::Right);
            frame(&mut map, &mut cx, &mut g, 1.0);
            map.tick(&mut cx);
            map.key(&mut cx, Key::Left);
            let reversal = map.next_walk.saturating_duration_since(map.frame_at.expect("the frame's clock"));
            assert_eq!(reversal, map.step.gap(), "{name}: a reversal keeps its time");
            frame(&mut map, &mut cx, &mut g, 1.0);
            for k in [Key::Right, Key::Down, Key::Down] {
                map.tick(&mut cx);
                map.key(&mut cx, k);
                frame(&mut map, &mut cx, &mut g, 1.0);
            }
            ended.push((cx.gs().position, cx.log.len()));
        }
        assert!(ended.windows(2).all(|p| p[0] == p[1]), "{ended:?}");
    }

    /// A tile with `n` free tiles in a row going right, on the start map (no walls, no markers).
    fn free_row(cx: &mut Cx, n: i64) -> Pos {
        let d = cx.data.clone();
        let start = cx.gs().clone();
        let t = explore::template_of(&d, explore::location(&d, &start)).clone();
        let (w, h) = explore::size_of(&t);
        let marked: Vec<Pos> = explore::markers_of(explore::location(&d, &start)).iter().map(|m| explore::marker_pos(&t, m, start.explore_int("loc_steps"))).collect();
        let free = |p: Pos| explore::is_open(&t, p.0, p.1) && !marked.contains(&p);
        (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).find(|&(x, y)| (0..n).all(|i| free((x + i, y)))).expect("a free row")
    }

    /// VIEWKEYS1: on the map a debug build's F6 redraws at the next zoom and F9 saves the next size, both in the log and
    /// neither taking a step; a release build ignores them (the rules are `viewkeys`' tests).
    #[test]
    fn f6_zooms_the_map_and_f9_sizes_the_window_in_a_debug_build() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let (start, q) = (cx.gs().position, map.geo.q);
        map.key(&mut cx, Key::Fn(6));
        frame(&mut map, &mut cx, &mut g, 1.0);
        let debug = cfg!(debug_assertions);
        assert_eq!(map.geo.q, if debug { crate::viewkeys::ZOOM_CYCLE[1] } else { q });
        assert_eq!(cx.log.last().is_some_and(|l| l.contains("1.5x")), debug);
        map.key(&mut cx, Key::Fn(9));
        assert_eq!(cx.settings.v["window_size"], json!(if debug { "1600x900" } else { "1280x720" }));
        assert_eq!(cx.log.last().is_some_and(|l| l.contains("1600x900")), debug);
        assert!(cx.notice.is_empty(), "the map's log ticker shows them");
        assert_eq!(cx.gs().position, start, "no step");
        // VIEWKEYS2: F11 takes effect on the next frame's tick, without Settings
        map.key(&mut cx, Key::Fn(11));
        map.tick(&mut cx);
        assert_eq!(map.step.base, if debug { crate::viewkeys::next_walk(&json!(mapview::WALK_STEP_MS.2)) } else { mapview::WALK_STEP_MS.2 });
        assert_eq!(cx.gs().position, start, "no step");
    }
}

/// REGIONGROUND1-L (spec section 8, checks L1-L6), under both views, on a test-only world and map, so no check reads
/// shipped ground rows: art off unless a check makes scratch art, every fixture ground flat, with wear and tone 0 unless a
/// check sets them, so a pixel well inside a ground is exactly its colour.
#[cfg(test)]
mod ground_tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx};
    use crate::screens::Game;
    use aegis_ui::canvas::Canvas;
    use aegis_ui::ground::{ground_composes, ground_expands};
    use emberwake_world::{new_game, Data};
    use std::path::Path;
    use std::sync::Arc;

    const WORLD: &str = "zz_world";
    /// Letter, id, order, kind, colour; the list is out of `order` on purpose.
    const GROUNDS: [(&str, &str, i32, &str, [u8; 3]); 5] =
        [("r", "zz_road", 60, "land", [150, 140, 90]), ("a", "zz_ash", 10, "land", [120, 60, 60]), ("w", "zz_water", 5, "water", [10, 30, 60]), ("b", "zz_moss", 20, "land", [60, 120, 60]), ("c", "zz_slate", 30, "land", [60, 60, 120])];
    /// Three more, for the large map's 8 layers.
    const MORE: [(&str, &str, i32, &str, [u8; 3]); 3] = [("d", "zz_d", 40, "land", [90, 90, 40]), ("e", "zz_e", 50, "land", [40, 90, 90]), ("f", "zz_f", 70, "land", [90, 40, 90])];
    const WALL: [u8; 3] = [200, 100, 40];
    const FACE: [u8; 3] = [170, 80, 30];
    const MAT: [u8; 3] = [230, 230, 20];
    const POST: [u8; 3] = [20, 230, 230];

    #[derive(Clone, Copy)]
    struct Fixture {
        cols: i64,
        rows: i64,
        layer: bool,
        tileset: &'static str,
        tone: f64,
        many: bool,
        boundary: &'static str,
    }

    const PLAIN: Fixture = Fixture { cols: 40, rows: 40, layer: false, tileset: "", tone: 0.0, many: false, boundary: "" };
    const LAYER: Fixture = Fixture { layer: true, ..PLAIN };

    impl Fixture {
        fn tile(&self, x: i64, y: i64) -> char {
            if x <= 0 || y <= 0 || x >= self.cols - 1 || y >= self.rows - 1 || ((8..=11).contains(&x) && (self.rows - 8..=self.rows - 7).contains(&y)) {
                '#'
            } else if (28..=32).contains(&x) && (4..=7).contains(&y) {
                '~'
            } else {
                '.'
            }
        }

        /// A wall takes the letter around it; `a` | `b` meet at column 13, `b` | `c` at 26, a road crosses at rows 10-12.
        fn letter(&self, x: i64, y: i64) -> char {
            if self.tile(x, y) == '~' {
                'w'
            } else if self.many {
                ['a', 'b', 'c', 'd', 'e', 'r', 'f'][((x / 6 + y / 5) % 7) as usize]
            } else if (10..=12).contains(&y) {
                'r'
            } else if x < 13 {
                'a'
            } else if x < 26 {
                'b'
            } else {
                'c'
            }
        }

        fn colour_of(&self, x: i64, y: i64) -> [u8; 3] {
            let l = self.letter(x, y).to_string();
            GROUNDS.iter().chain(&MORE).find(|g| g.0 == l).expect("a fixture letter").4
        }

        fn spawn(&self) -> Pos {
            (20, self.rows - 4)
        }

        fn mat(&self) -> Pos {
            (17, self.rows - 9)
        }

        fn post(&self) -> Pos {
            (23, self.rows - 9)
        }

        /// Open and at least one cell away from another letter, a wall and water (so its centre is plain ground), and
        /// clear of the party, the markers and the props.
        fn plain_cell(&self, at: Pos, party: Pos) -> bool {
            let same = (-1..=1).all(|dy| (-1..=1).all(|dx| self.letter(at.0 + dx, at.1 + dy) == self.letter(at.0, at.1) && self.tile(at.0 + dx, at.1 + dy) == '.'));
            let taken = [party, self.spawn(), (3, 3), (36, 3), self.mat(), self.post()];
            same && !taken.iter().any(|t| (t.0 - at.0).abs() <= 2 && (t.1 - at.1).abs() <= 2)
        }

        fn data(&self, edit: impl FnOnce(&mut Value)) -> Data {
            let lines = |f: &dyn Fn(i64, i64) -> char| -> Vec<String> { (0..self.rows).map(|y| (0..self.cols).map(|x| f(x, y)).collect()).collect() };
            let spawn = self.spawn();
            let (mat, post) = (self.mat(), self.post());
            let mut map = json!({"id": "zz_yard", "rows": lines(&|x, y| self.tile(x, y)),
                "anchors": [{"id": "spawn", "x": spawn.0, "y": spawn.1}, {"id": "job", "x": 3, "y": 3}, {"id": "far", "x": 36, "y": 3}],
                "props": [{"id": "mat", "art": "zz_mat", "x": mat.0, "y": mat.1, "w": 1, "h": 1, "layer": "ground", "variant": -1, "flip": false},
                          {"id": "post", "art": "zz_post", "x": post.0, "y": post.1, "w": 1, "h": 1, "layer": "sort", "variant": -1, "flip": false}]});
            if self.layer {
                map["ground"] = json!(lines(&|x, y| self.letter(x, y)));
                let mut legend: serde_json::Map<String, Value> = GROUNDS.iter().chain(&MORE).map(|g| (g.0.to_string(), json!(g.1))).collect();
                legend.insert("z".into(), json!("zz_unused"));
                map["ground_legend"] = Value::Object(legend);
            }
            edit(&mut map);
            let row = |g: &(&str, &str, i32, &str, [u8; 3]), tone: f64| json!({"id": g.1, "name": g.1, "order": g.2, "kind": g.3, "colour": format!("#{:02x}{:02x}{:02x}", g.4[0], g.4[1], g.4[2]), "wear": 0, "tone": tone});
            let mut grounds: Vec<Value> = GROUNDS.iter().chain(&MORE).map(|g| row(g, self.tone)).collect();
            grounds.push(json!({"id": "zz_unused", "name": "Unused", "order": 99, "colour": "#ffffff"}));
            let catalog = json!({"classes": [{"id": "fighter", "name": "Fighter"}], "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}], "grounds": grounds});
            let owned = |rows: Value| -> Value {
                let list = rows.as_array().expect("rows").iter().map(|r| {
                    let mut r = r.clone();
                    r["_world"] = json!(WORLD);
                    r["_qid"] = json!(format!("{WORLD}:{}", r["id"].as_str().unwrap_or("")));
                    r
                });
                Value::Array(list.collect())
            };
            let worlds = json!({WORLD: {"manifest": {"id": WORLD, "name": "Yard", "start_location": "hub", "start_party": ["hero"]}, "own": {
                "locations": owned(json!([{"id": "hub", "name": "Hub", "map": "zz_yard", "tileset": self.tileset, "boundary": self.boundary, "markers": [
                    {"id": "m_job", "kind": "encounter", "encounter": "job", "anchor": "job"},
                    {"id": "m_far", "kind": "encounter", "encounter": "job", "anchor": "far", "unless_flags": [{"key": "zz_gone"}]}]}])),
                "encounters": owned(json!([{"id": "job", "name": "Job", "location": "hub", "enemies": []}]))
            }}});
            Data::from_values(&catalog, &worlds, &json!({"maps": [map]}))
        }

        fn cx(&self, view: &str) -> Cx {
            self.cx_on(self.data(|_| ()), view)
        }

        fn cx_on(&self, data: Data, view: &str) -> Cx {
            let mut cx = game_cx();
            let d = Arc::new(data);
            let mut gs = new_game::create_for_world(&d, WORLD, 1).expect("a new game");
            explore::enter(&d, &mut gs, "hub", explore::SPAWN);
            // no random fight in a few steps
            gs.explore.insert("next_at".into(), json!(9999));
            gs.explore.insert("steps".into(), json!(0));
            cx.app.data = d;
            cx.app.game = Some(Game::new(gs, -1, true));
            cx.settings.v["region_view"] = json!(view);
            cx.settings.v["map_legend_open"] = json!(false);
            cx
        }
    }

    fn views() -> impl Iterator<Item = &'static str> {
        mapview::VIEWS.iter().map(|v| v.0)
    }

    fn px(g: &Gfx, x: i32, y: i32) -> [u8; 3] {
        let i = ((y * g.canvas.w + x) * 4) as usize;
        [g.canvas.px[i], g.canvas.px[i + 1], g.canvas.px[i + 2]]
    }

    fn mid(map: &RegionMap, at: Pos) -> (i32, i32) {
        let c = map.geo.cell_of(at);
        (c.x + c.w / 2, c.y + c.h / 2)
    }

    /// The cells whose centres show well inside the map area (away from the ticker and the bar).
    fn seen_cells(f: &Fixture, map: &RegionMap) -> Vec<Pos> {
        let inner = Rect::new(40, HUD_H + 16, W - 80, H - HUD_H - 64);
        let inside = |p: (i32, i32)| p.0 >= inner.x && p.1 >= inner.y && p.0 < inner.x + inner.w && p.1 < inner.y + inner.h;
        (0..f.rows).flat_map(|y| (0..f.cols).map(move |x| (x, y))).filter(|at| inside(mid(map, *at))).collect()
    }

    fn scratch(tag: &str) -> aegis_core::scratch::Scratch {
        let root = aegis_core::scratch::Scratch::new(&format!("emberwake_ground_{tag}"));
        root
    }

    fn solid(root: &Path, rel: &str, w: u32, h: u32, c: [u8; 3]) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(path).unwrap()), w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let row: Vec<u8> = (0..w * h).flat_map(|_| [c[0], c[1], c[2], 255]).collect();
        enc.write_header().unwrap().write_image_data(&row).unwrap();
    }

    fn art_of(root: &Path) -> aegis_ui::art::Art {
        aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![root.to_path_buf()]))
    }

    /// L1: no ground keeps every tile (their stand-in fills with the art off), composes nothing and says null, and a layer
    /// the plan refuses draws those same bytes, so a bad row never takes a map's tiles away.
    #[test]
    fn a_map_without_ground_draws_its_tiles_and_composes_nothing() {
        for view in views() {
            let f = PLAIN;
            let (mut cx, mut g) = (f.cx(view), gfx());
            let start = ground_composes();
            let mut map = RegionMap::new(&mut cx);
            frame(&mut map, &mut cx, &mut g, 1.0);
            assert_eq!(map.state(&cx)["ground"], Value::Null, "{view}");
            assert_eq!(ground_composes(), start, "{view}: nothing composed");
            let tilted = view == "tilted";
            let mut probed = 0;
            for at in seen_cells(&f, &map) {
                let ch = f.tile(at.0, at.1);
                let want = match ch {
                    '.' if f.plain_cell(at, cx.gs().position) => [70, 90, 64],
                    '~' => [40, 90, 100],
                    '#' if !tilted => [WALL_FILL[0], WALL_FILL[1], WALL_FILL[2]],
                    _ => continue,
                };
                let p = mid(&map, at);
                assert_eq!(px(&g, p.0, p.1), want, "{view}: {at:?} '{ch}'");
                probed += 1;
            }
            assert!(probed > 50, "{view}: only {probed} cells probed");
            let today = g.canvas.px.clone();
            let refused: [fn(&mut Value); 3] = [|m| m["ground_legend"] = json!({"a": "zz_ash"}), |m| m["ground_legend"]["a"] = json!("zz_no_such_row"), |m| m["ground"] = json!(["ab"])];
            for (i, edit) in refused.into_iter().enumerate() {
                let (mut cx, mut g) = (f.cx_on(LAYER.data(edit), view), gfx());
                let mut map = RegionMap::new(&mut cx);
                frame(&mut map, &mut cx, &mut g, 1.0);
                assert_eq!(map.state(&cx)["ground"], Value::Null, "{view}: refused {i}");
                assert!(g.canvas.px == today, "{view}: refused layer {i} draws today's frame");
            }
            assert_eq!(ground_composes(), start, "{view}: still nothing composed");
        }
    }

    /// L2: everything that walks or clicks reads `rows`: the camera, the click areas, the state, every path and a walk
    /// equal the same map without the layer.
    #[test]
    fn walking_clicks_arrows_and_pans_equal_the_map_without_ground() {
        for view in views() {
            let (mut plain, mut layered) = (PLAIN.cx(view), LAYER.cx(view));
            let (mut g1, mut g2) = (gfx(), gfx());
            let (mut a, mut b) = (RegionMap::new(&mut plain), RegionMap::new(&mut layered));
            let same = |a: &RegionMap, b: &RegionMap, c1: &Cx, c2: &Cx, step: &str| {
                assert_eq!(a.geo, b.geo, "{view} {step}: the camera");
                assert_eq!(a.widgets(), b.widgets(), "{view} {step}: the click areas");
                let drop_ground = |m: &RegionMap, c: &Cx| {
                    let mut v = m.state(c);
                    v.as_object_mut().expect("a state").remove("ground");
                    v
                };
                assert_eq!(drop_ground(a, c1), drop_ground(b, c2), "{view} {step}: the state");
                for y in 0..PLAIN.rows {
                    for x in 0..PLAIN.cols {
                        assert_eq!(a.path_to(c1, (x, y), false), b.path_to(c2, (x, y), false), "{view} {step}: a path to ({x}, {y})");
                    }
                }
            };
            frame(&mut a, &mut plain, &mut g1, 1.0);
            frame(&mut b, &mut layered, &mut g2, 1.0);
            same(&a, &b, &plain, &layered, "first frame");
            assert_eq!(b.state(&layered)["ground"], json!({"layers": GROUNDS.len()}), "{view}: an unused legend entry adds no layer");
            assert!(!b.arrows.is_empty(), "{view}: a marker out of view has its arrow");
            // a click on an edge arrow pans; the same widgets, so the same click
            let arrow = b.widgets().iter().find(|w| s(json::dict(w), "id").contains("arrow")).cloned().expect("the arrow's widget");
            let (x, y, ww, hh) = crate::hud::hud_tests::rect(&arrow);
            let pt = ((x + ww / 2) as i32, (y + hh / 2) as i32);
            for (m, c, g) in [(&mut a, &mut plain, &mut g1), (&mut b, &mut layered, &mut g2)] {
                m.draw(c, g, pt, true);
                assert!(m.click(c, pt), "{view}: the arrow takes the click");
                assert!(m.pan.is_some(), "{view}: the arrow pans");
                frame(m, c, g, 1.0);
            }
            same(&a, &b, &plain, &layered, "after a pan");
            for (m, c, g) in [(&mut a, &mut plain, &mut g1), (&mut b, &mut layered, &mut g2)] {
                m.pan = None;
                frame(m, c, g, 1.0);
            }
            // a walk by a tile click: the same path and the same end
            let to = (PLAIN.spawn().0 - 3, PLAIN.spawn().1 - 1);
            for (m, c, g) in [(&mut a, &mut plain, &mut g1), (&mut b, &mut layered, &mut g2)] {
                let pt = mid(m, to);
                m.draw(c, g, pt, true);
                assert!(m.click(c, pt), "{view}: the map takes the click");
                while !m.walk.is_empty() {
                    m.next_walk = Instant::now();
                    m.tick(c);
                }
                assert_eq!(c.gs().position, to, "{view}: the walk ends on the cell");
                // the glide over (the two maps ran on their own wall clocks)
                m.tick_at(c, Instant::now() + Duration::from_secs(1));
                frame(m, c, g, 1.0);
            }
            same(&a, &b, &plain, &layered, "after a walk");
        }
    }

    /// L2: a pixel showing the neighbour's ground (the edge wanders over cells) still clicks its own cell.
    #[test]
    fn a_click_where_the_neighbours_ground_shows_walks_to_that_cell() {
        for view in views() {
            let f = LAYER;
            let (mut cx, mut g) = (f.cx(view), gfx());
            let mut map = RegionMap::new(&mut cx);
            frame(&mut map, &mut cx, &mut g, 1.0);
            let (mut found, mut clicked) = (0, 0);
            for y in f.rows - 12..f.rows - 2 {
                for (x, other) in [(12, 13), (13, 12)] {
                    if f.tile(x, y) != '.' || f.tile(other, y) != '.' {
                        continue;
                    }
                    let c = map.geo.cell(x as i32, y as i32);
                    let theirs = f.colour_of(other, y);
                    let at = (c.y..c.y + c.h).flat_map(|py| (c.x..c.x + c.w).map(move |x| (x, py))).find(|p| px(&g, p.0, p.1) == theirs);
                    let Some(pt) = at else { continue };
                    found += 1;
                    if map.path_to(&cx, (x, y), false).is_none() {
                        continue;
                    }
                    map.draw(&mut cx, &mut g, pt, true);
                    assert!(map.click(&mut cx, pt), "{view}: the map takes the click");
                    let end = map.walk.iter().fold(cx.gs().position, |p, d| (p.0 + d.0, p.1 + d.1));
                    assert_eq!(end, (x, y), "{view}: a pixel of ({other}, {y})'s ground in ({x}, {y})");
                    map.walk.clear();
                    map.walk_target = None;
                    clicked += 1;
                    frame(&mut map, &mut cx, &mut g, 1.0);
                }
            }
            assert!(found > 0 && clicked > 0, "{view}: no edge crossed a cell ({found} found, {clicked} clicked)");
        }
    }

    /// L3: a hundred frames with steps, a flag and a marker going compose once and make one copy; a view flip (F7)
    /// composes once and flipping back only copies. No frame makes a job or a key (`builds`, so no texture hashed or file
    /// read: making one is the only path to either).
    #[test]
    fn a_map_composes_once_and_a_view_flip_once_more() {
        let f = LAYER;
        let (mut cx, mut g) = (f.cx("square"), gfx());
        let (c0, e0) = (ground_composes(), ground_expands());
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let key = map.ground.key().cloned().expect("a key");
        let arrows = map.arrows.len();
        for i in 0..100 {
            if i % 10 == 0 {
                map.step(&mut cx, if i % 20 == 0 { (1, 0) } else { (-1, 0) });
            }
            if i == 50 {
                cx.gs_mut().set_flag("zz_gone", json!(true));
            }
            frame(&mut map, &mut cx, &mut g, 1.0);
        }
        assert_eq!(map.arrows.len() + 1, arrows, "the marker went");
        assert_eq!((ground_composes() - c0, ground_expands() - e0, map.ground.builds), (1, 1, 1), "100 frames");
        assert_eq!(map.ground.key(), Some(&key), "the same key");
        let flip = |map: &mut RegionMap, cx: &mut Cx| {
            if cfg!(debug_assertions) {
                map.key(cx, Key::Fn(7));
            } else {
                flip_view(cx);
            }
        };
        flip(&mut map, &mut cx);
        for _ in 0..3 {
            frame(&mut map, &mut cx, &mut g, 1.0);
        }
        assert_eq!(map.geo.view, mapview::View::Tilted);
        assert_eq!((ground_composes() - c0, ground_expands() - e0, map.ground.builds), (2, 2, 2), "a flip composes once");
        assert_ne!(map.ground.key(), Some(&key), "each view its own key");
        flip(&mut map, &mut cx);
        for _ in 0..3 {
            frame(&mut map, &mut cx, &mut g, 1.0);
        }
        assert_eq!((ground_composes() - c0, ground_expands() - e0, map.ground.builds), (2, 3, 3), "flipping back only copies");
        assert_eq!(map.ground.key(), Some(&key), "the same key again");
    }

    /// L3, EN 4.3: new data or new art makes a new job, and only what really changed composes; a file edited under a
    /// running `Art` is not seen (a frame reads no file) until a new `Art`.
    #[test]
    fn a_data_or_art_edit_recomposes_only_when_the_picture_changes() {
        for view in views() {
            let f = LAYER;
            let (mut cx, mut g) = (f.cx(view), gfx());
            let root = scratch(&format!("edit_{view}"));
            cx.art = art_of(&root);
            let mut map = RegionMap::new(&mut cx);
            frame(&mut map, &mut cx, &mut g, 1.0);
            let probe = (f.spawn().0 + 4, f.spawn().1 - 1);
            assert!(f.plain_cell(probe, f.spawn()) && f.letter(probe.0, probe.1) == 'b', "{view}: the probe");
            let seen = |map: &RegionMap, g: &Gfx| {
                let p = mid(map, probe);
                px(g, p.0, p.1)
            };
            assert_eq!(seen(&map, &g), [60, 120, 60], "{view}: the flat colour");
            let c0 = ground_composes();
            let b0 = map.ground.builds;
            // the same data read again: a new job, the same key, nothing composed
            cx.app.data = Arc::new((*cx.app.data).clone());
            frame(&mut map, &mut cx, &mut g, 1.0);
            assert_eq!((ground_composes() - c0, map.ground.builds - b0), (0, 1), "{view}: the same data");
            // a row's colour: one compose
            let mut data = (*cx.app.data).clone();
            data.edit_cat(|cat| cat.grounds.get_mut("zz_moss").expect("the row").colour = "#3c5a78".into());
            cx.app.data = Arc::new(data);
            frame(&mut map, &mut cx, &mut g, 1.0);
            assert_eq!((ground_composes() - c0, map.ground.builds - b0), (1, 2), "{view}: a colour edit");
            assert_eq!(seen(&map, &g), [0x3c, 0x5a, 0x78], "{view}: the new colour");
            // new art over the chain: one compose; the same file through another `Art`: none
            let rel = "tilesets/common/ground/zz_moss/v0.png";
            solid(&root, rel, 256, 256, [33, 66, 99]);
            frame(&mut map, &mut cx, &mut g, 1.0);
            assert_eq!(seen(&map, &g), [0x3c, 0x5a, 0x78], "{view}: a frame looks for no new file");
            assert_eq!((ground_composes() - c0, map.ground.builds - b0), (1, 2), "{view}: a frame reads no file");
            cx.art = art_of(&root);
            frame(&mut map, &mut cx, &mut g, 1.0);
            assert_eq!((ground_composes() - c0, map.ground.builds - b0), (2, 3), "{view}: new art");
            assert_eq!(seen(&map, &g), [33, 66, 99], "{view}: the texture");
            cx.art = art_of(&root);
            frame(&mut map, &mut cx, &mut g, 1.0);
            assert_eq!((ground_composes() - c0, map.ground.builds - b0), (2, 4), "{view}: the same file reduced the same way");
            solid(&root, rel, 256, 256, [99, 66, 33]);
            frame(&mut map, &mut cx, &mut g, 1.0);
            assert_eq!(seen(&map, &g), [33, 66, 99], "{view}: a file edited under a running `Art`");
            cx.art = art_of(&root);
            frame(&mut map, &mut cx, &mut g, 1.0);
            assert_eq!((ground_composes() - c0, map.ground.builds - b0), (3, 5), "{view}: a retouched file");
            assert_eq!(seen(&map, &g), [99, 66, 33], "{view}: the retouched texture");
            let _ = std::fs::remove_dir_all(&root);
        }
    }

    /// Spec section 3, LOOK1 5.4: a ground's art comes from the location's `tileset`, then the world lineage, then `common`.
    #[test]
    fn a_grounds_art_follows_the_tile_set_chain() {
        for view in views() {
            let root = scratch(&format!("chain_{view}"));
            let rel = |set: &str| format!("tilesets/{set}/ground/zz_moss/v0.png");
            let probe = (LAYER.spawn().0 + 4, LAYER.spawn().1 - 1);
            let shown = |f: Fixture| {
                let (mut cx, mut g) = (f.cx(view), gfx());
                cx.art = art_of(&root);
                let mut map = RegionMap::new(&mut cx);
                frame(&mut map, &mut cx, &mut g, 1.0);
                let p = mid(&map, probe);
                px(&g, p.0, p.1)
            };
            let bound = Fixture { tileset: "zz_bound", ..LAYER };
            assert_eq!(shown(bound), [60, 120, 60], "{view}: no art: the colour");
            solid(&root, &rel("common"), 256, 256, [11, 22, 33]);
            assert_eq!(shown(bound), [11, 22, 33], "{view}: common");
            solid(&root, &rel(WORLD), 256, 256, [44, 55, 66]);
            assert_eq!(shown(bound), [44, 55, 66], "{view}: the world over common");
            solid(&root, &rel("zz_bound"), 256, 256, [77, 88, 99]);
            assert_eq!(shown(bound), [77, 88, 99], "{view}: the location's tile set first");
            assert_eq!(shown(LAYER), [44, 55, 66], "{view}: no `tileset`: the world");
            let _ = std::fs::remove_dir_all(&root);
        }
    }

    /// The opaque things on a map at exact colours, and where they stand.
    fn solid_things(g: &Gfx) -> Vec<(i32, i32, [u8; 3])> {
        let party = [crate::props::PARTY[0], crate::props::PARTY[1], crate::props::PARTY[2]];
        let mut out = Vec::new();
        for y in 0..H {
            for x in 0..W {
                let c = px(g, x, y);
                if [WALL, FACE, MAT, POST, party].contains(&c) {
                    out.push((x, y, c));
                }
            }
        }
        out
    }

    /// L4: a wall, a ground prop, a standing prop and the party stand on exactly the same pixels with ground and without,
    /// so ground is under them all, the tilted block still sorts with the party, and the bar is untouched. The party north
    /// of a wall makes the sort matter in the tilted view.
    #[test]
    fn walls_props_and_the_party_draw_over_the_ground_in_todays_order() {
        for view in views() {
            let root = scratch(&format!("over_{view}"));
            solid(&root, "tilesets/common/wall/v0.png", 32, 32, WALL);
            solid(&root, "tilesets/common/wall_face/v0.png", 128, 48, FACE);
            solid(&root, "props/common/zz_mat/v0.png", 32, 32, MAT);
            solid(&root, "props/common/zz_post/v0.png", 32, 64, POST);
            let shot = |f: Fixture| {
                let (mut cx, mut g) = (f.cx(view), gfx());
                cx.art = art_of(&root);
                let mut map = RegionMap::new(&mut cx);
                cx.gs_mut().position = (10, f.rows - 9);
                frame(&mut map, &mut cx, &mut g, 1.0);
                (map.geo, g)
            };
            let ((geo_a, a), (geo_b, b)) = (shot(PLAIN), shot(LAYER));
            assert_eq!(geo_a, geo_b, "{view}");
            let (seen_a, seen_b) = (solid_things(&a), solid_things(&b));
            for c in [WALL, MAT, POST] {
                assert!(seen_a.iter().any(|p| p.2 == c), "{view}: {c:?} is on the map");
            }
            assert!(seen_a == seen_b, "{view}: {} and {} pixels of walls, props and party", seen_a.len(), seen_b.len());
            let bar = (HUD_H * W * 4) as usize;
            assert!(a.canvas.px[..bar] == b.canvas.px[..bar], "{view}: the map's clip keeps the ground off the bar");
            assert_ne!(a.canvas.px, b.canvas.px, "{view}: the ground did draw");
            let _ = std::fs::remove_dir_all(&root);
        }
    }

    /// L5: every ground without art shows its `colour` deep in its own cells: exact with tone 0, within tone with tone on.
    #[test]
    fn a_ground_without_art_draws_its_colour_within_tone() {
        for view in views() {
            for tone in [0.0, 0.13] {
                let f = Fixture { tone, ..LAYER };
                let (mut cx, mut g) = (f.cx(view), gfx());
                let mut map = RegionMap::new(&mut cx);
                let mut seen: std::collections::HashSet<[u8; 3]> = Default::default();
                let mut letters: std::collections::HashSet<char> = Default::default();
                for at in [f.spawn(), (6, 8), (20, 8), (34, 16), (20, 11)] {
                    cx.gs_mut().position = at;
                    frame(&mut map, &mut cx, &mut g, 1.0);
                    for cell in seen_cells(&f, &map).into_iter().filter(|c| f.plain_cell(*c, at)) {
                        let want = f.colour_of(cell.0, cell.1);
                        let p = mid(&map, cell);
                        let got = px(&g, p.0, p.1);
                        for i in 0..3 {
                            let room = want[i] as f64 * tone / 2.0 + 1.0;
                            assert!((got[i] as f64 - want[i] as f64).abs() <= room, "{view} tone {tone}: {cell:?} is {got:?}, its colour {want:?}");
                        }
                        seen.insert(got);
                        letters.insert(f.letter(cell.0, cell.1));
                    }
                }
                assert_eq!(letters.len(), 4, "{view}: all the land grounds probed: {letters:?}");
                // the water's centre, sunken: its own colour, no `~` tile and no lip
                cx.gs_mut().position = (30, 9);
                frame(&mut map, &mut cx, &mut g, 1.0);
                let p = mid(&map, (30, 5));
                let (got, want) = (px(&g, p.0, p.1), f.colour_of(30, 5));
                assert!((0..3).all(|i| (got[i] as f64 - want[i] as f64).abs() <= want[i] as f64 * tone / 2.0 + 1.0), "{view} tone {tone}: water {got:?}, its colour {want:?}");
                if tone == 0.0 {
                    assert_eq!(seen.len(), 4, "{view}: exact colours: {seen:?}");
                } else {
                    assert!(seen.len() > 4, "{view}: tone varies a ground: {seen:?}");
                }
            }
        }
    }

    /// L5: the text sizes read clean over a map with ground.
    #[test]
    fn the_region_map_with_ground_reads_clean_at_every_text_size() {
        for view in views() {
            let (mut cx, mut g) = (LAYER.cx(view), gfx());
            super::map_tests::reads_clean(&mut cx, &mut g, view);
        }
    }

    /// EN 4.2 and 4.3's frame on the largest shipped size (48 x 30, 8 layers): prints a first frame and a frame while the
    /// camera pans, with ground and without, at the spec's three window sizes.
    /// `cargo test --release -p emberwake_gui ground_frame_time -- --ignored --nocapture`
    #[test]
    #[ignore = "a measurement; run by hand in a release build"]
    fn ground_frame_time_on_a_large_map() {
        for view in views() {
            for (w, h, z) in [(1280, 720, 1.0), (1920, 1080, 1.0), (3840, 2160, 1.0), (1280, 720, 2.0), (1920, 1080, 2.0), (1920, 1080, 3.0)] {
                for layer in [false, true] {
                    let f = Fixture { cols: 48, rows: 30, layer, tone: 0.13, many: true, ..PLAIN };
                    let (mut cx, mut g) = (f.cx(view), gfx());
                    cx.settings.v["map_zoom"] = json!(z);
                    cx.art = aegis_ui::art::Art::new(Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
                    g.canvas = Canvas::new(w, h);
                    g.view = aegis_ui::painter::View::fit(w as u32, h as u32);
                    let mut map = RegionMap::new(&mut cx);
                    let t0 = Instant::now();
                    frame(&mut map, &mut cx, &mut g, 1.0);
                    let first = t0.elapsed();
                    frame(&mut map, &mut cx, &mut g, 1.0);
                    let mut times = Vec::new();
                    for i in 0..120 {
                        map.pan = Some((10 + i % 28, 6 + i % 18));
                        map.pan_loc = "hub".into();
                        let t = Instant::now();
                        frame(&mut map, &mut cx, &mut g, 1.0);
                        times.push(t.elapsed().as_secs_f64() * 1000.0);
                    }
                    times.sort_by(f64::total_cmp);
                    println!("{view} {w}x{h} zoom {z} ground={layer}: first {:.1} ms, frame median {:.2} ms, p95 {:.2} ms, composes so far {}", first.as_secs_f64() * 1000.0, times[60], times[114], ground_composes());
                }
            }
        }
    }

    /// REGIONVIEW1: the smooth camera, its clamp and the map's border, over the same fixture on a map larger than the view.
    mod camera_tests {
        use super::*;

        const BIG: Fixture = Fixture { cols: 120, rows: 80, ..LAYER };
        /// Whole scales and a fractional one.
        const WINDOWS: [(i32, i32); 3] = [(1280, 720), (1920, 1080), (1517, 853)];
        const BOUNDARY: [u8; 3] = [90, 30, 150];
        const BOUNDARY_FACE: [u8; 3] = [50, 10, 90];

        fn gfx_at(w: i32, h: i32) -> Gfx {
            let mut g = gfx();
            g.canvas = Canvas::new(w, h);
            g.view = aegis_ui::painter::View::fit(w as u32, h as u32);
            g
        }

        fn shot_at(map: &mut RegionMap, cx: &mut Cx, g: &mut Gfx, t: Instant) {
            map.tick_at(cx, t);
            frame(map, cx, g, 1.0);
        }

        fn sixtieth(t0: Instant, i: u32) -> Instant {
            t0 + Duration::from_secs_f64(i as f64 / 60.0)
        }

        /// The steps a hold from frame 0 has taken by frame `i`: the press, and one per `step` from its time.
        fn steps_by(i: u32) -> i64 {
            1 + (i as f64 / 60.0 * 1000.0 / crate::mapview::WALK_STEP_MS.2).floor() as i64
        }

        fn widget(map: &RegionMap, id: &str) -> Value {
            map.widgets().into_iter().find(|w| w["id"] == id).expect("the widget")
        }

        fn rect(w: &Value) -> (i32, i32, i32, i32) {
            let n = |i: usize| w["rect"][i].as_i64().unwrap_or(0) as i32;
            (n(0), n(1), n(2), n(3))
        }

        /// The party's screen place stays put through a held walk at any scale and in both views, and the camera moves by
        /// the walking speed each frame (never a cell); WALK2's rate is kept. At the zoom's ends and its default.
        #[test]
        fn a_held_walk_keeps_the_party_still_and_moves_the_ground_in_small_even_steps() {
            for view in views() {
                for (w, h, z) in WINDOWS.map(|(w, h)| (w, h, 1.0)).into_iter().chain(WINDOWS.map(|(w, h)| (w, h, 2.0))).chain([(1280, 720, 1.25), (1280, 720, 3.0)]) {
                    for (key, dir) in [(Key::Left, (-1, 0)), (Key::Up, (0, -1))] {
                        let (mut cx, mut g) = (BIG.cx(view), gfx_at(w, h));
                        cx.settings.v["map_zoom"] = json!(z);
                        let mut map = RegionMap::new(&mut cx);
                        let start = (60, 50);
                        cx.gs_mut().position = start;
                        let t0 = Instant::now();
                        shot_at(&mut map, &mut cx, &mut g, t0);
                        let s = g.view.s;
                        let cell = if dir.0 != 0 { map.geo.cell_size().0 } else { map.geo.cell_size().1 };
                        let speed = cell as f64 * s as f64 / 60.0 * 1000.0 / crate::mapview::WALK_STEP_MS.2;
                        map.key(&mut cx, key);
                        cx.held.dir = Some(dir);
                        let party = map.cam.party;
                        let mut was = map.cam.px;
                        let mut moved = 0.0;
                        for i in 1..=60 {
                            shot_at(&mut map, &mut cx, &mut g, sixtieth(t0, i));
                            assert_eq!(map.cam.party, party, "{view} {w} {dir:?}: frame {i}: the party's place");
                            let d = (map.cam.px.0 - was.0, map.cam.px.1 - was.1);
                            let along = if dir.0 != 0 { d.0 } else { d.1 } as f64;
                            let across = if dir.0 != 0 { d.1 } else { d.0 };
                            assert_eq!(across, 0, "{view} {w} {dir:?}: frame {i}");
                            assert!(along >= speed.floor() && along <= speed.ceil(), "{view} {w} {dir:?}: frame {i} moved {along}, speed {speed}");
                            assert!(along < (cell as f32 * s) as f64 / 2.0, "{view} {w}: small steps");
                            assert_eq!(cx.gs().position, (start.0 + dir.0 * steps_by(i), start.1 + dir.1 * steps_by(i)), "{view} {w}: frame {i}: the step rate");
                            was = map.cam.px;
                            moved += along;
                        }
                        let want = speed * 60.0;
                        assert!((moved - want).abs() <= 2.0, "{view} {w}: {moved} over a second, {want} wanted");
                    }
                }
            }
        }

        /// The painted ground follows the camera to the physical px: a ground's edge on a row stays at the same distance from
        /// the camera in every frame, at whole and fractional scales.
        #[test]
        fn the_painted_ground_moves_with_the_camera() {
            let third = GROUNDS[4].4;
            for view in views() {
                for (w, h, z) in WINDOWS.map(|(w, h)| (w, h, 1.0)).into_iter().chain(WINDOWS.map(|(w, h)| (w, h, 2.0))).chain([(1280, 720, 1.25), (1280, 720, 3.0)]) {
                    let (mut cx, mut g) = (BIG.cx(view), gfx_at(w, h));
                    cx.settings.v["map_zoom"] = json!(z);
                    let mut map = RegionMap::new(&mut cx);
                    cx.gs_mut().position = (30, 30);
                    let t0 = Instant::now();
                    shot_at(&mut map, &mut cx, &mut g, t0);
                    // a run: at the walk's pace a 3x cell moves a whole logical px every 60th of a second
                    cx.held.shift = true;
                    map.key(&mut cx, Key::Left);
                    cx.held.dir = Some((-1, 0));
                    let s = g.view.s;
                    let (row, from) = (g.view.y(120.0), g.view.x(100.0));
                    let mut seen = None;
                    let mut between = false;
                    for i in 1..=30 {
                        shot_at(&mut map, &mut cx, &mut g, sixtieth(t0, i));
                        between |= (map.cam.px.0 as f32 / s).fract() != 0.0;
                        let edge = (from..g.view.x(500.0)).find(|x| px(&g, *x, row) == third).expect("the edge of the third ground");
                        assert_eq!(*seen.get_or_insert(edge - map.cam.px.0), edge - map.cam.px.0, "{view} {w}: frame {i}");
                    }
                    assert!(between, "{view} {w}: the camera was between logical px");
                }
            }
        }

        /// A click in a walking frame goes to the cell drawn under the cursor at the frame's fractional camera, and a
        /// marker's click area is the marker's drawn cell. The mouse is logical: a logical px whose physical px are both
        /// inside the cell is exact.
        #[test]
        fn a_click_during_a_walk_lands_on_the_cell_under_the_cursor() {
            for (view, z) in views().flat_map(|v| mapview::zoom_steps().map(move |z| (v, z))) {
                let data = BIG.data(|m| {
                    m["anchors"][1]["x"] = json!(57);
                    m["anchors"][1]["y"] = json!(48);
                });
                let (mut cx, mut g) = (BIG.cx_on(data, view), gfx_at(1280, 720));
                cx.settings.v["map_zoom"] = json!(z);
                let mut map = RegionMap::new(&mut cx);
                cx.gs_mut().position = (60, 50);
                let t0 = Instant::now();
                shot_at(&mut map, &mut cx, &mut g, t0);
                // a run: at the walk's pace a 1.5x cell moves a whole logical px every 60th of a second
                cx.held.shift = true;
                map.key(&mut cx, Key::Left);
                cx.held.dir = Some((-1, 0));
                let mut i = 1;
                loop {
                    shot_at(&mut map, &mut cx, &mut g, sixtieth(t0, i));
                    i += 1;
                    if map.cam.px.0 % 2 != 0 {
                        break;
                    }
                    assert!(i < 60, "a frame between logical px");
                }
                cx.held.dir = None;
                let (cw, ch) = map.geo.cell_size();
                let cam = map.cam.px;
                let party = cx.gs().position;
                let target = (party.0 + 3, party.1 - 2);
                let (x0, y0) = (cam.0 + target.0 as i32 * cw * 2, cam.1 + target.1 as i32 * ch * 2);
                let (x1, y1) = (x0 + cw * 2, y0 + ch * 2);
                let first = |a: i32| (a + 1).div_euclid(2);
                let last = |b: i32| (b - 2).div_euclid(2);
                for at in [(first(x0), first(y0)), (last(x1), first(y0)), (first(x0), last(y1)), (last(x1), last(y1)), ((first(x0) + last(x1)) / 2, (first(y0) + last(y1)) / 2)] {
                    map.draw(&mut cx, &mut g, at, true);
                    map.walk.clear();
                    assert_eq!(map.geo.cell_at(at.0, at.1), target, "{view}: {at:?} under camera {cam:?}");
                    assert!(map.click(&mut cx, at), "{view}: the map takes the click");
                    let end = map.walk.iter().fold(party, |p, d| (p.0 + d.0, p.1 + d.1));
                    assert_eq!(end, target, "{view}: {at:?}: the walk ends under the cursor");
                }
                // the marker: its widget is its cell's whole logical px, under a logical px from where it is drawn
                map.walk.clear();
                frame(&mut map, &mut cx, &mut g, 1.0);
                let (mx, my, _, mh) = rect(&widget(&map, "marker.m_job"));
                assert!((mx * 2 - (cam.0 + 57 * cw * 2)).abs() <= 1, "{view} {z}: the marker's left edge");
                assert!(((my + mh) * 2 - (cam.1 + 49 * ch * 2)).abs() <= 1, "{view} {z}: the marker's feet");
            }
        }

        /// The clamp on the real screen: at each corner of the map only the margin shows past it, the party off centre on
        /// its cell; a pan to the corner stops at the same place, wherever the party is.
        #[test]
        fn the_margin_at_all_four_corners_in_both_views_and_a_pan_clamps_the_same() {
            for view in views() {
                for (w, h, z) in WINDOWS.map(|(w, h)| (w, h, 2.0)).into_iter().chain(mapview::zoom_steps().map(|z| (1280, 720, z))) {
                    let (cols, rows) = (BIG.cols, BIG.rows);
                    for (x, y) in [(1, 1), (cols - 2, 1), (1, rows - 2), (cols - 2, rows - 2)] {
                        let (mut cx, mut g) = (BIG.cx(view), gfx_at(w, h));
                        cx.settings.v["map_zoom"] = json!(z);
                        let mut map = RegionMap::new(&mut cx);
                        cx.gs_mut().position = (x, y);
                        frame(&mut map, &mut cx, &mut g, 1.0);
                        let a = Rect::new(0, HUD_H, W, H - HUD_H);
                        let (cw, ch) = map.geo.cell_size();
                        let s = g.view.s;
                        let round = |v: i32| (v as f32 * s).round() as i32;
                        let m = mapview::MARGIN_CELLS;
                        let want_x = if x == 1 { round(a.x + m * cw) } else { round(a.x + a.w - m * cw - cols as i32 * cw) };
                        let want_y = if y == 1 { round(a.y + m * ch) } else { round(a.y + a.h - m * ch - rows as i32 * ch) };
                        assert_eq!(map.cam.px, (want_x, want_y), "{view} {w} at ({x}, {y})");
                        let party = rect(&widget(&map, "party"));
                        assert_eq!((party.0, party.1), (map.geo.ox + x as i32 * cw, map.geo.oy + y as i32 * ch), "{view} {w}: on its cell, off centre");
                        assert!(a.contains(party.0, party.1), "{view} {w}: in view");
                        cx.gs_mut().position = (60, 40);
                        map.pan = Some((x, y));
                        map.pan_loc = cx.gs().location_id.clone();
                        frame(&mut map, &mut cx, &mut g, 1.0);
                        assert_eq!(map.cam.px, (want_x, want_y), "{view} {w}: a pan to ({x}, {y})");
                    }
                }
            }
        }

        /// A map that fits centres in the view on both axes, wherever the party is, in both views.
        #[test]
        fn a_small_map_centres_in_the_view() {
            for view in views() {
                let f = Fixture { cols: 30, rows: 14, ..LAYER };
                for z in [1.0, 1.25] {
                    let (mut cx, mut g) = (f.cx(view), gfx());
                    cx.settings.v["map_zoom"] = json!(z);
                    let mut map = RegionMap::new(&mut cx);
                    let a = Rect::new(0, HUD_H, W, H - HUD_H);
                    for at in [(1, 1), (28, 1), (1, 12), (28, 12), (15, 7)] {
                        cx.gs_mut().position = at;
                        frame(&mut map, &mut cx, &mut g, 1.0);
                        let (cw, ch) = map.geo.cell_size();
                        assert_eq!((map.geo.ox, map.geo.oy), (a.x + (a.w - 30 * cw) / 2, a.y + (a.h - 14 * ch) / 2), "{view} {z} at {at:?}");
                    }
                }
            }
        }

        /// A legend pan eases there and clamps; Esc eases back; under reduce motion both are at once, and the party still
        /// glides.
        #[test]
        fn a_pan_eases_and_reduce_motion_does_not() {
            for view in views() {
                for (reduce, z) in [(false, 1.0), (true, 1.0), (false, 2.0), (true, 2.0)] {
                    let (mut cx, mut g) = (BIG.cx(view), gfx_at(1280, 720));
                    cx.settings.v["map_zoom"] = json!(z);
                    cx.settings.v[aegis_ui::vignette::REDUCE_MOTION] = json!(reduce);
                    let mut map = RegionMap::new(&mut cx);
                    cx.gs_mut().position = (60, 40);
                    let t0 = Instant::now();
                    shot_at(&mut map, &mut cx, &mut g, t0);
                    let home = map.cam.px;
                    map.act(&mut cx, RAct::Pan("m_job".into()));
                    assert_eq!(map.pan, Some((3, 3)));
                    let (cw, ch) = map.geo.cell_size();
                    let area = Rect::new(0, HUD_H, W, H - HUD_H);
                    let there = mapview::camera_px((cw, ch), area, (BIG.cols as i32, BIG.rows as i32), (3.0, 3.0), 2.0);
                    assert_eq!(there.0, mapview::MARGIN_CELLS * cw * 2, "{view} {z}: the pan stops at the map's west margin");
                    shot_at(&mut map, &mut cx, &mut g, sixtieth(t0, 2));
                    if reduce {
                        assert_eq!(map.cam.px, there, "{view}: at once");
                    } else {
                        assert!(map.cam.px != there, "{view}: on its way: {:?} from {home:?}", map.cam.px);
                        let mut was = map.cam.px.0;
                        for i in 3..=40 {
                            shot_at(&mut map, &mut cx, &mut g, sixtieth(t0, i));
                            assert!(map.cam.px.0 >= was, "{view}: frame {i}: towards the pan");
                            was = map.cam.px.0;
                        }
                        assert_eq!(map.cam.px, there, "{view}: there in the ease's time");
                    }
                    map.back(&mut cx);
                    shot_at(&mut map, &mut cx, &mut g, sixtieth(t0, 42));
                    if reduce {
                        assert_eq!(map.cam.px, home, "{view}: back at once");
                    } else {
                        assert!(map.cam.px != home, "{view}: on its way back");
                    }
                    shot_at(&mut map, &mut cx, &mut g, sixtieth(t0, 90));
                    assert_eq!(map.cam.px, home, "{view}: back on the party");
                    // the party's glide does not read the setting
                    map.key(&mut cx, Key::Right);
                    let was = map.cam.px;
                    shot_at(&mut map, &mut cx, &mut g, sixtieth(t0, 91));
                    assert!(map.cam.px.0 < was.0 && map.cam.px.0 > was.0 - cw * 2, "{view} reduce={reduce}: a small step of the glide: {} from {}", map.cam.px.0, was.0);
                }
            }
        }

        /// The walk only moves the blit: the composer neither composes nor expands, at a fractional scale too.
        #[test]
        fn a_walk_shifts_the_ground_without_a_new_picture() {
            for view in views() {
                for (w, h, z) in WINDOWS.map(|(w, h)| (w, h, 1.0)).into_iter().chain(WINDOWS.map(|(w, h)| (w, h, 2.0))) {
                    let (mut cx, mut g) = (BIG.cx(view), gfx_at(w, h));
                    cx.settings.v["map_zoom"] = json!(z);
                    let mut map = RegionMap::new(&mut cx);
                    cx.gs_mut().position = (60, 40);
                    let t0 = Instant::now();
                    shot_at(&mut map, &mut cx, &mut g, t0);
                    let (c0, e0) = (ground_composes(), ground_expands());
                    map.key(&mut cx, Key::Left);
                    cx.held.dir = Some((-1, 0));
                    let was = map.cam.px;
                    for i in 1..=30 {
                        shot_at(&mut map, &mut cx, &mut g, sixtieth(t0, i));
                    }
                    assert!(map.cam.px != was, "{view} {w}: it moved");
                    assert_eq!((ground_composes() - c0, ground_expands() - e0, map.ground.builds), (0, 0, 1), "{view} {w}: 30 frames");
                }
            }
        }

        /// What the map's border is made of.
        fn border_art(root: &Path, with_boundary: bool) {
            solid(root, "tilesets/common/wall/v0.png", 32, 32, WALL);
            solid(root, "tilesets/common/wall_face/v0.png", 128, 48, FACE);
            if with_boundary {
                solid(root, "tilesets/common/boundary/zz_fence/wall/v0.png", 32, 32, BOUNDARY);
                solid(root, "tilesets/common/boundary/zz_fence/wall_face/v0.png", 128, 48, BOUNDARY_FACE);
            }
        }

        /// Past the map (the margin and the ring of walls) only boundary art shows, right up to the map's edges: no cap, no
        /// highlight line and no block over the last open row or column; with no `boundary`, the wall tile.
        #[test]
        fn past_the_map_only_boundary_art_shows_and_the_seam_is_clean() {
            for view in views() {
                let tilted = view == "tilted";
                for (with_boundary, z) in [(false, 1.0), (true, 1.0), (true, 2.0), (true, 1.25), (false, 3.0)] {
                    let root = scratch(&format!("edge_{view}_{with_boundary}_{z}"));
                    border_art(&root, with_boundary);
                    let f = Fixture { boundary: if with_boundary { "zz_fence" } else { "" }, ..BIG };
                    let (cols, rows) = (f.cols as i32, f.rows as i32);
                    let (wall, face) = if with_boundary { (BOUNDARY, BOUNDARY_FACE) } else { (WALL, FACE) };
                    for corner in [(cols - 2, rows - 2), (1, 1), (1, rows - 2), (cols - 2, 1)] {
                        let (mut cx, mut g) = (f.cx(view), gfx());
                        cx.settings.v["map_zoom"] = json!(z);
                        cx.art = art_of(&root);
                        let mut map = RegionMap::new(&mut cx);
                        cx.gs_mut().position = (corner.0 as i64, corner.1 as i64);
                        frame(&mut map, &mut cx, &mut g, 1.0);
                        let geo = map.geo;
                        let ch = geo.cell_size().1;
                        let a = Rect::new(0, HUD_H, W, H - HUD_H);
                        // probes along the seams, through the middle of the screen: clear of the arrows' band and the ticker
                        let (x, y) = (a.x + a.w / 2, a.y + a.h / 2);
                        if corner.1 == rows - 2 {
                            let edge = geo.cell(0, rows - 1).y;
                            assert_ne!(px(&g, x, edge - 1), wall, "{view}: the last open row is uncovered");
                            assert_ne!(px(&g, x, edge - mapview::lift_of(ch)), [170, 172, 186], "{view}: no highlight line");
                            for py in edge..a.y + a.h {
                                assert_eq!(px(&g, x, py), wall, "{view}: ({x}, {py}) on the south border");
                            }
                        } else {
                            let top = geo.cell(0, 1).y;
                            for py in a.y + 16..top - if tilted { mapview::lift_of(ch) } else { 0 } {
                                assert_eq!(px(&g, x, py), wall, "{view}: ({x}, {py}) on the north border");
                            }
                            if tilted {
                                assert_eq!(px(&g, x, top - 1), face, "{view}: the face ends on the first open row");
                            }
                            assert_ne!(px(&g, x, top + mapview::shade_of(ch)), wall, "{view}: the open row starts");
                        }
                        if corner.0 == 1 {
                            for px_x in a.x..geo.cell(1, 0).x {
                                assert_eq!(px(&g, px_x, y), wall, "{view}: ({px_x}, {y}) on the west border");
                            }
                            assert_ne!(px(&g, geo.cell(1, 0).x, y), wall, "{view}: the first open column is uncovered");
                        } else {
                            for px_x in geo.cell(cols - 1, 0).x..a.x + a.w {
                                assert_eq!(px(&g, px_x, y), wall, "{view}: ({px_x}, {y}) on the east border");
                            }
                            assert_ne!(px(&g, geo.cell(cols - 1, 0).x - 1, y), wall, "{view}: the last open column is uncovered");
                        }
                    }
                    let _ = std::fs::remove_dir_all(&root);
                }
            }
        }

        /// A location whose boundary has no art draws today's tile.
        #[test]
        fn a_boundary_without_art_falls_back_to_the_wall_tile() {
            for view in views() {
                let root = scratch(&format!("edge_none_{view}"));
                border_art(&root, false);
                let f = Fixture { boundary: "zz_fence", ..BIG };
                let (mut cx, mut g) = (f.cx(view), gfx());
                cx.art = art_of(&root);
                let mut map = RegionMap::new(&mut cx);
                cx.gs_mut().position = (f.cols - 2, f.rows - 2);
                frame(&mut map, &mut cx, &mut g, 1.0);
                let south = map.geo.cell(100, (f.rows - 1) as i32);
                assert_eq!(px(&g, south.x, south.y + 2), WALL, "{view}");
                let _ = std::fs::remove_dir_all(&root);
            }
        }

        /// Prop art reaching past the map is cut on its edge in both views; inside the map it draws whole.
        #[test]
        fn prop_art_past_the_map_is_cut_on_its_edge() {
            for (view, z) in views().flat_map(|v| [(v, 1.0), (v, 2.0)]) {
                let root = scratch(&format!("edge_prop_{view}_{z}"));
                border_art(&root, false);
                solid(&root, "props/common/zz_post/v0.png", 64, 128, POST);
                let data = BIG.data(|m| {
                    m["props"] = json!([
                        {"id": "north", "art": "zz_post", "x": 12, "y": 0, "w": 1, "h": 1, "layer": "sort", "variant": -1, "flip": false},
                        {"id": "west", "art": "zz_post", "x": 0, "y": 4, "w": 1, "h": 1, "layer": "sort", "variant": -1, "flip": false},
                        {"id": "inside", "art": "zz_post", "x": 16, "y": 6, "w": 1, "h": 1, "layer": "sort", "variant": -1, "flip": false}
                    ]);
                });
                let (mut cx, mut g) = (BIG.cx_on(data, view), gfx());
                cx.settings.v["map_zoom"] = json!(z);
                cx.art = art_of(&root);
                let mut map = RegionMap::new(&mut cx);
                cx.gs_mut().position = (2, 2);
                frame(&mut map, &mut cx, &mut g, 1.0);
                // art of 64 x 128 px at 2 art px a logical px at 1x
                let geo = map.geo;
                let (w, h) = (32 * geo.q / 4, 64 * geo.q / 4);
                let west = geo.cell(0, 4);
                let row = west.y + west.h - 2;
                for x in (west.x + west.w / 2 - w / 2).max(0)..west.x + west.w / 2 + w / 2 {
                    assert_eq!(px(&g, x, row) == POST, x >= geo.ox, "{view}: ({x}, {row}) against the map's left edge at {}", geo.ox);
                }
                let inside = geo.cell(16, 6);
                assert_eq!(px(&g, inside.x, inside.y + inside.h - h), POST, "{view}: a prop in the map draws whole");
                let north = geo.cell(12, 0);
                assert_eq!(px(&g, north.x + 1, geo.oy - 1), WALL, "{view}: above the map the wall");
                assert_eq!(px(&g, north.x + 1, geo.oy), POST, "{view}: inside it the prop");
                let _ = std::fs::remove_dir_all(&root);
            }
        }

        /// The rect of the exact colour `c` on the canvas (None: not there).
        fn bbox(g: &Gfx, c: [u8; 3]) -> Option<Rect> {
            let mut b: Option<(i32, i32, i32, i32)> = None;
            for y in HUD_H..H {
                for x in 0..W {
                    if px(g, x, y) == c {
                        let o = b.unwrap_or((x, y, x, y));
                        b = Some((o.0.min(x), o.1.min(y), o.2.max(x), o.3.max(y)));
                    }
                }
            }
            b.map(|(x0, y0, x1, y1)| Rect::new(x0, y0, x1 - x0 + 1, y1 - y0 + 1))
        }

        /// One factor sizes the scene (the user: else the units look gigantic against it): at 2x a standing prop's and a
        /// ground prop's painted rects and a sprite's rect are exactly twice their 1x size around the same anchor, the
        /// bottom middle of their cell or footprint, whose own size also doubles.
        #[test]
        fn at_2x_props_and_sprites_are_exactly_twice_their_size_around_the_same_anchor() {
            for view in views() {
                let root = scratch(&format!("twice_{view}"));
                border_art(&root, false);
                solid(&root, "props/common/zz_mat/v0.png", 32, 32, MAT);
                solid(&root, "props/common/zz_post/v0.png", 32, 64, POST);
                let f = BIG;
                let props_at = |z: f64| {
                    let (mut cx, mut g) = (f.cx(view), gfx());
                    cx.settings.v["map_zoom"] = json!(z);
                    cx.art = art_of(&root);
                    let mut map = RegionMap::new(&mut cx);
                    cx.gs_mut().position = (20, f.rows - 11);
                    frame(&mut map, &mut cx, &mut g, 1.0);
                    let (post, mat) = (bbox(&g, POST).expect("the post"), bbox(&g, MAT).expect("the mat"));
                    let (pc, mc) = (map.geo.cell_of(f.post()), map.geo.cell_of(f.mat()));
                    assert_eq!((post.x + post.w / 2, post.y + post.h), (pc.x + pc.w / 2, pc.y + pc.h), "{view} {z}: the post on its footprint");
                    assert_eq!((mat.x + mat.w / 2, mat.y + mat.h), (mc.x + mc.w / 2, mc.y + mc.h), "{view} {z}: the mat on its footprint");
                    (post, mat, pc)
                };
                let ((post1, mat1, cell1), (post2, mat2, cell2)) = (props_at(1.0), props_at(2.0));
                assert_eq!((post2.w, post2.h), (post1.w * 2, post1.h * 2), "{view}: the standing prop");
                assert_eq!((mat2.w, mat2.h), (mat1.w * 2, mat1.h * 2), "{view}: the ground prop");
                assert_eq!((cell2.w, cell2.h), (cell1.w * 2, cell1.h * 2), "{view}: its cell");
                let _ = std::fs::remove_dir_all(&root);
                // a sprite, on the game's art
                let sprite_at = |z: f64| {
                    let (mut cx, mut g) = (game_cx(), gfx());
                    cx.art = aegis_ui::art::Art::new(Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
                    cx.settings.v["region_view"] = json!(view);
                    cx.settings.v["map_zoom"] = json!(z);
                    let mut map = RegionMap::new(&mut cx);
                    frame(&mut map, &mut cx, &mut g, 1.0);
                    let state = map.state(&cx);
                    let sp = state["sprites"].as_array().and_then(|l| l.iter().find(|v| v["id"] == "party").cloned()).expect("the leader");
                    let n = |i: usize| sp["rect"][i].as_i64().unwrap_or(0) as i32;
                    let c = map.geo.cell_of(cx.gs().position);
                    assert_eq!((n(0) + n(2) / 2, n(1) + n(3)), (c.x + c.w / 2, c.y + c.h), "{view} {z}: the sprite on its cell");
                    (n(2), n(3), sp["px"].clone(), c.w, c.h)
                };
                let ((w1, h1, _, cw1, ch1), (w2, h2, _, cw2, ch2)) = (sprite_at(1.0), sprite_at(2.0));
                assert_eq!((w2, h2), (w1 * 2, h1 * 2), "{view}: the sprite");
                assert_eq!((cw2, ch2), (cw1 * 2, ch1 * 2), "{view}: its cell");
            }
        }

        /// The HUD bar, the legend and the ticker do not zoom.
        #[test]
        fn the_hud_the_legend_and_the_ticker_keep_their_size() {
            let f = BIG;
            let rects = |z: f64| {
                let (mut cx, mut g) = (f.cx("square"), gfx());
                cx.settings.v["map_zoom"] = json!(z);
                cx.settings.v["map_legend_open"] = json!(true);
                let mut map = RegionMap::new(&mut cx);
                cx.log("A line.".to_string());
                frame(&mut map, &mut cx, &mut g, 1.0);
                let fixed: Vec<Value> = map.widgets().into_iter().filter(|w| {
                    let id = w["id"].as_str().unwrap_or("");
                    !(id == "map" || id == "party" || id.starts_with("marker.") || id.starts_with("arrow."))
                }).map(|w| json!([w["id"], w["rect"]])).collect();
                fixed
            };
            let one = rects(1.0);
            assert!(one.iter().any(|w| w[0].as_str().is_some_and(|i| i.starts_with("here."))), "the legend's rows are among them");
            for z in mapview::zoom_steps() {
                assert_eq!(rects(z), one, "at {z}");
            }
        }

        /// A zoom step recomposes once (art px per cell follow it up to the composer's limit); past that limit the same
        /// picture stretches, and only its physical copy is new.
        #[test]
        fn a_zoom_step_makes_one_new_picture_and_then_none() {
            for view in views() {
                let f = Fixture { cols: 48, rows: 30, ..LAYER };
                let (mut cx, mut g) = (f.cx(view), gfx());
                cx.settings.v["map_zoom"] = json!(1.0);
                let mut map = RegionMap::new(&mut cx);
                frame(&mut map, &mut cx, &mut g, 1.0);
                let c0 = ground_composes();
                let at = |map: &mut RegionMap, cx: &mut Cx, g: &mut Gfx, z: f64| {
                    cx.settings.v["map_zoom"] = json!(z);
                    frame(map, cx, g, 1.0);
                    frame(map, cx, g, 1.0);
                };
                at(&mut map, &mut cx, &mut g, 2.0);
                assert_eq!((ground_composes() - c0, map.ground.builds), (1, 2), "{view}: to 2x");
                at(&mut map, &mut cx, &mut g, 3.0);
                assert_eq!(ground_composes() - c0, 1, "{view}: 3x stretches 2x's picture");
                at(&mut map, &mut cx, &mut g, 2.0);
                assert_eq!(ground_composes() - c0, 1, "{view}: back to a kept picture");
            }
        }

        /// A game with no saved value draws at 1x, today's view.
        #[test]
        fn a_game_with_no_saved_zoom_draws_at_the_default() {
            let (mut cx, mut g) = (BIG.cx("square"), gfx());
            cx.settings.v.as_object_mut().expect("settings").remove("map_zoom");
            let mut map = RegionMap::new(&mut cx);
            frame(&mut map, &mut cx, &mut g, 1.0);
            assert_eq!(map.geo.q, 4);
            assert_eq!(map.geo.cell_size(), (mapview::CELL_W, mapview::SQUARE_H));
        }

        /// REGIONVIEW1's measurement on the shipped keep at 1080p: the first frame (job, compose, copy) and a walking frame.
        #[test]
        #[ignore = "a measurement; run by hand in a release build"]
        fn the_keeps_frame_time_at_each_zoom() {
            for view in views() {
                for z in [1.0, 2.0, 3.0] {
                    let (mut cx, mut g) = (game_cx(), gfx_at(1920, 1080));
                    cx.art = aegis_ui::art::Art::new(Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
                    cx.settings.v["region_view"] = json!(view);
                    cx.settings.v["map_zoom"] = json!(z);
                    cx.settings.v["map_legend_open"] = json!(false);
                    let mut map = RegionMap::new(&mut cx);
                    let start = cx.gs().position;
                    let c0 = ground_composes();
                    let t0 = Instant::now();
                    shot_at(&mut map, &mut cx, &mut g, t0);
                    let first = t0.elapsed();
                    let mut times = Vec::new();
                    for i in 1..=240u32 {
                        cx.gs_mut().position = (start.0 + (i as i64 / 6) % 3, start.1);
                        let at = sixtieth(t0, i);
                        let t = Instant::now();
                        map.tick_at(&mut cx, at);
                        frame(&mut map, &mut cx, &mut g, 1.0);
                        times.push(t.elapsed().as_secs_f64() * 1000.0);
                    }
                    times.sort_by(f64::total_cmp);
                    println!("{view} {z}x 1080p: first frame {:.1} ms ({} composes), walking frame median {:.2} ms, p95 {:.2} ms, ground {}", first.as_secs_f64() * 1000.0, ground_composes() - c0, times[120], times[228], map.ground.state());
                }
            }
        }

        /// On a map ringed with walls, as every shipped map is, 2 rows of wall show past the last walkable row at each corner:
        /// the ring and one cell (user 2026-11-02), in both views, whatever the zoom.
        #[test]
        fn a_ringed_map_shows_two_rows_of_wall_past_the_last_walkable_row() {
            for view in views() {
                for z in [1.0, 2.0] {
                    let (cols, rows) = (BIG.cols, BIG.rows);
                    for (x, y) in [(1, 1), (cols - 2, 1), (1, rows - 2), (cols - 2, rows - 2)] {
                        let (mut cx, mut g) = (BIG.cx(view), gfx());
                        cx.settings.v["map_zoom"] = json!(z);
                        let mut map = RegionMap::new(&mut cx);
                        cx.gs_mut().position = (x, y);
                        frame(&mut map, &mut cx, &mut g, 1.0);
                        let a = Rect::new(0, HUD_H, W, H - HUD_H);
                        let (cw, ch) = map.geo.cell_size();
                        let walkable = map.geo.cell_of((x, y));
                        let past_x = if x == 1 { walkable.x - a.x } else { a.x + a.w - (walkable.x + cw) };
                        let past_y = if y == 1 { walkable.y - a.y } else { a.y + a.h - (walkable.y + ch) };
                        assert_eq!((past_x, past_y), (2 * cw, 2 * ch), "{view} {z} at ({x}, {y})");
                    }
                }
            }
        }

        /// The Settings screen's Map zoom row: a pick is a draft until Apply; then the setting holds the step and the map
        /// draws at it.
        #[test]
        fn picking_a_map_zoom_and_applying_it_zooms_the_map() {
            let (mut cx, mut g) = (BIG.cx("square"), gfx());
            let mut s = crate::menus::SettingsScreen::new(&cx);
            s.draw(&mut cx, &mut g, (-500, -500), true);
            s.click_id(&mut cx, "settings.tab.gameplay").expect("the gameplay tab");
            s.draw(&mut cx, &mut g, (-500, -500), true);
            let row = s.widgets().into_iter().find(|w| w["id"] == "settings.map_zoom").expect("the row");
            assert_eq!(row["text"], json!("1x"), "the default");
            g.kit.open = Some("settings.map_zoom".to_string());
            s.draw(&mut cx, &mut g, (-500, -500), true);
            let steps: Vec<f64> = mapview::zoom_steps().collect();
            let labels: Vec<Value> = s.widgets().into_iter().filter(|w| w["kind"] == "option").map(|w| w["text"].clone()).collect();
            assert_eq!(labels.len(), steps.len(), "every step is listed: {labels:?}");
            let at = steps.iter().position(|z| *z == 1.75).expect("the step");
            s.click_id(&mut cx, &format!("settings.map_zoom.opt.{at}")).expect("the option");
            g.kit.open = None;
            s.draw(&mut cx, &mut g, (-500, -500), true);
            assert_eq!(s.widgets().into_iter().find(|w| w["id"] == "settings.map_zoom").expect("the row")["text"], json!("1.75x"), "the row shows the pick");
            let at_1x = |cx: &mut Cx, g: &mut Gfx| {
                let mut map = RegionMap::new(cx);
                frame(&mut map, cx, g, 1.0);
                map.geo.q
            };
            assert_eq!(at_1x(&mut cx, &mut g), 4, "only a draft so far");
            s.click_id(&mut cx, "settings.apply").expect("apply");
            assert_eq!(cx.settings.v["map_zoom"], json!(1.75));
            assert_eq!(at_1x(&mut cx, &mut g), 7, "the map draws at the applied step");
            let _ = std::fs::remove_dir_all(&cx.shot_dir);
        }

        /// The camera's trail is a measuring aid: no `state` reply carries it, and nothing is kept, unless the flag is set; with
        /// it, the trail stays bounded.
        #[test]
        fn the_camera_trail_is_off_unless_asked_for_and_bounded() {
            let (mut cx, mut g) = (BIG.cx("square"), gfx());
            let mut map = RegionMap::new(&mut cx);
            let t0 = Instant::now();
            for i in 0..20 {
                shot_at(&mut map, &mut cx, &mut g, sixtieth(t0, i));
            }
            assert!(map.cam_trail.is_empty(), "nothing kept");
            let state = map.state(&cx);
            assert!(state["camera"]["trail"].is_null(), "not in the state");
            assert!(state["camera"]["px"].is_array(), "the camera itself is");
            cx.settings.v["camera_trail"] = json!(true);
            for i in 20..(20 + CAM_TRAIL as u32 + 30) {
                map.tick_at(&mut cx, sixtieth(t0, i));
                map.draw(&mut cx, &mut g, (-500, -500), true);
            }
            assert_eq!(map.cam_trail.len(), CAM_TRAIL, "bounded");
            assert_eq!(map.state(&cx)["camera"]["trail"].as_array().map(Vec::len), Some(CAM_TRAIL));
        }

        /// Arrows toward one side never stack: each stays clear of those before it, on the edge it sits on.
        #[test]
        fn edge_arrows_do_not_stack() {
            let a = Rect::new(0, HUD_H, W, H - HUD_H);
            let at = (200.0, (a.y + ARROW) as f32);
            let mut placed = Vec::new();
            for _ in 0..4 {
                let p = beside_arrows(a, at, &placed, &[]);
                assert_eq!(p.1, at.1, "along the top edge");
                assert!(placed.iter().all(|o: &(f32, f32)| (o.0 - p.0).abs() >= ARROW as f32), "{p:?} clear of {placed:?}");
                placed.push(p);
            }
            let side = ((a.x + ARROW) as f32, 150.0);
            let p = beside_arrows(a, side, &[side], &[]);
            assert_eq!(p.0, side.0, "along the left edge");
            assert!((p.1 - side.1).abs() >= ARROW as f32);
            assert_eq!(beside_arrows(a, at, &[(300.0, at.1)], &[]), at, "nothing in the way: it stays");
        }
    }
}
