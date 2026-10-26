//! RULESET1-D on a scratch catalog built here (no shipped data): the deck rules of RULESET1 9 (D1-D5) under the rule
//! `deck`. Expectations come from the rules and the fixture's own rows: the ruleset's `basics` are open to every unit
//! without a slot and no other basic is usable (D1); the deck's actives, else the first class-path families, fill the
//! ruleset's active slots (D2); a slottable passive applies only while slotted, a catalog unit owning the ones its deck
//! names (D3, D4); the order lists and the auto order (D5, intent FU4); a party member built outside the battle gets the
//! rule at the start. Classic builds every unit as before.

use indexmap::IndexMap;
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::poise;
use rpgkit_battle::round_intent::RoundIntent;
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::{EncounterDef, Prebuilt, Session};
use rpgkit_battle::world::{AiProfile, UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, DECK, POISE, ROUND_INTENT, TIMELINE};
use rpgkit_rules::{build_unit, Unit};
use serde_json::{json, Value};
use std::sync::Arc;

/// The class path, in path order (every family unlocks at level 1).
const PATH: [&str; 5] = ["d_one", "d_two", "d_three", "d_four", "d_ward"];
/// The test ruleset's basics, in their order.
const BASICS: [&str; 5] = ["move", "rest", "strike", "active_guard", "flank_cut"];
/// The Max Poise the slotted passive `d_braced` adds (its row below).
const BRACED: i32 = 20;

const AUTO: UnitId = 0;
const DECKED: UnitId = 1;
const FOE: UnitId = 2;

/// Engine skills, the two timeline basics that are not catalog basics (`active_guard`, `flank_cut`), the class path
/// (four attacks and a guard skill); passives of each slot kind; a class-derived hero without a deck, one with a full
/// deck and order lists, a fixed foe.
fn catalog() -> Catalog {
    let atk = |id: &str| json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.5, "target_mode": "ENEMY"});
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "active_guard", "name": "Active Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "mitigation": 0.5}),
        json!({"id": "flank_cut", "name": "Flank Cut", "kind": "ATTACK", "round_intent": "maneuver", "level": 1, "blocks": 1, "dash_tiles": 2, "target_mode": "ENEMY"}),
        atk("d_one"),
        atk("d_two"),
        atk("d_three"),
        atk("d_four"),
        json!({"id": "d_ward", "name": "d_ward", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.3}),
    ];
    let passives = json!([
        {"id": "d_braced", "name": "Braced", "slot": "passive", "effects": [{"type": "stat", "stat": "max_poise", "value": BRACED}]},
        {"id": "d_spare", "name": "Spare", "slot": "passive", "effects": [{"type": "stat", "stat": "max_poise", "value": 5}]},
        {"id": "d_anvil", "name": "Anvil", "slot": "keystone", "effects": [{"type": "stagger_taken", "mult": 0.0, "max_blocks": 1}]}
    ]);
    let path: serde_json::Map<String, Value> = PATH.iter().map(|f| (f.to_string(), json!(1))).collect();
    let hero = |id: &str, deck: Value| json!({"id": id, "name": id, "side": "party", "class_id": "drill", "level": 1, "stat_mode": "derived", "deck": deck});
    Catalog::from_value(&json!({
        "skills": skills,
        "passives": passives,
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10}, "skill_path": path}],
        "units": [
            hero("d_auto", json!({})),
            hero("d_decked", json!({"active": ["d_four", "d_ward", "d_two"], "passive": ["d_braced", "d_spare"], "keystone": ["d_anvil"],
                "order": {"assault": ["d_two", "strike", "d_four"]}})),
            {"id": "d_foe", "name": "d_foe", "side": "enemy", "stat_mode": "fixed", "hp": 300, "speed": 8, "attack_range": 1, "skill_cap": 5}
        ],
        "derived_stats": [{"id": "max_poise", "name": "Max Poise", "formula": "CONS * per_cons", "constants": {"per_cons": 2}, "min": 1, "max": 200, "rulesets": ["timeline"]}]
    }))
}

/// A timeline ruleset with `rules`, the test basics and the deck slots `slots`.
fn ruleset(rules: &[&str], slots: Value) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules, "basics": BASICS, "deck_slots": slots, "poise_stat": "max_poise"}).as_object().unwrap())
}

fn slots() -> Value {
    json!({"active": 3, "passive": 2, "keystone": 1})
}

fn deck_rules() -> Ruleset {
    ruleset(&[ROUND_INTENT, DECK, POISE], slots())
}

fn session_with(rules: Ruleset, party: Vec<Prebuilt>) -> Session {
    let roster = if party.is_empty() { json!(["d_auto", "d_decked"]) } else { json!([]) };
    let row = json!({"id": "d_fight", "name": "Fight", "party": roster, "enemies": ["d_foe"], "total_blocks": 6, "unit_ap": 10, "max_rounds": 8});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = rules;
    Session::start_with(Arc::new(catalog()), def, &IndexMap::new(), party)
}

