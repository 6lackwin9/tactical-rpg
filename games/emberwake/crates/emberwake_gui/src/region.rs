//! The region map (scripts/ui/overworld/region_map_view.gd + dialogue_panel.gd + shop_panel.gd): walking a
//! location's map over the game state (ExploreRunner) under the HUD bar, the floating "Here" legend and the event log
//! ticker (PLAYUI1-M: the full-width map, a camera centred on the party with walls drawn past the edge), the dialogue
//! box at the bottom centre (DialogueRunner; PLAYUI1-D: the camera keeps both speakers above it), the shop popup over a
//! dimmed map (ShopRules), the turn-in picker and lore pages. Fights start the battle screen; a world-map exit opens
//! the world map. Map-less locations show their exits as buttons.

use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use crate::hud::{hud_bar, hud_key, hud_open, Hud, HUD_H};
use crate::music;
use crate::screens::{show_errors, Cx, GameCx, Key, Nav, Screen};
use aegis_ui::ui::{draw_glyph, tip, Gfx, Glyph, Ui, Widget, BODY, BORDER, CARD, DIM, GOLD, GOOD, H, INFO, PANEL_HI, SMALL, TEXT, W};
use aegis_ui::painter::Painter;
use aegis_ui::widget_plumbing;
use aegis_core::json::{self, s, s_or, Dict};
use emberwake_world::dialogue::DialogueRunner;
use emberwake_world::explore::{self, Pos};
use emberwake_world::{quest, shop};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

/// A map tile (logical px; 32 px at 1280 wide, MAPVIEW1).
const TILE: i32 = 16;
/// The dialogue box (spec PLAYUI1 3.1, logical px at text size Normal): its width (times the text size, at most the
/// canvas less 40), its gap above the bottom edge, its least height (its most is `TALK_MAX_PCT` % of the canvas).
const TALK_W: i32 = 400;
const TALK_GAP: i32 = 6;
const TALK_MIN_H: i32 = 104;
const TALK_MAX_PCT: i32 = 45;
/// The left column (portrait, name, faction band) and the portrait in it.
const TALK_LEFT: i32 = 76;
const PORTRAIT: i32 = 64;
/// The box's inner margin.
const TALK_PAD: i32 = 6;
/// The NPC's line scrolls past this many lines, the replies past this many rows; a reply row is at least this tall.
const TALK_LINES: i32 = 4;
const TALK_ROWS: usize = 5;
const REPLY_H: i32 = 14;

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
    /// Fold or open the legend (`map_legend_open`).
    Legend,
    /// Show the last events in the ticker, or only the latest (L).
    Log,
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
    pub(crate) lore: Option<(String, String)>,
    walk: Vec<Pos>,
    walk_target: Option<String>,
    next_walk: Instant,
    /// Map geometry of the last frame: (origin x, origin y, tile size).
    geo: (i32, i32, i32),
    /// The ticker shows the last `LOG_OPEN` events (else only the latest).
    log_open: bool,
    /// The conversation node shown last ("character/node"): a new node starts its line and replies scrolled to the top.
    talk_node: String,
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
            log_open: false,
            talk_node: String::new(),
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
            RAct::Legend => {
                let open = cx.settings.v["map_legend_open"].as_bool().unwrap_or(true);
                cx.settings.v["map_legend_open"] = json!(!open);
                if let Err(e) = cx.settings.save() {
                    cx.log(format!("Settings could not be saved: {e}"));
                }
            }
            RAct::Log => self.log_open = !self.log_open,
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
        let gs = cx.gs().clone();
        // the open conversation; a new node starts its line and its replies scrolled to the top
        let talk_view = self.runner.as_ref().filter(|r| !r.closed).map(|r| r.view(&d, &gs));
        let node = self.runner.as_ref().filter(|r| !r.closed).map(|r| format!("{}/{}", r.character_qid, r.node_id)).unwrap_or_default();
        if node != self.talk_node {
            g.kit.scroll.remove("dialogue.text");
            g.kit.scroll.remove("dialogue.choices");
            self.talk_node = node;
        }
        let mut ui: Ui<RAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), rgb(14, 18, 26));
        let loc = explore::location(&d, &gs);
        let t = explore::template_of(&d, loc);
        let talking = self.talking();
        let modal = self.modal();
        let shop_qid = self.runner.as_ref().filter(|r| !r.closed).map(|r| r.shop_qid.clone()).unwrap_or_default();
        // measured before the map: the camera keeps both speakers above the box (also while the shop hides it, so the
        // map does not jump when the shop opens or closes)
        let talk_box = talk_view.as_ref().map(|v| talk_layout(&mut ui.p, v));
        // -- Map (PLAYUI1-M, spec PLAYUI1 3.2): the full width under the HUD bar; the camera centres on the party with no
        // clamp (while talking, on the midpoint of the party and the speaker, inside the map above the dialogue box), and
        // past the map's edge the location's wall tile is drawn
        let map_area = Rect::new(0, HUD_H, W, H - HUD_H);
        let focus_area = talk_box.as_ref().map_or(map_area, |b| Rect::new(0, HUD_H, W, b.r.y - HUD_H));
        let names_always = cx.settings.v["map_names"].as_str() == Some("always");
        if !t.is_empty() {
            let ts = TILE;
            let other = talk_view.as_ref().and_then(|v| self.speaker_tile(cx, v)).unwrap_or(gs.position);
            let ox = focus_area.x + focus_area.w / 2 - ((gs.position.0 + other.0) as i32 * ts + ts) / 2;
            let oy = focus_area.y + focus_area.h / 2 - ((gs.position.1 + other.1) as i32 * ts + ts) / 2;
            self.geo = (ox, oy, ts);
            // ASSETORG2 4.5: the world lineage's tile sets, child first, then common
            let mut lineage = d.lineage(&gs.world_id);
            lineage.reverse();
            let sets = emberwake_content::art::region_sets(&lineage);
            let set_refs: Vec<&str> = sets.iter().map(String::as_str).collect();
            ui.push_clip(map_area);
            let (x0, x1) = ((map_area.x - ox).div_euclid(ts), (map_area.x + map_area.w - ox - 1).div_euclid(ts));
            let (y0, y1) = ((map_area.y - oy).div_euclid(ts), (map_area.y + map_area.h - oy - 1).div_euclid(ts));
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let r = Rect::new(ox + x * ts, oy + y * ts, ts, ts);
                    // render only: off the map is the wall (`#`), nothing walkable
                    let ch = explore::tile(t, x as i64, y as i64).unwrap_or('#');
                    let tile = emberwake_content::art::region_tile(ch);
                    match tile.and_then(|t| cx.art.tile_in(&set_refs, t, x, y, ts)) {
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
                ui.push("map", "map", s_or(loc, "name", ""), map_area, true, false, Some(RAct::MapClick), Vec::new());
            }
            let steps = gs.explore_int("loc_steps");
            let ahead = (gs.position.0 + self.facing.0, gs.position.1 + self.facing.1);
            let legend_hover = self.legend_hover(live);
            // (marker, name, the name sits over a badge)
            let mut labels: Vec<(Rect, String, bool)> = Vec::new();
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
                let seen = r.intersect(map_area);
                if seen.w <= 0 || seen.h <= 0 {
                    continue;
                }
                let kind = s(&m, "kind");
                let id = s(&m, "id");
                let label = s(&view, "label");
                let (cx_, cy_) = (r.x as f32 + ts as f32 / 2.0, r.y as f32 + ts as f32 / 2.0);
                let mut services = Vec::new();
                if kind == "npc" {
                    let who = d.resolve(&gs.world_id, "characters", &s(&m, "character"));
                    services = explore::npc_services(&d, &gs, &s(who, "_qid"));
                    let (body, palette, _) = crate::screens::character_art(&d, who);
                    let (col, _) = Self::marker_style(&kind, &state);
                    ui.p.circle(cx_, cy_, ts as f32 * 0.42, col);
                    let tok = Rect::new(r.x + ts / 8, r.y + ts / 8, ts - ts / 4, ts - ts / 4);
                    match cx.art.first_token(&emberwake_content::art::body_chain(&body), &palette, tok.w) {
                        Some(img) => ui.p.image(img, tok),
                        None => ui.p.text_center(cx_, cy_ - 4.0, &s(who, "name").chars().next().unwrap_or('?').to_string(), BODY, rgb(20, 20, 20)),
                    }
                    // 2.2: the first service is the badge (turn_in and offer are today's `ready` and `quest`)
                    if let Some(first) = services.first() {
                        let br = Rect::new(r.x + ts - 5, r.y - 5, 9, 9);
                        let px = ui.icon_px(Rect::new(0, 0, br.w + 4, br.h + 4));
                        match cx.art.image(&service_icon(first), px, px) {
                            Some(img) => ui.p.image(img, br),
                            None => service_glyph(&mut ui, first, br),
                        }
                    }
                } else {
                    let px = ui.icon_px(Rect::new(0, 0, ts + 4, ts + 4));
                    match cx.art.image(&emberwake_content::art::marker(marker_icon(&kind, &state)), px, px) {
                        Some(img) => ui.p.image(img, r),
                        None => {
                            let (col, glyph) = Self::marker_style(&kind, &state);
                            ui.p.panel(r.inset(ts / 5), col, rgba(255, 255, 255, 90), 2.0);
                            ui.p.text_center(cx_, cy_ - 4.5, glyph, BODY, rgb(16, 16, 20));
                        }
                    }
                }
                // 2.3: names always (people and exits, MAPREAD1), or on hover: the marker or its legend row under the
                // pointer, or the marker the party faces
                let named = if names_always { kind == "npc" || kind == "exit" } else { (!modal && ui.hover(r)) || legend_hover.as_deref() == Some(id.as_str()) || at == ahead };
                if named {
                    labels.push((r, label.clone(), !services.is_empty()));
                }
                if !modal {
                    let mut t = tip(&label, &s(&view, "verb"));
                    if !services.is_empty() {
                        t.push((services_text(&services), INFO));
                    }
                    ui.push(format!("marker.{id}"), "marker", label.clone(), r, true, false, Some(RAct::Marker(id)), t);
                }
            }
            // the party
            let pr = Rect::new(ox + gs.position.0 as i32 * ts, oy + gs.position.1 as i32 * ts, ts, ts);
            let (pcx, pcy) = (pr.x as f32 + ts as f32 / 2.0, pr.y as f32 + ts as f32 / 2.0);
            ui.p.circle(pcx, pcy, ts as f32 * 0.3, rgb(250, 214, 60));
            ui.p.line(pcx, pcy, pcx + self.facing.0 as f32 * ts as f32 * 0.32, pcy + self.facing.1 as f32 * ts as f32 * 0.32, rgb(40, 30, 10), 0.0);
            ui.push("party", "party", format!("Party at {}, {}", gs.position.0, gs.position.1), pr, true, false, None, Vec::new());
            // the names over everything on the map, kept inside the map area
            for (r, label, badge) in labels {
                let w = ui.p.text_w(&label, SMALL);
                let lx = (r.x as f32 + ts as f32 / 2.0 - w / 2.0).min((map_area.x + map_area.w) as f32 - 2.0 - w).max(map_area.x as f32 + 2.0);
                let ly = (r.y as f32 - if badge { 5.0 } else { 1.0 } - ui.p.line_h(SMALL)).max(map_area.y as f32 + 1.0);
                ui.p.text(lx + 0.5, ly + 0.5, &label, SMALL, rgba(0, 0, 0, 200));
                ui.p.text(lx, ly, &label, SMALL, TEXT);
            }
            ui.pop_clip();
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
        let name = s_or(loc, "name", "").to_string();
        let why = if modal { "Finish the conversation, shop or page first." } else { "" };
        hud_bar(&mut ui, cx, 4, &name, ("region.menu", "Esc"), why, &REGION_HELP, RAct::Hud);
        // -- The legend (2.1 A) at the top right and the event log ticker (MAPLOG1) at the bottom left; off in a talk
        if !t.is_empty() && !talking {
            self.draw_legend(&mut ui, cx, modal);
        }
        if !talking {
            self.draw_ticker(&mut ui, cx, modal);
        }
        // -- The dialogue box (3.1); while the shop is open (3.3) the box hides and the shop sits centred over the dimmed map
        if let (Some(view), Some(b)) = (talk_view.as_ref(), talk_box.as_ref()) {
            if shop_qid.is_empty() {
                draw_talk(&mut ui, cx, view, b, self.pending.is_some());
            } else {
                ui.p.fill(map_area, SHOP_DIM);
                self.draw_shop(&mut ui, cx, &shop_qid);
            }
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
        } else if matches!(k, Key::Char(c) if c.eq_ignore_ascii_case(&'l')) {
            self.act(cx, RAct::Log);
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
            "lore": self.lore.as_ref().map(|l| l.0.clone()), "legend_open": cx.settings.v["map_legend_open"].as_bool().unwrap_or(true), "log_open": self.log_open})
    }

    /// MUSIC2: shop while the shop is open, else the location's kind.
    fn music(&self, cx: &Cx) -> Option<String> {
        let shop = self.runner.as_ref().is_some_and(|r| !r.closed && !r.shop_qid.is_empty());
        cx.game.as_ref().and_then(|g| music::game_track(cx, music::region_context(&cx.data, &g.gs, shop)))
    }

    widget_plumbing!();
}

