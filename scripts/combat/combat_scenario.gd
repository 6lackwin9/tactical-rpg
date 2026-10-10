class_name CombatScenario
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat Scenario Model & Asset Container
# Encapsulates encounter configuration, combatant rosters, starting positions,
# conditional waves/reinforcements, and asset texture bindings.
# ==============================================================================

var scenario_name: String = "Tactical Encounter"
var description: String = ""
var arena_type: CombatGrid.ArenaType = CombatGrid.ArenaType.CLOSED
var exit_coords: Array[Vector2i] = [Vector2i(8, 4)]

var allies: Array[Combatant] = []
var enemies: Array[Combatant] = []
var reinforcements: Array[Combatant] = []

var initial_positions: Dictionary = {} # Combatant -> Vector2i
var token_textures: Dictionary = {}    # Combatant -> Texture2D
var sprite_textures: Dictionary = {}   # Combatant -> Texture2D
var tile_textures: Dictionary = {}     # String -> Texture2D

static func load_svg_texture(res_path: String) -> Texture2D:
	if FileAccess.file_exists(res_path):
		var file := FileAccess.open(res_path, FileAccess.READ)
		if file != null:
			var svg_text: String = file.get_as_text()
			var img := Image.new()
			var err: Error = img.load_svg_from_buffer(svg_text.to_utf8_buffer())
			if err == OK:
				return ImageTexture.create_from_image(img)
	return null

func get_token_texture(c: Combatant) -> Texture2D:
	if c != null and token_textures.has(c):
		@warning_ignore("unsafe_cast")
		return token_textures[c] as Texture2D
	return null

func get_sprite_texture(c: Combatant) -> Texture2D:
	if c != null and sprite_textures.has(c):
		@warning_ignore("unsafe_cast")
		return sprite_textures[c] as Texture2D
	return null

func get_initial_position(c: Combatant) -> Vector2i:
	if c != null and initial_positions.has(c):
		@warning_ignore("unsafe_cast")
		return initial_positions[c] as Vector2i
	return Vector2i(-1, -1)

static func create_default_arena_scenario() -> CombatScenario:
	var scenario := CombatScenario.new()
	scenario.scenario_name = "Warrior & Ranger vs Goblin Pack"
	scenario.description = "A standard 9x9 tactical closed arena encounter featuring an escape exit."
	scenario.arena_type = CombatGrid.ArenaType.CLOSED
	scenario.exit_coords = [Vector2i(8, 4)]

	# Tile textures
	scenario.tile_textures["floor"] = load_svg_texture("res://assets/tile_floor.svg")
	scenario.tile_textures["exit"] = load_svg_texture("res://assets/tile_exit.svg")
	scenario.tile_textures["center"] = load_svg_texture("res://assets/tile_center.svg")

	# Allied Party
	var warrior := Combatant.new("Warrior", 100, 15, 1)
	var ranger := Combatant.new("Ranger", 80, 14, 2)
	scenario.allies = [warrior, ranger]
	scenario.initial_positions[warrior] = Vector2i(4, 3)
	scenario.initial_positions[ranger] = Vector2i(3, 4)

	# Initial Enemies
	var brawler := Combatant.new("Enemy Brawler", 45, 12, 1)
	var scout := Combatant.new("Fleeing Scout", 30, 16, 1)
	scenario.enemies = [brawler, scout]
	scenario.initial_positions[brawler] = Vector2i(4, 1)
	scenario.initial_positions[scout] = Vector2i(6, 4)

	# Reinforcement Enemies
	var berserker := Combatant.new("Goblin Berserker", 55, 11, 1)
	var skirmisher := Combatant.new("Goblin Skirmisher", 40, 14, 1)
	var shaman := Combatant.new("Goblin Shaman", 35, 10, 2)
	var archer := Combatant.new("Goblin Archer", 35, 13, 3)
	scenario.reinforcements = [berserker, skirmisher, shaman, archer]
	scenario.initial_positions[berserker] = Vector2i(4, 5)
	scenario.initial_positions[skirmisher] = Vector2i(5, 4)
	scenario.initial_positions[shaman] = Vector2i(6, 4)
	scenario.initial_positions[archer] = Vector2i(6, 5)

	# Token Textures
	var tex_warrior := load_svg_texture("res://assets/token_warrior.svg")
	var tex_ranger := load_svg_texture("res://assets/token_ranger.svg")
	var tex_brawler := load_svg_texture("res://assets/token_brawler.svg")
	var tex_scout := load_svg_texture("res://assets/token_scout.svg")
	var tex_berserker := load_svg_texture("res://assets/token_berserker.svg")
	var tex_skirmisher := load_svg_texture("res://assets/token_skirmisher.svg")
	var tex_shaman := load_svg_texture("res://assets/token_shaman.svg")
	var tex_archer := load_svg_texture("res://assets/token_archer.svg")

	scenario.token_textures[warrior] = tex_warrior
	scenario.token_textures[ranger] = tex_ranger
	scenario.token_textures[brawler] = tex_brawler
	scenario.token_textures[scout] = tex_scout
	scenario.token_textures[berserker] = tex_berserker
	scenario.token_textures[skirmisher] = tex_skirmisher
	scenario.token_textures[shaman] = tex_shaman
	scenario.token_textures[archer] = tex_archer

	# Character Full Sprite Textures
	scenario.sprite_textures[warrior] = load_svg_texture("res://assets/sprite_warrior.svg")
	scenario.sprite_textures[ranger] = load_svg_texture("res://assets/sprite_ranger.svg")
	scenario.sprite_textures[brawler] = load_svg_texture("res://assets/sprite_brawler.svg")
	scenario.sprite_textures[scout] = load_svg_texture("res://assets/sprite_scout.svg")
	scenario.sprite_textures[berserker] = load_svg_texture("res://assets/sprite_berserker.svg")
	scenario.sprite_textures[skirmisher] = load_svg_texture("res://assets/sprite_skirmisher.svg")
	scenario.sprite_textures[shaman] = load_svg_texture("res://assets/sprite_shaman.svg")
	scenario.sprite_textures[archer] = load_svg_texture("res://assets/sprite_archer.svg")

	return scenario
