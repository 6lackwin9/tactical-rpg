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
var reactions: Array[Dictionary] = []   # counters this command provoked (RiposteRules.counter results), logged after it

var ap_cost: int = 0                 # skill AP (excludes the stance switch)
var stance_switch_cost: int = 0     # extra AP for changing stance on start
var stance_on_start: int = -1       # CombatTypes.StanceType to enter at start, -1 = keep
var est_damage: int = 0              # planner's expected hit (attacks), for previews
var status_effect: String = ""        # status this command tries to inflict (StatusEffects)
var status_potency: int = 0
var status_magnitude: int = 0         # carried by the inflicted instance (unit from the status registry)
var rng: RandomNumberGenerator = null   # battle RNG, set by the resolver before execute()

# Combos (S12C-L). A flow move ends in `stance_on_end` (free, set by the resolver right after execute() unless the
# command failed); attacks carry the skill's disrupt and exploit and report what they broke.
var stance_on_end: int = -1          # CombatTypes.StanceType after execute(), -1 = none
var failed: bool = false             # execute() could not act (inactive unit, no target, out of reach): no flow
var disrupt: int = SkillDef.Disrupt.NONE
var exploit: Array[Dictionary] = []  # [{"status", "power_mult"}]
var exploit_consume: bool = false
var broken_targets: Array[Combatant] = []   # filled by execute(): units whose stance this command broke
var combo_notes: Array[String] = []         # break log lines, emitted after the reactions

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

# Exploit multiplier against `target` right now (statuses ACTIVE on it): {"status", "mult"} (SkillDef.exploit_match).
func exploit_against(target: Combatant) -> Dictionary:
	return SkillDef.exploit_match(exploit, target)

# After one damaging hit's status roll and riposte reaction (S12C-L rule 5): stance break, then consume.
# `taken` is the damage the hit dealt; `matched` the exploit_against result used for it.
func after_hit(target: Combatant, taken: int, matched: Dictionary) -> void:
	if disrupt == SkillDef.Disrupt.STANCE_BREAK and taken > 0 and target.is_active() \
			and (target.current_stance == CombatTypes.StanceType.OFFENSIVE or target.current_stance == CombatTypes.StanceType.DEFENSIVE):
		target.set_stance(CombatTypes.StanceType.TACTICAL)
		target.defense_mitigation_ratio = 0.0
		broken_targets.append(target)
		combo_notes.append("[Break] %s breaks %s's stance (now Tactical)" % [source.name, target.name])
	var status: String = str(matched.get("status", ""))
	if exploit_consume and not status.is_empty():
		StatusEffects.remove_active(target, status)

# Log suffix of an exploited hit: " (exploits Exposed)", "" when no exploit applied.
static func exploit_note(matched: Dictionary) -> String:
	var status: String = str(matched.get("status", ""))
	return "" if status.is_empty() else " (exploits %s)" % StatusEffects.display_name(status)