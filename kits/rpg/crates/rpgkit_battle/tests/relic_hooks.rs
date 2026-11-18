//! `PR7-8` (`RELIC-SIG1`) on a scratch catalog built here (no shipped data): the item hooks the three relic signatures
//! use, each against the rule it reads:
//!
//! - the `when` key `holds_meter` on a worn item's effect row: the effect holds only while a passive gives the wearer
//!   that unit meter (also when the item was worn before the passive came, as a member's build does);
//! - the `skill_potency` group `attack`: every ATTACK skill's power gains the value (1 + sum);
//! - the effect `meter_floor` with `when.from_act`: a held gauge starts at and never decays below the floor once the
//!   unit's campaign act reaches the key (`Unit::set_campaign_act`); before it, or without the meter, nothing;
//! - a worn item's trigger with `when.from_act`: fires only from that act; a trigger without it fires in every act;
//! - the trigger effect `shockwave {tiles, pct_of_hit}`: each foe on the `tiles` tiles past the struck unit, along the
//!   striker's line, takes `pct_of_hit` percent of the hit (`trigger::part`); a friend on the line and a foe beyond it
//!   take nothing.

use rpgkit_battle::command::Command;
use rpgkit_battle::grid::V2;
use rpgkit_battle::meter;
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::session::{EncounterDef, Roster, Session};
use rpgkit_battle::trigger;
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::{Catalog, PassiveDef, UnitDef};
use rpgkit_rules::gear::{Equipment, Slot};
use rpgkit_rules::ruleset::{Ruleset, TIMELINE, TRIGGERS, UNIT_METERS};
use rpgkit_rules::Overrides;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
/// The test rows' numbers (the expectations derive from them).
const POTENCY: f64 = 0.3;
const FLOOR: i32 = 20;
const FLOOR_ACT: i32 = 2;
const VENT: i32 = 10;
const STRAIN_START: i32 = 50;
const DECAY: i32 = 30;
const WAVE_TILES: i32 = 2;
const WAVE_PCT: i32 = 50;

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const FOE2: UnitId = 3;
const FOE3: UnitId = 4;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 1, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "t_jab", "name": "Jab", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.5, "target_mode": "ENEMY"}),
        json!({"id": "t_mend", "name": "Mend", "kind": "SUPPORT", "level": 1, "blocks": 1, "heal_base": 5, "target_mode": "ALLY"}),
    ];
    let meters = vec![json!({"id": "strain", "name": "Strain", "short": "St", "min": 0, "max": 100, "start": 0, "show": "bar", "decay": [{"delta": -DECAY}]})];
    let unit = |id: &str, side: &str, speed: i32| {
        json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 5000, "power": 100, "armor": 0, "speed": speed, "attack_range": 1, "skill_cap": 5,
            "deck": {"active": ["t_mend"]}})
    };
    let item = |id: &str, slot: &str, effects: Value| json!({"id": id, "name": id, "item_type": "ACCESSORY", "slot": slot, "effects": effects});
    Catalog::from_value(&json!({
        "skills": skills,
        "unit_meters": meters,
        "passives": [{"id": "strained", "name": "Strained", "slot": "", "effects": [{"type": "meter", "meter": "strain", "start": STRAIN_START}]}],
        "items": [
            // the vertebra's pattern: an attack bonus while strained, a floor from act 2
            item("t_spine", "ACCESSORY_1", json!([
                {"type": "skill_potency", "group": "attack", "value": POTENCY, "when": {"holds_meter": "strain"}},
                {"type": "meter_floor", "meter": "strain", "floor": FLOOR, "when": {"from_act": FLOOR_ACT}}])),
            // the scalpel's pattern: bleed on every hit, a vent from act 2
            item("t_saw", "ACCESSORY_1", json!([
                {"type": "trigger", "on": "hit_dealt", "limit": "none", "effects": [{"type": "inflict", "status": "bleed", "potency": 100, "rounds": 2, "to": "other"}]},
                {"type": "trigger", "on": "hit_dealt", "when": {"from_act": FLOOR_ACT}, "limit": "none", "effects": [{"type": "meter", "meter": "strain", "delta": -VENT, "to": "self"}]}])),
            // the hammer's pattern: a melee hit carried 2 tiles on
            item("t_maul", "ACCESSORY_1", json!([
                {"type": "trigger", "on": "hit_dealt", "when": {"melee": true}, "limit": "none", "effects": [{"type": "shockwave", "tiles": WAVE_TILES, "pct_of_hit": WAVE_PCT}]}])),
        ],
        "units": [unit("p_hero", "party", 14), unit("p_ally", "party", 12), unit("p_foe", "enemy", 10), unit("p_foe2", "enemy", 9), unit("p_foe3", "enemy", 8)],
    }))
}

