class_name ClassUnitFields
extends RefCounted

# The classes and units field tables of ContentSchema plus their shared gear slots (moved here by CLN-9b;
# ContentSchema.fields("classes" / "units") / field() are the API).

static func _f(key: String, label: String, type: String, default: Variant, extra: Dictionary = {}) -> Dictionary:
	return ContentSchema._f(key, label, type, default, extra)

# The five Forge sections of a class (FORGE1_CLASSES 2) and the extra unit group for AI behaviour (4).
const CLASS_GROUPS: Array[String] = ["identity", "stats", "proficiency", "skills", "loadout"]
const UNIT_AI_GROUP: String = "ai"
const UNIT_AI_ADVANCED_GROUP: String = "ai_advanced"
const AI_ROLES: Array[String] = ["AGGRESSOR", "SKIRMISHER", "SNIPER", "SUPPORT"]
const ID_MIN_LEN: int = 2
const ID_MAX_LEN: int = 24
const NAME_MAX_LEN: int = 32
const DESCRIPTION_MAX_LEN: int = 240
const HEAL_BELOW_MIN: float = 0.1
const HEAL_BELOW_MAX: float = 1.0

# One-release converter (FORGE1_CLASSES 3.5): a list-shaped skill_path is converted on normalize while this is on.
# Off since CLASS1-M migrated the shipped data (a list is now kept as-is and is a validation error); tests flip the
# static var to check both modes. The converter code is removed in a later cleanup pass.
const LIST_PATH_CONVERTER: bool = false
static var list_path_converter: bool = LIST_PATH_CONVERTER

# Default gear of a class (and per-slot overrides on units): one item id per slot.
static func _loadout_fields(inherit: bool) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		out.append(_f(slot, slot.capitalize().replace("_", " "), "ref", "", {"ref": "items", "optional": true, "group": "loadout", "ref_filter": {"item_type": ContentSchema.EQUIP_TYPES}, "help": "Empty = the class's gear; 'none' = no item." if inherit else "Default gear of the class."}))
	var pouch_opts: Dictionary = {"ref": "items", "group": "loadout", "ref_filter": {"item_type": ["CONSUMABLE"]}}
	if inherit:
		pouch_opts["help"] = "Empty = the class's pouch; ['none'] = no consumables."
	out.append(_f("pouch", "Battle pouch", "refs", [], pouch_opts))
	return out

# Key fields (locked on system classes, FORGE1_CLASSES 2.0): id, base_stats, growth, proficiency, ai_role (+ origin). Gear,
# pouch and skill_path are only defaults and stay editable; name, description and art too.
static func class_fields() -> Array[Dictionary]:
	var list: Array[Dictionary] = [
		_f("id", "ID", "id", "", {"group": "identity", "key_field": true, "min_len": ID_MIN_LEN, "max_len": ID_MAX_LEN, "help": "Unique snake_case id (2-24 characters); read-only after the first save."}),
		_f("name", "Name", "string", "New Class", {"group": "identity", "max_len": NAME_MAX_LEN}),
		_f("description", "Description", "text", "", {"group": "identity", "max_len": DESCRIPTION_MAX_LEN}),
		_f("art", "Art", "enum", "warrior", {"group": "identity", "options_from": "art", "help": "Token / portrait template (UnitArt)."}),
		_f("ai_role", "Default AI role", "enum", "AGGRESSOR", {"group": "identity", "key_field": true, "options": AI_ROLES, "help": "The role units of this class play unless they set their own. Behaviour thresholds are unit fields."}),
		_f("base_stats", "Base stats (Lv 1)", "stats", {}, {"group": "stats", "key_field": true, "keys": ContentSchema.STAT_KEYS, "min": StatRules.STAT_FLOOR, "max": StatRules.STAT_CEILING, "help": "Every class totals exactly %d (%d-%d per stat): classes differ only in distribution." % [StatRules.CLASS_BUDGET, StatRules.STAT_FLOOR, StatRules.STAT_CEILING]}),
		_f("growth", "Growth / level", "stats", {}, {"group": "stats", "key_field": true, "keys": ContentSchema.STAT_KEYS, "min": 0, "max": StatRules.GROWTH_STAT_MAX, "help": "Every class gains exactly %d per level (max %d per stat); units also get 1 free point per level to spend." % [StatRules.GROWTH_TOTAL, StatRules.GROWTH_STAT_MAX]}),
		_f("proficiency", "Gear proficiency", "dict_pick", {}, {"group": "proficiency", "key_field": true, "help": "Gear type -> {level 1-%d, scaling %.1f-%.1f}: training level (power, requirement discount, crit, burden, block cost, skill gates) and how easy the gear is to wield (Unit Power multiplier). Unlisted gear can't be equipped; at least one weapon type." % [Proficiency.MAX_LEVEL, Proficiency.MIN_SCALING, Proficiency.MAX_SCALING]}),
		_f("skill_path", "Skill path", "table", {}, {"group": "skills", "min": 1, "max": StatRules.MAX_LEVEL, "help": "Skill FAMILY (tier-1 head) -> the unit level it is owned at; its higher tiers open 2 and 4 levels later unless a rank sets unlock_offset. Advanced families need level %d+. No limit on entries; the first %d by (level, position) are active by default. Basics are never listed (every unit owns them)." % [StatRules.ADVANCED_MIN_LEVEL, StatRules.MAX_CLASS_SLOTS]}),
	]
	list.append_array(_loadout_fields(false))
	return list

