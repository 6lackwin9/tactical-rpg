//! Derived-stat rows (DERIVE1-S, docs/specs/DERIVE1.md 4.1): the loader keeps the rows that pass in dependency order
//! and reports the rest, one error per broken rule with the field named; cycles name their path; gear and passive stat
//! fields accept a derived id once a row defines it, and only derived ids take a passive `percent`; the writer takes the
//! kind and refuses a row that adds a problem. Scratch data folders only.

use aegis_content::schema::{fields, normalize};
use aegis_core::json::{self, Dict};
use rpgkit_content::check::{self, derived};
use rpgkit_content::{source, store};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// A scratch data folder `<tmp>/<tag>/data` with an empty catalog folder.
fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_derived_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

/// Writes a catalog file as a fixture (the writer is tested below; these files are the test's own input).
fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

/// A row as the loader normalizes it.
fn row(v: Value) -> Dict {
    normalize(fields(source::schema(), "content", derived::KIND), &dict(v), false)
}

/// The load problems of the derived-stat file (a scratch folder also lacks the XP curve, which the loader reports).
fn derived_errors(src: &source::Source) -> Vec<String> {
    src.errors.iter().filter(|e| e.contains("derived_stats.json")).cloned().collect()
}

fn ids(list: &Value) -> Vec<String> {
    json::arr(list).iter().map(|e| json::s(json::dict(e), "id")).collect()
}

#[test]
fn passing_rows_load_typed_in_dependency_order() {
    let (_scratch, data) = scratch("order");
    write_kind(
        &data,
        derived::KIND,
        1,
        json!([
            {"id": "focus", "name": "Focus", "formula": "nerve + 1"},
            {"id": "nerve", "name": "Nerve", "label": "Nrv", "formula": "base + KNOW * per_know + level", "constants": {"base": 3, "per_know": 2}, "rulesets": ["timeline"], "show_on": ["battle"], "suffix": "%"},
            {"id": "poise", "name": "Poise", "formula": "max(CONS, 4)", "min": -5, "max": 50, "status_modifiers": [{"status": "weaken", "flat": -3, "percent": -0.25}]},
        ]),
    );
    let src = source::load(&data);
    assert_eq!(derived_errors(&src), Vec::<String>::new());
    assert_eq!(ids(&src.catalog[derived::KIND]), ["nerve", "focus", "poise"], "a row loads after the rows its formula names; the rest keep file order");
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    assert_eq!(cat.derived_stats.keys().cloned().collect::<Vec<_>>(), ["nerve", "focus", "poise"]);
    let nerve = &cat.derived_stats["nerve"];
    assert_eq!(nerve.formula.source(), "base + KNOW * per_know + level");
    assert_eq!(nerve.formula.names(), ["base", "KNOW", "per_know", "level"]);
    assert_eq!(nerve.constants.get("per_know"), Some(&2.0));
    assert_eq!((nerve.header(), nerve.in_ruleset("timeline"), nerve.in_ruleset("classic"), nerve.shows_on("battle"), nerve.shows_on("party")), ("Nrv", true, false, true, false));
    let focus = &cat.derived_stats["focus"];
    assert_eq!((focus.header(), focus.min, focus.max, focus.in_ruleset("classic"), focus.shows_on("party"), focus.shows_on("dojo")), ("Focus", 0, 100, true, true, true), "schema defaults");
    let poise = &cat.derived_stats["poise"];
    assert_eq!((poise.min, poise.max), (-5, 50));
    assert_eq!(poise.status_modifiers, [rpgkit_data::StatusModifier { status: "weaken".into(), flat: -3, percent: -0.25 }]);
    drop_scratch(&data);
}

#[test]
fn a_missing_file_is_no_rows_and_no_error() {
    let (_scratch, data) = scratch("missing");
    let src = source::load(&data);
    assert_eq!(derived_errors(&src), Vec::<String>::new());
    assert_eq!(src.catalog[derived::KIND], json!([]));
    assert!(rpgkit_data::Catalog::from_value(&src.catalog).derived_stats.is_empty());
    drop_scratch(&data);
}

