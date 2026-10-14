class_name CatalogEncounter
extends CombatEncounter

# ==============================================================================
# Tactical RPG - Catalog Encounter (a battle defined in the content catalog)
# Loads an "encounters" entry from ContentCatalog: rules (rounds, blocks, AP),
# zone and exits, and the party / enemy unit ids. Units are spawned fresh by
# ContentFactory on every build (start and reset), with their AI profiles, so
# the same entry drives the game, the Content Forge playtest and the tests.
# Player-planned: the party is planned by the player (or AI presets), enemies by
# TacticalAI each round.
#   var session := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
# ==============================================================================

var encounter_id: String = ""
var definition: Dictionary = {}
var units_by_id: Dictionary = {}   # unit id -> Array of Combatants spawned from it (encounter order)
var wave: Array[Combatant] = []     # reinforcements: off the field until wave_round / wave_block

func _init(p_encounter_id: String = "") -> void:
	encounter_id = p_encounter_id
	load_definition()

# An encounter from an entry that isn't (or isn't yet) stored, e.g. a Forge sandbox.
static func from_entry(entry: Dictionary) -> CatalogEncounter:
	var enc := CatalogEncounter.new()
	enc.encounter_id = str(entry.get("id", ""))
	enc.apply_definition(entry)
	return enc

# (Re)reads the stored entry; build() picks up the new rules and roster.
func load_definition() -> void:
	apply_definition(ContentCatalog.get_entry("encounters", encounter_id))

func apply_definition(entry: Dictionary) -> void:
	definition = ContentSchema.normalize("encounters", entry)
	encounter_name = str(definition.get("name", encounter_id))
	zone_id = str(definition["zone"])
	max_rounds = DictUtil.to_int(definition["max_rounds"], 6)
	total_blocks = DictUtil.to_int(definition["total_blocks"], 6)
	ap_regen = DictUtil.to_int(definition["ap_regen"], 4)
	unit_ap = DictUtil.to_int(definition["unit_ap"], 12)
	rng_seed = DictUtil.to_int(definition["rng_seed"], 1337)

func is_defined() -> bool:
	return ContentCatalog.has("encounters", encounter_id)

func build() -> void:
	var exits: Array[Vector2i] = []
	var raw_exits: Variant = definition["exits"]
	if raw_exits is Array:
		var exit_list: Array = raw_exits
		for x: Variant in exit_list:
			var pair: Array = x if x is Array else []
			if pair.size() == 2:
				exits.append(Vector2i(DictUtil.to_int(pair[0]), DictUtil.to_int(pair[1])))
	var arena: CombatGrid.ArenaType = CombatGrid.ArenaType.OPEN if str(definition["arena"]) == "OPEN" else CombatGrid.ArenaType.CLOSED
	grid = CombatGrid.new(arena, exits)
	grid.apply_zone(zone_id)
	reinforcements_spawned = false

	var party: Array[Combatant] = []
	var foes: Array[Combatant] = []
	units_by_id.clear()
	unit_classes.clear()
	ai_profiles.clear()
	wave.clear()
	var party_ids: Array[String] = _ids("party")
	var enemy_ids: Array[String] = _ids("enemies")
	var wave_ids: Array[String] = _ids("wave_units")
	var names: Dictionary = _display_names(party_ids + enemy_ids + wave_ids)
	var slot: int = 0
	for id: String in party_ids:
		party.append(_spawn(id, str(names[slot])))
		slot += 1
	for id: String in enemy_ids:
		foes.append(_spawn(id, str(names[slot])))
		slot += 1
	var deployed: Array[Combatant] = foes.duplicate()
	for id: String in wave_ids:
		var arriving: Combatant = _spawn(id, str(names[slot]))
		wave.append(arriving)
		foes.append(arriving)   # on the enemy side from the start, off the field until the wave
		slot += 1
	allies = party
	enemies = foes
	grid.deploy_combatants(allies, deployed)
	_place_leftovers(allies, true)
	_place_leftovers(deployed, false)

	party_stash = Inventory.new()
	ItemCatalog.populate_party_starter_stash(party_stash)

# The `index`-th unit spawned from catalog unit `unit_id` (null if none).
func unit(unit_id: String, index: int = 0) -> Combatant:
	var raw: Variant = units_by_id.get(unit_id, [])
	var list: Array = raw if raw is Array else []
	if index < 0 or index >= list.size():
		return null
	var c: Combatant = list[index]
	return c

func _ids(key: String) -> Array[String]:
	var out: Array[String] = []
	var raw: Variant = definition.get(key, [])
	if raw is Array:
		var list: Array = raw
		for v: Variant in list:
			if not _unit_entry(str(v)).is_empty():
				out.append(str(v))
	return out

# The unit entry roster id `unit_id` spawns from (virtual: a catalog unit here; a
# world character in WorldEncounter). Empty = unknown, skipped.
func _unit_entry(unit_id: String) -> Dictionary:
	return ContentCatalog.get_entry("units", unit_id)

# Virtual: tweak a freshly spawned unit (world lore, tuning, items).
func _customize(_c: Combatant, _unit_id: String) -> void:
	pass

