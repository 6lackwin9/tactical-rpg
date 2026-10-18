class_name CombatGrid
extends RefCounted

const GRID_W: int = 12
const GRID_H: int = 9
# Standard deployment columns (user 2026-10-17: party on the left, enemies on the right). Front ranks
# sit 2 tiles apart and back ranks 3 tiles from the opposing front, the same gaps as the old 9x9 ring.
const PARTY_FRONT_X: int = 5
const PARTY_BACK_X: int = 4
const ENEMY_FRONT_X: int = 7
const ENEMY_BACK_X: int = 8
const DEPLOY_ROWS: Array[int] = [4, 3, 5, 2, 6, 1, 7]
const DEPLOY_ANCHOR_Y: int = 4   # ambush / centre arrivals take the free tile nearest (group front, this row)
# Deployment table (OBJ1 1.2): [front x, back x] of the party and of enemy groups A and B ([] = no group B: every
# enemy is in A). Ambush and centre split the enemy roster by order: even index to A, odd to B.
const DEPLOYMENTS: Dictionary = {
	"standard": {"party": [PARTY_FRONT_X, PARTY_BACK_X], "a": [ENEMY_FRONT_X, ENEMY_BACK_X], "b": []},
	"ambush": {"party": [5, 4], "a": [7, 8], "b": [2, 1]},
	"centre": {"party": [5, 6], "a": [8, 9], "b": [3, 2]},
}

enum ArenaType {
	OPEN,
	CLOSED
}

var arena_type: ArenaType = ArenaType.OPEN
var designated_exits: Array[Vector2i] = []
var combatant_positions: Dictionary = {}  # Combatant -> Vector2i
var withdrawn_combatants: Array[Combatant] = []

signal combatant_placed(combatant: Combatant, pos: Vector2i)
signal combatant_moved(combatant: Combatant, from_pos: Vector2i, to_pos: Vector2i)
signal combatant_withdrawn(combatant: Combatant, exit_pos: Vector2i)

func _init(p_arena_type: ArenaType = ArenaType.OPEN, p_exits: Array[Vector2i] = []) -> void:
	arena_type = p_arena_type
	designated_exits = p_exits

func is_valid_coord(coord: Vector2i) -> bool:
	return coord.x >= 0 and coord.x < GRID_W and coord.y >= 0 and coord.y < GRID_H

# Distance to the arena middle in half tiles (the middle lies between columns 5 and 6 on row 4, so
# no single tile is the centre). Only compared, never shown: tie-breaks that prefer central tiles.
static func centre_distance(coord: Vector2i) -> int:
	return absi(2 * coord.x - (GRID_W - 1)) + absi(2 * coord.y - (GRID_H - 1))

# True for the party's (left) half of the arena, false for the enemies' (right) half.
static func is_party_half(coord: Vector2i) -> bool:
	return coord.x * 2 < GRID_W

func is_exit_tile(coord: Vector2i) -> bool:
	if arena_type == ArenaType.OPEN:
		return coord.x == 0 or coord.x == GRID_W - 1 or coord.y == 0 or coord.y == GRID_H - 1
	return designated_exits.has(coord)

# ------------------------------------------------------------------------------
# Terrain (default: plain arena floor everywhere)
# ------------------------------------------------------------------------------

var terrain: Dictionary = {}  # Vector2i -> Terrain.Type (missing = FLOOR)

func set_terrain(coord: Vector2i, terrain_type: Terrain.Type) -> void:
	if is_valid_coord(coord):
		terrain[coord] = terrain_type

func get_terrain(coord: Vector2i) -> Terrain.Type:
	var raw: Variant = terrain.get(coord, Terrain.Type.FLOOR)
	return raw if raw is Terrain.Type else Terrain.Type.FLOOR

func is_walkable(coord: Vector2i) -> bool:
	return is_valid_coord(coord) and Terrain.is_walkable(get_terrain(coord))

# Cost of entering `coord` (Terrain.move_cost of its type; S12T-L).
func move_cost(coord: Vector2i) -> int:
	return Terrain.move_cost(get_terrain(coord))

# Guard share a ranged hit loses on `coord` (or behind it when it blocks movement).
func cover_guard(coord: Vector2i) -> float:
	return Terrain.cover_guard(get_terrain(coord)) if is_valid_coord(coord) else 0.0

func has_hazard(coord: Vector2i) -> bool:
	return is_valid_coord(coord) and Terrain.has_hazard(get_terrain(coord))

# The hazard of the tile at `coord` (Terrain.hazard: {} when none).
func hazard_at(coord: Vector2i) -> Dictionary:
	return Terrain.hazard(get_terrain(coord)) if is_valid_coord(coord) else {}

