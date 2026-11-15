//! PR5-33 (LAYER-UP-DATA, audit 2026-11-10 2.2 item 3): the presentation fields of catalog rows, read typed so a game's
//! screens pick a look from the data instead of a table keyed by skill, weapon family or terrain id. No rule reads
//! them. The values are the game's (look keys of its effect library, body clip names, prop art keys, colours); the kit
//! only carries them, with the schema defaults (`kits/rpg/schema.json`, every field `omit_default`) as fallbacks.
//!
//! - skills: `fx` (the use's look key) and `clip` (the body clip the actor tries first): `SkillDef::fx`, `SkillDef::clip`;
//! - item families: `fx`, `fx_far`, `attack_clips`, `area_clips`: `FamilyLook` (`Catalog::family_looks`);
//! - terrain: `relief`, `relief_depth`, `prop`, `color`, `face`: `TerrainLook` (the battle's terrain table carries one
//!   per type);
//! - statuses: the kit status table's `fx` (`rpgkit_rules::status::StatusRow::fx`);
//! - PR5-53 (VFX-DATA1): the parameters of each look key (colours, speeds, particle counts, lengths, rings, flashes,
//!   trails, orbits, tints): `FxPreset`, one row of the kind `fx_presets` per key (`Catalog::fx_presets`).

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

// ---- Effect presets (PR5-53 VFX-DATA1) ---------------------------------------------------------------------------

/// PR5-53 (VFX-DATA1, FB-231 (d)): the catalog kind of the effect presets (file `data/catalog/fx_presets.json`; a
/// missing file is an empty table). A row's id is a look key: a skill's or an item family's `fx` (a family's `fx_far`)
/// and a status row's `fx` name one, and the row carries the look's parameters (colours, speeds, particle counts,
/// lengths), so several rows sharing a look share its parameters. No row for a key: the game's own look of that key.
pub const FX_PRESETS_KIND: &str = "fx_presets";

/// An effect colour: red, green, blue, alpha.
pub type Rgba = [u8; 4];

/// The preset fields (`kits/rpg/schema.json` kind `fx_presets`, every one but `id` and `role` `omit_default`).
pub const FX_ROLE_KEY: &str = "role";
pub const FX_DELIVERY_KEY: &str = "delivery";
pub const FX_SPEED_KEY: &str = "speed";
pub const FX_ARC_KEY: &str = "arc";
pub const FX_TRAIL_KEY: &str = "trail";
pub const FX_SHOT_KEY: &str = "shot";
pub const FX_RING_KEY: &str = "ring";
pub const FX_FLASH_KEY: &str = "flash";
pub const FX_SPARKS_KEY: &str = "sparks";
pub const FX_BEAM_KEY: &str = "beam";
pub const FX_WAVE_KEY: &str = "wave";
pub const FX_VOLLEY_KEY: &str = "volley";
pub const FX_DROP_KEY: &str = "drop";
pub const FX_AIM_RING_KEY: &str = "aim_ring";
pub const FX_AIM_FLASH_KEY: &str = "aim_flash";
pub const FX_ON_ACTOR_KEY: &str = "on_actor";
pub const FX_SLOT_KEY: &str = "slot";
pub const FX_DY_KEY: &str = "dy";
pub const FX_PERIOD_KEY: &str = "period";
pub const FX_ORBIT_KEY: &str = "orbit";
pub const FX_TINT_KEY: &str = "tint";

/// The keys of each parameter object (anything else is refused by the content check).
pub const TRAIL_KEYS: [&str; 6] = ["radius", "from", "to", "width", "color", "ms"];
pub const SHOT_KEYS: [&str; 5] = ["look", "size", "color", "color2", "tail"];
pub const RING_KEYS: [&str; 4] = ["r0", "r1", "color", "ms"];
pub const FLASH_KEYS: [&str; 4] = ["rx", "ry", "color", "ms"];
pub const SPARKS_KEYS: [&str; 10] = ["count", "color", "color2", "dir", "spread", "speed", "gravity", "size", "ms", "foot"];
pub const BEAM_KEYS: [&str; 4] = ["width", "core", "glow", "ms"];
pub const ORBIT_KEYS: [&str; 9] = ["rx", "ry", "ms", "count", "size", "color", "color2", "phase", "star"];
pub const TINT_KEYS: [&str; 3] = ["color", "lo", "hi"];

