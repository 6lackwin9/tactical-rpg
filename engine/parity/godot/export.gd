extends SceneTree

# Aegis parity exporter (engine/docs/ENGINE_SPEC.md section 6). Read-only on the Godot project: builds units and hits
# through the shipped code and writes what it computed as JSON for the Rust parity tests. New file; nothing in the
# Godot project calls it.
#   godot --headless --path <project> -s <abs path to this file> -- out=<abs dir> [only=catalog,units,damage]
# Files: catalog.json (resolved entries per kind), units.json (unit snapshots), damage.json (DamageCalculator cases).

const KINDS: Array[String] = ["classes", "units", "skills", "items", "passives", "encounters", "terrain"]
const TIERS: Array[String] = ["hero", "minion", "standard", "elite", "boss"]
const STANCES: Array[int] = [0, 1, 2, 3]   # CombatTypes.StanceType NEUTRAL, OFFENSIVE, DEFENSIVE, TACTICAL
const SLOTS: Array[String] = ["MAIN_HAND", "OFF_HAND", "HEAD", "BODY", "ACCESSORY_1", "ACCESSORY_2"]

var out_dir: String = ""
var only: Array[String] = ["catalog", "units", "damage"]

func _initialize() -> void:
	for arg: String in OS.get_cmdline_user_args():
		if arg.begins_with("out="):
			out_dir = arg.substr(4)
		elif arg.begins_with("only="):
			only.assign(arg.substr(5).split(","))
	if out_dir.is_empty():
		print("§ export: missing out=<dir>")
		quit(1)
		return
	DirAccess.make_dir_recursive_absolute(out_dir)
	if only.has("catalog"):
		_write("catalog.json", _catalog())
	var units: Array[Dictionary] = _unit_specs()
	if only.has("units"):
		var rows: Array = []
		for spec: Dictionary in units:
			rows.append(_snapshot(spec))
		_write("units.json", {"units": rows})
	if only.has("damage"):
		_write("damage.json", {"cases": _damage(units)})
	quit()

func _write(file_name: String, data: Variant) -> void:
	var path: String = out_dir.path_join(file_name)
	var f: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if f == null:
		print("§ export: cannot write ", path)
		return
	f.store_string(JSON.stringify(data, "", false))
	f.close()
	print("§ export: wrote ", path)

func _catalog() -> Dictionary:
	var out: Dictionary = {}
	for kind: String in KINDS:
		out[kind] = ContentCatalog.entries(kind)
	return out

# Every catalog unit, plus each class at every level (hero tier, default spend) and at every tier at level 5.
func _unit_specs() -> Array[Dictionary]:
	var specs: Array[Dictionary] = []
	for id: String in ContentCatalog.ids("units"):
		specs.append({"key": "unit:" + id, "entry": ContentCatalog.get_entry("units", id)})
	for class_id: String in ContentCatalog.ids("classes"):
		for level: int in range(1, StatRules.MAX_LEVEL + 1):
			specs.append({"key": "class:%s:%d:hero" % [class_id, level], "entry": {"id": "parity_%s_%d" % [class_id, level], "name": class_id, "class_id": class_id, "level": level, "tier": "hero"}})
		for tier: String in TIERS:
			if tier != "hero":
				specs.append({"key": "class:%s:5:%s" % [class_id, tier], "entry": {"id": "parity_%s_5_%s" % [class_id, tier], "name": class_id, "class_id": class_id, "level": 5, "tier": tier}})
	return specs

