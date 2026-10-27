//! Derived-stat rules (DERIVE1-R, docs/specs/DERIVE1.md 4.2-4.3): the value is the row's formula over the unit's
//! effective base stats, `level`, constants and other rows, then `clamp(roundi((raw + flat) × (1 + percent)), min,
//! max)` with the modifiers of gear, tiers, passives and the row's status modifiers of active statuses; a row outside
//! the ruleset reads as absent; the resist chance is `clamp(potency − value, 0, 100)` and a battle check draws exactly
//! once. Every expectation is worked out here from those rules over a scratch catalog (no shipped data).

use aegis_core::json::roundi;
use aegis_core::rng::GodotRng;
use rpgkit_data::{Catalog, STATS};
use rpgkit_rules::derived::{self, Cap, InputKind, ResistCheck};
use rpgkit_rules::gear::{Equipment, Slot};
use rpgkit_rules::{build_unit, Unit};
use serde_json::{json, Value};

const CLASSIC: &str = "classic";
const TIMELINE: &str = "timeline";

/// The scratch catalog: one class and unit, derived rows of every input kind, passives, a charm with a tier bonus.
fn catalog() -> Catalog {
    Catalog::from_value(&json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 14}, "growth": {"KNOW": 1, "CONS": 1}}],
        "units": [{"id": "probe", "name": "Probe", "class_id": "drill", "level": 3, "side": "player"}],
        "derived_stats": [
            {"id": "nerve", "name": "Nerve", "formula": "base + KNOW * per_know + CONS * per_cons", "constants": {"base": 0, "per_know": 2, "per_cons": 1}, "min": 0, "max": 100,
             "status_modifiers": [{"status": "weaken", "flat": -5, "percent": 0.0}, {"status": "empower", "flat": 0, "percent": 0.1}]},
            {"id": "grit", "name": "Grit", "formula": "nerve / 3 + level", "min": 0, "max": 100},
            {"id": "dread", "name": "Dread", "formula": "100 - KNOW", "min": 0, "max": 100, "rulesets": ["timeline"]},
            {"id": "tiny", "name": "Tiny", "formula": "STR * 10 - 200", "min": 0, "max": 5}
        ],
        "passives": [
            {"id": "steady", "name": "Steady", "effects": [{"type": "stat", "stat": "nerve", "value": 3}]},
            {"id": "shaken", "name": "Shaken", "effects": [{"type": "stat", "stat": "nerve", "value": 0, "percent": -0.1}]},
            {"id": "sworded", "name": "Sworded", "effects": [{"type": "stat", "stat": "nerve", "value": 7, "context": {"weapon": "sword"}}]},
            {"id": "keen", "name": "Keen", "effects": [{"type": "stat", "stat": "KNOW", "value": 1, "percent": 0.5}]}
        ],
        "items": [
            {"id": "charm", "name": "Charm", "item_type": "ACCESSORY", "slot": "ACCESSORY_1", "stat_bonuses": {"nerve": 4},
             "threshold_tiers": [{"when": {"type": "unit_level_at_least", "value": 1}, "bonus": {"stat_bonuses": {"nerve": 2}}}]}
        ]
    }))
}

fn probe(cat: &Catalog) -> Unit {
    build_unit(cat, &cat.units["probe"], 0)
}

fn stat(u: &Unit, s: &str) -> f64 {
    u.stats.stat(s, u.weapon_context().as_deref()) as f64
}

/// Nerve's formula written out in Rust over the unit's effective stats.
fn nerve_raw(u: &Unit) -> f64 {
    stat(u, "KNOW") * 2.0 + stat(u, "CONS")
}

fn rule(raw: f64, flat: i32, pct: f64, min: i32, max: i32) -> i32 {
    (roundi((raw + flat as f64) * (1.0 + pct)) as i32).clamp(min, max)
}

fn nerve(cat: &Catalog, u: &Unit) -> i32 {
    derived::value(cat, CLASSIC, u, "nerve").expect("nerve is in every ruleset")
}

fn passive(cat: &Catalog, u: &mut Unit, id: &str) {
    u.apply_passive(cat, &cat.passives[id]);
}

#[test]
fn a_unit_without_modifiers_reads_the_formula_over_its_stats() {
    let cat = catalog();
    let u = probe(&cat);
    assert_eq!(nerve(&cat, &u), rule(nerve_raw(&u), 0, 0.0, 0, 100));
    let b = derived::breakdown(&cat, CLASSIC, &u, "nerve").unwrap();
    assert_eq!(b.formula, "base + KNOW * per_know + CONS * per_cons");
    let inputs: Vec<(&str, InputKind, f64)> = b.inputs.iter().map(|i| (i.name.as_str(), i.kind, i.value)).collect();
    assert_eq!(
        inputs,
        [("base", InputKind::Constant, 0.0), ("KNOW", InputKind::Stat, stat(&u, "KNOW")), ("per_know", InputKind::Constant, 2.0), ("CONS", InputKind::Stat, stat(&u, "CONS")), ("per_cons", InputKind::Constant, 1.0)]
    );
    assert_eq!((b.raw, b.parts.len(), b.flat, b.percent, b.cap, b.value), (nerve_raw(&u), 0, 0, 0.0, None, nerve(&cat, &u)));
    assert_eq!((b.min, b.max, b.uncapped), (0, 100, roundi(nerve_raw(&u)) as i32));
}

