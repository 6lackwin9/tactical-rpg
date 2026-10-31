//! TravelRules (runtime) and TravelRunner (scripts/travel/): the world-map graph, known nodes, best paths, the
//! map view, and auto-pathed trips leg by leg (clock, supplies, scripted and road fights). CAMPAIGN1 (engine-only,
//! inert outside a campaign world): each leg ticks the campaign (days, then leg; K2, M3) and may end the trip (status
//! `over`, K4); upkeep per day and the zero-upkeep and band modifiers of the road-fight chance (P2, P3, M7); the map
//! view's checkpoint lines (STORYUI1 B10). CAMPAIGN1-TM: in a campaign world with travel modes, each leg takes the
//! party's mode's `days` more (`leg_days`: path search, trip cost, the clock, upkeep and the days tick all count them)
//! and its leg tick names the mode (meter inputs `modes`); the mode's road-fight change is `campaign::purse`'s.

use crate::data::Data;
use crate::explore::{self, SPAWN};
use crate::state::{GameState, PartyState};
use crate::{quest, rolls};
use aegis_core::json::{self, int, Dict};
use serde_json::Value;

pub const REPEAT_HALF: i64 = -1;
pub const STARVED_PCT: i64 = 150;
pub const PCT_MAX: i64 = 100;

fn world_of(qid: &str) -> String {
    aegis_content::split_qid(qid).map(|(w, _)| w).unwrap_or_default()
}

/// The routes `w` travels on: its merged routes, then the two-way routes of other worlds that reach it with a
/// visible endpoint. Each is a copy plus `_from` / `_to`.
pub fn graph(d: &Data, w: &str) -> Vec<Dict> {
    let mut out = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let chain = d.lineage(w);
    for r in d.merged_entries(w, "routes") {
        add_route(d, w, r, &mut out, &mut seen, false);
    }
    for x in d.world_ids() {
        if chain.contains(&x) || d.lineage(&x).iter().any(|y| y == w) || !d.can_reach_world(&x, w) {
            continue;
        }
        for r in d.merged_entries(&x, "routes") {
            if json::b(r, "two_way", true) && !chain.contains(&json::s(r, "_world")) {
                add_route(d, w, r, &mut out, &mut seen, true);
            }
        }
    }
    out
}

fn add_route(d: &Data, w: &str, r: &Dict, out: &mut Vec<Dict>, seen: &mut Vec<String>, needs_visible: bool) {
    let rq = json::s(r, "_qid");
    if seen.contains(&rq) {
        return;
    }
    let a = endpoint(d, w, r, "from");
    let b = endpoint(d, w, r, "to");
    if a.is_empty() || b.is_empty() {
        return;
    }
    if needs_visible && !visible(d, w, &a) && !visible(d, w, &b) {
        return;
    }
    seen.push(rq);
    let mut copy = r.clone();
    copy.insert("_from".into(), a.into());
    copy.insert("_to".into(), b.into());
    out.push(copy);
}

pub fn endpoint(d: &Data, w: &str, route: &Dict, key: &str) -> String {
    let home = json::s_or(route, "_world", w);
    let hit = d.resolve(&home, "locations", &json::s(route, key));
    if hit.is_empty() {
        return String::new();
    }
    let q = json::s(hit, "_qid");
    json::s_or(d.resolve(w, "locations", &q), "_qid", &q)
}

fn visible(d: &Data, w: &str, qid: &str) -> bool {
    let lw = world_of(qid);
    !lw.is_empty() && d.can_reach_world(w, &lw) && d.has_entry(w, "locations", qid)
}

pub fn nodes(d: &Data, w: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for r in graph(d, w) {
        for key in ["_from", "_to"] {
            let q = json::s(&r, key);
            if !out.contains(&q) {
                out.push(q);
            }
        }
    }
    out
}

pub fn traversed_flag(route_qid: &str) -> String {
    format!("{}.traversed", explore::dotted(route_qid))
}

pub fn event_done_flag(route_qid: &str, event: &str) -> String {
    format!("{}.{event}.done", explore::dotted(route_qid))
}

