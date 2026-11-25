//! PK3 bleed-out countdown and occupied-cell revive tests on a minimal story-free scratch catalog.

use aegis_core::rng::GodotRng;
use indexmap::IndexMap;
use rpgkit_battle::ai;
use rpgkit_battle::command::{CmdKind, Command};
use rpgkit_battle::encounter::PartyFate;
use rpgkit_battle::grid::{GRID_W, V2};
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::round_intent::{Pick, RoundIntent, RoundTarget};
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::{EncounterDef, Roster, Session};
use rpgkit_battle::world::{Role, Tactic, UnitId};
use rpgkit_data::{Catalog, UnitDef};
use rpgkit_rules::ruleset::{Ruleset, ROUND_INTENT, TIMELINE};
use rpgkit_rules::Overrides;
use serde_json::json;
use std::sync::Arc;

const HERO: UnitId = 0;
const MEDIC: UnitId = 1;
const GUEST: UnitId = 2;
const FOE: UnitId = 3;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "ap_cost": 2, "basic": true, "target_mode": "ENEMY", "power_mult": 1.0}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 1, "ap_cost": 1, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "t_revive", "name": "Field Triage", "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 2, "target_mode": "ALLY", "reach_min": 1, "reach_max": 3, "heal_base": 14, "revive": true}),
        json!({"id": "t_mend", "name": "Patch", "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 1, "target_mode": "ALLY", "reach_min": 1, "reach_max": 3, "heal_base": 10}),
    ];
    let items = vec![
        json!({"id": "t_salts", "name": "Revive Salts", "family": "consumable", "item_type": "CONSUMABLE", "consumable_type": "HEALTH_POTION", "heal": 15, "damage": 0, "use_blocks": 1, "use_reach": 1, "revive": true, "value": 15, "max_stack": 2}),
        json!({"id": "t_tonic", "name": "Plain Tonic", "family": "consumable", "item_type": "CONSUMABLE", "consumable_type": "HEALTH_POTION", "heal": 15, "damage": 0, "use_blocks": 1, "use_reach": 1, "value": 15, "max_stack": 2}),
    ];
    let unit = |id: &str, name: &str, side: &str, hp: i32, power: i32, speed: i32, role: &str| {
        json!({"id": id, "name": name, "side": side, "stat_mode": "fixed", "hp": hp, "power": power, "armor": 0, "speed": speed, "attack_range": 1, "skill_cap": 5, "ai_role": role, "flee_ratio": 0.0})
    };
    Catalog::from_value(&json!({
        "skills": skills,
        "items": items,
        "units": [
            unit("p_hero", "Hero", "party", 50, 25, 14, "AGGRESSOR"),
            unit("p_medic", "Medic", "party", 50, 25, 12, "SUPPORT"),
            unit("p_guest", "Guide", "party", 50, 25, 11, "SUPPORT"),
            unit("p_foe", "Foe", "enemy", 100, 120, 10, "AGGRESSOR")
        ]
    }))
}

struct TestRoster;

impl Roster for TestRoster {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        cat.units.get(r).cloned()
    }

    fn overrides(&self, _cat: &Catalog, _r: &str) -> Overrides {
        Overrides::default()
    }
}

fn skill_id(cat: &Catalog, id: &str) -> usize {
    cat.skills.iter().position(|s| s.id == id).expect(id)
}

fn start_session(rules: &[&str], bleed_out_rounds: i32, medic_skills: &[&str], medic_pouch: &[&str]) -> Session {
    let row = json!({
        "id": "t_fight",
        "name": "Test Fight",
        "party": ["p_hero", "p_medic"],
        "guests": ["p_guest"],
        "enemies": ["p_foe"],
        "exits": [[11, 4]],
        "total_blocks": 6,
        "unit_ap": 10,
        "ap_regen": 4,
        "max_rounds": 8,
        "rng_seed": 42
    });
    let rs = if rules.is_empty() {
        Ruleset::classic()
    } else {
        Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules, "basics": ["move", "rest", "strike", "guard"]}).as_object().unwrap())
    };
    let mut def = EncounterDef::from_obj_with(row.as_object().unwrap(), rs);
    def.bleed_out_rounds = bleed_out_rounds;
    let mut s = Session::start(Arc::new(catalog()), def, &TestRoster);
    for (c, pos) in [(HERO, V2::new(4, 4)), (MEDIC, V2::new(3, 4)), (GUEST, V2::new(2, 4)), (FOE, V2::new(6, 4))] {
        s.w.grid.positions[c] = Some(pos);
    }
    for sk in medic_skills {
        s.w.units[MEDIC].known_skills.push((*sk).to_string());
    }
    for it in medic_pouch {
        let item = s.w.cat.items[*it].clone();
        s.w.units[MEDIC].inventory.add_to_pouch(&item);
    }
    s.w.add_item_skills();
    s
}

