//! A4.2 (docs/sessions/A4_EFFECTS.md section 1, acceptance 2; C08, FB-57(5), FB-58): the 5 status vocabularies (skill
//! `inflict` / `grant`, trigger `inflict` / `grant`, ground hazard rows, arena hazard effects, encounter phase `status`
//! effects) normalize into one `EffectSpec` routed through S1 (`status::apply_effect`). Each vocabulary's spec lands
//! exactly as its direct S1 call did before (same status state, same RNG draws), and all five share the resistance and
//! stacking rules. Rows are picked from the status table by rule and the vocabulary rows are built here, never read from
//! shipped data.

use aegis_core::rng::GodotRng;
use aegis_core::value::Obj;
use rpgkit_battle::trigger::BATTLE_ROUNDS;
use rpgkit_battle::grid::{self, Hazard, TerrainTable, TERRAIN_SOURCE};
use rpgkit_battle::{arena, encounter, ground, trigger};
use rpgkit_data::{Catalog, HazardDef};
use rpgkit_rules::status::{self, EffectSpec, Stacking, Timing};
use rpgkit_rules::{build_unit, Unit};
use serde_json::{json, Value};

const SEED: u64 = 41;
const POTENCY: i32 = 60;

fn unit() -> Unit {
    let cat = Catalog::from_value(&json!({"units": [{"id": "dummy", "name": "Dummy", "stat_mode": "fixed", "hp": 40}]}));
    build_unit(&cat, &cat.units["dummy"], 10)
}

fn obj(v: Value) -> Obj {
    v.as_object().unwrap().clone()
}

/// A resistable status row (rolled against resistance).
fn resistable() -> &'static str {
    status::ROWS.iter().find(|r| r.resistable).expect("a resistable row").id
}

/// A resistable row that stacks potency (its magnitude adds up to the cap).
fn stacking_debuff() -> &'static str {
    status::ROWS.iter().find(|r| r.resistable && status::stacking_of(r.id) == Stacking::StackPotency).expect("a resistable stack_potency row").id
}

/// A non-resistable `defend` grant row (a DEFEND buff, active at once).
fn defend_buff() -> &'static str {
    status::defend_buffs().iter().find(|s| !status::resistable(s)).expect("a non-resistable defend buff")
}

/// A non-resistable buff that is not a DEFEND buff (queued by a skill grant).
fn other_buff() -> &'static str {
    status::ROWS.iter().find(|r| !r.resistable && r.buff && !status::defend_buffs().contains(&r.id)).expect("a non-resistable buff").id
}

/// The status state and the RNG's next draw: equal when two paths applied the same thing with the same draws.
fn outcome(u: &Unit, rng: &mut GodotRng) -> (String, i32) {
    (format!("{:?}", u.st), rng.randi_range(1, 1_000_000))
}

/// Applies `spec` with `timing` `times` times to a fresh unit from a fresh RNG; the landed flags and the outcome.
fn through_spec(spec: &EffectSpec, timing: Timing, bonus: f64, times: usize) -> (Vec<bool>, (String, i32)) {
    let (mut u, mut rng) = (unit(), GodotRng::new(SEED));
    let landed = (0..times).map(|_| status::apply_effect(&mut u, spec, timing, bonus, Some(&mut rng))).collect();
    (landed, outcome(&u, &mut rng))
}

