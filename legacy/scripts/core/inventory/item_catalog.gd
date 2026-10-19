class_name ItemCatalog
extends RefCounted

# ==============================================================================
# Tactical RPG - Item & Equipment Catalog
# Items are authored in the content catalog (res://data/catalog/items.json, edited
# in the Content Forge); create(id) builds a fresh instance. The create_* helpers
# below are shorthands for the items the code, dojo and tests use by name.
# ==============================================================================

# A new instance of catalog item `id` (null if unknown).
static func create(id: String) -> Item:
	if not ContentCatalog.has("items", id):
		push_error("ItemCatalog: unknown item '%s'" % id)
		return null
	return ContentFactory.item(ContentCatalog.get_entry("items", id))

static func create_equipment(id: String) -> Equipment:
	var it: Item = create(id)
	return it as Equipment

static func create_consumable(id: String) -> Consumable:
	var it: Item = create(id)
	return it as Consumable

# ------------------------------------------------------------------------------
# Named shorthands
# ------------------------------------------------------------------------------

static func create_iron_claymore() -> Equipment:
	return create_equipment("wpn_claymore")

static func create_titan_warhammer() -> Equipment:
	return create_equipment("wpn_warhammer")

static func create_oak_shortbow() -> Equipment:
	return create_equipment("wpn_shortbow")

static func create_steel_broadsword() -> Equipment:
	return create_equipment("wpn_broadsword")

static func create_iron_shortsword() -> Equipment:
	return create_equipment("wpn_shortsword")

static func create_shadow_dagger() -> Equipment:
	return create_equipment("wpn_dagger")

static func create_mystic_tome() -> Equipment:
	return create_equipment("wpn_tome")

static func create_wand() -> Equipment:
	return create_equipment("wpn_wand")

static func create_runed_staff() -> Equipment:
	return create_equipment("wpn_staff")

static func create_iron_kite_shield() -> Equipment:
	return create_equipment("shd_kite")

static func create_buckler() -> Equipment:
	return create_equipment("shd_buckler")

static func create_steel_plate_cuirass() -> Equipment:
	return create_equipment("arm_plate")

static func create_leather_brigandine() -> Equipment:
	return create_equipment("arm_leather")

static func create_ring_of_the_bear() -> Equipment:
	return create_equipment("acc_bear_ring")

static func create_swiftness_amulet() -> Equipment:
	return create_equipment("acc_swift_amulet")

static func create_healing_draught() -> Consumable:
	return create_consumable("con_heal_potion")

static func create_focus_elixir() -> Consumable:
	return create_consumable("con_focus_elixir")

static func create_throwing_fire_bomb() -> Consumable:
	return create_consumable("con_fire_bomb")

# ------------------------------------------------------------------------------
# Inventory & Party Loadout Population Helpers
# ------------------------------------------------------------------------------

# Adds every catalog item flagged "starter_stash", in catalog order.
static func populate_party_starter_stash(inv: Inventory) -> void:
	if inv == null:
		return
	for entry: Dictionary in ContentCatalog.entries("items"):
		if DictUtil.to_bool(entry.get("starter_stash", false)):
			inv.add_item(ContentFactory.item(entry))

static func setup_default_hero_loadout(hero: Combatant, archetype_name: String) -> void:
	if hero == null or hero.inventory == null:
		return

	if archetype_name == "Warrior":
		hero.inventory.equip(create_steel_broadsword())
		hero.inventory.equip(create_buckler(), Equipment.EquipSlot.OFF_HAND)
		hero.inventory.equip(create_leather_brigandine(), Equipment.EquipSlot.BODY)
		hero.inventory.equip(create_ring_of_the_bear(), Equipment.EquipSlot.ACCESSORY_1)
		hero.inventory.add_to_battle_pouch(create_healing_draught())
		hero.inventory.add_to_battle_pouch(create_throwing_fire_bomb())
	elif archetype_name == "Ranger":
		hero.inventory.equip(create_oak_shortbow())
		hero.inventory.equip(create_leather_brigandine(), Equipment.EquipSlot.BODY)
		hero.inventory.equip(create_swiftness_amulet(), Equipment.EquipSlot.ACCESSORY_2)
		hero.inventory.add_to_battle_pouch(create_focus_elixir())
		hero.inventory.add_to_battle_pouch(create_healing_draught())

	hero.sync_from_stats_and_gear()
