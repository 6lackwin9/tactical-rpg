//! SIMCOUNT1 (docs/specs/INPUT1.md 11.8) on a scratch catalog built here (no shipped data): the `feel` rule counts. Each
//! counter is read off a scripted round whose count the board decides; `melee_diagonal` reaches them through the sim's
//! keys; asking for the counts changes no fight (the same seeds, the same outcomes, every other line the same).

use indexmap::IndexMap;
use rpgkit_battle::command::{CmdKind, Command, MoveKind};
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::PlanStep;
use rpgkit_battle::round_intent::{self, ManeuverKind, RoundIntent, RoundTarget};
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{FlankCost, MeleeDiagonal, Ruleset, AP_COST, FLANK, ROUND_INTENT, TIMELINE};
use rpgkit_sim::args::SimArgs;
use rpgkit_sim::feel::{Counts, Feel, Side, JSON_PREFIX, RULE_COUNTERS};
use rpgkit_sim::runner::Runner;
use rpgkit_sim::SimGame;
use rpgkit_world::Data;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const UNIT_AP: i32 = 10;
const BASICS: [&str; 4] = ["move", "rest", "strike", "flank_cut"];

fn atk(id: &str, intent: &str, blocks: i32, ap: i32, extra: Value) -> Value {
    let mut row = json!({"id": id, "name": id, "kind": "ATTACK", "round_intent": intent, "level": 1, "blocks": blocks, "ap_cost": ap, "power_mult": 0.8, "target_mode": "ENEMY"});
    for (k, v) in extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    row
}

fn skills() -> Vec<Value> {
    vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "round_intent": "assault", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        atk("flank_cut", "maneuver", 1, 2, json!({"position": "flank", "reach_max": 2})),
        atk("t_lunge", "assault", 1, 2, json!({"dash_tiles": 2})),
        atk("t_shot", "assault", 1, 2, json!({"power_mult": 1.1})),
        atk("t_lob", "assault", 1, 2, json!({"power_mult": 1.1, "reach_min": 2, "reach_max": 2})),
        atk("t_thrust", "assault", 1, 2, json!({"reach_min": 1, "reach_max": 2})),
        json!({"id": "t_dressing", "name": "Dressing", "kind": "SUPPORT", "round_intent": "support", "level": 1, "blocks": 1, "ap_cost": 2, "heal_base": 8, "heal_mult": 0.3, "target_mode": "ALLY_OR_SELF", "reach_max": 3}),
    ]
}

fn catalog() -> Catalog {
    let unit = |id: &str, side: &str, range: i32, order: Value| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": range, "skill_cap": 5, "deck": {"order": order}});
    let units = vec![
        unit("t_blade", "party", 1, json!({"assault": ["strike", "t_lunge"], "maneuver": ["flank_cut"]})),
        unit("t_pike", "party", 1, json!({"assault": ["t_thrust"]})),
        unit("t_bow", "party", 3, json!({"assault": ["t_shot", "t_lob"], "support": ["t_dressing"]})),
        unit("t_mate", "party", 1, json!({})),
        unit("t_wall", "party", 1, json!({})),
        unit("t_foe", "enemy", 1, json!({})),
        unit("t_foe_b", "enemy", 1, json!({})),
    ];
    let items = vec![json!({"id": "t_potion", "name": "Potion", "item_type": "CONSUMABLE", "heal": 20})];
    Catalog::from_value(&json!({"skills": skills(), "units": units, "items": items}))
}

const BLADE: UnitId = 0;
const PIKE: UnitId = 1;
const BOW: UnitId = 2;
const MATE: UnitId = 3;
const WALL: UnitId = 4;
const FOE: UnitId = 5;
const FOE_B: UnitId = 6;

fn ruleset(diag: MeleeDiagonal) -> Ruleset {
    let mut r = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, AP_COST, FLANK], "basics": BASICS}).as_object().unwrap());
    r.numbers.melee_diagonal = diag;
    r.numbers.flank_protected = FlankCost::Full;
    r
}

