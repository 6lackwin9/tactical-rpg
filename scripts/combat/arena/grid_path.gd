class_name GridPath
extends RefCounted

# ==============================================================================
# Tactical RPG - Grid Path (breadth-first routes on the combat grid)
# Dependency-free so commands (attack follow, approach) and the planner can share
# it without referencing each other.
# ==============================================================================

const DIRS: Array[Vector2i] = [Vector2i(1, 0), Vector2i(0, -1), Vector2i(-1, 0), Vector2i(0, 1)]
const NONE: Vector2i = Vector2i(-1, -1)

# Shortest list of unit steps from `start` to any goal tile. Among equally short
# routes the destination closest to the arena centre wins (then DIRS order), so
# units take central, predictable positions. Empty when already on a goal or when
# no goal is reachable. Exit tiles are only entered as goals.
static func find_path(grid: CombatGrid, start: Vector2i, goals: Dictionary, blocked: Dictionary, goals_are_exits: bool) -> Array[Vector2i]:
	var steps: Array[Vector2i] = []
	if goals.has(start):
		return steps
	var came_from: Dictionary = {start: start}
	var depth: Dictionary = {start: 0}
	var queue: Array[Vector2i] = [start]
	var found: Vector2i = NONE
	var found_depth: int = -1
	var head: int = 0
	while head < queue.size():
		var cur: Vector2i = queue[head]
		head += 1
		var cur_depth: int = DictUtil.int_of(depth, cur)
		if found_depth >= 0 and cur_depth >= found_depth:
			break
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
			depth[nxt] = cur_depth + 1
			if is_goal:
				found_depth = cur_depth + 1
				if found == NONE or grid.get_distance(nxt, CombatGrid.CENTER) < grid.get_distance(found, CombatGrid.CENTER):
					found = nxt
				continue
			queue.append(nxt)
	if found == NONE:
		return steps
	var cell: Vector2i = found
	while cell != start:
		var prev: Vector2i = came_from[cell]
		steps.push_front(cell - prev)
		cell = prev
	return steps
