//! A3.2-S1 (`docs/sessions/A3_SHAPES.md`, FB-57): the additive area shapes RING and ARC beside the legacy SWEEP. A
//! skill's `area` reads them; RING covers the 8 tiles around the user whatever the aim; ARC the 3-tile front arc
//! toward the aim (along an axis: the tile ahead and its two side neighbours; off-axis: SWEEP's quadrant); both are
//! drawn from the user, so a skill without its own reach keeps the (1, 1) band. SWEEP, PLUS, BURST and LINE are
//! unchanged. A3.2-K2 draws ARC on the source tile (`shape_direction.rs`).

use rpgkit_battle::grid::V2;
use rpgkit_battle::skill::{self, Area};
use rpgkit_data::{Catalog, SkillDef};
use serde_json::json;

const fn v(x: i32, y: i32) -> V2 {
    V2::new(x, y)
}

fn skill_with(area: &str) -> SkillDef {
    let cat = Catalog::from_value(&json!({"skills": [{"id": "t_area", "name": "Area", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0, "area": area}]}));
    cat.skills.into_iter().next().expect("the test skill")
}

fn sorted(mut c: Vec<V2>) -> Vec<(i32, i32)> {
    c.sort_by_key(|p| (p.x, p.y));
    c.into_iter().map(|p| (p.x, p.y)).collect()
}

#[test]
fn area_reads_ring_and_arc_beside_sweep() {
    for (id, want) in [("RING", Area::Ring), ("ARC", Area::Arc), ("SWEEP", Area::Sweep), ("LINE", Area::Line), ("NONE", Area::None)] {
        let s = skill_with(id);
        assert_eq!(skill::area(&s), want, "{id}");
        assert_eq!(skill::is_area(&s), want != Area::None, "{id} is an area attack");
    }
}

#[test]
fn ring_is_the_eight_tiles_around_the_user_whatever_the_aim() {
    let s = skill_with("RING");
    let me = v(4, 4);
    let ring = skill::area_cells(&s, me, v(9, 9));
    assert_eq!(ring.len(), 8);
    assert!(!ring.contains(&me), "never the user's tile");
    assert!(ring.iter().all(|c| (c.x - me.x).abs() <= 1 && (c.y - me.y).abs() <= 1), "each a neighbour");
    assert_eq!(sorted(ring), sorted(skill::area_cells(&s, me, me)), "the aim is not read");
}

#[test]
fn arc_is_the_front_arc_toward_the_aim() {
    let s = skill_with("ARC");
    let me = v(4, 4);
    assert_eq!(sorted(skill::area_cells(&s, me, v(5, 4))), [(5, 3), (5, 4), (5, 5)], "east: ahead and both sides");
    // A3.2-K2 (IR6): the arc is drawn on the source tile (S1 drew it beside the user whatever the aim)
    assert_eq!(sorted(skill::area_cells(&s, me, v(4, 1))), [(3, 1), (4, 1), (5, 1)], "north from afar: on the source");
    let sweep = skill_with("SWEEP");
    for aim in [v(5, 5), v(3, 3), v(5, 3), v(3, 5)] {
        assert_eq!(skill::area_cells(&s, me, aim), skill::area_cells(&sweep, me, aim), "off-axis as SWEEP's quadrant at {aim:?}");
    }
    assert!(skill::area_cells(&s, me, me).is_empty(), "no direction, no tiles");
}

#[test]
fn legacy_shapes_unchanged() {
    let me = v(4, 4);
    assert_eq!(sorted(skill::area_cells(&skill_with("SWEEP"), me, v(5, 5))), [(4, 5), (5, 4), (5, 5)]);
    assert_eq!(sorted(skill::area_cells(&skill_with("LINE"), me, v(5, 4))), [(5, 4), (6, 4), (7, 4)]);
    assert_eq!(skill::area_cells(&skill_with("PLUS"), me, v(6, 4)).len(), 5);
    assert_eq!(skill::area_cells(&skill_with("BURST"), me, v(6, 4)).len(), 9);
}
