//! Portrait cues (PORTRAITFX1-S, docs/specs/PORTRAITFX1.md 5.1, 5.4, 5.5 and 7; checks S1, S2, S6, S7 of
//! PORTRAITFX1_TASKS.md 1.2): the schema rows carry the kit's vocabulary; each rule of 5.1 refuses its bad row with its
//! own message and the reserved trigger names CAMPAIGN1-DOC; a condition the kit does not know warns; two rows with the
//! same trigger, match and effect are errors on both; the loader keeps the passing rows beside the catalog kinds and
//! the writer stores only the keys that differ from the defaults; the face chain and the sidecars a found portrait
//! reads; the asset checks A11 (the face tool's fixture list) and A12; the `expression` of a dialogue node and a `line`
//! card. Scratch data and asset layers only.

use aegis_content::schema::{fields, normalize};
use aegis_core::assets::Layers;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_content::check::{self, assets, portrait};
use rpgkit_content::{art, source, store};
use rpgkit_data::{PortraitCue, DEFAULT_FACES, PORTRAIT_EFFECTS, PORTRAIT_EVENTS, PORTRAIT_JOLTS, PORTRAIT_SURFACES, PORTRAIT_TRIGGERS};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const W: &str = "ward";
/// The derived stat the scratch catalog defines (a `derived` row names it).
const STAT: &str = "nerve";

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_portrait_cues_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds").join(W)).unwrap();
    (t, data)
}

