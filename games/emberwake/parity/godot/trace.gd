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
	var pending_intents: Array = []   # the tells read while the round was being planned (INTENT1)

	func on_start(session: CombatSession) -> void:
		current = {"start": Recorder.units_state(session), "rounds": []}
		last_round = -1
		pending_intents = Recorder.intents_state(session)

	func on_round(session: CombatSession) -> void:
		pending_intents = Recorder.intents_state(session)

	func on_step(session: CombatSession) -> void:
		if session.current_round != last_round:
			last_round = session.current_round
			var actions: Array = []
			for a: CombatResolver.ScheduledAction in session.resolver.scheduled_actions:
				actions.append([a.combatant.name, a.command.command_name, a.start_block, a.end_block, a.command.ap_cost, a.command.stance_switch_cost])
			var round_list: Array = current["rounds"]
			round_list.append({"round": session.current_round, "actions": actions, "intents": pending_intents, "steps": []})
		var rounds: Array = current["rounds"]
		var step: Dictionary = {"block": session.current_block, "phase": int(session.phase), "units": Recorder.units_state(session)}
		var rd: Dictionary = rounds[rounds.size() - 1]
		var steps: Array = rd["steps"]
		steps.append(step)

	func on_end(session: CombatSession, result: Dictionary) -> void:
		current["result"] = {"l": result.get("l", 0.0), "r": result.get("r", 0.0), "rounds": result.get("rounds", 0), "outcome": session.outcome}
		fights.append(current)

	# The enemy tells (CombatSession.enemy_intents) in the engine's trace shape (emberwake_battle::intent::trace): per
	# enemy [name, line, revealed, enraged, [reader names], [plan lines], morale state, morale group]; a line is
	# [kind, target name or "", cell x, cell y, start block, end block, skill].
	static func intents_state(session: CombatSession) -> Array:
		var out: Array = []
		for r: Dictionary in session.enemy_intents():
			var readers: Array = []
			for c: Combatant in r["readers"]:
				readers.append(c.name)
			var plan: Array = []
			for p: Dictionary in r["plan"]:
				plan.append(Recorder.intent_line(p))
			var unit: Combatant = r["unit"]
			out.append([unit.name, Recorder.intent_line(r), r["revealed"], r["enraged"], readers, plan, r.get("morale_state", ""), r.get("morale_group", -1)])
		return out

	static func intent_line(r: Dictionary) -> Array:
		var target: Combatant = r["target"]
		var cell: Vector2i = r["cell"]
		var blocks: Vector2i = r["blocks"]
		return [r["kind"], target.name if target != null else "", cell.x, cell.y, blocks.x, blocks.y, r["skill"]]

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
	var pouch: String = ""   # pouch=<id,id,id>: every party unit carries these and may use them (SimArgs pouch)
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
			"pouch":
				pouch = kv[1]
			"mode":
				mode = kv[1]
	if mode == "world":
		_world_traces(seed_count, preset)
		quit()
		return
	var args := SimArgs.new()
	var sim_args: PackedStringArray = PackedStringArray(["duel", str(level), str(seed_count), "jitter=%s" % jitter])
	if not pouch.is_empty():
		sim_args.append("pouch=%s" % pouch)
	if not args.parse(sim_args):
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
		"encounters":   # every catalog encounter as authored (zone, exits, waves...), its own party vs its enemies
			for enc_id: String in ContentCatalog.ids("encounters"):
				var enc: Dictionary = ContentCatalog.get_entry("encounters", enc_id)
				var base: Dictionary = enc.duplicate(true)
				for key: String in ["party", "enemies", "max_rounds", "rng_seed"]:
					base.erase(key)
				matchups.append([DictUtil.array_of(enc.get("party", [])), DictUtil.array_of(enc.get("enemies", [])), base])
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
		var base: Dictionary = DictUtil.dict_of(m[2]) if m.size() > 2 else {}
		for s: int in args.seeds():
			var before: int = recorder.fights.size()
			runner.fight(left, right, s, base)
			if recorder.fights.size() > before:
				meta.append({"left": left, "right": right, "seed": s, "preset": preset, "base": base})
		var roster: Array[String] = left + right
		for raw: Variant in DictUtil.array_of(base.get("wave_units", [])) + DictUtil.array_of(base.get("guests", [])):
			roster.append(str(raw))
		for id: String in roster:
			units[id] = ContentCatalog.get_entry("units", id)
	var fights: Array = []
	for i: int in range(recorder.fights.size()):
		var f: Dictionary = recorder.fights[i]
		var info: Dictionary = meta[i]
		f.merge(info)
		fights.append(f)
	_clear_sandbox()
	DirAccess.make_dir_recursive_absolute(out_dir)
	var path: String = out_dir.path_join("traces_%s_l%d%s.json" % [mode, level, "" if pouch.is_empty() else "_items"])
	var file: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	file.store_string(JSON.stringify({"units": units, "jitter": jitter, "level": level, "pouch": pouch, "fights": fights}, "", false))
	file.close()
	print("§ trace: %d fights -> %s" % [fights.size(), path])
	quit()

