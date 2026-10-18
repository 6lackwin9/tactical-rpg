---
class: PlaytestRecorder
extends: RefCounted
description: "Tactical RPG - Playtest Recorder (PLAYTEST1 rules 1, 2, 5) One recording (PlaytestRecord data) that follows CombatSession signals: mounted          a new battle entry (a round-1 redo, which restarts the battle, keeps the entry) round_started    a round entry with the RNG state the round starts from (a redo re-reads it) plan_edited      edit count + trail event round_confirmed  the confirmed plans (PlanCodec), plan time, source (player / auto: CombatSession.confirm_source) round_reset      the attempt's plans move to `discarded`, attempts + 1 round_concluded  outcome (RoundOutcome) and battle end, then the file is flushed Also holds the RefCounted parts of the UI trail and notes (capped trail, key filter, note stamps). Writes nothing unless PlaytestStore.enabled() (debug builds; tests force it)."
---

# Properties
* `data`: Dictionary = {}
* `path`: String = ""
* `session`: CombatSession = null
* `encounter_id`: String = ""
* `seed_source`: String = "encounter"
* `battle_index`: int = -1        # battle entry fed by the attached session (-1 = none yet)
* `scene`: String = ""            # current scene path (set by the hub; stamped into notes)
* `WHEEL_BUTTONS`: Array[String] = ["wheel_up", "wheel_down", "wheel_left", "wheel_right"] (const)
* `ERROR_MAX`: int = 500 (const)
* `CTL_PATH_SEGMENTS`: int = 4   # trail control descriptions (section F) (const)
* `CTL_TEXT_MAX`: int = 40 (const)
* `CTL_TIP_MAX`: int = 60 (const)
* `_start_ms`: int = 0
* `_error_count`: int = 0
* `_assets_seen`: Dictionary = {}
* `_logger`: PlaytestLogger = null
* `_redo_round`: int = -1         # round being redone (between round_reset and the next round_started)
* `_round_start_ms`: int = 0
* `_paused_ms`: int = 0
* `_pause_since`: int = -1
* `_edits`: int = 0
* `_confirmed`: bool = false

# Methods

### `static open(origin: String = "user", dir: String = "", p_id: String = "", content_root: String = ContentFingerprint.DEFAULT_ROOT) -> PlaytestRecorder`
```logic
var rec := PlaytestRecorder.new()
var folder: String = PlaytestStore.root if dir.is_empty() else dir
var rec_id: String = p_id if not p_id.is_empty() else PlaytestStore.new_id(folder)
rec.data = PlaytestRecord.create(rec_id, origin)
rec.data["content"] = ContentFingerprint.compute(content_root)
rec.path = PlaytestStore.path_for(rec_id, folder)
rec._start_ms = Time.get_ticks_msec()
return rec
```

### `static branch(source: Dictionary, battle_number: int, round_n: int, origin: String = "user", dir: String = "") -> PlaytestRecorder`
```logic
var rec: PlaytestRecorder = open(origin, dir)
rec.data["parent"] = {"id": str(source.get("id", "")), "battle": battle_number, "round": round_n}
rec.data["scenario"] = str(source.get("scenario", ""))
var src: Dictionary = PlaytestRecord.battle_at(source, battle_number)
var copy: Dictionary = src.duplicate(true)
var kept: Array = []
for raw: Variant in DictUtil.array_of(copy.get("rounds", [])):
	var r: Dictionary = DictUtil.dict_of(raw)
	if DictUtil.to_int(r.get("round", 0)) < round_n:
		kept.append(r)
copy["rounds"] = kept
copy["index"] = 0
copy["seed_source"] = "branch"
copy["end"] = PlaytestRecord.end_entry("abandoned", kept.size())
rec.data["battles"] = [copy]
rec.battle_index = 0
rec.encounter_id = str(copy.get("encounter", ""))
rec.seed_source = "branch"
return rec
```