/// The defaults of the parameter keys a row may leave out: a burst's gravity and dot size.
pub const SPARKS_GRAVITY: f32 = 120.0;
pub const SPARKS_SIZE: u8 = 1;

/// Where a preset applies (its `role`): a melee weapon family's blow, a ranged weapon family's shot, a skill's strike
/// (also a family's `fx_far`), a support skill's look, or a status row's look on the token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FxRole {
    Blow,
    Shot,
    #[default]
    Strike,
    Support,
    Status,
}

impl FxRole {
    /// The field values in variant order.
    pub const IDS: [&'static str; 5] = ["blow", "shot", "strike", "support", "status"];

    pub fn parse(id: &str) -> Self {
        match id {
            "blow" => FxRole::Blow,
            "shot" => FxRole::Shot,
            "support" => FxRole::Support,
            "status" => FxRole::Status,
            _ => FxRole::Strike,
        }
    }

    pub fn id(self) -> &'static str {
        Self::IDS[self as usize]
    }
}

/// How an action's effect reaches its target (`delivery`; "" none).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FxDelivery {
    #[default]
    None,
    /// A blow, at the attack clip's impact frame.
    Melee,
    /// A shot flying `speed` px/s raised `arc` px at its middle.
    Shot,
    /// A beam, landing as it leaves.
    Beam,
    /// Support and items, at the launch.
    Cast,
}

impl FxDelivery {
    pub const IDS: [&'static str; 5] = ["", "melee", "shot", "beam", "cast"];

    pub fn parse(id: &str) -> Self {
        match id {
            "melee" => FxDelivery::Melee,
            "shot" => FxDelivery::Shot,
            "beam" => FxDelivery::Beam,
            "cast" => FxDelivery::Cast,
            _ => FxDelivery::None,
        }
    }
}

/// Where a status look sits on a token (`slot`; "" none): a loop at the head, body or feet, a tint, or the token's
/// alpha (a fade).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FxSlot {
    #[default]
    None,
    Head,
    Body,
    Feet,
    Tint,
    Alpha,
}

impl FxSlot {
    pub const IDS: [&'static str; 6] = ["", "head", "body", "feet", "tint", "alpha"];

    pub fn parse(id: &str) -> Self {
        match id {
            "head" => FxSlot::Head,
            "body" => FxSlot::Body,
            "feet" => FxSlot::Feet,
            "tint" => FxSlot::Tint,
            "alpha" => FxSlot::Alpha,
            _ => FxSlot::None,
        }
    }
}

/// A projectile's shape (`shot.look`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FxShotLook {
    /// A shaft `size` px long and a 2 px head (`color` the shaft, `color2` the head).
    #[default]
    Arrow,
    /// A shaft `size` px long and a 1 px head.
    Bolt,
    /// A `size` px square with a highlight (`color2`).
    Stone,
    /// A core of radius `size` (`color`), a glow (`color2`) and `tail` trailing dots.
    Orb,
}

impl FxShotLook {
    pub const IDS: [&'static str; 4] = ["arrow", "bolt", "stone", "orb"];

    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "arrow" => Some(FxShotLook::Arrow),
            "bolt" => Some(FxShotLook::Bolt),
            "stone" => Some(FxShotLook::Stone),
            "orb" => Some(FxShotLook::Orb),
            _ => None,
        }
    }
}

/// An arc trail (`trail`): radius, the arc from and to (deg; 0 = toward the target, negative = up), width, colour and
/// length (ms; 0 = the game's swing length).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FxTrail {
    pub radius: f32,
    pub from: f32,
    pub to: f32,
    pub width: u8,
    pub color: Rgba,
    pub ms: u32,
}

/// A projectile (`shot`): its shape, size, two colours and an orb's tail dots.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FxShot {
    pub look: FxShotLook,
    pub size: f32,
    pub color: Rgba,
    pub color2: Rgba,
    pub tail: u8,
}

/// A growing ring (`ring`, `wave`, `aim_ring`): radius `r0` to `r1`, colour, length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FxRing {
    pub r0: f32,
    pub r1: f32,
    pub color: Rgba,
    pub ms: u32,
}

/// A flash (`flash`, `aim_flash`): an `rx` x `ry` ellipse, colour, length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FxFlash {
    pub rx: f32,
    pub ry: f32,
    pub color: Rgba,
    pub ms: u32,
}

