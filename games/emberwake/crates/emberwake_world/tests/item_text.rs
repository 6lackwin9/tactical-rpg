//! ItemText.render on the shipped catalog: every item description renders with its tokens filled.

use emberwake_world::Data;
use std::path::Path;

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("data").0
}

#[test]
fn every_item_description_renders_without_tokens() {
    let d = data();
    let mut templated = 0;
    for id in d.cat.items.keys() {
        let raw = aegis_core::json::s(d.entry("items", id), "description");
        let text = d.item_description(id);
        assert!(!text.contains('{') && !text.contains('}'), "{id}: {text}");
        if raw.contains('{') {
            templated += 1;
            assert_ne!(text, raw, "{id} keeps its tokens");
        }
    }
    assert!(templated > 0, "the catalog has templated descriptions");
}

/// Each token an item's description uses renders as that item's resolved number.
#[test]
fn tokens_take_the_resolved_numbers() {
    use aegis_core::json;
    let d = data();
    let mut checked = 0;
    for (id, def) in &d.cat.items {
        let row = d.entry("items", id);
        let raw = json::s(row, "description");
        let text = d.item_description(id);
        let tier1 = json::aget(row, "tiers").first().and_then(|t| t.as_object()).map(|t| json::dget(t, "when").clone()).unwrap_or_default();
        let tier_value = json::to_int(tier1.get("value").or_else(|| tier1.get("level")).unwrap_or(&serde_json::Value::Null));
        for (token, value) in [("{mitigation}", def.mitigation as i64), ("{burden}", def.burden as i64), ("{req:STR}", def.required_stats.get("STR").copied().unwrap_or(0) as i64), ("{tier1.value}", tier_value)] {
            if raw.contains(token) {
                assert_eq!(emberwake_content::source::render_item(token, row), value.to_string(), "{id}: {token}");
                assert!(text.contains(&value.to_string()), "{id}: '{text}'");
                checked += 1;
            }
        }
    }
    assert!(checked > 0);
}
