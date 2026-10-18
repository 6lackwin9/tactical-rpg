class_name ObjectiveTestSuite
extends RefCounted

# ==============================================================================
# OBJ1-L: encounter objectives, guests and deployment (EncounterObjectives).
# Stage 1 (schema): the nine fields and their defaults in both schemas, old entries normalising
# to rout / standard / no guests, every validation message of spec 1.1 in both catalogs, a full
# valid entry per objective clean in both catalogs, the generated text (rule 9) and the World
# Forge preview lines (rule 12). Fixture refs are picked by rule (side / role / terrain); world
# writes go to a sandbox fork only. Nothing depends on shipped numbers.
# ==============================================================================

const TAG: String = "Objective"
const MAX_ROUNDS: int = 6

const FIELDS: Dictionary = {
	"objective": "rout", "objective_tiles": [], "objective_rounds": 3, "objective_unit": "",
	"objective_count": 1, "objective_text": "", "deployment": "standard", "guests": [], "protect": [],
}

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING OBJECTIVE TEST (OBJ1-L)                 ===")
	print("=======================================================")
	var ok: bool = true
	ok = _test_schema_defaults() and ok
	ok = _test_generated_text() and ok
	ok = _test_catalog_validation() and ok
	ok = _test_world_validation() and ok
	ok = _test_shipped_clean() and ok
	return ok

# ------------------------------------------------------------------------------
# Fixtures
# ------------------------------------------------------------------------------

# Catalog refs by rule: {party, guest, a} = a party unit, a second party unit (or the same one) and an enemy unit.
static func _catalog_refs() -> Dictionary:
	var party: Array[String] = []
	var foe: String = ""
	for u: Dictionary in ContentCatalog.entries("units"):
		var side: String = str(u.get("side", ""))
		if side == "party" and party.size() < 2:
			party.append(str(u["id"]))
		elif side == "enemy" and foe.is_empty():
			foe = str(u["id"])
	if party.is_empty() or foe.is_empty():
		return {}
	return {"party": party[0], "guest": party[party.size() - 1], "a": foe}

# A world with a party and an enemy character: {world, party, guest, a} (the guest is the party character).
static func _world_refs() -> Dictionary:
	for w: String in WorldCatalog.world_ids():
		var party: String = ""
		var foe: String = ""
		for c: Dictionary in WorldCatalog.merged_entries(w, "characters"):
			var role: String = str(c.get("role", ""))
			if role == "party" and party.is_empty():
				party = WorldCatalog.ref_value(w, c)
			elif role == "enemy" and foe.is_empty():
				foe = WorldCatalog.ref_value(w, c)
		if not party.is_empty() and not foe.is_empty():
			return {"world": w, "party": party, "guest": party, "a": foe}
	return {}

# The first zone with a tile that cannot be walked on: {zone, cell}; {} when every zone is open.
static func _blocked_tile() -> Dictionary:
	for zone: String in CombatZones.zone_ids():
		var terrain: Dictionary = CombatZones.terrain_map(zone)
		for cell: Variant in terrain:
			var raw: Variant = terrain[cell]
			var t: Terrain.Type = raw if raw is Terrain.Type else Terrain.Type.FLOOR
			if not Terrain.is_walkable(t):
				return {"zone": zone, "cell": cell}
	return {}

static func _entry(refs: Dictionary, changes: Dictionary) -> Dictionary:
	var e: Dictionary = {
		"id": "objective_probe", "name": "Objective Probe", "zone": "stone_arena", "max_rounds": MAX_ROUNDS,
		"party": [refs["party"]], "enemies": [refs["a"]],
	}
	e.merge(changes, true)
	return e

# One valid entry per objective (and a guest / protect / ambush rout).
static func _good_cases(refs: Dictionary) -> Dictionary:
	return {
		"rout with a protected guest, ambush": {"objective": "rout", "deployment": "ambush", "guests": [refs["guest"]], "protect": [refs["guest"]]},
		"hold on the ring": {"objective": "hold", "objective_rounds": MAX_ROUNDS, "deployment": "centre"},
		"hold on own tiles": {"objective": "hold", "objective_tiles": [[4, 4], [5, 3]], "objective_rounds": 1, "objective_text": "x".repeat(EncounterObjectives.MAX_TEXT)},
		"escape": {"objective": "escape", "objective_count": 4},
		"escape in an open arena": {"objective": "escape", "arena": "OPEN", "exits": []},
		"slay": {"objective": "slay", "objective_unit": refs["a"]},
	}

