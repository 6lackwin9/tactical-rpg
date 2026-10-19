class_name PlaybackControlsUI
extends HBoxContainer

# ==============================================================================
# Tactical RPG - Playback Controls Bar UI
# One compact row (COMBAT-UI2-L1): replay stepping, autoplay, reset, popout, phase badge, round advance.
# COMBAT-UI3: the bottom status bar: "Round 1 · Planning" and the hint (status line, plan warning or autoplay note).
# No gear button: equipment is changed outside battle.
# ==============================================================================

signal step_prev_pressed()
signal play_pressed()
signal step_next_pressed()
signal reset_round_pressed()
signal reset_game_pressed()
signal popout_pressed()
signal next_round_pressed()

var btn_step_prev: Button
var btn_play: Button
var btn_step_next: Button
var btn_reset_round: Button
var btn_reset_game: Button
var btn_popout: Button

var phase_badge_lbl: Label
var status_label: Label
var btn_next_round: Button

func _init() -> void:
	name = "PlaybackControlsUI"
	add_theme_constant_override("separation", 4)
	_build_ui()

func _build_ui() -> void:
	var fs: int = UiTheme.hud_font()
	var w: int = fs * 2 + 8
	btn_step_prev = _btn("⏮", "Step previous block (replay)", w, fs, func() -> void: step_prev_pressed.emit())
	btn_play = _btn("▶", "Autoplay round from BLK 1 / pause [Space]", w, fs, func() -> void: play_pressed.emit())
	btn_step_next = _btn("⏭", "Step next block", w, fs, func() -> void: step_next_pressed.emit())
	btn_reset_round = _btn("↺", "Reset active round (restart planning for the current round)", w, fs, func() -> void: reset_round_pressed.emit())
	btn_reset_game = _btn("⟲", "Full reset: restart the whole battle from round 1", w, fs, func() -> void: reset_game_pressed.emit())
	btn_popout = _btn("⛶", "Full-screen timeline [F / Tab]", w, fs, func() -> void: popout_pressed.emit())

	phase_badge_lbl = UiTheme.label("Round 1 · Planning", fs, Color(0.95, 0.8, 0.35))
	phase_badge_lbl.name = "PhaseBadge"
	phase_badge_lbl.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	add_child(phase_badge_lbl)

	# COMBAT-UI3: the hint beside the phase (the status line, a plan warning or the autoplay pause reason).
	status_label = UiTheme.label("Set party actions, then confirm the round.", fs, Color(0.7, 0.85, 0.95))
	status_label.name = "StatusHint"
	var gap := Control.new()   # a little air between the phase and the hint
	gap.custom_minimum_size = Vector2(6, 0)
	add_child(gap)
	status_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	status_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	status_label.clip_text = true
	status_label.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
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

func _btn(text: String, tip: String, width: int, fs: int, on_press: Callable) -> Button:
	var btn := Button.new()
	btn.text = text
	btn.tooltip_text = tip
	btn.focus_mode = Control.FOCUS_NONE
	btn.custom_minimum_size = Vector2(width, 0)
	btn.add_theme_font_size_override("font_size", fs)
	btn.pressed.connect(on_press)
	add_child(btn)
	return btn

func set_phase_badge(text: String, col: Color = Color(0.95, 0.8, 0.35)) -> void:
	if phase_badge_lbl != null:
		phase_badge_lbl.text = text
		phase_badge_lbl.add_theme_color_override("font_color", col)

func set_status_text(text: String, col: Color = Color(0.7, 0.85, 0.95)) -> void:
	if status_label != null:
		status_label.text = text
		status_label.tooltip_text = text
		status_label.add_theme_color_override("font_color", col)
		if phase_badge_lbl != null:
			phase_badge_lbl.tooltip_text = text

func set_play_state(is_playing: bool, tooltip_txt: String = "") -> void:
	if btn_play != null:
		btn_play.text = "⏸" if is_playing else "▶"
		if not tooltip_txt.is_empty():
			btn_play.tooltip_text = tooltip_txt

func set_next_round_state(p_visible: bool, btn_text: String = "", tooltip_txt: String = "") -> void:
	if btn_next_round != null:
		btn_next_round.visible = p_visible
		if not btn_text.is_empty():
			btn_next_round.text = btn_text
		if not tooltip_txt.is_empty():
			btn_next_round.tooltip_text = tooltip_txt
