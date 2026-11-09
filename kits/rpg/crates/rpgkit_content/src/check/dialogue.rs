//! Dialogue rules of the world checks (DialogueRules): trees, nodes, choices, choice conditions and effects, service
//! lines, and the soft tree and gone-NPC warnings. CAMPAIGN1 11.3: every effect list (nodes, choices, and the meter
//! rules, checkpoints and triggers of `check::campaign`) knows the campaign effects; flag effects refuse reserved keys.
//! STORYUI1 4.7: every list knows the story effects `vignette`, `sequence` and `open_board`, and a sequence's effects
//! step is a list of its own (no effect that opens a screen or starts a sequence); the service line `board`.
//! PORTRAITFX1 7: a node's `expression` (shared with the `line` cards of `check::story`).
//! T1.2-S2 (FB-54b): the choice effect `roll` (`stat`, `dc`, `key`, optional `who`); T1.2-S3 (FB-54c): a `stat`
//! condition and a roll name a primary stat or a U1 standing rating (`STANDING_RATINGS` or a derived-stat row id).

use super::quest::{flag_key_ok, flag_value_ok, rep_value_error};
use super::world::{party_character_errors, ref_errors};
use super::View;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_data::{ROLL_EFFECT, STANDING_RATINGS, STATS};
use indexmap::IndexMap;
use serde_json::Value;

const MAX_NODES: usize = 64;
const MAX_CHOICES: usize = 6;
const MAX_EFFECTS: usize = 8;
const MAX_TEXT: usize = 1000;
/// PORTRAITFX1 7: `expression`, the speaker's face on the node.
const NODE_KEYS: [&str; 8] = ["id", "speaker", "expression", "text", "choices", "effects", "next", "end"];
const CHOICE_KEYS: [&str; 6] = ["text", "to", "requires", "unless", "effects", "show_locked"];
/// Condition kind -> its keys besides `kind`.
const CONDITION_KINDS: [(&str, &[&str]); 5] = [("flag", &["key", "value", "op"]), ("quest", &["quest", "status"]), ("items", &["item", "count"]), ("gold", &["min"]), ("stat", &["stat", "min", "threshold"])];
const QUEST_STATUSES: [&str; 7] = ["locked", "available", "unavailable", "active", "ready", "blocked", "completed"];
const GOLD_CONDITION_MAX: i64 = 9999999;
const STAT_MIN: i64 = 1;
const STAT_MAX: i64 = 30;
/// T1.2-S3: the bar of a `stat` condition naming a standing rating (ratings run past the primary stats' 30).
const RATING_BAR_MAX: i64 = 999;
/// T1.2-S2: a roll's `dc`.
const ROLL_DC_MIN: i64 = 1;
const ROLL_DC_MAX: i64 = 999;
/// Effect -> its keys besides `effect`.
const EFFECTS: [(&str, &[&str]); 10] = [
    ("set_flag", &["key", "value"]),
    ("add_flag", &["key", "delta", "min", "max"]),
    ("start_quest", &["quest"]),
    ("turn_in", &["quest"]),
    ("give", &["items", "gold"]),
    ("take", &["items", "gold"]),
    ("open_shop", &["shop"]),
    ("fight", &["encounter"]),
    ("recruit", &["character", "level"]),
    (ROLL_EFFECT, &rpgkit_data::ROLL_EFFECT_KEYS),
];
const NODE_EFFECTS: [&str; 3] = ["set_flag", "add_flag", "give"];
/// STORYUI1 4.7: the story effect a node takes besides NODE_EFFECTS (the choice order rules keep NODE_EFFECTS).
const NODE_STORY_EFFECTS: [&str; 1] = ["vignette"];
/// CAMPAIGN1 T2: the dialogue effects that open a screen (not in meter rules, checkpoints or triggers); STORYUI1's
/// `open_board` is one.
const SCREEN_EFFECTS: [&str; 4] = ["turn_in", "open_shop", "fight", "open_board"];
/// STORYUI1 4.7: choice effects that end the choice's effects (the board pane opens, the sequence starts).
const LAST_STORY_EFFECTS: [&str; 2] = ["open_board", "sequence"];

/// Which effect list is checked: a node's, a choice's (the order rules apply), a campaign row's (a meter rule's, a
/// checkpoint's or a trigger's: CAMPAIGN1 T2, no effect that opens a screen), or a sequence's effects step (STORYUI1
/// 4.3: none of SEQUENCE_BANNED_EFFECTS).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectList {
    Node,
    Choice,
    Campaign,
    Sequence,
}

/// Every effect kind with its keys besides `effect`: the dialogue effects, the story effects (STORYUI1 4.7), then the
/// campaign effects (CAMPAIGN1 11.3).
fn all_effects() -> impl Iterator<Item = &'static (&'static str, &'static [&'static str])> {
    EFFECTS.iter().chain(rpgkit_data::STORY_EFFECTS.iter()).chain(rpgkit_data::CAMPAIGN_EFFECTS.iter())
}

/// An effect kind's keys besides `effect`.
fn effect_keys(effect: &str) -> Option<&'static [&'static str]> {
    all_effects().find(|x| x.0 == effect).map(|x| x.1)
}

/// Whether list `list` refuses effect kind `effect` (a campaign row: the screen effects; a sequence step: 4.3's).
fn refused(list: EffectList, effect: &str) -> bool {
    match list {
        EffectList::Campaign => SCREEN_EFFECTS.contains(&effect),
        EffectList::Sequence => rpgkit_data::SEQUENCE_BANNED_EFFECTS.contains(&effect),
        _ => false,
    }
}

