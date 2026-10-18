---
class: CombatPlan
extends: RefCounted
description: "Tactical RPG - Combat Plan (a round's command sequence per unit) Each unit's plan is an ordered list of PlanSteps. The plan is compiled by simulating the unit through the round (position, stance, AP, blocks, target lock, expected damage), placing every command at the lowest free block. Hard errors (no free blocks, no path, out of reach, locked skill, moving after acting, rooted) make the sequence INVALID and block Confirm. Overspending AP is allowed: the unit goes into AP debt and pays next round (less recovery; 3+ debt staggers, 5+ stuns) - the compile projects it as a warning. Rules shaping a round: - Statuses from last round take blocks away (Stagger: B1+B4, Stun: B1-B2) or forbid moving (Root). They show as locked blocks on the timeline. - Moving must come first (manual move, go-to, or an Engage approach), once, up to 2 blocks at the unit's tiles per block, 1 AP per tile. - Attacks need the target inside the skill's reach band: melee 1, ranged 2..range (the off-hand jab covers distance 1). Area attacks aim at a tile. - Attacking a different target than the current lock inserts a free Switch Target block. Stance changes are automatic (+1 AP, no block). - Allies plan around each other's planned moves: a tile an ally leaves is free once that ally has left it (it moves in block 1 and acts first). Headless: used by the player's planning UI, the AI presets and the enemy AI."
---

# Properties
* `MAX_MOVE_BLOCKS`: int = 2 (const)
* `DIRS`: Array[Vector2i] = [Vector2i(1, 0), Vector2i(0, -1), Vector2i(-1, 0), Vector2i(0, 1)] (const)
* `NO_CELL`: Vector2i = Vector2i(-1, -1) (const)
* `path_penalties`: Dictionary = {}
* `goal_preferences`: Dictionary = {}
* `ai_pressure`: Dictionary = {}
* `pos`: Vector2i = Vector2i(-1, -1)
* `stance`: CombatTypes.StanceType = CombatTypes.StanceType.NEUTRAL
* `ap_left`: int = 0                   # projected AP (negative = AP debt)
* `max_ap`: int = 10
* `total_blocks`: int = 10
* `next_block`: int = 1
* `blocked_blocks`: Dictionary = {}    # block -> status name (Stagger / Stun)
* `rooted`: bool = false
* `withdrawn`: bool = false
* `acted`: bool = false                # anything but a status block planned (moving is no longer allowed)
* `moved`: bool = false
* `positioned`: bool = false           # moved by hand (tile move / withdraw): its attacks don't chase
* `lock`: Combatant = null            # current target lock (attacking another costs a Switch Target block)
* `est_hp`: Dictionary = {}            # foe -> expected HP after the planned strikes
* `consumed`: Dictionary = {}          # foe -> Array[String] statuses an earlier planned exploit hit consumes (S12C-L)
* `entries`: Array[PlanEntry] = []
* `error_index`: int = -1              # first invalid step (-1 = valid)
* `error_text`: String = ""
* `dry_entry`: int = -1                # first entry that pushes AP below zero (-1 = none)
* `dry_step`: int = -1                 # the step that produced it
* `context`: PlanContext = null
* `sequences`: Dictionary = {}  # Combatant -> Array[PlanStep]
* `_blocked_override`: Variant = null   # Dictionary while evaluating an ally's first move
* `result`: Array[PlanStep] = []
* `raw`: Variant = sequences.get(c, null)
* `raw_list`: Array = raw
* `grid`: CombatGrid = context.grid
* `foe_pos`: Vector2i = grid.get_position(foe)
* `basic`: Vector2i = SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(c)
* `lo`: int = near if near > 0 else basic.x
* `hi`: int = far if far > 0 else basic.y
* `blocked`: Dictionary = blocked_cells(c)
* `goals`: Dictionary = {}
* `d`: int = absi(dx) + absi(dy)
* `cell`: Vector2i = foe_pos + Vector2i(dx, dy)
* `penalty`: Callable = path_penalties.get(c, Callable())
* `prefer`: Callable = goal_preferences.get(c, Callable())
* `used`: Array[String] = []
* `raw`: Variant = st.consumed.get(foe, [])
* `raw_list`: Array = raw
* `out`: Array[PlanOption] = []
* `base`: SimState = compile(c)
* `out`: Array[PlanOption] = []
* `base`: SimState = compile(c)
* `cap`: int = context.skill_cap_of(c)
* `foe_cell`: Vector2i = context.grid.get_position(target)
* `out`: Array[PlanOption] = []
* `start`: SimState = base if base != null else compile(c)
* `con`: Consumable = ItemCommand.pouch_item(c, item_id)
* `left`: int = items_left(c, item_id)
* `option`: PlanOption = _probe(c, PlanStep.use_item(item_id, target), start)
* `out`: Array[PlanOption] = []
* `base`: SimState = compile(c)
* `out`: Array[Vector2i] = []
* `cell`: Vector2i = e.pos_before
* `result`: Array[CombatResolver.ScheduledAction] = []
* `c`: Combatant = key

