//! BattleBridge (scripts/exploration/battle_bridge.gd): the shared way out of a battle into the game state.
//! won: XP to every active member, the quest hook, the encounter purse; lost / fled change nothing here (the
//! runners apply the defeat and flee rules). Building the battle itself (GameEncounter) lives with the session.

use crate::data::Data;
use crate::json::{self, Dict};
use crate::state::GameState;
use crate::{progression, purse, quest};
use serde_json::Value;

pub const OUTCOMES: [&str; 3] = ["won", "lost", "fled"];

/// (level, tier) of the encounter's enemies, reinforcements and phase waves, seen from `world_id`.
pub fn enemy_levels(d: &Data, world_id: &str, encounter_qid: &str) -> Vec<(i64, String)> {
    let enc = d.resolve(world_id, "encounters", encounter_qid);
    let mut refs = json::sget(enc, "enemies");
    refs.extend(json::sget(enc, "wave_units"));
    for phase in json::aget(enc, "phases") {
        for fx in json::aget(json::dict(phase), "effects") {
            let fx = json::dict(fx);
            if json::s(fx, "type") == "wave" {
                refs.extend(json::sget(fx, "units"));
            }
        }
    }
    let mut out = Vec::new();
    for r in refs {
        let entry = d.character_unit_entry(d.resolve(world_id, "characters", &r));
        if entry.is_empty() {
            continue;
        }
        out.push((json::i(&entry, "level", 1), json::s_or(&entry, "tier", "standard")));
    }
    out
}

/// {ok, errors, xp: member id -> award, advanced, purse?}.
pub fn apply(d: &Data, gs: &mut GameState, encounter_qid: &str, outcome: &str) -> Dict {
    let done = |ok: bool, errors: &[String], xp: Dict, advanced: &[String]| {
        let mut out = json::ok_result(ok, errors);
        out.insert("xp".into(), Value::Object(xp));
        out.insert("advanced".into(), json::strs(advanced));
        out
    };
    if !OUTCOMES.contains(&outcome) {
        return done(false, &[format!("Unknown battle outcome '{outcome}' (use won, lost or fled).")], Dict::new(), &[]);
    }
    if outcome != "won" {
        return done(true, &[], Dict::new(), &[]);
    }
    let enemies = enemy_levels(d, &gs.world_id, encounter_qid);
    let xp = progression::award_encounter(d, &mut gs.party, &enemies);
    let advanced = quest::on_encounter_won(d, gs, encounter_qid);
    let mut out = done(true, &[], xp, &advanced);
    let paid = purse::pay(d, gs, encounter_qid);
    if !paid.is_empty() {
        out.insert("purse".into(), Value::Object(paid));
    }
    out
}
