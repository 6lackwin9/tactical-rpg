class_name WorldFactory
extends RefCounted

# ==============================================================================
# Tactical RPG - World Factory (world entries -> playable units & items)
# A world character is a base catalog unit wearing lore: the factory takes the
# base unit's entry, swaps in the character's name and art, spawns it through
# ContentFactory (so vitals, class, loadout, skills and AI are the base's), then
# applies the character's explicit extras: named items, disciplines and tuning
# deltas. With none of those, a world character plays EXACTLY like its base unit.
# ==============================================================================

# The unit entry (ContentCatalog "units" shape) a world character spawns from.
static func unit_entry(character: Dictionary) -> Dictionary:
	var entry: Dictionary = ContentCatalog.get_entry("units", str(character.get("base_unit", "")))
	if entry.is_empty():
		return {}
	entry["name"] = str(character.get("name", entry.get("name", "")))
	var art: String = str(character.get("art", ""))
	if not art.is_empty():
		entry["art"] = art
	var role: String = str(character.get("role", ""))
	if role == "party" or role == "enemy":
		entry["side"] = role
	entry["description"] = str(character.get("background", entry.get("description", "")))
	return entry

# A named item as a playable Item: the base item with the item's name, description
# and its tuning deltas / extra stat bonuses applied.
static func named_item(item_entry: Dictionary) -> Item:
	var base: Dictionary = ContentCatalog.get_entry("items", str(item_entry.get("base_item", "")))
	if base.is_empty():
		return null
	var tuning: Dictionary = {}
	var raw_tuning: Variant = item_entry.get("tuning", {})
	if raw_tuning is Dictionary:
		tuning = raw_tuning
	base["name"] = str(item_entry.get("name", base.get("name", "")))
	var story: String = str(item_entry.get("description", ""))
	if not story.is_empty():
		base["description"] = story
	base["base_power"] = maxi(0, DictUtil.to_int(base.get("base_power", 0)) + DictUtil.to_int(tuning.get("power", 0)))
	base["mitigation"] = maxi(0, DictUtil.to_int(base.get("mitigation", 0)) + DictUtil.to_int(tuning.get("mitigation", 0)))
	base["burden"] = maxi(0, DictUtil.to_int(base.get("burden", 0)) + DictUtil.to_int(tuning.get("burden", 0)))
	var bonuses: Dictionary = {}
	var base_bonus: Variant = base.get("stat_bonuses", {})
	if base_bonus is Dictionary:
		bonuses = DictUtil.dict_of(base_bonus).duplicate()
	var extra: Variant = item_entry.get("stat_bonuses", {})
	if extra is Dictionary:
		var extra_dict: Dictionary = extra
		for stat: Variant in extra_dict:
			bonuses[str(stat)] = DictUtil.to_int(bonuses.get(str(stat), 0)) + DictUtil.to_int(extra_dict[stat])
	base["stat_bonuses"] = bonuses
	base["id"] = "%s__%s" % [str(base.get("id", "item")), str(item_entry.get("_qid", item_entry.get("id", ""))).replace(":", "_")]
	return ContentFactory.item(base)

# Applies a character's named items, inherited passives and tuning to its freshly spawned unit.
# `world_id` is the world the character is played in (refs resolve from there).
static func apply_character(c: Combatant, character: Dictionary, world_id: String) -> void:
	c.lore_ref = str(character.get("_qid", character.get("id", "")))
	var geared: bool = false
	var refs_raw: Variant = character.get("named_items", [])
	var item_refs: Array = refs_raw if refs_raw is Array else []
	for ref: Variant in item_refs:
		var item_entry: Dictionary = WorldCatalog.resolve(world_id, "items", str(ref))
		if item_entry.is_empty():
			continue
		var worn: Item = named_item(item_entry)
		if worn is Equipment:
			var eq: Equipment = worn
			c.inventory.equip(eq, eq.slot)
			geared = true
	ContentFactory.apply_passives(c, character_passives(character, world_id))
	# Only class-derived units take their vitals from stats and gear; fixed units keep theirs.
	var base_unit: Dictionary = ContentCatalog.get_entry("units", str(character.get("base_unit", "")))
	if geared and c.stats != null and str(base_unit.get("stat_mode", "fixed")) == "derived":
		c.sync_from_stats_and_gear()
		c.hp = c.max_hp
	apply_tuning(c, character.get("tuning", {}))

