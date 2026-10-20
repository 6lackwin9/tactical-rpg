//! Balance sim world encounters (tools/sim/sim_world.gd, BAL4-ATTR): the Ashlands story fights in quest order, read
//! from the world data, with story characters mapped to their base catalog units. BAL5-SIM K1: any world's
//! encounters; a character with extras (named items, passives, disciplines, tuning) plays as a sandbox copy
//! simw_<world>_<id> of its base unit that spawns with those extras, as in a world battle.

use crate::runner::{Def, Runner};
use crate::scenarios::{catalog_ids, Enc};
use aegis_core::json::{self, Dict};
use emberwake_world::Data;
use serde_json::{json, Map, Value};

pub const WORLD: &str = "ashlands";
/// The flag of the "bar the gate" choice.
pub const STEEL_FLAG: &str = "ashlands.steel_to_gate";
/// MORALE1-S: morale fields copied into the base of full encounters.
const MORALE_KEYS: [&str; 5] = ["morale", "morale_break", "morale_leader", "morale_groups", "morale_text"];

/// The main story quests in order: the auto-start root with no requirements, then every quest that (transitively)
/// requires it, in file order.
pub fn story_quests(d: &Data) -> Vec<Dict> {
    let mut in_story: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for q in d.own_entries(WORLD, "quests") {
        let reqs = json::sget(q, "requires");
        let root = reqs.is_empty() && json::b(q, "auto_start", false);
        let chained = reqs.iter().any(|r| in_story.contains(r));
        if root || chained {
            in_story.push(json::s(q, "id"));
            out.push(q.clone());
        }
    }
    out
}

/// Scripted route encounters (any world) gated on `flag`.
pub fn scripted_encounters_for_flag(d: &Data, flag: &str) -> Vec<String> {
    let mut out = Vec::new();
    for w in d.world_ids() {
        for route in d.own_entries(&w, "routes") {
            for sc in json::aget(route, "scripted") {
                let sc = json::dict(sc);
                for cond in json::aget(sc, "requires_flags") {
                    if json::s(json::dict(cond), "key") == flag {
                        // WorldCatalog.split_qid(ref)[-1]: the part after the first colon.
                        if let Some((_, id)) = json::s(sc, "encounter").split_once(':') {
                            out.push(id.to_string());
                        }
                    }
                }
            }
        }
    }
    out
}

/// The step encounter of quest `q` at step `i`, with the steel_to_gate variant applied.
fn step_encounter(q: &Dict, i: usize, steel_to_gate: bool) -> String {
    let steps = json::aget(q, "steps");
    let mut enc = json::s(json::dict(&steps[i]), "encounter");
    for v in json::aget(q, "variants") {
        let v = json::dict(v);
        let needs = json::aget(v, "when_flags").iter().any(|c| json::s(json::dict(c), "key") == STEEL_FLAG);
        if needs && steel_to_gate {
            let over = json::dget(json::dget(json::dget(v, "set"), "steps"), &i.to_string());
            if over.contains_key("encounter") {
                enc = json::s(over, "encounter");
            }
        }
    }
    enc
}

/// One story fight: its quest, its encounter, whether a scripted route raid gives it.
pub struct StoryFight {
    pub quest: String,
    pub encounter: String,
    pub scripted: bool,
}

/// Every story fight in order.
pub fn story_fights(d: &Data, steel_to_gate: bool) -> Vec<StoryFight> {
    let mut out = Vec::new();
    for q in story_quests(d) {
        let steps = json::aget(&q, "steps");
        for i in 0..steps.len() {
            let step = json::dict(&steps[i]);
            let enc = step_encounter(&q, i, steel_to_gate);
            if !enc.is_empty() {
                out.push(StoryFight { quest: json::s(&q, "id"), encounter: enc, scripted: false });
            }
            let flag = json::s(step, "flag");
            if !flag.is_empty() {
                for raid in scripted_encounters_for_flag(d, &flag) {
                    out.push(StoryFight { quest: json::s(&q, "id"), encounter: raid, scripted: true });
                }
            }
        }
    }
    out
}

/// WorldFactory.divergence is not empty: the character has tuning, named items, disciplines or inherited passives.
fn has_extras(d: &Data, character: &Dict, world: &str) -> bool {
    let tuning = json::dget(character, "tuning").values().any(|v| json::to_int(v) != 0);
    tuning || !json::aget(character, "named_items").is_empty() || !json::aget(character, "disciplines").is_empty() || !d.character_passives(character, world).is_empty()
}

