class_name CombatActionLogPanel
extends CombatPanel

# ==============================================================================
# Tactical RPG - Combat Action Log Panel
# One-line log ticker (ActionLogDockUI, newest line). ◀ ▶ step through entries,
# clicking the line or Expand asks the host for the full log modal over the grid.
# ==============================================================================

var dock: ActionLogDockUI

func _init() -> void:
	super._init()
	panel_id = "action_log"
	panel_title = "Resolution Log"
	panel_icon = "📜"
	default_min_size = Vector2(200, 28)
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	dock = ActionLogDockUI.new()
	dock.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	dock.set_ticker(true, UiTheme.hud_font())
	add_child(dock)

func _on_bound() -> void:
	dock.prev_pressed.connect(func() -> void: session.log_prev())
	dock.next_pressed.connect(func() -> void: session.log_next())
	dock.expand_requested.connect(func() -> void: modal_requested.emit("log"))
	dock.entry_clicked.connect(func(e: ActionLogEntry) -> void: session.select_log_entry(e))
	session.log_reset.connect(refresh)

func refresh() -> void:
	if session == null:
		return
	dock.empty_text = "No actions yet: %s" % session.status_text if session.is_planning() and not session.status_text.is_empty() else "No actions yet"
	dock.update_active_cell(session.log_entries, session.active_log_index)
