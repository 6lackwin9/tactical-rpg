//! VNSTAGE1 (PR7-27): the vignette player's story stage. A card may put actors on the stage (full-body pictures, a
//! code-drawn figure where a layer holds none) that stand left or right and enter from the left or leave to the right
//! while the card plays, under one of three cameras: Static (the backdrop holds), Scroll (the backdrop scrolls past
//! while the left side walks in place) and Pan (the backdrop and everything on it slide left while the left side walks
//! a little forward). A right-side actor is pinned to the backdrop: under Scroll and Pan it starts off the frame's right
//! edge and comes into view as the backdrop moves. Dialogue and narration (line and text cards) keep the scene's last
//! painted backdrop on the one stage rectangle (`STAGE`, the image cards' picture area), dimmed by `DIM_ALPHA`, instead
//! of a plain colour. Positions are pure functions of the card and its walk progress, so tests read them without
//! drawing. Which actors a card holds and their pictures are the caller's to resolve (the card's `actors` rows name
//! them); this module names no game.

use aegis_core::json;
use aegis_ui::art::Art;
use aegis_ui::canvas::{rgba, Rect, Rgba};
use aegis_ui::painter::Painter;
use aegis_ui::theme::th;
use serde_json::Value;

/// The stage rectangle: every painted backdrop is fitted to it (the image cards' picture area).
pub const STAGE: Rect = crate::vignette::PICTURE;
/// The dim over a kept backdrop under a line or text card (102 of 255, about 40 %).
pub const DIM_ALPHA: u8 = 102;
/// How long a card's walk takes when it names none (`walk_ms`).
pub const WALK_MS: u32 = 1600;
/// How far the backdrop moves under Scroll and Pan over the walk.
pub const SCROLL_PX: i32 = 280;
/// How far a left-side actor walks forward under Pan (the camera follows it).
pub const PAN_FOLLOW_PX: i32 = 60;
/// An actor's picture box (full-body pictures are square; the feet sit `FEET_INSET` above the stage's bottom).
pub const ACTOR_PX: i32 = 200;
pub const FEET_INSET: i32 = 4;
/// The walk's bob (px) for an actor on the move.
pub const BOB_PX: f32 = 2.0;
/// Where each side stands, as a share of the stage's width (the actor's centre).
pub const LEFT_HOME: f32 = 0.25;
pub const RIGHT_HOME: f32 = 0.72;
/// The most actors a card holds.
pub const ACTORS_MAX: usize = 4;
/// The step between actors on the same side: the first stands at its home, each next one this far behind it (toward
/// its own edge), so a party reads as a line, not a stack.
pub const SLOT_PX: i32 = 56;

/// The side an actor stands on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

/// What an actor does over its card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Stay,
    /// Walks in from off the frame's left edge (a right-side actor from off its right edge under Static).
    Enter,
    /// Walks out past the frame's right edge.
    Exit,
}

/// The stage camera of a card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StageCamera {
    #[default]
    Static,
    Scroll,
    Pan,
}

impl Side {
    pub fn parse(s: &str) -> Option<Side> {
        match s {
            "left" => Some(Side::Left),
            "right" => Some(Side::Right),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Side::Left => "left",
            Side::Right => "right",
        }
    }
}

impl Motion {
    pub fn parse(s: &str) -> Option<Motion> {
        match s {
            "stay" => Some(Motion::Stay),
            "enter" => Some(Motion::Enter),
            "exit" => Some(Motion::Exit),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Motion::Stay => "stay",
            Motion::Enter => "enter",
            Motion::Exit => "exit",
        }
    }
}

impl StageCamera {
    pub fn parse(s: &str) -> Option<StageCamera> {
        match s {
            "static" => Some(StageCamera::Static),
            "scroll" => Some(StageCamera::Scroll),
            "pan" => Some(StageCamera::Pan),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            StageCamera::Static => "static",
            StageCamera::Scroll => "scroll",
            StageCamera::Pan => "pan",
        }
    }
}

