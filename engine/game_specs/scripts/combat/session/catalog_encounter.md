---
class: CatalogEncounter
extends: CombatEncounter
description: "Tactical RPG - Catalog Encounter (a battle defined in the content catalog) Loads an \"encounters\" entry from ContentCatalog: rules (rounds, blocks, AP), zone and exits, and the party / enemy unit ids. Units are spawned fresh by ContentFactory on every build (start and reset), with their AI profiles, so the same entry drives the game, the Content Forge playtest and the tests. Player-planned: the party is planned by the player (or AI presets), enemies by TacticalAI each round. var session := CombatSession.new(CatalogEncounter.new(\"goblin_skirmish\"))"
---

# Properties
* `encounter_id`: String = ""
* `definition`: Dictionary = {}
* `units_by_id`: Dictionary = {}   # unit id -> Array of Combatants spawned from it (encounter order)
* `wave`: Array[Combatant] = []     # reinforcements: off the field until wave_round / wave_block
* `phase_runtime`: EncounterPhases = EncounterPhases.new()   # BOSS1 phases of this battle (rebuilt in build())
* `objective_runtime`: EncounterObjectives = EncounterObjectives.new()   # OBJ1 objective of this battle (rebuilt in build())
* `deployment`: String = EncounterObjectives.STANDARD
* `protected_guests`: Array[Combatant] = []   # guests that must live (OBJ1 `protect`)
* `_verdict`: String = ""   # the last non-empty verdict (round_summary reads it)
* `FLED_TEXT`: String = "The party withdrew." (const)
* `out`: Array[String] = []
* `raw`: Variant = definition.get(key, [])
* `list`: Array = raw
* `out`: Array[String] = []
* `out`: Array[Combatant] = []

# Methods

### `_init(p_encounter_id: String = "") -> void`
```logic
encounter_id = p_encounter_id
load_definition()
```

### `static from_entry(entry: Dictionary) -> CatalogEncounter`
```logic
var enc := CatalogEncounter.new()
enc.encounter_id = str(entry.get("id", ""))
enc.apply_definition(entry)
return enc
```

### `load_definition() -> void`
```logic
apply_definition(ContentCatalog.get_entry("encounters", encounter_id))
```

### `apply_definition(entry: Dictionary) -> void`
```logic
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
```

### `is_defined() -> bool`
```logic
return ContentCatalog.has("encounters", encounter_id)
```

### `build() -> void`
```logic
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
```

### `unit(unit_id: String, index: int = 0) -> Combatant`
```logic
var raw: Variant = units_by_id.get(unit_id, [])
var list: Array = raw if raw is Array else []
if index < 0 or index >= list.size():
	return null
var c: Combatant = list[index]
return c
```

### `_unit_entry(unit_id: String) -> Dictionary`
```logic
return ContentCatalog.get_entry("units", unit_id)
```

### `_customize(_c: Combatant, _unit_id: String) -> void`
```logic
pass
```

### `_display_names(roster: Array[String]) -> Dictionary`
```logic
var seen: Dictionary = {}
var out: Dictionary = {}
for i: int in range(roster.size()):
	var base: String = str(_unit_entry(roster[i]).get("name", roster[i]))
	var n: int = DictUtil.int_of(seen, base) + 1
	seen[base] = n
	out[i] = base if n == 1 else "%s %d" % [base, n]
return out
```

### `_spawn(unit_id: String, display_name: String) -> Combatant`
```logic
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
```

### `_place_leftovers(units: Array[Combatant], party_side: bool) -> void`
```logic
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
```

### `on_block_start(round_num: int, block_num: int) -> ActionLogEntry`
```logic
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
```

### `is_reserved(c: Combatant) -> bool`
```logic
return phase_runtime.is_reserved(c)
```

### `verdict(_round_num: int, _block_num: int, end_of_round: bool) -> String`
```logic
var target: Combatant = null
if objective_runtime.objective == EncounterObjectives.SLAY:
	target = unit(objective_runtime.unit_ref, 0)
_verdict = objective_runtime.verdict(heroes(), guests, protected_guests, live_enemies(), grid, end_of_round, target)
return _verdict
```

### `objective_state() -> Dictionary`
```logic
return objective_runtime.state()
```

### `restore_objective_state(state: Dictionary) -> void`
```logic
objective_runtime.restore(state)
_verdict = ""
```

### `objective_text() -> String`
```logic
return EncounterObjectives.text_of(definition, func(ref: String) -> String: return _ref_name(ref))
```

### `_ref_name(ref: String) -> String`
```logic
var c: Combatant = unit(ref, 0)
return c.name if c != null else str(_unit_entry(ref).get("name", ref))
```

### `on_block_end(_round_num: int, _block_num: int) -> void`
```logic
phase_runtime.check_block_end(func(ref: String) -> Combatant: return unit(ref, 0))
```

### `phase_state() -> Dictionary`
```logic
return phase_runtime.state()
```

### `restore_phase_state(state: Dictionary, round_num: int) -> void`
```logic
phase_runtime.restore(state)
_apply_phase_profiles(round_num)
```

### `phase_info(entry: ActionLogEntry) -> Dictionary`
```logic
return phase_runtime.info(entry)
```

### `_apply_phase_profiles(round_num: int) -> void`
```logic
phase_runtime.apply_profiles(ai_profiles, units_of_ref, round_num)
```

### `_fire_phase(id: String, round_num: int, block_num: int) -> ActionLogEntry`
```logic
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
			StatusEffects.grant_now(c, status, DictUtil.to_int(fx["rounds"]), DictUtil.to_int(fx["magnitude"]), "phase:%s" % id)
			granted = true
	if granted:
		parts.append(StatusEffects.display_name(status))
if not EncounterPhases.effects_of(p, EncounterPhases.EFFECT_AI).is_empty():
	parts.append("new tactics from the next round" if block_num > 0 else "new tactics")
var entry: ActionLogEntry = CombatEncounter.make_event_entry(round_num, block_num, actor, false, str(p["name"]), EncounterPhases.SUBTYPE,
	EncounterPhases.OUTCOME, detail, ", ".join(parts))
phase_runtime.mark_fired(id, round_num, block_num, entry)
return entry
```

### `prepare_round(round_num: int) -> Dictionary`
```logic
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
```

### `_intro(round_num: int) -> Dictionary`
```logic
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
```

### `_defeat_summary() -> Dictionary`
```logic
return {
	"phase": "💀 DEFEAT - PARTY FALLEN",
	"status": "💀 Your party was wiped out. Reset the battle to try a new plan.",
	"log": "\n[color=#f87171][b]💀 DEFEAT: the party has fallen.[/b][/color]",
}
```

### `party_alive() -> bool`
```logic
for a: Combatant in allies:
	if a.is_active():
		return true
return false
```

### `round_summary(round_num: int, combat_over: bool) -> Dictionary`
```logic
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
```

