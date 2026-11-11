//! Campaign runtime (CAMPAIGN1, kit hook; engine-only, no Godot original): the campaign state `gs.campaign` (11.5),
//! the one entry point `tick` (X4), the meters' rules and inputs (`meter.rs`), the act and its floors (M4), the
//! campaign effects (11.3), the read keys (11.4) and the run-over state (F1, F3). Everything is inert unless the
//! world's lineage sets the manifest field `campaign` (X1): no state, no result key, no read key answered.
//!
//! Built in CAMPAIGN1-M: gauges, bands, rules, inputs (battle, day, leg), floors, `set_act`, `meter_add`,
//! `meter_set`, `meter_show`, `end`, the read keys and the run-over state. CAMPAIGN1-C: the days and leg ticks
//! (travel, rest, retreat), the clock effects (`clock.rs`), checkpoints (`checkpoint.rs`), the purse, rests, the
//! retreat, road-fight modifiers and `battle_mods` (`purse.rs`). CAMPAIGN1-T: triggers (`trigger.rs`, after the
//! checkpoints; `Cause::Enter` from `explore::enter`), the follow-up ticks (owed rest days, act changes inside a tick),
//! and the previews of a run-ending step (`preview.rs`). Story-free: a `then` item (a vignette, a battle, a scene) is
//! queued in `pending` and handed on through `pending_head` / `pop_pending` (the GUI plays it; `{vignette}` items at
//! the head move to the story queue through `story::pull_pending`, which `story::next_scene` calls, STORYUI1-W; a
//! `{battle}` item is `trigger::fight`). Every effect list (meter rules, checkpoints, triggers) runs through the one
//! shared effect body, `dialogue::apply_effects`. CAMPAIGN1-U: the HUD view (`view.rs`, W4) the game draws.

pub mod checkpoint;
pub mod clock;
pub mod meter;
pub mod preview;
pub mod purse;
pub mod trigger;
pub mod view;

pub use preview::{preview_battle, preview_days, preview_trip};
pub use view::hud;

use crate::data::Data;
use crate::quest;
use crate::state::GameState;
use aegis_core::ghash::string_hash;
use aegis_core::json::{self, int, Dict};
use indexmap::IndexMap;
use meter::{tags_match, MeterDef, MeterState};
use rpgkit_data::CampaignManifest;
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::Arc;

/// X4: a rule's effects that move a meter re-enter the rule pass at most this many times per tick (the first pass
/// plus REENTRIES_MAX more); a crossing still left after the last pass is not fired: the tick's end takes the values
/// as the new baseline. A tick also runs at most this many follow-up ticks (owed rest days, an act change); what is
/// left of them waits for the next cause.
pub const REENTRIES_MAX: usize = 8;
/// 11.5: the keys of the saved campaign block (CAMPAIGN1-TM: `mode`, written only when not the default).
pub const BLOCK_KEYS: [&str; 8] = ["act", "run", "meters", "fired", "armed", "checkpoints", "pending", "mode"];

/// CAMPAIGN1-TM2: the load note of a saved mode `id` the world no longer has (the save loads in the default mode).
pub fn mode_dropped_text(id: &str) -> String {
    format!("campaign.mode: {} Loaded in the default mode.", purse::unknown_mode_text(id))
}
/// Acts are numbered 0 (before the first) to ACT_MAX.
pub const ACT_MAX: i64 = 9;
/// F3: the refusal of every save while the run is over.
pub const RUN_OVER_TEXT: &str = "The run is over.";
/// 11.4: the refusal of `set_flag` and `add_flag` on a read key.
pub fn reserved_text(key: &str) -> String {
    format!("Flag key '{key}' is reserved (meter., campaign. and roster. keys are read-only).")
}
/// T2: the keys of a `then` item (exactly one of the first three).
pub const THEN_KINDS: [&str; 3] = ["vignette", "battle", "dialogue"];

// ---- manifest ----

/// The nearest manifest block `key` along the world's lineage that holds anything ({} = today's rules; 11.1).
pub fn manifest_block(d: &Data, w: &str, key: &str) -> Dict {
    for lw in d.lineage(w).iter().rev() {
        if let Some(Value::Object(o)) = d.manifest(lw).get(key) {
            if !o.is_empty() {
                return o.clone();
            }
        }
    }
    Dict::new()
}

