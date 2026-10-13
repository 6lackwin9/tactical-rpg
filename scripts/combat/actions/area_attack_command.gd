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
var skill_mult: float = 1.0

var last_damage: int = 0         # total dealt
var last_hits: int = 0
var last_breakdown: String = ""

func _init(p_source: Combatant, p_grid: CombatGrid, p_cells: Array[Vector2i], p_skill_mult: float, p_block_cost: int = 2) -> void:
	super(p_source, 0, p_block_cost)
	grid = p_grid
	cells = p_cells
	skill_mult = p_skill_mult
	command_name = "Area Attack"

func execute() -> void:
	last_damage = 0
	last_hits = 0
	if not source.is_active():
		execution_log = "[Action Failed] %s cannot act (unit is inactive)." % source.name
		return
	var parts: Array[String] = []
	var breakdowns: Array[String] = []
	for cell: Vector2i in cells:
		var victim: Combatant = grid.get_combatant_at(cell)
		if victim == null or victim == source or not victim.is_active():
			continue
		var res: Dictionary = DamageCalculator.compute(source, victim, source.attack_range > 1, false, skill_mult, false, rng)
		var taken: int = victim.take_damage(DictUtil.to_int(res["final_dmg"]), true)
		last_damage += taken
		last_hits += 1
		parts.append("%s %d%s%s" % [victim.name, taken, " CRIT" if DictUtil.to_bool(res["is_crit"]) else "", " DEFEATED" if not victim.is_alive() else ""])
		breakdowns.append("%s: %s" % [victim.name, DamageCalculator.describe(res)])
	last_breakdown = "\n".join(breakdowns)
	if last_hits == 0:
		execution_log = "[Area Miss] %s's %s hit nobody." % [source.name, command_name]
	else:
		execution_log = "[Attack] %s's %s hit %d unit%s for %d damage (%s)" % [source.name, command_name, last_hits, "" if last_hits == 1 else "s", last_damage, ", ".join(parts)]