/// The effect kinds a list takes, for messages.
fn effect_names(list: EffectList) -> Vec<&'static str> {
    all_effects().map(|x| x.0).filter(|n| !refused(list, n)).collect()
}
const EFFECT_GOLD_MAX: i64 = 99999;
const ADD_DELTA_MAX: i64 = 100;
const ITEM_COUNT_MAX: i64 = 99;
const MAX_ITEM_KEYS: usize = 8;
/// ServiceCatalog's line ids, then STORYUI1's `board` (4.7).
const SERVICE_IDS: [&str; 8] = ["greeting", "offer", "accept", "decline", "turn_in", "shop", "farewell", rpgkit_data::BOARD_SERVICE];
const TOKENS: [&str; 4] = ["npc", "quest", "shop", "gold"];
const TEXT_TOKENS: [&str; 2] = ["npc", "gold"];

fn int_in(v: &Value, lo: i64, hi: i64) -> bool {
    json::is_int(v) && (lo..=hi).contains(&json::to_int(v))
}

/// T1.2-S3 (FB-54c): a name a dialogue stat check in world `w` reads: a primary stat (true) or a U1 standing rating
/// (false), None when it is neither (a rating is one of `STANDING_RATINGS` or a derived-stat row id). AUDIT-05
/// (FB-143): a derived stat counts only when the world's ruleset keeps the row (`rpgkit_rules::derived::row`, the
/// lookup `rpgkit_world::dialogue::member_stat` evaluates through), so a row its `rulesets` leaves out is refused
/// rather than read as 0.
fn stat_kind(v: &View, w: &str, raw: &Value) -> Option<bool> {
    let name = raw.as_str()?;
    if STATS.contains(&name) {
        Some(true)
    } else {
        (STANDING_RATINGS.contains(&name) || rpgkit_rules::derived::row(&v.typed, &world_ruleset_id(v, w), name).is_some()).then_some(false)
    }
}

/// AUDIT-05: the id of the ruleset world `w` plays by, as the run time reads it (`Data::ruleset`): its lineage's nearest
/// `ruleset`, classic when that names no ruleset row.
fn world_ruleset_id(v: &View, w: &str) -> String {
    use rpgkit_rules::ruleset::{world_ruleset, CLASSIC};
    let id = world_ruleset(v.worlds.lineage(w).iter().rev().map(|lw| v.worlds.manifest(lw)));
    if v.typed.rulesets.contains_key(&id) {
        id
    } else {
        CLASSIC.to_string()
    }
}

fn stat_name_error(raw: &Value, label: &str) -> String {
    format!("{label}: stat '{}' is not one of {} or a standing rating ({} or a derived stat id of the world's ruleset).", gstr(raw), STATS.join(", "), STANDING_RATINGS.join(", "))
}

/// DialogueRules.tokens_of: the `{token}` names of `text`, in order (a token holds no braces).
pub fn tokens_of(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut open: Option<usize> = None;
    for (i, ch) in text.char_indices() {
        match ch {
            '{' => open = Some(i + 1),
            '}' => {
                if let Some(start) = open.take() {
                    out.push(text[start..i].to_string());
                }
            }
            _ => {}
        }
    }
    out
}

/// Node id -> node for every node object with a valid id (the first one wins).
fn nodes_by_id(nodes: &[Value]) -> IndexMap<String, &Dict> {
    let mut out = IndexMap::new();
    for node in nodes.iter().filter_map(Value::as_object) {
        if let Some(id) = node.get("id").and_then(Value::as_str).filter(|id| json::id_ok(id)) {
            out.entry(id.to_string()).or_insert(node);
        }
    }
    out
}

/// DialogueRules.tree_errors.
pub fn tree_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let raw = e.get("nodes").cloned().unwrap_or(Value::Array(Vec::new()));
    if !raw.is_array() {
        errors.push("Nodes must be a list of node objects.".into());
        return errors;
    }
    let nodes = json::normalize(&raw);
    let nodes = json::arr(&nodes);
    if nodes.is_empty() || nodes.len() > MAX_NODES {
        errors.push(format!("A dialogue holds 1..{MAX_NODES} nodes."));
    }
    let by_id = nodes_by_id(nodes);
    let mut seen: Vec<String> = Vec::new();
    for (i, node) in nodes.iter().enumerate() {
        let nid = json::s(json::dict(node), "id");
        if node.is_object() && json::id_ok(&nid) {
            if seen.contains(&nid) {
                errors.push(format!("Node id '{nid}' is used twice."));
            }
            seen.push(nid);
        }
        errors.extend(node_errors(v, w, i, node, &by_id));
    }
    let start = json::s(e, "start");
    match by_id.get(&start) {
        None => errors.push(format!("Start node '{start}' does not exist.")),
        Some(node) if json::b(e, "services", true) && json::aget(node, "choices").is_empty() => errors.push("With services on, the start node needs choices.".into()),
        Some(_) => {}
    }
    errors
}

