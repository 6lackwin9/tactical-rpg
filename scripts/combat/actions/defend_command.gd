class_name DefendCommand
extends BattleCommand

var mitigation_ratio: float

func _init(p_source: Combatant, p_mitigation_ratio: float = 0.5, p_block_cost: int = 2, p_priority: int = 1) -> void:
	super(p_source, p_priority, p_block_cost)
	mitigation_ratio = p_mitigation_ratio
	command_name = "Defend"

var grant_status: String = ""   # buff made active the moment the defend starts (StatusEffects.defend_buffs(), e.g. riposte)
var status_rounds: int = 1      # rounds it lasts, counting the current one

# Runs after AP and the (Defensive) stance switch, so the riposte survives its own skill's switch.
func on_start() -> String:
	if grant_status.is_empty() or not source.is_active():
		return ""
	if StatusEffects.defend_buffs().has(grant_status):
		StatusEffects.grant_now(source, grant_status, status_rounds)
		return "[Riposte] %s readies a riposte (%d round%s, ends on a stance change)" % [source.name, status_rounds, "" if status_rounds == 1 else "s"]
	# A support-grant buff (taunt) is queued like every support buff: it starts next round.
	StatusEffects.queue(source, grant_status, status_rounds)
	return "[%s] %s will be %s next round (%d round%s)" % [grant_status.capitalize(), source.name, StatusEffects.display_name(grant_status).to_lower(), status_rounds, "" if status_rounds == 1 else "s"]

func execute() -> void:
	if not source.is_alive():
		return
	source.defense_mitigation_ratio = mitigation_ratio
	var mit_pct: int = int(mitigation_ratio * 100.0)
	print("[%s] assumes a Defensive Stance and raises guard (mitigates %d%% incoming damage)!" % [
		source.name,
		mit_pct
	])
	execution_log = "[Defend] %s raised guard (mitigates %d%% damage, Defensive Stance)" % [
		source.name, mit_pct
	]
