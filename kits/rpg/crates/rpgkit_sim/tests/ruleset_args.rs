//! RULESET1 3.1.3: the sim picks its fights' ruleset with `ruleset=` (default classic) and switches single extensions
//! with `rules=-id,+id`; options cannot be switched one by one, unknown ids and a ruleset without its row are argument
//! errors. A world encounter without `ruleset=` plays by the ruleset of the world that resolves it; catalog and
//! generated fights stay classic. Scratch data built here.

use rpgkit_rules::ruleset::{Ruleset, CLASSIC, PIERCE, POISE, REACH, ROUND_INTENT, TIMELINE};
use rpgkit_sim::args::SimArgs;
use rpgkit_sim::runner::Runner;
use rpgkit_world::Data;
use serde_json::json;

/// World `vale` (timeline) owns encounter `e_vale`; `glen` (classic) extends it.
fn data(with_row: bool) -> Data {
    let rows = if with_row { json!([{"id": "timeline", "name": "Timeline", "rules": ["round_intent", "poise", "pierce"]}]) } else { json!([]) };
    let worlds = json!({
        "vale": {"manifest": {"id": "vale", "ruleset": "timeline"}, "own": {"encounters": [{"id": "e_vale", "name": "Vale fight", "enemies": []}]}},
        "glen": {"manifest": {"id": "glen", "extends": ["vale"], "ruleset": "classic"}, "own": {}}
    });
    Data::from_values(&json!({"rulesets": rows}), &worlds, &json!({}))
}

fn parse(d: &Data, line: &str) -> Result<SimArgs, String> {
    SimArgs::parse(d, &line.split_whitespace().map(String::from).collect::<Vec<_>>())
}

#[test]
fn the_default_is_classic_and_ruleset_picks_the_row() {
    let d = data(true);
    assert_eq!(parse(&d, "duel").unwrap().ruleset, Ruleset::classic());
    assert_eq!(parse(&d, "duel ruleset=classic").unwrap().ruleset.id, CLASSIC);
    let a = parse(&d, "duel ruleset=timeline").unwrap();
    assert_eq!(a.ruleset.id, TIMELINE);
    assert_eq!(a.ruleset.rules, [ROUND_INTENT, POISE, PIERCE]);
}

#[test]
fn rules_switch_single_extensions_in_any_token_order() {
    let d = data(true);
    let a = parse(&d, "duel rules=-pierce,+reach ruleset=timeline").unwrap();
    assert_eq!(a.ruleset.rules, [ROUND_INTENT, POISE, REACH], "register order");
    let classic = parse(&d, "duel rules=+poise").unwrap();
    assert_eq!((classic.ruleset.id.as_str(), classic.ruleset.rules.clone()), (CLASSIC, vec![POISE]), "an extension on over classic");
    assert!(classic.ruleset.stances(), "an extension leaves the classic stances alone");
}

#[test]
fn bad_picks_are_argument_errors() {
    let d = data(true);
    assert_eq!(parse(&d, "duel ruleset=chess").unwrap_err(), "ruleset must be one of [\"classic\", \"timeline\"] (got 'chess')");
    assert!(parse(&d, "duel ruleset=timeline rules=-round_intent").unwrap_err().contains("'round_intent' is an option"));
    assert!(parse(&d, "duel rules=+chains").unwrap_err().contains("unknown rule 'chains'"));
    assert!(parse(&d, "duel rules=poise").unwrap_err().contains("must be +<rule> or -<rule>"));
    assert!(parse(&data(false), "duel ruleset=timeline").unwrap_err().contains("has no row in rulesets"));
}

/// The ruleset a fight of encounter `e_vale` resolved in `world` plays by under the arguments `line` (None: a catalog
/// or generated fight).
fn fight_ruleset(d: &Data, line: &str, world: Option<&str>) -> Ruleset {
    let a = parse(d, line).unwrap();
    let mut r = Runner::new(d, &a);
    if let Some(w) = world {
        let enc = rpgkit_sim::world::encounter(&mut r, "e_vale", 1, false, w, true).expect("the encounter resolves");
        r.world_ruleset = enc.ruleset;
    }
    r.fight_ruleset()
}

#[test]
fn a_world_encounter_plays_by_its_world_unless_ruleset_is_given() {
    let d = data(true);
    let vale = fight_ruleset(&d, "party", Some("vale"));
    assert_eq!((vale.id.as_str(), vale.rules.clone()), (TIMELINE, vec![ROUND_INTENT, POISE, PIERCE]), "the timeline world's encounter");
    assert_eq!(fight_ruleset(&d, "party", Some("glen")), Ruleset::classic(), "the same row resolved in a classic world");
    assert_eq!(fight_ruleset(&d, "party rules=-pierce,+reach", Some("vale")).rules, [ROUND_INTENT, POISE, REACH], "rules= switch the world's ruleset");
    assert_eq!(fight_ruleset(&d, "party ruleset=classic", Some("vale")), Ruleset::classic(), "ruleset= overrides the world");
    assert_eq!(fight_ruleset(&d, "party", None), Ruleset::classic(), "a catalog or generated fight is classic");
    assert_eq!(fight_ruleset(&d, "party ruleset=timeline", None).id, TIMELINE);
}
