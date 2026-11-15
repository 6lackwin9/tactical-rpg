//! GEAR-REWORK1 (PR5-51a pass 2, `docs/sessions/U3_GEAR_REWORK.md` 2-4) on a scratch catalog built here (no shipped
//! data; resolved item rows carry their family's keys as the loader writes them; the skill ids are test ids, named to
//! the kit only through the item rows). Expectations come from the rules and the rows below:
//!
//! - a `1h_light` off-hand weapon's jab reaches as the weapon does (1 to its `range_band` max), keeps its authored
//!   power factor, blocks and AP, and the AI's single attacks offer it to a melee unit too; a pre-rework off-hand
//!   weapon's jab stays at distance 1 and melee units do not get it;
//! - an ammunition item (`empower_next`) gives its wearer the meter under the rule `unit_meters`: a ranged main-hand
//!   shot spends the cost and hits at `power_mult`, at less it fires plain; the item's Reload is open only at 0 and
//!   refills the meter; without the rule nothing is spent or scaled;
//! - pass 3 (U3.3 2 / 4 "unlocks ranged 2-3 Throw"): an ammunition rule's `throw_skill` is granted to a `1h_light`
//!   main-hand weapon beside the item (not to a heavy, two-handed or pre-rework main, nor beside other off-hand gear),
//!   reaches its own 2-3 band, is open only while the meter covers the cost (refused with the kit's reason at less,
//!   at the start too), spends the cost and is not empowered; the melee attack stays as it was; Reload restores it;
//! - the Off-Hand button (`offhand::action`) names the worn item's action, greyed with the kit's reason, and is absent
//!   for an empty off hand or gear from before the rework.

use rpgkit_battle::command::{CmdKind, Command};
use rpgkit_battle::grid::V2;
use rpgkit_battle::meter;
use rpgkit_battle::offhand::{self, RELOAD_NOT_EMPTY, RELOAD_NO_METER, THROW_NO_AMMO, THROW_NO_LIGHT, THROW_NO_METER};
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::session::{EncounterDef, Roster, Session};
use rpgkit_battle::skill;
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::{Catalog, UnitDef};
use rpgkit_rules::gear::{Equipment, OffHandUse, Slot};
use rpgkit_rules::ruleset::{Ruleset, TIMELINE, UNIT_METERS};
use rpgkit_rules::Overrides;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const FOE: UnitId = 1;
/// The test rows' numbers (the expectations derive from them).
const AMMO_MAX: i32 = 3;
const AMMO_COST: i32 = 1;
const EMPOWER: f64 = 1.5;
const KNIFE_REACH: i32 = 2;
const THROW_BAND: (i32, i32) = (2, 3);

fn weapon(id: &str, family: &str, extra: Value) -> Value {
    let mut r = json!({"id": id, "name": id, "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": family, "base_power": 12});
    for (k, v) in extra.as_object().unwrap() {
        r[k] = v.clone();
    }
    r
}

fn off(id: &str, family: &str, extra: Value) -> Value {
    let mut e = json!({"slot": "OFF_HAND", "family_slot_rule": "off_only", "off_hand_category": family, "base_power": 0});
    for (k, v) in extra.as_object().unwrap() {
        e[k] = v.clone();
    }
    weapon(id, family, e)
}

