//! Combat hook fields (HOOKS1-S, docs/specs/HOOKS1_TASKS.md 1.1-1.3 and 1.6; engine-only, no Godot original): the
//! checks of a skill's `forced_move`, `hazard`, `triggers`, `riders` and `meters`, a consumable's `hazard`, the passive
//! and item effects `trigger`, `forced_move` and `meter`, a unit's and a phase's `tactics`, an encounter's
//! `arena_hazards` and phase effect `deck`, and a world manifest's `classes`. Every field defaults to off, so a row
//! that sets none of them reports nothing (classic data never changes). The battle reads them only under their rules
//! (`rpgkit_rules::ruleset`, HOOKS1 4.1).

use super::View;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_battle::grid::{GRID_H, GRID_W};
use rpgkit_data::{
    ALLY_EVENTS, ARENA_DIRS, ARENA_EFFECT_KEYS, ARENA_HAZARD_KEYS, CLASS_OVERLAY_KEYS, DECK_EFFECT, EFFECT_TO, FORCED_FROM, FORCED_IMMUNE, FORCED_KINDS, FORCED_MOVE_KEYS, FORCED_WHEN, GRANT_UNTIL,
    HAZARD_AT, HAZARD_METER_KEYS, HAZARD_PLACE_KEYS, HAZARD_SHAPES, HIT_EVENTS, MARK, METER_SET, METER_SET_KEYS, RIDER_KEYS, ROLL_ROW_KEYS, ROLL_TABLE, ROLL_WEIGHT, STRIKE_NEAREST, TACTICS, TRIGGER_EFFECTS,
    TRIGGER_EVENTS, TRIGGER_KEYS, TRIGGER_LIMITS, WHEN_KEYS,
};
use rpgkit_rules::ruleset;
use rpgkit_rules::status;
use serde_json::Value;

/// FM1: tiles a skill pushes or pulls; a collision's multiplier and stagger.
pub const FORCED_TILES: (i64, i64) = (1, 3);
pub const COLLIDE_MULT: (f64, f64) = (0.0, 3.0);
pub const COLLIDE_STAGGER: (i64, i64) = (0, 50);
/// FM5: the extra tiles of a `forced_move` effect.
pub const BONUS_TILES: (i64, i64) = (0, 2);
/// GH2: a placement's own duration (0 = the row's).
pub const PLACE_ROUNDS: (i64, i64) = (0, 5);
/// 1.1: rows of a skill's `triggers` and `riders`; entries of its `meters` and their delta.
pub const SKILL_ROWS_MAX: usize = 3;
pub const METER_COSTS_MAX: usize = 2;
pub const METER_COST: (i64, i64) = (-100, 100);
/// HC1: a skill's HP cost (0 = none).
pub const HP_COST: (i64, i64) = (0, 99);
/// 1.3: effects of a trigger row.
pub const EFFECTS_MAX: usize = 4;
/// 8.2: the ranges of the conditions and effects.
pub const PCT: (i64, i64) = (1, 99);
pub const RANGE: (i64, i64) = (1, 6);
pub const AP: (i64, i64) = (1, 4);
pub const GRANT_ROUNDS: (i64, i64) = (1, 5);
pub const POTENCY: (i64, i64) = (1, 100);
pub const MITIGATION: (f64, f64) = (0.1, 0.9);
pub const GUARD_BLOCKS: (i64, i64) = (0, 6);
pub const REDIRECT: (i64, i64) = (1, 100);
pub const METER_DELTA: (i64, i64) = (-10, 10);
pub const CRIT: (i64, i64) = (1, 100);
pub const RESIST: (i64, i64) = (1, 100);
pub const STAGGER: (i64, i64) = (1, 100);
pub const DAMAGE: (i64, i64) = (1, 99);
/// PK1-S1 (PK1_CORRUPTION 3.2-3.6): an `ap` `set` and its rounds, a `damage` `pct_of_hit`, a `strike_nearest` power and
/// its status rounds, a `mark` id's length and value, a hazard `meter` delta.
pub const AP_SET: (i64, i64) = (0, 4);
pub const AP_SET_ROUNDS: (i64, i64) = (1, 5);
pub const PCT_OF_HIT: (i64, i64) = (1, 100);
pub const STRIKE_POWER: (f64, f64) = (0.1, 3.0);
pub const MARK_ID_MAX: usize = 24;
pub const MARK_VALUE: (i64, i64) = (1, 99);
pub const HAZARD_METER_DELTA: (i64, i64) = (-100, 100);
/// PK1-S2a (PK1 4.1, 4.3 item 5): a skill's `channel.cost` and `channel.power`, its `hp_cost_pct`.
pub const CHANNEL_COST: (i64, i64) = (0, 100);
pub const CHANNEL_POWER: (f64, f64) = (1.0, 3.0);
pub const HP_COST_PCT: (i64, i64) = (0, 99);
/// HEALPCT1 (FB-177): a SUPPORT skill's `heal_pct` (0 = none; 100 = the target's full max HP).
pub const HEAL_PCT: (i64, i64) = (0, 100);
/// AH1: arena hazards per encounter, the name and line lengths, the effect numbers.
pub const ARENA_MAX: usize = 4;
pub const ARENA_NAME_MAX: usize = 24;
pub const ARENA_LINE_MAX: usize = 160;
pub const ARENA_BLOCKS_MAX: i64 = 6;
pub const ARENA_DAMAGE: (i64, i64) = (0, 99);
pub const ARENA_DAMAGE_PCT: (i64, i64) = (0, 50);
pub const ARENA_STAGGER: (i64, i64) = (0, 50);
pub const ARENA_COLLIDE: (i64, i64) = (0, 50);
/// CN1: a class overlay's name and description lengths, a path level.
pub const CLASS_NAME_MAX: usize = 32;
pub const CLASS_DESCRIPTION_MAX: usize = 240;
pub const PATH_LEVEL: (i64, i64) = (1, 30);

/// What the hook checks read from the rest of the catalog: the skill families (tier-one skill ids), the unit meter
/// rows (id, min, max) and the ground hazard ids.
#[derive(Debug, Clone, Default)]
pub struct Ctx {
    pub families: Vec<String>,
    /// U3.2-S2: the gear tokens (`skill::gear_tokens` of the family table; a passive's `proficiency.gear`).
    pub gear: Vec<String>,
    pub meters: Vec<(String, i64, i64)>,
    /// U1.2-S5: each meter's `delta_max` (a `meter` effect's change on it lies within it).
    pub meter_deltas: Vec<(String, i64)>,
    pub hazards: Vec<String>,
}

impl Ctx {
    /// From loaded skill rows (flat rank rows), unit meter rows and hazard rows.
    pub fn from_rows(skills: &[Dict], meters: &[Dict], hazards: &[Dict]) -> Self {
        Self {
            families: skills.iter().filter(|s| json::i(s, "tier", 1) <= 1).map(|s| json::s(s, "id")).collect(),
            gear: super::skill::gear_tokens(&[]),
            meters: meters.iter().map(|m| (json::s(m, "id"), json::i(m, "min", 0), json::i(m, "max", 10))).collect(),
            // a `delta_max` out of range is the row's own error: its effects check against the default meanwhile
            meter_deltas: meters
                .iter()
                .map(|m| (json::s(m, "id"), Some(json::i(m, "delta_max", i64::from(rpgkit_data::METER_DELTA_MAX))).filter(|d| (1..=i64::from(rpgkit_data::METER_DELTA_CAP)).contains(d)).unwrap_or(METER_DELTA.1)))
                .collect(),
            hazards: hazards.iter().map(|h| json::s(h, "id")).collect(),
        }
    }

    /// From the checked catalog.
    pub fn of(v: &View) -> Self {
        let mut ctx = Self::from_rows(v.entries("skills"), v.entries(rpgkit_data::UNIT_METERS_KIND), v.entries(rpgkit_data::HAZARDS_KIND));
        ctx.gear = super::skill::gear_tokens(v.entries("item_families"));
        ctx
    }

