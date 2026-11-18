//! Campaign endings (PR6-11, `docs/sessions/PK6_PK7_BOSSES_ENDINGS.md` section 4; engine-only, no Godot original;
//! story-free): the ordered branch evaluator of the world kind `endings` (`pick`: which row an
//! `{effect: "end", kind: "ending"}` finishes on) and the card evaluator of the epilogue (`epilogue`: the row's
//! paragraphs, the outcome lines whose flags hold or not, and one fate card per member, those still in the party first,
//! then the departed). The rows and their keys are `rpgkit_data::ending`'s; the run-over state records the ending
//! (`RunOver::ending`, runtime only: a save never holds it).

use crate::data::Data;
use crate::progression;
use crate::quest;
use crate::state::GameState;
use aegis_core::json::{self, Dict};
use rpgkit_data::ending::{EndingRow, ENDINGS_KIND};
use serde_json::Value;

/// The ending rows of world `w` (merged along the lineage), in row order.
pub fn rows(d: &Data, w: &str) -> Vec<EndingRow> {
    d.merged_entries(w, ENDINGS_KIND).into_iter().map(EndingRow::from_obj).filter(|r| !r.id.is_empty()).collect()
}

/// Whether ending `row`'s gates all hold now: its flag conditions and its meter conditions (`meter.<id>` read keys, the
/// meter named by a ref resolved through the lineage).
pub fn holds(d: &Data, gs: &GameState, row: &EndingRow) -> bool {
    if !quest::all_hold(gs, &row.requires_flags) {
        return false;
    }
    row.requires_meter.iter().all(|c| {
        let r = json::s(c, "id");
        let id = json::s(d.resolve(&gs.world_id, "meters", &r), "id");
        let id = if id.is_empty() { r } else { id };
        let cond = json::obj([("key", Value::String(format!("meter.{id}"))), ("op", Value::String(json::s_or(c, "op", ">="))), ("value", c.get("value").cloned().unwrap_or(Value::from(0)))]);
        quest::condition_met(gs, &cond)
    })
}

/// The ordered branch: the ending an `end` of kind `ending` finishes on: row `named` (a ref) while its gates hold,
/// else the first row in order whose gates hold; "" when none holds (the end's own card and text stand alone).
pub fn pick(d: &Data, gs: &GameState, named: &str) -> String {
    let all = rows(d, &gs.world_id);
    if !named.is_empty() {
        let id = json::s(d.resolve(&gs.world_id, ENDINGS_KIND, named), "id");
        if let Some(r) = all.iter().find(|r| r.id == id).filter(|r| holds(d, gs, r)) {
            return r.id.clone();
        }
    }
    all.iter().find(|r| holds(d, gs, r)).map(|r| r.id.clone()).unwrap_or_default()
}

/// One member's fate card on the epilogue.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FateCard {
    pub member: String,
    pub name: String,
    /// The member left the party before the end (`party.departed`).
    pub departed: bool,
    /// The departure's cause ("" for a member still in the party).
    pub cause: String,
    /// The member's stage on the template's track (0 = none) and the stage's name ("" = none).
    pub stage: i64,
    pub stage_name: String,
    /// The keepsake a departed member left (its display name; "" = none).
    pub keepsake: String,
    /// The template's line with its placeholders filled ("" = the template has none for this member).
    pub text: String,
}

/// The epilogue of an ending: the row's title, tone, art and paragraphs, its outcome lines and the fate cards.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Epilogue {
    pub id: String,
    pub title: String,
    pub tone: String,
    pub art: String,
    pub paragraphs: Vec<String>,
    pub outcomes: Vec<String>,
    pub fates: Vec<FateCard>,
}