### `attach(p_session: CombatSession, p_encounter_id: String = "", p_seed_source: String = "") -> void`
```logic
detach()
session = p_session
if not p_encounter_id.is_empty():
	encounter_id = p_encounter_id
if not p_seed_source.is_empty():
	seed_source = p_seed_source
session.mounted.connect(_on_mounted)
session.round_started.connect(_on_round_started)
session.plan_edited.connect(_on_plan_edited)
session.round_confirmed.connect(_on_plan_confirmed)
session.round_reset.connect(_on_round_reset)
session.round_concluded.connect(_on_round_concluded)
if session.resolver != null:
	if battle_index < 0:
		_on_mounted()
	if session.is_planning():
		_on_round_started(session.current_round)
```

### `detach() -> void`
```logic
if session == null:
	return
for pair: Array in [[session.mounted, _on_mounted], [session.round_started, _on_round_started], [session.plan_edited, _on_plan_edited],
		[session.round_confirmed, _on_plan_confirmed], [session.round_reset, _on_round_reset], [session.round_concluded, _on_round_concluded]]:
	var sig: Signal = pair[0]
	var fn: Callable = pair[1]
	if sig.is_connected(fn):
		sig.disconnect(fn)
session = null
```

### `flush() -> Error`
```logic
if not PlaytestStore.enabled() or path.is_empty():
	return ERR_UNAVAILABLE
drain_errors()
return PlaytestStore.save_json(path, data)
```

### `id() -> String`
```logic
return str(data.get("id", ""))
```

### `battle() -> Dictionary`
```logic
return PlaytestRecord.battle_at(data, battle_index)
```

### `round_entry(round_num: int) -> Dictionary`
```logic
for raw: Variant in DictUtil.array_of(battle().get("rounds", [])):
	var r: Dictionary = DictUtil.dict_of(raw)
	if DictUtil.to_int(r.get("round", 0)) == round_num:
		return r
return {}
```

### `refresh_rng_state() -> void`
```logic
var r: Dictionary = round_entry(session.current_round) if session != null else {}
if not r.is_empty():
	r["rng_state"] = str(session.resolver.rng.state)
```

### `_encounter_of(s: CombatSession) -> String`
```logic
if not encounter_id.is_empty():
	return encounter_id
if s.encounter is CatalogEncounter:
	var enc: CatalogEncounter = s.encounter
	return enc.encounter_id
return ""
```

### `_on_mounted() -> void`
```logic
if _redo_round == 1:
	return   # reset_round in round 1 restarts the battle: same entry, a new attempt of round 1
var battles: Array = DictUtil.array_of(data.get("battles", []))
if battle_index >= 0:
	add_trail("battle_reset", {"battle": battle_index, "round": session.current_round})
var b: Dictionary = PlaytestRecord.battle_defaults()
b["index"] = battles.size()
b["encounter"] = _encounter_of(session)
b["seed"] = session.encounter.rng_seed
b["seed_source"] = seed_source
b["roster"] = PlanCodec.roster(session)
b["pouches"] = PlanCodec.pouches(session)   # ITEMUSE1: replays rebuild the heroes' pouches from this
battles.append(b)
data["battles"] = battles
battle_index = battles.size() - 1
```

### `_on_round_started(round_num: int) -> void`
```logic
if battle_index < 0:
	return
var r: Dictionary = round_entry(round_num)
if r.is_empty():
	r = PlaytestRecord.round_defaults()
	r["round"] = round_num
	var b: Dictionary = battle()
	var rounds: Array = DictUtil.array_of(b.get("rounds", []))
	rounds.append(r)
	b["rounds"] = rounds
r["rng_state"] = str(session.resolver.rng.state)
r["plans"] = []
r["outcome"] = {}
r["edits"] = 0
r["plan_ms"] = 0
r["source"] = "player"
_redo_round = -1
_edits = 0
_confirmed = false
_round_start_ms = Time.get_ticks_msec()
_paused_ms = 0
_pause_since = -1
```

### `_on_plan_edited(kind: String, actor: Combatant) -> void`
```logic
_edits += 1
add_trail("plan_edit", {"kind": kind, "unit": PlanCodec.unit_ref(session, actor), "round": session.current_round})
```

