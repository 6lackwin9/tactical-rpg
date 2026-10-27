//! WAR2H1: the warrior's look follows the equipped gear (`art::Grip`, `art::gear_body_chain`): a two-handed weapon,
//! or a weapon with no shield, gives the 2H look of its family; a light shield (a buckler) the 1H + buckler look; a
//! heavier shield the class sheet (1H + kite shield); named bodies keep their own art. Fixture items are built here;
//! the shipped checks derive what to expect from the item families, the warrior class, the shields and the layers.

use aegis_core::assets::{self, Clips, Layers, View};
use aegis_core::json;
use emberwake_content::art::{self, Grip, OffHand, BUCKLER_LOOK, BUCKLER_MAX_BURDEN};
use emberwake_content::{source, store};
use emberwake_data::{Catalog, ItemDef};
use emberwake_rules::gear::{Equipment, Inventory, Slot};
use serde_json::json;
use std::path::PathBuf;

fn item(item_type: &str, family: &str, two_handed: bool, burden: i32) -> ItemDef {
    let off_cat = if item_type == "SHIELD" { "shield" } else { "" };
    let slot = if item_type == "SHIELD" { "OFF_HAND" } else { "MAIN_HAND" };
    ItemDef::from_obj(json!({"id": format!("test_{family}"), "item_type": item_type, "weapon_type": family, "two_handed": two_handed,
                             "off_hand_category": off_cat, "slot": slot, "burden": burden}).as_object().unwrap())
}

fn weapon(family: &str, two: bool) -> ItemDef {
    item("WEAPON", family, two, 1)
}

fn shield() -> ItemDef {
    item("SHIELD", "shield", false, BUCKLER_MAX_BURDEN + 2)
}

fn buckler() -> ItemDef {
    item("SHIELD", "shield", false, BUCKLER_MAX_BURDEN)
}

fn chain(body: &str, main: Option<&ItemDef>, off: Option<&ItemDef>) -> Vec<String> {
    art::gear_body_chain(body, &Grip::of(main, off))
}

#[test]
fn the_look_follows_the_main_hand_and_the_off_hand() {
    let (w2, w1, sh) = (weapon("hammer", true), weapon("axe", false), shield());
    assert_eq!(chain("warrior", Some(&w2), None), ["warrior/2h_hammer", "warrior/2h_sword", "warrior"], "a two-handed weapon");
    assert_eq!(chain("warrior", Some(&w1), Some(&sh)), ["warrior"], "one hand and a shield: the class sheet");
    assert_eq!(chain("warrior", Some(&w1), None), ["warrior/2h_axe", "warrior/2h_sword", "warrior"], "an empty off hand");
    assert_eq!(chain("warrior", Some(&weapon("sword", true)), None), ["warrior/2h_sword", "warrior"], "the fallback is not tried twice");
    assert_eq!(chain("warrior", Some(&w2), Some(&weapon("dagger", false))), ["warrior/2h_hammer", "warrior/2h_sword", "warrior"], "a 2H weapon with a spare off-hand weapon");
    assert_eq!(chain("warrior", Some(&w1), Some(&item("WEAPON", "orb", false, 0))), ["warrior/2h_axe", "warrior/2h_sword", "warrior"], "a focus is no shield");
    assert_eq!(chain("warrior", Some(&weapon("mace", false)), None)[1], "warrior/2h_sword", "a family without its own look falls back to the greatsword");
    assert_eq!(chain("warrior", None, Some(&sh)), ["warrior"], "no weapon");
    assert_eq!(chain("warrior", None, None), ["warrior"]);
    assert_eq!(chain("warrior/base", Some(&w2), None)[0], "warrior/2h_hammer", "the explicit base key is the class base");
    assert_eq!(chain("warrior", Some(&item("ARMOR", "plate", false, 0)), None), ["warrior"], "only a weapon counts in the main hand");
}