# Methods

### `copy() -> SimState`
```logic
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
	positioned = o.positioned
	lock = o.lock
	est_hp = o.est_hp.duplicate()
	consumed = o.consumed.duplicate(true)
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
```

### `_init(p_context: PlanContext) -> void`
```logic
context = p_context
```

### `total_blocks() -> int`
```logic
return context.total_blocks
```

### `max_move_tiles(c: Combatant) -> int`
```logic
return c.move_per_block() * MAX_MOVE_BLOCKS
```

### `move_blocks_needed(c: Combatant, from: Vector2i, path: Array[Vector2i]) -> int`
```logic
return GridPath.blocks_needed(context.grid, from, path, c.move_per_block())
```

### `fits_move(c: Combatant, from: Vector2i, path: Array[Vector2i]) -> bool`
```logic
return move_blocks_needed(c, from, path) <= MAX_MOVE_BLOCKS
```

### `_too_far(c: Combatant, from: Vector2i, path: Array[Vector2i], what: String) -> String`
```logic
var cost: int = GridPath.path_cost(context.grid, from, path)
return "Too far: %d tiles %s (move cost %d), %s can move %d cost per block for %d blocks in a round." % [path.size(), what, cost, c.name, c.move_per_block(), MAX_MOVE_BLOCKS]
```

### `static reach(c: Combatant, skill: SkillDef) -> Vector2i`
```logic
return skill.reach_for(c)
```

### `add_step(c: Combatant, step: PlanStep, allow_dry: bool = true) -> String`
```logic
var steps: Array[PlanStep] = steps_of(c)
steps.append(step)
sequences[c] = steps
var st: SimState = compile(c)
var problem: String = st.error_text if st.error_index >= 0 else ("Not enough AP for %s." % step.title() if (not allow_dry and st.runs_dry()) else "")
if not problem.is_empty():
	steps.pop_back()
	sequences[c] = steps
return problem
```

### `remove_step(c: Combatant, index: int) -> void`
```logic
var steps: Array[PlanStep] = steps_of(c)
if index >= 0 and index < steps.size():
	steps.remove_at(index)
	sequences[c] = steps
```

### `move_step(c: Combatant, from_index: int, to_index: int) -> void`
```logic
var steps: Array[PlanStep] = steps_of(c)
if from_index < 0 or from_index >= steps.size() or to_index < 0 or to_index >= steps.size() or from_index == to_index:
	return
var step: PlanStep = steps[from_index]
steps.remove_at(from_index)
steps.insert(to_index, step)
sequences[c] = steps
```

### `set_steps(c: Combatant, steps: Array[PlanStep]) -> void`
```logic
sequences[c] = steps
```

### `clear(c: Combatant) -> void`
```logic
sequences.erase(c)
```

### `clear_all() -> void`
```logic
sequences.clear()
```

### `is_empty() -> bool`
```logic
for key: Variant in sequences:
	var raw: Variant = sequences[key]
	if raw is Array:
		var raw_list: Array = raw
		if not raw_list.is_empty():
			return false
return true
```

### `_start_state(c: Combatant) -> SimState`
```logic
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
```

### `compile(c: Combatant) -> SimState`
```logic
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
```

### `is_valid(c: Combatant) -> bool`
```logic
return compile(c).error_index < 0
```

### `ap_left(c: Combatant) -> int`
```logic
return compile(c).ap_left
```

### `_apply(c: Combatant, st: SimState, step: PlanStep, index: int) -> String`
```logic
if st.withdrawn:
	return "%s has already left the battle." % c.name
if not c.is_active() or st.pos == NO_CELL:
	return "%s is not on the field." % c.name
var cap: int = context.skill_cap_of(c)
if step.skill != null and step.skill.level > cap:
	return "%s isn't unlocked yet (needs skill level %d)." % [step.skill.display_name, step.skill.level]
if step.skill != null and not c.knows_skill(step.skill.id):
	return "%s doesn't know %s." % [c.name, step.skill.display_name]
if step.skill != null and not c.tier_unlocked(step.skill):
	return "%s unlocks at unit level %d." % [step.skill.display_name, c.skill_unlock_level(step.skill)]
if step.skill != null and step.skill.kind == SkillDef.Kind.DISENGAGE:
	return _apply_disengage(c, st, step, index)
match step.mode:
	PlanStep.Mode.SELF:
		if step.skill.kind == SkillDef.Kind.SUPPORT:
			return _apply_support_self(c, st, step, index)
		return _add_entry(c, st, step.skill, index)
	PlanStep.Mode.ALLY:
		return _apply_ally(c, st, step, index)
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
	PlanStep.Mode.ITEM:
		return _apply_item(c, st, step, index)
return "Unknown step."
```