static func unit_fields() -> Array[Dictionary]:
	var fixed: Dictionary = {"key": "stat_mode", "in": ["fixed"]}
	var list: Array[Dictionary] = [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Unit", {"help": "Shown on the field; duplicates in one encounter get a number."}),
		_f("side", "Side", "enum", "enemy", {"options": ["party", "enemy", "neutral"]}),
		_f("description", "Description", "text", ""),
		_f("class_id", "Class", "ref", "", {"ref": "classes", "optional": true, "help": "One of the base classes: stats, growth, gear proficiency, skill path and default gear come from it."}),
		_f("level", "Level", "int", 1, {"min": 1, "max": StatRules.MAX_LEVEL}),
		_f("tier", "Tier", "enum", "hero", {"options": ["hero", "minion", "standard", "elite", "boss"], "help": "Scales class-derived HP and attack power (minion 40% / 60% ... boss 170% / 120%). Heroes: 100%."}),
		_f("spend", "Spend points", "stats", {}, {"keys": ContentSchema.STAT_KEYS, "help": "4 + (level - 1) free points, spent exactly (empty = follow the class weights). This is what makes two units of one class differ."}),
		_f("stat_mode", "Stat model", "enum", "derived", {"options": ["derived", "fixed"], "help": "derived: from class, level, spend and gear. fixed: hand-authored vitals (special units such as the training dummy)."}),
		_f("hp", "HP", "int", 50, {"min": 1, "max": 9999, "when": fixed}),
		_f("speed", "Speed", "int", 12, {"min": 1, "max": 40, "when": fixed}),
		_f("attack_range", "Range", "int", 1, {"min": 1, "max": 6, "when": fixed}),
		_f("ap", "AP pool", "int", 0, {"min": 0, "max": 40, "help": "0 = the encounter's unit AP."}),
		_f("power", "Strike power", "int", 0, {"min": 0, "max": 500, "help": "0 = weapon power + unit power (unarmed: STR + DEX). Set to override."}),
		_f("armor", "Armor rating", "int", -1, {"min": -1, "max": 200, "help": "-1 = gear mitigation + STR / 4. Damage taken is cut by rating / (rating + 50)."}),
		_f("art", "Art", "enum", "", {"options_from": "art_inherit", "help": "Token / portrait template (empty = the class's)."}),
		_f("skill_cap", "Skill level cap", "int", 0, {"group": UNIT_AI_ADVANCED_GROUP, "min": 0, "max": 5, "help": "Explicit AP ceiling for the AI (0 = none: any affordable, unlocked tier). Skill unlocks follow the class path levels."}),
		_f("extra_skills", "Extra skills", "refs", [], {"ref": "skills", "ref_filter": {"system": false}, "help": "Skills beyond the class path (learned through events)."}),
		_f("loadout", "Active loadout", "strings", [], {"help": "Skill FAMILIES active in combat (max 5 class + 3 extra, basics are free). Empty = default: the first 5 class families reached by level plus the extra skills' families (max 3)."}),
		_f("ai_role", "AI role", "enum", "", {"group": UNIT_AI_GROUP, "options": [""] + AI_ROLES, "help": "Empty = the class's role."}),
		_f("target_pref", "Targets", "enum", "ROLE", {"group": UNIT_AI_GROUP, "options": ["ROLE", "NEAREST", "WEAKEST"], "help": "ROLE: aggressors pick the nearest foe, others the weakest."}),
		_f("flee_ratio", "Flee at HP", "float", 0.25, {"group": UNIT_AI_GROUP, "min": 0.0, "max": 1.0, "step": 0.05, "help": "Withdraw at or below this HP fraction (0 = never)."}),
		_f("defend_below", "Guard at HP", "float", 0.4, {"group": UNIT_AI_GROUP, "min": 0.0, "max": 1.0, "step": 0.05}),
		_f("prefer_level", "Preferred skill level", "int", 0, {"group": UNIT_AI_ADVANCED_GROUP, "min": 0, "max": 5, "help": "0 = highest affordable."}),
		_f("status_resist", "Status resist %", "stats", {}, {"group": UNIT_AI_ADVANCED_GROUP, "keys": StatusEffects.resistable(), "help": "100 = immune."}),
		_f("reckless", "Reckless", "bool", false, {"group": UNIT_AI_GROUP, "help": "AI: never flees or guards, always charges the nearest foe (berserker behaviour)."}),
		_f("heal_below", "Heal allies below HP", "float", AiProfile.DEFAULT_HEAL_BELOW, {"group": UNIT_AI_GROUP, "min": HEAL_BELOW_MIN, "max": HEAL_BELOW_MAX, "step": 0.05, "help": "SUPPORT role: heal the most wounded ally at or below this HP fraction (AiProfile.heal_below)."}),
	]
	list.append_array(_loadout_fields(true))
	return list
