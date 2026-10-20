//! The sim's per-unit KPI table (tools/sim/sim_metrics.gd): averages of BattleMetrics records summed over fights.

use aegis_core::fmt::fixed;
use indexmap::IndexMap;

/// Field sums of one unit over fights, plus "fights" (BattleMetrics accumulate).
pub type Acc = IndexMap<&'static str, f64>;

fn get(a: &Acc, f: &str) -> f64 {
    a.get(f).copied().unwrap_or(0.0)
}

pub fn per_fight(a: &Acc, field: &str) -> f64 {
    get(a, field) / get(a, "fights").max(1.0)
}

pub fn ratio(a: &Acc, num: &str, den: &str) -> f64 {
    let d = get(a, den);
    if d > 0.0 {
        get(a, num) / d
    } else {
        0.0
    }
}

pub fn kpi_header() -> String {
    format!(
        "  {:<22} {:>6} {:>5} {:>6} {:>4} {:>4} {:>5} {:>6} {:>5} {:>6} {:>4} {:>9} {:>9} {:>11} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5}",
        "unit (per fight)",
        "dealt",
        "rng%",
        "taken",
        "dot",
        "ff",
        "atk_r",
        "heal",
        "ovh",
        "heal_r",
        "cln",
        "buffs/rds",
        "debuf/rds",
        "area c/f/a",
        "kills",
        "surv%",
        "hpend",
        "minhp",
        "saves",
        "dist",
        "adj%",
        "hold%",
        "taunt%"
    )
}

/// One KPI line of an aggregated record ("n/a" for overheal when a heal's skill was not found).
pub fn kpi_line(label: &str, a: &Acc) -> String {
    let blocks = get(a, "blocks_alive");
    let pct = |f: &str| if blocks > 0.0 { ((get(a, f) / blocks * 100.0).round() as i64).to_string() } else { "n/a".into() };
    let ovh = if get(a, "overheal_unknown") > 0.0 { "n/a".to_string() } else { fixed(per_fight(a, "overheal"), 0) };
    let pf = |f: &str, d: usize| fixed(per_fight(a, f), d);
    format!(
        "  {:<22} {:>6} {:>5} {:>6} {:>4} {:>4} {:>5} {:>6} {:>5} {:>6} {:>4} {:>4}/{:<4} {:>4}/{:<4} {:>3}/{:>3}/{:>3} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5}",
        label,
        pf("dmg_dealt", 0),
        (ratio(a, "dmg_dealt_ranged", "dmg_dealt") * 100.0).round() as i64,
        pf("dmg_taken", 0),
        pf("dot_taken", 0),
        pf("ff_taken", 0),
        pf("attacks_received", 1),
        pf("heal_done", 0),
        ovh,
        pf("heal_received", 0),
        pf("cleanses", 1),
        pf("buffs_applied", 1),
        pf("buff_rounds", 1),
        pf("debuffs_applied", 1),
        pf("debuff_rounds", 1),
        pf("area_casts", 1),
        pf("area_foes_hit", 1),
        pf("area_allies_hit", 1),
        pf("kills", 2),
        (per_fight(a, "survived") * 100.0).round() as i64,
        pf("hp_end_frac", 2),
        pf("min_hp_frac", 2),
        pf("near_death_saves", 2),
        fixed(ratio(a, "dist_sum", "dist_samples"), 2),
        pct("blocks_adjacent"),
        pct("hold_blocks"),
        pct("taunt_blocks")
    )
}
