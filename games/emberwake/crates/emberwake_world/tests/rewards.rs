//! REWARD1: rewards per defeated enemy (games/emberwake/docs/specs/REWARD1.md section 11). Scratch copies of the shipped
//! data get the purses, loot tables, key loot and objective rewards a case needs (written through the content store).
//! ENCOUNTER2-W: a won hold escape read by the world (the stay flag, the holder captured, the save round trip).

use aegis_core::json::{self, Dict};
use emberwake_content::store;
use emberwake_world::{battle, save, Data, GameState};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn data_dir() -> PathBuf {
    Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let p = e.path();
        if p.is_dir() {
            copy_dir(&p, &to.join(e.file_name()));
        } else if p.extension().is_some_and(|x| x == "json") {
            std::fs::copy(&p, to.join(e.file_name())).unwrap();
        }
    }
}

/// A scratch copy of the shipped data; `edits` are (world or None, kind, id, {field: value}) merged into entries.
use aegis_core::scratch::Scratch;

fn scratch(name: &str, edits: &[(Option<&str>, &str, &str, Value)]) -> (Scratch, Data) {
    let tmp = Scratch::vacant(&format!("rewards_{name}"));
    for sub in ["catalog", "worlds", "maps", "quests", "dialogue", "progression"] {
        copy_dir(&data_dir().join(sub), &tmp.join(sub));
    }
    for (world, kind, id, fields) in edits {
        let d = Data::open(&tmp).unwrap().0;
        let mut e: Dict = match world {
            Some(w) => {
                let hit = d.resolve(w, kind, id);
                hit.iter().filter(|(k, _)| !k.starts_with('_')).map(|(k, v)| (k.clone(), v.clone())).collect()
            }
            None => d.entry(kind, id).clone(),
        };
        if e.is_empty() {
            e.insert("id".into(), json!(id));
        }
        for (k, v) in fields.as_object().unwrap() {
            e.insert(k.clone(), v.clone());
        }
        store::upsert(&tmp, *world, kind, &e, None).unwrap_or_else(|err| panic!("{kind} {id}: {err:?}"));
    }
    let d = Data::open(&tmp).unwrap().0;
    (tmp, d)
}

fn envelope(gs: &GameState, version: i64) -> Dict {
    json::obj([("format", json!(save::FORMAT)), ("version", json!(version)), ("saved_at", json!(0)), ("game_state", gs.to_value())])
}

#[test]
fn the_roster_keys_each_ref_by_its_earlier_entries_and_weighs_kill_xp() {
    // the reinforcement wave as an arrival phase (PHASEFOLD), the first phase: its units follow the enemies
    let arrival = emberwake_battle::encounter::arrival_phase(&["ash_brawler".to_string()], 1, 4, emberwake_battle::encounter::ARRIVAL_LABEL);
    let (_s, d) = scratch("roster", &[(Some("ashlands"), "encounters", "ash_raiders", json!({"phases": [arrival]}))]);
    let r = battle::roster(&d, "ashlands", "ashlands:ash_raiders");
    let keys: Vec<&str> = r.iter().map(|f| f.key.as_str()).collect();
    assert_eq!(keys, ["ash_brawler#0", "ash_skirmisher#0", "ash_shaman#0", "ash_brawler#1"]);
    assert!(r.iter().all(|f| f.tier == "minion" && f.level == 3 && f.weight == 1250), "minion L3 weighs 1250: {r:?}");
    let patrol = battle::roster(&d, "ashlands", "ashlands:ash_patrol");
    assert_eq!(patrol.iter().map(|f| f.weight).collect::<Vec<_>>(), [3000, 1250], "the warchief (standard L4) and a minion");
}

