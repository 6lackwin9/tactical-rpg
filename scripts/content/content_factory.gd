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
	s.offhand = DictUtil.to_bool(e["offhand"])
	s.stance_override = -1 if str(e["stance"]) == "AUTO" else _enum(CombatTypes.StanceType.keys(), str(e["stance"]), -1)
	s.status_effect = str(e["status_effect"])
	s.status_potency = DictUtil.to_int(e["status_potency"])
	s.reach_min = DictUtil.to_int(e["reach_min"])
	s.reach_max = DictUtil.to_int(e["reach_max"])
	s.mitigation = DictUtil.to_float(e["mitigation"])
	s.boost = DictUtil.to_float(e["boost"])
	s.boost_rounds = DictUtil.to_int(e["boost_rounds"])
	s.recover_ap = DictUtil.to_int(e["recover_ap"])
	s.system = DictUtil.to_bool(e["system"])
	return s

static func skill_to_entry(s: SkillDef) -> Dictionary:
	return ContentSchema.normalize("skills", {
		"id": s.id, "name": s.display_name, "kind": _name(SkillDef.Kind.keys(), s.kind),
		"level": s.level, "blocks": s.blocks, "description": s.description, "system": s.system,
		"power_mult": s.power_mult, "area": _name(SkillDef.Area.keys(), s.area), "offhand": s.offhand,
		"stance": "AUTO" if s.stance_override < 0 else _name(CombatTypes.StanceType.keys(), s.stance_override),
		"status_effect": s.status_effect, "status_potency": s.status_potency,
		"reach_min": s.reach_min, "reach_max": s.reach_max, "mitigation": s.mitigation,
		"boost": s.boost, "boost_rounds": s.boost_rounds, "recover_ap": s.recover_ap,
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
		result = con
	elif ContentSchema.EQUIP_TYPES.has(type):
		var tiers: Array[Dictionary] = []
		var raw_tiers: Variant = e["threshold_tiers"]
		if raw_tiers is Array and type == "WEAPON":
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
		result = eq
	else:
		result = Item.new(str(e["id"]), str(e["name"]), _enum(Item.ItemType.keys(), type, Item.ItemType.MATERIAL) as Item.ItemType,
			DictUtil.to_float(e["weight"]), DictUtil.to_int(e["value"]), rarity, "", DictUtil.to_int(e["max_stack"]))
	result.description = str(e["description"])
	result.damage_type = str(e["damage_type"])
	result.icon_id = str(e["icon_id"])
	return result

static func item_to_entry(it: Item, starter_stash: bool = false) -> Dictionary:
	var e: Dictionary = {
		"id": it.id, "name": it.name, "item_type": _name(Item.ItemType.keys(), it.item_type),
		"rarity": _name(Item.ItemRarity.keys(), it.rarity), "weight": it.weight, "value": it.value_gold,
		"description": it.description, "damage_type": it.damage_type, "icon_id": it.icon_id,
		"starter_stash": starter_stash, "max_stack": it.max_stack,
	}
	if it is Equipment:
		var eq: Equipment = it
		e.merge({
			"slot": _name(Equipment.EquipSlot.keys(), eq.slot), "weapon_type": eq.weapon_type,
			"base_power": eq.base_power, "base_range": eq.base_range, "two_handed": eq.is_two_handed,
			"off_hand_category": eq.off_hand_category, "scaling_stat": eq.scaling_stat,
			"block_cost": eq.block_cost, "burden": eq.burden, "mitigation": eq.mitigation,
			"required_stats": eq.required_stats, "stat_bonuses": eq.stat_bonuses, "threshold_tiers": eq.threshold_tiers,
		})
	elif it is Consumable:
		var con: Consumable = it
		e.merge({
			"consumable_type": _name(Consumable.ConsumableType.keys(), con.consumable_type),
			"heal": con.heal_amount, "damage": con.damage_amount, "buff_stat": con.buff_stat,
			"buff_pct": con.buff_percent, "duration": con.buff_duration_rounds, "use_blocks": con.block_cost,
		})
	return ContentSchema.normalize("items", e)

# True when catalog item `entry` may sit in engine slot `slot_name` ("OFF_HAND", ...).
static func item_fits_slot(entry: Dictionary, slot_name: String) -> bool:
	var built: Item = item(entry)
	if not (built is Equipment):
		return false
	var eq: Equipment = built
	return eq.can_equip_in_slot(_enum(Equipment.EquipSlot.keys(), slot_name, 0) as Equipment.EquipSlot)

# ------------------------------------------------------------------------------
# Classes
# ------------------------------------------------------------------------------

static func class_archetype(entry: Dictionary) -> ClassArchetype:
	var e: Dictionary = ContentSchema.normalize("classes", entry)
	return ClassArchetype.new(str(e["id"]), str(e["name"]), _ints(e["base_stats"]), _ints(e["growth"]), _ints(e["masteries"]), _strings(e["allowed_gear"]))

static func class_to_entry(c: ClassArchetype, description: String = "") -> Dictionary:
	return ContentSchema.normalize("classes", {
		"id": c.id, "name": c.display_name, "description": description, "base_stats": c.base_stats,
		"growth": c.growth_per_level, "masteries": c.innate_masteries, "allowed_gear": c.allowed_gear_types,
	})

# ------------------------------------------------------------------------------
# Units
# ------------------------------------------------------------------------------

# A fresh Combatant for unit `entry`, named `display_name` (default: the entry's).
# AP comes from the entry, else `default_ap` (0 = Combatant default).
static func unit(entry: Dictionary, display_name: String = "", default_ap: int = 0) -> Combatant:
	var e: Dictionary = ContentSchema.normalize("units", entry)
	var shown: String = display_name if not display_name.is_empty() else str(e["name"])
	var class_id: String = str(e["class_id"])
	var stats: CharacterStats = null
	if not class_id.is_empty() and ContentCatalog.has("classes", class_id):
		stats = CharacterStats.new(class_archetype(ContentCatalog.get_entry("classes", class_id)), DictUtil.to_int(e["level"]))
	var derived: bool = str(e["stat_mode"]) == "derived"
	var c: Combatant
	if derived:
		c = Combatant.new(shown, 1, 1, 1, stats)
	else:
		c = Combatant.new(shown, DictUtil.to_int(e["hp"]), DictUtil.to_int(e["speed"]), DictUtil.to_int(e["attack_range"]), stats)
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		var item_id: String = str(e[slot])
		if item_id.is_empty() or not ContentCatalog.has("items", item_id):
			continue
		var built: Item = item(ContentCatalog.get_entry("items", item_id))
		if built is Equipment:
			var eq: Equipment = built
			c.inventory.equip(eq, _enum(Equipment.EquipSlot.keys(), slot.to_upper(), 0))
	for item_id: String in _strings(e["pouch"]):
		if ContentCatalog.has("items", item_id):
			var pouch_item: Item = item(ContentCatalog.get_entry("items", item_id))
			if pouch_item is Consumable:
				var con: Consumable = pouch_item
				c.inventory.add_to_battle_pouch(con)
	if derived:
		c.sync_from_stats_and_gear()
		c.hp = c.max_hp
	var ap: int = DictUtil.to_int(e["ap"]) if DictUtil.to_int(e["ap"]) > 0 else default_ap
	if ap > 0:
		c.max_ap = ap
		c.ap = ap
	c.power_override = DictUtil.to_int(e["power"])
	c.armor_override = DictUtil.to_int(e["armor"])
	c.known_skills = _strings(e["skills"])
	var resist: Dictionary = _ints(e["status_resist"])
	for status: Variant in resist:
		c.status_resist[status] = resist[status]
	c.catalog_id = str(e["id"])
	return c

# How unit `entry` (spawned as `c`) plans its rounds.
static func ai_profile(entry: Dictionary, _c: Combatant) -> AiProfile:
	var e: Dictionary = ContentSchema.normalize("units", entry)
	var role: AiProfile.Role = _enum(AiProfile.Role.keys(), str(e["ai_role"]), 0) as AiProfile.Role
	var profile: AiProfile = AiProfile.create(role, DictUtil.to_float(e["flee_ratio"]), DictUtil.to_int(e["skill_cap"]))
	profile.defend_below = DictUtil.to_float(e["defend_below"])
	profile.prefer_level = DictUtil.to_int(e["prefer_level"])
	if str(e["target_pref"]) != "ROLE":
		profile.target_pref = _enum(AiProfile.TargetPref.keys(), str(e["target_pref"]), 0) as AiProfile.TargetPref
	return profile
