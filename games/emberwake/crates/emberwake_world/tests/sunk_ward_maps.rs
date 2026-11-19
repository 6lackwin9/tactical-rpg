//! SUNKWARD-MAPS1 (PR5-55) on the shipped data, read only: the 9 Act 1 Sunk Ward region maps, walked through the real
//! explore path (`explore::enter`, `explore::step`, `explore::interact`). For every map-bound `sunk_ward` location:
//! the map is well formed (equal rows, a zone grid of the same size whose letters the legend names, anchors on the
//! map); every arrival anchor (`spawn`, where the world map and the scenes put the party, and each `to_anchor` an exit
//! elsewhere names) stands on an open tile; from each arrival every marker the player uses is reached on foot with the
//! map's own rules (open `.` tiles, a present npc / chest / lore marker blocks, as `explore::step` and the region
//! screen's walk-to agree): a person, chest or lore marker from a tile beside it (`explore::interact` facing it), an
//! exit, point or encounter by stepping onto it; each roamer's patrol stays on open tiles one step apart and names an
//! encounter that resolves; and each roofed indoor zone (REVEAL1) is entered on foot, `explore::indoor_zone_at` names
//! it there, so the region screen's fade target (the zone under the party) is that roof's `roof_of`. Expectations
//! come from the rules and the location's own markers, never from shipped numbers.

use aegis_core::json::{self, Dict};
use emberwake_world::explore::{self, Pos};
use emberwake_world::state::PartyState;
use emberwake_world::{bunks, new_game, Data, GameState};
use serde_json::Value;
use std::collections::{HashSet, VecDeque};
use std::path::Path;

const WORLD: &str = "sunk_ward";

/// The 9 locations PR5-55 binds a map on (the row names them).
const LOCATIONS: [&str; 9] = [
    "sluice_undercroft",
    "drowned_runoff_grate",
    "corvos_gate",
    "slag_gate",
    "alleyway_junction_7",
    "ash_crawl_crossroads",
    "fourth_kiln",
    "tallow_vats",
    "crucible_watchpost",
];

/// The marker kinds that stand solid on the map and are used from a tile beside them (`explore` SOLID_KINDS).
const SOLID: [&str; 3] = ["npc", "chest", "lore"];

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("data").0
}

/// A `sunk_ward` game whose party is the world's first playable party character (picked by rule).
fn game(d: &Data) -> GameState {
    let m = d
        .catalogs
        .own_entries(WORLD, "characters")
        .iter()
        .filter(|c| json::s(c, "role") == "party")
        .find_map(|c| new_game::hero_from_character(d, WORLD, &json::s(c, "id")))
        .expect("a playable party character in sunk_ward");
    GameState::create(d, WORLD, PartyState { active_ids: vec![m.id.clone()], members: vec![m], ..PartyState::default() })
}

fn ok(r: &Dict) -> bool {
    json::b(r, "ok", false)
}

fn kinds(r: &Dict) -> Vec<String> {
    json::aget(r, "events").iter().map(|e| json::s(json::dict(e), "kind")).collect()
}

/// The location `id` with its template, resolved as the explore path does.
fn location_and_map(d: &Data, gs: &GameState, id: &str) -> (Dict, Dict) {
    let loc = explore::resolve_location(d, gs, id).clone();
    assert!(!loc.is_empty(), "{WORLD} has the location {id}");
    let t = explore::template_of(d, &loc).clone();
    assert!(!t.is_empty(), "{id}: its map '{}' loads", json::s(&loc, "map"));
    (loc, t)
}

