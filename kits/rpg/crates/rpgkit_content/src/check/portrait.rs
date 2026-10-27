//! Portrait cue rows (PORTRAITFX1-S, docs/specs/PORTRAITFX1.md 5.1; engine-only, no Godot original): the checks of
//! one row of the kind `portrait_cues` (id, the trigger and its match, the effect and its numbers, colour, expression,
//! priority, surfaces, rulesets, origin) and across rows (unique ids; two rows with the same trigger, match and effect
//! are an error on both). The loader keeps the rows that pass, in file order. A condition the kit does not know is a
//! warning, not an error (the game may name conditions before the kit carries them).

use super::View;
use crate::source::fields;
use aegis_core::assets::hex_colour;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_data::{EXPRESSIONS, MEMBER_CONDITIONS, PORTRAIT_COLOR_EFFECTS, PORTRAIT_EFFECTS, PORTRAIT_EVENTS, PORTRAIT_JOLTS, PORTRAIT_METER_TRIGGER, PORTRAIT_SURFACES, PORTRAIT_TRIGGERS};
use rpgkit_rules::ruleset::RULESETS;
use rpgkit_rules::status;

/// The catalog kind (file data/catalog/portrait_cues.json).
pub const KIND: &str = rpgkit_data::PORTRAIT_CUES_KIND;
/// The id's length range (a letter first, then a-z, 0-9 or _).
pub const ID_MIN: usize = 2;
pub const ID_MAX: usize = 32;
/// The longest condition id.
pub const CONDITION_MAX: usize = 24;
/// The `below` and `drop_pct` range (when used).
pub const PERCENT_MAX: i64 = 100;
/// An event row's `ms`: a flash, a jolt with its own duration (0 = the preset's), any other effect.
pub const FLASH_MS: (i64, i64) = (60, 1000);
pub const JOLT_MS: (i64, i64) = (60, 600);
pub const TIMED_MS: (i64, i64) = (100, 3000);
/// The intensity range.
pub const INTENSITY: (f64, f64) = (0.05, 1.0);
/// The priority range.
pub const PRIORITY_MAX: i64 = 100;
/// The refusal of the reserved trigger.
pub const METER_MESSAGE: &str = "Trigger 'meter' is reserved (unit meters: CAMPAIGN1-DOC).";

/// The effects that run once when fired: refused on state rows, which hold and never fire.
pub const ONE_SHOT_EFFECTS: [&str; 2] = ["flash", "jolt"];
/// The keys a row matches on besides its trigger (the cross-row check compares them with the effect).
const MATCH_KEYS: [&str; 6] = ["event", "status", "condition", "stat", "below", "drop_pct"];

/// What a row's checks read from the rest of the catalog: the derived-stat ids.
pub struct Refs<'a> {
    pub derived: &'a [String],
}

fn chars(s: &str) -> usize {
    s.chars().count()
}