#[test]
fn a_pre_reward_save_is_refused_and_a_current_one_keeps_reward_paid() {
    // CLN-47: the development-era REWARD1 step (v11 -> v12, rebuilding reward_paid from purses) is gone; such a save
    // is refused, and a current save carries its reward_paid as written
    let d = Data::open(&data_dir()).unwrap().0;
    let mut gs = emberwake_world::new_game::create_for_world(&d, "ashlands", 7).expect("a new game");
    assert_eq!(save::migrate(&d, &envelope(&gs, 11)).err(), Some(vec![save::older_build_message(11)]));
    gs.purses = json::obj([("ashlands:ash_patrol", json!(1))]);
    gs.reward_paid = json::obj([("ashlands:ash_patrol", json!(["objective"]))]);
    let (out, warnings) = save::migrate(&d, &envelope(&gs, save::VERSION)).expect("a current save reads");
    assert!(warnings.is_empty(), "{warnings:?}");
    let back = GameState::from_dict(&d, out["game_state"].as_object().unwrap());
    assert_eq!(back.reward_paid, gs.reward_paid);
    gs.reward_paid = Dict::new();
    let (out, _) = save::migrate(&d, &envelope(&gs, save::VERSION)).unwrap();
    assert!(GameState::from_dict(&d, out["game_state"].as_object().unwrap()).reward_paid.is_empty());
}

#[test]
fn reward_paid_round_trips_and_a_bad_one_is_a_load_problem() {
    let d = Data::open(&data_dir()).unwrap().0;
    let mut gs = emberwake_world::new_game::create_for_world(&d, "ashlands", 3).unwrap();
    gs.reward_paid = json::obj([("ashlands:ash_patrol", json!(["ash_warchief#0", "objective"]))]);
    let back = GameState::from_dict(&d, gs.to_value().as_object().unwrap());
    assert_eq!(back.reward_paid, gs.reward_paid);
    let mut bad = gs.to_value();
    bad["reward_paid"] = json!({"ashlands:ash_patrol": [3]});
    let loaded = GameState::from_dict(&d, bad.as_object().unwrap());
    assert!(loaded.validate(&d).contains(&"reward_paid must map encounter ids to lists of keys.".to_string()), "{:?}", loaded.validate(&d));
    bad["reward_paid"] = json!([]);
    let loaded = GameState::from_dict(&d, bad.as_object().unwrap());
    assert!(loaded.validate(&d).contains(&"reward_paid must map encounter ids to lists of keys.".to_string()));
}

// ---- commit 2: the rewards ------------------------------------------------------------------------------------

use emberwake_battle::session::{EncounterDef, Fate, Session};
use emberwake_world::battle::{Foe, Tally, WorldRoster};
use emberwake_world::{explore, new_game, progression};

fn session(d: &Data, encounter: &str, enemies: Option<Value>) -> Session {
    let mut entry = d.resolve("ashlands", "encounters", encounter).clone();
    if let Some(e) = enemies {
        entry.insert("enemies".into(), e);
    }
    let roster = WorldRoster { d, world: "ashlands".into() };
    Session::start_with(d.cat.clone(), EncounterDef::from_obj(&entry), &roster, Vec::new())
}

#[test]
fn fates_read_killed_surrendered_fled_and_standing() {
    let d = Data::open(&data_dir()).unwrap().0;
    let mut s = session(&d, "ash_raiders", None);
    assert!(!s.morale.groups.is_empty() || s.morale.on, "ash_raiders fights with morale");
    let e: Vec<usize> = s.w.enemies.clone();
    s.w.units[e[0]].hp = 0;
    s.w.units[e[1]].is_withdrawn = true;
    s.morale.surrendered.push("ash_skirmisher#0".into());
    s.w.units[e[2]].is_withdrawn = true;
    let fates: Vec<(String, Fate)> = s.fates();
    assert_eq!(fates, [("ash_brawler".to_string(), Fate::Killed), ("ash_skirmisher".to_string(), Fate::Surrendered), ("ash_shaman".to_string(), Fate::Fled)]);

    let ambush = session(&d, "pass_ambush", None);
    let fates = ambush.fates();
    assert_eq!(fates.len(), 4, "the reinforcements are on the list before they come");
    assert!(fates[2..].iter().all(|f| f.1 == Fate::Standing), "a wave that never entered stands: {fates:?}");

    let twice = session(&d, "ash_straggler", Some(json!(["ash_brawler", "ash_brawler"])));
    let keys: Vec<String> = Tally::from_session(&twice).foes.into_iter().map(|f| f.key).collect();
    assert_eq!(keys, ["ash_brawler#0", "ash_brawler#1"]);
}