#[test]
fn bleed_timer_decrements_at_round_end_and_only_party_heroes_bleed_out() {
    let mut s = start_session(&[], 3, &["t_revive"], &["t_salts"]);
    assert_eq!(s.w.bleed_heroes, vec![HERO, MEDIC], "guests and enemies never bleed out");
    // Park Foe far away with cleared steps so it does not wipe Medic mid-countdown.
    s.w.grid.positions[FOE] = Some(V2::new(11, 0));
    s.plan.clear(FOE);

    // Knock down party hero and guest.
    s.w.damage(HERO, 999, true);
    s.w.damage(GUEST, 999, true);
    assert_eq!(s.w.bleed_left(HERO), Some(3));
    assert!(s.w.can_revive(HERO));
    assert!(!s.w.is_dead(HERO));
    assert_eq!(s.w.bleed_left(GUEST), None, "guest never enters bleed-out");
    assert!(!s.w.can_revive(GUEST));

    // Enemy dropping to 0 HP does not enter bleed-out.
    let mut s_foe = start_session(&[], 3, &[], &[]);
    s_foe.w.damage(FOE, 999, true);
    assert_eq!(s_foe.w.bleed_left(FOE), None, "enemy never bleeds out");
    assert!(!s_foe.w.can_revive(FOE));

    assert!(s.confirm_plan());
    s.resolve_round();
    assert_eq!(s.w.bleed_left(HERO), Some(2));
    assert!(!s.w.is_dead(HERO));

    s.advance_round();
    s.plan.clear(FOE);
    assert!(s.confirm_plan());
    s.resolve_round();
    assert_eq!(s.w.bleed_left(HERO), Some(1));
    assert!(!s.w.is_dead(HERO));

    s.advance_round();
    s.plan.clear(FOE);
    assert!(s.confirm_plan());
    s.resolve_round();
    assert_eq!(s.w.bleed_left(HERO), None);
    assert!(s.w.is_dead(HERO));
    assert!(!s.w.can_revive(HERO));

    let fates: IndexMap<UnitId, PartyFate> = s.party_fates().into_iter().collect();
    assert_eq!(fates[&HERO], PartyFate::Dead);
    assert_eq!(fates[&MEDIC], PartyFate::Standing);
    assert!(s.log_lines(0, 0).iter().any(|l| l == "Hero bleeds out."), "expected bleed-out log line, got {:?}", s.log_lines(0, 0));

    // Dead hero cannot be targeted for revive by skill or consumable.
    s.advance_round();
    let rev = skill_id(&s.w.cat, "t_revive");
    assert!(!s.plan.add_step(&s.w, MEDIC, PlanStep::ally(rev, HERO), false).is_empty());
    assert!(!s.plan.add_step(&s.w, MEDIC, PlanStep::use_item("t_salts", HERO), false).is_empty());
}

#[test]
fn classic_bleed_zero_leaves_downed_in_non_revivable_stasis() {
    let mut s = start_session(&[], 0, &["t_revive"], &["t_salts"]);
    s.w.damage(HERO, 999, true);
    assert_eq!(s.w.bleed_left(HERO), None);
    assert!(!s.w.can_revive(HERO));
    assert!(!s.w.is_dead(HERO));

    let fates: IndexMap<UnitId, PartyFate> = s.party_fates().into_iter().collect();
    assert_eq!(fates[&HERO], PartyFate::Downed);

    let rev = skill_id(&s.w.cat, "t_revive");
    assert!(!s.plan.add_step(&s.w, MEDIC, PlanStep::ally(rev, HERO), false).is_empty());
    assert!(!s.plan.add_step(&s.w, MEDIC, PlanStep::use_item("t_salts", HERO), false).is_empty());
}

