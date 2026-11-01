//! QuestRules (runtime parts), QuestRunner and RewardGrant (scripts/quests/): flag conditions, the quest as the
//! flags make it now (variants), the stored quest state machine and rewards as a passive / skill / item source.

use crate::data::Data;
use crate::dialogue::rep;
use crate::progression;
use crate::terms;
use crate::state::{flag_key_ok, flag_value_ok, qid_ok, GameState, GOLD_MAX};
use aegis_core::json::{self, int, strs, Dict};
use serde_json::Value;

pub const MAX_OBJECTIVES: i64 = 8;
const VARIANT_FIELDS: [&str; 10] = ["giver", "turn_in", "description", "reward", "rewards_from", "steps", "links", "available", "loot_pool", "loot_picks"];
pub const ORDER_OPS: [&str; 4] = [">=", "<=", ">", "<"];

// ---- QuestRules ----------------------------------------------------------------------------------------------

/// The one evaluation path of every flag condition: `==` strict (a missing flag reads false), ordering ops on ints;
/// read keys (CAMPAIGN1 11.4) answer from the campaign (`GameState::read_flag`).
pub fn condition_met(gs: &GameState, cond: &Dict) -> bool {
    let want = json::normalize(cond.get("value").unwrap_or(&Value::Bool(true)));
    let key = json::s(cond, "key");
    let op = json::s_or(cond, "op", "==");
    match op.as_str() {
        "==" | "!=" => {
            let have = gs.read_flag(&key).unwrap_or(Value::Bool(false));
            let same = json::same(&have, &want);
            if op == "==" {
                same
            } else {
                !same
            }
        }
        ">=" | "<=" | ">" | "<" => {
            let raw = gs.read_flag(&key).unwrap_or(int(0));
            if !json::is_int(&raw) || !json::is_int(&want) {
                return false;
            }
            let (a, b) = (json::to_int(&raw), json::to_int(&want));
            match op.as_str() {
                ">=" => a >= b,
                "<=" => a <= b,
                ">" => a > b,
                _ => a < b,
            }
        }
        _ => false,
    }
}

/// Every condition of a list holds (normalised first).
pub fn all_hold(gs: &GameState, raw: &Value) -> bool {
    json::arr(&json::normalize(raw)).iter().all(|c| condition_met(gs, json::dict(c)))
}

pub fn any_holds(gs: &GameState, raw: &Value) -> bool {
    json::arr(&json::normalize(raw)).iter().any(|c| condition_met(gs, json::dict(c)))
}

/// add_flag: Ok(new int) or Err(why). Missing reads 0; clamped to lo / hi when ints, to the rep range for rep keys.
pub fn added_flag(gs: &GameState, key: &str, delta: i64, lo: &Value, hi: &Value) -> Result<i64, String> {
    let cur = gs.flag(key).cloned().unwrap_or(int(0));
    if !json::is_int(&cur) {
        return Err(format!("Flag '{key}' is not a number."));
    }
    let mut v = json::to_int(&cur) + delta;
    if json::is_int(lo) {
        v = v.max(json::to_int(lo));
    }
    if json::is_int(hi) {
        v = v.min(json::to_int(hi));
    }
    if rep::is_rep_key(key) {
        v = v.clamp(rep::MIN, rep::MAX);
    }
    Ok(v)
}

pub fn variant_applies(gs: &GameState, variant: &Dict) -> bool {
    json::aget(variant, "when_flags").iter().all(|c| condition_met(gs, json::dict(c)))
        && !json::aget(variant, "unless_flags").iter().any(|c| condition_met(gs, json::dict(c)))
}