#[test]
fn rows_naming_each_other_are_a_cycle_on_each_row() {
    let (_scratch, data) = scratch("cycle");
    write_kind(
        &data,
        derived::KIND,
        1,
        json!([
            {"id": "grit", "name": "Grit", "formula": "nerve + 1"},
            {"id": "nerve", "name": "Nerve", "formula": "grit * 2"},
            {"id": "calm", "name": "Calm", "formula": "nerve"},
            {"id": "loop", "name": "Loop", "formula": "loop + 1"},
            {"id": "steady", "name": "Steady", "formula": "5"},
        ]),
    );
    let src = source::load(&data);
    let at = data.join("catalog").join("derived_stats.json").display().to_string();
    assert_eq!(
        derived_errors(&src),
        [
            format!("{at}: derived stat 'grit': Formula: cycle grit -> nerve -> grit."),
            format!("{at}: derived stat 'nerve': Formula: cycle nerve -> grit -> nerve."),
            format!("{at}: derived stat 'calm': Formula: derived stat 'nerve' has errors, so this row does not load either."),
            format!("{at}: derived stat 'loop': Formula: cycle loop -> loop."),
        ]
    );
    assert_eq!(ids(&src.catalog[derived::KIND]), ["steady"], "only the row outside the cycle loads");
    let report = store::check(&data);
    for e in &derived_errors(&src) {
        assert!(report.contains(&format!("load: {e}")), "content check lists {e}: {report:?}");
    }
    drop_scratch(&data);
}