### `_move_problem(c: Combatant, st: SimState) -> String`
```logic
if st.rooted:
	return "%s is rooted this round and can't move." % c.name
if st.acted:
	return "Moving must come first: %s has already acted this round." % c.name
return ""
```

### `_apply_move(c: Combatant, st: SimState, step: PlanStep, index: int) -> String`
```logic
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
if not fits_move(c, st.pos, path):
	return _too_far(c, st.pos, path, "that way")
return _add_move(c, st, path, null, index)
```

### `_apply_goto(c: Combatant, st: SimState, step: PlanStep, index: int) -> String`
```logic
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
var path: Array[Vector2i] = _route(c, st.pos, {step.cell: true}, blocked, false)
if path.is_empty():
	return "No path to that tile."
if not fits_move(c, st.pos, path):
	return _too_far(c, st.pos, path, "away")
return _add_move(c, st, path, null, index)
```

### `_apply_withdraw(c: Combatant, st: SimState, index: int) -> String`
```logic
var problem: String = _move_problem(c, st)
if not problem.is_empty():
	return problem
var exits: Dictionary = {}
for exit_tile: Vector2i in context.grid.designated_exits:
	exits[exit_tile] = true
if exits.is_empty():
	return "This arena has no exit."
var path: Array[Vector2i] = _route(c, st.pos, exits, blocked_cells(c), true)
if path.is_empty():
	return "No route to an exit."
var reaches_exit: bool = fits_move(c, st.pos, path)
if not reaches_exit:
	path = GridPath.fit_blocks(context.grid, st.pos, path, c.move_per_block(), MAX_MOVE_BLOCKS)   # run as far as you can; finish next round
var move_problem: String = _add_move(c, st, path, null, index)
if move_problem.is_empty() and reaches_exit:
	st.withdrawn = true
return move_problem
```

### `_apply_engage(c: Combatant, st: SimState, step: PlanStep, index: int) -> String`
```logic
var foe: Combatant = step.target
if foe == null or not foe.is_active() or not context.grid.has_combatant(foe):
	return "Choose an enemy to attack."
if step.skill.is_area():
	return _apply_area(c, st, PlanStep.area_at(step.skill, context.grid.get_position(foe)), index)
if step.skill.dash_tiles > 0:
	return _apply_dash(c, st, step, index)
var band: Vector2i = reach(c, step.skill)
var foe_pos: Vector2i = context.grid.get_position(foe)
var dist: int = context.grid.get_distance(st.pos, foe_pos)
if dist < band.x or dist > band.y:
	var problem: String = _move_problem(c, st)
	if not problem.is_empty():
		if dist < band.x:
			return "%s is too close for %s (needs distance %d-%d)%s." % [foe.name, step.skill.display_name, band.x, band.y, "; use an off-hand jab" if c.attack_range > 1 else ""]
		return "%s is out of reach: %s" % [foe.name, problem]
	# The approach destination is fixed when the step is planned (step.cell); later
	# compiles re-check that same tile instead of re-routing around other plans.
	var path: Array[Vector2i] = []
	if step.cell != NO_CELL:
		var stored_dist: int = context.grid.get_distance(step.cell, foe_pos)
		if stored_dist < band.x or stored_dist > band.y:
			return "The planned approach to %s no longer reaches it." % foe.name
		if blocked_cells(c).has(step.cell):
			return "The planned approach tile %s is taken; re-plan this attack." % str(step.cell)
		path = _route(c, st.pos, {step.cell: true}, blocked_cells(c), false)
	else:
		path = approach_path(c, st.pos, foe, band.x, band.y)
		if not path.is_empty():
			var dest: Vector2i = st.pos
			for d: Vector2i in path:
				dest += d
			step.cell = dest
	if path.is_empty():
		return "No path to get %s within reach of %s." % [c.name, foe.name]
	if not fits_move(c, st.pos, path):
		return _too_far(c, st.pos, path, "to get in reach")
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
	var attack_entry: PlanEntry = st.entries[st.entries.size() - 1]
	# A damage-focused attack adjusts by one tile if its target shifted; after a
	# hand-picked tile move the unit holds its position.
	attack_entry.chase = not st.positioned
	st.est_hp[foe] = DictUtil.int_of(st.est_hp, foe) - attack_entry.est_damage
return attack_problem
```

