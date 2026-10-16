extends SceneTree

# Balance simulator: headless AI-vs-AI battles on a sandbox copy of the catalog (BAL2_SIM.md).
#   tools/fc.sh sim <mode> [level] [seeds] [key=value ...]     (tools/fc.sh sim help: the usage)
#   modes duel | tier | levels | group | stats; keys cap, smart_skills, loadout, spend, preset,
#   distance, score, quick, metrics (tools/sim/sim_args.gd).
# Scores per fight (left side): binary (KO, else the HP lead with a 0.02 band) and HP margin
# (KO 1 / 0, else 0.5 + (l - r) / 2); cells print "bin/hp" in percent. Duel, tier, levels and group
# run every matchup with sides swapped over every seed, so side/position bias cancels out.
# Helpers under tools/sim/ (preload, no class_name). The shipped catalog is never written.

const SimArgs = preload("res://tools/sim/sim_args.gd")
const SimRunner = preload("res://tools/sim/sim_runner.gd")
const SimVariants = preload("res://tools/sim/sim_variants.gd")

const CLASSES: Array[String] = ["warrior", "ranger", "priest", "brawler", "rogue", "mage"]
const TIER_ORDER: Array[String] = ["minion", "standard", "elite", "boss"]
const LEVEL_STEPS: Array[int] = [1, 3, 5, 7]
const STATS_LEVELS: Array[int] = [1, 3, 5, 7]

var sandbox: String = ""   # per process, so sims can run in parallel
var args: SimArgs = SimArgs.new()
var runner: SimRunner = null

func _initialize() -> void:
	if not args.parse(OS.get_cmdline_user_args()):
		_out("SIM argument error: " + args.error)
		for line: String in SimArgs.usage():
			_out(line)
		quit()
		return
	sandbox = "user://balance_sim_%d" % OS.get_process_id()
	_clear_sandbox()
	ContentCatalog.fork_to(sandbox)
	runner = SimRunner.new(args)
	_out(args.header())
	match args.mode:
		"duel":
			_duel(args.level)
		"tier":
			_tier(args.level)
		"levels":
			_levels()
		"group":
			_group(args.level)
		"stats":
			_stats()
	runner.distance_note()
	if args.metrics:
		_out("metrics: the per-unit collector arrives with BAL2-L commit 1b (nothing collected)")
	_clear_sandbox()
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	quit()

func _clear_sandbox() -> void:
	for kind: String in ContentSchema.KINDS:
		DirAccess.remove_absolute(ContentCatalog.path_for(kind, sandbox))

func _ids(id: String, count: int = 1) -> Array[String]:
	var out: Array[String] = []
	for i: int in range(count):
		out.append(id)
	return out

# Row average of score cells.
func _avg(results: Array[Dictionary]) -> String:
	var b: float = 0.0
	var h: float = 0.0
	var n: int = 0
	for res: Dictionary in results:
		if DictUtil.to_int(res.get("n", 0)) > 0:
			b += DictUtil.to_float(res["bin"])
			h += DictUtil.to_float(res["hp"])
			n += 1
	if n == 0:
		return "n/a"
	return runner.cell({"bin": b / float(n), "hp": h / float(n), "n": n})

func _duel(level: int) -> void:
	for c: String in CLASSES:
		runner.define("simp_%s" % c, c, level, "hero", true)
		runner.define("sime_%s" % c, c, level, "hero", false)
	_out("DUEL level %d, hero vs hero, score of the ROW class (%s)" % [level, _score_label()])
	var header: String = "%-10s" % ""
	for c: String in CLASSES:
		header += "%-10s" % c
	_out(header + "avg")
	for a: String in CLASSES:
		var line: String = "%-10s" % a
		var row: Array[Dictionary] = []
		for b: String in CLASSES:
			if a == b:
				line += "%-10s" % "-"
				continue
			var res: Dictionary = runner.matchup(_ids("simp_%s" % a), _ids("sime_%s" % b))
			row.append(res)
			line += "%-10s" % runner.cell(res)
		_out(line + "%-10s" % _avg(row) + _band_flag(row))

# Duel diagnostic band on the row's mean hp score (key band=lo-hi): "LOW" / "HIGH" / "" (not a gate).
func _band_flag(results: Array[Dictionary]) -> String:
	var h: float = 0.0
	var n: int = 0
	for res: Dictionary in results:
		if DictUtil.to_int(res.get("n", 0)) > 0:
			h += DictUtil.to_float(res["hp"])
			n += 1
	if n == 0:
		return ""
	var pct: int = roundi(h / float(n) * 100.0)
	if pct < args.band_lo:
		return "LOW (band %d-%d)" % [args.band_lo, args.band_hi]
	if pct > args.band_hi:
		return "HIGH (band %d-%d)" % [args.band_lo, args.band_hi]
	return ""

