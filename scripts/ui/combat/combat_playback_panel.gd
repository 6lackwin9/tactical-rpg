class_name CombatPlaybackPanel
extends CombatPanel

# ==============================================================================
# Tactical RPG - Playback & Round Control Panel
# Step / auto-play / reset buttons, phase badge, status line and the round
# advance button. ⛶ and 🎒 ask the host for the timeline / gear screens.
# ==============================================================================

var controls: PlaybackControlsUI

func _init() -> void:
	super._init()
	panel_id = "playback"
	panel_title = "Playback"
	panel_icon = "⏯"
	default_min_size = Vector2(170, 90)
	controls = PlaybackControlsUI.new()
	add_child(controls)

func _on_bound() -> void:
	controls.step_prev_pressed.connect(func() -> void: session.step_prev())
	controls.play_pressed.connect(func() -> void: session.toggle_auto_play())
	controls.step_next_pressed.connect(func() -> void: session.step_next())
	controls.reset_round_pressed.connect(func() -> void: session.reset_round())
	controls.reset_game_pressed.connect(func() -> void: session.reset_battle())
	controls.next_round_pressed.connect(func() -> void: session.advance_round())
	controls.popout_pressed.connect(func() -> void: modal_requested.emit("timeline"))
	controls.gear_pressed.connect(func() -> void: modal_requested.emit("gear"))
	controls.plan_pressed.connect(func() -> void: modal_requested.emit("plan"))

func refresh() -> void:
	if session == null:
		return
	controls.set_phase_badge(session.phase_text, session.phase_color)
	controls.set_plan_state(session.is_planning())
	controls.set_status_text(session.status_text)
	var play_tip: String = "Pause auto-play [Space]" if session.is_auto_playing else ("Auto-resolve Round %d [Space]" % session.current_round if session.is_live_view() else "Play replay from Block 1 [Space]")
	controls.set_play_state(session.is_auto_playing, play_tip)
	controls.set_next_round_state(not session.next_round_label.is_empty(), session.next_round_label, "Conclude Round %d and begin the next round" % session.current_round)
