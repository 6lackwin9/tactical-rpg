//! The battle screen (COMBAT-UI3 layout A). Two layers: the battlefield is pixel art scaled with nearest sampling;
//! panels, text, hover cards and menus are drawn smooth at the window's real resolution (painter.rs). Every change goes
//! through the command API (emberwake_host on aegis_host), the same one `aegis serve` and the agent link use.
//! Two modes: the Test Arena (a catalog encounter, Restart / Play again) and a world battle (GameEncounter: the saved
//! party at its stored HP against a world encounter; on Continue the HP is written back, the runner's after_fight
//! settles the outcome and the results popup opens over the map).
//! Mouse: click a hero to select; click a foe, an ally, the hero itself or a tile for its options; hover anything for
//! details; right click closes a menu or undoes the hero's last step; the wheel scrolls the log. Keys: Enter confirm,
//! Space step / next round, A autoplay, Z undo, L log, R restart (arena), Esc close.

use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use crate::music;
use crate::intent_text;
use crate::stage::{self, Geom, Item, Pass, Through, View};
use crate::tuning::Tuning;
use crate::screens::{show_errors, Cx, GameCx, Key, Nav, Screen};
use emberwake_battle::intent::Intent;
use aegis_ui::audio::Bus;
use aegis_ui::painter::Painter;
use aegis_ui::ui::{Gfx, Tip, Ui, Widget, BODY, H, RADIUS, SMALL, TITLE, W};
use emberwake_battle::command::CmdKind;
use emberwake_battle::grid::{V2, GRID_H, GRID_W};
use emberwake_battle::plan::expected;
use emberwake_battle::session::{EncounterDef, Phase, Prebuilt, Session, StartGrant};
use aegis_core::assets::{ClipFrame, ClipPlayer};
use aegis_ui::art::Art;
use aegis_ui::canvas::Image;
use aegis_core::json::{self, s, Dict};
use emberwake_rules::Stance;
use emberwake_world::battle::WorldRoster;
use emberwake_battle::skill::Kind;
use emberwake_battle::world::{AiProfile, World};
use emberwake_data::UnitDef;
use emberwake_host::{Battle, Host};
use emberwake_world::{dialogue, explore, progression, travel};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

/// The Flat view's cell (and the grid panel's: 12 x 9 cells of `TILE` at (`GRID_X`, `GRID_Y`)).
const TILE: i32 = 32;
/// The battle screen's panels by name (`BattleScreen::panels`; the UI Gallery's battle panel entries).
pub const PANELS: [&str; 7] = ["strip", "party", "hero", "grid", "hostiles", "intel", "bar"];
const GRID_X: i32 = 128;
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

const BG: Rgba = rgb(10, 14, 24);
const PANEL: Rgba = rgba(22, 32, 54, 245);
const PANEL_HI: Rgba = rgba(34, 50, 82, 250);
const CARD: Rgba = rgba(16, 24, 42, 250);
const BORDER: Rgba = rgb(58, 78, 116);
const TEXT: Rgba = rgb(222, 228, 240);
const DIM: Rgba = rgb(132, 146, 172);
const GOLD: Rgba = rgb(244, 198, 84);
const ALLY: Rgba = rgb(110, 170, 255);
const FOE: Rgba = rgb(240, 100, 92);
const GOOD: Rgba = rgb(96, 210, 110);

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Select(usize),
    UnitClick(usize),
    TileClick(V2),
    Choose(usize),
    ClosePopup,
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
}

#[derive(Debug, Clone)]
enum RowAct {
    Step(Value),
    PartyTactic(usize),
    HeroTactic(usize),
}

#[derive(Debug, Clone)]
struct Row {
    label: String,
    detail: String,
    valid: bool,
    act: RowAct,
    tip: Tip,
}

#[derive(Debug, Clone)]
struct Popup {
    title: String,
    x: i32,
    y: i32,
    w: i32,
    rows: Vec<Row>,
}

struct Float {
    unit: usize,
    text: String,
    color: Rgba,
    born: Instant,
}

/// What a unit was when its token last caught up with the battle (SPRITE1-U).
#[derive(Debug, Clone, Default, PartialEq)]
struct Seen {
    pos: Option<V2>,
    hp: i32,
    alive: bool,
    guarded: bool,
}

/// One unit's token animation (SPRITE1-U): its clips, a slide to a new tile, its fall. Times are on the battle's
/// animation clock (`BattleScreen::anim_ms`, scaled by the animation speed setting).
#[derive(Debug, Clone, Default)]
struct UnitAnim {
    player: ClipPlayer,
    /// An action clip to start (candidates best first, `action_clips`) and when; the draw picks the first the body has.
    want: Option<(Vec<&'static str>, u64)>,
    /// The tile it slides from and when the slide started.
    slide: Option<(V2, u64)>,
    /// When it fell (the death clip plays and holds, then the dead marker shows); None while it stands.
    fell: Option<u64>,
    seen: Seen,
    /// FACE1: the facing it keeps (true = left, the sheet frame mirrored; the rig's sheets face right).
    left: bool,
    /// FACE1: while the party plans, the side of its planned or intended target (None: no target, or one in its column).
    planned: Option<bool>,
}

impl UnitAnim {
    /// FACE1: the side the token faces now: its planned target's while planning, else the facing it keeps.
    fn faces_left(&self) -> bool {
        self.planned.unwrap_or(self.left)
    }
}

/// FACE1: the facing that looks from `from` toward `to`: Some(true) = left, Some(false) = right, None when `to` is in
/// the same column (the unit keeps its facing).
fn facing_toward(from: V2, to: V2) -> Option<bool> {
    (to.x != from.x).then_some(to.x < from.x)
}

/// The battle's token animations (SPRITE1-U): one per unit, the last popped action already animated and the action
/// log length last seen.
#[derive(Debug, Clone, Default)]
struct Tokens {
    anims: Vec<UnitAnim>,
    popped_seen: Option<(i32, i32, usize)>,
    log_len_seen: usize,
    /// FACE1: the Show enemy tells setting at the last catch-up (hidden tells: enemies do not show their intent).
    tells: bool,
}

impl Tokens {
    /// The tokens catch up with `s` at `now` (the animation clock). While the battle moves forward (the action log
    /// does not shrink) a unit that moved slides there playing `walk` for `walk_ms`, a hit plays `hurt` (`hurt_guarded` while it
    /// guards), a fall plays `death`, raising a guard plays `defend`, and the action just resolved plays its actor's
    /// attack clip (`action_clips`); the base loop is `idle_guarded` while it guards, else `idle`. Anything else (a new
    /// battle, another session) only takes the new state; `instant` (the skip setting) takes the end state at once.
    /// FACE1 (also under `instant`): a unit turns toward the x of its step, then toward the target (or the area's aim)
    /// of the action it just resolved; while the party plans each unit shows its planned or intended target
    /// (`plan_facing`; an enemy only while `tells`, the Show enemy tells setting, is on), and the confirm keeps that
    /// facing. A new battle faces the party right and the foes left.
    fn catch_up(&mut self, s: &Session, now: u64, instant: bool, walk_ms: u64, tells: bool) {
        self.tells = tells;
        let n = s.w.units.len();
        let animate = self.anims.len() == n && s.log.len() >= self.log_len_seen;
        if self.anims.len() != n {
            self.anims = vec![UnitAnim::default(); n];
        }
        // The plan is confirmed (or the battle moved on): each unit keeps the facing its planned target gave it.
        if !s.is_planning() {
            for a in &mut self.anims {
                if let Some(left) = a.planned.take() {
                    a.left = left;
                }
            }
        }
        // Where each unit stood before this change (an action's target may have fallen off the grid since).
        let before: Vec<Option<V2>> = self.anims.iter().map(|a| a.seen.pos).collect();
        for (i, u) in s.w.units.iter().enumerate() {
            let pos = s.w.grid.pos(i);
            let guarded = u.guard > 0.0 || u.stance == Stance::Defensive;
            let seen = Seen { pos, hp: u.hp, alive: u.is_alive(), guarded };
            let a = &mut self.anims[i];
            if animate {
                let was = &a.seen;
                if let (Some(from), Some(to)) = (was.pos, pos) {
                    if let Some(left) = facing_toward(from, to) {
                        a.left = left;
                    }
                    if from != to && !instant && walk_ms > 0 {
                        a.slide = Some((from, now));
                        a.player.play_for("walk", now, walk_ms);
                    }
                }
                if was.alive && !seen.alive {
                    a.fell = Some(now);
                    a.slide = None;
                    if !instant {
                        a.player.play_and_hold("death", now);
                    }
                } else if seen.alive && seen.hp < was.hp && !instant {
                    a.player.play(if guarded { "hurt_guarded" } else { "hurt" }, now);
                } else if seen.alive && guarded && !was.guarded && !instant {
                    a.player.play("defend", now);
                }
            } else {
                a.player.stop();
                a.want = None;
                a.slide = None;
                // Fallen before this view: the dead marker at once.
                a.fell = if seen.alive { None } else { Some(0) };
                a.left = !s.w.allies.contains(&i);
                a.planned = None;
            }
            if seen.alive && a.fell.is_some() {
                a.fell = None;
                a.player.stop();
            }
            a.player.set_base(if guarded { "idle_guarded" } else { "idle" }, now);
            a.seen = seen;
        }
        // Only this round's pop: `scheduled` is rebuilt each round, so an older index names another action.
        let popped = s.last_popped.filter(|p| animate && Some(*p) != self.popped_seen && p.0 == s.current_round);
        if let Some(act) = popped.and_then(|(_, _, idx)| s.resolver.scheduled.get(idx)) {
            let (kind, aim) = match &act.command.kind {
                CmdKind::Attack { target, .. } => ("attack", target.and_then(|t| s.w.grid.pos(t).or(before[t]))),
                CmdKind::Area { aim, .. } => ("area", Some(*aim)),
                CmdKind::Support { target, .. } => ("support", s.w.grid.pos(*target).or(before[*target])),
                CmdKind::Item { target, .. } => ("", s.w.grid.pos(*target).or(before[*target])),
                _ => ("", None),
            };
            let u = &s.w.units[act.unit];
            if !act.cancelled && u.is_alive() {
                let at = s.w.grid.pos(act.unit);
                if let Some(left) = at.zip(aim.filter(|c| *c != emberwake_battle::grid::NO_CELL)).and_then(|(from, to)| facing_toward(from, to)) {
                    self.anims[act.unit].left = left;
                }
                if !kind.is_empty() && !instant {
                    let skill = s.w.cat.skills.iter().find(|k| k.name == act.command.name).map_or("", |k| k.id.as_str());
                    let weapon = u.inventory.main_weapon().map_or("", |w| w.weapon_type());
                    self.anims[act.unit].want = Some((action_clips(skill, kind, weapon), now));
                }
            }
        }
        self.plan_facing(s);
        self.popped_seen = s.last_popped;
        self.log_len_seen = s.log.len();
    }

    /// FACE1: while the party plans, each unit shows the side of its target: an enemy its tell's while tells are shown
    /// (INTENT1: the target unit, else the tile it aims at; hidden tells: it keeps its facing until it acts), a hero the first step of its plan that names another unit or an area tile; a
    /// target in its own column, or none, shows the facing it keeps. Once the plan is confirmed the units keep the
    /// facing they showed (`catch_up`).
    fn plan_facing(&mut self, s: &Session) {
        if !s.is_planning() {
            return;
        }
        let side = |u: usize, to: Option<V2>| s.w.grid.pos(u).zip(to.filter(|c| *c != emberwake_battle::grid::NO_CELL)).and_then(|(from, to)| facing_toward(from, to));
        for a in &mut self.anims {
            a.planned = None;
        }
        for r in if self.tells { s.enemy_intents() } else { Vec::new() } {
            let to = r.line.target.and_then(|t| s.w.grid.pos(t)).or(Some(r.line.cell));
            if let Some(a) = self.anims.get_mut(r.unit) {
                a.planned = side(r.unit, to);
            }
        }
        for &h in &s.w.allies {
            let first = s.plan.steps_of(h).into_iter().find_map(|st| match (st.mode, st.target) {
                (emberwake_battle::plan::Mode::Area, _) => Some(st.cell.get()),
                (_, Some(t)) if t != h => s.w.grid.pos(t),
                _ => None,
            });
            if let Some(a) = self.anims.get_mut(h) {
                a.planned = side(h, first);
            }
        }
    }
}

/// Skills with a clip of their own on the sprite rig (SPRITE1 section 2; ids from tools/sprites/classes.py).
const SKILL_CLIPS: [(&str, &str); 5] = [("shield_bash", "shield_bash"), ("offhand_jab", "jab"), ("haymaker", "haymaker"), ("sweep_kick", "sweep_kick"), ("cyclone_kick", "sweep_kick")];
/// Weapon types that fight with fists (the brawler's jab and kicks).
const FIST_WEAPONS: [&str; 3] = ["", "fist", "unarmed"];

/// SPRITE1-U: the clips an action's actor may play, best first (the first one its body has is played; none: no clip).
/// `kind` is the command's: "attack" (one target), "area" or "support"; `weapon` the actor's main weapon type ("" =
/// none). A skill with its own rig clip plays it; fists jab one target and sweep an area; any other attack plays
/// `attack`; a support skill plays `cast` (no rig clip yet).
fn action_clips(skill: &str, kind: &str, weapon: &str) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = SKILL_CLIPS.iter().filter(|(s, _)| *s == skill).map(|(_, c)| *c).collect();
    let fists = FIST_WEAPONS.contains(&weapon);
    match kind {
        "attack" if fists => out.extend(["jab", "attack"]),
        "area" if fists => out.extend(["sweep_kick", "kick", "jab", "attack"]),
        "attack" | "area" => out.push("attack"),
        "support" => out.push("cast"),
        _ => {}
    }
    out
}

enum Mode {
    Arena,
    /// A world fight: the fight dict, the runner that settles it (explore / travel / dialogue) and the member ids of
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
    /// SPRITE1-U: the token animations, the animation clock (ms, advanced at the animation speed setting) and when it
    /// last advanced.
    tokens: Tokens,
    anim_ms: f64,
    anim_at: Instant,
}

