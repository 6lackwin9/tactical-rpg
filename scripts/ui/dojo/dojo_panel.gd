class_name DojoPanel
extends MarginContainer

# ==============================================================================
# Tactical RPG - Dojo Panel Base
# A self-contained dojo component. It knows nothing about windows or layout:
# the workspace wraps it in a DojoWindow. Subclasses build their UI in _init(),
# then redraw from the shared DojoSession in refresh().
# ==============================================================================

var session: DojoSession = null
var panel_id: String = ""
var panel_title: String = ""
var panel_short_title: String = ""
var panel_icon: String = ""
var default_min_size: Vector2 = Vector2(260, 160)

func _init() -> void:
	add_theme_constant_override("margin_left", 8)
	add_theme_constant_override("margin_right", 8)
	add_theme_constant_override("margin_top", 6)
	add_theme_constant_override("margin_bottom", 6)

func bind(p_session: DojoSession) -> void:
	if session != null and session.changed.is_connected(refresh):
		session.changed.disconnect(refresh)
	session = p_session
	session.changed.connect(refresh)
	_on_bound()
	refresh()

func _on_bound() -> void:
	pass

func refresh() -> void:
	pass
