//! Checkpoints (CAMPAIGN1 C1-C3; kit hook, engine-only, no Godot original): the world kind `checkpoints` read with its
//! defaults, the tick's checkpoint step (met, warnings, missed, the bad end), and the readers the mission board and the
//! world map show (`checkpoint` for STORYUI1 B6, B7 and B10; `view`).

use super::{apply_effects, Campaign, RunOver};
use crate::data::Data;
use crate::quest;
use crate::state::GameState;
use aegis_core::json::{self, int, Dict};
use serde_json::Value;

/// C2: the default warning days.
pub const WARN_DAYS: [i64; 3] = [7, 3, 1];

/// A checkpoint row (11.2) read with the schema's defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckpointDef {
    pub id: String,
    pub name: String,
    pub act: i64,
    /// The quest's qid ("" when the checkpoint uses `met_when`).
    pub quest: String,
    pub met_when: Vec<Value>,
    pub deadline: i64,
    /// Descending.
    pub warn_days: Vec<i64>,
    /// `end_run` or `go_on`.
    pub go_on: bool,
    pub card: String,
    pub text: String,
    pub effects: Vec<Dict>,
}

impl CheckpointDef {
    pub fn from_row(d: &Data, e: &Dict) -> Self {
        let home = json::s(e, "_world");
        let r = json::s(e, "quest");
        let quest = if r.is_empty() { String::new() } else { json::s_or(d.resolve(&home, "quests", &r), "_qid", &r) };
        let mut warn_days: Vec<i64> = match e.get("warn_days") {
            Some(v) => json::arr(v).iter().filter(|x| json::is_int(x)).map(json::to_int).collect(),
            None => WARN_DAYS.to_vec(),
        };
        warn_days.sort_by_key(|w| std::cmp::Reverse(*w));
        Self {
            id: json::s(e, "id"),
            name: json::s(e, "name"),
            act: json::i(e, "act", 1),
            quest,
            met_when: json::aget(e, "met_when").to_vec(),
            deadline: json::i(e, "deadline", 1),
            warn_days,
            go_on: json::s_or(e, "mode", "end_run") == "go_on",
            card: json::s(e, "card"),
            text: json::s(e, "text"),
            effects: json::aget(e, "effects").iter().filter_map(|x| x.as_object().cloned()).collect(),
        }
    }

    /// C1: the condition holds now: the quest's objectives are done (status `ready` or `completed`), or every
    /// `met_when` condition holds.
    pub fn holds(&self, d: &Data, gs: &GameState) -> bool {
        if self.quest.is_empty() {
            quest::all_hold(gs, &Value::Array(self.met_when.clone()))
        } else {
            matches!(quest::status(d, gs, &self.quest).as_str(), "ready" | "completed")
        }
    }
}

/// The checkpoint rows of world `w` (merged along the lineage), in data order.
pub fn defs(d: &Data, w: &str) -> Vec<CheckpointDef> {
    d.merged_entries(w, "checkpoints").into_iter().map(|e| CheckpointDef::from_row(d, e)).filter(|c| !c.id.is_empty()).collect()
}

/// The rows the tick looks at: those of the current act not yet met or missed.
fn open(d: &Data, gs: &GameState) -> Vec<CheckpointDef> {
    let Some(c) = gs.campaign.as_ref() else { return Vec::new() };
    defs(d, &gs.world_id).into_iter().filter(|cp| cp.act == c.act && !c.checkpoints.contains_key(&cp.id)).collect()
}

fn record(gs: &mut GameState, id: &str, state: &str) {
    if let Some(c) = gs.campaign.as_mut() {
        c.checkpoints.insert(id.to_string(), state.to_string());
    }
}

