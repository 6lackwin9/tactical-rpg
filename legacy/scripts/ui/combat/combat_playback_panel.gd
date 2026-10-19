class_name CombatPlaybackPanel
extends CombatPanel

# ==============================================================================
# Tactical RPG - Playback & Round Control Panel (COMBAT-UI3: the bottom status bar)
# Step / auto-play / reset buttons, "Round 1 · Planning", the one-line hint and the round advance button.
# The hint is the autoplay pause / wait reason when there is one (note_fn, set by the dock host), else the planning
# hero's plan warning, else the session's status line. ⛶ asks the host for the timeline screen (no gear button:
# equipment is changed outside battle). Every button plays the UI click (AudioDirector.ui_event).
# ==============================================================================

var controls: PlaybackControlsUI
var note_fn: Callable = Callable()   # func() -> String: the autoplay note ("" = none)

func _init() -> void:
	super._init()
	panel_id = "playback"
	panel_title = "Playback"
	panel_icon = "⏯"
	default_min_size = Vector2(300, 28)
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	controls = PlaybackControlsUI.new()
	controls.size_flags_horizontal = Control.SIZE_EXPAND_FILL
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

# "Round 1 · Planning" / "Round 1 · Resolving"; the session's own phase text after the round (victory, replay ...).
func phase_line() -> String:
	if session.is_planning():
		return "Round %d · Planning" % session.current_round
	if session.phase == CombatSession.Phase.RESOLVING and session.is_live_view() and not session.combat_over:
		return "Round %d · Resolving" % session.current_round
	return session.phase_text

# The hint and whether it is a warning: autoplay note, else the plan warning, else the status line.
func hint() -> Array:
	var note: String = str(note_fn.call()) if note_fn.is_valid() else ""
	if not note.is_empty():
		return [note, true]
	if session.is_planning():
		var info: Dictionary = session.plan_info()
		var text: String = str(info.get("text", ""))
		var at: int = text.find("⚠")
		if DictUtil.to_bool(info.get("warn", false)) and at >= 0:
			return ["%s: %s" % [session.focused_hero.name, text.substr(at)], true]
	return [session.status_text, false]

func refresh() -> void:
	if session == null:
		return
	controls.set_phase_badge(phase_line(), session.phase_color)
	var h: Array = hint()
	controls.set_status_text(str(h[0]), UiTheme.COL_WARN if DictUtil.to_bool(h[1]) else Color(0.7, 0.85, 0.95))
	var play_tip: String = "Pause auto-play [Space]" if session.is_auto_playing else ("Auto-resolve Round %d [Space]" % session.current_round if session.is_live_view() else "Play replay from Block 1 [Space]")
	controls.set_play_state(session.is_auto_playing, play_tip)
	controls.set_next_round_state(not session.next_round_label.is_empty(), session.next_round_label, "Conclude Round %d and begin the next round" % session.current_round)
