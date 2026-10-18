//! The region map (scripts/ui/overworld/region_map_view.gd + dialogue_panel.gd + shop_panel.gd): walking a
//! location's map over the game state (ExploreRunner), the "Here" list and event log, the dialogue dock at the bottom
//! (DialogueRunner), the shop popup (ShopRules), the turn-in picker and lore pages. Fights start the battle screen;
//! a world-map exit opens the world map. Map-less locations show their exits as buttons.

use crate::canvas::{rgb, rgba, Rect, Rgba};
use crate::screens::{show_errors, Cx, Key, Nav, Screen};
use crate::ui::{tip, Gfx, Ui, Widget, BODY, BORDER, CARD, DIM, GOLD, GOOD, H, INFO, SMALL, TEXT, TITLE, W};
use crate::widget_plumbing;
use aegis_world::dialogue::DialogueRunner;
use aegis_world::explore::{self, Pos};
use aegis_world::json::{self, s, s_or, Dict};
use aegis_world::{quest, shop};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

const MAP: Rect = Rect::new(0, 23, 500, 337);
const SIDE_X: i32 = 502;

#[derive(Debug, Clone)]
pub enum RAct {
    MapClick,
    Marker(String),
    Exit(String),
    Menu,
    Choose(i64),
    Advance,
    Farewell,
    ShopTab(bool),
    Buy(String, String, i64),
    Sell(String),
    CloseShop,
    TogglePick(String),
    Recipient,
    ConfirmTurnIn,
    CancelTurnIn,
    CloseLore,
}

pub struct RegionMap {
    w: Vec<Widget<RAct>>,
    mouse: (i32, i32),
    facing: Pos,
    runner: Option<DialogueRunner>,
    shop_buy: bool,
    shop_detail: String,
    pending: Option<Dict>,
    picks: Vec<String>,
    recipient: usize,
    lore: Option<(String, String)>,
    walk: Vec<Pos>,
    walk_target: Option<String>,
    next_walk: Instant,
    /// Map geometry of the last frame: (origin x, origin y, tile size).
    geo: (i32, i32, i32),
}

/// Starts a world fight: the battle screen, or the Won / Fled / Lost popup in a debug game.
pub fn begin_fight(cx: &mut Cx, fight: &Dict, runner: &str) {
    if cx.debug {
        cx.nav.push(Nav::Push(Box::new(crate::runner::DebugFight::new(fight, runner))));
        return;
    }
    match crate::battle::BattleScreen::world(cx, fight, runner) {
        Ok(b) => cx.nav.push(Nav::Push(Box::new(b))),
        Err(e) => show_errors(cx, vec![e]),
    }
}

/// Event lines for the log; `autosave` writes the autosave.
pub fn log_events(cx: &mut Cx, events: &[Value]) {
    for e in events {
        let e = json::dict(e);
        match s(e, "kind").as_str() {
            "entered" => {
                let loc = s(e, "location");
                let name = cx.location_name(&cx.gs().world_id.clone(), &loc);
                cx.log(format!("Entered {name}."));
                for q in json::sget(e, "advanced") {
                    let qn = s_or(&quest::resolved(&cx.data, cx.gs(), &q), "name", &q);
                    cx.log(format!("Quest updated: {qn}."));
                }
            }
            "rested" => cx.log(format!("The party rests (+{} HP).", json::i(e, "restored", 0))),
            "locked" => cx.log(format!("Locked: {}", s(e, "text"))),
            "camped" => cx.log(format!("The party makes camp (-{} supplies, +{} HP).", json::i(e, "spent", 0), json::i(e, "restored", 0))),
            "defeat" => {
                let name = cx.location_name(&cx.gs().world_id.clone(), &s(e, "location"));
                let how = match s(e, "mode").as_str() {
                    "rest_place" => "rests there for free".to_string(),
                    "forced_rest" => format!("rests ({} supplies)", json::i(e, "spent", 0)),
                    _ => "limps back to town".to_string(),
                };
                cx.log(format!("Defeated. The party wakes at {name} and {how}."));
            }
            "exit" => log_events(cx, json::aget(e, "events")),
            "autosave" => cx.autosave(),
            _ => {}
        }
    }
}

impl RegionMap {
    pub fn new(cx: &mut Cx) -> Self {
        let d = cx.data.clone();
        let res = explore::resume(&d, cx.gs_mut());
        if !json::b(&res, "ok", false) {
            cx.notice = json::sget(&res, "errors");
        }
        log_events(cx, json::aget(&res, "events"));
        Self {
            w: Vec::new(),
            mouse: (0, 0),
            facing: (0, 1),
            runner: None,
            shop_buy: true,
            shop_detail: String::new(),
            pending: None,
            picks: Vec::new(),
            recipient: 0,
            lore: None,
            walk: Vec::new(),
            walk_target: None,
            next_walk: Instant::now(),
            geo: (0, 0, 1),
        }
    }

    fn talking(&self) -> bool {
        self.runner.as_ref().is_some_and(|r| !r.closed)
    }

    fn modal(&self) -> bool {
        self.talking() || self.lore.is_some()
    }

