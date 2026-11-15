//! Terrain rows (TerrainValidator) and effect lists (Passives.validate_effects, used by passives and item gear).

use aegis_core::fmt::fixed;
use aegis_core::json::{self, Dict};
use rpgkit_battle::grid::{glyph_of, terrain_walkable, TerrainTable, GLYPH_KEY, GLYPHS, TERRAIN_IDS};
use rpgkit_rules::{intent, status};
use serde_json::Value;

const TRIGGERS: [&str; 3] = ["enter", "round_end", "both"];

/// LAYER-DOWN-2: the battle's terrain table of the loaded terrain rows `rows` (the built-in types, then the rows' new
/// ones with their glyphs; `grid::TerrainTable::from_catalog`).
pub fn terrain_table(rows: &[Dict]) -> TerrainTable {
    let mut cat = rpgkit_data::Catalog::default();
    cat.terrain = rows.iter().map(|r| (json::s(r, "id"), r.clone())).collect();
    TerrainTable::from_catalog(&cat)
}

/// TerrainValidator.entry_errors of a normalized terrain row. LAYER-DOWN-2: any id names a terrain type (a built-in one
/// or a new one); a new type needs its board glyph, one character no other type has; a built-in type keeps its own.
pub fn terrain_errors(v: &super::View, e: &Dict, current_id: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let id = json::s(e, "id");
    if !aegis_content::schema::is_id(&id) {
        errors.push(format!("Unknown terrain type '{id}'."));
        return errors;
    }
    let glyph = json::s(e, GLYPH_KEY);
    match TERRAIN_IDS.iter().position(|t| *t == id) {
        Some(i) if !glyph.is_empty() => errors.push(format!("Built-in terrain type '{id}' keeps its glyph '{}' (leave the glyph empty).", GLYPHS[i])),
        Some(_) => {}
        None => {
            if glyph.chars().count() != 1 || glyph.chars().any(char::is_whitespace) {
                errors.push(format!("Terrain type '{id}' needs a one-character glyph (its letter in encounter and zone rows)."));
            } else {
                let ch = glyph.chars().next();
                let others = v.entries("terrain").iter().filter(|r| { let o = json::s(r, "id"); o != id && o != current_id });
                let taken = TERRAIN_IDS.iter().map(|t| t.to_string()).filter(|t| *t != id && glyph_of(t, None) == ch).chain(others.filter(|r| glyph_of(&json::s(r, "id"), Some(r)) == ch).map(|r| json::s(r, "id"))).next();
                if let Some(other) = taken {
                    errors.push(format!("Glyph '{glyph}' is taken by terrain type '{other}'."));
                }
            }
        }
    }
    // P1.2-S1: the row's `walkable`, else its missing-key fallback
    let walkable = e.get("walkable").map_or_else(|| terrain_walkable(&id), json::to_bool);
    let cost = json::i(e, "move_cost", 1);
    if walkable && !(1..=3).contains(&cost) {
        errors.push("Move cost must be 1-3.".into());
    }
    let cover = json::f(e, "cover_guard", 0.0);
    if !(0.0..=0.5).contains(&cover) {
        errors.push("Cover must be 0.0-0.5.".into());
    }
    let st = json::s(e, "hazard_status");
    if !st.is_empty() {
        if !status::debuffs().contains(&st.as_str()) {
            errors.push(format!("Hazard status '{st}' is not a debuff."));
        }
        if !walkable {
            errors.push("Only walkable terrain can carry a hazard.".into());
        }
    }
    if !(1..=100).contains(&json::i(e, "hazard_potency", 100)) {
        errors.push("Hazard potency must be 1-100.".into());
    }
    let magnitude = json::i(e, "hazard_magnitude", 0);
    match status::row_of(&st).filter(|r| r.magnitude_max > 0) {
        Some(r) => {
            if magnitude < r.magnitude_min as i64 || magnitude > r.magnitude_max as i64 {
                errors.push(format!("{st} needs a magnitude {}-{}", r.magnitude_min, r.magnitude_max));
            }
        }
        None if magnitude != 0 => {
            // S1.2-S4: `guard` (queued only by a multi-round Guard skill) is not a hazard's
            let ids: Vec<&str> = status::ROWS.iter().filter(|r| r.magnitude_max > 0 && (!r.buff || !r.grant.is_empty())).map(|r| r.id).collect();
            errors.push(format!("magnitude only applies to {}", ids.join(", ")));
        }
        None => {}
    }
    if !(1..=5).contains(&json::i(e, "hazard_rounds", 1)) {
        errors.push("Hazard rounds must be 1-5.".into());
    }
    if !TRIGGERS.contains(&json::s(e, "hazard_on").as_str()) {
        errors.push("Hazard trigger must be enter, round_end or both.".into());
    }
    if json::s_or(e, "origin", super::ORIGIN_SYSTEM) != super::ORIGIN_SYSTEM {
        errors.push("Terrain rows are system assets (origin system).".into());
    }
    // PR5-33: the look colour is empty (the game's default) or #rrggbb
    let color = json::s(e, rpgkit_data::looks::COLOR_KEY);
    if !color.is_empty() && (!color.starts_with('#') || rpgkit_data::looks::parse_color(&color).is_none()) {
        errors.push(format!("Colour '{color}' must be #rrggbb (or empty for the default)."));
    }
    errors
}

