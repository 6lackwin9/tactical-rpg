class_name WorldEncounter
extends CatalogEncounter

# ==============================================================================
# Tactical RPG - World Encounter (a battle of a lore world, played in that world)
# Same engine as CatalogEncounter, but the rosters are WORLD CHARACTERS resolved
# in `world_id`: their names, art and extras (named items, disciplines, tuning)
# come from that world, their behaviour from the base units they play as.
#
# The id format is "world:encounter". The encounter may be inherited: playing
# "tidewrack:skirmish" runs ashlands' encounter with tidewrack's characters, so
# an override in the child world re-skins the battle without touching gameplay.
#   var session := CombatSession.new(WorldEncounter.create("tidewrack:skirmish"))
# ==============================================================================

var world_id: String = ""
var local_id: String = ""
var intro_text: String = ""
var location_name: String = ""

func _init(p_world_id: String = "", p_local_id: String = "") -> void:
	super("")
	world_id = p_world_id
	local_id = p_local_id
	encounter_id = "%s:%s" % [p_world_id, p_local_id]
	load_definition()

# `id` is "world:encounter".
static func create(id: String) -> WorldEncounter:
	var parts: Array[String] = WorldCatalog.split_qid(id)
	if parts.size() != 2:
		return WorldEncounter.new("", id)
	return WorldEncounter.new(parts[0], parts[1])

func load_definition() -> void:
	var entry: Dictionary = WorldCatalog.resolve(world_id, "encounters", local_id)
	apply_definition(entry)
	intro_text = str(entry.get("intro", ""))
	var loc: Dictionary = WorldCatalog.resolve(world_id, "locations", str(entry.get("location", "")))
	location_name = str(loc.get("name", ""))

func is_defined() -> bool:
	return WorldCatalog.has_entry(world_id, "encounters", local_id)

func _unit_entry(unit_ref: String) -> Dictionary:
	var character: Dictionary = WorldCatalog.resolve(world_id, "characters", unit_ref)
	if character.is_empty():
		return {}
	return WorldFactory.unit_entry(character)

func _customize(c: Combatant, unit_ref: String) -> void:
	WorldFactory.apply_character(c, WorldCatalog.resolve(world_id, "characters", unit_ref), world_id)

func prepare_round(round_num: int) -> Dictionary:
	var info: Dictionary = super(round_num)
	if round_num == 1:
		var where: String = (" at %s" % location_name) if not location_name.is_empty() else ""
		var text: String = "[color=cyan]>>> %s%s begins. The enemy has planned its round (hidden).[/color]" % [encounter_name, where]
		if not intro_text.strip_edges().is_empty():
			text += "\n[i]%s[/i]" % intro_text.replace("[", "[lb]")
		info["log"] = text
	return info

# The world's party cap (WorldCatalog.party_max), so the HUD shows that many party slots.
func party_slots() -> int:
	return WorldCatalog.party_max(world_id) if not world_id.is_empty() else party_cap
