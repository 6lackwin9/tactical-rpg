---
class: TrailReport
extends: RefCounted
description: "Tactical RPG - Trail Report (PLAYTEST1 rule 12, `tools/replay.gd --trail-report`) Reads a recording's UI trail (every scene) and flags interactions that may feel wrong even when the screen looks fine. Flags {kind, t, ctl, detail, note}; lines `FLAG <kind> t=<ms> <ctl> <detail>`: dead_click        unhandled click (or on a non-interactive control) with no reaction within 500 ms repeat_click      3+ clicks on one control or cell within 2 s (one flag per burst) long_hover        hover dwell >= 3000 ms refused_confirm   a refused plan confirm, with its reason error             an engine / script error, with the last user event before it undo_churn        3+ remove / undo / clear plan edits for one unit in one round double_open       a panel / modal opens again without a close close_without_open left_open         still open at a scene change or the end (not the scene's own panel: opened within 1 s of the scene) unprompted_open   no click / button / key in the 1000 ms before it, and not at scene start slow_open         more than 500 ms after the user event before it leak              a scene event with leaked > 0 asset_fallback    every asset event (the summary also prints asset_templates) Each flag names the nearest earlier note within 30 s. The report never fails a run."
---

# Properties
* `DEAD_CLICK_MS`: int = 500 (const)
* `REPEAT_WINDOW_MS`: int = 2000 (const)
* `REPEAT_MIN`: int = 3 (const)
* `LONG_HOVER_MS`: int = 3000 (const)
* `CHURN_MIN`: int = 3 (const)
* `SCENE_START_MS`: int = 1000 (const)
* `PROMPT_WINDOW_MS`: int = 1000 (const)
* `SLOW_OPEN_MS`: int = 500 (const)
* `NOTE_WINDOW_MS`: int = 30000 (const)
* `REACTIONS`: Array[String] = ["button", "panel", "modal", "plan_edit", "confirm", "view"] (const)
* `USER_EVENTS`: Array[String] = ["click", "button", "key"] (const)
* `CHURN_KINDS`: Array[String] = ["remove", "undo", "clear"] (const)
* `KINDS`: Array[String] = ["dead_click", "repeat_click", "long_hover", "refused_confirm", "error", "undo_churn", "double_open", (const)
* `INTERACTIVE`: Array[String] = ["Button", "CheckBox", "CheckButton", "OptionButton", "MenuButton", "LinkButton", "TextureButton", (const)
* `out`: Array[String] = []
* `f`: Dictionary = DictUtil.dict_of(raw)
* `note`: int = DictUtil.to_int(f.get("note", 0))
* `detail`: String = str(f.get("detail", ""))
* `parts`: Array[String] = []
* `counts`: Dictionary = DictUtil.dict_of(report.get("counts", {}))

# Methods

### `static build(data: Dictionary) -> Dictionary`
```logic
var trail: Array[Dictionary] = []
for raw: Variant in DictUtil.array_of(data.get("trail", [])):
	trail.append(DictUtil.dict_of(raw))
trail.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return DictUtil.to_int(a.get("t", 0)) < DictUtil.to_int(b.get("t", 0)))
var flags: Array[Dictionary] = []
_clicks(trail, flags)
_singles(trail, flags)
_churn(trail, flags)
_open_close(trail, flags)
flags.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return DictUtil.to_int(a["t"]) < DictUtil.to_int(b["t"]))
var notes: Array = DictUtil.array_of(data.get("notes", []))
var counts: Dictionary = {}
for kind: String in KINDS:
	counts[kind] = 0
for f: Dictionary in flags:
	f["note"] = _nearest_note(notes, DictUtil.to_int(f["t"]))
	counts[str(f["kind"])] = DictUtil.int_of(counts, str(f["kind"])) + 1
return {"flags": flags, "counts": counts, "events": trail.size(), "dropped": DictUtil.to_int(data.get("trail_dropped", 0)),
	"asset_templates": DictUtil.to_int(data.get("asset_templates", 0))}
```

### `static _flag(flags: Array[Dictionary], kind: String, e: Dictionary, detail: String, ctl_text: String = "") -> void`
```logic
flags.append({"kind": kind, "t": DictUtil.to_int(e.get("t", 0)), "ctl": ctl_text if not ctl_text.is_empty() else target_text(e), "detail": detail})
```

### `static target_text(e: Dictionary) -> String`
```logic
var ctl: Dictionary = DictUtil.dict_of(e.get("ctl", {}))
if not str(ctl.get("path", "")).is_empty():
	return str(ctl["path"])
var cell: Array = DictUtil.array_of(e.get("cell", []))
if cell.size() == 2:
	return "cell[%d,%d]" % [DictUtil.to_int(cell[0]), DictUtil.to_int(cell[1])]
return "-"
```

### `static _event_text(e: Dictionary) -> String`
```logic
var type: String = str(e.get("type", ""))
if type == "key":
	return "key %s%s" % [str(e.get("mods", "")), str(e.get("key", ""))]
return "%s %s t=%d" % [type, target_text(e), DictUtil.to_int(e.get("t", 0))]
```

### `static _interactive(e: Dictionary) -> bool`
```logic
var ctl: Variant = e.get("ctl", null)
if ctl == null:
	return true   # over the world (grid): the click is the game's to handle
var d: Dictionary = DictUtil.dict_of(ctl)
if d.has("interactive"):
	return DictUtil.to_bool(d["interactive"])
return INTERACTIVE.has(str(d.get("cls", "")))
```

### `static _clicks(trail: Array[Dictionary], flags: Array[Dictionary]) -> void`
```logic
var bursts: Dictionary = {}   # target -> [times of the current burst]
var flagged: Dictionary = {}  # target -> true while the current burst is flagged
for i: int in range(trail.size()):
	var e: Dictionary = trail[i]
	if str(e.get("type", "")) != "click":
		continue
	var t: int = DictUtil.to_int(e.get("t", 0))
	var unhandled: bool = not DictUtil.to_bool(e.get("handled", true))
	if unhandled or not _interactive(e):
		var reacted: bool = false
		for k: int in range(i + 1, trail.size()):
			var next: Dictionary = trail[k]
			if DictUtil.to_int(next.get("t", 0)) - t > DEAD_CLICK_MS:
				break
			if REACTIONS.has(str(next.get("type", ""))):
				reacted = true
				break
		if not reacted:
			_flag(flags, "dead_click", e, "%s click, %s, nothing reacted within %d ms" % [str(e.get("button", "")), "unhandled" if unhandled else "non-interactive control", DEAD_CLICK_MS])
	var target: String = target_text(e)
	if target == "-":
		continue
	var times: Array[int] = []
	times.assign(DictUtil.array_of(bursts.get(target, [])))
	if not times.is_empty() and t - times[times.size() - 1] > REPEAT_WINDOW_MS:
		times.clear()
		flagged.erase(target)
	times.append(t)
	while t - times[0] > REPEAT_WINDOW_MS:
		times.remove_at(0)
	bursts[target] = times
	if times.size() >= REPEAT_MIN and not flagged.has(target):
		flagged[target] = true
		_flag(flags, "repeat_click", e, "%d clicks within %d ms" % [times.size(), REPEAT_WINDOW_MS])
```

### `static _singles(trail: Array[Dictionary], flags: Array[Dictionary]) -> void`
```logic
var last_user: Dictionary = {}
for e: Dictionary in trail:
	var type: String = str(e.get("type", ""))
	if USER_EVENTS.has(type):
		last_user = e
	elif type == "hover" and DictUtil.to_int(e.get("ms", 0)) >= LONG_HOVER_MS:
		_flag(flags, "long_hover", e, "%d ms" % DictUtil.to_int(e.get("ms", 0)))
	elif type == "confirm" and not DictUtil.to_bool(e.get("ok", true)):
		_flag(flags, "refused_confirm", e, "round %d: %s" % [DictUtil.to_int(e.get("round", 0)), str(e.get("reason", ""))])
	elif type == "error":
		var after: String = ("after " + _event_text(last_user)) if not last_user.is_empty() else "no user event before it"
		_flag(flags, "error", e, "%s %s at %s (%s)" % [str(e.get("level", "")), str(e.get("msg", "")), str(e.get("where", "")), after], "-")
	elif type == "scene" and DictUtil.to_int(e.get("leaked", 0)) > 0:
		_flag(flags, "leak", e, "%d node(s) of the previous scene left in the tree (now %s)" % [DictUtil.to_int(e["leaked"]), str(e.get("path", ""))], "-")
	elif type == "asset":
		_flag(flags, "asset_fallback", e, "%s/%s/%s %s in %s" % [str(e.get("ns", "")), str(e.get("key", "")), str(e.get("variant", "")), str(e.get("source", "")), str(e.get("scene", ""))], "-")
```

### `static _churn(trail: Array[Dictionary], flags: Array[Dictionary]) -> void`
```logic
var counts: Dictionary = {}
for e: Dictionary in trail:
	if str(e.get("type", "")) != "plan_edit" or not CHURN_KINDS.has(str(e.get("kind", ""))):
		continue
	var key: String = "%s|%d" % [PlanCodec.ref_text(e.get("unit", null)), DictUtil.to_int(e.get("round", 0))]
	var n: int = DictUtil.int_of(counts, key) + 1
	counts[key] = n
	if n == CHURN_MIN:
		_flag(flags, "undo_churn", e, "%d remove / undo / clear edits for %s in round %d" % [n, PlanCodec.ref_text(e.get("unit", null)), DictUtil.to_int(e.get("round", 0))], PlanCodec.ref_text(e.get("unit", null)))
```

### `static _open_close(trail: Array[Dictionary], flags: Array[Dictionary]) -> void`
```logic
var open: Dictionary = {}   # control path -> {event, own: opened at scene start}
var scene_t: int = -1
var last_user: Dictionary = {}
for e: Dictionary in trail:
	var type: String = str(e.get("type", ""))
	var t: int = DictUtil.to_int(e.get("t", 0))
	if USER_EVENTS.has(type):
		last_user = e
	if type == "scene":
		_left_open(open, flags, "scene change to %s" % str(e.get("path", "")))
		open.clear()
		scene_t = t
		continue
	if type != "panel" and type != "modal":
		continue
	var key: String = target_text(e)
	if DictUtil.to_bool(e.get("open", false)):
		var at_start: bool = scene_t >= 0 and t - scene_t <= SCENE_START_MS
		if open.has(key):
			_flag(flags, "double_open", e, "%s opened again without a close" % type)
		elif not at_start:
			var since: int = t - DictUtil.to_int(last_user.get("t", -1000000)) if not last_user.is_empty() else -1
			if last_user.is_empty() or since > PROMPT_WINDOW_MS:
				_flag(flags, "unprompted_open", e, "%s opened with no click, button or key in the %d ms before" % [type, PROMPT_WINDOW_MS])
			elif since > SLOW_OPEN_MS:
				_flag(flags, "slow_open", e, "%s opened %d ms after %s" % [type, since, _event_text(last_user)])
		open[key] = {"event": e, "own": at_start}
	else:
		if not open.has(key):
			_flag(flags, "close_without_open", e, "%s closed but never opened" % type)
		open.erase(key)
_left_open(open, flags, "end of the recording")
```

### `static _left_open(open: Dictionary, flags: Array[Dictionary], at: String) -> void`
```logic
for key: Variant in open:
	var entry: Dictionary = DictUtil.dict_of(open[key])
	if DictUtil.to_bool(entry.get("own", false)):
		continue
	var e: Dictionary = DictUtil.dict_of(entry.get("event", {}))
	_flag(flags, "left_open", e, "%s still open at the %s" % [str(e.get("type", "")), at])
```

### `static _nearest_note(notes: Array, t: int) -> int`
```logic
var best: int = 0
var best_t: int = -1
for raw: Variant in notes:
	var n: Dictionary = DictUtil.dict_of(raw)
	var nt: int = DictUtil.to_int(n.get("t", 0))
	if nt <= t and t - nt <= NOTE_WINDOW_MS and nt > best_t:
		best_t = nt
		best = DictUtil.to_int(n.get("n", 0))
return best
```

