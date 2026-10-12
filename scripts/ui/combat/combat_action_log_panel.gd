class_name CombatActionLogPanel
extends CombatPanel

# ==============================================================================
# Tactical RPG - Combat Action Log Panel
# Compact single-cell resolution log (ActionLogDockUI). ◀ ▶ step through entries,
# clicking a cell jumps to it, ⛶ EXPAND asks the host for the full log modal.
# ==============================================================================

var dock: ActionLogDockUI

func _init() -> void:
	super._init()
	panel_id = "action_log"
	panel_title = "Resolution Log"
	panel_icon = "📜"
	default_min_size = Vector2(430, 84)
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	dock = ActionLogDockUI.new()
	dock.size_flags_horizontal = Control.SIZE_EXPAND_FILL
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
	dock.update_active_cell(session.log_entries, session.active_log_index)