/// The refs' passives (ref -> passive ids), as a world character's overrides.
struct Kit(Vec<(&'static str, Vec<&'static str>)>);

impl Roster for Kit {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        cat.units.get(r).cloned()
    }

    fn overrides(&self, _cat: &Catalog, r: &str) -> Overrides {
        let Some((_, passives)) = self.0.iter().find(|(who, _)| *who == r) else { return Overrides::default() };
        Overrides { passives: passives.iter().map(|p| p.to_string()).collect(), ..Overrides::default() }
    }
}

fn rules(list: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": list}).as_object().unwrap())
}

/// Hero (4,4) faces the foe (5,4); past it on the hero's row the second foe (6,4), the ally (7,4) and the third foe
/// (8,4).
fn session(ruleset: Ruleset, kit: Kit, worn: &[(UnitId, &str)]) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe", "p_foe2", "p_foe3"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &kit);
    for (c, cell) in [(HERO, V2::new(4, 4)), (FOE, V2::new(5, 4)), (FOE2, V2::new(6, 4)), (ALLY, V2::new(7, 4)), (FOE3, V2::new(8, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    // worn as a member's gear (a fixed-stat scratch unit re-reads no named item), its gauges restarted
    for (c, item) in worn {
        wear(&mut s.w, *c, item);
        meter::restart_unit(&mut s.w, *c);
    }
    s
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn jab(w: &World, c: UnitId, t: UnitId) -> Command {
    let mut plan = Plan::new();
    let id = sid(w, "t_jab");
    assert_eq!(plan.add_step(w, c, PlanStep::engage(id, t), true), "");
    plan.entries_for(w, c).into_iter().find(|e| e.skill == id).expect("the jab's entry").build_command(w, c)
}

/// One jab of the hero at the foe in block 1.
fn hero_jabs(w: &mut World) -> Resolver {
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(HERO, jab(w, HERO, FOE), 1));
    for i in res.prepare_block(w, 1) {
        res.execute_single(w, i);
    }
    res
}

fn wear(w: &mut World, c: UnitId, item: &str) {
    let cat = w.cat.clone();
    let def = cat.items[item].clone();
    w.units[c].inventory.equip(Equipment::new(&def), Slot::Accessory1);
    w.units[c].sync_from_stats_and_gear(&cat);
}

fn jab_power(w: &World, c: UnitId) -> f64 {
    w.units[c].power_for(w.cat.skill("t_jab").expect("t_jab"))
}

/// `holds_meter` and the group `attack`: a strained wearer's jab gains `POTENCY` over the same unit bare; an unstrained
/// wearer gains nothing. The item worn before the passive came (a member's build order) still counts.
#[test]
fn the_attack_bonus_holds_only_while_the_meter_is_held() {
    let s = session(rules(&[UNIT_METERS]), Kit(vec![("p_hero", vec!["strained"])]), &[(HERO, "t_spine"), (ALLY, "t_spine")]);
    let bare = session(rules(&[UNIT_METERS]), Kit(vec![("p_hero", vec!["strained"])]), &[]);
    assert!(s.w.units[HERO].holds_meter("strain") && !s.w.units[ALLY].holds_meter("strain"));
    let base = jab_power(&bare.w, HERO);
    assert!((jab_power(&s.w, HERO) - base * (1.0 + POTENCY)).abs() < 1e-9, "strained: {} vs {}", jab_power(&s.w, HERO), base);
    assert!((jab_power(&s.w, ALLY) - base).abs() < 1e-9, "unstrained: no bonus");
    // a SUPPORT skill is not in the group
    let mend = s.w.cat.skill("t_mend").expect("t_mend");
    assert!((s.w.units[HERO].potency_for_skill(mend) - bare.w.units[HERO].potency_for_skill(mend)).abs() < 1e-9);
    // worn first, the meter given after (a member's build applies its passives after its gear): the bonus opens
    let mut w = s.w.clone();
    let cat = w.cat.clone();
    w.units[ALLY].apply_effects(&cat, &[json!({"type": "meter", "meter": "strain"})], "passive:late", false);
    assert!((jab_power(&w, ALLY) - base * (1.0 + POTENCY)).abs() < 1e-9, "opened by the later meter");
}

/// `meter_floor` with `from_act`: in act `FLOOR_ACT` the strained wearer's gauge starts at the larger of its start and
/// the floor and decays no lower than it; in act 1 (and outside a campaign) the gauge keeps the row's floor (0). A wearer
/// who holds no Strain gets no gauge from the item.
#[test]
fn the_meter_floor_counts_from_its_act_and_gives_no_meter() {
    for (act, floor) in [(0, 0), (1, 0), (FLOOR_ACT, FLOOR), (FLOOR_ACT + 1, FLOOR)] {
        let s = session(rules(&[UNIT_METERS]), Kit(vec![("p_hero", vec!["strained"])]), &[(HERO, "t_spine"), (ALLY, "t_spine")]);
        let mut w = s.w.clone();
        let cat = w.cat.clone();
        w.units[HERO].set_campaign_act(&cat, act);
        meter::restart_unit(&mut w, HERO);
        let g = w.meters.gauges.iter().find(|g| g.unit == HERO && g.meter == "strain").expect("the hero's gauge");
        assert_eq!((g.floor, g.value), (floor, STRAIN_START.max(floor)), "act {act}");
        assert_eq!(w.units[HERO].meter_floor("strain"), (floor > 0).then_some(floor), "act {act}");
        assert!(meter::value(&w, ALLY, "strain").is_none(), "no meter from the item");
        let mut res = Resolver::new(BLOCKS, 1);
        for _ in 0..3 {
            res.apply_round_end_effects(&mut w);
        }
        assert_eq!(meter::value(&w, HERO, "strain"), Some(floor), "act {act}: decays to the floor");
    }
}

/// A worn item's triggers: the bleed (no `from_act`) is rolled on every hit in every act; the vent (`from_act` 2) lowers the
/// wielder's Strain by `VENT` only from act 2. Without the meter the vent does nothing in any act.
#[test]
fn the_act_gated_trigger_fires_only_from_its_act() {
    for act in [1, FLOOR_ACT] {
        for strained in [true, false] {
            let passives = if strained { vec!["strained"] } else { vec![] };
            let s = session(rules(&[UNIT_METERS, TRIGGERS]), Kit(vec![("p_hero", passives)]), &[(HERO, "t_saw")]);
            let mut w = s.w.clone();
            let cat = w.cat.clone();
            w.units[HERO].set_campaign_act(&cat, act);
            let res = hero_jabs(&mut w);
            // the bleed is rolled (resistable): fired either way
            let bleed = |l: &String| l.starts_with("[Trigger] p_hero's t_saw: ") && l.ends_with(" Bleeding on p_foe");
            assert!(res.log.iter().any(bleed), "act {act}: {:?}", res.log);
            let want = match (strained, act >= FLOOR_ACT) {
                (true, true) => Some(STRAIN_START - VENT),
                (true, false) => Some(STRAIN_START),
                (false, _) => None,
            };
            assert_eq!(meter::value(&w, HERO, "strain"), want, "act {act}, strained {strained}: {:?}", res.log);
        }
    }
}

/// `shockwave`: the jab's hit on the foe carries `WAVE_TILES` tiles on: the second foe right behind takes
/// `WAVE_PCT`% of the hit; the ally 2 tiles past takes nothing (a friend), nor does the third foe 3 tiles past (beyond
/// the reach). The hammer worn by the ally (out of line) adds nothing to the hero's hit.
#[test]
fn a_shockwave_carries_the_hit_past_the_struck_foe() {
    let s = session(rules(&[TRIGGERS]), Kit(vec![]), &[(HERO, "t_maul")]);
    let mut w = s.w.clone();
    let hp: Vec<i32> = w.units.iter().map(|u| u.hp).collect();
    let res = hero_jabs(&mut w);
    let hit = hp[FOE] - w.units[FOE].hp;
    assert!(hit > 0, "the jab lands: {:?}", res.log);
    let wave = trigger::part(hit, WAVE_PCT);
    assert_eq!(hp[FOE2] - w.units[FOE2].hp, wave, "{:?}", res.log);
    assert_eq!((hp[ALLY] - w.units[ALLY].hp, hp[FOE3] - w.units[FOE3].hp), (0, 0), "{:?}", res.log);
    assert!(res.log.iter().any(|l| l == &format!("[Trigger] p_hero's t_maul: shockwave: {wave} damage to p_foe2")), "{:?}", res.log);
    // the same hit without the item carries nothing
    let plain = session(rules(&[TRIGGERS]), Kit(vec![]), &[]);
    let mut w = plain.w.clone();
    let before = w.units[FOE2].hp;
    hero_jabs(&mut w);
    assert_eq!(w.units[FOE2].hp, before);
    // worn by the ally (not the striker), it carries nothing on the hero's hit
    let other = session(rules(&[TRIGGERS]), Kit(vec![]), &[(ALLY, "t_maul")]);
    let mut w = other.w.clone();
    let before = w.units[FOE2].hp;
    hero_jabs(&mut w);
    assert_eq!(w.units[FOE2].hp, before);
}

/// The scratch passive table is what the catalog loads (a guard against a renamed key silently giving no meter).
#[test]
fn the_scratch_rows_load() {
    let cat = catalog();
    let p: &PassiveDef = &cat.passives["strained"];
    assert_eq!(p.effects.len(), 1);
    assert_eq!(cat.items["t_spine"].effects.len(), 2);
}

