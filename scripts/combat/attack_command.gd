class_name AttackCommand
extends BattleCommand

var target: Combatant
var power: int
var grid: CombatGrid = null

func _init(p_source: Combatant, p_target: Combatant = null, p_power: int = 25, p_block_cost: int = 2, p_priority: int = 0, p_grid: CombatGrid = null) -> void:
	super(p_source, p_priority, p_block_cost)
	target = p_target
	power = p_power
	grid = p_grid
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
	if not source.is_active():
		return false
	var effective_target: Combatant = _get_effective_target()
	if effective_target == null or not effective_target.is_active():
		return false
	if grid != null and not grid.can_target(source, effective_target, source.attack_range):
		return false
	return true

func execute() -> void:
	if not source.is_active():
		print("[%s] cannot act (defeated or withdrawn)." % source.name)
		execution_log = "[Action Failed] %s cannot act (unit is inactive)." % source.name
		return

	var effective_target: Combatant = _get_effective_target()
	if effective_target == null or not effective_target.is_active():
		var target_name: String = effective_target.name if effective_target != null else "None"
		print("[%s] targeted [%s], but they are no longer in active combat." % [source.name, target_name])
		execution_log = "[Attack Miss] %s targeted %s, but target is no longer in active combat." % [source.name, target_name]
		return

	if grid != null and not grid.can_target(source, effective_target, source.attack_range):
		var dist: int = grid.get_distance(grid.get_position(source), grid.get_position(effective_target))
		print("[%s] cannot reach [%s] (range %d vs dist %d)!" % [source.name, effective_target.name, source.attack_range, dist])
		execution_log = "[Attack Out of Range] %s cannot reach %s (distance %d exceeds range %d)!" % [
			source.name, effective_target.name, dist, source.attack_range
		]
		return

	# Calculate stance matchup and buff modifiers
	var stance_mult: float = CombatTypes.get_matchup_multiplier(source.current_stance, effective_target.current_stance)
	var buff_mult: float = source.attack_boost_multiplier
	var raw_dmg: float = float(power) * stance_mult * buff_mult
	var final_power: int = maxi(1, int(roundf(raw_dmg)))
	var defense_mit_pct: int = int(roundf(effective_target.defense_mitigation_ratio * 100.0))

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

	var details: Array[String] = []
	if stance_mult > 1.05:
		details.append("x%.2f Stance Adv" % stance_mult)
	elif stance_mult < 0.95:
		details.append("x%.2f Stance Disadv" % stance_mult)
	if buff_mult > 1.0:
		details.append("x%.1f Buff" % buff_mult)
	if defense_mit_pct > 0:
		details.append("-%d%% Defended" % defense_mit_pct)

	var detail_suffix: String = (" (" + ", ".join(details) + ")") if not details.is_empty() else ""

	if not effective_target.is_alive():
		execution_log = "[Attack] %s struck %s for %d damage%s -> %s DEFEATED!" % [
			source.name, effective_target.name, taken, detail_suffix, effective_target.name
		]
	else:
		execution_log = "[Attack] %s struck %s for %d damage%s (Target HP: %d/%d)" % [
			source.name, effective_target.name, taken, detail_suffix, effective_target.hp, effective_target.max_hp
		]