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
# Static part: the shared validator `problems` (both catalogs), the leader rule (`leader_ref`), the
# viewer line (`summary`), the state table (`state_for`), the flee bonus (`flee_for`) and the crit
# reader of a resolved command (`crit_events`). Runtime part: groups, losses, breaks and surrenders.
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

# ------------------------------------------------------------------------------
# Runtime (one instance per battle, built by CatalogEncounter.build(); spec 1.2 and section 2)
# Unit keys are "<roster ref>#<n>" (the n-th unit spawned from that ref, CatalogEncounter.unit(ref, n)).
# ------------------------------------------------------------------------------

const LINE_LEADER: String = "%s falls. The rest throw down their arms."
const LINE_ONE_GROUP: String = "The rest throw down their arms."
const LINE_GROUPS: String = "%s throw down their arms."
const SUBTYPE: String = "Morale"
const OUTCOME: String = "MORALE"

var on: bool = false
var brk: int = DEFAULT_BREAK
var text: String = ""
# {members: Array[String], size: int, value: int, broken: bool, at: [round, block], cause: String,
#  announced: bool, line: String, said: [round, block] ([] = not logged yet), source: String}
var groups: Array[Dictionary] = []
var leader: String = ""             # unit key or ""
var leader_down: bool = false
var surrendered: Array[String] = [] # unit keys
var down: Array[String] = []        # members already counted down (defeated or withdrawn)
var pending: Array[Dictionary] = [] # this block's crit events {key, killed}, in order

# A runtime for a normalised entry. `starting` are the keys of the starting enemies in `enemies` order;
# `leader_key` is the resolved leader ("" = none).
static func create(entry: Dictionary, starting: Array[String], leader_key: String) -> EncounterMorale:
	var m := EncounterMorale.new()
	m.on = enabled(entry)
	if not m.on:
		return m
	m.brk = clampi(DictUtil.to_int(entry.get("morale_break", DEFAULT_BREAK), DEFAULT_BREAK), BREAK_MIN, BREAK_MAX)
	m.text = str(entry.get("morale_text", "")).strip_edges()
	m.leader = leader_key
	for idx: Array in groups_of(entry):
		var members: Array[String] = []
		for i: Variant in idx:
			var n: int = DictUtil.to_int(i)
			if n >= 0 and n < starting.size() and not starting[n].is_empty():
				members.append(starting[n])
		m.form_group(members, members.size(), "start")
	return m

# Rule 3: a new group at START (late arrivals are fresh). `size` = members that count when it forms.
func form_group(members: Array[String], size: int, source: String) -> void:
	var keys: Array[String] = []
	for key: String in members:
		if not key.is_empty() and not keys.has(key):
			keys.append(key)
	if not on or keys.is_empty():
		return
	groups.append({"members": keys, "size": maxi(1, size), "value": START, "broken": false, "at": [],
		"cause": "", "announced": false, "line": "", "said": [], "source": source})

func has_group(source: String) -> bool:
	for g: Dictionary in groups:
		if str(g["source"]) == source:
			return true
	return false

# Index of the group holding `key`, -1 when none.
func group_of(key: String) -> int:
	for i: int in range(groups.size()):
		var members: Array = groups[i]["members"]
		if members.has(key):
			return i
	return -1

# Rule 4: the state of a value at break line `b`.
static func state_for(value: int, b: int) -> String:
	var span: int = 100 - b
	if value <= b:
		return BROKEN
	if value <= b + span / 3:
		return WAVERING
	if value <= b + (2 * span) / 3:
		return SHAKEN
	return STEADY

func group_state(i: int) -> String:
	if i < 0 or i >= groups.size():
		return ""
	if DictUtil.to_bool(groups[i]["broken"]):
		return BROKEN
	return state_for(DictUtil.to_int(groups[i]["value"]), brk)

# Rule 8: the effective flee ratio of a non-fearless unit in a group of state `state`.
static func flee_for(base: float, state: String) -> float:
	var bonus: float = 0.0
	if state == SHAKEN:
		bonus = FLEE_SHAKEN
	elif state == WAVERING:
		bonus = FLEE_WAVERING
	if bonus <= 0.0 or base <= 0.0:
		return base
	return maxf(base, minf(FLEE_CAP, base + bonus))

# Rule 6: a crit taken by member `key` (killed = the crit defeated it).
func record_crit(key: String, killed: bool) -> void:
	if on and group_of(key) >= 0:
		pending.append({"key": key, "killed": killed})

