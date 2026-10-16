extends RefCounted

# Balance sim arguments (BAL2_SIM.md section 1): positional mode, level, seeds, then key=value
# tokens in any order. Loaded by tools/balance_sim.gd with preload (no class_name).

const MODES: Array[String] = ["duel", "tier", "levels", "group", "stats"]
const SPENDS: Array[String] = ["class", "cons", "primary", "mixed"]
const LOADOUTS: Array[String] = ["default", "newest", "role"]
const SCORES: Array[String] = ["both", "hp", "binary"]
const ON_OFF: Array[String] = ["on", "off"]
const CAP_MIN: int = 1
const CAP_MAX: int = 30
const DISTANCE_MIN: int = 2
const DISTANCE_MAX: int = 6
const METRIC_MODES: Array[String] = ["party", "role", "sweep"]   # metrics default on (commit 1b modes)

var mode: String = "duel"
var level: int = 5
var all_levels: bool = false     # `stats all`
var seed_count: int = 3
var cap: int = 10
var smart_skills: bool = true
var loadout: String = "default"
var spend: String = "class"      # one of SPENDS or "all"
var preset: String = "balanced"  # a PlanPresets id or "both"
var distance: int = 0            # 0 = default deployment
var score: String = "both"
var quick: bool = false
var metrics: bool = false
var band_lo: int = 30            # duel diagnostic band on the row hp average (DECISIONS BAL2 answers; a flag, not a gate)
var band_hi: int = 70
var error: String = ""           # non-empty: print the usage and run nothing

# Parses `args` (OS.get_cmdline_user_args()). False (and `error` set) on an unknown key or value.
func parse(args: PackedStringArray) -> bool:
	var positional: Array[String] = []
	var metrics_set: bool = false
	for raw: String in args:
		var token: String = raw.strip_edges()
		if token.is_empty():
			continue
		if token == "quick":
			quick = true
			continue
		if not token.contains("="):
			positional.append(token)
			continue
		var key: String = token.get_slice("=", 0)
		var value: String = token.substr(key.length() + 1)
		match key:
			"cap":
				if not value.is_valid_int() or int(value) < CAP_MIN or int(value) > CAP_MAX:
					return _fail("cap must be %d..%d (got '%s')" % [CAP_MIN, CAP_MAX, value])
				cap = int(value)
			"smart_skills":
				if not ON_OFF.has(value):
					return _fail("smart_skills must be on or off (got '%s')" % value)
				smart_skills = value == "on"
			"loadout":
				if not LOADOUTS.has(value):
					return _fail("loadout must be one of %s (got '%s')" % [str(LOADOUTS), value])
				if value == "role":
					return _fail("loadout=role needs the role loadouts of BAL2-L commit 1b")
				loadout = value
			"spend":
				if not SPENDS.has(value) and value != "all":
					return _fail("spend must be one of %s or all (got '%s')" % [str(SPENDS), value])
				spend = value
			"preset":
				if not PlanPresets.IDS.has(value) and value != "both":
					return _fail("preset must be one of %s or both (got '%s')" % [str(PlanPresets.IDS), value])
				preset = value
			"distance":
				if value == "default":
					distance = 0
				elif value.is_valid_int() and int(value) >= DISTANCE_MIN and int(value) <= DISTANCE_MAX:
					distance = int(value)
				else:
					return _fail("distance must be default or %d..%d (got '%s')" % [DISTANCE_MIN, DISTANCE_MAX, value])
			"score":
				if not SCORES.has(value):
					return _fail("score must be one of %s (got '%s')" % [str(SCORES), value])
				score = value
			"metrics":
				if not ON_OFF.has(value):
					return _fail("metrics must be on or off (got '%s')" % value)
				metrics = value == "on"
				metrics_set = true
			"quick":
				quick = true
			"band":
				var parts: PackedStringArray = value.split("-")
				if parts.size() != 2 or not parts[0].is_valid_int() or not parts[1].is_valid_int() or int(parts[0]) < 0 or int(parts[1]) > 100 or int(parts[0]) > int(parts[1]):
					return _fail("band must be lo-hi in percent, 0 <= lo <= hi <= 100 (got '%s')" % value)
				band_lo = int(parts[0])
				band_hi = int(parts[1])
			_:
				return _fail("unknown key '%s'" % key)
	if positional.size() > 0:
		mode = positional[0]
	if not MODES.has(mode):
		return _fail("unknown mode '%s'" % mode)
	if positional.size() > 1:
		if positional[1] == "all" and mode == "stats":
			all_levels = true
		elif positional[1].is_valid_int() and int(positional[1]) >= 1 and int(positional[1]) <= StatRules.MAX_LEVEL:
			level = int(positional[1])
		else:
			return _fail("level must be 1..%d%s (got '%s')" % [StatRules.MAX_LEVEL, " or all" if mode == "stats" else "", positional[1]])
	if positional.size() > 2:
		if not positional[2].is_valid_int() or int(positional[2]) < 1:
			return _fail("seeds must be a positive number (got '%s')" % positional[2])
		seed_count = int(positional[2])
	if positional.size() > 3:
		return _fail("too many positional arguments")
	if not metrics_set:
		metrics = METRIC_MODES.has(mode)
	if quick:
		seed_count = 1
	return true

