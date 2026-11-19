//! TRANSITIONS1 (`PR7-14`, games/emberwake/docs/specs/TRANSITIONS1.md): the beats between the story, the battle and
//! the hub. T1, the battle beat from a story vignette into its fight: the ash cover over the new battle stage (the
//! combat sting playing), the ash wipe opening it, the fight's banner (its name, objective and enemy count) sliding in
//! while the party walks in from the edge to its start cells; control starts as the banner leaves (`INTRO_MS`). A click
//! or a key skips it; with Reduce motion it is a plain `PLAIN_MS` fade. T2, the victory beat: the last hit in slow motion
//! (`SLOWMO_RATE` of the FX clock for `SLOWMO_MS`), the Victory banner, then the results over the dimmed field (sliding
//! in), Continue fading to black with the victory stinger, and the hub's arrival card with the change lines in the
//! ticker (`Arrival`); when the story goes on to a vignette the arrival is dropped. The timing is pure (`intro_frame`,
//! `victory_frame`, `results_shift`, `out_alpha`): the screens hold a `Beat` and draw its frame. Placeholder art and
//! stings until Art/UI `AI-18` lands (the paths below; each piece falls back to a drawn one when missing).

use aegis_core::clock::Instant;
use aegis_core::json::{self, s, s_or};
use aegis_ui::art::Art;
use aegis_ui::canvas::{rgba, Image, Rect, Rgba};
use aegis_ui::painter::Painter;
use aegis_ui::theme::th;
use aegis_ui::ui::{BODY, H, HEAD, SMALL, TITLE, W};
use emberwake_world::state::GameState;
use emberwake_world::Data;

/// T1: the whole battle beat (ms), and Reduce motion's plain fade.
pub const INTRO_MS: u64 = 2000;
pub const PLAIN_MS: u64 = 300;
/// T1 steps 1-2: the ash cover holds while the sting hits, then the wipe opens the stage until `WIPE_END_MS`.
pub const COVER_HOLD_MS: u64 = 250;
pub const WIPE_END_MS: u64 = 850;
/// T1 step 3: the banner slides in at `BANNER_IN_MS` and out over the beat's last `BANNER_SLIDE_MS`; the party walks in
/// from `WALK_CELLS` cells toward the nearer side edge, starting at `WALK_AT_MS` for `WALK_MS` on the FX clock.
pub const BANNER_IN_MS: u64 = 700;
pub const BANNER_SLIDE_MS: u64 = 200;
pub const WALK_AT_MS: u64 = 850;
pub const WALK_MS: u64 = 800;
pub const WALK_CELLS: i32 = 3;
/// T2 step 1: the victory beat (ms) before the results open, its slow motion and the Victory banner's entry.
pub const VICTORY_MS: u64 = 1800;
pub const SLOWMO_MS: u64 = 700;
pub const SLOWMO_RATE: f64 = 0.3;
pub const VICTORY_BANNER_AT_MS: u64 = 250;
/// T2 step 2: the results slide in from the right over `RESULTS_SLIDE_MS` (by `RESULTS_SLIDE_PX`).
pub const RESULTS_SLIDE_MS: u64 = 250;
pub const RESULTS_SLIDE_PX: f32 = 120.0;
/// T2 step 3: Continue's fade to black (Reduce motion: `PLAIN_MS`).
pub const OUT_MS: u64 = 500;
/// T2 step 4: the arrival card's time on the map (its last `ARRIVAL_FADE_MS` fade it out).
pub const ARRIVAL_MS: u64 = 4000;
pub const ARRIVAL_FADE_MS: u64 = 400;

/// The art and stings (`AI-18` final names; placeholder copies until the finals land in place).
pub const ASH_MASK: &str = "ui/transitions/ash_wipe_mask.png";
pub const ASH_FLECKS: &str = "ui/transitions/ash_flecks.png";
pub const OBJECTIVE_BANNER: &str = "ui/transitions/objective_banner.png";
pub const VICTORY_BANNER: &str = "ui/transitions/victory_banner.png";
pub const COMBAT_STING: &str = "stings/combat_sting.ogg";
pub const VICTORY_STINGER: &str = "stings/victory_stinger.ogg";

