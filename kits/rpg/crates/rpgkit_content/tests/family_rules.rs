//! U3.2-S2: the gear rules read the family table's engine rules, never a family id. A data-only family table (test
//! families named after no gear the kit knows, in a scratch data folder) gets each existing engine rule's behaviour with
//! zero Rust changes (U3 acceptance test 1, worded per FB-32): `always_ranged` (also at base range 1, FB-31), `focus`
//! (never under a two-handed main weapon), `shield`, `reach` (a melee reach weapon) and `spell_source` (an attack skill
//! that needs one is in the passive group "spell"). The resolved marks stay out of the stored form, and a row the loader
//! did not resolve reads the pre-family rules (`rpgkit_data::LEGACY_FAMILY_RULES`).

use aegis_core::json::Dict;
use aegis_core::scratch::Scratch;
use rpgkit_content::{source, store};
use rpgkit_data::{Catalog, ItemDef, SkillDef, ITEM_FAMILY_RULES, ITEM_REACH_RULE, LEGACY_FAMILY_RULES, SKILL_SPELL_SOURCE};
use rpgkit_rules::gear::{off_hand_ok, Equipment};
use rpgkit_rules::unit::skill_in_group;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// One test family: one hand, base range `range`, the engine rules `rules`.
fn family(id: &str, item_type: &str, hand: &str, range: i64, rules: Value) -> Value {
    json!({"id": id, "name": id, "item_type": item_type, "hands": [hand], "default_hand": hand, "scaling_stats": ["STR"], "damage_type": "blunt",
           "base": {hand: {"power": 8, "range": range, "burden": 0, "mitigation": 0}}, "engine_rules": rules})
}

fn write(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join(format!("catalog/{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

/// The scratch data folder: the test families, one item of each (`i_<family>`) and two attack skills.
fn scratch_data(t: &Path) -> std::path::PathBuf {
    let data = t.join("data");
    let _ = fs::remove_dir_all(&data);
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    let families = json!([
        family("t_launcher", "WEAPON", "main_2h", 1, json!(["always_ranged"])),
        family("t_lens", "WEAPON", "off", 1, json!(["focus"])),
        family("t_guard", "SHIELD", "off", 1, json!(["shield"])),
        family("t_pike", "WEAPON", "main_2h", 1, json!(["reach"])),
        family("t_rod", "WEAPON", "main_1h", 1, json!(["spell_source"])),
        family("t_club", "WEAPON", "main_1h", 1, json!([])),
        family("t_cleaver", "WEAPON", "main_2h", 1, json!([])),
        family("t_parry", "WEAPON", "off", 1, json!([])),
    ]);
    let items: Vec<Value> = families.as_array().unwrap().iter().map(|f| {
        let id = f["id"].as_str().unwrap();
        json!({"id": format!("i_{id}"), "name": id, "family": id})
    }).collect();
    write(&data, "item_families", 1, families);
    write(&data, "items", 2, Value::Array(items));
    write(&data, "skills", 2, json!([
        {"id": "t_bolt", "name": "Bolt", "kind": "ATTACK", "weapon_types": ["t_rod"], "min_proficiency": 1},
        {"id": "t_chop", "name": "Chop", "kind": "ATTACK", "weapon_types": ["t_club"], "min_proficiency": 1},
    ]));
    data
}

fn item<'a>(cat: &'a Catalog, family: &str) -> &'a ItemDef {
    cat.items.get(&format!("i_{family}")).unwrap_or_else(|| panic!("fixture item of {family}"))
}

fn skill<'a>(cat: &'a Catalog, id: &str) -> &'a SkillDef {
    cat.skills.iter().find(|s| s.id == id).unwrap_or_else(|| panic!("fixture skill {id}"))
}

