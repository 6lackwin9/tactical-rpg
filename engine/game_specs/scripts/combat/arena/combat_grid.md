---
class: CombatGrid
extends: RefCounted
---

# Properties
* `GRID_W`: int = 12 (const)
* `GRID_H`: int = 9 (const)
* `PARTY_FRONT_X`: int = 5 (const)
* `PARTY_BACK_X`: int = 4 (const)
* `ENEMY_FRONT_X`: int = 7 (const)
* `ENEMY_BACK_X`: int = 8 (const)
* `DEPLOY_ROWS`: Array[int] = [4, 3, 5, 2, 6, 1, 7] (const)
* `DEPLOY_ANCHOR_Y`: int = 4   # ambush / centre arrivals take the free tile nearest (group front, this row) (const)
* `DEPLOYMENTS`: Dictionary = { (const)
* `arena_type`: ArenaType = ArenaType.OPEN
* `designated_exits`: Array[Vector2i] = []
* `combatant_positions`: Dictionary = {}  # Combatant -> Vector2i
* `withdrawn_combatants`: Array[Combatant] = []
* `terrain`: Dictionary = {}  # Vector2i -> Terrain.Type (missing = FLOOR)
* `raw`: Variant = terrain.get(coord, Terrain.Type.FLOOR)
* `layout`: Dictionary = DictUtil.dict_of(DEPLOYMENTS.get(deployment, DEPLOYMENTS["standard"]))
* `out`: Array[int] = []
* `out`: Array[int] = []
* `probe`: Variant
* `dir`: int = signi(back_x - front_x)
* `x`: int = back_x + dir
* `lo`: int = mini(front_x, back_x) - 1
* `hi`: int = maxi(front_x, back_x) + 1

# Signals
* `combatant_placed(combatant: Combatant, pos: Vector2i)`
* `combatant_moved(combatant: Combatant, from_pos: Vector2i, to_pos: Vector2i)`
* `combatant_withdrawn(combatant: Combatant, exit_pos: Vector2i)`

# Methods

### `_init(p_arena_type: ArenaType = ArenaType.OPEN, p_exits: Array[Vector2i] = []) -> void`
```logic
arena_type = p_arena_type
designated_exits = p_exits
```

### `is_valid_coord(coord: Vector2i) -> bool`
```logic
return coord.x >= 0 and coord.x < GRID_W and coord.y >= 0 and coord.y < GRID_H
```

### `static centre_distance(coord: Vector2i) -> int`
```logic
return absi(2 * coord.x - (GRID_W - 1)) + absi(2 * coord.y - (GRID_H - 1))
```

### `static is_party_half(coord: Vector2i) -> bool`
```logic
return coord.x * 2 < GRID_W
```

### `is_exit_tile(coord: Vector2i) -> bool`
```logic
if arena_type == ArenaType.OPEN:
	return coord.x == 0 or coord.x == GRID_W - 1 or coord.y == 0 or coord.y == GRID_H - 1
return designated_exits.has(coord)
```

### `set_terrain(coord: Vector2i, terrain_type: Terrain.Type) -> void`
```logic
if is_valid_coord(coord):
	terrain[coord] = terrain_type
```

### `is_walkable(coord: Vector2i) -> bool`
```logic
return is_valid_coord(coord) and Terrain.is_walkable(get_terrain(coord))
```

### `move_cost(coord: Vector2i) -> int`
```logic
return Terrain.move_cost(get_terrain(coord))
```

### `cover_guard(coord: Vector2i) -> float`
```logic
return Terrain.cover_guard(get_terrain(coord)) if is_valid_coord(coord) else 0.0
```

### `has_hazard(coord: Vector2i) -> bool`
```logic
return is_valid_coord(coord) and Terrain.has_hazard(get_terrain(coord))
```

### `hazard_at(coord: Vector2i) -> Dictionary`
```logic
return Terrain.hazard(get_terrain(coord)) if is_valid_coord(coord) else {}
```

### `apply_zone(zone_id: String) -> void`
```logic
terrain = CombatZones.terrain_map(zone_id)
```

### `is_occupied(coord: Vector2i) -> bool`
```logic
for c: Combatant in combatant_positions.keys():
	if c.is_active() and combatant_positions[c] == coord:
		return true
return false
```

### `get_combatant_at(coord: Vector2i) -> Combatant`
```logic
for c: Combatant in combatant_positions.keys():
	if c.is_active() and combatant_positions[c] == coord:
		return c
return null
```

### `get_position(combatant: Combatant) -> Vector2i`
```logic
if combatant_positions.has(combatant):
	@warning_ignore("unsafe_cast")
	return combatant_positions[combatant] as Vector2i
return Vector2i(-1, -1)
```

### `has_combatant(combatant: Combatant) -> bool`
```logic
return combatant_positions.has(combatant) and (combatant as Combatant).is_active()
```

### `get_distance(a: Vector2i, b: Vector2i) -> int`
```logic
return absi(a.x - b.x) + absi(a.y - b.y)
```

### `place_combatant(combatant: Combatant, coord: Vector2i) -> bool`
```logic
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
```

### `remove_combatant(combatant: Combatant) -> void`
```logic
combatant_positions.erase(combatant)
```

### `can_target(source: Combatant, target: Combatant, max_range: int) -> bool`
```logic
if not has_combatant(source) or not has_combatant(target):
	return false
var dist: int = get_distance(get_position(source), get_position(target))
return dist <= max_range
```

### `move_combatant(combatant: Combatant, offset: Vector2i) -> bool`
```logic
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
```

### `jump_combatant(combatant: Combatant, coord: Vector2i) -> bool`
```logic
if not combatant.is_active() or not combatant_positions.has(combatant):
	return false
if not is_valid_coord(coord) or not is_walkable(coord) or is_exit_tile(coord) or is_occupied(coord):
	return false
var from_pos: Vector2i = get_position(combatant)
combatant_positions[combatant] = coord
combatant_moved.emit(combatant, from_pos, coord)
print("[%s] jumped from %s to %s." % [combatant.name, str(from_pos), str(coord)])
return true
```

### `deploy_combatants(allies: Array[Combatant], enemies: Array[Combatant], deployment: String = "standard") -> void`
```logic
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
```

### `static group_of(index: int, deployment: String) -> int`
```logic
return 0 if layout_columns(deployment, "b").is_empty() else index % 2
```

### `_deploy_side(units: Array[Combatant], front_x: int, back_x: int) -> void`
```logic
var spill: Array[int] = spill_columns(front_x, back_x)
for c: Combatant in units:
	var columns: Array[int] = [front_x, back_x]
	if c.attack_range > 1:
		columns = [back_x, front_x]
	columns.append_array(spill)
	for col: int in columns:
		if _place_in_column(c, col):
			break
```

### `place_near_front(c: Combatant, front_x: int) -> bool`
```logic
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
```

### `_place_in_column(c: Combatant, x: int) -> bool`
```logic
for y: int in DEPLOY_ROWS:
	var cell := Vector2i(x, y)
	if is_walkable(cell) and not is_exit_tile(cell) and not is_occupied(cell):
		return place_combatant(c, cell)
return false
```

