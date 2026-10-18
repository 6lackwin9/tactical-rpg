---
class: DialoguePanel
extends: PanelContainer
description: "Tactical RPG - Dialogue debug panel (S11, spec DLG1-L commit 3) Thin view over one DialogueRunner: speaker, text, visible choices (locked ones greyed and disabled), Continue for `next` / end nodes, and the turn-in step (recipient picker over the active members, one checkbox per offered loot, Confirm / Cancel). Every action goes through the runner; the panel only re-renders. A shop or fight result is handed to the owner through the signals; the owner calls shop_closed() when its shop panel closes."
---

# Properties
* `runner`: DialogueRunner = null
* `pending`: Dictionary = {}          # the turn-in step: choose()'s {pending, quest, offers, picks_allowed}
* `recipient_id`: String = ""
* `picks`: Array[String] = []
* `_speaker`: Label = null
* `_text`: Label = null
* `_choices`: VBoxContainer = null
* `_turn_in`: VBoxContainer = null

# Signals
* `shop_opened(shop_qid: String)`
* `fight_started(fight: Dictionary)`
* `finished()`
* `noted(text: String)`

# Methods

### `_init() -> void`
```logic
add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_PANEL_BG))
var root := VBoxContainer.new()
add_child(root)
_speaker = UiTheme.label("", UiTheme.FONT_LG, UiTheme.COL_TITLE)
root.add_child(_speaker)
_text = UiTheme.label("")
_text.autowrap_mode = TextServer.AUTOWRAP_WORD
_text.custom_minimum_size = Vector2(240, 0)
root.add_child(_text)
_choices = VBoxContainer.new()
root.add_child(_choices)
_turn_in = VBoxContainer.new()
root.add_child(_turn_in)
visible = false
```

### `show_runner(r: DialogueRunner) -> void`
```logic
runner = r
pending = {}
visible = true
refresh()
```

### `is_open() -> bool`
```logic
return runner != null and visible
```

### `pick(index: int) -> Dictionary`
```logic
if runner == null:
	return {}
var res: Dictionary = runner.choose(index)
if not DictUtil.to_bool(res.get("ok", false)):
	noted.emit("Refused: %s" % str(res.get("errors", [])))
elif str(res.get("pending", "")) == "turn_in":
	pending = res
	picks = []
	recipient_id = runner.gs.party.active_ids[0] if not runner.gs.party.active_ids.is_empty() else ""
_route(res)
return res
```

### `advance() -> Dictionary`
```logic
if runner == null:
	return {}
var res: Dictionary = runner.advance()
if not DictUtil.to_bool(res.get("ok", false)):
	noted.emit("Refused: %s" % str(res.get("errors", [])))
_route(res)
return res
```

### `set_recipient(id: String) -> void`
```logic
recipient_id = id
```

### `toggle_pick(lqid: String, on: bool) -> void`
```logic
if on and not picks.has(lqid):
	picks.append(lqid)
elif not on:
	picks.erase(lqid)
```

### `confirm_turn_in() -> Dictionary`
```logic
if runner == null or pending.is_empty():
	return {}
var res: Dictionary = runner.confirm_turn_in(recipient_id, picks)
if DictUtil.to_bool(res.get("ok", false)):
	pending = {}
	noted.emit("Quest complete: %s" % str(res.get("granted", {})))
else:
	noted.emit("Turn-in refused: %s" % str(res.get("errors", [])))
_route(res)
return res
```

### `cancel_turn_in() -> void`
```logic
if runner == null:
	return
var _r: Dictionary = runner.cancel()
pending = {}
refresh()
```

### `shop_closed() -> void`
```logic
if runner == null:
	return
visible = true
_route(runner.close_shop())
```

### `_route(res: Dictionary) -> void`
```logic
for n: Variant in DictUtil.array_of(res.get("notes", [])):
	noted.emit(str(n))   # e.g. the gold-cap note
if res.has("shop") and DictUtil.to_bool(res.get("ok", false)):
	visible = false
	shop_opened.emit(str(res["shop"]))
	return
if res.has("fight"):
	_close()
	fight_started.emit(DictUtil.dict_of(res["fight"]))
	return
if runner != null and runner.closed:
	_close()
	finished.emit()
	return
refresh()
```

### `_close() -> void`
```logic
runner = null
pending = {}
visible = false
```

### `speaker_text() -> String`
```logic
return _speaker.text
```

### `refresh() -> void`
```logic
UiTheme.clear_children(_choices)
UiTheme.clear_children(_turn_in)
if runner == null:
	return
var v: Dictionary = runner.view()
_speaker.text = str(v.get("speaker_name", ""))
if not str(v.get("band", "")).is_empty():   # REP1-L 2.4: the talked-to character's faction band
	_speaker.text += "  (%s)" % RepRules.standing_text(str(v.get("faction_name", "")), str(v["band"]))
_text.text = str(v.get("text", ""))
if not pending.is_empty():
	_build_turn_in()
	return
var choices: Array = DictUtil.array_of(v.get("choices", []))
for i: int in range(choices.size()):
	var c: Dictionary = DictUtil.dict_of(choices[i])
	var locked: bool = DictUtil.to_bool(c.get("locked", false))
	var btn: Button = UiTheme.button(str(c.get("text", "")), "Locked" if locked else "DialogueRunner.choose(%d)" % i, pick.bind(i))
	btn.disabled = locked
	_choices.add_child(btn)
if choices.is_empty() and (DictUtil.to_bool(v.get("next", false)) or DictUtil.to_bool(v.get("end", false))):
	_choices.add_child(UiTheme.button("Continue", "DialogueRunner.advance", advance))
```

### `_build_turn_in() -> void`
```logic
_turn_in.add_child(UiTheme.header("Turn in: %s" % str(pending.get("quest", ""))))
var who := OptionButton.new()
for id: String in runner.gs.party.active_ids:
	who.add_item(id)
	if id == recipient_id:
		who.select(who.item_count - 1)
who.item_selected.connect(func(i: int) -> void: set_recipient(who.get_item_text(i)))
_turn_in.add_child(who)
var allowed: int = DictUtil.to_int(pending.get("picks_allowed", 0))
if allowed > 0:
	_turn_in.add_child(UiTheme.label("Pick up to %d:" % allowed, UiTheme.FONT_SM, UiTheme.COL_MUTED))
for raw: Variant in DictUtil.array_of(pending.get("offers", [])):
	var lqid: String = str(raw)
	var box := CheckBox.new()
	box.text = str(WorldCatalog.resolve(runner.gs.world_id, "loot", lqid).get("name", lqid))
	box.button_pressed = picks.has(lqid)
	box.toggled.connect(func(on: bool) -> void: toggle_pick(lqid, on))
	_turn_in.add_child(box)
_turn_in.add_child(UiTheme.button("Confirm", "DialogueRunner.confirm_turn_in", confirm_turn_in))
_turn_in.add_child(UiTheme.button("Cancel", "DialogueRunner.cancel", cancel_turn_in))
```