const FRAME_LINE: Rgba = rgb(140, 196, 236);
/// The dim over the map while the shop is open (spec PLAYUI1 3.3).
const SHOP_DIM: Rgba = rgba(0, 0, 0, 150);

impl RegionMap {
    /// The tile of the person speaking (the node's speaker, else the one talked to): their marker here that is not
    /// absent, the nearest to the party when several show them; None when they have no marker on this map.
    fn speaker_tile(&self, cx: &Cx, view: &Dict) -> Option<Pos> {
        let d = &cx.data;
        let gs = cx.gs();
        let loc = explore::location(d, gs);
        let t = explore::template_of(d, loc);
        let loc_qid = s(loc, "_qid");
        let steps = gs.explore_int("loc_steps");
        let talked = self.runner.as_ref().map(|r| r.character_qid.clone()).unwrap_or_default();
        for who in [s(view, "speaker"), talked] {
            if who.is_empty() {
                continue;
            }
            let near = explore::markers_of(loc)
                .into_iter()
                .filter(|m| s(m, "kind") == "npc" && emberwake_world::dialogue::character_qid(d, gs, &s(m, "character")) == who && explore::presence(d, gs, &loc_qid, m) != "absent")
                .map(|m| explore::marker_pos(t, &m, steps))
                .filter(|at| at.0 >= 0)
                .min_by_key(|at| (at.0 - gs.position.0).abs() + (at.1 - gs.position.1).abs());
            if near.is_some() {
                return near;
            }
        }
        None
    }

