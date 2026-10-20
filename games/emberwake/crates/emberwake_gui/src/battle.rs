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
use crate::screens::{show_errors, Cx, GameCx, Key, Nav, Screen};
use emberwake_battle::intent::Intent;
use aegis_ui::audio::Bus;
use aegis_ui::painter::Painter;
use aegis_ui::ui::{Gfx, Tip, Ui, Widget, BODY, H, RADIUS, SMALL, TITLE, W};
use emberwake_battle::command::CmdKind;
use emberwake_battle::grid::{V2, GRID_H, GRID_W};
use emberwake_battle::plan::expected;
use emberwake_battle::session::{EncounterDef, Phase, Prebuilt, Session, StartGrant};
use aegis_core::json::{self, s, Dict};
use emberwake_world::battle::WorldRoster;
use emberwake_battle::skill::Kind;
use emberwake_battle::world::{AiProfile, World};
use emberwake_data::UnitDef;
use emberwake_host::{Battle, Host};
use emberwake_world::{dialogue, explore, progression, travel};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

const TILE: i32 = 32;
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
    /// (requests, checkpoints) of the session at its last save.
    saved: (usize, usize),
    /// Playtest trail events of the battle (plan edits, confirms, the end), handed to the hub on the next tick.
    trail_out: Vec<Value>,
    /// The battle's end is in the trail.
    end_logged: bool,
    /// Sessions an agent-link `new` ended before the hub could be reached (saved on the next tick).
    parked: Vec<(Option<usize>, aegis_host::playtest::Session)>,
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
            saved: (0, 0),
            parked: Vec::new(),
            trail_out: Vec::new(),
            end_logged: false,
        }
    }

    /// A session worth recording: it starts with `new`, so it replays (a party battle of the world map starts outside
    /// it: PLAYTEST2).
    fn recordable(&self) -> bool {
        self.host.history.first().is_some_and(|r| r["cmd"] == json!("new"))
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
        cx.playtest.put_session(&mut self.slot, aegis_host::playtest::Session::of(&self.host));
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
            let art = if !base.art.is_empty() { base.art.clone() } else { d.cat.classes.get(&base.class_id).map(|c| c.art.clone()).unwrap_or_else(|| "unit".into()) };
            party.push(Prebuilt { unit, art, profile: AiProfile::for_unit(&d.cat, &base), key: m.id.clone() });
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
            "travel" => travel::after_fight(&d, gs, fight, &outcome),
            "dialogue" => dialogue::after_fight(&d, gs, fight, &outcome),
            _ => explore::after_fight(&d, gs, fight, &outcome),
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

    fn tick(&mut self, cx: &mut Cx) {
        let now = Instant::now();
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
            self.parked.push((self.slot, aegis_host::playtest::Session::of(&self.host)));
        }
        let reply = self.host.handle(body);
        if restart && reply["ok"] == json!(true) {
            self.new_session();
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

fn hp_bar(p: &mut Painter, r: Rect, hp: i32, max: i32, color: Rgba) {
    p.fill(r, rgb(46, 24, 30));
    let w = if max > 0 { (r.w * hp.max(0) + max - 1) / max } else { 0 };
    p.fill(Rect::new(r.x, r.y, w.min(r.w), r.h), color);
}

fn cell_rect(c: V2) -> Rect {
    Rect::new(GRID_X + c.x * TILE, GRID_Y + c.y * TILE, TILE, TILE)
}

fn terrain_color(id: &str) -> Rgba {
    match id {
        "wall" => rgb(46, 46, 54),
        "tree" => rgb(30, 70, 40),
        "deep_water" => rgb(30, 54, 110),
        "shallow_water" => rgb(56, 92, 140),
        "lava" => rgb(150, 50, 20),
        "grass" => rgb(60, 104, 56),
        "dirt" => rgb(104, 84, 56),
        "sand" | "snow" => rgb(170, 160, 120),
        "stone" | "rubble" => rgb(92, 92, 100),
        _ => rgb(80, 82, 92),
    }
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
    ui.p.panel(Rect::new(2, 2, W - 4, 24), PANEL, BORDER, RADIUS);
    let hero_name = sel.map(|u| w.units[u].name.clone()).unwrap_or_else(|| "No hero".into());
    ui.p.text_fit(8.0, 9.0, &format!("{hero_name} plan"), TITLE, 80.0, GOLD);
    ui.push("strip.title", "label", format!("{hero_name} plan"), Rect::new(8, 8, 80, 10), true, false, None, Vec::new());
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
    let (bx, bw) = (92, 34);
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
    let mut x = bx + 6 * bw + 6;
    ui.dropdown("strip.party", Rect::new(x, 6, 82, 16), &format!("Party: {}", TACTIC_LABELS[g.party_tactic]), Action::PartyMenu, planning, vec![("Party tactic".into(), GOLD), ("AI-drafts every hero with a tactic (you can edit the steps after).".into(), TEXT)]);
    x += 84;
    ui.dropdown("strip.hero", Rect::new(x, 6, 80, 16), &format!("Hero: {}", TACTIC_LABELS[g.hero_tactic]), Action::HeroMenu, planning && sel.is_some(), vec![("Hero tactic".into(), GOLD), ("AI-drafts the selected hero only.".into(), TEXT)]);
    x += 82;
    ui.button_full("strip.auto", Rect::new(x, 6, 34, 16), "Auto", Action::Autoplay, !s.combat_over, g.auto, BODY, vec![("Autoplay".into(), GOLD), ("Drafts with the party tactic, confirms and plays rounds until the battle ends. Click again to stop.".into(), TEXT)]);
    x += 36;
    ui.button_full("strip.clear", Rect::new(x, 6, 34, 16), "Clear", Action::Clear, planning && sel.is_some(), false, BODY, vec![("Clears the selected hero's steps.".into(), TEXT)]);
    x += 36;
    let (label, action, enabled) = if planning {
        ("Confirm round", Action::Confirm, true)
    } else if s.phase == Phase::Replay && s.can_advance_round() {
        ("Next round", Action::NextRound, !g.pending_next_round)
    } else {
        (if s.combat_over { "Battle over" } else if s.phase == Phase::Replay { "Round limit" } else { "Resolving" }, Action::NextRound, false)
    };
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
    if let Some(h) = sel {
        let u = &w.units[h];
        let r = Rect::new(6, y + 2, 116, 318 - y - 6);
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

    // -- Grid (pixel layer): terrain, exits, plan ghost, intents, units, floating numbers.
    ui.p.panel(Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4), rgb(8, 10, 16), BORDER, RADIUS);
    for gy in 0..GRID_H {
        for gx in 0..GRID_W {
            let c = V2::new(gx, gy);
            let r = cell_rect(c);
            let tid = w.grid.terrain_id(c);
            match cx.art.tile(tid, gx, gy, TILE) {
                Some(img) => ui.p.image(img, r),
                None => ui.p.fill(r, terrain_color(tid)),
            }
            if w.grid.is_exit(c) {
                ui.p.panel(r.inset(7), rgba(70, 200, 90, 220), rgba(200, 255, 210, 255), 2.0);
                ui.p.text_center(r.x as f32 + 16.0, r.y as f32 + 11.0, "EXIT", SMALL, BG);
            }
            if planning && sel.is_some() && ui.hover(r) && g.popup.is_none() {
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
    for i in 0..w.units.len() {
        let Some(pos) = w.grid.pos(i) else { continue };
        let u = &w.units[i];
        let r = cell_rect(pos);
        let ally = w.allies.contains(&i);
        if !u.is_alive() {
            ui.p.text_center(r.x as f32 + 16.0, r.y as f32 + 11.0, "x", BODY, DIM);
            continue;
        }
        let flash = g.flashes.iter().any(|f| f.0 == i);
        let shake = if flash { tuning.hit_shake_px as f32 * if (now.elapsed().as_millis() / 30) % 2 == 0 { 1.0 } else { -1.0 } } else { 0.0 };
        let tp = tuning.token_px;
        let tr = Rect::new(r.x + (TILE - tp) / 2 + shake as i32, r.y + (TILE - tp) / 2, tp, tp);
        let ring = if ally { ALLY } else { FOE };
        ui.p.circle(r.x as f32 + 16.0 + shake, r.y as f32 + 16.0, tp as f32 / 2.0 + 0.5, ring);
        match cx.art.token(&s.art[i], tp) {
            Some(img) => ui.p.image(img, tr),
            None => {
                ui.p.panel(tr.inset(4), ring, ring, 6.0);
                ui.p.text_center(r.x as f32 + 16.0, r.y as f32 + 11.0, &u.name.chars().next().unwrap_or('?').to_string(), BODY, BG);
            }
        }
        if flash {
            ui.p.fill(tr, rgba(255, 255, 255, 110));
        }
        hp_bar(&mut ui.p, Rect::new(r.x + 3, r.y + TILE - 3, TILE - 6, 2), u.hp, u.max_hp, if ally { GOOD } else { FOE });
        if Some(i) == sel {
            ui.p.panel(r, rgba(0, 0, 0, 0), GOLD, 2.0);
        }
        let tip = unit_tip(s, i, sel, &tells);
        ui.push(format!("unit.{i}"), "unit", format!("{} {}/{}", u.name, u.hp, u.max_hp), r, true, Some(i) == sel, Some(Action::UnitClick(i)), tip);
    }
    if show_tells {
        for r in &tells {
            let Some(from) = w.grid.pos(r.unit) else { continue };
            let (fx, fy) = center(from);
            let to = r.line.target.and_then(|t| w.grid.pos(t)).or((r.line.cell != emberwake_battle::grid::NO_CELL).then_some(r.line.cell));
            if let Some(to) = to.filter(|c| *c != from) {
                let (tx, ty) = center(to);
                let (dx, dy) = (tx - fx, ty - fy);
                let len = (dx * dx + dy * dy).sqrt().max(1.0);
                // From the token's edge to just short of the target's.
                let (sx, sy, ex, ey) = (fx + dx / len * 10.0, fy + dy / len * 10.0, tx - dx / len * 9.0, ty - dy / len * 9.0);
                let dash = if matches!(r.line.kind, "move" | "flee") { 3.0 } else { 0.0 };
                ui.p.arrow(sx, sy, ex, ey, rgba(240, 100, 92, 200), dash, 4.0);
            }
            let chip = Rect::new(GRID_X + from.x * TILE + 1, GRID_Y + from.y * TILE + 1, 10, 10);
            match cx.art.get("ui", &format!("intent_{}", if r.line.kind == emberwake_battle::intent::IDLE { "rest" } else { r.line.kind }), "icon", 10) {
                Some(img) => ui.p.image(img, chip),
                None => {
                    ui.p.panel(chip, rgba(48, 28, 38, 230), FOE, 2.0);
                    ui.p.text_center(chip.x as f32 + 5.0, chip.y as f32 + 1.0, &r.line.kind[..1].to_uppercase(), SMALL, TEXT);
                }
            }
        }
    }
    for f in &g.floats {
        if let Some(pos) = w.grid.pos(f.unit) {
            let t = now.duration_since(f.born).as_millis() as f32 / tuning.float_ms.max(1) as f32;
            let (cx, cy) = center(pos);
            let yy = cy - 14.0 - tuning.float_rise_px as f32 * t;
            ui.p.text_center(cx + 0.6, yy + 0.6, &f.text, TITLE, rgba(0, 0, 0, 220));
            ui.p.text_center(cx, yy, &f.text, TITLE, f.color);
        }
    }

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
        let r = Rect::new(right.x + 4, y, 116, 22);
        let hover = ui.hover(r) || w.grid.pos(e).is_some_and(|p| ui.hover(cell_rect(p)));
        if hover {
            hovered_foe = Some(e);
        }
        ui.p.panel(r, if hover { rgba(48, 28, 38, 255) } else { CARD }, if hover { FOE } else { BORDER }, RADIUS);
        let col = if u.is_active() { FOE } else { DIM };
        ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 1.5, &short_name(&u.name), BODY, 70.0, col);
        ui.p.text_right(r.x as f32 + r.w as f32 - 3.0, r.y as f32 + 1.5, &format!("{}/{}", u.hp, u.max_hp), BODY, if u.is_active() { GOOD } else { DIM });
        let tell = tells.iter().find(|t| t.unit == e);
        if u.is_active() {
            let next = tell.map(|t| intent_text::card_line(w, &t.line)).unwrap_or_else(|| format!("Next: {}", next_action(s, e)));
            ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 11.5, &next, SMALL, if tell.is_some_and(|t| t.revealed) { 98.0 } else { 110.0 }, GOLD);
            if tell.is_some_and(|t| t.revealed) {
                if let Some(img) = cx.art.get("ui", "badge_foresight", "icon", 9) {
                    ui.p.image(img, Rect::new(r.x + r.w - 11, r.y + 11, 9, 9));
                }
            }
        } else {
            ui.p.text(r.x as f32 + 3.0, r.y as f32 + 11.5, if u.is_withdrawn { "Fled" } else { "Down" }, SMALL, DIM);
        }
        let tip = unit_tip(s, e, sel, &tells);
        ui.push(format!("foe.{e}"), "card", format!("{} {}/{} {}", u.name, u.hp, u.max_hp, tell.map(|t| intent_text::card_line(w, &t.line)).unwrap_or_else(|| format!("Next: {}", next_action(s, e)))), r, true, hover, Some(Action::UnitClick(e)), tip);
        y += 24;
    }
    let detail_foe = hovered_foe.or_else(|| w.enemies.iter().copied().find(|e| w.units[*e].is_active() && w.grid.pos(*e).is_some()));
    if let Some(e) = detail_foe {
        let r = Rect::new(right.x + 4, y + 2, 116, 318 - y - 6);
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

#[cfg(test)]
mod tests {
    use super::{boss_fight, sound_of};
    use emberwake_battle::session::{EncounterDef, Session};
    use emberwake_world::battle::WorldRoster;
    use emberwake_world::Data;

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
