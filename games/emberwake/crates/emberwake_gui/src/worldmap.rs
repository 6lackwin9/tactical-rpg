//! The world map (scripts/ui/overworld/world_map_view.gd): known nodes and routes of the party's world
//! (TravelRules.view), trips by clicking a node (TravelRunner.travel: clock, supplies, road fights), making camp
//! (HealRules.make_camp) and entering the location the party stands on. Esc / Back returns to the region map.
//! STORYUI1-U1: like the region map, it hands over to the story stage and plays queued vignettes (`story::map_tick`).
//! CAMPAIGN1-U: campaign notices and the run-over screen (`runover::map_tick`); a trip or a camp whose days would end
//! the run asks first (W3, No as the default); a trip the run ended on stops like an arrival; a road fight that could
//! end the run gets the notice. Slice 2: a queued battle plays (`story::map_tick`; a queued dialogue waits for the
//! region map); the tips, the Camp button and the log name the purse's upkeep (P1, R1); the Travel tip adds the arrival
//! day and the days left (W5); a node's tip lists its checkpoint lines (`deadlines`, STORYUI1 B10). STORYUI1-U2 (B10):
//! a node holding jobs of opened boards wears the quest badge (widget `node.<qid>.job`) and its tip lists
//! "Job: <name>". CAMPAIGN1-TM: in a campaign world with travel modes the side panel's mode button
//! (`world_map.mode`) cycles the party's mode (`campaign::purse::set_travel_mode`); the trips, their tips and the road
//! labels count its days, and the trip ask runs the trip's legs in it (`runover::trip_ask`).
//! MAP-HOOKS1 pass A: the background is the current location's region's (`backgrounds/world_map/<region>.png`, up its
//! `parent` chain) before the world's; a layout sidecar beside it (`<name>.json`, `MapLayout`) places the landmarks over
//! `map_pos`; a location with `map/landmarks/<id>.png` shows it (`landmark_box`), else its kind pin.
//! MAP-HOOKS1 pass B (AO-01 (2), (4), (5), AO-02; Art/UI's preview `tools/mock_ui/previews/world_diamond.py`): the map
//! viewport and padding of the preview (`MAP`, `MAP_PAD`, the background covering it); a hovered or focused place
//! glows, a locked one is grey (`locked_look`); roads are focus only (`route_layers`): smooth lantern dots through the
//! sidecar's waypoints (`spline`), the hovered place's open roads dimmed, and with a destination picked every way
//! there (`travel::paths`): the chosen one gold, the others pale, previewed on hover and picked by a click on the map or
//! in the side panel, travelled by `travel::travel_via`; the game manifest's `world_map_lantern_flicker` (off in
//! Emberwake, never under reduce motion) flickers the dimmed roads (`flicker`). The HUD: the cyan party ring, the
//! pulsing quest badge (`pulse`), faint names above the landmarks lit on hover, focus and pick (`label_rect`), and the
//! floating Legend (`FloatPanel`: a free corner or the middle and unlocked, drag when unlocked, minimize to a named
//! button in place). Player text carries display names only.

use aegis_core::clock::Instant;
use aegis_ui::theme::th;
use aegis_ui::canvas::{rgb, rgba, Image, Rect, Rgba};
use crate::music;
use crate::hud::{hud_bar, hud_key, hud_open, purse_word, Hud, PurseWord};
use crate::screens::{show_errors, Cx, GameCx, Key, Nav, Screen};
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, H, SMALL, W};
use aegis_ui::widget_plumbing;
use aegis_core::json::{self, s};
use emberwake_world::{explore, progression, travel};
use emberwake_content::world_map::{self, Owner};
use aegis_ui::layout_editor::{LayoutAct, LayoutEdit, LayoutEditor, Marker, MarkerBox, Polyline, Viewport};
use aegis_ui::screen::PointerPhase;
use std::rc::Rc;
use serde_json::{json, Value};

/// MAP-HOOKS1: the map viewport, as Art/UI's signed-off preview draws it (`world_diamond.py`: 496 x 328 at (4, 24),
/// the side panel at 504).
const MAP: Rect = Rect::new(4, 24, 496, 328);
/// MAP-HOOKS1: the viewport's padding around the area `map_pos` 0..1 and the sidecar's positions span
/// (`world_diamond.py` `PAD_X`, `PAD_Y`), so a landmark at y 0.99 stands inside the map with its foot.
pub const MAP_PAD: (i32, i32) = (34, 22);

/// The area positions 0..1 span on the map: the viewport less its padding (x, y, w, h).
pub fn anchor_rect() -> (f32, f32, f32, f32) {
    ((MAP.x + MAP_PAD.0) as f32, (MAP.y + MAP_PAD.1) as f32, (MAP.w - 2 * MAP_PAD.0) as f32, (MAP.h - 2 * MAP_PAD.1) as f32)
}

#[derive(Debug, Clone)]
pub enum WAct {
    Node(String),
    Travel,
    Enter,
    Camp,
    /// CAMPAIGN1-TM: the next travel mode.
    Mode,
    Back,
    /// A HUD bar button (PLAYUI1-H; Menu is one of them).
    Hud(Hud),
    /// MAP-HOOKS1: makes way `i` to the destination the chosen one.
    Way(usize),
    /// MAP-HOOKS1: the Legend's unlock / lock button.
    LegendLock,
    /// MAP-HOOKS1: the Legend's minimize button.
    LegendMin,
    /// MAP-HOOKS1: the minimized Legend's button (a click without a drag opens it in place).
    LegendOpen,
    /// A panel that takes the clicks over it and does nothing with them (the Legend's body).
    Nothing,
    /// PR5-48: the debug layout editing switched on or off (F7, the side panel's Edit layout).
    LayoutEdit,
    /// PR5-48: an action of the layout editor while editing.
    Layout(LayoutAct),
}

/// The world map's controls (the Help button's hover card and popup).
const WORLD_HELP: [&str; 5] = ["Click a place to pick it, then Travel", "Pick another way on the map or in the panel", "Enter (button): go into the place the party is at", "P: party   J: journal   I: stash", "Esc: back to the region map"];

/// CAMPAIGN1 W5: the arrival of a trip of `time` in a campaign world: "Arrive on day 9, 34 days left." (the days left
/// of the first running visible countdown by HUD order at arrival; without one only the day); None outside one.
pub fn arrival(gs: &emberwake_world::GameState, time: i64) -> Option<String> {
    let c = gs.campaign.as_ref()?;
    let at = gs.time + time.max(0);
    let mut clocks: Vec<(i64, usize, i64)> = c
        .defs
        .iter()
        .enumerate()
        .filter_map(|(i, (id, def))| {
            let st = c.meters.get(id).filter(|st| def.countdown && st.running && st.visible)?;
            Some((json::i(json::dget(&def.row, "hud"), "order", 0), i, st.value(def, at)))
        })
        .collect();
    clocks.sort();
    // X2: day 1 at a new game (`campaign.day` is the clock plus one)
    let day = at + 1;
    Some(match clocks.first() {
        Some((.., left)) => format!("Arrive on day {day}, {} left.", crate::runover::days(*left)),
        None => format!("Arrive on day {day}."),
    })
}

/// A trip's length and upkeep for the tips: "2 time, 2 supplies" (today's), "2 days, 2 Tallow" in a campaign world.
fn trip_cost(cx: &Cx, gs: &emberwake_world::GameState, trip: &serde_json::Map<String, Value>) -> String {
    cost_text(cx, gs, json::i(trip, "time", 0), json::i(trip, "supplies", 0))
}

/// `time` and `supplies` as the tips say them ("2 days, 2 Tallow" in a campaign world).
fn cost_text(cx: &Cx, gs: &emberwake_world::GameState, time: i64, supplies: i64) -> String {
    let length = if gs.campaign.is_some() { crate::runover::days(time) } else { format!("{time} time") };
    format!("{length}, {supplies} {}", purse_word(cx, PurseWord::Upkeep))
}

/// CAMPAIGN1-TM: the mode button's tip: the mode's own tip, its days a leg and the road-fight change, and how to switch.
fn mode_tip(m: &emberwake_data::TravelMode, many: bool) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !m.tip.is_empty() {
        parts.push(m.tip.clone());
    }
    if m.days > 0 {
        parts.push(format!("Each leg takes {} more.", crate::runover::days(m.leg_days())));
    }
    if m.road_pct != 100 {
        parts.push(format!("Ambush chance x{}%.", m.road_pct));
    }
    if many {
        parts.push("Click for the next mode.".into());
    }
    parts.join(" ")
}

// ---- MAP-HOOKS1: the act background, its layout sidecar and the landmarks ---------------------------------------

/// A landmark's drawn side at scale 1, in logical px (AO-01: the 96 px cut shown at 48).
pub const LANDMARK_PX: f32 = 48.0;
/// How far below its ground point a landmark's box ends (logical px): a larger landmark grows upward.
pub const LANDMARK_FOOT: f32 = 8.0;
/// The kind pin's side (the landmark's fallback).
pub const PIN_PX: i32 = 14;

/// A landmark of a layout sidecar: its ground point (map fractions, not clamped, so an edge anchor such as 1.042
/// works) and its map scale (1 = `LANDMARK_PX`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Landmark {
    pub pos: (f32, f32),
    pub scale: f32,
}

/// A world map layout sidecar (`backgrounds/world_map/<name>.json`, `art::world_map_layout`): `{landmarks: {id: {pos:
/// [x, y], scale?}}, routes: {id: [[x, y], ...]}}`, keyed by local ids. A route's points are its waypoints between its
/// two endpoints (endpoints excluded), in the route's `from` to `to` order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MapLayout {
    pub landmarks: std::collections::BTreeMap<String, Landmark>,
    pub routes: std::collections::BTreeMap<String, Vec<(f32, f32)>>,
}

fn point(v: &Value) -> Option<(f32, f32)> {
    match v.as_array()?.as_slice() {
        [x, y] => Some((x.as_f64()? as f32, y.as_f64()? as f32)),
        _ => None,
    }
}

fn local_id(id: &str) -> &str {
    id.rsplit_once(':').map_or(id, |(_, l)| l)
}

impl MapLayout {
    /// None unless `v` is an object. A landmark without a two-number `pos` is skipped; a `scale` that is missing or not
    /// a positive number is 1. A route holding any point that is not two numbers is skipped.
    pub fn parse(v: &Value) -> Option<MapLayout> {
        let root = v.as_object()?;
        let mut out = MapLayout::default();
        for (id, e) in root.get("landmarks").and_then(Value::as_object).into_iter().flatten() {
            let Some(pos) = e.get("pos").and_then(point) else { continue };
            let scale = e.get("scale").and_then(Value::as_f64).filter(|s| *s > 0.0).unwrap_or(1.0) as f32;
            out.landmarks.insert(id.clone(), Landmark { pos, scale });
        }
        for (id, pts) in root.get("routes").and_then(Value::as_object).into_iter().flatten() {
            let Some(pts) = pts.as_array() else { continue };
            let parsed: Option<Vec<(f32, f32)>> = pts.iter().map(point).collect();
            if let Some(p) = parsed {
                out.routes.insert(id.clone(), p);
            }
        }
        Some(out)
    }

    /// The landmark of location `id` (a qid or a local id).
    pub fn landmark(&self, id: &str) -> Option<&Landmark> {
        self.landmarks.get(id).or_else(|| self.landmarks.get(local_id(id)))
    }

    /// The waypoints of route `id` (a qid or a local id).
    pub fn route(&self, id: &str) -> Option<&[(f32, f32)]> {
        self.routes.get(id).or_else(|| self.routes.get(local_id(id))).map(Vec::as_slice)
    }
}

/// The names the world map's background and layout are looked up by, in order: the current location's region, then its
/// parent regions (`parent`), then the world lineage child first.
pub fn map_names(d: &emberwake_world::Data, gs: &emberwake_world::GameState) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let loc = explore::resolve_location(d, gs, &gs.location_id);
    let mut world = json::s_or(loc, "_world", &gs.world_id);
    let mut region_ref = s(loc, "region");
    while !region_ref.is_empty() {
        let region = d.resolve(&world, "regions", &region_ref);
        let id = s(region, "id");
        if id.is_empty() || out.contains(&id) {
            break;
        }
        out.push(id);
        world = json::s_or(region, "_world", &world);
        region_ref = s(region, "parent");
    }
    for w in d.lineage(&gs.world_id).into_iter().rev() {
        if !out.contains(&w) {
            out.push(w);
        }
    }
    out
}

/// The layout of the first of `names` whose sidecar a layer holds and that reads as a layout; a missing file or a bad
/// one (not JSON, not an object) is passed over, and None leaves every node on its `map_pos`.
pub fn read_layout(layers: &aegis_core::assets::Layers, names: &[String]) -> Option<MapLayout> {
    names.iter().find_map(|n| aegis_core::assets::read_json(layers, &emberwake_content::art::world_map_layout(n)).and_then(|v| MapLayout::parse(&v)))
}

/// A node's ground point on map rect `rect` (x, y, w, h) and its scale: the layout's landmark (not clamped), else a
/// two-number `map_pos` clamped to 0..1, else slot `i` of `n` on a circle (graph node order), at scale 1.
pub fn place(layout: Option<&MapLayout>, qid: &str, map_pos: &[Value], i: usize, n: usize, rect: (f32, f32, f32, f32)) -> ((f32, f32), f32) {
    let at = |fx: f32, fy: f32| (rect.0 + fx * rect.2, rect.1 + fy * rect.3);
    if let Some(l) = layout.and_then(|l| l.landmark(qid)) {
        return (at(l.pos.0, l.pos.1), l.scale);
    }
    let pos = match map_pos {
        [x, y] if x.is_number() && y.is_number() => at(json::to_float_or(x, 0.0).clamp(0.0, 1.0) as f32, json::to_float_or(y, 0.0).clamp(0.0, 1.0) as f32),
        _ => {
            let angle = std::f32::consts::TAU * i as f32 / n.max(1) as f32 - std::f32::consts::FRAC_PI_2;
            let radius = rect.2.min(rect.3) * 0.38;
            (rect.0 + rect.2 / 2.0 + angle.cos() * radius, rect.1 + rect.3 / 2.0 + angle.sin() * radius)
        }
    };
    (pos, 1.0)
}

/// The box a landmark at ground point (`x`, `y`) and `scale` is drawn in: `LANDMARK_PX * scale` square, centred on `x`,
/// its bottom `LANDMARK_FOOT` below `y`, so it grows upward.
pub fn landmark_box(x: f32, y: f32, scale: f32) -> Rect {
    let side = LANDMARK_PX * scale;
    Rect::new((x - side / 2.0).round() as i32, (y + LANDMARK_FOOT - side).round() as i32, side.round() as i32, side.round() as i32)
}

/// The kind pin's box (the fallback when the location has no landmark): `PIN_PX` square centred on the ground point.
pub fn pin_box(x: f32, y: f32) -> Rect {
    Rect::new(x as i32 - PIN_PX / 2, y as i32 - PIN_PX / 2, PIN_PX, PIN_PX)
}

/// The rect an `iw` x `ih` picture covers `r` with, centred (the preview's `object-fit: cover`): the scale that fills
/// both sides, the overflow cropped evenly by the caller's clip.
pub fn cover_rect(iw: i32, ih: i32, r: Rect) -> Rect {
    if iw <= 0 || ih <= 0 {
        return r;
    }
    let k = (r.w as f32 / iw as f32).max(r.h as f32 / ih as f32);
    let (w, h) = ((iw as f32 * k).round() as i32, (ih as f32 * k).round() as i32);
    Rect::new(r.x + (r.w - w) / 2, r.y + (r.h - h) / 2, w, h)
}

// ---- MAP-HOOKS1 pass B: roads, labels, badges and the Legend -------------------------------------------------------

/// A smooth line through `pts` (the preview's `smoothPath`: Catmull-Rom as cubic Béziers, end points repeated),
/// sampled every few px; two points give the straight segment, fewer give nothing to draw.
pub fn spline(pts: &[(f32, f32)]) -> Vec<(f32, f32)> {
    if pts.len() < 3 {
        return if pts.len() == 2 { pts.to_vec() } else { Vec::new() };
    }
    let mut out = vec![pts[0]];
    for i in 0..pts.len() - 1 {
        let (p0, p1, p2, p3) = (pts[i.saturating_sub(1)], pts[i], pts[i + 1], pts[(i + 2).min(pts.len() - 1)]);
        let c1 = (p1.0 + (p2.0 - p0.0) / 6.0, p1.1 + (p2.1 - p0.1) / 6.0);
        let c2 = (p2.0 - (p3.0 - p1.0) / 6.0, p2.1 - (p3.1 - p1.1) / 6.0);
        let len = ((p2.0 - p1.0).powi(2) + (p2.1 - p1.1).powi(2)).sqrt();
        let steps = (len / 2.0).ceil().max(1.0) as usize;
        for k in 1..=steps {
            let t = k as f32 / steps as f32;
            let u = 1.0 - t;
            let (a, b, c, e) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            out.push((a * p1.0 + b * c1.0 + c * c2.0 + e * p2.0, a * p1.1 + b * c1.1 + c * c2.1 + e * p2.1));
        }
    }
    out
}

/// Points along the polyline `line` every `step` px of its length, from its start (the lantern dots).
pub fn dots(line: &[(f32, f32)], step: f32) -> Vec<(f32, f32)> {
    let mut out = Vec::new();
    let Some(&first) = line.first() else { return out };
    out.push(first);
    let mut carry = 0.0f32;
    for w in line.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        let mut at = step - carry;
        while at <= len {
            let t = at / len;
            out.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
            at += step;
        }
        carry = len - (at - step);
    }
    out
}

/// How a road is drawn (AO-01 (4)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteStyle {
    /// An open road of the hovered or focused place, dimmed.
    Near,
    /// Another way to the destination, pale.
    Alt,
    /// The other way the pointer is over, lit (its preview).
    Peek,
    /// The chosen way to the destination, gold.
    Main,
}

impl RouteStyle {
    pub fn name(self) -> &'static str {
        match self {
            RouteStyle::Near => "near",
            RouteStyle::Alt => "alt",
            RouteStyle::Peek => "peek",
            RouteStyle::Main => "main",
        }
    }
}

