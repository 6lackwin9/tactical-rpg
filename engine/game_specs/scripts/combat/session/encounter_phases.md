---
class: EncounterPhases
extends: RefCounted
description: "Tactical RPG - Encounter Phases (BOSS1, spec docs/specs/BOSS1.md section 1) Data-driven boss phases on an encounter's `phases` field (catalog and world encounters). A phase = {id, name, trigger, effects, pause_autoplay}; it issues no commands: its effects only change inputs between blocks or rounds (a line, a wave, a buff, an AI tweak). Static part: parsing with defaults and the shared validator (both catalogs). Instance part (one per battle, owned by CatalogEncounter, spec rules 1 to 6): fired    phase id -> [round, block] (block 0 = round start), in fire order due      ids met but not yet fired (fire before the next block, or at the next round start) dropped  ids that can no longer fire (their hp_below trigger unit went down / withdrew first) state() / restore() copy all three for BlockSnapshot.phase_state. Phase-wave units are spawned in build() and stay reserved (off the field, skipped by the end-of-battle check) until their phase fires. AI changes apply from the next enemy draft (live profile = base profile + the ai settings of every fired phase in fire order). No random numbers. Phase object: id              String  snake_case, unique in the encounter (required) name            String  1..40 chars (required) trigger         Dict    {\"kind\": \"hp_below\", \"unit\": ref, \"pct\": 1..99} or {\"kind\": \"round\", \"round\": 2..max_rounds} effects         Array   1..5 of: {\"type\": \"line\", \"text\": 1..160 chars}                                 at most one per phase {\"type\": \"wave\", \"units\": [refs], \"label\": \"\"}                         at most one per phase {\"type\": \"status\", \"unit\": ref, \"status\": buff, \"rounds\": 2, \"magnitude\": 0} {\"type\": \"ai\", \"unit\": ref, role / target_pref / reckless / flee_ratio / defend_below / heal_below} pause_autoplay  bool    default true Refs are ids as written in the encounter's lists (catalog unit ids or world character refs)."
---

