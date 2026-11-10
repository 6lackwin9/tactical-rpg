//! The story checks (STORYUI1 4.1-4.3, 4.6, kit hook; engine-only, no Godot original): the rows of the world kinds
//! `vignettes` (their cards) and `sequences` (their steps), a world manifest's `start_sequence`, and the soft warnings
//! for card images, cues and music that no asset layer holds. Worlds without the story kinds report as before.
//! PORTRAITFX1 7: a `line` card's `expression`. STORYMUSIC1: a vignette's and a step's `music` (a music key too).

use super::campaign::read_key_error;
use super::dialogue::{self, EffectList};
use super::quest::flag_key_ok;
use super::world::ref_errors;
use super::View;
use aegis_content::schema::is_id;
use aegis_core::assets::hex_colour;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_data::{
    music_ask, split_token, step_keys, MusicAsk, AUDIO_DIR, CARDS_MAX, CARD_KEYS, CARD_STYLES, CARD_SUBTITLE_MAX, CARD_TEXT_MAX, CARD_TITLE_MAX, END_TO, FADE_MS_MAX, FIGHT_OUTCOMES, HOLD_MS_MAX, LAST_STEP_KINDS, MUSIC_DIR, MUSIC_EXT, MUSIC_NONE, STEPS_MAX, STEP_EFFECTS_MAX, STEP_KINDS, TOKEN_KINDS,
};
use serde_json::Value;

fn int_in(v: &Value, lo: i64, hi: i64) -> bool {
    json::is_int(v) && (lo..=hi).contains(&json::to_int(v))
}

fn chars(s: &str) -> usize {
    s.chars().count()
}

/// A required world ref held in `d[key]`: "choose a ..." when empty, else the ref's problems.
fn required_ref(v: &View, w: &str, d: &Dict, key: &str, ref_kind: &str, label: &str) -> Vec<String> {
    match d.get(key).map(gstr).filter(|r| !r.is_empty()) {
        None => {
            let what = ref_kind.strip_suffix('s').unwrap_or(ref_kind);
            let article = if what.starts_with(['a', 'e', 'i', 'o', 'u']) { "an" } else { "a" };
            vec![format!("{label}: choose {article} {what}.")]
        }
        Some(r) => ref_errors(v, w, &format!("{label} {key}"), ref_kind, &r),
    }
}

/// `d[key]`, when present, is text of at most `max` characters.
fn text_key(d: &Dict, key: &str, max: usize, label: &str) -> Option<String> {
    match d.get(key)?.as_str() {
        None => Some(format!("{label}: {key} must be text.")),
        Some(t) if chars(t) > max => Some(format!("{label}: {key} is longer than {max} characters.")),
        Some(_) => None,
    }
}

/// `d[key]`, when present, is one of `options`.
fn enum_key(d: &Dict, key: &str, options: &[&str], label: &str) -> Option<String> {
    let x = d.get(key)?;
    (!x.as_str().is_some_and(|s| options.contains(&s))).then(|| format!("{label}: {key} '{}' is not one of {}.", gstr(x), options.join(", ")))
}

// ---- 4.1, 4.2 vignettes and cards ----

/// STORYMUSIC1: the problem of a vignette's or a step's `music`: empty (the music goes on), `none` (silence), a track
/// ending `.ogg`, or a music key: one of the game's (`View::music_keys`, its manifest's `music_keys`), or the key of a
/// track the asset layers hold (`audio/music/<key>.ogg`). A listed key needs no track (the music playing goes on until
/// one ships).
pub fn music_error(v: &View, music: &str, label: &str) -> Option<String> {
    let MusicAsk::Key(key) = music_ask(music) else { return None };
    let shape = format!("{label} '{music}' must be empty (the music goes on), {MUSIC_NONE} (silence) or a track ending {MUSIC_EXT}, or name a music key (a-z, 0-9, _)");
    if !is_id(key) {
        return Some(format!("{shape}."));
    }
    if v.music_keys.iter().any(|k| k == key) || v.assets.has(&format!("{AUDIO_DIR}{MUSIC_DIR}{key}{MUSIC_EXT}")) {
        return None;
    }
    let listed = if v.music_keys.is_empty() { "none".to_string() } else { v.music_keys.join(", ") };
    Some(format!("{label}: '{key}' is no music key of the game (game.json {}: {listed}) and no track {MUSIC_DIR}{key}{MUSIC_EXT} in the asset folders' {AUDIO_DIR}.", crate::source::MUSIC_KEYS))
}

