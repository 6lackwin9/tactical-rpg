---
class: TerrainRulesTestSuite
extends: RefCounted
description: "Tactical RPG - Terrain Rules Test Suite (S12T-L) Part B (rules): cost-weighted paths (pinned to the old breadth-first result on uniform maps), move chunking with minimum progress, dash budget, cover (tile and directional), hazards on enter / at round end, disengage landing, preview chips, zone hazard tiles. Part C (AI): hazard route penalty for AI drafts only, cover preference among equally ranked cells, covered targets estimated lower. Part A (schema): terrain ids and ordinals, the catalog kind `terrain` (fields, validator messages, code defaults, missing rows), Terrain.describe() from the catalog, and the DamageCalculator cover input with GUARD_CAP. Fixtures live in a sandbox fork of the catalog; expectations derive from the rules (registry ranges, defaults)."
---

# Properties
* `TAG`: String = "TerrainRules" (const)
* `MUD_COST`: int = 2 (const)
* `SNOW_COST`: int = 3 (const)
* `RUBBLE_COVER`: float = 0.25 (const)
* `WALL_COVER`: float = 0.5 (const)
* `TREE_COVER`: float = 0.3 (const)
* `out`: Array[Vector2i] = []
* `out`: Array[int] = []
* `steps`: Array[Vector2i] = []
* `came_from`: Dictionary = {start: start}
* `depth`: Dictionary = {start: 0}
* `queue`: Array[Vector2i] = [start]
* `found`: Vector2i = GridPath.NONE
* `found_depth`: int = -1
* `head`: int = 0
* `cur`: Vector2i = queue[head]
* `cur_depth`: int = DictUtil.int_of(depth, cur)
* `nxt`: Vector2i = cur + d
* `is_goal`: bool = goals.has(nxt)
* `cell`: Vector2i = found
* `prev`: Vector2i = came_from[cell]
* `out`: Array[Vector2i] = []
* `cell`: Vector2i = start
* `dirs`: Array[Vector2i] = []

# Methods

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING TERRAIN RULES TEST (S12T-L)             ===")
print("=======================================================")
var ok: bool = true
ok = _test_ids() and ok
ok = _test_validator() and ok
ok = _test_catalog_rows() and ok
ok = _test_cover_input() and ok
# Part B (rules) runs on a fresh fork with test-only terrain rows.
ok = _test_fixture_rows() and ok
ok = _test_move_chunking() and ok
ok = _test_slow_exit() and ok
ok = _test_path_cost() and ok
ok = _test_cover_rules() and ok
ok = _test_hazards() and ok
ok = _test_disengage_and_dash() and ok
ok = _test_zones() and ok
# Part C (AI, rule 8) on the same fixture rows.
ok = _test_ai_hazard_penalty() and ok
ok = _test_ai_cover_preference() and ok
TestHelpers.cleanup(SANDBOX)
return ok
```

### `static _test_ids() -> bool`
```logic
var checks: Dictionary = {}
var round_trip: bool = true
var unique: Dictionary = {}
for t: int in Terrain.INFO:
	var tt: Terrain.Type = t as Terrain.Type
	var id: String = Terrain.id_of(tt)
	round_trip = round_trip and not id.is_empty() and Terrain.has_id(id) and Terrain.from_id(id) == tt
	unique[id] = true
checks["every type has an id and from_id(id_of(t)) == t"] = round_trip
checks["ids are unique, one per type"] = unique.size() == Terrain.Type.size() and Terrain.ids().size() == Terrain.Type.size()
checks["existing ordinals unchanged (FLOOR 0 ... RUBBLE 13)"] = Terrain.Type.FLOOR == 0 and Terrain.Type.MUD == 8 and Terrain.Type.WALL == 10 and Terrain.Type.RUBBLE == 13
checks["three hazard types appended after RUBBLE"] = Terrain.Type.BURNING_GROUND == Terrain.Type.RUBBLE + 1 and Terrain.Type.TOXIC_BOG == Terrain.Type.RUBBLE + 2 and Terrain.Type.ICE == Terrain.Type.RUBBLE + 3
checks["the id is the enum name in lower snake case"] = Terrain.id_of(Terrain.Type.SHALLOW_WATER) == "shallow_water" and Terrain.id_of(Terrain.Type.BURNING_GROUND) == "burning_ground"
checks["new types: glyphs f / p / i, walkable"] = Terrain.from_glyph("f") == Terrain.Type.BURNING_GROUND and Terrain.from_glyph("p") == Terrain.Type.TOXIC_BOG and Terrain.from_glyph("i") == Terrain.Type.ICE and Terrain.is_walkable(Terrain.Type.BURNING_GROUND) and Terrain.is_walkable(Terrain.Type.TOXIC_BOG) and Terrain.is_walkable(Terrain.Type.ICE)
var glyphs: Dictionary = {}
for t: int in Terrain.INFO:
	glyphs[Terrain.glyph(t as Terrain.Type)] = true