    /// One step; handles fights, world-map exits and the log. False when the walk should stop.
    fn step(&mut self, cx: &mut Cx, dir: Pos) -> bool {
        self.facing = dir;
        let d = cx.data.clone();
        let res = explore::step(&d, cx.gs_mut(), dir);
        if !json::b(&res, "ok", false) {
            for e in json::sget(&res, "errors") {
                cx.log(e);
            }
            return false;
        }
        let events = json::aget(&res, "events").to_vec();
        log_events(cx, &events);
        let mut go_on = true;
        for e in &events {
            let e = json::dict(e);
            match s(e, "kind").as_str() {
                "fight" => {
                    self.start_fight(cx, e, "explore");
                    go_on = false;
                }
                "exit" if s(e, "to") == explore::WORLD_MAP => {
                    cx.nav.push(Nav::Push(Box::new(crate::worldmap::WorldMap::new())));
                    go_on = false;
                }
                "exit" | "blocked" | "locked" => go_on = false,
                _ => {}
            }
        }
        go_on
    }

    pub fn start_fight(&mut self, cx: &mut Cx, fight: &Dict, runner: &str) {
        let name = s_or(cx.data.resolve(&cx.gs().world_id, "encounters", &s(fight, "encounter")), "name", "Enemies");
        cx.log(format!("Fight: {name}!"));
        self.walk.clear();
        begin_fight(cx, fight, runner);
    }

    fn interact(&mut self, cx: &mut Cx, dir: Pos) {
        self.facing = dir;
        let d = cx.data.clone();
        let res = explore::interact(&d, cx.gs_mut(), dir);
        if !json::b(&res, "ok", false) {
            show_errors(cx, json::sget(&res, "errors"));
            return;
        }
        match s(&res, "kind").as_str() {
            "npc" => self.talk(cx, &s(&res, "character")),
            "lore" => self.lore = Some((s(&res, "name"), s(&res, "text"))),
            "chest" => {
                if json::b(&res, "empty", false) {
                    cx.log("The chest is empty.");
                } else {
                    let mut got: Vec<String> = json::ints(json::get(&res, "items")).iter().map(|(id, n)| format!("{} x{n}", cx.data.item_name(id))).collect();
                    if json::i(&res, "gold", 0) > 0 {
                        got.push(format!("{} gold", json::i(&res, "gold", 0)));
                    }
                    if json::i(&res, "supplies", 0) > 0 {
                        got.push(format!("{} supplies", json::i(&res, "supplies", 0)));
                    }
                    cx.log(if got.is_empty() { "The chest holds nothing useful.".to_string() } else { format!("Found: {}.", got.join(", ")) });
                    for n in json::sget(&res, "notes") {
                        cx.log(n);
                    }
                }
            }
            _ => cx.log("Nothing there."),
        }
    }

    fn talk(&mut self, cx: &mut Cx, character: &str) {
        let d = cx.data.clone();
        let r = DialogueRunner::open(&d, cx.gs_mut(), character);
        if r.closed {
            for e in &r.errors {
                cx.log(e.clone());
            }
            return;
        }
        cx.log(format!("You speak with {}.", r.character_name));
        cx.set_busy(true);
        self.runner = Some(r);
    }

    fn end_talk(&mut self, cx: &mut Cx) {
        self.runner = None;
        self.pending = None;
        cx.set_busy(false);
    }

    /// Handles a dialogue result: fights, the turn-in picker, notes, the end.
    fn dialogue_result(&mut self, cx: &mut Cx, res: Dict) {
        if !json::b(&res, "ok", false) {
            for e in json::sget(&res, "errors") {
                cx.log(e);
            }
        }
        for n in json::sget(&res, "notes") {
            cx.log(n);
        }
        if let Some(g) = res.get("granted").and_then(Value::as_object) {
            let mut got = Vec::new();
            if json::i(g, "xp", 0) > 0 {
                got.push(format!("{} XP", json::i(g, "xp", 0)));
            }
            if json::i(g, "gold", 0) > 0 {
                got.push(format!("{} gold", json::i(g, "gold", 0)));
            }
            for (id, n) in json::ints(json::get(g, "items")) {
                got.push(format!("{} x{n}", cx.data.item_name(&id)));
            }
            if !got.is_empty() {
                cx.log(format!("Quest complete: {}.", got.join(", ")));
            }
        }
        if res.contains_key("pending") {
            self.picks.clear();
            self.recipient = 0;
            self.pending = Some(res);
            return;
        }
        if let Some(f) = res.get("fight").and_then(Value::as_object).cloned() {
            self.end_talk(cx);
            cx.set_busy(false);
            self.start_fight(cx, &f, "dialogue");
            return;
        }
        if self.runner.as_ref().is_some_and(|r| r.closed) {
            self.end_talk(cx);
        }
    }

