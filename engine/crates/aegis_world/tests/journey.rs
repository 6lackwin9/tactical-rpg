//! Replays the Godot journeys (engine/parity/journey.sh -> parity/out/journeys.json): every recorded action runs
//! through the Rust game loop, and its result, the game state after it and the screen views must equal Godot's
//! (numbers compared as numbers, object key order ignored). The first difference of a journey is reported with
//! its path; later steps of that journey are skipped (the states have diverged).

use aegis_world::dialogue::{self, DialogueRunner};
use aegis_world::json::{self, int, Dict};
use aegis_world::{explore, new_game, progression, quest, shop, travel, Data, GameState};
use serde_json::Value;
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../parity/out")
}

/// The first path where `a` (Rust) and `b` (Godot) differ.
fn diff(a: &Value, b: &Value, at: &str) -> Option<String> {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            for (k, v) in y {
                match x.get(k) {
                    None => return Some(format!("{at}.{k}: missing in Rust (Godot {})", short(v))),
                    Some(w) => {
                        if let Some(d) = diff(w, v, &format!("{at}.{k}")) {
                            return Some(d);
                        }
                    }
                }
            }
            x.keys().find(|k| !y.contains_key(*k)).map(|k| format!("{at}.{k}: extra in Rust ({})", short(&x[k])))
        }
        (Value::Array(x), Value::Array(y)) => {
            for (i, (p, q)) in x.iter().zip(y).enumerate() {
                if let Some(d) = diff(p, q, &format!("{at}[{i}]")) {
                    return Some(d);
                }
            }
            (x.len() != y.len()).then(|| format!("{at}: length {} vs Godot {} (Rust {} / Godot {})", x.len(), y.len(), short(a), short(b)))
        }
        _ => (!json::deep_equal(a, b)).then(|| format!("{at}: Rust {} vs Godot {}", short(a), short(b))),
    }
}

fn short(v: &Value) -> String {
    let s = v.to_string();
    if s.len() > 300 {
        format!("{}...", &s[..300])
    } else {
        s
    }
}

fn dir_of(act: &Dict) -> (i64, i64) {
    match json::aget(act, "dir") {
        [x, y] => (json::to_int(x), json::to_int(y)),
        _ => (0, 0),
    }
}

struct Journey<'a> {
    d: &'a Data,
    gs: GameState,
    runner: Option<DialogueRunner>,
}

