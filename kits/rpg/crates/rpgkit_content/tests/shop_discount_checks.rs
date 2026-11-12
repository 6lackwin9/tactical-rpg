//! ACT1-HOOKS1 (FB-241 c), content side of the shop field `discount` `{"requires_flags", "mult"}`: the schema carries
//! it as a json field stored only when set (`{}` default, `omit_default`), so shops written before it re-save byte for
//! byte; the shop check refuses a block that is not an object, an unknown key, no flag condition, a bad condition, and a
//! `mult` outside 0.1..1.0 or with more than two decimals (a discount, never a markup).

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::check::{shop, View};
use rpgkit_content::{source, store};
use serde_json::{json, Value};

fn view() -> View {
    let worlds = json!({"w": {"manifest": {"id": "w", "name": "W"}, "own": {}}});
    View::new(&json!({}), &worlds, &json!({}), Default::default())
}

fn forge(discount: Value) -> Dict {
    json::dict(&json!({"id": "forge", "name": "Forge", "keeper": "smith", "stock": [], "loot": [], "buy_mult": 1.0, "sell_mult": 0.5, "supply_price": 0,
        "buys": ["WEAPON"], "requires_flags": [], "discount": discount}))
    .clone()
}

fn errors(discount: Value) -> Vec<String> {
    shop::shop_errors(&view(), "w", &forge(discount))
}

fn gate() -> Value {
    json!([{"key": "w.friend", "value": true}])
}

#[test]
fn the_field_is_stored_only_when_set() {
    let f = fields(source::schema(), "world", "shops").iter().map(json::dict).find(|f| json::s(f, "key") == "discount").cloned().expect("the schema field");
    assert_eq!((json::s(&f, "type"), json::get(&f, "default").clone()), ("json".to_string(), json!({})));
    assert!(json::b(&f, "omit_default", false), "stored only when set");
    let mut plain = forge(json!({}));
    plain.shift_remove("discount");
    assert!(!store::world_storage("shops", &plain).contains_key("discount"), "a shop without it re-saves without it");
    assert!(!store::world_storage("shops", &forge(json!({}))).contains_key("discount"), "{{}} is not written");
    let set = forge(json!({"requires_flags": gate(), "mult": 0.8}));
    assert_eq!(store::world_storage("shops", &set).get("discount"), set.get("discount"), "a set discount is written");
}

#[test]
fn a_valid_discount_passes() {
    assert!(errors(json!({})).is_empty());
    for mult in [0.1, 0.75, 1.0] {
        assert!(errors(json!({"requires_flags": gate(), "mult": mult})).is_empty(), "mult {mult}");
    }
}

#[test]
fn a_bad_discount_names_each_problem() {
    assert_eq!(errors(json!([])), ["Discount must be an object {requires_flags, mult}."]);
    assert_eq!(errors(json!({"requires_flags": gate(), "mult": 0.8, "pct": 20})), ["Discount: unknown key 'pct'."]);
    assert_eq!(errors(json!({"mult": 0.8})), ["Discount needs at least one flag condition (requires_flags)."]);
    assert_eq!(errors(json!({"requires_flags": gate()})), ["Discount needs a mult (0.1..1.0)."]);
    assert_eq!(errors(json!({"requires_flags": gate(), "mult": 1.2})), ["Discount mult must be 0.1..1.0."]);
    assert_eq!(errors(json!({"requires_flags": gate(), "mult": 0.05})), ["Discount mult must be 0.1..1.0."]);
    assert_eq!(errors(json!({"requires_flags": gate(), "mult": 0.825})), ["Discount mult takes at most two decimals."]);
    assert_eq!(errors(json!({"requires_flags": gate(), "mult": "half"})), ["Discount mult must be a number."]);
    assert!(!errors(json!({"requires_flags": "w.friend", "mult": 0.8})).is_empty(), "the conditions must be a list");
}
