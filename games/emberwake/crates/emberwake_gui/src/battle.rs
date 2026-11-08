//! The battle screen (COMBAT-UI3 layout A). Two layers: the battlefield is pixel art scaled with nearest sampling;
//! panels, text, hover cards and menus are drawn smooth at the window's real resolution (painter.rs). Every change goes
//! through the command API (emberwake_host on aegis_host), the same one `aegis serve` and the agent link use.
//! Two modes: the Test Arena (a catalog encounter, Restart / Play again) and a world battle (GameEncounter: the saved
//! party at its stored HP against a world encounter; on Continue the kit's `finish_world` (the runner's after_fight)
//! settles the battle's tally (the kit settles the party: HP, the stay-behind and roster rules; then the rewards) and
//! the results popup opens over the map).
//! Mouse: click a hero to select; click a foe, an ally, the hero itself or a tile for its options; hover anything for
//! details; right click closes a menu or undoes the hero's last step; the wheel scrolls the log. Keys: Enter confirm,
//! Space step / next round, A autoplay, Z undo, L log, R restart (arena), Esc close; hold Alt to outline every unit,
//! covered or not (STAGE1-O). Timeline battles (INPUT1-U): a click on a target sets the round target and opens the skill
//! menu (picks through the host's `pick`, the planner's ghost fills the rest), Shift+1-9 are the hero's skill hotkeys
//! (INPUT2; keys 1-4 select the party's members in every battle), F or a click on a ghost block accepts the ghost as
//! picks, S targets the hero itself, Z undoes the last pick. INPUT1-T: T opens the full timeline view over the grid panel
//! (`timeline_view.rs`, every unit's track; T or Esc closes it). INPUT1-PB: during a timeline battle's playback each
//! impact draws a short tracer from its actor to what it hit, the strip frames the resolving block, P cycles the
//! playback speed (1x, 0.5x, block by block; N plays the next block), the log groups its lines by round and block, and
//! an open timeline view pauses the playback at the resolving block's end.
//! ENCOUNTER2-C2 (docs/specs/ENCOUNTER2_C.md 3.3): each new talk cue of `Session::talks` opens the shared talk box
//! (`talk.rs`) once its step's impacts have played, over a 35 % veil, the speaker's token in the gold ring; while it is
//! open the screen sends no request, autoplay waits and every board click and battle key is swallowed; replies run in
//! the game in play (a world fight) or a scratch game state of the Arena's world.
//! VISCLAR1-D (timeline battles; classic keeps its old flow): after a round plays, the next round's planning starts by itself
//! (setting Auto-confirm next round; not while watching a recording), and a banner names each new round.


mod board;
mod draw;
mod flow;
mod input;
mod marks;
mod overlays;
mod playback;
mod skill_menu;
mod talks;
mod tips;
mod tokens;
#[cfg(test)]
mod tests;

pub(crate) use marks::{party_foresight_off, seen_intents};
// VISCLAR1: only the debug-build settings row and the tests read it
#[allow(unused_imports)]
pub(crate) use marks::FORESIGHT_OFF;
pub(crate) use overlays::{overlay_default, Layers, OVERLAY_GROUPS};
pub(crate) use tips::{action_rows, ap_lines, kind_color};
use draw::{draw, log_groups_json};
use flow::boss_fight;
use input::{pick_json, Deferred, Popup};
use marks::{held_guard, hold_pick_ids, meters_of, push_forecasts, push_json, Arrow, HookMarks};
use overlays::{overlay_json, overlay_of};
use playback::PlaySpeed;
use talks::BattleTalk;
use tips::{ApAnim, ApPips};
use tokens::{Float, Tokens};
use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use crate::music;
use crate::fx::{self, Impact};
use crate::portraits::Faces;
use crate::talk::{self, TalkAct};
use crate::timeline_view;
use aegis_ui::fx::{Camera, Clock, Fx};
use aegis_ui::pacing::{FrameNeed, SLOW_LIFE};
use crate::stage::View;
use crate::screens::{Cx, GameCx, Key, Nav, Screen};
use aegis_ui::ui::{Gfx, Widget};
use emberwake_battle::grid::V2;
use emberwake_battle::round_intent::{self as ri};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::{Phase, Session};
use aegis_core::json::{self, s, Dict};
use emberwake_battle::world::World;
use emberwake_host::{Battle, Host};
use emberwake_world::story;
use serde_json::{json, Value};
use std::time::{Duration, Instant};


/// The Flat view's cell (and the grid panel's: 12 x 9 cells of `TILE` at (`GRID_X`, `GRID_Y`)).
const TILE: i32 = 32;
/// PORTRAITFX1-P (6.2 row 3): the hero and intel panels' portrait side, its inset from the panel's top-left, where the
/// text lines beside it start (from the panel's left edge) and the cue rows' surface.
const PANEL_PORTRAIT: i32 = 32;
const PANEL_INSET: i32 = 4;
const PANEL_TEXT_X: i32 = 40;
const PANEL_SURFACE: &str = "panel";
/// PORTRAITFX1-TL (6.2 row 4): the full timeline view's track icons draw on the cue rows' `timeline` surface.
const TIMELINE_SURFACE: &str = "timeline";
/// PORTRAITFX1-TL (user 2026-11-01): the events an enemy's timeline row reacts to: the blows (when the board shows them).
const TIMELINE_ENEMY_EVENTS: [&str; 3] = ["hit", "crit", "guard_hit"];
/// INPUT1 MR12: a dash (a skill with a position moving to it at its start) crosses its route in this many ms at 1x
/// (the FX clock: the animation speed applies), playing the rig's `dash` clip, else its walk (until DASH1's clip).
const DASH_MS: u64 = 250;
/// INPUT1-AP (AP6): the AP gauge's most pips (the user's first try); a `max_ap` above it puts `ceil(max_ap / AP_PIPS)`
/// AP in a pip.
const AP_PIPS: i32 = 12;
/// AP10: paid pips turn hollow, and regained ones fill, over this many ms (at once with skip animations or reduce
/// motion).
const AP_ANIM_MS: f32 = 150.0;
/// INPUT1 PB1: an impact tracer's life on the FX clock (ms; the animation speed and PB3's 0.5x apply), its alpha at the
/// impact and how far it stops short of each token centre (px).
const TRACER_MS: u64 = 400;
const TRACER_ALPHA: u8 = 200;
const TRACER_INSET: f32 = 3.0;
/// INPUT1 PB3: the settings key of the playback speed and its values (setting value, chip label, chip widget id).
pub(crate) const PLAYBACK_SPEED: &str = "playback_speed";
pub(crate) const PLAYBACK_SPEEDS: [(&str, &str, &str); 3] = [("1x", "1x", "playback.speed.1"), ("0.5x", "0.5x", "playback.speed.half"), ("block", "Block", "playback.speed.block")];
/// VISCLAR1-D: the settings key of "Auto-confirm next round" (Settings, Gameplay; on by default): after a round of a timeline
/// battle plays, the battle goes to the next round's planning by itself.
pub(crate) const AUTO_CONFIRM: &str = "auto_confirm";
/// The battle screen's panels by name (`BattleScreen::panels`; the UI Gallery's battle panel entries).
pub const PANELS: [&str; 7] = ["strip", "party", "hero", "grid", "hostiles", "intel", "bar"];
const GRID_X: i32 = 128;
/// INPUT1 TV6: the timeline view's padding inside the grid panel's rect.
const TV_PAD: i32 = 4;
const GRID_Y: i32 = 30;
const TACTICS: [&str; 6] = ["balanced", "aggressive", "focus_weakest", "defensive", "ranged_kite", "economy"];
const TACTIC_LABELS: [&str; 6] = ["Balanced", "All-out Attack", "Focus Weakest", "Guard & Counter", "Kite at Range", "Economy"];
const TACTIC_HELP: [&str; 6] = [
    "The unit's own role: fights, guards when hurt, flees when nearly down.",
    "Nearest enemy, strongest affordable skills, no guarding or fleeing.",
    "Goes for the enemy that is closest to falling.",
    "Guards first, then trades blows.",
    "Holds maximum range and backs away from melee.",
    "Uses level 1 skills to stretch AP over more actions.",
];
/// RULESET1_INTENT AI5: what each tactic picks under the round intent (the timeline ruleset).
const TACTIC_INTENT_HELP: [&str; 6] = [
    "The AI table: heals a hurt friend, guards when hurt, flanks an engaged foe, else Assault.",
    "Assault on the first ranked enemy.",
    "Assault on the lowest-HP enemy it can reach this round.",
    "Guard on yourself.",
    "Ranged: Assault and kite. Melee: Disengage when offered, else Assault.",
    "Assault with the cheapest opener, then Rest.",
];