/// Skill `inflict` (Attack / Area / dojo): an inflict spec with the attacker's KNOW term lands like `try_inflict_with`,
/// and shows the same chance; skill grants (DEFEND `grant_status`, Support `grant_status`, `extra_grants`) are never
/// rolled, a DEFEND buff lands now and anything else next round, as `grant_now` / `queue` did.
#[test]
fn skill_inflict_and_grant_route_through_effect_spec() {
    let st = resistable();
    let bonus = 7.0;
    let spec = EffectSpec::inflict(st, POTENCY, 2, 0, "Hero");
    assert_eq!(spec.resistable, None, "a skill inflict takes its row's resistable");
    assert!(spec.rolled());
    assert_eq!(spec.chance(&unit(), bonus), status::apply_chance_with(&unit(), st, POTENCY, bonus));
    for seed in 0..20 {
        let (mut a, mut ra) = (unit(), GodotRng::new(seed));
        let (mut b, mut rb) = (unit(), GodotRng::new(seed));
        let x = status::apply_effect(&mut a, &spec, Timing::Next, bonus, Some(&mut ra));
        let y = status::try_inflict_with(&mut b, st, POTENCY, bonus, Some(&mut rb), 2, 0, "Hero");
        assert_eq!((x, outcome(&a, &mut ra)), (y, outcome(&b, &mut rb)), "seed {seed}");
    }
    for (buff, defend) in [(defend_buff(), true), (defend_buff(), false), (other_buff(), true)] {
        let spec = EffectSpec::grant(buff, 2, 0, "Hero");
        assert_eq!(spec.resistable, Some(false));
        let timing = status::grant_timing(buff, defend);
        assert_eq!(timing, if defend && buff == defend_buff() { Timing::Now } else { Timing::Next });
        let mut a = unit();
        assert!(status::apply_effect(&mut a, &spec, timing, 0.0, None));
        let mut b = unit();
        if timing == Timing::Now {
            status::grant_now(&mut b, buff, 2, 0, "Hero");
        } else {
            status::queue(&mut b, buff, 2, 0, "Hero");
        }
        assert_eq!(format!("{:?}", a.st), format!("{:?}", b.st), "{buff} defend={defend}");
        let mut c = unit();
        let rows = [json!({"status": buff, "rounds": 2})];
        assert_eq!(status::grant_extras(&mut c, &rows, defend, "Hero"), [buff.to_string()]);
        assert_eq!(format!("{:?}", c.st), format!("{:?}", b.st), "extra_grants {buff} defend={defend}");
    }
}

/// Trigger `inflict` / `grant`: `trigger::effect_of` keeps the old defaults (inflict potency 100, rounds 1, next round;
/// grant at least 1 round, a negative count the whole battle, now) and lands like `try_inflict` / `grant_now`.
#[test]
fn trigger_inflict_and_grant_route_through_effect_spec() {
    let st = resistable();
    let (spec, timing) = trigger::effect_of(&obj(json!({"type": "inflict", "status": st})), "Holder").unwrap();
    assert_eq!(spec, EffectSpec { status_id: st.into(), potency: 100, rounds: 1, magnitude: 0, resistable: None, source: "Holder".into() });
    assert_eq!(timing, Timing::Next);
    let (spec, timing) = trigger::effect_of(&obj(json!({"type": "inflict", "status": st, "potency": POTENCY, "rounds": 3})), "Holder").unwrap();
    for seed in 0..20 {
        let (mut a, mut ra) = (unit(), GodotRng::new(seed));
        let (mut b, mut rb) = (unit(), GodotRng::new(seed));
        let x = status::apply_effect(&mut a, &spec, timing, 0.0, Some(&mut ra));
        let y = status::try_inflict(&mut b, st, POTENCY, Some(&mut rb), 3, 0, "Holder");
        assert_eq!((x, outcome(&a, &mut ra)), (y, outcome(&b, &mut rb)), "seed {seed}");
    }
    let buff = defend_buff();
    for (rounds, want) in [(-1, BATTLE_ROUNDS), (0, 1), (3, 3)] {
        let (spec, timing) = trigger::effect_of(&obj(json!({"type": "grant", "status": buff, "rounds": rounds, "magnitude": 5})), "Holder").unwrap();
        assert_eq!((spec.rounds, spec.resistable, timing), (want, Some(false), Timing::Now));
        let (mut a, mut b) = (unit(), unit());
        status::apply_effect(&mut a, &spec, timing, 0.0, None);
        status::grant_now(&mut b, buff, want, 5, "Holder");
        assert_eq!(format!("{:?}", a.st), format!("{:?}", b.st));
    }
    assert!(trigger::effect_of(&obj(json!({"type": "grant", "status": ""})), "Holder").is_none(), "an empty status does nothing");
    assert!(trigger::effect_of(&obj(json!({"type": "crit", "status": st})), "Holder").is_none(), "only grant and inflict");
}

