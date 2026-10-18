class_name HazardRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Hazard Rules (S12T-L rule 6)
# Hazard tiles (Terrain.hazard: status, potency, magnitude, rounds, on) roll their status through
# StatusEffects.try_inflict (resistance applies; the status starts next round like every queued status):
#   enter      a walk, dash or disengage that enters hazard tiles rolls each status once per unit per command
#   round_end  after the round-end ticks, every active unit standing on a hazard tile is rolled
#   both       both of the above
# The source name is "terrain" (sim metrics credit the damage to no unit).
# ==============================================================================

const SOURCE: String = "terrain"

static func triggers_on_enter(h: Dictionary) -> bool:
	var on: String = str(h.get("on", ""))
	return on == TerrainFields.ON_ENTER or on == TerrainFields.ON_BOTH

static func triggers_at_round_end(h: Dictionary) -> bool:
	var on: String = str(h.get("on", ""))
	return on == TerrainFields.ON_ROUND_END or on == TerrainFields.ON_BOTH

# The enter-hazards of the cells a command entered, one per status (the first tile of each status wins):
# [{status, potency, magnitude, rounds, on, cell}], in the order entered.
static func entered(grid: CombatGrid, cells: Array[Vector2i]) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var seen: Dictionary = {}
	if grid == null:
		return out
	for cell: Vector2i in cells:
		var h: Dictionary = grid.hazard_at(cell)
		if h.is_empty() or not triggers_on_enter(h) or seen.has(h["status"]):
			continue
		seen[h["status"]] = true
		var row: Dictionary = h.duplicate()
		row["cell"] = cell
		out.append(row)
	return out

# Rolls the enter-hazards of `cells` on `unit` (once per status). Returns the log lines of successful rolls.
static func on_enter(grid: CombatGrid, unit: Combatant, cells: Array[Vector2i], rng: RandomNumberGenerator) -> Array[String]:
	var lines: Array[String] = []
	if unit == null or not unit.is_active():
		return lines
	for h: Dictionary in entered(grid, cells):
		var line: String = _roll(grid, unit, h, rng)
		if not line.is_empty():
			lines.append(line)
	return lines

# Round end: every active unit in `units` standing on a round-end hazard tile is rolled. Returns the log lines.
static func at_round_end(grid: CombatGrid, units: Array[Combatant], rng: RandomNumberGenerator) -> Array[String]:
	var lines: Array[String] = []
	if grid == null:
		return lines
	for unit: Combatant in units:
		if not unit.is_active() or not grid.has_combatant(unit):
			continue
		var cell: Vector2i = grid.get_position(unit)
		var h: Dictionary = grid.hazard_at(cell)
		if h.is_empty() or not triggers_at_round_end(h):
			continue
		h = h.duplicate()
		h["cell"] = cell
		var line: String = _roll(grid, unit, h, rng)
		if not line.is_empty():
			lines.append(line)
	return lines

static func _roll(grid: CombatGrid, unit: Combatant, h: Dictionary, rng: RandomNumberGenerator) -> String:
	var status: String = str(h["status"])
	var hit: bool = StatusEffects.try_inflict(unit, status, DictUtil.to_int(h["potency"], 100), rng, DictUtil.to_int(h["rounds"], 1), DictUtil.to_int(h["magnitude"]), SOURCE)
	if not hit:
		return ""
	var cell: Vector2i = h.get("cell", Vector2i.ZERO)
	return "[Hazard] %s is %s by %s" % [unit.name, StatusEffects.display_name(status), Terrain.display_name(grid.get_terrain(cell))]

# Preview chip text of the hazards a planned route enters ("" when none): "Hazard: <terrain name> (<status>)".
static func preview_chip(grid: CombatGrid, start: Vector2i, dirs: Array[Vector2i]) -> String:
	var cells: Array[Vector2i] = []
	var cell: Vector2i = start
	for d: Vector2i in dirs:
		cell += d
		cells.append(cell)
	return preview_chip_cells(grid, cells)

static func preview_chip_cells(grid: CombatGrid, cells: Array[Vector2i]) -> String:
	var parts: Array[String] = []
	if grid == null:
		return ""
	for h: Dictionary in entered(grid, cells):
		var c: Vector2i = h["cell"]
		parts.append("Hazard: %s (%s)" % [Terrain.display_name(grid.get_terrain(c)), StatusEffects.display_name(str(h["status"]))])
	return ", ".join(parts)
