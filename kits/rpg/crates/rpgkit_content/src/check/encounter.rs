//! Encounter rows: ContentCatalog._encounter_errors with the shared encounter validators (EncounterPhases,
//! EncounterObjectives, EncounterMorale, PurseRules: `problems`, also for world encounters) and the warnings (the
//! PACE1 boss without an enrage, the PURSE1 purse).

use super::View;
use crate::source::content;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_battle::encounter::{
    ARRIVAL, ARRIVAL_BLOCK, ARRIVAL_ROUND, BROKEN, BROKEN_KEYS, DOWNED, DOWNED_KEYS, FALLBACKS, KILL_EFFECT, KILL_EFFECT_KEYS, KILL_MAX_UNITS, KILL_MAX_WITHIN, PARTY_NAME, PARTY_PREFIX, PICK_DEFAULTS, ROUND_KEYS, STAY_HOLD,
    STAY_MODES, STAY_STAND, STAY_THEN, TALK_EFFECT, TALK_EFFECT_KEYS, SIDE_EFFECT, SIDE_EFFECT_KEYS, SIDE_TARGETS, SWAP_EFFECT, SWAP_EFFECT_KEYS, SWAP_HP, SWAP_KEEP_PCT, ENEMY_SIDE, PARTY_SIDE,
};
use rpgkit_battle::grid::{rows_valid, Grid, TerrainTable, GRID_H, GRID_W, V2};
use rpgkit_rules::status;
use serde_json::{json, Value};
use std::sync::OnceLock;

const MAX_PARTY: usize = 4;

// ---- The arrival (PHASEFOLD) ----

/// PHASEFOLD 1.3: the retired `wave_units` field (label "Reinforcements", enemy units or enemy characters, dupes
/// allowed). The arrival's units are checked as that field was (its messages, at its place after `enemies`), and it
/// filters the refs of every phase wave.
pub fn wave_field(world: bool) -> &'static Dict {
    static CONTENT: OnceLock<Dict> = OnceLock::new();
    static WORLD: OnceLock<Dict> = OnceLock::new();
    let make = |r: &str, filter: Value| json!({"key": "wave_units", "label": "Reinforcements", "type": "refs", "default": [], "ref": r, "ref_filter": filter, "dupes": true}).as_object().cloned().unwrap_or_default();
    if world {
        WORLD.get_or_init(|| make("characters", json!({"role": "enemy"})))
    } else {
        CONTENT.get_or_init(|| make("units", json!({"side": "enemy"})))
    }
}

fn is_arrival(phase: &Dict) -> bool {
    json::s(json::dget(phase, "trigger"), "kind") == ARRIVAL
}

/// The encounter's first arrival phase (PHASEFOLD), if any.
pub fn arrival(e: &Dict) -> Option<&Dict> {
    dicts(json::get(e, "phases")).into_iter().find(|p| is_arrival(p))
}

/// The units of the arrival's wave effects as one list (the value the retired wave_units field held).
pub fn arrival_units(e: &Dict) -> Option<Value> {
    arrival(e).map(|p| Value::Array(dicts(json::get(p, "effects")).into_iter().filter(|fx| json::s(fx, "type") == "wave").flat_map(|fx| json::arr(json::get(fx, "units")).to_vec()).collect()))
}

/// PHASEFOLD 1.2: an arrival past the end of the round or after the last round (the retired wave fields' messages, at
/// their place: after the party and enemy counts).
pub fn arrival_errors(e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(t) = arrival(e).map(|p| json::dget(p, "trigger")) else { return errors };
    let read = |key: &str, default: i64| t.get(key).cloned().unwrap_or(json::int(default));
    let (block, round) = (read("block", ARRIVAL_BLOCK as i64), read("round", ARRIVAL_ROUND as i64));
    let total = json::to_int(json::get(e, "total_blocks"));
    if whole(&block) && json::to_int(&block) > total {
        errors.push(format!("The wave block is past the end of the round ({total} blocks)."));
    }
    if whole(&round) && json::to_int(&round) > json::to_int(json::get(e, "max_rounds")) {
        errors.push("The wave round is after the last round.".into());
    }
    errors
}

/// PHASEFX1: the side of unit row `u` (its `side`, the schema default `enemy` when empty); None for no row.
pub fn unit_row_side(u: &Dict) -> Option<String> {
    if u.is_empty() {
        return None;
    }
    let side = json::s(u, "side");
    Some(if side.is_empty() { ENEMY_SIDE.to_string() } else { side })
}

/// An int, or a float with no fraction (EncounterPhases._is_whole; JSON numbers may load as floats).
fn whole(v: &Value) -> bool {
    json::is_int(v) || v.as_f64().is_some_and(|f| f == f.floor())
}

fn strings(v: &Value) -> Vec<String> {
    json::arr(v).iter().map(gstr).collect()
}

fn dicts(v: &Value) -> Vec<&Dict> {
    json::arr(v).iter().filter_map(Value::as_object).collect()
}

fn cell_of(pair: &[Value]) -> V2 {
    V2::new(json::to_int(&pair[0]) as i32, json::to_int(&pair[1]) as i32)
}

fn on_grid(c: V2) -> bool {
    c.x >= 0 && c.y >= 0 && c.x < GRID_W && c.y < GRID_H
}

/// ContentCatalog._encounter_errors of a normalized catalog encounter.
pub fn errors(v: &View, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let party = json::arr(json::get(e, "party"));
    let foes = json::arr(json::get(e, "enemies"));
    if party.is_empty() {
        errors.push("Add at least one party member.".into());
    }
    if foes.is_empty() {
        errors.push("Add at least one enemy.".into());
    }
    if party.len() > MAX_PARTY {
        errors.push(format!("The party holds at most {MAX_PARTY} members."));
    }
    errors.extend(arrival_errors(e));
    errors.extend(phase_problems(
        e,
        &|r: &str| {
            let target = v.get("units", r);
            !target.is_empty() && super::passes_filter(wave_field(false), target)
        },
        &|r: &str| unit_row_side(v.get("units", r)),
    ));
    // ENCOUNTER2-C DT4: dialogue trees are world rows
    if !talk_effects(e).is_empty() {
        errors.push("A talk needs a world encounter (dialogue trees live in worlds).".into());
    }
    errors.extend(objective_problems(v, e, &|_: &str| true));
    errors.extend(morale_problems(e));
    errors.extend(purse_problems(e));
    if e.contains_key("start_status") {
        errors.push("Start buffs (start_status) are for world encounters only.".into());
    }
    match json::get(e, "exits") {
        Value::Array(list) => {
            for x in list {
                let pair = json::arr(x);
                if pair.len() != 2 {
                    errors.push("Exit tiles are [x, y] pairs.".into());
                    break;
                }
                let cell = cell_of(pair);
                if !on_grid(cell) {
                    errors.push(format!("Exit {cell} is off the 12x9 grid."));
                }
            }
        }
        _ => errors.push("Exit tiles must be a list of [x, y] pairs.".into()),
    }
    errors.extend(rows_problems(e));
    // ENCOUNTER2 S1 and section 6: unkillable refs and the stay-behind block
    errors.extend(unkillable_problems(e));
    errors.extend(stay_behind_problems(v, e, false));
    // ENCOUNTER2-C 5.1: the scripted rounds
    errors.extend(script_problems(v, e, &|r: &str| v.get("units", r).clone()));
    // HOOKS1 AH1 and DU2: the arena hazards and the deck phase effects
    errors.extend(super::hooks::arena_hazards_errors(e, &v.ids(rpgkit_data::UNIT_METERS_KIND)));
    errors.extend(super::hooks::deck_effect_errors(v, e, None, &|r: &str| v.get("units", r).clone()));
    errors
}

/// ENCOUNTER2 R0: `rows` is empty (the zone's rows) or GRID_H strings of GRID_W known terrain glyphs (catalog and
/// world encounters).
pub fn rows_problems(e: &Dict) -> Vec<String> {
    match json::get(e, "rows") {
        Value::Array(list) if list.is_empty() => Vec::new(),
        Value::Array(list) if list.iter().all(Value::is_string) && rows_valid(&strings(json::get(e, "rows"))) => Vec::new(),
        Value::Null => Vec::new(),
        _ => vec![format!("Rows are {GRID_H} strings of {GRID_W} terrain glyphs.")],
    }
}

/// ContentCatalog.warnings_for("encounters"): EncounterPhases.pace_warnings and PurseRules.warnings.
pub fn warnings(v: &View, e: &Dict) -> Vec<String> {
    let mut out = pace_warnings(e, &|r: &str| json::s(&content("units", v.get("units", r)), "tier"));
    out.extend(purse_warnings(e));
    // ENCOUNTER2-C 5.3: a catalog script plays only under the timeline ruleset
    out.extend(script_warnings(e));
    // HOOKS1 DU1 (user 2026-10-29 (b)): an add that pushes out an automatic deck's families
    out.extend(super::hooks::deck_effect_warnings(v, e, None, &|r: &str| v.get("units", r).clone()));
    out
}

