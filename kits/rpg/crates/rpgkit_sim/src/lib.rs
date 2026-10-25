//! The balance simulator (tools/balance_sim.gd and tools/sim/*, BAL2_SIM.md): headless AI-vs-AI battles on sandbox
//! units built from class + level + tier, in nine modes (duel, tier, levels, group, stats, party, role, sweep,
//! campaign). `run` returns the lines the sim prints; `aegis sim <mode> ...` prints them. Scores per fight (left side):
//! binary (KO, else the HP lead with a 0.02 band) and HP margin (KO 1 / 0, else 0.5 + (l - r) / 2).

pub mod args;
pub mod campaign;
pub mod kpi;
pub mod modes;
pub mod party;
pub mod role_mode;
pub mod roles;
pub mod runner;
pub mod scenarios;
pub mod targets;
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
            return out;
        }
    };
    let mut r = Runner::new(d, &a);
    r.metrics.keep_totals = a.metrics && !args::METRIC_MODES.contains(&a.mode.as_str());
    r.print(a.header());
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