/// A spark burst (`sparks`): count, two colours, the direction (None: along the attack), spread, speed, gravity, dot
/// size, life (0: the game's spark length) and whether it rises from the foot point instead of the chest.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FxSparks {
    pub count: u8,
    pub colors: [Rgba; 2],
    pub dir: Option<f32>,
    pub spread: f32,
    pub speed: f32,
    pub gravity: f32,
    pub size: u8,
    pub ms: u32,
    pub foot: bool,
}

/// A beam (`beam`): width, core and glow colours, life.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FxBeam {
    pub width: u8,
    pub core: Rgba,
    pub glow: Rgba,
    pub ms: u32,
}

/// Dots circling an ellipse (`orbit`): radii, period, count, dot size, two colours, start angle (deg), stars.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FxOrbit {
    pub rx: f32,
    pub ry: f32,
    pub ms: u32,
    pub count: u8,
    pub size: u8,
    pub colors: [Rgba; 2],
    pub phase: f32,
    pub star: bool,
}

/// A body tint (`tint`): its colour and its alpha from `lo` to `hi` and back over the row's `period`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FxTint {
    pub color: Rgba,
    pub lo: u8,
    pub hi: u8,
}

/// One effect preset row (kind `fx_presets`), typed. An action look (roles blow, shot, strike, support) reads
/// `delivery` to `on_actor`; a status look reads `slot` to `tint`. A parameter object a row leaves out is None.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FxPreset {
    pub id: String,
    pub role: FxRole,
    pub delivery: FxDelivery,
    /// A shot's speed (px/s) and arc (px at its middle).
    pub speed: f32,
    pub arc: f32,
    pub trail: Option<FxTrail>,
    pub shot: Option<FxShot>,
    /// On each unit the action lands on: a ring, a flash and sparks.
    pub ring: Option<FxRing>,
    pub flash: Option<FxFlash>,
    pub sparks: Option<FxSparks>,
    pub beam: Option<FxBeam>,
    /// Rings along an area's cells from the actor out, on the ground.
    pub wave: Option<FxRing>,
    /// One shot per area cell, at most this many (0: one shot).
    pub volley: u8,
    /// A shot falling from this many px above its target (0: from the actor).
    pub drop: f32,
    /// An area's aim: a ring (its `r1` per cell of the area's radius) and a flash.
    pub aim_ring: Option<FxRing>,
    pub aim_flash: Option<FxFlash>,
    /// The impact plays on the actor instead of the target.
    pub on_actor: bool,
    /// A status look's slot, its loop's lowering (px) and period (ms; a tint's too).
    pub slot: FxSlot,
    pub dy: f32,
    pub period: u32,
    pub orbit: Option<FxOrbit>,
    pub tint: Option<FxTint>,
}

fn f32_of(o: &Obj, key: &str, default: f32) -> f32 {
    float(o, key, default as f64) as f32
}

fn u8_of(o: &Obj, key: &str, default: u8) -> u8 {
    int(o, key, default as i32).clamp(0, 255) as u8
}

fn u32_of(o: &Obj, key: &str, default: u32) -> u32 {
    int(o, key, default as i32).max(0) as u32
}

/// A colour key of a parameter object (opaque black when it does not parse: the content check refuses it).
fn rgba_of(o: &Obj, key: &str) -> Rgba {
    parse_rgba(&string(o, key, "")).unwrap_or([0, 0, 0, 255])
}

/// The two colours of a parameter object: `color`, then `color2` (left out: the first again).
fn colors_of(o: &Obj) -> [Rgba; 2] {
    let first = rgba_of(o, "color");
    [first, if o.contains_key("color2") { rgba_of(o, "color2") } else { first }]
}

/// The parameter object `key` of row `o` (None: left out, empty or not an object).
fn part<'a>(o: &'a Obj, key: &str) -> Option<&'a Obj> {
    obj(o, key).filter(|p| !p.is_empty())
}

fn ring_of(p: &Obj) -> FxRing {
    FxRing { r0: f32_of(p, "r0", 0.0), r1: f32_of(p, "r1", 0.0), color: rgba_of(p, "color"), ms: u32_of(p, "ms", 0) }
}

fn flash_of(p: &Obj) -> FxFlash {
    FxFlash { rx: f32_of(p, "rx", 0.0), ry: f32_of(p, "ry", 0.0), color: rgba_of(p, "color"), ms: u32_of(p, "ms", 0) }
}

