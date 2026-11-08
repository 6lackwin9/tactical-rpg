//! W.2-S1 (FB-50a, W_WORLD.md section 3) on a scratch world: `WorldRoster::new` is the one constructor of a world
//! battle's roster (it resolves the world's characters like the struct it replaced), and `battle::finish_world` settles
//! a world fight by the runner that started it: each runner's settle exactly (`travel`, `dialogue`, `story`, `trigger`,
//! anything else `explore`), on the same state with the same result. A trigger fight settled through it ticks the
//! campaign like the trigger runner. Every expectation follows from the dispatch rule and the fixture's own numbers.
//! W.2-S2 (FB-50b, W_WORLD.md acceptance 1): `battle::begin_world` starts a world fight headlessly (the one hero
//! builder, the world's foes, the art callback), the auto driver plays it out and `finish_world` settles it: the party's
//! fates, the defeated foes' XP and the outcome's battle tick. W.2-S3 (FB-13): the world stage is the campaign's act,
//! 0 without a campaign; FB-139: an enemy fights at `level + stage * growth_per_stage` (capped at the top level) in the
//! battle and in its XP weight alike, while heroes, the party and guests keep their level.

use aegis_core::json::{self, Dict};
use rpgkit_battle::session::{Phase, Roster, Session};
use rpgkit_world::battle::{self, WorldRoster};
use rpgkit_world::campaign::trigger;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{dialogue, explore, progression, roster, story, travel, Data, GameState};
use serde_json::{json, Value};

const LOUD_WON: i64 = 8;

fn with_ids(w: &str, rows: Value) -> Value {
    let rows = json::arr(&rows)
        .iter()
        .map(|r| {
            let mut r = json::dict(r).clone();
            let id = json::s(&r, "id");
            r.insert("_world".into(), w.into());
            r.insert("_qid".into(), format!("{w}:{id}").into());
            Value::Object(r)
        })
        .collect();
    Value::Array(rows)
}

/// World `w`: a campaign from act 1 with an `alert` gauge moved +8 by a won loud battle, the hub, the encounter
/// `enc_loud` (tagged loud, no enemies) and the character Clerk (a party-side unit row).
fn data() -> Data {
    let w = "w";
    let meters = json!([{"id": "alert", "name": "Alert", "max": 100, "bands": [{"id": "calm", "name": "Calm", "from": 0, "tone": "calm"}],
                         "inputs": [{"on": "battle", "tags": ["loud"], "outcomes": ["won"], "delta": LOUD_WON}]}]);
    let worlds = json!({
        w: {"manifest": {"id": w, "start_location": "hub", "start_party": ["hero"], "campaign": {"start_act": 1}}, "own": {
            "locations": with_ids(w, json!([{"id": "hub", "name": "Hub"}])),
            "encounters": with_ids(w, json!([{"id": "enc_loud", "name": "Loud", "tags": ["loud"], "enemies": []}])),
            "characters": with_ids(w, json!([{"id": "clerk", "name": "Clerk", "role": "npc", "base_unit": "hero"}])),
            "meters": with_ids(w, meters)
        }}
    });
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let mut gs = GameState::create(d, "w", party);
    explore::enter(d, &mut gs, "hub", explore::SPAWN);
    if let Some(c) = gs.campaign.as_mut() {
        c.sync(gs.time);
    }
    gs
}

fn alert(gs: &GameState) -> i64 {
    gs.campaign.as_ref().map_or(-1, |c| c.value("alert", gs.time))
}

type Settle = fn(&Data, &mut GameState, &Dict, &str, Option<&battle::Tally>) -> Dict;

