//! CLN-40: the content checks' golden, message for message. Each case is an entry of one kind with one or a few fields
//! set wrong (or a hand-made one); the entry's errors and warnings (a world entry's errors and blocking errors) must be
//! exactly the messages recorded in `tests/fixtures/checks_golden.json`.
//!
//! The fixture holds the case inputs in a slim form: `bases` are the untouched entries by `<world>|<kind>|<current id>`,
//! and each case names its base and the fields it `set` (and `drop`s). The inputs were chosen once from the retired
//! Godot recording (`parity/out/checks.json`, 6,721 cases): one case per (kind, changed field) and per message shape
//! (quoted names and numbers left out), 1,012 cases (decision "Godot parity retired" Q-A3).
//!
//! The frozen data the cases are checked against: the catalog is the timeline golden's
//! (`emberwake_battle/tests/fixtures/timeline_kit.json`, shared with the rules and sim goldens), the worlds and game data
//! the sim golden's (`emberwake_sim/tests/fixtures/sim_data.json`); no asset layers (the art checks are
//! `asset_checks.rs`'). Re-freezing either fixture means re-recording this golden too. The messages were recorded once
//! from the engine (CLN-40, 2026-11-01). Re-record only for an intended check change, named in the commit and a
//! docs/HISTORY.md row; never edit the messages by hand (the inputs are kept, only the messages are written again):
//!
//! `RECORD=1 tools/fc.sh test --test checks_golden`
//!
//! `CHECKS_SHOW=<n>` prints n differences per kind (default 2).

use emberwake_content::check::{self, View};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

type Dict = Map<String, Value>;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn golden_path() -> PathBuf {
    crate_dir().join("tests/fixtures/checks_golden.json")
}

fn recording() -> bool {
    std::env::var("RECORD").is_ok_and(|v| v == "1")
}

fn read_json(path: &Path, what: &str) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("missing {what} {}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()))
}

fn frozen_view() -> View {
    let catalog = read_json(&crate_dir().join("../emberwake_battle/tests/fixtures/timeline_kit.json"), "the frozen catalog");
    let wg = read_json(&crate_dir().join("../emberwake_sim/tests/fixtures/sim_data.json"), "the frozen worlds and game data");
    View::new(&catalog, &wg["worlds"], &wg["game"], aegis_core::assets::Layers::default())
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array().map(|a| a.iter().map(|s| s.as_str().unwrap_or("").to_string()).collect()).unwrap_or_default()
}

/// One case's input: its world ("" for a catalog kind), kind, current id and the entry (its base with the case's
/// fields set and dropped, in the base's key order).
struct Case {
    world: String,
    kind: String,
    current: String,
    entry: Dict,
    fields: Vec<String>,
}

/// The loader-computed skill keys (U3.2-S2 `spell_source`) the frozen catalog's rows carry and the slim bases (chosen
/// before the key existed) lack. A family head without one would not fold with its frozen higher ranks (which have
/// it), and the template checks of the head would be skipped; the base takes the frozen row's value (PR4-2).
const LOADER_SKILL_KEYS: [&str; 1] = ["spell_source"];

/// `view` (the frozen data; None where only the inputs matter) aligns a catalog skill base with its frozen row's
/// loader-computed keys before the case's fields are set.
fn case_of(bases: &Dict, row: &Value, view: Option<&View>) -> Case {
    let name = row["base"].as_str().expect("base");
    let mut parts = name.splitn(3, '|');
    let (world, kind, current) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""), parts.next().unwrap_or(""));
    let mut entry = bases.get(name).and_then(Value::as_object).unwrap_or_else(|| panic!("no base {name}")).clone();
    if let Some(view) = view.filter(|_| world.is_empty() && kind == "skills") {
        let frozen = view.get("skills", current);
        for k in LOADER_SKILL_KEYS {
            if let (false, Some(v)) = (entry.contains_key(k), frozen.get(k)) {
                entry.insert(k.into(), v.clone());
            }
        }
    }
    let mut fields = Vec::new();
    for (f, v) in row["set"].as_object().into_iter().flatten() {
        entry.insert(f.clone(), v.clone());
        fields.push(f.clone());
    }
    for f in strings(&row["drop"]) {
        entry.shift_remove(&f);
        fields.push(f);
    }
    Case { world: world.into(), kind: kind.into(), current: current.into(), entry, fields }
}

/// The case's messages now: (errors, warnings) of a catalog entry, (errors, blocking) of a world entry.
fn messages(view: &View, c: &Case) -> (Vec<String>, Vec<String>) {
    if c.world.is_empty() {
        (check::validate_entry(view, &c.kind, &c.entry, &c.current), check::warnings_for(view, &c.kind, &c.entry))
    } else {
        (check::world::validate_entry(view, &c.world, &c.kind, &c.entry, &c.current), check::world::blocking_errors(view, &c.world, &c.kind, &c.entry, &c.current))
    }
}

fn second_key(c: &Case) -> &'static str {
    if c.world.is_empty() {
        "warnings"
    } else {
        "blocking"
    }
}

fn golden() -> (Dict, Vec<Value>) {
    let doc = read_json(&golden_path(), "the checks golden");
    let bases = doc["bases"].as_object().expect("bases").clone();
    let rows = doc["cases"].as_array().expect("cases").clone();
    (bases, rows)
}