pub fn is_open(gs: &GameState, route: &Dict) -> bool {
    quest::all_hold(gs, json::get(route, "requires_flags")) && !quest::any_holds(gs, json::get(route, "unless_flags"))
}

pub fn repeat_chance(route: &Dict) -> i64 {
    let repeat = json::i(route, "repeat_pct", REPEAT_HALF);
    if repeat == REPEAT_HALF {
        (json::i(route, "encounter_pct", 0) as f64 / 2.0).floor() as i64
    } else {
        repeat
    }
}

pub fn leg_chance(gs: &GameState, route: &Dict, starved: bool) -> i64 {
    let mut chance = json::i(route, "encounter_pct", 0);
    if gs.flag_true(&traversed_flag(&json::s(route, "_qid"))) {
        chance = repeat_chance(route);
    }
    if starved {
        starved_chance(chance)
    } else {
        chance
    }
}

pub fn starved_chance(chance: i64) -> i64 {
    PCT_MAX.min(chance.max(0) * STARVED_PCT / 100)
}

pub fn start_supplies(d: &Data, w: &str) -> i64 {
    for lw in d.lineage(w).iter().rev() {
        let v = json::i(d.manifest(lw), "start_supplies", -1);
        if v >= 0 {
            return v;
        }
    }
    crate::state::SUPPLIES_START
}

pub fn is_visited(gs: &GameState, qid: &str) -> bool {
    gs.flag_true(&explore::visited_flag(qid))
}

/// Directed legs over open routes: (route, from, to).
pub fn legs(d: &Data, gs: &GameState, w: &str) -> Vec<(Dict, String, String)> {
    directed(graph(d, w).into_iter().filter(|r| is_open(gs, r)))
}

/// CAMPAIGN1-SIM: every directed leg of `w`'s graph, route gates ignored (the full graph `sim deadlines` plans on).
pub fn all_legs(d: &Data, w: &str) -> Vec<(Dict, String, String)> {
    directed(graph(d, w).into_iter())
}

/// Each route as a leg from `_from` to `_to`, and back when it is two-way: (route, from, to).
fn directed(routes: impl Iterator<Item = Dict>) -> Vec<(Dict, String, String)> {
    let mut out = Vec::new();
    for r in routes {
        let (a, b) = (json::s(&r, "_from"), json::s(&r, "_to"));
        let two = json::b(&r, "two_way", true);
        out.push((r.clone(), a.clone(), b.clone()));
        if two {
            out.push((r, b, a));
        }
    }
    out
}

/// Known nodes (visited, or one open leg from a visited node), node order.
pub fn known(d: &Data, gs: &GameState) -> Vec<String> {
    let all = nodes(d, &gs.world_id);
    let mut hit: Vec<String> = all.iter().filter(|q| is_visited(gs, q)).cloned().collect();
    for (_, from, to) in legs(d, gs, &gs.world_id) {
        if is_visited(gs, &from) && !hit.contains(&to) {
            hit.push(to);
        }
    }
    all.into_iter().filter(|q| hit.contains(q)).collect()
}

fn better(a: &(i64, Vec<String>), b: &(i64, Vec<String>)) -> bool {
    if a.0 != b.0 {
        return a.0 < b.0;
    }
    if a.1.len() != b.1.len() {
        return a.1.len() < b.1.len();
    }
    for (x, y) in a.1.iter().zip(&b.1) {
        if x != y {
            return x < y;
        }
    }
    false
}

/// CAMPAIGN1-TM: the days the party's travel mode adds to every leg (0 outside a campaign world and without modes).
pub fn mode_days(d: &Data, gs: &GameState) -> i64 {
    crate::campaign::purse::travel_mode(d, gs).map_or(0, |m| (m.days as i64).max(0))
}

/// The days a leg on `route` takes: its `time` plus `extra` (the travel mode's `mode_days`).
pub fn leg_days(route: &Dict, extra: i64) -> i64 {
    json::i(route, "time", 0) + extra
}

