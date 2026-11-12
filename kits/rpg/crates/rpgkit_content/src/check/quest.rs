//! Quest rules of the world checks (QuestRules: rewards, flag conditions, world quests, loot entries and their soft
//! warnings) and the reputation keys (RepRules.value_error / key_warnings).

use super::world::{qid, ref_errors};
use super::View;
use aegis_core::json::{self, gstr, Dict};
use serde_json::Value;

/// ACT1-HOOKS1 (FB-241 c): `supplies` (travel supplies, 0..the purse's top) joined the reward keys.
const REWARD_KEYS: [&str; 6] = ["xp", "items", "passives", "skills", "gold", "supplies"];
const XP_MAX: i64 = 2000;
const GOLD_MAX: i64 = 5000;
const ITEM_COUNT_MAX: i64 = 99;
const MAX_ITEM_KEYS: usize = 8;
const SOFT_MAX_PASSIVES: i64 = 1;
const SOFT_MAX_SKILLS: i64 = 1;
const SOFT_MAX_ITEMS: i64 = 6;
const SOFT_MAX_PICKS: i64 = 1;
const MAX_VARIANTS: usize = 8;
const VARIANT_KEYS: [&str; 4] = ["id", "when_flags", "unless_flags", "set"];
const VARIANT_FIELDS: [&str; 10] = ["giver", "turn_in", "description", "reward", "rewards_from", "steps", "links", "available", "loot_pool", "loot_picks"];
const LINK_KINDS: [(&str, &str); 6] = [("follows", "quests"), ("leads_to", "quests"), ("related_quests", "quests"), ("related_characters", "characters"), ("related_locations", "locations"), ("related_factions", "factions")];
const MAX_POOL: usize = 8;
const MAX_PICKS: i64 = 3;
const LOOT_KINDS: [&str; 3] = ["item", "passive", "skill"];
const OPS: [&str; 6] = ["==", "!=", ">=", "<=", ">", "<"];
const ORDER_OPS: [&str; 4] = [">=", "<=", ">", "<"];
const CONDITION_KEYS: [&str; 3] = ["key", "value", "op"];
const REP_DELTA_MAX: i64 = 50;
const MAX_REP_KEYS: usize = 4;
pub const SET_FLAG_LISTS: [&str; 2] = ["complete_flags", "sets_flags"];

/// RepRules: flag keys "rep.<world>.<faction>" hold a standing in REP_MIN..REP_MAX.
pub const REP_PREFIX: &str = "rep.";
pub const REP_MIN: i64 = -100;
pub const REP_MAX: i64 = 100;

/// The step keys an objective kind binds (the first is required).
fn binding_keys(kind: &str) -> &'static [&'static str] {
    match kind {
        "reach_location" => &["location"],
        "win_encounter" => &["encounter", "location"],
        "flag" => &["flag", "value"],
        _ => &[],
    }
}

/// A flag value: a bool, a whole number or a string.
pub fn flag_value_ok(v: &Value) -> bool {
    v.is_boolean() || v.is_string() || json::is_int(&json::normalize(v))
}

/// GameState.flag_key_ok: non-empty [a-z0-9_.].
pub fn flag_key_ok(key: &str) -> bool {
    !key.is_empty() && key.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.')
}

/// RepRules.value_error: "" unless `key` is a rep key whose value is not a whole number in REP_MIN..REP_MAX.
pub fn rep_value_error(key: &str, value: &Value) -> String {
    if !key.starts_with(REP_PREFIX) {
        return String::new();
    }
    let v = json::normalize(value);
    if !json::is_int(&v) || !(REP_MIN..=REP_MAX).contains(&json::to_int(&v)) {
        return format!("Reputation '{key}' must be a whole number from {REP_MIN} to {REP_MAX}.");
    }
    String::new()
}