/// A flag value as text (`{value}`): a string as is, a number or bool as written, else "".
fn value_text(v: Option<Value>) -> String {
    match v {
        Some(Value::String(s)) => s,
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

/// The card evaluator's outcome lines: each `world_outcomes` line's `text_if_true` when its flag condition holds, else
/// its `text_if_false` (`{value}` = the flag's value), skipping an empty text.
pub fn outcome_lines(gs: &GameState, row: &EndingRow) -> Vec<String> {
    row.outcomes
        .iter()
        .filter_map(|o| {
            let key = json::s(o, "flag");
            let mut cond = json::obj([("key", Value::String(key.clone()))]);
            for k in ["op", "value"] {
                if let Some(v) = o.get(k) {
                    cond.insert(k.into(), v.clone());
                }
            }
            let text = json::s(o, if quest::condition_met(gs, &cond) { "text_if_true" } else { "text_if_false" });
            (!text.trim().is_empty()).then(|| text.replace("{value}", &value_text(gs.read_flag(&key))))
        })
        .collect()
}

/// The card evaluator's fate cards: one per party member (in party order), then one per departed member (in departure
/// order), the template's line filled in.
pub fn fate_cards(d: &Data, gs: &GameState, row: &EndingRow) -> Vec<FateCard> {
    let t = &row.fates;
    let track = json::s(t, "track");
    let def = d.cat.unit_tracks.get(&track);
    let stage_of = |m: &crate::state::UnitState| -> (i64, String) {
        if track.is_empty() {
            return (0, String::new());
        }
        let n = m.track_stage(&track);
        let name = def.filter(|_| n > 0).and_then(|r| r.stages.get(n as usize - 1)).map(|s| s.name.clone()).unwrap_or_default();
        (n, name)
    };
    let fill = |line: &str, name: &str, stage: (i64, &str), cause: &str| -> String {
        let stage_text = if !stage.1.is_empty() { stage.1.to_string() } else if stage.0 != 0 { stage.0.to_string() } else { String::new() };
        line.replace("{name}", name).replace("{stage}", &stage_text).replace("{cause}", cause)
    };
    let mut out = Vec::new();
    for m in &gs.party.members {
        let name = progression::display_name(d, m);
        let (stage, stage_name) = stage_of(m);
        let text = fill(&json::s(t, "member"), &name, (stage, &stage_name), "");
        out.push(FateCard { member: m.id.clone(), name, departed: false, cause: String::new(), stage, stage_name, keepsake: String::new(), text });
    }
    for dep in &gs.party.departed {
        let m = &dep.member;
        let name = progression::display_name(d, m);
        let (stage, stage_name) = stage_of(m);
        let by_cause = json::s(json::dget(t, "causes"), &dep.cause);
        let line = if by_cause.is_empty() { json::s(t, "departed") } else { by_cause };
        let text = fill(&line, &name, (stage, &stage_name), &dep.cause);
        let keepsake = if dep.keepsake.is_empty() {
            String::new()
        } else {
            gs.party.keepsakes.iter().find(|k| k.id == dep.keepsake).map(|k| k.name.clone()).unwrap_or_else(|| d.item_name(&dep.keepsake))
        };
        out.push(FateCard { member: m.id.clone(), name, departed: true, cause: dep.cause.clone(), stage, stage_name, keepsake, text });
    }
    out
}

/// The epilogue of ending `id` (a ref) in the game's world; None for an unknown ending.
pub fn epilogue(d: &Data, gs: &GameState, id: &str) -> Option<Epilogue> {
    if id.is_empty() {
        return None;
    }
    let raw: &Dict = d.resolve(&gs.world_id, ENDINGS_KIND, id);
    if raw.is_empty() {
        return None;
    }
    let row = EndingRow::from_obj(raw);
    Some(Epilogue {
        id: row.id.clone(),
        title: row.title.clone(),
        tone: if row.tone.is_empty() { "calm".into() } else { row.tone.clone() },
        art: row.art.clone(),
        paragraphs: row.paragraphs.clone(),
        outcomes: outcome_lines(gs, &row),
        fates: fate_cards(d, gs, &row),
    })
}