fn catalog() -> Catalog {
    let stats = json!({"STR": 12, "DEX": 12, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "class_id": "hand", "level": 3, "side": side, "skill_cap": 5, "deck": {"active": ["t_shot"]}});
    let support = |id: &str, name: &str| json!({"id": id, "name": name, "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 1, "target_mode": "SELF", "heal_base": 1, "learned_only": true});
    Catalog::from_value(&json!({
        "classes": [{"id": "hand", "name": "Hand", "base_stats": stats, "growth": {}, "skill_path": {"t_shot": 1}}],
        "units": [unit("p_hero", "party"), unit("p_foe", "enemy")],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "ap_cost": 1, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
            {"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "t_jab", "name": "Jab", "kind": "ATTACK", "level": 1, "blocks": 1, "ap_cost": 1, "basic": true, "power_mult": 0.5, "weapon_hand": "off", "target_mode": "ENEMY"},
            {"id": "t_shot", "name": "Shot", "kind": "ATTACK", "level": 1, "blocks": 1, "ap_cost": 1, "target_mode": "ENEMY"},
            {"id": "t_throw", "name": "Throw", "kind": "ATTACK", "level": 1, "blocks": 1, "ap_cost": 1, "target_mode": "ENEMY", "learned_only": true, "reach_min": THROW_BAND.0, "reach_max": THROW_BAND.1},
            support("t_reload", "Reload"),
            support("t_fade", "Fade-Step"),
        ],
        "unit_meters": [{"id": "t_ammo", "name": "Ammo", "short": "Am", "min": 0, "max": AMMO_MAX, "start": 0, "start_full": true, "show": "pips"}],
        "items": [
            weapon("bow", "bow", json!({"two_handed": true, "base_range": 4, "scaling_stat": "DEX", "family_slot_rule": "2h_rig", "family_pairs_with": ["ammo", "cloak"], "family_rules": ["always_ranged"]})),
            weapon("saber", "sword", json!({"family_slot_rule": "1h_heavy"})),
            weapon("knife", "knife", json!({"family_slot_rule": "1h_light", "family_reach_max": KNIFE_REACH})),
            weapon("dagger", "dagger", json!({"family_slot_rule": "1h_light"})),
            weapon("old_knife", "knife", json!({})),
            off("quiver", "ammo", json!({"family_off_hand_skill": "t_reload", "family_empower_next": {"meter": "t_ammo", "cost": AMMO_COST, "power_mult": EMPOWER}})),
            off("bandolier", "ammo", json!({"family_off_hand_skill": "t_reload", "family_empower_next": {"meter": "t_ammo", "cost": AMMO_COST, "power_mult": EMPOWER, "throw_skill": "t_throw"}})),
            off("mantle", "cloak", json!({"family_off_hand_skill": "t_fade"})),
            off("orb", "orb", json!({})),
        ],
    }))
}

struct Units;

impl Roster for Units {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        cat.units.get(r).cloned()
    }

    fn overrides(&self, _cat: &Catalog, _r: &str) -> Overrides {
        Overrides::default()
    }
}

fn rules(list: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": list}).as_object().unwrap())
}

/// The hero wields `main` and `off_item` (the off hand empty for None) from the battle's start; the foe stands `gap`
/// tiles east of it.
fn session(ruleset: Ruleset, main: &str, off_item: Option<&str>, gap: i32) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["p_hero"], "enemies": ["p_foe"], "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 6, "rng_seed": 5});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &Units);
    s.w.grid.positions[HERO] = Some(V2::new(4, 4));
    s.w.grid.positions[FOE] = Some(V2::new(4 + gap, 4));
    arm(&mut s.w, main, off_item);
    s
}

/// Re-arms the hero (gear sync, then its meters as at a battle start).
fn arm(w: &mut World, main: &str, off_item: Option<&str>) {
    let cat = w.cat.clone();
    let u = &mut w.units[HERO];
    u.inventory.unequip(Slot::MainHand);
    u.inventory.unequip(Slot::OffHand);
    u.inventory.equip(Equipment::new(&cat.items[main]), Slot::MainHand);
    if let Some(o) = off_item {
        u.inventory.equip(Equipment::new(&cat.items[o]), Slot::OffHand);
    }
    u.sync_from_stats_and_gear(&cat);
    meter::restart_unit(w, HERO);
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn command(w: &World, step: Step, skill: &str) -> Command {
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, HERO, step, true), "", "{skill}");
    let id = sid(w, skill);
    plan.entries_for(w, HERO).into_iter().find(|e| e.skill == id).expect("the skill's entry").build_command(w, HERO)
}

fn mult(c: &Command) -> f64 {
    match &c.kind {
        CmdKind::Attack { skill_mult, .. } | CmdKind::Area { skill_mult, .. } => *skill_mult,
        _ => panic!("an attack"),
    }
}

/// Books `cmd` at block 1 and runs its start; returns its index.
fn start(w: &mut World, res: &mut Resolver, cmd: Command) -> usize {
    assert!(res.schedule(HERO, cmd, 1), "booked");
    let i = res.scheduled.len() - 1;
    for k in res.prepare_block(w, 1) {
        res.execute_single(w, k);
    }
    i
}

fn ammo(w: &World) -> Option<i32> {
    meter::value(w, HERO, "t_ammo")
}

fn set_ammo(w: &mut World, v: i32) {
    w.meters.gauges.iter_mut().find(|g| g.unit == HERO && g.meter == "t_ammo").expect("the ammo gauge").value = v;
}

