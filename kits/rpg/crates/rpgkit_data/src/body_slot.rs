//! Body slots and grafts (PK2-S2, `docs/sessions/PK2_GRAFTS.md` section 3; engine-only, no Godot original): a second
//! slot set beside the gear slots (kind `body_slots`), each row with a `role`: `class` (opened by the member's class
//! rite, taking class pieces only) or `monster` (any monster piece). A body slot holds an item of item type `GRAFT`
//! (a piece with a `tier` 1-3, the `body_role` it fits, an optional `keystone` mark and, for a class piece, the
//! `class_id` it belongs to) with its own effect list `body_effects`, in the passive grammar: a body item may grant a
//! meter (and a floor from the graft count, the meter effect's `floors`); worn gear still may not. The kind ships
//! without rows until the asset lane fills it, so no member holds a body slot.

use aegis_core::value::*;
use serde_json::Value;

/// The catalog kind (file data/catalog/body_slots.json; a missing file is an empty table).
pub const BODY_SLOTS_KIND: &str = "body_slots";
/// The slot roles (the schema's order): `monster` (any monster piece), `class` (opened by the member's class rite;
/// class pieces only).
pub const BODY_ROLES: [&str; 2] = [BODY_ROLE_MONSTER, BODY_ROLE_CLASS];
pub const BODY_ROLE_CLASS: &str = "class";
pub const BODY_ROLE_MONSTER: &str = "monster";
/// The item type of a body piece.
pub const GRAFT: &str = "GRAFT";
/// A graft's tiers (1-3); a top-tier install needs `GRAFT_TOP_TIER_NEEDS` grafts first.
pub const GRAFT_TIERS: (i32, i32) = (1, 3);
/// How many grafts a member holds before a top-tier (tier 3) graft may go in (PK2 3: "any Tier III surgery needs at
/// least 3 grafts already"; an overwrite counts the slot it fills).
pub const GRAFT_TOP_TIER_NEEDS: usize = 3;
/// A `GRAFT` item's own effect list key (`GraftSpec::effects`).
pub const BODY_EFFECTS_KEY: &str = "body_effects";
/// The most body slot rows a catalog holds.
pub const BODY_SLOTS_MAX: usize = 8;

/// One body slot row.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BodySlotDef {
    pub id: String,
    pub name: String,
    /// `class` or `monster` (`BODY_ROLES`).
    pub role: String,
}

impl BodySlotDef {
    pub fn from_obj(o: &Obj) -> Self {
        Self { id: string(o, "id", ""), name: string(o, "name", ""), role: string(o, "role", BODY_ROLE_MONSTER) }
    }

    /// The slot takes class pieces only (opened by the member's class rite).
    pub fn is_class(&self) -> bool {
        self.role == BODY_ROLE_CLASS
    }
}

/// A `GRAFT` item's body fields (`ItemDef::graft`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GraftSpec {
    /// 1-3 (`GRAFT_TIERS`).
    pub tier: i32,
    /// The slot role it fits (`BODY_ROLES`).
    pub role: String,
    /// Its learned skill takes the deck's keystone slot (one per member; a top-tier class piece).
    pub keystone: bool,
    /// A class piece's class ("" = any class).
    pub class_id: String,
    /// `body_effects`: its own effect list (the passive grammar, no member grant), applied while it sits in a body slot.
    pub effects: Vec<Value>,
}

impl GraftSpec {
    pub fn from_obj(o: &Obj) -> Self {
        Self { tier: int(o, "tier", GRAFT_TIERS.0), role: string(o, "body_role", BODY_ROLE_MONSTER), keystone: boolean(o, "keystone", false), class_id: string(o, "class_id", ""), effects: array(o, BODY_EFFECTS_KEY).to_vec() }
    }

    /// A top-tier graft (the 3-graft gate applies): tier 3, and a keystone graft whatever its tier (the content check
    /// refuses a keystone below tier 3; hand-built data still can't skip the gate).
    pub fn top_tier(&self) -> bool {
        self.tier >= GRAFT_TIERS.1 || self.keystone
    }
}
