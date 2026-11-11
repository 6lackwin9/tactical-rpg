//! PK2-S3 (`docs/sessions/PK2_GRAFTS.md` section 4) on a scratch world built here (no shipped data): the NPC service
//! `recipe`. A character offers the recipe rows naming it while their `requires` hold (the service badge, the
//! conversation's service choice opening and closing the pane); a run checks everything first and changes nothing on a
//! refusal, else pays the cost (purse and stash items) and applies the effects in order: `install` (an empty slot, or
//! an overwrite destroying the old graft), `remove` (excision), `track` (a raise never into a departure stage, a
//! lowering needs a stage), `max_hp_pct` (a percent of the max HP now, for good), `floor_offset` (a member with a
//! graft), `train_proficiency` (one level of a family of the class role, from `to - 1`; saved as `trained`, read by the
//! build), `salvage` (the stash item broken down into gold and the salvage its rarity and theme name), `doctrine`
//! (reserved: nothing changes but the cost); a `weapon` recipe is refused while weapon instances are not active (its
//! runs are `tests/weapon_recipes.rs`'); PK2-S4's `drill` is refused while mastery is closed (its runs are
//! `tests/mastery.rs`').

use aegis_core::json;
use rpgkit_world::dialogue::DialogueRunner;
use rpgkit_world::recipe::{self, Pick};
use rpgkit_world::state::{PartyState, UnitState, TRAINED_KEY};
use rpgkit_world::{explore, new_game, progression, Data, GameState};
use serde_json::{json, Value};

const SURGEON: &str = "w:surgeon";
const SMITH: &str = "w:smith";
const GOLD: i64 = 100;
const SURGERY_GOLD: i64 = 10;

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

fn graft(id: &str, skill: &str, tier: i64) -> Value {
    json!({"id": id, "name": id, "item_type": "GRAFT", "tier": tier, "body_role": "monster",
        "body_effects": [{"type": "skill", "skill": skill}, {"type": "meter", "meter": "t_gauge", "floors": [5, 15, 30, 50]}]})
}

fn recipe(id: &str, npc: &str, target: &str, cost: Value, effects: Value) -> Value {
    json!({"id": id, "name": id, "npc": npc, "requires": [], "cost": cost, "target": target, "effects": effects})
}

