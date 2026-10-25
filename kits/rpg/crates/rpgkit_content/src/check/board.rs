//! The mission board checks (STORYUI1 4.4-4.5, kit hook; engine-only, no Godot original): the rows of the world kind
//! `boards` and the board fields of a world quest (`board`, `track`, `show_locked`, `closed_text`). Quests without a
//! board report as before.

use super::quest::condition_errors;
use super::View;
use aegis_core::json::{self, Dict};
use rpgkit_data::{BOARD_INTRO_MAX, CLOSED_TEXT_MAX};
use serde_json::Value;

/// The problems of a board row (4.4).
pub fn board_errors(e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    if json::s(e, "keeper").is_empty() {
        errors.push("Choose the board keeper.".into());
    }
    if json::s(e, "intro").chars().count() > BOARD_INTRO_MAX {
        errors.push(format!("Intro is longer than {BOARD_INTRO_MAX} characters."));
    }
    errors.extend(condition_errors(e.get("requires_flags").unwrap_or(&Value::Array(Vec::new())), "Open when"));
    errors
}

/// The problems of a quest's board fields (4.5): the Closed line's length; a board quest needs a template, and its
/// giver is empty or the board's keeper (the same character through the lineage). The `board` ref itself and the
/// `track` option are the schema's checks.
pub fn quest_board_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    if json::s(e, "closed_text").chars().count() > CLOSED_TEXT_MAX {
        errors.push(format!("Closed text is longer than {CLOSED_TEXT_MAX} characters."));
    }
    let board = json::s(e, "board");
    if board.is_empty() {
        return errors;
    }
    if json::s(e, "template").is_empty() {
        errors.push(format!("A board quest needs a template (board '{board}' never offers a lore-only quest)."));
    }
    let b = v.worlds.resolve(w, "boards", &board);
    let giver = json::s(e, "giver");
    if b.is_empty() || giver.is_empty() {
        return errors;
    }
    let keeper = json::s(b, "keeper");
    let keeper_qid = json::s(v.worlds.resolve(&json::s(b, "_world"), "characters", &keeper), "_qid");
    let giver_qid = json::s(v.worlds.resolve(w, "characters", &giver), "_qid");
    if keeper_qid.is_empty() || giver_qid != keeper_qid {
        errors.push(format!("Given by '{giver}' is not the keeper of board '{board}' ('{keeper}'): leave it empty or name the keeper."));
    }
    errors
}
