//! RULESET1-G (RULESET1_TASKS 3, 4.1 G): the sim's `feel` mode on a scratch catalog built here (no shipped data). Its
//! defaults come from the tasks (timeline, 10 seeds, the game's encounters); it is deterministic (the same arguments
//! print the same lines) and ends with one FEEL_JSON line; its counts follow the rules: every planned unit-round has
//! one intent, classic has no intents, no poise bars and no breaks; the help text (Godot parity) does not name it.
//! RULESET1-G2 (RULESET1_BALANCE1 3.1, 4.2, 4.3): the feel keys and their refusals, the outcome columns, the tactics,
//! the run's armour, and the `timed` party driver (TD1-TD4).

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

// ---- RULESET1-G2 (RULESET1_BALANCE1 3.1, 4.2, 4.3): the feel keys, the outcome columns, the timed driver ----------

#[test]
fn feel_keys_parse_and_refuse_by_the_spec() {
    let d = data();
    let g = game();
    let a = parse(&d, "feel tactic=guard_counter party=both armor=t_foe:50,t_hero:0 tune=stagger_windup_mult:3,rest_ap_cap:4", &g).unwrap();
    assert_eq!((a.tactic.as_str(), a.tactic_preset()), ("guard_counter", "defensive"), "AI5: the tactic drafts with its PlanPresets id");
    assert_eq!(a.drivers.len(), 2, "party=both runs each driver");
    assert_eq!(a.armor.get("t_foe").copied(), Some(50));
    assert_eq!((a.ruleset.numbers.stagger_windup_mult, a.ruleset.numbers.rest_ap_cap), (3.0, 4), "tune= sets the run's ruleset numbers");
    let header = a.header();
    for part in ["tactic guard_counter", "party both", "armor t_foe:50,t_hero:0", "tune stagger_windup_mult:3,rest_ap_cap:4"] {
        assert!(header.contains(part), "the header prints every non-default key: {part} in {header}");
    }
    assert!(!parse(&d, "feel", &g).unwrap().header().contains("tactic"), "defaults are not printed");
    let err = |line: &str| parse(&d, line, &g).unwrap_err();
    assert!(err("feel tactic=rush").contains("[\"balanced\", \"all_out\", \"focus_weakest\", \"guard_counter\", \"kite\", \"economy\"]"), "an unknown tactic lists the six");
    assert!(err("feel party=manual").starts_with("party must be one of"));
    assert!(err("feel armor=t_nobody:10").contains("in no fought roster"), "armor= names a fought unit");
    assert!(err("feel armor=t_foe:201").starts_with("armor must be"), "armor= is 0-200");
    assert!(err("feel tune=stagger_windup_mult:9").contains("must be"), "tune= stays in the schema range");
    assert!(err("feel tune=poise_stat:1").contains("is not a ruleset tuning number"));
    assert!(err("feel tune=rest_ap_cap:1.5").contains("whole number"));
    assert_eq!(err("feel preset=aggressive"), "preset is classic-only; use tactic=", "3.1: preset= under timeline is an error");
    assert!(parse(&d, "feel preset=aggressive ruleset=classic", &g).is_ok(), "preset= still works under classic");
    assert!(parse(&d, "feel preset=balanced", &g).is_ok(), "balanced is the timeline's one preset");
    assert!(SimArgs::parse_for(&d, &argv("duel tactic=all_out"), &g).unwrap_err().contains("feel key"), "no other mode takes a feel key");
    assert!(!args::usage(&g).iter().any(|l| ["tactic=", "party=", "armor=", "tune="].iter().any(|k| l.contains(k))), "the help text (Godot parity) is unchanged");
}

#[test]
fn feel_outcomes_sum_to_the_fights_and_name_the_driver() {
    let d = data();
    for line in ["feel seeds=3", "feel seeds=3 cap=1", "feel seeds=3 party=timed"] {
        let out = rpgkit_sim::run(&d, &argv(line), &game());
        let r = report(&out);
        for x in ENCOUNTERS.iter().map(|e| &r["encounters"][*e]).chain([&r["total"]]) {
            let sum: i64 = ["won", "lost", "capped", "fled"].iter().map(|k| x[*k].as_i64().unwrap()).sum();
            assert_eq!(json!(sum), x["fights"], "{line}: won + lost + capped + fled = fights ({x})");
            assert!(x["median_rounds_won"].is_number());
        }
        assert!(out.iter().any(|l| l.starts_with("FEEL ") && l.contains("(won ") && l.contains("capped ")), "{line}: the outcome columns");
        assert!(r["driver"].is_string());
    }
    let both = rpgkit_sim::run(&d, &argv("feel seeds=2 party=both"), &game());
    let drivers: Vec<Value> = both.iter().filter_map(|l| l.strip_prefix(JSON_PREFIX)).map(|j| serde_json::from_str::<Value>(j).unwrap()["driver"].clone()).collect();
    assert_eq!(drivers, [json!("auto"), json!("timed")], "party=both prints a block and a FEEL_JSON line per driver");
}

#[test]
fn feel_tactics_change_the_party_intents() {
    let d = data();
    let share = |tactic: &str, intent: &str| {
        let r = report(&rpgkit_sim::run(&d, &argv(&format!("feel enc=t_pair seeds=3 jitter=off tactic={tactic}")), &game()));
        r["total"]["party"]["intent_share_pct"].get(intent).and_then(Value::as_f64).unwrap_or(0.0)
    };
    assert!(share("guard_counter", "guard") > share("all_out", "guard"), "Guard & Counter guards more than All-out Attack");
    assert!(share("all_out", "assault") > share("guard_counter", "assault"), "All-out Attack assaults more than Guard & Counter");
}