func apply_zone(zone_id: String) -> void:
	terrain = CombatZones.terrain_map(zone_id)

func is_occupied(coord: Vector2i) -> bool:
	for c: Combatant in combatant_positions.keys():
		if c.is_active() and combatant_positions[c] == coord:
			return true
	return false

func get_combatant_at(coord: Vector2i) -> Combatant:
	for c: Combatant in combatant_positions.keys():
		if c.is_active() and combatant_positions[c] == coord:
			return c
	return null

func get_position(combatant: Combatant) -> Vector2i:
	if combatant_positions.has(combatant):
		@warning_ignore("unsafe_cast")
		return combatant_positions[combatant] as Vector2i
	return Vector2i(-1, -1)

func has_combatant(combatant: Combatant) -> bool:
	return combatant_positions.has(combatant) and (combatant as Combatant).is_active()

func get_distance(a: Vector2i, b: Vector2i) -> int:
	return absi(a.x - b.x) + absi(a.y - b.y)

func place_combatant(combatant: Combatant, coord: Vector2i) -> bool:
	if not is_valid_coord(coord):
		push_error("Cannot place [%s] at invalid coordinate %s" % [combatant.name, str(coord)])
		return false
	if not is_walkable(coord):
		push_error("Cannot place [%s] on blocked terrain %s (%s)" % [combatant.name, str(coord), Terrain.display_name(get_terrain(coord))])
		return false
	if is_occupied(coord):
		push_error("Tile %s is already occupied" % str(coord))
		return false

	combatant_positions[combatant] = coord
	combatant_placed.emit(combatant, coord)
	return true

func remove_combatant(combatant: Combatant) -> void:
	combatant_positions.erase(combatant)

# MORALE1 rule 9: a unit leaves the field at its own tile with exit bookkeeping (no exit or route needed).
func withdraw_in_place(combatant: Combatant) -> bool:
	if not combatant.is_active() or not combatant_positions.has(combatant):
		return false
	var at: Vector2i = get_position(combatant)
	combatant_positions.erase(combatant)
	if not withdrawn_combatants.has(combatant):
		withdrawn_combatants.append(combatant)
	combatant.withdraw()
	combatant_withdrawn.emit(combatant, at)
	return true

func can_target(source: Combatant, target: Combatant, max_range: int) -> bool:
	if not has_combatant(source) or not has_combatant(target):
		return false
	var dist: int = get_distance(get_position(source), get_position(target))
	return dist <= max_range

func move_combatant(combatant: Combatant, offset: Vector2i) -> bool:
	if not combatant.is_active() or not combatant_positions.has(combatant):
		return false

	var current_pos: Vector2i = get_position(combatant)
	var target_pos: Vector2i = current_pos + offset

	# Check if step lands on or moves past an exit tile -> Partial Withdrawal!
	if is_exit_tile(target_pos) or (arena_type == ArenaType.OPEN and not is_valid_coord(target_pos)):
		var exit_coord: Vector2i = target_pos if is_valid_coord(target_pos) else current_pos
		combatant_positions.erase(combatant)
		if not withdrawn_combatants.has(combatant):
			withdrawn_combatants.append(combatant)
		combatant.withdraw()
		combatant_withdrawn.emit(combatant, exit_coord)
		print("[%s] reached exit at %s and safely withdrew from combat." % [combatant.name, str(exit_coord)])
		return true

	if not is_valid_coord(target_pos):
		print("[%s] move to %s blocked by arena boundary." % [combatant.name, str(target_pos)])
		return false

	if not is_walkable(target_pos):
		print("[%s] move to %s blocked by %s." % [combatant.name, str(target_pos), Terrain.display_name(get_terrain(target_pos))])
		return false

	if is_occupied(target_pos):
		var blocker: Combatant = get_combatant_at(target_pos)
		var blocker_name: String = blocker.name if blocker != null else "Obstacle"
		print("[%s] move to %s blocked by [%s]." % [combatant.name, str(target_pos), blocker_name])
		return false

	combatant_positions[combatant] = target_pos
	combatant_moved.emit(combatant, current_pos, target_pos)
	print("[%s] moved from %s to %s." % [combatant.name, str(current_pos), str(target_pos)])
	return true

# Jump: puts the combatant on `coord` if it is a valid, walkable, free, non-exit tile (intermediate
# tiles don't matter). Returns false and changes nothing otherwise.
func jump_combatant(combatant: Combatant, coord: Vector2i) -> bool:
	if not combatant.is_active() or not combatant_positions.has(combatant):
		return false
	if not is_valid_coord(coord) or not is_walkable(coord) or is_exit_tile(coord) or is_occupied(coord):
		return false
	var from_pos: Vector2i = get_position(combatant)
	combatant_positions[combatant] = coord
	combatant_moved.emit(combatant, from_pos, coord)
	print("[%s] jumped from %s to %s." % [combatant.name, str(from_pos), str(coord)])
	return true

