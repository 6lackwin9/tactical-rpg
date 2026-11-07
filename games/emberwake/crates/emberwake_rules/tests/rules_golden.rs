//! CLN-38: the unit and damage golden, the direct tests of the kit's unit building (`build_unit`, stats, gear,
//! skills) and damage formula (`damage::compute`, power, armour, crit).
//!
//! Units: every catalog unit, every class at every level (hero tier) and at every other tier at level 5, built on a
//! frozen catalog; each snapshot (stats, scales, HP, speed, range, AP, move, burden, power by hand, armour, crit,
//! debuff resistance, gear, pouch, skills, families, status resist) is compared with `tests/fixtures/rules_golden.json`.
//! Damage: a sample over a pool (every catalog unit and every class at levels 1, 5 and 10 and as a level 5 boss),
//! each attacker against two targets of the pool with each branch: hand (main, off), range (melee, ranged), crit
//! (forced, rolled high and low, none), skill power, guard, cover and armour pierce; plus the stance matrix on the
//! classes at level 5.
//!
//! The frozen catalog is the timeline golden's (`emberwake_battle/tests/fixtures/timeline_kit.json`, the shipped data
//! frozen when RULESET1-X landed; decision "Godot parity retired" Q-A6). The golden was recorded once from the engine
//! (CLN-38, 2026-11-01; it replaced the Godot unit and DamageCalculator exports). Re-record only for an intended rule
//! change (or a re-frozen timeline catalog), named in the commit and a docs/HISTORY.md row; never edit it by hand:
//!
//! `RECORD=1 tools/fc.sh test --test rules_golden`
//!
//! The units and the catalog are game data, so the test lives in the game's façade crate (the kit names no game).

use aegis_core::value::Obj;
use emberwake_data::{Catalog, UnitDef, STATS};
use emberwake_rules::damage::{self, HitInput};
use emberwake_rules::stats::MAX_LEVEL;
use emberwake_rules::{build_unit, Stance, Unit};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::OnceLock;

/// The non-hero tiers, each built at level 5.
const TIERS: [&str; 4] = ["minion", "standard", "elite", "boss"];
/// Float tolerance (the golden's floats are written in shortest form and parsed back).
const EPS: f64 = 1e-9;
/// The damage rows' columns: the inputs, then the outputs.
const COLUMNS: [&str; 19] = [
    "attacker", "target", "ranged", "offhand", "skill_mult", "crit", "crit_roll", "attacker_stance", "defender_stance", "guard", "cover",
    "ignore_armor", "attack_power", "is_crit", "crit_rate", "stance_factor", "armor_rating", "armor_pierced", "final_dmg",
];
/// Where the outputs start in `COLUMNS`.
const OUTPUTS: usize = 12;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn golden_path() -> PathBuf {
    crate_dir().join("tests/fixtures/rules_golden.json")
}

fn recording() -> bool {
    std::env::var("RECORD").is_ok_and(|v| v == "1")
}

/// The frozen catalog (shared with the timeline golden).
fn catalog() -> &'static Catalog {
    static CAT: OnceLock<Catalog> = OnceLock::new();
    CAT.get_or_init(|| {
        let path = crate_dir().join("../emberwake_battle/tests/fixtures/timeline_kit.json");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("missing the frozen catalog {}", path.display()));
        Catalog::from_value(&serde_json::from_str(&text).unwrap())
    })
}

/// Every unit of the golden, in order: (key, definition).
fn unit_specs(cat: &Catalog) -> Vec<(String, UnitDef)> {
    let mut out: Vec<(String, UnitDef)> = cat.units.iter().map(|(id, u)| (format!("unit:{id}"), u.clone())).collect();
    let class_unit = |class: &str, level: i32, tier: &str| -> UnitDef {
        let o: Obj = json!({"id": format!("golden_{class}_{level}_{tier}"), "name": class, "class_id": class, "level": level, "tier": tier})
            .as_object()
            .unwrap()
            .clone();
        UnitDef::from_obj(&o)
    };
    for class in cat.classes.keys() {
        for level in 1..=MAX_LEVEL {
            out.push((format!("class:{class}:{level}:hero"), class_unit(class, level, "hero")));
        }
        for tier in TIERS {
            out.push((format!("class:{class}:5:{tier}"), class_unit(class, 5, tier)));
        }
    }
    out
}

