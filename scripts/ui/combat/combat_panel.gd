class_name CombatPanel
extends UiPanel

# ==============================================================================
# Tactical RPG - Combat Panel Base
# UiPanel with a strongly-typed CombatSession. Each combat panel wraps one view
# component (grid, roster, inspector...), pushes session state into it in
# refresh(), and turns its input signals into session actions. Panels never
# reference each other; requests the host must fulfil (full-screen modals) go
# through `modal_requested`.
# ==============================================================================

signal modal_requested(kind: String) # "log" | "timeline" | "gear"

var session: CombatSession = null

func _init() -> void:
	super._init()
	for side: String in ["left", "right", "top", "bottom"]:
		add_theme_constant_override("margin_%s" % side, 0)

func _bind_session(p_session: RefCounted) -> void:
	session = p_session as CombatSession