### `_apply_dash(c: Combatant, st: SimState, step: PlanStep, index: int) -> String`
```logic
var skill: SkillDef = step.skill
var foe: Combatant = step.target
var grid: CombatGrid = context.grid
var band: Vector2i = reach(c, skill)
var dist: int = grid.get_distance(st.pos, grid.get_position(foe))
if dist >= band.x and dist <= band.y:
	return "%s is already in reach; %s is for closing a gap." % [foe.name, skill.display_name]
if dist < band.x:
	return "%s is too close for %s." % [foe.name, skill.display_name]
var problem: String = _move_problem(c, st)
if not problem.is_empty():
	return "%s can't dash: %s" % [c.name, problem]
var path: Array[Vector2i] = approach_path(c, st.pos, foe, band.x, band.y)
if path.is_empty():
	return "No route for %s to get within reach of %s." % [c.name, foe.name]
if not dash_fits(context.grid, st.pos, path, skill.dash_tiles):
	return "%s is too far for %s: %d tiles to cross (move cost %d), it covers %d." % [foe.name, skill.display_name, path.size(), GridPath.path_cost(context.grid, st.pos, path), skill.dash_tiles]
if st.lock != null and st.lock != foe:
	var switch_problem: String = _add_entry(c, st, SkillCatalog.find("switch_target"), index, foe)
	if not switch_problem.is_empty():
		return switch_problem
var from: Vector2i = st.pos
var dest: Vector2i = from
for d: Vector2i in path:
	dest += d
st.lock = foe
var attack_problem: String = _add_entry(c, st, skill, index, foe, path)
if attack_problem.is_empty():
	var entry: PlanEntry = st.entries[st.entries.size() - 1]
	entry.pos_before = from
	entry.pos_after = dest
	entry.goal = dest
	st.pos = dest
	st.moved = true
	st.positioned = false
	st.est_hp[foe] = DictUtil.int_of(st.est_hp, foe) - entry.est_damage
return attack_problem
```

### `static dash_fits(grid: CombatGrid, from: Vector2i, path: Array[Vector2i], dash_tiles: int) -> bool`
```logic
return dash_tiles > 0 and GridPath.blocks_needed(grid, from, path, dash_tiles) <= 1
```

### `_apply_disengage(c: Combatant, st: SimState, step: PlanStep, index: int) -> String`
```logic
var skill: SkillDef = step.skill
var problem: String = _move_problem(c, st)
if not problem.is_empty():
	return "%s can't disengage: %s" % [c.name, problem]
var grid: CombatGrid = context.grid
var threat: Combatant = step.target
if threat == null:
	var best_dist: int = 1 << 20
	for foe: Combatant in context.foes_of(c):
		if foe.is_active() and grid.has_combatant(foe):
			var d: int = grid.get_distance(st.pos, grid.get_position(foe))
			if d < best_dist:
				best_dist = d
				threat = foe
if threat == null or not threat.is_active() or not grid.has_combatant(threat) or not context.foes_of(c).has(threat):
	return "No enemy to jump away from."
var threat_pos: Vector2i = grid.get_position(threat)
var here: int = grid.get_distance(st.pos, threat_pos)
var blocked: Dictionary = blocked_cells(c)
var reachable: Dictionary = jump_tiles_from(st.pos, skill.jump_tiles)
var dest: Vector2i = NO_CELL
var dest_steps: int = 0
if step.cell != NO_CELL:
	if not reachable.has(step.cell) or blocked.has(step.cell):
		return "%s can't jump to %s (walls, other units or more than %d tiles)." % [c.name, str(step.cell), skill.jump_tiles]
	if grid.get_distance(step.cell, threat_pos) <= here:
		return "%s is not away from %s: a disengage must end farther from the enemy." % [str(step.cell), threat.name]
	dest = step.cell
	dest_steps = DictUtil.int_of(reachable, dest)
else:
	for key: Variant in reachable:
		var cell: Vector2i = key
		if blocked.has(cell):
			continue
		var d: int = grid.get_distance(cell, threat_pos)
		if d <= here:
			continue
		if dest == NO_CELL or _better_jump(grid, cell, d, DictUtil.int_of(reachable, cell), dest, grid.get_distance(dest, threat_pos), dest_steps):
			dest = cell
			dest_steps = DictUtil.int_of(reachable, cell)
	if dest == NO_CELL:
		return "No free tile within %d to jump to away from %s." % [skill.jump_tiles, threat.name]
	step.cell = dest
var from: Vector2i = st.pos
var add_problem: String = _add_entry(c, st, skill, index, threat)
if add_problem.is_empty():
	var entry: PlanEntry = st.entries[st.entries.size() - 1]
	entry.tiles = dest_steps
	entry.pos_before = from
	entry.pos_after = dest
	entry.goal = dest
	st.pos = dest
	st.moved = true
	st.positioned = true
return add_problem
```

### `_better_jump(grid: CombatGrid, a: Vector2i, a_dist: int, a_steps: int, b: Vector2i, b_dist: int, b_steps: int) -> bool`
```logic
if a_dist != b_dist:
	return a_dist > b_dist
if a_steps != b_steps:
	return a_steps < b_steps
var a_centre: int = CombatGrid.centre_distance(a)
var b_centre: int = CombatGrid.centre_distance(b)
if a_centre != b_centre:
	return a_centre < b_centre
if a.x != b.x:
	return a.x < b.x
return a.y < b.y
```

