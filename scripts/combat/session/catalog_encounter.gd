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
var phase_runtime: EncounterPhases = EncounterPhases.new()   # BOSS1 phases of this battle (rebuilt in build())
var objective_runtime: EncounterObjectives = EncounterObjectives.new()   # OBJ1 objective of this battle (rebuilt in build())
var deployment: String = EncounterObjectives.STANDARD
var protected_guests: Array[Combatant] = []   # guests that must live (OBJ1 `protect`)
var _verdict: String = ""   # the last non-empty verdict (round_summary reads it)
const FLED_TEXT: String = "The party withdrew."
const PHASE_SOURCE: String = "phase:"   # status source prefix of a phase's status grants ("phase:<id>"; BattleMetrics credits the enemy recipient)

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
	var raw_deployment: String = str(definition.get("deployment", EncounterObjectives.STANDARD))
	deployment = raw_deployment if EncounterObjectives.DEPLOYMENTS.has(raw_deployment) else EncounterObjectives.STANDARD

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
	var guest_ids: Array[String] = _guest_ids()
	phase_runtime = EncounterPhases.create(definition.get("phases", []))
	var phase_ids: Array[String] = []   # phase-wave refs in phase order (parallel: the phase each belongs to)
	var phase_of: Array[String] = []
	for p: Dictionary in phase_runtime.phases:
		for ref: String in EncounterPhases.wave_refs_of(p):
			if not _unit_entry(ref).is_empty():
				phase_ids.append(ref)
				phase_of.append(str(p["id"]))
	var names: Dictionary = _display_names(party_ids + enemy_ids + wave_ids + phase_ids + guest_ids)   # guests last: names unchanged
	var slot: int = 0
	for id: String in party_ids:
		party.append(_spawn(id, str(names[slot])))
		slot += 1
	guests.clear()
	protected_guests.clear()
	var protect_ids: Array[String] = JsonUtil.strings(definition.get("protect", []))
	var guest_slot: int = party_ids.size() + enemy_ids.size() + wave_ids.size() + phase_ids.size()
	for id: String in guest_ids:   # OBJ1 rule 1: after the heroes, on the party side
		var guest: Combatant = _spawn(id, str(names[guest_slot]))
		guests.append(guest)
		party.append(guest)
		if protect_ids.has(id):
			protected_guests.append(guest)
		guest_slot += 1
	for id: String in enemy_ids:
		foes.append(_spawn(id, str(names[slot])))
		slot += 1
	var deployed: Array[Combatant] = foes.duplicate()
	for id: String in wave_ids:
		var arriving: Combatant = _spawn(id, str(names[slot]))
		wave.append(arriving)
		foes.append(arriving)   # on the enemy side from the start, off the field until the wave
		slot += 1
	for i: int in range(phase_ids.size()):
		var reserved: Combatant = _spawn(phase_ids[i], str(names[slot]))
		phase_runtime.reserve(phase_of[i], reserved)
		foes.append(reserved)   # registered like the wave; reserved (off the field) until its phase fires
		slot += 1
	allies = party
	enemies = foes
	objective_runtime = EncounterObjectives.create(definition)
	_verdict = ""
	grid.deploy_combatants(allies, deployed, deployment)
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

# Guest roster refs (OBJ1): known ones, each once, at most MAX_GUESTS.
func _guest_ids() -> Array[String]:
	var out: Array[String] = []
	for id: String in _ids("guests"):
		if not out.has(id) and out.size() < EncounterObjectives.MAX_GUESTS:
			out.append(id)
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
	unit_classes[c] = ContentFactory.art_of(entry)
	ai_profiles[c] = ContentFactory.ai_profile(entry, c)
	return c

