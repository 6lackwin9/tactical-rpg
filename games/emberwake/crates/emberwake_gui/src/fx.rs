//! Battle effects, the timeline part (FX1-L1, spec games/emberwake/docs/specs/FX1.md section 4): how a popped action
//! reaches its target (`Delivery`, 4.3), when it lands (`impact_at`, 4.2 step 2 and decision "Attack timing on the
//! board" A: the attack clip's `impact` frame), the queue of impacts waiting to show the step's hit (`Impact`, rules I
//! and F; rule P waits for them in `BattleScreen::tick`), the struck token's shake (4.5) and the anchors of both board
//! views (4.1). Slice 2: the looks (`style`, 4.3 and 4.4.1-4.4.2 with the `arcane_bolt` and `shield_bash` rows), the
//! effects they push on the engine's live list (`delivery_effects` at the pop, `impact_effects` at the impact, with the
//! crit and mitigated modifiers) and the battle shakes (4.5). FX1-L2: the skill rows of 4.4.3 (beams, falling orbs,
//! volleys, shockwaves, sweeps, area aims), support and items (4.4.4), falls, the miss's overshoot and sidestep (4.3),
//! a swing up or down turned toward its target, and the impact budget under the live list's cap. Engine-only: Godot
//! had token tweens in `scripts/ui/combat/fx_presets.gd` (its hit shake is ported here without the bug that kept it
//! on one side; its miss sidestep smaller). Presentation only: nothing here reads or changes the battle.
//! STATUSFX1-L (spec STATUSFX1.md sections 3-4): the status looks on the tokens (`STATUS_LOOKS`: looped effects at the
//! head, body and feet, tints, stealth's alpha), one per slot cycling (`status_frame`), pending at half alpha, the
//! apply burst at the impact.

use crate::stage::{Geom, View};
use crate::tuning::Tuning;
use aegis_ui::canvas::Rgba;
use aegis_ui::fx::{Beam, Effect, Flash, Layer, Look, Looped, Orbit, Prim, Projectile, Pt, Ring, Shake, ShakeTable, Sparks, Trail, MAX_LIVE};
use emberwake_battle::grid::V2;

/// Weapon families whose strikes at range read as arcane (4.3 rule 2).
pub const ARCANE_FAMILIES: [&str; 4] = ["staff", "wand", "orb", "tome"];

/// How an action's effect reaches its target (4.3; its timing, 4.2 step 2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Delivery {
    /// No effect (move, defend, focus, rest, status; 4.3 rule 7): the step's changes show at once.
    None,
    /// A swing or a blow: it lands at the attack clip's impact frame (`melee_hit_ms` for a clip without one).
    Melee,
    /// A shot flying `speed` px/s, raised `arc` px at its middle: it leaves at the attack clip's impact frame
    /// (`launch_ms` without one, or with `shot_from_clip` off) and lands after its travel.
    Shot { speed: f32, arc: f32 },
    /// A beam (FX1-L2, `arc_lance`): it lands as it leaves, at the attack clip's impact frame (as a shot's release;
    /// `launch_ms` without one).
    Beam,
    /// Support and items: they land at `launch_ms`.
    Cast,
}

/// What `delivery` reads of a popped action: the command kind (`attack`, `area`, `support`, `item`, else none), the
/// skill id, the actor's main weapon family ("" none), whether that weapon is ranged (`Equipment::is_ranged_weapon`)
/// or deals magic damage, and the grid distance to the target or aim; for an item, whether it is used on its user and
/// whether it deals damage (a damage consumable).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Act<'a> {
    pub kind: &'a str,
    pub skill: &'a str,
    pub family: &'a str,
    pub ranged: bool,
    pub magic: bool,
    pub dist: i32,
    pub on_self: bool,
    pub harms: bool,
}

/// A projectile's flight over `px` at `speed` px/s in clock ms, clamped to the tuning's travel range (4.2 step 2).
pub fn travel_ms(px: f32, speed: f32, t: &Tuning) -> u64 {
    let ms = if speed > 0.0 { (px.max(0.0) / speed * 1000.0).round() as u64 } else { t.travel_max_ms };
    ms.clamp(t.travel_min_ms, t.travel_max_ms)
}

/// The clock ms the action lands (4.2 step 2), from `t0` (the pop, or the end of the actor's slide): a blow at the
/// attack clip's impact frame (`clip_ms`, the ms before it), else `melee_hit_ms`; a shot leaves at `clip_ms` (with
/// `shot_from_clip`) or `launch_ms` and flies `path_px`; a cast at `launch_ms`. None: no delivery (at once).
pub fn impact_at(d: Delivery, t0: u64, clip_ms: Option<u64>, path_px: f32, t: &Tuning) -> Option<u64> {
    match d {
        Delivery::None => None,
        Delivery::Melee => Some(t0 + clip_ms.unwrap_or(t.melee_hit_ms)),
        Delivery::Shot { speed, .. } => {
            let launch = clip_ms.filter(|_| t.shot_from_clip).unwrap_or(t.launch_ms);
            Some(t0 + launch + travel_ms(path_px, speed, t))
        }
        Delivery::Beam => Some(t0 + clip_ms.filter(|_| t.shot_from_clip).unwrap_or(t.launch_ms)),
        Delivery::Cast => Some(t0 + t.launch_ms),
    }
}

/// An effect's anchor on a unit at `c` (4.1): the Stage's chest point (the foot raised a third of the sprite); the
/// Flat token's centre raised 2 px.
pub fn anchor(geo: &Geom, c: V2) -> (f32, f32) {
    let (x, y) = geo.chest(c);
    match geo.view {
        View::Stage => (x as f32, y as f32),
        View::Flat => (x as f32, y as f32 - 2.0),
    }
}

/// What one unit shows on the board after a step (rule I: the board keeps the state before the step until the impact).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shown {
    pub hp: i32,
    pub alive: bool,
    pub guarded: bool,
    /// WINDUP1 W6, W7: its poise is broken this round; it has the Staggered status (active or pending); a wind-up of
    /// its shows after the step (one that ended waits for the impact).
    pub broken: bool,
    pub staggered: bool,
    pub winding: bool,
}

/// The board feedback of one step, held until its action lands (rule I) or fired at once.
#[derive(Debug, Clone, PartialEq)]
pub struct Impact {
    /// The clock ms it lands; None until the actor's clip is known (`BattleScreen::resolve_impacts`).
    pub at: Option<u64>,
    /// The popped action: its actor, the delivery, `t0`, its actor's clip candidates, the cells it goes between.
    pub actor: usize,
    pub delivery: Delivery,
    pub t0: u64,
    pub clips: Vec<&'static str>,
    /// WINDUP1 W4: the actor's clip starts at its `strike` frame (it struck from a wind-up the board showed), so the
    /// impact comes the ms between `strike` and `impact` after `t0`.
    pub strike: bool,
    pub from: V2,
    pub to: V2,
    /// Each unit's state after the step (the tokens react to it at the impact).
    pub after: Vec<Shown>,
    /// The floats (unit, text, colour), the units whose HP fell (hit flash and token shake), the sounds (sfx keys).
    pub floats: Vec<(usize, String, Rgba)>,
    pub struck: Vec<usize>,
    pub sounds: Vec<String>,
    /// Slice 2: the action's look, an area (its impact effects play on each struck unit), its effect seed, its skill's
    /// blocks (a Heavy hit shakes), and from the step's log the modifiers and whether it broke a unit.
    pub style: Style,
    pub area: bool,
    pub seed: u64,
    pub blocks: i32,
    pub mods: Mods,
    pub broke: bool,
    /// The delivery effects are on the live list (pushed once, when `at` is known).
    pub launched: bool,
    /// FX1-L2: an area's cells (a volley's arrows, a shockwave's rings, a beam's end), its radius in cells around the
    /// aim (the aim ring), and an attack's target unit (a miss's sidestep).
    pub cells: Vec<V2>,
    pub radius: i32,
    pub target: Option<usize>,
    /// STATUSFX1-L: each unit's statuses with a look after the step (`marks_of`; they show at the impact, rule I).
    pub marks: Vec<Vec<Mark>>,
    /// INPUT1 PB1: the action's impact tracers (they draw from the impact).
    pub traces: Vec<Trace>,
    /// PORTRAITFX1-P (5.3, 6.2 row 3): the step's portrait events (unit, event) the panel portraits react to at the
    /// impact.
    pub portrait: Vec<(usize, &'static str)>,
}

/// INPUT1 PB1: one impact tracer, a straight line between two cells' token centres in its colour, dashed for a miss.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Trace {
    pub from: V2,
    pub to: V2,
    pub col: Rgba,
    pub dashed: bool,
}

impl Impact {
    /// Any HP fell in the step (the hit-stop holds).
    pub fn hit(&self) -> bool {
        !self.struck.is_empty()
    }
}

/// The hit-stop a firing impact holds (4.2 step 3): `hit_stop_ms` when any HP fell, else none.
pub fn hit_stop(i: &Impact, t: &Tuning) -> f64 {
    if i.hit() {
        t.hit_stop_ms as f64
    } else {
        0.0
    }
}

/// 4.5, the struck token's shake: `px` to the right, then the left, switching every 30 wall ms from its impact for
/// `flash_ms`; 0 after it and under reduce motion (x only; FX1 fixes the shake that kept to one side).
pub fn token_shake(px: i32, age_ms: u64, flash_ms: u64, reduce_motion: bool) -> i32 {
    if reduce_motion || age_ms >= flash_ms {
        return 0;
    }
    if (age_ms / 30).is_multiple_of(2) {
        px
    } else {
        -px
    }
}

/// The hit flash's silhouette colour (FQ7: white at alpha 150).
pub const FLASH: Rgba = [255, 255, 255, 150];

/// 4.3, a miss: the target steps `SIDESTEP_PX` away from its attacker over `SIDESTEP_OUT_MS` and back over
/// `SIDESTEP_BACK_MS` (clock ms; Godot `FxPresets` miss, smaller for the 40 px tokens); a shot flies `OVERSHOOT_PX` past.
pub const SIDESTEP_PX: f32 = 3.0;
pub const SIDESTEP_OUT_MS: u64 = 80;
pub const SIDESTEP_BACK_MS: u64 = 140;
pub const OVERSHOOT_PX: f32 = 8.0;

/// How far (px) a sidestepping target stands from its place `age_ms` after the miss's impact: out over 80 ms, back
/// over 140 ms, then 0.
pub fn sidestep_px(age_ms: u64) -> f32 {
    if age_ms < SIDESTEP_OUT_MS {
        SIDESTEP_PX * age_ms as f32 / SIDESTEP_OUT_MS as f32
    } else if age_ms < SIDESTEP_OUT_MS + SIDESTEP_BACK_MS {
        SIDESTEP_PX * (1.0 - (age_ms - SIDESTEP_OUT_MS) as f32 / SIDESTEP_BACK_MS as f32)
    } else {
        0.0
    }
}

/// A sidestep: the unit, the clock ms of its miss's impact and the unit direction away from the attacker (board px).
pub type Sidestep = (usize, u64, Pt);

/// Unit `i`'s sidestep offset (whole px) at clock `now`: the latest sidestep it started (none: 0, 0).
pub fn sidestep_of(steps: &[Sidestep], i: usize, now: u64) -> (i32, i32) {
    match steps.iter().rev().find(|s| s.0 == i && now >= s.1) {
        Some(&(_, at, d)) => {
            let k = sidestep_px(now - at);
            ((d.0 * k).round() as i32, (d.1 * k).round() as i32)
        }
        None => (0, 0),
    }
}

/// The unit direction from `a` to `b` ((1, 0) when they meet).
pub fn unit_dir(a: Pt, b: Pt) -> Pt {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-4 {
        (1.0, 0.0)
    } else {
        (dx / len, dy / len)
    }
}

// ---- The looks (FX1 4.3, 4.4.1-4.4.2 and the L1 rows of 4.4.3; FX1_TASKS section 8) ------------------------------

/// The art bible's effect hues (FX1_TASKS 8): Polished Steel, Linen, Ash, Amber, Candle Gold, Resonant Orchid (FQ8)
/// and the crit hue Arterial Red.
pub const STEEL: Rgba = [0x9c, 0xa3, 0xaf, 255];
pub const LINEN: Rgba = [0xd9, 0xcb, 0xb4, 255];
pub const ASH: Rgba = [0x6b, 0x72, 0x80, 255];
pub const AMBER: Rgba = [0xd9, 0x77, 0x06, 255];
pub const GOLD: Rgba = [0xfb, 0xbf, 0x24, 255];
pub const ORCHID: Rgba = [0xc0, 0x84, 0xfc, 255];
pub const CYAN: Rgba = [0x06, 0xb6, 0xd4, 255];
pub const CRIT: Rgba = [0xe1, 0x1d, 0x48, 255];

const fn alpha(c: Rgba, a: u8) -> Rgba {
    [c[0], c[1], c[2], a]
}

/// A swing's trail (4.4.1): radius, the arc from and to (deg; 0 = toward the target, negative = up), width, colour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Swing {
    pub radius: f32,
    pub from_deg: f32,
    pub to_deg: f32,
    pub width: u8,
    pub col: Rgba,
}

/// A spark burst row: count, the two colours, the direction (None: along the attack, from the actor to the struck
/// unit), spread, speed, gravity and size (gravity 120 and size 1 unless the row gives them); FX1-L2: its life (0:
/// `spark_ms`) and whether it rises from the foot point instead of the chest (support motes).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Burst {
    pub count: u8,
    pub cols: [Rgba; 2],
    pub dir: Option<f32>,
    pub spread: f32,
    pub speed: f32,
    pub gravity: f32,
    pub size: u8,
    pub ms: u32,
    pub foot: bool,
}