/// A tally of `qid`'s roster: `fate(key)` per roster foe.
fn tally(d: &Data, qid: &str, fate: &dyn Fn(&str) -> Fate) -> Tally {
    Tally { foes: battle::roster(d, "ashlands", qid).into_iter().map(|f| Foe { fate: fate(&f.key), key: f.key, unit_ref: f.unit_ref }).collect(), ..Default::default() }
}

fn game(d: &Data) -> GameState {
    new_game::create_for_world(d, "ashlands", 1).expect("a new game")
}

#[test]
fn without_a_tally_a_win_pays_the_old_xp_and_a_loss_or_flight_nothing() {
    let d = Data::open(&data_dir()).unwrap().0;
    for e in d.own_entries("ashlands", "encounters") {
        let qid = json::s(e, "_qid");
        let mut gs = game(&d);
        let level = gs.party.members[0].level(&d);
        let want = progression::encounter_xp(&d, &battle::enemy_levels(&d, "ashlands", &qid), level);
        let won = battle::settle(&d, &mut gs, &qid, "won", None);
        assert_eq!(json::i(json::dget(json::dget(&won, "xp"), &gs.party.members[0].id), "xp", -1), want, "{qid}");
        for outcome in ["lost", "fled"] {
            let mut gs = game(&d);
            let before = gs.to_value();
            let res = battle::settle(&d, &mut gs, &qid, outcome, None);
            assert_eq!(res["xp"], json!({}), "{qid} {outcome}");
            assert_eq!(gs.to_value(), before, "{qid} {outcome} leaves the state as it was");
        }
    }
}

#[test]
fn a_fled_fight_pays_the_xp_of_the_foes_it_defeated() {
    let d = Data::open(&data_dir()).unwrap().0;
    let mut gs = game(&d);
    let t = tally(&d, "ashlands:ash_scouts", &|k| if k == "ash_skirmisher#0" { Fate::Killed } else { Fate::Standing });
    let res = battle::apply(&d, &mut gs, "ashlands:ash_scouts", "fled", &t);
    let m = &gs.party.members[0];
    let want = progression::kill_xp(&d, 3, "minion", m.level(&d));
    assert_eq!(json::i(json::dget(json::dget(&res, "xp"), &m.id), "xp", -1), want);
    if m.level(&d) == 5 {
        assert_eq!(want, 9, "a minion L3 gives an L5 hero 9 XP (spec section 3)");
    }
    let none = battle::apply(&d, &mut game(&d), "ashlands:ash_scouts", "fled", &tally(&d, "ashlands:ash_scouts", &|_| Fate::Standing));
    assert_eq!(none["xp"], json!({}));
}

fn purse(g: i64, once: bool) -> Value {
    json!({"purse": {"gold_min": g, "gold_max": g, "once": once}})
}

