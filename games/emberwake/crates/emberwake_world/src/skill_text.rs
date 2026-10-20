//! Rank texts for the Skills tab (SKILLUI1 R8, no Godot counterpart: Godot had no rank choice). One line per rank of a
//! family: the AP the battle charges, the blocks, and only the fields that differ between the family's ranks, so the
//! < > arrows read as "what changes". A locked rank reads "unlocks at Lv n".

use emberwake_data::{Catalog, SkillDef};
use emberwake_rules::unit::Unit;

#[derive(Debug, Clone, PartialEq)]
pub struct RankLine {
    pub tier: i32,
    pub unlocked: bool,
    pub unlock_level: i32,
    pub ap: i32,
    pub text: String,
}

fn rounds(n: i32) -> String {
    format!("{n} round{}", if n == 1 { "" } else { "s" })
}

/// Every rank of `family` for `unit`, ascending.
pub fn rank_lines(cat: &Catalog, unit: &Unit, family: &str) -> Vec<RankLine> {
    let rows = cat.family_rows(family);
    let differs = |f: &dyn Fn(&SkillDef) -> String| rows.iter().map(|s| f(s)).collect::<std::collections::BTreeSet<_>>().len() > 1;
    // The fields a rank can change, each written the way the tab shows it ("" when the rank has none of it).
    let fields: [&dyn Fn(&SkillDef) -> String; 9] = [
        &|s| if s.power_mult > 0.0 { format!("power x{:.2}", s.power_mult) } else { String::new() },
        &|s| if s.heal_base > 0 || s.heal_mult > 0.0 { format!("heal {} + {:.2} x {}", s.heal_base, s.heal_mult, s.heal_stat) } else { String::new() },
        &|s| {
            if s.status_effect.is_empty() {
                return String::new();
            }
            let magnitude = if s.status_magnitude > 0 { format!(" ({})", s.status_magnitude) } else { String::new() };
            format!("{} {}{magnitude} for {}", s.status_effect, s.status_potency, rounds(s.status_rounds))
        },
        &|s| if s.mitigation > 0.0 { format!("blocks {}% damage", (s.mitigation * 100.0).round() as i32) } else { String::new() },
        &|s| if s.boost > 0.0 { format!("attack x{:.2} for {}", s.boost, rounds(s.boost_rounds)) } else { String::new() },
        &|s| match (s.grant_status.is_empty(), s.extra_grants.len()) {
            (true, 0) => String::new(),
            (true, n) => format!("+{n} effects"),
            (false, 0) => format!("grants {}", s.grant_status),
            (false, n) => format!("grants {} +{n} more effects", s.grant_status),
        },
        &|s| if s.cleanse_statuses.is_empty() { String::new() } else { format!("cleanses {}", s.cleanse_statuses.join(", ")) },
        &|s| {
            let mut v = vec![format!("reach {}-{}", s.reach_min, s.reach_max)];
            if !s.area.is_empty() && s.area != "NONE" {
                v.push(format!("area {}", s.area.to_lowercase()));
            }
            v.join(", ")
        },
        &|s| {
            let mut v = Vec::new();
            if s.dash_tiles > 0 { v.push(format!("dash {}", s.dash_tiles)); }
            if s.jump_tiles > 0 { v.push(format!("jump {}", s.jump_tiles)); }
            if s.recover_ap > 0 { v.push(format!("+{} AP back", s.recover_ap)); }
            v.join(", ")
        },
    ];
    let changing: Vec<&dyn Fn(&SkillDef) -> String> = fields.into_iter().filter(|f| differs(*f)).collect();
    rows.iter()
        .map(|s| {
            let unlocked = unit.owns_skill(cat, &s.id) && unit.tier_unlocked(cat, s);
            let unlock_level = unit.skill_unlock_level(cat, s);
            let ap = emberwake_battle::skill::ap_cost(s, Some(unit));
            let text = if unlocked {
                let mut bits = vec![format!("{ap} AP"), format!("{} block{}", s.blocks, if s.blocks == 1 { "" } else { "s" })];
                bits.extend(changing.iter().map(|f| f(s)).filter(|t| !t.is_empty()));
                bits.join(", ")
            } else {
                format!("unlocks at Lv {unlock_level}")
            };
            RankLine { tier: s.tier, unlocked, unlock_level, ap, text }
        })
        .collect()
}