checks["glyphs stay unique"] = glyphs.size() == Terrain.INFO.size()
checks["INFO holds code facts only (no cover / move_cost / hazard)"] = not Terrain.info(Terrain.Type.RUBBLE).has("cover") and not Terrain.info(Terrain.Type.MUD).has("move_cost") and not Terrain.info(Terrain.Type.WALL).has("hazard")
checks["unknown id: has_id false"] = not Terrain.has_id("lava_pit")
return TestHelpers.report(TAG, "terrain ids and ordinals", checks)
```

### `static _row(id: String, extra: Dictionary = {}) -> Dictionary`
```logic
var raw: Dictionary = {"id": id}
raw.merge(extra, true)
return ContentSchema.normalize("terrain", raw)
```

### `static _only(errors: Array[String], message: String) -> bool`
```logic
return errors.size() == 1 and errors[0] == message
```

### `static _test_validator() -> bool`
```logic
var checks: Dictionary = {}
var sand: String = Terrain.id_of(Terrain.Type.SAND)
var wall: String = Terrain.id_of(Terrain.Type.WALL)
var burning: String = Terrain.id_of(Terrain.Type.BURNING_GROUND)
var burn: String = StatusEffects.BURN
checks["every default row is valid"] = Terrain.ids().all(func(id: String) -> bool: return TerrainValidator.entry_errors(Terrain.default_row(Terrain.from_id(id))).is_empty())
checks["unknown id"] = _only(_errors("lava_pit"), "Unknown terrain type 'lava_pit'.")
checks["move cost above 3 on a walkable type"] = _only(_errors(sand, {"move_cost": TerrainValidator.MOVE_COST_MAX + 1}), "Move cost must be 1-3.")
checks["move cost below 1 on a walkable type"] = _only(_errors(sand, {"move_cost": TerrainValidator.MOVE_COST_MIN - 1}), "Move cost must be 1-3.")
checks["blocked types ignore move cost"] = _errors(wall, {"move_cost": TerrainValidator.MOVE_COST_MAX + 1}).is_empty()
checks["cover above 0.5"] = _only(_errors(sand, {"cover_guard": TerrainValidator.COVER_MAX + 0.1}), "Cover must be 0.0-0.5.")
checks["negative cover"] = _only(_errors(sand, {"cover_guard": -0.1}), "Cover must be 0.0-0.5.")
var buff: String = StatusEffects.buffs()[0]
checks["a buff is not a hazard"] = _only(_errors(sand, {"hazard_status": buff}), "Hazard status '%s' is not a debuff." % buff)
var mag_min: int = StatusRegistry.magnitude_min(burn)
var mag_max: int = StatusRegistry.magnitude_max(burn)
checks["a WALL row with a hazard is refused"] = _errors(wall, {"hazard_status": burn, "hazard_magnitude": mag_min}).has("Only walkable terrain can carry a hazard.")
checks["potency 0"] = _only(_errors(burning, {"hazard_status": burn, "hazard_magnitude": mag_min, "hazard_potency": TerrainValidator.POTENCY_MIN - 1}), "Hazard potency must be 1-100.")
checks["potency above 100"] = _only(_errors(burning, {"hazard_status": burn, "hazard_magnitude": mag_min, "hazard_potency": TerrainValidator.POTENCY_MAX + 1}), "Hazard potency must be 1-100.")
checks["burn magnitude above its registry range"] = _only(_errors(burning, {"hazard_status": burn, "hazard_magnitude": mag_max + 1}), "%s needs a magnitude %d-%d" % [burn, mag_min, mag_max])
checks["burn magnitude 0 (below its range)"] = _only(_errors(burning, {"hazard_status": burn, "hazard_magnitude": 0}), "%s needs a magnitude %d-%d" % [burn, mag_min, mag_max])
var plain: String = ""
for id: String in StatusEffects.debuffs():
	if not StatusEffects.has_magnitude(id):
		plain = id
		break
checks["a magnitude on a status without one"] = not plain.is_empty() and _only(_errors(burning, {"hazard_status": plain, "hazard_magnitude": 3}), "magnitude only applies to %s" % ", ".join(StatusRegistry.magnitude_ids()))
checks["a magnitude without a status"] = _only(_errors(burning, {"hazard_magnitude": 3}), "magnitude only applies to %s" % ", ".join(StatusRegistry.magnitude_ids()))
checks["rounds 0"] = _only(_errors(burning, {"hazard_rounds": TerrainValidator.ROUNDS_MIN - 1}), "Hazard rounds must be 1-5.")
checks["rounds above 5"] = _only(_errors(burning, {"hazard_rounds": TerrainValidator.ROUNDS_MAX + 1}), "Hazard rounds must be 1-5.")
checks["unknown trigger"] = _only(_errors(burning, {"hazard_on": "always"}), "Hazard trigger must be enter, round_end or both.")
checks["user origin refused"] = _only(_errors(burning, {"origin": ContentSchema.ORIGIN_USER}), "Terrain rows are system assets (origin system).")
checks["walkable row with burn, magnitude 5 is valid (5 inside the registry range)"] = mag_min <= 5 and mag_max >= 5 and _errors(burning, {"hazard_status": burn, "hazard_magnitude": 5}).is_empty()
for on: String in TerrainFields.TRIGGERS:
	checks["trigger '%s' is valid" % on] = _errors(burning, {"hazard_status": burn, "hazard_magnitude": mag_min, "hazard_on": on}).is_empty()
checks["field defaults: cost 1, cover 0, no hazard, potency 100, rounds 1, trigger both"] = _row(sand).get("move_cost") == 1 and is_zero_approx(DictUtil.to_float(_row(sand).get("cover_guard"))) and str(_row(sand).get("hazard_status")) == "" and _row(sand).get("hazard_potency") == 100 and _row(sand).get("hazard_rounds") == 1 and str(_row(sand).get("hazard_on")) == TerrainFields.ON_BOTH
return TestHelpers.report(TAG, "terrain validator messages", checks)
```

### `static _test_catalog_rows() -> bool`
```logic
var checks: Dictionary = {}
checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
# Independent of the shipped rows: the ICE row is removed in the fork (SAFE DELETE scope), so ICE is a new id
# below and the catalog has a missing row once terrain.json is saved.
var ice_id: String = Terrain.id_of(Terrain.Type.ICE)
if ContentCatalog.has("terrain", ice_id):
	ContentCatalog.begin_system_edit(true)
	checks["the sandbox drops its ICE row"] = ContentCatalog.remove("terrain", ice_id).is_empty()
	ContentCatalog.end_system_edit()
checks["terrain is a catalog kind"] = ContentSchema.KINDS.has("terrain") and not ContentSchema.fields("terrain").is_empty()
var sand: Terrain.Type = Terrain.Type.SAND
var grass: Terrain.Type = Terrain.Type.GRASS
checks["code defaults equal the old INFO move costs (sand 2, grass 1)"] = Terrain.default_move_cost(sand) == 2 and Terrain.default_move_cost(grass) == 1 and Terrain.default_move_cost(Terrain.Type.MUD) == 2 and Terrain.default_move_cost(Terrain.Type.FLOOR) == 1
if ContentCatalog.entries("terrain").is_empty():
	checks["no catalog rows: sand costs 2, grass 1"] = Terrain.move_cost(sand) == 2 and Terrain.move_cost(grass) == 1
	checks["no catalog rows: no cover, no hazard"] = is_zero_approx(Terrain.cover_guard(Terrain.Type.RUBBLE)) and Terrain.hazard(Terrain.Type.BURNING_GROUND).is_empty()
if not ContentCatalog.terrain_file_exists():
	var warned: Dictionary = ContentCatalog.warnings_all()
	checks["no terrain.json: warnings_all lists every missing row"] = ContentFactory._strings(warned.get("terrain:missing", [])).size() == TerrainValidator.missing_ids(ContentCatalog.entries("terrain")).size()
	checks["no terrain.json: validate_all has no terrain:missing"] = not ContentCatalog.validate_all().has("terrain:missing")
