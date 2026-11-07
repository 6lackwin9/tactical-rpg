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
    // FB-107: the run of suppressed tokens takes its separator with it
    assert_eq!(off, on.replace(", ends in Offensivebreaks stance", ""));
    assert!(!off.contains("Offensive") && !off.contains("stance") && !off.contains(", ."), "{off}");
}

/// FB-107 (W2-4): with stances off a stance token that renders empty drops one separator: the preceding `", "` (or
/// `"; "`) when there is one, else the following one; with stances on the text is unchanged.
#[test]
fn stances_off_drops_the_separator_of_an_empty_stance_token() {
    let row = attack();
    let off = |t: &str| source::render(t, "ATTACK", &row, 1, false);
    let on = |t: &str| source::render(t, "ATTACK", &row, 1, true);
    for t in STANCE_TOKENS {
        let cases = [
            // end
            (format!("Hits hard, {{{t}}}."), "Hits hard."),
            (format!("Hits hard; {{{t}}}"), "Hits hard"),
            // middle
            (format!("Hits hard, {{{t}}}, then rests."), "Hits hard, then rests."),
            (format!("Hits hard {{{t}}} and rests."), "Hits hard and rests."),
            (format!("Hits hard {{{t}}}."), "Hits hard."),
            // start
            (format!("{{{t}}}, then rests."), "then rests."),
            (format!("{{{t}}} then rests."), "then rests."),
            (format!("Note: {{{t}}}, then rests."), "Note: then rests."),
        ];
        for (template, want) in cases {
            assert_eq!(off(&template), want, "{template}");
            let shown = on(&template);
            assert!(shown.contains(if t == "flow" { "ends in Offensive" } else { "breaks stance" }), "{template}: {shown}");
        }
    }
    // both tokens in a row: one separator for the run
    assert_eq!(off("Hits hard, {flow}, {disrupt}, then rests."), "Hits hard, then rests.");
    assert_eq!(off("Hits hard, {flow}{disrupt}."), "Hits hard.");
    // the shipped template shape (FB-107): "power, {flow}. Needs" reads "power. Needs"
    assert_eq!(off("{name}: {power} power, {flow}. Needs training."), source::render("{name}: {power} power. Needs training.", "ATTACK", &row, 1, true));
}

/// Stances off leaves a template without stance tokens exactly as stances on renders it (other empty tokens keep
/// their separators, as the writer and the checks expect).
#[test]
fn stances_off_leaves_other_templates_unchanged() {
    let row = attack();
    for t in ["{name}: {power} power, {position}.", "Range {range}{position}.", "{name}, {weapons}, {proficiency}."] {
        assert_eq!(source::render(t, "ATTACK", &row, 1, false), source::render(t, "ATTACK", &row, 1, true), "{t}");
    }
}

/// `skill_description`: stances on shows the stored text; off re-renders the rank's template without the stance
/// tokens when the stored text is that template's render, and keeps a literal text as stored.
#[test]
fn skill_description_follows_the_stances_flag() {
    let mut row = attack();
    let template = "{name}: {power} power, {flow}. Needs training.";
    row.insert("template".into(), json!(template));
    row.insert("tier".into(), json!(1));
    let stored = source::render(template, "ATTACK", &row, 1, true);
    row.insert("description".into(), json!(stored));
    assert_eq!(source::skill_description(&row, true), stored);
    let off = source::skill_description(&row, false);
    assert_eq!(off, source::render(template, "ATTACK", &row, 1, false));
    assert!(!off.contains("Offensive") && !off.contains(", ."), "{off}");
    row.insert("description".into(), json!("A literal rank text, ends in Offensive."));
    assert_eq!(source::skill_description(&row, false), "A literal rank text, ends in Offensive.", "a literal text is shown as stored");
}
