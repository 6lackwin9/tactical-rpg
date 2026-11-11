//! U4.2-S1 proof (docs/sessions/U4_PASSIVES.md, FB-25 to FB-27, FB-30): the typed grant view (`grants::read`, applied
//! by `Unit::apply_grants`) leaves every unit exactly as the raw-JSON reader it replaced left it. The old reader is
//! kept below, verbatim, as the oracle (`old_apply_effects`, `old_apply_band`, `old_with_drills`); the double load runs
//! every passive row of the shipped catalog (`data/catalog/passives.json`) and of the frozen one (the timeline golden's
//! `timeline_kit.json`) on every unit of its catalog: as a passive, as worn gear (the six item types and the gear
//! filter), as a meter band (with `crit`), one at a time and all in order (trigger keys, the foresight source, meter
//! order). Then each combine rule from hand-built rows, and two mutations the double load must catch (a resist combine
//! rule, trigger order). Expectations come from the oracle or from the rule, never from shipped numbers.
//!
//! The rows are game data, so the test lives in the game's façade crate (the kit names no game).

use aegis_core::value::{self, to_int, to_str, Obj};
use emberwake_data::{Catalog, ClassDef, PassiveDef, TriggerRow};
use emberwake_rules::grants::{self, Combine, Grant, Immunity, Modifier, SkillGrant, SkillSet, Target};
use emberwake_rules::stats::{Proficiency, StatModifier};
use emberwake_rules::unit::{BandApplied, ForcedTraits, HeldMeter, HeldTrigger, HitSource, StaggerTaken, FORESIGHT_RANGE_MAX, STAGGER_TAKEN_MULT_MAX};
use emberwake_rules::unit::gear_effect_source;
use emberwake_rules::{build_unit, Unit};
use indexmap::IndexMap;
use serde_json::{json, Map, Value};
use std::path::PathBuf;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The shipped catalog: each `data/catalog/<kind>.json`'s entries under its kind.
fn shipped() -> Catalog {
    let dir = crate_dir().join("../../data/catalog");
    let mut root = Map::new();
    for entry in std::fs::read_dir(&dir).unwrap_or_else(|_| panic!("missing {}", dir.display())) {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|x| x == "json") {
            let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            root.insert(path.file_stem().unwrap().to_string_lossy().to_string(), v.get("entries").cloned().unwrap_or(json!([])));
        }
    }
    Catalog::from_value(&Value::Object(root))
}

/// The frozen catalog of the timeline golden.
fn frozen() -> Catalog {
    let path = crate_dir().join("../emberwake_battle/tests/fixtures/timeline_kit.json");
    Catalog::from_value(&serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("missing {}", path.display()))).unwrap())
}

fn catalogs() -> Vec<(&'static str, Catalog)> {
    let out = vec![("shipped", shipped()), ("frozen", frozen())];
    for (name, cat) in &out {
        assert!(!cat.passives.is_empty() && !cat.units.is_empty(), "{name}: passives and units found");
        println!("{name}: {} passive rows, {} units", cat.passives.len(), cat.units.len());
    }
    out
}

fn units(cat: &Catalog) -> Vec<Unit> {
    cat.units.values().map(|e| build_unit(cat, e, 0)).collect()
}

fn same(a: &Unit, b: &Unit) -> bool {
    format!("{a:?}") == format!("{b:?}")
}

fn assert_same(a: &Unit, b: &Unit, what: &str) {
    assert!(same(a, b), "{what}: the typed view and the raw reader differ\nraw:   {a:?}\ntyped: {b:?}");
}

// ---- The oracle: the raw-JSON reader as it was before U4.2-S1 (unit.rs, stats.rs), kept verbatim ------------------
// U3.2-S4 (FB-35) changed its gear branch on purpose, marked below: a worn item's `stat` goes under GEAR_EFFECT_SOURCE
// (AUDIT-05: `gear_effect_source`, naming its item),
// its `skill` to `gear_skills` (both rebuilt on every sync), and gear applies no `proficiency`. U4.2-S4 (FB-30) adds
// the source records of the hit-time targets (`old_record`): skill_potency, stagger_taken and forced_move tiles.
// PK2-S1 (`docs/sessions/PK2_GRAFTS.md` 2) adds the two member grants of the passive grammar, passives only (gear and
// bands drop them): `forbid_hands` (the largest `hands`) and `pierce_melee`; PR5-19 (PK2-S6) ships them in `calcify_4`.