# Rules 6 and 7 at a block end. `unit_of(key) -> Combatant`, `fearless(c) -> bool`, `reserved(c) -> bool`.
# Returns the units that surrender now (rule 9); the caller takes them off the field.
func block_end(round_num: int, block_num: int, unit_of: Callable, fearless: Callable, reserved: Callable) -> Array[Combatant]:
	var out: Array[Combatant] = []
	if not on:
		return out
	var deltas: Array[int] = []
	for _g: Dictionary in groups:
		deltas.append(0)
	var crit_killed: Array[String] = []
	for ev: Dictionary in pending:
		var gi: int = group_of(str(ev["key"]))
		if gi < 0:
			continue
		if DictUtil.to_bool(ev["killed"]):
			crit_killed.append(str(ev["key"]))
		else:
			deltas[gi] -= CRIT_HIT
	pending.clear()
	var leader_fell: bool = false
	for gi: int in range(groups.size()):
		var g: Dictionary = groups[gi]
		var size: int = maxi(1, DictUtil.to_int(g["size"]))
		for raw: Variant in DictUtil.array_of(g["members"]):
			var key: String = str(raw)
			if down.has(key) or surrendered.has(key):
				continue
			var c: Combatant = unit_of.call(key)
			if c == null or c.is_active() or DictUtil.to_bool(reserved.call(c)):
				continue
			down.append(key)
			deltas[gi] -= (100 + size - 1) / size + (CRIT_KILL_EXTRA if crit_killed.has(key) else 0)
			if key == leader and not leader_down:
				leader_fell = true
	if leader_fell:
		leader_down = true
		for gi: int in range(groups.size()):
			deltas[gi] -= LEADER_DOWN
	var first_break: bool = true
	for g: Dictionary in groups:
		if DictUtil.to_bool(g["broken"]):
			first_break = false
	var leader_group: int = group_of(leader) if not leader.is_empty() else -1
	for gi: int in range(groups.size()):
		var g: Dictionary = groups[gi]
		if DictUtil.to_bool(g["broken"]):
			continue   # frozen (no rally in v1)
		g["value"] = clampi(DictUtil.to_int(g["value"]) + deltas[gi], 0, START)
		if DictUtil.to_int(g["value"]) > brk:
			continue
		g["broken"] = true
		g["at"] = [round_num, block_num]
		g["cause"] = "leader" if leader_fell else "losses"
		var names: Array[String] = []
		for raw: Variant in DictUtil.array_of(g["members"]):
			var key: String = str(raw)
			var c: Combatant = unit_of.call(key)
			if c == null or not c.is_active() or DictUtil.to_bool(reserved.call(c)) or DictUtil.to_bool(fearless.call(c)):
				continue
			surrendered.append(key)
			names.append(c.name)
			out.append(c)
		if names.is_empty():
			g["announced"] = true   # rule 12: not logged when nobody surrendered
		else:
			g["line"] = _line(first_break, leader_fell and gi == leader_group, unit_of, names)
		first_break = false
	return out

func _line(first: bool, leader_cause: bool, unit_of: Callable, names: Array[String]) -> String:
	if first and not text.is_empty():
		return text
	if leader_cause:
		var c: Combatant = unit_of.call(leader)
		return LINE_LEADER % (c.name if c != null else leader)
	if groups.size() == 1:
		return LINE_ONE_GROUP
	return LINE_GROUPS % " and ".join(names)

# Break lines not logged yet (at most `limit`; 0 = all), in group order; marks them said at [round, block].
func take_lines(round_num: int, block_num: int, limit: int = 0) -> Array[String]:
	var out: Array[String] = []
	for g: Dictionary in groups:
		if DictUtil.to_bool(g["announced"]) or str(g["line"]).is_empty():
			continue
		if limit > 0 and out.size() >= limit:
			break
		g["announced"] = true
		g["said"] = [round_num, block_num]
		out.append(str(g["line"]))
	return out

# Lines logged at [round, block] before (a reset re-logs round-start lines).
func lines_said_at(round_num: int, block_num: int) -> Array[String]:
	var out: Array[String] = []
	for g: Dictionary in groups:
		var said: Array = DictUtil.array_of(g["said"])
		if said.size() == 2 and DictUtil.to_int(said[0]) == round_num and DictUtil.to_int(said[1]) == block_num:
			out.append(str(g["line"]))
	return out

# Kept in every BlockSnapshot under objective_state()["morale"] (deep copies).
func state() -> Dictionary:
	var copy: Array = []
	for g: Dictionary in groups:
		copy.append(g.duplicate(true))
	return {"groups": copy, "leader": leader, "leader_down": leader_down, "surrendered": surrendered.duplicate(),
		"down": down.duplicate(), "pending": pending.duplicate(true)}

func restore(saved: Dictionary) -> void:
	if not on:
		return
	groups.clear()
	for raw: Variant in DictUtil.array_of(saved.get("groups", [])):
		if raw is Dictionary:
			var g: Dictionary = raw
			groups.append(g.duplicate(true))
	leader = str(saved.get("leader", leader))
	leader_down = DictUtil.to_bool(saved.get("leader_down", false))
	surrendered.assign(DictUtil.array_of(saved.get("surrendered", [])).duplicate())
	down.assign(DictUtil.array_of(saved.get("down", [])).duplicate())
	pending.clear()

# Rule 6 hook: the crits one resolved command dealt, [{unit: Combatant, killed: bool}] in hit order (attacks, area
# attacks, then the ripostes it provoked). Status ticks never crit.
static func crit_events(cmd: BattleCommand) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if cmd == null:
		return out
	if cmd is AttackCommand:
		var attack: AttackCommand = cmd
		if attack.last_crit and attack.last_target != null:
			out.append({"unit": attack.last_target, "killed": not attack.last_target.is_alive()})
	elif cmd is AreaAttackCommand:
		var area: AreaAttackCommand = cmd
		for hit: Dictionary in area.crit_hits:
			out.append(hit)
	for reaction: Dictionary in cmd.reactions:
		var raw: Variant = reaction.get("attacker", null)
		if DictUtil.to_bool(reaction.get("crit", false)) and raw is Combatant:
			var hit_unit: Combatant = raw
			out.append({"unit": hit_unit, "killed": DictUtil.to_bool(reaction.get("defeated", false))})
	return out