#[test]
fn finish_world_settles_each_runner_exactly_like_its_own_after_fight() {
    let d = data();
    let runners: [(&str, Settle); 6] = [
        ("explore", explore::after_fight),
        ("travel", travel::after_fight),
        ("dialogue", dialogue::after_fight),
        (story::FIGHT_SOURCE, story::after_fight),
        (trigger::FIGHT_SOURCE, trigger::after_fight),
        ("some_other_source", explore::after_fight),
    ];
    for (runner, own) in runners {
        for outcome in battle::OUTCOMES {
            let fight = json::dict(&json!({"source": runner, "encounter": "w:enc_loud"})).clone();
            let (mut via_kit, mut direct) = (state(&d), state(&d));
            let got = battle::finish_world(&d, &mut via_kit, runner, &fight, outcome, None);
            let want = own(&d, &mut direct, &fight, outcome, None);
            assert_eq!(got, want, "{runner} {outcome}: the same result");
            assert_eq!(via_kit, direct, "{runner} {outcome}: the same state after");
        }
    }
}

#[test]
fn a_trigger_fight_finished_in_the_kit_ticks_the_campaign() {
    let d = data();
    let mut gs = state(&d);
    let before = alert(&gs);
    let fight = json::dict(&json!({"source": trigger::FIGHT_SOURCE, "encounter": "w:enc_loud"})).clone();
    let res = battle::finish_world(&d, &mut gs, trigger::FIGHT_SOURCE, &fight, "won", None);
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(alert(&gs), before + LOUD_WON, "the battle tick of a won loud fight");
}

#[test]
fn world_roster_new_resolves_the_worlds_characters() {
    let d = data();
    let cat = d.cat.clone();
    let from_str = WorldRoster::new(&d, "w");
    let from_string = WorldRoster::new(&d, String::from("w"));
    assert_eq!((from_str.world.as_str(), from_string.world.as_str()), ("w", "w"));
    assert!(std::ptr::eq(from_str.d, &d), "the roster reads the given data");
    assert_eq!(from_str.lore_ref(&cat, "clerk"), "w:clerk", "a character's lore ref is its qualified id");
    assert_eq!(from_str.entry(&cat, "clerk").map(|u| u.id), from_string.entry(&cat, "clerk").map(|u| u.id), "either form resolves alike");
    assert!(from_str.entry(&cat, "clerk").is_some(), "the world's character spawns as its unit");
    assert!(WorldRoster::new(&d, "elsewhere").entry(&cat, "clerk").is_none(), "another world's roster does not see it");
}

// ---- W.2-S2 (FB-50b): `begin_world` and the headless round trip; W.2-S3 (FB-13): the world stage --------------------

/// The alert gauge's battle input per outcome of a `fight` tagged fight (distinct, so the tick names its outcome).
const FIGHT_DELTA: [(&str, i64); 3] = [("won", 5), ("lost", 3), ("fled", 2)];
/// The hero's stored wound (HP lost before the fight).
const HP_LOST: i64 = 3;

/// The growing units' base level and `growth_per_stage` (FB-139).
const GROW_BASE: i64 = 2;
const GROWTH: i64 = 2;