    fn draw_shop(&mut self, ui: &mut Ui<RAct>, cx: &Cx, sq: &str) {
        let d = &cx.data;
        let gs = cx.gs();
        let sh = shop::shop_of(d, gs, sq);
        // centred under the HUD bar (spec PLAYUI1 3.3; PLAYUI1-P makes it the three-column trade)
        let (fw, fh) = (373, 266);
        let fr = Rect::new((W - fw) / 2, HUD_H + (H - HUD_H - fh) / 2, fw, fh);
        ui.p.panel(fr, aegis_ui::ui::MODAL, FRAME_LINE, 4.0);
        ui.push("shop", "panel", s_or(sh, "name", sq), fr, true, false, None, Vec::new());
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

impl RegionMap {
    /// The marker whose legend row the pointer was over in the last frame (2.3: its marker shows its name).
    fn legend_hover(&self, live: bool) -> Option<String> {
        if !live {
            return None;
        }
        self.w.iter().find(|w| w.kind == "row" && w.rect.contains(self.mouse.0, self.mouse.1)).and_then(|w| w.id.strip_prefix("here.")).map(str::to_string)
    }

    /// The legend (spec PLAYUI1 2.1 A, MAPNAV1-U's collapse): a pane floating at the map's top right under the HUD bar.
    /// Its header "Here (N)" with a chevron collapses it to the header (kept in the settings as `map_legend_open`); a
    /// row per marker shows the marker's map icon, its name and (people) their service icons at the right; spent
    /// markers dim. A row click walks there as before. Width from the texts, at most `LEGEND_MAX` (times the text size).
    fn draw_legend(&mut self, ui: &mut Ui<RAct>, cx: &mut Cx, modal: bool) {
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        let loc = explore::location(&d, &gs);
        // (id, kind, state, label, verb, services)
        let mut rows: Vec<(String, String, String, String, String, Vec<&'static str>)> = Vec::new();
        for m in explore::markers_of(loc) {
            let view = explore::marker_view(&d, &gs, loc, &m);
            let (kind, state) = (s(&m, "kind"), s(&view, "state"));
            if state == "absent" || kind == "point" {
                continue;
            }
            let services = if kind == "npc" { explore::npc_services(&d, &gs, &s(d.resolve(&gs.world_id, "characters", &s(&m, "character")), "_qid")) } else { Vec::new() };
            rows.push((s(&m, "id"), kind, state, s(&view, "label"), s(&view, "verb"), services));
        }
        let open = cx.settings.v["map_legend_open"].as_bool().unwrap_or(true);
        let rh = (ui.p.line_h(BODY).ceil() as i32 + 4).max(14);
        let title = format!("Here ({})", rows.len());
        let mut w = 14 + ui.p.text_w(&title, BODY).ceil() as i32 + 8;
        for (_, _, _, label, _, services) in &rows {
            w = w.max(4 + ICON + 4 + ui.p.text_w(label, BODY).ceil() as i32 + 6 + services.len() as i32 * (SERVICE + 1) + 4);
        }
        let w = w.min((LEGEND_MAX as f32 * ui.p.fonts.scale).round() as i32);
        let x = W - 4 - w;
        let head = Rect::new(x, HUD_H + 4, w, rh);
        ui.p.fill(head, PANE);
        let t = tip(&title, if open { "Click to fold the legend to this line." } else { "Click to open the legend: who and what is here." });
        ui.section("legend", head, &title, "", open, RAct::Legend, t);
        if !open || rows.is_empty() {
            return;
        }
        let top = head.y + head.h + 1;
        let content = rows.len() as i32 * rh;
        let pane = Rect::new(x, top, w, content.min(H - 4 - top));
        ui.p.fill(pane, PANE);
        ui.p.panel(pane, rgba(0, 0, 0, 0), BORDER, 3.0);
        let off = ui.scroll_begin("legend.rows", pane, content);
        let px = ui.icon_px(Rect::new(0, 0, ICON + 4, ICON + 4));
        let spx = ui.icon_px(Rect::new(0, 0, SERVICE + 4, SERVICE + 4));
        for (i, (id, kind, state, label, verb, services)) in rows.into_iter().enumerate() {
            let r = Rect::new(x, top - off + i as i32 * rh, w, rh);
            let hover = !modal && ui.hover(r);
            if hover {
                ui.p.panel(r, PANEL_HI, rgb(110, 140, 196), 3.0);
            }
            let ir = Rect::new(r.x + 4, r.y + (rh - ICON) / 2, ICON, ICON);
            match cx.art.image(&emberwake_content::art::marker(marker_icon(&kind, &state)), px, px) {
                Some(img) => ui.p.image(img, ir),
                None => ui.p.fill(Rect::new(ir.x + 2, ir.y + 2, ICON - 4, ICON - 4), Self::marker_style(&kind, &state).0),
            }
            let sw = services.len() as i32 * (SERVICE + 1);
            let tx = ir.x + ICON + 4;
            ui.p.text_fit(tx as f32, r.y as f32 + (rh as f32 - ui.p.line_h(BODY)) / 2.0, &label, BODY, (r.x + r.w - 4 - sw - tx) as f32, if state == "spent" { DIM } else { TEXT });
            let mut sx = r.x + r.w - 4 - sw;
            for sv in &services {
                let sr = Rect::new(sx, r.y + (rh - SERVICE) / 2, SERVICE, SERVICE);
                match cx.art.image(&service_icon(sv), spx, spx) {
                    Some(img) => ui.p.image(img, sr),
                    None => service_glyph(ui, sv, sr),
                }
                sx += SERVICE + 1;
            }
            let mut t = tip(&label, &verb);
            if !services.is_empty() {
                t.push((services_text(&services), INFO));
            }
            ui.push(format!("here.{id}"), "row", label.clone(), r, true, false, (!modal).then(|| RAct::Marker(id.clone())), t);
        }
        ui.scroll_end();
    }

    /// The event log ticker (MAPLOG1-U, spec PLAYUI1 2.1 A): the latest event on one line at the map's bottom left; L
    /// or a click opens the last `LOG_OPEN` events (newest at the bottom), and again shows only the latest.
    fn draw_ticker(&mut self, ui: &mut Ui<RAct>, cx: &Cx, modal: bool) {
        let Some(latest) = cx.log.last().cloned() else { return };
        let lh = ui.p.line_h(SMALL).ceil() as i32 + 1;
        let max_w = ((TICKER_MAX as f32 * ui.p.fonts.scale).round() as i32).min(W - 8);
        let inner = max_w - 15 - 4;
        let lines: Vec<String> = if self.log_open {
            let room = ((H - HUD_H - 16) / lh).max(1) as usize;
            let mut out = Vec::new();
            for l in cx.log.iter().rev().take(LOG_OPEN) {
                let mut wrapped = ui.p.wrap(l, SMALL, inner as f32);
                wrapped.reverse();
                out.extend(wrapped);
            }
            out.truncate(room);
            out.reverse();
            out
        } else {
            vec![latest.clone()]
        };
        let text_w = lines.iter().map(|l| ui.p.text_w(l, SMALL).ceil() as i32).max().unwrap_or(0);
        let w = (15 + text_w + 4).min(max_w);
        let h = lines.len() as i32 * lh + 5;
        let r = Rect::new(4, H - 4 - h, w, h);
        let hover = !modal && ui.hover(r);
        ui.p.fill(r, PANE);
        ui.p.panel(r, rgba(0, 0, 0, 0), if hover { GOLD } else { BORDER }, 3.0);
        let last_y = r.y + 3 + (lines.len() as i32 - 1) * lh;
        draw_glyph(&mut ui.p, if self.log_open { Glyph::ChevronDown } else { Glyph::ChevronRight }, Rect::new(r.x + 4, last_y + (lh - 8) / 2, 8, 8), if hover { GOLD } else { DIM });
        for (i, l) in lines.iter().enumerate() {
            let ly = (r.y + 3 + i as i32 * lh) as f32;
            ui.p.text_fit((r.x + 15) as f32, ly, l, SMALL, inner as f32, if i + 1 == lines.len() { TEXT } else { DIM });
        }
        let mut t = tip("Event log (L)", if self.log_open { "Click or press L to show only the latest event." } else { "Click or press L for the last 12 events." });
        if !self.log_open && ui.p.text_w(&latest, SMALL) > inner as f32 {
            t.push((latest.clone(), TEXT));
        }
        ui.push("log", "button", latest, r, !modal, self.log_open, Some(RAct::Log), t);
    }
}

/// The legend pane's and the ticker's fill.
const PANE: Rgba = rgba(12, 18, 28, 235);
/// The legend's widest at text size Normal (spec PLAYUI1 2.1 A), and the ticker's.
const LEGEND_MAX: i32 = 170;
const TICKER_MAX: i32 = 360;
/// The events the opened ticker shows (MAPLOG1-U).
pub const LOG_OPEN: usize = 12;
/// A legend row's marker icon and a service icon (logical px).
const ICON: i32 = 12;
const SERVICE: i32 = 10;

/// The marker's icon in `map/markers/` (`emberwake_content::art::marker`; the MARKERS1 idea may move it to `ui/markers/`).
fn marker_icon(kind: &str, state: &str) -> &'static str {
    match (kind, state) {
        ("exit", "locked") => "exit_locked",
        ("exit", _) => "exit",
        ("chest", "spent") => "chest_open",
        ("chest", _) => "chest",
        ("lore", _) => "lore",
        ("encounter", _) => "encounter",
        ("roamer", _) => "roamer",
        ("npc", _) => "npc",
        ("point", _) => "poi",
        _ => "objective",
    }
}

/// A service's icon (spec PLAYUI1 section 6): `turn_in` and `offer` are the quest badges `ready` and `quest`.
fn service_icon(service: &str) -> String {
    match service {
        "turn_in" => emberwake_content::art::badge("ready"),
        "offer" => emberwake_content::art::badge("quest"),
        _ => format!("ui/services/{service}.png"),
    }
}

/// "Quest to turn in, Shop" for a hover card.
fn services_text(services: &[&str]) -> String {
    services
        .iter()
        .map(|sv| match *sv {
            "turn_in" => "Quest to turn in",
            "offer" => "New quest",
            "shop" => "Shop",
            "recruit" => "Recruits",
            _ => "",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The drawn stand-in of a service icon: today's `?` and `!` badges, a coin for a shop, a plus for recruits.
fn service_glyph(ui: &mut Ui<RAct>, service: &str, r: Rect) {
    let (cx_, cy_, sz) = (r.x as f32 + r.w as f32 / 2.0, r.y as f32 + r.h as f32 / 2.0, r.w as f32);
    match service {
        "turn_in" | "offer" => {
            let (text, col) = if service == "turn_in" { ("?", GOOD) } else { ("!", GOLD) };
            let size = SMALL.max(r.h as f32 * 0.9 / 1.28);
            ui.p.text_center(cx_, cy_ - ui.p.line_h(size) / 2.0, text, size, col);
        }
        "shop" => {
            ui.p.circle(cx_, cy_, sz * 0.45, rgb(150, 110, 30));
            ui.p.circle(cx_, cy_, sz * 0.33, GOLD);
        }
        _ => {
            ui.p.line(cx_ - sz * 0.4, cy_, cx_ + sz * 0.4, cy_, GOOD, 0.0);
            ui.p.line(cx_, cy_ - sz * 0.4, cx_, cy_ + sz * 0.4, GOOD, 0.0);
        }
    }
}

/// A reply row of the dialogue box: its widget id, the number shown (1-9 choose it), the text the agent link reads,
/// the wrapped lines, locked, its action and its height.
struct Reply {
    id: String,
    num: String,
    label: String,
    lines: Vec<String>,
    locked: bool,
    act: RAct,
    h: i32,
}

/// The dialogue box measured for one frame (spec PLAYUI1 3.1): the box, its left column's width, the speaker's name
/// and faction band lines, the NPC's line in its (scrolling) view, and the replies in theirs.
struct TalkBox {
    r: Rect,
    left: i32,
    name: Vec<String>,
    band: Vec<String>,
    text: Vec<String>,
    text_view: Rect,
    replies: Vec<Reply>,
    num_w: i32,
    rows_view: Rect,
    rows_h: i32,
}

/// At most `n` lines of `lines`, the rest joined into the last one (drawn shortened, with the full text in the audit).
fn capped(mut lines: Vec<String>, n: usize) -> Vec<String> {
    if lines.len() > n {
        let rest = lines.split_off(n - 1).join(" ");
        lines.push(rest);
    }
    lines
}

/// Measures the dialogue box for `view` (a `DialogueRunner::view`) at the current text size: `TALK_W` wide (times the
/// text size, at most the canvas less 40), centred, `TALK_GAP` above the bottom; its height from the content, at least
/// `TALK_MIN_H`, at most `TALK_MAX_PCT` % of the canvas (the replies, then the line, scroll when they do not fit).
fn talk_layout(p: &mut Painter, view: &Dict) -> TalkBox {
    let sc = p.fonts.scale;
    let bw = (W - 40).min((TALK_W as f32 * sc).round() as i32);
    let left = TALK_LEFT.max((TALK_LEFT as f32 * sc).round() as i32);
    let (lb, ls) = (p.line_h(BODY).ceil() as i32, p.line_h(SMALL).ceil() as i32);
    let name = capped(p.wrap(&s(view, "speaker_name"), BODY, (left - 6) as f32), 2);
    let band_text = if s(view, "faction").is_empty() { String::new() } else { format!("{}: {}", s(view, "faction_name"), s(view, "band")) };
    let band = if band_text.is_empty() { Vec::new() } else { capped(p.wrap(&band_text, SMALL, (left - 6) as f32), 2) };
    let left_h = TALK_PAD + PORTRAIT + 3 + name.len() as i32 * lb + if band.is_empty() { 0 } else { 1 + band.len() as i32 * ls } + TALK_PAD;
    // the right part: the line on top, a 1 px divider, the replies below (3 px kept for a scroll bar)
    let rw = bw - left - 2 - TALK_PAD;
    let text = p.wrap(&s(view, "text"), BODY, (rw - 5) as f32);
    let text_h = text.len() as i32 * lb;
    let mut text_view_h = text_h.min(TALK_LINES * lb).max(lb);
    let num_w = p.text_w("8", BODY).ceil() as i32 + 6;
    let line_w = (rw - 4 - 5 - num_w - 4) as f32;
    let mut replies = Vec::new();
    let choices = json::aget(view, "choices");
    for (i, c) in choices.iter().enumerate() {
        let c = json::dict(c);
        let locked = json::b(c, "locked", false);
        let mut shown = s(c, "text");
        if locked {
            shown = format!("{shown}  ({})", s(c, "reason"));
        }
        let lines = p.wrap(&shown, BODY, line_w);
        let h = REPLY_H.max(lines.len() as i32 * lb + 5);
        let num = if i < 9 { (i + 1).to_string() } else { String::new() };
        replies.push(Reply { id: format!("choice.{i}"), label: format!("{}  {shown}", i + 1), num, lines, locked, act: RAct::Choose(i as i64), h });
    }
    // Continue is a row when the node goes on (or ends with no reply)
    if json::b(view, "next", false) || (json::b(view, "end", false) && choices.is_empty()) {
        replies.push(Reply { id: "dialogue.continue".into(), num: String::new(), label: "Continue".into(), lines: vec!["Continue".into()], locked: false, act: RAct::Advance, h: REPLY_H.max(lb + 5) });
    }
    let rows_h: i32 = replies.iter().map(|r| r.h + 2).sum::<i32>() - if replies.is_empty() { 0 } else { 2 };
    let mut rows_view_h: i32 = replies.iter().take(TALK_ROWS).map(|r| r.h + 2).sum::<i32>() - if replies.is_empty() { 0 } else { 2 };
    let max_h = H * TALK_MAX_PCT / 100;
    let content = |text_view_h: i32, rows_view_h: i32| TALK_PAD + text_view_h + 9 + rows_view_h + TALK_PAD;
    let mut over = content(text_view_h, rows_view_h).max(left_h) - max_h;
    if over > 0 {
        let least = replies.first().map_or(0, |r| r.h);
        let cut = over.min(rows_view_h - least).max(0);
        rows_view_h -= cut;
        over -= cut;
        text_view_h -= over.min(text_view_h - lb).max(0);
    }
    let bh = content(text_view_h, rows_view_h).max(left_h).max(TALK_MIN_H).min(max_h);
    let r = Rect::new((W - bw) / 2, H - TALK_GAP - bh, bw, bh);
    let rx = r.x + left + 2;
    let text_view = Rect::new(rx, r.y + TALK_PAD, rw, text_view_h);
    let rows_view = Rect::new(rx, text_view.y + text_view.h + 9, rw, rows_view_h);
    TalkBox { r, left, name, band, text, text_view, replies, num_w, rows_view, rows_h }
}

/// Text centred on `cx`, shortened to `max_w` (the audit keeps the full text).
fn text_center_fit(p: &mut Painter, cx: f32, y: f32, text: &str, size: f32, max_w: f32, col: Rgba) {
    let shown = p.fit(text, size, max_w);
    p.text_center(cx, y, &shown, size, col);
    if shown != text {
        p.mark_cut(text);
    }
}

/// Draws the dialogue box `b` (spec PLAYUI1 3.1): the portrait at the top of the left column with the speaker's name
/// (GOLD) and faction band (DIM) under it; at the right the NPC's line, a divider and the numbered replies (locked
/// ones dim with their reason; all off while `busy`, a turn-in picker over it). No key hint line: the numbers show
/// 1-9, Esc is on the Help button's card.
fn draw_talk(ui: &mut Ui<RAct>, cx: &mut Cx, view: &Dict, b: &TalkBox, busy: bool) {
    ui.p.panel(b.r, rgba(14, 20, 34, 250), FRAME_LINE, 4.0);
    ui.push("dialogue", "panel", s(view, "speaker_name"), b.r, true, false, None, Vec::new());
    let pr = Rect::new(b.r.x + (b.left - PORTRAIT) / 2, b.r.y + TALK_PAD, PORTRAIT, PORTRAIT);
    ui.p.panel(pr, rgb(22, 30, 48), BORDER, 2.0);
    let speaker = cx.data.resolve(&cx.gs().world_id, "characters", &s(view, "speaker"));
    let (_, _, portraits) = crate::screens::character_art(&cx.data, speaker);
    let px = ui.icon_px(Rect::new(0, 0, PORTRAIT + 4, PORTRAIT + 4));
    match cx.art.first(&portraits, px, px) {
        Some(img) => ui.p.image(img, pr),
        None => {
            let initial = s(view, "speaker_name").chars().next().unwrap_or('?').to_string();
            ui.p.text_center(pr.x as f32 + PORTRAIT as f32 / 2.0, pr.y as f32 + (PORTRAIT as f32 - ui.p.line_h(20.0)) / 2.0, &initial, 20.0, GOLD);
        }
    }
    let mid = b.r.x as f32 + b.left as f32 / 2.0;
    let max_w = (b.left - 6) as f32;
    let (lb, ls) = (ui.p.line_h(BODY).ceil(), ui.p.line_h(SMALL).ceil());
    let mut y = (pr.y + PORTRAIT + 3) as f32;
    for l in &b.name {
        text_center_fit(&mut ui.p, mid, y, l, BODY, max_w, GOLD);
        y += lb;
    }
    y += 1.0;
    for l in &b.band {
        text_center_fit(&mut ui.p, mid, y, l, SMALL, max_w, DIM);
        y += ls;
    }
    // the NPC's line (scrolls past TALK_LINES lines)
    let tv = b.text_view;
    let off = ui.scroll_begin("dialogue.text", tv, b.text.len() as i32 * lb as i32);
    for (i, l) in b.text.iter().enumerate() {
        ui.p.text(tv.x as f32, (tv.y - off) as f32 + i as f32 * lb, l, BODY, TEXT);
    }
    ui.scroll_end();
    let dy = (tv.y + tv.h + 4) as f32 + 0.5;
    ui.p.line(tv.x as f32, dy, (tv.x + tv.w) as f32, dy, BORDER, 0.0);
    // the replies (scroll past TALK_ROWS rows)
    let rv = b.rows_view;
    let off = ui.scroll_begin("dialogue.choices", rv, b.rows_h);
    let mut ry = rv.y - off;
    for rep in &b.replies {
        let rr = Rect::new(rv.x, ry, rv.w - 4, rep.h);
        let on = !rep.locked && !busy;
        let hover = on && ui.hover(rr);
        ui.p.panel(rr, if hover { PANEL_HI } else { CARD }, if hover { GOLD } else if rep.locked { BORDER } else { rgb(100, 130, 186) }, 2.0);
        let ty = rr.y as f32 + (rep.h as f32 - rep.lines.len() as f32 * lb) / 2.0;
        if !rep.num.is_empty() {
            ui.p.text(rr.x as f32 + 4.0, ty, &rep.num, BODY, if on { GOLD } else { DIM });
        }
        for (i, l) in rep.lines.iter().enumerate() {
            ui.p.text_fit((rr.x + 4 + b.num_w) as f32, ty + i as f32 * lb, l, BODY, (rr.w - 8 - b.num_w) as f32, if rep.locked { DIM } else { TEXT });
        }
        ui.push(rep.id.clone(), "choice", rep.label.clone(), rr, on, false, Some(rep.act.clone()), Vec::new());
        ry += rep.h + 2;
    }
    ui.scroll_end();
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

/// The region map's controls (the Help button's hover card and popup).
pub const REGION_HELP: [&str; 7] = [
    "Arrows or WASD: move",
    "E or Enter: use what the party faces",
    "Click: walk there, or on a marker to use it",
    "1-9: pick a reply in a conversation",
    "P: party   J: journal   I: stash",
    "L: the last events, or only the latest",
    "Esc: back, or the game menu",
];

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

/// PLAYUI1-M (spec PLAYUI1 2.1-2.3, 3.2): the full-width region map, its legend, the event log ticker, names on hover
/// and the centred camera, drawn headless over a debug game of the first playable world (the art layers are off, so
/// tiles are flat fills and icons their drawn stand-ins).
#[cfg(test)]
mod map_tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx, rect};
    use aegis_ui::painter::TextRec;
    use aegis_ui::screen::{Settings, TEXT_SIZES};

    /// One frame with the mouse at `mouse` (text size Normal); the texts drawn.
    fn frame_at(map: &mut RegionMap, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32)) -> Vec<TextRec> {
        g.fonts.scale = 1.0;
        g.fonts.audit = Some(Vec::new());
        map.draw(cx, g, mouse, true);
        g.fonts.audit.take().unwrap_or_default()
    }

    fn widget(map: &RegionMap, id: &str) -> Option<Value> {
        map.widgets().into_iter().find(|w| w["id"] == id)
    }

    fn ids_with(map: &RegionMap, prefix: &str) -> Vec<String> {
        map.widgets().iter().map(|w| s(json::dict(w), "id")).filter(|id| id.starts_with(prefix)).collect()
    }

    fn area() -> Rect {
        Rect::new(0, HUD_H, W, H - HUD_H)
    }

    /// The markers the legend lists, in order: present (or spent or locked) and not a bare point.
    fn listed(cx: &Cx) -> Vec<Dict> {
        let gs = cx.gs();
        let loc = explore::location(&cx.data, gs);
        explore::markers_of(loc)
            .into_iter()
            .filter(|m| s(&explore::marker_view(&cx.data, gs, loc, m), "state") != "absent" && s(m, "kind") != "point")
            .collect()
    }

    /// A text drawn as `marker`'s name: `label` just above the marker's tile (shifted sideways at most to stay inside).
    fn named_above(recs: &[TextRec], label: &str, marker: (i64, i64, i64, i64)) -> bool {
        let (mx, my, mw, _) = marker;
        recs.iter().any(|r| r.text == label && r.y + r.h <= my as f32 + 2.0 && r.y >= my as f32 - 24.0 && (r.x + r.w / 2.0 - (mx + mw / 2) as f32).abs() <= r.w / 2.0 + TILE as f32)
    }

    #[test]
    fn the_map_takes_the_full_width_under_the_bar() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let m = widget(&map, "map").expect("the map");
        assert_eq!(rect(&m), (0, HUD_H as i64, W as i64, (H - HUD_H) as i64));
        // a click on an open tile next to the party walks there (the click maps through the centred camera)
        let gs = cx.gs().clone();
        let t = explore::template_of(&cx.data, explore::location(&cx.data, &gs));
        let open = explore::DIRS.into_iter().map(|d| (gs.position.0 + d.0, gs.position.1 + d.1)).find(|p| explore::is_open(t, p.0, p.1));
        if let Some(p) = open {
            let (ox, oy, ts) = map.geo;
            let at = (ox + p.0 as i32 * ts + ts / 2, oy + p.1 as i32 * ts + ts / 2);
            frame_at(&mut map, &mut cx, &mut g, at);
            assert!(map.click(&mut cx, at), "the map takes the click");
            assert_eq!(map.state(&cx)["walking"], json!(true), "a click next to the party walks there");
        }
    }

    /// 3.2 A: no clamp; with the party on each corner tile of every map of the world, the party is drawn at the centre
    /// of the map area (within a tile), and past the map's edge the wall is drawn (render only).
    #[test]
    fn the_camera_centres_on_the_party_and_walls_fill_past_the_edge() {
        let (mut cx, mut g) = (game_cx(), gfx());
        cx.settings.v["map_legend_open"] = json!(false);
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        let mut maps = 0;
        for loc in d.merged_entries(&world, "locations") {
            let t = explore::template_of(&d, loc);
            if t.is_empty() {
                continue;
            }
            maps += 1;
            let (w, h) = explore::size_of(t);
            for (x, y) in [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)] {
                cx.gs_mut().location_id = s(loc, "id");
                let mut map = RegionMap::new(&mut cx);
                // after `resume` (which puts the party back on a safe tile): the corner itself, wall or not
                cx.gs_mut().position = (x, y);
                cx.log.clear();
                frame(&mut map, &mut cx, &mut g, 1.0);
                let (px, py, pw, ph) = rect(&widget(&map, "party").expect("the party"));
                let a = area();
                let (dx, dy) = ((px + pw / 2) as i32 - (a.x + a.w / 2), (py + ph / 2) as i32 - (a.y + a.h / 2));
                assert!(dx.abs() <= TILE && dy.abs() <= TILE, "{} at ({x}, {y}): the party is {dx}, {dy} off the centre", s(loc, "id"));
                // the canvas pixel just outside the map, away from the corner the party stands on
                let ox = if x == 0 { a.x + 2 } else { a.x + a.w - 3 };
                let oy = if y == 0 { a.y + 20 } else { a.y + a.h - 3 };
                let i = ((oy * g.canvas.w + ox) * 4) as usize;
                assert_eq!(&g.canvas.px[i..i + 3], &[40, 60, 50], "{} at ({x}, {y}): the wall is drawn past the edge at ({ox}, {oy})", s(loc, "id"));
            }
        }
        assert!(maps > 0, "the world has a location with a map");
    }

    #[test]
    fn the_legend_lists_the_markers_and_folds_to_its_header() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let head = widget(&map, "legend").expect("the legend header");
        assert_eq!(head["kind"], json!("section"));
        let (hx, hy, hw, _) = rect(&head);
        assert_eq!((hx + hw, hy), ((W - 4) as i64, (HUD_H + 4) as i64), "top right under the bar");
        assert!(hw <= LEGEND_MAX as i64, "at most {LEGEND_MAX} wide at Normal: {hw}");
        let markers = listed(&cx);
        assert!(head["text"].as_str().is_some_and(|t| t == format!("Here ({})", markers.len())), "{head}");
        let want: Vec<String> = markers.iter().map(|m| format!("here.{}", s(m, "id"))).collect();
        let rows = ids_with(&map, "here.");
        assert_eq!(rows, want);
        // a person's row names their services on its hover card
        let gs = cx.gs().clone();
        for m in markers.iter().filter(|m| s(m, "kind") == "npc") {
            let who = cx.data.resolve(&gs.world_id, "characters", &s(m, "character"));
            let services = explore::npc_services(&cx.data, &gs, &s(who, "_qid"));
            let card = widget(&map, &format!("here.{}", s(m, "id"))).map(|w| w["tip"].to_string()).unwrap_or_default();
            assert_eq!(card.contains(&services_text(&services)) && !services.is_empty(), !services.is_empty(), "{card}");
        }
        // a click folds it to the header, kept in the settings file; another opens it again
        map.click_id(&mut cx, "legend").expect("fold");
        assert_eq!(map.state(&cx)["legend_open"], json!(false));
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert!(widget(&map, "legend").is_some() && ids_with(&map, "here.").is_empty(), "folded to its header");
        let saved = Settings::load(cx.shot_dir.join("settings.json"), crate::screens::default_settings());
        assert_eq!(saved.v["map_legend_open"], json!(false), "the fold is saved");
        map.click_id(&mut cx, "legend").expect("open");
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(ids_with(&map, "here."), want);
    }