/// Every unit parked on the edges; only what a test plans acts.
fn session(diag: MeleeDiagonal) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["t_blade", "t_pike", "t_bow", "t_mate", "t_wall"], "enemies": ["t_foe", "t_foe_b"], "total_blocks": BLOCKS, "unit_ap": UNIT_AP, "ap_regen": 0, "max_rounds": 8});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset(diag);
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [V2::new(0, 0), V2::new(0, 8), V2::new(2, 0), V2::new(2, 8), V2::new(4, 0), V2::new(11, 0), V2::new(11, 8)].into_iter().enumerate() {
        place(&mut s.w, c, cell);
    }
    drop_enemy_plans(&mut s);
    s
}

fn drop_enemy_plans(s: &mut Session) {
    let enemies = s.w.enemies.clone();
    s.resolver.scheduled.retain(|a| !enemies.contains(&a.unit));
}

fn place(w: &mut World, c: UnitId, cell: V2) {
    w.grid.positions[c] = Some(cell);
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

/// `FOE` on (6, 4) engaged with `MATE` on (7, 4), so it faces right; its flanks are (6, 3) and (6, 5).
fn board(s: &mut Session) {
    place(&mut s.w, FOE, V2::new(6, 4));
    place(&mut s.w, MATE, V2::new(7, 4));
    round_intent::set_intent(&mut s.w, FOE, RoundIntent::Assault).unwrap();
    round_intent::set_target(&mut s.w, FOE, RoundTarget::Unit(MATE)).unwrap();
}

/// `BLADE` on (5, 2) plans Flank Cut on `FOE`.
fn flank_cut(s: &mut Session) {
    place(&mut s.w, BLADE, V2::new(5, 2));
    round_intent::set_intent(&mut s.w, BLADE, RoundIntent::Maneuver).unwrap();
    round_intent::set_target(&mut s.w, BLADE, RoundTarget::Unit(FOE)).unwrap();
    round_intent::set_maneuver(&mut s.w, BLADE, ManeuverKind::Engage).unwrap();
    let k = sid(&s.w, "flank_cut");
    assert_eq!(s.plan.add_step(&s.w, BLADE, PlanStep::engage(k, FOE), false), "");
}

/// `c` plans `skill` on `t` under the assault intent.
fn assault(s: &mut Session, c: UnitId, skill: &str, t: UnitId) {
    round_intent::set_intent(&mut s.w, c, RoundIntent::Assault).unwrap();
    round_intent::set_target(&mut s.w, c, RoundTarget::Unit(t)).unwrap();
    let k = sid(&s.w, skill);
    assert_eq!(s.plan.add_step(&s.w, c, PlanStep::engage(k, t), false), "");
}

fn confirm(s: &mut Session) {
    assert!(s.confirm_plan());
    drop_enemy_plans(s);
}

fn play_out(s: &mut Session) {
    while s.phase != Phase::Replay {
        s.step_next();
    }
}

/// What the feel observer counts of the round just resolved.
fn observed(s: &Session) -> Counts {
    let mut f = Feel::default();
    f.start(s);
    f.round_end(s);
    f.counts
}

/// A side's counts as (key, count), only the ones that counted.
fn counted(side: &Side) -> Vec<(&'static str, i64)> {
    RULE_COUNTERS.iter().map(|(k, _, _)| (*k, side.rule_count(k))).filter(|(_, n)| *n != 0).collect()
}

/// MR5, MR8 c: the flank closed after planning: one protected flank, one Maneuver command, no dash, no hit. It is one
/// count, though the log writes the miss twice (at the start and at the impact).
#[test]
fn a_protected_flank_counts_once() {
    let mut s = session(MeleeDiagonal::Maneuver);
    board(&mut s);
    flank_cut(&mut s);
    confirm(&mut s);
    // the flanks and both diagonal fallbacks taken
    for (c, cell) in [(FOE_B, V2::new(6, 3)), (PIKE, V2::new(6, 5)), (BOW, V2::new(5, 3)), (WALL, V2::new(5, 5))] {
        place(&mut s.w, c, cell);
    }
    play_out(&mut s);
    let lines = s.resolver.log.iter().filter(|l| l.starts_with("[Flank Protected]")).count();
    let c = observed(&s);
    assert_eq!(lines, 2, "the log writes it at the start and at the impact");
    assert_eq!(counted(&c.party), [("protected_flanks", 1), ("maneuver_commands", 1)]);
    assert!(counted(&c.enemy).is_empty());
    assert_eq!(c.report_json(true)["party"]["rule_counts"]["protected_share_pct"], json!(100.0));
}

/// MR8 a, MR10: under `maneuver` both flanks taken: Flank Cut dashes to the diagonal fallback and lands from it: one dash,
/// one diagonal hit. Under `none` it ends beside its target: the dash counts, the diagonal does not.
#[test]
fn a_dash_and_a_diagonal_hit_follow_melee_diagonal() {
    let round = |diag: MeleeDiagonal, flanks_taken: &[(UnitId, V2)]| {
        let mut s = session(diag);
        board(&mut s);
        for (c, cell) in flanks_taken {
            place(&mut s.w, *c, *cell);
        }
        flank_cut(&mut s);
        confirm(&mut s);
        play_out(&mut s);
        // under `none` a plain Move may come first (MR7): the Flank Cut is the command with the position
        let a = s.resolver.scheduled.iter().find(|a| a.unit == BLADE && a.command.position.is_some()).unwrap();
        assert_eq!(a.command.moved, Some(MoveKind::Dash), "{diag:?}");
        assert_eq!(a.command.hits.len(), 1, "{diag:?}: the hit lands");
        observed(&s)
    };
    let both = [(FOE_B, V2::new(6, 3)), (PIKE, V2::new(6, 5))];
    let c = round(MeleeDiagonal::Maneuver, &both);
    assert_eq!(counted(&c.party), [("maneuver_commands", 1), ("dashes", 1), ("diagonal_hits", 1)]);
    let c = round(MeleeDiagonal::None, &both[..1]);
    assert_eq!(counted(&c.party), [("maneuver_commands", 1), ("dashes", 1)]);
    // an open flank under `maneuver`: the same dash, no diagonal
    let c = round(MeleeDiagonal::Maneuver, &[]);
    assert_eq!(counted(&c.party), [("maneuver_commands", 1), ("dashes", 1)]);
}

/// MR10: under `all` a plain Strike lands from a diagonal; under `maneuver` it has to walk beside its target first. A
/// reach weapon on the same tile needs no rule, so that hit is not a diagonal hit.
#[test]
fn a_strike_is_a_diagonal_hit_only_under_all() {
    let round = |diag: MeleeDiagonal, c: UnitId, skill: &str| {
        let mut s = session(diag);
        place(&mut s.w, FOE, V2::new(6, 4));
        place(&mut s.w, c, V2::new(5, 3));
        assault(&mut s, c, skill, FOE);
        confirm(&mut s);
        play_out(&mut s);
        assert!(s.resolver.scheduled.iter().filter(|a| a.unit == c).any(|a| a.command.hits.len() == 1), "{diag:?}: the hit lands");
        observed(&s).party.rule_count("diagonal_hits")
    };
    assert_eq!(round(MeleeDiagonal::All, BLADE, "strike"), 1);
    assert_eq!(round(MeleeDiagonal::Maneuver, BLADE, "strike"), 0);
    assert_eq!(round(MeleeDiagonal::None, BLADE, "strike"), 0);
    assert_eq!(round(MeleeDiagonal::All, PIKE, "t_thrust"), 0, "band 1-2: the tile is in reach without the rule");
}

/// A plain `dash_tiles` skill (outside the Maneuver intent) is a dash, not a Maneuver command, with no board event of its
/// own (`moved` stays None, as it was).
#[test]
fn a_plain_dash_counts_as_a_dash_only() {
    let mut s = session(MeleeDiagonal::Maneuver);
    place(&mut s.w, FOE, V2::new(6, 4));
    place(&mut s.w, BLADE, V2::new(4, 4));
    assault(&mut s, BLADE, "t_lunge", FOE);
    confirm(&mut s);
    play_out(&mut s);
    assert_eq!(s.w.pos(BLADE), V2::new(5, 4), "the dash ends beside its target");
    let c = observed(&s);
    assert_eq!(counted(&c.party), [("dashes", 1)]);
}

/// MR16: the target stepped after planning: each skill kind chases one tile. Melee and a reach weapon (band up to 2) read
/// as melee, a longer band as ranged, a heal on an ally as Support.
#[test]
fn chases_count_by_skill_kind() {
    let attack = |c: UnitId, skill: &str, from: V2, to: V2| {
        let mut s = session(MeleeDiagonal::Maneuver);
        place(&mut s.w, c, from);
        place(&mut s.w, FOE, V2::new(6, 4));
        assault(&mut s, c, skill, FOE);
        confirm(&mut s);
        place(&mut s.w, FOE, to);
        play_out(&mut s);
        counted(&observed(&s).party)
    };
    assert_eq!(attack(BLADE, "strike", V2::new(5, 4), V2::new(7, 4)), [("chases_melee", 1)]);
    assert_eq!(attack(PIKE, "t_thrust", V2::new(4, 4), V2::new(7, 4)), [("chases_melee", 1)]);
    assert_eq!(attack(BOW, "t_shot", V2::new(3, 4), V2::new(7, 4)), [("chases_ranged", 1)]);
    // A short band is still ranged: it does not start at melee reach
    assert_eq!(attack(BOW, "t_lob", V2::new(4, 4), V2::new(7, 4)), [("chases_ranged", 1)]);
    let mut s = session(MeleeDiagonal::Maneuver);
    place(&mut s.w, BOW, V2::new(3, 4));
    place(&mut s.w, MATE, V2::new(6, 4));
    round_intent::set_intent(&mut s.w, BOW, RoundIntent::Support).unwrap();
    round_intent::set_target(&mut s.w, BOW, RoundTarget::Unit(MATE)).unwrap();
    let k = sid(&s.w, "t_dressing");
    assert_eq!(s.plan.add_step(&s.w, BOW, PlanStep::ally(k, MATE), false), "");
    confirm(&mut s);
    place(&mut s.w, MATE, V2::new(7, 4));
    play_out(&mut s);
    assert_eq!(counted(&observed(&s).party), [("chases_support", 1)]);
}

/// A count goes to the side of the acting unit: an enemy's chase is the enemy's.
#[test]
fn an_enemy_chase_counts_for_the_enemy() {
    let mut s = session(MeleeDiagonal::Maneuver);
    place(&mut s.w, FOE, V2::new(5, 4));
    place(&mut s.w, MATE, V2::new(7, 4));
    confirm(&mut s);
    let mut cmd = Command::new(FOE, 0, 1, CmdKind::Attack { target: Some(MATE), skill_mult: 1.0, use_offhand: false, follow_tiles: 1, dash_tiles: 0, status_rounds: 0, min_range: 1, max_range: 1, last_damage: 0, last_crit: false, last_target: None });
    cmd.name = "t_swing".into();
    assert!(s.resolver.schedule(FOE, cmd, 1));
    play_out(&mut s);
    let c = observed(&s);
    assert_eq!(counted(&c.enemy), [("chases_melee", 1)]);
    assert!(counted(&c.party).is_empty());
}

/// A command that never started counts nothing: its unit fell before its start.
#[test]
fn a_command_that_never_started_counts_nothing() {
    let mut s = session(MeleeDiagonal::Maneuver);
    board(&mut s);
    flank_cut(&mut s);
    confirm(&mut s);
    s.w.units[BLADE].hp = 0;
    play_out(&mut s);
    assert!(counted(&observed(&s).party).is_empty());
}

/// An item that landed counts once, in total and by id; one that failed counts nothing.
#[test]
fn items_used_count_by_id_and_only_when_used() {
    let round = |in_pouch: bool| {
        let mut s = session(MeleeDiagonal::Maneuver);
        let item = s.w.cat.items["t_potion"].clone();
        s.w.units[MATE].hp = 100;
        if in_pouch {
            s.w.units[MATE].inventory.pouch.push(item.clone());
        }
        confirm(&mut s);
        let cmd = Command::new(MATE, 0, 1, CmdKind::Item { target: MATE, item: Box::new(item), used: false, last_heal: 0, last_damage: 0, last_boost: 0.0 });
        assert!(s.resolver.schedule(MATE, cmd, 1));
        play_out(&mut s);
        observed(&s).party
    };
    let side = round(true);
    assert_eq!(counted(&side), [("items_used", 1)]);
    assert_eq!(side.items_by_id.get("t_potion"), Some(&1));
    let side = round(false);
    assert!(counted(&side).is_empty(), "no potion in the pouch: the command fails");
    assert!(side.items_by_id.is_empty());
}

/// Rest blocks a round: the side's Rest blocks over the fight's rounds; a fight's counts per fight likewise.
#[test]
fn rest_blocks_per_round_and_counts_per_fight() {
    let mut s = session(MeleeDiagonal::Maneuver);
    board(&mut s);
    flank_cut(&mut s);
    confirm(&mut s);
    play_out(&mut s);
    let mut f = Feel::default();
    f.start(&s);
    f.planned(&s);
    s.w.round_states.entry(MATE).or_default().rest_blocks = 3;
    f.round_end(&s);
    f.end(&s, 2);
    f.end(&s, 2);
    let side = &f.counts.report_json(true)["party"]["rule_counts"];
    assert_eq!(f.counts.party.rest_blocks, 3);
    assert_eq!(side["rest_blocks_per_round"], json!(0.75), "3 blocks over two fights of 2 rounds");
    assert_eq!(side["counts"]["dashes"], json!(1));
    assert_eq!(side["per_fight"]["dashes"], json!(0.5));
}

// ---- The sim's keys, on an AI-vs-AI fight of the scratch catalog ----------------------------------------------------

const ENCOUNTERS: [&str; 2] = ["t_crowd", "t_duel"];

fn data() -> Data {
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 90, "speed": 12, "attack_range": 1, "skill_cap": 5, "poise": 30, "deck": {"order": {"assault": ["strike"], "maneuver": ["flank_cut"]}}});
    let enc = |id: &str, party: &[&str], enemies: &[&str]| json!({"id": id, "name": id, "zone": "stone_arena", "arena": "CLOSED", "max_rounds": 6, "total_blocks": 6, "ap_regen": 4, "unit_ap": 10, "party": party, "enemies": enemies});
    let catalog = json!({
        "skills": skills(),
        "units": [unit("t_a", "party"), unit("t_b", "party"), unit("t_c", "party"), unit("t_x", "enemy"), unit("t_y", "enemy"), unit("t_z", "enemy")],
        "encounters": [enc("t_crowd", &["t_a", "t_b", "t_c"], &["t_x", "t_y", "t_z"]), enc("t_duel", &["t_a"], &["t_x"])],
        "rulesets": [{"id": TIMELINE, "name": "Timeline", "rules": ["round_intent", "enemy_moves", "poise", "ap_cost", "flank"], "basics": ["move", "rest", "strike", "flank_cut"]}],
    });
    Data::from_values(&catalog, &json!({}), &json!({}))
}

