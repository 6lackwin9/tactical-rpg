//! TERMS1-L's catalog kind `terms` (TERMS1.md 5.1; scratch data): a missing file is an empty table; each error text of
//! 5.1, one per mistake (a `world` term such as `currency` refused with where it is named); the loader keeps the rows
//! that pass and reports the rest; the writer takes a valid row and refuses a bad one; the loaded row reaches
//! `Catalog::term`.

use aegis_core::json::{self, Dict};
use rpgkit_content::check::terms::{self, NAME_MAX, NO_NAME, SHORT_MAX, TIP_MAX, WORD_MAX};
use rpgkit_content::{check, source, store};
use rpgkit_data::terms::{ids, TermScope, CURRENCY, INTENT_GUARD, UPKEEP};
use rpgkit_data::{Catalog, TERMS_KIND};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// A scratch data folder `<tmp>/<tag>/data` with empty catalog and worlds folders.
fn scratch(tag: &str) -> PathBuf {
    let t = std::env::temp_dir().join(format!("rpgkit_terms_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&t);
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    data
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn write_terms(data: &Path, entries: Value) {
    fs::write(data.join("catalog").join(format!("{TERMS_KIND}.json")), json!({"kind": TERMS_KIND, "version": 1, "entries": entries}).to_string()).unwrap();
}

fn dict(v: Value) -> Dict {
    json::dict(&v).clone()
}

fn long(n: usize) -> String {
    "x".repeat(n)
}

#[test]
fn a_missing_file_is_an_empty_table() {
    let data = scratch("missing");
    let src = source::load(&data);
    assert!(json::arr(&src.catalog[TERMS_KIND]).is_empty(), "the kind loads as no rows");
    assert!(src.errors.iter().all(|e| !e.contains(&format!("{TERMS_KIND}.json"))), "{:?}", src.errors);
    assert!(Catalog::from_value(&src.catalog).terms.is_empty());
    drop_scratch(&data);
}

#[test]
fn each_mistake_has_its_text() {
    let data = scratch("texts");
    let v = store::view(&data);
    let check = |e: Value| check::validate_entry(&v, TERMS_KIND, &dict(e), "");
    assert_eq!(check(json!({"id": INTENT_GUARD, "name": "Bulwark"})), Vec::<String>::new(), "a game term with a name");
    let game = ids(TermScope::Game).join(", ");
    assert_eq!(check(json!({"id": "intent_dance", "name": "Dance"})), [format!("Unknown term 'intent_dance'. Terms: {game}.")]);
    assert_eq!(check(json!({"id": CURRENCY, "name": "Scrap"})), ["'currency' is named per world: set the world's purse block."], "Q-T1: the purse names stay per world");
    assert_eq!(check(json!({"id": UPKEEP, "name": "Oil"})), ["'upkeep' is named per world: set the world's purse block."]);
    assert_eq!(check(json!({"id": INTENT_GUARD})), [NO_NAME]);
    assert_eq!(check(json!({"id": INTENT_GUARD, "name": "  "})), [NO_NAME]);
    assert_eq!(check(json!({"id": INTENT_GUARD, "name": long(NAME_MAX + 1)})), [format!("Name is {} characters (max {NAME_MAX}).", NAME_MAX + 1)]);
    assert_eq!(check(json!({"id": INTENT_GUARD, "name": "B", "short": long(SHORT_MAX + 1)})), [format!("Short is {} characters (max {SHORT_MAX}).", SHORT_MAX + 1)]);
    assert_eq!(check(json!({"id": INTENT_GUARD, "name": "B", "word": long(WORD_MAX + 1)})), [format!("Word is {} characters (max {WORD_MAX}).", WORD_MAX + 1)]);
    assert_eq!(check(json!({"id": INTENT_GUARD, "name": "B", "tip": long(TIP_MAX + 1)})), [format!("Tip is {} characters (max {TIP_MAX}).", TIP_MAX + 1)]);
    let at_max = json!({"id": INTENT_GUARD, "name": long(NAME_MAX), "short": long(SHORT_MAX), "word": long(WORD_MAX), "tip": long(TIP_MAX)});
    assert_eq!(check(at_max), Vec::<String>::new(), "every length at its max");
    assert_eq!(terms::id_problem(INTENT_GUARD), "");
    drop_scratch(&data);
}

#[test]
fn the_loader_keeps_the_rows_that_pass() {
    let data = scratch("loader");
    write_terms(&data, json!([{"id": INTENT_GUARD, "name": "Bulwark"}, {"id": CURRENCY, "name": "Scrap"}, {"id": INTENT_GUARD, "name": "Again"}]));
    let src = source::load(&data);
    let kept: Vec<String> = json::arr(&src.catalog[TERMS_KIND]).iter().map(|e| json::s(json::dict(e), "id")).collect();
    assert_eq!(kept, [INTENT_GUARD], "the refused rows are skipped");
    let found: Vec<&String> = src.errors.iter().filter(|e| e.contains("term '")).collect();
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].ends_with("term 'currency': 'currency' is named per world: set the world's purse block."), "{found:?}");
    assert!(found[1].ends_with("term 'intent_guard': ID 'intent_guard' is already used by another term."), "{found:?}");
    assert_eq!(Catalog::from_value(&src.catalog).term(INTENT_GUARD).name, "Bulwark", "the loaded row reaches Catalog::term");
    let v = store::view(&data);
    assert_eq!(check::validate_entry(&v, TERMS_KIND, &dict(json!({"id": INTENT_GUARD, "name": "Ward"})), INTENT_GUARD), Vec::<String>::new(), "an edit of the row itself");
    assert_eq!(check::validate_entry(&v, TERMS_KIND, &dict(json!({"id": INTENT_GUARD, "name": "Ward"})), ""), ["ID 'intent_guard' is already used by another term."], "a new row with a used id");
    drop_scratch(&data);
}

#[test]
fn the_writer_takes_a_row_and_refuses_a_world_term() {
    let data = scratch("writer");
    let file = data.join("catalog").join(format!("{TERMS_KIND}.json"));
    let refused = store::upsert(&data, None, TERMS_KIND, &dict(json!({"id": CURRENCY, "name": "Scrap"})), None).err().expect("a currency row is refused");
    assert!(refused.iter().any(|e| e.ends_with("'currency' is named per world: set the world's purse block.")), "{refused:?}");
    assert!(!file.exists(), "a refused first row leaves no file");
    store::upsert(&data, None, TERMS_KIND, &dict(json!({"id": INTENT_GUARD, "name": "Bulwark", "short": "Wall"})), None).expect("a game term row");
    let text = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, None, TERMS_KIND).unwrap(), text, "a re-save without changes is byte-identical");
    let cat = Catalog::from_value(&source::load(&data).catalog);
    let t = cat.term(INTENT_GUARD);
    assert_eq!((t.name.as_str(), t.short.as_str(), t.word.as_str()), ("Bulwark", "Wall", "Wall"));
    assert!(store::check(&data).iter().all(|p| !p.contains("term '")), "{:?}", store::check(&data));
    store::remove(&data, None, TERMS_KIND, INTENT_GUARD).unwrap();
    assert!(Catalog::from_value(&source::load(&data).catalog).terms.is_empty());
    drop_scratch(&data);
}
