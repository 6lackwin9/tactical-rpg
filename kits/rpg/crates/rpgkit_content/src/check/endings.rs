//! Ending rules of the world checks (PR6-11, `docs/sessions/PK6_PK7_BOSSES_ENDINGS.md` section 4; engine-only, no Godot
//! original; the tone and the art are field-table checks): an `endings` row's title, gates (flag conditions, meter
//! conditions on the world's meters), epilogue paragraphs, outcome lines and member fate template, and an `end`
//! effect's `ending` (kind `ending` only: a known row, and a world whose last ending has no gates, so every finish
//! reaches one).

use super::campaign::{text_key, unknown_keys};
use super::quest::condition_errors;
use super::world::ref_errors;
use super::View;
use aegis_core::json::{self, Dict};
use rpgkit_data::ending::{
    EndingRow, END_ENDING_KEY, END_KIND_ENDING, ENDINGS_KIND, FATE_KEYS, LINE_MAX, METER_CONDS_MAX, METER_COND_KEYS, METER_OPS, OUTCOMES_MAX, OUTCOME_KEYS, PARAGRAPHS_MAX, PARAGRAPH_MAX, TITLE_MAX,
};
use serde_json::Value;

const L: &str = "Endings";

/// The problems of an `endings` row in world `w` beyond its field table (the tone enum and the art ref are checked
/// there).
pub fn ending_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    errors.extend(text_key(e, "title", TITLE_MAX, false, L));
    if let Some(c) = e.get("requires_flags") {
        errors.extend(condition_errors(c, &format!("{L} requires flags")));
    }
    errors.extend(meter_errors(v, w, e.get("requires_meter")));
    errors.extend(paragraph_errors(e.get("epilogue_paragraphs")));
    errors.extend(outcome_errors(e.get("world_outcomes")));
    errors.extend(fate_errors(v, e.get("member_fates_template")));
    errors
}

/// `requires_meter`: at most `METER_CONDS_MAX` `{id, op, value}` on meters of the world.
fn meter_errors(v: &View, w: &str, raw: Option<&Value>) -> Vec<String> {
    let Some(raw) = raw else { return Vec::new() };
    let Some(list) = raw.as_array() else {
        return vec![format!("{L}: requires_meter must be a list of {{id, op, value}}.")];
    };
    let mut errors = Vec::new();
    if list.len() > METER_CONDS_MAX {
        errors.push(format!("{L}: at most {METER_CONDS_MAX} meter conditions."));
    }
    for (i, c) in list.iter().enumerate() {
        let el = format!("{L} meter condition {}", i + 1);
        let Some(c) = c.as_object() else {
            errors.push(format!("{el}: must be an object {{id, op, value}}."));
            continue;
        };
        errors.extend(unknown_keys(c, &METER_COND_KEYS, &el));
        let id = json::s(c, "id");
        if id.is_empty() {
            errors.push(format!("{el}: choose a meter."));
        } else {
            errors.extend(ref_errors(v, w, &format!("{el} meter"), "meters", &id));
        }
        let op = json::s_or(c, "op", ">=");
        if !METER_OPS.contains(&op.as_str()) {
            errors.push(format!("{el}: op '{op}' is not one of {}.", METER_OPS.join(", ")));
        }
        if !json::is_int(json::get(c, "value")) {
            errors.push(format!("{el}: value must be a whole number."));
        }
    }
    errors
}

/// `epilogue_paragraphs`: 1..`PARAGRAPHS_MAX` texts of at most `PARAGRAPH_MAX` characters, none blank.
fn paragraph_errors(raw: Option<&Value>) -> Vec<String> {
    let Some(list) = raw.and_then(Value::as_array) else {
        return vec![format!("{L}: epilogue_paragraphs must be a list of 1..{PARAGRAPHS_MAX} texts.")];
    };
    let mut errors = Vec::new();
    if list.is_empty() || list.len() > PARAGRAPHS_MAX {
        errors.push(format!("{L}: epilogue_paragraphs must hold 1..{PARAGRAPHS_MAX} texts."));
    }
    for (i, p) in list.iter().enumerate() {
        match p.as_str() {
            Some(t) if t.trim().is_empty() => errors.push(format!("{L} paragraph {}: must not be empty.", i + 1)),
            Some(t) if t.chars().count() > PARAGRAPH_MAX => errors.push(format!("{L} paragraph {}: is longer than {PARAGRAPH_MAX} characters.", i + 1)),
            Some(_) => {}
            None => errors.push(format!("{L} paragraph {}: must be text.", i + 1)),
        }
    }
    errors
}