/// The world's `campaign` block (X1: `on` once it holds anything).
pub fn manifest(d: &Data, w: &str) -> CampaignManifest {
    CampaignManifest::from_obj(&manifest_block(d, w, "campaign"))
}

/// X1: whether world `w` runs a campaign.
pub fn is_on(d: &Data, w: &str) -> bool {
    manifest(d, w).on
}

/// The meter rows of world `w` (merged along the lineage; a nearer row with the same id replaces a farther one in
/// place), read with their defaults.
pub fn meter_defs(d: &Data, w: &str) -> IndexMap<String, MeterDef> {
    let mut out: IndexMap<String, MeterDef> = IndexMap::new();
    for e in d.merged_entries(w, "meters") {
        let def = MeterDef::from_row(e);
        if !def.id.is_empty() {
            out.insert(def.id.clone(), def);
        }
    }
    out
}

/// F5: the run id, 8 hex digits from the new game's seed.
pub fn run_id(seed: i64) -> String {
    format!("{:08x}", string_hash(&format!("run|{seed}")))
}

/// F5: the key of the act-start save of act `act` in run `run`.
pub fn act_save_key(act: i64, run: &str) -> String {
    format!("act{act}_{run}")
}

// ---- state (11.5) ----

/// F1: the run is over (runtime only, never saved: F3).
#[derive(Debug, Clone, PartialEq)]
pub struct RunOver {
    /// One of `rpgkit_data::END_KINDS`.
    pub kind: String,
    pub card: String,
    pub text: String,
    /// The `run_over` event was reported (later ticks do nothing).
    pub told: bool,
}

/// `gs.campaign` (11.5), saved as the key `campaign` only in a campaign world (X3).
#[derive(Debug, Clone)]
pub struct Campaign {
    pub act: i64,
    pub run: String,
    /// Meter id -> state, in the meters' data order.
    pub meters: IndexMap<String, MeterState>,
    /// Once-items fired, sorted: `rule:<meter>/<rule>`, `trigger:<id>`.
    pub fired: Vec<String>,
    /// Repeatable trigger ids that fired and whose `when` has held at every tick since (T1: one leaves when its `when`
    /// stops holding and may then fire again).
    pub armed: Vec<String>,
    /// Checkpoint id -> "met" or "missed" (CAMPAIGN1-C).
    pub checkpoints: IndexMap<String, String>,
    /// Queued `then` items, head first (T2).
    pub pending: Vec<Dict>,
    /// CAMPAIGN1-TM: the travel mode the party picked ("" = the world's default; saved only when set).
    pub mode: String,
    /// Runtime: the run is over (F1), never saved.
    pub over: Option<RunOver>,
    /// Runtime: the world's meter rows, read when the state was made or loaded.
    pub defs: Arc<IndexMap<String, MeterDef>>,
    /// Runtime: each meter's value at the end of the last tick (a rule fires on the move from here to now; M5).
    pub last: IndexMap<String, i64>,
    /// Runtime: the travel clock (`gs.time`) at the end of the last tick (checkpoint warnings and deadlines, C1, C2).
    pub last_time: i64,
    /// Runtime: days a `rest` effect passed inside an effect list; they run as their own days(n, rest) tick within the
    /// next `tick` call (first, after a dialogue list; right after the tick whose rule, checkpoint or trigger rested).
    pub owed_days: i64,
    /// Runtime: the load notes `from_value` left (CAMPAIGN1-TM2: a saved mode the world no longer has); `save::decode`
    /// takes them as load warnings. Never saved.
    pub load_notes: Vec<String>,
}

/// Equal when the saved parts are (the runtime parts follow from them or are never saved).
impl PartialEq for Campaign {
    fn eq(&self, o: &Self) -> bool {
        self.act == o.act && self.run == o.run && self.meters == o.meters && self.fired == o.fired && self.armed == o.armed && self.checkpoints == o.checkpoints && self.pending == o.pending && self.mode == o.mode
    }
}

