//! PR5-52 (RULESET-MODES1, FB-223, `P-ARCH-02`): the mutually exclusive rule switches of a ruleset row live under its
//! typed `modes` (`rpgkit_rules::ruleset::MODES`, `Modes`); the old flat keys load as aliases and the writer stores them
//! under `modes`; a mode set twice with two values, a list, an unknown mode or value is a load error; and `classic` is
//! the kit's built-in preset, which resolves without a row (a game ships one ruleset row). Scratch data folders and
//! fixture rows only.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::{check, source, store};
use rpgkit_rules::ruleset::{self, AttackLock, Chase, FlankCost, MeleeDiagonal, Modes, PoiseMode, RestAp, Ruleset, CLASSIC, MODES, MODES_KEY};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_ruleset_modes_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn write_world(data: &Path, id: &str, manifest: Value) {
    let dir = data.join("worlds").join(id);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("world.json"), manifest.to_string()).unwrap();
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

fn base_catalog(data: &Path) {
    let skill = |id: &str, kind: &str| json!({"id": id, "name": id, "kind": kind, "ranks": []});
    write_kind(data, "skills", 2, json!([skill("move", "MOVE"), skill("rest", "REST"), skill("strike", "ATTACK"), skill("brace", "DEFEND")]));
    write_kind(data, "derived_stats", 1, json!([{"id": "max_poise", "name": "Max Poise", "formula": "CONS * 2"}]));
}

/// A timeline-like row that passes every check, with `extra` keys over it.
fn row(extra: Value) -> Value {
    let mut r = json!({
        "id": "timeline", "name": "Timeline",
        "rules": ["round_intent", "deck", "poise", "ap_cost"],
        "intents": ["assault", "guard", "support", "maneuver"],
        "basics": ["move", "rest", "strike", "brace"],
        "poise_stat": "max_poise",
        "deck_slots": {"active": 3, "passive": 2, "keystone": 1}
    });
    for (k, v) in extra.as_object().unwrap() {
        r[k] = v.clone();
    }
    r
}

fn ruleset_errors(src: &source::Source) -> Vec<String> {
    src.errors.iter().filter(|e| e.contains("rulesets.json")).cloned().collect()
}

/// The first row of `entries` as the loader keeps it (None when it is dropped) and the rulesets file's load errors.
fn load(entries: Value) -> (Option<Dict>, Vec<String>) {
    let (_scratch, data) = scratch("load");
    base_catalog(&data);
    write_kind(&data, ruleset::KIND, 1, entries);
    let src = source::load(&data);
    let kept = src.catalog[ruleset::KIND].as_array().and_then(|rows| rows.first()).and_then(Value::as_object).cloned();
    let errors = ruleset_errors(&src);
    drop_scratch(&data);
    (kept, errors)
}

/// The old flat keys of the timeline row the game shipped before PR5-52, and the same values under `modes`.
fn flat_and_modes() -> (Value, Value) {
    let flat = json!({"melee_diagonal": "maneuver", "flank_protected_cost": "full", "rest_ap": "round_end", "chase": "every_skill", "attack_lock": "melee", "poise_mode": "pips"});
    let modes = json!({"modes": {"melee_diagonal": "maneuver", "flank_protected_cost": "full", "rest_ap": "round_end", "chase": "every_skill", "attack_lock": "melee", "poise": "pips"}});
    (flat, modes)
}

/// The schema holds one `modes` field (an object) and none of the old flat keys; the kit's table names each mode once,
/// with its old key and the values of its typed enum.
#[test]
fn the_schema_and_the_table_list_the_modes() {
    let keys: Vec<String> = fields(source::schema(), "content", ruleset::KIND).iter().map(|f| json::s(json::dict(f), "key")).collect();
    assert!(keys.iter().any(|k| k == MODES_KEY), "the schema has modes");
    for m in MODES.iter() {
        assert!(!keys.iter().any(|k| k == m.alias || k == m.key), "{} is not a flat schema field", m.alias);
    }
    let f = fields(source::schema(), "content", ruleset::KIND).iter().map(json::dict).find(|f| json::s(f, "key") == MODES_KEY).unwrap().clone();
    assert_eq!(json::s(&f, "type"), "json");
    let typed: [(&str, &[&str]); 6] = [
        ("melee_diagonal", &MeleeDiagonal::IDS),
        ("flank_protected_cost", &FlankCost::IDS),
        ("rest_ap", &RestAp::IDS),
        ("chase", &Chase::IDS),
        ("attack_lock", &AttackLock::IDS),
        ("poise", &PoiseMode::IDS),
    ];
    assert_eq!(MODES.iter().map(|m| (m.key, m.values)).collect::<Vec<_>>(), typed.to_vec());
    assert_eq!(MODES.iter().find(|m| m.key == "poise").map(|m| m.alias), Some("poise_mode"), "poise_mode loads as modes.poise");
    // every mode's fill (a loaded row's value when it sets none) and default read as the typed default
    for m in MODES.iter().filter(|m| !m.fill.is_empty()) {
        assert_eq!(Modes::default().id(m.key), Some(m.fill), "{}", m.key);
    }
}

