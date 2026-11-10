//! PR5-26 (FB-191 (f)) on a scratch catalog built here (no shipped data): the sim reaches the Channel rule. `tune=` sets
//! `channel_power`, `channel_ai_edge` and the three `channel_cost` numbers (`channel_cost_1..3`, whole, 0-100) on the
//! run's ruleset; every Channel `tune=` key the refusal names is one `Numbers::set` takes. `counts=on` adds per side
//! the Channel counts: marrow actions and Vents (`RULE_COUNTERS`), Channel rounds, the channel meter's own triggers
//! (its crisis), the rounds by the meter's band and the poise breaks the side took; the sides' breaks add up to the
//! run's breaks and each holder's round ends in one band. PR5-26 follow-up (FB-201): `crises` counts the firings of the
//! meter's crisis rows (a `round_end` row at its max) as they fire, not the other meter triggers.

use rpgkit_sim::args::{self, SimArgs};
use rpgkit_sim::feel::JSON_PREFIX;
use rpgkit_sim::SimGame;
use rpgkit_world::Data;
use serde_json::{json, Value};

const MAX: i32 = 100;

/// Every unit holds Strain (the ruleset's `unit_pools`; decay 10 a round, a crisis at MAX setting it to 50, bands calm
/// and edge) and marks Heavy marrow; Feed (+5) and Vent (-20, gate 50) are the meter's skills.
fn data() -> Data {
    Data::from_values(&catalog(), &json!({}), &json!({}))
}

fn catalog() -> Value {
    let atk = |id: &str, blocks: i32, ap: i32, stagger: i32| json!({"id": id, "name": id, "kind": "ATTACK", "round_intent": "assault", "level": 1, "blocks": blocks, "ap_cost": ap, "power_mult": 1.0, "stagger": stagger, "target_mode": "ENEMY"});
    let unit = |id: &str, side: &str| {
        json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 120, "speed": 12, "attack_range": 1, "skill_cap": 5, "poise": 30, "extra_skills": ["t_heavy", "t_jab"],
            "deck": {"marrow": ["t_heavy"]}})
    };
    let enc = |id: &str, party: &[&str], enemies: &[&str]| json!({"id": id, "name": id, "zone": "stone_arena", "arena": "CLOSED", "max_rounds": 8, "total_blocks": 6, "ap_regen": 4, "unit_ap": 10, "party": party, "enemies": enemies});
    let crisis = json!({"on": "round_end", "when": {"meter": {"id": "strain", "at_least": MAX}}, "limit": "none", "effects": [{"type": "meter_set", "meter": "strain", "value": 50}]});
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
            {"id": "t_feed", "name": "Feed", "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 0, "target_mode": "SELF", "heal_base": 3, "learned_only": true,
                "channel": {"only": true}, "meters": [{"meter": "strain", "delta": 5}]},
            {"id": "t_vent", "name": "Vent", "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 0, "target_mode": "SELF", "learned_only": true,
                "channel": {"only": true}, "meters": [{"meter": "strain", "delta": -20}], "requires_meter": {"meter": "strain", "min": 50}, "hp_cost_pct": 10}
        ],
        "unit_meters": [{"id": "strain", "name": "Strain", "short": "St", "min": 0, "max": MAX, "start": 0, "show": "bar", "skills": ["t_feed", "t_vent"],
            "decay": [{"delta": -10, "unless_gained": true}], "bands": [{"id": "calm", "name": "Calm", "from": 0}, {"id": "edge", "name": "Edge", "from": 50}], "triggers": [crisis]}],
        "units": [unit("t_hero", "party"), unit("t_mate", "party"), unit("t_foe", "enemy"), unit("t_foe_b", "enemy")],
        "encounters": [enc("t_pair", &["t_hero", "t_mate"], &["t_foe", "t_foe_b"])],
        "rulesets": [{"id": "t_channel", "name": "Channel", "rules": ["round_intent", "poise", "ap_cost", "unit_meters", "triggers", "channel"], "basics": ["move", "rest", "strike", "active_guard"],
            "unit_pools": ["strain"], "channel_meter": "strain", "channel_rest": "t_feed"}],
    });
    catalog
}

fn game() -> SimGame {
    SimGame { feel_encounters: vec!["t_pair".to_string()], ..SimGame::default() }
}

fn argv(line: &str) -> Vec<String> {
    line.split_whitespace().map(String::from).collect()
}

fn report(out: &[String]) -> Value {
    let last = out.last().expect("a line");
    serde_json::from_str(last.strip_prefix(JSON_PREFIX).unwrap_or_else(|| panic!("the last line is FEEL_JSON: {last}"))).expect("JSON")
}

