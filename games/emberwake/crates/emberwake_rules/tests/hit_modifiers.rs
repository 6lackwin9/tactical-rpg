//! U4.2-S4 proof (docs/sessions/U4_PASSIVES.md grant kind `modifier`, FB-30 slice 4; U1_STATS's pipeline): the
//! hit-time modifier grants (skill potency, `stagger_taken`, `forced_move` tiles) reach a hit as stage modifiers that
//! name their sources, with the very numbers of the accumulator path. Two paths over every passive row and every item
//! with effects of the shipped catalog and the frozen one (the timeline golden's), applied to sample units as a passive,
//! as worn gear (and as a meter band for the passives):
//!
//! - the accumulator path: `potency_for_skill`, the product `poise::factors` reads, `forced_traits().bonus_tiles`, the
//!   heal formula as it was (`roundi(base x potency)`, floor 0), `damage::compute`;
//! - the source path: the unit's records (`hit_sources`, `gear_hit_sources`) folded by each target's combine rule in the
//!   accumulators' order, and the evaluators' modifiers (`potency_modifier`, `stagger_taken_modifier`,
//!   `forced_tile_parts`), the HEAL pipeline and the traced hit (`compute_traced_skill`).
//!
//! Equal bit for bit, and every part names the source that granted it. A band left takes its sources with it.
//! The rows are game data, so the test lives in the game's façade crate.

use emberwake_data::{Catalog, SkillDef};
use emberwake_rules::damage::{self, HitInput};
use emberwake_rules::grants::{SkillSet, Target};
use emberwake_rules::pipeline::{Op, Part, StepKind};
use emberwake_rules::stats::roundi;
use emberwake_rules::unit::{skill_in_group, HitSource, GROUPS};
use emberwake_rules::{build_unit, Unit};
use serde_json::{json, Map, Value};
use std::path::PathBuf;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

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

fn frozen() -> Catalog {
    let path = crate_dir().join("../emberwake_battle/tests/fixtures/timeline_kit.json");
    Catalog::from_value(&serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap())
}

/// Sample units: the first, a middle and the last of the catalog.
fn units(cat: &Catalog) -> Vec<Unit> {
    let all: Vec<_> = cat.units.values().collect();
    let picks = [0, all.len() / 2, all.len() - 1];
    picks.iter().map(|&i| build_unit(cat, all[i], 0)).collect()
}

/// The potency the records give, folded as `potency_for_skill` folds its accumulators: 1 + (the family's passive sum)
/// + (its worn sum), then per group (passive sum + worn sum); each sum from 0 in apply order.
fn potency_from_records(u: &Unit, s: &SkillDef) -> f64 {
    let sum = |list: &[HitSource], set: &dyn Fn(&SkillSet) -> bool| -> f64 {
        list.iter().filter(|h| matches!(&h.target, Target::SkillPotency(x) if set(x))).fold(0.0, |a, h| a + h.value)
    };
    let fam = s.family_id();
    let family = |x: &SkillSet| matches!(x, SkillSet::Family(f) if f == fam);
    let mut total = 1.0 + sum(&u.hit_sources, &family) + sum(&u.gear_hit_sources, &family);
    for g in GROUPS.iter().filter(|g| skill_in_group(s, g)) {
        let group = |x: &SkillSet| matches!(x, SkillSet::Group(y) if y == g);
        total += sum(&u.hit_sources, &group) + sum(&u.gear_hit_sources, &group);
    }
    total
}

fn heals(s: &SkillDef) -> bool {
    s.kind == "SUPPORT" && (s.heal_base > 0 || s.heal_mult > 0.0)
}

/// The heal formula before U4.2-S4 (SupportRules.heal_amount), verbatim.
fn old_heal(c: &Unit, s: &SkillDef) -> i32 {
    let stat = if s.heal_stat.is_empty() { 0 } else { c.stats.stat(&s.heal_stat, c.weapon_context().as_deref()) };
    let base = s.heal_base + roundi(stat as f64 * s.heal_mult);
    roundi(base as f64 * c.potency_for_skill(s)).max(0)
}

fn parts_of(list: &[HitSource], keep: impl Fn(&Target) -> bool) -> Vec<Part> {
    list.iter().filter(|h| keep(&h.target)).map(|h| Part { source: h.source.clone().into(), value: h.value }).collect()
}

