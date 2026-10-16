class_name ContentSchema
extends RefCounted

# ==============================================================================
# Tactical RPG - Content Schema (what each catalog entry looks like)
# One field list per catalog kind. The schema drives everything generic:
# normalising JSON (types, defaults, key order), field validation, and the
# Content Forge form (labels, widgets, ranges, help, conditional visibility).
#
# Field: {"key", "label", "type", "default", ...}
#   type: id | string | text | int | float | bool | enum | ref | refs | stats | strings | json
#   min / max / step     numeric range
#   options              enum choices (Array[String]); "options_from": "zones" | "art" | "status"
#   ref                  catalog kind a ref/refs field points at; "ref_filter": {key: value}
#   when                 {"key": k, "in": [...]} or {"key": k, "not_in": [...]}: shown/saved only then
#   keys                 stats: the allowed keys
#   readonly             shown, never edited (e.g. the "system" flag)
#   help                 tooltip text
# Enum values are stored by name ("ATTACK", "MAIN_HAND") so the JSON stays readable.
# ==============================================================================

const KINDS: Array[String] = ["skills", "items", "passives", "classes", "units", "encounters"]
const ORIGIN_SYSTEM: String = "system"
const ORIGIN_USER: String = "user"

const STAT_KEYS: Array[String] = ["STR", "DEX", "INT", "CONS", "PERC", "KNOW"]   # = StatRules.STATS
const EQUIP_TYPES: Array[String] = ["WEAPON", "SHIELD", "ARMOR", "ACCESSORY"]
const LOADOUT_SLOTS: Array[String] = ["main_hand", "off_hand", "head", "body", "accessory_1", "accessory_2"]

static func kind_label(kind: String) -> String:
	match kind:
		"skills":
			return "Skills"
		"items":
			return "Items"
		"passives":
			return "Passives"
		"classes":
			return "Classes"
		"units":
			return "Units"
		"encounters":
			return "Encounters"
	return kind.capitalize()

static func fields(kind: String) -> Array[Dictionary]:
	var list: Array[Dictionary] = []
	match kind:
		"skills":
			return _skill_fields()
		"items":
			list = _item_fields()
		"passives":
			list = _passive_fields()
		"classes":
			list = _class_fields()
		"units":
			list = _unit_fields()
		"encounters":
			list = _encounter_fields()
		_:
			return list
	list.append(origin_field())
	return list

# The common field of every Forge kind (FORGE1 section 12): who owns the asset. Default system; only user assets store it.
static func origin_field() -> Dictionary:
	return _f("origin", "Origin", "enum", ORIGIN_SYSTEM, {"options": [ORIGIN_SYSTEM, ORIGIN_USER], "readonly": true, "key_field": true, "help": "system: shipped (key fields locked, delete only through SAFE DELETE). user: your own asset. Set by the Forge, never typed."})

static func field(kind: String, key: String) -> Dictionary:
	if kind == "skills":   # look up in the cache and copy only the one field (fields() deep-copies the whole list)
		if _skill_cache.is_empty():
			_skill_cache = _build_skill_fields()
		for f: Dictionary in _skill_cache:
			if str(f.get("key", "")) == key:
				return f.duplicate(true)
		return {}
	for f: Dictionary in fields(kind):
		if str(f.get("key", "")) == key:
			return f
	return {}

# ------------------------------------------------------------------------------
# Field lists
# ------------------------------------------------------------------------------

static func _f(key: String, label: String, type: String, default: Variant, extra: Dictionary = {}) -> Dictionary:
	var f: Dictionary = {"key": key, "label": label, "type": type, "default": default}
	f.merge(extra, true)
	return f

static var _skill_cache: Array[Dictionary] = []

# The skill field list (cached: it is generated from SkillKinds, so it is built once). Callers get a deep copy, like
# every other kind's freshly built list, so editing a returned field can never change the schema.
static func _skill_fields() -> Array[Dictionary]:
	if _skill_cache.is_empty():
		_skill_cache = _build_skill_fields()
	var out: Array[Dictionary] = []
	out.assign(_skill_cache.duplicate(true))
	return out