/// The ash cover's colour (theme `ash_cover`).
pub const ASH: Rgba = rgba(14, 12, 12, 255);

/// The ash wipe's mask is read at this size (logical 640 x 360 drawn from it, nearest sampled) and in this many steps.
const MASK_W: i32 = 320;
const MASK_H: i32 = 180;
const MASK_STEPS: f32 = 48.0;
/// The ragged front's width in mask levels (the flecks show there).
const FRONT: f32 = 14.0;

/// Which beat runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Intro,
    Victory,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Intro => "intro",
            Kind::Victory => "victory",
        }
    }
}

/// A beat that runs on the wall clock from `started`; `plain` with Reduce motion on.
#[derive(Debug, Clone, Copy)]
pub struct Beat {
    pub kind: Kind,
    pub plain: bool,
    pub started: Instant,
}

impl Beat {
    pub fn new(kind: Kind, plain: bool) -> Self {
        Self { kind, plain, started: Instant::now() }
    }

    /// Wall ms since it started.
    pub fn ms(&self, now: Instant) -> u64 {
        now.saturating_duration_since(self.started).as_millis() as u64
    }

    /// Its length (ms): T1 two seconds (Reduce motion: the plain fade), T2 the victory beat.
    pub fn length(&self) -> u64 {
        match (self.kind, self.plain) {
            (Kind::Intro, true) => PLAIN_MS,
            (Kind::Intro, false) => INTRO_MS,
            (Kind::Victory, _) => VICTORY_MS,
        }
    }

    pub fn done(&self, now: Instant) -> bool {
        self.ms(now) >= self.length()
    }

    /// T2 step 1: the FX clock's rate (the last hit in slow motion; Reduce motion: none).
    pub fn clock_rate(&self, now: Instant) -> f64 {
        if self.kind == Kind::Victory && !self.plain && self.ms(now) < SLOWMO_MS {
            SLOWMO_RATE
        } else {
            1.0
        }
    }

    /// T1 step 3: the party walks in (never with Reduce motion).
    pub fn walks(&self) -> bool {
        self.kind == Kind::Intro && !self.plain
    }

    pub fn frame(&self, now: Instant) -> Frame {
        let ms = self.ms(now);
        match self.kind {
            Kind::Intro => intro_frame(ms, self.plain),
            Kind::Victory => victory_frame(ms, self.plain),
        }
    }

    /// The agent link's `beat`.
    pub fn state(&self, now: Instant) -> serde_json::Value {
        let f = self.frame(now);
        serde_json::json!({"kind": self.kind.name(), "plain": self.plain, "ms": self.ms(now), "length": self.length(), "cover": f.cover, "fade": f.fade, "banner": f.banner})
    }
}

/// One frame of a beat: how much of the screen the ash covers (1 all, 0 none), the plain black fade's opacity and the
/// banner's place (None: not shown; 0 out of view .. 1 in place).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub cover: f32,
    pub fade: f32,
    pub banner: Option<f32>,
}

fn ease(u: f32) -> f32 {
    let u = u.clamp(0.0, 1.0);
    1.0 - (1.0 - u) * (1.0 - u)
}

fn ratio(ms: u64, from: u64, to: u64) -> f32 {
    if to <= from {
        return 1.0;
    }
    ((ms.saturating_sub(from)) as f32 / (to - from) as f32).clamp(0.0, 1.0)
}

/// T1 at `ms`: the cover holds, then the wipe opens the stage; the banner slides in, holds and slides out as control
/// starts. Plain: only a black fade from 1 to 0 over `PLAIN_MS`.
pub fn intro_frame(ms: u64, plain: bool) -> Frame {
    if plain {
        return Frame { cover: 0.0, fade: 1.0 - ratio(ms, 0, PLAIN_MS), banner: None };
    }
    let cover = if ms < COVER_HOLD_MS { 1.0 } else { 1.0 - ratio(ms, COVER_HOLD_MS, WIPE_END_MS) };
    let banner = if !(BANNER_IN_MS..INTRO_MS).contains(&ms) {
        None
    } else if ms >= INTRO_MS - BANNER_SLIDE_MS {
        Some(1.0 - ease(ratio(ms, INTRO_MS - BANNER_SLIDE_MS, INTRO_MS)))
    } else {
        Some(ease(ratio(ms, BANNER_IN_MS, BANNER_IN_MS + BANNER_SLIDE_MS)))
    };
    Frame { cover, fade: 0.0, banner }
}

