//! PK1-KITFIX (a), (b), (d) (FB-184 Option A, FB-181 (b); `docs/sessions/PK1_CORRUPTION.md` 2.1) on a scratch catalog
//! built here (no shipped data). Expectations come from the rules: a unit meter row's `triggers` run `meter_set` (the
//! crisis at round end sets its meter to the value); `inflict` with `rounds` -1 lasts the rest of the battle
//! (`trigger::BATTLE_ROUNDS`, as `grant`); `rating_set` holds a rating at its value at once, whatever the statuses on
//! that rating add, for the rest of the round and the next `rounds` rounds (or the battle), a later set of the same
//! rating replacing it; the planner's `est_heal` (skill and item) applies the target's `heal_taken` as `World::heal`
//! does, and is unchanged without it.

use rpgkit_battle::ai;
use rpgkit_battle::command;
use rpgkit_battle::grid::V2;
use rpgkit_battle::meter;
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::session::{EncounterDef, Roster, Session};
use rpgkit_battle::trigger::{self, BATTLE_ROUNDS};
use rpgkit_battle::world::{heal_taken, UnitId, World};
use rpgkit_data::{Catalog, ItemDef, PassiveDef, UnitDef};
use rpgkit_rules::damage::armor_rating;
use rpgkit_rules::ruleset::{Ruleset, TIMELINE, TRIGGERS, UNIT_METERS};
use rpgkit_rules::status::{self, EffectSpec, Rating, Timing};
use rpgkit_rules::Overrides;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const HP: i32 = 500;
/// The hero's and the ally's fixed armour (the base rating a set overrides).
const ARMOR: i32 = 30;
/// Strain: max MAX, the hero starts at MAX; the crisis (at MAX, round end) sets it to AFTER.
const MAX: i32 = 100;
const AFTER: i32 = 50;
/// Tension: the hero starts at 30, in its band from 25 holding `heal_taken` HEAL_TAKEN.
const HEAL_TAKEN: i32 = -25;
const MEND: i32 = 40;

/// The crisis shape of PK1 2.1 with one table row (so the draw is fixed): the armour held at 0 and a bleed for the
/// battle, then Strain set to AFTER.
fn crisis() -> Value {
    json!({"on": "round_end", "when": {"meter": {"id": "strain", "at_least": MAX}}, "limit": "none", "effects": [
        {"type": "roll_table", "rows": [{"weight": 1, "effects": [
            {"type": "rating_set", "rating": "armor", "value": 0, "rounds": -1},
            {"type": "inflict", "status": "bleed", "potency": 100, "rounds": -1}]}]},
        {"type": "meter_set", "meter": "strain", "value": AFTER}]})
}

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 1, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "t_mend", "name": "Mend", "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 1, "target_mode": "ALLY_OR_SELF", "reach_min": 0, "reach_max": 3, "heal_base": MEND}),
    ];
    let meters = vec![
        json!({"id": "strain", "name": "Strain", "short": "St", "min": 0, "max": MAX, "start": MAX, "show": "bar", "bands": [{"id": "calm", "from": 0}], "triggers": [crisis()]}),
        json!({"id": "tension", "name": "Tension", "short": "Te", "min": 0, "max": 100, "start": 30, "show": "bar",
            "bands": [{"id": "calm", "from": 0}, {"id": "tense", "from": 25, "effects": [{"type": "heal_taken", "value": HEAL_TAKEN}]}]}),
    ];
    let unit = |id: &str, side: &str, speed: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": HP, "power": 100, "armor": ARMOR, "speed": speed, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({
        "skills": skills,
        "unit_meters": meters,
        "units": [unit("p_hero", "party", 14), unit("p_ally", "party", 12), unit("p_foe", "enemy", 10)],
    }))
}

/// The hero holds the meters `meters` names (passives `m_strain`, `m_tension`).
struct Holders {
    meters: &'static [&'static str],
}