/// World `fw` (a campaign from act 1) and `cw` (no campaign): a drill class with basic strike, guard, move and rest;
/// the hero (level 3) and the rat (an enemy at level 1, a world character); the encounter `brawl` (tagged fight, one
/// rat) and the gauge `alert` moved by every battle outcome of a fight-tagged encounter (`FIGHT_DELTA`). FB-139: the
/// encounter `pack` fights the wolf (a standard enemy) and the rival (a hero-tier enemy) beside the guest squire (a
/// standard party-side unit), all three at `GROW_BASE` with `GROWTH` per stage.
fn fight_data() -> Data {
    let inputs: Vec<Value> = FIGHT_DELTA.iter().map(|(o, d)| json!({"on": "battle", "tags": ["fight"], "outcomes": [o], "delta": d})).collect();
    let meters = json!([{"id": "alert", "name": "Alert", "max": 100, "bands": [{"id": "calm", "name": "Calm", "from": 0, "tone": "calm"}], "inputs": inputs}]);
    let own = |w: &str| {
        json!({
            "locations": with_ids(w, json!([{"id": "hub", "name": "Hub"}])),
            "encounters": with_ids(w, json!([
                {"id": "brawl", "name": "Brawl", "tags": ["fight"], "enemies": ["rat"], "intro": "A rat."},
                {"id": "pack", "name": "Pack", "enemies": ["wolf", "rival"], "guests": ["squire"]}
            ])),
            "characters": with_ids(w, json!([
                {"id": "rat", "name": "Rat", "role": "npc", "base_unit": "rat"},
                {"id": "wolf", "name": "Wolf", "role": "npc", "base_unit": "wolf"},
                {"id": "rival", "name": "Rival", "role": "npc", "base_unit": "rival"},
                {"id": "squire", "name": "Squire", "role": "npc", "base_unit": "squire"}
            ])),
            "meters": with_ids(w, meters.clone())
        })
    };
    let worlds = json!({
        "fw": {"manifest": {"id": "fw", "start_location": "hub", "start_party": ["hero"], "campaign": {"start_act": 1}}, "own": own("fw")},
        "cw": {"manifest": {"id": "cw", "start_location": "hub", "start_party": ["hero"]}, "own": own("cw")}
    });
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}
        ],
        "units": [
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 3},
            {"id": "rat", "name": "Rat", "side": "enemy", "class_id": "drill", "level": 1},
            {"id": "wolf", "name": "Wolf", "side": "enemy", "class_id": "drill", "level": GROW_BASE, "tier": "standard", "growth_per_stage": GROWTH},
            {"id": "rival", "name": "Rival", "side": "enemy", "class_id": "drill", "level": GROW_BASE, "tier": "hero", "growth_per_stage": GROWTH},
            {"id": "squire", "name": "Squire", "side": "party", "class_id": "drill", "level": GROW_BASE, "tier": "standard", "growth_per_stage": GROWTH}
        ]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn fight_state(d: &Data, w: &str) -> GameState {
    let mut party = PartyState::default();
    let mut hero = UnitState::create("hero", "hero");
    hero.hp_lost = HP_LOST;
    party.members.push(hero);
    let mut gs = GameState::create(d, w, party);
    explore::enter(d, &mut gs, "hub", explore::SPAWN);
    if let Some(c) = gs.campaign.as_mut() {
        c.sync(gs.time);
    }
    gs
}

fn brawl(w: &str) -> Dict {
    json::dict(&json!({"source": trigger::FIGHT_SOURCE, "encounter": format!("{w}:brawl")})).clone()
}

/// The auto driver (every side drafted by the balanced preset) plays `s` to its end; the session's final outcome.
fn play_out(s: &mut Session) -> String {
    while !s.combat_over {
        if s.phase == Phase::Replay {
            if !s.can_advance_round() {
                break;
            }
            s.advance_round();
        }
        s.auto_draft("balanced");
        assert!(s.confirm_plan(), "the drafted plan confirms");
        s.resolve_round();
    }
    s.final_outcome()
}

#[test]
fn begin_world_fields_the_party_by_the_one_builder_and_the_foes_from_the_world() {
    let d = fight_data();
    let mut gs = fight_state(&d, "fw");
    let mut seen = Vec::new();
    let f = battle::begin_world_with(&d, &mut gs, &brawl("fw"), &mut |world, m| {
        seen.push((world.to_string(), m.id.clone()));
        (format!("art_{}", m.id), "pal".into())
    })
    .unwrap_or_else(|e| panic!("the fight starts: {e}"));
    assert_eq!((f.world.as_str(), f.intro.as_str(), f.stage), ("fw", "A rat.", 1), "the fight world, its intro, stage = act 1");
    assert_eq!(seen, [("fw".to_string(), "hero".to_string())], "the art callback sees each fielded member once");
    assert_eq!(f.members.iter().map(|(m, _)| m.as_str()).collect::<Vec<_>>(), ["hero"]);
    let s = &f.session;
    assert_eq!(s.def.id, "fw:brawl");
    assert_eq!((s.w.allies.len(), s.w.enemies.len()), (1, 1), "the hero and the rat");
    let built = roster::field_unit(&d, &gs, &gs.party.members[0], "fw").expect("the hero builds");
    let hero = &s.w.units[s.w.allies[0]];
    assert_eq!((hero.hp, hero.max_hp), (built.unit.hp, built.unit.max_hp), "the one builder's unit at its stored HP");
    assert_eq!(hero.hp as i64, progression::start_hp(hero.max_hp as i64, HP_LOST), "the stored wound");
    assert!(battle::begin_world(&d, &mut gs, &brawl("nowhere")).is_err_and(|e| e.contains("Unknown encounter")));
    gs.party.active_ids = vec!["nobody".into()];
    assert_eq!(battle::begin_world(&d, &mut gs, &brawl("fw")).err().as_deref(), Some("No active party member to fight."), "no active member: refused (not a party_min refusal)");
}

