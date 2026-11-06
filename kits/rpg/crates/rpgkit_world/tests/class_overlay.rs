//! HOOKS1-C on scratch worlds (no shipped data): class names and paths per world (HOOKS1 12, CN1-CN3). World `a`
//! names `warrior` Vanguard with an empty path, `b` extends `a`, `c` has no overlay, `e` extends `a` with a name-only
//! entry (the nearest entry wins whole), `f` replaces the path with one family. A world without the overlay builds
//! from the base catalog itself, exactly as before.

use rpgkit_battle::session::Session;
use rpgkit_world::battle::{self, WorldRoster};
use rpgkit_world::{new_game, progression, Data};
use serde_json::{json, Value};
use std::sync::Arc;

/// The base class path (every family unlocks at level 1).
const PATH: [&str; 2] = ["w_cleave", "w_bash"];
const EXTRA: &str = "w_extra";
const BASE_NAME: &str = "Warrior";
const BASE_DESC: &str = "Fights up front.";

fn with_ids(w: &str, rows: Value) -> Value {
    let rows = rows
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let mut r = r.as_object().unwrap().clone();
            let id = r["id"].as_str().unwrap().to_string();
            r.insert("_world".into(), w.into());
            r.insert("_qid".into(), format!("{w}:{id}").into());
            Value::Object(r)
        })
        .collect();
    Value::Array(rows)
}

/// A world's own founder (party), foe (enemy) and the fight between them.
fn own(w: &str) -> Value {
    json!({
        "characters": with_ids(w, json!([
            {"id": "founder", "name": "Founder", "role": "party", "base_unit": "hero"},
            {"id": "foe", "name": "Foe", "role": "enemy", "base_unit": "brute"}
        ])),
        "encounters": with_ids(w, json!([{"id": "fight", "party": ["founder"], "enemies": ["foe"]}]))
    })
}