/// U4.2-S4: what `apply_modifier` records for a hit-time target (`Unit::hit_sources` / `gear_hit_sources`).
fn old_record(u: &mut Unit, id: &str, gear: bool, target: Target, value: f64) {
    let held = HitSource { source: if gear { id.to_string() } else { format!("passive:{id}") }, target, value };
    if gear {
        u.gear_hit_sources.push(held);
    } else {
        u.hit_sources.push(held);
    }
}

fn old_forced_add(f: &mut ForcedTraits, e: &Obj) {
    match value::string(e, "immune", "").as_str() {
        "always" => f.immune_always = true,
        "guarding" => f.immune_guarding = true,
        _ => {}
    }
    f.bonus_tiles += value::int(e, "bonus_tiles", 0).clamp(0, 2);
}

fn old_apply_effects(u: &mut Unit, cat: &Catalog, effects: &[Value], source: &str, gear: bool) {
    let id = source.trim_start_matches("passive:").to_string();
    for raw in effects {
        let Some(e) = raw.as_object() else { continue };
        match value::string(e, "type", "").as_str() {
            "stat" => {
                let mut tag = "always".to_string();
                if let Some(ctx) = value::obj(e, "context") {
                    for (k, v) in ctx {
                        tag = format!("{}:{}", k, to_str(v));
                    }
                }
                let stat = value::string(e, "stat", "");
                // AUDIT-05 (on purpose): the gear source names its item (`gear_effect:<source>`)
                let source = if gear { gear_effect_source(&id) } else { format!("passive:{id}") };
                let mut m = StatModifier::permanent(&stat, value::int(e, "value", 0), &source, &tag);
                if cat.derived_stats.contains_key(&stat) {
                    m.percent = value::float(e, "percent", 0.0);
                }
                u.stats.modifiers.push(m);
            }
            "proficiency" if !gear => u.stats.proficiency.add_bonus(&value::string(e, "gear", ""), value::int(e, "levels", 0)),
            "resist" => {
                let status = value::string(e, "status", "");
                let target = if gear { &mut u.gear_status_resist } else { &mut u.status_resist };
                let cur = target.get(&status).copied().unwrap_or(0);
                target.insert(status, (cur + value::int(e, "value", 0)).min(100));
            }
            "ap_regen" => u.ap_regen_bonus += value::int(e, "value", 0),
            "skill" if gear => {
                let skill = value::string(e, "skill", "");
                if !u.gear_skills.contains(&skill) {
                    u.gear_skills.push(skill);
                }
            }
            "skill" => u.learn_skill(cat, &value::string(e, "skill", "")),
            "skill_potency" => {
                let v = value::float(e, "value", 0.0);
                let group = value::string(e, "group", "");
                let set = if group.is_empty() { SkillSet::Family(value::string(e, "family", "")) } else { SkillSet::Group(group.clone()) };
                old_record(u, &id, gear, Target::SkillPotency(set), v);
                if !group.is_empty() {
                    let g = if gear { &mut u.gear_potency_group } else { &mut u.potency_group };
                    *g.entry(group).or_insert(0.0) += v;
                } else {
                    let fam = value::string(e, "family", "");
                    let f = if gear { &mut u.gear_potency_family } else { &mut u.potency_bonus };
                    *f.entry(fam).or_insert(0.0) += v;
                }
            }
            "status_potency" => {
                if gear {
                    u.gear_status_potency += value::int(e, "value", 0)
                } else {
                    u.passive_status_potency += value::int(e, "value", 0)
                }
            }
            "spell_reach" => {
                if gear {
                    u.gear_spell_reach += value::int(e, "value", 0)
                } else {
                    u.passive_spell_reach += value::int(e, "value", 0)
                }
            }
            "unlock" if !gear => {
                let levels = value::int(e, "levels", 0).clamp(0, 2);
                let group = value::string(e, "group", "");
                let (map, key) = if group.is_empty() { (&mut u.unlock_bonus_family, value::string(e, "family", "")) } else { (&mut u.unlock_bonus_group, group) };
                let cur = map.get(&key).copied().unwrap_or(0);
                map.insert(key, cur.max(levels));
                u.refresh_path_skills(cat);
            }
            "foresight" if !gear => {
                if u.foresight_source.is_empty() {
                    u.foresight_source = id.clone();
                }
                u.foresight_range = u.foresight_range.max(value::int(e, "range", 0).clamp(0, FORESIGHT_RANGE_MAX));
            }
            "stagger_taken" if !gear => {
                let mult = value::float(e, "mult", 1.0).clamp(0.0, STAGGER_TAKEN_MULT_MAX);
                old_record(u, &id, gear, Target::StaggerTaken { max_blocks: value::int(e, "max_blocks", 0).max(0) }, mult);
                u.stagger_taken.push(StaggerTaken { mult, max_blocks: value::int(e, "max_blocks", 0).max(0) });
            }
            "forced_move" => {
                old_record(u, &id, gear, Target::ForcedTiles, f64::from(value::int(e, "bonus_tiles", 0).clamp(0, 2)));
                if gear {
                    old_forced_add(&mut u.gear_forced, e);
                } else {
                    old_forced_add(&mut u.passive_forced, e);
                }
            }
            "trigger" => {
                let from = if gear { format!("gear:{}", source.trim_start_matches("gear:").split(':').next().unwrap_or("")) } else { source.to_string() };
                let list = if gear { &mut u.gear_triggers } else { &mut u.passive_triggers };
                let key = format!("{source}#{}", list.len());
                list.push(HeldTrigger { source: from, key, row: TriggerRow::from_obj(e) });
            }
            "meter" if !gear => {
                let floor = e.get("floor").filter(|v| v.is_number()).map(|v| to_int(v, 0));
                u.passive_meters.push(HeldMeter { meter: value::string(e, "meter", ""), start: value::int(e, "start", -1), floor });
            }
            // PK2-S1: the member grants (`rpgkit_data::MEMBER_GRANTS`), passives only
            "forbid_hands" if !gear => u.forbid_hands = u.forbid_hands.max(value::int(e, "hands", 2)),
            "pierce_melee" if !gear => u.pierce_melee = true,
            _ => {}
        }
    }
}