var sand_id: String = Terrain.id_of(sand)
var rubble_id: String = Terrain.id_of(Terrain.Type.RUBBLE)
var burning_id: String = Terrain.id_of(Terrain.Type.BURNING_GROUND)
var wall_id: String = Terrain.id_of(Terrain.Type.WALL)
var burn: String = StatusEffects.BURN
var mag: int = StatusRegistry.magnitude_min(burn)
ContentCatalog.begin_system_edit()
checks["a sand row with cost 3 is stored"] = ContentCatalog.upsert("terrain", {"id": sand_id, "move_cost": 3}).is_empty()
checks["a rubble row with cover 0.25 is stored"] = ContentCatalog.upsert("terrain", {"id": rubble_id, "move_cost": 2, "cover_guard": 0.25}).is_empty()
checks["a burning-ground row with burn is stored"] = ContentCatalog.upsert("terrain", {"id": burning_id, "hazard_status": burn, "hazard_magnitude": mag, "hazard_on": TerrainFields.ON_ENTER}).is_empty()
checks["a WALL row with a hazard is refused by upsert"] = ContentCatalog.upsert("terrain", {"id": wall_id, "hazard_status": burn, "hazard_magnitude": mag}).has("Only walkable terrain can carry a hazard.")
checks["an unknown terrain id is refused by upsert"] = ContentCatalog.upsert("terrain", {"id": "lava_pit"}).has("Unknown terrain type 'lava_pit'.")
checks["a WALL row is stored with move cost 1 (ignored)"] = ContentCatalog.upsert("terrain", {"id": wall_id, "move_cost": 3, "cover_guard": 0.5}).is_empty() and ContentCatalog.get_entry("terrain", wall_id).get("move_cost") == 1
ContentCatalog.end_system_edit()
checks["a new row outside the system scope is refused (user origin)"] = not ContentCatalog.has("terrain", ice_id) and ContentCatalog.upsert("terrain", {"id": ice_id}).has("Terrain rows are system assets (origin system).")
# Editing an existing system row outside the scope works like every system asset: numbers may change (only the
# key field `id` is locked) and the row stays origin system; an explicit user origin is refused.
var sand_edit: Dictionary = ContentCatalog.get_entry("terrain", sand_id)
sand_edit.erase("origin")
checks["an outside-scope edit of a system row keeps origin system"] = ContentCatalog.upsert("terrain", sand_edit).is_empty() and ContentCatalog.is_system_origin(ContentCatalog.get_entry("terrain", sand_id))
var as_user: Dictionary = ContentCatalog.get_entry("terrain", sand_id)
as_user["origin"] = ContentSchema.ORIGIN_USER
checks["an edit that makes a system row user origin is refused"] = ContentCatalog.upsert("terrain", as_user).has("Terrain rows are system assets (origin system).")
checks["the catalog row wins over the code default"] = Terrain.move_cost(sand) == 3 and is_equal_approx(Terrain.cover_guard(Terrain.Type.RUBBLE), 0.25)
var h: Dictionary = Terrain.hazard(Terrain.Type.BURNING_GROUND)
checks["hazard() reads the row"] = str(h.get("status", "")) == burn and DictUtil.to_int(h.get("magnitude", -1)) == mag and str(h.get("on", "")) == TerrainFields.ON_ENTER and DictUtil.to_int(h.get("potency", 0)) == 100 and DictUtil.to_int(h.get("rounds", 0)) == 1
checks["describe() shows the move cost from the catalog"] = Terrain.describe(sand).contains("move cost 3")
checks["describe() shows cover"] = Terrain.describe(Terrain.Type.RUBBLE).contains("cover 25%")
checks["describe() shows the hazard status"] = Terrain.describe(Terrain.Type.BURNING_GROUND).contains(StatusEffects.display_name(burn))
checks["describe() of a blocked type"] = Terrain.describe(Terrain.Type.WALL).contains("blocked") and Terrain.describe(Terrain.Type.WALL).contains("cover 50%")
checks["the terrain rows validate"] = ContentCatalog.validate_all().keys().filter(func(k: Variant) -> bool: return str(k).begins_with("terrain:") and str(k) != "terrain:missing").is_empty()
checks["saving writes terrain.json in the sandbox"] = ContentCatalog.save("terrain") == OK and ContentCatalog.terrain_file_exists() and ContentCatalog.get_root() == SANDBOX
var missing: Array[String] = TerrainValidator.missing_ids(ContentCatalog.entries("terrain"))
checks["with terrain.json, missing rows are a validate_all error"] = ContentCatalog.validate_all().has("terrain:missing") == not missing.is_empty() and not missing.is_empty()
checks["with terrain.json, warnings_all no longer lists missing rows"] = not ContentCatalog.warnings_all().has("terrain:missing")
var rows: Array = []
for id: String in missing:
	rows.append(Terrain.default_row(Terrain.from_id(id)))
