class_name WalkCommand
extends BattleCommand

# ==============================================================================
# Tactical RPG - Walk Command (a planned move along any path, one block)
# Planned moves longer than a unit's tiles-per-block are split into one Walk
# per block, each landing at the end of its own block. A walk follows its
# planned steps; if the unit isn't where the plan expected (an earlier half was
# cut short), it re-paths toward the final destination around whoever stands
# there now. A blocked step stops it short: the unit stays on the last free
# tile, unused tiles are refunded as AP, and the log says what blocked it.
# ==============================================================================

const NONE: Vector2i = Vector2i(-1, -1)

var grid: CombatGrid = null
var path: Array[Vector2i] = []        # unit steps, in order
var expected_start: Vector2i = NONE   # where the plan has the unit when this walk starts
var goal: Vector2i = NONE             # the move's final destination (for re-pathing)
var approach_target: Combatant = null  # set when this walk is an Engage's approach (display only)

# Outcome of the last execute() (read by the log): tiles walked, and why it stopped early.
var moved_tiles: int = 0
var cut_short: bool = false
var blocked_by: String = ""

func _init(p_source: Combatant, p_grid: CombatGrid, p_path: Array[Vector2i], p_block_cost: int = 1) -> void:
	super(p_source, 0, p_block_cost)
	grid = p_grid
	path = p_path
	command_name = "Move"

# First step of the walk (display: the timeline's arrow).
func direction() -> Vector2i:
	return path[0] if not path.is_empty() else Vector2i.ZERO

func can_execute() -> bool:
	return super.can_execute() and source.is_active() and grid != null and grid.has_combatant(source)

func execute() -> void:
	moved_tiles = 0
	cut_short = false
	blocked_by = ""
	if not can_execute():
		execution_log = "[Move Failed] %s could not execute movement (inactive or off grid)." % source.name
		return
	var start_pos: Vector2i = grid.get_position(source)
	var walk: Array[Vector2i] = path
	if expected_start != NONE and goal != NONE and start_pos != expected_start:
		walk = _repath(start_pos)
	for dir: Vector2i in walk:
		var next_cell: Vector2i = grid.get_position(source) + dir
		if not source.is_active() or not grid.move_combatant(source, dir):
			cut_short = true
			blocked_by = _describe_blocker(next_cell)
			break
		moved_tiles += 1
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

# Up to this walk's tile count toward `goal`, around current occupants.
func _repath(from: Vector2i) -> Array[Vector2i]:
	var blocked: Dictionary = {}
	for other: Variant in grid.combatant_positions:
		if other != source and other is Combatant:
			blocked[grid.combatant_positions[other]] = true
	var route: Array[Vector2i] = GridPath.find_path(grid, from, {goal: true}, blocked, grid.is_exit_tile(goal))
	return route.slice(0, path.size())

func _describe_blocker(cell: Vector2i) -> String:
	var occupant: Combatant = grid.get_combatant_at(cell)
	if occupant != null:
		return occupant.name
	if not grid.is_valid_coord(cell):
		return "the arena edge"
	if not grid.is_walkable(cell):
		return Terrain.display_name(grid.get_terrain(cell))
	return "an obstacle"