impl BattleScreen {
    fn blank(cx: &Cx, mode: Mode) -> Self {
        Self {
            host: Host::new(Battle::with_data(cx.data.clone())),
            w: Vec::new(),
            mouse: (0, 0),
            selected: None,
            popup: None,
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
            anim_ms: 0.0,
            anim_at: Instant::now(),
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
        let mut b = Self::blank(cx, Mode::Arena);
        b.encounter = encounter.to_string();
        b.arena_world = world;
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
        b.message = format!("{verdict} {}", if b.viewing { "Watching (not recorded); Restart plays it afresh." } else { "Your battle from here." });
        b
    }

    /// A world fight with the saved party (BattleBridge.encounter_for + GameEncounter).
    pub fn world(cx: &mut Cx, fight: &Dict, runner: &str) -> Result<Self, String> {
        let d = cx.data.clone();
        let gs = cx.gs();
        let enc_ref = s(fight, "encounter");
        let hit = d.resolve(&gs.world_id, "encounters", &enc_ref);
        if hit.is_empty() {
            return Err(format!("Unknown encounter '{enc_ref}'."));
        }
        let active = gs.party.active_members();
        if active.is_empty() {
            return Err("No active party member to fight.".into());
        }
        let hit_world = s(hit, "_world");
        let world = if d.lineage(&gs.world_id).contains(&hit_world) { gs.world_id.clone() } else { hit_world };
        let entry = d.resolve(&world, "encounters", &s(hit, "id"));
        let mut def = EncounterDef::from_obj(entry);
        let zone = s(fight, "zone");
        if !zone.is_empty() {
            def.zone = zone;
        }
        def.id = format!("{world}:{}", s(hit, "id"));
        // PACE1 rule 7 (GameEncounter): a start_status entry is granted when its requires_flags all hold.
        let grants: Vec<bool> = def.start_status.iter().map(|sb| json::aget(json::dict(sb), "requires_flags").iter().all(|c| emberwake_world::quest::condition_met(gs, json::dict(c)))).collect();
        def.start_grant = StartGrant::Only(grants);
        let roster = WorldRoster { d: &d, world: world.clone() };
        let mut party = Vec::new();
        let mut members = Vec::new();
        for m in active {
            let Some(built) = progression::build(&d, m) else { continue };
            let mut unit = built.unit;
            unit.hp = progression::start_hp(unit.max_hp as i64, m.hp_lost) as i32;
            let base = d.cat.units.get(&m.base_unit).cloned().unwrap_or_else(|| UnitDef::from_obj(&Default::default()));
            // ASSETORG2: the member's character's body and palette (its base unit's, its class's when it has none)
            let character = d.resolve(&world, "characters", &m.lore_ref);
            let (art, palette, _) = if character.is_empty() {
                let unit = d.entry("units", &m.base_unit);
                let class = d.entry("classes", &base.class_id);
                (emberwake_content::art::body_of(&Default::default(), unit, class), emberwake_content::art::palette_of(&Default::default(), unit), Vec::new())
            } else {
                crate::screens::character_art(&d, character)
            };
            party.push(Prebuilt { unit, art, palette, profile: AiProfile::for_unit(&d.cat, &base), key: m.id.clone() });
            members.push((m.id.clone(), built.pouch_unloaded));
        }
        let session = Session::start_with(d.cat.clone(), def, &roster, party);
        let mut b = Self::blank(cx, Mode::World { fight: fight.clone(), runner: runner.to_string(), members });
        b.host.begin(session);
        b.host.restart();
        b.after_start(&s(entry, "intro"));
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

    pub fn after_change(&mut self, cx: &Cx) {
        let now = Instant::now();
        let hps: Vec<i32> = self.session().w.units.iter().map(|u| u.hp).collect();
        if hps.len() == self.last_hp.len() {
            for (i, (&old, &new)) in self.last_hp.iter().zip(hps.iter()).enumerate() {
                if new < old {
                    self.floats.push(Float { unit: i, text: format!("-{}", old - new), color: cx.tuning.value.damage_color, born: now });
                    self.flashes.push((i, now));
                    if new <= 0 {
                        cx.audio.play(Bus::Sfx, "sfx/defeat.wav");
                    }
                } else if new > old {
                    self.floats.push(Float { unit: i, text: format!("+{}", new - old), color: cx.tuning.value.heal_color, born: now });
                }
            }
        }
        self.last_hp = hps;
        let log = &self.session().resolver.log;
        for line in log.iter().skip(self.log_seen) {
            if let Some(key) = sound_of(line) {
                cx.audio.play(Bus::Sfx, &format!("sfx/{key}.wav"));
            }
        }
        self.log_seen = log.len();
        let (n, last) = { let s = self.session(); (s.events.len(), s.events.last().cloned()) };
        if n > self.events_seen {
            self.message = last.unwrap_or_default();
            self.events_seen = n;
        }
        self.sync_anims(cx);
    }

    /// SPRITE1-U: the tokens catch up with the battle (`Tokens::catch_up`; the skip animations setting applies).
    fn sync_anims(&mut self, cx: &Cx) {
        let instant = cx.settings.v["skip_anim"].as_bool().unwrap_or(false);
        if let Some(s) = self.host.session.as_ref() {
            let tells = cx.settings.v["show_tells"].as_bool().unwrap_or(true);
            self.tokens.catch_up(s, self.anim_ms as u64, instant, cx.tuning.value.walk_ms, tells);
        }
    }

    /// Starts the arena's encounter through the command host (a world one as "<world>:<id>"), so the session replays.
    fn start_battle(&mut self) {
        let encounter = match &self.arena_world {
            Some(world) => format!("{world}:{}", self.encounter),
            None => self.encounter.clone(),
        };
        let mut req = json!({"cmd": "new", "encounter": encounter});
        if let Some(s) = self.seed {
            req["seed"] = json!(s);
        }
        let reply = self.host.handle(&req);
        if reply["ok"] != json!(true) {
            self.message = reply["error"].as_str().unwrap_or("could not start").to_string();
            return;
        }
        self.new_session();
        self.after_start(reply["intro"].as_str().unwrap_or(""));
    }

    fn after_start(&mut self, intro: &str) {
        self.tokens = Tokens::default();
        self.last_hp = self.session().w.units.iter().map(|u| u.hp).collect();
        self.log_seen = self.session().resolver.log.len();
        self.selected = self.session().w.allies.first().copied();
        self.popup = None;
        self.playing = false;
        self.auto = false;
        self.pending_next_round = false;
        self.floats.clear();
        self.log_scroll = 0;
        let (objective, n) = {
            let s = self.session();
            (emberwake_battle::encounter::Objectives::text_of(&s.def.raw, |r| s.units_of_ref(r).first().map(|&c| s.w.units[c].name.clone()).unwrap_or_else(|| r.to_string())), s.events.len())
        };
        self.events_seen = n;
        self.message = if intro.is_empty() { format!("{objective} Plan your round, then confirm.") } else { format!("{intro} {objective}") };
    }

    fn planning(&self) -> bool {
        let s = self.session();
        s.phase == Phase::Planning && !s.plan_confirmed
    }

    /// BattleBridge.outcome: the session's verdict, else won / lost / fled by who stands; "" while it can go on.
    fn outcome(&self) -> String {
        let s = self.session();
        if !s.outcome.is_empty() {
            return s.outcome.clone();
        }
        let standing = s.w.allies.iter().any(|a| s.w.units[*a].is_active());
        let alive = s.w.allies.iter().any(|a| s.w.units[*a].is_alive());
        if s.combat_over {
            return if standing { "won" } else if alive { "fled" } else { "lost" }.into();
        }
        if s.phase == Phase::Replay && !s.can_advance_round() {
            return "fled".into();
        }
        String::new()
    }

    /// World battle over: HP written back (won / fled), the runner settles the outcome, the results popup opens.
    fn settle(&mut self, cx: &mut Cx) {
        let outcome = self.outcome();
        let Mode::World { fight, runner, members } = &self.mode else { return };
        if outcome.is_empty() {
            return;
        }
        let d = cx.data.clone();
        // REWARD1: how the battle left each enemy, for the rewards per defeated enemy
        let tally = emberwake_world::battle::Tally::from_session(self.session());
        let units: Vec<(i64, i64, bool, Vec<String>)> = self.session().w.allies.iter().map(|a| {
            let u = &self.session().w.units[*a];
            (u.hp as i64, u.max_hp as i64, u.is_alive(), u.inventory.pouch.iter().map(|p| p.id.clone()).collect())
        }).collect();
        let gs = cx.gs_mut();
        // ITEMUSE1 rule 7 (GameEncounter.write_back_pouches): the heroes' unused pouch items go back to their saved
        // pouches on every outcome, then the ids that found no battle slot; guests have no member.
        for ((id, unloaded), (.., left)) in members.iter().zip(&units) {
            if let Some(m) = gs.party.member_mut(id) {
                m.pouch = left.iter().chain(unloaded).cloned().collect();
            }
        }
        if outcome == "won" || outcome == "fled" {
            for ((id, _), (hp, max, alive, _)) in members.iter().zip(units) {
                if let Some(m) = gs.party.member_mut(id) {
                    let hp = progression::hp_after(if alive { hp.min(max) } else { 0 }, max, &outcome);
                    m.hp_lost = (max - hp).max(0);
                }
            }
        }
        let res = match runner.as_str() {
            "travel" => travel::after_fight(&d, gs, fight, &outcome, Some(&tally)),
            "dialogue" => dialogue::after_fight(&d, gs, fight, &outcome, Some(&tally)),
            _ => explore::after_fight(&d, gs, fight, &outcome, Some(&tally)),
        };
        cx.set_busy(false);
        if !json::b(&res, "ok", false) {
            show_errors(cx, json::sget(&res, "errors"));
        }
        crate::region::log_events(cx, json::aget(&res, "events"));
        cx.autosave();
        let fight = fight.clone();
        cx.nav.push(Nav::Pop);
        cx.nav.push(Nav::Push(Box::new(crate::hub::Results::new(&res, &outcome, &fight))));
    }

    // ---- Actions (all mutations are command-API requests) -----------------------------------------------------

    /// A player's action (a click or a key): the playback controls click (SoundEvents ui_click), then `run`.
    fn act(&mut self, cx: &mut Cx, a: Action) {
        if matches!(a, Action::Confirm | Action::Step | Action::Autoplay | Action::NextRound) {
            cx.audio.play(Bus::Ui, "sfx/ui_click.wav");
        }
        self.run(cx, a);
    }

    /// An action, from the player or chained by another action or autoplay.
    fn run(&mut self, cx: &mut Cx, a: Action) {
        match a {
            Action::Select(u) => {
                self.selected = Some(u);
                self.popup = None;
            }
            Action::UnitClick(u) => self.unit_click(u),
            Action::TileClick(c) => self.tile_click(c),
            Action::Choose(i) => {
                let Some(p) = self.popup.take() else { return };
                let Some(row) = p.rows.get(i).filter(|r| r.valid).cloned() else { return };
                match row.act {
                    RowAct::Step(step) => {
                        if let Some(h) = self.selected {
                            let r = self.send(cx, json!({"cmd": "add_step", "unit": h, "step": step}));
                            if r["ok"] == json!(true) {
                                self.message = format!("{}: {}", self.session().w.units[h].name, row.label);
                            }
                        }
                    }
                    RowAct::PartyTactic(t) => {
                        self.party_tactic = t;
                        self.send(cx, json!({"cmd": "preset", "preset": TACTICS[t]}));
                        self.message = format!("Party drafted: {}", TACTIC_LABELS[t]);
                    }
                    RowAct::HeroTactic(t) => {
                        self.hero_tactic = t;
                        if let Some(h) = self.selected {
                            self.send(cx, json!({"cmd": "preset", "unit": h, "preset": TACTICS[t]}));
                            self.message = format!("{} drafted: {}", self.session().w.units[h].name, TACTIC_LABELS[t]);
                        }
                    }
                }
            }
            Action::ClosePopup => self.popup = None,
            Action::Confirm => {
                let r = self.send(cx, json!({"cmd": "confirm"}));
                if r["ok"] == json!(true) {
                    self.playing = true;
                    self.next_at = Instant::now();
                    self.message = "Resolving the round...".into();
                }
            }
            Action::Clear => {
                if let Some(h) = self.selected {
                    self.send(cx, json!({"cmd": "clear", "unit": h}));
                }
            }
            Action::Undo => {
                if let Some(h) = self.selected {
                    self.send(cx, json!({"cmd": "undo", "unit": h}));
                }
            }
            Action::Autoplay => {
                self.auto = !self.auto;
                if self.auto && self.planning() {
                    self.send(cx, json!({"cmd": "preset", "preset": TACTICS[self.party_tactic]}));
                    self.run(cx, Action::Confirm);
                }
            }
            Action::PartyMenu => self.tactic_menu(true),
            Action::HeroMenu => self.tactic_menu(false),
            Action::NextRound => {
                let r = self.send(cx, json!({"cmd": "advance"}));
                if r["ok"] == json!(true) {
                    self.message = format!("Round {}: plan your party.", self.session().current_round);
                    if self.auto {
                        self.send(cx, json!({"cmd": "preset", "preset": TACTICS[self.party_tactic]}));
                        self.run(cx, Action::Confirm);
                    }
                }
            }
            Action::Step => {
                if self.session().phase == Phase::Replay {
                    self.run(cx, Action::NextRound);
                } else {
                    self.send(cx, json!({"cmd": "step"}));
                }
            }
            Action::Restart => {
                if !self.world_mode() {
                    self.save_session(cx);
                    self.start_battle();
                }
            }
            Action::ToggleLog => {
                self.show_log = !self.show_log;
                self.log_scroll = 0;
            }
            Action::Continue => self.settle(cx),
            Action::Leave => cx.nav.push(Nav::Pop),
        }
    }

    /// Dropdown of the six tactics under the Party / Hero picker.
    fn tactic_menu(&mut self, party: bool) {
        let current = if party { self.party_tactic } else { self.hero_tactic };
        let rows = (0..TACTICS.len())
            .map(|i| Row {
                label: TACTIC_LABELS[i].to_string(),
                detail: if i == current { "current".into() } else { String::new() },
                valid: true,
                act: if party { RowAct::PartyTactic(i) } else { RowAct::HeroTactic(i) },
                tip: vec![(TACTIC_LABELS[i].to_string(), GOLD), (TACTIC_HELP[i].to_string(), TEXT)],
            })
            .collect();
        let anchor = self.w.iter().find(|w| w.id == if party { "strip.party" } else { "strip.hero" }).map(|w| w.rect).unwrap_or_default();
        self.popup = Some(Popup { title: if party { "Draft the whole party".into() } else { "Draft the selected hero".into() }, x: anchor.x, y: anchor.y + anchor.h + 1, w: 120, rows });
    }

    fn unit_click(&mut self, u: usize) {
        let s = self.session();
        let is_ally = s.w.allies.contains(&u);
        let names: Vec<String> = s.w.units.iter().map(|x| x.name.clone()).collect();
        if !self.planning() || self.selected.is_none() {
            if is_ally {
                self.selected = Some(u);
            }
            return;
        }
        let h = self.selected.unwrap();
        if is_ally && u != h {
            let rows = self.options_for(h, "ally", Some(u), None);
            if rows.iter().any(|r| r.valid) {
                self.open_popup(format!("{} -> {}", names[h], names[u]), rows);
            } else {
                self.selected = Some(u);
            }
            return;
        }
        let (title, rows) = if u == h { (format!("{}: own actions", names[h]), self.options_for(h, "self", None, None)) } else { (format!("Attack {}", names[u]), self.options_for(h, "foe", Some(u), None)) };
        self.open_popup(title, rows);
    }

    fn tile_click(&mut self, c: V2) {
        if !self.planning() {
            return;
        }
        let Some(h) = self.selected else { return };
        let rows = self.options_for(h, "tile", None, Some(c));
        self.open_popup(format!("Tile {}, {}", c.x, c.y), rows);
    }

    fn open_popup(&mut self, title: String, rows: Vec<Row>) {
        if rows.is_empty() {
            self.message = "Nothing to do there.".into();
            return;
        }
        let (mx, my) = self.mouse;
        let w = 170;
        let h = 15 + rows.len() as i32 * 11;
        self.popup = Some(Popup { title, x: mx.clamp(2, W - w - 2), y: my.clamp(2, H - h - 2), w, rows });
    }

    /// Candidate steps for hero `h` (`what`: foe / ally / self / tile), each probed through the API.
    fn options_for(&mut self, h: usize, what: &str, target: Option<usize>, cell: Option<V2>) -> Vec<Row> {
        let skills = self.host.handle(&json!({"cmd": "skills", "unit": h}))["skills"].as_array().cloned().unwrap_or_default();
        let mut steps: Vec<(String, Value, Tip)> = Vec::new();
        let target_cell = target.map(|t| {
            let p = self.session().w.pos(t);
            json!([p.x, p.y])
        });
        for sk in &skills {
            let id = sk["skill"].as_str().unwrap_or("").to_string();
            let name = sk["name"].as_str().unwrap_or("").to_string();
            let kind = sk["kind"].as_str().unwrap_or("");
            let area = sk["area"].as_str().unwrap_or("NONE") != "NONE";
            let mut tip: Tip = vec![(name.clone(), GOLD)];
            if let Some(d) = sk["description"].as_str().filter(|d| !d.is_empty()) {
                tip.push((d.to_string(), TEXT));
            }
            tip.push((format!("{} AP, {} block{}, reach {}-{}", sk["ap"], sk["blocks"], if sk["blocks"] == json!(1) { "" } else { "s" }, sk["reach"][0], sk["reach"][1]), DIM));
            match what {
                "foe" => match kind {
                    "ATTACK" if area => steps.push((name, json!({"mode": "area", "skill": id, "cell": target_cell}), tip)),
                    "ATTACK" | "DISENGAGE" => steps.push((name, json!({"mode": "engage", "skill": id, "target": target}), tip)),
                    _ => {}
                },
                "ally" if kind == "SUPPORT" => steps.push((name, json!({"mode": "ally", "skill": id, "target": target}), tip)),
                "self" if matches!(kind, "DEFEND" | "FOCUS" | "REST" | "DISENGAGE" | "SUPPORT") => steps.push((name, json!({"mode": "self", "skill": id}), tip)),
                "tile" if kind == "ATTACK" && area => steps.push((name, json!({"mode": "area", "skill": id, "cell": cell.map(|c| json!([c.x, c.y]))}), tip)),
                _ => {}
            }
        }
        // ITEMUSE1 rule 2: "Use <item> (<n> left)" for each pouch item that may be aimed at the clicked unit.
        if let Some(t) = if what == "self" { Some(h) } else { target.filter(|_| what != "tile") } {
            let items = self.host.handle(&json!({"cmd": "items", "unit": h, "target": t}))["items"].as_array().cloned().unwrap_or_default();
            for it in items {
                let name = it["name"].as_str().unwrap_or("").to_string();
                let mut tip: Tip = vec![(name.clone(), GOLD)];
                if let Some(d) = it["description"].as_str().filter(|d| !d.is_empty()) {
                    tip.push((d.to_string(), TEXT));
                }
                let reach = match it["reach"].as_i64().unwrap_or(0) {
                    0 => "self only".to_string(),
                    r => format!("reach {r}"),
                };
                tip.push((format!("{} AP, {} block{}, {reach}; used when the action resolves", it["ap"], it["blocks"], if it["blocks"] == json!(1) { "" } else { "s" }), DIM));
                steps.push((format!("Use {name} ({} left)", it["left"]), json!({"mode": "item", "item": it["item"], "target": t}), tip));
            }
        }
        if what == "tile" {
            if let Some(c) = cell {
                steps.insert(0, ("Move here".into(), json!({"mode": "goto", "cell": [c.x, c.y]}), vec![("Move here".into(), GOLD), ("1 AP per tile; moving must come first in a round.".into(), TEXT)]));
            }
        }
        let mut rows = Vec::new();
        for (label, step, mut tip) in steps {
            let p = self.host.handle(&json!({"cmd": "probe", "unit": h, "step": step}));
            let valid = p["valid"] == json!(true);
            let detail = if valid {
                let es = p["entries"].as_array().cloned().unwrap_or_default();
                let dmg: i64 = es.iter().map(|e| e["est_damage"].as_i64().unwrap_or(0)).sum();
                let heal: i64 = es.iter().map(|e| e["est_heal"].as_i64().unwrap_or(0)).sum();
                let first = es.first().map(|e| e["start"].as_i64().unwrap_or(0)).unwrap_or(0);
                let last = es.last().map(|e| e["end"].as_i64().unwrap_or(0)).unwrap_or(0);
                tip.push((format!("Blocks {first}-{last}, AP left after: {}", p["ap_left"]), ALLY));
                let mut d = format!("{} AP", p["ap"].as_i64().unwrap_or(0));
                if dmg > 0 {
                    d.push_str(&format!("  ~{dmg} dmg"));
                }
                if heal > 0 {
                    d.push_str(&format!("  +{heal} HP"));
                }
                d
            } else {
                let r = p["reason"].as_str().unwrap_or("").to_string();
                tip.push((r.clone(), FOE));
                r
            };
            rows.push(Row { label, detail, valid, act: RowAct::Step(step), tip });
        }
        rows.sort_by_key(|r| !r.valid);
        rows
    }
}

impl Screen<GameCx> for BattleScreen {
    fn id(&self) -> &'static str {
        if self.world_mode() {
            "battle"
        } else {
            "test_arena"
        }
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        self.mouse = mouse;
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
        // SPRITE1-U: the token animations run on their own clock at the animation speed setting (FxSettings 0.5-2x).
        let speed = cx.settings.v["anim_speed"].as_f64().unwrap_or(1.0).clamp(0.5, 2.0);
        self.anim_ms += now.duration_since(self.anim_at).as_secs_f64() * 1000.0 * speed;
        self.anim_at = now;
        // FACE1: the Show enemy tells setting changed (I or the settings): the planning facing follows it.
        if self.tokens.tells != cx.settings.v["show_tells"].as_bool().unwrap_or(true) {
            self.sync_anims(cx);
        }
        if now.duration_since(self.tuning_checked) > Duration::from_millis(500) {
            self.tuning_checked = now;
            if cx.tuning.reload_if_changed() {
                self.message = "Tuning reloaded.".into();
            }
        }
        let t = cx.tuning.value.clone();
        self.floats.retain(|f| now.duration_since(f.born) < Duration::from_millis(t.float_ms));
        self.flashes.retain(|f| now.duration_since(f.1) < Duration::from_millis(t.hit_flash_ms));
        // The playtest session is saved after every concluded round (and once more at the end).
        let ended = self.host.session.as_ref().is_some_and(|s| s.combat_over) && !self.end_logged;
        if self.host.checkpoints.len() != self.saved.1 || !self.parked.is_empty() || !self.trail_out.is_empty() || ended {
            self.save_session(cx);
        }
        if self.host.session.is_none() {
            return;
        }
        if self.pending_next_round {
            if now >= self.next_at && self.session().phase == Phase::Replay {
                self.pending_next_round = false;
                self.run(cx, Action::NextRound);
            }
            return;
        }
        if self.playing && now >= self.next_at {
            if self.session().phase == Phase::Replay {
                self.playing = false;
                return;
            }
            self.send(cx, json!({"cmd": "step"}));
            self.next_at = now + Duration::from_millis(t.step_ms);
            if self.session().phase == Phase::Replay {
                self.playing = false;
                let s = self.session();
                let can_advance = s.can_advance_round();
                let message = if s.combat_over {
                    match s.outcome.as_str() {
                        "won" => "Victory!".to_string(),
                        "lost" => "Defeat.".to_string(),
                        o => format!("Battle over: {o}"),
                    }
                } else if !can_advance {
                    "Round limit reached.".into()
                } else {
                    "Round over. Space (or Next) for the next round.".into()
                };
                self.message = message;
                if self.auto && can_advance {
                    self.next_at = now + Duration::from_millis(t.round_pause_ms);
                    self.pending_next_round = true;
                }
            }
        }
    }

    fn key(&mut self, cx: &mut Cx, k: Key) {
        if self.host.session.is_none() {
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
            _ => {}
        }
    }

    fn back(&mut self, cx: &mut Cx) {
        if self.popup.is_some() {
            self.popup = None;
        } else if self.show_log {
            self.show_log = false;
        } else if self.host.session.is_some() && self.planning() {
            self.act(cx, Action::Undo);
        }
    }

    fn escape(&mut self, cx: &mut Cx) {
        if self.popup.is_some() {
            self.popup = None;
        } else if self.show_log {
            self.show_log = false;
        } else if !self.world_mode() {
            cx.nav.push(Nav::Pop);
        }
    }

    fn wheel(&mut self, _cx: &mut Cx, lines: i32) {
        if self.show_log {
            self.log_scroll = (self.log_scroll + lines * 3).max(0);
        }
    }

    fn state(&self, _cx: &Cx) -> Value {
        json!({"message": self.message, "playing": self.playing, "auto": self.auto, "log_open": self.show_log, "world": self.world_mode(),
            "outcome": if self.host.session.is_some() { self.outcome() } else { String::new() },
            "popup": self.popup.as_ref().map(|p| json!({"title": p.title, "rows": p.rows.iter().map(|r| json!({"label": r.label, "detail": r.detail, "valid": r.valid})).collect::<Vec<_>>()}))})
    }

    fn command(&mut self, cx: &Cx, body: &Value) -> Option<Value> {
        // An agent-link `new` ends this session: park it for the hub (saved on the next tick), then start a new one.
        let restart = body["cmd"] == json!("new");
        if restart && self.recordable() {
            self.parked.push((self.slot, self.recorded()));
        }
        let reply = self.host.handle(body);
        if restart && reply["ok"] == json!(true) {
            self.new_session();
            self.tokens = Tokens::default();
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

/// Whether any enemy unit has the boss tier (MusicDirector.combat_context). Wave and phase units are built when the
/// battle starts (held in reserve), so they count from the first round like Godot's encounter refs.
fn boss_fight(s: &Session) -> bool {
    (0..s.w.units.len()).any(|c| !s.w.allies.contains(&c) && s.tier_of(c) == "boss")
}

/// The sound a resolver log line plays (scripts/audio/sound_events.gd FOR_PRESET); None: silent. The engine's log
/// has no mitigation tag, so a hit for 0 plays mitigated; a status that is a buff plays nothing.
fn sound_of(line: &str) -> Option<&'static str> {
    let tag = line.split_once(']').map(|(t, _)| t.trim_start_matches('['))?;
    match tag {
        "Attack Miss" | "Area Miss" => Some("miss"),
        "Attack" if line.contains(" CRIT") => Some("crit"),
        "Attack" if line.contains(" for 0 ") => Some("mitigated"),
        "Attack" => Some("hit"),
        "Status" => {
            let id = line.split_once("(inflicts ")?.1.split(')').next()?;
            (!emberwake_rules::status::row_of(id).is_some_and(|r| r.buff)).then_some("debuff")
        }
        "Defend" if !line.contains("guard was broken") => Some("defend"),
        "Focus" => Some("buff"),
        "Support" => Some("heal"),
        "Move" | "Move Cut Short" | "Withdrawal" | "Dash" | "Chase" | "Disengage" => Some("move_step"),
        _ => None,
    }
}

// ---- Drawing ------------------------------------------------------------------------------------------------------

pub(crate) fn kind_color(kind: &str) -> Rgba {
    match kind {
        "ATTACK" => rgb(160, 56, 56),
        "MOVE" => rgb(52, 92, 160),
        "DEFEND" => rgb(40, 122, 122),
        "SUPPORT" => rgb(52, 128, 68),
        "FOCUS" => rgb(128, 98, 34),
        "REST" => rgb(74, 80, 96),
        "STATUS" => rgb(104, 56, 128),
        "TARGET" => rgb(92, 76, 120),
        "DISENGAGE" => rgb(56, 116, 160),
        _ => rgb(60, 66, 84),
    }
}

/// The frame a battle token shows: the clip `player` plays at `now`, fitted to a `size` square; mirrored when the unit
/// faces left (FACE1, `UnitAnim::faces_left`; the rig's sheets face right).
fn token_frame<'a>(art: &'a mut Art, body: &str, palette: &str, player: &ClipPlayer, now: u64, size: i32, left: bool) -> Option<(ClipFrame, &'a Image)> {
    art.played(body, palette, player, now, size, left)
}

/// The highlight outline of the token frame `f` (`token_frame`), mirrored with it when the unit faces left.
fn token_outline<'a>(art: &'a mut Art, body: &str, palette: &str, f: &ClipFrame, size: i32, col: Rgba, left: bool) -> Option<&'a Image> {
    art.body_outline(body, palette, &f.clip, f.frame, size, col, left)
}

