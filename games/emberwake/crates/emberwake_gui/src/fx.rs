//! Battle effects, the timeline part (FX1-L1, spec games/emberwake/docs/specs/FX1.md section 4): how a popped action
//! reaches its target (`Delivery`, 4.3), when it lands (`impact_at`, 4.2 step 2 and decision "Attack timing on the
//! board" A: the attack clip's `impact` frame), the queue of impacts waiting to show the step's hit (`Impact`, rules I
//! and F; rule P waits for them in `BattleScreen::tick`), the struck token's shake (4.5) and the anchors of both board
//! views (4.1). Slice 2: the looks (`style`, 4.3 and 4.4.1-4.4.2 with the `arcane_bolt` and `shield_bash` rows), the
//! effects they push on the engine's live list (`delivery_effects` at the pop, `impact_effects` at the impact, with the
//! crit and mitigated modifiers) and the battle shakes (4.5). Engine-only: Godot had token tweens in
//! `scripts/ui/combat/fx_presets.gd` (its hit shake is ported here without the bug that kept it on one side).
//! Presentation only: nothing here reads or changes the battle.

use crate::stage::{Geom, View};
use crate::tuning::Tuning;
use aegis_ui::canvas::Rgba;
use aegis_ui::fx::{Effect, Flash, Projectile, Pt, Ring, Shake, ShakeTable, Sparks, Trail};
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
    /// Support and items: they land at `launch_ms`.
    Cast,
}

/// What `delivery` reads of a popped action: the command kind (`attack`, `area`, `support`, `item`, else none), the
/// skill id, the actor's main weapon family ("" none), whether that weapon is ranged (`Equipment::is_ranged_weapon`)
/// or deals magic damage, and the grid distance to the target or aim.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Act<'a> {
    pub kind: &'a str,
    pub skill: &'a str,
    pub family: &'a str,
    pub ranged: bool,
    pub magic: bool,
    pub dist: i32,
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

/// A shot's path in px (4.1): from the actor's anchor moved 6 px toward its target, to the target's anchor.
pub fn path_px(geo: &Geom, from: V2, to: V2) -> f32 {
    let (a, b) = (anchor(geo, from), anchor(geo, to));
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = (dx * dx + dy * dy).sqrt();
    (len - 6.0).max(0.0)
}

/// What one unit shows on the board after a step (rule I: the board keeps the state before the step until the impact).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shown {
    pub hp: i32,
    pub alive: bool,
    pub guarded: bool,
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

// ---- The looks (FX1 4.3, 4.4.1-4.4.2 and the L1 rows of 4.4.3; FX1_TASKS section 8) ------------------------------

/// The art bible's effect hues (FX1_TASKS 8): Polished Steel, Linen, Ash, Amber, Candle Gold, Resonant Orchid (FQ8)
/// and the crit hue Arterial Red.
pub const STEEL: Rgba = [0x9c, 0xa3, 0xaf, 255];
pub const LINEN: Rgba = [0xd9, 0xcb, 0xb4, 255];
pub const ASH: Rgba = [0x6b, 0x72, 0x80, 255];
pub const AMBER: Rgba = [0xd9, 0x77, 0x06, 255];
pub const GOLD: Rgba = [0xfb, 0xbf, 0x24, 255];
pub const ORCHID: Rgba = [0xc0, 0x84, 0xfc, 255];
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
/// unit), spread, speed, gravity and size (gravity 120 and size 1 unless the row gives them).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Burst {
    pub count: u8,
    pub cols: [Rgba; 2],
    pub dir: Option<f32>,
    pub spread: f32,
    pub speed: f32,
    pub gravity: f32,
    pub size: u8,
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

/// An action's look (4.3): its delivery (and timing), the swing trail or the projectile, and its impact.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub delivery: Delivery,
    pub swing: Option<Swing>,
    pub shot: Option<aegis_ui::fx::Look>,
    pub hit: Hit,
}

const NONE: Style = Style { delivery: Delivery::None, swing: None, shot: None, hit: Hit { ring: None, flash: None, sparks: None } };

const fn burst(count: u8, cols: [Rgba; 2], spread: f32, speed: f32) -> Burst {
    Burst { count, cols, dir: None, spread, speed, gravity: 120.0, size: 1 }
}

const fn swing(radius: f32, from_deg: f32, to_deg: f32, width: u8, col: Rgba) -> Option<Swing> {
    Some(Swing { radius, from_deg, to_deg, width, col })
}