// ---- EncounterPhases (BOSS1, PACE1) ----

const MAX_PHASES: usize = 4;
const MAX_EFFECTS: usize = 5;
const MAX_NAME: usize = 40;
const MAX_LINE: usize = 160;
const MAX_UNITS: usize = 12;
const PCT_MIN: i64 = 1;
const PCT_MAX: i64 = 99;
/// An hp_below trigger's `or_round` floor (Godot's).
const ROUND_MIN: i64 = 2;
/// ENCOUNTER2-C PT1: a `round` trigger may name round 1 (it fires before round 1 is planned; Godot's floor was 2).
const ROUND_TRIGGER_MIN: i64 = 1;
const STATUS_ROUNDS_MIN: i64 = 1;
const STATUS_ROUNDS_MAX: i64 = 5;
const DEFAULT_STATUS_ROUNDS: i64 = 2;
const ENRAGE_POWER_MIN: i64 = 1;
const ENRAGE_POWER_MAX: i64 = 25;
const DEFAULT_ENRAGE_POWER: i64 = 20;
/// The phase effects (BOSS1; HOOKS1 DU1 adds `deck`; FX1 5.1 `shake`; ENCOUNTER2 P1 `kill`; ENCOUNTER2-C DT1 `talk`;
/// PHASEFX1 `side` and `swap`).
const EFFECTS: [&str; 11] = ["line", "wave", "status", "ai", "enrage", rpgkit_data::DECK_EFFECT, rpgkit_data::SHAKE_EFFECT, KILL_EFFECT, TALK_EFFECT, SIDE_EFFECT, SWAP_EFFECT];
/// ENCOUNTER2 P2 adds `group`.
const PHASE_KEYS: [&str; 6] = ["id", "name", "trigger", "effects", "pause_autoplay", "group"];
/// ENCOUNTER2 P2: a group is 1 to 24 characters a-z, 0-9 or _.
const MAX_GROUP: usize = 24;
const AI_ROLES: [&str; 4] = ["AGGRESSOR", "SKIRMISHER", "SNIPER", "SUPPORT"];
/// HOOKS1 AT1 adds SOFTEST and TOUGHEST (read under the rule ai_tactics).
const AI_TARGET_PREFS: [&str; 4] = ["NEAREST", "WEAKEST", "SOFTEST", "TOUGHEST"];
const AI_RATIOS: [&str; 3] = ["flee_ratio", "defend_below", "heal_below"];
/// HOOKS1 AT2 adds `tactics` (it replaces the unit's list).
const AI_SETTINGS: [&str; 7] = ["role", "target_pref", "reckless", "flee_ratio", "defend_below", "heal_below", "tactics"];
const PACES: [&str; 4] = ["auto", "mob", "elite", "boss"];

fn trigger_keys(kind: &str) -> &'static [&'static str] {
    match kind {
        "hp_below" => &["kind", "unit", "pct", "or_round"],
        ARRIVAL => &["kind", "round", "block"],
        // ENCOUNTER2-C PT1-PT3
        DOWNED => &DOWNED_KEYS,
        BROKEN => &BROKEN_KEYS,
        _ => &ROUND_KEYS,
    }
}

fn effect_keys(kind: &str) -> &'static [&'static str] {
    match kind {
        "line" => &["type", "text"],
        "wave" => &["type", "units", "label"],
        "status" => &["type", "unit", "status", "rounds", "magnitude"],
        "ai" => &["type", "unit", "role", "target_pref", "reckless", "flee_ratio", "defend_below", "heal_below", "tactics"],
        rpgkit_data::DECK_EFFECT => &rpgkit_data::DECK_EFFECT_KEYS,
        rpgkit_data::SHAKE_EFFECT => &rpgkit_data::SHAKE_EFFECT_KEYS,
        KILL_EFFECT => &KILL_EFFECT_KEYS,
        TALK_EFFECT => &TALK_EFFECT_KEYS,
        SIDE_EFFECT => &SIDE_EFFECT_KEYS,
        SWAP_EFFECT => &SWAP_EFFECT_KEYS,
        _ => &["type", "unit", "power"],
    }
}

/// The wave units of one phase (every wave effect, in order: EncounterPhases.wave_refs_of of the parsed phase).
fn wave_refs_of(phase: &Dict) -> Vec<String> {
    dicts(json::get(phase, "effects")).into_iter().filter(|fx| json::s(fx, "type") == "wave").flat_map(|fx| strings(json::get(fx, "units"))).collect()
}

/// EncounterPhases.reserved_refs: every phase-wave unit of an encounter, in phase order (the arrival's first: it is
/// first in the list, though never reserved in battle).
pub fn reserved_refs(e: &Dict) -> Vec<String> {
    dicts(json::get(e, "phases")).into_iter().flat_map(wave_refs_of).collect()
}

