//! PurseRules runtime (scripts/exploration/purse_rules.gd): an encounter's gold range. The gold roll has its own
//! RandomNumberGenerator seeded from ("<qid>|<time>|<paid>").hash(). REWARD1 retired `purse.loot` (key loot is
//! `key_loot` on the encounter).

use crate::battle::RosterFoe;
use crate::state::GameState;
use aegis_core::ghash::string_hash;
use aegis_core::json::{self, int, Dict};
use aegis_core::rng::GodotRng;
use serde_json::Value;

pub const GOLD_CAP: i64 = 500;

/// (gold_min, gold_max, once); None = no purse.
pub fn read(entry: &Dict) -> Option<(i64, i64, bool)> {
    let p = entry.get("purse")?.as_object()?;
    if p.is_empty() {
        return None;
    }
    let lo = json::i(p, "gold_min", 0);
    let once = match p.get("once") {
        Some(Value::Bool(b)) => *b,
        _ => true,
    };
    Some((lo, json::i(p, "gold_max", lo), once))
}

pub fn seed_of(qid: &str, time: i64, paid: i64) -> u32 {
    string_hash(&format!("{qid}|{time}|{paid}"))
}

/// REWARD1 3.3-3.5: the gold share the units `defeated` in this fight (roster order) earn from the encounter's purse.
/// G is rolled as PURSE1 (`seed_of(qid, time, paid)`); the share is G when every roster unit's weight is paid, else
/// floor(G x paid weight / roster weight). A `once` purse pays each roster key once per save (`reward_paid`). Bumps
/// `purses[qid]` when anything was defeated (paid or not), so the next roll differs.
pub fn share(gs: &mut GameState, enc: &Dict, roster: &[RosterFoe], defeated: &[bool]) -> i64 {
    let Some((gmin, gmax, once)) = read(enc) else { return 0 };
    if !defeated.iter().any(|x| *x) {
        return 0;
    }
    let qid = json::s(enc, "_qid");
    let paid = gs.purses.get(&qid).map(json::to_int).unwrap_or(0);
    gs.purses.insert(qid.clone(), int(paid + 1));
    let before: Vec<String> = json::aget(&gs.reward_paid, &qid).iter().map(json::gstr).collect();
    let newly: Vec<&RosterFoe> = roster.iter().zip(defeated).filter(|(f, d)| **d && !(once && before.contains(&f.key))).map(|(f, _)| f).collect();
    if newly.is_empty() {
        return 0;
    }
    let total: i64 = roster.iter().map(|f| f.weight).sum();
    let part: i64 = newly.iter().map(|f| f.weight).sum();
    if once {
        let mut keys = json::aget(&gs.reward_paid, &qid).to_vec();
        keys.extend(newly.iter().map(|f| Value::from(f.key.clone())));
        gs.reward_paid.insert(qid.clone(), Value::Array(keys));
    }
    let lo = gmin.clamp(0, GOLD_CAP);
    let hi = gmax.clamp(lo, GOLD_CAP);
    let g = GodotRng::from_i64(seed_of(&qid, gs.time, paid) as i64).randi_range(lo as i32, hi as i32) as i64;
    if part >= total {
        g
    } else {
        g * part / total.max(1)
    }
}