/// One class with roles on its proficiency (main, secondary, third, two minors), three monster body slots, grafts of
/// tiers 1 and 3, a three-stage track whose last stage departs, items to break down, and the recipes of a surgeon and a
/// smith.
fn data() -> Data {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10});
    let attack = |id: &str| json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY", "channel": {"only": true}});
    let mut recipes = vec![
        recipe("surgery_a", "surgeon", "member", json!({"gold": SURGERY_GOLD, "items": {"core": 1, "claw_mat": 1}}), json!([{"effect": "install", "item": "g_claw", "slot": "s_a"}])),
        recipe("surgery_b", "surgeon", "member", json!({}), json!([{"effect": "install", "item": "g_bite", "slot": "s_b"}])),
        recipe("surgery_c", "surgeon", "member", json!({}), json!([{"effect": "install", "item": "g_sting", "slot": "s_c"}])),
        recipe("surgery_top", "surgeon", "member", json!({}), json!([{"effect": "install", "item": "g_top", "slot": "s_a"}, {"effect": "track", "track": "rot", "delta": 1}])),
        recipe("excise_a", "surgeon", "member", json!({}), json!([{"effect": "remove", "slot": "s_a"}, {"effect": "max_hp_pct", "pct": -15}])),
        recipe("flush", "surgeon", "member", json!({"items": {"salt": 1}}), json!([{"effect": "floor_offset", "delta": -10}])),
        recipe("doctrine_only", "surgeon", "member", json!({"gold": 1}), json!([{"effect": "doctrine", "axis": "flesh", "delta": 1}])),
        recipe("train_secondary", "smith", "member", json!({"gold": 5}), json!([{"effect": "train_proficiency", "role": "secondary", "to": 3}])),
        recipe("train_minor", "smith", "member", json!({}), json!([{"effect": "train_proficiency", "role": "minor", "to": 2}])),
        recipe("break_down", "smith", "item", json!({}), json!([{"effect": "salvage", "gold": 3, "tiers": {"RARE": "fine_salvage"}, "themes": {"order": "order_steel"}}])),
        recipe("break_doctrine", "smith", "item", json!({}), json!([{"effect": "salvage", "gold": 1}, {"effect": "doctrine", "axis": "steel", "delta": 1}])),
        recipe("drill_it", "smith", "member", json!({"gold": 1}), json!([{"effect": "drill", "xp": 10}])),
        recipe("repair_it", "smith", "weapon", json!({"gold": 1}), json!([{"effect": "repair"}])),
    ];
    let mut scrape = recipe("scrape", "surgeon", "member", json!({}), json!([{"effect": "track", "track": "rot", "delta": -1}, {"effect": "max_hp_pct", "pct": -10}]));
    scrape["requires"] = json!([{"kind": "flag", "key": "scrape_open"}]);
    recipes.push(scrape);
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "skill_path": {"d_one": 1}, "base_stats": stats, "proficiency": {
            "sword": {"level": 3, "role": "main"}, "bow": {"level": 2, "role": "secondary"}, "axe": {"level": 2, "role": "third"},
            "mace": {"level": 1, "role": "minor"}, "dagger": {"level": 1, "role": "minor"}}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
            {"id": "d_one", "name": "One", "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY"},
            attack("m_claw"), attack("m_bite"), attack("m_sting"), attack("m_top")
        ],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1}],
        "items": [
            graft("g_claw", "m_claw", 1), graft("g_bite", "m_bite", 1), graft("g_sting", "m_sting", 1), graft("g_top", "m_top", 3),
            {"id": "core", "name": "Core", "item_type": "MATERIAL", "value": 0},
            {"id": "claw_mat", "name": "Claw", "item_type": "MATERIAL", "value": 0},
            {"id": "salt", "name": "Salt", "item_type": "MATERIAL", "value": 0},
            {"id": "fine_salvage", "name": "Fine Salvage", "item_type": "MATERIAL", "value": 0},
            {"id": "order_steel", "name": "Order Steel", "item_type": "MATERIAL", "value": 0},
            {"id": "order_blade", "name": "Order Blade", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10, "value": 5, "rarity": "RARE", "theme": "order"},
            {"id": "plain_blade", "name": "Plain Blade", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 8, "value": 2, "rarity": "COMMON"},
            {"id": "letter", "name": "Letter", "item_type": "QUEST", "value": 0}
        ],
        "unit_meters": [{"id": "t_gauge", "name": "Gauge", "short": "Gauge", "min": 0, "max": 100, "start": 0, "show": "bar"}],
        "body_slots": [{"id": "s_a", "name": "A", "role": "monster"}, {"id": "s_b", "name": "B", "role": "monster"}, {"id": "s_c", "name": "C", "role": "monster"}],
        "unit_tracks": [{"id": "rot", "name": "Rot", "mark": "rot", "cause": "rotted", "stages": [{"name": "One"}, {"name": "Two"}, {"name": "Gone", "departure": "lost"}]}],
        "recipes": recipes
    });
    let worlds = json!({"w": {"manifest": {"id": "w"}, "own": {
        "characters": with_ids("w", json!([
            {"id": "named", "name": "Named", "role": "party", "base_unit": "hero"},
            {"id": "surgeon", "name": "Surgeon"},
            {"id": "smith", "name": "Smith"},
            {"id": "idler", "name": "Idler"}
        ]))
    }}});
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn game(d: &Data) -> GameState {
    let m = new_game::hero_from_character(d, "w", "named").expect("a party character");
    let mut gs = GameState::create(d, "w", PartyState { active_ids: vec![m.id.clone()], members: vec![m], ..PartyState::default() });
    gs.party.gold = GOLD;
    for item in ["core", "claw_mat", "salt"] {
        gs.party.add_to_stash(item, 2);
    }
    gs
}