fn node_errors(v: &View, w: &str, index: usize, raw: &Value, by_id: &IndexMap<String, &Dict>) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(node) = raw.as_object() else {
        return vec![format!("Node {} must be an object.", index + 1)];
    };
    let nid = json::s(node, "id");
    let mut label = format!("Node '{nid}'");
    for key in node.keys().filter(|k| !NODE_KEYS.contains(&k.as_str())) {
        errors.push(format!("Node '{nid}': unknown key '{key}'."));
    }
    if !node.get("id").is_some_and(Value::is_string) || !json::id_ok(&nid) {
        errors.push(format!("Node {}: id '{nid}' must be lowercase snake_case (a-z, 0-9, _).", index + 1));
        label = format!("Node {}", index + 1);
    }
    match node.get("speaker").unwrap_or(&Value::from("")) {
        Value::String(s) if s.is_empty() => {}
        Value::String(s) => errors.extend(ref_errors(v, w, &format!("{label} speaker"), "characters", s)),
        _ => errors.push(format!("{label}: speaker must be a character ref or \"\".")),
    }
    errors.extend(expression_error(node, &label, &v.faces()));
    errors.extend(text_errors(node.get("text").unwrap_or(&Value::from("")), &label));
    let choices_raw = node.get("choices").cloned().unwrap_or(Value::Array(Vec::new()));
    let next_raw = node.get("next").cloned().unwrap_or(Value::from(""));
    let end_raw = node.get("end").cloned().unwrap_or(Value::Bool(false));
    if !choices_raw.is_array() {
        errors.push(format!("{label}: choices must be a list."));
    }
    if !next_raw.is_string() {
        errors.push(format!("{label}: next must be a node id."));
    }
    if !end_raw.is_boolean() {
        errors.push(format!("{label}: end must be true or false."));
    }
    let choices = json::arr(&choices_raw);
    let next = next_raw.as_str().unwrap_or("");
    let has_end = end_raw.as_bool().unwrap_or(false);
    if (!choices.is_empty()) as u8 + (!next.is_empty()) as u8 + has_end as u8 != 1 {
        errors.push(format!("Node '{nid}' needs exactly one of choices, next or end."));
    }
    if !next.is_empty() && !by_id.contains_key(next) {
        errors.push(format!("{label}: next '{next}' is not a node of this dialogue."));
    }
    if choices.len() > MAX_CHOICES {
        errors.push(format!("{label} holds 1..{MAX_CHOICES} choices (has {}).", choices.len()));
    }
    for (j, c) in choices.iter().enumerate() {
        errors.extend(choice_errors(v, w, &format!("{label} choice {}", j + 1), c, by_id));
    }
    errors.extend(effects_errors(v, w, node.get("effects").unwrap_or(&Value::Array(Vec::new())), &label, true));
    errors
}

/// PORTRAITFX1 7: a node's or a `line` card's `expression`, when present: "" (neutral) or one of the game's `faces`
/// (PORTRAITFX2 FL4: `View::faces`).
pub fn expression_error(d: &Dict, label: &str, faces: &[String]) -> Option<String> {
    let x = d.get("expression")?;
    let ok = x.as_str().is_some_and(|s| s.is_empty() || faces.iter().any(|f| f == s));
    (!ok).then(|| format!("{label}: expression '{}' is not one of {} (or \"\").", gstr(x), faces.join(", ")))
}

fn text_errors(raw: &Value, label: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(text) = raw.as_str().filter(|t| !t.trim().is_empty()) else {
        return vec![format!("{label} needs text.")];
    };
    if text.chars().count() > MAX_TEXT {
        errors.push(format!("{label}: text is longer than {MAX_TEXT} characters."));
    }
    for token in tokens_of(text).iter().filter(|t| !TEXT_TOKENS.contains(&t.as_str())) {
        errors.push(format!("{label}: Unknown token '{token}' (authored texts allow {{npc}} and {{gold}})."));
    }
    errors
}

fn choice_errors(v: &View, w: &str, label: &str, raw: &Value, by_id: &IndexMap<String, &Dict>) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(c) = raw.as_object() else {
        return vec![format!("{label} must be an object {{\"text\", \"to\", ...}}.")];
    };
    for key in c.keys().filter(|k| !CHOICE_KEYS.contains(&k.as_str())) {
        errors.push(format!("{label}: unknown key '{key}'."));
    }
    errors.extend(text_errors(c.get("text").unwrap_or(&Value::from("")), label));
    match c.get("to").unwrap_or(&Value::from("")) {
        Value::String(to) if !to.is_empty() && !by_id.contains_key(to) => errors.push(format!("{label}: to '{to}' is not a node of this dialogue.")),
        Value::String(_) => {}
        _ => errors.push(format!("{label}: to must be a node id or \"\" (end).")),
    }
    let empty = Value::Array(Vec::new());
    errors.extend(conditions_errors(v, w, c.get("requires").unwrap_or(&empty), &format!("{label} requires")));
    errors.extend(conditions_errors(v, w, c.get("unless").unwrap_or(&empty), &format!("{label} unless")));
    if !c.get("show_locked").unwrap_or(&Value::Bool(false)).is_boolean() {
        errors.push(format!("{label}: show_locked must be true or false."));
    }
    errors.extend(effects_errors(v, w, c.get("effects").unwrap_or(&empty), label, false));
    errors
}