/// T2 at `ms`: the Victory banner slides in after the last hit (plain: in place at once) and stays until the results.
pub fn victory_frame(ms: u64, plain: bool) -> Frame {
    let banner = if plain {
        Some(1.0)
    } else if ms < VICTORY_BANNER_AT_MS {
        None
    } else {
        Some(ease(ratio(ms, VICTORY_BANNER_AT_MS, VICTORY_BANNER_AT_MS + BANNER_SLIDE_MS)))
    };
    Frame { cover: 0.0, fade: 0.0, banner }
}

/// T2 step 2: the results' shift right (px) `ms` after they opened (plain: none).
pub fn results_shift(ms: u64, plain: bool) -> i32 {
    if plain {
        0
    } else {
        ((1.0 - ease(ratio(ms, 0, RESULTS_SLIDE_MS))) * RESULTS_SLIDE_PX).round() as i32
    }
}

/// T2 step 3: Continue's fade length, and the black's opacity (0-255) `ms` into it.
pub fn out_ms(plain: bool) -> u64 {
    if plain {
        PLAIN_MS
    } else {
        OUT_MS
    }
}

pub fn out_alpha(ms: u64, plain: bool) -> u8 {
    (ratio(ms, 0, out_ms(plain)) * 255.0).round() as u8
}

/// T2 step 4: the arrival card's opacity (0-255) `ms` after it showed (plain: no fade), 0 once its time is up.
pub fn arrival_alpha(ms: u64, plain: bool) -> u8 {
    if ms >= ARRIVAL_MS {
        0
    } else if plain || ms < ARRIVAL_MS - ARRIVAL_FADE_MS {
        255
    } else {
        ((1.0 - ratio(ms, ARRIVAL_MS - ARRIVAL_FADE_MS, ARRIVAL_MS)) * 255.0).round() as u8
    }
}

fn with_alpha(c: Rgba, a: f32) -> Rgba {
    rgba(c[0], c[1], c[2], (c[3] as f32 * a.clamp(0.0, 1.0)).round() as u8)
}

/// The ash wipe's last built cover (its step and image), so a frame at the same step reuses it.
#[derive(Default)]
pub struct Ash {
    built: Option<(i32, Option<Image>)>,
}

impl Ash {
    /// The cover at `cover` (0..1) from the mask (`ASH_MASK`: a pixel is ash while its level is under `cover x 255`; the
    /// front band takes the flecks' colours, else the ember); None without the mask.
    fn image(&mut self, art: &mut Art, cover: f32) -> Option<&Image> {
        let step = (cover * MASK_STEPS).round() as i32;
        if self.built.as_ref().map(|b| b.0) != Some(step) {
            let t = step as f32 / MASK_STEPS * 255.0;
            let mask = art.image(ASH_MASK, MASK_W, MASK_H).cloned();
            let flecks = art.image(ASH_FLECKS, MASK_W, MASK_H).cloned();
            let (ash, ember) = (th().game("ash_cover", ASH), th().game("ash_ember", rgba(0xd9, 0x77, 0x06, 230)));
            let img = mask.map(|m| {
                let mut px = vec![0u8; (MASK_W * MASK_H * 4) as usize];
                for i in 0..(MASK_W * MASK_H) as usize {
                    let level = m.px.get(i * 4).copied().unwrap_or(255) as f32;
                    let c = if level < t - FRONT {
                        ash
                    } else if level < t {
                        match flecks.as_ref().map(|f| &f.px[i * 4..i * 4 + 4]) {
                            Some(f) if f[3] > 0 => rgba(f[0], f[1], f[2], 255),
                            _ => ember,
                        }
                    } else {
                        continue;
                    };
                    px[i * 4..i * 4 + 4].copy_from_slice(&c);
                }
                Image { w: MASK_W, h: MASK_H, px }
            });
            self.built = Some((step, img));
        }
        self.built.as_ref().and_then(|b| b.1.as_ref())
    }

