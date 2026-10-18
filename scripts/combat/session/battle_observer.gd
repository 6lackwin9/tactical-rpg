class_name BattleObserver
extends RefCounted

# ==============================================================================
# Tactical RPG - Battle observer hooks (BREPORT1-L 1.1, moved from tools/sim/sim_observer.gd)
# The base does nothing. The balance sim runner (tools/sim/sim_runner.gd) calls the four hooks; the per-unit
# collector BattleMetrics extends it (and can also attach to a live session, BREPORT1 1.2).
# ==============================================================================

# After session.start() and the start-distance placement, before the first plan.
func on_start(_session: CombatSession) -> void:
	pass

# After every session.step_next().
func on_step(_session: CombatSession) -> void:
	pass

# After session.advance_round() (statuses rolled over, AP recovered), before the next plan.
func on_round(_session: CombatSession) -> void:
	pass

# After the last round, before session.dispose(); `result` is the runner's fight record.
func on_end(_session: CombatSession, _result: Dictionary) -> void:
	pass
