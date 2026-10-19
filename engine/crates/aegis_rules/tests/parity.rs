//! Exact parity with the Godot reference: unit snapshots and DamageCalculator cases exported by
//! `engine/parity/godot/export.gd` (run `engine/parity/export.sh` first; the fixtures are not committed).

use aegis_core::value::{self, Obj};
use aegis_data::{Catalog, UnitDef};
use aegis_rules::damage::{self, HitInput};
use aegis_rules::{build_unit, Stance, Unit};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../parity/out")
}

fn load(name: &str) -> Value {
    let path = out_dir().join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("missing {} - run engine/parity/export.sh to export the Godot reference", path.display()));
    serde_json::from_str(&text).unwrap()
}

fn catalog() -> Catalog {
    Catalog::from_value(&load("catalog.json"))
}

struct Diffs(Vec<String>);

impl Diffs {
    fn int(&mut self, key: &str, field: &str, godot: &Value, rust: i32) {
        if value::to_int(godot, i32::MIN) != rust {
            self.0.push(format!("{key} {field}: godot {godot} rust {rust}"));
        }
    }
    fn float(&mut self, key: &str, field: &str, godot: &Value, rust: f64) {
        if (value::to_float(godot, f64::NAN) - rust).abs() > 1e-9 {
            self.0.push(format!("{key} {field}: godot {godot} rust {rust}"));
        }
    }
    fn strs(&mut self, key: &str, field: &str, godot: &Value, rust: &[String]) {
        let g: Vec<String> = godot.as_array().map(|a| a.iter().map(value::to_str).collect()).unwrap_or_default();
        if g != rust {
            self.0.push(format!("{key} {field}: godot {g:?} rust {rust:?}"));
        }
    }
    fn check(self, what: &str, total: usize) {
        if !self.0.is_empty() {
            let shown: Vec<&String> = self.0.iter().take(40).collect();
            panic!("{what}: {} mismatches over {total} rows\n{}", self.0.len(), shown.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n"));
        }
        eprintln!("{what}: {total} rows match Godot");
    }
}

fn snapshot_rows() -> Vec<Obj> {
    load("units.json")["units"].as_array().unwrap().iter().map(|v| v.as_object().unwrap().clone()).collect()
}

fn unit_of(cat: &Catalog, row: &Obj) -> Unit {
    build_unit(cat, &UnitDef::from_obj(row["entry"].as_object().unwrap()), 0)
}

#[test]
fn unit_snapshots_match_godot() {
    let cat = catalog();
    let rows = snapshot_rows();
    let mut d = Diffs(Vec::new());
    for row in &rows {
        let key = value::string(row, "key", "");
        let k = key.as_str();
        let u = unit_of(&cat, row);
        let ctx = u.weapon_context();
        for (stat, v) in value::obj(row, "stats").unwrap() {
            d.int(k, &format!("stat {stat}"), v, u.stats.stat(stat, ctx.as_deref()));
        }
        for (stat, v) in value::obj(row, "base_stats").unwrap() {
            d.int(k, &format!("base {stat}"), v, u.stats.base(stat));
        }
        d.int(k, "unallocated", &row["unallocated"], u.stats.unallocated);
        d.int(k, "level", &row["level"], u.stats.level);
        d.float(k, "hp_scale", &row["hp_scale"], u.hp_scale);
        d.float(k, "power_scale", &row["power_scale"], u.power_scale);
        d.int(k, "max_hp", &row["max_hp"], u.max_hp);
        d.int(k, "speed", &row["speed"], u.speed);
        d.int(k, "attack_range", &row["attack_range"], u.attack_range);
        d.int(k, "min_attack_range", &row["min_attack_range"], u.min_attack_range());
        d.int(k, "max_ap", &row["max_ap"], u.max_ap);
        d.int(k, "move_per_block", &row["move_per_block"], u.move_per_block());
        d.int(k, "worn_burden", &row["worn_burden"], u.worn_burden());
        d.int(k, "power_melee", &row["power_melee"], damage::power_of(&u, false, false));
        d.int(k, "power_ranged", &row["power_ranged"], damage::power_of(&u, true, false));
        d.int(k, "power_offhand", &row["power_offhand"], damage::power_of(&u, false, true));
        d.int(k, "armor_rating", &row["armor_rating"], damage::armor_rating(&u));
        d.float(k, "crit_chance", &row["crit_chance"], damage::crit_chance(&u));
        d.float(k, "debuff_resistance", &row["debuff_resistance"], u.debuff_resistance());
        if value::boolean(row, "trained_fists", false) != damage::trained_fists(&u) {
            d.0.push(format!("{k} trained_fists differs"));
        }
        d.strs(k, "known_skills", &row["known_skills"], &u.known_skills);
        d.strs(k, "active_class_families", &row["active_class_families"], &u.active_class_families);
        d.strs(k, "active_extra_families", &row["active_extra_families"], &u.active_extra_families);
        let gear: Vec<String> = u.inventory.equipped.iter().map(|(s, e)| format!("{}={}", s.engine_name(), e.id())).collect();
        let mut g: Vec<String> = value::obj(row, "gear").unwrap().iter().map(|(s, id)| format!("{s}={}", value::to_str(id))).collect();
        let mut r = gear.clone();
        g.sort();
        r.sort();
        if g != r {
            d.0.push(format!("{k} gear: godot {g:?} rust {r:?}"));
        }
        let pouch: Vec<String> = u.inventory.pouch.iter().map(|i| i.id.clone()).collect();
        d.strs(k, "pouch", &row["pouch"], &pouch);
    }
    d.check("unit snapshots", rows.len());
}

#[test]
fn damage_cases_match_godot() {
    let cat = catalog();
    let mut units: HashMap<String, Unit> = HashMap::new();
    for row in snapshot_rows() {
        units.insert(value::string(&row, "key", ""), unit_of(&cat, &row));
    }
    let cases = load("damage.json");
    let cases = cases["cases"].as_array().unwrap();
    let mut d = Diffs(Vec::new());
    for c in cases {
        let c = c.as_object().unwrap();
        let ak = value::string(c, "attacker", "");
        let tk = value::string(c, "target", "");
        let attacker = &units[&ak];
        let mut target = units[&tk].clone();
        target.stance = Stance::from_index(value::int(c, "defender_stance", 0));
        target.guard = value::float(c, "guard", 0.0);
        let stance = value::int(c, "attacker_stance", -1);
        let hit = damage::compute(
            attacker,
            &target,
            HitInput {
                ranged: value::boolean(c, "ranged", false),
                offhand: value::boolean(c, "offhand", false),
                skill_mult: value::float(c, "skill_mult", 1.0),
                force_crit: value::boolean(c, "crit", false),
                attacker_stance: (stance >= 0).then(|| Stance::from_index(stance)),
                cover: value::float(c, "cover", 0.0),
                ..HitInput::default()
            },
        );
        let key = format!("{ak} -> {tk} {:?}", c.iter().filter(|(k, _)| !["attacker", "target", "attack_power", "armor_rating", "final_dmg"].contains(&k.as_str())).map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>());
        d.int(&key, "attack_power", &c["attack_power"], hit.attack_power);
        d.int(&key, "armor_rating", &c["armor_rating"], hit.armor_rating);
        d.int(&key, "final_dmg", &c["final_dmg"], hit.final_dmg);
    }
    d.check("damage cases", cases.len());
}

#[test]
fn rng_matches_godot() {
    use aegis_core::rng::GodotRng;
    let path = out_dir().join("rng.json");
    let rows: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("missing rng.json - run engine/parity/export.sh")).unwrap();
    let mut n = 0;
    for row in rows.as_array().unwrap() {
        let seed = row["seed"].as_i64().unwrap();
        let mut r = GodotRng::from_i64(seed);
        for d in row["draws"].as_array().unwrap() {
            let kind = d[0].as_str().unwrap();
            let want = &d[1];
            match kind {
                "randi" => assert_eq!(r.randi(), want.as_i64().unwrap(), "seed {seed} draw {n}"),
                "range" => {
                    let got = if n % 5 == 2 { r.randi_range(-1, 1) } else { r.randi_range(0, 99) };
                    assert_eq!(got as i64, want.as_i64().unwrap(), "seed {seed} draw {n}");
                }
                "randf" => assert!((r.randf() - want.as_f64().unwrap()).abs() < 1e-12, "seed {seed} draw {n}"),
                "rangef" => assert!((r.randf_range(0.5, 2.0) - want.as_f64().unwrap()).abs() < 1e-12, "seed {seed} draw {n}"),
                _ => unreachable!(),
            }
            n += 1;
        }
    }
    eprintln!("rng: {n} draws match Godot");
}

#[test]
fn godot_hash_matches() {
    use aegis_core::ghash::{hash, GVar};
    use aegis_core::rng::GodotRng;
    let rows: Value = serde_json::from_str(&std::fs::read_to_string(out_dir().join("hash.json")).expect("missing hash.json - run engine/parity/export.sh")).unwrap();
    let mut n = 0;
    for r in rows.as_array().unwrap() {
        let want = r["hash"].as_u64().unwrap() as u32;
        let got = if let Some(s) = r.get("string") {
            hash(&GVar::Str(s.as_str().unwrap().into()))
        } else if let Some(i) = r.get("int") {
            hash(&GVar::Int(i.as_i64().unwrap()))
        } else {
            let v = GVar::Arr(vec![GVar::Int(r["seed"].as_i64().unwrap()), GVar::Str(r["stream"].as_str().unwrap().into()), GVar::Int(r["n"].as_i64().unwrap())]);
            let h = hash(&v);
            assert_eq!(GodotRng::new(h as u64).randi(), r["roll"].as_i64().unwrap(), "GameRolls draw for {r}");
            h
        };
        assert_eq!(got, want, "hash of {r}");
        n += 1;
    }
    eprintln!("hash: {n} samples match Godot");
}