    /// BFS over open, unblocked tiles to `target` (or next to it); the step directions.
    fn path_to(&self, cx: &Cx, target: Pos, adjacent: bool) -> Option<Vec<Pos>> {
        let d = &cx.data;
        let gs = cx.gs();
        let loc = explore::location(d, gs);
        let t = explore::template_of(d, loc);
        let loc_qid = s(loc, "_qid");
        let steps = gs.explore_int("loc_steps");
        let mut blocked = Vec::new();
        for m in explore::markers_of(loc) {
            let st = explore::presence(d, gs, &loc_qid, &m);
            if st == "locked" || (st == "present" && ["npc", "chest", "lore"].contains(&s(&m, "kind").as_str())) {
                blocked.push(explore::marker_pos(t, &m, steps));
            }
        }
        let start = gs.position;
        let mut prev: std::collections::HashMap<Pos, (Pos, Pos)> = std::collections::HashMap::new();
        let mut queue = std::collections::VecDeque::from([start]);
        let mut seen = vec![start];
        while let Some(cur) = queue.pop_front() {
            let reached = if adjacent { (cur.0 - target.0).abs() + (cur.1 - target.1).abs() == 1 } else { cur == target };
            if reached {
                let mut dirs = Vec::new();
                let mut c = cur;
                while c != start {
                    let (p, dir) = prev[&c];
                    dirs.push(dir);
                    c = p;
                }
                dirs.reverse();
                return Some(dirs);
            }
            for dir in explore::DIRS {
                let n = (cur.0 + dir.0, cur.1 + dir.1);
                if seen.contains(&n) || !explore::is_open(t, n.0, n.1) || (blocked.contains(&n) && (adjacent || n != target)) {
                    continue;
                }
                seen.push(n);
                prev.insert(n, (cur, dir));
                queue.push_back(n);
            }
        }
        None
    }

    fn walk_to_marker(&mut self, cx: &mut Cx, id: &str) {
        let d = cx.data.clone();
        let gs = cx.gs();
        let loc = explore::location(&d, gs);
        let t = explore::template_of(&d, loc);
        let Some(m) = explore::markers_of(loc).into_iter().find(|m| s(m, "id") == id) else { return };
        let at = explore::marker_pos(t, &m, gs.explore_int("loc_steps"));
        let solid = ["npc", "chest", "lore"].contains(&s(&m, "kind").as_str()) || explore::presence(&d, gs, &s(loc, "_qid"), &m) == "locked";
        if solid {
            let dir = (at.0 - gs.position.0, at.1 - gs.position.1);
            if explore::DIRS.contains(&dir) {
                self.interact(cx, dir);
                return;
            }
        }
        match self.path_to(cx, at, solid) {
            Some(path) if !path.is_empty() => {
                self.walk = path;
                self.walk_target = if solid { Some(id.to_string()) } else { None };
            }
            _ => cx.log("You can't get there."),
        }
    }

    fn act(&mut self, cx: &mut Cx, a: RAct) {
        let d = cx.data.clone();
        match a {
            RAct::MapClick => {
                let (ox, oy, ts) = self.geo;
                let tx = ((self.mouse.0 - ox) as f32 / ts as f32).floor() as i64;
                let ty = ((self.mouse.1 - oy) as f32 / ts as f32).floor() as i64;
                if let Some(path) = self.path_to(cx, (tx, ty), false) {
                    self.walk = path;
                    self.walk_target = None;
                }
            }
            RAct::Marker(id) => self.walk_to_marker(cx, &id),
            RAct::Exit(to) => {
                let res = explore::take_exit(&d, cx.gs_mut(), &to, explore::SPAWN);
                if !json::b(&res, "ok", false) {
                    show_errors(cx, json::sget(&res, "errors"));
                    return;
                }
                let events = json::aget(&res, "events").to_vec();
                log_events(cx, &events);
                if to == explore::WORLD_MAP {
                    cx.nav.push(Nav::Push(Box::new(crate::worldmap::WorldMap::new())));
                }
            }
            RAct::Menu => cx.nav.push(Nav::Push(Box::new(crate::hub::Hub::default()))),
            RAct::Choose(i) => {
                let Some(r) = self.runner.as_mut() else { return };
                let res = r.choose(&d, cx.gs_mut(), i);
                self.dialogue_result(cx, res);
            }
            RAct::Advance => {
                let Some(r) = self.runner.as_mut() else { return };
                let res = r.advance(&d, cx.gs_mut());
                self.dialogue_result(cx, res);
            }
            RAct::Farewell => {
                let Some(r) = self.runner.as_ref() else { return };
                let view = r.view(&d, cx.gs());
                let farewell = json::aget(&view, "choices").iter().position(|c| s(json::dict(c), "service") == "farewell");
                match farewell {
                    Some(i) => self.act(cx, RAct::Choose(i as i64)),
                    None => self.end_talk(cx),
                }
            }
            RAct::ShopTab(buy) => self.shop_buy = buy,
            RAct::Buy(kind, id, n) => {
                let Some(sq) = self.runner.as_ref().map(|r| r.shop_qid.clone()) else { return };
                let res = shop::buy(&d, cx.gs_mut(), &sq, &kind, &id, n);
                if json::b(&res, "ok", false) {
                    cx.log(format!("Bought for {} gold.", json::i(&res, "price", 0)));
                } else {
                    show_errors(cx, json::sget(&res, "errors"));
                }
            }
            RAct::Sell(item) => {
                let Some(sq) = self.runner.as_ref().map(|r| r.shop_qid.clone()) else { return };
                let res = shop::sell(&d, cx.gs_mut(), &sq, &item, 1);
                if json::b(&res, "ok", false) {
                    cx.log(format!("Sold {} for {} gold.", d.item_name(&item), json::i(&res, "gold", 0)));
                } else {
                    show_errors(cx, json::sget(&res, "errors"));
                }
            }
            RAct::CloseShop => {
                let Some(r) = self.runner.as_mut() else { return };
                let res = r.close_shop(&d, cx.gs_mut());
                self.dialogue_result(cx, res);
            }
            RAct::TogglePick(id) => {
                let allowed = self.pending.as_ref().map(|p| json::i(p, "picks_allowed", 0)).unwrap_or(0) as usize;
                if let Some(i) = self.picks.iter().position(|p| *p == id) {
                    self.picks.remove(i);
                } else if self.picks.len() < allowed {
                    self.picks.push(id);
                }
            }
            RAct::Recipient => {
                let n = cx.gs().party.members.len().max(1);
                self.recipient = (self.recipient + 1) % n;
            }
            RAct::ConfirmTurnIn => {
                let rec = cx.gs().party.members.get(self.recipient).map(|m| m.id.clone()).unwrap_or_default();
                let picks = self.picks.clone();
                let Some(r) = self.runner.as_mut() else { return };
                let res = r.confirm_turn_in(&d, cx.gs_mut(), &rec, &picks);
                if json::b(&res, "ok", false) {
                    self.pending = None;
                }
                self.dialogue_result(cx, res);
            }
            RAct::CancelTurnIn => {
                if let Some(r) = self.runner.as_mut() {
                    r.cancel();
                }
                self.pending = None;
            }
            RAct::CloseLore => self.lore = None,
        }
    }

