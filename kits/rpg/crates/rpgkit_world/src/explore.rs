//! MapRules (runtime reads) and ExploreRunner (scripts/exploration/): walking a location's region map over
//! GameState. Stateless; every call returns {ok, errors, events...} and errors change nothing. The caller starts a
//! battle on a `fight` event and calls after_fight; it writes the save on `autosave` (no I/O here).

use crate::data::Data;
use crate::state::{GameState, HOME_WORLD_KEY, NO_POSITION, SUPPLIES_MAX};
use crate::{dialogue, progression, quest, rolls, travel};
use aegis_core::json::{self, int, Dict};
use serde_json::Value;

pub type Pos = (i64, i64);
pub const DIRS: [Pos; 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
pub const SPAWN: &str = "spawn";
pub const WORLD_MAP: &str = "world_map";
pub const DEFAULT_ELITE_PCT: i64 = 10;
pub const DEFAULT_STEPS_MIN: i64 = 12;
pub const DEFAULT_STEPS_MAX: i64 = 20;
const SOLID_KINDS: [&str; 3] = ["npc", "chest", "lore"];

// ---- MapRules ------------------------------------------------------------------------------------------------

pub fn rows_of(t: &Dict) -> Vec<Vec<char>> {
    json::aget(t, "rows").iter().map(|r| json::gstr(r).chars().collect()).collect()
}

pub fn size_of(t: &Dict) -> Pos {
    let rows = rows_of(t);
    (rows.first().map(|r| r.len()).unwrap_or(0) as i64, rows.len() as i64)
}

/// The tile char at (x, y); None off the map.
pub fn tile(t: &Dict, x: i64, y: i64) -> Option<char> {
    let rows = json::aget(t, "rows");
    if y < 0 || y as usize >= rows.len() || x < 0 {
        return None;
    }
    json::gstr(&rows[y as usize]).chars().nth(x as usize)
}

pub fn is_open(t: &Dict, x: i64, y: i64) -> bool {
    tile(t, x, y) == Some('.')
}

fn legend(t: &Dict) -> &Dict {
    json::dget(t, "zone_legend")
}

pub fn zone_kind(t: &Dict, zone_id: &str) -> String {
    for z in legend(t).values() {
        if let Some(z) = z.as_object() {
            if json::s(z, "id") == zone_id {
                return json::s(z, "kind");
            }
        }
    }
    String::new()
}

/// The zone id under (x, y); "" = no zone (SAFE).
pub fn zone_at(t: &Dict, x: i64, y: i64) -> String {
    let zones = json::aget(t, "zones");
    if y < 0 || y as usize >= zones.len() || x < 0 {
        return String::new();
    }
    let row = json::gstr(&zones[y as usize]);
    match row.chars().nth(x as usize) {
        None | Some('-') => String::new(),
        Some(c) => legend(t).get(&c.to_string()).and_then(Value::as_object).map(|z| json::s(z, "id")).unwrap_or_default(),
    }
}

pub fn anchor_pos(t: &Dict, anchor: &str) -> Pos {
    for a in json::aget(t, "anchors") {
        let a = json::dict(a);
        if json::s(a, "id") == anchor {
            return (json::i(a, "x", -1), json::i(a, "y", -1));
        }
    }
    NO_POSITION
}

pub fn path_points(t: &Dict, path: &str) -> Vec<Pos> {
    json::dget(t, "paths")
        .get(path)
        .map(json::arr)
        .unwrap_or(&[])
        .iter()
        .filter_map(|p| match json::arr(p) {
            [x, y] => Some((json::to_int(x), json::to_int(y))),
            _ => None,
        })
        .collect()
}

/// Ping-pong index along a path of `length` points after `loc_steps` steps.
pub fn roamer_index(length: usize, loc_steps: i64) -> usize {
    if length <= 1 {
        return 0;
    }
    let period = 2 * length as i64 - 2;
    let t = loc_steps.rem_euclid(period);
    (if t < length as i64 { t } else { period - t }) as usize
}

/// A marker with its kind's defaults filled.
pub fn marker_full(m: &Dict) -> Dict {
    let kind = json::s(m, "kind");
    let mut out = json::obj([
        ("id", "".into()),
        ("kind", "".into()),
        ("requires_flags", Value::Array(Vec::new())),
        ("unless_flags", Value::Array(Vec::new())),
        ("sets_flags", Value::Array(Vec::new())),
        ("label", "".into()),
        ("note", "".into()),
    ]);
    if kind != "roamer" {
        out.insert("anchor".into(), "".into());
    }
    let defaults: Vec<(&str, Value)> = match kind.as_str() {
        "exit" => vec![("to", "".into()), ("to_anchor", "spawn".into()), ("locked_text", "The way is shut.".into())],
        "encounter" => vec![("encounter", "".into()), ("trigger", "touch".into()), ("radius", int(1)), ("once", Value::Bool(true))],
        "roamer" => vec![("encounter", "".into()), ("path", "".into()), ("radius", int(0)), ("once", Value::Bool(true))],
        "npc" => vec![("character", "".into())],
        "chest" => vec![("items", Value::Object(Dict::new())), ("loot", "".into()), ("gold", int(0)), ("supplies", int(0))],
        "lore" => vec![("lore", "".into())],
        _ => vec![],
    };
    for (k, v) in defaults {
        out.insert(k.into(), v);
    }
    for (k, v) in json::normalize_dict(m) {
        out.insert(k, v);
    }
    out
}

pub fn binding_full(b: &Dict) -> Dict {
    let mut out = json::obj([
        ("encounters", Value::Array(Vec::new())),
        ("elites", Value::Array(Vec::new())),
        ("elite_pct", int(DEFAULT_ELITE_PCT)),
        ("steps_min", int(DEFAULT_STEPS_MIN)),
        ("steps_max", int(DEFAULT_STEPS_MAX)),
    ]);
    for (k, v) in json::normalize_dict(b) {
        out.insert(k, v);
    }
    out
}

pub fn exit_targets(e: &Dict) -> Vec<String> {
    json::aget(e, "exits").iter().map(|x| json::s(json::dict(x), "to")).filter(|t| !t.is_empty()).collect()
}

/// The world's start location qid (nearest manifest that sets one), seen from `world_id`.
pub fn start_location(d: &Data, world_id: &str) -> String {
    for w in d.lineage(world_id).iter().rev() {
        let r = json::s(d.manifest(w), "start_location");
        if r.is_empty() {
            continue;
        }
        let hit = d.resolve(w, "locations", &r);
        if hit.is_empty() {
            return String::new();
        }
        let q = json::s(hit, "_qid");
        return json::s_or(d.resolve(world_id, "locations", &q), "_qid", &q);
    }
    String::new()
}

pub fn dotted(qid: &str) -> String {
    qid.replace(':', ".")
}

pub fn done_flag(loc_qid: &str, marker: &str) -> String {
    format!("{}.{marker}.done", dotted(loc_qid))
}

pub fn visited_flag(loc_qid: &str) -> String {
    format!("{}.visited", dotted(loc_qid))
}

// ---- ExploreRunner: lookups ----------------------------------------------------------------------------------

pub fn location<'a>(d: &'a Data, gs: &GameState) -> &'a Dict {
    if gs.location_id.is_empty() {
        return json::empty();
    }
    d.resolve(&gs.world_id, "locations", &gs.location_id)
}

