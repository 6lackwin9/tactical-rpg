//! ROSTER1-L on scratch worlds (no shipped data): the roster's battle calls. Slice 2 moved the battle screen's hero
//! build and HP write-back into `rpgkit_world::roster` with no rule change: `field_unit` is exactly the old build
//! (`progression::build_in` in the battle's world under that world's deck rule, at `start_hp`, the base unit's AI
//! profile), so it follows the played world's class path and deck rule (decision "HOOKS1-UI slice 1 answers");
//! `settle_party` is ITEMUSE1 rule 7's pouch write-back on every outcome and HEAL1's `hp_after` on won and fled;
//! `hero_ends` pairs the fielded members with the session's allies and carries ENCOUNTER2 S3's fates. Slice 3 (the
//! tests from `ROSTER1_TASKS.md` 1, rule ids of `ROSTER1.md`): the rule rows (3.2, L3-L6), wounded (W1-W3),
//! `depart` and the named keepsake (L1, L2, KS1, KS2) and the save keys (11.4).

use rpgkit_battle::encounter::PartyFate;
use rpgkit_battle::session::{Prebuilt, Session};
use rpgkit_battle::world::AiProfile;
use rpgkit_rules::DeckRule;
use rpgkit_world::battle::{self, WorldRoster};
use rpgkit_world::roster::{self, FightEnd, HeroEnd};
use rpgkit_world::state::{Departed, PartyState, UnitState};
use rpgkit_world::{campaign, new_game, progression, save, Data, GameState};
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
        "items": [
            {"id": "ks_founder", "name": "Founder's Charm", "item_type": "ACCESSORY", "slot": "ACCESSORY_1", "value": 0, "special": true},
            {"id": "ring", "name": "Ring", "item_type": "ACCESSORY", "slot": "ACCESSORY_1", "value": 5},
            {"id": "tonic", "name": "Tonic", "item_type": "CONSUMABLE", "value": 2}
        ],
        "rulesets": [{"id": "timeline", "name": "Timeline", "rules": ["round_intent", "deck"], "basics": ["move", "rest", "strike", "active_guard"],
            "deck_slots": {"active": 1, "passive": 1, "keystone": 1}}]
    });
    let worlds = json!({
        "w": {"manifest": {"id": "w"}, "own": own("w")},
        "tw": {"manifest": {"id": "tw", "ruleset": "timeline"}, "own": own("tw")},
        "ow": {"manifest": {"id": "ow", "classes": {"drill": {"name": "Bash-Hand", "skill_path": {PATH[1]: 1}}}}, "own": own("ow")},
        // ROSTER1-D's shape (ROSTER1_TASKS 5.1) on a campaign world: Act 1 wounds, Act 2 captures
        "cw": {"manifest": {"id": "cw", "campaign": {"start_act": 1}, "roster": {
            "rules": [
                {"acts": [0, 1], "downed_won": "wounded", "downed_left": "wounded", "downed_lost": "wounded", "standing_lost": "wounded",
                 "wounded_max_hp_pct": WOUNDED_PCT, "keepsakes_on": ["captured"]},
                {"acts": [2, 3], "downed_won": "wounded", "downed_left": "captured", "downed_lost": "captured", "standing_lost": "wounded",
                 "wounded_max_hp_pct": WOUNDED_PCT}],
            "keepsakes": {"on": ["captured", "dead"]}}}, "own": keepsaker("cw")},
        // one open row with `hp_after` full (no campaign: act 0)
        "fw": {"manifest": {"id": "fw", "roster": {"rules": [{"hp_after": "full", "downed_won": "wounded", "downed_lost": "dead"}]}}, "own": own("fw")}
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// The row's wounded cut in `cw` (not the default, so the build is seen to read the row).
const WOUNDED_PCT: i32 = 30;

/// `own(w)` with the founder's named keepsake (KS2).
fn keepsaker(w: &str) -> Value {
    let mut v = own(w);
    v["characters"][0]["keepsake"] = json!("ks_founder");
    v
}

/// A fresh game state in world `w` with no members.
fn gs_of(d: &Data, w: &str) -> GameState {
    GameState::create(d, w, PartyState::default())
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
            let f = roster::field_unit(&d, &gs_of(&d, world), &m, world).expect("fielded");
            let (unit, unloaded, profile) = old_build(&d, &m, world);
            assert_eq!(format!("{:?}", f.unit), format!("{unit:?}"), "{world}, hp_lost {hp_lost}");
            assert_eq!((f.pouch_unloaded, f.profile), (unloaded, profile), "{world}, hp_lost {hp_lost}");
            let max = f.unit.max_hp as i64;
            let want = if hp_lost >= max { 1 } else { max - hp_lost.max(0) };
            assert_eq!(f.unit.hp as i64, want, "{world}: start HP = max - hp_lost, at least 1, at most max");
        }
    }
    // the played world decides the build: the deck rule's one active slot in `tw`, the overlay path in `ow`
    let field = |w: &str| roster::field_unit(&d, &gs_of(&d, w), &new_game::hero_from_character(&d, w, "founder").unwrap(), w).unwrap().unit;
    let (classic, timeline, overlay) = (field("w"), field("tw").active_families(), field("ow").known_skills);
    assert!(classic.active_families().len() > 1 && timeline.len() == 1, "{:?} / {timeline:?}", classic.active_families());
    assert!(PATH.iter().all(|p| has(&classic.known_skills, p)) && has(&overlay, PATH[1]) && !has(&overlay, PATH[0]), "{overlay:?}");
    let m = new_game::hero_from_character(&d, "w", "founder").unwrap();
    assert_eq!(format!("{:?}", roster::field_unit(&d, &gs_of(&d, "w"), &m, "ow").unwrap().unit), format!("{:?}", old_build(&d, &m, "ow").0), "a member of w played in ow");
    let mut broken = m.clone();
    broken.base_unit = "ghost".into();
    assert!(roster::field_unit(&d, &gs_of(&d, "w"), &broken, "w").is_none(), "an invalid member does not take the field");
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
        roster::settle_party(&d, &mut gs, &FightEnd::new(outcome, "w"), &[end("a", hp, max, alive)]);
        let (a, b) = (gs.party.member_mut("a").unwrap().clone(), gs.party.member_mut("b").unwrap().clone());
        assert_eq!(a.hp_lost, want, "{outcome}, hp {hp}, alive {alive}");
        assert_eq!(a.pouch, ["left", "spare"], "{outcome}: the unused pouch, then the ids that found no slot");
        assert_eq!((b.hp_lost, b.pouch), (5, vec!["old".to_string()]), "{outcome}: a member not fielded is untouched");
    }
    let mut gs = party_of(&d, &[("a", 5)]);
    roster::settle_party(&d, &mut gs, &FightEnd::new("won", "w"), &[end("nobody", 0, max, false)]);
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
        let f = roster::field_unit(&d, &gs_of(&d, "w"), m, "w").unwrap();
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
    roster::settle_party(&d, &mut gs, &FightEnd::new("won", "w"), &ends);
    let max = ends[0].max_hp;
    assert_eq!(gs.party.members[0].hp_lost, max - progression::hp_after(0, max, "won"));
    assert_eq!(gs.party.members[1].hp_lost, ends[1].max_hp - progression::hp_after(9, ends[1].max_hp, "won"));
}

