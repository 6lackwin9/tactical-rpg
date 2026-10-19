class_name DojoPanel
extends UiPanel

# ==============================================================================
# Tactical RPG - Dojo Panel Base
# UiPanel with a strongly-typed DojoSession reference. Dojo components extend
# this and read/mutate state only through `session`.
# ==============================================================================

var session: DojoSession = null

func _bind_session(p_session: RefCounted) -> void:
	session = p_session as DojoSession