## Deployment (DEPLOYMENTS; standard: the party lines up on the left, enemies on the right). Melee units
## (attack_range 1) take the front column, ranged units the back column, rows filled from
## the middle out (DEPLOY_ROWS); a full or blocked column spills into the other one, then
## further away from the middle. Units that still don't fit stay off the grid (CatalogEncounter places them).
## `allies` lists the heroes first, then the guests (same party rule).
func deploy_combatants(allies: Array[Combatant], enemies: Array[Combatant], deployment: String = "standard") -> void:
	var party: Array[int] = layout_columns(deployment, "party")
	_deploy_side(allies, party[0], party[1])
	var groups: Array[Array] = [[], []]
	for i: int in range(enemies.size()):
		groups[group_of(i, deployment)].append(enemies[i])
	for g: int in range(2):
		var cols: Array[int] = layout_columns(deployment, "a" if g == 0 else "b")
		var units: Array[Combatant] = []
		units.assign(groups[g])
		if cols.size() == 2 and not units.is_empty():
			_deploy_side(units, cols[0], cols[1])

# [front x, back x] of `side` ("party", "a" or "b") in `deployment` (unknown ids read as standard; [] = none).
static func layout_columns(deployment: String, side: String) -> Array[int]:
	var layout: Dictionary = DictUtil.dict_of(DEPLOYMENTS.get(deployment, DEPLOYMENTS["standard"]))
	var out: Array[int] = []
	for v: Variant in DictUtil.array_of(layout.get(side, [])):
		out.append(DictUtil.to_int(v))
	return out

# Enemy group of the `index`-th enemy of the roster: 0 = A, 1 = B (standard: always A).
static func group_of(index: int, deployment: String) -> int:
	return 0 if layout_columns(deployment, "b").is_empty() else index % 2

# Columns after front and back when both are full: away from the middle on the same half (GRID12 rule); a side
# split across the middle (centre party) spills outwards from both columns in turn.
static func spill_columns(front_x: int, back_x: int) -> Array[int]:
	var out: Array[int] = []
	var probe := Vector2i(front_x, 0)
	if is_party_half(probe) == is_party_half(Vector2i(back_x, 0)):
		var dir: int = signi(back_x - front_x)
		var x: int = back_x + dir
		while x >= 0 and x < GRID_W and is_party_half(Vector2i(x, 0)) == is_party_half(probe):
			out.append(x)
			x += dir
		return out
	var lo: int = mini(front_x, back_x) - 1
	var hi: int = maxi(front_x, back_x) + 1
	while lo >= 0 or hi < GRID_W:
		if lo >= 0:
			out.append(lo)
		if hi < GRID_W:
			out.append(hi)
		lo -= 1
		hi += 1
	return out

func _deploy_side(units: Array[Combatant], front_x: int, back_x: int) -> void:
	var spill: Array[int] = spill_columns(front_x, back_x)
	for c: Combatant in units:
		var columns: Array[int] = [front_x, back_x]
		if c.attack_range > 1:
			columns = [back_x, front_x]
		columns.append_array(spill)
		for col: int in columns:
			if _place_in_column(c, col):
				break

# Ambush / centre arrivals (leftovers, waves, phase waves): the free walkable, non-exit tile nearest
# (front_x, DEPLOY_ANCHOR_Y) by grid distance; ties by DEPLOY_ROWS order, then the column away from the middle.
# False (nothing placed) when no tile is free.
func place_near_front(c: Combatant, front_x: int) -> bool:
	var anchor := Vector2i(front_x, DEPLOY_ANCHOR_Y)
	var right: bool = not is_party_half(anchor)
	var best := Vector2i(-1, -1)
	var best_score: int = 1 << 30
	for y: int in range(GRID_H):
		var rank: int = DEPLOY_ROWS.find(y)
		if rank < 0:
			rank = DEPLOY_ROWS.size()
		for x: int in range(GRID_W):
			var cell := Vector2i(x, y)
			if not is_walkable(cell) or is_exit_tile(cell) or is_occupied(cell):
				continue
			var outward: int = (GRID_W - 1 - x) if right else x
			var score: int = get_distance(cell, anchor) * 10000 + rank * 100 + outward
			if score < best_score:
				best_score = score
				best = cell
	return best != Vector2i(-1, -1) and place_combatant(c, best)

func _place_in_column(c: Combatant, x: int) -> bool:
	for y: int in DEPLOY_ROWS:
		var cell := Vector2i(x, y)
		if is_walkable(cell) and not is_exit_tile(cell) and not is_occupied(cell):
			return place_combatant(c, cell)
	return false