ContentCatalog.begin_system_edit()
checks["the missing rows store at once"] = ContentCatalog.upsert_many("terrain", rows).is_empty()
ContentCatalog.end_system_edit()
checks["every type has a row: no terrain:missing"] = not ContentCatalog.validate_all().has("terrain:missing")
checks["reload keeps the saved rows"] = ContentCatalog.save("terrain") == OK and _reloaded_cost(sand) == 3
return TestHelpers.report(TAG, "terrain catalog rows and code defaults", checks)
```

### `static _reloaded_cost(t: Terrain.Type) -> int`
```logic
ContentCatalog.reload_from_disk()
return Terrain.move_cost(t)
```

### `static _test_cover_input() -> bool`
```logic
var checks: Dictionary = {}
var grid := CombatGrid.new()
var attacker: Combatant = TestHelpers.unit("Shooter", grid, Vector2i(0, 0), 400, 10, 4, 1000)
var target: Combatant = TestHelpers.unit("Target", grid, Vector2i(5, 0))
target.defense_mitigation_ratio = 0.0
var plain: Dictionary = DamageCalculator.compute(attacker, target, true, false)
var zero: Dictionary = DamageCalculator.compute(attacker, target, true, false, 1.0, false, null, -1, true, 0.0)
var base: int = DictUtil.to_int(plain["final_dmg"])
checks["cover 0.0 equals the old call"] = base == DictUtil.to_int(zero["final_dmg"]) and is_zero_approx(DictUtil.to_float(zero["cover_ratio"]))
var cover: float = 0.25
var covered: Dictionary = DamageCalculator.compute(attacker, target, true, false, 1.0, false, null, -1, true, cover)
checks["cover reduces the hit like guard"] = DictUtil.to_int(covered["final_dmg"]) == maxi(1, roundi(float(base) * (1.0 - cover)))
checks["the result records the cover"] = is_equal_approx(DictUtil.to_float(covered["cover_ratio"]), cover)
checks["the breakdown shows cover"] = DamageCalculator.describe(covered).contains("cover -%d%%" % roundi(cover * 100.0)) and not DamageCalculator.describe(plain).contains("cover")
target.defense_mitigation_ratio = 0.5
var estimate: Dictionary = DamageCalculator.compute(attacker, target, true, false, 1.0, false, null, -1, false, cover)
checks["the estimate (include_guard false) still takes cover"] = DictUtil.to_int(estimate["final_dmg"]) == DictUtil.to_int(covered["final_dmg"])
var guard: float = 0.8
target.defense_mitigation_ratio = guard
var capped: Dictionary = DamageCalculator.compute(attacker, target, true, false, 1.0, false, null, -1, true, cover)
checks["guard + cover above GUARD_CAP is capped (takes 1 - GUARD_CAP of the pre-guard hit)"] = guard + cover > DamageCalculator.GUARD_CAP and DictUtil.to_int(capped["final_dmg"]) == maxi(1, roundi(float(base) * (1.0 - DamageCalculator.GUARD_CAP)))
var guarded: Dictionary = DamageCalculator.compute(attacker, target, true, false)
checks["guard alone below the cap is unchanged"] = DictUtil.to_int(guarded["final_dmg"]) == maxi(1, roundi(float(base) * (1.0 - guard)))
target.defense_mitigation_ratio = 0.0
return TestHelpers.report(TAG, "cover input and GUARD_CAP", checks)
```

### `static _test_fixture_rows() -> bool`
```logic
var checks: Dictionary = {}
checks["fresh sandbox fork"] = TestHelpers.fork(SANDBOX)
var burn: String = StatusEffects.BURN
var poison: String = StatusEffects.POISON
var rows: Array = [
	{"id": Terrain.id_of(Terrain.Type.FLOOR), "move_cost": 1},
	{"id": Terrain.id_of(Terrain.Type.MUD), "move_cost": MUD_COST},
	{"id": Terrain.id_of(Terrain.Type.SNOW), "move_cost": SNOW_COST},
	{"id": Terrain.id_of(Terrain.Type.RUBBLE), "move_cost": 1, "cover_guard": RUBBLE_COVER},
	{"id": Terrain.id_of(Terrain.Type.WALL), "cover_guard": WALL_COVER},
	{"id": Terrain.id_of(Terrain.Type.TREE), "cover_guard": TREE_COVER},
	{"id": Terrain.id_of(Terrain.Type.BURNING_GROUND), "move_cost": 1, "hazard_status": burn, "hazard_magnitude": StatusRegistry.magnitude_min(burn), "hazard_potency": 100, "hazard_on": TerrainFields.ON_BOTH},
	{"id": Terrain.id_of(Terrain.Type.TOXIC_BOG), "move_cost": 1, "hazard_status": poison, "hazard_magnitude": StatusRegistry.magnitude_min(poison), "hazard_potency": 100, "hazard_on": TerrainFields.ON_ENTER},
	{"id": Terrain.id_of(Terrain.Type.ICE), "move_cost": 1, "hazard_status": poison, "hazard_magnitude": StatusRegistry.magnitude_min(poison), "hazard_potency": 100, "hazard_on": TerrainFields.ON_ROUND_END},
]
ContentCatalog.begin_system_edit()
var errors: Array[String] = ContentCatalog.upsert_many("terrain", rows)
ContentCatalog.end_system_edit()
checks["fixture rows stored (%s)" % ", ".join(errors)] = errors.is_empty()
checks["the rows apply"] = Terrain.move_cost(Terrain.Type.MUD) == MUD_COST and is_equal_approx(Terrain.cover_guard(Terrain.Type.RUBBLE), RUBBLE_COVER) and not Terrain.hazard(Terrain.Type.BURNING_GROUND).is_empty()
# The Terrain row cache: an upsert, a reload and ContentCatalog.clear_cache() all reach the next read.
var mud_id: String = Terrain.id_of(Terrain.Type.MUD)
var cached: int = Terrain.move_cost(Terrain.Type.MUD)   # fills the cache
ContentCatalog.begin_system_edit()
var changed: bool = ContentCatalog.upsert("terrain", {"id": mud_id, "move_cost": MUD_COST + 1}).is_empty()
ContentCatalog.end_system_edit()
checks["an upserted terrain row changes move_cost immediately (cache cleared)"] = cached == MUD_COST and changed and Terrain.move_cost(Terrain.Type.MUD) == MUD_COST + 1
ContentCatalog.reload_from_disk()   # the upsert was never saved: the fork's file has no mud row
var on_disk: Dictionary = ContentCatalog.get_entry("terrain", mud_id)   # the fork's own row (none before S12T-A)
var after_reload: int = DictUtil.to_int(on_disk.get("move_cost", 0)) if not on_disk.is_empty() else Terrain.default_move_cost(Terrain.Type.MUD)
checks["a reload drops the unsaved row from the cache"] = Terrain.move_cost(Terrain.Type.MUD) == after_reload and after_reload != MUD_COST + 1
ContentCatalog.begin_system_edit()
var restored: Array[String] = ContentCatalog.upsert_many("terrain", rows)
ContentCatalog.end_system_edit()
checks["the fixture rows apply again after the reload"] = restored.is_empty() and Terrain.move_cost(Terrain.Type.MUD) == MUD_COST and Terrain.move_cost(Terrain.Type.SNOW) == SNOW_COST
return TestHelpers.report(TAG, "part B fixture rows", checks)
```

### `static _closed_grid() -> CombatGrid`
```logic
var exits: Array[Vector2i] = []
return CombatGrid.new(CombatGrid.ArenaType.CLOSED, exits)
```

### `static _test_move_chunking() -> bool`
```logic
var checks: Dictionary = {}
var grid: CombatGrid = _closed_grid()
var start := Vector2i(2, 4)
grid.set_terrain(start + Vector2i(2, 0), Terrain.Type.MUD)   # floor (3,4), mud (4,4), floor (5,4)
var right: Array[Vector2i] = _dirs(3, Vector2i(1, 0))
checks["route cost = 1 + mud + 1"] = GridPath.path_cost(grid, start, right) == 2 + MUD_COST
var one: Array[Array] = GridPath.chunk(grid, start, right, 1)
checks["budget 1: one tile per block, the mud tile entered in its own block (minimum progress)"] = _sizes(one) == [1, 1, 1]
checks["budget 1: 3 blocks for 3 tiles"] = GridPath.blocks_needed(grid, start, right, 1) == 3
var two: Array[Array] = GridPath.chunk(grid, start, right, 2)
checks["budget 2: floor, then stops before mud (1 + mud > 2), mud next block"] = 1 + MUD_COST > 2 and _sizes(two) == [1, 1, 1]
var all_floor: CombatGrid = _closed_grid()
checks["budget 2 on floor: two tiles per block (unchanged)"] = _sizes(GridPath.chunk(all_floor, start, right, 2)) == [2, 1]
checks["fit_blocks keeps the tiles of the first blocks"] = GridPath.fit_blocks(grid, start, right, 1, 2).size() == 2
# The planner chunks a move the same way (speed 10: two cost units per block).
var unit: Combatant = TestHelpers.unit("Walker", grid, start)
var foe: Combatant = TestHelpers.unit("Far", grid, Vector2i(10, 7))
var allies: Array[Combatant] = [unit]
var enemies: Array[Combatant] = [foe]
var plan := CombatPlan.new(TestHelpers.hand_ctx(grid, allies, enemies))
var budget: int = unit.move_per_block()
var to_mud: Vector2i = start + Vector2i(2, 0)
var problem: String = plan.add_step(unit, PlanStep.go_to(SkillCatalog.lowest(SkillDef.Kind.MOVE), to_mud))
var moves: Array[int] = []
for e: PlanEntry in plan.entries_for(unit):
	if e.skill.kind == SkillDef.Kind.MOVE:
		moves.append(e.tiles)
