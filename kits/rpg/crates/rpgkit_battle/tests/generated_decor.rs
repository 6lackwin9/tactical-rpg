//! DECOR-GEN1 (PR7-86) on a scratch catalog built here (no shipped data): a generated board (an encounter with no
//! rows and no decor) draws 2 to 6 display-only decor pieces from its `decor_pool`, from the battle seed alone: the
//! same seed draws the same; never on an exit, a deploy cell or a hold tile; cover keys on blocked inner cells, props
//! on back cells (the top row, or the side columns in rows 0 to 2). Authored rows or decor draw none; the rules never read it (the board and the fight are unchanged).

use indexmap::IndexMap;
use rpgkit_battle::encounter::{generated_decor, is_back_cell, is_cover_key, is_edge_cell, Decor, GENERATED_DECOR};
use rpgkit_battle::grid::{GRID_H, GRID_W, V2};
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_data::Catalog;
use serde_json::{json, Value};
use std::sync::Arc;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 60, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({"skills": skills, "units": [unit("g_a", "party"), unit("g_b", "party"), unit("g_x", "enemy"), unit("g_y", "enemy")]}))
}

/// A generated board (no rows) in code zone `zone` with `extra` merged over the row.
fn row(zone: &str, extra: Value) -> Value {
    let mut row = json!({"id": "roam", "name": "Roam", "zone": zone, "party": ["g_a", "g_b"], "enemies": ["g_x", "g_y"], "total_blocks": 6, "unit_ap": 10, "max_rounds": 3, "rng_seed": 1337});
    for (k, v) in extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    row
}

fn session(row: &Value, pool: &[&str], seed: i64) -> Session {
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.decor_pool = pool.iter().map(|k| k.to_string()).collect();
    def.rng_seed = seed;
    Session::start(Arc::new(catalog()), def, &IndexMap::new())
}

const POOL: [&str; 4] = ["cover/v3", "lantern_pole", "barrel_stack", "crates_tall"];

fn deploy_cells(s: &Session) -> Vec<V2> {
    (0..s.w.units.len()).flat_map(|c| s.w.cells_of(c)).collect()
}

fn cells(decor: &[Decor]) -> Vec<V2> {
    decor.iter().map(|d| d.at).collect()
}

#[test]
fn a_generated_board_draws_two_to_six_decor_off_exits_and_deploy_cells() {
    for zone in ["stone_arena", "ruined_courtyard", "lava_cavern"] {
        for seed in 1..=40 {
            let s = session(&row(zone, json!({})), &POOL, seed);
            let n = s.def.decor.len() as i32;
            assert!((GENERATED_DECOR.0..=GENERATED_DECOR.1).contains(&n), "{zone} seed {seed}: {n} decor");
            let deployed = deploy_cells(&s);
            let at = cells(&s.def.decor);
            for d in &s.def.decor {
                assert!(s.w.grid.is_valid(d.at), "{zone} seed {seed}: {d:?} off the grid");
                assert!(!s.w.grid.is_exit(d.at), "{zone} seed {seed}: {d:?} on an exit");
                assert!(!deployed.contains(&d.at), "{zone} seed {seed}: {d:?} on a deploy cell");
                assert_eq!(at.iter().filter(|c| **c == d.at).count(), 1, "{zone} seed {seed}: two decor on {}", d.at);
                assert!(POOL.contains(&d.art.as_str()), "{zone} seed {seed}: {d:?} not from the pool");
                if is_cover_key(&d.art) && !is_edge_cell(d.at) {
                    assert!(!s.w.grid.is_walkable(d.at), "{zone} seed {seed}: cover {d:?} on a walkable inner cell");
                }
                if is_cover_key(&d.art) {
                    assert!(is_edge_cell(d.at) || !s.w.grid.is_walkable(d.at), "{zone} seed {seed}: cover {d:?} in the open");
                } else {
                    assert!(is_back_cell(d.at), "{zone} seed {seed}: prop {d:?} off the back of the board");
                    assert!(d.at.y == 0 || d.at.x == 0 || d.at.x == GRID_W - 1, "{zone} seed {seed}: prop {d:?} inside");
                    assert!(d.at.y <= 2, "{zone} seed {seed}: prop {d:?} in the fight area (rows 3-8)");
                }
            }
        }
    }
}

#[test]
fn the_same_seed_draws_the_same_decor_and_seeds_differ() {
    let r = row("ruined_courtyard", json!({}));
    let first = session(&r, &POOL, 77).def.decor;
    assert_eq!(first, session(&r, &POOL, 77).def.decor, "same seed, same decor");
    let layouts: Vec<Vec<Decor>> = (1..=10).map(|seed| session(&r, &POOL, seed).def.decor).collect();
    assert!(layouts.iter().any(|l| *l != layouts[0]), "ten seeds draw one layout");
}