# The base passive ids a character inherits, once each: its own, its faction's, its
# disciplines' and its named items' (the lore sources; classes and base units grant none).
static func character_passives(character: Dictionary, world_id: String) -> Array[String]:
	var out: Array[String] = []
	_add_ids(out, character.get("passives", []))
	var faction: Dictionary = WorldCatalog.resolve(world_id, "factions", str(character.get("faction", "")))
	_add_ids(out, faction.get("grants", []))
	for ref: Variant in DictUtil.array_of(character.get("disciplines", [])):
		_add_ids(out, WorldCatalog.resolve(world_id, "disciplines", str(ref)).get("passives", []))
	for ref: Variant in DictUtil.array_of(character.get("named_items", [])):
		_add_ids(out, WorldCatalog.resolve(world_id, "items", str(ref)).get("grants", []))
	return out

static func _add_ids(out: Array[String], raw: Variant) -> void:
	for id: Variant in DictUtil.array_of(raw):
		if not out.has(str(id)):
			out.append(str(id))

# Explicit gameplay deltas vs the base unit (see WorldSchema.TUNING_KEYS).
static func apply_tuning(c: Combatant, raw: Variant) -> void:
	if not (raw is Dictionary):
		return
	var tuning: Dictionary = raw
	var hp_pct: int = DictUtil.to_int(tuning.get("hp_pct", 0))
	if hp_pct != 0:
		c.max_hp = maxi(1, roundi(float(c.max_hp) * (1.0 + float(hp_pct) / 100.0)))
		c.hp = c.max_hp
	var power: int = DictUtil.to_int(tuning.get("power", 0))
	if power != 0:
		c.power_override = maxi(1, DamageCalculator.power_of(c, c.attack_range > 1) + power)
	var armor: int = DictUtil.to_int(tuning.get("armor", 0))
	if armor != 0:
		c.armor_override = maxi(0, DamageCalculator.armor_rating(c) + armor)
	var speed: int = DictUtil.to_int(tuning.get("speed", 0))
	if speed != 0:
		c.speed = maxi(1, c.speed + speed)
	var ap: int = DictUtil.to_int(tuning.get("ap", 0))
	if ap != 0:
		c.max_ap = maxi(1, c.max_ap + ap)
		c.ap = c.max_ap

# Human-readable list of what a character changes vs its base unit ("" = nothing).
static func divergence(character: Dictionary, world_id: String) -> String:
	var parts: Array[String] = []
	var tuning_raw: Variant = character.get("tuning", {})
	if tuning_raw is Dictionary:
		var tuning: Dictionary = tuning_raw
		for key: Variant in tuning:
			if DictUtil.to_int(tuning[key]) != 0:
				parts.append("%s %+d%s" % [str(key).trim_suffix("_pct"), DictUtil.to_int(tuning[key]), "%" if str(key).ends_with("_pct") else ""])
	var items_raw: Variant = character.get("named_items", [])
	if items_raw is Array and not DictUtil.array_of(items_raw).is_empty():
		var names: Array[String] = []
		for ref: Variant in DictUtil.array_of(items_raw):
			var it: Dictionary = WorldCatalog.resolve(world_id, "items", str(ref))
			names.append(str(it.get("name", str(ref))))
		parts.append("wears " + ", ".join(names))
	var disc_raw: Variant = character.get("disciplines", [])
	if disc_raw is Array and not DictUtil.array_of(disc_raw).is_empty():
		var dnames: Array[String] = []
		for ref: Variant in DictUtil.array_of(disc_raw):
			var d: Dictionary = WorldCatalog.resolve(world_id, "disciplines", str(ref))
			dnames.append(str(d.get("name", str(ref))))
		parts.append("trained: " + ", ".join(dnames))
	var inherited: Array[String] = character_passives(character, world_id)
	if not inherited.is_empty():
		var pnames: Array[String] = []
		for id: String in inherited:
			pnames.append(str(ContentCatalog.get_entry("passives", id).get("name", id)))
		parts.append("passives: " + ", ".join(pnames))
	return "; ".join(parts)