impl Campaign {
    /// A new game's campaign in world `w` (None without one, X1): act `start_act`, each meter's start, run id from
    /// `seed`.
    pub fn fresh(d: &Data, w: &str, seed: i64) -> Option<Self> {
        let m = manifest(d, w);
        if !m.on {
            return None;
        }
        let defs = meter_defs(d, w);
        let meters = defs.iter().map(|(id, def)| (id.clone(), MeterState::fresh(def))).collect();
        let mut c = Self {
            act: m.start_act as i64,
            run: run_id(seed),
            meters,
            fired: Vec::new(),
            armed: Vec::new(),
            checkpoints: IndexMap::new(),
            pending: Vec::new(),
            mode: String::new(),
            over: None,
            defs: Arc::new(defs),
            last: IndexMap::new(),
            last_time: 0,
            owed_days: 0,
            load_notes: Vec::new(),
        };
        c.sync(0);
        Some(c)
    }

    /// A meter's value now (M1; 0 for an unknown id).
    pub fn value(&self, id: &str, time: i64) -> i64 {
        match (self.defs.get(id), self.meters.get(id)) {
            (Some(def), Some(st)) => st.value(def, time),
            _ => 0,
        }
    }

    /// The baseline of the next rule pass and checkpoint step: every meter's value at `time`, and `time`.
    pub fn sync(&mut self, time: i64) {
        let values: IndexMap<String, i64> = self.meters.keys().map(|id| (id.clone(), self.value(id, time))).collect();
        self.last = values;
        self.last_time = time;
    }

    pub fn is_over(&self) -> bool {
        self.over.is_some()
    }

    pub fn to_value(&self) -> Value {
        let mut meters = Dict::new();
        for (id, st) in &self.meters {
            if let Some(def) = self.defs.get(id) {
                meters.insert(id.clone(), st.to_value(def));
            }
        }
        let mut fired = self.fired.clone();
        fired.sort();
        let mut out = Value::Object(json::obj([
            ("act", int(self.act)),
            ("run", Value::String(self.run.clone())),
            ("meters", Value::Object(meters)),
            ("fired", json::strs(&fired)),
            ("armed", json::strs(&self.armed)),
            ("checkpoints", Value::Object(self.checkpoints.iter().map(|(k, v)| (k.clone(), Value::String(v.clone()))).collect())),
            ("pending", Value::Array(self.pending.iter().cloned().map(Value::Object).collect())),
        ]));
        // CAMPAIGN1-TM: the default mode writes nothing, so saves without a mode pick stay as before
        if let (Value::Object(o), false) = (&mut out, self.mode.is_empty()) {
            o.insert("mode".into(), Value::String(self.mode.clone()));
        }
        out
    }

