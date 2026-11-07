//! CLN-41: the journey golden, the only regression over the whole game loop. Seeded random walks (`SEEDS` per playable
//! world, `STEPS` actions each; seeds 3 and 4 were picked so that the four walks together cover the loop, see
//! `the_golden_covers_the_game_loop`) pick a legal action from what the screens show (explore: walk to a marker, talk,
//! open chests, take exits; dialogue: choices, quest turn-ins, offers, shops, boards; the world map: travel, camp,
//! enter; and in between: hurt, level-ups, abandon, journal turn-ins, a save round trip through `save::write` and
//! `save::read`) and run it through the game loop. Fights are not played (the battle goldens cover them): an outcome
//! is drawn and handed to the after-fight of the runner that started the fight (explore, travel or dialogue).
//!
//! Each walk is a trace of lines (`tests/fixtures/journey_golden.json`): per step the action, its result and digests
//! of the game state and of the screen views after it, the full state every `FULL_EVERY` steps; compared line by line.
//! The walker is the port of the retired Godot recorder (`parity/godot/journey.gd`, which replayed into this test until
//! CLN-41), plus the quest board (accept, close).
//!
//! The frozen data (decision "Godot parity retired" Q-A6): the catalog is the timeline golden's
//! (`emberwake_battle/tests/fixtures/timeline_kit.json`) and the worlds and game data are the sim golden's
//! (`emberwake_sim/tests/fixtures/sim_data.json`), so shipped content edits never churn the golden. The golden was
//! recorded once from the engine (CLN-41, 2026-11-01). Re-record only for an intended game-loop change, named in the
//! commit and a docs/HISTORY.md row; never edit it by hand:
//!
//! `RECORD=1 tools/fc.sh test --test journey`

use aegis_core::ghash::string_hash;
use aegis_core::json::{self, int, Dict};
use aegis_core::rng::GodotRng;
use emberwake_world::dialogue::{self, DialogueRunner, OFFER_PREFIX};
use emberwake_world::{board, explore, new_game, progression, quest, save, shop, travel, Data, GameState};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::path::{Path, PathBuf};

const SEEDS: [i64; 2] = [3, 4];
const STEPS: usize = 700;
const FULL_EVERY: usize = 50;
const DIRS: [(i64, i64); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
const STATS: [&str; 6] = ["STR", "DEX", "INT", "CONS", "PERC", "KNOW"];
/// Goal-directed walking: a present marker is picked by these weights.
const GOAL_WEIGHTS: [(&str, i64); 7] = [("npc", 4), ("exit", 2), ("chest", 2), ("lore", 1), ("encounter", 3), ("roamer", 2), ("point", 1)];
const SOLID: [&str; 3] = ["npc", "chest", "lore"];

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn golden_path() -> PathBuf {
    crate_dir().join("tests/fixtures/journey_golden.json")
}

fn recording() -> bool {
    std::env::var("RECORD").is_ok_and(|v| v == "1")
}

fn read_json(path: &Path, what: &str) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("missing {what} {}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()))
}

fn frozen_data() -> Data {
    let catalog = read_json(&crate_dir().join("../emberwake_battle/tests/fixtures/timeline_kit.json"), "the frozen catalog");
    let wg = read_json(&crate_dir().join("../emberwake_sim/tests/fixtures/sim_data.json"), "the frozen worlds and game data");
    Data::from_values(&catalog, &wg["worlds"], &wg["game"])
}

/// FNV-1a (64 bit) of the value's key-sorted JSON text: the same on every machine.
fn digest(v: &Value) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in json::sorted(v).to_string().bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

fn dir_value(d: (i64, i64)) -> Value {
    json!([d.0, d.1])
}

fn dir_of(act: &Dict) -> (i64, i64) {
    match json::aget(act, "dir") {
        [x, y] => (json::to_int(x), json::to_int(y)),
        _ => (0, 0),
    }
}

fn act<const N: usize>(pairs: [(&str, Value); N]) -> Dict {
    json::obj(pairs)
}

struct Walker<'a> {
    d: &'a Data,
    gs: GameState,
    rng: GodotRng,
    runner: Option<DialogueRunner>,
    talk_to: String,
    /// The choose() result of a turn-in waiting for confirm.
    pending: Dict,
    fight: Dict,
    fight_runner: String,
    on_world_map: bool,
    dialogue_actions: i64,
    goal: String,
    goal_loc: String,
    save_dir: PathBuf,
}

