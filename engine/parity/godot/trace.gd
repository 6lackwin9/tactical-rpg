extends SceneTree

# Aegis battle traces (engine/docs/ENGINE_SPEC.md section 6): runs balance-sim duels through the shipped SimRunner
# (tools/sim/sim_runner.gd, unchanged) and records every step, so the Rust sim can be replayed against them seed by
# seed. Works on a sandbox fork of the catalog like tools/balance_sim.gd; the shipped catalog is never written.
#   godot --headless --path <project> -s <abs path> -- out=<abs dir> [mode=duel|group|teams] [level=1] [seeds=2] [classes=warrior,mage]
#     [preset=balanced] [jitter=on|off]
# Writes traces_<mode>_l<level>.json: {"units": {id: entry}, "fights": [{left, right, seed, preset, start, rounds: [...], result}]}.
# Modes: duel (every class pair), group (hero vs 3 of its class at low tiers), teams (mixed trios).

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
			var round_list: Array = current["rounds"]
			round_list.append({"round": session.current_round, "actions": actions, "steps": []})
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
	var mode: String = "duel"
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
			"mode":
				mode = kv[1]
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
	var matchups: Array = []   # [left ids, right ids]
	match mode:
		"group":   # balance_sim group mode: a hero vs 3 of its class at minion / standard tiers
			var low: int = maxi(1, level - 2)
			for c: String in classes:
				runner.define("simp_%s" % c, c, level, "hero", true)
				for tier: Array in [["minion_low", low, "minion"], ["standard_low", low, "standard"], ["minion_eq", level, "minion"]]:
					var foe: String = "sime_%s_%s" % [c, tier[0]]
					runner.define(foe, c, DictUtil.to_int(tier[1]), str(tier[2]), false)
					matchups.append([["simp_%s" % c], [foe, foe, foe]])
		"teams":   # mixed three-unit parties, both sides AI (taunt, heals, areas, buffs)
			var comps: Array = [[["warrior", "priest", "mage"], ["ranger", "rogue", "brawler"]], [["warrior", "ranger", "priest"], ["brawler", "rogue", "mage"]],
				[["warrior", "priest", "rogue"], ["warrior", "mage", "ranger"]], [["brawler", "mage", "priest"], ["warrior", "ranger", "priest"]]]
			for c: String in SimArgs.CLASSES:
				runner.define("simp_%s" % c, c, level, "hero", true)
				runner.define("sime_%s" % c, c, level, "hero", false)
			for comp: Array in comps:
				var p_ids: Array[String] = []
				var e_ids: Array[String] = []
				for c: Variant in comp[0]:
					p_ids.append("simp_%s" % str(c))
				for c: Variant in comp[1]:
					e_ids.append("sime_%s" % str(c))
				matchups.append([p_ids, e_ids])
				var p_swap: Array[String] = []
				var e_swap: Array[String] = []
				for c: Variant in comp[1]:
					p_swap.append("simp_%s" % str(c))
				for c: Variant in comp[0]:
					e_swap.append("sime_%s" % str(c))
				matchups.append([p_swap, e_swap])
		_:   # duel: every ordered class pair, both sides
			for c: String in classes:
				runner.define("simp_%s" % c, c, level, "hero", true)
				runner.define("sime_%s" % c, c, level, "hero", false)
			for a: String in classes:
				for b: String in classes:
					if a != b:
						matchups.append([["simp_%s" % a], ["sime_%s" % b]])
						matchups.append([["simp_%s" % b], ["sime_%s" % a]])
	var meta: Array = []
	var units: Dictionary = {}
	for m: Array in matchups:
		var left: Array[String] = []
		var right: Array[String] = []
		left.assign(DictUtil.array_of(m[0]))
		right.assign(DictUtil.array_of(m[1]))
		for s: int in args.seeds():
			var before: int = recorder.fights.size()
			runner.fight(left, right, s)
			if recorder.fights.size() > before:
				meta.append({"left": left, "right": right, "seed": s, "preset": preset})
		for id: String in left + right:
			units[id] = ContentCatalog.get_entry("units", id)
	var fights: Array = []
	for i: int in range(recorder.fights.size()):
		var f: Dictionary = recorder.fights[i]
		var info: Dictionary = meta[i]
		f.merge(info)
		fights.append(f)
	_clear_sandbox()
	DirAccess.make_dir_recursive_absolute(out_dir)
	var path: String = out_dir.path_join("traces_%s_l%d.json" % [mode, level])
	var file: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	file.store_string(JSON.stringify({"units": units, "jitter": jitter, "level": level, "fights": fights}, "", false))
	file.close()
	print("§ trace: %d fights -> %s" % [fights.size(), path])
	quit()

func _clear_sandbox() -> void:
	for kind: String in ContentSchema.KINDS:
		DirAccess.remove_absolute(ContentCatalog.path_for(kind, sandbox))