#[test]
fn a_formula_reads_level_and_the_final_value_of_another_row() {
    let cat = catalog();
    let mut u = probe(&cat);
    passive(&cat, &mut u, "steady");
    let n = nerve(&cat, &u);
    let level = u.stats.level as f64;
    assert_eq!(derived::value(&cat, CLASSIC, &u, "grit"), Some(rule(n as f64 / 3.0 + level, 0, 0.0, 0, 100)));
    let b = derived::breakdown(&cat, CLASSIC, &u, "grit").unwrap();
    let inputs: Vec<(&str, InputKind, f64)> = b.inputs.iter().map(|i| (i.name.as_str(), i.kind, i.value)).collect();
    assert_eq!(inputs, [("nerve", InputKind::Derived, n as f64), ("level", InputKind::Level, level)]);
}

#[test]
fn a_passive_stat_effect_adds_its_flat_value() {
    let cat = catalog();
    let mut u = probe(&cat);
    let before = nerve(&cat, &u);
    let stats_before: Vec<i32> = STATS.iter().map(|s| u.stats.stat(s, None)).collect();
    passive(&cat, &mut u, "steady");
    assert_eq!(nerve(&cat, &u), (before + 3).min(100), "value rises by k unless the cap is hit");
    assert_eq!(STATS.iter().map(|s| u.stats.stat(s, None)).collect::<Vec<_>>(), stats_before, "no base stat moves");
    let b = derived::breakdown(&cat, CLASSIC, &u, "nerve").unwrap();
    assert_eq!(b.parts.iter().map(|p| (p.source.as_str(), p.flat, p.percent)).collect::<Vec<_>>(), [("passive:steady", 3, 0.0)]);
}

#[test]
fn a_passive_percent_counts_on_a_derived_id_only() {
    let cat = catalog();
    let mut u = probe(&cat);
    passive(&cat, &mut u, "steady");
    passive(&cat, &mut u, "shaken");
    assert_eq!(nerve(&cat, &u), rule(nerve_raw(&u), 3, -0.1, 0, 100), "flat first, then the percent");
    let know = u.stats.base("KNOW");
    passive(&cat, &mut u, "keen");
    assert_eq!(u.stats.stat("KNOW", None), know + 1, "a base stat keeps its flat value and ignores percent");
}

#[test]
fn weapon_conditional_modifiers_follow_the_weapon_context() {
    let cat = catalog();
    let mut u = probe(&cat);
    assert_eq!(u.weapon_context(), None);
    let before = nerve(&cat, &u);
    passive(&cat, &mut u, "sworded");
    assert_eq!(nerve(&cat, &u), before, "a sword-only modifier does not count without a sword");
}

#[test]
fn gear_and_tier_bonuses_count_and_split_by_item() {
    let cat = catalog();
    let mut u = probe(&cat);
    let before = nerve(&cat, &u);
    let hp = u.max_hp;
    u.inventory.equip(Equipment { item: cat.items["charm"].clone() }, Slot::Accessory1);
    u.sync_from_stats_and_gear(&cat);
    assert_eq!(nerve(&cat, &u), (before + 4 + 2).min(100));
    assert_eq!(u.max_hp, hp, "a derived bonus touches no built-in number");
    let b = derived::breakdown(&cat, CLASSIC, &u, "nerve").unwrap();
    assert_eq!(b.parts.iter().map(|p| (p.source.as_str(), p.flat)).collect::<Vec<_>>(), [("gear:charm", 4), ("gear_tier:charm", 2)]);
    assert_eq!((b.flat, b.value), (6, nerve(&cat, &u)));
}

#[test]
fn status_modifiers_apply_only_while_the_status_is_active() {
    let cat = catalog();
    let mut u = probe(&cat);
    let raw = nerve_raw(&u);
    u.st.pending.insert("weaken".into());
    assert_eq!(nerve(&cat, &u), rule(raw, 0, 0.0, 0, 100), "a pending status does not count");
    u.st.active.insert("weaken".into());
    assert_eq!(nerve(&cat, &u), rule(raw, -5, 0.0, 0, 100));
    u.st.active.insert("empower".into());
    assert_eq!(nerve(&cat, &u), rule(raw, -5, 0.1, 0, 100));
    let b = derived::breakdown(&cat, CLASSIC, &u, "nerve").unwrap();
    assert_eq!(b.parts.iter().map(|p| (p.source.as_str(), p.flat, p.percent)).collect::<Vec<_>>(), [("status:weaken", -5, 0.0), ("status:empower", 0, 0.1)]);
    u.st.active.shift_remove("weaken");
    u.st.active.shift_remove("empower");
    assert_eq!(nerve(&cat, &u), rule(raw, 0, 0.0, 0, 100));
}