/// Ground hazard rows: `ground::effect_of` is an inflict with the row's potency, status rounds and magnitude and the
/// placer as the source, landing like `try_inflict`.
#[test]
fn ground_hazard_rows_route_through_effect_spec() {
    let st = stacking_debuff();
    let def = HazardDef::from_obj(&obj(json!({"id": "h", "status": st, "potency": POTENCY, "status_rounds": 2, "magnitude": 3})));
    let spec = ground::effect_of(&def, "Placer");
    assert_eq!(spec, EffectSpec::inflict(st, POTENCY, 2, 3, "Placer"));
    for seed in 0..20 {
        let (mut a, mut ra) = (unit(), GodotRng::new(seed));
        let (mut b, mut rb) = (unit(), GodotRng::new(seed));
        let x = status::apply_effect(&mut a, &spec, Timing::Next, 0.0, Some(&mut ra));
        let y = status::try_inflict(&mut b, st, POTENCY, Some(&mut rb), 2, 3, "Placer");
        assert_eq!((x, outcome(&a, &mut ra)), (y, outcome(&b, &mut rb)), "seed {seed}");
    }
}

/// P1.2-S2 (FB-51d): terrain hazard rows (`hazard_status` / `hazard_potency` / `hazard_rounds` / `hazard_magnitude`)
/// share the vocabulary: `grid::effect_of` is an inflict with the row's numbers and the source "terrain", landing like
/// the direct `try_inflict` the terrain roll made before (same draws, same stacking, one resistance formula), a
/// non-resistable row with no draw, and nothing on a dead unit.
#[test]
fn terrain_hazard_rows_route_through_effect_spec() {
    let st = stacking_debuff();
    let row = json!({"id": "burning_ground", "name": "Burning", "hazard_status": st, "hazard_potency": POTENCY, "hazard_rounds": 2, "hazard_magnitude": 3, "hazard_on": "enter"});
    let table = TerrainTable::from_catalog(&Catalog::from_value(&json!({"terrain": [row]})));
    let i = grid::TERRAIN_IDS.iter().position(|t| *t == "burning_ground").unwrap();
    let h = table.rows[i].hazard.clone().expect("the row's hazard");
    let spec = grid::effect_of(&h);
    assert_eq!(spec, EffectSpec::inflict(st, POTENCY, 2, 3, TERRAIN_SOURCE));
    assert_eq!(TERRAIN_SOURCE, "terrain", "the source the terrain roll always carried");
    assert_eq!(spec.chance(&unit(), 0.0), status::apply_chance(&unit(), st, POTENCY), "one resistance formula");
    let mut landed = Vec::new();
    for seed in 0..20 {
        let (mut a, mut ra) = (unit(), GodotRng::new(seed));
        let (mut b, mut rb) = (unit(), GodotRng::new(seed));
        for _ in 0..3 {
            let x = status::apply_effect(&mut a, &spec, Timing::Next, 0.0, Some(&mut ra));
            let y = status::try_inflict(&mut b, st, POTENCY, Some(&mut rb), 2, 3, "terrain");
            assert_eq!(x, y, "seed {seed}");
            landed.push(x);
        }
        assert_eq!(outcome(&a, &mut ra), outcome(&b, &mut rb), "seed {seed}");
    }
    assert!(landed.contains(&true) && landed.contains(&false), "potency {POTENCY} both lands and is resisted");
    // A non-resistable row: queued with no draw on a living unit; nothing and no draw on a dead one.
    let free = other_buff();
    let h = Hazard { status: free.into(), potency: POTENCY, magnitude: 0, rounds: 1, on: "both".into() };
    let spec = grid::effect_of(&h);
    let (mut a, mut ra) = (unit(), GodotRng::new(SEED));
    let (mut b, mut rb) = (unit(), GodotRng::new(SEED));
    assert!(status::apply_effect(&mut a, &spec, Timing::Next, 0.0, Some(&mut ra)));
    assert!(status::try_inflict(&mut b, free, POTENCY, Some(&mut rb), 1, 0, "terrain"));
    assert_eq!(outcome(&a, &mut ra), outcome(&b, &mut rb), "queued with no draw");
    let (mut dead, mut rng) = (unit(), GodotRng::new(SEED));
    dead.hp = 0;
    let before = format!("{:?}", dead.st);
    assert!(!status::apply_effect(&mut dead, &spec, Timing::Next, 0.0, Some(&mut rng)), "a dead unit takes nothing");
    assert_eq!(format!("{:?}", dead.st), before);
    assert_eq!(rng.randi_range(1, 1_000_000), GodotRng::new(SEED).randi_range(1, 1_000_000), "no draw spent");
}