    /// From a saved `campaign` value in world `w` (time `time`, for the baseline); a missing key keeps the fresh
    /// state's, a saved meter the world no longer has is a load problem, a meter the save lacks starts fresh.
    pub fn from_value(d: &Data, w: &str, seed: i64, time: i64, raw: &Value, problems: &mut Vec<String>) -> Option<Self> {
        let mut c = Self::fresh(d, w, seed)?;
        let Some(o) = raw.as_object() else {
            problems.push("campaign must be an object {act, run, meters, fired, armed, checkpoints, pending}.".into());
            return Some(c);
        };
        for k in o.keys().filter(|k| !BLOCK_KEYS.contains(&k.as_str())) {
            problems.push(format!("campaign: unknown key '{k}'."));
        }
        if let Some(v) = o.get("act") {
            if json::is_int(v) {
                c.act = json::to_int(v);
            } else {
                problems.push("campaign.act must be a whole number.".into());
            }
        }
        if let Some(v) = o.get("run") {
            match v.as_str() {
                Some(s) => c.run = s.to_string(),
                None => problems.push("campaign.run must be text.".into()),
            }
        }
        let empty = Dict::new();
        let mut object = |key: &str| -> &Dict {
            match o.get(key) {
                None => &empty,
                Some(Value::Object(m)) => m,
                Some(_) => {
                    problems.push(format!("campaign.{key} must be an object."));
                    &empty
                }
            }
        };
        let (meters, checkpoints) = (object("meters"), object("checkpoints"));
        for (id, st) in meters {
            match c.defs.get(id) {
                Some(def) => {
                    let parsed = MeterState::from_value(def, st, problems);
                    c.meters.insert(id.clone(), parsed);
                }
                None => problems.push(format!("campaign.meters: '{id}' is not a meter of this world.")),
            }
        }
        let mut strings = |key: &str, slot: &mut Vec<String>| {
            if let Some(v) = o.get(key) {
                match v.as_array().filter(|a| a.iter().all(Value::is_string)) {
                    Some(a) => *slot = a.iter().map(json::gstr).collect(),
                    None => problems.push(format!("campaign.{key} must be a list of text.")),
                }
            }
        };
        strings("fired", &mut c.fired);
        strings("armed", &mut c.armed);
        for (id, v) in checkpoints {
            match v.as_str() {
                Some(st) => {
                    c.checkpoints.insert(id.clone(), st.to_string());
                }
                None => problems.push(format!("campaign.checkpoints.{id} must be met or missed.")),
            }
        }
        // CAMPAIGN1-TM: an absent mode is the default (older saves); CAMPAIGN1-TM2 (decision "Saves and data edits"): a
        // mode the world no longer has loads in the default with a load note
        if let Some(v) = o.get("mode") {
            match v.as_str() {
                Some(m) if m.is_empty() || purse::travel_modes(d, w).get(m).is_some() => {
                    c.mode = if m == purse::travel_modes(d, w).default { String::new() } else { m.to_string() };
                }
                Some(m) => c.load_notes.push(mode_dropped_text(m)),
                None => problems.push("campaign.mode must be text.".into()),
            }
        }
        if let Some(v) = o.get("pending") {
            match v.as_array() {
                Some(a) => {
                    for item in a {
                        match item.as_object() {
                            Some(d) => c.pending.push(d.clone()),
                            None => problems.push("campaign.pending items must be objects.".into()),
                        }
                    }
                }
                None => problems.push("campaign.pending must be a list.".into()),
            }
        }
        c.sync(time);
        Some(c)
    }

    /// GameState::validate's part (11.5): meters known and in range, act 0-9, the run id, fired, armed, checkpoints
    /// and pending items well formed.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if !(0..=ACT_MAX).contains(&self.act) {
            errors.push(format!("campaign.act must be 0..{ACT_MAX}."));
        }
        if self.run.len() != 8 || !self.run.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
            errors.push(format!("campaign.run '{}' must be 8 hex digits.", self.run));
        }
        for (id, st) in &self.meters {
            let Some(def) = self.defs.get(id) else {
                errors.push(format!("campaign.meters: '{id}' is not a meter of this world."));
                continue;
            };
            if !def.countdown && !(def.min..=def.max).contains(&st.value) {
                errors.push(format!("campaign.meters.{id}.value must be {}..{}.", def.min, def.max));
            }
            if def.countdown && st.due < 0 {
                errors.push(format!("campaign.meters.{id}.due must be >= 0."));
            }
        }
        for f in &self.fired {
            let ok = f.strip_prefix("rule:").is_some_and(|r| r.split_once('/').is_some_and(|(m, r)| json::id_ok(m) && json::id_ok(r)))
                || f.strip_prefix("trigger:").is_some_and(|t| !t.is_empty());
            if !ok {
                errors.push(format!("campaign.fired: '{f}' must be rule:<meter>/<rule> or trigger:<id>."));
            }
        }
        for (i, id) in self.armed.iter().enumerate() {
            if id.is_empty() || self.armed[..i].contains(id) {
                errors.push(format!("campaign.armed: '{id}' must be a trigger id, listed once."));
            }
        }
        for (id, st) in &self.checkpoints {
            if st != "met" && st != "missed" {
                errors.push(format!("campaign.checkpoints.{id} must be met or missed."));
            }
        }
        for item in &self.pending {
            let kinds: Vec<&str> = THEN_KINDS.iter().copied().filter(|k| item.contains_key(*k)).collect();
            let ok = kinds.len() == 1
                && item.keys().all(|k| THEN_KINDS.contains(&k.as_str()) || k == "speaker")
                && item.values().all(|v| v.as_str().is_some_and(|s| !s.is_empty()));
            if !ok {
                errors.push(format!("campaign.pending: {} must hold exactly one of vignette, battle or dialogue.", Value::Object(item.clone())));
            }
        }
        errors
    }
}

