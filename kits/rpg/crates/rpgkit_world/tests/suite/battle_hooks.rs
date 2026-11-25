//! PR6-7 (`docs/sessions/PK6_PK7_BOSSES_ENDINGS.md` sections 1-2, the world side) on a scratch world built here (no
//! shipped data): a fight's battle-start meter grants are the `start_meter` entries of the regions its place sits in
//! (innermost first) then the encounter's own, filtered by `unless_flag` and `requires_flags` (no game state: no flag
//! is set); the ref `$captive_unit` plays the latest departed member captured staying behind (its snapshot, never a
//! hero key), else the encounter's `captive_fallback` character; a battle with neither field is built as before.

use aegis_core::json::{self, Dict};
use rpgkit_battle::session::Roster;
use rpgkit_data::{CAPTIVE_CAUSE, CAPTIVE_LIFE, CAPTIVE_UNIT};
use rpgkit_world::battle::{self, WorldRoster};
use rpgkit_world::campaign::trigger;
use rpgkit_world::state::{Departed, PartyState, UnitState};
use rpgkit_world::{explore, Data, GameState};
use serde_json::{json, Value};

const DONE: &str = "w.done";
const READY: &str = "w.ready";

fn with_ids(w: &str, rows: Value) -> Value {
    Value::Array(
        json::arr(&rows)
            .iter()
            .map(|r| {
                let mut r = json::dict(r).clone();
                let id = json::s(&r, "id");
                r.insert("_world".into(), w.into());
                r.insert("_qid".into(), format!("{w}:{id}").into());
                Value::Object(r)
            })
            .collect(),
    )
}

fn data() -> Data {
    let worlds = json!({"w": {"manifest": {"id": "w", "start_location": "hub", "start_party": ["lead"]}, "own": {
        "regions": with_ids("w", json!([
            {"id": "outer", "name": "Outer", "start_meter": [{"meter": "tension", "value": 3}]},
            {"id": "inner", "name": "Inner", "parent": "outer", "start_meter": [
                {"meter": "tension", "value": 10, "requires_track": {"id": "rot", "at_least": 1}, "unless_flag": DONE},
                {"meter": "tension", "value": 5, "requires_flags": [{"key": READY, "value": true}]}
            ]}
        ])),
        "locations": with_ids("w", json!([{"id": "hub", "name": "Hub", "region": "inner"}, {"id": "wild", "name": "Wild"}])),
        "encounters": with_ids("w", json!([
            {"id": "trial", "name": "Trial", "enemies": ["rat"], "start_meter": [{"meter": "tension", "value": 7}]},
            {"id": "far", "name": "Far", "location": "wild", "enemies": ["rat"]},
            {"id": "plain", "name": "Plain", "location": "wild", "enemies": ["rat"]},
            {"id": "crucible", "name": "Crucible", "location": "wild", "enemies": ["rat", CAPTIVE_UNIT], "captive_fallback": "stand_in"},
            {"id": "bare", "name": "Bare", "location": "wild", "enemies": ["rat", CAPTIVE_UNIT]}
        ])),
        "characters": with_ids("w", json!([
            {"id": "lead", "name": "Lead", "role": "party", "base_unit": "hero"},
            {"id": "second", "name": "Second", "role": "party", "base_unit": "hero"},
            {"id": "rat", "name": "Rat", "role": "npc", "base_unit": "rat"},
            {"id": "stand_in", "name": "Stand-in", "role": "enemy", "base_unit": "rat"}
        ]))
    }}});
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}
        ],
        "units": [
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 3},
            {"id": "rat", "name": "Rat", "side": "enemy", "class_id": "drill", "level": 1}
        ]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// Lead fielded at the hub (region `inner` in `outer`); `flags` set; `departed` records added.
fn state(d: &Data, flags: &[(&str, Value)], departed: Vec<Departed>) -> GameState {
    let members = vec![UnitState::create("lead", "hero")];
    let mut gs = GameState::create(d, "w", PartyState { members, active_ids: vec!["lead".into()], departed, ..PartyState::default() });
    explore::enter(d, &mut gs, "hub", explore::SPAWN);
    for (k, v) in flags {
        gs.set_flag(k, v.clone());
    }
    gs
}

fn entry<'a>(d: &'a Data, id: &str) -> &'a Dict {
    d.resolve("w", "encounters", id)
}

/// The value of each grant, in order.
fn grants(list: &[rpgkit_data::StartMeter]) -> Vec<i32> {
    list.iter().map(|g| g.value).collect()
}

fn gone(id: &str, life: &str, cause: &str) -> Departed {
    let mut member = UnitState::create(id, "hero");
    member.name = format!("{id} the taken");
    Departed { member, life: life.into(), cause: cause.into(), ..Departed::default() }
}

// ---- start_meter -----------------------------------------------------------------------------------------------------

