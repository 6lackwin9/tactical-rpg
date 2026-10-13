class_name RestCommand
extends BattleCommand

# ==============================================================================
# Tactical RPG - Rest Command (an idle block that recovers AP)
# Costs no AP, returns the unit to Neutral stance for free (set on start by the
# planner) and restores recover_ap when the block resolves.
# ==============================================================================

var recover_ap: int = 2

func _init(p_source: Combatant, p_recover_ap: int = 2, p_block_cost: int = 1) -> void:
	super(p_source, 0, p_block_cost)
	recover_ap = p_recover_ap
	command_name = "Rest"

func execute() -> void:
	if not source.is_alive():
		return
	var before: int = source.ap
	source.ap = mini(source.max_ap, source.ap + recover_ap)
	execution_log = "[Rest] %s catches their breath (+%d AP, now %d/%d)" % [source.name, source.ap - before, source.ap, source.max_ap]