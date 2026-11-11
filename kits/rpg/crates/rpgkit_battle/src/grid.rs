//! CombatGrid, Terrain, CombatZones, CoverRules (scripts/combat/arena/). Units are battle indices; a unit keeps its
//! tile after it falls (Godot leaves dead units in combatant_positions), and leaves the grid when it withdraws.

use aegis_core::value;
use indexmap::IndexMap;
use rpgkit_data::looks::{Face, Relief, TerrainLook};
use rpgkit_data::{Catalog, ZoneDef};
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

/// Terrain.Type in enum order (ids are the lower-case names; the catalog keys): the built-in terrain types, the code
/// zones' alphabet (`GLYPHS`) and the types of a catalog without terrain rows. LAYER-DOWN-2: a catalog's terrain rows
/// add their new types after these (`terrain_ids`, `TerrainTable::ids`); the battle reads the table, not this list.
pub const TERRAIN_IDS: [&str; 17] = [
    "floor", "grass", "dirt", "stone", "sand", "snow", "shallow_water", "deep_water", "mud", "lava", "wall", "tree", "bridge", "rubble",
    "burning_ground", "toxic_bog", "ice",
];
/// The terrain glyph per type, in TERRAIN_IDS order (CombatZones' row glyphs; also the glyphs of authored rows).
pub const GLYPHS: [char; 17] = ['.', ',', ':', 'o', 's', '*', '~', 'W', 'm', 'L', '#', 'T', '=', 'r', 'f', 'p', 'i'];
/// LAYER-DOWN-2: the terrain row key giving a new type its board glyph (one character; a built-in type keeps its
/// `GLYPHS` one and leaves the key empty).
pub const GLYPH_KEY: &str = "glyph";

/// LAYER-DOWN-2: the terrain types of a catalog whose terrain rows have the ids `row_ids` (in file order): the built-in
/// types (`TERRAIN_IDS`, in their order, so a built-in type keeps its index), then each new row id once in file order.
pub fn terrain_ids_of<'a>(row_ids: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut out: Vec<String> = TERRAIN_IDS.iter().map(|t| t.to_string()).collect();
    for id in row_ids {
        if !id.is_empty() && !out.iter().any(|t| t == id) {
            out.push(id.to_string());
        }
    }
    out
}

/// LAYER-DOWN-2: the terrain types of catalog `cat` (`terrain_ids_of` its terrain rows).
pub fn terrain_ids(cat: &Catalog) -> Vec<String> {
    terrain_ids_of(cat.terrain.keys().map(String::as_str))
}

/// LAYER-DOWN-2: the board glyph of terrain type `id` with row `row`: a built-in type's `GLYPHS` one, else the row's
/// `glyph` (its first character; None when it has none, so no board row can place the type).
pub fn glyph_of(id: &str, row: Option<&value::Obj>) -> Option<char> {
    match TERRAIN_IDS.iter().position(|t| *t == id) {
        Some(i) => Some(GLYPHS[i]),
        None => row.and_then(|r| value::string(r, GLYPH_KEY, "").chars().next()),
    }
}
/// P1.2-S1: the missing-key fallback of a terrain row's `walkable` (and of a type with no row): deep water, lava, wall
/// and tree block. The rules read the row's `walkable` (`TerrainTable`); this only fills a row that leaves it out (the
/// frozen kit's rows, a row written before the field existed).
const BLOCKED: [usize; 4] = [7, 9, 10, 11]; // deep water, lava, wall, tree

/// P1.2-S1: the missing-key fallback of a terrain row's `walkable` by type id (an unknown id is walkable); the rules read
/// the row (`Grid::is_walkable`, `TerrainTable`).
pub fn terrain_walkable(id: &str) -> bool {
    TERRAIN_IDS.iter().position(|t| *t == id).is_none_or(|i| !BLOCKED.contains(&i))
}

/// P1.2-S2 (FB-109): the missing-key rule of a terrain row's `walkable` and `stops_forced_move`, shared by the loader
/// (`rpgkit_content` `source`) and `TerrainTable`: `walkable` as given, else its type's (`terrain_walkable`);
/// `stops_forced_move` as given, else not walkable. `(walkable, stops_forced_move)`.
pub fn terrain_walk_keys(id: &str, walkable: Option<bool>, stops_forced_move: Option<bool>) -> (bool, bool) {
    let walkable = walkable.unwrap_or_else(|| terrain_walkable(id));
    (walkable, stops_forced_move.unwrap_or(!walkable))
}

/// A row's boolean `key` as `value::boolean` reads it, None when it is missing or not a boolean or number (the
/// missing-key rule applies).
fn given_bool(r: &value::Obj, key: &str) -> Option<bool> {
    match r.get(key) {
        Some(serde_json::Value::Bool(b)) => Some(*b),
        Some(serde_json::Value::Number(n)) => n.as_f64().map(|f| f != 0.0),
        _ => None,
    }
}
/// The move cost of a type with no catalog row (a row's `move_cost` is read from the row).
const SLOW: [usize; 5] = [4, 5, 6, 8, 13]; // sand, snow, shallow water, mud, rubble