/// The problems of a vignette row (4.1) and its cards (4.2).
pub fn vignette_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut errors: Vec<String> = music_error(v, &json::s(e, "music"), "Music").into_iter().collect();
    let frame = json::s(e, "frame");
    if hex_colour(&frame).is_none() {
        errors.push(format!("Frame colour '{frame}' must be #rrggbb."));
    }
    let Some(cards) = json::get(e, "cards").as_array() else {
        errors.push("Cards must be a list.".into());
        return errors;
    };
    if cards.is_empty() || cards.len() > CARDS_MAX {
        errors.push(format!("A vignette holds 1-{CARDS_MAX} cards (has {}).", cards.len()));
    }
    for (i, raw) in cards.iter().enumerate() {
        errors.extend(card_errors(v, w, i, raw));
    }
    errors
}

/// The problems of card `i` (4.2).
fn card_errors(v: &View, w: &str, i: usize, raw: &Value) -> Vec<String> {
    let label = format!("Card {}", i + 1);
    let Some(c) = raw.as_object() else {
        return vec![format!("{label} must be an object {{\"style\", ...}}.")];
    };
    let mut errors: Vec<String> = c.keys().filter(|k| !CARD_KEYS.contains(&k.as_str())).map(|k| format!("{label}: unknown key '{k}'.")).collect();
    let style = c.get("style").map(gstr).unwrap_or_else(|| CARD_STYLES[0].to_string());
    if !CARD_STYLES.contains(&style.as_str()) {
        errors.push(format!("{label}: style '{style}' is not one of {}.", CARD_STYLES.join(", ")));
    }
    for (key, max) in [("image", usize::MAX), ("title", CARD_TITLE_MAX), ("subtitle", CARD_SUBTITLE_MAX), ("text", CARD_TEXT_MAX), ("speaker", usize::MAX), ("cue", usize::MAX)] {
        errors.extend(text_key(c, key, max, &label));
    }
    let has = |key: &str| !json::s(c, key).trim().is_empty();
    if has("image") && style != "image" && CARD_STYLES.contains(&style.as_str()) {
        errors.push(format!("{label}: image is for an image card only."));
    }
    for key in ["top", "bottom"] {
        if let Some(x) = c.get(key).filter(|x| x.as_str().and_then(hex_colour).is_none()) {
            errors.push(format!("{label}: {key} '{}' must be #rrggbb.", gstr(x)));
        }
    }
    if c.get("ash").is_some_and(|x| !x.is_boolean()) {
        errors.push(format!("{label}: ash must be true or false."));
    }
    match style.as_str() {
        "title" if !has("title") => errors.push(format!("{label}: a title card needs a title.")),
        "text" | "line" if !has("text") => errors.push(format!("{label}: a {style} card needs text.")),
        _ => {}
    }
    errors.extend(token_errors(v, w, &json::s(c, "text"), &label));
    let speaker = json::s(c, "speaker");
    match (style == "line", speaker.is_empty()) {
        (true, true) => errors.push(format!("{label}: a line card needs a speaker.")),
        (false, false) => errors.push(format!("{label}: speaker is for a line card only.")),
        (true, false) => errors.extend(ref_errors(v, w, &format!("{label} speaker"), "characters", &speaker)),
        (false, true) => {}
    }
    // PORTRAITFX1 7: the speaker's face on a line card
    if c.get("expression").is_some_and(|x| x.as_str() != Some("")) && style != "line" && CARD_STYLES.contains(&style.as_str()) {
        errors.push(format!("{label}: expression is for a line card only."));
    } else {
        errors.extend(dialogue::expression_error(c, &label, &v.faces()));
    }
    // FX1 5.1: the card's camera shake ("" = none)
    if let Some(x) = c.get("shake").filter(|x| !x.as_str().is_some_and(|s| s.is_empty() || rpgkit_data::SHAKE_PRESETS.contains(&s))) {
        errors.push(format!("{label}: shake '{}' is not one of {} (or \"\").", gstr(x), rpgkit_data::SHAKE_PRESETS.join(", ")));
    }
    for (key, max) in [("fade_in_ms", FADE_MS_MAX), ("fade_out_ms", FADE_MS_MAX), ("hold_ms", HOLD_MS_MAX)] {
        if c.get(key).is_some_and(|x| !int_in(x, 0, max)) {
            errors.push(format!("{label}: {key} must be a whole number 0..{max}."));
        }
    }
    if let Some(when) = c.get("when") {
        errors.extend(dialogue::conditions_errors(v, w, when, &format!("{label} when")));
    }
    errors
}

