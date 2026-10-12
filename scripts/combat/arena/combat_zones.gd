class_name CombatZones
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat Zone Presets
# Hand-authored 9x9 terrain maps (one character per tile, see Terrain glyphs).
# Every zone keeps the arena core (columns 2-6, rows 2-6) and the row-4 lane to
# the east exit walkable, so center-ring deployment and exits always work.
# Region encounters will pick (or generate) zones by biome later.
#
#   .  floor     ,  grass     :  dirt      o  cobble    s  sand     *  snow
#   ~  shallow   W  deep water m  mud      L  lava      #  wall     T  tree
#   =  bridge    r  rubble
# ==============================================================================

const ZONES: Dictionary = {
	"stone_arena": {
		"name": "Stone Arena", "biome": "Arena",
		"rows": [".........", ".........", ".........", ".........", ".........", ".........", ".........", ".........", "........."],
	},
	"forest_clearing": {
		"name": "Forest Clearing", "biome": "Forest",
		"rows": ["TT,,T,,TT", "T,,,,,,,T", ",,,:,,,T,", ",,,:,,,,,", "::::::::,", ",,,,,,,,T", "T,,,,,,,,", "TT,,,,T,T", "TTT,,TTTT"],
	},
	"river_crossing": {
		"name": "River Crossing", "biome": "Riverlands",
		"rows": [",,T,,,~W,", ",,,,,,~W,", "T,,,,,~W,", ",,,,,,~W,", "::::::~=:", ",,,,,,~W,", ",T,,,,~W,", ",,,,,,~WT", ",,,T,,~W,"],
	},
	"ruined_courtyard": {
		"name": "Ruined Courtyard", "biome": "Ruins",
		"rows": ["##o##o###", "#ooooooo#", "#o.r..o.#", "oo......#", "o........", "#o....r.#", "#o.r..o.#", "#ooooooo#", "###o##o##"],
	},
	"desert_outpost": {
		"name": "Desert Outpost", "biome": "Desert",
		"rows": ["sss##ssss", "s#sssss#s", "ssss:ssss", "ss::::sss", "ss:::::::", "sss::ssr#", "s#sssssss", "ssssr#sss", "##sssssss"],
	},
	"frozen_pass": {
		"name": "Frozen Pass", "biome": "Mountains",
		"rows": ["##**T**##", "#*******#", "T**r*****", "****:****", "*********", "****:**r*", "T*******#", "#*****T*#", "##*****##"],
	},
	"lava_cavern": {
		"name": "Lava Cavern", "biome": "Volcanic",
		"rows": ["#########", "#LL...LL#", "#L.....r#", "#...r...#", "#........", "#r......#", "#L.....L#", "#LL.r.LL#", "#########"],
	},
	"swamp_bog": {
		"name": "Swamp Bog", "biome": "Swamp",
		"rows": ["TmmT~~mmT", "mm,,,~~mm", "~m,,m,,,m", "~~,,,,m,,", "m,,,,,,,,", ",,m,,,,m~", "T,,,,m,~~", "mm~~,,,mT", "TT~~mmmTT"],
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
	for y: int in range(mini(rows.size(), CombatGrid.GRID_SIZE)):
		var row: String = str(rows[y])
		for x: int in range(mini(row.length(), CombatGrid.GRID_SIZE)):
			result[Vector2i(x, y)] = Terrain.from_glyph(row[x])
	return result
