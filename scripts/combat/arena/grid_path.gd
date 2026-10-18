class_name GridPath
extends RefCounted

# ==============================================================================
# Tactical RPG - Grid Path (cost-weighted routes on the combat grid; S12T-L)
# Dependency-free so commands (attack follow, approach) and the planner can share
# it without referencing each other.
# Entering a cell costs its terrain move cost (enter_cost: CombatGrid.move_cost; exit tiles cost 1).
# A move block has a budget of cost units: chunk() splits a route into blocks, the
# first tile of a block is always taken (minimum progress, so mud never roots).
# ==============================================================================

const DIRS: Array[Vector2i] = [Vector2i(1, 0), Vector2i(0, -1), Vector2i(-1, 0), Vector2i(0, 1)]
const NONE: Vector2i = Vector2i(-1, -1)

# Priority key packing (cost, steps, insertion order): the grid is 12x9, so every part fits its field.
const _SEQ_BITS: int = 20
const _STEP_BITS: int = 10
const _SEQ_MASK: int = (1 << _SEQ_BITS) - 1
const _STEP_MASK: int = (1 << _STEP_BITS) - 1

# Cheapest list of unit steps from `start` to any goal tile (uniform-cost search). Among routes of equal cost the
# fewer steps win, then the destination closest to the arena centre, then DIRS order (on an all-floor map this
# is exactly the old breadth-first result). Empty when already on a goal or when no goal is reachable. Exit
# tiles are only entered as goals. `penalty` (optional, AI only): Callable(cell: Vector2i) -> int added to the
# cost of entering `cell` (players always see true costs). `prefer` (optional, AI only): Callable(cell: Vector2i)
# -> float; among equally ranked destinations the higher value wins before the centre rule.
static func find_path(grid: CombatGrid, start: Vector2i, goals: Dictionary, blocked: Dictionary, goals_are_exits: bool, penalty: Callable = Callable(), prefer: Callable = Callable()) -> Array[Vector2i]:
	var steps: Array[Vector2i] = []
	if goals.has(start):
		return steps
	var came_from: Dictionary = {start: start}
	var best: Dictionary = {start: 0}   # cell -> packed (cost, steps) of the best known route
	var closed: Dictionary = {}
	var heap: Array[int] = []
	var cells: Dictionary = {}          # insertion seq -> cell
	var seq: int = 0
	_push(heap, 0, seq)
	cells[seq] = start
	seq += 1
	var found: Vector2i = NONE
	var found_key: int = -1
	var found_pref: float = 0.0
	while not heap.is_empty():
		var key: int = _pop(heap)
		var cur: Vector2i = cells[key & _SEQ_MASK]
		var rank: int = key >> _SEQ_BITS   # (cost << _STEP_BITS) | steps
		if closed.has(cur) or DictUtil.int_of(best, cur) != rank:
			continue
		if found_key >= 0 and rank > found_key:
			break
		closed[cur] = true
		if goals.has(cur) and cur != start:
			var pref: float = DictUtil.to_float(prefer.call(cur)) if prefer.is_valid() else 0.0
			if found == NONE or pref > found_pref or (is_equal_approx(pref, found_pref) and CombatGrid.centre_distance(cur) < CombatGrid.centre_distance(found)):
				found = cur
				found_key = rank
				found_pref = pref
			continue
		var cur_cost: int = rank >> _STEP_BITS
		var cur_steps: int = rank & _STEP_MASK
		for d: Vector2i in DIRS:
			var nxt: Vector2i = cur + d
			if closed.has(nxt) or not grid.is_valid_coord(nxt):
				continue
			var is_goal: bool = goals.has(nxt)
			if is_goal:
				if not goals_are_exits and (blocked.has(nxt) or not grid.is_walkable(nxt)):
					continue
			elif blocked.has(nxt) or not grid.is_walkable(nxt) or grid.is_exit_tile(nxt):
				continue
			var enter: int = enter_cost(grid, nxt)
			if penalty.is_valid():
				var extra: Variant = penalty.call(nxt)
				enter += maxi(0, DictUtil.to_int(extra))
			var nxt_rank: int = ((cur_cost + enter) << _STEP_BITS) + cur_steps + 1
			if best.has(nxt) and DictUtil.int_of(best, nxt) <= nxt_rank:
				continue
			best[nxt] = nxt_rank
			came_from[nxt] = cur
			_push(heap, nxt_rank, seq)
			cells[seq] = nxt
			seq += 1
	if found == NONE:
		return steps
	var cell: Vector2i = found
	while cell != start:
		var prev: Vector2i = came_from[cell]
		steps.push_front(cell - prev)
		cell = prev
	return steps

