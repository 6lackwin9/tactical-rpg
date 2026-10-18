class_name PlaytestRecord
extends RefCounted

# ==============================================================================
# Tactical RPG - Playtest Record (PLAYTEST1 schema: defaults, normalisation, validation)
# A recording is plain JSON-shaped data (docs/specs/PLAYTEST1_L.md, sections A-I):
#   recording {format, version, id, origin, created, engine, source_rev, scenario, parent,
#              content, battles, trail, trail_dropped, asset_templates, notes}
#   battle    {index, encounter, seed, seed_source, roster, rounds, end}
#   round     {round, rng_state, plans, attempts, discarded, plan_ms, edits, outcome, source}
#             source: who confirmed the plans, "player" or "auto" (AUTO1 battle autoplay); older files default to player
#   note      {n, t, origin, scene, battle, round, block, phase, mouse, viewport, ctl, tags, text, image}
# Unknown keys are kept and ignored; missing keys take the defaults below. RNG states are decimal strings.
# ==============================================================================

const FORMAT: String = "playtest"
const VERSION: int = 1
const SCENARIO_FORMAT: String = "playtest_scenario"
const VARIANT_FORMAT: String = "playtest_variant"

const ORIGINS: Array[String] = ["user", "agent"]
const SEED_SOURCES: Array[String] = ["encounter", "scenario", "branch", "tool"]
const END_RESULTS: Array[String] = ["won", "lost", "fled", "timeout", "abandoned"]
const PLAN_SOURCES: Array[String] = ["player", "auto"]
const NOTE_TAGS: Array[String] = ["mechanic", "ui", "ux", "bug"]
const NOTE_PHASES: Array[String] = ["planning", "resolving", "replay", ""]
const TRAIL_MAX: int = 20000
const NOTE_TEXT_MAX: int = 2000

# ------------------------------------------------------------------------------
# Defaults
# ------------------------------------------------------------------------------

static func recording_defaults() -> Dictionary:
	return {"format": FORMAT, "version": VERSION, "id": "", "origin": "user", "created": "", "engine": "", "source_rev": "",
		"scenario": "", "parent": null, "content": {}, "battles": [], "trail": [], "trail_dropped": 0, "asset_templates": 0, "notes": []}

static func battle_defaults() -> Dictionary:
	return {"index": 0, "encounter": "", "seed": 0, "seed_source": "encounter", "roster": [], "rounds": [], "end": end_entry("abandoned", 0)}

static func round_defaults() -> Dictionary:
	return {"round": 1, "rng_state": "0", "plans": [], "attempts": 1, "discarded": [], "plan_ms": 0, "edits": 0, "outcome": {}, "source": "player"}

static func note_defaults() -> Dictionary:
	return {"n": 1, "t": 0, "origin": "user", "scene": "", "battle": -1, "round": 0, "block": 0, "phase": "", "mouse": [0, 0],
		"viewport": [0, 0], "ctl": {}, "tags": [], "text": "", "image": ""}

static func end_entry(result: String, rounds: int) -> Dictionary:
	return {"result": result, "rounds": rounds}

# A fresh recording (the content fingerprint is filled by the caller).
static func create(id: String, origin: String = "user") -> Dictionary:
	var data: Dictionary = recording_defaults()
	data["id"] = id
	data["origin"] = origin
	data["created"] = Time.get_datetime_string_from_system(false, false)
	data["engine"] = str(Engine.get_version_info().get("string", ""))
	data["source_rev"] = PlaytestStore.source_rev()
	return data

# ------------------------------------------------------------------------------
# Normalisation: missing keys take defaults (in place for nested battles / rounds / notes); unknown keys stay.
# ------------------------------------------------------------------------------