/// One broken rule per case on a row beside a valid one: exactly that error, the field named.
#[test]
fn every_field_rule_has_its_error() {
    let other = json!({"id": "other", "name": "Other", "label": "Oth", "formula": "STR"});
    let long = "a".repeat(25);
    let cases: Vec<(Value, &str)> = vec![
        (json!({"id": ""}), "ID is required."),
        (json!({"id": "Grit"}), "ID 'Grit' must be lowercase snake_case (a-z, 0-9, _)."),
        (json!({"id": "g"}), "ID 'g' must be 2-24 characters long."),
        (json!({"id": long}), "ID 'aaaaaaaaaaaaaaaaaaaaaaaaa' must be 2-24 characters long."),
        (json!({"id": "know"}), "ID 'know' is a base stat."),
        (json!({"id": "level"}), "ID 'level' is the unit level input."),
        (json!({"id": "gear_mitigation"}), "ID 'gear_mitigation' is the worn gear mitigation input."),
        (json!({"id": "clamp"}), "ID 'clamp' is a formula function."),
        (json!({"id": "other"}), "ID 'other' is already used by another derived stat."),
        (json!({"name": " "}), "Name is required."),
        (json!({"name": "N".repeat(25)}), "Name is 25 characters (max 24)."),
        (json!({"label": "L".repeat(17)}), "Label is 17 characters (max 16)."),
        (json!({"label": "Oth"}), "Label 'Oth' is already used by derived stat 'other'."),
        (json!({"formula": "KNOW * nope"}), "Formula: unknown name 'nope' (a base stat STR, DEX, INT, CONS, PERC, KNOW, level, gear_mitigation, a derived stat id or a constant)."),
        (json!({"formula": "know + 1"}), "Formula: base stats are upper case ('KNOW', not 'know')."),
        (json!({"constants": [1]}), "Constants must be an object of name to number."),
        (json!({"constants": {"Str": 1}}), "Constants: 'Str' is a base stat."),
        (json!({"constants": {"level": 1}}), "Constants: 'level' is the unit level input."),
        (json!({"constants": {"gear_mitigation": 1}}), "Constants: 'gear_mitigation' is the worn gear mitigation input."),
        (json!({"constants": {"other": 1}}), "Constants: 'other' is a derived stat id."),
        (json!({"constants": {"2x": 1}}), "Constants: '2x' is not a name (a letter or _ first, then letters, digits or _; not a function)."),
        (json!({"constants": {"round": 1}}), "Constants: 'round' is not a name (a letter or _ first, then letters, digits or _; not a function)."),
        (json!({"constants": {"k": 1000001}}), "Constants: 'k' must be a number up to 1000000 in size."),
        (json!({"constants": {"k": "2"}}), "Constants: 'k' must be a number up to 1000000 in size."),
        (json!({"min": 10, "max": 9}), "Min 10 is above max 9."),
        (json!({"status_modifiers": {}}), "Status modifiers must be a list."),
        (json!({"status_modifiers": [3]}), "Status modifier 1 is not an object."),
        (json!({"status_modifiers": [{"status": "dazed", "flat": 1}]}), "Status modifier 1: unknown status 'dazed'."),
        (json!({"status_modifiers": [{"status": "weaken", "flat": 1}, {"status": "weaken", "flat": 2}]}), "Status modifier 2: status 'weaken' is listed twice."),
        (json!({"status_modifiers": [{"status": "weaken", "flat": 101}]}), "Status modifier 1: flat must be a whole number -100..100."),
        (json!({"status_modifiers": [{"status": "weaken", "flat": 1.5}]}), "Status modifier 1: flat must be a whole number -100..100."),
        (json!({"status_modifiers": [{"status": "weaken", "percent": -1.5}]}), "Status modifier 1: percent must be -1.0..1.0."),
        (json!({"status_modifiers": [{"status": "weaken", "flat": 0, "percent": 0.0}]}), "Status modifier 1: flat and percent are both 0; remove it."),
        (json!({"status_modifiers": [{"status": "weaken", "flat": 1, "rounds": 2}]}), "Status modifier 1: key 'rounds' is not allowed (status, flat, percent)."),
        (json!({"rulesets": ["Hard Core"]}), "Rulesets: 'Hard Core' must be a ruleset id (lowercase snake_case)."),
        (json!({"rulesets": ["classic", "classic"]}), "Rulesets lists 'classic' twice."),
        (json!({"show_on": ["menu"]}), "Show on: 'menu' is not one of party, dojo, battle."),
        (json!({"suffix": "pts"}), "Suffix must be empty or '%' (not 'pts')."),
        (json!({"origin": "mod"}), "Origin: 'mod' is not one of system, user."),
    ];
    for (change, want) in cases {
        let mut bad = dict(json!({"id": "grit", "name": "Grit", "formula": "KNOW * 2 + other"}));
        bad.extend(dict(change.clone()));
        let checked = derived::check_rows(&[row(other.clone()), row(Value::Object(bad))]);
        assert!(checked.errors[0].is_empty(), "{change}: the valid row stays clean: {:?}", checked.errors[0]);
        assert_eq!(checked.errors[1], [want], "{change}");
        assert_eq!(checked.order, [0], "{change}: the broken row does not load");
    }
}

/// RULESET1 P1 (tasks 1.7): `gear_mitigation` is a reserved formula input like `level` (the Max Poise row's shape).
#[test]
fn gear_mitigation_is_a_formula_input() {
    let poise = json!({"id": "max_poise", "name": "Max Poise", "formula": "CONS * per_cons + gear_mitigation * armour_factor", "constants": {"per_cons": 2, "armour_factor": 1.0}, "min": 1, "max": 200});
    let checked = derived::check_rows(&[row(poise)]);
    assert!(checked.errors[0].is_empty(), "{:?}", checked.errors[0]);
    assert_eq!(checked.order, [0]);
}