/// Route qids of the best path over open legs between known nodes, in the party's travel mode ([] when equal, unknown
/// or unreachable).
pub fn path(d: &Data, gs: &GameState, from: &str, to: &str) -> Vec<String> {
    let known_nodes = known(d, gs);
    if from == to || !known_nodes.iter().any(|q| q == from) || !known_nodes.iter().any(|q| q == to) {
        return Vec::new();
    }
    best_path_with(&legs(d, gs, &gs.world_id), &known_nodes, from, to, mode_days(d, gs)).map(|b| b.1).unwrap_or_default()
}

/// The best path from `from` to `to` over `all` (directed legs, as `legs` returns them) through `nodes` only: the
/// fewest days (route `time` summed), then the fewest legs, then the smaller route qids. (days, route qids); None when
/// `to` cannot be reached. `path` plans on the open legs between known nodes, `sim deadlines` on the full graph.
pub fn best_path(all: &[(Dict, String, String)], nodes: &[String], from: &str, to: &str) -> Option<(i64, Vec<String>)> {
    best_path_with(all, nodes, from, to, 0)
}

/// `best_path` with `extra` days on every leg (CAMPAIGN1-TM: a travel mode's `days`), so the path and its days are the
/// ones a trip in that mode takes.
pub fn best_path_with(all: &[(Dict, String, String)], nodes: &[String], from: &str, to: &str, extra: i64) -> Option<(i64, Vec<String>)> {
    let mut best: indexmap::IndexMap<String, (i64, Vec<String>)> = indexmap::IndexMap::new();
    best.insert(from.into(), (0, Vec::new()));
    let mut done: Vec<String> = Vec::new();
    loop {
        let mut cur: Option<String> = None;
        for (q, label) in &best {
            if !done.contains(q) && cur.as_ref().map_or(true, |c| better(label, &best[c])) {
                cur = Some(q.clone());
            }
        }
        let Some(cur) = cur else { break };
        if cur == to {
            break;
        }
        done.push(cur.clone());
        let label = best[&cur].clone();
        for (route, lf, lt) in all {
            if *lf != cur || done.contains(lt) || !nodes.contains(lt) {
                continue;
            }
            let mut routes = label.1.clone();
            routes.push(json::s(route, "_qid"));
            let cand = (label.0 + leg_days(route, extra), routes);
            if best.get(lt).map_or(true, |b| better(&cand, b)) {
                best.insert(lt.clone(), cand);
            }
        }
    }
    best.get(to).cloned()
}

pub fn route_in(d: &Data, w: &str, route_qid: &str) -> Dict {
    graph(d, w).into_iter().find(|r| json::s(r, "_qid") == route_qid).unwrap_or_default()
}

fn quest_targets(d: &Data, gs: &GameState) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for qid in quest::quests_with(d, gs, "active") {
        let obj = quest::current_objective(d, gs, &qid);
        if obj.is_empty() {
            continue;
        }
        let q = quest::resolved(d, gs, &qid);
        let steps = json::aget(&q, "steps");
        let index = json::i(&obj, "index", -1);
        let step = if index >= 0 && (index as usize) < steps.len() { json::dict(&steps[index as usize]) } else { json::empty() };
        let mut refs = Vec::new();
        match json::s(&obj, "kind").as_str() {
            "reach_location" => refs.push(json::s(step, "location")),
            "win_encounter" => {
                refs.push(json::s(step, "location"));
                refs.push(json::s(d.resolve(&gs.world_id, "encounters", &json::s(step, "encounter")), "location"));
            }
            _ => {}
        }
        for r in refs {
            if r.is_empty() {
                continue;
            }
            let hit = json::s(explore::resolve_location(d, gs, &r), "_qid");
            if !hit.is_empty() {
                out.push((hit, json::s(&obj, "text")));
            }
        }
    }
    out
}

pub fn quest_nodes(d: &Data, gs: &GameState) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (node, _) in quest_targets(d, gs) {
        if !out.contains(&node) {
            out.push(node);
        }
    }
    out
}

pub fn quest_steps_at(d: &Data, gs: &GameState, node: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (n, text) in quest_targets(d, gs) {
        if n == node && !text.is_empty() && !out.contains(&text) {
            out.push(text);
        }
    }
    out
}

