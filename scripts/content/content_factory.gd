class_name ContentFactory
extends RefCounted

# ==============================================================================
# Tactical RPG - Content Factory (catalog entries <-> engine objects)
# Builds SkillDef / Item / ClassArchetype / Combatant (+ AiProfile) from
# ContentCatalog entries, and serialises engine objects back to entries (used to
# seed the catalog from code and by the round-trip tests).
# ==============================================================================

static func _enum(names: Array, value: String, fallback: int) -> int:
	var i: int = names.find(value)
	return i if i >= 0 else fallback

static func _name(names: Array, value: int) -> String:
	return str(names[value]) if value >= 0 and value < names.size() else ""

const NO_AUTO_SPEND: String = "_no_auto_spend"   # unit(): an empty spend stays empty (PartyBuilder preview, UI2-L)

static func _ints(d: Variant) -> Dictionary:
	var out: Dictionary = {}
	if d is Dictionary:
		var src: Dictionary = d
		for key: Variant in src:
			out[str(key)] = DictUtil.to_int(src[key])
	return out

static func _strings(v: Variant) -> Array[String]:
	var out: Array[String] = []
	if v is Array:
		var arr: Array = v
		for x: Variant in arr:
			out.append(str(x))
	return out

# ------------------------------------------------------------------------------
# Skills
# ------------------------------------------------------------------------------

static func skill(entry: Dictionary) -> SkillDef:
	var e: Dictionary = ContentSchema.normalize("skills", entry)
	var kind: int = _enum(SkillDef.Kind.keys(), str(e["kind"]), SkillDef.Kind.ATTACK)
	var s: SkillDef = SkillDef.create(str(e["id"]), str(e["name"]), kind as SkillDef.Kind, DictUtil.to_int(e["level"]), DictUtil.to_int(e["blocks"]), str(e["description"]))
	s.power_mult = DictUtil.to_float(e["power_mult"])
	s.area = _enum(SkillDef.Area.keys(), str(e["area"]), SkillDef.Area.NONE) as SkillDef.Area
	s.weapon_hand = _enum(SkillDef.WeaponHand.keys(), str(e["weapon_hand"]).to_upper(), SkillDef.WeaponHand.MAIN) as SkillDef.WeaponHand
	s.stance_override = -1 if str(e["stance"]) == "AUTO" else _enum(CombatTypes.StanceType.keys(), str(e["stance"]), -1)
	s.status_effect = str(e["status_effect"])
	s.status_potency = DictUtil.to_int(e["status_potency"])
	s.status_magnitude = DictUtil.to_int(e["status_magnitude"])
	s.reach_min = DictUtil.to_int(e["reach_min"])
	s.reach_max = DictUtil.to_int(e["reach_max"])
	s.weapon_types = _strings(e["weapon_types"])
	s.min_proficiency = DictUtil.to_int(e["min_proficiency"])
	s.mitigation = DictUtil.to_float(e["mitigation"])
	s.boost = DictUtil.to_float(e["boost"])
	s.boost_rounds = DictUtil.to_int(e["boost_rounds"])
	s.recover_ap = DictUtil.to_int(e["recover_ap"])
	s.dash_tiles = DictUtil.to_int(e["dash_tiles"])
	s.status_rounds = DictUtil.to_int(e["status_rounds"], 1)
	s.target_mode = _enum(SkillDef.TargetMode.keys(), str(e["target_mode"]), SkillDef.TargetMode.ALLY_OR_SELF) as SkillDef.TargetMode
	s.heal_stat = str(e["heal_stat"])
	s.heal_base = DictUtil.to_int(e["heal_base"])
	s.heal_mult = DictUtil.to_float(e["heal_mult"])
	s.grant_status = str(e["grant_status"])
	s.cleanse_statuses = _strings(e["cleanse_statuses"])
	s.jump_tiles = DictUtil.to_int(e["jump_tiles"])
	s.ends_in_stance = -1 if str(e["ends_in_stance"]).is_empty() else _enum(CombatTypes.StanceType.keys(), str(e["ends_in_stance"]), -1)
	s.disrupt = _enum(SkillDef.Disrupt.keys(), str(e["disrupt"]), SkillDef.Disrupt.NONE)
	s.exploit = _exploit_rows(e["exploit"])
	s.exploit_consume = DictUtil.to_bool(e["exploit_consume"])
	s.learned_only = DictUtil.to_bool(e["learned_only"])
	s.family = "" if str(e["family"]) == s.id else str(e["family"])
	s.tier = DictUtil.to_int(e["tier"], 1)
	s.basic = DictUtil.to_bool(e["basic"])
	s.grade = _enum(SkillDef.Grade.keys(), str(e["grade"]).to_upper(), SkillDef.Grade.NOVICE) as SkillDef.Grade
	s.requires = _strings(e["requires"])
	s.unlock_offset = DictUtil.to_int(e["unlock_offset"], -1)
	s.system = DictUtil.to_bool(e["system"])
	s.template = str(e["template"])
	s.origin = str(e["origin"])
	return s