#[test]
fn a_formula_that_does_not_parse_names_the_column() {
    for src in ["KNOW +", "(KNOW", "pow(KNOW, 2)", "KNOW / 0", ""] {
        let checked = derived::check_rows(&[row(json!({"id": "grit", "name": "Grit", "formula": src}))]);
        let e = aegis_core::formula::Formula::parse(src).unwrap_err();
        assert_eq!(checked.errors[0], [format!("Formula: column {}: {}.", e.column, e.message)], "{src}");
        assert!(checked.order.is_empty(), "{src}: the row does not load");
    }
}

/// A data folder with one derived row `grit` (or none) and a sword family for gear.
fn gear_view(tag: &str, with_row: bool) -> (PathBuf, check::View) {
    let (scratch_dir, data) = scratch(tag);
    scratch_dir.until_thread_end();
    if with_row {
        write_kind(&data, derived::KIND, 1, json!([{"id": "grit", "name": "Grit", "formula": "KNOW"}]));
    }
    let band = |lo: i64, hi: i64| json!({"min": lo, "max": hi});
    write_kind(
        &data,
        "item_families",
        1,
        json!([{
            "id": "sword", "name": "Sword", "item_type": "WEAPON", "hands": ["main_1h"], "default_hand": "main_1h", "scaling_stats": ["STR"],
            "base": {"main_1h": {"power": 14, "range": 1, "block_cost": 1, "burden": 0, "mitigation": 0}},
            "bands": {"main_1h": {"power": band(10, 23), "range": band(1, 2), "block_cost": band(1, 3), "burden": band(0, 4), "mitigation": band(0, 0), "bonus_value": band(0, 4)}},
            "adjustable": ["power"], "modifier_kinds": ["adjust", "stat_bonus", "requirement", "effect"], "max_modifiers": 8, "max_tiers": 3, "damage_type": "slash",
        }]),
    );
    let v = store::view(&data);
    (data, v)
}

fn item_errors(v: &check::View) -> Vec<String> {
    let item = dict(json!({
        "id": "wpn_test", "name": "Test Blade", "family": "sword", "hand": "main_1h",
        "modifiers": [{"kind": "stat_bonus", "stat": "grit", "value": 1}, {"kind": "requirement", "stat": "grit", "value": 5}],
        "tiers": [{"label": "Keen", "when": {"type": "unit_level_at_least", "value": 2}, "bonus": {"stat_bonuses": {"grit": 2}}}],
    }));
    check::validate_entry(v, "items", &item, "")
}

#[test]
fn gear_stat_bonuses_take_a_derived_id_once_a_row_defines_it() {
    let base = "STR, DEX, INT, CONS, PERC, KNOW";
    let (data, v) = gear_view("gear_on", true);
    let errors = item_errors(&v);
    assert!(!errors.iter().any(|e| e.starts_with("Modifier 1: unknown stat")), "a stat bonus takes grit: {errors:?}");
    assert!(errors.contains(&format!("Modifier 2: unknown stat 'grit' (one of {base}).")), "requirements keep the base stats: {errors:?}");
    assert!(!errors.contains(&"Tier 1: unknown stat 'grit'.".to_string()), "a tier bonus takes grit: {errors:?}");
    drop_scratch(&data);
    let (data, v) = gear_view("gear_off", false);
    let errors = item_errors(&v);
    assert!(errors.contains(&format!("Modifier 1: unknown stat 'grit' (one of {base}).")), "{errors:?}");
    assert!(errors.contains(&"Tier 1: unknown stat 'grit'.".to_string()), "{errors:?}");
    drop_scratch(&data);
}

fn passive_errors(v: &check::View, effect: Value) -> Vec<String> {
    check::validate_entry(v, "passives", &dict(json!({"id": "pas_test", "name": "Test", "effects": [effect]})), "")
}