#[test]
fn a_light_shield_gives_the_buckler_look() {
    let w1 = weapon("axe", false);
    let want = [format!("warrior/{BUCKLER_LOOK}"), "warrior".to_string()];
    assert_eq!(chain("warrior", Some(&w1), Some(&buckler())), want, "one hand and a buckler (no 2H fallback)");
    assert_eq!(chain("warrior", None, Some(&buckler())), want, "a buckler alone still shows it");
    assert_eq!(Grip::of(Some(&w1), Some(&buckler())).off_hand, OffHand::Buckler);
    let heavier = item("SHIELD", "shield", false, BUCKLER_MAX_BURDEN + 1);
    assert_eq!(Grip::of(Some(&w1), Some(&heavier)).off_hand, OffHand::Shield, "one point of burden over the buckler is a shield");
    assert_eq!(chain("warrior/brask_toller", Some(&w1), Some(&buckler())), art::body_chain("warrior/brask_toller"), "named bodies keep their art");
}

#[test]
fn named_and_variant_bodies_keep_their_own_art() {
    let w2 = weapon("sword", true);
    assert_eq!(chain("warrior/grakka_ashmaw", Some(&w2), None), art::body_chain("warrior/grakka_ashmaw"));
    assert_eq!(chain("brawler/goblin", Some(&weapon("axe", false)), None), ["brawler/goblin", "brawler"]);
}

#[test]
fn the_grip_reads_the_equipped_gear() {
    let mut inv = Inventory::new();
    assert_eq!(Grip::of_inventory(&inv), Grip { main_family: String::new(), two_handed: false, off_hand: OffHand::Empty });
    inv.equip(Equipment::new(&weapon("axe", false)), Slot::MainHand);
    inv.equip(Equipment::new(&shield()), Slot::OffHand);
    assert_eq!(Grip::of_inventory(&inv), Grip { main_family: "axe".into(), two_handed: false, off_hand: OffHand::Shield });
    assert_eq!(Grip::of_inventory(&inv).look(), None);
    inv.equip(Equipment::new(&buckler()), Slot::OffHand);
    assert_eq!(Grip::of_inventory(&inv).look().as_deref(), Some(BUCKLER_LOOK));
    inv.unequip(Slot::OffHand);
    assert_eq!(Grip::of_inventory(&inv).look().as_deref(), Some("2h_axe"));
}

fn game_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn clips(layers: &Layers, body: &str) -> Clips {
    let v = assets::read_json(layers, &assets::body_file(body, "clips.json")).unwrap_or_else(|| panic!("{body}: no clips.json"));
    Clips::parse(&v).unwrap_or_else(|e| panic!("{body}: {e:?}"))
}

/// The body `chain` resolves to: the first key of the chain, in order, that is a body in the layers.
fn resolved(layers: &Layers, chain: Vec<String>) -> Option<String> {
    let bodies = layers.bodies();
    chain.into_iter().find(|k| bodies.contains(k))
}