fn old_apply_band(u: &mut Unit, cat: &Catalog, effects: &[Value], source: &str) -> BandApplied {
    let resist = u.status_resist.clone();
    let (fam, group) = (u.potency_bonus.clone(), u.potency_group.clone());
    let (potency, stagger) = (u.passive_status_potency, u.stagger_taken.len());
    let mut crit = 0.0;
    let kinds = ["stat", "resist", "stagger_taken", "skill_potency", "status_potency"];
    let passive: Vec<Value> = effects.iter().filter(|e| e.as_object().is_some_and(|o| kinds.contains(&value::string(o, "type", "").as_str()))).cloned().collect();
    old_apply_effects(u, cat, &passive, source, false);
    for e in effects.iter().filter_map(Value::as_object).filter(|o| value::string(o, "type", "") == "crit") {
        crit += value::float(e, "value", 0.0);
    }
    u.trigger_crit += crit;
    let diff_i = |now: &IndexMap<String, i32>, was: &IndexMap<String, i32>| -> Vec<(String, i32)> {
        now.iter().filter_map(|(k, v)| Some((k.clone(), v - was.get(k).copied().unwrap_or(0))).filter(|(_, d)| *d != 0)).collect()
    };
    let diff_f = |now: &IndexMap<String, f64>, was: &IndexMap<String, f64>| -> Vec<(String, f64)> {
        now.iter().filter_map(|(k, v)| Some((k.clone(), v - was.get(k).copied().unwrap_or(0.0))).filter(|(_, d)| *d != 0.0)).collect()
    };
    BandApplied {
        source: format!("passive:{}", source.trim_start_matches("passive:")),
        resist: diff_i(&u.status_resist, &resist),
        resist_new: u.status_resist.keys().filter(|k| !resist.contains_key(*k)).cloned().collect(),
        potency_family: diff_f(&u.potency_bonus, &fam),
        potency_group: diff_f(&u.potency_group, &group),
        status_potency: u.passive_status_potency - potency,
        stagger: u.stagger_taken[stagger..].to_vec(),
        crit,
        // PK1-S1: the band-only heal_taken came after this reference (no shipped band holds it)
        heal_taken: 0,
    }
}

