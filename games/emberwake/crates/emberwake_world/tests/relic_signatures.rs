//! `PR7-8` (`RELIC-SIG1`) on the shipped `sunk_ward` data, read only: the three Act 1 relics carry their CANON 5.1
//! signatures through the kit hooks (the hooks themselves fire in `rpgkit_battle/tests/relic_hooks.rs`), and a fielded
//! member reads the campaign act its `from_act` effects gate on.
//!
//! - `dreamers_vertebra`: a `skill_potency` group `attack` of `VERTEBRA_DAMAGE` while the wearer holds Strain
//!   (`holds_meter`), and a Strain `meter_floor` of `VERTEBRA_FLOOR` from act `SIG_ACT`;
//! - `quells_marrow_scalpel`: a hit trigger inflicting bleed in every act, and one venting `SCALPEL_VENT` Strain from
//!   act `SIG_ACT`;
//! - `grendels_slag_hammer`: a melee hit trigger carrying a `shockwave` of `HAMMER_TILES` tiles.
//!
//! Each member is picked by rule (the first party character whose build wears the relic); Strain is given as a
//! passive's `meter` effect would (no graft needed). The relic rows are the asset lane's seed
//! (`seeds/pr7_8.json`).

use emberwake_rules::Unit;
use emberwake_world::{new_game, roster, Data, GameState};
use serde_json::{json, Value};
use std::path::Path;

const WORLD: &str = "sunk_ward";
const STRAIN: &str = "strain";
/// CANON 5.1's numbers (the seed's rows carry them).
const SIG_ACT: i64 = 2;
const VERTEBRA_DAMAGE: f64 = 0.3;
const VERTEBRA_FLOOR: i32 = 20;
const SCALPEL_VENT: i64 = 10;
const HAMMER_TILES: i64 = 2;

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("shipped data").0
}

/// A `sunk_ward` game in act `act` with every party character of the world in the party.
fn game(d: &Data, act: i64) -> GameState {
    let mut gs = new_game::create_for_world(d, WORLD, 1).expect("new sunk_ward game");
    for c in d.catalogs.own_entries(WORLD, "characters").iter().filter(|c| aegis_core::json::s(c, "role") == "party") {
        let id = aegis_core::json::s(c, "id");
        if gs.party.members.iter().any(|m| m.id == id) {
            continue;
        }
        if let Some(mut m) = new_game::hero_from_character(d, WORLD, &id) {
            m.id = id;
            gs.party.members.push(m);
        }
    }
    gs.campaign.as_mut().expect("sunk_ward is a campaign world").act = act;
    gs
}

/// The first member whose build wears `item` in `slot` (a save slot key), fielded in `gs`'s act; None when no one can.
fn wearer(d: &Data, gs: &GameState, slot: &str, item: &str) -> Option<Unit> {
    gs.party.members.iter().find_map(|m| {
        let mut m = m.clone();
        m.equipment.insert(slot.into(), item.into());
        let f = roster::field_unit(d, gs, &m, WORLD)?;
        let worn = f.unit.inventory.gear().any(|e| e.id() == item);
        worn.then_some(f.unit)
    })
}

/// The same member bare of `slot`'s item (the comparison build).
fn bare(d: &Data, gs: &GameState, name: &str, slot: &str) -> Unit {
    let m = gs.party.members.iter().find(|m| roster::field_unit(d, gs, m, WORLD).is_some_and(|f| f.unit.name == name)).expect("the wearer's member").clone();
    let mut m = m;
    m.equipment.shift_remove(slot);
    roster::field_unit(d, gs, &m, WORLD).expect("fielded bare").unit
}

fn give_strain(d: &Data, u: &mut Unit) {
    u.apply_effects(&d.cat, &[json!({"type": "meter", "meter": STRAIN})], "passive:test_strain", false);
}

fn strike_potency(d: &Data, u: &Unit) -> f64 {
    u.potency_for_skill(d.cat.skill("strike").expect("the basic strike"))
}

fn description(d: &Data, item: &str) -> String {
    aegis_core::json::s(d.entry("items", item), "description").to_lowercase()
}

