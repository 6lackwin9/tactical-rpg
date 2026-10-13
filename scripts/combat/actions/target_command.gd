class_name TargetCommand
extends BattleCommand

var target_value: Variant

func _init(p_source: Combatant, p_target: Variant = null, p_priority: int = 2) -> void:
	super(p_source, p_priority, 1)
	target_value = p_target
	command_name = "Target"

func execute() -> void:
	if not source.is_alive():
		return

	if target_value != null:
		source.set_target(target_value)
		if target_value is Combatant:
			print("[%s] focuses and locks target on [%s]." % [source.name, target_value.name])
			execution_log = "[Target] %s locked target on %s" % [source.name, target_value.name]
		else:
			print("[%s] designates target location at %s." % [source.name, str(target_value)])
			execution_log = "[Target] %s designated target location at %s" % [source.name, str(target_value)]
	else:
		source.clear_target()
		print("[%s] disengages and clears active target focus." % source.name)
		execution_log = "[Target] %s disengaged and cleared target focus" % source.name
