---
class: CombatZones
extends: RefCounted
description: "Tactical RPG - Combat Zone Presets Hand-authored 12x9 terrain maps (one character per tile, see Terrain glyphs). Every zone keeps the arena core (columns 3-8, rows 2-6) and the row-4 lane to the east exit walkable, so the standard deployment (party columns 4-5, enemy columns 7-8) and exits always work. (The 9x9 maps were widened on 2026-10-17 by repeating columns 1, 4 and 7, so each keeps its look.) Region encounters will pick (or generate) zones by biome later. .  floor     ,  grass     :  dirt      o  cobble    s  sand     *  snow ~  shallow   W  deep water m  mud      L  lava      #  wall     T  tree =  bridge    r  rubble    f  burning ground (hazard)   p  toxic bog (hazard)   i  ice (hazard) Hazard tiles (S12T-L) sit outside the core and the exit lane: lava_cavern f, swamp_bog p, frozen_pass i."
---

# Properties
* `ZONES`: Dictionary = { (const)
* `ids`: Array[String] = []

# Methods

### `static zone_name(zone_id: String) -> String`
```logic
var raw: Variant = ZONES.get(zone_id, {})
var zone: Dictionary = raw if raw is Dictionary else {}
return str(zone.get("name", zone_id))
```

### `static terrain_map(zone_id: String) -> Dictionary`
```logic
var result: Dictionary = {}
var raw: Variant = ZONES.get(zone_id, null)
if not (raw is Dictionary):
	return result
var zone: Dictionary = raw
var rows_raw: Variant = zone.get("rows", [])
var rows: Array = rows_raw if rows_raw is Array else []
for y: int in range(mini(rows.size(), CombatGrid.GRID_H)):
	var row: String = str(rows[y])
	for x: int in range(mini(row.length(), CombatGrid.GRID_W)):
		result[Vector2i(x, y)] = Terrain.from_glyph(row[x])
return result
```