pub fn template_of<'a>(d: &'a Data, loc: &Dict) -> &'a Dict {
    d.map(&json::s(loc, "map"))
}

pub fn markers_of(loc: &Dict) -> Vec<Dict> {
    json::aget(loc, "markers").iter().map(|m| marker_full(json::dict(m))).collect()
}

fn is_done(gs: &GameState, loc_qid: &str, m: &Dict) -> bool {
    gs.flag_true(&done_flag(loc_qid, &json::s(m, "id")))
}

/// "present", "locked" or "absent".
pub fn presence(d: &Data, gs: &GameState, loc_qid: &str, m: &Dict) -> &'static str {
    let req_ok = quest::all_hold(gs, json::get(m, "requires_flags"));
    let unless_ok = !quest::any_holds(gs, json::get(m, "unless_flags"));
    let kind = json::s(m, "kind");
    if (kind == "encounter" || kind == "roamer") && json::b(m, "once", true) && is_done(gs, loc_qid, m) {
        return "absent";
    }
    if kind == "exit" && !req_ok && unless_ok {
        return "locked";
    }
    if kind == "npc" && dialogue::is_gone(d, gs, &json::s(m, "character")) {
        return "absent";
    }
    if req_ok && unless_ok {
        "present"
    } else {
        "absent"
    }
}

fn is_spent(gs: &GameState, loc_qid: &str, m: &Dict) -> bool {
    match json::s(m, "kind").as_str() {
        "chest" => is_done(gs, loc_qid, m),
        "point" => {
            let sets = json::normalize(json::get(m, "sets_flags"));
            !json::arr(&sets).is_empty() && quest::all_hold(gs, &sets)
        }
        _ => false,
    }
}

fn npc_badge(d: &Data, gs: &GameState, character_qid: &str) -> &'static str {
    if character_qid.is_empty() {
        return "";
    }
    for qid in quest::quests_with(d, gs, "ready") {
        if dialogue::turn_in_qid(d, gs, &quest::resolved(d, gs, &qid)) == character_qid {
            return "ready";
        }
    }
    for qid in quest::quests_with(d, gs, "available") {
        if dialogue::giver_qid(d, gs, &quest::resolved(d, gs, &qid)) == character_qid {
            return "quest";
        }
    }
    ""
}