/// 4.4.1, a blow by the main weapon's family ("" and `unarmed` read as `fist`; a family the table does not name
/// swings as the sword).
pub fn melee_style(family: &str) -> Style {
    let blow = |swing: Option<Swing>, ring: Option<Bloom>, sparks: Burst| Style { delivery: Delivery::Melee, swing, shot: None, hit: Hit { ring, flash: None, sparks: Some(sparks) } };
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
    use aegis_ui::fx::Look;
    let shot = |speed: f32, arc: f32, look: Look, sparks: Burst| Style { delivery: Delivery::Shot { speed, arc }, swing: None, shot: Some(look), hit: Hit { sparks: Some(sparks), ..Hit::default() } };
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
    swing: None,
    shot: Some(aegis_ui::fx::Look::Orb { r: 1.5, core: LINEN, glow: STEEL, tail: 2 }),
    hit: Hit { ring: None, flash: Some(Bloom { r0: 4.0, r1: 3.0, col: alpha(LINEN, 120), ms: 100 }), sparks: None },
};

/// 4.4.3 `arcane_bolt` (and 4.3 rule 2): an Orchid orb, a flash and sparks all round.
pub const ARCANE_BOLT: Style = Style {
    delivery: Delivery::Shot { speed: 420.0, arc: 0.0 },
    swing: None,
    shot: Some(aegis_ui::fx::Look::Orb { r: 2.5, core: LINEN, glow: ORCHID, tail: 3 }),
    hit: Hit { ring: None, flash: Some(Bloom { r0: 5.0, r1: 4.0, col: alpha(ORCHID, 150), ms: 120 }), sparks: Some(burst(6, [ORCHID, LINEN], 360.0, 50.0)) },
};

/// 4.4.3 `shield_bash`: no trail; a Steel ring and Steel and Gold sparks.
pub const SHIELD_BASH: Style = Style {
    delivery: Delivery::Melee,
    swing: None,
    shot: None,
    hit: Hit { ring: Some(Bloom { r0: 3.0, r1: 10.0, col: alpha(STEEL, 220), ms: 180 }), flash: None, sparks: Some(burst(8, [STEEL, GOLD], 120.0, 70.0)) },
};

/// The cast of support and items (4.4.4 is FX1-L2's): it lands at `launch_ms` and draws nothing yet.
const CAST: Style = Style { delivery: Delivery::Cast, ..NONE };

