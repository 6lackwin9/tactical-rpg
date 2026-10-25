//! Balance sim `party` and `sweep` modes (tools/sim/sim_party.gd, BAL2_SIM.md sections 5, 7, 8.2): asymmetric party
//! fights (seeds only, no side swap) over the spend x preset arguments; values outside their band are marked X.

use crate::args::{self, CLASSES};
use crate::kpi;
use crate::runner::Runner;
use crate::scenarios::{self, FightSet};
use crate::targets;
use crate::world;
use aegis_core::fmt::fixed;
use aegis_core::rng::GodotRng;
use rpgkit_rules::stats::MAX_LEVEL;

/// Sweep dupes=on sample=n draw.
pub const SAMPLE_SEED: u64 = 7;
pub const TOP_N: usize = 5;

/// One composition against one encounter over every spend variant, preset and seed.
pub fn run_comp(r: &mut Runner, comp: &[String], enc_id: &str) -> FightSet {
    let a = r.args;
    let party = scenarios::define_party(r, comp, a.level);
    let world_enc = args::parse_ref(enc_id, &a.world).and_then(|(w, id)| world::encounter(r, &id, (a.level + a.elevel).clamp(1, MAX_LEVEL), a.elevel_set, &w, true));
    let is_world = world_enc.is_some();
    let enc = match world_enc {
        Some(e) => e,
        None => scenarios::encounter(r, enc_id, a.level, a.elevel, a.elevel_set, &a.elite_class, &a.boss_class, a.boss_phase),
    };
    r.objective_wins = is_world;
    r.rounds_override = enc.max_rounds;
    r.key_kills = enc.key_kills.clone();
    r.world_ruleset = enc.ruleset.clone();
    let mut acc = FightSet { purse_mean: enc.purse.map_or(0.0, |(lo, hi)| (lo + hi) as f64 / 2.0), ..FightSet::default() };
    for sv in a.spend_variants() {
        r.spend_variant = sv;
        for pv in a.preset_variants() {
            r.preset = pv;
            scenarios::run_seeds(r, &party, &enc.enemies, &enc.base, &a.seeds(), "", &mut acc);
        }
    }
    r.spend_variant = "class".into();
    r.preset = "balanced".into();
    r.objective_wins = false;
    r.rounds_override = 0;
    r.key_kills.clear();
    r.world_ruleset = None;
    acc.party_ids = party;
    acc
}

/// "" or " X" for a clear % against the encounter's band (generated encounters only).
pub fn clear_mark(r: &Runner, enc_id: &str, pct: f64) -> &'static str {
    let key = format!("clear_{enc_id}");
    if targets::has(&key) {
        targets::mark(&r.args.targets, &key, pct)
    } else {
        ""
    }
}

pub fn band_of(r: &Runner, enc_id: &str) -> String {
    let key = format!("clear_{enc_id}");
    if targets::has(&key) {
        targets::band_text(&r.args.targets, &key)
    } else {
        "-".into()
    }
}

pub fn party_mode(r: &mut Runner) {
    let a = r.args;
    let (spends, presets) = (a.spend_variants(), a.preset_variants());
    r.print(format!(
        "PARTY level {}, comp {} ({} of PARTY_MAX {}), {} fights per encounter (spend {} x preset {} x {} seeds, no side swap)",
        a.level,
        a.comp.join(","),
        a.comp.len(),
        rpgkit_world::data::PARTY_MAX,
        spends.len() * presets.len() * a.seed_count,
        spends.join(","),
        presets.join(","),
        a.seed_count
    ));
    r.print(format!("{:<16} {:>8} {:>7} {:>7} {:>10} {:>9} {:>9}", "encounter", "clear%", "band", "rounds", "surv/size", "party_hp", "hp_score"));
    for enc_id in a.encounters.clone() {
        let acc = run_comp(r, &a.comp, &enc_id);
        let pct = scenarios::clear_pct(&acc);
        let rounds = if acc.cleared > 0.0 { fixed(acc.rounds_cleared_sum / acc.cleared, 1) } else { "-".into() };
        let clear = format!("{}{}", pct.round() as i64, clear_mark(r, &enc_id, pct));
        let surv = format!("{}/{}", fixed(scenarios::mean(&acc, acc.survivors_sum), 1), a.comp.len());
        let line = format!(
            "{:<16} {:>8} {:>7} {:>7} {:>10} {:>9} {:>9}",
            enc_id,
            clear,
            band_of(r, &enc_id),
            rounds,
            surv,
            fixed(scenarios::mean(&acc, acc.party_hp_sum), 2),
            fixed(scenarios::mean(&acc, acc.hp_sum), 2)
        );
        r.print(line);
        if acc.skipped > 0.0 {
            r.print(format!("  {enc_id}: {} fights skipped (PLAN SKIP, jitter); the cell is over {} fights", acc.skipped.round() as i64, acc.fights.round() as i64));
        }
        if a.rewards {
            r.print(format!("  {enc_id} rewards: {}", acc.rewards.line(acc.purse_mean)));
        }
        if a.metrics {
            print_units(r, &acc);
        }
    }
}