/// MAPREAD1-L1: {kind, state, label, verb, badge, icon} of a marker, read-only.
pub fn marker_view(d: &Data, gs: &GameState, loc: &Dict, m: &Dict) -> Dict {
    let mk = marker_full(m);
    let kind = json::s(&mk, "kind");
    let loc_qid = json::s(loc, "_qid");
    let mut state = presence(d, gs, &loc_qid, &mk);
    if state == "present" && is_spent(gs, &loc_qid, &mk) {
        state = "spent";
    }
    let spent = state == "spent";
    let label = json::s(&mk, "label").trim().to_string();
    let mut badge = "";
    let (default_label, verb) = match kind.as_str() {
        "npc" => {
            let who = d.resolve(&gs.world_id, "characters", &json::s(&mk, "character"));
            badge = npc_badge(d, gs, &json::s(who, "_qid"));
            (json::s_or(who, "name", "Someone"), "Talk".to_string())
        }
        "chest" => ("Chest".into(), if spent { "Empty" } else { "Open" }.into()),
        "lore" => (json::s_or(d.resolve(&gs.world_id, "lore", &json::s(&mk, "lore")), "name", "Lore"), "Read".into()),
        "exit" => {
            let to = json::s(&mk, "to");
            let lbl = if to == WORLD_MAP { "World map".into() } else { format!("To {}", json::s_or(resolve_location(d, gs, &to), "name", &to)) };
            let verb = if state == "locked" { format!("Locked: {}", json::s(&mk, "locked_text")) } else { "Walk onto it".into() };
            (lbl, verb)
        }
        "encounter" | "roamer" => (
            json::s_or(d.resolve(&gs.world_id, "encounters", &json::s(&mk, "encounter")), "name", "Enemies"),
            if kind == "encounter" { "Fight" } else { "Patrol: fight on contact" }.into(),
        ),
        _ => ("Something here".into(), if spent { "Nothing more here" } else { "Look" }.into()),
    };
    json::obj([
        ("kind", kind.clone().into()),
        ("state", state.into()),
        ("label", if label.is_empty() { default_label } else { label }.into()),
        ("verb", verb.into()),
        ("badge", badge.into()),
        ("icon", format!("overlay_{}", if kind == "point" { "poi" } else { kind.as_str() }).into()),
    ])
}

/// The NPC services in badge order.
pub const SERVICES: [&str; 4] = ["turn_in", "offer", "shop", "recruit"];

/// PLAYUI1-M (spec PLAYUI1 2.2 A; engine-only, no Godot function): what the character `character_qid` offers now, in
/// `SERVICES` order: `turn_in` (a ready quest turns in here), `offer` (an available quest starts here; both as the
/// marker badges `ready` and `quest`), `shop` (keeper of an open shop), `recruit` (a choice in its dialogue tree with a
/// `recruit` effect for a character not in the party, the choice's conditions holding). Read-only; `marker_view` stays
/// as Godot had it (the journey parity compares it).
pub fn npc_services(d: &Data, gs: &GameState, character_qid: &str) -> Vec<&'static str> {
    let mut out = Vec::new();
    if character_qid.is_empty() {
        return out;
    }
    if quest::quests_with(d, gs, "ready").iter().any(|q| dialogue::turn_in_qid(d, gs, &quest::resolved(d, gs, q)) == character_qid) {
        out.push("turn_in");
    }
    if quest::quests_with(d, gs, "available").iter().any(|q| dialogue::giver_qid(d, gs, &quest::resolved(d, gs, q)) == character_qid) {
        out.push("offer");
    }
    let keeps_shop = d.merged_entries(&gs.world_id, "shops").into_iter().any(|sh| {
        dialogue::character_qid(d, gs, &json::s(sh, "keeper")) == character_qid && crate::shop::is_open(d, gs, &json::s(sh, "_qid"))
    });
    if keeps_shop {
        out.push("shop");
    }
    let c = d.resolve(&gs.world_id, "characters", character_qid);
    let tree_ref = json::s(c, "dialogue");
    let tree = if tree_ref.is_empty() { json::empty() } else { d.resolve(&gs.world_id, "dialogues", &tree_ref) };
    let recruits = json::arr(&json::normalize(json::get(tree, "nodes"))).iter().any(|n| {
        json::arr(&json::normalize(json::get(json::dict(n), "choices"))).iter().any(|ch| {
            let ch = json::dict(ch);
            let fx = json::normalize(json::get(ch, "effects"));
            json::arr(&fx).iter().any(|f| {
                let f = json::dict(f);
                json::s(f, "effect") == "recruit" && !dialogue::is_member(d, gs, &json::s(f, "character"))
            }) && dialogue::choice_open(d, gs, ch)
        })
    });
    if recruits {
        out.push("recruit");
    }
    out
}