static func normalize(data: Dictionary) -> Dictionary:
	var out: Dictionary = data.duplicate(true)
	_fill(out, recording_defaults())
	var battles: Array = []
	for raw: Variant in DictUtil.array_of(out["battles"]):
		var b: Dictionary = DictUtil.dict_of(raw).duplicate(true)
		_fill(b, battle_defaults())
		_fill(DictUtil.dict_of(b["end"]), end_entry("abandoned", 0))
		var rounds: Array = []
		for raw_round: Variant in DictUtil.array_of(b["rounds"]):
			var r: Dictionary = DictUtil.dict_of(raw_round).duplicate(true)
			_fill(r, round_defaults())
			rounds.append(r)
		b["rounds"] = rounds
		battles.append(b)
	out["battles"] = battles
	var notes: Array = []
	for raw_note: Variant in DictUtil.array_of(out["notes"]):
		var n: Dictionary = DictUtil.dict_of(raw_note).duplicate(true)
		_fill(n, note_defaults())
		notes.append(n)
	out["notes"] = notes
	return out

static func _fill(target: Dictionary, defaults: Dictionary) -> void:
	for key: String in defaults:
		if not target.has(key):
			target[key] = _copy(defaults[key])

static func _copy(value: Variant) -> Variant:
	if value is Dictionary:
		var d: Dictionary = value
		return d.duplicate(true)
	if value is Array:
		var a: Array = value
		return a.duplicate(true)
	return value

# ------------------------------------------------------------------------------
# Validation (refusals). Battle-level problems are separate: a battle that fails them is refused alone.
# ------------------------------------------------------------------------------

static func errors(data: Dictionary) -> Array[String]:
	var out: Array[String] = []
	if str(data.get("format", "")) != FORMAT:
		out.append("Not a playtest recording (format '%s')." % str(data.get("format", "")))
		return out
	var raw_version: Variant = data.get("version", null)
	if not (raw_version is int):
		out.append("Recording version missing or not a whole number.")
	else:
		var version: int = raw_version
		if version > VERSION:
			out.append("Newer recording (version %d; this build reads version %d)." % [version, VERSION])
		elif version < 1:
			out.append("Recording version %d is not valid." % version)
	if not ORIGINS.has(str(data.get("origin", "user"))):
		out.append("Unknown origin '%s' (use %s)." % [str(data.get("origin", "")), ", ".join(ORIGINS)])
	var parent: Variant = data.get("parent", null)
	if parent != null:
		var p: Dictionary = DictUtil.dict_of(parent)
		if p.is_empty() or DictUtil.to_int(p.get("round", 0)) < 1:
			out.append("Parent must be {id, battle, round} with round >= 1.")
	var content: Dictionary = DictUtil.dict_of(data.get("content", {}))
	if str(content.get("total", "")).is_empty():
		out.append("Content fingerprint missing (content.total).")
	if DictUtil.array_of(data.get("trail", [])).size() > TRAIL_MAX:
		out.append("Trail has more than %d events." % TRAIL_MAX)
	var seen: Dictionary = {}
	var notes: Array = DictUtil.array_of(data.get("notes", []))
	for i: int in range(notes.size()):
		var note: Dictionary = DictUtil.dict_of(notes[i])
		var n: int = DictUtil.to_int(note.get("n", 0))
		if n < 1 or seen.has(n):
			out.append("Note %d: n must be unique and >= 1 (got %d)." % [i + 1, n])
		seen[n] = true
		out.append_array(note_errors(note, "Note %d" % n))
	return out

static func note_errors(note: Dictionary, where: String) -> Array[String]:
	var out: Array[String] = []
	var text: String = str(note.get("text", ""))
	if text.length() < 1 or text.length() > NOTE_TEXT_MAX:
		out.append("%s: text must be 1..%d characters." % [where, NOTE_TEXT_MAX])
	for tag: String in JsonUtil.strings(note.get("tags", [])):
		if not NOTE_TAGS.has(tag):
			out.append("%s: unknown tag '%s' (use %s)." % [where, tag, ", ".join(NOTE_TAGS)])
	if not ORIGINS.has(str(note.get("origin", "user"))):
		out.append("%s: unknown origin '%s'." % [where, str(note.get("origin", ""))])
	if not NOTE_PHASES.has(str(note.get("phase", ""))):
		out.append("%s: unknown phase '%s'." % [where, str(note.get("phase", ""))])
	return out