const PANEL: Rgba = rgba(22, 32, 54, 245);
const ALLY: Rgba = rgb(110, 170, 255);
/// RULESET1-U slice 5: the poise bar's colour (a pale steel blue, apart from HP green and red).
const POISE: Rgba = rgb(150, 190, 235);
/// RULESET1-U slice 4: the enemy facing chevron (pale, so it reads apart from the red tell arrows).
const FACING: Rgba = rgb(250, 226, 160);
/// WINDUP1 W9: Tallow Amber (art bible 5), the wind-up outline.
const AMBER: Rgba = rgb(0xd9, 0x77, 0x06);
/// ENCOUNTER2-C DT7: the veil over the board and panels while a battle talk is open (35 % black).
const TALK_VEIL: Rgba = rgba(0, 0, 0, 89);

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Select(usize),
    UnitClick(usize),
    TileClick(V2),
    Choose(usize),
    ClosePopup,
    /// VISCLAR1-D: a click inside the skill menu's panel, off its cells: nothing.
    MenuGap,
    Confirm,
    Clear,
    Undo,
    Autoplay,
    PartyMenu,
    HeroMenu,
    NextRound,
    Step,
    Restart,
    ToggleLog,
    /// World battle over: settle it and show the results.
    Continue,
    /// Leave the Test Arena.
    Leave,
    /// INPUT1 SL4 (round intent): the hero panel's Self button and key S: the selected hero targets itself (the self
    /// menu).
    SelfTarget,
    /// ENCOUNTER2 H1: the hero picked to hold a hold escape's tiles.
    StayPick(usize),
    /// ENCOUNTER2-C DT9: a reply or Continue in the open battle talk's box.
    Talk(TalkAct),
    /// INPUT1 TV1: key T opens and closes the full timeline view.
    TimelineView,
    /// TV5: a click in the open timeline view: a party track (or one of its spans) selects that hero; anything else in
    /// the view (a block, an enemy track, its backdrop) does nothing, so the board under it takes no click.
    TimelinePick(Option<usize>),
    /// INPUT1 PB3: a playback speed chip (an index of `PLAYBACK_SPEEDS`; key P cycles them).
    PlaybackSpeed(usize),
    /// PB3, PB5: Next block (key N): a playback held at a block's end plays the next block.
    NextBlock,
    /// ENCOUNTER2-C4 (decision "ENCOUNTER2-C answers" Q-C5): a locked round's Skip (`battle.skip`): resolves the rest of
    /// the round at once (host `resolve`: the same result, no animation).
    Skip,
    /// INPUT2: key F: the selected hero's whole ghost becomes its picks (host `accept_ghost`).
    FillGhost,
    /// INPUT2: a click on a strip block showing a ghost entry (the entry's index in the plan): the ghost up to it becomes
    /// picks, so that command is the hero's latest pick.
    GhostSlot(usize),
}

/// ASSETORG2: a fielded member's body and palette in world `world` (the kit's `begin_world_with` art callback): its
/// character's, else its base unit's (its class's body when the unit has none).
fn member_art(d: &emberwake_world::Data, world: &str, m: &emberwake_world::UnitState) -> (String, String) {
    let character = d.resolve(world, "characters", &m.lore_ref);
    if character.is_empty() {
        let unit = d.entry("units", &m.base_unit);
        let class = d.entry("classes", &s(unit, "class_id"));
        (emberwake_content::art::body_of(&Default::default(), unit, class), emberwake_content::art::palette_of(&Default::default(), unit))
    } else {
        let (art, palette, _) = crate::screens::character_art(d, character);
        (art, palette)
    }
}

/// The log lines of a settled world fight: its events, and a story fight's notes ("Try again.", the effects'); its
/// campaign events (CAMPAIGN1 W1: log lines, notices the map shows next, the act-start save).
pub fn log_settled(cx: &mut Cx, runner: &str, res: &Dict) {
    crate::region::log_events(cx, json::aget(res, "events"));
    crate::runover::take(cx, res);
    if runner == story::FIGHT_SOURCE {
        for n in json::sget(res, "notes") {
            cx.log(n);
        }
    }
}

enum Mode {
    Arena,
    /// A world fight: the fight dict, the runner that settles it (explore / travel / dialogue / story) and the member ids of
    /// the party units in battle order.
    /// `members`: party member id and the saved pouch ids that found no battle slot (kept on write-back), per ally.
    World { fight: Dict, runner: String, members: Vec<(String, Vec<String>)> },
}