### `_on_plan_confirmed(round_num: int) -> void`
```logic
var r: Dictionary = round_entry(round_num)
if r.is_empty():
	return
r["plans"] = PlanCodec.encode_plan(session, session.plan)
r["edits"] = _edits
r["source"] = session.confirm_source
r["plan_ms"] = maxi(0, Time.get_ticks_msec() - _round_start_ms - _paused_total())
_confirmed = true
add_trail("confirm", {"ok": true, "reason": "", "round": round_num})
```

### `confirm_refused(reason: String) -> void`
```logic
add_trail("confirm", {"ok": false, "reason": reason, "round": session.current_round if session != null else 0})
```

### `_on_round_reset(round_num: int) -> void`
```logic
var r: Dictionary = round_entry(round_num)
if not r.is_empty():
	var attempt: Array = DictUtil.array_of(r.get("plans", [])) if _confirmed else PlanCodec.encode_plan(session, session.plan)
	var discarded: Array = DictUtil.array_of(r.get("discarded", []))
	discarded.append(attempt)
	r["discarded"] = discarded
	r["attempts"] = DictUtil.to_int(r.get("attempts", 1), 1) + 1
	r["plans"] = []
	r["outcome"] = {}
	var b: Dictionary = battle()
	b["end"] = PlaytestRecord.end_entry("abandoned", round_num - 1)
_redo_round = round_num
add_trail("round_reset", {"battle": battle_index, "round": round_num})
```

### `_on_round_concluded(round_num: int, _over: bool) -> void`
```logic
var r: Dictionary = round_entry(round_num)
if r.is_empty():
	return
r["outcome"] = RoundOutcome.capture(session)
var end: Dictionary = PlaytestRecord.end_of(session, round_num)
var b: Dictionary = battle()
b["end"] = end
if str(end["result"]) != "abandoned":
	add_trail("battle_end", {"battle": battle_index, "round": round_num, "result": str(end["result"])})
flush()
```

### `pause_clock(paused: bool) -> void`
```logic
if paused and _pause_since < 0:
	_pause_since = Time.get_ticks_msec()
elif not paused and _pause_since >= 0:
	_paused_ms += Time.get_ticks_msec() - _pause_since
	_pause_since = -1
```

### `_paused_total() -> int`
```logic
return _paused_ms + (Time.get_ticks_msec() - _pause_since if _pause_since >= 0 else 0)
```

### `elapsed_ms() -> int`
```logic
return Time.get_ticks_msec() - _start_ms
```

### `add_trail(type: String, fields: Dictionary = {}) -> bool`
```logic
var trail: Array = DictUtil.array_of(data.get("trail", []))
if trail.size() >= PlaytestRecord.TRAIL_MAX:
	data["trail_dropped"] = DictUtil.to_int(data.get("trail_dropped", 0)) + 1
	return false
var event: Dictionary = {"t": elapsed_ms(), "type": type}
event.merge(fields, true)
trail.append(event)
data["trail"] = trail
return true
```

### `add_key(key: String, mods: String, text_focus: bool) -> bool`
```logic
if text_focus:
	return false
return add_trail("key", {"key": key, "mods": mods})
```

### `add_click(button: String, pos: Vector2, ctl: Variant, cell: Variant, handled: bool) -> bool`
```logic
if WHEEL_BUTTONS.has(button):
	return false
var fields: Dictionary = {"button": button, "pos": [roundi(pos.x), roundi(pos.y)], "ctl": ctl, "handled": handled}
if cell is Vector2i:
	var c: Vector2i = cell
	fields["cell"] = [c.x, c.y]
return add_trail("click", fields)
```

### `add_scene(scene_path: String, leaked: int = 0) -> bool`
```logic
scene = scene_path
return add_trail("scene", {"path": scene_path, "leaked": leaked})
```

### `add_error(level: String, msg: String, where: String) -> bool`
```logic
if _error_count >= ERROR_MAX:
	return false
_error_count += 1
return add_trail("error", {"level": level, "msg": msg.left(PlaytestLogger.MSG_MAX), "where": where})
```

