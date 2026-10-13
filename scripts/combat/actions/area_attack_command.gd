class_name AreaAttackCommand
extends BattleCommand

# ==============================================================================
# Tactical RPG - Area Attack Command (an attack that hits every unit in an area)
# Resolves at the end of its last block against whoever stands on `cells` then
# (allies included: friendly fire). Each hit uses the stance matchup against
# that unit and its own guard. The caster's own tile is never hit.
# ==============================================================================

var grid: CombatGrid = null
var cells: Array[Vector2i] = []
var power: int = 10

var last_damage: int = 0         # total dealt
var last_hits: int = 0
var last_breakdown: String = ""

func _init(p_source: Combatant, p_grid: CombatGrid, p_cells: Array[Vector2i], p_power: int, p_block_cost: int = 2) -> void:
	super(p_source, 0, p_block_cost)
	grid = p_grid
	cells = p_cells
	power = p_power
	command_name = "Area Attack"

func execute() -> void:
	last_damage = 0
	last_hits = 0
	if not source.is_active():
		execution_log = "[Action Failed] %s cannot act (unit is inactive)." % source.name
		return
	var parts: Array[String] = []
	for cell: Vector2i in cells:
		var victim: Combatant = grid.get_combatant_at(cell)
		if victim == null or victim == source or not victim.is_active():
			continue
		var mult: float = CombatTypes.get_matchup_multiplier(source.current_stance, victim.current_stance)
		var raw: int = maxi(1, roundi(float(power) * mult * source.attack_boost_multiplier))
		var taken: int = victim.take_damage(raw)
		last_damage += taken
		last_hits += 1
		parts.append("%s %d%s" % [victim.name, taken, " DEFEATED" if not victim.is_alive() else ""])
	last_breakdown = "Power %d per target × stance × buff, each target's guard applies" % power
	if last_hits == 0:
		execution_log = "[Area Miss] %s's %s hit nobody." % [source.name, command_name]
	else:
		execution_log = "[Attack] %s's %s hit %d unit%s for %d damage (%s)" % [source.name, command_name, last_hits, "" if last_hits == 1 else "s", last_damage, ", ".join(parts)]