fn old_with_drills<'a>(mut p: Proficiency, cat: &Catalog, ids: impl IntoIterator<Item = &'a String>) -> Proficiency {
    for passive in ids.into_iter().filter_map(|id| cat.passives.get(id)).filter(|p| p.slot.is_empty()) {
        for effect in passive.effects.iter().filter_map(Value::as_object) {
            if value::string(effect, "type", "") == "proficiency" {
                p.add_bonus(&value::string(effect, "gear", ""), value::int(effect, "levels", 0));
            }
        }
    }
    p
}

/// Both readers on a copy of `base`; panics on the first difference.
fn double_load(base: &Unit, cat: &Catalog, effects: &[Value], source: &str, gear: bool, what: &str) {
    let (mut a, mut b) = (base.clone(), base.clone());
    old_apply_effects(&mut a, cat, effects, source, gear);
    b.apply_effects(cat, effects, source, gear);
    assert_same(&a, &b, what);
}

/// A band of `effects` plus a crit row and a kind no band applies, through both readers, then off again.
fn double_band(base: &Unit, cat: &Catalog, effects: &[Value], what: &str) {
    let mut band: Vec<Value> = effects.to_vec();
    band.push(json!({"type": "crit", "value": 4.5}));
    band.push(json!({"type": "ap_regen", "value": 1}));
    let (mut a, mut b) = (base.clone(), base.clone());
    let (ra, rb) = (old_apply_band(&mut a, cat, &band, "meter:m"), b.apply_band(cat, &band, "meter:m"));
    assert_eq!(ra, rb, "{what}: band applied");
    assert_same(&a, &b, &format!("{what}: band on"));
    a.remove_band(&ra);
    b.remove_band(&rb);
    assert_same(&a, &b, &format!("{what}: band off"));
}

// ---- The double load ----------------------------------------------------------------------------------------------

#[test]
fn every_passive_row_evaluates_the_same_through_the_typed_view() {
    for (name, cat) in catalogs() {
        let rows: Vec<&PassiveDef> = cat.passives.values().collect();
        for (ui, base) in units(&cat).iter().enumerate() {
            for p in &rows {
                let what = format!("{name} unit {ui} passive {}", p.id);
                double_load(base, &cat, &p.effects, &format!("passive:{}", p.id), false, &what);
                double_load(base, &cat, &p.effects, &format!("gear:{}:tier1", p.id), true, &format!("{what} as gear"));
                double_band(base, &cat, &p.effects, &what);
            }
            // all rows in order and in reverse, one unit: trigger keys, the foresight source and meter order accumulate
            for order in [rows.clone(), rows.iter().rev().copied().collect()] {
                let (mut a, mut b) = (base.clone(), base.clone());
                for p in &order {
                    old_apply_effects(&mut a, &cat, &p.effects, &format!("passive:{}", p.id), false);
                    b.apply_effects(&cat, &p.effects, &format!("passive:{}", p.id), false);
                }
                assert_same(&a, &b, &format!("{name} unit {ui}: every row in turn"));
            }
            // as `apply_passive` (once per id, the slot rule, the path refresh) against the old body
            let (mut a, mut b) = (base.clone(), base.clone());
            for p in &rows {
                if !p.id.is_empty() && !a.passives.contains(&p.id) && a.slotted(p) {
                    a.passives.push(p.id.clone());
                    old_apply_effects(&mut a, &cat, &p.effects, &format!("passive:{}", p.id), false);
                    a.refresh_path_skills(&cat);
                }
                b.apply_passive(&cat, p);
            }
            assert_same(&a, &b, &format!("{name} unit {ui}: apply_passive"));
        }
    }
}

