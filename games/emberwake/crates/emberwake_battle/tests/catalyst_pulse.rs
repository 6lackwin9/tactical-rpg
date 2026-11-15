//! GEAR-REWORK1 follow-up (`PR5-57`): `orb` off-hand action `catalyst_pulse` (`1` AP, `1` block, range `1-2`, `[PROVISIONAL]`).

use emberwake_battle::grid::{V2, GRID_H, GRID_W};
use emberwake_battle::offhand;
use emberwake_battle::round_intent::RoundTarget;
use emberwake_battle::round_intent_plan::{self, Pick};
use emberwake_battle::session::{EncounterDef, Phase, Session};
use emberwake_data::UnitDef;
use emberwake_rules::ruleset::{Ruleset, TIMELINE};
use emberwake_world::Data;
use indexmap::IndexMap;
use std::path::Path;

#[test]
fn orb_exposes_and_executes_catalyst_pulse_on_the_shipped_mage() {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    let d = Data::open(&dir).expect("data").0;

    let orb = d.cat.items.get("wpn_orb").expect("wpn_orb in catalog");
    assert_eq!(orb.off_hand_skill, "catalyst_pulse");
    assert!(d.cat.skill("catalyst_pulse").is_some(), "catalyst_pulse skill shipped");

    let ruleset = Ruleset::of(&d.cat, TIMELINE).expect("timeline ruleset");
    let mut def = EncounterDef::from_obj_with(&d.cat.encounters["tl_duel_reach"], ruleset);
    def.party = vec!["dojo_mage".into()];
    let mut s = Session::start(d.cat.clone(), def, &IndexMap::<String, UnitDef>::new());
    let mage = s.w.allies[0];
    let foe = s.w.enemies[0];

    let mut pair = None;
    'search: for y in 0..GRID_H {
        for x in 0..(GRID_W - 2) {
            let a = V2::new(x, y);
            let b = V2::new(x + 2, y);
            if s.w.grid.is_walkable(a) && !s.w.grid.is_exit(a) && s.w.grid.is_walkable(b) && !s.w.grid.is_exit(b) {
                pair = Some((a, b));
                break 'search;
            }
        }
    }
    let (a_cell, b_cell) = pair.expect("two walkable non-exit cells 2 tiles apart");
    for u in 0..s.w.units.len() {
        s.w.remove(u);
    }
    assert!(s.w.place(mage, a_cell) && s.w.place(foe, b_cell));

    let act = offhand::action(&s.w, mage).expect("orb exposes offhand::action");
    assert_eq!(act.item, orb.name);
    assert_eq!(act.problem, None);
    let skill = &s.w.cat.skills[act.skill];
    assert_eq!(skill.id, "catalyst_pulse");
    assert_eq!((skill.ap_cost, skill.blocks, skill.reach_min, skill.reach_max), (1, 1, 1, 2));
    assert_eq!(skill.stagger, 8);
    assert_eq!(skill.status_effect, "expose");
    assert!(s.w.cat.skills.iter().filter(|sk| sk.weapon_types.iter().any(|w| w == "sprayer")).all(|sk| sk.spell_source), "all sprayer skills resolve spell_source");

    let ap_before = s.w.units[mage].ap;
    let foe_hp_before = s.w.units[foe].hp;
    round_intent_plan::set_autofill(&mut s.w, &mut s.plan, false);
    let (picked, later) = round_intent_plan::pick(&mut s.w, &mut s.plan, mage, RoundTarget::Unit(foe), &[Pick::Skill { skill: act.skill, kind: None }]);
    assert_eq!(picked, Ok(()));
    assert!(later.is_empty());

    let entries = s.plan.entries_for(&s.w, mage);
    assert_eq!(entries[0].skill, act.skill);
    assert_eq!(entries[0].total_ap(), 1);
    assert_eq!(entries[0].last_block(&s.w) - entries[0].start_block + 1, 1);

    assert!(s.confirm_plan());
    for _ in 0..100 {
        if s.phase == Phase::Replay {
            break;
        }
        s.step_next();
    }
    assert_eq!(s.w.pos(mage), a_cell, "range 1-2 casts without chasing");
    assert_eq!(s.w.units[mage].ap, ap_before - 1);
    assert!(s.w.units[foe].hp < foe_hp_before, "catalyst_pulse dealt damage");
}
