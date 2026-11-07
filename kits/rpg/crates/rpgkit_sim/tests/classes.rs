//! KITIDS1 (SILOS1 T4): the kit sim names no class; the game passes its classes, roles and rosters (`SimClasses`).
//! Checked on an empty catalog with made-up class ids: without classes a class mode is an argument error, and the
//! defaults, trios, usage and role fits all come from the game's table.

use rpgkit_sim::args::SimArgs;
use rpgkit_sim::roles::{self, ClassRoles};
use rpgkit_sim::scenarios::{Generated, Scenario};
use rpgkit_sim::{campaign, class_problem, party, SimClasses, SimGame, StoryRun};
use rpgkit_world::Data;
use serde_json::json;

fn data() -> Data {
    Data::from_values(&json!({}), &json!({}), &json!({}))
}

fn argv(line: &str) -> Vec<String> {
    line.split_whitespace().map(String::from).collect()
}

const ROLES: [ClassRoles; 2] = [
    ClassRoles { class: "lancer", primary: "striker", secondary: &["tank"], loadouts: &[("striker", &["jab"])], l7_swaps: &[("striker", "jab", "thrust")] },
    ClassRoles { class: "seer", primary: "support", secondary: &[], loadouts: &[], l7_swaps: &[] },
];

const CLASSES: SimClasses = SimClasses {
    ids: &["lancer", "seer", "warden", "slinger"],
    default_comp: &["seer", "lancer"],
    trio_lead: "warden",
    elite: "lancer",
    boss: "seer",
    support_baseline: "slinger",
    roles: &ROLES,
    generated: &[
        Generated { id: "mob", enemies: &[("lancer", "minion")], wave: &[] },
        Generated { id: "elite", enemies: &[("@elite", "elite")], wave: &[] },
        Generated { id: "boss", enemies: &[("seer", "standard")], wave: &[("@boss", "boss")] },
    ],
    scenarios: &[Scenario { id: "tank_hold", role: "tank", allies: &[], enemies: &[("seer", "standard")], priority: false, block: (0, 0), tier_until: None }],
    s2_enemies: &[],
};

fn game() -> SimGame {
    SimGame { classes: CLASSES, ..SimGame::default() }
}

#[test]
fn a_game_without_classes_gets_an_argument_error() {
    let d = data();
    for line in ["duel 1 1", "stats 1", "sweep 1 1", "role 1 quick"] {
        let out = rpgkit_sim::run(&d, &argv(line), &SimGame::default());
        let mode = line.split(' ').next().unwrap_or_default();
        assert_eq!(out[0], format!("SIM argument error: mode '{mode}' needs the game's sim classes (SimGame.classes names none)"), "{line}");
    }
    let a = SimArgs::parse(&d, &argv("party 3")).unwrap();
    assert!(a.comp.is_empty() && a.elite_class.is_empty() && a.boss_class.is_empty() && a.classes.is_empty(), "no class default in the kit");
    assert_eq!(SimArgs::parse(&d, &argv("party 3 comp=lancer")).unwrap_err(), "comp: unknown class 'lancer' (one of [])");
}

#[test]
fn the_class_defaults_are_the_games() {
    let d = data();
    let a = SimArgs::parse_for(&d, &argv("party 3"), &game()).unwrap();
    assert_eq!((a.comp.clone(), a.elite_class.as_str(), a.boss_class.as_str()), (vec!["seer".to_string(), "lancer".to_string()], "lancer", "seer"));
    assert_eq!(a.classes, CLASSES.ids);
    assert!(SimArgs::parse_for(&d, &argv("party 3 boss=nobody"), &game()).unwrap_err().starts_with("boss must be a class id of [\"lancer\""));
    let usage = rpgkit_sim::args::usage(&game()).join("\n");
    assert!(usage.contains("comp=c1,c2,.. (seer,lancer)") && usage.contains("elite=<class> (lancer)  boss=<class> (seer)"), "{usage}");
    assert!(usage.contains("(size=3 lead=warden: the 3 trios)"), "3 others: 3 pairs");
    let none = rpgkit_sim::args::usage(&SimGame::default()).join("\n");
    assert!(none.contains("comp=c1,c2,.. (none)") && none.contains("lead=none: the 0 trios"), "{none}");
}

