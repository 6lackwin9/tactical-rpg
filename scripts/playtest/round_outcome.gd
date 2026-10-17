class_name RoundOutcome
extends RefCounted

# ==============================================================================
# Tactical RPG - Round Outcome (PLAYTEST1 section E)
# {units: [{ref, hp, ap, pos, stance, statuses, withdrawn}], over, log_count}, units in roster order
# (allies then enemies); statuses = sorted "<id>:<rounds left>". Compared field by field, exact equality.
# ==============================================================================

const UNIT_FIELDS: Array[String] = ["hp", "ap", "pos", "stance", "statuses", "withdrawn"]

static func capture(session: CombatSession) -> Dictionary:
	var units: Array = []
	for c: Combatant in session.allies + session.enemies:
		units.append(unit_state(session, c))
	return {"units": units, "over": session.combat_over, "log_count": session.log_entries.size()}

static func unit_state(session: CombatSession, c: Combatant) -> Dictionary:
	var pos: Vector2i = session.grid.get_position(c) if session.grid != null else Vector2i(-1, -1)
	var statuses: Array[String] = []
	for raw: Variant in c.statuses:
		var id: String = str(raw)
		statuses.append("%s:%d" % [id, DictUtil.int_of(c.status_rounds_left, id, 1)])
	statuses.sort()
	return {"ref": PlanCodec.unit_ref(session, c), "hp": c.hp, "ap": c.ap, "pos": [pos.x, pos.y], "stance": CombatTypes.stance_to_string(c.current_stance),
		"statuses": statuses, "withdrawn": c.is_withdrawn}

# The first difference: {match: true} or {match: false, ref, field, expected, actual}.
static func compare(expected: Dictionary, actual: Dictionary) -> Dictionary:
	var diffs: Array[Dictionary] = differences(expected, actual, true)
	if diffs.is_empty():
		return {"match": true}
	var first: Dictionary = diffs[0]
	first["match"] = false
	return first

# Every difference (or only the first when `first_only`): [{ref, field, expected, actual}]; ref null = battle-level field.
static func differences(expected: Dictionary, actual: Dictionary, first_only: bool = false) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var exp_units: Array = DictUtil.array_of(expected.get("units", []))
	var act_units: Array = DictUtil.array_of(actual.get("units", []))
	if exp_units.size() != act_units.size():
		out.append({"ref": null, "field": "units", "expected": exp_units.size(), "actual": act_units.size()})
		return out
	for i: int in range(exp_units.size()):
		var eu: Dictionary = DictUtil.dict_of(exp_units[i])
		var au: Dictionary = DictUtil.dict_of(act_units[i])
		for field: String in UNIT_FIELDS:
			if not JsonUtil.deep_equal(eu.get(field, null), au.get(field, null)):
				out.append({"ref": eu.get("ref", au.get("ref", null)), "field": field, "expected": eu.get(field, null), "actual": au.get(field, null)})
				if first_only:
					return out
	for field: String in ["over", "log_count"]:
		if not JsonUtil.deep_equal(expected.get(field, null), actual.get(field, null)):
			out.append({"ref": null, "field": field, "expected": expected.get(field, null), "actual": actual.get(field, null)})
			if first_only:
				return out
	return out

static func diff_text(d: Dictionary) -> String:
	var ref: Variant = d.get("ref", null)
	var who: String = PlanCodec.ref_text(ref) if ref != null else "battle"
	return "%s %s %s -> %s" % [who, str(d.get("field", "")), JSON.stringify(d.get("expected", null)), JSON.stringify(d.get("actual", null))]
