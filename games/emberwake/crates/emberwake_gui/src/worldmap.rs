//! The world map (scripts/ui/overworld/world_map_view.gd): known nodes and routes of the party's world
//! (TravelRules.view), trips by clicking a node (TravelRunner.travel: clock, supplies, road fights), making camp
//! (HealRules.make_camp) and entering the location the party stands on. Esc / Back returns to the region map.
//! STORYUI1-U1: like the region map, it hands over to the story stage and plays queued vignettes (`story::map_tick`).
//! CAMPAIGN1-U: campaign notices and the run-over screen (`runover::map_tick`); a trip or a camp whose days would end
//! the run asks first (W3, No as the default); a trip the run ended on stops like an arrival; a road fight that could
//! end the run gets the notice. Slice 2: a queued battle plays (`story::map_tick`; a queued dialogue waits for the
//! region map); the tips, the Camp button and the log name the purse's upkeep (P1, R1); the Travel tip adds the arrival
//! day and the days left (W5); a node's tip lists its checkpoint lines (`deadlines`, STORYUI1 B10). STORYUI1-U2 (B10):
//! a node holding jobs of opened boards wears the quest badge at its pin's top left (widget `node.<qid>.job`) and its
//! tip lists "Job: <name>". CAMPAIGN1-TM: in a campaign world with travel modes the side panel's mode button
//! (`world_map.mode`) cycles the party's mode (`campaign::purse::set_travel_mode`); the trips, their tips and the road
//! labels count its days, and the trip ask runs the trip's legs in it (`runover::trip_ask`).
//! MAP-HOOKS1 pass A: the background is the current location's region's (`backgrounds/world_map/<region>.png`, up its
//! `parent` chain) before the world's; a layout sidecar beside it (`<name>.json`, `MapLayout`) places the landmarks over
//! `map_pos`; a location with `map/landmarks/<id>.png` shows it (`landmark_box`), else its kind pin.

use aegis_ui::theme::th;
use aegis_ui::canvas::{rgb, rgba, Rect};
use crate::music;
use crate::hud::{hud_bar, hud_key, hud_open, purse_word, Hud, PurseWord};
use crate::screens::{show_errors, Cx, GameCx, Key, Nav, Screen};
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, H, SMALL, W};
use aegis_ui::widget_plumbing;
use aegis_core::json::{self, s};
use emberwake_world::{explore, progression, travel};
use serde_json::{json, Value};

const MAP: Rect = Rect::new(0, 22, 500, 316);

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
}

/// The world map's controls (the Help button's hover card and popup).
const WORLD_HELP: [&str; 4] = ["Click a place to pick it, then Travel", "Enter (button): go into the place the party is at", "P: party   J: journal   I: stash", "Esc: back to the region map"];

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
    let time = json::i(trip, "time", 0);
    let length = if gs.campaign.is_some() { crate::runover::days(time) } else { format!("{time} time") };
    format!("{length}, {} {}", json::i(trip, "supplies", 0), purse_word(cx, PurseWord::Upkeep))
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

    /// The waypoints of route `id` (a qid or a local id). MAP-HOOKS1 pass B draws the roads through them; until then
    /// only the tests read it.
    #[allow(dead_code)]
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

/// The cached layout: the names it was read for, the art generation, and the layout read.
type LayoutCache = (Vec<String>, u64, Option<MapLayout>);

#[derive(Default)]
pub struct WorldMap {
    w: Vec<Widget<WAct>>,
    selected: String,
    /// MAP-HOOKS1: the layout sidecar, read once per name chain and art generation.
    layout: Option<LayoutCache>,
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

