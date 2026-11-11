//! PK2-S4 (`docs/sessions/PK2_GRAFTS.md` 5) on a scratch catalog built here (no shipped data): weapon techniques in
//! battle. The expectations derive from the rows below and the kit's mastery table (`rpgkit_data::mastery`): mastery n
//! opens the main-hand weapon family's first n techniques (none without mastery, none for another family), a masterwork
//! main hand adds the weapon's own; an open technique is owned and open without a deck slot, offered only in a standard
//! round (a Channel round greys it "Channel round" and its order leaves it out); a grip tag greys a technique under the
//! other grip, with or without the rule channel; an attack skill started by a hero built from member state records its
//! main-hand weapon's family for the settle's mastery (a catalog unit records nothing).

use indexmap::IndexMap;
use rpgkit_battle::channel;
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::round_intent::RoundIntent;
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::{EncounterDef, Roster, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::mastery::{MASTERY_MAX, NEEDS_1H, NEEDS_2H};
use rpgkit_data::{Catalog, PassiveDef, UnitDef};
use rpgkit_rules::gear::{Equipment, Slot};
use rpgkit_rules::ruleset::{Ruleset, AP_COST, CHANNEL, DECK, ROUND_INTENT, TIMELINE, UNIT_METERS};
use rpgkit_rules::Overrides;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const FOE: UnitId = 1;
/// The sword family's techniques in order (the second needs a two-handed grip, the third a one-handed one).
const TECHNIQUES: [&str; 3] = ["tq_one", "tq_two", "tq_three"];

fn attack(id: &str) -> Value {
    json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "ap_cost": 1, "power_mult": 0.5, "target_mode": "ENEMY"})
}

fn sword(id: &str, two_handed: bool) -> Value {
    json!({"id": id, "name": id, "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10, "two_handed": two_handed,
        "family_techniques": [{"skill": TECHNIQUES[0]}, {"skill": TECHNIQUES[1], "grip": "2h"}, {"skill": TECHNIQUES[2], "grip": "1h"}],
        "family_keystone": "tq_key", "technique": {"skill": "tq_own"}})
}

fn catalog() -> Catalog {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "class_id": "blade", "level": 3, "side": side, "skill_cap": 5, "default_intent": "assault", "deck": {"active": ["t_jab", "t_brace"]}});
    let mut skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "ap_cost": 1, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "t_brace", "name": "Brace", "kind": "DEFEND", "level": 1, "blocks": 1, "ap_cost": 1, "mitigation": 0.3}),
        attack("t_jab"),
    ];
    skills.extend(TECHNIQUES.iter().chain(&["tq_own", "tq_key"]).map(|id| attack(id)));
    Catalog::from_value(&json!({
        "classes": [{"id": "blade", "name": "Blade", "base_stats": stats, "growth": {}, "proficiency": {"sword": {"level": 3}}, "skill_path": {"t_jab": 1, "t_brace": 1}}],
        "units": [unit("p_hero", "party"), unit("p_foe", "enemy")],
        "skills": skills,
        "items": [sword("blade_1h", false), sword("blade_2h", true), {"id": "hatchet", "name": "Hatchet", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "axe", "base_power": 9}],
        "unit_meters": [{"id": "strain", "name": "Strain", "short": "St", "min": 0, "max": 100, "start": 0, "show": "bar"}],
    }))
}

/// The hero holds the strain meter (so it can Channel under the rule).
struct Heroes;

impl Roster for Heroes {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        cat.units.get(r).cloned()
    }

    fn overrides(&self, _cat: &Catalog, r: &str) -> Overrides {
        let passives = if r == "p_hero" { vec!["inoculated".to_string()] } else { Vec::new() };
        Overrides { passives, ..Overrides::default() }
    }
}

fn rules(channel_on: bool) -> Ruleset {
    let mut list = vec![ROUND_INTENT, DECK, AP_COST, UNIT_METERS];
    let mut row = json!({"id": TIMELINE, "name": "Timeline", "basics": ["move", "rest", "strike"], "deck_slots": {"active": 3, "passive": 0, "keystone": 1}});
    if channel_on {
        list.push(CHANNEL);
        row["channel_meter"] = json!("strain");
    }
    row["rules"] = json!(list);
    Ruleset::from_row(row.as_object().unwrap())
}