# Units the standard deployment couldn't fit (and reinforcement waves) go to the
# free tile nearest the arena middle on their side's half (party left, enemies right).
# Ambush / centre (OBJ1 1.2): enemy arrivals alternate groups A, B, A... and take the free tile nearest the group front.
func _place_leftovers(units: Array[Combatant], party_side: bool) -> void:
	if not party_side and deployment != EncounterObjectives.STANDARD:
		var turn: int = 0
		for c: Combatant in units:
			if grid.has_combatant(c):
				continue
			var cols: Array[int] = CombatGrid.layout_columns(deployment, "a" if turn % 2 == 0 else "b")
			grid.place_near_front(c, cols[0])
			turn += 1
		return
	for c: Combatant in units:
		if grid.has_combatant(c):
			continue
		var best: Vector2i = Vector2i(-1, -1)
		var best_score: int = 1 << 30
		for y: int in range(CombatGrid.GRID_H):
			for x: int in range(CombatGrid.GRID_W):
				var cell := Vector2i(x, y)
				if CombatGrid.is_party_half(cell) != party_side or not grid.is_walkable(cell) or grid.is_exit_tile(cell) or grid.is_occupied(cell):
					continue
				var score: int = CombatGrid.centre_distance(cell) * 1000 + y * 20 + x
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
	var due_id: String = phase_runtime.next_due()   # BOSS1 rule 2: a phase met at the end of the last block fires first
	if not due_id.is_empty():
		return _fire_phase(due_id, round_num, block_num)
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
# Phases (BOSS1, EncounterPhases; spec rules 1 to 6)
# ------------------------------------------------------------------------------

func is_reserved(c: Combatant) -> bool:
	return phase_runtime.is_reserved(c)

# ------------------------------------------------------------------------------
# Objective (OBJ1, EncounterObjectives; spec section 2 rules 3 to 7)
# ------------------------------------------------------------------------------

func verdict(_round_num: int, _block_num: int, end_of_round: bool) -> String:
	var target: Combatant = null
	if objective_runtime.objective == EncounterObjectives.SLAY:
		target = unit(objective_runtime.unit_ref, 0)
	_verdict = objective_runtime.verdict(heroes(), guests, protected_guests, live_enemies(), grid, end_of_round, target)
	return _verdict

func objective_state() -> Dictionary:
	return objective_runtime.state()

func restore_objective_state(state: Dictionary) -> void:
	objective_runtime.restore(state)
	_verdict = ""

# The objective line (rule 9): the authored text or the generated one, names from this battle's units.
func objective_text() -> String:
	return EncounterObjectives.text_of(definition, func(ref: String) -> String: return _ref_name(ref))

func _ref_name(ref: String) -> String:
	var c: Combatant = unit(ref, 0)
	return c.name if c != null else str(_unit_entry(ref).get("name", ref))

func on_block_end(_round_num: int, _block_num: int) -> void:
	phase_runtime.check_block_end(func(ref: String) -> Combatant: return unit(ref, 0))

func phase_state() -> Dictionary:
	return phase_runtime.state()

func restore_phase_state(state: Dictionary, round_num: int) -> void:
	phase_runtime.restore(state)
	_apply_phase_profiles(round_num)

func phase_info(entry: ActionLogEntry) -> Dictionary:
	return phase_runtime.info(entry)

# Every unit spawned from roster ref `ref` (encounter order).
func units_of_ref(ref: String) -> Array[Combatant]:
	var out: Array[Combatant] = []
	out.assign(DictUtil.array_of(units_by_id.get(ref, [])))
	return out

func _apply_phase_profiles(round_num: int) -> void:
	phase_runtime.apply_profiles(ai_profiles, units_of_ref, round_num)

