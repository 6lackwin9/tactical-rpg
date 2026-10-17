class_name CurrentGame
extends RefCounted

# ==============================================================================
# Tactical RPG - Current game (UI1-L, S13): the GameState being played, shared by the menus, the hub and the scene
# hosts. Static; the state lives on the Engine singleton like the catalogs (survives scene changes).
#   state        the GameState in play (null = no game)
#   slot         the last slot written or loaded (-1 = none)
#   clean_copy   state.to_dict() at the last save or load (is_dirty compares against it)
#   busy         set by a battle or dialogue scene host while it runs (saving is refused then)
# ==============================================================================

const STATE_KEY: StringName = &"current_game_state"

static func _data() -> Dictionary:
	if not Engine.has_meta(STATE_KEY):
		Engine.set_meta(STATE_KEY, {"state": null, "slot": -1, "clean_copy": {}, "busy": false})
	var d: Dictionary = Engine.get_meta(STATE_KEY)
	return d

static func state() -> GameState:
	return _data()["state"] as GameState

static func slot() -> int:
	return DictUtil.to_int(_data()["slot"], -1)

static func clean_copy() -> Dictionary:
	return DictUtil.dict_of(_data()["clean_copy"])

static func busy() -> bool:
	return DictUtil.to_bool(_data()["busy"])

static func set_busy(value: bool) -> void:
	_data()["busy"] = value

static func has_game() -> bool:
	return state() != null

# Starts playing `gs` (a new game or a loaded save); the clean copy is taken now, slot stays unknown until mark_saved.
static func start(gs: GameState, from_slot: int = -1) -> void:
	var d: Dictionary = _data()
	d["state"] = gs
	d["slot"] = from_slot
	d["clean_copy"] = gs.to_dict() if gs != null else {}
	d["busy"] = false

static func end() -> void:
	var d: Dictionary = _data()
	d["state"] = null
	d["slot"] = -1
	d["clean_copy"] = {}
	d["busy"] = false

# True when the state changed since the last save or load.
static func is_dirty() -> bool:
	var gs: GameState = state()
	return gs != null and not JsonUtil.deep_equal(gs.to_dict(), clean_copy())

# Records a successful write to (or read from) `p_slot`.
static func mark_saved(p_slot: int) -> void:
	var d: Dictionary = _data()
	d["slot"] = p_slot
	var gs: GameState = state()
	d["clean_copy"] = gs.to_dict() if gs != null else {}