/// QuestRules.reward_ref_error: "" when `r` can be granted as a reward of `kind`, else why not.
pub fn reward_ref_error(v: &View, kind: &str, r: &str) -> String {
    if r.is_empty() {
        return format!("a {kind} reward needs an id");
    }
    match kind {
        "item" if !v.has("items", r) => format!("unknown item '{r}'"),
        "passive" if !v.has("passives", r) => format!("unknown passive '{r}'"),
        "skill" => {
            let rows: Vec<Dict> = v.entries("skills").iter().filter(|s| super::skill::head_of(s) == r).map(|s| crate::source::content("skills", s)).collect();
            if rows.is_empty() || super::skill::fold(r, &rows).is_err() {
                format!("unknown skill family '{r}' (name the family head)")
            } else if v.typed.is_basic_family(r) {
                format!("'{r}' is a basic skill every unit owns; it can't be a reward")
            } else {
                String::new()
            }
        }
        "item" | "passive" => String::new(),
        _ => format!("unknown reward kind '{kind}'"),
    }
}

/// QuestRules.rewards_errors: a reward object {xp, items, passives, skills, gold, supplies}.
pub fn rewards_errors(v: &View, raw: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(rewards) = json::normalize(raw).as_object().cloned() else {
        errors.push("Rewards must be an object {xp, items, passives, skills, gold, supplies}.".into());
        return errors;
    };
    for key in rewards.keys().filter(|k| !REWARD_KEYS.contains(&k.as_str())) {
        errors.push(format!("Unknown reward key '{key}' (allowed: {}).", REWARD_KEYS.join(", ")));
    }
    if let Some(xp) = rewards.get("xp") {
        if !json::is_int(xp) {
            errors.push("Rewards xp must be a whole number.".into());
        } else if !(0..=XP_MAX).contains(&json::to_int(xp)) {
            errors.push(format!("Rewards xp must be 0..{XP_MAX}."));
        }
    }
    if let Some(gold) = rewards.get("gold") {
        if !json::is_int(gold) || !(0..=GOLD_MAX).contains(&json::to_int(gold)) {
            errors.push(format!("Rewards gold must be a whole number 0..{GOLD_MAX}."));
        }
    }
    if let Some(supplies) = rewards.get("supplies") {
        let top = crate::source::PurseCaps::SUPPLIES_MAX;
        if !json::is_int(supplies) || !(0..=top).contains(&json::to_int(supplies)) {
            errors.push(format!("Rewards supplies must be a whole number 0..{top}."));
        }
    }
    if let Some(items) = rewards.get("items") {
        match items.as_object() {
            None => errors.push("Rewards items must be an object {item_id: count}.".into()),
            Some(items) => {
                if items.len() > MAX_ITEM_KEYS {
                    errors.push(format!("Rewards list at most {MAX_ITEM_KEYS} items."));
                }
                for (id, count) in items {
                    let why = reward_ref_error(v, "item", id);
                    if !why.is_empty() {
                        errors.push(format!("Rewards items: {why}."));
                    }
                    if !json::is_int(count) || !(1..=ITEM_COUNT_MAX).contains(&json::to_int(count)) {
                        errors.push(format!("Rewards items: '{id}' count must be a whole number 1..{ITEM_COUNT_MAX}."));
                    }
                }
            }
        }
    }
    for (key, kind) in [("passives", "passive"), ("skills", "skill")] {
        let Some(list) = rewards.get(key) else { continue };
        let Some(list) = list.as_array() else {
            errors.push(format!("Rewards {key} must be a list of ids."));
            continue;
        };
        let mut seen: Vec<String> = Vec::new();
        for x in list {
            let id = gstr(x);
            if !x.is_string() {
                errors.push(format!("Rewards {key}: '{id}' is not an id."));
                continue;
            }
            if seen.contains(&id) {
                errors.push(format!("Rewards {key} lists '{id}' twice."));
            }
            seen.push(id.clone());
            let why = reward_ref_error(v, kind, &id);
            if !why.is_empty() {
                errors.push(format!("Rewards {key}: {why}."));
            }
        }
    }
    errors
}

