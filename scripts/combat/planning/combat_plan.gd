class_name CombatPlan
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat Plan (a round's command sequence per unit)
# Each unit's plan is an ordered list of PlanSteps. The plan is compiled by
# simulating the unit through the round (position, stance, AP, blocks, target
# lock, expected damage), placing every command at the lowest free block.
# Hard errors (no free blocks, no path, out of reach, locked skill, moving after
# acting, rooted) make the sequence INVALID and block Confirm. Overspending AP is
# allowed: the unit goes into AP debt and pays next round (less recovery; 3+ debt
# staggers, 5+ stuns) - the compile projects it as a warning.
# Rules shaping a round:
#   - Statuses from last round take blocks away (Stagger: B1+B4, Stun: B1-B2) or
#     forbid moving (Root). They show as locked blocks on the timeline.
#   - Moving must come first (manual move, go-to, or an Engage approach), once,
#     up to 2 blocks at the unit's tiles per block, 1 AP per tile.
#   - Attacks need the target inside the skill's reach band: melee 1, ranged
#     2..range (the off-hand jab covers distance 1). Area attacks aim at a tile.
#   - Attacking a different target than the current lock inserts a free
#     Switch Target block. Stance changes are automatic (+1 AP, no block).
#   - Allies plan around each other's planned moves: a tile an ally leaves is
#     free once that ally has left it (it moves in block 1 and acts first).
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
	var ap_left: int = 0                   # projected AP (negative = AP debt)
	var max_ap: int = 10
	var total_blocks: int = 10
	var next_block: int = 1
	var blocked_blocks: Dictionary = {}    # block -> status name (Stagger / Stun)
	var rooted: bool = false
	var withdrawn: bool = false
	var acted: bool = false                # anything but a status block planned (moving is no longer allowed)
	var moved: bool = false
	var lock: Combatant = null             # current target lock (attacking another costs a Switch Target block)
	var est_hp: Dictionary = {}            # foe -> expected HP after the planned strikes
	var entries: Array[PlanEntry] = []
	var error_index: int = -1              # first invalid step (-1 = valid)
	var error_text: String = ""
	var dry_entry: int = -1                # first entry that pushes AP below zero (-1 = none)
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
		blocked_blocks = o.blocked_blocks.duplicate()
		rooted = o.rooted
		withdrawn = o.withdrawn
		acted = o.acted
		moved = o.moved
		lock = o.lock
		est_hp = o.est_hp.duplicate()
		entries = o.entries.duplicate()
		error_index = o.error_index
		error_text = o.error_text
		dry_entry = o.dry_entry
		dry_step = o.dry_step

	# Free (not status-locked) blocks from the next block on.
	func blocks_left() -> int:
		var free: int = 0
		for b: int in range(next_block, total_blocks + 1):
			if not blocked_blocks.has(b):
				free += 1
		return free

	# True when the plan overspends AP (the unit ends the round in AP debt).
	func runs_dry() -> bool:
		return ap_left < 0

	func debt() -> int:
		return maxi(0, -ap_left)

var context: PlanContext = null
var sequences: Dictionary = {}  # Combatant -> Array[PlanStep]
var _blocked_override: Variant = null   # Dictionary while evaluating an ally's first move

func _init(p_context: PlanContext) -> void:
	context = p_context

func total_blocks() -> int:
	return context.total_blocks

# Tiles a unit can cover with its one move this round.
func max_move_tiles(c: Combatant) -> int:
	return c.move_per_block() * MAX_MOVE_BLOCKS

# Distance band (min, max) at which `c` can use attack `skill`.
static func reach(c: Combatant, skill: SkillDef) -> Vector2i:
	return skill.reach_for(c)

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
# `allow_dry` false a step that would put the unit into AP debt is refused too
# (the AI plans within its means; the player may over-commit and is warned).
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

func _start_state(c: Combatant) -> SimState:
	var st := SimState.new()
	var grid: CombatGrid = context.grid
	st.pos = grid.get_position(c) if grid.has_combatant(c) else NO_CELL
	st.stance = c.current_stance
	st.ap_left = c.ap
	st.max_ap = c.max_ap
	st.total_blocks = context.total_blocks
	st.rooted = c.has_status(StatusEffects.ROOT)
	if c.active_target is Combatant:
		var locked: Combatant = c.active_target
		st.lock = locked if locked.is_active() else null
	for foe: Combatant in context.foes_of(c):
		st.est_hp[foe] = foe.hp
	# Blocks lost to last round's statuses come first on the timeline.
	for status: Variant in c.statuses:
		for b: int in StatusEffects.blocked_blocks(str(status)):
			if b <= st.total_blocks and not st.blocked_blocks.has(b):
				st.blocked_blocks[b] = str(status)
	for b: int in range(1, st.total_blocks + 1):
		if st.blocked_blocks.has(b):
			var status_name: String = str(st.blocked_blocks[b])
			var lost: PlanEntry = PlanEntry.create(SkillCatalog.find("stunned" if status_name == StatusEffects.STUN else "staggered"), b, -1)
			lost.status_name = status_name
			lost.pos_before = st.pos
			lost.pos_after = st.pos
			lost.stance_before = st.stance
			lost.stance_after = st.stance
			st.entries.append(lost)
	return st

func compile(c: Combatant) -> SimState:
	var st: SimState = _start_state(c)
	var steps: Array[PlanStep] = steps_of(c)
	for i: int in range(steps.size()):
		var backup: SimState = st.copy()
		var ap_before: int = st.ap_left
		var problem: String = _apply(c, st, steps[i], i)
		if not problem.is_empty():
			st.assign_from(backup)
			st.error_index = i
			st.error_text = problem
			break
		if st.ap_left < 0 and ap_before >= 0 and st.dry_step < 0:
			st.dry_step = i
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
		PlanStep.Mode.AREA:
			return _apply_area(c, st, step, index)
	return "Unknown step."

# "" when `c` may move now, otherwise why not.
func _move_problem(c: Combatant, st: SimState) -> String:
	if st.rooted:
		return "%s is rooted this round and can't move." % c.name
	if st.acted:
		return "Moving must come first: %s has already acted this round." % c.name
	return ""

func _apply_move(c: Combatant, st: SimState, step: PlanStep, index: int) -> String:
	var problem: String = _move_problem(c, st)
	if not problem.is_empty():
		return problem
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
	var problem: String = _move_problem(c, st)
	if not problem.is_empty():
		return problem
	var grid: CombatGrid = context.grid
	var blocked: Dictionary = blocked_cells(c)
	if not grid.is_valid_coord(step.cell) or not grid.is_walkable(step.cell) or grid.is_exit_tile(step.cell):
		return "That tile can't be moved to."
	if blocked.has(step.cell):
		return "Another unit is (or will be) standing there."
	if st.pos == step.cell:
		return "%s is already there." % c.name
	var path: Array[Vector2i] = GridPath.find_path(grid, st.pos, {step.cell: true}, blocked, false)
	if path.is_empty():
		return "No path to that tile."
	if path.size() > max_move_tiles(c):
		return "Too far: %d tiles away, %s can move %d in a round." % [path.size(), c.name, max_move_tiles(c)]
	return _add_move(c, st, path, null, index)

func _apply_withdraw(c: Combatant, st: SimState, index: int) -> String:
	var problem: String = _move_problem(c, st)
	if not problem.is_empty():
		return problem
	var exits: Dictionary = {}
	for exit_tile: Vector2i in context.grid.designated_exits:
		exits[exit_tile] = true
	if exits.is_empty():
		return "This arena has no exit."
	var path: Array[Vector2i] = GridPath.find_path(context.grid, st.pos, exits, blocked_cells(c), true)
	if path.is_empty():
		return "No route to an exit."
	var reaches_exit: bool = path.size() <= max_move_tiles(c)
	if not reaches_exit:
		path = path.slice(0, max_move_tiles(c))   # run as far as you can; finish next round
	var move_problem: String = _add_move(c, st, path, null, index)
	if move_problem.is_empty() and reaches_exit:
		st.withdrawn = true
	return move_problem

func _apply_engage(c: Combatant, st: SimState, step: PlanStep, index: int) -> String:
	var foe: Combatant = step.target
	if foe == null or not foe.is_active() or not context.grid.has_combatant(foe):
		return "Choose an enemy to attack."
	if step.skill.is_area():
		return _apply_area(c, st, PlanStep.area_at(step.skill, context.grid.get_position(foe)), index)
	var band: Vector2i = reach(c, step.skill)
	var foe_pos: Vector2i = context.grid.get_position(foe)
	var dist: int = context.grid.get_distance(st.pos, foe_pos)
	if dist < band.x or dist > band.y:
		var problem: String = _move_problem(c, st)
		if not problem.is_empty():
			if dist < band.x:
				return "%s is too close for %s (needs distance %d-%d)%s." % [foe.name, step.skill.display_name, band.x, band.y, "; use an off-hand jab" if c.attack_range > 1 else ""]
			return "%s is out of reach: %s" % [foe.name, problem]
		var path: Array[Vector2i] = approach_path(c, st.pos, foe, band.x, band.y)
		if path.is_empty():
			return "No path to get %s within reach of %s." % [c.name, foe.name]
		if path.size() > max_move_tiles(c):
			return "Too far: %d tiles to get in reach, %s can move %d in a round." % [path.size(), c.name, max_move_tiles(c)]
		var move_problem: String = _add_move(c, st, path, foe, index)
		if not move_problem.is_empty():
			return move_problem
	# Attacking someone other than the current lock costs a block to switch target.
	if st.lock != null and st.lock != foe:
		var switch_problem: String = _add_entry(c, st, SkillCatalog.find("switch_target"), index, foe)
		if not switch_problem.is_empty():
			return switch_problem
	st.lock = foe
	var attack_problem: String = _add_entry(c, st, step.skill, index, foe)
	if attack_problem.is_empty():
		st.est_hp[foe] = DictUtil.int_of(st.est_hp, foe) - st.entries[st.entries.size() - 1].est_damage
	return attack_problem

# Area attack aimed at a tile: no auto-move, the tile must already be in reach.
func _apply_area(c: Combatant, st: SimState, step: PlanStep, index: int) -> String:
	var skill: SkillDef = step.skill
	if skill == null or not skill.is_area():
		return "That isn't an area attack."
	var grid: CombatGrid = context.grid
	if not grid.is_valid_coord(step.cell):
		return "Aim inside the arena."
	var band: Vector2i = reach(c, skill)
	var dist: int = grid.get_distance(st.pos, step.cell)
	if skill.area == SkillDef.Area.LINE and (dist != 1):
		return "%s needs a tile right next to %s." % [skill.display_name, c.name]
	if skill.area != SkillDef.Area.LINE and (dist < band.x or dist > band.y):
		return "That tile is out of reach for %s (distance %d, needs %d-%d)." % [skill.display_name, dist, band.x, band.y]
	var cells: Array[Vector2i] = []
	for cell: Vector2i in skill.area_cells(st.pos, step.cell):
		if grid.is_valid_coord(cell) and cell != st.pos:
			cells.append(cell)
	var problem: String = _add_entry(c, st, skill, index, null, [], 0, cells)
	if problem.is_empty():
		var entry: PlanEntry = st.entries[st.entries.size() - 1]
		for foe: Combatant in context.foes_of(c):
			if foe.is_active() and grid.has_combatant(foe) and cells.has(grid.get_position(foe)):
				var hit: int = estimate_damage(c, entry.power, skill, entry.stance_after, foe)
				entry.est_damage += hit
				st.est_hp[foe] = DictUtil.int_of(st.est_hp, foe) - hit
		for friend: Combatant in context.friends_of(c):
			if friend != c and friend.is_active() and cells.has(planned_position(friend)):
				entry.hits_allies += 1
	return problem

# Unit steps from `from` to the nearest tile at distance min..max of `foe`
# (defaults: the unit's own reach band). Empty when already there or unreachable.
func approach_path(c: Combatant, from: Vector2i, foe: Combatant, near: int = -1, far: int = -1) -> Array[Vector2i]:
	var grid: CombatGrid = context.grid
	var foe_pos: Vector2i = grid.get_position(foe)
	var basic: Vector2i = SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(c)
	var lo: int = near if near > 0 else basic.x
	var hi: int = far if far > 0 else basic.y
	var blocked: Dictionary = blocked_cells(c)
	var goals: Dictionary = {}
	for dx: int in range(-hi, hi + 1):
		for dy: int in range(-hi, hi + 1):
			var d: int = absi(dx) + absi(dy)
			var cell: Vector2i = foe_pos + Vector2i(dx, dy)
			if d >= lo and d <= hi and grid.is_valid_coord(cell) and grid.is_walkable(cell) and not grid.is_exit_tile(cell) and not blocked.has(cell):
				goals[cell] = true
	return GridPath.find_path(grid, from, goals, blocked, false)

# The move as one Move entry per block (up to the unit's tiles per block, 1 AP
# per tile): a 2-block move covers its first stretch at the end of block 1 and the
# rest at the end of block 2. `chase` makes both halves live approaches.
func _add_move(c: Combatant, st: SimState, dirs: Array[Vector2i], chase: Combatant, index: int) -> String:
	var skill: SkillDef = SkillCatalog.lowest(SkillDef.Kind.MOVE)
	var goal: Vector2i = st.pos
	for d: Vector2i in dirs:
		goal += d
	var per_block: int = c.move_per_block()
	var i: int = 0
	while i < dirs.size():
		var chunk: Array[Vector2i] = dirs.slice(i, mini(dirs.size(), i + per_block))
		var problem: String = _add_entry(c, st, skill, index, chase, chunk, 1)
		if not problem.is_empty():
			return problem
		for d: Vector2i in chunk:
			st.pos += d
		var entry: PlanEntry = st.entries[st.entries.size() - 1]
		entry.pos_after = st.pos
		entry.goal = goal
		i += chunk.size()
	st.moved = true
	return ""

# Lowest block where `n` consecutive blocks are free (not status-locked), or -1.
func _free_start(st: SimState, n: int) -> int:
	for s: int in range(st.next_block, st.total_blocks - n + 2):
		var ok: bool = true
		for b: int in range(s, s + n):
			if st.blocked_blocks.has(b):
				ok = false
				break
		if ok:
			return s
	return -1

# Places one skill at the lowest free block and pays its AP (skill + automatic
# stance switch). Overspending is allowed (AP debt), and flagged.
func _add_entry(c: Combatant, st: SimState, skill: SkillDef, index: int, foe: Combatant = null, dirs: Array[Vector2i] = [], blocks_override: int = 0, cells: Array[Vector2i] = []) -> String:
	var nblocks: int = blocks_override if blocks_override > 0 else skill.blocks
	var start: int = _free_start(st, nblocks)
	if start < 0:
		return "Not enough free blocks: %s needs %d in a row, %d left." % [skill.display_name, nblocks, st.blocks_left()]
	var entry: PlanEntry = PlanEntry.create(skill, start, index)
	entry.blocks_override = blocks_override
	entry.target = foe
	entry.power = context.power_of(c)
	entry.path = dirs
	entry.tiles = dirs.size()
	entry.cells = cells
	entry.direction = dirs[0] if not dirs.is_empty() else Vector2i.ZERO
	entry.skill_ap = skill.ap_cost(c) * (maxi(1, dirs.size()) if skill.kind == SkillDef.Kind.MOVE else 1)
	entry.switch_ap = skill.switch_cost(st.stance)
	entry.stance_before = st.stance
	entry.stance_after = skill.stance_after(st.stance)
	entry.pos_before = st.pos
	entry.pos_after = st.pos
	if skill.kind == SkillDef.Kind.ATTACK and foe != null and not skill.is_area():
		entry.est_damage = estimate_damage(c, entry.power, skill, entry.stance_after, foe)
	st.entries.append(entry)
	st.next_block = start + nblocks
	st.acted = true
	var was_solvent: bool = st.ap_left >= 0
	st.ap_left -= entry.total_ap()
	if st.ap_left < 0 and was_solvent and st.dry_entry < 0:
		st.dry_entry = st.entries.size() - 1
	st.stance = entry.stance_after
	if skill.kind == SkillDef.Kind.REST:
		st.ap_left = mini(st.max_ap, st.ap_left + skill.recover_ap)
	return ""

# Expected hit: strike power x skill x stance matchup (against the target's stance
# now) x the attacker's attack boost. Ignores the target's guard.
func estimate_damage(c: Combatant, power: int, skill: SkillDef, attacker_stance: CombatTypes.StanceType, foe: Combatant) -> int:
	var mult: float = CombatTypes.get_matchup_multiplier(attacker_stance, foe.current_stance)
	return maxi(1, roundi(float(power) * skill.power_mult * mult * c.attack_boost_multiplier))

# ------------------------------------------------------------------------------
# Occupancy: enemies where they stand, allies where their plans take them
# ------------------------------------------------------------------------------

# Where `c` ends up after its planned move (its current tile if it isn't moving).
func planned_position(c: Combatant) -> Vector2i:
	var moves: Dictionary = _ally_moves()
	var raw: Variant = moves.get(c, null)
	if raw is Dictionary:
		var move: Dictionary = raw
		var to_raw: Variant = move.get("to", NO_CELL)
		return to_raw if to_raw is Vector2i else NO_CELL
	return context.grid.get_position(c) if context.grid.has_combatant(c) else NO_CELL

# Tiles `c` can't enter or path through.
func blocked_cells(c: Combatant) -> Dictionary:
	if _blocked_override is Dictionary:
		var override: Dictionary = _blocked_override
		return override
	return _blocked_for(c, _ally_moves())

# Blocked tiles for `c` given its friends' planned moves (friend -> {to, end, speed}).
# A friend's start tile is free only if it leaves in block 1 and acts before `c`.
func _blocked_for(c: Combatant, moves: Dictionary) -> Dictionary:
	var blocked: Dictionary = {}
	var grid: CombatGrid = context.grid
	var friends: Array[Combatant] = context.friends_of(c)
	for other: Combatant in context.allies + context.enemies:
		if other == c or not grid.has_combatant(other):
			continue
		var start: Vector2i = grid.get_position(other)
		var raw: Variant = moves.get(other, null) if friends.has(other) else null
		if raw is Dictionary:
			var move: Dictionary = raw
			var to_raw: Variant = move.get("to", NO_CELL)
			var to: Vector2i = to_raw if to_raw is Vector2i else NO_CELL
			if to != NO_CELL:
				blocked[to] = true
			var leaves_first: bool = DictUtil.int_of(move, "leave", 99) == 1 and other.speed > c.speed
			if not leaves_first:
				blocked[start] = true
		else:
			blocked[start] = true
	return blocked

# Each planning unit's move (from its first step): {to, end}. Two passes so a
# unit can step into a tile a faster friend vacates in block 1.
func _ally_moves() -> Dictionary:
	var first_pass: Dictionary = {}
	for key: Variant in sequences:
		if key is Combatant:
			var unit: Combatant = key
			var move: Dictionary = _first_move(unit, _blocked_for(unit, {}))
			if not move.is_empty():
				first_pass[unit] = move
	var second_pass: Dictionary = {}
	for key: Variant in sequences:
		if key is Combatant:
			var unit: Combatant = key
			var move: Dictionary = _first_move(unit, _blocked_for(unit, first_pass))
			if not move.is_empty():
				second_pass[unit] = move
	return second_pass

# Destination and end block of `c`'s opening move, evaluated against `blocked`.
func _first_move(c: Combatant, blocked: Dictionary) -> Dictionary:
	var steps: Array[PlanStep] = steps_of(c)
	if steps.is_empty() or not c.is_active():
		return {}
	var mode: PlanStep.Mode = steps[0].mode
	if mode != PlanStep.Mode.MOVE and mode != PlanStep.Mode.GOTO and mode != PlanStep.Mode.WITHDRAW and mode != PlanStep.Mode.ENGAGE:
		return {}
	var previous: Variant = _blocked_override
	_blocked_override = blocked
	var st: SimState = _start_state(c)
	var problem: String = _apply(c, st, steps[0], 0)
	_blocked_override = previous
	if not problem.is_empty():
		return {}
	var leave: int = -1
	var last: PlanEntry = null
	for e: PlanEntry in st.entries:
		if e.skill.kind == SkillDef.Kind.MOVE:
			if leave < 0:
				leave = e.end_block()
			last = e
	if last == null:
		return {}
	return {"to": NO_CELL if st.withdrawn else last.pos_after, "end": last.end_block(), "leave": leave}

# ------------------------------------------------------------------------------
# Valid next inputs
# ------------------------------------------------------------------------------

# Every candidate next step for `c`, probed against the current sequence.
func options(c: Combatant) -> Array[PlanOption]:
	var out: Array[PlanOption] = []
	var base: SimState = compile(c)
	if base.error_index >= 0 or base.withdrawn or not c.is_active():
		return out
	for foe: Combatant in context.foes_of(c):
		out.append_array(target_options(c, foe))
	out.append_array(target_options(c, c))
	if not context.grid.designated_exits.is_empty():
		out.append(_probe(c, PlanStep.withdraw(), base))
	return out

# Options when `c` clicks `target`: attacks (single and area) for a foe, self
# skills for itself, nothing for allies yet.
func target_options(c: Combatant, target: Combatant) -> Array[PlanOption]:
	var out: Array[PlanOption] = []
	var base: SimState = compile(c)
	if base.error_index >= 0 or base.withdrawn or not c.is_active():
		return out
	var cap: int = context.skill_cap_of(c)
	if target == c:
		for kind: SkillDef.Kind in [SkillDef.Kind.DEFEND, SkillDef.Kind.FOCUS, SkillDef.Kind.REST]:
			for skill: SkillDef in SkillCatalog.available(kind, cap):
				out.append(_probe(c, PlanStep.self_skill(skill), base))
	elif context.foes_of(c).has(target) and target.is_active() and context.grid.has_combatant(target):
		for skill: SkillDef in SkillCatalog.single_attacks(c, cap):
			out.append(_probe(c, PlanStep.engage(skill, target), base))
		var foe_cell: Vector2i = context.grid.get_position(target)
		for skill: SkillDef in SkillCatalog.area_attacks(cap):
			out.append(_probe(c, PlanStep.area_at(skill, foe_cell), base))
	return out

# Options when `c` clicks an empty tile (or the exit): move there, or aim an area attack.
func cell_options(c: Combatant, cell: Vector2i) -> Array[PlanOption]:
	var out: Array[PlanOption] = []
	var base: SimState = compile(c)
	if base.error_index >= 0 or base.withdrawn or not c.is_active():
		return out
	if context.grid.is_exit_tile(cell):
		out.append(_probe(c, PlanStep.withdraw(), base))
		return out
	out.append(_probe(c, PlanStep.go_to(SkillCatalog.lowest(SkillDef.Kind.MOVE), cell), base))
	for skill: SkillDef in SkillCatalog.area_attacks(context.skill_cap_of(c)):
		out.append(_probe(c, PlanStep.area_at(skill, cell), base))
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
	if not option.valid:
		return option
	var hits_allies: int = 0
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
			hits_allies += e.hits_allies
	var debt: int = st.debt()
	if debt > 0:
		var penalty: String = StatusEffects.debt_status(debt)
		option.warning = "AP debt %d: %s next round." % [debt, ("%s and less AP" % StatusEffects.display_name(penalty)) if not penalty.is_empty() else "less AP"]
	elif hits_allies > 0:
		option.warning = "Friendly fire: hits %d all%s." % [hits_allies, "y" if hits_allies == 1 else "ies"]
	elif step.mode == PlanStep.Mode.ENGAGE and DictUtil.int_of(base.est_hp, step.target) <= 0:
		option.warning = "%s is already expected to fall; this is an insurance attack." % step.target.name
	if step.mode == PlanStep.Mode.SELF and step.skill.kind == SkillDef.Kind.REST:
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