fn units(cat: &Catalog) -> Vec<(String, Unit)> {
    unit_specs(cat).into_iter().map(|(k, e)| (k, build_unit(cat, &e, 0))).collect()
}

fn snapshot(key: &str, u: &Unit) -> Value {
    let ctx = u.weapon_context();
    let stats: serde_json::Map<String, Value> = STATS.iter().map(|s| (s.to_string(), json!(u.stats.stat(s, ctx.as_deref())))).collect();
    let base: serde_json::Map<String, Value> = STATS.iter().map(|s| (s.to_string(), json!(u.stats.base(s)))).collect();
    let gear: Vec<String> = u.inventory.equipped.iter().map(|(s, e)| format!("{}={}", s.engine_name(), e.id())).collect();
    let pouch: Vec<&str> = u.inventory.pouch.iter().map(|i| i.id.as_str()).collect();
    let resist: serde_json::Map<String, Value> = u.status_resist.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
    json!({
        "key": key, "name": u.name, "catalog_id": u.catalog_id, "tier": u.tier, "level": u.stats.level,
        "tier_hp": u.tier_hp, "tier_power": u.tier_power, "base_stats": base, "stats": stats, "unallocated": u.stats.unallocated,
        "max_hp": u.max_hp, "initiative": u.initiative, "attack_range": u.attack_range, "min_attack_range": u.min_attack_range(), "max_ap": u.max_ap,
        "move_per_block": u.move_per_block(), "worn_burden": u.worn_burden(), "power_override": u.power_override,
        "armor_override": u.armor_override, "power_melee": damage::power_of(u, false, false),
        "power_ranged": damage::power_of(u, true, false), "power_offhand": damage::power_of(u, false, true),
        "armor_rating": damage::armor_rating(u), "crit_chance": damage::crit_chance(u), "trained_fists": damage::trained_fists(u),
        "debuff_resistance": u.debuff_resistance(), "gear": gear, "pouch": pouch, "known_skills": u.known_skills,
        "active_class_families": u.active_class_families, "active_extra_families": u.active_extra_families,
        "status_resist": resist,
    })
}

/// One damage case's inputs (the first `OUTPUTS` columns).
#[derive(Clone, Copy)]
struct Case<'a> {
    attacker: &'a str,
    target: &'a str,
    ranged: bool,
    offhand: bool,
    skill_mult: f64,
    crit: bool,
    crit_roll: Option<f64>,
    attacker_stance: i32,
    defender_stance: i32,
    guard: f64,
    cover: f64,
    ignore_armor: bool,
}

impl<'a> Case<'a> {
    fn plain(attacker: &'a str, target: &'a str) -> Self {
        Self {
            attacker,
            target,
            ranged: false,
            offhand: false,
            skill_mult: 1.0,
            crit: false,
            crit_roll: None,
            attacker_stance: -1,
            defender_stance: 0,
            guard: 0.0,
            cover: 0.0,
            ignore_armor: false,
        }
    }

    fn from_row(row: &'a [Value]) -> Self {
        let f = |i: usize| row[i].as_f64().unwrap();
        let b = |i: usize| row[i].as_bool().unwrap();
        let n = |i: usize| row[i].as_i64().unwrap() as i32;
        Self {
            attacker: row[0].as_str().unwrap(),
            target: row[1].as_str().unwrap(),
            ranged: b(2),
            offhand: b(3),
            skill_mult: f(4),
            crit: b(5),
            crit_roll: row[6].as_f64(),
            attacker_stance: n(7),
            defender_stance: n(8),
            guard: f(9),
            cover: f(10),
            ignore_armor: b(11),
        }
    }