/// `world_outcomes`: at most `OUTCOMES_MAX` `{flag, op?, value?, text_if_true, text_if_false}`, each a flag condition
/// with at least one text.
fn outcome_errors(raw: Option<&Value>) -> Vec<String> {
    let Some(raw) = raw else { return Vec::new() };
    let Some(list) = raw.as_array() else {
        return vec![format!("{L}: world_outcomes must be a list of {{flag, text_if_true, text_if_false}}.")];
    };
    let mut errors = Vec::new();
    if list.len() > OUTCOMES_MAX {
        errors.push(format!("{L}: at most {OUTCOMES_MAX} world outcomes."));
    }
    for (i, o) in list.iter().enumerate() {
        let el = format!("{L} outcome {}", i + 1);
        let Some(o) = o.as_object() else {
            errors.push(format!("{el}: must be an object {{flag, text_if_true, text_if_false}}."));
            continue;
        };
        errors.extend(unknown_keys(o, &OUTCOME_KEYS, &el));
        // the flag condition, checked as one
        let mut cond = json::obj([("key", o.get("flag").cloned().unwrap_or(Value::from("")))]);
        for k in ["op", "value"] {
            if let Some(x) = o.get(k) {
                cond.insert(k.into(), x.clone());
            }
        }
        errors.extend(condition_errors(&Value::Array(vec![Value::Object(cond)]), &el));
        for k in ["text_if_true", "text_if_false"] {
            errors.extend(text_key(o, k, LINE_MAX, false, &el));
        }
        if json::s(o, "text_if_true").trim().is_empty() && json::s(o, "text_if_false").trim().is_empty() {
            errors.push(format!("{el}: needs text_if_true or text_if_false."));
        }
    }
    errors
}

/// `member_fates_template`: `{track?, member, departed, causes?}`: a known unit track, texts of at most `LINE_MAX`
/// characters, causes `{cause: text}`.
fn fate_errors(v: &View, raw: Option<&Value>) -> Vec<String> {
    let Some(raw) = raw else { return Vec::new() };
    let Some(t) = raw.as_object() else {
        return vec![format!("{L}: member_fates_template must be an object {{{}}}.", FATE_KEYS.join(", "))];
    };
    let el = format!("{L} member fates");
    let mut errors = unknown_keys(t, &FATE_KEYS, &el);
    let track = json::s(t, "track");
    if !track.is_empty() && !v.has(rpgkit_data::unit_track::UNIT_TRACKS_KIND, &track) {
        errors.push(format!("{el}: unknown unit track '{track}'."));
    }
    for k in ["member", "departed"] {
        errors.extend(text_key(t, k, LINE_MAX, false, &el));
    }
    match t.get("causes") {
        None => {}
        Some(Value::Object(m)) => {
            for (cause, line) in m {
                if !line.as_str().is_some_and(|s| s.chars().count() <= LINE_MAX) {
                    errors.push(format!("{el}: cause '{cause}' must be text of at most {LINE_MAX} characters."));
                }
            }
        }
        Some(_) => errors.push(format!("{el}: causes must be an object {{cause: text}}.")),
    }
    errors
}

/// The problems of an `end` effect's `ending` (`el`: the effect's label): only kind `ending` takes it; it names a
/// known row; kind `ending` needs the world's endings to end on a row without gates (the fallback).
pub fn end_ending_errors(v: &View, w: &str, fx: &Dict, el: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let kind = json::s(fx, "kind");
    let named = json::s(fx, END_ENDING_KEY);
    if kind != END_KIND_ENDING {
        if fx.contains_key(END_ENDING_KEY) {
            errors.push(format!("{el}: {END_ENDING_KEY} is only for kind {END_KIND_ENDING}."));
        }
        return errors;
    }
    if !named.is_empty() {
        errors.extend(ref_errors(v, w, &format!("{el} ending"), ENDINGS_KIND, &named));
    }
    match v.worlds.merged_entries(w, ENDINGS_KIND).last() {
        None => errors.push(format!("{el}: the world has no endings to finish on.")),
        Some(last) if !EndingRow::from_obj(last).ungated() => errors.push(format!("{el}: the world's last ending must have no requires (the fallback every finish reaches).")),
        Some(_) => {}
    }
    errors
}