/// MAP-HOOKS1 (AO-01 (4), focus only): the roads the map draws, each once, in draw order. `routes` are the view's
/// (qid, from, to, open); `near` the hovered or focused place; `ways` the ways to the destination (route qids each, as
/// `travel::paths` ranks them), `pick` the chosen one and `peek` the one previewed. Nothing at rest; the open roads of
/// `near` dimmed (`Near`), never a locked one; then each other way's roads pale (`Alt`; `Peek` for the previewed one),
/// then the chosen way's (`Main`) last, on top. A road of the chosen way is drawn only as `Main`, a road shared by
/// other ways once (the previewed way first), and a road of any way never as `Near`.
pub fn route_layers(routes: &[(String, String, String, bool)], near: Option<&str>, ways: &[Vec<String>], pick: usize, peek: Option<usize>) -> Vec<(String, RouteStyle)> {
    let main: &[String] = ways.get(pick).map_or(&[], Vec::as_slice);
    let in_ways = |rq: &str| ways.iter().any(|w| w.iter().any(|r| r == rq));
    let mut out: Vec<(String, RouteStyle)> = Vec::new();
    if let Some(n) = near {
        for (rq, from, to, open) in routes {
            if *open && (from == n || to == n) && !in_ways(rq) {
                out.push((rq.clone(), RouteStyle::Near));
            }
        }
    }
    let mut others: Vec<usize> = (0..ways.len()).filter(|i| *i != pick).collect();
    // the previewed way first, so a road it shares with another pale way is drawn lit
    others.sort_by_key(|i| Some(*i) != peek);
    let mut alts: Vec<(String, RouteStyle)> = Vec::new();
    for i in others {
        let style = if Some(i) == peek { RouteStyle::Peek } else { RouteStyle::Alt };
        for rq in &ways[i] {
            if !main.contains(rq) && !alts.iter().any(|(r, _)| r == rq) {
                alts.push((rq.clone(), style));
            }
        }
    }
    // pale under lit: the plain alternatives first
    alts.sort_by_key(|(_, st)| *st == RouteStyle::Peek);
    out.extend(alts);
    out.extend(main.iter().map(|rq| (rq.clone(), RouteStyle::Main)));
    out
}

/// The quest badge's glow period (the preview's `wdQuest`, 1.8 s).
pub const PULSE_MS: u64 = 1800;

/// The quest badge's glow, 0..1, at `ms`: a smooth pulse over `PULSE_MS`, held at its middle under reduce motion.
pub fn pulse(ms: u64, reduce_motion: bool) -> f32 {
    if reduce_motion {
        return 0.5;
    }
    let t = (ms % PULSE_MS) as f32 / PULSE_MS as f32;
    0.5 - 0.5 * (t * std::f32::consts::TAU).cos()
}

/// The lantern flicker's period (the preview's `wdFlick`, 3.1 s) and how far apart two roads run in it (0.73 s).
pub const FLICKER_MS: u64 = 3100;
pub const FLICKER_STAGGER_MS: u64 = 730;

/// MAP-HOOKS1: the lantern flicker's brightness of road `k` at `ms`, 0..1 (the preview's keyframes: 1, 0.55 at 35 %,
/// 0.85 at 52 %, 0.6 at 70 %, 1 again; road `k` runs `k` staggers ahead): 1 when the game's
/// `world_map_lantern_flicker` is off (`on`) or under reduce motion.
pub fn flicker(ms: u64, k: usize, on: bool, reduce_motion: bool) -> f32 {
    if !on || reduce_motion {
        return 1.0;
    }
    let t = ((ms + k as u64 * FLICKER_STAGGER_MS) % FLICKER_MS) as f32 / FLICKER_MS as f32;
    const KEYS: [(f32, f32); 5] = [(0.0, 1.0), (0.35, 0.55), (0.52, 0.85), (0.70, 0.6), (1.0, 1.0)];
    let i = KEYS.windows(2).position(|w| t <= w[1].0).unwrap_or(3);
    let ((t0, a), (t1, b)) = (KEYS[i], KEYS[i + 1]);
    a + (b - a) * ((t - t0) / (t1 - t0)).clamp(0.0, 1.0)
}

/// AO-01 (2): a locked place's mark: grey (85 % greyscale) at 85 % brightness and 80 % opacity, grey but not dark (a
/// half-bright grey vanished on a dark map).
pub fn locked_look(img: &Image) -> Image {
    let mut out = Image { w: img.w, h: img.h, px: img.px.clone() };
    for p in out.px.chunks_exact_mut(4) {
        let lum = 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
        for c in p.iter_mut().take(3) {
            *c = ((*c as f32 * 0.15 + lum * 0.85) * 0.85).round().clamp(0.0, 255.0) as u8;
        }
        p[3] = (p[3] as f32 * 0.8).round() as u8;
    }
    out
}

/// MAP-HOOKS1 (AO-01 (5)): where a place's name strip goes: `w` x `h`, centred on `cx`, its bottom at `bottom` (above
/// the mark), kept inside `map` side to side; a strip that would leave the map's top is pinned to the top edge, still
/// over the mark, never moved below or beside it.
pub fn label_rect(cx: f32, bottom: i32, w: i32, h: i32, map: Rect) -> Rect {
    let x = ((cx - w as f32 / 2.0).round() as i32).clamp(map.x + 1, (map.x + map.w - w - 1).max(map.x + 1));
    Rect::new(x, (bottom - h).max(map.y + 1), w, h)
}

/// Whether two rects share any pixel.
fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// The minimize button's side and the Legend's margin from the map edge.
pub const PANEL_BTN: i32 = 10;
const PANEL_MARGIN: i32 = 4;

/// MAP-HOOKS1 (`preferences/ui.md` "Floating panels", AO-02): a floating panel over an area. It starts in a free corner
/// (top right, top left, bottom right, bottom left), or in the middle and unlocked when every corner is taken;
/// unlocked, it drags anywhere inside the area; minimized, it collapses into a named button whose top right is where the
/// minimize button's top right was, so minimize and restore toggle in place; that button drags too while unlocked, the
/// panel's place moving with it. Nothing here names what the panel holds.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FloatPanel {
    /// The open panel's top left; None until placed.
    pub pos: Option<(i32, i32)>,
    pub locked: bool,
    pub minimized: bool,
    /// The pointer's offset from the dragged rect's top left while a drag runs.
    grab: Option<(i32, i32)>,
    /// The pointer's last place over the panel or its button before a drag (the press a drag starts from).
    over: Option<(i32, i32)>,
}

impl FloatPanel {
    /// Places a panel of `size` in `area` once (later calls keep its place): the first corner whose rect overlaps none
    /// of `taken`, locked; else the middle, unlocked.
    pub fn place(&mut self, area: Rect, size: (i32, i32), taken: &[Rect]) {
        if self.pos.is_some() {
            return;
        }
        let (w, h) = size;
        let (l, r) = (area.x + PANEL_MARGIN, area.x + area.w - w - PANEL_MARGIN);
        let (t, b) = (area.y + PANEL_MARGIN, area.y + area.h - h - PANEL_MARGIN);
        match [(r, t), (l, t), (r, b), (l, b)].into_iter().find(|&(x, y)| !taken.iter().any(|o| overlaps(Rect::new(x, y, w, h), *o))) {
            Some(at) => {
                self.pos = Some(at);
                self.locked = true;
            }
            None => {
                self.pos = Some((area.x + (area.w - w) / 2, area.y + (area.h - h) / 2));
                self.locked = false;
            }
        }
    }

    /// The open panel's rect at `size` (at the area's origin before it is placed).
    pub fn rect(&self, size: (i32, i32)) -> Rect {
        let (x, y) = self.pos.unwrap_or((0, 0));
        Rect::new(x, y, size.0, size.1)
    }

    /// The minimize button of the open panel `panel`: its top-right corner.
    pub fn min_button(panel: Rect) -> Rect {
        Rect::new(panel.x + panel.w - PANEL_BTN - 3, panel.y + 3, PANEL_BTN, PANEL_BTN - 1)
    }

    /// The unlock button, left of the minimize button.
    pub fn lock_button(panel: Rect) -> Rect {
        let m = Self::min_button(panel);
        Rect::new(m.x - PANEL_BTN - 2, m.y, PANEL_BTN, m.h)
    }

    /// The minimized panel's `chip` w x h button: its top right where the minimize button's top right is.
    pub fn chip_rect(&self, size: (i32, i32), chip: (i32, i32)) -> Rect {
        let m = Self::min_button(self.rect(size));
        Rect::new(m.x + m.w - chip.0, m.y - 1, chip.0, chip.1)
    }

    /// The rect shown now: the chip when minimized, else the panel.
    pub fn shown(&self, size: (i32, i32), chip: (i32, i32)) -> Rect {
        if self.minimized { self.chip_rect(size, chip) } else { self.rect(size) }
    }

    pub fn toggle_lock(&mut self) {
        self.locked = !self.locked;
    }

    pub fn minimize(&mut self) {
        self.minimized = true;
    }

    /// Opens the panel where it was (its minimize button where the chip's top right was), nudged inside `area`.
    pub fn restore(&mut self, area: Rect, size: (i32, i32)) {
        self.minimized = false;
        self.clamp(area, size);
    }

    /// Keeps the open panel inside `area`.
    pub fn clamp(&mut self, area: Rect, size: (i32, i32)) {
        if let Some((x, y)) = self.pos {
            self.pos = Some((x.clamp(area.x + 2, (area.x + area.w - size.0 - 2).max(area.x + 2)), y.clamp(area.y + 2, (area.y + area.h - size.1 - 2).max(area.y + 2))));
        }
    }

    /// One frame of the drag: `held` is whether the pointer drags the panel (or its chip) now, `pointer` where it is and
    /// `now` the rect it drags (`shown`). Locked, nothing moves. A drag keeps the pointer where it took hold (the last
    /// place it was over `now` before the drag) and `now` inside `area`; the panel's place moves with it.
    pub fn follow(&mut self, pointer: (i32, i32), held: bool, now: Rect, area: Rect) {
        if !held || self.locked {
            self.grab = None;
            if now.contains(pointer.0, pointer.1) {
                self.over = Some(pointer);
            }
            return;
        }
        let grab = *self.grab.get_or_insert_with(|| {
            let at = self.over.filter(|o| now.contains(o.0, o.1)).unwrap_or(pointer);
            ((at.0 - now.x).clamp(0, now.w.max(1) - 1), (at.1 - now.y).clamp(0, now.h.max(1) - 1))
        });
        let x = (pointer.0 - grab.0).clamp(area.x + 2, (area.x + area.w - now.w - 2).max(area.x + 2));
        let y = (pointer.1 - grab.1).clamp(area.y + 2, (area.y + area.h - now.h - 2).max(area.y + 2));
        let (px, py) = self.pos.unwrap_or((now.x, now.y));
        self.pos = Some((px + x - now.x, py + y - now.y));
    }
}

/// The drag payload of the Legend and its minimized button (`Ui::draggable`).
pub const LEGEND_DRAG: &str = "world_map.legend";

/// One place's drawing on the map this frame.
struct Mark {
    qid: String,
    name: String,
    x: f32,
    y: f32,
    /// The landmark's box or the pin's.
    r: Rect,
    landmark: bool,
    is_here: bool,
    selected: bool,
    lit: bool,
    locked: bool,
    badge: Option<Rect>,
    label: Rect,
}

/// The cached layout: the names it was read for, the art generation, and the layout read.
type LayoutCache = (Vec<String>, u64, Option<MapLayout>);

/// The widget id of node `qid`.
fn node_id(qid: &str) -> String {
    format!("node.{}", qid.replace(':', "."))
}

#[derive(Default)]
pub struct WorldMap {
    w: Vec<Widget<WAct>>,
    selected: String,
    /// MAP-HOOKS1: the layout sidecar, read once per name chain and art generation.
    layout: Option<LayoutCache>,
    /// MAP-HOOKS1: the chosen way to the destination (an index into `travel::paths`; 0, the cheapest, by default).
    pick: usize,
    /// MAP-HOOKS1: the node marks of the last frame (mark rect, qid), for hover.
    hits: Vec<(Rect, String)>,
    /// MAP-HOOKS1: the floating Legend and its open size in the last frame.
    pub legend: FloatPanel,
    legend_size: (i32, i32),
    /// MAP-HOOKS1: the roads drawn in the last frame and how.
    shown: Vec<(String, RouteStyle)>,
    /// MAP-HOOKS1: the clock the badge pulse and the flicker run on.
    born: Option<Instant>,
    /// PR5-48: the layout being edited in place (a debug build with the dev tools, `crate::DEV_KEYS`).
    edit: Option<LayoutEditing>,
}