fn hp_bar(p: &mut Painter, r: Rect, hp: i32, max: i32, color: Rgba) {
    p.fill(r, rgb(46, 24, 30));
    let w = if max > 0 { (r.w * hp.max(0) + max - 1) / max } else { 0 };
    p.fill(Rect::new(r.x, r.y, w.min(r.w), r.h), color);
}

fn cell_rect(c: V2) -> Rect {
    Rect::new(GRID_X + c.x * TILE, GRID_Y + c.y * TILE, TILE, TILE)
}

fn short_name(name: &str) -> String {
    name.strip_prefix("Goblin ").unwrap_or(name).to_string()
}

fn stance_name(u: &emberwake_rules::Unit) -> &'static str {
    ["Neutral", "Offensive", "Defensive", "Tactical"][u.stance as usize]
}

/// The actions the resolver holds for `u` this round (after confirm): (start, end, name, kind, target name).
pub(crate) fn committed(s: &Session, u: usize) -> Vec<(i32, i32, String, String, Option<String>)> {
    action_rows(&s.w, &s.resolver.scheduled, u)
}

/// Unit `u`'s actions among `actions` (this round's, or a past round's from the archive): (start, end, name, kind,
/// target name).
pub(crate) fn action_rows(w: &World, actions: &[emberwake_battle::resolver::ScheduledAction], u: usize) -> Vec<(i32, i32, String, String, Option<String>)> {
    actions
        .iter()
        .filter(|a| a.unit == u)
        .map(|a| {
            let (kind, target) = match &a.command.kind {
                CmdKind::Attack { target, .. } => ("ATTACK", *target),
                CmdKind::Area { .. } => ("ATTACK", None),
                CmdKind::Defend { .. } => ("DEFEND", None),
                CmdKind::Focus { .. } => ("FOCUS", None),
                CmdKind::Rest { .. } => ("REST", None),
                CmdKind::StatusBlock { .. } => ("STATUS", None),
                CmdKind::Support { target, .. } => ("SUPPORT", Some(*target)),
                CmdKind::Item { target, .. } => ("ITEM", Some(*target)),
                CmdKind::Target { target } => ("TARGET", *target),
                CmdKind::Walk { .. } => ("MOVE", None),
                CmdKind::Disengage { .. } => ("DISENGAGE", None),
            };
            let name = if a.cancelled { format!("{} (cancelled)", a.command.name) } else { a.command.name.clone() };
            (a.start, a.end, name, kind.to_string(), target.filter(|t| *t != u).map(|t| w.units[t].name.clone()))
        })
        .collect()
}

/// A unit's next (or current) action this round, for cards and hover cards.
fn next_action(s: &Session, e: usize) -> String {
    if s.phase == Phase::Replay {
        return "-".into();
    }
    let planning = s.phase == Phase::Planning && !s.plan_confirmed;
    match s.resolver.scheduled.iter().find(|a| a.unit == e && !a.cancelled && (planning || a.end >= s.current_block)) {
        Some(a) => match a.command.kind {
            CmdKind::Attack { target: Some(t), .. } => format!("{} on {} (B{}-{})", a.command.name, short_name(&s.w.units[t].name), a.start, a.end),
            _ => format!("{} (B{}-{})", a.command.name, a.start, a.end),
        },
        None => "-".into(),
    }
}

/// Hover card of unit `i` (`tells`: the enemy intents while planning, INTENT1).
fn unit_tip(s: &Session, i: usize, sel: Option<usize>, tells: &[Intent]) -> Tip {
    let w = &s.w;
    let u = &w.units[i];
    let ally = w.allies.contains(&i);
    let mut t: Tip = vec![(u.name.clone(), if ally { ALLY } else { FOE })];
    t.push((format!("HP {}/{}   AP {}/{}   {}", u.hp, u.max_hp, u.ap, u.max_ap, stance_name(u)), TEXT));
    t.push((format!("Level {} {}   Speed {}   Reach {}", u.stats.level, u.tier, u.speed, u.attack_range), DIM));
    if !u.st.active.is_empty() {
        t.push((format!("Now: {}", u.st.active.iter().map(|x| emberwake_rules::status::display_name(x)).collect::<Vec<_>>().join(", ")), FOE));
    }
    if !u.st.pending.is_empty() {
        t.push((format!("Next round: {}", u.st.pending.iter().map(|x| emberwake_rules::status::display_name(x)).collect::<Vec<_>>().join(", ")), GOLD));
    }
    if !ally && u.is_active() {
        match tells.iter().find(|r| r.unit == i) {
            Some(r) => {
                t.push((format!("Intent: {}", intent_text::first_line(w, &r.line)), GOLD));
                if r.revealed {
                    let head = if r.readers.is_empty() { "Enraged: its whole plan shows".to_string() } else { intent_text::read_by(w, r, |p| s.w.cat.passives.get(p).map(|x| x.name.clone()).unwrap_or_else(|| p.to_string())) };
                    t.push((head, ALLY));
                    for l in &r.plan {
                        t.push((intent_text::plan_line(w, l), TEXT));
                    }
                }
                if let Some(f) = sel.and_then(|h| intent_text::forecast(r, h)) {
                    t.push((f, FOE));
                }
            }
            None => t.push((format!("Next: {}", next_action(s, i)), GOLD)),
        }
        if let Some(h) = sel.filter(|h| w.units[*h].is_active()) {
            let strike = w.book().lowest(Kind::Attack);
            t.push((format!("{} deals about {} with a Strike, takes about {}", w.units[h].name, expected(w, h, i, strike, None, 0.0, 1.0), expected(w, i, h, strike, None, 0.0, 1.0)), ALLY));
        }
    }
    if !u.is_alive() {
        t.push(("Down".into(), DIM));
    } else if u.is_withdrawn {
        t.push(("Left the battle".into(), DIM));
    }
    t
}