static func _build_skill_fields() -> Array[Dictionary]:
	var list: Array[Dictionary] = [
		_f("id", "ID", "id", "", {"key_field": true, "help": "Unique snake_case id used by units and tests. Ranks are <id>, <id>_2, <id>_3."}),
		_f("name", "Name", "string", "New Skill", {"max_len": 32, "help": "Rank names are derived: Mend, Mend II, Mend III (all must fit 32 characters)."}),
		_f("kind", "Kind", "enum", "ATTACK", {"options": SkillKinds.names(), "key_field": true, "help": "MOVE / TARGET / STATUS / REST are engine skills; author ATTACK, DEFEND, FOCUS, SUPPORT (heal / cleanse / buff a friendly unit or yourself) or DISENGAGE (jump away from an enemy)."}),
		_f("level", "Level (= AP cost)", "int", 1, {"min": 1, "max": 5}),
		_f("blocks", "Blocks", "int", 1, {"min": 1, "max": 6, "help": "Timeline length; the effect lands at the end of the last block."}),
		_f("description", "Description", "template", "", {"max_len": 240, "help": "A template: {name} {ap} {blocks} {power} {dash} {reach} {weapons} {proficiency} {status} {potency} {magnitude} {rounds} {heal} {cleanse} {mitigation} {boost} {jump} {recover} {unlock}. {{ and }} are literal braces. Each rank renders it with its own numbers."}),
		_f("template", "Description template (internal)", "string", "", {"internal": true, "help": "Flat rows carry the template their description was rendered from."}),
		_f("system", "System skill", "bool", false, {"readonly": true, "key_field": true, "help": "Engine skills the planner inserts itself (Move, Rest, Switch Target, status blocks)."}),
		_f("basic", "Basic skill", "bool", false, {"key_field": true, "help": "Owned by every unit from level 1; not counted against the active loadout slots; exactly one rank; not valid in a class path."}),
		_f("learned_only", "Learn only", "bool", false, {"help": "Unrestricted hand-built units (tests, sandbox) don't know it unless they learn it; class-derived units get it from skill paths / extra skills as usual."}),
		_f("grade", "Grade", "enum", "novice", {"options": ["novice", "advanced", "expert"], "key_field": true, "help": "novice, advanced or expert (reserved: no expert skill ships). Advanced and expert class-path steps need unit level 5+ and KNOW can open them earlier (never below level 3)."}),
		_f("requires", "Requires", "refs", [], {"ref": "skills", "ref_filter": {"system": false, "basic": false}, "key_field": true, "help": "Skill families that must ALL be owned to own or learn this skill; each of a strictly lower grade."}),
		origin_field(),
		_f("stance", "Stance", "enum", "AUTO", {"options": ["AUTO", "OFFENSIVE", "TACTICAL", "DEFENSIVE", "NEUTRAL"], "help": "AUTO: attacks use Offensive."}),
		_f("unlock_offset", "Unlock offset", "int", -1, {"min": -1, "max": 8, "help": "Levels after the family's unlock level (its class path level) at which THIS rank opens. -1 = derive: 0 / 2 / 4 for ranks 1 / 2 / 3. Rank 1 is always 0."}),
		_f("weapon_types", "Needs weapon type", "multi_pick", [], {"options_from": "weapon_types", "help": "Empty = any. Otherwise the wielded weapon must be one of these gear types, or the token 'unarmed' (no weapon in that hand; its proficiency is the class's fist level). Needs a proficiency of 1+. Class-derived units only."}),
		_f("min_proficiency", "Needs proficiency", "int", 0, {"min": 0, "max": 3, "help": "Training level in the wielded weapon type required to use the skill (0 = none)."}),
		_f("reach_min", "Reach min", "int", 0, {"min": 0, "max": 8, "help": "0 = the unit's default band (attacks: melee 1, ranged 2-3; support: 0-3)."}),
		_f("reach_max", "Reach max", "int", 0, {"min": 0, "max": 8}),
		_f("area", "Area", "enum", "NONE", {"options": ["NONE", "PLUS", "BURST", "LINE", "SWEEP"], "help": "NONE = single target. PLUS: tile + 4 neighbours, BURST: 3x3, LINE: 3 tiles from an adjacent tile, SWEEP: an L of 3 tiles around you (aim at a diagonal neighbour)."}),
		_f("power_mult", "Power x", "float", 1.0, {"min": 0.0, "max": 6.0, "step": 0.05, "help": "Multiplier on the unit's strike power. 0 = no damage (a debuff-only attack; needs an Inflicts status)."}),
		_f("weapon_hand", "Weapon hand", "enum", "main", {"options": ["main", "off", "both"], "help": "main: the main weapon. off: the off-hand weapon, or bare hands (melee, distance 1). both: dual-wield strike (engine pending)."}),
		_f("dash_tiles", "Gap-close tiles", "int", 0, {"min": 0, "max": 6, "help": "Lunge: move up to this many tiles to get into reach, then strike. Only offered while the target is out of reach; it is the unit's move for the round (one per round, blocked by root, terrain and units). AP is the skill's level alone."}),
		_f("status_effect", "Inflicts", "enum", "", {"options_from": "status"}),
		_f("status_potency", "Status potency %", "int", 0, {"min": 0, "max": 200, "when": {"key": "status_effect", "not_in": [""]}, "help": "Chance = potency - target debuff resistance - passive resistance."}),
		_f("status_magnitude", "Status magnitude %", "int", 0, {"min": 0, "max": 95, "when": {"key": "status_effect", "in": ["weaken"]}, "help": "Weaken: percent of outgoing skill power the afflicted unit loses (1-25). Not rolled; carried by the applied instance. 0 for every other status."}),
		_f("status_rounds", "Status rounds", "int", 1, {"min": 1, "max": 5, "help": "How many rounds an inflicted / granted status lasts once active (it starts next round; a DEFEND skill's riposte is active at once and counts the current round)."}),
		_f("target_mode", "Targets", "enum", "ALLY_OR_SELF", {"options": ["SELF", "ALLY", "ALLY_OR_SELF"], "help": "SELF: the caster only. ALLY: a friendly unit other than the caster. ALLY_OR_SELF: either."}),
		_f("heal_stat", "Heal scales with", "enum", "", {"options": ["", "INT", "KNOW"], "help": "Heal = heal base + stat x heal per point (SupportRules.heal_amount), never DamageCalculator."}),
		_f("heal_base", "Heal base", "int", 0, {"min": 0, "max": 500}),
		_f("heal_mult", "Heal per stat point", "float", 0.0, {"min": 0.0, "max": 10.0, "step": 0.05}),
		_f("grant_status", "Grants", "enum", "", {"options_from": "status_buff", "help": "SUPPORT: haste (+1 tile per move block) or taunt (enemy AI must target it while reachable), queued for next round. DEFEND: riposte (counters adjacent melee attackers for 50% power), active at once, lost on a stance change; or any support buff (taunt, haste), queued for next round like a support grant."}),
		_f("cleanse_statuses", "Cleanses", "multi_pick", [], {"options_from": "debuffs_or_all", "help": "Debuffs removed from the target (active and pending, never buffs): stagger, stun, root, bleed, or the keyword 'all' for every debuff. Empty = none. Combine freely with heal and grants."}),
		_f("mitigation", "Mitigation", "float", 0.5, {"min": 0.0, "max": 0.95, "step": 0.05}),
		_f("boost", "Attack boost x", "float", 1.5, {"min": 1.0, "max": 5.0, "step": 0.05}),
		_f("boost_rounds", "Boost rounds", "int", 2, {"min": 1, "max": 10}),
		_f("jump_tiles", "Jump tiles", "int", 0, {"min": 0, "max": 6, "help": "Disengage: jump up to this many tiles away from the nearest (or a chosen) enemy. It is the unit's move for the round (first in the sequence, refused when rooted); AP is the skill's level; units can be leapt over, walls cannot."}),
		_f("recover_ap", "AP recovered", "int", 2, {"min": 0, "max": 12}),
		_f("family", "Skill family", "string", "", {"readonly": true, "help": "Derived: the head skill id of a higher rank (empty on rank 1)."}),
		_f("tier", "Rank", "int", 1, {"min": 1, "max": 5, "readonly": true, "help": "Derived: 1 for the head skill, 2 / 3 for <id>_2 / <id>_3."}),
	]
	# Generated from SkillKinds: the kind lists (`when`), Forge sections and the rank flag.
	for f: Dictionary in list:
		var key: String = str(f["key"])
		var kinds: Array[String] = SkillKinds.kinds_with(key)
		if not f.has("when") and kinds.size() < SkillKinds.names().size():
			f["when"] = {"key": "kind", "in": kinds}
		var sections: Array[String] = []
		for section: Variant in SkillKinds.SECTION_FIELDS:
			if SkillKinds.section_fields(str(section)).has(key):
				sections.append(str(section))
		if not sections.is_empty():
			f["section"] = sections
		f["rank"] = SkillRanks.RANK_KEYS.has(key)
	return list