#[test]
fn trios_and_sweep_order_follow_the_games_classes() {
    let trios = campaign::trios(&CLASSES);
    assert_eq!(trios.len(), 3, "the lead plus every pair of the three others");
    assert!(trios.iter().all(|t| t[0] == CLASSES.trio_lead && t.iter().filter(|c| *c == CLASSES.trio_lead).count() == 1));
    assert_eq!(party::sorted_key(CLASSES.ids, &["slinger".into(), "lancer".into()]), "lancer,slinger", "the game's table order");
    let d = data();
    let a = SimArgs::parse_for(&d, &argv("campaign 5"), &game()).unwrap();
    assert!(a.header().contains("comp all trios (warden + 2 of lancer,seer,slinger)"), "{}", a.header());
}

#[test]
fn roles_and_rosters_come_from_the_game() {
    assert_eq!((roles::fit(&CLASSES, "lancer", "striker"), roles::fit(&CLASSES, "lancer", "tank"), roles::fit(&CLASSES, "lancer", "aoe")), ("P", "S", "-"));
    assert_eq!(roles::primary_role(&CLASSES, "slinger"), roles::DEFAULT_ROLE, "a class without a roles row");
    assert_eq!(roles::primary_role(&SimClasses::default(), "lancer"), roles::DEFAULT_ROLE);
    assert!(roles::kpis(&CLASSES, "support").iter().any(|k| k.label == "surv vs slinger"), "the support baseline is the game's");
    let d = data();
    let problem = |line: &str| class_problem(&SimArgs::parse_for(&d, &argv(line), &game()).unwrap());
    assert_eq!(problem("party 3 enc=mob"), "");
    assert_eq!(problem("party 3 enc=line"), "enc: the game gives no roster for the generated encounter 'line' (SimGame.classes.generated)");
    assert_eq!(problem("role 3 role=tank"), "");
    assert_eq!(problem("role 3 role=aoe"), "role: the game gives no limit test aoe_swarm for role 'aoe' (SimGame.classes.scenarios)");
}

/// Each class default a mode reads is refused when the game leaves it empty (and the mode's argument does not set it).
#[test]
fn each_empty_class_default_a_mode_reads_is_refused() {
    let d = data();
    let story = StoryRun { world: String::new(), branch_flag: "f".into(), choice: "c".into(), branches: vec![("a".into(), false)], usage: String::new(), extra_example: String::new() };
    let problem = |classes: SimClasses, line: &str| {
        let g = SimGame { classes, story: Some(story.clone()), ..SimGame::default() };
        class_problem(&SimArgs::parse_for(&d, &argv(line), &g).unwrap())
    };
    let need = |mode: &str, field: &str| format!("mode '{mode}' needs SimGame.classes.{field} (the game names none)");
    assert_eq!(problem(SimClasses { default_comp: &[], ..CLASSES }, "party 3 enc=mob"), need("party", "default_comp"));
    assert_eq!(problem(SimClasses { default_comp: &[], ..CLASSES }, "party 3 enc=mob comp=seer"), "", "comp= stands in");
    assert_eq!(problem(SimClasses { trio_lead: "", ..CLASSES }, "campaign 3"), need("campaign", "trio_lead"));
    assert_eq!(problem(SimClasses { trio_lead: "", ..CLASSES }, "campaign 3 comp=seer"), "", "comp= picks the one party");
    assert_eq!(problem(SimClasses { support_baseline: "", ..CLASSES }, "role 3 role=all"), need("role", "support_baseline"));
    assert_eq!(problem(SimClasses { elite: "", ..CLASSES }, "party 3 enc=elite"), need("party", "elite"));
    assert_eq!(problem(SimClasses { elite: "", ..CLASSES }, "party 3 enc=elite elite=seer"), "", "elite= stands in");
    assert_eq!(problem(SimClasses { elite: "", ..CLASSES }, "party 3 enc=mob"), "", "no elite slot in play");
    assert_eq!(problem(SimClasses { boss: "", ..CLASSES }, "sweep 3 enc=boss"), need("sweep", "boss"), "the wave's boss slot");
    assert_eq!(problem(CLASSES, "party 3 enc=mob,elite,boss"), "");
}