#[test]
fn the_purse_splits_by_kill_weight_over_the_whole_roster() {
    let (_s, d) = scratch("shares", &[
        (Some("ashlands"), "encounters", "ash_patrol", purse(15, false)),
        (Some("ashlands"), "encounters", "ash_raiders", purse(7, false)),
        (Some("ashlands"), "encounters", "pass_ambush", purse(28, false)),
        (Some("ashlands"), "encounters", "ashmaw_stand", purse(55, false)),
    ]);
    let gold = |qid: &str, outcome: &str, fate: &dyn Fn(&str) -> Fate| {
        let mut gs = game(&d);
        let res = battle::apply(&d, &mut gs, qid, outcome, &tally(&d, qid, fate));
        json::i(json::dget(&res, "rewards"), "gold", -1)
    };
    let kill = |keys: &'static [&'static str]| move |k: &str| if keys.contains(&k) { Fate::Killed } else { Fate::Standing };
    assert_eq!(gold("ashlands:ash_patrol", "won", &kill(&["ash_warchief#0", "ash_berserker#0"])), 15, "a full clear pays the roll");
    assert_eq!(gold("ashlands:ash_patrol", "fled", &kill(&["ash_berserker#0"])), 4, "floor(15 x 1250 / 4250)");
    assert_eq!(gold("ashlands:ash_patrol", "lost", &kill(&["ash_warchief#0"])), 10, "floor(15 x 3000 / 4250)");
    let raiders = |k: &str| match k {
        "ash_shaman#0" => Fate::Killed,
        "ash_brawler#0" => Fate::Surrendered,
        _ => Fate::Standing,
    };
    assert_eq!(gold("ashlands:ash_raiders", "fled", &raiders), 4, "floor(7 x 2500 / 3750)");
    assert_eq!(gold("ashlands:pass_ambush", "fled", &kill(&["skarn_ironjaw#0", "vex_cutpurse#0"])), 14, "the wave that never came keeps its share");
    assert_eq!(gold("ashlands:ashmaw_stand", "won", &kill(&["grakka_ashmaw#0"])), 41, "floor(55 x 20000 / 26250)");
    // every subset of a roster pays at most the roll
    let roster = battle::roster(&d, "ashlands", "ashlands:ashmaw_stand");
    for mask in 0..(1u32 << roster.len()) {
        let keys: Vec<String> = roster.iter().enumerate().filter(|(i, _)| mask & (1 << i) != 0).map(|(_, f)| f.key.clone()).collect();
        let g = gold("ashlands:ashmaw_stand", "fled", &|k: &str| if keys.iter().any(|x| x == k) { Fate::Killed } else { Fate::Standing });
        assert!((0..=55).contains(&g), "{keys:?} pays {g}");
    }
}

#[test]
fn a_once_purse_pays_each_foe_once_per_save() {
    let (_s, d) = scratch("once", &[(Some("ashlands"), "encounters", "pass_ambush", json!({"purse": {"gold_min": 20, "gold_max": 30, "once": true}}))]);
    let qid = "ashlands:pass_ambush";
    let mut gs = game(&d);
    let two = |k: &str| if k == "skarn_ironjaw#0" || k == "vex_cutpurse#0" { Fate::Killed } else { Fate::Standing };
    let first = battle::apply(&d, &mut gs, qid, "fled", &tally(&d, qid, &two));
    let second = battle::apply(&d, &mut gs, qid, "won", &tally(&d, qid, &|_| Fate::Killed));
    let third = battle::apply(&d, &mut gs, qid, "won", &tally(&d, qid, &|_| Fate::Killed));
    let g = |r: &Dict| json::i(json::dget(r, "rewards"), "gold", -1);
    assert!(g(&first) > 0 && g(&second) > 0, "both parts pay: {} {}", g(&first), g(&second));
    assert!(g(&first) + g(&second) <= 30, "never more than gold_max over the parts");
    assert_eq!(g(&third), 0, "every share is paid");
    assert!(!third["xp"].as_object().unwrap().is_empty(), "the third win still gives XP");
    assert_eq!(gs.purses[qid], json!(3), "every settled fight with a defeated foe counts");
    assert_eq!(gs.reward_paid[qid].as_array().unwrap().len(), 4);
}

