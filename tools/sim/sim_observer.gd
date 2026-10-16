extends RefCounted

# Fight observer hook of the balance sim runner (tools/sim/sim_runner.gd). The base does nothing;
# the commit 1b metrics collector extends it (reads session.log_entries and samples units per step).

# After session.start() and the start-distance placement, before the first plan.
func on_start(_session: CombatSession) -> void:
	pass

# After every session.step_next().
func on_step(_session: CombatSession) -> void:
	pass

# After the last round, before session.dispose(); `result` is the runner's fight record.
func on_end(_session: CombatSession, _result: Dictionary) -> void:
	pass
