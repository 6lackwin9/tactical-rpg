class_name EnterNeutralCommand
extends BattleCommand

func _init(p_source: Combatant, p_priority: int = 1) -> void:
	super(p_source, p_priority, 1)
	action_type = CombatTypes.ActionType.ENTER_NEUTRAL
	resulting_stance = CombatTypes.StanceType.NEUTRAL
	is_self_targeted = true
	command_name = "Enter Neutral"

func execute() -> void:
	if not source.is_alive():
		return
	source.set_stance(CombatTypes.StanceType.NEUTRAL)
	print("[%s] transitions into Neutral Stance (unlocking all action pools starting next block)." % source.name)
	execution_log = "[Enter Neutral] %s transitioned to Neutral Stance" % source.name
