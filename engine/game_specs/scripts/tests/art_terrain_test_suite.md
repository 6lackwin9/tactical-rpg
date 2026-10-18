---
class: ArtTerrainTestSuite
extends: RefCounted
description: "TEST 6: GENERATED ART, TERRAIN & COMBAT ZONES"
---

# Methods

### `static run(_tree_parent: Node = null) -> bool`
```logic
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
# Art key comes from the unit's catalog `art` field (CLN-15), not a class fallback.
sides_ok = sides_ok and enc.class_of(enc.unit("goblin_shaman")) == str(ContentCatalog.get_entry("units", "goblin_shaman").get("art", ""))

# 4. Terrain glyph round-trip and walkability rules
var terrain_ok: bool = true
for t: int in Terrain.INFO:
	var tt: Terrain.Type = t as Terrain.Type
	if Terrain.from_glyph(Terrain.glyph(tt)) != tt:
		terrain_ok = false
terrain_ok = terrain_ok and not Terrain.is_walkable(Terrain.Type.WALL) and not Terrain.is_walkable(Terrain.Type.DEEP_WATER) and Terrain.is_walkable(Terrain.Type.BRIDGE) and Terrain.is_walkable(Terrain.Type.MUD)

# 5. Every zone is a valid 12x9 map with a walkable core and exit lane
var zones_ok: bool = CombatZones.zone_ids().size() >= 8
for zone_id: String in CombatZones.zone_ids():
	var map: Dictionary = CombatZones.terrain_map(zone_id)
	if map.size() != CombatGrid.GRID_W * CombatGrid.GRID_H:
		zones_ok = false
	for x: int in range(3, 9):
		for y: int in range(2, 7):
			var core_raw: Variant = map.get(Vector2i(x, y), Terrain.Type.FLOOR)
			if core_raw is Terrain.Type and not Terrain.is_walkable(DictUtil.to_int(core_raw) as Terrain.Type):
				zones_ok = false
	for x: int in range(5, CombatGrid.GRID_W):
		var lane_raw: Variant = map.get(Vector2i(x, 4), Terrain.Type.FLOOR)
		if lane_raw is Terrain.Type and not Terrain.is_walkable(DictUtil.to_int(lane_raw) as Terrain.Type):
			zones_ok = false

# 6. Grid enforces terrain: no placing/moving onto blocked tiles; deployment skips them
var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(11, 4)])
grid.set_terrain(Vector2i(CombatGrid.PARTY_FRONT_X, 4), Terrain.Type.WALL)
grid.set_terrain(Vector2i(2, 3), Terrain.Type.LAVA)
var hero := Combatant.new("Tester", 50, 10, 1)
var allies_g: Array[Combatant] = [hero]
var none: Array[Combatant] = []
grid.deploy_combatants(allies_g, none)
var deployed_off_wall: bool = grid.get_position(hero) != Vector2i(CombatGrid.PARTY_FRONT_X, 4) and grid.is_walkable(grid.get_position(hero))
var walker := Combatant.new("Walker", 50, 10, 1)
var placed_lava: bool = grid.place_combatant(walker, Vector2i(2, 3))
grid.place_combatant(walker, Vector2i(2, 4))
var moved_into_lava: bool = grid.move_combatant(walker, Vector2i(0, -1))
var grid_ok: bool = deployed_off_wall and not placed_lava and not moved_into_lava and grid.get_position(walker) == Vector2i(2, 4)

# 6b. Standard deployment (GRID12): party left, enemies right; melee front column, ranged back column,
# rows from DEPLOY_ROWS; a walled-off column spills to the next one; halves split at the middle.
var dgrid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(11, 4)])
var p_melee := Combatant.new("PMelee", 50, 10, 1)
var p_ranged := Combatant.new("PRanged", 50, 10, 3)
var p_melee2 := Combatant.new("PMelee2", 50, 10, 1)
var e_melee := Combatant.new("EMelee", 50, 10, 1)
var e_ranged := Combatant.new("ERanged", 50, 10, 3)
var d_allies: Array[Combatant] = [p_melee, p_ranged, p_melee2]
var d_enemies: Array[Combatant] = [e_melee, e_ranged]
dgrid.deploy_combatants(d_allies, d_enemies)
var r0: int = CombatGrid.DEPLOY_ROWS[0]
var r1: int = CombatGrid.DEPLOY_ROWS[1]
var deploy_ok: bool = dgrid.get_position(p_melee) == Vector2i(CombatGrid.PARTY_FRONT_X, r0) and dgrid.get_position(p_ranged) == Vector2i(CombatGrid.PARTY_BACK_X, r0)
deploy_ok = deploy_ok and dgrid.get_position(p_melee2) == Vector2i(CombatGrid.PARTY_FRONT_X, r1)
deploy_ok = deploy_ok and dgrid.get_position(e_melee) == Vector2i(CombatGrid.ENEMY_FRONT_X, r0) and dgrid.get_position(e_ranged) == Vector2i(CombatGrid.ENEMY_BACK_X, r0)
var wgrid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(11, 4)])
for y: int in range(CombatGrid.GRID_H):
	wgrid.set_terrain(Vector2i(CombatGrid.PARTY_FRONT_X, y), Terrain.Type.WALL)
var w_melee := Combatant.new("WMelee", 50, 10, 1)
var w_allies: Array[Combatant] = [w_melee]
wgrid.deploy_combatants(w_allies, none)
deploy_ok = deploy_ok and wgrid.get_position(w_melee) == Vector2i(CombatGrid.PARTY_BACK_X, r0)
deploy_ok = deploy_ok and CombatGrid.is_party_half(Vector2i((CombatGrid.GRID_W >> 1) - 1, 0)) and not CombatGrid.is_party_half(Vector2i(CombatGrid.GRID_W >> 1, 0))
deploy_ok = deploy_ok and CombatGrid.centre_distance(Vector2i(5, 4)) == CombatGrid.centre_distance(Vector2i(6, 4)) and CombatGrid.centre_distance(Vector2i(5, 4)) < CombatGrid.centre_distance(Vector2i(4, 4))
for enc_id: String in ["goblin_skirmish", "veterans_drill"]:
	var dsession := CombatSession.new(CatalogEncounter.new(enc_id))
	dsession.start()
	for c: Combatant in dsession.allies:
		deploy_ok = deploy_ok and dsession.grid.has_combatant(c) and CombatGrid.is_party_half(dsession.grid.get_position(c))
	for c: Combatant in dsession.enemies:
		deploy_ok = deploy_ok and dsession.grid.has_combatant(c) and not CombatGrid.is_party_half(dsession.grid.get_position(c))
	dsession.dispose()

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
var tiles_ok: bool = TileArt.overlay("exit") != null and TileArt.overlay("poi") != null
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

# 10. Item icons go through the registry (S14-L1 rules 4-5): icon_id must be an item art key; no category branch
var plain_sword: Item = ItemCatalog.create_iron_shortsword()
plain_sword.icon_id = ""
var unknown_icon: Item = ItemCatalog.create_iron_shortsword()
unknown_icon.icon_id = "qa_no_such_icon"
var staff: Item = ItemCatalog.create_runed_staff()
var bad_row: Dictionary = ContentCatalog.get_entry("items", "wpn_shortsword").duplicate(true)
bad_row["icon_id"] = "qa_no_such_icon"
var type_row: Dictionary = ContentCatalog.get_entry("items", "wpn_shortsword").duplicate(true)
type_row["icon_id"] = "dagger"
var registry_ok: bool = (
	ItemArt.icon_key(plain_sword) == "sword"
	and ItemArt.icon_key(unknown_icon) == "sword"
	and ItemArt.icon(staff) == AssetRegistry.texture("item", "runed_staff", "icon", int(staff.rarity))
	and ContentCatalog.validate_entry("items", bad_row, "wpn_shortsword").has("Icon 'qa_no_such_icon' isn't a known item icon key.")
	and ContentCatalog.validate_entry("items", type_row, "wpn_shortsword").is_empty()
	and ItemFormModel.icon_options() == [""] + AssetRegistry.keys("item")
)

print("\n=======================================================")
print("=== ART, TERRAIN & ZONE ASSERTIONS                  ===")
print("=======================================================")
print("  [Art] Team-colored class tokens (blue/red/yellow + rim shapes): %s" % ("PASS" if tokens_ok else "FAIL"))
print("  [Art] Class portraits for every class and team: %s" % ("PASS" if portraits_ok else "FAIL"))
print("  [Art] Encounter art follows sides (allies friendly, enemies hostile): %s" % ("PASS" if sides_ok else "FAIL"))
print("  [Terrain] Glyph round-trip & walkability rules: %s" % ("PASS" if terrain_ok else "FAIL"))
print("  [Terrain] All zones valid 12x9 with walkable core & exit lane: %s" % ("PASS" if zones_ok else "FAIL"))
print("  [Terrain] Grid blocks placement/movement on blocked tiles; deploy skips them: %s" % ("PASS" if grid_ok else "FAIL"))
print("  [Terrain] Standard deployment: party left, enemies right, melee front, ranged back, blocked column spills: %s" % ("PASS" if deploy_ok else "FAIL"))
print("  [Terrain] Goblin Ambush (with its wave) deploys and concludes on every zone: %s" % ("PASS" if all_zones_play else "FAIL"))
print("  [Art] Terrain tiles (all variants) & overlays rasterize: %s" % ("PASS" if tiles_ok else "FAIL"))
print("  [Art] Item icons by gear type + bespoke rare weapon icons: %s" % ("PASS" if icons_ok else "FAIL"))
print("  [Art] Item icons through the registry; icon_id validated against item art keys: %s" % ("PASS" if registry_ok else "FAIL"))

return tokens_ok and portraits_ok and sides_ok and terrain_ok and zones_ok and grid_ok and deploy_ok and all_zones_play and tiles_ok and icons_ok and registry_ok
```

