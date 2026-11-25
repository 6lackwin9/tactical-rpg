//! WORLD-SIM1 (PR5-42): `sim feel` on a world encounter (`world:<world>/<id>`, `enc=` or a positional reference, or
//! `world=` with a local id) fights it with its authored side as the world builds it
//! (`rpgkit_world::battle::authored_fight`, shared with the host's `new` "<world>:<id>"): the encounter's `party` and
//! `guests` as the world's characters at their authored levels (never staged), by the world's ruleset unless `ruleset=`
//! picks one, within the encounter's round limit. The refusals (no authored party, encounters with different default
//! rulesets, `armor=` on a world fight), the determinism, and the `counts=on` hook counts (CAST1 T10-T14), which change
//! no other line. Checked on a scratch catalog and world built here (no shipped data).

use rpgkit_battle::session::Session;
use rpgkit_rules::ruleset::TIMELINE;
use rpgkit_sim::args::SimArgs;
use rpgkit_sim::feel::{HOOK_COUNTERS, JSON_PREFIX};
use rpgkit_sim::runner::Runner;
use rpgkit_sim::SimGame;
use rpgkit_world::Data;
use serde_json::{json, Value};

const WORLD: &str = "vale";
/// The world's own ruleset row (the same rules as the timeline row under another id).
const WORLD_RULES: &str = "vale_rules";
const FIGHT: &str = "ambush";
const REF: &str = "world:vale/ambush";
/// Units (class-built, so the level reaches the stats): two heroes, a guest and a foe that grow with the stage
/// (`growth_per_stage`).
const GROWTH: i64 = 1;