#[test]
fn a_light_off_hand_weapon_jabs_at_its_reach_with_its_authored_cost() {
    let s = session(rules(&[]), "saber", Some("knife"), KNIFE_REACH);
    let w = &s.w;
    let jab = &w.cat.skills[sid(w, "t_jab")];
    assert_eq!(skill::reach_by(jab, &w.units[HERO], &w.ruleset), (1, KNIFE_REACH), "the knife's band");
    assert!(w.units[HERO].gear_skill("t_jab"), "the jab is open while the light weapon is worn");
    let c = command(w, PlanStep::engage(sid(w, "t_jab"), FOE), "t_jab");
    assert_eq!((c.block_cost, c.ap_cost), (jab.blocks, jab.ap_cost), "the row's blocks and AP");
    assert!((mult(&c) - w.units[HERO].power_for(jab)).abs() < 1e-9 && (jab.power_mult - 0.5).abs() < 1e-9, "the row's 0.5x");
    let p = rpgkit_rules::damage::attack_power(&w.units[HERO], false, true);
    assert_eq!(p.weapon, "knife", "the off-hand weapon's power");
    assert!(w.book().single_attacks(&w.units[HERO], 5).contains(&sid(w, "t_jab")), "a melee unit with a light off hand jabs");
    // a light weapon without a band jabs at 1; a pre-rework off-hand weapon too, and melee units keep it out
    let mut s = session(rules(&[]), "saber", Some("dagger"), 1);
    assert_eq!(skill::reach_by(&s.w.cat.skills[sid(&s.w, "t_jab")], &s.w.units[HERO], &s.w.ruleset), (1, 1));
    arm(&mut s.w, "saber", Some("old_knife"));
    let w = &s.w;
    assert_eq!(skill::reach_by(&w.cat.skills[sid(w, "t_jab")], &w.units[HERO], &w.ruleset), (1, 1), "today's jab");
    assert!(!w.units[HERO].gear_skill("t_jab"), "nothing granted for gear from before the rework");
    assert!(!w.book().single_attacks(&w.units[HERO], 5).contains(&sid(w, "t_jab")), "the legacy cap");
}

#[test]
fn ammo_empowers_ranged_shots_while_it_lasts_and_reload_refills_it_at_zero() {
    let mut s = session(rules(&[UNIT_METERS]), "bow", Some("quiver"), 3);
    assert_eq!(ammo(&s.w), Some(AMMO_MAX), "start_full");
    let shot = |w: &World| command(w, PlanStep::engage(sid(w, "t_shot"), FOE), "t_shot");
    let plain = mult(&shot(&s.w));
    let mut res = Resolver::new(BLOCKS, 5);
    let cmd = shot(&s.w);
    let i = start(&mut s.w, &mut res, cmd);
    assert_eq!(ammo(&s.w), Some(AMMO_MAX - AMMO_COST), "the shot spent its cost");
    assert!((mult(&res.scheduled[i].command) - plain * EMPOWER).abs() < 1e-9, "empowered");
    assert!(res.log.iter().any(|l| l.contains("empowered")), "logged");
    // the Reload is closed above 0
    let reload = s.w.cat.skills[sid(&s.w, "t_reload")].clone();
    assert_eq!(rpgkit_battle::channel::problem(&s.w, HERO, &reload), Some(RELOAD_NOT_EMPTY));
    assert_eq!(Plan::new().add_step(&s.w, HERO, PlanStep::self_skill(sid(&s.w, "t_reload")), true), RELOAD_NOT_EMPTY, "the planner refuses it");
    // at 0 the shot fires plain and the Reload opens, refilling the meter
    set_ammo(&mut s.w, 0);
    let mut res = Resolver::new(BLOCKS, 5);
    let cmd = shot(&s.w);
    let i = start(&mut s.w, &mut res, cmd);
    assert_eq!(ammo(&s.w), Some(0));
    assert!((mult(&res.scheduled[i].command) - plain).abs() < 1e-9, "fires plain at 0");
    assert_eq!(rpgkit_battle::channel::problem(&s.w, HERO, &reload), None);
    let mut res = Resolver::new(BLOCKS, 5);
    let cmd = command(&s.w, PlanStep::self_skill(sid(&s.w, "t_reload")), "t_reload");
    start(&mut s.w, &mut res, cmd);
    assert_eq!(ammo(&s.w), Some(AMMO_MAX), "reloaded");
}

#[test]
fn only_a_ranged_main_hand_weapon_shot_spends_ammo() {
    let mut s = session(rules(&[UNIT_METERS]), "saber", Some("quiver"), 1);
    assert_eq!(ammo(&s.w), Some(AMMO_MAX), "the wearer holds the meter");
    let w = &s.w;
    assert!(!offhand::empowers(w, HERO, &w.cat.skills[sid(w, "t_shot")]), "a melee weapon");
    assert!(!offhand::empowers(w, HERO, &w.cat.skills[sid(w, "t_jab")]), "an off-hand jab");
    let cmd = command(&s.w, PlanStep::engage(sid(&s.w, "t_shot"), FOE), "t_shot");
    let plain = mult(&cmd);
    let mut res = Resolver::new(BLOCKS, 5);
    let i = start(&mut s.w, &mut res, cmd);
    assert_eq!(ammo(&s.w), Some(AMMO_MAX), "nothing spent");
    assert!((mult(&res.scheduled[i].command) - plain).abs() < 1e-9);
}

