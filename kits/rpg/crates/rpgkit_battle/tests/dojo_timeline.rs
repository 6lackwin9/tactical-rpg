//! RULESET1-G on a scratch catalog built here (no shipped data): the Gear Dojo under the timeline ruleset. Expectations
//! come from the rules: classic builds the heroes as `build_unit` (classic unchanged); the toggle rebuilds them with
//! `build_unit_in` under the deck rule, keeping their gear; a deck set in the Dojo fills the active slots and slots its
//! passives (RULESET1 9 D2, D3); the planner view (RULESET1_INTENT UI4) is deterministic (PL1), adds up its AP and Rest
//! gain (RS2) and makes one plan per intent of the ruleset; a hit's stagger follows P6 on the dummy's fixed Max Poise
//! (P2) and the bars refill at the next round (P3); landed Rest blocks pay their AP at the round's end (RS2). Slice 2
//! (review): a rebuild keeps the spent points and the pouch; a broken dummy loses its action (P7); Reset rebuilds under
//! timeline; the Rest AP is lost on a hero switch; the planner shows its scratch battle's AP; Rest matches by kind.
//! S2.2-S4: under pips a break queues `stagger`, so the dummy loses its next round's action.

use rpgkit_battle::dojo::{DojoSession, HEROES};
use rpgkit_battle::round_intent::{self, RoundIntent};
use rpgkit_data::{Catalog, Deck, STATS};
use rpgkit_rules::gear::{Equipment, Slot};
use rpgkit_rules::ruleset::{Ruleset, AP_COST, CLASSIC, DECK, POISE, ROUND_INTENT, TIMELINE};
use rpgkit_rules::stats::roundi;
use rpgkit_rules::build_unit;
use serde_json::json;
use std::sync::Arc;

const BASICS: [&str; 4] = ["move", "rest", "strike", "active_guard"];
/// The dummy row's fixed Max Poise (P2) and the heavy attack's stagger (their rows below).
const DUMMY_POISE: i32 = 40;
const HEAVY_STAGGER: i32 = 12;

fn catalog() -> Catalog {
    Catalog::from_value(&catalog_value())
}