/// The quest as the flags make it now: template objectives and rewards attached, applying variants merged in order,
/// `available` (default true), `applied_variants`, `turn_in` falling back to the giver.
pub fn resolve_quest(d: &Data, gs: &GameState, entry: &Dict) -> Dict {
    let mut out: Dict = Dict::new();
    for (k, v) in entry {
        if !k.starts_with('_') {
            out.insert(k.clone(), json::normalize(v));
        }
    }
    for key in ["_world", "_qid"] {
        if let Some(v) = entry.get(key) {
            out.insert(key.into(), v.clone());
        }
    }
    let template = d.quest_template(&json::s(&out, "template"));
    out.insert("objectives".into(), Value::Array(json::aget(template, "objectives").to_vec()));
    out.insert("rewards".into(), Value::Object(json::dget(template, "rewards").clone()));
    out.insert("available".into(), Value::Bool(true));
    let mut applied = Vec::new();
    for raw in json::aget(&out.clone(), "variants") {
        let v = json::dict(raw);
        if !variant_applies(gs, v) {
            continue;
        }
        applied.push(json::s(v, "id"));
        for (k, val) in json::dget(v, "set") {
            if !VARIANT_FIELDS.contains(&k.as_str()) {
                continue;
            }
            match k.as_str() {
                "steps" => {
                    let mut steps: Vec<Value> = json::aget(&out, "steps").to_vec();
                    for (idx, partial) in json::dict(val) {
                        if let Ok(i) = idx.parse::<i64>() {
                            if i >= 0 && (i as usize) < steps.len() {
                                let mut step = json::dict(&steps[i as usize]).clone();
                                for (pk, pv) in json::dict(partial) {
                                    step.insert(pk.clone(), pv.clone());
                                }
                                steps[i as usize] = Value::Object(step);
                            }
                        }
                    }
                    out.insert("steps".into(), Value::Array(steps));
                }
                "rewards_from" => {
                    let rewards = json::dget(d.quest_template(&json::gstr(val)), "rewards").clone();
                    out.insert("rewards".into(), Value::Object(rewards));
                }
                _ => {
                    out.insert(k.clone(), json::normalize(val));
                }
            }
        }
    }
    if json::s(&out, "turn_in").is_empty() {
        let giver = json::s(&out, "giver");
        out.insert("turn_in".into(), Value::String(giver));
    }
    out.insert("applied_variants".into(), strs(&applied));
    out
}

// ---- QuestRunner ---------------------------------------------------------------------------------------------

/// Is a quest of `quest_world` readable from `current` (lineage, connected, or connecting to it; one hop)?
pub fn world_visible(d: &Data, current: &str, quest_world: &str) -> bool {
    if d.can_reach_world(current, quest_world) {
        return true;
    }
    let here = d.lineage(current);
    d.lineage(quest_world).iter().any(|w| json::sget(d.manifest(w), "connects").iter().any(|t| here.contains(t)))
}

fn entry_from<'a>(d: &'a Data, gs: &GameState, world: &str, r: &str) -> &'a Dict {
    let found = d.resolve(world, "quests", r);
    if found.is_empty() || !world_visible(d, &gs.world_id, &json::s_or(found, "_world", world)) {
        return json::empty();
    }
    found
}

fn entry<'a>(d: &'a Data, gs: &GameState, r: &str) -> &'a Dict {
    entry_from(d, gs, &gs.world_id, r)
}

fn runnable(d: &Data, e: &Dict) -> bool {
    let t = json::s(e, "template");
    !e.is_empty() && !t.is_empty() && d.has_quest_template(&t)
}

/// The base world for refs written in a quest: the party's world for its own lineage, else the quest's own.
pub fn world_of(d: &Data, gs: &GameState, e: &Dict) -> String {
    let w = json::s_or(e, "_world", &gs.world_id);
    if d.lineage(&gs.world_id).contains(&w) {
        gs.world_id.clone()
    } else {
        w
    }
}