/// W3: the tokens of a card text: `{flag:KEY}` and `{member:KEY}` with a flag key (a reserved key must be a read key).
fn token_errors(v: &View, w: &str, text: &str, label: &str) -> Vec<String> {
    let mut errors = Vec::new();
    for token in dialogue::tokens_of(text) {
        match split_token(&token) {
            Some((kind, key)) if TOKEN_KINDS.contains(&kind) && flag_key_ok(key) => {
                let why = read_key_error(v, w, key);
                if !why.is_empty() {
                    errors.push(format!("{label}: token '{{{token}}}': {why}"));
                }
            }
            _ => errors.push(format!("{label}: unknown token '{{{token}}}' (card texts allow {{flag:KEY}} and {{member:KEY}}).")),
        }
    }
    errors
}

// ---- 4.3 sequences and steps ----

/// The problems of a sequence row (4.3).
pub fn sequence_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let Some(steps) = json::get(e, "steps").as_array() else {
        return vec!["Steps must be a list.".into()];
    };
    let mut errors = Vec::new();
    if steps.is_empty() || steps.len() > STEPS_MAX {
        errors.push(format!("A sequence holds 1-{STEPS_MAX} steps (has {}).", steps.len()));
    }
    for (i, raw) in steps.iter().enumerate() {
        errors.extend(step_errors(v, w, i, raw, i + 1 == steps.len()));
    }
    errors
}

/// The problems of step `i` (`last`: the sequence's last step).
fn step_errors(v: &View, w: &str, i: usize, raw: &Value, last: bool) -> Vec<String> {
    let Some(d) = raw.as_object() else {
        return vec![format!("Step {} must be an object {{\"kind\", ...}}.", i + 1)];
    };
    let kind = json::s(d, "kind");
    if !STEP_KINDS.contains(&kind.as_str()) {
        return vec![format!("Step {}: kind '{kind}' is not one of {}.", i + 1, STEP_KINDS.join(", "))];
    }
    let label = format!("Step {} ({kind})", i + 1);
    let allowed = step_keys(&kind);
    let mut errors: Vec<String> = d.keys().filter(|k| !allowed.contains(&k.as_str())).map(|k| format!("{label}: unknown key '{k}'.")).collect();
    // STORYMUSIC1: the step's music, as a vignette's
    match d.get("music") {
        Some(Value::String(m)) => errors.extend(music_error(v, m, &format!("{label}: music"))),
        Some(other) => errors.push(format!("{label}: music must be text (has {other}).")),
        None => {}
    }
    match kind.as_str() {
        "vignette" => errors.extend(required_ref(v, w, d, "vignette", "vignettes", &label)),
        "fight" => {
            errors.extend(required_ref(v, w, d, "encounter", "encounters", &label));
            errors.extend(enum_key(d, "on_lost", &FIGHT_OUTCOMES, &label));
            errors.extend(enum_key(d, "on_fled", &FIGHT_OUTCOMES, &label));
        }
        "effects" => {
            let raw = d.get("effects").cloned().unwrap_or(Value::Array(Vec::new()));
            if raw.as_array().is_some_and(Vec::is_empty) {
                errors.push(format!("{label}: an effects step needs 1-{STEP_EFFECTS_MAX} effects."));
            }
            errors.extend(dialogue::list_errors(v, w, &raw, &label, EffectList::Sequence));
        }
        "enter" => {
            let found = required_ref(v, w, d, "location", "locations", &label);
            let ok = found.is_empty();
            errors.extend(found);
            errors.extend(text_key(d, "anchor", usize::MAX, &label));
            let anchor = json::s(d, "anchor");
            if ok && !anchor.is_empty() {
                errors.extend(anchor_error(v, w, &json::s(d, "location"), &anchor, &label));
            }
        }
        _ => errors.extend(enum_key(d, "to", &END_TO, &label)),
    }
    if !last && LAST_STEP_KINDS.contains(&kind.as_str()) {
        errors.push(format!("{label}: {} may only be the last step.", LAST_STEP_KINDS.join(" and ")));
    }
    errors
}

