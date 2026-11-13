//! A1.2-K3 (`docs/sessions/A1_VALUE.md` sections 1, 3 and 4, INTENT2 IR11, IR12, IR21 item mapping, INTENT2-K3): the
//! kit's item rules without a battle. The item switches `item_mitigation`, `thrown_miss_spends` and `pouch_stacks` are
//! plain tunables: off unless the row sets `on`, whatever its rules (no hidden ruleset gate), shown in the row form; the
//! empower row caps at 50 (FB-55(1)); the item damage pipeline is the flat damage without mitigation and splash, and
//! applies splash, then guard and cover (capped at `GUARD_CAP`), then armour, floor 1; the item derivation maps heal,
//! buff, area and single damage items (FB-19) and its Field bonus is `reach` for a throw; a pouch stack holds
//! `min(5, max_stack)` copies and `pouch_fits` reads both pouch forms.

use rpgkit_data::{ItemDef, POUCH_STACK_CAP};
use rpgkit_rules::damage::{self, ItemHit, GUARD_CAP};
use rpgkit_rules::gear::pouch_fits;
use rpgkit_rules::intent::{self, FieldBonus, ItemInput};
use rpgkit_rules::ruleset::{Numbers, Ruleset, FIELD_INTENT, ROUND_INTENT};
use rpgkit_rules::stats::roundi;
use rpgkit_rules::status;
use serde_json::json;

fn numbers(row: serde_json::Value) -> Numbers {
    Numbers::from_row(row.as_object().unwrap())
}

#[test]
fn the_item_switches_are_plain_tunables() {
    let classic = numbers(json!({"rules": []}));
    assert!(!classic.item_mitigation && !classic.thrown_miss_spends && !classic.pouch_stacks, "classic keeps the old item rules");
    let bare = numbers(json!({"rules": [ROUND_INTENT, FIELD_INTENT]}));
    assert!(!bare.item_mitigation && !bare.thrown_miss_spends && !bare.pouch_stacks, "off whatever the rules");
    let intent = numbers(json!({"rules": [ROUND_INTENT], "item_mitigation": "on", "thrown_miss_spends": "on", "pouch_stacks": "on"}));
    assert!(intent.item_mitigation && intent.thrown_miss_spends && intent.pouch_stacks, "on as the row sets them");
    let on = numbers(json!({"rules": [], "pouch_stacks": "on"}));
    assert!(on.pouch_stacks && !on.item_mitigation, "classic may set one too");
    for key in ["item_mitigation", "thrown_miss_spends", "pouch_stacks"] {
        assert_eq!(intent.field(key).as_deref(), Some("on"), "{key}");
        assert_eq!(classic.field(key).as_deref(), Some("off"), "{key}");
        assert!(rpgkit_rules::ruleset::field_rule_of(key).is_some(), "{key} has its field rule");
    }
}

#[test]
fn empower_caps_at_fifty() {
    assert_eq!(status::cap_of("empower"), 50);
    assert_eq!(status::stack(0, 0, 2, 50, "empower"), (2, 50), "a 50 % item buff is not clipped");
    assert_eq!(status::stack(0, 0, 3, 30, "empower"), (3, 30), "a 30 % item buff is not clipped");
    assert_eq!(status::stack(0, 0, 2, 80, "empower").1, 50, "above the cap clips to it");
}

