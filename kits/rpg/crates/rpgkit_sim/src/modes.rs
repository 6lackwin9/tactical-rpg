//! The matrix modes of the balance sim (tools/balance_sim.gd): duel, tier, levels, group and stats. Duel, tier,
//! levels and group run every matchup with sides swapped over every seed, so side and position bias cancel out.

use crate::runner::{Cell, Def, Runner};
use crate::variants;
use aegis_core::fmt::fixed;
use rpgkit_rules::damage;

const TIER_ORDER: [&str; 4] = ["minion", "standard", "elite", "boss"];
const LEVEL_STEPS: [i32; 4] = [1, 3, 5, 7];
const STATS_LEVELS: [i32; 4] = [1, 3, 5, 7];

fn ids(id: &str, n: usize) -> Vec<String> {
    vec![id.to_string(); n]
}

fn score_label(r: &Runner) -> &'static str {
    match r.args.score.as_str() {
        "hp" => "hp margin %",
        "binary" => "binary %",
        _ => "bin/hp %",
    }
}

/// Row average of score cells.
fn avg(r: &Runner, row: &[Cell]) -> String {
    let counted: Vec<&Cell> = row.iter().filter(|c| c.n > 0).collect();
    if counted.is_empty() {
        return "n/a".into();
    }
    let n = counted.len();
    let b: f64 = counted.iter().map(|c| c.bin).sum();
    let h: f64 = counted.iter().map(|c| c.hp).sum();
    r.cell(Cell { bin: b / n as f64, hp: h / n as f64, n })
}

/// The duel band flag on the row's mean hp score: "X LOW (band ..)" / "X HIGH (band ..)" / "".
fn band_flag(r: &Runner, row: &[Cell]) -> String {
    let counted: Vec<&Cell> = row.iter().filter(|c| c.n > 0).collect();
    if counted.is_empty() {
        return String::new();
    }
    let pct = (counted.iter().map(|c| c.hp).sum::<f64>() / counted.len() as f64 * 100.0).round() as i32;
    let (lo, hi) = (r.args.band_lo, r.args.band_hi);
    if pct < lo {
        format!("X LOW (band {lo}-{hi})")
    } else if pct > hi {
        format!("X HIGH (band {lo}-{hi})")
    } else {
        String::new()
    }
}

pub fn duel(r: &mut Runner, level: i32) {
    let classes = r.args.game.classes.ids;
    for &c in classes {
        r.define(&format!("simp_{c}"), Def::new(c, level, "hero", true));
        r.define(&format!("sime_{c}"), Def::new(c, level, "hero", false));
    }
    r.print(format!("DUEL level {level}, hero vs hero, score of the ROW class ({})", score_label(r)));
    let mut header = format!("{:<10}", "");
    for &c in classes {
        header += &format!("{c:<10}");
    }
    r.print(format!("{header}avg"));
    for &a in classes {
        let mut line = format!("{a:<10}");
        let mut row = Vec::new();
        for &b in classes {
            if a == b {
                line += &format!("{:<10}", "-");
                continue;
            }
            let res = r.matchup(&ids(&format!("simp_{a}"), 1), &ids(&format!("sime_{b}"), 1));
            row.push(res);
            line += &format!("{:<10}", r.cell(res));
        }
        let line = format!("{line}{:<10}{}", avg(r, &row), band_flag(r, &row));
        r.print(line);
    }
}

pub fn tier(r: &mut Runner, level: i32) {
    let mirror = r.args.tier_foe == crate::args::MIRROR;
    let foe = if mirror { "its own class".to_string() } else { format!("a {}", r.args.tier_foe) };
    r.print(format!("TIER level {level}: hero class vs {foe} at a tier (score of the hero, {})", score_label(r)));
    for &c in r.args.game.classes.ids {
        r.define(&format!("simp_{c}"), Def::new(c, level, "hero", true));
        let mut line = format!("{c:<10}");
        let foe_class = r.args.tier_foe_for(c);
        for tier in TIER_ORDER {
            let foe = format!("sime_{foe_class}_{tier}");
            r.define(&foe, Def::new(&foe_class, level, tier, false));
            let p = ids(&format!("simp_{c}"), 1);
            let one = r.matchup(&p, &ids(&foe, 1));
            let three = r.matchup(&p, &ids(&foe, 3));
            line += &format!("{tier} 1v1 {} / 1v3 {}   ", r.cell(one), r.cell(three));
        }
        r.print(line);
    }
}