/// EncounterPhases.problems: the `phases` list (the arrival included, PHASEFOLD 1.3) and the unit cap across enemies
/// and phase waves. `ref_ok(ref)` says whether a phase-wave ref resolves under the wave field's enemy filter (the
/// arrival's refs are checked as that field, `wave_field`); `unit_side(ref)` is the side of the unit row a swap's `to`
/// names (None: no such unit; PHASEFX1).
pub fn phase_problems(e: &Dict, ref_ok: &dyn Fn(&str) -> bool, unit_side: &dyn Fn(&str) -> Option<String>) -> Vec<String> {
    let mut errors = Vec::new();
    let raw = e.get("phases").cloned().unwrap_or(Value::Array(Vec::new()));
    let Value::Array(list) = raw else {
        errors.push("Phases must be a list of phase objects.".into());
        return errors;
    };
    let enemies = strings(json::get(e, "enemies"));
    let max_rounds = json::to_int(json::get(e, "max_rounds"));
    let total_blocks = json::to_int(json::get(e, "total_blocks"));
    let arrivals = list.iter().filter_map(Value::as_object).filter(|p| is_arrival(p)).count();
    if list.len() - arrivals > MAX_PHASES {
        errors.push(format!("An encounter holds at most {MAX_PHASES} phases."));
    }
    let mut known: Vec<String> = enemies.clone();
    let mut phase_units = 0;
    for p in list.iter().filter_map(Value::as_object) {
        let refs = wave_refs_of(p);
        phase_units += refs.len();
        known.extend(refs);
    }
    if enemies.len() + phase_units > MAX_UNITS {
        errors.push(format!("Encounter enemies, waves and phase waves hold at most {MAX_UNITS} units."));
    }
    let mut seen: Vec<String> = Vec::new();
    let mut enrages = 0;
    // HOOKS1 DU2: one deck effect per unit ref
    let mut deck_units: Vec<String> = Vec::new();
    for fx in list.iter().filter_map(Value::as_object).flat_map(|p| dicts(json::get(p, "effects"))).filter(|fx| json::s(fx, "type") == rpgkit_data::DECK_EFFECT) {
        let unit = json::s(fx, "unit");
        if deck_units.contains(&unit) {
            errors.push(format!("Phases hold one deck effect per unit ('{unit}' has two)."));
        }
        deck_units.push(unit);
    }
    for (index, p) in list.iter().enumerate() {
        let Some(phase) = p.as_object() else {
            errors.push("Each phase must be an object.".into());
            continue;
        };
        enrages += dicts(json::get(phase, "effects")).iter().filter(|fx| json::s(fx, "type") == "enrage").count();
        let id = json::s(phase, "id");
        let tag = if id.is_empty() { "?".to_string() } else { id.clone() };
        if id.is_empty() || !aegis_content::schema::is_id(&id) {
            errors.push(format!("Phase '{tag}': the id must be lowercase snake_case (a-z, 0-9, _)."));
        } else if seen.contains(&id) {
            errors.push(format!("Phase ids must be unique: '{id}'."));
        }
        seen.push(id);
        for key in phase.keys() {
            if !PHASE_KEYS.contains(&key.as_str()) {
                errors.push(format!("Phase '{tag}': unknown key '{key}'."));
            }
        }
        let pname = json::s(phase, "name");
        let pname = pname.trim();
        if pname.is_empty() {
            errors.push(format!("Phase '{tag}' needs a name."));
        } else if pname.chars().count() > MAX_NAME {
            errors.push(format!("Phase '{tag}': the name holds at most {MAX_NAME} characters."));
        }
        if phase.get("pause_autoplay").is_some_and(|b| !b.is_boolean()) {
            errors.push(format!("Phase '{tag}': pause_autoplay must be true or false."));
        }
        if let Some(g) = phase.get("group") {
            let ok = g.as_str().is_some_and(|s| (1..=MAX_GROUP).contains(&s.len()) && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'));
            if !ok {
                errors.push(format!("Phase '{tag}': the group is 1 to {MAX_GROUP} characters a-z, 0-9 or _."));
            }
        }
        let arrival = is_arrival(phase);
        // ENCOUNTER2 P5: an hp_below unit may be an enemy or a phase-wave ref (the message is Godot's)
        errors.extend(trigger_errors(&tag, json::get(phase, "trigger"), &known, max_rounds, total_blocks));
        errors.extend(event_trigger_errors(&tag, json::get(phase, "trigger"), e, &known));
        errors.extend(effects_errors(&tag, json::get(phase, "effects"), &known, ref_ok, arrival));
        errors.extend(kill_errors(&tag, phase, e, &known));
        errors.extend(side_errors(&tag, phase, e, &known));
        errors.extend(swap_errors(&tag, phase, e, &known, unit_side));
        errors.extend(enrage_errors(&tag, phase, &enemies));
        if arrival {
            errors.extend(arrival_shape_errors(&tag, phase, index));
        }
    }
    if enrages > 1 {
        errors.push("An encounter holds at most one enrage.".into());
    }
    if arrivals > 1 {
        errors.push("An encounter holds at most one arrival.".into());
    }
    errors
}

/// PHASEFOLD 1.3: an arrival holds one wave and nothing else, comes first and never pauses autoplay.
fn arrival_shape_errors(tag: &str, phase: &Dict, index: usize) -> Vec<String> {
    let mut errors = Vec::new();
    let effects = json::arr(json::get(phase, "effects"));
    if effects.len() != 1 || effects.iter().any(|fx| json::s(json::dict(fx), "type") != "wave") {
        errors.push(format!("Phase '{tag}': an arrival holds one wave and nothing else."));
    }
    if index != 0 {
        errors.push(format!("Phase '{tag}': the arrival comes first in the phase list."));
    }
    if phase.get("pause_autoplay") == Some(&Value::Bool(true)) {
        errors.push(format!("Phase '{tag}': an arrival never pauses autoplay."));
    }
    errors
}

fn trigger_errors(tag: &str, raw: &Value, enemies: &[String], max_rounds: i64, total_blocks: i64) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(t) = raw.as_object() else {
        errors.push(format!("Phase '{tag}' needs a trigger."));
        return errors;
    };
    let kind = json::s(t, "kind");
    if kind != "hp_below" && kind != "round" && kind != ARRIVAL && kind != DOWNED && kind != BROKEN {
        errors.push(format!("Phase '{tag}': unknown trigger '{kind}'."));
        return errors;
    }
    for key in t.keys() {
        if key == "or_round" && kind != "hp_below" {
            errors.push(format!("Phase '{tag}': or_round needs an hp_below trigger."));
        } else if !trigger_keys(&kind).contains(&key.as_str()) {
            errors.push(format!("Phase '{tag}': unknown trigger key '{key}'."));
        }
    }
    let in_range = |v: &Value, lo: i64, hi: i64| whole(v) && (lo..=hi).contains(&json::to_int(v));
    if kind == "hp_below" {
        let unit = json::s(t, "unit");
        if !enemies.contains(&unit) {
            errors.push(format!("Phase '{tag}': trigger unit '{unit}' is not in enemies."));
        }
        if !in_range(json::get(t, "pct"), PCT_MIN, PCT_MAX) {
            errors.push(format!("Phase '{tag}': pct must be {PCT_MIN}..{PCT_MAX}."));
        }
        let or_round = t.get("or_round").cloned().unwrap_or(json::int(0));
        let n = json::to_int(&or_round);
        if !whole(&or_round) || (n != 0 && (n < ROUND_MIN || n > max_rounds)) {
            errors.push(format!("Phase '{tag}': or_round must be {ROUND_MIN}..{max_rounds}."));
        }
    } else if kind == ARRIVAL {
        // Past the end: the retired wave field's messages (arrival_errors); here only below 1 or not whole.
        for (key, default, hi) in [("round", ARRIVAL_ROUND as i64, max_rounds), ("block", ARRIVAL_BLOCK as i64, total_blocks)] {
            let v = t.get(key).cloned().unwrap_or(json::int(default));
            if !whole(&v) || json::to_int(&v) < 1 {
                errors.push(format!("Phase '{tag}': {key} must be 1..{hi}."));
            }
        }
    } else if kind == "round" {
        if !in_range(json::get(t, "round"), ROUND_TRIGGER_MIN, max_rounds) {
            errors.push(format!("Phase '{tag}': round must be {ROUND_TRIGGER_MIN}..{max_rounds}."));
        }
        // ENCOUNTER2-C PT1: 0 = the round start, else that block's start
        if !in_range(t.get("block").unwrap_or(&json::int(0)), 0, total_blocks) {
            errors.push(format!("Phase '{tag}': block must be 0..{total_blocks}."));
        }
    }
    errors
}

/// ENCOUNTER2-C PT2, PT3: a `downed` or `broken` trigger's unit name (`party`, `party:<id>` with `<id>` in the party, or a
/// ref of the encounter: an enemy, a phase-wave ref or a guest; no `#n`) and a `downed` count of 1 to the units the name
/// covers.
fn event_trigger_errors(tag: &str, raw: &Value, e: &Dict, known: &[String]) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(t) = raw.as_object() else { return errors };
    let kind = json::s(t, "kind");
    if kind != DOWNED && kind != BROKEN {
        return errors;
    }
    let (party, guests) = (strings(json::get(e, "party")), strings(json::get(e, "guests")));
    let unit = json::s(t, "unit");
    let covered = if unit == PARTY_NAME {
        party.len()
    } else if let Some(id) = unit.strip_prefix(PARTY_PREFIX) {
        if party.iter().any(|p| p == id) {
            1
        } else {
            errors.push(format!("Phase '{tag}': trigger hero '{id}' is not in this encounter's party."));
            return errors;
        }
    } else {
        known.iter().chain(&guests).filter(|r| **r == unit).count()
    };
    if covered == 0 {
        errors.push(format!("Phase '{tag}': trigger unit '{unit}' is not in this encounter."));
        return errors;
    }
    if kind == DOWNED {
        let count = t.get("count").cloned().unwrap_or(json::int(1));
        if !whole(&count) || !(1..=covered as i64).contains(&json::to_int(&count)) {
            errors.push(format!("Phase '{tag}': count must be 1..{covered}."));
        }
    }
    errors
}

/// ENCOUNTER2-C DT1: every `talk` effect of an encounter's phases, with its phase id.
pub fn talk_effects(e: &Dict) -> Vec<(String, &Dict)> {
    dicts(json::get(e, "phases")).into_iter().flat_map(|p| dicts(json::get(p, "effects")).into_iter().filter(|fx| json::s(fx, "type") == TALK_EFFECT).map(move |fx| (json::s(p, "id"), fx))).collect()
}

/// ENCOUNTER2-C 5.2 (world encounters, world `w`): each talk's dialogue resolves there and its tree runs in a battle
/// (only set_flag / add_flag effects, `dialogue::battle_talk_errors`), its speaker is "" or a character there; a
/// `broken` trigger needs the poise rule in the world's ruleset.
pub fn world_talk_problems(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    for (phase, fx) in talk_effects(e) {
        let d = json::s(fx, "dialogue");
        if !d.is_empty() {
            let tree = v.worlds.resolve(w, "dialogues", &d);
            if tree.is_empty() {
                errors.push(format!("Unknown dialogue '{d}'."));
            } else {
                errors.extend(super::dialogue::battle_talk_errors(&d, tree));
            }
        }
        let speaker = json::s(fx, "speaker");
        if !speaker.is_empty() {
            errors.extend(super::world::ref_errors(v, w, &format!("Phase '{phase}': talk speaker"), "characters", &speaker));
        }
    }
    if has_broken_trigger(e) && !super::hooks::lists_rule(super::hooks::world_ruleset_row(v, w), rpgkit_rules::ruleset::POISE) {
        errors.push("A broken trigger needs the poise rule.".into());
    }
    errors
}

/// ENCOUNTER2-C PT3: some phase of the encounter has a `broken` trigger.
pub fn has_broken_trigger(e: &Dict) -> bool {
    dicts(json::get(e, "phases")).into_iter().any(|p| json::s(json::dget(p, "trigger"), "kind") == BROKEN)
}