### `jump_tiles_from(from: Vector2i, max_steps: int) -> Dictionary`
```logic
var grid: CombatGrid = context.grid
var depth: Dictionary = {from: 0}
var queue: Array[Vector2i] = [from]
var head: int = 0
while head < queue.size():
	var cur: Vector2i = queue[head]
	head += 1
	var cur_depth: int = DictUtil.int_of(depth, cur)
	if cur_depth >= max_steps:
		continue
	for d: Vector2i in DIRS:
		var nxt: Vector2i = cur + d
		if depth.has(nxt) or not grid.is_valid_coord(nxt) or not grid.is_walkable(nxt) or grid.is_exit_tile(nxt):
			continue
		depth[nxt] = cur_depth + 1
		queue.append(nxt)
depth.erase(from)
return depth
```

### `_apply_support_self(c: Combatant, st: SimState, step: PlanStep, index: int) -> String`
```logic
if step.skill.target_mode == SkillDef.TargetMode.ALLY:
	return "%s must be aimed at an ally." % step.skill.display_name
var problem: String = _add_entry(c, st, step.skill, index, c)
if problem.is_empty():
	var entry: PlanEntry = st.entries[st.entries.size() - 1]
	entry.est_heal = SupportRules.effective_heal(c, SupportRules.heal_amount(c, step.skill))
	entry.est_cleanse = SupportRules.cleansable(c, step.skill)
return problem
```

### `_apply_ally(c: Combatant, st: SimState, step: PlanStep, index: int) -> String`
```logic
var skill: SkillDef = step.skill
var ally: Combatant = step.target
if skill == null or skill.kind != SkillDef.Kind.SUPPORT:
	return "That isn't a support skill."
if ally == c:
	return _apply_support_self(c, st, PlanStep.self_skill(skill), index)
var problem: String = SupportRules.target_problem(c, skill, ally, context.friends_of(c))
if not problem.is_empty():
	return problem
var grid: CombatGrid = context.grid
if not grid.has_combatant(ally):
	return "%s is not on the field." % ally.name
var band: Vector2i = skill.support_reach(c)
var near: int = maxi(1, band.x)
var ally_pos: Vector2i = grid.get_position(ally)
var dist: int = grid.get_distance(st.pos, ally_pos)
if dist < near or dist > band.y:
	var move_problem: String = _move_problem(c, st)
	if not move_problem.is_empty():
		return "%s is out of reach for %s: %s" % [ally.name, skill.display_name, move_problem]
	# Like an Engage approach, the destination is fixed once planned (step.cell).
	var path: Array[Vector2i] = []
	if step.cell != NO_CELL:
		var stored_dist: int = grid.get_distance(step.cell, ally_pos)
		if stored_dist < near or stored_dist > band.y:
			return "The planned approach to %s no longer reaches it." % ally.name
		if blocked_cells(c).has(step.cell):
			return "The planned approach tile %s is taken; re-plan this skill." % str(step.cell)
		path = _route(c, st.pos, {step.cell: true}, blocked_cells(c), false)
	else:
		path = approach_path(c, st.pos, ally, near, band.y)
		if not path.is_empty():
			var dest: Vector2i = st.pos
			for d: Vector2i in path:
				dest += d
			step.cell = dest
	if path.is_empty():
		return "No path to get %s within reach of %s." % [c.name, ally.name]
	if not fits_move(c, st.pos, path):
		return _too_far(c, st.pos, path, "to get in reach")
	var approach_problem: String = _add_move(c, st, path, null, index)
	if not approach_problem.is_empty():
		return approach_problem
var add_problem: String = _add_entry(c, st, skill, index, ally)
if add_problem.is_empty():
	var entry: PlanEntry = st.entries[st.entries.size() - 1]
	entry.est_heal = SupportRules.effective_heal(ally, SupportRules.heal_amount(c, skill))
	entry.est_cleanse = SupportRules.cleansable(ally, skill)
return add_problem
```

### `_apply_item(c: Combatant, st: SimState, step: PlanStep, index: int) -> String`
```logic
var con: Consumable = ItemCommand.pouch_item(c, step.item_id)
var item_name: String = con.name if con != null else str(ContentCatalog.get_entry("items", step.item_id).get("name", step.item_id))
if con == null or items_left(c, step.item_id, index) <= 0:
	return "No %s left in %s's pouch." % [item_name, c.name]
if not ItemCommand.offered(con):
	return "%s can't be used in battle." % item_name
var target: Combatant = step.target if step.target != null else c
var side: String = ItemCommand.side_problem(c, con, target, context.foes_of(c))
if not side.is_empty():
	return side
if not target.is_active() or not context.grid.has_combatant(target):
	return "%s is not on the field." % target.name
if target != c and context.grid.get_distance(st.pos, context.grid.get_position(target)) > con.reach():
	return "Out of reach for %s." % item_name
var problem: String = _add_entry(c, st, ItemCommand.skill_for(con), index, target)
if not problem.is_empty():
	return problem
var entry: PlanEntry = st.entries[st.entries.size() - 1]
entry.item = con
entry.skill_ap = ItemCommand.ITEM_AP
st.ap_left += entry.skill.ap_cost(c) - ItemCommand.ITEM_AP   # _add_entry charged the skill's AP; an item costs ITEM_AP
if st.dry_entry == st.entries.size() - 1 and st.ap_left >= 0:
	st.dry_entry = -1
if con.heal_amount > 0:
	entry.est_heal = mini(con.heal_amount, maxi(0, target.max_hp - target.hp))
if con.damage_amount > 0:
	entry.est_damage = con.damage_amount
	st.est_hp[target] = maxi(0, DictUtil.int_of(st.est_hp, target, target.hp) - con.damage_amount)
return ""
```

