class_name AttackCommand
extends BattleCommand

var target: Combatant
var power: int

func _init(p_source: Combatant, p_target: Combatant = null, p_power: int = 25, p_block_cost: int = 2, p_priority: int = 0) -> void:
	super(p_source, p_priority, p_block_cost)
	target = p_target
	power = p_power
	action_type = CombatTypes.ActionType.ATTACK
	resulting_stance = CombatTypes.StanceType.OFFENSIVE
	command_name = "Attack"

func _get_effective_target() -> Combatant:
	if target != null:
		return target
	if source.active_target is Combatant:
		@warning_ignore("unsafe_cast")
		return source.active_target as Combatant
	return null

func can_execute() -> bool:
	if not super.can_execute():
		return false
	var effective_target: Combatant = _get_effective_target()
	if effective_target == null or not effective_target.is_alive():
		return false
	return true

func execute() -> void:
	if not source.is_alive():
		print("[%s] is knocked out and cannot act." % source.name)
		return

	var effective_target: Combatant = _get_effective_target()
	if effective_target == null or not effective_target.is_alive():
		var target_name: String = effective_target.name if effective_target != null else "None"
		print("[%s] targeted [%s], but they cannot be targeted." % [source.name, target_name])
		return

	# Calculate stance matchup and buff modifiers
	var stance_mult: float = CombatTypes.get_matchup_multiplier(source.current_stance, effective_target.current_stance)
	var buff_mult: float = source.attack_boost_multiplier
	var raw_dmg: float = float(power) * stance_mult * buff_mult
	var final_power: int = maxi(1, int(roundf(raw_dmg)))

	print("[%s] (%s) strikes [%s] (%s) for %d raw damage (x%.2f stance, x%.1f buff)!" % [
		source.name,
		CombatTypes.stance_to_string(source.current_stance),
		effective_target.name,
		CombatTypes.stance_to_string(effective_target.current_stance),
		final_power,
		stance_mult,
		buff_mult
	])

	var taken: int = effective_target.take_damage(final_power)
	print("  -> [%s] took %d damage (HP: %d/%d)" % [effective_target.name, taken, effective_target.hp, effective_target.max_hp])