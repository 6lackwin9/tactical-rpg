class_name CombatGrid
extends RefCounted

const GRID_SIZE: int = 9
const CENTER: Vector2i = Vector2i(4, 4)

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
	return coord.x >= 0 and coord.x < GRID_SIZE and coord.y >= 0 and coord.y < GRID_SIZE

func is_exit_tile(coord: Vector2i) -> bool:
	if arena_type == ArenaType.OPEN:
		return coord.x == 0 or coord.x == GRID_SIZE - 1 or coord.y == 0 or coord.y == GRID_SIZE - 1
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

## Deploys combatants in a radial pattern centered around CENTER (4, 4).
## Ensures every combatant is positioned so that its distance to CENTER <= combatant.attack_range.
func deploy_combatants(allies: Array[Combatant], enemies: Array[Combatant]) -> void:
	# Candidate candidate offsets by quadrant and distance:
	# Allies deploy South/West (y >= 4 or x <= 4)
	# Enemies deploy North/East (y <= 4 or x >= 4)
	
	# Ally slots by distance from CENTER:
	var ally_slots_d1: Array[Vector2i] = [
		CENTER + Vector2i(0, 1),   # (4, 5) - South
		CENTER + Vector2i(-1, 0),  # (3, 4) - West
		CENTER + Vector2i(-1, 1),  # (3, 5) - South-West (dist 2)
	]
	var ally_slots_d2: Array[Vector2i] = [
		CENTER + Vector2i(0, 2),   # (4, 6) - South
		CENTER + Vector2i(-2, 0),  # (2, 4) - West
		CENTER + Vector2i(-1, 1),  # (3, 5) - South-West
		CENTER + Vector2i(-2, 1),  # (2, 5) - West-South (dist 3)
	]

	# Enemy slots by distance from CENTER:
	var enemy_slots_d1: Array[Vector2i] = [
		CENTER + Vector2i(0, -1),  # (4, 3) - North
		CENTER + Vector2i(1, 0),   # (5, 4) - East
		CENTER + Vector2i(1, -1),  # (5, 3) - North-East (dist 2)
	]
	var enemy_slots_d2: Array[Vector2i] = [
		CENTER + Vector2i(0, -2),  # (4, 2) - North
		CENTER + Vector2i(2, 0),   # (6, 4) - East
		CENTER + Vector2i(1, -1),  # (5, 3) - North-East
		CENTER + Vector2i(1, -2),  # (5, 2) - North-East (dist 3)
	]

	# Deploy allies
	for ally: Combatant in allies:
		var placed: bool = false
		var preferred_slots: Array[Vector2i] = ally_slots_d2 if ally.attack_range > 1 else ally_slots_d1
		for slot: Vector2i in preferred_slots:
			if not is_occupied(slot) and is_walkable(slot) and get_distance(slot, CENTER) <= ally.attack_range:
				place_combatant(ally, slot)
				placed = true
				break
		if not placed:
			# Fallback to any valid unoccupied slot within range
			for dx: int in range(-ally.attack_range, ally.attack_range + 1):
				for dy: int in range(-ally.attack_range, ally.attack_range + 1):
					var cand: Vector2i = CENTER + Vector2i(dx, dy)
					if is_walkable(cand) and not is_occupied(cand) and get_distance(cand, CENTER) <= ally.attack_range:
						place_combatant(ally, cand)
						placed = true
						break
				if placed:
					break

	# Deploy enemies
	for enemy: Combatant in enemies:
		var placed: bool = false
		var preferred_slots: Array[Vector2i] = enemy_slots_d2 if enemy.attack_range > 1 else enemy_slots_d1
		for slot: Vector2i in preferred_slots:
			if not is_occupied(slot) and is_walkable(slot) and get_distance(slot, CENTER) <= enemy.attack_range:
				place_combatant(enemy, slot)
				placed = true
				break
		if not placed:
			# Fallback to any valid unoccupied slot within range
			for dx: int in range(-enemy.attack_range, enemy.attack_range + 1):
				for dy: int in range(-enemy.attack_range, enemy.attack_range + 1):
					var cand: Vector2i = CENTER + Vector2i(dx, dy)
					if is_walkable(cand) and not is_occupied(cand) and get_distance(cand, CENTER) <= enemy.attack_range:
						place_combatant(enemy, cand)
						placed = true
						break
				if placed:
					break