#[test]
fn a_loot_table_drops_one_row_by_its_share_of_the_hundred() {
    let rows = json!([{"item": "a", "chance": 3}, {"item": "b", "chance": 3}, {"item": "c", "chance": 2}]);
    let rows = rows.as_array().unwrap();
    let hits: Vec<String> = (0..100).map(|r| battle::loot_row(rows, r).map(|x| json::s(x, "item")).unwrap_or_default()).collect();
    assert_eq!(hits.iter().filter(|h| *h == "a").count(), 3);
    assert_eq!(hits.iter().filter(|h| *h == "b").count(), 3);
    assert_eq!(hits.iter().filter(|h| *h == "c").count(), 2);
    assert_eq!((&hits[0], &hits[3], &hits[6], &hits[8]), (&"a".to_string(), &"b".to_string(), &"c".to_string(), &String::new()));
    let full = json!([{"item": "a", "chance": 60}, {"item": "b", "chance": 40}]);
    assert!((0..100).all(|r| battle::loot_row(full.as_array().unwrap(), r).is_some()), "a hundred in all always drops");
}

#[test]
fn defeated_foes_roll_their_tables_into_the_stash() {
    let (_s, d) = scratch("drops", &[
        (None, "units", "goblin_brawler", json!({"loot": [{"item": "con_heal_potion", "chance": 100, "count": 2}]})),
        // the scouts' units without tables, whatever the shipped data gives them
        (None, "units", "goblin_skirmisher", json!({"loot": []})),
        (None, "units", "goblin_archer", json!({"loot": []})),
    ]);
    let qid = "ashlands:ash_raiders";
    let mut gs = game(&d);
    let before = gs.party.stash.get("con_heal_potion").copied().unwrap_or(0);
    let res = battle::apply(&d, &mut gs, qid, "lost", &tally(&d, qid, &|k| if k == "ash_brawler#0" { Fate::Surrendered } else { Fate::Standing }));
    assert_eq!(json::aget(json::dget(&res, "rewards"), "drops").len(), 1, "a surrendered foe rolls too (Q8)");
    assert_eq!(gs.party.stash.get("con_heal_potion").copied().unwrap_or(0), before + 2);
    assert_eq!(gs.rolls.get("loot"), Some(&json!(1)));
    let mut quiet = game(&d);
    battle::apply(&d, &mut quiet, "ashlands:ash_scouts", "won", &tally(&d, "ashlands:ash_scouts", &|_| Fate::Killed));
    assert!(!quiet.rolls.contains_key("loot"), "units without a table draw nothing");
}

fn axe() -> Vec<(Option<&'static str>, &'static str, &'static str, Value)> {
    vec![
        (Some("ashlands"), "loot", "warchiefs_axe", json!({"name": "Warchief's Axe", "reward_kind": "item", "ref": "wpn_battle_axe", "count": 1, "rarity": "rare"})),
        (Some("ashlands"), "encounters", "ash_patrol", json!({"key_loot": [{"loot": "warchiefs_axe", "kill": "ash_warchief"}], "purse": {"gold_min": 15, "gold_max": 15, "once": false}})),
    ]
}

#[test]
fn key_loot_needs_the_kill_and_the_win() {
    let (_s, d) = scratch("key", &axe());
    let qid = "ashlands:ash_patrol";
    let both = |_: &str| Fate::Killed;
    let warchief_fled = |k: &str| if k == "ash_warchief#0" { Fate::Fled } else { Fate::Killed };
    let mut gs = game(&d);
    let res = battle::apply(&d, &mut gs, qid, "won", &tally(&d, qid, &warchief_fled));
    assert_eq!(json::aget(json::dget(&res, "rewards"), "key_missed").len(), 1, "fled on its own: missed");
    for outcome in ["fled", "lost"] {
        let res = battle::apply(&d, &mut gs, qid, outcome, &tally(&d, qid, &both));
        assert!(json::aget(json::dget(&res, "rewards"), "key_loot").is_empty(), "killed but not won ({outcome}): no axe");
    }
    let res = battle::apply(&d, &mut gs, qid, "won", &tally(&d, qid, &both));
    assert_eq!(json::sget(json::dget(&res, "rewards"), "key_loot"), ["ashlands:warchiefs_axe"]);
    assert!(gs.loot_claimed.contains(&"ashlands:warchiefs_axe".to_string()));
    assert!(gs.party.stash.contains_key("wpn_battle_axe"), "the item is in the stash");
    let again = battle::apply(&d, &mut gs, qid, "won", &tally(&d, qid, &both));
    let r = json::dget(&again, "rewards");
    assert!(json::aget(r, "key_loot").is_empty() && json::aget(r, "key_missed").is_empty(), "claimed: skipped silently");
    assert_eq!(json::i(r, "gold", 0), 15, "the gold still pays");
}

