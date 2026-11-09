//! Balance sim variants (tools/sim/sim_variants.gd, BAL2_SIM.md section 3): the "newest" loadout, the spend variants,
//! the start-distance placement and the clumped enemy block (BAL3 3.2). Pure helpers over the catalog and the grid.

use rpgkit_battle::grid::{Grid, V2, GRID_H, GRID_W};
use rpgkit_battle::world::{UnitId, World};
use indexmap::IndexMap;
use rpgkit_data::{Catalog, UnitDef};
use rpgkit_rules::stats::{spend_pool, MAX_CLASS_SLOTS};
use serde_json::{json, Map, Value};

/// 3.1: owned class-path families of a `level` unit (path level 1 first in path order, then the rest by path level
/// descending, ties in path order; area families after every other), the first MAX_CLASS_SLOTS.
pub fn newest_loadout(cat: &Catalog, class_id: &str, level: i32) -> Vec<String> {
    let mut first: Vec<(String, i32)> = Vec::new();
    if let Some(cls) = cat.classes.get(class_id) {
        for (step_level, skill) in cls.path_steps() {
            if skill.is_empty() || cat.is_always_active(&skill) || step_level > level {
                continue;
            }
            let fam = cat.family_of(&skill).to_string();
            if !first.iter().any(|(f, _)| *f == fam) {
                first.push((fam, step_level));
            }
        }
    }
    let mut ranked: Vec<usize> = (0..first.len()).collect();
    ranked.sort_by(|&a, &b| {
        let (area_a, area_b) = (is_area_family(cat, &first[a].0), is_area_family(cat, &first[b].0));
        let (la, lb) = (first[a].1, first[b].1);
        area_a.cmp(&area_b).then(((la != 1) as i32).cmp(&((lb != 1) as i32))).then(lb.cmp(&la)).then(a.cmp(&b))
    });
    ranked.into_iter().take(MAX_CLASS_SLOTS).map(|i| first[i].0.clone()).collect()
}

/// A family is an area family when its tier-one row has an area shape.
pub fn is_area_family(cat: &Catalog, family: &str) -> bool {
    cat.tier_one(family).is_some_and(|s| !s.area.is_empty() && s.area != "NONE")
}

/// 3.2: the unit `spend` of variant `variant` for `class_id` at `level` (empty = class weights).
pub fn spend_for(cat: &Catalog, variant: &str, class_id: &str, level: i32) -> Map<String, Value> {
    spend_by(variant, primary_stat(cat, class_id), level)
}

/// 3.2 for catalog unit `e` at its level: its primary stat read through its role template (`primary_stat_of`; R4-1
/// slice 2, FB-144), so a templated unit spends by its template's growth identity.
pub fn spend_for_unit(cat: &Catalog, variant: &str, e: &UnitDef) -> Map<String, Value> {
    spend_by(variant, primary_stat_of(cat, e), e.level)
}

fn spend_by(variant: &str, primary: String, level: i32) -> Map<String, Value> {
    let pool = spend_pool(level);
    let mut m = Map::new();
    match variant {
        "cons" => {
            m.insert("CONS".into(), json!(pool));
        }
        "primary" => {
            m.insert(primary, json!(pool));
        }
        "mixed" => {
            let half = pool / 2;
            if primary == "CONS" {
                m.insert("CONS".into(), json!(pool));
            } else {
                m.insert(primary, json!(half));
                m.insert("CONS".into(), json!(pool - half));
            }
        }
        _ => {}
    }
    m
}

/// The stat of the largest class growth share (U2.2: the growth identity, not one level's rounded gain; an integer row
/// reads as growth / 5, so the same stat as before; ties: the first in the class's growth order).
pub fn primary_stat(cat: &Catalog, class_id: &str) -> String {
    cat.classes.get(class_id).map(|cls| largest(&cls.unit_shares().growth)).unwrap_or_default()
}

/// R4-1 slice 2 (FB-144): `primary_stat` of catalog unit `e` read through its role template
/// (`rpgkit_rules::unit::spend_growth`, `template_for`): a templated unit's template growth shares, else its class's.
pub fn primary_stat_of(cat: &Catalog, e: &UnitDef) -> String {
    largest(&rpgkit_rules::unit::spend_growth(cat, e))
}

