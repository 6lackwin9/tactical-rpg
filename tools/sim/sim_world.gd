extends RefCounted

# Balance sim world encounters (BAL4-ATTR, docs/specs/DESIGN_REVIEW1.md 3.3): the Ashlands story
# fights in quest order, read from the world data through WorldCatalog (read-only), with the story
# characters mapped to their base catalog units (CAMP1: every foe is a catalog unit at its authored
# level). BAL5-SIM K1: any world's encounters (own or inherited, WorldCatalog.resolve); a character
# with extras (named items, passives, disciplines, tuning: WorldFactory.divergence) plays as a sandbox
# copy simw_<world>_<id> of its base unit that spawns through WorldFactory.apply_character, as in
# WorldEncounter. Loaded with preload (no class_name); only sandbox units are written.

const SimScenarios = preload("res://tools/sim/sim_scenarios.gd")
const SimRunner = preload("res://tools/sim/sim_runner.gd")

const WORLD: String = "ashlands"
const STEEL_FLAG: String = "ashlands.steel_to_gate"   # the flag of the "bar the gate" choice (variant steel_to_gate)

# The main story quests in order: the auto-start root with no requirements, then every quest that
# (transitively) requires it, in file order (the optional side quests have no such requirement).
static func story_quests() -> Array[Dictionary]:
	var quests: Array[Dictionary] = WorldCatalog.own_entries(WORLD, "quests")
	var in_story: Dictionary = {}
	var out: Array[Dictionary] = []
	for q: Dictionary in quests:
		var reqs: Array = DictUtil.array_of(q.get("requires", []))
		var root: bool = reqs.is_empty() and DictUtil.to_bool(q.get("auto_start", false))
		var chained: bool = false
		for r: Variant in reqs:
			if in_story.has(str(r)):
				chained = true
		if root or chained:
			in_story[str(q["id"])] = true
			out.append(q)
	return out

# Scripted route encounters (Ashlands or any world's routes) gated on `flag` being set: the fights a
# step "reach / set <flag>" triggers on the way, e.g. the Slag Gate road raid of the steel quest.
static func scripted_encounters_for_flag(flag: String) -> Array[String]:
	var out: Array[String] = []
	for world_id: String in WorldCatalog.world_ids():
		for route: Dictionary in WorldCatalog.own_entries(world_id, "routes"):
			for raw: Variant in DictUtil.array_of(route.get("scripted", [])):
				var sc: Dictionary = DictUtil.dict_of(raw)
				for cond: Variant in DictUtil.array_of(sc.get("requires_flags", [])):
					if str(DictUtil.dict_of(cond).get("key", "")) == flag:
						out.append(WorldCatalog.split_qid(str(sc.get("encounter", "")))[-1])
	return out

# The step encounter of quest `q` at step index `i`, with the steel_to_gate variant applied.
static func _step_encounter(q: Dictionary, i: int, steel_to_gate: bool) -> String:
	var steps: Array = DictUtil.array_of(q.get("steps", []))
	var enc: String = str(DictUtil.dict_of(steps[i]).get("encounter", ""))
	for raw: Variant in DictUtil.array_of(q.get("variants", [])):
		var v: Dictionary = DictUtil.dict_of(raw)
		var needs: bool = false
		for cond: Variant in DictUtil.array_of(v.get("when_flags", [])):
			needs = needs or str(DictUtil.dict_of(cond).get("key", "")) == STEEL_FLAG
		if needs and steel_to_gate:
			var step_set: Dictionary = DictUtil.dict_of(DictUtil.dict_of(v.get("set", {})).get("steps", {}))
			var over: Dictionary = DictUtil.dict_of(step_set.get(str(i), {}))
			if over.has("encounter"):
				enc = str(over["encounter"])
	return enc

# Every story fight in order: [{"quest", "encounter", "scripted"}]. A quest step with an encounter
# gives that fight; a step that sets a flag a scripted route fight waits for gives that fight.
static func story_fights(steel_to_gate: bool) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for q: Dictionary in story_quests():
		var steps: Array = DictUtil.array_of(q.get("steps", []))
		for i: int in range(steps.size()):
			var step: Dictionary = DictUtil.dict_of(steps[i])
			var enc: String = _step_encounter(q, i, steel_to_gate)
			if not enc.is_empty():
				out.append({"quest": str(q["id"]), "encounter": enc, "scripted": false})
			var flag: String = str(step.get("flag", ""))
			if not flag.is_empty():
				for raid: String in scripted_encounters_for_flag(flag):
					out.append({"quest": str(q["id"]), "encounter": raid, "scripted": true})
	return out

