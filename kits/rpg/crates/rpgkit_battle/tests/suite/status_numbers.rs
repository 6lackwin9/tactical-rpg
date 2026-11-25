//! S1.2-S2 and S1.2-S3 (docs/sessions/S1_STATUS.md sections 1-4) in a battle, on a scratch catalog built here (no
//! shipped data): the round-end ticks run the `tick` rows in the old order (bleed, poison, burn, then regen) and bleed
//! deals the ruleset's `bleed_pct` of max HP; riposte's counter scales with the ruleset's `counter_mult`; a trigger's
//! `resist` effect folds by U4's combine rule for `resist` (summed, capped). Expectations come from the formulas.

use indexmap::IndexMap;
use rpgkit_battle::command;
use rpgkit_battle::grid::V2;
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::trigger;
use rpgkit_battle::world::UnitId;
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, TIMELINE, TRIGGERS};
use rpgkit_rules::stats::roundi;
use rpgkit_rules::status;
use serde_json::json;
use std::sync::Arc;

const HP: i32 = 400;
const HERO: UnitId = 0;
const FOE: UnitId = 1;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": HP, "power": 100, "armor": 0, "speed": 10, "attack_range": 1});
    Catalog::from_value(&json!({"skills": skills, "units": [unit("p_hero", "party"), unit("p_foe", "enemy")]}))
}

/// The hero (4,4) faces the foe (5,4).
fn session(ruleset: Ruleset) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["p_hero"], "enemies": ["p_foe"], "total_blocks": 6, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(4, 4)), (FOE, V2::new(5, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

#[test]
fn the_round_end_ticks_run_the_tick_rows_and_bleed_reads_bleed_pct() {
    assert_eq!(Ruleset::classic().numbers.bleed_pct, 0.06, "S1 section 3 default, the old BLEED_PCT");
    for pct in [0.06, 0.25] {
        let mut ruleset = Ruleset::classic();
        ruleset.numbers.bleed_pct = pct;
        let mut s = session(ruleset);
        s.w.units[FOE].hp = HP / 2;
        for (st, mag) in [("regen", 4), ("burn", 5), ("poison", 3), ("bleed", 0)] {
            status::grant_now(&mut s.w.units[FOE], st, 2, mag, "");
        }
        s.resolver.apply_round_end_effects(&mut s.w);
        let steps: Vec<(&str, i32)> = s.resolver.round_end_steps.iter().filter(|r| r.unit == FOE).map(|r| (r.status, r.amount)).collect();
        let pct_of = |p: f64| roundi(HP as f64 * p).max(1);
        assert_eq!(steps, [("bleed", pct_of(pct)), ("poison", 3), ("burn", pct_of(0.05)), ("regen", pct_of(0.04))], "bleed_pct {pct}");
    }
}

#[test]
fn the_riposte_counter_scales_with_counter_mult() {
    let counter = |mult: f64| {
        let mut ruleset = Ruleset::classic();
        ruleset.numbers.counter_mult = mult;
        let mut s = session(ruleset);
        status::grant_now(&mut s.w.units[FOE], "riposte", 2, 0, "");
        let mut rng = s.resolver.rng.clone();
        command::riposte_counter(&mut s.w, &mut rng, HERO, FOE, false).expect("an adjacent melee hit is countered").damage
    };
    let (low, high) = (counter(0.1), counter(1.5));
    assert!(low > 0 && high > low, "counter_mult 0.1 deals {low}, 1.5 deals {high}");
    assert_eq!(Ruleset::classic().numbers.counter_mult, 0.5, "S1 section 3 default, the old COUNTER_MULT");
    assert_eq!(counter(Ruleset::classic().numbers.counter_mult), counter(0.5));
}

#[test]
fn a_trigger_resist_folds_by_the_resist_combine_rule() {
    const GAIN: i32 = 60;
    let mut s = session(Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [TRIGGERS]}).as_object().unwrap()));
    let cat = s.w.cat.clone();
    let e = json!({"type": "trigger", "on": "round_start", "when": {}, "effects": [{"type": "resist", "status": "stun", "value": GAIN}], "limit": "none"});
    s.w.units[HERO].apply_effects(&cat, &[e], "passive:steel", false);
    // U4's resist rule: summed, capped at 100
    for want in [GAIN, 100] {
        trigger::round_start(&mut s.resolver, &mut s.w);
        assert_eq!(s.w.units[HERO].status_resist.get("stun").copied(), Some(want));
    }
}
