//! Bunk talks (BUNKS1 BK5-BK8, spec BUNKS1 section 2; engine-only, no Godot original;
//! story-free): the world kind `talks`. A talk row `{id, pair | template, requires_flags, min_together, temperament,
//! origin, dialogue, reward}` is a short conversation two members have in the hub. A `pair` talk names two characters
//! (written for them); a `template` talk is picked for a generated recruit by its `temperament` or `origin` and paired
//! with a founder member, its text filling `{a}` (the founder) and `{b}` (the recruit) from their names. `dialogue`
//! names a `dialogues` row of the world; `reward` an item given once (often a keepsake). The pair counter
//! (`together`, saved on the party) counts the won or withdrawn battles a pair fought side by side; `min_together`
//! reads it. A talk plays once (its seen flag `talk.<id>`; a template once per recruit, `talk.<id>.<member>`).

use aegis_core::json::{self, Dict};
use serde_json::Value;

/// The world kind (file `worlds/<id>/talks.json`; a missing file is an empty table).
pub const TALKS_KIND: &str = "talks";
/// The seen flag's prefix (`talk.<id>`, a template's `talk.<id>.<member>`).
pub const SEEN_PREFIX: &str = "talk.";
/// The highest `min_together` (battles fought side by side).
pub const MIN_TOGETHER_MAX: i64 = 999;
/// The text tokens of a template talk: the founder's name and the recruit's.
pub const TOKEN_A: &str = "a";
pub const TOKEN_B: &str = "b";
/// The pair counter's key separator (`"a|b"`, the two member ids sorted).
pub const PAIR_SEP: &str = "|";

/// One talk row, read leniently (the content checks report bad rows).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TalkRow {
    pub id: String,
    /// The two characters of a written talk (empty for a template).
    pub pair: Vec<String>,
    pub template: bool,
    pub requires_flags: Value,
    pub min_together: i64,
    /// A template's pick: the recruit's temperament passive, else its origin passive ("" = not this way).
    pub temperament: String,
    pub origin: String,
    pub dialogue: String,
    /// An item id given when the talk plays ("" = none).
    pub reward: String,
}

impl TalkRow {
    pub fn from_obj(o: &Dict) -> Self {
        Self {
            id: json::s(o, "id"),
            pair: json::aget(o, "pair").iter().filter_map(Value::as_str).map(str::to_string).collect(),
            template: json::b(o, "template", false),
            requires_flags: o.get("requires_flags").cloned().unwrap_or(Value::Array(Vec::new())),
            min_together: json::i(o, "min_together", 0),
            temperament: json::s(o, "temperament"),
            origin: json::s(o, "origin"),
            dialogue: json::s(o, "dialogue"),
            reward: json::s(o, "reward"),
        }
    }
}

/// BK6: the pair counter key of members `a` and `b` (`"a|b"`, sorted).
pub fn pair_key(a: &str, b: &str) -> String {
    if a <= b {
        format!("{a}{PAIR_SEP}{b}")
    } else {
        format!("{b}{PAIR_SEP}{a}")
    }
}

/// BK8: the seen flag of talk `id` (a template's carries the recruit's member id: once per recruit).
pub fn seen_key(id: &str, recruit: &str) -> String {
    if recruit.is_empty() {
        format!("{SEEN_PREFIX}{id}")
    } else {
        format!("{SEEN_PREFIX}{id}.{recruit}")
    }
}