impl Walker<'_> {
    fn pick(&mut self, size: usize) -> usize {
        self.rng.randi_range(0, size as i32 - 1) as usize
    }

    fn roll(&mut self, from: i32, to: i32) -> i64 {
        i64::from(self.rng.randi_range(from, to))
    }

    fn random_dir(&mut self) -> Value {
        let i = self.pick(DIRS.len());
        dir_value(DIRS[i])
    }

    fn random_member(&mut self) -> String {
        let i = self.pick(self.gs.party.members.len());
        self.gs.party.members[i].id.clone()
    }

    // ------------------------------------------------------------------ choosing

    fn choose(&mut self) -> Dict {
        if !self.fight.is_empty() {
            let r = self.roll(0, 99);
            let outcome = if r < 60 { "won" } else if r < 80 { "fled" } else { "lost" };
            return act([("op", "after_fight".into()), ("runner", self.fight_runner.clone().into()), ("fight", Value::Object(self.fight.clone())), ("outcome", outcome.into())]);
        }
        if !self.talk_to.is_empty() {
            return act([("op", "talk".into()), ("character", self.talk_to.clone().into())]);
        }
        if self.runner.is_some() {
            return self.choose_dialogue();
        }
        if self.on_world_map {
            return self.choose_map();
        }
        self.choose_explore()
    }

    fn choose_dialogue(&mut self) -> Dict {
        let d = self.d;
        self.dialogue_actions += 1;
        let r = self.runner.as_ref().expect("runner");
        let (closed, shop_qid, board_qid, view) = (r.closed, r.shop_qid.clone(), r.board_qid.clone(), r.view(d, &self.gs));
        if closed || self.dialogue_actions > 14 {
            return act([("op", "leave".into())]);
        }
        if !shop_qid.is_empty() {
            let k = self.roll(0, 9);
            if k < 4 {
                let offers = shop::offers(d, &self.gs, &shop_qid);
                if !offers.is_empty() {
                    let o = &offers[self.pick(offers.len())];
                    let count = self.roll(1, 2);
                    return act([("op", "buy".into()), ("kind", json::s(o, "kind").into()), ("id", json::s(o, "id").into()), ("count", int(count))]);
                }
            } else if k < 7 {
                let sells = shop::sell_list(d, &self.gs, &shop_qid);
                if !sells.is_empty() {
                    let s = &sells[self.pick(sells.len())];
                    return act([("op", "sell".into()), ("item", json::s(s, "item").into()), ("count", int(1))]);
                }
            }
            return act([("op", "close_shop".into())]);
        }
        if !board_qid.is_empty() {
            let open = board::quests(d, &self.gs, &board_qid);
            if !open.is_empty() && self.roll(0, 9) < 6 {
                let q = open[self.pick(open.len())].clone();
                return act([("op", "accept".into()), ("quest", q.into())]);
            }
            return act([("op", "close_board".into())]);
        }
        if !self.pending.is_empty() {
            if self.roll(0, 9) == 0 {
                return act([("op", "cancel".into())]);
            }
            let offered = json::sget(&self.pending, "offers");
            let allowed = json::i(&self.pending, "picks_allowed", 0);
            let mut picks = Vec::new();
            for id in offered {
                if (picks.len() as i64) < allowed && self.roll(0, 1) == 0 {
                    picks.push(id);
                }
            }
            let recipient = if self.roll(0, 2) == 0 { self.random_member() } else { String::new() };
            return act([("op", "confirm".into()), ("recipient", recipient.into()), ("picks", json::strs(&picks))]);
        }
        let choices = json::aget(&view, "choices").to_vec();
        if json::b(&view, "end", false) || (json::b(&view, "next", false) && (choices.is_empty() || self.roll(0, 1) == 0)) {
            return act([("op", "advance".into())]);
        }
        if choices.is_empty() {
            return act([("op", "leave".into())]);
        }
        let mut open = Vec::new();
        let mut quest_services = Vec::new();
        for (i, c) in choices.iter().enumerate() {
            let c = json::dict(c);
            if !json::b(c, "locked", false) {
                open.push(i as i64);
                if ["turn_in", "offer"].contains(&json::s(c, "service").as_str()) {
                    quest_services.push(i as i64);
                }
            }
        }
        if open.is_empty() || self.roll(0, 9) == 0 {
            let index = self.roll(-1, choices.len() as i32);
            return act([("op", "choose".into()), ("index", int(index))]);
        }
        // Quest services first (turn in, hear offers), and an offer node is mostly accepted: walks reach turn-ins.
        if !quest_services.is_empty() && self.roll(0, 9) < 8 {
            let index = quest_services[self.pick(quest_services.len())];
            return act([("op", "choose".into()), ("index", int(index))]);
        }
        if json::s(&view, "node").starts_with(OFFER_PREFIX) && open.contains(&0) && self.roll(0, 9) < 8 {
            return act([("op", "choose".into()), ("index", int(0))]);
        }
        let index = open[self.pick(open.len())];
        act([("op", "choose".into()), ("index", int(index))])
    }

    fn choose_map(&mut self) -> Dict {
        let d = self.d;
        let node = travel::current_node(d, &self.gs);
        if node.is_empty() {
            return act([("op", "close_map".into())]);
        }
        let r = self.roll(0, 9);
        if r < 6 {
            let known: Vec<String> = travel::known(d, &self.gs).into_iter().filter(|n| *n != node).collect();
            if !known.is_empty() {
                let to = known[self.pick(known.len())].clone();
                return act([("op", "travel".into()), ("to", to.into())]);
            }
        }
        if r < 8 && progression::can_camp(d, &self.gs).is_empty() {
            return act([("op", "camp".into())]);
        }
        act([("op", "enter".into()), ("to", node.into())])
    }

    fn choose_explore(&mut self) -> Dict {
        let d = self.d;
        let r = self.roll(0, 99);
        if r < 3 {
            let member = self.random_member();
            let hp_lost = self.roll(0, 40);
            return act([("op", "hurt".into()), ("member", member.into()), ("hp_lost", int(hp_lost))]);
        }
        if r < 6 {
            let i = self.pick(self.gs.party.members.len());
            let member = self.gs.party.members[i].id.clone();
            let points = self.gs.party.members[i].unspent(d).max(0) + self.roll(0, 1);
            let mut draft: BTreeMap<String, i64> = BTreeMap::new();
            for _ in 0..points {
                let stat = STATS[self.pick(STATS.len())];
                *draft.entry(stat.into()).or_default() += 1;
            }
            let draft: Dict = draft.into_iter().map(|(k, v)| (k, int(v))).collect();
            return act([("op", "level_up".into()), ("member", member.into()), ("draft", Value::Object(draft))]);
        }
        if r < 7 {
            let active = quest::quests_with(d, &self.gs, "active");
            if !active.is_empty() {
                let q = active[self.pick(active.len())].clone();
                return act([("op", "abandon".into()), ("quest", q.into())]);
            }
        }
        if r < 9 {
            let ready = quest::quests_with(d, &self.gs, "ready");
            if !ready.is_empty() {
                let q = ready[self.pick(ready.len())].clone();
                let allowed = quest::picks_allowed(d, &self.gs, &q).max(0) as usize;
                let picks: Vec<String> = quest::offers(d, &self.gs, &q).into_iter().take(allowed).collect();
                return act([("op", "journal".into()), ("quest", q.into()), ("picks", json::strs(&picks))]);
            }
        }
        if r < 10 {
            return act([("op", "reload".into())]);
        }
        let loc = explore::location(d, &self.gs);
        if json::s(loc, "map").is_empty() {
            let menu = explore::menu(d, &self.gs);
            if menu.is_empty() {
                return act([("op", "begin".into())]);
            }
            let to = json::s(&menu[self.pick(menu.len())], "to");
            return act([("op", "exit".into()), ("to", to.into())]);
        }
        if self.roll(0, 99) < 12 {
            let dir = self.random_dir();
            return act([("op", "step".into()), ("dir", dir)]);
        }
        self.toward_goal(loc)
    }

    /// A present marker is picked (weighted by kind); the party walks there by BFS over open, unblocked tiles, then
    /// faces and interacts (npc, chest, lore) or steps onto it (exit, encounter, point).
    fn toward_goal(&mut self, loc: &Dict) -> Dict {
        let d = self.d;
        let t = explore::template_of(d, loc);
        let loc_qid = json::s(loc, "_qid");
        let loc_steps = json::i(&self.gs.explore, "loc_steps", 0);
        let markers = explore::markers_of(loc);
        let weight = |m: &Dict| GOAL_WEIGHTS.iter().find(|(k, _)| *k == json::s(m, "kind")).map_or(1, |(_, w)| *w);
        let mut target = None;
        if self.goal_loc == loc_qid {
            target = markers.iter().rfind(|m| json::s(m, "id") == self.goal && explore::presence(d, &self.gs, &loc_qid, m) != "absent").cloned();
        }
        let target = match target {
            Some(t) => t,
            None => {
                let pool: Vec<&Dict> = markers.iter().filter(|m| explore::presence(d, &self.gs, &loc_qid, m) != "absent").collect();
                if pool.is_empty() {
                    let dir = self.random_dir();
                    return act([("op", "step".into()), ("dir", dir)]);
                }
                let total: i64 = pool.iter().map(|m| weight(m)).sum();
                let mut r = self.roll(0, total as i32 - 1);
                let mut picked = pool[0].clone();
                for m in &pool {
                    r -= weight(m);
                    if r < 0 {
                        picked = (*m).clone();
                        break;
                    }
                }
                self.goal = json::s(&picked, "id");
                self.goal_loc = loc_qid.clone();
                picked
            }
        };
        let at = explore::marker_pos(t, &target, loc_steps);
        let solid = SOLID.contains(&json::s(&target, "kind").as_str()) || explore::presence(d, &self.gs, &loc_qid, &target) == "locked";
        if solid {
            let p = self.gs.position;
            if let Some(dir) = DIRS.iter().find(|dd| (p.0 + dd.0, p.1 + dd.1) == at) {
                self.goal.clear();
                return act([("op", "interact".into()), ("dir", dir_value(*dir))]);
            }
        }
        match self.bfs_first(t, &loc_qid, &markers, loc_steps, at, solid) {
            Some(first) => act([("op", "step".into()), ("dir", dir_value(first))]),
            None => {
                self.goal.clear();
                let dir = self.random_dir();
                act([("op", "step".into()), ("dir", dir)])
            }
        }
    }

    /// The first step of a shortest path to `at` (or to a tile next to it when `adjacent`); None when unreachable.
    fn bfs_first(&self, t: &Dict, loc_qid: &str, markers: &[Dict], loc_steps: i64, at: (i64, i64), adjacent: bool) -> Option<(i64, i64)> {
        let (d, gs) = (self.d, &self.gs);
        let mut blocked = BTreeSet::new();
        for m in markers {
            let state = explore::presence(d, gs, loc_qid, m);
            if state == "locked" || (state == "present" && SOLID.contains(&json::s(m, "kind").as_str())) {
                blocked.insert(explore::marker_pos(t, m, loc_steps));
            }
        }
        let start = gs.position;
        let mut first_of: HashMap<(i64, i64), (i64, i64)> = HashMap::from([(start, (0, 0))]);
        let mut queue = VecDeque::from([start]);
        while let Some(cur) = queue.pop_front() {
            let reached = if adjacent { (cur.0 - at.0).abs() + (cur.1 - at.1).abs() == 1 } else { cur == at };
            if reached && cur != start {
                return Some(first_of[&cur]);
            }
            for dd in DIRS {
                let nxt = (cur.0 + dd.0, cur.1 + dd.1);
                if first_of.contains_key(&nxt) || !explore::is_open(t, nxt.0, nxt.1) {
                    continue;
                }
                if blocked.contains(&nxt) && (adjacent || nxt != at) {
                    continue;
                }
                let f = if cur == start { dd } else { first_of[&cur] };
                first_of.insert(nxt, f);
                queue.push_back(nxt);
            }
        }
        None
    }

    // ------------------------------------------------------------------ running

    fn scan_events(&mut self, res: &Dict) {
        for e in json::aget(res, "events") {
            let e = json::dict(e);
            match json::s(e, "kind").as_str() {
                "fight" => {
                    self.fight = e.clone();
                    self.fight_runner = "explore".into();
                }
                "exit" if json::s(e, "to") == explore::WORLD_MAP => self.on_world_map = true,
                _ => {}
            }
        }
    }

    fn runner(&mut self) -> &mut DialogueRunner {
        self.runner.as_mut().expect("runner")
    }

    fn run(&mut self, a: &Dict) -> Dict {
        let d = self.d;
        let s = |k: &str| json::s(a, k);
        match s("op").as_str() {
            "begin" => explore::begin(d, &mut self.gs),
            "step" => {
                let res = explore::step(d, &mut self.gs, dir_of(a));
                self.scan_events(&res);
                res
            }
            "interact" => {
                let res = explore::interact(d, &mut self.gs, dir_of(a));
                if json::s(&res, "kind") == "npc" {
                    self.talk_to = json::s(&res, "character");
                }
                res
            }
            "exit" => {
                let res = explore::take_exit(d, &mut self.gs, &s("to"), explore::SPAWN);
                self.scan_events(&res);
                res
            }
            "talk" => {
                self.talk_to.clear();
                self.dialogue_actions = 0;
                let r = DialogueRunner::open(d, &mut self.gs, &s("character"));
                let res = json::obj([("errors", json::strs(&r.errors)), ("closed", Value::Bool(r.closed))]);
                self.runner = if r.closed { None } else { Some(r) };
                res
            }
            "choose" => {
                let index = json::i(a, "index", 0);
                let mut r = self.runner.take().expect("runner");
                let res = r.choose(d, &mut self.gs, index);
                if res.contains_key("fight") {
                    self.fight = json::dget(&res, "fight").clone();
                    self.fight_runner = "dialogue".into();
                } else {
                    if res.contains_key("pending") {
                        self.pending = res.clone();
                    }
                    self.runner = Some(r);
                }
                res
            }
            "advance" => {
                let mut r = self.runner.take().expect("runner");
                let res = r.advance(d, &mut self.gs);
                self.runner = Some(r);
                res
            }
            "confirm" => {
                let mut r = self.runner.take().expect("runner");
                let res = r.confirm_turn_in(d, &mut self.gs, &s("recipient"), &json::sget(a, "picks"));
                self.runner = Some(r);
                if json::b(&res, "ok", false) {
                    self.pending.clear();
                }
                res
            }
            "cancel" => {
                self.pending.clear();
                self.runner().cancel()
            }
            "buy" => {
                let sq = self.runner().shop_qid.clone();
                shop::buy(d, &mut self.gs, &sq, &s("kind"), &s("id"), json::i(a, "count", 1))
            }
            "sell" => {
                let sq = self.runner().shop_qid.clone();
                shop::sell(d, &mut self.gs, &sq, &s("item"), json::i(a, "count", 1))
            }
            "close_shop" => {
                let mut r = self.runner.take().expect("runner");
                let res = r.close_shop(d, &mut self.gs);
                self.runner = Some(r);
                res
            }
            "accept" => {
                let mut r = self.runner.take().expect("runner");
                let res = r.accept(d, &mut self.gs, &s("quest"));
                self.runner = Some(r);
                res
            }
            "close_board" => {
                let mut r = self.runner.take().expect("runner");
                let res = r.close_board(d, &mut self.gs);
                self.runner = Some(r);
                res
            }
            "leave" => {
                self.runner = None;
                self.pending.clear();
                Dict::new()
            }
            "after_fight" => {
                let f = json::dget(a, "fight");
                let outcome = s("outcome");
                // no tally: the default one (REWARD1 section 9)
                let res = match s("runner").as_str() {
                    "explore" => explore::after_fight(d, &mut self.gs, f, &outcome, None),
                    "travel" => travel::after_fight(d, &mut self.gs, f, &outcome, None),
                    _ => dialogue::after_fight(d, &mut self.gs, f, &outcome, None),
                };
                self.fight.clear();
                res
            }
            "travel" => {
                let res = travel::travel(d, &mut self.gs, &s("to"));
                if json::s(&res, "status") == "fight" {
                    self.fight = json::dget(&res, "fight").clone();
                    self.fight_runner = "travel".into();
                }
                res
            }
            "camp" => progression::make_camp(d, &mut self.gs),
            "enter" => {
                self.on_world_map = false;
                explore::enter(d, &mut self.gs, &s("to"), explore::SPAWN)
            }
            "close_map" => {
                self.on_world_map = false;
                Dict::new()
            }
            "hurt" => {
                if let Some(m) = self.gs.party.member_mut(&s("member")) {
                    m.hp_lost = json::i(a, "hp_lost", 0);
                }
                Dict::new()
            }
            "level_up" => {
                let id = s("member");
                let idx = self.gs.party.members.iter().position(|m| m.id == id).expect("member");
                let errors = progression::confirm(d, &mut self.gs.party.members[idx], json::dget(a, "draft"));
                json::obj([("errors", json::strs(&errors))])
            }
            "abandon" => json::obj([("errors", json::strs(&quest::abandon(d, &mut self.gs, &s("quest"))))]),
            "journal" => dialogue::complete_from_journal(d, &mut self.gs, &s("quest"), "", &json::sget(a, "picks")),
            "reload" => self.save_round_trip(),
            other => panic!("unknown walker op {other}"),
        }
    }

    /// Saves to slot 1 and loads it back (the real save files; a fixed save time); then resumes the exploration.
    fn save_round_trip(&mut self) -> Dict {
        let d = self.d;
        let refused = save::write(d, &self.save_dir, &self.gs, 1, 1);
        if !refused.is_empty() {
            return json::obj([("save_errors", json::strs(&refused))]);
        }
        match save::read(d, &self.save_dir, 1) {
            Ok(loaded) => {
                self.gs = loaded.state;
                let mut res = explore::resume(d, &mut self.gs);
                res.insert("loaded".into(), Value::Bool(true));
                res.insert("warnings".into(), json::strs(&loaded.warnings));
                res.insert("valid".into(), json::strs(&self.gs.validate(d)));
                res
            }
            Err(errors) => json::obj([("load_errors", json::strs(&errors))]),
        }
    }

    /// What the screens read after an action (all read-only).
    fn views(&self) -> Dict {
        let (d, gs) = (self.d, &self.gs);
        let mut v = Dict::new();
        let mut quests = Dict::new();
        for e in d.merged_entries(&gs.world_id, "quests") {
            let qid = json::s(e, "_qid");
            quests.insert(qid.clone(), quest::status(d, gs, &qid).into());
        }
        v.insert("quests".into(), Value::Object(quests));
        let objectives: Vec<Value> = quest::quests_with(d, gs, "active").iter().map(|q| Value::Object(quest::current_objective(d, gs, q))).collect();
        v.insert("objectives".into(), Value::Array(objectives));
        let mut turn_ins = Dict::new();
        for qid in quest::quests_with(d, gs, "ready") {
            let o = json::obj([("offers", json::strs(&quest::offers(d, gs, &qid))), ("picks", int(quest::picks_allowed(d, gs, &qid)))]);
            turn_ins.insert(qid, Value::Object(o));
        }
        v.insert("turn_ins".into(), Value::Object(turn_ins));
        let hp: Vec<Value> = gs
            .party
            .members
            .iter()
            .map(|m| {
                let (h, max) = progression::current_hp(d, m, &gs.world_id);
                json!({"hp": h, "max": max})
            })
            .collect();
        v.insert("hp".into(), Value::Array(hp));
        v.insert("camp".into(), progression::can_camp(d, gs).into());
        v.insert("menu".into(), Value::Array(explore::menu(d, gs).into_iter().map(Value::Object).collect()));
        let loc = explore::location(d, gs);
        let markers: Vec<Value> = json::aget(loc, "markers").iter().map(|m| Value::Object(explore::marker_view(d, gs, loc, json::dict(m)))).collect();
        v.insert("markers".into(), Value::Array(markers));
        if self.on_world_map {
            v.insert("map".into(), Value::Object(travel::view(d, gs)));
        }
        if let Some(r) = &self.runner {
            v.insert("dialogue".into(), Value::Object(r.view(d, gs)));
            if !r.shop_qid.is_empty() {
                v.insert("shop_offers".into(), Value::Array(shop::offers(d, gs, &r.shop_qid).into_iter().map(Value::Object).collect()));
                v.insert("shop_sell".into(), Value::Array(shop::sell_list(d, gs, &r.shop_qid).into_iter().map(Value::Object).collect()));
            }
            if !r.board_qid.is_empty() {
                v.insert("board".into(), Value::Object(board::view(d, gs, &r.board_qid)));
            }
        }
        v
    }

    /// One trace line: the step, the action, its result and the digests of the state and the views after it.
    fn line(&self, i: usize, a: &Dict, res: Dict) -> String {
        let state = self.gs.to_value();
        let mut l = json!({"i": i, "action": a, "result": res, "state": digest(&state), "views": digest(&Value::Object(self.views()))});
        if i % FULL_EVERY == 0 {
            l["full_state"] = state;
        }
        json::sorted(&l).to_string()
    }
}