fn draw(g: &mut BattleScreen, cx: &mut Cx, gfx: &mut Gfx, mouse: (i32, i32), live: bool) {
    let now = Instant::now();
    let tuning = cx.tuning.value.clone();
    gfx.canvas.clear(BG);
    let mut ui: Ui<Action> = Ui::new(gfx, mouse, live);
    let Some(s) = g.host.session.as_ref() else {
        ui.p.text(8.0, 8.0, &g.message, BODY, FOE);
        g.w = ui.widgets;
        return;
    };
    let w = &s.w;
    let planning = s.phase == Phase::Planning && !s.plan_confirmed;
    let sel = g.selected.filter(|u| w.allies.contains(u));
    // INTENT1: the enemy tells while planning; their chips and arrows on the grid follow the Show enemy tells setting (I).
    let tells: Vec<Intent> = s.enemy_intents();
    let show_tells = cx.settings.v["show_tells"].as_bool().unwrap_or(true);

    // -- Top strip: the selected hero's six blocks and the round controls.
    g.panels = vec![("strip", Rect::new(2, 2, W - 4, 24)), ("grid", Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4)), ("bar", Rect::new(2, 322, W - 4, 36))];
    ui.p.panel(Rect::new(2, 2, W - 4, 24), PANEL, BORDER, RADIUS);
    let hero_name = sel.map(|u| w.units[u].name.clone()).unwrap_or_else(|| "No hero".into());
    ui.p.text_fit(8.0, 9.0, &format!("{hero_name} plan"), TITLE, 80.0, GOLD);
    ui.push("strip.title", "label", format!("{hero_name} plan"), Rect::new(8, 8, 80, ui.p.line_h(TITLE).ceil() as i32), true, false, None, Vec::new());
    let entries: Vec<(i32, i32, String, String, Option<String>, i32, i32)> = match sel {
        Some(u) if s.plan_confirmed => committed(s, u).into_iter().map(|c| (c.0, c.1, c.2, c.3, c.4, 0, 0)).collect(),
        Some(u) if w.units[u].is_active() => s
            .plan
            .compile(w, u)
            .entries
            .iter()
            .map(|e| {
                let sk = &w.cat.skills[e.skill];
                (e.start_block, e.end_block(w), sk.name.clone(), sk.kind.clone(), e.target.filter(|t| *t != u).map(|t| w.units[t].name.clone()), e.total_ap(), e.est_damage)
            })
            .collect(),
        _ => Vec::new(),
    };
    // TEXTFIT1: the controls right of the blocks take their text's width; the blocks give up width first (34 down to
    // 26 each), then the two dropdowns, the wider first (their text cuts).
    let party_label = format!("Party: {}", TACTIC_LABELS[g.party_tactic]);
    let hero_label = format!("Hero: {}", TACTIC_LABELS[g.hero_tactic]);
    let (label, action, enabled) = if planning {
        ("Confirm round", Action::Confirm, true)
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
    let room = W - 6 - bx - 6 - 8 - auto_w - clear_w - confirm_w;
    let bw = ((room - party_w - hero_w) / blocks).clamp(26, 34);
    let mut over = blocks * bw + party_w + hero_w - room;
    while over > 0 && (party_w > 82 || hero_w > 80) {
        let (big, floor) = if party_w - 82 >= hero_w - 80 { (&mut party_w, 82) } else { (&mut hero_w, 80) };
        let cut = over.min(*big - floor);
        *big -= cut;
        over -= cut;
    }
    let live_block = if !planning && s.phase == Phase::Resolving { s.current_block } else { 0 };
    for b in 1..=s.def.total_blocks {
        let r = Rect::new(bx + (b - 1) * bw, 6, bw - 1, 16);
        let entry = entries.iter().find(|e| b >= e.0 && b <= e.1);
        let mut tip: Tip = Vec::new();
        match entry {
            Some(e) => {
                tip.push((e.2.clone(), GOLD));
                tip.push((format!("Blocks {}-{}", e.0, e.1), TEXT));
                if let Some(t) = &e.4 {
                    tip.push((format!("Target: {t}"), FOE));
                }
                if e.5 > 0 {
                    tip.push((format!("{} AP", e.5), ALLY));
                }
                if e.6 > 0 {
                    tip.push((format!("Expected damage about {}", e.6), GOLD));
                }
            }
            None => {
                ui.p.panel(r, rgba(14, 20, 34, 255), if live_block == b { GOLD } else { BORDER }, RADIUS);
                ui.p.text(r.x as f32 + 3.0, r.y as f32 + 4.0, &format!("{b} Idle"), SMALL, DIM);
                tip.push((format!("Block {b}: nothing planned"), DIM));
            }
        }
        ui.push(format!("strip.block{b}"), "block", entry.map(|e| e.2.clone()).unwrap_or_else(|| "empty".into()), r, true, live_block == b, None, tip);
    }
    // Each action is one continuous span over its blocks, coloured by kind (COMBAT-UI3).
    for e in &entries {
        let r = Rect::new(bx + (e.0 - 1) * bw, 6, (e.1 - e.0 + 1) * bw - 1, 16);
        let live = live_block >= e.0 && live_block <= e.1;
        ui.p.panel(r, kind_color(&e.3), if live { GOLD } else { rgba(255, 255, 255, 50) }, RADIUS);
        ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 4.0, &e.2, SMALL, r.w as f32 - 6.0, TEXT);
    }
    let mut x = bx + blocks * bw + 6;
    ui.dropdown("strip.party", Rect::new(x, 6, party_w, 16), &party_label, Action::PartyMenu, planning, vec![("Party tactic".into(), GOLD), ("AI-drafts every hero with a tactic (you can edit the steps after).".into(), TEXT)]);
    x += party_w + 2;
    ui.dropdown("strip.hero", Rect::new(x, 6, hero_w, 16), &hero_label, Action::HeroMenu, planning && sel.is_some(), vec![("Hero tactic".into(), GOLD), ("AI-drafts the selected hero only.".into(), TEXT)]);
    x += hero_w + 2;
    ui.button_full("strip.auto", Rect::new(x, 6, auto_w, 16), "Auto", Action::Autoplay, !s.combat_over, g.auto, BODY, vec![("Autoplay".into(), GOLD), ("Drafts with the party tactic, confirms and plays rounds until the battle ends. Click again to stop.".into(), TEXT)]);
    x += auto_w + 2;
    ui.button_full("strip.clear", Rect::new(x, 6, clear_w, 16), "Clear", Action::Clear, planning && sel.is_some(), false, BODY, vec![("Clears the selected hero's steps.".into(), TEXT)]);
    x += clear_w + 2;
    ui.button_full("strip.confirm", Rect::new(x, 6, W - 6 - x, 16), label, action, enabled, planning, BODY, vec![(label.into(), GOLD)]);

    // -- Left: party cards and the selected hero.
    ui.p.panel(Rect::new(2, 28, 124, 290), PANEL, BORDER, RADIUS);
    let alive = w.allies.iter().filter(|a| w.units[**a].is_active()).count();
    ui.p.text(8.0, 32.0, &format!("Party {alive}/{}", w.allies.len()), TITLE, ALLY);
    let mut y = 44;
    for &a in &w.allies {
        let u = &w.units[a];
        let r = Rect::new(6, y, 116, 30);
        let is_sel = Some(a) == sel;
        let hover = ui.hover(r);
        ui.p.panel(r, if is_sel { PANEL_HI } else if hover { rgba(28, 40, 66, 255) } else { CARD }, if is_sel { GOLD } else { BORDER }, RADIUS);
        ui.p.text_fit(r.x as f32 + 4.0, r.y as f32 + 1.5, &u.name, BODY, 108.0, if u.is_active() { ALLY } else { DIM });
        hp_bar(&mut ui.p, Rect::new(r.x + 4, r.y + 12, 108, 4), u.hp, u.max_hp, GOOD);
        ui.p.text(r.x as f32 + 4.0, r.y as f32 + 18.0, &format!("HP {}/{}", u.hp, u.max_hp), SMALL, TEXT);
        ui.p.text_right(r.x as f32 + r.w as f32 - 4.0, r.y as f32 + 18.0, &format!("AP {}/{}", u.ap, u.max_ap), SMALL, ALLY);
        let tip = unit_tip(s, a, sel, &tells);
        ui.push(format!("party.{a}"), "card", format!("{} HP {}/{} AP {}/{}", u.name, u.hp, u.max_hp, u.ap, u.max_ap), r, true, is_sel, Some(Action::Select(a)), tip);
        y += 33;
    }
    g.panels.push(("party", Rect::new(2, 28, 124, y - 26)));
    if let Some(h) = sel {
        let u = &w.units[h];
        let r = Rect::new(6, y + 2, 116, 318 - y - 6);
        g.panels.push(("hero", r));
        ui.p.panel(r, CARD, BORDER, RADIUS);
        ui.p.text_fit(r.x as f32 + 4.0, r.y as f32 + 2.0, &u.name, BODY, 64.0, GOLD);
        ui.p.text_right(r.x as f32 + r.w as f32 - 4.0, r.y as f32 + 2.0, stance_name(u), SMALL, DIM);
        let st = s.plan.compile(w, h);
        let weapon = u.inventory.main_weapon().map(|e| e.item.name.clone()).unwrap_or_else(|| "Bare hands".into());
        let mut lines: Vec<(String, Rgba)> = vec![
            (format!("Speed {}  Reach {}", u.speed, u.attack_range), TEXT),
            (format!("AP left {}/{}", if s.plan_confirmed { u.ap } else { st.ap_left }, u.max_ap), ALLY),
            (format!("Gear: {weapon}"), GOLD),
        ];
        if !u.st.active.is_empty() {
            lines.push((format!("Now: {}", u.st.active.iter().map(|x| emberwake_rules::status::display_name(x)).collect::<Vec<_>>().join(", ")), FOE));
        }
        if !u.st.pending.is_empty() {
            lines.push((format!("Next: {}", u.st.pending.iter().map(|x| emberwake_rules::status::display_name(x)).collect::<Vec<_>>().join(", ")), DIM));
        }
        if s.plan_confirmed {
            lines.push(("This round".into(), TEXT));
            for (start, _, name, _, tgt) in committed(s, h) {
                let tgt = tgt.map(|t| format!(" > {}", short_name(&t))).unwrap_or_default();
                lines.push((format!("B{start} {name}{tgt}"), TEXT));
            }
        } else {
            lines.push(("Plan".into(), TEXT));
            if st.entries.iter().all(|e| e.step_index < 0) {
                lines.push(("No plan yet".into(), GOLD));
            }
            for e in st.entries.iter().filter(|e| e.step_index >= 0) {
                let name = &w.cat.skills[e.skill].name;
                let tgt = e.target.filter(|t| *t != h).map(|t| format!(" > {}", short_name(&w.units[t].name))).unwrap_or_default();
                lines.push((format!("B{} {name}{tgt}", e.start_block), TEXT));
            }
            let empty: Vec<String> = (1..=s.def.total_blocks).filter(|b| !st.entries.iter().any(|e| *b >= e.start_block && *b <= e.end_block(w))).map(|b| b.to_string()).collect();
            if !empty.is_empty() && u.is_active() {
                lines.push((format!("Empty: B{}", empty.join(", B")), GOLD));
            }
            if st.error_index >= 0 {
                lines.push((st.error_text.clone(), FOE));
            }
        }
        let mut ly = r.y as f32 + 12.0;
        for (text, col) in lines {
            for part in ui.p.wrap(&text, SMALL, 108.0) {
                if ly > (r.y + r.h) as f32 - 9.0 {
                    break;
                }
                ui.p.text(r.x as f32 + 4.0, ly, &part, SMALL, col);
                ly += 8.0;
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
    let board = Board { s, tokens: &mut g.tokens, floats: &g.floats, flashes: &g.flashes, popup: g.popup.is_some(), anim_ms: g.anim_ms, sel, planning, tells: &tells, show_tells, card_hover };
    let board_hover = match view {
        View::Stage => draw_stage(board, &mut ui, cx, &tuning, now),
        View::Flat => {
            draw_flat(board, &mut ui, cx, &tuning, now);
            None
        }
    };

    // -- Right: hostile cards and the hovered (or first) foe's detail card.
    let right = Rect::new(W - 126, 28, 124, 290);
    ui.p.panel(right, PANEL, BORDER, RADIUS);
    let foes_active = w.enemies.iter().filter(|e| w.units[**e].is_active()).count();
    ui.p.text(right.x as f32 + 6.0, 32.0, &format!("Hostiles {foes_active}"), TITLE, FOE);
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
        let next = tell.map(|t| intent_text::card_line(w, &t.line)).unwrap_or_else(|| format!("Next: {}", next_action(s, e)));
        let next_w = if tell.is_some_and(|t| t.revealed) { 98.0 } else { 110.0 };
        let next_lines = if u.is_active() { ui.p.wrap(&next, SMALL, next_w) } else { Vec::new() };
        let two = next_lines.len() > 1;
        let r = Rect::new(right.x + 4, y, 116, if two { 22 + ui.p.line_h(SMALL).ceil() as i32 } else { 22 });
        let on_board = match view {
            View::Stage => board_hover == Some(e),
            View::Flat => w.grid.pos(e).is_some_and(|p| ui.hover(cell_rect(p))),
        };
        let hover = ui.hover(r) || on_board;
        if hover {
            hovered_foe = Some(e);
        }
        ui.p.panel(r, if hover { rgba(48, 28, 38, 255) } else { CARD }, if hover { FOE } else { BORDER }, RADIUS);
        let col = if u.is_active() { FOE } else { DIM };
        ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 1.5, &short_name(&u.name), BODY, 70.0, col);
        ui.p.text_right(r.x as f32 + r.w as f32 - 3.0, r.y as f32 + 1.5, &format!("{}/{}", u.hp, u.max_hp), BODY, if u.is_active() { GOOD } else { DIM });
        if u.is_active() {
            if two {
                ui.p.text(r.x as f32 + 3.0, r.y as f32 + 11.5, &next_lines[0], SMALL, GOLD);
                // the rest of the line as written (a word `wrap` broke mid-word stays whole)
                let rest = next.get(next_lines[0].len()..).unwrap_or("").trim_start();
                ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 11.5 + ui.p.line_h(SMALL), rest, SMALL, next_w, GOLD);
            } else {
                ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 11.5, &next, SMALL, next_w, GOLD);
            }
            if tell.is_some_and(|t| t.revealed) {
                if let Some(img) = cx.art.image(&emberwake_content::art::badge("foresight"), 9, 9) {
                    ui.p.image(img, Rect::new(r.x + r.w - 11, r.y + 11, 9, 9));
                }
            }
        } else {
            ui.p.text(r.x as f32 + 3.0, r.y as f32 + 11.5, if u.is_withdrawn { "Fled" } else { "Down" }, SMALL, DIM);
        }
        let tip = unit_tip(s, e, sel, &tells);
        ui.push(format!("foe.{e}"), "card", format!("{} {}/{} {}", u.name, u.hp, u.max_hp, next), r, true, hover, Some(Action::UnitClick(e)), tip);
        y += r.h + 2;
    }
    g.panels.push(("hostiles", Rect::new(right.x, 28, right.w, y - 26)));
    debug_assert!(g.panels.iter().all(|p| PANELS.contains(&p.0)), "a panel recorded under a name PANELS lacks");
    let detail_foe = hovered_foe.or_else(|| w.enemies.iter().copied().find(|e| w.units[*e].is_active() && w.grid.pos(*e).is_some()));
    if let Some(e) = detail_foe {
        let r = Rect::new(right.x + 4, y + 2, 116, 318 - y - 6);
        g.panels.push(("intel", r));
        ui.p.panel(r, CARD, FOE, RADIUS);
        let mut ly = r.y as f32 + 3.0;
        for (i, (t, col)) in unit_tip(s, e, sel, &tells).into_iter().enumerate() {
            for part in ui.p.wrap(&t, if i == 0 { BODY } else { SMALL }, 108.0) {
                if ly > (r.y + r.h) as f32 - 9.0 {
                    break;
                }
                ui.p.text(r.x as f32 + 4.0, ly, &part, if i == 0 { BODY } else { SMALL }, col);
                ly += if i == 0 { 10.0 } else { 8.0 };
            }
        }
    }

    // -- Bottom bar: playback, round and status, the last log line.
    ui.p.panel(Rect::new(2, 322, W - 4, 36), PANEL, BORDER, RADIUS);
    ui.button_full("bar.restart", Rect::new(6, 325, 40, 14), "Restart", Action::Restart, !g.world_mode(), false, BODY, vec![("Restart the battle (R)".into(), TEXT)]);
    ui.button_full("bar.step", Rect::new(48, 325, 34, 14), "Step", Action::Step, !planning || s.plan_confirmed, false, BODY, vec![("Resolve one action (Space)".into(), TEXT)]);
    // Test Arena only: leave at any time (Esc does the same); a world battle is left only through its end.
    let ox = if g.world_mode() {
        0
    } else {
        ui.button_full("bar.exit", Rect::new(84, 325, 30, 14), "Exit", Action::Leave, true, false, BODY, vec![("Back to the screen you came from (Esc)".into(), TEXT)]);
        32
    };
    let phase = match s.phase {
        Phase::Planning => "Planning",
        Phase::Resolving => "Resolving",
        Phase::Replay => "Round over",
    };
    ui.p.text((88 + ox) as f32, 326.0, &format!("Round {}/{}  {phase}", s.current_round, s.def.max_rounds), BODY, GOLD);
    ui.p.text_fit((200 + ox) as f32, 326.0, &g.message, BODY, (430 - ox) as f32, TEXT);
    ui.push("bar.status", "label", format!("Round {} {phase}: {}", s.current_round, g.message), Rect::new(88 + ox, 325, 540 - ox, 12), true, false, None, Vec::new());
    let last = s.resolver.log.last().cloned().unwrap_or_else(|| "No actions yet".into());
    let log_r = Rect::new(6, 341, W - 64, 14);
    ui.p.panel(log_r, CARD, BORDER, RADIUS);
    ui.p.text_fit(10.0, 342.5, &last, SMALL, (W - 72) as f32, DIM);
    ui.push("bar.last", "label", last.clone(), log_r, true, false, Some(Action::ToggleLog), vec![(last, TEXT), ("Click to open the full log.".into(), DIM)]);
    ui.button_full("bar.log", Rect::new(W - 54, 341, 48, 14), if g.show_log { "Close" } else { "Expand" }, Action::ToggleLog, true, g.show_log, BODY, vec![("Full combat log (L); the wheel scrolls".into(), TEXT)]);

    let outcome = g.outcome();
    if !outcome.is_empty() {
        let r = Rect::new(GRID_X + 92, GRID_Y + 110, 200, 56);
        ui.p.panel(r, rgba(10, 14, 22, 240), GOLD, 4.0);
        let (title, col) = match outcome.as_str() {
            "won" => ("Victory", GOOD),
            "lost" => ("Defeat", FOE),
            "fled" => ("Fled", GOLD),
            _ => ("Battle over", GOLD),
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
        ui.push("log.panel", "panel", format!("{} lines", s.resolver.log.len()), r, true, true, Some(Action::Step), Vec::new());
        ui.p.panel(r, rgba(10, 14, 24, 246), BORDER, 3.0);
        ui.p.text(r.x as f32 + 6.0, r.y as f32 + 3.0, &format!("Combat log ({} entries)", s.resolver.log.len()), TITLE, GOLD);
        let mut wrapped: Vec<(String, Rgba)> = Vec::new();
        for l in &s.resolver.log {
            let col = if l.starts_with("[Attack]") { TEXT } else if l.starts_with("[Support]") { GOOD } else if l.contains("Failed") || l.contains("Out of Range") { FOE } else { DIM };
            for (k, part) in ui.p.wrap(l, SMALL, (r.w - 22) as f32).into_iter().enumerate() {
                wrapped.push((if k == 0 { part } else { format!("   {part}") }, col));
            }
        }
        let visible = ((r.h - 18) as f32 / 8.0) as i32;
        let max_scroll = (wrapped.len() as i32 - visible).max(0);
        g.log_scroll = g.log_scroll.min(max_scroll);
        let start = (wrapped.len() as i32 - visible - g.log_scroll).max(0) as usize;
        for (k, (t, col)) in wrapped.iter().skip(start).take(visible as usize).enumerate() {
            ui.p.text(r.x as f32 + 6.0, r.y as f32 + 15.0 + k as f32 * 8.0, t, SMALL, *col);
        }
        if max_scroll > 0 {
            let track = Rect::new(r.x + r.w - 8, r.y + 15, 4, r.h - 19);
            ui.p.panel(track, rgba(30, 40, 60, 255), rgba(30, 40, 60, 255), 1.5);
            let thumb_h = (track.h * visible / wrapped.len() as i32).max(8);
            let pos = (track.h - thumb_h) * (max_scroll - g.log_scroll) / max_scroll;
            ui.p.panel(Rect::new(track.x, track.y + pos, 4, thumb_h), GOLD, GOLD, 1.5);
        }
    }

    if let Some(pop) = g.popup.clone() {
        let h = 15 + pop.rows.len() as i32 * 11;
        let r = Rect::new(pop.x, pop.y, pop.w, h);
        ui.push("popup.backdrop", "backdrop", "", Rect::new(0, 0, W, H), true, false, Some(Action::ClosePopup), Vec::new());
        ui.p.panel(r, rgba(14, 20, 34, 252), GOLD, 3.0);
        ui.p.text_fit(r.x as f32 + 5.0, r.y as f32 + 2.0, &pop.title, BODY, r.w as f32 - 10.0, GOLD);
        for (k, row) in pop.rows.iter().enumerate() {
            let rr = Rect::new(r.x + 2, r.y + 13 + k as i32 * 11, r.w - 4, 11);
            let hover = ui.hover(rr);
            if hover && row.valid {
                ui.p.panel(rr, PANEL_HI, rgb(110, 140, 196), 1.5);
            }
            let col = if row.valid { TEXT } else { DIM };
            ui.p.text_fit(rr.x as f32 + 4.0, rr.y as f32 + 1.5, &row.label, BODY, 78.0, col);
            ui.p.text_fit(rr.x as f32 + 84.0, rr.y as f32 + 2.0, &row.detail, SMALL, rr.w as f32 - 86.0, if row.valid { GOLD } else { DIM });
            ui.push(format!("popup.{k}"), "option", format!("{} {}", row.label, row.detail), rr, row.valid, hover, Some(Action::Choose(k)), row.tip.clone());
        }
    }

    g.w = ui.widgets;
}