# Bad cases: label -> [changes, needle]. Every message of spec 1.1 (plus the generic enum / ref guards).
static func _bad_cases(refs: Dictionary) -> Dictionary:
	var g: String = str(refs["guest"])
	var cases: Dictionary = {
		"bad objective": [{"objective": "win"}, "Objective: 'win' is not one of"],
		"bad deployment": [{"deployment": "flank"}, "Deployment: 'flank' is not one of"],
		"hold tile off the grid": [{"objective": "hold", "objective_tiles": [[CombatGrid.GRID_W, 4]]}, "Objective tile %s is off the grid, blocked or an exit." % str(Vector2i(CombatGrid.GRID_W, 4))],
		"hold tile not a pair": [{"objective": "hold", "objective_tiles": [[5]]}, "is off the grid, blocked or an exit."],
		"hold tile on an exit": [{"objective": "hold", "exits": [[6, 4]]}, "Objective tile %s is off the grid, blocked or an exit." % str(Vector2i(6, 4))],
		"hold tile on an open edge": [{"objective": "hold", "arena": "OPEN", "objective_tiles": [[0, 4]]}, "Objective tile %s is off the grid, blocked or an exit." % str(Vector2i(0, 4))],
		"hold rounds past max": [{"objective": "hold", "objective_rounds": MAX_ROUNDS + 1}, "Hold rounds must be 1..%d." % MAX_ROUNDS],
		"slay without a unit": [{"objective": "slay"}, "Slay needs a unit from enemies."],
		"slay a non-enemy": [{"objective": "slay", "objective_unit": refs["party"]}, "Slay needs a unit from enemies."],
		"escape without exits": [{"objective": "escape", "exits": []}, "Escape needs an exit tile."],
		"long text": [{"objective_text": "x".repeat(EncounterObjectives.MAX_TEXT + 1)}, "Objective text holds at most %d characters." % EncounterObjectives.MAX_TEXT],
		"three guests": [{"guests": [g, g, g]}, "An encounter holds at most %d guests." % EncounterObjectives.MAX_GUESTS],
		"protect not a guest": [{"protect": [g]}, "Protect '%s' is not a guest." % g],
		"enemy as guest": [{"guests": [refs["a"]]}, "Guests: '%s' can't be used here." % str(refs["a"])],
	}
	var blocked: Dictionary = _blocked_tile()
	if not blocked.is_empty():
		var cell: Vector2i = blocked["cell"]
		cases["hold tile blocked in %s" % str(blocked["zone"])] = [{"objective": "hold", "zone": blocked["zone"], "objective_tiles": [[cell.x, cell.y]]}, "Objective tile %s is off the grid, blocked or an exit." % str(cell)]
	return cases

static func _matrix(refs: Dictionary, validate: Callable) -> Dictionary:
	var checks: Dictionary = {}
	var good: Dictionary = _good_cases(refs)
	for label: Variant in good:
		var errors: Array[String] = validate.call(_entry(refs, good[label]))
		checks["valid: %s (%s)" % [str(label), "; ".join(errors)]] = errors.is_empty()
	var bad: Dictionary = _bad_cases(refs)
	for label: Variant in bad:
		var pair: Array = bad[label]
		var errors: Array[String] = validate.call(_entry(refs, pair[0]))
		checks["%s -> '%s' (%s)" % [str(label), str(pair[1]), "; ".join(errors)]] = TestHelpers.any_contains(errors, str(pair[1]))
	checks["blocked-tile fixture found (some zone has a wall)"] = not _blocked_tile().is_empty()
	return checks

# ------------------------------------------------------------------------------
# Tests
# ------------------------------------------------------------------------------