    /// Draws the cover over the whole screen: all of it at 1, the mask's wipe between, a plain left-to-right sweep
    /// without the mask.
    pub fn draw(&mut self, p: &mut Painter, art: &mut Art, cover: f32) {
        if cover <= 0.0 {
            return;
        }
        let ash = th().game("ash_cover", ASH);
        if cover >= 1.0 {
            p.fill(Rect::new(0, 0, W, H), ash);
            return;
        }
        match self.image(art, cover) {
            Some(img) => p.image(img, Rect::new(0, 0, W, H)),
            None => p.fill(Rect::new(0, 0, (W as f32 * cover).round() as i32, H), ash),
        }
    }
}

/// A plain black fade at opacity `fade` (0..1).
pub fn draw_fade(p: &mut Painter, fade: f32) {
    if fade > 0.0 {
        p.fill(Rect::new(0, 0, W, H), with_alpha(th().game("fade_black", rgba(0, 0, 0, 255)), fade));
    }
}

/// A banner frame: the art (`rel`) scaled into `r`, else a drawn panel in the frame colours.
fn banner_frame(p: &mut Painter, art: &mut Art, rel: &str, r: Rect) {
    match art.image(rel, r.w, r.h) {
        Some(img) => p.image(img, r),
        None => p.panel(r, th().game("banner_fill", rgba(17, 19, 26, 240)), th().edge, 3.0),
    }
}

/// T1 step 3: the fight's banner (name, objective, enemy count).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IntroCard {
    pub name: String,
    pub objective: String,
    pub foes: usize,
}

impl IntroCard {
    pub fn count_line(&self) -> String {
        match self.foes {
            1 => "1 enemy".to_string(),
            n => format!("{n} enemies"),
        }
    }
}

/// The intro banner's rect when in place, and where it slides from (above the screen).
pub const INTRO_BANNER: Rect = Rect::new((W - 440) / 2, 34, 440, 70);

/// Draws T1's banner at place `at` (0..1).
pub fn draw_intro_banner(p: &mut Painter, art: &mut Art, card: &IntroCard, at: f32) {
    let r = INTRO_BANNER;
    let y = (-(r.h as f32) + (r.y + r.h) as f32 * at).round() as i32;
    let r = Rect::new(r.x, y, r.w, r.h);
    banner_frame(p, art, OBJECTIVE_BANNER, r);
    let mid = r.x as f32 + r.w as f32 / 2.0;
    p.text_center(mid, r.y as f32 + 8.0, &card.name, TITLE + 2.0, th().gold);
    let line = p.wrap(&card.objective, BODY, r.w as f32 - 40.0).into_iter().next().unwrap_or_default();
    p.text_center(mid, r.y as f32 + 30.0, &line, BODY, th().text);
    p.text_center(mid, r.y as f32 + 48.0, &card.count_line(), SMALL, th().dim);
}

/// T2's banner rect when in place.
pub const VICTORY_RECT: Rect = Rect::new((W - 300) / 2, 120, 300, 70);

/// Draws T2's Victory banner at place `at` (0..1): it drops into place as it shows.
pub fn draw_victory_banner(p: &mut Painter, art: &mut Art, at: f32) {
    let r = VICTORY_RECT;
    let r = Rect::new(r.x, r.y - ((1.0 - at) * 24.0).round() as i32, r.w, r.h);
    banner_frame(p, art, VICTORY_BANNER, r);
    p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + (r.h as f32 - p.line_h(HEAD + 10.0)) / 2.0, "Victory", HEAD + 10.0, with_alpha(th().gold, at));
}

// ---- T2 step 4: the arrival ------------------------------------------------------------------------------------------

/// What the hub shows on arrival: the one-line card (read again as the hub opens: `RegionMap` takes it then) and the
/// change lines for the ticker.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Arrival {
    pub card: String,
    pub changes: Vec<String>,
}

/// What the arrival compares (taken as the fight starts): the waiting bunk talks and each shop's offers.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Snapshot {
    talks: Vec<String>,
    stock: Vec<(String, String)>,
}

