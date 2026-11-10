//! Combat hook kinds (HOOKS1-S, docs/specs/HOOKS1_TASKS.md 1.4-1.6; engine-only, no Godot original): the checks of one
//! row of the kinds `hazards` (GH1), `unit_meters` (UM1-UM6) and `zones` (ZN1-ZN2), and across rows (unique ids). The
//! numbers' ranges live in these checks (the schema rows carry no `min` / `max`, so normalizing never clamps): a value
//! out of range is refused with its reason. The loader keeps the rows that pass, in file order.

use super::hooks::{effects_errors, trigger_row_errors_with, whole_in, Ctx, SKILL_ROWS_MAX};
use super::View;
use crate::source::fields;
use aegis_content::schema::is_id;
use aegis_core::assets::hex_colour;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_battle::grid::{rows_valid, TERRAIN_IDS, ZONE_IDS};
use rpgkit_data::{BAND_EFFECTS, HAZARDS_KIND, HAZARD_AFFECTS, HAZARD_ON, HEAL_TAKEN, METER_DELTA_CAP, METER_PIPS_MAX, METER_SHOW, TRIGGER_EVENTS, UNIT_METERS_KIND, ZONES_KIND};
use rpgkit_rules::status;
use serde_json::Value;

/// The id's length range (a letter first, then a-z, 0-9 or _).
pub const ID_MIN: usize = 2;
pub const ID_MAX: usize = 24;
/// GH1, UM1: the name and short name lengths.
pub const NAME_MAX: usize = 24;
pub const SHORT_MAX: usize = 8;
/// UM1: the largest meter max; the decay, band and rule rows; a decay delta.
pub const METER_MAX: i64 = 200;
pub const DECAY_MAX: usize = 3;
pub const BANDS_MAX: usize = 8;
pub const RULES_MAX: usize = 6;
pub const DECAY_DELTA: (i64, i64) = (-50, 50);
/// UM5: a meter rule's crossing directions.
pub const RULE_DIRS: [&str; 2] = ["up", "down"];
/// GH1 (HOOKS1_TASKS 1.4): a hazard row's number ranges (key, label, lo, hi).
pub const HAZARD_RANGES: [(&str, &str, i64, i64); 5] =
    [("potency", "Potency", 0, 100), ("status_rounds", "Status rounds", 1, 5), ("damage", "Damage", 0, 50), ("move_cost", "Move cost", 0, 3), ("duration", "Duration", 1, 5)];
/// GH1: a hazard's cover against ranged hits.
pub const HAZARD_COVER: (f64, f64) = (0.0, 0.5);
/// PK1-S1 (PK1_CORRUPTION 3.1): a band's `heal_taken` percent (not 0).
pub const HEAL_TAKEN_PCT: (i64, i64) = (-100, 100);

/// What the rows' checks read from the rest of the catalog: the loaded skill rows (flat rank rows).
pub struct Refs<'a> {
    pub skills: &'a [Dict],
    /// U1.2-S5: the loaded derived-stat ids (a meter's `max_stat`).
    pub derived: &'a [String],
    /// U3.2-S2: the item family table (the gear tokens a `proficiency` effect may name; empty = the code list).
    pub families: &'a [Dict],
    /// PK1-S1 (3.6): the unit meter ids a hazard row's `meter` may name (None: not loaded yet, as when the loader reads
    /// the hazards before the meters; only its shape is checked then).
    pub meters: Option<&'a [String]>,
}

/// The noun of a row of `kind` in load messages.
pub fn noun(kind: &str) -> &'static str {
    match kind {
        HAZARDS_KIND => "hazard",
        UNIT_METERS_KIND => "unit meter",
        _ => "zone",
    }
}

fn chars(s: &str) -> usize {
    s.chars().count()
}

/// `[a-z][a-z0-9_]{ID_MIN - 1, ID_MAX - 1}`.
fn id_ok(id: &str) -> bool {
    id.chars().next().is_some_and(|c| c.is_ascii_lowercase()) && (ID_MIN..=ID_MAX).contains(&chars(id)) && is_id(id)
}

/// The schema options of a field of `kind` (origin and the enums).
fn options(kind: &str, key: &str) -> Vec<String> {
    fields("content", kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).map(|f| json::aget(f, "options").iter().map(gstr).collect()).unwrap_or_default()
}

