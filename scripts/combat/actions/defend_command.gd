class_name DefendCommand
extends BattleCommand

var mitigation_ratio: float

func _init(p_source: Combatant, p_mitigation_ratio: float = 0.5, p_block_cost: int = 2, p_priority: int = 1) -> void:
	super(p_source, p_priority, p_block_cost)
	mitigation_ratio = p_mitigation_ratio
	command_name = "Defend"

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