fn game() -> SimGame {
    SimGame { feel_encounters: ENCOUNTERS.iter().map(|e| e.to_string()).collect(), ..SimGame::default() }
}

fn argv(line: &str) -> Vec<String> {
    line.split_whitespace().map(String::from).collect()
}

fn run(d: &Data, line: &str) -> Vec<String> {
    rpgkit_sim::run(d, &argv(line), &game())
}

fn report(out: &[String]) -> Value {
    let last = out.iter().rev().find(|l| l.starts_with(JSON_PREFIX)).expect("a FEEL_JSON line");
    serde_json::from_str(last.strip_prefix(JSON_PREFIX).unwrap()).expect("JSON")
}

/// Drops the rule counts from a report, leaving what a run without the key prints.
fn without_rule_counts(mut v: Value) -> Value {
    let mut scopes: Vec<String> = ENCOUNTERS.iter().map(|e| e.to_string()).collect();
    scopes.push("total".into());
    for scope in scopes {
        let block = if scope == "total" { &mut v["total"] } else { &mut v["encounters"][&scope] };
        for side in ["party", "enemy"] {
            block[side].as_object_mut().unwrap().shift_remove("rule_counts");
        }
    }
    v
}

/// Counting changes no fight: with the key every fight comes out as without it, the old lines and keys unchanged.
#[test]
fn asking_for_the_counts_changes_no_result() {
    let d = data();
    for rest in ["party=both", "melee_diagonal=none", "melee_diagonal=all party=timed"] {
        let off = run(&d, &format!("feel seeds=4 {rest}"));
        let on = run(&d, &format!("feel seeds=4 {rest} counts=on"));
        let strip = |lines: &[String]| -> Vec<String> { lines.iter().filter(|l| !l.contains(" rule counts: ") && !l.starts_with("ARGS") && !l.starts_with(JSON_PREFIX)).cloned().collect() };
        assert_eq!(strip(&on), strip(&off), "{rest}: the same lines beside the counts");
        assert!(on.iter().any(|l| l.contains("party rule counts: ")), "{rest}");
        assert!(!off.iter().any(|l| l.contains(" rule counts: ")), "{rest}: no counts without the key");
        let (on_json, off_json) = (report(&on), report(&off));
        assert_eq!(without_rule_counts(on_json), off_json, "{rest}: the same FEEL_JSON beside the counts");
    }
}