    fn act(&mut self, cx: &mut Cx, a: WAct) {
        let d = cx.data.clone();
        match a {
            WAct::Node(q) => {
                if q == cx.gs().location_id {
                    self.selected.clear();
                } else {
                    self.selected = q;
                }
            }
            WAct::Travel => {
                if self.selected.is_empty() {
                    return;
                }
                match crate::runover::trip_ask(cx, &self.selected) {
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
        }
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

    /// The trip to the selected place (TravelRunner.travel): its log lines and campaign events; a road fight starts the
    /// battle (with the notice when it could end the run, W3); an arrival or a trip the run ended on (status `over`)
    /// clears the pick.
    fn travel(&mut self, cx: &mut Cx) {
        let d = cx.data.clone();
        let to = self.selected.clone();
        let res = travel::travel(&d, cx.gs_mut(), &to);
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

impl Screen<GameCx> for WorldMap {
    fn id(&self) -> &'static str {
        "world_map"
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        let mut ui: Ui<WAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().game("worldmap_ground", rgb(12, 16, 24)));
        // MAP-HOOKS1: the current location's region chain, then the world lineage child first
        let names = map_names(&d, &gs);
        let backgrounds: Vec<String> = names.iter().map(|n| emberwake_content::art::world_map_background(n)).collect();
        let layout = self.layout_for(cx, &names);
        match cx.art.first(&backgrounds, 500, 500) {
            Some(img) => ui.p.image(img, Rect::new(MAP.x, MAP.y + (MAP.h - 281) / 2, 500, 281)),
            None => ui.p.fill(MAP, rgb(30, 30, 36)),
        }
        let view = travel::view(&d, &gs);
        // WorldMapView.layout: a layout sidecar's landmark places the node (MAP-HOOKS1, not clamped), else a valid
        // map_pos (0..1) in the letterboxed map rect; any other node gets a fixed slot on a circle (graph node order), so
        // pins keep their place as the fog lifts.
        let rect = (MAP.x as f32, MAP.y as f32 + (MAP.h - 281) as f32 / 2.0, 500.0f32, 281.0f32);
        let all = travel::nodes(&d, &gs.world_id);
        let placed: std::collections::HashMap<String, ((f32, f32), f32)> = all
            .iter()
            .enumerate()
            .map(|(i, q)| (q.clone(), place(layout.as_ref(), q, json::aget(explore::resolve_location(&d, &gs, q), "map_pos"), i, all.len(), rect)))
            .collect();
        let pins: std::collections::HashMap<String, (f32, f32)> = placed.iter().map(|(q, (p, _))| (q.clone(), *p)).collect();
        let pos_of = |n: &serde_json::Map<String, Value>| -> (f32, f32) { pins.get(&s(n, "qid")).copied().unwrap_or((rect.0 + rect.2 / 2.0, rect.1 + rect.3 / 2.0)) };
        let nodes: Vec<serde_json::Map<String, Value>> = json::aget(&view, "nodes").iter().map(|n| json::dict(n).clone()).collect();
        let find = |q: &str| nodes.iter().find(|n| s(n, "qid") == q).map(pos_of);
        for r in json::aget(&view, "routes") {
            let r = json::dict(r);
            let (Some(a), Some(b)) = (find(&s(r, "from")), find(&s(r, "to"))) else { continue };
            let open = json::b(r, "open", true);
            ui.p.line(a.0, a.1, b.0, b.1, if open { rgb(220, 206, 160) } else { rgb(200, 90, 100) }, if open { 0.0 } else { 3.0 });
            // CAMPAIGN1-TM: a leg's days in the party's travel mode (`days`) when the world has modes
            let leg = if r.contains_key("days") { json::i(r, "days", 0) } else { json::i(r, "time", 0) };
            ui.p.text_center((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0 - 8.0, &leg.to_string(), SMALL, rgb(230, 220, 190));
        }
        let here = gs.location_id.clone();
        for n in &nodes {
            let q = s(n, "qid");
            let (x, y) = pos_of(n);
            let is_here = q == here;
            let sel = q == self.selected;
            if is_here || sel {
                ui.p.circle(x, y, 9.0, if is_here { rgba(244, 198, 84, 160) } else { rgba(140, 196, 236, 160) });
            }
            // MAP-HOOKS1: the location's landmark at its layout scale, else the kind pin, else a circle
            let landmark = emberwake_content::art::landmark(&q);
            let r = if cx.art.has(&landmark) {
                let r = landmark_box(x, y, placed.get(&q).map_or(1.0, |p| p.1));
                let px = (r.w as f32 * ui.p.v.s).round().max(1.0) as i32;
                if let Some(img) = cx.art.image(&landmark, px, px) {
                    ui.p.image(img, r);
                }
                r
            } else {
                let r = pin_box(x, y);
                match cx.art.image(&emberwake_content::art::pin(&s(n, "kind")), PIN_PX, PIN_PX) {
                    Some(img) => ui.p.image(img, r),
                    None => ui.p.circle(x, y, 5.0, rgb(210, 210, 220)),
                }
                r
            };
            // the name sits right of the mark (pass B moves it above the landmark)
            let label_x = x + (r.w / 2) as f32 + 2.0;
            if json::b(n, "quest", false) {
                ui.p.text(x + 5.0, y - 15.0, "!", 8.0, th().gold);
            }
            // STORYUI1 B10: the jobs of opened boards here: the quest badge (9 px) at the pin's top left
            let jobs = json::sget(n, "jobs");
            let job_r = Rect::new(r.x - 4, r.y - 4, 9, 9);
            if !jobs.is_empty() {
                let px = ui.icon_px(Rect::new(0, 0, job_r.w + 4, job_r.h + 4));
                match cx.art.image(&crate::region::service_icon("board"), px, px) {
                    Some(img) => ui.p.image(img, job_r),
                    None => crate::region::service_glyph(&mut ui, "board", job_r),
                }
            }
            let label = s(n, "name");
            // TEXTFIT1: the second line sits a line below the name at any text size, and the node's box holds both.
            let line = ui.p.line_h(BODY);
            ui.p.text(label_x, y - 6.0, &label, BODY, if is_here { th().gold } else if json::b(n, "visited", false) { th().text } else { th().dim });
            if is_here {
                ui.p.text(label_x, y - 6.0 + line, "You are here", BODY, th().gold);
            }
            let trip = travel::trip(&d, &gs, &here, &q);
            let mut t = tip(&label, &format!("{}{}", s(n, "region"), if json::b(n, "visited", false) { "" } else { " (not visited)" }));
            if json::b(&trip, "ok", false) {
                t.push((format!("Trip: {}", trip_cost(cx, &gs, &trip)), th().info));
            }
            for step in travel::quest_steps_at(&d, &gs, &q) {
                t.push((format!("Quest: {step}"), th().gold));
            }
            for job in &jobs {
                t.push((format!("Job: {job}"), th().gold));
            }
            // STORYUI1 B10, CAMPAIGN1 W5: the checkpoint lines of the quests whose node this is
            for due in json::aget(n, "deadlines") {
                t.push((s(json::dict(due), "text"), crate::board::DEADLINE));
            }
            let lw = ui.p.text_w(&s(n, "name"), BODY).max(if is_here { ui.p.text_w("You are here", BODY) } else { 0.0 }).ceil() as i32;
            let bottom = (y - 6.0 + line * if is_here { 2.0 } else { 1.0 }).ceil() as i32;
            let top = (r.y - 2).min((y - 6.0).floor() as i32);
            let job_tip = if jobs.is_empty() { Vec::new() } else { t.clone() };
            ui.push(format!("node.{}", q.replace(':', ".")), "node", label, Rect::new(r.x - 2, top, r.w + 4 + lw, (r.y + r.h + 2).max(bottom) - top), true, sel, Some(WAct::Node(q.clone())), t);
            // the badge picks the node too and shows its card
            if !jobs.is_empty() {
                ui.push(format!("node.{}.job", q.replace(':', ".")), "badge", format!("Job: {}", jobs.join(", ")), job_r, true, sel, Some(WAct::Node(q.clone())), job_tip);
            }
        }
        // -- Legend
        let lg = Rect::new(4, 238, 76, 100);
        ui.p.panel(lg, th().game("legend_fill", rgba(10, 14, 22, 220)), th().edge, 2.0);
        let mut ly = lg.y as f32 + 3.0;
        for (kind, label) in [("fortress", "Fortress"), ("town", "Town"), ("camp", "Camp"), ("ruin", "Ruin"), ("gate", "Gate"), ("wild", "Wild")] {
            if let Some(img) = cx.art.image(&emberwake_content::art::pin(kind), 9, 9) {
                ui.p.image(img, Rect::new(lg.x + 4, ly as i32, 9, 9));
            }
            ui.p.text(lg.x as f32 + 17.0, ly, label, SMALL, th().text);
            ly += 10.0;
        }
        ui.p.line(lg.x as f32 + 4.0, ly + 4.0, lg.x as f32 + 14.0, ly + 4.0, rgb(220, 206, 160), 0.0);
        ui.p.text(lg.x as f32 + 17.0, ly, "Open road", SMALL, th().text);
        ly += 10.0;
        ui.p.line(lg.x as f32 + 4.0, ly + 4.0, lg.x as f32 + 14.0, ly + 4.0, rgb(200, 90, 100), 2.0);
        ui.p.text(lg.x as f32 + 17.0, ly, "Closed road", SMALL, th().text);
        // -- HUD bar (PLAYUI1-H): Back and the location name; gold, supplies and the icon buttons at the right
        let back_w = (ui.p.text_w("Back", BODY) + 8.0).ceil().max(26.0) as i32;
        let here_name = cx.location_name(&gs.world_id, &here);
        hud_bar(&mut ui, cx, back_w + 8, &here_name, ("world_map.menu", ""), "", &WORLD_HELP, WAct::Hud);
        ui.button("world_map.back", Rect::new(3, 3, back_w, 14), "Back", WAct::Back, true);
        // -- Side panel: the time, actions and the log
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
            let trip = travel::trip(&d, &gs, &here, &self.selected);
            if json::b(&trip, "ok", false) {
                // CAMPAIGN1 W5: the arrival day and the days left in a campaign world
                let mut text = format!("{}. Roads can be ambushed.", trip_cost(cx, &gs, &trip));
                if let Some(m) = &mode {
                    text = format!("{}: {text}", m.name);
                }
                if let Some(at) = arrival(&gs, json::i(&trip, "time", 0)) {
                    text = format!("{text} {at}");
                }
                ui.button_tip("world_map.travel", Rect::new(sx, y, 130, 16), &format!("Travel to {name}"), WAct::Travel, true, tip("Travel", &text));
            } else {
                ui.p.text_fit(sx as f32, y as f32 + 3.0, &format!("No known way to {name}."), SMALL, 130.0, th().foe);
            }
            y += 20;
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
                if y > H - 10 {
                    break;
                }
                ui.p.text(sx as f32, y as f32, &part, SMALL, th().text);
                y += 8;
            }
        }
        self.w = ui.widgets;
    }

    /// P, J, I and ? open the HUD bar's popups (PLAYUI1-H).
    fn key(&mut self, cx: &mut Cx, k: Key) {
        if crate::viewkeys::window_key(cx, k, crate::viewkeys::Shown::Notice) {
            return;
        }
        if let Some(h) = hud_key(k) {
            self.act(cx, WAct::Hud(h));
        }
    }

    fn state(&self, cx: &Cx) -> Value {
        json!({"selected": self.selected, "view": travel::view(&cx.data, cx.gs())})
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
}
