//! CombatGrid, Terrain, CombatZones, CoverRules (scripts/combat/arena/). Units are battle indices; a unit keeps its
//! tile after it falls (Godot leaves dead units in combatant_positions), and leaves the grid when it withdraws.

use aegis_data::value;
use aegis_data::Catalog;
use std::ops::{Add, Mul, Sub};

pub const GRID_W: i32 = 12;
pub const GRID_H: i32 = 9;
pub const DEPLOY_ROWS: [i32; 7] = [4, 3, 5, 2, 6, 1, 7];
pub const DEPLOY_ANCHOR_Y: i32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct V2 {
    pub x: i32,
    pub y: i32,
}

pub const NO_CELL: V2 = V2 { x: -1, y: -1 };
pub const ZERO: V2 = V2 { x: 0, y: 0 };
/// GridPath.DIRS / CombatPlan.DIRS order.
pub const DIRS: [V2; 4] = [V2 { x: 1, y: 0 }, V2 { x: 0, y: -1 }, V2 { x: -1, y: 0 }, V2 { x: 0, y: 1 }];

impl V2 {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    pub fn dist(self, o: V2) -> i32 {
        (self.x - o.x).abs() + (self.y - o.y).abs()
    }
}

impl Add for V2 {
    type Output = V2;
    fn add(self, o: V2) -> V2 {
        V2::new(self.x + o.x, self.y + o.y)
    }
}

impl Sub for V2 {
    type Output = V2;
    fn sub(self, o: V2) -> V2 {
        V2::new(self.x - o.x, self.y - o.y)
    }
}

impl Mul<i32> for V2 {
    type Output = V2;
    fn mul(self, k: i32) -> V2 {
        V2::new(self.x * k, self.y * k)
    }
}

impl std::fmt::Display for V2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// Terrain.Type in enum order (ids are the lower-case names; the catalog keys).
pub const TERRAIN_IDS: [&str; 17] = [
    "floor", "grass", "dirt", "stone", "sand", "snow", "shallow_water", "deep_water", "mud", "lava", "wall", "tree", "bridge", "rubble",
    "burning_ground", "toxic_bog", "ice",
];
const GLYPHS: [char; 17] = ['.', ',', ':', 'o', 's', '*', '~', 'W', 'm', 'L', '#', 'T', '=', 'r', 'f', 'p', 'i'];
const BLOCKED: [usize; 4] = [7, 9, 10, 11]; // deep water, lava, wall, tree
const SLOW: [usize; 5] = [4, 5, 6, 8, 13]; // sand, snow, shallow water, mud, rubble

#[derive(Debug, Clone, PartialEq)]
pub struct Hazard {
    pub status: String,
    pub potency: i32,
    pub magnitude: i32,
    pub rounds: i32,
    /// "enter", "round_end" or "both".
    pub on: String,
}

#[derive(Debug, Clone)]
pub struct TerrainRow {
    pub walkable: bool,
    pub move_cost: i32,
    pub cover_guard: f64,
    pub hazard: Option<Hazard>,
}

/// Terrain numbers per type: the catalog row, else the code defaults.
#[derive(Debug, Clone)]
pub struct TerrainTable {
    pub rows: Vec<TerrainRow>,
}

impl TerrainTable {
    pub fn from_catalog(cat: &Catalog) -> Self {
        let rows = TERRAIN_IDS
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let walkable = !BLOCKED.contains(&i);
                let default_cost = if SLOW.contains(&i) { 2 } else { 1 };
                let Some(r) = cat.terrain.get(*id) else {
                    return TerrainRow { walkable, move_cost: if walkable { default_cost } else { 1 }, cover_guard: 0.0, hazard: None };
                };
                let status = value::string(r, "hazard_status", "");
                let hazard = (!status.is_empty() && walkable).then(|| Hazard {
                    status,
                    potency: value::int(r, "hazard_potency", 100),
                    magnitude: value::int(r, "hazard_magnitude", 0),
                    rounds: value::int(r, "hazard_rounds", 1),
                    on: value::string(r, "hazard_on", "both"),
                });
                TerrainRow {
                    walkable,
                    move_cost: if walkable { value::int(r, "move_cost", default_cost).max(1) } else { 1 },
                    cover_guard: value::float(r, "cover_guard", 0.0),
                    hazard,
                }
            })
            .collect();
        Self { rows }
    }
}

