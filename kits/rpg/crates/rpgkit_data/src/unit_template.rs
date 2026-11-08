//! Role templates (R4-1 `U2.3-TIERS`, FB-78, `docs/sessions/U2_CLASS.md` item 2; engine-only, no Godot original): the
//! reference stats a non-hero derived unit takes by its AI role (`AGGRESSOR`, `SKIRMISHER`, `SNIPER`, `SUPPORT`, the
//! schema's `ai_role` enum) instead of its class's, so a change to a playable class never moves an enemy's stats. A row
//! carries the two share vectors of the unit budget (`base_shares`, `growth_shares`); the growth curve is the budget's
//! (`unit_budget::budget_at`: 62 at level 1, +5 a level) read through the row's `growth_shares`, and the free points
//! (`stats::spend_pool`) are spent by the same `growth_shares`. The class keeps everything else (gear proficiency, skill
//! path, basics, loadout). The kind ships without rows until the asset lane fills it; with no row for a role the unit
//! keeps its class's stats (`rpgkit_rules::unit::templated`).

use crate::unit_budget::ClassShares;
use crate::{float_map, ClassDef};
use aegis_core::value::*;

/// The catalog kind (file data/catalog/unit_templates.json; a missing file is an empty table).
pub const UNIT_TEMPLATES_KIND: &str = "unit_templates";

/// One role template row.
#[derive(Debug, Clone, PartialEq)]
pub struct UnitTemplate {
    pub id: String,
    pub name: String,
    /// The AI role whose units take this row (one of the schema's `ai_role` options).
    pub role: String,
    pub shares: ClassShares,
}

impl UnitTemplate {
    pub fn from_obj(o: &Obj) -> Self {
        Self {
            id: string(o, "id", ""),
            name: string(o, "name", ""),
            role: string(o, "role", ""),
            shares: ClassShares { base: obj(o, "base_shares").map(float_map).unwrap_or_default(), growth: obj(o, "growth_shares").map(float_map).unwrap_or_default() },
        }
    }

    /// `class` with this row's share vectors in place of its own (its computed `base_stats` / `growth` views follow);
    /// everything else is the class's.
    pub fn over(&self, class: &ClassDef) -> ClassDef {
        let mut c = class.clone();
        c.base_stats = crate::unit_budget::stats_at(&self.shares, 1);
        c.growth = crate::unit_budget::gain_at(&self.shares, 2);
        c.shares = Some(self.shares.clone());
        c
    }
}