static func _item_fields() -> Array[Dictionary]:
	var equip: Dictionary = {"key": "item_type", "in": EQUIP_TYPES}
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
		_f("required_stats", "Requires", "stats", {}, {"keys": STAT_KEYS, "when": equip}),
		_f("stat_bonuses", "Stat bonuses", "stats", {}, {"keys": STAT_KEYS, "when": equip}),
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

static func _passive_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Passive"),
		_f("description", "Description", "text", ""),
		_f("effects", "Effects", "json", [], {"help": "[{\"type\": \"stat\", \"stat\": \"KNOW\", \"value\": 3, \"context\": {\"biome\": \"ruins\"}}, {\"type\": \"proficiency\", \"gear\": \"sword\", \"levels\": 1}, {\"type\": \"resist\", \"status\": \"stagger\", \"value\": 25}, {\"type\": \"ap_regen\", \"value\": 1}, {\"type\": \"skill\", \"skill\": \"quick_stab\"}, {\"type\": \"skill_potency\", \"family\": \"strike\", \"value\": 0.25}, {\"type\": \"unlock\", \"family\": \"riposte\", \"levels\": 2}]"}),
	]

# Default gear of a class (and per-slot overrides on units): one item id per slot.
static func _loadout_fields(inherit: bool) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for slot: String in LOADOUT_SLOTS:
		out.append(_f(slot, slot.capitalize().replace("_", " "), "ref", "", {"ref": "items", "optional": true, "ref_filter": {"item_type": EQUIP_TYPES}, "help": "Empty = the class's gear." if inherit else "Default gear of the class."}))
	out.append(_f("pouch", "Battle pouch", "refs", [], {"ref": "items", "ref_filter": {"item_type": ["CONSUMABLE"]}}))
	return out

