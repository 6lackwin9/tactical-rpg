---
class: GridPath
extends: RefCounted
description: "Tactical RPG - Grid Path (cost-weighted routes on the combat grid; S12T-L) Dependency-free so commands (attack follow, approach) and the planner can share it without referencing each other. Entering a cell costs its terrain move cost (enter_cost: CombatGrid.move_cost; exit tiles cost 1). A move block has a budget of cost units: chunk() splits a route into blocks, the first tile of a block is always taken (minimum progress, so mud never roots)."
---

# Properties
* `DIRS`: Array[Vector2i] = [Vector2i(1, 0), Vector2i(0, -1), Vector2i(-1, 0), Vector2i(0, 1)] (const)
* `NONE`: Vector2i = Vector2i(-1, -1) (const)
* `_SEQ_BITS`: int = 20 (const)
* `_STEP_BITS`: int = 10 (const)
* `_SEQ_MASK`: int = (1 << _SEQ_BITS) - 1 (const)
* `_STEP_MASK`: int = (1 << _STEP_BITS) - 1 (const)
* `steps`: Array[Vector2i] = []
* `came_from`: Dictionary = {start: start}
* `best`: Dictionary = {start: 0}   # cell -> packed (cost, steps) of the best known route
* `closed`: Dictionary = {}
* `heap`: Array[int] = []
* `cells`: Dictionary = {}          # insertion seq -> cell
* `seq`: int = 0
* `found`: Vector2i = NONE
* `found_key`: int = -1
* `found_pref`: float = 0.0
* `key`: int = _pop(heap)
* `cur`: Vector2i = cells[key & _SEQ_MASK]
* `rank`: int = key >> _SEQ_BITS   # (cost << _STEP_BITS) | steps
* `pref`: float = DictUtil.to_float(prefer.call(cur)) if prefer.is_valid() else 0.0
* `cur_cost`: int = rank >> _STEP_BITS
* `cur_steps`: int = rank & _STEP_MASK
* `nxt`: Vector2i = cur + d
* `is_goal`: bool = goals.has(nxt)
* `enter`: int = enter_cost(grid, nxt)
* `extra`: Variant = penalty.call(nxt)
* `nxt_rank`: int = ((cur_cost + enter) << _STEP_BITS) + cur_steps + 1
* `cell`: Vector2i = found
* `prev`: Vector2i = came_from[cell]
* `out`: Array[Array] = []
* `cap`: int = maxi(1, budget)
* `cell`: Vector2i = start
* `current`: Array[Vector2i] = []
* `spent`: int = 0
* `cost`: int = enter_cost(grid, cell + d)
* `out`: Array[Vector2i] = []
* `parts`: Array[Array] = chunk(grid, start, dirs, budget)
* `part`: Array[Vector2i] = []

# Methods

### `static enter_cost(grid: CombatGrid, cell: Vector2i) -> int`
```logic
return 1 if grid.is_exit_tile(cell) else grid.move_cost(cell)
```

### `static path_cost(grid: CombatGrid, start: Vector2i, dirs: Array[Vector2i]) -> int`
```logic
var total: int = 0
var cell: Vector2i = start
for d: Vector2i in dirs:
	cell += d
	total += enter_cost(grid, cell)
return total
```

### `static blocks_needed(grid: CombatGrid, start: Vector2i, dirs: Array[Vector2i], budget: int) -> int`
```logic
return chunk(grid, start, dirs, budget).size()
```

### `static _push(heap: Array[int], rank: int, seq: int) -> void`
```logic
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
```

### `static _pop(heap: Array[int]) -> int`
```logic
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
```