pub struct BattleScreen {
    pub host: Host,
    w: Vec<Widget<Action>>,
    mouse: (i32, i32),
    selected: Option<usize>,
    popup: Option<Popup>,
    /// VISCLAR1-D: the menu's Move from a unit's menu waits for a tile: the hero and the round (the next target click ends it).
    move_aim: Option<(usize, i32)>,
    /// The tiles that Move can go to, marked on the board while it waits.
    move_tiles: Vec<rip::MoveTile>,
    playing: bool,
    auto: bool,
    next_at: Instant,
    pending_next_round: bool,
    floats: Vec<Float>,
    flashes: Vec<(usize, Instant)>,
    last_hp: Vec<i32>,
    party_tactic: usize,
    hero_tactic: usize,
    pub message: String,
    show_log: bool,
    log_scroll: i32,
    tuning_checked: Instant,
    mode: Mode,
    encounter: String,
    seed: Option<i64>,
    /// Arena only: the world the encounter belongs to (played with that world's characters); None = catalog.
    arena_world: Option<String>,
    /// Arena only (RULESET1-R): the ruleset picked in the Scene Runner (classic or timeline); None = the encounter's own
    /// (its world's, classic for a catalog encounter).
    arena_ruleset: Option<String>,
    /// Arena only (RULESET1-G): the extensions switched for the run ("-id,+id", the host's `rules`; "" = none).
    arena_rules: String,
    /// Arena only (RULESET1 V2): "Reveal enemy plans" (the host's `plan_reveal`).
    arena_reveal: bool,
    /// Arena only (INPUT1 MR11): the "Diagonal melee" pick (none, maneuver, all; "" = the ruleset row's value: the
    /// host's `melee_diagonal`).
    arena_diagonal: String,
    /// HOOKS1 CN2 (HOOKS1-UI slice 2): the world whose `classes` overlay names the units' classes on the hover cards: a
    /// world fight's world; "" (the class rows' names) in the Arena, a catalog screen.
    class_world: String,
    /// Session events already shown in the message bar.
    events_seen: usize,
    /// Resolver log lines already sounded.
    log_seen: usize,
    /// Where the playtest hub keeps this battle's session (None until first saved).
    slot: Option<usize>,
    /// The panels of the last frame by name (UIGALLERY1: the UI Gallery shows each one on its own), named from
    /// `PANELS`.
    pub panels: Vec<(&'static str, Rect)>,
    /// (requests, checkpoints) of the session at its last save.
    saved: (usize, usize),
    /// Playtest trail events of the battle (plan edits, confirms, the end), handed to the hub on the next tick.
    trail_out: Vec<Value>,
    /// The battle's end is in the trail.
    end_logged: bool,
    /// Sessions an agent-link `new` ended before the hub could be reached (saved on the next tick).
    parked: Vec<(Option<usize>, aegis_host::playtest::Session)>,
    /// A playback of a recorded session (Scene Runner Playtests): watched, not recorded again.
    viewing: bool,
    /// A take-over: where its session branched off (recording id, battle, request), kept in its `end`.
    parent: Value,
    /// SPRITE1-U: the token animations; FX1-L1: the FX clock that replaced the animation clock (ms at the animation
    /// speed setting, standing still through a hit-stop; clips run on it) and when it last advanced.
    tokens: Tokens,
    clock: Clock,
    clock_at: Instant,
    /// FPSCAP1: the last frame the battle was not at rest (`at_rest`).
    astir_at: Instant,
    /// FX1-L1 rule I: the step feedback waiting for its action to land (at most one: rule F fires it before the next).
    impacts: Vec<Impact>,
    /// FX1-L1 rule P: autoplay waits `impact_gap_ms` after the last impact.
    gap_until: Option<Instant>,
    /// The command of the request being shown (only a single `step` defers its feedback; rule 7).
    req_cmd: String,
    /// The battle view at the last tick (a view change flushes the impacts, rule F).
    view_seen: Option<View>,
    /// FX1-L1 slice 2: the live effects on the FX clock (board px of the current view), the camera whose shake moves
    /// the board (4.1, 4.5), and the phases already fired when last seen (a newly fired one may shake).
    fx: Fx,
    camera: Camera,
    phases_seen: usize,
    /// FX1-L2 (4.3): the targets stepping aside from a miss (unit, clock ms of the impact, direction away).
    sidesteps: Vec<fx::Sidestep>,
    /// The battle's revision (every request and battle start moves it): the previews below are kept for one.
    rev: u64,
    /// ENCOUNTER2-C (3.3): the open talk (None = none), how many of the session's talk cues were taken, the Arena's
    /// scratch game state of its world (made at the first talk, dropped with the battle), autoplay's plan held back
    /// for a talk (sent when it closes), and the talk box's portraits and node shown.
    talk: Option<BattleTalk>,
    talks_seen: usize,
    talk_gs: Option<emberwake_world::GameState>,
    auto_after_talk: bool,
    faces: Faces,
    talk_node: String,
    /// PORTRAITFX1-P (6.2 row 3, surface `panel`): the hero and intel panels' portraits, on a copy of the FX clock (a
    /// hit-stop pauses them, the animation speed scales them, 4.5); they react to each step's portrait events at its
    /// impact (rule I).
    panel_faces: Faces,
    /// PORTRAITFX1-TL (6.2 row 4, surface `timeline`): the full timeline view's track icons, on the same FX clock copy
    /// as the panels and reacting at the same impacts, capped at the minimal tier (4.6: flash, tints, fade; no jolt,
    /// no breath) however large the window scales them.
    timeline_faces: Faces,
    /// PORTRAITFX2 5.3: the battle's own portrait events: `battle_start` (and `ambush`) still to fire on the first
    /// frame, the round and outcome the last change left (`round_start`, `victory`) and the last popped action whose
    /// `skill` fired.
    portrait_start: bool,
    portrait_round: i32,
    portrait_outcome: String,
    portrait_popped: Option<(i32, i32, usize)>,
    /// ENCOUNTER2-C4: the last locked round whose playback the screen started by itself (0: none yet this battle).
    cutscene: i32,
    /// INPUT1 GH2 b: the members whose round target the player set this round (their ghost is the auto plan on it).
    player_target: Vec<usize>,
    /// GH2 a: each member's hero tactic (an index of `TACTICS`; none = the party tactic).
    member_tactic: std::collections::HashMap<usize, usize>,
    /// GH2 a, IN6 (decision "INPUT1 answers, part 4" Q18): members whose ghost would change their intent and drop
    /// intent-bound buffs: the change that realizes it waits for Confirm, the ghost shows on copies meanwhile.
    deferred: Vec<(usize, Deferred)>,
    /// The round whose planning start drafted the ghosts (GH2 a, GH6).
    ghosted: i32,
    /// IF5: the hovered menu row's preview (revision, row, the plan as if picked); MR13 reads it too.
    preview: Option<(u64, usize, Option<(World, emberwake_battle::plan::Plan)>)>,
    /// GH2 a: the selected deferred member's ghost (revision, member, its plan on copies).
    deferred_view: Option<(u64, usize, Option<(World, emberwake_battle::plan::Plan)>)>,
    /// MR13: the position tiles the last frame marked (the agent link's `overlay.positions`).
    positions_shown: Vec<(V2, bool)>,
    /// VISCLAR1 AR3, VC2: the target arrows and the focus set the last frame drew (the agent link's `overlay.arrows`
    /// and `focus`).
    arrows_shown: Vec<Arrow>,
    focus_shown: Vec<usize>,
    /// TEXTFIT1: the menus' row pitch at the last frame's text size (11 at Normal).
    menu_row: i32,
    /// INPUT1 SL2, SL5, SL7, IS1: the inspected unit and the round it was inspected in (the intel panel is pinned on it
    /// until another inspect, Esc or that round's end).
    inspect: Option<(usize, i32)>,
    /// INPUT1-AP (AP10): the AP gauge's animation (its member, round and look, the levels drawn from and to).
    ap_anim: Option<ApAnim>,
    /// AP10: the round and each party member's AP while it was planned (the gauge's hollow pips during playback).
    round_ap: (i32, Vec<(usize, i32)>),
    /// The AP gauge as last drawn (the agent link's `ap_gauge`; None: hidden).
    ap_gauge: Option<Value>,
    /// VISCLAR1-B: the planned blocks' labels as last drawn, each span's (the agent link's `strip_labels`).
    strip_labels: Vec<Value>,
    /// INPUT1 TV1: the full timeline view is open (key T), and the view as last drawn (the agent link's
    /// `timeline_view`).
    tv_open: bool,
    tv_shown: Option<aegis_ui::timeline::Timeline>,
    /// INPUT1 PB1: the live impact tracers with the FX clock ms each started at, and the popped action last traced.
    tracers: Vec<(fx::Trace, u64)>,
    traced: Option<(i32, i32, usize)>,
    /// PB2: the round and block the last playback step resolved (the strip's cursor, the view's playhead).
    play_block: Option<(i32, i32)>,
    /// PB3, PB5: the block the last step finished (its end may hold the playback: Block mode, the timeline view open),
    /// and the block the playback waits after with whether Block mode holds it (the agent link's `playback.waiting`).
    block_ended: Option<i32>,
    waiting: Option<(i32, bool)>,
    /// VISCLAR1-D: each party card's AP pips over time, the cards as last drawn (the agent link's `party_cards`), and the
    /// round banner's round with the FX clock ms it started at.
    card_ap: Vec<(usize, ApPips)>,
    cards_shown: Vec<Value>,
    banner: Option<(i32, u64)>,
    /// VISCLAR1-D: the round's summary shown before the next round's prompt while that prompt stands (prompt, summary), and
    /// when the last advance of a timeline battle came (`CONFIRM_GUARD_MS`).
    round_note: Option<(String, String)>,
    advanced_at: Option<Instant>,
}

/// VISCLAR1-D: how long after an advance the player's Confirm is ignored: "Next round" and "Confirm round" are one button,
/// so a click or Enter meant for the round's end must not take the plan that just came up.
pub(crate) const CONFIRM_GUARD_MS: u64 = 350;

/// FPSCAP1: how long after the last stir a battle must stay at rest before it draws slower.
pub(crate) const REST_AFTER: Duration = Duration::from_millis(1500);

impl BattleScreen {
    /// FPSCAP1: waiting for the player (a plan, Next after a round, the end banner) with nothing playing and no short-lived
    /// effect alive. Only `playing`, `auto` and `pending_next_round` move a battle by themselves. VISCLAR1-D: the round
    /// banner and the cards' AP pips count as effects, both on the FX clock, so a slow animation speed keeps them smooth.
    pub(crate) fn at_rest(&self) -> bool {
        self.host.session.is_some() && !self.viewing && !self.playing && !self.auto && !self.pending_next_round && self.talk.is_none()
            && self.impacts_done(Instant::now()) && self.fx.is_empty() && self.floats.is_empty() && self.flashes.is_empty() && self.tracers.is_empty() && self.sidesteps.is_empty()
            && !self.round_banner_alive() && !self.pips_moving()
    }

