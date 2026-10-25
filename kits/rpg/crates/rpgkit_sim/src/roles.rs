//! Balance sim roles (tools/sim/sim_roles.gd, BAL2_SIM.md section 6): the class -> role map, the role loadouts and
//! the role KPIs with their section 8.1 bands.

use crate::kpi::{per_fight, ratio, Acc};
use crate::scenarios::FightSet;
use crate::targets::{self, Targets};
use aegis_core::fmt::fixed;
use rpgkit_battle::metrics::EARLY_DOWN_ROUND;
use rpgkit_data::Catalog;
use rpgkit_rules::stats::MAX_CLASS_SLOTS;

pub fn role_label(role: &str) -> &'static str {
    match role {
        "tank" => "Tank",
        "ranged" => "Ranged",
        "aoe" => "AoE",
        "support" => "Support",
        _ => "Striker",
    }
}

/// 6.2 (rogue = melee striker, brawler = melee AoE).
pub fn primary_role(class_id: &str) -> &'static str {
    match class_id {
        "warrior" => "tank",
        "ranger" => "ranged",
        "priest" => "support",
        "brawler" => "aoe",
        "rogue" => "striker",
        "mage" => "aoe",
        _ => "tank",
    }
}

pub fn secondary_roles(class_id: &str) -> &'static [&'static str] {
    match class_id {
        "warrior" => &["support"],
        "ranger" => &["aoe"],
        "priest" => &["striker"],
        "brawler" => &["tank", "striker"],
        "mage" => &["ranged"],
        _ => &[],
    }
}

/// "P" primary, "S" secondary, "-" off-role.
pub fn fit(class_id: &str, role: &str) -> &'static str {
    if primary_role(class_id) == role {
        "P"
    } else if secondary_roles(class_id).contains(&role) {
        "S"
    } else {
        "-"
    }
}

/// 6.3 role loadouts at L5.
fn loadouts(class_id: &str, role: &str) -> Option<&'static [&'static str]> {
    Some(match (class_id, role) {
        ("warrior", "tank") => &["taunt_shout", "shield_bash", "intimidate", "shield_wall"],
        ("warrior", "support") => &["taunt_shout", "intimidate", "shield_bash", "sunder", "riposte"],
        ("ranger", "ranged") => &["aimed_shot", "pinning_shot", "leap_back", "barbed_arrow", "deadeye"],
        ("ranger", "aoe") => &["aimed_shot", "pinning_shot", "leap_back", "volley", "deadeye"],
        ("priest", "support") => &["rally", "mend", "purify", "greater_mend", "renewal"],
        ("priest", "striker") => &["smite", "rally", "hex", "mend", "greater_mend"],
        ("brawler", "aoe") | ("brawler", "tank") => &["body_blow", "lunge", "sweep_kick", "cyclone_kick", "haymaker"],
        ("brawler", "striker") => &["body_blow", "lunge", "charge_in", "hamstring", "haymaker"],
        ("rogue", "striker") => &["quick_stab", "pounce", "lacerate", "backflip", "hamstring"],
        ("mage", "aoe") => &["arcane_bolt", "arc_lance", "firebomb", "glacial_burst", "focus"],
        ("mage", "ranged") => &["arcane_bolt", "arc_lance", "hex", "focus", "firebomb"],
        _ => return None,
    })
}

/// The bracketed L7 deltas of 6.3 apply from this level on.
pub const L7_LEVEL: i32 = 7;

fn l7_swap(class_id: &str, role: &str) -> Option<(&'static str, &'static str)> {
    match (class_id, role) {
        ("priest", "striker") => Some(("mend", "judgement")),
        ("rogue", "striker") => Some(("hamstring", "assassinate")),
        ("mage", "aoe") => Some(("focus", "inferno")),
        _ => None,
    }
}

/// The class's loadout for `role` (its primary loadout when it has none for the role), with the L7 swap, cut to the
/// families owned at `level` and filled by path order up to MAX_CLASS_SLOTS.
pub fn role_loadout(cat: &Catalog, class_id: &str, role: &str, level: i32) -> Vec<String> {
    let key = if loadouts(class_id, role).is_some() { role } else { primary_role(class_id) };
    let mut wanted: Vec<String> = loadouts(class_id, key).unwrap_or(&[]).iter().map(|s| s.to_string()).collect();
    if let Some((from, to)) = l7_swap(class_id, key) {
        if level >= L7_LEVEL {
            if let Some(i) = wanted.iter().position(|f| f == from) {
                wanted[i] = to.to_string();
            }
        }
    }
    let owned = owned_families(cat, class_id, level);
    let mut out: Vec<String> = Vec::new();
    for f in wanted {
        if owned.contains(&f) && !out.contains(&f) {
            out.push(f);
        }
    }
    for f in owned {
        if out.len() >= MAX_CLASS_SLOTS {
            break;
        }
        if !out.contains(&f) {
            out.push(f);
        }
    }
    out
}