fn drop_scratch(dir: &Path) {
    let _ = fs::remove_dir_all(dir.parent().unwrap());
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

fn write_catalog(data: &Path, kind: &str, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
}

fn strs(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

/// The kit's default faces (a catalog without `portrait_expressions` rows: PORTRAITFX2 FL1).
fn faces() -> Vec<String> {
    strs(&DEFAULT_FACES)
}

/// A row as the loader normalizes it.
fn row(v: &Value) -> Dict {
    normalize(fields(source::schema(), "content", portrait::KIND), v.as_object().unwrap(), false)
}

/// The own problems of one row (the scratch catalog's derived stat known).
fn errors(v: &Value) -> Vec<String> {
    let derived = vec![STAT.to_string()];
    portrait::row_errors(&row(v), &portrait::Refs { derived: &derived, faces: &faces() })
}

/// A status id of the kit's status table.
fn a_status() -> &'static str {
    rpgkit_rules::status::ROWS[0].id
}

/// One good row per trigger and effect family (the shape of PORTRAITFX1_TASKS 2.3 and 2.4.3).
fn good_rows() -> Vec<Value> {
    vec![
        json!({"id": "hit_flash", "event": "hit", "intensity": 0.6, "ms": 140, "color": "#d9cbb4"}),
        json!({"id": "hit_jolt", "event": "hit", "effect": "jolt", "preset": "hit"}),
        json!({"id": "marked", "trigger": "status", "status": a_status(), "effect": "fade", "intensity": 0.6}),
        json!({"id": "wounded", "trigger": "condition", "condition": "wounded", "effect": "desaturate", "intensity": 0.35}),
        json!({"id": "hp_half", "trigger": "hp", "below": 50, "effect": "desaturate", "intensity": 0.2}),
        json!({"id": "downed_still", "trigger": "down", "effect": "still"}),
        json!({"id": "shaken", "trigger": "derived", "stat": STAT, "drop_pct": 25, "effect": "tint", "intensity": 0.5, "color": "#06b6d4"}),
        json!({"id": "face_hit", "event": "hit", "effect": "face", "expression": "hurt", "ms": 900, "priority": 60}),
    ]
}

/// The good row `id` with `key` set to `value`.
fn changed(id: &str, key: &str, value: Value) -> Value {
    let mut r = good_rows().into_iter().find(|r| r["id"] == id).unwrap();
    r[key] = value;
    r
}

fn options(key: &str) -> Vec<String> {
    let f = fields(source::schema(), "content", portrait::KIND).iter().map(json::dict).find(|f| json::s(f, "key") == key).unwrap();
    json::aget(f, "options").iter().map(gstr).collect()
}

#[test]
fn the_schema_carries_the_kit_vocabulary() {
    let keys: Vec<String> = fields(source::schema(), "content", portrait::KIND).iter().map(|f| json::s(json::dict(f), "key")).collect();
    assert_eq!(keys, strs(&["id", "trigger", "event", "min_level", "status", "condition", "stat", "below", "drop_pct", "effect", "intensity", "ms", "preset", "color", "expression", "priority", "surfaces", "rulesets", "origin"]));
    assert_eq!(options("trigger"), strs(&PORTRAIT_TRIGGERS), "the meter trigger is not offered");
    assert_eq!(options("event"), [vec![String::new()], strs(&PORTRAIT_EVENTS)].concat());
    assert_eq!(options("effect"), strs(&PORTRAIT_EFFECTS));
    assert_eq!(options("preset"), [vec![String::new()], strs(&PORTRAIT_JOLTS)].concat());
    assert_eq!(options("surfaces"), strs(&PORTRAIT_SURFACES));
    // CLN-44: the ruleset ids are open: the field offers the ruleset rows
    let rulesets = fields(source::schema(), "content", portrait::KIND).iter().map(json::dict).find(|f| json::s(f, "key") == "rulesets").unwrap();
    assert_eq!((json::s(rulesets, "options_from"), options("rulesets")), ("rulesets".to_string(), Vec::<String>::new()));
    assert_eq!(art::DEFAULT_FACES, ["neutral", "hurt", "angry", "afraid"], "PQ2, PORTRAITFX2 FL1");
    // the typed row's defaults are the schema's
    assert_eq!(PortraitCue::from_obj(&row(&json!({"id": "plain"}))), PortraitCue::from_obj(&dict(json!({"id": "plain"}))));
}

#[test]
fn each_rule_refuses_its_bad_row() {
    for r in good_rows() {
        assert_eq!(errors(&r), Vec::<String>::new(), "{r}");
    }
    let events = PORTRAIT_EVENTS.join(", ");
    let colours = rpgkit_data::PORTRAIT_COLOR_EFFECTS.join(", ");
    let cases: Vec<(Value, String)> = vec![
        // S1, in the order of PORTRAITFX1_TASKS 1.2
        (changed("hit_flash", "event", json!("")), format!("Event: an event row needs one of {events}.")),
        (changed("marked", "status", json!("glitter")), "Status: unknown status 'glitter'.".into()),
        (changed("downed_still", "below", json!(10)), "Below is for hp and derived rows only.".into()),
        (changed("shaken", "below", json!(20)), "A derived row sets exactly one of Below and Drop %.".into()),
        (changed("hit_flash", "ms", json!(0)), "Duration ms must be 60-1000 on an event flash (is 0).".into()),
        (changed("hp_half", "ms", json!(200)), "Duration ms must be 0 on a state row (it holds while its hp matches; is 200).".into()),
        (changed("marked", "color", json!("#ffffff")), format!("Colour is only for {colours} (not fade).")),
        (changed("hit_jolt", "preset", json!("")), "Jolt preset: a jolt needs one of hit, crit.".into()),
        (changed("face_hit", "expression", json!("smirk")), format!("Expression 'smirk' is not one of {}.", DEFAULT_FACES.join(", "))),
        (changed("hit_flash", "surfaces", json!(["menu"])), format!("Surfaces: 'menu' is not one of {}.", PORTRAIT_SURFACES.join(", "))),
        (changed("hit_flash", "trigger", json!("meter")), portrait::METER_MESSAGE.into()),
        // the other rules of 5.1
        (changed("hit_flash", "id", json!("9lives")), "ID '9lives' must be 2-32 characters of a-z, 0-9 and _, a letter first.".into()),
        (changed("hit_flash", "event", json!("sneeze")), format!("Event: 'sneeze' is not one of {events}.")),
        (changed("hit_flash", "status", json!(a_status())), "Status is for a status row only.".into()),
        (changed("marked", "status", json!("")), "Status: a status row needs a status id.".into()),
        (changed("wounded", "condition", json!("Wounded")), "Condition 'Wounded' must be 1-24 characters of a-z and _.".into()),
        (changed("hp_half", "below", json!(0)), "Below must be 1-100 on an hp row (is 0).".into()),
        (changed("hp_half", "drop_pct", json!(10)), "Drop % is for a derived row only.".into()),
        (changed("shaken", "drop_pct", json!(101)), "Drop % must be 1-100 (is 101).".into()),
        (changed("shaken", "stat", json!("grit")), "Derived stat: unknown derived stat 'grit'.".into()),
        (changed("hit_flash", "effect", json!("sparkle")), format!("Effect: 'sparkle' is not one of {}.", PORTRAIT_EFFECTS.join(", "))),
        (changed("hit_flash", "intensity", json!(0.0)), "Intensity must be 0.05-1.0 (is 0.0).".into()),
        (changed("hit_jolt", "ms", json!(30)), "Duration ms must be 0 (the preset's) or 60-600 on an event jolt (is 30).".into()),
        (changed("face_hit", "ms", json!(50)), "Duration ms must be 100-3000 on an event face (is 50).".into()),
        (changed("hit_flash", "preset", json!("hit")), "Jolt preset is for a jolt only.".into()),
        (changed("hit_jolt", "preset", json!("shove")), "Jolt preset: 'shove' is not one of hit, crit.".into()),
        (changed("hit_flash", "color", json!("amber")), "Colour 'amber' must be \"\" or #rrggbb.".into()),
        (changed("hit_flash", "expression", json!("hurt")), "Expression is for a face row only.".into()),
        (changed("face_hit", "expression", json!("")), format!("Expression: a face row needs one of {}.", DEFAULT_FACES.join(", "))),
        (changed("face_hit", "priority", json!(101)), "Priority must be 0-100 (is 101).".into()),
        (changed("hit_flash", "rulesets", json!(["Hard Core"])), "Rulesets: 'Hard Core' must be a ruleset id (lowercase snake_case).".into()),
        (changed("hit_flash", "surfaces", json!(["panel", "panel"])), "Surfaces lists 'panel' twice.".into()),
    ];
    for (bad, want) in cases {
        assert_eq!(errors(&bad), [want], "{bad}");
    }
    assert_eq!(errors(&changed("hit_jolt", "ms", json!(400))), Vec::<String>::new(), "a jolt may set its own duration");
}

/// Planner ruling 2026-10-26: a state row holds (ms 0) and never fires, so the one-shot reactions flash and jolt are
/// refused on every state trigger, one message each; an event row takes both.
#[test]
fn state_rows_refuse_the_one_shot_reactions() {
    let state_rows = [("marked", "status"), ("wounded", "condition"), ("hp_half", "hp"), ("shaken", "derived"), ("downed_still", "down")];
    for (id, trigger) in state_rows {
        for effect in portrait::ONE_SHOT_EFFECTS {
            let bad = changed(id, "effect", json!(effect));
            assert_eq!(errors(&bad), [format!("Effect {effect} is a one-shot reaction: it needs an event row (a {trigger} row holds and never fires it).")], "{bad}");
        }
    }
    assert_eq!(errors(&good_rows()[0]), Vec::<String>::new(), "an event flash");
    assert_eq!(errors(&good_rows()[1]), Vec::<String>::new(), "an event jolt");
}

#[test]
fn an_unknown_condition_warns_and_duplicates_are_errors_on_both() {
    let warned = portrait::warnings(&row(&changed("wounded", "condition", json!("frostbite"))));
    assert_eq!(warned, ["Condition 'frostbite' is not a kit condition (wounded); no member carries it."]);
    assert_eq!(portrait::warnings(&row(&changed("wounded", "id", json!("hurt_bad")))), Vec::<String>::new(), "wounded is a kit condition");
    let rows: Vec<Dict> = [good_rows(), vec![changed("hit_flash", "id", json!("hit_flash_2"))]].concat().iter().map(row).collect();
    let checked = portrait::check_rows(&rows, &portrait::Refs { derived: &[STAT.to_string()], faces: &faces() });
    let dupes: Vec<(String, Vec<String>)> = rows.iter().zip(&checked).filter(|(_, e)| !e.is_empty()).map(|(r, e)| (json::s(r, "id"), e.clone())).collect();
    assert_eq!(
        dupes,
        [
            ("hit_flash".to_string(), vec!["Portrait cue 'hit_flash_2' has the same trigger, match and effect.".to_string()]),
            ("hit_flash_2".to_string(), vec!["Portrait cue 'hit_flash' has the same trigger, match and effect.".to_string()]),
        ]
    );
    let again: Vec<Dict> = vec![row(&good_rows()[0]), row(&changed("hit_flash", "ms", json!(200)))];
    let checked = portrait::check_rows(&again, &portrait::Refs { derived: &[], faces: &faces() });
    assert_eq!(checked[1], ["ID 'hit_flash' is already used by another portrait cue.", "Portrait cue 'hit_flash' has the same trigger, match and effect."]);
}

/// A scratch catalog with the derived stat the `derived` rows name.
fn catalog(tag: &str) -> PathBuf {
    let (scratch_dir, data) = scratch(tag);
    scratch_dir.until_thread_end();
    write_catalog(&data, "derived_stats", json!([{"id": STAT, "name": "Nerve", "formula": "KNOW"}]));
    data
}

fn cue_errors(src: &source::Source) -> Vec<String> {
    src.errors.iter().filter(|e| e.contains("portrait_cues.json")).cloned().collect()
}

#[test]
fn the_loader_keeps_the_passing_rows_beside_the_catalog_kinds() {
    let data = catalog("loader");
    let mut rows = good_rows();
    rows.push(changed("hit_flash", "id", json!("hit_flash_2")));
    rows.push(json!({"id": "gauge", "trigger": "meter", "effect": "tint"}));
    write_catalog(&data, portrait::KIND, Value::Array(rows));
    let src = source::load(&data);
    let at = data.join("catalog").join("portrait_cues.json").display().to_string();
    assert_eq!(
        cue_errors(&src),
        [
            format!("{at}: portrait cue 'hit_flash': Portrait cue 'hit_flash_2' has the same trigger, match and effect."),
            format!("{at}: portrait cue 'hit_flash_2': Portrait cue 'hit_flash' has the same trigger, match and effect."),
            format!("{at}: portrait cue 'gauge': {}", portrait::METER_MESSAGE),
        ]
    );
    let loaded: Vec<String> = json::arr(&src.catalog[portrait::KIND]).iter().map(|e| json::s(json::dict(e), "id")).collect();
    assert_eq!(loaded, ["hit_jolt", "marked", "wounded", "hp_half", "downed_still", "shaken", "face_hit"], "file order; the failing rows are skipped");
    let typed = rpgkit_data::Catalog::from_value(&src.catalog).portrait_cues;
    assert_eq!(typed.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), loaded);
    let shaken = typed.iter().find(|c| c.id == "shaken").unwrap();
    assert_eq!((shaken.trigger.as_str(), shaken.stat.as_str(), shaken.drop_pct, shaken.below, shaken.color.as_str(), shaken.priority), ("derived", STAT, 25, 0, "#06b6d4", 50));
    // the derived stat the row names must load
    fs::remove_file(data.join("catalog").join("derived_stats.json")).unwrap();
    let src = source::load(&data);
    assert!(cue_errors(&src).contains(&format!("{at}: portrait cue 'shaken': Derived stat: unknown derived stat '{STAT}'.")), "{:?}", cue_errors(&src));
    // no file: no rows, no problem
    fs::remove_file(data.join("catalog").join("portrait_cues.json")).unwrap();
    let src = source::load(&data);
    assert_eq!(src.catalog[portrait::KIND], json!([]));
    assert!(cue_errors(&src).is_empty());
    drop_scratch(&data);
}