impl FxPreset {
    pub fn from_obj(o: &Obj) -> Self {
        FxPreset {
            id: string(o, "id", ""),
            role: FxRole::parse(&string(o, FX_ROLE_KEY, "")),
            delivery: FxDelivery::parse(&string(o, FX_DELIVERY_KEY, "")),
            speed: f32_of(o, FX_SPEED_KEY, 0.0),
            arc: f32_of(o, FX_ARC_KEY, 0.0),
            trail: part(o, FX_TRAIL_KEY).map(|p| FxTrail { radius: f32_of(p, "radius", 0.0), from: f32_of(p, "from", 0.0), to: f32_of(p, "to", 0.0), width: u8_of(p, "width", 1), color: rgba_of(p, "color"), ms: u32_of(p, "ms", 0) }),
            shot: part(o, FX_SHOT_KEY).map(|p| {
                let [color, color2] = colors_of(p);
                FxShot { look: FxShotLook::parse(&string(p, "look", "")).unwrap_or_default(), size: f32_of(p, "size", 0.0), color, color2, tail: u8_of(p, "tail", 0) }
            }),
            ring: part(o, FX_RING_KEY).map(ring_of),
            flash: part(o, FX_FLASH_KEY).map(flash_of),
            sparks: part(o, FX_SPARKS_KEY).map(|p| FxSparks {
                count: u8_of(p, "count", 0),
                colors: colors_of(p),
                dir: p.get("dir").filter(|v| v.is_number()).map(|v| to_float(v, 0.0) as f32),
                spread: f32_of(p, "spread", 0.0),
                speed: f32_of(p, "speed", 0.0),
                gravity: f32_of(p, "gravity", SPARKS_GRAVITY),
                size: u8_of(p, "size", SPARKS_SIZE),
                ms: u32_of(p, "ms", 0),
                foot: boolean(p, "foot", false),
            }),
            beam: part(o, FX_BEAM_KEY).map(|p| FxBeam { width: u8_of(p, "width", 1), core: rgba_of(p, "core"), glow: rgba_of(p, "glow"), ms: u32_of(p, "ms", 0) }),
            wave: part(o, FX_WAVE_KEY).map(ring_of),
            volley: u8_of(o, FX_VOLLEY_KEY, 0),
            drop: f32_of(o, FX_DROP_KEY, 0.0),
            aim_ring: part(o, FX_AIM_RING_KEY).map(ring_of),
            aim_flash: part(o, FX_AIM_FLASH_KEY).map(flash_of),
            on_actor: boolean(o, FX_ON_ACTOR_KEY, false),
            slot: FxSlot::parse(&string(o, FX_SLOT_KEY, "")),
            dy: f32_of(o, FX_DY_KEY, 0.0),
            period: u32_of(o, FX_PERIOD_KEY, 0),
            orbit: part(o, FX_ORBIT_KEY).map(|p| FxOrbit {
                rx: f32_of(p, "rx", 0.0),
                ry: f32_of(p, "ry", 0.0),
                ms: u32_of(p, "ms", 0),
                count: u8_of(p, "count", 1),
                size: u8_of(p, "size", 1),
                colors: colors_of(p),
                phase: f32_of(p, "phase", 0.0),
                star: boolean(p, "star", false),
            }),
            tint: part(o, FX_TINT_KEY).map(|p| FxTint { color: rgba_of(p, "color"), lo: u8_of(p, "lo", 0), hi: u8_of(p, "hi", 0) }),
        }
    }
}

/// `#rrggbb` (alpha 255) or `#rrggbbaa` as its four channels; None for anything else.
pub fn parse_rgba(s: &str) -> Option<Rgba> {
    let hex = s.strip_prefix('#')?;
    if !(hex.len() == 6 || hex.len() == 8) || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let ch = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some([ch(0)?, ch(2)?, ch(4)?, if hex.len() == 8 { ch(6)? } else { 255 }])
}

/// The data form of colour `c`: `#rrggbb` when opaque, else `#rrggbbaa` (lowercase; `parse_rgba` reads it back).
pub fn rgba_hex(c: Rgba) -> String {
    if c[3] == 255 {
        format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
    } else {
        format!("#{:02x}{:02x}{:02x}{:02x}", c[0], c[1], c[2], c[3])
    }
}