fn session(channel_on: bool) -> Session {
    let row = json!({"id": "tq_fight", "name": "Fight", "party": ["p_hero"], "enemies": ["p_foe"], "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 6, "rng_seed": 77});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = rules(channel_on);
    let mut cat = catalog();
    cat.passives.insert("inoculated".into(), PassiveDef { id: "inoculated".into(), name: "Inoculated".into(), effects: vec![json!({"type": "meter", "meter": "strain"})], slot: String::new() });
    let mut s = Session::start(Arc::new(cat), def, &Heroes);
    s.w.grid.positions[HERO] = Some(V2::new(4, 4));
    s.w.grid.positions[FOE] = Some(V2::new(5, 4));
    s
}

/// Unit `c` wields item `item` in the main hand at sword mastery `level` (None: no mastery), masterwork or not.
fn wield(w: &mut World, c: UnitId, item: &str, level: Option<i32>, masterwork: bool) {
    let cat = w.cat.clone();
    let u = &mut w.units[c];
    u.inventory.unequip(Slot::MainHand);
    u.inventory.equip(Equipment::new(&cat.items[item]), Slot::MainHand);
    u.mastery = level.map(|l| IndexMap::from([("sword".to_string(), l)]));
    u.masterwork = masterwork;
    u.sync_from_stats_and_gear(&cat);
}

fn open(w: &World, c: UnitId) -> Vec<String> {
    w.units[c].techniques.iter().map(|t| t.skill.clone()).collect()
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn skill<'a>(w: &'a World, id: &str) -> &'a rpgkit_data::SkillDef {
    &w.cat.skills[sid(w, id)]
}

/// Mastery n opens the first n techniques of the wielded family (at most the top level); no mastery or another family's
/// weapon opens none; a masterwork main hand adds its own.
#[test]
fn mastery_opens_the_wielded_familys_techniques_in_order() {
    let mut s = session(false);
    wield(&mut s.w, HERO, "blade_1h", None, false);
    assert!(open(&s.w, HERO).is_empty(), "no mastery, no technique");
    for level in 0..=MASTERY_MAX + 1 {
        wield(&mut s.w, HERO, "blade_1h", Some(level), false);
        let n = level.clamp(0, MASTERY_MAX) as usize;
        assert_eq!(open(&s.w, HERO), TECHNIQUES[..n].to_vec(), "mastery {level}");
    }
    wield(&mut s.w, HERO, "blade_1h", Some(1), true);
    assert_eq!(open(&s.w, HERO), [TECHNIQUES[0], "tq_own"], "the masterwork's own technique, a 4th");
    wield(&mut s.w, HERO, "hatchet", Some(MASTERY_MAX), true);
    assert!(open(&s.w, HERO).is_empty(), "sword mastery opens nothing with an axe");
}

/// An open technique is owned and open without a deck slot (it takes none); a standard round offers it (its order holds
/// it); a Channel round greys it "Channel round" and its order leaves it out.
#[test]
fn a_technique_takes_no_slot_and_is_offered_only_in_a_standard_round() {
    let mut s = session(true);
    wield(&mut s.w, HERO, "blade_1h", Some(1), false);
    let u = &s.w.units[HERO];
    let tq = skill(&s.w, TECHNIQUES[0]);
    assert!(u.knows_skill(&s.w.cat, &tq.id) && u.always_active(&s.w.cat, &tq.id), "owned and open without a slot");
    assert!(!u.active_families().iter().any(|f| f == TECHNIQUES[0]), "no deck slot taken");
    assert_eq!(channel::problem(&s.w, HERO, tq), None, "a standard round offers it");
    assert!(rip::order(&s.w, HERO, RoundIntent::Assault).contains(&sid(&s.w, TECHNIQUES[0])));
    assert!(channel::can_channel(&s.w, HERO));
    let _ = s.set_channel(HERO, true);
    assert!(channel::is_channel(&s.w, HERO));
    assert!(!channel::opens(&s.w, HERO, skill(&s.w, TECHNIQUES[0])));
    assert_eq!(channel::problem(&s.w, HERO, skill(&s.w, TECHNIQUES[0])), Some(channel::CHANNEL_ROUND));
    assert!(!rip::order(&s.w, HERO, RoundIntent::Assault).contains(&sid(&s.w, TECHNIQUES[0])));
}

/// A grip tag greys a technique under the other grip (with or without the rule channel): `2h` with a one-handed weapon,
/// `1h` with a two-handed one; an untagged technique works with either; the plan refuses the greyed one.
#[test]
fn a_grip_tag_greys_a_technique_under_the_other_grip() {
    for channel_on in [false, true] {
        let mut s = session(channel_on);
        wield(&mut s.w, HERO, "blade_1h", Some(MASTERY_MAX), false);
        let problem = |w: &World, id: &str| channel::problem(w, HERO, skill(w, id));
        assert_eq!([problem(&s.w, TECHNIQUES[0]), problem(&s.w, TECHNIQUES[1]), problem(&s.w, TECHNIQUES[2])], [None, Some(NEEDS_2H), None], "one-handed");
        let mut plan = Plan::new();
        assert_eq!(plan.add_step(&s.w, HERO, PlanStep::engage(sid(&s.w, TECHNIQUES[1]), FOE), true), NEEDS_2H);
        assert!(!rip::order(&s.w, HERO, RoundIntent::Assault).contains(&sid(&s.w, TECHNIQUES[1])));
        wield(&mut s.w, HERO, "blade_2h", Some(MASTERY_MAX), false);
        assert_eq!([problem(&s.w, TECHNIQUES[0]), problem(&s.w, TECHNIQUES[1]), problem(&s.w, TECHNIQUES[2])], [None, None, Some(NEEDS_1H)], "two-handed");
    }
}

/// `c` attacks `target` with `skill_id` in block 1.
fn resolve(w: &mut World, c: UnitId, skill_id: &str, target: UnitId) {
    let step: Step = PlanStep::engage(sid(w, skill_id), target);
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, c, step, true), "", "{skill_id}");
    let id = sid(w, skill_id);
    let cmd = plan.entries_for(w, c).into_iter().find(|e| e.skill == id).expect("the entry").build_command(w, c);
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(c, cmd, 1));
    for i in res.prepare_block(w, 1) {
        res.execute_single(w, i);
    }
}

/// An attack skill started by a hero built from member state (`rank_choice`) records its main-hand family once; a
/// catalog unit records nothing.
#[test]
fn an_attack_records_the_main_hand_family_of_a_member() {
    let s = session(false);
    let mut catalog_unit = s.w.clone();
    wield(&mut catalog_unit, HERO, "blade_1h", None, false);
    resolve(&mut catalog_unit, HERO, "t_jab", FOE);
    assert!(catalog_unit.weapon_attacks.is_empty(), "a catalog unit records nothing");
    let mut member = s.w.clone();
    wield(&mut member, HERO, "blade_1h", None, false);
    member.units[HERO].rank_choice = Some(IndexMap::new());
    resolve(&mut member, HERO, "t_jab", FOE);
    resolve(&mut member, HERO, "t_jab", FOE);
    assert_eq!(member.weapon_attacks.get(&HERO).cloned().unwrap_or_default(), ["sword"], "once per family");
}