static func _class_fields() -> Array[Dictionary]:
	var list: Array[Dictionary] = [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Class"),
		_f("description", "Description", "text", ""),
		_f("art", "Art", "enum", "warrior", {"options_from": "art", "help": "Token / portrait template (UnitArt)."}),
		_f("ai_role", "Default AI role", "enum", "AGGRESSOR", {"options": ["AGGRESSOR", "SKIRMISHER", "SNIPER", "SUPPORT"]}),
		_f("base_stats", "Base stats (Lv 1)", "stats", {}, {"keys": STAT_KEYS, "help": "Every class totals exactly 62 (6-18 per stat): classes differ only in distribution."}),
		_f("growth", "Growth / level", "stats", {}, {"keys": STAT_KEYS, "help": "Every class gains exactly 5 per level (max 3 per stat); units also get 1 free point per level to spend."}),
		_f("proficiency", "Gear proficiency", "json", {}, {"help": "{\"sword\": {\"level\": 3, \"scaling\": 1.5}, \"leather\": {\"level\": 2, \"scaling\": 1.5}}. Level 1-3 (power, requirement discount, crit, burden, block cost, skill gates); scaling 1.0-2.0 = how easy the gear is to wield (Unit Power multiplier). Unlisted gear can't be equipped."}),
		_f("skill_path", "Skill path", "json", [], {"help": "[{\"level\": 1, \"skill\": \"strike\"}, {\"level\": 3, \"skill\": \"sunder\"}]: the level at which each skill FAMILY (tier-1 row) is owned; its higher tiers open 2 and 4 levels later unless a row sets unlock_offset. Advanced families need level 5+. One step per family; extra skills come from events and passives."}),
	]
	list.append_array(_loadout_fields(false))
	return list