/// Class-path families reached by `level`, path order, always-active skipped.
pub fn owned_families(cat: &Catalog, class_id: &str, level: i32) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Some(cls) = cat.classes.get(class_id) {
        for (step_level, skill) in cls.path_steps() {
            if skill.is_empty() || cat.is_always_active(&skill) || step_level > level {
                continue;
            }
            let fam = cat.family_of(&skill).to_string();
            if !out.contains(&fam) {
                out.push(fam);
            }
        }
    }
    out
}

/// One KPI of a role (6.1): its id, label, band (a targets key or ""), whether it prints as a percent, and whether
/// lower ranks better.
#[derive(Debug, Clone)]
pub struct Kpi {
    pub id: &'static str,
    pub label: String,
    pub band: &'static str,
    pub pct: bool,
    pub lower: bool,
}

fn k(id: &'static str, label: &str, band: &'static str, pct: bool) -> Kpi {
    Kpi { id, label: label.into(), band, pct, lower: false }
}

/// The KPIs of `role` (the first is its ranking KPI).
pub fn kpis(role: &str) -> Vec<Kpi> {
    match role {
        "tank" => vec![
            k("taken_share", "dmg_taken share%", "tank_taken_share", true),
            k("attacks_share", "atk_rcv share%", "tank_attacks_share", true),
            k("taunt_frac", "taunt/alive%", "", true),
            k("ally_hp_won", "allies hp_end won%", "tank_ally_hp", true),
            k("ally_hp", "allies hp_end%", "tank_ally_hp_all", true),
            k("hold_frac", "hold/alive%", "", true),
            k("survival", "survival%", "tank_survival", true),
        ],
        "ranged" => vec![
            k("dist", "nearest_foe", "ranged_dist", false),
            k("ranged_share", "ranged dmg%", "ranged_share", true),
            k("attacks_per_round", "atk_rcv/rnd", "ranged_attacks_per_round", false),
            k("dmg_per_round", "dmg/rnd", "", false),
            k("survival", "survival%", "", true),
        ],
        "aoe" => vec![
            k("foes_per_cast", "foes/cast", "aoe_foes_per_cast", false),
            k("allies_per_cast", "allies/cast", "aoe_allies_per_cast", false),
            k("clear_rounds", "rnds to clear", "", false),
            k("dmg_share", "dmg share%", "", true),
            k("swarm_clear", "swarm clear%", "aoe_swarm_clear", true),
        ],
        "support" => vec![
            k("heal_share", "heal/party hp%", "support_heal_share", true),
            k("saves", "saves/fight", "support_saves", false),
            k("cleanses", "cleanse/fight", "", false),
            k("buff_rounds", "buff rds/fight", "", false),
            k("debuff_rounds", "debuff rds/f", "", false),
            k("survivor_gain", "surv vs brawler", "support_survivor_gain", false),
            k("heal_casts", "heal casts/f", "", false),
            k("down_early", &format!("down<r{EARLY_DOWN_ROUND}%"), "", true),
            k("down_round", "down rnd", "", false),
        ],
        _ => vec![
            Kpi { lower: true, ..k("down_round", "prio down rnd", "", false) },
            k("down_by5", "down by r5%", "striker_down_by_5", true),
            k("priority_share", "prio dmg%", "striker_priority_share", true),
            k("dmg_per_round", "dmg/rnd", "", false),
            k("survival", "survival%", "", true),
        ],
    }
}

fn get(a: &Acc, f: &str) -> f64 {
    a.get(f).copied().unwrap_or(0.0)
}

fn share(t: &Acc, p: &Acc, field: &str) -> f64 {
    let total = get(p, field);
    if total > 0.0 {
        get(t, field) / total * 100.0
    } else {
        0.0
    }
}

