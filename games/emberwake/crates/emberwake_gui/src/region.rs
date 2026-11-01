//! The region map (scripts/ui/overworld/region_map_view.gd + dialogue_panel.gd + shop_panel.gd): walking a
//! location's map over the game state (ExploreRunner) under the HUD bar, the floating "Here" legend and the event log
//! ticker (PLAYUI1-M: the full-width map, a camera centred on the party with walls drawn past the edge), the dialogue
//! box at the bottom centre (DialogueRunner; PLAYUI1-D: the camera keeps both speakers above it), the shop's trade over
//! a dimmed map (`shop.rs`, PLAYUI1-P), the turn-in picker and lore pages. Fights start the battle screen; a world-map exit opens
//! the world map. Map-less locations show their exits as buttons. PLAYUI1-R (spec PLAYUI1 2.4): the party leader and the
//! people stand on their tiles as body sprites (`walk` while moving, else `idle`; side view, mirrored by the last
//! left or right step; a person in the conversation faces the party), with today's tokens as the fallback.
//! STORYUI1-U1: with nothing open the map hands over to the story stage while a sequence runs and plays queued
//! vignettes (`story::map_tick`); a `sequence` choice ends the talk and the stage takes over. STORYUI1-U2: an open
//! board shows its pane (`board.rs`) in the shop's place until Close, Esc or a right click closes it back to the
//! conversation; the keeper of a board with work wears the quest badge ("New work on the board"). CAMPAIGN1-U: a queued `{dialogue}` scene
//! opens as the conversation when nothing is open (T2); a reply whose `rest` would end the run asks first (W3); the
//! log lines name the purse (P1) and a retreat (R4).

use aegis_core::assets::{ClipPlayer, Facing, View};
use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use crate::hud::{hud_bar, hud_key, hud_open, purse_word, Hud, PurseWord, HUD_H};
use crate::runover::days as day_count;
use crate::music;
use crate::props::{Item, PropDraw, BOX_EDGE, BOX_FACE, BOX_FACE_H, BOX_FILL};
use crate::board::{BoardAct, BoardPane};
use crate::portraits::Faces;
use crate::shop::{ShopAct, ShopPane};
use crate::talk::{self, talk_layout, TalkAct, TalkBox};
use crate::screens::{show_errors, Cx, GameCx, Key, Nav, Screen};
use aegis_ui::ui::{draw_glyph, tip, Gfx, Glyph, Ui, Widget, BODY, BORDER, DIM, GOLD, GOOD, H, INFO, PANEL_HI, SMALL, TEXT, W};
use aegis_ui::widget_plumbing;
use aegis_core::json::{self, s, s_or, Dict};
use emberwake_world::dialogue::DialogueRunner;
use emberwake_world::explore::{self, Pos};
use emberwake_world::quest;
use serde_json::{json, Value};
use std::time::{Duration, Instant};

/// A map tile (logical px; 32 px at 1280 wide, MAPVIEW1).
const TILE: i32 = 16;
/// A character sprite on the map (spec PLAYUI1 2.4): 1.5 tiles tall (the 48 px frame at 24 logical px), standing
/// bottom-centred on its tile.
const SPRITE: i32 = TILE * 3 / 2;
/// The leader keeps walking this long (animation ms) after a step, so key steps and walked paths play `walk` without
/// a stop between tiles.
const WALK_LINGER_MS: f64 = 220.0;

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
    /// A click in the shop's trade (PLAYUI1-P, `shop.rs`).
    Shop(ShopAct),
    /// Leaves the shop for the conversation (the trade is empty or discarded).
    CloseShop,
    /// Closes the open board back to the conversation (STORYUI1 B11).
    CloseBoard,
    /// A click on the board pane (STORYUI1-U2, `board.rs`).
    Board(BoardAct),
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
    /// The open shop's trade (PLAYUI1-P).
    shop: ShopPane,
    /// The open board's pick and refusals (STORYUI1-U2).
    board: BoardPane,
    pending: Option<Dict>,
    /// CAMPAIGN1 W3: the choice waiting on the ask before its `rest` would end the run (a Yes picks it).
    rest_ask: Option<i64>,
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
    /// PLAYUI1-R: the sprites' animation clock (ms, at the animation speed setting) and when it last advanced.
    anim_ms: f64,
    anim_at: Instant,
    /// The leader's clips (`walk` / `idle`), the clock time of the last step that moved the party, and the facing
    /// (side view, left after a step to the left until a step to the right; DIR4-M adds up and down).
    leader: ClipPlayer,
    moved_at: Option<f64>,
    leader_face: Facing,
    /// The sprites drawn last frame (agent link `state.sprites`, tests).
    sprites: Vec<Value>,
    /// PORTRAITFX1-D: the talk box's animated portrait (its clock moves with the frame's wall time).
    faces: Faces,
}

/// Starts a world fight: the battle screen, or the Skip / Fled / Lost popup in a debug game.
pub fn begin_fight(cx: &mut Cx, fight: &Dict, runner: &str) {
    if let Err(e) = try_fight(cx, fight, runner) {
        show_errors(cx, vec![e]);
    }
}

/// `begin_fight` that hands the reason back when the battle cannot start (the story stage shows it itself).
pub fn try_fight(cx: &mut Cx, fight: &Dict, runner: &str) -> Result<(), String> {
    if cx.debug {
        cx.nav.push(Nav::Push(Box::new(crate::runner::DebugFight::new(fight, runner))));
        return Ok(());
    }
    let b = crate::battle::BattleScreen::world(cx, fight, runner)?;
    cx.nav.push(Nav::Push(Box::new(b)));
    Ok(())
}

