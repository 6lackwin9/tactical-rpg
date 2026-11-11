//! PK2-S4 (`docs/sessions/PK2_GRAFTS.md` section 5) on a scratch world built here (no shipped data): weapon mastery.
//! The expectations derive from the kit's mastery table (`rpgkit_data::mastery`: the level totals, the use XP, the rate
//! x1 at proficiency 2 and x1.5 at 3) and the rows below. Mastery is gained only from the manifest's
//! `durability.from_act` (no key = never); only a main-hand weapon family at proficiency 2 or more gains it (never a
//! shield, an off-hand focus or a proficiency 1 family), up to the top level's total; the sources are the settle's use
//! (each family the member attacked with), a `MASTERY_BOOK` read from the stash and a `drill` recipe (the picked family
//! when the member may master more than one). `mastery_xp` and `forge_rite` are saved only when set. The build opens
//! the wielded family's first n techniques at mastery n, and its keystone at mastery 3 once the forge rite is done.

use aegis_core::json;
use rpgkit_data::mastery::{level_of, scaled, MASTERY_LEVEL_XP, MASTERY_MAX, USE_XP};
use rpgkit_world::mastery::{self, NOT_YET_TEXT};
use rpgkit_world::recipe::{self, Pick};
use rpgkit_world::roster::{self, FightEnd, HeroEnd};
use rpgkit_world::state::{PartyState, UnitState, FORGE_RITE_KEY, MASTERY_XP_KEY};
use rpgkit_world::{new_game, progression, Data, GameState};
use serde_json::{json, Value};

const SMITH: &str = "w:smith";
/// The act the scratch manifest starts mastery in.
const FROM_ACT: i64 = 2;
/// A book's and a drill's XP before the rate.
const BOOK_XP: i64 = 40;
const DRILL_XP: i64 = 20;
const TECHNIQUES: [&str; 3] = ["t_one", "t_two", "t_three"];

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

/// One class with sword at proficiency 3, axe at 2, mace at 1, a shield and a tome (off-hand focus) at 2; a sword with
/// three techniques and a keystone; books for sword and tome; a smith's drill; a campaign world from act 1.
fn data() -> Data {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10});
    let attack = |id: &str| json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY"});
    let catalog = json!({
        "classes": [{"id": "blade", "name": "Blade", "skill_path": {"d_one": 1}, "base_stats": stats, "proficiency": {
            "sword": {"level": 3, "role": "main"}, "axe": {"level": 2, "role": "secondary"}, "mace": {"level": 1, "role": "minor"},
            "shield": {"level": 2}, "tome": {"level": 2}}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
            attack("d_one"), attack(TECHNIQUES[0]), attack(TECHNIQUES[1]), attack(TECHNIQUES[2]), attack("k_sword")
        ],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "blade", "level": 1}],
        "items": [
            {"id": "blade", "name": "Blade", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10,
                "family_techniques": [{"skill": TECHNIQUES[0]}, {"skill": TECHNIQUES[1]}, {"skill": TECHNIQUES[2]}], "family_keystone": "k_sword"},
            {"id": "sword_book", "name": "Sword Book", "item_type": "MASTERY_BOOK", "mastery_family": "sword", "mastery_xp": BOOK_XP},
            {"id": "tome_book", "name": "Tome Book", "item_type": "MASTERY_BOOK", "mastery_family": "tome", "mastery_xp": BOOK_XP},
            {"id": "scrap", "name": "Scrap", "item_type": "MATERIAL", "value": 0}
        ],
        "recipes": [{"id": "drill", "name": "Drill", "npc": "smith", "requires": [], "cost": {"gold": 2}, "target": "member", "effects": [{"effect": "drill", "xp": DRILL_XP}]}]
    });
    let families = json!([
        {"id": "sword", "item_type": "WEAPON", "hands": ["main_1h", "main_2h"]},
        {"id": "axe", "item_type": "WEAPON", "hands": ["main_1h"]},
        {"id": "mace", "item_type": "WEAPON", "hands": ["main_1h"]},
        {"id": "shield", "item_type": "SHIELD", "hands": ["off"]},
        {"id": "tome", "item_type": "WEAPON", "hands": ["off"]},
        {"id": "manual", "item_type": "MASTERY_BOOK"}
    ]);
    let worlds = json!({"w": {"manifest": {"id": "w", "campaign": {"start_act": 1}}, "own": {
        "characters": with_ids("w", json!([{"id": "named", "name": "Named", "role": "party", "base_unit": "hero"}, {"id": "smith", "name": "Smith"}]))
    }}});
    let mut d = Data::from_values(&catalog, &worlds, &json!({"item_families": families}));
    d.from_act = FROM_ACT;
    d
}

/// A new game in act `act`, the hero wielding the blade, 10 gold.
fn game(d: &Data, act: i64) -> GameState {
    let mut m = new_game::hero_from_character(d, "w", "named").expect("a party character");
    m.equipment.insert("main_hand".into(), "blade".into());
    let mut gs = GameState::create(d, "w", PartyState { active_ids: vec![m.id.clone()], members: vec![m], ..PartyState::default() });
    gs.campaign.as_mut().expect("a campaign world").act = act;
    gs.party.gold = 10;
    gs
}

