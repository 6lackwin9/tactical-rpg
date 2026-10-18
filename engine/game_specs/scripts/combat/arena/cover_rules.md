---
class: CoverRules
extends: RefCounted
description: "Tactical RPG - Cover Rules (S12T-L rule 4) Terrain cover of one hit, passed to DamageCalculator.compute as `cover_ratio` (it adds to the guard term). Melee hits never see cover. A ranged hit takes the larger of (a) the cover_guard of the target's own tile, and (b) directional cover: the target's neighbour toward the attacker (both orthogonal neighbours for a diagonal attacker); a neighbour counts only when it blocks movement (wall, tree, deep water, lava). An adjacent attacker's own tile is walkable, so it never gives directional cover."
---

# Properties
* `dx`: int = signi(attacker_pos.x - target_pos.x)
* `dy`: int = signi(attacker_pos.y - target_pos.y)
* `out`: Array[Vector2i] = []

# Methods

### `static cover_for(grid: CombatGrid, attacker_pos: Vector2i, target_pos: Vector2i, is_ranged: bool) -> float`
```logic
if grid == null or not is_ranged or not grid.is_valid_coord(target_pos):
	return 0.0
var best: float = grid.cover_guard(target_pos)
for cell: Vector2i in directional_cells(attacker_pos, target_pos):
	if grid.is_valid_coord(cell) and not grid.is_walkable(cell):
		best = maxf(best, grid.cover_guard(cell))
return best
```

### `static cover_between(grid: CombatGrid, attacker: Combatant, target: Combatant, is_ranged: bool) -> float`
```logic
if grid == null or attacker == null or target == null or not grid.has_combatant(attacker) or not grid.has_combatant(target):
	return 0.0
return cover_for(grid, grid.get_position(attacker), grid.get_position(target), is_ranged)
```