static func battle_errors(battle: Dictionary) -> Array[String]:
	var out: Array[String] = []
	var where: String = "Battle %d" % DictUtil.to_int(battle.get("index", 0))
	var encounter: String = str(battle.get("encounter", ""))
	if encounter.is_empty() or not EncounterSource.exists(encounter):
		out.append("%s: unknown encounter '%s'." % [where, encounter])
	if not (battle.get("seed", null) is int):
		out.append("%s: seed missing or not a whole number." % where)
	if not SEED_SOURCES.has(str(battle.get("seed_source", "encounter"))):
		out.append("%s: unknown seed_source '%s'." % [where, str(battle.get("seed_source", ""))])
	if DictUtil.array_of(battle.get("roster", [])).is_empty():
		out.append("%s: roster missing." % where)
	var end: Dictionary = DictUtil.dict_of(battle.get("end", {}))
	if not END_RESULTS.has(str(end.get("result", "abandoned"))):
		out.append("%s: unknown end result '%s'." % [where, str(end.get("result", ""))])
	var expected: int = 1
	for raw: Variant in DictUtil.array_of(battle.get("rounds", [])):
		var r: Dictionary = DictUtil.dict_of(raw)
		var num: int = DictUtil.to_int(r.get("round", 0))
		if num != expected:
			out.append("%s: rounds must ascend from 1 without gaps (found round %d where %d was expected)." % [where, num, expected])
			break
		var state: String = str(r.get("rng_state", ""))
		if not state.is_valid_int():
			out.append("%s round %d: rng_state '%s' is not a decimal integer." % [where, num, state])
		if not PLAN_SOURCES.has(str(r.get("source", "player"))):
			out.append("%s round %d: unknown source '%s'." % [where, num, str(r.get("source", ""))])
		expected += 1
	return out

# Scenario file (H): {format: playtest_scenario, version: 1, id, title, why, encounter, seed}; seed required.
static func scenario_errors(s: Dictionary) -> Array[String]:
	var out: Array[String] = []
	if str(s.get("format", "")) != SCENARIO_FORMAT:
		out.append("Not a playtest scenario (format '%s')." % str(s.get("format", "")))
		return out
	if DictUtil.to_int(s.get("version", 0)) != 1:
		out.append("Scenario version %s is not supported (1)." % str(s.get("version", "")))
	if not JsonUtil.id_ok(str(s.get("id", ""))):
		out.append("Scenario id '%s' must be lowercase [a-z0-9_]." % str(s.get("id", "")))
	var encounter: String = str(s.get("encounter", ""))
	if encounter.is_empty() or not EncounterSource.exists(encounter):
		out.append("Scenario %s: unknown encounter '%s'." % [str(s.get("id", "")), encounter])
	if not (s.get("seed", null) is int):
		out.append("Scenario %s: seed missing or not a whole number." % str(s.get("id", "")))
	return out

# ------------------------------------------------------------------------------
# Battle end (shared by the recorder and the replayer)
# ------------------------------------------------------------------------------

# won / lost / fled from the session's outcome (OBJ1 rule 11) once the battle is over; timeout when the last round
# concluded with both sides standing; abandoned otherwise (the battle was left first).
static func end_of(session: CombatSession, rounds_done: int) -> Dictionary:
	if not session.outcome.is_empty():
		return end_entry(session.outcome, rounds_done)
	if session.combat_over:
		return end_entry(BattleBridge.outcome(session), rounds_done)
	if session.phase == CombatSession.Phase.REPLAY and session.current_round >= session.encounter.max_rounds:
		return end_entry("timeout", rounds_done)
	return end_entry("abandoned", rounds_done)

# Concluded rounds of a battle (rounds whose outcome was captured), in order.
static func concluded_rounds(battle: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for raw: Variant in DictUtil.array_of(battle.get("rounds", [])):
		var r: Dictionary = DictUtil.dict_of(raw)
		if DictUtil.dict_of(r.get("outcome", {})).is_empty():
			break
		out.append(r)
	return out

static func battle_at(data: Dictionary, index: int) -> Dictionary:
	var battles: Array = DictUtil.array_of(data.get("battles", []))
	return DictUtil.dict_of(battles[index]) if index >= 0 and index < battles.size() else {}

static func phase_name(session: CombatSession) -> String:
	if session == null:
		return ""
	match session.phase:
		CombatSession.Phase.PLANNING:
			return "planning"
		CombatSession.Phase.RESOLVING:
			return "resolving"
		CombatSession.Phase.REPLAY:
			return "replay"
	return ""
