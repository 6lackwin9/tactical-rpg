extends RefCounted

# Balance sim `role` mode (BAL2_SIM.md sections 6.4 and 8.1): each class runs the limit test of a
# role under the variant grid spend {class, cons, primary} x preset {balanced, aggressive} (quick:
# one variant, one seed). Per (class, scenario) every KPI prints as "mean [floor..ceiling]" over the
# variants; a primary class's range outside the band, a missed ranking and a primary clear rate
# outside the scenario band are marked X. Loaded with preload (no class_name).

const SimArgs = preload("res://tools/sim/sim_args.gd")
const SimRunner = preload("res://tools/sim/sim_runner.gd")
const SimMetrics = preload("res://tools/sim/sim_metrics.gd")
const SimScenarios = preload("res://tools/sim/sim_scenarios.gd")
const SimRoles = preload("res://tools/sim/sim_roles.gd")
const SimTargets = preload("res://tools/sim/sim_targets.gd")

const GRID_SPENDS: Array[String] = ["class", "cons", "primary"]
const GRID_PRESETS: Array[String] = ["balanced", "aggressive"]
const SUPPORT_BASELINE_CLASS: String = "brawler"   # 8.1 support band: survivors versus this class in the slot

var args: SimArgs
var runner: SimRunner
var metrics: SimMetrics

func _init(p_args: SimArgs, p_runner: SimRunner, p_metrics: SimMetrics) -> void:
	args = p_args
	runner = p_runner
	metrics = p_metrics

func spends() -> Array[String]:
	return GRID_SPENDS.slice(0, 1) if args.quick else GRID_SPENDS.duplicate()

func presets() -> Array[String]:
	return GRID_PRESETS.slice(0, 1) if args.quick else GRID_PRESETS.duplicate()

# Variant records of `class_id` in `scenario_id` (one per spend x preset). `plain`: the test slot is
# a reference unit (class spend, default loadout; the support baseline).
func run_class(scenario_id: String, class_id: String, plain: bool = false) -> Array[Dictionary]:
	var sc: Dictionary = SimScenarios.scenario(runner, scenario_id, class_id, args.level, plain)
	var party: Array[String] = []
	party.assign(DictUtil.array_of(sc["party"]))
	var foes: Array[String] = []
	foes.assign(DictUtil.array_of(sc["enemies"]))
	var refs: Array[String] = []
	refs.assign(DictUtil.array_of(sc["refs"]))
	var variants: Array[Dictionary] = []
	runner.enemy_block = SimScenarios.scenario_block(scenario_id)
	for sv: String in spends():
		runner.spend_variant = sv
		for pv: String in presets():
			runner.preset = pv
			var acc: Dictionary = SimScenarios.empty_set()
			SimScenarios.run_seeds(runner, metrics, party, foes, {}, args.seeds(), str(sc["priority"]), acc)
			if DictUtil.to_float(acc["fights"]) <= 0.0:
				continue
			var test_names: Array[String] = [str(sc["test"])]
			acc["test"] = SimScenarios.sum_units(acc, test_names)
			acc["party"] = SimScenarios.sum_units(acc, party)
			acc["refs"] = SimScenarios.sum_units(acc, refs)
			acc["cap"] = args.cap
			variants.append(acc)
	runner.spend_variant = "class"
	runner.preset = "balanced"
	runner.enemy_block = Vector2i.ZERO
	return variants

func role_mode() -> void:
	out("ROLE level %d: variants spend %s x preset %s x %d seeds; KPI cells mean [floor..ceiling] over the variants; P primary, S secondary; X = outside the section 8.1 band (primary) or ranking miss" % [
		args.level, ",".join(spends()), ",".join(presets()), args.seed_count])
	var baseline: float = -1.0
	for role: String in args.roles:
		var scenario_id: String = str(SimScenarios.ROLE_SCENARIO[role])
		if role == "support" and baseline < 0.0:
			var base_runs: Array[Dictionary] = run_class(scenario_id, SUPPORT_BASELINE_CLASS, true)
			baseline = DictUtil.to_float(SimRoles.spread("survivors", base_runs)["mean"])
			out("support baseline: reference %s in the support slot of %s: %.2f survivors per fight" % [SUPPORT_BASELINE_CLASS, scenario_id, baseline])
		var ks: Array[Dictionary] = SimRoles.kpis(role)
		out("%s (%s) - %s" % [scenario_id, SimRoles.ROLE_LABEL[role], _bands_line(ks)])
		var header: String = "%-8s %-2s %-18s %-20s" % ["class", "", "clear (%s)" % SimTargets.band_text(args.targets, "scenario_clear"), "survivors"]
		for k: Dictionary in ks:
			header += " %-22s" % str(k["label"])
		out(header + " rank")
		var results: Dictionary = {}
		var means: Dictionary = {}
		for class_id: String in args.classes:
			var runs: Array[Dictionary] = run_class(scenario_id, class_id)
			if role == "support":
				for v: Dictionary in runs:
					v["baseline_survivors"] = baseline
			results[class_id] = runs
			means[class_id] = DictUtil.to_float(SimRoles.spread(str(ks[0]["id"]), runs)["mean"])
		var ranks: Dictionary = SimRoles.rank_marks(role, means)
		for class_id: String in args.classes:
			var runs_c: Array[Dictionary] = []
			runs_c.assign(DictUtil.array_of(results[class_id]))
			if runs_c.is_empty():
				out("%-8s n/a (every variant refused)" % class_id)
				continue
			var fit: String = SimRoles.fit(class_id, role)
			var primary: bool = fit == "P"
			var clear_s: Dictionary = SimRoles.spread("clear", runs_c)
			var clear_text: String = SimRoles.cell({"pct": true, "band": "scenario_clear"}, clear_s, args.targets, false)
			if primary:
				clear_text += SimTargets.mark(args.targets, "scenario_clear", DictUtil.to_float(clear_s["mean"]))
			var line: String = "%-8s %-2s %-18s %-20s" % [class_id, fit, clear_text, SimRoles.cell({"pct": false, "band": ""}, SimRoles.spread("survivors", runs_c), args.targets, false)]
			for k: Dictionary in ks:
				line += " %-22s" % SimRoles.cell(k, SimRoles.spread(str(k["id"]), runs_c), args.targets, primary)
			out(line + " " + str(ranks.get(class_id, "")))
		out("loadouts: " + _loadouts_line(role))
	out("block misses: %d" % runner.block_misses)
	if runner.block_misses > 0:
		out("BLOCK MISS: %d fights where the enemy block found no room (enemies stayed put); results may be skewed" % runner.block_misses)

func _bands_line(ks: Array[Dictionary]) -> String:
	var parts: Array[String] = []
	for k: Dictionary in ks:
		if not str(k["band"]).is_empty():
			parts.append("%s %s" % [str(k["label"]), SimTargets.band_text(args.targets, str(k["band"]))])
	return "bands: " + ", ".join(parts) + "; rank on %s (%s first)" % [str(ks[0]["label"]), "lowest" if DictUtil.to_bool(ks[0]["lower"]) else "highest"]

func _loadouts_line(role: String) -> String:
	var parts: Array[String] = []
	for class_id: String in args.classes:
		parts.append("%s [%s]" % [class_id, ",".join(SimRoles.role_loadout(class_id, role, args.level))])
	return "; ".join(parts)

static func out(text: String) -> void:
	print("§ " + text)
