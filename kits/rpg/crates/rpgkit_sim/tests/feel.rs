//! RULESET1-G (RULESET1_TASKS 3, 4.1 G): the sim's `feel` mode on a scratch catalog built here (no shipped data). Its
//! defaults come from the tasks (timeline, 10 seeds, the game's encounters); it is deterministic (the same arguments
//! print the same lines) and ends with one FEEL_JSON line; its counts follow the rules: every planned unit-round has
//! one intent, classic has no intents, no poise bars and no breaks; the help text (Godot parity) does not name it.

use rpgkit_rules::ruleset::{CLASSIC, TIMELINE};
use rpgkit_sim::args::{self, SimArgs};
use rpgkit_sim::feel::{FEEL_SEEDS, JSON_PREFIX};
use rpgkit_sim::SimGame;
use rpgkit_world::Data;
use serde_json::{json, Value};

const ENCOUNTERS: [&str; 2] = ["t_duel", "t_pair"];

fn data() -> Data {
    let atk = |id: &str, blocks: i32, ap: i32, stagger: i32| json!({"id": id, "name": id, "kind": "ATTACK", "round_intent": "assault", "level": 1, "blocks": blocks, "ap_cost": ap, "power_mult": 1.0, "stagger": stagger, "target_mode": "ENEMY"});
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 90, "speed": 12, "attack_range": 1, "skill_cap": 5, "poise": 30, "extra_skills": ["t_heavy", "t_jab"]});
    let enc = |id: &str, party: &[&str], enemies: &[&str]| json!({"id": id, "name": id, "zone": "stone_arena", "arena": "CLOSED", "max_rounds": 6, "total_blocks": 6, "ap_regen": 4, "unit_ap": 10, "party": party, "enemies": enemies});
    let catalog = json!({
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY", "round_intent": "assault"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3},
            {"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
            {"id": "active_guard", "name": "Active Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 1, "mitigation": 0.5},
            atk("t_heavy", 2, 3, 12),
            atk("t_jab", 1, 1, 4),
        ],
        "units": [unit("t_hero", "party"), unit("t_mate", "party"), unit("t_foe", "enemy"), unit("t_foe_b", "enemy")],
        "encounters": [enc("t_duel", &["t_hero"], &["t_foe"]), enc("t_pair", &["t_hero", "t_mate"], &["t_foe", "t_foe_b"]), enc("t_empty", &[], &["t_foe"])],
        "rulesets": [{"id": TIMELINE, "name": "Timeline", "rules": ["round_intent", "enemy_moves", "poise", "ap_cost", "flank"], "basics": ["move", "rest", "strike", "active_guard"]}],
    });
    Data::from_values(&catalog, &json!({}), &json!({}))
}

fn game() -> SimGame {
    SimGame { feel_encounters: ENCOUNTERS.iter().map(|e| e.to_string()).collect(), ..SimGame::default() }
}

fn argv(line: &str) -> Vec<String> {
    line.split_whitespace().map(String::from).collect()
}

fn parse(d: &Data, line: &str, g: &SimGame) -> Result<SimArgs, String> {
    SimArgs::parse_for(d, &argv(line), g)
}

/// The FEEL_JSON object of a run (the last line).
fn report(out: &[String]) -> Value {
    let last = out.last().expect("a line");
    serde_json::from_str(last.strip_prefix(JSON_PREFIX).unwrap_or_else(|| panic!("the last line is FEEL_JSON: {last}"))).expect("JSON")
}

#[test]
fn feel_defaults_to_timeline_ten_seeds_and_the_games_encounters() {
    let d = data();
    let a = parse(&d, "feel", &game()).unwrap();
    assert_eq!((a.ruleset.id.as_str(), a.seed_count, a.encounters.clone()), (TIMELINE, FEEL_SEEDS, ENCOUNTERS.iter().map(|e| e.to_string()).collect::<Vec<_>>()));
    let b = parse(&d, "feel enc=t_duel seeds=2 ruleset=classic", &game()).unwrap();
    assert_eq!((b.ruleset.id.as_str(), b.seed_count, b.encounters.clone()), (CLASSIC, 2, vec!["t_duel".to_string()]));
    assert_eq!(parse(&d, "feel 5 3 seeds=7", &game()).unwrap().seed_count, 3, "the positional seeds win");
    assert!(parse(&d, "feel", &SimGame::default()).unwrap_err().contains("feel needs enc="), "a game without feel encounters");
    assert_eq!(parse(&d, "feel enc=t_empty", &game()).unwrap_err(), "feel: 't_empty' is not a catalog encounter with a party");
    assert!(!args::usage(&game()).iter().any(|l| l.contains("feel") || l.contains("seeds=")), "the help text (Godot parity) is unchanged");
}

#[test]
fn feel_is_deterministic_and_ends_with_its_json() {
    let d = data();
    let line = argv("feel seeds=2");
    let out = rpgkit_sim::run(&d, &line, &game());
    assert_eq!(out, rpgkit_sim::run(&d, &line, &game()), "the same arguments, the same lines");
    let r = report(&out);
    assert_eq!(r["ruleset"], json!(TIMELINE));
    assert_eq!(r["total"]["fights"], json!(2 * ENCOUNTERS.len()), "a fight per seed and encounter");
    for e in ENCOUNTERS {
        let x = &r["encounters"][e];
        assert_eq!(x["fights"], json!(2), "{e}");
        assert!(out.iter().any(|l| l.starts_with(&format!("FEEL {e} "))), "{e}: its block");
        for side in ["party", "enemy"] {
            let share: f64 = x[side]["intent_share_pct"].as_object().unwrap().values().map(|v| v.as_f64().unwrap()).sum();
            assert!((share - 100.0).abs() < 0.5, "{e} {side}: every unit-round has one intent ({share})");
            assert!(x[side]["intent_share_pct"].get("none").is_none(), "{e} {side}: the round intent always picks one");
        }
        assert_eq!(x["enemy"]["ap_per_round"], json!(0.0), "{e}: enemy_moves: the enemies pay no AP (T11)");
        assert_eq!(x["enemy"]["rest_ap"], json!(0), "{e}: and gain none from Rest");
    }
}

#[test]
fn classic_feel_has_no_intents_no_bars_and_no_breaks() {
    let d = data();
    let out = rpgkit_sim::run(&d, &argv("feel seeds=2 ruleset=classic"), &game());
    let t = &report(&out)["total"];
    assert_eq!((t["breaks"].clone(), t["windup_hits"].clone(), t["chance_rolls"].clone()), (json!(0), json!(0), json!(0)), "no poise in classic");
    assert_eq!(t["lock_rate_pct"], json!(0.0));
    for side in ["party", "enemy"] {
        let share = t[side]["intent_share_pct"].as_object().unwrap();
        assert_eq!(share.keys().collect::<Vec<_>>(), ["none"], "{side}: classic has no round intents");
        assert_eq!(t[side]["rest_ap"], json!(0), "{side}: no Rest gain at the round's end (RS2 is timeline's)");
    }
}