impl WorldMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// MAP-HOOKS1: the layout sidecar for `names` (`read_layout`), read again only when the names or the art change.
    fn layout_for(&mut self, cx: &Cx, names: &[String]) -> Option<MapLayout> {
        let generation = cx.art.generation();
        if !self.layout.as_ref().is_some_and(|(n, g, _)| n.as_slice() == names && *g == generation) {
            self.layout = Some((names.to_vec(), generation, read_layout(cx.art.layers(), names)));
        }
        self.layout.as_ref().and_then(|c| c.2.clone())
    }

    /// The topmost widget of the last frame under `mouse` (none when not live).
    fn under(&self, mouse: (i32, i32), live: bool) -> Option<&Widget<WAct>> {
        if !live {
            return None;
        }
        self.w.iter().rev().find(|w| w.rect.contains(mouse.0, mouse.1))
    }

    /// The place the pointer was over in the last frame (its mark the topmost widget there).
    fn hovered_node(&self, mouse: (i32, i32), live: bool) -> Option<String> {
        let top = self.under(mouse, live)?;
        if top.kind != "node" {
            return None;
        }
        self.hits.iter().rev().find(|(r, _)| *r == top.rect).map(|(_, q)| q.clone())
    }

    /// The other way the pointer was over in the last frame (its road on the map or its row in the side panel).
    fn peeked(&self, mouse: (i32, i32), live: bool) -> Option<usize> {
        let top = self.under(mouse, live)?;
        let rest = top.id.strip_prefix("world_map.way.")?;
        rest.split('.').next()?.parse().ok()
    }

    /// The ways from the party to the destination (`travel::paths`; none without a destination).
    fn ways(&self, cx: &Cx) -> Vec<travel::TravelPath> {
        if self.selected.is_empty() {
            return Vec::new();
        }
        travel::paths(&cx.data, cx.gs(), &cx.gs().location_id, &self.selected)
    }

    /// The chosen way's routes (none without a way: `travel` plans its own).
    fn chosen(&self, cx: &Cx) -> Option<Vec<String>> {
        let ways = self.ways(cx);
        ways.get(self.pick).or(ways.first()).map(|w| w.routes.clone())
    }

    fn ms(&mut self) -> u64 {
        self.born.get_or_insert_with(Instant::now).elapsed().as_millis() as u64
    }

    fn act(&mut self, cx: &mut Cx, a: WAct) {
        let d = cx.data.clone();
        match a {
            WAct::Node(q) => {
                if q == cx.gs().location_id {
                    self.selected.clear();
                } else if q != self.selected {
                    self.selected = q;
                    self.pick = 0;
                }
            }
            WAct::Way(i) => self.pick = i,
            WAct::Travel => {
                if self.selected.is_empty() {
                    return;
                }
                let ask = match self.chosen(cx) {
                    Some(routes) => crate::runover::trip_ask_via(cx, &routes),
                    None => crate::runover::trip_ask(cx, &self.selected),
                };
                match ask {
                    Some(q) => cx.ask_tag_no(q, crate::runover::ASK_TRIP),
                    None => self.travel(cx),
                }
            }
            WAct::Enter => {
                let here = cx.gs().location_id.clone();
                let res = explore::enter(&d, cx.gs_mut(), &here, explore::SPAWN);
                if !json::b(&res, "ok", false) {
                    show_errors(cx, json::sget(&res, "errors"));
                    return;
                }
                crate::region::log_events(cx, json::aget(&res, "events"));
                crate::runover::take(cx, &res);
                cx.nav.push(Nav::Pop);
            }
            WAct::Camp => {
                // a refused rest says why at once; an allowed one whose days would end the run asks first
                let days = emberwake_world::campaign::purse::rest_rule(&d, &cx.gs().world_id).days as i64;
                let ask = if progression::can_camp(&d, cx.gs()).is_empty() { crate::runover::days_ask(cx, days, "Resting", "Rest") } else { None };
                match ask {
                    Some(q) => cx.ask_tag_no(q, crate::runover::ASK_REST),
                    None => self.camp(cx),
                }
            }
            WAct::Mode => self.next_mode(cx),
            WAct::Back => cx.nav.push(Nav::Pop),
            WAct::Hud(h) => hud_open(cx, h, &WORLD_HELP),
            WAct::LegendLock => self.legend.toggle_lock(),
            WAct::LegendMin => self.legend.minimize(),
            WAct::LegendOpen => self.legend.restore(MAP, self.legend_size),
            WAct::Nothing => {}
            WAct::LayoutEdit => self.toggle_edit(cx),
            WAct::Layout(a) => {
                if let Some(e) = self.edit.as_mut() {
                    e.act(cx, a);
                }
            }
        }
    }

    /// PR5-48: switches the in-place layout editing on (a debug build with the dev tools only, `crate::DEV_KEYS`) or off.
    /// It edits the sidecar the map reads (the first of `map_names` with one, else the current region's or world's
    /// name): the map draws by the editor's layout with its real labels, badges and Legend, the editor's handles over
    /// it and its controls in the side panel. Switched off, what was not saved is dropped (the log says so).
    fn toggle_edit(&mut self, cx: &mut Cx) {
        if let Some(e) = self.edit.take() {
            if e.editor.dirty() {
                cx.log(format!("Layout edits to {} not saved were dropped.", e.name));
            }
            return;
        }
        if !crate::DEV_KEYS {
            return;
        }
        let names = map_names(&cx.data, cx.gs());
        let Some(name) = names.iter().find(|n| cx.art.has(&emberwake_content::art::world_map_layout(n))).or(names.first()).cloned() else { return };
        let mut e = LayoutEditing::open(cx, &name, false);
        e.editor.backdrop = false;
        e.editor.grid = false;
        e.editor.labels = false;
        self.edit = Some(e);
    }

    /// CAMPAIGN1-TM: picks the world's next travel mode (data order, round) and logs it.
    fn next_mode(&mut self, cx: &mut Cx) {
        let d = cx.data.clone();
        let modes = emberwake_world::campaign::purse::travel_modes(&d, &cx.gs().world_id);
        let Some(now) = emberwake_world::campaign::purse::travel_mode(&d, cx.gs()) else { return };
        let at = modes.modes.iter().position(|m| m.id == now.id).unwrap_or(0);
        let next = modes.modes[(at + 1) % modes.modes.len()].clone();
        let res = emberwake_world::campaign::purse::set_travel_mode(&d, cx.gs_mut(), &next.id);
        if !json::b(&res, "ok", false) {
            show_errors(cx, json::sget(&res, "errors"));
            return;
        }
        cx.log(format!("Travel mode: {}.", next.name));
    }

    /// The trip to the selected place along the chosen way (`travel::travel_via`; TravelRunner.travel when there is no
    /// way to choose): its log lines and campaign events; a road fight starts the battle (with the notice when it could
    /// end the run, W3); an arrival or a trip the run ended on (status `over`) clears the pick.
    fn travel(&mut self, cx: &mut Cx) {
        let d = cx.data.clone();
        let to = self.selected.clone();
        let res = match self.chosen(cx) {
            Some(routes) => travel::travel_via(&d, cx.gs_mut(), &to, &routes),
            None => travel::travel(&d, cx.gs_mut(), &to),
        };
        if !json::b(&res, "ok", false) {
            show_errors(cx, json::sget(&res, "errors"));
            return;
        }
        crate::region::log_events(cx, json::aget(&res, "events"));
        crate::runover::take(cx, &res);
        if json::i(&res, "starved_legs", 0) > 0 {
            // CAMPAIGN1 P1, P3: the upkeep's name; the roads are only more dangerous when zero upkeep raises the chance
            let word = purse_word(cx, PurseWord::Upkeep);
            if emberwake_world::campaign::purse::purse(&d, &cx.gs().world_id).road_pct > 100 {
                cx.log(format!("Out of {word}: the roads were more dangerous."));
            } else {
                cx.log(format!("Out of {word}."));
            }
        }
        if s(&res, "status") == "fight" {
            let fight = json::dget(&res, "fight").clone();
            let name = json::s_or(d.resolve(&cx.gs().world_id, "encounters", &s(&fight, "encounter")), "name", "Enemies");
            cx.log(format!("Ambushed on the road: {name}!"));
            crate::region::begin_fight(cx, &fight, "travel");
            crate::runover::warn_fight(cx, &s(&fight, "encounter"));
        } else {
            self.selected.clear();
            self.pick = 0;
        }
    }

    /// The side panel: the time, the travel mode, the pick and its ways, Enter, Make camp and the log (above the debug
    /// layout button's row when the build has one).
    #[allow(clippy::too_many_arguments)]
    fn side_panel(&mut self, ui: &mut Ui<WAct>, cx: &mut Cx, d: &emberwake_world::Data, gs: &emberwake_world::GameState, here: &str, ways: &[travel::TravelPath], peek: Option<usize>) {
        let sx = 504;
        let mut y = 28;
        ui.p.text(sx as f32, y as f32, "World map", BODY, th().info);
        ui.p.text_right(W as f32 - 6.0, y as f32, &format!("Time {}", gs.time), BODY, th().text);
        y += 14;
        // CAMPAIGN1-TM: the travel mode button (only in a campaign world with modes)
        let mode = emberwake_world::campaign::purse::travel_mode(&d, &gs);
        if let Some(m) = &mode {
            let many = emberwake_world::campaign::purse::travel_modes(&d, &gs.world_id).modes.len() > 1;
            ui.button_tip("world_map.mode", Rect::new(sx, y, 130, 16), &format!("Travel: {}", m.name), WAct::Mode, many, tip("Travel mode", &mode_tip(m, many)));
            y += 20;
        }
        if !self.selected.is_empty() {
            let name = cx.location_name(&gs.world_id, &self.selected);
            match ways.get(self.pick) {
                Some(way) => {
                    // CAMPAIGN1 W5: the arrival day and the days left in a campaign world; MAP-HOOKS1: of the chosen way
                    let mut text = format!("{}. Roads can be ambushed.", cost_text(cx, &gs, way.days, way.supplies));
                    if let Some(m) = &mode {
                        text = format!("{}: {text}", m.name);
                    }
                    if let Some(at) = arrival(&gs, way.days) {
                        text = format!("{text} {at}");
                    }
                    ui.button_tip("world_map.travel", Rect::new(sx, y, 130, 16), &format!("Travel to {name}"), WAct::Travel, true, tip("Travel", &text));
                    y += 19;
                    if ways.len() > 1 {
                        ui.p.text(sx as f32, y as f32, "Pick a way:", SMALL, th().dim);
                        y += 8;
                    }
                    // each way by what sets it apart, with its days and upkeep; hover previews it, a click picks it
                    let line = ui.p.line_h(SMALL).ceil() as i32;
                    for (i, w) in ways.iter().enumerate() {
                        let r = Rect::new(sx, y, 130, 2 * line + 3);
                        let on = i == self.pick;
                        let hot = peek == Some(i) || ui.hover(r);
                        ui.p.panel(r, if on { rgba(44, 34, 30, 180) } else { rgba(10, 11, 16, 120) }, if on { rgb(217, 119, 6) } else if hot { rgb(180, 83, 9) } else { th().edge }, 0.0);
                        ui.p.text_fit(sx as f32 + 3.0, y as f32 + 1.0, &w.label, SMALL, 124.0, if on { th().gold } else if hot { rgb(253, 230, 138) } else { th().text });
                        ui.p.text_fit(sx as f32 + 3.0, (y + line + 1) as f32, &cost_text(cx, &gs, w.days, w.supplies), SMALL, 124.0, th().dim);
                        let t = tip(&w.label, &format!("{}. Click to take this way.", cost_text(cx, &gs, w.days, w.supplies)));
                        ui.push(format!("world_map.way.{i}"), "row", w.label.clone(), r, true, on, Some(WAct::Way(i)), t);
                        y += r.h + 2;
                    }
                    y += 2;
                }
                None => {
                    ui.p.text_fit(sx as f32, y as f32 + 3.0, &format!("No open road to {name}."), SMALL, 130.0, th().foe);
                    y += 20;
                }
            }
        }
        ui.button("world_map.enter", Rect::new(sx, y, 130, 16), &format!("Enter {}", cx.location_name(&gs.world_id, &here)), WAct::Enter, !travel::current_node(&d, &gs).is_empty());
        y += 20;
        // CAMPAIGN1 P1, R1: the rest's cost in the upkeep's name ("Make camp (3 supplies)" in the Ashlands) and its days
        let camp = progression::can_camp(&d, &gs);
        let rule = emberwake_world::campaign::purse::rest_rule(&d, &gs.world_id);
        let camp_label = format!("Make camp ({} {})", rule.cost, purse_word(cx, PurseWord::Upkeep));
        let camp_tip = match (camp.is_empty(), rule.days) {
            (false, _) => camp.to_string(),
            (true, n) if n > 0 => format!("Rest every hero to full HP. Takes {}.", crate::runover::days(n as i64)),
            _ => "Rest every hero to full HP.".to_string(),
        };
        ui.button_tip("world_map.camp", Rect::new(sx, y, 130, 16), &camp_label, WAct::Camp, camp.is_empty(), tip("Make camp", &camp_tip));
        y += 24;
        let lines: Vec<String> = cx.log.iter().rev().take(16).cloned().collect();
        for l in lines.iter().rev() {
            for part in ui.p.wrap(l, SMALL, 130.0) {
                if y > if crate::DEV_KEYS { H - 30 } else { H - 10 } {
                    break;
                }
                ui.p.text(sx as f32, y as f32, &part, SMALL, th().text);
                y += 8;
            }
        }
    }

    /// Makes camp (HealRules.make_camp; a campaign world's rest rule, CAMPAIGN1 R2): its log lines and campaign events.
    fn camp(&mut self, cx: &mut Cx) {
        let d = cx.data.clone();
        let res = progression::make_camp(&d, cx.gs_mut());
        if !json::b(&res, "ok", false) {
            show_errors(cx, json::sget(&res, "errors"));
            return;
        }
        crate::region::log_events(cx, json::aget(&res, "events"));
        crate::runover::take(cx, &res);
    }
}

/// The act background, covering the map viewport (the preview's `object-fit: cover`), at the size it is drawn.
fn draw_background(ui: &mut Ui<WAct>, cx: &mut Cx, backgrounds: &[String]) {
    let Some(rel) = cx.art.first_rel(backgrounds) else {
        ui.p.fill(MAP, rgb(30, 30, 36));
        if let Some(first) = backgrounds.first() {
            // the missing-file report of the first name, as before
            let _ = cx.art.first(std::slice::from_ref(first), 1, 1);
        }
        return;
    };
    let (iw, ih) = cx.art.size_of(&rel).unwrap_or((MAP.w, MAP.h));
    let r = cover_rect(iw, ih, MAP);
    let s = ui.p.v.s;
    let (pw, ph) = (((r.w as f32) * s).round().max(1.0) as i32, ((r.h as f32) * s).round().max(1.0) as i32);
    match cx.art.image_fit(&rel, pw, ph) {
        Some(img) => {
            ui.p.clip(Some(MAP));
            ui.p.image(img, r);
            ui.p.clip(None);
        }
        None => ui.p.fill(MAP, rgb(30, 30, 36)),
    }
}

/// One road as lantern dots in its style (`alpha` the flicker's brightness).
fn draw_road(ui: &mut Ui<WAct>, line: &[(f32, f32)], style: RouteStyle, alpha: f32) {
    let a = |v: f32| (v * alpha).round().clamp(0.0, 255.0) as u8;
    match style {
        RouteStyle::Near => {
            for (x, y) in dots(line, 4.2) {
                ui.p.ellipse(x, y, 0.8, 0.8, rgba(251, 191, 36, a(110.0)));
            }
        }
        RouteStyle::Alt | RouteStyle::Peek => {
            let col = if style == RouteStyle::Peek { rgba(253, 230, 138, 255) } else { rgba(217, 203, 180, 150) };
            for (x, y) in dots(line, 3.6) {
                ui.p.ellipse(x, y, 0.9, 0.9, col);
            }
        }
        RouteStyle::Main => {
            for (x, y) in dots(line, 3.6) {
                ui.p.ellipse(x, y, 1.9, 1.9, rgba(245, 158, 11, 70));
                ui.p.ellipse(x, y, 1.2, 1.2, rgba(251, 191, 36, 255));
            }
        }
    }
}

/// A small padlock in `r`, its shackle open (lifted to the right) when `open`.
fn padlock(ui: &mut Ui<WAct>, r: Rect, open: bool, col: Rgba) {
    let (cx, cy) = (r.x as f32 + r.w as f32 / 2.0, r.y as f32 + r.h as f32 / 2.0);
    ui.p.fill(Rect::new(cx as i32 - 3, cy as i32, 6, 4), col);
    let (l, rr, top) = (cx - 2.0, cx + 1.5, cy - 3.0);
    let lift = if open { 1.5 } else { 0.0 };
    ui.p.line(l, cy - lift, l, top - lift, col, 0.0);
    ui.p.line(l, top - lift, rr, top - lift, col, 0.0);
    ui.p.line(rr, top - lift, rr, if open { top + 0.5 } else { cy }, col, 0.0);
}

/// The cyan of the party's ring.
const PARTY: Rgba = [34, 211, 238, 255];

