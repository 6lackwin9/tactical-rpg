//! The game's sim crate: a façade over the RPG kit crate rpgkit_sim (kits/rpg/crates/rpgkit_sim, KIT1). KIT1-SIM: the
//! kit names no world; `game()` tells it Emberwake's default world and story run, and `parse` and `run` pass them.
pub use rpgkit_sim::*;

use emberwake_world::Data;

/// The Ashlands: the default world of `world=` and `extra=`, and the story the `campaign` mode plays.
pub const WORLD: &str = "ashlands";
/// The flag of the "bar the gate" choice that branches the story: arms = ashmaw_stand (full finale), gate =
/// ashmaw_stand_barred.
pub const STEEL_FLAG: &str = "ashlands.steel_to_gate";

/// Emberwake's sim description: the Ashlands as default world and story run, branched on the steel choice; the
/// timeline test encounters for `feel`; its classes, roles and generated rosters (`CLASSES`).
pub fn game() -> SimGame {
    SimGame {
        default_world: WORLD.into(),
        story: Some(StoryRun {
            world: WORLD.into(),
            branch_flag: STEEL_FLAG.into(),
            choice: "steel".into(),
            branches: vec![("arms".into(), false), ("gate".into(), true)],
            usage: "       campaign [level]   Ashlands story fights in quest order, HP carried over (level = party level, default 5): all 10 trios, both steel choices".into(),
            extra_example: "4:ash_raiders".into(),
        }),
        feel_encounters: FEEL_ENCOUNTERS.iter().map(|e| e.to_string()).collect(),
        classes: CLASSES,
    }
}

/// RULESET1-G (RULESET1_TASKS 2.7, 3): the timeline test encounters the `feel` mode fights by default.
pub const FEEL_ENCOUNTERS: [&str; 5] = ["tl_duel_windup", "tl_duel_reach", "tl_duel_armour", "tl_flank", "tl_skirmish"];

/// Parses the sim command line `argv` (mode first) on `d` for Emberwake: `world=` defaults to the Ashlands and the
/// campaign mode plays its story run. Err = the message after "SIM argument error: ".
pub fn parse(d: &Data, argv: &[String]) -> Result<args::SimArgs, String> {
    args::SimArgs::parse_for(d, argv, &game())
}

/// Runs the sim command line `argv` (mode first, as `tools/fc.sh sim` takes it) on `d` for Emberwake; the printed
/// lines.
pub fn run(d: &Data, argv: &[String]) -> Vec<String> {
    rpgkit_sim::run(d, argv, &game())
}

/// KITIDS1 (SILOS1 T4): Emberwake's classes as the sim plays them (BAL2_SIM.md sections 5, 6.2-6.4): the table order,
/// the party default, the campaign trio lead (the warrior plus every pair of the other five), the generated elite and
/// boss, the support baseline, the roles and role loadouts, the generated rosters and the role limit tests.
pub const CLASSES: SimClasses = SimClasses {
    ids: &["warrior", "ranger", "priest", "brawler", "rogue", "mage"],
    default_comp: &["warrior", "ranger", "priest", "mage"],
    trio_lead: "warrior",
    elite: "warrior",
    boss: "brawler",
    support_baseline: "brawler",
    roles: &ROLES,
    generated: &GENERATED,
    scenarios: &SCENARIOS,
    s2_enemies: &[("ranger", "standard"), ("brawler", "standard"), ("mage", "standard"), ("rogue", "standard")],
};

