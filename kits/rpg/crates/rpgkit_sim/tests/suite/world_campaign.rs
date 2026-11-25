//! CAMPAIGN-SIM1 (ENGINE-28): the world campaign run (`campaign world=<world>`) plays the world's story fights in
//! story order with the party each fields, carrying XP and levels by the kit's rules: levels rise between fights, a
//! member who leaves is no longer fielded, a sequence that unlocks a quest plays before it, side fights come only
//! before quest fights (a repeat paying its binding's percent), and its keys are its own. Scratch world built here.

use rpgkit_battle::session::Roster;
use rpgkit_sim::args::{SimArgs, SIDE_MAX};
use rpgkit_sim::world_campaign::{beats, side_pool, story_order, JSON_PREFIX};
use rpgkit_sim::SimGame;
use rpgkit_world::battle::WorldRoster;
use rpgkit_world::Data;
use serde_json::{json, Value};

const WORLD: &str = "vale";

/// World `vale`: the opening sequence fights `e1` (ash alone); quest `q1` fights `e2` (ash and bo); the sequence
/// `seq_mid` fights `e_mid` and sets the flag quest `q2` needs; `q2` fights `e3` (bo alone: ash has left). One
/// location paces patrols of `roam` (visit cap 1, a repeat paying 50 %). XP: 20 a level.
fn data() -> Data {
    let unit = |id: &str, side: &str, tier: &str, level: i64| json!({"id": id, "name": id, "side": side, "class_id": "drill", "level": level, "tier": tier});
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}
        ],
        "units": [unit("wolf", "enemy", "standard", 3), unit("ash", "party", "hero", 1), unit("bo", "party", "hero", 2)]
    });
    let fight = |id: &str, party: &[&str]| json!({"id": id, "name": id, "party": party, "enemies": ["wolf"], "max_rounds": 2});
    let worlds = json!({
        WORLD: {
            "manifest": {"id": WORLD},
            "own": {
                "characters": [
                    {"id": "wolf", "name": "Wolf", "base_unit": "wolf"},
                    {"id": "ash", "name": "Ash", "base_unit": "ash"},
                    {"id": "bo", "name": "Bo", "base_unit": "bo"}
                ],
                "encounters": [fight("e1", &["ash"]), fight("e2", &["ash", "bo"]), fight("e_mid", &["ash", "bo"]), fight("e3", &["bo"]), fight("roam", &["ash", "bo"])],
                "sequences": [
                    {"id": "new_game", "steps": [{"kind": "vignette"}, {"kind": "fight", "encounter": "e1"}]},
                    {"id": "seq_mid", "steps": [{"kind": "fight", "encounter": "e_mid"}, {"kind": "effects", "effects": [{"effect": "set_flag", "key": "vale.mid", "value": true}]}]}
                ],
                "quests": [
                    {"id": "q1", "track": "main", "steps": [{"encounter": "e2"}]},
                    {"id": "q_side", "track": "side", "steps": [{"encounter": "roam"}]},
                    {"id": "q2", "track": "main", "requires_flags": [{"key": "vale.mid", "value": true}], "steps": [{"encounter": "e3"}]}
                ],
                "locations": [{"id": "wood", "zones": [{"encounters": [["roam", 1]], "visit_cap": 1, "repeat_xp_pct": [50]}]}]
            }
        }
    });
    Data::from_values(&catalog, &worlds, &json!({"xp_curve": {"xp_to_next": [20, 20, 20, 20, 20, 20, 20, 20, 20]}}))
}

fn argv(line: &str) -> Vec<String> {
    line.split_whitespace().map(String::from).collect()
}

fn parse(d: &Data, line: &str) -> Result<SimArgs, String> {
    SimArgs::parse_for(d, &argv(line), &SimGame::default())
}

/// The CAMPAIGN_JSON object of a run (the last line).
fn report(d: &Data, line: &str) -> Value {
    let out = rpgkit_sim::run(d, &argv(line), &SimGame::default());
    let last = out.last().expect("a line");
    serde_json::from_str(last.strip_prefix(JSON_PREFIX).unwrap_or_else(|| panic!("the last line is CAMPAIGN_JSON: {out:?}"))).expect("JSON")
}

/// The story: the opening sequence, then the main quests in order, the sequence that unlocks `q2` just before it; the
/// side quest is no story fight.
#[test]
fn the_story_order_follows_the_opening_the_main_quests_and_their_unlocks() {
    let d = data();
    let order: Vec<(String, bool)> = story_order(&d, WORLD);
    let want = [("e1", false), ("e2", true), ("e_mid", false), ("e3", true)];
    assert_eq!(order, want.map(|(e, q)| (e.to_string(), q)).to_vec());
}