/// One walk's trace: a start line (the new game's state) and a line per action.
fn walk(d: &Data, world: &str, seed: i64) -> Vec<String> {
    let gs = new_game::create_for_world(d, world, seed).unwrap_or_else(|e| panic!("{world}: new game refused: {e:?}"));
    let save_dir = aegis_core::scratch::Scratch::new(&format!("emberwake_journey_{world}_{seed}"));
    let rng = GodotRng::from_i64(seed * 7919 + i64::from(string_hash(world)));
    let mut w = Walker {
        d,
        gs,
        rng,
        runner: None,
        talk_to: String::new(),
        pending: Dict::new(),
        fight: Dict::new(),
        fight_runner: String::new(),
        on_world_map: false,
        dialogue_actions: 0,
        goal: String::new(),
        goal_loc: String::new(),
        save_dir: save_dir.to_path_buf(),
    };
    let mut lines = vec![json::sorted(&json!({"start": w.gs.to_value()})).to_string()];
    let first = act([("op", "begin".into())]);
    let res = w.run(&first);
    lines.push(w.line(0, &first, res));
    for i in 1..=STEPS {
        let a = w.choose();
        let res = w.run(&a);
        lines.push(w.line(i, &a, res));
    }
    let _ = std::fs::remove_dir_all(&save_dir);
    lines
}