#[test]
fn caps_hold_over_random_stats() {
    let cat = catalog();
    let mut u = probe(&cat);
    let mut rng = GodotRng::new(4711);
    for _ in 0..300 {
        for s in STATS {
            u.stats.core.insert(s.to_string(), rng.randi_range(1, 40));
        }
        for d in cat.derived_stats.values() {
            let v = derived::value(&cat, TIMELINE, &u, &d.id).unwrap();
            assert!(d.min <= v && v <= d.max, "{} = {v} outside {}..{}", d.id, d.min, d.max);
        }
        assert_eq!(nerve(&cat, &u), rule(nerve_raw(&u), 0, 0.0, 0, 100));
        let tiny = derived::breakdown(&cat, CLASSIC, &u, "tiny").unwrap();
        let uncapped = (stat(&u, "STR") * 10.0 - 200.0) as i32;
        let cap = if uncapped < 0 { Some(Cap::Min) } else if uncapped > 5 { Some(Cap::Max) } else { None };
        assert_eq!((tiny.uncapped, tiny.cap, tiny.value), (uncapped, cap, uncapped.clamp(0, 5)));
    }
}

#[test]
fn a_row_outside_the_ruleset_reads_as_absent() {
    let cat = catalog();
    let u = probe(&cat);
    assert_eq!(derived::value(&cat, CLASSIC, &u, "dread"), None);
    assert!(derived::breakdown(&cat, CLASSIC, &u, "dread").is_none());
    assert_eq!(derived::value(&cat, TIMELINE, &u, "dread"), Some(rule(100.0 - stat(&u, "KNOW"), 0, 0.0, 0, 100)));
    assert_eq!(derived::value(&cat, TIMELINE, &u, "unknown_row"), None);
}

#[test]
fn reading_a_value_stores_nothing() {
    let cat = catalog();
    let u = probe(&cat);
    let before = format!("{u:?}");
    for d in cat.derived_stats.values() {
        let _ = derived::value(&cat, TIMELINE, &u, &d.id);
        let _ = derived::breakdown(&cat, TIMELINE, &u, &d.id);
    }
    assert_eq!(format!("{u:?}"), before);
}

#[test]
fn an_unchecked_cycle_reads_without_hanging() {
    let cat = Catalog::from_value(&json!({
        "classes": [{"id": "drill", "name": "Drill"}],
        "units": [{"id": "probe", "name": "Probe", "class_id": "drill"}],
        "derived_stats": [{"id": "aa", "name": "A", "formula": "bb + 1"}, {"id": "bb", "name": "B", "formula": "aa + 1"}]
    }));
    let u = probe(&cat);
    assert!(derived::value(&cat, CLASSIC, &u, "aa").is_some());
}

#[test]
fn resist_chance_is_potency_minus_value_clamped() {
    let cat = catalog();
    let u = probe(&cat);
    let v = nerve(&cat, &u);
    for potency in (-20..=220).step_by(7) {
        assert_eq!(derived::resist_chance(&cat, CLASSIC, &u, "nerve", potency), (potency - v).clamp(0, 100));
        assert_eq!(derived::resist_chance(&cat, CLASSIC, &u, "dread", potency), potency.clamp(0, 100), "absent row: the potency itself");
    }
}

#[test]
fn a_battle_check_draws_once_and_no_check_draws_nothing() {
    let cat = catalog();
    let mut u = probe(&cat);
    for (potency, seed) in [(1, 7_u64), (60, 8), (200, 9)] {
        let check = ResistCheck { resist: "nerve".into(), potency };
        let mut with = GodotRng::new(seed);
        let mut alone = GodotRng::new(seed);
        let landed = derived::lands(&cat, CLASSIC, &u, &check, &mut with);
        let expected = alone.randi_range(1, 100) <= derived::resist_chance(&cat, CLASSIC, &u, "nerve", potency);
        assert_eq!(landed, expected);
        assert_eq!(with.draws, alone.draws, "exactly one randi_range(1, 100)");
        assert_eq!((0..8).map(|_| with.randi()).collect::<Vec<_>>(), (0..8).map(|_| alone.randi()).collect::<Vec<_>>(), "the stream after the check is the same");
    }
    u.hp = 0;
    let mut rng = GodotRng::new(3);
    assert!(!derived::lands(&cat, CLASSIC, &u, &ResistCheck { resist: "nerve".into(), potency: 200 }, &mut rng));
    assert_eq!(rng.draws, 0, "a fallen target draws nothing");
}