fn data() -> Data {
    let atk = |id: &str, blocks: i32, ap: i32, stagger: i32| json!({"id": id, "name": id, "kind": "ATTACK", "round_intent": "assault", "level": 1, "blocks": blocks, "ap_cost": ap, "power_mult": 1.0, "stagger": stagger, "target_mode": "ENEMY"});
    let unit = |id: &str, side: &str, tier: &str, level: i64| {
        json!({"id": id, "name": id, "side": side, "class_id": "drill", "tier": tier, "level": level, "growth_per_stage": GROWTH, "attack_range": 1, "skill_cap": 5, "poise": 30, "extra_skills": ["t_heavy", "t_jab"]})
    };
    let rules = ["round_intent", "enemy_moves", "poise", "ap_cost", "flank"];
    let basics = ["move", "rest", "strike", "active_guard"];
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY", "round_intent": "assault"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
            {"id": "active_guard", "name": "Active Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 1, "mitigation": 0.5},
            atk("t_heavy", 2, 3, 12),
            atk("t_jab", 1, 1, 4),
        ],
        "units": [unit("t_hero", "party", "hero", 3), unit("t_mate", "party", "hero", 4), unit("t_guest", "party", "standard", 2), unit("t_foe", "enemy", "standard", 2)],
        "encounters": [{"id": "t_duel", "name": "t_duel", "zone": "stone_arena", "arena": "CLOSED", "max_rounds": 6, "total_blocks": 6, "ap_regen": 4, "unit_ap": 10, "party": ["t_hero"], "enemies": ["t_foe"]}],
        "rulesets": [
            {"id": TIMELINE, "name": "Timeline", "rules": rules, "basics": basics},
            {"id": WORLD_RULES, "name": "Vale", "rules": rules, "basics": basics},
        ],
    });
    let fight = |id: &str, party: &[&str], guests: &[&str]| {
        json!({"id": id, "name": format!("The {id}"), "zone": "stone_arena", "arena": "CLOSED", "max_rounds": 5, "total_blocks": 6, "ap_regen": 4, "unit_ap": 10, "party": party, "guests": guests, "enemies": ["foe", "foe"]})
    };
    let worlds = json!({
        WORLD: {
            "manifest": {"id": WORLD, "ruleset": WORLD_RULES},
            "own": {
                "characters": [
                    {"id": "hero", "name": "Hero", "role": "party", "base_unit": "t_hero", "default_intent": "guard"},
                    {"id": "mate", "name": "Mate", "role": "party", "base_unit": "t_mate"},
                    {"id": "guest", "name": "Guest", "role": "npc", "base_unit": "t_guest"},
                    {"id": "foe", "name": "Foe", "role": "enemy", "base_unit": "t_foe"}
                ],
                "encounters": [fight(FIGHT, &["hero", "mate"], &["guest"]), fight("no_party", &[], &[])]
            }
        }
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn argv(line: &str) -> Vec<String> {
    line.split_whitespace().map(String::from).collect()
}

fn parse(d: &Data, line: &str) -> Result<SimArgs, String> {
    SimArgs::parse_for(d, &argv(line), &SimGame::default())
}

fn run(d: &Data, line: &str) -> Vec<String> {
    rpgkit_sim::run(d, &argv(line), &SimGame::default())
}

/// The FEEL_JSON object of a run (the last line).
fn report(out: &[String]) -> Value {
    let last = out.last().expect("a line");
    serde_json::from_str(last.strip_prefix(JSON_PREFIX).unwrap_or_else(|| panic!("the last line is FEEL_JSON: {last}"))).expect("JSON")
}

/// The encounter row as the world resolves it.
fn row(d: &Data, id: &str) -> serde_json::Map<String, Value> {
    d.resolve(WORLD, "encounters", id).clone()
}

/// The authored side of `FIGHT`: (character name, its base unit's level), the party then the guests.
fn authored_side(d: &Data) -> Vec<(String, i32)> {
    let enc = row(d, FIGHT);
    aegis_core::json::sget(&enc, "party")
        .into_iter()
        .chain(aegis_core::json::sget(&enc, "guests"))
        .map(|id| {
            let c = d.resolve(WORLD, "characters", &id);
            let level = aegis_core::json::i(d.entry("units", &aegis_core::json::s(c, "base_unit")), "level", 1) as i32;
            (aegis_core::json::s(c, "name"), level)
        })
        .collect()
}

/// The started battle of `FIGHT` at `stage`: (name, level) of each ally, and of each enemy.
fn sides(d: &Data, stage: i64) -> (Vec<(String, i32)>, Vec<(String, i32)>) {
    let (def, roster) = rpgkit_world::battle::authored_fight(d, WORLD, FIGHT, stage).expect("the scratch encounter");
    let s = Session::start(d.cat.clone(), def, &roster);
    let of = |ids: &[usize]| ids.iter().map(|&c| (s.w.units[c].name.clone(), s.w.units[c].stats.level)).collect::<Vec<_>>();
    (of(&s.w.allies), of(&s.w.enemies))
}

#[test]
fn a_world_reference_resolves_in_every_form_and_plays_by_the_worlds_ruleset() {
    let d = data();
    for line in [format!("feel {REF}"), format!("feel enc={REF}"), format!("feel world={WORLD} enc={FIGHT}")] {
        let a = parse(&d, &line).unwrap_or_else(|e| panic!("{line}: {e}"));
        assert_eq!(a.encounters, [REF], "{line}");
        assert_eq!(a.ruleset.id, WORLD_RULES, "{line}: the world's ruleset, not the feel mode's catalog default");
    }
    // a positional reference follows an `enc=` list; ruleset= picks the rules over the world's
    let a = parse(&d, &format!("feel enc=t_duel {REF} ruleset={TIMELINE}")).expect("parses");
    assert_eq!(a.encounters, ["t_duel", REF]);
    assert_eq!(a.ruleset.id, TIMELINE);
    // a catalog encounter keeps the timeline default
    assert_eq!(parse(&d, "feel enc=t_duel").expect("parses").ruleset.id, TIMELINE);
}

#[test]
fn a_world_feel_run_is_refused_without_an_authored_party_mixed_defaults_or_armor() {
    let d = data();
    let err = parse(&d, "feel world:vale/no_party").expect_err("no authored party");
    assert!(err.contains("no authored party"), "{err}");
    let err = parse(&d, &format!("feel enc=t_duel {REF}")).expect_err("two default rulesets");
    assert!(err.contains(TIMELINE) && err.contains(WORLD_RULES) && err.contains("ruleset="), "{err}");
    let err = parse(&d, &format!("feel {REF} armor=t_foe:50")).expect_err("armor= on a world fight");
    assert!(err.contains("armor"), "{err}");
    let err = parse(&d, "feel world:vale/nowhere").expect_err("unknown");
    assert!(err.contains("unknown world encounter"), "{err}");
}

#[test]
fn the_authored_side_fights_at_its_authored_levels_and_never_grows_with_the_stage() {
    let d = data();
    let side = authored_side(&d);
    let (allies, enemies) = sides(&d, 0);
    assert_eq!(allies, side, "the encounter's party then its guests, as the world's characters at their base levels");
    let base_foe = enemies[0].1;
    let (allies, enemies) = sides(&d, 2);
    assert_eq!(allies, side, "the party and the guest (a standard tier that grows) never stage");
    assert!(enemies.iter().all(|(_, l)| i64::from(*l) == i64::from(base_foe) + 2 * GROWTH), "{enemies:?}");
    // the sim's fight is that battle: its party side and the encounter's round limit
    let a = parse(&d, &format!("feel {REF} seeds=1")).expect("parses");
    let mut r = Runner::new(&d, &a);
    r.rounds_override = aegis_core::json::i(&row(&d, FIGHT), "max_rounds", 0) as i32;
    let res = r.fight_world(WORLD, FIGHT, 11);
    assert!(res.ok);
    assert_eq!(res.party_size as usize, side.len());
    assert_eq!(res.party_max.keys().cloned().collect::<Vec<_>>(), side.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>());
    assert!(res.rounds <= r.rounds_override, "within the encounter's max_rounds");
}

#[test]
fn a_world_feel_run_reports_every_seed_deterministically() {
    let d = data();
    let line = format!("feel {REF} seeds=3");
    let out = run(&d, &line);
    assert_eq!(out, run(&d, &line), "the same arguments print the same lines");
    assert!(!out.iter().any(|l| l.starts_with("SIM ")), "{out:?}");
    let j = report(&out);
    assert_eq!(j["ruleset"], json!(WORLD_RULES));
    assert_eq!(j["encounters"][REF]["fights"], json!(3));
    assert!(out.iter().any(|l| l.starts_with(&format!("FEEL {REF} (The {FIGHT}): 3 fights"))), "{out:?}");
    let max = aegis_core::json::i(&row(&d, FIGHT), "max_rounds", 0);
    assert!(j["encounters"][REF]["median_rounds"].as_f64().unwrap() <= max as f64);
}

#[test]
fn counts_on_adds_the_hook_counts_and_changes_no_other_line() {
    let d = data();
    let plain = run(&d, &format!("feel {REF} seeds=2"));
    let counted = run(&d, &format!("feel {REF} seeds=2 counts=on"));
    let kept: Vec<&String> = counted.iter().filter(|l| !l.contains(" rule counts: ") && !l.contains(" hooks: ") && !l.starts_with("ARGS") && !l.starts_with(JSON_PREFIX)).collect();
    let base: Vec<&String> = plain.iter().filter(|l| !l.starts_with("ARGS") && !l.starts_with(JSON_PREFIX)).collect();
    assert_eq!(kept, base, "asking for the counts changes no fight");
    // per encounter and in total: a hooks line per side and one of the fight tallies, every hook key per side
    assert_eq!(counted.iter().filter(|l| l.starts_with("  hooks: arena strikes ")).count(), 2);
    assert_eq!(counted.iter().filter(|l| l.starts_with("  party hooks: ") || l.starts_with("  enemy hooks: ")).count(), 4);
    let j = report(&counted);
    let hooks = &j["total"]["hooks"];
    for key in ["arena_strikes", "windup_breaks_by_skill", "phases_fired", "fights_with"] {
        assert!(hooks.get(key).is_some(), "{key}: {hooks}");
    }
    for side in ["party", "enemy"] {
        let h = &j["total"][side]["rule_counts"]["hooks"];
        assert_eq!(h.as_object().unwrap().len(), HOOK_COUNTERS.len(), "{side}: {h}");
        // every planned unit-round of the side is one unit's intent
        let by_unit: i64 = j["total"][side]["rule_counts"]["intents_by_unit"].as_object().unwrap().values().flat_map(|m| m.as_object().unwrap().values()).map(|v| v.as_i64().unwrap()).sum();
        assert_eq!(by_unit, j["total"][side]["unit_rounds"].as_i64().unwrap(), "{side}");
    }
    let names: Vec<String> = authored_side(&d).into_iter().map(|(n, _)| n).collect();
    let party_units: Vec<String> = j["total"]["party"]["rule_counts"]["intents_by_unit"].as_object().unwrap().keys().cloned().collect();
    assert_eq!(party_units, names, "the authored side's units, by name");
    // fights_with counts fights, so never more than the fights; the windup breaks by skill sum to the windup breaks
    let fights = j["total"]["fights"].as_i64().unwrap();
    assert!(hooks["fights_with"].as_object().unwrap().values().all(|v| v.as_i64().unwrap() <= fights), "{hooks}");
    let by_skill: i64 = hooks["windup_breaks_by_skill"].as_object().unwrap().values().map(|v| v.as_i64().unwrap()).sum();
    assert_eq!(by_skill, j["total"]["windup_breaks"].as_i64().unwrap());
}
