class_name TacticalAI
extends RefCounted

# ==============================================================================
# Tactical RPG - Tactical AI (round planner for computer-controlled units)
# Plans one unit's whole 10-block round from the state at round start, tracking
# only its own predicted movement and the damage it expects to deal. Targets
# may move, so plans can fizzle: that is the planning-phase gamble. Used for
# enemies each round and for the player's "Auto-plan".
#   1. Flee: at low HP, run for an exit tile.
#   2. Pick a target (aggressors prefer the nearest, others the weakest).
#   3. Role pattern: approach / attack / retreat / guard / focus.
# ==============================================================================

const DIRS: Array[Vector2i] = [Vector2i(1, 0), Vector2i(0, -1), Vector2i(-1, 0), Vector2i(0, 1)]
const NO_CELL: Vector2i = Vector2i(-1, -1)

static func plan(actor: Combatant, foes: Array[Combatant], friends: Array[Combatant], grid: CombatGrid, profile: AiProfile) -> Array[PlanEntry]:
	var out: Array[PlanEntry] = []
	if actor == null or grid == null or not actor.is_active() or not grid.has_combatant(actor):
		return out

	var pos: Vector2i = grid.get_position(actor)
	var blocked: Dictionary = {}
	var others: Array[Combatant] = []
	others.append_array(foes)
	others.append_array(friends)
	for c: Combatant in others:
		if c != actor and grid.has_combatant(c):
			blocked[grid.get_position(c)] = true
	var est_hp: Dictionary = {}
	for f: Combatant in foes:
		est_hp[f] = f.hp

	var block: int = 1
	var ap: int = CombatPlan.ROUND_AP
	var focused: bool = false
	var guarded: bool = false
	var rng: int = actor.attack_range
	var fleeing: bool = profile.flee_ratio > 0.0 and float(actor.hp) <= float(actor.max_hp) * profile.flee_ratio and not grid.designated_exits.is_empty()

	while block <= CombatPlan.TOTAL_BLOCKS and ap > 0:
		if fleeing:
			var exits: Dictionary = {}
			for exit_tile: Vector2i in grid.designated_exits:
				exits[exit_tile] = true
			var flee_path: Array[Vector2i] = _path(grid, pos, exits, blocked, true)
			if flee_path.is_empty():
				break
			var flee_entry: PlanEntry = _step_entry(flee_path, block, ap, 0)
			out.append(flee_entry)
			block += flee_entry.blocks()
			ap -= flee_entry.ap_cost()
			pos += _step_total(flee_entry)
			continue

		if profile.role == AiProfile.Role.SUPPORT and not focused and ap >= PlanEntry.ap_of(PlanEntry.Kind.FOCUS) and block + 1 <= CombatPlan.TOTAL_BLOCKS:
			focused = true
			out.append(PlanEntry.create(PlanEntry.Kind.FOCUS, block))
			block += 2
			ap -= PlanEntry.ap_of(PlanEntry.Kind.FOCUS)
			continue

		var target: Combatant = _pick_target(pos, foes, est_hp, grid, profile)
		if target == null:
			break
		var target_pos: Vector2i = grid.get_position(target)
		var dist: int = grid.get_distance(pos, target_pos)
		var low_hp: bool = float(actor.hp) <= float(actor.max_hp) * profile.defend_below

		# Guard once when hurt and a foe is within reach.
		if low_hp and not guarded and dist <= rng + 1 and block + 1 <= CombatPlan.TOTAL_BLOCKS and ap >= PlanEntry.ap_of(PlanEntry.Kind.DEFEND):
			guarded = true
			out.append(PlanEntry.create(PlanEntry.Kind.DEFEND, block))
			block += 2
			ap -= PlanEntry.ap_of(PlanEntry.Kind.DEFEND)
			continue

		var want_retreat: bool = false
		var want_approach: bool = dist > rng
		match profile.role:
			AiProfile.Role.SNIPER, AiProfile.Role.SUPPORT:
				want_retreat = dist < rng
			AiProfile.Role.SKIRMISHER:
				want_retreat = low_hp and dist <= 1

		if want_retreat:
			var back: Vector2i = _retreat_step(grid, pos, foes, blocked, rng, target_pos)
			if back != Vector2i.ZERO:
				out.append(PlanEntry.create(PlanEntry.Kind.MOVE, block, back))
				block += 1
				ap -= PlanEntry.ap_of(PlanEntry.Kind.MOVE)
				pos += back
				continue
			want_approach = false  # cornered: stand and fight

		if want_approach:
			var goals: Dictionary = {}
			for dx: int in range(-rng, rng + 1):
				for dy: int in range(-rng, rng + 1):
					var cell: Vector2i = target_pos + Vector2i(dx, dy)
					if absi(dx) + absi(dy) <= rng and grid.is_valid_coord(cell) and grid.is_walkable(cell) and not grid.is_exit_tile(cell) and not blocked.has(cell):
						goals[cell] = true
			var path: Array[Vector2i] = _path(grid, pos, goals, blocked, false)
			if path.is_empty():
				# No route: brace instead.
				if block + 1 <= CombatPlan.TOTAL_BLOCKS and ap >= PlanEntry.ap_of(PlanEntry.Kind.DEFEND):
					out.append(PlanEntry.create(PlanEntry.Kind.DEFEND, block))
					block += 2
					ap -= PlanEntry.ap_of(PlanEntry.Kind.DEFEND)
					continue
				break
			var approach_entry: PlanEntry = _step_entry(path, block, ap, PlanEntry.ap_of(PlanEntry.Kind.ATTACK))
			out.append(approach_entry)
			block += approach_entry.blocks()
			ap -= approach_entry.ap_cost()
			pos += _step_total(approach_entry)
			continue

		# In range: strike.
		if block + 1 <= CombatPlan.TOTAL_BLOCKS and ap >= PlanEntry.ap_of(PlanEntry.Kind.ATTACK):
			out.append(PlanEntry.create(PlanEntry.Kind.ATTACK, block, Vector2i.ZERO, target, profile.power))
			est_hp[target] = int(est_hp.get(target, 0)) - profile.power
			block += 2
			ap -= PlanEntry.ap_of(PlanEntry.Kind.ATTACK)
		else:
			break

	return out

