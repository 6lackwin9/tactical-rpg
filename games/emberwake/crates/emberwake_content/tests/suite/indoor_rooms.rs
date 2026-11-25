//! REVEAL1 2.1 (PR5-54): the `floorboards` ground the indoor rooms of homes, shops and workshops lie on. The row ships
//! in `grounds.json`: it must be there, have the spec's shape, pass its checks and find its art along the chain every
//! world reads (`tilesets/common/ground/floorboards/v0.png`, the asset layers of the game). The values asserted
//! are the spec's (REVEAL1 2.1, REGIONGROUND1's table), not counted from the data.

use aegis_core::assets::Layers;
use aegis_core::json;
use emberwake_content::check::ground;
use emberwake_content::{art, source};
use emberwake_data::{Catalog, GROUNDS_KIND};
use std::path::PathBuf;

const ID: &str = "floorboards";

fn game_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn floorboards_is_an_indoor_land_ground_with_its_colour_and_art() {
    let src = source::load(&game_dir().join("data"));
    let rows: Vec<json::Dict> = json::arr(&src.catalog[GROUNDS_KIND]).iter().map(|r| json::dict(r).clone()).collect();
    let at = rows.iter().position(|r| json::s(r, "id") == ID).expect("REVEAL1 2.1: the 'floorboards' ground row ships in grounds.json");
    let errors = ground::check_rows(&rows).swap_remove(at);
    assert!(errors.is_empty(), "{errors:?}");
    let cat = Catalog::from_value(&src.catalog);
    let row = &cat.grounds[ID];
    assert_eq!((row.kind.as_str(), row.colour.as_str(), row.indoor), ("land", "#3a2c22", true));
    assert!(!row.is_water() && !row.strays);
    let layers = Layers::for_game(&game_dir());
    let dirs = art::ground_dirs(&art::region_sets(&[], ""), ID);
    assert!(dirs.iter().any(|d| layers.find(&format!("{d}/v0.png")).is_some()), "a texture (or its placeholder) along {dirs:?}");
}
