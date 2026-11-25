//! ACT1-HOOKS2 (FB-244 (1)), content side of the effect `install` `{"member_from_flag", "item", "slot"}` and the
//! dialogue condition `in_active_party` `{"character"}`. `install` is known to a choice, a sequence's effects step and a
//! campaign row (it opens no screen); its `member_from_flag` is a flag key, its `item` a graft, its optional `slot` ("" =
//! the first empty slot that fits) a body slot of the graft's role; other keys are unknown. `in_active_party` names a
//! party character of the world.

use rpgkit_content::check::dialogue::{self, EffectList};
use rpgkit_content::check::View;
use serde_json::{json, Value};

const LABEL: &str = "Choice";

fn view() -> View {
    let catalog = json!({
        "items": [
            {"id": "g_beast", "name": "Beast Graft", "item_type": "GRAFT", "tier": 1, "body_role": "monster"},
            {"id": "g_rank", "name": "Rank Graft", "item_type": "GRAFT", "tier": 1, "body_role": "class"},
            {"id": "plain_rock", "name": "Plain Rock", "item_type": "MATERIAL"}
        ],
        "body_slots": [{"id": "s_head", "name": "Head", "role": "class"}, {"id": "s_arm", "name": "Arm", "role": "monster"}]
    });
    let worlds = json!({"w": {"manifest": {"id": "w", "name": "W"}, "own": {"characters": [
        {"id": "hero", "name": "Hero", "role": "party", "_world": "w", "_qid": "w:hero"},
        {"id": "clerk", "name": "Clerk", "role": "npc", "_world": "w", "_qid": "w:clerk"}
    ]}}});
    View::new(&catalog, &worlds, &json!({}), Default::default())
}

fn errors(list: EffectList, fx: Value) -> Vec<String> {
    dialogue::list_errors(&view(), "w", &json!([fx]), LABEL, list)
}

fn install(extra: Value) -> Value {
    let mut fx = json!({"effect": "install", "member_from_flag": "w.chosen", "item": "g_beast"});
    for (k, v) in extra.as_object().unwrap() {
        fx[k] = v.clone();
    }
    fx
}

fn conds(c: Value) -> Vec<String> {
    dialogue::conditions_errors(&view(), "w", &json!([c]), "Requires")
}

#[test]
fn install_passes_in_choices_sequences_and_campaign_rows() {
    for list in [EffectList::Choice, EffectList::Sequence, EffectList::Campaign] {
        assert_eq!(errors(list, install(json!({}))), Vec::<String>::new(), "{list:?}");
        assert!(errors(list, install(json!({"slot": "s_arm"}))).is_empty(), "{list:?}: a slot of its role");
        assert!(errors(list, install(json!({"slot": ""}))).is_empty(), "{list:?}: the first slot that fits");
    }
}

#[test]
fn a_bad_install_names_each_problem() {
    let el = format!("{LABEL} effect 1 (install)");
    let choice = |extra: Value| errors(EffectList::Choice, install(extra));
    assert_eq!(choice(json!({"member_from_flag": ""})), [format!("{el}: member_from_flag '' must be a flag key, non-empty [a-z0-9_.].")]);
    assert_eq!(choice(json!({"item": "g_nope"})), [format!("{el}: unknown item 'g_nope'.")]);
    assert_eq!(choice(json!({"item": "plain_rock"})), [format!("{el}: 'plain_rock' is not a graft.")]);
    assert_eq!(choice(json!({"slot": "s_tail"})), [format!("{el}: unknown body slot 's_tail'.")]);
    assert_eq!(choice(json!({"slot": "s_head"})), [format!("{el}: body slot 's_head' does not take monster grafts.")]);
    assert_eq!(choice(json!({"item": "g_rank", "slot": "s_arm"})), [format!("{el}: body slot 's_arm' does not take class grafts.")]);
    assert_eq!(choice(json!({"slot": 3})), [format!("{el}: slot must be a body slot id or \"\".")]);
    assert_eq!(choice(json!({"member": "hero"})), [format!("{el}: unknown key 'member'.")]);
}

#[test]
fn in_active_party_names_a_party_character() {
    assert!(conds(json!({"kind": "in_active_party", "character": "hero"})).is_empty());
    assert!(conds(json!({"kind": "in_active_party", "character": "w:hero"})).is_empty(), "a qualified ref");
    assert_eq!(conds(json!({"kind": "in_active_party", "character": "clerk"})), ["Requires 1 (in_active_party): character 'clerk' is not a party character (role party)."]);
    let unknown = conds(json!({"kind": "in_active_party", "character": "ghost"}));
    assert!(unknown.len() == 1 && unknown[0].starts_with("Requires 1 (in_active_party) character"), "{unknown:?}");
    assert_eq!(conds(json!({"kind": "in_active_party"})), ["Requires 1 (in_active_party) character: choose a character."]);
    assert_eq!(conds(json!({"kind": "in_active_party", "character": "hero", "value": false})), ["Requires 1 (in_active_party): unknown key 'value'."]);
}
