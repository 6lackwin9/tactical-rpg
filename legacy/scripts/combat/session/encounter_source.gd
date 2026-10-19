class_name EncounterSource
extends RefCounted

# ==============================================================================
# Tactical RPG - Encounter Source (every battle the game can run)
# One place that lists and creates encounters from both layers:
#   "goblin_ambush"       a content-catalog encounter (lore-free base battle)
#   "ashlands:skirmish"   a world encounter, played in that world (its characters)
# Used by the Scene Runner, the arena's --encounter argument and the World Forge.
# ==============================================================================

static func exists(id: String) -> bool:
	if id.contains(":"):
		var parts: Array[String] = WorldCatalog.split_qid(id)
		return parts.size() == 2 and WorldCatalog.has_entry(parts[0], "encounters", parts[1])
	return ContentCatalog.has("encounters", id)

static func create(id: String) -> CombatEncounter:
	if id.contains(":"):
		return WorldEncounter.create(id)
	return CatalogEncounter.new(id)

# [{"id", "label", "world"}]: catalog encounters, then every world's encounters
# (inherited ones are offered too: they play with that world's lore).
static func options() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for id: String in ContentCatalog.ids("encounters"):
		out.append({"id": id, "label": str(ContentCatalog.get_entry("encounters", id).get("name", id)), "world": ""})
	for w: String in WorldCatalog.world_ids():
		var world_name: String = str(WorldCatalog.manifest(w).get("name", w))
		for e: Dictionary in WorldCatalog.merged_entries(w, "encounters"):
			var inherited: String = "" if str(e["_world"]) == w else " (from %s)" % str(e["_world"])
			out.append({"id": "%s:%s" % [w, str(e.get("id", ""))], "label": "[%s] %s%s" % [world_name, str(e.get("name", "")), inherited], "world": w})
	return out
