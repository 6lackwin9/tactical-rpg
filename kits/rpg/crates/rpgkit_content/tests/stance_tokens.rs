//! A1.2-S2 (docs/sessions/A1_VALUE.md, FB-55): the skill token renderer takes the ruleset's `stances`; off, the stance
//! tokens `{flow}` and `{disrupt}` render as nothing, every other token as with stances on. Test-only rows.

use aegis_core::json::Dict;
use rpgkit_content::source::{self, STANCE_TOKENS};
use serde_json::json;

fn dict(v: serde_json::Value) -> Dict {
    v.as_object().unwrap().clone()
}

fn attack() -> Dict {
    dict(json!({"id": "t_feint", "name": "Feint", "kind": "ATTACK", "power_mult": 0.7, "ends_in_stance": "OFFENSIVE", "disrupt": "STANCE_BREAK"}))
}

#[test]
fn stance_tokens_are_flow_and_disrupt() {
    assert_eq!(STANCE_TOKENS, ["flow", "disrupt"]);
}

/// Stances on (classic): the tokens render as before.
#[test]
fn stances_on_renders_the_stance_tokens() {
    let row = attack();
    assert_eq!(source::render("{flow}", "ATTACK", &row, 1, true), "ends in Offensive");
    assert_eq!(source::render("{disrupt}", "ATTACK", &row, 1, true), "breaks stance");
}

/// Stances off (the timeline default): the stance tokens are empty, the rest of the text is the same.
#[test]
fn stances_off_renders_the_stance_tokens_empty() {
    let row = attack();
    for t in STANCE_TOKENS {
        assert_eq!(source::render(&format!("{{{t}}}"), "ATTACK", &row, 1, false), "", "{{{t}}}");
    }
    let template = "{name}: {power} power, {flow}{disrupt}.";
    let on = source::render(template, "ATTACK", &row, 1, true);
    let off = source::render(template, "ATTACK", &row, 1, false);
    assert_eq!(off, on.replace("ends in Offensive", "").replace("breaks stance", ""));
    assert!(!off.contains("Offensive") && !off.contains("stance"), "{off}");
}
