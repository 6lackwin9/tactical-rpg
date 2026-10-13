class_name CombatPlan
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat Plan (a round's command sequence per unit)
# Each unit's plan is an ordered list of PlanSteps. The plan is compiled by
# simulating the unit through the round (position, stance, AP, blocks, expected
# damage), placing every command at the lowest free block:
#   - hard errors (block overrun, no path, bad target, locked skill) make the
#     sequence INVALID and block Confirm;
#   - running out of AP is NOT an error: AP is a runtime resource. The compile
#     projects it and flags where the unit "runs dry" (`dry_step`); at
#     resolution the sequence simply stops there. Steps after it are not compiled.
# Stance changes are automatic: using a skill from another stance costs +1 AP
# and no block (Move keeps stance; Rest returns to Neutral for free).
# Headless: used by the player's planning UI, the AI presets and the enemy AI.
# ==============================================================================

const TOTAL_BLOCKS: int = 10
const DIRS: Array[Vector2i] = [Vector2i(1, 0), Vector2i(0, -1), Vector2i(-1, 0), Vector2i(0, 1)]
const NO_CELL: Vector2i = Vector2i(-1, -1)

# The simulated end state of a (prefix of a) sequence.
class SimState:
	extends RefCounted
	var pos: Vector2i = Vector2i(-1, -1)
	var stance: CombatTypes.StanceType = CombatTypes.StanceType.NEUTRAL
	var ap_left: int = 0                   # projected AP after the commands that can be paid
	var max_ap: int = 10
	var next_block: int = 1
	var withdrawn: bool = false
	var est_hp: Dictionary = {}            # foe -> expected HP after the planned strikes
	var entries: Array[PlanEntry] = []
	var error_index: int = -1              # first invalid step (-1 = valid)
	var error_text: String = ""
	var dry_entry: int = -1                # index into `entries` of the first command AP can't pay (-1 = none)
	var dry_step: int = -1                 # the step that produced it

	func copy() -> SimState:
		var c := SimState.new()
		c.assign_from(self)
		return c

	func assign_from(o: SimState) -> void:
		pos = o.pos
		stance = o.stance
		ap_left = o.ap_left
		max_ap = o.max_ap
		next_block = o.next_block
		withdrawn = o.withdrawn
		est_hp = o.est_hp.duplicate()
		entries = o.entries.duplicate()
		error_index = o.error_index
		error_text = o.error_text
		dry_entry = o.dry_entry
		dry_step = o.dry_step

	func blocks_left() -> int:
		return TOTAL_BLOCKS - next_block + 1

	func runs_dry() -> bool:
		return dry_entry >= 0

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

# Appends `step` if it compiles; returns "" or the reason it doesn't. With
# `allow_dry` false a step that would run the unit out of AP is refused too (the
# AI plans within its means; the player may over-commit and is warned).
func add_step(c: Combatant, step: PlanStep, allow_dry: bool = true) -> String:
	var steps: Array[PlanStep] = steps_of(c)
	steps.append(step)
	sequences[c] = steps
	var st: SimState = compile(c)
	var problem: String = st.error_text if st.error_index >= 0 else ("Not enough AP for %s." % step.title() if (not allow_dry and st.runs_dry()) else "")
	if not problem.is_empty():
		steps.pop_back()
		sequences[c] = steps
	return problem

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
	st.ap_left = c.ap
	st.max_ap = c.max_ap
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
		if st.runs_dry():
			st.dry_step = i
			break  # the unit stops here; later steps never run
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
			return _add_entry(c, st, step.skill, index)
		PlanStep.Mode.MOVE:
			return _apply_move(c, st, step, index)
		PlanStep.Mode.GOTO:
			return _apply_goto(c, st, step, index)
		PlanStep.Mode.WITHDRAW:
			return _apply_withdraw(c, st, index)
		PlanStep.Mode.ENGAGE:
			return _apply_engage(c, st, step, index)
	return "Unknown step."

func _apply_move(c: Combatant, st: SimState, step: PlanStep, index: int) -> String:
	if absi(step.direction.x) + absi(step.direction.y) != 1:
		return "Pick a direction to move."
	if step.tiles < 1 or step.tiles > c.move_per_block():
		return "%s can move at most %d tile%s per block." % [c.name, c.move_per_block(), "" if c.move_per_block() == 1 else "s"]
	var blocked: Dictionary = blocked_cells(c)
	var grid: CombatGrid = context.grid
	var cell: Vector2i = st.pos
	var path: Array[Vector2i] = []
	for i: int in range(step.tiles):
		cell += step.direction
		if not grid.is_valid_coord(cell):
			return "The arena edge blocks that move."
		if grid.is_exit_tile(cell):
			return "Use Withdraw to leave through the exit."
		if not grid.is_walkable(cell):
			return "%s blocks that move." % Terrain.display_name(grid.get_terrain(cell))
		if blocked.has(cell):
			return "Another unit blocks that move."
		path.append(step.direction)
	return _add_move(c, st, path, null, index)

func _apply_goto(c: Combatant, st: SimState, step: PlanStep, index: int) -> String:
	var grid: CombatGrid = context.grid
	var blocked: Dictionary = blocked_cells(c)
	if not grid.is_valid_coord(step.cell) or not grid.is_walkable(step.cell) or grid.is_exit_tile(step.cell):
		return "That tile can't be moved to."
	if blocked.has(step.cell):
		return "Another unit is standing there."
	if st.pos == step.cell:
		return "%s is already there." % c.name
	var path: Array[Vector2i] = find_path(grid, st.pos, {step.cell: true}, blocked, false)
	if path.is_empty():
		return "No path to that tile."
	return _walk(c, st, path, index, null)

func _apply_withdraw(c: Combatant, st: SimState, index: int) -> String:
	var exits: Dictionary = {}
	for exit_tile: Vector2i in context.grid.designated_exits:
		exits[exit_tile] = true
	if exits.is_empty():
		return "This arena has no exit."
	var path: Array[Vector2i] = find_path(context.grid, st.pos, exits, blocked_cells(c), true)
	if path.is_empty():
		return "No route to an exit."
	var problem: String = _walk(c, st, path, index, null)
	if problem.is_empty() and not st.runs_dry():
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
		if st.runs_dry():
			return ""
	var problem: String = _add_entry(c, st, step.skill, index, foe)
	if problem.is_empty() and not st.runs_dry():
		st.est_hp[foe] = DictUtil.int_of(st.est_hp, foe) - maxi(1, roundi(float(context.power_of(c)) * step.skill.power_mult))
	return problem

# Unit steps from `from` to the nearest tile within weapon range of `foe`. Empty
# when already in range or when no route exists.
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

# Walks `path` in Move blocks of up to the unit's tiles-per-block (any turns).
# `chase` makes them live approaches toward that foe.
func _walk(c: Combatant, st: SimState, path: Array[Vector2i], index: int, chase: Combatant) -> String:
	var per_block: int = c.move_per_block()
	var i: int = 0
	while i < path.size():
		var chunk: Array[Vector2i] = path.slice(i, mini(path.size(), i + per_block))
		var problem: String = _add_move(c, st, chunk, chase, index)
		if not problem.is_empty():
			return problem
		if st.runs_dry():
			return ""
		i += chunk.size()
	return ""

func _add_move(c: Combatant, st: SimState, dirs: Array[Vector2i], chase: Combatant, index: int) -> String:
	var skill: SkillDef = SkillCatalog.lowest(SkillDef.Kind.MOVE)
	var problem: String = _add_entry(c, st, skill, index, chase, dirs)
	if problem.is_empty() and not st.runs_dry():
		for d: Vector2i in dirs:
			st.pos += d
		st.entries[st.entries.size() - 1].pos_after = st.pos
	return problem

# Places one skill at the lowest free block: checks blocks, then pays AP (skill +
# automatic stance switch). If AP can't cover it the entry is still recorded but
# flagged as where the unit runs dry (no further state changes are applied).
func _add_entry(c: Combatant, st: SimState, skill: SkillDef, index: int, foe: Combatant = null, dirs: Array[Vector2i] = []) -> String:
	if skill.blocks > st.blocks_left():
		return "Not enough blocks: %s needs %d, %d left." % [skill.display_name, skill.blocks, st.blocks_left()]
	var entry: PlanEntry = PlanEntry.create(skill, st.next_block, index)
	entry.target = foe
	entry.power = context.power_of(c)
	entry.path = dirs
	entry.tiles = dirs.size()
	entry.direction = dirs[0] if not dirs.is_empty() else Vector2i.ZERO
	entry.skill_ap = skill.ap_cost(c) * (maxi(1, dirs.size()) if skill.kind == SkillDef.Kind.MOVE else 1)
	entry.switch_ap = skill.switch_cost(st.stance)
	entry.stance_before = st.stance
	entry.pos_before = st.pos
	entry.pos_after = st.pos
	entry.stance_after = skill.stance_after(st.stance)
	st.entries.append(entry)
	st.next_block += skill.blocks
	if st.runs_dry():
		return ""
	if entry.total_ap() > st.ap_left:
		st.dry_entry = st.entries.size() - 1
		return ""
	st.ap_left -= entry.total_ap()
	st.stance = entry.stance_after
	if skill.kind == SkillDef.Kind.REST:
		st.ap_left = mini(st.max_ap, st.ap_left + skill.recover_ap)
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

# Every candidate next step for `c`, probed against the current sequence. Options
# that would run the unit out of AP are listed as valid but carry a warning.
func options(c: Combatant) -> Array[PlanOption]:
	var out: Array[PlanOption] = []
	var base: SimState = compile(c)
	if base.error_index >= 0 or base.withdrawn or base.runs_dry() or not c.is_active():
		return out
	var cap: int = context.skill_cap_of(c)
	for foe: Combatant in context.foes_of(c):
		if not foe.is_active() or not context.grid.has_combatant(foe):
			continue
		for skill: SkillDef in SkillCatalog.available(SkillDef.Kind.ATTACK, cap):
			out.append(_probe(c, PlanStep.engage(skill, foe), base))
	for kind: SkillDef.Kind in [SkillDef.Kind.DEFEND, SkillDef.Kind.FOCUS, SkillDef.Kind.REST]:
		for skill: SkillDef in SkillCatalog.available(kind, cap):
			out.append(_probe(c, PlanStep.self_skill(skill), base))
	var mover: SkillDef = SkillCatalog.lowest(SkillDef.Kind.MOVE)
	for dir: Vector2i in DIRS:
		for tiles: int in range(1, c.move_per_block() + 1):
			out.append(_probe(c, PlanStep.move(mover, dir, tiles), base))
	if not context.grid.designated_exits.is_empty():
		out.append(_probe(c, PlanStep.withdraw(), base))
	return out

# Options when c clicks 	arget: attacks for a foe, self skills for itself, nothing for allies yet.
func target_options(c: Combatant, target: Combatant) -> Array[PlanOption]:
	var out: Array[PlanOption] = []
	var base: SimState = compile(c)
	if base.error_index >= 0 or base.withdrawn or base.runs_dry() or not c.is_active():
		return out
	var cap: int = context.skill_cap_of(c)
	if target == c:
		for kind: SkillDef.Kind in [SkillDef.Kind.DEFEND, SkillDef.Kind.FOCUS, SkillDef.Kind.REST]:
			for skill: SkillDef in SkillCatalog.available(kind, cap):
				out.append(_probe(c, PlanStep.self_skill(skill), base))
	elif context.foes_of(c).has(target) and target.is_active() and context.grid.has_combatant(target):
		for skill: SkillDef in SkillCatalog.available(SkillDef.Kind.ATTACK, cap):
			out.append(_probe(c, PlanStep.engage(skill, target), base))
	return out

# Options when c clicks an empty tile (or the exit).
func cell_options(c: Combatant, cell: Vector2i) -> Array[PlanOption]:
	var out: Array[PlanOption] = []
	var base: SimState = compile(c)
	if base.error_index >= 0 or base.withdrawn or base.runs_dry() or not c.is_active():
		return out
	if context.grid.is_exit_tile(cell):
		out.append(_probe(c, PlanStep.withdraw(), base))
	else:
		out.append(_probe(c, PlanStep.go_to(SkillCatalog.lowest(SkillDef.Kind.MOVE), cell), base))
	return out

# The compiled state as if step were appended (without changing the plan).
func preview(c: Combatant, step: PlanStep) -> SimState:
	var steps: Array[PlanStep] = steps_of(c)
	steps.append(step)
	sequences[c] = steps
	var st: SimState = compile(c)
	steps.pop_back()
	sequences[c] = steps
	return st

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
		for i: int in range(base.entries.size(), st.entries.size()):
			option.ap += st.entries[i].total_ap()
			option.blocks += st.entries[i].blocks()
			if st.entries[i].skill.kind == SkillDef.Kind.MOVE:
				option.moves += 1
			if st.entries[i].switch_ap > 0:
				option.switch_ap += st.entries[i].switch_ap
		if st.runs_dry():
			option.warning = "Not enough AP (%d left): the sequence would stop here." % base.ap_left
		elif step.mode == PlanStep.Mode.SELF and step.skill.kind == SkillDef.Kind.REST:
			option.ap = -step.skill.recover_ap
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
