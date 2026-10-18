class_name BattleReport
extends RefCounted

# ==============================================================================
# Tactical RPG - Battle report (BREPORT1-L 1.4, rules 4-7): the per-hero table and the best-moment line of the results
# popup, built once from a finished BattleMetrics. Pure and static: build() reads the collector's records and details
# and the session's party side (heroes in roster order, then guests); problems() validates the dict shape.
# ==============================================================================

const VERSION: int = 1
const COLUMNS: Array[String] = ["name", "dealt", "taken", "healed", "effects", "combos"]   # table order (spec section "What the report shows")
const ALWAYS: Array[String] = ["name", "dealt", "taken"]                                     # rule 5: the others only when some row is non-zero
const NUMBERS: Array[String] = ["dealt", "dealt_dot", "ff_dealt", "taken", "taken_dot", "healed", "regen", "effects", "combos", "kills"]
const STATES: Array[String] = ["standing", "down", "left"]
const COMBO_LABELS: Dictionary = {"exploit": "Exploit", "break": "Stance break", "flow": "Flow"}

# The report of the fight `metrics` followed in `session` (rows from the party side only; rule 7: units that never
# deployed are not in the battle).
static func build(metrics: BattleMetrics, session: CombatSession, outcome: String) -> Dictionary:
	var rows: Array[Dictionary] = []
	var details: Array[Dictionary] = []
	for c: Combatant in session.allies:
		var rec: Dictionary = DictUtil.dict_of(metrics.units.get(c.name, {}))
		var det: Dictionary = DictUtil.dict_of(metrics.detail.get(c.name, {}))
		var guest: bool = session.encounter.is_guest(c)
		var state: String = "left" if c.is_withdrawn else ("standing" if c.is_alive() else "down")
		var status_counts: Dictionary = DictUtil.dict_of(det.get("status_counts", {})).duplicate(true)
		var combo_counts: Dictionary = DictUtil.dict_of(det.get("combo_counts", {})).duplicate(true)
		rows.append({"ref": session.encounter.hero_key(c), "name": c.name, "guest": guest, "state": state,
			"dealt": DictUtil.int_of(rec, "dmg_dealt") + DictUtil.int_of(rec, "dot_dealt"), "dealt_dot": DictUtil.int_of(rec, "dot_dealt"),
			"ff_dealt": DictUtil.int_of(rec, "ff_dealt"),
			"taken": DictUtil.int_of(rec, "dmg_taken") + DictUtil.int_of(rec, "dot_taken") + DictUtil.int_of(rec, "ff_taken"),
			"taken_dot": DictUtil.int_of(rec, "dot_taken"), "healed": DictUtil.int_of(rec, "heal_done"), "regen": DictUtil.int_of(det, "regen_done"),
			"effects": DictUtil.int_of(rec, "buffs_applied") + DictUtil.int_of(rec, "debuffs_applied"), "combos": DictUtil.int_of(rec, "combos"),
			"kills": DictUtil.int_of(rec, "kills"), "status_counts": status_counts, "combo_counts": combo_counts})
		details.append(det)
	return {"version": VERSION, "outcome": outcome, "rounds": session.current_round, "rows": rows, "columns": columns_for(rows),
		"best": best_moment(rows, details)}

# Rule 5: name, dealt, taken always; healed, effects, combos only when some row has a non-zero value.
static func columns_for(rows: Array[Dictionary]) -> Array[String]:
	var out: Array[String] = []
	for id: String in COLUMNS:
		if ALWAYS.has(id):
			out.append(id)
			continue
		for r: Dictionary in rows:
			if DictUtil.int_of(r, id) != 0:
				out.append(id)
				break
	return out

# Rule 6: a save, else a combo finish, else the biggest hit, else "". `details[i]` belongs to `rows[i]`. Within a rule
# the strongest candidate wins (save: the lowest HP before the heal; hit: the largest amount), then the earliest entry.
static func best_moment(rows: Array[Dictionary], details: Array[Dictionary]) -> String:
	var save: Dictionary = _pick(rows, details, "best_save", func(m: Dictionary) -> int: return -DictUtil.int_of(m, "hp"))
	if not save.is_empty():
		var ally: String = str(save["ally"])
		return "%s healed %s at %d HP, and %s made it through." % [str(save["who"]), ally, DictUtil.int_of(save, "hp"), ally]
	var finish: Dictionary = _pick(rows, details, "best_finish", func(_m: Dictionary) -> int: return 0)
	if not finish.is_empty():
		var how: String = "broke the guard" if str(finish.get("combo", "")) == "break" else "exploited %s" % StatusEffects.display_name(str(finish.get("status", "")))
		return "%s finished %s with %s (%s)." % [str(finish["who"]), str(finish["target"]), str(finish["skill"]), how]
	var hit: Dictionary = _pick(rows, details, "best_hit", func(m: Dictionary) -> int: return DictUtil.int_of(m, "amount"))
	if not hit.is_empty() and DictUtil.int_of(hit, "amount") >= 1:
		return "%s's %s hit %s for %d." % [str(hit["who"]), str(hit["skill"]), str(hit["target"]), DictUtil.int_of(hit, "amount")]
	return ""