/// Every playable world with every seed, in that order (the walks run on threads; each is independent).
fn walks(d: &Data) -> Vec<Value> {
    let runs: Vec<(String, i64)> = new_game::playable_worlds(d).into_iter().flat_map(|w| SEEDS.map(|s| (w.clone(), s))).collect();
    std::thread::scope(|scope| {
        let handles: Vec<_> = runs.iter().map(|(world, seed)| scope.spawn(move || json!({"world": world, "seed": seed, "lines": walk(d, world, *seed)}))).collect();
        handles.into_iter().map(|h| h.join().expect("a walk panicked")).collect()
    })
}

fn golden() -> Value {
    read_json(&golden_path(), "the journey golden")
}

#[test]
fn journeys_match_the_golden() {
    let d = frozen_data();
    if recording() {
        let doc = json!({"steps": STEPS, "seeds": SEEDS, "walks": walks(&d)});
        std::fs::create_dir_all(golden_path().parent().unwrap()).unwrap();
        std::fs::write(golden_path(), serde_json::to_string_pretty(&doc).unwrap() + "\n").unwrap();
        eprintln!("recorded {} walks to {}", doc["walks"].as_array().map_or(0, Vec::len), golden_path().display());
        return;
    }
    let g = golden();
    let want = g["walks"].as_array().expect("walks");
    let got = walks(&d);
    let label = |w: &Value| format!("{} seed {}", w["world"].as_str().unwrap_or(""), w["seed"]);
    let want_labels: Vec<String> = want.iter().map(label).collect();
    let got_labels: Vec<String> = got.iter().map(label).collect();
    assert_eq!(got_labels, want_labels, "the walks (playable worlds x SEEDS) differ from the golden's (re-record after an intended change)");
    let mut failures = Vec::new();
    for (w, g) in want.iter().zip(&got) {
        let a: Vec<&str> = w["lines"].as_array().expect("lines").iter().map(|l| l.as_str().unwrap_or("")).collect();
        let b: Vec<&str> = g["lines"].as_array().expect("lines").iter().map(|l| l.as_str().unwrap_or("")).collect();
        match a.iter().zip(&b).position(|(x, y)| x != y) {
            Some(k) => failures.push(format!("{} line {k}:\n  golden {}\n  now    {}", label(w), short(a[k]), short(b[k]))),
            None if a.len() != b.len() => failures.push(format!("{}: golden {} lines, now {}", label(w), a.len(), b.len())),
            None => {}
        }
    }
    assert!(failures.is_empty(), "{} of the {} walks differ from the golden:\n{}", failures.len(), want.len(), failures.join("\n"));
}