/// Problems with the template's shape: rows, the zone grid, the legend, anchors and paths on the map.
fn shape_problems(t: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let rows = explore::rows_of(t);
    let (w, h) = explore::size_of(t);
    if w == 0 || h == 0 {
        out.push("the map has no rows".to_string());
        return out;
    }
    if let Some((y, r)) = rows.iter().enumerate().find(|(_, r)| r.len() as i64 != w) {
        out.push(format!("row {y} is {} wide, not {w}", r.len()));
    }
    let legend = json::dget(t, "zone_legend");
    let zones = json::aget(t, "zones");
    if !zones.is_empty() {
        if zones.len() as i64 != h {
            out.push(format!("the zone grid has {} rows, not {h}", zones.len()));
        }
        for (y, z) in zones.iter().enumerate() {
            let z = json::gstr(z);
            if z.chars().count() as i64 != w {
                out.push(format!("zone row {y} is {} wide, not {w}", z.chars().count()));
            }
            if let Some(c) = z.chars().find(|c| *c != '-' && !legend.contains_key(&c.to_string())) {
                out.push(format!("zone row {y} uses '{c}', which the legend does not name"));
            }
        }
    }
    for (k, z) in legend {
        let z = json::dict(z);
        if json::s(z, "id").is_empty() || !["safe", "danger"].contains(&json::s(z, "kind").as_str()) {
            out.push(format!("legend '{k}' needs an id and a kind safe / danger"));
        }
    }
    let on_map = |p: Pos| p.0 >= 0 && p.1 >= 0 && p.0 < w && p.1 < h;
    for a in json::aget(t, "anchors").iter().map(json::dict) {
        let p = (json::i(a, "x", -1), json::i(a, "y", -1));
        if !on_map(p) {
            out.push(format!("anchor '{}' at {p:?} is off the map", json::s(a, "id")));
        }
    }
    for path in json::dget(t, "paths").keys() {
        if let Some(p) = explore::path_points(t, path).into_iter().find(|p| !on_map(*p)) {
            out.push(format!("path '{path}' point {p:?} is off the map"));
        }
    }
    out
}

/// Where the party arrives in location `id`: `spawn` (the world map and the scenes) and each `to_anchor` of an exit
/// marker in any `sunk_ward` location that leads here.
fn arrivals(d: &Data, gs: &GameState, id: &str) -> Vec<String> {
    let qid = json::s(explore::resolve_location(d, gs, id), "_qid");
    let mut out = vec![explore::SPAWN.to_string()];
    for e in d.catalogs.own_entries(WORLD, "locations") {
        let from = explore::resolve_location(d, gs, &json::s(e, "id"));
        for m in explore::markers_of(from).iter().filter(|m| json::s(m, "kind") == "exit") {
            let to = explore::resolve_location(d, gs, &json::s(m, "to"));
            let anchor = json::s_or(m, "to_anchor", explore::SPAWN);
            if !to.is_empty() && json::s(to, "_qid") == qid && !out.contains(&anchor) {
                out.push(anchor);
            }
        }
    }
    out
}

/// A marker the player uses, and where it stands.
struct Target {
    id: String,
    kind: String,
    at: Pos,
    /// Used from a tile beside it (a solid marker), else by stepping onto it.
    beside: bool,
}

/// Every non-roamer marker of the location (people incl. the flavour citizens, chests, lore, points, exits and the
/// mission encounters).
fn targets(loc: &Dict, t: &Dict) -> Vec<Target> {
    explore::markers_of(loc)
        .iter()
        .filter(|m| json::s(m, "kind") != "roamer")
        .map(|m| {
            let kind = json::s(m, "kind");
            Target { id: json::s(m, "id"), at: explore::marker_pos(t, m, 0), beside: SOLID.contains(&kind.as_str()), kind }
        })
        .collect()
}

/// The cells a walk avoids. Always: every solid marker (whatever its flags, so a route found holds in any story
/// state). `strict` adds every exit, point and encounter cell (stepping there leaves, sets flags or starts a fight);
/// otherwise only exits (a point or a mission fight can be crossed). The target's own cell is never avoided.
fn avoided(loc: &Dict, t: &Dict, target: Pos, strict: bool) -> HashSet<Pos> {
    explore::markers_of(loc)
        .iter()
        .filter(|m| {
            let kind = json::s(m, "kind");
            SOLID.contains(&kind.as_str()) || kind == "exit" || (strict && (kind == "point" || kind == "encounter"))
        })
        .map(|m| explore::marker_pos(t, m, 0))
        .filter(|p| *p != target)
        .collect()
}

