class_name CoverRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Cover Rules (S12T-L rule 4)
# Terrain cover of one hit, passed to DamageCalculator.compute as `cover_ratio` (it adds to the guard term).
# Melee hits never see cover. A ranged hit takes the larger of
#   (a) the cover_guard of the target's own tile, and
#   (b) directional cover: the target's neighbour toward the attacker (both orthogonal neighbours for a
#       diagonal attacker); a neighbour counts only when it blocks movement (wall, tree, deep water, lava).
# An adjacent attacker's own tile is walkable, so it never gives directional cover.
# ==============================================================================

static func cover_for(grid: CombatGrid, attacker_pos: Vector2i, target_pos: Vector2i, is_ranged: bool) -> float:
	if grid == null or not is_ranged or not grid.is_valid_coord(target_pos):
		return 0.0
	var best: float = grid.cover_guard(target_pos)
	for cell: Vector2i in directional_cells(attacker_pos, target_pos):
		if grid.is_valid_coord(cell) and not grid.is_walkable(cell):
			best = maxf(best, grid.cover_guard(cell))
	return best

# The target's neighbours toward the attacker: one for a straight line, two for a diagonal.
static func directional_cells(attacker_pos: Vector2i, target_pos: Vector2i) -> Array[Vector2i]:
	var dx: int = signi(attacker_pos.x - target_pos.x)
	var dy: int = signi(attacker_pos.y - target_pos.y)
	var out: Array[Vector2i] = []
	if dx == 0 and dy == 0:
		return out
	if dx == 0 or dy == 0:
		out.append(target_pos + Vector2i(dx, dy))
	else:
		out.append(target_pos + Vector2i(dx, 0))
		out.append(target_pos + Vector2i(0, dy))
	return out

# Cover of `attacker` hitting `target` where both stand now (0.0 when either is off the grid).
static func cover_between(grid: CombatGrid, attacker: Combatant, target: Combatant, is_ranged: bool) -> float:
	if grid == null or attacker == null or target == null or not grid.has_combatant(attacker) or not grid.has_combatant(target):
		return 0.0
	return cover_for(grid, grid.get_position(attacker), grid.get_position(target), is_ranged)
