class_name DojoLogEntry
extends ActionLogEntry

# ==============================================================================
# Tactical RPG - Dojo Test Log Entry
# Standard battle-log entry plus an optional step-by-step math breakdown, so the
# dojo log can reuse the exact same cell renderer as the combat log.
# ==============================================================================

var breakdown_bbcode: String = ""

func _init() -> void:
	super._init()