/// CombatZones.ZONES (glyph rows).
pub fn zone_rows(zone: &str) -> Option<[&'static str; 9]> {
    Some(match zone {
        "stone_arena" => ["............"; 9],
        "forest_clearing" => ["TTT,,TT,,TTT", "T,,,,,,,,,,T", ",,,,:,,,,TT,", ",,,,:,,,,,,,", ":::::::::::,", ",,,,,,,,,,,T", "T,,,,,,,,,,,", "TTT,,,,,T,,T", "TTTT,,,TTTTT"],
        "river_crossing" => [",,,T,,,,~WW,", ",,,,,,,,~WW,", "T,,,,,,,~WW,", ",,,,,,,,~WW,", "::::::::~==:", ",,,,,,,,~WW,", ",TT,,,,,~WW,", ",,,,,,,,~WWT", ",,,,T,,,~WW,"],
        "ruined_courtyard" => ["###o###o####", "#oooooooooo#", "#oo.r...o..#", "ooo........#", "o...........", "#oo.....r..#", "#oo.r...o..#", "#oooooooooo#", "####o###o###"],
        "desert_outpost" => ["ssss###sssss", "s##ssssss##s", "sssss::sssss", "sss:::::ssss", "sss:::::::::", "ssss:::ssrr#", "s##sssssssss", "sssssrr#ssss", "###sssssssss"],
        "frozen_pass" => ["###**TT**###", "#**********#", "Ti**r*******", "*****::**i**", "************", "*****::**rri", "T**********#", "#*******T**#", "###******###"],
        "lava_cavern" => ["############", "#LLLf...LLL#", "#LL......rr#", "#f...rr....#", "#...........", "#rr........#", "#LL......LL#", "#LLL.rrfLLL#", "############"],
        "swamp_bog" => ["TmmmT~~~mmmT", "mmm,,,,~~mmm", "~mm,,mm,,,,m", "~~~,,,,,m,,,", "m,,,,,,,,,,,", "p,,m,,,,,pm~", "T,,,,,,m,~~~", "mmm~~,,,,mmT", "TTT~~mmmmTTT"],
        _ => return None,
    })
}

#[derive(Debug, Clone)]
pub struct Grid {
    pub open: bool,
    pub exits: Vec<V2>,
    /// Terrain type index per tile (row-major).
    pub terrain: Vec<usize>,
    pub table: TerrainTable,
    /// Tile per battle unit (None = never placed or withdrawn). Dead units keep their tile.
    pub positions: Vec<Option<V2>>,
    pub withdrawn: Vec<usize>,
}

impl Grid {
    pub fn new(open: bool, exits: Vec<V2>, table: TerrainTable, unit_count: usize) -> Self {
        Self { open, exits, terrain: vec![0; (GRID_W * GRID_H) as usize], table, positions: vec![None; unit_count], withdrawn: Vec::new() }
    }

    pub fn apply_zone(&mut self, zone: &str) {
        self.terrain = vec![0; (GRID_W * GRID_H) as usize];
        if let Some(rows) = zone_rows(zone) {
            for (y, row) in rows.iter().enumerate() {
                for (x, ch) in row.chars().enumerate() {
                    self.terrain[y * GRID_W as usize + x] = GLYPHS.iter().position(|g| *g == ch).unwrap_or(0);
                }
            }
        }
    }

    pub fn is_valid(&self, c: V2) -> bool {
        c.x >= 0 && c.x < GRID_W && c.y >= 0 && c.y < GRID_H
    }

    fn row(&self, c: V2) -> &TerrainRow {
        let t = if self.is_valid(c) { self.terrain[(c.y * GRID_W + c.x) as usize] } else { 0 };
        &self.table.rows[t]
    }

    pub fn terrain_id(&self, c: V2) -> &'static str {
        if self.is_valid(c) {
            TERRAIN_IDS[self.terrain[(c.y * GRID_W + c.x) as usize]]
        } else {
            "floor"
        }
    }

    pub fn is_walkable(&self, c: V2) -> bool {
        self.is_valid(c) && self.row(c).walkable
    }

    pub fn move_cost(&self, c: V2) -> i32 {
        self.row(c).move_cost
    }

    pub fn cover_guard(&self, c: V2) -> f64 {
        if self.is_valid(c) {
            self.row(c).cover_guard
        } else {
            0.0
        }
    }

    pub fn has_hazard(&self, c: V2) -> bool {
        self.is_valid(c) && self.row(c).hazard.is_some()
    }

    pub fn hazard_at(&self, c: V2) -> Option<&Hazard> {
        if self.is_valid(c) {
            self.row(c).hazard.as_ref()
        } else {
            None
        }
    }

    pub fn is_exit(&self, c: V2) -> bool {
        if self.open {
            c.x == 0 || c.x == GRID_W - 1 || c.y == 0 || c.y == GRID_H - 1
        } else {
            self.exits.contains(&c)
        }
    }

    /// Half tiles from the arena middle (tie-break toward central tiles).
    pub fn centre_distance(c: V2) -> i32 {
        (2 * c.x - (GRID_W - 1)).abs() + (2 * c.y - (GRID_H - 1)).abs()
    }

    pub fn is_party_half(c: V2) -> bool {
        c.x * 2 < GRID_W
    }

    pub fn pos(&self, u: usize) -> Option<V2> {
        self.positions[u]
    }

    /// get_position: (-1, -1) when the unit has no tile.
    pub fn position(&self, u: usize) -> V2 {
        self.positions[u].unwrap_or(NO_CELL)
    }
}