/// A growing ring (radius `r0` to `r1`) or a flash (an `r0` x `r1` ellipse) on the struck unit, for `ms`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bloom {
    pub r0: f32,
    pub r1: f32,
    pub col: Rgba,
    pub ms: u32,
}

/// What an action shows on each unit it lands on (4.4: rings, flashes and sparks).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Hit {
    pub ring: Option<Bloom>,
    pub flash: Option<Bloom>,
    pub sparks: Option<Burst>,
}

/// A beam's look (4.4.3 `arc_lance`): width (1-3), core and glow colours, life.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub width: u8,
    pub core: Rgba,
    pub glow: Rgba,
    pub ms: u32,
}

/// What an area's aim shows (4.4.3 BURST rows): a ring from `ring.r0` to `ring.r1` px per cell of the area's radius
/// (32 x 0.6) and a flash, at the aim cell's foot point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aim {
    pub ring: Bloom,
    pub flash: Option<Bloom>,
}

/// An action's look (4.3): its delivery (and timing), the swing trail or the projectile, and its impact. FX1-L2 (the
/// rows of 4.4.3-4.4.4): a beam to the area's far end, rings along the area's cells from the actor out (`wave`, on
/// the ground), one shot per area cell (`volley`, at most that many; 0 one shot), a shot falling from `drop` px above
/// the target, the aim's ring and flash, and an impact on the actor instead of the target (`on_actor`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub delivery: Delivery,
    pub swing: Option<Swing>,
    pub shot: Option<Look>,
    pub hit: Hit,
    pub beam: Option<Ray>,
    pub wave: Option<Bloom>,
    pub volley: u8,
    pub drop: f32,
    pub aim: Option<Aim>,
    pub on_actor: bool,
}

const NONE: Style = Style { delivery: Delivery::None, swing: None, shot: None, hit: Hit { ring: None, flash: None, sparks: None }, beam: None, wave: None, volley: 0, drop: 0.0, aim: None, on_actor: false };

const fn burst(count: u8, cols: [Rgba; 2], spread: f32, speed: f32) -> Burst {
    Burst { count, cols, dir: None, spread, speed, gravity: 120.0, size: 1, ms: 0, foot: false }
}

const fn bloom(r0: f32, r1: f32, col: Rgba, a: u8, ms: u32) -> Option<Bloom> {
    Some(Bloom { r0, r1, col: alpha(col, a), ms })
}

const fn swing(radius: f32, from_deg: f32, to_deg: f32, width: u8, col: Rgba) -> Option<Swing> {
    Some(Swing { radius, from_deg, to_deg, width, col })
}

/// 4.4.1, a blow by the main weapon's family ("" and `unarmed` read as `fist`; a family the table does not name
/// swings as the sword).
pub fn melee_style(family: &str) -> Style {
    let blow = |swing: Option<Swing>, ring: Option<Bloom>, sparks: Burst| Style { delivery: Delivery::Melee, swing, hit: Hit { ring, flash: None, sparks: Some(sparks) }, ..NONE };
    match family {
        "axe" => blow(swing(13.0, -70.0, 60.0, 2, STEEL), None, burst(7, [STEEL, AMBER], 90.0, 70.0)),
        "dagger" => blow(swing(9.0, -30.0, 30.0, 1, STEEL), None, burst(4, [LINEN, STEEL], 60.0, 60.0)),
        "mace" | "hammer" | "staff" => blow(
            swing(12.0, -80.0, 40.0, 3, ASH),
            Some(Bloom { r0: 3.0, r1: 9.0, col: alpha(LINEN, 200), ms: 180 }),
            Burst { gravity: 160.0, size: 2, ..burst(8, [ASH, LINEN], 140.0, 50.0) },
        ),
        "" | "unarmed" | "fist" => blow(None, Some(Bloom { r0: 2.0, r1: 7.0, col: alpha(LINEN, 200), ms: 150 }), burst(5, [LINEN, LINEN], 120.0, 50.0)),
        _ => blow(swing(13.0, -60.0, 50.0, 2, STEEL), None, burst(6, [STEEL, LINEN], 90.0, 70.0)),
    }
}

/// 4.4.2, a shot by the ranged weapon's family (another ranged family shoots as the bow).
pub fn shot_style(family: &str) -> Style {
    let shot = |speed: f32, arc: f32, look: Look, sparks: Burst| Style { delivery: Delivery::Shot { speed, arc }, shot: Some(look), hit: Hit { sparks: Some(sparks), ..Hit::default() }, ..NONE };
    let bow_sparks = burst(4, [LINEN, STEEL], 70.0, 60.0);
    match family {
        "crossbow" => shot(720.0, 2.0, Look::Bolt { len: 5.0, shaft: STEEL, head: LINEN }, bow_sparks),
        "sling" => shot(480.0, 8.0, Look::Stone { size: 2.0, col: ASH, hi: LINEN }, burst(4, [ASH, LINEN], 120.0, 40.0)),
        _ => shot(560.0, 6.0, Look::Arrow { len: 7.0, shaft: LINEN, head: STEEL }, bow_sparks),
    }
}

/// 4.4.2's last row: the generic reach bolt (a strike at 2 or more cells with no other look).
pub const REACH_BOLT: Style = Style {
    delivery: Delivery::Shot { speed: 480.0, arc: 0.0 },
    shot: Some(Look::Orb { r: 1.5, core: LINEN, glow: STEEL, tail: 2 }),
    hit: Hit { ring: None, flash: bloom(4.0, 3.0, LINEN, 120, 100), sparks: None },
    ..NONE
};

/// 4.4.3 `arcane_bolt` (and 4.3 rule 2): an Orchid orb, a flash and sparks all round.
pub const ARCANE_BOLT: Style = Style {
    delivery: Delivery::Shot { speed: 420.0, arc: 0.0 },
    shot: Some(Look::Orb { r: 2.5, core: LINEN, glow: ORCHID, tail: 3 }),
    hit: Hit { ring: None, flash: bloom(5.0, 4.0, ORCHID, 150, 120), sparks: Some(burst(6, [ORCHID, LINEN], 360.0, 50.0)) },
    ..NONE
};

/// 4.4.3 `shield_bash`: no trail; a Steel ring and Steel and Gold sparks.
pub const SHIELD_BASH: Style = Style {
    delivery: Delivery::Melee,
    hit: Hit { ring: bloom(3.0, 10.0, STEEL, 220, 180), flash: None, sparks: Some(burst(8, [STEEL, GOLD], 120.0, 70.0)) },
    ..NONE
};

/// 4.4.3 `hex`: an Orchid orb on a small arc, an Orchid ring.
pub const HEX: Style = Style {
    delivery: Delivery::Shot { speed: 360.0, arc: 4.0 },
    shot: Some(Look::Orb { r: 2.0, core: ORCHID, glow: ORCHID, tail: 2 }),
    hit: Hit { ring: bloom(2.0, 8.0, ORCHID, 220, 200), flash: None, sparks: None },
    ..NONE
};

/// 4.4.3 `arc_lance` (LINE): a beam to the line's last cell, Orchid sparks on each struck unit (all round, as the
/// arcane bolt's: the row gives no spread or speed).
pub const ARC_LANCE: Style = Style {
    delivery: Delivery::Beam,
    beam: Some(Ray { width: 2, core: LINEN, glow: ORCHID, ms: 160 }),
    hit: Hit { ring: None, flash: None, sparks: Some(burst(4, [ORCHID, LINEN], 360.0, 50.0)) },
    ..NONE
};

/// The BURST aim (4.4.3 `firebomb`): a ring of 4 px growing to 32 x 0.6 px per cell of the area's radius.
pub const AIM_RING_PX_PER_CELL: f32 = 32.0 * 0.6;

/// 4.4.3 `firebomb`, `inferno` (BURST): a gold and amber orb lobbed to the aim; an amber ring and flash on the aim;
/// rising embers on each struck unit.
pub const FIREBOMB: Style = Style {
    delivery: Delivery::Shot { speed: 380.0, arc: 16.0 },
    shot: Some(Look::Orb { r: 3.0, core: GOLD, glow: AMBER, tail: 3 }),
    hit: Hit { ring: None, flash: None, sparks: Some(Burst { dir: Some(-90.0), gravity: -20.0, ..burst(10, [AMBER, GOLD], 160.0, 70.0) }) },
    aim: Some(Aim { ring: Bloom { r0: 4.0, r1: AIM_RING_PX_PER_CELL, col: alpha(AMBER, 230), ms: 260 }, flash: bloom(10.0, 7.0, AMBER, 140, 160) }),
    ..NONE
};

/// 4.4.3 `glacial_burst` (BURST): as the firebomb in Cyan (no flash), heavier sparks.
pub const GLACIAL_BURST: Style = Style {
    delivery: Delivery::Shot { speed: 380.0, arc: 16.0 },
    shot: Some(Look::Orb { r: 3.0, core: LINEN, glow: CYAN, tail: 3 }),
    hit: Hit { ring: None, flash: None, sparks: Some(Burst { dir: Some(-90.0), gravity: 80.0, ..burst(8, [CYAN, LINEN], 160.0, 70.0) }) },
    aim: Some(Aim { ring: Bloom { r0: 4.0, r1: AIM_RING_PX_PER_CELL, col: alpha(CYAN, 230), ms: 260 }, flash: None }),
    ..NONE
};

/// How far above the target's chest the `smite` orb starts.
pub const SMITE_DROP_PX: f32 = 48.0;

/// 4.4.3 `smite`, `judgement`: a gold orb falling onto the target; a gold flash and sparks upward (the row gives no
/// speed or tail: the bow's 60 and 2).
pub const SMITE: Style = Style {
    delivery: Delivery::Shot { speed: 600.0, arc: 0.0 },
    shot: Some(Look::Orb { r: 2.0, core: LINEN, glow: GOLD, tail: 2 }),
    drop: SMITE_DROP_PX,
    hit: Hit { ring: None, flash: bloom(8.0, 6.0, GOLD, 160, 140), sparks: Some(Burst { dir: Some(-90.0), ..burst(6, [GOLD, LINEN], 200.0, 60.0) }) },
    ..NONE
};

/// A volley's arrows launch this far apart (clock ms).
pub const VOLLEY_GAP_MS: u64 = 30;
/// A shockwave's rings start this far apart, cell by cell from the actor out (clock ms).
pub const WAVE_GAP_MS: u64 = 40;

/// 4.4.3 `shockwave` (LINE): Ash rings along the line (ground layer; the row gives no life: 200 ms) at the clip's
/// impact frame, Ash sparks on each struck unit (the row gives no spread or speed: the fist's 120 and 50).
pub const SHOCKWAVE: Style = Style {
    delivery: Delivery::Melee,
    wave: bloom(3.0, 12.0, ASH, 200, 200),
    hit: Hit { ring: None, flash: None, sparks: Some(Burst { gravity: 160.0, ..burst(5, [ASH, ASH], 120.0, 50.0) }) },
    ..NONE
};

/// 4.4.3 `haymaker`, `crushing_blow`: a short Linen swing, a ring and Linen and Ash sparks (the sword's spread and
/// speed).
pub const HAYMAKER: Style = Style {
    delivery: Delivery::Melee,
    swing: swing(10.0, -40.0, 60.0, 2, LINEN),
    hit: Hit { ring: bloom(2.0, 9.0, LINEN, 220, 180), flash: None, sparks: Some(burst(7, [LINEN, ASH], 90.0, 70.0)) },
    ..NONE
};

/// 4.4.4: the heal motes (`mend` and any support without its own row) in `col` and Linen: sparks rising from the
/// target's foot point for 500 ms and a soft flash at its chest.
const fn motes(col: Rgba) -> Style {
    Style {
        delivery: Delivery::Cast,
        hit: Hit {
            ring: None,
            flash: bloom(7.0, 5.0, col, 80, 300),
            sparks: Some(Burst { count: 8, cols: [col, LINEN], dir: Some(-90.0), spread: 50.0, speed: 30.0, gravity: -30.0, size: 1, ms: 500, foot: true }),
        },
        ..NONE
    }
}

pub const MEND: Style = motes(GOLD);
pub const PURIFY: Style = motes(CYAN);

/// 4.4.4 `aegis`: a Cyan ring and flash at the target's chest (the row gives the flash no life: the motes' 300 ms).
pub const AEGIS: Style = Style { delivery: Delivery::Cast, hit: Hit { ring: bloom(4.0, 12.0, CYAN, 220, 260), flash: bloom(7.0, 5.0, CYAN, 90, 300), sparks: None }, ..NONE };

/// 4.4.4 `rally`, `taunt_shout`: a wide Linen ring at the actor's chest.
pub const RALLY: Style = Style { delivery: Delivery::Cast, hit: Hit { ring: bloom(4.0, 20.0, LINEN, 200, 300), flash: None, sparks: None }, on_actor: true, ..NONE };

/// 4.4.4: the flask an item flies in when used on another unit.
pub const FLASK: Look = Look::Stone { size: 2.0, col: LINEN, hi: STEEL };

/// 4.4.4, an item: a damage consumable the firebomb's impact on its target, else the mend motes; thrown in a flask
/// (420 px/s, arc 10) when used on another unit, cast at once on its user.
pub fn item_style(on_self: bool, harms: bool) -> Style {
    let case = if harms { Style { delivery: Delivery::Cast, shot: None, ..FIREBOMB } } else { MEND };
    if on_self {
        case
    } else {
        Style { delivery: Delivery::Shot { speed: 420.0, arc: 10.0 }, shot: Some(FLASK), ..case }
    }
}

