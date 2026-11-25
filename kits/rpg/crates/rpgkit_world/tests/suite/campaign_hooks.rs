//! PR6-6 (`docs/sessions/PK4_PK5_CAMPAIGN_HUB.md` 1-2, B3-B5) on a scratch campaign world: a recipe's `doctrine`
//! moves the campaign gauge its `axis` names and the run ticks the campaign with cause `recipe` (recipe triggers by id
//! and tags); a settled battle carries each fielded hero's telemetry (`HeroEnd` strain_peak / strain_end /
//! channel_actions) to the meter inputs (once per hero meeting a `hero` filter) and the battle triggers (one hero
//! meeting it); a signed unit track leans -3..3 (0 left out of the save); `clock_set` with `keep_longer` never
//! shortens a running clock; an `after_flag` checkpoint is armed the day its flag is set and due `offset_days` later
//! (`Campaign::checkpoint_armed_at`, saved only when not empty). Every expectation follows from the rule and the
//! fixture's own numbers. Review (FB-297): a recipe run is no full tick: only recipe inputs and `on: recipe` triggers
//! (no `any` trigger, no checkpoint step, no time change).

use aegis_core::json::{self, Dict};
use rpgkit_world::battle::{self, Tally};
use rpgkit_world::campaign::{self, Cause};
use rpgkit_world::recipe::{self, Pick};
use rpgkit_world::roster::HeroEnd;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{save, Data, GameState};
use serde_json::{json, Value};

const W: &str = "w";
const SMITH: &str = "w:smith";
/// The doctrine gauge's range and a recipe's doctrine delta.
const SPAN: i64 = 100;
const PUSH: i64 = 4;
/// The hot input: +HOT per fielded hero that ended a won battle at HOT_AT or more on the channel meter.
const HOT: i64 = 2;
const HOT_AT: i64 = 50;
/// The clock: a long set, then a shorter one.
const LONG: i64 = 18;
const SHORT: i64 = 5;
/// A recipe input's move (recipes tagged slab).
const SLAB_INPUT: i64 = 3;
/// The armed checkpoint's offset.
const OFFSET: i64 = 10;

fn with_ids(rows: Value) -> Value {
    let rows = json::arr(&rows)
        .iter()
        .map(|r| {
            let mut r = json::dict(r).clone();
            let id = json::s(&r, "id");
            r.insert("_world".into(), W.into());
            r.insert("_qid".into(), format!("{W}:{id}").into());
            Value::Object(r)
        })
        .collect();
    Value::Array(rows)
}

fn recipe(id: &str, effects: Value, tags: Value) -> Value {
    json!({"id": id, "name": id, "npc": "smith", "requires": [], "cost": {}, "target": "member", "effects": effects, "tags": tags})
}

/// `doctrine`: a gauge -SPAN..SPAN; `heat`: moved by HOT per hot hero on a won battle; `slabbed`: moved by SLAB_INPUT
/// per run of a recipe tagged slab; `clock`: a countdown.
fn meters() -> Value {
    json!([
        {"id": "doctrine", "name": "Doctrine", "min": -SPAN, "max": SPAN, "start": 0, "bands": [{"id": "all", "name": "All", "from": -SPAN, "tone": "calm"}]},
        {"id": "heat", "name": "Heat", "max": 999, "bands": [{"id": "low", "name": "Low", "from": 0, "tone": "calm"}],
         "inputs": [{"on": "battle", "outcomes": ["won"], "delta": HOT, "hero": {"strain_end": HOT_AT}}]},
        {"id": "slabbed", "name": "Slabbed", "max": 99, "bands": [{"id": "low", "name": "Low", "from": 0, "tone": "calm"}],
         "inputs": [{"on": "recipe", "tags": ["slab"], "delta": SLAB_INPUT}]},
        {"id": "clock", "name": "Clock", "kind": "countdown", "min": 0, "max": 99, "unit": "days", "bands": [{"id": "due", "name": "Due", "from": 0, "tone": "danger"}]}
    ])
}

fn data(triggers: Value, checkpoints: Value) -> Data {
    let manifest = json!({"id": W, "start_location": "hub", "start_party": ["hero"], "campaign": {"start_act": 1}});
    let worlds = json!({
        W: {"manifest": manifest, "own": {
            "locations": with_ids(json!([{"id": "hub", "name": "Hub"}])),
            "encounters": with_ids(json!([{"id": "enc_x", "name": "X", "enemies": []}])),
            "characters": with_ids(json!([{"id": "smith", "name": "Smith", "role": "npc"}])),
            "meters": with_ids(meters()),
            "checkpoints": with_ids(checkpoints),
            "triggers": with_ids(triggers)
        }}
    });
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}],
        "unit_tracks": [{"id": "lean", "name": "Lean", "signed": true, "stages": []}],
        "recipes": [
            recipe("push", json!([{"effect": "doctrine", "axis": "doctrine", "delta": PUSH}]), json!(["slab"])),
            recipe("pull", json!([{"effect": "doctrine", "axis": "doctrine", "delta": -PUSH}]), json!(["hearth"])),
            recipe("stray", json!([{"effect": "doctrine", "axis": "nowhere", "delta": PUSH}]), json!([])),
            recipe("lean_down", json!([{"effect": "track", "track": "lean", "delta": -1}]), json!([]))
        ]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let mut gs = GameState::create(d, W, party);
    if let Some(c) = gs.campaign.as_mut() {
        c.sync(gs.time);
    }
    gs
}

