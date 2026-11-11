//! PR5-33 (LAYER-UP-DATA, audit 2026-11-10 2.2 item 3): the presentation fields of catalog rows, read typed so a game's
//! screens pick a look from the data instead of a table keyed by skill, weapon family or terrain id. No rule reads
//! them. The values are the game's (look keys of its effect library, body clip names, prop art keys, colours); the kit
//! only carries them, with the schema defaults (`kits/rpg/schema.json`, every field `omit_default`) as fallbacks.
//!
//! - skills: `fx` (the use's look key) and `clip` (the body clip the actor tries first): `SkillDef::fx`, `SkillDef::clip`;
//! - item families: `fx`, `fx_far`, `attack_clips`, `area_clips`: `FamilyLook` (`Catalog::family_looks`);
//! - terrain: `relief`, `relief_depth`, `prop`, `color`, `face`: `TerrainLook` (the battle's terrain table carries one
//!   per type);
//! - statuses: the kit status table's `fx` (`rpgkit_rules::status::StatusRow::fx`).

use aegis_core::value::*;

/// The look key field of a skill, an item family and a status row.
pub const FX_KEY: &str = "fx";
/// A skill's body clip field.
pub const CLIP_KEY: &str = "clip";
/// A family's look at 2 or more cells.
pub const FX_FAR_KEY: &str = "fx_far";
/// A family's clips for a one-target attack and for an area, before `attack`.
pub const ATTACK_CLIPS_KEY: &str = "attack_clips";
pub const AREA_CLIPS_KEY: &str = "area_clips";
/// The terrain look fields.
pub const RELIEF_KEY: &str = "relief";
pub const RELIEF_DEPTH_KEY: &str = "relief_depth";
pub const PROP_KEY: &str = "prop";
pub const COLOR_KEY: &str = "color";
pub const FACE_KEY: &str = "face";

/// The look of a weapon family's strikes (its item family row).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FamilyLook {
    /// The look key of a strike with the family: a blow (melee) or a shot (ranged); "" = the game's default.
    pub fx: String,
    /// The look key of a strike at 2 or more cells, before the shot ("" = none).
    pub fx_far: String,
    /// The body clips a one-target attack tries before `attack`, best first.
    pub attack_clips: Vec<String>,
    /// The body clips an area attack tries before `attack`, best first.
    pub area_clips: Vec<String>,
}

impl FamilyLook {
    pub fn from_obj(o: &Obj) -> Self {
        FamilyLook { fx: string(o, FX_KEY, ""), fx_far: string(o, FX_FAR_KEY, ""), attack_clips: strings(o, ATTACK_CLIPS_KEY), area_clips: strings(o, AREA_CLIPS_KEY) }
    }
}

/// How the tilted board draws a terrain tile (the field `relief`; "" = `Ground`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Relief {
    /// Flat ground.
    #[default]
    Ground,
    /// Lowered by the relief depth.
    Sunken,
    /// A standing prop on the ground of its neighbours.
    Standing,
    /// Lifted, with a cap and a face.
    Block,
}

impl Relief {
    /// The field values in variant order ("" is `Ground`).
    pub const IDS: [&'static str; 4] = ["", "sunken", "standing", "block"];

    /// The relief of field value `id` (an unknown value is `Ground`).
    pub fn parse(id: &str) -> Self {
        match id {
            "sunken" => Relief::Sunken,
            "standing" => Relief::Standing,
            "block" => Relief::Block,
            _ => Relief::Ground,
        }
    }
}

/// The board's front face under a bottom-row tile (the field `face`; "" = `Earth`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Face {
    Wall,
    Water,
    Lava,
    #[default]
    Earth,
}

impl Face {
    /// The field values ("" is `Earth`).
    pub const IDS: [&'static str; 4] = ["", "wall", "water", "lava"];

    /// The face of field value `id` (an unknown value is `Earth`).
    pub fn parse(id: &str) -> Self {
        match id {
            "wall" => Face::Wall,
            "water" => Face::Water,
            "lava" => Face::Lava,
            _ => Face::Earth,
        }
    }
}

/// A terrain type's look (its terrain row; a type without a row, or a row without the fields, is flat ground with no
/// prop, no colour of its own and an earth face).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TerrainLook {
    pub relief: Relief,
    /// Px a sunken tile is lowered (0 for every other relief).
    pub depth: i32,
    /// The prop art key ("" = none).
    pub prop: String,
    /// The flat colour (None: the game's default).
    pub color: Option<[u8; 3]>,
    pub face: Face,
}

impl TerrainLook {
    /// The look of terrain row `row` (None: the defaults).
    pub fn of(row: Option<&Obj>) -> Self {
        let Some(r) = row else { return Self::default() };
        let relief = Relief::parse(&string(r, RELIEF_KEY, ""));
        TerrainLook {
            relief,
            depth: if relief == Relief::Sunken { int(r, RELIEF_DEPTH_KEY, 0).max(0) } else { 0 },
            prop: string(r, PROP_KEY, ""),
            color: parse_color(&string(r, COLOR_KEY, "")),
            face: Face::parse(&string(r, FACE_KEY, "")),
        }
    }
}

/// `#rrggbb` (or `rrggbb`) as its three channels; None for anything else (the empty default included).
pub fn parse_color(s: &str) -> Option<[u8; 3]> {
    let hex = s.strip_prefix('#').unwrap_or(s);
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let ch = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some([ch(0)?, ch(2)?, ch(4)?])
}