    fn blank(cx: &Cx, mode: Mode) -> Self {
        Self {
            host: Host::new(Battle::with_data(cx.data.clone())),
            w: Vec::new(),
            mouse: (0, 0),
            selected: None,
            popup: None,
            move_aim: None,
            move_tiles: Vec::new(),
            playing: false,
            auto: false,
            next_at: Instant::now(),
            pending_next_round: false,
            floats: Vec::new(),
            flashes: Vec::new(),
            last_hp: Vec::new(),
            party_tactic: 0,
            hero_tactic: 0,
            message: String::new(),
            show_log: false,
            log_scroll: 0,
            tuning_checked: Instant::now(),
            mode,
            encounter: cx.encounter.clone(),
            seed: cx.seed,
            arena_world: None,
            arena_ruleset: None,
            arena_rules: String::new(),
            arena_reveal: false,
            arena_diagonal: String::new(),
            class_world: String::new(),
            events_seen: 0,
            log_seen: 0,
            slot: None,
            panels: Vec::new(),
            saved: (0, 0),
            parked: Vec::new(),
            trail_out: Vec::new(),
            end_logged: false,
            viewing: false,
            parent: Value::Null,
            tokens: Tokens::default(),
            clock: Clock::default(),
            clock_at: Instant::now(),
            astir_at: Instant::now(),
            impacts: Vec::new(),
            gap_until: None,
            req_cmd: String::new(),
            view_seen: None,
            fx: Fx::default(),
            camera: Camera::default(),
            phases_seen: 0,
            sidesteps: Vec::new(),
            rev: 0,
            talk: None,
            talks_seen: 0,
            talk_gs: None,
            auto_after_talk: false,
            faces: Faces::default(),
            panel_faces: Faces::default(),
            timeline_faces: Faces::capped(aegis_ui::portrait_fx::Tier::Minimal),
            portrait_start: false,
            portrait_round: 0,
            portrait_outcome: String::new(),
            portrait_popped: None,
            talk_node: String::new(),
            cutscene: 0,
            player_target: Vec::new(),
            member_tactic: Default::default(),
            deferred: Vec::new(),
            ghosted: 0,
            preview: None,
            deferred_view: None,
            positions_shown: Vec::new(),
            arrows_shown: Vec::new(),
            focus_shown: Vec::new(),
            menu_row: 11,
            inspect: None,
            ap_anim: None,
            round_ap: (0, Vec::new()),
            ap_gauge: None,
            strip_labels: Vec::new(),
            tv_open: false,
            tv_shown: None,
            tracers: Vec::new(),
            traced: None,
            play_block: None,
            block_ended: None,
            waiting: None,
            card_ap: Vec::new(),
            cards_shown: Vec::new(),
            banner: None,
            round_note: None,
            advanced_at: None,
        }
    }

    /// A session worth recording: it starts with `new`, so it replays (a party battle of the world map starts outside
    /// it: PLAYTEST2).
    fn recordable(&self) -> bool {
        !self.viewing && self.host.history.first().is_some_and(|r| r["cmd"] == json!("new"))
    }

    /// Hands the battle's command-host session to the playtest hub when it changed since the last save (the hub saves
    /// it in place while the battle runs).
    fn save_session(&mut self, cx: &mut Cx) {
        for (mut slot, s) in self.parked.drain(..) {
            cx.playtest.put_session(&mut slot, s);
        }
        if let Some(s) = self.host.session.as_ref().filter(|s| s.combat_over && !self.end_logged) {
            self.end_logged = true;
            self.trail_out.push(json!({"type": "battle_end", "round": s.current_round, "result": s.outcome}));
        }
        for e in self.trail_out.drain(..) {
            cx.playtest.trail(e);
        }
        let now = (self.host.history.len(), self.host.checkpoints.len());
        if !self.recordable() || now == self.saved {
            return;
        }
        self.saved = now;
        let s = self.recorded();
        cx.playtest.put_session(&mut self.slot, s);
    }

    /// The command-host session as the recording keeps it (a take-over names its parent in `end`).
    fn recorded(&self) -> aegis_host::playtest::Session {
        let mut s = aegis_host::playtest::Session::of(&self.host);
        if !self.parent.is_null() {
            s.end["parent"] = self.parent.clone();
        }
        s
    }

    /// The playtest trail events of one request the player made: plan edits (add, undo, clear, plan, preset,
    /// preset_all with the unit and round) and confirms (ok, or refused with the reason).
    fn trail_of(&mut self, req: &Value, reply: &Value) {
        let Some(s) = self.host.session.as_ref() else { return };
        let ok = reply["ok"] == json!(true);
        let kind = match req["cmd"].as_str().unwrap_or("") {
            "add_step" => "add",
            "undo" => "undo",
            "clear" => "clear",
            "plan" => "plan",
            "preset" if req.get("unit").is_some() => "preset",
            "preset" => "preset_all",
            "confirm" => {
                let reason = if ok { String::new() } else { reply["error"].as_str().unwrap_or("refused").to_string() };
                self.trail_out.push(json!({"type": "confirm", "ok": ok, "reason": reason, "round": s.current_round}));
                return;
            }
            _ => return,
        };
        if ok {
            self.trail_out.push(json!({"type": "plan_edit", "kind": kind, "unit": req.get("unit").cloned().unwrap_or(Value::Null), "round": s.current_round}));
        }
    }

    /// A new battle starts a new session.
    fn new_session(&mut self) {
        self.slot = None;
        self.saved = (0, 0);
        self.end_logged = false;
        self.viewing = false;
        self.parent = Value::Null;
    }

