//! The balance simulator (tools/balance_sim.gd and tools/sim/*, BAL2_SIM.md): headless AI-vs-AI battles on sandbox
//! units built from class + level + tier, in nine modes (duel, tier, levels, group, stats, party, role, sweep,
//! campaign). `run` returns the lines the sim prints; `aegis sim <mode> ...` prints them. Scores per fight (left side):
//! binary (KO, else the HP lead with a 0.02 band) and HP margin (KO 1 / 0, else 0.5 + (l - r) / 2). RULESET1-G adds a
//! tenth, engine-only mode, `feel` (feel.rs): catalog encounters with their party (WORLD-SIM1: or world encounters with
//! their authored party), read for how the rules feel.
//! CAMPAIGN1-SIM adds an eleventh, engine-only and fightless, `deadlines` (deadlines.rs): the best and worst day each
//! campaign checkpoint can be met.

pub mod args;
pub mod campaign;
pub mod config;
pub mod deadlines;
pub mod feel;
pub mod kpi;
pub mod modes;
pub mod party;
pub mod role_mode;
pub mod roles;
pub mod runner;
pub mod scenarios;
pub mod targets;
pub mod timed;
pub mod variants;
pub mod world;

use args::SimArgs;
use rpgkit_world::Data;
use runner::Runner;

/// What a game tells the sim (KIT1-SIM): the kit names no world, flag or story; the game's sim crate passes them.
#[derive(Debug, Clone, Default)]
pub struct SimGame {
    /// The world of `world=` (bare enc= ids, `enc=world:<id>`) and of the `extra=` check.
    pub default_world: String,
    /// The story the `campaign` mode plays; None: the mode prints that the game has none.
    pub story: Option<StoryRun>,
    /// RULESET1-G: the catalog encounters the `feel` mode fights without `enc=` (empty: `feel` needs `enc=`).
    pub feel_encounters: Vec<String>,
    /// KITIDS1: the classes the sim fights with, their roles and the generated rosters (empty: the class modes refuse
    /// to run).
    pub classes: SimClasses,
}

/// KITIDS1 (SILOS1 T4): the game's classes as the sim plays them; the kit names no class id. Static tables, so a game
/// writes them as constants.
#[derive(Debug, Clone, Copy, Default)]
pub struct SimClasses {
    /// The classes in table order: duel, tier, levels, group and stats rows, sweep pools, class=all, the campaign trios.
    pub ids: &'static [&'static str],
    /// comp= default (party mode).
    pub default_comp: &'static [&'static str],
    /// The campaign trios: this class plus every pair of the other ids.
    pub trio_lead: &'static str,
    /// elite= and boss= defaults: the classes of the generated "@elite" and "@boss" units.
    pub elite: &'static str,
    pub boss: &'static str,
    /// The role mode's support baseline: this class as a plain reference in the support slot.
    pub support_baseline: &'static str,
    /// Each class's roles and role loadouts (a class without a row plays roles::DEFAULT_ROLE with no loadout).
    pub roles: &'static [roles::ClassRoles],
    /// The rosters of the generated encounters (args::GENERATED ids).
    pub generated: &'static [scenarios::Generated],
    /// The role limit tests (scenarios::role_scenario ids).
    pub scenarios: &'static [scenarios::Scenario],
    /// BAL5 S2: the support_attrition foes under `s2=on`.
    pub s2_enemies: scenarios::Roster,
}

/// The modes that fight or list the game's classes (feel plays authored parties; deadlines has no fights; campaign
/// checks its own story first).
const CLASS_MODES: [&str; 8] = ["duel", "tier", "levels", "group", "stats", "party", "role", "sweep"];

/// KITIDS1: why `a` cannot run with the game's classes ("" = it can): a class mode needs classes, a generated
/// encounter its roster, a role its limit test.
pub fn class_problem(a: &SimArgs) -> String {
    let c = &a.game.classes;
    let needs = CLASS_MODES.contains(&a.mode.as_str()) || (a.mode == "campaign" && a.game.story.is_some());
    if needs && c.ids.is_empty() {
        return format!("mode '{}' needs the game's sim classes (SimGame.classes names none)", a.mode);
    }
    if !needs {
        return String::new();
    }
    // Each class default the chosen mode reads (an argument that sets it, such as comp= or elite=, stands in for it)
    let empty = |field: &str| format!("mode '{}' needs SimGame.classes.{field} (the game names none)", a.mode);
    if a.mode == "party" && !a.comp_set && a.comp.is_empty() {
        return empty("default_comp");
    }
    if a.mode == "campaign" && !a.comp_set && c.trio_lead.is_empty() {
        return empty("trio_lead");
    }
    if a.mode == "role" && a.roles.iter().any(|r| r == "support") && c.support_baseline.is_empty() {
        return empty("support_baseline");
    }
    if matches!(a.mode.as_str(), "party" | "sweep") {
        if let Some(e) = a.encounters.iter().find(|e| args::GENERATED.contains(&e.as_str()) && scenarios::generated(c, e).is_none()) {
            return format!("enc: the game gives no roster for the generated encounter '{e}' (SimGame.classes.generated)");
        }
        // the "@elite" / "@boss" slots of the generated rosters in play
        let slots: Vec<&str> = a.encounters.iter().filter_map(|e| scenarios::generated(c, e)).flat_map(|(en, wave)| en.iter().chain(wave.iter()).map(|(class, _)| *class)).collect();
        if slots.contains(&"@elite") && a.elite_class.is_empty() {
            return empty("elite");
        }
        if slots.contains(&"@boss") && a.boss_class.is_empty() {
            return empty("boss");
        }
    }
    if a.mode == "role" {
        if let Some(role) = a.roles.iter().find(|r| scenarios::scenario_def(c, scenarios::role_scenario(r)).is_none()) {
            return format!("role: the game gives no limit test {} for role '{role}' (SimGame.classes.scenarios)", scenarios::role_scenario(role));
        }
        // the role tests fill "@elite" / "@boss" slots with the game's elite and boss
        let slots: Vec<&str> = a.roles.iter().filter_map(|r| scenarios::scenario_def(c, scenarios::role_scenario(r))).flat_map(|s| s.enemies.iter().map(|(class, _)| *class)).collect();
        if slots.contains(&"@elite") && c.elite.is_empty() {
            return empty("elite");
        }
        if slots.contains(&"@boss") && c.boss.is_empty() {
            return empty("boss");
        }
    }
    String::new()
}

