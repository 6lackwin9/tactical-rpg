//! Travel rules of the world checks (TravelRules): route shape and content, scripted fights, the route graph a world
//! travels on (its nodes are the world map) and the soft route warnings.

use super::map::pair_list_errors;
use super::quest::{condition_errors, flag_list_errors};
use super::world::ref_errors;
use super::View;
use aegis_content::world::split_qid;
use aegis_core::json::{self, Dict};
use serde_json::Value;

const TIME_MIN: i64 = 1;
const TIME_MAX: i64 = 20;
const PCT_MAX: i64 = 100;
const MAX_SCRIPTED: usize = 4;
const SCRIPTED_KEYS: [&str; 5] = ["id", "encounter", "requires_flags", "unless_flags", "sets_flags"];
const SCRIPTED_FLAG_KEYS: [&str; 3] = ["requires_flags", "unless_flags", "sets_flags"];

fn int_in(v: &Value, lo: i64, hi: i64) -> bool {
    json::is_int(v) && (lo..=hi).contains(&json::to_int(v))
}

/// TravelRules.route_errors: a normalized route beyond the generic id / name / ref checks.
pub fn route_errors(v: &View, w: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let (from, to) = (json::s(e, "from"), json::s(e, "to"));
    if from.is_empty() {
        errors.push("Choose where the route starts.".into());
    }
    if to.is_empty() {
        errors.push("Choose where the route leads.".into());
    }
    if !from.is_empty() && !to.is_empty() {
        let (a, b) = (v.worlds.resolve(w, "locations", &from), v.worlds.resolve(w, "locations", &to));
        if !a.is_empty() && !b.is_empty() && json::s(a, "_qid") == json::s(b, "_qid") {
            errors.push("A route can't lead to its own start.".into());
        }
    }
    if !int_in(json::get(e, "time"), TIME_MIN, TIME_MAX) {
        errors.push(format!("Travel time must be {TIME_MIN}..{TIME_MAX}."));
    }
    let zone = e.get("zone").cloned().unwrap_or(Value::from(""));
    // HOOKS1 ZN2: the encounters' zone options (code zones, then the data zone rows)
    if !zone.is_string() || (!json::gstr(&zone).is_empty() && !super::zone_ids(v).contains(&json::gstr(&zone))) {
        errors.push(format!("Unknown zone '{}'.", json::gstr(&zone)));
    }
    if !int_in(json::get(e, "encounter_pct"), 0, PCT_MAX) {
        errors.push(format!("Encounter chance must be 0..{PCT_MAX}."));
    }
    let repeat = json::get(e, "repeat_pct");
    if !int_in(repeat, 0, PCT_MAX) && !(json::is_int(repeat) && json::to_int(repeat) == -1) {
        errors.push(format!("Repeat chance must be -1 (half the encounter chance) or 0..{PCT_MAX}."));
    }
    if !int_in(json::get(e, "elite_pct"), 0, PCT_MAX) {
        errors.push(format!("Elite chance must be 0..{PCT_MAX}."));
    }
    let empty = Value::Array(Vec::new());
    for key in ["encounters", "elites"] {
        errors.extend(pair_list_errors(v, w, "Route", key, e.get(key).unwrap_or(&empty)));
    }
    errors.extend(scripted_errors(v, w, e.get("scripted").unwrap_or(&empty)));
    errors.extend(condition_errors(e.get("requires_flags").unwrap_or(&empty), "Open when"));
    errors.extend(condition_errors(e.get("unless_flags").unwrap_or(&empty), "Closed when"));
    errors
}

fn scripted_errors(v: &View, w: &str, raw: &Value) -> Vec<String> {
    let Some(list) = raw.as_array() else {
        return vec!["Scripted fights must be a list.".into()];
    };
    let mut errors = Vec::new();
    if list.len() > MAX_SCRIPTED {
        errors.push(format!("At most {MAX_SCRIPTED} scripted fights."));
    }
    let mut ids: Vec<String> = Vec::new();
    for (i, raw) in list.iter().enumerate() {
        if !raw.is_object() {
            errors.push(format!("Scripted fight {} must be an object.", i + 1));
            continue;
        }
        let s = json::normalize(raw);
        let s = json::dict(&s);
        let sid = json::s(s, "id");
        if !s.get("id").is_some_and(Value::is_string) || !json::id_ok(&sid) {
            errors.push(format!("Scripted fight {}: id '{sid}' must be lowercase [a-z0-9_].", i + 1));
            continue;
        }
        if ids.contains(&sid) {
            errors.push(format!("Scripted fight id '{sid}' is used twice."));
        }
        ids.push(sid.clone());
        for key in s.keys().filter(|k| !SCRIPTED_KEYS.contains(&k.as_str())) {
            errors.push(format!("Scripted fight '{sid}': unknown key '{key}'."));
        }
        match s.get("encounter").and_then(Value::as_str).filter(|x| !x.is_empty()) {
            None => errors.push(format!("Scripted fight '{sid}': choose an encounter.")),
            Some(enc) => errors.extend(ref_errors(v, w, &format!("Scripted fight '{sid}' encounter"), "encounters", enc)),
        }
        for fk in SCRIPTED_FLAG_KEYS {
            errors.extend(flag_list_errors(s.get(fk).unwrap_or(&Value::Array(Vec::new())), &format!("Scripted fight '{sid}' {fk}"), fk));
        }
    }
    errors
}

