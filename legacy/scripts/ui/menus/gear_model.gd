class_name GearModel
extends RefCounted

# ==============================================================================
# Tactical RPG - Gear model (UI2-L rule 6): a draft of one member's equipment (slot -> item id; unlisted = the class
# default) and of the party stash. equip() moves a stash item onto the member and the replaced explicit item back to the
# stash; unequip() returns the explicit item and falls back to the class default (the "none" item is not offered). Each
# equip is checked by UnitState.validate (gear messages verbatim); confirm() writes both drafts back.
# ITEMUSE1 rule 1: the battle pouch drafts the same way: pouch_add() moves one consumable from the stash into the next
# pouch slot, pouch_remove() returns a slot's item to the stash; confirm() writes the pouch with the gear.
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
	errors.assign(DictUtil.array_of(state.validate_split()["other"]))
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

# [{index, item, name}]: the draft pouch in slot order.
func pouch_rows() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if work == null:
		return out
	for i: int in range(work.pouch.size()):
		out.append({"index": i, "item": work.pouch[i], "name": item_name(work.pouch[i])})
	return out

# [{item, name, count}]: stash consumables that may go into the pouch (types offered in battle).
func pouch_candidates() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for row: Dictionary in stash_rows():
		if _pouch_problem(str(row["item"])).is_empty():
			out.append({"item": row["item"], "name": row["name"], "count": row["count"]})
	return out

func _pouch_problem(item_id: String) -> String:
	var entry: Dictionary = ContentCatalog.get_entry("items", item_id)
	if str(entry.get("item_type", "")) != "CONSUMABLE":
		return "'%s' is not a consumable." % item_name(item_id)
	var built: Item = ContentFactory.item(entry)
	if not (built is Consumable) or not ItemCommand.offered(built as Consumable):
		return "%s can't be used in battle." % item_name(item_id)
	return ""

func pouch_add(item_id: String) -> Array[String]:
	var errors: Array[String] = []
	if work == null:
		errors.append("No member.")
		return errors
	if DictUtil.to_int(stash.get(item_id, 0)) <= 0:
		errors.append("'%s' is not in the stash." % item_name(item_id))
		return errors
	var problem: String = _pouch_problem(item_id)
	if not problem.is_empty():
		errors.append(problem)
		return errors
	if work.pouch.size() >= UnitState.POUCH_MAX:
		errors.append("The pouch holds at most %d items." % UnitState.POUCH_MAX)
		return errors
	work.pouch.append(item_id)
	_from_stash(item_id)
	return errors

func pouch_remove(index: int) -> Array[String]:
	var errors: Array[String] = []
	if work == null or index < 0 or index >= work.pouch.size():
		errors.append("No item in pouch slot %d." % (index + 1))
		return errors
	_to_stash(work.pouch[index])
	work.pouch.remove_at(index)
	return errors

func confirm() -> Array[String]:
	var errors: Array[String] = _gear_errors(work)
	if not errors.is_empty():
		return errors
	var m: UnitState = _member()
	m.equipment = work.equipment.duplicate()
	m.pouch = work.pouch.duplicate()
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