    fn marker_style(kind: &str, state: &str) -> (Rgba, &'static str) {
        match (kind, state) {
            ("exit", "locked") => (rgb(180, 70, 60), ">"),
            ("exit", _) => (rgb(70, 190, 100), ">"),
            ("chest", "spent") => (rgb(90, 74, 50), "c"),
            ("chest", _) => (rgb(200, 150, 60), "c"),
            ("lore", _) => (rgb(150, 110, 220), "L"),
            ("encounter", _) => (rgb(220, 70, 60), "!"),
            ("roamer", _) => (rgb(230, 110, 60), "R"),
            ("npc", _) => (rgb(230, 190, 80), ""),
            _ => (rgb(230, 220, 120), "?"),
        }
    }
}

fn region_key(ch: char, x: i64, y: i64) -> Option<String> {
    let base = match ch {
        '.' => "region_ash_floor",
        '#' => "region_basalt_wall",
        '~' => "region_water",
        _ => return None,
    };
    Some(format!("ui/{base}_{}_icon", (x * 7 + y * 13).rem_euclid(3)))
}

impl Screen for RegionMap {
    fn id(&self) -> &'static str {
        "region_map"
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        self.mouse = mouse;
        let d = cx.data.clone();
        let mut ui: Ui<RAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), rgb(14, 18, 26));
        let gs = cx.gs().clone();
        let loc = explore::location(&d, &gs);
        let t = explore::template_of(&d, loc);
        let talking = self.talking();
        let modal = self.modal();
        // -- Map
        let map_area = if talking { Rect::new(0, 23, 500, 180) } else { MAP };
        if !t.is_empty() {
            let (mw, mh) = explore::size_of(t);
            // MAPVIEW1: 32 px tiles at 1280 wide (16 logical), the camera centred on the party and clamped to the map;
            // a map smaller than the view is centred.
            let ts = 16;
            let cam = |area_pos: i32, area_len: i32, party: i64, tiles: i64| -> i32 {
                let len = ts * tiles as i32;
                if len <= area_len {
                    area_pos + (area_len - len) / 2
                } else {
                    (area_pos + area_len / 2 - (party as i32 * ts + ts / 2)).clamp(area_pos + area_len - len, area_pos)
                }
            };
            let ox = cam(map_area.x, map_area.w, gs.position.0, mw);
            let oy = cam(map_area.y, map_area.h, gs.position.1, mh);
            self.geo = (ox, oy, ts);
            for (y, row) in explore::rows_of(t).iter().enumerate() {
                for (x, ch) in row.iter().enumerate() {
                    let r = Rect::new(ox + x as i32 * ts, oy + y as i32 * ts, ts, ts);
                    if r.y + r.h <= map_area.y || r.y >= map_area.y + map_area.h || r.x + r.w <= map_area.x || r.x >= map_area.x + map_area.w {
                        continue;
                    }
                    let key = region_key(*ch, x as i64, y as i64);
                    match key.and_then(|k| cx.art.get(&k, ts)) {
                        Some(img) => ui.p.image(img, r),
                        None => ui.p.fill(r, match ch {
                            '.' => rgb(70, 90, 64),
                            '#' => rgb(40, 60, 50),
                            '~' => rgb(40, 90, 100),
                            _ => rgb(20, 20, 24),
                        }),
                    }
                }
            }
            if !modal {
                ui.push("map", "map", s_or(loc, "name", ""), Rect::new(ox.max(map_area.x), oy.max(map_area.y), (ts * mw as i32).min(map_area.w), (ts * mh as i32).min(map_area.h)), true, false, Some(RAct::MapClick), Vec::new());
            }
            let steps = gs.explore_int("loc_steps");
            for raw in json::aget(loc, "markers") {
                let m = explore::marker_full(json::dict(raw));
                let view = explore::marker_view(&d, &gs, loc, &m);
                let state = s(&view, "state");
                if state == "absent" {
                    continue;
                }
                let at = explore::marker_pos(t, &m, steps);
                if at.0 < 0 {
                    continue;
                }
                let r = Rect::new(ox + at.0 as i32 * ts, oy + at.1 as i32 * ts, ts, ts);
                if r.y < map_area.y || r.y + r.h > map_area.y + map_area.h + ts / 2 || r.x < map_area.x || r.x + r.w > map_area.x + map_area.w {
                    continue;
                }
                let kind = s(&m, "kind");
                let (col, glyph) = Self::marker_style(&kind, &state);
                let (cx_, cy_) = (r.x as f32 + ts as f32 / 2.0, r.y as f32 + ts as f32 / 2.0);
                if kind == "npc" {
                    let who = d.resolve(&gs.world_id, "characters", &s(&m, "character"));
                    let art = s(who, "art");
                    ui.p.circle(cx_, cy_, ts as f32 * 0.42, col);
                    let tok = Rect::new(r.x + ts / 8, r.y + ts / 8, ts - ts / 4, ts - ts / 4);
                    match cx.art.token(&art, tok.w).or(None) {
                        Some(img) => ui.p.image(img, tok),
                        None => ui.p.text_center(cx_, cy_ - 4.0, &s(who, "name").chars().next().unwrap_or('?').to_string(), BODY, rgb(20, 20, 20)),
                    }
                    ui.p.text_center(cx_ + 0.5, r.y as f32 - 8.5, &s(&view, "label"), SMALL, rgba(0, 0, 0, 200));
                    ui.p.text_center(cx_, r.y as f32 - 9.0, &s(&view, "label"), SMALL, TEXT);
                    match s(&view, "badge").as_str() {
                        "quest" => {
                            ui.p.text(r.x as f32 + ts as f32 - 5.0, r.y as f32 - 2.0, "!", TITLE, GOLD);
                        }
                        "ready" => {
                            ui.p.text(r.x as f32 + ts as f32 - 5.0, r.y as f32 - 2.0, "?", TITLE, GOOD);
                        }
                        _ => {}
                    }
                } else {
                    ui.p.panel(r.inset(ts / 5), col, rgba(255, 255, 255, 90), 2.0);
                    ui.p.text_center(cx_, cy_ - 4.5, glyph, BODY, rgb(16, 16, 20));
                    if kind == "exit" {
                        ui.p.text_center(cx_ + 0.5, r.y as f32 - 8.5, &s(&view, "label"), SMALL, rgba(0, 0, 0, 200));
                        ui.p.text_center(cx_, r.y as f32 - 9.0, &s(&view, "label"), SMALL, TEXT);
                    }
                }
                if !modal {
                    let id = s(&m, "id");
                    ui.push(format!("marker.{id}"), "marker", s(&view, "label"), r, true, false, Some(RAct::Marker(id)), tip(&s(&view, "label"), &s(&view, "verb")));
                }
            }
            // the party
            let pr = Rect::new(ox + gs.position.0 as i32 * ts, oy + gs.position.1 as i32 * ts, ts, ts);
            let (pcx, pcy) = (pr.x as f32 + ts as f32 / 2.0, pr.y as f32 + ts as f32 / 2.0);
            ui.p.circle(pcx, pcy, ts as f32 * 0.3, rgb(250, 214, 60));
            ui.p.line(pcx, pcy, pcx + self.facing.0 as f32 * ts as f32 * 0.32, pcy + self.facing.1 as f32 * ts as f32 * 0.32, rgb(40, 30, 10), 0.0);
            ui.push("party", "party", format!("Party at {}, {}", gs.position.0, gs.position.1), pr, true, false, None, Vec::new());
        } else {
            ui.p.text(20.0, 40.0, &s_or(loc, "name", "Nowhere"), 11.0, GOLD);
            let y = ui.para(20, 58, 440, &s(loc, "description"), BODY, TEXT);
            for (i, ex) in explore::menu(&d, &gs).iter().enumerate() {
                let to = s(ex, "to");
                let label = s(ex, "label");
                ui.button(&format!("exit.{to}"), Rect::new(20, y + 10 + i as i32 * 22, 200, 18), &label, RAct::Exit(to.clone()), !modal);
            }
        }
        // -- Top bar
        ui.p.fill(Rect::new(0, 0, W, 23), rgba(10, 14, 22, 255));
        let name = s_or(loc, "name", "");
        let end = ui.p.text(4.0, 2.5, &name, BODY, GOLD);
        ui.p.text(end + 8.0, 3.0, &format!("Supplies {} / 99", gs.party.supplies), SMALL, TEXT);
        ui.p.text(4.0, 12.5, "Arrows / WASD: move   E: interact   Click: walk there, or a marker to use it", SMALL, DIM);
        ui.button_full("region.menu", Rect::new(586, 5, 50, 16), "Menu", RAct::Menu, !modal, false, BODY, tip("Game menu (Esc)", "Party, journal, save, load, settings."));
        // -- Right panel: here + log
        let side = Rect::new(SIDE_X - 2, 23, W - SIDE_X + 2, H - 23);
        ui.p.fill(side, rgba(12, 18, 28, 235));
        let mut y = 28;
        if !t.is_empty() && !talking {
            ui.p.text(SIDE_X as f32 + 2.0, y as f32, "Here", BODY, INFO);
            y += 12;
            for raw in json::aget(loc, "markers") {
                let m = explore::marker_full(json::dict(raw));
                let view = explore::marker_view(&d, &gs, loc, &m);
                if s(&view, "state") == "absent" || s(&m, "kind") == "point" {
                    continue;
                }
                let id = s(&m, "id");
                let (col, _) = Self::marker_style(&s(&m, "kind"), &s(&view, "state"));
                let r = Rect::new(SIDE_X, y, W - SIDE_X - 4, 12);
                ui.p.fill(Rect::new(r.x + 2, r.y + 3, 5, 6), col);
                ui.row(&format!("here.{id}"), Rect::new(r.x + 8, r.y, r.w - 8, r.h), &s(&view, "label"), TEXT, (!modal).then(|| RAct::Marker(id.clone())), false, tip(&s(&view, "label"), &s(&view, "verb")));
                y += 13;
            }
            y += 4;
        }
        let max_lines = ((H - y - 4) / 8).max(1) as usize;
        let mut lines: Vec<String> = Vec::new();
        for l in cx.log.iter().rev() {
            let mut wrapped = ui.p.wrap(l, SMALL, (W - SIDE_X - 8) as f32);
            wrapped.reverse();
            lines.extend(wrapped);
            if lines.len() >= max_lines {
                break;
            }
        }
        lines.truncate(max_lines);
        lines.reverse();
        for l in &lines {
            ui.p.text(SIDE_X as f32 + 2.0, y as f32, l, SMALL, TEXT);
            y += 8;
        }
        // -- Dialogue dock
        if let Some(r) = self.runner.as_ref().filter(|r| !r.closed) {
            let view = r.view(&d, &gs);
            let dock = Rect::new(0, 203, W, H - 203);
            ui.p.fill(dock, rgb(14, 20, 34));
            ui.p.line(0.0, 203.0, W as f32, 203.0, FRAME_LINE, 0.0);
            let pbox = Rect::new(6, 208, 93, 120);
            ui.p.panel(pbox, rgb(22, 30, 48), BORDER, 2.0);
            let speaker = d.resolve(&gs.world_id, "characters", &s(&view, "speaker"));
            let art = s(speaker, "art");
            match cx.art.get(&format!("unit/{art}_portrait"), 89) {
                Some(img) => ui.p.image(img, Rect::new(8, 214, 89, 89)),
                None => ui.p.text_center(52.0, 250.0, &s(&view, "speaker_name").chars().next().unwrap_or('?').to_string(), 20.0, GOLD),
            }
            let mut head = s(&view, "speaker_name");
            if !s(&view, "faction").is_empty() {
                head = format!("{head}  ({}: {})", s(&view, "faction_name"), s(&view, "band"));
            }
            ui.p.text_fit(106.0, 210.0, &head, TITLE + 1.0, 280.0, GOLD);
            ui.para(106, 226, 280, &s(&view, "text"), BODY, TEXT);
            let mut cy = 209;
            let choices = json::aget(&view, "choices");
            let busy = self.pending.is_some() || !r.shop_qid.is_empty();
            for (i, c) in choices.iter().enumerate() {
                let c = json::dict(c);
                let locked = json::b(c, "locked", false);
                let mut text = format!("{}  {}", i + 1, s(c, "text"));
                if locked {
                    text = format!("{text}  ({})", s(c, "reason"));
                }
                let lines = ui.p.wrap(&text, BODY, 228.0);
                let hgt = 6 + lines.len() as i32 * 10;
                let rr = Rect::new(394, cy, 240, hgt);
                let hover = !locked && !busy && ui.hover(rr);
                ui.p.panel(rr, if hover { rgb(34, 50, 82) } else { CARD }, if hover { GOLD } else if locked { BORDER } else { rgb(100, 130, 186) }, 2.0);
                let mut ly = cy as f32 + 3.0;
                for l in &lines {
                    ui.p.text(rr.x as f32 + 5.0, ly, l, BODY, if locked { DIM } else { TEXT });
                    ly += 10.0;
                }
                ui.push(format!("choice.{i}"), "choice", text, rr, !locked && !busy, false, Some(RAct::Choose(i as i64)), Vec::new());
                cy += hgt + 3;
            }
            if json::b(&view, "next", false) || (json::b(&view, "end", false) && choices.is_empty()) {
                ui.button("dialogue.continue", Rect::new(394, cy, 240, 16), "Continue", RAct::Advance, !busy);
            }
            ui.p.text(106.0, 345.0, "1-9 choose   Esc or right click: Farewell", SMALL, DIM);
        }
        let shop_qid = self.runner.as_ref().filter(|r| !r.closed).map(|r| r.shop_qid.clone()).unwrap_or_default();
        if !shop_qid.is_empty() {
            self.draw_shop(&mut ui, cx, &shop_qid);
        }
        if let Some(p) = self.pending.clone().filter(|_| self.talking()) {
            self.draw_turn_in(&mut ui, cx, &p);
        }
        if let Some((name, text)) = self.lore.clone() {
            ui.p.fill(Rect::new(0, 0, W, H), rgba(0, 0, 0, 120));
            let fr = Rect::new(150, 60, 340, 220);
            ui.frame(fr, &name, false);
            ui.para(fr.x + 10, fr.y + 30, fr.w - 20, &text, BODY, TEXT);
            ui.button("lore.close", Rect::new(fr.x + fr.w - 60, fr.y + fr.h - 24, 50, 16), "Close", RAct::CloseLore, true);
        }
        self.w = ui.widgets;
    }

    fn tick(&mut self, cx: &mut Cx) {
        if self.walk.is_empty() || self.modal() || Instant::now() < self.next_walk {
            return;
        }
        self.next_walk = Instant::now() + Duration::from_millis(90);
        let dir = self.walk.remove(0);
        if !self.step(cx, dir) {
            self.walk.clear();
            self.walk_target = None;
            return;
        }
        if self.walk.is_empty() {
            if let Some(id) = self.walk_target.take() {
                self.walk_to_marker(cx, &id);
            }
        }
    }

    fn key(&mut self, cx: &mut Cx, k: Key) {
        if let Some(r) = self.runner.as_ref().filter(|r| !r.closed) {
            if self.pending.is_some() {
                if k == Key::Enter {
                    self.act(cx, RAct::ConfirmTurnIn);
                }
                return;
            }
            if !r.shop_qid.is_empty() {
                return;
            }
            match k {
                Key::Char(c) if c.is_ascii_digit() && c != '0' => self.act(cx, RAct::Choose(c as i64 - '1' as i64)),
                Key::Enter | Key::Space => {
                    let view = r.view(&cx.data, cx.gs());
                    if json::b(&view, "next", false) || json::b(&view, "end", false) {
                        self.act(cx, RAct::Advance);
                    }
                }
                _ => {}
            }
            return;
        }
        if self.lore.is_some() {
            if matches!(k, Key::Enter | Key::Space) {
                self.lore = None;
            }
            return;
        }
        if let Some(dir) = k.dir() {
            self.walk.clear();
            let t = explore::template_of(&cx.data, explore::location(&cx.data, cx.gs()));
            if !t.is_empty() {
                self.step(cx, dir);
            }
        } else if k == Key::Char('e') || k == Key::Enter {
            let dir = self.facing;
            self.interact(cx, dir);
        }
    }

    fn back(&mut self, cx: &mut Cx) {
        if self.lore.is_some() {
            self.lore = None;
        } else if self.pending.is_some() {
            self.act(cx, RAct::CancelTurnIn);
        } else if self.runner.as_ref().is_some_and(|r| !r.shop_qid.is_empty()) {
            self.act(cx, RAct::CloseShop);
        } else if self.talking() {
            self.act(cx, RAct::Farewell);
        } else {
            self.walk.clear();
            self.act(cx, RAct::Menu);
        }
    }

    fn state(&self, cx: &Cx) -> Value {
        let gs = cx.gs();
        json!({"location": gs.location_id, "position": [gs.position.0, gs.position.1], "facing": [self.facing.0, self.facing.1],
            "walking": !self.walk.is_empty(), "log": cx.log.iter().rev().take(8).rev().collect::<Vec<_>>(),
            "dialogue": self.runner.as_ref().filter(|r| !r.closed).map(|r| Value::Object(r.view(&cx.data, gs))),
            "turn_in": self.pending.as_ref().map(|p| json!({"quest": p["quest"], "offers": p["offers"], "picks": self.picks})),
            "lore": self.lore.as_ref().map(|l| l.0.clone())})
    }

    widget_plumbing!();
}