/// The region screen's walk-to rule (BFS over open tiles, 4 directions): the step directions from `start` to a cell
/// `goal` accepts; None when no open route gets there.
fn route(t: &Dict, start: Pos, avoid: &HashSet<Pos>, goal: impl Fn(Pos) -> bool) -> Option<Vec<Pos>> {
    let mut prev: std::collections::HashMap<Pos, (Pos, Pos)> = std::collections::HashMap::new();
    let mut queue = VecDeque::from([start]);
    let mut seen = HashSet::from([start]);
    while let Some(cur) = queue.pop_front() {
        if goal(cur) {
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
            if seen.contains(&n) || !explore::is_open(t, n.0, n.1) || avoid.contains(&n) {
                continue;
            }
            seen.insert(n);
            prev.insert(n, (cur, dir));
            queue.push_back(n);
        }
    }
    None
}

/// A route to target `tg` from `start`: around every other marker first, else crossing points and mission fights.
fn route_to(loc: &Dict, t: &Dict, start: Pos, tg: &Target) -> Option<Vec<Pos>> {
    let goal = |p: Pos| if tg.beside { (p.0 - tg.at.0).abs() + (p.1 - tg.at.1).abs() == 1 } else { p == tg.at };
    route(t, start, &avoided(loc, t, tg.at, true), goal).or_else(|| route(t, start, &avoided(loc, t, tg.at, false), goal))
}

/// Sets the flags marker `m` needs to be present (its `requires_flags` hold, its `unless_flags` do not).
fn make_present(d: &Data, gs: &mut GameState, m: &Dict) {
    for c in json::arr(&json::normalize(json::get(m, "requires_flags"))) {
        let c = json::dict(c);
        let err = emberwake_world::quest::set_flag(d, gs, &json::s(c, "key"), c.get("value").cloned().unwrap_or(Value::Bool(true)));
        assert!(err.is_empty(), "set {}: {err}", json::s(c, "key"));
    }
    for c in json::arr(&json::normalize(json::get(m, "unless_flags"))) {
        let c = json::dict(c);
        let key = json::s(c, "key");
        match c.get("value") {
            Some(Value::Bool(b)) => {
                let err = emberwake_world::quest::set_flag(d, gs, &key, Value::Bool(!b));
                assert!(err.is_empty(), "set {key}: {err}");
            }
            _ => {
                gs.flags.shift_remove(&key);
            }
        }
    }
    // PR7-41 (PT-15): a job's fight is offered once its job is accepted, so accept every quest naming the fight
    let enc = json::s(m, "encounter");
    if !enc.is_empty() {
        for e in d.merged_entries(WORLD, "quests") {
            let qid = json::s(e, "_qid");
            let names = json::aget(&emberwake_world::quest::resolved(d, gs, &qid), "steps").iter().any(|s| json::s(json::dict(s), "encounter") == enc);
            if names && !gs.quests.contains_key(&qid) {
                gs.quests.insert(qid, serde_json::json!({"state": "active", "objective": 0}));
            }
        }
    }
}

/// Walks `dirs` with `explore::step` from the party's tile; the last step is the use for a stepped-on target. Returns
/// why the walk failed, if it did.
fn walk(d: &Data, gs: &mut GameState, dirs: &[Pos], last_is_use: bool) -> Result<Option<Dict>, String> {
    for (i, dir) in dirs.iter().enumerate() {
        let from = gs.position;
        let r = explore::step(d, gs, *dir);
        if !ok(&r) {
            return Err(format!("step {i} {dir:?} from {from:?} failed: {:?}", json::sget(&r, "errors")));
        }
        let last = i + 1 == dirs.len();
        if last && last_is_use {
            return Ok(Some(r));
        }
        if !kinds(&r).iter().any(|k| k == "moved") || gs.position != (from.0 + dir.0, from.1 + dir.1) {
            return Err(format!("step {i} {dir:?} from {from:?} did not move: {:?}", kinds(&r)));
        }
    }
    Ok(None)
}