#[test]
fn the_item_damage_pipeline() {
    assert_eq!(damage::item_damage(45, ItemHit::default()), 45, "the exact port");
    assert_eq!(damage::item_damage(45, ItemHit { splash: 0.5, mitigation: None, armor_piercing: false }), roundi(45.0 * 0.5));
    let rating = 25;
    let want = roundi(40.0 * 0.5 * (1.0 - 0.35) * (1.0 - damage::armor_reduction(rating))).max(1);
    assert_eq!(damage::item_damage(40, ItemHit { splash: 0.5, mitigation: Some((0.35, rating)), armor_piercing: false }), want);
    let capped = roundi(100.0 * (1.0 - GUARD_CAP)).max(1);
    assert_eq!(damage::item_damage(100, ItemHit { splash: 1.0, mitigation: Some((1.5, 0)), armor_piercing: false }), capped, "guard and cover cap at GUARD_CAP");
    let (v, trace) = damage::item_damage_traced(40, ItemHit { splash: 1.0, mitigation: Some((0.3, rating)), armor_piercing: false });
    assert_eq!(v, damage::item_damage(40, ItemHit { splash: 1.0, mitigation: Some((0.3, rating)), armor_piercing: false }));
    let ids: Vec<&str> = trace.steps.iter().map(|s| s.id.as_ref()).collect();
    assert_eq!(ids, ["item", "round", "splash", "guard_cover", "armour", "round"], "base, splash, guard and cover, armour");
    assert_eq!(damage::item_heal(60), 60);
}

fn input(heal: bool, buff: bool, dmg: bool, area: &str, reach: i32) -> ItemInput {
    ItemInput { heal, buff, damage: dmg, area: area.into(), reach }
}

#[test]
fn the_item_derivation() {
    assert_eq!(intent::derive_item(&input(true, false, false, "", 1)), ["support", "guard"]);
    assert_eq!(intent::derive_item(&input(true, false, false, "", 0)), ["guard"], "a self-only heal is Stand");
    assert_eq!(intent::derive_item(&input(false, true, false, "", 0)), ["guard"]);
    assert_eq!(intent::derive_item(&input(false, false, true, "PLUS", 3)), ["field", "assault"]);
    assert_eq!(intent::derive_item(&input(false, false, true, "BURST", 3)), ["field", "assault"]);
    assert_eq!(intent::derive_item(&input(false, false, true, "BURST", 1)), ["field"]);
    assert_eq!(intent::derive_item(&input(false, false, true, "NONE", 3)), ["assault"]);
    assert_eq!(intent::item_field_bonus(&input(false, false, true, "PLUS", 3)), Some(FieldBonus::Reach));
    assert_eq!(intent::item_field_bonus(&input(false, false, true, "NONE", 3)), None);
    let field = Ruleset::from_row(json!({"id": "t", "rules": [ROUND_INTENT, FIELD_INTENT]}).as_object().unwrap());
    let legacy = Ruleset::from_row(json!({"id": "t", "rules": [ROUND_INTENT]}).as_object().unwrap());
    let bomb = input(false, false, true, "PLUS", 3);
    assert_eq!(intent::item_allowed(&field, &bomb, "maneuver"), ["field", "assault"]);
    assert_eq!(intent::item_allowed(&legacy, &bomb, "maneuver"), ["maneuver"], "the legacy intent without field_intent");
}

#[test]
fn pouch_stacks_and_their_limit() {
    let item = |max_stack: i32| ItemDef::from_obj(json!({"id": "x", "item_type": "CONSUMABLE", "max_stack": max_stack}).as_object().unwrap());
    assert_eq!(item(2).stack_limit(), 2);
    assert_eq!(item(99).stack_limit(), POUCH_STACK_CAP, "capped at 5");
    assert_eq!(ItemDef::from_obj(json!({"id": "y", "item_type": "CONSUMABLE"}).as_object().unwrap()).stack_limit(), 1, "max_stack unset reads 1");
    let ids = ["a", "a", "b"];
    assert!(!pouch_fits(ids.iter().copied(), 3, false, "c", 5), "classic: 3 copies in all");
    assert!(pouch_fits(ids.iter().copied(), 3, true, "c", 5), "stacks: a third distinct item");
    assert!(pouch_fits(ids.iter().copied(), 3, true, "a", 3), "stacks: a's stack has room");
    assert!(!pouch_fits(ids.iter().copied(), 3, true, "a", 2), "stacks: a's stack is full");
    let full = ["a", "b", "c"];
    assert!(!pouch_fits(full.iter().copied(), 3, true, "d", 5), "stacks: no fourth item");
}
