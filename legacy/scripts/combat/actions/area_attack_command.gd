class_name AreaAttackCommand
extends BattleCommand

# ==============================================================================
# Tactical RPG - Area Attack Command (an attack that hits every unit in an area)
# Resolves at the end of its last block against whoever stands on `cells` then
# (allies included: friendly fire). Each victim takes its own DamageCalculator hit
# (its stance, guard and armor; its own crit roll). The caster's tile is never hit.
# ==============================================================================

var grid: CombatGrid = null
var cells: Array[Vector2i] = []
var aim_cell: Vector2i = Vector2i(-1, -1)   # the tile the area was aimed at (display only: intent tells; -1 = unknown)
var skill_mult: float = 1.0
var status_rounds: int = 1       # how long an inflicted status lasts once active

var last_damage: int = 0         # total dealt
var last_hits: int = 0
var last_breakdown: String = ""
var crit_hits: Array[Dictionary] = []   # [{unit: Combatant, killed: bool}] per critical hit, in hit order (MORALE1 rule 6)

func _init(p_source: Combatant, p_grid: CombatGrid, p_cells: Array[Vector2i], p_skill_mult: float, p_block_cost: int = 2) -> void:
	super(p_source, 0, p_block_cost)
	grid = p_grid
	cells = p_cells
	skill_mult = p_skill_mult
	command_name = "Area Attack"

func execute() -> void:
	last_damage = 0
	last_hits = 0
	crit_hits.clear()
	reactions.clear()
	broken_targets.clear()
	combo_notes.clear()
	exploited.clear()
	failed = false
	if not source.is_active():
		failed = true
		execution_log = "[Action Failed] %s cannot act (unit is inactive)." % source.name
		return
	var parts: Array[String] = []
	var breakdowns: Array[String] = []
	for cell: Vector2i in cells:
		var victim: Combatant = grid.get_combatant_at(cell)
		if victim == null or victim == source or not victim.is_active():
			continue
		if skill_mult <= 0.0:
			continue   # a zero-power area attack never damages or provokes (schema rejects it; D1 rule kept)
		var cover: float = CoverRules.cover_for(grid, grid.get_position(source), cell, source.attack_range > 1)   # each hit unit's tile vs the caster (S12T-L)
		var matched: Dictionary = exploit_against(victim)   # per unit hit: its active statuses (S12C-L)
		var res: Dictionary = DamageCalculator.compute(source, victim, source.attack_range > 1, false, skill_mult * DictUtil.to_float(matched["mult"], 1.0), false, rng, -1, true, cover)
		var taken: int = victim.take_damage(DictUtil.to_int(res["final_dmg"]), true)
		last_damage += taken
		last_hits += 1
		if DictUtil.to_bool(res["is_crit"]):
			crit_hits.append({"unit": victim, "killed": not victim.is_alive()})
		var status_note: String = _roll_status(victim)   # one independent roll per unit hit
		parts.append("%s %d%s%s%s%s" % [victim.name, taken, " CRIT" if DictUtil.to_bool(res["is_crit"]) else "", " DEFEATED" if not victim.is_alive() else "", status_note, exploit_note(matched)])
		breakdowns.append("%s: %s" % [victim.name, DamageCalculator.describe(res)])
		var counter: Dictionary = RiposteRules.counter(source, victim, grid, source.attack_range > 1, rng)
		if not counter.is_empty():
			reactions.append(counter)
		after_hit(victim, taken, matched)   # stance break, then consume (S12C-L rule 5)
		note_exploit(victim, matched)
	last_breakdown = "\n".join(breakdowns)
	if last_hits == 0:
		execution_log = "[Area Miss] %s's %s hit nobody." % [source.name, command_name]
	else:
		execution_log = "[Attack] %s's %s hit %d unit%s for %d damage (%s)" % [source.name, command_name, last_hits, "" if last_hits == 1 else "s", last_damage, ", ".join(parts)]

# Rolls the skill's status on one unit hit (allies too: friendly fire). Returns a log suffix.
func _roll_status(victim: Combatant) -> String:
	if status_effect.is_empty() or not victim.is_alive():
		return ""
	var chance: int = StatusEffects.apply_chance(victim, status_effect, status_potency)
	if StatusEffects.try_inflict(victim, status_effect, status_potency, rng, status_rounds, status_magnitude, source.name):
		return " [%s next round (%d%%)]" % [StatusEffects.display_name(status_effect), chance]
	return " [resisted %s (%d%%)]" % [StatusEffects.display_name(status_effect).to_lower(), chance]