#[test]
fn a_data_only_family_gets_each_engine_rule() {
    let t = Scratch::vacant("rpgkit_family_rules");
    let data = scratch_data(&t);
    let src = source::load(&data);
    let cat = Catalog::from_value(&src.catalog);
    let eq = |f: &str| Equipment::new(item(&cat, f));

    // always_ranged: ranged at base range 1 (FB-31: the rule, not a family id list); a plain family stays melee
    assert_eq!(item(&cat, "t_launcher").base_range, 1, "the fixture's range");
    assert!(eq("t_launcher").is_ranged_weapon() && !eq("t_launcher").is_melee_weapon());
    assert!(eq("t_club").is_melee_weapon() && !eq("t_club").is_ranged_weapon());

    // focus: not held under a two-handed main weapon; a plain off-hand weapon is; under a one-handed main both are
    let (cleaver, club) = (item(&cat, "t_cleaver"), item(&cat, "t_club"));
    assert!(cleaver.two_handed && !club.two_handed, "the fixture's hands");
    assert!(!off_hand_ok(Some(cleaver), item(&cat, "t_lens")), "a focus under a 2H main");
    assert!(off_hand_ok(Some(cleaver), item(&cat, "t_parry")), "a non-focus off-hand weapon under a 2H main");
    assert!(off_hand_ok(Some(club), item(&cat, "t_lens")) && off_hand_ok(None, item(&cat, "t_lens")));

    // shield: the rule makes the shield; another off-hand item is not one
    assert!(eq("t_guard").is_shield());
    assert!(!eq("t_lens").is_shield() && !eq("t_parry").is_shield());

    // reach: a melee weapon of a reach family has reach; the others do not
    assert!(eq("t_pike").has_reach() && item(&cat, "t_pike").reach_rule);
    assert!(!eq("t_club").has_reach() && !eq("t_launcher").has_reach());

    // spell_source: an attack skill needing such a weapon is in the group "spell"
    assert!(skill_in_group(skill(&cat, "t_bolt"), "spell"));
    assert!(!skill_in_group(skill(&cat, "t_chop"), "spell"));

    // an off-hand family's id is its off-hand category (no fixed list in the loader)
    assert_eq!(item(&cat, "t_lens").off_hand_category, "t_lens");
    assert_eq!(item(&cat, "t_club").off_hand_category, "");
}

#[test]
fn the_resolved_marks_stay_out_of_the_stored_form() {
    let t = Scratch::vacant("rpgkit_family_rules_store");
    let data = scratch_data(&t);
    let src = source::load(&data);
    let families: Vec<Dict> = src.game["item_families"].as_array().unwrap().iter().filter_map(|f| f.as_object().cloned()).collect();
    let rows = |kind: &str| -> Vec<Dict> { src.catalog[kind].as_array().unwrap().iter().filter_map(|r| r.as_object().cloned()).collect() };
    let items = rows("items");
    let pike = items.iter().find(|r| r["id"] == "i_t_pike").expect("resolved pike");
    assert_eq!(pike[ITEM_FAMILY_RULES], json!(["reach"]), "the loader writes the family's rules");
    for row in &items {
        let stored = store::catalog_storage("items", row, &families).expect("stored item");
        assert!(!stored.contains_key(ITEM_FAMILY_RULES) && !stored.contains_key(ITEM_REACH_RULE), "{}: {stored:?}", row["id"]);
    }
    let skills = rows("skills");
    let bolt = skills.iter().find(|r| r["id"] == "t_bolt").expect("resolved bolt");
    assert_eq!(bolt[SKILL_SPELL_SOURCE], json!(true));
    for row in &skills {
        let stored = store::catalog_storage("skills", row, &families).expect("stored skill");
        assert!(!stored.contains_key(SKILL_SPELL_SOURCE), "{}: {stored:?}", row["id"]);
    }
}

#[test]
fn a_row_without_resolved_rules_reads_the_legacy_rules() {
    for (gear, rule) in LEGACY_FAMILY_RULES {
        let row = json!({"id": "x", "item_type": "WEAPON", "weapon_type": gear}).as_object().cloned().unwrap();
        assert!(ItemDef::from_obj(&row).has_rule(rule), "{gear}: {rule} on a pre-family row");
        let mut resolved = row.clone();
        resolved.insert(ITEM_FAMILY_RULES.into(), json!([]));
        assert!(!ItemDef::from_obj(&resolved).has_rule(rule), "{gear}: the resolved rules win");
        let skill = json!({"id": "s", "kind": "ATTACK", "weapon_types": [gear]}).as_object().cloned().unwrap();
        assert_eq!(SkillDef::from_obj(&skill).spell_source, rule == rpgkit_data::SPELL_SOURCE_RULE, "{gear}: a pre-family skill row");
    }
    let shield = json!({"id": "x", "item_type": "SHIELD"}).as_object().cloned().unwrap();
    assert!(Equipment::new(&ItemDef::from_obj(&shield)).is_shield(), "a pre-family SHIELD row");
    let reach = json!({"id": "x", "item_type": "WEAPON", ITEM_REACH_RULE: true}).as_object().cloned().unwrap();
    assert!(ItemDef::from_obj(&reach).reach_rule, "the pre-family reach key");
}
