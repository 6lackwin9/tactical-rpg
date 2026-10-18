---
class: PlaytestHub
extends: Node
description: "Tactical RPG - Playtest Hub (PLAYTEST1 rules 1, 5, 7) Child of the Scene Runner (survives scene changes). Opens one recording per app run on the first launch while the `record_playtests` pref is on, follows every battle the arena creates (attach_session), keeps the UI trail, writes F8 notes with a screenshot, starts take-over branch recordings, and flushes at round ends (the recorder), notes, scene changes and window close. Inert outside debug builds (PlaytestStore.enabled())."
---

# Properties
* `MENU_SCENE`: String = "res://scenes/scene_runner.tscn" (const)
* `JPG_QUALITY`: float = 0.85 (const)
* `record_enabled`: bool = true           # the Scene Runner pref `record_playtests`
* `recorder`: PlaytestRecorder = null
* `trail`: PlaytestUiTrail = null
* `note_box`: PlaytestNoteBox = null
* `shown_session`: CombatSession = null   # the battle on screen (live or playback)
* `playback`: Dictionary = {}             # {id, path, battle} while a recording is played back
* `scene_path`: String = ""
* `_layer`: CanvasLayer = null
* `note_window`: Window = null            # exclusive embedded window: while open, keys and clicks never reach the game
* `blocker`: ColorRect = null             # full-screen click blocker under the note window
* `_note_image`: Image = null
* `_note_ctx`: Dictionary = {}
* `_was_playing`: bool = false
* `_last_view`: Vector2i = Vector2i(-1, -1)

# Signals
* `take_over_requested(path: String, battle: int, round_num: int)`

# Methods

### `_enter_tree() -> void`
```logic
instance = self
```

### `_exit_tree() -> void`
```logic
close_recording()
if instance == self:
	instance = null
```

### `_ready() -> void`
```logic
name = "PlaytestHub"
process_mode = Node.PROCESS_MODE_ALWAYS
set_process_input(false)   # F8 is hooked only while recording
trail = PlaytestUiTrail.new()
trail.hub = self
add_child(trail)
_layer = CanvasLayer.new()
_layer.layer = 200
add_child(_layer)
blocker = ColorRect.new()
blocker.name = "NoteBlocker"
blocker.color = Color(0, 0, 0, 0.35)
blocker.mouse_filter = Control.MOUSE_FILTER_STOP
blocker.set_anchors_preset(Control.PRESET_FULL_RECT)
blocker.visible = false
_layer.add_child(blocker)
note_window = Window.new()
note_window.name = "NoteWindow"
note_window.borderless = true
note_window.exclusive = true
note_window.transient = true
note_window.unresizable = true
note_window.visible = false
note_window.close_requested.connect(_close_note)
add_child(note_window)
note_box = PlaytestNoteBox.new()
note_box.saved.connect(_on_note_saved)
note_box.cancelled.connect(_close_note)
note_window.add_child(note_box)
```

### `_notification(what: int) -> void`
```logic
if what == NOTIFICATION_WM_CLOSE_REQUEST and recorder != null:
	recorder.flush()
```

### `is_recording() -> bool`
```logic
return recorder != null
```

### `set_record_enabled(on: bool) -> void`
```logic
record_enabled = on
if not on:
	close_recording()
```

### `scene_changed(path: String, root: Node) -> void`
```logic
scene_path = path
if note_window.visible:
	_close_note()
if recorder != null and recorder.data.get("parent", null) != null:
	close_recording()
_show(null)   # the next scene attaches its own battle (attach_session / show_playback / start_branch)
playback = {}
if recorder != null:
	recorder.detach()
	recorder.flush()
if recorder == null and record_enabled and PlaytestStore.enabled() and path != MENU_SCENE:
	_start(PlaytestRecorder.open("user"))
trail.scene_changed(path, root)
```

### `_start(rec: PlaytestRecorder) -> void`
```logic
recorder = rec
recorder.scene = scene_path
recorder.start_hooks()
trail.start()
set_process_input(true)
```

### `close_recording() -> void`
```logic
if recorder == null:
	return
recorder.detach()
recorder.flush()
recorder.stop_hooks()
recorder = null
trail.stop()
set_process_input(false)
```

### `attach_session(session: CombatSession, encounter_id: String = "", seed_source: String = "", scenario: String = "") -> void`
```logic
_show(session)
playback = {}
if recorder == null:
	return
if not scenario.is_empty():
	recorder.data["scenario"] = scenario
recorder.attach(session, encounter_id, seed_source)
```