/// The id, name and origin of a row.
fn common_errors(kind: &str, e: &Dict, out: &mut Vec<String>) {
    let id = json::s(e, "id");
    if id.is_empty() {
        out.push("ID is required.".into());
    } else if !id_ok(&id) {
        out.push(format!("ID '{id}' must be {ID_MIN}-{ID_MAX} characters of a-z, 0-9 and _, a letter first."));
    }
    let name = json::s(e, "name");
    if name.trim().is_empty() {
        out.push("Name is required.".into());
    } else if chars(&name) > NAME_MAX {
        out.push(format!("Name is {} characters (max {NAME_MAX}).", chars(&name)));
    }
    let origin = json::s(e, "origin");
    let origins = options(kind, "origin");
    if !origins.contains(&origin) {
        out.push(format!("Origin: '{origin}' is not one of {}.", origins.join(", ")));
    }
}

/// The problems of each row (same order as the rows), on its own and against the others (unique ids).
pub fn check_rows(kind: &str, rows: &[Dict], refs: &Refs) -> Vec<Vec<String>> {
    let mut ctx = Ctx::from_rows(refs.skills, if kind == UNIT_METERS_KIND { rows } else { &[] }, &[]);
    ctx.gear = super::skill::gear_tokens(refs.families);
    let ids: Vec<String> = rows.iter().map(|r| json::s(r, "id")).collect();
    let mut out: Vec<Vec<String>> = rows.iter().map(|r| row_errors(kind, r, &ctx, refs)).collect();
    for (i, found) in out.iter_mut().enumerate() {
        if !ids[i].is_empty() && ids[..i].contains(&ids[i]) {
            found.push(format!("ID '{}' is already used by another {}.", ids[i], noun(kind)));
        }
    }
    out
}

/// The own problems of one (normalized) row of `kind`.
pub fn row_errors(kind: &str, e: &Dict, ctx: &Ctx, refs: &Refs) -> Vec<String> {
    let mut out = Vec::new();
    common_errors(kind, e, &mut out);
    match kind {
        HAZARDS_KIND => {
            hazard_errors(e, &mut out);
            out.extend(super::hooks::hazard_meter_errors(e.get("meter"), refs.meters, "Hazard"));
        }
        UNIT_METERS_KIND => meter_errors(e, ctx, refs, &mut out),
        ZONES_KIND => zone_errors(e, &mut out),
        _ => {}
    }
    out
}

// ---- hazards (GH1) -----------------------------------------------------------------------------------------------

fn hazard_errors(e: &Dict, out: &mut Vec<String>) {
    for (key, label, list) in [("on", "Acts on", &HAZARD_ON[..]), ("affects", "Affects", &HAZARD_AFFECTS[..])] {
        let v = json::s(e, key);
        if !list.contains(&v.as_str()) {
            out.push(format!("{label}: '{v}' is not one of {}.", list.join(", ")));
        }
    }
    let st = json::s(e, "status");
    let magnitude = json::i(e, "magnitude", 0);
    if st.is_empty() {
        if magnitude != 0 {
            out.push("Magnitude needs a status.".into());
        }
    } else if !status::debuffs().contains(&st.as_str()) {
        out.push(format!("Status: '{st}' is not a debuff ({}).", status::debuffs().join(", ")));
    } else {
        match status::row_of(&st).filter(|r| r.magnitude_max > 0) {
            Some(r) if magnitude < r.magnitude_min as i64 || magnitude > r.magnitude_max as i64 => out.push(format!("{st} needs a magnitude {}-{}.", r.magnitude_min, r.magnitude_max)),
            None if magnitude != 0 => out.push(format!("{st} takes no magnitude (0).")),
            _ => {}
        }
    }
    for (key, label, lo, hi) in HAZARD_RANGES {
        let n = json::i(e, key, lo);
        if !(lo..=hi).contains(&n) {
            out.push(format!("{label} must be {lo}-{hi} (is {n})."));
        }
    }
    let cover = json::f(e, "cover_guard", 0.0);
    if !(HAZARD_COVER.0..=HAZARD_COVER.1).contains(&cover) {
        out.push(format!("Cover must be {:.1}-{:.1} (is {cover}).", HAZARD_COVER.0, HAZARD_COVER.1));
    }
    if json::b(e, "hidden", false) && json::s(e, "affects") != "foes" {
        out.push("Hidden needs affects foes (a hidden hazard is hidden from the placer's foes).".into());
    }
    let tint = json::s(e, "tint");
    if hex_colour(&tint).is_none() {
        out.push(format!("Tint '{tint}' must be #rrggbb."));
    }
    let glyph = json::s(e, "glyph");
    if chars(&glyph) != 1 {
        out.push(format!("Glyph '{glyph}' must be one character."));
    }
}

// ---- unit meters (UM1-UM6) ---------------------------------------------------------------------------------------

