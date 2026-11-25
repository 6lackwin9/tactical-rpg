//! PR5-50 (`KIT-HOOKS-CLEANUP`, FB-204) on a scratch catalog built here (no shipped data): the skill-scoped reach bonus
//! `skill_reach {reach, intent?, kind?}` replaces the spell-only `spell_reach`. A skill takes a bonus when every filter
//! given matches (`kind` the skill's kind or `SPELL`, the spell group; `intent` one of its derived or legacy intents);
//! worn bonuses sum to at most `GEAR_REACH_CAP`, passive ones sum on top; a shape drawn from the user's own tile takes
//! none. The retired `spell_reach {value}` reads exactly as `skill_reach {reach: value, kind: SPELL}` (grants, unit,
//! band and budget). Expectations come from the rules (the skill's own band plus the matching bonus).

use rpgkit_battle::skill;
use rpgkit_data::{Catalog, PassiveDef, SkillReach, SKILL_REACH, SKILL_REACH_SPELL, SPELL_REACH_ALIAS};
use rpgkit_rules::gear::{Equipment, Slot};
use rpgkit_rules::unit::{skill_reach_matches, GEAR_REACH_CAP};
use rpgkit_rules::{budget, build_unit, grants, Unit};
use serde_json::{json, Value};

/// A spell bolt (spell group, 2-4), a plain cut (melee), a shaped burst (IR21: Field and Assault, 1-3), a shaped spell
/// burst (spell and Field, 1-3), an ally mend (Support, 1-2) and a spell line (drawn from the user's tile).
const BOLT: &str = "t_bolt";
const CUT: &str = "t_cut";
const BURST: &str = "t_burst";
const SPELL_BURST: &str = "t_spell_burst";
const MEND: &str = "t_mend";
const LINE: &str = "t_spell_line";
const SKILLS: [&str; 6] = [BOLT, CUT, BURST, SPELL_BURST, MEND, LINE];

fn reach(fx: Value) -> Value {
    let mut row = json!({"type": SKILL_REACH});
    for (k, v) in fx.as_object().unwrap() {
        row[k] = v.clone();
    }
    row
}

fn catalog() -> Catalog {
    let attack = |id: &str, extra: Value| {
        let mut row = json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0, "target_mode": "ENEMY"});
        for (k, v) in extra.as_object().unwrap() {
            row[k] = v.clone();
        }
        row
    };
    let passive = |id: &str, fx: Value| json!({"id": id, "name": id, "slot": "", "effects": [fx]});
    let focus = |id: &str, slot: &str, fx: Value| json!({"id": id, "name": id, "item_type": "ACCESSORY", "slot": slot, "effects": [fx]});
    Catalog::from_value(&json!({
        "skills": [
            attack(BOLT, json!({"spell_source": true, "reach_min": 2, "reach_max": 4})),
            attack(CUT, json!({})),
            attack(BURST, json!({"area": "BURST", "reach_min": 1, "reach_max": 3})),
            attack(SPELL_BURST, json!({"area": "BURST", "reach_min": 1, "reach_max": 3, "spell_source": true})),
            {"id": MEND, "name": MEND, "kind": "SUPPORT", "level": 1, "blocks": 1, "heal_base": 5, "target_mode": "ALLY", "reach_min": 1, "reach_max": 2},
            attack(LINE, json!({"area": "LINE", "spell_source": true})),
        ],
        "passives": [
            passive("p_kind", reach(json!({"reach": 1, "kind": SKILL_REACH_SPELL}))),
            passive("p_intent", reach(json!({"reach": 1, "intent": "field"}))),
            passive("p_both", reach(json!({"reach": 1, "intent": "field", "kind": SKILL_REACH_SPELL}))),
            passive("p_none", reach(json!({"reach": 1}))),
            passive("p_support", reach(json!({"reach": 2, "intent": "support", "kind": "SUPPORT"}))),
            passive("p_old", json!({"type": SPELL_REACH_ALIAS, "value": 1})),
        ],
        "items": [
            focus("t_lens", "ACCESSORY_1", reach(json!({"reach": 2, "kind": SKILL_REACH_SPELL}))),
            focus("t_charm", "ACCESSORY_2", reach(json!({"reach": 1, "kind": SKILL_REACH_SPELL}))),
            focus("t_old_charm", "ACCESSORY_2", json!({"type": SPELL_REACH_ALIAS, "value": 1})),
        ],
        "units": [{"id": "t_adept", "name": "Adept", "side": "party", "stat_mode": "fixed", "hp": 100, "speed": 12, "attack_range": 1, "skill_cap": 5}]
    }))
}