/// 6.2 (rogue = melee striker, brawler = melee AoE) and the 6.3 role loadouts at L5 with their L7 swaps.
const ROLES: [roles::ClassRoles; 6] = [
    roles::ClassRoles {
        class: "warrior",
        primary: "tank",
        secondary: &["support"],
        loadouts: &[
            ("tank", &["taunt_shout", "shield_bash", "intimidate", "shield_wall"]),
            ("support", &["taunt_shout", "intimidate", "shield_bash", "sunder", "riposte"]),
        ],
        l7_swaps: &[],
    },
    roles::ClassRoles {
        class: "ranger",
        primary: "ranged",
        secondary: &["aoe"],
        loadouts: &[
            ("ranged", &["aimed_shot", "pinning_shot", "leap_back", "barbed_arrow", "deadeye"]),
            ("aoe", &["aimed_shot", "pinning_shot", "leap_back", "volley", "deadeye"]),
        ],
        l7_swaps: &[],
    },
    roles::ClassRoles {
        class: "priest",
        primary: "support",
        secondary: &["striker"],
        loadouts: &[
            ("support", &["rally", "mend", "purify", "greater_mend", "renewal"]),
            ("striker", &["smite", "rally", "hex", "mend", "greater_mend"]),
        ],
        l7_swaps: &[("striker", "mend", "judgement")],
    },
    roles::ClassRoles {
        class: "brawler",
        primary: "aoe",
        secondary: &["tank", "striker"],
        loadouts: &[
            ("aoe", &["body_blow", "lunge", "sweep_kick", "cyclone_kick", "haymaker"]),
            ("tank", &["body_blow", "lunge", "sweep_kick", "cyclone_kick", "haymaker"]),
            ("striker", &["body_blow", "lunge", "charge_in", "hamstring", "haymaker"]),
        ],
        l7_swaps: &[],
    },
    roles::ClassRoles {
        class: "rogue",
        primary: "striker",
        secondary: &[],
        loadouts: &[("striker", &["quick_stab", "pounce", "lacerate", "backflip", "hamstring"])],
        l7_swaps: &[("striker", "hamstring", "assassinate")],
    },
    roles::ClassRoles {
        class: "mage",
        primary: "aoe",
        secondary: &["ranged"],
        loadouts: &[
            ("aoe", &["arcane_bolt", "arc_lance", "firebomb", "glacial_burst", "focus"]),
            ("ranged", &["arcane_bolt", "arc_lance", "hex", "focus", "firebomb"]),
        ],
        l7_swaps: &[("aoe", "focus", "inferno")],
    },
];

/// 5: the generated encounters ("@elite" / "@boss": elite= / boss=).
const GENERATED: [scenarios::Generated; 5] = [
    scenarios::Generated { id: "mob", enemies: &[("brawler", "minion"), ("rogue", "minion"), ("ranger", "minion"), ("priest", "minion"), ("brawler", "minion")], wave: &[] },
    scenarios::Generated { id: "line", enemies: &[("warrior", "standard"), ("brawler", "standard"), ("ranger", "standard"), ("rogue", "standard")], wave: &[] },
    scenarios::Generated {
        id: "wave",
        enemies: &[("brawler", "standard"), ("rogue", "standard"), ("ranger", "standard")],
        wave: &[("brawler", "minion"), ("rogue", "minion"), ("priest", "minion")],
    },
    scenarios::Generated { id: "elite", enemies: &[("@elite", "elite"), ("rogue", "standard"), ("mage", "standard"), ("warrior", "standard")], wave: &[] },
    scenarios::Generated { id: "boss", enemies: &[("@boss", "boss"), ("ranger", "standard"), ("priest", "standard")], wave: &[("ranger", "minion"), ("priest", "minion")] },
];

/// 6.4: the five role limit tests.
const SCENARIOS: [scenarios::Scenario; 5] = [
    scenarios::Scenario {
        id: "tank_hold",
        role: "tank",
        allies: &["ranger", "mage"],
        enemies: &[("brawler", "standard"), ("rogue", "standard"), ("brawler", "standard"), ("rogue", "standard")],
        priority: false,
        block: (0, 0),
        tier_until: None,
    },
    scenarios::Scenario {
        id: "ranged_dive",
        role: "ranged",
        allies: &["warrior"],
        enemies: &[("rogue", "standard"), ("rogue", "standard"), ("brawler", "standard")],
        priority: false,
        block: (0, 0),
        tier_until: None,
    },
    scenarios::Scenario {
        id: "aoe_swarm",
        role: "aoe",
        allies: &["warrior", "priest"],
        enemies: &[("brawler", "minion"), ("rogue", "minion"), ("brawler", "minion"), ("brawler", "minion"), ("rogue", "minion"), ("priest", "minion")],
        priority: false,
        block: (2, 3),
        tier_until: None,
    },
    scenarios::Scenario {
        id: "support_attrition",
        role: "support",
        allies: &["warrior", "brawler"],
        enemies: &[("ranger", "standard"), ("brawler", "standard"), ("mage", "standard"), ("rogue", "standard")],
        priority: false,
        block: (0, 0),
        tier_until: None,
    },
    scenarios::Scenario {
        id: "striker_priority",
        role: "striker",
        allies: &["warrior"],
        enemies: &[("mage", "elite"), ("warrior", "standard"), ("warrior", "standard")],
        priority: true,
        block: (0, 0),
        tier_until: Some((3, 0, "standard")),
    },
];