#[test]
fn the_objective_reward_pays_a_won_fight_once() {
    let (_s, d) = scratch("objective", &[(Some("ashlands"), "encounters", "courtyard_skirmish", json!({"objective_reward": {"gold": 10, "once": true}, "purse": {"gold_min": 18, "gold_max": 18, "once": true}}))]);
    let qid = "ashlands:courtyard_skirmish";
    let vex = |k: &str| if k == "vex_cutpurse#0" { Fate::Killed } else { Fate::Standing };
    let mut gs = game(&d);
    let gold0 = gs.party.gold;
    let fled = battle::apply(&d, &mut gs, qid, "fled", &tally(&d, qid, &|_| Fate::Standing));
    assert_eq!(json::dget(json::dget(&fled, "rewards"), "objective"), &Dict::new(), "no objective on a flight");
    let won = battle::apply(&d, &mut gs, qid, "won", &tally(&d, qid, &vex));
    let r = json::dget(&won, "rewards");
    assert_eq!((json::i(r, "gold", -1), json::i(json::dget(r, "objective"), "gold", -1)), (9, 10), "the hold win: Vex's share and the objective");
    assert_eq!(gs.party.gold, gold0 + 19);
    let again = battle::apply(&d, &mut gs, qid, "won", &tally(&d, qid, &|_| Fate::Standing));
    assert_eq!(json::dget(json::dget(&again, "rewards"), "objective"), &Dict::new(), "once: the second win pays no bonus");
}

#[test]
fn a_skipped_fight_pays_everything() {
    let mut edits = axe();
    edits.push((None, "units", "goblin_berserker", json!({"loot": [{"item": "con_heal_potion", "chance": 100}]})));
    edits.push((Some("ashlands"), "encounters", "ash_patrol", json!({"key_loot": [{"loot": "warchiefs_axe", "kill": "ash_warchief"}], "purse": {"gold_min": 15, "gold_max": 15, "once": true}, "objective_reward": {"gold": 5}})));
    let (_s, d) = scratch("skip", &edits);
    let mut gs = game(&d);
    let fight = json::obj([("encounter", json!("ashlands:ash_patrol")), ("source", json!("roamer"))]);
    let res = explore::after_fight(&d, &mut gs, &fight, "won", None);
    let r = json::dget(&res, "rewards");
    assert_eq!(json::i(r, "gold", -1), 15, "the full purse");
    assert_eq!(json::i(&res, "gold", -1), 20, "with the objective's");
    assert_eq!(json::aget(r, "drops").len(), 1);
    assert_eq!(json::sget(r, "key_loot"), ["ashlands:warchiefs_axe"]);
    assert_eq!(json::dget(r, "defeated")["slain"], json!(2));
}