fn largest(growth: &IndexMap<String, f64>) -> String {
    let mut best = (String::new(), -1.0);
    for (k, &v) in growth {
        if v > best.1 {
            best = (k.clone(), v);
        }
    }
    best.0
}

/// Godot's str() of a spend dict with spaces and quotes removed: {STR:6,CONS:6}.
pub fn spend_text(m: &Map<String, Value>) -> String {
    format!("{{{}}}", m.iter().map(|(k, v)| format!("{k}:{v}")).collect::<Vec<_>>().join(","))
}

fn nearest(anchors: &[V2], tile: V2) -> i32 {
    anchors.iter().map(|a| a.dist(tile)).min().unwrap_or(1 << 30)
}

fn anchors(w: &World, left: &[UnitId]) -> Vec<V2> {
    left.iter().filter(|&&c| w.has(c)).map(|&c| w.pos(c)).collect()
}

/// 3.3: moves every deployed right-side unit to the walkable, free, non-exit tile at distance `dist` from the nearest
/// left-side unit, closest to its tile (ties: lowest y, then x). Returns how many found no such tile.
pub fn apply_distance(w: &mut World, left: &[UnitId], right: &[UnitId], dist: i32) -> i32 {
    let anchors = anchors(w, left);
    let mut not_moved = 0;
    for &c in right {
        if !w.has(c) {
            continue;
        }
        let here = w.pos(c);
        let mut best: Option<V2> = None;
        let mut best_d = 1 << 30;
        for y in 0..GRID_H {
            for x in 0..GRID_W {
                let tile = V2::new(x, y);
                if !w.grid.is_walkable(tile) || w.grid.is_exit(tile) || (tile != here && w.is_occupied(tile)) || nearest(&anchors, tile) != dist {
                    continue;
                }
                let d = here.dist(tile);
                if d < best_d {
                    best_d = d;
                    best = Some(tile);
                }
            }
        }
        match best {
            None => not_moved += 1,
            Some(b) if b != here => {
                w.remove(c);
                w.place(c, b);
            }
            _ => {}
        }
    }
    not_moved
}

/// The tiles of a block (columns x rows) whose front column is `front_x` and middle row `mid_y`, front column first,
/// each column top to bottom.
fn block_tiles(front_x: i32, mid_y: i32, size: (i32, i32)) -> Vec<V2> {
    let top = mid_y - (size.1 - 1) / 2;
    (0..size.0).flat_map(|dx| (0..size.1).map(move |dy| V2::new(front_x + dx, top + dy))).collect()
}

/// BAL3 3.2: the first columns x rows deployed right-side units move into one block on the right half (middle row =
/// grid middle + `row_shift`, front column the nearest that fits, `min_dist` from the left side when > 0). False
/// (nobody moves) when no column fits.
pub fn apply_block(w: &mut World, left: &[UnitId], right: &[UnitId], size: (i32, i32), min_dist: i32, row_shift: i32) -> bool {
    let block: Vec<UnitId> = right.iter().copied().filter(|&c| w.has(c)).take((size.0 * size.1) as usize).collect();
    if block.is_empty() {
        return false;
    }
    let anchors = anchors(w, left);
    let half_rows = (size.1 - 1) / 2;
    let mid_y = (GRID_H / 2 + row_shift).clamp(half_rows, GRID_H - size.1 + half_rows);
    let mut chosen: Vec<V2> = Vec::new();
    for x in 0..=(GRID_W - size.0) {
        let tiles = block_tiles(x, mid_y, size);
        let fits = tiles.iter().all(|&t| {
            let holder = w.occupant(t);
            !Grid::is_party_half(t) && w.grid.is_walkable(t) && !w.grid.is_exit(t) && holder.is_none_or(|h| block.contains(&h)) && !(min_dist > 0 && nearest(&anchors, t) < min_dist)
        });
        if fits {
            chosen = tiles;
            break;
        }
    }
    if chosen.is_empty() {
        return false;
    }
    for &c in &block {
        w.remove(c);
    }
    for (i, &c) in block.iter().enumerate() {
        w.place(c, chosen[i]);
    }
    true
}