#[test]
fn the_writer_stores_the_keys_that_differ_from_the_defaults() {
    let data = catalog("writer");
    let file = data.join("catalog").join("portrait_cues.json");
    for r in good_rows() {
        store::upsert(&data, None, portrait::KIND, &dict(r.clone()), None).unwrap_or_else(|e| panic!("{r}: {e:?}"));
    }
    let stored = aegis_content::file::raw_entries(&file);
    let keys = |id: &str| stored.iter().find(|e| json::s(e, "id") == id).unwrap().keys().cloned().collect::<Vec<_>>();
    assert_eq!(keys("hit_flash"), ["id", "event", "intensity", "ms", "color"], "the trigger event and the effect flash are the defaults");
    assert_eq!(keys("downed_still"), ["id", "trigger", "effect"]);
    assert_eq!(keys("face_hit"), ["id", "event", "effect", "ms", "expression", "priority"]);
    for (r, e) in good_rows().iter().zip(&stored) {
        assert_eq!(e.keys().count(), r.as_object().unwrap().len(), "{r}: the stored keys are the given non-default keys");
    }
    let text = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, None, portrait::KIND).unwrap(), text, "a re-save without changes is byte-identical");
    // load, write and load again give the same rows
    let first = source::load(&data).catalog[portrait::KIND].clone();
    for r in json::arr(&first) {
        store::upsert(&data, None, portrait::KIND, json::dict(r), None).unwrap();
    }
    assert_eq!(fs::read_to_string(&file).unwrap(), text, "a loaded row writes back as it was stored");
    assert_eq!(source::load(&data).catalog[portrait::KIND], first);
    // refused writes, with their reasons; the file is put back
    let meter = store::upsert(&data, None, portrait::KIND, &dict(json!({"id": "gauge", "trigger": "meter", "effect": "tint"})), None).err().expect("refused");
    assert_eq!(meter, [portrait::METER_MESSAGE]);
    let bad = store::upsert(&data, None, portrait::KIND, &dict(json!({"id": "hp_low", "trigger": "hp", "below": 25, "effect": "fade", "color": "#ffffff"})), None).err().expect("refused");
    assert!(bad.iter().any(|e| e.ends_with("portrait cue 'hp_low': Colour is only for flash, tint, glow, veins, eye_glow, ash, smoke (not fade).")), "{bad:?}");
    assert_eq!(fs::read_to_string(&file).unwrap(), text, "a refused write puts the file back");
    // the Forge's check of one row sees the loaded rows
    let v = store::view(&data);
    assert_eq!(check::validate_entry(&v, portrait::KIND, &dict(changed("hit_flash", "id", json!("hit_flash_2"))), ""), ["Portrait cue 'hit_flash' has the same trigger, match and effect."]);
    assert_eq!(check::validate_entry(&v, portrait::KIND, &dict(changed("hit_flash", "ms", json!(200))), "hit_flash"), Vec::<String>::new(), "its own row is not a duplicate");
    // a condition warning is reported per row
    store::upsert(&data, None, portrait::KIND, &dict(json!({"id": "frozen", "trigger": "condition", "condition": "frostbite", "effect": "tint"})), None).unwrap();
    let warnings = check::warnings_all(&store::view(&data));
    assert_eq!(warnings.get("portrait_cues:frozen").cloned().unwrap_or_default(), ["Condition 'frostbite' is not a kit condition (wounded); no member carries it."]);
    drop_scratch(&data);
}