// ---- read keys (11.4) ----

/// The value of read key `key` (`meter.<id>`, `meter.<id>.band`, `meter.<id>.running`, `campaign.act`,
/// `campaign.day`); None outside a campaign world or for an unknown meter. The `roster.` keys (`roster.living` among
/// them) are the roster's (`roster::read_key`, ROSTER1 RK1), answered in every world.
pub fn read_key(gs: &GameState, key: &str) -> Option<Value> {
    let c = gs.campaign.as_ref()?;
    match key {
        "campaign.act" => return Some(int(c.act)),
        // X2: day 1 at a new game
        "campaign.day" => return Some(int(gs.time + 1)),
        _ => {}
    }
    let id = rpgkit_data::meter_of_read_key(key)?;
    let (def, st) = (c.defs.get(id)?, c.meters.get(id)?);
    let value = st.value(def, gs.time);
    Some(match key.strip_prefix("meter.").and_then(|r| r.split_once('.')).map(|(_, s)| s) {
        None => int(value),
        Some("band") => Value::String(def.band(value).map(|b| b.id.clone()).unwrap_or_default()),
        Some(_) => Value::Bool(def.countdown && st.running),
    })
}

// ---- the tick (X4) ----

/// What a tick runs after (X4).
#[derive(Debug, Clone, PartialEq)]
pub enum Cause {
    /// `n` days passed (K2); `why` is travel (once per leg), rest or retreat. Day inputs run `n` times.
    Days { n: i64, why: String },
    /// One travel leg on a route carrying `route_tags`, in travel mode `mode` ("" = none; CAMPAIGN1-TM) (leg inputs).
    Leg { route_tags: Vec<String>, mode: String },
    /// A settled battle: the encounter's qid and tags, the outcome (won, lost, fled).
    Battle { encounter: String, tags: Vec<String>, outcome: String },
    /// The party entered the location with this qid (`explore::enter`; enter triggers, T1).
    Enter { location: String },
    /// A dialogue choice, a trigger or a rule applied effects.
    Effects,
    /// The act changed to `n` (act triggers, T1): after an effect list that set it, and after a tick whose rules,
    /// checkpoints or triggers set it.
    Act(i64),
}

/// A `meter_warning` event (W1).
fn warning(def: &MeterDef, rule: &meter::Rule, value: i64) -> Value {
    let band = def.band(value);
    Value::Object(json::obj([
        ("event", "meter_warning".into()),
        ("meter", def.id.clone().into()),
        ("rule", rule.id.clone().into()),
        ("text", rule.warn.clone().into()),
        ("band", band.map(|b| b.id.clone()).unwrap_or_default().into()),
        ("tone", band.map(|b| b.tone.clone()).unwrap_or_default().into()),
    ]))
}

/// X4: runs the campaign after `cause`: inputs, meter rules (gauges, then countdowns), checkpoints (C1-C3), triggers
/// (T1, T2), then the run-over check. Returns the events for the action's result (W1: `meter_warning`,
/// `checkpoint_warning`, `checkpoint_missed`, `trigger_fired`, `run_over`, `act_start`); empty and untouched outside a
/// campaign world, and once the run is over. Follow-up ticks run in the same call (`tick_all`).
pub fn tick(d: &Data, gs: &mut GameState, cause: &Cause) -> Vec<Value> {
    tick_all(d, gs, vec![cause.clone()])
}