#[test]
fn passive_stat_effects_take_a_derived_id_and_only_it_a_percent() {
    let (data, v) = gear_view("passive_on", true);
    assert_eq!(passive_errors(&v, json!({"type": "stat", "stat": "grit", "value": -5})), Vec::<String>::new());
    assert_eq!(passive_errors(&v, json!({"type": "stat", "stat": "grit", "value": 0, "percent": -0.1})), Vec::<String>::new(), "a percent alone is a change");
    assert_eq!(passive_errors(&v, json!({"type": "stat", "stat": "grit", "value": 2, "percent": 0.5})), Vec::<String>::new());
    assert_eq!(passive_errors(&v, json!({"type": "stat", "stat": "KNOW", "value": 2})), Vec::<String>::new(), "the classic grammar is unchanged");
    assert_eq!(passive_errors(&v, json!({"type": "stat", "stat": "KNOW", "value": 2, "percent": 0.1})), ["Effect 1: percent is for derived stats only (base stats take a flat value)."]);
    assert_eq!(passive_errors(&v, json!({"type": "stat", "stat": "grit", "value": 1, "percent": 1.5})), ["Effect 1: percent must be -1.0..1.0."]);
    assert_eq!(passive_errors(&v, json!({"type": "stat", "stat": "grit", "value": 0, "percent": 0.0})), ["Effect 1: the stat bonus needs a non-zero value."]);
    assert_eq!(passive_errors(&v, json!({"type": "stat", "stat": "nerve", "value": 1})), ["Effect 1: unknown stat 'nerve'."]);
    drop_scratch(&data);
    let (data, v) = gear_view("passive_off", false);
    assert_eq!(passive_errors(&v, json!({"type": "stat", "stat": "grit", "value": 1})), ["Effect 1: unknown stat 'grit'."]);
    drop_scratch(&data);
}

#[test]
fn the_forge_check_of_one_row_sees_the_loaded_rows() {
    let (data, v) = gear_view("forge", true);
    assert_eq!(check::validate_entry(&v, derived::KIND, &dict(json!({"id": "nerve", "name": "Nerve", "formula": "grit + 1"})), ""), Vec::<String>::new());
    assert_eq!(check::validate_entry(&v, derived::KIND, &dict(json!({"id": "grit", "name": "Grit", "formula": "grit"})), "grit"), ["Formula: cycle grit -> grit."]);
    assert_eq!(check::validate_entry(&v, derived::KIND, &dict(json!({"id": "grit", "name": "Grit 2", "formula": "1"})), ""), ["ID 'grit' is already used by another derived stat."]);
    drop_scratch(&data);
}

/// Editing the first of two loaded rows: an id or a label taken by the later row is a duplicate; the row's own id and
/// label are not.
#[test]
fn the_forge_check_of_a_rename_sees_later_rows() {
    let (_scratch, data) = scratch("rename");
    write_kind(&data, derived::KIND, 1, json!([{"id": "grit", "name": "Grit", "label": "Grt", "formula": "KNOW"}, {"id": "nerve", "name": "Nerve", "label": "Nrv", "formula": "CONS"}]));
    let v = store::view(&data);
    assert_eq!(v.derived_ids(), ["grit", "nerve"]);
    let edit = |e: Value| check::validate_entry(&v, derived::KIND, &dict(e), "grit");
    assert_eq!(edit(json!({"id": "grit", "name": "Grit", "label": "Grt", "formula": "KNOW + 1"})), Vec::<String>::new(), "its own id and label");
    assert_eq!(edit(json!({"id": "nerve", "name": "Grit", "label": "Grt", "formula": "KNOW"})), ["ID 'nerve' is already used by another derived stat."]);
    assert_eq!(edit(json!({"id": "grit", "name": "Grit", "label": "Nrv", "formula": "KNOW"})), ["Label 'Nrv' is already used by derived stat 'nerve'."]);
    assert_eq!(edit(json!({"id": "focus", "name": "Focus", "label": "Foc", "formula": "KNOW"})), Vec::<String>::new(), "a rename to a free id");
    drop_scratch(&data);
}

