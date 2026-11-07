//! AUDIT-10: the skill family rule "every base-only field identical across the ranks" reads a missing key as the
//! field's schema default, so a row written before a field existed (no `guard_rounds`, no `position`) matches a rank
//! that states the default; a real difference still errors. In-memory flat family rows only.

use rpgkit_content::check::{self, View};
use serde_json::{json, Value};

/// A two-rank family `t_wall` / `t_wall_2` of `kind`, each rank with its own extra keys.
fn family_view(kind: &str, head: Value, rank2: Value) -> View {
    let row = |id: &str, tier: i64, extra: &Value| {
        let mut e = json!({"id": id, "name": "Wall", "kind": kind, "family": "t_wall", "tier": tier, "level": 1, "blocks": 2, "description": "Walls."});
        e.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        e
    };
    let catalog = json!({"skills": [row("t_wall", 1, &head), row("t_wall_2", 2, &rank2)]});
    View::new(&catalog, &json!({}), &json!({}), aegis_core::assets::Layers::default())
}

fn tier_errors(v: &View, id: &str, label: &str) -> Vec<String> {
    let needle = format!("{label} must match across all tiers");
    check::validate_entry(v, "skills", v.get("skills", id), id).into_iter().filter(|m| m.contains(&needle)).collect()
}

#[test]
fn a_missing_key_matches_its_schema_default_across_tiers() {
    // guard_rounds (default 1): absent on one rank, the default on the other
    let v = family_view("DEFEND", json!({"mitigation": 0.2}), json!({"mitigation": 0.2, "guard_rounds": 1}));
    assert!(tier_errors(&v, "t_wall", "guard rounds").is_empty());
    assert!(tier_errors(&v, "t_wall_2", "guard rounds").is_empty());
    // position (default ""): absent on one rank, explicit "" on the other
    let v = family_view("ATTACK", json!({}), json!({"position": ""}));
    assert!(tier_errors(&v, "t_wall", "position").is_empty());
    assert!(tier_errors(&v, "t_wall_2", "position").is_empty());
}

#[test]
fn a_real_difference_still_errors_across_tiers() {
    let v = family_view("DEFEND", json!({"mitigation": 0.2}), json!({"mitigation": 0.2, "guard_rounds": 3}));
    assert_eq!(tier_errors(&v, "t_wall", "guard rounds"), ["Family 't_wall': guard rounds must match across all tiers (t_wall_2 differs)."]);
    assert_eq!(tier_errors(&v, "t_wall_2", "guard rounds"), ["Family 't_wall': guard rounds must match across all tiers (t_wall differs)."]);
    let v = family_view("ATTACK", json!({"position": "flank"}), json!({}));
    assert_eq!(tier_errors(&v, "t_wall_2", "position"), ["Family 't_wall': position must match across all tiers (t_wall differs)."]);
}