/// TerrainValidator.missing_ids: terrain types without a row, in terrain order.
pub fn missing_terrain(rows: &[Dict]) -> Vec<&'static str> {
    TERRAIN_IDS.iter().copied().filter(|id| !rows.iter().any(|r| json::s(r, "id") == *id)).collect()
}

/// Passives.EFFECT_TYPES; RULESET1-S adds `stagger_taken` (rule poise, P6), HOOKS1-S `forced_move`, `trigger` and `meter`
/// (`check::hooks`); PR5-50 (FB-204) lists `skill_reach` where `spell_reach` stood (the alias still checks, unlisted).
const TYPES: [&str; 15] = [
    "stat",
    "proficiency",
    "resist",
    "ap_regen",
    "skill",
    "skill_potency",
    "status_potency",
    rpgkit_data::SKILL_REACH,
    "unlock",
    "foresight",
    "stagger_taken",
    "forced_move",
    "trigger",
    "meter",
    "crit",
];
/// RULESET1-S: a `stagger_taken` effect's multiplier and the longest action (blocks) it matches (0 = any).
const STAGGER_MULT_MAX: f64 = 2.0;
const STAGGER_MAX_BLOCKS: i64 = 6;
/// RULESET1 V4: a foresight effect's optional reveal range in tiles (0 = today's rule).
const FORESIGHT_RANGE_MAX: i64 = 8;
/// The effect types gear may carry; HOOKS1-S adds `trigger` and `forced_move`; U3.2-S4 (FB-35, OFFHAND1) `skill` (a worn
/// skill: always active while worn, no slot) and `stat` (a U1 modifier), both taken off with the item. `proficiency`
/// stays out (training is the class's and the drills'; an item training its own gear would undo the wield penalty).
pub(crate) const ITEM_TYPES: [&str; 8] = ["skill_potency", "status_potency", rpgkit_data::SKILL_REACH, "resist", "trigger", "forced_move", "skill", "stat"];
/// PR5-50 (FB-204): a `skill_reach` effect's `kind` filter values (the authorable skill kinds and the spell group) and
/// its `intent` filter values (the round intents IR21 derives; `rpgkit_rules::unit::skill_reach_matches`).
pub const SKILL_REACH_KINDS: [&str; 6] = ["ATTACK", "DEFEND", "FOCUS", "SUPPORT", "DISENGAGE", rpgkit_data::SKILL_REACH_SPELL];
pub const SKILL_REACH_INTENTS: [&str; 5] = [intent::ASSAULT, intent::STAND, intent::SUPPORT, intent::MANEUVER, intent::FIELD];
const GROUPS: [&str; 3] = ["spell", "heal", "channel"];
const STATS: [&str; 6] = ["STR", "DEX", "INT", "CONS", "PERC", "KNOW"];

