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
	var stance_str: String = CombatTypes.stance_to_string(source.current_stance)
	print("[%s] holds %s Stance (consecutive duration: %d blocks)." % [
		source.name,
		stance_str,
		source.stance_hold_duration
	])
	execution_log = "[Hold Stance] %s maintained %s Stance (duration: %db)" % [
		source.name, stance_str, source.stance_hold_duration
	]