fn effects_errors(tag: &str, raw: &Value, known: &[String], ref_ok: &dyn Fn(&str) -> bool, arrival: bool) -> Vec<String> {
    let mut errors = Vec::new();
    let list = json::arr(raw);
    if list.is_empty() || list.len() > MAX_EFFECTS {
        errors.push(format!("Phase '{tag}' needs 1 to {MAX_EFFECTS} effects."));
    }
    let (mut lines, mut waves, mut shakes, mut talks) = (0, 0, 0, 0);
    for item in list {
        let Some(fx) = item.as_object() else {
            errors.push(format!("Phase '{tag}': each effect must be an object."));
            continue;
        };
        let t = json::s(fx, "type");
        if !EFFECTS.contains(&t.as_str()) {
            errors.push(format!("Phase '{tag}': unknown effect '{t}'."));
            continue;
        }
        for key in fx.keys() {
            if !effect_keys(&t).contains(&key.as_str()) {
                errors.push(format!("Phase '{tag}': unknown {t} key '{key}'."));
            }
        }
        match t.as_str() {
            "line" => {
                lines += 1;
                let text = json::s(fx, "text");
                let n = text.trim().chars().count();
                if n == 0 || n > MAX_LINE {
                    errors.push(format!("Phase '{tag}': a line needs 1 to {MAX_LINE} characters."));
                }
            }
            "wave" => {
                waves += 1;
                errors.extend(wave_errors(tag, fx, ref_ok, arrival));
            }
            "status" => errors.extend(status_errors(tag, fx, known)),
            "ai" => errors.extend(ai_errors(tag, fx, known)),
            rpgkit_data::DECK_EFFECT => {
                errors.extend(unit_errors(tag, fx, known));
                errors.extend(super::hooks::deck_shape_errors(tag, fx));
            }
            rpgkit_data::SHAKE_EFFECT => {
                shakes += 1;
                let preset = json::s(fx, "preset");
                if !fx.get("preset").is_some_and(Value::is_string) || !rpgkit_data::SHAKE_PRESETS.contains(&preset.as_str()) {
                    errors.push(format!("Phase '{tag}': a shake needs a preset, one of {}.", rpgkit_data::SHAKE_PRESETS.join(", ")));
                }
            }
            TALK_EFFECT => {
                talks += 1;
                if !fx.get("dialogue").is_some_and(|d| d.as_str().is_some_and(|d| !d.is_empty())) {
                    errors.push(format!("Phase '{tag}': a talk needs a dialogue."));
                }
                if fx.get("speaker").is_some_and(|s| !s.is_string()) {
                    errors.push(format!("Phase '{tag}': the talk speaker must be a character ref or \"\"."));
                }
            }
            _ => {}
        }
    }
    if talks > 1 {
        errors.push(format!("Phase '{tag}': one talk per phase."));
    }
    if lines > 1 {
        errors.push(format!("Phase '{tag}': one line per phase."));
    }
    if shakes > 1 {
        errors.push(format!("Phase '{tag}': one shake per phase."));
    }
    if waves > 1 {
        errors.push(format!("Phase '{tag}': one wave per phase."));
    }
    errors
}

/// One wave effect; an arrival's unit refs are checked as the wave field instead (`wave_field`, validate_entry).
fn wave_errors(tag: &str, fx: &Dict, ref_ok: &dyn Fn(&str) -> bool, arrival: bool) -> Vec<String> {
    let mut errors = Vec::new();
    let units = match fx.get("units") {
        Some(Value::Array(a)) if !a.is_empty() => a,
        _ => {
            errors.push(format!("Phase '{tag}': a wave needs at least one unit."));
            return errors;
        }
    };
    for u in units.iter().filter(|_| !arrival) {
        let r = gstr(u);
        if !u.is_string() || !ref_ok(&r) {
            errors.push(format!("Phase '{tag}': wave unit '{r}' is not an enemy."));
        }
    }
    if fx.get("label").is_some_and(|l| !l.is_string()) {
        errors.push(format!("Phase '{tag}': the wave label must be text."));
    }
    errors
}

/// ENCOUNTER2 P1 and section 6: each `kill` effect of `phase`: 1 to KILL_MAX_UNITS refs, each an enemy, a phase-wave
/// ref or a guest, never a hero (a `party` ref) or a protected guest; `by` "" or a unit of the encounter; `within`
/// 0..=KILL_MAX_WITHIN, and above 0 only with a `by`. One error per mistake.
fn kill_errors(tag: &str, phase: &Dict, e: &Dict, known: &[String]) -> Vec<String> {
    let mut errors = Vec::new();
    let (party, guests, protect) = (strings(json::get(e, "party")), strings(json::get(e, "guests")), strings(json::get(e, "protect")));
    for fx in dicts(json::get(phase, "effects")).into_iter().filter(|fx| json::s(fx, "type") == KILL_EFFECT) {
        let units = match fx.get("units") {
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        };
        if units.is_empty() || units.len() > KILL_MAX_UNITS {
            errors.push(format!("Phase '{tag}': a kill names 1 to {KILL_MAX_UNITS} units."));
        }
        for u in &units {
            let r = gstr(u);
            if !u.is_string() {
                errors.push(format!("Phase '{tag}': kill unit '{r}' is not in this encounter."));
            } else if party.contains(&r) {
                errors.push(format!("Kill names hero '{r}'."));
            } else if protect.contains(&r) {
                errors.push(format!("Kill names the protected guest '{r}'."));
            } else if !known.contains(&r) && !guests.contains(&r) {
                errors.push(format!("Phase '{tag}': kill unit '{r}' is not in this encounter."));
            }
        }
        let by = fx.get("by").cloned().unwrap_or(Value::from(""));
        let by_ref = gstr(&by);
        if !by.is_string() || !(by_ref.is_empty() || party.contains(&by_ref) || known.contains(&by_ref) || guests.contains(&by_ref)) {
            errors.push(format!("Phase '{tag}': kill by '{by_ref}' is not in this encounter."));
        }
        let within = fx.get("within").cloned().unwrap_or(json::int(0));
        if !whole(&within) || !(0..=KILL_MAX_WITHIN as i64).contains(&json::to_int(&within)) {
            errors.push(format!("Phase '{tag}': within must be 0..{KILL_MAX_WITHIN}."));
        } else if json::to_int(&within) > 0 && by_ref.is_empty() {
            errors.push(format!("Phase '{tag}': a kill within tiles needs a by unit."));
        }
    }
    errors
}

/// PHASEFX1 (ACTS23 PK6 hook l): each `side` effect of `phase`: 1 to KILL_MAX_UNITS refs, each an enemy, a phase-wave
/// ref or a guest, never a hero (a `party` ref), a protected guest or a slay objective's target; `to` one of
/// SIDE_TARGETS. One error per mistake.
fn side_errors(tag: &str, phase: &Dict, e: &Dict, known: &[String]) -> Vec<String> {
    let mut errors = Vec::new();
    let (party, guests, protect) = (strings(json::get(e, "party")), strings(json::get(e, "guests")), strings(json::get(e, "protect")));
    let slay = if json::s(e, "objective") == "slay" { json::s(e, "objective_unit") } else { String::new() };
    for fx in dicts(json::get(phase, "effects")).into_iter().filter(|fx| json::s(fx, "type") == SIDE_EFFECT) {
        let units = json::arr(json::get(fx, "units"));
        if units.is_empty() || units.len() > KILL_MAX_UNITS {
            errors.push(format!("Phase '{tag}': a side names 1 to {KILL_MAX_UNITS} units."));
        }
        for u in units {
            let r = gstr(u);
            if u.is_string() && party.contains(&r) {
                errors.push(format!("Side names hero '{r}'."));
            } else if u.is_string() && protect.contains(&r) {
                errors.push(format!("Side names the protected guest '{r}'."));
            } else if u.is_string() && !slay.is_empty() && r == slay {
                errors.push(format!("Side names the slay target '{r}'."));
            } else if !u.is_string() || (!known.contains(&r) && !guests.contains(&r)) {
                errors.push(format!("Phase '{tag}': side unit '{r}' is not in this encounter."));
            }
        }
        let to = fx.get("to").cloned().unwrap_or(Value::Null);
        if !to.as_str().is_some_and(|t| SIDE_TARGETS.contains(&t)) {
            errors.push(format!("Phase '{tag}': side to '{}' must be one of {}.", gstr(&to), SIDE_TARGETS.join(", ")));
        }
    }
    errors
}

/// PHASEFX1 (ACTS23 PK6 hook m): each `swap` effect of `phase`: `unit` an enemy, a phase-wave ref or a guest (never a
/// hero); `to` a unit row (`unit_side`) of the swapped unit's side (a guest's is the party side, the others' the enemy
/// side); `hp` one of SWAP_HP (default keep_pct). One error per mistake.
fn swap_errors(tag: &str, phase: &Dict, e: &Dict, known: &[String], unit_side: &dyn Fn(&str) -> Option<String>) -> Vec<String> {
    let mut errors = Vec::new();
    let (party, guests) = (strings(json::get(e, "party")), strings(json::get(e, "guests")));
    for fx in dicts(json::get(phase, "effects")).into_iter().filter(|fx| json::s(fx, "type") == SWAP_EFFECT) {
        let unit = fx.get("unit").cloned().unwrap_or(Value::Null);
        let r = gstr(&unit);
        let mut side = None;
        if unit.is_string() && party.contains(&r) {
            errors.push(format!("Swap names hero '{r}'."));
        } else if unit.is_string() && guests.contains(&r) && !known.contains(&r) {
            side = Some(PARTY_SIDE);
        } else if unit.is_string() && known.contains(&r) {
            side = Some(ENEMY_SIDE);
        } else {
            errors.push(format!("Phase '{tag}': swap unit '{r}' is not in this encounter."));
        }
        let to = fx.get("to").cloned().unwrap_or(Value::Null);
        let to_ref = gstr(&to);
        match to.as_str().filter(|t| !t.is_empty()).and_then(unit_side) {
            None => errors.push(format!("Phase '{tag}': swap to '{to_ref}' is not a unit.")),
            Some(row_side) => {
                if let Some(side) = side.filter(|s| *s != row_side) {
                    errors.push(format!("Phase '{tag}': swap to '{to_ref}' is on the {row_side} side; '{r}' fights on the {side} side."));
                }
            }
        }
        let hp = fx.get("hp").cloned().unwrap_or(Value::from(SWAP_KEEP_PCT));
        if !hp.as_str().is_some_and(|h| SWAP_HP.contains(&h)) {
            errors.push(format!("Phase '{tag}': swap hp must be {}.", SWAP_HP.join(" or ")));
        }
    }
    errors
}

