class_name CombatEncounter
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat Encounter (plug-in definition of a battle)
# Describes WHAT a battle is: combatants, grid and deployment, per-round plans,
# mid-round triggers (e.g. reinforcements) and round narration. CombatSession
# owns HOW it plays out (resolution, snapshots, playback, selection, log).
#
# The combatants created in build() are the single source of truth: the session
# simulates them and every UI panel (tokens, roster, inspector) renders them.
# Subclass and override the virtual methods below.
# ==============================================================================

var encounter_name: String = "Encounter"
var max_rounds: int = 1
var grid: CombatGrid = null
var allies: Array[Combatant] = []
var enemies: Array[Combatant] = []        # includes reinforcements not yet on the field
var party_stash: Inventory = null
var reinforcements_spawned: bool = false  # generic "extra wave has entered" flag (snapshotted)

var token_textures: Dictionary = {}   # Combatant -> Texture2D
var sprite_textures: Dictionary = {}  # Combatant -> Texture2D
var tile_textures: Dictionary = {}    # "floor" | "exit" | "center" -> Texture2D

# Creates combatants, grid and initial deployment. Called on start and full reset.
func build() -> void:
	pass

# Applies the starting state of `round_num` (round 1 right after build()).
# Returns intro info: {"entry": ActionLogEntry or null, "phase": String, "status": String, "log": String}
func prepare_round(_round_num: int) -> Dictionary:
	return {}

# Schedules every combatant's commands for `round_num` (player plans + enemy AI).
func schedule_round(_resolver: CombatResolver, _round_num: int) -> void:
	pass

# Mid-round trigger evaluated before a block resolves. Return an EVENT entry to
# announce something (the block then resolves on the next step), or null.
func on_block_start(_round_num: int, _block_num: int) -> ActionLogEntry:
	return null

# Narration when a round concludes: {"phase": String, "status": String, "log": String}
func round_summary(_round_num: int, _combat_over: bool) -> Dictionary:
	return {}

func advance_label(round_num: int) -> String:
	return "⚔ ADVANCE TO ROUND %d" % (round_num + 1)

func get_token_texture(c: Combatant) -> Texture2D:
	return _texture_from(token_textures, c)

func get_sprite_texture(c: Combatant) -> Texture2D:
	return _texture_from(sprite_textures, c)

func get_tile_texture(key: String) -> Texture2D:
	var raw: Variant = tile_textures.get(key, null)
	if raw is Texture2D:
		var tex: Texture2D = raw
		return tex
	return null

static func _texture_from(map: Dictionary, c: Combatant) -> Texture2D:
	if c == null:
		return null
	var raw: Variant = map.get(c, null)
	if raw is Texture2D:
		var tex: Texture2D = raw
		return tex
	return null

# Convenience for subclasses: an EVENT log entry for narration / triggers.
static func make_event_entry(round_num: int, block_num: int, actor: String, actor_is_ally: bool, skill: String, subtype: String, outcome: String, detail: String, status: String) -> ActionLogEntry:
	var entry := ActionLogEntry.new(round_num, block_num, "EVENT", actor, "", detail, 0, outcome)
	entry.actor_is_ally = actor_is_ally
	entry.skill_name = skill
	entry.skill_subtype = subtype
	entry.status_text = status
	return entry
