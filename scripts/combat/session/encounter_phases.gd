class_name EncounterPhases
extends RefCounted

# ==============================================================================
# Tactical RPG - Encounter Phases (BOSS1, spec docs/specs/BOSS1.md section 1)
# Data-driven boss phases on an encounter's `phases` field (catalog and world encounters).
# A phase = {id, name, trigger, effects, pause_autoplay}; it issues no commands: its
# effects only change inputs between blocks or rounds (a line, a wave, a buff, an AI tweak).
#
# Step 1 (schema contract): parsing with defaults and the shared validator. The runtime
# (firing, reserved units, snapshot state) lands in step 2; until then `phases` is
# accepted and ignored by the battle code.
#
# Phase object:
#   id              String  snake_case, unique in the encounter (required)
#   name            String  1..40 chars (required)
#   trigger         Dict    {"kind": "hp_below", "unit": ref, "pct": 1..99} or {"kind": "round", "round": 2..max_rounds}
#   effects         Array   1..5 of:
#     {"type": "line", "text": 1..160 chars}                                 at most one per phase
#     {"type": "wave", "units": [refs], "label": ""}                         at most one per phase
#     {"type": "status", "unit": ref, "status": buff, "rounds": 2, "magnitude": 0}
#     {"type": "ai", "unit": ref, role / target_pref / reckless / flee_ratio / defend_below / heal_below}
#   pause_autoplay  bool    default true
# Refs are ids as written in the encounter's lists (catalog unit ids or world character refs).
# ==============================================================================

const MAX_PHASES: int = 4
const MAX_EFFECTS: int = 5
const MAX_NAME: int = 40
const MAX_LINE: int = 160
const MAX_UNITS: int = 12          # enemies + wave_units + every phase wave
const PCT_MIN: int = 1
const PCT_MAX: int = 99
const ROUND_MIN: int = 2
const STATUS_ROUNDS_MIN: int = 1
const STATUS_ROUNDS_MAX: int = 5
const DEFAULT_STATUS_ROUNDS: int = 2

const TRIGGER_HP_BELOW: String = "hp_below"
const TRIGGER_ROUND: String = "round"
const TRIGGERS: Array[String] = [TRIGGER_HP_BELOW, TRIGGER_ROUND]

const EFFECT_LINE: String = "line"
const EFFECT_WAVE: String = "wave"
const EFFECT_STATUS: String = "status"
const EFFECT_AI: String = "ai"
const EFFECTS: Array[String] = [EFFECT_LINE, EFFECT_WAVE, EFFECT_STATUS, EFFECT_AI]

const PHASE_KEYS: Array[String] = ["id", "name", "trigger", "effects", "pause_autoplay"]
const TRIGGER_KEYS: Dictionary = {
	TRIGGER_HP_BELOW: ["kind", "unit", "pct"],
	TRIGGER_ROUND: ["kind", "round"],
}
const AI_ROLES: Array[String] = ["AGGRESSOR", "SKIRMISHER", "SNIPER", "SUPPORT"]
const AI_TARGET_PREFS: Array[String] = ["NEAREST", "WEAKEST"]
const AI_RATIOS: Array[String] = ["flee_ratio", "defend_below", "heal_below"]
const AI_SETTINGS: Array[String] = ["role", "target_pref", "reckless", "flee_ratio", "defend_below", "heal_below"]
const EFFECT_KEYS: Dictionary = {
	EFFECT_LINE: ["type", "text"],
	EFFECT_WAVE: ["type", "units", "label"],
	EFFECT_STATUS: ["type", "unit", "status", "rounds", "magnitude"],
	EFFECT_AI: ["type", "unit", "role", "target_pref", "reckless", "flee_ratio", "defend_below", "heal_below"],
}

# ------------------------------------------------------------------------------
# Parsing (defaults filled; call on a validated entry)
# ------------------------------------------------------------------------------

# The encounter's phases as typed dictionaries with defaults: pause_autoplay true, status rounds 2 /
# magnitude 0, wave label "" and wave units as Array[String]. Non-dictionary items are skipped.
static func parse(raw: Variant) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for p: Variant in DictUtil.array_of(raw):
		if not (p is Dictionary):
			continue
		var src: Dictionary = p
		var effects: Array[Dictionary] = []
		for fx: Variant in DictUtil.array_of(src.get("effects", [])):
			if fx is Dictionary:
				effects.append(_parse_effect(fx))
		out.append({
			"id": str(src.get("id", "")),
			"name": str(src.get("name", "")),
			"trigger": _parse_trigger(DictUtil.dict_of(src.get("trigger", {}))),
			"effects": effects,
			"pause_autoplay": DictUtil.to_bool(src.get("pause_autoplay", true), true),
		})
	return out

