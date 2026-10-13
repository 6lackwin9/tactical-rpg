class_name CombatPlan
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat Plan (a round's command sequence per unit)
# Each unit's plan is an ordered list of PlanSteps. The plan is compiled by
# simulating the unit through the round (position, stance, AP, blocks, expected
# damage), placing every command at the lowest free block, and validating:
#   - stance flow (CombatTypes.can_flow) - switching pools needs Reset Stance
#   - AP budget (skill level = AP) and the 10-block timeline
#   - paths, targets and exits; Engage inserts the moves to reach range
# A sequence with an error cannot be confirmed. Headless: used by the player's
# planning UI, the AI presets and the enemy AI.
# ==============================================================================

const TOTAL_BLOCKS: int = 10
const ROUND_AP: int = 10
const DIRS: Array[Vector2i] = [Vector2i(1, 0), Vector2i(0, -1), Vector2i(-1, 0), Vector2i(0, 1)]
const NO_CELL: Vector2i = Vector2i(-1, -1)

# The simulated end state of a (prefix of a) sequence.
class SimState:
	extends RefCounted
	var pos: Vector2i = Vector2i(-1, -1)
	var stance: CombatTypes.StanceType = CombatTypes.StanceType.NEUTRAL
	var ap_left: int = 0
	var next_block: int = 1
	var withdrawn: bool = false
	var est_hp: Dictionary = {}            # foe -> expected HP after the planned strikes
	var entries: Array[PlanEntry] = []
	var error_index: int = -1              # first invalid step (-1 = valid)
	var error_text: String = ""

	func copy() -> SimState:
		var c := SimState.new()
		c.assign_from(self)
		return c

	func assign_from(o: SimState) -> void:
		pos = o.pos
		stance = o.stance
		ap_left = o.ap_left
		next_block = o.next_block
		withdrawn = o.withdrawn
		est_hp = o.est_hp.duplicate()
		entries = o.entries.duplicate()
		error_index = o.error_index
		error_text = o.error_text

	func blocks_left() -> int:
		return TOTAL_BLOCKS - next_block + 1

var context: PlanContext = null
var sequences: Dictionary = {}  # Combatant -> Array[PlanStep]

func _init(p_context: PlanContext) -> void:
	context = p_context

# ------------------------------------------------------------------------------
# Sequence editing
# ------------------------------------------------------------------------------

func steps_of(c: Combatant) -> Array[PlanStep]:
	var result: Array[PlanStep] = []
	var raw: Variant = sequences.get(c, null)
	if raw is Array:
		var raw_list: Array = raw
		result.assign(raw_list)
	return result

# Appends `step` if it is valid as the next input; returns "" or the reason it isn't.
func add_step(c: Combatant, step: PlanStep) -> String:
	var steps: Array[PlanStep] = steps_of(c)
	steps.append(step)
	sequences[c] = steps
	var st: SimState = compile(c)
	if st.error_index >= 0:
		steps.pop_back()
		sequences[c] = steps
		return st.error_text
	return ""

func remove_step(c: Combatant, index: int) -> void:
	var steps: Array[PlanStep] = steps_of(c)
	if index >= 0 and index < steps.size():
		steps.remove_at(index)
		sequences[c] = steps

# Reorders a step. The result may be invalid; compile() flags where.
func move_step(c: Combatant, from_index: int, to_index: int) -> void:
	var steps: Array[PlanStep] = steps_of(c)
	if from_index < 0 or from_index >= steps.size() or to_index < 0 or to_index >= steps.size() or from_index == to_index:
		return
	var step: PlanStep = steps[from_index]
	steps.remove_at(from_index)
	steps.insert(to_index, step)
	sequences[c] = steps

func set_steps(c: Combatant, steps: Array[PlanStep]) -> void:
	sequences[c] = steps

func clear(c: Combatant) -> void:
	sequences.erase(c)

func clear_all() -> void:
	sequences.clear()

func is_empty() -> bool:
	for key: Variant in sequences:
		var raw: Variant = sequences[key]
		if raw is Array:
			var raw_list: Array = raw
			if not raw_list.is_empty():
				return false
	return true

# ------------------------------------------------------------------------------
# Compilation: simulate the unit through its sequence
# ------------------------------------------------------------------------------

func compile(c: Combatant) -> SimState:
	var st := SimState.new()
	var grid: CombatGrid = context.grid
	st.pos = grid.get_position(c) if grid.has_combatant(c) else NO_CELL
	st.stance = c.current_stance
	st.ap_left = ROUND_AP
	for foe: Combatant in context.foes_of(c):
		st.est_hp[foe] = foe.hp
	var steps: Array[PlanStep] = steps_of(c)
	for i: int in range(steps.size()):
		var backup: SimState = st.copy()
		var problem: String = _apply(c, st, steps[i], i)
		if not problem.is_empty():
			st.assign_from(backup)
			st.error_index = i
			st.error_text = problem
			break
	return st

func entries_for(c: Combatant) -> Array[PlanEntry]:
	return compile(c).entries

func is_valid(c: Combatant) -> bool:
	return compile(c).error_index < 0

func ap_left(c: Combatant) -> int:
	return compile(c).ap_left

func _apply(c: Combatant, st: SimState, step: PlanStep, index: int) -> String:
	if st.withdrawn:
		return "%s has already left the battle." % c.name
	if not c.is_active() or st.pos == NO_CELL:
		return "%s is not on the field." % c.name
	var cap: int = context.skill_cap_of(c)
	if step.skill != null and step.skill.level > cap:
		return "%s isn't unlocked yet (needs skill level %d)." % [step.skill.display_name, step.skill.level]
	match step.mode:
		PlanStep.Mode.SELF:
			return _add_entry(c, st, step.skill, Vector2i.ZERO, null, index)
		PlanStep.Mode.MOVE:
			return _apply_move(c, st, step, index)
		PlanStep.Mode.WITHDRAW:
			return _apply_withdraw(c, st, index)
		PlanStep.Mode.ENGAGE:
			return _apply_engage(c, st, step, index)
	return "Unknown step."

func _apply_move(c: Combatant, st: SimState, step: PlanStep, index: int) -> String:
	if absi(step.direction.x) + absi(step.direction.y) != 1:
		return "Pick a direction to move."
	var blocked: Dictionary = blocked_cells(c)
	var grid: CombatGrid = context.grid
	var cell: Vector2i = st.pos
	for i: int in range(step.skill.steps):
		cell += step.direction
		if not grid.is_valid_coord(cell):
			return "The arena edge blocks that move."
		if grid.is_exit_tile(cell):
			return "Use Withdraw to leave through the exit."
		if not grid.is_walkable(cell):
			return "%s blocks that move." % Terrain.display_name(grid.get_terrain(cell))
		if blocked.has(cell):
			return "Another unit blocks that move."
	var problem: String = _add_entry(c, st, step.skill, step.direction, null, index)
	if problem.is_empty():
		st.pos = cell
	return problem

func _apply_withdraw(c: Combatant, st: SimState, index: int) -> String:
	var exits: Dictionary = {}
	for exit_tile: Vector2i in context.grid.designated_exits:
		exits[exit_tile] = true
	if exits.is_empty():
		return "This arena has no exit."
	var path: Array[Vector2i] = find_path(context.grid, st.pos, exits, blocked_cells(c), true)
	if path.is_empty():
		return "No route to an exit."
	var problem: String = _walk(c, st, path, index)
	if problem.is_empty():
		st.withdrawn = true
	return problem

func _apply_engage(c: Combatant, st: SimState, step: PlanStep, index: int) -> String:
	var foe: Combatant = step.target
	if foe == null or not foe.is_active() or not context.grid.has_combatant(foe):
		return "Choose an enemy to attack."
	if DictUtil.int_of(st.est_hp, foe) <= 0:
		return "%s is already expected to fall." % foe.name
	var foe_pos: Vector2i = context.grid.get_position(foe)
	if context.grid.get_distance(st.pos, foe_pos) > c.attack_range:
		var path: Array[Vector2i] = approach_path(c, st.pos, foe)
		if path.is_empty():
			return "No path to get %s within range of %s." % [c.name, foe.name]
		var walk_problem: String = _walk(c, st, path, index, foe)
		if not walk_problem.is_empty():
			return walk_problem
	var problem: String = _add_entry(c, st, step.skill, Vector2i.ZERO, foe, index)
	if problem.is_empty():
		st.est_hp[foe] = DictUtil.int_of(st.est_hp, foe) - maxi(1, roundi(float(context.power_of(c)) * step.skill.power_mult))
	return problem

# Walks `path` (unit steps) using the biggest unlocked move skill for each
# straight run, so higher move levels cost fewer blocks for the same AP per tile.
func approach_path(c: Combatant, from: Vector2i, foe: Combatant) -> Array[Vector2i]:
	var grid: CombatGrid = context.grid
	var foe_pos: Vector2i = grid.get_position(foe)
	var reach: int = c.attack_range
	var blocked: Dictionary = blocked_cells(c)
	var goals: Dictionary = {}
	for dx: int in range(-reach, reach + 1):
		for dy: int in range(-reach, reach + 1):
			var cell: Vector2i = foe_pos + Vector2i(dx, dy)
			if absi(dx) + absi(dy) <= reach and grid.is_valid_coord(cell) and grid.is_walkable(cell) and not grid.is_exit_tile(cell) and not blocked.has(cell):
				goals[cell] = true
	return find_path(grid, from, goals, blocked, false)

func _walk(c: Combatant, st: SimState, path: Array[Vector2i], index: int, chase: Combatant = null) -> String:
	var movers: Array[SkillDef] = SkillCatalog.available(SkillDef.Kind.MOVE, context.skill_cap_of(c))
	movers.reverse()
	var i: int = 0
	while i < path.size():
		var dir: Vector2i = path[i]
		var run: int = 1
		while i + run < path.size() and path[i + run] == dir:
			run += 1
		var left: int = run
		while left > 0:
			var chosen: SkillDef = null
			for m: SkillDef in movers:
				if m.steps <= left:
					chosen = m
					break
			if chosen == null:
				return "No move skill available."
			var problem: String = _add_entry(c, st, chosen, dir, chase, index)
			if not problem.is_empty():
				return problem
			st.pos += dir * chosen.steps
			left -= chosen.steps
		i += run
	return ""

# Places one skill at the lowest free block after checking stance, blocks and AP.
func _add_entry(c: Combatant, st: SimState, skill: SkillDef, dir: Vector2i, foe: Combatant, index: int) -> String:
	if not CombatTypes.can_flow(st.stance, skill.stance()):
		return "%s stance can't flow into %s. Add Reset Stance first." % [CombatTypes.stance_to_string(st.stance), skill.display_name]
	if skill.blocks > st.blocks_left():
		return "Not enough blocks: %s needs %d, %d left." % [skill.display_name, skill.blocks, st.blocks_left()]
	var cost: int = skill.ap_cost(c)
	if cost > st.ap_left:
		return "Not enough AP: %s costs %d, %d left." % [skill.display_name, cost, st.ap_left]
	st.entries.append(PlanEntry.create(skill, st.next_block, dir, foe, context.power_of(c), index, cost))
	st.next_block += skill.blocks
	st.ap_left -= cost
	st.stance = skill.stance()
	return ""

# Tiles occupied by other units (their round-start positions).
func blocked_cells(c: Combatant) -> Dictionary:
	var blocked: Dictionary = {}
	for other: Combatant in context.allies + context.enemies:
		if other != c and context.grid.has_combatant(other):
			blocked[context.grid.get_position(other)] = true
	return blocked

# ------------------------------------------------------------------------------
# Valid next inputs
# ------------------------------------------------------------------------------

# Every candidate next step for `c`, probed against the current sequence.
func options(c: Combatant) -> Array[PlanOption]:
	var out: Array[PlanOption] = []
	var base: SimState = compile(c)
	if base.error_index >= 0 or base.withdrawn or not c.is_active():
		return out
	var cap: int = context.skill_cap_of(c)
	for foe: Combatant in context.foes_of(c):
		if not foe.is_active() or not context.grid.has_combatant(foe):
			continue
		for skill: SkillDef in SkillCatalog.available(SkillDef.Kind.ATTACK, cap):
			out.append(_probe(c, PlanStep.engage(skill, foe), base))
	for kind: SkillDef.Kind in [SkillDef.Kind.DEFEND, SkillDef.Kind.FOCUS, SkillDef.Kind.NEUTRAL]:
		for skill: SkillDef in SkillCatalog.available(kind, cap):
			out.append(_probe(c, PlanStep.self_skill(skill), base))
	for skill: SkillDef in SkillCatalog.available(SkillDef.Kind.MOVE, cap):
		for dir: Vector2i in DIRS:
			out.append(_probe(c, PlanStep.move(skill, dir), base))
	if not context.grid.designated_exits.is_empty():
		out.append(_probe(c, PlanStep.withdraw(), base))
	return out

func _probe(c: Combatant, step: PlanStep, base: SimState) -> PlanOption:
	var option := PlanOption.new()
	option.step = step
	var steps: Array[PlanStep] = steps_of(c)
	steps.append(step)
	sequences[c] = steps
	var st: SimState = compile(c)
	steps.pop_back()
	sequences[c] = steps
	option.valid = st.error_index < 0
	option.reason = st.error_text
	if option.valid:
		option.ap = base.ap_left - st.ap_left
		option.blocks = st.next_block - base.next_block
		for i: int in range(base.entries.size(), st.entries.size()):
			if st.entries[i].skill.kind == SkillDef.Kind.MOVE:
				option.moves += 1
	return option

# ------------------------------------------------------------------------------
# Hand-off to the resolver / timeline preview
# ------------------------------------------------------------------------------

# Schedules every unit's compiled commands; returns how many were accepted.
func commit(resolver: CombatResolver) -> int:
	var count: int = 0
	for key: Variant in sequences:
		if not (key is Combatant):
			continue
		var c: Combatant = key
		for e: PlanEntry in entries_for(c):
			if resolver.schedule_action(c, e.build_command(c, context.grid), e.start_block):
				count += 1
	return count

# Unregistered ScheduledActions mirroring the plan (timeline preview while planning).
func preview_actions() -> Array[CombatResolver.ScheduledAction]:
	var result: Array[CombatResolver.ScheduledAction] = []
	for key: Variant in sequences:
		if not (key is Combatant):
			continue
		var c: Combatant = key
		for e: PlanEntry in entries_for(c):
			result.append(CombatResolver.ScheduledAction.new(c, e.build_command(c, context.grid), e.start_block, e.end_block()))
	return result

# ------------------------------------------------------------------------------
# Path-finding (shared with TacticalAI)
# ------------------------------------------------------------------------------

# Shortest list of unit steps from `start` to any goal tile. Empty when already
# on a goal or when no goal is reachable. Exit tiles are only entered as goals.
static func find_path(grid: CombatGrid, start: Vector2i, goals: Dictionary, blocked: Dictionary, goals_are_exits: bool) -> Array[Vector2i]:
	var steps: Array[Vector2i] = []
	if goals.has(start):
		return steps
	var came_from: Dictionary = {start: start}
	var queue: Array[Vector2i] = [start]
	var found: Vector2i = NO_CELL
	var head: int = 0
	while head < queue.size() and found == NO_CELL:
		var cur: Vector2i = queue[head]
		head += 1
		for d: Vector2i in DIRS:
			var nxt: Vector2i = cur + d
			if came_from.has(nxt) or not grid.is_valid_coord(nxt):
				continue
			var is_goal: bool = goals.has(nxt)
			if is_goal:
				if not goals_are_exits and (blocked.has(nxt) or not grid.is_walkable(nxt)):
					continue
			elif blocked.has(nxt) or not grid.is_walkable(nxt) or grid.is_exit_tile(nxt):
				continue
			came_from[nxt] = cur
			if is_goal:
				found = nxt
				break
			queue.append(nxt)
	if found == NO_CELL:
		return steps
	var cell: Vector2i = found
	while cell != start:
		var prev: Vector2i = came_from[cell]
		steps.push_front(cell - prev)
		cell = prev
	return steps