struct Ctx {
    world: String,
    level: i32,
    shifted: bool,
}

/// The unit id of story reference `id`: a plain character (or catalog unit) is its base unit (a level-shifted copy
/// when shifted); a character with extras is the copy simw_<world>_<id> that spawns with them.
fn map_ref(r: &mut Runner, ctx: &Ctx, id: &str) -> String {
    if id.is_empty() {
        return String::new();
    }
    let d = r.d;
    let character = d.resolve(&ctx.world, "characters", id);
    let base_id = if character.is_empty() { id.to_string() } else { json::s_or(character, "base_unit", id) };
    if character.is_empty() || !has_extras(d, character, &ctx.world) {
        return catalog_ids(r, &[base_id], ctx.level, ctx.shifted).remove(0);
    }
    let e = d.entry("units", &base_id);
    let copy = format!("simw_{}", json::s_or(character, "_qid", id).replace(':', "_"));
    let level = if ctx.shifted { ctx.level } else { json::i(e, "level", 1) as i32 };
    let def = Def { copy_of: base_id.clone(), character: Some((ctx.world.clone(), id.to_string())), ..Def::new(&json::s(e, "class_id"), level, &json::s_or(e, "tier", "standard"), false) };
    r.define(&copy, def);
    copy
}

fn map_refs(r: &mut Runner, ctx: &Ctx, ids: &[Value]) -> Vec<String> {
    ids.iter().map(|v| map_ref(r, ctx, &json::gstr(v))).collect()
}

/// Phases (and any node) with every `unit` / `units` reference mapped like the roster.
fn map_node(r: &mut Runner, ctx: &Ctx, node: &Value) -> Value {
    match node {
        Value::Object(o) => {
            let mut res = Map::new();
            for (k, v) in o {
                let mapped = match (k.as_str(), v) {
                    ("unit", Value::String(s)) => json!(map_ref(r, ctx, s)),
                    ("units", Value::Array(a)) => json!(map_refs(r, ctx, a)),
                    _ => map_node(r, ctx, v),
                };
                res.insert(k.clone(), mapped);
            }
            Value::Object(res)
        }
        Value::Array(a) => Value::Array(a.iter().map(|v| map_node(r, ctx, v)).collect()),
        other => other.clone(),
    }
}

/// Encounter `id` as seen from `world`, roster mapped to units; elevel_set: level-shifted copies at `foe_level`.
/// full: also the objective, deployment, phases, guests, protect and morale fields. None when it does not exist.
pub fn encounter(r: &mut Runner, id: &str, foe_level: i32, elevel_set: bool, world: &str, full: bool) -> Option<Enc> {
    let entry = r.d.resolve(world, "encounters", id).clone();
    if entry.is_empty() {
        return None;
    }
    let ctx = Ctx { world: world.to_string(), level: foe_level, shifted: elevel_set };
    let mut base = Map::new();
    for key in ["zone", "arena", "exits", "total_blocks", "unit_ap", "ap_regen", "wave_round", "wave_block", "wave_label"] {
        if let Some(v) = entry.get(key) {
            base.insert(key.into(), v.clone());
        }
    }
    let enemies = map_refs(r, &ctx, json::aget(&entry, "enemies"));
    let wave = map_refs(r, &ctx, json::aget(&entry, "wave_units"));
    base.insert("wave_units".into(), json!(wave));
    if full {
        for key in ["objective", "objective_tiles", "objective_rounds", "objective_count", "objective_text", "deployment"] {
            if let Some(v) = entry.get(key) {
                base.insert(key.into(), v.clone());
            }
        }
        let unit = map_ref(r, &ctx, &json::s(&entry, "objective_unit"));
        base.insert("objective_unit".into(), json!(unit));
        let guests = map_refs(r, &ctx, json::aget(&entry, "guests"));
        base.insert("guests".into(), json!(guests));
        let protect = map_refs(r, &ctx, json::aget(&entry, "protect"));
        base.insert("protect".into(), json!(protect));
        let phases: Vec<Value> = json::aget(&entry, "phases").iter().map(|p| map_node(r, &ctx, p)).collect();
        base.insert("phases".into(), Value::Array(phases));
        for key in MORALE_KEYS {
            if let Some(v) = entry.get(key) {
                let v = if key == "morale_leader" { json!(map_ref(r, &ctx, &json::gstr(v))) } else { v.clone() };
                base.insert(key.into(), v);
            }
        }
    }
    Some(Enc { enemies, base, label: id.into(), max_rounds: json::i(&entry, "max_rounds", 0) as i32, extra: false })
}