/// The source name a terrain hazard's status carries.
pub const TERRAIN_SOURCE: &str = "terrain";

#[derive(Debug, Clone, PartialEq)]
pub struct Hazard {
    pub status: String,
    pub potency: i32,
    pub magnitude: i32,
    pub rounds: i32,
    /// "enter", "round_end" or "both".
    pub on: String,
}

/// P1.2-S2 (FB-51d, A4.2 C08): a terrain row's hazard status as an EffectSpec, the cell vocabulary's converter (as
/// `ground::effect_of` for ground hazard rows): an inflict (rolled when its status row is `resistable`) carrying the row's
/// `hazard_potency`, `hazard_rounds` and `hazard_magnitude`, with the source `TERRAIN_SOURCE`.
pub fn effect_of(h: &Hazard) -> rpgkit_rules::status::EffectSpec {
    rpgkit_rules::status::EffectSpec::inflict(&h.status, h.potency, h.rounds, h.magnitude, TERRAIN_SOURCE)
}

#[derive(Debug, Clone)]
pub struct TerrainRow {
    pub walkable: bool,
    /// P1.2-S1 (FB-51): a forced move cannot enter the tile (the row's `stops_forced_move`, by default `!walkable`).
    pub stops_forced_move: bool,
    pub move_cost: i32,
    pub cover_guard: f64,
    pub hazard: Option<Hazard>,
}

/// Terrain numbers per type: the catalog row, else the code defaults. LAYER-DOWN-2: one entry per type of the
/// catalog (`terrain_ids`: the built-in types, then the rows' new ones), with its id and board glyph.
#[derive(Debug, Clone)]
pub struct TerrainTable {
    pub rows: Vec<TerrainRow>,
    /// LAYER-DOWN-2: the type id of each row (a grid tile's terrain index points here).
    pub ids: Vec<String>,
    /// LAYER-DOWN-2: the board glyph of each row (None: a new type without one).
    pub glyphs: Vec<Option<char>>,
    /// PR5-33: the look of each row (`TerrainLook::of` its catalog row; presentation only, no rule reads it).
    pub looks: Vec<TerrainLook>,
}

impl TerrainTable {
    /// The built-in types with their code defaults (a catalog without terrain rows).
    pub fn builtin() -> Self {
        Self::from_catalog(&Catalog::default())
    }

    pub fn from_catalog(cat: &Catalog) -> Self {
        let ids = terrain_ids(cat);
        let glyphs = ids.iter().map(|id| glyph_of(id, cat.terrain.get(id))).collect();
        let looks = ids.iter().map(|id| TerrainLook::of(cat.terrain.get(id))).collect();
        let rows = ids
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let default_cost = if SLOW.contains(&i) { 2 } else { 1 };
                let Some(r) = cat.terrain.get(id) else {
                    let (walkable, stops_forced_move) = terrain_walk_keys(id, None, None);
                    return TerrainRow { walkable, stops_forced_move, move_cost: if walkable { default_cost } else { 1 }, cover_guard: 0.0, hazard: None };
                };
                // P1.2-S1: the row's own `walkable` and `stops_forced_move`, else their missing-key fallbacks (P1.2-S2: the
                // loader's rule, `terrain_walk_keys`)
                let (walkable, stops_forced_move) = terrain_walk_keys(id, given_bool(r, "walkable"), given_bool(r, "stops_forced_move"));
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
                    stops_forced_move,
                    move_cost: if walkable { value::int(r, "move_cost", default_cost).max(1) } else { 1 },
                    cover_guard: value::float(r, "cover_guard", 0.0),
                    hazard,
                }
            })
            .collect();
        Self { rows, ids, glyphs, looks }
    }

    /// PR5-33: the look of terrain type `id` (an unknown type: the default look, flat ground).
    pub fn look(&self, id: &str) -> &TerrainLook {
        static FLAT: TerrainLook = TerrainLook { relief: Relief::Ground, depth: 0, prop: String::new(), color: None, face: Face::Earth };
        self.index_of(id).and_then(|i| self.looks.get(i)).unwrap_or(&FLAT)
    }

    /// LAYER-DOWN-2: the type index of board glyph `ch` (None: no type has it).
    pub fn index_of_glyph(&self, ch: char) -> Option<usize> {
        self.glyphs.iter().position(|g| *g == Some(ch))
    }

    /// LAYER-DOWN-2: the type index of terrain id `id`.
    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.ids.iter().position(|t| t == id)
    }

    /// ENCOUNTER2 R0 over this table's glyphs: GRID_H strings of GRID_W glyphs of its types.
    pub fn rows_valid(&self, rows: &[String]) -> bool {
        rows.len() == GRID_H as usize && rows.iter().all(|r| r.chars().count() == GRID_W as usize && r.chars().all(|c| self.index_of_glyph(c).is_some()))
    }
}

/// CombatZones.zone_ids: the zones in table order.
pub const ZONE_IDS: [&str; 8] = ["stone_arena", "forest_clearing", "river_crossing", "ruined_courtyard", "desert_outpost", "frozen_pass", "lava_cavern", "swamp_bog"];

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

