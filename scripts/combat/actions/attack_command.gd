class_name AttackCommand
extends BattleCommand

var target: Combatant
var skill_mult: float = 1.0       # the skill's power multiplier (DamageCalculator does the rest)
var use_offhand: bool = false     # off-hand jab: off-hand weapon or bare hands
var grid: CombatGrid = null

# Planner single-target attacks adjust by this many tiles when they start if the
# target is out of the reach band: step closer, or (ranged, target too close) step
# back. 1 AP per tile, may go into debt. Area and post-tile-move attacks use 0.
var follow_tiles: int = 0

# Gap-close (lunge): when the attack starts, move up to this many tiles to get into
# reach of the target. The skill's AP already pays for it (no per-tile cost). A rooted
# unit, or a route blocked by terrain / units, can't dash and the strike then falls short.
var dash_tiles: int = 0
var status_rounds: int = 1   # how long an inflicted status lasts once active
var dash_note: String = ""   # filled by on_start(): what the dash did (for tests / the log)

# Reach band: -1 = the unit's own (ranged units: 2..range; melee: 1). Off-hand jabs use 1..1.
var min_range: int = -1
var max_range: int = -1

# Filled in by execute(): what was dealt and why (shown in the log; formula hover in test mode).
var last_damage: int = 0
var last_breakdown: String = ""
var last_crit: bool = false

func _init(p_source: Combatant, p_target: Combatant = null, p_skill_mult: float = 1.0, p_block_cost: int = 2, p_priority: int = 0, p_grid: CombatGrid = null) -> void:
	super(p_source, p_priority, p_block_cost)
	target = p_target
	skill_mult = p_skill_mult
	grid = p_grid
	command_name = "Attack"

func _get_effective_target() -> Combatant:
	if target != null:
		return target
	if source.active_target is Combatant:
		@warning_ignore("unsafe_cast")
		return source.active_target as Combatant
	return null

func reach_min() -> int:
	return min_range if min_range > 0 else source.min_attack_range()

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
	dash_note = ""
	if dash_tiles > 0:
		dash_note = _dash()
		return dash_note
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

# Gap-close: path (around units and terrain) to the nearest tile in reach of the target,
# at most dash_tiles long. Returns the log line ("" when no dash was needed).
func _dash() -> String:
	var effective_target: Combatant = _get_effective_target()
	if grid == null or effective_target == null or not effective_target.is_active() or not source.is_active() or in_reach(effective_target):
		return ""
	if source.has_status(StatusEffects.ROOT):
		return "[Dash Blocked] %s is rooted and can't dash at %s." % [source.name, effective_target.name]
	var from: Vector2i = grid.get_position(source)
	var target_pos: Vector2i = grid.get_position(effective_target)
	var blocked: Dictionary = {}
	for other: Variant in grid.combatant_positions:
		if other != source and other is Combatant:
			var occupant: Combatant = other
			if occupant.is_active():
				blocked[grid.combatant_positions[other]] = true
	var goals: Dictionary = {}
	var hi: int = reach_max()
	for dx: int in range(-hi, hi + 1):
		for dy: int in range(-hi, hi + 1):
			var cell: Vector2i = target_pos + Vector2i(dx, dy)
			var d: int = absi(dx) + absi(dy)
			if d >= reach_min() and d <= hi and grid.is_valid_coord(cell) and grid.is_walkable(cell) and not grid.is_exit_tile(cell) and not blocked.has(cell):
				goals[cell] = true
	var path: Array[Vector2i] = GridPath.find_path(grid, from, goals, blocked, false)
	if path.is_empty() or path.size() > dash_tiles:
		return "[Dash Blocked] %s can't reach %s (%s)." % [source.name, effective_target.name, "no route" if path.is_empty() else ("%d tiles, dash covers %d" % [path.size(), dash_tiles])]
	for dir: Vector2i in path:
		if not grid.move_combatant(source, dir):
			break
	var to: Vector2i = grid.get_position(source)
	if to == from:
		return "[Dash Blocked] %s couldn't move toward %s." % [source.name, effective_target.name]
	return "[Dash] %s dashes at %s: %s to %s" % [source.name, effective_target.name, str(from), str(to)]

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
	last_crit = false
	reactions.clear()
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

	source.active_target = effective_target   # the lock a later attack must switch away from

	var details: Array[String] = []
	var damaging: bool = skill_mult > 0.0   # a zero-power attack (intimidate) only rolls its status: no damage, no crit, no reaction
	if damaging:
		# The one damage formula: attack power x Focus x skill x stance x crit, then guard and armor.
		var res: Dictionary = DamageCalculator.compute(source, effective_target, source.attack_range > 1 and not use_offhand, use_offhand, skill_mult, false, rng)
		var taken_now: int = effective_target.take_damage(DictUtil.to_int(res["final_dmg"]), true)
		last_damage = taken_now
		last_crit = DictUtil.to_bool(res["is_crit"])
		last_breakdown = DamageCalculator.describe(res)
		print("[%s] strikes [%s]: %s" % [source.name, effective_target.name, last_breakdown])
		var stance_mult: float = DictUtil.to_float(res["stance_factor"])
		if stance_mult > 1.05:
			details.append("x%.2f Stance Adv" % stance_mult)
		elif stance_mult < 0.95:
			details.append("x%.2f Stance Disadv" % stance_mult)
		if DictUtil.to_float(res["buff_mult"]) > 1.0:
			details.append("x%.1f Focus" % DictUtil.to_float(res["buff_mult"]))
		if last_crit:
			details.append("CRIT x%.1f" % DictUtil.to_float(res["crit_dmg_mult"]))
		if DictUtil.to_float(res["mit_ratio"]) > 0.0:
			details.append("-%d%% Guard" % roundi(DictUtil.to_float(res["mit_ratio"]) * 100.0))
		if DictUtil.to_int(res["armor_rating"]) > 0:
			details.append("-%d%% Armor" % roundi(DictUtil.to_float(res["armor_ratio"]) * 100.0))
	var taken: int = last_damage

	if not status_effect.is_empty() and effective_target.is_alive():
		var chance: int = StatusEffects.apply_chance(effective_target, status_effect, status_potency)
		if StatusEffects.try_inflict(effective_target, status_effect, status_potency, rng, status_rounds, status_magnitude):
			details.append("%s next round (%d%%)" % [StatusEffects.display_name(status_effect), chance])
		else:
			details.append("resisted %s (%d%%)" % [StatusEffects.display_name(status_effect).to_lower(), chance])

	var detail_suffix: String = (" (" + ", ".join(details) + ")") if not details.is_empty() else ""

	if not damaging:
		execution_log = "[Status] %s -> %s%s" % [source.name, effective_target.name, detail_suffix]
		return

	# A riposting target answers a melee hit it survived (once per attacker per round).
	var counter: Dictionary = RiposteRules.counter(source, effective_target, grid, source.attack_range > 1 and not use_offhand, rng)
	if not counter.is_empty():
		reactions.append(counter)

	if not effective_target.is_alive():
		execution_log = "[Attack] %s struck %s for %d damage%s -> %s DEFEATED!" % [
			source.name, effective_target.name, taken, detail_suffix, effective_target.name
		]
	else:
		execution_log = "[Attack] %s struck %s for %d damage%s (Target HP: %d/%d)" % [
			source.name, effective_target.name, taken, detail_suffix, effective_target.hp, effective_target.max_hp
		]