/// A route of a world's graph with its endpoints as seen from that world.
struct Edge {
    from: String,
    to: String,
    two_way: bool,
}

fn world_of(location_qid: &str) -> String {
    split_qid(location_qid).map(|(w, _)| w).unwrap_or_default()
}

/// TravelRules.endpoint: a route endpoint resolved where the route is written, then seen from `w`.
fn endpoint(v: &View, w: &str, route: &Dict, key: &str) -> String {
    let home = json::s_or(route, "_world", w);
    let hit = v.worlds.resolve(&home, "locations", &json::s(route, key));
    if hit.is_empty() {
        return String::new();
    }
    let q = json::s(hit, "_qid");
    let seen = v.worlds.resolve(w, "locations", &q);
    if seen.is_empty() {
        q
    } else {
        json::s(seen, "_qid")
    }
}

fn visible(v: &View, w: &str, location_qid: &str) -> bool {
    let lw = world_of(location_qid);
    !lw.is_empty() && v.worlds.can_reach_world(w, &lw) && v.worlds.has_entry(w, "locations", location_qid)
}

/// TravelRules.graph: the world's merged routes, then the two-way routes of the other worlds that reach it.
fn graph(v: &View, w: &str) -> Vec<Edge> {
    let mut out = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let chain = v.worlds.lineage(w);
    let mut add = |r: &Dict, needs_visible: bool, out: &mut Vec<Edge>| {
        let rq = json::s(r, "_qid");
        if seen.contains(&rq) {
            return;
        }
        let (a, b) = (endpoint(v, w, r, "from"), endpoint(v, w, r, "to"));
        if a.is_empty() || b.is_empty() || (needs_visible && !visible(v, w, &a) && !visible(v, w, &b)) {
            return;
        }
        seen.push(rq);
        out.push(Edge { from: a, to: b, two_way: json::b(r, "two_way", true) });
    };
    for r in v.worlds.merged_entries(w, "routes") {
        add(r, false, &mut out);
    }
    for x in v.worlds.world_ids() {
        if chain.contains(&x) || v.worlds.lineage(&x).contains(&w.to_string()) || !v.worlds.can_reach_world(&x, w) {
            continue;
        }
        for r in v.worlds.merged_entries(&x, "routes") {
            if json::b(r, "two_way", true) && !chain.contains(&json::s(r, "_world")) {
                add(r, true, &mut out);
            }
        }
    }
    out
}

/// TravelRules.nodes: every endpoint of the world's route graph, first-seen order.
pub fn nodes(v: &View, w: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for edge in graph(v, w) {
        for q in [edge.from, edge.to] {
            if !out.contains(&q) {
                out.push(q);
            }
        }
    }
    out
}

/// TravelRules.warnings: a chance with nothing to fight, and a route into a world with no way back.
pub fn warnings(v: &View, w: &str) -> Vec<String> {
    let mut out = Vec::new();
    let chain = v.worlds.lineage(w);
    for e in v.worlds.own_entries(w, "routes") {
        let rq = json::s(e, "_qid");
        let pct = json::i(e, "encounter_pct", 0);
        if pct > 0 && json::aget(e, "encounters").is_empty() && json::aget(e, "elites").is_empty() {
            out.push(format!("{rq}: encounter chance {pct}% but no encounters or elites (it never fights)."));
        }
        let to_hit = v.worlds.resolve(w, "locations", &json::s(e, "to"));
        if to_hit.is_empty() {
            continue;
        }
        let to_world = json::s(to_hit, "_world");
        let way_back = graph(v, &to_world).iter().any(|r| {
            let (a_home, b_home) = (chain.contains(&world_of(&r.from)), chain.contains(&world_of(&r.to)));
            (!a_home && b_home) || (r.two_way && a_home && !b_home)
        });
        if chain.contains(&to_world) || way_back {
            continue;
        }
        out.push(format!("{rq}: leads into world '{to_world}' with no route back."));
    }
    out
}
