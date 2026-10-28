//! RULESET1-G on a scratch catalog built here (no shipped data): the Gear Dojo under the timeline ruleset. Expectations
//! come from the rules: classic builds the heroes as `build_unit` (classic unchanged); the toggle rebuilds them with
//! `build_unit_in` under the deck rule, keeping their gear; a deck set in the Dojo fills the active slots and slots its
//! passives (RULESET1 9 D2, D3); the planner view (RULESET1_INTENT UI4) is deterministic (PL1), adds up its AP and Rest
//! gain (RS2) and makes one plan per intent of the ruleset; a hit's stagger follows P6 on the dummy's fixed Max Poise
//! (P2) and the bars refill at the next round (P3); landed Rest blocks pay their AP at the round's end (RS2).

use rpgkit_battle::dojo::{DojoSession, HEROES};
use rpgkit_battle::round_intent::{self, RoundIntent};
use rpgkit_data::{Catalog, Deck};
use rpgkit_rules::gear::{Equipment, Slot};
use rpgkit_rules::ruleset::{Ruleset, AP_COST, DECK, POISE, ROUND_INTENT, TIMELINE};
use rpgkit_rules::stats::roundi;
use rpgkit_rules::build_unit;
use serde_json::json;
use std::sync::Arc;

const BASICS: [&str; 4] = ["move", "rest", "strike", "active_guard"];
/// The dummy row's fixed Max Poise (P2) and the heavy attack's stagger (their rows below).
const DUMMY_POISE: i32 = 40;
const HEAVY_STAGGER: i32 = 12;

fn catalog() -> Catalog {
    let atk = |id: &str, blocks: i32, ap: i32, stagger: i32| json!({"id": id, "name": id, "kind": "ATTACK", "round_intent": "assault", "level": 1, "blocks": blocks, "ap_cost": ap, "power_mult": 1.0, "stagger": stagger, "target_mode": "ENEMY"});
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "active_guard", "name": "Active Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 1, "mitigation": 0.5}),
        atk("t_heavy", 2, 3, HEAVY_STAGGER),
        atk("t_jab", 1, 1, 4),
        atk("t_lunge", 1, 2, 6),
        json!({"id": "t_brace", "name": "t_brace", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 2, "mitigation": 0.3}),
    ];
    let unit = |id: &str, side: &str, poise: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": 1, "skill_cap": 5, "poise": poise});
    Catalog::from_value(&json!({
        "skills": skills,
        "passives": [{"id": "t_tough", "name": "Tough", "slot": "passive", "effects": [{"type": "stat", "stat": "max_poise", "value": 5}]}],
        "items": [{"id": "t_sword", "name": "Sword", "item_type": "WEAPON", "base_power": 5}],
        "units": [unit(HEROES[0], "party", 30), unit(HEROES[1], "party", 30), unit("training_dummy", "enemy", DUMMY_POISE)],
    }))
}

/// The timeline ruleset of the fixture: the round intent, AP by `ap_cost`, poise and the deck (2 actives, 1 passive).
fn timeline() -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, AP_COST, POISE, DECK], "basics": BASICS, "deck_slots": {"active": 2, "passive": 1, "keystone": 1}}).as_object().unwrap())
}

fn dojo(rules: Option<Ruleset>) -> DojoSession {
    let mut s = DojoSession::new(Arc::new(catalog()));
    if let Some(r) = rules {
        s.set_ruleset(r);
    }
    s
}

#[test]
fn classic_builds_the_heroes_as_before_and_the_toggle_rebuilds_them_under_the_deck_rule() {
    let cat = catalog();
    let mut s = dojo(None);
    for (i, id) in HEROES.iter().enumerate() {
        let plain = build_unit(&cat, cat.units.get(*id).unwrap(), 0);
        assert_eq!((s.heroes[i].active_families(), s.heroes[i].passives.clone(), s.heroes[i].deck_rule.is_some()), (plain.active_families(), plain.passives.clone(), false), "{id}: classic = build_unit");
    }
    let sword = cat.items.get("t_sword").unwrap().clone();
    s.heroes[0].inventory.equip(Equipment::new(&sword), Slot::MainHand);
    s.set_ruleset(timeline());
    assert!(s.heroes.iter().all(|h| h.deck_rule.is_some()), "timeline: every hero carries the deck rule");
    assert_eq!(s.heroes[0].inventory.get(Slot::MainHand).map(|e| e.item.id.clone()), Some("t_sword".into()), "the gear stays");
    s.set_ruleset(Ruleset::classic());
    let plain = build_unit(&cat, cat.units.get(HEROES[0]).unwrap(), 0);
    assert_eq!((s.heroes[0].active_families(), s.heroes[0].deck_rule.is_some()), (plain.active_families(), false), "back to classic: build_unit again");
    assert!(s.heroes[0].inventory.get(Slot::MainHand).is_some());
}

