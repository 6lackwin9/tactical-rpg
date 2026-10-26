//! Mission board (STORYUI1 5.5; kit hook, engine-only, no Godot original): a keeper's board lists the quests whose
//! `board` names it, by section (main, jobs, under way, locked, closed), with each quest's node; the conversation opens
//! it (`open_board`, the runner's `board_qid`), Accept starts a quest there, the first open sets `board.<id>.opened`,
//! and the world map shows the jobs of opened boards on their nodes (B10). A world without boards never sees any of it.

use crate::data::Data;
use crate::state::GameState;
use crate::{dialogue, explore, quest};
use aegis_core::json::{self, Dict};
use serde_json::Value;

/// B5: the sections in order (id, title).
pub const SECTIONS: [(&str, &str); 5] = [("main", "Main"), ("jobs", "Jobs"), ("under_way", "Under way"), ("locked", "Locked"), ("closed", "Closed")];
/// B3: the busy rule of a conversation while the board pane is open.
pub const BUSY_TEXT: &str = "Close the board first.";
pub const CLOSED_TEXT: &str = "The board is closed.";
/// B4: the hover text of the keeper's board badge.
pub const BADGE_TEXT: &str = "New work on the board";

/// The board `r` names (from the party's world), {} when unknown.
pub fn board_of<'a>(d: &'a Data, gs: &GameState, r: &str) -> &'a Dict {
    d.resolve(&gs.world_id, "boards", r)
}

/// The boards of the party's world lineage, in merged file order.
pub fn boards<'a>(d: &'a Data, gs: &GameState) -> Vec<&'a Dict> {
    d.merged_entries(&gs.world_id, "boards")
}

/// The keeper's qid of board `b` ("" when unknown).
pub fn keeper_qid(d: &Data, gs: &GameState, b: &Dict) -> String {
    dialogue::character_qid(d, gs, &json::s(b, "keeper"))
}

/// B1: a board is open when its keeper exists and is not gone and its `requires_flags` hold.
pub fn is_open(d: &Data, gs: &GameState, r: &str) -> bool {
    let b = board_of(d, gs, r);
    let keeper = keeper_qid(d, gs, b);
    !b.is_empty() && !keeper.is_empty() && !dialogue::is_gone(d, gs, &keeper) && quest::all_hold(gs, json::get(b, "requires_flags"))
}

/// The qid of the board quest `q` (an entry or a resolved quest) is listed on ("" for a quest without a board).
pub fn board_qid_of(d: &Data, gs: &GameState, q: &Dict) -> String {
    let r = json::s(q, "board");
    if r.is_empty() {
        return String::new();
    }
    json::s(d.resolve(&json::s_or(q, "_world", &gs.world_id), "boards", &r), "_qid")
}

/// Whether quest `q` (an entry or a resolved quest) is a board quest (B3: offered only on its board).
pub fn on_a_board(q: &Dict) -> bool {
    !json::s(q, "board").is_empty()
}

/// B2: the qids of the quests on board `board_qid`, in merged file order.
pub fn quests(d: &Data, gs: &GameState, board_qid: &str) -> Vec<String> {
    d.merged_entries(&gs.world_id, "quests").into_iter().filter(|q| !board_qid.is_empty() && board_qid_of(d, gs, q) == board_qid).map(|q| json::s(q, "_qid")).collect()
}

/// B9: board `r`'s first open sets its `opened` flag (true); later opens change nothing.
pub fn mark_opened(d: &Data, gs: &mut GameState, r: &str) {
    let id = json::s(board_of(d, gs, r), "id");
    if id.is_empty() {
        return;
    }
    let key = rpgkit_data::board_opened_flag(&id);
    if !gs.flag_true(&key) {
        quest::set_flag(d, gs, &key, Value::Bool(true));
    }
}

/// Whether board `b` was opened once (B9).
pub fn was_opened(gs: &GameState, b: &Dict) -> bool {
    gs.flag_true(&rpgkit_data::board_opened_flag(&json::s(b, "id")))
}