fn meter_errors(e: &Dict, ctx: &Ctx, refs: &Refs, out: &mut Vec<String>) {
    let id = json::s(e, "id");
    let short = json::s(e, "short");
    if short.trim().is_empty() || chars(&short) > SHORT_MAX {
        out.push(format!("Short name needs 1 to {SHORT_MAX} characters."));
    }
    let (min, max, start) = (json::i(e, "min", 0), json::i(e, "max", 10), json::i(e, "start", 0));
    if min >= max || max > METER_MAX {
        out.push(format!("Min must be below max, and max at most {METER_MAX} (min {min}, max {max})."));
    }
    if start < min || start > max {
        out.push(format!("Start must be {min}-{max} (is {start})."));
    }
    let show = json::s(e, "show");
    if !METER_SHOW.contains(&show.as_str()) {
        out.push(format!("Show: '{show}' is not one of {}.", METER_SHOW.join(", ")));
    } else if show == "pips" && max - min > METER_PIPS_MAX as i64 {
        // UM7: one pip a step from min to max, as the battle screen draws them (`meter::shown`)
        out.push(format!("Pips show at most {METER_PIPS_MAX} steps (max - min is {}); use bar.", max - min));
    }
    // U1.2-S5: the pool fields
    let max_stat = json::s(e, "max_stat");
    if !max_stat.is_empty() && !refs.derived.contains(&max_stat) {
        out.push(format!("Max stat: unknown derived stat '{max_stat}'."));
    }
    if !whole_in(json::get(e, "delta_max"), (1, i64::from(METER_DELTA_CAP))) {
        out.push(format!("Largest change (delta_max) must be a whole number 1-{METER_DELTA_CAP}."));
    }
    let list = |key: &str, label: &str, cap: usize, out: &mut Vec<String>| -> Vec<Dict> {
        let raw = json::get(e, key);
        let Some(a) = raw.as_array() else {
            out.push(format!("{label} must be a list."));
            return Vec::new();
        };
        if a.len() > cap {
            out.push(format!("{label} holds at most {cap} rows."));
        }
        let mut rows = Vec::new();
        for (i, item) in a.iter().enumerate() {
            match item.as_object() {
                Some(o) => rows.push(o.clone()),
                None => out.push(format!("{label} row {} must be an object.", i + 1)),
            }
        }
        rows
    };
    for (i, d) in list("decay", "Decay", DECAY_MAX, out).iter().enumerate() {
        let at = format!("Decay {}", i + 1);
        for k in d.keys().filter(|k| !["delta", "unless_gained", "if_guarded"].contains(&k.as_str())) {
            out.push(format!("{at}: unknown key '{k}' (keys: delta, unless_gained, if_guarded)."));
        }
        let delta = json::get(d, "delta");
        if !whole_in(delta, DECAY_DELTA) || json::to_int(delta) == 0 {
            out.push(format!("{at}: delta must be a whole number {}..{}, not 0.", DECAY_DELTA.0, DECAY_DELTA.1));
        }
        for k in ["unless_gained", "if_guarded"] {
            if d.get(k).is_some_and(|b| !b.is_boolean()) {
                out.push(format!("{at}: {k} must be true or false."));
            }
        }
    }
    let bands = list("bands", "Bands", BANDS_MAX, out);
    let mut band_ids: Vec<String> = Vec::new();
    let mut last_from: Option<i64> = None;
    for (i, b) in bands.iter().enumerate() {
        let bid = json::s(b, "id");
        let at = format!("Band '{}'", if bid.is_empty() { (i + 1).to_string() } else { bid.clone() });
        for k in b.keys().filter(|k| !["id", "name", "from", "from_pct", "effects", "statuses"].contains(&k.as_str())) {
            out.push(format!("{at}: unknown key '{k}' (keys: id, name, from or from_pct, effects, statuses)."));
        }
        if !is_id(&bid) {
            out.push(format!("{at}: the id must be lowercase snake_case (a-z, 0-9, _)."));
        } else if band_ids.contains(&bid) {
            out.push(format!("Band ids must be unique: '{bid}'."));
        }
        band_ids.push(bid);
        // U1.2-S5: a band starts at a value (`from`) or at a percent of the max (`from_pct`; checked at the row's max)
        let pct = b.get("from_pct");
        let from = json::get(b, "from");
        let start = match pct {
            Some(_) if b.contains_key("from") => {
                out.push(format!("{at}: set from or from_pct, not both."));
                None
            }
            Some(p) if !whole_in(p, (0, 100)) => {
                out.push(format!("{at}: from_pct must be a whole number 0-100."));
                None
            }
            Some(p) => Some(rpgkit_rules::bands::pct_point(json::to_int(p), max).max(min)),
            None if !whole_in(from, (min, max)) => {
                out.push(format!("{at}: from must be a whole number {min}-{max}."));
                None
            }
            None => Some(json::to_int(from)),
        };
        if let Some(f) = start {
            if i == 0 && f != min {
                out.push(format!("{at}: the first band starts at min ({min})."));
            }
            if last_from.is_some_and(|l| f <= l) {
                out.push(format!("{at}: bands run in ascending from."));
            }
            last_from = Some(f);
        }
        band_effect_errors(json::get(b, "effects"), &at, refs, out);
        for (j, s) in json::aget(b, "statuses").iter().enumerate() {
            let sat = format!("{at} status {}", j + 1);
            let Some(s) = s.as_object() else {
                out.push(format!("{sat} must be an object {{status, magnitude}}."));
                continue;
            };
            // U1.2-S5: any kit status (a pool's low band holds a debuff such as weaken)
            let st = json::s(s, "status");
            if status::row_of(&st).is_none() {
                out.push(format!("{sat}: '{st}' is not a status."));
                continue;
            }
            let m = json::get(s, "magnitude");
            match status::row_of(&st).filter(|r| r.magnitude_max > 0) {
                Some(r) if !whole_in(m, (r.magnitude_min as i64, r.magnitude_max as i64)) => out.push(format!("{sat}: {st} needs a magnitude {}-{}.", r.magnitude_min, r.magnitude_max)),
                None if !m.is_null() && !whole_in(m, (0, 0)) => out.push(format!("{sat}: {st} takes no magnitude (0).")),
                _ => {}
            }
        }
    }
    let mut rule_ids: Vec<String> = Vec::new();
    for (i, r) in list("rules", "Rules", RULES_MAX, out).iter().enumerate() {
        let rid = json::s(r, "id");
        let at = format!("Rule '{}'", if rid.is_empty() { (i + 1).to_string() } else { rid.clone() });
        for k in r.keys().filter(|k| !["id", "at", "dir", "once", "effects"].contains(&k.as_str())) {
            out.push(format!("{at}: unknown key '{k}' (keys: id, at, dir, once, effects)."));
        }
        if !is_id(&rid) {
            out.push(format!("{at}: the id must be lowercase snake_case (a-z, 0-9, _)."));
        } else if rule_ids.contains(&rid) {
            out.push(format!("Rule ids must be unique: '{rid}'."));
        }
        rule_ids.push(rid);
        let point = json::get(r, "at");
        let ok = match point {
            Value::String(s) => band_ids.contains(s),
            _ => whole_in(point, (min, max)),
        };
        if !ok {
            out.push(format!("{at}: at must be a band id or a whole number {min}-{max}."));
        }
        if r.get("dir").is_some_and(|d| !d.as_str().is_some_and(|d| RULE_DIRS.contains(&d))) {
            out.push(format!("{at}: dir must be up or down."));
        }
        if r.get("once").is_some_and(|b| !b.is_boolean()) {
            out.push(format!("{at}: once must be true or false."));
        }
        out.extend(effects_errors(json::get(r, "effects"), ctx, &at, Some(""), true));
    }
    // U1.2-S5: the trigger rows every holder holds; PK1-KITFIX (a): their effects take `meter_set` too, as the rules'
    for (i, t) in list("triggers", "Triggers", SKILL_ROWS_MAX, out).iter().enumerate() {
        out.extend(trigger_row_errors_with(t, ctx, &format!("Trigger {}", i + 1), &TRIGGER_EVENTS, &[], true));
    }
    // UM6: a meter skill spends this meter (a negative delta on it); PK1-S2a (FB-167 Q8): a marrow-only one (`channel.only`,
    // the Channel basics Feed and Vent) moves it either way (a delta on it)
    for s in json::sget(e, "skills") {
        let row = refs.skills.iter().find(|r| json::s(r, "id") == s);
        let delta_on = |r: &Dict, ok: &dyn Fn(i64) -> bool| json::aget(r, "meters").iter().any(|m| json::s(json::dict(m), "meter") == id && ok(json::to_int(json::get(json::dict(m), "delta"))));
        match row {
            None => out.push(format!("Meter skills: unknown skill '{s}'.")),
            Some(r) if json::b(json::dget(r, "channel"), "only", false) => {
                if !delta_on(r, &|d| d != 0) {
                    out.push(format!("Meter skills: '{s}' does not move {id} (a meters delta on it)."));
                }
            }
            Some(r) if !delta_on(r, &|d| d < 0) => {
                out.push(format!("Meter skills: '{s}' does not spend {id} (a negative meters delta on it)."));
            }
            _ => {}
        }
    }
}