### `items_left(c: Combatant, item_id: String, before: int = -1) -> int`
```logic
var n: int = ItemCommand.pouch_count(c, item_id)
var steps: Array[PlanStep] = steps_of(c)
var upto: int = steps.size() if before < 0 else mini(before, steps.size())
for i: int in range(upto):
	if steps[i].mode == PlanStep.Mode.ITEM and steps[i].item_id == item_id:
		n -= 1
return n
```

### `_apply_area(c: Combatant, st: SimState, step: PlanStep, index: int) -> String`
```logic
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
if skill.area == SkillDef.Area.SWEEP and (absi(step.cell.x - st.pos.x) != 1 or absi(step.cell.y - st.pos.y) != 1):
	return "%s is aimed at a diagonal neighbour of %s (picks the quadrant)." % [skill.display_name, c.name]
if skill.area != SkillDef.Area.LINE and skill.area != SkillDef.Area.SWEEP and (dist < band.x or dist > band.y):
	return "That tile is out of reach for %s (distance %d, needs %d-%d)." % [skill.display_name, dist, band.x, band.y]
var cells: Array[Vector2i] = []
for cell: Vector2i in skill.area_cells(st.pos, step.cell):
	if grid.is_valid_coord(cell) and cell != st.pos:
		cells.append(cell)
var problem: String = _add_entry(c, st, skill, index, null, [], 0, cells)
if problem.is_empty():
	var entry: PlanEntry = st.entries[st.entries.size() - 1]
	entry.aim = step.cell
	for foe: Combatant in context.foes_of(c):
		if foe.is_active() and grid.has_combatant(foe) and cells.has(grid.get_position(foe)):
			var hit: int = _estimate_hit(c, st, skill, skill.stance_during(entry.stance_before), foe, st.pos)
			entry.est_damage += hit
			st.est_hp[foe] = DictUtil.int_of(st.est_hp, foe) - hit
	for friend: Combatant in context.friends_of(c):
		if friend != c and friend.is_active() and cells.has(planned_position(friend)):
			entry.hits_allies += 1
return problem
```

### `_add_move(c: Combatant, st: SimState, dirs: Array[Vector2i], chase: Combatant, index: int) -> String`
```logic
var skill: SkillDef = SkillCatalog.lowest(SkillDef.Kind.MOVE)
var goal: Vector2i = st.pos
for d: Vector2i in dirs:
	goal += d
for part: Array in GridPath.chunk(context.grid, st.pos, dirs, c.move_per_block()):
	var chunk: Array[Vector2i] = []
	chunk.assign(part)
	var problem: String = _add_entry(c, st, skill, index, chase, chunk, 1)
	if not problem.is_empty():
		return problem
	for d: Vector2i in chunk:
		st.pos += d
	var entry: PlanEntry = st.entries[st.entries.size() - 1]
	entry.pos_after = st.pos
	entry.goal = goal
st.moved = true
st.positioned = chase == null
return ""
```

### `_attack_from(st: SimState, dirs: Array[Vector2i]) -> Vector2i`
```logic
var at: Vector2i = st.pos
for d: Vector2i in dirs:
	at += d
return at
```

### `_free_start(st: SimState, n: int) -> int`
```logic
for s: int in range(st.next_block, st.total_blocks - n + 2):
	var ok: bool = true
	for b: int in range(s, s + n):
		if st.blocked_blocks.has(b):
			ok = false
			break
	if ok:
		return s
return -1
```

### `_add_entry(c: Combatant, st: SimState, skill: SkillDef, index: int, foe: Combatant = null, dirs: Array[Vector2i] = [], blocks_override: int = 0, cells: Array[Vector2i] = []) -> String`
```logic
var nblocks: int = blocks_override if blocks_override > 0 else skill.blocks
var start: int = _free_start(st, nblocks)
if start < 0:
	return "Not enough free blocks: %s needs %d in a row, %d left." % [skill.display_name, nblocks, st.blocks_left()]
var entry: PlanEntry = PlanEntry.create(skill, start, index)
entry.blocks_override = blocks_override
entry.target = foe
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
	entry.est_damage = _estimate_hit(c, st, skill, skill.stance_during(entry.stance_before), foe, _attack_from(st, dirs))
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
```