fn scope<'a>(d: &'a Data, gs: &GameState) -> Vec<&'a Dict> {
    let mut out = d.merged_entries(&gs.world_id, "quests");
    let mut seen: Vec<String> = out.iter().map(|e| json::s(e, "_qid")).collect();
    for qid in gs.quests.keys() {
        if seen.contains(qid) {
            continue;
        }
        let extra = entry(d, gs, qid);
        if runnable(d, extra) && &json::s(extra, "_qid") == qid {
            out.push(extra);
            seen.push(qid.clone());
        }
    }
    out
}

fn stored<'a>(gs: &'a GameState, qid: &str) -> &'a Dict {
    gs.quests.get(qid).map(json::dict).unwrap_or_else(|| json::empty())
}

pub fn ready_flag(qid: &str) -> String {
    format!("quest.{}.ready", qid.replace(':', "."))
}

fn count(q: &Dict) -> i64 {
    json::aget(q, "objectives").len() as i64
}

/// invalid | completed | blocked | ready | active | unavailable | available | locked.
pub fn status(d: &Data, gs: &GameState, r: &str) -> String {
    let e = entry(d, gs, r);
    if !runnable(d, e) {
        return "invalid".into();
    }
    let st = json::s(stored(gs, &json::s(e, "_qid")), "state");
    if st == "completed" {
        return st;
    }
    let available = json::b(&resolve_quest(d, gs, e), "available", true);
    if !st.is_empty() {
        return if available { st } else { "blocked".into() };
    }
    if !available {
        return "unavailable".into();
    }
    let home = world_of(d, gs, e);
    for req in json::sget(e, "requires") {
        let other = entry_from(d, gs, &home, &req);
        if other.is_empty() || json::s(stored(gs, &json::s(other, "_qid")), "state") != "completed" {
            return "locked".into();
        }
    }
    if !json::aget(e, "requires_flags").iter().all(|c| condition_met(gs, json::dict(c))) {
        return "locked".into();
    }
    "available".into()
}

pub fn resolved(d: &Data, gs: &GameState, r: &str) -> Dict {
    let e = entry(d, gs, r);
    if runnable(d, e) {
        resolve_quest(d, gs, e)
    } else {
        Dict::new()
    }
}

pub fn quests_with(d: &Data, gs: &GameState, wanted: &str) -> Vec<String> {
    scope(d, gs).into_iter().map(|e| json::s(e, "_qid")).filter(|q| status(d, gs, q) == wanted).collect()
}

/// {index, kind, text, target, value} of the current objective; {} unless active.
pub fn current_objective(d: &Data, gs: &GameState, r: &str) -> Dict {
    if status(d, gs, r) != "active" {
        return Dict::new();
    }
    let e = entry(d, gs, r);
    let home = world_of(d, gs, e);
    let q = resolve_quest(d, gs, e);
    let index = json::i(stored(gs, &json::s(e, "_qid")), "objective", 0);
    let objectives = json::aget(&q, "objectives");
    if index < 0 || index as usize >= objectives.len() {
        return Dict::new();
    }
    let kind = json::s(json::dict(&objectives[index as usize]), "kind");
    let steps = json::aget(&q, "steps");
    let step = steps.get(index as usize).map(json::dict).unwrap_or_else(|| json::empty());
    let target = match kind.as_str() {
        "reach_location" => json::s(d.resolve(&home, "locations", &json::s(step, "location")), "_qid"),
        "win_encounter" => json::s(d.resolve(&home, "encounters", &json::s(step, "encounter")), "_qid"),
        "flag" => json::s(step, "flag"),
        _ => String::new(),
    };
    json::obj([
        ("index", int(index)),
        ("kind", kind.into()),
        ("text", json::s(step, "text").into()),
        ("target", target.into()),
        ("value", step.get("value").cloned().unwrap_or(Value::Bool(true))),
    ])
}