static func _unit_fields() -> Array[Dictionary]:
	var fixed: Dictionary = {"key": "stat_mode", "in": ["fixed"]}
	var list: Array[Dictionary] = [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Unit", {"help": "Shown on the field; duplicates in one encounter get a number."}),
		_f("side", "Side", "enum", "enemy", {"options": ["party", "enemy", "neutral"]}),
		_f("description", "Description", "text", ""),
		_f("class_id", "Class", "ref", "", {"ref": "classes", "optional": true, "help": "One of the base classes: stats, growth, gear proficiency, skill path and default gear come from it."}),
		_f("level", "Level", "int", 1, {"min": 1, "max": StatRules.MAX_LEVEL}),
		_f("tier", "Tier", "enum", "hero", {"options": ["hero", "minion", "standard", "elite", "boss"], "help": "Scales class-derived HP and attack power (minion 40% / 60% ... boss 170% / 120%). Heroes: 100%."}),
		_f("spend", "Spend points", "stats", {}, {"keys": STAT_KEYS, "help": "4 + (level - 1) free points, spent exactly (empty = follow the class weights). This is what makes two units of one class differ."}),
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

static func _encounter_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Encounter"),
		_f("description", "Description", "text", ""),
		_f("zone", "Zone", "enum", "stone_arena", {"options_from": "zones"}),
		_f("arena", "Arena", "enum", "CLOSED", {"options": ["CLOSED", "OPEN"], "help": "OPEN: every edge is an exit."}),
		_f("exits", "Exit tiles", "json", [[8, 4]], {"help": "[[x, y], ...] on the 9x9 grid (CLOSED arenas)."}),
		_f("max_rounds", "Max rounds", "int", 6, {"min": 1, "max": 30}),
		_f("total_blocks", "Blocks / round", "int", 6, {"min": 3, "max": 10}),
		_f("ap_regen", "AP regen / round", "int", 4, {"min": -1, "max": 20, "help": "-1 = refill to max each round."}),
		_f("unit_ap", "Unit AP", "int", 12, {"min": 0, "max": 40, "help": "Starting / max AP for units without their own pool."}),
		_f("rng_seed", "RNG seed", "int", 1337, {"min": 0, "max": 2147483647}),
		_f("party", "Party", "refs", [], {"ref": "units", "ref_filter": {"side": "party"}}),
		_f("enemies", "Enemies", "refs", [], {"ref": "units", "ref_filter": {"side": "enemy"}, "dupes": true, "help": "The same unit may appear more than once."}),
		_f("wave_units", "Reinforcements", "refs", [], {"ref": "units", "ref_filter": {"side": "enemy"}, "dupes": true, "help": "Enemies that join mid-battle (one wave). They are planned by the AI from the next round."}),
		_f("wave_round", "Wave round", "int", 1, {"min": 1, "max": 30}),
		_f("wave_block", "Wave block", "int", 4, {"min": 1, "max": 10, "help": "The wave enters before this block resolves."}),
		_f("wave_label", "Wave label", "string", "Reinforcements arrive!"),
	]

# ------------------------------------------------------------------------------
# Options & visibility
# ------------------------------------------------------------------------------

