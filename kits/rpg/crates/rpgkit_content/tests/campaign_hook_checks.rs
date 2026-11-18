//! PR6-6 (`docs/sessions/PK4_PK5_CAMPAIGN_HUB.md` 1-2, B3-B5), content side of the campaign hooks, on values and a
//! scratch data folder built here (no shipped data): a `signed` unit track holds no stages and no cause; a recipe's
//! `tags` are ids; a battle input's and a battle trigger's `hero` filter is an object of known minimums; a trigger's
//! `recipe` / `recipe_tags` belong to `on: recipe` and the recipe must exist; a checkpoint's `after_flag` needs
//! `offset_days` 1..999 (and the reverse); `clock_set`'s `keep_longer` is true or false and no other clock effect takes
//! it. Each new field is `omit_default`: a row without it re-saves without it.

use aegis_core::json::{self, Dict};
use rpgkit_content::check::{campaign, recipe, unit_track, View};
use rpgkit_content::store;
use serde_json::{json, Value};
use std::fs;

fn dict(v: Value) -> Dict {
    json::dict(&v).clone()
}

fn view() -> View {
    let catalog = json!({"recipes": [{"id": "push", "name": "Push", "npc": "smith", "target": "member", "effects": []}]});
    let worlds = json!({"w": {"manifest": {"id": "w", "name": "W", "campaign": {"start_act": 1}}, "own": {}}});
    View::new(&catalog, &worlds, &json!({}), Default::default())
}

fn trigger(extra: Value) -> Vec<String> {
    let mut t = dict(json!({"id": "t", "name": "T", "on": "any", "outcome": "any", "when": [], "then": [], "effects": [{"effect": "set_flag", "key": "w.x"}]}));
    t.extend(dict(extra));
    campaign::trigger_errors(&view(), "w", &t)
}

fn checkpoint(extra: Value) -> Vec<String> {
    let mut c = dict(json!({"id": "cp", "name": "Cp", "act": 1, "deadline": 1, "met_when": [{"key": "w.done", "value": true}], "warn_days": [7, 3, 1],
        "mode": "go_on", "effects": [{"effect": "set_flag", "key": "w.missed"}]}));
    c.extend(dict(extra));
    campaign::checkpoint_errors(&view(), "w", &c)
}

#[test]
fn a_signed_track_holds_no_stages_and_no_cause() {
    let row = |extra: Value| {
        let mut r = dict(json!({"id": "lean", "name": "Lean", "mark": "", "cause": "", "stages": [], "origin": "system"}));
        r.extend(dict(extra));
        unit_track::row_errors(&r, &[])
    };
    assert_eq!(row(json!({"signed": true})), Vec::<String>::new());
    assert_eq!(row(json!({"signed": true, "stages": [{"name": "One", "passive": ""}]})).len(), 1, "stages on a signed track");
    assert_eq!(row(json!({"signed": true, "cause": "t_cause"})).len(), 1, "a cause on a signed track");
    assert_eq!(row(json!({"signed": "yes"})).len(), 2, "not a bool (and then an unsigned row needs stages)");
    assert_eq!(row(json!({"signed": false})).len(), 1, "an unsigned row still needs 1-9 stages");
}

#[test]
fn recipe_tags_are_ids() {
    let loaded = dict(json!({}));
    let row = |tags: Value| {
        let r = dict(json!({"id": "push", "name": "Push", "npc": "smith", "requires": [], "cost": {}, "target": "member", "effects": [{"effect": "doctrine", "axis": "doctrine", "delta": 4}], "origin": "system", "tags": tags}));
        recipe::row_errors(&r, &loaded)
    };
    assert_eq!(row(json!(["slab", "tier_3"])), Vec::<String>::new());
    assert_eq!(row(json!(["Not An Id"])).len(), 1);
    assert_eq!(row(json!("slab")).len(), 1);
}

