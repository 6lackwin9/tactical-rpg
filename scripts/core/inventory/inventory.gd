class_name Inventory
extends RefCounted

# ==============================================================================
# Tactical RPG - Inventory & Equipment Loadout Model
# Manages party bag, equipped slots, and 3-slot combat battle pouches
# ==============================================================================

signal item_added(item: Item)
signal item_removed(item: Item)
signal equipment_changed(slot: Equipment.EquipSlot, item: Equipment)

var items: Array[Item] = []
var equipped_gear: Dictionary = {}  # Equipment.EquipSlot (int) -> Equipment
var battle_pouch: Array[Consumable] = []
var max_pouch_slots: int = 3

# Two-handed main weapons can be paired with an off-hand weapon, but only one of
# the two is "active" at a time; swapping costs a Switch Weapon action (1 block).
var active_weapon_slot: Equipment.EquipSlot = Equipment.EquipSlot.MAIN_HAND

func _init() -> void:
	equipped_gear.clear()
	battle_pouch.clear()

# ------------------------------------------------------------------------------
# Equipment Slot Management
# ------------------------------------------------------------------------------

func equip(equipment: Equipment, slot_override: int = -1) -> Equipment:
	if equipment == null:
		return null

	var target_slot: Equipment.EquipSlot = equipment.slot
	if slot_override >= 0 and slot_override in Equipment.EquipSlot.values():
		target_slot = slot_override as Equipment.EquipSlot

	var previous: Equipment = null
	if equipped_gear.has(target_slot):
		var prev_raw: Variant = equipped_gear[target_slot]
		if prev_raw is Equipment:
			previous = prev_raw
			items.append(previous)

	equipped_gear[target_slot] = equipment
	if items.has(equipment):
		items.erase(equipment)

	_normalize_active_weapon()
	equipment_changed.emit(target_slot, equipment)
	return previous

func unequip(slot: Equipment.EquipSlot) -> Equipment:
	if not equipped_gear.has(slot):
		return null

	var raw_eq: Variant = equipped_gear[slot]
	if raw_eq is Equipment:
		var eq: Equipment = raw_eq
		equipped_gear.erase(slot)
		items.append(eq)
		_normalize_active_weapon()
		equipment_changed.emit(slot, null)
		return eq
	return null

func get_equipped(slot: Equipment.EquipSlot) -> Equipment:
	if equipped_gear.has(slot):
		var raw_eq: Variant = equipped_gear[slot]
		if raw_eq is Equipment:
			return raw_eq
	return null

func get_main_weapon() -> Equipment:
	return get_equipped(Equipment.EquipSlot.MAIN_HAND)

func get_offhand_weapon() -> Equipment:
	var off_eq: Equipment = get_equipped(Equipment.EquipSlot.OFF_HAND)
	if off_eq != null and off_eq.item_type == Item.ItemType.WEAPON:
		return off_eq
	return null

# True when a 2H main weapon is paired with an off-hand weapon: only one is ready
# at a time and the other needs a Switch Weapon action. 1H pairs never need it.
func needs_weapon_switch() -> bool:
	var main_w: Equipment = get_main_weapon()
	return main_w != null and main_w.is_two_handed and get_offhand_weapon() != null

func get_active_weapon() -> Equipment:
	if needs_weapon_switch() and active_weapon_slot == Equipment.EquipSlot.OFF_HAND:
		return get_offhand_weapon()
	return get_main_weapon()

func is_offhand_active() -> bool:
	return needs_weapon_switch() and active_weapon_slot == Equipment.EquipSlot.OFF_HAND

# Swaps the ready weapon. Returns false when there is nothing to switch to.
func toggle_active_weapon() -> bool:
	if not needs_weapon_switch():
		active_weapon_slot = Equipment.EquipSlot.MAIN_HAND
		return false
	active_weapon_slot = Equipment.EquipSlot.MAIN_HAND if is_offhand_active() else Equipment.EquipSlot.OFF_HAND
	return true