#[test]
fn the_face_chain_tries_the_expression_then_neutral_then_the_portrait() {
    let chain = art::portrait_chain("hero", "kaelen", "knight");
    let with = |head: &[&str]| [strs(head), chain.clone()].concat();
    assert_eq!(art::face_chain("hero", "kaelen", "knight", "hurt"), with(&["portraits/heroes/faces/kaelen/hurt.png", "portraits/heroes/faces/kaelen/neutral.png"]));
    assert_eq!(art::face_chain("hero", "kaelen", "knight", ""), with(&["portraits/heroes/faces/kaelen/neutral.png"]));
    assert_eq!(art::face_chain("hero", "kaelen", "knight", "neutral"), with(&["portraits/heroes/faces/kaelen/neutral.png"]));
    assert_eq!(art::face_chain("common", "grunt", "knight/goblin", "angry"), art::portrait_chain("common", "grunt", "knight/goblin"), "a common tier has no frames");
    assert_eq!(art::face_sidecars("portraits/heroes/faces/kaelen/hurt.png"), ["portraits/heroes/faces/kaelen/hurt.face.json", "portraits/heroes/kaelen.face.json"]);
    assert_eq!(art::face_sidecars("portraits/heroes/kaelen.png"), ["portraits/heroes/kaelen.face.json"]);
    assert_eq!(art::face_sidecars("portraits/common/knight_base.png"), ["portraits/common/knight_base.face.json"]);
}