#[test]
fn a_world_fight_round_trips_headlessly_party_then_rewards_then_the_clock() {
    let d = fight_data();
    let mut gs = fight_state(&d, "fw");
    let mut f = battle::begin_world(&d, &mut gs, &brawl("fw")).unwrap_or_else(|e| panic!("the fight starts: {e}"));
    let outcome = play_out(&mut f.session);
    assert!(battle::OUTCOMES.contains(&outcome.as_str()), "a settled outcome, got '{outcome}'");
    assert!(f.session.current_round >= 1, "the battle was played");
    let tally = f.tally();
    let (alert_before, level_before) = (alert(&gs), gs.party.members[0].level(&d));
    let foes = battle::roster_at(&d, "fw", "fw:brawl", f.stage);
    let (fates, _) = tally.fates_of(&foes);
    let defeated: Vec<(i64, String)> = foes.iter().zip(&fates).filter(|(_, x)| x.defeated()).map(|(r, _)| (r.level, r.tier.clone())).collect();
    let res = battle::finish_world(&d, &mut gs, trigger::FIGHT_SOURCE, &brawl("fw"), &outcome, Some(&tally));
    assert!(json::b(&res, "ok", false), "{res:?}");
    // 1. the party side: the hero's battle fate as the tally saw it, and its HP settled by the default rule (no rule row)
    let want_fates: Vec<Value> = tally.party.iter().filter_map(|e| e.fate.map(|x| json!([e.member, x.name()]))).collect();
    assert!(!want_fates.is_empty(), "the fielded hero has a fate");
    assert_eq!(res.get("party_fates"), Some(&Value::Array(want_fates)));
    let end = &tally.party[0];
    assert_eq!(gs.party.members[0].hp_lost, end.max_hp - progression::hp_after(end.hp, end.max_hp, &outcome), "{outcome}: the hero's HP after");
    // 2. the rewards: the roster's foes and the XP of the defeated ones at the hero's level before the award
    let rewards = json::dget(&res, "rewards");
    assert_eq!(json::i(rewards, "foes", -1), foes.len() as i64);
    let award = json::i(json::dget(json::dget(&res, "xp"), "hero"), "xp", 0);
    assert_eq!(award, progression::encounter_xp(&d, &defeated, level_before), "{outcome}: XP of the defeated foes");
    // 3. the clock: the battle tick of this outcome
    let delta = FIGHT_DELTA.iter().find(|(o, _)| *o == outcome).map(|(_, d)| *d).unwrap();
    assert_eq!(alert(&gs), alert_before + delta, "{outcome}: the battle tick");
}

#[test]
fn the_world_stage_is_the_campaign_act_and_zero_without_a_campaign() {
    let d = fight_data();
    let mut gs = fight_state(&d, "fw");
    assert_eq!(battle::world_stage(&gs), 1, "the campaign's start act");
    gs.campaign.as_mut().unwrap().act = 3;
    assert_eq!(battle::world_stage(&gs), 3, "follows the act");
    assert_eq!(battle::begin_world(&d, &mut gs, &brawl("fw")).map(|f| f.stage).unwrap_or(-1), 3, "begin_world plays at it");
    let mut plain = fight_state(&d, "cw");
    assert!(plain.campaign.is_none());
    assert_eq!(battle::world_stage(&plain), 0, "no campaign: stage 0");
    assert_eq!(battle::begin_world(&d, &mut plain, &brawl("cw")).map(|f| f.stage).unwrap_or(-1), 0);
    assert_eq!(WorldRoster::new(&d, "fw").stage, 0, "the host's and the sim's roster: stage 0");
    // FB-139: growth 0 (the rat, every row without the field) or stage 0 changes no foe's level or XP weight
    for stage in [0, 1, 3] {
        assert_eq!(battle::foe_level(4, stage, 0), 4, "growth 0 at stage {stage}");
        assert_eq!(battle::foe_level(4, 0, GROWTH), 4, "stage 0");
        assert_eq!(battle::roster_at(&d, "fw", "fw:brawl", stage), battle::roster(&d, "fw", "fw:brawl"));
        assert_eq!(battle::enemy_levels_at(&d, "fw", "fw:brawl", stage), battle::enemy_levels(&d, "fw", "fw:brawl"));
    }
    assert_eq!(battle::roster_at(&d, "fw", "fw:pack", 0), battle::roster(&d, "fw", "fw:pack"), "stage 0: every foe at its base");
    // the formula and its cap at the top unit level
    let top = i64::from(rpgkit_rules::stats::MAX_LEVEL);
    assert_eq!(battle::foe_level(GROW_BASE, 3, GROWTH), GROW_BASE + 3 * GROWTH);
    assert_eq!(battle::foe_level(GROW_BASE, top, GROWTH), top, "never above the top level");
}