/// A fresh game in location `id` at anchor `anchor`, with `m` (when given) made present.
fn entered(d: &Data, id: &str, anchor: &str, m: Option<&Dict>) -> GameState {
    let mut gs = game(d);
    let r = explore::enter(d, &mut gs, id, anchor);
    assert!(ok(&r), "{id}: entering at '{anchor}' works: {:?}", json::sget(&r, "errors"));
    if let Some(m) = m {
        make_present(d, &mut gs, m);
    }
    gs
}

/// Walks to target `tg` from arrival `anchor` on a fresh game and uses it; "" when that worked, else why not.
fn reach_and_use(d: &Data, id: &str, anchor: &str, loc: &Dict, t: &Dict, tg: &Target) -> String {
    let m = explore::markers_of(loc).into_iter().find(|m| json::s(m, "id") == tg.id).expect("the marker");
    let mut gs = entered(d, id, anchor, Some(&m));
    let lq = json::s(loc, "_qid");
    let state = explore::presence(d, &gs, &lq, &m);
    if state != "present" {
        return format!("is {state} with its flags set");
    }
    let start = gs.position;
    let Some(dirs) = route_to(loc, t, start, tg) else {
        return format!("no open route from {start:?} to {}{:?}", if tg.beside { "a tile beside " } else { "" }, tg.at);
    };
    if tg.beside {
        if let Err(e) = walk(d, &mut gs, &dirs, false) {
            return e;
        }
        let face = (tg.at.0 - gs.position.0, tg.at.1 - gs.position.1);
        let r = explore::interact(d, &mut gs, face);
        if !ok(&r) || json::s(&r, "marker") != tg.id || json::s(&r, "kind") != tg.kind {
            return format!("interact {face:?} from {:?} gave kind '{}' marker '{}'", gs.position, json::s(&r, "kind"), json::s(&r, "marker"));
        }
        return String::new();
    }
    if dirs.is_empty() {
        return format!("the arrival tile {start:?} is the marker itself");
    }
    let r = match walk(d, &mut gs, &dirs, true) {
        Err(e) => return e,
        Ok(r) => r.expect("the last step"),
    };
    let events = json::aget(&r, "events");
    let has = |kind: &str| events.iter().map(json::dict).any(|e| json::s(e, "kind") == kind && json::s(e, "marker") == tg.id);
    let moved_on = gs.position == tg.at || kinds(&r).iter().any(|k| k == "exit");
    let used = match tg.kind.as_str() {
        "exit" => has("exit"),
        "encounter" => events.iter().map(json::dict).any(|e| json::s(e, "kind") == "fight" && json::s(e, "marker") == tg.id && !json::s(e, "encounter").is_empty()),
        _ => moved_on && kinds(&r).iter().any(|k| k == "moved"),
    };
    if used {
        String::new()
    } else {
        format!("stepping onto {:?} gave {:?}", tg.at, kinds(&r))
    }
}

#[test]
fn every_sunk_ward_location_binds_a_well_formed_map() {
    let d = data();
    let gs = game(&d);
    let bound: Vec<String> = d.catalogs.own_entries(WORLD, "locations").iter().filter(|e| !json::s(e, "map").is_empty()).map(|e| json::s(e, "id")).collect();
    for id in LOCATIONS {
        assert!(bound.iter().any(|b| b == id), "{id} binds a map");
        let (loc, t) = location_and_map(&d, &gs, id);
        let problems = shape_problems(&t);
        assert!(problems.is_empty(), "{id} ({}): {problems:?}", json::s(&loc, "map"));
        for anchor in arrivals(&d, &gs, id) {
            let p = explore::anchor_pos(&t, &anchor);
            assert!(explore::is_open(&t, p.0, p.1), "{id}: arrival '{anchor}' at {p:?} is an open tile");
            let on = explore::markers_of(&loc).into_iter().find(|m| SOLID.contains(&json::s(m, "kind").as_str()) && explore::marker_pos(&t, m, 0) == p);
            assert!(on.is_none(), "{id}: arrival '{anchor}' is not under a person, chest or lore marker");
            assert!(explore::entry_problem(&d, &gs, id, &anchor).is_empty(), "{id}: '{anchor}' is enterable");
        }
        // every marker but a roamer names an anchor on the map
        for m in explore::markers_of(&loc).iter().filter(|m| json::s(m, "kind") != "roamer") {
            let p = explore::marker_pos(&t, m, 0);
            assert_ne!(p, emberwake_world::state::NO_POSITION, "{id}: marker '{}' anchor '{}' is on the map", json::s(m, "id"), json::s(m, "anchor"));
        }
    }
}