/// Every check of the two paths on unit `u` (`what` names the case; `sources` the source prefixes its records may hold).
fn both_paths(u: &Unit, target: &Unit, cat: &Catalog, what: &str, sources: &[String]) -> usize {
    let mut checked = 0;
    for h in u.hit_sources.iter().chain(&u.gear_hit_sources) {
        assert!(sources.iter().any(|p| h.source.starts_with(p.as_str())), "{what}: source {} is one of {sources:?}", h.source);
    }
    for s in cat.skills.iter() {
        let m = u.potency_modifier(s);
        let old = u.potency_for_skill(s);
        assert_eq!(m.op, Op::Mul(old), "{what}: {} potency", s.id);
        assert_eq!(potency_from_records(u, s).to_bits(), old.to_bits(), "{what}: {} potency from the records", s.id);
        assert_eq!(m.parts, u.potency_parts(s), "{what}: {}", s.id);
        if heals(s) {
            let (traced, trace) = damage::heal_traced(u, s);
            assert_eq!((damage::heal(u, s), traced), (old_heal(u, s), old_heal(u, s)), "{what}: {} heal", s.id);
            assert_eq!(trace.step("potency").map(|st| st.parts.clone()), Some(u.potency_parts(s)), "{what}: {} heal trace", s.id);
            checked += 1;
        }
        if s.kind == "ATTACK" && s.power_mult > 0.0 && !m.parts.is_empty() {
            let input = HitInput { skill_mult: u.power_for(s), ..HitInput::default() };
            let (plain, traced) = (damage::compute(u, target, input), damage::compute_traced_skill(u, target, input, s));
            assert_eq!(traced.final_dmg, plain.final_dmg, "{what}: {} hit", s.id);
            let step = traced.trace.step("skill").expect("the skill step");
            assert_eq!((step.kind, step.parts.clone()), (StepKind::Apply(Op::Mul(u.power_for(s))), u.potency_parts(s)), "{what}: {} hit trace", s.id);
            checked += 1;
        }
    }
    for blocks in 1..=6 {
        let m = u.stagger_taken_modifier(blocks);
        let product: f64 = u.stagger_taken.iter().filter(|e| e.matches(blocks)).map(|e| e.mult).product();
        assert_eq!(m.op, Op::Mul(product), "{what}: stagger_taken at {blocks} blocks");
        let from_records: f64 = u.hit_sources.iter().filter_map(|h| match h.target {
            Target::StaggerTaken { max_blocks } if max_blocks <= 0 || blocks <= max_blocks => Some(h.value),
            _ => None,
        }).product();
        assert_eq!(from_records.to_bits(), product.to_bits(), "{what}: stagger_taken from the records at {blocks} blocks");
        assert_eq!(m.parts.len(), u.stagger_taken.iter().filter(|e| e.matches(blocks)).count(), "{what}: one part per row");
    }
    let tiles = u.forced_tile_parts();
    let nonzero = |p: &Part| p.value != 0.0;
    assert_eq!(tiles, parts_of(&u.hit_sources, |t| *t == Target::ForcedTiles).into_iter().chain(parts_of(&u.gear_hit_sources, |t| *t == Target::ForcedTiles)).filter(nonzero).collect::<Vec<_>>(), "{what}: a 0-tile row names no part");
    assert_eq!(tiles.iter().map(|p| p.value as i32).sum::<i32>(), u.forced_traits().bonus_tiles, "{what}: forced tiles");
    checked
}