#[test]
fn a_deck_set_in_the_dojo_fills_the_active_slots_and_slots_its_passive() {
    let mut s = dojo(Some(timeline()));
    let deck = Deck { active: vec!["t_jab".into()], passive: vec!["t_tough".into()], ..Deck::default() };
    s.set_deck(deck.clone());
    assert_eq!(s.hero().active_families(), deck.active, "D2: the deck's actives fill the slots");
    assert!(s.hero().passives.iter().any(|p| p == "t_tough"), "D3, D4: the named passive is slotted");
    assert_eq!(s.deck_of(s.active), deck);
    s.set_deck(Deck::default());
    assert!(!s.hero().passives.iter().any(|p| p == "t_tough"), "the automatic deck slots no passive");
    assert!(s.log.last().is_some_and(|e| e.title == "Deck"));
}

#[test]
fn the_planner_view_is_deterministic_adds_up_and_makes_one_plan_per_intent() {
    let s = dojo(Some(timeline()));
    let v = s.plan_view();
    assert!(v.refused.is_empty(), "Assault on the dummy is offered: {}", v.refused);
    assert!(!v.rows.is_empty());
    assert_eq!(v.target, s.dummy.name);
    assert_eq!(v.ap_after, v.ap_before - v.rows.iter().map(|r| r.ap).sum::<i32>(), "AP after = before - the commands' AP");
    assert_eq!(v.rest_ap, round_intent::rest_gain(&s.ruleset.numbers, v.rest_blocks), "RS2");
    assert_eq!(v.rest_poise, s.ruleset.numbers.rest_poise_per_block * v.rest_blocks, "RS3");
    assert_eq!(s.plan_view(), v, "PL1: the same inputs, the same plan");
    let intents: Vec<String> = s.ruleset.numbers.intents.iter().filter_map(|i| RoundIntent::parse(i)).map(|i| i.name().to_string()).collect();
    assert_eq!(s.plans_by_intent().iter().map(|p| p.intent.clone()).collect::<Vec<_>>(), intents, "one plan per intent");
    let classic = dojo(None);
    assert!(!classic.plan_view().refused.is_empty() && classic.plan_view().rows.is_empty() && classic.plans_by_intent().is_empty(), "classic plans nothing");
}

#[test]
fn a_hit_into_the_dummys_windup_staggers_by_p6_and_the_next_round_refills_the_bars() {
    let mut s = dojo(Some(timeline()));
    assert_eq!(s.dummy_poise.max, DUMMY_POISE, "P2: the row's fixed Max Poise");
    s.toggle_dummy_windup();
    s.toggle_dummy_defense();
    s.use_skill(Some("t_heavy"));
    let n = &s.ruleset.numbers;
    let taken = roundi(HEAVY_STAGGER as f64 * n.stagger_windup_mult * n.stagger_guard_mult);
    let after = (DUMMY_POISE - taken).max(0);
    assert_eq!(s.dummy_poise.value, after, "P6: stagger x windup x guard");
    let hit = s.log.iter().rev().find(|e| e.title == "t_heavy").expect("the hit's entry");
    assert!(hit.breakdown.contains(&format!("stagger {HEAVY_STAGGER} x windup")) && hit.breakdown.contains(&format!("= {taken} -> poise {DUMMY_POISE} -> {after}")), "{}", hit.breakdown);
    s.next_round();
    assert_eq!(s.dummy_poise.value, s.dummy_poise.max, "P3: the bar refills");
    let classic = dojo(None);
    assert_eq!(classic.dummy_poise.max, 0, "classic has no bars");
}

#[test]
fn rest_blocks_pay_their_ap_at_the_rounds_end() {
    let (mut a, mut b) = (dojo(Some(timeline())), dojo(Some(timeline())));
    a.heroes[0].ap = 6;
    b.heroes[0].ap = 6;
    let rest = a.cat.skills.iter().find(|k| k.id == "rest").unwrap().clone();
    let cost = a.skill_ap(&rest);
    a.use_skill(Some("rest"));
    a.use_skill(Some("rest"));
    assert_eq!(a.hero().ap, 6 - 2 * cost, "RS2: no AP when the Rest lands");
    let gain = round_intent::rest_gain(&a.ruleset.numbers, 2 * rest.blocks.max(1));
    a.next_round();
    b.next_round();
    assert_eq!(a.hero().ap, b.hero().ap - 2 * cost + gain, "the Rest gain comes at the round's end");
}
