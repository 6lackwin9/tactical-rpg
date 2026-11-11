//! PK2-S2 (`docs/sessions/PK2_GRAFTS.md` section 3, FB-167 Q6) on a scratch world built here (no shipped data): the
//! member's body slots in the world. The saved keys `body`, `class_rite` and `floor_offset` are additive (written only
//! when set, read back the same); `install_graft` refuses a misfit (an unknown slot or item, not a graft, the slot's
//! role, a class slot before the class rite, another class's piece), a second keystone graft and a top-tier graft
//! before 3 grafts (an overwrite counts its slot), and an overwrite returns the destroyed graft; `remove_graft` empties
//! a slot; `validate` reports a bad body and the load drops it with a warning; a member's build holds its body (the
//! graft's skill owned, the floor by the graft count plus the offset); the deck's channeled marks are one per graft; the
//! settle clears the floor offset and a member who falls `dead` while the party holds the field leaves its character's
//! `remains` (else its class's) in the stash (`remains` result key), a captured one or a lost fight nothing.

use aegis_core::json;
use rpgkit_battle::encounter::PartyFate;
use rpgkit_world::battle::{self, Tally};
use rpgkit_world::roster::{self, FightEnd, HeroEnd};
use rpgkit_world::state::{PartyState, UnitState, BODY_KEY, CLASS_RITE_KEY, FLOOR_OFFSET_KEY};
use rpgkit_world::{new_game, progression, Data, GameState};
use serde_json::{json, Value};

/// The floor table every test graft's meter carries (by graft count 1, 2, 3, 4).
const FLOORS: [i64; 4] = [5, 15, 30, 50];
const OFFSET: i64 = -10;

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

fn graft(id: &str, skill: &str, tier: i64, role: &str, extra: Value) -> Value {
    let mut row = json!({"id": id, "name": id, "item_type": "GRAFT", "tier": tier, "body_role": role,
        "body_effects": [{"type": "skill", "skill": skill}, {"type": "meter", "meter": "t_gauge", "floors": FLOORS}]});
    for (k, v) in extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    row
}

/// One class (with `remains`), a second class without; characters with and without their own `remains`; five body slots
/// (two class slots); monster grafts of tiers 1 and 3, a class piece, two keystone pieces (tier 3) and another class's piece.
fn data() -> Data {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10});
    let attack = |id: &str| json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY", "channel": {"only": true}});
    let catalog = json!({
        "classes": [
            {"id": "drill", "name": "Drill", "skill_path": {"d_one": 1, "d_two": 1, "d_three": 1}, "base_stats": stats, "remains": "core_class"},
            {"id": "other", "name": "Other", "skill_path": {"d_one": 1}, "base_stats": stats}
        ],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
            {"id": "d_one", "name": "One", "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY"},
            {"id": "d_two", "name": "Two", "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY"},
            {"id": "d_three", "name": "Three", "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY"},
            attack("m_claw"), attack("m_bite"), attack("m_sting"), attack("m_top"), attack("c_rite"), attack("c_crown"), attack("c_alien")
        ],
        "units": [
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1},
            {"id": "stranger", "name": "Stranger", "side": "party", "class_id": "other", "level": 1},
            {"id": "brute", "name": "Brute", "side": "enemy", "class_id": "drill", "level": 1}
        ],
        "items": [
            graft("g_claw", "m_claw", 1, "monster", json!({})),
            graft("g_bite", "m_bite", 1, "monster", json!({})),
            graft("g_sting", "m_sting", 1, "monster", json!({})),
            graft("g_top", "m_top", 3, "monster", json!({})),
            graft("g_rite", "c_rite", 1, "class", json!({"class_id": "drill"})),
            graft("g_crown", "c_crown", 3, "class", json!({"keystone": true})),
            graft("g_crown2", "c_crown", 3, "class", json!({"keystone": true})),
            graft("g_soft_key", "c_crown", 1, "class", json!({"keystone": true})),
            graft("g_alien", "c_alien", 1, "class", json!({"class_id": "other"})),
            {"id": "core_class", "name": "Class Core", "item_type": "MATERIAL", "value": 0},
            {"id": "core_own", "name": "Named Core", "item_type": "MATERIAL", "value": 0},
            {"id": "plain_sword", "name": "Plain Sword", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10, "value": 5}
        ],
        "unit_meters": [{"id": "t_gauge", "name": "Gauge", "short": "Gauge", "min": 0, "max": 100, "start": 0, "show": "bar"}],
        "body_slots": [
            {"id": "s_class", "name": "Class", "role": "class"},
            {"id": "s_class2", "name": "Class Two", "role": "class"},
            {"id": "s_a", "name": "A", "role": "monster"},
            {"id": "s_b", "name": "B", "role": "monster"},
            {"id": "s_c", "name": "C", "role": "monster"}
        ]
    });
    let worlds = json!({"w": {"manifest": {"id": "w"}, "own": {
        "characters": with_ids("w", json!([
            {"id": "named", "name": "Named", "role": "party", "base_unit": "hero", "remains": "core_own"},
            {"id": "recruit", "name": "Recruit", "role": "party", "base_unit": "hero"},
            {"id": "stranger", "name": "Stranger", "role": "party", "base_unit": "stranger"},
            {"id": "foe", "name": "Foe", "role": "enemy", "base_unit": "brute"}
        ])),
        "encounters": with_ids("w", json!([{"id": "fight", "party": ["named"], "enemies": ["foe"]}]))
    }}});
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn member(d: &Data, id: &str) -> UnitState {
    new_game::hero_from_character(d, "w", id).expect("a party character")
}