/// 4.3, first match: the skill rows (`arcane_bolt`, `shield_bash`; the rest are FX1-L2's), an arcane strike at 2 or
/// more cells, a ranged weapon by family, any other strike at 2 or more cells (the reach bolt), an adjacent strike by
/// family; support and items cast; anything else has no effect.
pub fn style(a: Act) -> Style {
    let hits = a.kind == "attack" || a.kind == "area";
    match a.skill {
        "arcane_bolt" => return ARCANE_BOLT,
        "shield_bash" => return SHIELD_BASH,
        _ => {}
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
        "support" | "item" => CAST,
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

/// Where the effects of one action go (4.1): the actor's and the target's anchors and the view's squash (Stage 0.75,
/// Flat 1) for rings and trails.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Place {
    pub from: Pt,
    pub to: Pt,
    pub squash: f32,
}

/// The effects of the delivery (4.2 step 2, pushed once the impact time `at` is known): the swing trail around the
/// actor's anchor (mirrored toward a target on its left), its head reaching the end of its arc at the impact (60 % of
/// `trail_ms` before it, never before `t0`), or the projectile from 6 px out toward the target, launched its travel
/// before the impact.
pub fn delivery_effects(s: &Style, pl: Place, t0: u64, at: u64, seed: u64, t: &Tuning) -> Vec<Effect> {
    let Place { from, to, squash } = pl;
    let mut out = Vec::new();
    if let Some(w) = s.swing {
        let start = at.saturating_sub(t.trail_ms * 6 / 10).max(t0);
        let trail = Trail { at: from, radius: w.radius, from_deg: w.from_deg, to_deg: w.to_deg, ms: t.trail_ms as u32, width: w.width, col: w.col, mirror: to.0 < from.0, squash };
        out.push(Effect::new(trail, start).seeded(seed));
    }
    if let (Some(look), Delivery::Shot { speed, arc }) = (s.shot, s.delivery) {
        let start = toward(from, to, 6.0);
        let (dx, dy) = (to.0 - start.0, to.1 - start.1);
        let travel = travel_ms((dx * dx + dy * dy).sqrt(), speed, t);
        let p = Projectile { from: start, to, travel_ms: travel as u32, arc_px: arc, look };
        out.push(Effect::new(p, at.saturating_sub(travel).max(t0)).seeded(seed));
    }
    out
}

/// The effects of the impact at `at` (4.2 step 3) on each point of `hits` (the struck units' anchors, or the target's):
/// the row's ring, flash and sparks (sparks along the attack from the actor unless the row gives a direction, the
/// modifiers `m` applied, `spark_ms` long), each hit seeded apart.
pub fn impact_effects(s: &Style, m: Mods, pl: Place, hits: &[Pt], at: u64, seed: u64, t: &Tuning) -> Vec<Effect> {
    let mut out = Vec::new();
    for (k, &p) in hits.iter().enumerate() {
        let seed = seed ^ (k as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        if let Some(r) = s.hit.ring {
            out.push(Effect::new(Ring { at: p, r0: r.r0, r1: r.r1, ms: r.ms, col: r.col, squash: pl.squash }, at).seeded(seed));
        }
        if let Some(f) = s.hit.flash {
            out.push(Effect::new(Flash { at: p, rx: f.r0, ry: f.r1, ms: f.ms, col: f.col }, at).seeded(seed));
        }
        if let Some(b) = s.hit.sparks.and_then(|b| modified(b, m)) {
            let dir_deg = b.dir.unwrap_or_else(|| deg(pl.from, p));
            let sparks = Sparks { at: p, ms: t.spark_ms as u32, count: b.count, dir_deg, spread_deg: b.spread, speed: b.speed, gravity: b.gravity, size: b.size, cols: b.cols };
            out.push(Effect::new(sparks, at).seeded(seed));
        }
    }
    out
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_ui::tuning::Tuned;
    use serde_json::json;

    fn act<'a>(kind: &'a str, skill: &'a str, family: &'a str, ranged: bool, dist: i32) -> Act<'a> {
        Act { kind, skill, family, ranged, magic: false, dist }
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
        let pl = Place { from: (100.0, 100.0), to: (132.0, 100.0), squash: 1.0 };
        let at = |m: Mods| impact_effects(&mace, m, pl, &[pl.to], 500, 7, &t);
        let s = sparks_of(&at(plain));
        assert_eq!((s.len(), s[0].count, s[0].cols, s[0].ms), (1, base.count, base.cols, t.spark_ms as u32));
        let c = sparks_of(&at(crit));
        assert_eq!((c[0].count, c[0].cols), (base.count * 2, [base.cols[0], CRIT]));
        let z = sparks_of(&at(zero));
        assert_eq!((z[0].count, z[0].cols), (base.count / 2, [STEEL, STEEL]));
        assert!(sparks_of(&at(miss)).is_empty(), "a miss has no sparks");
        assert_eq!(at(miss).len(), 1, "the ring stays");
        assert_eq!(s[0].dir_deg, 0.0, "sparks fly along the attack (to the right)");
        assert_eq!(impact_effects(&mace, plain, pl, &[pl.to, (164.0, 100.0)], 500, 7, &t).len(), 4, "each struck unit gets the row");
    }

    /// 4.2 step 2 with slice 2's effects: a swing's trail ends its sweep (60 % of `trail_ms`) at the impact, never
    /// starting before `t0`; a projectile leaves 6 px out from the actor and lands on the target at the impact.
    #[test]
    fn the_delivery_reaches_the_target_at_the_impact() {
        let t = tuning();
        let pl = Place { from: (100.0, 100.0), to: (60.0, 100.0), squash: 0.75 };
        let sword = melee_style("sword");
        let e = delivery_effects(&sword, pl, 1000, 1000 + 270, 3, &t);
        let aegis_ui::fx::Prim::Trail(tr) = e[0].prim else { panic!("a trail") };
        assert_eq!((e.len(), e[0].start, tr.ms as u64, tr.mirror, tr.squash), (1, 1270 - t.trail_ms * 6 / 10, t.trail_ms, true, 0.75), "a target on the left mirrors it");
        assert_eq!(delivery_effects(&sword, pl, 1000, 1000 + 20, 3, &t)[0].start, 1000, "never before t0");
        let bow = shot_style("bow");
        let e = delivery_effects(&bow, pl, 1000, 1500, 3, &t);
        let aegis_ui::fx::Prim::Projectile(p) = e[0].prim else { panic!("a projectile") };
        assert_eq!((p.from, p.to, e[0].end()), ((94.0, 100.0), pl.to, 1500));
        assert_eq!(p.travel_ms as u64, travel_ms(34.0, 560.0, &t));
        assert!(delivery_effects(&melee_style(""), pl, 1000, 1100, 3, &t).is_empty(), "a fist has no trail");
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
        assert_eq!(delivery(act("item", "", "", false, 0)), Delivery::Cast);
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
        assert_eq!(path_px(&flat, c, V2::new(7, 4)), (b.0 - a.0) - 6.0);
    }
}