    /// The Test Arena: the catalog encounter given on the command line (goblin_skirmish by default).
    pub fn sandbox(cx: &Cx) -> Self {
        let mut b = Self::blank(cx, Mode::Arena);
        b.start_battle();
        b
    }

    /// The arena on any encounter (Scene Runner): world None = a catalog encounter, else that world's encounter
    /// played with its authored party of world characters (WorldEncounter).
    pub fn arena(cx: &Cx, world: Option<String>, encounter: &str) -> Self {
        Self::arena_with(cx, world, encounter, None, "", false)
    }

    /// `arena` played by `ruleset` (classic or timeline; None = the encounter's own: RULESET1 3.1.3) with the extensions
    /// `rules` switches ("-id,+id"; "" = none: 3.2) and, with `plan_reveal`, every enemy plan shown (V2).
    pub fn arena_with(cx: &Cx, world: Option<String>, encounter: &str, ruleset: Option<String>, rules: &str, plan_reveal: bool) -> Self {
        Self::arena_full(cx, world, encounter, ruleset, rules, "", plan_reveal)
    }

    /// `arena_with` with the Scene Runner's "Diagonal melee" pick (INPUT1 MR11: none, maneuver or all; "" = the
    /// ruleset row's value), kept on Restart.
    pub fn arena_full(cx: &Cx, world: Option<String>, encounter: &str, ruleset: Option<String>, rules: &str, diagonal: &str, plan_reveal: bool) -> Self {
        let mut b = Self::blank(cx, Mode::Arena);
        b.encounter = encounter.to_string();
        b.arena_world = world;
        b.arena_ruleset = ruleset;
        b.arena_rules = rules.to_string();
        b.arena_diagonal = diagonal.to_string();
        b.arena_reveal = plan_reveal;
        b.start_battle();
        b
    }

    /// A recorded battle replayed into the arena (Scene Runner Playtests). `take_over` None plays it back to its end
    /// to watch (not recorded again); Some((parent, at)) stops after `at` requests and hands the battle to the player
    /// as a new session that names `parent` (recording id, battle, request) in its end. Restart then starts the
    /// encounter afresh with the recorded seed. The message says whether every checkpoint replayed as recorded.
    pub fn playback(cx: &Cx, session: &aegis_host::playtest::Session, take_over: Option<(Value, usize)>) -> Self {
        let mut b = Self::blank(cx, Mode::Arena);
        let first = session.requests.first().cloned().unwrap_or(Value::Null);
        let encounter = first["encounter"].as_str().unwrap_or("").to_string();
        (b.arena_world, b.encounter) = match encounter.split_once(':') {
            Some((w, e)) => (Some(w.to_string()), e.to_string()),
            None => (None, encounter),
        };
        b.seed = first["seed"].as_i64();
        b.arena_ruleset = first["ruleset"].as_str().filter(|r| !r.is_empty()).map(String::from);
        b.arena_rules = first["rules"].as_str().unwrap_or("").to_string();
        b.arena_reveal = first["plan_reveal"].as_bool().unwrap_or(false);
        b.arena_diagonal = first["melee_diagonal"].as_str().unwrap_or("").to_string();
        let until = take_over.as_ref().map(|(_, at)| *at);
        let report = aegis_host::playtest::replay(&mut b.host, session, until);
        if b.host.session.is_none() {
            b.message = match &report.broken {
                Some((i, e)) => format!("The recording does not replay: request {} was refused ({e}).", i + 1),
                None => "The recording has no battle.".into(),
            };
            return b;
        }
        let verdict = match (&report.broken, report.matched()) {
            (Some((i, e)), _) => format!("Replay stopped at request {}: {e}.", i + 1),
            (None, true) => format!("Replayed {} requests as recorded.", report.sent),
            (None, false) => format!("Replayed {} requests; the battle diverged from the recording (the data changed?).", report.sent),
        };
        b.new_session();
        match take_over {
            None => b.viewing = true,
            Some((parent, _)) => b.parent = parent,
        }
        b.after_start("");
        // INPUT1 GH2: the replayed round keeps the ghosts it recorded (no new draft)
        b.ghosted = b.session().current_round;
        b.message = format!("{verdict} {}", if b.viewing { "Watching (not recorded); Restart plays it afresh." } else { "Your battle from here." });
        b
    }

    /// A world fight with the saved party (BattleBridge.encounter_for + GameEncounter): the kit's `begin_world` builds
    /// it (W.2-S2: the fight world, BK6's party_min refusal, the auto-fill, the ruleset, grants, the one hero builder,
    /// the foes, the start-AP change); the screen gives the heroes their art and logs the start's lines.
    pub fn world(cx: &mut Cx, fight: &Dict, runner: &str) -> Result<Self, String> {
        let d = cx.data.clone();
        let started = emberwake_world::battle::begin_world_with(&d, cx.gs_mut(), fight, &mut |world, m| member_art(&d, world, m))?;
        let emberwake_world::battle::WorldFight { session, world, members, stash_short, mods, intro, .. } = started;
        // CAMPAIGN1 P3, W6: at zero upkeep the party starts with less AP (read before the session, logged once)
        if !mods.note.is_empty() {
            cx.log(mods.note.clone());
        }
        let mut b = Self::blank(cx, Mode::World { fight: fight.clone(), runner: runner.to_string(), members });
        b.class_world = world;
        b.host.begin(session);
        b.host.restart();
        b.after_start(&intro);
        if !mods.note.is_empty() {
            b.message = format!("{} {}.", b.message, mods.note);
        }
        for line in stash_short {
            cx.log(line);
        }
        cx.set_busy(true);
        Ok(b)
    }

    fn session(&self) -> &Session {
        self.host.session.as_ref().expect("a battle is running")
    }

    fn world_mode(&self) -> bool {
        matches!(self.mode, Mode::World { .. })
    }

    /// Sends one request through the command API and turns HP changes into floating numbers.
    fn send(&mut self, cx: &Cx, req: Value) -> Value {
        self.flush(cx);
        self.req_cmd = req["cmd"].as_str().unwrap_or("").to_string();
        self.note_step();
        let reply = self.host.handle(&req);
        if cx.playtest.on {
            self.trail_of(&req, &reply);
        }
        if reply["ok"] != json!(true) {
            self.message = reply["error"].as_str().unwrap_or("refused").to_string();
        }
        self.after_change(cx);
        reply
    }
}

impl Screen<GameCx> for BattleScreen {
    /// A4.2-K1b: while the selected hero plans a Field round the arrows aim its source (`field_nudge`), so they do not
    /// move the keyboard focus then; any other time they do, as before.
    fn arrow_focus(&self) -> bool {
        !self.field_aiming()
    }

    /// FPSCAP1: a battle waiting for the player moves slowly (idle clips, portraits, status loops); any other time every
    /// frame. `REST_AFTER` lets the last clips and reactions of a playback run at full rate.
    fn frame_need(&self, _cx: &Cx) -> FrameNeed {
        if self.at_rest() && self.astir_at.elapsed() >= REST_AFTER {
            FrameNeed::Within(SLOW_LIFE)
        } else {
            FrameNeed::Full
        }
    }

    fn id(&self) -> &'static str {
        if self.world_mode() {
            "battle"
        } else {
            "test_arena"
        }
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        self.mouse = mouse;
        self.resolve_impacts(cx);
        self.react_start_portraits(cx);
        self.follow_round_banner();
        draw(self, cx, g, mouse, live);
    }

    fn leave(&mut self, cx: &mut Cx) {
        self.save_session(cx);
    }