/// DialogueRules.conditions_errors: a dialogue choice's conditions (flag, quest, items, gold, stat).
pub fn conditions_errors(v: &View, w: &str, raw: &Value, label: &str) -> Vec<String> {
    let mut errors = Vec::new();
    if !raw.is_array() {
        return vec![format!("{label} must be a list of conditions.")];
    }
    let list = json::normalize(raw);
    for (i, c) in json::arr(&list).iter().enumerate() {
        let n = i + 1;
        let Some(cond) = c.as_object() else {
            errors.push(format!("{label} {n} must be an object {{\"kind\", ...}}."));
            continue;
        };
        let kind_raw = cond.get("kind").cloned().unwrap_or(Value::from("flag"));
        let kind = gstr(&kind_raw);
        let Some(&(_, allowed)) = CONDITION_KINDS.iter().find(|k| k.0 == kind).filter(|_| kind_raw.is_string()) else {
            errors.push(format!("{label} {n}: unknown condition kind '{kind}' (use {}).", CONDITION_KINDS.iter().map(|k| k.0).collect::<Vec<_>>().join(", ")));
            continue;
        };
        let cl = format!("{label} {n} ({kind})");
        for key in cond.keys().filter(|k| *k != "kind" && !allowed.contains(&k.as_str())) {
            errors.push(format!("{cl}: unknown key '{key}'."));
        }
        match kind.as_str() {
            "flag" => {
                let key = cond.get("key").cloned().unwrap_or(Value::from(""));
                if !key.is_string() || !flag_key_ok(&gstr(&key)) {
                    errors.push(format!("{cl}: flag key '{}' must be non-empty [a-z0-9_.].", gstr(&key)));
                }
                if cond.get("value").is_some_and(|x| !flag_value_ok(x)) {
                    errors.push(format!("{cl}: value must be a bool, a whole number or a String."));
                }
                errors.extend(super::quest::op_errors(cond, &cl));
            }
            "quest" => {
                errors.extend(ref_value_errors(v, w, cond.get("quest").unwrap_or(&Value::from("")), "quests", &format!("{cl} quest")));
                let status = cond.get("status").cloned().unwrap_or(Value::from(""));
                if !status.is_string() || !QUEST_STATUSES.contains(&gstr(&status).as_str()) {
                    errors.push(format!("{cl}: status '{}' is not one of {}.", gstr(&status), QUEST_STATUSES.join(", ")));
                }
            }
            "items" => {
                let item = cond.get("item").cloned().unwrap_or(Value::from(""));
                if !item.is_string() || !v.has("items", &gstr(&item)) {
                    errors.push(format!("{cl}: unknown base item '{}'.", gstr(&item)));
                }
                if !int_in(cond.get("count").unwrap_or(&json::int(1)), 1, ITEM_COUNT_MAX) {
                    errors.push(format!("{cl}: count must be a whole number 1..{ITEM_COUNT_MAX}."));
                }
            }
            "gold" => {
                if !int_in(json::get(cond, "min"), 0, GOLD_CONDITION_MAX) {
                    errors.push(format!("{cl}: min must be a whole number 0..{GOLD_CONDITION_MAX}."));
                }
            }
            _ => {
                let stat = cond.get("stat").cloned().unwrap_or(Value::from(""));
                let primary = stat_kind(v, w, &stat);
                if primary.is_none() {
                    errors.push(stat_name_error(&stat, &cl));
                }
                // T1.2-S1 (FB-54a): `threshold` is a pure alias of `min` (one of them, not both)
                if cond.contains_key("min") && cond.contains_key("threshold") {
                    errors.push(format!("{cl}: give min or threshold (its alias), not both."));
                }
                let key = if cond.contains_key("threshold") && !cond.contains_key("min") { "threshold" } else { "min" };
                // T1.2-S3: a standing rating's bar runs to RATING_BAR_MAX
                let max = if primary == Some(false) { RATING_BAR_MAX } else { STAT_MAX };
                if !int_in(json::get(cond, key), STAT_MIN, max) {
                    errors.push(format!("{cl}: {key} must be a whole number {STAT_MIN}..{max}."));
                }
            }
        }
    }
    errors
}

/// A required world ref held in a json value: a non-empty string that resolves.
fn ref_value_errors(v: &View, w: &str, raw: &Value, ref_kind: &str, label: &str) -> Vec<String> {
    match raw.as_str().filter(|s| !s.is_empty()) {
        None => vec![format!("{label}: choose a {}.", ref_kind.strip_suffix('s').unwrap_or(ref_kind))],
        Some(r) => ref_errors(v, w, label, ref_kind, r),
    }
}

/// DialogueRules.effects_errors (`node_level`: node effects, set_flag / add_flag / give and STORYUI1's vignette;
/// otherwise choice effects with the order rules).
pub fn effects_errors(v: &View, w: &str, raw: &Value, label: &str, node_level: bool) -> Vec<String> {
    list_errors(v, w, raw, label, if node_level { EffectList::Node } else { EffectList::Choice })
}