fn grafted(d: &Data, id: &str, body: &[(&str, &str)]) -> UnitState {
    let mut m = member(d, id);
    for (slot, item) in body {
        m.body.insert(slot.to_string(), item.to_string());
    }
    m
}

fn game(d: &Data, members: Vec<UnitState>) -> GameState {
    let ids = members.iter().map(|m| m.id.clone()).collect();
    GameState::create(d, "w", PartyState { members, active_ids: ids, ..PartyState::default() })
}

#[test]
fn the_saved_keys_are_written_only_when_set() {
    let d = data();
    let v = member(&d, "named").to_value();
    assert!(v.get(BODY_KEY).is_none() && v.get(CLASS_RITE_KEY).is_none() && v.get(FLOOR_OFFSET_KEY).is_none(), "a member without a body saves as before");
    let mut m = grafted(&d, "named", &[("s_a", "g_claw"), ("s_class", "g_rite")]);
    m.set_class_rite(true);
    m.floor_offset = OFFSET;
    let v = m.to_value();
    assert_eq!((&v[BODY_KEY], &v[CLASS_RITE_KEY], &v[FLOOR_OFFSET_KEY]), (&json!({"s_a": "g_claw", "s_class": "g_rite"}), &json!(true), &json!(OFFSET)));
    let back = UnitState::from_dict(json::dict(&v));
    assert_eq!((back.graft_count(), back.class_rite, back.floor_offset), (2, true, OFFSET));
    assert_eq!(back.to_value(), v, "read back the same");
    assert!(back.validate(&d).is_empty(), "{:?}", back.validate(&d));
}

#[test]
fn install_graft_refuses_a_misfit() {
    let d = data();
    let mut m = member(&d, "named");
    let refuse = |m: &mut UnitState, slot: &str, item: &str| m.install_graft(&d, slot, item).expect_err("refused");
    assert_eq!(refuse(&mut m, "s_nope", "g_claw"), "unknown body slot 's_nope'.");
    assert_eq!(refuse(&mut m, "s_a", "g_nope"), "unknown item 'g_nope' in body slot s_a.");
    assert_eq!(refuse(&mut m, "s_a", "plain_sword"), "Plain Sword in body slot s_a is not a graft.");
    assert_eq!(refuse(&mut m, "s_a", "g_rite"), "body slot s_a takes monster grafts; g_rite is a class graft.");
    assert_eq!(refuse(&mut m, "s_class", "g_claw"), "body slot s_class takes class grafts; g_claw is a monster graft.");
    assert_eq!(refuse(&mut m, "s_class", "g_rite"), "body slot s_class opens with the member's class rite.");
    m.set_class_rite(true);
    assert_eq!(refuse(&mut m, "s_class", "g_alien"), "g_alien is a piece of class 'other'.");
    assert_eq!(m.install_graft(&d, "s_class", "g_rite"), Ok(None), "the rite opens the class slot");
    assert!(m.body.len() == 1 && m.validate(&d).is_empty());
}

