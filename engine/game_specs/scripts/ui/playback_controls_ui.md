---
class: PlaybackControlsUI
extends: HBoxContainer
description: "Tactical RPG - Playback Controls Bar UI One compact row (COMBAT-UI2-L1): replay stepping, autoplay, reset, popout, phase badge, round advance. No gear button: equipment is changed outside battle."
---

# Properties
* `btn_step_prev`: Button
* `btn_play`: Button
* `btn_step_next`: Button
* `btn_reset_round`: Button
* `btn_reset_game`: Button
* `btn_popout`: Button
* `phase_badge_lbl`: Label
* `status_label`: Label
* `btn_next_round`: Button

# Signals
* `step_prev_pressed()`
* `play_pressed()`
* `step_next_pressed()`
* `reset_round_pressed()`
* `reset_game_pressed()`
* `popout_pressed()`
* `next_round_pressed()`

# Methods

### `_init() -> void`
```logic
name = "PlaybackControlsUI"
add_theme_constant_override("separation", 4)
_build_ui()
```

### `_build_ui() -> void`
```logic
var fs: int = UiTheme.hud_font()
var w: int = fs * 2 + 8
btn_step_prev = _btn("⏮", "Step previous block (replay)", w, fs, func() -> void: step_prev_pressed.emit())
btn_play = _btn("▶", "Autoplay round from BLK 1 / pause [Space]", w, fs, func() -> void: play_pressed.emit())
btn_step_next = _btn("⏭", "Step next block", w, fs, func() -> void: step_next_pressed.emit())
btn_reset_round = _btn("↺", "Reset active round (restart planning for the current round)", w, fs, func() -> void: reset_round_pressed.emit())
btn_reset_game = _btn("⟲", "Full reset: restart the whole battle from round 1", w, fs, func() -> void: reset_game_pressed.emit())
btn_popout = _btn("⛶", "Full-screen timeline [F / Tab]", w, fs, func() -> void: popout_pressed.emit())

phase_badge_lbl = UiTheme.label("PLANNING PHASE", fs, Color(0.95, 0.8, 0.35))
phase_badge_lbl.name = "PhaseBadge"
phase_badge_lbl.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
add_child(phase_badge_lbl)

# The hint is not a second text in the strip: it is the badge's tooltip and the log ticker's empty line.
status_label = UiTheme.label("Set party actions, then confirm the round.", fs, Color(0.7, 0.85, 0.95))
status_label.visible = false
add_child(status_label)

btn_next_round = Button.new()
btn_next_round.text = "⚔ ADVANCE TO ROUND 2"
btn_next_round.focus_mode = Control.FOCUS_NONE
btn_next_round.add_theme_font_size_override("font_size", fs)
var next_btn_style := UiTheme.box(Color(0.7, 0.45, 0.1, 0.95), Color(1.0, 0.85, 0.3, 1.0), 4, 6, 1)
btn_next_round.add_theme_stylebox_override("normal", next_btn_style)
btn_next_round.visible = false
btn_next_round.pressed.connect(func() -> void: next_round_pressed.emit())
add_child(btn_next_round)
```

### `_btn(text: String, tip: String, width: int, fs: int, on_press: Callable) -> Button`
```logic
var btn := Button.new()
btn.text = text
btn.tooltip_text = tip
btn.focus_mode = Control.FOCUS_NONE
btn.custom_minimum_size = Vector2(width, 0)
btn.add_theme_font_size_override("font_size", fs)
btn.pressed.connect(on_press)
add_child(btn)
return btn
```

### `set_phase_badge(text: String, col: Color = Color(0.95, 0.8, 0.35)) -> void`
```logic
if phase_badge_lbl != null:
	phase_badge_lbl.text = text
	phase_badge_lbl.add_theme_color_override("font_color", col)
```

### `set_status_text(text: String, col: Color = Color(0.7, 0.85, 0.95)) -> void`
```logic
if status_label != null:
	status_label.text = text
	status_label.add_theme_color_override("font_color", col)
	if phase_badge_lbl != null:
		phase_badge_lbl.tooltip_text = text
```

### `set_play_state(is_playing: bool, tooltip_txt: String = "") -> void`
```logic
if btn_play != null:
	btn_play.text = "⏸" if is_playing else "▶"
	if not tooltip_txt.is_empty():
		btn_play.tooltip_text = tooltip_txt
```

### `set_next_round_state(p_visible: bool, btn_text: String = "", tooltip_txt: String = "") -> void`
```logic
if btn_next_round != null:
	btn_next_round.visible = p_visible
	if not btn_text.is_empty():
		btn_next_round.text = btn_text
	if not tooltip_txt.is_empty():
		btn_next_round.tooltip_text = tooltip_txt
```

