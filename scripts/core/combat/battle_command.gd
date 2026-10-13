class_name BattleCommand
extends RefCounted

# ==============================================================================
# Tactical RPG - Battle Command (one scheduled action on a unit's timeline)
# Built from a PlanEntry. When it STARTS the resolver charges its AP (skill +
# automatic stance switch; debt allowed), enters stance_on_start and calls
# on_start(); its effect (execute) lands at the end of its last block.
# ==============================================================================

var source: Combatant
var priority: int = 0  # High-priority actions (Switch Target, Defend) resolve first within a block
var block_cost: int = 1  # Timeline duration in blocks
var command_name: String = "Command"
var execution_log: String = ""

var ap_cost: int = 0                 # skill AP (excludes the stance switch)
var stance_switch_cost: int = 0     # extra AP for changing stance on start
var stance_on_start: int = -1       # CombatTypes.StanceType to enter at start, -1 = keep
var est_damage: int = 0              # planner's expected hit (attacks), for previews
var status_effect: String = ""        # status this command tries to inflict (StatusEffects)
var status_potency: int = 0
var rng: RandomNumberGenerator = null   # battle RNG, set by the resolver before execute()

func _init(p_source: Combatant, p_priority: int = 0, p_block_cost: int = 1) -> void:
	source = p_source
	priority = p_priority
	block_cost = p_block_cost

# Returns effective speed tier for intra-block turn execution order
func get_initiative() -> int:
	return (priority * 1000) + source.speed

func can_execute() -> bool:
	if not source.is_alive():
		return false
	return true

# Virtual: runs when a planner command STARTS (after AP and stance). Returns a log line or "".
func on_start() -> String:
	return ""

# Virtual method: calculate and apply state modifications
func execute() -> void:
	pass