#[test]
fn hero_filters_and_recipe_filters_belong_to_their_cause() {
    assert_eq!(trigger(json!({"on": "battle", "hero": {"strain_end": 50, "channel_actions": 1}})), Vec::<String>::new());
    assert_eq!(trigger(json!({"on": "recipe", "recipe": "push", "recipe_tags": ["slab"]})), Vec::<String>::new());
    assert_eq!(trigger(json!({"on": "day", "hero": {"strain_end": 50}})), ["Hero is for on battle only."]);
    assert_eq!(trigger(json!({"on": "battle", "hero": {"strain": 50}})), ["Hero: hero: unknown key 'strain'."]);
    assert_eq!(trigger(json!({"on": "battle", "hero": {"strain_end": -1}})), ["Hero: hero: strain_end must be a whole number 0..999."]);
    assert_eq!(trigger(json!({"on": "battle", "recipe_tags": ["slab"]})), ["Recipe and Recipe tags are for on recipe only."]);
    assert_eq!(trigger(json!({"on": "recipe", "recipe": "nope"})), ["Recipe: unknown recipe 'nope'."]);
    // a battle input's hero filter
    let meter = |input: Value| {
        let m = dict(json!({"id": "heat", "name": "Heat", "max": 999, "bands": [{"id": "low", "name": "Low", "from": 0, "tone": "calm"}],
            "rules": [], "inputs": [input], "modifiers": [], "floors": [], "hud": {}}));
        campaign::meter_errors(&view(), "w", &m)
    };
    assert_eq!(meter(json!({"on": "battle", "delta": 2, "hero": {"strain_end": 50}})), Vec::<String>::new());
    assert_eq!(meter(json!({"on": "battle", "delta": 2, "hero": {"nope": 1}})).len(), 1);
    assert_eq!(meter(json!({"on": "day", "delta": 2, "hero": {"strain_end": 50}})).len(), 1, "a day input takes no hero");
    // a recipe input moves by the recipe's tags and takes no hero
    assert_eq!(meter(json!({"on": "recipe", "tags": ["slab"], "delta": 3})), Vec::<String>::new());
    assert_eq!(meter(json!({"on": "recipe", "delta": 3, "hero": {"strain_end": 50}})).len(), 1);
}

#[test]
fn an_after_flag_checkpoint_needs_its_offset() {
    assert_eq!(checkpoint(json!({})), Vec::<String>::new());
    assert_eq!(checkpoint(json!({"after_flag": "w.boss_down", "offset_days": 10})), Vec::<String>::new());
    assert_eq!(checkpoint(json!({"after_flag": "w.boss_down"})), ["Offset days must be 1..999 for a checkpoint armed by a flag."]);
    assert_eq!(checkpoint(json!({"offset_days": 10})), ["Offset days is for a checkpoint armed by a flag (after_flag) only."]);
    assert_eq!(checkpoint(json!({"after_flag": "Bad Key", "offset_days": 10})), ["Armed by flag: 'Bad Key' is not a flag key."]);
    assert_eq!(checkpoint(json!({"after_flag": "meter.nope", "offset_days": 10})), ["Armed by flag: 'meter.nope' names no meter of this world."]);
}

#[test]
fn keep_longer_is_a_clock_set_bool() {
    let fx = |effect: &str, keep: Value| {
        let f = dict(json!({"effect": effect, "meter": "clock", "days": 5, "keep_longer": keep}));
        campaign::effect_errors(&view(), "w", effect, &f, "E")
    };
    assert!(fx("clock_set", json!(true)).iter().all(|e| !e.contains("keep_longer")));
    assert!(fx("clock_set", json!("yes")).contains(&"E: keep_longer must be true or false.".to_string()));
    let start = trigger(json!({"effects": [{"effect": "clock_start", "meter": "clock", "days": 5, "keep_longer": true}]}));
    assert!(start.iter().any(|e| e.ends_with("unknown key 'keep_longer'.")), "{start:?}");
}

#[test]
fn rows_without_the_new_fields_resave_without_them() {
    let t = aegis_core::scratch::Scratch::vacant("rpgkit_campaign_hook_checks");
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    let track = dict(json!({"id": "tt", "name": "Track", "stages": [{"name": "One", "passive": ""}]}));
    store::upsert(&data, None, rpgkit_data::UNIT_TRACKS_KIND, &track, None).expect("a valid track");
    let lean = dict(json!({"id": "lean", "name": "Lean", "signed": true, "stages": []}));
    store::upsert(&data, None, rpgkit_data::UNIT_TRACKS_KIND, &lean, None).expect("a valid signed track");
    let text = fs::read_to_string(data.join("catalog").join(format!("{}.json", rpgkit_data::UNIT_TRACKS_KIND))).unwrap();
    assert_eq!(text.matches("\"signed\"").count(), 1, "only the signed row writes signed: {text}");
    assert_eq!(store::resave_text(&data, None, rpgkit_data::UNIT_TRACKS_KIND).unwrap(), text, "a re-save is byte-identical");
    let _ = fs::remove_dir_all(&t);
}