impl Screen<GameCx> for WorldMap {
    fn id(&self) -> &'static str {
        "world_map"
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        let ms = self.ms();
        let reduce = cx.settings.reduce_motion();
        let hovered = self.hovered_node(mouse, live);
        let peek = self.peeked(mouse, live);
        let mut ui: Ui<WAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().game("worldmap_ground", rgb(12, 16, 24)));
        // MAP-HOOKS1: the current location's region chain, then the world lineage child first
        let names = map_names(&d, &gs);
        let backgrounds: Vec<String> = names.iter().map(|n| emberwake_content::art::world_map_background(n)).collect();
        // PR5-48: while editing, the map draws by the editor's layout
        let layout = match self.edit.as_ref() {
            Some(e) => Some(e.layout()),
            None => self.layout_for(cx, &names),
        };
        draw_background(&mut ui, cx, &backgrounds);
        let view = travel::view(&d, &gs);
        // WorldMapView.layout: a layout sidecar's landmark places the node (MAP-HOOKS1, not clamped), else a valid
        // map_pos (0..1) in the padded map area; any other node gets a fixed slot on a circle (graph node order), so
        // pins keep their place as the fog lifts.
        let rect = anchor_rect();
        let all = travel::nodes(&d, &gs.world_id);
        let placed: std::collections::HashMap<String, ((f32, f32), f32)> = all
            .iter()
            .enumerate()
            .map(|(i, q)| (q.clone(), place(layout.as_ref(), q, json::aget(explore::resolve_location(&d, &gs, q), "map_pos"), i, all.len(), rect)))
            .collect();
        let pos_of = |q: &str| -> (f32, f32) { placed.get(q).map(|p| p.0).unwrap_or((rect.0 + rect.2 / 2.0, rect.1 + rect.3 / 2.0)) };
        let nodes: Vec<serde_json::Map<String, Value>> = json::aget(&view, "nodes").iter().map(|n| json::dict(n).clone()).collect();
        let here = gs.location_id.clone();
        // -- the roads (AO-01 (4), focus only): the hovered or focused place's open roads, the ways to the destination
        let focused = nodes.iter().map(|n| s(n, "qid")).find(|q| ui.focused(&node_id(q)));
        let near = hovered.clone().or(focused);
        let ways = if self.selected.is_empty() { Vec::new() } else { travel::paths(&d, &gs, &here, &self.selected) };
        if self.pick >= ways.len() {
            self.pick = 0;
        }
        let routes: Vec<(String, String, String, bool)> =
            json::aget(&view, "routes").iter().map(|r| json::dict(r)).map(|r| (s(r, "qid"), s(r, "from"), s(r, "to"), json::b(r, "open", true))).collect();
        let way_routes: Vec<Vec<String>> = ways.iter().map(|w| w.routes.clone()).collect();
        let shown = route_layers(&routes, near.as_deref(), &way_routes, self.pick, peek);
        let line_of = |rq: &str| -> Vec<(f32, f32)> {
            let Some((_, from, to, _)) = routes.iter().find(|r| r.0 == rq) else { return Vec::new() };
            let mut pts = vec![pos_of(from)];
            if let Some(via) = layout.as_ref().and_then(|l| l.route(rq)) {
                pts.extend(via.iter().map(|(x, y)| (rect.0 + x * rect.2, rect.1 + y * rect.3)));
            }
            pts.push(pos_of(to));
            spline(&pts)
        };
        let flicker_on = d.world_map_lantern_flicker;
        for (k, (rq, style)) in shown.iter().enumerate() {
            let alpha = if *style == RouteStyle::Near { flicker(ms, k, flicker_on, reduce) } else { 1.0 };
            draw_road(&mut ui, &line_of(rq), *style, alpha);
        }
        // the other ways take a hover (their preview) and a click (the pick) along their roads; under the places
        for (i, w) in ways.iter().enumerate().filter(|(i, _)| *i != self.pick) {
            let mut k = 0;
            for rq in &w.routes {
                for (x, y) in dots(&line_of(rq), 9.0) {
                    let t = tip(&w.label, &format!("{}. Click to take this way.", cost_text(cx, &gs, w.days, w.supplies)));
                    ui.push(format!("world_map.way.{i}.at.{k}"), "backdrop", w.label.clone(), Rect::new(x as i32 - 4, y as i32 - 4, 9, 9), true, false, Some(WAct::Way(i)), t);
                    k += 1;
                }
            }
        }
        // -- the places: marks, then badges and the party ring, then the names (lit ones last)
        let label_h = (ui.p.line_h(SMALL) + 2.0).ceil() as i32;
        let mut marks: Vec<Mark> = Vec::new();
        for n in &nodes {
            let q = s(n, "qid");
            let (x, y) = pos_of(&q);
            let is_here = q == here;
            let selected = q == self.selected;
            let trip = travel::trip(&d, &gs, &here, &q);
            let locked = !is_here && !json::b(&trip, "ok", false);
            let lit = selected || near.as_deref() == Some(q.as_str());
            let landmark = cx.art.has(&emberwake_content::art::landmark(&q));
            let r = if landmark { landmark_box(x, y, placed.get(&q).map_or(1.0, |p| p.1)) } else { pin_box(x, y) };
            let quest = json::b(n, "quest", false) || !json::sget(n, "jobs").is_empty();
            // the quest badge at the mark's top right (inside a landmark's box, over a pin's corner)
            let badge = quest.then(|| if landmark { Rect::new(r.x + r.w - 13, r.y + 4, 13, 13) } else { Rect::new(r.x + r.w - 5, r.y - 4, 9, 9) });
            let name = s(n, "name");
            let lw = (ui.p.text_w(&name, SMALL) + 6.0).ceil() as i32;
            // the name's foot: over a landmark's top (its art has room there), above a pin and its badge
            let bottom = if landmark { r.y + 2 } else { r.y - 5 };
            let label = label_rect(x, bottom, lw, label_h, MAP);
            marks.push(Mark { qid: q, name, x, y, r, landmark, is_here, selected, lit, locked, badge, label });
        }
        for m in &marks {
            let rel = if m.landmark { emberwake_content::art::landmark(&m.qid) } else { emberwake_content::art::pin(&s(nodes.iter().find(|n| s(n, "qid") == m.qid).unwrap_or(&json::Dict::new()), "kind")) };
            let px = (m.r.w as f32 * ui.p.v.s).round().max(1.0) as i32;
            if m.lit && !m.landmark {
                // hover, focus or pick: a warm glow behind the pin
                ui.p.ellipse(m.x, m.y, 10.0, 10.0, rgba(251, 191, 36, if m.selected { 90 } else { 60 }));
            }
            let img = if m.landmark { cx.art.image(&rel, px, px).cloned() } else { cx.art.image(&rel, PIN_PX, PIN_PX).cloned() };
            match img {
                Some(img) => {
                    if m.lit && m.landmark {
                        // the glow from the landmark's outline (AO-01 (2))
                        let glow = img.outline(rgba(251, 191, 36, if m.selected { 255 } else { 190 }));
                        ui.p.image(&glow, Rect::new(m.r.x - 1, m.r.y - 1, m.r.w + 2, m.r.h + 2));
                    }
                    let img = if m.locked { locked_look(&img) } else { img };
                    ui.p.image(&img, m.r);
                }
                None => ui.p.circle(m.x, m.y, 5.0, if m.locked { rgba(170, 170, 176, 204) } else { rgb(210, 210, 220) }),
            }
            if m.lit && !m.landmark {
                ui.p.circle(m.x, m.y, 8.5, rgba(251, 191, 36, if m.selected { 255 } else { 200 }));
            }
        }
        let glow = pulse(ms, reduce);
        for m in &marks {
            if m.is_here {
                // the party's cyan ring: beside a landmark's foot, around a pin
                let (rx, ry, rad) = if m.landmark { (m.r.x as f32 + 1.0, m.y - 3.0, 7.0) } else { (m.x, m.y, 10.0) };
                ui.p.ellipse(rx, ry, rad + 1.5, rad + 1.5, rgba(6, 182, 212, 50));
                ui.p.ellipse(rx, ry, rad, rad, rgba(6, 182, 212, 64));
                ui.p.ellipse_ring(rx, ry, rad, rad, PARTY);
                if m.landmark {
                    if let Some(img) = cx.art.image(&emberwake_content::art::pin("here"), 10, 10) {
                        ui.p.image(img, Rect::new(rx as i32 - 5, ry as i32 - 5, 10, 10));
                    }
                }
            }
            if let Some(b) = m.badge {
                // the lit, pulsing glow (still under reduce motion), then the badge
                let (bx, by) = (b.x as f32 + b.w as f32 / 2.0, b.y as f32 + b.h as f32 / 2.0);
                let rad = b.w as f32 * (0.7 + 0.25 * glow);
                ui.p.ellipse(bx, by, rad, rad, rgba(245, 158, 11, (50.0 + 90.0 * glow) as u8));
                let px = ui.icon_px(Rect::new(0, 0, b.w + 4, b.h + 4));
                match cx.art.image(&crate::region::service_icon("board"), px, px) {
                    Some(img) => ui.p.image(img, b),
                    None => crate::region::service_glyph(&mut ui, "board", b),
                }
            }
        }
        let mut order: Vec<&Mark> = marks.iter().collect();
        order.sort_by_key(|m| m.lit);
        for m in order {
            let l = m.label;
            let tx = l.x as f32 + 3.0;
            let ty = l.y as f32 + 1.0;
            if m.lit {
                // the full plate (P-UI-02): solid, gold border and text
                ui.p.panel(l, rgba(10, 11, 16, 240), rgb(217, 119, 6), 0.0);
                ui.p.text(tx, ty, &m.name, SMALL, th().gold);
            } else {
                // faint: a nearly clear strip, dim text with a soft shadow, no border
                let fade = if m.locked { 0.8 } else { 1.0 };
                ui.p.fill(l, rgba(10, 11, 16, (71.0 * fade) as u8));
                ui.p.text(tx + 0.5, ty + 0.5, &m.name, SMALL, rgba(0, 0, 0, (200.0 * fade) as u8));
                ui.p.text(tx, ty, &m.name, SMALL, rgba(217, 203, 180, (189.0 * fade) as u8));
            }
        }
        // the places' widgets: the mark picks it, as does the job badge
        let mut hits: Vec<(Rect, String)> = Vec::new();
        for m in &marks {
            let n = nodes.iter().find(|n| s(n, "qid") == m.qid).cloned().unwrap_or_default();
            let jobs = json::sget(&n, "jobs");
            let trip = travel::trip(&d, &gs, &here, &m.qid);
            let mut t = tip(&m.name, &format!("{}{}", s(&n, "region"), if json::b(&n, "visited", false) { "" } else { " (not visited)" }));
            if m.is_here {
                t.push(("Your party is here.".into(), th().info));
            } else if json::b(&trip, "ok", false) {
                t.push((format!("Trip: {}", trip_cost(cx, &gs, &trip)), th().info));
            } else {
                t.push(("No open road yet.".into(), th().dim));
            }
            for step in travel::quest_steps_at(&d, &gs, &m.qid) {
                t.push((format!("Quest: {step}"), th().gold));
            }
            for job in &jobs {
                t.push((format!("Job: {job}"), th().gold));
            }
            // STORYUI1 B10, CAMPAIGN1 W5: the checkpoint lines of the quests whose node this is
            for due in json::aget(&n, "deadlines") {
                t.push((s(json::dict(due), "text"), crate::board::DEADLINE));
            }
            let job_tip = if jobs.is_empty() { Vec::new() } else { t.clone() };
            ui.push(node_id(&m.qid), "node", m.name.clone(), m.r, true, m.selected, Some(WAct::Node(m.qid.clone())), t);
            hits.push((m.r, m.qid.clone()));
            if let (Some(b), false) = (m.badge, jobs.is_empty()) {
                ui.push(format!("{}.job", node_id(&m.qid)), "badge", format!("Job: {}", jobs.join(", ")), b, true, m.selected, Some(WAct::Node(m.qid.clone())), job_tip);
            }
        }
        self.hits = hits;
        // -- the Legend (AO-01 (5), AO-02): the district's quests and the party, floating over the map
        let mut rows: Vec<(String, String)> = Vec::new();
        for (node, name) in travel::quest_places(&d, &gs) {
            if nodes.iter().any(|n| s(n, "qid") == node) && !rows.iter().any(|(_, t)| *t == name) {
                rows.push((node, name));
            }
        }
        for n in &nodes {
            for job in json::sget(n, "jobs") {
                if !rows.iter().any(|(_, t)| *t == job) {
                    rows.push((s(n, "qid"), job));
                }
            }
        }
        let row_h = (ui.p.line_h(SMALL) + 1.0).ceil() as i32;
        let head = "Active quests";
        let head_w = ui.p.text_w(head, SMALL + 1.0) + 2.0 * PANEL_BTN as f32 + 8.0;
        let widest = rows.iter().map(|(_, t)| ui.p.text_w(t, SMALL)).fold(head_w, f32::max);
        let size = (((widest + 18.0).ceil() as i32).clamp(96, 150), 4 + row_h + 2 + row_h * rows.len().max(1) as i32 + 3 + row_h + 3);
        let chip = ((ui.p.text_w("Legend", SMALL + 1.0) + 18.0).ceil() as i32, row_h + 4);
        let taken: Vec<Rect> = marks.iter().flat_map(|m| [m.r, m.label]).collect();
        self.legend.place(MAP, size, &taken);
        self.legend_size = size;
        let held = ui.dragging() == Some(LEGEND_DRAG);
        let now = self.legend.shown(size, chip);
        self.legend.follow(mouse, held, now, MAP);
        if !self.legend.minimized && !held {
            self.legend.clamp(MAP, size);
        }
        let unlocked = !self.legend.locked;
        if self.legend.minimized {
            let c = self.legend.chip_rect(size, chip);
            let t = if unlocked { tip("Legend", "Drag to move it; click to open the legend here.") } else { tip("Legend", "Click to open the legend here.") };
            ui.push("world_map.legend.open", "button", "Legend", c, true, false, Some(WAct::LegendOpen), t);
            if unlocked {
                ui.draggable(LEGEND_DRAG);
            }
            ui.p.panel(c, th().game("legend_fill", rgba(10, 14, 22, 220)), if unlocked { rgb(217, 119, 6) } else { th().edge }, 0.0);
            // three short bars, then the name
            for k in 0..3 {
                ui.p.fill(Rect::new(c.x + 4, c.y + 3 + k * 3, 6, 1), th().text);
            }
            ui.p.text(c.x as f32 + 13.0, c.y as f32 + 2.0, "Legend", SMALL + 1.0, th().gold);
        } else {
            let p = self.legend.rect(size);
            let body_tip = if unlocked { tip("Legend", "Drag it anywhere on the map; lock it to keep it there.") } else { tip("Legend", "Unlock it to drag it anywhere on the map.") };
            // the body takes the clicks over it and, unlocked, the drag (under its buttons and rows)
            ui.push("world_map.legend", "backdrop", "Legend", p, true, false, Some(WAct::Nothing), body_tip);
            if unlocked {
                ui.draggable(LEGEND_DRAG);
                ui.p.panel(Rect::new(p.x - 1, p.y - 1, p.w + 2, p.h + 2), rgba(0, 0, 0, 0), rgba(217, 119, 6, 140), 0.0);
            }
            ui.p.panel(p, th().game("legend_fill", rgba(10, 14, 22, 220)), if unlocked { rgb(217, 119, 6) } else { th().edge }, 0.0);
            ui.p.text(p.x as f32 + 4.0, p.y as f32 + 3.0, head, SMALL + 1.0, th().gold);
            let lb = FloatPanel::lock_button(p);
            if unlocked {
                // the unlocked lock glows
                ui.p.ellipse(lb.x as f32 + lb.w as f32 / 2.0, lb.y as f32 + lb.h as f32 / 2.0, 8.0, 7.0, rgba(245, 158, 11, 80));
            }
            ui.p.panel(lb, rgba(20, 23, 31, 255), if unlocked { rgb(217, 119, 6) } else { th().edge }, 0.0);
            padlock(&mut ui, lb, unlocked, if unlocked { th().gold } else { th().dim });
            let lock_tip = if unlocked { tip("Lock", "Lock the legend in place.") } else { tip("Unlock", "Unlock the legend to drag it anywhere on the map.") };
            ui.push("world_map.legend.lock", "button", if unlocked { "Lock" } else { "Unlock" }, lb, true, unlocked, Some(WAct::LegendLock), lock_tip);
            let mb = FloatPanel::min_button(p);
            ui.p.panel(mb, rgba(20, 23, 31, 255), th().edge, 0.0);
            ui.p.fill(Rect::new(mb.x + 3, mb.y + mb.h / 2, mb.w - 6, 1), th().text);
            ui.push("world_map.legend.min", "button", "Minimize", mb, true, false, Some(WAct::LegendMin), tip("Minimize", "Shrink the legend to a button here."));
            let mut y = p.y + 4 + row_h + 2;
            if rows.is_empty() {
                ui.p.text(p.x as f32 + 4.0, y as f32, "None open", SMALL, th().dim);
                y += row_h;
            }
            for (i, (node, text)) in rows.iter().enumerate() {
                let r = Rect::new(p.x + 2, y, p.w - 4, row_h);
                let on = *node == self.selected;
                if on || ui.hover(r) {
                    ui.p.panel(r, rgba(44, 34, 30, 180), rgb(217, 119, 6), 0.0);
                }
                let b = Rect::new(r.x + 2, r.y + (row_h - 7) / 2, 7, 7);
                if let Some(img) = cx.art.image(&crate::region::service_icon("board"), 7, 7) {
                    ui.p.image(img, b);
                }
                ui.p.text(r.x as f32 + 12.0, r.y as f32, text, SMALL, if on { th().gold } else { th().text });
                let place = cx.location_name(&gs.world_id, node);
                ui.push(format!("world_map.legend.quest.{i}"), "row", text.clone(), r, true, on, Some(WAct::Node(node.clone())), tip(text, &format!("At {place}. Click to pick it.")));
                y += row_h;
            }
            ui.p.fill(Rect::new(p.x + 3, y + 1, p.w - 6, 1), th().edge);
            let r = Rect::new(p.x + 2, y + 3, p.w - 4, row_h);
            if ui.hover(r) {
                ui.p.panel(r, rgba(44, 34, 30, 180), rgb(217, 119, 6), 0.0);
            }
            let (rx, ry) = (r.x as f32 + 5.5, r.y as f32 + row_h as f32 / 2.0);
            ui.p.ellipse(rx, ry, 3.5, 3.5, rgba(6, 182, 212, 64));
            ui.p.ellipse_ring(rx, ry, 3.5, 3.5, PARTY);
            ui.p.text(r.x as f32 + 12.0, r.y as f32, "Your party", SMALL, th().text);
            let party_place = cx.location_name(&gs.world_id, &here);
            ui.push("world_map.legend.party", "row", "Your party", r, true, false, Some(WAct::Node(here.clone())), tip("Your party", &format!("At {party_place}, marked by the cyan ring.")));
        }
        // -- HUD bar (PLAYUI1-H): Back and the location name; gold, supplies and the icon buttons at the right
        let back_w = (ui.p.text_w("Back", BODY) + 8.0).ceil().max(26.0) as i32;
        let here_name = cx.location_name(&gs.world_id, &here);
        hud_bar(&mut ui, cx, back_w + 8, &here_name, ("world_map.menu", ""), "", &WORLD_HELP, WAct::Hud);
        ui.button("world_map.back", Rect::new(3, 3, back_w, 14), "Back", WAct::Back, true);
        // -- Side panel: the time, actions and the log; PR5-48: the layout editor's controls while editing (debug)
        match self.edit.as_ref() {
            Some(e) => {
                let side = Rect::new(504, 24, 132, H - 50);
                e.editor.draw_stage(&mut ui, &WAct::Layout);
                e.editor.draw_controls(&mut ui, side, &WAct::Layout);
            }
            None => self.side_panel(&mut ui, cx, &d, &gs, &here, &ways, peek),
        }
        if crate::DEV_KEYS {
            let (label, t) = if self.edit.is_some() { ("Done editing", tip("Done editing", "Back to the map (F7). Save first: what is not saved is dropped.")) } else { ("Edit layout", tip("Edit layout (debug)", "Edit the landmarks and the roads in place over the map (F7); Save writes the layout sidecar.")) };
            ui.button_tip("world_map.layout_edit", Rect::new(504, H - 20, 130, 14), label, WAct::LayoutEdit, true, t);
        }
        self.w = ui.widgets;
        self.shown = shown;
    }

    /// P, J, I and ? open the HUD bar's popups (PLAYUI1-H). PR5-48: F7 switches the debug layout editing (a debug build
    /// with the dev tools; a release build takes the key and does nothing); while editing the editor takes its keys.
    fn key(&mut self, cx: &mut Cx, k: Key) {
        if k == Key::Fn(7) {
            self.toggle_edit(cx);
            return;
        }
        if let Some(e) = self.edit.as_mut() {
            if e.key(cx, k) {
                return;
            }
        }
        if crate::viewkeys::window_key(cx, k, crate::viewkeys::Shown::Notice) {
            return;
        }
        if let Some(h) = hud_key(k) {
            self.act(cx, WAct::Hud(h));
        }
    }

    /// The pick, the chosen way and the ways offered, the roads drawn in the last frame (MAP-HOOKS1) and the Legend.
    fn state(&self, cx: &Cx) -> Value {
        let ways: Vec<Value> = self.ways(cx).iter().map(|w| json!({"label": w.label, "days": w.days, "supplies": w.supplies, "routes": w.routes})).collect();
        let roads: Vec<Value> = self.shown.iter().map(|(r, st)| json!({"route": r, "style": st.name()})).collect();
        let l = &self.legend;
        json!({
            "selected": self.selected, "pick": self.pick, "ways": ways, "roads": roads,
            "legend": {"pos": l.pos.map(|p| vec![p.0, p.1]), "locked": l.locked, "minimized": l.minimized},
            "view": travel::view(&cx.data, cx.gs()),
            // PR5-48: the layout being edited in place and the editor's state
            "layout_edit": self.edit.as_ref().map(|e| e.name.clone()),
            aegis_ui::layout_editor::STATE_KEY: self.edit.as_ref().map(|e| e.editor.state()),
        })
    }

    /// PR5-48: while editing the arrows nudge (the editor's), not the focus.
    fn arrow_focus(&self) -> bool {
        self.edit.is_none()
    }

    /// PR5-48: Esc ends the layout editing first.
    fn escape(&mut self, cx: &mut Cx) {
        if self.edit.is_some() {
            self.toggle_edit(cx);
        } else {
            self.back(cx);
        }
    }

    /// PR5-48: a handle drag of the layout editor while editing; any other press clicks.
    fn pointer(&mut self, cx: &mut Cx, id: &str, phase: PointerPhase, pos: (i32, i32)) {
        if self.edit.as_mut().is_some_and(|e| e.editor.pointer(id, phase, pos)) {
            return;
        }
        if phase == PointerPhase::Down {
            self.click(cx, pos);
        }
    }

    /// PR5-48: the layout editor's agent commands while editing (`layout_markers`, `layout`).
    fn command(&mut self, _cx: &Cx, body: &Value) -> Option<Value> {
        self.edit.as_mut()?.editor.command(body)
    }

    /// STORYUI1 W2, 5.3: a running sequence takes over and queued vignettes play (the map has nothing of its own open).
    /// CAMPAIGN1: a Yes to the trip or camp ask goes ahead; campaign notices and the run-over screen come first.
    fn tick(&mut self, cx: &mut Cx) {
        if cx.take_answer(crate::runover::ASK_TRIP) {
            self.travel(cx);
            return;
        }
        if cx.take_answer(crate::runover::ASK_REST) {
            self.camp(cx);
            return;
        }
        if !crate::runover::map_tick(cx) {
            crate::story::map_tick(cx);
        }
    }

    fn music(&self, cx: &Cx) -> Option<String> {
        music::game_track(cx, music::TRAVEL)
    }

    widget_plumbing!();
}

// ---- PR5-48 (MAP-LAYOUT-ED): the layout editor over a world map ------------------------------------------------

/// A landmark image's side as the editor loads it (the 96 px cut, shown at `LANDMARK_PX` times its scale).
const LANDMARK_IMG: i32 = 96;