const FRAME_LINE: Rgba = rgb(140, 196, 236);

impl RegionMap {
    fn draw_shop(&mut self, ui: &mut Ui<RAct>, cx: &Cx, sq: &str) {
        let d = &cx.data;
        let gs = cx.gs();
        let sh = shop::shop_of(d, gs, sq);
        let fr = Rect::new(133, 47, 373, 266);
        ui.p.panel(fr, crate::ui::MODAL, FRAME_LINE, 4.0);
        ui.p.text(fr.x as f32 + 9.0, fr.y as f32 + 7.0, &s_or(sh, "name", sq), 10.0, GOLD);
        ui.p.text_right((fr.x + fr.w) as f32 - 9.0, fr.y as f32 + 9.0, &format!("Gold: {}  Supplies: {}/99", gs.party.gold, gs.party.supplies), BODY, TEXT);
        let fq = shop::keeper_faction(d, gs, sh);
        if !fq.is_empty() {
            ui.p.text(fr.x as f32 + 9.0, fr.y as f32 + 25.0, &format!("{}: {}", s(d.resolve(&gs.world_id, "factions", &fq), "name"), shop::keeper_band(d, gs, sh)), BODY, DIM);
        }
        ui.tab("shop.buy", Rect::new(fr.x + 9, fr.y + 40, 28, 17), "Buy", RAct::ShopTab(true), self.shop_buy);
        ui.tab("shop.sell", Rect::new(fr.x + 39, fr.y + 40, 28, 17), "Sell", RAct::ShopTab(false), !self.shop_buy);
        let mut y = fr.y + 62;
        if self.shop_buy {
            for o in shop::offers(d, gs, sq) {
                let kind = s(&o, "kind");
                let left = json::i(&o, "left", -1);
                let (label, id, item) = if kind == shop::SUPPLIES { ("Supplies".to_string(), shop::SUPPLIES.to_string(), String::new()) } else { (d.item_name(&s(&o, "item")), s(&o, "id"), s(&o, "item")) };
                let price = json::i(&o, "price", 0);
                let counts: &[i64] = if kind == shop::SUPPLIES { &[1, 5] } else { &[1] };
                for &n in counts {
                    let mut text = format!("{label} x{} - {} gold", if kind == "loot" { json::i(&o, "count", 1) } else { n }, price * n);
                    if left >= 0 && kind != shop::SUPPLIES {
                        text = format!("{text}  ({left} left)");
                    }
                    let rr = Rect::new(fr.x + 9, y, fr.w - 18, 17);
                    let ok = left != 0 && gs.party.gold >= price * n;
                    ui.button_full(&format!("shop.offer.{id}.{n}"), rr, &text, RAct::Buy(kind.clone(), id.clone(), n), ok, false, BODY, tip(&label, &item_line(cx, &item)));
                    if ui.hover(rr) && !item.is_empty() {
                        self.shop_detail = item.clone();
                    }
                    y += 21;
                }
            }
        } else {
            let list = shop::sell_list(d, gs, sq);
            if list.is_empty() {
                ui.p.text(fr.x as f32 + 9.0, y as f32, "Nothing here the shop buys.", BODY, DIM);
            }
            for r in list {
                let item = s(&r, "item");
                let rr = Rect::new(fr.x + 9, y, fr.w - 18, 17);
                ui.button_full(&format!("shop.sell.{item}"), rr, &format!("{} x{} - {} gold each", d.item_name(&item), json::i(&r, "count", 0), json::i(&r, "price", 0)), RAct::Sell(item.clone()), true, false, BODY, tip(&d.item_name(&item), &item_line(cx, &item)));
                if ui.hover(rr) {
                    self.shop_detail = item.clone();
                }
                y += 21;
            }
        }
        if !self.shop_detail.is_empty() {
            ui.p.text_fit(fr.x as f32 + 9.0, (fr.y + fr.h) as f32 - 36.0, &format!("{}: {}", d.item_name(&self.shop_detail), item_line(cx, &self.shop_detail)), BODY, (fr.w - 18) as f32, INFO);
        }
        ui.p.text(fr.x as f32 + 9.0, (fr.y + fr.h) as f32 - 17.0, "Esc or right click: close", SMALL, DIM);
        ui.button("shop.close", Rect::new(fr.x + fr.w - 44, fr.y + fr.h - 22, 35, 17), "Close", RAct::CloseShop, true);
    }