fn short(s: &str) -> String {
    if s.chars().count() > 600 {
        format!("{}...", s.chars().take(600).collect::<String>())
    } else {
        s.to_string()
    }
}

/// The recorded walks touch every runner and screen action (explore, dialogue, shop, quest journal, world map, rest,
/// level-up), fights end through the explore and travel runners, exits change location, and the save round trip
/// loads a valid state. A dialogue-started fight needs a deep quest state no walk reaches (the Godot walks never did
/// either); `dialogue::after_fight` is covered by `rewards.rs`.
#[test]
fn the_golden_covers_the_game_loop() {
    let g = golden();
    let mut ops: BTreeSet<String> = BTreeSet::new();
    let mut fights: BTreeSet<String> = BTreeSet::new();
    let mut events: BTreeSet<String> = BTreeSet::new();
    let mut loads = 0;
    for w in g["walks"].as_array().expect("walks") {
        for l in w["lines"].as_array().expect("lines") {
            let l: Value = serde_json::from_str(l.as_str().unwrap_or("")).expect("a line is JSON");
            let Some(op) = l["action"]["op"].as_str() else { continue };
            ops.insert(op.into());
            for e in l["result"]["events"].as_array().into_iter().flatten() {
                events.insert(e["kind"].as_str().unwrap_or("").into());
            }
            if op == "after_fight" {
                fights.insert(l["action"]["runner"].as_str().unwrap_or("").into());
            }
            if op == "reload" && l["result"]["loaded"] == Value::Bool(true) && l["result"]["valid"].as_array().is_some_and(Vec::is_empty) {
                loads += 1;
            }
        }
    }
    let wanted = [
        "begin", "step", "interact", "talk", "choose", "advance", "confirm", "buy", "sell", "close_shop", "leave", "after_fight", "travel",
        "camp", "enter", "hurt", "level_up", "abandon", "journal", "reload",
    ];
    let missing: Vec<&str> = wanted.iter().copied().filter(|o| !ops.contains(*o)).collect();
    assert!(missing.is_empty(), "the walks never run {missing:?} (ran {ops:?})");
    for runner in ["explore", "travel"] {
        assert!(fights.contains(runner), "no fight ends through the {runner} runner (ended through {fights:?})");
    }
    for kind in ["exit", "entered", "fight", "defeat", "camped"] {
        assert!(events.contains(kind), "no step result has a {kind} event (had {events:?})");
    }
    assert!(loads > 0, "no save round trip loaded a valid state");
}