/// A top-tier graft needs 3 grafts first; an overwrite counts the slot it fills and returns the destroyed graft (the
/// count stays); the keystone graft is one per member (a keystone over the keystone is an overwrite).
#[test]
fn the_top_tier_gate_and_the_keystone_are_kept() {
    let d = data();
    let mut m = member(&d, "named");
    m.set_class_rite(true);
    let gate = |n: usize| format!("a tier 3 graft needs 3 grafts first (the member holds {n}).");
    assert_eq!(m.install_graft(&d, "s_a", "g_top"), Err(gate(0)));
    assert_eq!(m.install_graft(&d, "s_a", "g_claw"), Ok(None));
    assert_eq!(m.install_graft(&d, "s_b", "g_bite"), Ok(None));
    assert_eq!(m.install_graft(&d, "s_c", "g_top"), Err(gate(2)));
    assert_eq!(m.install_graft(&d, "s_class", "g_crown"), Err(gate(2)), "the keystone graft is a tier 3 graft too");
    assert_eq!(m.install_graft(&d, "s_class", "g_soft_key"), Err(gate(2)), "a hand-built tier 1 keystone is gated as top tier");
    assert_eq!(m.install_graft(&d, "s_c", "g_sting"), Ok(None));
    assert_eq!(m.install_graft(&d, "s_c", "g_top"), Ok(Some("g_sting".into())), "an overwrite: 3 held, the old graft destroyed");
    assert_eq!(m.graft_count(), 3);
    assert_eq!(m.install_graft(&d, "s_class", "g_crown"), Ok(None), "the keystone as the 4th graft");
    assert_eq!(m.install_graft(&d, "s_class", "g_crown2"), Ok(Some("g_crown".into())), "a keystone over the keystone");
    assert_eq!(m.graft_count(), 4);
    assert_eq!(m.remove_graft("s_class"), Some("g_crown2".into()));
    assert_eq!((m.remove_graft("s_class"), m.graft_count()), (None, 3));
    // a second keystone in another class slot is refused; a hand-made one is a validation error and the load keeps
    // the first
    m.install_graft(&d, "s_class", "g_crown2").expect("a keystone again");
    assert_eq!(m.install_graft(&d, "s_class2", "g_crown"), Err("the member already holds a keystone graft (one per member).".into()));
    let mut two = grafted(&d, "named", &[("s_class", "g_crown2"), ("s_class2", "g_crown")]);
    two.set_class_rite(true);
    assert_eq!(two.validate(&d), ["Unit 'named': body holds 2 keystone grafts (one per member)."]);
    assert_eq!(two.sanitize_body(&d), ["Unit 'named': a second keystone graft in body slot s_class2. The graft is dropped."]);
    assert_eq!((two.graft_count(), two.validate(&d).len()), (1, 0));
}

#[test]
fn a_bad_body_is_refused_and_dropped_at_load() {
    let d = data();
    let mut m = grafted(&d, "named", &[("s_a", "g_claw"), ("s_class", "g_rite"), ("s_gone", "g_bite")]);
    let errors = m.validate(&d);
    assert_eq!(
        errors,
        ["Unit 'named': body slot s_class opens with the member's class rite.", "Unit 'named': unknown body slot 's_gone'."],
        "no class rite yet"
    );
    assert!(progression::build_played(&d, &m, "w").is_none(), "an invalid member does not build");
    let warnings = m.sanitize_body(&d);
    assert_eq!(
        warnings,
        [
            "Unit 'named': body slot s_class opens with the member's class rite. The graft is dropped.",
            "Unit 'named': unknown body slot 's_gone'. The graft is dropped."
        ]
    );
    assert_eq!((m.graft_count(), m.validate(&d).len()), (1, 0));
    assert!(m.sanitize_body(&d).is_empty(), "nothing left to fix");
}