### `observe_asset(ns: String, key: String, variant: String, source: String) -> void`
```logic
if source == "template":
	data["asset_templates"] = DictUtil.to_int(data.get("asset_templates", 0)) + 1
	return
var dedup: String = "%s/%s/%s/%s" % [ns, key, variant, source]
if _assets_seen.has(dedup):
	return
_assets_seen[dedup] = true
add_trail("asset", {"ns": ns, "key": key, "variant": variant, "source": source, "scene": scene})
```

### `start_hooks() -> void`
```logic
if not PlaytestStore.enabled() or _logger != null:
	return
_logger = PlaytestLogger.new()
OS.add_logger(_logger)
AssetRegistry.resolve_observer = observe_asset
```

### `stop_hooks() -> void`
```logic
drain_errors()
if _logger != null:
	OS.remove_logger(_logger)
	_logger = null
if AssetRegistry.resolve_observer == Callable(self, "observe_asset"):
	AssetRegistry.resolve_observer = Callable()
```

### `hooks_active() -> bool`
```logic
return _logger != null
```

### `drain_errors() -> void`
```logic
if _logger == null:
	return
for e: Dictionary in _logger.drain():
	add_error(str(e["level"]), str(e["msg"]), str(e["where"]))
```

### `note_stamp(ctx: Dictionary) -> Dictionary`
```logic
var note: Dictionary = PlaytestRecord.note_defaults()
note["n"] = DictUtil.array_of(data.get("notes", [])).size() + 1
note["t"] = elapsed_ms()
note["origin"] = str(ctx.get("origin", "user"))
note["scene"] = str(ctx.get("scene", scene))
note["battle"] = battle_index if session != null else -1
note["round"] = session.viewed_round if session != null else 0
note["block"] = session.playhead() if session != null else 0
note["phase"] = PlaytestRecord.phase_name(session)
for key: String in ["round", "block", "battle"]:
	if ctx.has(key):
		note[key] = DictUtil.to_int(ctx[key])
if ctx.has("phase"):
	note["phase"] = str(ctx["phase"])
note["mouse"] = _pair(ctx.get("mouse", [0, 0]))
note["viewport"] = _pair(ctx.get("viewport", [0, 0]))
note["ctl"] = DictUtil.dict_of(ctx.get("ctl", {})).duplicate(true)
note["tags"] = JsonUtil.strings(ctx.get("tags", []))
note["text"] = str(ctx.get("text", ""))
note["image"] = str(ctx.get("image", ""))
return note
```

### `add_note(ctx: Dictionary) -> Dictionary`
```logic
var note: Dictionary = note_stamp(ctx)
var problems: Array[String] = PlaytestRecord.note_errors(note, "Note %d" % DictUtil.to_int(note["n"]))
if not problems.is_empty():
	return {"ok": false, "note": note, "errors": problems}
var notes: Array = DictUtil.array_of(data.get("notes", []))
notes.append(note)
data["notes"] = notes
add_trail("note", {"n": note["n"]})
flush()
return {"ok": true, "note": note, "errors": problems}
```

### `static describe_control(path_text: String, cls: String, text: String, tip: String) -> Dictionary`
```logic
var parts: PackedStringArray = path_text.split("/", false)
var tail: PackedStringArray = parts.slice(maxi(0, parts.size() - CTL_PATH_SEGMENTS))
return {"path": "/".join(tail), "cls": cls, "text": text.left(CTL_TEXT_MAX), "tip": tip.left(CTL_TIP_MAX)}
```

### `static _pair(v: Variant) -> Array`
```logic
if v is Vector2:
	var p: Vector2 = v
	return [roundi(p.x), roundi(p.y)]
if v is Vector2i:
	var q: Vector2i = v
	return [q.x, q.y]
var a: Array = DictUtil.array_of(v)
return [DictUtil.to_int(a[0]), DictUtil.to_int(a[1])] if a.size() == 2 else [0, 0]
```

