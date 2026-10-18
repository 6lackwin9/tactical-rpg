extends SceneTree

# Aegis battle traces (engine/docs/ENGINE_SPEC.md section 6): runs balance-sim duels through the shipped SimRunner
# (tools/sim/sim_runner.gd, unchanged) and records every step, so the Rust sim can be replayed against them seed by
# seed. Works on a sandbox fork of the catalog like tools/balance_sim.gd; the shipped catalog is never written.
#   godot --headless --path <project> -s <abs path> -- out=<abs dir> [level=1] [seeds=2] [classes=warrior,mage]
#     [preset=balanced] [jitter=on|off]
# Writes traces.json: {"units": {id: entry}, "fights": [{left, right, seed, preset, start, rounds: [...], result}]}.

const SimArgs = preload("res://tools/sim/sim_args.gd")
const SimRunner = preload("res://tools/sim/sim_runner.gd")
const Observer = preload("res://scripts/combat/session/battle_observer.gd")

class Recorder extends Observer:
	var fights: Array = []
	var current: Dictionary = {}
	var last_round: int = -1

	func on_start(session: CombatSession) -> void:
		current = {"start": Recorder.units_state(session), "rounds": []}
		last_round = -1

	func on_step(session: CombatSession) -> void:
		if session.current_round != last_round:
			last_round = session.current_round
			var actions: Array = []
			for a: CombatResolver.ScheduledAction in session.resolver.scheduled_actions:
				actions.append([a.combatant.name, a.command.command_name, a.start_block, a.end_block, a.command.ap_cost, a.command.stance_switch_cost])
			current["rounds"].append({"round": session.current_round, "actions": actions, "steps": []})
		var rounds: Array = current["rounds"]
		var step: Dictionary = {"block": session.current_block, "phase": int(session.phase), "units": Recorder.units_state(session)}
		var rd: Dictionary = rounds[rounds.size() - 1]
		var steps: Array = rd["steps"]
		steps.append(step)

	func on_end(session: CombatSession, result: Dictionary) -> void:
		current["result"] = {"l": result.get("l", 0.0), "r": result.get("r", 0.0), "rounds": result.get("rounds", 0), "outcome": session.outcome}
		fights.append(current)

	static func units_state(session: CombatSession) -> Array:
		var out: Array = []
		for c: Combatant in session.allies + session.enemies:
			var pos: Vector2i = session.grid.get_position(c)
			var active: Array = c.statuses.keys()
			var pending: Array = c.pending_statuses.keys()
			active.sort()
			pending.sort()
			out.append([c.name, c.hp, c.ap, pos.x, pos.y, int(c.current_stance), active, pending, c.is_withdrawn])
		return out

var out_dir: String = ""
var sandbox: String = ""

func _initialize() -> void:
	var level: int = 1
	var seed_count: int = 2
	var preset: String = "balanced"
	var jitter: String = "on"
	var classes: Array[String] = []
	for arg: String in OS.get_cmdline_user_args():
		var kv: PackedStringArray = arg.split("=", true, 1)
		if kv.size() != 2:
			continue
		match kv[0]:
			"out":
				out_dir = kv[1]
			"level":
				level = int(kv[1])
			"seeds":
				seed_count = int(kv[1])
			"preset":
				preset = kv[1]
			"jitter":
				jitter = kv[1]
			"classes":
				classes.assign(kv[1].split(","))
	var args := SimArgs.new()
	if not args.parse(PackedStringArray(["duel", str(level), str(seed_count), "jitter=%s" % jitter])):
		print("§ trace: argument error: ", args.error)
		quit(1)
		return
	if classes.is_empty():
		classes.assign(SimArgs.CLASSES)
	sandbox = "user://aegis_trace_%d" % OS.get_process_id()
	_clear_sandbox()
	ContentCatalog.fork_to(sandbox)
	var runner: SimRunner = SimRunner.new(args)
	var recorder := Recorder.new()
	runner.observer = recorder
	runner.preset = preset
	for c: String in classes:
		runner.define("simp_%s" % c, c, level, "hero", true)
		runner.define("sime_%s" % c, c, level, "hero", false)
	var meta: Array = []
	for a: String in classes:
		for b: String in classes:
			if a == b:
				continue
			for s: int in args.seeds():
				for pair: Array in [[a, b], [b, a]]:
					var left: Array[String] = ["simp_%s" % pair[0]]
					var right: Array[String] = ["sime_%s" % pair[1]]
					var before: int = recorder.fights.size()
					runner.fight(left, right, s)
					if recorder.fights.size() > before:
						meta.append({"left": left, "right": right, "seed": s, "preset": preset})
	var units: Dictionary = {}
	for c: String in classes:
		for id: String in ["simp_%s" % c, "sime_%s" % c]:
			units[id] = ContentCatalog.get_entry("units", id)
	var fights: Array = []
	for i: int in range(recorder.fights.size()):
		var f: Dictionary = recorder.fights[i]
		f.merge(meta[i])
		fights.append(f)
	_clear_sandbox()
	DirAccess.make_dir_recursive_absolute(out_dir)
	var path: String = out_dir.path_join("traces.json")
	var file: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	file.store_string(JSON.stringify({"units": units, "jitter": jitter, "level": level, "fights": fights}, "", false))
	file.close()
	print("§ trace: %d fights -> %s" % [fights.size(), path])
	quit()

func _clear_sandbox() -> void:
	for kind: String in ContentSchema.KINDS:
		DirAccess.remove_absolute(ContentCatalog.path_for(kind, sandbox))