/// Every family the warrior may wield two-handed (its hands allow main_2h and the class has proficiency in it) has a
/// baked 2H look with the base's side clips less the shield's; the buckler look has the base's side clips; every look
/// shares the base's palette slots (DIR4 L3) and is the body its grip resolves to. View clips (DIR4) are compared
/// apart: a look has them only once its grip and weapon have view drawings (DIR4 L2), and then the same ones.
#[test]
fn every_warrior_look_is_baked() {
    let v = store::view(&game_dir().join("data"));
    let layers = Layers::for_game(&game_dir());
    let prof = json::dict(json::get(v.get("classes", "warrior"), "proficiency"));
    let families: Vec<String> = v
        .entries("item_families")
        .iter()
        .filter(|f| json::strings(json::get(f, "hands")).iter().any(|h| h == "main_2h") && prof.contains_key(&json::s(f, "id")))
        .map(|f| json::s(f, "id"))
        .collect();
    assert!(!families.is_empty(), "the warrior wields some two-handed family");
    let base = clips(&layers, "warrior");
    let all: Vec<&str> = base.clips.iter().filter(|c| c.view == View::Side).map(|c| c.id.as_str()).collect();
    let views = |c: &Clips| c.clips.iter().filter(|c| c.view != View::Side).map(|c| (c.of.clone(), c.view)).collect::<Vec<_>>();
    let no_shield: Vec<&str> = all.iter().copied().filter(|id| !id.starts_with("shield")).collect();
    let mut grips: Vec<(Grip, Vec<&str>)> =
        families.iter().map(|fam| (Grip { main_family: fam.clone(), two_handed: true, off_hand: OffHand::Empty }, no_shield.clone())).collect();
    grips.push((Grip { main_family: "sword".into(), two_handed: false, off_hand: OffHand::Buckler }, all.clone()));
    for (grip, want) in grips {
        let key = format!("warrior/{}", grip.look().unwrap());
        let look = clips(&layers, &key);
        assert_eq!(look.palette, base.palette, "{key}: the look shares the base's palette slots");
        assert_eq!(look.clips.iter().filter(|c| c.view == View::Side).map(|c| c.id.as_str()).collect::<Vec<_>>(), want, "{key}: clips");
        let lv = views(&look);
        assert!(lv.is_empty() || lv == views(&base), "{key}: view clips {lv:?}, the base's {:?}", views(&base));
        assert_eq!(resolved(&layers, art::gear_body_chain("warrior", &grip)).as_deref(), Some(key.as_str()));
    }
}

/// The shipped shields split by the burden rule: at least one buckler and one heavier shield, each resolving to a
/// baked body.
#[test]
fn the_shipped_shields_have_both_looks() {
    let cat = Catalog::from_value(&source::load(&game_dir().join("data")).catalog);
    let layers = Layers::for_game(&game_dir());
    let sword = cat.items.values().find(|i| i.weapon_type == "sword" && !i.two_handed).expect("a one-hand sword");
    let shields: Vec<&ItemDef> = cat.items.values().filter(|i| i.item_type == "SHIELD").collect();
    let looks: Vec<OffHand> = shields.iter().map(|s| Grip::of(Some(sword), Some(s)).off_hand).collect();
    assert!(looks.contains(&OffHand::Buckler) && looks.contains(&OffHand::Shield), "shields {:?} -> {looks:?}", shields.iter().map(|s| (&s.id, s.burden)).collect::<Vec<_>>());
    for s in shields {
        assert!(resolved(&layers, art::gear_body_chain("warrior", &Grip::of(Some(sword), Some(s)))).is_some(), "{}", s.id);
    }
}

/// The RGBA pixels of a body's sheet.
fn pixels(layers: &Layers, body: &str) -> Vec<[u8; 4]> {
    let path = layers.find(&assets::body_file(body, "sheet.png")).unwrap_or_else(|| panic!("{body}: no sheet"));
    let mut reader = png::Decoder::new(std::fs::File::open(path).unwrap()).read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    assert_eq!(info.color_type, png::ColorType::Rgba, "{body}: an RGBA sheet");
    buf[..info.buffer_size()].chunks(4).map(|c| [c[0], c[1], c[2], c[3]]).collect()
}

/// Only the class base draws the kite shield: its `shield` slot colour is in the base's sheet and in no look's.
#[test]
fn only_the_class_base_draws_the_kite_shield() {
    let layers = Layers::for_game(&game_dir());
    let kite = *clips(&layers, "warrior").palette.get("shield").expect("the base has a shield slot");
    let has = |body: &str| pixels(&layers, body).iter().any(|p| p[3] == 255 && p[..3] == kite);
    assert!(has("warrior"), "the class base keeps 1H + kite shield");
    let looks: Vec<String> = layers.bodies().into_iter().filter(|b| b.starts_with("warrior/") && clips(&layers, b).palette.get("shield") == Some(&kite)).collect();
    assert!(!looks.is_empty());
    for b in looks {
        assert!(!has(&b), "{b} draws the kite shield");
    }
}