/// What the board draws from the battle screen in one frame (borrowed apart from the session).
struct Board<'a> {
    s: &'a Session,
    tokens: &'a mut Tokens,
    floats: &'a [Float],
    flashes: &'a [(usize, Instant)],
    popup: bool,
    anim_ms: f64,
    sel: Option<usize>,
    planning: bool,
    /// INTENT1: the enemy tells while planning; drawn while `show_tells` (the Show enemy tells setting, I).
    tells: &'a [Intent],
    show_tells: bool,
    /// STAGE1 3.9: the unit whose party or hostile card the mouse is over.
    card_hover: Option<usize>,
}

/// An action clip a token wants starts with the first candidate its body has (SPRITE1-U).
fn start_wanted(tokens: &mut Tokens, art: &mut Art, i: usize, body: Option<&str>) {
    if let (Some(a), Some(b)) = (tokens.anims.get_mut(i), body) {
        if let Some((cands, t0)) = a.want.take() {
            if let Some(c) = cands.into_iter().find(|c| art.has_clip(b, c)) {
                a.player.play(c, t0);
            }
        }
    }
}

/// A fallen unit still playing its death clip (and holding its last frame for `death_hold_ms`); after it (or with no
/// clip) the dead marker shows.
fn still_dying(art: &mut Art, a: &UnitAnim, body: Option<&str>, now: u64, instant: bool, hold_ms: u64) -> bool {
    match (body, a.fell) {
        (Some(b), Some(_)) if !instant && art.has_clip(b, "death") => {
            art.clips(b).is_some_and(|clips| a.player.playing(clips, now) == "death" && a.player.held_for(clips, now).is_none_or(|h| h < hold_ms))
        }
        _ => false,
    }
}

/// STAGE1 4.1, the Flat view (the board as it was before STAGE1, until the user's review): 32 x 32 cells at (128, 30),
/// the `common` tiles, tree tiles flat, tokens `flat_token_px` on a filled foot ellipse at `foot_alpha`, arrows centre
/// to centre over the units, the cell widgets.
fn draw_flat(b: Board, ui: &mut Ui<Action>, cx: &mut Cx, tuning: &Tuning, now: Instant) {
    let Board { s, tokens, floats, flashes, popup, anim_ms, sel, planning, tells, show_tells, .. } = b;
    let w = &s.w;
    ui.p.panel(Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4), rgb(8, 10, 16), BORDER, RADIUS);
    for gy in 0..GRID_H {
        for gx in 0..GRID_W {
            let c = V2::new(gx, gy);
            let r = cell_rect(c);
            let tid = w.grid.terrain_id(c);
            match cx.art.tile_in(&emberwake_content::art::BATTLE_SETS, tid, gx, gy, TILE) {
                Some(img) => ui.p.image(img, r),
                None => ui.p.fill(r, stage::terrain_color(tid)),
            }
            if w.grid.is_exit(c) {
                ui.p.panel(r.inset(7), rgba(70, 200, 90, 220), rgba(200, 255, 210, 255), 2.0);
                ui.p.text_center(r.x as f32 + 16.0, r.y as f32 + 11.0, "EXIT", SMALL, BG);
            }
            if planning && sel.is_some() && ui.hover(r) && !popup {
                ui.p.panel(r.inset(1), rgba(255, 255, 255, 30), rgba(255, 255, 255, 160), 1.5);
            }
            let tip: Tip = if w.grid.is_exit(c) { vec![("Exit".into(), GOOD), ("A unit that steps here leaves the battle.".into(), TEXT)] } else { Vec::new() };
            ui.push(format!("tile.{gx}.{gy}"), "tile", tid, r, planning, false, if w.occupant(c).is_none() { Some(Action::TileClick(c)) } else { None }, tip);
        }
    }
    let center = |c: V2| ((GRID_X + c.x * TILE + TILE / 2) as f32, (GRID_Y + c.y * TILE + TILE / 2) as f32);
    if let Some(h) = sel {
        if w.units[h].is_active() && planning {
            let st = s.plan.compile(w, h);
            let mut from = w.pos(h);
            for e in &st.entries {
                if e.pos_after != from && e.pos_after.x >= 0 {
                    let (a, b) = (center(from), center(e.pos_after));
                    ui.p.line(a.0, a.1, b.0, b.1, GOLD, 3.0);
                    ui.p.panel(cell_rect(e.pos_after).inset(3), rgba(244, 198, 84, 40), GOLD, 2.0);
                    from = e.pos_after;
                }
            }
        }
    }
    if tuning.intent_arrows && planning {
        let mut seen = Vec::new();
        for a in &s.resolver.scheduled {
            if !w.enemies.contains(&a.unit) || seen.contains(&a.unit) {
                continue;
            }
            if let CmdKind::Attack { target: Some(t), .. } = a.command.kind {
                seen.push(a.unit);
                let (p, q) = (center(w.pos(a.unit)), center(w.pos(t)));
                ui.p.line(p.0, p.1, q.0, q.1, rgba(240, 100, 92, 210), 4.0);
            }
        }
    }
    // SPRITE1-U: each unit stands on a flat ellipse in its side's colour and plays its clips (anims), mirrored while
    // it faces left (FACE1: toward its target or its step, `Tokens::catch_up`; `token_frame`, its outline with it);
    // the selected hero's sprite has a gold outline, a hovered unit's a faint white one. A unit without a body keeps
    // the letter.
    let anim_now = anim_ms as u64;
    let instant = cx.settings.v["skip_anim"].as_bool().unwrap_or(false);
    let still = UnitAnim::default();
    let geo = Geom::new(View::Flat, tuning);
    for i in 0..w.units.len() {
        let Some(pos) = w.grid.pos(i) else { continue };
        let u = &w.units[i];
        let r = cell_rect(pos);
        let ally = w.allies.contains(&i);
        let body = cx.art.first_body(&emberwake_content::art::body_chain(&s.art[i]));
        let palette = s.palettes.get(i).map_or("", |p| p.as_str());
        start_wanted(tokens, &mut cx.art, i, body.as_deref());
        let a = tokens.anims.get(i).unwrap_or(&still);
        let left = if tokens.anims.get(i).is_some() { a.faces_left() } else { !ally };
        if !u.is_alive() && !still_dying(&mut cx.art, a, body.as_deref(), anim_now, instant, tuning.death_hold_ms) {
            ui.p.text_center(r.x as f32 + 16.0, r.y as f32 + 11.0, "x", BODY, DIM);
            continue;
        }
        let flash = flashes.iter().any(|f| f.0 == i);
        let shake = if flash { tuning.hit_shake_px as f32 * if (now.elapsed().as_millis() / 30).is_multiple_of(2) { 1.0 } else { -1.0 } } else { 0.0 };
        let tp = geo.token_px;
        // A moving unit slides from its old tile.
        let (mut dx, mut dy) = (0, 0);
        if let Some((from, t0)) = a.slide {
            let t = anim_now.saturating_sub(t0);
            if t < tuning.walk_ms && !instant {
                (dx, dy) = geo.slide(from, pos, 1.0 - t as f32 / tuning.walk_ms.max(1) as f32);
            }
        }
        let tr = geo.token_rect(pos, shake as i32 + dx, dy);
        let ring = if ally { ALLY } else { FOE };
        if u.is_alive() {
            ui.p.ellipse(tr.x as f32 + tp as f32 / 2.0, (tr.y + tp) as f32 - 2.5, tp as f32 * 0.36, tp as f32 * 0.11, rgba(ring[0], ring[1], ring[2], tuning.foot_alpha));
        }
        let hover = ui.hover(r) && !popup;
        let outline = if Some(i) == sel { Some(GOLD) } else if hover { Some(rgba(255, 255, 255, tuning.hover_outline_alpha)) } else { None };
        let frame = body.as_deref().and_then(|b| token_frame(&mut cx.art, b, palette, &a.player, anim_now, tp, left).map(|(f, img)| {
            ui.p.image(img, tr);
            f
        }));
        match (frame, body.as_deref()) {
            (Some(f), Some(b)) => {
                if let Some(o) = outline.and_then(|col| token_outline(&mut cx.art, b, palette, &f, tp, col, left)) {
                    ui.p.image(o, Rect::new(tr.x - 1, tr.y - 1, tp + 2, tp + 2));
                }
            }
            _ => {
                ui.p.panel(tr.inset(4), ring, outline.unwrap_or(ring), 6.0);
                ui.p.text_center(tr.x as f32 + tp as f32 / 2.0, r.y as f32 + 11.0 + dy as f32, &u.name.chars().next().unwrap_or('?').to_string(), BODY, BG);
            }
        }
        if flash {
            ui.p.fill(tr, rgba(255, 255, 255, 110));
        }
        if !u.is_alive() {
            continue;
        }
        hp_bar(&mut ui.p, Rect::new(r.x + 3 + dx, r.y + TILE - 3 + dy, TILE - 6, 2), u.hp, u.max_hp, if ally { GOOD } else { FOE });
        let tip = unit_tip(s, i, sel, tells);
        ui.push(format!("unit.{i}"), "unit", format!("{} {}/{}", u.name, u.hp, u.max_hp), r, true, Some(i) == sel, Some(Action::UnitClick(i)), tip);
    }
    if show_tells {
        for r in tells {
            let Some(from) = w.grid.pos(r.unit) else { continue };
            let (fx, fy) = center(from);
            let to = r.line.target.and_then(|t| w.grid.pos(t)).or((r.line.cell != emberwake_battle::grid::NO_CELL).then_some(r.line.cell));
            if let Some(to) = to.filter(|c| *c != from) {
                let (tx, ty) = center(to);
                let (dx, dy) = (tx - fx, ty - fy);
                let len = (dx * dx + dy * dy).sqrt().max(1.0);
                // From the token's edge to just short of the target's.
                let (sx, sy, ex, ey) = (fx + dx / len * 10.0, fy + dy / len * 10.0, tx - dx / len * 9.0, ty - dy / len * 9.0);
                ui.p.arrow(sx, sy, ex, ey, rgba(240, 100, 92, 200), stage::arrow_dash(r.line.kind), 4.0);
            }
            let chip = Rect::new(GRID_X + from.x * TILE + 1, GRID_Y + from.y * TILE + 1, 10, 10);
            intent_chip(ui, cx, r.line.kind, chip);
        }
    }
    for f in floats {
        if let Some(pos) = w.grid.pos(f.unit) {
            let t = now.duration_since(f.born).as_millis() as f32 / tuning.float_ms.max(1) as f32;
            let (cx, cy) = center(pos);
            float_text(ui, &f.text, f.color, cx, cy - 14.0 - tuning.float_rise_px as f32 * t);
        }
    }
}