#[test]
fn item_effects_evaluate_the_same_through_the_typed_view() {
    // the six item types (FB-25) in one hand-built list, plus every catalog item's effects
    let six = vec![
        json!({"type": "skill_potency", "family": "fire", "value": 0.15}),
        json!({"type": "skill_potency", "group": "spell", "value": 0.1}),
        json!({"type": "status_potency", "value": 5}),
        json!({"type": "spell_reach", "value": 1}),
        json!({"type": "resist", "status": "burn", "value": 60}),
        json!({"type": "resist", "status": "burn", "value": 60}),
        json!({"type": "trigger", "on": "hit", "effects": [{"type": "crit", "value": 5}], "limit": "round"}),
        json!({"type": "trigger", "on": "kill", "effects": [], "limit": "battle"}),
        json!({"type": "forced_move", "immune": "guarding", "bonus_tiles": 3}),
        // kinds an item may not carry: the gear filter drops these, the rest apply as before
        json!({"type": "unlock", "family": "fire", "levels": 2}),
        json!({"type": "foresight", "range": 3}),
        json!({"type": "stagger_taken", "mult": 0.5}),
        json!({"type": "meter", "meter": "m", "floor": 1}),
        json!({"type": "ap_regen", "value": 1}),
    ];
    let mut items = 0;
    for (name, cat) in catalogs() {
        for (ui, base) in units(&cat).iter().enumerate() {
            double_load(base, &cat, &six, "gear:ring", true, &format!("{name} unit {ui}: six item types"));
            double_load(base, &cat, &six, "gear:ring:tier2", true, &format!("{name} unit {ui}: six item types, tier"));
            for item in cat.items.values().filter(|i| !i.effects.is_empty()) {
                items += 1;
                double_load(base, &cat, &item.effects, &format!("gear:{}", item.id), true, &format!("{name} unit {ui}: item {}", item.id));
            }
        }
    }
    assert!(items > 0, "a catalog item carries effects");
}

#[test]
fn drills_read_the_same_through_the_typed_view() {
    for (name, cat) in catalogs() {
        let ids: Vec<String> = cat.passives.keys().cloned().collect();
        assert!(cat.passives.values().any(|p| p.effects.iter().any(|e| e["type"] == "proficiency")), "{name}: a drill row");
        for class in cat.classes.values() {
            let old = old_with_drills(Proficiency::from_class(class), &cat, &ids);
            let new = Proficiency::from_class(class).with_drills(&cat, &ids);
            assert_eq!(format!("{old:?}"), format!("{new:?}"), "{name} class {}", class.id);
        }
    }
    // a slottable drill trains only while slotted (a battle's deck), so the gear rule never counts it
    let cat = Catalog::from_value(&json!({"passives": [
        {"id": "d1", "effects": [{"type": "proficiency", "gear": "Sword", "levels": 1}]},
        {"id": "d2", "slot": "passive", "effects": [{"type": "proficiency", "gear": "sword", "levels": 2}]}
    ]}));
    let ids = vec!["d1".to_string(), "d2".to_string()];
    let p = Proficiency::from_class(&ClassDef::from_obj(&Map::new())).with_drills(&cat, &ids);
    assert_eq!(p.bonus_levels.get("sword").copied(), Some(1), "the always-on drill only, keyed lowercase");
}

// ---- Combine rules (FB-26) ----------------------------------------------------------------------------------------

/// A bare unit of the frozen catalog and the catalog.
fn bare() -> (Catalog, Unit) {
    let cat = frozen();
    let u = units(&cat).remove(0);
    (cat, u)
}