/// 4.4.3, the skill rows by id (the dash rows `lunge`, `pounce`, `charge_in` take their family's row: None).
pub fn skill_style(skill: &str) -> Option<Style> {
    Some(match skill {
        "arcane_bolt" => ARCANE_BOLT,
        "shield_bash" => SHIELD_BASH,
        "hex" => HEX,
        "arc_lance" => ARC_LANCE,
        "firebomb" | "inferno" => FIREBOMB,
        "glacial_burst" => GLACIAL_BURST,
        "smite" | "judgement" => SMITE,
        "volley" => Style { volley: 5, ..shot_style("bow") },
        "shockwave" => SHOCKWAVE,
        "sweep_kick" | "cyclone_kick" => Style { swing: swing(14.0, -150.0, 150.0, 2, LINEN), ..melee_style("fist") },
        "haymaker" | "crushing_blow" => HAYMAKER,
        _ => return None,
    })
}

/// 4.4.4, support by skill id (any other support: the mend motes).
pub fn support_style(skill: &str) -> Style {
    match skill {
        "purify" => PURIFY,
        "aegis" => AEGIS,
        "rally" | "taunt_shout" => RALLY,
        _ => MEND,
    }
}

/// 4.3, first match: the skill rows of 4.4.3, an arcane strike at 2 or more cells, a ranged weapon by family, any
/// other strike at 2 or more cells (the reach bolt), an adjacent strike by family; support and items by 4.4.4;
/// anything else has no effect.
pub fn style(a: Act) -> Style {
    let hits = a.kind == "attack" || a.kind == "area";
    if let Some(s) = skill_style(a.skill) {
        return s;
    }
    if hits && a.dist >= 2 && (ARCANE_FAMILIES.contains(&a.family) || a.magic) {
        return ARCANE_BOLT;
    }
    if hits && a.ranged {
        return shot_style(a.family);
    }
    if hits && a.dist >= 2 {
        return REACH_BOLT;
    }
    match a.kind {
        "attack" | "area" => melee_style(a.family),
        "support" => support_style(a.skill),
        "item" => item_style(a.on_self, a.harms),
        _ => NONE,
    }
}

/// The impact modifiers of a step (4.3) from its resolver log lines and whether any HP fell: a crit (an `[Attack]` line
/// with " CRIT"), a miss (`[Attack Miss]`, `[Area Miss]`) and a hit for 0 (an `[Attack]` line that lowered no HP).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Mods {
    pub crit: bool,
    pub zero: bool,
    pub miss: bool,
}

impl Mods {
    pub fn of(lines: &[String], hp_fell: bool) -> Mods {
        let attack = |l: &String| l.starts_with("[Attack]");
        let miss = lines.iter().any(|l| l.starts_with("[Attack Miss]") || l.starts_with("[Area Miss]"));
        let crit = lines.iter().any(|l| attack(l) && l.contains(" CRIT"));
        Mods { crit, zero: !miss && !hp_fell && lines.iter().any(attack), miss }
    }
}

/// 4.3's modifiers on a burst: a crit doubles the sparks (at most 24) and makes the second colour the crit hue; a hit
/// for 0 halves them (at least 1) in Steel; a miss has none.
pub fn modified(b: Burst, m: Mods) -> Option<Burst> {
    if m.miss {
        return None;
    }
    let mut b = b;
    if m.crit {
        b.count = b.count.saturating_mul(2).min(24);
        b.cols[1] = CRIT;
    }
    if m.zero {
        b.count = (b.count / 2).max(1);
        b.cols = [STEEL, STEEL];
    }
    Some(b)
}

fn toward(a: Pt, b: Pt, px: f32) -> Pt {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-4 {
        a
    } else {
        (a.0 + dx / len * px.min(len), a.1 + dy / len * px.min(len))
    }
}

fn deg(a: Pt, b: Pt) -> f32 {
    (b.1 - a.1).atan2(b.0 - a.0).to_degrees()
}

fn dist(a: Pt, b: Pt) -> f32 {
    ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt()
}

/// Where the effects of one action go (4.1): the actor's anchor, the target's (an area's aim: the aim cell's foot
/// point), the view's squash (Stage 0.75, Flat 1) for rings and trails, and whether the actor faces up or down to it
/// (DIR4-G: the vertical distance is the larger; its swing turns toward the target).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Place {
    pub from: Pt,
    pub to: Pt,
    pub squash: f32,
    pub vertical: bool,
}

/// A unit's two anchors (4.1): the chest (rings, flashes, sparks) and the foot point (support motes, falls).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    pub chest: Pt,
    pub foot: Pt,
}

/// A single shot's flight (4.2 step 2): from 6 px out of the actor toward the target, or (`drop`) from straight above
/// the target, to the target.
pub fn flight(s: &Style, pl: Place) -> (Pt, Pt) {
    if s.drop > 0.0 {
        ((pl.to.0, pl.to.1 - s.drop), pl.to)
    } else {
        (toward(pl.from, pl.to, 6.0), pl.to)
    }
}

/// The px a shot flies (`impact_at`'s path).
pub fn flight_px(s: &Style, pl: Place) -> f32 {
    let (a, b) = flight(s, pl);
    dist(a, b)
}

/// A volley's targets (4.4.3 `volley`): one per area cell (foot points), the aim last, at most `s.volley`.
pub fn volley_targets(s: &Style, cells: &[Pt], aim: Pt) -> Vec<Pt> {
    let mut v: Vec<Pt> = cells.iter().copied().filter(|c| dist(*c, aim) > 0.5).collect();
    v.push(aim);
    let keep = (s.volley.max(1) as usize).min(v.len());
    v.split_off(v.len() - keep)
}

/// How much later than its single delivery an action lands (4.4.3): a volley's last arrow leaves 30 ms per earlier
/// arrow after the first; a shockwave reaches the line's last cell 40 ms per cell after the first.
pub fn spread_ms(s: &Style, cells: &[Pt], aim: Pt) -> u64 {
    if s.volley > 0 {
        VOLLEY_GAP_MS * (volley_targets(s, cells, aim).len() as u64 - 1)
    } else if s.wave.is_some() {
        WAVE_GAP_MS * cells.len().saturating_sub(1) as u64
    } else {
        0
    }
}

/// The effects of the delivery (4.2 step 2, pushed once the impact time `at` is known; `cells` the area's foot points):
/// - the swing trail around the actor's anchor (mirrored toward a target on its left; turned toward a target above or
///   below), its head reaching the end of its arc at the impact (60 % of `trail_ms` before it, never before `t0`);
/// - the projectile from 6 px out toward the target (or falling from above it), launched its travel before the impact;
///   on a miss it flies `OVERSHOOT_PX` past the target at the same speed;
/// - a volley: one arrow per area cell, launched `VOLLEY_GAP_MS` apart, the one to the aim landing at the impact;
/// - a beam from the actor to the area's farthest cell (else the target) at the impact;
/// - a shockwave: rings on the ground layer along the area's cells from the actor out, `WAVE_GAP_MS` apart, the last
///   at the impact.
#[allow(clippy::too_many_arguments)]
pub fn delivery_effects(s: &Style, pl: Place, cells: &[Pt], m: Mods, t0: u64, at: u64, seed: u64, t: &Tuning) -> Vec<Effect> {
    let Place { from, to, squash, vertical } = pl;
    let mut out = Vec::new();
    if let Some(w) = s.swing {
        let start = at.saturating_sub(t.trail_ms * 6 / 10).max(t0);
        let (rot, mirror) = if vertical { (deg(from, to), false) } else { (0.0, to.0 < from.0) };
        let trail = Trail { at: from, radius: w.radius, from_deg: w.from_deg + rot, to_deg: w.to_deg + rot, ms: t.trail_ms as u32, width: w.width, col: w.col, mirror, squash };
        out.push(Effect::new(trail, start).seeded(seed));
    }
    if let (Some(look), Delivery::Shot { speed, arc }) = (s.shot, s.delivery) {
        if s.volley > 0 {
            let targets = volley_targets(s, cells, to);
            let n = targets.len() as u64;
            let main = travel_ms(dist(toward(from, to, 6.0), to), speed, t);
            let first = at.saturating_sub(main + VOLLEY_GAP_MS * (n - 1)).max(t0);
            for (k, &c) in targets.iter().enumerate() {
                let a = toward(from, c, 6.0);
                let p = Projectile { from: a, to: c, travel_ms: travel_ms(dist(a, c), speed, t) as u32, arc_px: arc, look };
                out.push(Effect::new(p, first + VOLLEY_GAP_MS * k as u64).seeded(seed ^ k as u64));
            }
        } else {
            let (a, b) = flight(s, pl);
            let len = dist(a, b);
            let travel = travel_ms(len, speed, t);
            let launch = at.saturating_sub(travel).max(t0);
            let (b, travel) = if m.miss && len > 1e-3 {
                let d = unit_dir(a, b);
                ((b.0 + d.0 * OVERSHOOT_PX, b.1 + d.1 * OVERSHOOT_PX), (travel as f32 * (len + OVERSHOOT_PX) / len).round() as u64)
            } else {
                (b, travel)
            };
            let p = Projectile { from: a, to: b, travel_ms: travel as u32, arc_px: arc, look };
            out.push(Effect::new(p, launch).seeded(seed));
        }
    }
    if let Some(r) = s.beam {
        let end = cells.iter().copied().fold(to, |best, c| if dist(from, c) > dist(from, best) { c } else { best });
        let b = Beam { from: toward(from, end, 6.0), to: end, ms: r.ms, width: r.width, core: r.core, glow: r.glow };
        out.push(Effect::new(b, at).seeded(seed));
    }
    if let Some(w) = s.wave {
        let mut cs = cells.to_vec();
        cs.sort_by(|a, b| dist(from, *a).total_cmp(&dist(from, *b)));
        let first = at.saturating_sub(WAVE_GAP_MS * cs.len().saturating_sub(1) as u64).max(t0);
        for (k, &c) in cs.iter().enumerate() {
            let ring = Ring { at: c, r0: w.r0, r1: w.r1, ms: w.ms, col: w.col, squash };
            out.push(Effect::new(ring, first + WAVE_GAP_MS * k as u64).on(Layer::Ground).seeded(seed ^ k as u64));
        }
    }
    out
}

