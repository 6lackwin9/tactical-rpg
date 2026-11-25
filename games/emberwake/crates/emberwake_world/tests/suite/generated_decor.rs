//! DECOR-GEN1 (PR7-86) on the shipped data (read only): a world fight on a generated Sunk Ward board (an encounter with
//! no rows and no decor, picked by rule) draws 2 to 6 display-only decor pieces from the world's `decor_pool`, the same
//! for the same battle seed, off the exits and the deploy cells; the board's walkability is the board without decor.

use aegis_core::json;
use emberwake_battle::encounter::GENERATED_DECOR;
use emberwake_world::{battle, new_game, Data};
use std::path::Path;

const WORLD: &str = "sunk_ward";

fn data() -> Data {
    Data::open(&Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")).unwrap().0
}

#[test]
fn a_generated_sunk_ward_board_draws_its_world_decor() {
    let d = data();
    let pool = battle::decor_pool(&d, WORLD);
    assert!(!pool.is_empty(), "the Sunk Ward names a decor pool");
    let generated: Vec<String> = d.catalogs.own_entries(WORLD, "encounters").iter().filter(|e| json::arr(json::get(e, "rows")).is_empty() && json::arr(json::get(e, "decor")).is_empty()).map(|e| json::s(e, "id")).collect();
    assert!(!generated.is_empty(), "the Sunk Ward has generated boards (roam, scavenger, patrol)");
    for id in &generated {
        let fight = json::obj([("encounter", id.clone().into())]);
        let mut gs = new_game::create_for_world(&d, WORLD, 1).expect("a new game");
        let f = battle::begin_world(&d, &mut gs, &fight).unwrap_or_else(|e| panic!("{id}: {e}"));
        let s = &f.session;
        let n = s.def.decor.len() as i32;
        assert!((GENERATED_DECOR.0..=GENERATED_DECOR.1).contains(&n), "{id}: {n} decor");
        let deployed: Vec<_> = (0..s.w.units.len()).flat_map(|c| s.w.cells_of(c)).collect();
        for dec in &s.def.decor {
            assert!(pool.contains(&dec.art), "{id}: {dec:?} from the pool");
            assert!(!s.w.grid.is_exit(dec.at) && !deployed.contains(&dec.at), "{id}: {dec:?} on an exit or a deploy cell");
        }
        // the same battle seed draws the same
        let mut again = new_game::create_for_world(&d, WORLD, 1).expect("a new game");
        let g = battle::begin_world(&d, &mut again, &fight).expect("the same fight");
        assert_eq!(g.session.def.rng_seed, s.def.rng_seed);
        assert_eq!(g.session.def.decor, s.def.decor, "{id}: same seed, same decor");
        // the board without decor: the same terrain
        let (mut def, roster) = battle::authored_fight(&d, WORLD, id, 0).expect("the authored fight");
        def.decor_pool.clear();
        let bare = emberwake_battle::session::Session::start(d.cat.clone(), def, &roster);
        assert!(bare.def.decor.is_empty());
        assert_eq!(bare.w.grid.terrain, s.w.grid.terrain, "{id}: decor changes no terrain");
    }
}