fn item_total(rewards: &Dict) -> i64 {
    json::dget(rewards, "items").values().map(json::to_int).sum()
}

/// QuestRules.condition_errors: a list of flag conditions {"key", "value", "op"}.
pub fn condition_errors(raw: &Value, label: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let Value::Array(list) = json::normalize(raw) else {
        errors.push(format!("{label} must be a list of {{\"key\", \"value\"}} conditions."));
        return errors;
    };
    for (i, c) in list.iter().enumerate() {
        let n = i + 1;
        let Some(cond) = c.as_object() else {
            errors.push(format!("{label} {n} must be an object {{\"key\", \"value\"}}."));
            continue;
        };
        let key = cond.get("key").cloned().unwrap_or(Value::from(""));
        if !key.is_string() || !flag_key_ok(&gstr(&key)) {
            errors.push(format!("{label} {n}: flag key '{}' must be non-empty [a-z0-9_.].", gstr(&key)));
        }
        if cond.get("value").is_some_and(|x| !flag_value_ok(x)) {
            errors.push(format!("{label} {n}: value must be a bool, a whole number or a String."));
        }
        errors.extend(op_errors(cond, &format!("{label} {n}")));
        for k in cond.keys().filter(|k| !CONDITION_KEYS.contains(&k.as_str())) {
            errors.push(format!("{label} {n}: unknown key '{k}'."));
        }
    }
    errors
}

/// QuestRules.op_errors: a known op; an ordering op needs a whole-number value (in the rep range for a rep key).
pub fn op_errors(cond: &Dict, cl: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(op_raw) = cond.get("op") else { return errors };
    let op = gstr(op_raw);
    if !op_raw.is_string() || !OPS.contains(&op.as_str()) {
        errors.push(format!("{cl}: op '{op}' must be one of == != >= <= > <."));
        return errors;
    }
    if !ORDER_OPS.contains(&op.as_str()) {
        return errors;
    }
    let value = json::normalize(cond.get("value").unwrap_or(&Value::Bool(true)));
    if !json::is_int(&value) {
        errors.push(format!("{cl}: op '{op}' needs a whole-number value."));
        return errors;
    }
    let why = rep_value_error(&json::s(cond, "key"), &value);
    if !why.is_empty() {
        errors.push(format!("{cl}: {why}"));
    }
    errors
}

/// QuestRules.flag_list_errors: a list that sets flags gets the rep value rule, every other a condition check.
pub fn flag_list_errors(raw: &Value, label: &str, list_key: &str) -> Vec<String> {
    if !SET_FLAG_LISTS.contains(&list_key) {
        return condition_errors(raw, label);
    }
    let mut errors = condition_errors(raw, label);
    if let Value::Array(list) = json::normalize(raw) {
        for (i, c) in list.iter().enumerate() {
            let cond = json::dict(c);
            let why = rep_value_error(&json::s(cond, "key"), cond.get("value").unwrap_or(&Value::Bool(true)));
            if !why.is_empty() {
                errors.push(format!("{label} {}: {why}", i + 1));
            }
        }
    }
    errors
}

fn template_kinds(template: &Dict) -> Vec<String> {
    json::aget(template, "objectives").iter().map(|o| json::s(json::dict(o), "kind")).collect()
}

