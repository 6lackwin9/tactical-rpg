//! WORLD-SIM1 (PR5-42, FB-239 (a)): `sim feel world:<world>/<id>` on the shipped Act 1 world. Every encounter of the
//! world with an authored `party` parses as a feel encounter, plays by the world's ruleset, fights one seed each with its
//! authored side (the encounter's `party` and `guests`, counted from the data) within its `max_rounds`, and reports the
//! `counts=on` hook counts the CAST1-B sweep reads (CAST1 T10-T14). The shipped data is only read; no number is pinned.

use emberwake_sim::feel::JSON_PREFIX;
use emberwake_sim::runner::Runner;
use emberwake_world::Data;
use serde_json::Value;
use std::path::Path;

/// The Act 1 world (`CAST1_ENCOUNTERS.md`).
const WORLD: &str = "sunk_ward";

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("data").0
}

/// The world's own encounters with an authored party: (reference, party and guest count, max_rounds).
fn authored(d: &Data) -> Vec<(String, usize, i64)> {
    d.own_entries(WORLD, "encounters")
        .iter()
        .filter(|e| !aegis_core::json::sget(e, "party").is_empty())
        .map(|e| {
            let side = aegis_core::json::sget(e, "party").len() + aegis_core::json::sget(e, "guests").len();
            (format!("world:{WORLD}/{}", aegis_core::json::s(e, "id")), side, aegis_core::json::i(e, "max_rounds", 0))
        })
        .collect()
}

#[test]
fn every_authored_world_fight_runs_in_feel_with_its_authored_side() {
    let d = data();
    let fights = authored(&d);
    assert!(!fights.is_empty(), "the world has encounters with an authored party");
    let refs: Vec<String> = fights.iter().map(|(r, _, _)| r.clone()).collect();
    let line = vec!["feel".to_string(), format!("enc={}", refs.join(",")), "seeds=1".into(), "counts=on".into()];
    let out = emberwake_sim::run(&d, &line);
    assert!(!out.iter().any(|l| l.starts_with("SIM ")), "{out:?}");
    let last = out.last().expect("a line");
    let j: Value = serde_json::from_str(last.strip_prefix(JSON_PREFIX).unwrap_or_else(|| panic!("FEEL_JSON last: {last}"))).expect("JSON");
    let world_rules = d.ruleset(WORLD).id;
    assert_eq!(j["ruleset"].as_str(), Some(world_rules.as_str()), "the world's ruleset");
    for (r, _, max) in &fights {
        let e = &j["encounters"][r];
        assert_eq!(e["fights"].as_i64(), Some(1), "{r}");
        assert!(e["median_rounds"].as_f64().unwrap() <= *max as f64, "{r}: within max_rounds {max}");
        assert!(e.get("hooks").is_some(), "{r}: the hook counts");
    }
    // the fight itself: the authored party and guests, as many as the encounter names
    let a = emberwake_sim::parse(&d, &line).expect("parses");
    let mut runner = Runner::new(&d, &a);
    for (r, side, max) in &fights {
        let (_, local) = r.rsplit_once('/').expect("a world reference");
        runner.rounds_override = *max as i32;
        let res = runner.fight_world(WORLD, local, 11);
        assert!(res.ok, "{r}");
        assert_eq!(res.party_size as usize, *side, "{r}: party and guests");
    }
}

#[test]
fn a_positional_world_reference_is_a_feel_encounter() {
    let d = data();
    let (first, _, _) = authored(&d).into_iter().next().expect("an authored fight");
    let a = emberwake_sim::parse(&d, &["feel".to_string(), first.clone()]).expect("parses");
    assert_eq!(a.encounters, [first]);
    assert_eq!(a.ruleset.id, d.ruleset(WORLD).id);
}

/// ACT1-HOOKS2 (FB-248 (c)): a world hold escape keeps its `stay_behind` block in the sims, so its holder and gate play
/// as in a world battle: `sim feel world:` fights the whole row (`authored_fight`), and `sim party` / `campaign`
/// (`world::encounter`, full) copy the block into the fight's base with its objective; a light copy (not full) has
/// none. Every escape of the world is read from the data.
#[test]
fn a_world_hold_escape_keeps_its_stay_behind_in_the_sims() {
    let d = data();
    let filled = |e: &&serde_json::Map<String, Value>| e.get("stay_behind").and_then(Value::as_object).is_some_and(|b| !b.is_empty());
    let escapes: Vec<_> = d.own_entries(WORLD, "encounters").iter().filter(filled).cloned().collect();
    assert!(!escapes.is_empty(), "the world has a stay-behind escape");
    let first = format!("world:{WORLD}/{}", aegis_core::json::s(&escapes[0], "id"));
    let a = emberwake_sim::parse(&d, &["feel".to_string(), first]).expect("parses");
    let mut runner = Runner::new(&d, &a);
    for e in &escapes {
        let id = aegis_core::json::s(e, "id");
        let (def, _) = emberwake_world::battle::authored_fight(&d, WORLD, &id, 0).expect("the feel fight");
        assert_eq!(def.raw.get("stay_behind"), e.get("stay_behind"), "{id}: sim feel");
        let full = emberwake_sim::world::encounter(&mut runner, &id, 1, false, WORLD, true).expect("the sim encounter");
        for key in ["stay_behind", "objective", "objective_count"] {
            assert_eq!(full.base.get(key), e.get(key), "{id}: sim party / campaign {key}");
        }
        let light = emberwake_sim::world::encounter(&mut runner, &id, 1, false, WORLD, false).expect("the sim encounter");
        assert!(light.base.get("stay_behind").is_none(), "{id}: a light copy");
    }
}
