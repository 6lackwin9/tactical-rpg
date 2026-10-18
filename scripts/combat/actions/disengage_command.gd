class_name DisengageCommand
extends BattleCommand

# ==============================================================================
# Tactical RPG - Disengage Command (a jump away from an enemy; M2c-2a2)
# The planner fixes the landing tile (`dest`); the jump lands at the end of the
# command's block. It leaps over units but not walls: the landing tile must still be
# a free, walkable tile at most `max_tiles` away. A rooted unit can't jump. The unit
# keeps its stance. AP = the skill's level (charged by the resolver at start).
# The jump ignores move cost; the landing tile's enter hazard applies (HazardRules, S12T-L).
# ==============================================================================

var grid: CombatGrid = null
var dest: Vector2i = Vector2i(-1, -1)
var max_tiles: int = 0

# Filled in by execute() (read by the log).
var landed: bool = false
var moved_tiles: int = 0

func _init(p_source: Combatant, p_grid: CombatGrid, p_dest: Vector2i, p_max_tiles: int, p_block_cost: int = 1) -> void:
	super(p_source, 0, p_block_cost)
	grid = p_grid
	dest = p_dest
	max_tiles = p_max_tiles
	command_name = "Disengage"

func can_execute() -> bool:
	return super.can_execute() and source.is_active() and grid != null and grid.has_combatant(source)

func execute() -> void:
	landed = false
	moved_tiles = 0
	if not can_execute():
		execution_log = "[Disengage Failed] %s cannot jump (inactive or off grid)." % source.name
		return
	if source.has_status(StatusEffects.ROOT):
		execution_log = "[Disengage Failed] %s is rooted and can't jump." % source.name
		return
	var from: Vector2i = grid.get_position(source)
	var dist: int = grid.get_distance(from, dest)
	if dist > max_tiles:
		execution_log = "[Disengage Failed] %s can't reach %s (%d tiles, the jump covers %d)." % [source.name, str(dest), dist, max_tiles]
		return
	if not grid.jump_combatant(source, dest):
		execution_log = "[Disengage Failed] %s can't land on %s." % [source.name, str(dest)]
		return
	landed = true
	moved_tiles = dist
	execution_log = "[Disengage] %s jumped %d tile%s from %s to %s" % [source.name, dist, "" if dist == 1 else "s", str(from), str(dest)]
	var landing: Array[Vector2i] = [dest]
	for note: String in HazardRules.on_enter(grid, source, landing, rng):
		execution_log += " " + note
