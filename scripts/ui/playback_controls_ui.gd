class_name PlaybackControlsUI
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Playback Controls Bar UI
# Modular playback panel managing replay stepping, autoplay, reset, and phase status
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
	custom_minimum_size = Vector2(165, 0)
	add_theme_constant_override("separation", 2)
	_build_ui()

func _build_ui() -> void:
	# Row 1: Playback & Action Buttons
	var btns_row := HBoxContainer.new()
	btns_row.add_theme_constant_override("separation", 3)
	add_child(btns_row)

	btn_step_prev = Button.new()
	btn_step_prev.text = "⏮"
	btn_step_prev.tooltip_text = "Step Previous Block (Replay)"
	btn_step_prev.custom_minimum_size = Vector2(24, 22)
	btn_step_prev.add_theme_font_size_override("font_size", 10)
	btn_step_prev.pressed.connect(func() -> void: step_prev_pressed.emit())
	btns_row.add_child(btn_step_prev)

	btn_play = Button.new()
	btn_play.text = "▶"
	btn_play.tooltip_text = "Autoplay Round from BLK 1 / Pause [Space]"
	btn_play.custom_minimum_size = Vector2(26, 22)
	btn_play.add_theme_font_size_override("font_size", 10)
	btn_play.pressed.connect(func() -> void: play_pressed.emit())
	btns_row.add_child(btn_play)

	btn_step_next = Button.new()
	btn_step_next.text = "⏭"
	btn_step_next.tooltip_text = "Step Next Block"
	btn_step_next.custom_minimum_size = Vector2(24, 22)
	btn_step_next.add_theme_font_size_override("font_size", 10)
	btn_step_next.pressed.connect(func() -> void: step_next_pressed.emit())
	btns_row.add_child(btn_step_next)

	btn_reset_round = Button.new()
	btn_reset_round.text = "↺"
	btn_reset_round.tooltip_text = "Reset Active Round (Restart Planning for Current Round)"
	btn_reset_round.custom_minimum_size = Vector2(24, 22)
	btn_reset_round.add_theme_font_size_override("font_size", 10)
	btn_reset_round.pressed.connect(func() -> void: reset_round_pressed.emit())
	btns_row.add_child(btn_reset_round)

	btn_reset_game = Button.new()
	btn_reset_game.text = "⟲"
	btn_reset_game.tooltip_text = "Full Reset: Restart Entire Tactical Battle from Round 1"
	btn_reset_game.custom_minimum_size = Vector2(24, 22)
	btn_reset_game.add_theme_font_size_override("font_size", 10)
	btn_reset_game.pressed.connect(func() -> void: reset_game_pressed.emit())
	btns_row.add_child(btn_reset_game)

	btn_popout = Button.new()
	btn_popout.text = "⛶"
	btn_popout.tooltip_text = "Popout Detailed Full-Screen Timeline Suite [F / ESC]"
	btn_popout.custom_minimum_size = Vector2(24, 22)
	btn_popout.add_theme_font_size_override("font_size", 10)
	btn_popout.pressed.connect(func() -> void: popout_pressed.emit())
	btns_row.add_child(btn_popout)

	# Row 2: Phase Badge
	phase_badge_lbl = Label.new()
	phase_badge_lbl.text = "PLANNING PHASE"
	phase_badge_lbl.add_theme_font_size_override("font_size", 8)
	phase_badge_lbl.add_theme_color_override("font_color", Color(0.95, 0.8, 0.35))
	add_child(phase_badge_lbl)

	# Row 3: Status / Instructions Label
	status_label = Label.new()
	status_label.text = "Set party actions, then Auto-Resolve."
	status_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	status_label.add_theme_font_size_override("font_size", 8)
	status_label.add_theme_color_override("font_color", Color(0.7, 0.85, 0.95))
	add_child(status_label)

	# Row 4: Round Advance Action Button
	btn_next_round = Button.new()
	btn_next_round.text = "⚔ ADVANCE TO ROUND 2"
	btn_next_round.tooltip_text = "Conclude Round 1 and advance to Round 2 (Warrior Last Stand)"
	btn_next_round.custom_minimum_size = Vector2(0, 22)
	btn_next_round.add_theme_font_size_override("font_size", 9)
	var next_btn_style := StyleBoxFlat.new()
	next_btn_style.bg_color = Color(0.7, 0.45, 0.1, 0.95)
	next_btn_style.border_color = Color(1.0, 0.85, 0.3, 1.0)
	next_btn_style.set_border_width_all(1)
	next_btn_style.set_corner_radius_all(4)
	next_btn_style.content_margin_left = 6
	next_btn_style.content_margin_right = 6
	next_btn_style.content_margin_top = 1
	next_btn_style.content_margin_bottom = 1
	btn_next_round.add_theme_stylebox_override("normal", next_btn_style)
	btn_next_round.visible = false
	btn_next_round.pressed.connect(func() -> void: next_round_pressed.emit())
	add_child(btn_next_round)

func set_phase_badge(text: String, col: Color = Color(0.95, 0.8, 0.35)) -> void:
	if phase_badge_lbl != null:
		phase_badge_lbl.text = text
		phase_badge_lbl.add_theme_color_override("font_color", col)

func set_status_text(text: String, col: Color = Color(0.7, 0.85, 0.95)) -> void:
	if status_label != null:
		status_label.text = text
		status_label.add_theme_color_override("font_color", col)

func set_play_state(is_playing: bool, tooltip_txt: String = "") -> void:
	if btn_play != null:
		btn_play.text = "⏸" if is_playing else "▶"
		if not tooltip_txt.is_empty():
			btn_play.tooltip_text = tooltip_txt

func set_next_round_state(is_visible: bool, btn_text: String = "", tooltip_txt: String = "") -> void:
	if btn_next_round != null:
		btn_next_round.visible = is_visible
		if not btn_text.is_empty():
			btn_next_round.text = btn_text
		if not tooltip_txt.is_empty():
			btn_next_round.tooltip_text = tooltip_txt