/// A row with `modes` and a row with the old flat keys load to one typed `Modes`; each value of each mode reads as its
/// enum; the old keys are folded under `modes` on load (one stored form).
#[test]
fn modes_read_typed_and_the_old_keys_load_as_aliases() {
    let (flat, modes) = flat_and_modes();
    let (by_modes, e1) = load(json!([row(modes)]));
    let (by_flat, e2) = load(json!([row(flat)]));
    assert_eq!((e1, e2), (Vec::<String>::new(), Vec::<String>::new()));
    let (by_modes, by_flat) = (by_modes.expect("the modes row loads"), by_flat.expect("the flat row loads"));
    let want = Modes { poise: PoiseMode::Pips, melee_diagonal: MeleeDiagonal::Maneuver, flank_protected: FlankCost::Full, attack_lock: AttackLock::Melee, rest_ap: RestAp::RoundEnd, chase: Chase::EverySkill };
    assert_eq!(*Ruleset::from_row(&by_modes).modes(), want);
    assert_eq!(Ruleset::from_row(&by_flat), Ruleset::from_row(&by_modes), "the aliases read as the modes");
    assert_eq!(by_flat, by_modes, "the loader folds the old keys under modes");
    for m in MODES.iter() {
        assert!(!by_flat.contains_key(m.alias) && json::dget(&by_flat, MODES_KEY).contains_key(m.key), "{} folded under modes", m.alias);
    }
    // a raw row (no loader) reads its old keys too
    assert_eq!(*Ruleset::from_row(&dict(row(flat_and_modes().0))).modes(), want);
    for m in MODES.iter() {
        for v in m.values {
            let one = Ruleset::from_row(&dict(row(json!({"modes": {m.key: v}}))));
            assert_eq!(one.modes().id(m.key), Some(*v), "modes.{} {v}", m.key);
            assert_eq!(one.numbers.field(&format!("modes.{}", m.key)).as_deref(), Some(*v), "the token form");
        }
    }
    let (plain, _) = load(json!([row(json!({}))]));
    let plain = plain.unwrap();
    assert_eq!(json::dget(&plain, MODES_KEY).clone(), dict(json!({"melee_diagonal": "maneuver", "flank_protected_cost": "full", "attack_lock": "none"})), "a row without modes takes the fills");
    assert_eq!(Ruleset::from_row(&plain).modes().poise, PoiseMode::Bar, "poise unset reads bar");
}

/// PR5-52 review H1: folding an old flat key keeps the order of the keys after it (an order-preserving erase, not a
/// swap), so a flat-form row with row-only keys after its aliases (the `item_mitigation`, `thrown_miss_spends`,
/// `pouch_stacks` tail of the shipped `timeline` row) writes byte-identical to the same row given in `modes` form.
#[test]
fn a_flat_row_writes_as_its_modes_form_with_the_row_only_tail_in_order() {
    let tail = json!({"poise_mode": "pips", "item_mitigation": "on", "thrown_miss_spends": "on", "pouch_stacks": "on"});
    let mut flat = row(json!({"melee_diagonal": "maneuver", "flank_protected_cost": "full", "rest_ap": "round_end", "chase": "every_skill", "attack_lock": "melee"}));
    for (k, v) in tail.as_object().unwrap() {
        flat[k] = v.clone();
    }
    let mut by_modes = row(flat_and_modes().1);
    for k in ["item_mitigation", "thrown_miss_spends", "pouch_stacks"] {
        by_modes[k] = json!("on");
    }
    let written = |entry: &Value| -> String {
        let (_scratch, data) = scratch("tail_order");
        base_catalog(&data);
        store::upsert(&data, None, ruleset::KIND, &dict(entry.clone()), None).unwrap();
        let text = fs::read_to_string(data.join("catalog").join("rulesets.json")).unwrap();
        drop_scratch(&data);
        text
    };
    let (a, b) = (written(&flat), written(&by_modes));
    assert_eq!(a, b, "the flat form writes as the modes form");
    let stored: Value = serde_json::from_str(&a).unwrap();
    let keys: Vec<&str> = stored["entries"][0].as_object().unwrap().keys().map(String::as_str).collect();
    let tail_keys: Vec<&str> = keys.iter().copied().filter(|k| ["item_mitigation", "thrown_miss_spends", "pouch_stacks"].contains(k)).collect();
    assert_eq!(tail_keys, ["item_mitigation", "thrown_miss_spends", "pouch_stacks"], "the row-only tail keeps its order: {keys:?}");
}

