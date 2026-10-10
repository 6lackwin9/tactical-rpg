class_name FocusCommand
extends BattleCommand

var boost_multiplier: float
var duration_rounds: int

func _init(p_source: Combatant, p_boost_multiplier: float = 1.5, p_duration_rounds: int = 2, p_block_cost: int = 2, p_priority: int = 0) -> void:
	super(p_source, p_priority, p_block_cost)
	boost_multiplier = p_boost_multiplier
	duration_rounds = p_duration_rounds
	action_type = CombatTypes.ActionType.SKILL
	resulting_stance = CombatTypes.StanceType.TACTICAL
	is_self_targeted = true
	command_name = "Focus"

func execute() -> void:
	if not source.is_alive():
		return
	source.apply_attack_boost(boost_multiplier, duration_rounds)
	print("[%s] channels Focus in Tactical Stance! Attack power boosted by +%d%% for %d rounds." % [
		source.name,
		int((boost_multiplier - 1.0) * 100.0),
		duration_rounds
	])