/// Levels rise between fights (the kit's kill XP at each member's own level, `xp_for_level` / `level_for`), a member
/// who leaves is not fielded again, and the same fight at the authored levels is reported beside it.
#[test]
fn levels_rise_between_fights_and_a_leave_removes_the_unit() {
    let d = data();
    let r = report(&d, "campaign world=vale seeds=1 side=0");
    let fights = r["fights"].as_array().expect("fights");
    let ids: Vec<&str> = fights.iter().map(|f| f["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["e1", "e2", "e_mid", "e3"]);
    let lv = |i: usize, m: &str| fights[i]["levels"][m].as_i64();
    assert_eq!((lv(0, "ash"), lv(0, "bo")), (Some(1), None), "bo joins at e2");
    assert!(lv(1, "ash").unwrap() > 1, "ash levelled after e1: {:?}", fights[1]);
    assert_eq!(lv(1, "bo"), Some(2), "bo joins at its authored level");
    assert!(lv(2, "bo").unwrap() > 2 && lv(3, "bo").unwrap() >= lv(2, "bo").unwrap());
    assert_eq!(lv(3, "ash"), None, "ash has left: not fielded in e3");
    assert!(fights.iter().all(|f| f["authored"]["won_pct"].is_number()));
    assert!(r["end_levels"]["bo"].as_i64().unwrap() >= lv(3, "bo").unwrap());
}

/// The levels reach the fight: an ally named in `ally_levels` is raised to that level (never lowered), a foe never.
#[test]
fn the_ally_levels_raise_the_named_allies() {
    let d = data();
    let cat = d.cat.clone();
    let mut at = WorldRoster { allies: vec!["ash".into(), "bo".into()], ..WorldRoster::new(&d, WORLD) };
    at.ally_levels.insert("ash".into(), 4);
    at.ally_levels.insert("bo".into(), 1);
    at.ally_levels.insert("wolf".into(), 9);
    assert_eq!(at.entry(&cat, "ash").map(|u| u.level), Some(4));
    assert_eq!(at.entry(&cat, "bo").map(|u| u.level), Some(2), "never below the authored level");
    assert_eq!(at.entry(&cat, "wolf").map(|u| u.level), Some(3), "a foe is no ally");
}

/// Side fights come only before quest fights, from the paced bindings (the default count their visit cap); a
/// repeat pays the binding's percent.
#[test]
fn side_fights_come_before_quest_fights_and_repeats_pay_less() {
    let d = data();
    let (pool, cap) = side_pool(&d, WORLD);
    assert_eq!((pool.keys().cloned().collect::<Vec<_>>(), cap), (vec!["roam".to_string()], 1));
    let plan = beats(&story_order(&d, WORLD), &pool, 1);
    let got: Vec<(&str, bool, i64)> = plan.iter().map(|b| (b.id.as_str(), b.side, b.xp_pct)).collect();
    assert_eq!(got, [("e1", false, 100), ("roam", true, 100), ("e2", false, 100), ("e_mid", false, 100), ("roam", true, 50), ("e3", false, 100)]);
    let r = report(&d, "campaign world=vale seeds=1");
    assert_eq!((r["side"].as_i64(), r["fights"].as_array().unwrap().len()), (Some(1), 6), "side defaults to the visit cap");
}

/// `story=` replaces the derived order; the keys belong to `campaign world=` and keep their ranges.
#[test]
fn the_keys_are_the_world_campaigns() {
    let d = data();
    let r = report(&d, "campaign world=vale seeds=1 story=e3,e1");
    let ids: Vec<&str> = r["fights"].as_array().unwrap().iter().filter(|f| f["kind"] == "story").map(|f| f["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["e3", "e1"]);
    let kinds: Vec<&str> = r["fights"].as_array().unwrap().iter().map(|f| f["kind"].as_str().unwrap()).collect();
    assert_eq!(kinds, ["side", "story", "story"], "a side fight before the quest fight e3, none before the sequence fight e1");
    let a = parse(&d, "campaign world=vale side=3").expect("parses");
    assert!(a.world_campaign() && a.side == Some(3));
    assert!(parse(&d, &format!("campaign world=vale side={}", SIDE_MAX + 1)).is_err());
    assert!(parse(&d, "campaign world=vale story=nope").is_err(), "an encounter of the world");
    assert!(parse(&d, "campaign side=1").is_err(), "needs world=");
    assert!(parse(&d, "feel enc=world:vale/e1 story=e1").is_err(), "a campaign key only");
}
