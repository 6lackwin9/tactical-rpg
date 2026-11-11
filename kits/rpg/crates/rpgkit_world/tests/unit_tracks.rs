//! PK2-S1 (`docs/sessions/PK2_GRAFTS.md` section 2, `FB-174` (5)) on a scratch world built here (no shipped data): the
//! member's unit tracks and max HP offset in the world. The saved keys are additive (written only when set, read back
//! the same; a bad stage is refused, an unknown track or a stage past the top is fixed at load with a warning); a member
//! holds every stage passive up to its stage (built and listed as its passives) and its max HP offset; `forbid_hands`
//! refuses two-handed items in `can_wear`; the settle moves each track by the battle's marks (`tracks` result key),
//! sends a now-forbidden two-hander to the stash (`stashed`), departs a member at the departure stage, and a hero still
//! hostile at the battle end, `lost` for the track's cause, before the rest of the settle; `hero_ends` pairs the members
//! with the battle-start party side, so a turned hero keeps its member. PR5-25: a Stage V member (a five-stage track)
//! turned by a breach and calmed by an ally's SUPPORT `unmark` skill settles at the stage below, without departing.

use aegis_core::json;
use rpgkit_battle::session::{Prebuilt, Session};
use rpgkit_world::battle::{self, Tally, WorldRoster};
use rpgkit_world::roster::{self, FightEnd, HeroEnd};
use rpgkit_world::state::{PartyState, UnitState, MAX_HP_OFFSET_KEY, TRACKS_KEY};
use rpgkit_world::{new_game, progression, Data, GameState};
use serde_json::{json, Value};
use rpgkit_data::NO_ITEM;

const TRACK: &str = "t_track";
const MARK: &str = "t_mark";
const OFFSET: i64 = 12;
/// The test track's departure cause (any id: the kit names none).
const CAUSE: &str = "t_cause";

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

/// One class, two founders and a foe; a three-stage track (stage two forbids two-handed items, stage three departs);
/// a two-handed staff and a one-handed sword.
fn data() -> Data {
    data_with(3)
}