/// Arena hazard effects: `arena::effect_of` is an inflict with the effect's numbers and the hazard's name as the source,
/// landing like `try_inflict`.
#[test]
fn arena_hazard_effects_route_through_effect_spec() {
    let st = resistable();
    let e = arena::Effect { status: st.into(), potency: POTENCY, status_rounds: 2, magnitude: 0, ..Default::default() };
    let spec = arena::effect_of(&e, "Rockfall");
    assert_eq!(spec, EffectSpec::inflict(st, POTENCY, 2, 0, "Rockfall"));
    for seed in 0..20 {
        let (mut a, mut ra) = (unit(), GodotRng::new(seed));
        let (mut b, mut rb) = (unit(), GodotRng::new(seed));
        let x = status::apply_effect(&mut a, &spec, Timing::Next, 0.0, Some(&mut ra));
        let y = status::try_inflict(&mut b, st, POTENCY, Some(&mut rb), 2, 0, "Rockfall");
        assert_eq!((x, outcome(&a, &mut ra)), (y, outcome(&b, &mut rb)), "seed {seed}");
    }
}

/// Encounter phase `status` effects (FB-58): `encounter::phase_effect` is never rolled (even a resistable row), so a
/// phase draws no RNG; it lands now for its rounds (default 2) with the source `phase:<id>`, like `grant_now`.
#[test]
fn phase_status_effects_route_through_effect_spec_without_a_draw() {
    let st = resistable();
    let spec = encounter::phase_effect(&obj(json!({"type": "status", "unit": "boss", "status": st})), "rage");
    assert_eq!(spec, EffectSpec::grant(st, 2, 0, "phase:rage"));
    assert!(!spec.rolled(), "a phase status effect is not rolled");
    let (mut a, mut ra) = (unit(), GodotRng::new(SEED));
    assert!(status::apply_effect(&mut a, &spec, Timing::Now, 0.0, Some(&mut ra)));
    let (mut b, mut rb) = (unit(), GodotRng::new(SEED));
    status::grant_now(&mut b, st, 2, 0, "phase:rage");
    assert_eq!(outcome(&a, &mut ra), outcome(&b, &mut rb), "no draw, same state");
    assert!(a.st.active.contains(st), "active now");
}