/// {ok, time, supplies} of the best known trip.
pub fn trip(d: &Data, gs: &GameState, from: &str, to: &str) -> Dict {
    let routes = path(d, gs, from, to);
    if routes.is_empty() {
        return json::obj([("ok", Value::Bool(false)), ("time", int(0)), ("supplies", int(0))]);
    }
    let (mut time, mut spent, mut have) = (0, 0, gs.party.supplies);
    let extra = mode_days(d, gs);
    for rq in routes {
        let leg = leg_days(&route_in(d, &gs.world_id, &rq), extra);
        let paid = PartyState::supplies_cost(have, crate::campaign::purse::leg_upkeep(d, &gs.world_id, leg));
        have -= paid;
        spent += paid;
        time += leg;
    }
    json::obj([("ok", Value::Bool(true)), ("time", int(time)), ("supplies", int(spent))])
}

/// {current, supplies, nodes: [{qid, name, visited, kind, region, quest, map_pos, jobs?, deadlines?}], routes: [{qid,
/// from, to, two_way, open, time, days?}], mode?}. STORYUI1 B10: `jobs` (the names of the available quests of boards
/// opened once whose node this is) and `deadlines` (the checkpoint lines of the quests whose node this is, CAMPAIGN1)
/// only when there are some. CAMPAIGN1-TM: in a world with travel modes, `mode` (the party's mode id) and each
/// route's `days` in that mode.
pub fn view(d: &Data, gs: &GameState) -> Dict {
    let known_nodes = known(d, gs);
    let quest_at = quest_nodes(d, gs);
    let jobs = crate::board::jobs(d, gs);
    let deadlines = crate::board::deadlines(d, gs);
    let mode = crate::campaign::purse::travel_mode(d, gs);
    let extra = mode.as_ref().map_or(0, |m| (m.days as i64).max(0));
    let mut node_list = Vec::new();
    for q in &known_nodes {
        let loc = explore::resolve_location(d, gs, q);
        let region_ref = json::s(loc, "region");
        let region = if region_ref.is_empty() { json::empty() } else { d.resolve(&json::s_or(loc, "_world", &gs.world_id), "regions", &region_ref) };
        let mut node = json::obj([
            ("qid", q.clone().into()),
            ("name", json::s_or(loc, "name", q).into()),
            ("visited", Value::Bool(is_visited(gs, q))),
            ("kind", json::s(loc, "kind").into()),
            ("region", json::s(region, "name").into()),
            ("quest", Value::Bool(quest_at.contains(q))),
            ("map_pos", Value::Array(json::aget(loc, "map_pos").to_vec())),
        ]);
        let here: Vec<String> = jobs.iter().filter(|(n, _)| n == q).map(|(_, name)| name.clone()).collect();
        if !here.is_empty() {
            node.insert("jobs".into(), json::strs(&here));
        }
        let due: Vec<Value> = deadlines.iter().filter(|(n, _)| n == q).map(|(_, line)| line.clone()).collect();
        if !due.is_empty() {
            node.insert("deadlines".into(), Value::Array(due));
        }
        node_list.push(Value::Object(node));
    }
    let mut route_list = Vec::new();
    for r in graph(d, &gs.world_id) {
        let (a, b) = (json::s(&r, "_from"), json::s(&r, "_to"));
        if !known_nodes.contains(&a) || !known_nodes.contains(&b) {
            continue;
        }
        let mut entry = json::obj([
            ("qid", json::s(&r, "_qid").into()),
            ("from", a.into()),
            ("to", b.into()),
            ("two_way", Value::Bool(json::b(&r, "two_way", true))),
            ("open", Value::Bool(is_open(gs, &r))),
            ("time", int(json::i(&r, "time", 0))),
        ]);
        if mode.is_some() {
            entry.insert("days".into(), int(leg_days(&r, extra)));
        }
        route_list.push(Value::Object(entry));
    }
    let mut out = json::obj([
        ("current", gs.location_id.clone().into()),
        ("supplies", int(gs.party.supplies)),
        ("nodes", Value::Array(node_list)),
        ("routes", Value::Array(route_list)),
    ]);
    if let Some(m) = mode {
        out.insert("mode".into(), m.id.into());
    }
    out
}

// ---- TravelRunner --------------------------------------------------------------------------------------------