#[test]
fn the_rest_of_the_worked_table_and_the_win_only_rewards() {
    let mut edits = axe();
    edits.push((Some("ashlands"), "encounters", "ashmaw_stand", purse(55, false)));
    edits.push((Some("ashlands"), "encounters", "courtyard_skirmish", json!({"objective_reward": {"gold": 10, "once": false}, "purse": {"gold_min": 18, "gold_max": 18, "once": false}})));
    let (_s, d) = scratch("table", &edits);
    let run = |qid: &str, outcome: &str, fate: &dyn Fn(&str) -> Fate| {
        let mut gs = game(&d);
        battle::apply(&d, &mut gs, qid, outcome, &tally(&d, qid, fate))
    };
    let r = |res: &Dict| json::dget(res, "rewards").clone();
    let patrol = "ashlands:ash_patrol";
    let fled_on_win = run(patrol, "won", &|k| if k == "ash_warchief#0" { Fate::Fled } else { Fate::Killed });
    assert_eq!(json::i(&r(&fled_on_win), "gold", -1), 15, "a fled foe is defeated: the full purse");
    let surrendered = run(patrol, "won", &|k| if k == "ash_warchief#0" { Fate::Surrendered } else { Fate::Killed });
    assert_eq!(json::aget(&r(&surrendered), "key_missed").len(), 1, "a surrendered kill target misses the key loot");
    let stand = "ashlands:ashmaw_stand";
    assert_eq!(json::i(&r(&run(stand, "won", &|_| Fate::Killed)), "gold", -1), 55, "every foe down");
    let beaten = run(stand, "lost", &|k| if k == "ash_shaman#0" || k == "ash_archer#0" { Fate::Killed } else { Fate::Standing });
    assert_eq!(json::i(&r(&beaten), "gold", -1), 5, "floor(55 x 2500 / 26250)");
    let court = "ashlands:courtyard_skirmish";
    let held = run(court, "won", &|_| Fate::Standing);
    assert_eq!((json::i(&r(&held), "gold", -1), json::i(json::dget(&r(&held), "objective"), "gold", -1)), (0, 10), "a hold win with nobody killed pays the objective");
    assert_eq!(held["xp"], json!({}), "and no XP");
    let lost = run(court, "lost", &|_| Fate::Killed);
    assert!(json::dget(&r(&lost), "objective").is_empty(), "no objective reward on a loss");
}

#[test]
fn the_same_save_and_tally_give_the_same_rewards() {
    let (_s, d) = scratch("determinism", &[
        (None, "units", "goblin_brawler", json!({"loot": [{"item": "con_heal_potion", "chance": 40}, {"item": "arm_leather", "chance": 30}]})),
        (Some("ashlands"), "encounters", "ash_raiders", json!({"purse": {"gold_min": 5, "gold_max": 30, "once": false}})),
    ]);
    let qid = "ashlands:ash_raiders";
    let t = tally(&d, qid, &|k| if k == "ash_skirmisher#0" { Fate::Standing } else { Fate::Killed });
    let start = game(&d);
    let (mut a, mut b) = (start.clone(), start);
    let (ra, rb) = (battle::apply(&d, &mut a, qid, "fled", &t), battle::apply(&d, &mut b, qid, "fled", &t));
    assert_eq!(ra, rb);
    assert_eq!(a.to_value(), b.to_value());
}

// ---- ENCOUNTER2-W: the world reads the hold escape's result ---------------------------------------------------

use emberwake_battle::session::Prebuilt;
use emberwake_world::roster;

/// The flag the test hold names its holder in (ENCOUNTER2 W2).
const HOLD_FLAG: &str = "test_winch_held";

