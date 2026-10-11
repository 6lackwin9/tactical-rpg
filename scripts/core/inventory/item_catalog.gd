class_name ItemCatalog
extends RefCounted

# ==============================================================================
# Tactical RPG - Preset Item & Equipment Catalog
# Provides pre-configured weapons, armor, accessories, and consumables
# for party stash management, testing, and initial hero loadouts.
# ==============================================================================

# ------------------------------------------------------------------------------
# Weapons
# ------------------------------------------------------------------------------

static func create_iron_claymore() -> Equipment:
	var tiers: Array[Dictionary] = [
		{"stat": "STR", "threshold": 18, "bonus_power": 4, "desc": "Sunder Guard: +4 Power"}
	]
	var eq := Equipment.new(
		"wpn_claymore",
		"Iron Claymore",
		Equipment.EquipSlot.MAIN_HAND,
		"sword",
		Item.ItemType.WEAPON,
		24,
		{"STR": 14},
		tiers,
		4.5,
		65,
		Item.ItemRarity.UNCOMMON
	)
	eq.base_range = 1
	eq.burden = 1
	eq.description = "A heavy double-edged greatsword. Gains +4 Power when wielded with 18+ STR."
	return eq

static func create_titan_warhammer() -> Equipment:
	var tiers: Array[Dictionary] = [
		{"stat": "STR", "threshold": 26, "bonus_power": 6, "desc": "Crushing Impact: +6 Power"}
	]
	var eq := Equipment.new(
		"wpn_warhammer",
		"Titan Warhammer",
		Equipment.EquipSlot.MAIN_HAND,
		"hammer",
		Item.ItemType.WEAPON,
		34,
		{"STR": 22},
		tiers,
		7.0,
		120,
		Item.ItemRarity.RARE
	)
	eq.base_range = 1
	eq.burden = 3
	eq.description = "Colossal warhammer requiring 22 STR. Inflicts severe deficit penalty if under-statted."
	return eq

static func create_oak_shortbow() -> Equipment:
	var tiers: Array[Dictionary] = [
		{"stat": "DEX", "threshold": 16, "bonus_power": 3, "desc": "True Flight: +3 Power"}
	]
	var eq := Equipment.new(
		"wpn_shortbow",
		"Oak Shortbow",
		Equipment.EquipSlot.MAIN_HAND,
		"bow",
		Item.ItemType.WEAPON,
		20,
		{"DEX": 13},
		tiers,
		2.0,
		55,
		Item.ItemRarity.UNCOMMON
	)
	eq.base_range = 3
	eq.burden = 0
	eq.description = "Supple curved bow capable of ranged volleys up to 3 tiles."
	return eq

static func create_steel_broadsword() -> Equipment:
	var eq := Equipment.new(
		"wpn_broadsword",
		"Steel Broadsword",
		Equipment.EquipSlot.MAIN_HAND,
		"sword",
		Item.ItemType.WEAPON,
		18,
		{"STR": 10},
		[],
		2.8,
		35,
		Item.ItemRarity.COMMON
	)
	eq.base_range = 1
	eq.burden = 0
	eq.description = "Reliable military broadsword with modest strength requirements."
	return eq

static func create_shadow_dagger() -> Equipment:
	var tiers: Array[Dictionary] = [
		{"stat": "DEX", "threshold": 15, "bonus_power": 4, "desc": "Puncture: +4 Power"}
	]
	var eq := Equipment.new(
		"wpn_dagger",
		"Shadow Dagger",
		Equipment.EquipSlot.MAIN_HAND,
		"dagger",
		Item.ItemType.WEAPON,
		16,
		{"DEX": 12},
		tiers,
		1.0,
		50,
		Item.ItemRarity.UNCOMMON
	)
	eq.base_range = 1
	eq.burden = 0
	eq.description = "Featherweight stiletto primed for rapid flanking strikes."
	return eq

static func create_runed_staff() -> Equipment:
	var tiers: Array[Dictionary] = [
		{"stat": "INT", "threshold": 18, "bonus_power": 5, "desc": "Arcane Surge: +5 Power"}
	]
	var eq := Equipment.new(
		"wpn_staff",
		"Runed Staff",
		Equipment.EquipSlot.MAIN_HAND,
		"staff",
		Item.ItemType.WEAPON,
		22,
		{"INT": 14},
		tiers,
		2.2,
		75,
		Item.ItemRarity.RARE
	)
	eq.base_range = 2
	eq.burden = 0
	eq.description = "Carved ash stave engraved with mystic glyphs. Scales with intelligence."
	return eq

# ------------------------------------------------------------------------------
# Shields & Armor
# ------------------------------------------------------------------------------

static func create_iron_kite_shield() -> Equipment:
	var eq := Equipment.new(
		"shd_kite",
		"Iron Kite Shield",
		Equipment.EquipSlot.OFF_HAND,
		"shield",
		Item.ItemType.SHIELD,
		0,
		{"STR": 12},
		[],
		4.0,
		45,
		Item.ItemRarity.COMMON
	)
	eq.mitigation = 8
	eq.burden = 2
	eq.description = "Sturdy teardrop shield granting +8 flat damage mitigation."
	return eq