/// Where a marker stands for `loc_steps` (roamers walk their path).
pub fn marker_pos(t: &Dict, m: &Dict, loc_steps: i64) -> Pos {
    if json::s(m, "kind") == "roamer" {
        let pts = path_points(t, &json::s(m, "path"));
        return if pts.is_empty() { NO_POSITION } else { pts[roamer_index(pts.len(), loc_steps)] };
    }
    anchor_pos(t, &json::s(m, "anchor"))
}

fn solid(state: &str, m: &Dict) -> bool {
    state == "locked" || (state == "present" && SOLID_KINDS.contains(&json::s(m, "kind").as_str()))
}

pub fn tile_safe(t: &Dict, p: Pos) -> bool {
    t.is_empty() || zone_kind(t, &zone_at(t, p.0, p.1)) != "danger"
}

pub fn binding_at(loc: &Dict, t: &Dict, p: Pos) -> Dict {
    let zone = zone_at(t, p.0, p.1);
    if zone.is_empty() || zone_kind(t, &zone) != "danger" {
        return Dict::new();
    }
    match json::dget(loc, "zone_bindings").get(&zone) {
        Some(b) => binding_full(json::dict(b)),
        None => Dict::new(),
    }
}

fn pos_value(p: Pos) -> Value {
    Value::Array(vec![int(p.0), int(p.1)])
}

fn result(ok: bool, errors: &[String], events: Vec<Value>) -> Dict {
    let mut out = json::ok_result(ok, errors);
    out.insert("events".into(), Value::Array(events));
    out
}

fn fail(message: &str) -> Dict {
    result(false, &[message.to_string()], Vec::new())
}

fn ev<const N: usize>(pairs: [(&str, Value); N]) -> Value {
    Value::Object(json::obj(pairs))
}

fn apply_sets(d: &Data, gs: &mut GameState, m: &Dict) -> Vec<String> {
    let mut keys = Vec::new();
    for c in json::arr(&json::normalize(json::get(m, "sets_flags"))) {
        let cond = json::dict(c);
        let key = json::s(cond, "key");
        if quest::set_flag(d, gs, &key, cond.get("value").cloned().unwrap_or(Value::Bool(true))).is_empty() {
            keys.push(key);
        }
    }
    keys
}

// ---- Entering and leaving ------------------------------------------------------------------------------------

/// The location `r` names: from gs.world_id first, else a qualified id in its own world.
pub fn resolve_location<'a>(d: &'a Data, gs: &GameState, r: &str) -> &'a Dict {
    let hit = d.resolve(&gs.world_id, "locations", r);
    if hit.is_empty() && r.contains(':') {
        if let Some((w, local)) = aegis_content::split_qid(r) {
            if d.has_world(&w) {
                return d.resolve(&w, "locations", &local);
            }
        }
    }
    hit
}

pub fn entry_problem(d: &Data, gs: &GameState, location_ref: &str, anchor: &str) -> String {
    let loc = resolve_location(d, gs, location_ref);
    if loc.is_empty() {
        return format!("Unknown location '{location_ref}'.");
    }
    let t = template_of(d, loc);
    if !json::s(loc, "map").is_empty() && t.is_empty() {
        return format!("Location '{}' names an unknown map '{}'.", json::s(loc, "_qid"), json::s(loc, "map"));
    }
    if !t.is_empty() && anchor_pos(t, anchor) == NO_POSITION {
        return format!("Map '{}' has no anchor '{anchor}'.", json::s(t, "id"));
    }
    String::new()
}

pub fn enter(d: &Data, gs: &mut GameState, location_ref: &str, anchor: &str) -> Dict {
    let problem = entry_problem(d, gs, location_ref, anchor);
    if !problem.is_empty() {
        return fail(&problem);
    }
    let loc = resolve_location(d, gs, location_ref);
    let t = template_of(d, loc);
    let pos = if t.is_empty() { NO_POSITION } else { anchor_pos(t, anchor) };
    let loc_world = json::s(loc, "_world");
    let home = json::s(&gs.explore, HOME_WORLD_KEY);
    if !home.is_empty() && d.lineage(&home).contains(&loc_world) {
        gs.world_id = home;
        gs.explore.shift_remove(HOME_WORLD_KEY);
    } else if !d.lineage(&gs.world_id).contains(&loc_world) {
        if home.is_empty() {
            gs.explore.insert(HOME_WORLD_KEY.into(), gs.world_id.clone().into());
        }
        gs.world_id = loc_world;
    }
    let qid = json::s(loc, "_qid");
    gs.location_id = qid.clone();
    gs.position = pos;
    gs.explore.insert("loc_steps".into(), int(0));
    quest::set_flag(d, gs, &visited_flag(&qid), Value::Bool(true));
    if t.is_empty() || tile_safe(t, anchor_pos(t, SPAWN)) {
        gs.explore.insert("last_safe".into(), qid.clone().into());
    }
    let mut restored = 0;
    if json::b(loc, "rest", false) {
        gs.explore.insert("last_rest".into(), qid.clone().into());
        restored = progression::rest(d, &mut gs.party);
    }
    quest::refresh(d, gs);
    let advanced = quest::on_location_reached(d, gs, &qid);
    let mut events = vec![ev([("kind", "entered".into()), ("location", qid.clone().into()), ("advanced", json::strs(&advanced))])];
    if restored > 0 {
        events.push(ev([("kind", "rested".into()), ("location", qid.into()), ("restored", int(restored))]));
    }
    events.push(ev([("kind", "autosave".into())]));
    result(true, &[], events)
}