/// The writer stores the modes under `modes` in their table order, an old flat key there instead, an empty by-rules
/// mode left out; a re-save without changes is byte-identical.
#[test]
fn the_writer_stores_the_modes_under_modes() {
    let (_scratch, data) = scratch("writer");
    base_catalog(&data);
    let file = data.join("catalog").join("rulesets.json");
    let stored = || -> Value { serde_json::from_str::<Value>(&fs::read_to_string(&file).unwrap()).unwrap()["entries"][0].clone() };
    store::upsert(&data, None, ruleset::KIND, &dict(row(flat_and_modes().0)), None).unwrap();
    let s = stored();
    for m in MODES.iter().filter(|m| m.alias != m.key) {
        assert!(s.get(m.alias).is_none(), "{} is not stored flat: {s}", m.alias);
    }
    for m in MODES.iter() {
        assert!(s.get(m.key).is_none(), "{} is not stored flat: {s}", m.key);
    }
    let order: Vec<&str> = s[MODES_KEY].as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(order, ["melee_diagonal", "flank_protected_cost", "rest_ap", "chase", "attack_lock", "poise"], "the table order");
    let text = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, None, ruleset::KIND).unwrap(), text, "a re-save without changes is byte-identical");
    store::upsert(&data, None, ruleset::KIND, &dict(row(json!({"modes": {"rest_ap": "", "chase": ""}}))), None).unwrap();
    let s = stored();
    assert!(s[MODES_KEY].get("rest_ap").is_none() && s[MODES_KEY].get("chase").is_none(), "empty by-rules modes are left out: {s}");
    drop_scratch(&data);
}

/// The values of a mode are mutually exclusive: a list, an old key that disagrees with `modes`, an unknown value and an
/// unknown mode are each one load error (the row is dropped); an old key that agrees is no error.
#[test]
fn mutually_exclusive_values_are_refused() {
    let cases = [
        (json!({"modes": {"poise": ["bar", "pips"]}}), "modes.poise takes one value, not a list: bar, pips are mutually exclusive."),
        (json!({"modes": {"poise": "pips"}, "poise_mode": "bar"}), "modes.poise \"pips\" and its old key poise_mode \"bar\" disagree: the values are mutually exclusive, set modes.poise only."),
        (json!({"modes": {"attack_lock": "melee"}, "attack_lock": "all"}), "modes.attack_lock \"melee\" and its old key attack_lock \"all\" disagree: the values are mutually exclusive, set modes.attack_lock only."),
        (json!({"modes": {"melee_diagonal": "both"}}), "modes.melee_diagonal: 'both' is not one of none, maneuver, all."),
        (json!({"modes": {"chase": "maybe"}}), "modes.chase: 'maybe' is not one of classic, every_skill (or empty: by the rules)."),
        (json!({"modes": {"stance": "on"}}), "Modes: 'stance' is not one of melee_diagonal, flank_protected_cost, rest_ap, chase, attack_lock, poise."),
    ];
    for (extra, want) in cases {
        let (kept, errors) = load(json!([row(extra.clone())]));
        assert_eq!(errors.len(), 1, "{extra}: {errors:?}");
        assert!(errors[0].ends_with(want), "{extra}: {errors:?}");
        assert!(kept.is_none(), "{extra}: the row is dropped");
    }
    let (kept, errors) = load(json!([row(json!({"modes": {"poise": "pips"}, "poise_mode": "pips"}))]));
    assert_eq!(errors, Vec::<String>::new(), "an agreeing old key");
    assert_eq!(Ruleset::from_row(&kept.unwrap()).modes().poise, PoiseMode::Pips);
    // the raw check agrees with the loader (the Forge's entry check)
    assert_eq!(ruleset::mode_errors(&dict(json!({"modes": "pips"}))), ["Modes must be an object of mode to value."]);
}