static func options_of(f: Dictionary) -> Array[String]:
	var out: Array[String] = []
	match str(f.get("options_from", "")):
		"zones":
			return CombatZones.zone_ids()
		"art":
			out.assign(UnitArt.CLASS_KEYS)
			return out
		"art_inherit":
			out.append("")
			out.append_array(UnitArt.CLASS_KEYS)
			return out
		"status":
			out.append("")
			out.append_array(StatusEffects.debuffs())
			return out
		"status_buff":
			out.append("")
			out.append_array(StatusEffects.buffs())
			return out
		"weapon_types":
			return SkillDef.weapon_type_tokens()
		"debuffs_or_all":
			out.append_array(StatusEffects.debuffs())
			out.append(SkillDef.CLEANSE_ALL)
			return out
	var raw: Variant = f.get("options", [])
	if raw is Array:
		var arr: Array = raw
		for v: Variant in arr:
			out.append(str(v))
	return out

# True when `f` applies to `entry` (its "when" condition holds).
static func is_visible(f: Dictionary, entry: Dictionary) -> bool:
	var raw: Variant = f.get("when", null)
	if not (raw is Dictionary):
		return true
	var cond: Dictionary = raw
	var value: String = str(entry.get(str(cond.get("key", "")), ""))
	if cond.has("in"):
		var allowed: Array = cond["in"]
		return allowed.has(value)
	if cond.has("not_in"):
		var denied: Array = cond["not_in"]
		return not denied.has(value)
	return true

# True when a ref target entry passes the field's "ref_filter".
static func passes_filter(f: Dictionary, target: Dictionary) -> bool:
	var raw: Variant = f.get("ref_filter", null)
	if not (raw is Dictionary):
		return true
	var filter: Dictionary = raw
	for key: Variant in filter:
		var want: Variant = filter[key]
		var have: Variant = target.get(key, null)
		if want is Array:
			var wants: Array = want
			if not wants.has(have):
				return false
		elif str(want) != str(have):
			return false
	return true

# ------------------------------------------------------------------------------
# Normalising: coerce types, fill defaults, schema key order
# ------------------------------------------------------------------------------

static func defaults(kind: String) -> Dictionary:
	var d: Dictionary = {}
	for f: Dictionary in fields(kind):
		d[str(f["key"])] = _copy(f["default"])
	return d

# A clean copy of `entry`: every schema field present and typed, unknown keys kept
# at the end (so hand-added data survives a round trip through the Forge).
static func normalize(kind: String, entry: Dictionary) -> Dictionary:
	var src: Dictionary = _legacy_skill(entry) if kind == "skills" else entry
	var out: Dictionary = {}
	for f: Dictionary in fields(kind):
		var key: String = str(f["key"])
		out[key] = coerce(f, src.get(key, f["default"]))
	for key: Variant in src:
		if not out.has(key):
			out[key] = _copy(src[key])
	return out

# Pre-FORGE1 skill keys, accepted for one release: advanced: true -> grade advanced, offhand: true -> weapon_hand off.
static func _legacy_skill(entry: Dictionary) -> Dictionary:
	if not entry.has("advanced") and not entry.has("offhand"):
		return entry
	var out: Dictionary = entry.duplicate()
	if out.has("advanced"):
		if DictUtil.to_bool(out["advanced"]) and not out.has("grade"):
			out["grade"] = "advanced"
		out.erase("advanced")
	if out.has("offhand"):
		if DictUtil.to_bool(out["offhand"]) and not out.has("weapon_hand"):
			out["weapon_hand"] = "off"
		out.erase("offhand")
	return out