    /// A playtest note over the battle records its session (the recording's battle index), round, block and phase.
    fn note_context(&self, _cx: &Cx) -> Value {
        let Some(s) = self.host.session.as_ref() else { return Value::Null };
        let phase = match s.phase {
            Phase::Planning => "planning",
            Phase::Resolving => "resolving",
            Phase::Replay => "replay",
        };
        json!({"battle": self.slot.map_or(-1, |i| i as i64), "round": s.current_round, "block": s.current_block, "phase": phase})
    }

    fn tick(&mut self, cx: &mut Cx) {
        let now = Instant::now();
        if !self.at_rest() {
            self.astir_at = now;
        }
        // SPRITE1-U, FX1-L1 3.1: the tokens and effects run on the FX clock at the animation speed setting (FxSettings
        // 0.5-2x; at most 100 wall ms a frame), which a hit-stop holds still.
        // INPUT1 PB3: at 0.5x the playback's clock runs at half the animation speed
        let speed = cx.settings.v["anim_speed"].as_f64().unwrap_or(1.0).clamp(0.5, 2.0) / self.pace(cx) as f64;
        self.clock.advance((now.duration_since(self.clock_at).as_secs_f64() * 1000.0).min(100.0), speed);
        self.clock_at = now;
        // PORTRAITFX1 4.5: the talk box's portrait clock moves by the frame's wall time
        self.faces.tick();
        // Rule F: the skip setting turning on or another battle view shows the pending impacts at once.
        let view = View::of_setting(&cx.settings.v["battle_view"]);
        if cx.settings.v["skip_anim"].as_bool().unwrap_or(false) || self.view_seen.is_some_and(|v| v != view) {
            self.flush(cx);
            // the live effects are in the old view's px; skip animations draws none (4.6)
            self.fx.clear();
        }
        self.view_seen = Some(view);
        self.fire_due(cx);
        // Slice 2: done effects leave the live list; a shake whose time is up ends.
        let fx_now = self.clock.now();
        self.fx.prune(fx_now);
        self.camera.offset(fx_now);
        self.sidesteps.retain(|s| fx_now < s.1 + fx::SIDESTEP_OUT_MS + fx::SIDESTEP_BACK_MS);
        // INPUT1 PB1: a tracer is gone after its time
        self.tracers.retain(|t| fx_now < t.1 + TRACER_MS);
        // FACE1: the Show enemy tells setting changed (I or the settings): the planning facing follows it. WINDUP1: so
        // do the planning poses, the bodies' pose clips once known and the reduce motion setting.
        let tells = self.tokens.tells != cx.settings.v["show_tells"].as_bool().unwrap_or(true);
        let still = self.tokens.still != cx.settings.reduce_motion();
        if self.learn_poses(cx) || tells || still {
            self.sync_anims(cx);
        }
        if now.duration_since(self.tuning_checked) > Duration::from_millis(500) {
            self.tuning_checked = now;
            if cx.tuning.reload_if_changed() {
                self.message = "Tuning reloaded.".into();
            }
        }
        let t = cx.tuning.value.clone();
        self.floats.retain(|f| now.duration_since(f.born) < Duration::from_millis(f.ms));
        self.flashes.retain(|f| now.duration_since(f.1) < Duration::from_millis(t.hit_flash_ms));
        // The playtest session is saved after every concluded round (and once more at the end).
        let ended = self.host.session.as_ref().is_some_and(|s| s.combat_over) && !self.end_logged;
        if self.host.checkpoints.len() != self.saved.1 || !self.parked.is_empty() || !self.trail_out.is_empty() || ended {
            self.save_session(cx);
        }
        if self.host.session.is_none() {
            return;
        }
        // ENCOUNTER2-C DT5: a talk open (or about to open) holds the playback and autoplay
        if self.talk_tick(cx, now) {
            return;
        }
        // INPUT1 GH2, GH7: a round's planning start drafts the ghosts; the Auto-fill setting reaches the battle (not
        // while watching a recording)
        if !self.viewing && ri::on(&self.session().w) && self.planning() {
            if self.ghosted != self.session().current_round {
                self.plan_start(cx);
            } else {
                let on = cx.settings.v["auto_fill"].as_bool().unwrap_or(true);
                if self.session().w.autofill != on {
                    self.sync_autofill(cx, on);
                }
            }
        }
        // ENCOUNTER2-C4: a locked round's playback starts by itself, once a round-start talk has closed (not while
        // watching a recording: its own requests play it)
        if !self.viewing && self.locked() && self.session().phase != Phase::Replay && self.cutscene != self.session().current_round {
            let s = self.session();
            let (round, banner) = (s.current_round, s.round_script().map(|r| r.banner.clone()).unwrap_or_default());
            self.cutscene = round;
            self.message = if banner.is_empty() { format!("Round {round}: the round plays out (Skip resolves it).") } else { format!("{banner} (Skip resolves the round.)") };
            if !self.playing {
                self.playing = true;
                self.next_at = now;
            }
        }
        // FX1 rule P: the next step waits for `step_ms` and for the last impact plus `impact_gap_ms`.
        let ready = now >= self.next_at && self.impacts_done(now);
        // INPUT1 PB3: at 0.5x the pacing between steps and rounds doubles
        let pace = self.pace(cx);
        // VISCLAR1-D: the player went ahead during the pause (Space, Next round): the wait is over
        if self.pending_next_round && self.session().phase != Phase::Replay {
            self.pending_next_round = false;
        }
        if self.pending_next_round {
            if ready {
                // PB3: Block mode holds autoplay after the round's last block too
                if self.hold_at_block_end(cx) {
                    return;
                }
                // VISCLAR1-D: a menu the player has open is not closed under them
                if self.popup.is_some() && !self.auto {
                    return;
                }
                self.pending_next_round = false;
                // the round's summary stays readable in the message bar as the next round starts (`summary_and_prompt`)
                let summary = Some(self.message.clone()).filter(|m| m.starts_with("Round over"));
                self.run(cx, Action::NextRound);
                self.round_note = summary.filter(|_| self.planning()).map(|m| (self.message.clone(), m));
            }
            return;
        }
        if self.playing && ready {
            if self.session().phase == Phase::Replay {
                // VISCLAR1-D: a Space during the playback may have played the round's last step (classic stays as it was)
                self.playing = false;
                if ri::on(&self.session().w) {
                    self.round_over(cx, now);
                }
                return;
            }
            // PB3, PB5: after a block the playback waits in Block mode and while the timeline view is open
            if self.hold_at_block_end(cx) {
                return;
            }
            self.send(cx, json!({"cmd": "step"}));
            self.next_at = now + Duration::from_millis(t.step_ms * pace);
            if self.session().phase == Phase::Replay {
                self.playing = false;
                self.round_over(cx, now);
            }
        }
    }