    #[test]
    fn the_ticker_shows_the_latest_event_and_l_shows_the_last_12() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        cx.log = (0..20).map(|i| format!("Event {i}.")).collect();
        let shown = |map: &RegionMap, recs: &[TextRec]| -> Vec<String> {
            let (x, y, w, h) = rect(&widget(map, "log").expect("the ticker"));
            recs.iter().filter(|r| r.x >= x as f32 && r.x < (x + w) as f32 && r.y >= y as f32 && r.y < (y + h) as f32).map(|r| r.text.clone()).collect()
        };
        let recs = frame(&mut map, &mut cx, &mut g, 1.0);
        let (x, y, _, h) = rect(&widget(&map, "log").expect("the ticker"));
        assert!(x < 10 && y + h >= (H - 10) as i64, "bottom left: {x}, {y}");
        assert_eq!(shown(&map, &recs), ["Event 19."]);
        map.key(&mut cx, Key::Char('l'));
        let recs = frame(&mut map, &mut cx, &mut g, 1.0);
        let want: Vec<String> = (20 - LOG_OPEN..20).map(|i| format!("Event {i}.")).collect();
        assert_eq!(shown(&map, &recs), want, "the last {LOG_OPEN}, newest at the bottom");
        map.click_id(&mut cx, "log").expect("the ticker");
        let recs = frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(shown(&map, &recs), ["Event 19."]);
        // hidden while talking (and the legend with it)
        let world = cx.gs().world_id.clone();
        let people: Vec<String> = cx.data.merged_entries(&world, "characters").iter().map(|c| s(c, "id")).collect();
        for p in people {
            map.talk(&mut cx, &p);
            if map.talking() {
                break;
            }
        }
        assert!(map.talking(), "someone in the world talks");
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert!(widget(&map, "log").is_none() && widget(&map, "legend").is_none(), "no ticker or legend in a conversation");
    }

    /// 2.3 A: names on hover by default (a marker or its legend row under the pointer, or the marker ahead), or always.
    #[test]
    fn names_show_on_hover_by_default_or_always() {
        let (mut cx, mut g) = (game_cx(), gfx());
        assert_eq!(cx.settings.v["map_names"], json!("hover"), "the default");
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let gs = cx.gs().clone();
        let ahead = (gs.position.0 + map.facing.0, gs.position.1 + map.facing.1);
        let t = explore::template_of(&cx.data, explore::location(&cx.data, &gs)).clone();
        let a = area();
        // (id, kind, label, rect, ahead) of the markers wholly inside the map area
        let markers: Vec<(String, String, String, (i64, i64, i64, i64), bool)> = map
            .widgets()
            .iter()
            .filter(|w| w["kind"] == "marker")
            .filter_map(|w| {
                let id = s(json::dict(w), "id").trim_start_matches("marker.").to_string();
                let m = explore::markers_of(explore::location(&cx.data, &gs)).into_iter().find(|m| s(m, "id") == id)?;
                let r = rect(w);
                let inside = r.0 >= 0 && r.1 >= a.y as i64 + 24 && r.0 + r.2 <= W as i64 && r.1 + r.3 <= H as i64;
                inside.then(|| (id, s(&m, "kind"), s(json::dict(w), "text"), r, explore::marker_pos(&t, &m, gs.explore_int("loc_steps")) == ahead))
            })
            .collect();
        assert!(!markers.is_empty(), "the start shows a marker");
        let recs = frame(&mut map, &mut cx, &mut g, 1.0);
        for (id, _, label, r, faced) in &markers {
            assert_eq!(named_above(&recs, label, *r), *faced, "{id}: named only when the party faces it");
        }
        // the pointer on a marker names it; on its legend row too
        let (id, _, label, r, _) = &markers[0];
        let centre = ((r.0 + r.2 / 2) as i32, (r.1 + r.3 / 2) as i32);
        let recs = frame_at(&mut map, &mut cx, &mut g, centre);
        assert!(named_above(&recs, label, *r), "{id}: hovered, named");
        if let Some(row) = widget(&map, &format!("here.{id}")) {
            let (x, y, w, h) = rect(&row);
            let on_row = ((x + w / 2) as i32, (y + h / 2) as i32);
            frame_at(&mut map, &mut cx, &mut g, on_row);
            let recs = frame_at(&mut map, &mut cx, &mut g, on_row);
            assert!(named_above(&recs, label, *r), "{id}: its legend row hovered, named");
        }
        // always: every person and exit
        cx.settings.v["map_names"] = json!("always");
        let recs = frame(&mut map, &mut cx, &mut g, 1.0);
        for (id, kind, label, r, _) in &markers {
            assert_eq!(named_above(&recs, label, *r), kind == "npc" || kind == "exit", "{id} ({kind}) with names always");
        }
    }

    /// TEXTFIT1: the map's own texts (legend open and folded, ticker short and opened, names on hover and always) fit
    /// at every text size: nothing cut, past the canvas, overlapping or out of its control.
    #[test]
    fn the_region_map_reads_clean_at_every_text_size() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        let place = cx.location_name(&cx.gs().world_id.clone(), &cx.gs().location_id.clone());
        for line in ["The party rests (+40 HP).".to_string(), format!("Entered {place}."), "The party makes camp (-3 supplies, +40 HP).".into(), "Found: Healing Draught x2, 15 gold.".into()] {
            cx.log(line);
        }
        for names in ["hover", "always"] {
            cx.settings.v["map_names"] = json!(names);
            for legend in [true, false] {
                cx.settings.v["map_legend_open"] = json!(legend);
                for log_open in [false, true] {
                    map.log_open = log_open;
                    for (size, scale) in TEXT_SIZES {
                        let recs = frame(&mut map, &mut cx, &mut g, scale as f32);
                        let boxes: Vec<(String, f32, f32, f32, f32)> = map
                            .widgets()
                            .iter()
                            .map(|w| {
                                let (x, y, ww, hh) = rect(w);
                                (s(json::dict(w), "id"), x as f32, y as f32, ww as f32, hh as f32)
                            })
                            .collect();
                        let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
                        let counts = &v["counts"];
                        let label = format!("names {names}, legend {legend}, log {log_open} at {size}: {v}");
                        assert_eq!((counts["cut"].clone(), counts["edge"].clone(), counts["spill"].clone()), (json!(0), json!(0), json!(0)), "{label}");
                        // names may meet each other where people stand side by side (content); nothing else may overlap
                        if names == "hover" {
                            assert_eq!(counts["overlap"], json!(0), "{label}");
                        }
                    }
                }
            }
        }
    }
}