/// One actor of a card: `who` (the row's id), and as the caller resolves them its shown `name` and its picture chain
/// (`sprite`, the first file a layer holds; none draws the code figure).
#[derive(Debug, Clone, PartialEq)]
pub struct Actor {
    pub who: String,
    pub name: String,
    pub sprite: Vec<String>,
    pub side: Side,
    pub motion: Motion,
}

impl Actor {
    /// An actor row `{"who", "side" (left), "move" (stay)}`; bad values fall back to the defaults; None without `who`.
    pub fn from_json(v: &Value) -> Option<Actor> {
        let d = json::dict(v);
        let who = json::s(d, "who");
        if who.is_empty() {
            return None;
        }
        Some(Actor { name: who.clone(), who, sprite: Vec::new(), side: Side::parse(&json::s(d, "side")).unwrap_or(Side::Left), motion: Motion::parse(&json::s(d, "move")).unwrap_or(Motion::Stay) })
    }

    /// Whether the actor is on the move (it bobs): walking in or out, or a left-side actor under Scroll or Pan.
    pub fn walking(&self, camera: StageCamera, t: f32) -> bool {
        t < 1.0 && (self.motion != Motion::Stay || (self.side == Side::Left && camera != StageCamera::Static))
    }
}

/// The actors of a card's JSON `actors` (at most `ACTORS_MAX`, rows without `who` left out).
pub fn actors_of(v: &Value) -> Vec<Actor> {
    v.as_array().map(|a| a.iter().filter_map(Actor::from_json).take(ACTORS_MAX).collect()).unwrap_or_default()
}

