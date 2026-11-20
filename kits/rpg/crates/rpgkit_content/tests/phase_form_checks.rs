//! BOSSUI1 BU3b (PR7-59): a phase's optional `form` (a form or state the boss bar shows after the boss unit's name in
//! place of its title). The content check takes a phase without it or with a text of 1 to 40 characters and gives one
//! error otherwise (empty, blank, too long, not a text); the battle's phase parse carries it, "" when absent. No files.

use rpgkit_content::check::encounter::phase_problems;
use serde_json::{json, Value};

/// An encounter of one boss with one `hp_below` phase, `form` merged over it (None: no `form` key).
fn encounter(form: Option<Value>) -> aegis_core::json::Dict {
    let mut phase = json!({"id": "unbound", "name": "The chains break", "trigger": {"kind": "hp_below", "unit": "boss", "pct": 50}, "effects": [{"type": "line", "text": "It breaks loose."}]});
    if let Some(f) = form {
        phase["form"] = f;
    }
    json!({"id": "fight", "name": "Fight", "enemies": ["boss"], "max_rounds": 8, "total_blocks": 6, "phases": [phase]}).as_object().unwrap().clone()
}

fn form_errors(form: Option<Value>) -> Vec<String> {
    phase_problems(&encounter(form), &|_| true, &|_| None).into_iter().filter(|e| e.contains("form") || e.contains("unknown key")).collect()
}

#[test]
fn a_phase_takes_an_optional_form_of_one_to_forty_characters() {
    assert!(form_errors(None).is_empty(), "no form: fine");
    assert!(form_errors(Some(json!("Unbound"))).is_empty(), "a form is a known key and a fine text");
    assert!(form_errors(Some(json!("x".repeat(40)))).is_empty(), "40 characters: fine");
    for bad in [json!(""), json!("   "), json!("x".repeat(41)), json!(3), json!(true)] {
        let e = form_errors(Some(bad.clone()));
        assert_eq!(e.len(), 1, "{bad}: one error: {e:?}");
        assert!(e[0].contains("Phase 'unbound': the form"), "{bad}: {e:?}");
    }
}

#[test]
fn the_phase_parse_carries_the_form() {
    use rpgkit_battle::encounter::parse_phases;
    let with = parse_phases(&[encounter(Some(json!("Unbound")))["phases"][0].clone()]);
    assert_eq!(with[0].form, "Unbound");
    assert_eq!(with[0].name, "The chains break", "the name stays the phase's own");
    let without = parse_phases(&[encounter(None)["phases"][0].clone()]);
    assert_eq!(without[0].form, "", "absent: none");
}