#[test]
fn revive_skill_and_item_resolve_in_place_or_radius_two_sorted_by_yx_else_refuse_blocked() {
    // Case 1: unoccupied -> revive in place with skill.
    let mut s = start_session(&[], 3, &["t_revive", "t_mend"], &["t_salts", "t_tonic"]);
    s.w.grid.positions[FOE] = Some(V2::new(11, 0));
    s.plan.clear(FOE);
    s.w.damage(HERO, 999, true);
    let rev = skill_id(&s.w.cat, "t_revive");
    let mend = skill_id(&s.w.cat, "t_mend");
    assert!(!s.plan.add_step(&s.w, MEDIC, PlanStep::ally(mend, HERO), false).is_empty(), "non-revive heal refuses");
    assert!(!s.plan.add_step(&s.w, MEDIC, PlanStep::use_item("t_tonic", HERO), false).is_empty(), "non-revive potion refuses");
    assert!(s.plan.add_step(&s.w, MEDIC, PlanStep::ally(rev, HERO), false).is_empty());
    assert!(s.confirm_plan());
    s.resolve_round();
    assert!(s.w.units[HERO].is_alive());
    assert_eq!(s.w.units[HERO].hp, 14);
    assert_eq!(s.w.pos(HERO), V2::new(4, 4));
    assert_eq!(s.w.bleed_left(HERO), None);

    // Case 2: downed cell (4,4) occupied by Foe; (3,3) blocked, (4,3) is exit -> (5,3) is smallest (y, x) at distance 1.
    let mut s2 = start_session(&[], 3, &[], &["t_salts"]);
    s2.w.damage(HERO, 999, true);
    s2.w.grid.positions[FOE] = Some(V2::new(4, 4));
    s2.plan.clear(FOE);
    s2.resolver.scheduled.clear();
    let rest2 = skill_id(&s2.w.cat, "rest");
    for _ in 0..6 {
        assert!(s2.plan.add_step(&s2.w, FOE, PlanStep::self_skill(rest2), false).is_empty());
    }
    let wall = s2.w.grid.table.rows.iter().position(|r| !r.walkable).expect("non-walkable terrain");
    s2.w.grid.terrain[(3 * GRID_W + 3) as usize] = wall;
    s2.w.grid.exits = vec![V2::new(4, 3)];
    assert_eq!(s2.w.revive_cell(HERO), Some(V2::new(5, 3)));
    assert!(s2.plan.add_step(&s2.w, MEDIC, PlanStep::use_item("t_salts", HERO), false).is_empty());
    assert!(s2.confirm_plan());
    s2.resolve_round();
    assert!(s2.w.units[HERO].is_alive());
    assert_eq!(s2.w.units[HERO].hp, 15);
    assert_eq!(s2.w.pos(HERO), V2::new(5, 3));

    // Case 3: radius 1..=2 entirely blocked -> both planner & round_intent & execution refuse with "Space is blocked".
    let mut s3 = start_session(&[ROUND_INTENT], 3, &["t_revive"], &["t_salts"]);
    s3.w.damage(HERO, 999, true);
    s3.w.grid.positions[FOE] = Some(V2::new(4, 4));
    s3.w.grid.positions[MEDIC] = Some(V2::new(5, 4)); // within item reach 1 & skill reach 3; still leaves (4,4) & all radius 1..=2 blocked!
    let wall3 = s3.w.grid.table.rows.iter().position(|r| !r.walkable).expect("non-walkable terrain");
    for y in 2..=6 {
        for x in 2..=6 {
            let p = V2::new(x, y);
            if p != V2::new(4, 4) && p != V2::new(5, 4) {
                s3.w.grid.terrain[(y * GRID_W + x) as usize] = wall3;
            }
        }
    }
    assert_eq!(s3.w.revive_cell(HERO), None);
    let rev3 = skill_id(&s3.w.cat, "t_revive");
    assert_eq!(s3.plan.add_step(&s3.w, MEDIC, PlanStep::ally(rev3, HERO), false), "Space is blocked");
    assert_eq!(s3.plan.add_step(&s3.w, MEDIC, PlanStep::use_item("t_salts", HERO), false), "Space is blocked");
    let opts = rip::menu_options(&s3.w, &s3.plan, MEDIC, RoundTarget::Unit(HERO));
    let rev_opt = opts.iter().find(|o| o.picks == vec![Pick::Skill { skill: rev3, kind: None }]).expect("revive row in menu");
    assert_eq!(rev_opt.problem.as_deref(), Some("Space is blocked"));
    let mut cmd = Command::new(
        MEDIC,
        0,
        1,
        CmdKind::Item {
            target: HERO,
            item: Box::new(s3.w.cat.items["t_salts"].clone()),
            used: false,
            last_heal: 0,
            last_damage: 0,
            last_boost: 0.0,
            cell: None,
            struck_units: Vec::new(),
        },
    );
    let mut rng = GodotRng::new(1);
    cmd.execute(&mut s3.w, &mut rng);
    assert!(cmd.failed && cmd.log.contains("Space is blocked"), "got {:?}", cmd.log);
}

