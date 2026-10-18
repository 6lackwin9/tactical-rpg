extends RefCounted

# Balance sim variants (BAL2_SIM.md section 3): the "newest" loadout, spend variants and the
# start-distance placement. Static helpers, loaded with preload (no class_name). Never edits the
# shipped catalog: callers write the results into the sandbox fork.

# 3.1: owned class-path families of a `level` unit of `class_id` (unreduced path levels): families at
# path level 1 first (path order), then the rest by path level descending (ties in path order), area
# families after every non-area family; the first StatRules.MAX_CLASS_SLOTS.
static func newest_loadout(class_id: String, level: int) -> Array[String]:
	var entry: Dictionary = ContentCatalog.get_entry("classes", class_id)
	var first_level: Dictionary = {}   # family -> lowest path level
	var order: Array[String] = []      # families in path order
	for step: Dictionary in ContentFactory.class_archetype(entry).path_steps():
		var skill_id: String = str(step.get("skill", ""))
		var step_level: int = DictUtil.to_int(step.get("level", 1), 1)
		if skill_id.is_empty() or SkillCatalog.is_always_active(skill_id) or step_level > level:
			continue
		var fam: String = SkillCatalog.family_of(skill_id)
		if not first_level.has(fam):
			first_level[fam] = step_level
			order.append(fam)
	var ranked: Array[String] = order.duplicate()
	ranked.sort_custom(func(a: String, b: String) -> bool:
		var area_a: bool = is_area_family(a)
		var area_b: bool = is_area_family(b)
		if area_a != area_b:
			return area_b
		var la: int = DictUtil.int_of(first_level, a)
		var lb: int = DictUtil.int_of(first_level, b)
		var ka: int = 0 if la == 1 else 1
		var kb: int = 0 if lb == 1 else 1
		if ka != kb:
			return ka < kb
		if la != lb:
			return la > lb
		return order.find(a) < order.find(b)
	)
	return ranked.slice(0, StatRules.MAX_CLASS_SLOTS)

# A family is an area family when its base row (tier one) has an area shape.
static func is_area_family(family: String) -> bool:
	var base: SkillDef = SkillCatalog.tier_one(family)
	return base != null and base.area != SkillDef.Area.NONE

# 3.2: the unit `spend` dict of variant `variant` for `class_id` at `level` ({} = class weights).
static func spend_for(variant: String, class_id: String, level: int) -> Dictionary:
	var pool: int = StatRules.spend_pool(level)
	var primary: String = primary_stat(class_id)
	match variant:
		"cons":
			return {"CONS": pool}
		"primary":
			return {primary: pool}
		"mixed":
			var half: int = floori(float(pool) / 2.0)
			if primary == "CONS":
				return {"CONS": pool}
			return {primary: half, "CONS": pool - half}
	return {}

# The stat of the largest class growth (ties: the first key of the class `growth` dict).
static func primary_stat(class_id: String) -> String:
	var growth: Dictionary = DictUtil.dict_of(ContentCatalog.get_entry("classes", class_id).get("growth", {}))
	var best: String = ""
	var best_value: int = -1
	for key: Variant in growth:
		var v: int = DictUtil.to_int(growth[key])
		if v > best_value:
			best_value = v
			best = str(key)
	return best

# 3.3: moves every deployed right-side unit to the walkable, free, non-exit tile at grid distance
# `dist` from the nearest left-side unit, the one closest to its current tile (ties: lowest y, then x).
# Returns the number of units that found no such tile (they stay).
static func apply_distance(grid: CombatGrid, left: Array[Combatant], right: Array[Combatant], dist: int) -> int:
	var anchors: Array[Vector2i] = []
	for c: Combatant in left:
		if grid.has_combatant(c):
			anchors.append(grid.get_position(c))
	var not_moved: int = 0
	for c: Combatant in right:
		if not grid.has_combatant(c):
			continue
		var here: Vector2i = grid.get_position(c)
		var best: Vector2i = Vector2i(-1, -1)
		var best_d: int = 1 << 30
		for y: int in range(CombatGrid.GRID_H):
			for x: int in range(CombatGrid.GRID_W):
				var tile := Vector2i(x, y)
				if not grid.is_walkable(tile) or grid.is_exit_tile(tile):
					continue
				if tile != here and grid.is_occupied(tile):
					continue
				if _nearest(grid, anchors, tile) != dist:
					continue
				var d: int = grid.get_distance(here, tile)
				if d < best_d:   # row-major scan: the first hit at a distance is the lowest y, then x
					best_d = d
					best = tile
		if best.x < 0:
			not_moved += 1
			continue
		if best != here:
			grid.remove_combatant(c)
			grid.place_combatant(c, best)
	return not_moved

static func _nearest(grid: CombatGrid, anchors: Array[Vector2i], tile: Vector2i) -> int:
	var out: int = 1 << 30
	for a: Vector2i in anchors:
		out = mini(out, grid.get_distance(a, tile))
	return out

