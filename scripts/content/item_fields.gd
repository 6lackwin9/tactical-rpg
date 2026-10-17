class_name ItemFields
extends RefCounted

# The items field table of ContentSchema (moved here by CLN-9b; ContentSchema.fields("items") / field() are the API).

static func _f(key: String, label: String, type: String, default: Variant, extra: Dictionary = {}) -> Dictionary:
	return ContentSchema._f(key, label, type, default, extra)

# The item_families field table (FORGE1_ITEMS 1.1). `off_hand_category` is derived (ItemFamilies.derive).
static func family_build() -> Array[Dictionary]:
	var handed: Dictionary = {"key": "item_type", "in": ItemFamilies.HAND_TYPES}
	var weapon: Dictionary = {"key": "item_type", "in": ["WEAPON"]}
	var slotted: Dictionary = {"key": "item_type", "in": ItemFamilies.SLOT_TYPES}
	return [
		_f("id", "ID", "id", "", {"key_field": true, "help": "A gear family (WEAPON, SHIELD, ARMOR) is named after its gear token (Proficiency.gear_types())."}),
		_f("name", "Name", "string", "New family", {"max_len": 32}),
		_f("description", "Description", "text", "", {"max_len": 240}),
		_f("item_type", "Item type", "enum", "WEAPON", {"options": ItemFamilies.ITEM_TYPES, "key_field": true}),
		_f("hands", "Hands", "multi_pick", [ItemFamilies.HAND_MAIN_1H], {"options": ItemFamilies.HANDS, "when": handed, "key_field": true, "help": "One category: main hands (main_1h, main_2h; damage weapons) or exactly off (support items)."}),
		_f("default_hand", "Default hand", "enum", ItemFamilies.HAND_MAIN_1H, {"options": ["", ItemFamilies.HAND_MAIN_1H, ItemFamilies.HAND_MAIN_2H, ItemFamilies.HAND_OFF], "when": handed}),
		_f("off_hand_category", "Off-hand category", "enum", "", {"options": ["", "shield", "orb", "tome"], "readonly": true, "key_field": true, "help": "Derived: the family id of an off-hand family (shield, orb, tome)."}),
		_f("slots", "Slots", "multi_pick", [], {"options": ItemFamilies.ARMOR_SLOTS + ItemFamilies.ACCESSORY_SLOTS, "when": slotted, "help": "Armor: HEAD, BODY; accessories: ACCESSORY_1, ACCESSORY_2. Weapons and shields derive the slot from the hand."}),
		_f("scaling_stats", "Scaling stats", "multi_pick", ["STR"], {"options": ItemFamilies.SCALING_STATS, "when": weapon, "help": "The first is the default of new items."}),
		_f("base", "Base numbers", "hand_grid", {}, {"grid": "base", "help": "Per hand (\"none\" without hands): the five numbers every item of the hand starts from (power, range, block cost, burden, mitigation)."}),
		_f("bands", "Soft bands", "hand_grid", {}, {"grid": "bands", "help": "Per hand, per number {min, max}, plus bonus_value {min, max} (GearBudget value of stat bonuses and effects). Outside = warning."}),
		_f("adjustable", "Adjustable numbers", "multi_pick", ItemFamilies.NUMBER_FIELDS.duplicate(), {"options": ItemFamilies.NUMBER_FIELDS, "key_field": true}),
		_f("modifier_kinds", "Modifier kinds", "multi_pick", ItemModifiers.kind_names(), {"options": ItemModifiers.kind_names(), "key_field": true}),
		_f("max_modifiers", "Max modifiers", "int", 8, {"min": 0, "max": 12}),
		_f("max_tiers", "Max tiers", "int", 3, {"min": 0, "max": 4}),
		_f("engine_rules", "Engine rules", "multi_pick", [], {"options": ItemFamilies.ENGINE_RULES, "key_field": true, "help": "Closed list (code-owned): focus, always_ranged, shield, spell_source."}),
		_f("damage_type", "Damage type", "enum", "", {"options": ["", "slash", "pierce", "blunt", "magic", "fire"], "help": "Default of the family's items."}),
		_f("art", "Art", "string", "", {"help": "Default icon key (ItemArt); an item's icon_id overrides it."}),
	]