impl Journey<'_> {
    fn run(&mut self, act: &Dict) -> Dict {
        let d = self.d;
        let gs = &mut self.gs;
        let s = |k: &str| json::s(act, k);
        match s("op").as_str() {
            "begin" => explore::begin(d, gs),
            "step" => explore::step(d, gs, dir_of(act)),
            "interact" => explore::interact(d, gs, dir_of(act)),
            "exit" => explore::take_exit(d, gs, &s("to"), explore::SPAWN),
            "talk" => {
                let r = DialogueRunner::open(d, gs, &s("character"));
                let res = json::obj([("errors", json::strs(&r.errors)), ("closed", Value::Bool(r.closed))]);
                self.runner = if r.closed { None } else { Some(r) };
                res
            }
            "choose" => {
                let r = self.runner.as_mut().expect("runner");
                let res = r.choose(d, gs, json::i(act, "index", 0));
                if res.contains_key("fight") {
                    self.runner = None;
                }
                res
            }
            "advance" => self.runner.as_mut().expect("runner").advance(d, gs),
            "confirm" => self.runner.as_mut().expect("runner").confirm_turn_in(d, gs, &s("recipient"), &json::sget(act, "picks")),
            "cancel" => self.runner.as_mut().expect("runner").cancel(),
            "buy" => {
                let sq = self.runner.as_ref().expect("runner").shop_qid.clone();
                shop::buy(d, gs, &sq, &s("kind"), &s("id"), json::i(act, "count", 1))
            }
            "sell" => {
                let sq = self.runner.as_ref().expect("runner").shop_qid.clone();
                shop::sell(d, gs, &sq, &s("item"), json::i(act, "count", 1))
            }
            "close_shop" => self.runner.as_mut().expect("runner").close_shop(d, gs),
            "leave" => {
                self.runner = None;
                Dict::new()
            }
            "after_fight" => {
                let f = json::dget(act, "fight");
                let outcome = s("outcome");
                match s("runner").as_str() {
                    "explore" => explore::after_fight(d, gs, f, &outcome),
                    "travel" => travel::after_fight(d, gs, f, &outcome),
                    _ => dialogue::after_fight(d, gs, f, &outcome),
                }
            }
            "travel" => travel::travel(d, gs, &s("to")),
            "camp" => progression::make_camp(d, gs),
            "enter" => explore::enter(d, gs, &s("to"), explore::SPAWN),
            "hurt" => {
                if let Some(m) = gs.party.member_mut(&s("member")) {
                    m.hp_lost = json::i(act, "hp_lost", 0);
                }
                Dict::new()
            }
            "level_up" => {
                let id = s("member");
                let draft = json::dget(act, "draft").clone();
                let idx = gs.party.members.iter().position(|m| m.id == id).expect("member");
                let errors = progression::confirm(d, &mut gs.party.members[idx], &draft);
                json::obj([("errors", json::strs(&errors))])
            }
            "abandon" => json::obj([("errors", json::strs(&quest::abandon(d, gs, &s("quest"))))]),
            "journal" => dialogue::complete_from_journal(d, gs, &s("quest"), "", &json::sget(act, "picks")),
            "reload" => {
                let text = json::sorted(&gs.to_value()).to_string();
                let parsed: Value = serde_json::from_str(&text).unwrap();
                *gs = GameState::from_dict(d, json::dict(&parsed));
                let mut res = explore::resume(d, gs);
                res.insert("valid".into(), json::strs(&gs.validate(d)));
                res
            }
            _ => Dict::new(),
        }
    }

    fn views(&self, want: &Dict) -> Dict {
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
                let (h, max) = progression::current_hp(d, m);
                Value::Object(json::obj([("hp", int(h)), ("max", int(max))]))
            })
            .collect();
        v.insert("hp".into(), Value::Array(hp));
        v.insert("camp".into(), progression::can_camp(d, gs).into());
        v.insert("menu".into(), Value::Array(explore::menu(d, gs).into_iter().map(Value::Object).collect()));
        let loc = explore::location(d, gs);
        let markers: Vec<Value> = json::aget(loc, "markers").iter().map(|m| Value::Object(explore::marker_view(d, gs, loc, json::dict(m)))).collect();
        v.insert("markers".into(), Value::Array(markers));
        if want.contains_key("map") {
            v.insert("map".into(), Value::Object(travel::view(d, gs)));
        }
        if let Some(r) = &self.runner {
            v.insert("dialogue".into(), Value::Object(r.view(d, gs)));
            if !r.shop_qid.is_empty() {
                v.insert("shop_offers".into(), Value::Array(shop::offers(d, gs, &r.shop_qid).into_iter().map(Value::Object).collect()));
                v.insert("shop_sell".into(), Value::Array(shop::sell_list(d, gs, &r.shop_qid).into_iter().map(Value::Object).collect()));
            }
        }
        v
    }
}

#[test]
fn journeys_match_godot() {
    let dir = out_dir();
    let path = dir.join("journeys.json");
    if !path.exists() {
        eprintln!("no {}: run engine/parity/journey.sh first", path.display());
        return;
    }
    let d = Data::load(&dir).expect("parity exports");
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let mut failures = Vec::new();
    let mut checked = 0;
    for j in json::aget(json::dict(&doc), "journeys") {
        let j = json::dict(j);
        let (world, seed) = (json::s(j, "world"), json::i(j, "seed", 0));
        let label = format!("{world} seed {seed}");
        let gs = match new_game::create_for_world(&d, &world, seed) {
            Ok(gs) => gs,
            Err(e) => {
                failures.push(format!("{label}: new game refused: {e:?}"));
                continue;
            }
        };
        if let Some(df) = diff(&gs.to_value(), json::get(j, "start"), "start") {
            failures.push(format!("{label}: {df}"));
            continue;
        }
        let mut jr = Journey { d: &d, gs, runner: None };
        for (i, rec) in json::aget(j, "records").iter().enumerate() {
            let rec = json::dict(rec);
            let act = json::dget(rec, "action");
            let res = jr.run(act);
            let views = jr.views(json::dget(rec, "views"));
            let found = diff(&Value::Object(res), json::get(rec, "result"), "result")
                .or_else(|| diff(&jr.gs.to_value(), json::get(rec, "state"), "state"))
                .or_else(|| diff(&Value::Object(views), json::get(rec, "views"), "views"));
            checked += 1;
            if let Some(df) = found {
                failures.push(format!("{label} step {i} {}: {df}", short(&Value::Object(act.clone()))));
                break;
            }
        }
    }
    eprintln!("journey actions checked: {checked}");
    assert!(failures.is_empty(), "{} journey(s) diverged:\n{}", failures.len(), failures.join("\n"));
}