/// The walk's progress at `card_ms` of a `walk_ms` walk, eased (smoothstep), 0..1; 1 at once for no walk.
pub fn progress(card_ms: u64, walk_ms: u32) -> f32 {
    if walk_ms == 0 {
        return 1.0;
    }
    let x = (card_ms as f32 / walk_ms as f32).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// The backdrop's horizontal offset at progress `t` (0 under Static; it moves left by `SCROLL_PX` under Scroll and
/// Pan).
pub fn backdrop_offset(camera: StageCamera, t: f32) -> i32 {
    match camera {
        StageCamera::Static => 0,
        StageCamera::Scroll | StageCamera::Pan => (-(SCROLL_PX as f32) * t).round() as i32,
    }
}

/// Each actor's slot: its place among the actors on its side, in the card's order (0 for the first).
pub fn slots(actors: &[Actor]) -> Vec<usize> {
    actors.iter().enumerate().map(|(i, a)| actors[..i].iter().filter(|b| b.side == a.side).count()).collect()
}

/// The x of `a`'s centre on the screen at progress `t` under `camera`, standing in slot `slot` of its side (`slots`):
/// the home moves `SLOT_PX` per slot toward the side's edge, and so does the off-frame start of a walk in, so a line
/// walks in together. Off the frame is half an actor box past the stage's edge.
pub fn actor_x(a: &Actor, camera: StageCamera, t: f32, slot: usize) -> i32 {
    let w = STAGE.w as f32;
    let step = (SLOT_PX * slot as i32) as f32;
    // the slot's own left edge of the stage: its home and its walk move with it
    let x0 = STAGE.x as f32 + if a.side == Side::Left { -step } else { step };
    let off_left = STAGE.x as f32 - ACTOR_PX as f32 / 2.0 - step;
    // a left line leaves past the same point (no one overtakes); a right line keeps its order past the edge
    let off_right = STAGE.x as f32 + w + ACTOR_PX as f32 / 2.0 + if a.side == Side::Right { step } else { 0.0 };
    let x = match a.side {
        Side::Left => {
            let follow = if camera == StageCamera::Pan { PAN_FOLLOW_PX as f32 * t } else { 0.0 };
            let home = x0 + w * LEFT_HOME + follow;
            match a.motion {
                Motion::Stay => home,
                Motion::Enter => lerp(off_left, home, t),
                Motion::Exit => lerp(x0 + w * LEFT_HOME, off_right, t),
            }
        }
        Side::Right => {
            let home = x0 + w * RIGHT_HOME;
            match (camera, a.motion) {
                // pinned to the backdrop: from off the right edge into its place as the backdrop moves
                (StageCamera::Scroll | StageCamera::Pan, _) => home + SCROLL_PX as f32 + backdrop_offset(camera, t) as f32,
                (StageCamera::Static, Motion::Stay) => home,
                (StageCamera::Static, Motion::Enter) => lerp(off_right, home, t),
                (StageCamera::Static, Motion::Exit) => lerp(home, off_right, t),
            }
        }
    };
    x.round() as i32
}

/// The rect a backdrop picture of `size` takes on the stage, moved by `offset`: fitted inside the stage under Static
/// (the image cards' letterbox), else covering the stage and `SCROLL_PX` more so the move never shows an edge.
pub fn backdrop_rect(size: (i32, i32), camera: StageCamera, offset: i32) -> Rect {
    let (iw, ih) = (size.0.max(1) as f32, size.1.max(1) as f32);
    let (aw, ah) = match camera {
        StageCamera::Static => (STAGE.w as f32, STAGE.h as f32),
        _ => ((STAGE.w + SCROLL_PX) as f32, STAGE.h as f32),
    };
    let k = match camera {
        StageCamera::Static => (aw / iw).min(ah / ih),
        _ => (aw / iw).max(ah / ih),
    };
    let (w, h) = ((iw * k).round() as i32, (ih * k).round() as i32);
    let x = match camera {
        StageCamera::Static => STAGE.x + (STAGE.w - w) / 2,
        _ => STAGE.x,
    };
    Rect::new(x + offset, STAGE.y + (STAGE.h - h) / 2, w, h)
}

/// Draws the backdrop picture `rel` on the stage (clipped to it) under `camera` at progress `t`, dimmed when `dim`;
/// false when no layer holds it (nothing drawn).
pub fn draw_backdrop(p: &mut Painter, art: &mut Art, rel: &str, camera: StageCamera, t: f32, dim: bool) -> bool {
    let Some(size) = art.size_of(rel).filter(|s| s.0 > 0 && s.1 > 0) else { return false };
    let r = backdrop_rect(size, camera, backdrop_offset(camera, t));
    let s = p.v.s;
    let (pw, ph) = (((r.w as f32 * s).round() as i32).clamp(1, size.0.max(1)), ((r.h as f32 * s).round() as i32).clamp(1, size.1.max(1)));
    let Some(img) = art.image(rel, pw, ph) else { return false };
    p.clip(Some(STAGE));
    p.image(img, r);
    if dim {
        p.fill(STAGE, rgba(0, 0, 0, DIM_ALPHA));
    }
    p.clip(None);
    true
}

/// Draws a card's actors on the stage (clipped to it) at progress `t` (`card_ms` for the bob; still under
/// `reduce_motion`): each in its picture box, feet on the stage floor, facing right; the speaker (`speaker`, an actor's
/// `who`) last and in full, the others a little faded while someone speaks.
pub fn draw_actors(p: &mut Painter, art: &mut Art, actors: &[Actor], camera: StageCamera, t: f32, card_ms: u64, speaker: &str, reduce_motion: bool) {
    if actors.is_empty() {
        return;
    }
    p.clip(Some(STAGE));
    let speaking = actors.iter().any(|a| a.who == speaker);
    let mut order: Vec<&Actor> = actors.iter().filter(|a| a.who != speaker).collect();
    order.extend(actors.iter().filter(|a| a.who == speaker));
    let slot = slots(actors);
    let slot_of = |a: &Actor| actors.iter().position(|b| std::ptr::eq(a, b)).map_or(0, |k| slot[k]);
    for (i, a) in order.into_iter().enumerate() {
        let bob = if a.walking(camera, t) && !reduce_motion { (((card_ms as f32 / 160.0) + i as f32).sin().abs() * BOB_PX).round() as i32 } else { 0 };
        let r = Rect::new(actor_x(a, camera, t, slot_of(a)) - ACTOR_PX / 2, STAGE.y + STAGE.h - FEET_INSET - ACTOR_PX - bob, ACTOR_PX, ACTOR_PX);
        let alpha = if speaking && a.who != speaker { 190 } else { 255 };
        let s = p.v.s;
        let px = ((ACTOR_PX as f32 * s).round() as i32).max(1);
        match art.first_rel(&a.sprite) {
            Some(rel) => match art.image(&rel, px, px) {
                Some(img) => p.image_alpha(img, r, alpha),
                None => figure(p, r, alpha),
            },
            None => {
                // the missing art goes to the playtest trail as for every chain
                let _ = art.first(&a.sprite, 1, 1);
                figure(p, r, alpha);
            }
        }
    }
    p.clip(None);
}

/// The code-drawn figure where no full-body picture exists (a placeholder: head, body and legs in the theme's dim).
pub fn figure(p: &mut Painter, r: Rect, alpha: u8) {
    // a shadowed silhouette (the theme's edge tone), so it reads as a stand-in, not as a character
    let c: Rgba = th().edge;
    let col = rgba(c[0], c[1], c[2], ((alpha as u32 * 230) / 255) as u8);
    let cx = (r.x + r.w / 2) as f32;
    let unit = r.h as f32 / 10.0;
    let floor = (r.y + r.h) as f32;
    let head_cy = floor - 8.4 * unit;
    p.ellipse(cx, head_cy, unit * 0.55, unit * 0.65, col);
    let shoulders = Rect::new((cx - unit * 1.15) as i32, (head_cy + unit * 0.75) as i32, (unit * 2.3) as i32, (unit * 3.6) as i32);
    p.panel(shoulders, col, col, unit * 0.5);
    let leg_top = shoulders.y + shoulders.h - (unit * 0.3) as i32;
    let leg_w = (unit * 0.8) as i32;
    let leg_h = floor as i32 - leg_top;
    p.panel(Rect::new((cx - unit * 0.9) as i32, leg_top, leg_w, leg_h), col, col, unit * 0.25);
    p.panel(Rect::new((cx + unit * 0.1) as i32, leg_top, leg_w, leg_h), col, col, unit * 0.25);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn actor(side: Side, motion: Motion) -> Actor {
        Actor { who: "a".into(), name: "A".into(), sprite: Vec::new(), side, motion }
    }

    #[test]
    fn rows_parse_with_defaults_and_drop_rows_without_who() {
        let a = actors_of(&json!([{"who": "x"}, {"who": "y", "side": "right", "move": "exit"}, {"side": "left"}, {"who": "z", "side": "up", "move": "fly"}]));
        assert_eq!(a.iter().map(|a| (a.who.as_str(), a.side, a.motion)).collect::<Vec<_>>(), [("x", Side::Left, Motion::Stay), ("y", Side::Right, Motion::Exit), ("z", Side::Left, Motion::Stay)]);
        assert_eq!(actors_of(&json!("nope")).len(), 0);
    }

    #[test]
    fn the_party_enters_left_and_exits_right() {
        let enter = actor(Side::Left, Motion::Enter);
        let exit = actor(Side::Left, Motion::Exit);
        let off_left = STAGE.x - ACTOR_PX / 2;
        let off_right = STAGE.x + STAGE.w + ACTOR_PX / 2;
        let home = (STAGE.x as f32 + STAGE.w as f32 * LEFT_HOME).round() as i32;
        for cam in [StageCamera::Static, StageCamera::Scroll] {
            assert_eq!(actor_x(&enter, cam, 0.0, 0), off_left);
            assert_eq!(actor_x(&enter, cam, 1.0, 0), home);
            assert_eq!(actor_x(&exit, cam, 0.0, 0), home);
            assert_eq!(actor_x(&exit, cam, 1.0, 0), off_right);
        }
        // monotonic left to right
        let xs: Vec<i32> = (0..=10).map(|k| actor_x(&enter, StageCamera::Static, k as f32 / 10.0, 0)).collect();
        assert!(xs.windows(2).all(|w| w[0] <= w[1]), "{xs:?}");
    }

    #[test]
    fn a_line_on_one_side_stands_in_slots_toward_its_edge() {
        let l = actor(Side::Left, Motion::Stay);
        let r = actor(Side::Right, Motion::Stay);
        assert_eq!(slots(&[l.clone(), r.clone(), l.clone(), r.clone()]), [0, 0, 1, 1]);
        assert_eq!(actor_x(&l, StageCamera::Static, 0.0, 1), actor_x(&l, StageCamera::Static, 0.0, 0) - SLOT_PX);
        assert_eq!(actor_x(&r, StageCamera::Static, 0.0, 1), actor_x(&r, StageCamera::Static, 0.0, 0) + SLOT_PX);
        // the second walks in from further off and leaves past the same point as the first
        let (enter, exit) = (actor(Side::Left, Motion::Enter), actor(Side::Left, Motion::Exit));
        assert!(actor_x(&enter, StageCamera::Static, 0.0, 1) < actor_x(&enter, StageCamera::Static, 0.0, 0));
        assert_eq!(actor_x(&exit, StageCamera::Static, 1.0, 1), actor_x(&exit, StageCamera::Static, 1.0, 0));
    }

    #[test]
    fn each_camera_moves_the_backdrop_and_the_pinned_right_side_its_own_way() {
        let npc = actor(Side::Right, Motion::Stay);
        let hero = actor(Side::Left, Motion::Stay);
        let home_r = (STAGE.x as f32 + STAGE.w as f32 * RIGHT_HOME).round() as i32;
        let home_l = (STAGE.x as f32 + STAGE.w as f32 * LEFT_HOME).round() as i32;
        // Static: nothing moves
        assert_eq!((backdrop_offset(StageCamera::Static, 1.0), actor_x(&npc, StageCamera::Static, 0.0, 0), actor_x(&hero, StageCamera::Static, 1.0, 0)), (0, home_r, home_l));
        // Scroll: the backdrop moves, the met one starts off the right edge and comes to its place, the hero walks in place
        assert_eq!(backdrop_offset(StageCamera::Scroll, 1.0), -SCROLL_PX);
        assert!(actor_x(&npc, StageCamera::Scroll, 0.0, 0) - ACTOR_PX / 2 >= STAGE.x + STAGE.w, "off the frame at the start");
        assert_eq!(actor_x(&npc, StageCamera::Scroll, 1.0, 0), home_r);
        assert_eq!(actor_x(&hero, StageCamera::Scroll, 1.0, 0), home_l);
        assert!(hero.walking(StageCamera::Scroll, 0.5) && !hero.walking(StageCamera::Static, 0.5));
        // Pan: as Scroll, and the hero walks forward with the camera
        assert_eq!(actor_x(&npc, StageCamera::Pan, 1.0, 0), home_r);
        assert_eq!(actor_x(&hero, StageCamera::Pan, 1.0, 0), home_l + PAN_FOLLOW_PX);
        // the backdrop covers the stage through the whole move
        for cam in [StageCamera::Scroll, StageCamera::Pan] {
            for t in [0.0, 0.5, 1.0] {
                let r = backdrop_rect((1280, 720), cam, backdrop_offset(cam, t));
                assert!(r.x <= STAGE.x && r.x + r.w >= STAGE.x + STAGE.w && r.y <= STAGE.y && r.y + r.h >= STAGE.y + STAGE.h, "{cam:?} {t}: {r:?}");
            }
        }
    }

    #[test]
    fn the_walk_eases_from_zero_to_one() {
        assert_eq!((progress(0, 1000), progress(1000, 1000), progress(5000, 1000), progress(10, 0)), (0.0, 1.0, 1.0, 1.0));
        assert!((progress(500, 1000) - 0.5).abs() < 1e-6);
    }
}
