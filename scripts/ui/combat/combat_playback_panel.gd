class_name CombatPlaybackPanel
extends CombatPanel

# ==============================================================================
# Tactical RPG - Playback & Round Control Panel
# Step / auto-play / reset buttons, phase badge, status line and the round
# advance button. ⛶ asks the host for the timeline screen (no gear button: equipment is changed outside battle).
# Every button plays the UI click (AudioDirector.ui_event).
# ==============================================================================

var controls: PlaybackControlsUI

func _init() -> void:
	super._init()
	panel_id = "playback"
	panel_title = "Playback"
	panel_icon = "⏯"
	default_min_size = Vector2(300, 28)
	controls = PlaybackControlsUI.new()
	add_child(controls)

func _on_bound() -> void:
	controls.step_prev_pressed.connect(_clicked.bind(func() -> void: session.step_prev()))
	controls.play_pressed.connect(_clicked.bind(func() -> void: session.toggle_auto_play()))
	controls.step_next_pressed.connect(_clicked.bind(func() -> void: session.step_next()))
	controls.reset_round_pressed.connect(_clicked.bind(func() -> void: session.reset_round()))
	controls.reset_game_pressed.connect(_clicked.bind(func() -> void: session.reset_battle()))
	controls.next_round_pressed.connect(_clicked.bind(func() -> void: session.advance_round()))
	controls.popout_pressed.connect(_clicked.bind(func() -> void: modal_requested.emit("timeline")))

# A button press: the UI click sound (S14-L3; silent without a director or sound), then the action.
func _clicked(action: Callable) -> void:
	AudioDirector.ui_event("click")
	action.call()

func refresh() -> void:
	if session == null:
		return
	controls.set_phase_badge(session.phase_text, session.phase_color)
	controls.set_status_text(session.status_text)
	var play_tip: String = "Pause auto-play [Space]" if session.is_auto_playing else ("Auto-resolve Round %d [Space]" % session.current_round if session.is_live_view() else "Play replay from Block 1 [Space]")
	controls.set_play_state(session.is_auto_playing, play_tip)
	controls.set_next_round_state(not session.next_round_label.is_empty(), session.next_round_label, "Conclude Round %d and begin the next round" % session.current_round)
