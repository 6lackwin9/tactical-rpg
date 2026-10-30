//! ROSTER1-L on scratch worlds (no shipped data): the roster's battle calls. Slice 2 moved the battle screen's hero
//! build and HP write-back into `rpgkit_world::roster` with no rule change: `field_unit` is exactly the old build
//! (`progression::build_in` in the battle's world under that world's deck rule, at `start_hp`, the base unit's AI
//! profile), so it follows the played world's class path and deck rule (decision "HOOKS1-UI slice 1 answers");
//! `settle_party` is ITEMUSE1 rule 7's pouch write-back on every outcome and HEAL1's `hp_after` on won and fled;
//! `hero_ends` pairs the fielded members with the session's allies and carries ENCOUNTER2 S3's fates.

use rpgkit_battle::encounter::PartyFate;
use rpgkit_battle::session::{Prebuilt, Session};
use rpgkit_battle::world::AiProfile;
use rpgkit_rules::DeckRule;
use rpgkit_world::battle::{self, WorldRoster};
use rpgkit_world::roster::{self, HeroEnd};
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{new_game, progression, Data, GameState};
use serde_json::{json, Value};

/// The class path (every family unlocks at level 1); the overlay world `ow` keeps only the second.
const PATH: [&str; 3] = ["d_one", "d_two", "d_three"];

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

/// A world's two founders, a party-side guest, a foe and the fight between them.
fn own(w: &str) -> Value {
    json!({
        "characters": with_ids(w, json!([
            {"id": "founder", "name": "Founder", "role": "party", "base_unit": "hero"},
            {"id": "second", "name": "Second", "role": "party", "base_unit": "hero"},
            {"id": "guide", "name": "Guide", "role": "npc", "base_unit": "hero"},
            {"id": "foe", "name": "Foe", "role": "enemy", "base_unit": "brute"}
        ])),
        "encounters": with_ids(w, json!([{"id": "fight", "party": ["founder"], "enemies": ["foe"], "guests": ["guide"]}]))
    })
}