# Cost of entering `cell` on a route: an exit tile costs 1 (spec S12T-L rule 1: "exit goals cost 1"; a unit
# stepping onto an exit leaves the arena, so the tile's terrain never slows it), any other tile its move cost.
# find_path, path_cost and chunk all price tiles through this, so a withdraw route fits the same blocks it
# was found with.
static func enter_cost(grid: CombatGrid, cell: Vector2i) -> int:
	return 1 if grid.is_exit_tile(cell) else grid.move_cost(cell)

# Summed entering cost of walking `dirs` from `start`.
static func path_cost(grid: CombatGrid, start: Vector2i, dirs: Array[Vector2i]) -> int:
	var total: int = 0
	var cell: Vector2i = start
	for d: Vector2i in dirs:
		cell += d
		total += enter_cost(grid, cell)
	return total

# `dirs` split into move blocks of at most `budget` cost units each: a block takes tiles while the summed cost
# stays within the budget, and always takes its first tile (minimum progress). Budget < 1 counts as 1.
static func chunk(grid: CombatGrid, start: Vector2i, dirs: Array[Vector2i], budget: int) -> Array[Array]:
	var out: Array[Array] = []
	var cap: int = maxi(1, budget)
	var cell: Vector2i = start
	var current: Array[Vector2i] = []
	var spent: int = 0
	for d: Vector2i in dirs:
		var cost: int = enter_cost(grid, cell + d)
		if not current.is_empty() and spent + cost > cap:
			out.append(current)
			current = []
			spent = 0
		current.append(d)
		spent += cost
		cell += d
	if not current.is_empty():
		out.append(current)
	return out

# Number of move blocks `dirs` needs at `budget` cost units per block.
static func blocks_needed(grid: CombatGrid, start: Vector2i, dirs: Array[Vector2i], budget: int) -> int:
	return chunk(grid, start, dirs, budget).size()

# The leading part of `dirs` that fits in `blocks` move blocks of `budget` cost units.
static func fit_blocks(grid: CombatGrid, start: Vector2i, dirs: Array[Vector2i], budget: int, blocks: int) -> Array[Vector2i]:
	var out: Array[Vector2i] = []
	var parts: Array[Array] = chunk(grid, start, dirs, budget)
	for i: int in range(mini(blocks, parts.size())):
		var part: Array[Vector2i] = []
		part.assign(parts[i])
		out.append_array(part)
	return out

# Binary min-heap of packed keys: (rank << _SEQ_BITS) | seq, so equal ranks pop in insertion order.
static func _push(heap: Array[int], rank: int, seq: int) -> void:
	heap.append((rank << _SEQ_BITS) | seq)
	var i: int = heap.size() - 1
	while i > 0:
		var parent: int = (i - 1) >> 1
		if heap[parent] <= heap[i]:
			break
		var tmp: int = heap[parent]
		heap[parent] = heap[i]
		heap[i] = tmp
		i = parent

static func _pop(heap: Array[int]) -> int:
	var top: int = heap[0]
	var last: int = heap.pop_back()
	if heap.is_empty():
		return top
	heap[0] = last
	var i: int = 0
	var n: int = heap.size()
	while true:
		var l: int = 2 * i + 1
		var r: int = l + 1
		var m: int = i
		if l < n and heap[l] < heap[m]:
			m = l
		if r < n and heap[r] < heap[m]:
			m = r
		if m == i:
			break
		var tmp: int = heap[m]
		heap[m] = heap[i]
		heap[i] = tmp
		i = m
	return top