fn value(gs: &GameState, id: &str) -> i64 {
    gs.campaign.as_ref().unwrap().value(id, gs.time)
}

fn fired(res: &Dict) -> Vec<String> {
    json::aget(res, "campaign").iter().map(json::dict).filter(|e| json::s(e, "event") == "trigger_fired").map(|e| json::s(e, "trigger")).collect()
}

fn pick() -> Pick {
    Pick { member: "hero".into(), ..Pick::default() }
}

fn hero_end(member: &str, strain_end: i64) -> HeroEnd {
    HeroEnd {
        member: member.into(),
        hp: 10,
        max_hp: 10,
        alive: true,
        pouch: Vec::new(),
        unloaded: Vec::new(),
        fate: None,
        marks: Vec::new(),
        hostile: String::new(),
        attacked: Vec::new(),
        worn: Vec::new(),
        strain_peak: strain_end,
        strain_end,
        channel_actions: 0,
    }
}

fn round_trip(d: &Data, gs: &GameState, tag: &str) -> GameState {
    let dir = aegis_core::scratch::Scratch::vacant(tag);
    assert!(save::write(d, &dir, gs, 1, 1).is_empty());
    let back = save::read(d, &dir, 1).expect("reads back");
    let _ = std::fs::remove_dir_all(&dir);
    assert!(back.warnings.is_empty(), "{:?}", back.warnings);
    back.state
}

// ---- B5: doctrine, recipe triggers, battle telemetry ----

#[test]
fn a_doctrine_effect_moves_the_gauge_its_axis_names_and_the_run_ticks_recipe_triggers() {
    let triggers = json!([
        {"id": "t_any_recipe", "name": "Any", "on": "recipe", "once": false, "when": [{"key": "meter.doctrine", "op": ">", "value": 0}], "effects": [{"effect": "set_flag", "key": "any_seen"}]},
        {"id": "t_slab", "name": "Slab", "on": "recipe", "recipe_tags": ["slab"], "effects": [{"effect": "set_flag", "key": "slab_seen"}]},
        {"id": "t_pull", "name": "Pull", "on": "recipe", "recipe": "pull", "effects": [{"effect": "set_flag", "key": "pull_seen"}]}
    ]);
    let d = data(triggers, json!([]));
    let mut gs = state(&d);
    let res = recipe::run(&d, &mut gs, SMITH, "push", &pick());
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(value(&gs, "doctrine"), PUSH, "the doctrine delta on the gauge axis names");
    assert_eq!(fired(&res), ["t_any_recipe", "t_slab"], "the recipe tick: any recipe, then the tag filter");
    assert!(!gs.flag_true("pull_seen"), "the id filter");
    let res = recipe::run(&d, &mut gs, SMITH, "pull", &pick());
    assert_eq!(value(&gs, "doctrine"), 0);
    assert_eq!(fired(&res), ["t_pull"], "a once trigger fires once; the id filter matches");
    // an axis no gauge has changes nothing and refuses nothing
    let res = recipe::run(&d, &mut gs, SMITH, "stray", &pick());
    assert!(json::b(&res, "ok", false));
    assert_eq!(value(&gs, "doctrine"), 0);
    // held in the gauge's range
    for _ in 0..(SPAN / PUSH + 2) {
        recipe::run(&d, &mut gs, SMITH, "push", &pick());
    }
    assert_eq!(value(&gs, "doctrine"), SPAN);
}

