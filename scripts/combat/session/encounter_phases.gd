class_name EncounterPhases
extends RefCounted

# ==============================================================================
# Tactical RPG - Encounter Phases (BOSS1, spec docs/specs/BOSS1.md section 1)
# Data-driven boss phases on an encounter's `phases` field (catalog and world encounters).
# A phase = {id, name, trigger, effects, pause_autoplay}; it issues no commands: its
# effects only change inputs between blocks or rounds (a line, a wave, a buff, an AI tweak).
#
# Static part: parsing with defaults and the shared validator (both catalogs).
# Instance part (one per battle, owned by CatalogEncounter, spec rules 1 to 6):
#   fired    phase id -> [round, block] (block 0 = round start), in fire order
#   due      ids met but not yet fired (fire before the next block, or at the next round start)
#   dropped  ids that can no longer fire (their hp_below trigger unit went down / withdrew first)
# state() / restore() copy all three for BlockSnapshot.phase_state. Phase-wave units are
# spawned in build() and stay reserved (off the field, skipped by the end-of-battle check)
# until their phase fires. AI changes apply from the next enemy draft (live profile = base
# profile + the ai settings of every fired phase in fire order). No random numbers.
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

const SUBTYPE: String = "Phase"     # EVENT entry skill_subtype of a phase
const OUTCOME: String = "PHASE"

# ------------------------------------------------------------------------------
# Runtime state (one battle)
# ------------------------------------------------------------------------------

var phases: Array[Dictionary] = []   # parsed, list order
var fired: Dictionary = {}           # phase id -> [round, block]
var due: Array[String] = []
var dropped: Array[String] = []
var wave_units: Dictionary = {}      # phase id -> Array[Combatant] (spawned in build, reserved until it fires)
var _reserved: Dictionary = {}       # Combatant -> phase id
var _entries: Dictionary = {}        # phase id -> ActionLogEntry of its last firing (re-logged after a reset)
var _entry_ids: Dictionary = {}      # ActionLogEntry -> phase id
var _base_profiles: Dictionary = {}  # Combatant -> AiProfile before any phase (captured on first use)

static func create(raw: Variant) -> EncounterPhases:
	var p := EncounterPhases.new()
	p.phases = parse(raw)
	return p

func is_empty() -> bool:
	return phases.is_empty()

func phase(id: String) -> Dictionary:
	for p: Dictionary in phases:
		if str(p["id"]) == id:
			return p
	return {}

func is_fired(id: String) -> bool:
	return fired.has(id)

func is_dropped(id: String) -> bool:
	return dropped.has(id)

# Registers a spawned phase-wave unit (reserved until phase `id` fires).
func reserve(id: String, c: Combatant) -> void:
	if not wave_units.has(id):
		var fresh: Array[Combatant] = []
		wave_units[id] = fresh
	var units: Array[Combatant] = wave_units[id]
	units.append(c)
	_reserved[c] = id

func is_reserved(c: Combatant) -> bool:
	return _reserved.has(c) and not fired.has(str(_reserved[c]))

func units_of(id: String) -> Array[Combatant]:
	var out: Array[Combatant] = []
	if wave_units.has(id):
		var units: Array[Combatant] = wave_units[id]
		out.assign(units)
	return out

# A copy of fired / due / dropped (BlockSnapshot.phase_state).
func state() -> Dictionary:
	return {"fired": fired.duplicate(true), "due": due.duplicate(), "dropped": dropped.duplicate()}

func restore(s: Dictionary) -> void:
	fired = DictUtil.dict_of(s.get("fired", {})).duplicate(true)
	due = _strings(s.get("due", []))
	dropped = _strings(s.get("dropped", []))

# Rule 1: after a block resolves, an unfired hp_below phase is due once its trigger unit is at or under pct % HP;
# a trigger unit that is down or withdrawn first drops it. `unit_of(ref) -> Combatant` = the first unit of that ref.
func check_block_end(unit_of: Callable) -> void:
	for p: Dictionary in phases:
		var id: String = str(p["id"])
		var trigger: Dictionary = p["trigger"]
		if str(trigger["kind"]) != TRIGGER_HP_BELOW or fired.has(id) or dropped.has(id) or due.has(id):
			continue
		var raw: Variant = unit_of.call(str(trigger["unit"]))
		var c: Combatant = raw as Combatant
		if c == null or not c.is_active():
			dropped.append(id)
		elif c.hp * 100 <= DictUtil.to_int(trigger["pct"]) * c.max_hp:
			due.append(id)

# Rule 1: round phases are due at the start of their round.
func mark_round(round_num: int) -> void:
	for p: Dictionary in phases:
		var id: String = str(p["id"])
		var trigger: Dictionary = p["trigger"]
		if str(trigger["kind"]) == TRIGGER_ROUND and DictUtil.to_int(trigger["round"]) == round_num and not fired.has(id) and not dropped.has(id) and not due.has(id):
			due.append(id)

# The first due phase in list order ("" = none).
func next_due() -> String:
	for p: Dictionary in phases:
		if due.has(str(p["id"])):
			return str(p["id"])
	return ""

