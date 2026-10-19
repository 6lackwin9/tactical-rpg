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
# PACE1 rule 7: with a game_state (GameEncounter), round 1 starts by granting each start_status entry whose
# requires_flags hold to every hero and guest on the field, one EVENT entry each; without one nothing is granted.
# ==============================================================================

var world_id: String = ""
var local_id: String = ""
var intro_text: String = ""
var location_name: String = ""
var game_state: GameState = null    # PACE1: the game's flags for start_status (GameEncounter; null = no start buffs)
var grant_all_start: bool = false   # PACE1-S hook (sim start=on): grant every start_status entry regardless of flags

const START_SOURCE: String = "start:"   # status source of a start buff ("start:<encounter id>"; BattleMetrics credits nobody)
const START_SUBTYPE: String = "Start"   # EVENT entry skill_subtype of a start buff
const START_OUTCOME: String = "START"

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
		var entries: Array[ActionLogEntry] = []
		entries.assign(DictUtil.array_of(info.get("entries", [])))
		entries.append_array(_grant_start_buffs())   # PACE1 rule 7: before the first planning phase
		info["entries"] = entries
		var where: String = (" at %s" % location_name) if not location_name.is_empty() else ""
		var text: String = "[color=cyan]>>> %s%s begins. The enemy has planned its round (hidden).[/color]" % [encounter_name, where]
		if not intro_text.strip_edges().is_empty():
			text += "\n[i]%s[/i]" % intro_text.replace("[", "[lb]")
		info["log"] = text
	return info

# PACE1 rule 7: the start_status entries granted in this battle: with a game state, those whose requires_flags all hold;
# with grant_all_start (the sim's start=on), every entry; with neither (previews, the Forge, the sim default), none.
func start_buffs_due() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if game_state == null and not grant_all_start:
		return out
	for sb: Dictionary in EncounterPhases.parse_start_status(definition.get("start_status", [])):
		var holds: bool = true
		if not grant_all_start:
			for cond: Variant in DictUtil.array_of(sb["requires_flags"]):
				holds = holds and QuestRules.condition_met(game_state, DictUtil.dict_of(cond))
		if holds:
			out.append(sb)
	return out

# Grants each due start buff to every hero and guest on the field (grant_now: its rounds count round 1; source
# "start:<encounter id>") and returns one EVENT entry per granted entry, carrying its text.
func _grant_start_buffs() -> Array[ActionLogEntry]:
	var out: Array[ActionLogEntry] = []
	for sb: Dictionary in start_buffs_due():
		var status: String = str(sb["status"])
		var names: Array[String] = []
		for c: Combatant in allies:
			if c.is_active() and grid.has_combatant(c):
				StatusEffects.grant_now(c, status, DictUtil.to_int(sb["rounds"]), DictUtil.to_int(sb["magnitude"]), START_SOURCE + local_id)
				names.append(c.name)
		if names.is_empty():
			continue
		var shown: String = StatusEffects.label_with_magnitude(status, StatusEffects.magnitude_or_min(status, DictUtil.to_int(sb["magnitude"])))
		out.append(CombatEncounter.make_event_entry(1, 0, encounter_name, true, StatusEffects.display_name(status), START_SUBTYPE,
			START_OUTCOME, str(sb["text"]), shown))
	return out

# The world's party cap (WorldCatalog.party_max), so the HUD shows that many party slots.
func party_slots() -> int:
	return WorldCatalog.party_max(world_id) if not world_id.is_empty() else party_cap
