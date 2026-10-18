extends RefCounted

# Balance sim `party` and `sweep` modes (BAL2_SIM.md sections 5, 7, 8.2). Party fights are
# asymmetric (seeds only, no side swap) over the spend x preset arguments. Tables mark values
# outside their section 8 band with X. Loaded with preload (no class_name).

const SimArgs = preload("res://tools/sim/sim_args.gd")
const SimRunner = preload("res://tools/sim/sim_runner.gd")
const SimMetrics = preload("res://tools/sim/sim_metrics.gd")
const SimScenarios = preload("res://tools/sim/sim_scenarios.gd")
const SimTargets = preload("res://tools/sim/sim_targets.gd")
const SimWorld = preload("res://tools/sim/sim_world.gd")

const SAMPLE_SEED: int = 7   # sweep dupes=on sample=n draw
const TOP_N: int = 5
const COMP_SIZE: int = 4

var args: SimArgs
var runner: SimRunner
var metrics: SimMetrics

func _init(p_args: SimArgs, p_runner: SimRunner, p_metrics: SimMetrics) -> void:
	args = p_args
	runner = p_runner
	metrics = p_metrics

# One composition against one encounter over every spend variant, preset and seed.
func run_comp(comp: Array[String], enc_id: String) -> Dictionary:
	var party: Array[String] = SimScenarios.define_party(runner, comp, args.level)
	var enc: Dictionary = world_encounter(enc_id)
	var is_world: bool = not enc.is_empty()
	if not is_world:
		enc = SimScenarios.encounter(runner, enc_id, args.level, args.elevel, args.elevel_set, args.elite_class, args.boss_class)
	runner.objective_wins = is_world
	runner.rounds_override = DictUtil.to_int(enc.get("max_rounds", 0))
	var acc: Dictionary = SimScenarios.empty_set()
	var foes: Array[String] = []
	foes.assign(DictUtil.array_of(enc["enemies"]))
	for sv: String in args.spend_variants():
		runner.spend_variant = sv
		for pv: String in args.preset_variants():
			runner.preset = pv
			SimScenarios.run_seeds(runner, metrics, party, foes, DictUtil.dict_of(enc["base"]), args.seeds(), "", acc)
	runner.spend_variant = "class"
	runner.preset = "balanced"
	runner.objective_wins = false
	runner.rounds_override = 0
	acc["party_ids"] = party
	return acc

# The world encounter of `enc=world:...` id `enc_id` (authored foes/levels unless elevel was given, the full
# objective / deployment / phases / guests, its own max_rounds); {} for any other id.
func world_encounter(enc_id: String) -> Dictionary:
	var ref: Array[String] = SimArgs.parse_ref(enc_id)
	if ref.is_empty():
		return {}
	var foe_level: int = clampi(args.level + args.elevel, 1, StatRules.MAX_LEVEL)
	return SimWorld.encounter(runner, ref[1], foe_level, args.elevel_set, ref[0], true)

# "" or " X" for a clear % against the encounter's band (generated encounters only).
func clear_mark(enc_id: String, pct: float) -> String:
	var key: String = "clear_%s" % enc_id
	return SimTargets.mark(args.targets, key, pct) if SimTargets.DEFAULTS.has(key) else ""

func band_of(enc_id: String) -> String:
	var key: String = "clear_%s" % enc_id
	return SimTargets.band_text(args.targets, key) if SimTargets.DEFAULTS.has(key) else "-"

# ------------------------------------------------------------------------------
# party
# ------------------------------------------------------------------------------

