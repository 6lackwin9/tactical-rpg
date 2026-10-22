//! The region map (scripts/ui/overworld/region_map_view.gd + dialogue_panel.gd + shop_panel.gd): walking a
//! location's map over the game state (ExploreRunner), the "Here" list and event log, the dialogue dock at the bottom
//! (DialogueRunner), the shop popup (ShopRules), the turn-in picker and lore pages. Fights start the battle screen;
//! a world-map exit opens the world map. Map-less locations show their exits as buttons.

use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use crate::music;
use crate::screens::{show_errors, Cx, GameCx, Key, Nav, Screen};
use aegis_ui::ui::{tip, Gfx, Glyph, Tip, Ui, Widget, BODY, BORDER, CARD, DIM, FOE, GOLD, GOOD, H, INFO, SMALL, TEXT, TITLE, W};
use aegis_ui::widget_plumbing;
use aegis_core::json::{self, s, s_or, Dict};
use emberwake_world::dialogue::DialogueRunner;
use emberwake_world::explore::{self, Pos};
use emberwake_world::state::SUPPLIES_MAX;
use emberwake_world::{quest, shop};
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
    /// A HUD bar button (PLAYUI1-H).
    Hud(Hud),
    Choose(i64),
    Advance,
    Farewell,
    ShopTab(bool),
    Buy(String, String, i64),
    Sell(String),
    CloseShop,
    TogglePick(String),
    PickRecipient(usize),
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