func _fail(text: String) -> bool:
	error = text
	return false

# Seed i = 11 + 12 i (the old sim's seeds).
func seeds() -> Array[int]:
	var out: Array[int] = []
	for i: int in range(seed_count):
		out.append(11 + 12 * i)
	return out

# The spend variants a matchup averages over (`all` = class, cons, primary; `quick` keeps the first).
func spend_variants() -> Array[String]:
	var out: Array[String] = []
	if spend == "all":
		out.append_array(["class", "cons", "primary"])
	else:
		out.append(spend)
	if quick:
		out.resize(1)
	return out

# The presets a matchup averages over (`both` = balanced, aggressive).
func preset_variants() -> Array[String]:
	var out: Array[String] = []
	if preset == "both":
		out.append_array(["balanced", "aggressive"])
	else:
		out.append(preset)
	if quick:
		out.resize(1)
	return out

func show_hp() -> bool:
	return score != "binary"

func show_binary() -> bool:
	return score != "hp"

# One line echoing the effective arguments (every table starts with it).
func header() -> String:
	var lvl: String = "all" if all_levels else str(level)
	return "ARGS mode %s, level %s, seeds %d, cap %d, smart_skills %s, loadout %s, spend %s, preset %s, distance %s, score %s, quick %s, metrics %s, band %d-%d" % [
		mode, lvl, seed_count, cap, "on" if smart_skills else "off", loadout, spend, preset,
		"default" if distance == 0 else str(distance), score, "on" if quick else "off", "on" if metrics else "off", band_lo, band_hi]

static func usage() -> Array[String]:
	return [
		"usage: tools/fc.sh sim <mode> [level] [seeds] [key=value ...]   (seed i = 11 + 12 i; level default 5, seeds 3)",
		"modes: duel [level]       class round robin, hero vs hero (bin/hp matrix, row averages)",
		"       tier [level]       hero vs its class at minion/standard/elite/boss, 1v1 and 1v3",
		"       levels             class at L vs L+2 for L = 1, 3, 5, 7",
		"       group [level]      hero L vs 3 minions at L-2, 3 standard at L-2, 3 minions at L",
		"       stats [level|all]  no fights: HP, armor, EHP, power, range, speed, crit, active families",
		"keys:  cap=1..30 (10)  smart_skills=on|off (on)  loadout=default|newest (default; party side)",
		"       spend=class|cons|primary|mixed|all (class; party side)  preset=<PlanPresets id>|both (balanced; every unit)",
		"       distance=default|2..6 (default)  score=both|hp|binary (both)  quick  metrics=on|off (off; collector in 1b)",
		"       band=lo-hi (30-70): duel row hp averages outside it are flagged LOW/HIGH (diagnostic, not a gate)",
	]
