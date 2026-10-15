class_name ArtTerrainTestSuite
extends RefCounted

# ==============================================================================
# TEST 6: GENERATED ART, TERRAIN & COMBAT ZONES
# ==============================================================================

static func run(_tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING ART, TERRAIN & COMBAT ZONE TEST        ===")
	print("=======================================================")

	# 1. Tokens: every class x team rasterizes; team decides color, hostiles get a spiked rim
	var tokens_ok: bool = true
	for class_key: String in UnitArt.CLASS_KEYS:
		for team: int in [UnitArt.Team.FRIENDLY, UnitArt.Team.HOSTILE, UnitArt.Team.NEUTRAL]:
			var tex: Texture2D = UnitArt.token(class_key, team as UnitArt.Team)
			if tex == null or tex.get_width() != 128:
				tokens_ok = false
	var friendly_svg: String = UnitArt.token_svg("warrior", UnitArt.Team.FRIENDLY)
	var hostile_svg: String = UnitArt.token_svg("warrior", UnitArt.Team.HOSTILE)
	var neutral_svg: String = UnitArt.token_svg("dummy", UnitArt.Team.NEUTRAL)
	tokens_ok = tokens_ok and "#60a5fa" in friendly_svg and "#f87171" in hostile_svg and "#facc15" in neutral_svg
	tokens_ok = tokens_ok and hostile_svg.count("<path d=\"M") > friendly_svg.count("<path d=\"M") and "stroke-dasharray" in neutral_svg

	# 2. Portraits: every class x team rasterizes; goblin classes get goblin skin
	var portraits_ok: bool = true
	for class_key: String in UnitArt.CLASS_KEYS:
		for team: int in [UnitArt.Team.FRIENDLY, UnitArt.Team.HOSTILE]:
			if UnitArt.portrait(class_key, team as UnitArt.Team) == null:
				portraits_ok = false
	portraits_ok = portraits_ok and "#7cb342" in UnitArt.portrait_svg("shaman", UnitArt.Team.HOSTILE) and not ("#7cb342" in UnitArt.portrait_svg("warrior", UnitArt.Team.FRIENDLY))

	# 3. Encounter art follows sides: allies friendly (blue), enemies hostile (red)
	var enc := CatalogEncounter.new("goblin_ambush")
	enc.build()
	var sides_ok: bool = true
	for c: Combatant in enc.allies:
		sides_ok = sides_ok and enc.team_of(c) == UnitArt.Team.FRIENDLY and enc.get_token_texture(c) != null and enc.get_sprite_texture(c) != null
	for c: Combatant in enc.enemies:
		sides_ok = sides_ok and enc.team_of(c) == UnitArt.Team.HOSTILE and enc.get_token_texture(c) != null
	sides_ok = sides_ok and enc.class_of(enc.unit("goblin_shaman")) == "priest"

	# 4. Terrain glyph round-trip and walkability rules
	var terrain_ok: bool = true
	for t: int in Terrain.INFO:
		var tt: Terrain.Type = t as Terrain.Type
		if Terrain.from_glyph(Terrain.glyph(tt)) != tt:
			terrain_ok = false
	terrain_ok = terrain_ok and not Terrain.is_walkable(Terrain.Type.WALL) and not Terrain.is_walkable(Terrain.Type.DEEP_WATER) and Terrain.is_walkable(Terrain.Type.BRIDGE) and Terrain.is_walkable(Terrain.Type.MUD)

	# 5. Every zone is a valid 9x9 map with a walkable core and exit lane
	var zones_ok: bool = CombatZones.zone_ids().size() >= 8
	for zone_id: String in CombatZones.zone_ids():
		var map: Dictionary = CombatZones.terrain_map(zone_id)
		if map.size() != 81:
			zones_ok = false
		for x: int in range(2, 7):
			for y: int in range(2, 7):
				var core_raw: Variant = map.get(Vector2i(x, y), Terrain.Type.FLOOR)
				if core_raw is Terrain.Type and not Terrain.is_walkable(DictUtil.to_int(core_raw) as Terrain.Type):
					zones_ok = false
		for x: int in range(4, 9):
			var lane_raw: Variant = map.get(Vector2i(x, 4), Terrain.Type.FLOOR)
			if lane_raw is Terrain.Type and not Terrain.is_walkable(DictUtil.to_int(lane_raw) as Terrain.Type):
				zones_ok = false

	# 6. Grid enforces terrain: no placing/moving onto blocked tiles; deployment skips them
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	grid.set_terrain(Vector2i(4, 5), Terrain.Type.WALL)
	grid.set_terrain(Vector2i(2, 3), Terrain.Type.LAVA)
	var hero := Combatant.new("Tester", 50, 10, 1)
	var allies_g: Array[Combatant] = [hero]
	var none: Array[Combatant] = []
	grid.deploy_combatants(allies_g, none)
	var deployed_off_wall: bool = grid.get_position(hero) != Vector2i(4, 5) and grid.is_walkable(grid.get_position(hero))
	var walker := Combatant.new("Walker", 50, 10, 1)
	var placed_lava: bool = grid.place_combatant(walker, Vector2i(2, 3))
	grid.place_combatant(walker, Vector2i(2, 4))
	var moved_into_lava: bool = grid.move_combatant(walker, Vector2i(0, -1))
	var grid_ok: bool = deployed_off_wall and not placed_lava and not moved_into_lava and grid.get_position(walker) == Vector2i(2, 4)

	# 7. The Goblin Ambush (wave included) deploys and plays to a conclusion on every zone
	var all_zones_play: bool = true
	for zone_id: String in CombatZones.zone_ids():
		var zone_enc := CatalogEncounter.new("goblin_ambush")
		zone_enc.zone_id = zone_id
		var session := CombatSession.new(zone_enc)
		session.start()
		for round_num: int in range(zone_enc.max_rounds):
			session.plan_apply_preset_all("balanced")
			session.confirm_plan()
			TestHelpers.resolve_until_replay(session)
			if session.combat_over or not session.can_advance_round():
				break
			session.advance_round()
		var concluded: bool = session.combat_over or session.current_round == zone_enc.max_rounds
		for c: Combatant in zone_enc.allies + zone_enc.enemies:
			if c.is_active() and not session.grid.has_combatant(c):
				concluded = false   # every unit (wave included) must have found a tile
		session.dispose()
		if not concluded:
			all_zones_play = false
			print("    zone failed to conclude: %s" % zone_id)

	# 8. Tiles & overlays rasterize for every terrain variant
	var tiles_ok: bool = TileArt.overlay("exit") != null and TileArt.overlay("center") != null
	for t: int in Terrain.INFO:
		for v: int in range(TileArt.VARIANTS):
			if TileArt.tile(t as Terrain.Type, v) == null:
				tiles_ok = false

	# 9. Item icons: every catalog item gets one; types map correctly; rare weapons use bespoke art
	var stash := Inventory.new()
	ItemCatalog.populate_party_starter_stash(stash)
	var icons_ok: bool = true
	for item: Item in stash.items:
		if ItemArt.icon(item) == null:
			icons_ok = false
	icons_ok = icons_ok and (
		ItemArt.icon_key(ItemCatalog.create_iron_claymore()) == "greatsword"
		and ItemArt.icon_key(ItemCatalog.create_iron_shortsword()) == "shortsword"
		and ItemArt.icon_key(ItemCatalog.create_buckler()) == "shield"
		and ItemArt.icon_key(ItemCatalog.create_steel_plate_cuirass()) == "plate"
		and ItemArt.icon_key(ItemCatalog.create_healing_draught()) == "potion"
		and ItemArt.icon_key(ItemCatalog.create_titan_warhammer()) == "titan_warhammer"
		and ItemArt.icon_key(ItemCatalog.create_runed_staff()) == "runed_staff"
	)

	print("\n=======================================================")
	print("=== ART, TERRAIN & ZONE ASSERTIONS                  ===")
	print("=======================================================")
	print("  [Art] Team-colored class tokens (blue/red/yellow + rim shapes): %s" % ("PASS" if tokens_ok else "FAIL"))
	print("  [Art] Class portraits for every class and team: %s" % ("PASS" if portraits_ok else "FAIL"))
	print("  [Art] Encounter art follows sides (allies friendly, enemies hostile): %s" % ("PASS" if sides_ok else "FAIL"))
	print("  [Terrain] Glyph round-trip & walkability rules: %s" % ("PASS" if terrain_ok else "FAIL"))
	print("  [Terrain] All zones valid 9x9 with walkable core & exit lane: %s" % ("PASS" if zones_ok else "FAIL"))
	print("  [Terrain] Grid blocks placement/movement on blocked tiles; deploy skips them: %s" % ("PASS" if grid_ok else "FAIL"))
	print("  [Terrain] Goblin Ambush (with its wave) deploys and concludes on every zone: %s" % ("PASS" if all_zones_play else "FAIL"))
	print("  [Art] Terrain tiles (all variants) & overlays rasterize: %s" % ("PASS" if tiles_ok else "FAIL"))
	print("  [Art] Item icons by gear type + bespoke rare weapon icons: %s" % ("PASS" if icons_ok else "FAIL"))

	return tokens_ok and portraits_ok and sides_ok and terrain_ok and zones_ok and grid_ok and all_zones_play and tiles_ok and icons_ok