/// The adept, bare, with the passives `held` applied.
fn adept(cat: &Catalog, held: &[&str]) -> Unit {
    let mut u = build_unit(cat, &cat.units["t_adept"], 0);
    for slot in Slot::ALL {
        u.inventory.unequip(slot);
    }
    u.sync_from_stats_and_gear(cat);
    for p in held {
        let def: &PassiveDef = &cat.passives[*p];
        u.apply_passive(cat, def);
    }
    u
}

fn wear(cat: &Catalog, u: &mut Unit, item: &str, slot: Slot) {
    u.inventory.equip(Equipment::new(&cat.items[item]), slot);
    u.sync_from_stats_and_gear(cat);
}

/// Each skill's bonus for `u`, in SKILLS order.
fn bonuses(cat: &Catalog, u: &Unit) -> Vec<i32> {
    SKILLS.iter().map(|s| u.skill_reach_bonus(cat.skill(s).unwrap())).collect()
}

/// Each skill's band for `u`, in SKILLS order.
fn bands(cat: &Catalog, u: &Unit) -> Vec<(i32, i32)> {
    SKILLS.iter().map(|s| skill::reach_for(cat.skill(s).unwrap(), u)).collect()
}

/// SKILLS order: bolt, cut, burst, spell burst, mend, line.
#[test]
fn a_kind_filter_reaches_only_that_kind() {
    let cat = catalog();
    assert_eq!(bonuses(&cat, &adept(&cat, &["p_kind"])), [1, 0, 0, 1, 0, 1], "SPELL: the spell group (an ATTACK with a spell source)");
    assert_eq!(bonuses(&cat, &adept(&cat, &["p_support"])), [0, 0, 0, 0, 2, 0], "SUPPORT and support: the mend");
}

#[test]
fn an_intent_filter_reaches_the_skills_offered_in_that_intent() {
    let cat = catalog();
    // IR21 derives Field (and Assault) for a burst with reach above 1; the bolt, cut, mend and the line's own band not
    assert_eq!(bonuses(&cat, &adept(&cat, &["p_intent"])), [0, 0, 1, 1, 0, 1]);
    for s in SKILLS {
        let def = cat.skill(s).unwrap();
        let derived = rpgkit_rules::intent::derive(&rpgkit_rules::intent::IntentInput::of_def(def));
        let r = SkillReach { reach: 1, intent: "field".into(), kind: String::new(), ranged: None };
        assert_eq!(skill_reach_matches(&r, def), derived.contains(&"field"), "{s}: the field filter follows IR21");
    }
}

#[test]
fn both_filters_must_match_and_no_filter_matches_every_skill() {
    let cat = catalog();
    assert_eq!(bonuses(&cat, &adept(&cat, &["p_both"])), [0, 0, 0, 1, 0, 1], "field and SPELL: the spell burst (and the line, filtered by its shape later)");
    assert_eq!(bonuses(&cat, &adept(&cat, &["p_none"])), [1; 6], "no filter: every skill");
    assert_eq!(bonuses(&cat, &adept(&cat, &["p_kind", "p_intent", "p_none"])), [2, 1, 2, 3, 1, 3], "passive bonuses sum");
}