// ---- slice 3: the rule rows, wounded, departures, keepsakes, save keys ----------------------------------------------

/// A game in world `w` at `act` (a campaign world's act; ignored without a campaign) with the founder, the second
/// and a benched third (members fielded: the first two), all at full HP.
fn cell(d: &Data, w: &str, act: i64) -> GameState {
    let mut party = PartyState::default();
    for c in ["founder", "second"] {
        party.members.push(new_game::hero_from_character(d, w, c).expect("a party character"));
    }
    party.members.push(UnitState::create("third", "hero"));
    party.active_ids = vec!["founder".into(), "second".into()];
    let mut gs = GameState::create(d, w, party);
    if let Some(c) = gs.campaign.as_mut() {
        c.act = act;
    }
    gs
}

/// A fielded hero's end at `hp` of the battle max `max` with ENCOUNTER2's `fate` (0 HP = not alive).
fn fated(member: &str, hp: i64, max: i64, fate: PartyFate) -> HeroEnd {
    HeroEnd { member: member.into(), hp, max_hp: max, alive: hp > 0, pouch: Vec::new(), unloaded: Vec::new(), fate: Some(fate) }
}

fn max_of(d: &Data, gs: &GameState, id: &str) -> i64 {
    progression::max_hp_of(d, gs.party.member(id).unwrap(), &gs.world_id)
}

