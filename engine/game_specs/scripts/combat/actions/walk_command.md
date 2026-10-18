---
class: WalkCommand
extends: BattleCommand
description: "Tactical RPG - Walk Command (a planned move along any path, one block) Planned moves longer than a unit's tiles-per-block are split into one Walk per block, each landing at the end of its own block. A walk follows its planned steps; if the unit isn't where the plan expected (an earlier half was cut short), it re-paths toward the final destination around whoever stands there now. A blocked step stops it short: the unit stays on the last free tile, unused tiles are refunded as AP, and the log says what blocked it. Hazard tiles entered roll their status once per status (HazardRules, S12T-L)."
---

# Properties
* `NONE`: Vector2i = Vector2i(-1, -1) (const)
* `grid`: CombatGrid = null
* `path`: Array[Vector2i] = []        # unit steps, in order
* `expected_start`: Vector2i = NONE   # where the plan has the unit when this walk starts
* `goal`: Vector2i = NONE             # the move's final destination (for re-pathing)
* `approach_target`: Combatant = null  # set when this walk is an Engage's approach (display only)
* `moved_tiles`: int = 0
* `cut_short`: bool = false
* `blocked_by`: String = ""
* `hazard_notes`: Array[String] = []   # "[Hazard] ..." lines of the statuses the walk's tiles inflicted
* `blocked`: Dictionary = {}
* `route`: Array[Vector2i] = GridPath.find_path(grid, from, {goal: true}, blocked, grid.is_exit_tile(goal))
* `budget`: int = maxi(source.move_per_block(), GridPath.path_cost(grid, expected_start, path))   # this walk's planned cost
* `first`: Array[Vector2i] = GridPath.fit_blocks(grid, from, route, budget, 1)

# Methods

### `_init(p_source: Combatant, p_grid: CombatGrid, p_path: Array[Vector2i], p_block_cost: int = 1) -> void`
```logic
super(p_source, 0, p_block_cost)
grid = p_grid
path = p_path
command_name = "Move"
```

### `direction() -> Vector2i`
```logic
return path[0] if not path.is_empty() else Vector2i.ZERO
```

### `can_execute() -> bool`
```logic
return super.can_execute() and source.is_active() and grid != null and grid.has_combatant(source)
```

### `execute() -> void`
```logic
moved_tiles = 0
cut_short = false
blocked_by = ""
hazard_notes = []
if not can_execute():
	execution_log = "[Move Failed] %s could not execute movement (inactive or off grid)." % source.name
	return
var start_pos: Vector2i = grid.get_position(source)
var walk: Array[Vector2i] = path
if expected_start != NONE and goal != NONE and start_pos != expected_start:
	walk = _repath(start_pos)
var entered: Array[Vector2i] = []
for dir: Vector2i in walk:
	var next_cell: Vector2i = grid.get_position(source) + dir
	if not source.is_active() or not grid.move_combatant(source, dir):
		cut_short = true
		blocked_by = _describe_blocker(next_cell)
		break
	moved_tiles += 1
	if grid.has_combatant(source):
		entered.append(next_cell)
hazard_notes = HazardRules.on_enter(grid, source, entered, rng)
if source.is_withdrawn:
	execution_log = "[Withdrawal] %s moved from %s and safely WITHDREW through Exit!" % [source.name, str(start_pos)]
	return
# Tiles the unit couldn't walk cost nothing.
if moved_tiles < path.size():
	source.ap = mini(source.max_ap, source.ap + (path.size() - moved_tiles))
	cut_short = true
	if blocked_by.is_empty():
		blocked_by = "no route"
var end_pos: Vector2i = grid.get_position(source)
if moved_tiles == 0:
	execution_log = "[Move Cut Short] %s couldn't move from %s (blocked by %s)." % [source.name, str(start_pos), blocked_by]
elif cut_short:
	execution_log = "[Move Cut Short] %s stopped at %s after %d of %d tiles (blocked by %s)." % [source.name, str(end_pos), moved_tiles, path.size(), blocked_by]
else:
	execution_log = "[Move] %s moved %d tile%s from %s to %s" % [source.name, moved_tiles, "" if moved_tiles == 1 else "s", str(start_pos), str(end_pos)]
for note: String in hazard_notes:
	execution_log += " " + note
```

### `_describe_blocker(cell: Vector2i) -> String`
```logic
var occupant: Combatant = grid.get_combatant_at(cell)
if occupant != null:
	return occupant.name
if not grid.is_valid_coord(cell):
	return "the arena edge"
if not grid.is_walkable(cell):
	return Terrain.display_name(grid.get_terrain(cell))
return "an obstacle"
```