# The exploit list of a skill row as typed {"status": String, "power_mult": float} rows (S12C-L).
static func _exploit_rows(raw: Variant) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for item: Variant in DictUtil.array_of(raw):
		var row: Dictionary = DictUtil.dict_of(item)
		out.append({"status": str(row.get("status", "")), "power_mult": DictUtil.to_float(row.get("power_mult", 1.0), 1.0)})
	return out

static func skill_to_entry(s: SkillDef) -> Dictionary:
	return ContentSchema.normalize("skills", {
		"id": s.id, "name": s.display_name, "kind": _name(SkillDef.Kind.keys(), s.kind),
		"level": s.level, "blocks": s.blocks, "description": s.description, "system": s.system,
		"power_mult": s.power_mult, "area": _name(SkillDef.Area.keys(), s.area), "weapon_hand": _name(SkillDef.WeaponHand.keys(), s.weapon_hand).to_lower(),
		"stance": "AUTO" if s.stance_override < 0 else _name(CombatTypes.StanceType.keys(), s.stance_override),
		"status_effect": s.status_effect, "status_potency": s.status_potency, "status_magnitude": s.status_magnitude,
		"reach_min": s.reach_min, "reach_max": s.reach_max, "weapon_types": s.weapon_types, "min_proficiency": s.min_proficiency, "mitigation": s.mitigation,
		"boost": s.boost, "boost_rounds": s.boost_rounds, "recover_ap": s.recover_ap,
		"dash_tiles": s.dash_tiles, "status_rounds": s.status_rounds, "target_mode": _name(SkillDef.TargetMode.keys(), s.target_mode),
		"heal_stat": s.heal_stat, "heal_base": s.heal_base, "heal_mult": s.heal_mult, "grant_status": s.grant_status, "learned_only": s.learned_only,
		"cleanse_statuses": s.cleanse_statuses, "jump_tiles": s.jump_tiles,
		"ends_in_stance": "" if s.ends_in_stance < 0 else _name(CombatTypes.StanceType.keys(), s.ends_in_stance),
		"disrupt": _name(SkillDef.Disrupt.keys(), s.disrupt), "exploit": s.exploit, "exploit_consume": s.exploit_consume,
		"family": s.family, "tier": s.tier, "template": s.template, "origin": s.origin, "basic": s.basic, "grade": _name(SkillDef.Grade.keys(), s.grade).to_lower(), "requires": s.requires, "unlock_offset": s.unlock_offset,
	})

# ------------------------------------------------------------------------------
# Items
# ------------------------------------------------------------------------------

