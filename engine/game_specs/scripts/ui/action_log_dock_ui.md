---
class: ActionLogDockUI
extends: PanelContainer
description: "Tactical RPG - Action Log Compact Resolution Dock Single-cell live resolution dock with action navigation buttons, active block badge, and modal expand trigger."
---

# Properties
* `btn_log_prev`: Button
* `btn_log_next`: Button
* `btn_expand_log`: Button
* `active_cell_dock`: PanelContainer
* `ticker`: bool = false   # true: one line with the newest entry (the combat screen); false: the full entry cell (Forge preview)
* `ticker_lbl`: Label
* `empty_text`: String = "No actions yet"

# Signals
* `prev_pressed()`
* `next_pressed()`
* `expand_requested()`
* `entry_clicked(entry: ActionLogEntry)`

# Methods

### `_init() -> void`
```logic
name = "ActionLogDockUI"
custom_minimum_size = Vector2(0, 28)
size_flags_horizontal = Control.SIZE_EXPAND_FILL
add_theme_stylebox_override("panel", UiTheme.box(Color(0.06, 0.08, 0.12, 0.95), Color(0.25, 0.32, 0.44, 0.8), 6, 6, 2))
_build_ui()
```

### `set_ticker(enabled: bool, fs: int = UiTheme.HUD_FONT) -> void`
```logic
ticker = enabled
for btn: Button in [btn_log_prev, btn_log_next, btn_expand_log]:
	btn.add_theme_font_size_override("font_size", fs)
	btn.custom_minimum_size = Vector2(0, 0)
btn_log_prev.custom_minimum_size = Vector2(fs * 2, 0)
btn_log_next.custom_minimum_size = Vector2(fs * 2, 0)
```

### `_build_ui() -> void`
```logic
var row := HBoxContainer.new()
row.add_theme_constant_override("separation", 6)
add_child(row)

active_cell_dock = PanelContainer.new()
active_cell_dock.size_flags_horizontal = Control.SIZE_EXPAND_FILL
active_cell_dock.size_flags_vertical = Control.SIZE_EXPAND_FILL
active_cell_dock.clip_contents = true
active_cell_dock.add_theme_stylebox_override("panel", UiTheme.box(Color(0.04, 0.06, 0.09, 0.7), Color(0.18, 0.24, 0.34, 0.5), 4, 4, 1))
active_cell_dock.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
active_cell_dock.gui_input.connect(func(event: InputEvent) -> void:
	var mouse_event := event as InputEventMouseButton
	if ticker and mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT:
		expand_requested.emit()
)
row.add_child(active_cell_dock)

btn_log_prev = Button.new()
btn_log_prev.text = "◀"
btn_log_prev.focus_mode = Control.FOCUS_NONE
btn_log_prev.tooltip_text = "Previous action [ [ ]"
btn_log_prev.pressed.connect(func() -> void: prev_pressed.emit())
row.add_child(btn_log_prev)

btn_log_next = Button.new()
btn_log_next.text = "▶"
btn_log_next.focus_mode = Control.FOCUS_NONE
btn_log_next.tooltip_text = "Next action [ ] ]"
btn_log_next.pressed.connect(func() -> void: next_pressed.emit())
row.add_child(btn_log_next)

btn_expand_log = Button.new()
btn_expand_log.text = "Expand"
btn_expand_log.focus_mode = Control.FOCUS_NONE
btn_expand_log.tooltip_text = "Open the full action log [L]"
btn_expand_log.pressed.connect(func() -> void: expand_requested.emit())
row.add_child(btn_expand_log)
```

### `update_active_cell(entries: Array[ActionLogEntry], active_idx: int) -> void`
```logic
for child in active_cell_dock.get_children():
	active_cell_dock.remove_child(child)
	child.queue_free()
ticker_lbl = null
var empty: bool = entries.is_empty() or active_idx < 0
if ticker or empty:
	ticker_lbl = UiTheme.label("", UiTheme.hud_font() if ticker else 9, Color(0.5, 0.6, 0.7) if empty else Color(0.85, 0.9, 0.95))
	ticker_lbl.clip_text = true
	ticker_lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	ticker_lbl.mouse_filter = Control.MOUSE_FILTER_IGNORE
	active_cell_dock.add_child(ticker_lbl)
if empty:
	ticker_lbl.text = empty_text
	btn_log_prev.disabled = true
	btn_log_next.disabled = true
	return
var entry: ActionLogEntry = entries[active_idx]
if ticker:
	ticker_lbl.text = "[%s] %s" % [entry.ref_tag, entry.detail_text if not entry.detail_text.is_empty() else "%s %s" % [entry.actor_name, entry.outcome_tag]]
else:
	active_cell_dock.add_child(CombatLogModal.format_action_cell(entry, false, func(e: ActionLogEntry) -> void: entry_clicked.emit(e)))
btn_log_prev.disabled = (active_idx <= 0)
btn_log_next.disabled = (active_idx >= entries.size() - 1)
```

