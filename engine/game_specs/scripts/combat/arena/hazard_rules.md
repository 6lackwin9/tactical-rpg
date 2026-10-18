---
class: HazardRules
extends: RefCounted
description: "Tactical RPG - Hazard Rules (S12T-L rule 6) Hazard tiles (Terrain.hazard: status, potency, magnitude, rounds, on) roll their status through StatusEffects.try_inflict (resistance applies; the status starts next round like every queued status): enter      a walk, dash or disengage that enters hazard tiles rolls each status once per unit per command round_end  after the round-end ticks, every active unit standing on a hazard tile is rolled both       both of the above The source name is \"terrain\" (sim metrics credit the damage to no unit)."
---

# Properties
* `SOURCE`: String = "terrain" (const)
* `out`: Array[Dictionary] = []
* `seen`: Dictionary = {}
* `h`: Dictionary = grid.hazard_at(cell)
* `row`: Dictionary = h.duplicate()
* `lines`: Array[String] = []
* `line`: String = _roll(grid, unit, h, rng)
* `lines`: Array[String] = []
* `cell`: Vector2i = grid.get_position(unit)
* `h`: Dictionary = grid.hazard_at(cell)
* `line`: String = _roll(grid, unit, h, rng)

# Methods

### `static triggers_on_enter(h: Dictionary) -> bool`
```logic
var on: String = str(h.get("on", ""))
return on == TerrainFields.ON_ENTER or on == TerrainFields.ON_BOTH
```

### `static triggers_at_round_end(h: Dictionary) -> bool`
```logic
var on: String = str(h.get("on", ""))
return on == TerrainFields.ON_ROUND_END or on == TerrainFields.ON_BOTH
```

### `static _roll(grid: CombatGrid, unit: Combatant, h: Dictionary, rng: RandomNumberGenerator) -> String`
```logic
var status: String = str(h["status"])
var hit: bool = StatusEffects.try_inflict(unit, status, DictUtil.to_int(h["potency"], 100), rng, DictUtil.to_int(h["rounds"], 1), DictUtil.to_int(h["magnitude"]), SOURCE)
if not hit:
	return ""
var cell: Vector2i = h.get("cell", Vector2i.ZERO)
return "[Hazard] %s is %s by %s" % [unit.name, StatusEffects.display_name(status), Terrain.display_name(grid.get_terrain(cell))]
```

### `static preview_chip(grid: CombatGrid, start: Vector2i, dirs: Array[Vector2i]) -> String`
```logic
var cells: Array[Vector2i] = []
var cell: Vector2i = start
for d: Vector2i in dirs:
	cell += d
	cells.append(cell)
return preview_chip_cells(grid, cells)
```

### `static preview_chip_cells(grid: CombatGrid, cells: Array[Vector2i]) -> String`
```logic
var parts: Array[String] = []
if grid == null:
	return ""
for h: Dictionary in entered(grid, cells):
	var c: Vector2i = h["cell"]
	parts.append("Hazard: %s (%s)" % [Terrain.display_name(grid.get_terrain(c)), StatusEffects.display_name(str(h["status"]))])
return ", ".join(parts)
```