static func _parse_trigger(t: Dictionary) -> Dictionary:
	var kind: String = str(t.get("kind", ""))
	if kind == TRIGGER_HP_BELOW:
		return {"kind": kind, "unit": str(t.get("unit", "")), "pct": DictUtil.to_int(t.get("pct", 0))}
	if kind == TRIGGER_ROUND:
		return {"kind": kind, "round": DictUtil.to_int(t.get("round", 0))}
	return {"kind": kind}

static func _parse_effect(fx: Dictionary) -> Dictionary:
	var type: String = str(fx.get("type", ""))
	match type:
		EFFECT_LINE:
			return {"type": type, "text": str(fx.get("text", ""))}
		EFFECT_WAVE:
			return {"type": type, "units": _strings(fx.get("units", [])), "label": str(fx.get("label", ""))}
		EFFECT_STATUS:
			return {
				"type": type,
				"unit": str(fx.get("unit", "")),
				"status": str(fx.get("status", "")),
				"rounds": DictUtil.to_int(fx.get("rounds", DEFAULT_STATUS_ROUNDS), DEFAULT_STATUS_ROUNDS),
				"magnitude": DictUtil.to_int(fx.get("magnitude", 0)),
			}
		EFFECT_AI:
			var out: Dictionary = {"type": type, "unit": str(fx.get("unit", ""))}
			for key: String in AI_SETTINGS:
				if not fx.has(key):
					continue
				if key == "reckless":
					out[key] = DictUtil.to_bool(fx[key])
				elif AI_RATIOS.has(key):
					out[key] = DictUtil.to_float(fx[key])
				else:
					out[key] = str(fx[key])
			return out
	return {"type": type}

# The wave units of one parsed phase (every wave effect, in order).
static func wave_refs_of(phase: Dictionary) -> Array[String]:
	var out: Array[String] = []
	for fx: Dictionary in _dicts(phase.get("effects", [])):
		if str(fx.get("type", "")) == EFFECT_WAVE:
			out.append_array(_strings(fx.get("units", [])))
	return out

# Every phase-wave unit of an encounter entry (the units reserved until their phase fires), in phase order.
static func reserved_refs(entry: Dictionary) -> Array[String]:
	var out: Array[String] = []
	for phase: Dictionary in parse(entry.get("phases", [])):
		out.append_array(wave_refs_of(phase))
	return out

# ------------------------------------------------------------------------------
# Validation (ContentCatalog._encounter_errors and WorldCatalog._encounter_errors)
# ------------------------------------------------------------------------------

# Problems with an encounter entry's `phases` (and the unit cap across enemies, wave_units and phase waves).
# `ref_ok(ref: String) -> bool` says whether a phase-wave ref resolves under the wave_units enemy filter.
static func problems(entry: Dictionary, ref_ok: Callable) -> Array[String]:
	var errors: Array[String] = []
	var raw: Variant = entry.get("phases", [])
	if not (raw is Array):
		errors.append("Phases must be a list of phase objects.")
		return errors
	var list: Array = raw
	var enemies: Array[String] = _strings(entry.get("enemies", []))
	var waves: Array[String] = _strings(entry.get("wave_units", []))
	var max_rounds: int = DictUtil.to_int(entry.get("max_rounds", 0))
	if list.size() > MAX_PHASES:
		errors.append("An encounter holds at most %d phases." % MAX_PHASES)
	# Units an effect may target: the encounter's enemies, its wave and every phase wave.
	var known: Array[String] = enemies.duplicate()
	known.append_array(waves)
	var phase_units: int = 0
	for p: Variant in list:
		if p is Dictionary:
			var parsed: Dictionary = parse([p])[0]
			var refs: Array[String] = wave_refs_of(parsed)
			known.append_array(refs)
			phase_units += refs.size()
	if enemies.size() + waves.size() + phase_units > MAX_UNITS:
		errors.append("Encounter enemies, waves and phase waves hold at most %d units." % MAX_UNITS)
	var seen: Dictionary = {}
	for p: Variant in list:
		if not (p is Dictionary):
			errors.append("Each phase must be an object.")
			continue
		var phase: Dictionary = p
		var id: String = str(phase.get("id", ""))
		var tag: String = id if not id.is_empty() else "?"
		if id.is_empty() or not ContentSchema.is_valid_id(id):
			errors.append("Phase '%s': the id must be lowercase snake_case (a-z, 0-9, _)." % tag)
		elif seen.has(id):
			errors.append("Phase ids must be unique: '%s'." % id)
		seen[id] = true
		for key: Variant in phase:
			if not PHASE_KEYS.has(str(key)):
				errors.append("Phase '%s': unknown key '%s'." % [tag, str(key)])
		var pname: String = str(phase.get("name", "")).strip_edges()
		if pname.is_empty():
			errors.append("Phase '%s' needs a name." % tag)
		elif pname.length() > MAX_NAME:
			errors.append("Phase '%s': the name holds at most %d characters." % [tag, MAX_NAME])
		if phase.has("pause_autoplay") and not (phase["pause_autoplay"] is bool):
			errors.append("Phase '%s': pause_autoplay must be true or false." % tag)
		errors.append_array(_trigger_errors(tag, phase.get("trigger", null), enemies, max_rounds))
		errors.append_array(_effects_errors(tag, phase.get("effects", null), known, ref_ok))
	return errors

