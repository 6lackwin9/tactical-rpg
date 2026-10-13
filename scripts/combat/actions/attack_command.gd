class_name AttackCommand
extends BattleCommand

var target: Combatant
var power: int
var grid: CombatGrid = null

# Planner single-target attacks adjust by this many tiles when they start if the
# target is out of the reach band: step closer, or (ranged, target too close) step
# back. 1 AP per tile, may go into debt. Scripted, area and post-tile-move attacks use 0.
var follow_tiles: int = 0

# Reach band: -1 = the unit's own (ranged units: 2..range; melee: 1). Off-hand jabs use 1..1.
var min_range: int = -1
var max_range: int = -1

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

# Scripted (legacy) attacks keep the old 1..range band so the reference battle is unchanged.
func reach_min() -> int:
	if min_range > 0:
		return min_range
	return source.min_attack_range() if planner_controlled else 1

func reach_max() -> int:
	return max_range if max_range > 0 else source.attack_range

func in_reach(target_unit: Combatant) -> bool:
	if grid == null:
		return true
	if not grid.has_combatant(source) or not grid.has_combatant(target_unit):
		return false
	var dist: int = grid.get_distance(grid.get_position(source), grid.get_position(target_unit))
	return dist >= reach_min() and dist <= reach_max()

func can_execute() -> bool:
	if not super.can_execute():
		return false
	if not source.is_active():
		return false
	var effective_target: Combatant = _get_effective_target()
	if effective_target == null or not effective_target.is_active():
		return false
	return in_reach(effective_target)

# When the attack starts: if the target stepped out of reach, close the gap (up to
# follow_tiles, 1 AP per tile) before winding up the strike.
func on_start() -> String:
	var effective_target: Combatant = _get_effective_target()
	if grid == null or follow_tiles <= 0 or effective_target == null or not effective_target.is_active() or not source.is_active() or in_reach(effective_target):
		return ""
	var from: Vector2i = grid.get_position(source)
	var too_close: bool = grid.get_distance(from, grid.get_position(effective_target)) < reach_min()
	_follow(effective_target)
	var to: Vector2i = grid.get_position(source)
	if to == from:
		return ""
	var verb: String = "backs off from" if too_close else "closes in on"
	return "[Chase] %s %s %s: %s to %s (1 AP)" % [source.name, verb, effective_target.name, str(from), str(to)]

# Steps toward a target that moved out of range, paying 1 AP per tile.
func _follow(effective_target: Combatant) -> void:
	var left: int = follow_tiles
	while left > 0 and not in_reach(effective_target):   # may run into AP debt like any planned cost
		var reach: int = reach_max()
		var near: int = reach_min()
		var target_pos: Vector2i = grid.get_position(effective_target)
		var blocked: Dictionary = {}
		for other: Variant in grid.combatant_positions:
			if other != source and other is Combatant:
				blocked[grid.combatant_positions[other]] = true
		var goals: Dictionary = {}
		for dx: int in range(-reach, reach + 1):
			for dy: int in range(-reach, reach + 1):
				var cell: Vector2i = target_pos + Vector2i(dx, dy)
				if absi(dx) + absi(dy) <= reach and absi(dx) + absi(dy) >= near and grid.is_valid_coord(cell) and grid.is_walkable(cell) and not grid.is_exit_tile(cell) and not blocked.has(cell):
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

	if grid != null and not in_reach(effective_target):
		var dist: int = grid.get_distance(grid.get_position(source), grid.get_position(effective_target))
		print("[%s] cannot reach [%s] (range %d-%d vs dist %d)!" % [source.name, effective_target.name, reach_min(), reach_max(), dist])
		if dist < reach_min():
			execution_log = "[Attack Out of Range] %s is too close to %s for this attack (distance %d, needs %d-%d)!" % [source.name, effective_target.name, dist, reach_min(), reach_max()]
		else:
			execution_log = "[Attack Out of Range] %s cannot reach %s (distance %d exceeds range %d)!" % [
				source.name, effective_target.name, dist, reach_max()
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

	if not status_effect.is_empty() and effective_target.is_alive():
		var chance: int = StatusEffects.apply_chance(effective_target, status_effect, status_potency)
		if StatusEffects.try_inflict(effective_target, status_effect, status_potency, rng):
			details.append("%s next round (%d%%)" % [StatusEffects.display_name(status_effect), chance])
		else:
			details.append("resisted %s (%d%%)" % [StatusEffects.display_name(status_effect).to_lower(), chance])

	var detail_suffix: String = (" (" + ", ".join(details) + ")") if not details.is_empty() else ""

	if not effective_target.is_alive():
		execution_log = "[Attack] %s struck %s for %d damage%s -> %s DEFEATED!" % [
			source.name, effective_target.name, taken, detail_suffix, effective_target.name
		]
	else:
		execution_log = "[Attack] %s struck %s for %d damage%s (Target HP: %d/%d)" % [
			source.name, effective_target.name, taken, detail_suffix, effective_target.hp, effective_target.max_hp
		]