fn session(rules: Ruleset) -> Session {
    session_with(rules, Vec::new())
}

fn usable(w: &World, c: UnitId, id: &str) -> bool {
    let s = w.cat.skill(id).unwrap_or_else(|| panic!("skill {id}"));
    w.units[c].can_use(&w.cat, s)
}

fn strings(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

/// The unit a classic battle would field for catalog row `id`.
fn classic_unit(id: &str) -> Unit {
    let cat = catalog();
    build_unit(&cat, &cat.units[id], 10)
}

/// D1: the ruleset's basics are open to every unit (no slot, never gated), every other basic is closed; classic keeps
/// the catalog's basics and never owns the timeline-only ones.
#[test]
fn the_rulesets_basics_are_open_and_every_other_basic_is_closed() {
    let s = session(deck_rules());
    for c in [AUTO, DECKED, FOE] {
        for b in BASICS {
            assert!(usable(&s.w, c, b), "{b} open to unit {c}");
        }
        for closed in ["guard", "switch_target"] {
            assert!(!usable(&s.w, c, closed), "{closed} closed to unit {c}");
        }
    }
    // the basics come from the row: one without `strike` closes it
    let mut fewer = deck_rules();
    fewer.numbers.basics.retain(|b| b != "strike");
    let s = session(fewer);
    assert!(!usable(&s.w, AUTO, "strike") && usable(&s.w, AUTO, "rest"));
    let s = session(Ruleset::classic());
    for c in [AUTO, DECKED] {
        assert!(usable(&s.w, c, "guard") && usable(&s.w, c, "strike"), "classic basics for unit {c}");
        assert!(!usable(&s.w, c, "active_guard") && !usable(&s.w, c, "flank_cut"), "no timeline basic in classic for unit {c}");
    }
}

/// D1 through the planner: under the deck rule `add_step` takes Active Guard and refuses Guard; classic the reverse.
#[test]
fn the_planner_takes_open_basics_and_refuses_closed_ones() {
    for (rules, open, closed) in [(ruleset(&[DECK], slots()), "active_guard", "guard"), (Ruleset::classic(), "guard", "active_guard")] {
        let s = session(rules);
        let w = &s.w;
        let mut plan = Plan::new();
        assert_eq!(plan.add_step(w, AUTO, PlanStep::self_skill(w.book().find(open).unwrap()), true), "", "{open}");
        let mut plan = Plan::new();
        assert_ne!(plan.add_step(w, AUTO, PlanStep::self_skill(w.book().find(closed).unwrap()), true), "", "{closed}");
    }
}

/// D2: without a deck the first class-path families (today's default loadout, in its order) fill the active slots;
/// a deck's actives fill them in the deck's order, whatever the path order; the rest of the path is not usable.
#[test]
fn the_deck_or_the_class_path_fills_the_active_slots() {
    let n = deck_rules().numbers.deck_slots.active as usize;
    let classic = classic_unit("d_auto").active_families();
    assert_eq!(classic, strings(&PATH), "classic: every path family is active");
    let s = session(deck_rules());
    assert_eq!(s.w.units[AUTO].active_families(), classic[..n].to_vec(), "auto: the first {n}");
    for fam in &classic[n..] {
        assert!(!usable(&s.w, AUTO, fam), "{fam} beyond the slots");
    }
    let deck = &catalog().units["d_decked"].deck.active;
    assert_eq!(&s.w.units[DECKED].active_families(), deck, "the deck's actives in the deck's order");
    for fam in PATH.iter().filter(|f| !deck.iter().any(|d| d == *f)) {
        assert!(!usable(&s.w, DECKED, fam), "{fam} not in the deck");
    }
    // the ruleset's count caps the deck
    let s = session(ruleset(&[DECK], json!({"active": 2, "passive": 2, "keystone": 1})));
    assert_eq!(s.w.units[DECKED].active_families(), deck[..2].to_vec());
    // classic: the deck is ignored
    let s = session(Ruleset::classic());
    assert_eq!(s.w.units[DECKED].active_families(), classic_unit("d_decked").active_families());
}

/// D3, D4: a catalog unit owns the slottable passives its deck names and they apply while slotted (within the slot
/// counts); without the deck rule a catalog unit gets none, as today.
#[test]
fn slotted_passives_apply_only_under_the_deck_rule() {
    let s = session(deck_rules());
    let u = &s.w.units[DECKED];
    assert_eq!(u.passives, strings(&["d_braced", "d_spare", "d_anvil"]));
    assert_eq!(u.stagger_taken.len(), 1, "the keystone's stagger_taken effect");
    assert!(s.w.units[AUTO].passives.is_empty(), "no deck, no passives");
    // the slot counts: one passive slot, no keystone slot
    let s = session(ruleset(&[DECK, POISE], json!({"active": 3, "passive": 1, "keystone": 0})));
    assert_eq!(s.w.units[DECKED].passives, strings(&["d_braced"]));
    assert!(s.w.units[DECKED].stagger_taken.is_empty());
    // Max Poise with the slotted passive: the hero without passives plus the passive's value (same class and level)
    assert_eq!(poise::max_poise(&s.w, DECKED), poise::max_poise(&s.w, AUTO) + BRACED);
    // poise without the deck rule: the deck's passives do not apply
    let s = session(ruleset(&[POISE], slots()));
    assert!(s.w.units[DECKED].passives.is_empty() && s.w.units[DECKED].stagger_taken.is_empty());
    assert_eq!(poise::max_poise(&s.w, DECKED), poise::max_poise(&s.w, AUTO));
    let s = session(Ruleset::classic());
    assert!(s.w.units[DECKED].passives.is_empty());
}

/// D3: a passive granted from outside the deck (a source, `apply_passive`) passes the slot gate only when it is
/// source-granted (`slot` "") or slotted.
#[test]
fn the_slot_gate_holds_for_granted_passives() {
    let cat = catalog();
    let s = session(ruleset(&[DECK], json!({"active": 3, "passive": 1, "keystone": 1})));
    let mut u = s.w.units[AUTO].clone();
    let innate = rpgkit_data::PassiveDef { id: "d_innate".into(), name: "Innate".into(), effects: Vec::new(), slot: String::new() };
    u.apply_passive(&cat, &innate);
    u.apply_passive(&cat, &cat.passives["d_braced"]);
    assert_eq!(u.passives, strings(&["d_innate"]), "the unslotted d_braced stays off");
    u.deck.passive = strings(&["d_braced"]);
    u.apply_passive(&cat, &cat.passives["d_braced"]);
    assert_eq!(u.passives, strings(&["d_innate", "d_braced"]));
    let mut classic = classic_unit("d_auto");
    classic.apply_passive(&cat, &cat.passives["d_spare"]);
    assert_eq!(classic.passives, strings(&["d_spare"]), "classic applies every passive");
}

/// D5, FU4: a deck's order list for an intent is the planner's order; without one, the deck's actives of the intent
/// in slot order, then the ruleset's basics of the intent in their order.
#[test]
fn the_order_list_else_the_auto_order() {
    let s = session(deck_rules());
    let w = &s.w;
    let ids = |c: UnitId, i: RoundIntent| rip::order(w, c, i).into_iter().map(|sid| w.cat.skills[sid].id.clone()).collect::<Vec<_>>();
    assert_eq!(ids(DECKED, RoundIntent::Assault), strings(&["d_two", "strike", "d_four"]), "the written order");
    assert_eq!(ids(DECKED, RoundIntent::Guard), strings(&["d_ward", "active_guard"]), "auto: the guard active, then the guard basic");
    assert_eq!(ids(AUTO, RoundIntent::Assault), strings(&["d_one", "d_two", "d_three", "strike"]), "auto: the slotted actives, then strike");
    assert_eq!(ids(AUTO, RoundIntent::Maneuver), strings(&["flank_cut"]));
    assert_eq!(ids(AUTO, RoundIntent::Guard), strings(&["active_guard"]), "d_ward is beyond the auto slots");
}

/// A party member built outside the battle without the rule (its passives are its builder's) gets the open basics and
/// its active slots at the start; under classic it is fielded as built.
#[test]
fn a_prebuilt_member_gets_the_rule_at_the_start() {
    let cat = catalog();
    let prebuilt = |id: &str| Prebuilt { unit: classic_unit(id), art: String::new(), palette: String::new(), profile: AiProfile::for_unit(&cat, &cat.units[id]), key: id.into() };
    let s = session_with(deck_rules(), vec![prebuilt("d_decked")]);
    let u = &s.w.units[0];
    assert!(u.deck_rule.is_some());
    assert_eq!(u.active_families(), cat.units["d_decked"].deck.active);
    assert!(usable(&s.w, 0, "active_guard") && !usable(&s.w, 0, "guard"));
    let s = session_with(Ruleset::classic(), vec![prebuilt("d_decked")]);
    let u = &s.w.units[0];
    assert!(u.deck_rule.is_none());
    assert_eq!(u.active_families(), classic_unit("d_decked").active_families());
}

/// Classic builds every unit exactly as `build_unit` (no deck rule, the catalog's loadout and basics, no passives).
#[test]
fn classic_units_are_built_as_before() {
    let s = session(Ruleset::classic());
    for (c, id) in [(AUTO, "d_auto"), (DECKED, "d_decked")] {
        let (u, want) = (&s.w.units[c], classic_unit(id));
        assert!(u.deck_rule.is_none());
        assert_eq!((&u.known_skills, u.active_families(), &u.passives, u.max_hp), (&want.known_skills, want.active_families(), &want.passives, want.max_hp), "{id}");
    }
}