static func _test_schema_defaults() -> bool:
	var checks: Dictionary = {}
	var old_catalog: Dictionary = ContentSchema.normalize("encounters", {"id": "old", "name": "Old"})
	var old_world: Dictionary = WorldSchema.normalize("encounters", {"id": "old", "name": "Old"})
	for key: String in FIELDS:
		var cf: Dictionary = ContentSchema.field("encounters", key)
		var wf: Dictionary = WorldSchema.field("encounters", key)
		checks["catalog field %s default %s" % [key, str(FIELDS[key])]] = not cf.is_empty() and str(cf.get("default", null)) == str(FIELDS[key])
		checks["world field %s default %s" % [key, str(FIELDS[key])]] = not wf.is_empty() and str(wf.get("default", null)) == str(FIELDS[key])
		checks["old catalog entry: %s = %s" % [key, str(old_catalog.get(key, null))]] = str(old_catalog.get(key, null)) == str(FIELDS[key])
		checks["old world entry: %s = %s" % [key, str(old_world.get(key, null))]] = str(old_world.get(key, null)) == str(FIELDS[key])
	checks["objective options"] = ContentSchema.options_of(ContentSchema.field("encounters", "objective")) == EncounterObjectives.OBJECTIVES
	checks["deployment options"] = ContentSchema.options_of(ContentSchema.field("encounters", "deployment")) == EncounterObjectives.DEPLOYMENTS
	checks["the ring is the default hold tiles"] = EncounterObjectives.tiles_of(old_catalog) == EncounterObjectives.RING
	checks["authored tiles replace the ring"] = EncounterObjectives.tiles_of({"objective_tiles": [[2, 3]]}) == ([Vector2i(2, 3)] as Array[Vector2i])
	return TestHelpers.report(TAG, "Nine fields with spec defaults in both schemas; old entries normalise to rout, standard, no guests", checks)

static func _test_generated_text() -> bool:
	var checks: Dictionary = {}
	var name_of: Callable = func(ref: String) -> String: return ref.capitalize()
	checks["rout"] = EncounterObjectives.text_of({"objective": "rout"}, name_of) == "Defeat every enemy."
	checks["hold"] = EncounterObjectives.text_of({"objective": "hold", "objective_rounds": 4}, name_of) == "Hold the ring for 4 rounds, or defeat every enemy."
	checks["escape"] = EncounterObjectives.text_of({"objective": "escape", "objective_count": 2}, name_of) == "Get 2 out by the exit, or defeat every enemy."
	checks["slay"] = EncounterObjectives.text_of({"objective": "slay", "objective_unit": "big_boss"}, name_of) == "Defeat Big Boss."
	checks["protect two"] = EncounterObjectives.text_of({"objective": "rout", "protect": ["wenn", "orsa"]}, name_of) == "Defeat every enemy. Keep Wenn and Orsa alive."
	checks["authored text wins"] = EncounterObjectives.text_of({"objective": "hold", "objective_text": "Hold it.", "protect": ["wenn"]}, name_of) == "Hold it."
	checks["objective_tiles not a list is refused"] = TestHelpers.any_contains(EncounterObjectives.problems({"objective": "hold", "objective_tiles": "x", "max_rounds": MAX_ROUNDS, "objective_rounds": 1}, func(_r: String) -> bool: return true), "Objective tiles must be a list")
	return TestHelpers.report(TAG, "Generated objective text (rule 9)", checks)

static func _test_catalog_validation() -> bool:
	var refs: Dictionary = _catalog_refs()
	if refs.is_empty():
		return TestHelpers.report_ok(TAG, "Catalog validation", false, "no party / enemy units in the catalog")
	var checks: Dictionary = _matrix(refs, func(e: Dictionary) -> Array[String]: return ContentCatalog.validate_entry("encounters", e))
	return TestHelpers.report(TAG, "Catalog encounters: every objective message fires on a bad entry; valid entries are clean", checks)

