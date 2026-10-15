class_name StatRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Stat Rules (the enforced rules for classes and units)
#
#   Budget    Every class starts with the SAME stat total (CLASS_BUDGET) spread
#             over the six stats (floor / ceiling per stat), so classes differ only
#             in distribution. On top of it a unit has SPEND points:
#                 spend pool(level) = SPEND_BASE + (level - 1) * FREE_PER_LEVEL
#   Growth    Every class gains the same total per level: GROWTH_TOTAL points in
#             its own weights (max GROWTH_STAT_MAX per stat), applied automatically.
#   Variance  Units of one class differ by how they spend their pool (authored for
#             NPCs / enemies, chosen by the player for party members). An empty
#             spend means "follow the class weights" (auto_spend).
#   Tiers     Heroes and enemies share the class curve; the tier scales HP and
#             attack power (a goblin brawler is a MINION Brawler).
#   Skills    A class path step {level, skill} says when a family is owned; tiers open at fixed
#             level steps after it (SK2). AP is only the per-round budget.
#
# validate_class / validate_spend return problem strings (empty = valid): the
# catalog refuses classes and units that break these rules.
# ==============================================================================

const STATS: Array[String] = ["STR", "DEX", "INT", "CONS", "PERC", "KNOW"]
const CLASS_BUDGET: int = 62          # a class's level-1 stat total
const STAT_FLOOR: int = 6
const STAT_CEILING: int = 18          # class base stat maximum at level 1
const GROWTH_TOTAL: int = 5           # class-weighted points gained per level
const GROWTH_STAT_MAX: int = 3
const SPEND_BASE: int = 4             # free points at level 1
const FREE_PER_LEVEL: int = 1         # free points gained per level above 1
const MAX_LEVEL: int = 10
const MAX_CLASS_SLOTS: int = 5        # active class skill families in combat (basics counted separately)
const MAX_EXTRA_SLOTS: int = 3        # active extra-slot skill families
const MAX_TIER: int = 5               # tiers of a basic skill family
const MAX_CLASS_TIER: int = 3         # tiers of a class skill family
# Skill unlocks (SK2): a class path step says at which unit level a family is owned; its tiers open at
# fixed steps after that (offset 0 / 2 / 4 for tiers 1 / 2 / 3 unless a row sets unlock_offset).
const TIER_UNLOCK_STEP: int = 2
const ADVANCED_MIN_LEVEL: int = 5     # an advanced family may not sit on a class path below this level
const MAX_UNLOCK_REDUCTION: int = 2   # passives / KNOW lower a family's unlock by at most this many levels
const ADVANCED_UNLOCK_FLOOR: int = 3  # an advanced family's effective unlock never drops below this
const KNOW_UNLOCK_BASE: int = 12
const KNOW_UNLOCK_STEP: int = 4
const MAX_UNLOCK_OFFSET: int = 8      # largest explicit skill unlock_offset

# tier -> {hp, power}: multipliers on the class-derived HP and attack power.
const TIERS: Dictionary = {
	"hero": {"hp": 1.0, "power": 1.0, "label": "Hero"},
	"minion": {"hp": 0.4, "power": 0.6, "label": "Minion"},
	"standard": {"hp": 0.7, "power": 0.8, "label": "Standard"},
	"elite": {"hp": 1.0, "power": 1.0, "label": "Elite"},
	"boss": {"hp": 1.7, "power": 1.2, "label": "Boss"},
}

static func tier_names() -> Array[String]:
	var out: Array[String] = []
	for key: Variant in TIERS:
		out.append(str(key))
	return out

static func tier_hp(tier: String) -> float:
	return DictUtil.to_float(DictUtil.dict_of(TIERS.get(tier, TIERS["hero"])).get("hp", 1.0), 1.0)

static func tier_power(tier: String) -> float:
	return DictUtil.to_float(DictUtil.dict_of(TIERS.get(tier, TIERS["hero"])).get("power", 1.0), 1.0)

# Free points a unit of `level` has to spend.
static func spend_pool(level: int) -> int:
	return SPEND_BASE + (clampi(level, 1, MAX_LEVEL) - 1) * FREE_PER_LEVEL