#[test]
fn tune_reaches_the_channel_numbers() {
    let d = data();
    let a = SimArgs::parse_for(&d, &argv("feel ruleset=t_channel tune=channel_power:1.5,channel_ai_edge:1.1,channel_cost_1:5,channel_cost_2:10,channel_cost_3:20"), &game()).unwrap();
    let n = &a.ruleset.numbers;
    assert_eq!((n.channel_power, n.channel_ai_edge, n.channel_cost), (1.5, 1.1, [5, 10, 20]));
    let err = |line: &str| SimArgs::parse_for(&d, &argv(line), &game()).unwrap_err();
    assert!(err("feel ruleset=t_channel tune=channel_power:9").contains("must be"), "the schema's range");
    assert!(err("feel ruleset=t_channel tune=channel_cost_2:101").contains("must be"), "0-100");
    assert!(err("feel ruleset=t_channel tune=channel_cost_1:2.5").contains("whole number"));
    // every Channel key the refusal lists is settable
    for key in args::tune_keys().into_iter().filter(|k| k.starts_with("channel_")) {
        let (lo, _) = args::tune_range(&key).expect("a range");
        let mut numbers = a.ruleset.numbers.clone();
        assert!(numbers.set(&key, lo).is_ok(), "{key}");
    }
    assert!(args::tune_keys().iter().any(|k| k == "channel_power") && args::tune_keys().iter().any(|k| k == "channel_cost_3"));
}

#[test]
fn counts_on_reports_the_channel_numbers_per_side() {
    let d = data();
    let out = rpgkit_sim::run(&d, &argv("feel seeds=3 ruleset=t_channel counts=on tune=channel_power:2,channel_ai_edge:1"), &game());
    let r = report(&out);
    let t = &r["total"];
    let side_n = |side: &str, k: &str| t[side]["rule_counts"][k].as_i64().unwrap_or_else(|| panic!("{side} {k}: {}", t[side]["rule_counts"]));
    assert_eq!(side_n("party", "breaks_taken") + side_n("enemy", "breaks_taken"), t["breaks"].as_i64().unwrap(), "the sides' breaks add up");
    let mut channel_rounds = 0;
    for side in ["party", "enemy"] {
        let c = &t[side]["rule_counts"]["channel"];
        let bands: i64 = c["bands"].as_object().unwrap().values().map(|v| v.as_i64().unwrap()).sum();
        let unit_rounds = t[side]["unit_rounds"].as_i64().unwrap_or(i64::MAX);
        assert!(bands > 0 && bands <= unit_rounds, "{side}: each holder's round ends in one band ({bands} of {unit_rounds})");
        let rounds = c["rounds"].as_i64().unwrap();
        assert!(rounds <= bands, "{side}: Channel rounds are holder rounds");
        channel_rounds += rounds;
        let marrow = t[side]["rule_counts"]["counts"]["marrow_actions"].as_i64().unwrap();
        let vents = t[side]["rule_counts"]["counts"]["vents"].as_i64().unwrap();
        assert!(rounds > 0 || (marrow == 0 && vents == 0), "{side}: marrow actions and Vents only in Channel rounds");
        assert!(c["meter_triggers"].as_i64().unwrap() >= 0);
    }
    assert!(channel_rounds > 0, "past an edge of 1 at power 2 the AI Channels: {t}");
    assert!(out.iter().any(|l| l.contains("party rule counts: ") && l.contains("Channel rounds ") && l.contains("breaks taken ")), "the human line");
}

/// PR5-26 follow-up (FB-201): the crises. Strain starts at its max and only Feed is the meter's skill (no Vent), so each
/// holder's first round ends at the max and its crisis row fires (the bands then read the reset value); a second
/// `round_end` row from 50 (not the max) fires too and is a meter trigger, not a crisis. Without `counts=on` no crisis
/// count prints.
#[test]
fn counts_on_counts_the_crises_as_they_fire() {
    let mut cat = catalog();
    let meter = &mut cat["unit_meters"][0];
    meter["start"] = json!(MAX);
    meter["skills"] = json!(["t_feed"]);
    meter["triggers"].as_array_mut().unwrap().push(json!({"on": "round_end", "when": {"meter": {"id": "strain", "at_least": 50}}, "limit": "battle", "effects": [{"type": "meter", "meter": "strain", "delta": -1}]}));
    let d = Data::from_values(&cat, &json!({}), &json!({}));
    let seeds = 3;
    let out = rpgkit_sim::run(&d, &argv(&format!("feel seeds={seeds} ruleset=t_channel counts=on")), &game());
    let t = &report(&out)["total"];
    for side in ["party", "enemy"] {
        let c = &t[side]["rule_counts"]["channel"];
        let (crises, triggers) = (c["crises"].as_i64().unwrap(), c["meter_triggers"].as_i64().unwrap());
        assert!(crises >= seeds, "{side}: each fight's first round ends at the max ({crises}): {c}");
        assert!(crises < triggers, "{side}: the row from 50 is a meter trigger, not a crisis ({crises} of {triggers})");
    }
    assert!(out.iter().any(|l| l.contains("party rule counts: ") && l.contains(", crises ")), "the human line");
    let plain = rpgkit_sim::run(&d, &argv(&format!("feel seeds={seeds} ruleset=t_channel")), &game());
    assert!(!plain.iter().any(|l| l.contains("crises")), "only with counts=on");
}