/// Per-unit KPI block of a fight set (party units first, then enemies).
pub fn print_units(r: &mut Runner, acc: &FightSet) {
    r.print(kpi::kpi_header());
    for party_pass in [true, false] {
        for (n, u) in &acc.units {
            if acc.party_ids.contains(n) == party_pass {
                r.print(kpi::kpi_line(n, u));
            }
        }
    }
}

/// size-class compositions (the 15 distinct 4-class ones by default; with dupes the multisets; sample = n drawn,
/// seed 7). lead=<class>: the lead first, the rest from the other classes.
pub fn compositions(r: &Runner) -> Vec<Vec<String>> {
    let a = r.args;
    let mut comps = Vec::new();
    if a.lead.is_empty() {
        combine(&args::strings(&CLASSES), a.comp_size, 0, &[], a.dupes, &mut comps);
    } else {
        let pool: Vec<String> = CLASSES.iter().filter(|c| a.dupes || **c != a.lead).map(|c| c.to_string()).collect();
        combine(&pool, a.comp_size, 0, std::slice::from_ref(&a.lead), a.dupes, &mut comps);
    }
    if a.dupes && a.sample > 0 && a.sample < comps.len() {
        let mut rng = GodotRng::new(SAMPLE_SEED);
        let mut idx: Vec<usize> = (0..comps.len()).collect();
        let mut i = idx.len();
        while i > 1 {
            i -= 1;
            let j = rng.randi_range(0, i as i32) as usize;
            idx.swap(i, j);
        }
        let mut keep: Vec<usize> = idx[..a.sample].to_vec();
        keep.sort();
        comps = keep.into_iter().map(|i| comps[i].clone()).collect();
    }
    comps
}

fn combine(pool: &[String], k: usize, start: usize, current: &[String], dupes: bool, found: &mut Vec<Vec<String>>) {
    if current.len() == k {
        found.push(current.to_vec());
        return;
    }
    for i in start..pool.len() {
        let mut next = current.to_vec();
        next.push(pool[i].clone());
        combine(pool, k, if dupes { i } else { i + 1 }, &next, dupes, found);
    }
}

struct Row {
    comp: Vec<String>,
    key: String,
    clear: f64,
    hp: f64,
    per_enc: Vec<(String, f64)>,
}