static func item(entry: Dictionary) -> Item:
	var e: Dictionary = ContentSchema.normalize("items", entry)
	var type: String = str(e["item_type"])
	var rarity: Item.ItemRarity = _enum(Item.ItemRarity.keys(), str(e["rarity"]), 0) as Item.ItemRarity
	var result: Item = null
	if type == "CONSUMABLE":
		var con := Consumable.new(str(e["id"]), str(e["name"]),
			_enum(Consumable.ConsumableType.keys(), str(e["consumable_type"]), 0) as Consumable.ConsumableType,
			DictUtil.to_int(e["heal"]), DictUtil.to_int(e["damage"]), str(e["buff_stat"]), DictUtil.to_float(e["buff_pct"]), DictUtil.to_int(e["duration"]),
			DictUtil.to_int(e["use_blocks"]), DictUtil.to_float(e["weight"]), DictUtil.to_int(e["value"]), rarity)
		con.max_stack = DictUtil.to_int(e["max_stack"])
		con.use_reach = DictUtil.to_int(e["use_reach"])
		result = con
	elif ContentSchema.EQUIP_TYPES.has(type):
		var tiers: Array[Dictionary] = []
		var raw_tiers: Variant = e["threshold_tiers"]
		if raw_tiers is Array:   # every gear type (armor and shield mitigation tiers, any-gear stat / effect tiers)
			var tier_list: Array = raw_tiers
			for t: Variant in tier_list:
				if t is Dictionary:
					var tier: Dictionary = t
					tiers.append(tier.duplicate(true))
		var eq := Equipment.new(str(e["id"]), str(e["name"]),
			_enum(Equipment.EquipSlot.keys(), str(e["slot"]), 0) as Equipment.EquipSlot,
			str(e["weapon_type"]), _enum(Item.ItemType.keys(), type, 0) as Item.ItemType,
			DictUtil.to_int(e["base_power"]) if type == "WEAPON" else 0, _ints(e["required_stats"]), tiers,
			DictUtil.to_float(e["weight"]), DictUtil.to_int(e["value"]), rarity)
		if type == "WEAPON":
			eq.base_range = DictUtil.to_int(e["base_range"])
			eq.is_two_handed = DictUtil.to_bool(e["two_handed"])
			eq.scaling_stat = str(e["scaling_stat"])
		eq.off_hand_category = str(e["off_hand_category"])
		eq.block_cost = DictUtil.to_int(e["block_cost"])
		eq.burden = DictUtil.to_int(e["burden"])
		eq.mitigation = DictUtil.to_int(e["mitigation"])
		eq.stat_bonuses = _ints(e["stat_bonuses"])
		eq.effects = DictUtil.array_of(e["effects"]).duplicate(true)
		result = eq
	else:
		result = Item.new(str(e["id"]), str(e["name"]), _enum(Item.ItemType.keys(), type, Item.ItemType.MATERIAL) as Item.ItemType,
			DictUtil.to_float(e["weight"]), DictUtil.to_int(e["value"]), rarity, "", DictUtil.to_int(e["max_stack"]))
	result.description = ItemText.render(str(e["description"]), e)   # a {token} template renders with the resolved values
	result.damage_type = str(e["damage_type"])
	result.icon_id = str(e["icon_id"])
	result.special = DictUtil.to_bool(e["special"])
	result.special_note = str(e["special_note"])
	return result

static func item_to_entry(it: Item, starter_stash: bool = false) -> Dictionary:
	var e: Dictionary = {
		"id": it.id, "name": it.name, "item_type": _name(Item.ItemType.keys(), it.item_type),
		"rarity": _name(Item.ItemRarity.keys(), it.rarity), "weight": it.weight, "value": it.value_gold,
		"description": it.description, "damage_type": it.damage_type, "icon_id": it.icon_id,
		"starter_stash": starter_stash, "max_stack": it.max_stack,
		"special": it.special, "special_note": it.special_note,
	}
	if it is Equipment:
		var eq: Equipment = it
		e.merge({
			"slot": _name(Equipment.EquipSlot.keys(), eq.slot), "weapon_type": eq.weapon_type,
			"base_power": eq.base_power, "base_range": eq.base_range, "two_handed": eq.is_two_handed,
			"off_hand_category": eq.off_hand_category, "scaling_stat": eq.scaling_stat,
			"block_cost": eq.block_cost, "burden": eq.burden, "mitigation": eq.mitigation,
			"required_stats": eq.required_stats, "stat_bonuses": eq.stat_bonuses, "threshold_tiers": eq.threshold_tiers,
			"effects": eq.effects,
		})
	elif it is Consumable:
		var con: Consumable = it
		e.merge({
			"consumable_type": _name(Consumable.ConsumableType.keys(), con.consumable_type),
			"heal": con.heal_amount, "damage": con.damage_amount, "buff_stat": con.buff_stat,
			"buff_pct": con.buff_percent, "duration": con.buff_duration_rounds, "use_blocks": con.block_cost,
			"use_reach": con.use_reach,
		})
	return ContentSchema.normalize("items", e)