/// PLAYUI1-D (spec PLAYUI1 3.1-3.3): the dialogue box, the camera on both speakers, the shop over the dimmed map, drawn
/// headless over a debug game of the first playable world (art layers off: portraits are initials, tiles flat fills).
#[cfg(test)]
mod talk_tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx, rect};
    use aegis_ui::painter::TextRec;
    use aegis_ui::screen::TEXT_SIZES;

    fn widget(map: &RegionMap, id: &str) -> Option<Value> {
        map.widgets().into_iter().find(|w| w["id"] == id)
    }

    fn replies(widgets: &[Value]) -> Vec<Value> {
        widgets.iter().filter(|w| w["kind"] == "choice").cloned().collect()
    }

    /// TEXTFIT1 counts (cut, edge, spill, overlap) of a frame, and the report.
    fn audit(widgets: &[Value], recs: &[TextRec]) -> (Vec<i64>, Value) {
        let boxes: Vec<(String, f32, f32, f32, f32)> = widgets
            .iter()
            .map(|w| {
                let (x, y, ww, hh) = rect(w);
                (s(json::dict(w), "id"), x as f32, y as f32, ww as f32, hh as f32)
            })
            .collect();
        let v = aegis_ui::text_audit::issues(recs, W as f32, H as f32, &boxes);
        let c = &v["counts"];
        (["cut", "edge", "spill", "overlap"].iter().map(|k| c[*k].as_i64().unwrap_or(-1)).collect(), v)
    }

    /// The box's width at text size `scale` (3.1: 400 times the text size, at most the canvas less 40).
    fn box_w(scale: f64) -> i64 {
        ((W - 40) as i64).min((TALK_W as f64 * scale).round() as i64)
    }

    /// Every conversation opened the way a player opens it: for each person on a map of the world, from each open tile
    /// next to them, facing them and pressing E. `each` gets the map (talking), the party's tile and the person's, and
    /// says whether to go on; the game state is put back after each. The number of conversations opened.
    fn each_talk(cx: &mut Cx, mut each: impl FnMut(&mut RegionMap, &mut Cx, Pos, Pos) -> bool) -> usize {
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        let start = cx.gs().clone();
        let mut opened = 0;
        for loc in d.merged_entries(&world, "locations") {
            let t = explore::template_of(&d, loc).clone();
            if t.is_empty() {
                continue;
            }
            cx.gs_mut().location_id = s(loc, "id");
            let _ = RegionMap::new(cx);
            let here = cx.gs().clone();
            let steps = here.explore_int("loc_steps");
            for m in explore::markers_of(loc) {
                if s(&m, "kind") != "npc" || explore::presence(&d, &here, &s(loc, "_qid"), &m) == "absent" {
                    continue;
                }
                let at = explore::marker_pos(&t, &m, steps);
                for dir in explore::DIRS {
                    let p = (at.0 - dir.0, at.1 - dir.1);
                    if !explore::is_open(&t, p.0, p.1) {
                        continue;
                    }
                    *cx.gs_mut() = here.clone();
                    let mut map = RegionMap::new(cx);
                    cx.gs_mut().position = p;
                    map.interact(cx, dir);
                    if map.talking() {
                        opened += 1;
                        let go_on = each(&mut map, cx, p, at);
                        map.end_talk(cx);
                        if !go_on {
                            *cx.gs_mut() = start;
                            return opened;
                        }
                    }
                }
            }
        }
        *cx.gs_mut() = start;
        opened
    }

    /// 3.1: 400 wide (times the text size), centred, 6 above the bottom, 104 to 45 % of the canvas tall; the name under
    /// the portrait; the line above the replies; no key hint line; the legend and the log ticker stay hidden.
    #[test]
    fn the_dialogue_box_is_a_small_centred_bottom_box() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let n = each_talk(&mut cx, |map, cx, _, _| {
            let name = s(json::dict(&map.state(cx)["dialogue"]), "speaker_name");
            for (size, scale) in TEXT_SIZES {
                let recs = frame(map, cx, &mut g, scale as f32);
                let (x, y, w, h) = rect(&widget(map, "dialogue").expect("the dialogue box"));
                assert_eq!(w, box_w(scale), "{size}: width");
                assert!((x - (W as i64 - x - w)).abs() <= 1, "{size}: centred ({x}, {w})");
                assert_eq!(H as i64 - (y + h), TALK_GAP as i64, "{size}: {TALK_GAP} above the bottom");
                assert!(h >= TALK_MIN_H as i64 && h <= (H * TALK_MAX_PCT / 100) as i64, "{size}: height {h}");
                let left = (TALK_LEFT as f64).max((TALK_LEFT as f64 * scale).round()) as f32;
                let under = (y + (TALK_PAD + PORTRAIT) as i64) as f32;
                let named = recs.iter().any(|r| !r.text.is_empty() && name.starts_with(r.text.as_str()) && r.y >= under && r.x >= x as f32 && r.x + r.w <= x as f32 + left);
                assert!(named, "{size}: {name} under the portrait");
                let (_, ty, _, th) = rect(&widget(map, "dialogue.text").expect("the line"));
                let rows = replies(&map.widgets());
                assert!(!rows.is_empty(), "{size}: a reply or Continue");
                for r in &rows {
                    let (rx, ry, rw, rh) = rect(r);
                    assert!(ry >= ty + th && rx >= x && ry + rh <= y + h && rx + rw <= x + w, "{size}: {r} below the line, inside the box");
                }
                assert!(!recs.iter().any(|r| r.text.contains("1-9")), "{size}: no key hint line");
                assert!(widget(map, "legend").is_none() && widget(map, "log").is_none(), "{size}: no legend or ticker while talking");
            }
            false
        });
        assert!(n > 0, "someone on a map of the world talks");
    }

    /// 3.2 acceptance: for every person on the world's maps, a conversation opened from any open tile next to them keeps
    /// both the party's and their tile inside the map above the box, at all four text sizes; every box reads clean.
    #[test]
    fn both_speakers_stay_in_view_above_the_box() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let n = each_talk(&mut cx, |map, cx, party, npc| {
            for (size, scale) in TEXT_SIZES {
                let recs = frame(map, cx, &mut g, scale as f32);
                let (_, by, _, _) = rect(&widget(map, "dialogue").expect("the dialogue box"));
                let view = Rect::new(0, HUD_H, W, by as i32 - HUD_H);
                let (ox, oy, ts) = map.geo;
                for (who, at) in [("party", party), ("speaker", npc)] {
                    let r = Rect::new(ox + at.0 as i32 * ts, oy + at.1 as i32 * ts, ts, ts);
                    let inside = r.x >= view.x && r.y >= view.y && r.x + r.w <= view.x + view.w && r.y + r.h <= view.y + view.h;
                    assert!(inside, "{} at {party:?} talking to {npc:?}, {size}: the {who}'s tile {r:?} is not inside {view:?}", cx.gs().location_id);
                }
                // the box's texts (the map's own are `map_tests`'; a drawn badge stand-in may sit on the party's tile
                // when the party stands just above a quest giver)
                let in_box: Vec<TextRec> = recs.into_iter().filter(|r| r.y >= by as f32).collect();
                let (counts, v) = audit(&map.widgets(), &in_box);
                assert_eq!(counts, [0, 0, 0, 0], "{} at {party:?}, {size}: {v}", cx.gs().location_id);
            }
            true
        });
        eprintln!("{n} conversations checked");
        assert!(n > 0, "someone on a map of the world talks");
    }

    /// 3.1: the number keys still choose (the farewell reply ends the conversation).
    #[test]
    fn a_number_key_chooses_a_reply() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut checked = false;
        each_talk(&mut cx, |map, cx, _, _| {
            frame(map, cx, &mut g, 1.0);
            let choices = json::aget(json::dict(&map.state(cx)["dialogue"]), "choices").to_vec();
            let Some(i) = choices.iter().position(|c| s(json::dict(c), "service") == "farewell").filter(|i| *i < 9) else { return true };
            map.key(cx, Key::Char(char::from(b'1' + i as u8)));
            assert!(!map.talking(), "key {} (farewell) ends the conversation", i + 1);
            checked = true;
            false
        });
        assert!(checked, "a conversation with a farewell reply");
    }

    /// 3.1 text audit with a long line and many replies: the box stays at most 45 % of the canvas, the line and the
    /// replies scroll inside it, nothing is cut, past the canvas, spilled or overlapping at any text size.
    #[test]
    fn a_long_line_and_many_replies_scroll_inside_the_box() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let world = cx.gs().world_id.clone();
        let speaker = cx.data.merged_entries(&world, "characters").first().map(|c| s(c, "id")).unwrap_or_default();
        let line = "The ash has not stopped falling since the forge went cold, and every road out of the valley is watched by the Hollow riders. ".repeat(4);
        let mut choices: Vec<Value> = (0..12)
            .map(|i| json!({"text": format!("Reply {i}: tell me more about the riders, the forge and the road north before I decide."), "locked": false, "reason": "", "service": ""}))
            .collect();
        choices[2] = json!({"text": "[Might 8] Force the gate open.", "locked": true, "reason": "Needs Might 8 (best 5)", "service": ""});
        let view = json::dict(&json!({"speaker": speaker, "speaker_name": "Warden Ilsa Varn", "node": "long", "text": line.trim_end(), "choices": choices,
            "end": false, "next": false, "faction": "f", "faction_name": "Ember Wardens", "band": "neutral"}))
        .clone();
        for (size, scale) in TEXT_SIZES {
            g.fonts.scale = scale as f32;
            g.fonts.audit = Some(Vec::new());
            let mut ui: Ui<RAct> = Ui::new(&mut g, (-500, -500), true);
            let b = talk_layout(&mut ui.p, &view);
            draw_talk(&mut ui, &mut cx, &view, &b, false);
            let widgets: Vec<Value> = ui.widgets.iter().map(|w| w.info()).collect();
            drop(ui);
            let recs = g.fonts.audit.take().unwrap_or_default();
            assert!(b.r.h <= H * TALK_MAX_PCT / 100 && b.r.y + b.r.h == H - TALK_GAP, "{size}: {:?}", b.r);
            let max = |id: &str| widgets.iter().find(|w| w["id"] == id).and_then(|w| w["text"].as_str()?.parse::<i32>().ok()).unwrap_or(0);
            assert!(max("dialogue.text") > 0 && max("dialogue.choices") > 0, "{size}: the line and the replies scroll");
            let shown: Vec<Value> = replies(&widgets).into_iter().filter(|w| rect(w).2 > 0 && rect(w).3 > 0).collect();
            assert!(shown.first().is_some_and(|w| w["id"] == "choice.0") && shown.len() < 12, "{size}: the first replies show, the rest scroll");
            for w in &shown {
                let (x, y, ww, hh) = rect(w);
                let inside = x >= b.r.x as i64 && y >= b.r.y as i64 && x + ww <= (b.r.x + b.r.w) as i64 && y + hh <= (b.r.y + b.r.h) as i64;
                assert!(inside, "{size}: {w} inside the box");
            }
            assert!(shown.iter().any(|w| w["id"] == "choice.2" && w["enabled"] == json!(false)), "{size}: the locked reply shows, off");
            let (counts, v) = audit(&widgets, &recs);
            assert_eq!(counts, [0, 0, 0, 0], "{size}: {v}");
        }
    }

    /// 3.3: the shop hides the box and sits centred under the HUD bar over the dimmed map; Esc brings the box back.
    #[test]
    fn the_shop_hides_the_box_over_a_dimmed_map() {
        let (mut cx, mut g) = (game_cx(), gfx());
        // the map's brightness left of the box (the legend and the ticker are hidden while talking)
        let light = |g: &Gfx| -> u64 {
            let mut sum = 0;
            for y in (HUD_H + 2)..(H - 10) {
                for x in 0..100 {
                    let i = ((y * g.canvas.w + x) * 4) as usize;
                    sum += g.canvas.px[i..i + 3].iter().map(|v| *v as u64).sum::<u64>();
                }
            }
            sum
        };
        let mut checked = false;
        each_talk(&mut cx, |map, cx, _, _| {
            let choices = json::aget(json::dict(&map.state(cx)["dialogue"]), "choices").to_vec();
            let Some(k) = choices.iter().position(|c| s(json::dict(c), "service") == "shop") else { return true };
            frame(map, cx, &mut g, 1.0);
            let before = light(&g);
            map.click_id(cx, &format!("choice.{k}")).expect("the shop reply");
            frame(map, cx, &mut g, 1.0);
            assert!(widget(map, "dialogue").is_none() && replies(&map.widgets()).is_empty(), "the box hides while the shop is open");
            let (x, y, w, h) = rect(&widget(map, "shop").expect("the shop"));
            assert!((x - (W as i64 - x - w)).abs() <= 1 && (y - HUD_H as i64 - (H as i64 - y - h)).abs() <= 1, "centred under the HUD bar: {x}, {y}, {w}, {h}");
            assert!(light(&g) * 10 < before * 7, "the map is dimmed: {} vs {before}", light(&g));
            map.back(cx);
            frame(map, cx, &mut g, 1.0);
            assert!(map.talking() && widget(map, "dialogue").is_some() && widget(map, "shop").is_none(), "Esc closes the shop, the box comes back");
            checked = true;
            false
        });
        if !checked {
            eprintln!("no shop keeper on the world's maps: nothing to check");
        }
    }
}