    fn meter(&self, id: &str) -> Option<(i64, i64)> {
        self.meters.iter().find(|m| m.0 == id).map(|m| (m.1, m.2))
    }

    /// U1.2-S5: the range of a `meter` effect's delta on meter `id` (its row's `delta_max` either way; the default
    /// METER_DELTA when the row sets none).
    fn meter_delta(&self, id: &str) -> (i64, i64) {
        self.meter_deltas.iter().find(|m| m.0 == id).map_or(METER_DELTA, |m| (-m.1, m.1))
    }
}

/// A whole number in `lo..=hi` (the loader reads every number as a float).
pub fn whole_in(v: &Value, (lo, hi): (i64, i64)) -> bool {
    v.as_f64().is_some_and(|x| x.fract() == 0.0 && (lo as f64..=hi as f64).contains(&x))
}

fn number_in(v: &Value, (lo, hi): (f64, f64)) -> bool {
    v.as_f64().is_some_and(|x| (lo..=hi).contains(&x))
}

fn chars(s: &str) -> usize {
    s.chars().count()
}

/// Unknown keys of `o` (not in `allowed`), one message each.
fn unknown_keys(o: &Dict, allowed: &[&str], at: &str, out: &mut Vec<String>) {
    for k in o.keys().filter(|k| !allowed.contains(&k.as_str())) {
        out.push(format!("{at}: unknown key '{k}' (keys: {}).", allowed.join(", ")));
    }
}

/// A whole-number key: required (`need`) or optional, in range.
fn whole_key(o: &Dict, key: &str, range: (i64, i64), need: bool, at: &str, out: &mut Vec<String>) {
    match o.get(key) {
        None if need => out.push(format!("{at}: {key} is required ({}-{}).", range.0, range.1)),
        Some(v) if !whole_in(v, range) => out.push(format!("{at}: {key} must be a whole number {}-{}.", range.0, range.1)),
        _ => {}
    }
}

fn bool_key(o: &Dict, key: &str, at: &str, out: &mut Vec<String>) {
    if o.get(key).is_some_and(|b| !b.is_boolean()) {
        out.push(format!("{at}: {key} must be true or false."));
    }
}

/// An enum key: required or optional, one of `options`.
fn enum_key(o: &Dict, key: &str, options: &[&str], need: bool, at: &str, out: &mut Vec<String>) {
    match o.get(key) {
        None if need => out.push(format!("{at}: {key} is required ({}).", options.join(", "))),
        Some(v) if !v.as_str().is_some_and(|s| options.contains(&s)) => out.push(format!("{at}: {key} '{}' is not one of {}.", gstr(v), options.join(", "))),
        _ => {}
    }
}

/// A status magnitude: in the status's range, 0 (or absent) for a status without one.
fn magnitude_key(o: &Dict, st: &str, at: &str, out: &mut Vec<String>) {
    let m = o.get("magnitude").cloned().unwrap_or(json::int(0));
    match status::row_of(st).filter(|r| r.magnitude_max > 0) {
        Some(r) if !whole_in(&m, (r.magnitude_min as i64, r.magnitude_max as i64)) => out.push(format!("{at}: {st} needs a magnitude {}-{}.", r.magnitude_min, r.magnitude_max)),
        None if !whole_in(&m, (0, 0)) => out.push(format!("{at}: {st} takes no magnitude (0).")),
        _ => {}
    }
}

/// An object field that holds off as `{}`: Some(object) when set, None when empty; a non-object is reported.
fn set_object<'a>(v: &'a Value, label: &str, out: &mut Vec<String>) -> Option<&'a Dict> {
    match v {
        Value::Object(o) if o.is_empty() => None,
        Value::Object(o) => Some(o),
        _ => {
            out.push(format!("{label} must be an object ({{}} = none)."));
            None
        }
    }
}

/// A list field that holds off as `[]`: the entries, or a message when it is not a list.
fn list_of<'a>(v: &'a Value, label: &str, max: usize, out: &mut Vec<String>) -> &'a [Value] {
    match v {
        Value::Array(a) => {
            if a.len() > max {
                out.push(format!("{label} holds at most {max} rows."));
            }
            a
        }
        Value::Null => &[],
        _ => {
            out.push(format!("{label} must be a list."));
            &[]
        }
    }
}

// ---- Skills (1.1) ------------------------------------------------------------------------------------------------

/// FM1: a skill's `forced_move` (`has_area`: the skill has an area, which `from` center needs).
pub fn forced_move_errors(raw: &Value, has_area: bool) -> Vec<String> {
    let mut out = Vec::new();
    let at = "Forced move";
    let Some(o) = set_object(raw, at, &mut out) else { return out };
    unknown_keys(o, &FORCED_MOVE_KEYS, at, &mut out);
    enum_key(o, "kind", &FORCED_KINDS, true, at, &mut out);
    whole_key(o, "tiles", FORCED_TILES, true, at, &mut out);
    enum_key(o, "from", &FORCED_FROM, false, at, &mut out);
    if json::s(o, "from") == "center" && !has_area {
        out.push(format!("{at}: from center needs an area attack (the area's aim cell)."));
    }
    enum_key(o, "when", &FORCED_WHEN, false, at, &mut out);
    for key in ["stop_moves", "collide_break"] {
        bool_key(o, key, at, &mut out);
    }
    if o.get("collide_mult").is_some_and(|v| !number_in(v, COLLIDE_MULT)) {
        out.push(format!("{at}: collide_mult must be {:.1}-{:.1}.", COLLIDE_MULT.0, COLLIDE_MULT.1));
    }
    whole_key(o, "collide_stagger", COLLIDE_STAGGER, false, at, &mut out);
    out
}

/// GH2: a skill's or consumable's `hazard`: a known hazard row, a shape (SKILL only on an area skill: `skill_area`),
/// where and how long.
pub fn hazard_place_errors(raw: &Value, ctx: &Ctx, skill_area: bool) -> Vec<String> {
    let mut out = Vec::new();
    let at = "Leaves hazard";
    let Some(o) = set_object(raw, at, &mut out) else { return out };
    unknown_keys(o, &HAZARD_PLACE_KEYS, at, &mut out);
    let id = json::s(o, "id");
    if id.is_empty() {
        out.push(format!("{at}: id is required (a {} row).", rpgkit_data::HAZARDS_KIND));
    } else if !ctx.hazards.contains(&id) {
        out.push(format!("{at}: unknown hazard '{id}'."));
    }
    enum_key(o, "shape", &HAZARD_SHAPES, false, at, &mut out);
    if json::s(o, "shape") == "SKILL" && !skill_area {
        out.push(format!("{at}: shape SKILL needs an area attack (use NONE, PLUS, BURST, LINE, RING or ARC)."));
    }
    enum_key(o, "at", &HAZARD_AT, false, at, &mut out);
    whole_key(o, "rounds", PLACE_ROUNDS, false, at, &mut out);
    out
}

/// UM2: a skill's `meters`: up to 2 entries, a known meter each (once), a delta in range and not 0.
pub fn meter_costs_errors(raw: &Value, ctx: &Ctx) -> Vec<String> {
    let mut out = Vec::new();
    let list = list_of(raw, "Meter costs", METER_COSTS_MAX, &mut out);
    let mut seen: Vec<String> = Vec::new();
    for (i, item) in list.iter().enumerate() {
        let at = format!("Meter cost {}", i + 1);
        let Some(o) = item.as_object() else {
            out.push(format!("{at} must be an object {{meter, delta}}."));
            continue;
        };
        unknown_keys(o, &["meter", "delta"], &at, &mut out);
        let meter = json::s(o, "meter");
        if ctx.meter(&meter).is_none() {
            out.push(format!("{at}: unknown unit meter '{meter}'."));
        } else if seen.contains(&meter) {
            out.push(format!("Meter costs list '{meter}' twice."));
        }
        seen.push(meter);
        let delta = json::get(o, "delta");
        if !whole_in(delta, METER_COST) || json::to_int(delta) == 0 {
            out.push(format!("{at}: delta must be a whole number {}..{}, not 0.", METER_COST.0, METER_COST.1));
        }
    }
    out
}

