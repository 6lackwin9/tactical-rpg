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
//! REVEAL1: a `roof` prop draws over the party and the walls and fades (180 ms, instant under reduce motion) while the
//! party stands in its indoor zone (`props::RoofFade`); the walls beside an indoor zone draw its `wall_set` first; past
//! the map, the boundary's edge dressing (`props/<set>/boundary_<boundary>/`) repeats over the border, never over the map.
//! Which zone the party is in, which walls and props belong to it and which boundary applies are the kit's
//! (`explore::indoor_zone_at`, `explore::wall_set_at`, `explore::roof_zone`, `art::boundary_of`); this file draws.
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
use aegis_ui::controls::Controls;
use aegis_ui::painter::{Snapshot, View as PView};
use aegis_ui::ui::{draw_glyph, tip, Gfx, Glyph, Ui, Widget, BODY, H, SMALL, W};
use aegis_ui::widget_plumbing;
use aegis_ui::float_panel::{FloatPanel, PANEL_BTN};
use aegis_core::json::{self, s, s_or, Dict};
use emberwake_world::explore::{self, Pos};
use emberwake_world::quest;
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use emberwake_world::GameState;
#[cfg(test)]
use std::time::Duration;
use aegis_core::clock::Instant;

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
    /// Minimize the legend to its "Here (N)" button, or open it again (`map_legend_open`).
    Legend,
    /// Show the last events in the ticker, or only the latest (L).
    Log,
    /// FLOATPANELS1: lock or unlock the legend, the ticker; minimize the ticker or open it again.
    LegendLock,
    LogLock,
    LogMin,
    LogOpen,
    /// FLOATPANELS1: a click on a floating panel's body (kept off the map under it).
    Panel,
    /// MAPNAV1-U: a legend row or an edge arrow: the camera looks at the marker (a second click on the row while it
    /// looks there walks there, as a marker click).
    Pan(String),
    /// MAPNAV1-U: the next standing active member leads (Q, or a click on the leader).
    Lead,
    /// BUNKS1 BK4: a click on a member idling in the hub, or on an occupied bunk: the member's card.
    Member(String),
    /// FOOTLOCKER1: a click on a bunk's footlocker: it opens and its member's loadout opens (the gear page over the
    /// shared stash).
    Locker(String),
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
    /// STYLE-PASS1B (`PR7-48`, FB-366): the tile a click-to-walk heads for (a marker's own tile for a walk-and-talk);
    /// its amber ring shows while the walk lasts (`walk_ring`).
    walk_dest: Option<Pos>,
    next_walk: Instant,
    /// WALK2, VIEWKEYS1: the time between steps, read from the settings each frame.
    step: mapview::WalkStep,
    /// WALK2: a movement key pressed on this map is held: the draw steps every `step` until it is let go.
    hold: bool,
    /// Map geometry of the last frame.
    geo: MapGeom,
    /// The ticker shows the last `LOG_OPEN` events (else only the latest).
    log_open: bool,
    /// FLOATPANELS1 (`preferences/ui.md` "Floating panels"): the legend and the ticker float (unlock, drag, minimize in
    /// place to a named button); their place and lock kept in the settings (`LEGEND_PANEL_KEY`, `LOG_PANEL_KEY`), the
    /// legend's minimized state as `map_legend_open`. The baselines kept and the sizes of the last frame.
    legend_panel: FloatPanel,
    legend_kept: Option<Value>,
    legend_size: (i32, i32),
    log_panel: FloatPanel,
    log_kept: Option<Value>,
    log_size: (i32, i32),
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
    /// ENGINE-50 (MAP-FRAME1): the map's still layers (ground, tiles, walls, border, dressing, shadows, ground props) as
    /// last drawn; under a moving camera only what comes into view is drawn (`Painter::scroll_begin`).
    floor: Snapshot,
    /// The data the floor was keyed on, held so that a reload cannot reuse its address (`floor_key`).
    floor_data: Option<Arc<emberwake_world::Data>>,
    /// ENGINE-50: the markers' views and services for one game state (`MarkerMemo`).
    marker_memo: MarkerMemo,
    /// REGIONVIEW1: this frame's wall time (set by the tick; None before the first: no time has passed, all at rest).
    frame_at: Option<Instant>,
    /// The scheduled time of a step taken since the last frame.
    step_at: Option<Instant>,
    /// ENGINE-50 (MAP-FRAME1): a key press's step since the last frame, dated at that frame (`tick_at` dates it now when
    /// that frame was long ago).
    press_at: Option<Instant>,
    /// ENGINE-51 (SWITCH-FEEL1): the location and view scale the world map's background was last asked ahead for
    /// (`worldmap::want_background`); None before this map's first frame, then empty until its second.
    map_wanted: Option<(String, f32)>,
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
    /// REVEAL1 1.3: each indoor zone's roof opacity over frames.
    roofs: crate::props::RoofFade,
    /// BUNKS1 BK9-BK12: in a hub, where each member idles (re-placed on entry, traded on a lead pass).
    hub: crate::bunks::HubState,
    /// BUNKS1 BK2, BK8: the bunks drawn last frame (agent link `state.bunks`, tests).
    bunks_drawn: Vec<Value>,
    /// BUNKS-LAYOUT3 (1): the dressed north wall runs drawn last frame (the wall band and its plank strip).
    bunk_walls: Vec<Rect>,
    /// BUNKS-UPGRADE-PROPS1: the props holding a bunk's place drawn last frame (`{prop, until_bunk, file}`).
    bunk_held: Vec<Value>,
    /// ENGINE-49 (WALK-FEEL1): the last frame's party cell, drawn place, camera and the camera's wanted place
    /// (`Screen::motion`); null with no map drawn.
    motion: Value,
    /// FOOTLOCKER1 (`PR7-51`): the footlockers drawn last frame (member, the tile it stands on, its logical rect, its
    /// file), and the member whose footlocker is open (their loadout is open over the map; "" none: it closes when the
    /// map is the top screen again).
    lockers: Vec<(String, Pos, Rect, String)>,
    locker_open: String,
    /// FOOTLOCKER1B (`PR7-58` (2)): the footlocker's beat (`bunks::LOCKER_BEAT_MS`): when it ends and whether it is the
    /// opening one (the open art shows, then the loadout comes up) or the closing one (the open art after the loadout
    /// closed, then the closed art); and whether the loadout it opened is up (the map ticks again only once it closed).
    locker_beat: Option<(Instant, bool)>,
    locker_up: bool,
    /// TRANSITIONS1 T2 step 4: the arrival card showing since when, and whether it opened the ticker for its change
    /// lines (closed again when the card goes).
    arrival: Option<(crate::transitions::Arrival, Instant)>,
    ticker_forced: bool,
    /// PR7-76 (PT-44, PT-51): the marker the last step set a flag on (a point of interest it answered for): the walk's
    /// arrival does not act on it again. So a point with both a use and `sets_flags` would skip its use on a click walk's
    /// arrival (no shipped point has both; see `point_listed`).
    point_fired: Option<String>,
    /// PR7-76 (PT-55): a click walk a fight broke off (the location, the party's tile, the walk's tile and its marker),
    /// taken up again after a win (`tick_at`).
    resume: Option<(String, Pos, Pos, Option<String>)>,
    /// ENGINE-53 (SWITCH-AHEAD1): where the party stands still (location, tile) and for how many frames; the nearest
    /// fight is started ahead once there (`ahead_fight`), and its drawing comes on the next frame.
    /// ENGINE-54 (WARM-SPREAD1): since when (and whether its fight was looked for).
    still: (String, Pos, Option<Instant>, bool),
    rehearse: Option<Rehearsal>,
    /// ENGINE-54: the fights this map has drawn unseen already (their art and board ground stay in the caches, so
    /// another start of one, from another tile, is only started ahead, not drawn again).
    rehearsed: Vec<Dict>,
    /// ENGINE-53: the click walk (its tile, and where it started) whose fight was last looked for.
    ahead_walk: Option<(Option<Pos>, Pos)>,
    /// ENGINE-54: the thread starting a fight ahead, and whether its warm draw follows.
    ahead_job: Option<(crate::battle::FightAheadJob, bool)>,
}

/// ENGINE-54 (WARM-SPREAD1): how long the party stands still (no walk, no step showing) before the nearest fight is
/// started ahead and warmed.
const AHEAD_STILL_MS: u64 = 300;

/// One frame's camera as the agent link reports it.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct CamShot {
    px: (i32, i32),
    party: (i32, i32),
    scale: f32,
}

/// ENGINE-49 (WALK-FEEL1): how far back a click walk's first step may be dated (the frame before, at 60 frames a second
/// or more; after a longer frame it is dated now, as before).
const CLICK_STEP_BACK: std::time::Duration = std::time::Duration::from_millis(25);

/// The frames `state.camera.trail` keeps.
const CAM_TRAIL: usize = 240;

/// Starts a world fight: the battle screen, or the Skip / Fled / Lost popup in a debug game.
pub fn begin_fight(cx: &mut Cx, fight: &Dict, runner: &str) {
    if let Err(e) = try_fight(cx, fight, runner) {
        show_errors(cx, vec![e]);
    }
}

/// What `try_fight` opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FightStart {
    /// The battle (or the debug game's fight popup).
    Battle,
    /// PR7-11: the Retire / Last Stand prompt (`FusePrompt`); the fight comes after the choice.
    FusePrompt,
}

/// `begin_fight` that hands the reason back when the battle cannot start (the story stage shows it itself); the prompt
/// starts the fight itself once the choice is made.
pub fn try_fight(cx: &mut Cx, fight: &Dict, runner: &str) -> Result<FightStart, String> {
    open_fight(cx, fight, runner, true)
}

/// PR7-11: `try_fight` for the story stage: a prompt the start opens only closes after the choice, and the stage runs
/// its fight step again (the battle, or the reason on the stage's error card, never a wait on nothing).
pub fn try_story_fight(cx: &mut Cx, fight: &Dict, runner: &str) -> Result<FightStart, String> {
    open_fight(cx, fight, runner, false)
}

fn open_fight(cx: &mut Cx, fight: &Dict, runner: &str, prompt_starts: bool) -> Result<FightStart, String> {
    // FZ4 (PK3 3.1, PR7-11): an active Living Martyr whose fuse is spent: the Retire / Last Stand prompt first, but
    // only for a fight that would start (the other refusals come first, so no choice is made for nothing; the kit's
    // battle start refuses until the choice, `roster::fuse_refusal`)
    if let Some(member) = emberwake_world::roster::fuse_choices(&cx.data, cx.gs()).first().cloned() {
        let why = emberwake_world::battle::start_refusal(&cx.data, cx.gs(), &s(fight, "encounter"));
        if !why.is_empty() {
            return Err(why);
        }
        cx.nav.push(Nav::Push(Box::new(FusePrompt::new(fight, runner, &member, prompt_starts))));
        return Ok(FightStart::FusePrompt);
    }
    // a debug game starts from the Scene Runner, so only a build with the dev tools has one
    #[cfg(feature = "dev_tools")]
    if cx.debug {
        cx.nav.push(Nav::Push(Box::new(crate::runner::DebugFight::new(fight, runner))));
        return Ok(FightStart::Battle);
    }
    let b = crate::battle::BattleScreen::world(cx, fight, runner)?;
    cx.nav.push(Nav::Push(Box::new(b)));
    Ok(FightStart::Battle)
}

/// FZ4 (PK3 3.1, PR7-11): the choice a Living Martyr with a spent fuse needs before its next deployment.
#[derive(Debug, Clone)]
pub enum FuseAct {
    /// `roster::retire`: the member departs `retired` (gear to the stash); the fight then starts without it.
    Retire,
    /// `roster::last_stand`: the member deploys with its template's `last_passive` and dies at the battle's settle.
    LastStand,
}

/// FZ4 (PK3 3.1, PR7-11): the Retire / Last Stand prompt `try_fight` opens beside BK6's `party_min` refusal when
/// `roster::fuse_refusal` fires: the Martyr's portrait (`roster::portrait_of` through `hub::member_portrait`), the
/// refusal text and its fuse, and the two choices; either one goes on to the fight (`fight_now`, which asks again for
/// the next spent Martyr; the story stage runs its step again instead). It opens only for a fight that would start
/// (`battle::start_refusal` first), and refuses a Retire that fight would then refuse. No way back: the deployment
/// waits on the choice.
pub struct FusePrompt {
    fight: Dict,
    runner: String,
    member: String,
    /// The prompt starts the fight after the choice (`begin_fight`); false for the story stage, which runs its fight
    /// step again when the prompt closes (`try_story_fight`).
    starts: bool,
    /// A refused choice's reason (a closed roster refuses Retire, and so does a Retire the fight's `party_min` refuses).
    message: String,
    faces: crate::portraits::Faces,
    w: Vec<Widget<FuseAct>>,
}

/// The prompt's frame and its portrait side.
const FUSE_BOX: (i32, i32) = (320, 132);
const FUSE_PORTRAIT: i32 = 72;

impl FusePrompt {
    pub fn new(fight: &Dict, runner: &str, member: &str, starts: bool) -> Self {
        Self { fight: fight.clone(), runner: runner.to_string(), member: member.to_string(), starts, message: String::new(), faces: Default::default(), w: Vec::new() }
    }

    /// Why member `self.member` can't retire for this fight ("" = it can): the roster's refusal, or the fight's start
    /// refusal without it (BK6's `party_min`, no active member left), tried on a copy of the game.
    fn retire_refusal(&self, cx: &Cx) -> String {
        let d = &cx.data;
        let mut trial = cx.gs().clone();
        if let Err(e) = emberwake_world::roster::retire(d, &mut trial, &self.member) {
            return e;
        }
        let why = emberwake_world::battle::start_refusal(d, &trial, &s(&self.fight, "encounter"));
        if why.is_empty() {
            return why;
        }
        let name = cx.gs().party.member(&self.member).map(|m| emberwake_world::progression::display_name(d, m)).unwrap_or_default();
        format!("{name} can't retire before this fight: {why}")
    }

    fn act(&mut self, cx: &mut Cx, a: FuseAct) {
        let d = cx.data.clone();
        let name = cx.gs().party.member(&self.member).map(|m| emberwake_world::progression::display_name(&d, m)).unwrap_or_default();
        let done = match a {
            FuseAct::Retire => match self.retire_refusal(cx) {
                why if !why.is_empty() => Err(why),
                _ => emberwake_world::roster::retire(&d, cx.gs_mut(), &self.member).map(|_| format!("{name} retires; the fuse is spent.")),
            },
            FuseAct::LastStand => emberwake_world::roster::last_stand(&d, cx.gs_mut(), &self.member).map(|()| format!("{name} goes on a last stand.")),
        };
        match done {
            Err(e) => self.message = e,
            Ok(line) => {
                cx.log(line);
                cx.nav.push(Nav::Pop);
                if self.starts {
                    fight_now(cx, &self.fight, &self.runner);
                }
            }
        }
    }
}

impl Screen<GameCx> for FusePrompt {
    fn id(&self) -> &'static str {
        "fuse_prompt"
    }
    fn overlay(&self) -> bool {
        true
    }
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        let mut ui: Ui<FuseAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().scrim);
        let r = Rect::new((W - FUSE_BOX.0) / 2, (H - FUSE_BOX.1) / 2, FUSE_BOX.0, FUSE_BOX.1);
        ui.frame(r, "The fuse is spent", false);
        let Some(m) = gs.party.member(&self.member) else {
            self.w = ui.widgets;
            return;
        };
        self.faces.begin();
        let pr = Rect::new(r.x + 10, r.y + 24, FUSE_PORTRAIT, FUSE_PORTRAIT);
        let (who, look) = crate::hub::member_portrait(&d, &gs, m);
        self.faces.draw(&mut ui.p, cx, &who, &look, "roster", pr);
        let name = emberwake_world::progression::display_name(&d, m);
        ui.push("fuse.portrait", "portrait", name.clone(), pr, true, false, None, Vec::new());
        let tx = (pr.x + pr.w + 10) as f32;
        let tw = (r.x + r.w - 10) as f32 - tx;
        ui.p.text_fit(tx, (r.y + 30) as f32, &name, BODY, tw, th().text);
        let why = emberwake_world::roster::fuse_refusal(&d, &gs);
        let mut y = (r.y + 46) as f32;
        for line in ui.p.wrap(&why, SMALL, tw) {
            ui.p.text(tx, y, &line, SMALL, th().dim);
            y += 12.0;
        }
        if let Some(def) = emberwake_world::roster::fuse_def(&d, m) {
            ui.p.text(tx, y + 4.0, &format!("Fuse {} of {}", emberwake_world::roster::fuse_value(&d, m, gs.time), def.max), SMALL, th().text);
        }
        if !self.message.is_empty() {
            ui.p.text_fit(tx, (r.y + r.h - 40) as f32, &self.message, SMALL, tw, th().foe);
        }
        let by = r.y + r.h - 26;
        ui.button_tip("fuse.retire", Rect::new(r.x + r.w - 190, by, 86, 18), "Retire", FuseAct::Retire, true, tip("Retire", "They leave the cell alive; their gear goes to the stash."));
        ui.button_tip("fuse.last_stand", Rect::new(r.x + r.w - 96, by, 86, 18), "Last Stand", FuseAct::LastStand, true, tip("Last Stand", "They fight once more, hardened, and die when the battle ends."));
        self.faces.tick();
        self.w = ui.widgets;
    }
    fn back(&mut self, _cx: &mut Cx) {}
    fn state(&self, cx: &Cx) -> Value {
        let d = &cx.data;
        let m = cx.gs().party.member(&self.member);
        let portrait = m.map(|m| emberwake_world::roster::portrait_of(d, m)).unwrap_or_default();
        json!({"member": self.member, "portrait": portrait, "refusal": emberwake_world::roster::fuse_refusal(d, cx.gs()), "message": self.message,
               "encounter": s(&self.fight, "encounter")})
    }
    widget_plumbing!();
}

/// Logs "Fight: <name>!" and starts world fight `fight` (`begin_fight`; PR7-11: the log waits while a spent fuse's
/// prompt opens first).
pub fn fight_now(cx: &mut Cx, fight: &Dict, runner: &str) {
    let name = s_or(cx.data.resolve(&cx.gs().world_id, "encounters", &s(fight, "encounter")), "name", "Enemies");
    // PR7-11: not while a spent fuse's prompt comes first (it logs this when the choice starts the fight)
    if emberwake_world::roster::fuse_choices(&cx.data, cx.gs()).is_empty() {
        cx.log(format!("Fight: {name}!"));
    }
    begin_fight(cx, fight, runner);
}

/// PR7-41 (PT-34): a hub bunk's widget label: its member's name (never the id), or "Empty bunk".
fn bunk_label(d: &emberwake_world::Data, gs: &emberwake_world::GameState, member: &str) -> String {
    match gs.party.member(member) {
        Some(m) => emberwake_world::progression::display_name(d, m),
        None if member.is_empty() => "Empty bunk".into(),
        None => member.to_string(),
    }
}

/// Event lines for the log; `autosave` writes the autosave.
/// PR7-37 (PT-29): `line` logged unless it is the last line already (a point stepped onto, then acted on).
fn log_once(cx: &mut Cx, line: String) {
    if cx.log.last() != Some(&line) {
        cx.log(line);
    }
}

/// PR7-37 (PT-29): the jobs that turned ready since `before` (a flag a point set finished one) say so.
pub(crate) fn log_newly_ready(cx: &mut Cx, before: &[String]) {
    let (d, gs) = (&cx.data, cx.gs());
    let lines: Vec<String> = quest::quests_with(d, gs, "ready")
        .into_iter()
        .filter(|q| !before.contains(q))
        .map(|q| format!("Quest updated: {} is ready to turn in.", s_or(&quest::resolved(d, gs, &q), "name", &q)))
        .collect();
    for line in lines {
        cx.log(line);
    }
}

/// PR7-37 (PT-03, PT-29): the party acts on point of interest `m` (E facing it or on it, or a click on its marker):
/// its line (`explore::point_line`), then its use (`explore::point_use`): `rest` rests there as Make camp does (a
/// refusal says why; a rest that would end the run asks first), `crypts` says who lies in the crypt places, `stash`
/// opens the party stash (the Character window's Gear tab).
pub fn act_on_point(cx: &mut Cx, m: &Dict) {
    log_once(cx, explore::point_line(m));
    match explore::point_use(m).as_str() {
        "rest" => {
            let d = cx.data.clone();
            let refusal = emberwake_world::progression::can_camp(&d, cx.gs());
            if !refusal.is_empty() {
                cx.log(refusal.to_string());
                return;
            }
            let days = emberwake_world::campaign::purse::rest_rule(&d, &cx.gs().world_id).days as i64;
            match crate::runover::days_ask(cx, days, "Resting", "Rest") {
                Some(q) => cx.ask_no(q, rest_here),
                None => rest_here(cx),
            }
        }
        "crypts" => {
            let (d, gs) = (cx.data.clone(), cx.gs().clone());
            let name = |id: &str| gs.party.member(id).map(|m| emberwake_world::progression::display_name(&d, m)).unwrap_or_else(|| id.to_string());
            let line = emberwake_world::roster::crypt_words(&gs, &name);
            if !line.is_empty() {
                cx.log(line);
            }
        }
        "stash" => hud_open(cx, Hud::Stash, &[]),
        _ => {}
    }
}

/// PR7-37 (PT-29): a rest at a point of interest (`progression::make_camp`, the world map's Make camp): its lines.
fn rest_here(cx: &mut Cx) {
    let d = cx.data.clone();
    let res = emberwake_world::progression::make_camp(&d, cx.gs_mut());
    if !json::b(&res, "ok", false) {
        for e in json::sget(&res, "errors") {
            cx.log(e);
        }
        return;
    }
    log_events(cx, json::aget(&res, "events"));
    crate::runover::take(cx, &res);
}

