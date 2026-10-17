extends Node2D

# ==============================================================================
# Tactical RPG - Combat Arena (foundation scene for the combat UI)
# Creates a CombatSession for an encounter and mounts the CombatDockHost UI.
#   Rules & battle flow -> scripts/combat/session/ (CombatSession, CombatEncounter)
#   Panels & layout     -> scripts/ui/combat/      (CombatPanel subclasses, CombatDockHost)
# Callers (game flow, test harness) may assign `encounter` before the scene
# enters the tree (the Scene Runner does), or pass `-- --encounter=<catalog id>`;
# it defaults to the Goblin Skirmish. An AudioDirector plays the presenter's sounds (S14-L3).
# ==============================================================================

var encounter: CombatEncounter = null
var session: CombatSession = null
var host: CombatDockHost = null
var audio: AudioDirector = null

func _ready() -> void:
	if encounter == null:
		encounter = encounter_from_args(OS.get_cmdline_user_args())
	if encounter == null:
		encounter = CatalogEncounter.new("goblin_skirmish")
	session = CombatSession.new(encounter)

	var canvas := CanvasLayer.new()
	add_child(canvas)
	host = CombatDockHost.new()
	canvas.add_child(host)
	host.setup(session)
	audio = AudioDirector.new()
	add_child(audio)
	var grid: CombatGridPanel = host.get_panel("grid") as CombatGridPanel
	if grid != null:
		audio.connect_presenter(grid.presenter)
	audio.apply_volumes()
	session.start()

# `-- --encounter=<id>` loads any content-catalog encounter into the arena, e.g.
#   godot --path . res://scenes/combat_arena.tscn -- --encounter=veterans_drill
static func encounter_from_args(args: PackedStringArray) -> CombatEncounter:
	for arg: String in args:
		if arg.begins_with("--encounter="):
			var id: String = arg.trim_prefix("--encounter=")
			if EncounterSource.exists(id):
				return EncounterSource.create(id)
			push_warning("combat_arena: unknown encounter '%s', using the default." % id)
	return null

func _exit_tree() -> void:
	if session != null:
		session.dispose()