/// An intent chip (INTENT1): the kind's icon, else a lettered plate.
fn intent_chip(ui: &mut Ui<Action>, cx: &mut Cx, kind: &str, chip: Rect) {
    match cx.art.image(&emberwake_content::art::intent(kind), 10, 10) {
        Some(img) => ui.p.image(img, chip),
        None => {
            ui.p.panel(chip, rgba(48, 28, 38, 230), FOE, 2.0);
            ui.p.text_center(chip.x as f32 + 5.0, chip.y as f32 + 1.0, &kind[..1].to_uppercase(), SMALL, TEXT);
        }
    }
}

/// A floating number with its shadow, centred on x at line top y.
fn float_text(ui: &mut Ui<Action>, text: &str, col: Rgba, x: f32, y: f32) {
    ui.p.text_center(x + 0.6, y + 0.6, text, TITLE, rgba(0, 0, 0, 220));
    ui.p.text_center(x, y, text, TITLE, col);
}

/// One unit on the Stage this frame (STAGE1 3.6, 3.8).
struct StageUnit {
    i: usize,
    pos: V2,
    ally: bool,
    alive: bool,
    /// Its sprite draws (standing, or still playing its death clip); else the dead marker shows.
    standing: bool,
    body: Option<String>,
    palette: String,
    left: bool,
    flash: bool,
    /// The foot point with the slide.
    foot: (i32, i32),
    /// The drawn sprite square (slide and hit shake).
    tr: Rect,
    /// The body's solid rect at its place (slide, no shake): the feet pass's rect and half of its widget rect.
    body_rect: Rect,
}

/// A 1 px edged plate (the Stage's ground markers, 3.7 pass 3).
fn plate(p: &mut Painter, r: Rect, fill: Rgba, edge: Rgba) {
    p.fill(r, fill);
    p.fill(Rect::new(r.x, r.y, r.w, 1), edge);
    p.fill(Rect::new(r.x, r.y + r.h - 1, r.w, 1), edge);
    p.fill(Rect::new(r.x, r.y + 1, 1, r.h - 2), edge);
    p.fill(Rect::new(r.x + r.w - 1, r.y + 1, 1, r.h - 2), edge);
}

/// A cell plate inset `ix` across and `iy` down.
fn inset2(r: Rect, ix: i32, iy: i32) -> Rect {
    Rect::new(r.x + ix, r.y + iy, r.w - 2 * ix, r.h - 2 * iy)
}

fn with_alpha(c: Rgba, a: u8) -> Rgba {
    [c[0], c[1], c[2], a]
}

/// STAGE1 option B, the Stage view: the board tilted to 32 x 24 cells, run as `stage::passes` (backdrop, ground,
/// markers, ground arrows, shadows, front face, the feet-sorted blocks, props and units, the outlines through what
/// covers a unit, the top: HP bars, chips, the exit sign, floats). Cell widgets first, then the units' in feet order
/// (front-most last, so it takes the click). Returns the unit the mouse is over on the board.
fn draw_stage(b: Board, ui: &mut Ui<Action>, cx: &mut Cx, tuning: &Tuning, now: Instant) -> Option<usize> {
    let Board { s, tokens, floats, flashes, popup, anim_ms, sel, planning, tells, show_tells, card_hover } = b;
    let w = &s.w;
    let geo = Geom::new(View::Stage, tuning);
    let tp = geo.token_px;
    let anim_now = anim_ms as u64;
    let instant = cx.settings.v["skip_anim"].as_bool().unwrap_or(false);
    let still = UnitAnim::default();
    ui.p.panel(Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4), rgb(8, 10, 16), BORDER, RADIUS);

    // The units' places this frame.
    let mut units: Vec<StageUnit> = Vec::new();
    for i in 0..w.units.len() {
        let Some(pos) = w.grid.pos(i) else { continue };
        let u = &w.units[i];
        let ally = w.allies.contains(&i);
        let body = cx.art.first_body(&emberwake_content::art::body_chain(&s.art[i]));
        let palette = s.palettes.get(i).cloned().unwrap_or_default();
        start_wanted(tokens, &mut cx.art, i, body.as_deref());
        let a = tokens.anims.get(i).unwrap_or(&still);
        let left = if tokens.anims.get(i).is_some() { a.faces_left() } else { !ally };
        let standing = u.is_alive() || still_dying(&mut cx.art, a, body.as_deref(), anim_now, instant, tuning.death_hold_ms);
        let (mut dx, mut dy) = (0, 0);
        if let Some((from, t0)) = a.slide {
            let t = anim_now.saturating_sub(t0);
            if t < tuning.walk_ms && !instant {
                (dx, dy) = geo.slide(from, pos, 1.0 - t as f32 / tuning.walk_ms.max(1) as f32);
            }
        }
        let flash = flashes.iter().any(|f| f.0 == i);
        let shake = if flash { tuning.hit_shake_px as f32 * if (now.elapsed().as_millis() / 30).is_multiple_of(2) { 1.0 } else { -1.0 } } else { 0.0 };
        let (fx, fy) = geo.foot(pos);
        let rest = geo.token_rect(pos, dx, dy);
        // 3.8: the body's solid bounds of its idle frame 0 (animation frames do not move it); no body: the letter plate.
        let body_rect = match body.as_deref().and_then(|k| cx.art.body_bounds(k, &palette, tp, left)) {
            Some(bb) => Rect::new(rest.x + bb.x, rest.y + bb.y, bb.w, bb.h),
            None => rest.inset(4),
        };
        units.push(StageUnit { i, pos, ally, alive: u.is_alive(), standing, body, palette, left, flash, foot: (fx + dx, fy + dy), tr: geo.token_rect(pos, dx + shake as i32, dy), body_rect });
    }
    // 3.7 pass 7: blocks, standing props and units sorted by their feet.
    let mut items = stage::board_items(&mut cx.art, &w.grid);
    items.extend(units.iter().filter(|t| t.standing).map(|t| stage::unit_item(t.i, t.foot, t.body_rect)));
    let order = stage::feet_order(&items);
    let unit_of = |i: usize| units.iter().find(|t| t.i == i);
    // 3.8: a standing unit's widget rect; the front-most under the mouse is the one a click reaches.
    let widget_rect = |t: &StageUnit| stage::unit_rect(geo.cell_rect(t.pos), t.body_rect);
    let board_hover = if ui.live && !popup {
        order.iter().rev().find_map(|p| match p.item {
            Item::Unit(i) => unit_of(i).filter(|t| t.alive && widget_rect(t).contains(ui.mouse.0, ui.mouse.1)).map(|t| t.i),
            _ => None,
        })
    } else {
        None
    };
    let hovered = board_hover.or(card_hover);
    // The cell a click would reach (the hovered unit's, else the one under the mouse) shows the hover plate.
    let hover_cell = if planning && sel.is_some() && ui.live && !popup {
        board_hover.and_then(|u| w.grid.pos(u)).or_else(|| geo.cell_at(ui.mouse.0, ui.mouse.1))
    } else {
        None
    };
    let foot_f = |c: V2| {
        let (x, y) = geo.foot(c);
        (x as f32, y as f32)
    };
    let mut frames: Vec<(usize, ClipFrame)> = Vec::new();

    ui.p.clip(Some(stage::INTERIOR));
    for pass in stage::passes() {
        match pass {
            Pass::Backdrop => stage::draw_backdrop(&mut ui.p, &mut cx.art, &w.grid, &s.def.zone),
            Pass::Ground => stage::draw_ground(&mut ui.p, &mut cx.art, &w.grid, tuning.grid_alpha),
            Pass::Markers => {
                if let Some(c) = hover_cell {
                    plate(&mut ui.p, geo.cell_rect(c), rgba(255, 255, 255, 30), rgba(255, 255, 255, 160));
                }
                // INTENT1 area tells: the tiles an enemy's area attack will hit.
                if show_tells {
                    for r in tells.iter().filter(|r| r.line.kind == "area") {
                        let cells = s.resolver.scheduled.iter().find_map(|a| match &a.command.kind {
                            CmdKind::Area { cells, aim, .. } if a.unit == r.unit && *aim == r.line.cell => Some(cells.clone()),
                            _ => None,
                        });
                        for c in cells.unwrap_or_default().into_iter().filter(|c| w.grid.is_valid(*c)) {
                            plate(&mut ui.p, geo.cell_rect(c).inset(1), rgba(240, 100, 92, 70), rgba(240, 100, 92, 230));
                        }
                    }
                }
                if let Some(h) = sel.filter(|h| w.units[*h].is_active() && planning) {
                    let st = s.plan.compile(w, h);
                    let mut from = w.pos(h);
                    for e in &st.entries {
                        if e.pos_after != from && e.pos_after.x >= 0 {
                            let (a, b) = (foot_f(from), foot_f(e.pos_after));
                            stage::wide_line(&mut ui.p, a.0, a.1 - 3.0, b.0, b.1 - 3.0, GOLD, 2.0, 0.0);
                            ui.p.panel(inset2(geo.cell_rect(e.pos_after), 3, 2), rgba(244, 198, 84, 40), GOLD, 2.0);
                            from = e.pos_after;
                        }
                    }
                }
                for c in w.grid.exits.iter().filter(|c| w.grid.is_valid(**c)) {
                    plate(&mut ui.p, inset2(geo.cell_rect(*c), 3, 2), rgba(70, 200, 90, 90), rgba(200, 255, 210, 200));
                }
            }
            Pass::Arrows => {
                let col = rgba(240, 100, 92, 210);
                let mut drawn: Vec<(V2, V2)> = Vec::new();
                if show_tells {
                    for r in tells {
                        let Some(from) = w.grid.pos(r.unit) else { continue };
                        let to = r.line.target.and_then(|t| w.grid.pos(t)).or((r.line.cell != emberwake_battle::grid::NO_CELL).then_some(r.line.cell));
                        if let Some(to) = to.filter(|c| *c != from) {
                            if let Some((x0, y0, x1, y1)) = stage::ground_arrow(foot_f(from), foot_f(to)) {
                                stage::wide_arrow(&mut ui.p, x0, y0, x1, y1, col, 1.5, stage::arrow_dash(r.line.kind), 4.0);
                            }
                            drawn.push((from, to));
                        }
                    }
                }
                if tuning.intent_arrows && planning {
                    let mut seen = Vec::new();
                    for a in &s.resolver.scheduled {
                        if !w.enemies.contains(&a.unit) || seen.contains(&a.unit) {
                            continue;
                        }
                        if let CmdKind::Attack { target: Some(t), .. } = a.command.kind {
                            seen.push(a.unit);
                            let (from, to) = (w.pos(a.unit), w.pos(t));
                            if from == to || drawn.contains(&(from, to)) {
                                continue;
                            }
                            if let Some((x0, y0, x1, y1)) = stage::ground_arrow(foot_f(from), foot_f(to)) {
                                stage::wide_arrow(&mut ui.p, x0, y0, x1, y1, col, 1.5, 0.0, 4.0);
                            }
                        }
                    }
                }
            }
            Pass::Shadows => {
                stage::draw_prop_shadows(&mut ui.p, &mut cx.art, &w.grid);
                for t in units.iter().filter(|t| t.alive) {
                    let (fx, fy) = (t.foot.0 as f32, t.foot.1 as f32);
                    ui.p.ellipse(fx + 1.0, fy, 0.2 * tp as f32, 2.4, rgba(8, 10, 6, tuning.shadow_alpha));
                    ui.p.ellipse_ring(fx, fy, 0.25 * tp as f32, 3.2, with_alpha(if t.ally { ALLY } else { FOE }, tuning.foot_ring_alpha));
                }
            }
            Pass::Face => stage::draw_face(&mut ui.p, &mut cx.art, &w.grid),
            Pass::Feet => {
                for p in &order {
                    match p.item {
                        Item::Block(c) => stage::draw_block(&mut ui.p, &mut cx.art, &w.grid, c),
                        Item::Prop(c) => stage::draw_prop(&mut ui.p, &mut cx.art, &w.grid, c),
                        Item::Unit(i) => {
                            let Some(t) = unit_of(i) else { continue };
                            let a = tokens.anims.get(i).unwrap_or(&still);
                            let ring = if t.ally { ALLY } else { FOE };
                            let outline = if Some(i) == sel { Some(GOLD) } else if hovered == Some(i) { Some(rgba(255, 255, 255, tuning.hover_outline_alpha)) } else { None };
                            let frame = t.body.as_deref().and_then(|b| token_frame(&mut cx.art, b, &t.palette, &a.player, anim_now, tp, t.left).map(|(f, img)| {
                                ui.p.image(img, t.tr);
                                f
                            }));
                            match (frame, t.body.as_deref()) {
                                (Some(f), Some(b)) => {
                                    if let Some(o) = outline.and_then(|col| token_outline(&mut cx.art, b, &t.palette, &f, tp, col, t.left)) {
                                        ui.p.image(o, Rect::new(t.tr.x - 1, t.tr.y - 1, tp + 2, tp + 2));
                                    }
                                    frames.push((i, f));
                                }
                                _ => {
                                    ui.p.panel(t.tr.inset(4), ring, outline.unwrap_or(ring), 6.0);
                                    let letter = w.units[i].name.chars().next().unwrap_or('?').to_string();
                                    ui.p.text_center(t.tr.x as f32 + tp as f32 / 2.0, (t.tr.y + tp / 2) as f32 - 5.0, &letter, BODY, BG);
                                }
                            }
                            if t.flash {
                                ui.p.fill(t.tr, rgba(255, 255, 255, 110));
                            }
                        }
                    }
                }
            }
            Pass::Through => {
                for (k, p) in order.iter().enumerate() {
                    let Item::Unit(i) = p.item else { continue };
                    let Some(t) = unit_of(i).filter(|t| t.alive) else { continue };
                    let (Some(b), Some((_, f))) = (t.body.as_deref(), frames.iter().find(|(j, _)| *j == i)) else { continue };
                    let col = match stage::through(&order, k, Some(i) == sel, hovered == Some(i)) {
                        Some(Through::Side) => with_alpha(if t.ally { ALLY } else { FOE }, 230),
                        Some(Through::Selected) => GOLD,
                        Some(Through::Hovered) => rgba(255, 255, 255, tuning.hover_outline_alpha),
                        None => continue,
                    };
                    if let Some(o) = token_outline(&mut cx.art, b, &t.palette, f, tp, col, t.left) {
                        ui.p.image(o, Rect::new(t.tr.x - 1, t.tr.y - 1, tp + 2, tp + 2));
                    }
                }
            }
            Pass::Top => {
                for t in &units {
                    let u = &w.units[t.i];
                    let (fx, fy) = t.foot;
                    if !t.standing {
                        ui.p.text_center(fx as f32, fy as f32 - ui.p.line_h(BODY) / 2.0, "x", BODY, DIM);
                    } else if t.alive {
                        hp_bar(&mut ui.p, Rect::new(fx - 12, fy + 3, 24, 2), u.hp, u.max_hp, if t.ally { GOOD } else { FOE });
                    }
                }
                if show_tells {
                    for r in tells {
                        let Some(t) = units.iter().find(|t| t.i == r.unit) else { continue };
                        let chip = Rect::new(t.foot.0 - 15, (t.foot.1 as f32 - 0.95 * tp as f32 - 2.0).round() as i32, 10, 10);
                        intent_chip(ui, cx, r.line.kind, chip);
                    }
                }
                for c in w.grid.exits.iter().filter(|c| w.grid.is_valid(**c)) {
                    let r = geo.cell_rect(*c);
                    let (cx_, cy_) = (r.x + r.w / 2, r.y + r.h / 2);
                    let sign = Rect::new(cx_ - 11, cy_ - 2 - 5, 22, 9);
                    ui.p.panel(sign, rgba(70, 200, 90, 230), rgba(200, 255, 210, 255), 2.0);
                    ui.p.text_center(cx_ as f32, sign.y as f32 + (sign.h as f32 - ui.p.line_h(SMALL)) / 2.0, "EXIT", SMALL, BG);
                }
                for f in floats {
                    let Some(pos) = w.grid.pos(f.unit) else { continue };
                    let t = now.duration_since(f.born).as_millis() as f32 / tuning.float_ms.max(1) as f32;
                    let (x, y) = geo.chest(pos);
                    float_text(ui, &f.text, f.color, x as f32, y as f32 - 14.0 - tuning.float_rise_px as f32 * t);
                }
            }
        }
    }
    ui.p.clip(None);

    // 3.8: the cell widgets (32 x 24), then the units' in feet order (front-most last).
    for gy in 0..GRID_H {
        for gx in 0..GRID_W {
            let c = V2::new(gx, gy);
            let tip: Tip = if w.grid.is_exit(c) { vec![("Exit".into(), GOOD), ("A unit that steps here leaves the battle.".into(), TEXT)] } else { Vec::new() };
            ui.push(format!("tile.{gx}.{gy}"), "tile", w.grid.terrain_id(c), geo.cell_rect(c), planning, false, if w.occupant(c).is_none() { Some(Action::TileClick(c)) } else { None }, tip);
        }
    }
    for p in &order {
        let Item::Unit(i) = p.item else { continue };
        let Some(t) = unit_of(i).filter(|t| t.alive) else { continue };
        let u = &w.units[i];
        let tip = unit_tip(s, i, sel, tells);
        ui.push(format!("unit.{i}"), "unit", format!("{} {}/{}", u.name, u.hp, u.max_hp), widget_rect(t), true, Some(i) == sel, Some(Action::UnitClick(i)), tip);
    }
    board_hover
}