# ------------------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------------------

# Aggressors go for the nearest foe; everyone else prefers the weakest.
static func _pick_target(pos: Vector2i, foes: Array[Combatant], est_hp: Dictionary, grid: CombatGrid, profile: AiProfile) -> Combatant:
	var best: Combatant = null
	var best_score: float = INF
	for f: Combatant in foes:
		if not f.is_active() or not grid.has_combatant(f) or int(est_hp.get(f, 0)) <= 0:
			continue
		var d: int = grid.get_distance(pos, grid.get_position(f))
		var score: float
		if profile.role == AiProfile.Role.AGGRESSOR:
			score = float(d) * 100.0 + float(est_hp.get(f, 0))
		else:
			score = float(est_hp.get(f, 0)) + float(d) * 4.0
		if score < best_score:
			best_score = score
			best = f
	return best

# Shortest list of unit steps from `start` to any goal tile. Empty when already
# on a goal or when no goal is reachable. Exit tiles are only entered as goals.
static func _path(grid: CombatGrid, start: Vector2i, goals: Dictionary, blocked: Dictionary, goals_are_exits: bool) -> Array[Vector2i]:
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

# A Sprint (two tiles, one block) when the path allows it and, after paying for it
# and walking the rest of the path, eserve_ap is still left (e.g. for an attack).
static func _step_entry(path: Array[Vector2i], block: int, ap: int, reserve_ap: int) -> PlanEntry:
	var sprint_cost: int = PlanEntry.ap_of(PlanEntry.Kind.SPRINT)
	if path.size() >= 2 and path[0] == path[1] and ap - sprint_cost - (path.size() - 2) >= reserve_ap:
		return PlanEntry.create(PlanEntry.Kind.SPRINT, block, path[0])
	return PlanEntry.create(PlanEntry.Kind.MOVE, block, path[0])

static func _step_total(entry: PlanEntry) -> Vector2i:
	return entry.direction * (2 if entry.kind == PlanEntry.Kind.SPRINT else 1)

# One step that gains distance from the nearest foe but stays within weapon range
# of the current target (so a ranged unit can keep shooting).
static func _retreat_step(grid: CombatGrid, pos: Vector2i, foes: Array[Combatant], blocked: Dictionary, rng: int, target_pos: Vector2i) -> Vector2i:
	var best: Vector2i = Vector2i.ZERO
	var best_gain: int = 0
	var cur_near: int = _nearest_foe(grid, pos, foes)
	for d: Vector2i in DIRS:
		var nxt: Vector2i = pos + d
		if not grid.is_valid_coord(nxt) or not grid.is_walkable(nxt) or grid.is_exit_tile(nxt) or blocked.has(nxt):
			continue
		if grid.get_distance(nxt, target_pos) > rng:
			continue
		var gain: int = _nearest_foe(grid, nxt, foes) - cur_near
		if gain > best_gain:
			best_gain = gain
			best = d
	return best

static func _nearest_foe(grid: CombatGrid, pos: Vector2i, foes: Array[Combatant]) -> int:
	var best: int = 99
	for f: Combatant in foes:
		if f.is_active() and grid.has_combatant(f):
			best = mini(best, grid.get_distance(pos, grid.get_position(f)))
	return best