#[test]
fn every_marker_is_reached_on_foot_from_every_arrival() {
    let d = data();
    let gs = game(&d);
    let mut failures = Vec::new();
    for id in LOCATIONS {
        let (loc, t) = location_and_map(&d, &gs, id);
        let tgs = targets(&loc, &t);
        assert!(!tgs.is_empty(), "{id} has markers");
        let anchors = arrivals(&d, &gs, id);
        for anchor in &anchors {
            for tg in &tgs {
                let why = reach_and_use(&d, id, anchor, &loc, &t, tg);
                if !why.is_empty() {
                    failures.push(format!("{id}: from '{anchor}', {} '{}' at {:?}: {why}", tg.kind, tg.id, tg.at));
                }
            }
        }
    }
    assert!(failures.is_empty(), "markers not reached on foot:\n{}", failures.join("\n"));
}

#[test]
fn roamers_patrol_open_tiles_and_name_real_encounters() {
    let d = data();
    let gs = game(&d);
    let mut roamers = 0;
    for id in LOCATIONS {
        let (loc, t) = location_and_map(&d, &gs, id);
        let spawn = explore::anchor_pos(&t, explore::SPAWN);
        for m in explore::markers_of(&loc).iter().filter(|m| json::s(m, "kind") == "roamer") {
            roamers += 1;
            let mid = json::s(m, "id");
            let enc = json::s(m, "encounter");
            assert!(!json::s(d.resolve(WORLD, "encounters", &enc), "_qid").is_empty(), "{id}: roamer '{mid}' encounter '{enc}' resolves");
            let pts = explore::path_points(&t, &json::s(m, "path"));
            assert!(pts.len() >= 2, "{id}: roamer '{mid}' walks a path of 2+ points");
            for (i, p) in pts.iter().enumerate() {
                assert!(explore::is_open(&t, p.0, p.1), "{id}: roamer '{mid}' point {i} {p:?} is an open tile");
            }
            for w in pts.windows(2) {
                assert_eq!((w[0].0 - w[1].0).abs() + (w[0].1 - w[1].1).abs(), 1, "{id}: roamer '{mid}' steps one tile at a time ({:?} -> {:?})", w[0], w[1]);
            }
            // where it stands at any step is on its path, and the party can walk to it from spawn
            for steps in 0..(2 * pts.len() as i64) {
                assert!(pts.contains(&explore::marker_pos(&t, m, steps)), "{id}: roamer '{mid}' stays on its path at step {steps}");
            }
            let avoid = avoided(&loc, &t, (-1, -1), false);
            assert!(route(&t, spawn, &avoid, |p| pts.contains(&p)).is_some(), "{id}: roamer '{mid}' patrols tiles the party reaches from spawn");
        }
    }
    assert!(roamers > 0, "the Sunk Ward maps hold roamers");
}

