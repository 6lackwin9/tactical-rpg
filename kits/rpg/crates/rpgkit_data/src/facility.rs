//! Facilities (PR6-8, `docs/sessions/PK4_PK5_CAMPAIGN_HUB.md` section 4; engine-only, no Godot original; story-free):
//! the world kind `facilities`. A facility row `{id, name, keeper_character, levels}` is kept by a character (the NPC
//! service `facility`); the party starts each facility at level `LEVEL_MIN` and raises it one level at a time up to
//! `LEVEL_MAX`. A level row `{level, name, scrap_cost, materials, requires_flags, effects}` names what the upgrade to it
//! costs (purse and stash items), when it may be bought (flag conditions) and its passive effects. Effects add up: a
//! facility at level 3 has the effects of its levels 1, 2 and 3. The saved level lives in `GameState.facilities`
//! (absent = `LEVEL_MIN`). The kind ships without rows until the asset lane fills it, so no character keeps one.

use aegis_core::json::{self, Dict};
use indexmap::IndexMap;
use serde_json::Value;

/// The world kind (file `worlds/<id>/facilities.json`; a missing file is an empty table).
pub const FACILITIES_KIND: &str = "facilities";
/// The NPC service id of the facility pane (`npc_services`, the dialogue's service choice).
pub const FACILITY_SERVICE: &str = "facility";
/// The level every facility starts at and the highest level.
pub const LEVEL_MIN: i64 = 1;
pub const LEVEL_MAX: i64 = 3;
/// The keys of a level row.
pub const LEVEL_KEYS: [&str; 6] = ["level", "name", "scrap_cost", "materials", "requires_flags", "effects"];
/// A level's purse cost and a material count range, and the most effects a level holds.
pub const SCRAP_COST_MAX: i64 = 99999;
pub const MATERIAL_COUNT_MAX: i64 = 99;
pub const LEVEL_EFFECTS_MAX: usize = 8;
/// A `shop_pct` / `recipe_pct` percent range (a discount: 100 = no change).
pub const PCT_RANGE: (i64, i64) = (10, 100);
/// A `rest_upkeep` delta range (added to the rest's upkeep cost, never below 0).
pub const REST_DELTA: (i64, i64) = (-9, 9);

/// `shop_pct` {pct}: every stock and loot-for-sale price at the shops the keeper runs is ceil(price x pct / 100), at
/// least 1 (after the shop's own discount; the supply price and the sell prices are unchanged).
pub const FX_SHOP_PCT: &str = "shop_pct";
/// `recipe_pct` {pct, recipe_tags?}: the purse cost of the recipes the keeper offers (and a `repair`'s purse share) is
/// ceil(cost x pct / 100); with `recipe_tags`, only recipes carrying one of them.
pub const FX_RECIPE_PCT: &str = "recipe_pct";
/// `rest_upkeep` {delta}: added to a campaign rest's upkeep cost (never below 0).
pub const FX_REST_UPKEEP: &str = "rest_upkeep";
/// `set_flag` {key, value}: the flag is set once, when the level is bought (ties a level to flag-gated rules such as a
/// roster's `bunks_flag`).
pub const FX_SET_FLAG: &str = "set_flag";
/// The level effects with their keys besides `effect` (the optional ones included).
pub const FACILITY_EFFECTS: [(&str, &[&str]); 4] = [(FX_SHOP_PCT, &["pct"]), (FX_RECIPE_PCT, &["pct", "recipe_tags"]), (FX_REST_UPKEEP, &["delta"]), (FX_SET_FLAG, &["key", "value"])];

/// One level row, read leniently (the content checks report bad rows).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FacilityLevel {
    pub level: i64,
    pub name: String,
    pub scrap_cost: i64,
    pub materials: IndexMap<String, i64>,
    pub requires_flags: Value,
    pub effects: Vec<Dict>,
}

impl FacilityLevel {
    pub fn from_obj(o: &Dict) -> Self {
        Self {
            level: json::i(o, "level", 0),
            name: json::s(o, "name"),
            scrap_cost: json::i(o, "scrap_cost", 0).max(0),
            materials: json::ints(json::get(o, "materials")).into_iter().filter(|(_, n)| *n > 0).collect(),
            requires_flags: o.get("requires_flags").cloned().unwrap_or(Value::Array(Vec::new())),
            effects: json::aget(o, "effects").iter().filter_map(Value::as_object).cloned().collect(),
        }
    }
}

/// The level rows of facility row `row`, in row order.
pub fn levels(row: &Dict) -> Vec<FacilityLevel> {
    json::aget(row, "levels").iter().filter_map(Value::as_object).map(FacilityLevel::from_obj).collect()
}

/// PR6-8 (B2): a shop row's optional `supply_price_mult_by_act` `{"<act>": mult}`: the supply price in act `act` is
/// round(supply_price x mult); an act it does not name (and a row without it) keeps the supply price.
pub const SUPPLY_MULT_KEY: &str = "supply_price_mult_by_act";
/// A supply multiplier's range.
pub const SUPPLY_MULT_RANGE: (f64, f64) = (0.1, 10.0);

/// The supply price of shop row `shop` in act `act`: round(supply_price x mult) for the act's multiplier, else the
/// supply price itself.
pub fn supply_price_in_act(shop: &Dict, act: i64) -> i64 {
    let base = json::i(shop, "supply_price", 0);
    match json::dget(shop, SUPPLY_MULT_KEY).get(&act.to_string()) {
        Some(m) if m.is_number() && base > 0 => json::roundi(base as f64 * json::to_float_or(m, 1.0)).max(1),
        _ => base,
    }
}