#[test]
fn the_band_takes_the_bonus_except_a_shape_drawn_from_the_users_tile() {
    let cat = catalog();
    let bare = bands(&cat, &adept(&cat, &[]));
    assert_eq!(bare, [(2, 4), (1, 1), (1, 3), (1, 3), (1, 2), (1, 1)], "the skills' own bands");
    let held = adept(&cat, &["p_none"]);
    let widened: Vec<(i32, i32)> = bare.iter().enumerate().map(|(i, &(lo, hi))| if SKILLS[i] == LINE { (lo, hi) } else { (lo, hi + 1) }).collect();
    assert_eq!(bands(&cat, &held), widened, "every far edge one further; the line stays (1, 1)");
}

#[test]
fn worn_bonuses_are_capped_and_passives_add_on_top() {
    let cat = catalog();
    let mut u = adept(&cat, &[]);
    wear(&cat, &mut u, "t_lens", Slot::Accessory1);
    wear(&cat, &mut u, "t_charm", Slot::Accessory2);
    assert_eq!(u.gear_skill_reach.iter().map(|r| r.reach).sum::<i32>(), 3, "both worn bonuses held");
    assert_eq!(u.skill_reach_bonus(cat.skill(BOLT).unwrap()), GEAR_REACH_CAP, "worn reach is capped");
    let def: &PassiveDef = &cat.passives["p_kind"];
    u.apply_passive(&cat, def);
    assert_eq!(u.skill_reach_bonus(cat.skill(BOLT).unwrap()), GEAR_REACH_CAP + 1, "a passive adds on top of the cap");
    u.inventory.unequip(Slot::Accessory1);
    u.inventory.unequip(Slot::Accessory2);
    u.sync_from_stats_and_gear(&cat);
    assert!(u.gear_skill_reach.is_empty(), "a sync takes the worn bonuses off");
    assert_eq!(u.skill_reach_bonus(cat.skill(BOLT).unwrap()), 1);
}

#[test]
fn the_spell_reach_alias_reads_as_skill_reach_of_kind_spell() {
    let cat = catalog();
    let old = json!({"type": SPELL_REACH_ALIAS, "value": 1});
    let new = reach(json!({"reach": 1, "kind": SKILL_REACH_SPELL}));
    assert_eq!(SkillReach::from_effect(old.as_object().unwrap()), SkillReach::from_effect(new.as_object().unwrap()));
    assert_eq!(grants::read(std::slice::from_ref(&old)), grants::read(std::slice::from_ref(&new)), "the same grant");
    assert_eq!(budget::effect_value(&old), budget::effect_value(&new), "priced the same");
    assert_eq!(budget::effect_value(&new), budget::W_SPELL_REACH);
    // a passive and a worn item in the old form give the unit exactly the new form's bonuses
    let (a, b) = (adept(&cat, &["p_old"]), adept(&cat, &["p_kind"]));
    assert_eq!(a.passive_skill_reach, b.passive_skill_reach);
    assert_eq!(bands(&cat, &a), bands(&cat, &b));
    let (mut a, mut b) = (adept(&cat, &[]), adept(&cat, &[]));
    wear(&cat, &mut a, "t_old_charm", Slot::Accessory2);
    wear(&cat, &mut b, "t_charm", Slot::Accessory2);
    assert_eq!(a.gear_skill_reach, b.gear_skill_reach);
    assert_eq!(bands(&cat, &a), bands(&cat, &b));
    assert_eq!(bands(&cat, &a)[0], (2, 5), "the bolt one tile further, as spell_reach gave");
}

#[test]
fn the_writer_form_of_the_alias_is_skill_reach() {
    let old = json!({"type": SPELL_REACH_ALIAS, "value": 2});
    let stored = SkillReach::migrated(old.as_object().unwrap()).expect("migrated");
    assert_eq!(Value::Object(stored), json!({"type": SKILL_REACH, "reach": 2, "kind": SKILL_REACH_SPELL}));
    assert!(SkillReach::migrated(reach(json!({"reach": 1})).as_object().unwrap()).is_none(), "the new form stays as written");
}