pub fn begin(d: &Data, gs: &mut GameState) -> Dict {
    let start = start_location(d, &gs.world_id);
    if start.is_empty() {
        return fail(&format!("World '{}' has no start location.", gs.world_id));
    }
    enter(d, gs, &start, SPAWN)
}

/// After a load: no hooks; a position off the map goes back to spawn.
pub fn resume(d: &Data, gs: &mut GameState) -> Dict {
    if gs.location_id.is_empty() {
        return begin(d, gs);
    }
    let loc = location(d, gs);
    if loc.is_empty() {
        return fail(&format!("Unknown location '{}'.", gs.location_id));
    }
    let t = template_of(d, loc);
    let mut events = Vec::new();
    if t.is_empty() {
        gs.position = NO_POSITION;
    } else if !is_open(t, gs.position.0, gs.position.1) {
        gs.position = anchor_pos(t, SPAWN);
        events.push(ev([("kind", "moved".into()), ("from", Value::Array(Vec::new())), ("to", pos_value(gs.position))]));
    }
    result(true, &[], events)
}

pub fn take_exit(d: &Data, gs: &mut GameState, to: &str, anchor: &str) -> Dict {
    let loc = location(d, gs);
    if !exit_targets(loc).iter().any(|x| x == to) && to != WORLD_MAP {
        return fail(&format!("No exit to '{to}' here."));
    }
    if to == WORLD_MAP {
        return result(true, &[], vec![ev([("kind", "exit".into()), ("to", WORLD_MAP.into())])]);
    }
    enter(d, gs, to, anchor)
}

/// {to, label} of a map-less location's exits, plus the world map when the location is a node.
pub fn menu(d: &Data, gs: &GameState) -> Vec<Dict> {
    let mut out = Vec::new();
    let loc = location(d, gs);
    if loc.is_empty() || !json::s(loc, "map").is_empty() {
        return out;
    }
    for x in json::aget(loc, "exits") {
        let ex = json::dict(x);
        out.push(json::obj([("to", json::s(ex, "to").into()), ("label", json::s_or(ex, "label", &json::s(ex, "to")).into())]));
    }
    if travel::nodes(d, &gs.world_id).contains(&json::s(loc, "_qid")) {
        out.push(json::obj([("to", WORLD_MAP.into()), ("label", "World map".into())]));
    }
    out
}

/// The defeat rule: heal, pick the checkpoint, wake there. Adds a `defeat` event.
pub fn wake(d: &Data, gs: &mut GameState, at: &str) -> Dict {
    let target = if at.is_empty() { gs.location_id.clone() } else { at.to_string() };
    let loc = resolve_location(d, gs, &target);
    let t = template_of(d, loc);
    let checkpoint = if !loc.is_empty() && (t.is_empty() || tile_safe(t, anchor_pos(t, SPAWN))) {
        json::s(loc, "_qid")
    } else {
        json::s(&gs.explore, "last_safe")
    };
    let defeat = progression::defeat_plan(d, gs, &checkpoint);
    let mut to = json::s(&defeat, "to");
    if to.is_empty() {
        to = start_location(d, &gs.world_id);
        if to.is_empty() {
            return fail(&format!("World '{}' has no start location.", gs.world_id));
        }
    }
    let problem = entry_problem(d, gs, &to, SPAWN);
    if !problem.is_empty() {
        return fail(&problem);
    }
    progression::settle_defeat(gs, &defeat);
    let mut woke = enter(d, gs, &to, SPAWN);
    if json::b(&woke, "ok", false) {
        let mut events = json::aget(&woke, "events").to_vec();
        events.push(ev([
            ("kind", "defeat".into()),
            ("mode", json::s(&defeat, "mode").into()),
            ("spent", int(json::i(&defeat, "spent", 0))),
            ("location", gs.location_id.clone().into()),
        ]));
        woke.insert("events".into(), Value::Array(events));
    }
    woke
}

// ---- Stepping ------------------------------------------------------------------------------------------------

