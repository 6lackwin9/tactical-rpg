//! ACT1-HOOKS2 (FB-244 (1)) on a scratch catalog built here (no shipped data): a skill the encounter grants a unit for
//! one battle (`Unit::battle_skills`) is owned and open without a slot (the unit's menu offers it; without the grant it
//! does not own the skill). A granted channeled-only skill plays in a Standard round for a unit that cannot Channel
//! (it holds no channel meter): no "Only in a Channel round" refusal, no channeled version and so no meter paid, and
//! the unit still holds no channel meter after it resolves. A unit that holds the channel meter keeps the channel rule
//! for the same grant: greyed in a Standard round, and in a Channel round it pays the skill's own `channel.cost`.

use rpgkit_battle::channel;
use rpgkit_battle::meter;
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::round_intent::{Pick, RoundTarget};
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::{EncounterDef, Roster, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::{Catalog, PassiveDef, UnitDef};
use rpgkit_rules::ruleset::{Ruleset, AP_COST, CHANNEL, DECK, FIELD_INTENT, ROUND_INTENT, TIMELINE, UNIT_METERS};
use rpgkit_rules::Overrides;
use serde_json::json;
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HOLDER: UnitId = 0;
const PLAIN: UnitId = 1;
const FOE: UnitId = 2;
/// The granted skill's own channel cost (the expectations derive from it).
const GRANT_COST: i32 = 15;
const GRANT: &str = "g_boon";

fn catalog() -> Catalog {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "class_id": "kin", "level": 3, "side": side, "skill_cap": 5, "default_intent": "assault"});
    let mut cat = Catalog::from_value(&json!({
        "classes": [{"id": "kin", "name": "Kin", "base_stats": stats, "growth": {}, "proficiency": {}, "skill_path": {"t_jab": 1}}],
        "units": [unit("p_holder", "party"), unit("p_plain", "party"), unit("p_foe", "enemy")],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "ap_cost": 1, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
            {"id": "t_jab", "name": "Jab", "kind": "ATTACK", "level": 1, "blocks": 1, "ap_cost": 1, "power_mult": 0.5, "target_mode": "ENEMY"},
            {"id": GRANT, "name": "Boon", "kind": "ATTACK", "level": 1, "blocks": 1, "ap_cost": 1, "power_mult": 0.5, "target_mode": "ENEMY",
                "learned_only": true, "channel": {"only": true, "cost": GRANT_COST}}
        ],
        "unit_meters": [{"id": "strain", "name": "Strain", "short": "St", "min": 0, "max": 100, "start": 0, "show": "bar"}],
    }));
    cat.passives.insert("carrier".into(), PassiveDef { id: "carrier".into(), name: "Carrier".into(), effects: vec![json!({"type": "meter", "meter": "strain"})], slot: String::new() });
    cat
}

/// Only `p_holder` holds the channel meter.
struct Holder;

impl Roster for Holder {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        cat.units.get(r).cloned()
    }

    fn overrides(&self, _cat: &Catalog, r: &str) -> Overrides {
        let passives = if r == "p_holder" { vec!["carrier".to_string()] } else { Vec::new() };
        Overrides { passives, ..Overrides::default() }
    }
}