fn wounded(gs: &GameState, id: &str) -> bool {
    gs.party.member(id).is_some_and(|m| m.has_condition(roster::WOUNDED))
}

#[test]
fn no_rule_row_settles_heal1_classic_whatever_the_fates() {
    // L3: a world with no row (w) settles every fate and objective as HEAL1 did; nobody is wounded or departs
    let d = data();
    for outcome in ["won", "fled", "lost"] {
        for escape in [false, true] {
            let mut gs = cell(&d, "w", 0);
            let max = max_of(&d, &gs, "founder");
            let f = FightEnd { escape, ..FightEnd::new(outcome, "w") };
            let ends = [fated("founder", 0, max, PartyFate::Downed), fated("second", 9, max, PartyFate::Escaped)];
            let settled = roster::settle_party(&d, &mut gs, &f, &ends);
            assert_eq!(settled, roster::Settled::default(), "{outcome}");
            let want = |hp: i64| if outcome == "lost" { 0 } else { max - progression::hp_after(hp, max, outcome) };
            assert_eq!((gs.party.members[0].hp_lost, gs.party.members[1].hp_lost), (want(0), want(9)), "{outcome}, escape {escape}");
            assert!(gs.party.members.iter().all(|m| m.conditions.is_empty()) && gs.party.departed.is_empty(), "{outcome}");
        }
    }
}

#[test]
fn act_one_row_wounds_the_downed_and_every_hero_of_a_lost_battle() {
    // 3.2 with ROSTER1_TASKS 5.1's Act 1 row: a won rout wounds the downed hero only; a lost battle wounds every
    // fielded hero (downed_lost, standing_lost) and, `carry` on a loss, brings them home healed (HEAL1's wake heal)
    let d = data();
    let mut gs = cell(&d, "cw", 1);
    let max = max_of(&d, &gs, "founder");
    let settled = roster::settle_party(&d, &mut gs, &FightEnd::new("won", "cw"), &[fated("founder", 0, max, PartyFate::Downed), fated("second", 9, max, PartyFate::Standing)]);
    assert_eq!(settled.wounded, ["founder"]);
    assert!(wounded(&gs, "founder") && !wounded(&gs, "second") && !wounded(&gs, "third"));
    assert_eq!(gs.party.members[1].hp_lost, max - progression::hp_after(9, max, "won"), "a standing hero keeps HEAL1's carry-over");
    let mut gs = cell(&d, "cw", 1);
    gs.party.members[1].hp_lost = 7;
    let settled = roster::settle_party(&d, &mut gs, &FightEnd::new("lost", "cw"), &[fated("founder", 0, max, PartyFate::Downed), fated("second", 9, max, PartyFate::Standing)]);
    assert_eq!(settled.wounded, ["founder", "second"]);
    assert!(settled.departed.is_empty() && gs.party.departed.is_empty(), "Act 1 has no capture");
    assert!(!wounded(&gs, "third"), "a benched member is untouched");
    assert_eq!((gs.party.members[0].hp_lost, gs.party.members[1].hp_lost), (0, 0), "a loss under carry comes home healed");
    // L6: the act picks the row; no campaign = act 0 (fw's open row)
    assert_eq!(roster::act(&cell(&d, "fw", 3)), 0);
    assert_eq!(roster::rule(&d, &cell(&d, "cw", 2)).unwrap().downed_left, "captured");
    assert!(roster::rule(&d, &cell(&d, "cw", 5)).is_none(), "an act no row covers is classic");
}