static func _trigger_errors(tag: String, raw: Variant, enemies: Array[String], max_rounds: int) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is Dictionary):
		errors.append("Phase '%s' needs a trigger." % tag)
		return errors
	var t: Dictionary = raw
	var kind: String = str(t.get("kind", ""))
	if not TRIGGERS.has(kind):
		errors.append("Phase '%s': unknown trigger '%s'." % [tag, kind])
		return errors
	var allowed: Array = TRIGGER_KEYS[kind]
	for key: Variant in t:
		if not allowed.has(str(key)):
			errors.append("Phase '%s': unknown trigger key '%s'." % [tag, str(key)])
	if kind == TRIGGER_HP_BELOW:
		var unit_ref: String = str(t.get("unit", ""))
		if not enemies.has(unit_ref):
			errors.append("Phase '%s': trigger unit '%s' is not in enemies." % [tag, unit_ref])
		var pct: Variant = t.get("pct", null)
		if not _is_whole(pct) or DictUtil.to_int(pct) < PCT_MIN or DictUtil.to_int(pct) > PCT_MAX:
			errors.append("Phase '%s': pct must be %d..%d." % [tag, PCT_MIN, PCT_MAX])
	else:
		var r: Variant = t.get("round", null)
		if not _is_whole(r) or DictUtil.to_int(r) < ROUND_MIN or DictUtil.to_int(r) > max_rounds:
			errors.append("Phase '%s': round must be %d..%d." % [tag, ROUND_MIN, max_rounds])
	return errors

static func _effects_errors(tag: String, raw: Variant, known: Array[String], ref_ok: Callable) -> Array[String]:
	var errors: Array[String] = []
	var list: Array = raw if raw is Array else []
	if list.is_empty() or list.size() > MAX_EFFECTS:
		errors.append("Phase '%s' needs 1 to %d effects." % [tag, MAX_EFFECTS])
	var lines: int = 0
	var waves: int = 0
	for item: Variant in list:
		if not (item is Dictionary):
			errors.append("Phase '%s': each effect must be an object." % tag)
			continue
		var fx: Dictionary = item
		var type: String = str(fx.get("type", ""))
		if not EFFECTS.has(type):
			errors.append("Phase '%s': unknown effect '%s'." % [tag, type])
			continue
		var allowed: Array = EFFECT_KEYS[type]
		for key: Variant in fx:
			if not allowed.has(str(key)):
				errors.append("Phase '%s': unknown %s key '%s'." % [tag, type, str(key)])
		match type:
			EFFECT_LINE:
				lines += 1
				var text: String = str(fx.get("text", "")).strip_edges()
				if text.is_empty() or text.length() > MAX_LINE:
					errors.append("Phase '%s': a line needs 1 to %d characters." % [tag, MAX_LINE])
			EFFECT_WAVE:
				waves += 1
				errors.append_array(_wave_errors(tag, fx, ref_ok))
			EFFECT_STATUS:
				errors.append_array(_status_errors(tag, fx, known))
			EFFECT_AI:
				errors.append_array(_ai_errors(tag, fx, known))
	if lines > 1:
		errors.append("Phase '%s': one line per phase." % tag)
	if waves > 1:
		errors.append("Phase '%s': one wave per phase." % tag)
	return errors