static func create_buckler() -> Equipment:
	var eq := Equipment.new(
		"shd_buckler",
		"Duelist Buckler",
		Equipment.EquipSlot.OFF_HAND,
		"shield",
		Item.ItemType.SHIELD,
		0,
		{"DEX": 10},
		[],
		1.5,
		30,
		Item.ItemRarity.COMMON
	)
	eq.mitigation = 4
	eq.burden = 0
	eq.description = "Light metal fist-shield providing 4 mitigation without slowing down."
	return eq

static func create_steel_plate_cuirass() -> Equipment:
	var eq := Equipment.new(
		"arm_plate",
		"Steel Plate Cuirass",
		Equipment.EquipSlot.BODY,
		"plate",
		Item.ItemType.ARMOR,
		0,
		{"CONS": 12},
		[],
		8.5,
		80,
		Item.ItemRarity.UNCOMMON
	)
	eq.mitigation = 10
	eq.burden = 3
	eq.description = "Interlocking steel plates providing 10 mitigation with 3 burden."
	return eq

static func create_leather_brigandine() -> Equipment:
	var eq := Equipment.new(
		"arm_leather",
		"Leather Brigandine",
		Equipment.EquipSlot.BODY,
		"leather",
		Item.ItemType.ARMOR,
		0,
		{"DEX": 10},
		[],
		3.0,
		40,
		Item.ItemRarity.COMMON
	)
	eq.mitigation = 4
	eq.burden = 1
	eq.description = "Riveted hardened leather protecting vital organs while keeping mobility."
	return eq

# ------------------------------------------------------------------------------
# Accessories
# ------------------------------------------------------------------------------

static func create_ring_of_the_bear() -> Equipment:
	var eq := Equipment.new(
		"acc_bear_ring",
		"Ring of the Bear",
		Equipment.EquipSlot.ACCESSORY_1,
		"",
		Item.ItemType.ACCESSORY,
		0,
		{},
		[],
		0.2,
		70,
		Item.ItemRarity.RARE
	)
	eq.stat_bonuses = {"STR": 2, "CONS": 2}
	eq.description = "Chiseled bone band infusing the wearer with ursine fortitude (+2 STR, +2 CONS)."
	return eq

static func create_swiftness_amulet() -> Equipment:
	var eq := Equipment.new(
		"acc_swift_amulet",
		"Amulet of Swiftness",
		Equipment.EquipSlot.ACCESSORY_2,
		"",
		Item.ItemType.ACCESSORY,
		0,
		{},
		[],
		0.2,
		65,
		Item.ItemRarity.RARE
	)
	eq.stat_bonuses = {"DEX": 3}
	eq.description = "Windstone pendant enhancing timeline reflexes (+3 DEX)."
	return eq

# ------------------------------------------------------------------------------
# Consumables
# ------------------------------------------------------------------------------

static func create_healing_draught() -> Consumable:
	var con := Consumable.new(
		"con_heal_potion",
		"Healing Draught",
		Consumable.ConsumableType.HEALTH_POTION,
		25, # p_heal
		0,  # p_damage
		"", # p_buff_stat
		0.0,# p_buff_pct
		0,  # p_duration
		1,  # p_block_cost
		0.5,# p_weight
		15, # p_value
		Item.ItemRarity.COMMON
	)
	con.description = "Herbal restorative that instantly patches wounds for +25 HP."
	return con

static func create_focus_elixir() -> Consumable:
	var con := Consumable.new(
		"con_focus_elixir",
		"Elixir of Focus",
		Consumable.ConsumableType.FOCUS_INCENSE,
		0,  # p_heal
		0,  # p_damage
		"STR", # p_buff_stat
		0.5,# p_buff_pct
		2,  # p_duration
		1,  # p_block_cost
		0.4,# p_weight
		25, # p_value
		Item.ItemRarity.UNCOMMON
	)
	con.description = "Distilled ginseng elixir imparting +50% attack power for 2 rounds."
	return con

static func create_throwing_fire_bomb() -> Consumable:
	var con := Consumable.new(
		"con_fire_bomb",
		"Throwing Fire Bomb",
		Consumable.ConsumableType.THROWABLE_BOMB,
		0,  # p_heal
		20, # p_damage
		"", # p_buff_stat
		0.0,# p_buff_pct
		0,  # p_duration
		1,  # p_block_cost
		1.0,# p_weight
		30, # p_value
		Item.ItemRarity.UNCOMMON
	)
	con.description = "Clay vessel of volatile pitch exploding on impact for 20 damage."
	return con

# ------------------------------------------------------------------------------
# Inventory & Party Loadout Population Helpers
# ------------------------------------------------------------------------------

static func populate_party_starter_stash(inv: Inventory) -> void:
	if inv == null:
		return
	inv.add_item(create_iron_claymore())
	inv.add_item(create_titan_warhammer())
	inv.add_item(create_oak_shortbow())
	inv.add_item(create_shadow_dagger())
	inv.add_item(create_runed_staff())
	inv.add_item(create_iron_kite_shield())
	inv.add_item(create_buckler())
	inv.add_item(create_steel_plate_cuirass())
	inv.add_item(create_leather_brigandine())
	inv.add_item(create_ring_of_the_bear())
	inv.add_item(create_swiftness_amulet())
	inv.add_item(create_healing_draught())
	inv.add_item(create_focus_elixir())
	inv.add_item(create_throwing_fire_bomb())

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