/// 1.1: a skill's `triggers` (up to 3 rows answering the skill events).
pub fn skill_triggers_errors(raw: &Value, ctx: &Ctx) -> Vec<String> {
    let mut out = Vec::new();
    let list = list_of(raw, "Triggers", SKILL_ROWS_MAX, &mut out);
    for (i, item) in list.iter().enumerate() {
        let at = format!("Trigger {}", i + 1);
        match item.as_object() {
            Some(o) => out.extend(trigger_row_errors(o, ctx, &at, &rpgkit_data::SKILL_TRIGGER_EVENTS, &[])),
            None => out.push(format!("{at} must be an object {{on, when, effects, limit}}.")),
        }
    }
    out
}

/// RD1: a skill's `riders` (up to 3 rows of conditions and effects; no event, no redirect).
pub fn riders_errors(raw: &Value, ctx: &Ctx) -> Vec<String> {
    let mut out = Vec::new();
    let list = list_of(raw, "Riders", SKILL_ROWS_MAX, &mut out);
    for (i, item) in list.iter().enumerate() {
        let at = format!("Rider {}", i + 1);
        let Some(o) = item.as_object() else {
            out.push(format!("{at} must be an object {{when, effects}}."));
            continue;
        };
        unknown_keys(o, &RIDER_KEYS, &at, &mut out);
        out.extend(when_errors(json::get(o, "when"), ctx, &at, None));
        out.extend(effects_errors(json::get(o, "effects"), ctx, &at, None, false));
    }
    out
}

/// The hook fields of one flat skill row of `kind` (only the fields the kind has; another kind's field is reported by
/// the belongs-to check).
pub fn skill_errors(v: &View, e: &Dict, kind: &str) -> Vec<String> {
    let mut out = Vec::new();
    let has = |key: &str| super::skill::has_field(kind, key);
    let empty = |key: &str| match json::get(e, key) {
        Value::Object(o) => o.is_empty(),
        Value::Array(a) => a.is_empty(),
        Value::Null => true,
        _ => false,
    };
    // HC1: the schema row carries no min / max, so an out-of-range cost is refused here, not clamped
    let cost = json::i(e, "hp_cost", 0);
    if has("hp_cost") && !(HP_COST.0..=HP_COST.1).contains(&cost) {
        out.push(format!("HP cost must be {}-{} (is {cost}).", HP_COST.0, HP_COST.1));
    }
    // PK1-S2a: the percent HP cost, refused out of range as the flat one
    let pct = json::i(e, "hp_cost_pct", 0);
    if has("hp_cost_pct") && !(HP_COST_PCT.0..=HP_COST_PCT.1).contains(&pct) {
        out.push(format!("HP cost % must be {}-{} (is {pct}).", HP_COST_PCT.0, HP_COST_PCT.1));
    }
    // HEALPCT1: the percent heal, refused out of range as the percent cost
    let heal_pct = json::i(e, "heal_pct", 0);
    if has("heal_pct") && !(HEAL_PCT.0..=HEAL_PCT.1).contains(&heal_pct) {
        out.push(format!("Heal % must be {}-{} (is {heal_pct}).", HEAL_PCT.0, HEAL_PCT.1));
    }
    let keys = ["forced_move", "hazard", "triggers", "riders", "meters", "requires_meter", "channel"];
    if keys.iter().all(|k| !has(k) || empty(k)) {
        return out;
    }
    let ctx = Ctx::of(v);
    let area = kind == "ATTACK" && json::s(e, "area") != "NONE";
    if has("forced_move") {
        out.extend(forced_move_errors(json::get(e, "forced_move"), area));
    }
    if has("hazard") {
        out.extend(hazard_place_errors(json::get(e, "hazard"), &ctx, area));
    }
    if has("triggers") {
        out.extend(skill_triggers_errors(json::get(e, "triggers"), &ctx));
    }
    if has("riders") {
        out.extend(riders_errors(json::get(e, "riders"), &ctx));
    }
    if has("meters") {
        out.extend(meter_costs_errors(json::get(e, "meters"), &ctx));
    }
    if has("requires_meter") {
        out.extend(meter_gate_errors(json::get(e, "requires_meter"), &ctx));
    }
    if has("channel") {
        out.extend(channel_errors(json::get(e, "channel"), e, &ctx));
    }
    out
}

/// PK1-S2a (PK1 4.3 item 5): a skill's `requires_meter` ({} = none): `{meter, min}`, a known meter and a whole `min`
/// within its row's range.
pub fn meter_gate_errors(raw: &Value, ctx: &Ctx) -> Vec<String> {
    let mut out = Vec::new();
    let Some(o) = set_object(raw, "Meter gate", &mut out) else { return out };
    let at = "Meter gate";
    unknown_keys(o, &rpgkit_data::MeterGate::KEYS, at, &mut out);
    let meter = json::s(o, "meter");
    match ctx.meter(&meter) {
        None => out.push(format!("{at}: unknown unit meter '{meter}'.")),
        Some(range) => whole_key(o, "min", range, true, at, &mut out),
    }
    out
}

/// PK1-S2a (PK1 4.1, 4.2): a skill's `channel` block ({} = the default: a derived marrow version when it has a single
/// intent): its keys, `none` and `only` (true or false, not both; `none` with nothing else), `cost` a whole 0-100,
/// `power` 1.0-3.0, `effects` 1-4 trigger effects (as a rider's: no event); any key but `none` needs a single-intent
/// skill (IR21: a dual-intent skill has no marrow version, and a marrow-only one is authored with one intent).
pub fn channel_errors(raw: &Value, e: &Dict, ctx: &Ctx) -> Vec<String> {
    let mut out = Vec::new();
    let Some(o) = set_object(raw, "Channel", &mut out) else { return out };
    let at = "Channel";
    unknown_keys(o, &rpgkit_data::ChannelSpec::KEYS, at, &mut out);
    bool_key(o, "none", at, &mut out);
    bool_key(o, "only", at, &mut out);
    let none = json::b(o, "none", false);
    if none && json::b(o, "only", false) {
        out.push(format!("{at}: none and only exclude each other."));
    }
    if none && ["cost", "power", "effects"].iter().any(|k| o.contains_key(*k)) {
        out.push(format!("{at}: none (no marrow version) takes no cost, power or effects."));
    }
    whole_key(o, "cost", CHANNEL_COST, false, at, &mut out);
    if o.get("power").is_some_and(|v| !number_in(v, CHANNEL_POWER)) {
        out.push(format!("{at}: power must be {}-{}.", CHANNEL_POWER.0, CHANNEL_POWER.1));
    }
    if o.contains_key("effects") {
        out.extend(effects_errors(json::get(o, "effects"), ctx, "Channel", None, false));
    }
    let input = rpgkit_rules::intent::IntentInput::of_row(e);
    if !none && !rpgkit_rules::intent::single_intent(&input) {
        let rows = rpgkit_rules::intent::derive(&input).join(" and ");
        out.push(format!("{at}: only a single-intent skill has a marrow version or is marrow-only (this one derives {rows})."));
    }
    out
}