fn hero(gs: &GameState) -> &UnitState {
    &gs.party.members[0]
}

fn on_hero(gs: &GameState) -> Pick {
    Pick { member: hero(gs).id.clone(), ..Pick::default() }
}

fn ok(d: &Data, gs: &mut GameState, npc: &str, id: &str, pick: &Pick) -> Vec<String> {
    let res = recipe::run(d, gs, npc, id, pick);
    assert!(json::b(&res, "ok", false), "{id}: {res:?}");
    json::sget(&res, "notes")
}

/// A refused run: the refusal, and the state is unchanged.
fn refused(d: &Data, gs: &mut GameState, npc: &str, id: &str, pick: &Pick) -> String {
    let before = gs.to_value();
    let res = recipe::run(d, gs, npc, id, pick);
    assert!(!json::b(&res, "ok", true), "{id} should be refused");
    assert_eq!(gs.to_value(), before, "{id}: a refusal changes nothing");
    json::sget(&res, "errors").join(" ")
}

#[test]
fn a_character_offers_its_rows_while_their_requires_hold() {
    let d = data();
    let mut gs = game(&d);
    let ids = |gs: &GameState, npc: &str| recipe::offered(&d, gs, npc).iter().map(|r| r.id.clone()).collect::<Vec<_>>();
    assert!(!ids(&gs, SURGEON).contains(&"scrape".to_string()), "its flag is not set");
    assert!(ids(&gs, SMITH).contains(&"break_down".to_string()) && !ids(&gs, SMITH).contains(&"surgery_a".to_string()));
    assert!(ids(&gs, "w:idler").is_empty());
    gs.set_flag("scrape_open", Value::Bool(true));
    assert!(ids(&gs, SURGEON).contains(&"scrape".to_string()), "its requires hold now");
    assert!(explore::npc_services(&d, &gs, SURGEON).contains(&"recipe"), "the service badge");
    assert!(!explore::npc_services(&d, &gs, "w:idler").contains(&"recipe"));
    let pick = on_hero(&gs);
    assert_eq!(refused(&d, &mut gs, "w:idler", "surgery_a", &pick), recipe::NOT_OFFERED_TEXT);
    gs.clear_flag("scrape_open");
    assert_eq!(refused(&d, &mut gs, SURGEON, "scrape", &pick), recipe::NOT_NOW_TEXT);
}

#[test]
fn the_conversation_opens_and_closes_the_recipe_pane() {
    let d = data();
    let mut gs = game(&d);
    let mut r = DialogueRunner::open(&d, &mut gs, SURGEON);
    let view = r.view(&d, &gs);
    let at = json::aget(&view, "choices").iter().position(|c| json::s(json::dict(c), "service") == "recipe").expect("the recipe service choice");
    let res = r.choose(&d, &mut gs, at as i64);
    assert_eq!(json::s(&res, "recipe"), SURGEON, "{res:?}");
    assert_eq!((r.recipe_npc.as_str(), json::s(&r.view(&d, &gs), "recipe")), (SURGEON, SURGEON.to_string()));
    assert!(!json::b(&r.choose(&d, &mut gs, 0), "ok", true), "the conversation waits while the pane is open");
    assert!(json::b(&r.close_recipes(&d, &mut gs), "ok", false));
    assert!(r.recipe_npc.is_empty() && !r.view(&d, &gs).contains_key("recipe"));
    let idle = DialogueRunner::open(&d, &mut gs, "w:idler");
    assert!(!json::aget(&idle.view(&d, &gs), "choices").iter().any(|c| json::s(json::dict(c), "service") == "recipe"), "nothing offered: no choice");
}