/// The effects_errors of any effect list (CAMPAIGN1 11.3: the campaign effects are known to every list; a node takes
/// set_flag / add_flag / give and vignette, a choice has the order rules, a campaign row takes no effect that opens a
/// screen, a sequence step none of SEQUENCE_BANNED_EFFECTS).
pub fn list_errors(v: &View, w: &str, raw: &Value, label: &str, kind: EffectList) -> Vec<String> {
    let mut errors = Vec::new();
    if !raw.is_array() {
        return vec![format!("{label}: effects must be a list.")];
    }
    let list = json::normalize(raw);
    let list = json::arr(&list);
    if list.len() > MAX_EFFECTS {
        errors.push(format!("{label} holds at most {MAX_EFFECTS} effects (has {}).", list.len()));
    }
    let mut names: Vec<String> = Vec::new();
    for (k, raw) in list.iter().enumerate() {
        let Some(fx) = raw.as_object() else {
            errors.push(format!("{label} effect {} must be an object {{\"effect\", ...}}.", k + 1));
            names.push(String::new());
            continue;
        };
        let effect = json::s(fx, "effect");
        let Some(allowed) = effect_keys(&effect) else {
            errors.push(format!("{label} effect {}: unknown effect '{effect}' (use {}).", k + 1, effect_names(kind).join(", ")));
            names.push(String::new());
            continue;
        };
        names.push(effect.clone());
        let el = format!("{label} effect {} ({effect})", k + 1);
        // STORYUI1 4.7: a node queues a vignette (the conversation goes on); `sequence` and `open_board` close the
        // conversation or open a screen, so they belong on a choice. Godot's message stays for its own effects.
        if kind == EffectList::Node && LAST_STORY_EFFECTS.contains(&effect.as_str()) {
            errors.push(format!("{el}: {effect} closes the conversation or opens a screen; put it on a choice (node effects: set_flag, add_flag, give, vignette)."));
        } else if kind == EffectList::Node && !NODE_EFFECTS.contains(&effect.as_str()) && !NODE_STORY_EFFECTS.contains(&effect.as_str()) {
            errors.push(format!("{el}: Node effects may only be set_flag, add_flag or give."));
        }
        // T1.2-S2: a roll happens when the player picks a choice (a node's list says so above)
        if effect == ROLL_EFFECT && matches!(kind, EffectList::Campaign | EffectList::Sequence) {
            errors.push(format!("{el}: roll is a choice effect (it rolls when the player picks the choice)."));
            continue;
        }
        if kind == EffectList::Campaign && SCREEN_EFFECTS.contains(&effect.as_str()) {
            errors.push(format!("{el}: {effect} opens a screen; queue a battle or a dialogue in then instead."));
            continue;
        }
        if kind == EffectList::Sequence && refused(kind, &effect) {
            errors.push(format!("{el}: {effect} can't run in a sequence step (use a fight step, or a dialogue outside the sequence)."));
            continue;
        }
        for key in fx.keys().filter(|k| *k != "effect" && !allowed.contains(&k.as_str())) {
            errors.push(format!("{el}: unknown key '{key}'."));
        }
        errors.extend(effect_errors(v, w, &effect, fx, &el));
    }
    if kind == EffectList::Choice {
        errors.extend(order_errors(&names, label));
        if names.iter().filter(|n| *n == "recruit").count() > 1 {
            errors.push(format!("{label}: at most one recruit per choice."));
        }
        // CAMPAIGN1 11.3: nothing runs after an end
        if names.iter().position(|n| n == "end").is_some_and(|at| at + 1 != names.len()) {
            errors.push(format!("{label}: end must be the last effect."));
        }
        // STORYUI1 4.7: the board pane opens and a sequence starts after the choice's other effects
        for last in LAST_STORY_EFFECTS {
            if names.iter().position(|n| n == last).is_some_and(|at| at + 1 != names.len()) {
                errors.push(format!("{label}: {last} must be the last effect."));
            }
        }
    }
    errors
}

