---
class: AttackCommand
extends: BattleCommand
---

# Properties
* `target`: Combatant
* `skill_mult`: float = 1.0       # the skill's power multiplier (DamageCalculator does the rest)
* `use_offhand`: bool = false     # off-hand jab: off-hand weapon or bare hands
* `grid`: CombatGrid = null
* `follow_tiles`: int = 0
* `dash_tiles`: int = 0
* `status_rounds`: int = 1   # how long an inflicted status lasts once active
* `dash_note`: String = ""   # filled by on_start(): what the dash did (for tests / the log)
* `min_range`: int = -1
* `max_range`: int = -1
* `last_damage`: int = 0
* `last_breakdown`: String = ""
* `last_crit`: bool = false

# Methods

### `_init(p_source: Combatant, p_target: Combatant = null, p_skill_mult: float = 1.0, p_block_cost: int = 2, p_priority: int = 0, p_grid: CombatGrid = null) -> void`
```logic
super(p_source, p_priority, p_block_cost)
target = p_target
skill_mult = p_skill_mult
grid = p_grid
command_name = "Attack"
```

### `_get_effective_target() -> Combatant`
```logic
if target != null:
	return target
if source.active_target is Combatant:
	@warning_ignore("unsafe_cast")
	return source.active_target as Combatant
return null
```

### `reach_min() -> int`
```logic
return min_range if min_range > 0 else source.min_attack_range()
```

### `reach_max() -> int`
```logic
return max_range if max_range > 0 else source.attack_range
```

### `in_reach(target_unit: Combatant) -> bool`
```logic
if grid == null:
	return true
if not grid.has_combatant(source) or not grid.has_combatant(target_unit):
	return false
var dist: int = grid.get_distance(grid.get_position(source), grid.get_position(target_unit))
return dist >= reach_min() and dist <= reach_max()
```

### `can_execute() -> bool`
```logic
if not super.can_execute():
	return false
if not source.is_active():
	return false
var effective_target: Combatant = _get_effective_target()
if effective_target == null or not effective_target.is_active():
	return false
return in_reach(effective_target)
```

### `on_start() -> String`
```logic
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
```

### `_dash() -> String`
```logic
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
if path.is_empty() or not CombatPlan.dash_fits(grid, from, path, dash_tiles):
	return "[Dash Blocked] %s can't reach %s (%s)." % [source.name, effective_target.name, "no route" if path.is_empty() else ("%d tiles, move cost %d, dash covers %d" % [path.size(), GridPath.path_cost(grid, from, path), dash_tiles])]
var entered: Array[Vector2i] = []
for dir: Vector2i in path:
	if not grid.move_combatant(source, dir):
		break
	entered.append(grid.get_position(source))
var to: Vector2i = grid.get_position(source)
if to == from:
	return "[Dash Blocked] %s couldn't move toward %s." % [source.name, effective_target.name]
var line: String = "[Dash] %s dashes at %s: %s to %s" % [source.name, effective_target.name, str(from), str(to)]
for note: String in HazardRules.on_enter(grid, source, entered, rng):
	line += " " + note
return line
```

### `_follow(effective_target: Combatant) -> void`
```logic
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
```

### `execute() -> void`
```logic
last_damage = 0
last_breakdown = ""
last_crit = false
reactions.clear()
broken_targets.clear()
combo_notes.clear()
exploited.clear()
failed = false
if not source.is_active():
	failed = true
	print("[%s] cannot act (defeated or withdrawn)." % source.name)
	execution_log = "[Action Failed] %s cannot act (unit is inactive)." % source.name
	return

var effective_target: Combatant = _get_effective_target()
if effective_target == null or not effective_target.is_active():
	failed = true
	var target_name: String = effective_target.name if effective_target != null else "None"
	print("[%s] targeted [%s], but they are no longer in active combat." % [source.name, target_name])
	execution_log = "[Attack Miss] %s targeted %s, but target is no longer in active combat." % [source.name, target_name]
	return

if grid != null and not in_reach(effective_target):
	failed = true
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
var matched: Dictionary = exploit_against(effective_target) if damaging else {"status": "", "mult": 1.0}   # active statuses at the hit (S12C-L)
if damaging:
	# The one damage formula: attack power x Focus x skill x stance x crit, then guard and armor.
	var is_ranged: bool = source.attack_range > 1 and not use_offhand
	var cover: float = CoverRules.cover_between(grid, source, effective_target, is_ranged)   # terrain cover (S12T-L)
	var res: Dictionary = DamageCalculator.compute(source, effective_target, is_ranged, use_offhand, skill_mult * DictUtil.to_float(matched["mult"], 1.0), false, rng, -1, true, cover)
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
	if DictUtil.to_float(res["cover_ratio"]) > 0.0:
		details.append("-%d%% Cover" % roundi(DictUtil.to_float(res["cover_ratio"]) * 100.0))
	if DictUtil.to_int(res["armor_rating"]) > 0:
		details.append("-%d%% Armor" % roundi(DictUtil.to_float(res["armor_ratio"]) * 100.0))
var taken: int = last_damage

if not status_effect.is_empty() and effective_target.is_alive():
	var chance: int = StatusEffects.apply_chance(effective_target, status_effect, status_potency)
	if StatusEffects.try_inflict(effective_target, status_effect, status_potency, rng, status_rounds, status_magnitude, source.name):
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
# Then the combo effects of this hit (S12C-L rule 5): stance break, consume.
after_hit(effective_target, taken, matched)
note_exploit(effective_target, matched)
var exploit_suffix: String = exploit_note(matched)

if not effective_target.is_alive():
	execution_log = "[Attack] %s struck %s for %d damage%s -> %s DEFEATED!%s" % [
		source.name, effective_target.name, taken, detail_suffix, effective_target.name, exploit_suffix
	]
else:
	execution_log = "[Attack] %s struck %s for %d damage%s (Target HP: %d/%d)%s" % [
		source.name, effective_target.name, taken, detail_suffix, effective_target.hp, effective_target.max_hp, exploit_suffix
	]
```