fn unit_errors(tag: &str, fx: &Dict, known: &[String]) -> Vec<String> {
    let r = json::s(fx, "unit");
    if known.contains(&r) {
        Vec::new()
    } else {
        vec![format!("Phase '{tag}': effect unit '{r}' is not in this encounter.")]
    }
}

fn status_errors(tag: &str, fx: &Dict, known: &[String]) -> Vec<String> {
    let mut errors = unit_errors(tag, fx, known);
    let st = json::s(fx, "status");
    if !status::buffs().contains(&st.as_str()) {
        errors.push(format!("Phase '{tag}': '{st}' is not a buff."));
        return errors;
    }
    if st == "empower" {
        errors.push(format!("Phase '{tag}': empower comes only from an enrage effect."));
        return errors;
    }
    let rounds = fx.get("rounds").cloned().unwrap_or(json::int(DEFAULT_STATUS_ROUNDS));
    if !whole(&rounds) || !(STATUS_ROUNDS_MIN..=STATUS_ROUNDS_MAX).contains(&json::to_int(&rounds)) {
        errors.push(format!("Phase '{tag}': rounds must be {STATUS_ROUNDS_MIN}..{STATUS_ROUNDS_MAX}."));
    }
    let magnitude = fx.get("magnitude").cloned().unwrap_or(json::int(0));
    let mag = json::to_int(&magnitude);
    match status::row_of(&st).filter(|r| r.magnitude_max > 0) {
        Some(r) => {
            if !whole(&magnitude) || mag < r.magnitude_min as i64 || mag > r.magnitude_max as i64 {
                errors.push(format!("Phase '{tag}': {st} magnitude must be {}..{}.", r.magnitude_min, r.magnitude_max));
            }
        }
        None if !whole(&magnitude) || mag != 0 => errors.push(format!("Phase '{tag}': {st} takes no magnitude (0).")),
        None => {}
    }
    errors
}

fn enrage_errors(tag: &str, phase: &Dict, enemies: &[String]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut any = false;
    for fx in dicts(json::get(phase, "effects")).into_iter().filter(|fx| json::s(fx, "type") == "enrage") {
        any = true;
        let r = json::s(fx, "unit");
        if !enemies.contains(&r) {
            errors.push(format!("Phase '{tag}': enrage unit '{r}' is not in enemies."));
        }
        let power = fx.get("power").cloned().unwrap_or(json::int(DEFAULT_ENRAGE_POWER));
        if !whole(&power) || !(ENRAGE_POWER_MIN..=ENRAGE_POWER_MAX).contains(&json::to_int(&power)) {
            errors.push(format!("Phase '{tag}': enrage power must be {ENRAGE_POWER_MIN}..{ENRAGE_POWER_MAX}."));
        }
    }
    if any && !json::to_bool_or(phase.get("pause_autoplay").unwrap_or(&Value::Bool(true)), true) {
        errors.push(format!("Phase '{tag}': an enrage phase must pause autoplay."));
    }
    errors
}

fn ai_errors(tag: &str, fx: &Dict, known: &[String]) -> Vec<String> {
    let mut errors = unit_errors(tag, fx, known);
    if !AI_SETTINGS.iter().any(|k| fx.contains_key(*k)) {
        errors.push(format!("Phase '{tag}': an ai effect needs at least one setting."));
        return errors;
    }
    if fx.contains_key("role") && !AI_ROLES.contains(&json::s(fx, "role").as_str()) {
        errors.push(format!("Phase '{tag}': role must be one of {}.", AI_ROLES.join(", ")));
    }
    if fx.contains_key("target_pref") && !AI_TARGET_PREFS.contains(&json::s(fx, "target_pref").as_str()) {
        errors.push(format!("Phase '{tag}': target_pref must be one of {}.", AI_TARGET_PREFS.join(", ")));
    }
    if fx.get("reckless").is_some_and(|b| !b.is_boolean()) {
        errors.push(format!("Phase '{tag}': reckless must be true or false."));
    }
    if let Some(t) = fx.get("tactics") {
        errors.extend(super::hooks::tactics_errors(t, &format!("Phase '{tag}': tactics")));
    }
    for key in AI_RATIOS {
        if let Some(v) = fx.get(key) {
            if !v.is_number() || !(0.0..=1.0).contains(&json::to_float_or(v, 0.0)) {
                errors.push(format!("Phase '{tag}': {key} must be 0.0..1.0."));
            }
        }
    }
    errors
}

/// EncounterPhases.pace_of: a set `pace` wins; auto = boss when any enemy (enemies, phase waves with the arrival) is
/// a boss, else elite when any is elite, else mob. `tier_of(ref)` is the unit tier of an encounter ref.
pub fn pace_of(e: &Dict, tier_of: &dyn Fn(&str) -> String) -> String {
    let pace = json::s_or(e, "pace", "auto");
    if PACES.contains(&pace.as_str()) && pace != "auto" {
        return pace;
    }
    let mut refs = strings(json::get(e, "enemies"));
    refs.extend(reserved_refs(e));
    let mut elite = false;
    for r in &refs {
        let tier = tier_of(r);
        if tier == "boss" {
            return "boss".into();
        }
        elite = elite || tier == "elite";
    }
    if elite { "elite" } else { "mob" }.into()
}

/// EncounterPhases.pace_warnings: a fight whose pace resolves to boss without an enrage phase.
pub fn pace_warnings(e: &Dict, tier_of: &dyn Fn(&str) -> String) -> Vec<String> {
    if pace_of(e, tier_of) != "boss" {
        return Vec::new();
    }
    let enraged = dicts(json::get(e, "phases")).into_iter().any(|p| dicts(json::get(p, "effects")).iter().any(|fx| json::s(fx, "type") == "enrage"));
    if enraged {
        return Vec::new();
    }
    vec![format!("Boss fight '{}' has no enrage phase (PACE1).", json::s(e, "id"))]
}

const START_MAX: usize = 2;
const START_TEXT_MAX: usize = 120;
const START_KEYS: [&str; 5] = ["status", "rounds", "magnitude", "requires_flags", "text"];

/// EncounterPhases.start_status_problems: a world encounter's `start_status` (PACE1 3.4): at most START_MAX buffs other
/// than taunt and the intent-bound ones (riposte; RULESET1-S: stealth), each with rounds, magnitude, flag conditions and
/// a text.
pub fn start_status_problems(raw: &Value) -> Vec<String> {
    let Some(list) = raw.as_array() else {
        return vec!["Start buffs must be a list of objects.".into()];
    };
    let mut errors = Vec::new();
    if list.len() > START_MAX {
        errors.push(format!("An encounter holds at most {START_MAX} start buffs."));
    }
    // RULESET1-S (planner pick): no intent-bound buff either (riposte, stealth): it needs a round intent, and no unit
    // has one at deployment
    let allowed: Vec<&str> = status::buffs().into_iter().filter(|s| *s != "taunt" && !status::intent_bound(s)).collect();
    for item in list {
        let Some(sb) = item.as_object() else {
            errors.push("Start buff: each entry must be an object.".into());
            continue;
        };
        for key in sb.keys().filter(|k| !START_KEYS.contains(&k.as_str())) {
            errors.push(format!("Start buff: unknown key '{key}'."));
        }
        let st = json::s(sb, "status");
        if !sb.get("status").is_some_and(Value::is_string) || !allowed.contains(&st.as_str()) {
            if status::intent_bound(&st) && st != "riposte" {
                // the Godot text names riposte, the one intent-bound row it had
                let bound: Vec<&str> = status::ROWS.iter().filter(|r| r.intent_bound).map(|r| r.id).collect();
                errors.push(format!("Start buff '{st}' must be a buff other than taunt or an intent-bound one ({}).", bound.join(", ")));
            } else {
                errors.push(format!("Start buff '{st}' must be a buff other than taunt or riposte."));
            }
            continue;
        }
        let rounds = sb.get("rounds").cloned().unwrap_or(json::int(DEFAULT_STATUS_ROUNDS));
        if !whole(&rounds) || !(STATUS_ROUNDS_MIN..=STATUS_ROUNDS_MAX).contains(&json::to_int(&rounds)) {
            errors.push(format!("Start buff '{st}': rounds must be {STATUS_ROUNDS_MIN}..{STATUS_ROUNDS_MAX}."));
        }
        let magnitude = sb.get("magnitude").cloned().unwrap_or(json::int(0));
        let (lo, hi) = status::row_of(&st).filter(|r| r.magnitude_max > 0).map_or((0, 0), |r| (r.magnitude_min as i64, r.magnitude_max as i64));
        if !whole(&magnitude) || !(lo..=hi).contains(&json::to_int(&magnitude)) {
            errors.push(format!("Start buff '{st}': magnitude must be {lo}..{hi}."));
        }
        errors.extend(super::quest::condition_errors(sb.get("requires_flags").unwrap_or(&Value::Array(Vec::new())), "Start buff flags"));
        let text = sb.get("text").and_then(Value::as_str).map(str::trim).unwrap_or("");
        if text.is_empty() || text.chars().count() > START_TEXT_MAX {
            errors.push(format!("Start buff '{st}' needs a text."));
        }
    }
    errors
}

