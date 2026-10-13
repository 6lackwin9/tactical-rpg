extends Node

# ==============================================================================
# Tactical RPG - Content Forge (scene host)
# Mounts the ContentForge authoring UI. Playtest swaps it for the real combat
# arena (combat_arena.tscn) running the chosen CatalogEncounter, with a
# "Back to Forge" button on top; closing the arena returns to the Forge with
# all unsaved catalog edits intact.
#   Tool    -> scripts/ui/forge/   (ContentForge, ForgeForm, ForgePreview)
#   Data    -> scripts/content/    (ContentCatalog, ContentSchema, ContentFactory)
# ==============================================================================

const COMBAT_ARENA_SCENE: PackedScene = preload("res://scenes/combat_arena.tscn")

var forge_layer: CanvasLayer = null
var forge: ContentForge = null
var arena: Node = null
var back_layer: CanvasLayer = null

func _ready() -> void:
	forge_layer = CanvasLayer.new()
	add_child(forge_layer)
	forge = ContentForge.new()
	forge_layer.add_child(forge)
	forge.playtest_requested.connect(open_playtest)

func open_playtest(encounter: CatalogEncounter) -> void:
	close_playtest()
	forge_layer.visible = false
	arena = COMBAT_ARENA_SCENE.instantiate()
	arena.set("encounter", encounter)
	add_child(arena)
	back_layer = CanvasLayer.new()
	back_layer.layer = 50
	add_child(back_layer)
	var back: Button = UiTheme.button("◀ Back to Forge", "Close the playtest and return to the Content Forge", close_playtest, UiTheme.FONT_MD)
	back.set_anchors_preset(Control.PRESET_TOP_RIGHT)
	back.offset_left = -150
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

func is_playtesting() -> bool:
	return arena != null

func _exit_tree() -> void:
	SkillCatalog.clear_cache()
	ContentCatalog.clear_cache()
