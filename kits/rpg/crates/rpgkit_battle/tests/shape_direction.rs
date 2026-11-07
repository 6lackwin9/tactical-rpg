//! A3.2-K2 (INTENT2 IR6, IR7; `docs/sessions/A3_SHAPES.md` section 1): the directed shapes LINE, ARC and CONE read
//! one direction, from the user's tile to the source tile, snapped: |dx| > |dy| left or right, |dy| > |dx| up or down,
//! |dx| = |dy| the exact diagonal. Each is drawn on the source: LINE the source and 2 more onward; ARC the source and
//! its two neighbours across the direction (on a diagonal the L with the source as the corner, its arms back toward the
//! user's row and column); CONE the source then the 3 tiles across one step on (on a diagonal the source and its 3
//! neighbours away from the user). A source on the user's own tile has no direction: no tiles. For an adjacent source
//! (the only one the planner offers a drawn shape today) the tiles are exactly the pre-K2 ones, so classic play is
//! unchanged. Expectations come from those rules.

use rpgkit_battle::grid::V2;
use rpgkit_battle::skill::{self, Area};
use rpgkit_data::{Catalog, SkillDef};
use serde_json::json;

const fn v(x: i32, y: i32) -> V2 {
    V2::new(x, y)
}

const ME: V2 = V2::new(5, 5);

fn skill_with(area: &str) -> SkillDef {
    let cat = Catalog::from_value(&json!({"skills": [{"id": "t_area", "name": "Area", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0, "area": area}]}));
    cat.skills.into_iter().next().expect("the test skill")
}

fn sorted(mut c: Vec<V2>) -> Vec<V2> {
    c.sort_by_key(|p| (p.x, p.y));
    c
}

fn neighbours() -> Vec<V2> {
    (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| v(dx, dy))).filter(|d| *d != v(0, 0)).collect()
}

#[test]
fn snap_follows_the_dominant_axis_and_ties_are_diagonal() {
    for dx in -4..=4 {
        for dy in -4..=4 {
            let got = skill::snap(v(dx, dy));
            let want = if dx.abs() > dy.abs() {
                v(dx.signum(), 0)
            } else if dy.abs() > dx.abs() {
                v(0, dy.signum())
            } else {
                v(dx.signum(), dy.signum())
            };
            assert_eq!(got, want, "({dx}, {dy})");
        }
    }
    assert_eq!(skill::snap(v(2, 1)), v(1, 0), "a knight's offset snaps straight");
    assert_eq!(skill::snap(v(-1, -3)), v(0, -1));
    assert_eq!(skill::snap(v(3, -3)), v(1, -1), "the exact diagonal");
    assert_eq!(skill::snap(v(0, 0)), v(0, 0), "no offset, no direction");
}

#[test]
fn cone_is_read_and_drawn_from_the_user() {
    let s = skill_with("CONE");
    assert_eq!(skill::area(&s), Area::Cone);
    assert!(skill::is_area(&s));
    for a in [Area::Line, Area::Arc, Area::Cone] {
        assert!(skill::directed(a) && skill::from_user(a), "{a:?} reads a direction from the user");
    }
    for a in [Area::Plus, Area::Burst, Area::Ring, Area::None, Area::Sweep] {
        assert!(!skill::directed(a), "{a:?} reads no direction");
    }
}

/// The classic guarantee: every adjacent source gives the tiles of the pre-K2 formulas (LINE: from + d x 1..3; ARC:
/// the tile ahead and its side neighbours, or SWEEP's quadrant on a diagonal).
#[test]
fn an_adjacent_source_keeps_the_pre_k2_tiles() {
    let (line, arc, sweep) = (skill_with("LINE"), skill_with("ARC"), skill_with("SWEEP"));
    for d in neighbours() {
        let src = ME + d;
        let old_line: Vec<V2> = (1..4).map(|i| ME + d * i).collect();
        assert_eq!(skill::area_cells(&line, ME, src), old_line, "LINE toward {d:?}");
        let old_arc = if d.x != 0 && d.y != 0 {
            skill::area_cells(&sweep, ME, src)
        } else {
            let across = v(d.y, d.x);
            vec![src - across, src, src + across]
        };
        assert_eq!(sorted(skill::area_cells(&arc, ME, src)), sorted(old_arc), "ARC toward {d:?}");
    }
}

#[test]
fn line_runs_three_tiles_from_the_source_onward() {
    let s = skill_with("LINE");
    assert_eq!(skill::area_cells(&s, ME, v(8, 5)), [v(8, 5), v(9, 5), v(10, 5)], "east from afar");
    assert_eq!(skill::area_cells(&s, ME, v(7, 6)), [v(7, 6), v(8, 6), v(9, 6)], "(2, 1) snaps east");
    assert_eq!(skill::area_cells(&s, ME, v(3, 3)), [v(3, 3), v(2, 2), v(1, 1)], "a tie runs the diagonal");
    assert!(skill::area_cells(&s, ME, ME).is_empty(), "the user's own tile: no direction");
}

#[test]
fn arc_is_drawn_on_the_source() {
    let s = skill_with("ARC");
    assert_eq!(sorted(skill::area_cells(&s, ME, v(5, 2))), [v(4, 2), v(5, 2), v(6, 2)], "north: across the direction");
    assert_eq!(sorted(skill::area_cells(&s, ME, v(8, 6))), [v(8, 5), v(8, 6), v(8, 7)], "(3, 1) snaps east");
    // the diagonal L: the source the corner, its arms back toward the user's row and column
    assert_eq!(sorted(skill::area_cells(&s, ME, v(7, 7))), [v(6, 7), v(7, 6), v(7, 7)], "south-east at 2");
    assert_eq!(sorted(skill::area_cells(&s, ME, v(3, 7))), [v(3, 6), v(3, 7), v(4, 7)], "south-west at 2");
    assert!(skill::area_cells(&s, ME, ME).is_empty());
}

#[test]
fn cone_is_the_source_then_three_across() {
    let s = skill_with("CONE");
    assert_eq!(sorted(skill::area_cells(&s, ME, v(6, 5))), [v(6, 5), v(7, 4), v(7, 5), v(7, 6)], "east, adjacent");
    assert_eq!(sorted(skill::area_cells(&s, ME, v(5, 2))), [v(4, 1), v(5, 1), v(5, 2), v(6, 1)], "north from afar");
    assert_eq!(sorted(skill::area_cells(&s, ME, v(6, 6))), [v(6, 6), v(6, 7), v(7, 6), v(7, 7)], "diagonal: the source and its 3 neighbours away");
    assert!(skill::area_cells(&s, ME, ME).is_empty());
    for d in neighbours() {
        let cells = skill::area_cells(&s, ME, ME + d);
        assert_eq!(cells.len(), 4, "toward {d:?}");
        assert!(cells.contains(&(ME + d)) && !cells.contains(&ME), "the source in, the user out ({d:?})");
        assert!(cells.iter().all(|c| ME.dist(*c) >= ME.dist(ME + d)), "never back toward the user ({d:?})");
    }
}