/// The ticks after `causes`, in order, with their follow-ups: days a `rest` effect owed run first as their own
/// days(n, rest) tick (a rest in a dialogue choice before its tick; a rest in the effects of a rule, a missed `go_on`
/// checkpoint or a trigger right after the tick that ran it), and an act changed inside a tick queues cause act(n)
/// after the causes already waiting. At most REENTRIES_MAX follow-ups; owed days left over wait for the next cause.
fn tick_all(d: &Data, gs: &mut GameState, causes: Vec<Cause>) -> Vec<Value> {
    let mut events = Vec::new();
    if gs.campaign.is_none() {
        return events;
    }
    let mut queue: VecDeque<Cause> = causes.into();
    let mut follow_ups = 0;
    loop {
        let owed = gs.campaign.as_ref().map_or(0, |c| c.owed_days);
        let next = if owed > 0 && follow_ups < REENTRIES_MAX {
            follow_ups += 1;
            if let Some(c) = gs.campaign.as_mut() {
                c.owed_days = 0;
            }
            Cause::Days { n: owed, why: "rest".into() }
        } else if let Some(next) = queue.pop_front() {
            next
        } else {
            break;
        };
        let act = gs.campaign.as_ref().map(|c| c.act);
        events.extend(tick_once(d, gs, &next));
        let Some(c) = gs.campaign.as_ref() else { break };
        if c.is_over() {
            break;
        }
        if act != Some(c.act) && follow_ups < REENTRIES_MAX {
            follow_ups += 1;
            queue.push_back(Cause::Act(c.act));
        }
    }
    events
}

fn tick_once(d: &Data, gs: &mut GameState, cause: &Cause) -> Vec<Value> {
    let mut events = Vec::new();
    let Some(c) = gs.campaign.as_ref() else { return events };
    let told = c.over.as_ref().is_some_and(|o| o.told);
    if told {
        return events;
    }
    let before = c.last.clone();
    let before_time = c.last_time;
    let last_day = before_time + 1;
    if c.over.is_none() {
        checkpoint::step_before(d, gs, last_day);
        inputs(gs, cause);
        rule_passes(d, gs, &mut events);
        // C1-C3 at today's day (X2); a missed checkpoint's effects may move meters: the rules run again
        if checkpoint::step(d, gs, gs.time + 1, last_day, &mut events) {
            rule_passes(d, gs, &mut events);
        }
        // T1, T2; a trigger's effects may move meters: the rules run again
        if trigger::step(d, gs, cause, &mut events) {
            rule_passes(d, gs, &mut events);
        }
    }
    let time = gs.time;
    let Some(c) = gs.campaign.as_mut() else { return events };
    if let Some(o) = c.over.as_mut().filter(|o| !o.told) {
        o.told = true;
        events.push(Value::Object(json::obj([("event", "run_over".into()), ("kind", o.kind.clone().into()), ("card", o.card.clone().into()), ("text", o.text.clone().into())])));
    }
    c.sync(time);
    // days a `rest` effect passed inside this tick are not ticked yet: the follow-up days(n, rest) tick sees them pass
    // (its checkpoint step warns and misses across them; the rule passes after the effect already saw the meters move)
    c.last_time = time - c.owed_days;
    // read keys moved (a meter, or the day): quest flag steps on them advance as after any flag change
    if (c.last != before || time != before_time) && !c.is_over() {
        quest::refresh(d, gs);
    }
    events
}

/// M3: the cause's inputs move the gauges (clamped, M4).
fn inputs(gs: &mut GameState, cause: &Cause) {
    let Some(c) = gs.campaign.as_mut() else { return };
    let defs = c.defs.clone();
    for (id, def) in defs.iter() {
        let delta = match cause {
            Cause::Battle { tags, outcome, .. } => def.input_delta("battle", c.act, |x| tags_match(&x.tags, tags) && (x.outcomes.is_empty() || x.outcomes.contains(outcome))),
            Cause::Days { n, .. } => def.input_delta("day", c.act, |_| true) * n,
            Cause::Leg { route_tags, mode } => def.input_delta("leg", c.act, |x| tags_match(&x.route_tags, route_tags) && (x.modes.is_empty() || x.modes.contains(mode))),
            _ => 0,
        };
        if delta != 0 {
            if let Some(st) = c.meters.get_mut(id) {
                st.value = def.clamp(st.value + delta, c.act);
            }
        }
    }
}

