---
class: ObjectiveTestSuite
extends: RefCounted
description: "OBJ1-L: encounter objectives, guests and deployment (EncounterObjectives). Stage 1 (schema): the nine fields and their defaults in both schemas, old entries normalising to rout / standard / no guests, every validation message of spec 1.1 in both catalogs, a full valid entry per objective clean in both catalogs, the generated text (rule 9) and the World Forge preview lines (rule 12). Fixture refs are picked by rule (side / role / terrain); world writes go to a sandbox fork only. Nothing depends on shipped numbers. Stage 2 (behaviour, spec 1.2, 1.3 and section 2): the deployment table (standard identical to the frozen GRID12 reference for every shipped encounter, ambush split, centre ring, arrivals), guests (built after the heroes, Balanced drafts, no hero key, no XP, not in PartyState), the verdict (hold counting, escape, slay, protect, every hero down, round limit), objective_state and reinforcements restored by reset_round, the outcome readers and a recorded rout replay. Session tests use IdleEncounter (nobody plans) and place units by hand, so no result depends on AI or balance."
---

# Properties
* `TAG`: String = "Objective" (const)
* `MAX_ROUNDS`: int = 6 (const)
* `MAX_STEPS`: int = 600 (const)
* `drafts`: Dictionary = {}
* `break_on`: String = ""
* `list`: Array = drafts.get(c.name, [])
* `out`: Array[PlanStep] = super(c, preset_id, shared)
* `unknown`: Variant
* `bad`: Array[PlanStep] = [PlanStep.self_skill(unknown)]
* `FIELDS`: Dictionary = { (const)

# Methods

### `schedule_round(_resolver: CombatResolver, _round_num: int) -> void`
```logic
	pass
```

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING OBJECTIVE TEST (OBJ1-L)                 ===")
print("=======================================================")
var ok: bool = true
ok = _test_schema_defaults() and ok
ok = _test_generated_text() and ok
ok = _test_catalog_validation() and ok
ok = _test_world_validation() and ok
ok = _test_shipped_clean() and ok
ok = _test_duplicate_guest() and ok
ok = _test_standard_unchanged() and ok
ok = _test_ambush_and_centre() and ok
ok = _test_hold_counting() and ok
ok = _test_hold_win_and_reset() and ok
ok = _test_wave_reset() and ok
ok = _test_escape() and ok
ok = _test_slay() and ok
ok = _test_guests_and_protect() and ok
ok = _test_game_guest() and ok
ok = _test_autoplay_guest_redraft() and ok
ok = _test_recorded_rout() and ok
return ok
```

### `static _catalog_refs() -> Dictionary`
```logic
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
```

### `static _world_refs() -> Dictionary`
```logic
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
```

### `static _blocked_tile() -> Dictionary`
```logic
for zone: String in CombatZones.zone_ids():
	var terrain: Dictionary = CombatZones.terrain_map(zone)
	for cell: Variant in terrain:
		var raw: Variant = terrain[cell]
		var t: Terrain.Type = raw if raw is Terrain.Type else Terrain.Type.FLOOR
		if not Terrain.is_walkable(t):
			return {"zone": zone, "cell": cell}
return {}
```

### `static _entry(refs: Dictionary, changes: Dictionary) -> Dictionary`
```logic
var e: Dictionary = {
	"id": "objective_probe", "name": "Objective Probe", "zone": "stone_arena", "max_rounds": MAX_ROUNDS,
	"party": [refs["party"]], "enemies": [refs["a"]],
}
e.merge(changes, true)
return e
```

### `static _good_cases(refs: Dictionary) -> Dictionary`
```logic
return {
	"rout with a protected guest, ambush": {"objective": "rout", "deployment": "ambush", "guests": [refs["guest"]], "protect": [refs["guest"]]},
	"hold on the ring": {"objective": "hold", "objective_rounds": MAX_ROUNDS, "deployment": "centre"},
	"hold on own tiles": {"objective": "hold", "objective_tiles": [[4, 4], [5, 3]], "objective_rounds": 1, "objective_text": "x".repeat(EncounterObjectives.MAX_TEXT)},
	"escape": {"objective": "escape", "objective_count": 4},
	"escape in an open arena": {"objective": "escape", "arena": "OPEN", "exits": []},
	"slay": {"objective": "slay", "objective_unit": refs["a"]},
}
```

### `static _bad_cases(refs: Dictionary) -> Dictionary`
```logic
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
```

### `static _matrix(refs: Dictionary, validate: Callable) -> Dictionary`
```logic
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
```

### `static _test_schema_defaults() -> bool`
```logic
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
```

### `static _test_generated_text() -> bool`
```logic
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
```

### `static _test_catalog_validation() -> bool`
```logic
var refs: Dictionary = _catalog_refs()
if refs.is_empty():
	return TestHelpers.report_ok(TAG, "Catalog validation", false, "no party / enemy units in the catalog")
var checks: Dictionary = _matrix(refs, func(e: Dictionary) -> Array[String]: return ContentCatalog.validate_entry("encounters", e))
return TestHelpers.report(TAG, "Catalog encounters: every objective message fires on a bad entry; valid entries are clean", checks)
```

### `static _test_world_validation() -> bool`
```logic
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
```

### `static _test_shipped_clean() -> bool`
```logic
var checks: Dictionary = {}
for e: Dictionary in ContentCatalog.entries("encounters"):
	var errs: Array[String] = EncounterObjectives.problems(ContentSchema.normalize("encounters", e), func(_r: String) -> bool: return true)
	checks["catalog %s (%s)" % [str(e.get("id", "")), "; ".join(errs)]] = errs.is_empty()
for w: String in WorldCatalog.world_ids():
	for e: Dictionary in WorldCatalog.own_entries(w, "encounters"):
		var errs: Array[String] = EncounterObjectives.problems(WorldSchema.normalize("encounters", e), func(r: String) -> bool: return WorldCatalog.guest_ok(w, r))
		checks["%s:%s (%s)" % [w, str(e.get("id", "")), "; ".join(errs)]] = errs.is_empty()
return TestHelpers.report(TAG, "Shipped encounters pass the objective rules", checks)
```

### `static _reach_refs() -> Dictionary`
```logic
var out: Dictionary = {}
for u: Dictionary in ContentCatalog.entries("units"):
	if str(u.get("side", "")) != "party":
		continue
	var c: Combatant = ContentFactory.unit(u, "probe", 12)
	var key: String = "melee" if c.attack_range <= 1 else "ranged"
	if not out.has(key):
		out[key] = str(u["id"])
return out
```

### `static _idle(entry: Dictionary) -> CombatSession`
```logic
var enc := IdleEncounter.new()
enc.encounter_id = str(entry.get("id", ""))
enc.apply_definition(entry)
var s := CombatSession.new(enc)
s.start()
return s
```

### `static _enc(s: CombatSession) -> CatalogEncounter`
```logic
return s.encounter as CatalogEncounter
```

### `static _put(s: CombatSession, c: Combatant, cell: Vector2i) -> void`
```logic
s.grid.combatant_positions[c] = cell
```

### `static _run_block(s: CombatSession) -> void`
```logic
var b: int = s.current_block
var r: int = s.current_round
for i: int in range(MAX_STEPS):
	s.step_next()
	if s.phase == CombatSession.Phase.REPLAY or s.current_block != b or s.current_round != r:
		return
```

### `static _play(s: CombatSession) -> bool`
```logic
if not s.confirm_plan(false):
	return false
TestHelpers.resolve_until_replay(s, MAX_STEPS)
return s.phase == CombatSession.Phase.REPLAY
```

### `static _held(s: CombatSession) -> int`
```logic
return _enc(s).objective_runtime.held
```

### `static _rt_entry(refs: Dictionary, changes: Dictionary) -> Dictionary`
```logic
var e: Dictionary = _entry(refs, changes)
e["id"] = "objective_rt"
return e
```

### `static _legacy_deploy(g: CombatGrid, allies: Array[Combatant], foes: Array[Combatant]) -> void`
```logic
_legacy_side(g, allies, 5, 4, -1)
_legacy_side(g, foes, 7, 8, 1)
```

### `static _legacy_side(g: CombatGrid, units: Array[Combatant], front_x: int, back_x: int, back_dir: int) -> void`
```logic
for c: Combatant in units:
	var columns: Array[int] = [front_x, back_x]
	if c.attack_range > 1:
		columns = [back_x, front_x]
	var x: int = back_x + back_dir
	while g.is_valid_coord(Vector2i(x, 0)) and CombatGrid.is_party_half(Vector2i(x, 0)) == CombatGrid.is_party_half(Vector2i(front_x, 0)):
		columns.append(x)
		x += back_dir
	for col: int in columns:
		if _legacy_column(g, c, col):
			break
```

### `static _legacy_column(g: CombatGrid, c: Combatant, x: int) -> bool`
```logic
for y: int in [4, 3, 5, 2, 6, 1, 7]:
	var cell := Vector2i(x, y)
	if g.is_walkable(cell) and not g.is_exit_tile(cell) and not g.is_occupied(cell):
		return g.place_combatant(c, cell)
return false
```

### `static _legacy_leftovers(g: CombatGrid, units: Array[Combatant], party_side: bool) -> void`
```logic
for c: Combatant in units:
	if g.has_combatant(c):
		continue
	var best: Vector2i = Vector2i(-1, -1)
	var best_score: int = 1 << 30
	for y: int in range(CombatGrid.GRID_H):
		for x: int in range(CombatGrid.GRID_W):
			var cell := Vector2i(x, y)
			if CombatGrid.is_party_half(cell) != party_side or not g.is_walkable(cell) or g.is_exit_tile(cell) or g.is_occupied(cell):
				continue
			var score: int = CombatGrid.centre_distance(cell) * 1000 + y * 20 + x
			if score < best_score:
				best_score = score
				best = cell
	if best != Vector2i(-1, -1):
		g.place_combatant(c, best)
```

### `static _compare_legacy(enc: CatalogEncounter) -> String`
```logic
enc.build()
var legacy := CombatGrid.new(enc.grid.arena_type, enc.grid.designated_exits)
legacy.terrain = enc.grid.terrain
var deployed: Array[Combatant] = []
for e: Combatant in enc.enemies:
	if not enc.wave.has(e) and not enc.is_reserved(e):
		deployed.append(e)
_legacy_deploy(legacy, enc.allies, deployed)
_legacy_leftovers(legacy, enc.allies, true)
_legacy_leftovers(legacy, deployed, false)
var batches: Array[Array] = [enc.wave]
for p: Dictionary in enc.phase_runtime.phases:
	batches.append(enc.phase_runtime.units_of(str(p["id"])))
for raw: Array in batches:
	var batch: Array[Combatant] = []
	batch.assign(raw)
	enc._place_leftovers(batch, false)
	_legacy_leftovers(legacy, batch, false)
for c: Combatant in enc.allies + enc.enemies:
	if enc.grid.get_position(c) != legacy.get_position(c):
		return "%s at %s, GRID12 %s" % [c.name, str(enc.grid.get_position(c)), str(legacy.get_position(c))]
return ""
```

### `static _test_duplicate_guest() -> bool`
```logic
var refs: Dictionary = _catalog_refs()
if refs.is_empty():
	return TestHelpers.report_ok(TAG, "Duplicate guest", false, "no party / enemy units in the catalog")
var g: String = str(refs["guest"])
var errors: Array[String] = ContentCatalog.validate_entry("encounters", _entry(refs, {"guests": [g, g]}))
var checks: Dictionary = {}
checks["a guest listed twice is refused (%s)" % "; ".join(errors)] = TestHelpers.any_contains(errors, "Guest '%s' is listed twice." % g)
var one: Array[String] = ContentCatalog.validate_entry("encounters", _entry(refs, {"guests": [g]}))
checks["a single guest is clean (%s)" % "; ".join(one)] = one.is_empty()
return TestHelpers.report(TAG, "Guests: a repeated id is refused", checks)
```

### `static _test_standard_unchanged() -> bool`
```logic
var checks: Dictionary = {}
for e: Dictionary in ContentCatalog.entries("encounters"):
	if str(ContentSchema.normalize("encounters", e).get("deployment", "")) != EncounterObjectives.STANDARD:
		continue
	var diff: String = _compare_legacy(CatalogEncounter.new(str(e["id"])))
	checks["catalog %s (%s)" % [str(e["id"]), diff]] = diff.is_empty()
for w: String in WorldCatalog.world_ids():
	for e: Dictionary in WorldCatalog.merged_entries(w, "encounters"):
		if str(WorldSchema.normalize("encounters", e).get("deployment", "")) != EncounterObjectives.STANDARD:
			continue
		var diff: String = _compare_legacy(WorldEncounter.new(w, str(e["id"])))
		checks["%s:%s (%s)" % [w, str(e["id"]), diff]] = diff.is_empty()
checks["at least one shipped encounter compared"] = not checks.is_empty()
return TestHelpers.report(TAG, "Standard deployment places every shipped unit on its GRID12 tile", checks)
```

### `static _test_ambush_and_centre() -> bool`
```logic
var checks: Dictionary = {}
var refs: Dictionary = _catalog_refs()
var reach: Dictionary = _reach_refs()
if refs.is_empty() or not reach.has("melee") or not reach.has("ranged"):
	return TestHelpers.report_ok(TAG, "Ambush and centre", false, "no melee and ranged party units")
var a: String = str(refs["a"])
var amb := CatalogEncounter.from_entry(_rt_entry(refs, {"deployment": "ambush", "enemies": [a, a, a, a], "wave_units": [a, a], "wave_round": 1, "wave_block": 1}))
amb.build()
var front: int = 0
var behind: int = 0
var deployed: Array[Combatant] = []
for e: Combatant in amb.enemies:
	if amb.wave.has(e):
		continue
	deployed.append(e)
	var x: int = amb.grid.get_position(e).x
	front += 1 if x == 7 or x == 8 else 0
	behind += 1 if x == 1 or x == 2 else 0
checks["ambush: two of four on x 7 or 8 (%d), two on x 1 or 2 (%d)" % [front, behind]] = deployed.size() == 4 and front == 2 and behind == 2
checks["ambush: roster order splits even to A, odd to B"] = deployed.size() == 4 and amb.grid.get_position(deployed[0]).x >= 7 and amb.grid.get_position(deployed[1]).x <= 2
amb._place_leftovers(amb.wave, false)
var w0: Vector2i = amb.grid.get_position(amb.wave[0])
var w1: Vector2i = amb.grid.get_position(amb.wave[1])
checks["ambush wave alternates A then B (%s, %s)" % [str(w0), str(w1)]] = not CombatGrid.is_party_half(w0) and CombatGrid.is_party_half(w1) and w1.x <= 3
var cen := CatalogEncounter.from_entry(_rt_entry(refs, {"deployment": "centre", "party": [reach["ranged"], reach["melee"]], "enemies": [a, a], "guests": [reach["melee"]]}))
cen.build()
var heroes: Array[Combatant] = cen.heroes()
var melee: Combatant = heroes[1] if heroes.size() == 2 else null
var ranged: Combatant = heroes[0] if heroes.size() == 2 else null
checks["centre: melee hero on (5, 4)"] = melee != null and cen.grid.get_position(melee) == Vector2i(5, 4)
checks["centre: ranged hero on (6, 4)"] = ranged != null and cen.grid.get_position(ranged) == Vector2i(6, 4)
var guest: Combatant = cen.guests[0] if cen.guests.size() == 1 else null
checks["centre: the guest deploys after the heroes on a party column"] = guest != null and cen.allies.back() == guest and cen.grid.get_position(guest) == Vector2i(5, 3)
checks["centre: enemies on group fronts (A x 8, B x 3)"] = cen.grid.get_position(cen.enemies[0]).x == 8 and cen.grid.get_position(cen.enemies[1]).x == 3
return TestHelpers.report(TAG, "Ambush splits the enemies front and behind; centre puts the party on the ring", checks)
```

### `static _test_hold_counting() -> bool`
```logic
var checks: Dictionary = {}
var refs: Dictionary = _catalog_refs()
if refs.is_empty():
	return TestHelpers.report_ok(TAG, "Hold counting", false, "no fixture refs")
var s: CombatSession = _idle(_rt_entry(refs, {"objective": "hold", "objective_rounds": MAX_ROUNDS, "max_rounds": MAX_ROUNDS}))
var hero: Combatant = s.allies[0]
var foe: Combatant = s.enemies[0]
var off_hero := Vector2i(3, 4)
var off_foe := Vector2i(9, 4)
_put(s, foe, off_foe)
_put(s, hero, Vector2i(5, 4))
checks["R1 played"] = _play(s)
checks["R1 hero alone on (5, 4): held 1 (%d)" % _held(s)] = _held(s) == 1 and s.outcome.is_empty()
s.advance_round()
_put(s, hero, off_hero)
checks["R2 confirmed"] = s.confirm_plan(false)
_run_block(s)
_put(s, hero, Vector2i(5, 4))
_run_block(s)
checks["mid-round on the ring counts nothing yet (%d)" % _held(s)] = _held(s) == 1
_put(s, hero, off_hero)
TestHelpers.resolve_until_replay(s, MAX_STEPS)
checks["R2 on the ring mid-round, off at the end: unchanged (%d)" % _held(s)] = _held(s) == 1
s.advance_round()
checks["R3 played"] = _play(s)
checks["R3 empty ring: unchanged (%d)" % _held(s)] = _held(s) == 1
s.advance_round()
_put(s, hero, Vector2i(5, 4))
_put(s, foe, Vector2i(6, 4))
checks["R4 played"] = _play(s)
checks["R4 both sides on the ring: unchanged (%d)" % _held(s)] = _held(s) == 1
s.advance_round()
_put(s, hero, off_hero)
checks["R5 played"] = _play(s)
checks["R5 an enemy alone on (6, 4): held - 1 (%d)" % _held(s)] = _held(s) == 0
s.advance_round()
checks["R6 played"] = _play(s)
checks["R6 enemy alone again: never below 0 (%d)" % _held(s)] = _held(s) == 0
checks["round limit with both sides standing: no verdict, the bridge says fled (X1)"] = s.outcome.is_empty() and not s.combat_over and BattleBridge.outcome(s) == "fled"
checks["the playtest end is the timeout"] = str(PlaytestRecord.end_of(s, MAX_ROUNDS).get("result", "")) == "timeout"
s.dispose()
return TestHelpers.report(TAG, "Hold counts only at round ends: +1 alone, -1 for an enemy alone (floor 0), empty or both unchanged", checks)
```

### `static _test_hold_win_and_reset() -> bool`
```logic
var checks: Dictionary = {}
var refs: Dictionary = _catalog_refs()
if refs.is_empty():
	return TestHelpers.report_ok(TAG, "Hold win", false, "no fixture refs")
var rounds: int = 3
var s: CombatSession = _idle(_rt_entry(refs, {"objective": "hold", "objective_rounds": rounds, "max_rounds": rounds}))
var hero: Combatant = s.allies[0]
_put(s, s.enemies[0], Vector2i(9, 4))
_put(s, hero, Vector2i(5, 4))
for r: int in range(1, rounds):
	checks["R%d played" % r] = _play(s)
	checks["R%d: held %d, battle goes on" % [r, r]] = _held(s) == r and s.outcome.is_empty() and not s.combat_over
	s.advance_round()
checks["round-start snapshot keeps held"] = s.snapshot_at(rounds, 0) != null and DictUtil.to_int(s.snapshot_at(rounds, 0).objective_state.get("held", -1)) == rounds - 1
checks["R%d played" % rounds] = _play(s)
checks["held reaches %d on the last round: won (%s)" % [rounds, s.outcome]] = s.outcome == "won" and s.combat_over and _held(s) == rounds
checks["the bridge and the playtest end read won"] = BattleBridge.outcome(s) == "won" and str(PlaytestRecord.end_of(s, rounds).get("result", "")) == "won"
checks["the round-end snapshot keeps held"] = DictUtil.to_int(s.snapshot_at(rounds, s.total_blocks).objective_state.get("held", -1)) == rounds
s.reset_round()
checks["reset_round restores held and clears the outcome (%d)" % _held(s)] = _held(s) == rounds - 1 and s.outcome.is_empty() and not s.combat_over
_put(s, hero, Vector2i(3, 4))
checks["redo played"] = _play(s)
checks["the redo off the ring: no win, round limit (%d)" % _held(s)] = _held(s) == rounds - 1 and s.outcome.is_empty() and BattleBridge.outcome(s) == "fled"
s.dispose()
return TestHelpers.report(TAG, "Hold 3: three round ends alone on the ring win on the last round; reset_round restores held", checks)
```

### `static _test_wave_reset() -> bool`
```logic
var checks: Dictionary = {}
var refs: Dictionary = _catalog_refs()
if refs.is_empty():
	return TestHelpers.report_ok(TAG, "Wave reset", false, "no fixture refs")
var s: CombatSession = _idle(_rt_entry(refs, {"wave_units": [refs["a"]], "wave_round": 2, "wave_block": 2}))
var wave_unit: Combatant = _enc(s).wave[0] if not _enc(s).wave.is_empty() else null
checks["R1 played"] = _play(s)
s.advance_round()
checks["R2 played"] = _play(s)
checks["the wave entered in round 2"] = wave_unit != null and s.grid.has_combatant(wave_unit) and s.encounter.reinforcements_spawned
s.reset_round()
checks["reset_round: the wave is off the field and not spawned"] = wave_unit != null and not s.grid.has_combatant(wave_unit) and not s.encounter.reinforcements_spawned
checks["redo played"] = _play(s)
checks["the redo brings the wave back"] = wave_unit != null and s.grid.has_combatant(wave_unit) and s.encounter.reinforcements_spawned
s.dispose()
return TestHelpers.report(TAG, "reset_round restores reinforcements_spawned: the wave re-enters on the redo", checks)
```

### `static _test_escape() -> bool`
```logic
var checks: Dictionary = {}
var refs: Dictionary = _catalog_refs()
if refs.is_empty():
	return TestHelpers.report_ok(TAG, "Escape", false, "no fixture refs")
var exit := Vector2i(0, 4)
var base: Dictionary = {"objective": "escape", "objective_count": 1, "exits": [[exit.x, exit.y]], "party": [refs["party"], refs["party"]]}
for with_guest: bool in [false, true]:
	var changes: Dictionary = base.duplicate(true)
	if with_guest:
		changes.merge({"guests": [refs["guest"]], "protect": [refs["guest"]]})
	var s: CombatSession = _idle(_rt_entry(refs, changes))
	var heroes: Array[Combatant] = s.encounter.heroes()
	checks["%s: exit tile walkable" % str(with_guest)] = s.grid.is_walkable(exit) and s.grid.is_exit_tile(exit)
	_put(s, heroes[0], exit + Vector2i(1, 0))
	s.grid.move_combatant(heroes[0], Vector2i(-1, 0))
	heroes[1].hp = 0
	checks["%s: confirmed" % str(with_guest)] = s.confirm_plan(false)
	_run_block(s)
	if with_guest:
		checks["a protect guest left on the field: not won, fled (%s)" % s.outcome] = s.outcome == "fled"
		checks["fled without a note reads 'The party withdrew.' (%s | %s)" % [s.phase_text, s.status_text]] = s.phase_text.contains("BATTLE LEFT") and s.status_text.contains(CatalogEncounter.FLED_TEXT)
	else:
		checks["one hero out by the exit, the other down: won (%s)" % s.outcome] = s.outcome == "won" and s.combat_over
		checks["the escape win with nobody left on the field reads victory (%s)" % s.phase_text] = s.phase_text.contains("VICTORY") and not s.phase_text.contains("DEFEAT")
	s.dispose()
return TestHelpers.report(TAG, "Escape 1: a hero out with the others down wins; not while a protect guest stays", checks)
```

### `static _test_slay() -> bool`
```logic
var checks: Dictionary = {}
var refs: Dictionary = _catalog_refs()
if refs.is_empty():
	return TestHelpers.report_ok(TAG, "Slay", false, "no fixture refs")
var a: String = str(refs["a"])
for flee: bool in [false, true]:
	var s: CombatSession = _idle(_rt_entry(refs, {"objective": "slay", "objective_unit": a, "enemies": [a, a]}))
	var target: Combatant = _enc(s).unit(a, 0)
	var other: Combatant = _enc(s).unit(a, 1)
	if flee:
		_put(s, target, Vector2i(10, 4))   # next to the default exit (11, 4)
		s.grid.move_combatant(target, Vector2i(1, 0))
	else:
		target.hp = 0
	checks["%s: confirmed" % str(flee)] = s.confirm_plan(false)
	_run_block(s)
	if flee:
		var note: String = "%s got away." % target.name
		checks["the target withdraws: fled, '%s' (%s; %s)" % [note, s.outcome, s.status_text]] = s.outcome == "fled" and s.status_text.contains(note) and target.is_withdrawn
	else:
		checks["the target down, another enemy standing: won (%s)" % s.outcome] = s.outcome == "won" and other != null and other.is_active()
	s.dispose()
return TestHelpers.report(TAG, "Slay: the unit down wins with others standing; the unit getting away is fled", checks)
```

### `static _test_guests_and_protect() -> bool`
```logic
var checks: Dictionary = {}
var refs: Dictionary = _catalog_refs()
if refs.is_empty():
	return TestHelpers.report_ok(TAG, "Guests", false, "no fixture refs")
var g: String = str(refs["guest"])
var s: CombatSession = _idle(_rt_entry(refs, {"guests": [g], "protect": [g]}))
var enc: CombatEncounter = s.encounter
var guest: Combatant = enc.guests[0] if enc.guests.size() == 1 else null
var hero: Combatant = enc.heroes()[0] if enc.heroes().size() == 1 else null
checks["the guest is built after the hero, on the party side"] = guest != null and hero != null and s.allies == ([hero, guest] as Array[Combatant]) and s.grid.has_combatant(guest)
checks["the guest has no hero key (never saved)"] = guest != null and enc.hero_key(guest).is_empty() and not enc.hero_key(hero).is_empty()
checks["AI party drafts give the guest Balanced"] = enc.party_preset_for(guest, "aggressive") == CombatEncounter.GUEST_PRESET and enc.party_preset_for(hero, "aggressive") == "aggressive"
s.plan_apply_preset_all("aggressive")
checks["preset_all drafts the guest too"] = guest != null and s.plan.is_valid(guest)
var auto := CombatAutoplay.new(s)
checks["autoplay refuses a tactic for the guest and saves none"] = not auto.set_hero_tactic(guest, "aggressive") and not auto.has_override(guest) and DictUtil.dict_of(auto.settings().get("heroes", {})).is_empty()
checks["autoplay still takes a tactic for the hero"] = auto.set_hero_tactic(hero, "defensive") and auto.has_override(hero)
auto.set_hero_tactic(hero, auto.get_party_tactic())
auto.detach()
s.plan_clear(guest)
s.plan_clear(hero)
guest.hp = 0
checks["confirmed"] = s.confirm_plan(false)
_run_block(s)
checks["a protect guest down with the hero standing: lost (%s)" % s.outcome] = s.outcome == "lost" and hero.is_active() and BattleBridge.outcome(s) == "lost"
checks["lost reads defeat (%s)" % s.phase_text] = s.phase_text.contains("DEFEAT")
s.dispose()
var t: CombatSession = _idle(_rt_entry(refs, {"guests": [g]}))
t.encounter.heroes()[0].hp = 0
checks["confirmed (unprotected guest)"] = t.confirm_plan(false)
_run_block(t)
checks["every hero down while a guest stands: lost (%s)" % t.outcome] = t.outcome == "lost" and t.encounter.guests[0].is_active()
t.dispose()
return TestHelpers.report(TAG, "Guests: after the heroes, Balanced, no key; protect down or every hero down is lost", checks)
```

### `static _test_game_guest() -> bool`
```logic
var refs: Dictionary = _world_refs()
var crefs: Dictionary = _catalog_refs()
if refs.is_empty() or crefs.is_empty():
	return TestHelpers.report_ok(TAG, "Game guest", false, "no world with party and enemy characters")
var world: String = str(refs["world"])
var path: String = TestHelpers.sandbox("objective_game")
if not TestHelpers.fork_worlds(path):
	TestHelpers.cleanup_worlds(path)
	return TestHelpers.report_ok(TAG, "Game guest", false, "world fork failed")
var checks: Dictionary = {}
var helper: Array[String] = WorldCatalog.upsert(world, "characters", {"id": "obj_game_helper", "name": "Game Helper", "role": "npc", "base_unit": crefs["guest"]})
var stored: Array[String] = WorldCatalog.upsert(world, "encounters", _entry(refs, {"id": "objective_game", "guests": ["obj_game_helper"]}))
checks["fixtures stored (%s | %s)" % ["; ".join(helper), "; ".join(stored)]] = helper.is_empty() and stored.is_empty()
var made: Dictionary = NewGame.create(world, [str(crefs["party"])], 1)
var gs: GameState = GameState.of(made.get("state", null))
var built: Dictionary = BattleBridge.encounter_for(gs, "objective_game") if gs != null else {}
var raw_enc: Variant = built.get("encounter", null)
if gs == null or not (raw_enc is GameEncounter):
	TestHelpers.cleanup_worlds(path)
	checks["game and encounter built (%s)" % str(built.get("errors", made.get("errors", [])))] = false
	return TestHelpers.report(TAG, "Game guest", checks)
var enc: GameEncounter = raw_enc
var members: int = gs.party.members.size()
var s := CombatSession.new(enc)
s.start()
var guest: Combatant = enc.guests[0] if enc.guests.size() == 1 else null
checks["the saved party plus one guest fight"] = guest != null and s.allies.size() == gs.party.active_members().size() + 1 and s.allies.back() == guest
checks["the guest has no hero key"] = guest != null and enc.hero_key(guest).is_empty()
for e: Combatant in s.enemies:
	e.hp = 0
checks["confirmed"] = s.confirm_plan(false)
_run_block(s)
checks["won (%s)" % s.outcome] = s.outcome == "won" and BattleBridge.outcome(s) == "won"
var res: Dictionary = BattleBridge.apply(gs, "objective_game", "won")
var xp: Dictionary = DictUtil.dict_of(res.get("xp", {}))
var only_members: bool = not xp.is_empty()
for key: Variant in xp:
	only_members = only_members and gs.party.member(str(key)) != null
checks["XP only for party members (%s)" % str(xp.keys())] = only_members
var guest_in_party: bool = false
for m: UnitState in gs.party.members:
	guest_in_party = guest_in_party or m.id == "obj_game_helper" or m.lore_ref.ends_with("obj_game_helper")
checks["the guest is not in PartyState"] = gs.party.members.size() == members and not guest_in_party
s.dispose()
TestHelpers.cleanup_worlds(path)
return TestHelpers.report(TAG, "Game battle with a guest: fights beside the party, no XP, never written back", checks)
```

### `static _test_recorded_rout() -> bool`
```logic
var checks: Dictionary = {}
var id: String = ""
for e: Dictionary in ContentCatalog.entries("encounters"):
	var n: Dictionary = ContentSchema.normalize("encounters", e)
	if str(n["objective"]) == EncounterObjectives.ROUT and DictUtil.array_of(n["guests"]).is_empty() and not DictUtil.array_of(n["party"]).is_empty():
		id = str(e["id"])
		break
if id.is_empty():
	return TestHelpers.report_ok(TAG, "Recorded rout", false, "no rout encounter in the catalog")
var path: String = TestHelpers.sandbox("objective_rec")
var saved_enabled: int = PlaytestStore.force_enabled
PlaytestStore.force_enabled = 1
PlaytestStore.use_root(path.path_join("rec"))
DirAccess.make_dir_recursive_absolute(path.path_join("rec"))
var rec: PlaytestRecorder = PlaytestRecorder.open("user")
var s := CombatSession.new(EncounterSource.create(id))
rec.attach(s)
s.start()
for r: int in range(3):
	s.plan_apply_preset_all("balanced")
	if not s.confirm_plan(false):
		break
	TestHelpers.resolve_until_replay(s, MAX_STEPS)
	if s.combat_over or not s.can_advance_round():
		break
	s.advance_round()
rec.flush()
rec.detach()
s.dispose()
var data: Dictionary = PlaytestRecord.normalize(DictUtil.dict_of(PlaytestStore.load_json(rec.path).get("data", {})))
var battle: Dictionary = PlaytestRecord.battle_at(data, 0)
var rp: PlaytestReplayer = PlaytestReplayer.for_battle(battle)
checks["replay of %s starts" % id] = rp.run()
checks["every round matches (%s)" % str(rp.first_problem().get("detail", ""))] = not rp.rounds.is_empty() and rp.all_match()
if rp.session != null:
	rp.session.dispose()
PlaytestStore.force_enabled = saved_enabled
PlaytestStore.use_root(PlaytestStore.USER_ROOT)
TestHelpers.remove_sandbox(path)
return TestHelpers.report(TAG, "A recorded rout battle replays identically", checks)
```

### `static _test_autoplay_guest_redraft() -> bool`
```logic
var checks: Dictionary = {}
var refs: Dictionary = _catalog_refs()
if refs.is_empty():
	return TestHelpers.report_ok(TAG, "Autoplay guest redraft", false, "no fixture refs")
var enc := DraftSpyEncounter.new()
var entry: Dictionary = _rt_entry(refs, {"guests": [refs["guest"]]})
enc.encounter_id = str(entry["id"])
enc.apply_definition(entry)
var s := CombatSession.new(enc)
s.start()
var hero: Combatant = enc.heroes()[0]
var guest: Combatant = enc.guests[0]
var auto := CombatAutoplay.new(s)
checks["party tactic set"] = auto.set_party_tactic("aggressive")
checks["hero override set"] = auto.set_hero_tactic(hero, "defensive")
enc.break_on = "defensive"
enc.drafts.clear()
auto._draft_round()
checks["fixture: the hero's override broke the guest row before the redraft"] = DictUtil.array_of(enc.drafts.get(hero.name, [])).has("defensive")
var guest_presets: Array = enc.drafts.get(guest.name, [])
checks["the guest row was redrafted after the override (%s)" % str(guest_presets)] = guest_presets.size() >= 2
var only_balanced: bool = not guest_presets.is_empty()
for p: Variant in guest_presets:
	only_balanced = only_balanced and str(p) == CombatEncounter.GUEST_PRESET
checks["every guest draft is Balanced (%s)" % str(guest_presets)] = only_balanced
checks["the hero drafts follow its tactics (%s)" % str(enc.drafts.get(hero.name, []))] = DictUtil.array_of(enc.drafts.get(hero.name, [])).has("defensive")
auto.detach()
s.dispose()
return TestHelpers.report(TAG, "Autoplay redrafts a broken guest row with Balanced (rule 2)", checks)
```