fn talk_keys(d: &Data, gs: &GameState) -> Vec<(String, String, String)> {
    emberwake_world::talks::waiting(d, gs).into_iter().map(|w| (format!("{}:{}:{}", w.qid, w.a, w.b), w.a, w.b)).collect()
}

fn stock_keys(d: &Data, gs: &GameState) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for shop in d.merged_entries(&gs.world_id, "shops") {
        let qid = s(shop, "_qid");
        for o in emberwake_world::shop::offers(d, gs, &qid) {
            let kind = s(&o, "kind");
            if kind == emberwake_world::shop::SUPPLIES || json::i(&o, "left", 0) == 0 {
                continue;
            }
            out.push((qid.clone(), format!("{kind}:{}", s(&o, "id")), s(&o, "item")));
        }
    }
    out
}

pub fn snapshot(d: &Data, gs: &GameState) -> Snapshot {
    Snapshot { talks: talk_keys(d, gs).into_iter().map(|t| t.0).collect(), stock: stock_keys(d, gs).into_iter().map(|(shop, key, _)| (shop, key)).collect() }
}

/// The change lines since `before`: each new waiting talk ("New talk at the bunks: Ash and Bryn."), then each shop with
/// new stock ("New stock at The Tinker's Hearth: Fire Flask, Rope.").
pub fn changes(d: &Data, gs: &GameState, before: &Snapshot) -> Vec<String> {
    let name = |id: &str| gs.party.member(id).map(|m| emberwake_world::progression::display_name(d, m)).unwrap_or_else(|| id.to_string());
    let mut out: Vec<String> = talk_keys(d, gs).into_iter().filter(|t| !before.talks.contains(&t.0)).map(|(_, a, b)| format!("New talk at the bunks: {} and {}.", name(&a), name(&b))).collect();
    let mut shops: Vec<(String, Vec<String>)> = Vec::new();
    for (shop, key, item) in stock_keys(d, gs) {
        if before.stock.contains(&(shop.clone(), key)) {
            continue;
        }
        let item = d.item_name(&item);
        match shops.iter_mut().find(|x| x.0 == shop) {
            Some(x) if !x.1.contains(&item) => x.1.push(item),
            Some(_) => {}
            None => shops.push((shop, vec![item])),
        }
    }
    for (shop, items) in shops {
        let row = d.resolve(&gs.world_id, "shops", &shop);
        out.push(format!("New stock at {}: {}.", s_or(row, "name", &s(row, "id")), items.join(", ")));
    }
    out
}

/// The card: "Back at <location> · Day <n>", then each visible running countdown ("Tithing 41 days").
pub fn card(d: &Data, gs: &GameState) -> String {
    let loc = d.resolve(&gs.world_id, "locations", &gs.location_id);
    let mut parts = vec![format!("Back at {}", s_or(loc, "name", &gs.location_id)), format!("Day {}", gs.time + 1)];
    if let Some(c) = gs.campaign.as_ref() {
        for (id, def) in c.defs.iter() {
            let Some(st) = c.meters.get(id).filter(|st| st.visible && st.running && def.countdown) else { continue };
            let short = s(json::dget(&def.row, "hud"), "short");
            let label = if !st.label.is_empty() { st.label.clone() } else if short.is_empty() { def.name.clone() } else { short };
            parts.push(format!("{label} {}", emberwake_world::campaign::view::with_unit(st.value(def, gs.time), &s(&def.row, "unit"))));
        }
    }
    parts.join(" \u{b7} ")
}

pub fn arrival(d: &Data, gs: &GameState, before: &Snapshot) -> Arrival {
    Arrival { card: card(d, gs), changes: changes(d, gs, before) }
}

/// The arrival card's rect (centred at the top, below the HUD) for text `w` px wide.
pub fn arrival_rect(text_w: f32) -> Rect {
    let w = (text_w.ceil() as i32 + 24).min(W - 16);
    Rect::new((W - w) / 2, 30, w, 20)
}

