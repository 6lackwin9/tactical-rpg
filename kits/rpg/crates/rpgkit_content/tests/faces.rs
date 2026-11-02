//! The face list is the game's data (PORTRAITFX2-F, the game spec PORTRAITFX2 5.2 FL1 and FL4; 5.3
//! `min_level` and `unspent`): `neutral`, then the `portrait_expressions` row ids in file order, the kit's four when
//! there are none; the readers (a dialogue node's and a `line` card's `expression`, a `face` cue's `expression`, the
//! A12 frame names) take a game face (`smug`) once its row exists and refuse it, naming the game's faces, before. The
//! cue rows' `min_level` (skill rows only, 0-9) and the `unspent` trigger. Scratch data and asset layers only.

use aegis_content::schema::{fields, normalize};
use aegis_core::assets::Layers;
use aegis_core::json::{self, Dict};
use rpgkit_content::check::{self, assets, portrait};
use rpgkit_content::{source, store};
use rpgkit_data::{faces_of, Catalog, DEFAULT_FACES, NEUTRAL, PORTRAIT_EXPRESSIONS_KIND as KIND, PORTRAIT_TRIGGERS};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const W: &str = "ward";
/// The game's own face of these tests (not a kit default).
const SMUG: &str = "smug";

fn scratch(tag: &str) -> PathBuf {
    let t = std::env::temp_dir().join(format!("rpgkit_faces_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&t);
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds").join(W)).unwrap();
    data
}

fn drop_scratch(dir: &Path) {
    let _ = fs::remove_dir_all(dir.parent().unwrap());
}

fn write_catalog(data: &Path, kind: &str, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
}

fn strs(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn defaults() -> Vec<String> {
    strs(&DEFAULT_FACES)
}

/// The face rows of a game with `smug` after the kit's hurt (each row draws something: a motion field).
fn face_rows() -> Value {
    json!([{"id": "hurt", "tint": "#e11d48"}, {"id": SMUG, "lean_px": 1}])
}

#[test]
fn fl1_the_faces_are_neutral_then_the_rows_in_file_order() {
    assert_eq!(DEFAULT_FACES, [NEUTRAL, "hurt", "angry", "afraid"]);
    assert_eq!(faces_of([]), defaults(), "no rows: the kit's four");
    assert_eq!(faces_of(["determined", "happy", "hurt"]), strs(&[NEUTRAL, "determined", "happy", "hurt"]), "neutral first, then file order");
    assert_eq!(faces_of([NEUTRAL, SMUG]), strs(&[NEUTRAL, SMUG]), "a neutral row is not listed twice");
    assert_eq!(faces_of([NEUTRAL]), defaults(), "only the plain face's motion row: the kit's four");
    assert_eq!(Catalog::default().faces(), defaults());
    // the loader's catalog and the checks' view read the same list
    let data = scratch("fl1");
    write_catalog(&data, KIND, json!([{"id": SMUG, "lean_px": 1}, {"id": NEUTRAL, "breath_ms": 4000}, {"id": "hurt", "tint": "#e11d48"}]));
    let src = source::load(&data);
    assert!(src.errors.iter().all(|e| !e.contains(KIND)), "{:?}", src.errors);
    let want = strs(&[NEUTRAL, SMUG, "hurt"]);
    assert_eq!(Catalog::from_value(&src.catalog).faces(), want);
    assert_eq!(store::view(&data).faces(), want);
    drop_scratch(&data);
}

/// A world with one character, a dialogue whose node and a vignette whose line card say `smug`.
fn talk_world(tag: &str) -> PathBuf {
    let data = scratch(tag);
    fs::write(aegis_content::world::manifest_path(&data, W), json!({"id": W, "name": "Ward", "extends": [], "connects": [], "version": 1}).to_string()).unwrap();
    let write = |kind: &str, entries: Value| fs::write(store::file_of(&data, Some(W), kind), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
    write("characters", json!([{"id": "keeper_a", "name": "Keeper A"}]));
    data
}

fn talk(expression: &str) -> Value {
    json!({"id": "talk_a", "name": "Talk", "speaker": "keeper_a", "start": "start", "services": false, "nodes": [{"id": "start", "expression": expression, "text": "Well?", "end": true}]})
}

fn reel(expression: &str) -> Value {
    json!({"id": "reel_a", "name": "Reel", "cards": [{"style": "line", "speaker": "keeper_a", "expression": expression, "text": "A line."}]})
}

/// The errors `entry` has beyond the same entry with an empty expression.
fn added(v: &check::View, kind: &str, base: &Value, entry: &Value) -> Vec<String> {
    let errors = |e: &Value| check::world::validate_entry(v, W, kind, e.as_object().unwrap(), &json::s(e.as_object().unwrap(), "id"));
    let before = errors(base);
    errors(entry).into_iter().filter(|e| !before.contains(e)).collect()
}

#[test]
fn fl4_dialogue_takes_a_game_face_once_its_row_exists() {
    let data = talk_world("fl4_talk");
    let v = store::view(&data);
    let names = defaults().join(", ");
    assert_eq!(added(&v, "dialogues", &talk(""), &talk(SMUG)), [format!("Node 'start': expression '{SMUG}' is not one of {names} (or \"\").")], "before the row");
    assert_eq!(added(&v, "vignettes", &reel(""), &reel(SMUG)), [format!("Card 1: expression '{SMUG}' is not one of {names} (or \"\").")]);
    write_catalog(&data, KIND, face_rows());
    let v = store::view(&data);
    for x in v.faces() {
        assert_eq!(added(&v, "dialogues", &talk(""), &talk(&x)), Vec::<String>::new(), "node expression '{x}'");
        assert_eq!(added(&v, "vignettes", &reel(""), &reel(&x)), Vec::<String>::new(), "card expression '{x}'");
    }
    let names = v.faces().join(", ");
    assert_eq!(names, format!("{NEUTRAL}, hurt, {SMUG}"));
    assert_eq!(added(&v, "dialogues", &talk(""), &talk("angry")), [format!("Node 'start': expression 'angry' is not one of {names} (or \"\").")], "a kit default the game has no row for");
    drop_scratch(&data);
}

/// A `face` cue row for `expression`, as the loader normalizes it.
fn face_cue(expression: &str) -> Dict {
    let v = json!({"id": "face_hit", "event": "hit", "effect": "face", "expression": expression, "ms": 900, "priority": 60});
    normalize(fields(source::schema(), "content", portrait::KIND), v.as_object().unwrap(), false)
}

#[test]
fn fl4_a_face_cue_takes_a_game_face_once_its_row_exists() {
    let derived: Vec<String> = Vec::new();
    let refused = portrait::row_errors(&face_cue(SMUG), &portrait::Refs { derived: &derived, faces: &defaults() });
    assert_eq!(refused, [format!("Expression '{SMUG}' is not one of {}.", DEFAULT_FACES.join(", "))]);
    let game = faces_of(["hurt", SMUG]);
    assert!(portrait::row_errors(&face_cue(SMUG), &portrait::Refs { derived: &derived, faces: &game }).is_empty());
    // the loader reads the face rows first: a cue naming `smug` loads only beside its row
    let data = scratch("fl4_cue");
    write_catalog(&data, portrait::KIND, json!([{"id": "face_hit", "event": "hit", "effect": "face", "expression": SMUG, "ms": 900, "priority": 60}]));
    let src = source::load(&data);
    assert!(json::arr(&src.catalog[portrait::KIND]).is_empty(), "refused before the row");
    assert!(src.errors.iter().any(|e| e.ends_with(&format!("portrait cue 'face_hit': Expression '{SMUG}' is not one of {}.", DEFAULT_FACES.join(", ")))), "{:?}", src.errors);
    write_catalog(&data, KIND, face_rows());
    let src = source::load(&data);
    assert_eq!(json::arr(&src.catalog[portrait::KIND]).len(), 1, "{:?}", src.errors);
    // the Forge's check reads the view's faces
    let v = store::view(&data);
    assert!(check::validate_entry(&v, portrait::KIND, &face_cue(SMUG), "face_hit").is_empty());
    drop_scratch(&data);
}

/// The face tool's fixture portraits (PORTRAITFX1-T: tools/portraits/fixtures/face/portraits/).
fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../tools/portraits/fixtures/face/portraits")
}

#[test]
fn fl4_a12_takes_a_game_face_frame_once_its_row_exists() {
    let t = std::env::temp_dir().join(format!("rpgkit_faces_a12_{}", std::process::id()));
    let _ = fs::remove_dir_all(&t);
    let root = t.join("layer").join("assets");
    for (from, to) in [("heroes/good.png", "portraits/heroes/hero_a.png"), ("heroes/good.png", format!("portraits/heroes/faces/hero_a/{SMUG}.png").as_str())] {
        let dest = root.join(to);
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::copy(fixtures().join(from), dest).unwrap();
    }
    let layers = Layers::of(vec![root.clone()]);
    let frame = format!("portraits/heroes/faces/hero_a/{SMUG}.png");
    assert_eq!(assets::portrait_file_warnings(&root, &layers, &defaults()), [format!("{frame}: '{SMUG}' is not one of {}.", DEFAULT_FACES.join(", "))]);
    assert!(assets::portrait_file_warnings(&root, &layers, &faces_of(["hurt", SMUG])).is_empty());
    // the whole-catalog warnings read the view's faces
    let bare = check::View::new(&json!({}), &json!({}), &json!({}), Layers::of(vec![root.clone()]));
    assert_eq!(check::warnings_all(&bare).get("assets:layer").cloned().unwrap_or_default().len(), 1);
    let with_row = check::View::new(&json!({KIND: face_rows()}), &json!({}), &json!({}), Layers::of(vec![root.clone()]));
    assert!(!check::warnings_all(&with_row).contains_key("assets:layer"), "{:?}", check::warnings_all(&with_row));
    let _ = fs::remove_dir_all(&t);
}

/// A cue row as the loader normalizes it.
fn cue(v: Value) -> Dict {
    normalize(fields(source::schema(), "content", portrait::KIND), v.as_object().unwrap(), false)
}

fn cue_errors(v: Value) -> Vec<String> {
    portrait::row_errors(&cue(v), &portrait::Refs { derived: &[], faces: &defaults() })
}

#[test]
fn min_level_is_for_skill_rows_and_unspent_holds() {
    assert_eq!(PORTRAIT_TRIGGERS.last(), Some(&"unspent"));
    let skill = |level: i64| json!({"id": "face_big_skill", "event": "skill", "min_level": level, "effect": "face", "expression": "angry", "ms": 1200, "priority": 55});
    for level in [0, 3, portrait::MIN_LEVEL_MAX] {
        assert_eq!(cue_errors(skill(level)), Vec::<String>::new(), "min_level {level}");
    }
    let over = portrait::MIN_LEVEL_MAX + 1;
    assert_eq!(cue_errors(skill(over)), [format!("Min level must be 0-{} on a skill row (is {over}).", portrait::MIN_LEVEL_MAX)]);
    assert_eq!(cue_errors(skill(-1)), [format!("Min level must be 0-{} on a skill row (is -1).", portrait::MIN_LEVEL_MAX)]);
    assert_eq!(cue_errors(json!({"id": "hit_flash", "event": "hit", "ms": 140, "min_level": 2})), ["Min level is for a skill event row only."]);
    assert_eq!(cue_errors(json!({"id": "level_up", "trigger": "unspent", "effect": "face", "expression": "angry", "min_level": 2})), ["Min level is for a skill event row only."]);
    // the new events and the state trigger pass as rows; `unspent` holds (ms 0) and takes no one-shot effect
    for event in ["battle_start", "ambush", "round_start", "ally_down", "victory"] {
        assert_eq!(cue_errors(json!({"id": "face_event", "event": event, "effect": "face", "expression": "afraid", "ms": 1500})), Vec::<String>::new(), "{event}");
    }
    assert_eq!(cue_errors(json!({"id": "face_level_up", "trigger": "unspent", "effect": "face", "expression": "angry", "surfaces": ["roster"]})), Vec::<String>::new());
    assert_eq!(cue_errors(json!({"id": "level_up", "trigger": "unspent", "effect": "face", "expression": "angry", "ms": 300})), ["Duration ms must be 0 on a state row (it holds while its unspent matches; is 300)."]);
    assert!(!cue_errors(json!({"id": "level_up", "trigger": "unspent", "effect": "flash", "ms": 0})).is_empty(), "a one-shot on a state row");
    // the writer leaves the default out
    let data = scratch("min_level");
    store::upsert(&data, None, portrait::KIND, json::dict(&skill(3)), None).expect("a skill row");
    store::upsert(&data, None, portrait::KIND, json::dict(&json!({"id": "hit_flash", "event": "hit", "ms": 140})), None).expect("a hit row");
    let doc: Value = serde_json::from_str(&fs::read_to_string(data.join("catalog").join(format!("{}.json", portrait::KIND))).unwrap()).unwrap();
    assert_eq!(doc["entries"][0]["min_level"], json!(3));
    assert!(doc["entries"][1].get("min_level").is_none(), "0 is left out");
    drop_scratch(&data);
}