/// 1.1 warnings of one flat skill row: a placement (an ATTACK of power 0 with a hazard) whose status, stagger or forced
/// move never land.
pub fn placement_warnings(e: &Dict) -> Vec<String> {
    let placement = json::s(e, "kind") == "ATTACK" && json::f(e, "power_mult", 1.0) <= 0.0 && json::get(e, "hazard").as_object().is_some_and(|h| !h.is_empty());
    let mut parts = Vec::new();
    if placement && !json::s(e, "status_effect").is_empty() {
        parts.push("status");
    }
    if placement && json::i(e, "stagger", 0) > 0 {
        parts.push("stagger");
    }
    if placement && json::get(e, "forced_move").as_object().is_some_and(|f| !f.is_empty()) {
        parts.push("forced move");
    }
    if parts.is_empty() {
        Vec::new()
    } else {
        vec![format!("A placement (power 0 with a hazard) lands no hit, so these never apply: {}.", parts.join(", "))]
    }
}

// ---- Trigger rows (1.3) ------------------------------------------------------------------------------------------

/// One trigger row (a skill's, or a passive or item `trigger` effect with `extra` keys such as `type`), answering one
/// of `events`.
pub fn trigger_row_errors(o: &Dict, ctx: &Ctx, at: &str, events: &[&str], extra: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let allowed: Vec<&str> = TRIGGER_KEYS.iter().chain(extra.iter()).copied().collect();
    unknown_keys(o, &allowed, at, &mut out);
    let on = json::s(o, "on");
    if on.is_empty() {
        out.push(format!("{at}: on is required ({}).", events.join(", ")));
    } else if !events.contains(&on.as_str()) {
        out.push(format!("{at}: on '{on}' is not one of {}.", events.join(", ")));
    }
    let event = events.contains(&on.as_str()).then_some(on.as_str());
    out.extend(when_errors(json::get(o, "when"), ctx, at, Some(event.unwrap_or(""))));
    out.extend(effects_errors(json::get(o, "effects"), ctx, at, Some(event.unwrap_or("")), false));
    enum_key(o, "limit", &TRIGGER_LIMITS, false, at, &mut out);
    out
}

/// 8.2: a `when` (absent or an object). `event` is the row's event (Some("") when unknown), None on a rider.
pub fn when_errors(raw: &Value, ctx: &Ctx, at: &str, event: Option<&str>) -> Vec<String> {
    let mut out = Vec::new();
    let o = match raw {
        Value::Null => {
            if event == Some("hp_below") {
                out.push(format!("{at}: an hp_below trigger needs when.pct ({}-{}).", PCT.0, PCT.1));
            }
            return out;
        }
        Value::Object(o) => o,
        _ => {
            out.push(format!("{at}: when must be an object."));
            return out;
        }
    };
    let at = format!("{at} when");
    unknown_keys(o, &WHEN_KEYS, &at, &mut out);
    match (event, o.get("pct")) {
        (Some("hp_below"), _) => whole_key(o, "pct", PCT, true, &at, &mut out),
        (_, Some(_)) => out.push(format!("{at}: pct is for an hp_below trigger only.")),
        _ => {}
    }
    if o.contains_key("range") {
        if event.is_some_and(|e| ALLY_EVENTS.contains(&e)) {
            whole_key(o, "range", RANGE, false, &at, &mut out);
        } else {
            out.push(format!("{at}: range is for {} only.", ALLY_EVENTS.join(" and ")));
        }
    }
    for key in ["ranged", "melee", "other_broken"] {
        bool_key(o, key, &at, &mut out);
    }
    if json::b(o, "ranged", false) && json::b(o, "melee", false) {
        out.push(format!("{at}: ranged and melee together never match."));
    }
    match o.get("skills") {
        None => {}
        Some(Value::Array(list)) => {
            for s in list {
                if !s.as_str().is_some_and(|s| ctx.families.iter().any(|f| f == s)) {
                    out.push(format!("{at}: skills: unknown skill family '{}'.", gstr(s)));
                }
            }
        }
        Some(_) => out.push(format!("{at}: skills must be a list of skill families.")),
    }
    whole_key(o, "other_hp_below", PCT, false, &at, &mut out);
    if let Some(st) = o.get("other_status") {
        if !st.as_str().is_some_and(|s| status::row_of(s).is_some()) {
            out.push(format!("{at}: other_status: unknown status '{}'.", gstr(st)));
        }
    }
    enum_key(o, "intent", &ruleset::INTENTS, false, &at, &mut out);
    if let Some(m) = o.get("meter") {
        match m.as_object() {
            Some(m) => {
                unknown_keys(m, &["id", "at_least"], &format!("{at} meter"), &mut out);
                let id = json::s(m, "id");
                match ctx.meter(&id) {
                    None => out.push(format!("{at}: meter: unknown unit meter '{id}'.")),
                    Some(range) => whole_key(m, "at_least", range, true, &format!("{at} meter"), &mut out),
                }
            }
            None => out.push(format!("{at}: meter must be an object {{id, at_least}}.")),
        }
    }
    out
}

/// 1.3: an `effects` list of 1-4 trigger effects. `event`: the row's event (Some("") unknown), None on a rider (no
/// redirect); `meter_set`: a meter rule's list (it also takes `meter_set`).
pub fn effects_errors(raw: &Value, ctx: &Ctx, at: &str, event: Option<&str>, meter_set: bool) -> Vec<String> {
    let mut out = Vec::new();
    let list = match raw {
        Value::Array(a) => a.as_slice(),
        _ => {
            out.push(format!("{at}: effects must be a list of 1-{EFFECTS_MAX} effects."));
            return out;
        }
    };
    if list.is_empty() || list.len() > EFFECTS_MAX {
        out.push(format!("{at}: effects holds 1-{EFFECTS_MAX} effects."));
    }
    let mut redirects = 0;
    for (i, item) in list.iter().enumerate() {
        let at = format!("{at} effect {}", i + 1);
        let Some(o) = item.as_object() else {
            out.push(format!("{at} must be an object {{type, ...}}."));
            continue;
        };
        let t = json::s(o, "type");
        // PK1-S1 (3.2): a share of the hit needs an event with a hit (a rider rides on one)
        if t == "damage" && o.contains_key("pct_of_hit") && event.is_some_and(|e| !e.is_empty() && !HIT_EVENTS.contains(&e)) {
            out.push(format!("{at}: pct_of_hit needs a hit event ({}) or a rider.", HIT_EVENTS.join(", ")));
        }
        // PK1-S1 (3.2): a rider's effect acts on the unit it landed on unless it says `to: self`; recoil is the holder's
        if t == "damage" && o.contains_key("pct_of_hit") && event.is_none() && !o.contains_key("to") {
            out.push(format!("{at}: pct_of_hit is recoil on the holder: a rider sets to: self."));
        }
        if t == "redirect" {
            redirects += 1;
            match event {
                None => out.push(format!("{at}: a rider cannot redirect.")),
                Some(e) if e != "ally_hit" => out.push(format!("{at}: redirect needs an ally_hit trigger.")),
                _ => {}
            }
        }
        // ROLLTABLE1-GUARD (FB-169 (5) A): on ally_hit only a top-level guard shapes the hit; a table rolls after it
        if t == ROLL_TABLE && event == Some("ally_hit") && table_guards(o) {
            out.push(format!("{at}: a roll_table on ally_hit cannot guard (it rolls after the hit); put guard at the top level."));
        }
        out.extend(effect_errors(o, &t, ctx, &at, meter_set));
    }
    if redirects > 1 {
        out.push(format!("{at}: one redirect per row."));
    }
    out
}