fn hero(gs: &GameState) -> &UnitState {
    &gs.party.members[0]
}

fn hero_id(gs: &GameState) -> String {
    hero(gs).id.clone()
}

fn xp(gs: &GameState, family: &str) -> i64 {
    hero(gs).mastery_xp.get(family).copied().unwrap_or(0)
}

#[test]
fn mastery_is_gained_only_from_the_manifests_act() {
    let mut d = data();
    let mut early = game(&d, FROM_ACT - 1);
    let id = hero_id(&early);
    assert_eq!(mastery::gain(&d, &mut early, &id, "sword", USE_XP), Err(NOT_YET_TEXT.to_string()), "before the act");
    let mut now = game(&d, FROM_ACT);
    assert!(mastery::gain(&d, &mut now, &id, "sword", USE_XP).is_ok(), "from the act");
    let mut later = game(&d, FROM_ACT + 1);
    assert!(mastery::gain(&d, &mut later, &id, "sword", USE_XP).is_ok(), "and after");
    d.from_act = 0;
    let mut none = game(&d, FROM_ACT + 1);
    assert_eq!(mastery::gain(&d, &mut none, &id, "sword", USE_XP), Err(NOT_YET_TEXT.to_string()), "no durability.from_act: never");
    assert!(hero(&none).mastery_xp.is_empty() && hero(&early).mastery_xp.is_empty());
}

#[test]
fn only_main_hand_weapon_families_at_proficiency_2_gain_at_their_rate() {
    let d = data();
    let mut gs = game(&d, FROM_ACT);
    let id = hero_id(&gs);
    assert_eq!(mastery::masterable_families(&d, hero(&gs)), ["sword", "axe"], "not the shield, the tome or the proficiency 1 mace");
    let g = mastery::gain(&d, &mut gs, &id, "sword", USE_XP).unwrap();
    assert_eq!((g.from, g.to), (0, scaled(USE_XP, 3)), "x1.5 at proficiency 3");
    assert_eq!(scaled(USE_XP, 3), (USE_XP as f64 * 1.5).round() as i64);
    mastery::gain(&d, &mut gs, &id, "axe", USE_XP).unwrap();
    assert_eq!(xp(&gs, "axe"), USE_XP, "x1 at proficiency 2");
    for family in ["shield", "tome", "mace", "bow"] {
        let before = hero(&gs).clone();
        assert!(mastery::gain(&d, &mut gs, &id, family, USE_XP).unwrap_err().contains("can't master"), "{family}");
        assert_eq!(hero(&gs), &before, "{family}: nothing changes");
    }
    let top = MASTERY_LEVEL_XP[MASTERY_LEVEL_XP.len() - 1];
    let g = mastery::gain(&d, &mut gs, &id, "sword", top).unwrap();
    assert_eq!((g.to, g.level(), level_of(top)), (top, MASTERY_MAX, MASTERY_MAX), "the bar stops at the top level's total");
    assert!(mastery::gain(&d, &mut gs, &id, "sword", USE_XP).unwrap_err().contains("mastered"));
}

fn end(member: &str, attacked: &[&str]) -> HeroEnd {
    HeroEnd {
        member: member.into(),
        hp: 10,
        max_hp: 10,
        alive: true,
        pouch: Vec::new(),
        unloaded: Vec::new(),
        fate: None,
        marks: Vec::new(),
        hostile: String::new(),
        attacked: attacked.iter().map(|s| s.to_string()).collect(),
    }
}

#[test]
fn the_settle_gains_the_use_xp_in_each_family_attacked_with() {
    let d = data();
    let mut gs = game(&d, FROM_ACT);
    let id = hero_id(&gs);
    let settled = roster::settle_party(&d, &mut gs, &FightEnd::new("won", "w"), &[end(&id, &["sword", "axe", "tome"])]);
    let gained: Vec<(String, i64)> = settled.mastery.iter().map(|g| (g.family.clone(), g.to - g.from)).collect();
    assert_eq!(gained, [("sword".to_string(), scaled(USE_XP, 3)), ("axe".to_string(), USE_XP)], "the tome gains nothing");
    assert_eq!((xp(&gs, "sword"), xp(&gs, "axe")), (scaled(USE_XP, 3), USE_XP));
    let mut early = game(&d, FROM_ACT - 1);
    let settled = roster::settle_party(&d, &mut early, &FightEnd::new("won", "w"), &[end(&id, &["sword"])]);
    assert!(settled.mastery.is_empty() && hero(&early).mastery_xp.is_empty(), "before the act the settle gains nothing");
}