/// The effects of the impact at `at` (4.2 step 3) on each spot of `hits` (the struck units', the target's, or the
/// actor's for `on_actor` rows): the row's ring and flash at the chest and its sparks (at the chest, or rising from the
/// foot point for support motes; along the attack from the actor unless the row gives a direction; the modifiers `m`
/// applied; `spark_ms` long unless the row gives a life), each hit seeded apart.
pub fn impact_effects(s: &Style, m: Mods, pl: Place, hits: &[Spot], at: u64, seed: u64, t: &Tuning) -> Vec<Effect> {
    let mut out = Vec::new();
    for (k, h) in hits.iter().enumerate() {
        let (p, seed) = (h.chest, seed ^ (k as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        if let Some(r) = s.hit.ring {
            out.push(Effect::new(Ring { at: p, r0: r.r0, r1: r.r1, ms: r.ms, col: r.col, squash: pl.squash }, at).seeded(seed));
        }
        if let Some(f) = s.hit.flash {
            out.push(Effect::new(Flash { at: p, rx: f.r0, ry: f.r1, ms: f.ms, col: f.col }, at).seeded(seed));
        }
        if let Some(b) = s.hit.sparks.and_then(|b| modified(b, m)) {
            let at_pt = if b.foot { h.foot } else { p };
            let dir_deg = b.dir.unwrap_or_else(|| deg(pl.from, p));
            let ms = if b.ms > 0 { b.ms } else { t.spark_ms as u32 };
            let sparks = Sparks { at: at_pt, ms, count: b.count, dir_deg, spread_deg: b.spread, speed: b.speed, gravity: b.gravity, size: b.size, cols: b.cols };
            out.push(Effect::new(sparks, at).seeded(seed));
        }
    }
    out
}

/// The aim's effects at the impact (4.3, 4.4.3): a BURST row's ring (its radius `r1` per cell of the area's `radius`,
/// at least one, on the ground layer) and flash at the aim cell's foot point; any other area plays its row's ring
/// there, unless a struck unit stands on the aim (its own ring shows).
#[allow(clippy::too_many_arguments)]
pub fn aim_effects(s: &Style, area: bool, aim: Pt, radius: i32, on_aim: bool, squash: f32, at: u64, seed: u64) -> Vec<Effect> {
    let seed = seed ^ 0xA1A1;
    let ring = |r: Bloom, r1: f32| Effect::new(Ring { at: aim, r0: r.r0, r1, ms: r.ms, col: r.col, squash }, at).on(Layer::Ground).seeded(seed);
    match (s.aim, s.hit.ring) {
        (Some(a), _) => {
            let mut out = vec![ring(a.ring, (a.ring.r1 * radius.max(1) as f32).max(a.ring.r0))];
            if let Some(f) = a.flash {
                out.push(Effect::new(Flash { at: aim, rx: f.r0, ry: f.r1, ms: f.ms, col: f.col }, at).seeded(seed));
            }
            out
        }
        (None, Some(r)) if area && !on_aim => vec![ring(r, r.r1)],
        _ => Vec::new(),
    }
}

/// 4.4.4, a unit falls: at its foot point on the ground layer, an Ash dust ring and heavy Ash and Linen dust.
pub fn fall_effects(foot: Pt, squash: f32, at: u64, seed: u64, t: &Tuning) -> Vec<Effect> {
    let seed = seed ^ 0xFA11;
    let ring = Ring { at: foot, r0: 3.0, r1: 10.0, ms: 300, col: alpha(ASH, 200), squash };
    let dust = Sparks { at: foot, ms: t.spark_ms as u32, count: 6, dir_deg: -90.0, spread_deg: 160.0, speed: 30.0, gravity: 120.0, size: 2, cols: [ASH, LINEN] };
    vec![Effect::new(ring, at).on(Layer::Ground).seeded(seed), Effect::new(dust, at).on(Layer::Ground).seeded(seed)]
}

/// The most effects one impact puts on the live list (`MAX_LIVE` less room for the deliveries in flight).
pub const IMPACT_BUDGET: usize = MAX_LIVE - 8;

/// FX1-L2 (an area hit on many units would pass the live list's cap and evict its own effects): keeps at most `max`
/// of an impact's effects by priority, sparks first (they carry the crit and mitigated reads), then rings (the aim's
/// first, as it is pushed first), then flashes; the kept ones keep their order.
pub fn budget(es: Vec<Effect>, max: usize) -> Vec<Effect> {
    if es.len() <= max {
        return es;
    }
    let rank = |e: &Effect| match e.kind() {
        "sparks" => 0,
        "flash" => 2,
        _ => 1,
    };
    let mut idx: Vec<usize> = (0..es.len()).collect();
    idx.sort_by_key(|&i| (rank(&es[i]), i));
    idx.truncate(max);
    idx.sort_unstable();
    idx.into_iter().map(|i| es[i]).collect()
}

/// The seed of a shake that no action carries (4.2 step 8): a break with no action to land, a fired phase (`key`: its
/// index among the fired phases); from the round and block, never the FX clock.
pub fn shake_seed(round: i32, block: i32, key: u64) -> u64 {
    seed_of(round, block, 0, 0) ^ key.wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x5348_414b
}

/// The `key` of a break's shake (`shake_seed`).
pub const BREAK_KEY: u64 = 0xB4EA;

/// An action's effect seed (4.2 step 8): from its round, block, index in the schedule and unit; effects never touch the
/// battle's RNG.
pub fn seed_of(round: i32, block: i32, idx: usize, unit: usize) -> u64 {
    ((round as u64 & 0xffff) << 48) ^ ((block as u64 & 0xffff) << 32) ^ ((idx as u64 & 0xffff) << 16) ^ (unit as u64 & 0xffff)
}

// ---- Shakes in battle (FX1 4.5) ------------------------------------------------------------------------------------

/// The preset of a Heavy hit and a break.
pub const HEAVY: &str = "heavy";

/// 4.5 rows 1-2: an impact shakes `heavy` when its skill takes 3 or more blocks and it lowered HP, or the step broke a
/// unit (a `[Break]` line); crits do not shake (FQ5).
pub fn heavy_shake(blocks: i32, hit: bool, broke: bool) -> bool {
    (blocks >= 3 && hit) || broke
}

/// A `[Break]` line among the step's resolver lines (RULESET1 P9).
pub fn broke(lines: &[String]) -> bool {
    lines.iter().any(|l| l.starts_with("[Break]"))
}

/// 4.5 row 3: of the presets the newly fired phases' `shake` effects name, the strongest (by `px`, then `ms`) in the
/// game's table; unknown names are skipped.
pub fn strongest<'a>(presets: impl IntoIterator<Item = &'a str>, table: &ShakeTable) -> Option<Shake> {
    presets.into_iter().filter_map(|p| table.get(p)).fold(None, |best: Option<Shake>, s| match best {
        Some(b) if (b.px, b.ms) >= (s.px, s.ms) => Some(b),
        _ => Some(s),
    })
}

// ---- Status looks (STATUSFX1-L, spec STATUSFX1.md sections 3-4) ---------------------------------------------------

/// The status hues not above (art bible): Arterial (the crit hue), Crimson, Iron and Blight Violet (SQ1).
pub const ARTERIAL: Rgba = CRIT;
pub const CRIMSON: Rgba = [0x99, 0x1b, 0x1b, 255];
pub const IRON: Rgba = [0x4b, 0x55, 0x63, 255];
pub const VIOLET: Rgba = [0x93, 0x33, 0xea, 255];

/// Where a status look sits on a token (S2: one look per slot and stealth's alpha).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Head,
    Body,
    Feet,
    Tint,
    Alpha,
}

/// The slots that cycle, in the agent link's order (S9; `Alpha` after them).
pub const SLOTS: [Slot; 4] = [Slot::Head, Slot::Body, Slot::Feet, Slot::Tint];

/// A status look's shape (section 3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    /// A looped primitive at its slot's anchor (`at` (0, 0), Stage px), lowered `dy` px, repeating every `period` ms.
    Loop { prim: Prim, dy: f32, period: u64 },
    /// The body silhouette in `col`, its alpha `lo` to `hi` and back (a sine) every `period` ms.
    Tint { col: Rgba, lo: u8, hi: u8, period: u64 },
    /// The token (sprite and outline) at `STEALTH_ALPHA`.
    Fade,
}

/// One row of the look table: the status id, its slot and its shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusLook {
    pub id: &'static str,
    pub slot: Slot,
    pub shape: Shape,
}

/// Stealth's token alpha (55 %).
pub const STEALTH_ALPHA: u8 = 140;
/// S4: the apply burst (a Ring r0 3 to r1 10 at alpha 200 for 220 ms) and the gap between several on one unit.
pub const BURST_MS: u32 = 220;
pub const BURST_GAP_MS: u64 = 40;

const O: Pt = (0.0, 0.0);

const fn on(id: &'static str, slot: Slot, prim: Prim, dy: f32, period: u64) -> StatusLook {
    StatusLook { id, slot, shape: Shape::Loop { prim, dy, period } }
}

const fn drift(count: u8, dir_deg: f32, spread_deg: f32, speed: f32, gravity: f32, ms: u32, cols: [Rgba; 2]) -> Prim {
    Prim::Sparks(Sparks { at: O, ms, count, dir_deg, spread_deg, speed, gravity, size: 2, cols })
}

const fn hoop(r0: f32, r1: f32, a: u8, ms: u32, col: Rgba) -> Prim {
    Prim::Ring(Ring { at: O, r0, r1, ms, col: alpha(col, a), squash: 1.0 })
}

const fn tint(id: &'static str, col: Rgba, lo: u8, hi: u8, period: u64) -> StatusLook {
    StatusLook { id, slot: Slot::Tint, shape: Shape::Tint { col, lo, hi, period } }
}

/// Section 3's table in its order (the order a slot cycles in, S2); `stagger` has none (WINDUP1's `dazed` loop). As
/// built (user 2026-10-29, the review board): the drifting sparks are 2 px and about twice as many, slow's and expose's
/// rings at full alpha, steadfast's flash at 160. S1.2-S4: `guard` (a multi-round guard's later rounds) a steel tint, a
/// placeholder until the board picks its look.
pub const STATUS_LOOKS: [StatusLook; 17] = [
    on("stun", Slot::Head, Prim::Orbit(Orbit { at: O, rx: 7.0, ry: 2.5, ms: 900, count: 3, size: 2, cols: [GOLD, LINEN], phase_deg: 0.0, star: true }), 0.0, 900),
    on("taunt", Slot::Head, hoop(2.0, 7.0, 200, 700, CRIMSON), 0.0, 1100),
    on("burn", Slot::Body, drift(7, -90.0, 70.0, 25.0, -20.0, 500, [AMBER, GOLD]), 4.0, 600),
    on("poison", Slot::Body, drift(5, -90.0, 50.0, 12.0, -15.0, 800, [VIOLET, ORCHID]), 0.0, 1000),
    on("bleed", Slot::Body, drift(4, 90.0, 20.0, 15.0, 90.0, 600, [ARTERIAL, CRIMSON]), 0.0, 900),
    on("expose", Slot::Body, hoop(9.0, 4.0, 255, 900, CRIMSON), 0.0, 1300),
    on("riposte", Slot::Body, Prim::Trail(Trail { at: O, radius: 9.0, from_deg: -40.0, to_deg: 40.0, ms: 300, width: 1, col: STEEL, mirror: false, squash: 1.0 }), 0.0, 1500),
    on("root", Slot::Feet, Prim::Orbit(Orbit { at: O, rx: 8.0, ry: 3.0, ms: 4000, count: 8, size: 1, cols: [IRON, STEEL], phase_deg: 0.0, star: false }), 0.0, 4000),
    on("slow", Slot::Feet, hoop(4.0, 9.0, 255, 1400, ASH), 0.0, 1400),
    on("steadfast", Slot::Feet, Prim::Flash(Flash { at: O, rx: 9.0, ry: 3.0, ms: 1200, col: alpha(GOLD, 160) }), 0.0, 1200),
    on("haste", Slot::Feet, drift(5, -90.0, 30.0, 70.0, 0.0, 220, [CYAN, LINEN]), 0.0, 600),
    on("regen", Slot::Feet, drift(4, -90.0, 40.0, 18.0, -10.0, 900, [GOLD, LINEN]), 0.0, 1100),
    tint("weaken", ASH, 40, 70, 1600),
    tint("empower", AMBER, 30, 70, 900),
    tint("fortify", CYAN, 25, 50, 1600),
    tint("guard", STEEL, 25, 50, 1600),
    StatusLook { id: "stealth", slot: Slot::Alpha, shape: Shape::Fade },
];

/// The look of status `id` (None: no row in the table, as `stagger`).
pub fn status_look(id: &str) -> Option<&'static StatusLook> {
    STATUS_LOOKS.iter().find(|l| l.id == id)
}