#[test]
fn each_target_has_its_combine_rule() {
    let rule = |e: Value| match grants::read(&[e]).remove(0) {
        Grant::Modifier(m) => m.target.combine(),
        g => panic!("not a modifier: {g:?}"),
    };
    assert_eq!(rule(json!({"type": "stat", "stat": "STR", "value": 1})), Combine::Sum);
    assert_eq!(rule(json!({"type": "ap_regen", "value": 1})), Combine::Sum);
    assert_eq!(rule(json!({"type": "status_potency", "value": 1})), Combine::Sum);
    assert_eq!(rule(json!({"type": "skill_potency", "value": 0.1})), Combine::Sum);
    assert_eq!(rule(json!({"type": "resist", "status": "burn", "value": 1})), Combine::SumCapped(100));
    assert_eq!(rule(json!({"type": "stagger_taken", "mult": 0.5})), Combine::Product);
    assert_eq!(rule(json!({"type": "foresight", "range": 2})), Combine::Max);
    assert_eq!(SkillGrant::UNLOCK_COMBINE, Combine::Max);
    let forced = grants::read(&[json!({"type": "forced_move", "immune": "always", "bonus_tiles": 1})]);
    let rules: Vec<(Target, Combine)> = forced
        .iter()
        .map(|g| match g {
            Grant::Modifier(m) => (m.target.clone(), m.target.combine()),
            g => panic!("not a modifier: {g:?}"),
        })
        .collect();
    assert_eq!(rules, vec![(Target::ForcedImmune(Immunity::Always), Combine::OrFlag), (Target::ForcedTiles, Combine::Sum)]);
    // the other kinds
    let kinds = grants::read(&[
        json!({"type": "proficiency", "gear": "bow", "levels": 1}),
        json!({"type": "skill", "skill": "s"}),
        json!({"type": "meter", "meter": "m"}),
        json!({"type": "trigger", "on": "hit"}),
        json!({"type": "no_such_type"}),
    ]);
    assert!(matches!(kinds[0], Grant::Proficiency { .. }));
    assert!(matches!(kinds[1], Grant::GrantSkill(SkillGrant::Learn(_))));
    assert!(matches!(kinds[2], Grant::GrantPool { start: -1, floor: None, .. }));
    assert!(matches!(kinds[3], Grant::Trigger(_)));
    assert_eq!(kinds.len(), 4, "an unknown type gives no grant");
}

#[test]
fn sum_adds_every_row() {
    let (cat, mut u) = bare();
    let (regen, potency, fam) = (u.ap_regen_bonus, u.passive_status_potency, u.potency_bonus.get("fire").copied().unwrap_or(0.0));
    u.apply_effects(&cat, &[json!({"type": "ap_regen", "value": 1}), json!({"type": "ap_regen", "value": 2})], "passive:a", false);
    u.apply_effects(&cat, &[json!({"type": "status_potency", "value": 3}), json!({"type": "status_potency", "value": 4})], "passive:b", false);
    u.apply_effects(&cat, &[json!({"type": "skill_potency", "family": "fire", "value": 0.25}), json!({"type": "skill_potency", "family": "fire", "value": 0.5})], "passive:c", false);
    assert_eq!(u.ap_regen_bonus, regen + 1 + 2);
    assert_eq!(u.passive_status_potency, potency + 3 + 4);
    assert_eq!(u.potency_bonus["fire"], fam + 0.25 + 0.5);
}

#[test]
fn sum_capped_caps_resist_per_status() {
    let (cat, mut u) = bare();
    u.status_resist.clear();
    u.apply_effects(&cat, &[json!({"type": "resist", "status": "burn", "value": 70}), json!({"type": "resist", "status": "burn", "value": 50}), json!({"type": "resist", "status": "chill", "value": 30})], "passive:r", false);
    assert_eq!(u.status_resist["burn"], 100, "70 + 50 capped at 100");
    assert_eq!(u.status_resist["chill"], 30, "each status apart");
}

#[test]
fn max_keeps_the_largest_and_the_first_source() {
    let (cat, mut u) = bare();
    u.foresight_source.clear();
    u.foresight_range = 0;
    for (id, range) in [("f1", 3), ("f2", 5), ("f3", 99)] {
        u.apply_effects(&cat, &[json!({"type": "foresight", "range": range})], &format!("passive:{id}"), false);
    }
    assert_eq!(u.foresight_range, FORESIGHT_RANGE_MAX, "the largest, capped");
    assert_eq!(u.foresight_source, "f1", "the first source keeps the id");
    u.unlock_bonus_family.clear();
    for levels in [1, 2, 1, 7] {
        u.apply_effects(&cat, &[json!({"type": "unlock", "family": "fire", "levels": levels})], "passive:u", false);
    }
    assert_eq!(u.unlock_bonus_family["fire"], 2, "the largest unlock level, 0-2");
}