#[test]
fn every_passive_and_item_reaches_a_hit_with_the_accumulators_numbers_and_its_source() {
    let (mut passives, mut items, mut hits) = (0, 0, 0);
    for (name, cat) in [("shipped", shipped()), ("frozen", frozen())] {
        for (ui, base) in units(&cat).iter().enumerate() {
            let target = base.clone();
            for p in cat.passives.values() {
                passives += 1;
                let mut u = base.clone();
                let src = format!("passive:{}", p.id);
                u.apply_effects(&cat, &p.effects, &src, false);
                let allowed = [src.clone()];
                hits += both_paths(&u, &target, &cat, &format!("{name} unit {ui}: passive {}", p.id), &allowed);
                // the same row as a meter band: its sources come and go with the band
                let mut b = base.clone();
                let before = b.hit_sources.clone();
                let applied = b.apply_band(&cat, &p.effects, &format!("band:{}", p.id));
                both_paths(&b, &target, &cat, &format!("{name} unit {ui}: band {}", p.id), &[format!("passive:band:{}", p.id)]);
                b.remove_band(&applied);
                assert_eq!(b.hit_sources, before, "{name} unit {ui}: band {} left, its sources gone", p.id);
            }
            // every item worn (with its tiers and family bonuses) in its slot
            for item in cat.items.values().filter(|i| i.is_equipment() && (!i.effects.is_empty() || !i.threshold_tiers.is_empty())) {
                items += 1;
                let mut u = base.clone();
                let slot = emberwake_rules::gear::Slot::parse(&item.slot).unwrap_or(emberwake_rules::gear::Slot::MainHand);
                u.inventory.equip(emberwake_rules::gear::Equipment::new(item), slot);
                u.sync_from_stats_and_gear(&cat);
                let mut allowed: Vec<String> = u.inventory.gear().map(|e| format!("gear:{}", e.id())).collect();
                allowed.extend(u.passives.iter().map(|p| format!("passive:{p}")));
                hits += both_paths(&u, &target, &cat, &format!("{name} unit {ui}: item {}", item.id), &allowed);
            }
        }
    }
    println!("{passives} passive cases, {items} item cases, {hits} heal and hit traces");
    assert!(passives > 0 && items > 0 && hits > 0);
}

/// Hand-built rows of every hit-time target, so each part kind is seen with a value (the shipped rows may not hold all).
#[test]
fn each_hit_time_target_names_its_sources_in_order() {
    let cat = frozen();
    let base = units(&cat).remove(0);
    let skill = cat.skills.iter().find(|s| s.kind == "ATTACK").expect("an attack").clone();
    let fam = skill.family_id().to_string();
    let mut u = base.clone();
    u.apply_effects(&cat, &[json!({"type": "skill_potency", "family": fam, "value": 0.25}), json!({"type": "stagger_taken", "mult": 0.5, "max_blocks": 2})], "passive:p_one", false);
    u.apply_effects(&cat, &[json!({"type": "skill_potency", "family": fam, "value": 0.1}), json!({"type": "stagger_taken", "mult": 0.8}), json!({"type": "forced_move", "bonus_tiles": 1})], "passive:p_two", false);
    u.apply_effects(&cat, &[json!({"type": "skill_potency", "family": fam, "value": 0.05}), json!({"type": "forced_move", "bonus_tiles": 2})], "gear:ring", true);
    let names = |parts: &[Part]| parts.iter().map(|p| (p.source.to_string(), p.value)).collect::<Vec<_>>();
    assert_eq!(names(&u.potency_parts(&skill)), [("passive:p_one".to_string(), 0.25), ("passive:p_two".to_string(), 0.1), ("gear:ring".to_string(), 0.05)]);
    assert_eq!(u.potency_modifier(&skill).op, Op::Mul(u.potency_for_skill(&skill)));
    assert_eq!(names(&u.stagger_taken_modifier(1).parts), [("passive:p_one".to_string(), 0.5), ("passive:p_two".to_string(), 0.8)]);
    assert_eq!(names(&u.stagger_taken_modifier(3).parts), [("passive:p_two".to_string(), 0.8)], "the 2-block row does not match 3 blocks");
    assert_eq!(names(&u.forced_tile_parts()), [("passive:p_two".to_string(), 1.0), ("gear:ring".to_string(), 2.0)]);
    let hit = damage::compute_traced_skill(&u, &base, HitInput { skill_mult: u.power_for(&skill), ..HitInput::default() }, &skill);
    let step = hit.trace.step("skill").unwrap();
    assert_eq!(names(&step.parts), names(&u.potency_parts(&skill)), "the hit's skill step names them");
    assert!(hit.trace.steps.iter().filter(|s| s.id != "skill").all(|s| s.parts.is_empty()), "rule terms carry no parts");
    // a sync drops the worn sources only
    u.sync_from_stats_and_gear(&cat);
    assert!(u.gear_hit_sources.is_empty() && u.hit_sources.len() == 5);
}