    /// The row: inputs then `damage::compute`'s outputs.
    fn run(&self, by_key: &[(String, Unit)]) -> Vec<Value> {
        let find = |k: &str| &by_key.iter().find(|(key, _)| key == k).unwrap_or_else(|| panic!("no unit {k} in the golden's units")).1;
        let attacker = find(self.attacker);
        let mut target = find(self.target).clone();
        target.stance = Stance::from_index(self.defender_stance);
        target.guard = self.guard;
        let hit = damage::compute(
            attacker,
            &target,
            HitInput {
                ranged: self.ranged,
                offhand: self.offhand,
                skill_mult: self.skill_mult,
                force_crit: self.crit,
                crit_roll: self.crit_roll,
                attacker_stance: (self.attacker_stance >= 0).then(|| Stance::from_index(self.attacker_stance)),
                cover: self.cover,
                ignore_armor: self.ignore_armor,
                ..HitInput::default()
            },
        );
        vec![
            json!(self.attacker),
            json!(self.target),
            json!(self.ranged),
            json!(self.offhand),
            json!(self.skill_mult),
            json!(self.crit),
            json!(self.crit_roll),
            json!(self.attacker_stance),
            json!(self.defender_stance),
            json!(self.guard),
            json!(self.cover),
            json!(self.ignore_armor),
            json!(hit.attack_power),
            json!(hit.is_crit),
            json!(hit.crit_rate),
            json!(hit.stance_factor),
            json!(hit.armor_rating),
            json!(hit.armor_pierced),
            json!(hit.final_dmg),
        ]
    }
}

/// The damage pool: every catalog unit and every class at levels 1, 5, 10 and as a level 5 boss.
fn in_pool(key: &str) -> bool {
    key.starts_with("unit:") || [":1:hero", ":5:hero", ":10:hero", ":5:boss"].iter().any(|s| key.ends_with(s))
}

/// Every damage case of the golden, in order.
fn damage_cases(keys: &[String]) -> Vec<Case<'_>> {
    let pool: Vec<&str> = keys.iter().map(String::as_str).filter(|k| in_pool(k)).collect();
    let n = pool.len();
    let mut out = Vec::new();
    for (a, attacker) in pool.iter().enumerate() {
        for t in [(a + 1) % n, (a + n / 2) % n] {
            let base = Case::plain(attacker, pool[t]);
            for flags in 0..8 {
                out.push(Case { ranged: flags & 1 != 0, offhand: flags & 2 != 0, crit: flags & 4 != 0, ..base });
            }
            out.push(Case { skill_mult: 1.75, guard: 0.3, ..base });
            out.push(Case { ranged: true, skill_mult: 0.6, guard: 0.5, cover: 0.5, ..base });
            out.push(Case { ignore_armor: true, ..base });
            out.push(Case { ignore_armor: true, guard: 0.5, cover: 0.5, ..base });
            out.push(Case { crit_roll: Some(0.0), ..base });
            out.push(Case { crit_roll: Some(0.999), ..base });
        }
    }
    let classes: Vec<&str> = keys.iter().map(String::as_str).filter(|k| k.starts_with("class:") && k.ends_with(":5:hero")).collect();
    for (a, attacker) in classes.iter().enumerate() {
        let target = classes[(a + 1) % classes.len()];
        for sa in 0..4 {
            for sd in 0..4 {
                out.push(Case { attacker_stance: sa, defender_stance: sd, ..Case::plain(attacker, target) });
            }
        }
    }
    out
}

/// The golden from the current engine: `{units: [snapshot], columns, damage: [row]}`, one unit or row per line.
fn record(cat: &Catalog) -> String {
    let built = units(cat);
    let unit_lines: Vec<String> = built.iter().map(|(k, u)| format!("    {}", snapshot(k, u))).collect();
    let keys: Vec<String> = built.iter().map(|(k, _)| k.clone()).collect();
    let rows: Vec<String> = damage_cases(&keys).iter().map(|c| format!("    {}", Value::Array(c.run(&built)))).collect();
    format!(
        "{{\n  \"units\": [\n{}\n  ],\n  \"columns\": {},\n  \"damage\": [\n{}\n  ]\n}}\n",
        unit_lines.join(",\n"),
        json!(COLUMNS),
        rows.join(",\n")
    )
}

fn golden() -> &'static Value {
    static GOLDEN: OnceLock<Value> = OnceLock::new();
    GOLDEN.get_or_init(|| {
        let path = golden_path();
        if recording() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, record(catalog())).unwrap();
            eprintln!("recorded {}", path.display());
        }
        let text = std::fs::read_to_string(&path).expect("tests/fixtures/rules_golden.json (recorded by CLN-38)");
        serde_json::from_str(&text).unwrap()
    })
}