# ------------------------------------------------------------------------------
# BAL3 3.1: seeded start jitter (default on; `jitter=off` keeps the standard deployment)
# ------------------------------------------------------------------------------

const JITTER_SALT: int = 7919   # keeps the jitter RNG stream apart from the fight seed

# The jitter RNG of fight seed `seed_value` (same seed, same layout).
static func jitter_rng(seed_value: int) -> RandomNumberGenerator:
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_value * JITTER_SALT + 1
	return rng

# A seeded Fisher-Yates copy of the party list (the deployment fills rows in list order).
static func shuffled(ids: Array[String], rng: RandomNumberGenerator) -> Array[String]:
	var out: Array[String] = ids.duplicate()
	for i: int in range(out.size() - 1, 0, -1):
		var j: int = rng.randi_range(0, i)
		var t: String = out[i]
		out[i] = out[j]
		out[j] = t
	return out

# Tiles unit `c` may jitter to: its own tile, then the four neighbours (grid distance 1) that are
# walkable, free, not exits and on the same half of the arena as its current tile.
static func jitter_options(grid: CombatGrid, c: Combatant) -> Array[Vector2i]:
	var here: Vector2i = grid.get_position(c)
	var out: Array[Vector2i] = [here]
	for off: Vector2i in [Vector2i(0, -1), Vector2i(-1, 0), Vector2i(1, 0), Vector2i(0, 1)]:
		var tile: Vector2i = here + off
		if not grid.is_walkable(tile) or grid.is_exit_tile(tile) or grid.is_occupied(tile):
			continue
		if CombatGrid.is_party_half(tile) != CombatGrid.is_party_half(here):
			continue
		out.append(tile)
	return out

# Shifts every deployed unit (left side first, list order) to a seeded pick of its jitter options.
# Returns the number of units that moved.
static func jitter_start(grid: CombatGrid, left: Array[Combatant], right: Array[Combatant], rng: RandomNumberGenerator) -> int:
	var moved: int = 0
	for c: Combatant in left + right:
		if not grid.has_combatant(c):
			continue
		var options: Array[Vector2i] = jitter_options(grid, c)
		var pick: Vector2i = options[rng.randi_range(0, options.size() - 1)]
		if pick != grid.get_position(c):
			grid.remove_combatant(c)
			grid.place_combatant(c, pick)
			moved += 1
	return moved

# ------------------------------------------------------------------------------
# BAL3 3.2: clumped enemy block (aoe_swarm)
# ------------------------------------------------------------------------------

# The tiles of a `size` (columns x rows) block whose front column is `front_x` and whose middle row
# is `mid_y`, front column first, each column top to bottom (adjacent tiles: one tile apart).
static func block_tiles(front_x: int, mid_y: int, size: Vector2i) -> Array[Vector2i]:
	var out: Array[Vector2i] = []
	var top: int = mid_y - floori(float(size.y - 1) / 2.0)
	for dx: int in range(size.x):
		for dy: int in range(size.y):
			out.append(Vector2i(front_x + dx, top + dy))
	return out

# Moves the first size.x * size.y deployed right-side units into one block on the right half: the
# middle row is the grid middle + `row_shift` (clamped inside the grid), the front column the one
# nearest the left side whose tiles are all walkable, non-exit, free of other units and at least
# `min_dist` (0 = no minimum) from the nearest left-side unit. False (nobody moves) when no column fits.
static func apply_block(grid: CombatGrid, left: Array[Combatant], right: Array[Combatant], size: Vector2i, min_dist: int, row_shift: int) -> bool:
	var block: Array[Combatant] = []
	for c: Combatant in right:
		if grid.has_combatant(c) and block.size() < size.x * size.y:
			block.append(c)
	if block.is_empty():
		return false
	var anchors: Array[Vector2i] = []
	for c: Combatant in left:
		if grid.has_combatant(c):
			anchors.append(grid.get_position(c))
	var half_rows: int = floori(float(size.y - 1) / 2.0)
	var mid_y: int = clampi(floori(float(CombatGrid.GRID_H) / 2.0) + row_shift, half_rows, CombatGrid.GRID_H - size.y + half_rows)
	var chosen: Array[Vector2i] = []
	for x: int in range(0, CombatGrid.GRID_W - size.x + 1):
		var tiles: Array[Vector2i] = block_tiles(x, mid_y, size)
		var fits: bool = true
		for tile: Vector2i in tiles:
			var holder: Combatant = grid.get_combatant_at(tile)
			if CombatGrid.is_party_half(tile) or not grid.is_walkable(tile) or grid.is_exit_tile(tile) \
					or (holder != null and not block.has(holder)) or (min_dist > 0 and _nearest(grid, anchors, tile) < min_dist):
				fits = false
				break
		if fits:
			chosen = tiles
			break
	if chosen.is_empty():
		return false
	for c: Combatant in block:
		grid.remove_combatant(c)
	for i: int in range(block.size()):
		grid.place_combatant(block[i], chosen[i])
	return true