# Marks `id` fired at [round, block] and keeps its EVENT entry (the encounter applies the effects).
func mark_fired(id: String, round_num: int, block_num: int, entry: ActionLogEntry) -> void:
	due.erase(id)
	fired[id] = [round_num, block_num]
	if _entries.has(id):
		_entry_ids.erase(_entries[id])
	_entries[id] = entry
	_entry_ids[entry] = id

# Entries of the phases fired at the start of `round_num` (fire order): re-logged after reset_round.
func round_start_entries(round_num: int) -> Array[ActionLogEntry]:
	var out: Array[ActionLogEntry] = []
	for id: Variant in fired:
		var at: Array = fired[id]
		if DictUtil.to_int(at[0]) == round_num and DictUtil.to_int(at[1]) == 0 and _entries.has(id):
			var entry: ActionLogEntry = _entries[id]
			out.append(entry)
	return out

# {id, name, actor, line, pause_autoplay} of a phase EVENT entry ({} for any other entry).
func info(entry: ActionLogEntry) -> Dictionary:
	if entry == null or not _entry_ids.has(entry):
		return {}
	var id: String = str(_entry_ids[entry])
	var p: Dictionary = phase(id)
	var line: String = ""
	for fx: Dictionary in effects_of(p, EFFECT_LINE):
		line = str(fx["text"])
	return {"id": id, "name": str(p.get("name", "")), "actor": entry.actor_name, "line": line, "pause_autoplay": DictUtil.to_bool(p.get("pause_autoplay", true), true)}

# Rule 5: every unit an ai effect names gets its live profile = base + the ai settings of every phase fired before
# `round_num` (or at its start), in fire order. `units_of_ref(ref) -> Array` of Combatants. Units no phase touches keep
# whatever profile they have (a sim preset, say), and nothing changes until a phase with an ai effect has fired.
func apply_profiles(profiles: Dictionary, units_of_ref: Callable, round_num: int) -> void:
	var settings: Array[Dictionary] = []   # ai effects that apply, in fire order
	for id: Variant in fired:
		var at: Array = fired[id]
		var r: int = DictUtil.to_int(at[0])
		if r < round_num or (r == round_num and DictUtil.to_int(at[1]) == 0):
			settings.append_array(effects_of(phase(str(id)), EFFECT_AI))
	if settings.is_empty() and _base_profiles.is_empty():
		return
	for fx: Dictionary in settings:   # capture the base of every touched unit before its first change
		for c: Combatant in _combatants(units_of_ref.call(str(fx["unit"]))):
			if not _base_profiles.has(c) and profiles.get(c, null) is AiProfile:
				var current: AiProfile = profiles[c]
				_base_profiles[c] = current.duplicate_profile()
	for c: Variant in _base_profiles:
		var base: AiProfile = _base_profiles[c]
		profiles[c] = base.duplicate_profile()
	for fx: Dictionary in settings:
		for c: Combatant in _combatants(units_of_ref.call(str(fx["unit"]))):
			if profiles.get(c, null) is AiProfile:
				var live: AiProfile = profiles[c]
				apply_ai(live, fx)

# One parsed ai effect onto a profile.
static func apply_ai(profile: AiProfile, fx: Dictionary) -> void:
	if fx.has("role"):
		var role_index: int = AI_ROLES.find(str(fx["role"]))
		if role_index >= 0:
			profile.role = role_index as AiProfile.Role
	if fx.has("target_pref"):
		profile.target_pref = AiProfile.TargetPref.WEAKEST if str(fx["target_pref"]) == "WEAKEST" else AiProfile.TargetPref.NEAREST
	if fx.has("reckless"):
		profile.reckless = DictUtil.to_bool(fx["reckless"])
	if fx.has("flee_ratio"):
		profile.flee_ratio = DictUtil.to_float(fx["flee_ratio"])
	if fx.has("defend_below"):
		profile.defend_below = DictUtil.to_float(fx["defend_below"])
	if fx.has("heal_below"):
		profile.heal_below = DictUtil.to_float(fx["heal_below"])

# The effects of a parsed phase of one type, in list order.
static func effects_of(p: Dictionary, type: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for fx: Dictionary in _dicts(p.get("effects", [])):
		if str(fx.get("type", "")) == type:
			out.append(fx)
	return out

static func _combatants(raw: Variant) -> Array[Combatant]:
	var out: Array[Combatant] = []
	for v: Variant in DictUtil.array_of(raw):
		if v is Combatant:
			var c: Combatant = v
			out.append(c)
	return out

# One line per phase for viewers: "Phase <name>: <trigger> -> <effect types>". `name_of(ref) -> String`.
static func summary_lines(raw: Variant, name_of: Callable) -> Array[String]:
	var out: Array[String] = []
	for p: Dictionary in parse(raw):
		var trigger: Dictionary = p["trigger"]
		var when: String = ""
		if str(trigger["kind"]) == TRIGGER_HP_BELOW:
			when = "%s at %d%% HP" % [str(name_of.call(str(trigger["unit"]))), DictUtil.to_int(trigger["pct"])]
		else:
			when = "round %d" % DictUtil.to_int(trigger.get("round", 0))
		var types: Array[String] = []
		for fx: Dictionary in _dicts(p["effects"]):
			types.append(str(fx["type"]))
		out.append("Phase %s: %s -> %s" % [str(p["name"]), when, ", ".join(types)])
	return out

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
