---
class: PlaytestNoteBox
extends: PanelContainer
description: "Tactical RPG - Playtest Note Box (PLAYTEST1 rule 7) The small F8 feedback box: multiline text, four tag toggles (PlaytestRecord.NOTE_TAGS), Save (Ctrl+Enter) and Cancel (Esc). The hub captures the screenshot before the box opens, shows the box in an exclusive embedded Window over a full-screen click blocker (keys and clicks never reach the game) and writes the note. While open, every key the text field does not use is consumed here."
---

# Properties
* `text_edit`: TextEdit = null
* `tag_boxes`: Dictionary = {}   # tag -> CheckBox
* `hint`: Label = null
* `out`: Array[String] = []
* `box`: CheckBox = tag_boxes[tag]

# Signals
* `saved(text: String, tags: Array[String])`
* `cancelled()`

# Methods

### `_init() -> void`
```logic
name = "PlaytestNoteBox"
visible = false
process_mode = Node.PROCESS_MODE_ALWAYS
set_anchors_preset(Control.PRESET_FULL_RECT)
custom_minimum_size = Vector2(460, 0)
add_theme_stylebox_override("panel", UiTheme.box(Color(0.06, 0.09, 0.14, 0.97), UiTheme.COL_TITLE, 8, 12, 10))
var col := VBoxContainer.new()
col.add_theme_constant_override("separation", 8)
add_child(col)
col.add_child(UiTheme.label("📝 PLAYTEST NOTE (F8)", UiTheme.FONT_LG, UiTheme.COL_TITLE))
text_edit = TextEdit.new()
text_edit.custom_minimum_size = Vector2(436, 120)
text_edit.placeholder_text = "What felt wrong, confusing or good here?"
text_edit.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
text_edit.add_theme_font_size_override("font_size", UiTheme.FONT_LG)
col.add_child(text_edit)
var tags := HBoxContainer.new()
tags.add_theme_constant_override("separation", 10)
col.add_child(tags)
for tag: String in PlaytestRecord.NOTE_TAGS:
	var box := CheckBox.new()
	box.text = tag
	box.focus_mode = Control.FOCUS_NONE
	box.add_theme_font_size_override("font_size", UiTheme.FONT_MD)
	tags.add_child(box)
	tag_boxes[tag] = box
hint = UiTheme.label("Ctrl+Enter saves, Esc cancels. A screenshot is attached.", UiTheme.FONT_SM, UiTheme.COL_MUTED)
col.add_child(hint)
var row := HBoxContainer.new()
row.alignment = BoxContainer.ALIGNMENT_END
col.add_child(row)
row.add_child(UiTheme.button("Cancel", "Discard this note (Esc)", func() -> void: cancel()))
row.add_child(UiTheme.button("💾 Save", "Save the note (Ctrl+Enter)", func() -> void: save()))
```

### `open_box() -> void`
```logic
text_edit.text = ""
for tag: String in tag_boxes:
	var box: CheckBox = tag_boxes[tag]
	box.button_pressed = false
hint.text = "Ctrl+Enter saves, Esc cancels. A screenshot is attached."
visible = true
```

### `focus_text() -> void`
```logic
text_edit.grab_focus()
```

### `close_box() -> void`
```logic
visible = false
text_edit.release_focus()
```

### `save() -> void`
```logic
var text: String = text_edit.text.strip_edges()
if text.is_empty():
	hint.text = "Write a few words first (or Esc to cancel)."
	return
saved.emit(text.left(PlaytestRecord.NOTE_TEXT_MAX), selected_tags())
```

### `cancel() -> void`
```logic
cancelled.emit()
```

### `_input(event: InputEvent) -> void`
```logic
if not visible:
	return
var key: InputEventKey = event as InputEventKey
if key == null or not key.pressed or key.echo:
	return
if key.keycode == KEY_ESCAPE:
	cancel()
	get_viewport().set_input_as_handled()
elif (key.keycode == KEY_ENTER or key.keycode == KEY_KP_ENTER) and key.ctrl_pressed:
	save()
	get_viewport().set_input_as_handled()
```

### `_unhandled_key_input(_event: InputEvent) -> void`
```logic
if visible:
	get_viewport().set_input_as_handled()
```

### `_shortcut_input(event: InputEvent) -> void`
```logic
if visible and event is InputEventKey:
	get_viewport().set_input_as_handled()
```

