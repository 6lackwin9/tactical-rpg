class_name ItemFields
extends RefCounted

# The items field table of ContentSchema (moved here by CLN-9b; ContentSchema.fields("items") / field() are the API).

static func _f(key: String, label: String, type: String, default: Variant, extra: Dictionary = {}) -> Dictionary:
	return ContentSchema._f(key, label, type, default, extra)

static func build() -> Array[Dictionary]:
	var equip: Dictionary = {"key": "item_type", "in": ContentSchema.EQUIP_TYPES}
	var weapon: Dictionary = {"key": "item_type", "in": ["WEAPON"]}
	var consumable: Dictionary = {"key": "item_type", "in": ["CONSUMABLE"]}
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Item"),
		_f("item_type", "Type", "enum", "WEAPON", {"options": ["WEAPON", "SHIELD", "ARMOR", "ACCESSORY", "CONSUMABLE", "MATERIAL", "QUEST"]}),
		_f("rarity", "Rarity", "enum", "COMMON", {"options": ["COMMON", "UNCOMMON", "RARE", "EPIC", "LEGENDARY"]}),
		_f("weight", "Weight", "float", 1.0, {"min": 0.0, "max": 50.0, "step": 0.1}),
		_f("value", "Value (gold)", "int", 10, {"min": 0, "max": 99999}),
		_f("description", "Description", "text", ""),
		_f("damage_type", "Damage type", "enum", "", {"options": ["", "slash", "pierce", "blunt", "magic", "fire"]}),
		_f("icon_id", "Icon id", "string", "", {"help": "Optional bespoke icon (ItemArt.UNIQUE_ICONS); empty = icon by gear type."}),
		_f("starter_stash", "In starter stash", "bool", false, {"help": "Added to the party stash of every encounter and the Gear Dojo."}),
		_f("special", "Special item", "bool", false, {"when": equip, "help": "May exceed the class gear budget; needs a special note."}),
		_f("special_note", "Special note", "text", "", {"when": equip, "help": "The trade-off or playstyle that justifies a special item."}),
		_f("slot", "Slot", "enum", "MAIN_HAND", {"options": ["MAIN_HAND", "OFF_HAND", "HEAD", "BODY", "ACCESSORY_1", "ACCESSORY_2"], "when": equip}),
		_f("weapon_type", "Gear type", "enum", "", {"options": ["", "sword", "axe", "mace", "hammer", "fist", "dagger", "bow", "crossbow", "staff", "wand", "orb", "tome", "shield", "plate", "leather", "cloth"], "when": equip, "help": "Checked against class proficiencies and weapon masteries."}),
		_f("base_power", "Power", "int", 0, {"min": 0, "max": 200, "when": weapon}),
		_f("base_range", "Range", "int", 1, {"min": 1, "max": 6, "when": weapon, "help": "2+ makes the wielder a ranged unit."}),
		_f("two_handed", "Two-handed", "bool", false, {"when": weapon}),
		_f("off_hand_category", "Off-hand category", "enum", "", {"options": ["", "shield", "shortsword", "knuckles", "dagger", "knife", "orb", "tome"], "when": equip, "help": "Set for off-hand-only weapons and shields."}),
		_f("scaling_stat", "Scales with", "enum", "STR", {"options": ["STR", "DEX", "INT"], "when": weapon}),
		_f("block_cost", "Block cost", "int", 1, {"min": 1, "max": 6, "when": equip}),
		_f("burden", "Burden", "int", 0, {"min": 0, "max": 10, "when": equip, "help": "Initiative penalty while worn."}),
		_f("mitigation", "Mitigation", "int", 0, {"min": 0, "max": 100, "when": equip}),
		_f("required_stats", "Requires", "stats", {}, {"keys": ContentSchema.STAT_KEYS, "when": equip}),
		_f("stat_bonuses", "Stat bonuses", "stats", {}, {"keys": ContentSchema.STAT_KEYS, "when": equip}),
		_f("effects", "Effects", "json", [], {"when": equip, "help": "Applies while equipped. Allowed: skill_potency (family or group spell/heal), status_potency (1-25), spell_reach (1-2), resist. [{\"type\": \"skill_potency\", \"group\": \"heal\", \"value\": 0.1}]"}),
		_f("threshold_tiers", "Threshold tiers", "json", [], {"when": weapon, "help":"[{\"stat\": \"STR\", \"threshold\": 18, \"bonus_power\": 4, \"desc\": \"...\"}]"}),
		_f("consumable_type", "Consumable type", "enum", "HEALTH_POTION", {"options": ["HEALTH_POTION", "STAMINA_BREW", "ANTIDOTE", "THROWABLE_BOMB", "FOCUS_INCENSE"], "when": consumable}),
		_f("heal", "Heal", "int", 0, {"min": 0, "max": 999, "when": consumable}),
		_f("damage", "Damage", "int", 0, {"min": 0, "max": 999, "when": consumable}),
		_f("buff_stat", "Buff stat", "enum", "", {"options": ["", "STR", "DEX", "INT", "CONS", "PERC", "KNOW"], "when": consumable}),
		_f("buff_pct", "Buff %", "float", 0.0, {"min": 0.0, "max": 5.0, "step": 0.05, "when": consumable}),
		_f("duration", "Buff rounds", "int", 0, {"min": 0, "max": 10, "when": consumable}),
		_f("use_blocks", "Use blocks", "int", 1, {"min": 1, "max": 6, "when": consumable}),
		_f("max_stack", "Max stack", "int", 1, {"min": 1, "max": 999, "when": {"key": "item_type", "in": ["CONSUMABLE", "MATERIAL", "QUEST"]}}),
	]
