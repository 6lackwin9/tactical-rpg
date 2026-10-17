class_name NewGameModel
extends RefCounted

# ==============================================================================
# Tactical RPG - New game model (UI1-L): the playable worlds with their fixed start party; choose() creates the game
# (NewGame.create_for_world), makes it current, writes the autosave and enters the hub.
# ==============================================================================

var router: MenuRouter = null
var last_errors: Array[String] = []   # errors of the last asked start (the host shows them)

func _init(p_router: MenuRouter) -> void:
	router = p_router

# [{id, name, tagline, party: [{unit, name, class, level}]}] for every playable world.
func worlds() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for w: String in NewGame.playable_worlds():
		var m: Dictionary = WorldCatalog.manifest(w)
		var party: Array[Dictionary] = []
		for unit_id: String in WorldCatalog.start_party(w):
			var e: Dictionary = ContentCatalog.get_entry("units", unit_id)
			var cls: Dictionary = ContentCatalog.get_entry("classes", str(e.get("class_id", "")))
			party.append({"unit": unit_id, "name": str(e.get("name", unit_id)), "class": str(cls.get("name", e.get("class_id", ""))), "level": DictUtil.to_int(e.get("level", 1), 1)})
		out.append({"id": w, "name": str(m.get("name", w)), "tagline": str(m.get("tagline", "")), "party": party})
	return out

# Q1: starting over replaces the autosave, so a readable one is asked about first.
func choose(world_id: String, p_seed: int = -1) -> Array[String]:
	var head: Dictionary = SaveGame.peek(SaveGame.AUTOSAVE_SLOT)
	if DictUtil.to_bool(head["ok"]) and NewGame.playable_worlds().has(world_id):
		var world: String = str(head["world_id"])
		var at: int = DictUtil.to_int(head["saved_at"])
		router.ask("Starting a new game replaces your autosave (%s, %s, %s). Continue?" % [str(WorldCatalog.manifest(world).get("name", world)),
			SaveSlotsModel.location_name(world, str(head["location_id"])), Time.get_datetime_string_from_unix_time(at, true) if at > 0 else "unknown time"], _start.bind(world_id, p_seed))
		var asked: Array[String] = []
		return asked
	return _start(world_id, p_seed)

func _start(world_id: String, p_seed: int) -> Array[String]:
	last_errors = _create(world_id, p_seed)
	return last_errors

func _create(world_id: String, p_seed: int) -> Array[String]:
	var res: Dictionary = NewGame.create_for_world(world_id, p_seed)
	var errors: Array[String] = []
	errors.assign(DictUtil.array_of(res["errors"]))
	var gs: GameState = GameState.of(res["state"])
	if gs == null:
		return errors
	CurrentGame.start(gs)
	errors = SaveGame.autosave(gs)
	if errors.is_empty():
		CurrentGame.mark_saved(SaveGame.AUTOSAVE_SLOT)
	router.enter_game(gs, errors)   # a refused autosave is shown once; the game still starts
	var none: Array[String] = []
	return none