/// M5, X4: fires every rule a meter's move since the last tick crossed, gauges first, then countdowns, re-entering
/// while effects move meters (the first pass plus at most REENTRIES_MAX more); stops at the first `end` (F1).
fn rule_passes(d: &Data, gs: &mut GameState, events: &mut Vec<Value>) {
    let Some(defs) = gs.campaign.as_ref().map(|c| c.defs.clone()) else { return };
    let order: Vec<&MeterDef> = defs.values().filter(|m| !m.countdown).chain(defs.values().filter(|m| m.countdown)).collect();
    for _ in 0..=REENTRIES_MAX {
        let mut moved = false;
        for def in &order {
            let time = gs.time;
            let Some(c) = gs.campaign.as_mut() else { return };
            if c.is_over() {
                return;
            }
            let now = c.value(&def.id, time);
            let was = c.last.get(&def.id).copied().unwrap_or(now);
            if was == now {
                continue;
            }
            c.last.insert(def.id.clone(), now);
            moved = true;
            let act = c.act;
            for rule in def.crossed(was, now, act) {
                let key = format!("rule:{}/{}", def.id, rule.id);
                let Some(c) = gs.campaign.as_mut() else { return };
                if c.is_over() {
                    return;
                }
                if rule.once {
                    if c.fired.contains(&key) {
                        continue;
                    }
                    c.fired.push(key);
                    c.fired.sort();
                }
                fire(d, gs, def, rule, now, events);
            }
        }
        if !moved {
            break;
        }
    }
}

/// M5: one rule fires: its warning, its effects at once, then (unless the run ended) its `then` items queued and a
/// countdown's restart.
fn fire(d: &Data, gs: &mut GameState, def: &MeterDef, rule: &meter::Rule, value: i64, events: &mut Vec<Value>) {
    if !rule.warn.is_empty() {
        events.push(warning(def, rule, value));
    }
    apply_effects(d, gs, &rule.effects, events);
    let time = gs.time;
    let Some(c) = gs.campaign.as_mut() else { return };
    if c.is_over() {
        return;
    }
    c.pending.extend(rule.then.iter().cloned());
    if let (true, Some(days)) = (def.countdown, rule.restart_days) {
        if let Some(st) = c.meters.get_mut(&def.id) {
            st.due = time + days;
            st.running = true;
        }
    }
}

// ---- effects (11.3) ----

/// Whether `effect` is a campaign effect kind (11.3).
pub fn is_campaign_effect(effect: &str) -> bool {
    rpgkit_data::CAMPAIGN_EFFECTS.iter().any(|(k, _)| *k == effect)
}

/// The meter id effect `fx` names (its `meter` resolved through the lineage), when the campaign has it.
fn meter_id(d: &Data, gs: &GameState, fx: &Dict) -> Option<String> {
    let r = json::s(fx, "meter");
    let id = json::s(d.resolve(&gs.world_id, "meters", &r), "id");
    let id = if id.is_empty() { r } else { id };
    gs.campaign.as_ref().filter(|c| c.meters.contains_key(&id)).map(|_| id)
}