#[test]
fn revived_unit_preserves_remaining_ap_and_cancelled_prior_plan_stays_cleared() {
    let mut s = start_session(&[], 3, &["t_revive"], &[]);
    let atk = skill_id(&s.w.cat, "strike");
    let rev = skill_id(&s.w.cat, "t_revive");
    let grd = skill_id(&s.w.cat, "guard");
    let rest = skill_id(&s.w.cat, "rest");

    // Foe at (4,3) is orthogonal distance 1 from Hero at (4,4) (no walking needed).
    // Block 1: Hero spends 1 AP on Guard (10 -> 9 AP), Foe strikes Hero to 0 HP (120 power * 0.7 > 50 HP),
    // cancelling Hero's block 2 Rest and block 3 Strike; Medic rests in block 1 and revives Hero in block 2.
    s.w.grid.positions[FOE] = Some(V2::new(4, 3));
    s.w.grid.positions[GUEST] = Some(V2::new(2, 4));
    s.plan.clear(FOE);
    s.resolver.scheduled.clear();
    assert!(s.plan.add_step(&s.w, HERO, PlanStep::self_skill(grd), false).is_empty());
    assert!(s.plan.add_step(&s.w, HERO, PlanStep::self_skill(rest), false).is_empty());
    assert!(s.plan.add_step(&s.w, HERO, PlanStep::engage(atk, FOE), false).is_empty());

    assert!(s.plan.add_step(&s.w, FOE, PlanStep::engage(atk, HERO), false).is_empty());
    while s.plan.add_step(&s.w, FOE, PlanStep::self_skill(rest), false).is_empty() {}

    assert!(s.plan.add_step(&s.w, MEDIC, PlanStep::self_skill(rest), false).is_empty());
    assert!(s.plan.add_step(&s.w, MEDIC, PlanStep::ally(rev, HERO), false).is_empty());

    let spent = s.plan.entries_for(&s.w, HERO)[0].total_ap();
    assert!(s.confirm_plan());
    s.resolve_round();

    assert!(s.w.units[HERO].is_alive(), "Hero should have been revived");
    assert_eq!(s.w.units[FOE].hp, s.w.units[FOE].max_hp, "cancelled Strike from before being downed must stay cleared");
    assert_eq!(s.w.units[HERO].ap, 10 - spent, "Hero keeps round AP less already spent before going down");

    // Next round: revived Hero resumes normal turn planning and can strike Foe.
    s.advance_round();
    s.plan.clear(FOE);
    s.resolver.scheduled.clear();
    assert!(s.plan.add_step(&s.w, HERO, PlanStep::engage(atk, FOE), false).is_empty());
}

#[test]
fn classic_and_round_intent_ai_prioritize_reviving_downed_bleeding_ally() {
    // Classic Support AI picks t_revive over patch/strike when Hero is bleeding.
    let mut s = start_session(&[], 3, &["t_mend", "t_revive"], &[]);
    s.w.damage(HERO, 999, true);
    let mut plan = Plan::new();
    let prof = s.w.profile_of(MEDIC);
    let steps = ai::build_steps(&s.w, &mut plan, MEDIC, &prof);
    let rev = skill_id(&s.w.cat, "t_revive");
    assert!(steps.iter().any(|st| st.skill == Some(rev) && st.target == Some(HERO)), "classic Support AI revives bleeding hero: {:?}", steps);

    // Non-support AI with use_items = true uses revive pouch item on adjacent bleeding ally.
    let mut s_item = start_session(&[], 3, &[], &["t_salts"]);
    s_item.w.damage(HERO, 999, true);
    let mut p_item = Plan::new();
    let mut agg = s_item.w.profile_of(MEDIC);
    agg.role = Role::Aggressor;
    agg.use_items = true;
    let steps_item = ai::build_steps(&s_item.w, &mut p_item, MEDIC, &agg);
    assert!(steps_item.iter().any(|st| st.item_id == "t_salts" && st.target == Some(HERO)), "item AI uses revive salts: {:?}", steps_item);

    // Round-intent AI selects Support round targeting bleeding Hero.
    let mut s_ri = start_session(&[ROUND_INTENT], 3, &["t_revive"], &[]);
    s_ri.w.damage(HERO, 999, true);
    let mut p_ri = Plan::new();
    let prof_ri = s_ri.w.profile_of(MEDIC);
    let _res = ai::plan_round_intent(&mut s_ri.w, &mut p_ri, MEDIC, &prof_ri, Tactic::Balanced);
    assert_eq!(
        s_ri.w.round_states.get(&MEDIC).and_then(|r| r.target),
        Some(RoundTarget::Unit(HERO)),
        "round-intent AI targets downed bleeding hero for revive"
    );
    assert_eq!(
        s_ri.w.round_states.get(&MEDIC).and_then(|r| r.intent),
        Some(RoundIntent::Support),
    );
}