# True when catalog item `entry` may sit in engine slot `slot_name` ("OFF_HAND", ...): the equip gate
# (Equipment.can_equip_in_slot). `keep_worn` accepts a plain main_1h weapon already worn in the off hand (saved gear
# is kept under strict hand typing; only NEW equips are gated, FORGE1_ITEMS 8.1).
static func item_fits_slot(entry: Dictionary, slot_name: String, keep_worn: bool = false) -> bool:
	var built: Item = item(entry)
	if not (built is Equipment):
		return false
	var eq: Equipment = built
	var slot: Equipment.EquipSlot = _enum(Equipment.EquipSlot.keys(), slot_name, 0) as Equipment.EquipSlot
	if keep_worn and slot == Equipment.EquipSlot.OFF_HAND and eq.hand() == Equipment.HAND_MAIN_1H:
		return true
	return eq.can_equip_in_slot(slot)

# ------------------------------------------------------------------------------
# Passives
# ------------------------------------------------------------------------------

# Applies the catalog passives `ids` to `c` (each once). Sources (world background,
# faction, quest, lore, items) decide which; classes and base units grant none.
static func apply_passives(c: Combatant, ids: Array[String]) -> void:
	for id: String in ids:
		var passive: Dictionary = ContentCatalog.get_entry("passives", id)
		if not passive.is_empty():
			Passives.apply(c, passive)

# ------------------------------------------------------------------------------
# Classes
# ------------------------------------------------------------------------------

static func class_archetype(entry: Dictionary) -> ClassArchetype:
	var e: Dictionary = ContentSchema.normalize("classes", entry)
	var c := ClassArchetype.new(str(e["id"]), str(e["name"]))
	c.description = str(e["description"])
	c.base_stats = _ints(e["base_stats"])
	c.growth_per_level = _ints(e["growth"])
	c.proficiency = DictUtil.dict_of(e["proficiency"]).duplicate(true)
	var path: Dictionary = DictUtil.dict_of(e["skill_path"])   # {family: level} in entry order (a list only when the converter is off: invalid)
	for fam: Variant in path:
		c.skill_path[str(fam)] = DictUtil.to_int(path[fam], 1)
	c.art = str(e["art"])
	c.ai_role = str(e["ai_role"])
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		if not str(e[slot]).is_empty():
			c.loadout[slot] = str(e[slot])
	if not _strings(e["pouch"]).is_empty():
		c.loadout["pouch"] = _strings(e["pouch"])
	return c

static func class_to_entry(c: ClassArchetype) -> Dictionary:
	var e: Dictionary = {
		"id": c.id, "name": c.display_name, "description": c.description, "art": c.art, "ai_role": c.ai_role,
		"base_stats": c.base_stats, "growth": c.growth_per_level, "proficiency": c.proficiency, "skill_path": c.skill_path.duplicate(),
	}
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		e[slot] = str(c.loadout.get(slot, ""))
	e["pouch"] = c.loadout.get("pouch", [])
	return ContentSchema.normalize("classes", e)

# ------------------------------------------------------------------------------
# Units
# ------------------------------------------------------------------------------

# Token / portrait key of a unit entry: its own art, else its class's, else a plain unit.
static func art_of(entry: Dictionary) -> String:
	var own: String = str(entry.get("art", ""))
	if not own.is_empty():
		return own
	var arch: ClassArchetype = class_of(entry)
	return arch.art if arch != null else "unit"

# The class a unit entry builds on (null for fixed / unknown).
static func class_of(e: Dictionary) -> ClassArchetype:
	var class_id: String = str(e.get("class_id", ""))
	if class_id.is_empty() or not ContentCatalog.has("classes", class_id):
		return null
	return ClassArchetype.create(class_id)