#[test]
fn install_fills_an_empty_slot_and_an_overwrite_destroys_the_old_graft() {
    let d = data();
    let mut gs = game(&d);
    let pick = on_hero(&gs);
    let (core, mat) = (gs.party.stash["core"], gs.party.stash["claw_mat"]);
    ok(&d, &mut gs, SURGEON, "surgery_a", &pick);
    assert_eq!(hero(&gs).body.get("s_a").map(String::as_str), Some("g_claw"));
    assert_eq!((gs.party.gold, gs.party.stash["core"], gs.party.stash["claw_mat"]), (GOLD - SURGERY_GOLD, core - 1, mat - 1), "the cost is paid");
    for id in ["surgery_b", "surgery_c"] {
        ok(&d, &mut gs, SURGEON, id, &pick);
    }
    let notes = ok(&d, &mut gs, SURGEON, "surgery_top", &pick);
    assert_eq!((hero(&gs).body["s_a"].as_str(), hero(&gs).graft_count()), ("g_top", 3), "overwritten; the count stays");
    assert!(notes.iter().any(|n| n.contains("g_claw") && n.contains("destroyed")), "{notes:?}");
    assert!(!gs.party.stash.contains_key("g_claw"), "no refund");
}

#[test]
fn a_refused_install_pays_nothing() {
    let d = data();
    let mut gs = game(&d);
    let pick = on_hero(&gs);
    let why = refused(&d, &mut gs, SURGEON, "surgery_top", &pick);
    assert!(why.contains("needs 3 grafts first"), "{why}");
    gs.party.gold = SURGERY_GOLD - 1;
    assert_eq!(refused(&d, &mut gs, SURGEON, "surgery_a", &pick), "Not enough gold.");
    gs.party.gold = GOLD;
    gs.party.stash.shift_remove("core");
    assert_eq!(refused(&d, &mut gs, SURGEON, "surgery_a", &pick), "Not enough Core.");
    assert_eq!(refused(&d, &mut gs, SURGEON, "surgery_b", &Pick::default()), "Pick a party member.");
}

#[test]
fn track_moves_the_stage_within_the_row_never_into_a_departure() {
    let d = data();
    let mut gs = game(&d);
    let pick = on_hero(&gs);
    gs.set_flag("scrape_open", Value::Bool(true));
    assert!(refused(&d, &mut gs, SURGEON, "scrape", &pick).contains("no Rot to lower"));
    for id in ["surgery_a", "surgery_b", "surgery_c", "surgery_top"] {
        ok(&d, &mut gs, SURGEON, id, &pick);
    }
    assert_eq!(hero(&gs).track_stage("rot"), 1, "the tier 3 surgery adds a stage");
    gs.party.members[0].tracks.insert("rot".into(), 2);
    let why = refused(&d, &mut gs, SURGEON, "surgery_top", &pick);
    assert!(why.contains("would reach the last stage of Rot"), "the next stage departs: {why}");
    ok(&d, &mut gs, SURGEON, "scrape", &pick);
    assert_eq!(hero(&gs).track_stage("rot"), 1, "bone-scraping lowers it");
    ok(&d, &mut gs, SURGEON, "scrape", &pick);
    assert!(!hero(&gs).tracks.contains_key("rot"), "stage 0 is no entry (not saved)");
}

#[test]
fn excision_empties_the_slot_and_lowers_max_hp_for_good() {
    let d = data();
    let mut gs = game(&d);
    let pick = on_hero(&gs);
    assert!(refused(&d, &mut gs, SURGEON, "excise_a", &pick).contains("body slot s_a is empty"));
    ok(&d, &mut gs, SURGEON, "surgery_a", &pick);
    let max = progression::max_hp_of(&d, hero(&gs), "w");
    ok(&d, &mut gs, SURGEON, "excise_a", &pick);
    assert!(hero(&gs).body.is_empty(), "the slot is empty");
    let delta = recipe::max_hp_delta(max, -15);
    assert!(delta < 0);
    assert_eq!(hero(&gs).max_hp_offset, delta);
    assert_eq!(progression::max_hp_of(&d, hero(&gs), "w"), max + delta, "the build reads the offset");
}