/// `[a-z][a-z0-9_]{ID_MIN - 1, ID_MAX - 1}`.
fn id_ok(id: &str) -> bool {
    id.chars().next().is_some_and(|c| c.is_ascii_lowercase()) && (ID_MIN..=ID_MAX).contains(&chars(id)) && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// `[a-z_]{1, CONDITION_MAX}`.
fn condition_ok(c: &str) -> bool {
    (1..=CONDITION_MAX).contains(&chars(c)) && c.chars().all(|ch| ch.is_ascii_lowercase() || ch == '_')
}

/// The schema options of a field of the kind (origin).
fn options(key: &str) -> Vec<String> {
    fields("content", KIND).iter().map(json::dict).find(|f| json::s(f, "key") == key).map(|f| json::aget(f, "options").iter().map(gstr).collect()).unwrap_or_default()
}

/// The problems of each row (same order as the rows), on its own and against the others.
pub fn check_rows(rows: &[Dict], refs: &Refs) -> Vec<Vec<String>> {
    let ids: Vec<String> = rows.iter().map(|r| json::s(r, "id")).collect();
    let mut out: Vec<Vec<String>> = rows.iter().map(|r| row_errors(r, refs)).collect();
    for (i, found) in out.iter_mut().enumerate() {
        if !ids[i].is_empty() && ids[..i].contains(&ids[i]) {
            found.push(format!("ID '{}' is already used by another portrait cue.", ids[i]));
        }
        for (j, other) in rows.iter().enumerate() {
            if j != i && same_cue(&rows[i], other) {
                found.push(format!("Portrait cue '{}' has the same trigger, match and effect.", ids[j]));
            }
        }
    }
    out
}

/// Two rows with the same trigger, the same match and the same effect.
fn same_cue(a: &Dict, b: &Dict) -> bool {
    ["trigger", "effect"].iter().chain(MATCH_KEYS.iter()).all(|k| json::deep_equal(json::get(a, k), json::get(b, k)))
}

/// The own problems of one (normalized) row.
pub fn row_errors(e: &Dict, refs: &Refs) -> Vec<String> {
    let mut out = Vec::new();
    let id = json::s(e, "id");
    if id.is_empty() {
        out.push("ID is required.".into());
    } else if !id_ok(&id) {
        out.push(format!("ID '{id}' must be {ID_MIN}-{ID_MAX} characters of a-z, 0-9 and _, a letter first."));
    }
    let trigger = json::s(e, "trigger");
    if trigger == PORTRAIT_METER_TRIGGER {
        out.push(METER_MESSAGE.into());
    } else if !PORTRAIT_TRIGGERS.contains(&trigger.as_str()) {
        out.push(format!("Trigger: '{trigger}' is not one of {}.", PORTRAIT_TRIGGERS.join(", ")));
    }
    if PORTRAIT_TRIGGERS.contains(&trigger.as_str()) {
        // an unknown or reserved trigger has no match to check
        match_errors(e, &trigger, refs, &mut out);
    }
    let effect = json::s(e, "effect");
    let state = trigger != "event" && PORTRAIT_TRIGGERS.contains(&trigger.as_str());
    if state && ONE_SHOT_EFFECTS.contains(&effect.as_str()) {
        // a state row holds (ms 0) and never fires, so a one-shot reaction would never run
        out.push(format!("Effect {effect} is a one-shot reaction: it needs an event row (a {trigger} row holds and never fires it)."));
    } else if PORTRAIT_EFFECTS.contains(&effect.as_str()) {
        // an unknown effect has no numbers to check
        effect_errors(e, &trigger, &effect, &mut out);
    } else {
        out.push(format!("Effect: '{effect}' is not one of {}.", PORTRAIT_EFFECTS.join(", ")));
    }
    let intensity = json::f(e, "intensity", 1.0);
    if !(INTENSITY.0..=INTENSITY.1).contains(&intensity) {
        out.push(format!("Intensity must be {:?}-{:?} (is {intensity:?}).", INTENSITY.0, INTENSITY.1));
    }
    let priority = json::i(e, "priority", 50);
    if !(0..=PRIORITY_MAX).contains(&priority) {
        out.push(format!("Priority must be 0-{PRIORITY_MAX} (is {priority})."));
    }
    let surfaces: Vec<String> = PORTRAIT_SURFACES.iter().map(|s| s.to_string()).collect();
    // RULESET1-R: the ruleset ids are code-owned (rpgkit_rules::ruleset::RULESETS; the schema lists the same)
    let rulesets: Vec<String> = RULESETS.iter().map(|s| s.to_string()).collect();
    for (key, label, allowed) in [("surfaces", "Surfaces", &surfaces), ("rulesets", "Rulesets", &rulesets)] {
        let mut seen: Vec<String> = Vec::new();
        for v in json::sget(e, key) {
            if !allowed.contains(&v) {
                out.push(format!("{label}: '{v}' is not one of {}.", allowed.join(", ")));
            } else if seen.contains(&v) {
                out.push(format!("{label} lists '{v}' twice."));
            }
            seen.push(v);
        }
    }
    let origin = json::s(e, "origin");
    let origins = options("origin");
    if !origins.contains(&origin) {
        out.push(format!("Origin: '{origin}' is not one of {}.", origins.join(", ")));
    }
    out
}

/// The fields that depend on a known effect: the duration, the jolt preset, the colour, the expression.
fn effect_errors(e: &Dict, trigger: &str, effect: &str, out: &mut Vec<String>) {
    ms_errors(e, trigger, effect, out);
    let preset = json::s(e, "preset");
    match (effect == "jolt", preset.is_empty()) {
        (true, true) => out.push(format!("Jolt preset: a jolt needs one of {}.", PORTRAIT_JOLTS.join(", "))),
        (true, false) if !PORTRAIT_JOLTS.contains(&preset.as_str()) => out.push(format!("Jolt preset: '{preset}' is not one of {}.", PORTRAIT_JOLTS.join(", "))),
        (false, false) => out.push("Jolt preset is for a jolt only.".into()),
        _ => {}
    }
    let color = json::s(e, "color");
    if !color.is_empty() {
        if hex_colour(&color).is_none() {
            out.push(format!("Colour '{color}' must be \"\" or #rrggbb."));
        }
        if !PORTRAIT_COLOR_EFFECTS.contains(&effect) {
            out.push(format!("Colour is only for {} (not {effect}).", PORTRAIT_COLOR_EFFECTS.join(", ")));
        }
    }
    let expression = json::s(e, "expression");
    match (effect == "face", expression.is_empty()) {
        (true, true) => out.push(format!("Expression: a face row needs one of {}.", EXPRESSIONS.join(", "))),
        (true, false) if !EXPRESSIONS.contains(&expression.as_str()) => out.push(format!("Expression '{expression}' is not one of {}.", EXPRESSIONS.join(", "))),
        (false, false) => out.push("Expression is for a face row only.".into()),
        _ => {}
    }
}

/// The match keys: the one the trigger needs is set and valid, the others hold their defaults.
fn match_errors(e: &Dict, trigger: &str, refs: &Refs, out: &mut Vec<String>) {
    let event = json::s(e, "event");
    if trigger == "event" {
        if event.is_empty() {
            out.push(format!("Event: an event row needs one of {}.", PORTRAIT_EVENTS.join(", ")));
        } else if !PORTRAIT_EVENTS.contains(&event.as_str()) {
            out.push(format!("Event: '{event}' is not one of {}.", PORTRAIT_EVENTS.join(", ")));
        }
    } else if !event.is_empty() {
        out.push("Event is for an event row only.".into());
    }
    let st = json::s(e, "status");
    if trigger == "status" {
        if st.is_empty() {
            out.push("Status: a status row needs a status id.".into());
        } else if status::row_of(&st).is_none() {
            out.push(format!("Status: unknown status '{st}'."));
        }
    } else if !st.is_empty() {
        out.push("Status is for a status row only.".into());
    }
    let condition = json::s(e, "condition");
    if trigger == "condition" {
        if condition.is_empty() {
            out.push("Condition: a condition row needs a condition id.".into());
        } else if !condition_ok(&condition) {
            out.push(format!("Condition '{condition}' must be 1-{CONDITION_MAX} characters of a-z and _."));
        }
    } else if !condition.is_empty() {
        out.push("Condition is for a condition row only.".into());
    }
    let stat = json::s(e, "stat");
    if trigger == "derived" {
        if stat.is_empty() {
            out.push("Derived stat: a derived row needs a derived stat id.".into());
        } else if !refs.derived.contains(&stat) {
            out.push(format!("Derived stat: unknown derived stat '{stat}'."));
        }
    } else if !stat.is_empty() {
        out.push("Derived stat is for a derived row only.".into());
    }
    let (below, drop) = (json::i(e, "below", 0), json::i(e, "drop_pct", 0));
    let in_range = |n: i64| (1..=PERCENT_MAX).contains(&n);
    match trigger {
        "hp" if !in_range(below) => out.push(format!("Below must be 1-{PERCENT_MAX} on an hp row (is {below}).")),
        "derived" if below != 0 && !in_range(below) => out.push(format!("Below must be 1-{PERCENT_MAX} on a derived row (is {below}).")),
        "hp" | "derived" => {}
        _ if below != 0 => out.push("Below is for hp and derived rows only.".into()),
        _ => {}
    }
    match trigger {
        "derived" if drop != 0 && !in_range(drop) => out.push(format!("Drop % must be 1-{PERCENT_MAX} (is {drop}).")),
        "derived" => {}
        _ if drop != 0 => out.push("Drop % is for a derived row only.".into()),
        _ => {}
    }
    if trigger == "derived" && (below != 0) == (drop != 0) {
        out.push("A derived row sets exactly one of Below and Drop %.".into());
    }
}

/// `ms`: an event row's duration by effect; a state row holds (0).
fn ms_errors(e: &Dict, trigger: &str, effect: &str, out: &mut Vec<String>) {
    let ms = json::i(e, "ms", 0);
    if trigger != "event" {
        if ms != 0 && PORTRAIT_TRIGGERS.contains(&trigger) {
            out.push(format!("Duration ms must be 0 on a state row (it holds while its {trigger} matches; is {ms})."));
        }
        return;
    }
    let within = |(lo, hi): (i64, i64)| (lo..=hi).contains(&ms);
    match effect {
        "flash" if !within(FLASH_MS) => out.push(format!("Duration ms must be {}-{} on an event flash (is {ms}).", FLASH_MS.0, FLASH_MS.1)),
        "jolt" if ms != 0 && !within(JOLT_MS) => out.push(format!("Duration ms must be 0 (the preset's) or {}-{} on an event jolt (is {ms}).", JOLT_MS.0, JOLT_MS.1)),
        "flash" | "jolt" => {}
        _ if !within(TIMED_MS) => out.push(format!("Duration ms must be {}-{} on an event {effect} (is {ms}).", TIMED_MS.0, TIMED_MS.1)),
        _ => {}
    }
}

/// The soft problems of one row: a condition the kit does not know (ROSTER1 W1).
pub fn warnings(e: &Dict) -> Vec<String> {
    let condition = json::s(e, "condition");
    if json::s(e, "trigger") == "condition" && condition_ok(&condition) && !MEMBER_CONDITIONS.contains(&condition.as_str()) {
        vec![format!("Condition '{condition}' is not a kit condition ({}); no member carries it.", MEMBER_CONDITIONS.join(", "))]
    } else {
        Vec::new()
    }
}

/// The Forge's check of one entry (normalized) as entry `current_id` ("" = a new entry): its own problems and those
/// against the loaded rows (its old row taken out); the entry is checked last.
pub fn entry_errors(v: &View, e: &Dict, current_id: &str) -> Vec<String> {
    let mut rows: Vec<Dict> = v.entries(KIND).iter().filter(|r| current_id.is_empty() || json::s(r, "id") != current_id).cloned().collect();
    rows.push(e.clone());
    let derived = v.derived_ids();
    check_rows(&rows, &Refs { derived: &derived }).pop().unwrap_or_default()
}