/// The fixture catalog's tables (`catalog`'s source).
fn catalog_value() -> serde_json::Value {
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
    json!({
        "skills": skills,
        "passives": [{"id": "t_tough", "name": "Tough", "slot": "passive", "effects": [{"type": "stat", "stat": "max_poise", "value": 5}]}],
        "items": [{"id": "t_sword", "name": "Sword", "item_type": "WEAPON", "base_power": 5}, {"id": "t_potion", "name": "Potion", "item_type": "CONSUMABLE", "starter_stash": true}],
        "units": [unit(HEROES[0], "party", 30), unit(HEROES[1], "party", 30), unit("training_dummy", "enemy", DUMMY_POISE)],
    })
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

#[test]
fn a_rebuild_keeps_the_points_spent_here_and_the_pouch() {
    let mut s = dojo(None);
    assert!(s.add_to_pouch("t_potion"), "the starter stash's consumable goes to the pouch");
    let stat = STATS[0];
    s.begin_allocation();
    s.allocate(stat);
    s.allocate(stat);
    s.confirm_allocation();
    let (core, free, pouch) = (s.hero().stats.core.clone(), s.hero().stats.unallocated, s.hero().inventory.pouch.iter().map(|i| i.id.clone()).collect::<Vec<_>>());
    assert_eq!(pouch, ["t_potion"]);
    let kept = |s: &DojoSession, when: &str| {
        assert_eq!(s.hero().stats.core, core, "{when}: the spent points stay");
        assert_eq!(s.hero().stats.unallocated, free, "{when}: the unspent points too");
        assert_eq!(s.hero().inventory.pouch.iter().map(|i| i.id.clone()).collect::<Vec<_>>(), pouch, "{when}: the pouch stays");
    };
    s.set_ruleset(timeline());
    kept(&s, "the toggle to timeline");
    s.set_deck(Deck { active: vec!["t_jab".into()], ..Deck::default() });
    kept(&s, "a deck set here");
    s.set_ruleset(Ruleset::classic());
    kept(&s, "back to classic");
}

#[test]
fn a_broken_dummy_loses_its_action_until_the_next_round() {
    let mut s = dojo(Some(timeline()));
    s.toggle_dummy_windup();
    for _ in 0..10 {
        if s.dummy_poise.broken {
            break;
        }
        s.heal_dummy();
        s.use_skill(Some("t_heavy"));
    }
    assert!(s.dummy_poise.broken, "P7: heavy hits into the windup break the dummy");
    let hp = s.hero().hp;
    s.dummy_attack();
    let last = s.log.last().unwrap();
    assert_eq!((last.outcome.as_str(), s.hero().hp), ("BROKEN", hp), "the broken dummy's strike is cancelled");
    s.next_round();
    assert!(!s.dummy_poise.broken, "P3: the bar refills");
    s.dummy_attack();
    assert_eq!(s.log.last().unwrap().category, "ATTACK", "the next round it strikes again");
}

#[test]
fn reset_rebuilds_the_heroes_under_the_timeline_ruleset() {
    let mut s = dojo(Some(timeline()));
    s.set_deck(Deck { active: vec!["t_jab".into()], ..Deck::default() });
    s.dummy_poise.value = 1;
    s.reset();
    assert_eq!(s.ruleset, timeline(), "the toggle stays");
    assert!(s.heroes.iter().all(|h| h.deck_rule.is_some()), "every hero rebuilt under the deck rule");
    assert!(s.deck_of(0).is_empty(), "the deck set here is gone (the baseline's)");
    assert_eq!((s.dummy_poise.value, s.dummy_poise.max), (DUMMY_POISE, DUMMY_POISE), "the bars refill");
    assert!(s.log.last().is_some_and(|e| e.title == "Dojo Reset"));
}

#[test]
fn the_rest_ap_belongs_to_the_hero_that_rested_and_is_lost_on_a_switch() {
    let (mut a, mut b) = (dojo(Some(timeline())), dojo(Some(timeline())));
    let rest = a.cat.skills.iter().find(|k| k.id == "rest").unwrap().clone();
    let cost = a.skill_ap(&rest);
    a.use_skill(Some("rest"));
    assert!(a.rest_blocks > 0);
    a.set_active(1);
    b.set_active(1);
    assert_eq!(a.rest_blocks, 0, "the next hero starts with no Rest blocks");
    a.next_round();
    b.next_round();
    assert_eq!(a.heroes[1].ap, b.heroes[1].ap, "the other hero gains no Rest AP");
    assert_eq!(a.heroes[0].ap, b.heroes[0].ap - cost, "the hero that rested paid and gained nothing");
}

#[test]
fn the_planner_shows_the_ap_its_scratch_battle_plans_with() {
    let mut s = dojo(Some(timeline()));
    let before = s.plan_view().ap_before;
    s.heroes[0].ap = before - 3;
    let v = s.plan_view();
    assert_eq!(v.ap_before, before, "the scratch battle's round-1 AP, not the hero's AP now");
    assert_eq!(v.ap_after, v.ap_before - v.rows.iter().map(|r| r.ap).sum::<i32>());
}

#[test]
fn rest_is_matched_by_its_kind_only() {
    let mut s = dojo(Some(timeline()));
    let ap = s.hero().ap;
    s.use_skill(Some("move"));
    assert_eq!(s.rest_blocks, 0, "a MOVE used in the Dojo books no Rest block");
    let mv = s.cat.skills.iter().find(|k| k.id == "move").unwrap().clone();
    assert_eq!(s.hero().ap, ap - s.skill_ap(&mv));
}

/// S2.2-S4 (FB-49d): under `poise_mode` pips the Dojo's break also queues `stagger`, so after "Next round" the bar is
/// full again and the dummy, staggered, loses its action (under bar it strikes again: the test above).
#[test]
fn a_pip_break_staggers_the_dummy_next_round() {
    let mut row = json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, AP_COST, POISE, DECK], "basics": BASICS, "deck_slots": {"active": 2, "passive": 1, "keystone": 1}});
    row["poise_mode"] = json!("pips");
    let mut s = dojo(Some(Ruleset::from_row(row.as_object().unwrap())));
    s.toggle_dummy_windup();
    for _ in 0..10 {
        if s.dummy_poise.broken {
            break;
        }
        s.heal_dummy();
        s.use_skill(Some("t_heavy"));
    }
    assert!(s.dummy_poise.broken, "heavy hits into the windup break the dummy");
    assert!(s.dummy.st.pending.contains(rpgkit_battle::poise::BREAK_STATUS), "stagger queued");
    s.next_round();
    assert!(!s.dummy_poise.broken && s.dummy_poise.pips == s.ruleset.numbers.poise_pips, "the pips refill");
    let hp = s.hero().hp;
    s.dummy_attack();
    let shown = rpgkit_rules::status::display_name(rpgkit_battle::poise::BREAK_STATUS).to_uppercase();
    assert_eq!((s.log.last().unwrap().outcome.as_str(), s.hero().hp), (shown.as_str(), hp), "staggered: no strike");
}

/// AUDIT-03: the Dojo builds its heroes under its starting (classic) row's wield terms (`Numbers::wield`), not the
/// defaults: a catalog classic row with its own untrained terms reaches every hero's proficiency.
#[test]
fn the_dojo_builds_its_heroes_under_its_rulesets_wield_terms() {
    let mut raw = catalog_value();
    raw["rulesets"] = json!([{"id": CLASSIC, "name": "Classic", "rules": [], "untrained_power_mult": 0.5, "untrained_burden_add": 3, "req_discount_per_level": 1}]);
    let cat = Catalog::from_value(&raw);
    let want = Ruleset::of_or_classic(&cat, CLASSIC).numbers.wield();
    assert_ne!(want, rpgkit_rules::stats::WieldRule::default(), "the fixture row sets its own terms");
    let s = DojoSession::new(Arc::new(cat));
    assert_eq!(s.heroes.len(), HEROES.len());
    for h in &s.heroes {
        assert_eq!(h.stats.proficiency.wield, want, "{}", h.name);
    }
}
