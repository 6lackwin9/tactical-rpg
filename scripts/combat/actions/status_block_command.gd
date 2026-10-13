class_name StatusBlockCommand
extends BattleCommand

# ==============================================================================
# Tactical RPG - Status Block (a block lost to Stagger / Stun)
# Occupies the block on the unit's timeline so nothing else can use it.
# ==============================================================================

var status: String = ""

func _init(p_source: Combatant, p_status: String) -> void:
	super(p_source, 0, 1)
	status = p_status
	action_type = CombatTypes.ActionType.HOLD_STANCE
	is_self_targeted = true
	command_name = StatusEffects.display_name(p_status)

func execute() -> void:
	execution_log = "[Status] %s is %s and loses this block." % [source.name, StatusEffects.display_name(status).to_lower()]