pub fn current_node(d: &Data, gs: &GameState) -> String {
    if nodes(d, &gs.world_id).contains(&gs.location_id) {
        gs.location_id.clone()
    } else {
        String::new()
    }
}

fn trip_result(ok: bool, errors: &[String], status: &str, legs_done: i64, time: i64, events: Vec<Value>, spent: i64, starved: i64) -> Dict {
    let mut out = json::ok_result(ok, errors);
    out.insert("status".into(), status.into());
    out.insert("legs_done".into(), int(legs_done));
    out.insert("time".into(), int(time));
    out.insert("events".into(), Value::Array(events));
    out.insert("supplies_spent".into(), int(spent));
    out.insert("starved_legs".into(), int(starved));
    out
}

/// Travels towards `to_ref` leg by leg; stops at the first fight. {ok, status: fight | arrived | over, ...,
/// campaign?}. CAMPAIGN1: after each leg's days (`leg_days`: its time plus the travel mode's days, CAMPAIGN1-TM) the
/// campaign ticks days(days, travel), then leg(route tags, mode); a tick
/// that ends the run stops the trip before the leg's road roll (status `over`, no autosave), and so does an arrival's
/// enter tick (CAMPAIGN1-T: enter triggers) at that node; `campaign` holds the ticks' events when there are some.
pub fn travel(d: &Data, gs: &mut GameState, to_ref: &str) -> Dict {
    let over = crate::campaign::save_refusal(gs);
    if !over.is_empty() {
        return json::fail(over);
    }
    let start = current_node(d, gs);
    if start.is_empty() {
        return json::fail("The party is not at a world map node.");
    }
    let target = explore::resolve_location(d, gs, to_ref);
    if target.is_empty() {
        return json::fail(&format!("Unknown location '{to_ref}'."));
    }
    let to_qid = json::s(target, "_qid");
    let route_qids = path(d, gs, &start, &to_qid);
    if route_qids.is_empty() {
        return json::fail(&format!("No known way from '{start}' to '{to_qid}'."));
    }
    let mut plan = Vec::new();
    let mut at = start;
    for rq in &route_qids {
        let route = route_in(d, &gs.world_id, rq);
        let leg_to = if json::s(&route, "_from") == at { json::s(&route, "_to") } else { json::s(&route, "_from") };
        plan.push((route, at.clone(), leg_to.clone()));
        at = leg_to;
    }
    let mut events = Vec::new();
    let mut ticked: Vec<Value> = Vec::new();
    let (mut spent, mut supplies_spent, mut starved_legs, mut legs_done) = (0, 0, 0, 0);
    // CAMPAIGN1-TM: the mode is read once: a trip runs in the mode it started in
    let mode = crate::campaign::purse::travel_mode(d, gs);
    let extra = mode.as_ref().map_or(0, |m| (m.days as i64).max(0));
    let mode_id = mode.map(|m| m.id).unwrap_or_default();
    for (route, from, to) in plan {
        let leg_time = leg_days(&route, extra);
        let starved = gs.party.supplies == 0;
        gs.time += leg_time;
        spent += leg_time;
        supplies_spent += gs.party.spend_supplies(crate::campaign::purse::leg_upkeep(d, &gs.world_id, leg_time));
        if starved {
            starved_legs += 1;
        }
        if gs.campaign.is_some() {
            ticked.extend(crate::campaign::tick(d, gs, &crate::campaign::Cause::Days { n: leg_time, why: "travel".into() }));
            ticked.extend(crate::campaign::tick(d, gs, &crate::campaign::Cause::Leg { route_tags: json::sget(&route, "tags"), mode: mode_id.clone() }));
            if !crate::campaign::save_refusal(gs).is_empty() {
                let mut out = trip_result(true, &[], "over", legs_done, spent, events, supplies_spent, starved_legs);
                crate::campaign::merge_events(&mut out, ticked);
                return out;
            }
        }
        let fight = leg_fight(d, gs, &route, &from, &to, starved);
        if !fight.is_empty() {
            let mut out = json::ok_result(true, &[]);
            out.insert("status".into(), "fight".into());
            out.insert("fight".into(), Value::Object(fight));
            out.insert("legs_done".into(), int(legs_done));
            out.insert("time".into(), int(spent));
            out.insert("events".into(), Value::Array(events));
            out.insert("supplies_spent".into(), int(supplies_spent));
            out.insert("starved_legs".into(), int(starved_legs));
            crate::campaign::merge_events(&mut out, ticked);
            return out;
        }
        let entered = complete_leg(d, gs, &json::s(&route, "_qid"), &to);
        if !json::b(&entered, "ok", false) {
            let mut out = trip_result(false, &json::sget(&entered, "errors"), "stopped", legs_done, spent, events, supplies_spent, starved_legs);
            crate::campaign::merge_events(&mut out, ticked);
            return out;
        }
        events.extend(json::aget(&entered, "events").iter().filter(|e| json::s(json::dict(e), "kind") != "autosave").cloned());
        legs_done += 1;
        // CAMPAIGN1 T1: the arrival's enter tick; one that ends the run stops the trip at this node
        ticked.extend(json::aget(&entered, "campaign").iter().cloned());
        if !crate::campaign::save_refusal(gs).is_empty() {
            let mut out = trip_result(true, &[], "over", legs_done, spent, events, supplies_spent, starved_legs);
            crate::campaign::merge_events(&mut out, ticked);
            return out;
        }
    }
    events.push(Value::Object(json::obj([("kind", "autosave".into())])));
    let mut out = json::ok_result(true, &[]);
    out.insert("status".into(), "arrived".into());
    out.insert("location".into(), gs.location_id.clone().into());
    out.insert("legs_done".into(), int(legs_done));
    out.insert("time".into(), int(spent));
    out.insert("events".into(), Value::Array(events));
    out.insert("supplies_spent".into(), int(supplies_spent));
    out.insert("starved_legs".into(), int(starved_legs));
    crate::campaign::merge_events(&mut out, ticked);
    out
}