/// Starts a world fight: the battle screen, or the Skip / Fled / Lost popup in a debug game.
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

    /// Opens the conversation with `character` (also the UI Gallery's dialogue and shop entries).
    pub fn talk(&mut self, cx: &mut Cx, character: &str) {
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
            RAct::Hud(h) => {
                self.walk.clear();
                hud_open(cx, h, &REGION_HELP);
            }
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
            RAct::PickRecipient(i) => self.recipient = i,
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

impl Screen<GameCx> for RegionMap {
    /// The arrow keys walk the party.
    fn arrow_focus(&self) -> bool {
        false
    }
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
            // ASSETORG2 4.5: the world lineage's tile sets, child first, then common
            let mut lineage = d.lineage(&gs.world_id);
            lineage.reverse();
            let sets = emberwake_content::art::region_sets(&lineage);
            let set_refs: Vec<&str> = sets.iter().map(String::as_str).collect();
            for (y, row) in explore::rows_of(t).iter().enumerate() {
                for (x, ch) in row.iter().enumerate() {
                    let r = Rect::new(ox + x as i32 * ts, oy + y as i32 * ts, ts, ts);
                    if r.y + r.h <= map_area.y || r.y >= map_area.y + map_area.h || r.x + r.w <= map_area.x || r.x >= map_area.x + map_area.w {
                        continue;
                    }
                    let tile = emberwake_content::art::region_tile(*ch);
                    match tile.and_then(|t| cx.art.tile_in(&set_refs, t, x as i32, y as i32, ts)) {
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
                    let (body, palette, _) = crate::screens::character_art(&d, who);
                    ui.p.circle(cx_, cy_, ts as f32 * 0.42, col);
                    let tok = Rect::new(r.x + ts / 8, r.y + ts / 8, ts - ts / 4, ts - ts / 4);
                    match cx.art.first_token(&emberwake_content::art::body_chain(&body), &palette, tok.w) {
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
        // -- HUD bar (PLAYUI1-H): the location name; gold, supplies and the icon buttons at the right
        ui.p.fill(Rect::new(0, 0, W, MAP.y), rgba(10, 14, 22, 255));
        let name = s_or(loc, "name", "").to_string();
        let why = if modal { "Finish the conversation, shop or page first." } else { "" };
        hud_bar(&mut ui, cx, 4, &name, ("region.menu", "Esc"), why, &REGION_HELP, RAct::Hud);
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
            let (_, _, portraits) = crate::screens::character_art(&d, speaker);
            match cx.art.first(&portraits, 89, 89) {
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
        } else if let Some(h) = hud_key(k) {
            self.act(cx, RAct::Hud(h));
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

    /// MUSIC2: shop while the shop is open, else the location's kind.
    fn music(&self, cx: &Cx) -> Option<String> {
        let shop = self.runner.as_ref().is_some_and(|r| !r.closed && !r.shop_qid.is_empty());
        cx.game.as_ref().and_then(|g| music::game_track(cx, music::region_context(&cx.data, &g.gs, shop)))
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
        ui.p.panel(fr, aegis_ui::ui::MODAL, FRAME_LINE, 4.0);
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
        let names: Vec<String> = gs.party.members.iter().map(|m| crate::hub::member_name(cx, m)).collect();
        ui.p.text(fr.x as f32 + 10.0, y as f32 + 5.0, "Passive / skill rewards go to", BODY, DIM);
        ui.select("turn_in.recipient", Rect::new(fr.x + 150, y + 2, 120, 15), &names, self.recipient, RAct::PickRecipient, gs.party.members.len() > 1, Vec::new());
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

// ---- HUD bar (PLAYUI1-H, spec PLAYUI1 3.4): the top bar of the region map and the world map ----------------------

/// The HUD bar's height (logical px).
pub const HUD_H: i32 = 20;
/// An icon button's hit box (it holds a 14 px icon).
const HUD_BOX: i32 = 18;

/// A HUD bar icon button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hud {
    Party,
    Journal,
    Stash,
    Help,
    Menu,
}

/// The buttons left to right: (button, widget id, name, key, icon in the asset layers, drawn stand-in, hover text). The
/// Menu button's id and key are the screen's own (`hud_bar`'s `menu`).
const HUD_BUTTONS: [(Hud, &str, &str, &str, &str, Glyph, &str); 5] = [
    (Hud::Party, "hud.party", "Party", "P", "ui/hud/party.png", Glyph::Letter('P'), "Roster, level up, skills and gear."),
    (Hud::Journal, "hud.journal", "Journal", "J", "ui/hud/journal.png", Glyph::Letter('J'), "Quests ready, active, on hold and completed."),
    (Hud::Stash, "hud.stash", "Stash", "I", "ui/hud/stash.png", Glyph::Letter('I'), "The party's shared stash, beside the selected hero's gear."),
    (Hud::Help, "hud.help", "Help", "?", "ui/help.png", Glyph::Help, ""),
    (Hud::Menu, "", "Game menu", "", "ui/hud/menu.png", Glyph::Menu, "Party, journal, save, load, settings."),
];

/// The region map's controls (the Help button's hover card and popup).
pub const REGION_HELP: [&str; 6] = [
    "Arrows or WASD: move",
    "E or Enter: use what the party faces",
    "Click: walk there, or on a marker to use it",
    "1-9: pick a reply in a conversation",
    "P: party   J: journal   I: stash",
    "Esc: back, or the game menu",
];

/// The HUD bar's keys: P party, J journal, I stash, ? help (Esc stays each screen's own).
pub fn hud_key(k: Key) -> Option<Hud> {
    match k {
        Key::Char(c) => match c.to_ascii_lowercase() {
            'p' => Some(Hud::Party),
            'j' => Some(Hud::Journal),
            'i' => Some(Hud::Stash),
            '?' => Some(Hud::Help),
            _ => None,
        },
        _ => None,
    }
}

/// What a HUD button opens: Party the Character popup on the first active member's Stats tab, Stash the same popup's
/// Gear tab (where the party's shared stash lives), Journal the journal (a notice while there are no quests), Help the
/// controls `help`, Menu the game menu.
pub fn hud_open(cx: &mut Cx, h: Hud, help: &[&str]) {
    let first = cx.gs().party.active_members().first().map(|m| m.id.clone()).unwrap_or_default();
    match h {
        Hud::Party => cx.nav.push(Nav::Push(Box::new(crate::party::Character::new(cx, &first, "stats")))),
        Hud::Stash => cx.nav.push(Nav::Push(Box::new(crate::party::Character::new(cx, &first, "gear")))),
        Hud::Journal if !crate::hub::has_quests(cx) => show_errors(cx, vec!["No quests yet.".into()]),
        Hud::Journal => cx.nav.push(Nav::Push(Box::new(crate::hub::Journal::default()))),
        Hud::Help => {
            let lines = std::iter::once("Controls".to_string()).chain(help.iter().map(|l| l.to_string())).collect();
            cx.nav.push(Nav::Push(Box::new(aegis_ui::screen::Notice::new(lines))));
        }
        Hud::Menu => cx.nav.push(Nav::Push(Box::new(crate::hub::Hub::default()))),
    }
}

/// Draws the HUD bar: `name` (gold) from `left`, then at the right gold, supplies and the icon buttons Party, Journal,
/// Stash, Help and Menu (icons from the asset layers, drawn shapes while a file is missing). `menu` is the Menu
/// button's widget id and key; `why` (when not empty) disables every button but Help and says why on its hover card;
/// `help` lists the controls; `wrap` makes the screen's action of a button.
#[allow(clippy::too_many_arguments)]
pub fn hud_bar<A: Clone>(ui: &mut Ui<A>, cx: &mut Cx, left: i32, name: &str, menu: (&str, &str), why: &str, help: &[&str], wrap: impl Fn(Hud) -> A) {
    ui.p.fill(Rect::new(0, 0, W, HUD_H), rgba(10, 14, 22, 255));
    let (gold, supplies) = cx.game.as_ref().map(|g| (g.gs.party.gold, g.gs.party.supplies)).unwrap_or((0, 0));
    let journal = crate::hub::has_quests(cx);
    let px = ui.icon_px(Rect::new(0, 0, HUD_BOX, HUD_BOX));
    let start = W - 3 - HUD_BUTTONS.len() as i32 * (HUD_BOX + 2) + 2;
    for (i, (h, id, label, key, rel, glyph, body)) in HUD_BUTTONS.into_iter().enumerate() {
        let r = Rect::new(start + i as i32 * (HUD_BOX + 2), (HUD_H - HUD_BOX) / 2, HUD_BOX, HUD_BOX);
        let (id, key) = if h == Hud::Menu { menu } else { (id, key) };
        let title = if key.is_empty() { label.to_string() } else { format!("{label} ({key})") };
        let mut t: Tip = tip(&title, body);
        if h == Hud::Help {
            t.extend(help.iter().map(|l| (l.to_string(), TEXT)));
        }
        let off = match h {
            Hud::Help => "",
            _ if !why.is_empty() => why,
            Hud::Journal if !journal => "No quests yet.",
            _ => "",
        };
        if !off.is_empty() {
            t.push((off.to_string(), FOE));
        }
        ui.icon_button(id, r, label, cx.art.image(rel, px, px), glyph, wrap(h), off.is_empty(), false, t);
    }
    // supplies, then gold, right to left: an icon and the amount
    let icon = Ui::<A>::icon_rect(Rect::new(0, 0, HUD_BOX, HUD_BOX)).w;
    let ty = (HUD_H as f32 - ui.p.line_h(BODY)) / 2.0;
    let mut right = start - 6;
    let counters = [
        ("hud.supplies", "Supplies", "ui/hud/supplies.png", format!("{supplies}/{SUPPLIES_MAX}"), "Food for the road: each day of travel eats some, and so does making camp."),
        ("hud.gold", "Gold", "ui/hud/gold.png", gold.to_string(), "Earned from fights, chests and quests; spent in shops."),
    ];
    for (id, label, rel, amount, body) in counters {
        let tw = ui.p.text_w(&amount, BODY).ceil() as i32;
        let r = Rect::new(right - (icon + 3 + tw), 0, icon + 3 + tw, HUD_H);
        let ir = Rect::new(r.x, (HUD_H - icon) / 2, icon, icon);
        match cx.art.image(rel, px, px) {
            Some(img) => ui.p.image(img, ir),
            None => counter_glyph(ui, id, ir),
        }
        ui.p.text((ir.x + icon + 3) as f32, ty, &amount, BODY, TEXT);
        ui.push(id, "label", amount.clone(), r, true, false, None, tip(label, body));
        right = r.x - 8;
    }
    ui.p.text_fit(left as f32, (HUD_H as f32 - ui.p.line_h(TITLE)) / 2.0, name, TITLE, (right - left - 4) as f32, GOLD);
}

/// The drawn stand-in of the gold or supplies icon: a coin, or a tied sack.
fn counter_glyph<A>(ui: &mut Ui<A>, id: &str, r: Rect) {
    let (cx_, cy_, s) = (r.x as f32 + r.w as f32 / 2.0, r.y as f32 + r.h as f32 / 2.0, r.w as f32);
    if id == "hud.gold" {
        ui.p.circle(cx_, cy_, s * 0.42, rgb(150, 110, 30));
        ui.p.circle(cx_, cy_, s * 0.32, GOLD);
    } else {
        ui.p.panel(Rect::new(r.x + r.w / 6, r.y + r.h / 3, r.w - r.w / 3, r.h - r.h / 3), rgb(150, 112, 70), rgb(96, 70, 40), 3.0);
        ui.p.line(cx_ - s * 0.18, r.y as f32 + s * 0.2, cx_ + s * 0.18, r.y as f32 + s * 0.2, rgb(196, 160, 110), 0.0);
    }
}

#[cfg(test)]
mod tests {
    use aegis_core::assets::Layers;
    use std::path::Path;

    /// ASSETORG2 section 3: every file under each asset root (the game's, its kit's, the engine's) matches a pattern of
    /// the layered tree (README.md files aside), and the region map's tiles of the Ashlands are there.
    #[test]
    fn the_assets_follow_the_layered_tree() {
        let game = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let layers = Layers::for_game(&game);
        assert_eq!(layers.roots.len(), 3, "game, kit and engine roots: {:?}", layers.notices);
        let mut stray = Vec::new();
        for root in &layers.roots {
            stray.extend(strays(root));
        }
        assert!(stray.is_empty(), "files outside the layered tree (docs/specs/ASSETORG2.md section 3): {stray:?}");
        for t in ["floor", "wall", "shallow_water"] {
            assert_eq!(layers.variants(&format!("tilesets/ashlands/{t}")), 3, "{t}");
        }
    }

    fn strays(root: &Path) -> Vec<String> {
        let mut stray = Vec::new();
        let mut walk = vec![root.to_path_buf()];
        while let Some(dir) = walk.pop() {
            for e in std::fs::read_dir(&dir).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk.push(p);
                    continue;
                }
                let rel = p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
                if !layout_ok(&rel) {
                    stray.push(format!("{}: {rel}", root.display()));
                }
            }
        }
        stray
    }

    /// One path of the section 3 tree (game, kit or engine root), plus the PLAYUI1 section 6 icons: toolkit glyphs
    /// `ui/<id>.png` and the `ui/hud`, `ui/markers` and `ui/services` folders.
    fn layout_ok(rel: &str) -> bool {
        let p: Vec<&str> = rel.split('/').collect();
        let png = |s: &str| s.ends_with(".png");
        let v = |s: &str| s.starts_with('v') && s.trim_start_matches('v').trim_end_matches(".png").parse::<u32>().is_ok() && png(s);
        if p.last() == Some(&"README.md") || matches!(p.as_slice(), ["fonts", ..]) {
            return true;
        }
        match p.as_slice() {
            ["units", _, _, "sheet.png" | "clips.json"] => true,
            ["units", _, _, "palettes", f] => f.ends_with(".json"),
            ["portraits", "common" | "heroes" | "npcs" | "elites" | "bosses", f] => png(f),
            ["items", _, f] => png(f),
            ["skills", "kinds", f] | ["skills", f] | ["statuses", f] | ["map", "markers", f] => png(f),
            ["ui", "pins" | "badges" | "chips" | "intents" | "hud" | "markers" | "services", f] | ["ui", f] => png(f),
            ["tilesets", _, _, f] => v(f),
            ["backgrounds", "menu" | "title" | "world_map" | "location", f] => png(f),
            ["audio", "sfx", f] => f.ends_with(".wav"),
            ["audio", "music", f] => f.ends_with(".ogg"),
            _ => false,
        }
    }
}

/// PLAYUI1-H: the HUD bar of the region map and the world map, drawn headless over a debug game of the first playable
/// world (nothing saved; the art layers are off, so every icon is its drawn stand-in).
#[cfg(test)]
mod hud_tests {
    use super::*;
    use crate::screens::{default_settings, Game};
    use aegis_ui::painter::{Fonts, TextRec, View};
    use aegis_ui::screen::{Settings, TEXT_SIZES};
    use emberwake_world::Data;
    use std::path::Path;
    use std::sync::Arc;

    fn game_cx() -> Cx {
        let tmp = std::env::temp_dir().join(format!("emberwake_hud_{}", std::process::id()));
        let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        let data = Arc::new(Data::open(&dir).expect("data").0);
        let world = emberwake_world::new_game::playable_worlds(&data).first().cloned().expect("a playable world");
        let gs = emberwake_world::new_game::create_for_world(&data, &world, 1).expect("a new game");
        let clean = gs.to_value();
        let app = GameCx {
            data,
            manifest: Default::default(),
            game: Some(Game { gs, slot: -1, clean, busy: false, sample: true }),
            save_dir: tmp.join("saves"),
            tuning: aegis_ui::tuning::TuningFile::new(None),
            log: Vec::new(),
            encounter: String::new(),
            seed: None,
            debug: true,
            data_dir: dir,
        };
        Cx {
            settings: Settings::load(tmp.join("settings.json"), default_settings()),
            art: aegis_ui::art::Art::new(None),
            audio: aegis_ui::audio::Audio::offline(None),
            nav: Vec::new(),
            notice: Vec::new(),
            shot_dir: tmp,
            quit: false,
            answers: Vec::new(),
            playtest: aegis_ui::playtest::Playtests::off(),
            app,
        }
    }

    fn gfx() -> Gfx {
        let font = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../engine/assets/fonts/OpenSans-SemiBold.ttf")).expect("font");
        Gfx { canvas: aegis_ui::canvas::Canvas::new(W, H), view: View::fit(W as u32, H as u32), fonts: Fonts::new(&font), kit: Default::default() }
    }

    /// One frame of `screen` at text size `scale` (the mouse off the canvas); the texts drawn.
    fn frame(screen: &mut dyn Screen<GameCx>, cx: &mut Cx, g: &mut Gfx, scale: f32) -> Vec<TextRec> {
        g.fonts.scale = scale;
        g.fonts.audit = Some(Vec::new());
        screen.draw(cx, g, (-500, -500), true);
        g.fonts.audit.take().unwrap_or_default()
    }

    fn rect(w: &Value) -> (i64, i64, i64, i64) {
        let n = |i: usize| w["rect"][i].as_i64().unwrap_or(0);
        (n(0), n(1), n(2), n(3))
    }

    /// The bar's widgets, left to right: (id, kind, enabled).
    fn bar(screen: &dyn Screen<GameCx>) -> Vec<(String, String, bool)> {
        let mut w: Vec<Value> = screen.widgets().into_iter().filter(|w| rect(w).1 + rect(w).3 <= HUD_H as i64).collect();
        w.sort_by_key(|w| rect(w).0);
        w.iter().map(|w| (s(json::dict(w), "id"), s(json::dict(w), "kind"), json::b(json::dict(w), "enabled", false))).collect()
    }

    fn ids(b: &[(String, String, bool)]) -> Vec<&str> {
        b.iter().map(|w| w.0.as_str()).collect()
    }

    /// The screen the last navigation opened: (id, state); the navigation is cleared.
    fn opened(cx: &mut Cx) -> (String, Value) {
        let out = match cx.nav.last() {
            Some(Nav::Push(s)) => (s.id().to_string(), s.state(cx)),
            _ => (String::new(), Value::Null),
        };
        cx.nav.clear();
        out
    }

    #[test]
    fn the_region_map_bar_is_the_name_then_gold_supplies_and_five_icon_buttons() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        let recs = frame(&mut map, &mut cx, &mut g, 1.0);
        let b = bar(&map);
        assert_eq!(ids(&b), ["hud.gold", "hud.supplies", "hud.party", "hud.journal", "hud.stash", "hud.help", "region.menu"], "{b:?}");
        assert!(b[2..].iter().all(|w| w.1 == "icon_button"), "{b:?}");
        let gs = cx.gs().clone();
        let name = s_or(explore::location(&cx.data, &gs), "name", "").to_string();
        let top: Vec<&TextRec> = recs.iter().filter(|r| r.y + r.h / 2.0 < HUD_H as f32).collect();
        let at = |t: &str| top.iter().find(|r| r.text == t).map(|r| r.x);
        assert!(!name.is_empty() && at(&name).is_some_and(|x| x < 10.0), "the location name at the top left: {top:?}");
        let (gold, supplies) = (at(&gs.party.gold.to_string()), at(&format!("{}/{SUPPLIES_MAX}", gs.party.supplies)));
        assert!(gold.zip(supplies).is_some_and(|(g, s)| g < s && g > W as f32 / 2.0), "gold, then supplies at the right: {top:?}");
        assert!(!recs.iter().any(|r| r.text.contains("WASD") || r.text.starts_with("Supplies")), "the help line and the supplies text left the top left");
        let widgets = map.widgets();
        let card = |id: &str| widgets.iter().find(|w| w["id"] == id).map(|w| w["tip"].to_string()).unwrap_or_default();
        for (id, key) in [("hud.party", "(P)"), ("hud.journal", "(J)"), ("hud.stash", "(I)"), ("hud.help", "(?)"), ("region.menu", "(Esc)")] {
            assert!(card(id).contains(key), "{id}'s card names its key: {}", card(id));
        }
        assert!(card("hud.help").contains("WASD"), "Help's card lists the controls");
    }

    #[test]
    fn the_bar_buttons_open_party_journal_stash_help_and_menu() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let first = cx.gs().party.active_members().first().map(|m| m.id.clone()).expect("an active member");
        map.click_id(&mut cx, "hud.party").expect("party");
        let (id, state) = opened(&mut cx);
        assert_eq!((id.as_str(), state["member"].clone(), state["tab"].clone()), ("character", json!(first), json!("stats")));
        map.click_id(&mut cx, "hud.stash").expect("stash");
        let (id, state) = opened(&mut cx);
        assert_eq!((id.as_str(), state["member"].clone(), state["tab"].clone()), ("character", json!(first), json!("gear")), "the shared stash is on the Gear tab");
        map.click_id(&mut cx, "hud.help").expect("help");
        let (id, state) = opened(&mut cx);
        assert_eq!(id, "notice");
        assert_eq!(state["lines"].as_array().map(|l| l.len()), Some(REGION_HELP.len() + 1), "a title and the controls");
        map.click_id(&mut cx, "region.menu").expect("menu");
        assert_eq!(opened(&mut cx).0, "hub");
        // Journal: on when there are quests (the Hub's rule), else off with the reason on its card
        let journal = crate::hub::has_quests(&cx);
        let b = bar(&map);
        assert_eq!(b.iter().find(|w| w.0 == "hud.journal").map(|w| w.2), Some(journal), "{b:?}");
        if journal {
            map.click_id(&mut cx, "hud.journal").expect("journal");
            assert_eq!(opened(&mut cx).0, "journal");
        }
        for (k, want) in [('i', "character"), ('P', "character"), ('?', "notice"), ('j', if journal { "journal" } else { "notice" })] {
            map.key(&mut cx, Key::Char(k));
            assert_eq!(opened(&mut cx).0, want, "key {k}");
        }
    }

    #[test]
    fn a_popup_turns_the_bar_buttons_off_but_help_and_the_counters_stay() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        map.lore = Some(("A page".into(), "Its text.".into()));
        frame(&mut map, &mut cx, &mut g, 1.0);
        let on: Vec<(String, bool)> = bar(&map).into_iter().map(|w| (w.0, w.2)).collect();
        let want = [("hud.gold", true), ("hud.supplies", true), ("hud.party", false), ("hud.journal", false), ("hud.stash", false), ("hud.help", true), ("region.menu", false)];
        assert_eq!(on, want.map(|(id, e)| (id.to_string(), e)));
        map.key(&mut cx, Key::Char('i'));
        assert!(cx.nav.is_empty(), "the keys wait too");
    }

    #[test]
    fn the_world_map_gets_the_same_bar() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut wm = crate::worldmap::WorldMap::new();
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let b = bar(&wm);
        assert_eq!(ids(&b), ["world_map.back", "hud.gold", "hud.supplies", "hud.party", "hud.journal", "hud.stash", "hud.help", "world_map.menu"], "{b:?}");
        wm.click_id(&mut cx, "hud.stash").expect("stash");
        assert_eq!(opened(&mut cx).1["tab"], json!("gear"));
        wm.key(&mut cx, Key::Char('p'));
        assert_eq!(opened(&mut cx).1["tab"], json!("stats"));
        wm.click_id(&mut cx, "world_map.menu").expect("menu");
        assert_eq!(opened(&mut cx).0, "hub");
    }

    /// TEXTFIT1: the bar's texts fit (nothing cut, past the canvas, overlapping or out of its control) and stay in the
    /// bar at every text size, on both maps.
    #[test]
    fn the_bar_reads_clean_at_every_text_size() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        let mut wm = crate::worldmap::WorldMap::new();
        let screens: [&mut dyn Screen<GameCx>; 2] = [&mut map, &mut wm];
        for screen in screens {
            for (size, scale) in TEXT_SIZES {
                let recs: Vec<TextRec> = frame(screen, &mut cx, &mut g, scale as f32).into_iter().filter(|r| r.y + r.h / 2.0 < HUD_H as f32).collect();
                let boxes: Vec<(String, f32, f32, f32, f32)> = screen
                    .widgets()
                    .iter()
                    .filter(|w| rect(w).1 < HUD_H as i64)
                    .map(|w| {
                        let (x, y, ww, hh) = rect(w);
                        (s(json::dict(w), "id"), x as f32, y as f32, ww as f32, hh as f32)
                    })
                    .collect();
                let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
                assert_eq!(v["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "{} at {size}: {v}", screen.id());
                for r in &recs {
                    assert!(r.y >= 0.0 && r.y + r.h <= HUD_H as f32 + 0.5, "{} at {size}: '{}' leaves the bar ({} to {})", screen.id(), r.text, r.y, r.y + r.h);
                }
            }
        }
    }
}
