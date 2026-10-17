class_name PartyGearBridge
extends RefCounted

# ==============================================================================
# Tactical RPG - Party gear bridge (UI2-L rule 6): runs CharacterEquipmentModal on a GearModel draft. Builds the member's
# Combatant (PartyBuilder.build) and a stash Inventory from the draft stash, and maps each equipment_updated back to
# GearModel.equip / unequip so the validator stays the only rule. A refused change rebuilds the modal from the draft.
# ==============================================================================

var model: GearModel = null
var hero: Combatant = null
var stash_inventory: Inventory = null
var last_errors: Array[String] = []

func _init(p_model: GearModel) -> void:
	model = p_model
	rebuild()

func rebuild() -> void:
	hero = PartyBuilder.build(model.work, false) if model.work != null else null
	stash_inventory = Inventory.new()
	for row: Dictionary in model.stash_rows():
		var entry: Dictionary = ContentCatalog.get_entry("items", str(row["item"]))
		for _i: int in range(DictUtil.to_int(row["count"])):
			var item: Item = ContentFactory.item(entry)
			if item != null:
				stash_inventory.add_item(item)

func attach(modal: CharacterEquipmentModal) -> void:
	var allies: Array[Combatant] = [hero]
	modal.setup_heroes(allies, stash_inventory)
	if not modal.equipment_updated.is_connected(_on_updated):
		modal.equipment_updated.connect(_on_updated.bind(modal))

static func equipped_id(c: Combatant, slot: String) -> String:
	var idx: int = Equipment.EquipSlot.keys().find(slot.to_upper())
	var eq: Equipment = c.inventory.get_equipped(idx as Equipment.EquipSlot) if idx >= 0 and c != null else null
	return eq.id if eq != null else ""

# Replays the modal's change on the draft: a slot whose item differs is equipped from the stash or reverted to default.
func sync(c: Combatant) -> Array[String]:
	last_errors.clear()
	for row: Dictionary in model.slots():
		var slot: String = str(row["slot"])
		var now: String = equipped_id(c, slot)
		if now == str(row["item"]):
			continue
		var errors: Array[String] = model.unequip(slot) if now.is_empty() else model.equip(slot, now)
		last_errors.append_array(errors)
	return last_errors

func _on_updated(c: Combatant, modal: CharacterEquipmentModal) -> void:
	if not sync(c).is_empty():
		rebuild()
		attach(modal)
