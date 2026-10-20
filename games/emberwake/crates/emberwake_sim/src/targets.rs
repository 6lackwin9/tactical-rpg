//! Balance sim targets (tools/sim/sim_targets.gd, BAL2_SIM.md section 8): closed bands [lo, hi] in the unit each
//! table prints; `target=<key>:<lo>-<hi>` overrides one. Report tables mark a value outside its band with X.

use aegis_core::fmt::fixed;
use indexmap::IndexMap;

/// Upper bound of "at least lo" bands.
pub const OPEN_HIGH: f64 = 999.0;

pub type Targets = IndexMap<String, (f64, f64)>;

pub const DEFAULTS: [(&str, f64, f64); 26] = [
    // 8.2 party clear % at equal level
    ("clear_mob", 90.0, 100.0),
    ("clear_line", 70.0, 90.0),
    ("clear_wave", 55.0, 80.0),
    ("clear_elite", 50.0, 75.0),
    ("clear_boss", 30.0, 50.0),
    // 8.2 sweep limits
    ("swap_delta", -10.0, 10.0),
    ("line_min", 50.0, 100.0),
    ("top5_min", 2.0, 5.0),
    // 8.3 duel diagnostic
    ("duel", 30.0, 70.0),
    // 8.1 role bands
    ("tank_taken_share", 40.0, 65.0),
    ("tank_attacks_share", 45.0, 100.0),
    ("tank_survival", 60.0, 100.0),
    ("tank_ally_hp", 40.0, 100.0),
    ("tank_ally_hp_all", 15.0, 100.0),
    ("ranged_dist", 2.5, OPEN_HIGH),
    ("ranged_share", 70.0, 100.0),
    ("ranged_attacks_per_round", 0.0, 1.0),
    ("aoe_foes_per_cast", 2.0, OPEN_HIGH),
    ("aoe_allies_per_cast", 0.0, 0.3),
    ("aoe_swarm_clear", 70.0, 100.0),
    ("support_heal_share", 20.0, 100.0),
    ("support_saves", 0.3, OPEN_HIGH),
    ("support_survivor_gain", 0.5, OPEN_HIGH),
    ("striker_down_by_5", 60.0, 100.0),
    ("striker_priority_share", 50.0, 100.0),
    ("scenario_clear", 50.0, 85.0),
];

pub fn defaults() -> Targets {
    DEFAULTS.iter().map(|(k, l, h)| (k.to_string(), (*l, *h))).collect()
}

pub fn has(key: &str) -> bool {
    DEFAULTS.iter().any(|(k, _, _)| *k == key)
}

pub fn lo(t: &Targets, key: &str) -> f64 {
    t.get(key).map_or(0.0, |b| b.0)
}

pub fn hi(t: &Targets, key: &str) -> f64 {
    t.get(key).map_or(OPEN_HIGH, |b| b.1)
}

pub fn inside(t: &Targets, key: &str, v: f64) -> bool {
    v >= lo(t, key) - 0.0001 && v <= hi(t, key) + 0.0001
}

/// True when [floor_v, ceil_v] overlaps the band of `key` (role bands).
pub fn overlaps(t: &Targets, key: &str, floor_v: f64, ceil_v: f64) -> bool {
    ceil_v >= lo(t, key) - 0.0001 && floor_v <= hi(t, key) + 0.0001
}

/// "" inside the band, " X" outside.
pub fn mark(t: &Targets, key: &str, v: f64) -> &'static str {
    if inside(t, key, v) {
        ""
    } else {
        " X"
    }
}

/// Godot's is_equal_approx.
pub fn approx(a: f64, b: f64) -> bool {
    if a == b {
        return true;
    }
    let tolerance = (a.abs() * 1e-5).max(1e-5);
    (a - b).abs() < tolerance
}

/// Band text for a header: "70-90", ">=90", "<=1.0", "+-10".
pub fn band_text(t: &Targets, key: &str) -> String {
    let (l, h) = (lo(t, key), hi(t, key));
    if l < 0.0 && approx(l, -h) {
        return format!("+-{}", num(h));
    }
    if h >= OPEN_HIGH || (h == 100.0 && l > 0.0) {
        return format!(">={}", num(l));
    }
    if l <= 0.0 {
        return format!("<={}", num(h));
    }
    format!("{}-{}", num(l), num(h))
}

pub fn num(v: f64) -> String {
    if approx(v, v.round()) {
        (v.round() as i64).to_string()
    } else {
        fixed(v, 1)
    }
}

fn is_float(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit() || b"+-.eE".contains(&b)) && s.parse::<f64>().is_ok()
}

/// Parses "<key>:<lo>-<hi>" into the targets (false = unknown key or bad numbers).
pub fn apply_override(t: &mut Targets, text: &str) -> bool {
    let Some((key, band)) = text.split_once(':') else { return false };
    if !has(key) {
        return false;
    }
    // A leading minus belongs to lo (swap_delta).
    let Some(cut) = band.get(1..).and_then(|b| b.find('-')).map(|i| i + 1) else { return false };
    let (a, b) = (&band[..cut], &band[cut + 1..]);
    if !is_float(a) || !is_float(b) {
        return false;
    }
    let (a, b) = (a.parse::<f64>().unwrap(), b.parse::<f64>().unwrap());
    if a > b {
        return false;
    }
    t.insert(key.to_string(), (a, b));
    true
}