var expected: Array[int] = _sizes(GridPath.chunk(grid, start, _dirs(2, Vector2i(1, 0)), budget))
checks["planner Move entries follow the cost chunks (%s vs %s)" % [str(moves), str(expected)]] = problem.is_empty() and moves == expected and budget == 2 and expected == [1, 1]
var walk_ap: int = 0
for e: PlanEntry in plan.entries_for(unit):
	walk_ap += e.skill_ap if e.skill.kind == SkillDef.Kind.MOVE else 0
checks["AP per tile walked is unchanged"] = walk_ap == 2 * SkillCatalog.lowest(SkillDef.Kind.MOVE).ap_cost(unit)
plan.clear(unit)
var far_cell: Vector2i = start + Vector2i(3, 0)
var far_problem: String = plan.add_step(unit, PlanStep.go_to(SkillCatalog.lowest(SkillDef.Kind.MOVE), far_cell))
var far_route: Array[Vector2i] = GridPath.find_path(grid, start, {far_cell: true}, {}, false)
checks["a route needing more than MAX_MOVE_BLOCKS blocks is refused (Too far)"] = GridPath.blocks_needed(grid, start, far_route, budget) > CombatPlan.MAX_MOVE_BLOCKS and far_problem.contains("Too far")
return TestHelpers.report(TAG, "move chunking by cost", checks)
```

### `static _test_slow_exit() -> bool`
```logic
var checks: Dictionary = {}
var exit_cell := Vector2i(11, 4)
var exits: Array[Vector2i] = [exit_cell]
var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, exits)
grid.set_terrain(exit_cell, Terrain.Type.SNOW)
var unit: Combatant = TestHelpers.unit("Runner", grid, Vector2i(7, 4))
var foe: Combatant = TestHelpers.unit("Chaser", grid, Vector2i(1, 8))
var budget: int = unit.move_per_block()
var route: Array[Vector2i] = GridPath.find_path(grid, Vector2i(7, 4), {exit_cell: true}, {}, true)
checks["the slow exit tile is priced 1 everywhere (snow costs %d)" % SNOW_COST] = SNOW_COST > 1 and GridPath.enter_cost(grid, exit_cell) == 1 and GridPath.path_cost(grid, Vector2i(7, 4), route) == route.size()
var fits: bool = route.size() <= budget * CombatPlan.MAX_MOVE_BLOCKS
checks["blocks_needed counts the exit at cost 1"] = GridPath.blocks_needed(grid, Vector2i(7, 4), route, budget) == ceili(float(route.size()) / float(budget))
var allies: Array[Combatant] = [unit]
var enemies: Array[Combatant] = [foe]
var plan := CombatPlan.new(TestHelpers.hand_ctx(grid, allies, enemies))
var problem: String = plan.add_step(unit, PlanStep.withdraw())
var st: CombatPlan.SimState = plan.compile(unit)
checks["withdraw across the slow exit reaches it this round (route %d tiles, %d per block)" % [route.size(), budget]] = fits and problem.is_empty() and st.withdrawn
var moves: int = 0
for e: PlanEntry in plan.entries_for(unit):
	moves += 1 if e.skill.kind == SkillDef.Kind.MOVE else 0
checks["no extra move block for the exit"] = moves == GridPath.blocks_needed(grid, Vector2i(7, 4), route, budget)
return TestHelpers.report(TAG, "withdraw across a slow exit tile", checks)
```

### `static _test_path_cost() -> bool`
```logic
var checks: Dictionary = {}
var grid: CombatGrid = _closed_grid()
var start := Vector2i(0, 0)
var goal := Vector2i(2, 1)
var plain: Array[Vector2i] = GridPath.find_path(grid, start, {goal: true}, {}, false)
grid.set_terrain(start + plain[0], Terrain.Type.MUD)   # make the old first step expensive
var cheap: Array[Vector2i] = GridPath.find_path(grid, start, {goal: true}, {}, false)
checks["equal-length routes: the cheaper one wins"] = cheap.size() == plain.size() and GridPath.path_cost(grid, start, cheap) == cheap.size() and cheap[0] != plain[0]
checks["a cheaper longer route beats a shorter costly one"] = _detour_ok()
# Pinned against the old breadth-first search on maps whose walkable tiles all cost 1 (floor, walls, trees).
var pinned: bool = true
var cases: int = 0
var uniform: CombatGrid = CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(11, 4)] as Array[Vector2i])
for cell: Vector2i in [Vector2i(4, 2), Vector2i(4, 3), Vector2i(7, 5), Vector2i(2, 6), Vector2i(9, 1)]:
	uniform.set_terrain(cell, Terrain.Type.WALL)
uniform.set_terrain(Vector2i(6, 6), Terrain.Type.TREE)
var blocked: Dictionary = {Vector2i(5, 4): true, Vector2i(8, 3): true}
var goal_sets: Array[Dictionary] = [{Vector2i(6, 4): true}, {Vector2i(10, 2): true, Vector2i(10, 6): true, Vector2i(9, 4): true}, {Vector2i(1, 1): true, Vector2i(3, 7): true}]
for y: int in range(CombatGrid.GRID_H):
	for x: int in range(CombatGrid.GRID_W):
		var from := Vector2i(x, y)
		if not uniform.is_walkable(from) or uniform.is_exit_tile(from):
			continue
		for goals: Dictionary in goal_sets:
			cases += 1
			pinned = pinned and GridPath.find_path(uniform, from, goals, blocked, false) == _bfs_reference(uniform, from, goals, blocked, false)
		var exits: Dictionary = {Vector2i(11, 4): true}
		cases += 1
		pinned = pinned and GridPath.find_path(uniform, from, exits, blocked, true) == _bfs_reference(uniform, from, exits, blocked, true)