/// Acceptance 2 (C08): the same status from each vocabulary follows one resistance rule (the inflicts roll alike, draw
/// for draw; a phase or grant spec made resistable rolls the same way) and one stacking rule (twice, by the row's
/// stacking, the same in every vocabulary, rolled or granted).
#[test]
fn all_five_vocabularies_share_resistance_and_stacking() {
    let st = stacking_debuff();
    let (rounds, magnitude) = (2, 3);
    let trig = trigger::effect_of(&obj(json!({"type": "inflict", "status": st, "potency": POTENCY, "rounds": rounds, "magnitude": magnitude})), "S").unwrap().0;
    let hz = ground::effect_of(&HazardDef::from_obj(&obj(json!({"status": st, "potency": POTENCY, "status_rounds": rounds, "magnitude": magnitude}))), "S");
    let ar = arena::effect_of(&arena::Effect { status: st.into(), potency: POTENCY, status_rounds: rounds, magnitude, ..Default::default() }, "S");
    let mut phase = encounter::phase_effect(&obj(json!({"status": st, "rounds": rounds, "magnitude": magnitude})), "p");
    phase.potency = POTENCY;
    phase.source = "S".into();
    phase.resistable = None;
    let skill = EffectSpec::inflict(st, POTENCY, rounds, magnitude, "S");
    let inflicts = [("skill", &skill), ("trigger", &trig), ("ground", &hz), ("arena", &ar), ("phase (row resistable)", &phase)];
    let base = through_spec(&skill, Timing::Next, 0.0, 6);
    assert!(base.0.contains(&true) && base.0.contains(&false), "potency {POTENCY} both lands and is resisted over 6 rolls: {:?}", base.0);
    for (name, spec) in inflicts {
        assert_eq!(spec.chance(&unit(), 0.0), status::apply_chance(&unit(), st, POTENCY), "{name}: one resistance formula");
        assert_eq!(through_spec(spec, Timing::Next, 0.0, 6), base, "{name}: same rolls, same stacking");
    }
    // Not rolled: the grant vocabularies and a phase effect stack the same way, now or next round, with no draw.
    let skill_grant = EffectSpec::grant(st, rounds, magnitude, "S");
    let trig_grant = trigger::effect_of(&obj(json!({"type": "grant", "status": st, "rounds": rounds, "magnitude": magnitude})), "S").unwrap().0;
    let phase_grant = EffectSpec { source: "S".into(), ..encounter::phase_effect(&obj(json!({"status": st, "rounds": rounds, "magnitude": magnitude})), "p") };
    for timing in [Timing::Now, Timing::Next] {
        let want = through_spec(&skill_grant, timing, 0.0, 2);
        assert_eq!(want.0, [true, true]);
        assert_eq!(want.1 .1, GodotRng::new(SEED).randi_range(1, 1_000_000), "{timing:?}: a grant draws nothing");
        for (name, spec) in [("trigger grant", &trig_grant), ("phase", &phase_grant)] {
            assert_eq!(through_spec(spec, timing, 0.0, 2), want, "{name} {timing:?}: same stacking, no draw");
        }
    }
}

/// An inflict of a non-resistable row lands like `try_inflict_with` did: queued with no draw on a living target, and
/// nothing (false, no draw) on a dead one; a grant is the only path that skips that guard.
#[test]
fn a_non_resistable_inflict_keeps_the_dead_target_guard() {
    let id = other_buff();
    let spec = EffectSpec::inflict(id, POTENCY, 1, 0, "test");
    let (mut alive, mut rng) = (unit(), GodotRng::new(SEED));
    let (mut twin, mut twin_rng) = (unit(), GodotRng::new(SEED));
    assert!(status::apply_effect(&mut alive, &spec, Timing::Next, 0.0, Some(&mut rng)));
    assert!(status::try_inflict_with(&mut twin, id, POTENCY, 0.0, Some(&mut twin_rng), 1, 0, "test"));
    assert_eq!(outcome(&alive, &mut rng), outcome(&twin, &mut twin_rng), "queued with no draw, as try_inflict_with");
    let (mut dead, mut rng) = (unit(), GodotRng::new(SEED));
    dead.hp = 0;
    let before = format!("{:?}", dead.st);
    assert!(!status::apply_effect(&mut dead, &spec, Timing::Next, 0.0, Some(&mut rng)), "a dead target takes no inflict");
    assert_eq!(format!("{:?}", dead.st), before, "nothing queued on a dead target");
    assert_eq!(rng.randi_range(1, 1_000_000), GodotRng::new(SEED).randi_range(1, 1_000_000), "no draw spent");
}