#[test]
fn product_multiplies_the_matching_stagger_rows() {
    let (cat, mut u) = bare();
    u.stagger_taken.clear();
    u.apply_effects(&cat, &[json!({"type": "stagger_taken", "mult": 0.5}), json!({"type": "stagger_taken", "mult": 0.5, "max_blocks": 1}), json!({"type": "stagger_taken", "mult": 9.0})], "passive:s", false);
    let factor = |blocks: i32| u.stagger_taken.iter().filter(|e| e.matches(blocks)).map(|e| e.mult).fold(1.0, |h, v| Combine::Product.fold_f(h, v));
    assert_eq!(factor(1), 0.5 * 0.5 * STAGGER_TAKEN_MULT_MAX, "every row matches a 1-block action; the mult is capped");
    assert_eq!(factor(3), 0.5 * STAGGER_TAKEN_MULT_MAX, "the 1-block row does not match a 3-block action");
}

#[test]
fn or_flag_sets_immunity_and_tiles_sum() {
    let (cat, mut u) = bare();
    u.passive_forced = ForcedTraits::default();
    u.apply_effects(&cat, &[json!({"type": "forced_move", "immune": "guarding", "bonus_tiles": 1}), json!({"type": "forced_move", "bonus_tiles": 5}), json!({"type": "forced_move", "immune": "guarding"})], "passive:m", false);
    assert_eq!(u.passive_forced, ForcedTraits { immune_always: false, immune_guarding: true, bonus_tiles: 1 + 2 }, "a row's tiles cap at 2");
    u.apply_effects(&cat, &[json!({"type": "forced_move", "immune": "always"})], "passive:n", false);
    assert!(u.passive_forced.immune_always && u.passive_forced.immune_guarding, "any source sets a flag; none clears one");
}

// ---- Mutations the double load catches ----------------------------------------------------------------------------

#[test]
fn a_changed_combine_rule_is_caught() {
    // resist folded by any other rule than its own gives another unit than the raw reader's
    let (cat, base) = bare();
    let rows = [json!({"type": "resist", "status": "burn", "value": 70}), json!({"type": "resist", "status": "burn", "value": 50})];
    let mut raw = base.clone();
    old_apply_effects(&mut raw, &cat, &rows, "passive:r", false);
    let held = base.status_resist.get("burn").copied().unwrap_or(0);
    let fold = |c: Combine| {
        grants::read(&rows).iter().fold(held, |h, g| match g {
            Grant::Modifier(Modifier { value, .. }) => c.fold_i(h, *value as i32),
            _ => h,
        })
    };
    let own = match &grants::read(&rows)[0] {
        Grant::Modifier(m) => m.target.combine(),
        _ => unreachable!(),
    };
    assert_eq!(fold(own), raw.status_resist["burn"], "the target's own rule matches the raw reader");
    for other in [Combine::Sum, Combine::Max, Combine::Product, Combine::OrFlag] {
        let mut mutated = base.clone();
        mutated.status_resist.insert("burn".into(), fold(other));
        assert!(!same(&raw, &mutated), "resist under {other:?} must differ from the raw reader");
    }
}

#[test]
fn a_changed_trigger_order_is_caught() {
    let (cat, base) = bare();
    let rows = [json!({"type": "trigger", "on": "hit", "limit": "round"}), json!({"type": "trigger", "on": "kill", "limit": "battle"})];
    let mut raw = base.clone();
    old_apply_effects(&mut raw, &cat, &rows, "passive:t", false);
    let mut typed = base.clone();
    typed.apply_grants(&cat, &grants::read(&rows), "passive:t", false);
    assert_same(&raw, &typed, "trigger order");
    let keys: Vec<&str> = typed.passive_triggers.iter().map(|t| t.key.as_str()).collect();
    assert_eq!(keys, ["passive:t#0", "passive:t#1"], "limit keys are source#index in row order");
    let mut reversed = grants::read(&rows);
    reversed.reverse();
    let mut mutated = base.clone();
    mutated.apply_grants(&cat, &reversed, "passive:t", false);
    assert!(!same(&raw, &mutated), "reversed triggers must differ from the raw reader");
}