impl Roster for Holders {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        cat.units.get(r).cloned()
    }

    fn overrides(&self, _cat: &Catalog, r: &str) -> Overrides {
        let passives = if r == "p_hero" { self.meters.iter().map(|m| format!("m_{m}")).collect() } else { Vec::new() };
        Overrides { passives, ..Overrides::default() }
    }
}

fn session(meters: &'static [&'static str]) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [TRIGGERS, UNIT_METERS]}).as_object().unwrap());
    let mut cat = catalog();
    for m in ["strain", "tension"] {
        let id = format!("m_{m}");
        cat.passives.insert(id.clone(), PassiveDef { id, name: m.into(), effects: vec![json!({"type": "meter", "meter": m})], slot: String::new() });
    }
    let mut s = Session::start(Arc::new(cat), def, &Holders { meters });
    for (c, cell) in [(HERO, V2::new(4, 4)), (ALLY, V2::new(4, 5)), (FOE, V2::new(8, 8))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

/// A fresh resolver with the three units registered (as a battle's is).
fn registered() -> Resolver {
    let mut res = Resolver::new(BLOCKS, 1);
    for c in [HERO, ALLY, FOE] {
        res.register(c);
    }
    res
}

/// Gives `c` a passive trigger row (source `passive:<id>`, limit none).
fn give(w: &mut World, c: UnitId, id: &str, on: &str, effects: Value) {
    let cat = w.cat.clone();
    let e = json!({"type": "trigger", "on": on, "when": {}, "effects": effects, "limit": "none"});
    w.units[c].apply_effects(&cat, &[e], &format!("passive:{id}"), false);
}

/// A status granted to `c` now (never rolled).
fn grant(w: &mut World, c: UnitId, st: &str, rounds: i32, magnitude: i32) {
    status::apply_effect(&mut w.units[c], &EffectSpec::grant(st, rounds, magnitude, "test"), Timing::Now, 0.0, None);
}

/// `c`'s round start (its statuses and held ratings count down).
fn roll(w: &mut World, c: UnitId) {
    let n = w.ruleset.numbers.clone();
    status::roll_over(&mut w.units[c], 0, &n);
}

// ---- (a) meter_set in a meter row's triggers, with (b) in the crisis -----------------------------------------------

/// At round end the hero at Strain MAX runs the crisis: the armour rating is 0 whatever fortify adds, a bleed lands for
/// the battle, Strain is AFTER; rounds later the rating is still 0 and the bleed still runs; at AFTER the crisis does
/// not fire again.
#[test]
fn the_crisis_sets_its_meter_holds_the_armour_and_bleeds_for_the_battle() {
    let mut w = session(&["strain"]).w;
    assert_eq!(meter::value(&w, HERO, "strain"), Some(MAX));
    // the fixture: no resistance to bleed, so the inflict always lands
    w.units[HERO].status_resist.insert("bleed".into(), -100);
    assert_eq!(EffectSpec::inflict("bleed", 100, 1, 0, "test").chance(&w.units[HERO], 0.0), 100, "the fixture's bleed always lands");
    grant(&mut w, HERO, "fortify", 2, 10);
    assert_eq!(armor_rating(&w.units[HERO]), ARMOR + 10, "fortify raises the rating before the crisis");
    let mut res = registered();
    trigger::round_end(&mut res, &mut w);
    assert_eq!(meter::value(&w, HERO, "strain"), Some(AFTER), "{:?}", res.log);
    assert_eq!(armor_rating(&w.units[HERO]), 0, "the set wins over fortify");
    assert!(res.log.iter().any(|l| l.contains("Armor held at 0 for the battle")), "{:?}", res.log);
    assert!(res.log.iter().any(|l| l.contains(&format!("set to {AFTER}"))), "{:?}", res.log);
    assert!(w.units[HERO].st.has_any("bleed"), "the bleed lands next round");
    for round in 1..=12 {
        roll(&mut w, HERO);
        assert_eq!(armor_rating(&w.units[HERO]), 0, "round {round}");
        assert!(w.units[HERO].has_status("bleed"), "round {round}: the bleed lasts the battle");
    }
    let mut res = registered();
    trigger::round_end(&mut res, &mut w);
    assert_eq!(meter::value(&w, HERO, "strain"), Some(AFTER), "below MAX the crisis does not fire");
    // the ally holds no Strain and nothing set
    assert_eq!(armor_rating(&w.units[ALLY]), ARMOR);
}

/// PR5-26 (FB-191 Option A): the battle's round end runs the `round_end` triggers before the meters' decay, so a
/// Strain at MAX with a decay row (-DECAY a round, not gained) fires the crisis (the armour held at 0, Strain set to
/// AFTER); a Strain one below MAX decays without a crisis. (Decay first would take MAX below the crisis's `at_least`
/// before it reads it.) PR5-26b (FB-198 Q2 Option A, PK1_CORRUPTION 2.1): the crisis's `meter_set` skips that round's
/// decay, so Strain ends the round at AFTER (no decay line), and the next round decays from AFTER as usual.
#[test]
fn the_round_end_crisis_reads_the_meter_before_it_decays() {
    const DECAY: i32 = 10;
    for (start, crisis) in [(MAX, true), (MAX - 1, false)] {
        let mut w = session(&["strain"]).w;
        std::sync::Arc::make_mut(&mut w.cat).unit_meters.get_mut("strain").expect("the strain row").decay = vec![json!({"delta": -DECAY}).as_object().unwrap().clone()];
        w.meters.gauges.iter_mut().find(|g| g.unit == HERO && g.meter == "strain").expect("a gauge").value = start;
        let mut res = registered();
        res.apply_round_end_effects(&mut w);
        let after = if crisis { AFTER } else { start - DECAY };
        assert_eq!(meter::value(&w, HERO, "strain"), Some(after), "start {start}: {:?}", res.log);
        assert_eq!(armor_rating(&w.units[HERO]) == 0, crisis, "start {start}: the crisis held the armour");
        assert_eq!(res.log.iter().any(|l| l.contains(&format!("set to {AFTER}"))), crisis, "{:?}", res.log);
        assert_eq!(res.log.iter().any(|l| l.contains("decays")), !crisis, "the set skips the round's decay: {:?}", res.log);
        if crisis {
            // the set mark lasts one round: the next round end (no crisis below MAX) decays from AFTER
            let mut res = registered();
            res.apply_round_end_effects(&mut w);
            assert_eq!(meter::value(&w, HERO, "strain"), Some(AFTER - DECAY), "the next round decays: {:?}", res.log);
        }
    }
}

/// PR5-26b (FB-198 Q2 Option A): only a `round_end` trigger's `meter_set` skips the decay. A `round_end` trigger that
/// raises the meter (a `meter` delta, not a set) leaves the decay on (PR5-26's order: raise, then decay); a set by a
/// trigger on another event (a `round_start` row) marks nothing for the round's end. Per meter: the hero's crisis
/// skips its Strain's decay only, its Tension decays the same round.
#[test]
fn only_a_round_end_set_skips_the_decay() {
    const DECAY: i32 = 10;
    const RAISE: i32 = 5;
    const BASE: i32 = 60;
    let decay = || vec![json!({"delta": -DECAY}).as_object().unwrap().clone()];
    // a round_end raise: not a set, the decay still runs (BASE + RAISE - DECAY)
    let mut w = session(&["tension"]).w;
    std::sync::Arc::make_mut(&mut w.cat).unit_meters.get_mut("tension").expect("the tension row").decay = decay();
    w.meters.gauges.iter_mut().find(|g| g.unit == HERO && g.meter == "tension").expect("a gauge").value = BASE;
    give(&mut w, HERO, "swell", "round_end", json!([{"type": "meter", "meter": "tension", "delta": RAISE}]));
    let mut res = registered();
    res.apply_round_end_effects(&mut w);
    assert_eq!(meter::value(&w, HERO, "tension"), Some(BASE + RAISE - DECAY), "{:?}", res.log);
    // a set by a round_start row: the round's end still decays it (BASE - DECAY)
    let mut w = session(&["tension"]).w;
    std::sync::Arc::make_mut(&mut w.cat).unit_meters.get_mut("tension").expect("the tension row").decay = decay();
    give(&mut w, HERO, "reset", "round_start", json!([{"type": "meter_set", "meter": "tension", "value": BASE}]));
    let mut res = registered();
    trigger::round_start(&mut res, &mut w);
    assert_eq!(meter::value(&w, HERO, "tension"), Some(BASE), "{:?}", res.log);
    res.apply_round_end_effects(&mut w);
    assert_eq!(meter::value(&w, HERO, "tension"), Some(BASE - DECAY), "{:?}", res.log);
    // per meter: the hero's crisis set skips Strain's decay only; its Tension decays the same round
    let mut w = session(&["strain", "tension"]).w;
    let cat = std::sync::Arc::make_mut(&mut w.cat);
    cat.unit_meters.get_mut("strain").expect("the strain row").decay = decay();
    cat.unit_meters.get_mut("tension").expect("the tension row").decay = decay();
    w.meters.gauges.iter_mut().find(|g| g.unit == HERO && g.meter == "tension").expect("a gauge").value = BASE;
    let mut res = registered();
    res.apply_round_end_effects(&mut w);
    assert_eq!(meter::value(&w, HERO, "strain"), Some(AFTER), "{:?}", res.log);
    assert_eq!(meter::value(&w, HERO, "tension"), Some(BASE - DECAY), "{:?}", res.log);
}

// ---- (b) inflict for the battle ---------------------------------------------------------------------------------------

#[test]
fn inflict_minus_one_lasts_the_battle_as_a_grant_does() {
    let spec = |e: Value| trigger::effect_of(e.as_object().unwrap(), "test").expect("a status effect").0.rounds;
    assert_eq!(spec(json!({"type": "inflict", "status": "bleed", "rounds": -1})), BATTLE_ROUNDS);
    assert_eq!(spec(json!({"type": "grant", "status": "fortify", "rounds": -1})), BATTLE_ROUNDS);
    assert_eq!(spec(json!({"type": "inflict", "status": "bleed", "rounds": 3})), 3, "a count stays as written");
    assert_eq!(spec(json!({"type": "inflict", "status": "bleed"})), 1, "the default");
}

// ---- (b) rating_set -------------------------------------------------------------------------------------------------

/// A set with `rounds` R made at a round's end holds through the next R rounds and then lets go (the rating reads its
/// statuses again).
#[test]
fn a_rating_set_holds_through_its_next_rounds() {
    let mut w = session(&[]).w;
    give(&mut w, ALLY, "brittle", "round_end", json!([{"type": "rating_set", "rating": "armor", "value": 5, "rounds": 2}]));
    let mut res = registered();
    trigger::round_end(&mut res, &mut w);
    assert_eq!(armor_rating(&w.units[ALLY]), 5);
    assert!(res.log.iter().any(|l| l.contains("Armor held at 5 for 2 rounds")), "{:?}", res.log);
    for round in 1..=2 {
        roll(&mut w, ALLY);
        assert_eq!(armor_rating(&w.units[ALLY]), 5, "round {round}");
    }
    roll(&mut w, ALLY);
    assert_eq!(armor_rating(&w.units[ALLY]), ARMOR, "it lets go after its rounds");
    assert!(w.units[ALLY].rating_sets.is_empty());
}

/// One set per rating (a later one replaces it); each rating reads only its own set: armour the rating, move rate the
/// tiles per block, attack power the statuses' attack factor (weaken x empower) as a percent.
#[test]
fn each_rating_reads_its_own_set_and_a_later_set_replaces_it() {
    let mut w = session(&[]).w;
    grant(&mut w, HERO, "fortify", 3, 10);
    grant(&mut w, HERO, "haste", 3, 0);
    grant(&mut w, HERO, "empower", 3, 50);
    let u = &mut w.units[HERO];
    let (armor, mpb, factor) = (armor_rating(u), u.move_per_block(), u.weaken_factor() * u.empower_factor());
    assert_eq!(armor, ARMOR + 10);
    assert!(factor > 1.0, "empower raises the factor");
    u.set_rating(Rating::Armor, 3, -1);
    u.set_rating(Rating::Armor, 7, 1);
    assert_eq!(u.rating_sets.len(), 1, "one set per rating");
    assert_eq!(armor_rating(u), 7, "the later set replaces the first");
    assert_eq!((u.move_per_block(), u.weaken_factor() * u.empower_factor()), (mpb, factor), "the other ratings read their statuses");
    u.set_rating(Rating::MoveRate, 2, -1);
    assert_eq!(u.move_per_block(), 2, "haste adds nothing to a held move rate");
    u.set_rating(Rating::AttackPower, 50, -1);
    assert_eq!(u.weaken_factor() * u.empower_factor(), 0.5, "the held attack factor");
    assert_eq!(u.rating_set(Rating::Guard), None);
    assert_eq!(armor_rating(&w.units[ALLY]), ARMOR, "another unit is untouched");
}

// ---- (d) est_heal applies heal_taken --------------------------------------------------------------------------------

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

/// `c`'s planned `step`: the plan and the estimate of its entry for `skill` (None: the item entry).
fn planned(w: &World, c: UnitId, step: Step, skill: Option<usize>) -> (Plan, i32) {
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, c, step, true), "", "the step plans");
    let entries = plan.entries_for(w, c);
    let e = entries.iter().find(|e| skill.map_or(e.item.is_some(), |s| e.skill == s && e.item.is_none())).expect("the entry");
    let est = e.est_heal;
    (plan, est)
}