/// The golden's layout: one base and one case per line (a check change shows as a one-line diff).
fn write_golden(bases: &Dict, rows: &[Value]) {
    let line = |v: &Value| serde_json::to_string(v).unwrap();
    let b: Vec<String> = bases.iter().map(|(n, e)| format!("{}: {}", line(&json!(n)), line(e))).collect();
    let c: Vec<String> = rows.iter().map(line).collect();
    let text = format!("{{\"bases\": {{\n{}\n}},\n\"cases\": [\n{}\n]}}\n", b.join(",\n"), c.join(",\n"));
    std::fs::write(golden_path(), text).unwrap();
}

#[test]
fn checks_report_their_golden() {
    let view = frozen_view();
    let (bases, rows) = golden();
    if recording() {
        let recorded: Vec<Value> = rows
            .iter()
            .map(|row| {
                let c = case_of(&bases, row, Some(&view));
                let (first, second) = messages(&view, &c);
                let mut out = Map::new();
                for k in ["base", "set", "drop"] {
                    if let Some(v) = row.get(k) {
                        out.insert(k.into(), v.clone());
                    }
                }
                out.insert("errors".into(), json!(first));
                out.insert(second_key(&c).into(), json!(second));
                Value::Object(out)
            })
            .collect();
        write_golden(&bases, &recorded);
        eprintln!("recorded {} check cases to {}", recorded.len(), golden_path().display());
        return;
    }
    let show: usize = std::env::var("CHECKS_SHOW").ok().and_then(|s| s.parse().ok()).unwrap_or(2);
    let mut per_kind: indexmap::IndexMap<String, (usize, usize)> = indexmap::IndexMap::new();
    let mut failures = Vec::new();
    for row in &rows {
        let c = case_of(&bases, row, Some(&view));
        let (first, second) = messages(&view, &c);
        let (want_first, want_second) = (strings(&row["errors"]), strings(&row[second_key(&c)]));
        let label = if c.world.is_empty() { c.kind.clone() } else { format!("{} {}", c.world, c.kind) };
        let slot = per_kind.entry(label.clone()).or_default();
        slot.0 += 1;
        if first != want_first || second != want_second {
            slot.1 += 1;
            if slot.1 <= show {
                failures.push(format!(
                    "{label} {} (fields {:?}):\n  golden errors {want_first:?}\n  now    errors {first:?}\n  golden {} {want_second:?}\n  now    {} {second:?}",
                    c.current,
                    c.fields,
                    second_key(&c),
                    second_key(&c)
                ));
            }
        }
    }
    for (k, (n, bad)) in &per_kind {
        if *bad > 0 {
            eprintln!("{k}: {bad} of {n} cases differ");
        }
    }
    assert!(per_kind.values().all(|(_, bad)| *bad == 0), "the content checks differ from the golden:\n{}", failures.join("\n"));
}

/// The world kinds the golden's cases cover (the old world kinds); the newer ones are checked by the kit's tests.
const WORLD_KINDS_IN_GOLDEN: [&str; 13] = ["regions", "locations", "factions", "characters", "items", "disciplines", "quests", "lore", "encounters", "loot", "dialogues", "shops", "routes"];

/// World kinds whose checks live in the kit's tests (`kits/rpg/crates/rpgkit_content/tests/`), not in this golden.
const WORLD_KINDS_ELSEWHERE: [(&str, &str); 6] = [
    ("meters", "campaign_schema.rs"),
    ("checkpoints", "campaign_schema.rs"),
    ("triggers", "campaign_schema.rs"),
    ("vignettes", "story_checks.rs"),
    ("sequences", "story_checks.rs"),
    ("boards", "story_checks.rs"),
];

/// Every check family is exercised: each catalog kind the checks cover and each old world kind has cases, at least one
/// of them reporting an error and one changing a field; every other world kind is named with the kit test that covers
/// it (a new kind must be added to one side).
#[test]
fn the_golden_covers_every_check_family() {
    if recording() {
        return; // the recording test is rewriting the file; run again without RECORD
    }
    let (bases, rows) = golden();
    let cases: Vec<(Case, &Value)> = rows.iter().map(|r| (case_of(&bases, r, None), r)).collect();
    let covered = |world: bool, kind: &str| {
        let of_kind: Vec<&(Case, &Value)> = cases.iter().filter(|(c, _)| c.world.is_empty() != world && c.kind == kind).collect();
        let errors = of_kind.iter().any(|(_, r)| !strings(&r["errors"]).is_empty());
        let fields = of_kind.iter().any(|(c, _)| !c.fields.is_empty());
        (of_kind.len(), errors, fields)
    };
    for kind in check::PORTED {
        let (n, errors, fields) = covered(false, kind);
        assert!(n > 0 && errors && fields, "catalog kind {kind}: {n} cases, an error {errors}, a changed field {fields}");
    }
    for kind in WORLD_KINDS_IN_GOLDEN {
        let (n, errors, fields) = covered(true, kind);
        assert!(n > 0 && errors && fields, "world kind {kind}: {n} cases, an error {errors}, a changed field {fields}");
    }
    let kit_tests = crate_dir().join("../../../../kits/rpg/crates/rpgkit_content/tests");
    for kind in check::world::KINDS {
        if WORLD_KINDS_IN_GOLDEN.contains(&kind) {
            continue;
        }
        let file = WORLD_KINDS_ELSEWHERE.iter().find(|(k, _)| *k == kind).map(|(_, f)| *f);
        let file = file.unwrap_or_else(|| panic!("world kind {kind} has no golden case and no kit test named"));
        assert!(kit_tests.join(file).is_file(), "world kind {kind}: the kit test {file} is missing");
    }
}
