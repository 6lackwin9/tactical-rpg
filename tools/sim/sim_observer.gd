extends RefCounted

# Fight observer hook of the balance sim runner (tools/sim/sim_runner.gd). The base does nothing;
# the commit 1b metrics collector (tools/sim/sim_metrics.gd) extends it: it reads session.log_entries
# and samples units after every step and every block.

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