/// QuestRules.world_quest_errors.
pub fn world_quest_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let template_id = json::s(e, "template");
    let has_template = !template_id.is_empty();
    let template = v.get("quest_templates", &template_id);
    if has_template && template.is_empty() {
        errors.push(format!("Template: unknown quest template '{template_id}'."));
    }
    errors.extend(condition_errors(e.get("requires_flags").unwrap_or(&Value::Array(Vec::new())), "Requires flags"));
    errors.extend(flag_list_errors(e.get("complete_flags").unwrap_or(&Value::Array(Vec::new())), "Complete flags", "complete_flags"));
    errors.extend(reputation_errors(v, w, e.get("reputation").unwrap_or(&Value::Object(Dict::new())), has_template));
    if json::b(e, "auto_start", false) && !has_template {
        errors.push("Auto start needs a template (a lore-only quest never runs).".into());
    }
    let steps = json::normalize(json::get(e, "steps"));
    let steps = json::arr(&steps);
    let kinds = template_kinds(template);
    if has_template && !template.is_empty() {
        if steps.len() != kinds.len() {
            errors.push(format!("The template has {} objectives but the quest has {} steps (one step per objective).", kinds.len(), steps.len()));
        }
        for (i, (step, kind)) in steps.iter().zip(&kinds).enumerate() {
            errors.extend(step_binding_errors(v, w, i, kind, json::dict(step), true));
        }
    }
    errors.extend(cycle_errors(v, w, e));
    errors.extend(links_errors(v, w, e, e.get("links").unwrap_or(&Value::Object(Dict::new())), "Links"));
    let pool = e.get("loot_pool").cloned().unwrap_or(Value::Array(Vec::new()));
    let pool_size = json::arr(&pool).len() as i64;
    errors.extend(loot_pool_errors(v, w, &pool, "Loot pool"));
    if pool_size > 0 && !has_template {
        errors.push("Loot pool needs a template (a lore-only quest grants nothing).".into());
    }
    let picks = e.get("loot_picks").cloned().unwrap_or(json::int(1));
    errors.extend(picks_errors(&picks, "Loot picks"));
    if pool_size > 0 && json::to_int(&picks) > pool_size {
        errors.push(format!("Loot picks ({}) is more than the pool holds ({pool_size}).", json::to_int(&picks)));
    }
    match e.get("variants").unwrap_or(&Value::Array(Vec::new())) {
        Value::Array(list) if list.is_empty() => {}
        Value::Array(_) => {
            if !has_template {
                errors.push("Variants need a template (a lore-only quest never runs).".into());
            }
            errors.extend(variants_errors(v, w, e, &kinds));
        }
        _ => errors.push("Variants must be a list.".into()),
    }
    errors
}

/// QuestRules.reputation_errors: {faction ref: delta}.
fn reputation_errors(v: &View, w: &str, raw: &Value, has_template: bool) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(rep) = json::normalize(raw).as_object().cloned() else {
        errors.push("Reputation must be an object {faction: delta}.".into());
        return errors;
    };
    if rep.is_empty() {
        return errors;
    }
    if !has_template {
        errors.push("Reputation needs a template.".into());
    }
    if rep.len() > MAX_REP_KEYS {
        errors.push(format!("Reputation lists at most {MAX_REP_KEYS} factions (has {}).", rep.len()));
    }
    for (r, d) in &rep {
        if r.is_empty() || v.worlds.resolve(w, "factions", r).is_empty() || !ref_errors(v, w, "Reputation", "factions", r).is_empty() {
            errors.push(format!("Reputation: unknown faction '{r}'."));
        }
        if !json::is_int(d) || json::to_int(d) == 0 || json::to_int(d).abs() > REP_DELTA_MAX {
            errors.push(format!("Reputation '{r}': delta must be a whole number from -{REP_DELTA_MAX} to {REP_DELTA_MAX}, not 0."));
        }
    }
    errors
}