pub fn levels(r: &mut Runner) {
    r.print(format!("LEVELS: class at level N vs the same class at N+2 (score of the lower level, {})", score_label(r)));
    for &c in r.args.game.classes.ids {
        let mut line = format!("{c:<10}");
        for lvl in LEVEL_STEPS {
            r.define(&format!("simp_{c}"), Def::new(c, lvl, "hero", true));
            r.define(&format!("sime_{c}"), Def::new(c, lvl + 2, "hero", false));
            let res = r.matchup(&ids(&format!("simp_{c}"), 1), &ids(&format!("sime_{c}"), 1));
            line += &format!("L{lvl} vs L{}: {}   ", lvl + 2, r.cell(res));
        }
        r.print(line);
    }
}

/// Hero L vs 3 minions at max(1, L-2), 3 standard at max(1, L-2), 3 minions at L (expected loss).
pub fn group(r: &mut Runner, level: i32) {
    let low = (level - 2).max(1);
    r.print(format!("GROUP level {level}: hero vs 3 of its class (score of the hero, {})", score_label(r)));
    for &c in r.args.game.classes.ids {
        r.define(&format!("simp_{c}"), Def::new(c, level, "hero", true));
        r.define(&format!("sime_{c}_minion_low"), Def::new(c, low, "minion", false));
        r.define(&format!("sime_{c}_standard_low"), Def::new(c, low, "standard", false));
        r.define(&format!("sime_{c}_minion_eq"), Def::new(c, level, "minion", false));
        let p = ids(&format!("simp_{c}"), 1);
        let a = r.matchup(&p, &ids(&format!("sime_{c}_minion_low"), 3));
        let b = r.matchup(&p, &ids(&format!("sime_{c}_standard_low"), 3));
        let e = r.matchup(&p, &ids(&format!("sime_{c}_minion_eq"), 3));
        let line = format!("{c:<10}3 minion L{low} {}   3 standard L{low} {}   3 minion L{level} {}", r.cell(a), r.cell(b), r.cell(e));
        r.print(line);
    }
}

/// No fights: per class the derived numbers of the party-side unit (spend / loadout variants apply).
pub fn stats(r: &mut Runner) {
    let levels: Vec<i32> = if r.args.all_levels { STATS_LEVELS.to_vec() } else { vec![r.args.level] };
    r.print("STATS: HP, armor rating, reduction %, EHP = HP / (1 - reduction), melee / ranged power, range, initiative, move rate, crit %, active class families");
    r.print(format!(
        "{:<8} {:<3} {:<16} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>6}  {}",
        "class", "L", "spend", "HP", "armor", "red%", "EHP", "melee", "rngd", "range", "init", "move", "crit%", "families"
    ));
    for lvl in levels {
        for &c in r.args.game.classes.ids {
            for sv in r.args.spend_variants() {
                r.spend_variant = sv.clone();
                r.define(&format!("simp_{c}"), Def::new(c, lvl, "hero", true));
                let Some(u) = r.build(&format!("simp_{c}")) else { continue };
                let rating = damage::armor_rating(&u);
                let red = damage::armor_reduction(rating);
                let ehp = (u.max_hp as f64 / (1.0 - red)).round() as i64;
                let spend = variants::spend_for(&r.d.cat, &sv, c, lvl);
                let spend_text = if spend.is_empty() { sv.clone() } else { format!("{sv}{}", variants::spend_text(&spend)) };
                let line = format!(
                    "{:<8} {:<3} {:<16} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>6}  {}",
                    c,
                    lvl,
                    spend_text,
                    u.max_hp,
                    rating,
                    fixed(red * 100.0, 1),
                    ehp,
                    damage::power_of(&u, false, false),
                    damage::power_of(&u, true, false),
                    u.attack_range,
                    u.initiative,
                    u.move_rate,
                    fixed(damage::crit_chance(&u), 1),
                    u.active_class_families.join(",")
                );
                r.print(line);
            }
        }
    }
    r.spend_variant = "class".into();
}
