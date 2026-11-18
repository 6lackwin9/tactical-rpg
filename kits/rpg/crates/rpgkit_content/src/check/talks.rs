//! Talk rules of the world checks (BUNKS1 BK5, spec BUNKS1 section 2; engine-only, no Godot
//! original; the refs (pair characters, temperament and origin passives, dialogue, reward item) are field-table
//! checks): a `talks` row is either a written pair (two different characters, no template pick) or a template (no
//! pair, picked by a temperament or an origin), with flag conditions, a pair counter minimum and a dialogue.

use super::campaign::int_key;
use super::quest::condition_errors;
use super::View;
use aegis_core::json::{self, Dict};
use rpgkit_data::talk::{MIN_TOGETHER_MAX, TALKS_KIND, TOKEN_A, TOKEN_B};

const L: &str = "Talks";

/// The problems of a `talks` row in world `w` beyond its field table.
pub fn talk_errors(_v: &View, _w: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let row = rpgkit_data::talk::TalkRow::from_obj(e);
    if row.template {
        if !row.pair.is_empty() {
            errors.push(format!("{L}: a template talk names no pair (it pairs a recruit with a founder)."));
        }
        if row.temperament.is_empty() && row.origin.is_empty() {
            errors.push(format!("{L}: a template talk needs a temperament or an origin to be picked by."));
        }
    } else {
        // two different characters: a name listed twice is the field table's error
        if row.pair.len() != 2 {
            errors.push(format!("{L}: a written talk names a pair of two characters (or set template)."));
        }
        if !row.temperament.is_empty() || !row.origin.is_empty() {
            errors.push(format!("{L}: temperament and origin pick template talks only."));
        }
    }
    if let Some(c) = e.get("requires_flags") {
        errors.extend(condition_errors(c, &format!("{L} requires flags")));
    }
    errors.extend(int_key(e, "min_together", 0, MIN_TOGETHER_MAX, L));
    if json::s(e, "dialogue").is_empty() {
        errors.push(format!("{L}: choose the dialogue."));
    }
    errors
}

/// BK7: whether dialogue `id` of world `w` is played by a template talk of any world (its texts may then name `{a}`
/// and `{b}`).
pub fn template_dialogue(v: &View, w: &str, id: &str) -> bool {
    let qid = format!("{w}:{id}");
    v.worlds.world_ids().iter().any(|tw| {
        v.worlds.own_entries(tw, TALKS_KIND).iter().any(|t| json::b(t, "template", false) && json::s(v.worlds.resolve(tw, "dialogues", &json::s(t, "dialogue")), "_qid") == qid)
    })
}

/// BK7: a dialogue text error about the `{a}` or `{b}` token (allowed in a template talk's dialogue).
pub fn name_token_error(e: &str) -> bool {
    [TOKEN_A, TOKEN_B].iter().any(|t| e.contains(&format!("Unknown token '{t}'")))
}