/// QuestRules.step_binding_errors: one templated step (`full`: a base step, every required key present).
fn step_binding_errors(v: &View, w: &str, index: usize, kind: &str, step: &Dict, full: bool) -> Vec<String> {
    let mut errors = Vec::new();
    let label = format!("Quest step {} ({kind})", index + 1);
    let keys = binding_keys(kind);
    let req = keys.first().copied().unwrap_or("");
    if full && json::s(step, req).trim().is_empty() {
        errors.push(format!("{label} needs \"{req}\"."));
    } else if step.contains_key(req) && json::s(step, req).trim().is_empty() {
        errors.push(format!("{label}: \"{req}\" can't be emptied."));
    }
    if !full && step.get("text").is_some_and(|t| !t.is_string() || gstr(t).trim().is_empty()) {
        errors.push(format!("{label}: \"text\" can't be emptied."));
    }
    for key in keys {
        let Some(x) = step.get(*key) else { continue };
        let s = gstr(x);
        match *key {
            "location" if !s.is_empty() => errors.extend(ref_errors(v, w, &format!("{label} location"), "locations", &s)),
            "encounter" if !s.is_empty() => errors.extend(ref_errors(v, w, &format!("{label} encounter"), "encounters", &s)),
            "flag" if !x.is_string() || !flag_key_ok(&s) => errors.push(format!("{label}: flag key '{s}' must be non-empty [a-z0-9_.].")),
            "value" if !flag_value_ok(x) => errors.push(format!("{label}: value must be a bool, a whole number or a String.")),
            _ => {}
        }
    }
    errors
}

/// QuestRules.cycle_errors: `requires` may not lead back to the quest.
fn cycle_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let own = qid(w, &json::s(e, "id"));
    let mut seen: Vec<String> = Vec::new();
    let mut queue: std::collections::VecDeque<String> = json::sget(e, "requires").into();
    let mut guard = 0;
    while let Some(r) = queue.pop_front() {
        guard += 1;
        if guard > 512 {
            break;
        }
        let hit = v.worlds.resolve(w, "quests", &r);
        if hit.is_empty() {
            continue;
        }
        let hit_qid = json::s(hit, "_qid");
        if hit_qid == own || json::s(hit, "override") == own {
            return vec![format!("Requires: the quest chain is circular (it leads back to '{}').", json::s(e, "id"))];
        }
        if seen.contains(&hit_qid) {
            continue;
        }
        seen.push(hit_qid);
        queue.extend(json::sget(hit, "requires"));
    }
    Vec::new()
}

/// QuestRules.links_errors: {follows, leads_to, related_*: [refs]}; no quest link names the quest itself.
fn links_errors(v: &View, w: &str, e: &Dict, raw: &Value, label: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(links) = raw.as_object() else {
        errors.push(format!("{label} must be an object {{follows, leads_to, related_*}}."));
        return errors;
    };
    let id = json::s(e, "id");
    let own = qid(w, &id);
    for (k, list) in links {
        let Some(&(_, ref_kind)) = LINK_KINDS.iter().find(|(name, _)| name == k) else {
            errors.push(format!("{label}: unknown key '{k}' (allowed: {}).", LINK_KINDS.iter().map(|l| l.0).collect::<Vec<_>>().join(", ")));
            continue;
        };
        let Some(list) = list.as_array() else {
            errors.push(format!("{label} {k} must be a list of refs."));
            continue;
        };
        let mut seen: Vec<String> = Vec::new();
        for x in list {
            let r = gstr(x);
            if seen.contains(&r) {
                errors.push(format!("{label} {k} lists '{r}' twice."));
            }
            seen.push(r.clone());
            errors.extend(ref_errors(v, w, &format!("{label} {k}"), ref_kind, &r));
            if ref_kind == "quests" && (r == id || r == own || json::s(v.worlds.resolve(w, "quests", &r), "_qid") == own) {
                errors.push(format!("{label} {k}: a quest can't name itself."));
            }
        }
    }
    errors
}

fn loot_pool_errors(v: &View, w: &str, raw: &Value, label: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(list) = raw.as_array() else {
        errors.push(format!("{label} must be a list of loot refs."));
        return errors;
    };
    if list.len() > MAX_POOL {
        errors.push(format!("{label} holds at most {MAX_POOL} entries (has {}).", list.len()));
    }
    let mut seen: Vec<String> = Vec::new();
    for x in list {
        let r = gstr(x);
        let hit = v.worlds.resolve(w, "loot", &r);
        let key = if hit.is_empty() { r.clone() } else { json::s(hit, "_qid") };
        if seen.contains(&key) {
            errors.push(format!("{label} lists '{r}' twice."));
        }
        seen.push(key);
        errors.extend(ref_errors(v, w, label, "loot", &r));
    }
    errors
}

