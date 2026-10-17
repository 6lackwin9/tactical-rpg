class_name GearModel
extends RefCounted

# ==============================================================================
# Tactical RPG - Gear model (UI2-L rule 6): a draft of one member's equipment (slot -> item id; unlisted = the class
# default) and of the party stash. equip() moves a stash item onto the member and the replaced explicit item back to the
# stash; unequip() returns the explicit item and falls back to the class default (the "none" item is not offered). Each
# equip is checked by UnitState.validate (gear messages verbatim); confirm() writes both drafts back.
# ==============================================================================

var member_id: String = ""
var work: UnitState = null
var stash: Dictionary = {}   # item id -> count (draft)

func _init(p_member: String) -> void:
	member_id = p_member
	cancel()

func _member() -> UnitState:
	return PartyPreview.member(member_id)

func cancel() -> void:
	var m: UnitState = _member()
	work = m.duplicate_state() if m != null else null
	stash = CurrentGame.state().party.stash.duplicate() if CurrentGame.has_game() else {}

func has_draft() -> bool:
	var m: UnitState = _member()
	return m != null and (not work.equals(m) or not JsonUtil.deep_equal(stash, CurrentGame.state().party.stash))

static func item_name(item_id: String) -> String:
	return str(ContentCatalog.get_entry("items", item_id).get("name", item_id)) if not item_id.is_empty() else ""

# [{slot, item, name, explicit}]: the draft's item per loadout slot; class-default items show with explicit false.
func slots() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var c: Combatant = PartyBuilder.build(work, false) if work != null else null
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		var explicit: bool = work != null and work.equipment.has(slot)
		var item_id: String = str(work.equipment[slot]) if explicit else ""
		if not explicit and c != null:
			var idx: int = Equipment.EquipSlot.keys().find(slot.to_upper())
			var eq: Equipment = c.inventory.get_equipped(idx as Equipment.EquipSlot) if idx >= 0 else null
			item_id = eq.id if eq != null else ""
		out.append({"slot": slot, "item": item_id, "name": item_name(item_id), "explicit": explicit})
	return out

# [{item, name, count, slots}] stash rows; `slots` lists the loadout slots the item fits.
func stash_rows() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var ids: Array = stash.keys()
	ids.sort()
	for raw: Variant in ids:
		var id: String = str(raw)
		var entry: Dictionary = ContentCatalog.get_entry("items", id)
		var fits: Array[String] = []
		for slot: String in ContentSchema.LOADOUT_SLOTS:
			if not entry.is_empty() and ContentFactory.item_fits_slot(entry, slot.to_upper(), true):
				fits.append(slot)
		out.append({"item": id, "name": item_name(id), "count": DictUtil.to_int(stash[raw]), "slots": fits})
	return out

func _gear_errors(state: UnitState) -> Array[String]:
	var errors: Array[String] = []
	for e: String in state.validate():
		if not e.contains("loadout"):
			errors.append(e)
	return errors

func equip(slot: String, item_id: String) -> Array[String]:
	var errors: Array[String] = []
	if not ContentSchema.LOADOUT_SLOTS.has(slot):
		errors.append("Unknown slot '%s'." % slot)
	elif DictUtil.to_int(stash.get(item_id, 0)) <= 0:
		errors.append("'%s' is not in the stash." % item_name(item_id))
	if not errors.is_empty():
		return errors
	var trial: UnitState = work.duplicate_state()
	trial.equipment[slot] = item_id
	errors = _gear_errors(trial)
	if not errors.is_empty():
		return errors
	if work.equipment.has(slot):
		_to_stash(str(work.equipment[slot]))
	work = trial
	_from_stash(item_id)
	return errors

func unequip(slot: String) -> Array[String]:
	var errors: Array[String] = []
	if not work.equipment.has(slot):
		errors.append("The %s slot already holds the class default." % slot.replace("_", " "))
		return errors
	_to_stash(str(work.equipment[slot]))
	work.equipment.erase(slot)
	return errors

func confirm() -> Array[String]:
	var errors: Array[String] = _gear_errors(work)
	if not errors.is_empty():
		return errors
	var m: UnitState = _member()
	m.equipment = work.equipment.duplicate()
	CurrentGame.state().party.stash = stash.duplicate()
	return errors

func _to_stash(item_id: String) -> void:
	stash[item_id] = DictUtil.to_int(stash.get(item_id, 0)) + 1

func _from_stash(item_id: String) -> void:
	var left: int = DictUtil.to_int(stash.get(item_id, 0)) - 1
	if left <= 0:
		stash.erase(item_id)
	else:
		stash[item_id] = left