/// Applies one campaign effect (11.3) in a campaign world; outside one, or after the run ended, it does nothing.
/// Returns false for an effect kind that is not a campaign effect. `set_act` adds an `act_start` event (F5: the GUI
/// writes the act-start save `save` when it is not ""); `rest` rests without a note (the dialogue's effect body calls
/// `purse::rest_effect` itself for the note).
pub fn apply_effect(d: &Data, gs: &mut GameState, fx: &Dict, events: &mut Vec<Value>) -> bool {
    let effect = json::s(fx, "effect");
    let id = meter_id(d, gs, fx);
    if effect == "rest" {
        purse::rest_effect(d, gs);
        return true;
    }
    let time = gs.time;
    let Some(c) = gs.campaign.as_mut() else { return is_campaign_effect(&effect) };
    if c.is_over() {
        return true;
    }
    if effect.starts_with("clock_") {
        if let Some(id) = id {
            clock::apply(c, &id, &effect, fx, time);
        }
        return is_campaign_effect(&effect);
    }
    let act = c.act;
    let defs = c.defs.clone();
    match effect.as_str() {
        "meter_add" | "meter_set" => {
            let Some(id) = id else { return true };
            let (Some(def), Some(st)) = (defs.get(&id), c.meters.get_mut(&id)) else { return true };
            if def.countdown {
                // days left move with a running clock
                if effect == "meter_add" && st.running {
                    let left = (st.due - gs.time).max(0);
                    st.due = gs.time + (left + json::i(fx, "delta", 0)).clamp(0, def.max);
                }
            } else {
                let to = if effect == "meter_add" { st.value + json::i(fx, "delta", 0) } else { json::i(fx, "value", st.value) };
                st.value = def.clamp(to, act);
            }
        }
        "meter_show" => {
            if let Some(st) = id.and_then(|id| c.meters.get_mut(&id)) {
                st.visible = json::b(fx, "visible", true);
            }
        }
        "set_act" => {
            let to = json::i(fx, "act", c.act).clamp(0, ACT_MAX);
            if to != c.act {
                c.act = to;
                // M4: the gauges rise to the new act's floor
                for (id, def) in defs.iter().filter(|(_, m)| !m.countdown) {
                    if let Some(st) = c.meters.get_mut(id) {
                        st.value = def.clamp(st.value, to);
                    }
                }
                let save = if manifest(d, &gs.world_id).act_saves { act_save_key(to, &c.run) } else { String::new() };
                events.push(Value::Object(json::obj([("event", "act_start".into()), ("act", int(to)), ("save", save.into())])));
            }
        }
        "end" => {
            if c.over.is_none() {
                c.over = Some(RunOver { kind: json::s_or(fx, "kind", "fail"), card: json::s(fx, "card"), text: json::s(fx, "text"), told: false });
            }
        }
        _ => return false,
    }
    // PK2-S5a: the act line: from the manifest's `durability.from_act` every owned weapon becomes an instance (before
    // the GUI writes the act-start save)
    if effect == "set_act" {
        crate::weapons::sync(d, gs);
    }
    true
}

/// The effects of a meter rule, a missed `go_on` checkpoint or a trigger (T2: the dialogue effects that need no screen,
/// plus 11.3), in order, through the one shared effect body (`dialogue::apply_effects`, which stops at the first `end`,
/// F1); returns the notes (`give`, `recruit`, `rest`, a refused sequence). No tick: the caller's tick goes on.
pub fn apply_effects(d: &Data, gs: &mut GameState, effects: &[Dict], events: &mut Vec<Value>) -> Vec<String> {
    let mut notes = Vec::new();
    crate::dialogue::apply_effects(d, gs, effects, &mut notes, events);
    notes
}

/// The tick after an effect list (cause `effects`), then cause `act` when the list changed the act (`act_before`:
/// the act before the list; None outside a campaign world); act changes inside the tick follow as `tick_all` says.
pub fn after_effects(d: &Data, gs: &mut GameState, act_before: Option<i64>) -> Vec<Value> {
    let mut causes = vec![Cause::Effects];
    if let (Some(was), Some(now)) = (act_before, gs.campaign.as_ref().map(|c| c.act)) {
        if was != now {
            causes.push(Cause::Act(now));
        }
    }
    tick_all(d, gs, causes)
}

/// Appends `events` to the result's `campaign` list (the key only when there are some, X3).
pub fn merge_events(out: &mut Dict, events: Vec<Value>) {
    if events.is_empty() {
        return;
    }
    let mut all = json::aget(out, "campaign").to_vec();
    all.extend(events);
    out.insert("campaign".into(), Value::Array(all));
}

// ---- run over and the queue ----

/// F3: the refusal of a save while the run is over ("" = may save).
pub fn save_refusal(gs: &GameState) -> &'static str {
    if gs.campaign.as_ref().is_some_and(Campaign::is_over) {
        RUN_OVER_TEXT
    } else {
        ""
    }
}

/// T2: the head of the queue of `then` items (the GUI plays it when a map screen is on top).
pub fn pending_head(gs: &GameState) -> Option<&Dict> {
    gs.campaign.as_ref().and_then(|c| c.pending.first())
}

/// T2: removes and returns the head of the queue (the GUI calls it when it starts playing the item; a `{vignette}`
/// item goes to the story layer's vignette player, STORYUI1-W).
pub fn pop_pending(gs: &mut GameState) -> Option<Dict> {
    let c = gs.campaign.as_mut()?;
    (!c.pending.is_empty()).then(|| c.pending.remove(0))
}
