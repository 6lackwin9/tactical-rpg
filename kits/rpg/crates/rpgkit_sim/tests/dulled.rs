//! PK2-S5b (`docs/sessions/PK2_GRAFTS.md` 7): the sims' durability keys. `durability=N` seeds wear on every ally's
//! wielded weapons (`rpgkit_battle::wear::seed`; the sims' allies are catalog units, which track none), so under the
//! rule `durability` a sim fight spends and dulls (`FightRes::worn`, `FightRes::dulled`); absent, nothing is tracked.
//! `dulled=power` (the default) keeps the dulled weapon's power penalty alone; `dulled=lock` also locks the techniques
//! that swing a dulled weapon (`rpgkit_battle::world::World::dulled_lock`; the lock itself is tested in
//! `rpgkit_battle/tests/durability.rs`, since a catalog ally opens no technique). The keys parse, refuse other values,
//! show in the ARGS header only when set, and their help prints apart from `usage` (which `sim_golden` holds). Checked
//! on a scratch catalog built here.

use rpgkit_sim::args::{SimArgs, DULLED_MODES, DULLED_USAGE, DURABILITY_USAGE};
use rpgkit_sim::runner::Runner;
use rpgkit_world::Data;
use serde_json::{json, Map};

/// A durability that outlasts every fight below.
const PLENTY: i32 = 500;

/// Class `drill`; skills strike (a basic) and `t_jab` (a standard attack on the class path); a blade; units `t_hero`
/// (party, wielding the blade) and `wolf` (enemy, sturdy).
fn data() -> Data {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10});
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": stats, "proficiency": {"sword": {"level": 2}}, "skill_path": {"t_jab": 1}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
            {"id": "t_jab", "name": "Jab", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0, "target_mode": "ENEMY"}
        ],
        "items": [{"id": "blade", "name": "Blade", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10}],
        "units": [
            {"id": "t_hero", "name": "t_hero", "side": "party", "class_id": "drill", "level": 3, "main_hand": "blade"},
            {"id": "wolf", "name": "wolf", "side": "enemy", "class_id": "drill", "level": 3, "tuning": {"hp_pct": 400}}
        ]
    });
    Data::from_values(&catalog, &json!({}), &json!({}))
}

/// One sim fight of `t_hero` against `wolf` (seed 7) under `line`'s keys, with the rule durability on or off: (worn,
/// dulled, won, rounds, the hero's end HP).
fn fight(d: &Data, line: &str, rule: bool) -> (i64, i32, bool, i32, Vec<i32>) {
    let game = rpgkit_sim::SimGame::default();
    let argv: Vec<String> = line.split_whitespace().map(String::from).collect();
    let mut a = SimArgs::parse_for(d, &argv, &game).expect("parses");
    if rule {
        a.ruleset.adjust("+durability").expect("an extension");
    }
    let mut r = Runner::new(d, &a);
    r.rounds_override = 4;
    let f = r.fight(&["t_hero".to_string()], &["wolf".to_string()], 7, &Map::new());
    assert!(f.ok, "{line}: the fight ran");
    (f.worn, f.dulled, f.won, f.rounds, f.party_hp.values().copied().collect())
}

#[test]
fn a_seeded_fight_wears_and_dulls_the_allies_weapons() {
    let d = data();
    let plain = fight(&d, "party", true);
    assert_eq!((plain.0, plain.1), (0, 0), "no seed: catalog allies track no wear");
    let sharp = fight(&d, &format!("party durability={PLENTY}"), true);
    assert!(sharp.0 > 0 && sharp.1 == 0, "a seeded run spends without dulling: {sharp:?}");
    assert_eq!((sharp.2, sharp.3, &sharp.4), (plain.2, plain.3, &plain.4), "wear short of 0 changes no fight");
    let dull = fight(&d, "party durability=1", true);
    assert!(dull.0 > 0 && dull.1 == 1, "a small seed dulls the blade: {dull:?}");
    let off = fight(&d, "party durability=1", false);
    assert_eq!((off.0, off.1), (0, 0), "no rule, no wear");
}

#[test]
fn the_durability_key_parses_refuses_bad_values_and_is_in_sim_help() {
    let d = data();
    let game = rpgkit_sim::SimGame::default();
    let parsed = |line: &str| SimArgs::parse_for(&d, &line.split_whitespace().map(String::from).collect::<Vec<_>>(), &game);
    let default = parsed("party").expect("parses");
    assert!(default.durability.is_none() && !default.header().contains("durability"), "absent: no wear, not echoed");
    let max = rpgkit_data::DURABILITY_MAX;
    for n in [0, 3, max] {
        let a = parsed(&format!("party durability={n}")).expect("parses");
        assert_eq!(a.durability, Some(n as i32));
        assert!(a.header().ends_with(&format!(", durability {n}")), "{}", a.header());
    }
    for bad in [String::from("-1"), (max + 1).to_string(), "x".into(), String::new()] {
        let err = parsed(&format!("party durability={bad}")).map(|a| a.durability);
        assert!(err.as_ref().is_err_and(|e| e.starts_with("durability must")), "durability={bad}: {err:?}");
    }
    let help = rpgkit_sim::run(&d, &["help".to_string()], &game);
    assert!(help.contains(&DURABILITY_USAGE.to_string()), "sim help lists durability=");
    assert!(!rpgkit_sim::args::usage(&game).contains(&DURABILITY_USAGE.to_string()), "apart from usage (sim_golden)");
}

#[test]
fn the_dulled_key_parses_refuses_bad_values_and_is_in_sim_help() {
    let d = data();
    let game = rpgkit_sim::SimGame::default();
    let argv = |line: &str| line.split_whitespace().map(String::from).collect::<Vec<_>>();
    let parsed = |line: &str| SimArgs::parse_for(&d, &argv(line), &game);
    let default = parsed("party").expect("parses");
    assert!(!default.dulled_lock && !default.header().contains("dulled"), "default: the power penalty alone, not echoed");
    assert_eq!(parsed(&format!("party dulled={}", DULLED_MODES[0])).map(|a| a.dulled_lock), Ok(false));
    let lock = parsed(&format!("party dulled={}", DULLED_MODES[1])).expect("parses");
    assert!(lock.dulled_lock && lock.header().ends_with(", dulled lock"), "{}", lock.header());
    for bad in ["", "on", "LOCK", "half"] {
        let err = parsed(&format!("party dulled={bad}")).map(|a| a.dulled_lock);
        assert!(err.as_ref().is_err_and(|e| e.starts_with("dulled must")), "dulled={bad}: {err:?}");
    }
    let help = rpgkit_sim::run(&d, &["help".to_string()], &game);
    assert!(help.contains(&DULLED_USAGE.to_string()), "sim help lists dulled=");
    assert!(!rpgkit_sim::args::usage(&game).contains(&DULLED_USAGE.to_string()), "apart from usage (sim_golden)");
}