fn data() -> Data {
    let atk = |id: &str| json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY"});
    let path: serde_json::Map<String, Value> = PATH.iter().map(|f| (f.to_string(), json!(1))).collect();
    let catalog = json!({
        "classes": [{"id": "warrior", "name": BASE_NAME, "description": BASE_DESC, "skill_path": path,
            "base_stats": {"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10}}],
        // the engine skills a battle looks up, then the path and the extra
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
            atk(PATH[0]), atk(PATH[1]), atk(EXTRA)
        ],
        "units": [
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "warrior", "level": 1, "extra_skills": [EXTRA]},
            {"id": "brute", "name": "Brute", "side": "enemy", "class_id": "warrior", "level": 1, "extra_skills": [EXTRA]}
        ]
    });
    let worlds = json!({
        "a": {"manifest": {"id": "a", "classes": {"warrior": {"name": "Vanguard", "description": "Holds the line.", "skill_path": {}}}}, "own": own("a")},
        "b": {"manifest": {"id": "b", "extends": ["a"]}, "own": {}},
        "c": {"manifest": {"id": "c"}, "own": own("c")},
        "e": {"manifest": {"id": "e", "extends": ["a"], "classes": {"warrior": {"name": "Ward-Captain"}}}, "own": {}},
        "f": {"manifest": {"id": "f", "classes": {"warrior": {"name": "Bash-Hand", "skill_path": {PATH[1]: 1}}}}, "own": own("f")}
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn has(ids: &[String], id: &str) -> bool {
    ids.iter().any(|s| s == id)
}

/// CN1, CN2: the overlay's name and description, inherited through `extends`, the nearest entry first; the class row's
/// without one.
#[test]
fn class_names_follow_the_world_lineage() {
    let d = data();
    assert_eq!(d.class_name("a", "warrior"), "Vanguard");
    assert_eq!(d.class_name("b", "warrior"), "Vanguard", "a child world inherits the overlay");
    assert_eq!(d.class_name("c", "warrior"), BASE_NAME, "no overlay: the class row's name");
    assert_eq!(d.class_name("e", "warrior"), "Ward-Captain", "the nearest manifest with the class key wins");
    assert_eq!(d.class_description("a", "warrior"), "Holds the line.");
    assert_eq!(d.class_description("e", "warrior"), BASE_DESC, "the nearest entry has no description: the class row's");
    assert_eq!(d.class_description("c", "warrior"), BASE_DESC);
}

/// CN3: `skill_path` present replaces the class path; absent (or only in a farther manifest) keeps it. A world that
/// replaces no path builds from the base catalog itself.
#[test]
fn the_overlay_path_replaces_the_class_path() {
    let d = data();
    let path_of = |w: &str| d.cat_in(w).classes["warrior"].skill_path.keys().cloned().collect::<Vec<String>>();
    assert!(path_of("a").is_empty() && path_of("b").is_empty(), "skill_path {{}}: no path, inherited by b");
    assert_eq!(path_of("f"), vec![PATH[1].to_string()]);
    for w in ["c", "e"] {
        assert!(Arc::ptr_eq(d.cat_in(w), &d.cat), "{w}: no overlay path, the base catalog");
    }
    assert_eq!(path_of("c"), PATH.map(String::from).to_vec());
}

/// CN3 for members (decision "HOOKS1-UI slice 1 answers", user 2026-10-30, superseding HOOKS1-C's "the character's
/// world"): a member builds with the class path of the world being played, whichever world its character comes from.
/// In `a` it owns only its own skills (extras, basics); in `c` the class path; in `f` the overlay's one family. The
/// Skills tab's owned families agree with the build. A world-free `build` takes the base catalog's path.
#[test]
fn members_build_with_the_played_world_path() {
    let d = data();
    for from in ["a", "c", "f"] {
        let m = new_game::hero_from_character(&d, from, "founder").expect("a party character");
        let built = |played: &str| (progression::build_in(&d, &m, played, None).expect("valid").unit.known_skills, m.owned_families(&d, played));
        let (a, owned_a) = built("a");
        assert!(PATH.iter().all(|p| !has(&a, p)) && has(&a, EXTRA) && has(&a, "strike"), "{from} in a: own skills only: {a:?}");
        assert!(PATH.iter().all(|p| !has(&owned_a, p)), "{from} in a: {owned_a:?}");
        let (c, owned_c) = built("c");
        assert!(PATH.iter().all(|p| has(&c, p)) && has(&c, EXTRA), "{from} in c: the class path: {c:?}");
        assert!(PATH.iter().all(|p| has(&owned_c, p)));
        let (f, owned_f) = built("f");
        assert!(has(&f, PATH[1]) && !has(&f, PATH[0]), "{from} in f: the overlay's family only: {f:?}");
        assert!(has(&owned_f, PATH[1]) && !has(&owned_f, PATH[0]));
        let base = progression::build(&d, &m).expect("valid").unit.known_skills;
        assert!(PATH.iter().all(|p| has(&base, p)), "{from}, world-free: the base path: {base:?}");
    }
}

/// A save is played in more than one world: a loadout naming a family the class path holds in some world validates;
/// built in a world whose path lacks it, the stored loadout is ignored with a warning (as any unowned entry), and the
/// out-of-battle max HP follows the played world's build.
#[test]
fn a_loadout_owned_in_another_world_validates_and_drops_where_unowned() {
    let d = data();
    let mut m = new_game::hero_from_character(&d, "a", "founder").expect("a party character");
    m.loadout_class = vec![PATH[0].to_string()];
    assert!(m.validate(&d).is_empty(), "{:?}", m.validate(&d));
    let in_c = progression::build_in(&d, &m, "c", None).expect("valid");
    assert!(in_c.warnings.is_empty() && has(&in_c.unit.active_families(), PATH[0]), "c: {:?}", in_c.warnings);
    let in_a = progression::build_in(&d, &m, "a", None).expect("valid");
    assert!(!in_a.warnings.is_empty() && !has(&in_a.unit.active_families(), PATH[0]), "a: {:?}", in_a.unit.active_families());
    for w in ["a", "c"] {
        assert_eq!(progression::max_hp_of(&d, &m, w), progression::build_played(&d, &m, w).unwrap().unit.max_hp as i64);
    }
}

/// CN3 for world characters in battle (the world roster's `overrides`: its `path_cat`): the foe in `a` fights without
/// the class path, with the same name and AP as in `c`, where it is built exactly as before.
#[test]
fn world_characters_fight_with_their_world_path() {
    let d = data();
    let foe = |w: &str| {
        let def = battle::encounter_def(&d, w, d.resolve(w, "encounters", "fight"));
        let s = Session::start(d.cat.clone(), def, &WorldRoster { d: &d, world: w.to_string() });
        let id = s.units_by_ref["foe"][0];
        let founder = s.units_by_ref["founder"][0];
        (s.w.units[id].clone(), s.w.units[founder].known_skills.clone())
    };
    let (in_a, founder_a) = foe("a");
    let (in_c, founder_c) = foe("c");
    assert!(PATH.iter().all(|p| !has(&in_a.known_skills, p)) && has(&in_a.known_skills, EXTRA), "a: {:?}", in_a.known_skills);
    assert!(PATH.iter().all(|p| !has(&founder_a, p)), "a: the party character too: {founder_a:?}");
    assert!(PATH.iter().all(|p| has(&in_c.known_skills, p)) && PATH.iter().all(|p| has(&founder_c, p)));
    assert_eq!((in_a.name.as_str(), in_a.max_ap, in_a.ap), (in_c.name.as_str(), in_c.max_ap, in_c.ap), "name and AP kept");
    let (in_f, _) = foe("f");
    assert!(has(&in_f.known_skills, PATH[1]) && !has(&in_f.known_skills, PATH[0]));
}