/// The names the Map Layout editor offers: each world map sidecar and background of the layers
/// (`world_map::sidecar_names`, `world_map::background_names`), sorted.
pub fn layout_names(layers: &aegis_core::assets::Layers) -> Vec<String> {
    let mut out = world_map::sidecar_names(layers);
    for n in world_map::background_names(layers) {
        if !out.contains(&n) {
            out.push(n);
        }
    }
    out.sort();
    out
}

/// Whether location `qid` sits in region `name` (its region or one up the region's `parent` chain).
fn in_region(d: &emberwake_world::Data, world: &str, qid: &str, name: &str) -> bool {
    let loc = d.resolve(world, "locations", qid);
    let mut w = json::s_or(loc, "_world", world);
    let mut region_ref = s(loc, "region");
    let mut seen: Vec<String> = Vec::new();
    while !region_ref.is_empty() {
        let region = d.resolve(&w, "regions", &region_ref);
        let id = s(region, "id");
        if id.is_empty() || seen.contains(&id) {
            return false;
        }
        if id == name {
            return true;
        }
        seen.push(id);
        w = json::s_or(region, "_world", &w);
        region_ref = s(region, "parent");
    }
    false
}

/// The world and the places (qids, in the world map's node order) that layout `name` lays out: a world's nodes, or a
/// region's world's nodes in that region; none for a pre-staged or unknown name (`world_map::owner`).
pub fn layout_places(d: &emberwake_world::Data, layers: &aegis_core::assets::Layers, name: &str) -> (Option<String>, Vec<String>) {
    match world_map::owner(&d.catalogs, layers, name) {
        Owner::World(w) => {
            let nodes = travel::nodes(d, &w);
            (Some(w), nodes)
        }
        Owner::Region { world } => {
            let nodes = travel::nodes(d, &world).into_iter().filter(|q| in_region(d, &world, q, name)).collect();
            (Some(world), nodes)
        }
        _ => (None, Vec::new()),
    }
}

/// A world map layout open in the layout editor (`aegis_ui::layout_editor`): the sidecar's name and the editor, whose
/// markers are the layout's places at their sidecar landmark or `map_pos` (`place` in map fractions) and whose
/// polylines are the places' routes with the sidecar's waypoints (none yet: empty), then any other route the sidecar
/// holds (a pre-staged act's). The viewport is the world map's (`MAP`, `MAP_PAD`) and the markers are boxed by its
/// landmark rule, so the editor shows the map as the player sees it. Save writes through the kit's writer.
pub struct LayoutEditing {
    pub name: String,
    pub editor: LayoutEditor,
    /// The painter scale the background was loaded for.
    bg_scale: Option<f32>,
}

impl LayoutEditing {
    /// Opens layout `name`; `images`: the landmark images on the markers (the Map Layout screen; the live map draws its
    /// own).
    pub fn open(cx: &mut Cx, name: &str, images: bool) -> Self {
        let d = cx.data.clone();
        let layers = cx.art.layers().clone();
        let layout = read_layout(&layers, &[name.to_string()]);
        let (world, places) = layout_places(&d, &layers, name);
        let unit = (0.0, 0.0, 1.0, 1.0);
        let mut markers = Vec::new();
        for (i, q) in places.iter().enumerate() {
            let loc = d.resolve(world.as_deref().unwrap_or_default(), "locations", q);
            let ((x, y), scale) = place(layout.as_ref(), q, json::aget(loc, "map_pos"), i, places.len(), unit);
            let local = local_id(q);
            // the sidecar's key: the local id, unless the sidecar keys this place by its qid
            let id = if layout.as_ref().is_some_and(|l| l.landmarks.contains_key(q.as_str())) { q.clone() } else { local.to_string() };
            let mut m = Marker::new(&id, &json::s_or(loc, "name", local), [x, y], scale);
            if images {
                m.image = cx.art.image(&emberwake_content::art::landmark(q), LANDMARK_IMG, LANDMARK_IMG).cloned().map(Rc::new);
            }
            markers.push(m);
        }
        let mut lines: Vec<Polyline> = Vec::new();
        if let Some(w) = &world {
            for r in travel::graph(&d, w) {
                let id = s(&r, "id");
                if places.contains(&s(&r, "_from")) && places.contains(&s(&r, "_to")) && !lines.iter().any(|l| l.id == id) {
                    let pts = layout.as_ref().and_then(|l| l.route(&id)).map(|p| p.iter().map(|(x, y)| [*x, *y]).collect()).unwrap_or_default();
                    lines.push(Polyline::new(&id, pts));
                }
            }
        }
        for (id, pts) in layout.iter().flat_map(|l| l.routes.iter()) {
            if !lines.iter().any(|l| &l.id == id) {
                lines.push(Polyline::new(id, pts.iter().map(|(x, y)| [*x, *y]).collect()));
            }
        }
        let mut editor = LayoutEditor::new(markers, lines);
        editor.set_view(Viewport::new(MAP, (MAP_PAD.0 as f32, MAP_PAD.1 as f32)));
        editor.marker_box = MarkerBox { side: LANDMARK_PX, foot: LANDMARK_FOOT };
        Self { name: name.to_string(), editor, bg_scale: None }
    }

    /// The act background on the editor's stage, loaded once per painter scale `s` at the size it covers the map.
    fn background(&mut self, cx: &mut Cx, scale: f32) {
        if self.bg_scale == Some(scale) {
            return;
        }
        self.bg_scale = Some(scale);
        let rel = emberwake_content::art::world_map_background(&self.name);
        self.editor.background = cx.art.size_of(&rel).and_then(|(iw, ih)| {
            let r = cover_rect(iw, ih, MAP);
            cx.art.image_fit(&rel, ((r.w as f32) * scale).round().max(1.0) as i32, ((r.h as f32) * scale).round().max(1.0) as i32).cloned().map(Rc::new)
        });
    }

    /// The layout as the editor holds it now (the live map draws by it while editing).
    pub fn layout(&self) -> MapLayout {
        let mut out = MapLayout::default();
        for m in self.editor.markers() {
            out.landmarks.insert(m.id.clone(), Landmark { pos: (m.pos[0], m.pos[1]), scale: m.scale });
        }
        for l in self.editor.polylines() {
            out.routes.insert(l.id.clone(), l.points.iter().map(|p| (p[0], p[1])).collect());
        }
        out
    }

    /// One of the pane's actions, then a Save it asked for (`settle`).
    pub fn act(&mut self, cx: &mut Cx, a: LayoutAct) {
        self.editor.act(a);
        self.settle(cx);
    }

    /// A key for the pane (the arrows with Shift, + and -, Ctrl+Z, PageUp / PageDown); true when it took it.
    pub fn key(&mut self, cx: &mut Cx, k: Key) -> bool {
        let used = self.editor.key(k, cx.held.shift, cx.held.ctrl);
        self.settle(cx);
        used
    }