// ---- EncounterObjectives (OBJ1) ----

const RING: [V2; 2] = [V2::new(5, 4), V2::new(6, 4)];
const MAX_GUESTS: usize = 2;
const MAX_OBJECTIVE_TEXT: usize = 80;
const DEFAULT_HOLD_ROUNDS: i64 = 3;

/// EncounterObjectives.problems: the objective fields. `guest_ok(ref)` is false for a guest that resolves but does
/// not fight on the party side.
pub fn objective_problems(v: &View, e: &Dict, guest_ok: &dyn Fn(&str) -> bool) -> Vec<String> {
    let mut errors = Vec::new();
    let objective = json::s_or(e, "objective", "rout");
    let max_rounds = json::to_int(json::get(e, "max_rounds"));
    let arena_open = json::s_or(e, "arena", "CLOSED") == "OPEN";
    let exits: Vec<V2> = json::arr(json::get(e, "exits")).iter().map(json::arr).filter(|p| p.len() == 2).map(cell_of).collect();
    match objective.as_str() {
        "hold" => {
            match json::get(e, "objective_tiles") {
                Value::Array(tiles) => {
                    let mut grid = Grid::new(false, Vec::new(), TerrainTable::from_catalog(&v.typed), 0);
                    grid.apply_board(&json::s(e, "zone"), &strings(json::get(e, "rows")), &v.typed.zones);
                    let tile_ok = |c: V2| {
                        if !on_grid(c) || !grid.is_walkable(c) {
                            return false;
                        }
                        if arena_open {
                            return c.x > 0 && c.y > 0 && c.x < GRID_W - 1 && c.y < GRID_H - 1;
                        }
                        !exits.contains(&c)
                    };
                    for raw in tiles {
                        let pair = json::arr(raw);
                        if pair.len() != 2 {
                            errors.push(format!("Objective tile {} is off the grid, blocked or an exit.", gstr(raw)));
                            continue;
                        }
                        let cell = cell_of(pair);
                        if !tile_ok(cell) {
                            errors.push(format!("Objective tile {cell} is off the grid, blocked or an exit."));
                        }
                    }
                    if tiles.is_empty() {
                        for cell in RING.into_iter().filter(|c| !tile_ok(*c)) {
                            errors.push(format!("Objective tile {cell} is off the grid, blocked or an exit."));
                        }
                    }
                }
                _ => errors.push("Objective tiles must be a list of [x, y] pairs.".into()),
            }
            let rounds = e.get("objective_rounds").map_or(DEFAULT_HOLD_ROUNDS, json::to_int);
            if rounds < 1 || rounds > max_rounds {
                errors.push(format!("Hold rounds must be 1..{max_rounds}."));
            }
        }
        "slay" => {
            let target = json::s(e, "objective_unit");
            if target.is_empty() || !strings(json::get(e, "enemies")).contains(&target) {
                errors.push("Slay needs a unit from enemies.".into());
            }
        }
        "escape" if !arena_open && exits.is_empty() => errors.push("Escape needs an exit tile.".into()),
        _ => {}
    }
    if json::s(e, "objective_text").chars().count() > MAX_OBJECTIVE_TEXT {
        errors.push(format!("Objective text holds at most {MAX_OBJECTIVE_TEXT} characters."));
    }
    let guests = strings(json::get(e, "guests"));
    if guests.len() > MAX_GUESTS {
        errors.push(format!("An encounter holds at most {MAX_GUESTS} guests."));
    }
    let mut seen: Vec<&String> = Vec::new();
    for r in &guests {
        if seen.contains(&r) {
            errors.push(format!("Guest '{r}' is listed twice."));
            continue;
        }
        seen.push(r);
        if !guest_ok(r) {
            errors.push(format!("Guest '{r}' must fight on the party side."));
        }
    }
    for r in strings(json::get(e, "protect")) {
        if !guests.contains(&r) {
            errors.push(format!("Protect '{r}' is not a guest."));
        }
    }
    errors
}

// ---- Stay-behind and unkillable (ENCOUNTER2 3 and 6; not in Godot) ----

/// The `stay_behind` keys (section 6); the hold-only and stand-only ones.
const STAY_KEYS: [&str; 13] = ["mode", "tiles", "tiles_label", "room", "fallback", "pick_default", "exits_open", "stand_hp_pct", "stand_statuses", "stand_line", "result_line", "flag", "then"];
const HOLD_ONLY: [&str; 2] = ["fallback", "pick_default"];
const STAND_ONLY: [&str; 4] = ["exits_open", "stand_hp_pct", "stand_statuses", "stand_line"];
const EXITS_OPEN: [&str; 2] = ["always", "after_stand"];
const MAX_HOLD_TILES: usize = 2;
const MAX_TILES_LABEL: usize = 24;
const MAX_STAY_LINE: usize = 160;
const MAX_STAND_STATUSES: usize = 3;
const STAND_ROUNDS: (i64, i64) = (1, 10);

/// The refs of every phase wave (the arrival's too).
fn phase_wave_refs(e: &Dict) -> Vec<String> {
    dicts(json::get(e, "phases"))
        .into_iter()
        .flat_map(|p| dicts(json::get(p, "effects")).into_iter().filter(|fx| json::s(fx, "type") == "wave").flat_map(|fx| strings(json::get(fx, "units"))).collect::<Vec<_>>())
        .collect()
}

/// ENCOUNTER2 S1: each `unkillable` ref is in `enemies` or a phase wave (catalog and world encounters).
pub fn unkillable_problems(e: &Dict) -> Vec<String> {
    let mut known = strings(json::get(e, "enemies"));
    known.extend(phase_wave_refs(e));
    strings(json::get(e, "unkillable")).into_iter().filter(|r| !known.contains(r)).map(|r| format!("Unkillable unit '{r}' is not in this encounter.")).collect()
}

/// A line with `{name}` as its only token, at most MAX_STAY_LINE characters.
fn stay_line_ok(text: &str) -> bool {
    text.chars().count() <= MAX_STAY_LINE && text.replace("{name}", "").chars().all(|c| c != '{' && c != '}')
}

