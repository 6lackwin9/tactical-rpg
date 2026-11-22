//! UISPLIT1-I (ENGINE-23) Stage A: the world map's view-model (`worldmap::WorldMap`). What the screen shows and offers,
//! as a `screen_layout::View`, for a layout file to place later (Stage B, its own row once the mock is signed off);
//! today's hand drawing stays (the map's marks, roads, Legend and side panel). Its fixture is
//! `ui/views/world_map.view.json` (`tests::the_fixture_has_the_view_shape`).
//!
//! Fields: `title` ("World map"); `here` (the party's place, the HUD bar's name); `time` ("Time N"); `places` (a row per
//! place the map shows, id its qid: its name, its region line as the sub ("(not visited)" until been to), its first
//! hover line as the detail (the party here, the trip's cost, or no road) toned `info` or `dim`, "Quest" as the second
//! detail in `gold` while it has a quest or job badge, its landmark (or pin) picture, the rest of its hover lines as
//! the note, chosen while picked, off while locked); `trip` (the Travel button's hover line for the chosen way, or
//! "No open road to X." in `foe`, "" with no pick); `hint` (the next step toward a pick with no way, else ""); `ways`
//! (a row per way to the pick, id its index: its label, days and upkeep as the sub, its hover line as the note, chosen
//! while picked); `quests` (the Legend's rows, id the place ("" for none): the quest or job, its hover line as the
//! note, on while the map shows its place); `upkeep` (the Legend's warning at 0 upkeep in `foe`, else ""); `log` (the
//! last 16 log lines, oldest first). Actions: `place` (a place's qid as arg), `way` (a way's index as arg), `travel`
//! ("Travel to X", on while the pick has a way), `enter` ("Enter X", on at a place one can enter), `camp` ("Make camp
//! (n Tallow)", on while the party can camp), `mode` ("Travel: X", on while the world has two modes or more; off and
//! "Travel mode" without modes) and `back`.

use crate::screens::Cx;
use crate::worldmap::{camp_line, legend_about, legend_rows, mode_line, place_lines, place_region, trip_text, upkeep_line, way_note, WAct, WorldMap};
use aegis_core::json::{self, s};
use aegis_ui::screen_layout::{Act, Field, RowView, View};
use emberwake_world::{explore, travel};

pub const SCREEN: &str = "world_map";
pub const PLACE: &str = "place";
pub const WAY: &str = "way";
pub const TRAVEL: &str = "travel";
pub const ENTER: &str = "enter";
pub const CAMP: &str = "camp";
pub const MODE: &str = "mode";
pub const BACK: &str = "back";

/// How many log lines the view carries (the side panel's).
const LOG_LINES: usize = 16;

