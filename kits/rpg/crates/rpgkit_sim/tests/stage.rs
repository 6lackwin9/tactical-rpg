//! PR4-1 (R3-STAGE-SIM): `sim stage=N` fights a world encounter at world stage N, so an enemy grows by its unit's
//! `growth_per_stage` exactly as the campaign's `WorldRoster` builds it (`rpgkit_world::battle::stage_level`: a hero
//! tier or a guest never grows, capped at the top unit level); a staged enemy copy goes through
//! `rpgkit_rules::unit::raise_level`, so an explicit `spend` gets the raised levels' points (an unstaged `elevel=`
//! copy does not yet: AUDIT-07's routing moves `sim_golden`, parked as limit-blocker FB-148).
//! Checked on a scratch world built here.

use rpgkit_rules::stats::{spend_pool, MAX_LEVEL};
use rpgkit_sim::args::SimArgs;
use rpgkit_sim::runner::Runner;
use rpgkit_sim::world;
use rpgkit_world::Data;
use serde_json::json;

const BASE: i32 = 2;
const GROWTH: i32 = 2;
/// The tuning `power` of character `champ` (its extra).
const CHAMP_POWER: i32 = 1;

/// Units `wolf` (standard, growth), `rival` (hero tier, growth), `squire` (a guest, growth), `old` (one below the top
/// level, growth), `brute` (standard, growth, an explicit spend of its whole base pool) and `rat` (no growth); world
/// `vale` with encounter `pack` (wolf, rival, rat, old; guest squire), `pit` (brute) and `duel` (`champ`: a wolf
/// character with tuning, so a sandbox copy).
fn data() -> Data {
    let unit = |id: &str, tier: &str, level: i32, growth: i32| json!({"id": id, "name": id, "side": "enemy", "class_id": "drill", "level": level, "tier": tier, "growth_per_stage": growth});
    let mut brute = unit("brute", "standard", BASE, GROWTH);
    brute["spend"] = json!({"STR": spend_pool(BASE)});
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}
        ],
        "units": [
            unit("wolf", "standard", BASE, GROWTH),
            unit("rival", "hero", BASE, GROWTH),
            unit("squire", "standard", BASE, GROWTH),
            unit("old", "standard", MAX_LEVEL - 1, GROWTH),
            unit("rat", "standard", BASE, 0),
            brute
        ]
    });
    let worlds = json!({
        "vale": {
            "manifest": {"id": "vale"},
            "own": {
                "characters": [
                    {"id": "wolf", "name": "Wolf", "base_unit": "wolf"},
                    {"id": "rival", "name": "Rival", "base_unit": "rival"},
                    {"id": "squire", "name": "Squire", "base_unit": "squire"},
                    {"id": "old", "name": "Old", "base_unit": "old"},
                    {"id": "rat", "name": "Rat", "base_unit": "rat"},
                    {"id": "brute", "name": "Brute", "base_unit": "brute"},
                    {"id": "champ", "name": "Champ", "base_unit": "wolf", "tuning": {"power": CHAMP_POWER}}
                ],
                "encounters": [
                    {"id": "pack", "name": "Pack", "enemies": ["wolf", "rival", "rat", "old"], "guests": ["squire"]},
                    {"id": "pit", "name": "Pit", "enemies": ["brute"]},
                    {"id": "duel", "name": "Duel", "enemies": ["champ"]}
                ]
            }
        }
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn args(stage: i32) -> SimArgs {
    let mut a = SimArgs::default();
    a.stage = stage;
    a
}

/// The level each enemy, then each guest, of `pack` spawns at with `a`.
fn pack_levels(d: &Data, a: &SimArgs) -> (Vec<i32>, Vec<i32>) {
    let mut r = Runner::new(d, a);
    let enc = world::encounter(&mut r, "pack", 1, false, "vale", true).expect("the scratch encounter");
    let mut level = |id: &String| r.spawn_row(id).map(|(row, _)| row.level).unwrap_or(-1);
    let foes = enc.enemies.iter().map(&mut level).collect();
    let guests = guests_of(&enc.base).iter().map(&mut level).collect();
    (foes, guests)
}

fn guests_of(base: &serde_json::Map<String, serde_json::Value>) -> Vec<String> {
    base.get("guests").and_then(|g| g.as_array()).map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default()
}

#[test]
fn stage_zero_is_the_authored_roster() {
    let d = data();
    let a = args(0);
    let mut r = Runner::new(&d, &a);
    let enc = world::encounter(&mut r, "pack", 1, false, "vale", true).expect("the scratch encounter");
    assert_eq!(enc.enemies, ["wolf", "rival", "rat", "old"], "stage 0: the base units themselves");
    assert_eq!(pack_levels(&d, &a), (vec![BASE, BASE, BASE, MAX_LEVEL - 1], vec![BASE]));
}

#[test]
fn a_growing_enemy_sims_at_base_plus_stage_times_growth_capped_at_the_top_level() {
    let d = data();
    let stage = 3;
    let (foes, guests) = pack_levels(&d, &args(stage));
    assert_eq!(foes[0], BASE + stage * GROWTH, "the wolf at base + stage x growth");
    assert_eq!(foes[1], BASE, "a hero-tier enemy never grows");
    assert_eq!(foes[2], BASE, "growth 0 never grows");
    assert_eq!(foes[3], MAX_LEVEL, "never above the top level");
    assert_eq!(guests, [BASE], "a guest fights on the party's side and never grows");
    // the same rule the campaign's roster reads
    let want: Vec<i32> = rpgkit_world::battle::enemy_levels_at(&d, "vale", "pack", i64::from(stage)).iter().map(|(l, _)| *l as i32).collect();
    assert_eq!(foes, want, "the sim's levels are the world's enemy_levels_at");
}

#[test]
fn a_staged_elevel_copy_grows_from_the_shifted_level() {
    let d = data();
    let stage = 2;
    let a = args(stage);
    let mut r = Runner::new(&d, &a);
    let shifted = BASE + 1;
    let enc = world::encounter(&mut r, "pack", shifted, true, "vale", true).expect("the scratch encounter");
    let (row, _) = r.spawn_row(&enc.enemies[0]).expect("the wolf writes");
    let mut at_shift = d.entry("units", "wolf").clone();
    at_shift.insert("level".into(), json!(shifted));
    let want = rpgkit_world::battle::stage_level(&at_shift, i64::from(stage)) as i32;
    assert_ne!(want, shifted, "the fixture grows");
    assert_eq!(row.level, want, "stage_level on the elevel-shifted row");
}

#[test]
fn a_staged_character_with_extras_is_its_own_staged_copy() {
    let d = data();
    let unstaged = args(0);
    let mut r = Runner::new(&d, &unstaged);
    let base = world::encounter(&mut r, "duel", 1, false, "vale", true).expect("the scratch encounter").enemies[0].clone();
    assert!(base.starts_with("simw_"), "a character with extras is a sandbox copy: {base}");
    assert_eq!(r.spawn_row(&base).map(|(row, _)| row.level), Some(BASE));
    let stage = 3;
    let a = args(stage);
    let mut r = Runner::new(&d, &a);
    let copy = world::encounter(&mut r, "duel", 1, false, "vale", true).expect("the scratch encounter").enemies[0].clone();
    assert_eq!(copy, format!("{base}_s{stage}"), "the staged copy's own id");
    let (row, over) = r.spawn_row(&copy).expect("the champ writes");
    let want = rpgkit_world::battle::stage_level(d.entry("units", "wolf"), i64::from(stage)) as i32;
    assert_eq!(row.level, want, "grown by the stage rule");
    assert_eq!(over.tuning.power, CHAMP_POWER, "still spawns with its extras");
}

#[test]
fn a_stage_raise_spends_the_extra_points_too() {
    let d = data();
    let stage = 2;
    let a = args(stage);
    let mut r = Runner::new(&d, &a);
    let enc = world::encounter(&mut r, "pit", 1, false, "vale", true).expect("the scratch encounter");
    let (row, _) = r.spawn_row(&enc.enemies[0]).expect("the brute writes");
    let grown = BASE + stage * GROWTH;
    assert_eq!((row.level, row.spend.values().sum::<i32>()), (grown, spend_pool(grown)));
}

#[test]
fn the_stage_key_parses_refuses_bad_values_and_is_in_sim_help() {
    let d = data();
    let game = rpgkit_sim::SimGame::default();
    let argv = |line: &str| line.split_whitespace().map(String::from).collect::<Vec<_>>();
    assert_eq!(SimArgs::parse_for(&d, &argv("party"), &game).map(|a| a.stage), Ok(0), "default 0: authored levels");
    assert_eq!(SimArgs::parse_for(&d, &argv("party stage=3"), &game).map(|a| a.stage), Ok(3));
    for bad in ["-1", "x", "", "99999999999"] {
        let err = SimArgs::parse_for(&d, &argv(&format!("party stage={bad}")), &game).map(|a| a.stage);
        assert!(err.as_ref().is_err_and(|e| e.starts_with("stage must")), "stage={bad}: {err:?}");
    }
    let help = rpgkit_sim::run(&d, &["help".to_string()], &game);
    assert!(help.contains(&rpgkit_sim::args::STAGE_USAGE.to_string()), "sim help lists stage=");
}
