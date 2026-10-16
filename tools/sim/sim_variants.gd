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
	for raw_step: Variant in DictUtil.array_of(entry.get("skill_path", [])):
		var step: Dictionary = DictUtil.dict_of(raw_step)
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
		for y: int in range(CombatGrid.GRID_SIZE):
			for x: int in range(CombatGrid.GRID_SIZE):
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