/// The holder (4,3) and the plain unit (4,4) both face the foe from beside it (5,4 is next to 4,4; 5,3 next to 4,3).
fn session() -> Session {
    let rules = Ruleset::from_row(
        json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, DECK, AP_COST, FIELD_INTENT, UNIT_METERS, CHANNEL], "basics": ["move", "rest", "strike"],
            "deck_slots": {"active": 3, "passive": 0, "keystone": 0}, "channel_meter": "strain"})
        .as_object()
        .unwrap(),
    );
    let row = json!({"id": "grant_fight", "name": "Fight", "party": ["p_holder", "p_plain"], "enemies": ["p_foe"], "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 6, "rng_seed": 4242});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = rules;
    let mut s = Session::start(Arc::new(catalog()), def, &Holder);
    for (c, cell) in [(HOLDER, V2::new(4, 3)), (PLAIN, V2::new(4, 4)), (FOE, V2::new(5, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

fn grant(s: &mut Session, c: UnitId) {
    s.w.units[c].battle_skills.push(GRANT.into());
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

/// `c`'s menu cell of `GRANT` on the foe: None when the menu offers none, else Some(its problem).
fn cell(s: &Session, c: UnitId) -> Option<Option<String>> {
    let k = sid(&s.w, GRANT);
    rip::menu_options(&s.w, &s.plan, c, RoundTarget::Unit(FOE)).into_iter().find(|o| matches!(o.picks.last(), Some(Pick::Skill { skill, .. }) if *skill == k)).map(|o| o.problem)
}

#[test]
fn a_grant_is_owned_and_offered_for_the_battle_only() {
    let mut s = session();
    let cat = s.w.cat.clone();
    assert!(!s.w.units[PLAIN].owns_skill(&cat, GRANT), "learned only: not owned without the grant");
    assert_eq!(cell(&s, PLAIN), None, "not offered");
    grant(&mut s, PLAIN);
    let u = &s.w.units[PLAIN];
    assert!(u.battle_skill(GRANT) && u.owns_skill(&cat, GRANT) && u.always_active(&cat, GRANT), "owned and open without a slot");
    assert_eq!(cell(&s, PLAIN), Some(None), "offered with no problem");
}

#[test]
fn a_unit_without_the_channel_meter_plays_its_granted_channeled_only_skill_in_a_standard_round_for_free() {
    let mut s = session();
    assert!(!channel::can_channel(&s.w, PLAIN) && channel::can_channel(&s.w, HOLDER));
    let boon = s.w.cat.skills[sid(&s.w, GRANT)].clone();
    assert_eq!(channel::problem(&s.w, PLAIN, &boon), Some(channel::ONLY_CHANNEL), "not granted: the channel rule");
    grant(&mut s, PLAIN);
    assert!(channel::standard_grant(&s.w, PLAIN, &boon));
    assert_eq!(channel::problem(&s.w, PLAIN, &boon), None);
    assert!(channel::version(&s.w, PLAIN, &boon).is_none() && channel::meters_for(&s.w, PLAIN, &boon).is_empty(), "no version: no meter paid");
    let mut plan = Plan::new();
    let k = sid(&s.w, GRANT);
    assert_eq!(plan.add_step(&s.w, PLAIN, PlanStep::engage(k, FOE), true), "");
    let cmd = plan.entries_for(&s.w, PLAIN).into_iter().find(|e| e.skill == k).expect("the granted skill's entry").build_command(&s.w, PLAIN);
    assert!(cmd.meters.is_empty(), "{:?}", cmd.meters);
    let mut w = s.w.clone();
    let hp = w.units[FOE].hp;
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(PLAIN, cmd, 1));
    for i in res.prepare_block(&mut w, 1) {
        res.execute_single(&mut w, i);
    }
    assert!(w.units[FOE].hp < hp, "it resolved");
    assert_eq!(meter::value(&w, PLAIN, "strain"), None, "still no channel meter");
}

#[test]
fn a_holder_keeps_the_channel_rule_for_its_grant() {
    let mut s = session();
    grant(&mut s, HOLDER);
    let boon = s.w.cat.skills[sid(&s.w, GRANT)].clone();
    assert!(!channel::standard_grant(&s.w, HOLDER, &boon));
    assert_eq!(cell(&s, HOLDER), Some(Some(channel::ONLY_CHANNEL.to_string())), "greyed in a Standard round");
    let _ = s.set_channel(HOLDER, true);
    assert_eq!(cell(&s, HOLDER), Some(None), "open in a Channel round");
    let paid: Vec<i32> = channel::meters_for(&s.w, HOLDER, &boon).iter().map(|d| d.delta).collect();
    assert_eq!(paid, [GRANT_COST], "its own channel cost");
}