/// UM4: a band's `effects` (the passive grammar of BAND_EFFECTS; `crit` as the trigger effect).
fn band_effect_errors(raw: &Value, at: &str, refs: &Refs, out: &mut Vec<String>) {
    let list = match raw {
        Value::Null => return,
        Value::Array(a) => a,
        _ => {
            out.push(format!("{at}: effects must be a list."));
            return;
        }
    };
    let has_skill = |id: &str| refs.skills.iter().any(|r| json::s(r, "id") == id);
    for (i, item) in list.iter().enumerate() {
        let eat = format!("{at} effect {}", i + 1);
        let t = json::s(json::dict(item), "type");
        if !BAND_EFFECTS.contains(&t.as_str()) {
            out.push(format!("{eat}: '{t}' is not one of {}.", BAND_EFFECTS.join(", ")));
        } else if t == "crit" {
            if !whole_in(json::get(json::dict(item), "value"), super::hooks::CRIT) {
                out.push(format!("{eat}: crit needs a value {}-{}.", super::hooks::CRIT.0, super::hooks::CRIT.1));
            }
        } else if t == HEAL_TAKEN {
            // PK1-S1 (3.1): band-only; a percent of every heal the holder receives
            let o = json::dict(item);
            for k in o.keys().filter(|k| !["type", "value"].contains(&k.as_str())) {
                out.push(format!("{eat}: unknown key '{k}' (keys: type, value)."));
            }
            let v = json::get(o, "value");
            if !whole_in(v, HEAL_TAKEN_PCT) || json::to_int(v) == 0 {
                out.push(format!("{eat}: heal_taken needs a value {}..{}, not 0.", HEAL_TAKEN_PCT.0, HEAL_TAKEN_PCT.1));
            }
        } else {
            let ctx = Ctx { gear: super::skill::gear_tokens(refs.families), ..Ctx::default() };
            for m in super::simple::effect_errors(&Value::Array(vec![item.clone()]), &has_skill, &[], true, false, &ctx) {
                out.push(format!("{at}: {m}"));
            }
        }
    }
}