fn leg_fight(d: &Data, gs: &mut GameState, route: &Dict, from: &str, to: &str, starved: bool) -> Dict {
    let rq = json::s(route, "_qid");
    for x in json::aget(route, "scripted") {
        let s = json::dict(&json::normalize(x)).clone();
        let sid = json::s(&s, "id");
        if gs.flag_true(&event_done_flag(&rq, &sid)) {
            continue;
        }
        if quest::all_hold(gs, json::get(&s, "requires_flags")) && !quest::any_holds(gs, json::get(&s, "unless_flags")) {
            return fight(d, gs, route, "scripted", &json::s(&s, "encounter"), &sid, false, from, to);
        }
    }
    // CAMPAIGN1 M7, P3: the band modifiers' encounters join the normal pool, the zero-upkeep and band modifiers move
    // the chance (outside a campaign world: no extra encounter and today's chance)
    let mut pool = json::aget(route, "encounters").to_vec();
    pool.extend(crate::campaign::purse::road_encounters(d, gs, route));
    let normal = &pool[..];
    let elites = json::aget(route, "elites");
    let chance = if gs.campaign.is_some() { crate::campaign::purse::road_chance(d, gs, route, leg_chance(gs, route, false), starved) } else { leg_chance(gs, route, starved) };
    if chance <= 0 || (normal.is_empty() && elites.is_empty()) {
        return Dict::new();
    }
    if !rolls::chance(gs, rolls::TRAVEL, chance) {
        return Dict::new();
    }
    let mut elite = false;
    if !elites.is_empty() && !normal.is_empty() {
        elite = rolls::chance(gs, rolls::TRAVEL, json::i(route, "elite_pct", 0));
    } else if !elites.is_empty() {
        elite = true;
    }
    let r = rolls::pick(gs, rolls::TRAVEL, if elite { elites } else { normal });
    if r.is_empty() {
        return Dict::new();
    }
    fight(d, gs, route, "road", &r, "", elite, from, to)
}

