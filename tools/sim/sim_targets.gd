extends RefCounted

# Balance sim targets (BAL2_SIM.md section 8; DECISIONS "BAL2 role answers (user 2026-10-17)").
# Every target is a closed band [lo, hi] in the unit its table prints (percent, tiles, per round,
# per cast, per fight). Defaults are the decided values; `target=<key>:<lo>-<hi>` overrides one
# (sim_args.gd). Report tables mark a value outside its band with X. Loaded with preload.

const OPEN_HIGH: float = 999.0   # upper bound of "at least lo" bands

const DEFAULTS: Dictionary = {
	# 8.2 party clear % at equal level (mean over the 15 distinct 4-class compositions in `sweep`)
	"clear_mob": [90.0, 100.0],
	"clear_line": [70.0, 90.0],
	"clear_wave": [55.0, 80.0],
	"clear_elite": [50.0, 75.0],
	"clear_boss": [30.0, 50.0],
	# 8.2 sweep limits
	"swap_delta": [-10.0, 10.0],      # points of clear %, per class
	"line_min": [50.0, 100.0],        # every composition's clear % on `line`
	"top5_min": [2.0, 5.0],           # appearances of a class in the top 5 compositions
	# 8.3 duel diagnostic (row hp average, percent)
	"duel": [30.0, 70.0],
	# 8.1 role bands (primary class, equal level)
	"tank_taken_share": [40.0, 65.0],
	"tank_attacks_share": [45.0, 100.0],
	"tank_survival": [60.0, 100.0],
	"tank_ally_hp": [40.0, 100.0],      # BAL5 T1: allies hp_end on won fights
	"tank_ally_hp_all": [15.0, 100.0],  # BAL5 T1: allies hp_end over all fights (floor from BAL3-R)
	"ranged_dist": [2.5, OPEN_HIGH],
	"ranged_share": [70.0, 100.0],
	"ranged_attacks_per_round": [0.0, 1.0],
	"aoe_foes_per_cast": [2.0, OPEN_HIGH],
	"aoe_allies_per_cast": [0.0, 0.3],
	"aoe_swarm_clear": [70.0, 100.0],
	"support_heal_share": [20.0, 100.0],
	"support_saves": [0.3, OPEN_HIGH],
	"support_survivor_gain": [0.5, OPEN_HIGH],
	"striker_down_by_5": [60.0, 100.0],
	"striker_priority_share": [50.0, 100.0],
	"scenario_clear": [50.0, 85.0],
}

# A fresh copy of the defaults (key -> [lo, hi] floats).
static func defaults() -> Dictionary:
	var out: Dictionary = {}
	for key: Variant in DEFAULTS:
		var pair: Array = DEFAULTS[key]
		out[key] = [DictUtil.to_float(pair[0]), DictUtil.to_float(pair[1])]
	return out

static func lo(targets: Dictionary, key: String) -> float:
	return DictUtil.to_float(DictUtil.array_of(targets.get(key, [0.0, OPEN_HIGH]))[0])

static func hi(targets: Dictionary, key: String) -> float:
	return DictUtil.to_float(DictUtil.array_of(targets.get(key, [0.0, OPEN_HIGH]))[1])

static func inside(targets: Dictionary, key: String, value: float) -> bool:
	return value >= lo(targets, key) - 0.0001 and value <= hi(targets, key) + 0.0001

# True when the range [floor_v, ceil_v] overlaps the band of `key` (role bands, BAL2_SIM 8.1).
static func overlaps(targets: Dictionary, key: String, floor_v: float, ceil_v: float) -> bool:
	return ceil_v >= lo(targets, key) - 0.0001 and floor_v <= hi(targets, key) + 0.0001

# "" inside the band, " X" outside (the report mark).
static func mark(targets: Dictionary, key: String, value: float) -> String:
	return "" if inside(targets, key, value) else " X"

# Band text for a header: "70-90", ">=90" (upper bound open or 100 %), "<=1.0".
static func band_text(targets: Dictionary, key: String) -> String:
	var l: float = lo(targets, key)
	var h: float = hi(targets, key)
	if l < 0.0 and is_equal_approx(l, -h):
		return "+-%s" % _num(h)
	if h >= OPEN_HIGH or (h == 100.0 and l > 0.0):
		return ">=%s" % _num(l)
	if l <= 0.0:
		return "<=%s" % _num(h)
	return "%s-%s" % [_num(l), _num(h)]

static func _num(v: float) -> String:
	return str(roundi(v)) if is_equal_approx(v, roundf(v)) else "%.1f" % v

# Parses "<key>:<lo>-<hi>" into targets (false = unknown key or bad numbers).
static func apply_override(targets: Dictionary, text: String) -> bool:
	var key: String = text.get_slice(":", 0)
	var band: String = text.substr(key.length() + 1)
	if not DEFAULTS.has(key) or not text.contains(":"):
		return false
	var cut: int = band.find("-", 1)   # a leading minus belongs to lo (swap_delta)
	if cut < 0:
		return false
	var a: String = band.substr(0, cut)
	var b: String = band.substr(cut + 1)
	if not a.is_valid_float() or not b.is_valid_float() or float(a) > float(b):
		return false
	targets[key] = [float(a), float(b)]
	return true