fn effect_errors(v: &View, w: &str, effect: &str, fx: &Dict, el: &str) -> Vec<String> {
    let mut errors = Vec::new();
    match effect {
        "set_flag" => {
            let key = fx.get("key").cloned().unwrap_or(Value::from(""));
            if !key.is_string() || !flag_key_ok(&gstr(&key)) {
                errors.push(format!("{el}: flag key '{}' must be non-empty [a-z0-9_.].", gstr(&key)));
            }
            if fx.get("value").is_some_and(|x| !flag_value_ok(x)) {
                errors.push(format!("{el}: value must be a bool, a whole number or a String."));
            }
            let why = rep_value_error(&gstr(&key), fx.get("value").unwrap_or(&Value::Bool(true)));
            if !why.is_empty() {
                errors.push(format!("{el}: {why}"));
            }
            errors.extend(reserved_errors(&gstr(&key), el));
        }
        "add_flag" => {
            let key = fx.get("key").cloned().unwrap_or(Value::from(""));
            if !key.is_string() || !flag_key_ok(&gstr(&key)) {
                errors.push(format!("{el}: flag key '{}' must be non-empty [a-z0-9_.].", gstr(&key)));
            }
            errors.extend(reserved_errors(&gstr(&key), el));
            let delta = json::get(fx, "delta");
            if !json::is_int(delta) || json::to_int(delta) == 0 || json::to_int(delta).abs() > ADD_DELTA_MAX {
                errors.push(format!("{el}: delta must be a whole number from -{ADD_DELTA_MAX} to {ADD_DELTA_MAX}, not 0."));
            }
            for bound in ["min", "max"] {
                if fx.get(bound).is_some_and(|b| !b.is_null() && !json::is_int(b)) {
                    errors.push(format!("{el}: {bound} must be a whole number."));
                }
            }
            let (lo, hi) = (json::get(fx, "min"), json::get(fx, "max"));
            if json::is_int(lo) && json::is_int(hi) && json::to_int(lo) > json::to_int(hi) {
                errors.push(format!("{el}: min must not be above max."));
            }
        }
        "start_quest" | "turn_in" => {
            let found = ref_value_errors(v, w, fx.get("quest").unwrap_or(&Value::from("")), "quests", &format!("{el} quest"));
            let ok = found.is_empty();
            errors.extend(found);
            let quest = json::s(fx, "quest");
            if ok && json::s(v.worlds.resolve(w, "quests", &quest), "template").is_empty() {
                errors.push(format!("{el}: quest '{quest}' has no template (a lore-only quest never runs)."));
            }
        }
        "give" | "take" => errors.extend(goods_errors(v, fx, el)),
        // T1.2-S2 (FB-54b): the stat or standing rating, the bar, the outcome flag and the optional member
        ROLL_EFFECT => {
            let stat = fx.get("stat").cloned().unwrap_or(Value::from(""));
            if stat_kind(v, w, &stat).is_none() {
                errors.push(stat_name_error(&stat, el));
            }
            if !int_in(json::get(fx, "dc"), ROLL_DC_MIN, ROLL_DC_MAX) {
                errors.push(format!("{el}: dc must be a whole number {ROLL_DC_MIN}..{ROLL_DC_MAX}."));
            }
            let key = fx.get("key").cloned().unwrap_or(Value::from(""));
            if !key.is_string() || !flag_key_ok(&gstr(&key)) {
                errors.push(format!("{el}: flag key '{}' must be non-empty [a-z0-9_.].", gstr(&key)));
            }
            // the outcome is true or false, which a reputation key never holds
            let why = rep_value_error(&gstr(&key), &Value::Bool(true));
            if !why.is_empty() {
                errors.push(format!("{el}: {why}"));
            }
            errors.extend(reserved_errors(&gstr(&key), el));
            match fx.get("who").unwrap_or(&Value::from("")) {
                Value::String(s) if s.is_empty() => {}
                Value::String(s) => errors.extend(ref_errors(v, w, &format!("{el} who"), "characters", s)),
                _ => errors.push(format!("{el}: who must be a character ref or \"\".")),
            }
        }
        "open_shop" => errors.extend(ref_value_errors(v, w, fx.get("shop").unwrap_or(&Value::from("")), "shops", &format!("{el} shop"))),
        "fight" => errors.extend(ref_value_errors(v, w, fx.get("encounter").unwrap_or(&Value::from("")), "encounters", &format!("{el} encounter"))),
        // STORYUI1 4.7
        "vignette" => errors.extend(ref_value_errors(v, w, fx.get("vignette").unwrap_or(&Value::from("")), "vignettes", &format!("{el} vignette"))),
        "sequence" => errors.extend(ref_value_errors(v, w, fx.get("sequence").unwrap_or(&Value::from("")), "sequences", &format!("{el} sequence"))),
        "open_board" => errors.extend(ref_value_errors(v, w, fx.get("board").unwrap_or(&Value::from("")), "boards", &format!("{el} board"))),
        "recruit" => {
            match fx.get("character").and_then(Value::as_str).filter(|s| !s.is_empty()) {
                None => errors.push(format!("{el}: choose a character.")),
                Some(id) if v.worlds.resolve(w, "characters", id).is_empty() => errors.push(format!("{el}: Recruit: unknown character '{id}'.")),
                Some(id) => errors.extend(party_character_errors(v, w, id).into_iter().map(|why| format!("{el}: {why}"))),
            }
            // ROSTER1 BK5: the join level
            use rpgkit_data::{RECRUIT_LEVELS, RECRUIT_LEVEL_MAX};
            if let Some(level) = fx.get("level").filter(|l| !(l.as_str().is_some_and(|s| RECRUIT_LEVELS.contains(&s)) || int_in(l, 1, RECRUIT_LEVEL_MAX))) {
                errors.push(format!("{el}: level '{}' must be {} or a whole number 1..{RECRUIT_LEVEL_MAX}.", gstr(level), RECRUIT_LEVELS.join(", ")));
            }
        }
        _ => errors.extend(super::campaign::effect_errors(v, w, effect, fx, el)),
    }
    errors
}

/// CAMPAIGN1 11.4: a flag effect may not write a reserved key (meter., campaign., roster.).
fn reserved_errors(key: &str, el: &str) -> Option<String> {
    let why = super::campaign::write_key_error(key);
    (!why.is_empty()).then(|| format!("{el}: {why}"))
}

/// give / take: items {base id: 1..99} (at most 8 keys), gold 0..EFFECT_GOLD_MAX, not both empty.
fn goods_errors(v: &View, fx: &Dict, el: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let items_raw = fx.get("items").cloned().unwrap_or(Value::Object(Dict::new()));
    let items = json::dict(&items_raw);
    if !items_raw.is_object() {
        errors.push(format!("{el}: items must be an object {{base item id: count}}."));
    } else if items.len() > MAX_ITEM_KEYS {
        errors.push(format!("{el} lists at most {MAX_ITEM_KEYS} items."));
    }
    for (id, count) in items {
        if !v.has("items", id) {
            errors.push(format!("{el}: unknown base item '{id}'."));
        }
        if !int_in(count, 1, ITEM_COUNT_MAX) {
            errors.push(format!("{el}: count of '{id}' must be a whole number 1..{ITEM_COUNT_MAX}."));
        }
    }
    let gold = fx.get("gold").cloned().unwrap_or(json::int(0));
    if !int_in(&gold, 0, EFFECT_GOLD_MAX) {
        errors.push(format!("{el}: gold must be a whole number 0..{EFFECT_GOLD_MAX}."));
    }
    if items.is_empty() && json::to_int(&gold) <= 0 {
        errors.push(format!("{el} needs items or gold."));
    }
    errors
}

/// The choice order rules: turn_in first (followed only by node effects), open_shop and fight last, fight never with
/// turn_in or open_shop.
fn order_errors(names: &[String], label: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let has = |n: &str| names.iter().any(|x| x == n);
    let share = has("fight") && (has("turn_in") || has("open_shop"));
    let movers = ["open_shop", "fight"];
    if let Some(turn_at) = names.iter().position(|n| n == "turn_in") {
        let bad = turn_at != 0 || names[turn_at + 1..].iter().any(|n| !n.is_empty() && !NODE_EFFECTS.contains(&n.as_str()) && !(share && movers.contains(&n.as_str())));
        if bad {
            errors.push(format!("{label}: turn_in must come first; only set_flag, add_flag or give may follow."));
        }
    }
    if share {
        errors.push(format!("{label}: fight can't share a choice with turn_in or open_shop."));
        return errors;
    }
    for last_only in movers {
        for (k, n) in names.iter().enumerate() {
            if n == last_only && k != names.len() - 1 {
                errors.push(format!("{label}: {last_only} must be the last effect."));
            }
        }
    }
    errors
}