# mode=world: every world's own encounters as WorldEncounter plays them (its own party of world characters, guests,
# objectives, phases, morale; start buffs on when the encounter has any), both sides drafted by the AI with `preset`.
# Seeds 1..seed_count replace the authored rng_seed. Nothing is written to the catalogs. Writes traces_world_l0.json:
# {"world": true, "fights": [{world, encounter, seed, preset, start_buffs, start, rounds, result}]}.
func _world_traces(seed_count: int, preset: String) -> void:
	var recorder := Recorder.new()
	var meta: Array = []
	# Synthetic variants (the authored entry with a few fields replaced) cover the rules no shipped encounter uses yet.
	var variants: Array[Dictionary] = [{}, {"objective": "escape", "objective_count": 1}, {"deployment": "centre"},
		{"morale": true, "morale_break": 70}, {"objective": "hold", "objective_tiles": [[5, 3], [6, 5]], "objective_rounds": 2},
		{"start_status": [{"status": "haste", "rounds": 2}, {"status": "fortify", "rounds": 3, "magnitude": 4}]}]
	for w: String in WorldCatalog.world_ids():
		for e: Dictionary in WorldCatalog.own_entries(w, "encounters"):
			for v: int in range(variants.size()):
				var over: Dictionary = variants[v]
				if v > 0 and str(e["id"]) not in ["ash_patrol", "ash_raiders", "ashmaw_stand", "strand_clash"]:
					continue
				var def_entry: Dictionary = e.duplicate(true)
				def_entry.merge(over, true)
				var start_buffs: bool = not DictUtil.array_of(def_entry.get("start_status", [])).is_empty()
				for s: int in range(1, seed_count + 1):
					var enc := WorldEncounter.new(w, str(e["id"]))
					enc.apply_definition(def_entry)
					enc.rng_seed = s
					enc.grant_all_start = start_buffs
					var session := CombatSession.new(enc)
					session.start()
					recorder.on_start(session)
					for _r: int in range(enc.max_rounds):
						session.plan_apply_preset_all(preset)
						if not session.confirm_plan():
							break
						for _i: int in range(600):
							if session.phase == CombatSession.Phase.REPLAY:
								break
							session.step_next()
							recorder.on_step(session)
						if session.combat_over or not session.can_advance_round():
							break
						session.advance_round()
						recorder.on_round(session)
					recorder.on_end(session, {"rounds": session.current_round})
					meta.append({"world": w, "encounter": str(e["id"]), "override": over, "seed": s, "preset": preset, "start_buffs": start_buffs})
					session.dispose()
	var fights: Array = []
	for i: int in range(recorder.fights.size()):
		var f: Dictionary = recorder.fights[i]
		f.merge(DictUtil.dict_of(meta[i]))
		fights.append(f)
	DirAccess.make_dir_recursive_absolute(out_dir)
	var path: String = out_dir.path_join("traces_world_%s.json" % preset)
	var file: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	file.store_string(JSON.stringify({"world": true, "fights": fights}, "", false))
	file.close()
	print("§ trace: %d world fights -> %s" % [fights.size(), path])

func _clear_sandbox() -> void:
	for kind: String in ContentSchema.KINDS:
		DirAccess.remove_absolute(ContentCatalog.path_for(kind, sandbox))