checks["uniform-cost maps: identical to the breadth-first result (%d cases)" % cases] = pinned and cases > 100
return TestHelpers.report(TAG, "cost-weighted paths", checks)
```

### `static _detour_ok() -> bool`
```logic
var grid: CombatGrid = _closed_grid()
for x: int in range(2, 5):
	grid.set_terrain(Vector2i(x, 1), Terrain.Type.MUD)
var start := Vector2i(1, 1)
var goal := Vector2i(5, 1)
var route: Array[Vector2i] = GridPath.find_path(grid, start, {goal: true}, {}, false)
var through: int = 3 * MUD_COST + 1
var around: int = 6
var cost: int = GridPath.path_cost(grid, start, route)
if around < through:
	return cost == around and route.size() == around
return cost == through and route.size() == 4
```

### `static _test_cover_rules() -> bool`
```logic
var checks: Dictionary = {}
var grid: CombatGrid = _closed_grid()
var target_pos := Vector2i(5, 4)
grid.set_terrain(target_pos, Terrain.Type.RUBBLE)
var shooter: Combatant = TestHelpers.unit("Archer", grid, Vector2i(5, 1), 400, 10, 4, 1000)
var brawler: Combatant = TestHelpers.unit("Brawler", grid, Vector2i(4, 4), 400, 10, 1, 1000)
var target: Combatant = TestHelpers.unit("Target", grid, target_pos, 100000)
checks["melee never sees cover"] = is_zero_approx(CoverRules.cover_for(grid, Vector2i(4, 4), target_pos, false))
checks["ranged: the target tile's cover"] = is_equal_approx(CoverRules.cover_for(grid, Vector2i(5, 1), target_pos, true), RUBBLE_COVER)
var expected_ranged: int = DictUtil.to_int(DamageCalculator.compute(shooter, target, true, false, 1.0, false, null, -1, true, RUBBLE_COVER)["final_dmg"])
var hp_before: int = target.hp
var shot := AttackCommand.new(shooter, target, 1.0, 2, 0, grid)
shot.execute()
checks["a ranged hit on rubble deals compute(cover_ratio = rubble cover)"] = hp_before - target.hp == expected_ranged and expected_ranged < DictUtil.to_int(DamageCalculator.compute(shooter, target, true, false)["final_dmg"])
var expected_melee: int = DictUtil.to_int(DamageCalculator.compute(brawler, target, false, false)["final_dmg"])
hp_before = target.hp
var punch := AttackCommand.new(brawler, target, 1.0, 2, 0, grid)
punch.execute()
checks["the same target in melee takes the uncovered amount"] = hp_before - target.hp == expected_melee
# Directional cover on a plain floor target.
var open: CombatGrid = _closed_grid()
var t := Vector2i(5, 4)
open.set_terrain(t + Vector2i(1, 0), Terrain.Type.WALL)
checks["attacker due east, wall on the target's east neighbour: the wall's cover"] = is_equal_approx(CoverRules.cover_for(open, Vector2i(9, 4), t, true), WALL_COVER)
checks["attacker due west: no cover (the wall is behind the target)"] = is_zero_approx(CoverRules.cover_for(open, Vector2i(1, 4), t, true))
open.set_terrain(t + Vector2i(0, -1), Terrain.Type.TREE)
checks["diagonal attacker: the larger of the two orthogonal neighbours"] = is_equal_approx(CoverRules.cover_for(open, Vector2i(8, 1), t, true), maxf(WALL_COVER, TREE_COVER))
checks["diagonal attacker on the other side: only the tree counts"] = is_equal_approx(CoverRules.cover_for(open, Vector2i(2, 1), t, true), TREE_COVER)
var floor_grid: CombatGrid = _closed_grid()
checks["an adjacent ranged attacker's own tile gives no cover"] = is_zero_approx(CoverRules.cover_for(floor_grid, t + Vector2i(1, 0), t, true))
open.set_terrain(t, Terrain.Type.RUBBLE)
checks["the larger of tile cover and directional cover"] = is_equal_approx(CoverRules.cover_for(open, Vector2i(9, 4), t, true), maxf(RUBBLE_COVER, WALL_COVER))
# The planner estimate takes cover (cover is known ahead).
var allies: Array[Combatant] = [shooter]
var enemies: Array[Combatant] = [target]
var ctx: PlanContext = TestHelpers.hand_ctx(grid, allies, enemies)
var plan := CombatPlan.new(ctx)
var basic: SkillDef = SkillCatalog.lowest(SkillDef.Kind.ATTACK)
checks["estimate_damage includes the target's cover"] = plan.estimate_damage(shooter, basic, shooter.current_stance, target) == DamageCalculator.expected(shooter, target, basic, shooter.current_stance, RUBBLE_COVER)
var options: Array[PlanOption] = plan.target_options(shooter, target)
var shows_cover: bool = false
for o: PlanOption in options:
	shows_cover = shows_cover or (o.valid and is_equal_approx(o.cover, RUBBLE_COVER) and o.cost_text().contains("cover -%d%%" % roundi(RUBBLE_COVER * 100.0)))