/// The view of map `wm` on the game `cx` holds.
pub fn view(cx: &Cx, wm: &WorldMap) -> View {
    let (d, gs) = (&cx.data, cx.gs());
    let here = gs.location_id.clone();
    let tv = travel::view(d, gs);
    let nodes: Vec<json::Dict> = json::aget(&tv, "nodes").iter().map(|n| json::dict(n).clone()).collect();
    let places: Vec<RowView> = nodes.iter().map(|n| place_row(cx, wm, &here, n)).collect();
    let ways = wm.ways(cx);
    let pick = ways.get(wm.pick);
    let mode = emberwake_world::campaign::purse::travel_mode(d, gs);
    let to = if wm.selected.is_empty() { String::new() } else { cx.location_name(&gs.world_id, &wm.selected) };
    let (trip, trip_tone, hint) = match (wm.selected.is_empty(), pick) {
        (true, _) => (String::new(), "", String::new()),
        (false, Some(w)) => (trip_text(cx, gs, w, mode.as_ref()), "", String::new()),
        (false, None) => (format!("No open road to {to}."), "foe", crate::worldmap::way_hint(cx, gs, &here, &wm.selected)),
    };
    let way_rows: Vec<RowView> = ways
        .iter()
        .enumerate()
        .map(|(i, w)| RowView { id: i.to_string(), label: w.label.clone(), sub: crate::worldmap::cost_text(cx, gs, w.days, w.supplies), note: way_note(cx, gs, w), selected: Some(i) == pick.map(|_| wm.pick), enabled: true, ..Default::default() })
        .collect();
    let (legend, quest_of) = legend_rows(d, gs, &nodes);
    let quests: Vec<RowView> = legend
        .iter()
        .enumerate()
        .map(|(i, (node, text))| {
            let shown = nodes.iter().any(|n| s(n, "qid") == *node);
            RowView {
                id: node.clone(),
                label: text.clone(),
                note: legend_about(cx, gs, &here, node, shown, quest_of.get(i).map(String::as_str)),
                selected: !node.is_empty() && *node == wm.selected,
                enabled: shown,
                ..Default::default()
            }
        })
        .collect();
    let upkeep = upkeep_line(cx, gs).unwrap_or_default();
    let log: Vec<RowView> = cx.log.iter().rev().take(LOG_LINES).rev().map(|l| RowView { label: l.clone(), enabled: true, ..Default::default() }).collect();
    let (camp_label, _, camp_on) = camp_line(cx, d, gs);
    let (mode_label, _, many) = mode_line(cx, gs).unwrap_or(("Travel mode".into(), String::new(), false));
    let here_name = cx.location_name(&gs.world_id, &here);
    View::new(SCREEN)
        .field("title", Field::Text("World map".into()))
        .field("here", Field::Text(here_name.clone()))
        .field("time", Field::Text(format!("Time {}", gs.time)))
        .field("places", Field::Rows(places))
        .field("trip", Field::Text(trip.clone()))
        .tone("trip", &[if trip.is_empty() { "" } else { trip_tone }])
        .field("hint", Field::Text(hint))
        .field("ways", Field::Rows(way_rows))
        .field("quests", Field::Rows(quests))
        .field("upkeep", Field::Text(upkeep.clone()))
        .tone("upkeep", &[if upkeep.is_empty() { "" } else { "foe" }])
        .field("log", Field::Rows(log))
        .action(PLACE, "Pick", true)
        .action(WAY, "Take this way", !ways.is_empty())
        .action(TRAVEL, &if to.is_empty() { "Travel".to_string() } else { format!("Travel to {to}") }, pick.is_some())
        .action(ENTER, &format!("Enter {here_name}"), !travel::current_node(d, gs).is_empty())
        .action(CAMP, &camp_label, camp_on)
        .action(MODE, &mode_label, many)
        .action(BACK, "Back", true)
}

/// A place's row: name, region line, first hover line (toned), the quest badge, its picture, the other lines, picked,
/// off while locked.
fn place_row(cx: &Cx, wm: &WorldMap, here: &str, n: &json::Dict) -> RowView {
    let (d, gs) = (&cx.data, cx.gs());
    let q = s(n, "qid");
    let lines = place_lines(cx, gs, here, n);
    let (first, tone) = lines.first().map(|(l, t)| (l.clone(), t.role().to_string())).unwrap_or_default();
    let badge = json::b(n, "quest", false) || !json::sget(n, "jobs").is_empty();
    let art = emberwake_content::art::landmark(&q, &s(explore::resolve_location(d, gs, &q), "art"));
    let picture = if cx.art.layers().has(&art) { art } else { emberwake_content::art::pin(&s(n, "kind")) };
    RowView {
        id: q.clone(),
        label: s(n, "name"),
        sub: place_region(n),
        detail: first,
        tone,
        detail2: if badge { "Quest".into() } else { String::new() },
        tone2: if badge { "gold".into() } else { String::new() },
        picture: Some(picture),
        note: lines.iter().skip(1).map(|(l, _)| l.as_str()).collect::<Vec<_>>().join(" "),
        selected: q == wm.selected,
        enabled: !travel::locked(d, gs, &q),
        ..Default::default()
    }
}