# Display names by roster slot; repeated names are numbered (Goblin Archer, Goblin Archer 2).
func _display_names(roster: Array[String]) -> Dictionary:
	var seen: Dictionary = {}
	var out: Dictionary = {}
	for i: int in range(roster.size()):
		var base: String = str(_unit_entry(roster[i]).get("name", roster[i]))
		var n: int = DictUtil.int_of(seen, base) + 1
		seen[base] = n
		out[i] = base if n == 1 else "%s %d" % [base, n]
	return out

func _spawn(unit_id: String, display_name: String) -> Combatant:
	var entry: Dictionary = _unit_entry(unit_id)
	var c: Combatant = ContentFactory.unit(entry, display_name, unit_ap)
	_customize(c, unit_id)
	if not units_by_id.has(unit_id):
		units_by_id[unit_id] = []
	var list: Array = units_by_id[unit_id]
	list.append(c)
	unit_classes[c] = str(entry.get("art", "unit"))
	ai_profiles[c] = ContentFactory.ai_profile(entry, c)
	return c

# Units the centre-ring deployment couldn't fit go to the nearest free tile on
# their side's half (party south/west, enemies north/east).
func _place_leftovers(units: Array[Combatant], party_side: bool) -> void:
	for c: Combatant in units:
		if grid.has_combatant(c):
			continue
		var best: Vector2i = Vector2i(-1, -1)
		var best_score: int = 1 << 30
		for y: int in range(CombatGrid.GRID_SIZE):
			for x: int in range(CombatGrid.GRID_SIZE):
				var cell := Vector2i(x, y)
				var own_half: bool = (y >= CombatGrid.CENTER.y or x < CombatGrid.CENTER.x) if party_side else (y <= CombatGrid.CENTER.y or x > CombatGrid.CENTER.x)
				if not own_half or not grid.is_walkable(cell) or grid.is_exit_tile(cell) or grid.is_occupied(cell):
					continue
				var score: int = grid.get_distance(cell, CombatGrid.CENTER) * 100 + y * 10 + x
				if score < best_score:
					best_score = score
					best = cell
		if best != Vector2i(-1, -1):
			grid.place_combatant(c, best)

# ------------------------------------------------------------------------------
# Reinforcement wave
# ------------------------------------------------------------------------------

# The wave enters before its block resolves (announced as an EVENT log cell). It
# has no plan for the rest of that round; the AI plans it from the next round.
func on_block_start(round_num: int, block_num: int) -> ActionLogEntry:
	if wave.is_empty() or reinforcements_spawned:
		return null
	if round_num != DictUtil.to_int(definition["wave_round"]) or block_num != DictUtil.to_int(definition["wave_block"]):
		return null
	reinforcements_spawned = true
	_place_leftovers(wave, false)
	var arrived: Array[String] = []
	for c: Combatant in wave:
		if grid.has_combatant(c):
			arrived.append(c.name)
	return CombatEncounter.make_event_entry(round_num, block_num, "Reinforcements", false, str(definition["wave_label"]), "Wave",
		"ARRIVED", "%s join the battle." % ", ".join(arrived), "%d more hostiles" % arrived.size())

# ------------------------------------------------------------------------------
# Round flow & narration
# ------------------------------------------------------------------------------

func prepare_round(round_num: int) -> Dictionary:
	if round_num == 1:
		return {
			"phase": "ROUND 1: PLANNING PHASE",
			"status": "Plan your party's round on the timeline, then confirm.",
			"log": "[color=cyan]>>> %s begins. The enemy has planned its round (hidden).[/color]" % encounter_name,
		}
	return {
		"phase": "ROUND %d: PLANNING PHASE" % round_num,
		"status": "Round %d: re-plan your party. The enemy has chosen its moves." % round_num,
		"log": "\n[color=#38bdf8][b]⚔ ROUND %d: PLANNING ⚔[/b][/color]" % round_num,
	}


func party_alive() -> bool:
	for a: Combatant in allies:
		if a.is_active():
			return true
	return false

func round_summary(round_num: int, combat_over: bool) -> Dictionary:
	if combat_over and party_alive():
		return {
			"phase": "🏆 VICTORY",
			"status": "🏆 %s: the enemy is down or has fled. Review the replay." % encounter_name,
			"log": "\n[color=#38bdf8][b]🏆 VICTORY: %s won![/b][/color]" % encounter_name,
		}
	if combat_over:
		return {
			"phase": "💀 DEFEAT - PARTY FALLEN",
			"status": "💀 Your party was wiped out. Reset the battle to try a new plan.",
			"log": "\n[color=#f87171][b]💀 DEFEAT: the party has fallen.[/b][/color]",
		}
	if round_num >= max_rounds:
		return {
			"phase": "ROUND %d CONCLUDED - STALEMATE" % round_num,
			"status": "Round limit reached with both sides standing.",
			"log": "\n[color=#fde047][b]%s ends in a stalemate.[/b][/color]" % encounter_name,
		}
	return {
		"phase": "ROUND %d CONCLUDED - REPLAY MODE" % round_num,
		"status": "Round %d complete. Review the replay, then advance." % round_num,
		"log": "",
	}