#[test]
fn a_mastery_book_is_read_from_the_stash() {
    let d = data();
    let mut gs = game(&d, FROM_ACT);
    let id = hero_id(&gs);
    let res = mastery::read_book(&d, &mut gs, &id, "sword_book");
    assert!(!json::b(&res, "ok", true), "none in the stash");
    gs.party.add_to_stash("sword_book", 2);
    gs.party.add_to_stash("tome_book", 1);
    gs.party.add_to_stash("scrap", 1);
    let res = mastery::read_book(&d, &mut gs, &id, "sword_book");
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!((xp(&gs, "sword"), gs.party.stash["sword_book"]), (scaled(BOOK_XP, 3), 1), "the rate applies; one copy used");
    for item in ["tome_book", "scrap"] {
        let before = gs.clone();
        let res = mastery::read_book(&d, &mut gs, &id, item);
        assert!(!json::b(&res, "ok", true), "{item}");
        assert_eq!((gs.party.stash.clone(), hero(&gs).clone()), (before.party.stash.clone(), hero(&before).clone()), "{item}: nothing changes");
    }
    let mut early = game(&d, FROM_ACT - 1);
    early.party.add_to_stash("sword_book", 1);
    assert!(!json::b(&mastery::read_book(&d, &mut early, &id, "sword_book"), "ok", true));
    assert_eq!(early.party.stash["sword_book"], 1, "kept before the act");
}

#[test]
fn a_drill_recipe_gains_mastery_in_the_picked_family() {
    let d = data();
    let mut gs = game(&d, FROM_ACT);
    let id = hero_id(&gs);
    let on = |family: &str| Pick { member: id.clone(), family: family.into(), ..Pick::default() };
    assert_eq!(recipe::refusal(&d, &gs, SMITH, "drill", &on("")), Some("Pick a weapon family (sword, axe).".to_string()));
    assert!(recipe::refusal(&d, &gs, SMITH, "drill", &on("tome")).is_some_and(|r| r.contains("can't master")));
    let res = recipe::run(&d, &mut gs, SMITH, "drill", &on("axe"));
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!((xp(&gs, "axe"), gs.party.gold), (scaled(DRILL_XP, 2), 8), "the rate applies; the cost is paid");
    let mut early = game(&d, FROM_ACT - 1);
    assert_eq!(recipe::refusal(&d, &early, SMITH, "drill", &on("axe")), Some(NOT_YET_TEXT.to_string()));
    assert!(!json::b(&recipe::run(&d, &mut early, SMITH, "drill", &on("axe")), "ok", true));
    assert_eq!(early.party.gold, 10, "a refusal pays nothing");
}

#[test]
fn mastery_and_the_forge_rite_save_only_when_set() {
    let d = data();
    let mut gs = game(&d, FROM_ACT);
    let id = hero_id(&gs);
    let v = hero(&gs).to_value();
    assert!(v.get(MASTERY_XP_KEY).is_none() && v.get(FORGE_RITE_KEY).is_none(), "a member without mastery saves as before");
    mastery::gain(&d, &mut gs, &id, "axe", USE_XP).unwrap();
    gs.party.members[0].set_forge_rite(true);
    let v = hero(&gs).to_value();
    assert_eq!((&v[MASTERY_XP_KEY], &v[FORGE_RITE_KEY]), (&json!({"axe": USE_XP}), &json!(true)));
    let back = UnitState::from_dict(json::dict(&v));
    assert_eq!(&back, hero(&gs), "the keys load back");
    let mut bad = back.clone();
    bad.mastery_xp.insert("axe".into(), -1);
    assert!(bad.validate(&d).iter().any(|e| e.ends_with("mastery_xp axe must be 0 or more.")));
}

/// The XP for mastery `level` in a family (0 = none).
fn xp_for(level: usize) -> i64 {
    if level == 0 {
        0
    } else {
        MASTERY_LEVEL_XP[level - 1]
    }
}

#[test]
fn the_build_opens_the_wielded_familys_techniques_and_its_keystone() {
    let d = data();
    let mut gs = game(&d, FROM_ACT);
    for level in 0..=MASTERY_MAX as usize {
        let m = &mut gs.party.members[0];
        m.mastery_xp.insert("sword".into(), xp_for(level));
        let u = progression::build(&d, m).expect("builds").unit;
        let open: Vec<String> = u.techniques.iter().map(|t| t.skill.clone()).collect();
        assert_eq!(open, TECHNIQUES[..level].to_vec(), "mastery {level}");
        assert!(u.mastery_keystone.is_empty(), "no keystone without the forge rite");
    }
    let m = &mut gs.party.members[0];
    m.set_forge_rite(true);
    let u = progression::build(&d, m).unwrap().unit;
    assert_eq!(u.mastery_keystone, "k_sword", "mastery 3 and the forge rite");
    assert!(u.keystone_skills.contains(&"k_sword".to_string()) && u.always_active(&d.cat, "k_sword"), "it sits in the keystone slot");
    m.mastery_xp.insert("sword".into(), xp_for(MASTERY_MAX as usize - 1));
    assert!(progression::build(&d, m).unwrap().unit.mastery_keystone.is_empty(), "below mastery 3");
    m.mastery_xp.insert("sword".into(), xp_for(MASTERY_MAX as usize));
    m.mastery_xp.insert("axe".into(), xp_for(1));
    m.equipment.shift_remove("main_hand");
    let bare = progression::build(&d, m).unwrap().unit;
    assert!(bare.techniques.is_empty() && bare.mastery_keystone.is_empty(), "nothing without a sword in the main hand");
}
