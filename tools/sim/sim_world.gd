extends RefCounted

# Balance sim world encounters (BAL4-ATTR, docs/specs/DESIGN_REVIEW1.md 3.3): the Ashlands story
# fights in quest order, read from the world data through WorldCatalog (read-only), with the story
# characters mapped to their base catalog units (CAMP1: every foe is a catalog unit at its authored
# level). Loaded with preload (no class_name); nothing is written.

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

# Base unit id of every story character (characters.json base_unit).
static func base_units() -> Dictionary:
	var out: Dictionary = {}
	for c: Dictionary in WorldCatalog.own_entries(WORLD, "characters"):
		out[str(c["id"])] = str(c.get("base_unit", ""))
	return out

# {"enemies": base unit ids, "base": encounter fields, "label", "max_rounds"} of Ashlands encounter
# `enc_id`; like SimScenarios.encounter for a catalog entry, with the roster mapped to base units.
# elevel_set: the foes are level-shifted sandbox copies at `foe_level` (as for catalog encounters).
static func encounter(runner: SimRunner, enc_id: String, foe_level: int, elevel_set: bool) -> Dictionary:
	var entry: Dictionary = {}
	for e: Dictionary in WorldCatalog.own_entries(WORLD, "encounters"):
		if str(e["id"]) == enc_id:
			entry = e
	if entry.is_empty():
		return {}
	var units: Dictionary = base_units()
	var base: Dictionary = {}
	for key: String in ["zone", "arena", "exits", "total_blocks", "unit_ap", "ap_regen", "wave_round", "wave_block", "wave_label"]:
		if entry.has(key):
			base[key] = entry[key]
	var enemies: Array[String] = SimScenarios._catalog_ids(runner, _mapped(DictUtil.array_of(entry.get("enemies", [])), units), foe_level, elevel_set)
	base["wave_units"] = SimScenarios._catalog_ids(runner, _mapped(DictUtil.array_of(entry.get("wave_units", [])), units), foe_level, elevel_set)
	return {"enemies": enemies, "base": base, "label": enc_id, "max_rounds": DictUtil.to_int(entry.get("max_rounds", 0))}

static func _mapped(ids: Array, units: Dictionary) -> Array:
	var out: Array = []
	for raw: Variant in ids:
		out.append(str(units.get(str(raw), str(raw))))
	return out