# World: the shared matrix plus the guest rule for npcs (test-only characters in a sandbox fork).
static func _test_world_validation() -> bool:
	var refs: Dictionary = _world_refs()
	var crefs: Dictionary = _catalog_refs()
	if refs.is_empty() or crefs.is_empty():
		return TestHelpers.report_ok(TAG, "World validation", false, "no world with party and enemy characters")
	var world: String = str(refs["world"])
	var path: String = TestHelpers.sandbox("objective_world")
	if not TestHelpers.fork_worlds(path):
		TestHelpers.cleanup_worlds(path)
		return TestHelpers.report_ok(TAG, "World validation", false, "world fork failed")
	var checks: Dictionary = _matrix(refs, func(e: Dictionary) -> Array[String]: return WorldCatalog.validate_entry(world, "encounters", e))
	var helper: Array[String] = WorldCatalog.upsert(world, "characters", {"id": "obj_probe_helper", "name": "Probe Helper", "role": "npc", "base_unit": crefs["guest"]})
	var turncoat: Array[String] = WorldCatalog.upsert(world, "characters", {"id": "obj_probe_turncoat", "name": "Probe Turncoat", "role": "npc", "base_unit": crefs["a"]})
	checks["fixture npcs stored (%s | %s)" % ["; ".join(helper), "; ".join(turncoat)]] = helper.is_empty() and turncoat.is_empty()
	var npc_ok: Array[String] = WorldCatalog.validate_entry(world, "encounters", _entry(refs, {"guests": ["obj_probe_helper"], "protect": ["obj_probe_helper"]}))
	checks["an npc with a party-side base unit is a valid guest (%s)" % "; ".join(npc_ok)] = npc_ok.is_empty()
	var npc_bad: Array[String] = WorldCatalog.validate_entry(world, "encounters", _entry(refs, {"guests": ["obj_probe_turncoat"]}))
	checks["an npc with an enemy base unit is refused (%s)" % "; ".join(npc_bad)] = TestHelpers.any_contains(npc_bad, "Guest 'obj_probe_turncoat' must fight on the party side.")
	var stored: Array[String] = WorldCatalog.upsert(world, "encounters", _entry(refs, {"objective": "slay", "objective_unit": refs["a"], "deployment": "ambush", "guests": ["obj_probe_helper"], "protect": ["obj_probe_helper"]}))
	checks["world upsert accepts the objective fields (%s)" % "; ".join(stored)] = stored.is_empty()
	var own: Dictionary = WorldCatalog.get_own(world, "encounters", "objective_probe")
	checks["world keeps the objective fields"] = str(own.get("objective", "")) == "slay" and str(own.get("deployment", "")) == "ambush" and DictUtil.array_of(own.get("guests", [])).size() == 1
	var preview: String = WorldPreview.describe(world, "encounters", own)
	var foe_name: String = str(WorldCatalog.resolve(world, "characters", str(refs["a"])).get("name", ""))
	checks["preview: Objective line (%s)" % foe_name] = preview.contains("Objective:[/color] Defeat %s. Keep Probe Helper alive." % foe_name)
	checks["preview: Deployment line"] = preview.contains("Deployment:[/color] ambush")
	TestHelpers.cleanup_worlds(path)
	return TestHelpers.report(TAG, "World encounters (%s): every objective message fires; valid entries are clean; npc guest rule; preview lines" % world, checks)

# Shipped encounters pass the objective rules (the new fields do not break existing data).
static func _test_shipped_clean() -> bool:
	var checks: Dictionary = {}
	for e: Dictionary in ContentCatalog.entries("encounters"):
		var errs: Array[String] = EncounterObjectives.problems(ContentSchema.normalize("encounters", e), func(_r: String) -> bool: return true)
		checks["catalog %s (%s)" % [str(e.get("id", "")), "; ".join(errs)]] = errs.is_empty()
	for w: String in WorldCatalog.world_ids():
		for e: Dictionary in WorldCatalog.own_entries(w, "encounters"):
			var errs: Array[String] = EncounterObjectives.problems(WorldSchema.normalize("encounters", e), func(r: String) -> bool: return WorldCatalog.guest_ok(w, r))
			checks["%s:%s (%s)" % [w, str(e.get("id", "")), "; ".join(errs)]] = errs.is_empty()
	return TestHelpers.report(TAG, "Shipped encounters pass the objective rules", checks)