# A fresh Combatant for unit `entry`, named `display_name` (default: the entry's).
# Class-derived units (the default): class base stats + spend + growth of their level,
# the class's gear (per-slot overrides on the unit), the class skill path up to their
# level plus extra skills, and the tier's HP / power scale. Fixed units keep their
# authored vitals. AP comes from the entry, else `default_ap` (0 = Combatant default).
static func unit(entry: Dictionary, display_name: String = "", default_ap: int = 0) -> Combatant:
	var e: Dictionary = ContentSchema.normalize("units", entry)
	var shown: String = display_name if not display_name.is_empty() else str(e["name"])
	var arch: ClassArchetype = class_of(e)
	var derived: bool = str(e["stat_mode"]) == "derived" and arch != null
	var level: int = DictUtil.to_int(e["level"], 1)
	var c: Combatant
	if derived:
		var spend: Dictionary = _ints(e["spend"])
		if spend.is_empty() and not DictUtil.to_bool(entry.get(NO_AUTO_SPEND, false)):
			spend = StatRules.auto_spend(arch.growth_per_level, StatRules.spend_pool(level))
		c = Combatant.new(shown, 1, 1, 1, CharacterStats.new(arch, level, spend))
	elif arch != null:
		c = Combatant.new(shown, DictUtil.to_int(e["hp"]), DictUtil.to_int(e["speed"]), DictUtil.to_int(e["attack_range"]), CharacterStats.new(arch, level))
	else:
		c = Combatant.new(shown, DictUtil.to_int(e["hp"]), DictUtil.to_int(e["speed"]), DictUtil.to_int(e["attack_range"]), CharacterStats.new())
	_equip(c, e, arch)
	if derived:
		c.tier = str(e["tier"])
		c.hp_scale = StatRules.tier_hp(c.tier)
		c.power_scale = StatRules.tier_power(c.tier)
		c.sync_from_stats_and_gear()
		c.hp = c.max_hp
		var extras: Array[String] = []
		for extra: String in _strings(e["extra_skills"]):
			if not extras.has(extra):
				extras.append(extra)
		c.known_skills = extras
		c.set_class_path(arch.path_steps())   # owned path skills by level, KNOW and passives (Combatant.refresh_path_skills)
		for basic_id: String in SkillCatalog.basic_ids():   # every unit owns every basic skill from level 1 (FORGE1 11.2)
			if not c.known_skills.has(basic_id):
				c.known_skills.append(basic_id)
		var slots: Dictionary = loadout_of(e, arch, level, c.known_skills)
		c.set_loadout(_strings(slots["class"]), _strings(slots["extra"]))
	var ap: int = DictUtil.to_int(e["ap"]) if DictUtil.to_int(e["ap"]) > 0 else default_ap
	if ap > 0:
		c.max_ap = ap
		c.ap = ap
	c.power_override = DictUtil.to_int(e["power"])
	c.armor_override = DictUtil.to_int(e["armor"])
	var resist: Dictionary = _ints(e["status_resist"])
	for status: Variant in resist:
		c.status_resist[status] = resist[status]
	c.catalog_id = str(e["id"])
	return c

# Non-basic skill families of `skill_ids`, first occurrence order.
static func families_of(skill_ids: Array[String]) -> Array[String]:
	var out: Array[String] = []
	for id: String in skill_ids:
		var fam: String = SkillCatalog.family_of(id)
		if not SkillCatalog.is_always_active(id) and not out.has(fam):
			out.append(fam)
	return out

