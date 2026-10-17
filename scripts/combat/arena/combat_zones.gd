class_name CombatZones
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat Zone Presets
# Hand-authored 12x9 terrain maps (one character per tile, see Terrain glyphs).
# Every zone keeps the arena core (columns 3-8, rows 2-6) and the row-4 lane to
# the east exit walkable, so the standard deployment (party columns 4-5, enemy
# columns 7-8) and exits always work. (The 9x9 maps were widened on 2026-10-17
# by repeating columns 1, 4 and 7, so each keeps its look.)
# Region encounters will pick (or generate) zones by biome later.
#
#   .  floor     ,  grass     :  dirt      o  cobble    s  sand     *  snow
#   ~  shallow   W  deep water m  mud      L  lava      #  wall     T  tree
#   =  bridge    r  rubble
# ==============================================================================

const ZONES: Dictionary = {
	"stone_arena": {
		"name": "Stone Arena", "biome": "Arena",
		"rows": ["............", "............", "............", "............", "............", "............", "............", "............", "............"],
	},
	"forest_clearing": {
		"name": "Forest Clearing", "biome": "Forest",
		"rows": ["TTT,,TT,,TTT", "T,,,,,,,,,,T", ",,,,:,,,,TT,", ",,,,:,,,,,,,", ":::::::::::,", ",,,,,,,,,,,T", "T,,,,,,,,,,,", "TTT,,,,,T,,T", "TTTT,,,TTTTT"],
	},
	"river_crossing": {
		"name": "River Crossing", "biome": "Riverlands",
		"rows": [",,,T,,,,~WW,", ",,,,,,,,~WW,", "T,,,,,,,~WW,", ",,,,,,,,~WW,", "::::::::~==:", ",,,,,,,,~WW,", ",TT,,,,,~WW,", ",,,,,,,,~WWT", ",,,,T,,,~WW,"],
	},
	"ruined_courtyard": {
		"name": "Ruined Courtyard", "biome": "Ruins",
		"rows": ["###o###o####", "#oooooooooo#", "#oo.r...o..#", "ooo........#", "o...........", "#oo.....r..#", "#oo.r...o..#", "#oooooooooo#", "####o###o###"],
	},
	"desert_outpost": {
		"name": "Desert Outpost", "biome": "Desert",
		"rows": ["ssss###sssss", "s##ssssss##s", "sssss::sssss", "sss:::::ssss", "sss:::::::::", "ssss:::ssrr#", "s##sssssssss", "sssssrr#ssss", "###sssssssss"],
	},
	"frozen_pass": {
		"name": "Frozen Pass", "biome": "Mountains",
		"rows": ["###**TT**###", "#**********#", "T***r*******", "*****::*****", "************", "*****::**rr*", "T**********#", "#*******T**#", "###******###"],
	},
	"lava_cavern": {
		"name": "Lava Cavern", "biome": "Volcanic",
		"rows": ["############", "#LLL....LLL#", "#LL......rr#", "#....rr....#", "#...........", "#rr........#", "#LL......LL#", "#LLL.rr.LLL#", "############"],
	},
	"swamp_bog": {
		"name": "Swamp Bog", "biome": "Swamp",
		"rows": ["TmmmT~~~mmmT", "mmm,,,,~~mmm", "~mm,,mm,,,,m", "~~~,,,,,m,,,", "m,,,,,,,,,,,", ",,,m,,,,,mm~", "T,,,,,,m,~~~", "mmm~~,,,,mmT", "TTT~~mmmmTTT"],
	},
}

static func zone_ids() -> Array[String]:
	var ids: Array[String] = []
	for id: String in ZONES:
		ids.append(id)
	return ids

static func zone_name(zone_id: String) -> String:
	var raw: Variant = ZONES.get(zone_id, {})
	var zone: Dictionary = raw if raw is Dictionary else {}
	return str(zone.get("name", zone_id))

# Terrain map for a zone: Vector2i -> Terrain.Type (empty if unknown).
static func terrain_map(zone_id: String) -> Dictionary:
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