/// ENCOUNTER2 section 6: the `stay_behind` block (empty, or the keys with their ranges; only with the escape objective).
/// `world`: a world encounter (`flag` and `then` are its own).
pub fn stay_behind_problems(v: &View, e: &Dict, world: bool) -> Vec<String> {
    let mut errors = Vec::new();
    let block = match json::get(e, "stay_behind") {
        Value::Null => return errors,
        Value::Object(b) if b.is_empty() => return errors,
        Value::Object(b) => b,
        _ => return vec!["Stay-behind must be an object ({} = none).".into()],
    };
    if json::s_or(e, "objective", "rout") != "escape" {
        errors.push("Stay-behind needs the escape objective.".into());
    }
    for key in block.keys().filter(|k| !STAY_KEYS.contains(&k.as_str())) {
        errors.push(format!("Stay-behind: unknown key '{key}'."));
    }
    let option = |key: &str, options: &[&str], errors: &mut Vec<String>| {
        if let Some(raw) = block.get(key) {
            if !raw.as_str().is_some_and(|s| options.contains(&s)) {
                errors.push(format!("Stay-behind {key} '{}' is not one of {}.", gstr(raw), options.join(", ")));
            }
        }
    };
    option("mode", &STAY_MODES, &mut errors);
    let hold = json::s_or(block, "mode", STAY_HOLD) != STAY_STAND;
    for key in block.keys() {
        let (only, other) = if HOLD_ONLY.contains(&key.as_str()) {
            (STAY_HOLD, !hold)
        } else if STAND_ONLY.contains(&key.as_str()) {
            (STAY_STAND, hold)
        } else {
            continue;
        };
        if other {
            errors.push(format!("Stay-behind {key} is for mode {only} only."));
        }
    }
    // the tiles: a hold names 1-2 walkable tiles on the board that are no exit; a stand names none
    match json::get(block, "tiles") {
        Value::Array(tiles) if !hold => {
            if !tiles.is_empty() {
                errors.push("A stand holds no tiles.".into());
            }
        }
        Value::Array(tiles) => {
            if tiles.is_empty() || tiles.len() > MAX_HOLD_TILES {
                errors.push(format!("A hold needs 1 to {MAX_HOLD_TILES} tiles."));
            }
            let arena_open = json::s_or(e, "arena", "CLOSED") == "OPEN";
            let exits: Vec<V2> = json::arr(json::get(e, "exits")).iter().map(json::arr).filter(|p| p.len() == 2).map(cell_of).collect();
            let mut grid = Grid::new(false, Vec::new(), TerrainTable::from_catalog(&v.typed), 0);
            grid.apply_board(&json::s(e, "zone"), &strings(json::get(e, "rows")), &v.typed.zones);
            for raw in tiles {
                let pair = json::arr(raw);
                let cell = (pair.len() == 2).then(|| cell_of(pair));
                let ok = cell.is_some_and(|c| {
                    on_grid(c) && grid.is_walkable(c) && !exits.contains(&c) && (!arena_open || (c.x > 0 && c.y > 0 && c.x < GRID_W - 1 && c.y < GRID_H - 1))
                });
                if !ok {
                    errors.push(format!("Hold tile {} is off the grid, blocked or an exit.", cell.map_or_else(|| gstr(raw), |c| c.to_string())));
                }
            }
        }
        Value::Null if hold => errors.push(format!("A hold needs 1 to {MAX_HOLD_TILES} tiles.")),
        Value::Null => {}
        _ => errors.push("Hold tiles must be a list of [x, y] pairs.".into()),
    }
    if let Some(label) = block.get("tiles_label") {
        let n = label.as_str().map_or(0, |s| s.trim().chars().count());
        if n == 0 || n > MAX_TILES_LABEL {
            errors.push(format!("Stay-behind tiles_label is 1 to {MAX_TILES_LABEL} characters."));
        }
    }
    let room = json::s(block, "room");
    if !room.is_empty() {
        // rooms come with ENCOUNTER2-R: until then no room id resolves
        errors.push(format!("Stay-behind room '{room}' is not a room of this encounter."));
    }
    option("fallback", &FALLBACKS, &mut errors);
    option("pick_default", &PICK_DEFAULTS, &mut errors);
    option("exits_open", &EXITS_OPEN, &mut errors);
    if let Some(pct) = block.get("stand_hp_pct") {
        if !whole(pct) || !(1..=100).contains(&json::to_int(pct)) {
            errors.push("Stay-behind stand_hp_pct must be 1..100.".into());
        }
    }
    match block.get("stand_statuses") {
        None => {}
        Some(Value::Array(list)) => {
            if list.len() > MAX_STAND_STATUSES {
                errors.push(format!("Stay-behind stand_statuses: at most {MAX_STAND_STATUSES}."));
            }
            for item in list {
                let st = item.as_object().map(|o| json::s(o, "status")).unwrap_or_default();
                if status::row_of(&st).is_none() {
                    errors.push(format!("Stand status '{st}' is not a status."));
                    continue;
                }
                let rounds = item.get("rounds").cloned().unwrap_or(json::int(STAND_ROUNDS.0));
                if !whole(&rounds) || !(STAND_ROUNDS.0..=STAND_ROUNDS.1).contains(&json::to_int(&rounds)) {
                    errors.push(format!("Stand status '{st}': rounds must be {}..{}.", STAND_ROUNDS.0, STAND_ROUNDS.1));
                }
            }
        }
        Some(_) => errors.push("Stay-behind stand_statuses must be a list of {status, rounds}.".into()),
    }
    for key in ["stand_line", "result_line"] {
        if let Some(line) = block.get(key) {
            if !line.as_str().is_some_and(stay_line_ok) {
                errors.push(format!("Stay-behind {key} holds at most {MAX_STAY_LINE} characters, {{name}} the only token."));
            }
        }
    }
    let flag = json::s(block, "flag");
    if world {
        if !flag.is_empty() && !super::quest::flag_key_ok(&flag) {
            errors.push(format!("Stay-behind flag '{flag}' must be a flag key (a-z, 0-9, _ and .)."));
        }
        option("then", &STAY_THEN, &mut errors);
    } else if !flag.is_empty() || block.get("then").is_some_and(|t| t.as_str() != Some("none")) {
        errors.push("Stay-behind flag and then are for world encounters only.".into());
    }
    errors
}

// ---- EncounterMorale (MORALE1) ----

const DEFAULT_BREAK: i64 = 50;
const BREAK_MIN: i64 = 0;
const BREAK_MAX: i64 = 90;
const MAX_MORALE_TEXT: usize = 80;

/// EncounterMorale.problems: the morale fields ([] while `morale` is off).
pub fn morale_problems(e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    if !json::to_bool(json::get(e, "morale")) {
        return errors;
    }
    let brk = e.get("morale_break").map_or(DEFAULT_BREAK, |b| json::to_int_or(b, DEFAULT_BREAK));
    if !(BREAK_MIN..=BREAK_MAX).contains(&brk) {
        errors.push(format!("Morale break must be {BREAK_MIN}..{BREAK_MAX}."));
    }
    let enemies = strings(json::get(e, "enemies"));
    let leader = json::s(e, "morale_leader");
    if !leader.is_empty() && !enemies.contains(&leader) {
        errors.push("Morale leader must be a unit from enemies.".into());
    }
    match e.get("morale_groups").unwrap_or(&Value::Array(Vec::new())) {
        Value::Array(groups) if !groups.is_empty() => errors.extend(group_problems(groups, enemies.len())),
        Value::Array(_) => {}
        _ => errors.push("Morale groups must be a list of enemy index lists.".into()),
    }
    if json::s(e, "morale_text").chars().count() > MAX_MORALE_TEXT {
        errors.push(format!("Morale text holds at most {MAX_MORALE_TEXT} characters."));
    }
    errors
}

fn group_problems(groups: &[Value], enemy_count: usize) -> Vec<String> {
    let mut errors = Vec::new();
    let mut seen = vec![0; enemy_count];
    let (mut bad_shape, mut empty, mut out_of_range) = (false, false, false);
    for g in groups {
        let Value::Array(members) = g else {
            bad_shape = true;
            continue;
        };
        empty |= members.is_empty();
        for x in members {
            let i = json::to_int(x);
            if !whole(x) || i < 0 || i >= enemy_count as i64 {
                out_of_range = true;
                continue;
            }
            seen[i as usize] += 1;
        }
    }
    if bad_shape {
        errors.push("Morale groups must be a list of enemy index lists.".into());
    }
    if empty {
        errors.push("A morale group cannot be empty.".into());
    }
    if out_of_range {
        errors.push("Morale group index out of range.".into());
    }
    if seen.iter().any(|n| *n != 1) {
        errors.push("Every enemy must be in exactly one morale group.".into());
    }
    errors
}

// ---- PurseRules (PURSE1) ----

const GOLD_CAP: i64 = 500;
const GOLD_SOFT_CAP: i64 = 100;
const PURSE_KEYS: [&str; 3] = ["gold_min", "gold_max", "once"];

/// PurseRules.world_loot_check: "" when `ref` resolves from `world` to an item reward, "missing" or "kind" otherwise
/// (a catalog encounter has no world: only a qualified `world:id` resolves).
pub fn world_loot_check(v: &View, world: &str, r: &str) -> &'static str {
    let hit = v.worlds.resolve(world, "loot", r);
    if hit.is_empty() {
        "missing"
    } else if json::s_or(hit, "reward_kind", "item") != "item" {
        "kind"
    } else {
        ""
    }
}

/// PurseRules.problems: the purse object. REWARD1 moved `loot` to the encounter's `key_loot`.
pub fn purse_problems(e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(raw) = e.get("purse") else {
        return errors;
    };
    let Some(p) = raw.as_object() else {
        errors.push("Purse must be an object {gold_min, gold_max, once}.".into());
        return errors;
    };
    if p.is_empty() {
        return errors;
    }
    for key in p.keys() {
        if key == "loot" {
            errors.push("Purse key 'loot' moved to key_loot (REWARD1).".into());
        } else if !PURSE_KEYS.contains(&key.as_str()) {
            errors.push(format!("Purse key '{key}' is not allowed."));
        }
    }
    let lo_raw = p.get("gold_min").cloned().unwrap_or(json::int(0));
    let lo_ok = whole(&lo_raw) && (0..=GOLD_CAP).contains(&json::to_int(&lo_raw));
    if !lo_ok {
        errors.push(format!("Purse gold_min must be a whole number 0..{GOLD_CAP}."));
    }
    let lo = if lo_ok { json::to_int(&lo_raw) } else { 0 };
    if let Some(hi) = p.get("gold_max") {
        if !whole(hi) || json::to_int(hi) < lo || json::to_int(hi) > GOLD_CAP {
            errors.push(format!("Purse gold_max must be gold_min..{GOLD_CAP}."));
        }
    }
    if p.get("once").is_some_and(|o| !o.is_boolean()) {
        errors.push("Purse once must be true or false.".into());
    }
    errors
}