fn picks_errors(raw: &Value, label: &str) -> Vec<String> {
    let x = json::normalize(raw);
    if !json::is_int(&x) || !(0..=MAX_PICKS).contains(&json::to_int(&x)) {
        vec![format!("{label} must be a whole number 0..{MAX_PICKS}.")]
    } else {
        Vec::new()
    }
}

fn variants_errors(v: &View, w: &str, e: &Dict, kinds: &[String]) -> Vec<String> {
    let mut errors = Vec::new();
    let list = json::normalize(json::get(e, "variants"));
    let list = json::arr(&list);
    if list.len() > MAX_VARIANTS {
        errors.push(format!("A quest has at most {MAX_VARIANTS} variants (has {}).", list.len()));
    }
    let mut ids: Vec<String> = Vec::new();
    for (i, raw) in list.iter().enumerate() {
        let Some(variant) = raw.as_object() else {
            errors.push(format!("Variant {} must be an object {{id, when_flags, unless_flags, set}}.", i + 1));
            continue;
        };
        let vid = json::s(variant, "id");
        let label = if vid.is_empty() { format!("Variant {}", i + 1) } else { format!("Variant '{vid}'") };
        if !json::id_ok(&vid) {
            errors.push(format!("{label}: id must be lowercase snake_case (a-z, 0-9, _)."));
        } else if ids.contains(&vid) {
            errors.push(format!("{label}: id is used twice in this quest."));
        }
        ids.push(vid);
        for key in variant.keys().filter(|k| !VARIANT_KEYS.contains(&k.as_str())) {
            errors.push(format!("{label}: unknown key '{key}' (allowed: {}).", VARIANT_KEYS.join(", ")));
        }
        let empty = Value::Array(Vec::new());
        errors.extend(condition_errors(variant.get("when_flags").unwrap_or(&empty), &format!("{label} when flag")));
        errors.extend(condition_errors(variant.get("unless_flags").unwrap_or(&empty), &format!("{label} unless flag")));
        if json::aget(variant, "when_flags").is_empty() && json::aget(variant, "unless_flags").is_empty() {
            errors.push(format!("{label} has no condition (an unconditional variant: edit the quest instead)."));
        }
        let Some(set) = variant.get("set").and_then(Value::as_object).filter(|s| !s.is_empty()) else {
            errors.push(format!("{label} needs a non-empty \"set\" object."));
            continue;
        };
        errors.extend(variant_set_errors(v, w, e, kinds, set, &label));
    }
    errors
}

fn variant_set_errors(v: &View, w: &str, e: &Dict, kinds: &[String], set: &Dict, label: &str) -> Vec<String> {
    let mut errors = Vec::new();
    for (k, value) in set {
        if !VARIANT_FIELDS.contains(&k.as_str()) {
            errors.push(format!("{label}: '{k}' can't be overridden (allowed: {}).", VARIANT_FIELDS.join(", ")));
            continue;
        }
        match k.as_str() {
            "giver" | "turn_in" => match value.as_str() {
                None => errors.push(format!("{label}: {k} must be a character ref or \"\".")),
                Some("") => {}
                Some(r) => errors.extend(ref_errors(v, w, &format!("{label} {k}"), "characters", r)),
            },
            "description" | "reward" if !value.is_string() => errors.push(format!("{label}: {k} must be text.")),
            "rewards_from" if !value.is_string() || v.get("quest_templates", &gstr(value)).is_empty() => {
                errors.push(format!("{label}: rewards_from names unknown quest template '{}'.", gstr(value)))
            }
            "available" if !value.is_boolean() => errors.push(format!("{label}: available must be true or false.")),
            "links" => errors.extend(links_errors(v, w, e, value, &format!("{label} links"))),
            "loot_pool" => errors.extend(loot_pool_errors(v, w, value, &format!("{label} loot pool"))),
            "loot_picks" => errors.extend(picks_errors(value, &format!("{label} loot picks"))),
            "steps" => errors.extend(variant_steps_errors(v, w, kinds, value, label)),
            _ => {}
        }
    }
    errors
}