/// `data` with a `top`-stage track (stages one and two as there, the last departs) and the SUPPORT skill Inoculate
/// (`unmark` of the track's mark, once a battle; PR5-25).
fn data_with(top: i32) -> Data {
    let names = ["One", "Two", "Three", "Four", "Five"];
    let stages: Vec<Value> = (1..=top)
        .map(|n| {
            let name = names[(n - 1) as usize];
            match n {
                _ if n == top => json!({"name": name, "departure": "lost"}),
                1 => json!({"name": name, "passive": "t_stage_one"}),
                2 => json!({"name": name, "passive": "t_stage_two"}),
                _ => json!({"name": name}),
            }
        })
        .collect();
    let catalog = json!({
        "classes": [
            {"id": "drill", "name": "Drill", "skill_path": {"d_one": 1}, "base_stats": {"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10}},
            {"id": "lance", "name": "Lance", "skill_path": {"d_one": 1}, "base_stats": {"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10}, "main_hand": "great_staff"}
        ],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
            {"id": "d_one", "name": "One", "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY"},
            {"id": "d_inoculate", "name": "Inoculate", "kind": "SUPPORT", "level": 1, "blocks": 1, "target_mode": "ALLY", "reach_min": 0, "reach_max": 9, "unmark": {"mark": MARK}, "uses_per_battle": 1}
        ],
        "units": [
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1},
            {"id": "lancer", "name": "Lancer", "side": "party", "class_id": "lance", "level": 1},
            {"id": "pikeman", "name": "Pikeman", "side": "party", "class_id": "drill", "level": 1, "main_hand": "great_staff"},
            {"id": "brute", "name": "Brute", "side": "enemy", "class_id": "drill", "level": 1}
        ],
        "passives": [
            {"id": "t_stage_one", "name": "Stage One", "slot": "", "effects": [{"type": "resist", "status": "bleed", "value": 5}]},
            {"id": "t_stage_two", "name": "Stage Two", "slot": "", "effects": [{"type": "forbid_hands", "hands": 2}, {"type": "pierce_melee"}]}
        ],
        "items": [
            {"id": "great_staff", "name": "Great Staff", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "staff", "two_handed": true, "base_power": 20, "value": 5},
            {"id": "plain_sword", "name": "Plain Sword", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10, "value": 5}
        ],
        "unit_tracks": [{"id": TRACK, "name": "Track", "mark": MARK, "cause": CAUSE, "stages": stages}]
    });
    let worlds = json!({"w": {"manifest": {"id": "w"}, "own": {
        "characters": with_ids("w", json!([
            {"id": "founder", "name": "Founder", "role": "party", "base_unit": "hero"},
            {"id": "second", "name": "Second", "role": "party", "base_unit": "hero"},
            {"id": "lancer", "name": "Lancer", "role": "party", "base_unit": "lancer"},
            {"id": "pikeman", "name": "Pikeman", "role": "party", "base_unit": "pikeman"},
            {"id": "foe", "name": "Foe", "role": "enemy", "base_unit": "brute"}
        ])),
        "encounters": with_ids("w", json!([{"id": "fight", "party": ["founder"], "enemies": ["foe"]}]))
    }}});
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn member(d: &Data, id: &str, stage: i64) -> UnitState {
    let mut m = new_game::hero_from_character(d, "w", id).expect("a party character");
    if stage > 0 {
        m.tracks.insert(TRACK.into(), stage);
    }
    m
}

fn game(d: &Data, members: Vec<UnitState>) -> GameState {
    let ids = members.iter().map(|m| m.id.clone()).collect();
    GameState::create(d, "w", PartyState { members, active_ids: ids, ..PartyState::default() })
}

fn end(member: &str, marks: &[(&str, i64)], hostile: &str) -> HeroEnd {
    let marks = marks.iter().map(|(m, v)| (m.to_string(), *v)).collect();
    HeroEnd { member: member.into(), hp: 10, max_hp: 10, alive: true, pouch: Vec::new(), unloaded: Vec::new(), fate: None, marks, hostile: hostile.into(), attacked: Vec::new(), worn: Vec::new() }
}

#[test]
fn the_saved_keys_are_written_only_when_set() {
    let d = data();
    let plain = member(&d, "founder", 0);
    let v = plain.to_value();
    assert!(v.get(TRACKS_KEY).is_none() && v.get(MAX_HP_OFFSET_KEY).is_none(), "a member off every track saves as before");
    let mut m = member(&d, "founder", 2);
    m.max_hp_offset = -OFFSET;
    m.tracks.insert("idle".into(), 0);
    let v = m.to_value();
    assert_eq!((&v[TRACKS_KEY], &v[MAX_HP_OFFSET_KEY]), (&json!({TRACK: 2}), &json!(-OFFSET)), "a 0 stage is not saved");
    let back = UnitState::from_dict(json::dict(&v));
    assert_eq!((back.track_stage(TRACK), back.track_stage("idle"), back.max_hp_offset), (2, 0, -OFFSET));
    assert_eq!(back.to_value(), v, "read back the same");
    let mut bad = member(&d, "founder", -1);
    bad.tracks.insert(TRACK.into(), -1);
    assert!(bad.validate(&d).iter().any(|e| e == "Unit 'founder': track 't_track' stage must be 0 or more."), "{:?}", bad.validate(&d));
}

#[test]
fn a_loaded_track_the_data_no_longer_allows_is_fixed_with_a_warning() {
    let d = data();
    let mut m = member(&d, "founder", 9);
    m.tracks.insert("gone".into(), 1);
    let warnings = m.sanitize_tracks(&d);
    assert_eq!(
        warnings,
        ["Unit 'founder': unit track 't_track' stage 9 is held at its top stage 3.", "Unit 'founder': unit track 'gone' is gone; its stage 1 is dropped."]
    );
    assert_eq!((m.track_stage(TRACK), m.track_stage("gone")), (3, 0));
    assert!(m.sanitize_tracks(&d).is_empty(), "nothing left to fix");
}

/// A member holds every stage passive up to its stage (gathered after its other passives, slotless), so its build has
/// them and their grants; the max HP offset is added to its built max.
#[test]
fn a_member_holds_its_stage_passives_and_offset() {
    let d = data();
    let base = progression::max_hp_of(&d, &member(&d, "founder", 0), "w");
    for (stage, passives) in [(0, vec![]), (1, vec!["t_stage_one"]), (2, vec!["t_stage_one", "t_stage_two"])] {
        let m = member(&d, "founder", stage);
        assert_eq!(progression::track_passives(&d, &m), passives, "stage {stage}");
        assert!(passives.iter().all(|p| progression::member_passives(&d, &m).contains(&p.to_string())));
        let u = progression::build_played(&d, &m, "w").expect("builds").unit;
        assert!(passives.iter().all(|p| u.passives.contains(&p.to_string())), "{:?}", u.passives);
        assert_eq!((u.forbid_hands, u.pierce_melee), if stage >= 2 { (2, true) } else { (0, false) }, "stage {stage}");
        assert_eq!(u.tracks.as_ref().and_then(|t| t.get(TRACK).copied()), (stage > 0).then_some(stage as i32), "the battle unit holds its stage");
    }
    let mut m = member(&d, "founder", 0);
    m.max_hp_offset = OFFSET;
    assert_eq!(progression::max_hp_of(&d, &m, "w"), base + OFFSET);
    let f = roster::field_unit(&d, &game(&d, vec![m.clone()]), &m, "w").unwrap();
    assert_eq!((f.unit.max_hp as i64, f.unit.hp as i64), (base + OFFSET, base + OFFSET), "fielded at full with the offset");
}

#[test]
fn forbid_hands_refuses_a_two_hander_in_can_wear() {
    let d = data();
    let free = member(&d, "founder", 1);
    assert_eq!(free.can_wear(&d, "great_staff", "main_hand"), Ok(()));
    let shaken = member(&d, "founder", 2);
    assert_eq!(shaken.can_wear(&d, "great_staff", "main_hand"), Err("Great Staff is two-handed: Founder can't wield two-handed items now.".to_string()));
    assert_eq!(shaken.can_wear(&d, "plain_sword", "main_hand"), Ok(()));
}

/// The settle moves each track by the member's marks (summed; another id moves nothing), lists the moves under
/// `tracks`, and the stage that forbids two-handed items sends the worn staff to the stash (`stashed`); a fight
/// without marks adds neither key.
#[test]
fn the_settle_moves_the_tracks_by_the_marks() {
    let d = data();
    let mut founder = member(&d, "founder", 0);
    founder.equipment.insert("main_hand".into(), "great_staff".into());
    let second = member(&d, "second", 0);
    let mut gs = game(&d, vec![founder, second]);
    let ends = vec![end("founder", &[(MARK, 1), ("other", 3), (MARK, 1)], ""), end("second", &[(MARK, 1)], "")];
    let tally = Tally { party: ends, world: "w".into(), ..Tally::default() };
    let res = battle::settle(&d, &mut gs, "w:fight", "won", Some(&tally));
    assert_eq!(res["tracks"], json!([{"member": "founder", "track": TRACK, "from": 0, "to": 2}, {"member": "second", "track": TRACK, "from": 0, "to": 1}]));
    assert_eq!(res["stashed"], json!([{"member": "founder", "item": "great_staff"}]));
    let f = gs.party.member("founder").unwrap();
    assert_eq!((f.track_stage(TRACK), f.equipment.get("main_hand")), (2, None));
    assert_eq!(gs.party.stash.get("great_staff").copied(), Some(1));
    assert!(gs.party.members.iter().all(|m| m.validate(&d).is_empty()), "the settled party is valid");
    let tally = Tally { party: vec![end("founder", &[], "")], world: "w".into(), ..Tally::default() };
    let res = battle::settle(&d, &mut gs, "w:fight", "won", Some(&tally));
    assert!(res.get("tracks").is_none() && res.get("stashed").is_none());
    assert_eq!(gs.party.member("founder").unwrap().track_stage(TRACK), 2);
}

/// PR5-26 (audit B2): a member whose main hand is left to a two-handed default (its class's, its base unit's) holds
/// `NO_ITEM` there once its stage forbids two-handed items, so its build leaves the hand empty instead of equipping the
/// default again; the default is no stash item (nothing stashed). A worn two-hander over such a default goes to the
/// stash and the hand is left empty too. The settled member stays valid and saves and loads its empty slot; the
/// result lists the emptied slot (`cleared`, review F2).
#[test]
fn a_forbidden_two_handed_default_leaves_the_hand_empty() {
    let d = data();
    let main = |m: &UnitState| progression::build_played(&d, m, "w").expect("builds").unit.inventory.main_weapon().map(|e| e.item.id.clone());
    for (who, worn) in [("lancer", false), ("pikeman", false), ("lancer", true)] {
        let mut m = member(&d, who, 1);
        if worn {
            m.equipment.insert("main_hand".into(), "great_staff".into());
        }
        assert_eq!(main(&m).as_deref(), Some("great_staff"), "{who}: the two-hander before");
        let mut gs = game(&d, vec![m]);
        let tally = Tally { party: vec![end(who, &[(MARK, 1)], "")], world: "w".into(), ..Tally::default() };
        let res = battle::settle(&d, &mut gs, "w:fight", "won", Some(&tally));
        let m = gs.party.member(who).unwrap().clone();
        assert_eq!(m.track_stage(TRACK), 2, "{who}: the stage that forbids");
        assert_eq!(m.equipment.get("main_hand").map(String::as_str), Some(NO_ITEM), "{who} (worn {worn})");
        assert_eq!(main(&m), None, "{who}: the build leaves the hand empty");
        let stashed = if worn { json!([{"member": who, "item": "great_staff"}]) } else { Value::Null };
        assert_eq!(res.get("stashed").cloned().unwrap_or(Value::Null), stashed, "{who} (worn {worn})");
        // review F2: the result names the emptied slot
        assert_eq!(res["cleared"], json!([{"member": who, "slot": "main_hand"}]), "{who} (worn {worn})");
        assert_eq!(gs.party.stash.get("great_staff").copied(), worn.then_some(1));
        assert!(m.validate(&d).is_empty(), "{:?}", m.validate(&d));
        let back = UnitState::from_dict(json::dict(&m.to_value()));
        assert_eq!(back.equipment.get("main_hand").map(String::as_str), Some(NO_ITEM), "saved and loaded");
    }
    // a one-handed default stays: only the forbidden default is emptied
    let mut sworded = member(&d, "founder", 1);
    sworded.equipment.insert("main_hand".into(), "great_staff".into());
    let mut gs = game(&d, vec![sworded]);
    let tally = Tally { party: vec![end("founder", &[(MARK, 1)], "")], world: "w".into(), ..Tally::default() };
    let res = battle::settle(&d, &mut gs, "w:fight", "won", Some(&tally));
    assert_eq!(gs.party.member("founder").unwrap().equipment.get("main_hand"), None, "the class default (none) comes back");
    assert!(res.get("cleared").is_none(), "no slot emptied");
}

/// The departure stage departs the member `lost` for the track's cause; a hero still hostile at the battle end departs
/// so too though its marks do not reach the stage; the others settle as before.
#[test]
fn the_departure_stage_and_a_hostile_end_depart_the_member() {
    let d = data();
    let mut gs = game(&d, vec![member(&d, "founder", 2), member(&d, "second", 1)]);
    let ends = vec![end("founder", &[(MARK, 1)], ""), end("second", &[], TRACK)];
    let settled = roster::settle_party(&d, &mut gs, &FightEnd::new("won", "w"), &ends);
    assert_eq!(settled.departed, vec![("founder".to_string(), "lost".to_string()), ("second".to_string(), "lost".to_string())]);
    assert!(gs.party.members.is_empty());
    let causes: Vec<(&str, &str)> = gs.party.departed.iter().map(|x| (x.member.id.as_str(), x.cause.as_str())).collect();
    assert_eq!(causes, [("founder", CAUSE), ("second", CAUSE)]);
    let departed_errors = |p: &PartyState| p.validate(&d).into_iter().filter(|e| e.starts_with("Departed")).collect::<Vec<_>>();
    assert!(departed_errors(&gs.party).is_empty(), "the track's cause is a known cause: {:?}", departed_errors(&gs.party));
    let mut bad = gs.party.clone();
    bad.departed[0].cause = "t_other".into();
    assert!(bad.validate(&d).iter().any(|e| e.starts_with("Departed 'founder': unknown cause 't_other' (known: ") && e.ends_with(&format!(", {CAUSE})."))), "{:?}", bad.validate(&d));
    let mut other = game(&d, vec![member(&d, "founder", 0)]);
    assert!(roster::depart(&d, &mut other, "founder", "lost", "t_other", "").is_err(), "an unknown cause is refused");
    assert!(roster::depart(&d, &mut other, "founder", "lost", CAUSE, "").is_ok(), "the track's cause departs");
    assert_eq!(gs.party.departed[0].member.track_stage(TRACK), 3, "the record keeps the stage it left at");
}

/// `hero_ends` pairs the members with the battle-start party side: a hero the departure stage turned on everyone (no
/// longer an ally) keeps its own member and carries its hostility; its partner keeps its own.
#[test]
fn hero_ends_pair_a_turned_hero_with_its_member() {
    let d = data();
    let members = vec![member(&d, "founder", 2), member(&d, "second", 0)];
    let gs = game(&d, members.clone());
    let mut party = Vec::new();
    let mut fielded = Vec::new();
    for m in &members {
        let f = roster::field_unit(&d, &gs, m, "w").unwrap();
        party.push(Prebuilt { unit: f.unit, art: String::new(), palette: String::new(), profile: f.profile, key: m.id.clone(), default_intent: f.default_intent });
        fielded.push((m.id.clone(), f.pouch_unloaded));
    }
    let def = battle::encounter_def(&d, "w", d.resolve("w", "encounters", "fight"));
    let mut s = Session::start_with(d.cat.clone(), def, &WorldRoster::new(&d, "w"), party);
    let (founder, second) = (s.w.allies[0], s.w.allies[1]);
    s.w.marks.record(founder, MARK, 1);
    let parts = rpgkit_battle::mark::sync_tracks(&mut s.w, founder, MARK);
    assert!(parts.iter().any(|p| p.ends_with("turns on everyone")), "{parts:?}");
    assert_eq!(s.w.allies, vec![second], "the founder left the allies");
    let ends = roster::hero_ends(&s, &fielded);
    let paired: Vec<(&str, &str)> = ends.iter().map(|e| (e.member.as_str(), e.hostile.as_str())).collect();
    assert_eq!(paired, [("founder", TRACK), ("second", "")]);
    assert_eq!(ends[0].marks, vec![(MARK.to_string(), 1)]);
}

/// PR5-25 (`FB-187` (5)): on a five-stage track, a breach takes the founder from Stage IV to Stage V (turned on
/// everyone); the second member's planned SUPPORT `unmark` skill reaches it, takes the breach back (Stage IV again) and
/// calms it at the next round start; `hero_ends` carries no hostility and no mark, so the settle keeps the founder at
/// Stage IV (no departure).
#[test]
fn a_stage_v_member_calmed_by_an_unmark_skill_settles_without_departing() {
    let top = 5;
    let d = data_with(top);
    let members = vec![member(&d, "founder", i64::from(top - 1)), member(&d, "second", 0)];
    let mut gs = game(&d, members.clone());
    let mut party = Vec::new();
    let mut fielded = Vec::new();
    for m in &members {
        let f = roster::field_unit(&d, &gs, m, "w").unwrap();
        party.push(Prebuilt { unit: f.unit, art: String::new(), palette: String::new(), profile: f.profile, key: m.id.clone(), default_intent: f.default_intent });
        fielded.push((m.id.clone(), f.pouch_unloaded));
    }
    let def = battle::encounter_def(&d, "w", d.resolve("w", "encounters", "fight"));
    let mut s = Session::start_with(d.cat.clone(), def, &WorldRoster::new(&d, "w"), party);
    let (founder, second) = (s.w.allies[0], s.w.allies[1]);
    // the keystone as the second member's (its family active, as a deck's keystone slot holds it)
    s.w.units[second].known_skills.push("d_inoculate".into());
    s.w.units[second].active_extra_families.push(d.cat.family_of("d_inoculate").to_string());
    s.w.marks.record(founder, MARK, 1);
    let parts = rpgkit_battle::mark::sync_tracks(&mut s.w, founder, MARK);
    assert!(parts.iter().any(|p| p == "Founder's Track is at stage 5 (Five)") && parts.iter().any(|p| p.ends_with("turns on everyone")), "{parts:?}");
    let ino = s.w.book().find("d_inoculate").unwrap();
    assert_eq!(s.plan.add_step(&s.w, second, rpgkit_battle::plan::PlanStep::ally(ino, founder), true), "", "the unmark reaches the turned founder");
    assert!(s.confirm_plan());
    while s.phase != rpgkit_battle::session::Phase::Replay && !s.combat_over {
        s.step_next();
    }
    assert!(s.resolver.log.iter().any(|l| l.ends_with("Founder calms at the next round start")), "{:?}", s.resolver.log);
    assert!(s.can_advance_round());
    s.advance_round();
    assert_eq!(s.w.allies, vec![founder, second], "calmed, back at its start place");
    let ends = roster::hero_ends(&s, &fielded);
    assert!(ends.iter().all(|e| e.hostile.is_empty() && e.marks.is_empty()), "{ends:?}");
    let settled = roster::settle_party(&d, &mut gs, &FightEnd::new("won", "w"), &ends);
    assert!(settled.departed.is_empty(), "{:?}", settled.departed);
    assert_eq!(gs.party.member("founder").map(|m| m.track_stage(TRACK)), Some(i64::from(top - 1)), "Stage IV kept");
}
