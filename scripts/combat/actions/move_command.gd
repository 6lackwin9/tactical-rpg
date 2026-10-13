class_name MoveCommand
extends BattleCommand

var grid: CombatGrid
var direction: Vector2i
var steps: int

func _init(p_source: Combatant, p_grid: CombatGrid, p_direction: Vector2i, p_steps: int = 1, p_block_cost: int = 1, p_priority: int = 0) -> void:
	super(p_source, p_priority, p_block_cost)
	grid = p_grid
	direction = p_direction
	steps = p_steps
	action_type = CombatTypes.ActionType.MOVE
	resulting_stance = CombatTypes.StanceType.NEUTRAL  # moving never changes stance
	is_self_targeted = true
	command_name = "Move" if p_steps == 1 else "Sprint"

func can_execute() -> bool:
	if not super.can_execute():
		return false
	if not source.is_active():
		return false
	if grid == null or not grid.has_combatant(source):
		return false
	return true

func execute() -> void:
	if not can_execute():
		execution_log = "[Move Failed] %s could not execute movement (inactive or off grid)." % source.name
		return

	var start_pos: Vector2i = grid.get_position(source)
	for s: int in range(steps):
		if not source.is_active():
			break
		var success: bool = grid.move_combatant(source, direction)
		if not success:
			break

	if source.is_withdrawn:
		execution_log = "[Withdrawal] %s moved from %s and safely WITHDREW through Exit!" % [
			source.name, str(start_pos)
		]
	elif source.is_active():
		var end_pos: Vector2i = grid.get_position(source)
		if start_pos == end_pos:
			execution_log = "[%s Blocked] %s attempted to move %s from %s, but path was blocked!" % [
				command_name, source.name, str(direction), str(start_pos)
			]
		else:
			var verb: String = "sprinted" if steps > 1 else "moved"
			execution_log = "[%s] %s %s from %s to %s" % [
				command_name, source.name, verb, str(start_pos), str(end_pos)
			]