fn variant_steps_errors(v: &View, w: &str, kinds: &[String], raw: &Value, label: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(steps) = raw.as_object() else {
        errors.push(format!("{label}: steps must be an object {{\"<index>\": {{partial step}}}}."));
        return errors;
    };
    if kinds.is_empty() {
        return errors;
    }
    for (s, partial) in steps {
        let idx = s.parse::<i64>().ok().filter(|i| *i >= 0 && (*i as usize) < kinds.len());
        let Some(idx) = idx else {
            errors.push(format!("{label}: step index '{s}' is out of range (0..{}).", kinds.len() - 1));
            continue;
        };
        let Some(partial) = partial.as_object() else {
            errors.push(format!("{label}: step {s} must be an object."));
            continue;
        };
        let kind = &kinds[idx as usize];
        let mut allowed = vec!["text"];
        allowed.extend(binding_keys(kind));
        for pk in partial.keys().filter(|pk| !allowed.contains(&pk.as_str())) {
            errors.push(format!("{label}: step {s} ({kind}) can't set '{pk}' (allowed: {}).", allowed.join(", ")));
        }
        errors.extend(step_binding_errors(v, w, idx as usize, kind, partial, false));
    }
    errors
}

/// QuestRules.loot_entry_errors: a world loot entry.
pub fn loot_entry_errors(v: &View, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let kind = json::s_or(e, "reward_kind", "item");
    if LOOT_KINDS.contains(&kind.as_str()) {
        let why = reward_ref_error(v, &kind, &json::s(e, "ref"));
        if !why.is_empty() {
            errors.push(format!("Reward: {why}."));
        }
    }
    let count = json::normalize(e.get("count").unwrap_or(&json::int(1)));
    if kind == "item" {
        if !json::is_int(&count) || !(1..=ITEM_COUNT_MAX).contains(&json::to_int(&count)) {
            errors.push(format!("Count must be 1..{ITEM_COUNT_MAX} for an item."));
        }
    } else if !json::is_int(&count) || json::to_int(&count) != 1 {
        errors.push(format!("Count must be 1 for a {kind}."));
    }
    errors.extend(condition_errors(e.get("requires_flags").unwrap_or(&Value::Array(Vec::new())), "Requires flags"));
    errors
}

/// QuestRules.link_warnings: one-sided chains (A leads_to B without B follows A, or the reverse).
pub fn link_warnings(v: &View, w: &str) -> Vec<String> {
    let mut out = Vec::new();
    for q in v.worlds.merged_entries(w, "quests") {
        let links = json::dget(q, "links");
        for (there, back_key) in [("leads_to", "follows"), ("follows", "leads_to")] {
            for r in json::sget(links, there) {
                let other = v.worlds.resolve(w, "quests", &r);
                if other.is_empty() {
                    continue;
                }
                let back = json::sget(json::dget(other, "links"), back_key).iter().any(|r2| json::s(v.worlds.resolve(w, "quests", r2), "_qid") == json::s(q, "_qid"));
                if !back {
                    out.push(format!("{} {there} {}, but {} does not list it in {back_key}.", json::s(q, "_qid"), json::s(other, "_qid"), json::s(other, "_qid")));
                }
            }
        }
    }
    out
}