/// ENCOUNTER2-C DT9 (decision "ENCOUNTER2-C answers" Q-C2): the effects a battle talk may run.
pub const BATTLE_TALK_EFFECTS: [&str; 2] = ["set_flag", "add_flag"];

/// ENCOUNTER2-C 5.2: tree `id` opened by a phase `talk`: every node and choice effect is set_flag or add_flag (one
/// error per other effect, by node).
pub fn battle_talk_errors(id: &str, tree: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    for node in json::aget(tree, "nodes").iter().filter_map(Value::as_object) {
        let nid = json::s(node, "id");
        let choice_effects = json::aget(node, "choices").iter().filter_map(Value::as_object).flat_map(|c| json::aget(c, "effects").iter());
        for fx in json::aget(node, "effects").iter().chain(choice_effects).filter_map(Value::as_object) {
            let effect = json::s(fx, "effect");
            if !BATTLE_TALK_EFFECTS.contains(&effect.as_str()) {
                errors.push(format!("Battle talk '{id}' node '{nid}': effect '{effect}' cannot run in a battle."));
            }
        }
    }
    errors
}

/// DialogueRules.line_errors: one service line (a known id, text, known tokens).
pub fn line_errors(id: &str, text: &Value, label: &str) -> Vec<String> {
    let mut errors = Vec::new();
    if !SERVICE_IDS.contains(&id) {
        errors.push(format!("{label}: unknown service line '{id}' (use {}).", SERVICE_IDS.join(", ")));
    }
    let Some(text) = text.as_str().filter(|t| !t.trim().is_empty()) else {
        errors.push(format!("{label}: service line '{id}' needs text."));
        return errors;
    };
    if text.chars().count() > MAX_TEXT {
        errors.push(format!("{label}: service line '{id}' is longer than {MAX_TEXT} characters."));
    }
    for token in tokens_of(text).iter().filter(|t| !TOKENS.contains(&t.as_str())) {
        errors.push(format!("{label}: Unknown token '{token}'."));
    }
    errors
}

/// DialogueRules.service_lines_errors: a manifest's {line id: text}.
pub fn service_lines_errors(raw: &Value, label: &str) -> Vec<String> {
    match raw.as_object() {
        None => vec![format!("{label} must be an object {{line id: text}}.")],
        Some(lines) => lines.iter().flat_map(|(id, text)| line_errors(id, text, label)).collect(),
    }
}

// ---- warnings ----

type Edges = IndexMap<String, Vec<(String, Dict)>>;

fn has_effect(raw: &Value, effect: &str) -> bool {
    json::arr(raw).iter().any(|fx| json::s(json::dict(fx), "effect") == effect)
}

/// Node id -> [(to, choice)] ({} for a `next` edge or the generated service edge back to the start).
fn edges(by_id: &IndexMap<String, &Dict>, start: &str, services: bool) -> Edges {
    let mut out = Edges::new();
    for (nid, node) in by_id {
        let mut list = Vec::new();
        if let Some(next) = node.get("next").and_then(Value::as_str).filter(|n| by_id.contains_key(*n)) {
            list.push((next.to_string(), Dict::new()));
        }
        for c in json::aget(node, "choices") {
            let choice = json::dict(c);
            let mut to = json::s(choice, "to");
            if to.is_empty() && has_effect(json::get(choice, "effects"), "open_shop") {
                to = nid.clone();
            }
            if by_id.contains_key(&to) {
                list.push((to, choice.clone()));
            }
        }
        if services && nid == start {
            list.push((start.to_string(), Dict::new()));
        }
        out.insert(nid.clone(), list);
    }
    out
}

/// Every node reachable in one or more steps from `from`.
fn reach(edges: &Edges, from: &str) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    let mut queue = std::collections::VecDeque::from([from.to_string()]);
    while let Some(at) = queue.pop_front() {
        for (to, _) in edges.get(&at).map(Vec::as_slice).unwrap_or(&[]) {
            if !seen.contains(to) {
                seen.push(to.clone());
                queue.push_back(to.clone());
            }
        }
    }
    seen
}

/// Same flag key, op and value (strict type equality, as QuestRules.condition_met).
fn same_condition(a: &Dict, b: &Dict) -> bool {
    let value = |d: &Dict| json::normalize(d.get("value").unwrap_or(&Value::Bool(true)));
    json::s(a, "key") == json::s(b, "key") && json::s_or(a, "op", "==") == json::s_or(b, "op", "==") && value(a) == value(b)
}

/// True when every way into `nid` is a choice guarded by an `unless` flag that the choice or the node sets.
fn guarded(edges: &Edges, by_id: &IndexMap<String, &Dict>, nid: &str) -> bool {
    let node = by_id[nid];
    for list in edges.values() {
        for (to, choice) in list {
            if to != nid {
                continue;
            }
            if choice.is_empty() {
                return false;
            }
            let sets: Vec<&Dict> = json::aget(node, "effects").iter().chain(json::aget(choice, "effects")).map(json::dict).collect();
            let ok = json::aget(choice, "unless").iter().map(json::dict).filter(|c| json::s_or(c, "kind", "flag") == "flag").any(|cond| sets.iter().any(|f| json::s(f, "effect") == "set_flag" && same_condition(f, cond)));
            if !ok {
                return false;
            }
        }
    }
    true
}