#[test]
fn indoor_zones_are_entered_on_foot_and_fade_their_own_roofs() {
    let d = data();
    let gs = game(&d);
    let mut indoor = 0;
    for id in LOCATIONS {
        let (loc, t) = location_and_map(&d, &gs, id);
        let roofs: Vec<String> = json::aget(&t, "props").iter().filter_map(|p| explore::roof_zone(json::dict(p))).collect();
        let zones: Vec<String> = json::dget(&t, "zone_legend").values().map(|z| json::s(json::dict(z), "id")).filter(|z| explore::zone_indoor(&t, z)).collect();
        for r in &roofs {
            assert!(zones.contains(r), "{id}: roof of '{r}' names an indoor zone of the legend");
        }
        let (w, h) = explore::size_of(&t);
        let spawn = explore::anchor_pos(&t, explore::SPAWN);
        let outside = explore::indoor_zone_at(&t, spawn.0, spawn.1);
        for zone in &zones {
            if !roofs.contains(zone) {
                continue;
            }
            indoor += 1;
            let cells: Vec<Pos> = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).filter(|p| explore::indoor_zone_at(&t, p.0, p.1) == *zone).collect();
            assert!(cells.iter().any(|p| explore::is_open(&t, p.0, p.1)), "{id}: indoor '{zone}' has an open tile");
            let avoid = avoided(&loc, &t, (-1, -1), true);
            let dirs = route(&t, spawn, &avoid, |p| cells.contains(&p)).unwrap_or_else(|| panic!("{id}: indoor '{zone}' is reached on foot from spawn"));
            let mut g = entered(&d, id, explore::SPAWN, None);
            if let Err(e) = walk(&d, &mut g, &dirs, false) {
                panic!("{id}: walking into '{zone}': {e}");
            }
            let inside = explore::indoor_zone_at(&t, g.position.0, g.position.1);
            assert_eq!(&inside, zone, "{id}: standing at {:?} the party is inside '{zone}'", g.position);
            // the fade target (REVEAL1 1.3) is the zone under the party: inside it that is this roof's `roof_of`, and at
            // spawn no roof's zone is under the party
            for r in &roofs {
                assert!(r != &outside || outside.is_empty(), "{id}: at spawn no roof of '{r}' fades");
            }
        }
        // a flavour citizen `npc_<zone>` of an indoor zone is used from inside its building
        for m in explore::markers_of(&loc).iter().filter(|m| json::s(m, "kind") == "npc") {
            let mid = json::s(m, "id");
            let Some(zone) = mid.strip_prefix("npc_").filter(|z| zones.iter().any(|x| x == z)) else { continue };
            let at = explore::marker_pos(&t, m, 0);
            let near = std::iter::once(at).chain(explore::DIRS.iter().map(|d| (at.0 + d.0, at.1 + d.1)));
            assert!(near.into_iter().any(|p| explore::indoor_zone_at(&t, p.0, p.1) == zone), "{id}: '{mid}' at {at:?} stands in or at its building '{zone}'");
        }
    }
    assert!(indoor > 0, "the Sunk Ward maps hold roofed indoor zones");
}

/// PR5-58 (1), PR7-23 (HUB-LAYOUT1): the `38 x 24` 7-room `sluice_undercroft` hub carries indoor + wall_set room
/// styles (the Art/UI layout `art_src/region_maps/sluice_undercroft.json`; PR7-36 / AI-23: basin, hearth, hall, entry and
/// hollows in the `drain_brick` interior walls, the crypts in stone) with `npc_c`
/// (`talia_rusk`) at `(15, 11)` and `lore_a` (`ivo`) at `(12, 11)` in the hall, all reachable from `spawn` and `gate_in`.
#[test]
fn hub_7_rooms_styles_talia_and_ivo_stay_reachable() {
    let d = data();
    let gs = game(&d);
    let (loc, t) = location_and_map(&d, &gs, "sluice_undercroft");
    assert_eq!(json::s(&loc, "map"), "sunk_ward_undercroft");
    assert_eq!(explore::size_of(&t), (38, 24));
    for (zone, want_set) in [
        ("slab", "interior_plaster"),
        ("hollows", "drain_brick"),
        ("basin", "drain_brick"),
        ("hearth", "drain_brick"),
        ("hall", "drain_brick"),
        ("entry", "drain_brick"),
        ("crypts", "interior_stone"),
    ] {
        assert!(explore::zone_indoor(&t, zone), "hub zone '{zone}' is indoor");
        assert_eq!(explore::zone_wall_set(&t, zone), want_set, "hub zone '{zone}' wall_set");
    }
    assert_eq!(explore::anchor_pos(&t, "npc_c"), (15, 11), "PR7-23 Talia anchor in the hall at (15, 11)");
    assert_eq!(explore::anchor_pos(&t, "lore_a"), (12, 11), "PR7-23 Ivo anchor in the hall at (12, 11)");
    let tgs = targets(&loc, &t);
    for anchor in ["spawn", "gate_in"] {
        // FOOTLOCKER1 (`PR7-51`): the stash chest left the hub (the bunks' footlockers open the loadout and the stash)
        for wanted in ["talia", "ivo", "rest", "crypts"] {
            let tg = tgs.iter().find(|tg| tg.id == wanted).unwrap_or_else(|| panic!("sluice_undercroft has marker '{wanted}'"));
            let why = reach_and_use(&d, "sluice_undercroft", anchor, &loc, &t, tg);
            assert!(why.is_empty(), "sluice_undercroft '{wanted}' from '{anchor}': {why}");
        }
    }
    let stash: Vec<String> = explore::markers_of(&loc).iter().filter(|m| explore::point_use(m) == "stash").map(|m| json::s(m, "id")).collect();
    assert!(stash.is_empty(), "FOOTLOCKER1: no stash point on the hub map: {stash:?}");
}