/// QuestRules.reward_warnings: the worst case over the base and every variant against the soft reward caps.
pub fn reward_warnings(v: &View, w: &str) -> Vec<String> {
    let mut out = Vec::new();
    for q in v.worlds.merged_entries(w, "quests") {
        let template_id = json::s(q, "template");
        if template_id.is_empty() {
            continue;
        }
        let mut reward_sets: Vec<Dict> = vec![json::dget(v.get("quest_templates", &template_id), "rewards").clone()];
        let mut pools = json::sget(q, "loot_pool");
        let mut picks = if pools.is_empty() { 0 } else { json::i(q, "loot_picks", 1) };
        for variant in json::aget(q, "variants") {
            let set = json::dget(json::dict(variant), "set");
            if let Some(from) = set.get("rewards_from") {
                reward_sets.push(json::dget(v.get("quest_templates", &gstr(from)), "rewards").clone());
            }
            if let Some(pool) = set.get("loot_pool") {
                for r in json::strings(pool) {
                    if !pools.contains(&r) {
                        pools.push(r);
                    }
                }
            }
            if let Some(p) = set.get("loot_picks") {
                picks = picks.max(json::to_int(p));
            }
        }
        let (mut passive, mut skill, mut item) = (0, 0, 0);
        for r in &pools {
            let loot = v.worlds.resolve(w, "loot", r);
            if loot.is_empty() {
                continue;
            }
            match json::s_or(loot, "reward_kind", "item").as_str() {
                "item" => item = item.max(json::i(loot, "count", 1)),
                "passive" => passive = passive.max(1),
                "skill" => skill = skill.max(1),
                _ => {}
            }
        }
        let q_id = json::s(q, "_qid");
        for r in &reward_sets {
            let passives = json::aget(r, "passives").len() as i64 + passive;
            let skills = json::aget(r, "skills").len() as i64 + skill;
            let items = item_total(r) + item;
            if passive > 0 && passives > SOFT_MAX_PASSIVES {
                out.push(format!("{q_id}: {passives} passives with a pooled passive (soft cap {SOFT_MAX_PASSIVES})."));
            }
            if skill > 0 && skills > SOFT_MAX_SKILLS {
                out.push(format!("{q_id}: {skills} skills with a pooled skill (soft cap {SOFT_MAX_SKILLS})."));
            }
            if item > 0 && items > SOFT_MAX_ITEMS {
                out.push(format!("{q_id}: {items} items with a pooled item (soft cap {SOFT_MAX_ITEMS})."));
            }
        }
        if picks > SOFT_MAX_PICKS {
            out.push(format!("{q_id}: {picks} loot picks (soft cap {SOFT_MAX_PICKS})."));
        }
    }
    out
}

/// RepRules.key_warnings: rep keys in a world's own entries that name no faction it can see.
pub fn rep_key_warnings(v: &View, w: &str) -> Vec<String> {
    fn keys_in(raw: &Value, out: &mut Vec<String>) {
        match raw {
            Value::Object(d) => {
                for (k, x) in d {
                    match x.as_str() {
                        Some(s) if (k == "key" || k == "flag") && s.starts_with(REP_PREFIX) => {
                            if !out.iter().any(|o| o == s) {
                                out.push(s.to_string());
                            }
                        }
                        _ => keys_in(x, out),
                    }
                }
            }
            Value::Array(a) => a.iter().for_each(|x| keys_in(x, out)),
            _ => {}
        }
    }
    let resolves = |key: &str| {
        let parts: Vec<&str> = key[REP_PREFIX.len()..].split('.').collect();
        if parts.len() != 2 || !json::id_ok(parts[0]) || !json::id_ok(parts[1]) {
            return false;
        }
        let fq = format!("{}:{}", parts[0], parts[1]);
        ref_errors(v, w, "Reputation", "factions", &fq).is_empty() && !v.worlds.resolve(w, "factions", &fq).is_empty()
    };
    let mut out = Vec::new();
    for kind in super::world::KINDS {
        for e in v.worlds.own_entries(w, kind) {
            let mut found = Vec::new();
            keys_in(&Value::Object(e.clone()), &mut found);
            for k in found.iter().filter(|k| !resolves(k)) {
                out.push(format!("{}: Reputation key '{k}' names no faction of this world.", json::s(e, "_qid")));
            }
        }
    }
    out
}