/// DialogueRules.tree_warnings: unreachable nodes, and unguarded `give` nodes and `roll` choices (AUDIT-03) on a cycle.
fn tree_warnings(e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let nodes = json::normalize(json::get(e, "nodes"));
    let nodes = json::arr(&nodes);
    let by_id = nodes_by_id(nodes);
    let start = json::s(e, "start");
    if !by_id.contains_key(&start) {
        return out;
    }
    let edges = edges(&by_id, &start, json::b(e, "services", true));
    let mut reached = reach(&edges, &start);
    reached.push(start.clone());
    for n in nodes {
        let nid = json::s(json::dict(n), "id");
        if by_id.contains_key(&nid) && !reached.contains(&nid) {
            out.push(format!("Node '{nid}' is not reachable from the start node."));
        }
    }
    for (nid, node) in &by_id {
        if !has_effect(json::get(node, "effects"), "give") || !reach(&edges, nid).contains(nid) || guarded(&edges, &by_id, nid) {
            continue;
        }
        out.push(format!("Node '{nid}' gives items or gold and sits on a dialogue cycle (it can be shown again and again); guard every way in with unless + set_flag."));
    }
    for (nid, node) in &by_id {
        if reach(&edges, nid).contains(nid) {
            out.extend(roll_loop_warnings(nid, node));
        }
    }
    out
}

/// AUDIT-03 (FB-143, T_STORY 1.1 anti-save-scum): the `roll` choices of node `nid`, which sits on a dialogue cycle (it
/// can be shown again), that nothing hides once picked: a roll choice is guarded when its `unless` holds a flag
/// condition that a `set_flag` of the choice or the node sets (as a `give` node's way in). An `unless` on the roll's
/// own `key` is no guard, nor is a `set_flag` of that key (R4-2 AUDIT-06): the roll sets `key` false on a failure and an
/// unset flag reads false, so the choice stays open after a failed roll. An unguarded one can be picked again and again until the roll succeeds.
fn roll_loop_warnings(nid: &str, node: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    for (j, c) in json::aget(node, "choices").iter().enumerate() {
        let choice = json::dict(c);
        let sets: Vec<&Dict> = json::aget(node, "effects").iter().chain(json::aget(choice, "effects")).map(json::dict).collect();
        let unless: Vec<&Dict> = json::aget(choice, "unless").iter().map(json::dict).filter(|c| json::s_or(c, "kind", "flag") == "flag").collect();
        for fx in json::aget(choice, "effects").iter().map(json::dict).filter(|fx| json::s(fx, "effect") == "roll") {
            let key = json::s(fx, "key");
            let guarded = unless.iter().any(|cond| sets.iter().any(|f| json::s(f, "effect") == "set_flag" && json::s(f, "key") != key && same_condition(f, cond)));
            if !guarded {
                out.push(format!("Node '{nid}' choice {} rolls '{key}' and sits on a dialogue cycle (it can be picked again until the roll succeeds); guard it with a set_flag the choice sets and an unless on that flag.", j + 1));
            }
        }
    }
    out
}

/// DialogueRules.dialogue_warnings: the tree warnings of a world's own dialogues.
pub fn dialogue_warnings(v: &View, w: &str) -> Vec<String> {
    v.worlds.own_entries(w, "dialogues").iter().flat_map(|e| tree_warnings(e).into_iter().map(move |warning| format!("{}: {warning}", json::s(e, "_qid")))).collect()
}

/// DialogueRules.quest_warnings: a giver or turn-in who can be gone without a variant that re-routes the quest.
pub fn quest_warnings(v: &View, w: &str) -> Vec<String> {
    let mut out = Vec::new();
    for q in v.worlds.merged_entries(w, "quests") {
        if json::s(q, "template").is_empty() {
            continue;
        }
        let variants = json::normalize(json::get(q, "variants"));
        let variants = json::arr(&variants);
        let mut roles = vec![("giver", json::s(q, "giver"), "giver")];
        let turn_in = json::s(q, "turn_in");
        if !turn_in.is_empty() {
            roles.push(("turn-in", turn_in, "turn_in"));
        }
        for (role, r, field) in roles {
            let who = v.worlds.resolve(w, "characters", &r);
            if r.is_empty() || who.is_empty() {
                continue;
            }
            let gone = json::normalize(json::get(who, "gone_flags"));
            for cond in json::arr(&gone).iter().map(json::dict) {
                let rerouted = variants.iter().map(json::dict).any(|variant| {
                    let set = json::dget(variant, "set");
                    let reroutes = set.contains_key(field) || set.get("available").is_some_and(|a| a.as_bool() == Some(false));
                    reroutes && json::aget(variant, "when_flags").iter().any(|wf| same_condition(json::dict(wf), cond))
                });
                if !rerouted {
                    let op = json::s_or(cond, "op", "==");
                    let text = format!("{} {} {}", json::s(cond, "key"), if op == "==" { "=" } else { op.as_str() }, gstr(&json::normalize(cond.get("value").unwrap_or(&Value::Bool(true)))));
                    out.push(format!("Quest '{}': {role} '{r}' can be gone ({text}) and no variant re-routes it.", json::s(q, "_qid")));
                }
            }
        }
    }
    out
}
