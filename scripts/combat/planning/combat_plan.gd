class_name CombatPlan
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat Plan (a round's command sequence per unit)
# Each unit's plan is an ordered list of PlanSteps. The plan is compiled by
# simulating the unit through the round (position, stance, AP, blocks, target
# lock, expected damage), placing every command at the lowest free block:
#   - hard errors (block overrun, no path, out of range after moving, locked
#     skill) make the sequence INVALID and block Confirm;
#   - running out of AP is NOT an error: AP is a runtime resource. The compile
#     projects it and flags where the unit "runs dry" (`dry_step`); at
#     resolution the sequence simply stops there. Steps after it are not compiled.
# Rules shaping a round:
#   - Stance changes are automatic: +1 AP, no block (Move keeps stance; Rest
#     returns to Neutral for free).
#   - A unit may MOVE once per round (manual move, go-to, or the approach of an
#     Engage): up to 2 blocks, up to its speed-based tiles per block, 1 AP per tile.
#   - After that, attacks need the target in range; attacking a different target
#     than the current lock costs a Switch Target block (no AP).
# Headless: used by the player's planning UI, the AI presets and the enemy AI.
# ==============================================================================

const MAX_MOVE_BLOCKS: int = 2
const DIRS: Array[Vector2i] = [Vector2i(1, 0), Vector2i(0, -1), Vector2i(-1, 0), Vector2i(0, 1)]
const NO_CELL: Vector2i = Vector2i(-1, -1)

# The simulated end state of a (prefix of a) sequence.
class SimState:
	extends RefCounted
	var pos: Vector2i = Vector2i(-1, -1)
	var stance: CombatTypes.StanceType = CombatTypes.StanceType.NEUTRAL
	var ap_left: int = 0                   # projected AP after the commands that can be paid
	var max_ap: int = 10
	var total_blocks: int = 10
	var next_block: int = 1
	var withdrawn: bool = false
	var moved: bool = false                # the round's one move has been used
	var lock: Combatant = null             # current target lock (attacking another costs a Switch Target block)
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
		total_blocks = o.total_blocks
		next_block = o.next_block
		withdrawn = o.withdrawn
		moved = o.moved
		lock = o.lock
		est_hp = o.est_hp.duplicate()
		entries = o.entries.duplicate()
		error_index = o.error_index
		error_text = o.error_text
		dry_entry = o.dry_entry
		dry_step = o.dry_step

	func blocks_left() -> int:
		return total_blocks - next_block + 1

	func runs_dry() -> bool:
		return dry_entry >= 0

var context: PlanContext = null
var sequences: Dictionary = {}  # Combatant -> Array[PlanStep]

func _init(p_context: PlanContext) -> void:
	context = p_context

func total_blocks() -> int:
	return context.total_blocks

# Tiles a unit can cover with its one move this round.
func max_move_tiles(c: Combatant) -> int:
	return c.move_per_block() * MAX_MOVE_BLOCKS

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
	st.total_blocks = context.total_blocks
	if c.active_target is Combatant:
		var locked: Combatant = c.active_target
		st.lock = locked if locked.is_active() else null
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
	if st.moved:
		return "%s has already moved this round." % c.name
	if absi(step.direction.x) + absi(step.direction.y) != 1:
		return "Pick a direction to move."
	if step.tiles < 1 or step.tiles > max_move_tiles(c):
		return "%s can move at most %d tiles in a round." % [c.name, max_move_tiles(c)]
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
	if st.moved:
		return "%s has already moved this round." % c.name
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
	if path.size() > max_move_tiles(c):
		return "Too far: %d tiles away, %s can move %d in a round." % [path.size(), c.name, max_move_tiles(c)]
	return _add_move(c, st, path, null, index)

func _apply_withdraw(c: Combatant, st: SimState, index: int) -> String:
	if st.moved:
		return "%s has already moved this round." % c.name
	var exits: Dictionary = {}
	for exit_tile: Vector2i in context.grid.designated_exits:
		exits[exit_tile] = true
	if exits.is_empty():
		return "This arena has no exit."
	var path: Array[Vector2i] = find_path(context.grid, st.pos, exits, blocked_cells(c), true)
	if path.is_empty():
		return "No route to an exit."
	var reaches_exit: bool = path.size() <= max_move_tiles(c)
	if not reaches_exit:
		path = path.slice(0, max_move_tiles(c))   # run as far as you can; finish next round
	var problem: String = _add_move(c, st, path, null, index)
	if problem.is_empty() and not st.runs_dry() and reaches_exit:
		st.withdrawn = true
	return problem

func _apply_engage(c: Combatant, st: SimState, step: PlanStep, index: int) -> String:
	var foe: Combatant = step.target
	if foe == null or not foe.is_active() or not context.grid.has_combatant(foe):
		return "Choose an enemy to attack."
	var foe_pos: Vector2i = context.grid.get_position(foe)
	var in_range: bool = context.grid.get_distance(st.pos, foe_pos) <= c.attack_range
	var path: Array[Vector2i] = []
	if not in_range:
		if st.moved:
			return "%s is out of range and %s has already moved this round." % [foe.name, c.name]
		path = approach_path(c, st.pos, foe)
		if path.is_empty():
			return "No path to get %s within range of %s." % [c.name, foe.name]
		if path.size() > max_move_tiles(c):
			return "Too far: %d tiles to get in range, %s can move %d in a round." % [path.size(), c.name, max_move_tiles(c)]
	# Attacking someone other than the current lock costs a block to switch target.
	if st.lock != null and st.lock != foe:
		var switch_problem: String = _add_entry(c, st, SkillCatalog.find("switch_target"), index, foe)
		if not switch_problem.is_empty():
			return switch_problem
		if st.runs_dry():
			return ""
	st.lock = foe
	if not path.is_empty():
		var move_problem: String = _add_move(c, st, path, foe, index)
		if not move_problem.is_empty():
			return move_problem
		if st.runs_dry():
			return ""
	var problem: String = _add_entry(c, st, step.skill, index, foe)
	if problem.is_empty() and not st.runs_dry():
		st.est_hp[foe] = DictUtil.int_of(st.est_hp, foe) - st.entries[st.entries.size() - 1].est_damage
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

# One Move entry for the whole path: 1-2 blocks (the unit's tiles per block), 1 AP
# per tile. `chase` makes it a live approach toward that foe.
func _add_move(c: Combatant, st: SimState, dirs: Array[Vector2i], chase: Combatant, index: int) -> String:
	var skill: SkillDef = SkillCatalog.lowest(SkillDef.Kind.MOVE)
	var blocks: int = ceili(float(dirs.size()) / float(c.move_per_block()))
	var problem: String = _add_entry(c, st, skill, index, chase, dirs, maxi(1, blocks))
	if problem.is_empty() and not st.runs_dry():
		for d: Vector2i in dirs:
			st.pos += d
		st.entries[st.entries.size() - 1].pos_after = st.pos
		st.moved = true
	return problem

# Places one skill at the lowest free block: checks blocks, then pays AP (skill +
# automatic stance switch). If AP can't cover it the entry is still recorded but
# flagged as where the unit runs dry (no further state changes are applied).
func _add_entry(c: Combatant, st: SimState, skill: SkillDef, index: int, foe: Combatant = null, dirs: Array[Vector2i] = [], blocks_override: int = 0) -> String:
	var nblocks: int = blocks_override if blocks_override > 0 else skill.blocks
	if nblocks > st.blocks_left():
		return "Not enough blocks: %s needs %d, %d left." % [skill.display_name, nblocks, st.blocks_left()]
	var entry: PlanEntry = PlanEntry.create(skill, st.next_block, index)
	entry.blocks_override = blocks_override
	entry.target = foe
	entry.power = context.power_of(c)
	entry.path = dirs
	entry.tiles = dirs.size()
	entry.direction = dirs[0] if not dirs.is_empty() else Vector2i.ZERO
	entry.skill_ap = skill.ap_cost(c) * (maxi(1, dirs.size()) if skill.kind == SkillDef.Kind.MOVE else 1)
	entry.switch_ap = skill.switch_cost(st.stance)
	entry.stance_before = st.stance
	entry.stance_after = skill.stance_after(st.stance)
	entry.pos_before = st.pos
	entry.pos_after = st.pos
	if skill.kind == SkillDef.Kind.ATTACK and foe != null:
		entry.est_damage = estimate_damage(c, entry.power, skill, entry.stance_after, foe)
	st.entries.append(entry)
	st.next_block += nblocks
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

# Expected hit: strike power x skill x stance matchup (against the target's stance
# now) x the attacker's attack boost. Ignores the target's guard.
func estimate_damage(c: Combatant, power: int, skill: SkillDef, attacker_stance: CombatTypes.StanceType, foe: Combatant) -> int:
	var mult: float = CombatTypes.get_matchup_multiplier(attacker_stance, foe.current_stance)
	return maxi(1, roundi(float(power) * skill.power_mult * mult * c.attack_boost_multiplier))

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
	for foe: Combatant in context.foes_of(c):
		out.append_array(target_options(c, foe))
	out.append_array(target_options(c, c))
	if not context.grid.designated_exits.is_empty():
		out.append(_probe(c, PlanStep.withdraw(), base))
	return out

# Options when `c` clicks `target`: attacks for a foe, self skills for itself, nothing for allies yet.
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

# Options when `c` clicks an empty tile (or the exit).
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

# The compiled state as if `step` were appended (without changing the plan).
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
	var st: SimState = preview(c, step)
	option.valid = st.error_index < 0
	option.reason = st.error_text
	if option.valid:
		for i: int in range(base.entries.size(), st.entries.size()):
			var e: PlanEntry = st.entries[i]
			option.ap += e.total_ap()
			option.blocks += e.blocks()
			if e.skill.kind == SkillDef.Kind.MOVE:
				option.moves += 1
			if e.switch_ap > 0:
				option.switch_ap += e.switch_ap
			if e.skill.kind == SkillDef.Kind.TARGET:
				option.switches_target = true
			if e.skill.kind == SkillDef.Kind.ATTACK:
				option.damage = e.est_damage
		if st.runs_dry():
			option.warning = "Not enough AP (%d left): the sequence would stop here." % base.ap_left
		elif step.mode == PlanStep.Mode.ENGAGE and DictUtil.int_of(base.est_hp, step.target) <= 0:
			option.warning = "%s is already expected to fall; this is an insurance attack." % step.target.name
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

# Shortest route helper (kept in GridPath so commands don't depend on the planner).
static func find_path(grid: CombatGrid, start: Vector2i, goals: Dictionary, blocked: Dictionary, goals_are_exits: bool) -> Array[Vector2i]:
	return GridPath.find_path(grid, start, goals, blocked, goals_are_exits)