static func _wave_errors(tag: String, fx: Dictionary, ref_ok: Callable) -> Array[String]:
	var errors: Array[String] = []
	var raw: Variant = fx.get("units", null)
	if not (raw is Array) or DictUtil.array_of(raw).is_empty():
		errors.append("Phase '%s': a wave needs at least one unit." % tag)
		return errors
	for u: Variant in DictUtil.array_of(raw):
		var ref: String = str(u)
		if not (u is String) or not bool(ref_ok.call(ref)):
			errors.append("Phase '%s': wave unit '%s' is not an enemy." % [tag, ref])
	if fx.has("label") and not (fx["label"] is String):
		errors.append("Phase '%s': the wave label must be text." % tag)
	return errors

static func _unit_errors(tag: String, fx: Dictionary, known: Array[String]) -> Array[String]:
	var errors: Array[String] = []
	var ref: String = str(fx.get("unit", ""))
	if not known.has(ref):
		errors.append("Phase '%s': effect unit '%s' is not in this encounter." % [tag, ref])
	return errors

static func _status_errors(tag: String, fx: Dictionary, known: Array[String]) -> Array[String]:
	var errors: Array[String] = _unit_errors(tag, fx, known)
	var status: String = str(fx.get("status", ""))
	if not StatusEffects.buffs().has(status):
		errors.append("Phase '%s': '%s' is not a buff." % [tag, status])
		return errors
	var rounds: Variant = fx.get("rounds", DEFAULT_STATUS_ROUNDS)
	if not _is_whole(rounds) or DictUtil.to_int(rounds) < STATUS_ROUNDS_MIN or DictUtil.to_int(rounds) > STATUS_ROUNDS_MAX:
		errors.append("Phase '%s': rounds must be %d..%d." % [tag, STATUS_ROUNDS_MIN, STATUS_ROUNDS_MAX])
	var magnitude: Variant = fx.get("magnitude", 0)
	var mag: int = DictUtil.to_int(magnitude)
	if StatusEffects.has_magnitude(status):
		var lo: int = StatusRegistry.magnitude_min(status)
		var hi: int = StatusRegistry.magnitude_max(status)
		if not _is_whole(magnitude) or mag < lo or mag > hi:
			errors.append("Phase '%s': %s magnitude must be %d..%d." % [tag, status, lo, hi])
	elif not _is_whole(magnitude) or mag != 0:
		errors.append("Phase '%s': %s takes no magnitude (0)." % [tag, status])
	return errors

static func _ai_errors(tag: String, fx: Dictionary, known: Array[String]) -> Array[String]:
	var errors: Array[String] = _unit_errors(tag, fx, known)
	var any: bool = false
	for key: String in AI_SETTINGS:
		if fx.has(key):
			any = true
	if not any:
		errors.append("Phase '%s': an ai effect needs at least one setting." % tag)
		return errors
	if fx.has("role") and not AI_ROLES.has(str(fx["role"])):
		errors.append("Phase '%s': role must be one of %s." % [tag, ", ".join(AI_ROLES)])
	if fx.has("target_pref") and not AI_TARGET_PREFS.has(str(fx["target_pref"])):
		errors.append("Phase '%s': target_pref must be one of %s." % [tag, ", ".join(AI_TARGET_PREFS)])
	if fx.has("reckless") and not (fx["reckless"] is bool):
		errors.append("Phase '%s': reckless must be true or false." % tag)
	for key: String in AI_RATIOS:
		if not fx.has(key):
			continue
		var v: Variant = fx[key]
		if not (v is int or v is float) or DictUtil.to_float(v) < 0.0 or DictUtil.to_float(v) > 1.0:
			errors.append("Phase '%s': %s must be 0.0..1.0." % [tag, key])
	return errors

# ------------------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------------------

# An int, or a float with no fraction (JSON numbers load as floats).
static func _is_whole(v: Variant) -> bool:
	if v is int:
		return true
	if v is float:
		var f: float = v
		return f == floorf(f)
	return false

static func _strings(raw: Variant) -> Array[String]:
	var out: Array[String] = []
	for v: Variant in DictUtil.array_of(raw):
		out.append(str(v))
	return out

static func _dicts(raw: Variant) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for v: Variant in DictUtil.array_of(raw):
		if v is Dictionary:
			out.append(v)
	return out