/// The built unit named `name` in `s` and its level.
fn built_level(s: &Session, name: &str) -> i64 {
    let u = s.w.units.iter().find(|u| u.name == name).unwrap_or_else(|| panic!("{name} is in the battle"));
    i64::from(u.stats.level)
}

#[test]
fn an_enemy_grows_with_the_stage_in_the_battle_and_its_xp_weight_alike() {
    let d = fight_data();
    let cat = d.cat.clone();
    let mut gs = fight_state(&d, "fw");
    gs.campaign.as_mut().unwrap().act = 3;
    let stage = battle::world_stage(&gs);
    let grown = GROW_BASE + stage * GROWTH;
    // the battle: the wolf at its grown level; the hero-tier rival and the guest squire at their base level
    let f = battle::begin_world(&d, &mut gs, &json::dict(&json!({"source": trigger::FIGHT_SOURCE, "encounter": "fw:pack"})).clone())
        .unwrap_or_else(|e| panic!("the fight starts: {e}"));
    assert_eq!(f.stage, 3);
    assert_eq!(f.session.guests.len(), 1, "the squire fights as a guest");
    assert_eq!(built_level(&f.session, "Wolf"), grown, "an enemy fights at base + stage x growth");
    assert_eq!(built_level(&f.session, "Rival"), GROW_BASE, "a hero-tier enemy does not scale");
    assert_eq!(built_level(&f.session, "Squire"), GROW_BASE, "a guest does not scale");
    // the roster entry the battle builds from: by stage, and never for an ally
    let at = |stage: i64, allies: Vec<String>| WorldRoster { stage, allies, ..WorldRoster::new(&d, "fw") };
    assert_eq!(at(stage, Vec::new()).entry(&cat, "wolf").map(|u| i64::from(u.level)), Some(grown));
    assert_eq!(at(0, Vec::new()).entry(&cat, "wolf").map(|u| i64::from(u.level)), Some(GROW_BASE), "the host's roster: stage 0");
    assert_eq!(at(stage, vec!["wolf".into()]).entry(&cat, "wolf").map(|u| i64::from(u.level)), Some(GROW_BASE), "an ally never scales");
    // the XP weight follows the level fought
    let foes = battle::roster_at(&d, "fw", "fw:pack", stage);
    let seen: Vec<(&str, i64, i64)> = foes.iter().map(|f| (f.unit_ref.as_str(), f.level, f.weight)).collect();
    let want = [("wolf", grown, progression::kill_weight(&d, grown, "standard")), ("rival", GROW_BASE, progression::kill_weight(&d, GROW_BASE, "hero"))];
    assert_eq!(seen, want, "each foe's level and weight as the battle builds it");
    assert_ne!(want[0].2, progression::kill_weight(&d, GROW_BASE, "standard"), "the grown wolf weighs more than at its base");
    assert_eq!(battle::enemy_levels_at(&d, "fw", "fw:pack", stage), [(grown, "standard".to_string()), (GROW_BASE, "hero".to_string())]);
}