/// Differences between the golden `g` and the engine's `r` (numbers within `EPS`), as "path: golden x engine y".
fn differences(at: &str, g: &Value, r: &Value, out: &mut Vec<String>) {
    match (g, r) {
        (Value::Number(a), Value::Number(b)) => {
            if (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() > EPS {
                out.push(format!("{at}: golden {g} engine {r}"));
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            let ka: Vec<&String> = a.keys().collect();
            let kb: Vec<&String> = b.keys().collect();
            if ka != kb {
                out.push(format!("{at}: golden keys {ka:?} engine keys {kb:?}"));
                return;
            }
            for (k, v) in a {
                differences(&format!("{at}.{k}"), v, &b[k], out);
            }
        }
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                differences(&format!("{at}[{i}]"), x, y, out);
            }
        }
        _ if g != r => out.push(format!("{at}: golden {g} engine {r}")),
        _ => {}
    }
}

fn check(what: &str, diffs: Vec<String>, total: usize) {
    assert!(diffs.is_empty(), "{what}: {} differences over {total} rows\n{}", diffs.len(), diffs.iter().take(40).cloned().collect::<Vec<_>>().join("\n"));
    eprintln!("{what}: {total} rows match the golden");
}

#[test]
fn unit_snapshots_match_the_golden() {
    let rows = golden()["units"].as_array().unwrap();
    let built = units(catalog());
    let keys: Vec<&str> = rows.iter().map(|r| r["key"].as_str().unwrap()).collect();
    let want: Vec<&str> = built.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(keys, want, "the golden's units are not the frozen catalog's (re-record after re-freezing it)");
    let mut d = Vec::new();
    for (row, (k, u)) in rows.iter().zip(&built) {
        differences(k, row, &snapshot(k, u), &mut d);
    }
    check("unit snapshots", d, rows.len());
}

#[test]
fn damage_cases_match_the_golden() {
    let g = golden();
    assert_eq!(g["columns"], json!(COLUMNS), "the golden's damage columns");
    let rows = g["damage"].as_array().unwrap();
    let built = units(catalog());
    let mut d = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let row = row.as_array().unwrap();
        let case = Case::from_row(row);
        let got = case.run(&built);
        let at = format!("case {i} {} -> {}", case.attacker, case.target);
        for c in OUTPUTS..COLUMNS.len() {
            differences(&format!("{at} {}", COLUMNS[c]), &row[c], &got[c], &mut d);
        }
    }
    check("damage cases", d, rows.len());
}

/// The golden holds exactly the cases this file records (a re-record cannot silently drop one), and the sample
/// reaches each branch: crits forced and rolled, guard, cover, pierce, off hand, ranged and every stance.
#[test]
fn the_damage_sample_covers_every_branch() {
    let g = golden();
    let rows = g["damage"].as_array().unwrap();
    let keys: Vec<String> = g["units"].as_array().unwrap().iter().map(|r| r["key"].as_str().unwrap().to_string()).collect();
    let cases = damage_cases(&keys);
    assert_eq!(rows.len(), cases.len(), "the golden's damage rows");
    let col = |name: &str| COLUMNS.iter().position(|c| *c == name).unwrap();
    let any = |name: &str, f: &dyn Fn(&Value) -> bool| rows.iter().any(|r| f(&r[col(name)]));
    assert!(any("is_crit", &|v| v == &json!(true)) && any("is_crit", &|v| v == &json!(false)), "crit and no crit");
    assert!(any("crit_roll", &|v| v.is_number()), "a rolled crit");
    assert!(any("armor_pierced", &|v| v == &json!(true)), "an armour pierce");
    assert!(any("guard", &|v| v.as_f64().unwrap() > 0.0) && any("cover", &|v| v.as_f64().unwrap() > 0.0), "guard and cover");
    for s in 0..4 {
        assert!(any("attacker_stance", &|v| v.as_i64() == Some(s)) && any("defender_stance", &|v| v.as_i64() == Some(s)), "stance {s}");
    }
    let classes = catalog().classes.len();
    let class_rows = keys.iter().filter(|k| k.starts_with("class:")).count();
    assert_eq!(class_rows, classes * (MAX_LEVEL as usize + TIERS.len()), "every class at every level and tier");
}