    fn key(&mut self, cx: &mut Cx, k: Key) {
        if crate::viewkeys::window_key(cx, k, crate::viewkeys::Shown::Notice) {
            return;
        }
        if self.host.session.is_none() {
            return;
        }
        // ENCOUNTER2-C DT5: the talk box's keys (1-9, Enter, Space) only
        if self.talk.is_some() {
            let pick = self.talk_state_of(cx).and_then(|gs| self.talk_view(cx, &gs)).and_then(|v| talk::talk_key(k, &v));
            if let Some(a) = pick {
                self.act(cx, Action::Talk(a));
            }
            return;
        }
        match k {
            Key::Enter if self.planning() => self.act(cx, Action::Confirm),
            Key::Enter if !self.outcome().is_empty() && self.world_mode() => self.act(cx, Action::Continue),
            Key::Space => self.act(cx, Action::Step),
            Key::PageUp if self.show_log => self.log_scroll += 10,
            Key::PageDown if self.show_log => self.log_scroll = (self.log_scroll - 10).max(0),
            Key::Char('a') => self.act(cx, Action::Autoplay),
            Key::Char('z') => self.act(cx, Action::Undo),
            Key::Char('l') => self.act(cx, Action::ToggleLog),
            Key::Char('i') => {
                let on = cx.settings.v["show_tells"].as_bool().unwrap_or(true);
                cx.settings.v["show_tells"] = serde_json::Value::Bool(!on);
                self.message = match cx.settings.save() {
                    Err(e) => format!("Tells toggled but the setting was not saved: {e}"),
                    Ok(()) if on => "Enemy tells hidden (I shows them).".into(),
                    Ok(()) => "Enemy tells shown (I hides them).".into(),
                };
            }
            Key::Char('r') => self.act(cx, Action::Restart),
            // INPUT2: keys 1-4 select the party's 1st to 4th member (the party cards' order)
            Key::Char(c @ '1'..='4') => self.select_member(c as usize - '0' as usize),
            // INPUT1 HK1-HK4, INPUT2: Shift+1-9 are the selected hero's skill hotkeys (classic: nothing)
            Key::ShiftDigit(d @ 1..=9) if self.planning() => self.hotkey(cx, d as usize),
            // INPUT2: F accepts the selected hero's auto-filled blocks as picks
            Key::Char('f') if self.planning() => self.act(cx, Action::FillGhost),
            // SL4: S targets the selected hero itself (the self menu)
            Key::Char('s') if self.planning() => self.act(cx, Action::SelfTarget),
            // INPUT1 TV1: T opens and closes the full timeline view (a timeline battle, planning and playback)
            Key::Char('t') => self.act(cx, Action::TimelineView),
            // INPUT1 PB3: P cycles the playback speed (1x, 0.5x, Block), N plays the next block (timeline battles)
            Key::Char('p') => self.act(cx, Action::PlaybackSpeed((PlaySpeed::of(&cx.settings.v).index() + 1) % PLAYBACK_SPEEDS.len())),
            Key::Char('n') => self.act(cx, Action::NextBlock),
            // A4.2-K1b: the arrows (`Key::dir`; the letters W A S D keep their battle keys) move the selected hero's
            // Field source one tile
            Key::Up | Key::Down | Key::Left | Key::Right if self.field_aiming() => {
                let (dx, dy) = k.dir().unwrap_or((0, 0));
                self.field_nudge(cx, emberwake_battle::grid::V2::new(dx as i32, dy as i32));
            }
            _ => {}
        }
    }

    fn back(&mut self, cx: &mut Cx) {
        if self.talk.is_some() {
            return;
        }
        // INPUT1 IF10, SL2: an open menu only closes; a right click on a unit (token or card) inspects it; elsewhere it
        // undoes the last pick
        if self.popup.is_some() {
            self.popup = None;
        } else if self.move_aim.take().is_some() {
            self.move_tiles.clear();
            self.message = "Move cancelled.".into();
        } else if self.show_log {
            self.show_log = false;
        } else if let Some(u) = self.unit_under_mouse() {
            self.inspect_unit(u);
        } else if self.host.session.is_some() && self.planning() {
            self.act(cx, Action::Undo);
        }
    }

    fn escape(&mut self, cx: &mut Cx) {
        if self.talk.is_some() {
            return;
        }
        // INPUT1 SL3 (round intent): the menu, the log, the inspected unit, then the selection; then as before. TV1: an
        // open timeline view closes first
        let timeline = self.host.session.as_ref().is_some_and(|s| ri::on(&s.w));
        if self.tv_open {
            self.close_view(cx);
        } else if self.popup.is_some() {
            self.popup = None;
        } else if self.move_aim.take().is_some() {
            self.move_tiles.clear();
            self.message = "Move cancelled.".into();
        } else if self.show_log {
            self.show_log = false;
        } else if self.inspected().is_some() {
            self.inspect = None;
            self.message = "Inspect cleared.".into();
        } else if timeline && self.selected.is_some() {
            self.selected = None;
        } else if !self.world_mode() {
            cx.nav.push(Nav::Pop);
        }
    }

    fn wheel(&mut self, _cx: &mut Cx, lines: i32) {
        if self.show_log {
            self.log_scroll = (self.log_scroll + lines * 3).max(0);
        }
    }

