//! PK2-S5b (`docs/sessions/PK2_GRAFTS.md` 7) on a scratch catalog built here (no shipped data): weapon durability in
//! battle (`rpgkit_battle::wear`). Under a ruleset with the rule `durability`, a unit that tracks its weapon's wear
//! (`Unit::wear`, a member's wielded instance) spends `SPEND_ATTACK` per standard attack skill and `SPEND_TECHNIQUE`
//! per technique from the weapon it swings; basics, support skills, items and a Channel round's actions spend nothing.
//! At 0 the weapon is dulled: its weapon power takes `DULLED_POWER_MULT`; under the sims' `dulled_lock` a dulled main
//! hand also greys its techniques. Without the rule (or without tracked wear) nothing is spent, dulled or locked. The
//! spend and the lock follow the weapon the skill swings (an off-hand skill: the off hand). The sims seed wear on their
//! allies (`wear::seed`) and read it back (`wear::tally`). The expectations derive from the kit's constants and the rows
//! below.

use indexmap::IndexMap;
use rpgkit_battle::channel;
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::session::{EncounterDef, Roster, Session};
use rpgkit_battle::wear::{self, DULLED, SPEND_ATTACK, SPEND_TECHNIQUE};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::{Catalog, PassiveDef, SkillDef, UnitDef};
use rpgkit_rules::damage;
use rpgkit_rules::gear::{Equipment, Slot, Wear, DULLED_POWER_MULT};
use rpgkit_rules::ruleset::{Ruleset, AP_COST, CHANNEL, DECK, DURABILITY, ROUND_INTENT, TIMELINE, UNIT_METERS};
use rpgkit_rules::Overrides;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const FOE: UnitId = 1;
/// The sword family's techniques: the first swings the main hand, the second the off hand (open at mastery 1 and 2).
const TECHNIQUE: &str = "tq_one";
const OFF_TECHNIQUE: &str = "tq_off";
/// An off-hand attack skill (not a technique).
const OFF_CUT: &str = "t_off";
/// A durability that outlasts every test's spend.
const PLENTY: i32 = 50;

fn attack(id: &str) -> Value {
    json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "ap_cost": 1, "power_mult": 0.5, "target_mode": "ENEMY"})
}

fn off_attack(id: &str) -> Value {
    let mut row = attack(id);
    row["weapon_hand"] = json!("off");
    row
}

