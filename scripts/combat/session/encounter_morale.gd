class_name EncounterMorale
extends RefCounted

# ==============================================================================
# Tactical RPG - Encounter Morale (MORALE1, spec docs/specs/MORALE1.md)
# Enemy groups lose morale as members fall, take critical hits or lose their leader; a broken group
# surrenders. Fields on catalog and world encounters (ignored while `morale` is false):
#   morale          bool    false = no morale rules at all                          (default false)
#   morale_break    int     0..90: a group is Broken at or below this value         (default 50)
#   morale_leader   String  "" = auto (first boss / elite enemy); else an id in `enemies` (default "")
#   morale_groups   json    [[enemy index, ...], ...]; [] = all enemies one group   (default [])
#   morale_text     String  0..80 chars; "" = the generated break line              (default "")
#
# Static part (schema slice): the shared validator `problems` (both catalogs), the leader rule
# (`leader_ref`) and the viewer line (`summary`).
# ==============================================================================

const START: int = 100
const CRIT_HIT: int = 10
const CRIT_KILL_EXTRA: int = 10
const LEADER_DOWN: int = 40
const FLEE_SHAKEN: float = 0.15
const FLEE_WAVERING: float = 0.30
const FLEE_CAP: float = 0.60

const STEADY: String = "steady"
const SHAKEN: String = "shaken"
const WAVERING: String = "wavering"
const BROKEN: String = "broken"
const FEARLESS: String = "fearless"
const STATES: Array[String] = [STEADY, SHAKEN, WAVERING, BROKEN]

const DEFAULT_BREAK: int = 50
const BREAK_MIN: int = 0
const BREAK_MAX: int = 90
const MAX_TEXT: int = 80
const LEADER_TIERS: Array[String] = ["boss", "elite"]

# ------------------------------------------------------------------------------
# Static helpers (both catalogs; `entry` is normalised)
# ------------------------------------------------------------------------------

static func enabled(entry: Dictionary) -> bool:
	return DictUtil.to_bool(entry.get("morale", false))

# Rule 2: `morale_leader` when set; else the first enemy (in `enemies` order) whose tier is boss or elite;
# else "". `tier_of(ref) -> String` resolves a roster ref to its unit tier.
static func leader_ref(entry: Dictionary, tier_of: Callable) -> String:
	var authored: String = str(entry.get("morale_leader", ""))
	if not authored.is_empty():
		return authored
	for ref: String in _strings(entry.get("enemies", [])):
		if LEADER_TIERS.has(str(tier_of.call(ref))):
			return ref
	return ""

# The starting groups as enemy indices (rule 3): `morale_groups`, or one group of every enemy when [].
static func groups_of(entry: Dictionary) -> Array[Array]:
	var out: Array[Array] = []
	var raw: Array = DictUtil.array_of(entry.get("morale_groups", []))
	if raw.is_empty():
		var all: Array[int] = []
		for i: int in range(_strings(entry.get("enemies", [])).size()):
			all.append(i)
		out.append(all)
		return out
	for g: Variant in raw:
		var members: Array[int] = []
		for x: Variant in DictUtil.array_of(g):
			members.append(DictUtil.to_int(x))
		out.append(members)
	return out

# Rule 16: "breaks at <B> or below[, <n> groups][, leader <name>]"; "" while morale is off.
static func summary(entry: Dictionary, name_of: Callable, tier_of: Callable) -> String:
	if not enabled(entry):
		return ""
	var parts: Array[String] = ["breaks at %d or below" % DictUtil.to_int(entry.get("morale_break", DEFAULT_BREAK), DEFAULT_BREAK)]
	var count: int = groups_of(entry).size()
	if count > 1:
		parts.append("%d groups" % count)
	var leader: String = leader_ref(entry, tier_of)
	if not leader.is_empty():
		parts.append("leader %s" % str(name_of.call(leader)))
	return ", ".join(parts)

# Problems with the morale fields of `entry` (spec 1.1); [] while `morale` is false.
static func problems(entry: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	if not enabled(entry):
		return errors
	var brk: int = DictUtil.to_int(entry.get("morale_break", DEFAULT_BREAK), DEFAULT_BREAK)
	if brk < BREAK_MIN or brk > BREAK_MAX:
		errors.append("Morale break must be %d..%d." % [BREAK_MIN, BREAK_MAX])
	var enemies: Array[String] = _strings(entry.get("enemies", []))
	var leader: String = str(entry.get("morale_leader", ""))
	if not leader.is_empty() and not enemies.has(leader):
		errors.append("Morale leader must be a unit from enemies.")
	var raw_groups: Variant = entry.get("morale_groups", [])
	if not (raw_groups is Array):
		errors.append("Morale groups must be a list of enemy index lists.")
	else:
		var groups: Array = raw_groups
		if not groups.is_empty():
			errors.append_array(_group_problems(groups, enemies.size()))
	if str(entry.get("morale_text", "")).length() > MAX_TEXT:
		errors.append("Morale text holds at most %d characters." % MAX_TEXT)
	return errors

static func _group_problems(groups: Array, enemy_count: int) -> Array[String]:
	var errors: Array[String] = []
	var seen: Dictionary = {}   # enemy index -> times listed
	var bad_shape: bool = false
	var empty: bool = false
	var out_of_range: bool = false
	for g: Variant in groups:
		if not (g is Array):
			bad_shape = true
			continue
		var members: Array = g
		if members.is_empty():
			empty = true
		for x: Variant in members:
			if not _whole(x):
				out_of_range = true
				continue
			var i: int = DictUtil.to_int(x)
			if i < 0 or i >= enemy_count:
				out_of_range = true
				continue
			seen[i] = DictUtil.to_int(seen.get(i, 0)) + 1
	if bad_shape:
		errors.append("Morale groups must be a list of enemy index lists.")
	if empty:
		errors.append("A morale group cannot be empty.")
	if out_of_range:
		errors.append("Morale group index out of range.")
	for i: int in range(enemy_count):
		if DictUtil.to_int(seen.get(i, 0)) != 1:
			errors.append("Every enemy must be in exactly one morale group.")
			break
	return errors

static func _whole(x: Variant) -> bool:
	if x is int:
		return true
	if x is float:
		var f: float = x
		return f == floorf(f)
	return false

static func _strings(raw: Variant) -> Array[String]:
	var out: Array[String] = []
	for x: Variant in DictUtil.array_of(raw):
		out.append(str(x))
	return out