/// Counting changes no fight, by the observer itself: the same seeds fought with no observer and with one give the same
/// outcome, rounds and HP.
#[test]
fn the_observer_changes_no_fight() {
    let d = data();
    let a = SimArgs::parse_for(&d, &argv("feel enc=t_crowd seeds=6"), &game()).unwrap();
    let entry = d.entry("encounters", "t_crowd").clone();
    let party: Vec<String> = ["t_a", "t_b", "t_c"].map(String::from).to_vec();
    let enemies: Vec<String> = ["t_x", "t_y", "t_z"].map(String::from).to_vec();
    let mut base = entry;
    for key in ["party", "enemies", "max_rounds", "rng_seed"] {
        base.shift_remove(key);
    }
    let fights = |observe: bool| {
        let mut r = Runner::new(&d, &a);
        r.rounds_override = 6;
        r.feel = observe.then(Feel::default);
        a.seeds().iter().map(|&seed| { let f = r.fight(&party, &enemies, seed, &base); (f.won, f.rounds, f.survivors, f.party_hp) }).collect::<Vec<_>>()
    };
    let (plain, watched) = (fights(false), fights(true));
    assert_eq!(plain.len(), 6);
    assert_eq!(plain, watched);
}

/// `melee_diagonal=` reaches the counts: none has no diagonal hit on any fight; all has them where the same fights under
/// none had to walk.
#[test]
fn the_counts_follow_melee_diagonal_in_a_run() {
    let d = data();
    let diagonal = |value: &str| {
        let r = report(&run(&d, &format!("feel enc=t_crowd seeds=12 counts=on melee_diagonal={value}")));
        (r["total"]["party"]["rule_counts"]["counts"]["diagonal_hits"].as_i64().unwrap(), r["total"]["enemy"]["rule_counts"]["counts"]["diagonal_hits"].as_i64().unwrap())
    };
    assert_eq!(diagonal("none"), (0, 0));
    let (party, enemy) = diagonal("all");
    assert!(party + enemy > 0, "under all some hit lands from a diagonal");
}

