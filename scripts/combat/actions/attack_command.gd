class_name AttackCommand
extends BattleCommand

var target: Combatant
var power: int
var grid: CombatGrid = null

# Planner attacks follow a target that stepped out of range by this many tiles
# (1 AP per tile, only if the unit still has AP). Scripted attacks never follow.
var follow_tiles: int = 0

# Filled in by execute(): what was dealt and why (shown in the log; formula hover in test mode).
var last_damage: int = 0
var last_breakdown: String = ""

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

# Steps toward a target that moved out of range, paying 1 AP per tile.
func _follow(effective_target: Combatant) -> void:
	var left: int = follow_tiles
	while left > 0 and source.ap >= 1 and not grid.can_target(source, effective_target, source.attack_range):
		var reach: int = source.attack_range
		var target_pos: Vector2i = grid.get_position(effective_target)
		var blocked: Dictionary = {}
		for other: Variant in grid.combatant_positions:
			if other != source and other is Combatant:
				blocked[grid.combatant_positions[other]] = true
		var goals: Dictionary = {}
		for dx: int in range(-reach, reach + 1):
			for dy: int in range(-reach, reach + 1):
				var cell: Vector2i = target_pos + Vector2i(dx, dy)
				if absi(dx) + absi(dy) <= reach and grid.is_valid_coord(cell) and grid.is_walkable(cell) and not grid.is_exit_tile(cell) and not blocked.has(cell):
					goals[cell] = true
		var path: Array[Vector2i] = GridPath.find_path(grid, grid.get_position(source), goals, blocked, false)
		if path.is_empty() or not grid.move_combatant(source, path[0]):
			return
		source.ap -= 1
		left -= 1

func execute() -> void:
	last_damage = 0
	last_breakdown = ""
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

	if grid != null and follow_tiles > 0:
		_follow(effective_target)

	if grid != null and not grid.can_target(source, effective_target, source.attack_range):
		var dist: int = grid.get_distance(grid.get_position(source), grid.get_position(effective_target))
		print("[%s] cannot reach [%s] (range %d vs dist %d)!" % [source.name, effective_target.name, source.attack_range, dist])
		execution_log = "[Attack Out of Range] %s cannot reach %s (distance %d exceeds range %d)!" % [
			source.name, effective_target.name, dist, source.attack_range
		]
		return

	if planner_controlled:
		source.active_target = effective_target   # the lock a later attack must switch away from

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
	last_damage = taken
	last_breakdown = "Power %d × stance %.2f (%s vs %s) × buff %.2f = %d" % [
		power, stance_mult, CombatTypes.stance_to_string(source.current_stance), CombatTypes.stance_to_string(effective_target.current_stance), buff_mult, final_power
	]
	if defense_mit_pct > 0:
		last_breakdown += " → guard -%d%% → %d dealt" % [defense_mit_pct, taken]
	else:
		last_breakdown += " → %d dealt" % taken
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
