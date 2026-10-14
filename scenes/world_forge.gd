extends Node

# ==============================================================================
# Tactical RPG - World Forge (scene host)
# Mounts the WorldForge authoring UI. "Play this encounter" swaps it for the real
# combat arena running the chosen WorldEncounter (the world's own characters),
# with a "Back to World Forge" button; closing the arena returns with all unsaved
# world edits intact.
#   Tool  -> scripts/ui/world/   (WorldForge, WorldForgeSource, WorldPreview)
#   Data  -> scripts/world/      (WorldCatalog, WorldSchema, WorldFactory)
# ==============================================================================

const COMBAT_ARENA_SCENE: PackedScene = preload("res://scenes/combat_arena.tscn")

var forge_layer: CanvasLayer = null
var forge: WorldForge = null
var arena: Node = null
var back_layer: CanvasLayer = null

func _ready() -> void:
	forge_layer = CanvasLayer.new()
	add_child(forge_layer)
	forge = WorldForge.new()
	forge_layer.add_child(forge)
	forge.playtest_requested.connect(open_playtest)

func open_playtest(encounter: CombatEncounter) -> void:
	close_playtest()
	forge_layer.visible = false
	arena = COMBAT_ARENA_SCENE.instantiate()
	arena.set("encounter", encounter)
	add_child(arena)
	back_layer = CanvasLayer.new()
	back_layer.layer = 50
	add_child(back_layer)
	var back: Button = UiTheme.button("◀ Back to World Forge", "Close the battle and return to the World Forge", close_playtest, UiTheme.FONT_MD)
	back.set_anchors_preset(Control.PRESET_TOP_RIGHT)
	back.offset_left = -190
	back.offset_right = -12
	back.offset_top = 84
	back.offset_bottom = 104
	back_layer.add_child(back)

func close_playtest() -> void:
	if arena != null:
		arena.queue_free()
		arena = null
	if back_layer != null:
		back_layer.queue_free()
		back_layer = null
	if forge_layer != null:
		forge_layer.visible = true

# SceneRunner asks before leaving: unsaved world edits would be lost.
func has_unsaved_changes() -> bool:
	return WorldCatalog.is_dirty() or (forge != null and forge.draft_dirty)

func is_playtesting() -> bool:
	return arena != null

# Leaving with unsaved edits discards them (the menu already asked).
func _exit_tree() -> void:
	if WorldCatalog.is_dirty():
		WorldCatalog.reload_from_disk()