/// A status a unit shows: its id and whether it is only pending (it acts next round, S3).
pub type Mark = (&'static str, bool);

/// S1: a unit's statuses with a look, active or pending, in table order (pending only: `true`).
pub fn marks_of(st: &emberwake_rules::unit::StatusState) -> Vec<Mark> {
    STATUS_LOOKS
        .iter()
        .filter_map(|l| {
            if st.active.contains(l.id) {
                Some((l.id, false))
            } else {
                st.pending.contains(l.id).then_some((l.id, true))
            }
        })
        .collect()
}

/// The head anchor's height above the foot point in token sizes (Stage) and above the Flat token's centre (user
/// 2026-10-29: just above the helmet of the 40 px bodies; the spec's 0.75 and 0.45 sat at the shoulders).
pub const HEAD_UP_STAGE: f32 = 0.95;
pub const HEAD_UP_FLAT: f32 = 0.5;

/// Section 3's anchors from a token's drawn foot point (Flat: the token's centre): the head raised `HEAD_UP_STAGE` x
/// `token_px` (Flat: `HEAD_UP_FLAT` x its size), the body at STAGE1's chest (Flat: the centre), the feet at the foot point (Flat: the centre
/// lowered 0.4 x its size).
pub fn slot_anchor(geo: &Geom, foot: Pt, slot: Slot) -> Pt {
    let tp = geo.token_px as f32;
    let up = match (geo.view, slot) {
        (View::Stage, Slot::Head) => HEAD_UP_STAGE * tp,
        (View::Stage, Slot::Body) => (geo.token_px / 3) as f32,
        (View::Flat, Slot::Head) => HEAD_UP_FLAT * tp,
        (View::Flat, Slot::Feet) => -0.4 * tp,
        _ => 0.0,
    };
    (foot.0, foot.1 - up)
}

/// Section 3: the px scale of the looks (Stage 1; Flat `flat_token_px / token_px`) and the squash of rings and orbits
/// (Stage 0.75, Flat 1).
pub fn status_scale(geo: &Geom, t: &Tuning) -> (f32, f32) {
    match geo.view {
        View::Stage => (1.0, 0.75),
        View::Flat => (t.flat_token_px as f32 / t.token_px.max(1) as f32, 1.0),
    }
}

/// `p` placed at `at`: lengths times `k` (sizes and widths at least 1 px), rings and orbits squashed, a trail mirrored
/// when `left`, every colour's alpha times `fade` (S3).
fn placed(p: Prim, at: Pt, k: f32, squash: f32, left: bool, fade: f32) -> Prim {
    let n = |v: u8| ((v as f32 * k).round() as u8).max(1);
    let a = |c: Rgba| alpha(c, (c[3] as f32 * fade).round() as u8);
    match p {
        Prim::Sparks(s) => Prim::Sparks(Sparks { at, speed: s.speed * k, gravity: s.gravity * k, size: n(s.size), cols: [a(s.cols[0]), a(s.cols[1])], ..s }),
        Prim::Ring(r) => Prim::Ring(Ring { at, r0: r.r0 * k, r1: r.r1 * k, col: a(r.col), squash, ..r }),
        Prim::Orbit(o) => Prim::Orbit(Orbit { at, rx: o.rx * k, ry: o.ry * k * squash, size: n(o.size), cols: [a(o.cols[0]), a(o.cols[1])], ..o }),
        Prim::Flash(f) => Prim::Flash(Flash { at, rx: f.rx * k, ry: f.ry * k, col: a(f.col), ..f }),
        Prim::Trail(t) => Prim::Trail(Trail { at, radius: t.radius * k, width: n(t.width), col: a(t.col), mirror: left, ..t }),
        other => other,
    }
}

/// The seed of unit `unit`'s loop of status `id` (sparks vary per cycle from it, `Looped`).
pub fn status_seed(unit: usize, id: &str) -> u64 {
    let k = STATUS_LOOKS.iter().position(|l| l.id == id).unwrap_or(0) as u64;
    (unit as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (k << 8) ^ 0x57A7
}

/// The loop of a `Shape::Loop` look on a token whose drawn foot point is `foot` (S5: it follows the slide), facing left
/// when `left`, from clock `start`: feet looks on the ground layer, head and body looks in the air; a pending status at
/// `status_pending_alpha` times its alphas (S3). None for tints and stealth.
#[allow(clippy::too_many_arguments)]
pub fn status_loop(look: &StatusLook, geo: &Geom, t: &Tuning, foot: Pt, left: bool, start: u64, seed: u64, pending: bool) -> Option<Looped> {
    let Shape::Loop { prim, dy, period } = look.shape else { return None };
    let (k, squash) = status_scale(geo, t);
    let (x, y) = slot_anchor(geo, foot, look.slot);
    let fade = if pending { t.status_pending_alpha } else { 1.0 };
    let layer = if look.slot == Slot::Feet { Layer::Ground } else { Layer::Air };
    Some(Looped::new(Effect::new(placed(prim, (x, y + dy * k), k, squash, left, fade), start).on(layer).seeded(seed), period))
}

/// S7: a tint's alpha `t_ms` after its start: `lo` to `hi` and back (a sine over its period; under reduce motion the
/// middle one, still), at most `status_tint_max`, times `status_pending_alpha` while pending. 0 for other looks.
pub fn tint_alpha(look: &StatusLook, t_ms: u64, still: bool, pending: bool, t: &Tuning) -> u8 {
    let Shape::Tint { lo, hi, period, .. } = look.shape else { return 0 };
    let w = if still { 0.5 } else { (1.0 - (std::f32::consts::TAU * (t_ms % period.max(1)) as f32 / period.max(1) as f32).cos()) / 2.0 };
    let a = (lo as f32 + (hi as f32 - lo as f32) * w).min(t.status_tint_max as f32);
    let fade = if pending { t.status_pending_alpha } else { 1.0 };
    (a * fade).round() as u8
}

/// The token's alpha: `STEALTH_ALPHA` while stealthed (pending: that much of the way, S3), else 255.
pub fn token_alpha(drawn: &[Drawn], t: &Tuning) -> u8 {
    match drawn.iter().find(|d| d.look.shape == Shape::Fade) {
        Some(d) => {
            let k = if d.pending { t.status_pending_alpha } else { 1.0 };
            (255.0 - (255 - STEALTH_ALPHA) as f32 * k).round() as u8
        }
        None => 255,
    }
}

/// The tint a token draws at clock `now` (its colour at full alpha and the alpha to draw it at), if any.
pub fn tint_of(drawn: &[Drawn], now: u64, still: bool, t: &Tuning) -> Option<(Rgba, u8)> {
    drawn.iter().find_map(|d| match d.look.shape {
        Shape::Tint { col, .. } => Some((col, tint_alpha(d.look, now.saturating_sub(d.start), still, d.pending, t))),
        _ => None,
    })
}

/// S2: which of a slot's `n` statuses shows `elapsed` wall ms after its list was set: each for `cycle_ms`, in table
/// order, round and round.
pub fn cycle_pick(n: usize, elapsed: u64, cycle_ms: u64) -> usize {
    if n == 0 {
        0
    } else {
        (elapsed / cycle_ms.max(1)) as usize % n
    }
}

/// S2, one slot of a token: the ids it cycles, the wall ms that list was set, the id it draws and the clock ms its loop
/// restarted (0: since the list changed, the status's own start).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SlotShow {
    pub ids: Vec<&'static str>,
    pub since_wall: u64,
    pub drawn: Option<&'static str>,
    pub from: u64,
}

impl SlotShow {
    /// The id this slot draws among `ids` (table order) at wall ms `wall` and clock ms `now`, and the clock ms its loop
    /// restarted: a new list starts the cycle again; a switch by the cycle restarts the loop (SQ4).
    pub fn pick(&mut self, ids: &[&'static str], wall: u64, now: u64, cycle_ms: u64) -> Option<(&'static str, u64)> {
        if self.ids != ids {
            self.ids = ids.to_vec();
            self.since_wall = wall;
            if self.drawn != ids.first().copied() {
                self.drawn = ids.first().copied();
                self.from = 0;
            }
        }
        let id = *ids.get(cycle_pick(ids.len(), wall.saturating_sub(self.since_wall), cycle_ms))?;
        if self.drawn != Some(id) {
            self.drawn = Some(id);
            self.from = now;
        }
        Some((id, self.from))
    }
}

/// A status a token shows (`UnitAnim::marks`): its id, pending only, and the clock ms its look starts (S5: its burst's).
pub type MarkOn = (&'static str, bool, u64);

/// One look a token draws this frame: the row, pending only, the clock ms its loop starts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drawn {
    pub look: &'static StatusLook,
    pub pending: bool,
    pub start: u64,
}

/// S1-S2: the looks a token showing `marks` (table order) draws at wall ms `wall` and clock ms `now`, in slot order
/// head, body, feet, tint, then stealth's alpha (S9's order): one per slot, cycling every `cycle_ms`.
pub fn status_frame(marks: &[MarkOn], slots: &mut [SlotShow; 4], wall: u64, now: u64, cycle_ms: u64) -> Vec<Drawn> {
    let mut out = Vec::new();
    for (slot, show) in SLOTS.iter().zip(slots.iter_mut()) {
        let ids: Vec<&'static str> = marks.iter().filter(|m| status_look(m.0).is_some_and(|l| l.slot == *slot)).map(|m| m.0).collect();
        if let Some((id, from)) = show.pick(&ids, wall, now, cycle_ms) {
            if let (Some(m), Some(look)) = (marks.iter().find(|m| m.0 == id), status_look(id)) {
                out.push(Drawn { look, pending: m.1, start: m.2.max(from) });
            }
        }
    }
    if let Some(m) = marks.iter().find(|m| status_look(m.0).is_some_and(|l| l.slot == Slot::Alpha)) {
        out.extend(status_look(m.0).map(|look| Drawn { look, pending: m.1, start: m.2 }));
    }
    out
}

/// A look's first colour (the burst's; S4). None for stealth.
pub fn first_col(look: &StatusLook) -> Option<Rgba> {
    match look.shape {
        Shape::Loop { prim, .. } => match prim {
            Prim::Sparks(s) => Some(s.cols[0]),
            Prim::Orbit(o) => Some(o.cols[0]),
            Prim::Ring(r) => Some(r.col),
            Prim::Flash(f) => Some(f.col),
            Prim::Trail(t) => Some(t.col),
            Prim::Beam(b) => Some(b.core),
            Prim::Projectile(_) => None,
        },
        Shape::Tint { col, .. } => Some(col),
        Shape::Fade => None,
    }
}

/// S4: the apply burst of `look` on a token at `foot`, at clock `at`: a Ring r0 3 to r1 10 at alpha 200 for 220 ms at
/// the chest in its first colour, in the air (scaled and squashed as the looks). None for stealth (no colour).
pub fn apply_burst(look: &StatusLook, geo: &Geom, t: &Tuning, foot: Pt, at: u64) -> Option<Effect> {
    let col = first_col(look)?;
    let (k, squash) = status_scale(geo, t);
    let ring = Ring { at: slot_anchor(geo, foot, Slot::Body), r0: 3.0 * k, r1: 10.0 * k, ms: BURST_MS, col: alpha(col, 200), squash };
    Some(Effect::new(ring, at))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_ui::tuning::Tuned;
    use serde_json::json;

    fn act<'a>(kind: &'a str, skill: &'a str, family: &'a str, ranged: bool, dist: i32) -> Act<'a> {
        Act { kind, skill, family, ranged, magic: false, dist, on_self: false, harms: false }
    }

    fn delivery(a: Act) -> Delivery {
        style(a).delivery
    }

    /// L1 (4.3, 4.4.1-4.4.2): a sword strike on an adjacent foe swings the sword trail; a bow shot 4 cells away flies an
    /// arrow; a wand strike 3 cells away the arcane bolt; a brawler with no weapon strikes the fist ring (no trail); a
    /// mace at 3 cells with no skill row the generic reach bolt; an adjacent staff the mace row; `shield_bash` its row.
    #[test]
    fn an_action_gets_the_look_of_its_kind_weapon_and_range() {
        use aegis_ui::fx::Look;
        let sword = style(act("attack", "strike", "sword", false, 1));
        assert_eq!(sword.swing, Some(Swing { radius: 13.0, from_deg: -60.0, to_deg: 50.0, width: 2, col: STEEL }));
        assert_eq!(sword.hit.sparks.map(|b| (b.count, b.cols)), Some((6, [STEEL, LINEN])));
        let bow = style(act("attack", "strike", "bow", true, 4));
        assert_eq!(bow.shot, Some(Look::Arrow { len: 7.0, shaft: LINEN, head: STEEL }));
        assert!(bow.swing.is_none());
        assert_eq!(style(act("attack", "strike", "crossbow", true, 4)).shot, Some(Look::Bolt { len: 5.0, shaft: STEEL, head: LINEN }));
        assert_eq!(style(act("attack", "strike", "sling", true, 4)).shot, Some(Look::Stone { size: 2.0, col: ASH, hi: LINEN }));
        assert_eq!(style(act("attack", "strike", "wand", false, 3)), ARCANE_BOLT);
        assert_eq!(style(act("attack", "arcane_bolt", "sword", false, 1)), ARCANE_BOLT, "the skill row first");
        let fist = style(act("attack", "strike", "", false, 1));
        assert_eq!((fist.delivery, fist.swing, fist.hit.ring.map(|r| (r.r0, r.r1))), (Delivery::Melee, None, Some((2.0, 7.0))));
        assert_eq!(style(act("attack", "strike", "unarmed", false, 1)), fist);
        assert_eq!(style(act("attack", "strike", "mace", false, 3)), REACH_BOLT);
        assert_eq!(style(act("attack", "strike", "staff", false, 1)), melee_style("mace"), "an adjacent staff is a blunt blow");
        assert_eq!(style(act("attack", "shield_bash", "sword", false, 1)), SHIELD_BASH);
        assert_eq!(style(act("support", "mend", "", false, 1)).delivery, Delivery::Cast);
        assert_eq!(style(act("", "guard", "sword", false, 0)), NONE);
    }

    fn sparks_of(es: &[Effect]) -> Vec<Sparks> {
        es.iter().filter_map(|e| if let aegis_ui::fx::Prim::Sparks(s) = e.prim { Some(s) } else { None }).collect()
    }

    /// L8 (4.3): a crit (an `[Attack]` line with " CRIT") doubles the sparks and makes the second colour the crit hue; a
    /// hit for 0 halves them in Steel; a miss has none; the ring stays.
    #[test]
    fn a_crit_doubles_the_sparks_in_the_crit_hue_and_a_hit_for_0_halves_them() {
        let t = tuning();
        let lines = |l: &[&str]| l.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let crit = Mods::of(&lines(&["[Attack] Rook struck Gob for 9 CRIT (HP 1/10)"]), true);
        let zero = Mods::of(&lines(&["[Attack] Rook struck Gob for 0 (HP 10/10)"]), false);
        let miss = Mods::of(&lines(&["[Attack Miss] Rook: target no longer in combat"]), false);
        let plain = Mods::of(&lines(&["[Attack] Rook struck Gob for 4 (HP 6/10)"]), true);
        assert_eq!((crit.crit, zero.zero, miss.miss, plain), (true, true, true, Mods::default()));
        let mace = melee_style("mace");
        let base = mace.hit.sparks.unwrap();
        let pl = Place { from: (100.0, 100.0), to: (132.0, 100.0), squash: 1.0, vertical: false };
        let spot = |p: Pt| Spot { chest: p, foot: (p.0, p.1 + 10.0) };
        let at = |m: Mods| impact_effects(&mace, m, pl, &[spot(pl.to)], 500, 7, &t);
        let s = sparks_of(&at(plain));
        assert_eq!((s.len(), s[0].count, s[0].cols, s[0].ms), (1, base.count, base.cols, t.spark_ms as u32));
        let c = sparks_of(&at(crit));
        assert_eq!((c[0].count, c[0].cols), (base.count * 2, [base.cols[0], CRIT]));
        let z = sparks_of(&at(zero));
        assert_eq!((z[0].count, z[0].cols), (base.count / 2, [STEEL, STEEL]));
        assert!(sparks_of(&at(miss)).is_empty(), "a miss has no sparks");
        assert_eq!(at(miss).len(), 1, "the ring stays");
        assert_eq!(s[0].dir_deg, 0.0, "sparks fly along the attack (to the right)");
        assert_eq!(impact_effects(&mace, plain, pl, &[spot(pl.to), spot((164.0, 100.0))], 500, 7, &t).len(), 4, "each struck unit gets the row");
    }

    /// 4.2 step 2 with slice 2's effects: a swing's trail ends its sweep (60 % of `trail_ms`) at the impact, never
    /// starting before `t0`; a projectile leaves 6 px out from the actor and lands on the target at the impact.
    #[test]
    fn the_delivery_reaches_the_target_at_the_impact() {
        let t = tuning();
        let pl = Place { from: (100.0, 100.0), to: (60.0, 100.0), squash: 0.75, vertical: false };
        let none = Mods::default();
        let sword = melee_style("sword");
        let e = delivery_effects(&sword, pl, &[], none, 1000, 1000 + 270, 3, &t);
        let aegis_ui::fx::Prim::Trail(tr) = e[0].prim else { panic!("a trail") };
        assert_eq!((e.len(), e[0].start, tr.ms as u64, tr.mirror, tr.squash), (1, 1270 - t.trail_ms * 6 / 10, t.trail_ms, true, 0.75), "a target on the left mirrors it");
        assert_eq!(delivery_effects(&sword, pl, &[], none, 1000, 1000 + 20, 3, &t)[0].start, 1000, "never before t0");
        let bow = shot_style("bow");
        let e = delivery_effects(&bow, pl, &[], none, 1000, 1500, 3, &t);
        let aegis_ui::fx::Prim::Projectile(p) = e[0].prim else { panic!("a projectile") };
        assert_eq!((p.from, p.to, e[0].end()), ((94.0, 100.0), pl.to, 1500));
        assert_eq!(p.travel_ms as u64, travel_ms(34.0, 560.0, &t));
        assert!(delivery_effects(&melee_style(""), pl, &[], none, 1000, 1100, 3, &t).is_empty(), "a fist has no trail");
    }

    /// L9's rules (4.5): a skill of 3 or more blocks that lowered HP shakes `heavy`, a 2-block one does not, nor a
    /// 3-block miss; a break does; of several phase presets the strongest wins; an unknown name starts none.
    #[test]
    fn heavy_hits_and_breaks_shake_and_phases_take_the_strongest() {
        assert!(heavy_shake(3, true, false) && !heavy_shake(2, true, false) && !heavy_shake(3, false, false));
        assert!(heavy_shake(1, false, true), "a break");
        assert!(broke(&["[Break] Gob is broken".to_string()]) && !broke(&["[Attack] x".to_string()]));
        let table = ShakeTable::default();
        assert_eq!(strongest(["toll", "tremor"], &table), table.get("tremor"), "the same px, the longer one");
        assert_eq!(strongest(["quake", "jolt"], &table), table.get("quake"));
        assert_eq!(strongest(["boom"], &table), None);
    }

    /// L1 (the delivery part of 4.3): a sword adjacent swings; a bow at 4 cells shoots; a wand at 3 cells flies the
    /// arcane bolt; a brawler without a weapon strikes; a mace at 3 cells with no skill row flies the reach bolt.
    #[test]
    fn an_action_gets_the_delivery_of_its_kind_weapon_and_range() {
        assert_eq!(delivery(act("attack", "strike", "sword", false, 1)), Delivery::Melee);
        assert_eq!(delivery(act("attack", "strike", "bow", true, 4)), Delivery::Shot { speed: 560.0, arc: 6.0 });
        assert_eq!(delivery(act("attack", "strike", "crossbow", true, 4)), Delivery::Shot { speed: 720.0, arc: 2.0 });
        assert_eq!(delivery(act("attack", "strike", "sling", true, 2)), Delivery::Shot { speed: 480.0, arc: 8.0 });
        assert_eq!(delivery(act("attack", "strike", "wand", false, 3)), Delivery::Shot { speed: 420.0, arc: 0.0 }, "the arcane bolt");
        assert_eq!(delivery(Act { magic: true, ..act("area", "x", "sword", false, 2) }), Delivery::Shot { speed: 420.0, arc: 0.0 }, "magic damage");
        assert_eq!(delivery(act("attack", "strike", "", false, 1)), Delivery::Melee, "a brawler without a weapon");
        assert_eq!(delivery(act("attack", "strike", "mace", false, 3)), Delivery::Shot { speed: 480.0, arc: 0.0 }, "the reach bolt");
        assert_eq!(delivery(act("attack", "arcane_bolt", "", false, 1)), Delivery::Shot { speed: 420.0, arc: 0.0 }, "the skill row first");
        assert_eq!(delivery(act("attack", "shield_bash", "sword", false, 1)), Delivery::Melee);
        assert_eq!(delivery(act("support", "mend", "mace", false, 2)), Delivery::Cast);
        assert_eq!(delivery(Act { on_self: true, ..act("item", "", "", false, 0) }), Delivery::Cast);
        assert_eq!(delivery(act("item", "", "", false, 2)), Delivery::Shot { speed: 420.0, arc: 10.0 }, "FX1-L2: a flask thrown to another unit");
        assert_eq!(delivery(act("", "guard", "sword", false, 0)), Delivery::None);
    }

    fn tuning() -> Tuning {
        Tuning::from_json(&json!({}))
    }

    /// L2: a blow lands at `t0 + melee_hit_ms` (no clip impact) or at the clip's impact frame; an arrow over 160 px at
    /// 560 px/s flies 286 ms, clamped to `travel_max_ms`, and lands at `t0 + launch_ms + 260`; a 20 px shot lands at
    /// `t0 + launch_ms + travel_min_ms`.
    #[test]
    fn the_impact_comes_at_the_clip_frame_the_flight_or_the_tuned_times() {
        let t = tuning();
        let t0 = 1000;
        assert_eq!(impact_at(Delivery::Melee, t0, None, 0.0, &t), Some(t0 + t.melee_hit_ms));
        assert_eq!(impact_at(Delivery::Melee, t0, Some(270), 0.0, &t), Some(t0 + 270), "the clip's impact frame wins");
        let bow = Delivery::Shot { speed: 560.0, arc: 6.0 };
        assert_eq!((160.0f32 / 560.0 * 1000.0).round() as u64, 286);
        assert_eq!(travel_ms(160.0, 560.0, &t), t.travel_max_ms);
        assert_eq!(impact_at(bow, t0, None, 160.0, &t), Some(t0 + t.launch_ms + t.travel_max_ms));
        assert_eq!((t.launch_ms, t.travel_max_ms, t.travel_min_ms), (70, 260, 60), "the spec's defaults (L2: t0 + 330, t0 + 70 + 60)");
        assert_eq!(impact_at(bow, t0, None, 20.0, &t), Some(t0 + t.launch_ms + t.travel_min_ms));
        assert_eq!(impact_at(bow, t0, Some(560), 20.0, &t), Some(t0 + 560 + t.travel_min_ms), "the shot leaves at the clip's release");
        let mut off = tuning();
        off.shot_from_clip = false;
        assert_eq!(impact_at(bow, t0, Some(560), 20.0, &off), Some(t0 + off.launch_ms + off.travel_min_ms), "shot_from_clip off: launch_ms");
        assert_eq!(impact_at(Delivery::Cast, t0, Some(999), 50.0, &t), Some(t0 + t.launch_ms));
        assert_eq!(impact_at(Delivery::None, t0, None, 50.0, &t), None);
    }

    /// L10: the struck token sits +px and -px on alternate 30 ms windows during `hit_flash_ms`, 0 after it and never
    /// under reduce motion.
    #[test]
    fn the_struck_token_shakes_both_ways_then_rests() {
        let flash = 180;
        let xs: Vec<i32> = (0..flash).step_by(30).map(|t| token_shake(1, t, flash, false)).collect();
        assert_eq!(xs, [1, -1, 1, -1, 1, -1]);
        assert_eq!((token_shake(1, 29, flash, false), token_shake(1, 30, flash, false)), (1, -1), "the switch at 30 ms");
        assert_eq!(token_shake(1, flash, flash, false), 0);
        assert!((0..flash).all(|t| token_shake(1, t, flash, true) == 0), "reduce motion");
    }

    /// L17 (FX1 5.1, from FX1-S): the kit check's own copy of the shake preset names equals the engine's.
    #[test]
    fn the_kit_shake_presets_are_the_engines() {
        assert_eq!(emberwake_data::SHAKE_PRESETS, aegis_ui::fx::SHAKE_PRESETS);
    }

    /// L13: both views' anchors: the Stage's chest point (STAGE1 3), the Flat token's centre raised 2 px; a shot's path
    /// starts 6 px out from the actor.
    #[test]
    fn effects_anchor_at_the_chest_on_the_stage_and_the_raised_centre_flat() {
        let t = tuning();
        let c = V2::new(3, 4);
        let stage = Geom::new(View::Stage, &t);
        let (fx, fy) = stage.foot(c);
        assert_eq!(anchor(&stage, c), (fx as f32, (fy - t.token_px / 3) as f32));
        let flat = Geom::new(View::Flat, &t);
        let r = flat.token_rect(c, 0, 0);
        assert_eq!(anchor(&flat, c), ((r.x + r.w / 2) as f32, (r.y + r.h / 2) as f32 - 2.0));
        let (a, b) = (anchor(&flat, c), anchor(&flat, V2::new(7, 4)));
        assert_eq!(flight_px(&shot_style("bow"), Place { from: a, to: b, squash: 1.0, vertical: false }), (b.0 - a.0) - 6.0);
    }

    // ---- FX1-L2 (M1-M7 and the FX1-L1 follow-ups) -----------------------------------------------------------------

    use aegis_ui::fx::Prim;

    const NO_MODS: Mods = Mods { crit: false, zero: false, miss: false };

    fn kinds(es: &[Effect]) -> Vec<&'static str> {
        es.iter().map(|e| e.kind()).collect()
    }

    fn spot(p: Pt) -> Spot {
        Spot { chest: p, foot: (p.0, p.1 + 13.0) }
    }

    fn place(from: Pt, to: Pt) -> Place {
        Place { from, to, squash: 0.75, vertical: false }
    }

    fn skill(s: &str) -> Style {
        style(act("attack", s, "staff", false, 3))
    }

    fn projectiles(es: &[Effect]) -> Vec<(Projectile, u64)> {
        es.iter().filter_map(|e| if let Prim::Projectile(p) = e.prim { Some((p, e.start)) } else { None }).collect()
    }

    /// M1 `hex` (4.4.3): an Orchid orb on a 4 px arc at 360 px/s; an Orchid ring at the impact.
    #[test]
    fn hex_flies_an_orchid_orb_and_rings_the_target() {
        let (t, pl) = (tuning(), place((100.0, 100.0), (196.0, 100.0)));
        let s = skill("hex");
        assert_eq!((s.delivery, s.shot), (Delivery::Shot { speed: 360.0, arc: 4.0 }, Some(Look::Orb { r: 2.0, core: ORCHID, glow: ORCHID, tail: 2 })));
        let d = delivery_effects(&s, pl, &[], NO_MODS, 0, 400, 1, &t);
        assert_eq!((kinds(&d), d[0].end()), (vec!["projectile"], 400));
        let i = impact_effects(&s, NO_MODS, pl, &[spot(pl.to)], 400, 1, &t);
        let Prim::Ring(r) = i[0].prim else { panic!("a ring") };
        assert_eq!((kinds(&i), r.r0, r.r1, r.col), (vec!["ring"], 2.0, 8.0, alpha(ORCHID, 220)));
    }

    /// M1 `arc_lance` (LINE, 4.4.3): a beam from the actor to the line's farthest cell at the impact (as a shot's
    /// release), 4 Orchid sparks on each struck unit.
    #[test]
    fn arc_lance_beams_to_the_last_line_cell_and_sparks_each_struck_unit() {
        let t = tuning();
        let s = skill("arc_lance");
        let line = [(132.0, 100.0), (164.0, 100.0), (196.0, 100.0)];
        let pl = place((100.0, 100.0), line[0]);
        assert_eq!(impact_at(s.delivery, 1000, Some(250), 0.0, &t), Some(1250), "it lands at the clip's release");
        assert_eq!(impact_at(s.delivery, 1000, None, 0.0, &t), Some(1000 + t.launch_ms));
        let d = delivery_effects(&s, pl, &line, NO_MODS, 1000, 1250, 1, &t);
        let Prim::Beam(b) = d[0].prim else { panic!("a beam") };
        assert_eq!((d.len(), b.to, b.from, b.width, b.ms, d[0].start), (1, line[2], (106.0, 100.0), 2, 160, 1250));
        let i = impact_effects(&s, NO_MODS, pl, &[spot(line[0]), spot(line[2])], 1250, 1, &t);
        let counts: Vec<u8> = i.iter().filter_map(|e| if let Prim::Sparks(s) = e.prim { Some(s.count) } else { None }).collect();
        assert_eq!((kinds(&i), counts), (vec!["sparks", "sparks"], vec![4, 4]));
    }

    /// M2 (4.3, 4.4.3): `firebomb` on a BURST that struck 3 units: one orb lobbed to the aim cell, one ring on the aim
    /// (radius 1: 4 to 32 x 0.6 px) and its flash, sparks on the 3 units, all at the orb's impact; `inferno` is the same.
    #[test]
    fn firebomb_lobs_one_orb_to_the_aim_and_bursts_on_each_struck_unit() {
        let t = tuning();
        let s = skill("firebomb");
        assert_eq!(skill("inferno"), s);
        let aim = (196.0, 113.0);
        let pl = place((100.0, 100.0), aim);
        let burst: Vec<Pt> = (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (aim.0 + 32.0 * dx as f32, aim.1 + 24.0 * dy as f32))).collect();
        let at = impact_at(s.delivery, 0, None, flight_px(&s, pl), &t).unwrap() + spread_ms(&s, &burst, aim);
        let d = delivery_effects(&s, pl, &burst, NO_MODS, 0, at, 1, &t);
        let p = projectiles(&d);
        assert_eq!((d.len(), p[0].0.to, p[0].0.arc_px, d[0].end()), (1, aim, 16.0, at), "one orb to the aim, landing at the impact");
        let a = aim_effects(&s, true, aim, 1, false, pl.squash, at, 1);
        let Prim::Ring(r) = a[0].prim else { panic!("the aim ring") };
        assert_eq!((kinds(&a), r.at, r.r0, r.r1, a[0].layer), (vec!["ring", "flash"], aim, 4.0, AIM_RING_PX_PER_CELL, Layer::Ground));
        let Prim::Ring(r2) = aim_effects(&s, true, aim, 2, false, pl.squash, at, 1)[0].prim else { panic!() };
        assert_eq!(r2.r1, 2.0 * AIM_RING_PX_PER_CELL, "the ring grows with the area's radius");
        let units = [spot(burst[0]), spot(burst[4]), spot(burst[8])];
        let i = impact_effects(&s, NO_MODS, pl, &units, at, 1, &t);
        assert_eq!(kinds(&i), vec!["sparks"; 3], "sparks on the 3 struck units and no ring on them");
        assert!(a.iter().chain(&i).all(|e| e.start == at), "all at the orb's impact");
    }

    /// M1 `glacial_burst` (4.4.3): the firebomb's lob in Cyan; a Cyan aim ring without a flash; 8 sparks, gravity 80.
    #[test]
    fn glacial_burst_is_the_firebomb_in_cyan() {
        let s = skill("glacial_burst");
        assert_eq!((s.delivery, s.shot), (FIREBOMB.delivery, Some(Look::Orb { r: 3.0, core: LINEN, glow: CYAN, tail: 3 })));
        let a = aim_effects(&s, true, (0.0, 0.0), 1, false, 0.75, 10, 1);
        let Prim::Ring(r) = a[0].prim else { panic!() };
        assert_eq!((kinds(&a), r.col), (vec!["ring"], alpha(CYAN, 230)));
        let b = s.hit.sparks.unwrap();
        assert_eq!((b.count, b.cols, b.gravity), (8, [CYAN, LINEN], 80.0));
    }

    /// M1 `smite`, `judgement` (4.4.3): a gold orb falls 48 px onto the target's chest at 600 px/s (80 ms); a gold flash
    /// and sparks upward.
    #[test]
    fn smite_drops_a_gold_orb_onto_the_target() {
        let t = tuning();
        let s = skill("smite");
        assert_eq!(skill("judgement"), s);
        let pl = place((100.0, 100.0), (196.0, 140.0));
        assert_eq!(flight(&s, pl), ((196.0, 140.0 - SMITE_DROP_PX), (196.0, 140.0)));
        let travel = travel_ms(SMITE_DROP_PX, 600.0, &t);
        assert_eq!(travel, 80);
        let at = impact_at(s.delivery, 0, None, flight_px(&s, pl), &t).unwrap();
        assert_eq!(at, t.launch_ms + travel);
        let p = projectiles(&delivery_effects(&s, pl, &[], NO_MODS, 0, at, 1, &t));
        assert_eq!((p[0].0.from, p[0].0.to, p[0].1 + travel), ((196.0, 92.0), (196.0, 140.0), at));
        let i = impact_effects(&s, NO_MODS, pl, &[spot(pl.to)], at, 1, &t);
        let Prim::Sparks(sp) = i[1].prim else { panic!() };
        assert_eq!((kinds(&i), sp.dir_deg, sp.cols), (vec!["flash", "sparks"], -90.0, [GOLD, LINEN]));
    }

    /// M3 (4.4.3 `volley`, PLUS): one bow arrow per area cell, at most 5, launched 30 ms apart; the arrow to the aim
    /// leaves last and lands at the impact; bow sparks per struck unit.
    #[test]
    fn volley_looses_one_arrow_per_cell_30_ms_apart_at_most_5() {
        let t = tuning();
        let s = skill("volley");
        let aim = (260.0, 113.0);
        let pl = place((100.0, 100.0), aim);
        let plus = [aim, (228.0, 113.0), (292.0, 113.0), (260.0, 89.0), (260.0, 137.0)];
        assert_eq!(spread_ms(&s, &plus, aim), 4 * VOLLEY_GAP_MS);
        let at = impact_at(s.delivery, 0, None, flight_px(&s, pl), &t).unwrap() + spread_ms(&s, &plus, aim);
        let p = projectiles(&delivery_effects(&s, pl, &plus, NO_MODS, 0, at, 1, &t));
        assert_eq!(p.len(), 5);
        assert!(p.windows(2).all(|w| w[1].1 == w[0].1 + VOLLEY_GAP_MS), "30 ms apart: {:?}", p.iter().map(|x| x.1).collect::<Vec<_>>());
        assert!(p.iter().all(|x| x.0.look == Look::Arrow { len: 7.0, shaft: LINEN, head: STEEL }));
        let mut ends: Vec<Pt> = p.iter().map(|x| x.0.to).collect();
        assert_eq!(ends.pop(), Some(aim), "the aim's arrow last");
        assert!(plus[1..].iter().all(|c| ends.contains(c)), "one arrow per cell");
        assert_eq!(p[4].1 + p[4].0.travel_ms as u64, at, "it lands at the impact");
        let seven: Vec<Pt> = (0..7).map(|k| (200.0 + 32.0 * k as f32, 113.0)).collect();
        assert_eq!(volley_targets(&s, &seven, seven[3]).len(), 5, "at most 5");
        assert_eq!(s.hit.sparks, shot_style("bow").hit.sparks);
    }

    /// M1 `shockwave` (LINE, 4.4.3): Ash rings on each line cell on the ground layer, 40 ms apart from the actor out,
    /// the last at the impact; 5 Ash sparks per struck unit, gravity 160.
    #[test]
    fn shockwave_rings_each_line_cell_from_the_actor_out() {
        let t = tuning();
        let s = skill("shockwave");
        let line = [(196.0, 100.0), (132.0, 100.0), (164.0, 100.0)];
        let pl = place((100.0, 100.0), line[1]);
        let at = impact_at(s.delivery, 0, None, 0.0, &t).unwrap() + spread_ms(&s, &line, pl.to);
        assert_eq!(at, t.melee_hit_ms + 2 * WAVE_GAP_MS);
        let d = delivery_effects(&s, pl, &line, NO_MODS, 0, at, 1, &t);
        let rings: Vec<(Pt, u64, Layer)> = d.iter().filter_map(|e| if let Prim::Ring(r) = e.prim { Some((r.at, e.start, e.layer)) } else { None }).collect();
        let want = [(line[1], t.melee_hit_ms), (line[2], t.melee_hit_ms + WAVE_GAP_MS), (line[0], at)];
        assert_eq!(rings, want.map(|(p, s)| (p, s, Layer::Ground)).to_vec());
        let b = s.hit.sparks.unwrap();
        assert_eq!((b.count, b.cols, b.gravity), (5, [ASH, ASH], 160.0));
    }

    /// M1 `sweep_kick`, `cyclone_kick` (RING since W2-4) and `haymaker`, `crushing_blow` (4.4.3): the sweep's wide Linen trail
    /// at the actor with the fist's impact; the haymaker's short trail, ring and sparks; the dash rows take the family's.
    #[test]
    fn sweeps_haymakers_and_dashes_take_their_rows() {
        let sweep = skill("sweep_kick");
        assert_eq!(skill("cyclone_kick"), sweep);
        assert_eq!((sweep.delivery, sweep.swing, sweep.hit), (Delivery::Melee, Some(Swing { radius: 14.0, from_deg: -150.0, to_deg: 150.0, width: 2, col: LINEN }), melee_style("fist").hit));
        let hay = skill("haymaker");
        assert_eq!(skill("crushing_blow"), hay);
        assert_eq!(hay.swing, Some(Swing { radius: 10.0, from_deg: -40.0, to_deg: 60.0, width: 2, col: LINEN }));
        assert_eq!((hay.hit.ring.map(|r| (r.r0, r.r1, r.col)), hay.hit.sparks.map(|b| (b.count, b.cols))), (Some((2.0, 9.0, alpha(LINEN, 220))), Some((7, [LINEN, ASH]))));
        for dash in ["lunge", "pounce", "charge_in"] {
            assert_eq!(style(act("attack", dash, "dagger", false, 1)), melee_style("dagger"), "{dash}");
        }
    }

    /// M4 (4.4.4): `mend` (and any support without a row) rises gold motes from the target's foot point for 500 ms with a
    /// soft flash at its chest; `purify` in Cyan; `aegis` a Cyan ring at the chest; `rally` a ring at the actor's chest.
    #[test]
    fn support_shows_motes_rings_or_a_ring_on_the_actor() {
        let t = tuning();
        let mend = style(act("support", "mend", "", false, 2));
        assert_eq!(mend.delivery, Delivery::Cast);
        assert_eq!(style(act("support", "renewal", "", false, 2)), mend, "any other support");
        let target = spot((196.0, 100.0));
        let i = impact_effects(&mend, NO_MODS, place((100.0, 100.0), target.chest), &[target], 70, 1, &t);
        let Prim::Sparks(m) = i[1].prim else { panic!("motes") };
        assert_eq!((kinds(&i), m.at, m.ms, m.cols, m.dir_deg, m.gravity), (vec!["flash", "sparks"], target.foot, 500, [GOLD, LINEN], -90.0, -30.0));
        let Prim::Flash(f) = i[0].prim else { panic!() };
        assert_eq!(f.at, target.chest);
        assert_eq!(style(act("support", "purify", "", false, 1)).hit.sparks.map(|b| b.cols), Some([CYAN, LINEN]));
        let aegis = style(act("support", "aegis", "", false, 1));
        assert_eq!(aegis.hit.ring.map(|r| (r.r0, r.r1, r.col, r.ms)), Some((4.0, 12.0, alpha(CYAN, 220), 260)));
        let rally = style(act("support", "rally", "", false, 1));
        assert!(rally.on_actor && style(act("support", "taunt_shout", "", false, 1)).on_actor);
        assert_eq!(rally.hit.ring.map(|r| (r.r0, r.r1)), Some((4.0, 20.0)));
    }

    /// M5 (4.4.4): an item on another unit flies a flask (420 px/s, arc 10) before its case; on its user it does not; a
    /// damage consumable shows the firebomb's impact on its target, a heal the mend motes.
    #[test]
    fn an_item_on_another_unit_throws_a_flask_first() {
        let t = tuning();
        let other = style(act("item", "", "", false, 2));
        assert_eq!((other.delivery, other.shot), (Delivery::Shot { speed: 420.0, arc: 10.0 }, Some(FLASK)));
        assert_eq!(other.hit, MEND.hit, "a heal: the motes");
        let pl = place((100.0, 100.0), (164.0, 100.0));
        let d = delivery_effects(&other, pl, &[], NO_MODS, 0, 300, 1, &t);
        assert_eq!(projectiles(&d)[0].0.look, FLASK);
        let own = style(Act { on_self: true, ..act("item", "", "", false, 0) });
        assert_eq!((own.delivery, own.shot, own.hit), (Delivery::Cast, None, MEND.hit));
        assert!(delivery_effects(&own, pl, &[], NO_MODS, 0, 70, 1, &t).is_empty(), "on self: no flask");
        let bomb = style(Act { harms: true, ..act("item", "", "", false, 2) });
        assert_eq!((bomb.shot, bomb.hit, bomb.aim), (Some(FLASK), FIREBOMB.hit, FIREBOMB.aim), "a damage consumable");
    }

    /// M6 (4.4.4): a fall puts an Ash dust ring (3 to 10 px, 300 ms) and dust at the unit's foot point, both on the
    /// ground layer.
    #[test]
    fn a_fall_raises_dust_on_the_ground_layer() {
        let t = tuning();
        let e = fall_effects((50.0, 60.0), 0.75, 900, 1, &t);
        assert_eq!(kinds(&e), vec!["ring", "sparks"]);
        assert!(e.iter().all(|e| e.layer == Layer::Ground && e.start == 900));
        let Prim::Ring(r) = e[0].prim else { panic!() };
        assert_eq!((r.at, r.r0, r.r1, r.ms, r.col), ((50.0, 60.0), 3.0, 10.0, 300, alpha(ASH, 200)));
        let Prim::Sparks(s) = e[1].prim else { panic!() };
        assert_eq!((s.count, s.size, s.cols, s.gravity), (6, 2, [ASH, LINEN], 120.0));
    }

    /// M7 (4.3): a missed shot flies 8 px past its target at the same speed (it still passes the target at the impact);
    /// the target steps 3 px away over 80 ms and back over 140 ms, along the attack.
    #[test]
    fn a_miss_overshoots_and_the_target_steps_aside_and_back() {
        let t = tuning();
        let bow = shot_style("bow");
        let pl = place((100.0, 100.0), (200.0, 100.0));
        let miss = Mods { miss: true, ..NO_MODS };
        let hit = projectiles(&delivery_effects(&bow, pl, &[], NO_MODS, 0, 500, 1, &t))[0];
        let over = projectiles(&delivery_effects(&bow, pl, &[], miss, 0, 500, 1, &t))[0];
        assert_eq!((over.0.to, over.1), ((208.0, 100.0), hit.1), "8 px past, launched as the hit");
        assert!(over.0.travel_ms > hit.0.travel_ms, "the longer flight at the same speed");
        assert_eq!((sidestep_px(0), sidestep_px(SIDESTEP_OUT_MS), sidestep_px(SIDESTEP_OUT_MS + SIDESTEP_BACK_MS)), (0.0, SIDESTEP_PX, 0.0));
        assert!((0..400).all(|a| sidestep_px(a) <= SIDESTEP_PX));
        let steps = [(4, 1000, unit_dir(pl.from, pl.to))];
        assert_eq!(sidestep_of(&steps, 4, 1000 + SIDESTEP_OUT_MS), (3, 0), "away from the attacker");
        assert_eq!(sidestep_of(&steps, 4, 1000 + 400), (0, 0));
        assert_eq!(sidestep_of(&steps, 5, 1000 + SIDESTEP_OUT_MS), (0, 0), "only the target");
    }

    /// FX1-L1 follow-up (4.4.1, DIR4-G): a swing at a target straight below or above turns its trail toward it (by the
    /// attack's angle, unmirrored); a side swing keeps its table angles.
    #[test]
    fn a_swing_up_or_down_turns_toward_its_target() {
        let t = tuning();
        let sword = melee_style("sword");
        let trail = |pl: Place| match delivery_effects(&sword, pl, &[], NO_MODS, 0, 200, 1, &t)[0].prim {
            Prim::Trail(tr) => (tr.from_deg, tr.to_deg, tr.mirror),
            _ => panic!("a trail"),
        };
        assert_eq!(trail(place((100.0, 100.0), (132.0, 100.0))), (-60.0, 50.0, false));
        let down = Place { vertical: true, ..place((100.0, 100.0), (100.0, 124.0)) };
        assert_eq!(trail(down), (30.0, 140.0, false), "down: turned 90 deg");
        let up = Place { vertical: true, ..place((100.0, 100.0), (100.0, 76.0)) };
        assert_eq!(trail(up), (-150.0, -40.0, false), "up: turned -90 deg");
    }

    /// FX1-L1 follow-up: an area that struck 12 units keeps its impact within `IMPACT_BUDGET` (under the live list's
    /// cap), every unit's sparks and the aim's ring first, dropping flashes before rings.
    #[test]
    fn a_wide_area_hit_keeps_within_the_budget_sparks_first() {
        let t = tuning();
        let s = Style { hit: Hit { flash: bloom(4.0, 3.0, LINEN, 120, 100), ..melee_style("mace").hit }, ..NONE };
        let pl = place((100.0, 100.0), (200.0, 100.0));
        let units: Vec<Spot> = (0..12).map(|k| spot((150.0 + 10.0 * k as f32, 120.0))).collect();
        let mut all = aim_effects(&s, true, pl.to, 1, false, pl.squash, 500, 1);
        all.extend(impact_effects(&s, NO_MODS, pl, &units, 500, 1, &t));
        assert!(all.len() > MAX_LIVE, "{} effects", all.len());
        let kept = budget(all.clone(), IMPACT_BUDGET);
        let count = |es: &[Effect], k: &str| es.iter().filter(|e| e.kind() == k).count();
        assert_eq!(kept.len(), IMPACT_BUDGET);
        assert_eq!((count(&kept, "sparks"), count(&kept, "flash")), (12, 0), "every unit's sparks; flashes go first");
        assert_eq!(kept[0], all[0], "the aim ring stays, in its place");
        assert_eq!(budget(all[..5].to_vec(), IMPACT_BUDGET), all[..5].to_vec(), "under the budget: unchanged");
    }

    /// FX1-L1 follow-up (4.2 step 8): a break's or a phase's shake seed comes from the round, the block and its key,
    /// not the clock.
    #[test]
    fn shake_seeds_come_from_the_round_block_and_key() {
        assert_eq!(shake_seed(2, 3, BREAK_KEY), shake_seed(2, 3, BREAK_KEY));
        assert_ne!(shake_seed(2, 3, 0), shake_seed(2, 3, 1), "another phase");
        assert_ne!(shake_seed(2, 3, 0), shake_seed(2, 4, 0), "another block");
        assert_ne!(shake_seed(2, 3, 0), shake_seed(3, 3, 0), "another round");
    }

    // ---- STATUSFX1-L (spec section 4 acceptance) ---------------------------------------------------------------------

    fn marks(ids: &[&'static str]) -> Vec<MarkOn> {
        ids.iter().map(|&id| (id, false, 0)).collect()
    }

    fn ids(d: &[Drawn]) -> Vec<&'static str> {
        d.iter().map(|x| x.look.id).collect()
    }

    fn prim_at(p: Prim) -> Pt {
        match p {
            Prim::Sparks(s) => s.at,
            Prim::Ring(r) => r.at,
            Prim::Orbit(o) => o.at,
            Prim::Flash(f) => f.at,
            Prim::Trail(t) => t.at,
            Prim::Beam(b) => b.from,
            Prim::Projectile(p) => p.from,
        }
    }

    /// 1: every status row but `stagger` (WINDUP1's `dazed`) has a look, so a new row without one fails here; every look
    /// names a row and the table holds each id once.
    #[test]
    fn every_status_row_but_stagger_has_a_look() {
        let rows = &*emberwake_rules::status::ROWS;
        for r in rows.iter() {
            assert_eq!(status_look(r.id).is_some(), r.id != "stagger", "{}: a look unless it is stagger", r.id);
        }
        assert!(STATUS_LOOKS.iter().all(|l| rows.iter().any(|r| r.id == l.id)), "no look without a status row");
        assert_eq!(STATUS_LOOKS.len(), rows.len() - 1, "one look per row");
        assert_eq!(status_look("stealth").map(|l| (l.slot, l.shape)), Some((Slot::Alpha, Shape::Fade)));
    }

    /// 2 (S2): stun and burn draw both (head and body); burn, poison and bleed share the body: burn, poison at 1500 ms,
    /// bleed at 3000, burn again at 4500 (wall ms); a switch restarts that look's loop. Every slot draws at once, in the
    /// order head, body, feet, tint, then stealth's alpha (S9).
    #[test]
    fn a_slot_cycles_its_statuses_in_table_order_and_the_slots_draw_together() {
        let t = tuning();
        assert_eq!(t.status_cycle_ms, 1500);
        let mut slots: [SlotShow; 4] = Default::default();
        assert_eq!(ids(&status_frame(&marks(&["stun", "burn"]), &mut slots, 0, 0, t.status_cycle_ms)), ["stun", "burn"]);
        let mut slots: [SlotShow; 4] = Default::default();
        let m = marks(&["burn", "poison", "bleed"]);
        for (wall, want) in [(0, "burn"), (1499, "burn"), (1500, "poison"), (2999, "poison"), (3000, "bleed"), (4500, "burn")] {
            assert_eq!(ids(&status_frame(&m, &mut slots, wall, wall, t.status_cycle_ms)), [want], "at {wall} ms");
        }
        let d = status_frame(&m, &mut slots, 6000, 7777, t.status_cycle_ms);
        assert_eq!((d[0].look.id, d[0].start), ("poison", 7777), "the switch restarts the loop on the clock");
        assert_eq!(status_frame(&m, &mut slots, 6100, 7877, t.status_cycle_ms)[0].start, 7777, "and it runs on");
        let mut slots: [SlotShow; 4] = Default::default();
        let all = marks(&["stun", "bleed", "root", "fortify", "stealth"]);
        assert_eq!(ids(&status_frame(&all, &mut slots, 0, 0, t.status_cycle_ms)), ["stun", "bleed", "root", "fortify", "stealth"]);
        // S5: a look starts at its status's start (its burst), not before
        let mut slots: [SlotShow; 4] = Default::default();
        assert_eq!(status_frame(&[("burn", false, 900)], &mut slots, 0, 500, t.status_cycle_ms)[0].start, 900);
        assert_eq!((cycle_pick(0, 99, 1500), cycle_pick(3, 99_000, 1500)), (0, 66 % 3));
    }

    /// 3 (S3): a status only pending draws at `status_pending_alpha` times its alphas (its loop's colours, a tint's alpha,
    /// stealth half way); active, at full alpha.
    #[test]
    fn a_pending_status_draws_at_the_pending_alpha() {
        let t = tuning();
        let geo = Geom::new(View::Stage, &t);
        let poison = status_look("poison").unwrap();
        let cols = |pending: bool| match status_loop(poison, &geo, &t, (100.0, 100.0), false, 0, 1, pending).unwrap().effect.prim {
            Prim::Sparks(s) => s.cols,
            _ => panic!("poison drifts sparks"),
        };
        assert_eq!(cols(false), [VIOLET, ORCHID], "active: full alpha");
        let half = (255.0 * t.status_pending_alpha).round() as u8;
        assert_eq!(t.status_pending_alpha, 0.5);
        assert_eq!(cols(true).map(|c| c[3]), [half, half], "pending: half");
        let weaken = status_look("weaken").unwrap();
        let mid = tint_alpha(weaken, 0, true, false, &t);
        assert_eq!(tint_alpha(weaken, 0, true, true, &t), (mid as f32 * t.status_pending_alpha).round() as u8);
        let stealth = Drawn { look: status_look("stealth").unwrap(), pending: false, start: 0 };
        assert_eq!(token_alpha(&[stealth], &t), STEALTH_ALPHA);
        assert_eq!(STEALTH_ALPHA, (0.55f32 * 255.0).round() as u8, "55 %");
        assert_eq!(token_alpha(&[Drawn { pending: true, ..stealth }], &t), (255.0 - (255 - STEALTH_ALPHA) as f32 * 0.5).round() as u8);
        assert_eq!(token_alpha(&[], &t), 255);
    }

    /// 5 (S7): a tint pulses `lo` to `hi` and back over its period (capped at `status_tint_max`); under reduce motion it
    /// holds its middle alpha.
    #[test]
    fn tints_pulse_and_hold_their_middle_alpha_under_reduce_motion() {
        let t = tuning();
        for id in ["weaken", "empower", "fortify"] {
            let l = status_look(id).unwrap();
            let Shape::Tint { lo, hi, period, .. } = l.shape else { panic!("{id} is a tint") };
            assert_eq!((tint_alpha(l, 0, false, false, &t), tint_alpha(l, period / 2, false, false, &t), tint_alpha(l, period, false, false, &t)), (lo, hi, lo), "{id} pulses");
            let mid = ((lo as f32 + hi as f32) / 2.0).round() as u8;
            assert!((0..period).step_by(97).all(|ms| tint_alpha(l, ms, true, false, &t) == mid), "{id}: still at the middle alpha");
        }
        let capped = Tuning { status_tint_max: 45, ..tuning() };
        assert_eq!(tint_alpha(status_look("weaken").unwrap(), 800, false, false, &capped), 45, "the ceiling");
    }

    /// 7: the anchors (Stage: head 0.95 x `token_px` up, the chest, the foot; Flat: 0.5 x the token up, the centre, 0.4
    /// down) with burn lowered 4 px; feet looks on the ground layer, head and body looks in the air; Flat lengths scale by
    /// `flat_token_px / token_px` (at least 1 px) and rings and orbits squash 0.75 on the Stage only.
    #[test]
    fn looks_sit_at_their_slot_anchors_on_their_layers_in_both_views() {
        let t = tuning();
        let foot = (200.0, 150.0);
        let stage = Geom::new(View::Stage, &t);
        let flat = Geom::new(View::Flat, &t);
        let tp = t.token_px as f32;
        let ft = t.flat_token_px as f32;
        let c = V2::new(3, 4);
        let (cx_, cy_) = stage.chest(c);
        let (fx_, fy_) = stage.foot(c);
        assert_eq!(slot_anchor(&stage, (fx_ as f32, fy_ as f32), Slot::Body), (cx_ as f32, cy_ as f32), "the Stage chest is STAGE1's");
        assert_eq!((HEAD_UP_STAGE, HEAD_UP_FLAT), (0.95, 0.5), "just above the helmet (user 2026-10-29)");
        assert_eq!(slot_anchor(&stage, foot, Slot::Head), (200.0, 150.0 - 0.95 * tp));
        assert_eq!(slot_anchor(&stage, foot, Slot::Feet), foot);
        assert_eq!(slot_anchor(&flat, foot, Slot::Head), (200.0, 150.0 - 0.5 * ft));
        assert_eq!(slot_anchor(&flat, foot, Slot::Body), foot);
        assert_eq!(slot_anchor(&flat, foot, Slot::Feet), (200.0, 150.0 + 0.4 * ft));
        for geo in [stage, flat] {
            let (k, squash) = status_scale(&geo, &t);
            for l in STATUS_LOOKS.iter() {
                let Some(lp) = status_loop(l, &geo, &t, foot, false, 0, 1, false) else { continue };
                assert_eq!(lp.layer(), if l.slot == Slot::Feet { Layer::Ground } else { Layer::Air }, "{}", l.id);
                let dy = if l.id == "burn" { 4.0 * k } else { 0.0 };
                let a = slot_anchor(&geo, foot, l.slot);
                assert_eq!(prim_at(lp.effect.prim), (a.0, a.1 + dy), "{} at its anchor", l.id);
            }
            let stun = status_loop(status_look("stun").unwrap(), &geo, &t, foot, false, 0, 1, false).unwrap();
            let Prim::Orbit(o) = stun.effect.prim else { panic!("stun orbits") };
            assert_eq!((o.rx, o.ry, o.size), (7.0 * k, 2.5 * k * squash, ((2.0 * k).round() as u8).max(1)));
            let slow = status_loop(status_look("slow").unwrap(), &geo, &t, foot, false, 0, 1, false).unwrap();
            assert!(matches!(slow.effect.prim, Prim::Ring(r) if r.squash == squash && r.r1 == 9.0 * k));
        }
        assert_eq!((status_scale(&stage, &t), status_scale(&flat, &t)), ((1.0, 0.75), (ft / tp, 1.0)));
        let riposte = |left: bool| match status_loop(status_look("riposte").unwrap(), &stage, &t, foot, left, 0, 1, false).unwrap().effect.prim {
            Prim::Trail(tr) => tr.mirror,
            _ => panic!("riposte is a trail"),
        };
        assert_eq!((riposte(false), riposte(true)), (false, true), "mirrored when facing left");
    }

    /// S4: the apply burst is a Ring r0 3 to r1 10 at alpha 200 for 220 ms at the chest in the look's first colour, in
    /// the air; stealth (no colour) has none.
    #[test]
    fn the_apply_burst_rings_the_chest_in_the_first_colour() {
        let t = tuning();
        let geo = Geom::new(View::Stage, &t);
        let foot = (200.0, 150.0);
        let e = apply_burst(status_look("poison").unwrap(), &geo, &t, foot, 1234).unwrap();
        assert_eq!((e.start, e.layer, e.ms()), (1234, Layer::Air, 220));
        assert_eq!(e.prim, Prim::Ring(Ring { at: slot_anchor(&geo, foot, Slot::Body), r0: 3.0, r1: 10.0, ms: 220, col: [VIOLET[0], VIOLET[1], VIOLET[2], 200], squash: 0.75 }));
        assert!(matches!(apply_burst(status_look("weaken").unwrap(), &geo, &t, foot, 0).unwrap().prim, Prim::Ring(r) if r.col == alpha(ASH, 200)));
        assert!(apply_burst(status_look("stealth").unwrap(), &geo, &t, foot, 0).is_none());
    }
}