pub fn step(d: &Data, gs: &mut GameState, dir: Pos) -> Dict {
    if !DIRS.contains(&dir) {
        return fail("A step is one tile up, down, left or right.");
    }
    let loc = location(d, gs);
    let t = template_of(d, loc);
    if t.is_empty() {
        return fail("This location has no map to walk.");
    }
    let mut events = Vec::new();
    let loc_qid = json::s(loc, "_qid");
    let markers = markers_of(loc);
    let mut loc_steps = gs.explore_int("loc_steps");
    let from = gs.position;
    let target = (from.0 + dir.0, from.1 + dir.1);
    if !is_open(t, target.0, target.1) {
        events.push(ev([("kind", "blocked".into()), ("at", pos_value(target))]));
        return result(true, &[], events);
    }
    for m in &markers {
        let state = presence(d, gs, &loc_qid, m);
        if solid(state, m) && marker_pos(t, m, loc_steps) == target {
            if state == "locked" {
                events.push(ev([("kind", "locked".into()), ("marker", json::s(m, "id").into()), ("text", json::s(m, "locked_text").into())]));
            } else {
                events.push(ev([("kind", "blocked".into()), ("at", pos_value(target)), ("marker", json::s(m, "id").into())]));
            }
            return result(true, &[], events);
        }
    }
    for m in &markers {
        if json::s(m, "kind") == "exit" && json::s(m, "to") == WORLD_MAP && presence(d, gs, &loc_qid, m) == "present" && marker_pos(t, m, loc_steps) == target {
            let keys = apply_sets(d, gs, m);
            if !keys.is_empty() {
                events.push(ev([("kind", "flags_set".into()), ("marker", json::s(m, "id").into()), ("flags", json::strs(&keys))]));
            }
            events.push(ev([("kind", "exit".into()), ("to", WORLD_MAP.into()), ("marker", json::s(m, "id").into())]));
            return result(true, &[], events);
        }
    }
    for m in &markers {
        if json::s(m, "kind") == "exit" && presence(d, gs, &loc_qid, m) == "present" && marker_pos(t, m, loc_steps) == target {
            let mut probe = gs.clone();
            let res = enter(d, &mut probe, &json::s(m, "to"), &json::s_or(m, "to_anchor", SPAWN));
            if !json::b(&res, "ok", false) {
                return result(false, &json::sget(&res, "errors"), events);
            }
        }
    }
    gs.position = target;
    events.push(ev([("kind", "moved".into()), ("from", pos_value(from)), ("to", pos_value(target))]));
    for m in &markers {
        if marker_pos(t, m, loc_steps) != target || presence(d, gs, &loc_qid, m) != "present" {
            continue;
        }
        match json::s(m, "kind").as_str() {
            "point" => {
                let keys = apply_sets(d, gs, m);
                if !keys.is_empty() {
                    events.push(ev([("kind", "flags_set".into()), ("marker", json::s(m, "id").into()), ("flags", json::strs(&keys))]));
                }
            }
            "exit" => {
                let keys = apply_sets(d, gs, m);
                if !keys.is_empty() {
                    events.push(ev([("kind", "flags_set".into()), ("marker", json::s(m, "id").into()), ("flags", json::strs(&keys))]));
                }
                let to = json::s(m, "to");
                let left = enter(d, gs, &to, &json::s_or(m, "to_anchor", SPAWN));
                if !json::b(&left, "ok", false) {
                    return result(false, &json::sget(&left, "errors"), events);
                }
                events.push(ev([("kind", "exit".into()), ("to", to.into()), ("marker", json::s(m, "id").into()), ("events", left["events"].clone())]));
                return result(true, &[], events);
            }
            _ => {}
        }
    }
    loc_steps += 1;
    gs.explore.insert("loc_steps".into(), int(loc_steps));
    let binding = binding_at(loc, t, target);
    if !binding.is_empty() {
        let steps = gs.explore_int("steps") + 1;
        gs.explore.insert("steps".into(), int(steps));
    }
    let mut fight = trigger(d, gs, loc, t, &markers, target, loc_steps, &binding);
    if !fight.is_empty() {
        fight.insert("from".into(), pos_value(from));
        fight.insert("location".into(), loc_qid.clone().into());
        events.push(Value::Object(fight));
    }
    for m in &markers {
        if !SOLID_KINDS.contains(&json::s(m, "kind").as_str()) || presence(d, gs, &loc_qid, m) != "present" {
            continue;
        }
        let at = marker_pos(t, m, loc_steps);
        for dd in DIRS {
            if (target.0 + dd.0, target.1 + dd.1) == at {
                events.push(ev([
                    ("kind", "interact_hint".into()),
                    ("marker", json::s(m, "id").into()),
                    ("marker_kind", json::s(m, "kind").into()),
                    ("dir", pos_value(dd)),
                ]));
            }
        }
    }
    result(true, &[], events)
}

fn manhattan(a: Pos, b: Pos) -> i64 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

fn fight_event(d: &Data, gs: &GameState, source: &str, r: &str, marker: &str, elite: bool) -> Dict {
    json::obj([
        ("kind", "fight".into()),
        ("source", source.into()),
        ("encounter", json::s(d.resolve(&gs.world_id, "encounters", r), "_qid").into()),
        ("marker", marker.into()),
        ("elite", Value::Bool(elite)),
    ])
}