# Properties
* `MAX_PHASES`: int = 4 (const)
* `MAX_EFFECTS`: int = 5 (const)
* `MAX_NAME`: int = 40 (const)
* `MAX_LINE`: int = 160 (const)
* `MAX_UNITS`: int = 12          # enemies + wave_units + every phase wave (const)
* `PCT_MIN`: int = 1 (const)
* `PCT_MAX`: int = 99 (const)
* `ROUND_MIN`: int = 2 (const)
* `STATUS_ROUNDS_MIN`: int = 1 (const)
* `STATUS_ROUNDS_MAX`: int = 5 (const)
* `DEFAULT_STATUS_ROUNDS`: int = 2 (const)
* `TRIGGER_HP_BELOW`: String = "hp_below" (const)
* `TRIGGER_ROUND`: String = "round" (const)
* `TRIGGERS`: Array[String] = [TRIGGER_HP_BELOW, TRIGGER_ROUND] (const)
* `EFFECT_LINE`: String = "line" (const)
* `EFFECT_WAVE`: String = "wave" (const)
* `EFFECT_STATUS`: String = "status" (const)
* `EFFECT_AI`: String = "ai" (const)
* `EFFECTS`: Array[String] = [EFFECT_LINE, EFFECT_WAVE, EFFECT_STATUS, EFFECT_AI] (const)
* `PHASE_KEYS`: Array[String] = ["id", "name", "trigger", "effects", "pause_autoplay"] (const)
* `TRIGGER_KEYS`: Dictionary = { (const)
* `AI_ROLES`: Array[String] = ["AGGRESSOR", "SKIRMISHER", "SNIPER", "SUPPORT"] (const)
* `AI_TARGET_PREFS`: Array[String] = ["NEAREST", "WEAKEST"] (const)
* `AI_RATIOS`: Array[String] = ["flee_ratio", "defend_below", "heal_below"] (const)
* `AI_SETTINGS`: Array[String] = ["role", "target_pref", "reckless", "flee_ratio", "defend_below", "heal_below"] (const)
* `EFFECT_KEYS`: Dictionary = { (const)
* `SUBTYPE`: String = "Phase"     # EVENT entry skill_subtype of a phase (const)
* `OUTCOME`: String = "PHASE" (const)
* `phases`: Array[Dictionary] = []   # parsed, list order
* `fired`: Dictionary = {}           # phase id -> [round, block]
* `due`: Array[String] = []
* `dropped`: Array[String] = []
* `wave_units`: Dictionary = {}      # phase id -> Array[Combatant] (spawned in build, reserved until it fires)
* `_reserved`: Dictionary = {}       # Combatant -> phase id
* `_entries`: Dictionary = {}        # phase id -> ActionLogEntry of its last firing (re-logged after a reset)
* `_entry_ids`: Dictionary = {}      # ActionLogEntry -> phase id
* `_base_profiles`: Dictionary = {}  # Combatant -> AiProfile before any phase (captured on first use)
* `out`: Array[Combatant] = []
* `units`: Array[Combatant] = wave_units[id]
* `out`: Array[ActionLogEntry] = []
* `at`: Array = fired[id]
* `entry`: ActionLogEntry = _entries[id]
* `out`: Array[Dictionary] = []
* `out`: Array[Combatant] = []
* `c`: Combatant = v
* `out`: Array[String] = []
* `trigger`: Dictionary = p["trigger"]
* `when`: String = ""
* `types`: Array[String] = []
* `out`: Array[Dictionary] = []
* `src`: Dictionary = p
* `effects`: Array[Dictionary] = []
* `out`: Array[String] = []
* `out`: Array[String] = []
* `errors`: Array[String] = []
* `raw`: Variant = entry.get("phases", [])
* `list`: Array = raw
* `enemies`: Array[String] = _strings(entry.get("enemies", []))
* `waves`: Array[String] = _strings(entry.get("wave_units", []))
* `max_rounds`: int = DictUtil.to_int(entry.get("max_rounds", 0))
* `known`: Array[String] = enemies.duplicate()
* `phase_units`: int = 0
* `parsed`: Dictionary = parse([p])[0]
* `refs`: Array[String] = wave_refs_of(parsed)
* `seen`: Dictionary = {}
* `phase`: Dictionary = p
* `id`: String = str(phase.get("id", ""))
* `tag`: String = id if not id.is_empty() else "?"
* `pname`: String = str(phase.get("name", "")).strip_edges()
* `errors`: Array[String] = []
* `t`: Dictionary = raw
* `kind`: String = str(t.get("kind", ""))
* `allowed`: Array = TRIGGER_KEYS[kind]
* `unit_ref`: String = str(t.get("unit", ""))
* `pct`: Variant = t.get("pct", null)
* `r`: Variant = t.get("round", null)
* `errors`: Array[String] = []
* `list`: Array = raw if raw is Array else []
* `lines`: int = 0
* `waves`: int = 0
* `fx`: Dictionary = item
* `type`: String = str(fx.get("type", ""))
* `allowed`: Array = EFFECT_KEYS[type]
* `text`: String = str(fx.get("text", "")).strip_edges()
* `errors`: Array[String] = []
* `raw`: Variant = fx.get("units", null)
* `ref`: String = str(u)
* `errors`: Array[String] = []
* `ref`: String = str(fx.get("unit", ""))
* `errors`: Array[String] = _unit_errors(tag, fx, known)
* `status`: String = str(fx.get("status", ""))
* `rounds`: Variant = fx.get("rounds", DEFAULT_STATUS_ROUNDS)
* `magnitude`: Variant = fx.get("magnitude", 0)
* `mag`: int = DictUtil.to_int(magnitude)
* `lo`: int = StatusRegistry.magnitude_min(status)
* `hi`: int = StatusRegistry.magnitude_max(status)
* `errors`: Array[String] = _unit_errors(tag, fx, known)
* `any`: bool = false
* `v`: Variant = fx[key]
* `out`: Array[String] = []
* `out`: Array[Dictionary] = []

# Methods

### `static create(raw: Variant) -> EncounterPhases`
```logic
var p := EncounterPhases.new()
p.phases = parse(raw)
return p
```

### `is_empty() -> bool`
```logic
return phases.is_empty()
```

### `phase(id: String) -> Dictionary`
```logic
for p: Dictionary in phases:
	if str(p["id"]) == id:
		return p
return {}
```

### `is_fired(id: String) -> bool`
```logic
return fired.has(id)
```

### `is_dropped(id: String) -> bool`
```logic
return dropped.has(id)
```

### `reserve(id: String, c: Combatant) -> void`
```logic
if not wave_units.has(id):
	var fresh: Array[Combatant] = []
	wave_units[id] = fresh
var units: Array[Combatant] = wave_units[id]
units.append(c)
_reserved[c] = id
```

### `is_reserved(c: Combatant) -> bool`
```logic
return _reserved.has(c) and not fired.has(str(_reserved[c]))
```

### `state() -> Dictionary`
```logic
return {"fired": fired.duplicate(true), "due": due.duplicate(), "dropped": dropped.duplicate()}
```

### `restore(s: Dictionary) -> void`
```logic
fired = DictUtil.dict_of(s.get("fired", {})).duplicate(true)
due = _strings(s.get("due", []))
dropped = _strings(s.get("dropped", []))
```

### `check_block_end(unit_of: Callable) -> void`
```logic
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
```

### `mark_round(round_num: int) -> void`
```logic
for p: Dictionary in phases:
	var id: String = str(p["id"])
	var trigger: Dictionary = p["trigger"]
	if str(trigger["kind"]) == TRIGGER_ROUND and DictUtil.to_int(trigger["round"]) == round_num and not fired.has(id) and not dropped.has(id) and not due.has(id):
		due.append(id)
```

### `next_due() -> String`
```logic
for p: Dictionary in phases:
	if due.has(str(p["id"])):
		return str(p["id"])
return ""
```

### `mark_fired(id: String, round_num: int, block_num: int, entry: ActionLogEntry) -> void`
```logic
due.erase(id)
fired[id] = [round_num, block_num]
if _entries.has(id):
	_entry_ids.erase(_entries[id])
_entries[id] = entry
_entry_ids[entry] = id
```

### `info(entry: ActionLogEntry) -> Dictionary`
```logic
if entry == null or not _entry_ids.has(entry):
	return {}
var id: String = str(_entry_ids[entry])
var p: Dictionary = phase(id)
var line: String = ""
for fx: Dictionary in effects_of(p, EFFECT_LINE):
	line = str(fx["text"])
return {"id": id, "name": str(p.get("name", "")), "actor": entry.actor_name, "line": line, "pause_autoplay": DictUtil.to_bool(p.get("pause_autoplay", true), true)}
```

### `apply_profiles(profiles: Dictionary, units_of_ref: Callable, round_num: int) -> void`
```logic
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
```

### `static apply_ai(profile: AiProfile, fx: Dictionary) -> void`
```logic
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
```

### `static _parse_trigger(t: Dictionary) -> Dictionary`
```logic
var kind: String = str(t.get("kind", ""))
if kind == TRIGGER_HP_BELOW:
	return {"kind": kind, "unit": str(t.get("unit", "")), "pct": DictUtil.to_int(t.get("pct", 0))}
if kind == TRIGGER_ROUND:
	return {"kind": kind, "round": DictUtil.to_int(t.get("round", 0))}
return {"kind": kind}
```

### `static _parse_effect(fx: Dictionary) -> Dictionary`
```logic
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
```

### `static _is_whole(v: Variant) -> bool`
```logic
if v is int:
	return true
if v is float:
	var f: float = v
	return f == floorf(f)
return false
```