### `estimate_damage(c: Combatant, skill: SkillDef, attacker_stance: CombatTypes.StanceType, foe: Combatant, from: Vector2i = NO_CELL, consumed: Array[String] = []) -> int`
```logic
if not skill.deals_damage():
	return 0   # a zero-power attack (intimidate) deals nothing
var exploit_mult: float = DictUtil.to_float(SkillDef.exploit_match(skill.exploit, foe, consumed)["mult"], 1.0)
return DamageCalculator.expected(c, foe, skill, attacker_stance, estimate_cover(c, skill, foe, from), exploit_mult)
```

### `_estimate_hit(c: Combatant, st: SimState, skill: SkillDef, attacker_stance: CombatTypes.StanceType, foe: Combatant, from: Vector2i) -> int`
```logic
var used: Array[String] = _consumed_on(st, foe)
var hit: int = estimate_damage(c, skill, attacker_stance, foe, from, used)
if skill.exploit_consume and skill.deals_damage():
	var status: String = str(SkillDef.exploit_match(skill.exploit, foe, used)["status"])
	if not status.is_empty():
		used.append(status)
		st.consumed[foe] = used
return hit
```

### `estimate_cover(c: Combatant, skill: SkillDef, foe: Combatant, from: Vector2i = NO_CELL) -> float`
```logic
var grid: CombatGrid = context.grid
if grid == null or foe == null or not grid.has_combatant(foe):
	return 0.0
var at: Vector2i = from if from != NO_CELL else grid.get_position(c)
return CoverRules.cover_for(grid, at, grid.get_position(foe), DamageCalculator.expected_is_ranged(c, skill))
```

### `planned_position(c: Combatant) -> Vector2i`
```logic
return _position_after(c, _ally_moves())
```

### `planned_positions(units: Array[Combatant]) -> Dictionary`
```logic
var moves: Dictionary = _ally_moves()
var out: Dictionary = {}
for c: Combatant in units:
	out[c] = _position_after(c, moves)
return out
```

### `_position_after(c: Combatant, moves: Dictionary) -> Vector2i`
```logic
var raw: Variant = moves.get(c, null)
if raw is Dictionary:
	var move: Dictionary = raw
	var to_raw: Variant = move.get("to", NO_CELL)
	return to_raw if to_raw is Vector2i else NO_CELL
return context.grid.get_position(c) if context.grid.has_combatant(c) else NO_CELL
```

### `blocked_cells(c: Combatant) -> Dictionary`
```logic
if _blocked_override is Dictionary:
	var override: Dictionary = _blocked_override
	return override
return _blocked_for(c, _ally_moves())
```

### `_blocked_for(c: Combatant, moves: Dictionary) -> Dictionary`
```logic
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
```

### `_ally_moves() -> Dictionary`
```logic
var moves: Dictionary = {}
var units: Array[Combatant] = []
var order: Dictionary = {}   # unit -> planning order (sort_custom is not stable)
var pinned: Dictionary = {}  # unit -> fixed destination of its opening move
for key: Variant in sequences:
	if key is Combatant:
		var member: Combatant = key
		order[member] = units.size()
		units.append(member)
		var cell: Vector2i = pinned_cell(member)
		if cell != NO_CELL:
			pinned[member] = cell
units.sort_custom(func(a: Combatant, b: Combatant) -> bool:
	return a.speed > b.speed or (a.speed == b.speed and DictUtil.int_of(order, a) < DictUtil.int_of(order, b)))
for unit: Combatant in units:
	var blocked: Dictionary = _blocked_for(unit, moves)
	if not pinned.has(unit):
		var friends: Array[Combatant] = context.friends_of(unit)
		for other: Variant in pinned:
			if other != unit and friends.has(other):
				blocked[pinned[other]] = true
	var move: Dictionary = _first_move(unit, blocked)
	if not move.is_empty():
		moves[unit] = move
return moves
```

### `pinned_cell(c: Combatant) -> Vector2i`
```logic
var steps: Array[PlanStep] = steps_of(c)
if steps.is_empty() or not context.grid.has_combatant(c):
	return NO_CELL
var first: PlanStep = steps[0]
if first.skill != null and first.skill.kind == SkillDef.Kind.DISENGAGE:
	return first.cell   # landing tile (stored once chosen)
match first.mode:
	PlanStep.Mode.MOVE:
		return context.grid.get_position(c) + first.direction * first.tiles
	PlanStep.Mode.GOTO, PlanStep.Mode.ENGAGE, PlanStep.Mode.ALLY:
		return first.cell
return NO_CELL
```