    fn draw_turn_in(&mut self, ui: &mut Ui<RAct>, cx: &Cx, p: &Dict) {
        let d = &cx.data;
        let gs = cx.gs();
        let qid = s(p, "quest");
        let q = quest::resolved(d, gs, &qid);
        let fr = Rect::new(160, 60, 320, 230);
        ui.p.fill(Rect::new(0, 0, W, H), rgba(0, 0, 0, 100));
        ui.frame(fr, &format!("Turn in: {}", s_or(&q, "name", &qid)), false);
        let mut y = fr.y + 30;
        let rewards = crate::hub::reward_lines(cx, json::dget(&q, "rewards"));
        if !rewards.is_empty() {
            ui.p.text_fit(fr.x as f32 + 10.0, y as f32, &format!("Rewards: {}", rewards.join(", ")), BODY, (fr.w - 20) as f32, TEXT);
            y += 14;
        }
        let allowed = json::i(p, "picks_allowed", 0);
        let offers = json::sget(p, "offers");
        if allowed > 0 {
            ui.p.text(fr.x as f32 + 10.0, y as f32, &format!("Choose {allowed} of:"), BODY, GOLD);
            y += 13;
            for lq in offers {
                let loot = d.resolve(&gs.world_id, "loot", &lq);
                let on = self.picks.contains(&lq);
                ui.checkbox(&format!("turn_in.pick.{lq}"), fr.x + 14, y, &s_or(loot, "name", &lq), on, RAct::TogglePick(lq.clone()), true);
                y += 14;
            }
        }
        let rec = gs.party.members.get(self.recipient).map(|m| crate::hub::member_name(cx, m)).unwrap_or_default();
        ui.p.text(fr.x as f32 + 10.0, y as f32 + 5.0, "Passive / skill rewards go to", BODY, DIM);
        ui.dropdown("turn_in.recipient", Rect::new(fr.x + 150, y + 2, 120, 15), &rec, RAct::Recipient, gs.party.members.len() > 1, Vec::new());
        ui.button("turn_in.cancel", Rect::new(fr.x + fr.w - 120, fr.y + fr.h - 24, 52, 16), "Cancel", RAct::CancelTurnIn, true);
        ui.button("turn_in.confirm", Rect::new(fr.x + fr.w - 62, fr.y + fr.h - 24, 52, 16), "Confirm", RAct::ConfirmTurnIn, self.picks.len() as i64 <= allowed);
    }
}

/// "+90 HP · Owned 2 · Anyone can use it" style detail of an item.
pub fn item_line(cx: &Cx, item: &str) -> String {
    if item.is_empty() {
        return "Travel supplies: one per day on the road.".into();
    }
    let e = cx.data.entry("items", item);
    let owned = cx.game.as_ref().map(|g| g.gs.party.stash.get(item).copied().unwrap_or(0)).unwrap_or(0);
    let mut parts = Vec::new();
    let desc = s(e, "description");
    if !desc.is_empty() {
        parts.push(desc);
    }
    parts.push(format!("{} · Owned {owned}", s_or(e, "item_type", "ITEM").to_lowercase()));
    parts.join(" · ")
}
