//! LAYER-DOWN-2 (PR5-30): a new weapon family is data only. On a scratch copy of the shipped data, a test-only family
//! (a copy of a plain one-handed shipped weapon family under a new id), an item of it and the first Dojo hero's class
//! trained in it are written through the content writer and the new rows add no problem to the folder check (no kit list of gear types is
//! asked); then the Dojo hero wears the item trained, untouched by the wield penalty, and strikes the dummy with it.
//! The family is picked by rule (WEAPON, one hand `main_1h`, no engine rule, with an item), no shipped id is named.

use aegis_core::json::{self, Dict};
use emberwake_battle::dojo::{DojoSession, DUMMY_MAX_HP};
use emberwake_content::store;
use emberwake_world::Data;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const FAMILY: &str = "t_glaive";
const ITEM: &str = "t_glaive_blade";

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let p = e.unwrap().path();
        let name = p.file_name().unwrap().to_owned();
        if p.is_dir() {
            copy_dir(&p, &to.join(name));
        } else {
            std::fs::copy(&p, to.join(name)).unwrap();
        }
    }
}

/// The stored rows of catalog file `kind` (the writer's form).
fn stored(dir: &Path, kind: &str) -> Vec<Dict> {
    let text = std::fs::read_to_string(dir.join("catalog").join(format!("{kind}.json"))).expect("a catalog file");
    let doc: Value = serde_json::from_str(&text).expect("JSON");
    json::arr(&doc["entries"]).iter().map(|e| json::dict(e).clone()).collect()
}

fn upsert(dir: &Path, kind: &str, entry: &Dict, replace: Option<&str>) {
    if let Err(e) = store::upsert(dir, None, kind, entry, replace) {
        panic!("{kind} '{}': {e:?}", json::s(entry, "id"));
    }
}

#[test]
fn a_new_weapon_family_validates_and_fights_without_kit_code() {
    let t = aegis_core::scratch::Scratch::vacant("emberwake_custom_family");
    let dir = t.join("data");
    copy_dir(&data_dir(), &dir);
    // the copy lies outside the game folder (no asset layers): the rows must add no problem to what it reports
    let before = store::check(&dir);
    let items = stored(&dir, "items");
    let plain = |f: &Dict| json::s(f, "item_type") == "WEAPON" && json::sget(f, "hands") == ["main_1h"] && json::sget(f, "engine_rules").is_empty();
    let base = stored(&dir, "item_families").into_iter().find(|f| plain(f) && items.iter().any(|i| json::s(i, "family") == json::s(f, "id"))).expect("a plain one-handed weapon family with an item");
    let mut family = base.clone();
    family.insert("id".into(), json!(FAMILY));
    family.insert("name".into(), json!("Test Glaive"));
    upsert(&dir, "item_families", &family, None);
    let mut item = items.iter().find(|i| json::s(i, "family") == json::s(&base, "id")).cloned().expect("an item of the family");
    item.insert("id".into(), json!(ITEM));
    item.insert("name".into(), json!("Test Glaive Blade"));
    item.insert("family".into(), json!(FAMILY));
    item.insert("origin".into(), json!("user"));
    upsert(&dir, "items", &item, None);
    // the first Dojo hero's class is trained in the new family
    let hero_class = stored(&dir, "units").iter().find(|u| json::s(u, "dojo") == "hero").map(|u| json::s(u, "class_id")).expect("a Dojo hero");
    let mut class = stored(&dir, "classes").into_iter().find(|c| json::s(c, "id") == hero_class).expect("the hero's class");
    let mut prof = json::dict(json::get(&class, "proficiency")).clone();
    prof.insert(FAMILY.into(), json!({"level": 1}));
    class.insert("proficiency".into(), Value::Object(prof));
    upsert(&dir, "classes", &class, Some(&hero_class));
    let problems = store::check(&dir);
    assert_eq!(problems, before, "the new rows add no problem");
    // the Dojo hero wears it trained and strikes with it
    let (d, errors) = Data::open(&dir).expect("the scratch data");
    assert!(errors.is_empty(), "{errors:?}");
    let mut s = DojoSession::new(d.cat.clone());
    let worn = s.cat.items.get(ITEM).cloned().expect("the new item loads");
    assert_eq!(worn.weapon_type, FAMILY, "the family is its gear type");
    s.clear_gear();
    s.stash.push(ITEM.into());
    assert!(s.equip(ITEM, "main_hand"), "the hero wears it");
    assert!(s.trained_for(&worn), "the class's training covers the new family");
    let main = s.hero().inventory.main_weapon().expect("a main weapon").clone();
    assert_eq!(main.item.id, ITEM);
    assert!(!main.wield_penalty(&s.hero().stats).untrained, "worn trained");
    s.strike("main");
    assert!(s.dummy.hp < DUMMY_MAX_HP, "the strike lands");
    let _ = std::fs::remove_dir_all(&t);
}
