class_name BattleCommand
extends RefCounted

var source: Combatant
var priority: int = 0  # High-priority actions (e.g., Defend) resolve first
var block_cost: int = 1  # Timeline duration in blocks (1 to 3)
var action_type: CombatTypes.ActionType = CombatTypes.ActionType.ATTACK
var required_stance: CombatTypes.StanceType = CombatTypes.StanceType.NEUTRAL
var resulting_stance: CombatTypes.StanceType = CombatTypes.StanceType.NEUTRAL
var is_self_targeted: bool = false
var is_disruptive: bool = false
var command_name: String = "Command"
var execution_log: String = ""

# Planner-controlled commands pay AP when they START and may switch stance then.
# (Scripted commands leave these at their defaults and keep the legacy rules.)
var planner_controlled: bool = false
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