#[test]
fn max_hp_delta_rounds_half_away_from_zero_and_never_to_zero() {
    assert_eq!(recipe::max_hp_delta(40, -15), -6);
    assert_eq!(recipe::max_hp_delta(30, -15), -5, "-4.5 rounds away from zero");
    assert_eq!(recipe::max_hp_delta(3, -10), -1, "a percent never rounds to nothing");
    assert_eq!(recipe::max_hp_delta(50, 10), 5);
}

#[test]
fn floor_offset_needs_a_graft_and_adds_up() {
    let d = data();
    let mut gs = game(&d);
    let pick = on_hero(&gs);
    assert!(refused(&d, &mut gs, SURGEON, "flush", &pick).contains("holds no graft"));
    ok(&d, &mut gs, SURGEON, "surgery_a", &pick);
    let salt = gs.party.stash["salt"];
    ok(&d, &mut gs, SURGEON, "flush", &pick);
    assert_eq!((hero(&gs).floor_offset, gs.party.stash["salt"]), (-10, salt - 1));
    ok(&d, &mut gs, SURGEON, "flush", &pick);
    assert_eq!(hero(&gs).floor_offset, -20);
}

#[test]
fn training_raises_one_family_of_the_role_by_a_level() {
    let d = data();
    let mut gs = game(&d);
    let pick = on_hero(&gs);
    let level = |gs: &GameState, fam: &str| {
        let m = hero(gs);
        m.proficiency(&d, &d.cat.classes[&m.class_id(&d)]).level_of(fam)
    };
    assert_eq!(level(&gs, "bow"), 2);
    assert!(hero(&gs).to_value().get(TRAINED_KEY).is_none(), "an untrained member saves as before");
    ok(&d, &mut gs, SMITH, "train_secondary", &pick);
    assert_eq!((level(&gs, "bow"), gs.party.gold), (3, GOLD - 5));
    let v = hero(&gs).to_value();
    assert_eq!(v[TRAINED_KEY], json!({"bow": 1}));
    assert_eq!(UnitState::from_dict(json::dict(&v)).to_value(), v, "read back the same");
    let built = progression::build(&d, hero(&gs)).expect("a build");
    assert_eq!(built.unit.stats.proficiency.level_of("bow"), 3, "the build reads the trained level");
    assert!(refused(&d, &mut gs, SMITH, "train_secondary", &pick).contains("bow level 3"), "already at the level");
    let why = refused(&d, &mut gs, SMITH, "train_minor", &pick);
    assert!(why.contains("Pick a minor family (mace, dagger)"), "{why}");
    ok(&d, &mut gs, SMITH, "train_minor", &Pick { family: "dagger".into(), ..pick.clone() });
    assert_eq!((level(&gs, "dagger"), level(&gs, "mace")), (2, 1));
    assert!(refused(&d, &mut gs, SMITH, "train_minor", &Pick { family: "bow".into(), ..pick }).contains("not a minor family"));
}

/// PK2-FOLLOWUP (FB-212 (d)): the pane's one family picker. A row lists the families its pick may name (`families`:
/// a role holding more than one), and each row reads the pick with `pick_for`, so a minor family picked for the minor
/// training never refuses a row with a single family (the secondary training) and the minor row runs on the pick.
#[test]
fn one_family_pick_serves_every_row_of_the_pane() {
    let d = data();
    let mut gs = game(&d);
    let pick = Pick { family: "mace".into(), ..on_hero(&gs) };
    let fams = |id: &str| recipe::families(&d, &gs, &d.cat.recipes[id], &pick);
    assert_eq!((fams("train_minor"), fams("train_secondary")), (vec!["mace".to_string(), "dagger".to_string()], Vec::<String>::new()), "the class's two minors; one secondary needs no pick");
    assert_eq!(recipe::pick_for(&d, &gs, &d.cat.recipes["train_secondary"], &pick).family, "", "a row that does not offer the family reads it unnamed");
    let view = recipe::view(&d, &gs, SMITH, &pick);
    let row = |id: &str| view.iter().map(json::dict).find(|r| json::s(r, "id") == id).cloned().unwrap();
    assert_eq!(json::sget(&row("train_minor"), "families"), ["mace", "dagger"]);
    assert!(json::b(&row("train_minor"), "ready", false) && json::b(&row("train_secondary"), "ready", false), "both rows ready under one pick: {view:?}");
    let minor = recipe::pick_for(&d, &gs, &d.cat.recipes["train_minor"], &pick);
    ok(&d, &mut gs, SMITH, "train_minor", &minor);
    let class = d.cat.classes["drill"].clone();
    assert_eq!((hero(&gs).proficiency(&d, &class).level_of("mace"), hero(&gs).proficiency(&d, &class).level_of("dagger")), (2, 1), "the picked minor trained");
}