/// PR5-58 (2)-(3) & FB-272(a-e): `slag_gate -> crucible_watchpost` exit lands at open `postern_in`, every danger zone
/// resolves a bound encounter, district scavengers split from `enc_scav_alley`, `ground_legend` has no unused keys,
/// every anchor is wired, `corvos_gate` `rubble` leaves `breach` open, and `_side` props match East/West walls.
#[test]
fn fb272_exits_danger_bindings_side_props_and_split_encounters() {
    let d = data();
    let gs = game(&d);

    let (slag_loc, slag_map) = location_and_map(&d, &gs, "slag_gate");
    let cw_exit = explore::markers_of(&slag_loc).into_iter().find(|m| json::s(m, "kind") == "exit" && json::s(m, "to") == "crucible_watchpost").expect("slag_gate has exit to crucible_watchpost");
    assert_eq!(json::s(&cw_exit, "to_anchor"), "postern_in", "slag_gate exit targets crucible_watchpost postern_in");
    let (cw_loc, cw_map) = location_and_map(&d, &gs, "crucible_watchpost");
    let postern_in = explore::anchor_pos(&cw_map, "postern_in");
    assert!(explore::is_open(&cw_map, postern_in.0, postern_in.1), "crucible_watchpost postern_in is open");
    assert!(explore::markers_of(&cw_loc).iter().all(|m| explore::marker_pos(&cw_map, m, 0) != postern_in), "postern_in is free of markers");
    let slag_spawn = explore::anchor_pos(&slag_map, explore::SPAWN);
    let slag_exit_pos = explore::marker_pos(&slag_map, &cw_exit, 0);
    assert!(route(&slag_map, slag_spawn, &avoided(&slag_loc, &slag_map, slag_exit_pos, false), |p| p == slag_exit_pos).is_some(), "slag_gate exit to crucible_watchpost reachable from spawn");

    let (_, cg_map) = location_and_map(&d, &gs, "corvos_gate");
    let rubble = json::aget(&cg_map, "props").iter().map(json::dict).find(|p| json::s(p, "id") == "rubble").expect("corvos_gate rubble prop");
    let (rx, ry, rw, rh) = (json::i(rubble, "x", -1), json::i(rubble, "y", -1), json::i(rubble, "w", 0), json::i(rubble, "h", 0));
    assert!(!(rx <= 5 && 5 < rx + rw && ry <= 10 && 10 < ry + rh), "rubble ({rx},{ry},{rw}x{rh}) must not overlap breach approach (5,10)");
    assert!(explore::is_open(&cg_map, 5, 10), "corvos_gate breach approach (5, 10) is open");

    let mut inbound_anchors: HashSet<(String, String)> = HashSet::new();
    for id in LOCATIONS {
        let loc = explore::resolve_location(&d, &gs, id);
        for m in explore::markers_of(loc).iter().filter(|m| json::s(m, "kind") == "exit") {
            let to = explore::resolve_location(&d, &gs, &json::s(m, "to"));
            let to_id = json::s(to, "id");
            let ta = json::s(m, "to_anchor");
            if !to_id.is_empty() && !ta.is_empty() {
                inbound_anchors.insert((to_id, ta));
            }
        }
    }

    let mut side_props = 0usize;
    let mut non_alley_enc_scav_alley = Vec::new();
    for id in LOCATIONS {
        let (loc, t) = location_and_map(&d, &gs, id);
        let bindings = json::dget(&loc, "zone_bindings");
        for zv in json::dget(&t, "zone_legend").values().map(json::dict) {
            if json::s(zv, "kind") == "danger" {
                let zid = json::s(zv, "id");
                let bound = bindings.get(&zid).and_then(|v| v.as_object()).unwrap_or_else(|| panic!("{id}: danger zone '{zid}' lacks zone_bindings"));
                let encs = json::aget(bound, "encounters");
                assert!(!encs.is_empty(), "{id}: danger zone '{zid}' has no encounters");
                for row in encs {
                    let eid = row.as_array().and_then(|a| a.first()).map(json::gstr).unwrap_or_default();
                    assert!(!json::s(d.resolve(WORLD, "encounters", &eid), "_qid").is_empty(), "{id}.{zid}: missing encounter '{eid}'");
                    if eid == "enc_scav_alley" && id != "alleyway_junction_7" {
                        non_alley_enc_scav_alley.push(format!("{id}.{zid}"));
                    }
                }
            }
        }

        let ground_tiles: String = json::aget(&t, "ground").iter().map(json::gstr).collect();
        for key in json::dget(&t, "ground_legend").keys() {
            assert!(ground_tiles.contains(key), "{id}: ground_legend key '{key}' is unused");
        }

        let used_marker_anchors: HashSet<String> = explore::markers_of(&loc).iter().map(|m| json::s(m, "anchor")).filter(|s| !s.is_empty()).collect();
        for a in json::aget(&t, "anchors").iter().map(json::dict) {
            let aid = json::s(a, "id");
            // BK9: `idle_*` anchors are the hub's idle spots (no marker uses them); each stands on an open cell
            if aid.starts_with(bunks::IDLE_ANCHOR_PREFIX) {
                let (ax, ay) = (json::i(a, "x", -1), json::i(a, "y", -1));
                assert!(explore::is_open(&t, ax, ay), "{id}: idle anchor '{aid}' at ({ax}, {ay}) is open");
                continue;
            }
            let wired = aid == explore::SPAWN
                || aid == "patrol_start"
                || (id == "tallow_vats" && aid == "siphon_in")
                || used_marker_anchors.contains(&aid)
                || inbound_anchors.contains(&(id.to_string(), aid.clone()));
            assert!(wired, "{id}: unused anchor '{aid}'");
        }

        let rows = explore::rows_of(&t);
        for p in json::aget(&t, "props").iter().map(json::dict) {
            if json::s(p, "art").ends_with("_side") {
                side_props += 1;
                let (x, y, w, h) = (json::i(p, "x", -1), json::i(p, "y", -1), json::i(p, "w", 0), json::i(p, "h", 0));
                assert_eq!(w, 1, "{id} side prop '{}' has width 1", json::s(p, "id"));
                let flip = json::b(p, "flip", false);
                for dy in 0..h {
                    let row = &rows[(y + dy) as usize];
                    assert_eq!(row[x as usize], '#', "{id} side prop '{}' must stand on wall tile", json::s(p, "id"));
                    let face_x = if flip { x - 1 } else { x + 1 };
                    assert_eq!(row[face_x as usize], '.', "{id} side prop '{}' (flip={flip}) must face open tile at x={face_x}", json::s(p, "id"));
                }
            }
        }
    }
    assert!(non_alley_enc_scav_alley.is_empty(), "FB-272(e): shared enc_scav_alley outside alleyway_junction_7: {non_alley_enc_scav_alley:?}");
    assert!(side_props >= 20, "Sunk Ward maps bind side-profile props ({side_props})");
}
