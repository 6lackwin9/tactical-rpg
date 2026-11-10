//! PR4-8 (SKILLCHECK-FOLD, FB-153): a skill family head whose ranks cannot fold (here a rank-2 key rank 1 does not
//! share) and reporting no other error gets one content error naming the fold problem, instead of its description
//! template being skipped silently; a family that folds, or an entry that already reports an error, gets no such
//! error. In-memory flat family rows only.

use rpgkit_content::check::{self, View};
use serde_json::{json, Value};

const NOT_CHECKED: &str = "the description template is not checked because the ranks do not fold";

/// A two-rank ATTACK family `t_cut` / `t_cut_2` with a `{power}` template; `rank2` adds keys to rank 2.
fn family_view(rank2: Value) -> View {
    let row = |id: &str, tier: i64| json!({"id": id, "name": "Cut", "kind": "ATTACK", "family": "t_cut", "tier": tier, "level": 1, "blocks": 2, "template": "Cuts for {power}.", "description": "Cuts."});
    let mut second = row("t_cut_2", 2);
    second.as_object_mut().unwrap().extend(rank2.as_object().unwrap().clone());
    let catalog = json!({"skills": [row("t_cut", 1), second]});
    View::new(&catalog, &json!({}), &json!({}), aegis_core::assets::Layers::default())
}

fn fold_errors(v: &View, id: &str) -> Vec<String> {
    check::validate_entry(v, "skills", v.get("skills", id), id).into_iter().filter(|m| m.contains(NOT_CHECKED)).collect()
}

#[test]
fn a_head_whose_ranks_do_not_fold_gets_one_fold_error() {
    let v = family_view(json!({"name": "Cut II", "stray_key": 1}));
    let errors = fold_errors(&v, "t_cut");
    assert_eq!(errors.len(), 1, "one error on the head: {errors:?}");
    assert!(errors[0].starts_with("Family 't_cut': "), "names the family: {}", errors[0]);
    assert!(errors[0].contains("stray_key"), "names the fold problem: {}", errors[0]);
    assert!(fold_errors(&v, "t_cut_2").is_empty(), "the template is checked on the head only");
}

#[test]
fn a_family_that_folds_gets_no_fold_error() {
    // rank 2 carries the derived name and no schema-default keys: the ranks are filled like the head before the fold
    let v = family_view(json!({"name": "Cut II"}));
    assert!(fold_errors(&v, "t_cut").is_empty(), "{:?}", fold_errors(&v, "t_cut"));
}

#[test]
fn an_entry_with_another_error_gets_no_extra_fold_error() {
    // the fold problem restates the entry's own error (a key ATTACK skills do not use), so only that error shows
    let v = family_view(json!({"name": "Cut II"}));
    let mut head = v.get("skills", "t_cut").clone();
    head.insert("heal_base".into(), json!(5));
    let errors = check::validate_entry(&v, "skills", &head, "t_cut");
    assert!(!errors.is_empty(), "the stray SUPPORT field is an error");
    assert!(errors.iter().all(|m| !m.contains(NOT_CHECKED)), "{errors:?}");
}
