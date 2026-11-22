//! The battle stage: the kit's (`rpgkit_ui::stage`, ENGINE-41 STAGE-KIT1). The game keeps its own choices only: its
//! tuning gives the board's sizes (`StageSizes`), and the tests below read its shipped art, data and settings.

pub use rpgkit_ui::stage::*;

use crate::tuning::Tuning;

impl StageSizes for Tuning {
    fn token_px(&self) -> i32 {
        self.token_px
    }
    fn flat_token_px(&self) -> i32 {
        self.flat_token_px
    }
    fn foot_inset_px(&self) -> i32 {
        self.foot_inset_px
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use aegis_ui::art::Art;
    use aegis_ui::canvas::Rect;
    use emberwake_battle::grid::{Grid, TerrainTable, V2};
    use emberwake_data::ZoneDef;
    use indexmap::IndexMap;
    use serde_json::{json, Value};
    use std::path::Path;

    /// The kit's stage fixture terrain rows (`rpgkit_ui::stage` tests), which the shipped rows must match.
    fn looks() -> emberwake_data::Catalog {
        emberwake_data::Catalog::from_value(&json!({"terrain": [
            {"id": "grass", "color": "#3c6838"},
            {"id": "dirt", "relief": "sunken", "relief_depth": 2, "color": "#685438"},
            {"id": "stone", "color": "#5c5c64"},
            {"id": "sand", "color": "#aaa078"},
            {"id": "snow", "color": "#aaa078"},
            {"id": "shallow_water", "relief": "sunken", "relief_depth": 3, "color": "#385c8c", "face": "water"},
            {"id": "deep_water", "relief": "sunken", "relief_depth": 5, "color": "#1e366e", "face": "water"},
            {"id": "mud", "relief": "sunken", "relief_depth": 1},
            {"id": "lava", "relief": "sunken", "relief_depth": 5, "color": "#963214", "face": "lava"},
            {"id": "wall", "relief": "block", "color": "#2e2e36", "face": "wall"},
            {"id": "tree", "relief": "standing", "prop": "tree", "color": "#1e4628"},
            {"id": "rubble", "prop": "stones", "color": "#5c5c64"},
            {"id": "toxic_bog", "relief": "sunken", "relief_depth": 2},
        ]}))
    }

    fn zone(id: &str) -> Grid {
        let mut g = Grid::new(false, Vec::new(), TerrainTable::from_catalog(&looks()), 0);
        g.apply_zone(id);
        g
    }

    fn stage() -> Geom {
        Geom::new(View::Stage, &Tuning::default())
    }

    /// ENGINE-41: the game's default tuning gives the sizes the kit's stage tests draw at, and its settings default
    /// to the Stage view with the selected unit shown through cover.
    #[test]
    fn the_default_tuning_and_settings_match_the_kits_stage_tests() {
        let kit = Sizes { token_px: 40, flat_token_px: 28, foot_inset_px: 5 };
        for view in [View::Stage, View::Flat] {
            assert_eq!(Geom::new(view, &Tuning::default()), Geom::new(view, &kit));
        }
        assert_eq!(View::of_setting(&crate::screens::default_settings()["battle_view"]), View::Stage, "the default");
    }

    fn game_art() -> Art {
        Art::new(Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")))
    }

    /// A body rect of a 40 px token on `c` from a real body's solid bounds.
    fn body_on(art: &mut Art, g: &Geom, key: &str, c: V2) -> Rect {
        let b = art.body_bounds(key, "", g.token_px, false).expect("the body loads");
        let tr = g.token_rect(c, 0, 0);
        Rect::new(tr.x + b.x, tr.y + b.y, b.w, b.h)
    }

    /// PR5-33: the shipped terrain rows give the fixture's looks (so `TODAYS_FRAME`, drawn on `looks()`, stays the
    /// shipped board): every terrain type of the shipped table has the look the fixture table gives it.
    #[test]
    fn the_shipped_terrain_rows_give_the_fixture_looks() {
        let dir = emberwake_world::Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        let shipped = TerrainTable::from_catalog(&emberwake_world::Data::open(&dir).expect("data").0.cat);
        let fixture = TerrainTable::from_catalog(&looks());
        for id in fixture.ids.iter().chain(&shipped.ids) {
            assert_eq!(shipped.look(id), fixture.look(id), "{id}");
        }
    }

    /// Check 9: the zone's backdrop, else the gradient (no file).
    #[test]
    fn a_zone_loads_its_backdrop_and_an_unknown_zone_draws_the_gradient() {
        let mut art = game_art();
        assert_eq!(backdrop_art(&mut art, "forest_clearing"), Some("backgrounds/battle/forest_clearing.png".into()));
        assert_eq!(art.size_of("backgrounds/battle/forest_clearing.png"), Some((BACKDROP.w, BACKDROP.h)), "drawn 1:1");
        assert_eq!(backdrop_art(&mut art, "no_such_zone"), None);
        assert_eq!(backdrop_art(&mut art, ""), None);
        assert_eq!(far_ground(&zone("forest_clearing")), "grass");
        assert_eq!(far_ground(&zone("frozen_pass")), "snow");
        assert_eq!(far_ground(&zone("lava_cavern")), "grass", "row 0 all walls: none, grass");
    }

    /// HOOKS1 ZN3: a data zone row draws its backdrop key when the file exists, else the gradient to its `ground`
    /// (not the far ground of the board); a code zone as before, also when a (bad) row reuses its id.
    #[test]
    fn a_data_zone_keys_its_backdrop_and_its_gradient_ground() {
        let mut art = game_art();
        // `zz_test_zone`: a test-only zone id with no file under backgrounds/battle/
        let one = |id: &str, backdrop: &str, ground: &str| {
            let z = ZoneDef::from_obj(json!({"id": id, "name": "Test", "backdrop": backdrop, "ground": ground}).as_object().unwrap());
            IndexMap::from([(id.to_string(), z)])
        };
        let board = zone("forest_clearing");
        assert!(!art.has("backgrounds/battle/zz_test_zone.png"), "fixture");
        let missing = one("zz_test_zone", "", "stone");
        assert_eq!(backdrop_source(&mut art, &board, "zz_test_zone", &missing), (None, "stone"), "no file: the gradient to the row's ground");
        let keyed = one("zz_test_zone", "backgrounds/battle/forest_clearing.png", "bridge");
        assert_eq!(backdrop_source(&mut art, &board, "zz_test_zone", &keyed), (Some("backgrounds/battle/forest_clearing.png".into()), "bridge"), "the row's key");
        let gone = one("zz_test_zone", "backgrounds/battle/zz_none.png", "sand");
        assert_eq!(backdrop_source(&mut art, &board, "zz_test_zone", &gone), (None, "sand"), "a missing key file: the gradient");
        let code = (Some("backgrounds/battle/forest_clearing.png".to_string()), "grass");
        assert_eq!(backdrop_source(&mut art, &board, "forest_clearing", &IndexMap::new()), code, "a code zone unchanged");
        let shadow = one("forest_clearing", "backgrounds/battle/zz_none.png", "sand");
        assert_eq!(backdrop_source(&mut art, &board, "forest_clearing", &shadow), code, "a row reusing a code id is ignored");
    }

    /// Checks 4 and 5: the feet order (unit north of a tree, the tree, the unit south of it); a block before a unit at
    /// equal feet; the southern of two units last. The unit behind the tree is outlined in its side colour, the one
    /// in front is not; a selected hero north of another unit is outlined gold; an unselected, unhovered unit behind
    /// a unit is not.
    #[test]
    fn the_feet_pass_sorts_by_feet_and_outlines_what_is_covered() {
        let mut art = game_art();
        let g = stage();
        let forest = zone("forest_clearing");
        assert_eq!((forest.terrain_id(V2::new(10, 1)), forest.terrain_id(V2::new(10, 2)), forest.terrain_id(V2::new(10, 3))), ("grass", "tree", "grass"));
        let board = board_items(&mut art, &forest);
        assert!(board.iter().any(|p| p.item == Item::Prop(V2::new(10, 2))), "the tree stands (STAGE1-T art)");
        let (n, s) = (V2::new(10, 1), V2::new(10, 3));
        let mut items = board.clone();
        items.push(unit_item(0, g.foot(n), body_on(&mut art, &g, "warrior", n)));
        items.push(unit_item(1, g.foot(s), body_on(&mut art, &g, "warrior", s)));
        let order = feet_order(&items);
        let pos = |it: Item| order.iter().position(|p| p.item == it).unwrap();
        assert!(pos(Item::Unit(0)) < pos(Item::Prop(V2::new(10, 2))) && pos(Item::Prop(V2::new(10, 2))) < pos(Item::Unit(1)));
        assert_eq!(through(&order, pos(Item::Unit(0)), false, false, Reveal::Always), Some(Through::Side), "behind the canopy");
        assert_eq!(through(&order, pos(Item::Unit(1)), false, false, Reveal::Always), None, "in front of the tree");
        // a block and a unit at equal feet: the block first
        let cell = g.cell_rect(V2::new(3, 3));
        let block = Placed { item: Item::Block(V2::new(3, 3)), foot_y: cell.y + cell.h, x: cell.x, rect: cell };
        let unit = unit_item(2, (cell.x + 16, cell.y + cell.h), cell);
        assert_eq!(feet_order(&[unit, block]).iter().map(|p| p.item).collect::<Vec<_>>(), vec![block.item, unit.item]);
        // two units in one column: the southern last; the northern selected is outlined gold, else not at all
        let open = zone("stone_arena");
        let (a, b) = (V2::new(5, 3), V2::new(5, 4));
        let two = [unit_item(1, g.foot(b), body_on(&mut art, &g, "warrior", b)), unit_item(0, g.foot(a), body_on(&mut art, &g, "warrior", a))];
        let mut with_board = board_items(&mut art, &open);
        with_board.extend(two);
        let order = feet_order(&with_board);
        assert_eq!(order.iter().map(|p| p.item).collect::<Vec<_>>(), vec![Item::Unit(0), Item::Unit(1)]);
        assert_eq!(through(&order, 0, true, false, Reveal::Always), Some(Through::Selected), "a selected hero north of a unit");
        assert_eq!(through(&order, 0, false, true, Reveal::Always), Some(Through::Hovered));
        assert_eq!(through(&order, 0, false, false, Reveal::Always), None, "unselected and unhovered behind a unit");
        assert_eq!(through(&order, 1, true, false, Reveal::Always), None, "nothing in front of the southern unit");
        // STAGE1-O: by default only the selected unit shows through; Alt held shows every covered unit, units included
        assert_eq!(through(&order, 0, true, false, Reveal::Selected), Some(Through::Selected), "selected, by default");
        assert_eq!(through(&order, 0, false, true, Reveal::Selected), None, "hovered alone stays hidden by default");
        assert_eq!(through(&order, 0, false, false, Reveal::Held), Some(Through::Side), "Alt held: behind a unit");
        assert_eq!(through(&order, 1, false, false, Reveal::Held), Some(Through::Side), "Alt held: uncovered units are outlined too");
    }

    /// STAGE1-O: the canopy hides a unit by default and reveals it while Alt is held or with the setting Always; the
    /// setting's values map to the reveal, Alt winning.
    #[test]
    fn units_behind_cover_show_only_when_selected_alt_held_or_always() {
        let mut art = game_art();
        let g = stage();
        let forest = zone("forest_clearing");
        let n = V2::new(10, 1);
        let mut items = board_items(&mut art, &forest);
        items.push(unit_item(0, g.foot(n), body_on(&mut art, &g, "warrior", n)));
        let order = feet_order(&items);
        let at = order.iter().position(|p| p.item == Item::Unit(0)).unwrap();
        assert_eq!(through(&order, at, false, false, Reveal::Selected), None, "behind the canopy, hidden by default");
        assert_eq!(through(&order, at, true, false, Reveal::Selected), Some(Through::Selected));
        assert_eq!(through(&order, at, false, false, Reveal::Held), Some(Through::Side));
        assert_eq!(through(&order, at, false, false, Reveal::Always), Some(Through::Side));
        assert_eq!(Reveal::of(&Value::from("selected"), false), Reveal::Selected);
        assert_eq!(Reveal::of(&Value::from("always"), false), Reveal::Always);
        assert_eq!(Reveal::of(&Value::from("always"), true), Reveal::Held, "Alt wins");
        assert_eq!(Reveal::of(&Value::Null, false), Reveal::Selected, "missing: the default");
        assert_eq!(Reveal::of(&crate::screens::default_settings()["see_through"], false), Reveal::Selected, "the default setting");
        assert_eq!(SEE_THROUGH[0].0, "selected", "the first value is the default");
    }

    /// DIORAMA1B: (a copy of the kit stage tests' helper, for the map view's test) a scratch root with body `units/painted/base`: a `w` x `h` frame (one `idle` row, 2 frames) whose
    /// figure fills columns w/4..3w/4 from row h/8 down to its `foot` row, painted.
    pub(crate) fn painted_root(name: &str, (w, h): (u32, u32), foot: u32) -> aegis_core::scratch::Scratch {
        let root = aegis_core::scratch::Scratch::vacant(&format!("stage_painted_{name}"));
        let sw = 2 * w;
        let mut px = vec![0u8; (sw * h * 4) as usize];
        for y in h / 8..=foot {
            for x in (w / 4..3 * w / 4).chain(w + w / 4..w + 3 * w / 4) {
                let i = ((y * sw + x) * 4) as usize;
                px[i..i + 4].copy_from_slice(&[90, 70, 50, 255]);
            }
        }
        let dir = root.join("units/painted/base");
        std::fs::create_dir_all(&dir).unwrap();
        let file = std::fs::File::create(dir.join("sheet.png")).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), sw, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&px).unwrap();
        let clips = json!({"frame": [w, h], "foot": foot, "painted": true, "clips": [{"id": "idle", "frames": 2, "ms": [200, 200]}]});
        std::fs::write(dir.join("clips.json"), clips.to_string()).unwrap();
        root
    }
}