### `_first_move(c: Combatant, blocked: Dictionary) -> Dictionary`
```logic
var steps: Array[PlanStep] = steps_of(c)
if steps.is_empty() or not c.is_active():
	return {}
var mode: PlanStep.Mode = steps[0].mode
var jumps: bool = steps[0].skill != null and steps[0].skill.kind == SkillDef.Kind.DISENGAGE
if not jumps and mode != PlanStep.Mode.MOVE and mode != PlanStep.Mode.GOTO and mode != PlanStep.Mode.WITHDRAW and mode != PlanStep.Mode.ENGAGE and mode != PlanStep.Mode.ALLY:
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
	if e.skill.kind == SkillDef.Kind.MOVE or (e.skill.kind == SkillDef.Kind.DISENGAGE and e.pos_after != e.pos_before):
		if leave < 0:
			leave = e.end_block()
		last = e
	elif e.skill.dash_tiles > 0 and e.pos_after != e.pos_before:
		if leave < 0:
			leave = e.start_block   # a dash happens as its command starts
		last = e
if last == null:
	return {}
return {"to": NO_CELL if st.withdrawn else last.pos_after, "end": last.end_block(), "leave": leave}
```

### `preview(c: Combatant, step: PlanStep) -> SimState`
```logic
var steps: Array[PlanStep] = steps_of(c)
steps.append(step)
sequences[c] = steps
var st: SimState = compile(c)
steps.pop_back()
sequences[c] = steps
return st
```

### `probe(c: Combatant, step: PlanStep) -> PlanOption`
```logic
return _probe(c, step, compile(c))
```

### `_probe(c: Combatant, step: PlanStep, base: SimState) -> PlanOption`
```logic
var option := PlanOption.new()
option.step = step
var st: SimState = preview(c, step)
option.valid = st.error_index < 0
option.reason = st.error_text
if not option.valid:
	return option
var hits_allies: int = 0
var support_note: String = ""
var entered: Array[Vector2i] = []   # tiles the step's route enters (walks, dash, disengage landing)
for i: int in range(base.entries.size(), st.entries.size()):
	var e: PlanEntry = st.entries[i]
	entered.append_array(_entered_cells(e))
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
		if e.target != null and not e.skill.is_area():
			option.cover = estimate_cover(c, e.skill, e.target, e.pos_after)
			var matched: String = str(SkillDef.exploit_match(e.skill.exploit, e.target, _consumed_on(base, e.target))["status"])   # statuses the plan's earlier hits consume do not count
			option.exploits = StatusEffects.display_name(matched) if not matched.is_empty() and e.skill.deals_damage() else ""
	if not e.flow_note().is_empty():
		option.flow = e.flow_note()
	if e.skill.kind == SkillDef.Kind.DISENGAGE:
		option.jump_to = e.pos_after
	if e.skill.kind == SkillDef.Kind.SUPPORT:
		option.heal = e.est_heal
		option.grants = e.skill.grant_status
		option.cleanses = e.est_cleanse
		var who: String = e.target.name if e.target != null else "the target"
		var useful: bool = (e.skill.heals() and e.est_heal > 0) or (e.skill.cleanses() and not e.est_cleanse.is_empty())
		if e.skill.grant_status.is_empty() and not useful and (e.skill.heals() or e.skill.cleanses()):
			if e.skill.heals() and e.skill.cleanses():
				support_note = "Nothing to heal or cleanse: %s is at full HP with no debuffs to remove." % who
			elif e.skill.heals():
				support_note = "Nothing to heal: %s is at full HP." % who
			else:
				support_note = "Nothing to cleanse: %s has no debuffs to remove." % who
var debt: int = st.debt()
if debt > 0:
	var penalty: String = StatusEffects.debt_status(debt)
	option.warning = "AP debt %d: %s next round." % [debt, ("%s and less AP" % StatusEffects.display_name(penalty)) if not penalty.is_empty() else "less AP"]
elif hits_allies > 0:
	option.warning = "Friendly fire: hits %d all%s." % [hits_allies, "y" if hits_allies == 1 else "ies"]
elif not support_note.is_empty():
	option.warning = support_note
elif step.mode == PlanStep.Mode.ENGAGE and DictUtil.int_of(base.est_hp, step.target) <= 0:
	option.warning = "%s is already expected to fall; this is an insurance attack." % step.target.name
if step.mode == PlanStep.Mode.SELF and step.skill.kind == SkillDef.Kind.REST:
	option.ap = -step.skill.recover_ap
option.hazard = HazardRules.preview_chip_cells(context.grid, entered)
return option
```

### `commit(resolver: CombatResolver) -> int`
```logic
var count: int = 0
for key: Variant in sequences:
	if not (key is Combatant):
		continue
	var c: Combatant = key
	for e: PlanEntry in entries_for(c):
		if resolver.schedule_action(c, e.build_command(c, context.grid), e.start_block):
			count += 1
return count
```