/// Draws the card at opacity `a` (0-255).
pub fn draw_arrival(p: &mut Painter, text: &str, a: u8) {
    if a == 0 {
        return;
    }
    let k = a as f32 / 255.0;
    let r = arrival_rect(p.text_w(text, BODY));
    p.panel(r, with_alpha(th().game("arrival_fill", rgba(17, 19, 26, 235)), k), with_alpha(th().gold, k), 3.0);
    p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + (r.h as f32 - p.line_h(BODY)) / 2.0, text, BODY, with_alpha(th().text, k));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_intro_covers_wipes_and_shows_the_banner_until_control_starts() {
        let f = |ms| intro_frame(ms, false);
        assert_eq!((f(0).cover, f(COVER_HOLD_MS - 1).cover), (1.0, 1.0), "step 2: the ash covers the new stage");
        assert!(f((COVER_HOLD_MS + WIPE_END_MS) / 2).cover > 0.0 && f((COVER_HOLD_MS + WIPE_END_MS) / 2).cover < 1.0, "the wipe opens it");
        assert_eq!(f(WIPE_END_MS).cover, 0.0);
        assert_eq!(f(BANNER_IN_MS - 1).banner, None);
        assert_eq!(f(BANNER_IN_MS + BANNER_SLIDE_MS).banner, Some(1.0), "step 3: the banner in place");
        assert!(f(INTRO_MS - 1).banner.is_some_and(|b| b < 0.1), "it leaves as control starts");
        assert_eq!(f(INTRO_MS).banner, None);
        assert!((0..INTRO_MS).all(|ms| f(ms).fade == 0.0));
    }

    #[test]
    fn reduce_motion_makes_the_intro_a_plain_fade() {
        let f = |ms| intro_frame(ms, true);
        assert_eq!(f(0), Frame { cover: 0.0, fade: 1.0, banner: None });
        assert_eq!(f(PLAIN_MS).fade, 0.0);
        assert!((0..PLAIN_MS).all(|ms| f(ms).cover == 0.0 && f(ms).banner.is_none()));
        let b = Beat { kind: Kind::Intro, plain: true, started: Instant::now() };
        assert_eq!(b.length(), PLAIN_MS);
        assert!(!b.walks(), "no walk-in");
        assert!(Beat { plain: false, ..b }.walks());
    }

    #[test]
    fn the_victory_beat_slows_the_last_hit_then_shows_the_banner() {
        let at = |kind, plain, ms| {
            let b = Beat { kind, plain, started: Instant::now() };
            (b, b.started + aegis_core::clock::Duration::from_millis(ms))
        };
        let (b, t) = at(Kind::Victory, false, 0);
        assert_eq!(b.clock_rate(t), SLOWMO_RATE);
        let (b, t) = at(Kind::Victory, false, SLOWMO_MS);
        assert_eq!(b.clock_rate(t), 1.0);
        let (b, t) = at(Kind::Victory, true, 0);
        assert_eq!(b.clock_rate(t), 1.0, "Reduce motion: no slow motion");
        let (b, t) = at(Kind::Intro, false, 0);
        assert_eq!(b.clock_rate(t), 1.0);
        assert_eq!(victory_frame(0, false).banner, None);
        assert_eq!(victory_frame(VICTORY_BANNER_AT_MS + BANNER_SLIDE_MS, false).banner, Some(1.0));
        assert_eq!(victory_frame(0, true).banner, Some(1.0), "plain: in place at once");
        assert_eq!((results_shift(0, false), results_shift(RESULTS_SLIDE_MS, false), results_shift(0, true)), (RESULTS_SLIDE_PX as i32, 0, 0));
        assert_eq!((out_alpha(0, false), out_alpha(OUT_MS, false), out_alpha(PLAIN_MS, true)), (0, 255, 255));
        assert_eq!((arrival_alpha(0, false), arrival_alpha(ARRIVAL_MS, false), arrival_alpha(ARRIVAL_MS - 1, true)), (255, 0, 255));
    }

    #[test]
    fn the_count_line_says_enemy_or_enemies() {
        let c = |n| IntroCard { foes: n, ..Default::default() }.count_line();
        assert_eq!((c(1), c(3)), ("1 enemy".to_string(), "3 enemies".to_string()));
    }
}