/// A view click as the map's own (`WAct`); None for an unknown or switched-off action, a place the map does not show
/// or a way that is not offered. Stage B's layout draw hands its clicks here; until then only the tests call it.
#[cfg_attr(not(test), allow(dead_code))]
pub fn worldmap_act(cx: &Cx, wm: &WorldMap, a: &Act) -> Option<WAct> {
    let v = view(cx, wm);
    v.act(&a.action).filter(|x| x.enabled)?;
    let arg = a.arg.clone().unwrap_or_default();
    match a.action.as_str() {
        PLACE => match v.get("places") {
            Some(Field::Rows(rows)) if rows.iter().any(|r| r.id == arg && !arg.is_empty()) => Some(WAct::Node(arg)),
            _ => None,
        },
        WAY => arg.parse::<usize>().ok().filter(|i| *i < wm.ways(cx).len()).map(WAct::Way),
        TRAVEL => Some(WAct::Travel),
        ENTER => Some(WAct::Enter),
        CAMP => Some(WAct::Camp),
        MODE => Some(WAct::Mode),
        BACK => Some(WAct::Back),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::hud_tests::{frame, gfx};
    use crate::screens::Screen;
    use crate::worldmap::tests::{q, ward_hub, TIER2};

    fn rows_of<'a>(v: &'a View, name: &str) -> &'a [RowView] {
        match v.get(name) {
            Some(Field::Rows(r)) => r,
            _ => panic!("no rows {name}"),
        }
    }

    fn text<'a>(v: &'a View, name: &str) -> &'a str {
        match v.get(name) {
            Some(Field::Text(t)) => t,
            _ => panic!("no text {name}"),
        }
    }

    /// The Sunk Ward hub with Tier 2 open and the Tallow Vats picked (more than one way there), drawn once.
    fn vats_picked() -> (Cx, WorldMap) {
        let mut cx = ward_hub(&TIER2, &["drowned_runoff_grate", "alleyway_junction_7", "ash_crawl_crossroads"], true);
        let (mut wm, mut g) = (WorldMap::new(), gfx());
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let act = worldmap_act(&cx, &wm, &Act { action: PLACE.into(), arg: Some(q("tallow_vats")) }).expect("the vats are shown");
        assert!(matches!(&act, WAct::Node(n) if *n == q("tallow_vats")));
        wm.click_id(&mut cx, &format!("node.{}", q("tallow_vats").replace(':', "."))).expect("the vats' mark");
        frame(&mut wm, &mut cx, &mut g, 1.0);
        (cx, wm)
    }

    /// At rest: the fields in order, every shown place a row (the party's place says so, picture set, locked ones off),
    /// no trip, no ways; Travel off, Back on; a click on a place the map does not show maps to nothing.
    #[test]
    fn the_view_lists_the_places_at_rest() {
        let mut cx = ward_hub(&[], &["drowned_runoff_grate"], false);
        let (mut wm, mut g) = (WorldMap::new(), gfx());
        frame(&mut wm, &mut cx, &mut g, 1.0);
        let v = view(&cx, &wm);
        let names: Vec<&str> = v.fields.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["title", "here", "time", "places", "trip", "hint", "ways", "quests", "upkeep", "log"]);
        assert_eq!(text(&v, "here"), cx.location_name(&cx.gs().world_id, &cx.gs().location_id));
        let tv = travel::view(&cx.data, cx.gs());
        let ids: Vec<String> = rows_of(&v, "places").iter().map(|r| r.id.clone()).collect();
        assert_eq!(ids, json::aget(&tv, "nodes").iter().map(|n| s(json::dict(n), "qid")).collect::<Vec<_>>(), "the map's places in view order");
        assert_eq!(ids, json::aget(json::dict(&wm.state(&cx)), "places").iter().map(|p| p.as_str().unwrap_or("").to_string()).collect::<Vec<_>>(), "the places drawn");
        let hub = rows_of(&v, "places").iter().find(|r| r.id == cx.gs().location_id).expect("the hub");
        assert_eq!((hub.detail.as_str(), hub.tone.as_str()), ("Your party is here.", "info"));
        assert!(rows_of(&v, "places").iter().all(|r| r.picture.as_deref().is_some_and(|p| !p.is_empty())));
        let locked: Vec<String> = json::aget(json::dict(&wm.state(&cx)), "locked").iter().map(|p| p.as_str().unwrap_or("").to_string()).collect();
        assert!(rows_of(&v, "places").iter().all(|r| r.enabled != locked.contains(&r.id)), "locked places off");
        assert!(text(&v, "trip").is_empty() && rows_of(&v, "ways").is_empty());
        assert!(!v.act(TRAVEL).expect("travel").enabled && v.act(BACK).expect("back").enabled);
        let act = |a: &str, arg: Option<&str>| worldmap_act(&cx, &wm, &Act { action: a.into(), arg: arg.map(str::to_string) });
        assert!(act(PLACE, Some("sunk_ward:nowhere")).is_none() && act(PLACE, None).is_none() && act(TRAVEL, None).is_none());
        assert!(matches!(act(BACK, None), Some(WAct::Back)) && act("nope", None).is_none());
    }

    /// A pick with more than one way: Travel on and named, the trip line the Travel tip's, a row per way (the chosen
    /// one selected, as `state.ways`), the picked place selected; a way click maps to it, past the last to nothing.
    #[test]
    fn a_pick_lists_its_ways_and_the_trip() {
        let (mut cx, mut wm) = vats_picked();
        let v = view(&cx, &wm);
        let ways = wm.ways(&cx);
        assert!(ways.len() >= 2);
        let rows = rows_of(&v, "ways");
        assert_eq!(rows.iter().map(|r| r.label.clone()).collect::<Vec<_>>(), ways.iter().map(|w| w.label.clone()).collect::<Vec<_>>());
        assert!(rows[0].selected && !rows[1].selected && rows[1].note.ends_with("Click to take this way."));
        assert!(text(&v, "trip").contains("Roads can be ambushed.") && v.tone_of("trip", 0).unwrap_or("").is_empty(), "{}", text(&v, "trip"));
        let travel = v.act(TRAVEL).expect("travel");
        assert!(travel.enabled && travel.label.starts_with("Travel to "), "{travel:?}");
        assert!(rows_of(&v, "places").iter().find(|r| r.id == q("tallow_vats")).is_some_and(|r| r.selected));
        let act = |cx: &Cx, wm: &WorldMap, a: &str, arg: &str| worldmap_act(cx, wm, &Act { action: a.into(), arg: Some(arg.into()) });
        assert!(matches!(act(&cx, &wm, WAY, "1"), Some(WAct::Way(1))));
        assert!(act(&cx, &wm, WAY, &ways.len().to_string()).is_none() && act(&cx, &wm, WAY, "x").is_none());
        assert!(matches!(act(&cx, &wm, TRAVEL, ""), Some(WAct::Travel)));
        // the second way picked: its row selected
        wm.click_id(&mut cx, "world_map.way.1").expect("the second way");
        let v = view(&cx, &wm);
        assert!(rows_of(&v, "ways")[1].selected && !rows_of(&v, "ways")[0].selected);
    }

    /// The agent link carries the view as `layout_view` (its `view` stays the kit's travel view).
    #[test]
    fn the_agent_link_has_the_view() {
        let (cx, wm) = vats_picked();
        let st = wm.state(&cx);
        assert_eq!(st["layout_view"], view(&cx, &wm).to_json());
        assert_eq!(st["view"], serde_json::Value::Object(travel::view(&cx.data, cx.gs())));
    }

    /// US6: the fixture has the view's shape (field names and kinds, action ids).
    #[test]
    fn the_fixture_has_the_view_shape() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let layers = aegis_core::assets::Layers::of(vec![root]);
        let fixture = aegis_ui::screen_layout::load_fixture(&layers, SCREEN).expect("the fixture file").expect("it parses");
        let shape = |v: &View| (v.screen.clone(), v.fields.iter().map(|(n, f)| (n.clone(), f.kind())).collect::<Vec<_>>(), v.actions.iter().map(|a| a.id.clone()).collect::<Vec<_>>());
        let (cx, wm) = vats_picked();
        assert_eq!(shape(&fixture), shape(&view(&cx, &wm)));
    }

    /// Writes the fixture from the Sunk Ward hub with the Tallow Vats picked (run by hand: `cargo test -p emberwake_gui
    /// write_world_map_fixture -- --ignored`), so it holds the sample's values.
    #[test]
    #[ignore]
    fn write_world_map_fixture() {
        let (cx, wm) = vats_picked();
        let v = view(&cx, &wm).to_json();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/views/world_map.view.json");
        std::fs::write(path, serde_json::to_string_pretty(&v).unwrap() + "\n").unwrap();
    }
}
