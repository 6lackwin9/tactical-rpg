class_name WalkCommand
extends MoveCommand

# ==============================================================================
# Tactical RPG - Walk Command (a planned multi-tile move along any path)
# One block. The path is a list of unit steps (each cardinal), so a single Move
# can turn corners. It stops early when a step is blocked. AP is paid when it
# starts (1 AP per planned tile).
# ==============================================================================

var path: Array[Vector2i] = []

func _init(p_source: Combatant, p_grid: CombatGrid, p_path: Array[Vector2i], p_block_cost: int = 1) -> void:
	super(p_source, p_grid, p_path[0] if not p_path.is_empty() else Vector2i.ZERO, p_path.size(), p_block_cost)
	path = p_path
	command_name = "Move"

func execute() -> void:
	if not can_execute():
		execution_log = "[Move Failed] %s could not execute movement (inactive or off grid)." % source.name
		return
	var start_pos: Vector2i = grid.get_position(source)
	var moved: int = 0
	for dir: Vector2i in path:
		if not source.is_active() or not grid.move_combatant(source, dir):
			break
		moved += 1
	if source.is_withdrawn:
		execution_log = "[Withdrawal] %s moved from %s and safely WITHDREW through Exit!" % [source.name, str(start_pos)]
	elif moved == 0:
		execution_log = "[Move Blocked] %s could not move from %s (path blocked)." % [source.name, str(start_pos)]
	else:
		execution_log = "[Move] %s moved %d tile%s from %s to %s" % [source.name, moved, "" if moved == 1 else "s", str(start_pos), str(grid.get_position(source))]