/// An `enter` step's anchor must be an anchor of the location's map.
fn anchor_error(v: &View, w: &str, location: &str, anchor: &str, label: &str) -> Option<String> {
    let map_id = json::s(v.worlds.resolve(w, "locations", location), "map");
    let template = v.get("maps", &map_id);
    if template.is_empty() {
        return Some(format!("{label}: anchor '{anchor}' needs a location with a map ('{location}' has none)."));
    }
    let found = json::aget(template, "anchors").iter().any(|a| json::s(json::dict(a), "id") == anchor);
    (!found).then(|| format!("{label}: anchor '{anchor}' is not an anchor of map '{map_id}'."))
}

// ---- 4.6 manifest ----

/// The problems of a world manifest's `start_sequence` ("" = the map, as before).
pub fn manifest_errors(v: &View, w: &str, m: &Dict) -> Vec<String> {
    let r = json::s(m, "start_sequence");
    if r.is_empty() {
        return Vec::new();
    }
    ref_errors(v, w, "Start sequence", "sequences", &r)
}

// ---- warnings ----

/// The soft problems of world `w`'s vignettes and sequence steps: music tracks, card images and cues that no asset
/// layer holds (3.1: warnings in the asset checks' style, not errors).
pub fn warnings(v: &View, w: &str) -> Vec<String> {
    let mut out = Vec::new();
    if v.assets.roots.is_empty() {
        return out;
    }
    let track_missing = |music: &str| matches!(music_ask(music), MusicAsk::Track(m) if !v.assets.has(&format!("{AUDIO_DIR}{m}")));
    for e in v.worlds.own_entries(w, "sequences") {
        // STORYMUSIC1: a step's music track, as a vignette's
        for (i, step) in json::aget(e, "steps").iter().enumerate() {
            let music = json::s(json::dict(step), "music");
            if track_missing(&music) {
                out.push(format!("{}: step {}: music '{music}' is in no asset folder's {AUDIO_DIR}.", json::s(e, "_qid"), i + 1));
            }
        }
    }
    for e in v.worlds.own_entries(w, "vignettes") {
        let qid = json::s(e, "_qid");
        let music = json::s(e, "music");
        if track_missing(&music) {
            out.push(format!("{qid}: music '{music}' is in no asset folder's {AUDIO_DIR}."));
        }
        for (i, card) in json::aget(e, "cards").iter().enumerate() {
            let c = json::dict(card);
            let image = json::s(c, "image");
            if !image.is_empty() && !v.assets.has(&image) {
                out.push(format!("{qid}: card {}: image '{image}' is in no asset folder (the backdrop shows).", i + 1));
            }
            let cue = json::s(c, "cue");
            if !cue.is_empty() && !v.assets.has(&format!("{AUDIO_DIR}{cue}")) {
                out.push(format!("{qid}: card {}: cue '{cue}' is in no asset folder's {AUDIO_DIR}.", i + 1));
            }
        }
    }
    out
}