# The entry as written to disk: fields hidden by "when" (and default "system" / "origin") are left out.
static func to_storage(kind: String, entry: Dictionary) -> Dictionary:
	var full: Dictionary = normalize(kind, entry)
	var out: Dictionary = {}
	for f: Dictionary in fields(kind):
		var key: String = str(f["key"])
		if not is_visible(f, full) or DictUtil.to_bool(f.get("internal", false)):
			continue
		if key == "system" and not DictUtil.to_bool(full[key]):
			continue
		if key == "origin" and str(full[key]) == ORIGIN_SYSTEM:
			continue
		out[key] = full[key]
	for key: Variant in full:
		if not out.has(key) and field(kind, str(key)).is_empty():
			out[key] = full[key]
	return out

static func coerce(f: Dictionary, value: Variant) -> Variant:
	var fallback: Variant = f.get("default", null)
	match str(f.get("type", "string")):
		"int":
			var n: int = DictUtil.to_int(value, DictUtil.to_int(fallback))
			if f.has("min"):
				n = maxi(n, DictUtil.to_int(f["min"]))
			if f.has("max"):
				n = mini(n, DictUtil.to_int(f["max"]))
			return n
		"float":
			var x: float = DictUtil.to_float(value, DictUtil.to_float(fallback))
			if f.has("min"):
				x = maxf(x, DictUtil.to_float(f["min"]))
			if f.has("max"):
				x = minf(x, DictUtil.to_float(f["max"]))
			return snappedf(x, 0.0001)
		"bool":
			return DictUtil.to_bool(value, DictUtil.to_bool(fallback))
		"refs", "strings", "multi_pick":
			var list: Array[String] = []
			if value is Array:
				var arr: Array = value
				for v: Variant in arr:
					if not str(v).is_empty():
						list.append(str(v))
			return list
		"stats":
			var stats: Dictionary = {}
			if value is Dictionary:
				var src: Dictionary = value
				for key: Variant in src:
					var raw: Variant = src[key]
					if raw is int or raw is float:
						var amount: int = DictUtil.to_int(raw)
						if amount != 0:
							stats[str(key)] = amount
			return stats
		"json":
			return _copy(value) if (value is Array or value is Dictionary) else _copy(fallback)
	return str(value) if value != null else str(fallback)

static func _copy(v: Variant) -> Variant:
	if v is Array:
		var arr: Array = v
		return _json_ints(arr.duplicate(true))
	if v is Dictionary:
		var dict: Dictionary = v
		return _json_ints(dict.duplicate(true))
	return v

# JSON parses every number as float; whole numbers inside free-form json fields go back to int.
static func _json_ints(v: Variant) -> Variant:
	if v is float:
		var x: float = v
		return roundi(x) if x == floorf(x) else x
	if v is Array:
		var arr: Array = v
		for i: int in range(arr.size()):
			arr[i] = _json_ints(arr[i])
		return arr
	if v is Dictionary:
		var dict: Dictionary = v
		for key: Variant in dict:
			dict[key] = _json_ints(dict[key])
		return dict
	return v

# Field-level problems (types are already coerced; this checks ids and required values).
static func field_errors(kind: String, entry: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var id: String = str(entry.get("id", ""))
	if id.is_empty():
		errors.append("ID is required.")
	elif not is_valid_id(id):
		errors.append("ID '%s' must be lowercase snake_case (a-z, 0-9, _)." % id)
	if str(entry.get("name", "")).strip_edges().is_empty():
		errors.append("Name is required.")
	for f: Dictionary in fields(kind):
		if str(f.get("type", "")) == "enum" and is_visible(f, entry):
			var value: String = str(entry.get(str(f["key"]), ""))
			if not options_of(f).has(value):
				errors.append("%s: '%s' is not one of %s." % [str(f["label"]), value, ", ".join(options_of(f))])
		if f.has("max_len") and str(entry.get(str(f["key"]), "")).length() > DictUtil.to_int(f["max_len"]):
			errors.append("%s is %d characters (max %d)." % [str(f["label"]), str(entry.get(str(f["key"]), "")).length(), DictUtil.to_int(f["max_len"])])
	return errors

static func is_valid_id(id: String) -> bool:
	var re := RegEx.new()
	re.compile("^[a-z][a-z0-9_]*$")
	return re.search(id) != null