/// One trigger effect of type `t` (8.2; `meter_set` too on a meter rule).
fn effect_errors(o: &Dict, t: &str, ctx: &Ctx, at: &str, meter_set: bool) -> Vec<String> {
    let mut out = Vec::new();
    let keys: &[&str] = match TRIGGER_EFFECTS.iter().find(|(k, _)| *k == t) {
        Some((_, keys)) => keys,
        None if meter_set && t == METER_SET => &METER_SET_KEYS,
        None => {
            let mut names: Vec<&str> = TRIGGER_EFFECTS.iter().map(|(k, _)| *k).collect();
            if meter_set {
                names.push(METER_SET);
            }
            out.push(format!("{at}: unknown effect type '{t}' (use {}).", names.join(", ")));
            return out;
        }
    };
    let allowed: Vec<&str> = std::iter::once("type").chain(keys.iter().copied()).collect();
    unknown_keys(o, &allowed, at, &mut out);
    enum_key(o, "to", &EFFECT_TO, false, at, &mut out);
    let st = json::s(o, "status");
    match t {
        // PK1-S1 (3.4): `set` (with `rounds`) instead of `value`
        "ap" if o.contains_key("set") => {
            if o.contains_key("value") {
                out.push(format!("{at}: ap takes value or set, not both."));
            }
            whole_key(o, "set", AP_SET, true, at, &mut out);
            whole_key(o, "rounds", AP_SET_ROUNDS, false, at, &mut out);
        }
        "ap" => {
            whole_key(o, "value", AP, true, at, &mut out);
            if o.contains_key("rounds") {
                out.push(format!("{at}: rounds goes with set."));
            }
        }
        "grant" => {
            if !status::buffs().contains(&st.as_str()) {
                out.push(format!("{at}: grant '{st}' is not a buff ({}).", status::buffs().join(", ")));
            } else {
                magnitude_key(o, &st, at, &mut out);
            }
            let rounds = o.get("rounds").cloned().unwrap_or(json::int(1));
            if !whole_in(&rounds, (-1, -1)) && !whole_in(&rounds, GRANT_ROUNDS) {
                out.push(format!("{at}: rounds must be -1 (the battle) or {}-{}.", GRANT_ROUNDS.0, GRANT_ROUNDS.1));
            }
            enum_key(o, "until", &GRANT_UNTIL, false, at, &mut out);
        }
        "inflict" => {
            if !status::debuffs().contains(&st.as_str()) {
                out.push(format!("{at}: inflict '{st}' is not a debuff ({}).", status::debuffs().join(", ")));
            } else {
                magnitude_key(o, &st, at, &mut out);
            }
            whole_key(o, "potency", POTENCY, true, at, &mut out);
            whole_key(o, "rounds", GRANT_ROUNDS, false, at, &mut out);
        }
        "guard" => {
            if !o.get("mitigation").is_some_and(|m| number_in(m, MITIGATION)) {
                out.push(format!("{at}: guard needs a mitigation {:.1}-{:.1}.", MITIGATION.0, MITIGATION.1));
            }
            whole_key(o, "blocks", GUARD_BLOCKS, false, at, &mut out);
        }
        "redirect" => whole_key(o, "pct", REDIRECT, true, at, &mut out),
        "move" => {
            enum_key(o, "kind", &FORCED_KINDS, true, at, &mut out);
            whole_key(o, "tiles", FORCED_TILES, true, at, &mut out);
        }
        "meter" | METER_SET => {
            let meter = json::s(o, "meter");
            match ctx.meter(&meter) {
                None => out.push(format!("{at}: unknown unit meter '{meter}'.")),
                Some(range) if t == METER_SET => whole_key(o, "value", range, true, at, &mut out),
                Some(_) => {
                    let delta = json::get(o, "delta");
                    let range = ctx.meter_delta(&meter);
                    if !whole_in(delta, range) || json::to_int(delta) == 0 {
                        out.push(format!("{at}: delta must be a whole number {}..{}, not 0.", range.0, range.1));
                    }
                }
            }
        }
        "crit" => whole_key(o, "value", CRIT, true, at, &mut out),
        "resist" => {
            let resistable: Vec<&str> = status::ROWS.iter().filter(|r| !r.buff && r.resistable).map(|r| r.id).collect();
            if !resistable.contains(&st.as_str()) {
                out.push(format!("{at}: resist '{st}' is not a resistable status ({}).", resistable.join(", ")));
            }
            whole_key(o, "value", RESIST, true, at, &mut out);
        }
        "stagger" => whole_key(o, "value", STAGGER, true, at, &mut out),
        // PK1-S1 (3.2): a flat `value` or `pct_of_hit`, a share of the event's hit (recoil)
        "damage" if o.contains_key("pct_of_hit") => {
            if o.contains_key("value") {
                out.push(format!("{at}: damage takes value or pct_of_hit, not both."));
            }
            whole_key(o, "pct_of_hit", PCT_OF_HIT, true, at, &mut out);
            if json::s(o, "to") == "other" {
                out.push(format!("{at}: pct_of_hit is recoil on the holder (to: self)."));
            }
        }
        "damage" => whole_key(o, "value", DAMAGE, true, at, &mut out),
        STRIKE_NEAREST => {
            if o.get("power").is_some_and(|p| !number_in(p, STRIKE_POWER)) {
                out.push(format!("{at}: power must be a number {:.1}-{:.1}.", STRIKE_POWER.0, STRIKE_POWER.1));
            }
            let then = json::s(o, "then_status");
            if !then.is_empty() && status::row_of(&then).is_none() {
                out.push(format!("{at}: then_status '{then}' is not a status."));
            }
            if o.contains_key("then_rounds") {
                if then.is_empty() {
                    out.push(format!("{at}: then_rounds needs a then_status."));
                }
                whole_key(o, "then_rounds", GRANT_ROUNDS, false, at, &mut out);
            }
        }
        MARK => {
            let id = json::s(o, "mark");
            if id.is_empty() || chars(&id) > MARK_ID_MAX || !aegis_content::schema::is_id(&id) {
                out.push(format!("{at}: mark needs an id of 1-{MARK_ID_MAX} characters (a-z, 0-9, _, a letter first)."));
            }
            whole_key(o, "value", MARK_VALUE, false, at, &mut out);
        }
        "reveal" => whole_key(o, "rounds", (1, 1), false, at, &mut out),
        ROLL_TABLE => out.extend(roll_table_errors(o, ctx, at, meter_set)),
        _ => {}
    }
    out
}

/// ROLLTABLE1-GUARD: whether any row of the table `o` holds a `guard` effect.
fn table_guards(o: &Dict) -> bool {
    let Some(Value::Array(rows)) = o.get("rows") else { return false };
    rows.iter()
        .filter_map(|r| r.get("effects").and_then(Value::as_array))
        .flatten()
        .any(|e| e.as_object().is_some_and(|e| json::s(e, "type") == "guard"))
}

/// ROLLTABLE1: a `roll_table`'s `rows`: at least one, each `{weight, effects}` with a whole weight in ROLL_WEIGHT and
/// 1-EFFECTS_MAX trigger effects (as its list's: `meter_set` on a meter rule), none of them a `roll_table` or a
/// `redirect` (a hit is claimed before any table rolls).
fn roll_table_errors(o: &Dict, ctx: &Ctx, at: &str, meter_set: bool) -> Vec<String> {
    let mut out = Vec::new();
    let rows = match o.get("rows") {
        Some(Value::Array(a)) if !a.is_empty() => a,
        _ => {
            out.push(format!("{at}: roll_table needs rows, a list of 1 or more {{weight, effects}}."));
            return out;
        }
    };
    for (r, row) in rows.iter().enumerate() {
        let at = format!("{at} row {}", r + 1);
        let Some(row) = row.as_object() else {
            out.push(format!("{at} must be an object {{weight, effects}}."));
            continue;
        };
        unknown_keys(row, &ROLL_ROW_KEYS, &at, &mut out);
        whole_key(row, "weight", ROLL_WEIGHT, true, &at, &mut out);
        let list = match row.get("effects") {
            Some(Value::Array(a)) if !a.is_empty() && a.len() <= EFFECTS_MAX => a.as_slice(),
            _ => {
                out.push(format!("{at}: effects holds 1-{EFFECTS_MAX} effects."));
                continue;
            }
        };
        for (i, item) in list.iter().enumerate() {
            let at = format!("{at} effect {}", i + 1);
            let Some(e) = item.as_object() else {
                out.push(format!("{at} must be an object {{type, ...}}."));
                continue;
            };
            let t = json::s(e, "type");
            match t.as_str() {
                ROLL_TABLE => out.push(format!("{at}: a roll_table cannot hold a roll_table.")),
                "redirect" => out.push(format!("{at}: a roll_table cannot redirect.")),
                _ => out.extend(effect_errors(e, &t, ctx, &at, meter_set)),
            }
        }
    }
    out
}