/// The face tool's fixture layer (PORTRAITFX1-T: tools/portraits/fixtures/face/).
fn tool_fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../tools/portraits/fixtures/face")
}

#[test]
fn a11_reports_what_the_face_tool_reports() {
    let root = tool_fixtures();
    let want: Vec<String> = fs::read_to_string(root.join("expected_check.txt")).unwrap().lines().filter(|l| !l.trim().is_empty()).map(String::from).collect();
    assert!(!want.is_empty());
    assert_eq!(assets::portrait_file_warnings(&root, &Layers::of(vec![root.clone()]), &faces()), want, "T2: the kit's A11 and face.py check print the same lines");
}

#[test]
fn a12_warns_once_per_misplaced_frame() {
    let t = aegis_core::scratch::Scratch::vacant("rpgkit_portrait_a12");
    let root = t.join("layer").join("assets");
    let fixtures = tool_fixtures().join("portraits");
    let copy = |from: &str, to: &str| {
        let dest = root.join(to);
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::copy(fixtures.join(from), dest).unwrap();
    };
    // a hero with a portrait and a sidecar with eyes; frames: angry with its own sidecar, hurt without, a smirk
    copy("heroes/good.png", "portraits/heroes/hero_a.png");
    copy("heroes/good.face.json", "portraits/heroes/hero_a.face.json");
    copy("heroes/good.png", "portraits/heroes/faces/hero_a/angry.png");
    copy("heroes/good.face.json", "portraits/heroes/faces/hero_a/angry.face.json");
    copy("heroes/good.png", "portraits/heroes/faces/hero_a/hurt.png");
    copy("heroes/good.png", "portraits/heroes/faces/hero_a/smirk.png");
    // a faces folder for an id without a portrait or a neutral frame; frames under a common folder
    copy("heroes/good.png", "portraits/heroes/faces/ghost/hurt.png");
    copy("common/plain.png", "portraits/common/faces/grunt/neutral.png");
    let layers = Layers::of(vec![root.clone()]);
    let found = assets::portrait_file_warnings(&root, &layers, &faces());
    assert_eq!(
        found,
        [
            "portraits/common/faces/: expression frames are for the unique tiers only (heroes, npcs, elites, bosses).",
            "portraits/heroes/faces/ghost/: no portrait portraits/heroes/ghost.png and no neutral.png.",
            "portraits/heroes/faces/hero_a/hurt.png: no hurt.face.json; it uses the base anchors (portraits/heroes/hero_a.face.json).",
            "portraits/heroes/faces/hero_a/smirk.png: 'smirk' is not one of neutral, hurt, angry, afraid.",
        ]
    );
    // the whole-catalog warnings name the layer
    let v = check::View::new(&json!({}), &json!({}), &json!({}), layers);
    assert_eq!(check::warnings_all(&v).get("assets:layer").cloned().unwrap_or_default(), found);
    // a neutral frame stands in for a missing portrait
    copy("heroes/good.png", "portraits/heroes/faces/ghost/neutral.png");
    assert!(!assets::portrait_file_warnings(&root, &Layers::of(vec![root.clone()]), &faces()).iter().any(|l| l.contains("/ghost/:")));
}