#[cfg(test)]
mod tests {
    use super::{action_clips, boss_fight, facing_toward, sound_of, token_frame, token_outline, Action, BattleScreen, Tokens};
    use crate::screens::Screen;
    use crate::stage::{Geom, View};
    use emberwake_battle::grid::GRID_H;
    use emberwake_battle::command::CmdKind;
    use emberwake_battle::grid::{V2, NO_CELL};
    use aegis_core::assets::{ClipPlayer, Clips, Layers};
    use aegis_ui::art::Art;
    use emberwake_battle::session::{EncounterDef, Phase, Session};
    use emberwake_host::{Battle, Host};
    use emberwake_rules::Stance;
    use emberwake_world::battle::WorldRoster;
    use emberwake_world::Data;
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn an_action_picks_the_rig_clip_of_its_skill_kind_and_weapon() {
        assert_eq!(action_clips("shield_bash", "attack", "sword"), ["shield_bash", "attack"], "a skill with its own clip first");
        assert_eq!(action_clips("cyclone_kick", "area", ""), ["sweep_kick", "sweep_kick", "kick", "jab", "attack"]);
        assert_eq!(action_clips("strike", "attack", "fist"), ["jab", "attack"], "fists jab");
        assert_eq!(action_clips("strike", "attack", "bow"), ["attack"]);
        assert_eq!(action_clips("volley", "area", "bow"), ["attack"]);
        assert_eq!(action_clips("mend", "support", "mace"), ["cast"]);
        assert!(action_clips("guard", "", "sword").is_empty(), "guards play through the stance, moves through the slide");
    }

    /// The rig's sheets face right: a token facing left (FACE1) shows the sheet frame mirrored, its outline with it; one
    /// facing right the sheet frame as drawn. A scratch body whose frame is lopsided shows the difference.
    #[test]
    fn a_token_facing_left_is_the_sheet_frame_mirrored_and_one_facing_right_is_not() {
        const F: u32 = 8;
        let root = std::env::temp_dir().join(format!("emberwake_foe_facing_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("units/brawler/goblin");
        std::fs::create_dir_all(&dir).unwrap();
        // A bar on the left of the 8x8 frame (x 1..3, the back) and one pixel on the right (x 6, the face).
        let mut px = vec![0u8; (F * F * 4) as usize];
        for x in [1u32, 2, 3, 6] {
            let i = ((4 * F + x) * 4) as usize;
            px[i..i + 4].copy_from_slice(&[90, 140, 60, 255]);
        }
        let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(dir.join("sheet.png")).unwrap()), F, F);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&px).unwrap();
        std::fs::write(dir.join("clips.json"), json!({"frame": [F, F], "clips": [{"id": "idle", "frames": 1, "ms": [100]}]}).to_string()).unwrap();
        let mut art = Art::layered(Layers::of(vec![root.clone()]));
        let (body, p, size) = ("brawler/goblin", ClipPlayer::default(), 3 * F as i32);
        let sheet = art.body_clip(body, "", "idle", 0, size).unwrap().clone();
        let left = token_frame(&mut art, body, "", &p, 0, size, true).unwrap().1.clone();
        let right = token_frame(&mut art, body, "", &p, 0, size, false).unwrap().1.clone();
        assert_eq!(left, sheet.mirrored(), "facing left: the sheet frame mirrored");
        assert_eq!(right, sheet, "facing right: as the sheet faces");
        assert_ne!(left, right);
        let f = token_frame(&mut art, body, "", &p, 0, size, true).unwrap().0;
        let gold = [244, 198, 84, 255];
        let left_ring = token_outline(&mut art, body, "", &f, size, gold, true).unwrap().clone();
        assert_eq!(left_ring, sheet.mirrored().outline(gold), "the selection outline follows the mirrored frame");
        assert_eq!(token_outline(&mut art, body, "", &f, size, gold, false).unwrap().clone(), sheet.outline(gold));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Every rig clip (SPRITE1 section 2), one frame each, looping as the export writes them.
    fn rig_clips() -> Clips {
        let ids = ["idle", "walk", "attack", "jab", "sweep_kick", "kick", "shield_bash", "haymaker", "hurt", "death", "defend", "idle_guarded", "hurt_guarded"];
        let clips: Vec<_> = ids.iter().map(|id| {
            let looped = ["idle", "walk", "idle_guarded"].contains(id);
            json!({"id": id, "frames": 1, "ms": [100], "loop": looped})
        }).collect();
        Clips::parse(&json!({"frame": [48, 48], "clips": clips})).unwrap()
    }

    /// SPRITE1-U: played step by step, every token follows its unit: a move slides and walks, a hit plays hurt (guarded
    /// while it guards), a fall plays death, the resolved attack queues its actor's clip; skip plays nothing.
    #[test]
    fn tokens_follow_the_battle_step_by_step() {
        let dir = Data::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        let mut host = Host::new(Battle::with_data(Arc::new(Data::open(&dir).expect("data").0)));
        assert_eq!(host.handle(&json!({"cmd": "new", "encounter": "goblin_skirmish", "seed": 3}))["ok"], json!(true));
        let clips = rig_clips();
        let (mut t, mut skip) = (Tokens::default(), Tokens::default());
        let mut now = 0u64;
        t.catch_up(host.session.as_ref().unwrap(), now, false, 300, true);
        skip.catch_up(host.session.as_ref().unwrap(), now, true, 300, true);
        assert!(t.anims.iter().all(|a| a.player.playing(&clips, now).is_empty() && a.slide.is_none()), "a new battle plays nothing");
        let (mut moves, mut hits, mut falls, mut attacks) = (0, 0, 0, 0);
        for _ in 0..600 {
            let s = host.session.as_ref().unwrap();
            if s.combat_over {
                break;
            }
            if s.phase == Phase::Replay {
                if !s.can_advance_round() {
                    break;
                }
                host.handle(&json!({"cmd": "advance"}));
            }
            if host.session.as_ref().unwrap().is_planning() {
                host.handle(&json!({"cmd": "preset", "preset": "aggressive"}));
            }
            let before: Vec<_> = t.anims.iter().map(|a| a.seen.clone()).collect();
            let popped = host.session.as_ref().unwrap().last_popped;
            assert_eq!(host.handle(&json!({"cmd": "step"}))["ok"], json!(true));
            now += 1000;
            let s = host.session.as_ref().unwrap();
            t.catch_up(s, now, false, 300, true);
            skip.catch_up(s, now, true, 300, true);
            for (i, u) in s.w.units.iter().enumerate() {
                let (a, was) = (&t.anims[i], &before[i]);
                let guarded = u.guard > 0.0 || u.stance == Stance::Defensive;
                let playing = a.player.playing(&clips, now);
                if was.alive && !u.is_alive() {
                    falls += 1;
                    assert_eq!((a.fell, playing), (Some(now), "death"), "{} fell", u.name);
                } else if u.is_alive() && u.hp < was.hp {
                    hits += 1;
                    assert_eq!(playing, if guarded { "hurt_guarded" } else { "hurt" }, "{} was hit", u.name);
                } else if u.is_alive() && guarded && !was.guarded {
                    assert_eq!(playing, "defend", "{} raised a guard", u.name);
                } else if was.pos.is_some() && s.w.grid.pos(i).is_some() && was.pos != s.w.grid.pos(i) {
                    moves += 1;
                    assert_eq!((a.slide, playing), (Some((was.pos.unwrap(), now)), "walk"), "{} moved", u.name);
                }
                assert_eq!(a.player.base(), if guarded { "idle_guarded" } else { "idle" });
                let k = &skip.anims[i];
                assert!(k.player.playing(&clips, now).is_empty() && k.slide.is_none() && k.want.is_none(), "skip: the end state at once");
                assert_eq!(k.fell.is_some(), !u.is_alive());
            }
            if let Some((_, _, idx)) = s.last_popped.filter(|p| Some(*p) != popped) {
                let act = &s.resolver.scheduled[idx];
                let attack = matches!(act.command.kind, emberwake_battle::command::CmdKind::Attack { .. } | emberwake_battle::command::CmdKind::Area { .. });
                if attack && !act.cancelled && s.w.units[act.unit].is_alive() {
                    attacks += 1;
                    let want = t.anims[act.unit].want.as_ref().expect("the attacker queues its clip");
                    assert_eq!(want.1, now);
                    assert!(want.0.last() == Some(&"attack"), "{:?} ends with the generic attack", want.0);
                }
            }
        }
        assert!(moves > 0 && hits > 0 && falls > 0 && attacks > 0, "moves {moves}, hits {hits}, falls {falls}, attacks {attacks}");
    }

    /// A goblin_skirmish battle through the command host (the data's own encounter; no number of it is assumed).
    fn skirmish(seed: i64) -> Host {
        let dir = Data::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        let mut host = Host::new(Battle::with_data(Arc::new(Data::open(&dir).expect("data").0)));
        assert_eq!(host.handle(&json!({"cmd": "new", "encounter": "goblin_skirmish", "seed": seed}))["ok"], json!(true));
        host
    }

    #[test]
    fn facing_looks_toward_the_x_of_the_target_and_keeps_in_its_column() {
        let at = V2 { x: 4, y: 3 };
        assert_eq!(facing_toward(at, V2 { x: 2, y: 3 }), Some(true), "a target on the left");
        assert_eq!(facing_toward(at, V2 { x: 6, y: 0 }), Some(false), "a target on the right (any row)");
        assert_eq!(facing_toward(at, V2 { x: 4, y: 7 }), None, "same column: keep the facing");
    }

    /// FACE1: a new battle faces the party right and the foes left; a step turns a unit toward its x, a purely vertical
    /// one keeps the facing it had.
    #[test]
    fn a_new_battle_faces_the_party_right_and_a_vertical_step_keeps_the_facing() {
        let mut host = skirmish(3);
        let mut t = Tokens::default();
        t.catch_up(host.session.as_ref().unwrap(), 0, false, 300, true);
        let s = host.session.as_ref().unwrap();
        for i in 0..s.w.units.len() {
            assert_eq!(t.anims[i].left, !s.w.allies.contains(&i), "{} at the start", s.w.units[i].name);
        }
        let h = s.w.allies[0];
        let up = if s.w.grid.pos(h).expect("the hero stands on the grid").y > 0 { -1 } else { 1 };
        let mut walk = |t: &mut Tokens, dx: i32, dy: i32| {
            let s = host.session.as_mut().unwrap();
            let at = s.w.grid.pos(h).unwrap();
            s.w.grid.positions[h] = Some(V2 { x: at.x + dx, y: at.y + dy });
            t.catch_up(s, 0, false, 300, true);
            t.anims[h].left
        };
        assert!(!walk(&mut t, 0, up), "a vertical step keeps right");
        assert!(walk(&mut t, -1, 0), "a step left turns left");
        assert!(walk(&mut t, 0, -up), "a vertical step keeps left");
        assert!(walk(&mut t, -1, up), "a diagonal step goes by its x");
        assert!(!walk(&mut t, 1, 0), "a step right turns right");
    }

