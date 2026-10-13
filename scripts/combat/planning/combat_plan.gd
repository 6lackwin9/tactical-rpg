class_name CombatPlan
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat Plan (per-round command plans on the 10-block timeline)
# Holds each unit's PlanEntry list for the round being planned, validates new
# entries (timeline bounds, overlaps, AP budget, targets), and commits them to a
# CombatResolver. Headless: used by the player planning UI and the enemy AI.
# ==============================================================================

const TOTAL_BLOCKS: int = 10
const ROUND_AP: int = 12

var entries: Dictionary = {}  # Combatant -> Array[PlanEntry], sorted by start block

func entries_for(c: Combatant) -> Array[PlanEntry]:
	var result: Array[PlanEntry] = []
	var raw: Variant = entries.get(c, null)
	if raw is Array:
		var raw_list: Array = raw
		result.assign(raw_list)
	return result

func used_ap(c: Combatant) -> int:
	var total: int = 0
	for e: PlanEntry in entries_for(c):
		total += e.ap_cost()
	return total

func ap_left(c: Combatant) -> int:
	return ROUND_AP - used_ap(c)

func entry_at(c: Combatant, block: int) -> PlanEntry:
	for e: PlanEntry in entries_for(c):
		if block >= e.start_block and block <= e.end_block():
			return e
	return null

func free_blocks(c: Combatant) -> int:
	var used: int = 0
	for e: PlanEntry in entries_for(c):
		used += e.blocks()
	return TOTAL_BLOCKS - used

# Entries that `entry` would overlap on `c`'s timeline.
func overlapping(c: Combatant, entry: PlanEntry) -> Array[PlanEntry]:
	var result: Array[PlanEntry] = []
	for e: PlanEntry in entries_for(c):
		if not (entry.end_block() < e.start_block or entry.start_block > e.end_block()):
			result.append(e)
	return result

# Empty string when `entry` can be placed, otherwise the reason it can't.
# With `replace`, overlapped entries are treated as removed (and refunded).
func check(c: Combatant, entry: PlanEntry, replace: bool = false) -> String:
	if c == null or not c.is_active():
		return "%s cannot act." % (c.name if c != null else "Unit")
	if entry.start_block < 1 or entry.end_block() > TOTAL_BLOCKS:
		return "%s needs %d block(s) and doesn't fit there." % [PlanEntry.kind_name(entry.kind), entry.blocks()]
	if PlanEntry.needs_direction(entry.kind) and absi(entry.direction.x) + absi(entry.direction.y) != 1:
		return "Pick a direction for %s." % PlanEntry.kind_name(entry.kind)
	if PlanEntry.needs_target(entry.kind) and (entry.target == null or not entry.target.is_active()):
		return "Select an enemy to attack."
	var clashes: Array[PlanEntry] = overlapping(c, entry)
	var refund: int = 0
	if not clashes.is_empty():
		if not replace:
			return "Blocks %d-%d are already planned." % [entry.start_block, entry.end_block()]
		for e: PlanEntry in clashes:
			refund += e.ap_cost()
	if used_ap(c) - refund + entry.ap_cost() > ROUND_AP:
		return "Not enough AP: %s costs %d, %d left." % [PlanEntry.kind_name(entry.kind), entry.ap_cost(), ap_left(c) + refund]
	return ""

# Places `entry`; returns "" on success or the rejection reason.
func add(c: Combatant, entry: PlanEntry, replace: bool = false) -> String:
	var problem: String = check(c, entry, replace)
	if not problem.is_empty():
		return problem
	var list: Array[PlanEntry] = entries_for(c)
	if replace:
		for e: PlanEntry in overlapping(c, entry):
			list.erase(e)
	list.append(entry)
	list.sort_custom(func(a: PlanEntry, b: PlanEntry) -> bool: return a.start_block < b.start_block)
	entries[c] = list
	return ""

func remove_at(c: Combatant, block: int) -> bool:
	var e: PlanEntry = entry_at(c, block)
	if e == null:
		return false
	var list: Array[PlanEntry] = entries_for(c)
	list.erase(e)
	entries[c] = list
	return true

func set_entries(c: Combatant, list: Array[PlanEntry]) -> void:
	entries[c] = list

func clear(c: Combatant) -> void:
	entries.erase(c)

func clear_all() -> void:
	entries.clear()

func is_empty() -> bool:
	for c: Variant in entries:
		var raw: Variant = entries[c]
		if raw is Array:
			var raw_list: Array = raw
			if not raw_list.is_empty():
				return false
	return true

# Schedules every planned command on `resolver`; returns how many were accepted.
func commit(resolver: CombatResolver, grid: CombatGrid) -> int:
	var count: int = 0
	for key: Variant in entries:
		if not (key is Combatant):
			continue
		var c: Combatant = key
		for e: PlanEntry in entries_for(c):
			if resolver.schedule_action(c, e.build_command(c, grid), e.start_block):
				count += 1
	return count

# Unregistered ScheduledActions mirroring the plan (timeline preview while planning).
func preview_actions(grid: CombatGrid) -> Array[CombatResolver.ScheduledAction]:
	var result: Array[CombatResolver.ScheduledAction] = []
	for key: Variant in entries:
		if not (key is Combatant):
			continue
		var c: Combatant = key
		for e: PlanEntry in entries_for(c):
			result.append(CombatResolver.ScheduledAction.new(c, e.build_command(c, grid), e.start_block, e.end_block()))
	return result
