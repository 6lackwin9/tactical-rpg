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
	resulting_stance = CombatTypes.StanceType.MOBILE
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
		return

	for s: int in range(steps):
		if not source.is_active():
			break
		var success: bool = grid.move_combatant(source, direction)
		if not success:
			break
