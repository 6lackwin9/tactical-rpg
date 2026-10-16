class_name ClassUnitFields
extends RefCounted

# The classes and units field tables of ContentSchema plus their shared gear slots (moved here by CLN-9b;
# ContentSchema.fields("classes" / "units") / field() are the API).

static func _f(key: String, label: String, type: String, default: Variant, extra: Dictionary = {}) -> Dictionary:
	return ContentSchema._f(key, label, type, default, extra)

# Default gear of a class (and per-slot overrides on units): one item id per slot.
static func _loadout_fields(inherit: bool) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		out.append(_f(slot, slot.capitalize().replace("_", " "), "ref", "", {"ref": "items", "optional": true, "ref_filter": {"item_type": ContentSchema.EQUIP_TYPES}, "help": "Empty = the class's gear; 'none' = no item." if inherit else "Default gear of the class."}))
	var pouch_opts: Dictionary = {"ref": "items", "ref_filter": {"item_type": ["CONSUMABLE"]}}
	if inherit:
		pouch_opts["help"] = "Empty = the class's pouch; ['none'] = no consumables."
	out.append(_f("pouch", "Battle pouch", "refs", [], pouch_opts))
	return out

static func class_fields() -> Array[Dictionary]:
	var list: Array[Dictionary] = [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Class"),
		_f("description", "Description", "text", ""),
		_f("art", "Art", "enum", "warrior", {"options_from": "art", "help": "Token / portrait template (UnitArt)."}),
		_f("ai_role", "Default AI role", "enum", "AGGRESSOR", {"options": ["AGGRESSOR", "SKIRMISHER", "SNIPER", "SUPPORT"]}),
		_f("base_stats", "Base stats (Lv 1)", "stats", {}, {"keys": ContentSchema.STAT_KEYS, "help": "Every class totals exactly 62 (6-18 per stat): classes differ only in distribution."}),
		_f("growth", "Growth / level", "stats", {}, {"keys": ContentSchema.STAT_KEYS, "help": "Every class gains exactly 5 per level (max 3 per stat); units also get 1 free point per level to spend."}),
		_f("proficiency", "Gear proficiency", "json", {}, {"help": "{\"sword\": {\"level\": 3, \"scaling\": 1.5}, \"leather\": {\"level\": 2, \"scaling\": 1.5}}. Level 1-3 (power, requirement discount, crit, burden, block cost, skill gates); scaling 1.0-2.0 = how easy the gear is to wield (Unit Power multiplier). Unlisted gear can't be equipped."}),
		_f("skill_path", "Skill path", "json", [], {"help": "[{\"level\": 1, \"skill\": \"strike\"}, {\"level\": 3, \"skill\": \"sunder\"}]: the level at which each skill FAMILY (tier-1 row) is owned; its higher tiers open 2 and 4 levels later unless a row sets unlock_offset. Advanced families need level 5+. One step per family; extra skills come from events and passives."}),
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
		_f("skill_cap", "Skill level cap", "int", 0, {"min": 0, "max": 5, "help": "Explicit AP ceiling for the AI (0 = none: any affordable, unlocked tier). Skill unlocks follow the class path levels."}),
		_f("extra_skills", "Extra skills", "refs", [], {"ref": "skills", "ref_filter": {"system": false}, "help": "Skills beyond the class path (learned through events)."}),
		_f("loadout", "Active loadout", "strings", [], {"help": "Skill FAMILIES active in combat (max 5 class + 3 extra, basics are free). Empty = default: the first 5 class families reached by level plus the extra skills' families (max 3)."}),
		_f("ai_role", "AI role", "enum", "", {"options": ["", "AGGRESSOR", "SKIRMISHER", "SNIPER", "SUPPORT"], "help": "Empty = the class's role."}),
		_f("target_pref", "Targets", "enum", "ROLE", {"options": ["ROLE", "NEAREST", "WEAKEST"], "help": "ROLE: aggressors pick the nearest foe, others the weakest."}),
		_f("flee_ratio", "Flee at HP", "float", 0.25, {"min": 0.0, "max": 1.0, "step": 0.05, "help": "Withdraw at or below this HP fraction (0 = never)."}),
		_f("defend_below", "Guard at HP", "float", 0.4, {"min": 0.0, "max": 1.0, "step": 0.05}),
		_f("prefer_level", "Preferred skill level", "int", 0, {"min": 0, "max": 5, "help": "0 = highest affordable."}),
		_f("status_resist", "Status resist %", "stats", {}, {"keys": StatusEffects.resistable(), "help": "100 = immune."}),
		_f("reckless", "Reckless", "bool", false, {"help": "AI: never flees or guards, always charges the nearest foe (berserker behaviour)."}),
	]
	list.append_array(_loadout_fields(true))
	return list