/// A world with one character, a dialogue whose node and a vignette whose line card carry `expression`.
fn talk_world(tag: &str, expression: Value) -> PathBuf {
    let (scratch_dir, data) = scratch(tag);
    scratch_dir.until_thread_end();
    fs::write(aegis_content::world::manifest_path(&data, W), json!({"id": W, "name": "Ward", "extends": [], "connects": [], "version": 1}).to_string()).unwrap();
    let write = |kind: &str, entries: Value| fs::write(store::file_of(&data, Some(W), kind), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
    write("characters", json!([{"id": "keeper_a", "name": "Keeper A"}]));
    write("dialogues", json!([talk(expression.clone())]));
    write("vignettes", json!([reel(expression)]));
    data
}

fn talk(expression: Value) -> Value {
    json!({"id": "talk_a", "name": "Talk", "speaker": "keeper_a", "start": "start", "services": false, "nodes": [{"id": "start", "expression": expression, "text": "Well?", "end": true}]})
}

fn reel(expression: Value) -> Value {
    json!({"id": "reel_a", "name": "Reel", "cards": [{"style": "line", "speaker": "keeper_a", "expression": expression, "text": "A line."}]})
}

fn world_errors(v: &check::View, kind: &str, entry: &Value) -> Vec<String> {
    check::world::validate_entry(v, W, kind, entry.as_object().unwrap(), &json::s(entry.as_object().unwrap(), "id"))
}

#[test]
fn a_node_and_a_line_card_take_an_expression() {
    let data = talk_world("expression", json!(""));
    let v = store::view(&data);
    // the errors an entry has beyond the same entry with an empty expression
    let added = |kind: &str, base: &Value, entry: &Value| -> Vec<String> {
        let before = world_errors(&v, kind, base);
        world_errors(&v, kind, entry).into_iter().filter(|e| !before.contains(e)).collect()
    };
    for x in DEFAULT_FACES {
        assert_eq!(added("dialogues", &talk(json!("")), &talk(json!(x))), Vec::<String>::new(), "node expression '{x}'");
        assert_eq!(added("vignettes", &reel(json!("")), &reel(json!(x))), Vec::<String>::new(), "card expression '{x}'");
    }
    let names = DEFAULT_FACES.join(", ");
    assert_eq!(added("dialogues", &talk(json!("")), &talk(json!("smirk"))), [format!("Node 'start': expression 'smirk' is not one of {names} (or \"\").")]);
    assert_eq!(added("vignettes", &reel(json!("")), &reel(json!("smirk"))), [format!("Card 1: expression 'smirk' is not one of {names} (or \"\").")]);
    let title = |x: &str| json!({"id": "reel_b", "name": "Reel", "cards": [{"style": "title", "title": "T", "expression": x}]});
    assert_eq!(added("vignettes", &title(""), &title("angry")), ["Card 1: expression is for a line card only."]);
    // the writer stores the key only when it is not ""
    for (kind, entry, list) in [("dialogues", talk(json!("")), "nodes"), ("vignettes", reel(json!("")), "cards")] {
        store::upsert(&data, Some(W), kind, &dict(entry), None).unwrap();
        let doc: Value = serde_json::from_str(&fs::read_to_string(store::file_of(&data, Some(W), kind)).unwrap()).unwrap();
        assert!(doc["entries"][0][list][0].get("expression").is_none(), "{kind}: an empty expression is left out");
    }
    for (kind, entry, list) in [("dialogues", talk(json!("angry")), "nodes"), ("vignettes", reel(json!("afraid")), "cards")] {
        store::upsert(&data, Some(W), kind, &dict(entry.clone()), None).unwrap();
        let doc: Value = serde_json::from_str(&fs::read_to_string(store::file_of(&data, Some(W), kind)).unwrap()).unwrap();
        assert_eq!(doc["entries"][0][list][0]["expression"], entry[list][0]["expression"], "{kind}: an expression is stored");
    }
    drop_scratch(&data);
}