#[test]
fn feel_armor_changes_the_run_units_by_the_armour_curve() {
    use rpgkit_rules::damage::{self, HitInput};
    let d = data();
    let a = parse(&d, "feel enc=t_duel seeds=1 armor=t_foe:50", &game()).unwrap();
    let mut r = rpgkit_sim::runner::Runner::new(&d, &a);
    rpgkit_sim::feel::feel_mode(&mut r);
    let armoured = r.build("t_foe").expect("the run's t_foe");
    assert_eq!(damage::armor_rating(&armoured), 50, "armor= sets the run's unit");
    assert_eq!(d.cat.units["t_foe"].armor, -1, "the data is untouched");
    let mut bare = d.cat.units["t_foe"].clone();
    bare.armor = 0;
    let bare = rpgkit_rules::build_unit(&d.cat, &bare, 12);
    let hero = rpgkit_rules::build_unit(&d.cat, &d.cat.units["t_hero"], 12);
    let hit = |t: &rpgkit_rules::Unit, pierce: bool| damage::compute(&hero, t, HitInput { ignore_armor: pierce, include_guard: false, ..HitInput::default() }).final_dmg;
    let curve = (hit(&bare, false) as f64 * (1.0 - damage::armor_reduction(50))).round() as i32;
    assert!((hit(&armoured, false) - curve).abs() <= 1, "a plain hit follows r / (r + 50): {} vs {curve}", hit(&armoured, false));
    assert_eq!(hit(&armoured, true), hit(&bare, true), "a pierced hit ignores the armour");
}

/// TD1-TD4 on the scratch duel played round by round: whenever the timed driver re-plans the hero, its Assault skill
/// lands in a block the enemy's tell shows winding up; with no visible windup the auto plan stands; nothing changes
/// under classic; the same round gives the same plan.
#[test]
fn timed_driver_lands_in_a_visible_windup_or_keeps_the_auto_plan() {
    use rpgkit_battle::grid::V2;
    use rpgkit_battle::session::{EncounterDef, Session};
    use rpgkit_sim::timed;
    let d = data();
    let (mut timed_rounds, mut kept_rounds) = (0, 0);
    for (ruleset, seeds) in [(TIMELINE, 11..19), (CLASSIC, 11..13)] {
        for seed in seeds {
            let mut def = EncounterDef::from_obj(&d.cat.encounters["t_duel"]);
            def.rng_seed = seed;
            def.ruleset = rpgkit_rules::ruleset::Ruleset::of(&d.cat, ruleset).unwrap();
            let mut s = Session::start(d.cat.clone(), def, &indexmap::IndexMap::new());
            // the scripted start: the foe beside the hero, so its round-1 tell is an attack winding up
            let (h, e) = (s.w.allies[0], s.w.enemies[0]);
            let beside = [V2::new(1, 0), V2::new(-1, 0), V2::new(0, 1), V2::new(0, -1)].iter().map(|o| s.w.pos(h) + *o).find(|&p| s.w.grid.is_walkable(p) && !s.w.is_occupied(p));
            s.w.remove(e);
            s.w.place(e, beside.expect("a free tile beside the hero"));
            s.replan_enemies();
            for _ in 0..s.def.max_rounds {
                s.auto_draft("balanced");
                let hero = s.w.allies[0];
                let titles = |s: &Session| -> Vec<String> { s.plan.steps_of(hero).iter().map(|p| p.title(&s.w)).collect() };
                let auto = titles(&s);
                let tells = s.enemy_intents();
                let n = timed::adjust(&mut s);
                let now = titles(&s);
                if ruleset == CLASSIC {
                    assert_eq!((n, now.clone()), (0, auto.clone()), "inert under classic");
                }
                if n == 1 {
                    timed_rounds += 1;
                    let target = match s.w.round_states[&hero].target {
                        Some(rpgkit_battle::round_intent::RoundTarget::Unit(t)) => t,
                        other => panic!("a timed round has a unit target ({other:?})"),
                    };
                    let visible = timed::visible_windup_blocks(tells.iter().find(|r| r.unit == target).expect("the target's tell"));
                    let skills = timed::timed_skills(&s.w, hero);
                    let lands = s.plan.entries_for(&s.w, hero).iter().find(|e| skills.contains(&e.skill)).map(|e| e.end_block(&s.w)).expect("a timed hit");
                    assert!(visible.contains(&lands), "the timed hit lands in a visible windup block ({lands} in {visible:?})");
                } else {
                    kept_rounds += 1;
                    assert_eq!(now, auto, "TD3: no timed hit, the auto plan stands");
                }
                if !s.confirm_plan() {
                    break;
                }
                s.resolve_round();
                if s.combat_over || !s.can_advance_round() {
                    break;
                }
                s.advance_round();
            }
        }
    }
    assert!(timed_rounds > 0 && kept_rounds > 0, "both cases occur ({timed_rounds} timed, {kept_rounds} kept)");
}

/// TD4: the timed feel run is deterministic (the same arguments, the same lines).
#[test]
fn timed_feel_is_deterministic() {
    let d = data();
    let line = argv("feel seeds=3 party=timed");
    assert_eq!(rpgkit_sim::run(&d, &line, &game()), rpgkit_sim::run(&d, &line, &game()));
}