#[allow(clippy::too_many_arguments)]
fn fight(d: &Data, gs: &GameState, route: &Dict, source: &str, r: &str, event: &str, elite: bool, from: &str, to: &str) -> Dict {
    let hit = d.resolve(&json::s_or(route, "_world", &gs.world_id), "encounters", r);
    let enc = json::s_or(hit, "_qid", r);
    let seen = d.resolve(&gs.world_id, "encounters", &enc);
    json::obj([
        ("source", source.into()),
        ("encounter", json::s_or(seen, "_qid", &enc).into()),
        ("route", json::s(route, "_qid").into()),
        ("event", event.into()),
        ("elite", Value::Bool(elite)),
        ("from", from.into()),
        ("to", to.into()),
        ("zone", json::s(route, "zone").into()),
    ])
}

pub fn complete_leg(d: &Data, gs: &mut GameState, route_qid: &str, to: &str) -> Dict {
    quest::set_flag(d, gs, &traversed_flag(route_qid), Value::Bool(true));
    explore::enter(d, gs, to, SPAWN)
}

/// won: apply, scripted done flag and sets_flags, then the leg completes; fled: nothing; lost: wake at the start.
/// REWARD1: `tally` from the battle (None = settled without one); every outcome pays its defeated enemies.
pub fn after_fight(d: &Data, gs: &mut GameState, fight: &Dict, outcome: &str, tally: Option<&crate::battle::Tally>) -> Dict {
    let applied = crate::battle::settle(d, gs, &json::s(fight, "encounter"), outcome, tally);
    let base = |ok: bool, errors: &[String], events: Vec<Value>, xp: Value, adv: Value| {
        let mut out = json::ok_result(ok, errors);
        out.insert("events".into(), Value::Array(events));
        out.insert("xp".into(), xp);
        out.insert("advanced".into(), adv);
        out
    };
    if !json::b(&applied, "ok", false) {
        return base(false, &json::sget(&applied, "errors"), Vec::new(), Value::Object(Dict::new()), Value::Array(Vec::new()));
    }
    let rq = json::s(fight, "route");
    let mut events = Vec::new();
    let mut woke_campaign = Vec::new();
    match outcome {
        "won" => {
            let event_id = json::s(fight, "event");
            if json::s(fight, "source") == "scripted" && !event_id.is_empty() {
                quest::set_flag(d, gs, &event_done_flag(&rq, &event_id), Value::Bool(true));
                let route = route_in(d, &gs.world_id, &rq);
                for x in json::aget(&route, "scripted") {
                    let s = json::dict(&json::normalize(x)).clone();
                    if json::s(&s, "id") != event_id {
                        continue;
                    }
                    let mut keys = Vec::new();
                    for c in json::aget(&s, "sets_flags") {
                        let cond = json::dict(c);
                        let key = json::s(cond, "key");
                        if quest::set_flag(d, gs, &key, cond.get("value").cloned().unwrap_or(Value::Bool(true))).is_empty() {
                            keys.push(key);
                        }
                    }
                    if !keys.is_empty() {
                        events.push(Value::Object(json::obj([
                            ("kind", "flags_set".into()),
                            ("route", rq.clone().into()),
                            ("event", event_id.clone().into()),
                            ("flags", json::strs(&keys)),
                        ])));
                    }
                }
            }
            let entered = complete_leg(d, gs, &rq, &json::s(fight, "to"));
            if !json::b(&entered, "ok", false) {
                return base(false, &json::sget(&entered, "errors"), events, applied["xp"].clone(), applied["advanced"].clone());
            }
            events.extend(json::aget(&entered, "events").iter().cloned());
            // CAMPAIGN1 T1: the arrival's enter tick events follow the battle's
            woke_campaign = json::aget(&entered, "campaign").to_vec();
        }
        "lost" => {
            let woke = explore::wake(d, gs, &json::s(fight, "from"));
            events.extend(json::aget(&woke, "events").iter().cloned());
            woke_campaign = json::aget(&woke, "campaign").to_vec();
        }
        _ => {}
    }
    let mut out = base(true, &[], events, applied["xp"].clone(), applied["advanced"].clone());
    for (k, v) in crate::battle::result_keys(&applied) {
        out.insert(k, v);
    }
    // CAMPAIGN1 R4: a retreat's tick events follow the battle's
    crate::campaign::merge_events(&mut out, woke_campaign);
    out
}