func _normalize_active_weapon() -> void:
	if not needs_weapon_switch():
		active_weapon_slot = Equipment.EquipSlot.MAIN_HAND

func get_shield() -> Equipment:
	var off_eq: Equipment = get_equipped(Equipment.EquipSlot.OFF_HAND)
	if off_eq != null and (off_eq.item_type == Item.ItemType.SHIELD or off_eq.off_hand_category == "shield"):
		return off_eq
	return null

# Returns every equipped piece as a typed array — single safe cast point.
func _iter_equipped_gear() -> Array[Equipment]:
	var result: Array[Equipment] = []
	for slot_key: Variant in equipped_gear:
		var raw_eq: Variant = equipped_gear[slot_key]
		if raw_eq is Equipment:
			var eq: Equipment = raw_eq
			result.append(eq)
	return result

func get_total_armor_burden() -> int:
	var total: int = 0
	for eq: Equipment in _iter_equipped_gear():
		total += eq.burden
	return total

func get_total_shield_mitigation() -> int:
	var off: Equipment = get_equipped(Equipment.EquipSlot.OFF_HAND)
	return off.mitigation if off != null else 0

func get_total_armor_mitigation() -> int:
	var body: Equipment = get_equipped(Equipment.EquipSlot.BODY)
	return body.mitigation if body != null else 0

func get_total_flat_mitigation() -> int:
	var total: int = 0
	for eq: Equipment in _iter_equipped_gear():
		total += eq.mitigation
	return total

func get_total_gear_stat_bonus(stat_name: String) -> int:
	var total: int = 0
	var clean: String = stat_name.to_upper()
	for eq: Equipment in _iter_equipped_gear():
		if eq.stat_bonuses.has(clean):
			var b: Variant = eq.stat_bonuses[clean]
			if b is int:
				total += b
	return total

# ------------------------------------------------------------------------------
# Item Stash Management
# ------------------------------------------------------------------------------

func add_item(item: Item) -> bool:
	if item == null:
		return false
	items.append(item)
	item_added.emit(item)
	return true

func remove_item(item: Item) -> bool:
	if items.has(item):
		items.erase(item)
		item_removed.emit(item)
		return true
	return false

func get_total_weight() -> float:
	var total: float = 0.0
	for itm: Item in items:
		total += itm.weight
	for slot_key: Variant in equipped_gear:
		var raw_eq: Variant = equipped_gear[slot_key]
		if raw_eq is Equipment:
			total += raw_eq.weight
	for c_itm: Consumable in battle_pouch:
		total += c_itm.weight
	return total

# ------------------------------------------------------------------------------
# Combat Battle Pouch (Quick slots for ItemCommand in combat)
# ------------------------------------------------------------------------------

func assign_pouch_item(index: int, consumable: Consumable) -> bool:
	if index < 0 or index >= max_pouch_slots or consumable == null:
		return false

	while battle_pouch.size() <= index:
		battle_pouch.append(null)

	battle_pouch[index] = consumable
	if items.has(consumable):
		items.erase(consumable)
	return true

func use_pouch_item(index: int, target: Combatant) -> Dictionary:
	if index < 0 or index >= battle_pouch.size():
		return {}

	var item: Consumable = battle_pouch[index]
	if item == null:
		return {}

	var outcome: Dictionary = item.apply_to_combatant(target)
	battle_pouch.remove_at(index)
	return outcome

func add_to_battle_pouch(consumable: Consumable) -> bool:
	if consumable == null or battle_pouch.size() >= max_pouch_slots:
		return false
	battle_pouch.append(consumable)
	if items.has(consumable):
		items.erase(consumable)
	return true

func remove_from_battle_pouch(consumable: Consumable) -> bool:
	if battle_pouch.has(consumable):
		battle_pouch.erase(consumable)
		return true
	return false
