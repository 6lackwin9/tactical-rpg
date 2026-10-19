class_name NewGameModel
extends RefCounted

# ==============================================================================
# Tactical RPG - New game model (UI1-L): the playable worlds with their fixed start party; choose() creates the game
# (NewGame.create_for_world), makes it current, writes the autosave and enters the game. UIR-2: the world cards are
# selectable (`selected`, the first playable world by default) and one Start (start()) begins the selected world.
# ==============================================================================

const NONE_SELECTED: String = "Choose a world first."

var router: MenuRouter = null
var last_errors: Array[String] = []   # errors of the last asked start (the host shows them)
var selected: String = ""             # the world card picked (UIR-2)

func _init(p_router: MenuRouter) -> void:
	router = p_router
	var playable: Array[String] = NewGame.playable_worlds()
	selected = playable[0] if not playable.is_empty() else ""

func select(world_id: String) -> Array[String]:
	var errors: Array[String] = []
	if not NewGame.playable_worlds().has(world_id):
		errors.append("World '%s' is not playable." % world_id)
	else:
		selected = world_id
	return errors

# Start: the selected world (choose asks first over a readable autosave).
func start(p_seed: int = -1) -> Array[String]:
	if selected.is_empty():
		var none: Array[String] = [NONE_SELECTED]
		return none
	return choose(selected, p_seed)

# "Party, all Lv 1: Aric (Warrior), Bryn (Mage)" when every member shares a level, else the level per member (UIR-2:
# no repeated "Lv" on every member). A class equal to the name is left out.
static func party_line(party: Array) -> String:
	var names: PackedStringArray = PackedStringArray()
	var levels: Dictionary = {}
	for raw: Variant in party:
		levels[DictUtil.to_int(DictUtil.dict_of(raw).get("level", 1), 1)] = true
	var shared: bool = levels.size() == 1
	for raw: Variant in party:
		var p: Dictionary = DictUtil.dict_of(raw)
		var cls: String = str(p.get("class", ""))
		var bits: PackedStringArray = PackedStringArray()
		if not cls.is_empty() and cls != str(p.get("name", "")):
			bits.append(cls)
		if not shared:
			bits.append("Lv %d" % DictUtil.to_int(p.get("level", 1), 1))
		names.append(str(p.get("name", "")) + (" (%s)" % ", ".join(bits) if not bits.is_empty() else ""))
	if names.is_empty():
		return "Party: none"
	return ("Party, all Lv %d: " % DictUtil.to_int(levels.keys()[0], 1) if shared else "Party: ") + ", ".join(names)

# [{id, name, tagline, party: [{unit, name, class, level}]}] for every playable world.
func worlds() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for w: String in NewGame.playable_worlds():
		var m: Dictionary = WorldCatalog.manifest(w)
		var party: Array[Dictionary] = []
		for unit_id: String in WorldCatalog.start_party(w):
			var base_id: String = unit_id
			var shown: String = ""
			if not ContentCatalog.has("units", unit_id):   # a world party character (RECRUIT1-L rule 2: its name)
				var hero: UnitState = NewGame.hero_from_character(w, unit_id)
				if hero != null:
					base_id = hero.base_unit
					shown = PartyBuilder.display_name(hero)
			var e: Dictionary = ContentCatalog.get_entry("units", base_id)
			var cls: Dictionary = ContentCatalog.get_entry("classes", str(e.get("class_id", "")))
			party.append({"unit": unit_id, "name": shown if not shown.is_empty() else str(e.get("name", unit_id)), "class": str(cls.get("name", e.get("class_id", ""))), "level": DictUtil.to_int(e.get("level", 1), 1)})
		out.append({"id": w, "name": str(m.get("name", w)), "tagline": str(m.get("tagline", "")), "party": party})
	return out

# Q1: starting over replaces the autosave, so a readable one is asked about first.
func choose(world_id: String, p_seed: int = -1) -> Array[String]:
	var head: Dictionary = SaveGame.peek(SaveGame.AUTOSAVE_SLOT)
	if DictUtil.to_bool(head["ok"]) and NewGame.playable_worlds().has(world_id):
		var world: String = str(head["world_id"])
		var at: int = DictUtil.to_int(head["saved_at"])
		router.ask("Starting a new game replaces your autosave (%s, %s, %s). Continue?" % [str(WorldCatalog.manifest(world).get("name", world)),
			SaveSlotsModel.location_name(world, str(head["location_id"])), Time.get_datetime_string_from_unix_time(at, true) if at > 0 else "unknown time"], _start.bind(world_id, p_seed), true)   # UIR-2: destructive, No focused
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