/// Logs "Fight: <name>!" and starts world fight `fight` (`begin_fight`).
pub fn fight_now(cx: &mut Cx, fight: &Dict, runner: &str) {
    let name = s_or(cx.data.resolve(&cx.gs().world_id, "encounters", &s(fight, "encounter")), "name", "Enemies");
    cx.log(format!("Fight: {name}!"));
    begin_fight(cx, fight, runner);
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
            // CAMPAIGN1 P1: the purse's names (the Ashlands lines are today's); R2: a rest's days
            "camped" => {
                let days = json::i(e, "days", 0);
                let took = if days > 0 { format!(", {}", day_count(days)) } else { String::new() };
                let upkeep = purse_word(cx, PurseWord::Upkeep);
                cx.log(format!("The party makes camp (-{} {upkeep}, +{} HP{took}).", json::i(e, "spent", 0), json::i(e, "restored", 0)));
            }
            "defeat" => {
                let name = cx.location_name(&cx.gs().world_id.clone(), &s(e, "location"));
                let upkeep = purse_word(cx, PurseWord::Upkeep);
                // R4: a retreat falls back to the hub, its days and upkeep spent
                if s(e, "mode") == "retreat" {
                    let days = json::i(e, "days", 0);
                    let cost = if days > 0 { format!(" ({}, -{} {upkeep})", day_count(days), json::i(e, "spent", 0)) } else { String::new() };
                    cx.log(format!("Defeated. The party falls back to {name}{cost}."));
                    continue;
                }
                let how = match s(e, "mode").as_str() {
                    "rest_place" => "rests there for free".to_string(),
                    "forced_rest" => format!("rests ({} {upkeep})", json::i(e, "spent", 0)),
                    _ => "limps back to town".to_string(),
                };
                cx.log(format!("Defeated. The party wakes at {name} and {how}."));
            }
            "retreat_failed" => cx.log(s(e, "text")),
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
        crate::runover::take(cx, &res);
        Self {
            w: Vec::new(),
            mouse: (0, 0),
            facing: (0, 1),
            runner: None,
            shop: ShopPane::default(),
            board: BoardPane::default(),
            pending: None,
            rest_ask: None,
            picks: Vec::new(),
            recipient: 0,
            lore: None,
            walk: Vec::new(),
            walk_target: None,
            next_walk: Instant::now(),
            geo: (0, 0, 1),
            log_open: false,
            talk_node: String::new(),
            anim_ms: 0.0,
            anim_at: Instant::now(),
            leader: ClipPlayer::default(),
            moved_at: None,
            leader_face: Facing::default(),
            sprites: Vec::new(),
            faces: Faces::default(),
        }
    }

    fn talking(&self) -> bool {
        self.runner.as_ref().is_some_and(|r| !r.closed)
    }

    /// The open shop of the conversation ("" when none).
    fn shop_qid(&self) -> String {
        self.runner.as_ref().filter(|r| !r.closed).map(|r| r.shop_qid.clone()).unwrap_or_default()
    }

    /// The open board of the conversation ("" when none; STORYUI1 B3).
    fn board_qid(&self) -> String {
        self.runner.as_ref().filter(|r| !r.closed).map(|r| r.board_qid.clone()).unwrap_or_default()
    }

    fn modal(&self) -> bool {
        self.talking() || self.lore.is_some()
    }

    /// One step; handles fights, world-map exits and the log. False when the walk should stop.
    fn step(&mut self, cx: &mut Cx, dir: Pos) -> bool {
        self.turn(dir);
        let d = cx.data.clone();
        let from = cx.gs().position;
        let res = explore::step(&d, cx.gs_mut(), dir);
        if cx.gs().position != from {
            self.moved_at = Some(self.anim_ms);
        }
        if !json::b(&res, "ok", false) {
            for e in json::sget(&res, "errors") {
                cx.log(e);
            }
            return false;
        }
        let events = json::aget(&res, "events").to_vec();
        log_events(cx, &events);
        crate::runover::take(cx, &res);
        let mut go_on = true;
        for e in &events {
            let e = json::dict(e);
            match s(e, "kind").as_str() {
                // CAMPAIGN1 W3: a marker fight that could end the run asks first (No: the party steps back); a fight
                // that cannot be refused gets the notice
                "fight" => {
                    go_on = false;
                    let enc = s(e, "encounter");
                    match (s(e, "source") == "marker", crate::runover::fight_ask(cx, &enc)) {
                        (true, Some(q)) => {
                            self.walk.clear();
                            explore::decline_fight(&d, cx.gs_mut(), e);
                            let f = e.clone();
                            cx.ask_no(q, move |cx| fight_now(cx, &f, "explore"));
                        }
                        (false, Some(_)) => {
                            self.start_fight(cx, e, "explore");
                            crate::runover::warn_fight(cx, &enc);
                        }
                        _ => self.start_fight(cx, e, "explore"),
                    }
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
        self.walk.clear();
        fight_now(cx, fight, runner);
    }

    /// Faces `dir` (a step or a use): the sprite turns four ways (DIR4-M, `Facing::toward`): up or down on an up or
    /// down one (keeping its last left/right for side rows), left or right on a horizontal one.
    fn turn(&mut self, dir: Pos) {
        self.facing = dir;
        self.leader_face = self.leader_face.toward(dir.0 as i32, dir.1 as i32);
    }

    /// Whether the leader walks at the clock's now: on a walked path, or within `WALK_LINGER_MS` of the last step.
    fn moving(&self) -> bool {
        !self.walk.is_empty() || self.moved_at.is_some_and(|t| self.anim_ms - t < WALK_LINGER_MS)
    }

    fn interact(&mut self, cx: &mut Cx, dir: Pos) {
        self.turn(dir);
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
                        got.push(format!("{} {}", json::i(&res, "gold", 0), purse_word(cx, PurseWord::Currency)));
                    }
                    if json::i(&res, "supplies", 0) > 0 {
                        got.push(format!("{} {}", json::i(&res, "supplies", 0), purse_word(cx, PurseWord::Upkeep)));
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

    /// Picks reply `i` of the conversation.
    fn choose(&mut self, cx: &mut Cx, i: i64) {
        let d = cx.data.clone();
        let Some(r) = self.runner.as_mut() else { return };
        let res = r.choose(&d, cx.gs_mut(), i);
        self.dialogue_result(cx, res);
    }

    /// CAMPAIGN1 W3, R3: the question before reply `i` when it rests (the `rest` effect) and the rest's days would end
    /// the run; None otherwise (also when the rest is refused: the choice says why).
    fn rest_question(&self, cx: &Cx, i: i64) -> Option<String> {
        let r = self.runner.as_ref()?;
        let d = &cx.data;
        if !r.choice_effects(d, cx.gs(), i).iter().any(|fx| s(fx, "effect") == "rest") || !emberwake_world::campaign::purse::rest_refusal(d, cx.gs()).is_empty() {
            return None;
        }
        let days = emberwake_world::campaign::purse::rest_rule(d, &cx.gs().world_id).days as i64;
        crate::runover::days_ask(cx, days, "Resting", "Rest")
    }

    /// CAMPAIGN1 T2: a queued `{dialogue}` scene opens as the conversation (`runover::pending_talk`). True when it did.
    fn play_pending_talk(&mut self, cx: &mut Cx) -> bool {
        if cx.busy() || !cx.nav.is_empty() {
            return false;
        }
        let Some(r) = crate::runover::pending_talk(cx) else { return false };
        cx.set_busy(true);
        self.runner = Some(r);
        true
    }

    fn end_talk(&mut self, cx: &mut Cx) {
        self.runner = None;
        self.pending = None;
        self.rest_ask = None;
        self.shop = ShopPane::default();
        self.board = BoardPane::default();
        cx.set_busy(false);
    }

    /// Handles a dialogue result: fights, the turn-in picker, notes, the end.
    fn dialogue_result(&mut self, cx: &mut Cx, res: Dict) {
        if !json::b(&res, "ok", false) {
            for e in json::sget(&res, "errors") {
                cx.log(e);
            }
        }
        crate::runover::take(cx, &res);
        // CAMPAIGN1 F3: a choice that ended the run ends the talk (the run-over screen follows)
        if crate::runover::is_over(cx) {
            self.end_talk(cx);
            return;
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
                got.push(format!("{} {}", json::i(g, "gold", 0), purse_word(cx, PurseWord::Currency)));
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
            // CAMPAIGN1 W3: a fight that could end the run asks first (No: the talk is over, nothing fought)
            match crate::runover::fight_ask(cx, &s(&f, "encounter")) {
                Some(q) => cx.ask_no(q, move |cx| fight_now(cx, &f, "dialogue")),
                None => self.start_fight(cx, &f, "dialogue"),
            }
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
                crate::runover::take(cx, &res);
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
                // CAMPAIGN1 W3, R3: a choice whose rest would end the run asks first (No: the talk goes on)
                if let Some(q) = self.rest_question(cx, i) {
                    self.rest_ask = Some(i);
                    cx.ask_tag_no(q, crate::runover::ASK_TALK_REST);
                    return;
                }
                self.choose(cx, i);
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
            RAct::Shop(a) => {
                let sq = self.shop_qid();
                if !sq.is_empty() && self.shop.act(cx, &sq, a) {
                    self.act(cx, RAct::CloseShop);
                }
            }
            RAct::CloseShop => {
                self.shop = ShopPane::default();
                let Some(r) = self.runner.as_mut() else { return };
                let res = r.close_shop(&d, cx.gs_mut());
                self.dialogue_result(cx, res);
            }
            RAct::CloseBoard => {
                self.board = BoardPane::default();
                let Some(r) = self.runner.as_mut() else { return };
                let res = r.close_board(&d, cx.gs_mut());
                self.dialogue_result(cx, res);
            }
            RAct::Board(a) => match a {
                BoardAct::Pick(q) => self.board.pick(q),
                BoardAct::Close => self.act(cx, RAct::CloseBoard),
                // B8: the runner's `start_quest` path; a refusal stays on the pane, a taken quest is logged
                BoardAct::Accept(q) => {
                    let Some(r) = self.runner.as_mut() else { return };
                    let res = r.accept(&d, cx.gs_mut(), &q);
                    if json::b(&res, "ok", false) {
                        self.board.errors.clear();
                        self.dialogue_result(cx, res);
                    } else {
                        self.board.errors = json::sget(&res, "errors");
                    }
                }
            },
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
        // Yes to "Discard this trade?" (PLAYUI1-P): back to the conversation
        if cx.take_answer(crate::shop::DISCARD) {
            self.act(cx, RAct::CloseShop);
        }
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        // the open conversation; a new node starts its line and its replies scrolled to the top
        let talk_view = self.runner.as_ref().filter(|r| !r.closed).map(|r| r.view(&d, &gs));
        let node = self.runner.as_ref().filter(|r| !r.closed).map(|r| format!("{}/{}", r.character_qid, r.node_id)).unwrap_or_default();
        talk::follow_node(g, &mut self.talk_node, node);
        let mut ui: Ui<RAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), rgb(14, 18, 26));
        // PORTRAITFX1-D: a portrait left out of the last frame (the conversation ended or hid) is dropped
        self.faces.begin();
        let loc = explore::location(&d, &gs);
        let t = explore::template_of(&d, loc);
        let talking = self.talking();
        let modal = self.modal();
        let shop_qid = self.shop_qid();
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
            // PROPS1 4.2-4.6: the ground props over the tiles; the standing props go into the sorted pass below. Props
            // are art only: no widget, and walking still comes from `rows`
            let props = crate::props::layout(t, &lineage, (ox, oy), ts, map_area, |dirs, variant, x, y| {
                let rel = cx.art.pick(dirs, variant, x, y)?;
                let size = cx.art.size_of(&rel)?;
                Some((rel, size))
            });
            for p in props.iter().filter(|p| p.ground) {
                draw_prop(&mut ui, cx, p);
            }
            let steps = gs.explore_int("loc_steps");
            // PLAYUI1-R: the people in the conversation turn to the party; the clock the sprites play at
            let talk_tiles: Vec<Pos> = talk_view.as_ref().map(|v| self.talk_tiles(cx, v)).unwrap_or_default();
            let now = self.anim_ms as u64;
            let ahead = (gs.position.0 + self.facing.0, gs.position.1 + self.facing.1);
            let legend_hover = self.legend_hover(live);
            // (marker, name, the name sits over a badge)
            let mut labels: Vec<(Rect, String, bool)> = Vec::new();
            // the markers to draw in the sorted pass (their widgets and names are taken here)
            let mut marks: Vec<MarkDraw> = Vec::new();
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
                let mut services = Vec::new();
                let mut npc = None;
                let mut top = r.y;
                if kind == "npc" {
                    let who = d.resolve(&gs.world_id, "characters", &s(&m, "character"));
                    services = explore::npc_services(&d, &gs, &s(who, "_qid"));
                    let (body, palette, _) = crate::screens::character_art(&d, who);
                    // 2.4: the person's body, else its class body (the chain), else today's token
                    let sprite = cx.art.first_body(&emberwake_content::art::body_chain(&body));
                    if sprite.is_some() {
                        top = sprite_rect(r).y;
                    }
                    // facing the party while in the conversation (four ways, DIR4-M: up or down when the party is
                    // more above or below, keeping the sheets' right); else the sheets' right
                    let face = if talk_tiles.contains(&at) { Facing::default().toward((gs.position.0 - at.0) as i32, (gs.position.1 - at.1) as i32) } else { Facing::default() };
                    npc = Some(NpcDraw { body, sprite, palette, initial: s(who, "name").chars().next().unwrap_or('?').to_string(), face, offset: idle_offset(&s(who, "id")) });
                }
                // 2.2: the first service is the badge (turn_in and offer are today's `ready` and `quest`)
                marks.push(MarkDraw { id: id.clone(), r, top, kind: kind.clone(), state: state.clone(), npc, badge: services.first().copied() });
                // 2.3: names always (people and exits, MAPREAD1), or on hover: the marker or its legend row under the
                // pointer, or the marker the party faces
                let named = if names_always { kind == "npc" || kind == "exit" } else { (!modal && ui.hover(r)) || legend_hover.as_deref() == Some(id.as_str()) || at == ahead };
                if named {
                    labels.push((Rect::new(r.x, top, r.w, r.h), label.clone(), !services.is_empty()));
                }
                if !modal {
                    let mut t = tip(&label, &s(&view, "verb"));
                    if !services.is_empty() {
                        t.push((services_text(&services), INFO));
                    }
                    ui.push(format!("marker.{id}"), "marker", label.clone(), r, true, false, Some(RAct::Marker(id)), t);
                }
            }
            // the sorted pass (PROPS1 4.4): standing props, markers and the party back to front by their feet (the
            // sprites stand on their tiles, so lower rows cover upper ones, spec PLAYUI1 2.4)
            let pr = Rect::new(ox + gs.position.0 as i32 * ts, oy + gs.position.1 as i32 * ts, ts, ts);
            let (pcx, pcy) = (pr.x as f32 + ts as f32 / 2.0, pr.y as f32 + ts as f32 / 2.0);
            // 2.4: the leader (the first active member): `walk` while moving, else `idle`
            let (leader_body, leader_palette) = leader_art(cx);
            let leader_sprite = cx.art.first_body(&emberwake_content::art::body_chain(&leader_body));
            let base = if self.moving() { "walk" } else { "idle" };
            self.leader.set_base(base, now);
            let drawn_party = if leader_sprite.is_some() { sprite_rect(pr) } else { pr };
            let order = crate::props::sorted(&props, &marks.iter().map(|m| m.r).collect::<Vec<_>>(), drawn_party);
            let idle = ClipPlayer::default();
            let mut sprites = Vec::new();
            let mut party_frame = None;
            for item in &order {
                match *item {
                    Item::Prop(i) => draw_prop(&mut ui, cx, &props[i]),
                    Item::Marker(i) => {
                        if let Some(sp) = draw_marker(&mut ui, cx, &marks[i], ts, &idle, now) {
                            sprites.push(sp.info(&format!("marker.{}", marks[i].id)));
                        }
                    }
                    Item::Party => {
                        let drawn = leader_sprite.as_deref().and_then(|b| draw_sprite(&mut ui, cx, b, &leader_palette, &self.leader, now, self.leader_face, pr, crate::props::PARTY));
                        match drawn {
                            Some(sp) => {
                                sprites.push(sp.info("party"));
                                party_frame = Some(sp);
                            }
                            None => {
                                ui.p.circle(pcx, pcy, ts as f32 * 0.3, crate::props::PARTY);
                                ui.p.line(pcx, pcy, pcx + self.facing.0 as f32 * ts as f32 * 0.32, pcy + self.facing.1 as f32 * ts as f32 * 0.32, rgb(40, 30, 10), 0.0);
                            }
                        }
                    }
                }
            }
            // 4.5: a standing prop drawn over the party: its outline over the props (the sprite's silhouette outline)
            if crate::props::party_hidden(&props, &order, drawn_party) {
                match party_frame.as_ref().and_then(|sp| cx.art.body_outline(&sp.body, &leader_palette, &sp.clip, sp.frame, sp.px, crate::props::PARTY, sp.flip)) {
                    Some(o) => ui.p.image(o, Rect::new(drawn_party.x - 1, drawn_party.y - 1, drawn_party.w + 2, drawn_party.h + 2)),
                    None => ui.p.circle(pcx, pcy, ts as f32 * 0.3, crate::props::PARTY),
                }
            }
            self.sprites = sprites;
            // overlays, never covered by a prop: the service badges (over a sprite's head), then the names
            for m in &marks {
                let Some(first) = m.badge else { continue };
                let br = Rect::new(m.r.x + ts - 5, m.top - 5, 9, 9);
                let px = ui.icon_px(Rect::new(0, 0, br.w + 4, br.h + 4));
                match cx.art.image(&service_icon(first), px, px) {
                    Some(img) => ui.p.image(img, br),
                    None => service_glyph(&mut ui, first, br),
                }
            }
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
        // -- The dialogue box (3.1); while the shop is open (3.3) the box hides and the shop sits centred over the dimmed
        // map; an open board (STORYUI1 B11) takes the shop's place
        let board_qid = self.board_qid();
        if let (Some(view), Some(b)) = (talk_view.as_ref(), talk_box.as_ref()) {
            if !shop_qid.is_empty() {
                ui.p.fill(map_area, SHOP_DIM);
                self.shop.draw(&mut ui, cx, &shop_qid, RAct::Shop);
            } else if !board_qid.is_empty() {
                ui.p.fill(map_area, SHOP_DIM);
                let bv = emberwake_world::board::view(&d, &gs, &board_qid);
                let pick = self.board.select(&bv);
                let det = if pick.is_empty() { Dict::new() } else { emberwake_world::board::detail(&d, &gs, &pick) };
                self.board.draw(&mut ui, cx, &bv, &det, RAct::Board);
            } else {
                draw_talk(&mut ui, cx, &mut self.faces, view, b, self.pending.is_some());
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
        // PLAYUI1-R: the sprites play on their own clock at the animation speed setting (as the battle tokens)
        let now = Instant::now();
        let speed = cx.settings.v["anim_speed"].as_f64().unwrap_or(1.0).clamp(0.5, 2.0);
        self.anim_ms += now.duration_since(self.anim_at).as_secs_f64() * 1000.0 * speed;
        self.anim_at = now;
        // PORTRAITFX1 4.5: the talk box's portrait clock moves by the frame's wall time (at most 100 ms a frame)
        self.faces.tick();
        // CAMPAIGN1 W3: a Yes to the ask before a resting reply picks it
        if cx.take_answer(crate::runover::ASK_TALK_REST) {
            if let Some(i) = self.rest_ask.take() {
                self.choose(cx, i);
            }
            return;
        }
        // STORYUI1 W2, 5.3: with nothing open, a running sequence takes over and queued vignettes play; CAMPAIGN1 W1,
        // F3: campaign notices first, then the run-over screen once the run is over; T2: the campaign queue's head
        // plays (vignettes and battles through `story::map_tick`, a dialogue scene here)
        if !self.modal() && (crate::runover::map_tick(cx) || crate::story::map_tick(cx) || self.play_pending_talk(cx)) {
            self.walk.clear();
            self.walk_target = None;
            return;
        }
        if self.walk.is_empty() || self.modal() || now < self.next_walk {
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
            if !r.shop_qid.is_empty() || !r.board_qid.is_empty() {
                return;
            }
            let view = r.view(&cx.data, cx.gs());
            if let Some(a) = talk::talk_key(k, &view) {
                self.act(cx, talk_act(a));
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
        } else if !self.shop_qid().is_empty() {
            self.act(cx, RAct::Shop(ShopAct::Close));
        } else if !self.board_qid().is_empty() {
            self.act(cx, RAct::CloseBoard);
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
            "lore": self.lore.as_ref().map(|l| l.0.clone()), "legend_open": cx.settings.v["map_legend_open"].as_bool().unwrap_or(true), "log_open": self.log_open,
            "shop": if self.shop_qid().is_empty() { Value::Null } else { self.shop.state() }, "board": Some(self.board_qid()).filter(|b| !b.is_empty()),
            "board_pane": if self.board_qid().is_empty() { Value::Null } else { self.board.state() }, "sprites": self.sprites,
            // PORTRAITFX1 4.11: the speaker's portrait while the talk box shows (the shop or the board hides it)
            "portraits": if self.talking() && self.shop_qid().is_empty() && self.board_qid().is_empty() { self.faces.state() } else { json!([]) }})
    }

    /// MUSIC2: shop while the shop is open, else the location's kind.
    fn music(&self, cx: &Cx) -> Option<String> {
        let shop = self.runner.as_ref().is_some_and(|r| !r.closed && !r.shop_qid.is_empty());
        cx.game.as_ref().and_then(|g| music::game_track(cx, music::region_context(&cx.data, &g.gs, shop)))
    }

    widget_plumbing!();
}

/// The dim over the map while the shop is open (spec PLAYUI1 3.3).
const SHOP_DIM: Rgba = rgba(0, 0, 0, 150);

impl RegionMap {
    /// The tile of the person speaking (the node's speaker, else the one talked to): their marker here that is not
    /// absent, the nearest to the party when several show them; None when they have no marker on this map.
    fn speaker_tile(&self, cx: &Cx, view: &Dict) -> Option<Pos> {
        self.talk_tiles(cx, view).first().copied()
    }

    /// The tiles of the people in the conversation (PLAYUI1-R: they face the party): the node's speaker's marker, then
    /// the one talked to (when someone else), each the nearest to the party of their markers here that are not absent.
    fn talk_tiles(&self, cx: &Cx, view: &Dict) -> Vec<Pos> {
        let d = &cx.data;
        let gs = cx.gs();
        let loc = explore::location(d, gs);
        let t = explore::template_of(d, loc);
        let loc_qid = s(loc, "_qid");
        let steps = gs.explore_int("loc_steps");
        let talked = self.runner.as_ref().map(|r| r.character_qid.clone()).unwrap_or_default();
        let mut out = Vec::new();
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
            if let Some(at) = near.filter(|at| !out.contains(at)) {
                out.push(at);
            }
        }
        out
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

/// A marker for the sorted pass (PROPS1 4.4): its id, tile, the top of what is drawn there (a sprite's head, else the
/// tile; badges and names sit over it), kind and state; a person's art; the first service's badge (drawn with the
/// overlays).
struct MarkDraw {
    id: String,
    r: Rect,
    top: i32,
    kind: String,
    state: String,
    npc: Option<NpcDraw>,
    badge: Option<&'static str>,
}

/// A person's art on the map (PLAYUI1-R): the body key, the first of its chain that loads (None: today's token), the
/// palette, the initial of the token stand-in, the facing and the person's idle offset (`idle_offset`).
struct NpcDraw {
    body: String,
    sprite: Option<String>,
    palette: String,
    initial: String,
    face: Facing,
    offset: u64,
}

/// A person's stable idle offset (ms, under `IDLE_SPREAD_MS`): FNV-1a of their character id, so the people on a map do
/// not idle in step and each always idles the same way (no random draw, no clock).
fn idle_offset(id: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in id.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h % IDLE_SPREAD_MS
}

/// The range of the idle offsets (longer than an idle loop, so any phase is possible).
const IDLE_SPREAD_MS: u64 = 4096;

/// A sprite as drawn (PLAYUI1-R): the body, the clip and frame shown, mirrored or not, the drawn rect and the pixel
/// size the frame was fitted to; the facing asked for and the view of the row shown (DIR4-M: a side row when the body
/// has no up or down row for the clip).
struct SpriteDrawn {
    body: String,
    clip: String,
    frame: u32,
    flip: bool,
    rect: Rect,
    px: i32,
    face: Facing,
    view: View,
}

impl SpriteDrawn {
    /// The agent link's line for it (`state.sprites`): `facing` the view asked for, `view` the row drawn (`side`,
    /// `down`, `up`).
    fn info(&self, id: &str) -> Value {
        json!({"id": id, "body": self.body, "clip": self.clip, "frame": self.frame, "left": self.flip, "facing": view_name(self.face.view), "view": view_name(self.view), "rect": [self.rect.x, self.rect.y, self.rect.w, self.rect.h]})
    }
}

/// A view's name on the agent link (`side`, `down`, `up`).
fn view_name(v: View) -> &'static str {
    match v {
        View::Side => "side",
        other => other.name(),
    }
}

/// The rect of a character sprite on tile `r` (spec PLAYUI1 2.4): `SPRITE` square, bottom-centred on the tile.
fn sprite_rect(r: Rect) -> Rect {
    Rect::new(r.x + (r.w - SPRITE) / 2, r.y + r.h - SPRITE, SPRITE, SPRITE)
}

/// The party leader's body key and palette (spec PLAYUI1 2.4): the first active member's character art (its base
/// unit's, its class's when it has no character), as the battle draws the member.
fn leader_art(cx: &Cx) -> (String, String) {
    let d = &cx.data;
    let gs = cx.gs();
    let Some(m) = gs.party.active_members().first().copied() else { return (String::new(), String::new()) };
    let character = d.resolve(&gs.world_id, "characters", &m.lore_ref);
    if character.is_empty() {
        let unit = d.entry("units", &m.base_unit);
        let class = d.entry("classes", &s(unit, "class_id"));
        (emberwake_content::art::body_of(&Default::default(), unit, class), emberwake_content::art::palette_of(&Default::default(), unit))
    } else {
        let (body, palette, _) = crate::screens::character_art(d, character);
        (body, palette)
    }
}

/// Body `body` standing on tile `r` (spec PLAYUI1 2.4): a foot shadow in `foot`, then the frame `player` plays at
/// `now` in `face` (`Art::played_facing`, DIR4-M: the up or down row when the body has one, else the side row,
/// mirrored facing left), fitted one source pixel per screen pixel. None when it does not play (the caller draws
/// today's token).
#[allow(clippy::too_many_arguments)]
fn draw_sprite(ui: &mut Ui<RAct>, cx: &mut Cx, body: &str, palette: &str, player: &ClipPlayer, now: u64, face: Facing, r: Rect, foot: Rgba) -> Option<SpriteDrawn> {
    let sr = sprite_rect(r);
    let px = (SPRITE as f32 * ui.p.v.s).round().max(1.0) as i32;
    let (f, img) = cx.art.played_facing(body, palette, player, now, px, face)?;
    let flip = face.mirrors(&f);
    let feet = (r.y + r.h) as f32 - 2.0;
    ui.p.ellipse(r.x as f32 + r.w as f32 / 2.0, feet, r.w as f32 * 0.4, r.h as f32 * 0.14, rgba(foot[0], foot[1], foot[2], FOOT_ALPHA));
    ui.p.image(img, sr);
    let view = if f.side { View::Side } else { face.view };
    Some(SpriteDrawn { body: body.to_string(), clip: f.clip, frame: f.frame, flip, rect: sr, px, face, view })
}

/// A marker on its tile: a person's sprite (PLAYUI1-R; their ring and token, or initial, without a body), else the
/// marker icon (or its drawn stand-in). The person's sprite as drawn.
fn draw_marker(ui: &mut Ui<RAct>, cx: &mut Cx, m: &MarkDraw, ts: i32, idle: &ClipPlayer, now: u64) -> Option<SpriteDrawn> {
    let r = m.r;
    let (cx_, cy_) = (r.x as f32 + ts as f32 / 2.0, r.y as f32 + ts as f32 / 2.0);
    if let Some(n) = &m.npc {
        let (col, _) = RegionMap::marker_style(&m.kind, &m.state);
        if let Some(sp) = n.sprite.as_deref().and_then(|b| draw_sprite(ui, cx, b, &n.palette, idle, now + n.offset, n.face, r, col)) {
            return Some(sp);
        }
        ui.p.circle(cx_, cy_, ts as f32 * 0.42, col);
        let tok = Rect::new(r.x + ts / 8, r.y + ts / 8, ts - ts / 4, ts - ts / 4);
        match cx.art.first_token(&emberwake_content::art::body_chain(&n.body), &n.palette, tok.w) {
            Some(img) => ui.p.image(img, tok),
            None => ui.p.text_center(cx_, cy_ - 4.0, &n.initial, BODY, rgb(20, 20, 20)),
        }
        return None;
    }
    let px = ui.icon_px(Rect::new(0, 0, ts + 4, ts + 4));
    match cx.art.image(&emberwake_content::art::marker(marker_icon(&m.kind, &m.state)), px, px) {
        Some(img) => ui.p.image(img, r),
        None => {
            let (col, glyph) = RegionMap::marker_style(&m.kind, &m.state);
            ui.p.panel(r.inset(ts / 5), col, rgba(255, 255, 255, 90), 2.0);
            ui.p.text_center(cx_, cy_ - 4.5, glyph, BODY, rgb(16, 16, 20));
        }
    }
    None
}

/// The foot shadow's alpha under a sprite.
const FOOT_ALPHA: u8 = 150;

/// A prop (PROPS1 4.3, 4.6): its art in its drawn rect (mirrored when flipped), one source pixel per screen pixel;
/// without art a standing prop is a plain block over its footprint (fill, a darker south face, a 1 px edge) and a
/// ground prop is nothing.
fn draw_prop(ui: &mut Ui<RAct>, cx: &mut Cx, p: &PropDraw) {
    let (w, h) = ((p.rect.w as f32 * ui.p.v.s).round().max(1.0) as i32, (p.rect.h as f32 * ui.p.v.s).round().max(1.0) as i32);
    let img = match &p.file {
        Some(f) if p.flip => cx.art.image_mirrored(f, w, h),
        Some(f) => cx.art.image(f, w, h),
        None => None,
    };
    if let Some(img) = img {
        ui.p.image(img, p.rect);
        return;
    }
    if p.ground {
        return;
    }
    let r = p.rect;
    ui.p.fill(r, BOX_FILL);
    let face = (r.h / 4).clamp(1, BOX_FACE_H);
    ui.p.fill(Rect::new(r.x, r.y + r.h - face, r.w, face), BOX_FACE);
    for edge in [Rect::new(r.x, r.y, r.w, 1), Rect::new(r.x, r.y + r.h - 1, r.w, 1), Rect::new(r.x, r.y, 1, r.h), Rect::new(r.x + r.w - 1, r.y, 1, r.h)] {
        ui.p.fill(edge, BOX_EDGE);
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

/// A service's icon (spec PLAYUI1 section 6): `turn_in` and `offer` are the quest badges `ready` and `quest`; a board
/// with work (STORYUI1 B4) is the quest badge too.
pub(crate) fn service_icon(service: &str) -> String {
    match service {
        "turn_in" => emberwake_content::art::badge("ready"),
        "offer" | "board" => emberwake_content::art::badge("quest"),
        _ => format!("ui/services/{service}.png"),
    }
}

/// "Quest to turn in, Shop" for a hover card.
pub(crate) fn services_text(services: &[&str]) -> String {
    services
        .iter()
        .map(|sv| match *sv {
            "turn_in" => "Quest to turn in",
            "offer" => "New quest",
            "board" => emberwake_world::board::BADGE_TEXT,
            "shop" => "Shop",
            "recruit" => "Recruits",
            _ => "",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The drawn stand-in of a service icon: today's `?` and `!` badges (a board with work is the `!`, STORYUI1 B4), a coin
/// for a shop, a plus for recruits.
pub(crate) fn service_glyph<A>(ui: &mut Ui<A>, service: &str, r: Rect) {
    let (cx_, cy_, sz) = (r.x as f32 + r.w as f32 / 2.0, r.y as f32 + r.h as f32 / 2.0, r.w as f32);
    match service {
        "turn_in" | "offer" | "board" => {
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

/// The talk box over the region map (`talk::draw_talk` in the game in play; its rows act as `RAct::Choose` and
/// `RAct::Advance`).
fn draw_talk(ui: &mut Ui<RAct>, cx: &mut Cx, faces: &mut Faces, view: &Dict, b: &TalkBox, busy: bool) {
    let gs = cx.gs().clone();
    talk::draw_talk(ui, cx, &gs, faces, view, b, busy, talk_act);
}

/// The region map's action of a talk box row or key.
fn talk_act(a: TalkAct) -> RAct {
    match a {
        TalkAct::Choose(i) => RAct::Choose(i),
        TalkAct::Advance => RAct::Advance,
    }
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

    /// PROPS1 3.1: map prop art is `props/<set>/<art>/v<n>.png` (a variant run, as tile sets); anything else under
    /// `props/` is a stray.
    #[test]
    fn prop_art_follows_the_variant_pattern() {
        for ok in ["props/common/tent/v0.png", "props/ashlands/keep_hall/v12.png"] {
            assert!(layout_ok(ok), "{ok}");
        }
        for stray in ["props/common/tent/tent.png", "props/common/tent/v0.jpg", "props/tent/v0.png", "props/common/tent/v0/a.png", "props/common/tent/vx.png"] {
            assert!(!layout_ok(stray), "{stray}");
        }
    }

    /// PORTRAITFX1 4.1 and 5.4: a portrait's face sidecar `<id>.face.json` sits beside it (any portrait folder);
    /// expression frames `faces/<id>/<expression>.png` (and their sidecars) only under the unique folders, named by
    /// the game's faces (PORTRAITFX2 FL4: `Catalog::faces`).
    #[test]
    fn portrait_sidecars_and_expression_frames_follow_the_layout() {
        for ok in ["portraits/npcs/tess.png", "portraits/npcs/tess.face.json", "portraits/common/beast_render_beast.face.json", "portraits/heroes/faces/kaelen/hurt.png", "portraits/bosses/faces/proctor_quell/neutral.face.json"] {
            assert!(layout_ok(ok), "{ok}");
        }
        for stray in ["portraits/npcs/tess.json", "portraits/heroes/faces/kaelen/smirk.png", "portraits/common/faces/beast/hurt.png", "portraits/npcs/faces/tess.png", "portraits/npcs/faces/tess/hurt/a.png", "portraits/npcs/faces/tess/hurt.jpg"] {
            assert!(!layout_ok(stray), "{stray}");
        }
    }

    /// The shipped data's faces (PORTRAITFX2 FL1), read once.
    fn shipped_faces() -> &'static [String] {
        static FACES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
        FACES.get_or_init(|| {
            let dir = emberwake_world::Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
            emberwake_world::Data::open(&dir).expect("data").0.cat.faces()
        })
    }

    /// One path of the section 3 tree (game, kit or engine root), plus the PLAYUI1 section 6 icons: toolkit glyphs
    /// `ui/<id>.png` and the `ui/hud`, `ui/markers` and `ui/services` folders, PROPS1's `props/<set>/<art>/`, the
    /// story scene pictures `backgrounds/story/<id>.png` (STORYART1, named by a vignette card's `image`) and the
    /// portrait sidecars and expression frames (PORTRAITFX1).
    fn layout_ok(rel: &str) -> bool {
        let p: Vec<&str> = rel.split('/').collect();
        let png = |s: &str| s.ends_with(".png");
        let face = |s: &str| png(s) || s.ends_with(".face.json");
        let expression = |s: &str| shipped_faces().iter().any(|e| s == format!("{e}.png") || s == format!("{e}.face.json"));
        let v = |s: &str| s.starts_with('v') && s.trim_start_matches('v').trim_end_matches(".png").parse::<u32>().is_ok() && png(s);
        if p.last() == Some(&"README.md") || matches!(p.as_slice(), ["fonts", ..]) {
            return true;
        }
        match p.as_slice() {
            ["units", _, _, "sheet.png" | "clips.json"] => true,
            ["units", _, _, "palettes", f] => f.ends_with(".json"),
            ["portraits", "common" | "heroes" | "npcs" | "elites" | "bosses", f] => face(f),
            ["portraits", "heroes" | "npcs" | "elites" | "bosses", "faces", _, f] => expression(f),
            ["items", _, f] => png(f),
            ["skills", "kinds", f] | ["skills", f] | ["statuses", f] | ["map", "markers", f] => png(f),
            ["ui", "pins" | "badges" | "chips" | "intents" | "hud" | "markers" | "services", f] | ["ui", f] => png(f),
            ["tilesets", _, _, f] | ["props", _, _, f] => v(f),
            ["backgrounds", "menu" | "title" | "world_map" | "location" | "battle" | "story", f] => png(f),
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
    use crate::talk::{PORTRAIT, TALK_GAP, TALK_LEFT, TALK_MAX_PCT, TALK_MIN_H, TALK_PAD, TALK_W};
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
    pub(super) fn each_talk(cx: &mut Cx, mut each: impl FnMut(&mut RegionMap, &mut Cx, Pos, Pos) -> bool) -> usize {
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
            draw_talk(&mut ui, &mut cx, &mut Faces::default(), &view, &b, false);
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

/// PORTRAITFX1-D (docs/specs/PORTRAITFX1.md 6.2 row 1; PORTRAITFX1_TASKS 1.5 D1, D2, D4, D5): the talk box's portrait
/// lives, drawn headless over a debug game of the first playable world with a scratch portrait layer (a sidecar with
/// eyes on a Tallow Amber fill, so the rim and the blink show) and cue rows set by each test, not the shipped ones.
#[cfg(test)]
mod portrait_tests {
    use super::talk_tests::each_talk;
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx, rect};
    use crate::portraits::tests::{layer, row};
    use crate::portraits::RIM;
    use aegis_core::assets::Layers;
    use aegis_ui::art::Art;
    use aegis_ui::fx::Clock;
    use aegis_ui::portrait_fx::{blink_phase, blink_starts, seed_of, tq, BLINK_MS, EASE_MS};
    use aegis_ui::screen::TEXT_SIZES;
    use emberwake_data::PortraitCue;

    /// A game whose cue rows are `rows` (none of the shipped data's).
    fn cx_with(rows: Vec<PortraitCue>) -> Cx {
        let mut cx = game_cx();
        std::sync::Arc::get_mut(&mut cx.app.data).expect("the test owns the data").edit_cat(|c| c.portrait_cues = rows);
        cx
    }

    /// A scratch layer holding `character`'s portrait (the first file of its chain) with eyes, a dark lid and the
    /// game's rim key ("").
    fn portrait_art(cx: &mut Cx, name: &str, character: &Dict) {
        let rel = crate::screens::character_art(&cx.data, character).2.first().cloned().expect("a portrait chain");
        let face = json!({"size": 96, "eyes": [[30, 40, 10, 6], [56, 40, 10, 6]], "lid": "#203040", "rim": ""});
        cx.art = Art::layered(Layers::of(vec![layer(name, &rel, 96, [RIM[0], RIM[1], RIM[2]], Some(face))]));
    }

    /// The speaker of the open conversation on `map`.
    fn speaker(map: &RegionMap, cx: &Cx) -> Dict {
        let view = map.state(cx)["dialogue"].clone();
        cx.data.resolve(&cx.gs().world_id, "characters", &s(json::dict(&view), "speaker")).clone()
    }

    /// The talk box portrait composed at clock `t` (one frame of the map).
    fn composed_at(map: &mut RegionMap, cx: &mut Cx, g: &mut Gfx, key: &str, t: u64) -> Vec<u8> {
        map.faces.clock = Clock { ms: t as f64, hold: 0.0 };
        frame(map, cx, g, 1.0);
        map.faces.get(key).and_then(|f| f.composed()).map(|i| i.px.clone()).expect("the speaker's portrait composed")
    }

    /// D1 and D5: the talk box portrait changes pixels over 4 s by default (the rim flickers; breath at the full tier);
    /// under reduce motion the frames differ only where a blink is closed or half closed (4.7). While the conversation
    /// is open the map's `state` lists the speaker's portrait; with none open, no portrait.
    #[test]
    fn d1_d5_the_talk_box_portrait_lives_and_shows_in_the_state() {
        let mut cx = cx_with(Vec::new());
        let mut g = gfx();
        let quiet = RegionMap::new(&mut cx);
        assert_eq!(quiet.state(&cx)["portraits"], json!([]), "no conversation: no portrait");
        let mut checked = false;
        each_talk(&mut cx, |map, cx, _, _| {
            let character = speaker(map, cx);
            let key = s(&character, "id");
            if key.is_empty() {
                return true;
            }
            portrait_art(cx, "talk_d1", &character);
            let seed = seed_of(&key);
            // every 100 ms over 4 s, and the middle of the first blink (wherever it falls)
            let first = blink_starts(seed, 10_000).first().copied().expect("a blink within 10 s");
            let mut times: Vec<u64> = (0..=40).map(|k| k * 100).collect();
            times.push(first + BLINK_MS / 2);
            let open = |t: u64| blink_phase(seed, tq(t)) == 0;
            for reduce in [false, true] {
                cx.settings.v["reduce_motion"] = json!(reduce);
                let frames: Vec<(u64, Vec<u8>)> = times.iter().map(|&t| (t, composed_at(map, cx, &mut g, &key, t))).collect();
                let still: Vec<&(u64, Vec<u8>)> = frames.iter().filter(|f| open(f.0)).collect();
                let blinks: Vec<&(u64, Vec<u8>)> = frames.iter().filter(|f| !open(f.0)).collect();
                assert!(!blinks.is_empty(), "a sample inside a blink");
                if reduce {
                    assert!(still.iter().all(|f| f.1 == still[0].1), "reduce motion: the open-eyed frames are equal");
                    assert!(blinks.iter().all(|f| f.1 != still[0].1), "reduce motion: a blink still shows");
                } else {
                    assert!(still.iter().any(|f| f.1 != still[0].1), "by default the open-eyed frames change over 4 s");
                }
            }
            let st = map.state(cx);
            let listed = st["portraits"].as_array().cloned().unwrap_or_default();
            assert!(listed.iter().any(|p| p["key"] == json!(key)), "the speaker's portrait is in the state: {st}");
            assert!(listed.iter().all(|p| p["key"] == json!(key)), "only the speaker's");
            map.end_talk(cx);
            frame(map, cx, &mut g, 1.0);
            assert_eq!(map.state(cx)["portraits"], json!([]), "the conversation ended: no portrait");
            checked = true;
            false
        });
        assert!(checked, "a conversation with a speaker");
    }

    /// A view of a conversation with `character` (as `DialogueRunner::view` shapes it).
    fn view_of(character: &Dict) -> Dict {
        json::dict(&json!({"speaker": s(character, "_qid"), "speaker_name": s(character, "name"), "node": "n", "text": "A line.", "choices": [{"text": "Farewell.", "locked": false, "reason": "", "service": "farewell"}],
            "end": false, "next": false, "faction": "", "faction_name": "", "band": "neutral"}))
        .clone()
    }

    /// Draws the talk box for `view` at clock `t` into `faces`, at text size `scale`: its widgets and the texts drawn.
    fn talk_at(cx: &mut Cx, g: &mut Gfx, faces: &mut Faces, view: &Dict, t: u64, scale: f32) -> (Vec<Value>, Vec<aegis_ui::painter::TextRec>) {
        faces.clock = Clock { ms: t as f64, hold: 0.0 };
        g.fonts.scale = scale;
        g.fonts.audit = Some(Vec::new());
        let mut ui: Ui<RAct> = Ui::new(g, (-500, -500), true);
        faces.begin();
        let b = talk_layout(&mut ui.p, view);
        draw_talk(&mut ui, cx, faces, view, &b, false);
        let widgets = ui.widgets.iter().map(|w| w.info()).collect();
        drop(ui);
        (widgets, g.fonts.audit.take().unwrap_or_default())
    }

    /// D2 and D4: a speaker played by a wounded party member draws the `wounded` row's desaturate 0.35 (with the rows of
    /// PORTRAITFX1_TASKS 2.3); a world character no member plays matches no `hp` row (it has no HP); the talk box with a
    /// drawn portrait reads clean (TEXTFIT1) at every text size.
    #[test]
    fn d2_d4_a_wounded_member_speaker_is_desaturated_and_the_box_reads_clean() {
        let rows = vec![row(json!({"id": "wounded", "trigger": "condition", "condition": "wounded", "effect": "desaturate", "intensity": 0.35})), row(json!({"id": "any_hp", "trigger": "hp", "below": 100, "effect": "tint", "intensity": 0.5}))];
        let mut cx = cx_with(rows);
        let mut g = gfx();
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        let at = cx.gs().party.members.iter().position(|m| !emberwake_world::progression::character_of(&d, m).is_empty()).expect("a member who plays a character");
        cx.gs_mut().party.members[at].conditions = vec!["wounded".to_string()];
        let character = emberwake_world::progression::character_of(&d, &cx.gs().party.members[at]).clone();
        portrait_art(&mut cx, "talk_d2", &character);
        let view = view_of(&character);
        let key = s(&character, "id");
        let mut faces = Faces::default();
        for t in [1000, 1000 + EASE_MS] {
            talk_at(&mut cx, &mut g, &mut faces, &view, t, 1.0);
        }
        let fx = faces.get(&key).expect("the member's portrait");
        let now = faces.now();
        assert!(fx.level("desaturate", now).is_some_and(|l| (l.i - 0.35).abs() < 1e-3), "wounded: desaturate 0.35");
        assert!(fx.level("tint", now).is_none(), "the member's HP is full: the hp row does not match");
        assert!(fx.live(now).contains(&"desaturate"));
        // a world character no member plays: no hp row, whatever its HP would be
        let played: Vec<String> = cx.gs().party.members.iter().map(|m| s(emberwake_world::progression::character_of(&d, m), "_qid")).collect();
        let other = d.merged_entries(&world, "characters").into_iter().find(|c| !played.contains(&s(c, "_qid"))).expect("a character no member plays").clone();
        portrait_art(&mut cx, "talk_d2_other", &other);
        let mut faces = Faces::default();
        let view = view_of(&other);
        for t in [1000, 1000 + EASE_MS] {
            talk_at(&mut cx, &mut g, &mut faces, &view, t, 1.0);
        }
        let fx = faces.get(&s(&other, "id")).expect("the character's portrait");
        assert!(fx.level("tint", faces.now()).is_none() && fx.level("desaturate", faces.now()).is_none(), "a world character matches no hp or condition row");
        // D4: the box reads clean with the portrait drawn, at every text size
        for (size, scale) in TEXT_SIZES {
            let (widgets, recs) = talk_at(&mut cx, &mut g, &mut faces, &view, 2000, scale as f32);
            assert!(faces.get(&s(&other, "id")).and_then(|f| f.composed()).is_some(), "{size}: the portrait drew");
            let boxes: Vec<(String, f32, f32, f32, f32)> = widgets
                .iter()
                .map(|w| {
                    let (x, y, ww, hh) = rect(w);
                    (s(json::dict(w), "id"), x as f32, y as f32, ww as f32, hh as f32)
                })
                .collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            assert_eq!(v["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "{size}: {v}");
        }
    }
}

/// PLAYUI1-R (spec PLAYUI1 2.4): the leader and the people as body sprites, drawn headless over a debug game of the
/// first playable world with the game's art layers (the game's, its kit's and the engine's assets).
#[cfg(test)]
mod sprite_tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx};

    /// A debug game with the game's art.
    fn art_cx() -> Cx {
        let mut cx = game_cx();
        cx.art = aegis_ui::art::Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        cx
    }

    fn sprite(map: &RegionMap, cx: &Cx, id: &str) -> Option<Value> {
        map.state(cx)["sprites"].as_array()?.iter().find(|s| s["id"] == id).cloned()
    }

    fn rect_of(v: &Value) -> Rect {
        let n = |i: usize| v["rect"][i].as_i64().unwrap_or(0) as i32;
        Rect::new(n(0), n(1), n(2), n(3))
    }

    /// The map tile at (`x`, `y`) in the last frame.
    fn tile_rect(map: &RegionMap, at: Pos) -> Rect {
        let (ox, oy, ts) = map.geo;
        Rect::new(ox + at.0 as i32 * ts, oy + at.1 as i32 * ts, ts, ts)
    }

    /// The tile a sprite stands on: under the middle of its bottom edge.
    fn stands_on(map: &RegionMap, r: Rect) -> Pos {
        let (ox, oy, ts) = map.geo;
        (((r.x + r.w / 2 - ox).div_euclid(ts)) as i64, ((r.y + r.h - 1 - oy).div_euclid(ts)) as i64)
    }

    /// The first active member's body as the art resolves it: their character's body, else the class body.
    fn leader_body(cx: &mut Cx) -> Option<String> {
        let d = cx.data.clone();
        let gs = cx.gs().clone();
        let m = gs.party.active_members().first().copied().cloned()?;
        let ch = d.resolve(&gs.world_id, "characters", &m.lore_ref);
        let (body, _, _) = crate::screens::character_art(&d, ch);
        cx.art.first_body(&emberwake_content::art::body_chain(&body))
    }

    /// 2.4: the leader stands on the party's tile, 1.5 tiles tall and bottom-centred, idle and facing right at the start.
    #[test]
    fn the_leader_stands_on_the_party_tile() {
        let (mut cx, mut g) = (art_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        let want = leader_body(&mut cx).expect("the leader has a body");
        let sp = sprite(&map, &cx, "party").expect("the leader is a sprite");
        assert_eq!(sp["body"], json!(want));
        assert_eq!((sp["clip"].clone(), sp["left"].clone()), (json!("idle"), json!(false)));
        let (r, t) = (rect_of(&sp), tile_rect(&map, cx.gs().position));
        assert_eq!((r.w, r.h), (TILE * 3 / 2, TILE * 3 / 2), "1.5 tiles tall");
        assert_eq!((r.y + r.h, r.x + r.w / 2), (t.y + t.h, t.x + t.w / 2), "bottom-centred on the tile");
    }

    /// The view a body draws `clip` in for a `facing` (`side`, `down`, `up`): the facing's view when the body has a
    /// row of that view for the clip, else the side row (DIR4 3.2).
    fn drawn_view(cx: &mut Cx, body: &str, clip: &str, facing: &str) -> &'static str {
        let has = cx.art.clips(body).is_some_and(|c| c.clips.iter().any(|c| c.of == clip && view_name(c.view) == facing));
        match facing {
            "down" if has => "down",
            "up" if has => "up",
            _ => "side",
        }
    }

    /// 2.4 and DIR4-M: a step plays `walk` and a step left mirrors the sprite; a down or up step faces down or up
    /// (`Facing::toward`, keeping the last left/right); a step right turns it back; standing still it plays `idle`
    /// again. The row drawn is the facing's when the body has one, else the side row.
    #[test]
    fn steps_walk_and_turn_the_leader() {
        let (mut cx, mut g) = (art_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        let d = cx.data.clone();
        let start = cx.gs().clone();
        let t = explore::template_of(&d, explore::location(&d, &start)).clone();
        let (w, h) = explore::size_of(&t);
        let marked: Vec<Pos> = explore::markers_of(explore::location(&d, &start)).iter().map(|m| explore::marker_pos(&t, m, start.explore_int("loc_steps"))).collect();
        let free = |p: Pos| explore::is_open(&t, p.0, p.1) && !marked.contains(&p);
        // a free tile with free tiles left of it and below that one
        let at = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).find(|&(x, y)| free((x, y)) && free((x - 1, y)) && free((x - 1, y + 1)) && free((x, y + 1))).expect("room for three steps");
        cx.gs_mut().position = at;
        let mut checks = 0;
        // (step, facing, left/right kept): left, down (keeps left), right, up (keeps right), back to the start
        for (dir, facing, left) in [((-1, 0), "side", true), ((0, 1), "down", true), ((1, 0), "side", false), ((0, -1), "up", false)] {
            let from = cx.gs().position;
            map.step(&mut cx, dir);
            assert_eq!(cx.gs().position, (from.0 + dir.0, from.1 + dir.1), "the step {dir:?} from {from:?}: {:?}", cx.log);
            let want = Facing { view: View::parse(facing).unwrap_or(View::Side), left };
            assert_eq!(map.leader_face, want, "the facing after the step {dir:?}");
            frame(&mut map, &mut cx, &mut g, 1.0);
            let sp = sprite(&map, &cx, "party").expect("the leader");
            let body = s(json::dict(&sp), "body");
            let view = drawn_view(&mut cx, &body, "walk", facing);
            assert_eq!((sp["clip"].as_str().map(|c| c.starts_with("walk")), sp["facing"].clone(), sp["view"].clone()), (Some(true), json!(facing), json!(view)), "after the step {dir:?}");
            // a side row mirrors facing left; an up or down row never does
            assert_eq!(sp["left"], json!(view == "side" && left), "mirrored after the step {dir:?}");
            checks += 1;
        }
        assert_eq!(checks, 4);
        map.anim_ms += WALK_LINGER_MS + 1.0;
        frame(&mut map, &mut cx, &mut g, 1.0);
        // standing still after the last step (up): idle in the view it faces, the side row when the body has no up row
        let sp = sprite(&map, &cx, "party").expect("the leader");
        let view = drawn_view(&mut cx, &s(json::dict(&sp), "body"), "idle", "up");
        let want = if view == "side" { "idle".to_string() } else { format!("idle_{view}") };
        assert_eq!((sp["clip"].clone(), sp["view"].clone()), (json!(want), json!(view)), "standing still");
    }

    /// 2.4 and DIR4-M: every person with a body is a sprite on their tile (idle, facing right); in a conversation
    /// opened from any side the person turns to the party four ways (`Facing::toward`: left when the party is to their
    /// left, down or up when it stands straight below or above, keeping the sheets' right), and the leader faces them
    /// the same way. The row drawn is the facing's when the body has one, else the side row.
    #[test]
    fn people_are_sprites_and_face_the_party_while_talking() {
        let (mut cx, mut g) = (art_cx(), gfx());
        // (left, right, down or up) conversations checked
        let mut sides = (0, 0, 0);
        let n = super::talk_tests::each_talk(&mut cx, |map, cx, party, npc| {
            frame(map, cx, &mut g, 1.0);
            let sprites = map.state(cx)["sprites"].as_array().cloned().unwrap_or_default();
            let them = sprites.iter().find(|s| s["id"] != "party" && stands_on(map, rect_of(s)) == npc).cloned();
            let Some(them) = them else {
                // no body anywhere in their chain: today's token
                return true;
            };
            let (dx, dy) = ((party.0 - npc.0) as i32, (party.1 - npc.1) as i32);
            let want = Facing::default().toward(dx, dy);
            let facing = view_name(want.view);
            let view = drawn_view(cx, &s(json::dict(&them), "body"), "idle", facing);
            assert_eq!(them["clip"].as_str().map(|c| c.starts_with("idle")), Some(true), "{npc:?}");
            assert_eq!((them["facing"].clone(), them["view"].clone()), (json!(facing), json!(view)), "{} at {npc:?} talking with the party at {party:?}", them["id"]);
            assert_eq!(them["left"], json!(view == "side" && want.left), "{} at {npc:?} talking with the party at {party:?}", them["id"]);
            // the leader faces them (from a fresh map: the sheets' right kept on an up or down turn)
            let lead = sprites.iter().find(|s| s["id"] == "party").expect("the leader");
            let lead_want = Facing::default().toward(-dx, -dy);
            assert_eq!(map.leader_face, lead_want, "the leader faces {npc:?} from {party:?}");
            assert_eq!(lead["facing"], json!(view_name(lead_want.view)), "the leader faces {npc:?} from {party:?}");
            if dy.abs() > dx.abs() {
                sides.2 += 1;
            } else if want.left {
                sides.0 += 1;
            } else {
                sides.1 += 1;
            }
            map.end_talk(cx);
            frame(map, cx, &mut g, 1.0);
            let after = sprite(map, cx, them["id"].as_str().unwrap_or("")).expect("still drawn");
            assert_eq!((after["left"].clone(), after["facing"].clone()), (json!(false), json!("side")), "out of the conversation the sheets face right");
            true
        });
        println!("conversations checked (left, right, down or up): {sides:?}");
        assert!(n > 0, "someone on a map of the world talks");
        assert!(sides.0 > 0 && sides.1 > 0 && sides.2 > 0, "talked to from both sides and from above or below: {sides:?}");
    }

    /// 2.4: on every map of the world each person whose body chain loads is a sprite, and the sprites draw in row
    /// order (lower rows cover upper ones); the names sit over the sprites' heads.
    #[test]
    fn sprites_draw_in_row_order_with_names_above_their_heads() {
        let (mut cx, mut g) = (art_cx(), gfx());
        cx.settings.v["map_names"] = json!("always");
        cx.settings.v["map_legend_open"] = json!(false);
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        let mut people = 0;
        for loc in d.merged_entries(&world, "locations") {
            let t = explore::template_of(&d, loc).clone();
            if t.is_empty() {
                continue;
            }
            cx.gs_mut().location_id = s(loc, "id");
            let _ = RegionMap::new(&mut cx);
            let here = cx.gs().clone();
            // the party on an open tile next to each person in turn (the camera follows it)
            let spots: Vec<Pos> = explore::markers_of(loc)
                .iter()
                .filter(|m| s(m, "kind") == "npc")
                .map(|m| explore::marker_pos(&t, m, here.explore_int("loc_steps")))
                .filter_map(|at| explore::DIRS.iter().map(|d| (at.0 + d.0, at.1 + d.1)).find(|p| explore::is_open(&t, p.0, p.1)))
                .collect();
            for spot in spots {
            *cx.gs_mut() = here.clone();
            let mut map = RegionMap::new(&mut cx);
            cx.gs_mut().position = spot;
            let recs = frame(&mut map, &mut cx, &mut g, 1.0);
            let sprites = map.state(&cx)["sprites"].as_array().cloned().unwrap_or_default();
            let feet: Vec<i32> = sprites.iter().map(|s| rect_of(s).y + rect_of(s).h).collect();
            assert!(feet.windows(2).all(|w| w[0] <= w[1]), "{}: sprites back to front by their feet: {feet:?}", s(loc, "id"));
            let gs = cx.gs().clone();
            for wd in map.widgets().iter().filter(|w| w["kind"] == "marker") {
                let id = s(json::dict(wd), "id");
                let Some(m) = explore::markers_of(loc).into_iter().find(|m| format!("marker.{}", s(m, "id")) == id) else { continue };
                if s(&m, "kind") != "npc" {
                    continue;
                }
                let who = d.resolve(&world, "characters", &s(&m, "character"));
                let (body, _, _) = crate::screens::character_art(&d, who);
                let has = cx.art.first_body(&emberwake_content::art::body_chain(&body)).is_some();
                let sp = sprite(&map, &cx, &id);
                assert_eq!(sp.is_some(), has, "{id}: a sprite when the body chain of '{body}' loads");
                let Some(sp) = sp else { continue };
                people += 1;
                let r = rect_of(&sp);
                let tr = tile_rect(&map, explore::marker_pos(&t, &m, gs.explore_int("loc_steps")));
                assert_eq!((r.y + r.h, r.x + r.w / 2), (tr.y + tr.h, tr.x + tr.w / 2), "{id}: bottom-centred on its tile");
                let label = s(json::dict(wd), "text");
                let named = recs.iter().any(|t| t.text == label && t.y + t.h <= r.y as f32 + 1.0 && t.x <= (r.x + r.w) as f32 && t.x + t.w >= r.x as f32);
                let inside = r.y > HUD_H + 14;
                assert!(named || !inside, "{id}: '{label}' above the sprite's head {r:?} at {:?}", gs.position);
            }
            }
        }
        assert!(people > 0, "a person with a body on a map of the world");
    }

    /// The idle offset is a pure function of the id (the same person always idles the same way) and spreads the ids.
    #[test]
    fn idle_offsets_are_stable_per_person() {
        let ids = ["mara", "tev", "hest", "brask_toller", "orsa_tallow", "wenn_tallow"];
        for id in ids {
            assert_eq!(idle_offset(id), idle_offset(id), "{id}");
            assert!(idle_offset(id) < IDLE_SPREAD_MS, "{id}");
        }
        let mut seen: Vec<u64> = ids.iter().map(|id| idle_offset(id)).collect();
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), ids.len(), "distinct ids, distinct offsets: {seen:?}");
    }

    /// Each person's idle plays at the clock plus their offset: on every map of the world (the party next to each
    /// person) every person's sprite shows the idle frame at that time, and somewhere two people with different ids
    /// show different frames at the same clock time.
    #[test]
    fn people_idle_out_of_step() {
        let (mut cx, mut g) = (art_cx(), gfx());
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        let mut apart = false;
        let mut checked = 0;
        for loc in d.merged_entries(&world, "locations") {
            let t = explore::template_of(&d, loc).clone();
            if t.is_empty() {
                continue;
            }
            cx.gs_mut().location_id = s(loc, "id");
            let _ = RegionMap::new(&mut cx);
            let here = cx.gs().clone();
            let steps = here.explore_int("loc_steps");
            let spots: Vec<Pos> = explore::markers_of(loc)
                .iter()
                .filter(|m| s(m, "kind") == "npc")
                .map(|m| explore::marker_pos(&t, m, steps))
                .filter_map(|at| explore::DIRS.iter().map(|d| (at.0 + d.0, at.1 + d.1)).find(|p| explore::is_open(&t, p.0, p.1)))
                .collect();
            for spot in spots {
                *cx.gs_mut() = here.clone();
                let mut map = RegionMap::new(&mut cx);
                cx.gs_mut().position = spot;
                map.anim_ms = 1000.0;
                frame(&mut map, &mut cx, &mut g, 1.0);
                // (character id, frame shown) of each person drawn
                let mut shown: Vec<(String, i64)> = Vec::new();
                for m in explore::markers_of(loc).into_iter().filter(|m| s(m, "kind") == "npc") {
                    let Some(sp) = sprite(&map, &cx, &format!("marker.{}", s(&m, "id"))) else { continue };
                    let who = d.resolve(&world, "characters", &s(&m, "character"));
                    let id = s(who, "id");
                    let body = s(json::dict(&sp), "body");
                    let want = cx.art.clips(&body).and_then(|c| c.frame_at("idle", 1000 + idle_offset(&id))).expect("an idle frame");
                    assert_eq!(sp["frame"], json!(want.frame), "{id} at clock 1000 with offset {}", idle_offset(&id));
                    checked += 1;
                    shown.push((id, sp["frame"].as_i64().unwrap_or(-1)));
                }
                apart |= shown.iter().any(|a| shown.iter().any(|b| a.0 != b.0 && a.1 != b.1));
            }
        }
        assert!(checked > 1, "people drawn: {checked}");
        assert!(apart, "two people with different ids at different idle frames");
    }

    /// DIR4-M: a scratch body `key` (its `body_dir`) under `root` with clips `ids` (one 8 x 8 frame each, 100 ms, looping),
    /// each `<of>_<view>` id whose side clip is listed a view row of it; every row a lopsided bar (so a mirror shows)
    /// in its own colour (so the rows differ).
    fn scratch_body(root: &std::path::Path, key: &str, ids: &[&str]) {
        const F: u32 = 8;
        let dir = root.join(aegis_core::assets::body_dir(key));
        std::fs::create_dir_all(&dir).unwrap();
        let mut px = vec![0u8; (F * F * ids.len() as u32 * 4) as usize];
        for row in 0..ids.len() as u32 {
            for x in [1u32, 2, 3, 6] {
                let i = (((row * F + 4) * F + x) * 4) as usize;
                px[i..i + 4].copy_from_slice(&[40 + 30 * row as u8, 140, 60, 255]);
            }
        }
        let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(dir.join("sheet.png")).unwrap()), F, F * ids.len() as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&px).unwrap();
        let clips: Vec<Value> = ids
            .iter()
            .map(|id| {
                let mut c = json!({"id": id, "frames": 1, "ms": [100], "loop": true});
                for v in ["up", "down"] {
                    if let Some(of) = id.strip_suffix(&format!("_{v}")).filter(|of| ids.contains(of)) {
                        c["of"] = json!(of);
                        c["view"] = json!(v);
                    }
                }
                c
            })
            .collect();
        std::fs::write(dir.join("clips.json"), json!({"frame": [F, F], "clips": clips}).to_string()).unwrap();
    }

    /// The canvas after a frame with the leader in `face`.
    fn canvas_facing(map: &mut RegionMap, cx: &mut Cx, g: &mut Gfx, face: Facing) -> Vec<u8> {
        map.leader_face = face;
        frame(map, cx, g, 1.0);
        g.canvas.px.clone()
    }

    /// Every facing: side, down and up, each right and left.
    fn facings() -> Vec<Facing> {
        [View::Side, View::Down, View::Up].into_iter().flat_map(|view| [false, true].map(|left| Facing { view, left })).collect()
    }

    /// DIR4-M (as DIR4-G check 5): a body without up or down rows (a scratch side body as the leader's, and the
    /// shipped leader's body when it has none) draws the map in any facing pixel for pixel as the side facing with
    /// the same left/right (PLAYUI1-R); a body with the rows draws them facing down or up (never mirrored), so the map
    /// differs from the side facing there.
    #[test]
    fn a_body_without_view_rows_draws_the_side_row_exactly_as_before() {
        let (mut cx, mut g) = (art_cx(), gfx());
        let chain = emberwake_content::art::body_chain(&leader_art(&cx).0);
        let key = chain.first().cloned().expect("the leader has a body chain");
        let same_as_side = |cx: &mut Cx, g: &mut Gfx, what: &str| {
            let mut map = RegionMap::new(cx);
            for f in facings() {
                let want = canvas_facing(&mut map, cx, g, Facing::side(f.left));
                let got = canvas_facing(&mut map, cx, g, f);
                assert!(got == want, "{what}: the map in {f:?} draws as the side facing");
                let sp = sprite(&map, cx, "party").expect("the leader is a sprite");
                assert_eq!((sp["view"].clone(), sp["left"].clone()), (json!("side"), json!(f.left)), "{what} in {f:?}");
            }
        };
        // the shipped leader's body, when it has no view rows yet
        let shipped = leader_body(&mut cx).expect("the leader has a body");
        if cx.art.clips(&shipped).is_some_and(|c| c.clips.iter().all(|c| c.view == View::Side)) {
            same_as_side(&mut cx, &mut g, &shipped);
        } else {
            println!("the shipped leader body {shipped} has view rows: checked by the scratch bodies only");
        }
        // scratch bodies as the leader's: side clips only, then with front and back rows
        let root = std::env::temp_dir().join(format!("emberwake_dir4m_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let side = root.join("side");
        scratch_body(&side, &key, &["idle", "walk"]);
        cx.art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![side.clone()]));
        assert_eq!(cx.art.body_problems(&key), Vec::<String>::new(), "the scratch side body loads");
        same_as_side(&mut cx, &mut g, "the scratch side body");
        let views = root.join("views");
        scratch_body(&views, &key, &["idle", "walk", "idle_down", "idle_up", "walk_down", "walk_up"]);
        cx.art = aegis_ui::art::Art::layered(aegis_core::assets::Layers::of(vec![views.clone()]));
        assert_eq!(cx.art.body_problems(&key), Vec::<String>::new(), "the scratch view body loads");
        let mut map = RegionMap::new(&mut cx);
        for f in facings().into_iter().filter(|f| f.view != View::Side) {
            let side_px = canvas_facing(&mut map, &mut cx, &mut g, Facing::side(f.left));
            let got = canvas_facing(&mut map, &mut cx, &mut g, f);
            let sp = sprite(&map, &cx, "party").expect("the leader is a sprite");
            let name = view_name(f.view);
            assert_eq!((sp["clip"].clone(), sp["view"].clone(), sp["left"].clone()), (json!(format!("idle_{name}")), json!(name), json!(false)), "in {f:?}");
            assert!(got != side_px, "the {name} row differs from the side row on the map ({f:?})");
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// 2.4 fallback: with no body (the art layers off) the party is today's token, and no sprite is drawn.
    #[test]
    fn without_a_body_the_party_is_todays_token() {
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut map = RegionMap::new(&mut cx);
        frame(&mut map, &mut cx, &mut g, 1.0);
        assert_eq!(map.state(&cx)["sprites"], json!([]));
        let t = tile_rect(&map, cx.gs().position);
        // the token's ring in the party colour on the party's tile
        let near = |px: &[u8]| px.iter().zip(crate::props::PARTY.iter()).all(|(a, b)| (*a as i32 - *b as i32).abs() <= 24);
        let ring = (t.y..t.y + t.h).flat_map(|y| (t.x..t.x + t.w).map(move |x| (x, y))).filter(|&(x, y)| near(&g.canvas.px[((y * g.canvas.w + x) * 4) as usize..][..3])).count();
        assert!(ring >= 8, "the token's ring on the party's tile: {ring} pixels");
    }
}
