//! Sieges (PR6-9, `docs/sessions/PK4_PK5_CAMPAIGN_HUB.md` section 5, B6; engine-only, no Godot original; story-free):
//! the world kind `sieges`. A siege row `{id, name, rooms, carry_hp, carry_downed, carry_pouch, no_camp, restocks,
//! win_effects, loss_effects}` is an ordered run of linked room fights (`rooms`: `{encounter, name, facility}`) the
//! effect `start_siege {siege}` opens. While it runs the party keeps its exact HP between rooms (`carry_hp`), a hero left
//! downed stays down until triaged (`carry_downed`), the pouches are not topped up from the stash after a room
//! (`carry_pouch`; `restocks` explicit re-stocks between rooms), and no rest is allowed (`no_camp`). Losing a room ends
//! the siege: the room's `facility` drops one level (never below the first) and the `loss_effects` run; clearing the
//! last room runs the `win_effects`. The kind ships without rows until the asset lane fills it.

use aegis_core::json::{self, Dict};
use serde_json::Value;

/// The world kind (file `worlds/<id>/sieges.json`; a missing file is an empty table).
pub const SIEGES_KIND: &str = "sieges";
/// The effect that opens a siege (`{effect: "start_siege", siege}`), in dialogue choices, meter rules, checkpoints,
/// triggers and sequence steps.
pub const START_SIEGE: &str = "start_siege";
/// The siege effects with their keys besides `effect`.
pub const SIEGE_EFFECTS: [(&str, &[&str]); 1] = [(START_SIEGE, &["siege"])];
/// The keys of a room row.
pub const ROOM_KEYS: [&str; 3] = ["encounter", "name", "facility"];
/// How many rooms a siege holds.
pub const ROOMS_RANGE: (usize, usize) = (1, 4);
/// The re-stocks between rooms: default and highest.
pub const RESTOCKS_DEFAULT: i64 = 1;
pub const RESTOCKS_MAX: i64 = 3;
/// The most effects a win or loss list holds.
pub const OUTCOME_EFFECTS_MAX: usize = 8;

/// One room of a siege, read leniently (the content checks report bad rows).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SiegeRoom {
    pub encounter: String,
    pub name: String,
    /// The facility ref the room guards ("" = none): a lost room lowers it one level.
    pub facility: String,
}

/// A siege row with its defaults.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SiegeDef {
    pub id: String,
    pub name: String,
    pub rooms: Vec<SiegeRoom>,
    pub carry_hp: bool,
    pub carry_downed: bool,
    pub carry_pouch: bool,
    pub no_camp: bool,
    pub restocks: i64,
    pub win_effects: Vec<Dict>,
    pub loss_effects: Vec<Dict>,
}

impl SiegeDef {
    pub fn from_obj(o: &Dict) -> Self {
        let dicts = |key: &str| -> Vec<Dict> { json::aget(o, key).iter().filter_map(Value::as_object).cloned().collect() };
        Self {
            id: json::s(o, "id"),
            name: json::s(o, "name"),
            rooms: json::aget(o, "rooms")
                .iter()
                .filter_map(Value::as_object)
                .map(|r| SiegeRoom { encounter: json::s(r, "encounter"), name: json::s(r, "name"), facility: json::s(r, "facility") })
                .collect(),
            carry_hp: json::b(o, "carry_hp", true),
            carry_downed: json::b(o, "carry_downed", true),
            carry_pouch: json::b(o, "carry_pouch", true),
            no_camp: json::b(o, "no_camp", true),
            restocks: json::i(o, "restocks", RESTOCKS_DEFAULT).clamp(0, RESTOCKS_MAX),
            win_effects: dicts("win_effects"),
            loss_effects: dicts("loss_effects"),
        }
    }
}
