use aegis_vm::{BattleOutcome, CombatSession, Combatant, Team};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
pub struct SimReport {
    pub scenario: String,
    pub iterations: usize,
    pub wins: usize,
    pub losses: usize,
    pub win_rate_pct: f64,
    pub avg_rounds: f64,
    pub avg_remaining_hp_pct: f64,
    pub elapsed_ms: f64,
    pub battles_per_sec: f64,
}

fn main() -> anyhow::Result<()> {
    println!("=== Aegis Headless Monte Carlo Simulation Harness ===");

    let scenarios = [
        ("Goblin Skirmish (Standard Trio)", 1_000),
        ("Bandit Ambush (High Pressure)", 1_000),
        ("Boss Encounter (Ashmaw Stand)", 1_000),
    ];

    let mut reports = Vec::new();
    let total_start = Instant::now();

    for (name, count) in scenarios {
        let report = run_monte_carlo(name, count);
        print_report(&report);
        reports.push(report);
    }

    let total_elapsed = total_start.elapsed().as_secs_f64() * 1000.0;
    println!("=====================================================");
    println!("All simulations completed in {:.2} ms!", total_elapsed);

    Ok(())
}

fn run_monte_carlo(scenario: &str, count: usize) -> SimReport {
    let start_time = Instant::now();

    let mut wins = 0;
    let mut losses = 0;
    let mut total_rounds = 0;
    let mut total_hp_pct = 0.0;

    for i in 0..count {
        let (allies, enemies) = create_scenario_units(scenario);
        let mut session = CombatSession::new(allies, enemies, 1337 + i as u64);

        let mut outcome = BattleOutcome::Ongoing;
        while outcome == BattleOutcome::Ongoing && session.current_round <= 15 {
            outcome = session.resolve_round_autoplay();
        }

        total_rounds += session.current_round;

        let total_max_hp: i32 = session.allies.iter().map(|a| a.max_hp).sum();
        let total_current_hp: i32 = session.allies.iter().map(|a| a.current_hp).sum();
        if total_max_hp > 0 {
            total_hp_pct += (total_current_hp as f64 / total_max_hp as f64) * 100.0;
        }

        match outcome {
            BattleOutcome::Victory => wins += 1,
            BattleOutcome::Defeat | BattleOutcome::Ongoing => losses += 1,
        }
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let elapsed_ms = elapsed * 1000.0;
    let win_rate_pct = (wins as f64 / count as f64) * 100.0;
    let avg_rounds = total_rounds as f64 / count as f64;
    let avg_remaining_hp_pct = total_hp_pct / count as f64;
    let battles_per_sec = count as f64 / elapsed.max(0.0001);

    SimReport {
        scenario: scenario.to_string(),
        iterations: count,
        wins,
        losses,
        win_rate_pct,
        avg_rounds,
        avg_remaining_hp_pct,
        elapsed_ms,
        battles_per_sec,
    }
}

fn create_scenario_units(scenario: &str) -> (Vec<Combatant>, Vec<Combatant>) {
    let allies = vec![
        Combatant::new("warrior", "Warrior", Team::Allies, 60, 16, 4, 10),
        Combatant::new("ranger", "Ranger", Team::Allies, 45, 14, 2, 12),
        Combatant::new("priest", "Priest", Team::Allies, 40, 10, 1, 9),
    ];

    let enemies = match scenario {
        "Bandit Ambush (High Pressure)" => vec![
            Combatant::new("bandit_leader", "Bandit Leader", Team::Enemies, 40, 15, 3, 11),
            Combatant::new("bandit_thug", "Bandit Thug", Team::Enemies, 30, 12, 2, 8),
            Combatant::new("bandit_archer", "Bandit Archer", Team::Enemies, 25, 13, 1, 13),
        ],
        "Boss Encounter (Ashmaw Stand)" => vec![
            Combatant::new("ashmaw", "Ashmaw the Devourer", Team::Enemies, 120, 22, 5, 8),
            Combatant::new("cinder_imp", "Cinder Imp", Team::Enemies, 20, 8, 1, 14),
        ],
        _ => vec![
            Combatant::new("goblin_grunt", "Goblin Grunt", Team::Enemies, 25, 9, 1, 9),
            Combatant::new("goblin_grunt", "Goblin Grunt", Team::Enemies, 25, 9, 1, 9),
            Combatant::new("goblin_shaman", "Goblin Shaman", Team::Enemies, 20, 11, 0, 7),
        ],
    };

    (allies, enemies)
}

fn print_report(r: &SimReport) {
    println!("\nScenario: {}", r.scenario);
    println!("  Iterations:       {}", r.iterations);
    println!("  Win Rate:         {:.1}% ({} wins / {} losses)", r.win_rate_pct, r.wins, r.losses);
    println!("  Average Rounds:   {:.2}", r.avg_rounds);
    println!("  Avg Hero HP Left: {:.1}%", r.avg_remaining_hp_pct);
    println!("  Execution Time:   {:.2} ms ({:.0} battles/sec)", r.elapsed_ms, r.battles_per_sec);
}
