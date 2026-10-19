class_name UiPanel
extends MarginContainer

# ==============================================================================
# Tactical RPG - UI Panel Base
# A self-contained UI component. It knows nothing about where it is hosted
# (static dock, floating UiWindow, overlay, modal column): hosts only read the
# metadata below, call bind(), and size it. Subclasses build their UI in _init(),
# then redraw from the bound session in refresh().
#
# A "session" is any headless RefCounted state object exposing `signal changed()`
# (DojoSession, CombatSession...). It is deliberately untyped here: storing a
# shared base-session type in this class while subclasses store their concrete
# session type crashes Godot 4.7 on script teardown, so typed adapters
# (e.g. DojoPanel) keep their own strongly-typed `session` reference instead.
# ==============================================================================

signal overlay_requested(content: UiPanel)
signal overlay_close_requested()

var ui_session: RefCounted = null
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

func bind(p_session: RefCounted) -> void:
	if ui_session != null:
		var old_changed := Signal(ui_session, &"changed")
		if old_changed.is_connected(refresh):
			old_changed.disconnect(refresh)
	ui_session = p_session
	Signal(ui_session, &"changed").connect(refresh)
	_bind_session(p_session)
	_on_bound()
	refresh()

# Typed adapters override this to keep a strongly-typed `session` reference.
func _bind_session(_p_session: RefCounted) -> void:
	pass

func _on_bound() -> void:
	pass

func refresh() -> void:
	pass
