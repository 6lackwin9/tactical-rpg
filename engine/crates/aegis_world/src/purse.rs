//! PurseRules runtime (scripts/exploration/purse_rules.gd): an encounter's gold range and loot paid on a win.
//! The gold roll has its own RandomNumberGenerator seeded from ("<qid>|<time>|<wins>").hash().

use crate::data::Data;
use crate::json::{self, int, Dict};
use crate::quest;
use crate::state::GameState;
use aegis_rules::ghash::string_hash;
use aegis_rules::rng::GodotRng;
use serde_json::Value;

pub const GOLD_CAP: i64 = 500;

/// {gold_min, gold_max, loot, once}; None = no purse.
pub fn read(entry: &Dict) -> Option<(i64, i64, String, bool)> {
    let p = entry.get("purse")?.as_object()?;
    if p.is_empty() {
        return None;
    }
    let lo = json::i(p, "gold_min", 0);
    let once = match p.get("once") {
        Some(Value::Bool(b)) => *b,
        _ => true,
    };
    Some((lo, json::i(p, "gold_max", lo), json::s(p, "loot"), once))
}

pub fn seed_of(qid: &str, time: i64, paid: i64) -> u32 {
    string_hash(&format!("{qid}|{time}|{paid}"))
}

/// {qid, has_purse, pays, gold, loot} for a win now (pure).
pub fn roll(d: &Data, gs: &GameState, encounter_ref: &str) -> Dict {
    let enc = d.resolve(&gs.world_id, "encounters", encounter_ref);
    let qid = json::s(enc, "_qid");
    let mut out = json::obj([("qid", qid.clone().into()), ("has_purse", Value::Bool(false)), ("pays", Value::Bool(false)), ("gold", int(0)), ("loot", "".into())]);
    let Some((gmin, gmax, loot, once)) = read(enc).filter(|_| !enc.is_empty()) else { return out };
    out.insert("has_purse".into(), Value::Bool(true));
    let paid = gs.purses.get(&qid).map(json::to_int).unwrap_or(0);
    let pays = paid == 0 || !once;
    out.insert("pays".into(), Value::Bool(pays));
    if !pays {
        return out;
    }
    let lo = gmin.clamp(0, GOLD_CAP);
    let hi = gmax.clamp(lo, GOLD_CAP);
    let mut rng = GodotRng::from_i64(seed_of(&qid, gs.time, paid) as i64);
    out.insert("gold".into(), int(rng.randi_range(lo as i32, hi as i32) as i64));
    out.insert("loot".into(), loot_qid(d, gs, &json::s(enc, "_world"), &loot).into());
    out
}

fn loot_qid(d: &Data, gs: &GameState, world: &str, r: &str) -> String {
    if r.is_empty() {
        return String::new();
    }
    let loot = d.resolve(world, "loot", r);
    if loot.is_empty() || json::s_or(loot, "reward_kind", "item") != "item" {
        return String::new();
    }
    let lqid = json::s(loot, "_qid");
    if gs.loot_claimed.contains(&lqid) || !quest::all_hold(gs, json::get(loot, "requires_flags")) {
        return String::new();
    }
    lqid
}

/// Applies a win: {} without a purse, else {gold, loot, notes}.
pub fn pay(d: &Data, gs: &mut GameState, encounter_ref: &str) -> Dict {
    let r = roll(d, gs, encounter_ref);
    if !json::b(&r, "has_purse", false) {
        return Dict::new();
    }
    let qid = json::s(&r, "qid");
    let mut notes: Vec<String> = Vec::new();
    let wins = gs.purses.get(&qid).map(json::to_int).unwrap_or(0) + 1;
    gs.purses.insert(qid, int(wins));
    let mut out = json::obj([("gold", int(0)), ("loot", "".into()), ("notes", Value::Array(Vec::new()))]);
    if !json::b(&r, "pays", false) {
        return out;
    }
    let mut lqid = json::s(&r, "loot");
    let mut loot: Vec<Dict> = Vec::new();
    if !lqid.is_empty() {
        loot.push(d.resolve(&gs.world_id, "loot", &lqid).clone());
    }
    let rewards = json::obj([("gold", int(json::i(&r, "gold", 0)))]);
    let mut granted = quest::reward_apply(d, gs, &rewards, "", &loot);
    if !json::b(&granted, "ok", false) && !loot.is_empty() {
        notes.push(format!("Purse loot '{lqid}' skipped: {}", json::sget(&granted, "errors").join("; ")));
        loot.clear();
        lqid.clear();
        granted = quest::reward_apply(d, gs, &rewards, "", &loot);
    }
    notes.extend(json::sget(&granted, "notes"));
    if !json::b(&granted, "ok", false) {
        notes.extend(json::sget(&granted, "errors"));
        out.insert("notes".into(), json::strs(&notes));
        return out;
    }
    if !lqid.is_empty() {
        let why = quest::claim_loot(gs, &lqid);
        if !why.is_empty() {
            notes.push(why);
        }
    }
    out.insert("gold".into(), int(json::i(json::dget(&granted, "granted"), "gold", 0)));
    out.insert("loot".into(), lqid.into());
    out.insert("notes".into(), json::strs(&notes));
    out
}

/// gold when > 0, loot when set.
pub fn result_keys(purse: &Dict) -> Dict {
    let mut out = Dict::new();
    if json::i(purse, "gold", 0) > 0 {
        out.insert("gold".into(), int(json::i(purse, "gold", 0)));
    }
    let loot = json::s(purse, "loot");
    if !loot.is_empty() {
        out.insert("loot".into(), loot.into());
    }
    out
}