# The active loadout of unit entry `e`: {"class": [families], "extra": [families]}.
# Default: the first MAX_CLASS_SLOTS class-path families reached by `level`, ordered by (unlock level, entry position)
# (ClassArchetype.path_steps, FORGE1_CLASSES 3.5), then the extra skills' families (MAX_EXTRA_SLOTS). A `loadout` list
# overrides it (families on the class path are class slots, others extra slots); it is validated by ContentCatalog, not trimmed here.
static func loadout_of(e: Dictionary, arch: ClassArchetype, level: int, owned_ids: Array[String] = []) -> Dictionary:
	var path_all: Array[String] = []
	for step: Dictionary in arch.path_steps():
		path_all.append(str(step.get("skill", "")))
	var path_families: Array[String] = families_of(path_all)
	var listed: Array[String] = _strings(e.get("loadout", []))
	var class_out: Array[String] = []
	var extra_out: Array[String] = []
	if not listed.is_empty():
		for fam: String in listed:
			if path_families.has(fam):
				class_out.append(fam)
			else:
				extra_out.append(fam)
		return {"class": class_out, "extra": extra_out}
	# Owned path families in path order (the unit's real ownership when given, else the unreduced path levels).
	var owned_path: Array[String] = []
	if owned_ids.is_empty():
		owned_path = arch.skills_at(level)
	else:
		for id: String in path_all:
			if owned_ids.has(id):
				owned_path.append(id)
	for fam: String in families_of(owned_path):
		if class_out.size() < StatRules.MAX_CLASS_SLOTS:
			class_out.append(fam)
	for fam: String in families_of(_strings(e.get("extra_skills", []))):
		if not class_out.has(fam) and not path_families.has(fam) and extra_out.size() < StatRules.MAX_EXTRA_SLOTS:
			extra_out.append(fam)
	return {"class": class_out, "extra": extra_out}

# Equips the class's default gear, with per-slot overrides from the unit entry:
# "" = the class's item, ContentSchema.NO_ITEM = the slot stays empty (no class fallback);
# pouch [] = the class pouch, [NO_ITEM] = no consumables.
static func _equip(c: Combatant, e: Dictionary, arch: ClassArchetype) -> void:
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		var item_id: String = str(e[slot])
		if item_id == ContentSchema.NO_ITEM:
			continue
		if item_id.is_empty() and arch != null:
			item_id = str(arch.loadout.get(slot, ""))
		if item_id.is_empty() or not ContentCatalog.has("items", item_id):
			continue
		var built: Item = item(ContentCatalog.get_entry("items", item_id))
		if built is Equipment:
			var eq: Equipment = built
			c.inventory.equip(eq, _enum(Equipment.EquipSlot.keys(), slot.to_upper(), 0))
	var pouch: Array[String] = _strings(e["pouch"])
	if pouch.has(ContentSchema.NO_ITEM):
		return
	if pouch.is_empty() and arch != null:
		pouch = _strings(arch.loadout.get("pouch", []))
	for item_id: String in pouch:
		if ContentCatalog.has("items", item_id):
			var pouch_item: Item = item(ContentCatalog.get_entry("items", item_id))
			if pouch_item is Consumable:
				var con: Consumable = pouch_item
				c.inventory.add_to_battle_pouch(con)

# How unit `entry` plans its rounds: its AI role (or the class's), thresholds and skill cap.
static func ai_profile(entry: Dictionary, _c: Combatant) -> AiProfile:
	var e: Dictionary = ContentSchema.normalize("units", entry)
	var arch: ClassArchetype = class_of(e)
	var role_name: String = str(e["ai_role"]) if not str(e["ai_role"]).is_empty() else (arch.ai_role if arch != null else "AGGRESSOR")
	var role: AiProfile.Role = _enum(AiProfile.Role.keys(), role_name, 0) as AiProfile.Role
	var cap: int = DictUtil.to_int(e["skill_cap"])
	if cap <= 0 and arch != null:
		var tier_cap: int = StatRules.tier_skill_cap(str(e["tier"]))   # BAL3 section 5: minion-tier default ceiling
		cap = tier_cap if tier_cap > 0 else AiProfile.NO_CEILING   # else no AP ceiling: the unlock levels gate (Combatant.tier_unlocked)
	elif cap <= 0:
		cap = 1
	var profile: AiProfile = AiProfile.create(role, DictUtil.to_float(e["flee_ratio"]), cap)
	profile.defend_below = DictUtil.to_float(e["defend_below"])
	profile.prefer_level = DictUtil.to_int(e["prefer_level"])
	profile.reckless = DictUtil.to_bool(e["reckless"])
	profile.heal_below = DictUtil.to_float(e["heal_below"], AiProfile.DEFAULT_HEAL_BELOW)
	if str(e["target_pref"]) != "ROLE":
		profile.target_pref = _enum(AiProfile.TargetPref.keys(), str(e["target_pref"]), 0) as AiProfile.TargetPref
	return profile