### `show_playback(session: CombatSession, data: Dictionary, battle: int, path: String) -> void`
```logic
_show(session)
playback = {"id": str(data.get("id", "")), "path": path, "battle": battle}
```

### `start_branch(source: Dictionary, battle: int, round_n: int, session: CombatSession) -> PlaytestRecorder`
```logic
_show(session)
playback = {}
if not PlaytestStore.enabled() or not record_enabled:
	return null
if recorder != null:
	close_recording()
var rec: PlaytestRecorder = PlaytestRecorder.branch(source, battle, round_n)
_start(rec)
rec.attach(session)
rec.add_scene(scene_path, 0)
rec.flush()
return rec
```

### `request_take_over(path: String, battle: int, round_num: int) -> void`
```logic
take_over_requested.emit(path, battle, round_num)
```

### `_show(session: CombatSession) -> void`
```logic
if shown_session != null and shown_session.changed.is_connected(_on_shown_changed):
	shown_session.changed.disconnect(_on_shown_changed)
	shown_session.plan_refused.disconnect(_on_plan_refused)
shown_session = session
_last_view = Vector2i(-1, -1)
if session != null:
	session.changed.connect(_on_shown_changed)
	session.plan_refused.connect(_on_plan_refused)
```

### `_on_plan_refused(_round_num: int, reason: String) -> void`
```logic
if recorder != null:
	recorder.confirm_refused(reason)
```

### `_on_shown_changed() -> void`
```logic
if recorder == null or shown_session == null or shown_session.is_live_view():
	_last_view = Vector2i(-1, -1)
	return
var view := Vector2i(shown_session.viewed_round, shown_session.playhead())
if view != _last_view:
	_last_view = view
	recorder.add_trail("view", {"round": view.x, "block": view.y})
```

### `_input(event: InputEvent) -> void`
```logic
var key: InputEventKey = event as InputEventKey
if key != null and key.pressed and not key.echo and key.keycode == KEY_F8 and recorder != null and not note_window.visible:
	open_note()
	get_viewport().set_input_as_handled()
```

### `open_note() -> void`
```logic
if recorder == null:
	return
_note_image = capture()
_note_ctx = note_context()
_was_playing = shown_session != null and shown_session.is_auto_playing
if _was_playing:
	shown_session.is_auto_playing = false
recorder.pause_clock(true)
blocker.visible = true
note_box.open_box()
note_window.size = Vector2i(note_box.get_combined_minimum_size().ceil())
note_window.popup_centered()
note_box.focus_text()
```

### `is_note_open() -> bool`
```logic
return note_window.visible
```

### `capture() -> Image`
```logic
if DisplayServer.get_name() == "headless":
	return null
var tex: ViewportTexture = get_viewport().get_texture()
return tex.get_image() if tex != null else null
```

### `note_context() -> Dictionary`
```logic
var vp: Viewport = get_viewport()
var ctx: Dictionary = {"scene": scene_path, "mouse": vp.get_mouse_position(), "viewport": vp.get_visible_rect().size, "ctl": trail.hovered_description()}
if shown_session != null and shown_session != recorder.session:
	ctx["battle"] = -1
	ctx["round"] = shown_session.viewed_round
	ctx["block"] = shown_session.playhead()
	ctx["phase"] = "replay" if not playback.is_empty() else PlaytestRecord.phase_name(shown_session)
if not playback.is_empty():
	ctx["playback"] = playback.duplicate()
return ctx
```

### `_on_note_saved(text: String, tags: Array[String]) -> void`
```logic
save_note(text, tags, _note_image, _note_ctx)
_close_note()
```

### `save_note(text: String, tags: Array[String], image: Image, ctx: Dictionary) -> Dictionary`
```logic
if recorder == null:
	return {"ok": false, "note": {}, "errors": ["Not recording."]}
var full: Dictionary = ctx.duplicate(true)
full["text"] = text
full["tags"] = tags
full["image"] = ""
var n: int = DictUtil.array_of(recorder.data.get("notes", [])).size() + 1
if image != null and not image.is_empty():
	var dir: String = PlaytestStore.image_dir(recorder.id(), recorder.path.get_base_dir())
	var file: String = "note_%d.jpg" % n
	if DirAccess.make_dir_recursive_absolute(dir) == OK and image.save_jpg(dir.path_join(file), JPG_QUALITY) == OK:
		full["image"] = file
return recorder.add_note(full)
```

### `_close_note() -> void`
```logic
note_box.close_box()
note_window.hide()
blocker.visible = false
_note_image = null
if recorder != null:
	recorder.pause_clock(false)
if _was_playing and shown_session != null:
	shown_session.is_auto_playing = true
	shown_session.notify_external_change()
_was_playing = false
```