#[test]
fn the_writer_takes_the_kind_and_refuses_a_row_that_adds_a_problem() {
    let (_scratch, data) = scratch("writer");
    let file = data.join("catalog").join("derived_stats.json");
    let up = |entry: Value| store::upsert(&data, None, derived::KIND, &dict(entry), None);
    let bad = up(json!({"id": "nerve", "name": "Nerve", "formula": "grit + 1"})).err().expect("the write is refused");
    assert!(bad.iter().any(|e| e.ends_with("derived stat 'nerve': Formula: unknown name 'grit' (a base stat STR, DEX, INT, CONS, PERC, KNOW, level, gear_mitigation, a derived stat id or a constant).")), "{bad:?}");
    assert!(!file.exists(), "a refused first row leaves no file");
    up(json!({"id": "grit", "name": "Grit", "formula": "KNOW * k", "constants": {"k": 2}})).unwrap();
    up(json!({"id": "nerve", "name": "Nerve", "formula": "grit + 1", "show_on": ["party", "battle"]})).unwrap();
    let text = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, None, derived::KIND).unwrap(), text, "a re-save without changes is byte-identical");
    let stored = aegis_content::file::raw_entries(&file);
    assert_eq!(stored.iter().map(|e| json::s(e, "id")).collect::<Vec<_>>(), ["grit", "nerve"], "the file keeps the write order");
    assert_eq!(stored[0].keys().cloned().collect::<Vec<_>>(), ["id", "name", "label", "description", "formula", "constants", "min", "max", "status_modifiers", "rulesets", "show_on", "order", "suffix"], "the storage form: every field but the system origin");
    let cycle = up(json!({"id": "grit", "name": "Grit", "formula": "nerve"})).err().expect("the write is refused");
    assert!(cycle.iter().any(|e| e.ends_with("derived stat 'grit': Formula: cycle grit -> nerve -> grit.")), "{cycle:?}");
    assert_eq!(fs::read_to_string(&file).unwrap(), text, "a refused write puts the file back");
    let gone = store::remove(&data, None, derived::KIND, "grit").err().expect("the write is refused");
    assert!(gone.iter().any(|e| e.ends_with("derived stat 'nerve': Formula: unknown name 'grit' (a base stat STR, DEX, INT, CONS, PERC, KNOW, level, gear_mitigation, a derived stat id or a constant).")), "{gone:?}");
    store::remove(&data, None, derived::KIND, "nerve").unwrap();
    assert!(store::check(&data).iter().all(|p| !p.contains("derived stat")), "{:?}", store::check(&data));
    drop_scratch(&data);
}

/// CAST1-S2: a stat bonus on a derived stat takes the derived range (1-50); a base stat keeps 1-8.
#[test]
fn a_derived_stat_bonus_has_its_own_range() {
    let (data, v) = gear_view("bonus_range", true);
    let errors_for = |stat: &str, value: i64| {
        let item = dict(json!({"id": "wpn_test", "name": "Test Blade", "family": "sword", "hand": "main_1h", "modifiers": [{"kind": "stat_bonus", "stat": stat, "value": value}]}));
        check::validate_entry(&v, "items", &item, "").into_iter().filter(|e| e.starts_with("Modifier 1:")).collect::<Vec<_>>()
    };
    assert_eq!(errors_for("grit", 30), Vec::<String>::new());
    assert_eq!(errors_for("grit", 12), Vec::<String>::new());
    assert_eq!(errors_for("grit", 50), Vec::<String>::new());
    assert_eq!(errors_for("grit", 0), ["Modifier 1: value 0 is outside 1 to 50."]);
    assert_eq!(errors_for("grit", 51), ["Modifier 1: value 51 is outside 1 to 50."]);
    assert_eq!(errors_for("STR", 8), Vec::<String>::new());
    assert_eq!(errors_for("STR", 9), ["Modifier 1: value 9 is outside 1 to 8."]);
    drop_scratch(&data);
}