/// B6: the node of quest `qid`: its first step's location (a `reach_location` step's location; a `win_encounter`
/// step's location, else its encounter's), resolved as the world map's quest targets are; "" when none.
pub fn node_of(d: &Data, gs: &GameState, qid: &str) -> String {
    let q = quest::resolved(d, gs, qid);
    let kind = json::s(json::aget(&q, "objectives").first().map(json::dict).unwrap_or_else(|| json::empty()), "kind");
    let step = json::aget(&q, "steps").first().map(json::dict).unwrap_or_else(|| json::empty());
    let mut refs = vec![json::s(step, "location")];
    if kind == "win_encounter" {
        refs.push(json::s(d.resolve(&gs.world_id, "encounters", &json::s(step, "encounter")), "location"));
    } else if kind != "reach_location" {
        return String::new();
    }
    refs.into_iter().filter(|r| !r.is_empty()).map(|r| json::s(explore::resolve_location(d, gs, &r), "_qid")).find(|n| !n.is_empty()).unwrap_or_default()
}

/// The section quest `qid` sits in (B5), with its line; None when it is not listed (completed, invalid, locked without
/// `show_locked`, closed without a `closed_text`).
fn section_of(d: &Data, gs: &GameState, qid: &str, keeper_name: &str) -> Option<(&'static str, String)> {
    let q = quest::resolved(d, gs, qid);
    let main = json::s_or(&q, "track", "side") == "main";
    match quest::status(d, gs, qid).as_str() {
        "available" => Some((if main { "main" } else { "jobs" }, String::new())),
        "active" => Some(("under_way", json::s(&quest::current_objective(d, gs, qid), "text"))),
        "ready" => Some(("under_way", format!("Report to {keeper_name}"))),
        "locked" if json::b(&q, "show_locked", false) => Some(("locked", lock_reason(d, gs, qid))),
        "unavailable" | "blocked" if !json::s(&q, "closed_text").is_empty() => Some(("closed", json::s(&q, "closed_text"))),
        _ => None,
    }
}

/// B5: why a locked quest is locked, in the dialogue lock reasons' words (its `requires` as quest conditions, then its
/// `requires_flags`; the first that fails).
pub fn lock_reason(d: &Data, gs: &GameState, qid: &str) -> String {
    let e = d.resolve(&gs.world_id, "quests", qid);
    let home = quest::world_of(d, gs, e);
    let mut conds: Vec<Value> = json::sget(e, "requires")
        .into_iter()
        .map(|r| {
            let other = json::s_or(d.resolve(&home, "quests", &r), "_qid", &r);
            Value::Object(json::obj([("kind", "quest".into()), ("quest", other.into()), ("status", "completed".into())]))
        })
        .collect();
    conds.extend(json::aget(e, "requires_flags").iter().cloned());
    dialogue::lock_reason(d, gs, &json::obj([("requires", Value::Array(conds))]))
}

/// B5, B6: board `r` as the pane shows it: {board, name, keeper, keeper_name, intro, open, sections: [{id, title,
/// rows: [{quest, name, status, node, node_name, line}]}]} (empty sections left out); {} for an unknown board.
pub fn view(d: &Data, gs: &GameState, r: &str) -> Dict {
    let b = board_of(d, gs, r);
    if b.is_empty() {
        return Dict::new();
    }
    let qid = json::s(b, "_qid");
    let keeper = keeper_qid(d, gs, b);
    let keeper_name = json::s_or(d.resolve(&gs.world_id, "characters", &keeper), "name", &keeper);
    let mut rows: Vec<(&str, Value)> = Vec::new();
    for q in quests(d, gs, &qid) {
        let Some((section, line)) = section_of(d, gs, &q, &keeper_name) else { continue };
        let node = node_of(d, gs, &q);
        let node_name = if node.is_empty() { String::new() } else { json::s_or(explore::resolve_location(d, gs, &node), "name", &node) };
        rows.push((
            section,
            Value::Object(json::obj([
                ("quest", q.clone().into()),
                ("name", json::s_or(&quest::resolved(d, gs, &q), "name", &q).into()),
                ("status", quest::status(d, gs, &q).into()),
                ("node", node.into()),
                ("node_name", node_name.into()),
                ("line", line.into()),
            ])),
        ));
    }
    let sections: Vec<Value> = SECTIONS
        .iter()
        .filter_map(|(id, title)| {
            let list: Vec<Value> = rows.iter().filter(|(s, _)| s == id).map(|(_, row)| row.clone()).collect();
            (!list.is_empty()).then(|| Value::Object(json::obj([("id", (*id).into()), ("title", (*title).into()), ("rows", Value::Array(list))])))
        })
        .collect();
    json::obj([
        ("board", qid.clone().into()),
        ("name", json::s(b, "name").into()),
        ("keeper", keeper.into()),
        ("keeper_name", keeper_name.into()),
        ("intro", json::s(b, "intro").into()),
        ("open", Value::Bool(is_open(d, gs, &qid))),
        ("sections", Value::Array(sections)),
    ])
}