#[test]
fn the_consumer_shape_is_validated_in_one_place() {
    let ids = vec!["nerve".to_string(), "grit".to_string()];
    let errs = |v: Value| derived::check_errors(&v, &ids);
    assert!(errs(json!({"resist": "nerve", "potency": 40})).is_empty());
    assert!(errs(json!({"resist": "grit", "potency": 200.0, "radius": 3})).is_empty(), "whole floats load from JSON; other keys are the consumer's");
    assert_eq!(errs(json!({"resist": "Nerve", "potency": 40})), ["Resist: 'Nerve' is not a derived stat."], "derived ids are exact lowercase");
    assert_eq!(errs(json!({"potency": 40})), ["Resist: a derived stat id is required."]);
    assert_eq!(errs(json!({"resist": 3, "potency": 40})), ["Resist must be a derived stat id (text)."]);
    for bad in [json!(0), json!(201), json!(1.5), json!("40"), Value::Null] {
        assert_eq!(errs(json!({"resist": "nerve", "potency": bad})), ["Potency must be a whole number 1-200."]);
    }
    assert_eq!(errs(json!({"resist": "nerve"})), ["Potency must be a whole number 1-200."]);
    assert_eq!(errs(json!(["nerve", 40])).len(), 1);
    assert_eq!(ResistCheck::from_value(&json!({"resist": "nerve", "potency": 40.0})), Some(ResistCheck { resist: "nerve".into(), potency: 40 }));
    assert_eq!(ResistCheck::from_value(&json!({"resist": "nerve", "potency": 0})), None);
    assert_eq!(derived::RESIST_STREAM, "resist");
}

/// RULESET1 P1 (tasks 1.7): `gear_mitigation` reads the summed `effective_mitigation` of the worn gear (the Max Poise
/// row's input), and the `Ruleset` wrappers read as the ruleset id does.
#[test]
fn gear_mitigation_is_the_worn_mitigation_and_rulesets_read_by_id() {
    use rpgkit_rules::ruleset::Ruleset;
    let cat = Catalog::from_value(&json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10}}],
        "units": [{"id": "probe", "name": "Probe", "class_id": "drill", "level": 1, "side": "player"}],
        "derived_stats": [{"id": "max_poise", "name": "Max Poise", "formula": "CONS * per_cons + gear_mitigation * armour_factor", "constants": {"per_cons": 2, "armour_factor": 1.0}, "min": 1, "max": 200, "rulesets": ["timeline"]}],
        "items": [
            {"id": "mail", "name": "Mail", "item_type": "ARMOR", "slot": "BODY", "mitigation": 6},
            {"id": "cap", "name": "Cap", "item_type": "ARMOR", "slot": "HEAD", "mitigation": 2}
        ]
    }));
    let mut u = probe(&cat);
    let poise = |u: &Unit| rule(stat(u, "CONS") * 2.0 + u.inventory.total_flat_mitigation(Some(&u.stats)) as f64, 0, 0.0, 1, 200);
    let bare = derived::value(&cat, TIMELINE, &u, "max_poise").unwrap();
    assert_eq!(bare, poise(&u));
    u.inventory.equip(Equipment { item: cat.items["mail"].clone() }, Slot::Body);
    u.inventory.equip(Equipment { item: cat.items["cap"].clone() }, Slot::Head);
    u.sync_from_stats_and_gear(&cat);
    let worn = u.inventory.total_flat_mitigation(Some(&u.stats));
    assert!(worn > 0, "the fixture wears mitigation");
    assert_eq!(derived::value(&cat, TIMELINE, &u, "max_poise").unwrap(), poise(&u));
    let b = derived::breakdown(&cat, TIMELINE, &u, "max_poise").unwrap();
    let gear = b.inputs.iter().find(|i| i.name == "gear_mitigation").unwrap();
    assert_eq!((gear.kind, gear.value), (InputKind::Gear, worn as f64));
    let timeline = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": []}).as_object().unwrap());
    for (rs, id) in [(Ruleset::classic(), CLASSIC), (timeline, TIMELINE)] {
        assert_eq!(derived::value_for(&cat, &rs, &u, "max_poise"), derived::value(&cat, id, &u, "max_poise"), "{id}");
        assert_eq!(derived::breakdown_for(&cat, &rs, &u, "max_poise"), derived::breakdown(&cat, id, &u, "max_poise"), "{id}");
        assert_eq!(derived::row_for(&cat, &rs, "max_poise").is_some(), id == TIMELINE, "{id}: the row lives in timeline only");
        assert_eq!(derived::resist_chance_for(&cat, &rs, &u, "max_poise", 90), derived::resist_chance(&cat, id, &u, "max_poise", 90), "{id}");
    }
}