#[test]
fn a_battle_carries_each_heros_telemetry_to_the_inputs_and_triggers() {
    let triggers = json!([{"id": "t_hot", "name": "Hot", "on": "battle", "outcome": "won", "hero": {"strain_end": HOT_AT}, "effects": [{"effect": "set_flag", "key": "hot_seen"}]}]);
    let d = data(triggers, json!([]));
    let mut gs = state(&d);
    // no fielded hero (a fight settled without a party side): no hero meets the filter
    let res = battle::settle(&d, &mut gs, "w:enc_x", "won", None);
    assert_eq!(value(&gs, "heat"), 0);
    assert!(fired(&res).is_empty());
    let cold = Tally { party: vec![hero_end("hero", HOT_AT - 1)], ..Tally::default() };
    battle::settle(&d, &mut gs, "w:enc_x", "won", Some(&cold));
    assert_eq!(value(&gs, "heat"), 0, "below the minimum");
    assert!(!gs.flag_true("hot_seen"));
    // the input counts each hero that meets it; the trigger needs one
    let heroes = [campaign::HeroTally { strain_end: HOT_AT, ..Default::default() }, campaign::HeroTally { strain_end: HOT_AT + 1, ..Default::default() }, campaign::HeroTally::default()];
    let cause = Cause::Battle { encounter: "w:enc_x".into(), tags: Vec::new(), outcome: "won".into(), heroes: heroes.to_vec() };
    let before = value(&gs, "heat");
    let events = campaign::tick(&d, &mut gs, &cause);
    assert_eq!(value(&gs, "heat"), before + 2 * HOT, "once per hot hero");
    assert!(events.iter().map(json::dict).any(|e| json::s(e, "trigger") == "t_hot"));
    // a lost battle moves nothing (the input's outcomes)
    let lost = Cause::Battle { encounter: "w:enc_x".into(), tags: Vec::new(), outcome: "lost".into(), heroes: heroes.to_vec() };
    let before = value(&gs, "heat");
    campaign::tick(&d, &mut gs, &lost);
    assert_eq!(value(&gs, "heat"), before);
    // the settle carries a tally's ends
    let hot = Tally { party: vec![hero_end("hero", HOT_AT)], ..Tally::default() };
    let before = value(&gs, "heat");
    battle::settle(&d, &mut gs, "w:enc_x", "won", Some(&hot));
    assert_eq!(value(&gs, "heat"), before + HOT);
}

#[test]
fn a_recipe_run_checks_only_recipe_triggers_and_inputs_not_a_full_tick() {
    let triggers = json!([
        {"id": "t_any", "name": "Any", "on": "any", "effects": [{"effect": "set_flag", "key": "any_seen"}]},
        {"id": "t_recipe", "name": "Recipe", "on": "recipe", "effects": [{"effect": "set_flag", "key": "recipe_seen"}]}
    ]);
    // a fixed-deadline checkpoint already past its day: only a tick's checkpoint step may miss it
    let cp = json!([{"id": "cp_due", "name": "Due", "act": 1, "deadline": 1, "met_when": [{"key": "never", "value": true}], "mode": "go_on",
                     "effects": [{"effect": "set_flag", "key": "due_missed"}]}]);
    let d = data(triggers, cp);
    let mut gs = state(&d);
    gs.time = 5;
    let res = recipe::run(&d, &mut gs, SMITH, "lean_down", &pick());
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(fired(&res), ["t_recipe"], "the recipe trigger, not the any trigger whose when holds");
    assert!(gs.flag_true("recipe_seen") && !gs.flag_true("any_seen"));
    let c = gs.campaign.as_ref().unwrap();
    assert!(c.checkpoints.is_empty(), "no checkpoint step");
    assert_eq!(gs.time, 5, "no time change");
    // the next real tick runs the any trigger and the checkpoint step as before
    let events = campaign::tick(&d, &mut gs, &Cause::Effects);
    assert!(events.iter().map(json::dict).any(|e| json::s(e, "trigger") == "t_any"));
    assert_eq!(gs.campaign.as_ref().unwrap().checkpoints.get("cp_due").map(String::as_str), Some("missed"));
}

#[test]
fn a_recipe_input_moves_its_gauge_by_the_recipes_tags() {
    let d = data(json!([]), json!([]));
    let mut gs = state(&d);
    recipe::run(&d, &mut gs, SMITH, "pull", &pick());
    assert_eq!(value(&gs, "slabbed"), 0, "another tag");
    recipe::run(&d, &mut gs, SMITH, "push", &pick());
    assert_eq!(value(&gs, "slabbed"), SLAB_INPUT, "the tagged recipe's input");
}

// ---- B5: signed unit tracks ----

#[test]
fn a_signed_track_leans_down_to_its_bottom_and_saves_only_a_lean() {
    let d = data(json!([]), json!([]));
    let row = d.cat.unit_tracks.get("lean").unwrap();
    assert_eq!((row.bottom(), row.top()), (-rpgkit_data::TRACK_SIGNED_MAX, rpgkit_data::TRACK_SIGNED_MAX));
    let mut gs = state(&d);
    let neutral = round_trip(&d, &gs, "rpgkit_hooks_lean0");
    assert!(neutral.party.members[0].tracks.is_empty(), "0 is left out");
    for n in 1..=i64::from(rpgkit_data::TRACK_SIGNED_MAX) {
        let res = recipe::run(&d, &mut gs, SMITH, "lean_down", &pick());
        assert!(json::b(&res, "ok", false), "{res:?}");
        assert_eq!(gs.party.members[0].track_stage("lean"), -n);
    }
    let res = recipe::run(&d, &mut gs, SMITH, "lean_down", &pick());
    assert!(!json::b(&res, "ok", true), "refused at the bottom");
    assert!(gs.party.members[0].validate(&d).is_empty());
    let back = round_trip(&d, &gs, "rpgkit_hooks_lean");
    assert_eq!(back.party.members[0].track_stage("lean"), -i64::from(rpgkit_data::TRACK_SIGNED_MAX));
    // a lean past the bottom is held there at load
    let mut m = gs.party.members[0].clone();
    m.tracks.insert("lean".into(), -9);
    assert_eq!(m.sanitize_tracks(&d).len(), 1);
    assert_eq!(m.track_stage("lean"), -i64::from(rpgkit_data::TRACK_SIGNED_MAX));
}