#[test]
fn without_unit_meters_ammo_neither_spends_nor_empowers() {
    let mut s = session(rules(&[]), "bow", Some("quiver"), 3);
    assert_eq!(ammo(&s.w), None, "no gauge without the rule");
    let cmd = command(&s.w, PlanStep::engage(sid(&s.w, "t_shot"), FOE), "t_shot");
    let plain = mult(&cmd);
    let mut res = Resolver::new(BLOCKS, 5);
    let i = start(&mut s.w, &mut res, cmd);
    assert!((mult(&res.scheduled[i].command) - plain).abs() < 1e-9);
    let reload = s.w.cat.skills[sid(&s.w, "t_reload")].clone();
    assert_eq!(offhand::reload_problem(&s.w, HERO, &reload), Some(RELOAD_NO_METER));
}

#[test]
fn the_off_hand_button_names_the_worn_items_action_or_is_absent() {
    let mut s = session(rules(&[UNIT_METERS]), "bow", Some("mantle"), 3);
    let a = offhand::action(&s.w, HERO).expect("a cloak's button");
    assert_eq!((a.skill, a.used.clone(), a.problem.clone(), a.ammo), (sid(&s.w, "t_fade"), OffHandUse::Skill { skill: "t_fade".into(), reload: false }, None, None));
    assert!(s.w.units[HERO].gear_skill("t_fade") && rpgkit_battle::round_intent_plan::hotkey_skills(&s.w, HERO).contains(&a.skill), "open without a slot, on the hotkeys");
    arm(&mut s.w, "bow", Some("quiver"));
    let a = offhand::action(&s.w, HERO).expect("the quiver's Reload");
    assert_eq!((a.ammo, a.problem.as_deref()), (Some((AMMO_MAX, AMMO_MAX)), Some(RELOAD_NOT_EMPTY)), "greyed while full");
    assert!(matches!(a.used, OffHandUse::Skill { reload: true, .. }));
    arm(&mut s.w, "saber", Some("knife"));
    let a = offhand::action(&s.w, HERO).expect("the knife's jab");
    assert_eq!((a.skill, a.used), (sid(&s.w, "t_jab"), OffHandUse::Jab { reach: KNIFE_REACH }));
    arm(&mut s.w, "saber", Some("orb"));
    assert_eq!(offhand::action(&s.w, HERO), None, "an off-hand item from before the rework");
    arm(&mut s.w, "saber", Some("old_knife"));
    assert_eq!(offhand::action(&s.w, HERO), None);
    arm(&mut s.w, "saber", None);
    assert_eq!(offhand::action(&s.w, HERO), None, "an empty off hand");
}

#[test]
fn a_light_main_beside_throwing_ammo_gains_a_throw_that_costs_the_pip() {
    let mut s = session(rules(&[UNIT_METERS]), "knife", Some("bandolier"), THROW_BAND.1);
    let cat = s.w.cat.clone();
    assert!(rpgkit_rules::gear::off_hand_ok(Some(&cat.items["knife"]), &cat.items["bandolier"]) && rpgkit_rules::gear::item_fits_slot(&cat.items["bandolier"], "off_hand"), "a one-handed main pairs freely");
    let throw = cat.skills[sid(&s.w, "t_throw")].clone();
    assert!(s.w.units[HERO].gear_skill("t_throw"), "granted beside the ammo");
    assert_eq!(skill::reach_by(&throw, &s.w.units[HERO], &s.w.ruleset), THROW_BAND, "its own band");
    assert_eq!(rpgkit_battle::channel::problem(&s.w, HERO, &throw), None, "open with pips");
    assert!(!offhand::empowers(&s.w, HERO, &throw), "the pip unlocks it; it does not empower it");
    assert!(s.w.book().single_attacks(&s.w.units[HERO], 5).contains(&sid(&s.w, "t_throw")), "the AI's attacks offer it");
    let cmd = command(&s.w, PlanStep::engage(sid(&s.w, "t_throw"), FOE), "t_throw");
    let plain = mult(&cmd);
    let mut res = Resolver::new(BLOCKS, 5);
    let i = start(&mut s.w, &mut res, cmd);
    assert!(!res.scheduled[i].cancelled);
    assert_eq!(ammo(&s.w), Some(AMMO_MAX - AMMO_COST), "the Throw spent its cost");
    assert!((mult(&res.scheduled[i].command) - plain).abs() < 1e-9, "not empowered");
    assert!(res.log.iter().any(|l| l.contains("thrown")), "logged");
    // the melee attack is unchanged: no ammo spent, no factor, the knife's band without the ammo too
    let shot = cat.skills[sid(&s.w, "t_shot")].clone();
    let band = skill::reach_by(&shot, &s.w.units[HERO], &s.w.ruleset);
    let cmd = command(&s.w, PlanStep::engage(sid(&s.w, "t_shot"), FOE), "t_shot");
    let plain = mult(&cmd);
    let before = ammo(&s.w);
    let mut res = Resolver::new(BLOCKS, 5);
    let i = start(&mut s.w, &mut res, cmd);
    assert_eq!(ammo(&s.w), before, "a melee swing spends nothing");
    assert!((mult(&res.scheduled[i].command) - plain).abs() < 1e-9);
    arm(&mut s.w, "knife", None);
    assert_eq!(skill::reach_by(&shot, &s.w.units[HERO], &s.w.ruleset), band, "the melee band without the ammo");
}

