class_name SaveSlotsModel
extends RefCounted

# ==============================================================================
# Tactical RPG - Save / load slots model (UI1-L): one row per slot 0..3 from SaveGame.peek (0 = autosave, never
# written by a manual save). Load enters the game (warnings shown once, rule 6); save asks before overwriting and is
# refused during a battle or conversation (rule 5); delete always asks. Answers to a question land in `last_errors`.
# ==============================================================================

const BUSY: String = "You can't save during a battle or a conversation."
const NO_GAME: String = "There is no game to save."
const MANUAL_ONLY: String = "Manual saves use slots 1-3 (slot 0 is the autosave)."

var router: MenuRouter = null
var saving: bool = false            # true = the hub's Save screen, false = Load
var last_errors: Array[String] = [] # problems of the last action (also after a confirmed question)

func _init(p_router: MenuRouter, p_saving: bool = false) -> void:
	router = p_router
	saving = p_saving

func slots() -> Array[int]:
	var out: Array[int] = [SaveGame.AUTOSAVE_SLOT]
	out.append_array(SaveGame.MANUAL_SLOTS)
	return out

# [{slot, label, status: empty|ok|unreadable|newer version, world, location, party, saved_at, can_load, can_save, can_delete}]
func rows() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for slot: int in slots():
		var head: Dictionary = SaveGame.peek(slot)
		var exists: bool = DictUtil.to_bool(head["exists"])
		var ok: bool = DictUtil.to_bool(head["ok"])
		var status: String = "ok" if ok else (str(head["error"]) if exists else "empty")
		var party: PackedStringArray = PackedStringArray()
		for raw: Variant in DictUtil.array_of(head["party"]):
			var m: Dictionary = DictUtil.dict_of(raw)
			var unit: Dictionary = ContentCatalog.get_entry("units", str(m["base_unit"]))
			party.append("%s Lv %d" % [str(unit.get("name", m["base_unit"])), DictUtil.to_int(m["level"])])
		var world_id: String = str(head["world_id"])
		var at: int = DictUtil.to_int(head["saved_at"])
		out.append({
			"slot": slot, "label": "Autosave" if slot == SaveGame.AUTOSAVE_SLOT else "Slot %d" % slot, "status": status,
			"world": str(WorldCatalog.manifest(world_id).get("name", world_id)), "location": location_name(world_id, str(head["location_id"])),
			"party": ", ".join(party), "saved_at": Time.get_datetime_string_from_unix_time(at, true) if exists and at > 0 else "",
			"can_load": ok and not saving, "can_save": saving and SaveGame.MANUAL_SLOTS.has(slot), "can_delete": exists,
		})
	return out

static func location_name(world_id: String, location_id: String) -> String:
	if location_id.is_empty():
		return ""
	var hit: Dictionary = WorldCatalog.resolve(world_id, "locations", location_id) if WorldCatalog.has_world(world_id) else {}
	return str(hit.get("name", location_id))

# Reads `slot` and enters the game; errors keep the player on the slots screen (rule 6).
static func load_slot(p_router: MenuRouter, slot: int) -> Array[String]:
	var res: Dictionary = SaveGame.read(slot)
	var errors: Array[String] = []
	errors.assign(DictUtil.array_of(res["errors"]))
	var gs: GameState = res["state"] as GameState
	if not DictUtil.to_bool(res["ok"]) or gs == null:
		if errors.is_empty():
			errors.append("Slot %d can't be loaded." % slot)
		return errors
	var warnings: Array[String] = []
	warnings.assign(DictUtil.array_of(res["warnings"]))
	CurrentGame.start(gs, slot)
	CurrentGame.mark_saved(slot)
	p_router.enter_game(gs, warnings)
	return errors

func load(slot: int) -> Array[String]:
	last_errors = load_slot(router, slot)
	return last_errors

# Manual save to slot 1..3; asks first when the slot holds a save.
func save(slot: int) -> Array[String]:
	last_errors = []
	if not CurrentGame.has_game():
		last_errors.append(NO_GAME)
	elif CurrentGame.busy():
		last_errors.append(BUSY)
	elif not SaveGame.MANUAL_SLOTS.has(slot):
		last_errors.append(MANUAL_ONLY)
	elif SaveGame.exists(slot):
		router.ask("Overwrite slot %d?" % slot, _save_now.bind(slot))
	else:
		_save_now(slot)
	return last_errors

func _save_now(slot: int) -> void:
	last_errors = SaveGame.write(CurrentGame.state(), slot)
	if last_errors.is_empty():
		CurrentGame.mark_saved(slot)
		router.changed.emit()

func delete(slot: int) -> Array[String]:
	last_errors = []
	if not SaveGame.exists(slot):
		last_errors.append("Slot %d is empty." % slot)
	else:
		router.ask("Delete the save in slot %d?" % slot, _delete_now.bind(slot))
	return last_errors

func _delete_now(slot: int) -> void:
	last_errors = []
	if not SaveGame.delete(slot):
		last_errors.append("Slot %d could not be deleted." % slot)
	router.changed.emit()
