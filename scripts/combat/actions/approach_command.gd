class_name ApproachCommand
extends MoveCommand

# ==============================================================================
# Tactical RPG - Approach Command (the live "move toward target" part of Engage)
# Unlike a plain Move (fixed direction), this re-paths from the CURRENT positions
# when it executes, so it chases a target that moved earlier in the round. It
# takes up to `steps` tiles, stops as soon as the target is within weapon range,
# and goes around obstacles and other units.
# ==============================================================================

var target: Combatant = null

func _init(p_source: Combatant, p_grid: CombatGrid, p_target: Combatant, p_steps: int = 1, p_block_cost: int = 1) -> void:
	super(p_source, p_grid, Vector2i.ZERO, p_steps, p_block_cost)
	target = p_target
	command_name = "Approach"

func execute() -> void:
	if not can_execute():
		execution_log = "[Approach Failed] %s could not move (inactive or off grid)." % source.name
		return
	if target == null or not target.is_active() or not grid.has_combatant(target):
		execution_log = "[Approach] %s has no target left to close in on." % source.name
		return

	var start_pos: Vector2i = grid.get_position(source)
	var moved: int = 0
	for s: int in range(steps):
		if not source.is_active() or grid.get_distance(grid.get_position(source), grid.get_position(target)) <= source.attack_range:
			break
		var dir: Vector2i = _next_step()
		if dir == Vector2i.ZERO or not grid.move_combatant(source, dir):
			break
		moved += 1

	var end_pos: Vector2i = grid.get_position(source)
	if moved == 0:
		var in_range: bool = grid.get_distance(start_pos, grid.get_position(target)) <= source.attack_range
		execution_log = "[Approach] %s holds at %s (%s)." % [source.name, str(start_pos), "already in range of %s" % target.name if in_range else "no route to %s" % target.name]
	else:
		execution_log = "[Approach] %s closed in on %s: %s to %s (Mobile Stance)" % [source.name, target.name, str(start_pos), str(end_pos)]

# First step of the shortest route to a tile within range of the target's current position.
func _next_step() -> Vector2i:
	var target_pos: Vector2i = grid.get_position(target)
	var reach: int = source.attack_range
	var blocked: Dictionary = {}
	for other: Variant in grid.combatant_positions:
		if other != source and other is Combatant:
			blocked[grid.combatant_positions[other]] = true
	var goals: Dictionary = {}
	for dx: int in range(-reach, reach + 1):
		for dy: int in range(-reach, reach + 1):
			var cell: Vector2i = target_pos + Vector2i(dx, dy)
			if absi(dx) + absi(dy) <= reach and grid.is_valid_coord(cell) and grid.is_walkable(cell) and not grid.is_exit_tile(cell) and not blocked.has(cell):
				goals[cell] = true
	var path: Array[Vector2i] = CombatPlan.find_path(grid, grid.get_position(source), goals, blocked, false)
	return path[0] if not path.is_empty() else Vector2i.ZERO