func party_mode() -> void:
	out("PARTY level %d, comp %s (%d of PARTY_MAX %d), %d fights per encounter (spend %s x preset %s x %d seeds, no side swap)" % [
		args.level, ",".join(args.comp), args.comp.size(), PartyState.PARTY_MAX,
		args.spend_variants().size() * args.preset_variants().size() * args.seed_count, ",".join(args.spend_variants()), ",".join(args.preset_variants()), args.seed_count])
	out("%-16s %8s %7s %7s %10s %9s %9s" % ["encounter", "clear%", "band", "rounds", "surv/size", "party_hp", "hp_score"])
	for enc_id: String in args.encounters:
		var acc: Dictionary = run_comp(args.comp, enc_id)
		var pct: float = SimScenarios.clear_pct(acc)
		var cleared: float = DictUtil.to_float(acc["cleared"])
		var rounds: String = "%.1f" % (DictUtil.to_float(acc["rounds_cleared_sum"]) / cleared) if cleared > 0.0 else "-"
		out("%-16s %8s %7s %7s %10s %9.2f %9.2f" % [enc_id, "%d%s" % [roundi(pct), clear_mark(enc_id, pct)], band_of(enc_id), rounds,
			"%.1f/%d" % [SimScenarios.mean(acc, "survivors_sum"), args.comp.size()], SimScenarios.mean(acc, "party_hp_sum"), SimScenarios.mean(acc, "hp_sum")])
		if args.metrics:
			print_units(acc)

# Per-unit KPI block of a fight set (party units first, then enemies).
func print_units(acc: Dictionary) -> void:
	out(SimMetrics.kpi_header())
	var per_unit: Dictionary = acc["units"]
	var party: Array = DictUtil.array_of(acc.get("party_ids", []))
	for pass_party: bool in [true, false]:
		for n: Variant in per_unit:
			if party.has(n) == pass_party:
				out(SimMetrics.kpi_line(str(n), DictUtil.dict_of(per_unit[n])))

# ------------------------------------------------------------------------------
# sweep
# ------------------------------------------------------------------------------

# 4-class compositions: the 15 distinct ones, or with dupes the 126 multisets (sample = n drawn, seed 7).
func compositions() -> Array:
	var comps: Array = []
	_combine(SimArgs.CLASSES, COMP_SIZE, 0, [], args.dupes, comps)
	if args.dupes and args.sample > 0 and args.sample < comps.size():
		var rng := RandomNumberGenerator.new()
		rng.seed = SAMPLE_SEED
		var idx: Array[int] = []
		for i: int in range(comps.size()):
			idx.append(i)
		for i: int in range(idx.size() - 1, 0, -1):   # Fisher-Yates with the seeded RNG
			var j: int = rng.randi_range(0, i)
			var t: int = idx[i]
			idx[i] = idx[j]
			idx[j] = t
		var keep: Array[int] = idx.slice(0, args.sample)
		keep.sort()
		var picked: Array = []
		for i: int in keep:
			picked.append(comps[i])
		comps = picked
	return comps

static func _combine(pool: Array[String], k: int, start: int, current: Array, dupes: bool, found: Array) -> void:
	if current.size() == k:
		var comp: Array[String] = []
		comp.assign(current)
		found.append(comp)
		return
	for i: int in range(start, pool.size()):
		var next: Array = current.duplicate()
		next.append(pool[i])
		_combine(pool, k, i if dupes else i + 1, next, dupes, found)