# Fires phase `id` at [round, block] (block 0 = round start): line, wave, status, then ai (rule 3). Returns its EVENT entry.
func _fire_phase(id: String, round_num: int, block_num: int) -> ActionLogEntry:
	var p: Dictionary = phase_runtime.phase(id)
	var trigger: Dictionary = p["trigger"]
	var actor: String = encounter_name
	if str(trigger["kind"]) == EncounterPhases.TRIGGER_HP_BELOW:
		var trigger_unit: Combatant = unit(str(trigger["unit"]), 0)
		if trigger_unit != null:
			actor = trigger_unit.name
	var detail: String = ""
	for fx: Dictionary in EncounterPhases.effects_of(p, EncounterPhases.EFFECT_LINE):
		detail = str(fx["text"])
	var parts: Array[String] = []
	for fx: Dictionary in EncounterPhases.effects_of(p, EncounterPhases.EFFECT_WAVE):
		var units: Array[Combatant] = phase_runtime.units_of(id)
		_place_leftovers(units, false)
		var arrived: int = 0
		for c: Combatant in units:
			if grid.has_combatant(c):
				arrived += 1
		var label: String = str(fx["label"])
		parts.append(label if not label.is_empty() else "%d more hostiles" % arrived)
	for fx: Dictionary in EncounterPhases.effects_of(p, EncounterPhases.EFFECT_STATUS):
		var status: String = str(fx["status"])
		var granted: bool = false
		for c: Combatant in units_of_ref(str(fx["unit"])):
			if c.is_active() and grid.has_combatant(c) and not is_reserved(c):
				StatusEffects.grant_now(c, status, DictUtil.to_int(fx["rounds"]), DictUtil.to_int(fx["magnitude"]), PHASE_SOURCE + id)
				granted = true
		if granted:
			parts.append(StatusEffects.display_name(status))
	if not EncounterPhases.effects_of(p, EncounterPhases.EFFECT_AI).is_empty():
		parts.append("new tactics from the next round" if block_num > 0 else "new tactics")
	var entry: ActionLogEntry = CombatEncounter.make_event_entry(round_num, block_num, actor, false, str(p["name"]), EncounterPhases.SUBTYPE,
		EncounterPhases.OUTCOME, detail, ", ".join(parts))
	phase_runtime.mark_fired(id, round_num, block_num, entry)
	return entry

# ------------------------------------------------------------------------------
# Round flow & narration
# ------------------------------------------------------------------------------

# Rule 2 / 6: phases fired at this round's start (after a reset: re-logged, not applied again), then round phases
# of this round and phases met at the end of the last round fire (list order) before the enemy draft; the live AI
# profiles are rebuilt for this round's draft. They come back under "entries".
func prepare_round(round_num: int) -> Dictionary:
	var info: Dictionary = _intro(round_num)
	var entries: Array[ActionLogEntry] = phase_runtime.round_start_entries(round_num)
	phase_runtime.mark_round(round_num)
	var id: String = phase_runtime.next_due()
	while not id.is_empty():
		entries.append(_fire_phase(id, round_num, 0))
		id = phase_runtime.next_due()
	_apply_phase_profiles(round_num)
	info["entries"] = entries
	return info

func _intro(round_num: int) -> Dictionary:
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

func _defeat_summary() -> Dictionary:
	return {
		"phase": "💀 DEFEAT - PARTY FALLEN",
		"status": "💀 Your party was wiped out. Reset the battle to try a new plan.",
		"log": "\n[color=#f87171][b]💀 DEFEAT: the party has fallen.[/b][/color]",
	}

func party_alive() -> bool:
	for a: Combatant in allies:
		if a.is_active():
			return true
	return false

# The battle end reads the verdict (OBJ1): won -> victory (also an escape with everyone out), lost -> defeat, fled ->
# "battle left" with the reason (a slay target got away, else the party withdrew).
func round_summary(round_num: int, combat_over: bool) -> Dictionary:
	var ending: String = _verdict
	if combat_over and ending.is_empty():   # no verdict stored (a subclass ended the battle): today's reading
		ending = EncounterObjectives.WON if party_alive() else EncounterObjectives.LOST
	if combat_over and ending == EncounterObjectives.WON:
		return {
			"phase": "🏆 VICTORY",
			"status": "🏆 %s: the enemy is down or has fled. Review the replay." % encounter_name,
			"log": "\n[color=#38bdf8][b]🏆 VICTORY: %s won![/b][/color]" % encounter_name,
		}
	if combat_over and ending == EncounterObjectives.FLED:
		var why: String = objective_runtime.note if not objective_runtime.note.is_empty() else FLED_TEXT
		return {
			"phase": "⚑ BATTLE LEFT",
			"status": "⚑ %s: %s" % [encounter_name, why],
			"log": "\n[color=#fde047][b]⚑ %s: %s[/b][/color]" % [encounter_name, why],
		}
	if combat_over:
		return _defeat_summary()
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