checks["the attack option shows the cover"] = shows_cover
return TestHelpers.report(TAG, "cover", checks)
```

### `static _test_hazards() -> bool`
```logic
var checks: Dictionary = {}
var burn: String = StatusEffects.BURN
var grid: CombatGrid = _closed_grid()
var start := Vector2i(2, 2)
grid.set_terrain(start + Vector2i(1, 0), Terrain.Type.BURNING_GROUND)
grid.set_terrain(start + Vector2i(2, 0), Terrain.Type.BURNING_GROUND)
var walker: Combatant = TestHelpers.unit("Walker", grid, start)
var rng := RandomNumberGenerator.new()
rng.seed = 7
var walk := WalkCommand.new(walker, grid, _dirs(3, Vector2i(1, 0)))
walk.rng = rng
walk.execute()
checks["walking through two burning tiles rolls burn once"] = walk.hazard_notes.size() == 1 and walk.execution_log.contains("[Hazard] Walker is %s by %s" % [StatusEffects.display_name(burn), Terrain.display_name(Terrain.Type.BURNING_GROUND)])
checks["the burn is queued for next round (not active yet)"] = walker.pending_statuses.has(burn) and not walker.has_status(burn)
checks["its source is terrain"] = str(walker.pending_status_sources.get(burn, "")) == HazardRules.SOURCE
# Round end: a unit standing on burning ground (enter + round end) is rolled again.
var resolver := CombatResolver.new()
resolver.grid = grid
var stander: Combatant = TestHelpers.unit("Stander", grid, start + Vector2i(1, 0))
resolver.register_combatant(stander)
resolver.register_combatant(walker)
var lines: Array[String] = []
resolver.log_emitted.connect(func(msg: String) -> void: lines.append(msg))
resolver.apply_round_end_effects()
checks["standing on burning ground at round end rolls burn"] = stander.pending_statuses.has(burn) and TestHelpers.any_contains(lines, "[Hazard] Stander")
checks["a unit off hazard tiles is not rolled"] = not TestHelpers.any_contains(lines, "[Hazard] Walker")
# Trigger kinds: enter-only tiles skip round end; round-end-only tiles skip walks.
var bog: Combatant = TestHelpers.unit("Bog", grid, Vector2i(8, 7))
grid.set_terrain(Vector2i(8, 7), Terrain.Type.TOXIC_BOG)
var ice_walker: Combatant = TestHelpers.unit("Skater", grid, Vector2i(8, 1))
grid.set_terrain(Vector2i(9, 1), Terrain.Type.ICE)
var skate := WalkCommand.new(ice_walker, grid, _dirs(1, Vector2i(1, 0)))
skate.rng = rng
skate.execute()
var units: Array[Combatant] = [bog]
checks["an enter-only tile is not rolled at round end"] = HazardRules.at_round_end(grid, units, rng).is_empty()
checks["a round-end-only tile is not rolled on enter"] = skate.hazard_notes.is_empty() and ice_walker.pending_statuses.is_empty()
units = [ice_walker]
checks["a round-end-only tile is rolled at round end"] = HazardRules.at_round_end(grid, units, rng).size() == 1
# Preview: a planned path through a hazard tile shows the chip.
var planner_grid: CombatGrid = _closed_grid()
planner_grid.set_terrain(Vector2i(4, 4), Terrain.Type.BURNING_GROUND)
var hero: Combatant = TestHelpers.unit("Hero", planner_grid, Vector2i(3, 4))
var foe: Combatant = TestHelpers.unit("Foe", planner_grid, Vector2i(10, 7))
var allies: Array[Combatant] = [hero]
var enemies: Array[Combatant] = [foe]
var plan := CombatPlan.new(TestHelpers.hand_ctx(planner_grid, allies, enemies))
var chip: String = "Hazard: %s (%s)" % [Terrain.display_name(Terrain.Type.BURNING_GROUND), StatusEffects.display_name(burn)]
var found: bool = false
for o: PlanOption in plan.cell_options(hero, Vector2i(5, 4)):
	found = found or (o.valid and o.hazard == chip and o.cost_text().contains(chip))
checks["a planned path through a hazard shows '%s'" % chip] = found
var clean: bool = true
for o: PlanOption in plan.cell_options(hero, Vector2i(3, 6)):
	clean = clean and o.hazard.is_empty()
checks["a path without hazards shows no chip"] = clean
return TestHelpers.report(TAG, "hazards", checks)
```

### `static _test_disengage_and_dash() -> bool`
```logic
var checks: Dictionary = {}
var poison: String = StatusEffects.POISON
var grid: CombatGrid = _closed_grid()
var start := Vector2i(3, 3)
grid.set_terrain(start + Vector2i(1, 0), Terrain.Type.MUD)
grid.set_terrain(start + Vector2i(2, 0), Terrain.Type.MUD)
var dest: Vector2i = start + Vector2i(3, 0)
grid.set_terrain(dest, Terrain.Type.TOXIC_BOG)
var jumper: Combatant = TestHelpers.unit("Jumper", grid, start)
var jump := DisengageCommand.new(jumper, grid, dest, 3)
var rng := RandomNumberGenerator.new()
rng.seed = 11
jump.rng = rng
jump.execute()
checks["disengage over mud lands with no cost check (3 tiles, route cost %d)" % GridPath.path_cost(grid, start, _dirs(3, Vector2i(1, 0)))] = jump.landed and grid.get_position(jumper) == dest and GridPath.path_cost(grid, start, _dirs(3, Vector2i(1, 0))) > 3
checks["landing on toxic bog rolls poison"] = jumper.pending_statuses.has(poison) and jump.execution_log.contains("[Hazard] Jumper")
# Dash: a cost budget with the minimum-progress rule.
var floor_two: Array[Vector2i] = _dirs(2, Vector2i(0, 1))
checks["dash 2 covers two floor tiles"] = CombatPlan.dash_fits(grid, start, floor_two, 2)
checks["dash 2 covers mud then floor only when their cost fits"] = CombatPlan.dash_fits(grid, start, [Vector2i(1, 0), Vector2i(0, 1)] as Array[Vector2i], 2) == (MUD_COST + 1 <= 2)
checks["dash 1 still enters one mud tile (minimum progress)"] = CombatPlan.dash_fits(grid, start, [Vector2i(1, 0)] as Array[Vector2i], 1)
checks["no dash without dash tiles"] = not CombatPlan.dash_fits(grid, start, floor_two, 0)
return TestHelpers.report(TAG, "disengage and dash", checks)
```

### `static _test_zones() -> bool`
```logic
var checks: Dictionary = {}
var hazard_types: Array[int] = [Terrain.Type.BURNING_GROUND, Terrain.Type.TOXIC_BOG, Terrain.Type.ICE]
var expected: Dictionary = {"lava_cavern": Terrain.Type.BURNING_GROUND, "swamp_bog": Terrain.Type.TOXIC_BOG, "frozen_pass": Terrain.Type.ICE}
for zone_id: String in CombatZones.zone_ids():
	var map: Dictionary = CombatZones.terrain_map(zone_id)
	var count: int = 0
	var outside_core: bool = true
	for key: Variant in map:
		var cell: Vector2i = key
		var t: int = DictUtil.to_int(map[key])
		if hazard_types.has(t):
			count += 1
			outside_core = outside_core and not (cell.x >= 3 and cell.x <= 8 and cell.y >= 2 and cell.y <= 6) and cell.y != 4
			outside_core = outside_core and (not expected.has(zone_id) or t == DictUtil.to_int(expected[zone_id]))
	if expected.has(zone_id):
		checks["%s has hazard tiles outside the core and exit lane" % zone_id] = count > 0 and outside_core
	else:
		checks["%s has no hazard tiles" % zone_id] = count == 0
var tiles_ok: bool = true
for t: int in hazard_types:
	var floor_svg: String = TileArt.tile_svg(Terrain.Type.FLOOR, 0)
	for v: int in range(TileArt.VARIANTS):
		tiles_ok = tiles_ok and TileArt.tile_svg(t as Terrain.Type, v) != TileArt.tile_svg(Terrain.Type.FLOOR, v) and TileArt.template_tile(t as Terrain.Type, v) != null
	tiles_ok = tiles_ok and not floor_svg.is_empty()