/// Loot qids offered now: resolved pool minus claimed minus entries whose requires_flags fail.
pub fn offers(d: &Data, gs: &GameState, r: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let q = resolved(d, gs, r);
    let home = world_of(d, gs, &q);
    for lref in json::sget(&q, "loot_pool") {
        let loot = d.resolve(&home, "loot", &lref);
        if loot.is_empty() {
            continue;
        }
        let lqid = json::s(loot, "_qid");
        if out.contains(&lqid) || gs.loot_claimed.contains(&lqid) {
            continue;
        }
        if all_hold(gs, json::get(loot, "requires_flags")) {
            out.push(lqid);
        }
    }
    out
}

pub fn picks_allowed(d: &Data, gs: &GameState, r: &str) -> i64 {
    json::i(&resolved(d, gs, r), "loot_picks", 0).min(offers(d, gs, r).len() as i64)
}

pub fn start(d: &Data, gs: &mut GameState, r: &str) -> Vec<String> {
    let st = status(d, gs, r);
    if st != "available" {
        return vec![format!("Quest '{r}' can't start (it is {st}).")];
    }
    let qid = json::s(entry(d, gs, r), "_qid");
    gs.quests.insert(qid, Value::Object(json::obj([("state", "active".into()), ("objective", int(0))])));
    refresh(d, gs);
    Vec::new()
}

pub fn abandon(d: &Data, gs: &mut GameState, r: &str) -> Vec<String> {
    let st = status(d, gs, r);
    if !["active", "ready", "blocked"].contains(&st.as_str()) {
        return vec![format!("Quest '{r}' can't be abandoned (it is {st}).")];
    }
    let qid = json::s(entry(d, gs, r), "_qid");
    gs.quests.shift_remove(&qid);
    gs.clear_flag(&ready_flag(&qid));
    Vec::new()
}

pub fn on_location_reached(d: &Data, gs: &mut GameState, location_ref: &str) -> Vec<String> {
    let target = json::s(d.resolve(&gs.world_id, "locations", location_ref), "_qid");
    on_event(d, gs, "reach_location", &target)
}

pub fn on_encounter_won(d: &Data, gs: &mut GameState, encounter_ref: &str) -> Vec<String> {
    let target = json::s(d.resolve(&gs.world_id, "encounters", encounter_ref), "_qid");
    on_event(d, gs, "win_encounter", &target)
}

fn on_event(d: &Data, gs: &mut GameState, kind: &str, target: &str) -> Vec<String> {
    let mut advanced = Vec::new();
    if target.is_empty() {
        return advanced;
    }
    for qid in quests_with(d, gs, "active") {
        let cur = current_objective(d, gs, &qid);
        if json::s(&cur, "kind") == kind && json::s(&cur, "target") == target {
            advance(gs, &qid);
            advanced.push(qid);
        }
    }
    refresh(d, gs);
    advanced
}

fn advance(gs: &mut GameState, qid: &str) {
    let next = json::i(stored(gs, qid), "objective", 0) + 1;
    gs.quests.insert(qid.into(), Value::Object(json::obj([("state", "active".into()), ("objective", int(next))])));
}

/// GameState.set_flag then refresh ("" = set).
pub fn set_flag(d: &Data, gs: &mut GameState, key: &str, value: Value) -> String {
    let err = gs.set_flag(key, value);
    if err.is_empty() {
        refresh(d, gs);
    }
    err
}

pub fn add_flag(d: &Data, gs: &mut GameState, key: &str, delta: i64, lo: &Value, hi: &Value) -> String {
    match added_flag(gs, key, delta, lo, hi) {
        Err(e) => e,
        Ok(v) => set_flag(d, gs, key, int(v)),
    }
}

