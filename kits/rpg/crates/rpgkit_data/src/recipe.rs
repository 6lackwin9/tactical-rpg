//! Recipes (PK2-S3, `docs/sessions/PK2_GRAFTS.md` section 4; engine-only, no Godot original): the exchange service. A
//! recipe row (kind `recipes`) is offered by a character (`npc`, the NPC service `recipe`) while its `requires` hold
//! (the dialogue condition grammar: flags, quest steps, items, gold, stats); running it pays its `cost` (purse amounts
//! and stash items) and applies its `effects` to its `target`: a party `member`, a `weapon` (weapon instances, not built
//! yet) or a stash `item` (break-down). The kind ships without rows until the asset lane fills it, so no NPC offers one.

use aegis_core::value::*;
use indexmap::IndexMap;
use serde_json::Value;

/// The catalog kind (file data/catalog/recipes.json; a missing file is an empty table).
pub const RECIPES_KIND: &str = "recipes";
/// The NPC service id of the recipe pane (`npc_services`, the dialogue's service choice).
pub const RECIPE_SERVICE: &str = "recipe";
/// What a recipe works on: a party member (picked when it runs), a weapon instance (not built yet) or a stash item
/// (picked when it runs).
pub const RECIPE_TARGETS: [&str; 3] = [TARGET_MEMBER, TARGET_WEAPON, TARGET_ITEM];
pub const TARGET_MEMBER: &str = "member";
pub const TARGET_WEAPON: &str = "weapon";
pub const TARGET_ITEM: &str = "item";
/// The cost keys: the purse amounts and the stash items.
pub const RECIPE_COST_KEYS: [&str; 3] = ["gold", "supplies", "items"];
/// A cost item's count range, and the most effects a row holds.
pub const RECIPE_COUNT_MAX: i64 = 99;
pub const RECIPE_EFFECTS_MAX: usize = 8;
/// A `max_hp_pct` effect's range (a permanent percent of the member's max HP, negative lowers it), a `track`,
/// `floor_offset` or `doctrine` delta's.
pub const RECIPE_PCT: (i64, i64) = (-90, 100);
pub const RECIPE_DELTA: (i64, i64) = (-100, 100);
/// The class proficiency roles (`ClassDef::proficiency_roles`): what a `train_proficiency` recipe names.
pub const PROF_ROLES: [&str; 4] = ["main", "secondary", "third", "minor"];

/// The member effects with their keys besides `effect`, and the target each works on: `install` (a `GRAFT` item into a
/// body slot, an occupied slot overwritten and its graft destroyed), `remove` (empties a body slot, its graft
/// destroyed), `track` (moves the member's stage on a unit track), `max_hp_pct` (a permanent max HP offset, a percent of
/// its max HP), `floor_offset` (added to the member's floor offset for its next battle), `train_proficiency` (one
/// proficiency level in a family of the class role, up to `to`), `drill` (mastery XP), `doctrine` (a delta a later
/// system reads; reserved, does nothing yet), `repair` and `reforge` (a weapon instance; `masterwork` makes it
/// masterwork, behind the recipe's `requires`), `salvage` (breaks a stash item down: `gold`, plus the salvage item its
/// rarity names in `tiers` and the one its theme names in `themes`).
pub const RECIPE_EFFECTS: [(&str, &[&str], &str); 11] = [
    (FX_INSTALL, &["item", "slot"], TARGET_MEMBER),
    (FX_REMOVE, &["slot"], TARGET_MEMBER),
    (FX_TRACK, &["track", "delta"], TARGET_MEMBER),
    (FX_MAX_HP_PCT, &["pct"], TARGET_MEMBER),
    (FX_FLOOR_OFFSET, &["delta"], TARGET_MEMBER),
    (FX_TRAIN, &["role", "to"], TARGET_MEMBER),
    (FX_DRILL, &["xp"], TARGET_MEMBER),
    (FX_DOCTRINE, &["axis", "delta"], ""),
    (FX_REPAIR, &[], TARGET_WEAPON),
    (FX_REFORGE, &["masterwork"], TARGET_WEAPON),
    (FX_SALVAGE, &["gold", "tiers", "themes"], TARGET_ITEM),
];
pub const FX_INSTALL: &str = "install";
pub const FX_REMOVE: &str = "remove";
pub const FX_TRACK: &str = "track";
pub const FX_MAX_HP_PCT: &str = "max_hp_pct";
pub const FX_FLOOR_OFFSET: &str = "floor_offset";
pub const FX_TRAIN: &str = "train_proficiency";
pub const FX_DRILL: &str = "drill";
pub const FX_DOCTRINE: &str = "doctrine";
pub const FX_REPAIR: &str = "repair";
pub const FX_REFORGE: &str = "reforge";
pub const FX_SALVAGE: &str = "salvage";
/// The effects whose system is not built yet (effect, the missing system): the check refuses them and a run refuses
/// the recipe, until the system lands (weapon mastery, weapon instances).
pub const RECIPE_PENDING: [(&str, &str); 3] = [(FX_DRILL, "weapon mastery"), (FX_REPAIR, "weapon instances"), (FX_REFORGE, "weapon instances")];