/// A whole number in `lo..=hi` (JSON numbers parse as floats).
fn whole_in(v: &Value, lo: i64, hi: i64) -> bool {
    v.as_f64().is_some_and(|x| x.fract() == 0.0 && (lo as f64..=hi as f64).contains(&x))
}

/// Passives.validate_effects: the problems of an effect list (`allow_empty`: none is fine; `items_only`: the kinds
/// gear may carry). DERIVE1 4.1: a `stat` effect may name one of the `derived` stat ids, and only those take an
/// optional `percent` (-1.0..1.0). HOOKS1-S: the hook effects read `hooks` (skill families, unit meters).
pub fn effect_errors(effects: &Value, skill_exists: &dyn Fn(&str) -> bool, derived: &[String], allow_empty: bool, items_only: bool, hooks: &super::hooks::Ctx) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(list) = effects.as_array() else {
        errors.push("Effects must be a list.".into());
        return errors;
    };
    if list.is_empty() && !allow_empty {
        errors.push("A passive needs at least one effect.".into());
    }
    let resistable: Vec<&str> = status::ROWS.iter().filter(|r| !r.buff && r.resistable).map(|r| r.id).collect();
    for (i, raw) in list.iter().enumerate() {
        let e = json::dict(raw);
        let t = json::s(e, "type");
        let at = format!("Effect {}", i + 1);
        // PK2-S1: the member grants are passives' only (a unit track's stage passive)
        let member = rpgkit_data::MEMBER_GRANTS.contains(&t.as_str());
        if items_only && (TYPES.contains(&t.as_str()) || member) && !ITEM_TYPES.contains(&t.as_str()) {
            errors.push(format!("{at}: '{t}' is not allowed on items (use stat_bonuses / passives)."));
            continue;
        }
        if !matches!(t.as_str(), "trigger" | "meter")
            && (TYPES.contains(&t.as_str()) || t == rpgkit_data::SKILL_BLOCK_COST || t == rpgkit_data::SPELL_REACH_ALIAS)
        {
            if let Some(w) = e.get("when") {
                errors.extend(super::hooks::when_errors(w, hooks, &at, None));
            }
        }
        let target_problem = |errors: &mut Vec<String>| {
            let fam = e.get("family").is_some_and(|v| !json::gstr(v).is_empty());
            let group = e.get("group").is_some_and(|v| !json::gstr(v).is_empty());
            if fam == group {
                errors.push(format!("{at}: give exactly one of family (a skill id) or group ({}).", GROUPS.join(", ")));
            } else if group && !GROUPS.contains(&json::s(e, "group").as_str()) {
                errors.push(format!("{at}: unknown skill group '{}' (use {}).", json::s(e, "group"), GROUPS.join(", ")));
            } else if fam && !skill_exists(&json::s(e, "family")) {
                errors.push(format!("{at}: unknown skill family '{}'.", json::s(e, "family")));
            }
        };
        match t.as_str() {
            "stat" => {
                let stat = json::s(e, "stat");
                let is_derived = derived.contains(&stat);
                if !STATS.contains(&stat.as_str()) && !is_derived {
                    errors.push(format!("{at}: unknown stat '{stat}'."));
                }
                let mut percent = 0.0;
                if let Some(p) = e.get("percent") {
                    if !is_derived {
                        errors.push(format!("{at}: percent is for derived stats only (base stats take a flat value)."));
                    } else if let Some(x) = p.as_f64().filter(|x| x.abs() <= 1.0) {
                        percent = x;
                    } else {
                        errors.push(format!("{at}: percent must be -1.0..1.0."));
                    }
                }
                if json::i(e, "value", 0) == 0 && percent == 0.0 {
                    errors.push(format!("{at}: the stat bonus needs a non-zero value."));
                }
                let ctx = e.get("context").cloned().unwrap_or(Value::Object(Default::default()));
                if !ctx.as_object().is_some_and(|c| c.len() <= 1) {
                    errors.push(format!("{at}: context is one {{key: value}} pair (or empty)."));
                }
            }
            "proficiency" => {
                if !hooks.gear.contains(&json::s(e, "gear")) {
                    errors.push(format!("{at}: unknown gear type '{}'.", json::s(e, "gear")));
                }
                if !(1..=3).contains(&json::i(e, "levels", 0)) {
                    errors.push(format!("{at}: levels must be 1-3."));
                }
            }
            "resist" => {
                if !resistable.contains(&json::s(e, "status").as_str()) {
                    errors.push(format!("{at}: unknown status '{}'.", json::s(e, "status")));
                }
                if !(1..=100).contains(&json::i(e, "value", 0)) {
                    errors.push(format!("{at}: resistance is 1-100."));
                }
            }
            "ap_regen" => {
                if !(1..=4).contains(&json::i(e, "value", 0)) {
                    errors.push(format!("{at}: extra AP per round is 1-4."));
                }
            }
            "skill" => {
                if !skill_exists(&json::s(e, "skill")) {
                    errors.push(format!("{at}: unknown skill '{}'.", json::s(e, "skill")));
                }
            }
            "skill_potency" => {
                target_problem(&mut errors);
                let pv = json::f(e, "value", 0.0);
                if pv.abs() < 0.00001 || !(-0.9..=2.0).contains(&pv) {
                    errors.push(format!("{at}: potency bonus must be non-zero, {} to {} (0.25 = +25%).", fixed(-0.9, 1), fixed(2.0, 1)));
                }
            }
            "unlock" => {
                target_problem(&mut errors);
                if !(1..=2).contains(&json::i(e, "levels", 0)) {
                    errors.push(format!("{at}: unlock levels must be 1-2."));
                }
            }
            "status_potency" => {
                if !(1..=25).contains(&json::i(e, "value", 0)) {
                    errors.push(format!("{at}: status potency is 1-25 points."));
                }
            }
            // PR5-50: the retired form, checked as before (it loads as `skill_reach {reach: value, kind: SPELL}`)
            rpgkit_data::SPELL_REACH_ALIAS => {
                if !(1..=2).contains(&json::i(e, "value", 0)) {
                    errors.push(format!("{at}: spell reach is 1-2 tiles."));
                }
            }
            // PR5-50 (FB-204): `skill_reach` {reach 1-2, intent?, kind?} (an absent or empty filter matches every skill)
            rpgkit_data::SKILL_REACH => {
                let (lo, hi) = rpgkit_data::SKILL_REACH_RANGE;
                if !e.get("reach").is_some_and(|r| whole_in(r, lo, hi)) {
                    errors.push(format!("{at}: skill_reach reach must be a whole number {lo}-{hi} (tiles)."));
                }
                if let Some(i) = e.get("intent").filter(|i| !i.as_str().is_some_and(|s| s.is_empty() || SKILL_REACH_INTENTS.contains(&s))) {
                    errors.push(format!("{at}: skill_reach intent '{}' is not one of {}.", json::gstr(i), SKILL_REACH_INTENTS.join(", ")));
                }
                if let Some(k) = e.get("kind").filter(|k| !k.as_str().is_some_and(|s| s.is_empty() || SKILL_REACH_KINDS.contains(&s))) {
                    errors.push(format!("{at}: skill_reach kind '{}' is not one of {}.", json::gstr(k), SKILL_REACH_KINDS.join(", ")));
                }
                if e.get("ranged").is_some_and(|v| !v.is_boolean()) {
                    errors.push(format!("{at}: skill_reach ranged must be true or false."));
                }
                for k in e.keys().filter(|k| *k != "when" && !rpgkit_data::SKILL_REACH_KEYS.contains(&k.as_str())) {
                    errors.push(format!("{at}: skill_reach takes only reach, intent, kind and ranged (not '{k}')."));
                }
            }
            "foresight" => {
                if e.get("range").is_some_and(|r| !whole_in(r, 0, FORESIGHT_RANGE_MAX)) {
                    errors.push(format!("{at}: foresight range must be a whole number 0-{FORESIGHT_RANGE_MAX} (0 = the plain rule)."));
                }
            }
            "stagger_taken" => {
                if !e.get("mult").and_then(Value::as_f64).is_some_and(|m| (0.0..=STAGGER_MULT_MAX).contains(&m)) {
                    errors.push(format!("{at}: stagger taken needs a mult 0.0-{} (0 = immune).", fixed(STAGGER_MULT_MAX, 1)));
                }
                if e.get("max_blocks").is_some_and(|b| !whole_in(b, 0, STAGGER_MAX_BLOCKS)) {
                    errors.push(format!("{at}: max_blocks must be a whole number 0-{STAGGER_MAX_BLOCKS} (0 = any action)."));
                }
            }
            "forced_move" | "trigger" | "meter" => errors.extend(super::hooks::passive_effect_errors(e, &t, hooks, &at)),
            // PK2-S1 (`docs/sessions/PK2_GRAFTS.md` 2): `forbid_hands` {hands: 2} and `pierce_melee` {} (no other key;
            // the unknown-type message keeps its list, checks_golden)
            rpgkit_data::FORBID_HANDS => {
                if !e.get("hands").is_some_and(|h| rpgkit_data::FORBID_HANDS_VALUES.iter().any(|&n| whole_in(h, n, n))) {
                    errors.push(format!("{at}: forbid_hands needs hands {} (two-handed items).", rpgkit_data::FORBID_HANDS_VALUES[0]));
                }
                for k in e.keys().filter(|k| !matches!(k.as_str(), "type" | "hands")) {
                    errors.push(format!("{at}: forbid_hands takes only hands (not '{k}')."));
                }
            }
            rpgkit_data::PIERCE_MELEE => {
                for k in e.keys().filter(|k| k.as_str() != "type") {
                    errors.push(format!("{at}: pierce_melee takes no keys (not '{k}')."));
                }
            }
            // ACT1-HOOKS2 (FB-244 (4)): `skill_block_cost` {skill, delta} on gear and passives (the unknown-type message
            // keeps its list, checks_golden)
            rpgkit_data::SKILL_BLOCK_COST => {
                let skill = json::s(e, "skill");
                if !skill_exists(&skill) {
                    errors.push(format!("{at}: skill_block_cost: unknown skill '{skill}'."));
                }
                let (lo, hi) = rpgkit_data::SKILL_BLOCK_COST_DELTA;
                if !e.get("delta").is_some_and(|d| whole_in(d, lo, hi)) || json::i(e, "delta", 0) == 0 {
                    errors.push(format!("{at}: skill_block_cost delta must be a whole number {lo}..{hi}, not 0."));
                }
                for k in e.keys().filter(|k| *k != "when" && !rpgkit_data::SKILL_BLOCK_COST_KEYS.contains(&k.as_str())) {
                    errors.push(format!("{at}: skill_block_cost takes only skill and delta (not '{k}')."));
                }
            }
            "crit" => {
                let valid = e.get("value").and_then(Value::as_f64).is_some_and(|cv| cv.abs() >= 0.00001 && (-100.0..=100.0).contains(&cv));
                if !valid {
                    errors.push(format!("{at}: crit bonus must be non-zero, {} to {} (percentage points).", fixed(-100.0, 1), fixed(100.0, 1)));
                }
            }
            _ => errors.push(format!("{at}: unknown effect type '{t}' (use {}).", if items_only { ITEM_TYPES.join(", ") } else { TYPES.join(", ") })),
        }
    }
    errors
}
