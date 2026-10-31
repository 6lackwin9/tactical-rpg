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
//! tip lists "Job: <name>".

use aegis_ui::canvas::{rgb, rgba, Rect};
use crate::music;
use crate::hud::{hud_bar, hud_key, hud_open, purse_word, Hud, PurseWord};
use crate::screens::{show_errors, Cx, GameCx, Key, Nav, Screen};
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, BORDER, DIM, FOE, GOLD, H, INFO, SMALL, TEXT, W};
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

#[derive(Default)]
pub struct WorldMap {
    w: Vec<Widget<WAct>>,
    selected: String,
}

impl WorldMap {
    pub fn new() -> Self {
        Self::default()
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
                let trip = travel::trip(&d, cx.gs(), &cx.gs().location_id, &self.selected);
                match crate::runover::days_ask(cx, json::i(&trip, "time", 0), "This trip", "Travel") {
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
            WAct::Back => cx.nav.push(Nav::Pop),
            WAct::Hud(h) => hud_open(cx, h, &WORLD_HELP),
        }
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
        ui.p.fill(Rect::new(0, 0, W, H), rgb(12, 16, 24));
        let lineage = d.lineage(&gs.world_id);
        let backgrounds: Vec<String> = lineage.iter().rev().map(|w| emberwake_content::art::world_map_background(w)).collect();
        match cx.art.first(&backgrounds, 500, 500) {
            Some(img) => ui.p.image(img, Rect::new(MAP.x, MAP.y + (MAP.h - 281) / 2, 500, 281)),
            None => ui.p.fill(MAP, rgb(30, 30, 36)),
        }
        let view = travel::view(&d, &gs);
        // WorldMapView.layout: a valid map_pos (0..1) places the pin in the letterboxed map rect; any other node gets a
        // fixed slot on a circle (graph node order), so pins keep their place as the fog lifts.
        let rect = (MAP.x as f32, MAP.y as f32 + (MAP.h - 281) as f32 / 2.0, 500.0f32, 281.0f32);
        let all = travel::nodes(&d, &gs.world_id);
        let pins: std::collections::HashMap<String, (f32, f32)> = all
            .iter()
            .enumerate()
            .map(|(i, q)| {
                let p = json::aget(explore::resolve_location(&d, &gs, q), "map_pos");
                let at = match p {
                    [x, y] if x.is_number() && y.is_number() => Some((json::to_float_or(x, 0.0).clamp(0.0, 1.0) as f32, json::to_float_or(y, 0.0).clamp(0.0, 1.0) as f32)),
                    _ => None,
                };
                let pos = match at {
                    Some((fx, fy)) => (rect.0 + fx * rect.2, rect.1 + fy * rect.3),
                    None => {
                        let angle = std::f32::consts::TAU * i as f32 / all.len() as f32 - std::f32::consts::FRAC_PI_2;
                        let radius = rect.2.min(rect.3) * 0.38;
                        (rect.0 + rect.2 / 2.0 + angle.cos() * radius, rect.1 + rect.3 / 2.0 + angle.sin() * radius)
                    }
                };
                (q.clone(), pos)
            })
            .collect();
        let pos_of = |n: &serde_json::Map<String, Value>| -> (f32, f32) { pins.get(&s(n, "qid")).copied().unwrap_or((rect.0 + rect.2 / 2.0, rect.1 + rect.3 / 2.0)) };
        let nodes: Vec<serde_json::Map<String, Value>> = json::aget(&view, "nodes").iter().map(|n| json::dict(n).clone()).collect();
        let find = |q: &str| nodes.iter().find(|n| s(n, "qid") == q).map(pos_of);
        for r in json::aget(&view, "routes") {
            let r = json::dict(r);
            let (Some(a), Some(b)) = (find(&s(r, "from")), find(&s(r, "to"))) else { continue };
            let open = json::b(r, "open", true);
            ui.p.line(a.0, a.1, b.0, b.1, if open { rgb(220, 206, 160) } else { rgb(200, 90, 100) }, if open { 0.0 } else { 3.0 });
            ui.p.text_center((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0 - 8.0, &json::i(r, "time", 0).to_string(), SMALL, rgb(230, 220, 190));
        }
        let here = gs.location_id.clone();
        for n in &nodes {
            let q = s(n, "qid");
            let (x, y) = pos_of(n);
            let r = Rect::new(x as i32 - 7, y as i32 - 7, 14, 14);
            let is_here = q == here;
            let sel = q == self.selected;
            if is_here || sel {
                ui.p.circle(x, y, 9.0, if is_here { rgba(244, 198, 84, 160) } else { rgba(140, 196, 236, 160) });
            }
            match cx.art.image(&emberwake_content::art::pin(&s(n, "kind")), 14, 14) {
                Some(img) => ui.p.image(img, r),
                None => ui.p.circle(x, y, 5.0, rgb(210, 210, 220)),
            }
            if json::b(n, "quest", false) {
                ui.p.text(x + 5.0, y - 15.0, "!", 8.0, GOLD);
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
            ui.p.text(x + 9.0, y - 6.0, &label, BODY, if is_here { GOLD } else if json::b(n, "visited", false) { TEXT } else { DIM });
            if is_here {
                ui.p.text(x + 9.0, y - 6.0 + line, "You are here", BODY, GOLD);
            }
            let trip = travel::trip(&d, &gs, &here, &q);
            let mut t = tip(&label, &format!("{}{}", s(n, "region"), if json::b(n, "visited", false) { "" } else { " (not visited)" }));
            if json::b(&trip, "ok", false) {
                t.push((format!("Trip: {}", trip_cost(cx, &gs, &trip)), INFO));
            }
            for step in travel::quest_steps_at(&d, &gs, &q) {
                t.push((format!("Quest: {step}"), GOLD));
            }
            for job in &jobs {
                t.push((format!("Job: {job}"), GOLD));
            }
            // STORYUI1 B10, CAMPAIGN1 W5: the checkpoint lines of the quests whose node this is
            for due in json::aget(n, "deadlines") {
                t.push((s(json::dict(due), "text"), crate::board::DEADLINE));
            }
            let lw = ui.p.text_w(&s(n, "name"), BODY).max(if is_here { ui.p.text_w("You are here", BODY) } else { 0.0 }).ceil() as i32;
            let bottom = (y - 6.0 + line * if is_here { 2.0 } else { 1.0 }).ceil() as i32;
            let top = (r.y - 2).min((y - 6.0).floor() as i32);
            let job_tip = if jobs.is_empty() { Vec::new() } else { t.clone() };
            ui.push(format!("node.{}", q.replace(':', ".")), "node", label, Rect::new(r.x - 2, top, 18 + lw, (r.y + 16).max(bottom) - top), true, sel, Some(WAct::Node(q.clone())), t);
            // the badge picks the node too and shows its card
            if !jobs.is_empty() {
                ui.push(format!("node.{}.job", q.replace(':', ".")), "badge", format!("Job: {}", jobs.join(", ")), job_r, true, sel, Some(WAct::Node(q.clone())), job_tip);
            }
        }
        // -- Legend
        let lg = Rect::new(4, 238, 76, 100);
        ui.p.panel(lg, rgba(10, 14, 22, 220), BORDER, 2.0);
        let mut ly = lg.y as f32 + 3.0;
        for (kind, label) in [("fortress", "Fortress"), ("town", "Town"), ("camp", "Camp"), ("ruin", "Ruin"), ("gate", "Gate"), ("wild", "Wild")] {
            if let Some(img) = cx.art.image(&emberwake_content::art::pin(kind), 9, 9) {
                ui.p.image(img, Rect::new(lg.x + 4, ly as i32, 9, 9));
            }
            ui.p.text(lg.x as f32 + 17.0, ly, label, SMALL, TEXT);
            ly += 10.0;
        }
        ui.p.line(lg.x as f32 + 4.0, ly + 4.0, lg.x as f32 + 14.0, ly + 4.0, rgb(220, 206, 160), 0.0);
        ui.p.text(lg.x as f32 + 17.0, ly, "Open road", SMALL, TEXT);
        ly += 10.0;
        ui.p.line(lg.x as f32 + 4.0, ly + 4.0, lg.x as f32 + 14.0, ly + 4.0, rgb(200, 90, 100), 2.0);
        ui.p.text(lg.x as f32 + 17.0, ly, "Closed road", SMALL, TEXT);
        // -- HUD bar (PLAYUI1-H): Back and the location name; gold, supplies and the icon buttons at the right
        let back_w = (ui.p.text_w("Back", BODY) + 8.0).ceil().max(26.0) as i32;
        let here_name = cx.location_name(&gs.world_id, &here);
        hud_bar(&mut ui, cx, back_w + 8, &here_name, ("world_map.menu", ""), "", &WORLD_HELP, WAct::Hud);
        ui.button("world_map.back", Rect::new(3, 3, back_w, 14), "Back", WAct::Back, true);
        // -- Side panel: the time, actions and the log
        let sx = 504;
        let mut y = 28;
        ui.p.text(sx as f32, y as f32, "World map", BODY, INFO);
        ui.p.text_right(W as f32 - 6.0, y as f32, &format!("Time {}", gs.time), BODY, TEXT);
        y += 14;
        if !self.selected.is_empty() {
            let name = cx.location_name(&gs.world_id, &self.selected);
            let trip = travel::trip(&d, &gs, &here, &self.selected);
            if json::b(&trip, "ok", false) {
                // CAMPAIGN1 W5: the arrival day and the days left in a campaign world
                let mut text = format!("{}. Roads can be ambushed.", trip_cost(cx, &gs, &trip));
                if let Some(at) = arrival(&gs, json::i(&trip, "time", 0)) {
                    text = format!("{text} {at}");
                }
                ui.button_tip("world_map.travel", Rect::new(sx, y, 130, 16), &format!("Travel to {name}"), WAct::Travel, true, tip("Travel", &text));
            } else {
                ui.p.text_fit(sx as f32, y as f32 + 3.0, &format!("No known way to {name}."), SMALL, 130.0, FOE);
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
                ui.p.text(sx as f32, y as f32, &part, SMALL, TEXT);
                y += 8;
            }
        }
        self.w = ui.widgets;
    }

    /// P, J, I and ? open the HUD bar's popups (PLAYUI1-H).
    fn key(&mut self, cx: &mut Cx, k: Key) {
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