/// An effect's keys besides `effect` (None: not a recipe effect).
pub fn effect_keys(effect: &str) -> Option<&'static [&'static str]> {
    RECIPE_EFFECTS.iter().find(|e| e.0 == effect).map(|e| e.1)
}

/// The target an effect works on ("" = any target).
pub fn effect_target(effect: &str) -> &'static str {
    RECIPE_EFFECTS.iter().find(|e| e.0 == effect).map_or("", |e| e.2)
}

/// The system an effect waits on (None: built).
pub fn pending_system(effect: &str) -> Option<&'static str> {
    RECIPE_PENDING.iter().find(|p| p.0 == effect).map(|p| p.1)
}

/// What a recipe costs: purse amounts and stash items (id -> count).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RecipeCost {
    pub gold: i64,
    pub supplies: i64,
    pub items: IndexMap<String, i64>,
}

impl RecipeCost {
    pub fn from_obj(o: &Obj) -> Self {
        let items = int_map(o, "items").into_iter().map(|(k, v)| (k, i64::from(v))).collect();
        Self { gold: i64::from(int(o, "gold", 0)), supplies: i64::from(int(o, "supplies", 0)), items }
    }

    /// Nothing to pay.
    pub fn is_free(&self) -> bool {
        self.gold == 0 && self.supplies == 0 && self.items.is_empty()
    }
}

/// One recipe row.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RecipeDef {
    pub id: String,
    pub name: String,
    pub description: String,
    /// The character who offers it (a world ref; "world:id" or an id the played world resolves).
    pub npc: String,
    /// Conditions in the dialogue grammar (`{kind, ...}`), all holding for the recipe to be offered.
    pub requires: Vec<Value>,
    pub cost: RecipeCost,
    /// `member`, `weapon` or `item` (`RECIPE_TARGETS`).
    pub target: String,
    /// The effect rows (`{effect, ...}`), applied in order.
    pub effects: Vec<Obj>,
}

impl RecipeDef {
    pub fn from_obj(o: &Obj) -> Self {
        Self {
            id: string(o, "id", ""),
            name: string(o, "name", ""),
            description: string(o, "description", ""),
            npc: string(o, "npc", ""),
            requires: array(o, "requires").to_vec(),
            cost: obj(o, "cost").map(RecipeCost::from_obj).unwrap_or_default(),
            target: string(o, "target", TARGET_MEMBER),
            effects: array(o, "effects").iter().filter_map(Value::as_object).cloned().collect(),
        }
    }

    /// The effect names in order.
    pub fn effect_names(&self) -> Vec<String> {
        self.effects.iter().map(|e| string(e, "effect", "")).collect()
    }
}