#[test]
fn a_won_hold_battle_captures_the_holder_and_names_it_in_the_flag() {
    // ENCOUNTER2_TASKS 1, row W (W2, W3; ROSTER1_TASKS 1, row W): a won hold battle with `flag` and `then` capture:
    // the flag holds the holder's member id; the holder is captured in `party.departed` (cause stayed) through
    // `roster::depart` and out of `active_ids`; a save round trip keeps both. A test-only encounter in a scratch copy
    // (the first shipped world encounter's rows as its template), fought by the start party as the battle screen
    // fields it, settled through the dialogue runner from the battle's tally.
    let shipped = Data::open(&data_dir()).unwrap().0;
    let mut row: Dict = shipped.resolve("ashlands", "encounters", "ash_scouts").iter().filter(|(k, _)| !k.starts_with('_')).map(|(k, v)| (k.clone(), v.clone())).collect();
    for (k, v) in json!({
        "id": "w_hold_test", "rows": vec![".".repeat(12); 9], "exits": [[11, 3], [11, 4]], "objective": "escape", "objective_count": 1,
        "stay_behind": {"mode": "hold", "tiles": [[10, 1]], "tiles_label": "the winch", "flag": HOLD_FLAG, "then": "capture"}
    }).as_object().unwrap() {
        row.insert(k.clone(), v.clone());
    }
    let (_s, d) = scratch("hold", &[(Some("ashlands"), "encounters", "w_hold_test", Value::Object(row))]);
    let mut gs = game(&d);
    // a test-only second member (a copy of the first under its own id), so someone escapes while the holder holds
    let mut escaper = gs.party.members[0].clone();
    escaper.id = "w_escaper".into();
    escaper.lore_ref = String::new();
    gs.party.members.push(escaper);
    gs.party.active_ids = gs.party.members.iter().map(|m| m.id.clone()).collect();
    let world = battle::fight_world(&d, &gs, "w_hold_test");
    let entry = d.resolve(&world, "encounters", "w_hold_test");
    let def = battle::encounter_def_in(&d, &gs, &world, entry);
    let (mut party, mut members) = (Vec::new(), Vec::new());
    for m in gs.party.active_members() {
        let f = roster::field_unit(&d, &gs, m, &world).expect("a start member takes the field");
        party.push(Prebuilt { unit: f.unit, art: String::new(), palette: String::new(), profile: f.profile, key: m.id.clone() });
        members.push((m.id.clone(), f.pouch_unloaded));
    }
    assert!(members.len() >= 2, "a holder and someone to escape: {members:?}");
    let mut s = Session::start_with(d.cat.clone(), def, &WorldRoster { d: &d, world: world.clone() }, party);
    assert!(s.stay.is_some(), "the scratch row is a hold escape");
    // the hold's end (H4): the first hero holds the winch, every other hero withdrew through an exit
    let heroes: Vec<usize> = s.w.allies.iter().copied().take(members.len()).collect();
    s.objective.stayed = vec![heroes[0]];
    for &h in &heroes[1..] {
        s.w.units[h].is_withdrawn = true;
    }
    let holder = members[0].0.clone();
    let name = progression::display_name(&d, gs.party.member(&holder).unwrap());
    let tally = Tally::of_world_fight(&s, &members, &world);
    let res = emberwake_world::dialogue::after_fight(&d, &mut gs, &json::obj([("encounter", "w_hold_test".into())]), "won", Some(&tally));
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(gs.flag(HOLD_FLAG), Some(&json!(holder)), "W2");
    let rec = gs.party.departed_of(&holder).expect("W3: the holder departed");
    assert_eq!((rec.life.as_str(), rec.cause.as_str(), rec.encounter.as_str()), ("captured", "stayed", "ashlands:w_hold_test"));
    assert!(gs.party.member(&holder).is_none() && !gs.party.active_ids.contains(&holder), "out of the party and the deployment");
    assert!(members[1..].iter().all(|(m, _)| gs.party.member(m).is_some()), "the escapers come home (no rule row: HEAL1 classic)");
    assert_eq!(res["stay_lines"], json!([emberwake_battle::encounter::DEFAULT_RESULT_LINE.replace("{name}", &name)]), "W4");
    assert_eq!(res["departed"], json!([{"member": holder, "life": "captured"}]));
    // the save round trip keeps the departure and the flag (ROSTER1-L's save keys; no key of W's own)
    let back = GameState::from_dict(&d, gs.to_value().as_object().unwrap());
    assert_eq!((back.party.departed.clone(), back.flag(HOLD_FLAG).cloned()), (gs.party.departed.clone(), Some(json!(holder))));
    assert!(back.validate(&d).is_empty(), "{:?}", back.validate(&d));
    assert_eq!(back.to_value(), gs.to_value());
}