# Unlock level of a family that is NOT on the unit's class path (extras, learned skills): the old
# "skill level <= ceil(unit level / 2)" rule inverted, i.e. 2 x AP level - 1. On-path families use their path level.
static func default_unlock(ap_level: int) -> int:
	return maxi(1, 2 * ap_level - 1)

# Levels a KNOW score lowers an ADVANCED family's unlock by: (KNOW - base) / step, 0..MAX_UNLOCK_REDUCTION.
static func know_unlock_reduction(know: int) -> int:
	return clampi(floori(float(know - KNOW_UNLOCK_BASE) / float(KNOW_UNLOCK_STEP)), 0, MAX_UNLOCK_REDUCTION)

static func total_of(stats: Dictionary) -> int:
	var sum: int = 0
	for key: Variant in stats:
		sum += DictUtil.to_int(stats[key])
	return sum

# `points` distributed over the stats in proportion to the class growth weights
# (largest weights first, deterministic): the default spend of a unit.
static func auto_spend(growth: Dictionary, points: int) -> Dictionary:
	var spend: Dictionary = {}
	if points <= 0:
		return spend
	var order: Array[String] = []
	for s: String in STATS:
		if DictUtil.int_of(growth, s) > 0:
			order.append(s)
	if order.is_empty():
		order.assign(STATS)
	order.sort_custom(func(a: String, b: String) -> bool:
		var wa: int = DictUtil.int_of(growth, a)
		var wb: int = DictUtil.int_of(growth, b)
		return wa > wb or (wa == wb and STATS.find(a) < STATS.find(b))
	)
	var weights: Array[int] = []
	var weight_sum: int = 0
	for s: String in order:
		var w: int = maxi(1, DictUtil.int_of(growth, s))
		weights.append(w)
		weight_sum += w
	var given: int = 0
	for i: int in range(order.size()):
		var share: int = floori(float(points * weights[i]) / float(weight_sum))
		spend[order[i]] = share
		given += share
	var i: int = 0
	while given < points:   # the remainder goes to the heaviest stats first
		spend[order[i % order.size()]] = DictUtil.int_of(spend, order[i % order.size()]) + 1
		given += 1
		i += 1
	var clean: Dictionary = {}
	for s: String in STATS:
		if DictUtil.int_of(spend, s) > 0:
			clean[s] = DictUtil.int_of(spend, s)
	return clean

# ------------------------------------------------------------------------------
# Validation
# ------------------------------------------------------------------------------

static func validate_class(base: Dictionary, growth: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var total: int = 0
	for s: String in STATS:
		var v: int = DictUtil.int_of(base, s, STAT_FLOOR)
		total += v
		if v < STAT_FLOOR or v > STAT_CEILING:
			errors.append("Base %s %d must be between %d and %d." % [s, v, STAT_FLOOR, STAT_CEILING])
	if total != CLASS_BUDGET:
		errors.append("Base stats total %d, every class must total %d (every stat set)." % [total, CLASS_BUDGET])
	var growth_total: int = total_of(growth)
	if growth_total != GROWTH_TOTAL:
		errors.append("Growth totals %d, every class gains exactly %d per level." % [growth_total, GROWTH_TOTAL])
	for s: String in STATS:
		if DictUtil.int_of(growth, s) > GROWTH_STAT_MAX or DictUtil.int_of(growth, s) < 0:
			errors.append("Growth %s must be 0-%d per level." % [s, GROWTH_STAT_MAX])
	return errors

# `spend` is empty (follow the class weights) or fits the pool of `level` exactly.
static func validate_spend(spend: Dictionary, level: int) -> Array[String]:
	var errors: Array[String] = []
	if spend.is_empty():
		return errors
	var pool: int = spend_pool(level)
	for key: Variant in spend:
		if not STATS.has(str(key)):
			errors.append("Spend: unknown stat '%s'." % str(key))
		elif DictUtil.to_int(spend[key]) < 0:
			errors.append("Spend %s can't be negative." % str(key))
	var total: int = total_of(spend)
	if total != pool:
		errors.append("Spend totals %d, a level %d unit spends exactly %d (leave it empty to follow the class)." % [total, level, pool])
	return errors