#[test]
fn a_wounded_member_fights_at_the_cut_max_and_writes_back_against_the_full_max() {
    // W2: battle max = max - roundi(max x pct / 100) (the row's pct), start HP at most that; HP written back against
    // the unwounded max
    let d = data();
    let mut gs = cell(&d, "cw", 1);
    let full = max_of(&d, &gs, "founder");
    let cut = full - (full as f64 * WOUNDED_PCT as f64 / 100.0).round() as i64;
    assert_eq!(roster::wounded_max(full, WOUNDED_PCT), cut);
    gs.party.members[0].add_condition(roster::WOUNDED);
    let f = roster::field_unit(&d, &gs, &gs.party.members[0], "cw").unwrap();
    assert_eq!((f.unit.max_hp as i64, f.unit.hp as i64), (cut, cut), "hp_lost 0: starts at the cut max");
    gs.party.members[0].hp_lost = full - cut + 3;
    let f = roster::field_unit(&d, &gs, &gs.party.members[0], "cw").unwrap();
    assert_eq!(f.unit.hp as i64, cut - 3, "start HP = min(start_hp(max, hp_lost), the cut max)");
    gs.party.members[0].hp_lost = 0;
    // a won fight it ends untouched at the cut max: hp_lost = full - cut (the rally cannot pass the battle max)
    roster::settle_party(&d, &mut gs, &FightEnd::new("won", "cw"), &[fated("founder", cut, cut, PartyFate::Standing)]);
    assert_eq!(gs.party.members[0].hp_lost, full - cut);
    // the second, unwounded, fields at its full max
    assert_eq!(roster::field_unit(&d, &gs, &gs.party.members[1], "cw").unwrap().unit.max_hp as i64, full);
}

#[test]
fn wounded_is_held_once_and_a_rest_clears_it() {
    // W1 (RQ5 a): downed twice while wounded keeps one entry; W3: every rest path clears it, the wake does not
    let d = data();
    let mut gs = cell(&d, "cw", 1);
    let max = max_of(&d, &gs, "founder");
    for _ in 0..2 {
        roster::settle_party(&d, &mut gs, &FightEnd::new("won", "cw"), &[fated("founder", 0, max, PartyFate::Downed)]);
    }
    assert_eq!(gs.party.members[0].conditions, [roster::WOUNDED]);
    assert!(progression::hurt(&gs.party), "a wounded member is hurt (Camp and the campaign rest are not refused)");
    progression::settle_defeat(&mut gs, &serde_json::Map::new());
    assert!(wounded(&gs, "founder"), "the defeat wake is not a rest");
    progression::rest(&d, &mut gs.party, "cw");
    assert!(!wounded(&gs, "founder") && !progression::hurt(&gs.party), "a rest clears wounded");
    // CAMPAIGN1 R3's `rest` effect rests the same way
    gs.party.members[1].add_condition(roster::WOUNDED);
    campaign::purse::rest_effect(&d, &mut gs);
    assert!(gs.party.members.iter().all(|m| m.conditions.is_empty()));
}