/// The build holds the body: each graft's skill owned, the meter at the floor of the graft count plus the offset; the
/// deck's channeled marks are one per graft.
#[test]
fn a_member_fights_with_its_body() {
    let d = data();
    for n in 1..=3 {
        let body = [("s_a", "g_claw"), ("s_b", "g_bite"), ("s_c", "g_sting")];
        let mut m = grafted(&d, "named", &body[..n]);
        let u = progression::build_played(&d, &m, "w").expect("builds").unit;
        assert_eq!(u.graft_count, Some(n as i32));
        let floors: Vec<Option<i32>> = u.passive_meters.iter().map(|h| h.floor).collect();
        assert_eq!(floors, vec![Some(FLOORS[n - 1] as i32); n], "{n} grafts");
        assert!(["m_claw", "m_bite", "m_sting"][..n].iter().all(|s| u.owns_skill(&d.cat, s)));
        m.floor_offset = OFFSET;
        let u = progression::build_played(&d, &m, "w").expect("builds").unit;
        assert_eq!(u.passive_meters[0].floor, Some((FLOORS[n - 1] + OFFSET) as i32), "{n} grafts and the offset");
        assert!(m.owned_families_anywhere(&d).contains(&"m_claw".to_string()), "a graft's skill may sit in the deck");
    }
    let mut m = grafted(&d, "named", &[("s_a", "g_claw")]);
    m.deck.active = vec!["d_one".into(), "d_two".into()];
    m.deck.channeled = vec!["d_one".into(), "d_two".into()];
    assert!(m.validate(&d).iter().any(|e| e == "Unit 'named': deck: 2 channeled entries (one per graft; the member holds 1)."), "{:?}", m.validate(&d));
    m.body.insert("s_b".into(), "g_bite".into());
    assert!(m.validate(&d).is_empty(), "{:?}", m.validate(&d));
}

fn end(member: &str, fate: PartyFate) -> HeroEnd {
    HeroEnd { member: member.into(), hp: 0, max_hp: 10, alive: false, pouch: Vec::new(), unloaded: Vec::new(), fate: Some(fate), marks: Vec::new(), hostile: String::new() }
}

/// A stayer the encounter's `then` kills departs `dead`: with the field held its character's `remains` (else its
/// class's) goes to the stash; a class without one leaves nothing; a capture or a fight the party does not hold leaves
/// nothing either. The installed grafts go with the member (no graft in the stash).
#[test]
fn a_fallen_member_leaves_its_remains_when_the_field_is_held() {
    let d = data();
    let party = || vec![grafted(&d, "named", &[("s_a", "g_claw")]), member(&d, "recruit"), member(&d, "stranger")];
    let ends = [end("named", PartyFate::Stayed), end("recruit", PartyFate::Stayed), end("stranger", PartyFate::Stayed)];
    let fight = |outcome: &str, then: &str| FightEnd { outcome: outcome.into(), world: "w".into(), then: then.into(), ..FightEnd::default() };
    let mut gs = game(&d, party());
    let out = roster::settle_party(&d, &mut gs, &fight("won", "kill"), &ends);
    assert_eq!(out.remains, [("named".to_string(), "core_own".to_string()), ("recruit".to_string(), "core_class".to_string())]);
    assert_eq!((gs.party.stash.get("core_own").copied(), gs.party.stash.get("core_class").copied()), (Some(1), Some(1)));
    assert!(gs.party.stash.get("g_claw").is_none(), "the installed graft is lost with the member");
    for (outcome, then) in [("won", "capture"), ("lost", "kill"), ("fled", "kill")] {
        let mut gs = game(&d, party());
        let out = roster::settle_party(&d, &mut gs, &fight(outcome, then), &ends);
        assert_eq!(out.departed.len(), 3, "{outcome} {then}: all departed");
        assert!(out.remains.is_empty() && gs.party.stash.is_empty(), "{outcome} {then}: nothing left");
    }
}

/// The settle clears the floor offset of every fielded member (the next battle only) and lists `remains` only when
/// some member left one.
#[test]
fn the_settle_clears_the_floor_offset() {
    let d = data();
    let mut m = grafted(&d, "named", &[("s_a", "g_claw")]);
    m.floor_offset = OFFSET;
    let mut bench = member(&d, "recruit");
    bench.floor_offset = OFFSET;
    let mut gs = game(&d, vec![m, bench]);
    let alive = HeroEnd { member: "named".into(), hp: 10, max_hp: 10, alive: true, pouch: Vec::new(), unloaded: Vec::new(), fate: None, marks: Vec::new(), hostile: String::new() };
    let tally = Tally { party: vec![alive], world: "w".into(), ..Tally::default() };
    let res = battle::settle(&d, &mut gs, "w:fight", "won", Some(&tally));
    assert!(res.get("remains").is_none());
    assert_eq!(gs.party.member("named").unwrap().floor_offset, 0, "used up by the battle");
    assert_eq!(gs.party.member("recruit").unwrap().floor_offset, OFFSET, "a member who did not fight keeps it");
    assert!(battle::PARTY_KEYS.contains(&"remains"));
}