pub fn sweep_mode(r: &mut Runner) {
    let a = r.args;
    let comps = compositions(r);
    let (spends, presets) = (a.spend_variants(), a.preset_variants());
    r.print(format!(
        "SWEEP level {}: {} compositions x {} encounters ({}), {} fights each (spend {} x preset {} x {} seeds)",
        a.level,
        comps.len(),
        a.encounters.len(),
        a.encounters.join(","),
        spends.len() * presets.len() * a.seed_count,
        spends.join(","),
        presets.join(","),
        a.seed_count
    ));
    let mut rows: Vec<Row> = Vec::new();
    for comp in comps {
        let (mut sum_clear, mut sum_hp) = (0.0, 0.0);
        let mut per_enc = Vec::new();
        for enc_id in &a.encounters {
            let acc = run_comp(r, &comp, enc_id);
            let c = scenarios::clear_pct(&acc);
            per_enc.push((enc_id.clone(), c));
            sum_clear += c;
            sum_hp += scenarios::mean(&acc, acc.party_hp_sum);
        }
        let n = a.encounters.len().max(1) as f64;
        rows.push(Row { key: comp.join(","), comp, clear: sum_clear / n, hp: sum_hp / n, per_enc });
    }
    let mut ranked: Vec<usize> = (0..rows.len()).collect();
    ranked.sort_by(|&x, &y| {
        let (a, b) = (&rows[x], &rows[y]);
        if !targets::approx(a.clear, b.clear) {
            return b.clear.partial_cmp(&a.clear).unwrap();
        }
        if !targets::approx(a.hp, b.hp) {
            return b.hp.partial_cmp(&a.hp).unwrap();
        }
        a.key.cmp(&b.key)
    });
    let mut header = format!("{:<4} {:<34} {:>6} {:>5}", "rank", "composition", "clear", "hp");
    for enc_id in &a.encounters {
        header += &format!(" {:>8}", enc_id.chars().take(8).collect::<String>());
    }
    r.print(format!("{header}   (line < {}% marked X)", targets::num(targets::lo(&a.targets, "line_min"))));
    for (i, &k) in ranked.iter().enumerate() {
        let row = &rows[k];
        let mut line = format!("{:<4} {:<34} {:>6} {:>5}", i + 1, row.key, row.clear.round() as i64, fixed(row.hp, 2));
        for (enc_id, v) in &row.per_enc {
            let mark = if enc_id == "line" { targets::mark(&a.targets, "line_min", *v) } else { "" };
            line += &format!(" {:>8}", format!("{}{mark}", v.round() as i64));
        }
        r.print(line);
    }
    sweep_summary(r, &rows, &ranked);
}

fn sweep_summary(r: &mut Runner, rows: &[Row], ranked: &[usize]) {
    let a = r.args;
    let mut means = "encounter clear % (mean over compositions):".to_string();
    for (e, enc_id) in a.encounters.iter().enumerate() {
        let s: f64 = rows.iter().map(|row| row.per_enc[e].1).sum();
        let m = s / rows.len().max(1) as f64;
        means += &format!("  {enc_id} {}{} ({})", m.round() as i64, clear_mark(r, enc_id, m), band_of(r, enc_id));
    }
    r.print(means);
    let by_key: Vec<(String, f64)> = rows.iter().map(|row| (sorted_key(&row.comp), row.clear)).collect();
    r.print(format!("{:<10} {:>12} {:>6} {:>12}", "class", "swap_delta", "pairs", format!("top{TOP_N}_count")));
    for x in CLASSES {
        let (mut delta_sum, mut pairs) = (0.0, 0);
        for row in rows {
            if row.comp.iter().any(|c| c == x) {
                continue;
            }
            let mut seen: Vec<&String> = Vec::new();
            for y in &row.comp {
                if seen.contains(&y) {
                    continue;
                }
                seen.push(y);
                let mut swapped = row.comp.clone();
                let at = swapped.iter().position(|c| c == y).unwrap();
                swapped[at] = x.to_string();
                let key = sorted_key(&swapped);
                if let Some((_, c)) = by_key.iter().find(|(k, _)| *k == key) {
                    delta_sum += c - row.clear;
                    pairs += 1;
                }
            }
        }
        let top = ranked.iter().take(TOP_N).filter(|&&k| rows[k].comp.iter().any(|c| c == x)).count();
        let delta = if pairs > 0 { delta_sum / pairs as f64 } else { 0.0 };
        let delta_text = if pairs > 0 { format!("{}{}{}", if delta >= 0.0 { "+" } else { "" }, fixed(delta, 1), targets::mark(&a.targets, "swap_delta", delta)) } else { "n/a".into() };
        let top_text = format!("{top}{}", targets::mark(&a.targets, "top5_min", top as f64));
        r.print(format!("{:<10} {:>12} {:>6} {:>12}", x, delta_text, pairs, top_text));
    }
}

/// A composition's classes in class order, joined (the sweep's swap lookup).
pub fn sorted_key(comp: &[String]) -> String {
    let mut order: Vec<&String> = comp.iter().collect();
    order.sort_by_key(|c| CLASSES.iter().position(|k| k == c));
    order.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(",")
}