// ---- Passive and item effects (1.2) ------------------------------------------------------------------------------

/// The passive and item effect types HOOKS1 adds (`meter` passives only).
pub const PASSIVE_EFFECTS: [&str; 3] = ["forced_move", "trigger", "meter"];
pub const ITEM_EFFECTS: [&str; 2] = ["forced_move", "trigger"];

/// One passive or item effect of a HOOKS1 type (`at` names it, e.g. "Effect 2").
pub fn passive_effect_errors(e: &Dict, t: &str, ctx: &Ctx, at: &str) -> Vec<String> {
    let mut out = Vec::new();
    match t {
        "forced_move" => {
            unknown_keys(e, &["type", "immune", "bonus_tiles"], at, &mut out);
            enum_key(e, "immune", &FORCED_IMMUNE, false, at, &mut out);
            whole_key(e, "bonus_tiles", BONUS_TILES, false, at, &mut out);
            if json::s(e, "immune").is_empty() && json::to_int(json::get(e, "bonus_tiles")) == 0 {
                out.push(format!("{at}: forced_move sets immune or bonus_tiles."));
            }
        }
        "trigger" => out.extend(trigger_row_errors(e, ctx, at, &TRIGGER_EVENTS, &["type"])),
        "meter" => {
            unknown_keys(e, &["type", "meter", "start", "floor"], at, &mut out);
            let meter = json::s(e, "meter");
            match ctx.meter(&meter) {
                None => out.push(format!("{at}: unknown unit meter '{meter}'.")),
                Some(range) => {
                    if e.get("start").is_some_and(|s| !whole_in(s, (-1, -1)) && !whole_in(s, range)) {
                        out.push(format!("{at}: start must be -1 (the meter's) or {}-{}.", range.0, range.1));
                    }
                    whole_key(e, "floor", range, false, at, &mut out);
                }
            }
        }
        _ => {}
    }
    out
}

// ---- Units and phase ai (1.2) ------------------------------------------------------------------------------------

/// AT2: a `tactics` list: known tactics, no repeats.
pub fn tactics_errors(raw: &Value, label: &str) -> Vec<String> {
    let mut out = Vec::new();
    let Some(list) = raw.as_array() else {
        out.push(format!("{label} must be a list of tactics ({}).", TACTICS.join(", ")));
        return out;
    };
    let mut seen: Vec<String> = Vec::new();
    for t in list.iter().map(gstr) {
        if !TACTICS.contains(&t.as_str()) {
            out.push(format!("{label}: '{t}' is not one of {}.", TACTICS.join(", ")));
        } else if seen.contains(&t) {
            out.push(format!("{label} lists '{t}' twice."));
        }
        seen.push(t);
    }
    out
}

// ---- Encounters (1.2, 1.6) ---------------------------------------------------------------------------------------

/// PK1-S1 (3.6): a hazard's `meter` {meter, delta} (an arena effect's or a ground hazard row's; `{}` or absent = none):
/// a known unit meter (`meters` None: not known yet, the loader's pass) and a whole delta in HAZARD_METER_DELTA, not 0.
pub fn hazard_meter_errors(raw: Option<&Value>, meters: Option<&[String]>, at: &str) -> Vec<String> {
    let mut out = Vec::new();
    let Some(raw) = raw.filter(|v| !v.is_null()) else { return out };
    let label = format!("{at} meter");
    let Some(o) = set_object(raw, &label, &mut out) else { return out };
    unknown_keys(o, &HAZARD_METER_KEYS, &label, &mut out);
    let id = json::s(o, "meter");
    if id.is_empty() {
        out.push(format!("{label}: meter is required (a unit meter id)."));
    } else if meters.is_some_and(|m| !m.contains(&id)) {
        out.push(format!("{label}: unknown unit meter '{id}'."));
    }
    let delta = json::get(o, "delta");
    if !whole_in(delta, HAZARD_METER_DELTA) || json::to_int(delta) == 0 {
        out.push(format!("{label}: delta must be a whole number {}..{}, not 0.", HAZARD_METER_DELTA.0, HAZARD_METER_DELTA.1));
    }
    out
}

/// AH1: an encounter's `arena_hazards` (catalog and world encounters); `meters` the unit meter ids (PK1-S1: an
/// effect's `meter`).
pub fn arena_hazards_errors(e: &Dict, meters: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let list = list_of(json::get(e, "arena_hazards"), "Arena hazards", ARENA_MAX, &mut out);
    let max_rounds = json::to_int(json::get(e, "max_rounds")).max(1);
    let max_block = json::to_int(json::get(e, "total_blocks")).clamp(1, ARENA_BLOCKS_MAX);
    let mut seen: Vec<String> = Vec::new();
    for (i, item) in list.iter().enumerate() {
        let Some(o) = item.as_object() else {
            out.push(format!("Arena hazard {} must be an object.", i + 1));
            continue;
        };
        let id = json::s(o, "id");
        let at = format!("Arena hazard '{}'", if id.is_empty() { (i + 1).to_string() } else { id.clone() });
        unknown_keys(o, &ARENA_HAZARD_KEYS, &at, &mut out);
        if !aegis_content::schema::is_id(&id) {
            out.push(format!("{at}: the id must be lowercase snake_case (a-z, 0-9, _)."));
        } else if seen.contains(&id) {
            out.push(format!("Arena hazard ids must be unique: '{id}'."));
        }
        seen.push(id);
        let name = json::s(o, "name");
        if name.trim().is_empty() || chars(&name) > ARENA_NAME_MAX {
            out.push(format!("{at}: the name needs 1 to {ARENA_NAME_MAX} characters."));
        }
        if o.get("line").is_some_and(|l| !l.as_str().is_some_and(|s| chars(s) <= ARENA_LINE_MAX)) {
            out.push(format!("{at}: the line is text of at most {ARENA_LINE_MAX} characters."));
        }
        let set: Vec<&str> = ["cells", "columns", "rows"].into_iter().filter(|k| o.get(*k).is_some_and(|v| !json::arr(v).is_empty())).collect();
        if set.len() != 1 {
            out.push(format!("{at}: set exactly one of cells, columns or rows."));
        }
        for item in json::arr(json::get(o, "cells")) {
            let pair = json::arr(item);
            let on = pair.len() == 2 && whole_in(&pair[0], (0, GRID_W as i64 - 1)) && whole_in(&pair[1], (0, GRID_H as i64 - 1));
            if !on {
                out.push(format!("{at}: cell {} is not an [x, y] on the {GRID_W}x{GRID_H} board.", gstr(item)));
            }
        }
        for (key, hi) in [("columns", GRID_W as i64 - 1), ("rows", GRID_H as i64 - 1)] {
            if json::arr(json::get(o, key)).iter().any(|x| !whole_in(x, (0, hi))) {
                out.push(format!("{at}: {key} are whole numbers 0-{hi}."));
            }
        }
        if json::arr(json::get(o, "rounds")).iter().any(|x| !whole_in(x, (1, max_rounds))) {
            out.push(format!("{at}: rounds are whole numbers 1-{max_rounds} ([] = every round)."));
        }
        let blocks = json::arr(json::get(o, "blocks"));
        if blocks.is_empty() || blocks.iter().any(|x| !whole_in(x, (1, max_block))) {
            out.push(format!("{at}: blocks lists at least one whole number 1-{max_block}."));
        }
        out.extend(arena_effect_errors(json::get(o, "effect"), &at, meters));
    }
    out
}