/// Until nothing changes: auto_start quests start, met flag objectives advance, finished quests turn ready.
pub fn refresh(d: &Data, gs: &mut GameState) -> Vec<String> {
    let mut changed: Vec<String> = Vec::new();
    let quests: Vec<Dict> = scope(d, gs).into_iter().cloned().collect();
    let passes = quests.len() as i64 * (MAX_OBJECTIVES + 1) + 1;
    for _ in 0..passes {
        let mut any = false;
        for e in &quests {
            let qid = json::s(e, "_qid");
            let mut st = status(d, gs, &qid);
            let touch = |changed: &mut Vec<String>| {
                if !changed.contains(&qid) {
                    changed.push(qid.clone());
                }
            };
            if st == "available" && json::b(e, "auto_start", false) {
                gs.quests.insert(qid.clone(), Value::Object(json::obj([("state", "active".into()), ("objective", int(0))])));
                st = "active".into();
                any = true;
                touch(&mut changed);
            }
            if st != "active" {
                continue;
            }
            let index = json::i(stored(gs, &qid), "objective", 0);
            if index >= count(&resolve_quest(d, gs, e)) {
                gs.quests.insert(qid.clone(), Value::Object(json::obj([("state", "ready".into()), ("objective", int(index))])));
                gs.set_flag(&ready_flag(&qid), Value::Bool(true));
                any = true;
                touch(&mut changed);
                continue;
            }
            let cur = current_objective(d, gs, &qid);
            if json::s(&cur, "kind") == "flag" {
                let cond = json::obj([("key", cur["target"].clone()), ("value", cur["value"].clone())]);
                if condition_met(gs, &cond) {
                    advance(gs, &qid);
                    any = true;
                    touch(&mut changed);
                }
            }
        }
        if !any {
            break;
        }
    }
    changed
}

/// Completes a ready quest: rewards and picked loot, claims, complete_flags, reputation, then one refresh.
/// {ok, errors, granted, notes}.
pub fn complete(d: &Data, gs: &mut GameState, r: &str, recipient: &str, picks: &[String]) -> Dict {
    let fail = |errors: Vec<String>| {
        let mut out = json::ok_result(false, &errors);
        out.insert("granted".into(), Value::Object(Dict::new()));
        out.insert("notes".into(), Value::Array(Vec::new()));
        out
    };
    if status(d, gs, r) != "ready" {
        return fail(vec!["quest is not ready".into()]);
    }
    let e = entry(d, gs, r).clone();
    let qid = json::s(&e, "_qid");
    let q = resolve_quest(d, gs, &e);
    let offered = offers(d, gs, r);
    let allowed = picks_allowed(d, gs, r);
    let mut errors = Vec::new();
    let mut loot: Vec<Dict> = Vec::new();
    let mut seen: Vec<&String> = Vec::new();
    if picks.len() as i64 > allowed {
        errors.push(format!("At most {allowed} loot picks are allowed (got {}).", picks.len()));
    }
    let home = world_of(d, gs, &e);
    for p in picks {
        if seen.contains(&p) {
            errors.push(format!("Loot '{p}' is picked twice."));
        }
        seen.push(p);
        if !offered.contains(p) {
            errors.push(format!("Loot '{p}' is not offered by this quest."));
        } else {
            loot.push(d.resolve(&home, "loot", p).clone());
        }
    }
    let mut flags_to_set: Vec<(String, Value)> = Vec::new();
    for c in json::aget(&q, "complete_flags") {
        let cond = json::dict(c);
        let key = json::s(cond, "key");
        let value = cond.get("value").cloned().unwrap_or(Value::Bool(true));
        if !flag_key_ok(&key) || !flag_value_ok(&value) {
            errors.push(format!("Complete flag '{key}' is invalid."));
        }
        flags_to_set.push((key, value));
    }
    let mut rep_keys: Vec<(String, String, i64)> = Vec::new();
    let rep_raw = json::normalize(json::get(&e, "reputation"));
    for (fref, delta) in json::dict(&rep_raw) {
        let faction = d.resolve(&json::s_or(&e, "_world", &gs.world_id), "factions", fref);
        if faction.is_empty() || !json::is_int(delta) {
            continue;
        }
        let fq = json::s(faction, "_qid");
        let rkey = rep::key(&fq);
        let mut after = gs.flag(&rkey).cloned().unwrap_or(int(0));
        for (k, v) in &flags_to_set {
            if *k == rkey {
                after = v.clone();
            }
        }
        if !json::is_int(&after) {
            errors.push(format!("Flag '{rkey}' is not a number."));
        }
        rep_keys.push((fq, rkey, json::to_int(delta)));
    }
    if !errors.is_empty() {
        return fail(errors);
    }
    let result = reward_apply(d, gs, json::dget(&q, "rewards"), recipient, &loot);
    if !json::b(&result, "ok", false) {
        return fail(json::sget(&result, "errors"));
    }
    for p in picks {
        gs.loot_claimed.push(p.clone());
    }
    for (k, v) in &flags_to_set {
        gs.set_flag(k, v.clone());
    }
    let mut applied = Dict::new();
    for (fq, rkey, delta) in &rep_keys {
        let before = gs.flag(rkey).map(json::to_int).unwrap_or(0);
        if let Ok(v) = added_flag(gs, rkey, *delta, &Value::Null, &Value::Null) {
            gs.set_flag(rkey, int(v));
            let prev = applied.get(fq).map(json::to_int).unwrap_or(0);
            applied.insert(fq.clone(), int(prev + v - before));
        }
    }
    gs.quests.insert(qid, Value::Object(json::obj([("state", "completed".into()), ("objective", int(count(&q)))])));
    refresh(d, gs);
    let mut granted = json::dget(&result, "granted").clone();
    granted.insert("reputation".into(), Value::Object(applied));
    let mut out = json::ok_result(true, &[]);
    out.insert("granted".into(), Value::Object(granted));
    out.insert("notes".into(), result.get("notes").cloned().unwrap_or(Value::Array(Vec::new())));
    out
}