#[test]
fn cover_keys_take_blocked_inner_cells_when_the_board_has_them() {
    // boards with inner blocks (deep water, lava): a cover-only pool lands on them while any is free, then on the edge
    for zone in ["river_crossing", "lava_cavern"] {
        for seed in 1..=20 {
            let s = session(&row(zone, json!({})), &["cover/v3"], seed);
            let deployed = deploy_cells(&s);
            let blocks = (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).filter(|c| !is_edge_cell(*c) && !s.w.grid.is_walkable(*c) && !s.w.grid.is_exit(*c) && !deployed.contains(c)).count();
            assert!(blocks > 0, "{zone} has inner blocks");
            let inner = s.def.decor.iter().filter(|d| !is_edge_cell(d.at)).count();
            assert_eq!(inner, s.def.decor.len().min(blocks), "{zone} seed {seed}: {:?}", s.def.decor);
            assert!(s.def.decor.iter().filter(|d| !is_edge_cell(d.at)).all(|d| !s.w.grid.is_walkable(d.at)));
        }
    }
    // an open floor has no inner blocks: cover falls back to the edge
    let open = session(&row("stone_arena", json!({})), &["cover"], 5);
    assert!(open.def.decor.iter().all(|d| is_edge_cell(d.at)), "{:?}", open.def.decor);
}

#[test]
fn hold_tiles_exits_and_taken_cells_stay_clear() {
    // a hold on two edge tiles and exits on the right edge: no decor on them over many seeds
    let tiles = [V2::new(0, 0), V2::new(11, 8)];
    let exits = [V2::new(11, 3), V2::new(11, 4), V2::new(11, 5)];
    let r = row("stone_arena", json!({"objective": "hold", "objective_tiles": [[0, 0], [11, 8]], "exits": [[11, 3], [11, 4], [11, 5]]}));
    for seed in 1..=60 {
        let s = session(&r, &["lantern_pole"], seed);
        for d in &s.def.decor {
            assert!(!tiles.contains(&d.at) && !exits.contains(&d.at), "seed {seed}: {d:?}");
        }
    }
    // the bare function: every edge cell taken but one leaves room for one piece only
    let s = session(&row("stone_arena", json!({})), &[], 1);
    let edge: Vec<V2> = (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).filter(|c| is_edge_cell(*c)).collect();
    let free = edge.iter().copied().find(|c| !s.w.grid.is_exit(*c)).unwrap();
    let taken: Vec<V2> = edge.iter().copied().filter(|c| *c != free).collect();
    let one = generated_decor(&s.w.grid, &["barrel_stack".to_string()], 9, "stone_arena", &taken);
    assert_eq!(cells(&one), vec![free]);
}

#[test]
fn props_stand_only_at_the_back_and_cover_may_use_any_edge() {
    let s = session(&row("stone_arena", json!({})), &[], 1);
    let all: Vec<V2> = (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).collect();
    let back: Vec<V2> = all.iter().copied().filter(|c| is_back_cell(*c)).collect();
    assert!(back.iter().all(|c| c.y == 0 || ((c.x == 0 || c.x == GRID_W - 1) && c.y <= 2)), "{back:?}");
    // every back cell taken: a prop pool places nothing, a cover pool still finds the front edge
    for seed in 1..=20 {
        assert!(generated_decor(&s.w.grid, &["lantern_pole".to_string()], seed, "stone_arena", &back).is_empty(), "seed {seed}");
        let cover = generated_decor(&s.w.grid, &["cover".to_string()], seed, "stone_arena", &back);
        assert!(!cover.is_empty() && cover.iter().all(|d| is_edge_cell(d.at) && !is_back_cell(d.at)), "seed {seed}: {cover:?}");
    }
}

#[test]
fn authored_rows_or_decor_or_no_pool_draw_no_generated_decor() {
    let authored = row("stone_arena", json!({"rows": vec![".".repeat(12); 9]}));
    assert!(session(&authored, &POOL, 3).def.decor.is_empty(), "authored rows: no generated decor");
    let own = row("stone_arena", json!({"decor": [{"art": "barrel_stack", "x": 0, "y": 0}]}));
    assert_eq!(session(&own, &POOL, 3).def.decor, vec![Decor { art: "barrel_stack".into(), at: V2::new(0, 0) }], "its own decor stands");
    assert!(session(&row("stone_arena", json!({})), &[], 3).def.decor.is_empty(), "no pool: no decor");
}

/// The rules never read the decor: with and without a pool the board, the deploy and a whole fight are the same.
#[test]
fn generated_decor_changes_no_walkability_and_no_fight() {
    let r = row("lava_cavern", json!({}));
    let (mut a, mut b) = (session(&r, &POOL, 11), session(&r, &[], 11));
    assert!(!a.def.decor.is_empty() && b.def.decor.is_empty());
    assert_eq!(a.w.grid.terrain, b.w.grid.terrain);
    assert_eq!(a.w.grid.positions, b.w.grid.positions);
    for s in [&mut a, &mut b] {
        loop {
            s.auto_draft("balanced");
            if !s.confirm_plan() {
                break;
            }
            s.resolve_round();
            if s.combat_over || !s.can_advance_round() {
                break;
            }
            s.advance_round();
        }
    }
    assert_eq!(a.resolver.log.len(), b.resolver.log.len());
    assert_eq!(a.w.grid.positions, b.w.grid.positions);
    assert_eq!(a.w.units.iter().map(|u| u.hp).collect::<Vec<_>>(), b.w.units.iter().map(|u| u.hp).collect::<Vec<_>>());
    assert_eq!(a.final_outcome(), b.final_outcome());
}