/// AH2: an arena hazard's `effect`: at least one of damage, damage_pct, stagger, a status, a push (PK1-S1: or a meter).
fn arena_effect_errors(raw: &Value, at: &str, meters: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let Some(o) = raw.as_object() else {
        out.push(format!("{at}: effect must be an object."));
        return out;
    };
    let at = format!("{at} effect");
    unknown_keys(o, &ARENA_EFFECT_KEYS, &at, &mut out);
    whole_key(o, "damage", ARENA_DAMAGE, false, &at, &mut out);
    whole_key(o, "damage_pct", ARENA_DAMAGE_PCT, false, &at, &mut out);
    whole_key(o, "stagger", ARENA_STAGGER, false, &at, &mut out);
    let positive = |k: &str| json::to_int(json::get(o, k)) > 0;
    if positive("damage") && positive("damage_pct") {
        out.push(format!("{at}: damage and damage_pct together (one of them)."));
    }
    let st = json::s(o, "status");
    if !st.is_empty() {
        if !status::debuffs().contains(&st.as_str()) {
            out.push(format!("{at}: status '{st}' is not a debuff."));
        } else {
            magnitude_key(o, &st, &at, &mut out);
        }
        whole_key(o, "potency", POTENCY, false, &at, &mut out);
        whole_key(o, "status_rounds", GRANT_ROUNDS, false, &at, &mut out);
    } else if ["potency", "magnitude", "status_rounds"].iter().any(|k| o.contains_key(*k)) {
        out.push(format!("{at}: potency, magnitude and status_rounds need a status."));
    }
    let push = o.get("push");
    if let Some(p) = push {
        match p.as_object() {
            Some(p) => {
                let at = format!("{at} push");
                unknown_keys(p, &["dir", "tiles", "collide_damage"], &at, &mut out);
                enum_key(p, "dir", &ARENA_DIRS, true, &at, &mut out);
                whole_key(p, "tiles", FORCED_TILES, true, &at, &mut out);
                whole_key(p, "collide_damage", ARENA_COLLIDE, false, &at, &mut out);
            }
            None => out.push(format!("{at}: push must be an object {{dir, tiles, collide_damage}}.")),
        }
    }
    out.extend(hazard_meter_errors(o.get("meter"), Some(meters), &at));
    let meter = o.get("meter").is_some_and(|m| m.as_object().is_some_and(|m| !m.is_empty()));
    if !positive("damage") && !positive("damage_pct") && !positive("stagger") && st.is_empty() && push.is_none() && !meter {
        out.push(format!("{at}: set at least one of damage, damage_pct, stagger, status, push or meter."));
    }
    out
}

/// The ruleset rows (of the catalog) that list `rule`.
fn rows_with<'a>(v: &'a View, rule: &str) -> Vec<&'a Dict> {
    v.entries(ruleset::KIND).iter().filter(|r| json::sget(r, "rules").iter().any(|x| x == rule)).collect()
}

/// The ruleset row world `w` plays by (its lineage's nearest `ruleset`), None in classic or without the row.
pub fn world_ruleset_row<'a>(v: &'a View, w: &str) -> Option<&'a Dict> {
    let lineage = v.worlds.lineage(w);
    let id = ruleset::world_ruleset(lineage.iter().rev().map(|lw| v.worlds.manifest(lw)));
    v.entries(ruleset::KIND).iter().find(|r| json::s(r, "id") == id)
}

/// Whether ruleset row `row` lists `rule`.
pub fn lists_rule(row: Option<&Dict>, rule: &str) -> bool {
    row.is_some_and(|r| json::sget(r, "rules").iter().any(|x| x == rule))
}

/// DU2: an encounter's phase effects `deck`: under the deck rule (`world`: the world's ruleset, else any ruleset row),
/// `add` families the unit owns (path or extra skills) within every deck ruleset's active slots, `order` entries among
/// the actives or the basics, of that intent. `unit_of(ref)` is the catalog unit a ref plays as.
pub fn deck_effect_errors(v: &View, e: &Dict, world: Option<&str>, unit_of: &dyn Fn(&str) -> Dict) -> Vec<String> {
    let mut out = Vec::new();
    let decks: Vec<(String, &Dict)> = json::aget(e, "phases")
        .iter()
        .filter_map(Value::as_object)
        .flat_map(|p| json::aget(p, "effects").iter().filter_map(Value::as_object).filter(|fx| json::s(fx, "type") == DECK_EFFECT).map(move |fx| (json::s(p, "id"), fx)))
        .collect();
    if decks.is_empty() {
        return out;
    }
    let deck_rows: Vec<&Dict> = match world {
        Some(w) => world_ruleset_row(v, w).filter(|r| lists_rule(Some(r), ruleset::DECK)).into_iter().collect(),
        None => rows_with(v, ruleset::DECK),
    };
    if deck_rows.is_empty() {
        out.push(match world {
            Some(_) => "A deck phase effect needs the deck rule in the world's ruleset.".to_string(),
            None => "A deck phase effect needs a ruleset row with the deck rule.".to_string(),
        });
        return out;
    }
    let basics: Vec<String> = deck_rows.iter().flat_map(|r| json::sget(r, "basics")).collect();
    for (phase, fx) in decks {
        let tag = format!("Phase '{phase}'");
        let unit = unit_of(&json::s(fx, "unit"));
        if unit.is_empty() {
            continue;
        }
        let owned = super::unit::owned_families(v, &unit);
        // CLASS-BASICS: the unit's class basics are basics too
        let basics = rpgkit_rules::unit::DeckRule { basics: basics.clone(), ..Default::default() }.with_class(&v.typed, &json::s(&unit, "class_id")).basics;
        let deck = json::dget(&unit, "deck");
        let mut actives = json::sget(deck, "active");
        for fam in json::aget(fx, "add").iter().map(gstr) {
            if v.typed.tier_one(&fam).is_none() {
                out.push(format!("{tag}: deck add: unknown skill family '{fam}'."));
            } else if !owned.contains(&fam) {
                out.push(format!("{tag}: deck add: the unit does not own the skill family '{fam}'."));
            }
            if !actives.contains(&fam) {
                actives.push(fam);
            }
        }
        for r in &deck_rows {
            let max = json::i(json::dget(r, "deck_slots"), "active", 0);
            if actives.len() as i64 > max {
                out.push(format!("{tag}: deck add makes {} actives (the {} ruleset's deck slots allow {max}).", actives.len(), json::s_or(r, "name", &json::s(r, "id"))));
            }
        }
        // planner 2026-10-29: an automatic deck's order may also name the automatic actives the add leaves in place
        let mut orderable = actives.clone();
        if json::sget(deck, "active").is_empty() {
            for r in &deck_rows {
                for fam in auto_after_add(v, &unit, fx, r).0 {
                    if !orderable.contains(&fam) {
                        orderable.push(fam);
                    }
                }
            }
        }
        for (intent, fams) in json::dget(fx, "order") {
            for fam in json::arr(fams).iter().map(gstr) {
                if !orderable.contains(&fam) && !basics.contains(&fam) {
                    out.push(format!("{tag}: deck order {intent}: '{fam}' is neither an active of the deck nor a basic."));
                    continue;
                }
                // A4.2-K1b (IR15): the derived rows under field_intent
                let head = v.get("skills", &fam);
                let rows = if head.is_empty() { Vec::new() } else { super::ruleset::intent_rows(head, super::ruleset::deck_plays_field(v)) };
                if !rows.contains(intent) {
                    let of = super::ruleset::rows_label(&rows);
                    out.push(format!("{tag}: deck order {intent}: '{fam}' is of intent {}.", if of.is_empty() { "none (unknown family)" } else { &of }));
                }
            }
        }
    }
    out
}