#[test]
fn salvage_breaks_an_item_into_gold_and_its_tier_and_theme_salvage() {
    let d = data();
    let mut gs = game(&d);
    for item in ["order_blade", "plain_blade", "letter"] {
        gs.party.add_to_stash(item, 1);
    }
    let rc = &d.cat.recipes["break_down"];
    assert!(!recipe::targets(&d, &gs, rc).contains(&"letter".to_string()), "a quest item can't be broken down");
    let item = |id: &str| Pick { item: id.into(), ..Pick::default() };
    ok(&d, &mut gs, SMITH, "break_down", &item("order_blade"));
    assert!(!gs.party.stash.contains_key("order_blade"));
    assert_eq!((gs.party.gold, gs.party.stash["fine_salvage"], gs.party.stash["order_steel"]), (GOLD + 3, 1, 1), "rarity RARE and theme order");
    ok(&d, &mut gs, SMITH, "break_down", &item("plain_blade"));
    assert_eq!((gs.party.gold, gs.party.stash["fine_salvage"]), (GOLD + 6, 1), "a common item gives the gold only");
    assert_eq!(refused(&d, &mut gs, SMITH, "break_down", &item("letter")), "Pick an item from the stash.");
    assert_eq!(refused(&d, &mut gs, SMITH, "break_down", &item("order_blade")), "Pick an item from the stash.", "none left");
}

#[test]
fn doctrine_is_reserved_and_a_weapon_recipe_waits_on_weapon_instances() {
    let d = data();
    let mut gs = game(&d);
    let pick = on_hero(&gs);
    let before = hero(&gs).to_value();
    ok(&d, &mut gs, SURGEON, "doctrine_only", &pick);
    assert_eq!((hero(&gs).to_value(), gs.party.gold), (before, GOLD - 1), "only the cost is paid");
    gs.party.add_to_stash("plain_blade", 1);
    ok(&d, &mut gs, SMITH, "break_doctrine", &Pick { item: "plain_blade".into(), ..Pick::default() });
    assert_eq!((gs.party.gold, gs.party.stash.contains_key("plain_blade")), (GOLD, false), "a doctrine beside a salvage applies nothing");
    // PK2-S4: `drill` is built; without the manifest's `durability.from_act` no mastery is gained (tests/mastery.rs)
    assert_eq!(refused(&d, &mut gs, SMITH, "drill_it", &pick), rpgkit_world::mastery::NOT_YET_TEXT);
    // PK2-FOLLOWUP: `repair` is built, but this world has no `durability.from_act`, so no weapon is an instance
    assert_eq!(refused(&d, &mut gs, SMITH, "repair_it", &pick), recipe::NO_WEAPONS_TEXT);
    let view = recipe::view(&d, &gs, SMITH, &pick);
    let row = view.iter().map(json::dict).find(|r| json::s(r, "id") == "repair_it").unwrap();
    assert!(!json::b(row, "ready", true) && json::s(row, "reason") == recipe::NO_WEAPONS_TEXT && json::sget(row, "targets").is_empty(), "the pane shows why");
}
