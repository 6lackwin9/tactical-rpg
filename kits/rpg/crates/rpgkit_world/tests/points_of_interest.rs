//! PR7-37 (PT-03, PT-29) on a scratch world (no shipped data): a point of interest the party stands on or faces is the
//! one it acts on (`explore::point_near`: its own tile first, then the tile ahead; an absent point is none); its line is
//! its label and note (`point_line`); its `use` is one of `rpgkit_data::POINT_USES` (`point_use`, anything else ""); the
//! crypt line counts the members housed in crypt places (`roster::crypt_words`).

use aegis_core::json::{self, Dict};
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{explore, roster, Data, GameState};
use serde_json::{json, Value};

fn point(id: &str, label: &str, note: &str, used: &str) -> Value {
    json!({"id": id, "kind": "point", "anchor": id, "label": label, "note": note, "use": used})
}

/// Room `room` (open floor) with the brazier at (3, 2), the vaults at (5, 2) and a hidden point at (3, 4) (shown only
/// with flag `lit`); the party spawns at (2, 2).
fn data() -> Data {
    let rows = ["########", "#......#", "#......#", "#......#", "#......#", "########"];
    let map = json!({"id": "room", "rows": rows, "anchors": [{"id": "spawn", "x": 2, "y": 2}, {"id": "brazier", "x": 3, "y": 2}, {"id": "vaults", "x": 5, "y": 2}, {"id": "hidden", "x": 3, "y": 4}]});
    let mut hidden = point("hidden", "Hidden", "", "");
    hidden["requires_flags"] = json!([{"key": "lit", "value": true}]);
    let markers = json!([point("brazier", "The brazier", "Warm embers.", "rest"), point("vaults", "The vaults", "", "crypts"), hidden]);
    let worlds = json!({"w": {"manifest": {"id": "w", "start_location": "room", "start_party": ["hero"]}, "own": {
        "locations": [{"id": "room", "name": "Room", "map": "room", "markers": markers, "_world": "w", "_qid": "w:room"}]
    }}});
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    Data::from_values(&catalog, &worlds, &json!({"maps": [map]}))
}

fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let mut gs = GameState::create(d, "w", party);
    explore::enter(d, &mut gs, "room", explore::SPAWN);
    gs
}

fn id(m: Option<Dict>) -> String {
    m.map(|m| json::s(&m, "id")).unwrap_or_default()
}

#[test]
fn the_point_underfoot_comes_first_then_the_one_ahead() {
    let d = data();
    let mut gs = state(&d);
    let start = gs.position;
    assert_eq!(id(explore::point_near(&d, &gs, (1, 0))), "brazier", "the brazier ahead");
    assert_eq!(id(explore::point_near(&d, &gs, (-1, 0))), "", "nothing behind");
    gs.position = (start.0 + 1, start.1);
    assert_eq!(id(explore::point_near(&d, &gs, (0, 1))), "brazier", "standing on it, facing away");
    gs.position = (start.0 + 1, start.1 + 1);
    assert_eq!(id(explore::point_near(&d, &gs, (0, 1))), "", "a point whose flags fail is not there");
    gs.set_flag("lit", json!(true));
    assert_eq!(id(explore::point_near(&d, &gs, (0, 1))), "hidden", "once its flag holds");
}

#[test]
fn a_points_line_and_use() {
    let d = data();
    let gs = state(&d);
    let marks = explore::markers_of(explore::location(&d, &gs));
    let by = |i: &str| marks.iter().find(|m| json::s(m, "id") == i).cloned().expect("the marker");
    assert_eq!(explore::point_line(&by("brazier")), "The brazier: Warm embers.");
    assert_eq!(explore::point_line(&by("vaults")), "The vaults.", "no note: the label alone");
    assert_eq!(explore::point_line(&json::obj([("note", "Just a note.".into())])), "Just a note.");
    assert_eq!(explore::point_line(&Dict::new()), "Something here.");
    assert_eq!(explore::point_use(&by("brazier")), "rest");
    assert_eq!(explore::point_use(&by("vaults")), "crypts");
    assert_eq!(explore::point_use(&by("hidden")), "");
    assert_eq!(explore::point_use(&json::obj([("use", "dance".into())])), "", "an unknown use does nothing");
    for u in rpgkit_data::POINT_USES {
        assert_eq!(explore::point_use(&json::obj([("use", u.into())])), u);
    }
}

#[test]
fn the_crypt_line_counts_the_members_in_crypt_places() {
    let d = data();
    let mut gs = state(&d);
    let name = |m: &str| m.to_uppercase();
    gs.party.roster.manifest.crypts = 0;
    assert_eq!(roster::crypt_words(&gs, &name), "", "no crypt places: nothing to say");
    gs.party.roster.manifest.crypts = 2;
    assert_eq!(roster::crypt_words(&gs, &name), format!("Crypt places: none of {} in use.", roster::crypt_cap(&gs)));
    assert_eq!(roster::crypts_used(&gs.party), 0, "the hero has a bunk");
    gs.party.members[0].housing = "crypt".into();
    let used = roster::crypts_used(&gs.party);
    assert_eq!(used, 1);
    assert_eq!(roster::crypt_words(&gs, &name), format!("Crypt places: {used} of {} (HERO).", roster::crypt_cap(&gs)), "who lies there, by the caller's names");
}