/// The key: its refusals, every table row in the JSON and on the human line, and the help (apart from `usage`, which the
/// sim golden holds).
#[test]
fn the_key_the_table_and_the_help() {
    let d = data();
    assert_eq!(run(&d, "feel counts=maybe")[0], "SIM argument error: counts must be on or off (got 'maybe')");
    assert_eq!(run(&d, "duel 5 1 counts=on")[0], "SIM argument error: counts= is a feel key (mode feel only)");
    let on = run(&d, "feel enc=t_duel seeds=1 counts=on");
    let r = report(&on);
    let line = on.iter().find(|l| l.contains("enemy rule counts: ")).unwrap();
    for (key, label, _) in RULE_COUNTERS {
        for side in ["party", "enemy"] {
            assert!(r["total"][side]["rule_counts"]["counts"][key].is_i64(), "{side} {key}");
            assert!(r["total"][side]["rule_counts"]["per_fight"][key].is_number(), "{side} {key}");
        }
        assert!(line.contains(label), "{label}");
    }
    let help = run(&d, "help");
    assert!(help.iter().any(|l| l.contains("counts=on|off")), "sim help names the key");
    assert!(!rpgkit_sim::args::usage(&game()).iter().any(|l| l.contains("counts=")), "usage stays as sim_golden holds it");
}