fn catalog() -> Catalog {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "class_id": "blade", "level": 3, "side": side, "skill_cap": 5, "default_intent": "assault", "deck": {"active": ["t_jab", "t_brace", OFF_CUT]}});
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "ap_cost": 1, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "t_brace", "name": "Brace", "kind": "DEFEND", "level": 1, "blocks": 1, "ap_cost": 1, "mitigation": 0.3}),
        attack("t_jab"),
        attack(TECHNIQUE),
        off_attack(OFF_TECHNIQUE),
        off_attack(OFF_CUT),
    ];
    let sword = json!({"id": "blade", "name": "Blade", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10,
        "family_techniques": [{"skill": TECHNIQUE}, {"skill": OFF_TECHNIQUE}]});
    let dirk = json!({"id": "dirk", "name": "Dirk", "item_type": "WEAPON", "slot": "OFF_HAND", "weapon_type": "sword", "base_power": 6});
    Catalog::from_value(&json!({
        "classes": [{"id": "blade", "name": "Blade", "base_stats": stats, "growth": {}, "proficiency": {"sword": {"level": 3}}, "skill_path": {"t_jab": 1, "t_brace": 1, "t_off": 1}}],
        "units": [unit("p_hero", "party"), unit("p_foe", "enemy")],
        "skills": skills,
        "items": [sword, dirk],
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

fn rules(durability: bool, channel_on: bool) -> Ruleset {
    let mut list = vec![ROUND_INTENT, DECK, AP_COST, UNIT_METERS];
    let mut row = json!({"id": TIMELINE, "name": "Timeline", "basics": ["move", "rest", "strike"], "deck_slots": {"active": 3, "passive": 0, "keystone": 1}});
    if channel_on {
        list.push(CHANNEL);
        row["channel_meter"] = json!("strain");
    }
    if durability {
        list.push(DURABILITY);
    }
    row["rules"] = json!(list);
    Ruleset::from_row(row.as_object().unwrap())
}

/// A battle under `rules(durability, channel_on)` whose hero wields the blade and an off-hand dirk at sword mastery 2
/// (both techniques open) and tracks the blade's wear from `start` (None: no wear tracked, as a catalog unit).
fn session(durability: bool, channel_on: bool, start: Option<i32>) -> Session {
    let row = json!({"id": "dw_fight", "name": "Fight", "party": ["p_hero"], "enemies": ["p_foe"], "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 6, "rng_seed": 77});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = rules(durability, channel_on);
    let mut cat = catalog();
    cat.passives.insert("inoculated".into(), PassiveDef { id: "inoculated".into(), name: "Inoculated".into(), effects: vec![json!({"type": "meter", "meter": "strain"})], slot: String::new() });
    let mut s = Session::start(Arc::new(cat), def, &Heroes);
    s.w.grid.positions[HERO] = Some(V2::new(4, 4));
    s.w.grid.positions[FOE] = Some(V2::new(5, 4));
    let cat = s.w.cat.clone();
    let u = &mut s.w.units[HERO];
    u.inventory.unequip(Slot::MainHand);
    u.inventory.equip(Equipment::new(&cat.items["blade"]), Slot::MainHand);
    u.inventory.equip(Equipment::new(&cat.items["dirk"]), Slot::OffHand);
    u.mastery = Some(IndexMap::from([("sword".to_string(), 2)]));
    u.sync_from_stats_and_gear(&cat);
    if let Some(start) = start {
        u.wear.insert(Slot::MainHand, Wear { start, spent: 0 });
    }
    s
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn skill<'a>(w: &'a World, id: &str) -> &'a SkillDef {
    &w.cat.skills[sid(w, id)]
}

/// `c` uses `skill_id` (on `FOE` for an attack) in block 1.
fn resolve(w: &mut World, c: UnitId, skill_id: &str) {
    let step: Step = PlanStep::engage(sid(w, skill_id), FOE);
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

fn spent(w: &World) -> i32 {
    w.units[HERO].wear.get(&Slot::MainHand).map_or(0, |wr| wr.spent)
}

/// The spend table: a standard attack `SPEND_ATTACK`, a technique `SPEND_TECHNIQUE`; a basic, a support skill and an
/// item nothing; a Channel round nothing; and nothing at all without the rule or without tracked wear.
#[test]
fn each_kind_of_action_spends_its_share() {
    let s = session(true, false, Some(PLENTY));
    let of = |w: &World, id: &str| wear::spend_of(w, HERO, skill(w, id));
    assert_eq!([of(&s.w, "t_jab"), of(&s.w, TECHNIQUE), of(&s.w, "strike"), of(&s.w, "t_brace")], [SPEND_ATTACK, SPEND_TECHNIQUE, 0, 0]);
    let mut item = skill(&s.w, "t_jab").clone();
    item.raw.insert("_item".into(), json!(true));
    assert_eq!(wear::spend_of(&s.w, HERO, &item), 0, "an item spends nothing");
    for w in [&session(false, false, Some(PLENTY)).w, &session(true, false, None).w] {
        assert_eq!([of(w, "t_jab"), of(w, TECHNIQUE)], [0, 0], "no rule or no wear, no spend");
    }
    let mut channel = session(true, true, Some(PLENTY));
    assert_eq!(of(&channel.w, "t_jab"), SPEND_ATTACK, "a standard round spends");
    assert!(channel::can_channel(&channel.w, HERO));
    let _ = channel.set_channel(HERO, true);
    assert!(channel::is_channel(&channel.w, HERO));
    assert_eq!(of(&channel.w, "t_jab"), 0, "the channeled path pays its meter, not the weapon");
}

/// Resolving spends from the swung weapon once its costs are paid; the battle reports the spend by slot save key
/// (`HeroEnd::worn`'s source); a basic adds nothing.
#[test]
fn resolving_spends_from_the_swung_weapon_and_reports_it() {
    let mut s = session(true, false, Some(PLENTY));
    assert!(wear::spent(&s.w, HERO).is_empty(), "nothing spent yet");
    resolve(&mut s.w, HERO, "t_jab");
    resolve(&mut s.w, HERO, TECHNIQUE);
    resolve(&mut s.w, HERO, "strike");
    let total = SPEND_ATTACK + SPEND_TECHNIQUE;
    assert_eq!(spent(&s.w), total);
    assert_eq!(wear::spent(&s.w, HERO), [("main_hand".to_string(), i64::from(total))]);
    assert_eq!(s.w.units[HERO].wear[&Slot::MainHand].left(), PLENTY - total);
    // without the rule the same actions spend nothing
    let mut plain = session(false, false, Some(PLENTY));
    resolve(&mut plain.w, HERO, "t_jab");
    resolve(&mut plain.w, HERO, TECHNIQUE);
    assert!(wear::spent(&plain.w, HERO).is_empty());
}

/// A weapon whose wear reaches 0 is dulled: still usable, its weapon power at `DULLED_POWER_MULT`; the spend keeps
/// counting past 0 (the settle clamps the instance).
#[test]
fn a_weapon_at_zero_is_dulled_and_loses_weapon_power() {
    let mut s = session(true, false, Some(SPEND_ATTACK));
    let before = damage::attack_power(&s.w.units[HERO], false, false);
    assert!(!before.dulled && before.weapon_power > 0.0);
    resolve(&mut s.w, HERO, "t_jab");
    let u = &s.w.units[HERO];
    assert!(u.dulled(Slot::MainHand));
    let after = damage::attack_power(u, false, false);
    assert!(after.dulled);
    assert_eq!(after.weapon_power, before.weapon_power * DULLED_POWER_MULT);
    assert_eq!(after.unit_power, before.unit_power, "only the weapon power drops");
    resolve(&mut s.w, HERO, TECHNIQUE);
    assert_eq!(spent(&s.w), SPEND_ATTACK + SPEND_TECHNIQUE, "a dulled weapon is still usable, techniques included");
    // an untracked unit never dulls
    let catalog_unit = session(true, false, None);
    assert!(!damage::attack_power(&catalog_unit.w.units[HERO], false, false).dulled);
}

/// The sims' alternative: under `dulled_lock` a dulled main hand greys its techniques (the plan refuses them); other
/// skills stay open; off by default, and nothing locks without the rule.
#[test]
fn the_lock_switch_greys_a_dulled_main_hands_techniques() {
    let mut s = session(true, false, Some(0));
    assert_eq!(channel::problem(&s.w, HERO, skill(&s.w, TECHNIQUE)), None, "the default: the power penalty alone");
    s.w.dulled_lock = true;
    assert_eq!(channel::problem(&s.w, HERO, skill(&s.w, TECHNIQUE)), Some(DULLED));
    assert_eq!(channel::problem(&s.w, HERO, skill(&s.w, "t_jab")), None, "only techniques lock");
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&s.w, HERO, PlanStep::engage(sid(&s.w, TECHNIQUE), FOE), true), DULLED);
    let mut sharp = session(true, false, Some(PLENTY));
    sharp.w.dulled_lock = true;
    assert_eq!(channel::problem(&sharp.w, HERO, skill(&sharp.w, TECHNIQUE)), None, "a sharp weapon keeps its techniques");
    let mut plain = session(false, false, Some(0));
    plain.w.dulled_lock = true;
    assert_eq!(channel::problem(&plain.w, HERO, skill(&plain.w, TECHNIQUE)), None, "no rule, no lock");
}

/// The session start clears every unit's wear without the rule (inert by construction) and keeps it with the rule.
#[test]
fn the_start_clears_wear_without_the_rule() {
    for (durability, kept) in [(true, true), (false, false)] {
        let mut s = session(durability, false, Some(PLENTY));
        wear::clear_without_rule(&mut s.w);
        assert_eq!(!s.w.units[HERO].wear.is_empty(), kept, "rule {durability}");
    }
}

/// An off-hand skill spends from the off hand (a technique too), never from the main hand; an untracked off hand spends
/// nothing.
#[test]
fn an_off_hand_skill_spends_the_off_hand() {
    let mut s = session(true, false, Some(PLENTY));
    resolve(&mut s.w, HERO, OFF_CUT);
    assert_eq!(spent(&s.w), 0, "the off hand is not tracked: nothing spent, the main hand untouched");
    s.w.units[HERO].wear.insert(Slot::OffHand, Wear { start: PLENTY, spent: 0 });
    resolve(&mut s.w, HERO, OFF_CUT);
    resolve(&mut s.w, HERO, OFF_TECHNIQUE);
    resolve(&mut s.w, HERO, "t_jab");
    assert_eq!(s.w.units[HERO].wear[&Slot::OffHand].spent, SPEND_ATTACK + SPEND_TECHNIQUE);
    assert_eq!(spent(&s.w), SPEND_ATTACK);
    assert_eq!(wear::spent(&s.w, HERO), [("main_hand".to_string(), i64::from(SPEND_ATTACK)), ("off_hand".to_string(), i64::from(SPEND_ATTACK + SPEND_TECHNIQUE))]);
}

/// Under the lock a technique greys while the weapon it swings is dulled: a dulled off hand locks the off-hand
/// technique only, a dulled main hand the main-hand one only.
#[test]
fn the_lock_follows_the_weapon_the_technique_swings() {
    let problems = |main: i32, off: i32| {
        let mut s = session(true, false, Some(main));
        s.w.units[HERO].wear.insert(Slot::OffHand, Wear { start: off, spent: 0 });
        s.w.dulled_lock = true;
        [channel::problem(&s.w, HERO, skill(&s.w, TECHNIQUE)), channel::problem(&s.w, HERO, skill(&s.w, OFF_TECHNIQUE))]
    };
    assert_eq!(problems(PLENTY, 0), [None, Some(DULLED)]);
    assert_eq!(problems(0, PLENTY), [Some(DULLED), None]);
    assert_eq!(problems(PLENTY, PLENTY), [None, None]);
}

/// The sims' seed: every listed unit tracks `n` on each weapon it wields (main hand, off-hand weapon), only under the
/// rule; the tally sums the spend and counts the dulled weapons.
#[test]
fn the_sims_seed_and_tally_wear() {
    let mut s = session(true, false, None);
    wear::seed(&mut s.w, &[HERO, FOE], SPEND_ATTACK);
    let start = |w: &World, c: UnitId| w.units[c].wear.iter().map(|(sl, wr)| (*sl, wr.start)).collect::<Vec<_>>();
    assert_eq!(start(&s.w, HERO), [(Slot::MainHand, SPEND_ATTACK), (Slot::OffHand, SPEND_ATTACK)]);
    let foe_weapons: Vec<Slot> = [Slot::MainHand, Slot::OffHand].into_iter().filter(|&sl| s.w.units[FOE].inventory.get(sl).is_some_and(Equipment::is_weapon)).collect();
    assert_eq!(s.w.units[FOE].wear.keys().copied().collect::<Vec<_>>(), foe_weapons, "only the weapons it wields");
    assert_eq!(wear::tally(&s.w, &[HERO]), (0, 0));
    resolve(&mut s.w, HERO, "t_jab");
    assert_eq!(wear::tally(&s.w, &[HERO]), (i64::from(SPEND_ATTACK), 1), "the main hand dulled, the off hand not");
    let mut plain = session(false, false, None);
    wear::seed(&mut plain.w, &[HERO], SPEND_ATTACK);
    assert!(plain.w.units[HERO].wear.is_empty(), "no rule, no seed");
}

/// The Dojo breakdown names the dulled factor (and only on a dulled weapon).
#[test]
fn the_dojo_breakdown_names_a_dulled_weapon() {
    let line = format!("Dulled x{DULLED_POWER_MULT:.2}");
    for (start, shown) in [(0, true), (PLENTY, false)] {
        let s = session(true, false, Some(start));
        let hit = damage::compute(&s.w.units[HERO], &s.w.units[FOE], damage::HitInput::default());
        let text = rpgkit_battle::dojo::breakdown_strike("Strike", "hero", &s.w.units[FOE], &hit, hit.final_dmg);
        assert_eq!(text.contains(&line), shown, "start {start}: {text}");
    }
}
