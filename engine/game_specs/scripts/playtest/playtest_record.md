---
class: PlaytestRecord
extends: RefCounted
description: "Tactical RPG - Playtest Record (PLAYTEST1 schema: defaults, normalisation, validation) A recording is plain JSON-shaped data (docs/specs/PLAYTEST1_L.md, sections A-I): recording {format, version, id, origin, created, engine, source_rev, scenario, parent, content, battles, trail, trail_dropped, asset_templates, notes} battle    {index, encounter, seed, seed_source, roster, pouches, rounds, end} (pouches: ITEMUSE1, [{ref, items}] per ally at battle start; [] in older recordings = units keep the pouch they are built with) round     {round, rng_state, plans, attempts, discarded, plan_ms, edits, outcome, source} source: who confirmed the plans, \"player\" or \"auto\" (AUTO1 battle autoplay); older files default to player note      {n, t, origin, scene, battle, round, block, phase, mouse, viewport, ctl, tags, text, image} Unknown keys are kept and ignored; missing keys take the defaults below. RNG states are decimal strings."
---

# Properties
* `FORMAT`: String = "playtest" (const)
* `VERSION`: int = 1 (const)
* `SCENARIO_FORMAT`: String = "playtest_scenario" (const)
* `VARIANT_FORMAT`: String = "playtest_variant" (const)
* `ORIGINS`: Array[String] = ["user", "agent"] (const)
* `SEED_SOURCES`: Array[String] = ["encounter", "scenario", "branch", "tool"] (const)
* `END_RESULTS`: Array[String] = ["won", "lost", "fled", "timeout", "abandoned"] (const)
* `PLAN_SOURCES`: Array[String] = ["player", "auto"] (const)
* `NOTE_TAGS`: Array[String] = ["mechanic", "ui", "ux", "bug"] (const)
* `NOTE_PHASES`: Array[String] = ["planning", "resolving", "replay", ""] (const)
* `TRAIL_MAX`: int = 20000 (const)
* `NOTE_TEXT_MAX`: int = 2000 (const)
* `out`: Array[String] = []
* `raw_version`: Variant = data.get("version", null)
* `version`: int = raw_version
* `parent`: Variant = data.get("parent", null)
* `p`: Dictionary = DictUtil.dict_of(parent)
* `content`: Dictionary = DictUtil.dict_of(data.get("content", {}))
* `seen`: Dictionary = {}
* `notes`: Array = DictUtil.array_of(data.get("notes", []))
* `note`: Dictionary = DictUtil.dict_of(notes[i])
* `n`: int = DictUtil.to_int(note.get("n", 0))
* `out`: Array[String] = []
* `text`: String = str(note.get("text", ""))
* `out`: Array[String] = []
* `where`: String = "Battle %d" % DictUtil.to_int(battle.get("index", 0))
* `encounter`: String = str(battle.get("encounter", ""))
* `end`: Dictionary = DictUtil.dict_of(battle.get("end", {}))
* `expected`: int = 1
* `r`: Dictionary = DictUtil.dict_of(raw)
* `num`: int = DictUtil.to_int(r.get("round", 0))
* `state`: String = str(r.get("rng_state", ""))
* `out`: Array[String] = []
* `encounter`: String = str(s.get("encounter", ""))
* `out`: Array[Dictionary] = []
* `r`: Dictionary = DictUtil.dict_of(raw)

# Methods

### `static recording_defaults() -> Dictionary`
```logic
return {"format": FORMAT, "version": VERSION, "id": "", "origin": "user", "created": "", "engine": "", "source_rev": "",
	"scenario": "", "parent": null, "content": {}, "battles": [], "trail": [], "trail_dropped": 0, "asset_templates": 0, "notes": []}
```

### `static battle_defaults() -> Dictionary`
```logic
return {"index": 0, "encounter": "", "seed": 0, "seed_source": "encounter", "roster": [], "pouches": [], "rounds": [], "end": end_entry("abandoned", 0)}
```

### `static round_defaults() -> Dictionary`
```logic
return {"round": 1, "rng_state": "0", "plans": [], "attempts": 1, "discarded": [], "plan_ms": 0, "edits": 0, "outcome": {}, "source": "player"}
```

### `static note_defaults() -> Dictionary`
```logic
return {"n": 1, "t": 0, "origin": "user", "scene": "", "battle": -1, "round": 0, "block": 0, "phase": "", "mouse": [0, 0],
	"viewport": [0, 0], "ctl": {}, "tags": [], "text": "", "image": ""}
```

### `static end_entry(result: String, rounds: int) -> Dictionary`
```logic
return {"result": result, "rounds": rounds}
```

### `static create(id: String, origin: String = "user") -> Dictionary`
```logic
var data: Dictionary = recording_defaults()
data["id"] = id
data["origin"] = origin
data["created"] = Time.get_datetime_string_from_system(false, false)
data["engine"] = str(Engine.get_version_info().get("string", ""))
data["source_rev"] = PlaytestStore.source_rev()
return data
```

### `static normalize(data: Dictionary) -> Dictionary`
```logic
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
```

### `static _fill(target: Dictionary, defaults: Dictionary) -> void`
```logic
for key: String in defaults:
	if not target.has(key):
		target[key] = _copy(defaults[key])
```

### `static _copy(value: Variant) -> Variant`
```logic
if value is Dictionary:
	var d: Dictionary = value
	return d.duplicate(true)
if value is Array:
	var a: Array = value
	return a.duplicate(true)
return value
```

### `static end_of(session: CombatSession, rounds_done: int) -> Dictionary`
```logic
if not session.outcome.is_empty():
	return end_entry(session.outcome, rounds_done)
if session.combat_over:
	return end_entry(BattleBridge.outcome(session), rounds_done)
if session.phase == CombatSession.Phase.REPLAY and session.current_round >= session.encounter.max_rounds:
	return end_entry("timeout", rounds_done)
return end_entry("abandoned", rounds_done)
```

### `static battle_at(data: Dictionary, index: int) -> Dictionary`
```logic
var battles: Array = DictUtil.array_of(data.get("battles", []))
return DictUtil.dict_of(battles[index]) if index >= 0 and index < battles.size() else {}
```

### `static phase_name(session: CombatSession) -> String`
```logic
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
```

