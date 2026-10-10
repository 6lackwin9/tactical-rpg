class_name HoldStanceCommand
extends BattleCommand

func _init(p_source: Combatant, p_priority: int = 0) -> void:
	super(p_source, p_priority, 1)
	action_type = CombatTypes.ActionType.HOLD_STANCE
	resulting_stance = p_source.current_stance
	is_self_targeted = true
	command_name = "Hold Stance"

func execute() -> void:
	if not source.is_alive():
		return
	source.stance_hold_duration += 1
	print("[%s] holds %s Stance (consecutive duration: %d blocks)." % [
		source.name,
		CombatTypes.stance_to_string(source.current_stance),
		source.stance_hold_duration
	])