// ---- RewardGrant ---------------------------------------------------------------------------------------------

/// Marks world loot `qid` claimed, keeping loot_claimed sorted. "" or why not.
pub fn claim_loot(gs: &mut GameState, qid: &str) -> String {
    if !qid_ok(qid) {
        return format!("Loot '{qid}' must be a qualified id world:id.");
    }
    if gs.loot_claimed.iter().any(|x| x == qid) {
        return format!("Loot '{qid}' is already claimed.");
    }
    gs.loot_claimed.push(qid.into());
    gs.loot_claimed.sort();
    String::new()
}

/// `rewards` {xp, items, passives, skills, gold} plus resolved loot entries. {ok, errors, granted, notes}.
/// Content validation of the reward block (QuestRules.rewards_errors) is not ported: shipped data is valid.
pub fn reward_apply(d: &Data, gs: &mut GameState, rewards: &Dict, recipient_id: &str, loot: &[Dict]) -> Dict {
    let r = json::normalize_dict(rewards);
    let mut errors = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let gold_raw = r.get("gold").cloned().unwrap_or(int(0));
    if !json::is_int(&gold_raw) || json::to_int(&gold_raw) < 0 || json::to_int(&gold_raw) > GOLD_MAX {
        errors.push(format!("Rewards gold must be a whole number 0..{GOLD_MAX}."));
    }
    let gold = json::to_int(&gold_raw);
    let xp = json::i(&r, "xp", 0);
    let mut items = r.get("items").map(json::ints).unwrap_or_default();
    let mut passives = json::sget(&r, "passives");
    let mut skills = json::sget(&r, "skills");
    let mut loot_ids = Vec::new();
    for e in loot {
        let lqid = json::s_or(e, "_qid", &json::s(e, "id"));
        loot_ids.push(lqid);
        let rf = json::s(e, "ref");
        match json::s_or(e, "reward_kind", "item").as_str() {
            "item" => *items.entry(rf).or_insert(0) += json::i(e, "count", 1),
            "passive" => passives.push(rf),
            "skill" => skills.push(rf),
            _ => {}
        }
    }
    let mut recipient: Option<usize> = None;
    if !passives.is_empty() || !skills.is_empty() {
        if recipient_id.is_empty() {
            match gs.party.active_indexes().first() {
                None => errors.push("No active party member to receive the passive / skill rewards.".into()),
                Some(i) => recipient = Some(*i),
            }
        } else {
            recipient = gs.party.members.iter().position(|m| m.id == recipient_id);
            if recipient.is_none() {
                errors.push(format!("Recipient '{recipient_id}' is not a party member."));
            }
        }
    } else if !recipient_id.is_empty() && gs.party.member(recipient_id).is_none() {
        notes.push(format!("Recipient '{recipient_id}' is not a party member (ignored: nothing here needs a recipient)."));
    }
    let recipient_name = recipient.map(|i| gs.party.members[i].id.clone()).unwrap_or_default();
    let mut granted = json::obj([
        ("xp", int(0)),
        ("items", Value::Object(Dict::new())),
        ("passives", Value::Array(Vec::new())),
        ("skills", Value::Array(Vec::new())),
        ("gold", int(0)),
        ("loot", strs(&loot_ids)),
        ("recipient", recipient_name.into()),
    ]);
    let done = |ok: bool, errors: &[String], granted: Dict, notes: &[String]| {
        let mut out = json::ok_result(ok, errors);
        out.insert("granted".into(), Value::Object(granted));
        out.insert("notes".into(), strs(notes));
        out
    };
    if !errors.is_empty() {
        return done(false, &errors, granted, &notes);
    }
    if gold > 0 {
        let added = gs.party.add_gold(gold);
        granted.insert("gold".into(), int(added));
        if added < gold {
            notes.push(terms::currency_capped(d, &gs.world_id, gold - added));
        }
    }
    if xp > 0 {
        for i in gs.party.active_indexes() {
            progression::award(d, &mut gs.party.members[i], xp);
        }
        granted.insert("xp".into(), int(xp));
    }
    let mut items_out = Dict::new();
    for (id, n) in &items {
        gs.party.add_to_stash(id, *n);
        items_out.insert(id.clone(), int(*n));
    }
    granted.insert("items".into(), Value::Object(items_out));
    let mut got_p = Vec::new();
    let mut got_s = Vec::new();
    if let Some(i) = recipient {
        for p in passives {
            let m = &mut gs.party.members[i];
            if m.passives.contains(&p) {
                notes.push(format!("{} already owns passive '{p}'.", m.id));
            } else {
                m.passives.push(p.clone());
                got_p.push(p);
            }
        }
        for s in skills {
            let owned = gs.party.members[i].owned_families(d, &gs.world_id);
            let m = &mut gs.party.members[i];
            if owned.contains(&s) || m.learned_skills.contains(&s) {
                notes.push(format!("{} already owns skill '{s}'.", m.id));
            } else {
                m.learned_skills.push(s.clone());
                got_s.push(s);
            }
        }
    }
    granted.insert("passives".into(), strs(&got_p));
    granted.insert("skills".into(), strs(&got_s));
    done(true, &[], granted, &notes)
}

/// Removes `items` and `gold` from the party (dialogue `take`); checks every holding first.
pub fn take(d: &Data, gs: &mut GameState, items: &Value, gold: i64) -> Vec<String> {
    let mut errors = Vec::new();
    let wanted = json::ints(items);
    for (id, n) in &wanted {
        if *n < 1 {
            errors.push(format!("Take count of '{id}' must be >= 1."));
        } else if gs.party.stash.get(id).copied().unwrap_or(0) < *n {
            errors.push(format!("Not enough {}.", d.item_name(id)));
        }
    }
    if gold < 0 {
        errors.push("Take gold must be >= 0.".into());
    } else if gs.party.gold < gold {
        errors.push(terms::not_enough_currency(d, &gs.world_id));
    }
    if errors.is_empty() {
        for (id, n) in &wanted {
            gs.party.remove_from_stash(id, *n);
        }
        gs.party.gold -= gold;
    }
    errors
}