/// B7: a board quest's detail: {quest, name, status, description (variants applied), steps: [{text, state: done,
/// current or to_come}], node, node_name, rewards (the resolved quest's), loot (the loot qids offered now),
/// loot_picks, reputation}; the game formats the rewards.
pub fn detail(d: &Data, gs: &GameState, qid: &str) -> Dict {
    let q = quest::resolved(d, gs, qid);
    if q.is_empty() {
        return Dict::new();
    }
    let status = quest::status(d, gs, qid);
    let count = json::aget(&q, "objectives").len();
    let at = match status.as_str() {
        "active" => json::i(&quest::current_objective(d, gs, qid), "index", 0) as usize,
        "ready" | "completed" => count,
        _ => 0,
    };
    let started = matches!(status.as_str(), "active" | "ready" | "completed");
    let steps: Vec<Value> = json::aget(&q, "steps")
        .iter()
        .take(count)
        .enumerate()
        .map(|(i, st)| {
            let state = if started && i < at {
                "done"
            } else if status == "active" && i == at {
                "current"
            } else {
                "to_come"
            };
            Value::Object(json::obj([("text", json::s(json::dict(st), "text").into()), ("state", state.into())]))
        })
        .collect();
    let node = node_of(d, gs, qid);
    let node_name = if node.is_empty() { String::new() } else { json::s_or(explore::resolve_location(d, gs, &node), "name", &node) };
    json::obj([
        ("quest", json::s(&q, "_qid").into()),
        ("name", json::s(&q, "name").into()),
        ("status", status.into()),
        ("description", json::s(&q, "description").into()),
        ("steps", Value::Array(steps)),
        ("node", node.into()),
        ("node_name", node_name.into()),
        ("rewards", Value::Object(json::dget(&q, "rewards").clone())),
        ("loot", json::strs(&quest::offers(d, gs, qid))),
        ("loot_picks", json::int(json::i(&q, "loot_picks", 0))),
        ("reputation", q.get("reputation").cloned().unwrap_or(Value::Object(Dict::new()))),
    ])
}

/// B4: whether the character `character_qid` keeps an open board with at least one available quest.
pub fn has_work(d: &Data, gs: &GameState, character_qid: &str) -> bool {
    !character_qid.is_empty()
        && boards(d, gs).into_iter().any(|b| {
            let qid = json::s(b, "_qid");
            keeper_qid(d, gs, b) == character_qid && is_open(d, gs, &qid) && quests(d, gs, &qid).iter().any(|q| quest::status(d, gs, q) == "available")
        })
}

/// B10: (node qid, quest name) of every available quest on a board opened once, whose node resolves, in board then
/// quest order.
pub fn jobs(d: &Data, gs: &GameState) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for b in boards(d, gs).into_iter().filter(|b| was_opened(gs, b)) {
        for q in quests(d, gs, &json::s(b, "_qid")) {
            if quest::status(d, gs, &q) != "available" {
                continue;
            }
            let node = node_of(d, gs, &q);
            if !node.is_empty() {
                out.push((node, json::s_or(&quest::resolved(d, gs, &q), "name", &q)));
            }
        }
    }
    out
}
