//! Encounter rows: ContentCatalog._encounter_errors with the shared encounter validators (EncounterPhases,
//! EncounterObjectives, EncounterMorale, PurseRules: `problems`, also for world encounters) and the warnings (the
//! PACE1 boss without an enrage, the PURSE1 purse).

use super::View;
use crate::source::content;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_battle::encounter::{ARRIVAL, ARRIVAL_BLOCK, ARRIVAL_ROUND};
use rpgkit_battle::grid::{terrain_walkable, Grid, TerrainTable, GRID_H, GRID_W, V2};
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
    errors.extend(phase_problems(e, &|r: &str| {
        let target = v.get("units", r);
        !target.is_empty() && super::passes_filter(wave_field(false), target)
    }));
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
    errors
}

/// ContentCatalog.warnings_for("encounters"): EncounterPhases.pace_warnings and PurseRules.warnings.
pub fn warnings(v: &View, e: &Dict) -> Vec<String> {
    let mut out = pace_warnings(e, &|r: &str| json::s(&content("units", v.get("units", r)), "tier"));
    out.extend(purse_warnings(e));
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
const ROUND_MIN: i64 = 2;
const STATUS_ROUNDS_MIN: i64 = 1;
const STATUS_ROUNDS_MAX: i64 = 5;
const DEFAULT_STATUS_ROUNDS: i64 = 2;
const ENRAGE_POWER_MIN: i64 = 1;
const ENRAGE_POWER_MAX: i64 = 25;
const DEFAULT_ENRAGE_POWER: i64 = 20;
const EFFECTS: [&str; 5] = ["line", "wave", "status", "ai", "enrage"];
const PHASE_KEYS: [&str; 5] = ["id", "name", "trigger", "effects", "pause_autoplay"];
const AI_ROLES: [&str; 4] = ["AGGRESSOR", "SKIRMISHER", "SNIPER", "SUPPORT"];
const AI_TARGET_PREFS: [&str; 2] = ["NEAREST", "WEAKEST"];
const AI_RATIOS: [&str; 3] = ["flee_ratio", "defend_below", "heal_below"];
const AI_SETTINGS: [&str; 6] = ["role", "target_pref", "reckless", "flee_ratio", "defend_below", "heal_below"];
const PACES: [&str; 4] = ["auto", "mob", "elite", "boss"];

fn trigger_keys(kind: &str) -> &'static [&'static str] {
    match kind {
        "hp_below" => &["kind", "unit", "pct", "or_round"],
        ARRIVAL => &["kind", "round", "block"],
        _ => &["kind", "round"],
    }
}

fn effect_keys(kind: &str) -> &'static [&'static str] {
    match kind {
        "line" => &["type", "text"],
        "wave" => &["type", "units", "label"],
        "status" => &["type", "unit", "status", "rounds", "magnitude"],
        "ai" => &["type", "unit", "role", "target_pref", "reckless", "flee_ratio", "defend_below", "heal_below"],
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
/// arrival's refs are checked as that field, `wave_field`).
pub fn phase_problems(e: &Dict, ref_ok: &dyn Fn(&str) -> bool) -> Vec<String> {
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
        let arrival = is_arrival(phase);
        errors.extend(trigger_errors(&tag, json::get(phase, "trigger"), &enemies, max_rounds, total_blocks));
        errors.extend(effects_errors(&tag, json::get(phase, "effects"), &known, ref_ok, arrival));
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
    if kind != "hp_below" && kind != "round" && kind != ARRIVAL {
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
    } else if !in_range(json::get(t, "round"), ROUND_MIN, max_rounds) {
        errors.push(format!("Phase '{tag}': round must be {ROUND_MIN}..{max_rounds}."));
    }
    errors
}

fn effects_errors(tag: &str, raw: &Value, known: &[String], ref_ok: &dyn Fn(&str) -> bool, arrival: bool) -> Vec<String> {
    let mut errors = Vec::new();
    let list = json::arr(raw);
    if list.is_empty() || list.len() > MAX_EFFECTS {
        errors.push(format!("Phase '{tag}' needs 1 to {MAX_EFFECTS} effects."));
    }
    let (mut lines, mut waves) = (0, 0);
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
            _ => {}
        }
    }
    if lines > 1 {
        errors.push(format!("Phase '{tag}': one line per phase."));
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
                    grid.apply_zone(&json::s(e, "zone"));
                    let tile_ok = |c: V2| {
                        if !on_grid(c) || !terrain_walkable(grid.terrain_id(c)) {
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