# {"enemies": unit ids, "base": encounter fields, "label", "max_rounds"} of encounter `enc_id` as seen
# from `world`; like SimScenarios.encounter for a catalog entry, with the roster mapped to units (_ref).
# elevel_set: the foes are level-shifted sandbox copies at `foe_level` (as for catalog encounters).
# full: also the objective, deployment, phases, guests and protect fields (refs mapped to the same unit ids).
static func encounter(runner: SimRunner, enc_id: String, foe_level: int, elevel_set: bool, world: String = WORLD, full: bool = false) -> Dictionary:
	var entry: Dictionary = WorldCatalog.resolve(world, "encounters", enc_id)
	if entry.is_empty():
		return {}
	var ctx: Dictionary = {"runner": runner, "world": world, "level": foe_level, "shifted": elevel_set}
	var base: Dictionary = {}
	for key: String in ["zone", "arena", "exits", "total_blocks", "unit_ap", "ap_regen", "wave_round", "wave_block", "wave_label"]:
		if entry.has(key):
			base[key] = entry[key]
	var enemies: Array[String] = _refs(ctx, DictUtil.array_of(entry.get("enemies", [])))
	base["wave_units"] = _refs(ctx, DictUtil.array_of(entry.get("wave_units", [])))
	if full:
		for key: String in ["objective", "objective_tiles", "objective_rounds", "objective_count", "objective_text", "deployment"]:
			if entry.has(key):
				base[key] = entry[key]
		base["objective_unit"] = _ref(ctx, str(entry.get("objective_unit", "")))
		base["guests"] = _refs(ctx, DictUtil.array_of(entry.get("guests", [])))
		base["protect"] = _refs(ctx, DictUtil.array_of(entry.get("protect", [])))
		base["phases"] = _phases(ctx, DictUtil.array_of(entry.get("phases", [])))
	return {"enemies": enemies, "base": base, "label": enc_id, "max_rounds": DictUtil.to_int(entry.get("max_rounds", 0))}

# The final sandbox/catalog unit id of story reference `id` ("" stays ""): a plain character (or a catalog
# unit id) is its base unit (a level-shifted copy when shifted); a character with extras is the copy
# simw_<world>_<id> of its base unit, defined with SimRunner "character" so it spawns with those extras.
static func _ref(ctx: Dictionary, id: String) -> String:
	if id.is_empty():
		return ""
	var runner: SimRunner = ctx["runner"]
	var world: String = str(ctx["world"])
	var level: int = DictUtil.to_int(ctx["level"])
	var shifted: bool = DictUtil.to_bool(ctx["shifted"])
	var character: Dictionary = WorldCatalog.resolve(world, "characters", id)
	var base_id: String = str(character.get("base_unit", id)) if not character.is_empty() else id
	if character.is_empty() or WorldFactory.divergence(character, world).is_empty():
		return SimScenarios._catalog_ids(runner, [base_id], level, shifted)[0]
	var e: Dictionary = ContentCatalog.get_entry("units", base_id)
	var copy: String = "simw_%s" % str(character.get("_qid", id)).replace(":", "_")
	runner.define(copy, str(e.get("class_id", "")), level if shifted else DictUtil.to_int(e.get("level", 1), 1), str(e.get("tier", "standard")), false,
		{"copy_of": base_id, "character": [world, id]})
	return copy

static func _refs(ctx: Dictionary, ids: Array) -> Array[String]:
	var out: Array[String] = []
	for raw: Variant in ids:
		out.append(_ref(ctx, str(raw)))
	return out

# Phases with every unit reference (`unit`, `units`) mapped like the roster.
static func _phases(ctx: Dictionary, phases: Array) -> Array:
	var out: Array = []
	for raw: Variant in phases:
		out.append(_map_node(ctx, raw))
	return out

static func _map_node(ctx: Dictionary, node: Variant) -> Variant:
	if node is Dictionary:
		var d: Dictionary = node
		var res: Dictionary = {}
		for k: Variant in d:
			var v: Variant = d[k]
			if str(k) == "unit" and v is String:
				res[k] = _ref(ctx, str(v))
			elif str(k) == "units" and v is Array:
				res[k] = _refs(ctx, DictUtil.array_of(v))
			else:
				res[k] = _map_node(ctx, v)
		return res
	if node is Array:
		var arr: Array = node
		var res_a: Array = []
		for v: Variant in arr:
			res_a.append(_map_node(ctx, v))
		return res_a
	return node