#[test]
fn act_two_row_captures_the_downed_left_behind() {
    // 3.2, L2: fled with a hero downed: captured (cause downed), out of the members (its bunk free) and active_ids,
    // its gear and pouch in the stash, the day recorded; the hero who escaped comes home
    let d = data();
    let mut gs = cell(&d, "cw", 2);
    gs.time = 6;
    gs.party.members[0].equipment.insert("accessory_1".into(), "ring".into());
    let max = max_of(&d, &gs, "founder");
    let mut down = fated("founder", 0, max, PartyFate::Downed);
    down.pouch = vec!["tonic".into()];
    let f = FightEnd { encounter: "cw:fight".into(), ..FightEnd::new("fled", "cw") };
    let settled = roster::settle_party(&d, &mut gs, &f, &[down, fated("second", 5, max, PartyFate::Escaped)]);
    assert_eq!(settled.departed, [("founder".to_string(), "captured".to_string())]);
    assert!(gs.party.member("founder").is_none() && gs.party.members.len() == 2);
    assert_eq!(gs.party.active_ids, ["second"]);
    let rec = gs.party.departed_of("founder").expect("recorded");
    assert_eq!((rec.life.as_str(), rec.cause.as_str(), rec.encounter.as_str(), rec.day), ("captured", "downed", "cw:fight", 6));
    assert_eq!(rec.member.equipment.get("accessory_1").map(String::as_str), Some("ring"), "the record keeps the gear it wore");
    assert_eq!((gs.party.stash.get("ring"), gs.party.stash.get("tonic")), (Some(&1), Some(&1)), "RQ2 (a): its gear and pouch to the stash");
    assert_eq!(gs.party.members[0].hp_lost, max - progression::hp_after(5, max, "fled"));
    // a won escape does not hold the field: the downed are left (downed_left); a held field wounds (downed_won)
    for (escape, departs) in [(true, true), (false, false)] {
        let mut gs = cell(&d, "cw", 2);
        let f = FightEnd { escape, ..FightEnd::new("won", "cw") };
        roster::settle_party(&d, &mut gs, &f, &[fated("founder", 0, max, PartyFate::Downed)]);
        assert_eq!((gs.party.departed.len() == 1, wounded(&gs, "founder")), (departs, !departs), "escape {escape}");
    }
    // `then` none: a stayer at 0 HP settles as downed (captured on a loss: cause lost_battle), one still up as escaped
    let mut gs = cell(&d, "cw", 2);
    roster::settle_party(&d, &mut gs, &FightEnd::new("lost", "cw"), &[fated("founder", 0, max, PartyFate::Stayed), fated("second", 4, max, PartyFate::Stood)]);
    assert_eq!(gs.party.departed_of("founder").map(|r| r.cause.as_str()), Some("lost_battle"));
    assert!(wounded(&gs, "second") && gs.party.departed.len() == 1, "standing_lost wounds the one still up");
}

#[test]
fn a_departure_pays_the_named_keepsake_once_when_the_row_lists_the_life() {
    // KS1, KS2: the row's keepsakes_on (else the manifest's keepsakes.on) lists the life: the stash gets the
    // character's keepsake once and the record keeps its id
    let d = data();
    let mut gs = cell(&d, "cw", 1);
    let rec = roster::depart(&d, &mut gs, "founder", "captured", "story", "").expect("departs");
    assert_eq!((rec.keepsake.as_str(), gs.party.stash.get("ks_founder")), ("ks_founder", Some(&1)));
    assert!(roster::depart(&d, &mut gs, "founder", "captured", "story", "").is_err(), "a departed member does not depart again");
    assert_eq!(gs.party.stash.get("ks_founder"), Some(&1), "paid once");
    // Act 1's row lists only captured: a death pays nothing; Act 2's row sets none, so keepsakes.on (dead) pays
    let mut gs = cell(&d, "cw", 1);
    assert_eq!(roster::depart(&d, &mut gs, "founder", "dead", "story", "").unwrap().keepsake, "");
    let mut gs = cell(&d, "cw", 2);
    assert_eq!(roster::depart(&d, &mut gs, "founder", "dead", "story", "").unwrap().keepsake, "ks_founder");
    // a member whose character has no keepsake, an unknown life or cause
    assert_eq!(roster::depart(&d, &mut gs, "second", "captured", "story", "").unwrap().keepsake, "");
    assert!(roster::depart(&d, &mut gs, "third", "vanished", "story", "").is_err() && roster::depart(&d, &mut gs, "third", "dead", "boredom", "").is_err());
    assert!(gs.party.member("third").is_some(), "a refused departure changes nothing");
}