// ---- zones (ZN1-ZN2) ---------------------------------------------------------------------------------------------

fn zone_errors(e: &Dict, out: &mut Vec<String>) {
    let id = json::s(e, "id");
    if ZONE_IDS.contains(&id.as_str()) {
        out.push(format!("ID '{id}' is a code zone (the 8 code zones stay code: {}).", ZONE_IDS.join(", ")));
    }
    let rows = json::get(e, "rows");
    let ok = match rows {
        Value::Array(a) => a.is_empty() || (a.iter().all(Value::is_string) && rows_valid(&json::strings(rows))),
        _ => false,
    };
    if !ok {
        out.push(format!("Rows are [] (9 rows of floor) or {} strings of {} terrain glyphs.", rpgkit_battle::grid::GRID_H, rpgkit_battle::grid::GRID_W));
    }
    let backdrop = json::s(e, "backdrop");
    if !backdrop.is_empty() && !asset_key_ok(&backdrop) {
        out.push(format!("Backdrop '{backdrop}' must be \"\" or an asset key (a relative .png path, no spaces or ..)."));
    }
    let ground = json::s(e, "ground");
    if !TERRAIN_IDS.contains(&ground.as_str()) {
        out.push(format!("Ground: '{ground}' is not a terrain id ({}).", TERRAIN_IDS.join(", ")));
    }
}

/// An asset key: a relative `.png` path of lowercase letters, digits, `_`, `-` and `/`, without `..`.
fn asset_key_ok(key: &str) -> bool {
    key.ends_with(".png") && !key.starts_with('/') && !key.contains("..") && key.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-' | '/' | '.'))
}

/// The Forge's check of one entry (normalized) of `kind` as entry `current_id` ("" = a new entry): its own problems and
/// those against the loaded rows (its old row taken out); the entry is checked last.
pub fn entry_errors(v: &View, kind: &str, e: &Dict, current_id: &str) -> Vec<String> {
    let mut rows: Vec<Dict> = v.entries(kind).iter().filter(|r| current_id.is_empty() || json::s(r, "id") != current_id).cloned().collect();
    rows.push(e.clone());
    let derived = v.derived_ids();
    let meters = v.ids(UNIT_METERS_KIND);
    check_rows(kind, &rows, &Refs { skills: v.entries("skills"), derived: &derived, families: v.entries("item_families"), meters: Some(&meters) }).pop().unwrap_or_default()
}