#[test]
fn the_throw_is_refused_below_the_cost_and_reload_restores_it() {
    let mut s = session(rules(&[UNIT_METERS]), "knife", Some("bandolier"), THROW_BAND.0);
    let throw = s.w.cat.skills[sid(&s.w, "t_throw")].clone();
    set_ammo(&mut s.w, AMMO_COST - 1);
    assert_eq!(rpgkit_battle::channel::problem(&s.w, HERO, &throw), Some(THROW_NO_AMMO));
    assert_eq!(Plan::new().add_step(&s.w, HERO, PlanStep::engage(sid(&s.w, "t_throw"), FOE), true), THROW_NO_AMMO, "the planner refuses it");
    let a = offhand::action(&s.w, HERO).expect("the bandolier's Reload");
    assert_eq!((a.skill, a.problem), (sid(&s.w, "t_reload"), None), "Reload open at 0");
    let mut res = Resolver::new(BLOCKS, 5);
    let cmd = command(&s.w, PlanStep::self_skill(sid(&s.w, "t_reload")), "t_reload");
    start(&mut s.w, &mut res, cmd);
    assert_eq!(ammo(&s.w), Some(AMMO_MAX), "reloaded");
    assert_eq!(rpgkit_battle::channel::problem(&s.w, HERO, &throw), None, "the Throw opens again");
    // a Throw planned with the pip fails unpaid when the meter no longer covers it at its start
    let cmd = command(&s.w, PlanStep::engage(sid(&s.w, "t_throw"), FOE), "t_throw");
    set_ammo(&mut s.w, AMMO_COST - 1);
    let mut res = Resolver::new(BLOCKS, 5);
    let i = start(&mut s.w, &mut res, cmd);
    assert!(res.scheduled[i].cancelled, "fails unpaid");
    assert_eq!(ammo(&s.w), Some(AMMO_COST - 1), "nothing spent");
    assert!(res.log.iter().any(|l| l.contains(&format!("fails unpaid ({THROW_NO_AMMO})"))), "logged with the reason");
}

#[test]
fn no_throw_without_a_light_main_beside_throwing_ammo() {
    let mut s = session(rules(&[UNIT_METERS]), "knife", Some("quiver"), THROW_BAND.0);
    assert!(!s.w.units[HERO].gear_skill("t_throw"), "ammo without a throw_skill");
    for (main, off_item) in [("knife", Some("mantle")), ("knife", None), ("saber", Some("bandolier")), ("bow", Some("bandolier")), ("old_knife", Some("bandolier"))] {
        arm(&mut s.w, main, off_item);
        assert!(!s.w.units[HERO].gear_skill("t_throw"), "{main} + {off_item:?}");
    }
    // known some other way, the Throw still wants the light main, and the meter
    let throw = s.w.cat.skills[sid(&s.w, "t_throw")].clone();
    arm(&mut s.w, "saber", Some("bandolier"));
    assert_eq!(offhand::throw_problem(&s.w, HERO, &throw), Some(THROW_NO_LIGHT));
    let s = session(rules(&[]), "knife", Some("bandolier"), THROW_BAND.0);
    assert!(s.w.units[HERO].gear_skill("t_throw"));
    assert_eq!(rpgkit_battle::channel::problem(&s.w, HERO, &throw), Some(THROW_NO_METER), "no meter without the rule");
}