/// The classic row the game shipped before PR5-52 (seed `cln44.json` with CLN-45 and ATTACKLOCK1), with its flat keys.
fn shipped_classic_row() -> Value {
    json!({"id": "classic", "name": "Classic", "rules": [], "intents": ["assault", "guard", "support", "maneuver"], "basics": [], "poise_stat": "",
           "stagger_windup_mult": 2.0, "stagger_guard_mult": 0.5, "stagger_repeat_mult": 0.5, "flank_stagger_mult": 1.5, "flank_power_mult": 1.0,
           "break_chance_scale": 0.0, "break_chance_cap": 50, "rest_ap_per_block": 0.5, "rest_ap_cap": 2, "rest_poise_per_block": 5, "deck_slots": {},
           "melee_diagonal": "none", "flank_protected_cost": "full", "flank_protected_ap": 1, "rest_ap": "on_use", "chase": "classic",
           "root_stops_chase": "off", "switch_target_step": "on", "stances": "on", "attack_lock": "none", "move_after_blocks": 3, "item_ap": 1})
}

/// `classic` is the kit's built-in preset: the classic row a game shipped before (fixture) reads as it, raw and loaded;
/// a catalog without the row resolves it with no error; the ids a world may name list it; a world and a derived stat
/// naming it are clean; the content check no longer warns of a missing classic row.
#[test]
fn classic_is_a_built_in_preset() {
    assert_eq!(ruleset::PRESETS, [CLASSIC]);
    assert_eq!(Ruleset::preset(CLASSIC), Some(Ruleset::classic()));
    assert_eq!(Ruleset::preset(ruleset::TIMELINE), None, "timeline is the game's row, not a preset");
    assert_eq!(Ruleset::from_row(&dict(shipped_classic_row())), Ruleset::classic(), "the old row reads as the preset");
    let (loaded, errors) = load(json!([shipped_classic_row()]));
    assert_eq!(errors, Vec::<String>::new());
    assert_eq!(Ruleset::from_row(&loaded.unwrap()), Ruleset::classic(), "the old row, loaded, reads as the preset");
    assert_eq!(check::ruleset::warnings(&dict(shipped_classic_row())), Vec::<String>::new(), "the preset's values raise no metadata warning");

    let (_scratch, data) = scratch("preset");
    base_catalog(&data);
    write_kind(&data, ruleset::KIND, 1, json!([row(json!({}))]));
    write_kind(&data, "derived_stats", 1, json!([{"id": "max_poise", "name": "Max Poise", "formula": "CONS * 2", "rulesets": ["timeline", "classic"]}]));
    write_world(&data, "fen", json!({"id": "fen", "name": "Fen", "ruleset": "classic"}));
    write_world(&data, "moor", json!({"id": "moor", "name": "Moor"}));
    let src = source::load(&data);
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    assert_eq!(Ruleset::ids(&cat), ["timeline"], "the game's one row");
    assert_eq!(Ruleset::known_ids(&cat), ["timeline", "classic"], "the row, then the preset");
    assert_eq!(Ruleset::of(&cat, CLASSIC), Ok(Ruleset::classic()), "classic without its row: the preset");
    assert_eq!(Ruleset::of(&cat, ""), Ok(Ruleset::classic()), "the default id");
    let moor = dict(json!({"id": "moor"}));
    assert_eq!(Ruleset::of(&cat, &ruleset::world_ruleset([&moor])).map(|r| r.id), Ok(CLASSIC.to_string()), "a world naming none plays classic");
    assert!(Ruleset::of(&cat, "chess").is_err(), "another id without its row is refused");
    let v = store::view(&data);
    let warnings = check::warnings_all(&v);
    assert!(!warnings.contains_key("rulesets:missing"), "no missing-classic warning: {warnings:?}");
    let problems = check::validate_all(&v);
    assert!(!problems.contains_key("derived_stats:max_poise"), "a list may name the preset: {problems:?}");
    assert!(problems.values().flatten().all(|p| !p.contains("Ruleset 'classic'")), "a world may name the preset: {problems:?}");
    drop_scratch(&data);
}