/// On a target holding a `heal_taken` band the estimate is the heal after it (as `World::heal` applies it), capped by
/// the HP missing; the round value counts that; a target without the band keeps the whole heal; an item's too.
#[test]
fn the_heal_estimate_applies_heal_taken() {
    let mut w = session(&["tension"]).w;
    assert_eq!(w.units[HERO].band_heal_taken, HEAL_TAKEN, "the band holds");
    w.units[HERO].hp = HP - 200;
    w.units[ALLY].hp = HP - 200;
    let mend = sid(&w, "t_mend");
    let amount = command::heal_amount_on(&w, ALLY, HERO, mend);
    assert!(amount > 0);
    let (plan, est) = planned(&w, ALLY, PlanStep::ally(mend, HERO), Some(mend));
    assert_eq!(est, heal_taken(amount, HEAL_TAKEN), "on the band holder");
    assert!(est < amount, "the band lowers it");
    assert_eq!(ai::round_value(&w, &plan, ALLY), i64::from(est));
    let (_, est) = planned(&w, HERO, PlanStep::ally(mend, ALLY), Some(mend));
    assert_eq!(est, command::heal_amount_on(&w, HERO, ALLY, mend), "no band: the whole heal");
    // the planned heal matches the heal the command gives
    let before = w.units[HERO].hp;
    w.heal(HERO, amount);
    assert_eq!(w.units[HERO].hp - before, heal_taken(amount, HEAL_TAKEN));
    w.units[HERO].hp = HP - 200;
    // an item: the potion's heal after the band
    let potion = ItemDef::from_obj(json!({"id": "t_potion", "name": "Potion", "item_type": "CONSUMABLE", "consumable_type": "HEALTH_POTION", "heal": MEND, "use_blocks": 1, "use_reach": 1}).as_object().unwrap());
    w.units[ALLY].inventory.pouch.push(potion.clone());
    w.add_item_skills();
    let (_, est) = planned(&w, ALLY, PlanStep::use_item("t_potion", HERO), None);
    assert_eq!(est, heal_taken(MEND, HEAL_TAKEN));
    w.units[HERO].inventory.pouch.push(potion);
    w.add_item_skills();
    let (_, est) = planned(&w, HERO, PlanStep::use_item("t_potion", ALLY), None);
    assert_eq!(est, MEND, "no band: the whole potion");
}