    /// FACE1: a unit that resolves an attack turns toward its target: left when the target stands left of it, right
    /// when it stands right (a foe too), and keeps its facing when the target is in its column; skip turns it the same.
    #[test]
    fn an_attacker_faces_its_target_on_either_side() {
        let mut host = skirmish(3);
        assert_eq!(host.handle(&json!({"cmd": "preset", "preset": "aggressive"}))["ok"], json!(true));
        assert_eq!(host.handle(&json!({"cmd": "confirm"}))["ok"], json!(true));
        let (mut t, mut skip) = (Tokens::default(), Tokens::default());
        let attacks: Vec<(usize, usize, usize)> = {
            let s = host.session.as_ref().unwrap();
            s.resolver.scheduled.iter().enumerate().filter_map(|(idx, a)| match a.command.kind {
                CmdKind::Attack { target: Some(tg), .. } if !a.cancelled && tg != a.unit && s.w.grid.pos(tg).is_some() => Some((idx, a.unit, tg)),
                _ => None,
            }).collect()
        };
        let mut sides = (false, false);
        let mut block = 0;
        for (idx, actor, target) in attacks {
            let tp = host.session.as_ref().unwrap().w.grid.pos(target).unwrap();
            for (dx, dy, keep) in [(1, 0, None), (-1, 0, None), (0, 1, Some(true)), (0, 1, Some(false))] {
                let s = host.session.as_mut().unwrap();
                s.w.grid.positions[actor] = Some(V2 { x: tp.x + dx, y: tp.y + dy });
                t.catch_up(s, 0, false, 300, true);
                skip.catch_up(s, 0, true, 300, true);
                if let Some(k) = keep {
                    (t.anims[actor].left, skip.anims[actor].left) = (k, k);
                }
                block += 1;
                s.last_popped = Some((s.current_round, block, idx));
                t.catch_up(s, 0, false, 300, true);
                skip.catch_up(s, 0, true, 300, true);
                let want = keep.unwrap_or(dx > 0);
                assert_eq!(t.anims[actor].faces_left(), want, "{} attacks from {dx},{dy} of its target", s.w.units[actor].name);
                assert_eq!(skip.anims[actor].faces_left(), want, "skip: the same facing");
            }
            if host.session.as_ref().unwrap().w.allies.contains(&actor) {
                sides.0 = true;
            } else {
                sides.1 = true;
            }
        }
        assert!(sides.0 && sides.1, "a hero's and a foe's attack were turned both ways: {sides:?}");
    }

    /// FACE1 (user 2026-10-23): with the enemy tells shown an enemy faces its intent's target while the party plans; with
    /// them hidden it keeps its facing through the planning and the confirm (until it acts). Heroes show their plan
    /// either way. An enemy is set left of its intent's target so its intent would turn it away from its start facing.
    #[test]
    fn hidden_tells_keep_the_enemies_facing_while_planning() {
        let mut host = skirmish(3);
        assert_eq!(host.handle(&json!({"cmd": "preset", "preset": "aggressive"}))["ok"], json!(true));
        let (e, target) = {
            let s = host.session.as_ref().unwrap();
            s.enemy_intents().into_iter().find_map(|r| r.line.target.filter(|t| s.w.grid.pos(*t).is_some()).map(|t| (r.unit, t))).expect("an enemy intends a unit")
        };
        let s = host.session.as_mut().unwrap();
        let (tp, home) = (s.w.grid.pos(target).unwrap(), s.w.grid.positions[e]);
        s.w.grid.positions[e] = Some(V2 { x: tp.x - 1, y: tp.y });
        let (mut shown, mut hidden) = (Tokens::default(), Tokens::default());
        shown.catch_up(s, 0, false, 300, true);
        hidden.catch_up(s, 0, false, 300, false);
        assert!(!shown.anims[e].faces_left(), "tells shown: {} faces its intent's target on its right", s.w.units[e].name);
        assert!(hidden.anims[e].faces_left(), "tells hidden: {} keeps its start facing (left)", s.w.units[e].name);
        for &h in &s.w.allies {
            assert_eq!(shown.anims[h].faces_left(), hidden.anims[h].faces_left(), "{} shows its plan either way", s.w.units[h].name);
        }
        let hero_planned = s.w.allies.iter().any(|&h| shown.anims[h].planned.is_some());
        assert!(hero_planned, "the aggressive preset gives a hero a target");
        // Back to its own tile (the test tile may be taken) so the plan confirms; the facing shown is kept. The tokens
        // take that tile as seen, so the teleport is no step.
        s.w.grid.positions[e] = home;
        (shown.anims[e].seen.pos, hidden.anims[e].seen.pos) = (home, home);
        assert_eq!(host.handle(&json!({"cmd": "confirm"}))["ok"], json!(true));
        let s = host.session.as_ref().unwrap();
        shown.catch_up(s, 0, false, 300, true);
        hidden.catch_up(s, 0, false, 300, false);
        assert!(!shown.anims[e].faces_left(), "tells shown: the confirm keeps the facing it showed");
        assert!(hidden.anims[e].faces_left(), "tells hidden: still its own facing after the confirm");
    }

    /// FACE1, played step by step (rule-derived over several seeds): after every step each unit faces the target of
    /// the action it just resolved when that stands in another column, else the x of its step, else it keeps its
    /// facing; while the party plans an enemy faces its tell's target. The skip setting ends every step in the same
    /// facing.
    #[test]
    fn units_face_their_targets_and_steps_and_skip_agrees() {
        let (mut turned_left, mut turned_right) = (0, 0);
        for seed in 1..=4 {
            let mut host = skirmish(seed);
            let (mut t, mut skip) = (Tokens::default(), Tokens::default());
            t.catch_up(host.session.as_ref().unwrap(), 0, false, 300, true);
            skip.catch_up(host.session.as_ref().unwrap(), 0, true, 300, true);
            let mut now = 0u64;
            for _ in 0..600 {
                let s = host.session.as_ref().unwrap();
                if s.combat_over {
                    break;
                }
                if s.phase == Phase::Replay {
                    if !s.can_advance_round() {
                        break;
                    }
                    host.handle(&json!({"cmd": "advance"}));
                }
                if host.session.as_ref().unwrap().is_planning() {
                    host.handle(&json!({"cmd": "preset", "preset": "aggressive"}));
                    let s = host.session.as_ref().unwrap();
                    t.catch_up(s, now, false, 300, true);
                    skip.catch_up(s, now, true, 300, true);
                    for r in s.enemy_intents() {
                        let to = r.line.target.and_then(|u| s.w.grid.pos(u)).unwrap_or(r.line.cell);
                        if let (Some(from), true) = (s.w.grid.pos(r.unit), to != NO_CELL) {
                            if let Some(left) = facing_toward(from, to) {
                                assert_eq!(t.anims[r.unit].faces_left(), left, "{} shows its tell's target", s.w.units[r.unit].name);
                            }
                        }
                    }
                }
                let before: Vec<(bool, Option<V2>)> = t.anims.iter().map(|a| (a.faces_left(), a.seen.pos)).collect();
                let popped = host.session.as_ref().unwrap().last_popped;
                assert_eq!(host.handle(&json!({"cmd": "step"}))["ok"], json!(true));
                now += 1000;
                let s = host.session.as_ref().unwrap();
                t.catch_up(s, now, false, 300, true);
                skip.catch_up(s, now, true, 300, true);
                let acted = s.last_popped.filter(|p| Some(*p) != popped && p.0 == s.current_round).map(|(_, _, idx)| &s.resolver.scheduled[idx]).filter(|a| !a.cancelled && s.w.units[a.unit].is_alive());
                for i in 0..s.w.units.len() {
                    let aim = acted.filter(|a| a.unit == i).and_then(|a| match &a.command.kind {
                        CmdKind::Attack { target, .. } => target.and_then(|u| s.w.grid.pos(u).or(before[u].1)),
                        CmdKind::Area { aim, .. } => Some(*aim),
                        CmdKind::Support { target, .. } | CmdKind::Item { target, .. } => s.w.grid.pos(*target).or(before[*target].1),
                        _ => None,
                    });
                    let turn = s.w.grid.pos(i).zip(aim.filter(|c| *c != NO_CELL)).and_then(|(from, to)| facing_toward(from, to));
                    let step = before[i].1.zip(s.w.grid.pos(i)).and_then(|(from, to)| facing_toward(from, to));
                    let want = turn.or(step).unwrap_or(before[i].0);
                    assert_eq!(t.anims[i].faces_left(), want, "seed {seed}: {} after the step", s.w.units[i].name);
                    assert_eq!(skip.anims[i].faces_left(), want, "seed {seed}: skip ends {} in the same facing", s.w.units[i].name);
                    match turn {
                        Some(true) => turned_left += 1,
                        Some(false) => turned_right += 1,
                        None => {}
                    }
                }
            }
        }
        assert!(turned_left > 0 && turned_right > 0, "actions turned left {turned_left}, right {turned_right}");
    }

    /// The Test Arena on `goblin_ambush` (the forest clearing) with the game's art, in battle view `view`.
    fn ambush(view: serde_json::Value) -> (crate::screens::Cx, aegis_ui::ui::Gfx, BattleScreen) {
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        cx.app.encounter = "goblin_ambush".into();
        cx.app.seed = Some(1);
        if !view.is_null() {
            cx.settings.v["battle_view"] = view;
        }
        let b = BattleScreen::sandbox(&cx);
        assert!(b.host.session.is_some(), "the ambush starts: {}", b.message);
        (cx, crate::hud::hud_tests::gfx(), b)
    }

    fn rect_of(b: &BattleScreen, id: &str) -> aegis_ui::canvas::Rect {
        b.w.iter().find(|w| w.id == id).unwrap_or_else(|| panic!("no widget {id}")).rect
    }

    /// STAGE1 check 10: the Battle view setting picks the board: Flat keeps today's 32 x 32 cells at (128, 30) and a
    /// unit's widget is its cell; Stage (also an unknown value and the default) the 32 x 24 cells from (128, 82).
    #[test]
    fn the_battle_view_setting_picks_the_stage_or_the_flat_board() {
        use aegis_ui::canvas::Rect;
        for (view, flat) in [(json!("flat"), true), (json!("stage"), false), (json!("sideways"), false), (serde_json::Value::Null, false)] {
            let (mut cx, mut g, mut b) = ambush(view.clone());
            b.draw(&mut cx, &mut g, (-500, -500), true);
            let (cell, last) = (rect_of(&b, "tile.0.0"), rect_of(&b, "tile.11.8"));
            if flat {
                assert_eq!((cell, last), (Rect::new(128, 30, 32, 32), Rect::new(480, 286, 32, 32)), "{view}");
                let s = b.host.session.as_ref().unwrap();
                let h = s.w.allies[0];
                let p = s.w.grid.pos(h).unwrap();
                assert_eq!(rect_of(&b, &format!("unit.{h}")), Rect::new(128 + p.x * 32, 30 + p.y * 32, 32, 32), "a unit's widget is its cell");
            } else {
                assert_eq!((cell, last), (Rect::new(128, 82, 32, 24), Rect::new(480, 274, 32, 24)), "{view}");
            }
            assert_eq!(b.w.iter().filter(|w| w.id.starts_with("tile.")).count(), 12 * 9, "every cell keeps its widget");
        }
    }

    /// STAGE1 check 6: on the Stage the hero's topmost solid pixel (his helmet, above his cell) clicks him; the top-left
    /// 2 x 2 px of a free cell north of a unit click that cell; where two units' rects overlap the southern one wins;
    /// the backdrop and the front face hold no cell.
    #[test]
    fn stage_clicks_reach_heads_free_cells_and_the_unit_in_front() {
        let (mut cx, mut g, mut b) = ambush(json!("stage"));
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let geo = Geom::new(View::Stage, &cx.tuning.value);
        let (h, pos, body, palette) = {
            let s = b.host.session.as_ref().unwrap();
            let h = s.w.allies[0];
            (h, s.w.grid.pos(h).unwrap(), cx.art.first_body(&emberwake_content::art::body_chain(&s.art[h])).expect("the hero has a body"), s.palettes.get(h).cloned().unwrap_or_default())
        };
        assert_eq!(h, 0, "the first hero is unit 0");
        // before the first change the tokens have no animations yet and a hero faces right (as the draw does)
        let left = b.tokens.anims.get(h).is_some_and(|a| a.faces_left());
        let bb = cx.art.body_bounds(&body, &palette, geo.token_px, left).expect("solid pixels");
        let frame = cx.art.body_clip_flipped(&body, &palette, "idle", 0, geo.token_px, left).unwrap().clone();
        let top_x = (bb.x..bb.x + bb.w).find(|x| frame.px[((bb.y * frame.w + x) * 4 + 3) as usize] >= aegis_ui::canvas::OUTLINE_ALPHA).unwrap();
        let tr = geo.token_rect(pos, 0, 0);
        let helmet = (tr.x + top_x, tr.y + bb.y);
        assert!(helmet.1 < geo.cell_rect(pos).y, "the helmet is above the hero's cell");
        assert_eq!(aegis_ui::ui::hit(&b.w, helmet), Some(Action::UnitClick(h)), "the helmet pixel clicks the hero");
        // A free cell north of a unit.
        let free = {
            let s = b.host.session.as_ref().unwrap();
            s.w.grid.positions.iter().flatten().map(|p| V2 { x: p.x, y: p.y - 1 }).find(|c| s.w.grid.is_valid(*c) && s.w.occupant(*c).is_none()).expect("a free cell north of a unit")
        };
        let r = geo.cell_rect(free);
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            assert_eq!(aegis_ui::ui::hit(&b.w, (r.x + dx, r.y + dy)), Some(Action::TileClick(free)), "the corner of free cell {free}");
        }
        for pt in [(300, 50), (300, 305)] {
            assert!(!matches!(aegis_ui::ui::hit(&b.w, pt), Some(Action::TileClick(_))), "{pt:?} holds no cell");
        }
        // Two units in one column: the southern one wins where their rects overlap.
        let (north, south) = {
            let s = b.host.session.as_mut().unwrap();
            let other = (0..s.w.units.len()).find(|i| *i != h && s.w.units[*i].is_alive() && s.w.grid.pos(*i).is_some()).expect("another unit");
            let below = if pos.y + 1 < GRID_H { V2 { x: pos.x, y: pos.y + 1 } } else { V2 { x: pos.x, y: pos.y - 1 } };
            s.w.grid.positions[other] = Some(below);
            if below.y > pos.y { (h, other) } else { (other, h) }
        };
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let (rn, rs) = (rect_of(&b, &format!("unit.{north}")), rect_of(&b, &format!("unit.{south}")));
        let m = rn.intersect(rs);
        assert!(m.w > 0 && m.h > 0, "the southern unit's head reaches into the northern one's rect");
        assert_eq!(aegis_ui::ui::hit(&b.w, (m.x + m.w / 2, m.y + m.h / 2)), Some(Action::UnitClick(south)), "the unit in front wins");
        assert_eq!(aegis_ui::ui::hit(&b.w, (rn.x + rn.w / 2, rn.y + 1)), Some(Action::UnitClick(north)), "the unit behind still clicks on its head");
    }

    #[test]
    fn a_boss_tier_enemy_makes_a_boss_fight() {
        let dir = Data::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        let d = Data::open(&dir).expect("data").0;
        let fight = |id: &str| {
            let def = EncounterDef::from_obj(d.resolve("ashlands", "encounters", id));
            boss_fight(&Session::start(d.cat.clone(), def, &WorldRoster { d: &d, world: "ashlands".into() }))
        };
        assert!(fight("ashmaw_stand"));
        assert!(!fight("courtyard_skirmish"));
    }

    #[test]
    fn log_lines_pick_their_sounds() {
        assert_eq!(sound_of("[Attack] Rook struck Goblin for 7 (HP 3/10)"), Some("hit"));
        assert_eq!(sound_of("[Attack] Rook struck Goblin for 14 CRIT (HP 0/10)"), Some("crit"));
        assert_eq!(sound_of("[Attack] Rook struck Goblin for 0 (HP 10/10)"), Some("mitigated"));
        assert_eq!(sound_of("[Attack Miss] Rook: target no longer in combat"), Some("miss"));
        assert_eq!(sound_of("[Area Miss] Mage hit nobody"), Some("miss"));
        assert_eq!(sound_of("[Status] Hex -> Goblin (inflicts poison)"), Some("debuff"));
        assert_eq!(sound_of("[Status] Hex -> Goblin (resisted poison)"), None);
        assert_eq!(sound_of("[Status] Bard -> Rook (inflicts haste)"), None);
        assert_eq!(sound_of("[Defend] Rook raises guard (40% less damage)"), Some("defend"));
        assert_eq!(sound_of("[Defend] Rook's guard was broken"), None);
        assert_eq!(sound_of("[Support] Cleric used Mend on Rook: healed 6"), Some("heal"));
        assert_eq!(sound_of("[Move] Rook moved 2 of 2 tiles to (3, 4)"), Some("move_step"));
        assert_eq!(sound_of("[Rest] Rook AP 3/4"), None);
    }
}
