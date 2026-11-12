//! ACT1-HOOKS1 (FB-241 c), content side of supplies as a reward: a quest template's `rewards` takes `supplies` (a whole
//! number 0..the purse's top, `PurseCaps::SUPPLIES_MAX`) and the content check now reads every quest template's
//! rewards (`quest_templates:<id>`); a dialogue `give` takes `supplies` (0..the same top) and needs items, gold or
//! supplies; a `take` still refuses the key.

use rpgkit_content::check::dialogue::{self, EffectList};
use rpgkit_content::check::{self, quest, View};
use rpgkit_content::source::PurseCaps;
use serde_json::{json, Value};

const TOP: i64 = PurseCaps::SUPPLIES_MAX;

fn view_with(templates: Value) -> View {
    let worlds = json!({"w": {"manifest": {"id": "w", "name": "W"}, "own": {}}});
    View::new(&json!({}), &worlds, &json!({"quests": templates}), Default::default())
}

fn choice(fx: Value) -> Vec<String> {
    dialogue::list_errors(&view_with(json!([])), "w", &json!([fx]), "Choice", EffectList::Choice)
}

#[test]
fn quest_rewards_take_supplies_in_range() {
    let v = view_with(json!([]));
    assert!(quest::rewards_errors(&v, &json!({"supplies": 3, "gold": 5})).is_empty());
    assert!(quest::rewards_errors(&v, &json!({"supplies": TOP})).is_empty(), "the purse's top");
    let want = format!("Rewards supplies must be a whole number 0..{TOP}.");
    assert_eq!(quest::rewards_errors(&v, &json!({"supplies": TOP + 1})), [want.clone()]);
    assert_eq!(quest::rewards_errors(&v, &json!({"supplies": -1})), [want.clone()]);
    assert_eq!(quest::rewards_errors(&v, &json!({"supplies": "lots"})), [want]);
}

#[test]
fn the_check_reads_every_quest_templates_rewards() {
    let ok = view_with(json!([{"id": "haul", "objectives": [{"kind": "win_encounter"}], "rewards": {"supplies": 2}}]));
    assert!(!check::validate_all(&ok).contains_key("quest_templates:haul"), "a valid template");
    let bad = view_with(json!([{"id": "haul", "objectives": [{"kind": "win_encounter"}], "rewards": {"supplies": TOP + 1, "luck": 1}}]));
    let found = check::validate_all(&bad).get("quest_templates:haul").cloned().unwrap_or_default();
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].starts_with("Unknown reward key 'luck'") && found[0].contains("supplies"), "{found:?}");
}

#[test]
fn a_give_hands_over_supplies_and_a_take_refuses_them() {
    assert!(choice(json!({"effect": "give", "supplies": 2})).is_empty(), "supplies alone");
    assert!(choice(json!({"effect": "give", "gold": 3, "supplies": 1})).is_empty());
    let el = "Choice effect 1 (give)";
    assert_eq!(choice(json!({"effect": "give", "supplies": TOP + 1})), [format!("{el}: supplies must be a whole number 0..{TOP}.")]);
    assert_eq!(choice(json!({"effect": "give"})), [format!("{el} needs items, gold or supplies.")]);
    let take = choice(json!({"effect": "take", "gold": 1, "supplies": 1}));
    assert!(take.iter().any(|e| e.contains("supplies")), "a take names no supplies: {take:?}");
}