func sweep_mode() -> void:
	var comps: Array = compositions()
	out("SWEEP level %d: %d compositions x %d encounters (%s), %d fights each (spend %s x preset %s x %d seeds)" % [
		args.level, comps.size(), args.encounters.size(), ",".join(args.encounters),
		args.spend_variants().size() * args.preset_variants().size() * args.seed_count, ",".join(args.spend_variants()), ",".join(args.preset_variants()), args.seed_count])
	var rows: Array[Dictionary] = []
	for raw: Variant in comps:
		var comp: Array[String] = []
		comp.assign(DictUtil.array_of(raw))
		var clears: Dictionary = {}
		var sum_clear: float = 0.0
		var sum_hp: float = 0.0
		for enc_id: String in args.encounters:
			var acc: Dictionary = run_comp(comp, enc_id)
			clears[enc_id] = SimScenarios.clear_pct(acc)
			sum_clear += SimScenarios.clear_pct(acc)
			sum_hp += SimScenarios.mean(acc, "party_hp_sum")
		var n: float = float(maxi(1, args.encounters.size()))
		rows.append({"comp": comp, "key": ",".join(comp), "clear": sum_clear / n, "hp": sum_hp / n, "per_enc": clears})
	var ranked: Array[Dictionary] = rows.duplicate()
	ranked.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		var ca: float = DictUtil.to_float(a["clear"])
		var cb: float = DictUtil.to_float(b["clear"])
		if not is_equal_approx(ca, cb):
			return ca > cb
		if not is_equal_approx(DictUtil.to_float(a["hp"]), DictUtil.to_float(b["hp"])):
			return DictUtil.to_float(a["hp"]) > DictUtil.to_float(b["hp"])
		return str(a["key"]) < str(b["key"]))
	var header: String = "%-4s %-34s %6s %5s" % ["rank", "composition", "clear", "hp"]
	for enc_id: String in args.encounters:
		header += " %8s" % enc_id.left(8)
	out(header + "   (line < %s%% marked X)" % SimTargets._num(SimTargets.lo(args.targets, "line_min")))
	for i: int in range(ranked.size()):
		var r: Dictionary = ranked[i]
		var line: String = "%-4d %-34s %6d %5.2f" % [i + 1, str(r["key"]), roundi(DictUtil.to_float(r["clear"])), DictUtil.to_float(r["hp"])]
		var per_enc: Dictionary = r["per_enc"]
		for enc_id: String in args.encounters:
			var v: float = DictUtil.to_float(per_enc[enc_id])
			line += " %8s" % ("%d%s" % [roundi(v), SimTargets.mark(args.targets, "line_min", v) if enc_id == "line" else ""])
		out(line)
	_sweep_summary(rows, ranked)

func _sweep_summary(rows: Array[Dictionary], ranked: Array[Dictionary]) -> void:
	var means: String = "encounter clear % (mean over compositions):"
	for enc_id: String in args.encounters:
		var s: float = 0.0
		for r: Dictionary in rows:
			s += DictUtil.to_float(DictUtil.dict_of(r["per_enc"])[enc_id])
		var m: float = s / float(maxi(1, rows.size()))
		means += "  %s %d%s (%s)" % [enc_id, roundi(m), clear_mark(enc_id, m), band_of(enc_id)]
	out(means)
	var by_key: Dictionary = {}
	for r: Dictionary in rows:
		by_key[_sorted_key(DictUtil.array_of(r["comp"]))] = DictUtil.to_float(r["clear"])
	out("%-10s %12s %6s %12s" % ["class", "swap_delta", "pairs", "top%d_count" % TOP_N])
	for x: String in SimArgs.CLASSES:
		var delta_sum: float = 0.0
		var pairs: int = 0
		for r: Dictionary in rows:
			var comp: Array = DictUtil.array_of(r["comp"])
			if comp.has(x):
				continue
			var seen_y: Dictionary = {}
			for y: Variant in comp:
				if seen_y.has(y):
					continue
				seen_y[y] = true
				var swapped: Array = comp.duplicate()
				swapped[swapped.find(y)] = x
				var key: String = _sorted_key(swapped)
				if by_key.has(key):
					delta_sum += DictUtil.to_float(by_key[key]) - DictUtil.to_float(r["clear"])
					pairs += 1
		var top: int = 0
		for i: int in range(mini(TOP_N, ranked.size())):
			if DictUtil.array_of(ranked[i]["comp"]).has(x):
				top += 1
		var delta: float = delta_sum / float(pairs) if pairs > 0 else 0.0
		out("%-10s %12s %6d %12s" % [x, ("%+.1f%s" % [delta, SimTargets.mark(args.targets, "swap_delta", delta)]) if pairs > 0 else "n/a", pairs,
			"%d%s" % [top, SimTargets.mark(args.targets, "top5_min", float(top))]])

static func _sorted_key(comp: Array) -> String:
	var order: Array[String] = []
	for c: Variant in comp:
		order.append(str(c))
	order.sort_custom(func(a: String, b: String) -> bool: return SimArgs.CLASSES.find(a) < SimArgs.CLASSES.find(b))
	return ",".join(order)

static func out(text: String) -> void:
	print("§ " + text)