/// The kit plumbing on the shipped data (no seed needed): a fielded member fights in the game state's campaign act.
#[test]
fn a_fielded_member_reads_the_campaign_act() {
    let d = data();
    for act in [1, SIG_ACT] {
        let gs = game(&d, act);
        let m = &gs.party.members[0];
        let u = roster::field_unit(&d, &gs, m, WORLD).expect("fielded").unit;
        assert_eq!(i64::from(u.campaign_act), act);
    }
}

#[test]
fn dreamers_vertebra_adds_damage_while_inoculated_and_a_strain_floor_from_act_2() {
    let d = data();
    for act in [1, SIG_ACT] {
        let gs = game(&d, act);
        let mut worn = wearer(&d, &gs, "accessory_1", "dreamers_vertebra").expect("a member wears the vertebra");
        let mut plain = bare(&d, &gs, &worn.name, "accessory_1");
        assert!((strike_potency(&d, &worn) - strike_potency(&d, &plain)).abs() < 1e-9, "act {act}: no Strain, no bonus");
        give_strain(&d, &mut worn);
        give_strain(&d, &mut plain);
        assert!((strike_potency(&d, &worn) - strike_potency(&d, &plain) - VERTEBRA_DAMAGE).abs() < 1e-9, "act {act}: +30% while Inoculated");
        let floor = (act >= SIG_ACT).then_some(VERTEBRA_FLOOR);
        assert_eq!(worn.meter_floor(STRAIN), floor, "act {act}: the Strain floor");
    }
    let text = description(&d, "dreamers_vertebra");
    assert!(text.contains("30%") && text.contains("20") && text.contains("strain"), "the text names the effect: {text}");
}

#[test]
fn quells_marrow_scalpel_bleeds_on_hit_and_vents_strain_from_act_2() {
    let d = data();
    let gs = game(&d, SIG_ACT);
    let u = wearer(&d, &gs, "main_hand", "quells_marrow_scalpel").expect("a member wields the scalpel");
    let rows: Vec<&emberwake_data::TriggerRow> = u.gear_triggers.iter().map(|h| &h.row).filter(|r| r.on == "hit_dealt").collect();
    let effect = |r: &emberwake_data::TriggerRow, t: &str| r.effects.iter().find(|e| aegis_core::json::s(e, "type") == t).cloned();
    let bleed = rows.iter().find_map(|r| effect(r, "inflict").filter(|e| aegis_core::json::s(e, "status") == "bleed").map(|e| (r.when.clone(), e))).expect("a bleed on hit");
    assert!(!bleed.0.contains_key("from_act"), "the bleed holds in every act");
    assert_eq!(aegis_core::json::s(&bleed.1, "to"), "other");
    let vent = rows.iter().find_map(|r| effect(r, "meter").map(|e| (r.when.clone(), e))).expect("a Strain vent on hit");
    assert_eq!(vent.0.get("from_act"), Some(&Value::from(SIG_ACT)), "the vent waits for act 2");
    assert_eq!((aegis_core::json::s(&vent.1, "meter"), aegis_core::json::i(&vent.1, "delta", 0), aegis_core::json::s(&vent.1, "to")), (STRAIN.to_string(), -SCALPEL_VENT, "self".to_string()));
    let text = description(&d, "quells_marrow_scalpel");
    assert!(text.contains("bleed") && text.contains("10 strain"), "the text names the effect: {text}");
}

#[test]
fn grendels_slag_hammer_strikes_carry_a_line_2_shockwave() {
    let d = data();
    let gs = game(&d, 1);
    let u = wearer(&d, &gs, "main_hand", "grendels_slag_hammer").expect("a member wields the hammer (STR by rule)");
    let wave = u
        .gear_triggers
        .iter()
        .filter(|h| h.row.on == "hit_dealt" && aegis_core::json::b(&h.row.when, "melee", false))
        .find_map(|h| h.row.effects.iter().find(|e| aegis_core::json::s(e, "type") == emberwake_data::SHOCKWAVE).cloned())
        .expect("a shockwave on a melee hit");
    assert_eq!(aegis_core::json::i(&wave, "tiles", 0), HAMMER_TILES);
    assert!(aegis_core::json::i(&wave, "pct_of_hit", 0) > 0);
    let text = description(&d, "grendels_slag_hammer");
    assert!(text.contains("shockwave") && text.contains("2 tiles"), "the text names the effect: {text}");
}
