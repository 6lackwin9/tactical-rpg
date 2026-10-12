class_name DojoLoadouts
extends RefCounted

# ==============================================================================
# Tactical RPG - Dojo Quick-Setup Loadout Presets
# Declarative gear presets. Pieces are taken from the shared stash when present
# and spawned from ItemCatalog otherwise, so any preset can always be applied.
# Class proficiency rules still apply: ineligible pieces are skipped and reported.
# ==============================================================================

static func get_presets() -> Array[Dictionary]:
	return [
		{
			"id": "sword_board",
			"name": "Sword & Board",
			"desc": "1H Broadsword, Buckler, Brigandine, Ring of the Bear",
			"gear": [
				{"slot": Equipment.EquipSlot.MAIN_HAND, "make": ItemCatalog.create_steel_broadsword},
				{"slot": Equipment.EquipSlot.OFF_HAND, "make": ItemCatalog.create_buckler},
				{"slot": Equipment.EquipSlot.BODY, "make": ItemCatalog.create_leather_brigandine},
				{"slot": Equipment.EquipSlot.ACCESSORY_1, "make": ItemCatalog.create_ring_of_the_bear},
			],
			"pouch": [ItemCatalog.create_healing_draught, ItemCatalog.create_throwing_fire_bomb],
		},
		{
			"id": "claymore_tank",
			"name": "Claymore Bruiser",
			"desc": "2H Iron Claymore, Plate Cuirass, Ring of the Bear (Tier 1 test)",
			"gear": [
				{"slot": Equipment.EquipSlot.MAIN_HAND, "make": ItemCatalog.create_iron_claymore},
				{"slot": Equipment.EquipSlot.BODY, "make": ItemCatalog.create_steel_plate_cuirass},
				{"slot": Equipment.EquipSlot.ACCESSORY_1, "make": ItemCatalog.create_ring_of_the_bear},
			],
			"pouch": [ItemCatalog.create_healing_draught, ItemCatalog.create_focus_elixir],
		},
		{
			"id": "claymore_backup",
			"name": "Claymore + Backup Blade",
			"desc": "2H Iron Claymore with an Iron Shortsword off-hand: Switch Weapon (1 block) to use either",
			"gear": [
				{"slot": Equipment.EquipSlot.MAIN_HAND, "make": ItemCatalog.create_iron_claymore},
				{"slot": Equipment.EquipSlot.OFF_HAND, "make": ItemCatalog.create_iron_shortsword},
				{"slot": Equipment.EquipSlot.BODY, "make": ItemCatalog.create_leather_brigandine},
				{"slot": Equipment.EquipSlot.ACCESSORY_1, "make": ItemCatalog.create_ring_of_the_bear},
			],
			"pouch": [ItemCatalog.create_healing_draught],
		},
		{
			"id": "titan_deficit",
			"name": "Titan Hammer (Deficit)",
			"desc": "2H Titan Warhammer with Plate - stresses deficit & burden penalties",
			"gear": [
				{"slot": Equipment.EquipSlot.MAIN_HAND, "make": ItemCatalog.create_titan_warhammer},
				{"slot": Equipment.EquipSlot.BODY, "make": ItemCatalog.create_steel_plate_cuirass},
			],
			"pouch": [],
		},
		{
			"id": "dual_wield",
			"name": "Dual Wield",
			"desc": "Broadsword + Iron Shortsword off-hand, Brigandine, Ring of the Bear",
			"gear": [
				{"slot": Equipment.EquipSlot.MAIN_HAND, "make": ItemCatalog.create_steel_broadsword},
				{"slot": Equipment.EquipSlot.OFF_HAND, "make": ItemCatalog.create_iron_shortsword},
				{"slot": Equipment.EquipSlot.BODY, "make": ItemCatalog.create_leather_brigandine},
				{"slot": Equipment.EquipSlot.ACCESSORY_1, "make": ItemCatalog.create_ring_of_the_bear},
			],
			"pouch": [ItemCatalog.create_healing_draught],
		},
		{
			"id": "archer",
			"name": "Archer",
			"desc": "Oak Shortbow, Brigandine, Swiftness Amulet",
			"gear": [
				{"slot": Equipment.EquipSlot.MAIN_HAND, "make": ItemCatalog.create_oak_shortbow},
				{"slot": Equipment.EquipSlot.BODY, "make": ItemCatalog.create_leather_brigandine},
				{"slot": Equipment.EquipSlot.ACCESSORY_2, "make": ItemCatalog.create_swiftness_amulet},
			],
			"pouch": [ItemCatalog.create_focus_elixir, ItemCatalog.create_healing_draught],
		},
		{
			"id": "duelist",
			"name": "Dagger Duelist",
			"desc": "Shadow Dagger main hand, Brigandine, Swiftness Amulet (DEX scaling)",
			"gear": [
				{"slot": Equipment.EquipSlot.MAIN_HAND, "make": ItemCatalog.create_shadow_dagger},
				{"slot": Equipment.EquipSlot.BODY, "make": ItemCatalog.create_leather_brigandine},
				{"slot": Equipment.EquipSlot.ACCESSORY_2, "make": ItemCatalog.create_swiftness_amulet},
			],
			"pouch": [ItemCatalog.create_healing_draught],
		},
		{
			"id": "arcanist",
			"name": "Arcanist",
			"desc": "Runed Staff with Mystic Tome off-hand (INT scaling)",
			"gear": [
				{"slot": Equipment.EquipSlot.MAIN_HAND, "make": ItemCatalog.create_runed_staff},
				{"slot": Equipment.EquipSlot.OFF_HAND, "make": ItemCatalog.create_mystic_tome},
			],
			"pouch": [ItemCatalog.create_focus_elixir],
		},
	]

static func find_preset(preset_id: String) -> Dictionary:
	for preset: Dictionary in get_presets():
		if str(preset.get("id", "")) == preset_id:
			return preset
	return {}