/// DU1 as built (user 2026-10-29 (b)): an `add` on an automatic deck (no `active` list) whose loadout (else class path
/// and extra skills) already fills a deck ruleset's active slots pushes out the last automatic families at run time;
/// a warning names them. Same rows and refs as `deck_effect_errors`.
pub fn deck_effect_warnings(v: &View, e: &Dict, world: Option<&str>, unit_of: &dyn Fn(&str) -> Dict) -> Vec<String> {
    let mut out = Vec::new();
    let decks: Vec<(String, &Dict)> = json::aget(e, "phases")
        .iter()
        .filter_map(Value::as_object)
        .flat_map(|p| json::aget(p, "effects").iter().filter_map(Value::as_object).filter(|fx| json::s(fx, "type") == DECK_EFFECT).map(move |fx| (json::s(p, "id"), fx)))
        .collect();
    if decks.is_empty() {
        return out;
    }
    let deck_rows: Vec<&Dict> = match world {
        Some(w) => world_ruleset_row(v, w).filter(|r| lists_rule(Some(r), ruleset::DECK)).into_iter().collect(),
        None => rows_with(v, ruleset::DECK),
    };
    for (phase, fx) in decks {
        let unit = unit_of(&json::s(fx, "unit"));
        if unit.is_empty() || !json::sget(json::dget(&unit, "deck"), "active").is_empty() {
            continue;
        }
        for r in &deck_rows {
            let max = json::i(json::dget(r, "deck_slots"), "active", 0).max(0);
            let pushed = auto_after_add(v, &unit, fx, r).1;
            if !pushed.is_empty() {
                out.push(format!(
                    "Phase '{phase}': deck add on the automatic deck of '{}' pushes out {} (the {} ruleset's deck slots allow {max}).",
                    json::s(fx, "unit"),
                    pushed.iter().map(|f| format!("'{f}'")).collect::<Vec<_>>().join(", "),
                    json::s_or(r, "name", &json::s(r, "id"))
                ));
            }
        }
    }
    out
}

/// DU1 as built: the automatic actives of `unit` (no authored `active` list) under deck ruleset row `r` that the
/// effect's `add` keeps, and the ones it pushes out (the last ones, one per new owned family when the slots are full),
/// as the run time does. The automatic list is the battle's own (`rpgkit_rules::unit::auto_actives`: the unit built
/// under the row's deck rule with its deck left automatic), so the check and the run time cannot drift.
fn auto_after_add(v: &View, unit: &Dict, fx: &Dict, r: &Dict) -> (Vec<String>, Vec<String>) {
    let owned = super::unit::owned_families(v, unit);
    let max = json::i(json::dget(r, "deck_slots"), "active", 0).max(0) as usize;
    // CLASS-BASICS: with the unit's class basics
    let rule = rpgkit_rules::unit::DeckRule::of(&ruleset::Ruleset::from_row(r)).unwrap_or_default().with_class(&v.typed, &json::s(unit, "class_id"));
    let basics = rule.basics.clone();
    let auto = rpgkit_rules::unit::auto_actives(&v.typed, &rpgkit_data::UnitDef::from_obj(unit), &rule);
    let mut new: Vec<String> = Vec::new();
    for f in json::aget(fx, "add").iter().map(gstr) {
        if owned.contains(&f) && !auto.contains(&f) && !basics.contains(&f) && !new.contains(&f) {
            new.push(f);
        }
    }
    if max == 0 || auto.len() + new.len() <= max {
        return (auto, Vec::new());
    }
    let keep = max.saturating_sub(new.len()).min(auto.len());
    (auto[..keep].to_vec(), auto[keep..].to_vec())
}

/// DU1: the shape of one phase effect `deck` (keys, `add` a list of families, `order` {intent: [families]}, at least
/// one of them set).
pub fn deck_shape_errors(tag: &str, fx: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let add_ok = match fx.get("add") {
        None => true,
        Some(Value::Array(a)) => a.iter().all(Value::is_string),
        Some(_) => false,
    };
    if !add_ok {
        out.push(format!("Phase '{tag}': deck add must be a list of skill families."));
    }
    match fx.get("order") {
        None => {}
        Some(Value::Object(order)) => {
            for (intent, fams) in order {
                if !ruleset::INTENTS.contains(&intent.as_str()) {
                    out.push(format!("Phase '{tag}': deck order: '{intent}' is not an intent (one of {}).", ruleset::INTENTS.join(", ")));
                } else if !fams.as_array().is_some_and(|a| a.iter().all(Value::is_string)) {
                    out.push(format!("Phase '{tag}': deck order {intent} must be a list of skill families."));
                }
            }
        }
        Some(_) => out.push(format!("Phase '{tag}': deck order must be an object {{intent: [skill families]}}.")),
    }
    let empty = |key: &str| match fx.get(key) {
        None => true,
        Some(Value::Array(a)) => a.is_empty(),
        Some(Value::Object(o)) => o.is_empty(),
        Some(_) => false,
    };
    if empty("add") && empty("order") {
        out.push(format!("Phase '{tag}': a deck effect sets add or order."));
    }
    out
}

// ---- World manifest (1.2, CN1) -----------------------------------------------------------------------------------

/// L3 (FB-138): the warnings of world `w`'s own `classes` overlays, as "<w>/world: Class names '<class>': ...": a class
/// basic of the base class row listed in the overlay's `skill_path` (`unit::basics_in_path_warnings`, the class row's
/// own warning), since the world's units are built from that path (`Data::cat_in`).
pub fn classes_warnings(v: &View, w: &str) -> Vec<String> {
    let Some(o) = json::get(v.worlds.manifest(w), "classes").as_object() else { return Vec::new() };
    let mut out = Vec::new();
    for (class_id, raw) in o {
        let Some(path) = raw.as_object().and_then(|c| c.get("skill_path")).and_then(Value::as_object) else { continue };
        let row = v.get("classes", class_id);
        out.extend(super::unit::basics_in_path_warnings(row, path).into_iter().map(|m| format!("{w}/world: Class names '{class_id}': {m}")));
    }
    out
}

/// CN1: a manifest's `classes`: base class ids, each {name 1-32, description 0-240, skill_path absent or {family:
/// level 1-30}}.
pub fn classes_errors(v: &View, m: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let o = match m.get("classes") {
        None => return out,
        Some(Value::Object(o)) => o,
        Some(_) => return vec!["Class names must be an object {class_id: {name, description, skill_path}}.".into()],
    };
    for (class_id, raw) in o {
        let at = format!("Class names '{class_id}'");
        if !v.has("classes", class_id) {
            out.push(format!("Class names: unknown class '{class_id}'."));
        }
        let Some(c) = raw.as_object() else {
            out.push(format!("{at} must be an object {{name, description, skill_path}}."));
            continue;
        };
        unknown_keys(c, &CLASS_OVERLAY_KEYS, &at, &mut out);
        let name = json::s(c, "name");
        if name.trim().is_empty() || chars(&name) > CLASS_NAME_MAX {
            out.push(format!("{at}: the name needs 1 to {CLASS_NAME_MAX} characters."));
        }
        if c.get("description").is_some_and(|d| !d.as_str().is_some_and(|s| chars(s) <= CLASS_DESCRIPTION_MAX)) {
            out.push(format!("{at}: the description is text of at most {CLASS_DESCRIPTION_MAX} characters."));
        }
        match c.get("skill_path") {
            None => {}
            Some(Value::Object(path)) => {
                for (fam, level) in path {
                    if v.typed.tier_one(fam).is_none() {
                        out.push(format!("{at}: skill path: unknown skill family '{fam}'."));
                    }
                    if !whole_in(level, PATH_LEVEL) {
                        out.push(format!("{at}: skill path {fam}: the level must be a whole number {}-{}.", PATH_LEVEL.0, PATH_LEVEL.1));
                    }
                }
            }
            Some(_) => out.push(format!("{at}: skill_path must be an object {{family: level}} ({{}} = no path).")),
        }
    }
    out
}