/// The story run of the `campaign` mode: the auto-start quest chain of `world`, played once per branch of the
/// choice flag `branch_flag`.
#[derive(Debug, Clone)]
pub struct StoryRun {
    pub world: String,
    pub branch_flag: String,
    /// What the output calls the choice (the word before a branch label in "<choice> <label>").
    pub choice: String,
    /// The branch labels in play order, each with whether the choice flag is set on that branch.
    pub branches: Vec<(String, bool)>,
    /// The usage line of the `campaign` mode.
    pub usage: String,
    /// The `extra=` example of its error message (`<battle>:<encounter>`).
    pub extra_example: String,
}

/// Runs the sim command line `argv` (mode first, as `tools/fc.sh sim` takes it) on `d` for `game`; the printed lines.
pub fn run(d: &Data, argv: &[String], game: &SimGame) -> Vec<String> {
    let a = match SimArgs::parse_for(d, argv, game) {
        Ok(a) => a,
        Err(e) => {
            let mut out = vec![format!("SIM argument error: {e}")];
            out.extend(args::usage(game));
            // SIMCOUNT1: apart from `usage`, which `sim_golden` holds; no golden run asks for help or refuses a feel line
            if matches!(argv.first().map(String::as_str), Some("help" | "feel")) {
                out.push(args::FEEL_COUNTS_USAGE.to_string());
                out.push(args::FEEL_WORLD_USAGE.to_string());
            }
            // BASERULES1-M2: the same, for the ruleset notes (no golden run asks for help)
            if argv.first().map(String::as_str) == Some("help") {
                out.push(args::RULE_NOTES_USAGE.to_string());
                // PR4-1: the same, for `stage=`
                out.push(args::STAGE_USAGE.to_string());
                // PK2-S5b: the same, for `dulled=`
                out.push(args::DULLED_USAGE.to_string());
                out.push(args::DURABILITY_USAGE.to_string());
                // PK2-FOLLOWUP: the same, for `mastery=`
                out.push(args::MASTERY_USAGE.to_string());
            }
            return out;
        }
    };
    let problem = class_problem(&a);
    if !problem.is_empty() {
        let mut out = vec![format!("SIM argument error: {problem}")];
        out.extend(args::usage(game));
        return out;
    }
    // CAMPAIGN1-SIM: no fights, so no runner and its own ARGS line
    if a.mode == "deadlines" {
        return deadlines::deadlines_mode(d, &a);
    }
    let mut r = Runner::new(d, &a);
    r.metrics.keep_totals = a.metrics && !args::METRIC_MODES.contains(&a.mode.as_str());
    r.print(a.header());
    // BASERULES1-M2 (FB-40): the picked ruleset's notes, right under ARGS (none for a clean row)
    for line in &a.rule_notes {
        r.print(line.clone());
    }
    // RULESET1-G: the feel report ends with its FEEL_JSON line (no score footers)
    if a.mode == "feel" {
        feel::feel_mode(&mut r);
        return r.out;
    }
    match a.mode.as_str() {
        "duel" => modes::duel(&mut r, a.level),
        "tier" => modes::tier(&mut r, a.level),
        "levels" => modes::levels(&mut r),
        "group" => modes::group(&mut r, a.level),
        "stats" => modes::stats(&mut r),
        "party" => party::party_mode(&mut r),
        "sweep" => party::sweep_mode(&mut r),
        "role" => role_mode::role_mode(&mut r),
        _ => campaign::campaign_mode(&mut r),
    }
    r.distance_note();
    if r.metrics.keep_totals && !r.metrics.totals.is_empty() {
        r.print("METRICS per unit over every fight of this table (the left/party unit and its foes):");
        r.print(kpi::kpi_header());
        let lines: Vec<String> = r.metrics.totals.iter().map(|(n, acc)| kpi::kpi_line(n, acc)).collect();
        r.out.extend(lines);
    }
    if r.fights > 0 {
        let (checks, fights, fails) = (r.metrics.checks, r.fights, r.metrics.fail_lines.len());
        r.print(format!("metric checks: {checks} over {fights} fights, {fails} METRIC FAIL"));
        let plan_fails = r.plan_fails;
        r.print(format!("plan checks: {plan_fails} PLAN FAIL over {fights} fights"));
    }
    if r.plan_skips > 0 || r.fights > 0 {
        let skips = r.plan_skips;
        r.print(format!("plan skips: {skips} PLAN SKIP fights (jitter, unreachable target; skipped, not failed)"));
    }
    r.out
}