static func build() -> Array[Dictionary]:
	var equip: Dictionary = {"key": "item_type", "in": ContentSchema.EQUIP_TYPES}
	var weapon: Dictionary = {"key": "item_type", "in": ["WEAPON"]}
	var consumable: Dictionary = {"key": "item_type", "in": ["CONSUMABLE"]}
	var handed: Dictionary = {"key": "item_type", "in": ItemFamilies.HAND_TYPES}
	# FORGE1_ITEMS 2: `family`, `hand`, `scaling_stat`, `slot` (armor, accessory), `modifiers` and `tiers` are STORED
	# (items.json version 2); item_type, weapon_type, weapon slot, two_handed, off_hand_category and the numbers are
	# derived from the family (ItemModifiers.resolve). Key fields (`key_field`) lock on system items (FORGE_SHARED 12.2).
	return [
		_f("id", "ID", "id", "", {"key_field": true}),
		_f("name", "Name", "string", "New Item", {"max_len": 32}),
		_f("item_type", "Type", "enum", "WEAPON", {"options": ItemFamilies.ITEM_TYPES, "help": "Derived from the family (version 2 items)."}),
		_f("family", "Family", "ref", "", {"ref": "item_families", "key_field": true, "help": "The item family (template): type, hands, slots, base numbers, modifier policy."}),
		_f("rarity", "Rarity", "enum", "COMMON", {"options": ["COMMON", "UNCOMMON", "RARE", "EPIC", "LEGENDARY"]}),
		_f("weight", "Weight", "float", 1.0, {"min": 0.0, "max": 50.0, "step": 0.1}),
		_f("value", "Value (gold)", "int", 10, {"min": 0, "max": 99999}),
		_f("description", "Description", "text", ""),
		_f("damage_type", "Damage type", "enum", "", {"options": ["", "slash", "pierce", "blunt", "magic", "fire"]}),
		_f("icon_id", "Icon id", "string", "", {"help": "Optional icon key (any item art key: bespoke, type or manifest); empty = icon by gear type."}),
		_f("starter_stash", "In starter stash", "bool", false, {"help": "Added to the party stash of every encounter and the Gear Dojo."}),
		_f("special", "Special item", "bool", false, {"when": equip, "help": "May exceed the class gear budget; needs a special note."}),
		_f("special_note", "Special note", "text", "", {"when": equip, "help": "The trade-off or playstyle that justifies a special item."}),
		_f("slot", "Slot", "enum", "MAIN_HAND", {"options": ["MAIN_HAND", "OFF_HAND", "HEAD", "BODY", "ACCESSORY_1", "ACCESSORY_2"], "when": equip, "key_field": true}),
		_f("hand", "Hand", "enum", "", {"options": ["", ItemFamilies.HAND_MAIN_1H, ItemFamilies.HAND_MAIN_2H, ItemFamilies.HAND_OFF], "when": handed, "key_field": true, "help": "Exactly one hand, one of the family's hands; the slot follows it."}),
		_f("weapon_type", "Gear type", "enum", "", {"options": ["", "sword", "axe", "mace", "hammer", "fist", "dagger", "bow", "crossbow", "staff", "wand", "orb", "tome", "shield", "plate", "leather", "cloth"], "when": equip, "help": "Checked against class proficiencies and weapon masteries."}),
		_f("base_power", "Power", "int", 0, {"min": 0, "max": 200, "when": weapon}),
		_f("base_range", "Range", "int", 1, {"min": 1, "max": 6, "when": weapon, "help": "2+ makes the wielder a ranged unit."}),
		_f("two_handed", "Two-handed", "bool", false, {"when": weapon}),
		# The legacy options (shortsword, knuckles, dagger, knife) stay until ITEM1-A migrates items.json to version 2;
		# version 2 rows derive "", shield, orb or tome from the family.
		_f("off_hand_category", "Off-hand category", "enum", "", {"options": ["", "shield", "shortsword", "knuckles", "dagger", "knife", "orb", "tome"], "when": equip, "help": "Set for off-hand-only weapons and shields."}),
		_f("scaling_stat", "Scales with", "enum", "STR", {"options": ItemFamilies.SCALING_STATS, "when": weapon, "key_field": true}),
		_f("block_cost", "Block cost", "int", 1, {"min": 1, "max": 6, "when": equip}),
		_f("burden", "Burden", "int", 0, {"min": 0, "max": 10, "when": equip, "help": "Initiative penalty while worn."}),
		_f("mitigation", "Mitigation", "int", 0, {"min": 0, "max": 100, "when": equip}),
		_f("required_stats", "Requires", "stats", {}, {"keys": ContentSchema.STAT_KEYS, "when": equip}),
		_f("stat_bonuses", "Stat bonuses", "stats", {}, {"keys": ContentSchema.STAT_KEYS, "when": equip}),
		_f("effects", "Effects", "json", [], {"when": equip, "help": "Applies while equipped. Allowed: skill_potency (family or group spell/heal), status_potency (1-25), spell_reach (1-2), resist. [{\"type\": \"skill_potency\", \"group\": \"heal\", \"value\": 0.1}]"}),
		# Derived from `tiers` for every gear type (weapons, shields, armor, accessories); version 2 files never store it.
		_f("threshold_tiers", "Threshold tiers", "json", [], {"when": equip, "readonly": true, "help": "Derived from the tier thresholds: [{\"stat\", \"threshold\", \"bonus_power\", \"desc\"}] (+ when / bonus for extended tiers)."}),
		_f("modifiers", "Modifiers", "modifier_table", [], {"when": equip, "key_field": true, "help": "Ordered list of {kind, ...}: adjust {field, delta}, stat_bonus {stat, value}, requirement {stat, value}, effect {effect}."}),
		_f("tiers", "Tier thresholds", "tier_table", [], {"when": equip, "key_field": true, "help": "Up to the family's max tiers: {label, when {stat_at_least | proficiency_at_least | unit_level_at_least}, bonus {power | mitigation | stat_bonuses | effects}, desc}."}),
		_f("consumable_type", "Consumable type", "enum", "HEALTH_POTION", {"options": ["HEALTH_POTION", "STAMINA_BREW", "ANTIDOTE", "THROWABLE_BOMB", "FOCUS_INCENSE"], "when": consumable}),
		_f("heal", "Heal", "int", 0, {"min": 0, "max": 999, "when": consumable}),
		_f("damage", "Damage", "int", 0, {"min": 0, "max": 999, "when": consumable}),
		_f("buff_stat", "Buff stat", "enum", "", {"options": ["", "STR", "DEX", "INT", "CONS", "PERC", "KNOW"], "when": consumable}),
		_f("buff_pct", "Buff %", "float", 0.0, {"min": 0.0, "max": 5.0, "step": 0.05, "when": consumable}),
		_f("duration", "Buff rounds", "int", 0, {"min": 0, "max": 10, "when": consumable}),
		_f("use_blocks", "Use blocks", "int", 1, {"min": 1, "max": 6, "when": consumable}),
		_f("max_stack", "Max stack", "int", 1, {"min": 1, "max": 999, "when": {"key": "item_type", "in": ["CONSUMABLE", "MATERIAL", "QUEST"]}}),
	]