#[test]
fn hp_after_full_brings_everyone_home_at_full() {
    // L4 `full`: wounded is then the only carry-over; a death departs (fw's open row: downed_lost dead)
    let d = data();
    let mut gs = cell(&d, "fw", 0);
    let max = max_of(&d, &gs, "founder");
    roster::settle_party(&d, &mut gs, &FightEnd::new("won", "fw"), &[fated("founder", 0, max, PartyFate::Downed), fated("second", 3, max, PartyFate::Standing)]);
    assert_eq!((gs.party.members[0].hp_lost, gs.party.members[1].hp_lost), (0, 0));
    assert!(wounded(&gs, "founder") && !wounded(&gs, "second"));
    roster::settle_party(&d, &mut gs, &FightEnd::new("lost", "fw"), &[fated("founder", 0, max, PartyFate::Downed)]);
    assert_eq!(gs.party.departed_of("founder").map(|r| (r.life.as_str(), r.cause.as_str())), Some(("dead", "lost_battle")));
}

#[test]
fn fight_end_reads_the_session() {
    let d = data();
    let def = battle::encounter_def(&d, "w", d.resolve("w", "encounters", "fight"));
    let s = Session::start_with(d.cat.clone(), def, &WorldRoster { d: &d, world: "w".into() }, Vec::new());
    let f = FightEnd::of(&s, "w");
    assert_eq!((f.encounter.as_str(), f.world.as_str(), f.escape, f.outcome.clone()), (s.def.id.as_str(), "w", false, s.final_outcome()));
}

fn envelope(gs: &GameState, version: i64) -> String {
    json!({"format": save::FORMAT, "version": version, "saved_at": 1, "game_state": gs.to_value()}).to_string()
}

#[test]
fn the_save_keys_round_trip_and_older_saves_read_empty() {
    // 11.4: `conditions` and `departed` are saved only when not empty (a classic party's save is unchanged); a save
    // of the last version before the shared bump reads them empty; a round trip keeps them
    let d = data();
    let mut gs = cell(&d, "cw", 1);
    let plain = gs.to_value();
    assert!(plain["party"].get("departed").is_none() && plain["party"]["members"][0].get("conditions").is_none());
    let old = save::decode(&d, &envelope(&gs, save::VERSION - 1)).expect("an older save loads");
    assert_eq!(old.state.to_value(), plain);
    assert!(old.state.party.departed.is_empty() && old.state.party.members.iter().all(|m| m.conditions.is_empty()));
    gs.time = 3;
    roster::depart(&d, &mut gs, "founder", "captured", "downed", "cw:fight").unwrap();
    gs.party.members[0].add_condition(roster::WOUNDED);
    let back = save::decode(&d, &envelope(&gs, save::VERSION)).expect("loads");
    assert_eq!(back.state.to_value(), gs.to_value());
    assert_eq!(back.state.party.departed, gs.party.departed);
    assert_eq!(back.state.party.members[0].conditions, [roster::WOUNDED]);
    // validate: an unknown condition, a doubled one, an unknown life or cause, a departed id that is also a member
    let mut bad = gs.clone();
    bad.party.members[0].conditions = vec!["cursed".into(), roster::WOUNDED.into(), roster::WOUNDED.into()];
    let ghost = Departed { member: bad.party.members[1].clone(), life: "lost_at_sea".into(), cause: "whim".into(), ..Departed::default() };
    bad.party.departed.push(ghost);
    let errors = bad.validate(&d);
    for want in ["unknown condition 'cursed'", "condition 'wounded' is listed twice", "unknown life 'lost_at_sea'", "unknown cause 'whim'", "is also a member"] {
        assert!(errors.iter().any(|e| e.contains(want)), "{want}: {errors:?}");
    }
    assert!(save::decode(&d, &envelope(&bad, save::VERSION)).is_err(), "a damaged roster is refused, never half-loaded");
}