// ---- B4: keep_longer ----

#[test]
fn keep_longer_never_shortens_a_running_clock() {
    let set = |days: i64, keep: bool| {
        let mut fx = json::obj([("effect", "clock_set".into()), ("meter", "clock".into()), ("days", json::int(days))]);
        if keep {
            fx.insert("keep_longer".into(), Value::Bool(true));
        }
        fx
    };
    let d = data(json!([]), json!([]));
    let mut gs = state(&d);
    let mut events = Vec::new();
    campaign::apply_effects(&d, &mut gs, &[set(LONG, true)], &mut events);
    assert_eq!(value(&gs, "clock"), LONG, "a stopped clock sets as any set");
    campaign::apply_effects(&d, &mut gs, &[set(SHORT, true)], &mut events);
    assert_eq!(value(&gs, "clock"), LONG, "the banked time stays");
    campaign::apply_effects(&d, &mut gs, &[set(SHORT, false)], &mut events);
    assert_eq!(value(&gs, "clock"), SHORT, "a plain set shortens");
    campaign::apply_effects(&d, &mut gs, &[set(LONG, true)], &mut events);
    assert_eq!(value(&gs, "clock"), LONG, "a longer keep_longer set extends");
}

// ---- B3: after_flag checkpoints ----

#[test]
fn an_after_flag_checkpoint_is_armed_by_its_flag_and_due_offset_days_later() {
    let cp = json!([{"id": "cp_late", "name": "Late", "act": 1, "deadline": 1, "after_flag": "boss_down", "offset_days": OFFSET,
                     "met_when": [{"key": "done", "value": true}], "mode": "go_on", "effects": [{"effect": "set_flag", "key": "late_missed"}]}]);
    let d = data(json!([]), cp);
    let mut gs = state(&d);
    let tick_days = |gs: &mut GameState, n: i64| {
        gs.time += n;
        campaign::tick(&d, gs, &Cause::Days { n, why: "travel".into() })
    };
    // unarmed: its fixed deadline is not read, nothing is missed
    tick_days(&mut gs, 5);
    let c = gs.campaign.as_ref().unwrap();
    assert!(c.checkpoints.is_empty() && c.checkpoint_armed_at.is_empty());
    assert!(campaign::checkpoint::view(&d, &gs).is_empty(), "an unarmed checkpoint is not shown");
    let plain = round_trip(&d, &gs, "rpgkit_hooks_unarmed");
    assert_eq!(plain, gs);
    let dir = aegis_core::scratch::Scratch::vacant("rpgkit_hooks_unarmed_raw");
    assert!(save::write(&d, &dir, &gs, 1, 1).is_empty());
    let raw: String = std::fs::read_dir(&dir).unwrap().filter_map(|e| std::fs::read_to_string(e.unwrap().path()).ok()).collect();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(!raw.contains("checkpoint_armed_at"), "no armed checkpoint writes no key");
    // the flag arms it at the next tick's day
    gs.set_flag("boss_down", json!(true));
    let armed_day = gs.time + 1;
    campaign::tick(&d, &mut gs, &Cause::Effects);
    assert_eq!(gs.campaign.as_ref().unwrap().checkpoint_armed_at.get("cp_late"), Some(&armed_day));
    let view = campaign::checkpoint::view(&d, &gs);
    assert_eq!(json::i(json::dict(&view[0]), "day", 0), armed_day + OFFSET);
    let back = round_trip(&d, &gs, "rpgkit_hooks_armed");
    assert_eq!(back.campaign.as_ref().unwrap().checkpoint_armed_at, gs.campaign.as_ref().unwrap().checkpoint_armed_at);
    // open through its deadline, missed the day after
    tick_days(&mut gs, OFFSET);
    assert!(gs.campaign.as_ref().unwrap().checkpoints.is_empty(), "on the deadline day");
    tick_days(&mut gs, 1);
    assert_eq!(gs.campaign.as_ref().unwrap().checkpoints.get("cp_late").map(String::as_str), Some("missed"));
    assert!(gs.flag_true("late_missed"));
}
