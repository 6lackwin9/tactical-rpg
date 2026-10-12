extends Node2D

# ==============================================================================
# Tactical RPG - Combat Arena (foundation scene for the combat UI)
# Creates a CombatSession for an encounter and mounts the CombatDockHost UI.
#   Rules & battle flow -> scripts/combat/session/ (CombatSession, CombatEncounter)
#   Panels & layout     -> scripts/ui/combat/      (CombatPanel subclasses, CombatDockHost)
# Callers (game flow, test harness) may assign `encounter` before the scene
# enters the tree; it defaults to the scripted Goblin Ambush demo.
# ==============================================================================

var encounter: CombatEncounter = null
var session: CombatSession = null
var host: CombatDockHost = null

func _ready() -> void:
	if encounter == null:
		encounter = GoblinAmbushEncounter.new()
	session = CombatSession.new(encounter)

	var canvas := CanvasLayer.new()
	add_child(canvas)
	host = CombatDockHost.new()
	canvas.add_child(host)
	host.setup(session)
	session.start()

func _exit_tree() -> void:
	if session != null:
		session.dispose()