pub fn log_events(cx: &mut Cx, events: &[Value]) {
    for e in events {
        let e = json::dict(e);
        match s(e, "kind").as_str() {
            "entered" => {
                let loc = s(e, "location");
                let name = cx.location_name(&cx.gs().world_id.clone(), &loc);
                // PR7-41 (PT-34): one arrival line: a travel's arrival enters the place, and the world map's Enter
                // enters it again; the second "Entered" line is dropped while it is the last one logged
                let line = format!("Entered {name}.");
                if cx.log.iter().rev().find(|l| l.starts_with("Entered ")) != Some(&line) {
                    cx.log(line);
                }
                for q in json::sget(e, "advanced") {
                    let qn = s_or(&quest::resolved(&cx.data, cx.gs(), &q), "name", &q);
                    cx.log(format!("Quest updated: {qn}."));
                }
            }
            "rested" => cx.log(format!("The party rests (+{} HP).", json::i(e, "restored", 0))),
            "locked" => cx.log(explore::locked_words(&s(e, "text"))),
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
            // PR7-37 (PT-29): a point the party stepped onto that set a flag says so (its line)
            "flags_set" => {
                let loc = explore::location(&cx.data, cx.gs()).clone();
                if let Some(m) = explore::markers_of(&loc).into_iter().find(|m| s(m, "id") == s(e, "marker") && s(m, "kind") == "point") {
                    log_once(cx, explore::point_line(&m));
                }
            }
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
            walk_dest: None,
            next_walk: Instant::now(),
            step: mapview::WalkStep::of(&cx.settings.v, cx.held.shift),
            hold: false,
            geo: MapGeom::default(),
            log_open: false,
            legend_panel: FloatPanel::default(),
            legend_kept: None,
            legend_size: (0, 0),
            log_panel: FloatPanel::default(),
            log_kept: None,
            log_size: (0, 0),
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
            floor: Snapshot::default(),
            floor_data: None,
            marker_memo: MarkerMemo::default(),
            frame_at: None,
            press_at: None,
            map_wanted: None,
            step_at: None,
            leg: mapview::Leg::plain(0.0),
            last_step: None,
            glide: mapview::Glide::default(),
            follow: mapview::Follow::default(),
            cam: CamShot::default(),
            cam_trail: VecDeque::new(),
            opened_at: Instant::now(),
            roofs: crate::props::RoofFade::default(),
            hub: crate::bunks::HubState::default(),
            bunks_drawn: Vec::new(),
            bunk_walls: Vec::new(),
            bunk_held: Vec::new(),
            motion: Value::Null,
            lockers: Vec::new(),
            locker_open: String::new(),
            locker_beat: None,
            locker_up: false,
            arrival: None,
            ticker_forced: false,
            point_fired: None,
            resume: None,
            still: (String::new(), (0, 0), None, false),
            rehearse: None,
            rehearsed: Vec::new(),
            ahead_walk: None,
            ahead_job: None,
        }
    }

    /// ENGINE-53 (SWITCH-AHEAD1), ENGINE-54 (WARM-SPREAD1): a fight the party is about to walk into, started ahead on a
    /// thread (`FightAhead::spawn`; nothing of it on the frame) and kept as the game's `fight_ahead`. During a click walk,
    /// once per walk: its steps tried on a copy of the game and the fight they end in (as `explore::step` tells it)
    /// started on that copy. Standing still `AHEAD_STILL_MS` on a tile (no walk, no step showing, nothing open), once per
    /// tile: the same for the walk to the nearest encounter marker, and a second start warmed unseen in slices
    /// (`warm_next`, `Rehearsal`), so its bodies, board ground and panels are in their caches. A walk that goes another
    /// way, or anything else that changes the game first, leaves it stale (`FightAhead::fits`); a fight that starts
    /// before the thread is done never waits on it.
    fn ahead_fight(&mut self, cx: &mut Cx) {
        if self.ahead_job.as_ref().is_some_and(|(job, _)| job.done()) {
            if let Some((job, draw)) = self.ahead_job.take() {
                self.take_ahead(cx, job, draw);
            }
        }
        let Some(g) = cx.game.as_ref() else { return };
        if self.modal() || g.busy || g.sample {
            self.still.2 = None;
            return;
        }
        let (loc, at) = (g.gs.location_id.clone(), g.gs.position);
        if self.still.0 != loc || self.still.1 != at {
            self.still = (loc, at, None, false);
        }
        if !self.walk.is_empty() {
            if self.ahead_walk.is_none_or(|w| w.0 != self.walk_dest) {
                self.ahead_walk = Some((self.walk_dest, at));
                let path = self.walk.clone();
                self.start_ahead(cx, path, false);
            }
            return;
        }
        self.ahead_walk = None;
        if self.moving() {
            self.still.2 = None;
            return;
        }
        let since = *self.still.2.get_or_insert_with(Instant::now);
        if self.still.3 || Instant::now().saturating_duration_since(since).as_millis() < AHEAD_STILL_MS as u128 {
            return;
        }
        self.still.3 = true;
        if let Some(path) = self.nearest_fight_walk(cx) {
            self.start_ahead(cx, path, true);
        }
    }

    /// ENGINE-53: the walk (its steps) to the nearest present encounter marker, as a click on it walks.
    fn nearest_fight_walk(&self, cx: &Cx) -> Option<Vec<Pos>> {
        let d = &cx.data;
        let gs = cx.gs();
        let loc = explore::location(d, gs);
        let t = explore::template_of(d, loc);
        let steps = gs.explore_int("loc_steps");
        let mut found: Vec<(i64, Pos)> = explore::markers_of(loc)
            .iter()
            .filter(|m| s(m, "kind") == "encounter" && explore::presence(d, gs, &s(loc, "_qid"), m) == "present")
            .map(|m| explore::marker_pos(t, m, steps))
            .map(|p| ((p.0 - gs.position.0).abs() + (p.1 - gs.position.1).abs(), p))
            .collect();
        found.sort();
        let (_, nearest) = *found.first()?;
        self.path_to(cx, nearest, false).filter(|p| !p.is_empty())
    }

    /// ENGINE-54: starts the thread that tries `path`'s steps on a copy of the game and starts the fight they end in
    /// (twice with `draw`: the second for the warm draw); one at a time (a newer one replaces it, the older left to end
    /// on its own).
    fn start_ahead(&mut self, cx: &Cx, path: Vec<Pos>, draw: bool) {
        let Some(g) = cx.game.as_ref() else { return };
        let gs = g.gs.clone();
        let job = crate::battle::FightAhead::spawn(cx.data.clone(), "explore", draw, move |d| {
            let mut after = gs;
            for dir in path {
                let res = explore::step(d, &mut after, dir);
                if !json::b(&res, "ok", false) {
                    return None;
                }
                let events = json::aget(&res, "events");
                if let Some(e) = events.iter().map(json::dict).find(|e| s(e, "kind") == "fight") {
                    return Some((after, e.clone()));
                }
                if events.iter().map(json::dict).any(|e| ["exit", "blocked", "locked"].contains(&s(e, "kind").as_str())) {
                    return None;
                }
            }
            None
        });
        self.ahead_job = job.map(|j| (j, draw));
    }

    /// ENGINE-54: a done thread's fight kept as the game's `fight_ahead` (unless one kept already fits, or a fight is
    /// on), and with `draw` its second start handed to the warm draw.
    fn take_ahead(&mut self, cx: &mut Cx, job: crate::battle::FightAheadJob, draw: bool) {
        let Some(found) = job.take() else { return };
        let Some(g) = cx.game.as_mut() else { return };
        if g.busy {
            return;
        }
        let slot = g.slot;
        if !g.fight_ahead.as_ref().is_some_and(|a| a.fits(&found.gs, &found.fight, "explore")) {
            g.fight_ahead = Some(found.ahead);
        }
        if draw && !self.rehearsed.iter().any(|f| f == &found.fight) {
            if let Some(second) = found.second {
                self.rehearsed.push(found.fight.clone());
                self.rehearse = Some(Rehearsal::fight(found.gs, slot, found.fight, "explore", second));
            }
        }
    }

    /// TRANSITIONS1 T2 step 4: a won fight's arrival (left by its results) shows its card; its change lines join the
    /// log and open the ticker while the card shows. Once the card's time is up it goes (and the ticker it opened
    /// closes).
    fn follow_arrival(&mut self, cx: &mut Cx, now: Instant) {
        if let Some(mut a) = cx.game.as_mut().and_then(|g| g.arrival.take()) {
            // the card reads the hub as it opens (a story's `enter` step moves the party after the results closed)
            a.card = crate::transitions::card(&cx.data, cx.gs());
            for line in &a.changes {
                cx.log(line.clone());
            }
            if !a.changes.is_empty() && !self.log_open {
                self.log_open = true;
                self.ticker_forced = true;
            }
            self.arrival = Some((a, now));
        }
        if self.arrival.as_ref().is_some_and(|(_, t)| now.saturating_duration_since(*t).as_millis() as u64 >= crate::transitions::ARRIVAL_MS) {
            self.arrival = None;
            if std::mem::take(&mut self.ticker_forced) {
                self.log_open = false;
            }
        }
    }

    /// BUNKS1 BK8: plays member `id`'s waiting talk as a scene (`talks::play`: the seen flag, the reward); false when
    /// none waits.
    fn play_talk(&mut self, cx: &mut Cx, id: &str) -> bool {
        let d = cx.data.clone();
        let Some(w) = emberwake_world::talks::waiting_for(&d, cx.gs(), id) else { return false };
        self.walk.clear();
        let (r, reward) = emberwake_world::talks::play(&d, cx.gs_mut(), &w);
        if r.closed {
            show_errors(cx, r.errors.clone());
            return true;
        }
        if !reward.is_empty() {
            cx.log(format!("Received {}.", d.item_name(&reward)));
        }
        self.svc.open_runner(cx, r);
        true
    }

    /// BUNKS1 BK8: the card's Talk answer (`bunks::TALK_ANSWER`), the member it names.
    fn take_talk_answer(cx: &mut Cx) -> Option<String> {
        let at = cx.answers.iter().position(|a| a.starts_with(crate::bunks::TALK_ANSWER))?;
        let a = cx.answers.remove(at);
        Some(a[crate::bunks::TALK_ANSWER.len()..].to_string())
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
        self.point_fired = None;
        let d = cx.data.clone();
        let from = cx.gs().position;
        // PR7-37 (PT-29): the jobs ready before the step, so a flag a point sets says what it changed (taken before the
        // step: whether a flag is set is known only after it; a list of ids, read once per step)
        let ready_before = quest::quests_with(&d, cx.gs(), "ready");
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
            // ENGINE-56 (AUDIO-FEEL2): a soft footstep on each step, at its own volume under Effects
            cx.audio.play_gain(aegis_ui::audio::Bus::Sfx, crate::mapview::MAP_STEP, crate::mapview::footstep_gain(&cx.settings.v));
        }
        if !json::b(&res, "ok", false) {
            for e in json::sget(&res, "errors") {
                cx.log(e);
            }
            return false;
        }
        let events = json::aget(&res, "events").to_vec();
        log_events(cx, &events);
        if let Some(e) = events.iter().map(json::dict).find(|e| s(e, "kind") == "flags_set") {
            log_newly_ready(cx, &ready_before);
            // PR7-76 (PT-44, PT-51): the point answered on this step (its line, the quest's); the arrival leaves it be
            self.point_fired = Some(s(e, "marker"));
        }
        crate::runover::take(cx, &res);
        let mut go_on = true;
        for e in &events {
            let e = json::dict(e);
            match s(e, "kind").as_str() {
                // CAMPAIGN1 W3: a marker fight that could end the run asks first (No: the party steps back); a fight
                // that cannot be refused gets the notice
                "fight" => {
                    go_on = false;
                    // PR7-76 (PT-55): a click walk still under way goes on after a win
                    self.keep_walk(cx);
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
                // PR7-76 (PT-54): a road's exit opens the world map with that road's next place picked
                "exit" if s(e, "to") == explore::WORLD_MAP => {
                    let map = crate::worldmap::WorldMap::toward(cx, &s(e, "marker"));
                    cx.nav.push(Nav::Push(Box::new(map)));
                    go_on = false;
                }
                "exit" | "blocked" | "locked" => go_on = false,
                _ => {}
            }
        }
        go_on
    }

    /// PR7-76 (PT-55): keeps the click walk under way (its tile and marker, where the party stands now) for after a
    /// fight; nothing for a key step or a walk already at its tile. A click walk is under way while steps are left on its
    /// path or its marker waits to be reached (`walk_dest` alone is not enough: a finished walk leaves it behind).
    fn keep_walk(&mut self, cx: &Cx) {
        let at = cx.gs().position;
        let under_way = !self.walk.is_empty() || self.walk_target.is_some();
        self.resume = match self.walk_dest {
            Some(dest) if under_way && (dest != at || self.walk_target.is_some()) => Some((cx.gs().location_id.clone(), at, dest, self.walk_target.clone())),
            _ => None,
        };
    }

    /// PR7-76 (PT-55): after a won fight with the party where it was found, the walk it broke off goes on (to its marker,
    /// else its tile); a loss, a flight (the party stepped back) or a move elsewhere drops it.
    fn resume_walk(&mut self, cx: &mut Cx) {
        let Some(outcome) = cx.game.as_mut().and_then(|g| g.fought.take()) else { return };
        let Some((loc, at, dest, target)) = self.resume.take() else { return };
        if outcome != "won" || cx.gs().location_id != loc || cx.gs().position != at || self.modal() {
            return;
        }
        match target {
            Some(id) => self.walk_to_marker(cx, &id),
            None => {
                if let Some(path) = self.path_to(cx, dest, false).filter(|p| !p.is_empty()) {
                    self.walk = path;
                    self.walk_target = None;
                    self.walk_dest = Some(dest);
                }
            }
        }
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
    pub(crate) fn tick_at(&mut self, cx: &mut Cx, now: Instant) {
        let prev = self.frame_at.replace(now);
        // ENGINE-50 (MAP-FRAME1): a key press steps as of the last frame, so the next frame shows it moving; no more than
        // `CLICK_STEP_BACK` before this one (as a click): dated back across a long frame, its glide would jump ahead and
        // the walk would take a quick step to catch up
        if let Some(at) = self.press_at.take() {
            let late = now.saturating_duration_since(at);
            if late > CLICK_STEP_BACK && self.last_step == Some(at) {
                let moved = late - CLICK_STEP_BACK;
                self.step_at = self.step_at.map(|a| a + moved);
                self.last_step = Some(at + moved);
                self.next_walk += moved;
            }
        }
        // PLAYUI1-R: the sprites play on their own clock at the animation speed setting (as the battle tokens)
        let speed = mapview::anim_speed(&cx.settings.v);
        // WALKRUN1: Shift held runs; the pace of a step is set as it is taken, so a release counts from the next step
        self.step = mapview::WalkStep::of(&cx.settings.v, cx.held.shift);
        self.anim_ms += now.duration_since(self.anim_at).as_secs_f64() * 1000.0 * speed;
        self.anim_at = now;
        // FOOTLOCKER1B (2): the map ticks again with the loadout a footlocker opened closed: the closing beat starts;
        // a beat ended: the loadout comes up (opening) or the footlocker closes; the map holds still while it opens
        if std::mem::take(&mut self.locker_up) {
            self.locker_beat = Some((now + crate::bunks::locker_beat(), false));
        }
        if let Some((due, opening)) = self.locker_beat {
            if now >= due {
                self.locker_beat = None;
                if opening {
                    self.locker_up = true;
                    cx.nav.push(Nav::Push(Box::new(crate::party::Character::new(cx, &self.locker_open.clone(), "gear"))));
                } else {
                    self.locker_open.clear();
                }
            }
            if opening {
                return;
            }
        }
        // PORTRAITFX1 4.5: the talk box's portrait clock moves; CAMPAIGN1 W3: a Yes to the ask before a resting reply
        // picks it (`Services::tick`)
        if let Some(fought) = self.svc.tick(cx) {
            if fought {
                self.walk.clear();
            }
            return;
        }
        // BUNKS1 BK8: the card's Talk plays the member's waiting talk
        if let Some(id) = Self::take_talk_answer(cx) {
            if !self.modal() {
                self.play_talk(cx, &id);
            }
            return;
        }
        // STORYUI1 W2, 5.3: with nothing open, a running sequence takes over and queued vignettes play; CAMPAIGN1 W1,
        // F3: campaign notices first, then the run-over screen once the run is over; T2: the campaign queue's head
        // plays (vignettes and battles through `story::map_tick`, a dialogue scene here)
        // PR7-37 (PT-31): then a road opened since the last look (a log line and a notice)
        if !self.modal() && (crate::runover::map_tick(cx) || crate::story::map_tick(cx) || self.svc.open_pending(cx) || crate::worldmap::note_new_roads(cx)) {
            self.walk.clear();
            self.walk_target = None;
            return;
        }
        self.resume_walk(cx);
        // WALK2: a held movement key keeps stepping at the step rate (the press itself stepped at once; the OS key
        // repeat is dropped by the shell, `hold_walk`), so the steps chain with the walk clip running.
        if cx.held.dir.is_none() {
            self.hold = false;
        }
        if self.hold && self.walk.is_empty() && !self.modal() {
            if let (Some(dir), true) = (cx.held.dir, now >= self.next_walk) {
                let at = self.step_time(now);
                self.plan_step(cx, at, dir);
                // PR7-76 (PT-55): a key step is no click walk (nothing to resume after a fight it walks into)
                self.walk_dest = None;
                self.walk_target = None;
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
        // ENGINE-49 (WALK-FEEL1): a walk ordered while the party stood (a click since the last frame) takes its first step
        // as of that frame, as a key press does, so the next frame already shows it moving (it was a frame late). Only
        // from a frame just before (`CLICK_STEP_BACK`): after a long frame the step would be dated far back and the walk
        // would run to catch up.
        let at = match prev {
            Some(p) if p >= self.next_walk && p < now && now.saturating_duration_since(p) <= CLICK_STEP_BACK => p,
            _ => self.step_time(now),
        };
        let dir = self.walk.remove(0);
        self.plan_step(cx, at, dir);
        if !self.step(cx, dir) {
            self.walk.clear();
            self.walk_target = None;
            self.walk_dest = None;
            return;
        }
        if self.walk.is_empty() {
            // the walk is over (a marker's walk below sets its own tile again)
            self.walk_dest = None;
            // PR7-76 (PT-44, PT-51): a point the last step already answered is not acted on again
            if let Some(id) = self.walk_target.take().filter(|id| self.point_fired.as_deref() != Some(id.as_str())) {
                self.walk_to_marker(cx, &id);
            }
        }
    }

    /// STYLE-PASS1B (`PR7-48`, FB-366): the tile of the click-to-walk under way, ringed in amber until the walk arrives,
    /// stops or a new order replaces it (none for a key step or a walk already over).
    fn walk_ring(&self) -> Option<Pos> {
        self.walk_dest.filter(|_| !self.walk.is_empty())
    }

    /// Whether the leader walks at the clock's now: on a walked path, or within a step and `WALK_LINGER_MS` of the last step
    /// (that step's own time: Shift pressed during it must not end it early, TURNFEEL1).
    fn moving(&self) -> bool {
        !self.walk.is_empty() || self.moved_at.is_some_and(|(t, step)| self.anim_ms - t < step + WALK_LINGER_MS)
    }

    pub(crate) fn interact(&mut self, cx: &mut Cx, dir: Pos) {
        self.turn(dir);
        // BUNKS1 BK8, BK11: a member idling ahead plays its waiting talk (the lead's too), else opens its card
        let ahead = (cx.gs().position.0 + dir.0, cx.gs().position.1 + dir.1);
        // FOOTLOCKER1B (`PR7-58` (1)): a footlocker ahead first, then what stands ahead (a member, a marker, a point),
        // and a footlocker underfoot last
        if let Some(member) = self.lockers.iter().find(|l| l.1 == ahead).map(|l| l.0.clone()) {
            self.open_locker(cx, &member);
            return;
        }
        let member = self.hub.member_at(ahead);
        if !member.is_empty() {
            if !self.play_talk(cx, &member) {
                cx.nav.push(Nav::Push(Box::new(crate::bunks::BunkCard::new(&member))));
            }
            return;
        }
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
            // PR7-37 (PT-03, PT-29): a point of interest underfoot or ahead answers (its line, then its use)
            _ => match explore::point_near(&d, cx.gs(), dir) {
                Some(m) => act_on_point(cx, &m),
                None => {
                    let here = cx.gs().position;
                    match self.lockers.iter().find(|l| l.1 == here).map(|l| l.0.clone()) {
                        Some(member) => self.open_locker(cx, &member),
                        None => cx.log("Nothing there."),
                    }
                }
            },
        }
    }

    /// FOOTLOCKER1 (`PR7-51`): `member`'s footlocker opens (the open art while the screen is up) and their loadout opens
    /// over the map: the Character window's gear page focused on them, beside the shared stash (gear and consumables).
    /// FOOTLOCKER1B (`PR7-58` (2)): the open art shows for the beat first (the frame's clock); the loadout comes up as it
    /// ends (`tick_at`). A footlocker already in its beat or open takes nothing more.
    fn open_locker(&mut self, _cx: &mut Cx, member: &str) {
        if self.locker_beat.is_some_and(|b| b.1) || self.locker_up {
            return;
        }
        self.walk.clear();
        self.walk_target = None;
        self.hold = false;
        self.locker_open = member.to_string();
        let now = self.frame_at.unwrap_or_else(Instant::now);
        self.locker_beat = Some((now + crate::bunks::locker_beat(), true));
    }

    /// FOOTLOCKER1B (2): a footlocker is in its opening beat (the map takes no input until the loadout is up).
    fn locker_opening(&self) -> bool {
        self.locker_beat.is_some_and(|b| b.1)
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
        // PR7-37 (PT-03, PT-29): a point of interest is acted on once the party stands on it or beside it (the walk
        // goes onto it, or beside it when its tile is walled off)
        if s(&m, "kind") == "point" && explore::presence(&d, gs, &s(loc, "_qid"), &m) == "present" {
            let dir = (at.0 - gs.position.0, at.1 - gs.position.1);
            if dir == (0, 0) || explore::DIRS.contains(&dir) {
                if dir != (0, 0) {
                    self.turn(dir);
                }
                act_on_point(cx, &m);
                return;
            }
            match self.path_to(cx, at, false).filter(|p| !p.is_empty()).or_else(|| self.path_to(cx, at, true).filter(|p| !p.is_empty())) {
                Some(path) => {
                    self.walk = path;
                    self.walk_target = Some(id.to_string());
                    self.walk_dest = Some(at);
                }
                None => cx.log("You can't get there."),
            }
            return;
        }
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
                self.walk_dest = Some(at);
            }
            // PR7-76 (PT-44): already there (a point that went once used): nothing to say
            Some(_) => {}
            None => cx.log("You can't get there."),
        }
    }

    fn act(&mut self, cx: &mut Cx, a: RAct) {
        if self.locker_opening() {
            return;
        }
        self.resume = None;
        let d = cx.data.clone();
        match a {
            RAct::MapClick => {
                let (tx, ty) = self.geo.cell_at(self.mouse.0, self.mouse.1);
                // MAPNAV1-U: a tile no open path reaches (a wall, water, a tile walled off) says so
                match self.path_to(cx, (tx, ty), false) {
                    Some(path) => {
                        self.walk = path;
                        self.walk_target = None;
                        self.walk_dest = Some((tx, ty));
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
            RAct::Member(id) => {
                self.walk.clear();
                cx.nav.push(Nav::Push(Box::new(crate::bunks::BunkCard::new(&id))));
            }
            RAct::Locker(id) => {
                self.walk.clear();
                self.open_locker(cx, &id);
            }
            RAct::Hud(h) => {
                self.walk.clear();
                hud_open(cx, h, &region_help(&crate::controls_view::hints(&cx.settings.v)));
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
            RAct::LegendLock => self.legend_panel.toggle_lock(),
            RAct::LogLock => self.log_panel.toggle_lock(),
            RAct::LogMin => self.log_panel.minimize(),
            RAct::LogOpen => self.log_panel.restore(MAP_AREA, self.log_size),
            RAct::Panel => {}
        }
    }

    fn marker_style(kind: &str, state: &str) -> (Rgba, &'static str) {
        match (kind, state) {
            ("exit", "locked") => (crate::tokens::exit(false), ">"),
            ("exit", _) => (crate::tokens::exit(true), ">"),
            ("chest", "spent") => (rgb(90, 74, 50), "c"),
            ("chest", _) => (rgb(200, 150, 60), "c"),
            ("lore", _) => (rgb(150, 110, 220), "L"),
            ("encounter", _) => (rgb(220, 70, 60), "!"),
            ("roamer", _) => (rgb(230, 110, 60), "R"),
            ("npc", _) | ("member", _) => (rgb(230, 190, 80), ""),
            _ => (rgb(230, 220, 120), "?"),
        }
    }
}

impl Screen<GameCx> for RegionMap {
    /// ENGINE-53 (SWITCH-AHEAD1), ENGINE-54: the nearest fight, started ahead on a thread, warmed unseen in slices.
    fn warm_next(&mut self, cx: &mut Cx) -> Option<Box<dyn Screen<GameCx>>> {
        if let Some(r) = self.rehearse.take() {
            return Some(Box::new(r));
        }
        self.ahead_fight(cx);
        None
    }
    /// The arrow keys walk the party; ENGINE-48 (INPUT-REACH1): while a conversation (its shop, board, recipes or facility)
    /// or a lore card is open, nothing walks, so they move the focus over its replies and rows.
    fn arrow_focus(&self) -> bool {
        self.svc.talking() || self.lore.is_some()
    }
    fn hold_walk(&self) -> bool {
        true
    }
    fn id(&self) -> &'static str {
        "region_map"
    }
    /// ENGINE-64 (LIVE-RELOAD1): on top, with no conversation or card open, the map is built again from the game state
    /// over the new data (where the party stands stays); under another screen it reads the new data as it draws.
    fn data_reloaded(&mut self, cx: &mut Cx, top: bool) -> Option<String> {
        if top && !self.svc.talking() && self.lore.is_none() && cx.has_game() {
            let map = RegionMap::new(cx);
            cx.nav.push(Nav::Reset(Box::new(map)));
        }
        None
    }

    /// ENGINE-49 (WALK-FEEL1): the party's cell, where it is drawn and the camera, for a `feel` replay's movement report.
    fn motion(&self) -> Value {
        self.motion.clone()
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
        // BUNKS1 BK9-BK12: in a hub, the members' idle places (a lead change trades two of them)
        let lead_now = leader_id(cx);
        self.hub.sync(&d, cx.gs_mut(), &lead_now);
        self.bunks_drawn.clear();
        self.bunk_walls.clear();
        self.bunk_held.clear();
        self.lockers.clear();
        let gs = cx.gs().clone();
        // the open conversation; a new node starts its line and its replies scrolled to the top; PORTRAITFX1-D: a
        // portrait left out of the last frame (the conversation ended or hid) is dropped
        let talk_view = self.svc.begin_frame(g, &d, &gs);
        // ENGINE-51: once this map has drawn a frame, the world map's background is read ahead (the map opens from
        // here); not in the first, whose own reads it would slow
        let wanted = (cx.gs().location_id.clone(), g.view.s);
        match &self.map_wanted {
            None => self.map_wanted = Some((String::new(), 0.0)),
            Some(was) if *was != wanted => {
                crate::worldmap::want_background(cx, wanted.1);
                self.map_wanted = Some(wanted);
            }
            Some(_) => {}
        }
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
            let has_ground = self.ground.has();
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
            // REVEAL1 2.2: the location's boundary, else the map's own
            let boundary = emberwake_content::art::boundary_of(loc, t);
            let walls = WallArt::of(&sets, &boundary);
            // REVEAL1 1.1: the tile sets of each indoor zone's wall set (its own first, then the region sets); none on a map
            // without one, so no wall looks for a zone
            let interior: Vec<(String, Vec<String>)> = interior_sets(t, &sets);
            let interior_walls: Vec<(&str, WallArt)> = interior.iter().map(|(set, names)| (set.as_str(), WallArt::of(names, &boundary))).collect();
            let wall_art_at = |x: i32, y: i32| -> &WallArt {
                if interior_walls.is_empty() {
                    return &walls;
                }
                let set = explore::wall_set_at(t, x as i64, y as i64);
                interior_walls.iter().find(|(s, _)| !set.is_empty() && *s == set).map_or(&walls, |(_, w)| w)
            };
            let wall_face_tiles = map_wall_face_tiles(loc, t);
            let dim_ambience = map_dim_ambience(loc, t);
            let freestanding = freestanding_prop_cells(t, (map_w, map_h), &ch_at);
            let is_wall = |x: i32, y: i32| ch_at(x, y) == '#' && !freestanding.contains(&(x, y));
            let mw_i = map_w as i32;
            let mh_i = map_h as i32;
            if !modal {
                ui.push("map", "map", s_or(loc, "name", ""), map_area, true, false, Some(RAct::MapClick), Vec::new());
            }
            // PROPS1 4.2-4.6: the ground props over the tiles; the standing props go into the sorted pass below. Props
            // are art only: no widget, and walking still comes from `rows`
            // BUNKS1 BK2: in a hub each bunk prop draws its state's art (`<art>_<state>`, where that art exists)
            // BUNKS-UPGRADE-PROPS1: the props holding a bunk's place show until the bunk cap reaches it, then the bunk
            let shown_t = if self.hub.on() { emberwake_world::bunks::shown_props(&gs, t) } else { None };
            let base_t = shown_t.as_ref().unwrap_or(t);
            let hub_bunks: Vec<emberwake_world::bunks::Bunk> = if self.hub.on() {
                let shown: Vec<String> = json::aget(base_t, "props").iter().map(|p| s(json::dict(p), "id")).collect();
                emberwake_world::bunks::bunks(&gs, t).into_iter().filter(|b| shown.contains(&b.prop)).collect()
            } else {
                Vec::new()
            };
            let state_t = crate::bunks::state_props(base_t, &hub_bunks, |art, x, y| {
                let dirs = emberwake_content::art::prop_dirs(&lineage, art);
                let dirs: Vec<&str> = dirs.iter().map(String::as_str).collect();
                cx.art.pick(&dirs, None, x as i32, y as i32).is_some()
            });
            let props = crate::props::layout(state_t.as_ref().unwrap_or(base_t), &lineage, (geo.ox, geo.oy), geo.cell_size(), paint_area, |dirs, variant, x, y| {
                let rel = cx.art.pick(dirs, variant, x, y)?;
                let size = cx.art.size_of(&rel)?;
                Some((rel, size))
            });
            if self.hub.on() {
                for raw in json::aget(base_t, "props") {
                    let held = s(json::dict(raw), emberwake_world::bunks::UNTIL_BUNK);
                    let id = s(json::dict(raw), "id");
                    if let Some(p) = props.iter().find(|p| !held.is_empty() && p.id == id) {
                        self.bunk_held.push(json!({"prop": id, "until_bunk": held, "file": p.file}));
                    }
                }
            }
            let prop_arts = collect_prop_arts(t);
            // REGIONVIEW1: past the map is boundary art only: a prop's art that overhangs it is cut there (a footprint
            // cannot leave the map: check P8)
            let map_rect = Rect::new(geo.ox, geo.oy, map_w as i32 * geo.cell_size().0, map_h as i32 * geo.cell_size().1);
            // ENGINE-50 (MAP-FRAME1): the still layers (the ground, tiles, walls, border, dressing, shadows and ground props)
            // are kept from the last frame and moved with the camera; only the strips that came into view are drawn (all
            // of it when anything they show changed: `floor_key`)
            let key = floor_key(cx, &gs.location_id, (view, q), &sets, has_ground, state_t.as_ref().unwrap_or(base_t));
            self.floor_data = Some(cx.data.clone());
            ui.p.v = base_view;
            let strips = ui.p.scroll_begin(&mut self.floor, key, map_area, camera);
            let kept_clip = ui.p.c.clip;
            let ground_fill = th().game("map_ground", rgb(14, 18, 26));
            for strip in strips {
                ui.p.c.clip = Some(strip.px);
                ui.p.v = map_view;
                // the strip in the map's logical px, by one more (the camera's rest is under a logical px)
                let area = Rect::new(strip.area.x - 1, strip.area.y - 1, strip.area.w + 2, strip.area.h + 2);
                ui.p.fill(area, ground_fill);
                self.ground.draw(&mut ui.p, &geo);
                let (x0, x1) = geo.cols(area);
                let (y0, y1) = geo.rows(area);
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        let r = geo.cell(x, y);
                        let ch = ch_at(x, y);
                        let vis_ch = if ch == '#' && freestanding.contains(&(x, y)) { '.' } else { ch };
                        // the border draws after this pass; tilted, a wall is a block in the sorted pass below
                        if border_at(x, y) || (tilted && vis_ch == '#') {
                            continue;
                        }
                        if has_ground && matches!(vis_ch, '.' | '~') {
                            continue;
                        }
                        if !tilted && wall_face_tiles > 0 && vis_ch == '#' {
                            let depth = wall_depth_at(x, y, wall_face_tiles, (mw_i, mh_i), &is_wall);
                            let is_face = |nx: i32, ny: i32| wall_depth_at(nx, ny, wall_face_tiles, (mw_i, mh_i), &is_wall) > 0;
                            let wa = wall_art_at(x, y);
                            if depth > 0 {
                                let up_face = is_face(x, y - 1);
                                draw_wall_face_at(&mut ui, cx, wa, false, x, y, r, r.h, depth == 1 && up_face, !up_face, !is_face(x + 1, y), !is_face(x - 1, y));
                            } else {
                                let tile = emberwake_content::art::region_tile('#').unwrap_or("wall");
                                match cx.art.tile_in(&wa.sets, tile, x, y, art_size) {
                                    Some(img) => ui.p.image(img, r),
                                    None => ui.p.fill(r, WALL_FILL),
                                }
                                draw_wall_cap_edges(&mut ui, r, r.h, !is_wall(x, y - 1), !is_wall(x - 1, y) || is_face(x - 1, y), !is_wall(x + 1, y) || is_face(x + 1, y));
                            }
                            continue;
                        }
                        let tile = emberwake_content::art::region_tile(vis_ch);
                        // REVEAL1 1.1: a wall beside an indoor zone draws its wall set first
                        let tile_sets = if vis_ch == '#' { &wall_art_at(x, y).sets } else { &set_refs };
                        match tile.and_then(|t| cx.art.tile_in(tile_sets, t, x, y, art_size)) {
                            Some(img) => ui.p.image(img, r),
                            None => ui.p.fill(r, match vis_ch {
                                '.' => rgb(70, 90, 64),
                                '#' => WALL_FILL,
                                '~' => rgb(40, 90, 100),
                                _ => rgb(20, 20, 24),
                            }),
                        }
                        if tilted && vis_ch == '~' && wall_face_tiles == 0 {
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
                            if wall_face_tiles > 0 && x >= 0 && y >= 0 && x < mw_i && y < mh_i {
                                let depth = wall_depth_at(x, y, wall_face_tiles, (mw_i, mh_i), &is_wall);
                                let is_face = |nx: i32, ny: i32| wall_depth_at(nx, ny, wall_face_tiles, (mw_i, mh_i), &is_wall) > 0;
                                let r = geo.cell(x, y);
                                if depth > 0 {
                                    let up_face = is_face(x, y - 1);
                                    draw_wall_face_at(&mut ui, cx, &walls, true, x, y, r, r.h, depth == 1 && up_face, !up_face, !is_face(x + 1, y), !is_face(x - 1, y));
                                } else {
                                    draw_border(&mut ui, cx, &walls, x, y, r, art_size, false);
                                    draw_wall_cap_edges(&mut ui, r, r.h, !is_wall(x, y - 1), !is_wall(x - 1, y) || is_face(x - 1, y), !is_wall(x + 1, y) || is_face(x + 1, y));
                                }
                            } else {
                                draw_border(&mut ui, cx, &walls, x, y, geo.cell(x, y), art_size, tilted && !is_wall(x, y + 1));
                            }
                        }
                    }
                }
                // REVEAL1 2.2: the boundary's edge dressing over the cells past the map (art only: no widget)
                if !boundary.is_empty() {
                    draw_dressing(&mut ui, cx, &lineage, &boundary, &geo, (map_w, map_h), (x0, x1), (y0, y1));
                }
                if wall_face_tiles > 0 {
                    draw_depth_floor_shadows_and_water(&mut ui, &geo, (mw_i, mh_i), (x0, x1), (y0, y1), has_ground, &ch_at, &is_wall);
                }
                for p in props.iter().filter(|p| p.ground) {
                    draw_prop(&mut ui, cx, p, None, false, map_rect, q);
                }
                ui.p.v = base_view;
            }
            ui.p.c.clip = kept_clip;
            ui.p.scroll_end(&mut self.floor);
            ui.p.v = map_view;
            // BUNKS1 BK3, BK4, BK8: each bunk's click area (its drawn art; under the markers and the party), its hover
            // name and condition, and whether a talk waits on its member (a candle, drawn over the scene below)
            let waiting = if self.hub.on() { emberwake_world::talks::waiting(&d, &gs) } else { Vec::new() };
            let mut candles: Vec<Rect> = Vec::new();
            for b in &hub_bunks {
                let Some(p) = props.iter().find(|p| p.id == b.prop) else { continue };
                let candle = !b.member.is_empty() && waiting.iter().any(|w| w.has(&b.member));
                let bed = crate::bunks::bed_art(&b.art);
                let drawn = crate::bunks::drawn_rect(&bed, p.rect);
                self.bunks_drawn.push(json!({"prop": b.prop, "member": b.member, "state": b.state.name(), "candle": candle, "file": p.file, "niche": bed.niche, "drawn": [drawn.x, drawn.y, drawn.w, drawn.h], "flip": p.flip}));
                if candle {
                    candles.push(drawn);
                }
                if modal {
                    continue;
                }
                let hit = crate::bunks::drawn_rect(&crate::bunks::bed_art(&b.art), p.rect).intersect(map_area);
                let (name, word) = if ui.hover(hit) { crate::bunks::bunk_tip(&d, &gs, b) } else { (String::new(), String::new()) };
                let act = (!b.member.is_empty()).then(|| RAct::Member(b.member.clone()));
                ui.push(format!("{}{}", crate::bunks::BUNK_PREFIX, b.prop), "bunk", bunk_label(&d, &gs, &b.member), hit, true, false, act, if name.is_empty() { Vec::new() } else { tip(&name, &word) });
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
                let (view, memo_services) = self.marker_memo.get(&d, &gs, loc, &m);
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
                    // MAPNAV1-U: a legend entry out of view gets an edge arrow (PR7-76, PT-43: a usable point too)
                    if kind != "point" || point_listed(&m, &state) {
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
                    services = memo_services;
                    let (body, palette, _) = crate::screens::character_art(&d, who);
                    // 2.4: the person's body, else its class body (the chain), else today's token
                    let sprite = cx.art.first_body(&emberwake_content::art::body_chain(&body));
                    if sprite.is_some() {
                        top = body_rect_of(cx, sprite.as_deref(), r);
                    }
                    // facing the party while in the conversation (four ways, DIR4-M: up or down when the party is
                    // more above or below, keeping their left or right); else the row's `face`, down by default
                    // (NPCFACE1; a body with no down row keeps its side view)
                    let rest = explore::marker_facing(&m);
                    let face = if talk_tiles.contains(&at) { rest.toward((gs.position.0 - at.0) as i32, (gs.position.1 - at.1) as i32) } else { rest };
                    npc = Some(NpcDraw { body, sprite, palette, initial: s(who, "name").chars().next().unwrap_or('?').to_string(), face, offset: idle_offset(&s(who, "id")) });
                }
                // 2.2: the first service is the badge (turn_in and offer are today's `ready` and `quest`)
                marks.push(MarkDraw { id: id.clone(), r, top: top.y, kind: kind.clone(), state: state.clone(), npc, badge: services.first().copied(), glyph: glyph_of(&kind, &explore::point_use(&m)) });
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
            // BUNKS1 BK9: the members idling in the hub, drawn and clicked like the people (their card on a click)
            let mut talk_marks: Vec<(Rect, i32)> = Vec::new();
            for (id, at) in self.hub.spots.clone() {
                let Some(m) = gs.party.member(&id) else { continue };
                let r = geo.cell_of(at);
                let seen = r.intersect(map_area);
                if seen.w <= 0 || seen.h <= 0 {
                    continue;
                }
                let (body, palette) = crate::bunks::member_art(&d, &gs, m);
                let sprite = cx.art.first_body(&emberwake_content::art::body_chain(&body));
                let top = if sprite.is_some() { body_rect_of(cx, sprite.as_deref(), r) } else { mapview::icon_rect(r) };
                let name = crate::hub::member_name(cx, m);
                let key = format!("{}{id}", crate::bunks::MEMBER_PREFIX);
                if waiting.iter().any(|w| w.has(&id)) {
                    talk_marks.push((r, top.y));
                }
                marks.push(MarkDraw { id: key.clone(), r, top: top.y, kind: "member".into(), state: String::new(), npc: Some(NpcDraw { body, sprite, palette, initial: name.chars().next().unwrap_or('?').to_string(), face: Facing::default(), offset: idle_offset(&id) }), badge: None, glyph: None });
                let name_rect = Rect::new(r.x, top.y, r.w, r.h);
                let named = !names_always && !modal && !tilted && ui.hover(r);
                if named || names_always {
                    labels.push((name_rect, name.clone(), false));
                }
                let word = if ui.hover(r) { emberwake_world::bunks::condition(&d, &gs, m) } else { "" };
                hits.push(MarkHit { id: key, label: name.clone(), hit: r, foot: r.y + r.h, tip: tip(&name, word), named: named || names_always, name_rect, badge: false });
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
                        if !is_wall(x, y) || border_at(x, y) {
                            continue;
                        }
                        let cell = geo.cell(x, y);
                        let depth = wall_depth_at(x, y, wall_face_tiles, (mw_i, mh_i), &is_wall);
                        let face = if wall_face_tiles > 0 { depth > 0 } else { !is_wall(x, y + 1) };
                        let rect = mapview::block_rect(cell, face);
                        let seen = Rect::new(rect.x, rect.y, rect.w, rect.h + if wall_face_tiles == 0 && face { mapview::shade_of(cell.h) } else { 0 }).intersect(paint_area);
                        if seen.w > 0 && seen.h > 0 {
                            blocks.push(Block { x, y, cell, rect, face });
                        }
                    }
                }
            }
            // STYLE-PASS1B (`PR7-48`, FB-366): the click-to-walk's destination, a small amber ring on the floor under
            // whatever stands there (static: no Reduce motion variant)
            if let Some(dest) = self.walk_ring() {
                draw_walk_ring(&mut ui.p, geo.cell(dest.0 as i32, dest.1 as i32));
            }
            // the sorted pass (PROPS1 4.4): standing props, markers and the party back to front by their feet (the
            // sprites stand on their tiles, so lower rows cover upper ones, spec PLAYUI1 2.4)
            // REGIONVIEW1: the party's cell where it is drawn, and the view that puts it on its own physical px
            let (pr, party_shift) = mapview::party_px(view, q, camera, drawn, scale);
            let party_view = PView { ox: base_view.ox + party_shift.0, oy: base_view.oy + party_shift.1, ..base_view };
            let shot = CamShot { px: camera, party: (party_view.x(pr.x as f32), party_view.y(pr.y as f32)), scale };
            self.cam = shot;
            self.motion = json!({"cell": [gs.position.0, gs.position.1], "drawn": [drawn.0, drawn.1], "cam": [camera.0, camera.1], "target": [target.0, target.1], "tile": ts as f32 * scale});
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
            let drawn_party = if leader_sprite.is_some() { body_rect_of(cx, leader_sprite.as_deref(), pr) } else { pr };
            // PAINTED-MAPS1: the party sorts by its tile's bottom edge (a painted body's frame drops under it)
            let sort_party = Rect::new(drawn_party.x, pr.y, drawn_party.w, pr.h);
            let order = crate::props::sorted(&props, &blocks, &marks.iter().map(|m| m.r).collect::<Vec<_>>(), sort_party);
            let idle = ClipPlayer::default();
            let mut sprites = Vec::new();
            let mut party_frame = None;
            // HUB-BUNKS2, BUNKS-LAYOUT3: the hub bunks' drawn rects (a plate keeps to the room its neighbours leave) and
            // the dressed north wall with the niches (drawn once, before the first bunk)
            let fit_rects: Vec<Rect> = props
                .iter()
                .filter_map(|p| hub_bunks.iter().find(|b| b.prop == p.id).map(|b| crate::bunks::drawn_rect(&crate::bunks::bed_art(&b.art), p.rect)))
                .collect();
            let mut wall_drawn = false;
            for item in &order {
                match *item {
                    Item::Prop(i) => {
                        let p = &props[i];
                        let (fit, mounted) = prop_mounted_fit(prop_art_of(&prop_arts, &geo, p), p, &geo, &is_wall, wall_face_tiles);
                        let bunk = hub_bunks.iter().find(|b| b.prop == p.id);
                        if bunk.is_some() && !wall_drawn {
                            wall_drawn = true;
                            let (cw, ch) = geo.cell_size();
                            let runs: Vec<(i32, i32, i32)> = crate::bunks::wall_runs(t, &hub_bunks).iter().map(|&(x0, x1, y)| (geo.ox + x0 as i32 * cw, geo.ox + x1 as i32 * cw, geo.oy + (y as i32 + 1) * ch)).collect();
                            let niches: Vec<(String, crate::bunks::Pen, String)> = hub_bunks
                                .iter()
                                .filter_map(|b| {
                                    let bed = crate::bunks::bed_art(&b.art);
                                    let pd = props.iter().find(|p| p.id == b.prop).filter(|_| bed.niche)?;
                                    let founder = gs.party.member(&b.member).map(|m| json::s(d.resolve(&gs.world_id, "characters", &m.lore_ref), "id")).unwrap_or_default();
                                    Some((b.prop.clone(), crate::bunks::Pen::of(&ui.p, &bed, pd.rect, pd.flip), founder))
                                })
                                .collect();
                            let k = ui.p.v.s * cw as f32 / crate::props::ART_TILE as f32;
                            let pens: Vec<(crate::bunks::Pen, String)> = niches.iter().map(|(_, pen, f)| (*pen, f.clone())).collect();
                            let (walls, emblems) = crate::bunks::draw_wall(&mut ui, cx, &lineage, &runs, &pens, k);
                            self.bunk_walls = walls;
                            // BUNKS-EMBLEM1: each niche bunk's emblem ("" a plain arch)
                            for ((prop, _, _), emblem) in niches.iter().zip(emblems) {
                                if let Some(e) = self.bunks_drawn.iter_mut().find(|e| e["prop"] == prop.as_str()) {
                                    e["emblem"] = json!(emblem);
                                }
                            }
                        }
                        let Some(b) = bunk else {
                            draw_prop(&mut ui, cx, p, fit, mounted, map_rect, q);
                            continue;
                        };
                        // BUNKS-LAYOUT3: the bed at the take 8 mock's place, its dressing and its brass plate (with the
                        // bunk, in depth)
                        let bed = crate::bunks::bed_art(&b.art);
                        let pen = crate::bunks::Pen::of(&ui.p, &bed, p.rect, p.flip);
                        let r = crate::bunks::drawn_rect(&bed, p.rect);
                        let kept = ui.p.c.clip;
                        let inside = ui.p.v.rect(map_rect);
                        ui.p.c.clip = Some(kept.map_or(inside, |c| c.intersect(inside)));
                        crate::bunks::draw_bed(&mut ui, cx, &pen, p.file.as_deref());
                        let dressed = crate::bunks::dress(&mut ui, cx, &lineage, b, &bed, &pen, p.file.as_deref());
                        if !bed.niche {
                            crate::bunks::draw_post(&mut ui, &pen, &bed);
                        }
                        // FOOTLOCKER1: the footlocker at the foot of an active or benched member's bunk, open while
                        // their loadout is
                        if emberwake_world::bunks::has_locker(b) {
                            let open = self.locker_open == b.member;
                            if let Some((file, lr)) = crate::bunks::draw_locker(&mut ui, cx, &lineage, &pen, &bed, open) {
                                let at = geo.cell_at(lr.x + lr.w / 2, lr.y + lr.h * 3 / 4);
                                self.lockers.push((b.member.clone(), at, lr, file));
                            }
                        }
                        ui.p.c.clip = kept;
                        let room = crate::bunks::plate_room(r, &fit_rects);
                        let entry = json!({"blanket": dressed.blanket, "gear": dressed.gear, "board": dressed.board, "asleep": dressed.asleep, "pose": dressed.pose, "head": dressed.head.map(|h| json!([h.x, h.y, h.w, h.h])), "fold": dressed.fold.map(|(a, b)| json!([a, b]))});
                        if let Some(m) = gs.party.member(&b.member) {
                            let (name, class, class_id) = (emberwake_world::progression::display_name(&d, m), crate::hub::class_name(cx, m), m.class_id(&d));
                            let plate = crate::bunks::plate_rect(&mut ui.p, r, &bed, &name, room, p.flip);
                            let full = crate::bunks::plate_need(&mut ui.p, &name) <= plate.w;
                            let shown = crate::bunks::plate_shown(&mut ui.p, &name, plate.w);
                            let glyph = crate::bunks::plate_glyph(&ui.p);
                            crate::bunks::draw_plate(&mut ui, cx, plate, &name, &class_id, &class);
                            if let Some(e) = self.bunks_drawn.iter_mut().find(|e| e["prop"] == p.id.as_str()) {
                                e["plate"] = json!({"name": name, "shown": shown, "class": class, "class_id": class_id, "rect": [plate.x, plate.y, plate.w, plate.h], "full": full, "wall": bed.niche, "room": room, "glyph": glyph});
                                e["dressed"] = entry;
                            }
                        } else {
                            // FB-360: every empty bunk, on the north wall and free-standing, wears the blank plate
                            let plate = crate::bunks::plate_rect(&mut ui.p, r, &bed, "", room, p.flip);
                            crate::bunks::draw_blank_plate(&mut ui, plate);
                            if let Some(e) = self.bunks_drawn.iter_mut().find(|e| e["prop"] == p.id.as_str()) {
                                e["blank_plate"] = json!([plate.x, plate.y, plate.w, plate.h]);
                            }
                        }
                    }
                    Item::Block(i) => {
                        let b = &blocks[i];
                        if wall_face_tiles > 0 {
                            draw_block_depth(&mut ui, cx, b, wall_art_at(b.x, b.y), wall_face_tiles, (mw_i, mh_i), &is_wall, &border_at);
                        } else {
                            draw_block(&mut ui, cx, b, wall_art_at(b.x, b.y), &ch_at, &border_at);
                        }
                    }
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
            // REVEAL1 1.3: the roofs over everything drawn so far, at their zone's opacity: gone while the party stands in
            // their indoor zone (a doorway cell tagged with it counts)
            // (every roof of the map, on screen or not, so one coming into view shows its zone's opacity)
            let roof_zones: Vec<String> = json::aget(t, "props").iter().filter_map(|p| p.as_object().and_then(explore::roof_zone)).collect();
            let inside = explore::indoor_zone_at(t, gs.position.0, gs.position.1);
            self.roofs.update(&gs.location_id, &roof_zones, &inside, self.frame_at, cx.settings.reduce_motion());
            let mut under_roof = false;
            let mut opaque_roofs: Vec<Rect> = Vec::new();
            for p in &props {
                let Some(zone) = &p.roof else { continue };
                let alpha = self.roofs.alpha(zone);
                draw_roof(&mut ui, cx, p, map_rect, q, (alpha * 255.0).round() as u8);
                let meets = p.rect.intersect(drawn_party);
                under_roof |= alpha >= crate::props::ROOF_HIDES_FROM && meets.w > 0 && meets.h > 0;
                if alpha > crate::props::ROOF_HIDES_FROM {
                    opaque_roofs.push(p.rect);
                }
            }
            if dim_ambience {
                apply_dim_ambience(&mut ui, &geo, &props, &prop_arts, &opaque_roofs, &is_wall, wall_face_tiles);
            }
            // 4.5: a standing prop or a wall block drawn over the party: its outline over them (the sprite's silhouette
            // outline); REVEAL1: a roof showing over them too
            if under_roof || crate::props::party_hidden(&props, &blocks, &order, drawn_party) {
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
                if opaque_roofs.iter().any(|rr| {
                    let hit = m.r.intersect(*rr);
                    hit.w > 0 && hit.h > 0
                }) {
                    continue;
                }
                let (side, over) = (9 * q / 4, 5 * q / 4);
                let br = Rect::new(m.r.x + ts - over, m.top - over, side, side);
                let px = ui.icon_px(Rect::new(0, 0, br.w + 4, br.h + 4));
                match cx.art.image(&service_icon(first), px, px) {
                    Some(img) => ui.p.image(img, br),
                    None => service_glyph(&mut ui, first, br),
                }
            }
            // BUNKS1 BK8, BK11: a candle on each bunk whose member has a waiting talk, a speech mark over each member
            // (the lead too) it waits on
            let side = 12 * q / 4;
            for r in &candles {
                crate::bunks::draw_mark(&mut ui, cx, crate::bunks::CANDLE_ICON, Rect::new(r.x + r.w - side, r.y, side, side));
            }
            if self.hub.on() && waiting.iter().any(|w| w.has(&lead_now)) {
                talk_marks.push((pr, drawn_party.y));
            }
            for (r, top) in &talk_marks {
                crate::bunks::draw_mark(&mut ui, cx, crate::bunks::TALK_ICON, Rect::new(r.x + r.w / 2 - side / 2, top - side, side, side));
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
                // FOOTLOCKER1: the footlockers over the people idling by them (a small area of its own)
                for (member, _, lr, _) in &self.lockers {
                    let hit = lr.intersect(map_area);
                    let name = gs.party.member(member).map(|m| crate::hub::member_name(cx, m)).unwrap_or_default();
                    let t = if ui.hover(hit) { tip(&format!("{name}'s footlocker"), "Open: their loadout and the shared stash.") } else { Vec::new() };
                    ui.push(format!("{}{member}", crate::bunks::LOCKER_PREFIX), "locker", format!("{name}'s footlocker"), hit, true, false, Some(RAct::Locker(member.clone())), t);
                }
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
        let help = region_help(&crate::controls_view::hints(&cx.settings.v));
        hud_bar(&mut ui, cx, 4, &name, ("region.menu", Some(Key::Esc)), why, &help, RAct::Hud);
        // -- The legend (2.1 A) at the top right and the event log ticker (MAPLOG1) at the bottom left; off in a talk
        let mut panes = Vec::new();
        if !t.is_empty() && !talking {
            panes.push(self.draw_legend(&mut ui, cx, modal));
        }
        if !talking {
            panes.extend(self.draw_ticker(&mut ui, cx, modal));
        }
        // TRANSITIONS1 T2 step 4: the arrival card under the HUD bar
        if let Some((a, at)) = self.arrival.as_ref() {
            let ms = Instant::now().saturating_duration_since(*at).as_millis() as u64;
            crate::transitions::draw_arrival(&mut ui.p, &a.card, crate::transitions::arrival_alpha(ms, cx.settings.reduce_motion()));
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
        self.follow_arrival(cx, Instant::now());
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
            if crate::DEV_KEYS {
                flip_view(cx);
            }
            return;
        }
        // FOOTLOCKER1B (2): a footlocker opening holds the map still
        if self.locker_opening() {
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
                // the frame's clock: the press moves from this frame on, and the steps chain from it (ENGINE-50: the
                // next tick dates it now when that frame was long ago)
                let at = self.frame_at.unwrap_or_else(Instant::now);
                self.plan_step(cx, at, dir);
                self.press_at = Some(at);
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
        if self.locker_opening() {
            return;
        }
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
            "walking": !self.walk.is_empty(),
            // BUNKS-FIX1 (`PR7-46` (3)): the marker the walk acts on when it arrives (a talk, a point; "" none)
            "walk_to": self.walk_target.clone().unwrap_or_default(),
            // STYLE-PASS1B (`PR7-48`): the tile the walk's amber ring marks (null: none)
            "walk_ring": self.walk_ring().map(|p| json!([p.0, p.1])),
            "log": cx.log.iter().rev().take(8).rev().collect::<Vec<_>>(),
            "dialogue": svc["dialogue"], "turn_in": svc["turn_in"],
            "lore": self.lore.as_ref().map(|l| l.0.clone()), "legend_open": cx.settings.v["map_legend_open"].as_bool().unwrap_or(true), "log_open": self.log_open,
            "arrival": self.arrival.as_ref().map(|(a, _)| json!({"card": a.card, "changes": a.changes})),
            "shop": svc["shop"], "board": svc["board"], "board_pane": svc["board_pane"], "sprites": self.sprites,
            // MAPNAV1-U: who leads, the edge arrows of the last frame and the tile the camera looks at (null: the party)
            // REGIONVIEW1: the camera (physical px of cell (0, 0)), the party's physical px, and the last frames of both
            "camera": {"px": [self.cam.px.0, self.cam.px.1], "party": [self.cam.party.0, self.cam.party.1], "scale": self.cam.scale, "zoom": self.geo.q as f64 / 4.0,
                "trail": (!self.cam_trail.is_empty()).then(|| self.cam_trail.iter().map(|(t, c)| json!([(t * 100000.0).round() / 100.0, c.px.0, c.px.1, c.party.0, c.party.1])).collect::<Vec<_>>())},
            "view": mapview::View::of_setting(&cx.settings.v["region_view"]).name(), "ground": self.ground.state(), "leader": leader_id(cx), "arrows": self.arrows, "pan": self.pan.filter(|_| self.pan_loc == gs.location_id).map(|p| json!([p.0, p.1])),
            // PORTRAITFX1 4.11: the speaker's portrait while the talk box shows (the shop or the board hides it)
            "portraits": svc["portraits"],
            // ENGINE-25: the talk box's view-model (`talk_view`)
            "talk_view": svc["talk_view"],
            // REVEAL1 1.3: each roofed indoor zone's roof opacity (0 gone, 1 opaque)
            "roofs": self.roofs.state(),
            // BUNKS1: in a hub, where each member idles and each bunk's member, state and candle (null elsewhere)
            "hub": self.hub.state(), "bunks": (!self.bunks_drawn.is_empty()).then(|| self.bunks_drawn.clone()),
            "bunk_held": (!self.bunk_held.is_empty()).then(|| self.bunk_held.clone()),
            // FOOTLOCKER1: each footlocker (member, tile, rect, open, file) and whose is open ("" none)
            "lockers": (!self.lockers.is_empty()).then(|| self.lockers.iter().map(|(m, at, r, f)| json!({"member": m, "at": [at.0, at.1], "rect": [r.x, r.y, r.w, r.h], "open": *m == self.locker_open, "file": f})).collect::<Vec<_>>()),
            "locker_open": self.locker_open,
            "bunk_walls": (!self.bunk_walls.is_empty()).then(|| self.bunk_walls.iter().map(|r| json!([r.x, r.y, r.w, r.h])).collect::<Vec<_>>())})
    }

    /// MUSIC2: shop while the shop is open, else the location's kind.
    fn music(&self, cx: &Cx) -> Option<String> {
        let shop = !self.svc.shop_qid().is_empty();
        cx.game.as_ref().and_then(|g| music::game_track(cx, music::region_context(&cx.data, &g.gs, shop)))
    }

    /// AMBIENCE1: the place's ambient loop, by the music's context (a shop keeps the town's, `shop` falls back to it).
    fn ambience(&self, cx: &Cx) -> Option<String> {
        let shop = !self.svc.shop_qid().is_empty();
        cx.game.as_ref().and_then(|g| music::game_ambience(cx, music::region_context(&cx.data, &g.gs, shop)))
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

    /// The legend (spec PLAYUI1 2.1 A; FLOATPANELS1): a floating panel (`aegis_ui::float_panel`) that starts at the
    /// map's top right under the HUD bar, locked; unlocked it drags anywhere on the map, and its minimize button shrinks
    /// it in place to a "Here (N)" button (kept in the settings as `map_legend_open`); a row per marker shows the marker's map icon, its name and (people) their service icons at the right; spent
    /// markers dim. A row click looks there (MAPNAV1-U: the camera pans to the marker); a second click while it looks
    /// there walks there as before. Width from the texts, at most `LEGEND_MAX` (times the text size). Its rect (the edge
    /// arrows keep off it).
    fn draw_legend(&mut self, ui: &mut Ui<RAct>, cx: &mut Cx, modal: bool) -> Rect {
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        let loc = explore::location(&d, &gs);
        // (id, kind, state, label, verb, services, a point's use)
        let mut rows: Vec<(String, String, String, String, String, Vec<&'static str>, String)> = Vec::new();
        for m in explore::markers_of(loc) {
            let (view, services) = self.marker_memo.get(&d, &gs, loc, &m);
            let (kind, state) = (s(&m, "kind"), s(&view, "state"));
            // PR7-76 (PT-43): a point of interest the party can use now is listed too
            if state == "absent" || (kind == "point" && !point_listed(&m, &state)) {
                continue;
            }
            rows.push((s(&m, "id"), kind, state, s(&view, "label"), s(&view, "verb"), services, explore::point_use(&m)));
        }
        let open = cx.settings.v["map_legend_open"].as_bool().unwrap_or(true);
        let rh = (ui.p.line_h(BODY).ceil() as i32 + 4).max(14);
        let title = format!("Here ({})", rows.len());
        // FLOATPANELS1: the head row holds the title and, at its right, the lock and minimize buttons
        let buttons = 2 * PANEL_BTN + 8;
        let mut w = 4 + ui.p.text_w(&title, BODY).ceil() as i32 + 6 + buttons;
        for (_, _, _, label, _, services, _) in &rows {
            w = w.max(4 + ICON + 4 + ui.p.text_w(label, BODY).ceil() as i32 + 6 + services.len() as i32 * (SERVICE + 1) + 4);
        }
        let w = w.min((LEGEND_MAX as f32 * ui.p.fonts.scale).round() as i32 + buttons);
        let top_y = HUD_H + 4;
        let content = rows.len() as i32 * rh;
        let size = (w, rh + 1 + content.min(H - 4 - top_y - rh - 1));
        let chip = aegis_ui::float_panel::chip_size(ui, &title);
        self.legend_panel.minimized = !open;
        let held = ui.dragging() == Some(LEGEND_PANEL_DRAG);
        crate::floats::frame(cx, &mut self.legend_panel, &mut self.legend_kept, crate::floats::REGION_LEGEND_KEY, MAP_AREA, (W - 4 - w, top_y), size, chip, self.mouse, held);
        self.legend_size = size;
        let acts = aegis_ui::float_panel::Acts { body: RAct::Panel, lock: RAct::LegendLock, min: RAct::Legend, open: RAct::Legend };
        // WEBFIX2 (a): the panel in the HUD frame
        let hud = |ui: &mut Ui<RAct>, p: Rect| crate::hud::hud_panel(ui, p, rgba(0, 0, 0, 0), th().edge, 3.0);
        let Some(p) = aegis_ui::float_panel::draw(ui, &self.legend_panel, "legend", &title, LEGEND_PANEL_DRAG, size, chip, &acts, th().game("map_pane", PANE), hud) else {
            return self.legend_panel.chip_rect(size, chip);
        };
        ui.p.text_fit(p.x as f32 + 4.0, p.y as f32 + (rh as f32 - ui.p.line_h(BODY)) / 2.0, &title, BODY, (p.w - 8 - buttons) as f32, th().gold);
        if rows.is_empty() {
            return p;
        }
        let (x, w) = (p.x, p.w);
        let top = p.y + rh + 1;
        let pane = Rect::new(x, top, w, p.y + p.h - top);
        ui.p.fill(Rect::new(x + 2, top - 1, w - 4, 1), th().edge);
        let off = ui.scroll_begin("legend.rows", pane, content);
        let px = ui.icon_px(Rect::new(0, 0, ICON + 4, ICON + 4));
        let spx = ui.icon_px(Rect::new(0, 0, SERVICE + 4, SERVICE + 4));
        for (i, (id, kind, state, label, verb, services, point_use)) in rows.into_iter().enumerate() {
            let r = Rect::new(x, top - off + i as i32 * rh, w, rh);
            let hover = !modal && ui.hover(r);
            if hover {
                ui.p.panel(r, th().card_hot, th().edge_hot, 3.0);
            }
            let ir = Rect::new(r.x + 4, r.y + (rh - ICON) / 2, ICON, ICON);
            // PR7-36 (UX-28): an exit, chest, lore or (PT-43) point row shows the map's glyph
            match (glyph_of(&kind, &point_use), cx.art.image(&emberwake_content::art::marker(marker_icon(&kind, &state)), px, px)) {
                (Some(g), _) => draw_map_glyph(&mut ui.p, ir, g, glyph_colour(&kind, &state)),
                (None, Some(img)) => ui.p.image(img, ir),
                (None, None) => ui.p.fill(Rect::new(ir.x + 2, ir.y + 2, ICON - 4, ICON - 4), Self::marker_style(&kind, &state).0),
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
        p
    }

    /// The event log ticker (MAPLOG1-U, spec PLAYUI1 2.1 A): the latest event on one line at the map's bottom left; L
    /// or a click opens the last `LOG_OPEN` events (newest at the bottom), and again shows only the latest. Its rect
    /// (None with an empty log).
    fn draw_ticker(&mut self, ui: &mut Ui<RAct>, cx: &mut Cx, modal: bool) -> Option<Rect> {
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
        // FLOATPANELS1: the ticker floats (a free corner: the bottom left, locked); its lock and minimize buttons sit
        // right of the text, and its bottom edge stays put while it opens and closes
        let buttons = 2 * PANEL_BTN + 8;
        let size = (w + buttons, h.max(PANEL_BTN + 4));
        if let Some((x, y)) = self.log_panel.pos.filter(|_| self.log_size.1 > 0 && self.log_size.1 != size.1) {
            self.log_panel.pos = Some((x, y + self.log_size.1 - size.1));
        }
        let chip = aegis_ui::float_panel::chip_size(ui, "Log");
        let held = ui.dragging() == Some(LOG_PANEL_DRAG);
        crate::floats::frame(cx, &mut self.log_panel, &mut self.log_kept, crate::floats::REGION_LOG_KEY, MAP_AREA, (4, H - 4 - size.1), size, chip, self.mouse, held);
        self.log_size = size;
        let acts = aegis_ui::float_panel::Acts { body: RAct::Panel, lock: RAct::LogLock, min: RAct::LogMin, open: RAct::LogOpen };
        let hud = |ui: &mut Ui<RAct>, p: Rect| crate::hud::hud_panel(ui, p, rgba(0, 0, 0, 0), th().edge, 3.0);
        let Some(p) = aegis_ui::float_panel::draw(ui, &self.log_panel, "log.panel", "Log", LOG_PANEL_DRAG, size, chip, &acts, th().game("map_pane", PANE), hud) else {
            return Some(self.log_panel.chip_rect(size, chip));
        };
        let r = Rect::new(p.x, p.y + p.h - h, w, h);
        let hover = !modal && ui.hover(r);
        if hover {
            ui.p.panel(r, rgba(0, 0, 0, 0), th().gold, 3.0);
        }
        let last_y = r.y + 3 + (lines.len() as i32 - 1) * lh;
        draw_glyph(&mut ui.p, if self.log_open { Glyph::ChevronDown } else { Glyph::ChevronRight }, Rect::new(r.x + 4, last_y + (lh - 8) / 2, 8, 8), if hover { th().gold } else { th().dim });
        for (i, l) in lines.iter().enumerate() {
            let ly = (r.y + 3 + i as i32 * lh) as f32;
            ui.p.text_fit((r.x + 15) as f32, ly, l, SMALL, inner as f32, if i + 1 == lines.len() { th().text } else { th().dim });
        }
        let l = crate::controls_view::hints(&cx.settings.v).key_hint(Key::Char('l'));
        let mut t = tip(&format!("Event log ({l})"), &if self.log_open { format!("Click or press {l} to show only the latest event.") } else { format!("Click or press {l} for the last 12 events.") });
        if !self.log_open && ui.p.text_w(&latest, SMALL) > inner as f32 {
            t.push((latest.clone(), th().text));
        }
        ui.push("log", "button", latest, r, !modal, self.log_open, Some(RAct::Log), t);
        Some(p)
    }
}

/// A marker for the sorted pass (PROPS1 4.4): its id, tile, the top of what is drawn there (a sprite's head, else the
/// tile; badges and names sit over it), kind and state; a person's art; the first service's badge (drawn with the
/// overlays).
/// ENGINE-50 (MAP-FRAME1): the key of the map's still layers (`RegionMap::floor`): the data and art loaded, the theme,
/// the location, view and zoom, the tile sets, whether a composed ground is under the tiles, and the props shown (a
/// hub's bunk states among them). The view and canvas are `Painter::scroll_begin`'s own part of the key.
fn floor_key(cx: &Cx, location: &str, (view, q): (mapview::View, i32), sets: &[String], has_ground: bool, props_t: &Dict) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    (Arc::as_ptr(&cx.data) as usize, cx.art.generation(), std::rc::Rc::as_ptr(&th()) as usize).hash(&mut h);
    (location, view.name(), q, sets, has_ground).hash(&mut h);
    json::get(props_t, "props").to_string().hash(&mut h);
    h.finish()
}

/// ENGINE-50 (MAP-FRAME1): each marker's view (`explore::marker_view`) and a person's services (`explore::npc_services`,
/// none for another kind) for one game state and data, which are all they read. The map and the legend asked for each
/// marker every frame, most of a walking frame; now once per state (a step changes it).
#[derive(Default)]
struct MarkerMemo {
    data: Option<Arc<emberwake_world::Data>>,
    gs: Option<GameState>,
    rows: HashMap<String, (Dict, Vec<&'static str>)>,
}

impl MarkerMemo {
    /// Marker `m`'s (full, `explore::marker_full`) view and services at `loc` in `gs`.
    fn get(&mut self, d: &Arc<emberwake_world::Data>, gs: &GameState, loc: &Dict, m: &Dict) -> (Dict, Vec<&'static str>) {
        if !self.data.as_ref().is_some_and(|k| Arc::ptr_eq(k, d)) || self.gs.as_ref() != Some(gs) {
            (self.data, self.gs) = (Some(d.clone()), Some(gs.clone()));
            self.rows.clear();
        }
        let read = || {
            let services = if s(m, "kind") == "npc" { explore::npc_services(d, gs, &s(d.resolve(&gs.world_id, "characters", &s(m, "character")), "_qid")) } else { Vec::new() };
            (explore::marker_view(d, gs, loc, m), services)
        };
        let id = s(m, "id");
        if id.is_empty() {
            return read();
        }
        self.rows.entry(id).or_insert_with(read).clone()
    }
}

struct MarkDraw {
    id: String,
    r: Rect,
    top: i32,
    kind: String,
    state: String,
    npc: Option<NpcDraw>,
    badge: Option<&'static str>,
    /// PR7-36 (UX-28): the drawn glyph of an exit or point of interest (`glyph_of`).
    glyph: Option<MapGlyph>,
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
        // BUNKS1 BK4: an idling member opens its card
        if let Some(id) = h.id.strip_prefix(crate::bunks::MEMBER_PREFIX) {
            let act = RAct::Member(id.to_string());
            ui.push(h.id.clone(), "member", h.label, h.hit, true, false, Some(act), h.tip);
            continue;
        }
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

/// ENGINE-53 (SWITCH-AHEAD1), ENGINE-54 (WARM-SPREAD1): the region map a save opens on, drawn unseen in the shell's
/// warm slices while the title shows (`Screen::warm_next`), so its first draw's art (bodies, markers, props, roofs) and
/// ground are in their caches before Continue. It draws over a game of its own, swapped in for each slice only (the
/// title has none), and gives back the log; the shell drops what it asked for (navigation, notices). A save opening on a
/// story stage is not rehearsed. A fight started ahead is warmed the same way (`fight`).
pub struct Rehearsal {
    game: Option<crate::screens::Game>,
    /// A fight (its event or row, and runner) to start on that game and draw instead of the map (a fight started ahead).
    fight: Option<(Dict, String)>,
    /// ENGINE-54 (WARM-SPREAD1): the screen drawn, made on the first slice and kept for the next ones (None after a
    /// start that failed).
    inner: Option<Box<dyn Screen<GameCx>>>,
    made: bool,
    /// ENGINE-54: set once a slice drew it with nothing left for later (the warming is done).
    done: std::rc::Rc<std::cell::Cell<bool>>,
}

impl Rehearsal {
    /// The map `gs` opens on (as `enter_game` would open it), or None for a game that opens on its story stage.
    pub fn of(cx: &Cx, gs: &GameState, slot: i64) -> Option<Self> {
        if emberwake_world::story::is_running(gs) || gs.location_id.is_empty() {
            return None;
        }
        let mut gs = gs.clone();
        if emberwake_world::travel::roads_seen(&gs).is_none() {
            emberwake_world::travel::note_new_roads(&cx.data.clone(), &mut gs);
        }
        Some(Self { game: Some(crate::screens::Game::new(gs, slot, false)), fight: None, inner: None, made: false, done: Default::default() })
    }

    /// ENGINE-53, ENGINE-54: `fight` by `runner` on a copy of `gs`, started already (`ahead`, on a thread: its slices
    /// never start a fight on the frame), drawn unseen (its bodies, board ground, panels).
    pub fn fight(gs: GameState, slot: i64, fight: Dict, runner: &str, ahead: crate::battle::FightAhead) -> Self {
        let mut game = crate::screens::Game::new(gs, slot, false);
        game.fight_ahead = Some(ahead);
        Self { game: Some(game), fight: Some((fight, runner.to_string())), inner: None, made: false, done: Default::default() }
    }

    /// ENGINE-54: set once the warming is done (a slice drew it with nothing left for later).
    pub fn done_flag(&self) -> std::rc::Rc<std::cell::Cell<bool>> {
        self.done.clone()
    }
}

impl Screen<GameCx> for Rehearsal {
    fn id(&self) -> &'static str {
        "region_rehearsal"
    }
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, _mouse: (i32, i32), _live: bool) {
        let Some(game) = self.game.take() else { return };
        let kept = cx.game.replace(game);
        let log = cx.log.clone();
        // the screen is made on one slice and drawn from the next (each its own frame)
        if !self.made {
            self.made = true;
            self.inner = match &self.fight {
                None => Some(Box::new(RegionMap::new(cx))),
                Some((fight, runner)) => crate::battle::BattleScreen::world(cx, fight, runner).ok().map(|b| Box::new(b) as Box<dyn Screen<GameCx>>),
            };
            cx.art.more();
            cx.log = log;
            self.game = std::mem::replace(&mut cx.game, kept);
            return;
        }
        let left = (cx.art.left(), aegis_ui::ground::left_later());
        if let Some(inner) = self.inner.as_mut() {
            inner.draw(cx, g, (-1, -1), false);
        }
        if (cx.art.left(), aegis_ui::ground::left_later()) == left {
            self.done.set(true);
        }
        cx.log = log;
        self.game = std::mem::replace(&mut cx.game, kept);
    }
    fn widgets(&self) -> Vec<Value> {
        Vec::new()
    }
    fn draw_tip(&self, _p: &mut aegis_ui::painter::Painter, _mouse: (i32, i32)) {}
    fn click(&mut self, _cx: &mut Cx, _pos: (i32, i32)) -> bool {
        false
    }
    fn click_id(&mut self, _cx: &mut Cx, id: &str) -> Result<(), String> {
        Err(format!("unknown widget '{id}'"))
    }
    fn widget_rect(&self, _id: &str) -> Option<(Rect, Vec<String>)> {
        None
    }
}

/// Body `body` standing on tile `r` (spec PLAYUI1 2.4): a foot shadow in `foot`, then the frame `player` plays at
/// `now` in `face` (`Art::played_facing`, DIR4-M: the up or down row when the body has one, else the side row,
/// mirrored facing left), filling its rect (`body_rect_of`: PAINTED-MAPS1 scales a painted body by its foot row) at the
/// zoomed size (SPRITEFIT1's exact fit). None when it does not play (the caller draws today's token).
#[allow(clippy::too_many_arguments)]
fn draw_sprite(ui: &mut Ui<RAct>, cx: &mut Cx, body: &str, palette: &str, player: &ClipPlayer, now: u64, face: Facing, r: Rect, foot: Rgba) -> Option<SpriteDrawn> {
    let sr = body_rect_of(cx, Some(body), r);
    let px = (sr.w as f32 * ui.p.v.s).round().max(1.0) as i32;
    let (f, img) = cx.art.played_facing(body, palette, player, now, px, face)?;
    let flip = face.mirrors(&f);
    let feet = (r.y + r.h) as f32 - r.w as f32 / 8.0;
    ui.p.ellipse(r.x as f32 + r.w as f32 / 2.0, feet, r.w as f32 * 0.4, r.h as f32 * 0.14, rgba(foot[0], foot[1], foot[2], FOOT_ALPHA));
    ui.p.image(img, sr);
    let view = if f.side { View::Side } else { face.view };
    Some(SpriteDrawn { body: body.to_string(), clip: f.clip, frame: f.frame, flip, rect: sr, px, face, view })
}

/// PAINTED-MAPS1 (PR7-73): the drawn square of body `body` (None: no body) on tile `r` (`mapview::body_rect` over its
/// sheet: a painted body by its foot row, any other the token square).
fn body_rect_of(cx: &mut Cx, body: Option<&str>, r: Rect) -> Rect {
    mapview::body_rect(r, body.and_then(|b| cx.art.clips(b)))
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
    if let Some(g) = m.glyph {
        draw_map_glyph(&mut ui.p, r, g, glyph_colour(&m.kind, &m.state));
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

fn map_wall_face_tiles(loc: &Dict, t: &Dict) -> i32 {
    let v = if loc.contains_key("wall_face_tiles") {
        json::i(loc, "wall_face_tiles", 0)
    } else {
        json::i(t, "wall_face_tiles", 0)
    };
    v.clamp(0, 2) as i32
}

fn map_dim_ambience(loc: &Dict, t: &Dict) -> bool {
    if loc.contains_key("dim_ambience") {
        json::b(loc, "dim_ambience", false)
    } else {
        json::b(t, "dim_ambience", false)
    }
}

fn prop_fit(art: &str) -> Option<(i32, i32)> {
    Some(match art {
        "portcullis" => (72, 84),
        "kiln_bunk" => (64, 56),
        "iron_ladder" => (40, 56),
        "crypt_bars" => (48, 52),
        _ => return None,
    })
}

fn light_pool(art: &str) -> Option<(i32, [u8; 3])> {
    Some(match art {
        "brazier" => (150, [255, 170, 80]),
        "tinker_hearth" => (140, [255, 140, 60]),
        "furnace_mouth" => (170, [255, 120, 40]),
        "lantern_pole" => (95, [255, 190, 110]),
        "watch_torch" => (95, [255, 130, 60]),
        "violet_lamp_post" => (100, [150, 130, 255]),
        "apothecary_slab" => (85, [255, 200, 130]),
        "retort_shelf" => (60, [255, 200, 130]),
        "kiln_bunk" => (45, [255, 190, 120]),
        "hearth_stove" => (70, [255, 150, 80]),
        "candle_stand" => (70, [255, 200, 120]),
        "shrine_altar" => (80, [255, 200, 120]),
        _ => return None,
    })
}

fn collect_prop_arts(t: &Dict) -> Vec<(String, i32, i32, String)> {
    json::aget(t, "props")
        .iter()
        .filter_map(|raw| raw.as_object())
        .map(|p| (s(p, "id"), json::i(p, "x", 0) as i32, json::i(p, "y", 0) as i32, s(p, "art")))
        .collect()
}

fn prop_art_of<'a>(arts: &'a [(String, i32, i32, String)], geo: &MapGeom, p: &PropDraw) -> Option<&'a str> {
    let (cw, ch) = geo.cell_size();
    let (px, py) = ((p.foot.x - geo.ox) / cw.max(1), (p.foot.y - geo.oy) / ch.max(1));
    arts.iter()
        .find(|(id, x, y, _)| *x == px && *y == py && (id == &p.id || p.id.is_empty()))
        .or_else(|| arts.iter().find(|(_, x, y, _)| *x == px && *y == py))
        .map(|(_, _, _, art)| art.as_str())
}

/// REGIONVIEW2: `#` cells under free-standing `sort` props (adjacent to >=1 non-`#`, off the outer map ring, and
/// excluding wall-mounted `prop_fit` props) render floor beneath them without altering `rows` collision.
fn freestanding_prop_cells(t: &Dict, (mw, mh): Pos, ch_at: &impl Fn(i32, i32) -> char) -> std::collections::HashSet<(i32, i32)> {
    let mut out = std::collections::HashSet::new();
    let (mw, mh) = (mw as i32, mh as i32);
    for raw in json::aget(t, "props") {
        let Some(p) = raw.as_object() else { continue };
        let layer = s(p, "layer");
        if layer == "ground" || layer == "roof" || explore::roof_zone(p).is_some() {
            continue;
        }
        let (px, py) = (json::i(p, "x", 0) as i32, json::i(p, "y", 0) as i32);
        let (pw, ph) = (json::i(p, "w", 1).max(1) as i32, json::i(p, "h", 1).max(1) as i32);
        let art = s(p, "art");
        let mut on_border = false;
        let mut has_open_neighbor = false;
        let mut has_wall = false;
        for dy in 0..ph {
            for dx in 0..pw {
                let (tx, ty) = (px + dx, py + dy);
                if tx <= 0 || ty <= 0 || tx >= mw - 1 || ty >= mh - 1 {
                    on_border = true;
                    break;
                }
                if ch_at(tx, ty) == '#' {
                    has_wall = true;
                }
                for (nx, ny) in [(tx - 1, ty), (tx + 1, ty), (tx, ty - 1), (tx, ty + 1)] {
                    if nx >= 0 && ny >= 0 && nx < mw && ny < mh && ch_at(nx, ny) != '#' {
                        has_open_neighbor = true;
                    }
                }
            }
            if on_border {
                break;
            }
        }
        if on_border || !has_wall || !has_open_neighbor || prop_fit(&art).is_some() {
            continue;
        }
        for dy in 0..ph {
            for dx in 0..pw {
                let (tx, ty) = (px + dx, py + dy);
                if ch_at(tx, ty) == '#' {
                    out.insert((tx, ty));
                }
            }
        }
    }
    out
}

fn prop_mounted_fit(art: Option<&str>, p: &PropDraw, geo: &MapGeom, is_wall: &impl Fn(i32, i32) -> bool, wall_face_tiles: i32) -> (Option<Rect>, bool) {
    if wall_face_tiles <= 0 || p.ground || p.roof.is_some() {
        return (None, false);
    }
    let Some(art) = art else { return (None, false) };
    let Some((fw, fh)) = prop_fit(art) else { return (None, false) };
    let (cw, ch) = geo.cell_size();
    let (tx0, ty0) = ((p.foot.x - geo.ox) / cw.max(1), (p.foot.y - geo.oy) / ch.max(1));
    let (tw, th) = ((p.foot.w / cw.max(1)).max(1), (p.foot.h / ch.max(1)).max(1));
    let on_wall = (0..th).any(|dy| (0..tw).any(|dx| is_wall(tx0 + dx, ty0 + dy)));
    if !on_wall {
        return (None, false);
    }
    let dw = (fw * cw + ART_TILE / 2) / ART_TILE;
    let dh = (fh * cw + ART_TILE / 2) / ART_TILE;
    (Some(Rect::new(p.foot.x + (p.foot.w - dw) / 2, p.foot.y + p.foot.h - dh, dw, dh)), true)
}

fn wall_depth_at(x: i32, y: i32, face_h: i32, (mw, mh): (i32, i32), is_wall: &impl Fn(i32, i32) -> bool) -> i32 {
    if face_h <= 0 || x < 0 || y < 0 || x >= mw || y >= mh || !is_wall(x, y) {
        return 0;
    }
    let dd = (1..=face_h).find(|&k| !is_wall(x, y + k)).unwrap_or(0);
    if dd == 2 && !is_wall(x, y - 1) {
        0
    } else {
        dd
    }
}

fn draw_wall_cap_edges(ui: &mut Ui<RAct>, cap: Rect, cell_h: i32, north_open: bool, west_open: bool, east_open: bool) {
    let line = mapview::line_of(cell_h);
    if north_open {
        ui.p.fill(Rect::new(cap.x, cap.y, cap.w, line), rgb(70, 70, 78));
    }
    if west_open {
        ui.p.fill(Rect::new(cap.x, cap.y, line, cap.h), rgb(70, 70, 78));
    }
    if east_open {
        ui.p.fill(Rect::new(cap.x + cap.w - line, cap.y, line, cap.h), rgb(14, 14, 18));
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_wall_face_at(
    ui: &mut Ui<RAct>,
    cx: &mut Cx,
    walls: &WallArt,
    border: bool,
    x: i32,
    y: i32,
    face: Rect,
    cell_h: i32,
    darken_lower: bool,
    top_cap_line: bool,
    east_seam: bool,
    west_seam: bool,
) {
    let wall = emberwake_content::art::region_tile('#').unwrap_or("wall");
    let dirs = walls.face_dirs(border);
    let dir_refs: Vec<&str> = dirs.iter().map(String::as_str).collect();
    let pw = (face.w as f32 * ui.p.v.s).round().max(1.0) as i32;
    let ph = (face.h as f32 * ui.p.v.s).round().max(1.0) as i32;
    let scale_rgb = |img: &aegis_ui::canvas::Image, factor: f32| {
        let mut out = img.clone();
        for (k, v) in out.px.iter_mut().enumerate() {
            if k % 4 != 3 {
                *v = (*v as f32 * factor).round().clamp(0.0, 255.0) as u8;
            }
        }
        out
    };
    if let Some(img) = cx.art.varied(&dir_refs, None, x, y, pw, ph) {
        if darken_lower {
            ui.p.image(&scale_rgb(img, 0.72), face);
        } else {
            ui.p.image(img, face);
        }
    } else {
        let wall_dirs: Vec<String> = walls.sets.iter().map(|set| format!("tilesets/{set}/{wall}")).collect();
        let wall_refs: Vec<&str> = wall_dirs.iter().map(String::as_str).collect();
        if let Some(img) = cx.art.varied(&wall_refs, None, x, y, pw, ph.max(cell_h)) {
            let cropped = img.crop(0, 0, img.w, ph.min(img.h));
            ui.p.image(&scale_rgb(&cropped, if darken_lower { 0.75 * 0.72 } else { 0.75 }), face);
        } else {
            ui.p.fill(face, if darken_lower { rgb(22, 32, 27) } else { rgb(30, 45, 38) });
        }
    }
    if top_cap_line {
        let cap_h = (cell_h / 8).max(2).min(face.h);
        ui.p.fill(Rect::new(face.x, face.y, face.w, cap_h), rgb(132, 130, 140));
    }
    let line = mapview::line_of(cell_h);
    if west_seam {
        ui.p.fill(Rect::new(face.x, face.y, line, face.h), rgb(70, 70, 78));
    }
    if east_seam {
        ui.p.fill(Rect::new(face.x + face.w - line, face.y, line, face.h), rgb(14, 14, 18));
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_block_depth(
    ui: &mut Ui<RAct>,
    cx: &mut Cx,
    b: &Block,
    walls: &WallArt,
    face_tiles: i32,
    size: (i32, i32),
    is_wall: &impl Fn(i32, i32) -> bool,
    border_at: &impl Fn(i32, i32) -> bool,
) {
    let depth = wall_depth_at(b.x, b.y, face_tiles, size, is_wall);
    let is_face = |nx: i32, ny: i32| wall_depth_at(nx, ny, face_tiles, size, is_wall) > 0;
    let sets = walls.sets.as_slice();
    let wall = emberwake_content::art::region_tile('#').unwrap_or("wall");
    let art_px = ART_TILE * b.cell.w / mapview::CELL_W;
    if depth == 0 {
        let cap = mapview::cap_rect(b.cell);
        if border_at(b.x, b.y + 1) {
            match cx.art.tile_in(sets, wall, b.x, b.y, art_px) {
                Some(img) => ui.p.image(img, b.cell),
                None => ui.p.fill(b.cell, WALL_FILL),
            }
        }
        match cx.art.tile_in(sets, wall, b.x, b.y, art_px) {
            Some(img) => ui.p.image(img, cap),
            None => ui.p.fill(cap, WALL_FILL),
        }
        draw_wall_cap_edges(ui, cap, b.cell.h, !is_wall(b.x, b.y - 1), !is_wall(b.x - 1, b.y) || is_face(b.x - 1, b.y), !is_wall(b.x + 1, b.y) || is_face(b.x + 1, b.y));
        return;
    }
    if !is_wall(b.x, b.y - 1) {
        let cap = mapview::cap_rect(b.cell);
        match cx.art.tile_in(sets, wall, b.x, b.y, art_px) {
            Some(img) => ui.p.image(img, cap),
            None => ui.p.fill(cap, WALL_FILL),
        }
        draw_wall_cap_edges(ui, cap, b.cell.h, true, !is_wall(b.x - 1, b.y), !is_wall(b.x + 1, b.y));
        let face = mapview::face_rect(b.cell);
        draw_wall_face_at(ui, cx, walls, false, b.x, b.y, face, b.cell.h, false, true, !is_face(b.x + 1, b.y), !is_face(b.x - 1, b.y));
    } else {
        let up_depth = wall_depth_at(b.x, b.y - 1, face_tiles, size, is_wall);
        let y_top = if up_depth == 0 && !border_at(b.x, b.y - 1) { b.cell.y - mapview::lift_of(b.cell.h) } else { b.cell.y };
        let face = Rect::new(b.cell.x, y_top, b.cell.w, b.cell.y + b.cell.h - y_top);
        draw_wall_face_at(ui, cx, walls, false, b.x, b.y, face, b.cell.h, depth == 1 && up_depth > 0, up_depth == 0, !is_face(b.x + 1, b.y), !is_face(b.x - 1, b.y));
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_depth_floor_shadows_and_water(
    ui: &mut Ui<RAct>,
    geo: &MapGeom,
    (mw, mh): (i32, i32),
    (x0, x1): (i32, i32),
    (y0, y1): (i32, i32),
    has_ground: bool,
    ch_at: &impl Fn(i32, i32) -> char,
    is_wall: &impl Fn(i32, i32) -> bool,
) {
    for y in y0.max(0)..=y1.min(mh - 1) {
        for x in x0.max(0)..=x1.min(mw - 1) {
            let r = geo.cell(x, y);
            if !is_wall(x, y) {
                if is_wall(x, y - 1) {
                    let sh_h = (r.h * 7 / 16).max(2);
                    for k in 0..sh_h {
                        let a = (120.0 * (1.0 - k as f32 / sh_h as f32).powf(1.5)).round() as u8;
                        if a > 0 {
                            ui.p.fill(Rect::new(r.x, r.y + k, r.w, 1), rgba(0, 0, 0, a));
                        }
                    }
                }
                if is_wall(x - 1, y) {
                    let sh_w = (r.w * 5 / 16).max(2);
                    for k in 0..sh_w {
                        let a = (95.0 * (1.0 - k as f32 / sh_w as f32).powf(1.5)).round() as u8;
                        if a > 0 {
                            ui.p.fill(Rect::new(r.x + k, r.y, 1, r.h), rgba(0, 0, 0, a));
                        }
                    }
                }
            }
            if ch_at(x, y) == '~' {
                let lip = (r.w / 8).max(2);
                if ch_at(x, y - 1) != '~' {
                    let off = if has_ground && ch_at(x, y - 1) != '#' { lip } else { 0 };
                    let n_h = (r.h * 9 / 32).max(2).min(r.h - off);
                    for k in 0..n_h {
                        let a = (130.0 * (1.0 - k as f32 / n_h as f32)).round() as u8;
                        if a > 0 {
                            ui.p.fill(Rect::new(r.x, r.y + off + k, r.w, 1), rgba(0, 0, 0, a));
                        }
                    }
                }
                if ch_at(x - 1, y) != '~' {
                    let off = if has_ground && ch_at(x - 1, y) != '#' { lip } else { 0 };
                    let w_w = (r.w * 7 / 32).max(2).min(r.w - off);
                    for k in 0..w_w {
                        let a = (110.0 * (1.0 - k as f32 / w_w as f32)).round() as u8;
                        if a > 0 {
                            ui.p.fill(Rect::new(r.x + off + k, r.y, 1, r.h), rgba(0, 0, 0, a));
                        }
                    }
                }
            }
        }
    }
}

fn canvas_visible_rect(c: &aegis_ui::canvas::Canvas, r: Rect) -> Option<Rect> {
    let mut out = r.intersect(Rect::new(0, 0, c.w, c.h));
    if let Some(clip) = c.clip {
        out = out.intersect(clip);
    }
    if let Some(bounds) = c.bounds {
        out = out.intersect(bounds);
    }
    (out.w > 0 && out.h > 0).then_some(out)
}

/// Pulls wall-mounted masonry tones (`max - min < 40 && max < 150`) 75% toward the sampled underlying wall face.
fn into_wall_tone(c: &aegis_ui::canvas::Canvas, dest: Rect, img: &aegis_ui::canvas::Image) -> aegis_ui::canvas::Image {
    let sample_rect = Rect::new(dest.x, dest.y, dest.w, (dest.h * 3 / 4).max(1));
    let Some(vis) = canvas_visible_rect(c, sample_rect) else { return img.clone() };
    let (mut sr, mut sg, mut sb, mut sc) = (0u64, 0u64, 0u64, 0u64);
    for py in vis.y..(vis.y + vis.h) {
        let row = (py * c.w) as usize;
        for px in vis.x..(vis.x + vis.w) {
            let i = (row + px as usize) * 4;
            sr += c.px[i] as u64;
            sg += c.px[i + 1] as u64;
            sb += c.px[i + 2] as u64;
            sc += 1;
        }
    }
    if sc == 0 {
        return img.clone();
    }
    let face_mean = [sr as f32 / sc as f32, sg as f32 / sc as f32, sb as f32 / sc as f32];
    let is_masonry = |r: u8, g: u8, b: u8, a: u8| {
        if a < 128 {
            return false;
        }
        let hi = r.max(g).max(b);
        let lo = r.min(g).min(b);
        hi - lo < 40 && hi < 150
    };
    let (mut pr, mut pg, mut pb, mut pc) = (0u64, 0u64, 0u64, 0u64);
    for ch in img.px.chunks_exact(4) {
        if is_masonry(ch[0], ch[1], ch[2], ch[3]) {
            pr += ch[0] as u64;
            pg += ch[1] as u64;
            pb += ch[2] as u64;
            pc += 1;
        }
    }
    if pc == 0 {
        return img.clone();
    }
    let shift = [
        (face_mean[0] - pr as f32 / pc as f32) * 0.75,
        (face_mean[1] - pg as f32 / pc as f32) * 0.75,
        (face_mean[2] - pb as f32 / pc as f32) * 0.75,
    ];
    let mut out = img.clone();
    for ch in out.px.chunks_exact_mut(4) {
        if is_masonry(ch[0], ch[1], ch[2], ch[3]) {
            for k in 0..3 {
                ch[k] = (ch[k] as f32 + shift[k]).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    out
}

/// REGIONVIEW2: opt-in evening ambience darkened over `map_area` and warmed cleanly by per-art light pools.
fn apply_dim_ambience(
    ui: &mut Ui<RAct>,
    geo: &MapGeom,
    props: &[PropDraw],
    prop_arts: &[(String, i32, i32, String)],
    opaque_roofs: &[Rect],
    is_wall: &impl Fn(i32, i32) -> bool,
    wall_face_tiles: i32,
) {
    let Some(area) = canvas_visible_rect(ui.p.c, Rect::new(0, 0, ui.p.c.w, ui.p.c.h)) else { return };
    struct Light {
        lx: f32,
        ly: f32,
        inv_rx2: f32,
        inv_ry2: f32,
        bx0: i32,
        by0: i32,
        bx1: i32,
        by1: i32,
        col: [f32; 3],
    }
    let (cw, ch) = geo.cell_size();
    let mut lights = Vec::new();
    for p in props {
        if p.roof.is_some() {
            continue;
        }
        if opaque_roofs.iter().any(|rr| {
            let hit = p.foot.intersect(*rr);
            hit.w > 0 && hit.h > 0
        }) {
            continue;
        }
        let Some(art) = prop_art_of(prop_arts, geo, p) else { continue };
        let Some((rad, col)) = light_pool(art) else { continue };
        let (fit, _) = prop_mounted_fit(Some(art), p, geo, is_wall, wall_face_tiles);
        let dr = fit.unwrap_or(p.rect);
        let lx = ui.p.v.x(dr.x as f32 + dr.w as f32 * 0.5) as f32;
        let ly = ui.p.v.y((dr.y + dr.h) as f32 - 14.0 * ch as f32 / ART_TILE as f32) as f32;
        let rx = (rad as f32 * cw as f32 / ART_TILE as f32) * ui.p.v.s;
        let ry = (rad as f32 * ch as f32 / ART_TILE as f32) * ui.p.v.s;
        if rx < 1.0 || ry < 1.0 {
            continue;
        }
        let bx0 = (lx - rx).floor().max(area.x as f32) as i32;
        let by0 = (ly - ry).floor().max(area.y as f32) as i32;
        let bx1 = (lx + rx).ceil().min((area.x + area.w) as f32) as i32;
        let by1 = (ly + ry).ceil().min((area.y + area.h) as f32) as i32;
        if bx1 > bx0 && by1 > by0 {
            lights.push(Light {
                lx,
                ly,
                inv_rx2: 1.0 / (rx * rx),
                inv_ry2: 1.0 / (ry * ry),
                bx0,
                by0,
                bx1,
                by1,
                col: [col[0] as f32, col[1] as f32, col[2] as f32],
            });
        }
    }
    let w = ui.p.c.w;
    for py in area.y..(area.y + area.h) {
        let fy = py as f32 + 0.5;
        let row = (py * w) as usize;
        for px in area.x..(area.x + area.w) {
            let fx = px as f32 + 0.5;
            let mut min_dark = 96.0_f32;
            let (mut gr, mut gg, mut gb, mut gw) = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
            for l in &lights {
                if px < l.bx0 || px >= l.bx1 || py < l.by0 || py >= l.by1 {
                    continue;
                }
                let dx = fx - l.lx;
                let dy = fy - l.ly;
                let d2 = dx * dx * l.inv_rx2 + dy * dy * l.inv_ry2;
                if d2 < 1.0 {
                    let d = d2.sqrt();
                    min_dark = min_dark.min(96.0 * d.powf(1.6));
                    let wt = (1.0 - d).powf(1.3) * 0.28;
                    gr += l.col[0] * wt;
                    gg += l.col[1] * wt;
                    gb += l.col[2] * wt;
                    gw += wt;
                }
            }
            let i = (row + px as usize) * 4;
            let da = min_dark / 255.0;
            let inv = 1.0 - da;
            let mut r = ui.p.c.px[i] as f32 * inv + 6.0 * da;
            let mut g = ui.p.c.px[i + 1] as f32 * inv + 8.0 * da;
            let mut b = ui.p.c.px[i + 2] as f32 * inv + 16.0 * da;
            if gw > 0.0 {
                let ga = gw.min(0.32);
                let ig = 1.0 - ga;
                r = r * ig + (gr / gw) * ga;
                g = g * ig + (gg / gw) * ga;
                b = b * ig + (gb / gw) * ga;
            }
            ui.p.c.px[i] = r.round().clamp(0.0, 255.0) as u8;
            ui.p.c.px[i + 1] = g.round().clamp(0.0, 255.0) as u8;
            ui.p.c.px[i + 2] = b.round().clamp(0.0, 255.0) as u8;
        }
    }
}

/// A prop, cut at the map's rect (REGIONVIEW1: only boundary art past the map). The clip is physical px under the
/// frame's view, so the cut moves with the map.
fn draw_prop(ui: &mut Ui<RAct>, cx: &mut Cx, p: &PropDraw, fit: Option<Rect>, mounted: bool, map: Rect, q: i32) {
    let draw_rect = fit.unwrap_or(p.rect);
    let leaves = draw_rect.intersect(map) != draw_rect;
    let kept = ui.p.c.clip;
    if leaves {
        let inside = ui.p.v.rect(map);
        ui.p.c.clip = Some(kept.map_or(inside, |c| c.intersect(inside)));
    }
    draw_prop_art_at(ui, cx, p, draw_rect, mounted, map, q);
    ui.p.c.clip = kept;
}

/// Legacy test entrypoint: draws prop art in `p.rect`.
#[cfg(test)]
fn draw_prop_art(ui: &mut Ui<RAct>, cx: &mut Cx, p: &PropDraw, q: i32) {
    draw_prop_art_at(ui, cx, p, p.rect, false, Rect::new(-1_000_000, -1_000_000, 2_000_000, 2_000_000), q);
}

/// A prop (PROPS1 4.3, 4.6): its art in its drawn rect (mirrored when flipped), one source pixel per screen pixel;
/// standing props draw ONLY a subtle thin dark base contact line at feet (NO cast oval floor shadows). Without art a
/// standing prop is a plain block over its footprint (`q`: the zoom).
fn draw_prop_art_at(ui: &mut Ui<RAct>, cx: &mut Cx, p: &PropDraw, draw_rect: Rect, mounted: bool, map: Rect, q: i32) {
    let (w, h) = ((draw_rect.w as f32 * ui.p.v.s).round().max(1.0) as i32, (draw_rect.h as f32 * ui.p.v.s).round().max(1.0) as i32);
    let img = match &p.file {
        Some(f) if p.flip => cx.art.image_mirrored(f, w, h),
        Some(f) => cx.art.image(f, w, h),
        None => None,
    };
    if let Some(img) = img {
        if !p.ground && p.roof.is_none() && !mounted {
            if let Some(b) = img.opaque_bounds(1) {
                let lx = draw_rect.x + (b.x * draw_rect.w) / img.w + 1;
                let rx = draw_rect.x + ((b.x + b.w) * draw_rect.w + img.w - 1) / img.w - 1;
                if rx > lx {
                    let by = draw_rect.y + ((b.y + b.h) * draw_rect.h + img.h / 2) / img.h;
                    let thin = (q / 4).max(1);
                    let contact = Rect::new(lx, by - thin, rx - lx, thin + 1).intersect(map);
                    if contact.w > 0 && contact.h > 0 {
                        ui.p.fill(contact, rgba(0, 0, 0, 95));
                    }
                }
            }
        }
        if mounted {
            let dest = ui.p.v.rect(draw_rect);
            let pulled = into_wall_tone(ui.p.c, dest, img);
            ui.p.image(&pulled, draw_rect);
        } else {
            ui.p.image(img, draw_rect);
        }
        return;
    }
    if p.ground {
        return;
    }
    let r = draw_rect;
    ui.p.fill(r, BOX_FILL);
    let face = if p.file.is_none() && r.y < p.foot.y { p.foot.y - r.y } else { (r.h / 4).clamp(1, BOX_FACE_H * q / 4) };
    ui.p.fill(Rect::new(r.x, r.y + r.h - face, r.w, face), BOX_FACE);
    let e = (q / 4).max(1);
    for edge in [Rect::new(r.x, r.y, r.w, e), Rect::new(r.x, r.y + r.h - e, r.w, e), Rect::new(r.x, r.y, e, r.h), Rect::new(r.x + r.w - e, r.y, e, r.h)] {
        ui.p.fill(edge, BOX_EDGE);
    }
}

/// REVEAL1 1.3: a roof prop at opacity `alpha` (0 draws nothing), cut at the map's rect as every prop: its art, else a
/// muted block with a darker eaves band along its bottom and a rim.
fn draw_roof(ui: &mut Ui<RAct>, cx: &mut Cx, p: &PropDraw, map: Rect, q: i32, alpha: u8) {
    if alpha == 0 {
        return;
    }
    let kept = ui.p.c.clip;
    if p.rect.intersect(map) != p.rect {
        let inside = ui.p.v.rect(map);
        ui.p.c.clip = Some(kept.map_or(inside, |c| c.intersect(inside)));
    }
    let (w, h) = ((p.rect.w as f32 * ui.p.v.s).round().max(1.0) as i32, (p.rect.h as f32 * ui.p.v.s).round().max(1.0) as i32);
    let img = match &p.file {
        Some(f) if p.flip => cx.art.image_mirrored(f, w, h),
        Some(f) => cx.art.image(f, w, h),
        None => None,
    };
    match img {
        Some(img) => ui.p.image_alpha(img, p.rect, alpha),
        None => {
            let with = |c: Rgba| rgba(c[0], c[1], c[2], alpha);
            let r = p.rect;
            ui.p.fill(r, with(crate::props::ROOF_FILL));
            let eaves = (r.h / 4).clamp(1, BOX_FACE_H * q / 4);
            ui.p.fill(Rect::new(r.x, r.y + r.h - eaves, r.w, eaves), with(crate::props::ROOF_EAVES));
            let e = (q / 4).max(1);
            for edge in [Rect::new(r.x, r.y, r.w, e), Rect::new(r.x, r.y, e, r.h - eaves), Rect::new(r.x + r.w - e, r.y, e, r.h - eaves)] {
                ui.p.fill(edge, with(crate::props::ROOF_RIM));
            }
        }
    }
    ui.p.c.clip = kept;
}

/// REVEAL1 1.1: each wall set an indoor zone of template `t` names, once, with the tile sets its walls draw from
/// (`art::interior_wall_sets`: its own, then the region `sets`).
fn interior_sets(t: &Dict, sets: &[String]) -> Vec<(String, Vec<String>)> {
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    for z in json::dget(t, "zone_legend").values().filter_map(Value::as_object) {
        let set = explore::zone_wall_set(t, &s(z, "id"));
        if !set.is_empty() && !out.iter().any(|(s, _)| *s == set) {
            let names = emberwake_content::art::interior_wall_sets(sets, &set);
            out.push((set, names));
        }
    }
    out
}

/// REVEAL1 2.2: the boundary's edge dressing past the map (cells `x0..=x1`, `y0..=y1` of the frame): its pieces
/// (`props/<set>/boundary_<boundary>/v<n>.png` over the world lineage, then `common`) repeat on a lattice of the first
/// piece's size in cells, each picked by its cell, and a piece is drawn only where all its cells lie past the map, so
/// nothing covers the map. Without art it draws nothing (the border's own art or wall tile stays).
#[allow(clippy::too_many_arguments)]
fn draw_dressing(ui: &mut Ui<RAct>, cx: &mut Cx, lineage: &[String], boundary: &str, geo: &MapGeom, (mw, mh): Pos, (x0, x1): (i32, i32), (y0, y1): (i32, i32)) {
    let dirs = emberwake_content::art::boundary_dressing_dirs(lineage, boundary);
    let dirs: Vec<&str> = dirs.iter().map(String::as_str).collect();
    let Some(first) = cx.art.pick(&dirs, Some(0), 0, 0) else { return };
    let Some((pw, ph)) = cx.art.size_of(&first) else { return };
    let (cw, ch) = geo.cell_size();
    // the lattice in cells (32 px of art a cell), at least one
    let (lw, lh) = (((pw + ART_TILE - 1) / ART_TILE).max(1), ((ph + ART_TILE - 1) / ART_TILE).max(1));
    let (mw, mh) = (mw as i32, mh as i32);
    let past = |x: i32, y: i32| x + lw <= 0 || y + lh <= 0 || x >= mw || y >= mh;
    for y in (y0 - lh + 1)..=y1 {
        for x in (x0 - lw + 1)..=x1 {
            if x.rem_euclid(lw) != 0 || y.rem_euclid(lh) != 0 || !past(x, y) {
                continue;
            }
            let Some(rel) = cx.art.pick(&dirs, None, x, y) else { continue };
            let Some((aw, ah)) = cx.art.size_of(&rel) else { continue };
            let cell = geo.cell(x, y);
            let r = Rect::new(cell.x, cell.y, (aw * cw + ART_TILE / 2) / ART_TILE, (ah * ch + ART_TILE / 2) / ART_TILE);
            let (w, h) = ((r.w as f32 * ui.p.v.s).round().max(1.0) as i32, (r.h as f32 * ui.p.v.s).round().max(1.0) as i32);
            if let Some(img) = cx.art.image(&rel, w, h) {
                ui.p.image(img, r);
            }
        }
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
/// FLOATPANELS1: the area the legend and the ticker float in (the map under the HUD bar), and their drag payloads.
const MAP_AREA: Rect = Rect::new(0, HUD_H, W, H - HUD_H);
const LEGEND_PANEL_DRAG: &str = "region.legend";
const LOG_PANEL_DRAG: &str = "region.log";
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

/// PR7-36 (UX-28, ART-19): what an exit or point of interest shows on the region map and its legend: a small glyph per
/// kind, drawn by the game in the theme's colours (`glyph_colour`) over the kit's generic marker art.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MapGlyph {
    Exit,
    Chest,
    Rest,
    Vault,
    Lore,
    Look,
}

/// The glyph of a marker of `kind` (a point's by its `use`: `explore::point_use`); None: the marker keeps its art.
pub(crate) fn glyph_of(kind: &str, point_use: &str) -> Option<MapGlyph> {
    Some(match (kind, point_use) {
        ("exit", _) => MapGlyph::Exit,
        ("chest", _) | ("point", "stash") => MapGlyph::Chest,
        ("lore", _) => MapGlyph::Lore,
        ("point", "rest") => MapGlyph::Rest,
        ("point", "crypts") => MapGlyph::Vault,
        ("point", _) => MapGlyph::Look,
        _ => return None,
    })
}

/// A glyph's colour: amber while it can be used (`tokens::exit`), iron once used (`state` "spent"), a locked exit
/// oxblood.
pub(crate) fn glyph_colour(kind: &str, state: &str) -> Rgba {
    match (kind, state) {
        ("exit", "locked") => crate::tokens::exit(false),
        (_, "spent") => th().edge,
        _ => crate::tokens::exit(true),
    }
}

/// PR7-76 (PT-43): a point of interest the party can use now: present (not spent) with a use (`explore::point_use`) or a
/// flag to set; it joins the Here list and gets an edge arrow out of view. A point that only speaks stays off.
/// Known limit (`point_fired`): a point with both a use and `sets_flags` would have its use skipped when a click walk
/// arrives on it (the step's flag answer stands for it); no shipped point has both.
pub(crate) fn point_listed(m: &Dict, state: &str) -> bool {
    state == "present" && (!explore::point_use(m).is_empty() || !json::arr(&json::normalize(json::get(m, "sets_flags"))).is_empty())
}

/// A glyph's 8 x 8 picture, `#` drawn.
fn glyph_rows(g: MapGlyph) -> [&'static str; 8] {
    match g {
        MapGlyph::Exit => ["...#....", "...##...", "...###..", "#######.", "#######.", "...###..", "...##...", "...#...."],
        MapGlyph::Chest => ["........", ".######.", ".#....#.", "########", "###..###", "########", ".######.", "........"],
        MapGlyph::Rest => ["...#....", "...##...", "..###...", "..####..", ".#####..", ".######.", "..####..", "#.####.#"],
        MapGlyph::Vault => ["..####..", ".#....#.", "#.#..#.#", "#.#..#.#", "#.#..#.#", "#.#..#.#", "########", "........"],
        MapGlyph::Lore => ["........", ".##..##.", "#..##..#", "#..##..#", "#..##..#", "###..###", "...##...", "........"],
        MapGlyph::Look => ["........", "...##...", "..#..#..", ".#.##.#.", ".#.##.#.", "..#..#..", "...##...", "........"],
    }
}

/// STYLE-PASS1B (`PR7-48`, FB-366): the walk destination's ring in cell `r`: a flat amber ellipse on the floor where a
/// sprite's feet stand (the lower part of the cell, wider than a foot shadow so it shows around one), two strokes thick
/// (the outer full, the inner softer).
pub(crate) fn draw_walk_ring(p: &mut aegis_ui::painter::Painter, r: Rect) {
    let col = crate::tokens::walk_ring();
    let (w, h) = (r.w as f32, r.h as f32);
    let (cx, cy) = (r.x as f32 + w / 2.0, r.y as f32 + h * WALK_RING.0);
    let (rx, ry) = (w * WALK_RING.1, h * WALK_RING.2);
    p.ellipse_ring(cx, cy, rx, ry, col);
    p.ellipse_ring(cx, cy, rx - 1.0, (ry - 1.0).max(1.0), rgba(col[0], col[1], col[2], 150));
}

/// The walk ring as shares of the cell (STYLE-PASS1B): its centre's height from the cell's top, its half width and its
/// half height.
pub(crate) const WALK_RING: (f32, f32, f32) = (0.75, 0.48, 0.22);

/// Draws glyph `g` in `col` on a dark plate filling `r` (a marker icon's rect), its cells whole logical px.
pub(crate) fn draw_map_glyph(p: &mut aegis_ui::painter::Painter, r: Rect, g: MapGlyph, col: Rgba) {
    p.panel(r, th().panel, rgba(col[0], col[1], col[2], 150), 2.0);
    let k = ((r.w.min(r.h) - 4) / 8).max(1);
    let (x0, y0) = (r.x + (r.w - 8 * k) / 2, r.y + (r.h - 8 * k) / 2);
    for (y, row) in glyph_rows(g).iter().enumerate() {
        for (x, c) in row.bytes().enumerate() {
            if c == b'#' {
                p.fill(Rect::new(x0 + x as i32 * k, y0 + y as i32 * k, k, k), col);
            }
        }
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
            // PK2-S3: the recipe service
            "recipe" => "Recipes",
            // PR6-8: the facility service
            "facility" => "Facilities",
            _ => "",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The drawn stand-in of a service icon: today's `?` and `!` badges (a board with work is the `!`, STORYUI1 B4), a coin
/// for a shop, a sheet with lines for recipes (PK2-S3), a house for facilities (PR6-8), a plus for recruits.
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
        "recipe" => {
            let (w, h) = (sz * 0.6, sz * 0.8);
            let sheet = Rect::new((cx_ - w / 2.0).round() as i32, (cy_ - h / 2.0).round() as i32, w.round().max(1.0) as i32, h.round().max(1.0) as i32);
            ui.p.fill(sheet, th().text);
            for k in 1..=3 {
                let y = cy_ - h / 2.0 + h * k as f32 / 4.0;
                ui.p.line(cx_ - w * 0.35, y, cx_ + w * 0.35, y, th().dim, 0.0);
            }
        }
        "facility" => {
            // a house: a body under a roof
            let (w, h) = (sz * 0.6, sz * 0.4);
            let body = Rect::new((cx_ - w / 2.0).round() as i32, cy_.round() as i32 - 1, w.round().max(1.0) as i32, h.round().max(1.0) as i32);
            ui.p.fill(body, th().text);
            ui.p.line(cx_ - sz * 0.42, cy_ - 1.0, cx_, cy_ - sz * 0.42, th().gold, 0.0);
            ui.p.line(cx_, cy_ - sz * 0.42, cx_ + sz * 0.42, cy_ - 1.0, th().gold, 0.0);
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

/// The region map's controls (the Help button's hover card and popup), with the keys `c` binds (CONTROLS-UI1B).
pub fn region_help(c: &Controls) -> Vec<String> {
    let k = |ch: char| c.key_hint(Key::Char(ch));
    vec![
        format!("Arrows or {}: move", walk_keys(c)),
        format!("{} or {}: use what the party faces", k('e'), c.key_hint(Key::Enter)),
        "Click: walk there, or on a marker to use it".into(),
        "Legend row or edge arrow: look there (again: walk there)".into(),
        format!("{} or a click on the leader: the next member leads", k('q')),
        "1-9: pick a reply in a conversation".into(),
        party_keys(c),
        format!("{}: the last events, or only the latest", k('l')),
        format!("{}: back, or the game menu", c.key_hint(Key::Esc)),
    ]
}

/// The walk keys as one word ("WASD"), or spaced when one is longer than a letter ("W A S Space").
pub fn walk_keys(c: &Controls) -> String {
    let keys: Vec<String> = ['w', 'a', 's', 'd'].map(|ch| c.key_hint(Key::Char(ch))).to_vec();
    keys.join(if keys.iter().all(|k| k.chars().count() == 1) { "" } else { " " })
}

/// The HUD keys' help line ("P: party   J: journal   I: stash"), as `c` binds them.
pub fn party_keys(c: &Controls) -> String {
    let k = |ch: char| c.key_hint(Key::Char(ch));
    format!("{}: party   {}: journal   {}: stash", k('p'), k('j'), k('i'))
}

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
        for ok in ["tilesets/ashlands/ground/mud/v0.png", "tilesets/common/boundary/keep/wall/v0.png", "tilesets/common/boundary/keep/wall_face/v2.png", "tilesets/sunk_ward_stage/front_face/floor/v3.png", "tilesets/sunk_ward_stage/stage.fx.json", "tilesets/sunk_ward_stage/cover/v0.png", "tilesets/sunk_ward_stage/wall_foot/v1.png", "tilesets/sunk_ward_foundry_stage/lava/front_face/v0.png", "tilesets/sunk_ward_stage/floor/sheet.png", "tilesets/sunk_ward_stage/floor/decal/v0.png", "tilesets/sunk_ward_stage/lip/lava/v1.png", "props/common/exit_path/v0.png", "props/common/exit_gate/v0.png", "tilesets/sunk_ward_stage/lava/over_floor/m01.png", "tilesets/sunk_ward_foundry_stage/floor/over_lava/m14.png"] {
            assert!(layout_ok(ok), "{ok}");
        }
        for stray in ["tilesets/ashlands/ground/mud/v1.png", "tilesets/ashlands/ground/mud.png", "tilesets/ashlands/ground/mud/over_ash/m01.png", "tilesets/common/boundary/keep/roof/v0.png", "tilesets/common/boundary/keep/wall/face.png", "tilesets/sunk_ward_stage/front_face/floor.png", "tilesets/sunk_ward_stage/floor/stage.fx.json", "tilesets/high_bastion_stage/backdrop_ramparts.png", "tilesets/sunk_ward_foundry_stage/lava/front_face/v0.jpg", "tilesets/sunk_ward_stage/floor/sheet1.png", "tilesets/sunk_ward_stage/floor/decal/a.png", "tilesets/sunk_ward_stage/lip/lava.png", "tilesets/ashlands/floor/over_mud/m01.png", "tilesets/sunk_ward_stage/lava/over_floor/m15.png", "tilesets/sunk_ward_stage/lava/over_floor/v0.png"] {
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
        for ok in ["portraits/npcs/talia_rusk.png", "portraits/npcs/talia_rusk.face.json", "portraits/common/beast_render_beast.face.json", "portraits/heroes/faces/kaelen/hurt.png", "portraits/bosses/faces/proctor_quell/neutral.face.json"] {
            assert!(layout_ok(ok), "{ok}");
        }
        for stray in ["portraits/npcs/talia_rusk.json", "portraits/heroes/faces/kaelen/smirk.png", "portraits/common/faces/beast/hurt.png", "portraits/npcs/faces/talia_rusk.png", "portraits/npcs/faces/talia_rusk/hurt/a.png", "portraits/npcs/faces/talia_rusk/hurt.jpg"] {
            assert!(!layout_ok(stray), "{stray}");
        }
    }

    /// MAP-HOOKS1: landmarks `map/landmarks/<id>.png`; a world map background `<name>.png` and its layout sidecar
    /// `<name>.json`; other background folders stay PNG only, and the location backdrops keep their arm.
    #[test]
    fn landmarks_and_world_map_sidecars_follow_the_layout() {
        for ok in ["map/landmarks/fourth_kiln.png", "backgrounds/world_map/sunk_ward_quarter.png", "backgrounds/world_map/sunk_ward_quarter.json", "backgrounds/location/keep.png"] {
            assert!(layout_ok(ok), "{ok}");
        }
        for stray in ["map/landmarks/fourth_kiln.json", "map/landmarks/a/fourth_kiln.png", "map/fourth_kiln.png", "backgrounds/world_map/x.txt", "backgrounds/title/ashlands.json", "backgrounds/location/keep.json"] {
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
            // PASSIVE-ICON1: the kit's passive icon archetypes
            ["passives", f] => png(f),
            // LOOK1-G1: the theme file and its nine-slice frames (`<role>.png` with a `<role>.json` sidecar)
            ["ui", "theme.json"] => true,
            // UISPLIT1: screen layout files and their view fixtures
            ["ui", "layouts", f] => f.ends_with(".json"),
            ["ui", "views", f] => f.ends_with(".view.json"),
            ["ui", "frames", f] => png(f) || f.ends_with(".json"),
            ["ui", "pins" | "badges" | "chips" | "intents" | "hud" | "markers" | "services", f] | ["ui", f] => png(f),
            // DIORAMA5 slice 0 (PR7-77): a terrain's ground sheet (before the `v*` pattern below)
            ["tilesets", _, _, "sheet.png"] => true,
            ["tilesets", _, _, f] | ["props", _, _, f] => v(f),
            // REGIONGROUND1: a ground's one texture (only `v0` is read), and a boundary's tiles
            ["tilesets", _, "ground", _, "v0.png"] => true,
            ["tilesets", _, "boundary", _, "wall" | "wall_face", f] => v(f),
            // DIORAMA2 R2 and DIORAMA3 R3 (ASSETORG2): a stage set's front face per terrain and its light spots
            ["tilesets", _, "front_face", _, f] => v(f),
            ["tilesets", _, "stage.fx.json"] => true,
            // DIORAMA4 S2 (PR7-74): a zone's terrain override folder's own front face (`<set>/<terrain>/front_face/v*`)
            ["tilesets", _, _, "front_face", f] => v(f),
            // DIORAMA5 slice 0 (PR7-77): a terrain's ground sheet, its accent decals, a sunken terrain's lip art, and
            // the exit marker's path decal and gate (`props/common/exit_path/v*`, `exit_gate/v*`, the props pattern)
            ["tilesets", _, _, "decal", f] => v(f),
            ["tilesets", _, "lip", _, f] => v(f),
            // A stage set's transition pieces (`<upper>/over_<lower>/m01..m14`, LOOK1 5.3, 64 x 48), a stopgap for
            // Art/UI's ragged edges until the composer draws the battle ground; region sets keep refusing them
            ["tilesets", set, _, over, f] if set.ends_with("_stage") && over.starts_with("over_") => {
                f.strip_prefix('m').and_then(|n| n.strip_suffix(".png")).filter(|n| n.len() == 2).and_then(|n| n.parse::<u32>().ok()).is_some_and(|n| (1..=14).contains(&n))
            }
            // MAP-HOOKS1: a location's landmark on the world map, and a world map background with its layout sidecar
            ["map", "landmarks", f] => png(f),
            ["backgrounds", "world_map", f] => png(f) || f.ends_with(".json"),
            // TITLEFX1 (PR7-28): a menu picture's fire spots `<picture>.fx.json` beside it
            ["backgrounds", "menu", f] => png(f) || f.ends_with(".fx.json"),
            ["backgrounds", "title" | "location" | "battle" | "story", f] => png(f),
            ["audio", "sfx", f] => f.ends_with(".wav"),
            ["audio", "music", f] => f.ends_with(".ogg"),
            // AMBIENCE1: a place's ambient loop
            ["audio", "ambience", f] => f.ends_with(".ogg"),
            // TRANSITIONS1 (AI-18): the battle beats' wipe and banners, and their one-shot stings
            ["ui", "transitions", f] => png(f),
            ["audio", "stings", f] => f.ends_with(".ogg"),
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

    /// The markers the legend lists, in order: present (or spent or locked), a point only while usable (PT-43).
    fn listed(cx: &Cx) -> Vec<Dict> {
        let gs = cx.gs();
        let loc = explore::location(&cx.data, gs);
        explore::markers_of(loc)
            .into_iter()
            .filter(|m| {
                let st = s(&explore::marker_view(&cx.data, gs, loc, m), "state");
                st != "absent" && (s(m, "kind") != "point" || point_listed(m, &st))
            })
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
        // FLOATPANELS1: a floating panel, its title row holding the lock and minimize buttons
        let head = widget(&map, "legend").expect("the legend panel");
        assert_eq!(head["kind"], json!("backdrop"));
        let (hx, hy, hw, _) = rect(&head);
        assert_eq!((hx + hw, hy), ((W - 4) as i64, (HUD_H + 4) as i64), "top right under the bar");
        let buttons = (2 * PANEL_BTN + 8) as i64;
        assert!(hw <= LEGEND_MAX as i64 + buttons, "at most {LEGEND_MAX} wide at Normal past its buttons: {hw}");
        assert!(widget(&map, "legend.lock").is_some() && widget(&map, "legend.min").is_some());
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
        // minimize shrinks it in place to its "Here (N)" button, kept in the settings file; the button opens it again
        let min = rect(&widget(&map, "legend.min").unwrap());
        map.click_id(&mut cx, "legend.min").expect("minimize");
        assert_eq!(map.state(&cx)["legend_open"], json!(false));
        frame(&mut map, &mut cx, &mut g, 1.0);
        let chip = widget(&map, "legend.open").expect("the named button");
        assert_eq!(chip["text"], head["text"]);
        let c = rect(&chip);
        assert_eq!((c.0 + c.2, c.1), (min.0 + min.2, min.1 - 1), "the button's top right where the minimize button's was");
        assert!(widget(&map, "legend").is_none() && ids_with(&map, "here.").is_empty(), "minimized");
        let saved = Settings::load(cx.shot_dir.join("settings.json"), crate::screens::default_settings());
        assert_eq!(saved.v["map_legend_open"], json!(false), "the fold is saved");
        map.click_id(&mut cx, "legend.open").expect("open");
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(ids_with(&map, "here."), want);
    }

    /// FLOATPANELS1 (`preferences/ui.md` "Floating panels"): the ticker floats: locked it stays put; unlocked its body
    /// drags and stays where it was dropped (kept in the settings with its lock); minimize shrinks it in place to a
    /// "Log" button that opens it again where it was; its bottom edge stays put while L opens and closes it.
    #[test]
    fn the_ticker_floats_drags_unlocked_and_minimizes_in_place() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        cx.log = (0..20).map(|i| format!("Event {i}.")).collect();
        frame(&mut map, &mut cx, &mut g, 1.0);
        let at = |map: &RegionMap, id: &str| rect(&widget(map, id).unwrap_or_else(|| panic!("no {id}")));
        let r = at(&map, "log.panel");
        assert!(r.0 <= 4 && r.1 + r.3 >= (H - 4) as i64, "bottom left: {r:?}");
        assert_eq!(widget(&map, "log.panel.lock").unwrap()["text"], json!("Unlock"), "starts locked");
        // L opens the last 12: the panel grows upward, its bottom edge in place
        map.key(&mut cx, Key::Char('l'));
        frame(&mut map, &mut cx, &mut g, 1.0);
        let open = at(&map, "log.panel");
        assert_eq!(open.1 + open.3, r.1 + r.3, "the bottom stays");
        assert!(open.3 > r.3);
        map.key(&mut cx, Key::Char('l'));
        frame(&mut map, &mut cx, &mut g, 1.0);
        // locked: a drag does nothing
        let held = aegis_ui::ui::Drag { key: LOG_PANEL_DRAG.into(), id: "log.panel".into(), text: "Log".into() };
        let grab = ((r.0 + r.2 - 30) as i32, (r.1 + r.3 / 2) as i32);
        map.draw(&mut cx, &mut g, grab, true);
        g.kit.drag = Some(held.clone());
        map.draw(&mut cx, &mut g, (grab.0 + 200, grab.1 - 100), true);
        g.kit.drag = None;
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(at(&map, "log.panel"), r, "locked: stays");
        // unlocked: it drags and the place is kept in the settings
        map.click_id(&mut cx, "log.panel.lock").expect("unlock");
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(widget(&map, "log.panel").unwrap()["drag"], json!(LOG_PANEL_DRAG));
        map.draw(&mut cx, &mut g, grab, true);
        g.kit.drag = Some(held);
        map.draw(&mut cx, &mut g, (grab.0 + 200, grab.1 - 100), true);
        g.kit.drag = None;
        frame(&mut map, &mut cx, &mut g, 1.0);
        let moved = at(&map, "log.panel");
        assert_eq!((moved.0, moved.1), (r.0 + 200, r.1 - 100), "dropped there");
        let saved = Settings::load(cx.shot_dir.join("settings.json"), crate::screens::default_settings());
        assert_eq!(saved.v[crate::floats::REGION_LOG_KEY], json!({"pos": [moved.0, moved.1], "locked": false}));
        // minimize toggles in place
        let min = at(&map, "log.panel.min");
        map.click_id(&mut cx, "log.panel.min").expect("minimize");
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert!(widget(&map, "log.panel").is_none() && widget(&map, "log").is_none(), "minimized");
        let chip = widget(&map, "log.panel.open").expect("the Log button");
        assert_eq!(chip["text"], json!("Log"));
        let c = rect(&chip);
        assert_eq!((c.0 + c.2, c.1), (min.0 + min.2, min.1 - 1), "where the minimize button was");
        map.click_id(&mut cx, "log.panel.open").expect("open");
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(at(&map, "log.panel.min"), min, "restored in place");
    }

    /// ENGINE-48 (INPUT-REACH1): the arrows walk the party, but in a conversation (nothing walks) they move the focus over
    /// its replies, so a pad's d-pad can pick one.
    #[test]
    fn the_arrows_move_the_focus_in_a_conversation() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert!(!map.arrow_focus(), "on the map the arrows walk");
        let world = cx.gs().world_id.clone();
        let people: Vec<String> = cx.data.merged_entries(&world, "characters").iter().map(|c| s(c, "id")).collect();
        for p in people {
            map.talk(&mut cx, &p);
            if map.svc.talking() {
                break;
            }
        }
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert!(map.svc.talking() && map.arrow_focus(), "in a conversation the arrows move the focus");
        let infos = map.widgets();
        // the replies that can be picked (one may be shut: its widget is off)
        let choices: Vec<String> = infos.iter().filter(|w| aegis_ui::focus::focusable(w, false)).filter_map(|w| w["id"].as_str()).filter(|i| i.starts_with("choice.")).map(String::from).collect();
        assert!(choices.len() > 1, "the conversation offers replies: {choices:?}");
        let reached = aegis_ui::reach::arrow_reach(&infos, choices.first().map(String::as_str));
        assert!(choices.iter().all(|c| reached.contains(c)), "the arrows reach every reply: {choices:?} in {reached:?}");
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
        // the map's brightness left of the box (the legend and the ticker are hidden while talking) and of the shop's
        // frame (PR7-36: the ornate main-panel tier, whose corners reach out past the panel, and the rows' icon tiles)
        let light = |g: &Gfx| -> u64 {
            let mut sum = 0;
            for y in (HUD_H + 2)..(H - 10) {
                for x in 0..14 {
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
    use rpgkit_ui::portrait_fx::{blink_phase, blink_starts, seed_of, tq, BLINK_MS, EASE_MS};
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
        // PAINTED-MAPS1: a painted leader stands by its foot row (`a_painted_leader_stands_on_the_tiles_foot_point`)
        let clips = cx.art.clips(&want).cloned();
        assert_eq!(r, mapview::body_rect(t, clips.as_ref()), "the body's rect");
        if clips.and_then(|c| c.foot).is_none() {
            assert_eq!((r.w, r.h), (SPRITE, SPRITE), "1.5 tiles tall");
            assert_eq!((r.y + r.h, r.x + r.w / 2), (t.y + t.h, t.x + t.w / 2), "bottom-centred on the tile");
        }
    }

    /// PAINTED-MAPS1 (PR7-73): a catalog unit whose body (as the art resolves it) has a `foot` row (`painted`) or has
    /// none: its id.
    fn unit_with(cx: &mut Cx, painted: bool) -> Option<String> {
        let d = cx.data.clone();
        let units: Vec<Dict> = d.content.get("units").map(|m| m.values().cloned().collect()).unwrap_or_default();
        let found = units.into_iter().find(|u| {
            let body = emberwake_content::art::body_of(&Dict::new(), u, d.entry("classes", &s(u, "class_id")));
            let shown = cx.art.first_body(&emberwake_content::art::body_chain(&body));
            shown.and_then(|b| cx.art.clips(&b).map(|k| k.foot.is_some())) == Some(painted)
        });
        found.map(|u| s(&u, "id"))
    }

    /// PAINTED-MAPS1 (PR7-73): under both views, a leader on a unit whose body has a `foot` row (picked by rule) stands
    /// `stage::PAINTED_CELLS` tile widths tall from its frame's top to its foot row, on the tile's foot point
    /// (`mapview::body_rect`); one whose body has none keeps the token square, bottom-centred on the tile.
    #[test]
    fn a_painted_leader_stands_on_the_tiles_foot_point() {
        for (view, _) in mapview::VIEWS {
            for painted in [true, false] {
                let (mut cx, mut g) = (art_cx(), gfx());
                cx.settings.v["region_view"] = json!(view);
                let who = unit_with(&mut cx, painted).expect("a unit with (or without) a painted body");
                let id = leader_id(&cx);
                let m = cx.gs_mut().party.member_mut(&id).expect("a leader");
                // the leader plays no character: its base unit's body (`leader_art`)
                (m.lore_ref, m.base_unit) = (String::new(), who.clone());
                let mut map = RegionMap::new(&mut cx);
                frame(&mut map, &mut cx, &mut g, 1.0);
                let sp = sprite(&map, &cx, "party").expect("the leader is a sprite");
                let body = sp["body"].as_str().unwrap_or_default().to_string();
                let clips = cx.art.clips(&body).cloned().expect("the body's sheet");
                assert_eq!(clips.foot.is_some(), painted, "{who}: body {body}");
                let (r, t) = (rect_of(&sp), tile_rect(&map, cx.gs().position));
                assert_eq!(r, mapview::body_rect(t, Some(&clips)), "{view} {who}: the drawn rect");
                if let Some(foot) = clips.foot {
                    let (long, h) = (clips.frame.0.max(clips.frame.1) as f32, clips.frame.1 as f32);
                    let k = r.w as f32 / long;
                    let tall = foot as f32 * k;
                    let want = crate::stage::PAINTED_CELLS * t.w as f32;
                    assert!((tall - want).abs() <= 1.0, "{view} {who}: about 1.15 tile widths tall: {tall} against {want}");
                    // the frame sits on the square's bottom edge, so its foot row is (h - foot) frame rows above it
                    let foot_y = (r.y + r.h) as f32 - (h - foot as f32) * k;
                    let (fx, fy) = mapview::body_foot(t);
                    assert!((foot_y - fy as f32).abs() <= 1.0, "{view} {who}: the foot row on the foot point: {foot_y} against {fy}");
                    assert_eq!(r.x + r.w / 2, fx, "{view} {who}: centred on it");
                } else {
                    assert_eq!(r, mapview::sprite_rect(t), "{view} {who}: the token square");
                }
            }
        }
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

    /// ENGINE-49 (WALK-FEEL1): a click walk ordered while the party stands shows the party moving on the very next
    /// frame, as a key press does (its first step is taken as of the frame the click came after; it showed a frame late).
    #[test]
    fn a_click_walk_moves_on_the_next_frame() {
        let (mut cx, mut g) = (art_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        let d = cx.data.clone();
        let start = cx.gs().clone();
        let loc = explore::location(&d, &start);
        let t = explore::template_of(&d, loc).clone();
        let t0 = Instant::now();
        map.tick_at(&mut cx, t0);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let far = explore::markers_of(loc).into_iter().map(|m| s(&m, "id")).find(|id| map.path_to(&cx, explore::marker_pos(&t, &explore::markers_of(loc).into_iter().find(|m| s(m, "id") == *id).unwrap(), start.explore_int("loc_steps")), true).is_some_and(|p| p.len() >= 3)).expect("a marker three steps away or more");
        let before = map.motion["drawn"].clone();
        let far_id = far.clone();
        map.act(&mut cx, RAct::Marker(far));
        assert!(!map.walk.is_empty(), "a walk is ordered");
        let next = t0 + Duration::from_millis(17);
        map.tick_at(&mut cx, next);
        assert_eq!(map.last_step, Some(t0), "the first step is dated at the frame the click came after");
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_ne!(map.motion["drawn"], before, "the next frame shows the party moving");
        assert_eq!(map.step_at, None, "the glide took the step");
        // after a long frame the first step is dated now (dated back, the walk would run to catch up)
        let (mut cx, mut g) = (art_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        map.tick_at(&mut cx, t0);
        frame(&mut map, &mut cx, &mut g, 1.0);
        map.act(&mut cx, RAct::Marker(far_id.clone()));
        let late = t0 + Duration::from_millis(300);
        map.tick_at(&mut cx, late);
        assert_ne!(map.last_step, Some(t0), "a long frame: not dated back to the frame before");
    }

    /// ENGINE-50 (MAP-FRAME1): a key press is dated at the frame before it when that frame was just before, and no more
    /// than `CLICK_STEP_BACK` back after a long frame (a 250 ms hitch), so a hitch adds no quick step to catch up.
    #[test]
    fn a_key_press_after_a_long_frame_is_dated_now() {
        let (mut cx, mut g) = (art_cx(), gfx());
        let _ = RegionMap::new(&mut cx);
        let start = cx.gs().clone();
        let d = cx.data.clone();
        let loc = explore::location(&d, &start);
        let t = explore::template_of(&d, loc).clone();
        let marked: Vec<Pos> = explore::markers_of(loc).iter().map(|m| explore::marker_pos(&t, m, start.explore_int("loc_steps"))).collect();
        let (w, h) = explore::size_of(&t);
        let free = |p: Pos| explore::is_open(&t, p.0, p.1) && !marked.contains(&p);
        let from = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).find(|&(x, y)| free((x, y)) && free((x + 1, y))).expect("two free tiles side by side");
        let t0 = Instant::now();
        for (gap, dated) in [(17u64, 0u64), (25, 0), (250, 225)] {
            let mut map = RegionMap::new(&mut cx);
            cx.gs_mut().position = from;
            map.tick_at(&mut cx, t0);
            frame(&mut map, &mut cx, &mut g, 1.0);
            map.key(&mut cx, Key::Right);
            assert_eq!(cx.gs().position, (from.0 + 1, from.1), "the press steps");
            let beat = map.next_walk.saturating_duration_since(t0);
            let now = t0 + Duration::from_millis(gap);
            map.tick_at(&mut cx, now);
            let at = t0 + Duration::from_millis(dated);
            assert_eq!((map.last_step, map.step_at), (Some(at), Some(at)), "{gap} ms after the frame: dated {dated} ms on");
            assert_eq!(map.next_walk, at + beat, "{gap} ms: the next step a beat after it");
            frame(&mut map, &mut cx, &mut g, 1.0);
        }
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

    /// The legend's markers (present, spent or locked, a point only while usable, PT-43) with a map place: (id, tile).
    fn listed(cx: &Cx) -> Vec<(String, Pos)> {
        let gs = cx.gs();
        let loc = explore::location(&cx.data, gs);
        let t = explore::template_of(&cx.data, loc);
        explore::markers_of(loc)
            .into_iter()
            .filter(|m| {
                let st = s(&explore::marker_view(&cx.data, gs, loc, m), "state");
                st != "absent" && (s(m, "kind") != "point" || point_listed(m, &st))
            })
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
        let p = |rect: Rect| PropDraw { id: "x".into(), rect, foot, file: None, flip: false, ground: false, roof: None, foot_y: foot.y + foot.h };
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
        let want = if crate::DEV_KEYS { "tilted" } else { "square" };
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

    /// STYLE-PASS1B (`PR7-48`, FB-366): a click on a floor tile rings that tile in amber while the walk lasts: the ring
    /// draws only inside the destination cell, a new click moves it to the new tile, and arrival clears it (state
    /// `walk_ring`). Static, so the same under Reduce motion.
    #[test]
    fn a_click_to_walk_rings_its_tile_until_arrival_or_a_new_order() {
        for still in [false, true] {
            let (mut cx, mut g) = (game_cx(), gfx());
            cx.settings.v["reduce_motion"] = json!(still);
            let mut map = RegionMap::new(&mut cx);
            let d = cx.data.clone();
            let start = cx.gs().clone();
            let t = explore::template_of(&d, explore::location(&d, &start)).clone();
            let (w, h) = explore::size_of(&t);
            let marked: Vec<Pos> = explore::markers_of(explore::location(&d, &start)).iter().map(|m| explore::marker_pos(&t, m, start.explore_int("loc_steps"))).collect();
            let free = |p: Pos| explore::is_open(&t, p.0, p.1) && !marked.contains(&p);
            let at = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).find(|&(x, y)| (0..5).all(|i| free((x + i, y)))).expect("a free row");
            cx.gs_mut().position = at;
            for _ in 0..3 {
                frame(&mut map, &mut cx, &mut g, 1.0);
            }
            assert_eq!((map.walk_ring(), map.state(&cx)["walk_ring"].clone()), (None, Value::Null), "no walk, no ring");
            let click = |map: &mut RegionMap, cx: &mut Cx, p: Pos| {
                let r = map.geo.cell(p.0 as i32, p.1 as i32);
                map.mouse = (r.x + r.w / 2, r.y + r.h / 2);
                map.act(cx, RAct::MapClick);
            };
            let dest = (at.0 + 4, at.1);
            click(&mut map, &mut cx, dest);
            assert_eq!(map.walk_ring(), Some(dest), "the clicked tile");
            assert_eq!(map.state(&cx)["walk_ring"], json!([dest.0, dest.1]));
            // the ring draws in the destination cell only, in the amber
            frame(&mut map, &mut cx, &mut g, 1.0);
            let ringed = g.canvas.px.clone();
            let kept = map.walk_dest.take();
            frame(&mut map, &mut cx, &mut g, 1.0);
            let plain = g.canvas.px.clone();
            map.walk_dest = kept;
            let cw = g.canvas.w as usize;
            let changed: Vec<(usize, usize)> = (0..ringed.len() / 4).filter(|i| ringed[i * 4..i * 4 + 3] != plain[i * 4..i * 4 + 3]).map(|i| (i % cw, i / cw)).collect();
            assert!(!changed.is_empty(), "the ring draws");
            let cell = map.geo.cell(dest.0 as i32, dest.1 as i32);
            let v = g.view;
            let (x0, y0, x1, y1) = (v.x(cell.x as f32) - 2, v.y(cell.y as f32) - 2, v.x((cell.x + cell.w) as f32) + 2, v.y((cell.y + cell.h) as f32) + 2);
            assert!(changed.iter().all(|&(x, y)| (x0..=x1).contains(&(x as i32)) && (y0..=y1).contains(&(y as i32))), "inside the cell {cell:?}");
            let amber = crate::tokens::walk_ring();
            let near = changed.iter().map(|&(x, y)| (0..3).map(|c| (ringed[(y * cw + x) * 4 + c] as i32 - amber[c] as i32).abs()).sum::<i32>()).min().unwrap();
            assert!(near <= 60, "the amber (an anti-aliased line: nearest {near})");
            // a new order moves the ring; arrival clears it
            let dest2 = (at.0 + 3, at.1);
            click(&mut map, &mut cx, dest2);
            assert_eq!(map.walk_ring(), Some(dest2), "a new order: its tile");
            let mut now = Instant::now() + Duration::from_secs(1);
            let mut ticks = 0;
            while !map.walk.is_empty() && ticks < 50 {
                now += Duration::from_millis(500);
                map.tick_at(&mut cx, now);
                ticks += 1;
            }
            assert_eq!(cx.gs().position, dest2, "arrived");
            assert_eq!((map.walk_ring(), map.state(&cx)["walk_ring"].clone()), (None, Value::Null), "arrival clears the ring");
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
        let debug = crate::DEV_KEYS;
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
                    cx.settings.v[rpgkit_ui::vignette::REDUCE_MOTION] = json!(reduce);
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

        /// The Settings screen's Map zoom row: a pick applies at once (SETTINGS-MOCK1); the setting holds the step and the
        /// map draws at it.
        #[test]
        fn picking_a_map_zoom_and_applying_it_zooms_the_map() {
            let (mut cx, mut g) = (BIG.cx("square"), gfx());
            let mut s = crate::menus::SettingsScreen::new(&cx);
            s.draw(&mut cx, &mut g, (-500, -500), true);
            // PR7-37 (PT-33): the row is on the Display tab (the first) only
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

    /// REVEAL1 (PR5-54): an indoor zone's roof, its wall set and the boundary's edge dressing over the yard fixture with a
    /// 7 x 6 building (a wall ring at x 24-30, y 24-29, its door at (27, 29)): the zone `zz_den` holds the floor inside and
    /// the door; the roof prop covers the whole building.
    mod reveal_tests {
        use super::*;

        const ROOF: [u8; 3] = [140, 30, 160];
        const PLASTER: [u8; 3] = [230, 200, 150];
        const BRICK: [u8; 3] = [150, 40, 30];
        const DEN: &str = "zz_den";
        const INSIDE: Pos = (27, 27);
        const PROBE: Pos = (27, 26);
        const RING: Pos = (24, 26);

        fn building(x: i64, y: i64) -> Option<char> {
            let ring = (24..=30).contains(&x) && (24..=29).contains(&y);
            let floor = (25..=29).contains(&x) && (25..=28).contains(&y);
            match () {
                _ if (x, y) == (27, 29) => Some('.'),
                _ if floor => Some('.'),
                _ if ring => Some('#'),
                _ => None,
            }
        }

        /// The fixture's map with the building, its indoor zone (`wall_set` zz_plaster) and its roof prop; `roof` false
        /// leaves the roof out.
        fn den(f: Fixture, roof: bool) -> impl FnOnce(&mut Value) {
            move |m: &mut Value| {
                let rows: Vec<String> = (0..f.rows).map(|y| (0..f.cols).map(|x| building(x, y).unwrap_or_else(|| f.tile(x, y))).collect()).collect();
                let zones: Vec<String> = (0..f.rows).map(|y| (0..f.cols).map(|x| if building(x, y) == Some('.') { 'h' } else { '-' }).collect()).collect();
                m["rows"] = json!(rows);
                m["zones"] = json!(zones);
                m["zone_legend"] = json!({"h": {"id": DEN, "kind": "safe", "indoor": true, "wall_set": "zz_plaster"}});
                if roof {
                    m["props"].as_array_mut().expect("props").push(json!({"id": "den_roof", "art": "zz_roof", "x": 24, "y": 24, "w": 7, "h": 6, "layer": "roof", "variant": -1, "flip": false, "roof_of": DEN}));
                }
            }
        }

        fn at(map: &RegionMap, g: &Gfx, cell: Pos) -> [u8; 3] {
            let p = mid(map, cell);
            px(g, p.0, p.1)
        }

        fn roof_of(map: &RegionMap, cx: &Cx) -> f64 {
            map.state(cx)["roofs"][DEN].as_f64().expect("the zone's roof")
        }

        /// The kit's rules the frame reads: the zone under the party, the wall set of the ring, the roof prop's zone.
        #[test]
        fn the_kit_says_which_zone_walls_and_roof_belong_to_the_building() {
            let d = PLAIN.data(den(PLAIN, true));
            let t = d.map("zz_yard").clone();
            assert!(!t.is_empty(), "the map");
            assert_eq!(explore::indoor_zone_at(&t, INSIDE.0, INSIDE.1), DEN);
            assert_eq!(explore::indoor_zone_at(&t, 27, 29), DEN, "the door is the zone's");
            assert_eq!(explore::indoor_zone_at(&t, 27, 31), "", "outdoors");
            assert_eq!(explore::wall_set_at(&t, RING.0, RING.1), "zz_plaster");
            assert_eq!(explore::wall_set_at(&t, 24, 24), "zz_plaster", "a corner borders the zone diagonally");
            assert_eq!(explore::wall_set_at(&t, 9, PLAIN.rows - 8), "", "a wall away from the building");
            let roofs: Vec<Option<String>> = json::aget(&t, "props").iter().map(|p| explore::roof_zone(json::dict(p))).collect();
            assert_eq!(roofs, vec![None, None, Some(DEN.to_string())]);
        }

        /// 1.3: outside, the roof hides the room (art, else the muted block); inside, it fades out over 180 ms and back in,
        /// both views; under reduce motion it snaps.
        #[test]
        fn the_roof_fades_while_the_party_is_inside_and_snaps_under_reduce_motion() {
            for view in views() {
                for with_art in [true, false] {
                    let root = scratch(&format!("roof_{view}_{with_art}"));
                    if with_art {
                        solid(&root, "props/common/zz_roof/v0.png", 7 * 32, 6 * 32, ROOF);
                    }
                    let (mut cx, mut g) = (PLAIN.cx_on(PLAIN.data(den(PLAIN, true)), view), gfx());
                    cx.art = art_of(&root);
                    let mut map = RegionMap::new(&mut cx);
                    let outside = (27, 33);
                    let shown = if with_art { ROOF } else { [crate::props::ROOF_FILL[0], crate::props::ROOF_FILL[1], crate::props::ROOF_FILL[2]] };
                    let t0 = Instant::now();
                    let shot = |map: &mut RegionMap, cx: &mut Cx, g: &mut Gfx, pos: Pos, ms: u64| {
                        cx.gs_mut().position = pos;
                        map.tick_at(cx, t0 + Duration::from_millis(ms));
                        frame(map, cx, g, 1.0);
                    };
                    shot(&mut map, &mut cx, &mut g, outside, 0);
                    assert_eq!(roof_of(&map, &cx), 1.0, "{view} {with_art}: outside");
                    assert_eq!(at(&map, &g, PROBE), shown, "{view} {with_art}: the roof over the room");
                    shot(&mut map, &mut cx, &mut g, INSIDE, 90);
                    assert_eq!(roof_of(&map, &cx), 0.5, "{view} {with_art}: half way after 90 ms");
                    let half = at(&map, &g, PROBE);
                    assert_ne!(half, shown, "{view} {with_art}: the room shows through");
                    shot(&mut map, &mut cx, &mut g, INSIDE, 180);
                    assert_eq!(roof_of(&map, &cx), 0.0, "{view} {with_art}: gone after 180 ms");
                    let room = at(&map, &g, PROBE);
                    assert_ne!(room, shown, "{view} {with_art}: the room");
                    assert_ne!(room, half, "{view} {with_art}: the room, no roof left");
                    shot(&mut map, &mut cx, &mut g, (27, 29), 400);
                    assert_eq!(roof_of(&map, &cx), 0.0, "{view} {with_art}: the door is inside");
                    shot(&mut map, &mut cx, &mut g, outside, 490);
                    assert_eq!(roof_of(&map, &cx), 0.5, "{view} {with_art}: coming back");
                    shot(&mut map, &mut cx, &mut g, outside, 580);
                    assert_eq!(roof_of(&map, &cx), 1.0, "{view} {with_art}: whole again");
                    assert_eq!(at(&map, &g, PROBE), shown, "{view} {with_art}: the roof again");
                    // reduce motion: one frame in, gone; one frame out, back
                    cx.settings.v["reduce_motion"] = json!(true);
                    shot(&mut map, &mut cx, &mut g, INSIDE, 590);
                    assert_eq!(roof_of(&map, &cx), 0.0, "{view} {with_art}: reduce motion snaps out");
                    assert_eq!(at(&map, &g, PROBE), room, "{view} {with_art}: the room at once");
                    shot(&mut map, &mut cx, &mut g, outside, 600);
                    assert_eq!(roof_of(&map, &cx), 1.0, "{view} {with_art}: reduce motion snaps back");
                    let _ = std::fs::remove_dir_all(&root);
                }
            }
        }

        /// 1.3: a party that arrives inside sees no fade (the first frame of a zone starts at its target), and a map without
        /// a roof prop keeps no roof state.
        #[test]
        fn arriving_inside_starts_faded_and_a_map_without_roofs_has_none() {
            let (mut cx, mut g) = (PLAIN.cx_on(PLAIN.data(den(PLAIN, true)), "square"), gfx());
            let mut map = RegionMap::new(&mut cx);
            cx.gs_mut().position = INSIDE;
            map.tick_at(&mut cx, Instant::now());
            frame(&mut map, &mut cx, &mut g, 1.0);
            assert_eq!(roof_of(&map, &cx), 0.0);
            let (mut cx, mut g) = (PLAIN.cx_on(PLAIN.data(den(PLAIN, false)), "square"), gfx());
            let mut map = RegionMap::new(&mut cx);
            frame(&mut map, &mut cx, &mut g, 1.0);
            assert_eq!(map.state(&cx)["roofs"], json!({}));
        }

        /// 1.1: the building's ring walls draw the zone's wall set; other walls the region's; a wall set without art draws
        /// the region's walls too.
        #[test]
        fn an_indoor_zones_walls_draw_its_wall_set_else_todays_walls() {
            for with_set in [true, false] {
                let root = scratch(&format!("walls_{with_set}"));
                solid(&root, "tilesets/common/wall/v0.png", 32, 32, WALL);
                if with_set {
                    solid(&root, "tilesets/zz_plaster/wall/v0.png", 32, 32, PLASTER);
                }
                let (mut cx, mut g) = (PLAIN.cx_on(PLAIN.data(den(PLAIN, true)), "square"), gfx());
                cx.art = art_of(&root);
                cx.settings.v["reduce_motion"] = json!(true);
                let mut map = RegionMap::new(&mut cx);
                cx.gs_mut().position = INSIDE;
                frame(&mut map, &mut cx, &mut g, 1.0);
                let ring = if with_set { PLASTER } else { WALL };
                for cell in [RING, (24, 24), (30, 27), (26, 29)] {
                    assert_eq!(at(&map, &g, cell), ring, "{with_set}: ring wall {cell:?}");
                }
                assert_eq!(at(&map, &g, (9, PLAIN.rows - 8)), WALL, "{with_set}: a wall away from the building");
                let _ = std::fs::remove_dir_all(&root);
            }
        }

        /// A map smaller than the screen, a map `boundary` `zz_brick` and a location boundary set by `loc`.
        fn small(loc: &'static str, view: &str) -> (Cx, Fixture) {
            let f = Fixture { cols: 24, rows: 14, boundary: loc, ..PLAIN };
            let data = f.data(|m| {
                m["boundary"] = json!("zz_brick");
                m["anchors"] = json!([{"id": "spawn", "x": 12, "y": 10}, {"id": "job", "x": 3, "y": 3}, {"id": "far", "x": 20, "y": 3}]);
                m["props"] = json!([]);
            });
            (f.cx_on(data, view), f)
        }

        /// 2.2: past a small map the boundary's dressing repeats on every cell and never on the map; the map's own
        /// boundary serves a location without one; a location's boundary wins; without dressing art the border keeps the
        /// wall tile.
        #[test]
        fn a_small_maps_margin_repeats_the_boundary_dressing_and_falls_back_to_the_wall() {
            for view in views() {
                let root = scratch(&format!("dressing_{view}"));
                solid(&root, "tilesets/common/wall/v0.png", 32, 32, WALL);
                solid(&root, "tilesets/common/wall_face/v0.png", 128, 48, FACE);
                let probe = |loc: &'static str| {
                    let (mut cx, f) = small(loc, view);
                    let mut g = gfx();
                    cx.art = art_of(&root);
                    let mut map = RegionMap::new(&mut cx);
                    frame(&mut map, &mut cx, &mut g, 1.0);
                    // clear of the legend's fold button (top right) and the ticker (bottom)
                    let inner = Rect::new(24, HUD_H + 24, W - 120, H - HUD_H - 72);
                    let cells: Vec<(Pos, [u8; 3])> = (-8..f.rows + 8)
                        .flat_map(|y| (-16..f.cols + 16).map(move |x| (x, y)))
                        .filter(|c| {
                            let p = mid(&map, *c);
                            p.0 >= inner.x && p.1 >= inner.y && p.0 < inner.x + inner.w && p.1 < inner.y + inner.h
                        })
                        .map(|c| (c, at(&map, &g, c)))
                        .collect();
                    (cells, f)
                };
                let past = |f: &Fixture, c: Pos| c.0 < 0 || c.1 < 0 || c.0 >= f.cols || c.1 >= f.rows;
                // no dressing art: the margin is the wall tile (square) and nothing is brick
                let (cells, f) = probe("");
                let margin: Vec<&(Pos, [u8; 3])> = cells.iter().filter(|(c, _)| past(&f, *c)).collect();
                assert!(margin.len() > 100, "{view}: the margin shows ({} cells)", margin.len());
                assert!(cells.iter().all(|(_, px)| *px != BRICK), "{view}: no art, no dressing");
                if view == "square" {
                    assert!(margin.iter().all(|(_, px)| *px == WALL), "{view}: the wall tile past the map");
                }
                solid(&root, "props/common/boundary_zz_brick/v0.png", 32, 32, BRICK);
                let (cells, f) = probe("");
                for (c, px) in &cells {
                    assert_eq!(*px == BRICK, past(&f, *c), "{view}: {c:?} brick only past the map");
                }
                // the location's own boundary wins (it has no dressing): no brick
                let (cells, _) = probe("zz_other");
                assert!(cells.iter().all(|(_, px)| *px != BRICK), "{view}: the location's boundary first");
                let _ = std::fs::remove_dir_all(&root);
            }
        }

        /// The proof shots (only with `AEGIS_REVIEW_DIR` set): the building with the party outside (the roof drawn) and
        /// inside (the roof faded, the plaster ring and the floor showing), square view, placeholder roof block.
        #[test]
        fn review_shots_of_the_roof_outside_and_inside() {
            let Ok(out) = std::env::var("AEGIS_REVIEW_DIR") else { return };
            let root = scratch("review");
            solid(&root, "tilesets/common/wall/v0.png", 32, 32, WALL);
            solid(&root, "tilesets/zz_plaster/wall/v0.png", 32, 32, PLASTER);
            for (name, pos) in [("reveal_outside", (27, 33)), ("reveal_inside", INSIDE)] {
                let (mut cx, mut g) = (LAYER.cx_on(LAYER.data(den(LAYER, true)), "square"), gfx());
                cx.art = art_of(&root);
                cx.settings.v["reduce_motion"] = json!(true);
                let mut map = RegionMap::new(&mut cx);
                cx.gs_mut().position = pos;
                frame(&mut map, &mut cx, &mut g, 1.0);
                let file = std::fs::File::create(Path::new(&out).join(format!("{name}.png"))).unwrap();
                let mut enc = png::Encoder::new(std::io::BufWriter::new(file), g.canvas.w as u32, g.canvas.h as u32);
                enc.set_color(png::ColorType::Rgba);
                enc.set_depth(png::BitDepth::Eight);
                enc.write_header().unwrap().write_image_data(&g.canvas.px).unwrap();
            }
            // the boundary's dressing round a small map, both views
            solid(&root, "props/common/boundary_zz_brick/v0.png", 32, 32, BRICK);
            for view in views() {
                let (mut cx, _) = small("", view);
                let mut g = gfx();
                cx.art = art_of(&root);
                let mut map = RegionMap::new(&mut cx);
                frame(&mut map, &mut cx, &mut g, 1.0);
                let file = std::fs::File::create(Path::new(&out).join(format!("reveal_boundary_{view}.png"))).unwrap();
                let mut enc = png::Encoder::new(std::io::BufWriter::new(file), g.canvas.w as u32, g.canvas.h as u32);
                enc.set_color(png::ColorType::Rgba);
                enc.set_depth(png::BitDepth::Eight);
                enc.write_header().unwrap().write_image_data(&g.canvas.px).unwrap();
            }
            let _ = std::fs::remove_dir_all(&root);
        }
    }

    /// REGIONVIEW2 (PR5-59): wandering water-clamped grounds, patchy/under layers, free-standing `sort` prop floors,
    /// 2-tile wall faces with pale caps + SE shadows + inner water rims, thin prop base contact lines (no cast oval
    /// shadows), `into_wall` masonry pull, opt-in `dim_ambience` + light pools, and `FB-271(b)` NPC badges under roofs.
    mod regionview2_tests {
        use super::*;

        fn rgba_png(root: &Path, rel: &str, w: i32, h: i32, mut f: impl FnMut(i32, i32) -> [u8; 4]) {
            let file = root.join(rel);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            let mut buf = Vec::with_capacity((w * h * 4) as usize);
            for y in 0..h {
                for x in 0..w {
                    buf.extend_from_slice(&f(x, y));
                }
            }
            let out = std::fs::File::create(&file).unwrap();
            let mut enc = png::Encoder::new(std::io::BufWriter::new(out), w as u32, h as u32);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header().unwrap().write_image_data(&buf).unwrap();
        }

        fn patch_loc(data: &mut emberwake_world::Data, world_id: &str, loc_id: &str, f: impl FnOnce(&mut Dict)) {
            if let Some(w) = data.catalogs.worlds.get_mut(world_id) {
                if let Some(locs) = w.own.get_mut("locations") {
                    if let Some(loc) = locs.iter_mut().find(|l| s(l, "id") == loc_id || s(l, "_qid") == loc_id) {
                        f(loc);
                    }
                }
            }
        }

        #[test]
        fn unflagged_maps_render_byte_identically_when_new_flags_omitted() {
            for view in views() {
                let d_omitted = PLAIN.data(|_| {});
                let d_explicit = PLAIN.data(|m| {
                    m["dim_ambience"] = json!(false);
                    m["wall_face_tiles"] = json!(0);
                });
                let (mut cx_a, mut g_a) = (PLAIN.cx_on(d_omitted, view), gfx());
                let (mut cx_b, mut g_b) = (PLAIN.cx_on(d_explicit, view), gfx());
                let mut m_a = RegionMap::new(&mut cx_a);
                let mut m_b = RegionMap::new(&mut cx_b);
                frame(&mut m_a, &mut cx_a, &mut g_a, 1.0);
                frame(&mut m_b, &mut cx_b, &mut g_b, 1.0);
                assert_eq!(g_a.canvas.px, g_b.canvas.px, "{view}: unflagged maps byte-identical");
            }
        }

        #[test]
        fn ground_water_clamp_stone_rim_and_patchy_under_compose_through_region_map() {
            let mut data = PLAIN.data(|m| {
                let mut g_rows: Vec<String> = Vec::new();
                for y in 0..PLAIN.rows {
                    let mut row = String::new();
                    for x in 0..PLAIN.cols {
                        row.push(if (15..=21).contains(&x) && (12..=18).contains(&y) { 'w' } else { 'p' });
                    }
                    g_rows.push(row);
                }
                let mut r_rows = m["rows"].as_array().unwrap().clone();
                for y in 12..=18 {
                    let mut chars: Vec<char> = r_rows[y as usize].as_str().unwrap().chars().collect();
                    for x in 15..=21 {
                        chars[x as usize] = '~';
                    }
                    r_rows[y as usize] = Value::String(chars.into_iter().collect());
                }
                m["rows"] = Value::Array(r_rows);
                m["ground"] = json!(g_rows);
                m["ground_legend"] = json!({"p": "zz_patch", "w": "zz_water"});
            });
            data.edit_cat(|cat| {
                cat.grounds.insert(
                    "zz_under".into(),
                    emberwake_data::GroundDef::from_obj(
                        json!({"id": "zz_under", "name": "Under", "order": 1, "kind": "land", "colour": "#305070", "wear": 0.0, "tone": 0.0})
                            .as_object()
                            .unwrap(),
                    ),
                );
                cat.grounds.insert(
                    "zz_patch".into(),
                    emberwake_data::GroundDef::from_obj(
                        json!({"id": "zz_patch", "name": "Patch", "order": 5, "kind": "land", "colour": "#a03020", "wear": 0.0, "tone": 0.0, "patchy": 0.5, "under": "zz_under"})
                            .as_object()
                            .unwrap(),
                    ),
                );
                cat.grounds.insert(
                    "zz_water".into(),
                    emberwake_data::GroundDef::from_obj(
                        json!({"id": "zz_water", "name": "Water", "order": 10, "kind": "water", "colour": "#143248", "wear": 0.0, "tone": 0.0})
                            .as_object()
                            .unwrap(),
                    ),
                );
            });
            let (mut cx, mut g) = (PLAIN.cx_on(data, "square"), gfx());
            cx.gs_mut().position = (12, 15);
            let mut map = RegionMap::new(&mut cx);
            frame(&mut map, &mut cx, &mut g, 1.0);
            // Water center stays exact clamped water colour; water lip next to land shows stone rim (96, 98, 108).
            let wc = mid(&map, (18, 15));
            assert_eq!(px(&g, wc.0, wc.1), [0x14, 0x32, 0x48]);
            let water_cell = map.geo.cell(15, 15);
            let lip_px = px(&g, water_cell.x, water_cell.y + water_cell.h / 2);
            assert_eq!(lip_px, [96, 98, 108], "stone rim on water border next to land");
            // Patchy land over unseen `zz_under` renders both patchy top and under colours.
            let (mut saw_top, mut saw_under) = (false, false);
            for cy in 8..16 {
                for cx_i in 6..13 {
                    let r = map.geo.cell(cx_i, cy);
                    for py in r.y..(r.y + r.h) {
                        for px_i in r.x..(r.x + r.w) {
                            let c = px(&g, px_i, py);
                            if c == [0xa0, 0x30, 0x20] {
                                saw_top = true;
                            }
                            if c == [0x30, 0x50, 0x70] {
                                saw_under = true;
                            }
                        }
                    }
                }
            }
            assert!(saw_top && saw_under, "patchy top={saw_top} and under={saw_under} both appear");
        }

        #[test]
        fn freestanding_sort_prop_renders_floor_under_wall_cells_without_changing_collision() {
            let floor = [72, 130, 88];
            for view in views() {
                let root = scratch(&format!("freestand_{view}"));
                solid(&root, "tilesets/common/floor/v0.png", 32, 32, floor);
                solid(&root, "tilesets/common/wall/v0.png", 32, 32, WALL);
                solid(&root, "tilesets/common/wall_face/v0.png", 32, 16, FACE);
                // Transparent cutout prop so underlying floor shines through at its edges.
                rgba_png(&root, "props/common/zz_brazier_tile/v0.png", 32, 32, |x, y| {
                    if (12..20).contains(&x) && (10..22).contains(&y) {
                        [210, 90, 40, 255]
                    } else {
                        [0, 0, 0, 0]
                    }
                });
                let data = PLAIN.data(|m| {
                    let mut rows: Vec<String> = m["rows"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
                    let set = |rows: &mut Vec<String>, x: usize, y: usize, ch: char| {
                        let mut c: Vec<char> = rows[y].chars().collect();
                        c[x] = ch;
                        rows[y] = c.into_iter().collect();
                    };
                    set(&mut rows, 18, 16, '#');
                    // Buried 3x3 `#` pocket whose center (25, 16) is completely enclosed by `#`
                    for dy in 15..=17 {
                        for dx in 24..=26 {
                            set(&mut rows, dx, dy, '#');
                        }
                    }
                    m["rows"] = json!(rows);
                    m["props"] = json!([
                        {"id": "p_free", "art": "zz_brazier_tile", "x": 18, "y": 16, "w": 1, "h": 1, "layer": "sort", "variant": 0, "flip": false},
                        {"id": "p_border", "art": "zz_brazier_tile", "x": 0, "y": 16, "w": 1, "h": 1, "layer": "sort", "variant": 0, "flip": false},
                        {"id": "p_buried", "art": "zz_brazier_tile", "x": 25, "y": 16, "w": 1, "h": 1, "layer": "sort", "variant": 0, "flip": false}
                    ]);
                });
                let (mut cx, mut g) = (PLAIN.cx_on(data.clone(), view), gfx());
                cx.art = art_of(&root);
                cx.gs_mut().position = (19, 16);
                let mut map = RegionMap::new(&mut cx);
                frame(&mut map, &mut cx, &mut g, 1.0);
                // Collision in `rows` remains blocked (`#`), while visual corner pixel of (18, 16) shows floor.
                let t = explore::template_of(&data, explore::location(&data, cx.gs()));
                assert_eq!(explore::tile(t, 18, 16), Some('#'));
                let cell = map.geo.cell(18, 16);
                assert_eq!(px(&g, cell.x + 1, cell.y + 1), floor, "{view}: free-standing sort prop gets floor under #");
                // Buried center (25, 16) has no cardinal open neighbor -> keeps wall!
                let buried = map.geo.cell(25, 16);
                assert_eq!(px(&g, buried.x + 1, buried.y + 1), WALL, "{view}: enclosed pocket stays wall");
                let _ = std::fs::remove_dir_all(&root);
            }
        }

        #[test]
        fn wall_face_tiles_depth_pale_cap_shadows_water_rim_and_wall_set_work_across_views() {
            let floor = [150, 160, 150];
            let face_base = [160, 160, 160];
            let face_stone = [120, 140, 180];
            for view in views() {
                let root = scratch(&format!("depth_{view}"));
                solid(&root, "tilesets/common/floor/v0.png", 32, 32, floor);
                solid(&root, "tilesets/common/wall/v0.png", 32, 32, WALL);
                solid(&root, "tilesets/common/wall_face/v0.png", 32, 16, face_base);
                solid(&root, "tilesets/common/water/v0.png", 32, 32, [40, 110, 160]);
                solid(&root, "tilesets/zz_stone/wall/v0.png", 32, 32, [70, 75, 90]);
                solid(&root, "tilesets/zz_stone/wall_face/v0.png", 32, 16, face_stone);
                // Portcullis with grey masonry pixels (max-min < 40, max < 150)
                solid(&root, "props/common/portcullis/v0.png", 72, 84, [60, 60, 60]);
                let mut data = PLAIN.data(|m| {
                    let mut rows: Vec<String> = vec![".".repeat(PLAIN.cols as usize); PLAIN.rows as usize];
                    for y in 0..PLAIN.rows as usize {
                        let mut c: Vec<char> = rows[y].chars().collect();
                        c[0] = '#';
                        c[PLAIN.cols as usize - 1] = '#';
                        rows[y] = c.into_iter().collect();
                    }
                    rows[0] = "#".repeat(PLAIN.cols as usize);
                    rows[PLAIN.rows as usize - 1] = "#".repeat(PLAIN.cols as usize);
                    // 3-row thick interior wall spanning x=4..18, y=8..=10, open at y=11, plus a vertical east-shadow wall at x=18,y=11..=13
                    let set = |rows: &mut Vec<String>, x: usize, y: usize, ch: char| {
                        let mut c: Vec<char> = rows[y].chars().collect();
                        c[x] = ch;
                        rows[y] = c.into_iter().collect();
                    };
                    for y in 8..=10 {
                        for x in 4..18 {
                            set(&mut rows, x, y, '#');
                        }
                    }
                    for y in 11..=13 {
                        set(&mut rows, 18, y, '#');
                    }
                    for y in 14..=16 {
                        for x in 10..=14 {
                            set(&mut rows, x, y, '~');
                        }
                    }
                    let mut zones = vec!["-".repeat(PLAIN.cols as usize); PLAIN.rows as usize];
                    set(&mut zones, 12, 11, 's');
                    m["rows"] = json!(rows);
                    m["zones"] = json!(zones);
                    m["zone_legend"] = json!({"s": {"id": "zz_hall", "kind": "safe", "indoor": true, "wall_set": "zz_stone"}});
                    m["wall_face_tiles"] = json!(2);
                    m["props"] = json!([
                        {"id": "p_gate", "art": "portcullis", "x": 6, "y": 9, "w": 2, "h": 2, "layer": "sort", "variant": 0, "flip": false}
                    ]);
                });
                let (mut cx, mut g) = (PLAIN.cx_on(data.clone(), view), gfx());
                cx.art = art_of(&root);
                cx.gs_mut().position = (14, 12);
                let mut map = RegionMap::new(&mut cx);
                frame(&mut map, &mut cx, &mut g, 1.0);
                // Upper face row (9) has base face color and pale top cap line (132, 130, 140); lower face row (10) is darker (* 0.72).
                let upper = mid(&map, (9, 9));
                let lower = mid(&map, (9, 10));
                assert_eq!(px(&g, upper.0, upper.1), face_base, "{view}: depth=2 upper face");
                assert_eq!(px(&g, lower.0, lower.1), [115, 115, 115], "{view}: depth=1 lower face darkened 0.72");
                let r9 = map.geo.cell(9, 9);
                let top_y = if view == "tilted" { r9.y - mapview::lift_of(r9.h) } else { r9.y };
                assert_eq!(px(&g, r9.x + r9.w / 2, top_y), [132, 130, 140], "{view}: pale top cap line");
                // Indoor zone `wall_set` `zz_stone` renders on (12, 10) adjacent to zone 's' at (12, 11).
                let zone_face = mid(&map, (12, 10));
                assert_eq!(px(&g, zone_face.0, zone_face.1), [86, 101, 130], "{view}: zone wall_set darkened face");
                // South wall floor shadow on top of (9, 11) and east shadow on left of (19, 12) are darker than clean open floor (9, 12).
                let open_floor = mid(&map, (9, 12));
                assert_eq!(px(&g, open_floor.0, open_floor.1), floor);
                let r11 = map.geo.cell(9, 11);
                assert!(px(&g, r11.x + r11.w / 2, r11.y)[0] < floor[0], "{view}: south wall floor shadow");
                let r19 = map.geo.cell(19, 12);
                assert!(px(&g, r19.x, r19.y + r19.h / 2)[0] < floor[0], "{view}: east wall floor shadow");
                // Dark inner N/W water rim on (10, 14) is darker than open water midpoint (12, 15).
                let w_mid = mid(&map, (12, 15));
                let w_nw = map.geo.cell(10, 14);
                assert!(px(&g, w_nw.x + 1, w_nw.y + 1)[2] < px(&g, w_mid.0, w_mid.1)[2], "{view}: dark inner water rim");
                // Portcullis at (6, 9..10) keeps walls intact behind and pulls dark grey masonry (60) upward toward wall face tone.
                let gate = mid(&map, (6, 9));
                assert!(px(&g, gate.0, gate.1)[0] > 75, "{view}: into_wall pulls masonry toward lighter face");
                let _ = &mut data;
                let _ = std::fs::remove_dir_all(&root);
            }
        }

        #[test]
        fn standing_prop_draws_only_thin_base_contact_line_and_no_cast_oval_shadow() {
            let floor = [192, 192, 192];
            let root = scratch("contact_only");
            solid(&root, "tilesets/common/floor/v0.png", 32, 32, floor);
            solid(&root, "tilesets/common/wall/v0.png", 32, 32, WALL);
            // Opaque narrow pedestal centered at x=10..22, y=6..24, transparent outer bottom corners and rows below y=24
            rgba_png(&root, "props/common/zz_pedestal/v0.png", 32, 32, |x, y| {
                if (10..22).contains(&x) && (6..24).contains(&y) {
                    [210, 160, 80, 255]
                } else {
                    [0, 0, 0, 0]
                }
            });
            let data = PLAIN.data(|m| {
                m["props"] = json!([
                    {"id": "p_ped", "art": "zz_pedestal", "x": 16, "y": 14, "w": 1, "h": 1, "layer": "sort", "variant": 0, "flip": false}
                ]);
            });
            let (mut cx, mut g) = (PLAIN.cx_on(data, "square"), gfx());
            cx.art = art_of(&root);
            cx.gs_mut().position = (10, 10);
            let mut map = RegionMap::new(&mut cx);
            frame(&mut map, &mut cx, &mut g, 1.0);
            let cell = map.geo.cell(16, 14);
            // opaque_bounds bottom in 16x16 cell is y=12 (24/2); contact line draws at y=12 within center x=6..10
            let contact_px = px(&g, cell.x + cell.w / 2, cell.y + 12);
            assert!(contact_px[0] < floor[0], "thin contact line at prop feet: {contact_px:?}");
            // Outer edges left/right of pedestal (where an oval shadow would extend) and below y=13 remain exact floor
            assert_eq!(px(&g, cell.x + 1, cell.y + 12), floor, "no wide oval shadow to left");
            assert_eq!(px(&g, cell.x + cell.w - 2, cell.y + 12), floor, "no wide oval shadow to right");
            assert_eq!(px(&g, cell.x + cell.w / 2, cell.y + 15), floor, "no cast oval shadow extending below feet");
            let _ = std::fs::remove_dir_all(&root);
        }

        #[test]
        fn dim_ambience_defaults_off_and_illuminates_light_pools_when_enabled() {
            let floor = [140, 150, 160];
            let root = scratch("dim_pools");
            solid(&root, "tilesets/common/floor/v0.png", 32, 32, floor);
            solid(&root, "tilesets/common/wall/v0.png", 32, 32, WALL);
            solid(&root, "props/common/brazier/v0.png", 32, 32, [220, 120, 40]);
            let make = |dim: bool| {
                PLAIN.data(|m| {
                    if dim {
                        m["dim_ambience"] = json!(true);
                    }
                    m["props"] = json!([
                        {"id": "p_fire", "art": "brazier", "x": 14, "y": 14, "w": 1, "h": 1, "layer": "sort", "variant": 0, "flip": false}
                    ]);
                })
            };
            let (mut cx_day, mut g_day) = (PLAIN.cx_on(make(false), "square"), gfx());
            cx_day.art = art_of(&root);
            cx_day.gs_mut().position = (18, 14);
            let mut m_day = RegionMap::new(&mut cx_day);
            frame(&mut m_day, &mut cx_day, &mut g_day, 1.0);
            let far_day = mid(&m_day, (26, 14));
            assert_eq!(px(&g_day, far_day.0, far_day.1), floor, "daylight map unaffected when dim_ambience omitted");

            let (mut cx_dim, mut g_dim) = (PLAIN.cx_on(make(true), "square"), gfx());
            cx_dim.art = art_of(&root);
            cx_dim.gs_mut().position = (18, 14);
            let mut m_dim = RegionMap::new(&mut cx_dim);
            frame(&mut m_dim, &mut cx_dim, &mut g_dim, 1.0);
            let far_dim_pt = mid(&m_dim, (26, 14));
            let far_dim = px(&g_dim, far_dim_pt.0, far_dim_pt.1);
            let near_pt = mid(&m_dim, (15, 14));
            let near_dim = px(&g_dim, near_pt.0, near_pt.1);
            assert!(far_dim[0] < floor[0] - 20, "far floor darkened under dim_ambience: {far_dim:?}");
            assert!(near_dim[0] > far_dim[0] + 35, "brazier light pool illuminates nearby floor: near={near_dim:?} far={far_dim:?}");
            assert!(near_dim[0] > near_dim[2], "warm amber light tint raises red above blue: {near_dim:?}");
            let _ = std::fs::remove_dir_all(&root);
        }

        #[test]
        fn fb271b_hides_npc_badge_under_opaque_roof_and_shows_when_faded() {
            let badge_col = [255, 20, 220];
            let root = scratch("fb271b_badge");
            solid(&root, "tilesets/common/floor/v0.png", 32, 32, [60, 70, 60]);
            solid(&root, "tilesets/common/wall/v0.png", 32, 32, WALL);
            solid(&root, "props/common/zz_roof/v0.png", 7 * 32, 6 * 32, [30, 30, 45]);
            solid(&root, "ui/services/shop.png", 16, 16, badge_col);
            let mut data = PLAIN.data(|m| {
                let mut rows: Vec<String> = (0..PLAIN.rows)
                    .map(|y| (0..PLAIN.cols).map(|x| {
                        let ring = (24..=30).contains(&x) && (24..=29).contains(&y);
                        let floor = (25..=29).contains(&x) && (25..=28).contains(&y);
                        if (x, y) == (27, 29) || floor { '.' } else if ring { '#' } else { PLAIN.tile(x, y) }
                    }).collect())
                    .collect();
                m["rows"] = json!(rows);
                m["zones"] = json!((0..PLAIN.rows).map(|y| (0..PLAIN.cols).map(|x| {
                    let floor = (25..=29).contains(&x) && (25..=28).contains(&y);
                    if (x, y) == (27, 29) || floor { 'h' } else { '-' }
                }).collect::<String>()).collect::<Vec<_>>());
                m["zone_legend"] = json!({"h": {"id": "zz_den", "kind": "safe", "indoor": true, "wall_set": "zz_plaster"}});
                m["props"].as_array_mut().unwrap().push(json!({
                    "id": "den_roof", "art": "zz_roof", "x": 24, "y": 24, "w": 7, "h": 6,
                    "layer": "roof", "variant": 0, "flip": false, "roof_of": "zz_den"
                }));
                let anchors = m["anchors"].as_array_mut().unwrap();
                anchors.push(json!({"id": "in_npc", "x": 26, "y": 26}));
                anchors.push(json!({"id": "out_npc", "x": 22, "y": 31}));
                let _ = &mut rows;
            });
            let (mut cx, mut g) = (PLAIN.cx_on(data.clone(), "square"), gfx());
            let world_id = cx.gs().world_id.clone();
            let loc_id = cx.gs().location_id.clone();
            if let Some(w) = data.catalogs.worlds.get_mut(&world_id) {
                w.own.entry("characters".into()).or_default().push(
                    json!({"id": "zz_keeper", "_world": world_id, "_qid": format!("{world_id}:zz_keeper"), "name": "Keeper"})
                        .as_object()
                        .unwrap()
                        .clone(),
                );
                w.own.entry("shops".into()).or_default().push(
                    json!({"id": "zz_shop", "_world": world_id, "_qid": format!("{world_id}:zz_shop"), "keeper": "zz_keeper"})
                        .as_object()
                        .unwrap()
                        .clone(),
                );
            }
            patch_loc(&mut data, &world_id, &loc_id, |loc| {
                loc.insert(
                    "markers".into(),
                    json!([
                        {"id": "m_in", "kind": "npc", "character": "zz_keeper", "anchor": "in_npc", "state": "present"},
                        {"id": "m_out", "kind": "npc", "character": "zz_keeper", "anchor": "out_npc", "state": "present"}
                    ]),
                );
            });
            cx.app.data = std::sync::Arc::new(data);
            cx.art = art_of(&root);
            cx.settings.v["reduce_motion"] = json!(true);
            let mut map = RegionMap::new(&mut cx);

            let has_badge_near = |map: &RegionMap, g: &Gfx, tile: Pos| {
                let r = map.geo.cell(tile.0 as i32, tile.1 as i32);
                (r.y - 12..r.y + r.h).any(|py| (r.x - 4..r.x + r.w + 16).any(|px_i| px(g, px_i, py) == badge_col))
            };
            // Party outside: roof is opaque -> indoor NPC badge suppressed, outdoor NPC badge visible
            cx.gs_mut().position = (27, 33);
            frame(&mut map, &mut cx, &mut g, 1.0);
            assert!(!has_badge_near(&map, &g, (26, 26)), "indoor NPC badge suppressed under opaque roof");
            assert!(has_badge_near(&map, &g, (22, 31)), "outdoor NPC badge stays visible");
            // Party inside: roof faded -> indoor NPC badge reappears
            cx.gs_mut().position = (27, 27);
            frame(&mut map, &mut cx, &mut g, 1.0);
            assert!(has_badge_near(&map, &g, (26, 26)), "indoor NPC badge visible when roof is faded");
            let _ = std::fs::remove_dir_all(&root);
        }
    }
}

/// PR7-11 (FZ4, PK3 3.1): the Retire / Last Stand prompt on a scratch Martyr (a test-only recruit template in the
/// played world; no shipped id is written).
#[cfg(test)]
mod fuse_prompt_tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx};
    use emberwake_world::roster;

    const TEMPLATE: &str = "zz_martyr";
    const ART: &str = "portraits/common/zz_martyr.png";

    /// A debug game whose first active member is a Living Martyr of `TEMPLATE` (a fuse of max 1, one deployment
    /// made, so it is spent; its last passive the catalog's first passive), a second active member beside it, the
    /// roster open: (cx, member, last passive, a fight of the world's first encounter).
    fn martyr_cx() -> (Cx, String, String, Dict) {
        let mut cx = game_cx();
        let world = cx.gs().world_id.clone();
        let last = cx.data.cat.passives.keys().next().cloned().expect("a catalog passive");
        let encounter = cx.data.catalogs.own_entries(&world, "encounters").first().map(|e| s(e, "id")).expect("an encounter");
        let row = json!({"id": TEMPLATE, "name": "Martyr", "class_id": "", "base_units": [], "names": ["Zed"], "art": ART,
                         "fuse": {"per_deploy": 1, "stasis_days": 7, "max": 1, "tiers": [], "last_passive": last},
                         "_world": world, "_qid": format!("{world}:{TEMPLATE}")});
        let data = std::sync::Arc::get_mut(&mut cx.app.data).expect("this test's own data");
        data.catalogs.worlds.get_mut(&world).expect("the world").own.entry("recruits".into()).or_default().push(json::dict(&row).clone());
        cx.gs_mut().party.roster.manifest.open_flag = String::new();
        let member = cx.gs().party.active_members().first().map(|m| m.id.clone()).expect("an active member");
        // a second active member (a copy of the first under a test id), so a Retire leaves someone to fight
        let party = &mut cx.gs_mut().party;
        let mut extra = party.member(&member).unwrap().clone();
        extra.id = "zz_second".into();
        party.members.push(extra);
        party.active_ids = vec![member.clone(), "zz_second".into()];
        party.max_active = party.max_active.max(2);
        make_martyr(&mut cx, &member);
        let fight = json::dict(&json!({"encounter": encounter, "source": "test"})).clone();
        (cx, member, last, fight)
    }

    /// Member `id` becomes a Martyr of `TEMPLATE` with its fuse spent (one deployment of max 1).
    fn make_martyr(cx: &mut Cx, id: &str) {
        let world = cx.gs().world_id.clone();
        let time = cx.gs().time;
        let m = cx.gs_mut().party.member_mut(id).expect("a member");
        m.template = format!("{world}:{TEMPLATE}");
        m.fuse = Some(emberwake_world::state::MartyrFuse { deploys: 1, since: time, ..Default::default() });
    }

    /// Sets the `party_min` of `fight`'s encounter (the played world's own row) to `n`.
    fn set_party_min(cx: &mut Cx, fight: &Dict, n: usize) {
        let world = cx.gs().world_id.clone();
        let id = s(fight, "encounter");
        let data = std::sync::Arc::get_mut(&mut cx.app.data).expect("this test's own data");
        let rows = data.catalogs.worlds.get_mut(&world).unwrap().own.get_mut("encounters").unwrap();
        rows.iter_mut().find(|e| s(e, "id") == id).expect("the encounter").insert("party_min".into(), json!(n));
    }

    fn actives(cx: &Cx) -> usize {
        cx.gs().party.active_members().len()
    }

    /// The screen the last navigation pushed (its id; "" for none).
    fn pushed(cx: &Cx) -> String {
        cx.nav.iter().rev().find_map(|n| match n {
            Nav::Push(s) => Some(s.id().to_string()),
            _ => None,
        }).unwrap_or_default()
    }

    /// Opens the prompt as a fight start does and draws it once: (the prompt, the texts drawn).
    fn opened(cx: &mut Cx, fight: &Dict) -> (Box<dyn Screen<GameCx>>, Vec<String>) {
        try_fight(cx, fight, "map").expect("the start opens the prompt");
        let Some(Nav::Push(mut prompt)) = cx.nav.pop() else { panic!("the prompt is pushed") };
        cx.nav.clear();
        let mut g = gfx();
        let texts = frame(prompt.as_mut(), cx, &mut g, 1.0).into_iter().map(|t| t.text).collect();
        (prompt, texts)
    }

    #[test]
    fn a_spent_fuse_opens_the_prompt_with_the_martyr_portrait_and_both_choices() {
        let (mut cx, member, _, fight) = martyr_cx();
        let why = roster::fuse_refusal(&cx.data, cx.gs());
        assert!(!why.is_empty(), "the scratch Martyr's fuse is spent");
        // the kit's battle start refuses with the same reason (a headless runner), the screen asks instead
        let d = cx.data.clone();
        assert_eq!(emberwake_world::battle::begin_fight(&d, &mut cx.gs().clone(), &s(&fight, "encounter")).err(), Some(why.clone()));
        let (prompt, texts) = opened(&mut cx, &fight);
        assert_eq!(prompt.id(), "fuse_prompt");
        let state = prompt.state(&cx);
        assert_eq!((state["member"].as_str(), state["portrait"].as_str(), state["refusal"].as_str()), (Some(member.as_str()), Some(ART), Some(why.as_str())));
        let ids: Vec<String> = prompt.widgets().iter().map(|w| s(json::dict(w), "id")).collect();
        for id in ["fuse.portrait", "fuse.retire", "fuse.last_stand"] {
            assert!(ids.iter().any(|w| w == id), "{id} in {ids:?}");
        }
        let first_word = why.split_whitespace().next().unwrap();
        assert!(texts.iter().any(|t| t.contains(first_word)) && texts.iter().any(|t| t == "Retire") && texts.iter().any(|t| t == "Last Stand"), "{texts:?}");
        // the portrait is the template's art under the member's name
        let m = cx.gs().party.member(&member).unwrap().clone();
        let (who, _) = crate::hub::member_portrait(&cx.data, cx.gs(), &m);
        assert_eq!((who.file.as_str(), who.key.clone()), (ART, format!("member:{member}")));
    }

    #[test]
    fn last_stand_deploys_with_the_last_passive_and_the_fight_goes_on() {
        let (mut cx, member, last, fight) = martyr_cx();
        let (mut prompt, _) = opened(&mut cx, &fight);
        prompt.click_id(&mut cx, "fuse.last_stand").expect("the button");
        let m = cx.gs().party.member(&member).expect("still a member until the settle");
        assert!(m.fuse.as_ref().is_some_and(|f| f.last_stand));
        assert!(roster::fuse_passives(&cx.data, m).contains(&last), "the last passive applies");
        assert!(roster::fuse_refusal(&cx.data, cx.gs()).is_empty());
        assert!(matches!(cx.nav.first(), Some(Nav::Pop)), "the prompt closes");
        assert_ne!(pushed(&cx), "fuse_prompt", "the fight starts, no second prompt");
    }

    #[test]
    fn retire_departs_the_martyr_and_a_closed_roster_keeps_the_prompt() {
        let (mut cx, member, _, fight) = martyr_cx();
        let (mut prompt, _) = opened(&mut cx, &fight);
        // a closed roster refuses Retire: the reason shows, the prompt stays
        let open = cx.gs().party.roster.manifest.open_flag.clone();
        cx.gs_mut().party.roster.manifest.open_flag = "zz_closed".into();
        prompt.click_id(&mut cx, "fuse.retire").expect("the button");
        assert_eq!(prompt.state(&cx)["message"].as_str(), Some(roster::CLOSED_TEXT));
        assert!(cx.nav.is_empty() && cx.gs().party.member(&member).is_some());
        cx.gs_mut().party.roster.manifest.open_flag = open;
        prompt.click_id(&mut cx, "fuse.retire").expect("the button");
        assert!(cx.gs().party.member(&member).is_none());
        assert_eq!(cx.gs().party.departed_of(&member).map(|r| r.life.as_str()), Some(roster::RETIRED));
        assert!(matches!(cx.nav.first(), Some(Nav::Pop)));
    }

    #[test]
    fn a_fight_refused_anyway_asks_nothing() {
        // the start's other refusals come first: party_min above the active members refuses before any choice
        let (mut cx, member, _, fight) = martyr_cx();
        let n = actives(&cx);
        set_party_min(&mut cx, &fight, n + 1);
        let why = emberwake_world::battle::start_refusal(&cx.data, cx.gs(), &s(&fight, "encounter"));
        assert!(!why.is_empty());
        assert_eq!(try_fight(&mut cx, &fight, "map").err(), Some(why));
        assert!(cx.nav.is_empty(), "no prompt for a refused fight");
        assert!(cx.gs().party.member(&member).is_some_and(|m| m.fuse.as_ref().is_some_and(|f| !f.last_stand)));
    }

    #[test]
    fn retire_is_refused_when_the_fight_would_then_be_short_of_heroes() {
        // party_min equal to the active members: retiring one would refuse the fight, so the prompt refuses Retire
        let (mut cx, member, _, fight) = martyr_cx();
        let n = actives(&cx);
        set_party_min(&mut cx, &fight, n);
        let (mut prompt, _) = opened(&mut cx, &fight);
        prompt.click_id(&mut cx, "fuse.retire").expect("the button");
        let msg = prompt.state(&cx)["message"].as_str().unwrap_or("").to_string();
        assert!(msg.contains("can't retire"), "{msg}");
        assert!(cx.nav.is_empty() && cx.gs().party.member(&member).is_some(), "the prompt stays, nobody leaves");
        // Last Stand still goes on
        prompt.click_id(&mut cx, "fuse.last_stand").expect("the button");
        assert!(matches!(cx.nav.first(), Some(Nav::Pop)));
        assert_ne!(pushed(&cx), "fuse_prompt");
    }

    #[test]
    fn two_spent_martyrs_prompt_one_after_another() {
        let (mut cx, first, _, fight) = martyr_cx();
        let second = cx.gs().party.active_members().get(1).map(|m| m.id.clone()).expect("the test's second active member");
        make_martyr(&mut cx, &second);
        let (mut prompt, _) = opened(&mut cx, &fight);
        assert_eq!(prompt.state(&cx)["member"].as_str(), Some(first.as_str()));
        prompt.click_id(&mut cx, "fuse.last_stand").expect("the button");
        let Some(Nav::Push(mut next)) = cx.nav.pop() else { panic!("the next prompt is pushed") };
        assert!(matches!(cx.nav.as_slice(), [Nav::Pop]));
        cx.nav.clear();
        assert_eq!((next.id(), next.state(&cx)["member"].as_str()), ("fuse_prompt", Some(second.as_str())));
        assert!(!cx.log.iter().any(|l| l.starts_with("Fight: ")), "no fight log while a prompt is still open");
        frame(next.as_mut(), &mut cx, &mut gfx(), 1.0);
        next.click_id(&mut cx, "fuse.retire").expect("the button");
        assert!(cx.gs().party.member(&second).is_none());
        assert_ne!(pushed(&cx), "fuse_prompt", "both chosen: the fight starts");
        assert!(cx.log.iter().any(|l| l.starts_with("Fight: ")));
    }
}

/// PR7-36 (UX-28, ART-19): exits and points of interest draw a glyph per kind, amber while usable, iron once used.
#[cfg(test)]
mod glyph_tests {
    use super::*;
    use crate::hud::hud_tests::gfx;

    #[test]
    fn each_kind_has_its_glyph_and_people_keep_theirs() {
        assert_eq!(glyph_of("exit", ""), Some(MapGlyph::Exit));
        assert_eq!(glyph_of("chest", ""), Some(MapGlyph::Chest));
        assert_eq!(glyph_of("point", "stash"), Some(MapGlyph::Chest), "a stash is a chest");
        assert_eq!(glyph_of("point", "rest"), Some(MapGlyph::Rest));
        assert_eq!(glyph_of("point", "crypts"), Some(MapGlyph::Vault));
        assert_eq!(glyph_of("lore", ""), Some(MapGlyph::Lore));
        assert_eq!(glyph_of("point", ""), Some(MapGlyph::Look), "a point with no use is looked at");
        for kind in ["npc", "member", "encounter", "roamer"] {
            assert_eq!(glyph_of(kind, ""), None, "{kind} keeps its sprite or art");
        }
        let all = [MapGlyph::Exit, MapGlyph::Chest, MapGlyph::Rest, MapGlyph::Vault, MapGlyph::Lore, MapGlyph::Look];
        for (k, g) in all.iter().enumerate() {
            assert!(glyph_rows(*g).iter().all(|r| r.len() == 8 && r.bytes().all(|c| c == b'#' || c == b'.')), "{g:?}: 8 x 8");
            assert!(all[k + 1..].iter().all(|o| glyph_rows(*o) != glyph_rows(*g)), "{g:?}: its own picture");
        }
    }

    #[test]
    fn usable_is_amber_used_is_iron_locked_is_oxblood() {
        assert_eq!(glyph_colour("exit", "present"), crate::tokens::exit(true));
        assert_eq!(glyph_colour("exit", "locked"), crate::tokens::exit(false));
        assert_eq!(glyph_colour("chest", "present"), crate::tokens::exit(true));
        assert_eq!(glyph_colour("chest", "spent"), th().edge);
        assert_eq!(glyph_colour("point", "spent"), th().edge);
        assert_ne!(crate::tokens::exit(true), th().edge, "the two read apart");
    }

    /// Every `#` of the picture is filled in the glyph's colour inside the plate, nothing outside the plate.
    #[test]
    fn a_glyph_draws_its_cells_on_its_plate() {
        let mut g = gfx();
        let col = crate::tokens::exit(true);
        let r = Rect::new(40, 40, mapview::ICON, mapview::ICON);
        {
            let mut ui: Ui<RAct> = Ui::new(&mut g, (-1, -1), true);
            draw_map_glyph(&mut ui.p, r, MapGlyph::Exit, col);
        }
        let pr = g.view.rect(r);
        let px = |x: i32, y: i32| {
            let i = ((y * g.canvas.w + x) * 4) as usize;
            [g.canvas.px[i], g.canvas.px[i + 1], g.canvas.px[i + 2]]
        };
        let lit = (pr.y..pr.y + pr.h).flat_map(|y| (pr.x..pr.x + pr.w).map(move |x| (x, y))).filter(|&(x, y)| px(x, y) == [col[0], col[1], col[2]]).count();
        let cells = glyph_rows(MapGlyph::Exit).iter().map(|r| r.bytes().filter(|c| *c == b'#').count()).sum::<usize>();
        let k = (g.view.rect(Rect::new(0, 0, 1, 1)).w).max(1) as usize;
        assert!(lit >= cells * k * k, "every cell lit: {lit} px for {cells} cells");
        let outside = Rect::new(pr.x + pr.w + 2, pr.y, 4, pr.h);
        assert!((outside.y..outside.y + outside.h).all(|y| (outside.x..outside.x + outside.w).all(|x| px(x, y) == [0, 0, 0])), "nothing past the plate");
    }
}

/// PR7-76 (ACT1-FIX2, FB-441) on the shipped game, every place and marker picked by rule: usable points of interest
/// join the Here list and get edge arrows (PT-43); a point walked onto answers once and no "You can't get there."
/// follows (PT-44, PT-51); a walk a fight broke off goes on after a win only (PT-55); a road's exit opens the world map
/// with the next place that way picked (PT-54).
#[cfg(test)]
mod act1_fix2_tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx};
    use crate::worldmap::tests::{ward_hub, TIER2};
    use emberwake_world::GameState;

    /// The first location of the world (data order) with a point `want` accepts (the marker and its view's state), with
    /// the party entered there; None when there is none.
    fn enter_point(cx: &mut Cx, want: impl Fn(&Dict, &str) -> bool) -> Option<Dict> {
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        for loc in d.merged_entries(&world, "locations") {
            if explore::template_of(&d, loc).is_empty() {
                continue;
            }
            let mut gs = cx.gs().clone();
            if !json::b(&explore::enter(&d, &mut gs, &s(loc, "id"), explore::SPAWN), "ok", false) {
                continue;
            }
            let here = explore::location(&d, &gs).clone();
            if let Some(m) = explore::markers_of(&here).into_iter().find(|m| s(m, "kind") == "point" && want(m, &s(&explore::marker_view(&d, &gs, &here, m), "state"))) {
                *cx.gs_mut() = gs;
                return Some(m);
            }
        }
        None
    }

    fn widget(map: &RegionMap, id: &str) -> Option<Value> {
        map.widgets().into_iter().find(|w| w["id"] == id)
    }

    /// An open tile that reaches `at` by a walk of at least two steps (`path_to`), the nearest first.
    fn start_for(map: &RegionMap, cx: &mut Cx, at: Pos) -> Option<Pos> {
        let d = cx.data.clone();
        let t = explore::template_of(&d, explore::location(&d, cx.gs())).clone();
        let (w, h) = explore::size_of(&t);
        let mut tiles: Vec<Pos> = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).filter(|p| explore::is_open(&t, p.0, p.1)).collect();
        tiles.sort_by_key(|p| (p.0 - at.0).abs() + (p.1 - at.1).abs());
        let keep = cx.gs().position;
        for p in tiles.into_iter().filter(|p| (p.0 - at.0).abs() + (p.1 - at.1).abs() >= 3) {
            cx.gs_mut().position = p;
            if map.path_to(cx, at, false).is_some_and(|w| w.len() >= 2) {
                return Some(p);
            }
        }
        cx.gs_mut().position = keep;
        None
    }

    fn walk_out(map: &mut RegionMap, cx: &mut Cx) {
        let mut now = Instant::now() + Duration::from_secs(1);
        for _ in 0..200 {
            if map.walk.is_empty() && map.walk_target.is_none() {
                break;
            }
            now += Duration::from_millis(500);
            map.tick_at(cx, now);
        }
    }

    /// PT-43: a usable point (present, with a use or a flag to set) has its Here row; a point that only speaks has none.
    #[test]
    fn pt43_usable_points_join_the_here_list() {
        let (mut cx, mut g) = (ward_hub(&TIER2, &[], false), gfx());
        let m = enter_point(&mut cx, |m, st| point_listed(m, st)).expect("a usable point in the world");
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let row = widget(&map, &format!("here.{}", s(&m, "id"))).expect("the point's Here row");
        assert_eq!(row["text"], json!(s(&explore::marker_view(&cx.data, cx.gs(), explore::location(&cx.data, cx.gs()), &m), "label")));
        assert!(!point_listed(&m, "spent") && !point_listed(&m, "absent"), "a spent or absent point is not listed");
        let mut bare = m.clone();
        bare.shift_remove("use");
        bare.insert("sets_flags".into(), json!([]));
        assert!(!point_listed(&bare, "present"), "a point that only speaks is not listed");
        // ENGINE-37 (LOOK-POINT1): a look-only point is listed with no flag to set, and acting on it sets nothing
        bare.insert("use".into(), json!("look"));
        assert!(point_listed(&bare, "present"), "a look-only point is listed");
        let flags = cx.gs().flags.clone();
        act_on_point(&mut cx, &bare);
        assert_eq!(cx.gs().flags, flags, "a look sets nothing");
        if let Some(talker) = enter_point(&mut cx, |m, st| st == "present" && !point_listed(m, st)) {
            let mut map = RegionMap::new(&mut cx);
            frame(&mut map, &mut cx, &mut g, 1.0);
            assert!(widget(&map, &format!("here.{}", s(&talker, "id"))).is_none(), "{talker:?} only speaks: no row");
        }
    }

    /// PT-43: a usable point out of view gets an edge arrow (the party as far from it as the map allows).
    #[test]
    fn pt43_an_off_screen_usable_point_gets_an_edge_arrow() {
        let (mut cx, mut g) = (ward_hub(&TIER2, &[], false), gfx());
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        // the first place (data order) with a usable point out of view from the tile farthest from it
        let mut found = None;
        'places: for loc in d.merged_entries(&world, "locations") {
            if explore::template_of(&d, loc).is_empty() {
                continue;
            }
            let mut gs = cx.gs().clone();
            if !json::b(&explore::enter(&d, &mut gs, &s(loc, "id"), explore::SPAWN), "ok", false) {
                continue;
            }
            let here = explore::location(&d, &gs).clone();
            let t = explore::template_of(&d, &here).clone();
            let (w, h) = explore::size_of(&t);
            let usable: Vec<Dict> = explore::markers_of(&here).into_iter().filter(|m| s(m, "kind") == "point" && point_listed(m, &s(&explore::marker_view(&d, &gs, &here, m), "state"))).collect();
            for m in usable {
                let at = explore::marker_pos(&t, &m, gs.explore_int("loc_steps"));
                let Some(far) = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).filter(|p| explore::is_open(&t, p.0, p.1)).max_by_key(|p| (p.0 - at.0).abs() + (p.1 - at.1).abs()) else { continue };
                gs.position = far;
                *cx.gs_mut() = gs.clone();
                let mut map = RegionMap::new(&mut cx);
                for _ in 0..3 {
                    frame(&mut map, &mut cx, &mut g, 1.0);
                }
                let r = map.geo.cell_of(at);
                let seen = r.intersect(MAP_AREA);
                if seen.w <= 0 || seen.h <= 0 {
                    found = Some((m, at, t, map, r));
                    break 'places;
                }
            }
        }
        let (m, at, t, map, r) = found.expect("a usable point out of view from some tile of its place");
        let (w, h) = explore::size_of(&t);
        let id = s(&m, "id");
        let arrow = map.state(&cx)["arrows"].as_array().cloned().unwrap_or_default().into_iter().find(|a| s(json::dict(a), "id") == id).expect("its edge arrow");
        // placed on the map area's edge band, pointing at the point's tile, with its widget
        let (x, y) = (arrow["at"][0].as_i64().unwrap() as i32, arrow["at"][1].as_i64().unwrap() as i32);
        let a = MAP_AREA;
        assert!(a.contains(x, y), "inside the map area: ({x}, {y})");
        let edge = (x - a.x).min(a.x + a.w - x).min(y - a.y).min(a.y + a.h - y);
        assert!(edge <= ARROW + 1, "at the edge: {edge} px in");
        let (tx, ty) = (r.x as f64 + r.w as f64 / 2.0 - x as f64, r.y as f64 + r.h as f64 / 2.0 - y as f64);
        let (ux, uy) = (arrow["dir"][0].as_f64().unwrap(), arrow["dir"][1].as_f64().unwrap());
        let cos = (tx * ux + ty * uy) / (tx * tx + ty * ty).sqrt();
        assert!(cos > 0.95, "points at the point: cos {cos}");
        let wd = widget(&map, &format!("arrow.{id}")).expect("the arrow's widget");
        assert_eq!(wd["kind"], json!("arrow"));
        // walked next to it, the arrow goes
        let near = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).filter(|p| explore::is_open(&t, p.0, p.1)).min_by_key(|p| (p.0 - at.0).abs() + (p.1 - at.1).abs()).unwrap();
        cx.gs_mut().position = near;
        let mut map = RegionMap::new(&mut cx);
        for _ in 0..3 {
            frame(&mut map, &mut cx, &mut g, 1.0);
        }
        assert!(widget(&map, &format!("arrow.{id}")).is_none(), "in view: no arrow");
    }

    /// PT-55 (review): a click walk that already arrived leaves nothing to resume; a key step into a fight then a win
    /// does not walk the party anywhere.
    #[test]
    fn pt55_a_key_step_into_a_fight_after_a_finished_walk_resumes_nothing() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        let d = cx.data.clone();
        let t = explore::template_of(&d, explore::location(&d, cx.gs())).clone();
        let (w, h) = explore::size_of(&t);
        let marked: Vec<Pos> = explore::markers_of(explore::location(&d, cx.gs())).iter().map(|m| explore::marker_pos(&t, m, 0)).collect();
        let free = |p: Pos| explore::is_open(&t, p.0, p.1) && !marked.contains(&p);
        let at = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).find(|&(x, y)| (0..6).all(|i| free((x + i, y)))).expect("a free row");
        cx.gs_mut().position = at;
        frame(&mut map, &mut cx, &mut g, 1.0);
        let dest = (at.0 + 3, at.1);
        let r = map.geo.cell(dest.0 as i32, dest.1 as i32);
        map.mouse = (r.x + r.w / 2, r.y + r.h / 2);
        map.act(&mut cx, RAct::MapClick);
        walk_out(&mut map, &mut cx);
        assert_eq!(cx.gs().position, dest, "the click walk arrived");
        // a key step on (as the held key does), into a fight that keeps what walk is under way
        cx.gs_mut().position = (dest.0 + 1, dest.1);
        map.keep_walk(&cx);
        cx.game.as_mut().unwrap().fought = Some("won".to_string());
        map.tick_at(&mut cx, Instant::now() + Duration::from_secs(3));
        assert!(map.walk.is_empty(), "no stale walk resumed: {:?}", map.walk);
        assert_eq!(cx.gs().position, (dest.0 + 1, dest.1), "the party stays");
    }

    /// PT-44, PT-51: a point that sets a flag, walked onto by a click on it, logs its line once and never "You can't get
    /// there." after it.
    #[test]
    fn pt44_pt51_a_point_walked_onto_answers_once() {
        let mut cx = ward_hub(&TIER2, &[], false);
        let fresh = |gs: &GameState, m: &Dict| json::arr(&json::normalize(json::get(m, "sets_flags"))).iter().any(|f| !gs.flag_true(&s(json::dict(f), "key")));
        let probe = cx.gs().clone();
        let m = enter_point(&mut cx, |m, st| st == "present" && fresh(&probe, m)).expect("a point that sets a flag not set yet");
        let mut map = RegionMap::new(&mut cx);
        let d = cx.data.clone();
        let t = explore::template_of(&d, explore::location(&d, cx.gs())).clone();
        let at = explore::marker_pos(&t, &m, cx.gs().explore_int("loc_steps"));
        start_for(&map, &mut cx, at).expect("a tile a walk away");
        // the job waiting on its flag is under way (its "Quest updated" line then comes between, as in a game)
        let keys: Vec<String> = json::arr(&json::normalize(json::get(&m, "sets_flags"))).iter().map(|f| s(json::dict(f), "key")).collect();
        let world = cx.gs().world_id.clone();
        let jobs: Vec<String> = d.merged_entries(&world, "quests").into_iter().filter(|q| json::aget(q, "steps").iter().any(|st| keys.contains(&s(json::dict(st), "flag")))).map(|q| s(q, "_qid")).collect();
        for q in &jobs {
            cx.gs_mut().quests.insert(q.clone(), json!({"state": "active", "objective": 0}));
        }
        // the roads open now are told first (that notice would stop the walk)
        crate::worldmap::note_new_roads(&mut cx);
        cx.notice.clear();
        cx.log.clear();
        map.act(&mut cx, RAct::Marker(s(&m, "id")));
        assert!(!map.walk.is_empty(), "a walk starts");
        walk_out(&mut map, &mut cx);
        let line = explore::point_line(&m);
        assert_eq!(cx.log.iter().filter(|l| **l == line).count(), 1, "its line once: {:?}", cx.log);
        assert!(!cx.log.iter().any(|l| l == "You can't get there."), "{:?}", cx.log);
        if !jobs.is_empty() {
            assert!(cx.log.iter().any(|l| l.starts_with("Quest updated: ")), "the job's line: {:?}", cx.log);
        }
    }

    /// PT-55: a click walk a fight broke off goes on after a win with the party where it was; a flight (the party back a
    /// tile) or a loss drops it.
    #[test]
    fn pt55_a_walk_broken_off_by_a_fight_goes_on_after_a_win() {
        for (outcome, back, goes_on) in [("won", false, true), ("fled", true, false), ("lost", false, false)] {
            let (mut cx, mut g) = (game_cx(), gfx());
            let mut map = RegionMap::new(&mut cx);
            let d = cx.data.clone();
            let t = explore::template_of(&d, explore::location(&d, cx.gs())).clone();
            let (w, h) = explore::size_of(&t);
            let marked: Vec<Pos> = explore::markers_of(explore::location(&d, cx.gs())).iter().map(|m| explore::marker_pos(&t, m, 0)).collect();
            let free = |p: Pos| explore::is_open(&t, p.0, p.1) && !marked.contains(&p);
            let at = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).find(|&(x, y)| (0..6).all(|i| free((x + i, y)))).expect("a free row");
            cx.gs_mut().position = at;
            frame(&mut map, &mut cx, &mut g, 1.0);
            let dest = (at.0 + 5, at.1);
            let r = map.geo.cell(dest.0 as i32, dest.1 as i32);
            map.mouse = (r.x + r.w / 2, r.y + r.h / 2);
            map.act(&mut cx, RAct::MapClick);
            map.tick_at(&mut cx, Instant::now() + Duration::from_secs(1));
            let found = cx.gs().position;
            assert_eq!(found, (at.0 + 1, at.1), "one step taken");
            // the fight: the walk is kept, then cleared as a fight starts
            map.keep_walk(&cx);
            map.walk.clear();
            map.walk_target = None;
            if back {
                cx.gs_mut().position = at;
            }
            cx.game.as_mut().unwrap().fought = Some(outcome.to_string());
            map.tick_at(&mut cx, Instant::now() + Duration::from_secs(3));
            assert_eq!(!map.walk.is_empty() || cx.gs().position != if back { at } else { found }, goes_on, "{outcome}: the walk goes on only after a win");
            assert!(cx.game.as_ref().unwrap().fought.is_none(), "the outcome is taken");
            if goes_on {
                assert_eq!(map.walk_dest, Some(dest), "to the same tile");
                walk_out(&mut map, &mut cx);
                assert_eq!(cx.gs().position, dest, "arrived");
            }
        }
    }

    /// PT-54: in a place with several world-map exits, each road's exit picks an open neighbour the exit's way (within
    /// 90 degrees on the world map), the place's own world-map exit none; stepping onto it opens the world map with it
    /// picked.
    #[test]
    fn pt54_a_roads_exit_picks_the_next_place_that_way() {
        let mut cx = ward_hub(&TIER2, &[], false);
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        let mut picked = 0;
        for loc in d.merged_entries(&world, "locations") {
            let exits: Vec<Dict> = explore::markers_of(loc).into_iter().filter(|m| s(m, "kind") == "exit" && s(m, "to") == explore::WORLD_MAP).collect();
            if exits.len() < 2 || explore::template_of(&d, loc).is_empty() {
                continue;
            }
            let mut gs = cx.gs().clone();
            if !json::b(&explore::enter(&d, &mut gs, &s(loc, "id"), explore::SPAWN), "ok", false) {
                continue;
            }
            *cx.gs_mut() = gs;
            assert_eq!(crate::worldmap::exit_toward(&d, cx.gs(), explore::WORLD_MAP), None, "the plain exit picks nothing");
            for e in &exits {
                let Some(q) = crate::worldmap::exit_toward(&d, cx.gs(), &s(e, "id")) else { continue };
                let here = cx.gs().location_id.clone();
                let view = emberwake_world::travel::view(&d, cx.gs());
                let road = json::aget(&view, "routes").iter().map(json::dict).any(|r| json::b(r, "open", true) && ((s(r, "from") == here && s(r, "to") == q) || (s(r, "to") == here && s(r, "from") == q)));
                assert!(road, "{q} is an open neighbour of {here}");
                assert_eq!(crate::worldmap::WorldMap::toward(&cx, &s(e, "id")).selected, q);
                picked += 1;
            }
        }
        assert!(picked > 0, "some road's exit picks a place");
        assert_eq!(crate::worldmap::WorldMap::toward(&cx, "").selected, "", "no exit: nothing picked");
    }
}
