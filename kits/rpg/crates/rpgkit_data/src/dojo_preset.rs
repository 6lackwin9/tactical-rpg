//! Dojo presets as data (PK1-KITFIX (c), FB-184 Option A; engine-only, no Godot original): the gear Dojo's quick
//! loadouts (kind `dojo_presets`): gear per slot and pouch items, plus passives the hero holds while the preset is
//! applied (a test passive that grants a meter, for example). LAYER-DOWN-2: the Dojo lists the rows in file order
//! (`rpgkit_battle::dojo::presets_of`); the kit has no built-in preset (the game's former built-ins are its rows).

use crate::LOADOUT_SLOTS;
use aegis_core::value::*;
use serde_json::Value;

/// The catalog kind (file data/catalog/dojo_presets.json; a missing file is an empty table).
pub const DOJO_PRESETS_KIND: &str = "dojo_presets";
/// The most pouch items a preset loads (the Dojo hero's battle pouch slots).
pub const DOJO_POUCH_MAX: usize = 3;
/// The most passives a preset grants.
pub const DOJO_PASSIVES_MAX: usize = 4;

/// One preset row.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DojoPresetDef {
    pub id: String,
    pub name: String,
    pub description: String,
    /// (slot key, item id) in `LOADOUT_SLOTS` order (the row's `gear` object; unknown slots are dropped).
    pub gear: Vec<(String, String)>,
    pub pouch: Vec<String>,
    /// The passives the hero holds while the preset is applied.
    pub passives: Vec<String>,
}

impl DojoPresetDef {
    pub fn from_obj(o: &Obj) -> Self {
        let gear_obj = obj(o, "gear");
        let gear = LOADOUT_SLOTS
            .iter()
            .filter_map(|slot| gear_obj.and_then(|g| g.get(*slot)).and_then(Value::as_str).filter(|i| !i.is_empty()).map(|i| (slot.to_string(), i.to_string())))
            .collect();
        let ids = |key: &str| array(o, key).iter().filter_map(Value::as_str).filter(|s| !s.is_empty()).map(str::to_string).collect();
        Self { id: string(o, "id", ""), name: string(o, "name", ""), description: string(o, "description", ""), gear, pouch: ids("pouch"), passives: ids("passives") }
    }
}