func _tier(level: int) -> void:
	_out("TIER level %d: hero class vs its own class at a tier (score of the hero, %s)" % [level, _score_label()])
	for c: String in CLASSES:
		runner.define("simp_%s" % c, c, level, "hero", true)
		var line: String = "%-10s" % c
		for tier: String in TIER_ORDER:
			var foe: String = "sime_%s_%s" % [c, tier]
			runner.define(foe, c, level, tier, false)
			var p: Array[String] = _ids("simp_%s" % c)
			line += "%s 1v1 %s / 1v3 %s   " % [tier, runner.cell(runner.matchup(p, _ids(foe))), runner.cell(runner.matchup(p, _ids(foe, 3)))]
		_out(line)

func _levels() -> void:
	_out("LEVELS: class at level N vs the same class at N+2 (score of the lower level, %s)" % _score_label())
	for c: String in CLASSES:
		var line: String = "%-10s" % c
		for lvl: int in LEVEL_STEPS:
			runner.define("simp_%s" % c, c, lvl, "hero", true)
			runner.define("sime_%s" % c, c, lvl + 2, "hero", false)
			line += "L%d vs L%d: %s   " % [lvl, lvl + 2, runner.cell(runner.matchup(_ids("simp_%s" % c), _ids("sime_%s" % c)))]
		_out(line)

# Hero L vs 3 minions at max(1, L-2), 3 standard at max(1, L-2), 3 minions at L (expected loss).
func _group(level: int) -> void:
	var low: int = maxi(1, level - 2)
	_out("GROUP level %d: hero vs 3 of its class (score of the hero, %s)" % [level, _score_label()])
	for c: String in CLASSES:
		runner.define("simp_%s" % c, c, level, "hero", true)
		runner.define("sime_%s_minion_low" % c, c, low, "minion", false)
		runner.define("sime_%s_standard_low" % c, c, low, "standard", false)
		runner.define("sime_%s_minion_eq" % c, c, level, "minion", false)
		var p: Array[String] = _ids("simp_%s" % c)
		var line: String = "%-10s" % c
		line += "3 minion L%d %s   " % [low, runner.cell(runner.matchup(p, _ids("sime_%s_minion_low" % c, 3)))]
		line += "3 standard L%d %s   " % [low, runner.cell(runner.matchup(p, _ids("sime_%s_standard_low" % c, 3)))]
		line += "3 minion L%d %s" % [level, runner.cell(runner.matchup(p, _ids("sime_%s_minion_eq" % c, 3)))]
		_out(line)

# No fights: per class the derived numbers of the party-side unit (spend / loadout variants apply).
func _stats() -> void:
	var levels: Array[int] = []
	levels.assign(STATS_LEVELS if args.all_levels else [args.level])
	_out("STATS: HP, armor rating, reduction %, EHP = HP / (1 - reduction), melee / ranged power, range, speed, crit %, active class families")
	_out("%-8s %-3s %-16s %5s %5s %5s %5s %5s %5s %5s %5s %6s  %s" % ["class", "L", "spend", "HP", "armor", "red%", "EHP", "melee", "rngd", "range", "speed", "crit%", "families"])
	for lvl: int in levels:
		for c: String in CLASSES:
			for sv: String in args.spend_variants():
				runner.spend_variant = sv
				runner.define("simp_%s" % c, c, lvl, "hero", true)
				var u: Combatant = runner.build("simp_%s" % c)
				if u == null:
					continue
				var rating: int = DamageCalculator.armor_rating(u)
				var red: float = DamageCalculator.armor_reduction(rating)
				var ehp: int = roundi(float(u.max_hp) / (1.0 - red))
				var spend_text: String = sv
				var spend: Dictionary = SimVariants.spend_for(sv, c, lvl)
				if not spend.is_empty():
					spend_text += str(spend).replace(" ", "").replace("\"", "")
				_out("%-8s %-3d %-16s %5d %5d %5.1f %5d %5d %5d %5d %5d %6.1f  %s" % [c, lvl, spend_text, u.max_hp, rating, red * 100.0, ehp,
					DamageCalculator.power_of(u, false), DamageCalculator.power_of(u, true), u.attack_range, u.speed,
					DamageCalculator.crit_chance(u), ",".join(u.active_class_families)])
	runner.spend_variant = "class"

func _score_label() -> String:
	match args.score:
		"hp":
			return "hp margin %"
		"binary":
			return "binary %"
	return "bin/hp %"

func _out(text: String) -> void:
	print("§ " + text)