checks["the three hazard types have their own drawings (4 variants)"] = tiles_ok
return TestHelpers.report(TAG, "zones and tile art", checks)
```

### `static _test_ai_hazard_penalty() -> bool`
```logic
var checks: Dictionary = {}
var grid: CombatGrid = _closed_grid()
var start := Vector2i(2, 4)
var goal := Vector2i(6, 4)
var hazard := Vector2i(3, 4)
grid.set_terrain(hazard, Terrain.Type.BURNING_GROUND)
var straight: Array[Vector2i] = GridPath.find_path(grid, start, {goal: true}, {}, false)
var ai_route: Array[Vector2i] = GridPath.find_path(grid, start, {goal: true}, {}, false, TacticalAI.hazard_penalty(grid))
var detour_extra: int = 2   # one row over and back
checks["players see true costs: the straight route through the hazard"] = _route_cells(start, straight).has(hazard) and straight.size() == 4
checks["AI routes avoid the hazard when the detour costs less than AI_HAZARD_PENALTY (%d < %d)" % [detour_extra, TacticalAI.AI_HAZARD_PENALTY]] = (detour_extra < TacticalAI.AI_HAZARD_PENALTY) == (not _route_cells(start, ai_route).has(hazard)) and ai_route.size() == (straight.size() + detour_extra if detour_extra < TacticalAI.AI_HAZARD_PENALTY else straight.size())
var far_hazard_grid: CombatGrid = _closed_grid()
for y: int in range(CombatGrid.GRID_H):
	far_hazard_grid.set_terrain(Vector2i(4, y), Terrain.Type.BURNING_GROUND)
var forced: Array[Vector2i] = GridPath.find_path(far_hazard_grid, start, {goal: true}, {}, false, TacticalAI.hazard_penalty(far_hazard_grid))
checks["the penalty never blocks: a full hazard column is still crossed"] = forced.size() == 4
# In a plan: only units with a penalty (AI drafts) route around it.
var walker: Combatant = TestHelpers.unit("Raider", grid, start)
var foe: Combatant = TestHelpers.unit("Guard", grid, Vector2i(10, 8))
var allies: Array[Combatant] = [walker]
var enemies: Array[Combatant] = [foe]
var ctx: PlanContext = TestHelpers.hand_ctx(grid, allies, enemies)
var plan := CombatPlan.new(ctx)
var dest := Vector2i(4, 4)
var move: SkillDef = SkillCatalog.lowest(SkillDef.Kind.MOVE)
var player_problem: String = plan.add_step(walker, PlanStep.go_to(move, dest))
var player_cells: Array[Vector2i] = _plan_cells(plan, walker, start)
plan.clear(walker)
plan.path_penalties[walker] = TacticalAI.hazard_penalty(grid)
var ai_problem: String = plan.add_step(walker, PlanStep.go_to(move, dest))
var ai_cells: Array[Vector2i] = _plan_cells(plan, walker, start)
checks["a player plan walks the true cheapest route (through the hazard)"] = player_problem.is_empty() and player_cells.has(hazard)
checks["an AI-penalised plan steps around the hazard"] = ai_problem.is_empty() and not ai_cells.is_empty() and not ai_cells.has(hazard) and ai_cells[ai_cells.size() - 1] == dest
# TacticalAI registers the penalty on its drafts.
var shared := CombatPlan.new(ctx)
TacticalAI.build_steps(foe, ctx, ctx.profile_of(foe), shared)
checks["TacticalAI.build_steps sets the hazard penalty on its draft"] = shared.path_penalties.has(foe) and not plan.path_penalties.has(foe)
return TestHelpers.report(TAG, "AI hazard penalty", checks)
```

### `static _test_ai_cover_preference() -> bool`
```logic
var checks: Dictionary = {}
var grid: CombatGrid = _closed_grid()
var start := Vector2i(5, 4)
var open_cell := Vector2i(5, 2)
var covered_cell := Vector2i(5, 6)
grid.set_terrain(covered_cell + Vector2i(0, 1), Terrain.Type.WALL)   # between the covered cell and the shooter
var shooter: Combatant = TestHelpers.unit("Sniper", grid, covered_cell + Vector2i(0, 2), 400, 10, 4)
var foes: Array[Combatant] = [shooter]
var goals: Dictionary = {open_cell: true, covered_cell: true}
var plain: Array[Vector2i] = GridPath.find_path(grid, start, goals, {}, false)
var prefer: Callable = TacticalAI.cover_preference(grid, foes)
var picked: Array[Vector2i] = GridPath.find_path(grid, start, goals, {}, false, Callable(), prefer)
var plain_end: Vector2i = _route_cells(start, plain)[plain.size() - 1]
var picked_end: Vector2i = _route_cells(start, picked)[picked.size() - 1]
checks["the two cells rank equally (same cost and steps)"] = plain.size() == picked.size() and GridPath.path_cost(grid, start, plain) == GridPath.path_cost(grid, start, picked)
checks["the covered cell has more cover against the nearest ranged foe"] = DictUtil.to_float(prefer.call(covered_cell)) > DictUtil.to_float(prefer.call(open_cell)) and is_equal_approx(DictUtil.to_float(prefer.call(covered_cell)), WALL_COVER)
checks["without the preference the old tie order picks the other cell"] = plain_end == open_cell
checks["the AI preference picks the covered cell"] = picked_end == covered_cell
var nearer: Dictionary = {covered_cell: true, start + Vector2i(1, 0): true}
var near_pick: Array[Vector2i] = GridPath.find_path(grid, start, nearer, {}, false, Callable(), prefer)
checks["cover never beats a cheaper cell (only among equal ranks)"] = near_pick.size() == 1
var no_shooters: Array[Combatant] = [TestHelpers.unit("Brute", grid, Vector2i(9, 1))]
checks["no ranged foe: no preference"] = not TacticalAI.cover_preference(grid, no_shooters).is_valid()
# Covered targets are estimated lower (rule 8c): same attacker, one target on rubble, one on floor.
var archer: Combatant = TestHelpers.unit("Archer", grid, Vector2i(1, 1), 400, 10, 4, 500)
var on_rubble: Combatant = TestHelpers.unit("Covered", grid, Vector2i(1, 4))
grid.set_terrain(Vector2i(1, 4), Terrain.Type.RUBBLE)
var on_floor: Combatant = TestHelpers.unit("Open", grid, Vector2i(4, 1))
var allies: Array[Combatant] = [archer]
var enemies: Array[Combatant] = [on_rubble, on_floor]
var plan := CombatPlan.new(TestHelpers.hand_ctx(grid, allies, enemies))
var basic: SkillDef = SkillCatalog.lowest(SkillDef.Kind.ATTACK)
checks["a covered target's expected hit is lower"] = plan.estimate_damage(archer, basic, archer.current_stance, on_rubble) < plan.estimate_damage(archer, basic, archer.current_stance, on_floor)
return TestHelpers.report(TAG, "AI cover preference", checks)
```

