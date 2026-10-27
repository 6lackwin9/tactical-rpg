//! Battle effects, the timeline part (FX1-L1, spec games/emberwake/docs/specs/FX1.md section 4): how a popped action
//! reaches its target (`Delivery`, 4.3), when it lands (`impact_at`, 4.2 step 2 and decision "Attack timing on the
//! board" A: the attack clip's `impact` frame), the queue of impacts waiting to show the step's hit (`Impact`, rules I
//! and F; rule P waits for them in `BattleScreen::tick`), the struck token's shake (4.5) and the anchors of both board
//! views (4.1). Engine-only: Godot had token tweens in `scripts/ui/combat/fx_presets.gd` (its hit shake is ported
//! here without the bug that kept it on one side). Presentation only: nothing here reads or changes the battle.

use crate::stage::{Geom, View};
use crate::tuning::Tuning;
use aegis_ui::canvas::Rgba;
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

/// 4.3, first match (the delivery part; the looks of each row are FX1-L2's and slice 2's): `arcane_bolt` and an
/// arcane strike at 2 or more cells fly the bolt (420 px/s); a ranged weapon shoots by family (bow 560 arc 6,
/// crossbow 720 arc 2, sling 480 arc 8, others as the bow); any other strike at 2 or more cells flies the reach bolt
/// (480); `shield_bash` and an adjacent strike are blows; support and items cast; anything else has none.
pub fn delivery(a: Act) -> Delivery {
    let hits = a.kind == "attack" || a.kind == "area";
    let shot = |speed: f32, arc: f32| Delivery::Shot { speed, arc };
    match a.skill {
        "arcane_bolt" => return shot(420.0, 0.0),
        "shield_bash" => return Delivery::Melee,
        _ => {}
    }
    if hits && a.dist >= 2 && (ARCANE_FAMILIES.contains(&a.family) || a.magic) {
        return shot(420.0, 0.0);
    }
    if hits && a.ranged {
        return match a.family {
            "crossbow" => shot(720.0, 2.0),
            "sling" => shot(480.0, 8.0),
            _ => shot(560.0, 6.0),
        };
    }
    if hits && a.dist >= 2 {
        return shot(480.0, 0.0);
    }
    match a.kind {
        "attack" | "area" => Delivery::Melee,
        "support" | "item" => Delivery::Cast,
        _ => Delivery::None,
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_ui::tuning::Tuned;
    use serde_json::json;

    fn act<'a>(kind: &'a str, skill: &'a str, family: &'a str, ranged: bool, dist: i32) -> Act<'a> {
        Act { kind, skill, family, ranged, magic: false, dist }
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