#[allow(clippy::too_many_arguments)]
fn trigger(d: &Data, gs: &mut GameState, loc: &Dict, t: &Dict, markers: &[Dict], at: Pos, loc_steps: i64, binding: &Dict) -> Dict {
    let loc_qid = json::s(loc, "_qid");
    for m in markers {
        if json::s(m, "kind") == "encounter" && json::s(m, "trigger") == "touch" && presence(d, gs, &loc_qid, m) == "present" && marker_pos(t, m, loc_steps) == at {
            return fight_event(d, gs, "marker", &json::s(m, "encounter"), &json::s(m, "id"), false);
        }
    }
    for m in markers {
        if json::s(m, "kind") == "roamer" && presence(d, gs, &loc_qid, m) == "present" {
            let rp = marker_pos(t, m, loc_steps);
            if rp != NO_POSITION && manhattan(rp, at) <= json::i(m, "radius", 0) {
                return fight_event(d, gs, "roamer", &json::s(m, "encounter"), &json::s(m, "id"), false);
            }
        }
    }
    for m in markers {
        if json::s(m, "kind") == "encounter" && json::s(m, "trigger") == "proximity" && presence(d, gs, &loc_qid, m) == "present" && manhattan(marker_pos(t, m, loc_steps), at) <= json::i(m, "radius", 1) {
            return fight_event(d, gs, "marker", &json::s(m, "encounter"), &json::s(m, "id"), false);
        }
    }
    if binding.is_empty() || gs.explore_int("steps") < json::i(&gs.explore, "next_at", 1) {
        return Dict::new();
    }
    counter_pick(d, gs, binding)
}

fn counter_pick(d: &Data, gs: &mut GameState, binding: &Dict) -> Dict {
    let normal = json::aget(binding, "encounters");
    let elites = json::aget(binding, "elites");
    let mut elite = false;
    if !elites.is_empty() && !normal.is_empty() {
        elite = rolls::chance(gs, rolls::EXPLORE, json::i(binding, "elite_pct", DEFAULT_ELITE_PCT));
    } else if !elites.is_empty() {
        elite = true;
    }
    let r = rolls::pick(gs, rolls::EXPLORE, if elite { elites } else { normal });
    if r.is_empty() {
        return Dict::new();
    }
    fight_event(d, gs, "counter", &r, "", elite)
}

/// steps = 0, next_at rolled from the bound danger zone under the party, else 12..20.
pub fn reset_counter(d: &Data, gs: &mut GameState) {
    let loc = location(d, gs);
    let t = template_of(d, loc);
    let binding = if t.is_empty() { Dict::new() } else { binding_at(loc, t, gs.position) };
    let lo = json::i(&binding, "steps_min", DEFAULT_STEPS_MIN);
    let hi = json::i(&binding, "steps_max", DEFAULT_STEPS_MAX);
    gs.explore.insert("steps".into(), int(0));
    let next = rolls::range_i(gs, rolls::EXPLORE, lo, hi);
    gs.explore.insert("next_at".into(), int(next));
}

/// BattleBridge.apply (REWARD1: `tally` from the battle, None = settled without one), then won: done flag and
/// sets_flags; fled: back a tile; lost: wake. Then the counter reset.
pub fn after_fight(d: &Data, gs: &mut GameState, fight: &Dict, outcome: &str, tally: Option<&crate::battle::Tally>) -> Dict {
    let applied = crate::battle::settle(d, gs, &json::s(fight, "encounter"), outcome, tally);
    if !json::b(&applied, "ok", false) {
        let mut out = result(false, &json::sget(&applied, "errors"), Vec::new());
        out.insert("xp".into(), Value::Object(Dict::new()));
        out.insert("advanced".into(), Value::Array(Vec::new()));
        return out;
    }
    let mut events = Vec::new();
    let marker_id = json::s(fight, "marker");
    match outcome {
        "won" if !marker_id.is_empty() => {
            let loc = location(d, gs).clone();
            for m in markers_of(&loc) {
                if json::s(&m, "id") != marker_id {
                    continue;
                }
                if json::b(&m, "once", true) {
                    quest::set_flag(d, gs, &done_flag(&json::s(&loc, "_qid"), &marker_id), Value::Bool(true));
                }
                let keys = apply_sets(d, gs, &m);
                if !keys.is_empty() {
                    events.push(ev([("kind", "flags_set".into()), ("marker", marker_id.clone().into()), ("flags", json::strs(&keys))]));
                }
            }
        }
        "fled" => {
            if let [x, y] = json::aget(fight, "from") {
                gs.position = (json::to_int(x), json::to_int(y));
            }
        }
        "lost" => {
            let woke = wake(d, gs, "");
            events.extend(json::aget(&woke, "events").iter().cloned());
        }
        _ => {}
    }
    reset_counter(d, gs);
    let mut out = result(true, &[], events);
    out.insert("xp".into(), applied["xp"].clone());
    out.insert("advanced".into(), applied["advanced"].clone());
    for (k, v) in crate::battle::result_keys(&applied) {
        out.insert(k, v);
    }
    out
}

