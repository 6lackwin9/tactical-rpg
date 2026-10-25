//! RULESET1 3.1.3: the host's `new` request picks the battle's ruleset (`ruleset`, default classic for a catalog
//! encounter) and switches single extensions (`rules`); a ruleset without its row and an option switched alone are
//! refused; the request replays like any other.

use emberwake_data::Catalog;
use emberwake_host::{Battle, Host};
use emberwake_rules::ruleset::{Ruleset, REACH};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;

/// The Godot export's catalog: it has no ruleset rows.
fn catalog() -> Option<Arc<Catalog>> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../parity/out/catalog.json");
    Catalog::load_resolved(&p).ok().map(Arc::new)
}

fn first_encounter(cat: &Catalog) -> String {
    cat.encounters.keys().next().cloned().expect("an encounter")
}

fn ruleset_of(host: &Host) -> Ruleset {
    host.commands.session.as_ref().expect("a battle").def.ruleset.clone()
}

#[test]
fn new_picks_the_ruleset_and_switches_extensions() {
    let Some(cat) = catalog() else {
        eprintln!("no parity/out/catalog.json: run games/emberwake/parity/export.sh");
        return;
    };
    let enc = first_encounter(&cat);
    let mut host = Host::new(Battle::new(cat.clone()));
    assert_eq!(host.handle(&json!({"cmd": "new", "encounter": enc}))["ok"], json!(true));
    assert_eq!(ruleset_of(&host), Ruleset::classic(), "a catalog encounter is classic by default");
    let r = host.handle(&json!({"cmd": "new", "encounter": enc, "ruleset": "classic", "rules": "+pierce,+reach,-pierce"}));
    assert_eq!(r["ok"], json!(true), "{r}");
    assert_eq!(ruleset_of(&host).rules, [REACH]);
    let history: Vec<Value> = host.handle(&json!({"cmd": "history"}))["history"].as_array().unwrap().clone();
    let mut again = Host::new(Battle::new(cat.clone()));
    for req in &history {
        again.handle(req);
    }
    assert_eq!(ruleset_of(&again).rules, [REACH], "the request replays with its ruleset");
    let no_row = host.handle(&json!({"cmd": "new", "encounter": enc, "ruleset": "timeline"}));
    assert!(no_row["error"].as_str().unwrap_or("").contains("has no row in rulesets"), "{no_row}");
    let option = host.handle(&json!({"cmd": "new", "encounter": enc, "rules": "+round_intent"}));
    assert!(option["error"].as_str().unwrap_or("").contains("is an option"), "{option}");
    assert_eq!(ruleset_of(&host).rules, [REACH], "a refused new leaves the battle as it was");
}