/// PurseRules.warnings: gold above the soft cap, an empty purse.
pub fn purse_warnings(e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let Some(p) = e.get("purse").and_then(Value::as_object).filter(|p| !p.is_empty()) else {
        return out;
    };
    let lo = json::to_int(p.get("gold_min").unwrap_or(&json::int(0)));
    let hi = p.get("gold_max").map_or(lo, |h| json::to_int_or(h, lo));
    if hi > GOLD_SOFT_CAP {
        out.push(format!("Purse gold above {GOLD_SOFT_CAP}: check the budget."));
    }
    // a purse that still holds the retired loot is refused (purse_problems), not called empty
    if hi <= 0 && !p.contains_key("loot") {
        out.push("Empty purse.".into());
    }
    out
}

// ---- Scripted rounds (ENCOUNTER2-C 5.1) ----

/// ENCOUNTER2-C 5.1 (catalog and world encounters): `script` is empty or 1-3 round scripts (round 1..max_rounds, each
/// once; `lock` a bool, a locked round 1 refused with a hold escape; `others` and `fill` auto or rest; a banner of at
/// most 40 characters; 1-16 entries, one per unit), each entry's unit and unit target named as SN1 (`ref`, `ref#n`,
/// `party:<id>`, `party#n`), its tiles on the grid, its picks 1-4 of SN4's shapes (a known family, which an enemy-side
/// unit owns; a known item, party side only; a move only first). `unit_of(ref)` is the unit row a ref plays (a world
/// character's base unit).
pub fn script_problems(v: &View, e: &Dict, unit_of: &dyn Fn(&str) -> Dict) -> Vec<String> {
    use rpgkit_battle::script::{self as sc, ScriptPick, ScriptTarget, UnitName};
    let mut errors = Vec::new();
    let rounds = match json::get(e, "script") {
        Value::Null => return errors,
        Value::Array(list) => list,
        _ => return vec!["Script must be a list of round scripts.".into()],
    };
    if rounds.len() > sc::MAX_SCRIPTED_ROUNDS {
        errors.push(format!("At most {} scripted rounds.", sc::MAX_SCRIPTED_ROUNDS));
    }
    let max_rounds = json::to_int(json::get(e, "max_rounds"));
    let (party, guests) = (strings(json::get(e, "party")), strings(json::get(e, "guests")));
    let mut foes = strings(json::get(e, "enemies"));
    foes.extend(phase_wave_refs(e));
    let hold = json::s(json::dict(json::get(e, "stay_behind")), "mode") == STAY_HOLD;
    // SN1: the unit a name covers as (canonical key, enemy side, unit ref) when it is in this encounter
    let named = |name: &str| -> Result<(String, bool, String), String> {
        let missing = || format!("Script unit '{name}' is not in this encounter.");
        match UnitName::parse(name).ok_or_else(missing)? {
            UnitName::Hero(id) if party.contains(&id) => Ok((format!("party:{id}"), false, id)),
            UnitName::Hero(id) => Err(format!("Script hero '{id}' is not in this encounter's party.")),
            UnitName::HeroAt(n) if n <= party.len() => Ok((format!("party#{n}"), false, party[n - 1].clone())),
            UnitName::HeroAt(_) => Err(missing()),
            UnitName::Ref(r, n) => {
                let built = foes.iter().filter(|f| **f == r).count();
                if n <= built {
                    Ok((format!("{r}#{n}"), true, r))
                } else if n == 1 && guests.contains(&r) {
                    Ok((format!("{r}#1"), false, r))
                } else {
                    Err(missing())
                }
            }
        }
    };
    let off_grid = |c: V2| format!("Script tile [{}, {}] is off the grid.", c.x, c.y);
    let basics: Vec<String> = v.typed.basic_ids().iter().map(|id| v.typed.family_of(id).to_string()).chain(v.entries(rpgkit_rules::ruleset::KIND).iter().flat_map(|r| json::sget(r, "basics"))).collect();
    let mut seen: Vec<i64> = Vec::new();
    for raw in rounds {
        let Some(rs) = raw.as_object() else {
            errors.push("Each round script must be an object.".into());
            continue;
        };
        for key in rs.keys().filter(|k| !sc::ROUND_KEYS.contains(&k.as_str())) {
            errors.push(format!("Script round: unknown key '{key}'."));
        }
        let round = json::get(rs, "round");
        let n = json::to_int(round);
        if !whole(round) || !(1..=max_rounds).contains(&n) {
            errors.push(format!("Script round must be 1..{max_rounds}."));
        } else if seen.contains(&n) {
            errors.push(format!("Round {n} is scripted twice."));
        }
        seen.push(n);
        let lock = rs.get("lock").cloned().unwrap_or(Value::Bool(true));
        if !lock.is_boolean() {
            errors.push("Script lock must be true or false.".into());
        } else if lock == Value::Bool(true) && n == 1 && hold {
            errors.push("A hold escape picks its holder before round 1: script round 2 or later.".into());
        }
        if rs.get("others").is_some_and(|o| !o.as_str().is_some_and(|o| sc::MODES.contains(&o))) {
            errors.push("Script others is auto or rest.".into());
        }
        if rs.get("banner").is_some_and(|b| !b.as_str().is_some_and(|b| b.chars().count() <= sc::MAX_BANNER)) {
            errors.push(format!("A script banner is text of at most {} characters.", sc::MAX_BANNER));
        }
        let entries = json::arr(json::get(rs, "entries"));
        if !(1..=sc::MAX_ENTRIES).contains(&entries.len()) {
            errors.push(format!("A round script holds 1 to {} entries.", sc::MAX_ENTRIES));
        }
        let mut units: Vec<String> = Vec::new();
        for entry in entries {
            let Some(en) = entry.as_object() else {
                errors.push("Each script entry must be an object.".into());
                continue;
            };
            for key in en.keys().filter(|k| !sc::ENTRY_KEYS.contains(&k.as_str())) {
                errors.push(format!("Script entry: unknown key '{key}'."));
            }
            let unit = json::s(en, "unit");
            let who = match named(&unit) {
                Ok(w) => Some(w),
                Err(m) => {
                    errors.push(m);
                    None
                }
            };
            if let Some((key, _, _)) = &who {
                // once per unit and round
                if units.iter().filter(|u| *u == key).count() == 1 {
                    errors.push(format!("{unit} has two entries in round {n}."));
                }
                units.push(key.clone());
            }
            match sc::parse_target(en.get("target")) {
                Ok(ScriptTarget::Cell(c)) if !on_grid(c) => errors.push(off_grid(c)),
                Ok(ScriptTarget::Unit(t)) => {
                    if let Err(m) = named(&t) {
                        errors.push(m);
                    }
                }
                Ok(_) => {}
                Err(m) => errors.push(m),
            }
            if en.get("fill").is_some_and(|f| !f.as_str().is_some_and(|f| sc::MODES.contains(&f))) {
                errors.push("Script fill is auto or rest.".into());
            }
            let picks = json::arr(json::get(en, "picks"));
            let parsed: Result<Vec<ScriptPick>, String> = picks.iter().map(sc::parse_pick).collect();
            let parsed = match parsed {
                Ok(p) if (1..=sc::MAX_PICKS).contains(&p.len()) => p,
                _ => {
                    errors.push(sc::PICKS_TEXT.into());
                    continue;
                }
            };
            // an enemy-side unit's families: its unit row's (none listed: it knows every skill, as in battle) and the basics
            // (CLASS-BASICS: and its class basics)
            let owned: Option<Vec<String>> = who.as_ref().filter(|w| w.1).and_then(|w| {
                let row = unit_of(&w.2);
                let owned = super::unit::owned_families(v, &row);
                (!owned.is_empty()).then(|| rpgkit_rules::unit::DeckRule { basics: owned, ..Default::default() }.with_class(&v.typed, &json::s(&row, "class_id")).basics)
            });
            for (i, p) in parsed.iter().enumerate() {
                match p {
                    ScriptPick::Skill { family, .. } => {
                        if v.typed.tier_one(family).is_none() {
                            errors.push(format!("Unknown skill '{family}'."));
                        } else if owned.as_ref().is_some_and(|o| !o.contains(family) && !basics.contains(family)) {
                            errors.push(format!("{unit} has no skill '{family}'."));
                        }
                    }
                    ScriptPick::Item(id) => {
                        if v.get("items", id).is_empty() {
                            errors.push(format!("Unknown item '{id}'."));
                        } else if who.as_ref().is_some_and(|w| w.1) {
                            errors.push(format!("{unit}: items are party picks."));
                        }
                    }
                    ScriptPick::Move(c) | ScriptPick::StepAfter(c) => {
                        if !on_grid(*c) {
                            errors.push(off_grid(*c));
                        }
                        if matches!(p, ScriptPick::Move(_)) && i > 0 {
                            errors.push("A move is the first pick.".into());
                        }
                    }
                    ScriptPick::Rest => {}
                }
            }
        }
    }
    errors
}

/// ENCOUNTER2-C 5.3: a catalog encounter's script plays only when the Arena picks the timeline ruleset.
pub fn script_warnings(e: &Dict) -> Vec<String> {
    if json::arr(json::get(e, "script")).is_empty() {
        Vec::new()
    } else {
        vec!["Plays scripted only under the timeline ruleset.".into()]
    }
}