// ---- Interacting ---------------------------------------------------------------------------------------------

/// The present npc / chest / lore marker on position + dir: {ok, errors, kind ("" = nothing), ...}.
pub fn interact(d: &Data, gs: &mut GameState, dir: Pos) -> Dict {
    let nothing = |ok: bool, errors: &[String]| {
        let mut out = json::ok_result(ok, errors);
        out.insert("kind".into(), "".into());
        out
    };
    if !DIRS.contains(&dir) {
        return nothing(false, &["Interact faces up, down, left or right.".into()]);
    }
    let loc = location(d, gs);
    let t = template_of(d, loc);
    if t.is_empty() {
        return nothing(false, &["This location has no map.".into()]);
    }
    let loc_qid = json::s(loc, "_qid");
    let at = (gs.position.0 + dir.0, gs.position.1 + dir.1);
    let loc_steps = gs.explore_int("loc_steps");
    for m in markers_of(loc) {
        let kind = json::s(&m, "kind");
        if !SOLID_KINDS.contains(&kind.as_str()) || presence(d, gs, &loc_qid, &m) != "present" || marker_pos(t, &m, loc_steps) != at {
            continue;
        }
        let mut out = json::ok_result(true, &[]);
        out.insert("kind".into(), kind.clone().into());
        out.insert("marker".into(), json::s(&m, "id").into());
        match kind.as_str() {
            "npc" => {
                out.insert("character".into(), json::s(d.resolve(&gs.world_id, "characters", &json::s(&m, "character")), "_qid").into());
            }
            "lore" => {
                let lore = d.resolve(&gs.world_id, "lore", &json::s(&m, "lore"));
                out.insert("lore".into(), json::s(lore, "_qid").into());
                out.insert("name".into(), json::s(lore, "name").into());
                out.insert("text".into(), json::s(lore, "text").into());
            }
            _ => {
                if is_done(gs, &loc_qid, &m) {
                    out.insert("empty".into(), Value::Bool(true));
                    return out;
                }
                let opened = open_chest(d, gs, &loc_qid, &m);
                if !json::b(&opened, "ok", false) {
                    return opened;
                }
                for (k, v) in opened {
                    out.insert(k, v);
                }
            }
        }
        let flags = apply_sets(d, gs, &m);
        out.insert("flags".into(), json::strs(&flags));
        return out;
    }
    nothing(true, &[])
}

fn open_chest(d: &Data, gs: &mut GameState, loc_qid: &str, m: &Dict) -> Dict {
    let mut items = json::ints(json::get(m, "items"));
    let mut loot_qid = String::new();
    let loot_ref = json::s(m, "loot");
    if !loot_ref.is_empty() {
        let loot = d.resolve(&gs.world_id, "loot", &loot_ref);
        let lqid = json::s(loot, "_qid");
        if !lqid.is_empty() && !gs.loot_claimed.contains(&lqid) && quest::all_hold(gs, json::get(loot, "requires_flags")) {
            *items.entry(json::s(loot, "ref")).or_insert(0) += json::i(loot, "count", 1);
            loot_qid = lqid;
        }
    }
    let gold = json::i(m, "gold", 0).max(0);
    let mut items_obj = Dict::new();
    for (k, v) in &items {
        items_obj.insert(k.clone(), int(*v));
    }
    let rewards = json::obj([("items", Value::Object(items_obj.clone())), ("gold", int(gold))]);
    let granted = quest::reward_apply(d, gs, &rewards, "", &[]);
    if !json::b(&granted, "ok", false) {
        let mut out = json::ok_result(false, &json::sget(&granted, "errors"));
        out.insert("kind".into(), "chest".into());
        return out;
    }
    if !loot_qid.is_empty() {
        quest::claim_loot(gs, &loot_qid);
    }
    let want = json::i(m, "supplies", 0).max(0);
    let added = gs.party.add_supplies(want);
    quest::set_flag(d, gs, &done_flag(loc_qid, &json::s(m, "id")), Value::Bool(true));
    let mut out = json::obj([
        ("ok", Value::Bool(true)),
        ("empty", Value::Bool(false)),
        ("items", Value::Object(items_obj)),
        ("loot", loot_qid.into()),
        ("gold", int(json::i(json::dget(&granted, "granted"), "gold", 0))),
        ("supplies", int(added)),
    ]);
    if added < want {
        out.insert("notes".into(), json::strs(&[format!("Supplies are capped at {SUPPLIES_MAX} ({} left behind).", want - added)]));
    }
    out
}