    fn state(&self, cx: &Cx) -> Value {
        let mut out = json!({"message": self.message, "playing": self.playing, "auto": self.auto, "log_open": self.show_log, "world": self.world_mode(),
            "fx": self.fx_state(), "tokens": self.tokens.state(),
            "outcome": if self.host.session.is_some() { self.outcome() } else { String::new() },
            "popup": self.popup.as_ref().map(|p| json!({"title": p.title, "rows": p.rows.iter().map(|r| json!({"label": r.label, "detail": r.detail, "valid": r.valid})).collect::<Vec<_>>()}))});
        // RULESET1-G: the rules the battle plays by, beyond classic (a classic battle's state keeps its shape)
        if let Some(s) = self.host.session.as_ref().filter(|s| !s.w.ruleset.rules.is_empty()) {
            out["ruleset"] = json!({"id": s.w.ruleset.id, "rules": s.w.ruleset.rules, "plan_reveal": s.w.plan_reveal, "melee_diagonal": s.w.ruleset.melee_diagonal().id()});
            // RULESET1-U: the Arena fight's ruleset chip as drawn
            if let Some(chip) = self.arena_chip() {
                out["ruleset"]["chip"] = json!(chip);
            }
        }
        // INPUT1 1.6: each party member's picks (round intent only), the ghosts waiting for Confirm and the player-set
        // targets (GH2)
        if let Some(sess) = self.host.session.as_ref().filter(|x| ri::on(&x.w)) {
            let w = &sess.w;
            out["picks"] = Value::Array(w.allies.iter().map(|&a| json!({"unit": a, "picks": rip::picks_of(w, a).iter().map(|p| pick_json(w, p)).collect::<Vec<_>>()})).collect());
            out["deferred"] = json!(self.deferred.iter().map(|d| d.0).collect::<Vec<_>>());
            out["player_target"] = json!(self.player_target);
            // IS1: the unit the intel panel is pinned on (null: it follows the hover)
            out["inspect"] = json!(self.inspected());
            // INPUT1-AP: the AP gauge as last drawn (pips "F H Hg Sg S", red, its "-<n>" label and side; null: hidden)
            out["ap_gauge"] = self.ap_gauge.clone().unwrap_or(Value::Null);
            out["strip_labels"] = json!(self.strip_labels);
            // VISCLAR1-D: the party cards as last drawn and the round banner showing (null: none)
            out["party_cards"] = json!(self.cards_shown);
            out["round_banner"] = json!(self.round_banner_now(cx).map(|b| json!({"round": b.0, "alpha": b.1})));
        }
        // INPUT1 PB1-PB4: the playback aids (timeline battles): the speed, whether the playback waits at a block's end
        // and the resolving block; the live tracers; the log by round and block
        if self.playback_aids() {
            let sess = self.session();
            out["playback"] = json!({"speed": PLAYBACK_SPEEDS[PlaySpeed::of(&cx.settings.v).index()].0, "waiting": self.waiting.is_some(), "block": self.cursor_block(Instant::now())});
            out["log_groups"] = log_groups_json(&sess.log_groups());
        }
        // INPUT1 TV1: the full timeline view as last drawn (null: closed; no key in classic)
        if self.timeline_view_on() {
            out["timeline_view"] = self.tv_shown.as_ref().filter(|_| self.timeline_view_open()).map(timeline_view::json).unwrap_or(Value::Null);
        }
        // RULESET1-U slice 5: the overlay groups' flags and, once the round is over, its summary (round intent only)
        if let Some(sess) = self.host.session.as_ref().filter(|x| ri::on(&x.w)) {
            out["overlay_layers"] = Layers::of(&cx.settings.v["battle_overlays"]).json();
            if sess.phase == Phase::Replay {
                let sum = ri::round_summary(&sess.w, &sess.resolver.scheduled);
                out["round_summary"] = json!({"breaks": sum.breaks, "windup_breaks": sum.windup_breaks, "windup_hits": sum.windup_hits, "flanks": sum.flanks, "follow_ups": sum.follow_ups, "rest_ap": sum.rest_ap});
            }
        }
        // HOOKS1-UI slice 1: the hook marks as drawn, while the battle plays by ground or arena hazards (classic: no key)
        if let Some(sess) = self.host.session.as_ref().filter(|x| emberwake_battle::ground::on(&x.w) || emberwake_battle::arena::on(&x.w)) {
            out["hook_marks"] = HookMarks::of(sess, &Layers::of(&cx.settings.v["battle_overlays"]), cx.held.alt).json();
        }
        // ENCOUNTER2-U: a hold escape's state (key only in a hold escape)
        if let Some(sess) = self.host.session.as_ref() {
            if let Some(gate) = &sess.w.hold {
                let cell = |c: &V2| json!([c.x, c.y]);
                out["hold"] = json!({"label": gate.label, "tiles": gate.tiles.iter().map(cell).collect::<Vec<_>>(), "holder": gate.holder, "open": gate.is_open(&sess.w),
                    "needs_pick": sess.needs_stay_pick(), "note": sess.gate_note(), "pick": hold_pick_ids(sess)});
            }
        }
        // HOOKS1-UI slice 4: the selected hero's push forecasts as drawn (key only while forced_move is on)
        if let Some(sess) = self.host.session.as_ref().filter(|x| emberwake_battle::forced::on(&x.w)) {
            let layers = Layers::of(&cx.settings.v["battle_overlays"]);
            let planning = sess.phase == Phase::Planning && !sess.plan_confirmed;
            let list: Vec<Value> = match self.selected.filter(|u| sess.w.allies.contains(u) && sess.w.units[*u].is_active()) {
                Some(h) if planning && layers.pushes => push_forecasts(sess, h).into_iter().flatten().map(|f| push_json(&f)).collect(),
                _ => Vec::new(),
            };
            out["push_forecasts"] = Value::Array(list);
        }
        // HOOKS1-UI slice 3: the meters and held guards as drawn (keys only while their rules are on)
        if let Some(sess) = self.host.session.as_ref() {
            let layers = Layers::of(&cx.settings.v["battle_overlays"]);
            if emberwake_battle::meter::on(&sess.w) {
                out["meters"] = Value::Array((0..sess.w.units.len()).filter_map(|i| {
                    let ms = meters_of(&sess.w, i, &layers);
                    (!ms.is_empty()).then(|| json!({"unit": i, "meters": ms.iter().map(|m| json!({"id": m.id, "value": m.value, "min": m.min, "max": m.max, "show": m.show, "band": m.band})).collect::<Vec<_>>()}))
                }).collect());
            }
            if emberwake_battle::trigger::on(&sess.w) {
                out["held_guards"] = Value::Array((0..sess.w.units.len()).filter(|_| layers.guard).filter_map(|i| held_guard(sess, i).map(|g| json!({"unit": i, "mitigation": g.mitigation, "last_block": g.last_block}))).collect());
            }
        }
        // RULESET1-U slice 3: the board overlays under the round intent (classic: no key)
        if let Some(mut o) = self.host.session.as_ref().and_then(|s| overlay_of(s, self.selected.filter(|u| s.w.allies.contains(u)), s.phase == Phase::Planning && !s.plan_confirmed)) {
            o.positions = self.positions_shown.clone();
            o.arrows = self.arrows_shown.clone();
            out["overlay"] = overlay_json(&o);
            // VISCLAR1 VC2: the units whose token bars the last frame drew by default
            out["focus"] = json!(self.focus_shown);
        }
        // INPUT1 PB1: the impact tracers drawing now (any timeline battle)
        if self.playback_aids() {
            if !out["overlay"].is_object() {
                out["overlay"] = json!({});
            }
            out["overlay"]["tracers"] = self.tracers_json(cx);
        }
        // ENCOUNTER2-C4: the scripted round (key only while one plays): its round, lock and banner, and whether Skip shows
        if let Some(rs) = self.host.session.as_ref().and_then(|s| s.round_script()) {
            out["script"] = json!({"round": rs.round, "lock": self.locked(), "banner": rs.banner, "skip": self.can_skip()});
        }
        // ENCOUNTER2-C DT11: the open talk (key only while one is open), its portrait as drawn and the speaker's token
        if let (Some(t), Some(gs)) = (self.talk.as_ref(), self.talk_state_of(cx)) {
            let view = t.runner.view(&cx.data, &gs);
            out["talk"] = talk::talk_state(&t.dialogue, &view);
            out["talk"]["portraits"] = self.faces.state();
            out["talk"]["speaker_unit"] = json!(self.speaker_unit(cx, &gs, &s(&view, "speaker")));
        }
        // PORTRAITFX1-P (4.11): the panel portraits kept (the hero and intel panels'), so a test or an agent can wait
        // for a reaction to end
        out["portraits"] = self.panel_faces.state();
        // PORTRAITFX1-TL: the timeline view's track icons kept (empty while the view is closed: rows react only while it
        // is open and `begin` drops the rest at the next draw)
        out["timeline_portraits"] = self.timeline_faces.state();
        out
    }

    fn command(&mut self, cx: &Cx, body: &Value) -> Option<Value> {
        // An agent-link `new` ends this session: park it for the hub (saved on the next tick), then start a new one.
        let restart = body["cmd"] == json!("new");
        if restart && self.recordable() {
            self.parked.push((self.slot, self.recorded()));
        }
        self.flush(cx);
        self.req_cmd = body["cmd"].as_str().unwrap_or("").to_string();
        self.note_step();
        let reply = self.host.handle(body);
        if restart && reply["ok"] == json!(true) {
            self.new_session();
            self.tokens = Tokens::default();
            self.reset_talks();
            self.card_ap.clear();
            self.banner = None;
            self.round_note = None;
            self.advanced_at = None;
        }
        if self.host.session.is_some() {
            self.after_change(cx);
        }
        Some(reply)
    }

    /// MUSIC1: boss when an enemy unit has the boss tier, else combat; scoped to the battle's world and location.
    fn music(&self, cx: &Cx) -> Option<String> {
        let ctx = if self.host.session.as_ref().is_some_and(boss_fight) { music::BOSS } else { music::COMBAT };
        match &self.arena_world {
            Some(w) => music::track(cx, ctx, w, ""),
            None if self.world_mode() => music::game_track(cx, ctx),
            None => music::track(cx, ctx, "", ""),
        }
    }

    aegis_ui::widget_plumbing!();
}
