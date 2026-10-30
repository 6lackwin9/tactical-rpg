//! ENCOUNTER2 S4 (ENCOUNTER2-HA): the sim's default sweep of the catalog (`enc=all`) skips what is not a balance fight:
//! encounters with a `stay_behind` block or `rooms`, and the kit's own (ids starting `kit_`); named with `enc=` they
//! still run. Scratch data built here.

use rpgkit_sim::args::{catalog_encounters, SimArgs};
use rpgkit_world::Data;
use serde_json::json;

/// Five catalog encounters: two balance fights, a hold escape, a room fight and a kit fixture (an empty `stay_behind`
/// and empty `rooms` do not count).
fn data() -> Data {
    let enc = |id: &str, extra: serde_json::Value| {
        let mut row = json!({"id": id, "name": id, "enemies": []});
        for (k, v) in extra.as_object().unwrap() {
            row[k] = v.clone();
        }
        row
    };
    let catalog = json!({"encounters": [
        enc("e_plain", json!({})),
        enc("e_hold", json!({"objective": "escape", "stay_behind": {"mode": "hold", "tiles": [[1, 1]]}})),
        enc("e_rooms", json!({"rooms": [{"id": "yard"}]})),
        enc("kit_arena", json!({})),
        enc("e_empty", json!({"stay_behind": {}, "rooms": []})),
    ]});
    Data::from_values(&catalog, &json!({}), &json!({}))
}

fn parse(d: &Data, line: &str) -> Result<SimArgs, String> {
    SimArgs::parse(d, &line.split_whitespace().map(String::from).collect::<Vec<_>>())
}

#[test]
fn the_catalog_sweep_skips_stay_behind_rooms_and_kit_fights() {
    let d = data();
    assert_eq!(catalog_encounters(&d), ["e_plain", "e_empty"]);
    let all = parse(&d, "duel enc=all").unwrap().encounters;
    for skipped in ["e_hold", "e_rooms", "kit_arena"] {
        assert!(!all.iter().any(|e| e == skipped), "{skipped} is not in enc=all");
    }
    assert!(all.iter().any(|e| e == "e_plain") && all.iter().any(|e| e == "e_empty"));
}

#[test]
fn named_they_still_run() {
    let d = data();
    assert_eq!(parse(&d, "duel enc=e_hold,e_rooms,kit_arena").unwrap().encounters, ["e_hold", "e_rooms", "kit_arena"]);
}
