//! The campaign checks (CAMPAIGN1-S, docs/specs/CAMPAIGN1.md 11.1-11.4) on the classic worlds: the frozen Godot data
//! (legacy/data: the Ashlands, Iron Reaches and Tidewrack) sets none of the campaign fields or kinds, so the campaign
//! checks add nothing to its reports (checks_parity.rs compares the whole reports with Godot's). The kit's own tests
//! are kits/rpg/crates/rpgkit_content/tests/campaign_schema.rs.

use aegis_core::json;
use emberwake_content::{check, store};
use serde_json::json;
use std::path::PathBuf;

#[test]
fn the_classic_worlds_report_nothing_new() {
    let frozen = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../legacy/data");
    if !frozen.is_dir() {
        eprintln!("no legacy/data");
        return;
    }
    let v = store::view(&frozen);
    let worlds = v.worlds.world_ids();
    assert!(!worlds.is_empty());
    for w in worlds {
        let m = v.worlds.manifest(&w);
        for key in ["campaign", "purse", "rest", "defeat"] {
            assert_eq!(json::get(m, key), &json!({}), "{w}: {key} reads as today's rule");
        }
        assert_eq!(check::campaign::manifest_errors(&v, &w, m), Vec::<String>::new(), "{w}");
        assert_eq!(check::campaign::warnings(&v, &w), Vec::<String>::new(), "{w}");
        for kind in check::world::KINDS {
            for e in v.worlds.own_entries(&w, kind) {
                assert_eq!(check::campaign::read_key_errors(&v, &w, e), Vec::<String>::new(), "{w}/{kind}:{}", json::s(e, "id"));
            }
        }
        for kind in emberwake_data::CAMPAIGN_KINDS {
            assert!(v.worlds.own_entries(&w, kind).is_empty(), "{w} has no {kind}");
        }
    }
}