/// C1 at a tick's start: a condition that holds now was reached before the days of this tick passed (time moves only
/// right before a days tick), so it is met when the day of the last tick (`last_day`) was on or before the deadline.
pub fn step_before(d: &Data, gs: &mut GameState, last_day: i64) {
    for cp in open(d, gs) {
        if last_day <= cp.deadline && cp.holds(d, gs) {
            record(gs, &cp.id, "met");
        }
    }
}

/// C1-C3, the tick's checkpoint step at day `day` (the last tick's day `last_day`): met when the condition holds on or
/// before the deadline; else one `checkpoint_warning` per `warn_days` value the days left crossed since the last tick
/// (in order); missed once the day is past the deadline: `end_run` puts the run over with the checkpoint's card
/// (kind `bad_end`), `go_on` runs its effects (a `checkpoint_missed` event). Stops at the first `end` (F1). Returns
/// whether effects ran (the rules run again).
pub fn step(d: &Data, gs: &mut GameState, day: i64, last_day: i64, events: &mut Vec<Value>) -> bool {
    let mut ran = false;
    for cp in open(d, gs) {
        if gs.campaign.as_ref().is_none_or(Campaign::is_over) {
            break;
        }
        if day <= cp.deadline && cp.holds(d, gs) {
            record(gs, &cp.id, "met");
            continue;
        }
        let (before, now) = (cp.deadline - last_day, cp.deadline - day);
        if now >= 0 {
            for w in cp.warn_days.iter().filter(|w| before > **w && now <= **w) {
                events.push(Value::Object(json::obj([
                    ("event", "checkpoint_warning".into()),
                    ("checkpoint", cp.id.clone().into()),
                    ("name", cp.name.clone().into()),
                    ("quest", cp.quest.clone().into()),
                    ("day", int(cp.deadline)),
                    ("left", int(now)),
                    ("warn", int(*w)),
                ])));
            }
            continue;
        }
        record(gs, &cp.id, "missed");
        if cp.go_on {
            events.push(Value::Object(json::obj([("event", "checkpoint_missed".into()), ("checkpoint", cp.id.clone().into()), ("name", cp.name.clone().into()), ("quest", cp.quest.clone().into())])));
            apply_effects(d, gs, &cp.effects, events);
            ran = true;
        } else if let Some(c) = gs.campaign.as_mut() {
            c.over = Some(RunOver { kind: "bad_end".into(), card: cp.card.clone(), text: cp.text.clone(), told: false });
        }
    }
    ran
}

/// STORYUI1 B6, B7, B10 (CAMPAIGN1 W5): the deadline of the checkpoint on quest `quest` (a ref or a qid) in the current
/// act while it is neither met nor missed: (deadline day, days left, never below 0); None outside a campaign world.
pub fn checkpoint(d: &Data, gs: &GameState, quest: &str) -> Option<(i64, i64)> {
    gs.campaign.as_ref()?;
    let qid = json::s_or(d.resolve(&gs.world_id, "quests", quest), "_qid", quest);
    let day = gs.time + 1;
    open(d, gs).into_iter().find(|cp| !cp.quest.is_empty() && cp.quest == qid).map(|cp| (cp.deadline, (cp.deadline - day).max(0)))
}

/// W5: every checkpoint of the world with its state: [{id, name, act, quest, day, left, state: open, met or missed}]
/// ([] outside a campaign world).
pub fn view(d: &Data, gs: &GameState) -> Vec<Value> {
    let Some(c) = gs.campaign.as_ref() else { return Vec::new() };
    let day = gs.time + 1;
    defs(d, &gs.world_id)
        .into_iter()
        .map(|cp| {
            let state = c.checkpoints.get(&cp.id).cloned().unwrap_or_else(|| "open".into());
            Value::Object(json::obj([
                ("id", cp.id.clone().into()),
                ("name", cp.name.clone().into()),
                ("act", int(cp.act)),
                ("quest", cp.quest.clone().into()),
                ("day", int(cp.deadline)),
                ("left", int((cp.deadline - day).max(0))),
                ("state", state.into()),
            ]))
        })
        .collect()
}