/// ENCOUNTER2 R0: authored terrain rows are GRID_H strings of GRID_W known terrain glyphs (the built-in types'; a
/// catalog's own types: `TerrainTable::rows_valid`).
pub fn rows_valid(rows: &[String]) -> bool {
    rows.len() == GRID_H as usize && rows.iter().all(|r| r.chars().count() == GRID_W as usize && r.chars().all(|c| GLYPHS.contains(&c)))
}

/// HOOKS1 ZN2: the data zone row of battle zone `zone` (kind `zones`); None for a code zone (a row reusing a code id
/// is a check error and never stands in for the code zone) or an unknown id.
pub fn data_zone<'a>(zone: &str, zones: &'a IndexMap<String, ZoneDef>) -> Option<&'a ZoneDef> {
    zones.get(zone).filter(|_| !ZONE_IDS.contains(&zone))
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
    /// HOOKS1 GH1-GH7 (rule `ground_hazards`): the ground hazard on each tile that has one, oldest placement first
    /// (`crate::ground`); empty in classic.
    pub ground: IndexMap<V2, crate::ground::Placed>,
}

impl Grid {
    pub fn new(open: bool, exits: Vec<V2>, table: TerrainTable, unit_count: usize) -> Self {
        Self { open, exits, terrain: vec![0; (GRID_W * GRID_H) as usize], table, positions: vec![None; unit_count], withdrawn: Vec::new(), ground: IndexMap::new() }
    }

    pub fn apply_zone(&mut self, zone: &str) {
        self.apply_rows(zone_rows(zone).into_iter().flatten());
    }

    /// The battle board (ENCOUNTER2 R0, HOOKS1 ZN3): the encounter's authored rows when they are valid (`rows_valid`),
    /// else a code zone's rows (unchanged), else the data zone row's (`zones`, kind `zones`: its rows when valid, else
    /// 9 rows of floor); an unknown zone is floor. The zone still picks the backdrop and tiles.
    pub fn apply_board(&mut self, zone: &str, rows: &[String], zones: &IndexMap<String, ZoneDef>) {
        if self.table.rows_valid(rows) {
            self.apply_rows(rows.iter().map(String::as_str));
        } else if let Some(z) = data_zone(zone, zones).filter(|z| self.table.rows_valid(&z.rows)) {
            self.apply_rows(z.rows.iter().map(String::as_str));
        } else {
            self.apply_zone(zone);
        }
    }

    /// Terrain from glyph rows (an unknown glyph or a missing tile is floor).
    fn apply_rows<'a>(&mut self, rows: impl IntoIterator<Item = &'a str>) {
        self.terrain = vec![0; (GRID_W * GRID_H) as usize];
        for (y, row) in rows.into_iter().enumerate().take(GRID_H as usize) {
            for (x, ch) in row.chars().enumerate().take(GRID_W as usize) {
                self.terrain[y * GRID_W as usize + x] = self.table.index_of_glyph(ch).unwrap_or(0);
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

    /// The terrain type id of tile `c` (the table's; "floor" off the grid).
    pub fn terrain_id(&self, c: V2) -> &str {
        let t = if self.is_valid(c) { self.terrain[(c.y * GRID_W + c.x) as usize] } else { 0 };
        self.table.ids.get(t).map_or("floor", String::as_str)
    }

    /// LAYER-DOWN-2: the board glyph of tile `c` (the table's; '.' when its type has none or off the grid).
    pub fn glyph(&self, c: V2) -> char {
        let t = if self.is_valid(c) { self.terrain[(c.y * GRID_W + c.x) as usize] } else { 0 };
        self.table.glyphs.get(t).copied().flatten().unwrap_or('.')
    }

    pub fn is_walkable(&self, c: V2) -> bool {
        self.is_valid(c) && self.row(c).walkable
    }

    /// P1.2-S1 (FB-51): a forced move stops before the tile (off the grid, or the row's `stops_forced_move`).
    pub fn stops_forced_move(&self, c: V2) -> bool {
        !self.is_valid(c) || self.row(c).stops_forced_move
    }

    /// The terrain's move cost, plus a ground hazard's `move_cost` (HOOKS1 GH5).
    pub fn move_cost(&self, c: V2) -> i32 {
        self.row(c).move_cost + self.ground.get(&c).map_or(0, |p| p.def.move_cost)
    }

    /// The terrain's cover, or a ground hazard's when larger (HOOKS1 GH5: smoke gives cover).
    pub fn cover_guard(&self, c: V2) -> f64 {
        if !self.is_valid(c) {
            return 0.0;
        }
        let terrain = self.row(c).cover_guard;
        self.ground.get(&c).map_or(terrain, |p| terrain.max(p.def.cover_guard))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_zone_id_has_rows() {
        for id in ZONE_IDS {
            assert!(zone_rows(id).is_some(), "{id} has no rows");
        }
        assert!(zone_rows("zz_unknown").is_none());
    }
}
