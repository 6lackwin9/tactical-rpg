//! Campaign endings (PR6-11, `docs/sessions/PK6_PK7_BOSSES_ENDINGS.md` section 4; engine-only, no Godot original;
//! story-free): the world kind `endings` and the `end` effect's kind `ending`. An ending row
//! `{id, title, tone, art, requires_flags, requires_meter, epilogue_paragraphs, world_outcomes, member_fates_template}`
//! is one way the campaign can finish: its gates (flag conditions and meter conditions, all holding) say when it may
//! be reached, its paragraphs, outcome lines and member fate cards make the epilogue. The rows are ordered: an
//! `{effect: "end", kind: "ending"}` without an `ending` id finishes on the first row whose gates hold; one naming a
//! row finishes there while its gates hold, else as without a name. The last row is the fallback (no gates). The kind
//! ships without rows until the asset lane fills it.

use aegis_core::json::{self, Dict};
use serde_json::Value;

/// The world kind (file `worlds/<id>/endings.json`; a missing file is an empty table).
pub const ENDINGS_KIND: &str = "endings";
/// The `end` effect kind that finishes the campaign (`END_KINDS`).
pub const END_KIND_ENDING: &str = "ending";
/// The `end` effect key naming an ending row (kind `ending` only; "" = the first row whose gates hold).
pub const END_ENDING_KEY: &str = "ending";
/// A title's and a text's (a paragraph, an outcome line, a fate template) longest length.
pub const TITLE_MAX: usize = 48;
pub const PARAGRAPH_MAX: usize = 600;
pub const LINE_MAX: usize = 200;
/// The most paragraphs, outcome lines and meter conditions an ending holds (at least one paragraph).
pub const PARAGRAPHS_MAX: usize = 6;
pub const OUTCOMES_MAX: usize = 8;
pub const METER_CONDS_MAX: usize = 4;
/// The keys of a `requires_meter` condition (`op` one of `METER_OPS`, `value` a whole number).
pub const METER_COND_KEYS: [&str; 3] = ["id", "op", "value"];
pub const METER_OPS: [&str; 6] = ["==", "!=", ">=", "<=", ">", "<"];
/// The keys of a `world_outcomes` line: a flag condition (`flag`, optional `op` and `value`, as a flag condition's
/// `key`, `op`, `value`; default: the flag is true) and the line shown when it holds or not ("" = no line). `{value}` in
/// a text is the flag's value.
pub const OUTCOME_KEYS: [&str; 5] = ["flag", "op", "value", "text_if_true", "text_if_false"];
/// The keys of `member_fates_template`: `track` (a unit track whose stage each card shows, optional), `member` (the
/// line of a member still in the party), `departed` (the line of a departed member) and `causes` ({cause: line}, a
/// departed member's line by its departure cause, optional). Placeholders: `{name}`, `{stage}` (the track stage's name,
/// else its number), `{cause}`.
pub const FATE_KEYS: [&str; 4] = ["track", "member", "departed", "causes"];
pub const FATE_PLACEHOLDERS: [&str; 3] = ["{name}", "{stage}", "{cause}"];

/// One ending row, read leniently (the content checks report bad rows).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EndingRow {
    pub id: String,
    pub title: String,
    /// One of `METER_TONES` (the epilogue's accent; "" reads `calm`).
    pub tone: String,
    /// A vignette played before the epilogue ("" = none).
    pub art: String,
    pub requires_flags: Value,
    pub requires_meter: Vec<Dict>,
    pub paragraphs: Vec<String>,
    pub outcomes: Vec<Dict>,
    pub fates: Dict,
}

impl EndingRow {
    pub fn from_obj(o: &Dict) -> Self {
        Self {
            id: json::s(o, "id"),
            title: json::s(o, "title"),
            tone: json::s(o, "tone"),
            art: json::s(o, "art"),
            requires_flags: o.get("requires_flags").cloned().unwrap_or(Value::Array(Vec::new())),
            requires_meter: json::aget(o, "requires_meter").iter().filter_map(Value::as_object).cloned().collect(),
            paragraphs: json::aget(o, "epilogue_paragraphs").iter().filter_map(Value::as_str).map(str::to_string).collect(),
            outcomes: json::aget(o, "world_outcomes").iter().filter_map(Value::as_object).cloned().collect(),
            fates: json::dget(o, "member_fates_template").clone(),
        }
    }

    /// Whether the row has no gates (the fallback).
    pub fn ungated(&self) -> bool {
        json::arr(&self.requires_flags).is_empty() && self.requires_meter.is_empty()
    }
}