# The best non-empty `key` moment over the rows (`score` higher wins, then the lower `order`), with "who" = the row name.
static func _pick(rows: Array[Dictionary], details: Array[Dictionary], key: String, score: Callable) -> Dictionary:
	var best: Dictionary = {}
	var best_score: int = 0
	for i: int in range(mini(rows.size(), details.size())):
		var m: Dictionary = DictUtil.dict_of(details[i].get(key, {}))
		if m.is_empty():
			continue
		var s: int = DictUtil.to_int(score.call(m))
		if best.is_empty() or s > best_score or (s == best_score and DictUtil.int_of(m, "order") < DictUtil.int_of(best, "order")):
			best = m.duplicate()
			best["who"] = str(rows[i]["name"])
			best_score = s
	return best

# ------------------------------------------------------------------------------
# Tips (the model shows them on hover)
# ------------------------------------------------------------------------------

static func dealt_tip(row: Dictionary) -> String:
	var dot: int = DictUtil.int_of(row, "dealt_dot")
	var tip: String = "%d from hits, %d from bleed, poison and burn." % [DictUtil.int_of(row, "dealt") - dot, dot]
	if DictUtil.int_of(row, "ff_dealt") > 0:
		tip += " Hit allies for %d." % DictUtil.int_of(row, "ff_dealt")
	return tip

static func taken_tip(row: Dictionary) -> String:
	var dot: int = DictUtil.int_of(row, "taken_dot")
	return "%d from hits, %d from effects." % [DictUtil.int_of(row, "taken") - dot, dot]

static func healed_tip(row: Dictionary) -> String:
	var regen: int = DictUtil.int_of(row, "regen")
	return "Includes %d from regeneration." % regen if regen > 0 else ""

# "Bleed x2, Slow x1, Haste on allies x1": every status put on, buffs marked "on allies".
static func effects_tip(row: Dictionary) -> String:
	var parts: PackedStringArray = PackedStringArray()
	var counts: Dictionary = DictUtil.dict_of(row.get("status_counts", {}))
	var buffs: Array[String] = StatusEffects.buffs()
	for s: Variant in counts:
		var n: int = DictUtil.int_of(counts, s)
		if n > 0:
			parts.append("%s%s x%d" % [StatusEffects.display_name(str(s)), " on allies" if buffs.has(str(s)) else "", n])
	return ", ".join(parts)

# "Exploit x1, Stance break x1, Flow x2" (kinds with a count only).
static func combos_tip(row: Dictionary) -> String:
	var parts: PackedStringArray = PackedStringArray()
	var counts: Dictionary = DictUtil.dict_of(row.get("combo_counts", {}))
	for kind: String in BattleMetrics.COMBO_KINDS:
		var n: int = DictUtil.int_of(counts, kind)
		if n > 0:
			parts.append("%s x%d" % [str(COMBO_LABELS[kind]), n])
	return ", ".join(parts)

# ------------------------------------------------------------------------------
# Validation (1.4)
# ------------------------------------------------------------------------------

# Missing keys, wrong types, negative numbers, unknown column ids or states; [] = a usable report.
static func problems(report: Dictionary) -> Array[String]:
	var out: Array[String] = []
	var top: Dictionary = {"version": TYPE_INT, "outcome": TYPE_STRING, "rounds": TYPE_INT, "rows": TYPE_ARRAY, "columns": TYPE_ARRAY, "best": TYPE_STRING}
	for key: Variant in top:
		if not report.has(key):
			out.append("missing key '%s'" % str(key))
		elif typeof(report[key]) != DictUtil.to_int(top[key]):
			out.append("'%s' has the wrong type" % str(key))
	if not out.is_empty():
		return out
	if DictUtil.to_int(report["rounds"]) < 0:
		out.append("'rounds' is negative")
	for raw: Variant in DictUtil.array_of(report["columns"]):
		if not (raw is String) or not COLUMNS.has(str(raw)):
			out.append("unknown column '%s'" % str(raw))
	var row_types: Dictionary = {"ref": TYPE_STRING, "name": TYPE_STRING, "guest": TYPE_BOOL, "state": TYPE_STRING, "status_counts": TYPE_DICTIONARY, "combo_counts": TYPE_DICTIONARY}
	for id: String in NUMBERS:
		row_types[id] = TYPE_INT
	var i: int = 0
	for raw: Variant in DictUtil.array_of(report["rows"]):
		if not (raw is Dictionary):
			out.append("row %d is not a dictionary" % i)
			i += 1
			continue
		var row: Dictionary = raw
		for key: Variant in row_types:
			if not row.has(key):
				out.append("row %d: missing key '%s'" % [i, str(key)])
			elif typeof(row[key]) != DictUtil.to_int(row_types[key]):
				out.append("row %d: '%s' has the wrong type" % [i, str(key)])
			elif NUMBERS.has(str(key)) and DictUtil.to_int(row[key]) < 0:
				out.append("row %d: '%s' is negative" % [i, str(key)])
		if row.has("state") and not STATES.has(str(row["state"])):
			out.append("row %d: unknown state '%s'" % [i, str(row["state"])])
		i += 1
	return out
