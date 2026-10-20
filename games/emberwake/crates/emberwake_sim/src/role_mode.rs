//! Balance sim `role` mode (tools/sim/sim_role_mode.gd, BAL2_SIM.md sections 6.4 and 8.1): each class runs a role's
//! limit test under spend {class, cons, primary} x preset {balanced, aggressive} (quick: one variant, one seed); every
//! KPI prints as "mean [floor..ceiling]" over the variants, with X for a primary class's range outside the band, a
//! missed ranking and a primary clear rate outside the scenario band.

use crate::roles::{self, Kpi};
use crate::runner::Runner;
use crate::scenarios::{self, FightSet};
use crate::targets;
use aegis_core::fmt::fixed;
use serde_json::Map;

const GRID_SPENDS: [&str; 3] = ["class", "cons", "primary"];
const GRID_PRESETS: [&str; 2] = ["balanced", "aggressive"];
/// 8.1 support band: survivors versus this class in the slot.
const SUPPORT_BASELINE_CLASS: &str = "brawler";

fn spends(r: &Runner) -> Vec<&'static str> {
    if r.args.quick {
        GRID_SPENDS[..1].to_vec()
    } else {
        GRID_SPENDS.to_vec()
    }
}

fn presets(r: &Runner) -> Vec<&'static str> {
    if r.args.quick {
        GRID_PRESETS[..1].to_vec()
    } else {
        GRID_PRESETS.to_vec()
    }
}

/// The variant records of `class_id` in a scenario (one per spend x preset). `plain`: the test slot is a reference
/// unit (the support baseline).
fn run_class(r: &mut Runner, scenario_id: &str, class_id: &str, plain: bool) -> Vec<FightSet> {
    let a = r.args;
    let sc = scenarios::scenario(r, scenario_id, class_id, a.level, plain, a.s2);
    let mut variants = Vec::new();
    r.enemy_block = scenarios::scenario_def(scenario_id).block;
    for sv in spends(r) {
        r.spend_variant = sv.into();
        for pv in presets(r) {
            r.preset = pv.into();
            let mut acc = FightSet::default();
            scenarios::run_seeds(r, &sc.party, &sc.enemies, &Map::new(), &a.seeds(), &sc.priority, &mut acc);
            if acc.fights <= 0.0 {
                continue;
            }
            acc.test = scenarios::sum_units(&acc, std::slice::from_ref(&sc.test), false);
            acc.party = scenarios::sum_units(&acc, &sc.party, false);
            acc.refs = scenarios::sum_units(&acc, &sc.refs, false);
            acc.refs_won = scenarios::sum_units(&acc, &sc.refs, true);
            acc.cap = a.cap;
            variants.push(acc);
        }
    }
    r.spend_variant = "class".into();
    r.preset = "balanced".into();
    r.enemy_block = (0, 0);
    variants
}

pub fn role_mode(r: &mut Runner) {
    let a = r.args;
    r.print(format!(
        "ROLE level {}: variants spend {} x preset {} x {} seeds; KPI cells mean [floor..ceiling] over the variants; P primary, S secondary; X = outside the section 8.1 band (primary) or ranking miss",
        a.level,
        spends(r).join(","),
        presets(r).join(","),
        a.seed_count
    ));
    let mut baseline = -1.0;
    for role in a.roles.clone() {
        let scenario_id = scenarios::role_scenario(&role);
        if role == "support" && baseline < 0.0 {
            let base_runs = run_class(r, scenario_id, SUPPORT_BASELINE_CLASS, true);
            baseline = roles::spread("survivors", &base_runs).mean;
            r.print(format!("support baseline: reference {SUPPORT_BASELINE_CLASS} in the support slot of {scenario_id}: {} survivors per fight", fixed(baseline, 2)));
        }
        let ks = roles::kpis(&role);
        r.print(format!("{scenario_id} ({}) - {}", roles::role_label(&role), bands_line(r, &ks)));
        let mut header = format!("{:<8} {:<2} {:<18} {:<20}", "class", "", format!("clear ({})", targets::band_text(&a.targets, "scenario_clear")), "survivors");
        for k in &ks {
            header += &format!(" {:<22}", k.label);
        }
        r.print(format!("{header} rank"));
        let mut results: Vec<(String, Vec<FightSet>)> = Vec::new();
        let mut means: Vec<(String, f64)> = Vec::new();
        for class_id in a.classes.clone() {
            let mut runs = run_class(r, scenario_id, &class_id, false);
            if role == "support" {
                for v in runs.iter_mut() {
                    v.baseline_survivors = baseline;
                }
            }
            means.push((class_id.clone(), roles::spread(ks[0].id, &runs).mean));
            results.push((class_id, runs));
        }
        let ranks = roles::rank_marks(&role, &means);
        for (class_id, runs) in &results {
            if runs.is_empty() {
                r.print(format!("{class_id:<8} n/a (every variant refused)"));
                continue;
            }
            let fit = roles::fit(class_id, &role);
            let primary = fit == "P";
            let clear_s = roles::spread("clear", runs);
            let mut clear_text = roles::cell(true, "scenario_clear", clear_s, &a.targets, false);
            if primary {
                clear_text += targets::mark(&a.targets, "scenario_clear", clear_s.mean);
            }
            let surv = roles::cell(false, "", roles::spread("survivors", runs), &a.targets, false);
            let mut line = format!("{:<8} {:<2} {:<18} {:<20}", class_id, fit, clear_text, surv);
            for k in &ks {
                line += &format!(" {:<22}", roles::cell(k.pct, k.band, roles::spread(k.id, runs), &a.targets, primary));
            }
            let rank = ranks.iter().find(|(c, _)| c == class_id).map(|(_, t)| t.clone()).unwrap_or_default();
            r.print(format!("{line} {rank}"));
        }
        let loadouts: Vec<String> = a.classes.iter().map(|c| format!("{c} [{}]", roles::role_loadout(&r.d.cat, c, &role, a.level).join(","))).collect();
        r.print(format!("loadouts: {}", loadouts.join("; ")));
    }
    r.print(format!("block misses: {}", r.block_misses));
    if r.block_misses > 0 {
        r.print(format!("BLOCK MISS: {} fights where the enemy block found no room (enemies stayed put); results may be skewed", r.block_misses));
    }
}

fn bands_line(r: &Runner, ks: &[Kpi]) -> String {
    let parts: Vec<String> = ks.iter().filter(|k| !k.band.is_empty()).map(|k| format!("{} {}", k.label, targets::band_text(&r.args.targets, k.band))).collect();
    format!("bands: {}; rank on {} ({} first)", parts.join(", "), ks[0].label, if ks[0].lower { "lowest" } else { "highest" })
}