#[test]
fn the_regions_innermost_first_then_the_encounter_grant() {
    let d = data();
    let gs = state(&d, &[(READY, json!(true))], Vec::new());
    assert_eq!(grants(&battle::start_meters(&d, Some(&gs), "w", entry(&d, "trial"))), [10, 5, 3, 7]);
    let first = &battle::start_meters(&d, Some(&gs), "w", entry(&d, "trial"))[0];
    assert_eq!(first.requires_track.as_ref().map(|t| (t.id.as_str(), t.at_least)), Some(("rot", 1)), "the battle reads the track gate");
    // the encounter's own location decides the region; a place in no region adds none
    assert!(battle::start_meters(&d, Some(&gs), "w", entry(&d, "far")).is_empty());
}

#[test]
fn flags_filter_the_grants() {
    let d = data();
    let gs = state(&d, &[(DONE, json!(true))], Vec::new());
    assert_eq!(grants(&battle::start_meters(&d, Some(&gs), "w", entry(&d, "trial"))), [3, 7], "unless_flag set, requires_flags unmet");
    // no game state: no flag is set
    assert_eq!(grants(&battle::start_meters(&d, None, "w", entry(&d, "trial"))), [7], "no place: the encounter's own; requires_flags never hold");
    let def = battle::encounter_def(&d, "w", entry(&d, "trial"));
    assert_eq!(grants(&def.start_meter), [7]);
    let def = battle::encounter_def_in(&d, &gs, "w", entry(&d, "trial"));
    assert_eq!(grants(&def.start_meter), [3, 7]);
}

// ---- $captive_unit ---------------------------------------------------------------------------------------------------

fn fight(enc: &str) -> Dict {
    json::dict(&json!({"source": trigger::FIGHT_SOURCE, "encounter": format!("w:{enc}")})).clone()
}

#[test]
fn the_captive_is_the_latest_member_captured_staying_behind() {
    let d = data();
    let records = vec![gone("first", CAPTIVE_LIFE, CAPTIVE_CAUSE), gone("dead_one", "dead", CAPTIVE_CAUSE), gone("latest", CAPTIVE_LIFE, CAPTIVE_CAUSE), gone("taken", CAPTIVE_LIFE, "downed")];
    let gs = state(&d, &[], records);
    let def = battle::captive_def(&d, &gs, "w").expect("a captive");
    assert_eq!(def.name, "latest the taken");
    let mut gs = gs;
    let f = battle::begin_world(&d, &mut gs, &fight("crucible")).expect("starts");
    let ids = f.session.units_by_ref.get(CAPTIVE_UNIT).cloned().unwrap_or_default();
    assert_eq!(ids.len(), 1);
    let c = ids[0];
    assert!(f.session.w.enemies.contains(&c));
    assert_eq!(f.session.w.units[c].name, "latest the taken");
    assert_eq!(f.session.w.units[c].stats.level, def.level);
}

#[test]
fn without_a_captive_the_fallback_plays_it() {
    let d = data();
    let gs = state(&d, &[], vec![gone("taken", CAPTIVE_LIFE, "downed")]);
    assert!(battle::captive_def(&d, &gs, "w").is_none(), "only a stayed capture counts");
    let mut gs = gs;
    let f = battle::begin_world(&d, &mut gs, &fight("crucible")).expect("starts");
    let c = f.session.units_by_ref[CAPTIVE_UNIT][0];
    assert_eq!(f.session.w.units[c].name, "Stand-in");
    // the host and the sim (no game state) play the fallback too
    let (_, roster) = battle::authored_fight(&d, "w", "crucible", 0).expect("resolves");
    let cat = d.cat.clone();
    assert_eq!(roster.entry(&cat, CAPTIVE_UNIT).map(|e| e.name), Some("Stand-in".to_string()));
    assert_eq!(roster.lore_ref(&cat, CAPTIVE_UNIT), "w:stand_in");
    // no fallback and no captive: the ref spawns nothing
    let mut gs = state(&d, &[], Vec::new());
    let f = battle::begin_world(&d, &mut gs, &fight("bare")).expect("starts");
    assert!(!f.session.units_by_ref.contains_key(CAPTIVE_UNIT));
    assert_eq!(f.session.w.enemies.len(), 1);
}

#[test]
fn a_snapshot_captive_is_never_a_hero_key_and_settles_without_a_note() {
    let d = data();
    let mut gs = state(&d, &[], vec![gone("latest", CAPTIVE_LIFE, CAPTIVE_CAUSE)]);
    let roster = WorldRoster { captive: battle::captive_def(&d, &gs, "w"), ..WorldRoster::new(&d, "w") };
    assert_eq!(roster.lore_ref(&d.cat, CAPTIVE_UNIT), "");
    let f = battle::begin_world(&d, &mut gs, &fight("crucible")).expect("starts");
    let tally = f.tally();
    let (_, notes) = tally.fates_of(&battle::roster(&d, "w", "w:crucible"));
    assert!(notes.is_empty(), "{notes:?}");
}

#[test]
fn a_battle_without_the_fields_is_built_as_before() {
    let d = data();
    let mut gs = state(&d, &[], Vec::new());
    let f = battle::begin_world(&d, &mut gs, &fight("plain")).expect("starts");
    assert!(f.session.def.start_meter.is_empty());
    assert_eq!(f.session.w.enemies.len(), 1);
}