/// A classic world `w`, a timeline world `tw` with the deck rule, and `ow` whose class overlay replaces the path.
fn data() -> Data {
    let atk = |id: &str| json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY"});
    let path: serde_json::Map<String, Value> = PATH.iter().map(|f| (f.to_string(), json!(1))).collect();
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "skill_path": path, "base_stats": {"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
            {"id": "active_guard", "name": "Active Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2},
            atk(PATH[0]), atk(PATH[1]), atk(PATH[2])
        ],
        "units": [
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1, "role": "SUPPORT", "defend_below": 0.45},
            {"id": "brute", "name": "Brute", "side": "enemy", "class_id": "drill", "level": 1}
        ],
        "rulesets": [{"id": "timeline", "name": "Timeline", "rules": ["round_intent", "deck"], "basics": ["move", "rest", "strike", "active_guard"],
            "deck_slots": {"active": 1, "passive": 1, "keystone": 1}}]
    });
    let worlds = json!({
        "w": {"manifest": {"id": "w"}, "own": own("w")},
        "tw": {"manifest": {"id": "tw", "ruleset": "timeline"}, "own": own("tw")},
        "ow": {"manifest": {"id": "ow", "classes": {"drill": {"name": "Bash-Hand", "skill_path": {PATH[1]: 1}}}}, "own": own("ow")}
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn has(ids: &[String], id: &str) -> bool {
    ids.iter().any(|s| s == id)
}

/// The battle screen's hero build before ROSTER1-L (gui battle.rs `BattleScreen::world`), kept here as the reference.
fn old_build(d: &Data, m: &UnitState, world: &str) -> (rpgkit_rules::Unit, Vec<String>, AiProfile) {
    let deck = DeckRule::of(&d.ruleset(world));
    let built = progression::build_in(d, m, world, deck.as_ref()).expect("valid");
    let mut unit = built.unit;
    unit.hp = progression::start_hp(unit.max_hp as i64, m.hp_lost) as i32;
    (unit, built.pouch_unloaded, AiProfile::for_unit(&d.cat, &d.cat.units["hero"]))
}

#[test]
fn field_unit_is_the_old_battle_build_in_the_played_world() {
    let d = data();
    for world in ["w", "tw", "ow"] {
        for hp_lost in [0, 7, 10_000] {
            let mut m = new_game::hero_from_character(&d, world, "founder").expect("a party character");
            m.hp_lost = hp_lost;
            let f = roster::field_unit(&d, &m, world).expect("fielded");
            let (unit, unloaded, profile) = old_build(&d, &m, world);
            assert_eq!(format!("{:?}", f.unit), format!("{unit:?}"), "{world}, hp_lost {hp_lost}");
            assert_eq!((f.pouch_unloaded, f.profile), (unloaded, profile), "{world}, hp_lost {hp_lost}");
            let max = f.unit.max_hp as i64;
            let want = if hp_lost >= max { 1 } else { max - hp_lost.max(0) };
            assert_eq!(f.unit.hp as i64, want, "{world}: start HP = max - hp_lost, at least 1, at most max");
        }
    }
    // the played world decides the build: the deck rule's one active slot in `tw`, the overlay path in `ow`
    let field = |w: &str| roster::field_unit(&d, &new_game::hero_from_character(&d, w, "founder").unwrap(), w).unwrap().unit;
    let (classic, timeline, overlay) = (field("w"), field("tw").active_families(), field("ow").known_skills);
    assert!(classic.active_families().len() > 1 && timeline.len() == 1, "{:?} / {timeline:?}", classic.active_families());
    assert!(PATH.iter().all(|p| has(&classic.known_skills, p)) && has(&overlay, PATH[1]) && !has(&overlay, PATH[0]), "{overlay:?}");
    let m = new_game::hero_from_character(&d, "w", "founder").unwrap();
    assert_eq!(format!("{:?}", roster::field_unit(&d, &m, "ow").unwrap().unit), format!("{:?}", old_build(&d, &m, "ow").0), "a member of w played in ow");
    let mut broken = m.clone();
    broken.base_unit = "ghost".into();
    assert!(roster::field_unit(&d, &broken, "w").is_none(), "an invalid member does not take the field");
}

fn party_of(d: &Data, members: &[(&str, i64)]) -> GameState {
    let mut party = PartyState::default();
    for (id, hp_lost) in members {
        let mut m = UnitState::create(id, "hero");
        m.hp_lost = *hp_lost;
        m.pouch = vec!["old".into()];
        party.members.push(m);
    }
    GameState::create(d, "w", party)
}

fn end(member: &str, hp: i64, max_hp: i64, alive: bool) -> HeroEnd {
    HeroEnd { member: member.into(), hp, max_hp, alive, pouch: vec!["left".into()], unloaded: vec!["spare".into()], fate: None }
}

#[test]
fn settle_party_is_heal1_classic() {
    let d = data();
    let max = 40;
    // HealRules: the rally is RALLY_PCT of max, rounded up; won adds it (a downed hero rallies to it), fled keeps HP
    // (a downed hero at 1), lost leaves HP to the defeat rule
    let rally = (max * progression::RALLY_PCT + 99) / 100;
    let cases: Vec<(&str, i64, bool, i64)> = vec![
        ("won", 0, false, max - rally),
        ("won", 10, true, max - (10 + rally)),
        ("won", 35, true, 0),
        ("won", 55, true, 0),
        ("fled", 0, false, max - 1),
        ("fled", 12, true, max - 12),
        ("fled", 55, true, 0),
        ("lost", 0, false, 5),
        ("lost", 30, true, 5),
    ];
    for (outcome, hp, alive, want) in cases {
        let mut gs = party_of(&d, &[("a", 5), ("b", 5)]);
        roster::settle_party(&d, &mut gs, outcome, &[end("a", hp, max, alive)]);
        let (a, b) = (gs.party.member_mut("a").unwrap().clone(), gs.party.member_mut("b").unwrap().clone());
        assert_eq!(a.hp_lost, want, "{outcome}, hp {hp}, alive {alive}");
        assert_eq!(a.pouch, ["left", "spare"], "{outcome}: the unused pouch, then the ids that found no slot");
        assert_eq!((b.hp_lost, b.pouch), (5, vec!["old".to_string()]), "{outcome}: a member not fielded is untouched");
    }
    let mut gs = party_of(&d, &[("a", 5)]);
    roster::settle_party(&d, &mut gs, "won", &[end("nobody", 0, max, false)]);
    assert_eq!(gs.party.members[0].hp_lost, 5, "an end without a member changes nothing");
}

#[test]
fn hero_ends_pair_the_fielded_members_with_the_allies() {
    let d = data();
    let ids = ["founder", "second"];
    let members: Vec<UnitState> = ids.iter().map(|c| new_game::hero_from_character(&d, "w", c).unwrap()).collect();
    let mut party = Vec::new();
    let mut fielded = Vec::new();
    for m in &members {
        let f = roster::field_unit(&d, m, "w").unwrap();
        party.push(Prebuilt { unit: f.unit, art: String::new(), palette: String::new(), profile: f.profile, key: m.id.clone() });
        fielded.push((m.id.clone(), f.pouch_unloaded));
    }
    let def = battle::encounter_def(&d, "w", d.resolve("w", "encounters", "fight"));
    let mut s = Session::start_with(d.cat.clone(), def, &WorldRoster { d: &d, world: "w".into() }, party);
    assert_eq!(s.w.allies.len(), 3, "two heroes and the guest");
    let (first, second) = (s.w.allies[0], s.w.allies[1]);
    s.w.units[first].hp = 0;
    s.w.units[second].hp = 9;
    let ends = roster::hero_ends(&s, &fielded);
    assert_eq!(ends.iter().map(|e| e.member.as_str()).collect::<Vec<_>>(), ids, "one end per member, in field order; the guest has none");
    assert_eq!((ends[0].hp, ends[0].alive, ends[0].fate), (0, false, Some(PartyFate::Downed)));
    assert_eq!((ends[1].hp, ends[1].alive, ends[1].fate), (9, true, Some(PartyFate::Standing)));
    assert_eq!(ends[1].max_hp, s.w.units[second].max_hp as i64);
    // settled as the battle screen did: won rallies the downed founder, the second keeps 9 + the rally
    let mut gs = GameState::create(&d, "w", PartyState { members: members.clone(), ..PartyState::default() });
    roster::settle_party(&d, &mut gs, "won", &ends);
    let max = ends[0].max_hp;
    assert_eq!(gs.party.members[0].hp_lost, max - progression::hp_after(0, max, "won"));
    assert_eq!(gs.party.members[1].hp_lost, ends[1].max_hp - progression::hp_after(9, ends[1].max_hp, "won"));
}