    /// The Save the editor asked for (`take_save`), written through the kit's writer (`world_map::write`: the edits
    /// checked, the canonical sidecar written into the game's own asset layer), acknowledged (`mark_saved`) and the art
    /// read afresh (a new `Art` generation), so every world map reads the new sidecar; a refusal shows its problems,
    /// writes nothing and keeps the values unsaved.
    pub fn settle(&mut self, cx: &mut Cx) {
        let Some(edits) = self.editor.take_save() else { return };
        let list = Value::Array(edits.iter().map(LayoutEdit::to_json).collect());
        match world_map::write(&cx.data.catalogs, cx.art.layers(), &self.name, &list) {
            Ok(w) => {
                self.editor.mark_saved();
                self.editor.note = format!("saved {} ({} changed)", emberwake_content::art::world_map_layout(&self.name), w.edits);
                cx.art = aegis_ui::art::Art::layered(cx.art.layers().clone());
                self.bg_scale = None;
            }
            Err(problems) => {
                self.editor.note = "save refused: nothing written".into();
                show_errors(cx, problems);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum LAct {
    Edit(LayoutAct),
    Pick(usize),
    Back,
}

/// PR5-48: the Scene Runner's Map Layout (dev only): any act's world map (`layout_names`: Act 1's sidecar and the
/// pre-staged backgrounds) in the layout editor, its stage on the world map's viewport with the act background and the
/// landmark images, the controls in the side panel's place; Save writes the sidecar (a native debug build only).
pub struct MapLayoutScreen {
    names: Vec<String>,
    at: usize,
    edit: LayoutEditing,
    w: Vec<Widget<LAct>>,
}

impl MapLayoutScreen {
    /// Opens on `name` when it is one of `layout_names`, else the first name with a sidecar, else the first name.
    pub fn new(cx: &mut Cx, name: Option<&str>) -> Self {
        let layers = cx.art.layers().clone();
        let names = layout_names(&layers);
        let sidecars = world_map::sidecar_names(&layers);
        let at = name.and_then(|n| names.iter().position(|x| x == n)).or_else(|| names.iter().position(|n| sidecars.contains(n))).unwrap_or(0);
        let edit = LayoutEditing::open(cx, names.get(at).map(String::as_str).unwrap_or_default(), true);
        Self { names, at, edit, w: Vec::new() }
    }

    fn act(&mut self, cx: &mut Cx, a: LAct) {
        match a {
            LAct::Edit(a) => self.edit.act(cx, a),
            LAct::Pick(i) if i < self.names.len() && i != self.at => {
                self.at = i;
                self.edit = LayoutEditing::open(cx, &self.names[i], true);
            }
            LAct::Pick(_) => {}
            LAct::Back => cx.nav.push(Nav::Pop),
        }
    }
}

impl Screen<GameCx> for MapLayoutScreen {
    fn id(&self) -> &'static str {
        "map_layout"
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<LAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().tool_bg);
        self.edit.background(cx, ui.p.v.s);
        let back_w = (ui.p.text_w("Back", BODY) + 8.0).ceil().max(26.0) as i32;
        ui.button("map_layout.back", Rect::new(3, 3, back_w, 14), "Back", LAct::Back, true);
        ui.p.text((back_w + 10) as f32, 5.0, "MAP LAYOUT", BODY, th().gold);
        let labels: Vec<String> = self.names.clone();
        let t = tip("World map", "The act's world map to lay out: a sidecar, or a background whose act has no data yet (no places).");
        ui.select("map_layout.name", Rect::new(back_w + 86, 3, 160, 14), &labels, self.at, LAct::Pick, labels.len() > 1, t);
        let (_, places) = layout_places(&cx.data, cx.art.layers(), &self.edit.name);
        let what = if places.is_empty() { "no places in the data yet".to_string() } else { format!("{} places, {} roads", self.edit.editor.markers().len(), self.edit.editor.polylines().len()) };
        ui.p.text_fit((back_w + 252) as f32, 5.0, &what, SMALL, 230.0, th().dim);
        self.edit.editor.draw_stage(&mut ui, &LAct::Edit);
        self.edit.editor.draw_controls(&mut ui, Rect::new(504, 24, 132, H - 28), &LAct::Edit);
        self.w = ui.widgets;
    }

    /// The arrows nudge the selection (Shift: more), + and - size the marker, Ctrl+Z undoes.
    fn key(&mut self, cx: &mut Cx, k: Key) {
        self.edit.key(cx, k);
    }

    fn arrow_focus(&self) -> bool {
        false
    }

    /// A handle drag moves a marker or a point (`LayoutEditor::pointer`); any other press clicks.
    fn pointer(&mut self, cx: &mut Cx, id: &str, phase: PointerPhase, pos: (i32, i32)) {
        if !self.edit.editor.pointer(id, phase, pos) && phase == PointerPhase::Down {
            self.click(cx, pos);
        }
    }

    fn state(&self, _cx: &Cx) -> Value {
        json!({"name": self.edit.name, "names": self.names, aegis_ui::layout_editor::STATE_KEY: self.edit.editor.state()})
    }

    /// The pane's agent commands (`layout_markers`, `layout {id, pos, scale}`, `layout {route, points}`).
    fn command(&mut self, _cx: &Cx, body: &Value) -> Option<Value> {
        self.edit.editor.command(body)
    }

    widget_plumbing!();
}

/// MAP-HOOKS1 pass A: the act background and layout lookup (region chain, then the world lineage), the layout sidecar's
/// reading (defaults, unclamped positions, a missing or bad file passed over), where a node stands and the box its
/// landmark or pin is drawn in, and the shipped Act 1 sidecar against the shipped world. Expectations follow from the
/// rules and the fixtures' own numbers.
#[cfg(test)]
mod tests {
    use super::*;
    use aegis_core::assets::Layers;
    use aegis_core::scratch::Scratch;
    use emberwake_world::state::{PartyState, UnitState};
    use emberwake_world::{Data, GameState};
    use std::path::Path;

    fn with_ids(w: &str, rows: Value) -> Value {
        Value::Array(
            json::arr(&rows)
                .iter()
                .map(|r| {
                    let mut r = json::dict(r).clone();
                    let id = s(&r, "id");
                    r.insert("_world".into(), w.into());
                    r.insert("_qid".into(), format!("{w}:{id}").into());
                    Value::Object(r)
                })
                .collect(),
        )
    }

    /// World `ward` extends `base`; its region `quarter` sits in region `district`; `hub` is in `quarter`, `wild` in
    /// no region.
    fn data() -> Data {
        let worlds = json!({
            "base": {"manifest": {"id": "base", "name": "Base"}, "own": {}},
            "ward": {"manifest": {"id": "ward", "name": "Ward", "extends": ["base"], "start_location": "hub", "start_party": ["hero"]}, "own": {
                "regions": with_ids("ward", json!([{"id": "quarter", "name": "Quarter", "parent": "district"}, {"id": "district", "name": "District", "parent": ""}])),
                "locations": with_ids("ward", json!([{"id": "hub", "name": "Hub", "region": "quarter"}, {"id": "wild", "name": "Wild"}])),
                "routes": with_ids("ward", json!([{"id": "r1", "from": "hub", "to": "wild", "time": 1}]))
            }}
        });
        let catalog = json!({"classes": [{"id": "fighter", "name": "Fighter"}], "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]});
        Data::from_values(&catalog, &worlds, &json!({}))
    }

    fn state(d: &Data, at: &str) -> GameState {
        let mut party = PartyState::default();
        party.members.push(UnitState::create("hero", "hero"));
        let mut gs = GameState::create(d, "ward", party);
        gs.location_id = at.into();
        gs
    }

    #[test]
    fn the_background_follows_the_region_chain_then_the_world_lineage() {
        let d = data();
        assert_eq!(map_names(&d, &state(&d, "ward:hub")), ["quarter", "district", "ward", "base"]);
        assert_eq!(map_names(&d, &state(&d, "ward:wild")), ["ward", "base"], "a place in no region uses the world's");
        let backgrounds: Vec<String> = map_names(&d, &state(&d, "ward:hub")).iter().map(|n| emberwake_content::art::world_map_background(n)).collect();
        assert_eq!(backgrounds[0], "backgrounds/world_map/quarter.png");
        assert_eq!(backgrounds[2], "backgrounds/world_map/ward.png");
    }

    #[test]
    fn a_layout_sidecar_reads_with_defaults_and_unclamped_positions() {
        let v = json!({
            "landmarks": {"hub": {"pos": [0.5, 1.042]}, "gate": {"pos": [-0.01, 0.2], "scale": 1.5}, "bad": {"pos": [0.1]}, "zero": {"pos": [0.2, 0.2], "scale": 0}},
            "routes": {"r1": [[0.1, 0.2], [0.3, 0.4]], "r2": [], "broken": [[0.1, "x"]]}
        });
        let l = MapLayout::parse(&v).expect("an object reads");
        assert_eq!(l.landmark("hub"), Some(&Landmark { pos: (0.5, 1.042), scale: 1.0 }), "scale defaults to 1, pos is not clamped");
        assert_eq!(l.landmark("ward:gate"), Some(&Landmark { pos: (-0.01, 0.2), scale: 1.5 }), "a qid finds the local id");
        assert_eq!(l.landmark("bad"), None, "a pos that is not two numbers drops the landmark");
        assert_eq!(l.landmark("zero").map(|x| x.scale), Some(1.0), "a scale that is not positive is 1");
        assert_eq!(l.route("ward:r1"), Some(&[(0.1, 0.2), (0.3, 0.4)][..]));
        assert_eq!(l.route("r2"), Some(&[][..]), "no waypoints: a straight leg");
        assert_eq!(l.route("broken"), None);
        assert_eq!(MapLayout::parse(&json!([1, 2])), None, "not an object");
        assert_eq!(MapLayout::parse(&json!({})), Some(MapLayout::default()));
    }

    #[test]
    fn a_missing_or_bad_sidecar_is_passed_over() {
        let root = Scratch::new("worldmap_layout");
        let dir = root.join("backgrounds/world_map");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("quarter.json"), "{ not json").unwrap();
        std::fs::write(dir.join("district.json"), "[1, 2]").unwrap();
        let layers = Layers::of(vec![root.path_buf()]);
        let names: Vec<String> = ["quarter", "district", "ward", "base"].iter().map(|n| n.to_string()).collect();
        assert_eq!(read_layout(&layers, &names), None, "bad files and no world file: every node keeps its map_pos");
        std::fs::write(dir.join("base.json"), r#"{"landmarks": {"hub": {"pos": [0.25, 0.75], "scale": 2}}}"#).unwrap();
        let l = read_layout(&layers, &names).expect("the lineage's file");
        assert_eq!(l.landmark("ward:hub"), Some(&Landmark { pos: (0.25, 0.75), scale: 2.0 }));
        std::fs::write(dir.join("quarter.json"), r#"{"landmarks": {"hub": {"pos": [0.5, 0.5]}}}"#).unwrap();
        assert_eq!(read_layout(&layers, &names).unwrap().landmark("hub").unwrap().pos, (0.5, 0.5), "the region's file comes first");
    }

    #[test]
    fn a_node_stands_on_its_landmark_then_its_map_pos_then_a_slot() {
        let rect = (10.0, 20.0, 500.0, 200.0);
        let layout = MapLayout::parse(&json!({"landmarks": {"hub": {"pos": [1.042, -0.1], "scale": 1.25}}})).unwrap();
        let mp = [json!(0.5), json!(0.5)];
        assert_eq!(place(Some(&layout), "ward:hub", &mp, 0, 2, rect), ((10.0 + 1.042 * 500.0, 20.0 - 0.1 * 200.0), 1.25), "the sidecar overrides map_pos, unclamped");
        assert_eq!(place(Some(&layout), "ward:wild", &mp, 1, 2, rect), ((260.0, 120.0), 1.0), "no landmark: map_pos");
        assert_eq!(place(None, "ward:wild", &[json!(1.5), json!(-1)], 1, 2, rect), ((510.0, 20.0), 1.0), "map_pos is clamped to 0..1");
        let ((x, y), scale) = place(None, "ward:wild", &[], 0, 4, rect);
        let radius = 200.0 * 0.38;
        assert_eq!(scale, 1.0);
        assert!((x - 260.0).abs() < 1e-3 && (y - (120.0 - radius)).abs() < 1e-3, "slot 0 sits at the top of the circle: {x}, {y}");
    }

    #[test]
    fn a_landmark_grows_upward_from_its_ground_point() {
        let side = LANDMARK_PX as i32;
        let foot = LANDMARK_FOOT as i32;
        assert_eq!(landmark_box(100.0, 200.0, 1.0), Rect::new(100 - side / 2, 200 + foot - side, side, side));
        let big = landmark_box(100.0, 200.0, 1.5);
        let s15 = (LANDMARK_PX * 1.5).round() as i32;
        assert_eq!((big.w, big.h), (s15, s15));
        assert_eq!(big.y + big.h, 200 + foot, "the bottom stays the same at any scale");
        assert_eq!(big.x + big.w / 2, 100, "centred on the ground point");
        assert_eq!(pin_box(100.0, 200.0), Rect::new(100 - PIN_PX / 2, 200 - PIN_PX / 2, PIN_PX, PIN_PX));
    }

    /// The shipped Act 1 sidecar (Art/UI owns its art; AO-01) reads, places only Sunk Ward locations, keys only Sunk Ward
    /// routes, and is the layout of the Act 1 region's lookup.
    #[test]
    fn the_shipped_act_one_layout_matches_the_world() {
        let game = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let layers = Layers::for_game(&game);
        let dir = emberwake_world::Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        let d = emberwake_world::Data::open(&dir).expect("data").0;
        let rel = emberwake_content::art::world_map_layout("sunk_ward_quarter");
        let v = aegis_core::assets::read_json(&layers, &rel).expect("the sidecar is JSON");
        let l = MapLayout::parse(&v).expect("a layout");
        assert!(!l.landmarks.is_empty() && !l.routes.is_empty());
        for (id, mark) in &l.landmarks {
            let loc = d.resolve("sunk_ward", "locations", id);
            assert_eq!(s(loc, "_world"), "sunk_ward", "{id} is a Sunk Ward location");
            assert_eq!(s(loc, "region"), "sunk_ward_quarter", "{id} sits in the region the sidecar is named for");
            assert!(mark.scale > 0.0, "{id}");
        }
        let routes: Vec<String> = d.merged_entries("sunk_ward", "routes").iter().map(|r| s(r, "id")).collect();
        for id in l.routes.keys() {
            assert!(routes.contains(id), "{id} is a Sunk Ward route");
        }
        assert_eq!(read_layout(&layers, &["sunk_ward_quarter".to_string()]), Some(l));
    }

    // ---- MAP-HOOKS1 pass B ----------------------------------------------------------------------------------------

    use crate::hud::hud_tests::{frame, gfx};

    const SW: &str = "sunk_ward";
    /// Act 1's Tier 1 missions and Vesper's lock ciphers (the flags the shipped routes name).
    const TIER2: [&str; 3] = ["sunk_ward.m1_done", "sunk_ward.m2_done", "sunk_ward.lock_ciphers_done"];

    fn q(id: &str) -> String {
        format!("{SW}:{id}")
    }

    /// The shipped game at the Sunk Ward hub once the Prologue is over, with `flags` set and the places `visited` been
    /// to (the fog lifts one open road past a visited place); the game's art and sidecar when `art`.
    fn ward_hub(flags: &[&str], visited: &[&str], art: bool) -> Cx {
        use emberwake_world::{battle, new_game, story};
        let mut cx = crate::hud::hud_tests::game_cx();
        let d = cx.data.clone();
        let gs = new_game::create_for_world(&d, SW, 1).expect("a new sunk_ward game");
        crate::screens::enter_game(&mut cx, gs, -1, Vec::new());
        cx.nav.clear();
        let gs = cx.gs_mut();
        for _ in 0..200 {
            if !story::is_running(gs) {
                break;
            }
            if let Some(fight) = story::fight(&d, gs) {
                assert!(json::b(&battle::finish_world(&d, gs, story::FIGHT_SOURCE, &fight, "won", None), "ok", false));
            } else if !json::b(&story::step_done(&d, gs), "ok", false) {
                assert!(json::b(&story::finish(&d, gs), "ok", false));
            }
        }
        assert_eq!(gs.location_id, q("sluice_undercroft"), "the Prologue ends at the hub");
        for f in flags {
            gs.set_flag(f, json!(true));
        }
        for v in visited {
            gs.set_flag(&explore::visited_flag(&q(v)), json!(true));
        }
        if art {
            cx.art = aegis_ui::art::Art::new(Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        }
        cx
    }

    /// Marks every board of the world opened (its jobs show on the map and in the Legend).
    fn open_boards(cx: &mut Cx) {
        let d = cx.data.clone();
        for b in d.merged_entries(SW, "boards") {
            emberwake_world::board::mark_opened(&d, cx.gs_mut(), &s(&b, "id"));
        }
    }

    fn widget(wm: &WorldMap, id: &str) -> Option<Value> {
        wm.widgets().into_iter().find(|w| w["id"] == id)
    }

    fn centre(wm: &WorldMap, id: &str) -> (i32, i32) {
        let (r, _) = wm.widget_rect(id).unwrap_or_else(|| panic!("{id}"));
        (r.x + r.w / 2, r.y + r.h / 2)
    }

    /// Two frames with the pointer at `mouse` (hover reads the frame before).
    fn hover(wm: &mut WorldMap, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32)) {
        for _ in 0..2 {
            wm.draw(cx, g, mouse, true);
        }
    }

    fn roads(wm: &WorldMap, cx: &Cx) -> Vec<(String, String)> {
        json::aget(json::dict(&wm.state(cx)), "roads").iter().map(|r| (s(json::dict(r), "route"), s(json::dict(r), "style"))).collect()
    }

    /// MAP-HOOKS1: the viewport and padding of Art/UI's preview (`world_diamond.py`: `px(mx) = 34 + mx * (496 - 68)`,
    /// `py(my) = 22 + my * (328 - 44)` inside the 496 x 328 viewport), so a landmark at y 0.99 keeps its foot on the map;
    /// the background covers the viewport, centred.
    #[test]
    fn the_map_uses_the_preview_viewport_padding_and_cover() {
        let (x, y, w, h) = anchor_rect();
        assert_eq!((MAP.w, MAP.h), (496, 328));
        assert_eq!((x - MAP.x as f32, y - MAP.y as f32, w, h), (34.0, 22.0, 496.0 - 68.0, 328.0 - 44.0));
        let ((gx, gy), _) = place(None, "w:hub", &[json!(0.6), json!(0.99)], 0, 1, anchor_rect());
        assert_eq!(gx, MAP.x as f32 + 34.0 + 0.6 * 428.0);
        let foot = landmark_box(gx, gy, 1.0);
        assert!(foot.y + foot.h <= MAP.y + MAP.h, "the hub's landmark ends on the map: {foot:?}");
        for (iw, ih) in [(1920, 1080), (100, 1000), (496, 328)] {
            let r = cover_rect(iw, ih, MAP);
            assert!(r.w >= MAP.w && r.h >= MAP.h && (r.w == MAP.w || r.h == MAP.h), "{iw}x{ih}: covers, one side exact: {r:?}");
            assert!(((r.x + r.w / 2) - (MAP.x + MAP.w / 2)).abs() <= 1 && ((r.y + r.h / 2) - (MAP.y + MAP.h / 2)).abs() <= 1, "centred: {r:?}");
        }
    }

    /// A road is a smooth line through its waypoints (each waypoint on it, the ends its places), a straight segment
    /// without any; the lantern dots fall every `step` px along it.
    #[test]
    fn a_road_runs_smoothly_through_its_waypoints() {
        assert_eq!(spline(&[(0.0, 0.0), (10.0, 0.0)]), [(0.0, 0.0), (10.0, 0.0)], "no waypoint: straight");
        assert!(spline(&[(1.0, 1.0)]).is_empty());
        let pts = [(0.0, 0.0), (50.0, 40.0), (100.0, 0.0), (140.0, 30.0)];
        let line = spline(&pts);
        assert_eq!((line[0], *line.last().unwrap()), (pts[0], pts[3]));
        for p in pts {
            assert!(line.contains(&p), "{p:?} is on the road");
        }
        assert!(line.windows(2).all(|w| ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt() <= 4.0), "sampled finely");
        assert_eq!(dots(&[(0.0, 0.0), (10.0, 0.0)], 4.0), [(0.0, 0.0), (4.0, 0.0), (8.0, 0.0)]);
        assert_eq!(dots(&[(0.0, 0.0), (3.0, 0.0), (6.0, 0.0)], 4.0), [(0.0, 0.0), (4.0, 0.0)], "the spacing carries over a bend");
    }

    /// AO-01 (4), focus only: nothing at rest; the hovered or focused place's open roads dimmed, never a locked one;
    /// with a destination the other ways pale first and the chosen way gold last; the previewed way lit; each road once.
    #[test]
    fn roads_show_on_focus_only_with_the_chosen_way_on_top() {
        let r = |id: &str, a: &str, b: &str, open: bool| (id.to_string(), a.to_string(), b.to_string(), open);
        let routes = vec![r("r1", "hub", "a", true), r("r2", "a", "b", true), r("r3", "hub", "c", false), r("r4", "hub", "b", true), r("r5", "c", "b", true)];
        let ids = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        let st = |v: Vec<(String, RouteStyle)>| v.into_iter().map(|(r, s)| (r, s.name())).collect::<Vec<_>>();
        let want = |v: &[(&str, &'static str)]| v.iter().map(|(a, b)| (a.to_string(), *b)).collect::<Vec<_>>();
        assert!(route_layers(&routes, None, &[], 0, None).is_empty(), "at rest nothing");
        assert_eq!(st(route_layers(&routes, Some("hub"), &[], 0, None)), want(&[("r1", "near"), ("r4", "near")]), "the hub's open roads; r3 is locked");
        assert_eq!(st(route_layers(&routes, Some("c"), &[], 0, None)), want(&[("r5", "near")]));
        let ways = vec![ids(&["r4"]), ids(&["r1", "r2"])];
        assert_eq!(st(route_layers(&routes, None, &ways, 0, None)), want(&[("r1", "alt"), ("r2", "alt"), ("r4", "main")]), "pale under, the chosen way last");
        assert_eq!(st(route_layers(&routes, None, &ways, 1, None)), want(&[("r4", "alt"), ("r1", "main"), ("r2", "main")]), "the other way picked");
        assert_eq!(st(route_layers(&routes, None, &ways, 0, Some(1))), want(&[("r1", "peek"), ("r2", "peek"), ("r4", "main")]), "hover previews it");
        assert_eq!(st(route_layers(&routes, Some("hub"), &ways, 0, None)), want(&[("r1", "alt"), ("r2", "alt"), ("r4", "main")]), "a way's road is the way's, not dimmed");
        let ways = vec![ids(&["x", "y"]), ids(&["x", "z"]), ids(&["w", "z"])];
        assert_eq!(st(route_layers(&routes, None, &ways, 0, Some(2))), want(&[("w", "peek"), ("z", "peek"), ("x", "main"), ("y", "main")]), "each road once, the previewed way lit");
    }

    /// The shipped Act 1 map: nothing at rest; hovering the grate shows its open roads only (Corvo's road is shut until
    /// M1); a picked destination shows every way `travel::paths` offers, the cheapest gold and last; a way's row
    /// previews it on hover and picks it on a click, as a click on its road on the map does.
    #[test]
    fn the_sunk_ward_map_shows_roads_on_hover_and_every_way_to_a_pick() {
        let grate = q("drowned_runoff_grate");
        let mut cx = ward_hub(&[], &["drowned_runoff_grate", "corvos_gate"], false);
        let (mut wm, mut g) = (WorldMap::new(), gfx());
        frame(&mut wm, &mut cx, &mut g, 1.0);
        assert!(roads(&wm, &cx).is_empty(), "at rest no road");
        let at = centre(&wm, &node_id(&grate));
        hover(&mut wm, &mut cx, &mut g, at);
        let view = travel::view(&cx.data, cx.gs());
        let (mut open, mut shut) = (Vec::new(), Vec::new());
        for r in json::aget(&view, "routes").iter().map(json::dict).filter(|r| s(r, "from") == grate || s(r, "to") == grate) {
            if json::b(r, "open", true) { open.push(s(r, "qid")) } else { shut.push(s(r, "qid")) }
        }
        assert!(!open.is_empty() && !shut.is_empty(), "the grate has open and shut roads on day 1: {open:?} {shut:?}");
        let got = roads(&wm, &cx);
        assert_eq!(got.iter().map(|(r, _)| r.clone()).collect::<Vec<_>>(), open, "its open roads in view order");
        assert!(got.iter().all(|(_, st)| st == "near"), "dimmed: {got:?}");
        hover(&mut wm, &mut cx, &mut g, (-500, -500));
        assert!(roads(&wm, &cx).is_empty(), "the pointer gone: nothing");
        // Tier 2 open: the Tallow Vats by more than one way
        let mut cx = ward_hub(&TIER2, &["drowned_runoff_grate", "alleyway_junction_7", "ash_crawl_crossroads"], false);
        let vats = q("tallow_vats");
        let mut wm = WorldMap::new();
        frame(&mut wm, &mut cx, &mut g, 1.0);
        wm.click_id(&mut cx, &node_id(&vats)).expect("the vats");
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let ways = travel::paths(&cx.data, cx.gs(), &cx.gs().location_id, &vats);
        assert!(ways.len() >= 2, "more than one way: {ways:?}");
        let state = wm.state(&cx);
        assert_eq!(json::aget(json::dict(&state), "ways").iter().map(|w| s(json::dict(w), "label")).collect::<Vec<_>>(), ways.iter().map(|w| w.label.clone()).collect::<Vec<_>>());
        let main_of = |got: &[(String, String)]| -> Vec<String> {
            let at = got.iter().position(|(_, st)| st == "main").expect("a main path");
            assert!(got[..at].iter().all(|(_, st)| st == "alt" || st == "peek") && got[at..].iter().all(|(_, st)| st == "main"), "alternatives first, the main path last: {got:?}");
            got[at..].iter().map(|(r, _)| r.clone()).collect()
        };
        assert_eq!(main_of(&roads(&wm, &cx)), ways[0].routes, "the cheapest is the main path");
        let pale: Vec<String> = ways[1].routes.iter().filter(|r| !ways[0].routes.contains(r)).cloned().collect();
        assert!(pale.iter().all(|r| roads(&wm, &cx).contains(&(r.clone(), "alt".to_string()))), "the other way pale");
        // the row's hover previews, its click picks
        let at = centre(&wm, "world_map.way.1");
        hover(&mut wm, &mut cx, &mut g, at);
        assert!(pale.iter().all(|r| roads(&wm, &cx).contains(&(r.clone(), "peek".to_string()))), "previewed: {:?}", roads(&wm, &cx));
        wm.click_id(&mut cx, "world_map.way.1").expect("the second way's row");
        frame(&mut wm, &mut cx, &mut g, 1.0);
        assert_eq!(wm.state(&cx)["pick"], json!(1));
        assert_eq!(main_of(&roads(&wm, &cx)), ways[1].routes, "now the main path");
        let on_map = wm.widgets().into_iter().find(|w| w["id"].as_str().is_some_and(|i| i.starts_with("world_map.way.0.at."))).expect("the first way's road takes a click");
        wm.click_id(&mut cx, on_map["id"].as_str().unwrap()).expect("its road");
        frame(&mut wm, &mut cx, &mut g, 1.0);
        assert_eq!(main_of(&roads(&wm, &cx)), ways[0].routes, "picked back on the map");
    }

    /// Travel goes along the picked way (`travel::travel_via` with its routes), not the cheapest.
    #[test]
    fn travel_takes_the_picked_way() {
        let mut cx = ward_hub(&TIER2, &["drowned_runoff_grate", "alleyway_junction_7", "ash_crawl_crossroads"], false);
        let (d, vats) = (cx.data.clone(), q("tallow_vats"));
        let (mut wm, mut g) = (WorldMap::new(), gfx());
        frame(&mut wm, &mut cx, &mut g, 1.0);
        wm.click_id(&mut cx, &node_id(&vats)).expect("the vats");
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let ways = travel::paths(&d, cx.gs(), &cx.gs().location_id, &vats);
        assert!(ways.len() >= 2 && ways[1].routes != ways[0].routes);
        let before = cx.gs().clone();
        let (mut via, mut cheap) = (before.clone(), before.clone());
        let want = travel::travel_via(&d, &mut via, &vats, &ways[1].routes);
        let first = travel::travel(&d, &mut cheap, &vats);
        wm.click_id(&mut cx, "world_map.way.1").expect("the second way");
        wm.click_id(&mut cx, "world_map.travel").expect("travel");
        let gs = cx.gs();
        assert_eq!((gs.time, gs.location_id.clone(), gs.party.supplies), (via.time, via.location_id.clone(), via.party.supplies), "the trip of the picked way");
        if s(&want, "status") == "arrived" && s(&first, "status") == "arrived" && ways[1].days != ways[0].days {
            assert_ne!(gs.time, cheap.time, "not the cheapest way's days");
        }
    }

    /// A token of map text that is a backend id or code: a snake_case id, a qid, a mission or job code ("M1", "JOB"),
    /// a node number ("Node 3", "03"), or an id of the world.
    fn backend_code<'a>(t: &'a str, ids: &[String]) -> Option<&'a str> {
        let words: Vec<&str> = t.split(|c: char| c.is_whitespace() || ",.()!?;\"·".contains(c)).filter(|w| !w.is_empty()).collect();
        for (i, w) in words.iter().enumerate() {
            let code = w.contains('_')
                || w.trim_end_matches(':').contains(':')
                || (w.len() >= 2 && w.starts_with('M') && w[1..].chars().all(|c| c.is_ascii_digit()))
                || *w == "JOB"
                || (*w == "Node" && words.get(i + 1).is_some_and(|n| n.chars().all(|c| c.is_ascii_alphanumeric())))
                || (w.len() == 2 && w.starts_with('0') && w.chars().all(|c| c.is_ascii_digit()))
                || ids.iter().any(|id| id == w);
            if code {
                return Some(w);
            }
        }
        None
    }

    /// AO-01 (5): no backend id or code in any text the world map shows (drawn texts, widget texts, hover cards) on
    /// the shipped Sunk Ward at several points of Act 1, at rest, with a place hovered, a destination picked and the
    /// boards opened, the Legend open or minimized.
    #[test]
    fn the_map_shows_no_backend_ids_or_codes() {
        let probe = ward_hub(&[], &[], false);
        let d = probe.data.clone();
        let mut ids: Vec<String> = Vec::new();
        for kind in ["locations", "routes", "quests", "encounters", "regions", "boards", "characters"] {
            ids.extend(d.merged_entries(SW, kind).iter().map(|e| s(e, "id")));
        }
        assert!(backend_code("Alleyway Junction 7", &ids).is_none() && backend_code("Trip: 1 day, 1 Tallow", &ids).is_none());
        assert!(["M1 · Tier 1", "sunk_ward:slag_gate", "JOB", "Node 07", "03 · The Kiln"].iter().all(|t| backend_code(t, &ids).is_some()), "the scan finds codes");
        let all: Vec<&str> = TIER2.iter().copied().chain(["sunk_ward.interlude_done", "sunk_ward.m5_done"]).collect();
        let every = ["drowned_runoff_grate", "alleyway_junction_7", "ash_crawl_crossroads", "fourth_kiln", "tallow_vats", "crucible_watchpost", "corvos_gate"];
        let scenes: [(&[&str], &[&str], &str, bool); 4] = [
            (&[], &["drowned_runoff_grate", "corvos_gate"], "alleyway_junction_7", false),
            (&TIER2, &["drowned_runoff_grate", "alleyway_junction_7", "ash_crawl_crossroads"], "tallow_vats", true),
            (&all, &every, "crucible_watchpost", true),
            (&all, &every, "slag_gate", false),
        ];
        let mut seen = 0;
        for (flags, visited, pick, boards) in scenes {
            let mut cx = ward_hub(flags, visited, false);
            if boards {
                open_boards(&mut cx);
            }
            let (mut wm, mut g) = (WorldMap::new(), gfx());
            frame(&mut wm, &mut cx, &mut g, 1.0);
            let hovered = centre(&wm, &node_id(&q("drowned_runoff_grate")));
            wm.click_id(&mut cx, &node_id(&q(pick))).expect("the pick");
            for minimized in [false, true] {
                if minimized {
                    wm.click_id(&mut cx, "world_map.legend.min").expect("minimize");
                }
                for mouse in [(-500, -500), hovered] {
                    wm.draw(&mut cx, &mut g, mouse, true);
                    g.fonts.audit = Some(Vec::new());
                    wm.draw(&mut cx, &mut g, mouse, true);
                    let mut texts: Vec<String> = g.fonts.audit.take().unwrap_or_default().into_iter().map(|r| r.text).collect();
                    for w in wm.widgets() {
                        texts.push(s(json::dict(&w), "text"));
                        texts.extend(json::aget(json::dict(&w), "tip").iter().map(|l| l.as_array().and_then(|a| a.first()).and_then(Value::as_str).unwrap_or("").to_string()));
                    }
                    for t in &texts {
                        assert!(backend_code(t, &ids).is_none(), "'{t}' shows '{}' ({pick}, {flags:?})", backend_code(t, &ids).unwrap());
                    }
                    seen += texts.len();
                }
            }
        }
        assert!(seen > 100, "the scan read the map's texts: {seen}");
    }

    /// AO-01 (5): a name sits above its mark; one that would leave the map's top is pinned to the top edge, still over
    /// the mark (never below or beside it); on the shipped map every name is drawn above its place.
    #[test]
    fn a_name_sits_above_its_mark_and_pins_to_the_top_edge_only_when_clipped() {
        assert_eq!(label_rect(200.0, 100, 60, 9, MAP), Rect::new(170, 91, 60, 9));
        let pinned = label_rect(200.0, MAP.y + 5, 60, 9, MAP);
        assert_eq!((pinned.x, pinned.y), (170, MAP.y + 1), "pinned to the top edge over the mark");
        assert_eq!(label_rect(MAP.x as f32, 100, 60, 9, MAP).x, MAP.x + 1, "kept on the map side to side");
        let all: Vec<&str> = TIER2.iter().copied().chain(["sunk_ward.interlude_done", "sunk_ward.m5_done"]).collect();
        let mut cx = ward_hub(&all, &["drowned_runoff_grate", "alleyway_junction_7", "ash_crawl_crossroads", "fourth_kiln", "tallow_vats", "crucible_watchpost"], false);
        let (mut wm, mut g) = (WorldMap::new(), gfx());
        let recs = frame(&mut wm, &mut cx, &mut g, 1.0);
        let nodes: Vec<Value> = wm.widgets().into_iter().filter(|w| w["kind"] == "node").collect();
        assert!(nodes.len() >= 8, "every Act 1 place: {}", nodes.len());
        for n in nodes {
            let (name, r) = (s(json::dict(&n), "text"), wm.widget_rect(n["id"].as_str().unwrap()).unwrap().0);
            let drawn: Vec<_> = recs.iter().filter(|t| t.text == name && t.x < (MAP.x + MAP.w) as f32 && t.y >= MAP.y as f32).collect();
            assert!(!drawn.is_empty(), "{name} is drawn");
            for t in drawn {
                assert!(t.y + t.h <= r.y as f32 + 0.5 && t.x < (r.x + r.w) as f32 && t.x + t.w > r.x as f32, "{name} above its mark: {:?} over {r:?}", (t.x, t.y, t.w, t.h));
            }
        }
    }

    /// AO-01 (2): a locked place's mark is grey (85 % greyscale), at 85 % brightness and 80 % opacity.
    #[test]
    fn a_locked_mark_is_grey_not_dark() {
        let img = Image { w: 2, h: 1, px: vec![200, 40, 40, 255, 40, 160, 220, 200] };
        let out = locked_look(&img);
        for (p, o) in out.px.chunks(4).zip(img.px.chunks(4)) {
            let lum = |c: &[u8]| 0.299 * c[0] as f32 + 0.587 * c[1] as f32 + 0.114 * c[2] as f32;
            let spread = |c: &[u8]| (*c[..3].iter().max().unwrap() as f32) - (*c[..3].iter().min().unwrap() as f32);
            assert!(spread(p) <= spread(o) * 0.15 + 1.0, "grey: {p:?} from {o:?}");
            assert!((lum(p) - 0.85 * lum(o)).abs() <= 1.5, "85 % brightness: {p:?} from {o:?}");
            assert_eq!(p[3], (o[3] as f32 * 0.8).round() as u8, "80 % opacity");
        }
    }

    /// Reduce motion holds the quest badge's glow and the lantern flicker still; the flicker runs only when the game
    /// turns it on (Emberwake does not), each road on its own beat.
    #[test]
    fn reduce_motion_holds_the_badge_glow_and_the_flicker() {
        let samples: Vec<u64> = (0..12).map(|k| k * 271).collect();
        assert!(samples.iter().all(|&ms| pulse(ms, true) == pulse(0, true)), "still under reduce motion");
        let moving: Vec<f32> = samples.iter().map(|&ms| pulse(ms, false)).collect();
        assert!(moving.iter().any(|v| (v - moving[0]).abs() > 0.2) && moving.iter().all(|v| (0.0..=1.0).contains(v)), "{moving:?}");
        for &ms in &samples {
            for k in 0..3 {
                assert_eq!((flicker(ms, k, false, false), flicker(ms, k, true, true)), (1.0, 1.0), "off, or reduce motion: steady");
            }
        }
        let lit: Vec<f32> = samples.iter().map(|&ms| flicker(ms, 0, true, false)).collect();
        assert!(lit.iter().any(|v| *v < 0.9) && lit.iter().all(|v| (0.55..=1.0).contains(v)), "{lit:?}");
        assert_ne!(flicker(1000, 0, true, false), flicker(1000, 1, true, false), "roads flicker apart");
        let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        assert!(!Data::open(&dir).expect("data").0.world_map_lantern_flicker, "Emberwake leaves the flicker off");
    }

    /// AO-02 (`preferences/ui.md`): a floating panel starts in a free corner (top right first), or in the middle and
    /// unlocked when every corner is taken; minimize and restore toggle in place; it drags only when unlocked, holding
    /// the grab point and staying inside its area; its minimized button drags the panel's place along.
    #[test]
    fn a_floating_panel_starts_free_toggles_in_place_and_drags_only_unlocked() {
        let (area, size, chip) = (Rect::new(0, 0, 400, 300), (100, 60), (40, 12));
        let m = PANEL_MARGIN;
        let placed = |taken: &[Rect]| {
            let mut p = FloatPanel::default();
            p.place(area, size, taken);
            p
        };
        let p = placed(&[]);
        assert_eq!((p.pos, p.locked), (Some((400 - 100 - m, m)), true), "the upper right");
        assert_eq!(placed(&[Rect::new(350, 10, 5, 5)]).pos, Some((m, m)), "then the upper left");
        assert_eq!(placed(&[Rect::new(350, 10, 5, 5), Rect::new(10, 10, 5, 5)]).pos, Some((400 - 100 - m, 300 - 60 - m)), "then the lower right");
        let corners = [Rect::new(350, 10, 5, 5), Rect::new(10, 10, 5, 5), Rect::new(350, 280, 5, 5), Rect::new(10, 280, 5, 5)];
        let p = placed(&corners);
        assert_eq!((p.pos, p.locked), (Some((150, 120)), false), "every corner taken: the middle, unlocked");
        let mut p = placed(&[]);
        p.place(area, size, &corners);
        assert_eq!(p.pos, Some((400 - 100 - m, m)), "placed once");
        // minimize and restore in place
        let min = FloatPanel::min_button(p.rect(size));
        let at = p.pos;
        p.minimize();
        let c = p.shown(size, chip);
        assert_eq!((c.x + c.w, c.y, c.w, c.h), (min.x + min.w, min.y - 1, chip.0, chip.1), "the button's top right where minimize was");
        p.restore(area, size);
        assert_eq!((p.pos, p.minimized), (at, false), "open again where it was");
        // locked: a drag moves nothing
        let now = p.shown(size, chip);
        p.follow((now.x + 20, now.y + 5), false, now, area);
        p.follow((50, 200), true, p.shown(size, chip), area);
        assert_eq!(p.pos, at, "locked");
        // unlocked: the grab point stays under the pointer, inside the area
        p.follow((0, 0), false, p.shown(size, chip), area);
        p.toggle_lock();
        let now = p.shown(size, chip);
        p.follow((now.x + 20, now.y + 5), false, now, area);
        p.follow((150, 200), true, p.shown(size, chip), area);
        assert_eq!(p.pos, Some((130, 195)));
        p.follow((-500, -500), true, p.shown(size, chip), area);
        assert_eq!(p.pos, Some((2, 2)), "kept inside");
        p.follow((-500, -500), false, p.shown(size, chip), area);
        // the minimized button drags the panel's place along
        p.pos = Some((200, 100));
        p.minimize();
        let c = p.shown(size, chip);
        p.follow((c.x + 5, c.y + 5), false, c, area);
        p.follow((c.x + 35, c.y + 45), true, p.shown(size, chip), area);
        assert_eq!((p.pos, p.shown(size, chip)), (Some((230, 140)), Rect::new(c.x + 30, c.y + 40, c.w, c.h)));
    }

    /// The Legend on the shipped Act 1 map: in the free upper right and locked; it lists the opened boards' quests and
    /// "Your party"; locked it does not drag, unlocked its body drags (the shell's drag of `LEGEND_DRAG`) and stays where
    /// it is dropped; minimized, its "Legend" button sits where the minimize button was and opens it in place.
    #[test]
    fn the_legend_starts_upper_right_drags_when_unlocked_and_minimizes_in_place() {
        let mut cx = ward_hub(&TIER2, &["drowned_runoff_grate", "alleyway_junction_7", "ash_crawl_crossroads"], false);
        open_boards(&mut cx);
        let (mut wm, mut g) = (WorldMap::new(), gfx());
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let r = wm.widget_rect("world_map.legend").expect("the Legend").0;
        assert_eq!((r.x + r.w, r.y, wm.legend.locked), (MAP.x + MAP.w - PANEL_MARGIN, MAP.y + PANEL_MARGIN, true), "upper right, locked");
        assert!(widget(&wm, "world_map.legend").unwrap().get("drag").is_none(), "locked: no drag");
        let rows: Vec<String> = wm.widgets().iter().filter(|w| w["id"].as_str().is_some_and(|i| i.starts_with("world_map.legend.quest."))).map(|w| s(json::dict(w), "text")).collect();
        let jobs = emberwake_world::board::jobs(&cx.data, cx.gs());
        assert!(!jobs.is_empty() && jobs.iter().all(|(_, name)| rows.contains(name)), "the boards' quests: {rows:?}");
        assert_eq!(widget(&wm, "world_map.legend.party").map(|w| w["text"].clone()), Some(json!("Your party")));
        // a drag while locked moves nothing
        let held = aegis_ui::ui::Drag { key: LEGEND_DRAG.into(), id: "world_map.legend".into(), text: "Legend".into() };
        let grab = (r.x + 10, r.y + 4);
        wm.draw(&mut cx, &mut g, grab, true);
        g.kit.drag = Some(held.clone());
        wm.draw(&mut cx, &mut g, (grab.0 - 200, grab.1 + 100), true);
        g.kit.drag = None;
        frame(&mut wm, &mut cx, &mut g, 1.0);
        assert_eq!(wm.widget_rect("world_map.legend").unwrap().0, r, "locked: stays");
        // unlocked: the body drags and stays where it was dropped
        wm.click_id(&mut cx, "world_map.legend.lock").expect("unlock");
        frame(&mut wm, &mut cx, &mut g, 1.0);
        assert_eq!(widget(&wm, "world_map.legend").unwrap()["drag"], json!(LEGEND_DRAG));
        wm.draw(&mut cx, &mut g, grab, true);
        g.kit.drag = Some(held);
        wm.draw(&mut cx, &mut g, (grab.0 - 200, grab.1 + 100), true);
        g.kit.drag = None;
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let moved = wm.widget_rect("world_map.legend").unwrap().0;
        assert_eq!((moved.x, moved.y), (r.x - 200, r.y + 100), "dropped there");
        // minimize toggles in place
        let min = wm.widget_rect("world_map.legend.min").unwrap().0;
        wm.click_id(&mut cx, "world_map.legend.min").expect("minimize");
        frame(&mut wm, &mut cx, &mut g, 1.0);
        assert!(widget(&wm, "world_map.legend").is_none());
        let chip = widget(&wm, "world_map.legend.open").expect("the Legend button");
        assert_eq!(chip["text"], json!("Legend"));
        assert_eq!(chip["drag"], json!(LEGEND_DRAG), "unlocked: the button drags too");
        let c = wm.widget_rect("world_map.legend.open").unwrap().0;
        assert_eq!((c.x + c.w, c.y), (min.x + min.w, min.y - 1), "where the minimize button was");
        wm.click_id(&mut cx, "world_map.legend.open").expect("open");
        frame(&mut wm, &mut cx, &mut g, 1.0);
        assert_eq!(wm.widget_rect("world_map.legend.min").unwrap().0, min, "restored in place");
    }

    /// Every corner of the map holding a place: the Legend starts in the middle, unlocked (AO-01 (5)).
    #[test]
    fn the_legend_starts_unlocked_in_the_middle_when_every_corner_is_taken() {
        let w = "ward";
        let spots = [("hub", 0.5, 0.5), ("nw", 0.0, 0.0), ("ne", 1.0, 0.0), ("sw", 0.0, 1.0), ("se", 1.0, 1.0)];
        let locations: Vec<Value> = spots.iter().map(|(id, x, y)| json!({"id": id, "name": format!("Place {}", id.to_uppercase()), "map_pos": [x, y]})).collect();
        let routes: Vec<Value> = spots[1..].iter().map(|(id, ..)| json!({"id": format!("to_{id}"), "name": "A road", "from": "hub", "to": id, "time": 1, "encounter_pct": 0})).collect();
        let worlds = json!({w: {"manifest": {"id": w, "name": "Ward", "start_location": "hub", "start_party": ["hero"]}, "own": {
            "locations": with_ids(w, Value::Array(locations)), "routes": with_ids(w, Value::Array(routes))
        }}});
        let catalog = json!({"classes": [{"id": "fighter", "name": "Fighter"}], "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]});
        let d = Data::from_values(&catalog, &worlds, &json!({}));
        let mut gs = state(&d, "ward:hub");
        gs.set_flag(&explore::visited_flag("ward:hub"), json!(true));
        let mut cx = crate::story::tests::story_cx(d);
        crate::screens::enter_game(&mut cx, gs, -1, Vec::new());
        cx.nav.clear();
        let (mut wm, mut g) = (WorldMap::new(), gfx());
        frame(&mut wm, &mut cx, &mut g, 1.0);
        assert_eq!(wm.widgets().iter().filter(|w| w["kind"] == "node").count(), 5, "every place is on the map");
        let r = wm.widget_rect("world_map.legend").expect("the Legend").0;
        assert!(((r.x + r.w / 2) - (MAP.x + MAP.w / 2)).abs() <= 1 && ((r.y + r.h / 2) - (MAP.y + MAP.h / 2)).abs() <= 1, "in the middle: {r:?}");
        assert!(!wm.legend.locked, "unlocked, ready to drag");
        assert_eq!(widget(&wm, "world_map.legend").unwrap()["drag"], json!(LEGEND_DRAG));
    }

    /// The Legend lists an active quest by its name and picks its place (`travel::quest_places`).
    #[test]
    fn the_legend_lists_an_active_quest_and_picks_its_place() {
        let mut cx = ward_hub(&[], &["drowned_runoff_grate"], false);
        open_boards(&mut cx);
        let d = cx.data.clone();
        let (mut wm, mut g) = (WorldMap::new(), gfx());
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let shown: Vec<String> = wm.widgets().iter().filter(|w| w["kind"] == "node").map(|w| s(json::dict(w), "id")).collect();
        // a job whose place is on the map, taken: it becomes an active quest there
        let boards: Vec<String> = d.merged_entries(SW, "boards").iter().map(|b| s(b, "_qid")).collect();
        let (qid, node) = boards
            .iter()
            .flat_map(|b| emberwake_world::board::quests(&d, cx.gs(), b))
            .map(|qq| (qq.clone(), emberwake_world::board::node_of(&d, cx.gs(), &qq)))
            .find(|(qq, n)| emberwake_world::quest::status(&d, cx.gs(), qq) == "available" && shown.contains(&node_id(n)) && *n != cx.gs().location_id)
            .expect("a job at a place on the map");
        assert!(emberwake_world::quest::start(&d, cx.gs_mut(), &qid).is_empty());
        let name = s(&emberwake_world::quest::resolved(&d, cx.gs(), &qid), "name");
        let places = travel::quest_places(&d, cx.gs());
        assert!(places.contains(&(node.clone(), name.clone())), "{places:?}");
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let row = wm.widgets().into_iter().find(|w| w["id"].as_str().is_some_and(|i| i.starts_with("world_map.legend.quest.")) && w["text"] == json!(name)).expect("its row");
        wm.click_id(&mut cx, row["id"].as_str().unwrap()).expect("the row");
        assert_eq!(wm.state(&cx)["selected"], json!(node), "the row picks its place");
    }

    // ---- PR5-48 (MAP-LAYOUT-ED) -----------------------------------------------------------------------------------

    fn game_layers() -> Layers {
        Layers::for_game(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
    }

    fn close(a: [f32; 2], b: (f32, f32)) -> bool {
        (a[0] - b.0).abs() < 1e-4 && (a[1] - b.1).abs() < 1e-4
    }

    /// Map Layout offers every world map sidecar and background of the layers and opens each in the editor: a sidecar's
    /// places as markers at the sidecar's landmarks (else `map_pos`), its routes as polylines with their waypoints, on the
    /// world map's viewport and landmark box; a pre-staged act (a background, no data) opens with no places.
    #[test]
    fn map_layout_opens_each_world_map_with_its_places_and_roads() {
        let mut cx = ward_hub(&[], &[], true);
        let layers = game_layers();
        let names = layout_names(&layers);
        for n in world_map::sidecar_names(&layers).iter().chain(world_map::background_names(&layers).iter()) {
            assert!(names.contains(n), "{n} is offered");
        }
        let mut g = gfx();
        for name in &names {
            let mut m = MapLayoutScreen::new(&mut cx, Some(name));
            frame(&mut m, &mut cx, &mut g, 1.0);
            assert_eq!(m.state(&cx)["name"], json!(name));
            let (world, places) = layout_places(&cx.data, &layers, name);
            let e = &m.edit.editor;
            assert_eq!(e.markers().len(), places.len(), "{name}: one marker per place");
            assert_eq!(e.view(), Viewport::new(MAP, (MAP_PAD.0 as f32, MAP_PAD.1 as f32)));
            assert_eq!(e.marker_box, MarkerBox { side: LANDMARK_PX, foot: LANDMARK_FOOT });
            for mk in e.markers() {
                assert!(m.widget_rect(&format!("layout.marker.{}", mk.id)).is_some(), "{name}: {} has a handle", mk.id);
            }
            match world_map::owner(&cx.data.catalogs, &layers, name) {
                Owner::PreStaged => assert!(places.is_empty() && world.is_none(), "{name}: a pre-staged act has no places"),
                Owner::Unknown => panic!("{name} is offered but is nothing"),
                _ => assert!(!places.is_empty(), "{name}: a world or region with places"),
            }
            let Some(layout) = read_layout(&layers, std::slice::from_ref(name)) else { continue };
            for (id, mark) in &layout.landmarks {
                let mk = e.marker(id).unwrap_or_else(|| panic!("{name}: the sidecar's {id} is a marker"));
                assert!(close(mk.pos, mark.pos) && (mk.scale - mark.scale).abs() < 1e-4, "{name}: {id} at its sidecar place");
            }
            for (id, pts) in &layout.routes {
                let l = e.polyline(id).unwrap_or_else(|| panic!("{name}: the sidecar's route {id} is a polyline"));
                assert_eq!(l.points.len(), pts.len(), "{name}: {id}");
                assert!(l.points.iter().zip(pts).all(|(a, b)| close(*a, *b)), "{name}: {id}'s waypoints");
            }
            assert!(!e.dirty(), "{name}: opening changes nothing");
        }
        assert!(names.iter().any(|n| read_layout(&layers, std::slice::from_ref(n)).is_some()), "a world map with a sidecar (Act 1)");
    }

    /// The live map's layout editing is a debug tool: its button and F7 exist only in a debug build with the dev tools
    /// (`crate::DEV_KEYS`); there it edits the sidecar the map reads in place (the editor's handles over the real marks,
    /// no backdrop, the arrows nudging, its state on the agent link), the map drawing by the editor's layout, and Esc ends
    /// it.
    #[test]
    fn the_live_map_layout_toggle_is_debug_only() {
        let mut cx = ward_hub(&[], &[], true);
        let mut g = gfx();
        let mut wm = WorldMap::new();
        frame(&mut wm, &mut cx, &mut g, 1.0);
        assert_eq!(widget(&wm, "world_map.layout_edit").is_some(), crate::DEV_KEYS, "the button only in a debug build");
        wm.key(&mut cx, Key::Fn(7));
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let st = wm.state(&cx);
        if !crate::DEV_KEYS {
            assert!(st["layout_edit"].is_null() && wm.arrow_focus(), "a release build takes F7 and does nothing");
            return;
        }
        let name = st["layout_edit"].as_str().expect("editing").to_string();
        assert!(map_names(&cx.data, cx.gs()).contains(&name) && cx.art.has(&emberwake_content::art::world_map_layout(&name)), "the sidecar the map reads");
        assert!(!wm.arrow_focus());
        let e = &wm.edit.as_ref().unwrap().editor;
        assert!(!e.backdrop && !e.labels, "over the real map and its real names");
        // the party's place, picked by rule: its marker's handle over its mark; a move through the agent link moves the mark
        let here = cx.gs().location_id.clone();
        let id = local_id(&here).to_string();
        assert!(widget(&wm, &format!("layout.marker.{id}")).is_some() && widget(&wm, &node_id(&here)).is_some());
        let reply = wm.command(&cx, &json!({"cmd": "layout", "id": id, "pos": [0.25, 0.5]})).expect("the editor's op");
        assert_eq!(reply["ok"], json!(true), "{reply}");
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let (x, y, w, h) = anchor_rect();
        let (r, _) = wm.widget_rect(&node_id(&here)).unwrap();
        let landmark = cx.art.has(&emberwake_content::art::landmark(&here));
        let scale = wm.edit.as_ref().unwrap().editor.marker(&id).unwrap().scale;
        let want = if landmark { landmark_box(x + 0.25 * w, y + 0.5 * h, scale) } else { pin_box(x + 0.25 * w, y + 0.5 * h) };
        assert_eq!(r, want, "the mark follows the editor");
        assert_eq!(wm.state(&cx)[aegis_ui::layout_editor::STATE_KEY]["dirty"], json!(true));
        wm.escape(&mut cx);
        assert!(wm.edit.is_none() && cx.nav.is_empty(), "Esc ends the editing, the map stays");
        assert!(cx.log.iter().any(|l| l.contains("not saved were dropped")), "the unsaved edit is dropped and said");
    }

    /// A scratch asset layer holding the game's world map sidecars (the backgrounds as stand-in files).
    fn scratch_layer(s: &Scratch) -> std::path::PathBuf {
        let root = s.path().join("assets");
        let to = root.join(world_map::DIR);
        std::fs::create_dir_all(&to).unwrap();
        for e in std::fs::read_dir(game_layers().roots[0].join(world_map::DIR)).unwrap() {
            let p = e.unwrap().path();
            match p.extension().and_then(|x| x.to_str()) {
                Some("json") => {
                    std::fs::copy(&p, to.join(p.file_name().unwrap())).unwrap();
                }
                Some("png") => std::fs::write(to.join(p.file_name().unwrap()), "png").unwrap(),
                _ => {}
            }
        }
        root
    }

    /// Save in the editor writes through the kit's writer into the game's asset layer (a scratch copy here, never the
    /// shipped files), acknowledges the values and reads the art afresh, so the world map draws the saved place; a sidecar
    /// the writer refuses is kept as it was, the values stay unsaved and the problems show. A build that may not write
    /// (`writes_allowed`) saves nothing.
    #[test]
    fn a_save_writes_the_sidecar_through_the_kit_writer_and_the_map_reads_it() {
        let s = Scratch::new("pr548_save");
        let root = scratch_layer(&s);
        let mut cx = ward_hub(&[], &[], false);
        cx.art = aegis_ui::art::Art::layered(Layers::of(vec![root.clone()]));
        let name = map_names(&cx.data, cx.gs()).into_iter().find(|n| root.join(emberwake_content::art::world_map_layout(n)).exists()).expect("the hub's map has a sidecar");
        let path = root.join(emberwake_content::art::world_map_layout(&name));
        let shipped = std::fs::read_to_string(&path).unwrap();
        let here = cx.gs().location_id.clone();
        let id = local_id(&here).to_string();
        let mut m = MapLayoutScreen::new(&mut cx, Some(&name));
        let mut g = gfx();
        frame(&mut m, &mut cx, &mut g, 1.0);
        assert_eq!(m.command(&cx, &json!({"cmd": "layout", "id": id, "pos": [0.3125, 0.4375]})).unwrap()["ok"], json!(true));
        let generation = cx.art.generation();
        m.act(&mut cx, LAct::Edit(LayoutAct::Save));
        if !aegis_ui::layout_editor::writes_allowed() {
            assert_eq!(std::fs::read_to_string(&path).unwrap(), shipped, "a release build writes nothing");
            return;
        }
        let saved: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(saved["landmarks"][&id]["pos"], json!([0.3125, 0.4375]));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), world_map::text(&saved), "the writer's canonical text");
        assert!(!m.edit.editor.dirty() && m.edit.editor.note.starts_with("saved "));
        assert_ne!(cx.art.generation(), generation, "the art is read afresh");
        // the world map reads the saved sidecar: the party's mark stands at the saved place
        let mut wm = WorldMap::new();
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let (x, y, w, h) = anchor_rect();
        assert_eq!(wm.widget_rect(&node_id(&here)).unwrap().0, pin_box(x + 0.3125 * w, y + 0.4375 * h), "no landmark image in the scratch layer: the pin");
        // a sidecar the writer refuses (a landmark no place of the world has): nothing written, still unsaved, said
        let mut broken = saved.clone();
        broken["landmarks"]["no_such_place"] = json!({"pos": [0.5, 0.5], "scale": 1.0});
        std::fs::write(&path, broken.to_string()).unwrap();
        m.command(&cx, &json!({"cmd": "layout", "id": id, "pos": [0.5, 0.5]})).unwrap();
        cx.nav.clear();
        m.act(&mut cx, LAct::Edit(LayoutAct::Save));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), broken.to_string(), "nothing written");
        assert!(m.edit.editor.dirty() && m.edit.editor.note.contains("refused"));
        assert!(matches!(cx.nav.as_slice(), [Nav::Push(n)] if n.id() == "notice"), "the problems show");
    }

    /// PR5-48 review renders at 1280 x 720 with the game's art and theme, only with `AEGIS_REVIEW_DIR` set: Map Layout on
    /// Act 1's sidecar (a landmark selected), on a pre-staged act, and the live map edited in place (a debug build).
    #[test]
    fn review_shots_of_the_map_layout_editor() {
        let Ok(out) = std::env::var("AEGIS_REVIEW_DIR") else { return };
        std::fs::create_dir_all(&out).unwrap();
        let shot = |screen: &mut dyn Screen<GameCx>, cx: &mut Cx, name: &str| {
            let theme = aegis_ui::theme::load(cx.art.layers()).expect("the game's theme");
            let mut g = gfx();
            g.fonts.sync(&theme);
            aegis_ui::theme::set_theme(theme);
            g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
            g.view = aegis_ui::painter::View::fit(1280, 720);
            for _ in 0..2 {
                screen.draw(cx, &mut g, (-500, -500), true);
            }
            let file = std::fs::File::create(Path::new(&out).join(format!("{name}.png"))).unwrap();
            let mut enc = png::Encoder::new(std::io::BufWriter::new(file), 1280, 720);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header().unwrap().write_image_data(&g.canvas.px).unwrap();
            println!("{name}: {}", screen.state(cx)[aegis_ui::layout_editor::STATE_KEY]["note"]);
        };
        let mut cx = ward_hub(&[], &[], true);
        let here = cx.gs().location_id.clone();
        let mut m = MapLayoutScreen::new(&mut cx, Some("sunk_ward_quarter"));
        let i = m.edit.editor.markers().iter().position(|mk| mk.id == local_id(&here)).unwrap_or(0);
        m.act(&mut cx, LAct::Edit(LayoutAct::Select(Some(aegis_ui::layout_editor::Sel::Marker(i)))));
        shot(&mut m, &mut cx, "1_map_layout_sunk_ward_quarter");
        let staged = layout_names(cx.art.layers()).into_iter().find(|n| world_map::owner(&cx.data.catalogs, cx.art.layers(), n) == Owner::PreStaged);
        if let Some(n) = staged {
            let mut m = MapLayoutScreen::new(&mut cx, Some(&n));
            shot(&mut m, &mut cx, &format!("2_map_layout_{n}"));
        }
        if crate::DEV_KEYS {
            let mut wm = WorldMap::new();
            frame(&mut wm, &mut cx, &mut gfx(), 1.0);
            wm.key(&mut cx, Key::Fn(7));
            shot(&mut wm, &mut cx, "3_live_world_map_editing");
        }
    }

    /// Review renders of the world map on the shipped Sunk Ward at 1280 x 720 with the game's art and theme: only with
    /// `AEGIS_REVIEW_DIR` set (MAP-HOOKS1 proof shots).
    #[test]
    fn review_shots_of_the_world_map() {
        let Ok(out) = std::env::var("AEGIS_REVIEW_DIR") else { return };
        std::fs::create_dir_all(&out).unwrap();
        let shot = |wm: &mut WorldMap, cx: &mut Cx, name: &str, mouse: (i32, i32)| {
            let theme = aegis_ui::theme::load(cx.art.layers()).expect("the game's theme");
            let mut g = gfx();
            g.fonts.sync(&theme);
            aegis_ui::theme::set_theme(theme);
            g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
            g.view = aegis_ui::painter::View::fit(1280, 720);
            hover(wm, cx, &mut g, mouse);
            let file = std::fs::File::create(Path::new(&out).join(format!("{name}.png"))).unwrap();
            let mut enc = png::Encoder::new(std::io::BufWriter::new(file), 1280, 720);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header().unwrap().write_image_data(&g.canvas.px).unwrap();
            println!("{name}: {}", wm.state(cx)["roads"]);
        };
        let off = (-500, -500);
        // Tier 1 open: the hub and the grate visited; Corvo's gate seen once but its road still shut (a locked place)
        let mut cx = ward_hub(&[], &["drowned_runoff_grate", "corvos_gate"], true);
        let mut wm = WorldMap::new();
        shot(&mut wm, &mut cx, "1_rest_tier1", off);
        let at = centre(&wm, &node_id(&q("drowned_runoff_grate")));
        shot(&mut wm, &mut cx, "2_hover_grate_tier1", at);
        // Tier 2 open: M1, M2 and the lock ciphers done, the Tier 1 places visited; the Tallow Vats picked
        let mut cx = ward_hub(&TIER2, &["drowned_runoff_grate", "alleyway_junction_7", "ash_crawl_crossroads"], true);
        open_boards(&mut cx);
        let mut wm = WorldMap::new();
        shot(&mut wm, &mut cx, "3_rest_tier2_jobs", off);
        wm.click_id(&mut cx, &node_id(&q("tallow_vats"))).expect("the vats");
        shot(&mut wm, &mut cx, "4_selected_vats_gold_and_pale", off);
        let row = centre(&wm, "world_map.way.1");
        shot(&mut wm, &mut cx, "5_selected_vats_peek_second_way", row);
        wm.click_id(&mut cx, "world_map.legend.min").expect("minimize");
        shot(&mut wm, &mut cx, "6_legend_minimized", off);
        wm.click_id(&mut cx, "world_map.legend.open").expect("open");
        frame(&mut wm, &mut cx, &mut gfx(), 1.0);
        wm.click_id(&mut cx, "world_map.legend.lock").expect("unlock");
        let kiln = centre(&wm, &node_id(&q("fourth_kiln")));
        shot(&mut wm, &mut cx, "7_legend_unlocked_hover_kiln", kiln);
        // every road open: the Interlude and M5 done
        let all: Vec<&str> = TIER2.iter().copied().chain(["sunk_ward.interlude_done", "sunk_ward.m5_done"]).collect();
        let mut cx = ward_hub(&all, &["drowned_runoff_grate", "alleyway_junction_7", "ash_crawl_crossroads", "fourth_kiln", "tallow_vats", "crucible_watchpost"], true);
        let mut wm = WorldMap::new();
        frame(&mut wm, &mut cx, &mut gfx(), 1.0);
        wm.click_id(&mut cx, &node_id(&q("crucible_watchpost"))).expect("the crucible");
        shot(&mut wm, &mut cx, "8_all_open_crucible_picked", off);
        println!("legend rows: {:?}", wm.widgets().iter().filter(|w| w["id"].as_str().is_some_and(|i| i.starts_with("world_map.legend"))).map(|w| w["text"].clone()).collect::<Vec<_>>());
    }
}
