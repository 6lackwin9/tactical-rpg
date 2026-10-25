//! GearBudget (scripts/core/specs/gear_budget.gd): one weighted value per equipment item, a class's default loadout
//! value as the sum over its slots, compared with TARGET_BUDGET +- TOLERANCE. Soft rules: warnings only.

use aegis_core::fmt::fixed;
use aegis_core::json;
use aegis_core::value::{boolean, string};
use emberwake_data::{ClassDef, ItemDef, LOADOUT_SLOTS};
use indexmap::IndexMap;
use serde_json::Value;

pub const W_POWER: f64 = 1.0;
pub const W_MITIGATION: f64 = 1.5;
pub const W_STAT_POINT: f64 = 2.0;
pub const W_BURDEN: f64 = -2.0;
pub const W_EXTRA_BLOCK: f64 = -3.0;
pub const W_SKILL_POTENCY: f64 = 20.0;
pub const W_STATUS_POTENCY: f64 = 0.2;
pub const W_SPELL_REACH: f64 = 3.0;
/// Near the median of the six original class loadouts (19 19 22 24 27 35).
pub const TARGET_BUDGET: f64 = 23.0;
pub const TOLERANCE: f64 = 0.20;
/// An empty main hand counts the trained-fists base power (an unarmed class is not read as under budget).
pub const UNARMED_VALUE: f64 = crate::damage::UNARMED_BASE_POWER as f64;

pub fn band_low() -> f64 {
    TARGET_BUDGET * (1.0 - TOLERANCE)
}

pub fn band_high() -> f64 {
    TARGET_BUDGET * (1.0 + TOLERANCE)
}

pub fn in_band(value: f64) -> bool {
    value >= band_low() - 0.0001 && value <= band_high() + 0.0001
}

pub fn is_special(item: &ItemDef) -> bool {
    boolean(&item.raw, "special", false)
}

/// Value of one item effect: skill_potency x20 per 1.0, status_potency x0.2, spell_reach x3 (resist 0).
pub fn effect_value(fx: &Value) -> f64 {
    let fx = json::dict(fx);
    match json::s(fx, "type").as_str() {
        "skill_potency" => json::f(fx, "value", 0.0) * W_SKILL_POTENCY,
        "status_potency" => json::i(fx, "value", 0) as f64 * W_STATUS_POTENCY,
        "spell_reach" => json::i(fx, "value", 0) as f64 * W_SPELL_REACH,
        _ => 0.0,
    }
}

/// Value of one item modifier (FORGE1_ITEMS 2.3): an adjust delta at its number's weight, a stat bonus at the stat
/// point weight, an effect as effect_value (a requirement is worth 0).
pub fn modifier_value(m: &Value) -> f64 {
    let m = json::dict(m);
    match json::s(m, "kind").as_str() {
        "adjust" => {
            let delta = json::i(m, "delta", 0) as f64;
            match json::s(m, "field").as_str() {
                "power" => delta * W_POWER,
                "mitigation" => delta * W_MITIGATION,
                "burden" => delta * W_BURDEN,
                "block_cost" => delta * W_EXTRA_BLOCK,
                _ => 0.0,
            }
        }
        "stat_bonus" => json::i(m, "value", 0) as f64 * W_STAT_POINT,
        "effect" => effect_value(json::get(m, "effect")),
        _ => 0.0,
    }
}

/// Weighted value of one item (0 for non-equipment).
pub fn item_value(item: &ItemDef) -> f64 {
    if !item.is_equipment() {
        return 0.0;
    }
    let bonus_total: i32 = item.stat_bonuses.values().sum();
    let mut v = item.base_power as f64 * W_POWER;
    v += item.mitigation as f64 * W_MITIGATION;
    v += bonus_total as f64 * W_STAT_POINT;
    v += item.burden as f64 * W_BURDEN;
    v += (item.block_cost - 1).max(0) as f64 * W_EXTRA_BLOCK;
    v + item.effects.iter().map(effect_value).sum::<f64>()
}

/// One filled gear slot of a loadout (`id` "" = bare hands).
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetRow {
    pub slot: String,
    pub id: String,
    pub name: String,
    pub value: f64,
    pub special: bool,
    pub note: String,
}

/// The class's filled gear slots in slot order, plus bare hands when the main hand is empty and any gear is worn.
pub fn loadout_breakdown(class: &ClassDef, items: &IndexMap<String, ItemDef>) -> Vec<BudgetRow> {
    let mut rows: Vec<BudgetRow> = Vec::new();
    for slot in LOADOUT_SLOTS {
        let Some(item) = class.loadout.get(slot).and_then(|id| items.get(id)) else {
            continue;
        };
        if !item.is_equipment() {
            continue;
        }
        let name = if item.raw.contains_key("name") { item.name.clone() } else { item.id.clone() };
        rows.push(BudgetRow { slot: slot.into(), id: item.id.clone(), name, value: item_value(item), special: is_special(item), note: string(&item.raw, "special_note", "") });
    }
    if !rows.is_empty() && !class.loadout.contains_key("main_hand") {
        rows.push(BudgetRow { slot: "main_hand".into(), id: String::new(), name: "Unarmed".into(), value: UNARMED_VALUE, special: false, note: String::new() });
    }
    rows
}

/// GearBudget.check_class: a default loadout outside the band (over it is fine with a documented special item).
pub fn check_class(class: &ClassDef, items: &IndexMap<String, ItemDef>) -> Vec<String> {
    let mut rows = loadout_breakdown(class, items);
    if rows.is_empty() {
        return Vec::new();
    }
    let total = rows.iter().fold(0.0, |t, r| t + r.value);
    if in_band(total) {
        return Vec::new();
    }
    let over = total > band_high();
    if over && rows.iter().any(|r| r.special && !r.note.trim().is_empty()) {
        return Vec::new();
    }
    rows.sort_by(|a, b| b.value.abs().total_cmp(&a.value.abs()));
    let signed = |x: f64| if x.is_sign_negative() { fixed(x, 1) } else { format!("+{}", fixed(x, 1)) };
    let tops: Vec<String> = rows.iter().take(3).map(|r| format!("{} {}", r.name, signed(r.value))).collect();
    vec![format!(
        "Gear budget: default loadout is {}, {} the budget {} (band {} to {}). Top items: {}.",
        fixed(total, 1),
        if over { "above" } else { "below" },
        fixed(TARGET_BUDGET, 1),
        fixed(band_low(), 1),
        fixed(band_high(), 1),
        tops.join(", ")
    )]
}

/// GearBudget.check_item: a special item needs a note on its trade-off.
pub fn check_item(item: &ItemDef) -> Vec<String> {
    if is_special(item) && string(&item.raw, "special_note", "").trim().is_empty() {
        return vec!["Special item needs a special note describing its trade-off or playstyle.".into()];
    }
    Vec::new()
}