/// The value of KPI `id` for a variant record (percent KPIs in percent). "down_round" reads the test unit's own down
/// round: Godot's match hit that arm before the priority-target one (kept as recorded).
pub fn kpi_value(id: &str, v: &FightSet) -> f64 {
    let (t, p, refs) = (&v.test, &v.party, &v.refs);
    let fights = v.fights.max(1.0);
    let rounds = v.rounds_sum.max(1.0);
    match id {
        "taken_share" => share(t, p, "dmg_taken"),
        "attacks_share" => share(t, p, "attacks_received"),
        "taunt_frac" => ratio(t, "taunt_blocks", "blocks_alive") * 100.0,
        "ally_hp" => per_fight(refs, "hp_end_frac") * 100.0,
        "ally_hp_won" => per_fight(&v.refs_won, "hp_end_frac") * 100.0,
        "heal_casts" => per_fight(t, "heal_casts"),
        "down_early" => per_fight(t, "down_early") * 100.0,
        "down_round" => ratio(t, "down_round", "downed"),
        "hold_frac" => ratio(t, "hold_blocks", "blocks_alive") * 100.0,
        "survival" => per_fight(t, "survived") * 100.0,
        "dist" => ratio(t, "dist_sum", "dist_samples"),
        "ranged_share" => ratio(t, "dmg_dealt_ranged", "dmg_dealt") * 100.0,
        "attacks_per_round" => get(t, "attacks_received") / rounds,
        "dmg_per_round" => get(t, "dmg_dealt") / rounds,
        "foes_per_cast" => ratio(t, "area_foes_hit", "area_casts"),
        "allies_per_cast" => ratio(t, "area_allies_hit", "area_casts"),
        "clear_rounds" => {
            if v.cleared > 0.0 {
                v.rounds_cleared_sum / v.cleared
            } else {
                v.cap as f64 + 1.0
            }
        }
        "dmg_share" => share(t, p, "dmg_dealt"),
        "swarm_clear" | "clear" => v.cleared / fights * 100.0,
        "heal_share" => {
            let max_hp = get(p, "max_hp");
            if max_hp > 0.0 {
                get(t, "heal_done") / max_hp * 100.0
            } else {
                0.0
            }
        }
        "saves" => per_fight(t, "near_death_saves"),
        "cleanses" => per_fight(t, "cleanses"),
        "buff_rounds" => per_fight(t, "buff_rounds"),
        "debuff_rounds" => per_fight(t, "debuff_rounds"),
        "survivor_gain" => v.survivors_sum / fights - v.baseline_survivors,
        "survivors" => v.survivors_sum / fights,
        "down_by5" => v.prio_by5 / fights * 100.0,
        "priority_share" => ratio(t, "priority_dmg", "dmg_dealt") * 100.0,
        _ => 0.0,
    }
}

/// Mean, floor and ceiling of a KPI over the variant records.
#[derive(Debug, Clone, Copy)]
pub struct Spread {
    pub mean: f64,
    pub floor: f64,
    pub ceil: f64,
}

pub fn spread(id: &str, variants: &[FightSet]) -> Spread {
    if variants.is_empty() {
        return Spread { mean: 0.0, floor: 0.0, ceil: 0.0 };
    }
    let (mut lo, mut hi, mut sum) = (f64::INFINITY, f64::NEG_INFINITY, 0.0);
    for v in variants {
        let x = kpi_value(id, v);
        lo = lo.min(x);
        hi = hi.max(x);
        sum += x;
    }
    Spread { mean: sum / variants.len() as f64, floor: lo, ceil: hi }
}

/// "mean [floor..ceiling]" with " X" when `marked` and the range misses the band.
pub fn cell(pct: bool, band: &str, s: Spread, t: &Targets, marked: bool) -> String {
    let mut text = if pct {
        format!("{} [{}..{}]", s.mean.round() as i64, s.floor.round() as i64, s.ceil.round() as i64)
    } else {
        format!("{} [{}..{}]", fixed(s.mean, 2), fixed(s.floor, 2), fixed(s.ceil, 2))
    };
    if marked && !band.is_empty() && !targets::overlaps(t, band, s.floor, s.ceil) {
        text += " X";
    }
    text
}

/// Ranking check (8.1) on the first KPI: primary classes in the top 2, a secondary class above every off-role class.
/// Returns class -> "rank N" (+ " X" on a miss), in `means` order.
pub fn rank_marks(role: &str, means: &[(String, f64)]) -> Vec<(String, String)> {
    let lower = kpis(role)[0].lower;
    let mut order: Vec<(String, f64)> = means.to_vec();
    order.sort_by(|a, b| {
        if targets::approx(a.1, b.1) {
            return a.0.cmp(&b.0);
        }
        let ord = a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal);
        if lower {
            ord
        } else {
            ord.reverse()
        }
    });
    let best_off = order.iter().position(|(c, _)| fit(c, role) == "-").unwrap_or(order.len());
    let ranks: Vec<(String, String)> = order
        .iter()
        .enumerate()
        .map(|(i, (c, _))| {
            let miss = (fit(c, role) == "P" && i >= 2) || (fit(c, role) == "S" && i > best_off);
            (c.clone(), format!("rank {}{}", i + 1, if miss { " X" } else { "" }))
        })
        .collect();
    means.iter().map(|(c, _)| ranks.iter().find(|r| &r.0 == c).cloned().unwrap()).collect()
}