func _snapshot(spec: Dictionary) -> Dictionary:
	var entry: Dictionary = spec["entry"]
	var c: Combatant = ContentFactory.unit(entry)
	var ctx: Dictionary = c.get_weapon_context()
	var stats: Dictionary = {}
	var base: Dictionary = {}
	for s: String in StatRules.STATS:
		stats[s] = c.stats.get_effective_stat(s, ctx)
		base[s] = c.stats.get_base_stat(s)
	var gear: Dictionary = {}
	for i: int in range(SLOTS.size()):
		var eq: Equipment = c.inventory.get_equipped(i as Equipment.EquipSlot)
		if eq != null:
			gear[SLOTS[i]] = eq.id
	var pouch: Array = []
	for con: Consumable in c.inventory.battle_pouch:
		pouch.append(con.id if con != null else "")
	var ai: AiProfile = ContentFactory.ai_profile(entry, c)
	return {
		"key": spec["key"], "entry": entry, "name": c.name, "catalog_id": c.catalog_id,
		"level": c.stats.level, "tier": c.tier, "hp_scale": c.hp_scale, "power_scale": c.power_scale,
		"base_stats": base, "stats": stats, "unallocated": c.stats.unallocated_attribute_points,
		"max_hp": c.max_hp, "speed": c.speed, "attack_range": c.attack_range, "min_attack_range": c.min_attack_range(),
		"max_ap": c.max_ap, "move_per_block": c.move_per_block(), "worn_burden": c.worn_burden(),
		"power_override": c.power_override, "armor_override": c.armor_override,
		"power_melee": DamageCalculator.power_of(c, false, false), "power_ranged": DamageCalculator.power_of(c, true, false),
		"power_offhand": DamageCalculator.power_of(c, false, true),
		"armor_rating": DamageCalculator.armor_rating(c), "crit_chance": DamageCalculator.crit_chance(c),
		"trained_fists": DamageCalculator.trained_fists(c), "debuff_resistance": c.debuff_resistance(),
		"gear": gear, "pouch": pouch, "known_skills": c.known_skills,
		"active_class_families": c.active_class_families, "active_extra_families": c.active_extra_families,
		"status_resist": c.status_resist,
		"ai": {"role": AiProfile.Role.keys()[ai.role], "skill_cap": ai.skill_cap, "flee_ratio": ai.flee_ratio,
			"defend_below": ai.defend_below, "target_pref": AiProfile.TargetPref.keys()[ai.target_pref],
			"prefer_level": ai.prefer_level, "reckless": ai.reckless, "heal_below": ai.heal_below},
	}

# DamageCalculator.compute over attacker x target pairs: hand, range, crit, skill power, guard and cover, plus the
# stance matrix on a few pairs. No RNG: crits are forced on or off.
func _damage(specs: Array[Dictionary]) -> Array:
	var pool: Array[Dictionary] = []
	for spec: Dictionary in specs:
		var key: String = str(spec["key"])
		if key.begins_with("unit:") or key.ends_with(":1:hero") or key.ends_with(":5:hero") or key.ends_with(":10:hero") or key.ends_with(":5:boss"):
			pool.append(spec)
	var built: Array[Combatant] = []
	for spec: Dictionary in pool:
		built.append(ContentFactory.unit(spec["entry"]))
	var cases: Array = []
	for a: int in range(pool.size()):
		for t: int in range(pool.size()):
			var attacker: Combatant = built[a]
			var target: Combatant = built[t]
			for flags: int in range(8):
				var ranged: bool = flags & 1 != 0
				var offhand: bool = flags & 2 != 0
				var crit: bool = flags & 4 != 0
				cases.append(_case(pool[a]["key"], pool[t]["key"], attacker, target, ranged, offhand, 1.0, crit, -1, 0.0, 0.0))
			cases.append(_case(pool[a]["key"], pool[t]["key"], attacker, target, false, false, 1.75, false, -1, 0.3, 0.0))
			cases.append(_case(pool[a]["key"], pool[t]["key"], attacker, target, true, false, 0.6, false, -1, 0.5, 0.5))
	var few: int = mini(pool.size(), 6)
	for a: int in range(few):
		for t: int in range(few):
			for sa: int in STANCES:
				for sd: int in STANCES:
					built[t].current_stance = sd as CombatTypes.StanceType
					cases.append(_case(pool[a]["key"], pool[t]["key"], built[a], built[t], false, false, 1.0, false, sa, 0.0, 0.0))
			built[t].current_stance = CombatTypes.StanceType.NEUTRAL
	return cases

func _case(ak: String, tk: String, attacker: Combatant, target: Combatant, ranged: bool, offhand: bool, skill_mult: float,
		crit: bool, attacker_stance: int, guard: float, cover: float) -> Dictionary:
	target.defense_mitigation_ratio = guard
	var res: Dictionary = DamageCalculator.compute(attacker, target, ranged, offhand, skill_mult, crit, null, attacker_stance, true, cover)
	target.defense_mitigation_ratio = 0.0
	return {
		"attacker": ak, "target": tk, "ranged": ranged, "offhand": offhand, "skill_mult": skill_mult, "crit": crit,
		"attacker_stance": attacker_stance, "defender_stance": int(target.current_stance), "guard": guard, "cover": cover,
		"attack_power": res["attack_power"], "armor_rating": res["armor_rating"], "final_dmg": res["final_dmg"],
	}
