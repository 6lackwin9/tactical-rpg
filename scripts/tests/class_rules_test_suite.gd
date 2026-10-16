class_name ClassRulesTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Class & Unit Rules Test Suite
# The enforced base rules: the six classes share one stat budget and growth, units
# differ only by their spend pool, enemies use the same classes at a tier, proficiency
# is an editable dict with real effects, passives come from sources (never the class),
# and class skill paths unlock by level with event-learned extras.
# ==============================================================================

const CLASSES: Array[String] = ["warrior", "ranger", "priest", "brawler", "rogue", "mage"]

const TAG: String = "Classes"

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING CLASS & UNIT RULES TEST                ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var ok: bool = true
	ok = _test_six_classes() and ok
	ok = _test_variance_and_tiers() and ok
	ok = _test_proficiency() and ok
	ok = _test_passives() and ok
	ok = _test_skill_paths() and ok
	return ok

static func _test_six_classes() -> bool:
	var checks: Dictionary = {}
	var ids: Array[String] = ContentCatalog.ids("classes")
	checks["exactly the six classes"] = ids.size() == CLASSES.size() and CLASSES.all(func(id: String) -> bool: return ids.has(id))
	var art_ok: bool = true
	var budget_ok: bool = true
	var growth_ok: bool = true
	var starts_ok: bool = true
	for id: String in CLASSES:
		var arch: ClassArchetype = ClassArchetype.create(id)
		budget_ok = budget_ok and StatRules.total_of(arch.base_stats) == StatRules.CLASS_BUDGET
		growth_ok = growth_ok and StatRules.total_of(arch.growth_per_level) == StatRules.GROWTH_TOTAL
		art_ok = art_ok and UnitArt.CLASS_KEYS.has(arch.art)
		starts_ok = starts_ok and not ContentFactory.unit({"id": "t_start", "name": "T", "class_id": id, "level": 1}).known_skills.is_empty() and not arch.loadout.is_empty()
		var level_one := CharacterStats.new(arch, 1)
		var level_five := CharacterStats.new(arch, 5)
		growth_ok = growth_ok and StatRules.total_of(level_five._core_stats) - StatRules.total_of(level_one._core_stats) == StatRules.GROWTH_TOTAL * 4
	checks["equal base budget"] = budget_ok
	checks["equal growth per level"] = growth_ok
	checks["every class has art"] = art_ok
	checks["every class has a starting skill set and gear"] = starts_ok
	# Loadouts sit inside the gear budget band (a documented special item may push it over); no class warnings.
	var band_ok: bool = true
	for entry: Dictionary in ContentCatalog.entries("classes"):
		band_ok = band_ok and (GearBudget.in_band(GearBudget.loadout_value(entry)) or GearBudget.has_documented_special(entry))
		band_ok = band_ok and ContentCatalog.warnings_for("classes", entry).is_empty()
	checks["loadouts inside the gear budget (or documented special)"] = band_ok
	# Role identity: the warrior is the tank (best armor rating and HP), the mage the INT caster, the priest supports.
	var tank_ok: bool = true
	var warrior_unit: Combatant = ContentFactory.unit({"id": "t_tank", "name": "T", "class_id": "warrior", "level": 5})
	for id: String in CLASSES:
		if id == "warrior":
			continue
		var other: Combatant = ContentFactory.unit({"id": "t_o_" + id, "name": "T", "class_id": id, "level": 5})
		tank_ok = tank_ok and DamageCalculator.armor_rating(warrior_unit) > DamageCalculator.armor_rating(other) and warrior_unit.max_hp >= other.max_hp
	checks["warrior has the best armor and HP"] = tank_ok
	checks["caster and support roles"] = ClassArchetype.create("mage").get_base_stat("INT") == 17 and ClassArchetype.create("priest").ai_role == "SUPPORT"
	# The rules reject drift.
	checks["off-budget class refused"] = not StatRules.validate_class({"STR": 18, "DEX": 18, "INT": 18, "CONS": 18, "PERC": 18, "KNOW": 18}, {"STR": 5}).is_empty()
	checks["growth over cap refused"] = not StatRules.validate_class({"STR": 12, "DEX": 10, "INT": 10, "CONS": 10, "PERC": 10, "KNOW": 10}, {"STR": 5}).is_empty()
	return TestHelpers.report_ok(TAG, "Six base classes: equal stat budget and growth, art, starting skills and gear; drift is refused", checks.values().all(func(v: Variant) -> bool: return DictUtil.to_bool(v)), TestHelpers.failed(checks))

static func _test_variance_and_tiers() -> bool:
	var checks: Dictionary = {}
	var arch: ClassArchetype = ClassArchetype.create("warrior")
	var pool: int = StatRules.spend_pool(3)
	var tank := CharacterStats.new(arch, 3, {"CONS": pool})
	var hitter := CharacterStats.new(arch, 3, {"STR": pool})
	checks["same class differs by spend"] = tank.get_base_stat("CONS") > hitter.get_base_stat("CONS") and hitter.get_base_stat("STR") > tank.get_base_stat("STR")
	checks["same total either way"] = StatRules.total_of(tank._core_stats) == StatRules.total_of(hitter._core_stats)
	checks["spend must equal the pool"] = not StatRules.validate_spend({"STR": 1}, 3).is_empty() and StatRules.validate_spend({"STR": pool}, 3).is_empty() and StatRules.validate_spend({}, 3).is_empty()
	checks["auto spend uses the whole pool"] = StatRules.total_of(StatRules.auto_spend(arch.growth_per_level, pool)) == pool
	# Enemies are the same classes at a tier: same stats, scaled HP and power.
	var hero_entry: Dictionary = {"id": "t_hero", "name": "T", "class_id": "brawler", "level": 3, "tier": "hero"}
	var minion_entry: Dictionary = {"id": "t_minion", "name": "T", "class_id": "brawler", "level": 3, "tier": "minion"}
	var boss_entry: Dictionary = {"id": "t_boss", "name": "T", "class_id": "brawler", "level": 3, "tier": "boss"}
	var hero: Combatant = ContentFactory.unit(hero_entry)
	var minion: Combatant = ContentFactory.unit(minion_entry)
	var boss: Combatant = ContentFactory.unit(boss_entry)
	checks["tier keeps class stats"] = hero.stats.get_base_stat("STR") == minion.stats.get_base_stat("STR") and hero.stats.get_base_stat("CONS") == boss.stats.get_base_stat("CONS")
	checks["tier scales hp"] = minion.max_hp < hero.max_hp and hero.max_hp < boss.max_hp
	checks["tier scales power"] = DamageCalculator.power_of(minion) < DamageCalculator.power_of(hero) and DamageCalculator.power_of(hero) < DamageCalculator.power_of(boss)
	var minion_weapon: Equipment = minion.inventory.get_main_weapon()
	var hero_weapon: Equipment = hero.inventory.get_main_weapon()
	var same_weapon: bool = (minion_weapon == null and hero_weapon == null) or (minion_weapon != null and hero_weapon != null and minion_weapon.id == hero_weapon.id)
	checks["same class, same gear and skills"] = minion.known_skills == hero.known_skills and same_weapon
	return TestHelpers.report_ok(TAG, "Unit variance (spend pool) and tiers: one base curve, scaled HP and power", checks.values().all(func(v: Variant) -> bool: return DictUtil.to_bool(v)), TestHelpers.failed(checks))

static func _test_proficiency() -> bool:
	var checks: Dictionary = {}
	var prof: Proficiency = Proficiency.from_dict({"sword": {"level": 3, "scaling": 1.8}, "bow": {"level": 1, "scaling": 1.2}, "plate": {"level": 2, "scaling": 1.5}})
	checks["dict keys are gear types"] = prof.level_of("sword") == 3 and prof.can_use("bow") and not prof.can_use("axe")
	checks["scaling per gear"] = is_equal_approx(prof.scaling_of("sword"), 1.8) and is_equal_approx(prof.scaling_of("bow"), 1.2)
	checks["power per level"] = prof.get_weapon_power_bonus("sword") == 3 and prof.get_weapon_power_bonus("bow") == 1
	checks["requirement discount"] = prof.get_weapon_requirement_discount("sword") == 6
	checks["crit above level 1"] = is_equal_approx(prof.crit_bonus("sword"), 2.0) and is_equal_approx(prof.crit_bonus("bow"), 0.0)
	checks["armor burden cut"] = prof.burden_reduction("plate") == 1 and prof.burden_reduction("leather") == 0
	checks["master block cost"] = prof.block_cost_reduction("sword") == 1 and prof.block_cost_reduction("bow") == 0
	# Scaling really changes damage: a higher scaling gives a higher Unit Power.
	var arch: ClassArchetype = ClassArchetype.create("warrior")
	var low: Combatant = ContentFactory.unit({"id": "t_low", "name": "T", "class_id": "warrior", "level": 1, "main_hand": "wpn_broadsword"})
	var high: Combatant = ContentFactory.unit({"id": "t_high", "name": "T", "class_id": "warrior", "level": 1, "main_hand": "wpn_broadsword"})
	high.stats.proficiency.set_entry("sword", 3, 2.0)
	low.stats.proficiency.set_entry("sword", 3, 1.0)
	checks["scaling changes unit power"] = DamageCalculator.power_of(high) > DamageCalculator.power_of(low)
	# Gear the class has no training in can't be equipped.
	var greataxe := Equipment.new("t_axe", "Axe", Equipment.EquipSlot.MAIN_HAND, "axe", Item.ItemType.WEAPON, 20, {})
	var priest: ClassArchetype = ClassArchetype.create("priest")
	checks["untrained gear refused"] = arch.can_equip_gear(greataxe) and not priest.can_equip_gear(greataxe)
	# Skills that need training are gated.
	var aimed: SkillDef = SkillCatalog.find("aimed_shot")
	var archer: Combatant = ContentFactory.unit({"id": "t_arch", "name": "T", "class_id": "ranger", "level": 5})
	checks["trained skill usable"] = archer.can_use(aimed)
	archer.stats.proficiency.set_entry("bow", 1)
	checks["under-trained skill gated"] = not archer.can_use(aimed) and archer.can_use(SkillCatalog.find("strike"))
	return TestHelpers.report_ok(TAG, "Proficiency dict: levels, scaling, requirement discount, crit, burden, block cost, skill gating", checks.values().all(func(v: Variant) -> bool: return DictUtil.to_bool(v)), TestHelpers.failed(checks))

static func _test_passives() -> bool:
	var checks: Dictionary = {}
	for id: String in ContentCatalog.ids("classes"):
		var arch: ClassArchetype = ClassArchetype.create(id)
		checks["class %s grants no passive" % id] = not arch.proficiency.has("passives") and not ContentFactory.unit({"id": "t_p", "name": "T", "class_id": id, "level": 1}).passives.size() > 0
	var unit: Combatant = ContentFactory.unit({"id": "t_pass", "name": "T", "class_id": "rogue", "level": 1})
	var base_regen: int = unit.ap_regen_bonus
	var bow_before: int = unit.stats.proficiency.level_of("bow")
	var know_before: int = unit.stats.get_effective_stat("KNOW", {"biome": "ruins"})
	ContentFactory.apply_passives(unit, ["second_wind", "bow_training", "steady_nerves", "ruin_lore", "dirty_tricks", "second_wind"])
	checks["ap_regen passive"] = unit.ap_regen_bonus == base_regen + 1
	checks["proficiency passive"] = unit.stats.proficiency.level_of("bow") == mini(Proficiency.MAX_LEVEL, bow_before + 1)
	checks["resist passive"] = DictUtil.int_of(unit.status_resist, "stagger") == 25
	checks["contextual stat passive"] = unit.stats.get_effective_stat("KNOW", {"biome": "ruins"}) == know_before + 3 and unit.stats.get_effective_stat("KNOW") == unit.stats.get_base_stat("KNOW")
	checks["skill passive"] = unit.granted_skills.has("quick_stab")
	checks["each passive applies once"] = unit.passives.count("second_wind") == 1
	var untrained: Combatant = ContentFactory.unit({"id": "t_pass2", "name": "T", "class_id": "priest", "level": 1})
	ContentFactory.apply_passives(untrained, ["axe_training"])
	checks["passive can open new gear"] = untrained.stats.proficiency.can_use("axe") and untrained.stats.proficiency.scaling_of("axe") < Proficiency.DEFAULT_SCALING
	return TestHelpers.report_ok(TAG, "Passives: stat (with context), proficiency, resist, AP regen, skill; applied once, never from classes", checks.values().all(func(v: Variant) -> bool: return DictUtil.to_bool(v)), TestHelpers.failed(checks))

static func _test_skill_paths() -> bool:
	var checks: Dictionary = {}
	var low: Combatant = ContentFactory.unit({"id": "t_s1", "name": "T", "class_id": "warrior", "level": 1})
	var mid: Combatant = ContentFactory.unit({"id": "t_s5", "name": "T", "class_id": "warrior", "level": 5})
	checks["level 1 knows the opening skills"] = low.knows_skill("strike") and low.knows_skill("guard") and not low.knows_skill("shield_bash")
	checks["path unlocks by level"] = mid.knows_skill("shield_bash") and mid.known_skills.size() > low.known_skills.size()
	checks["system skills always known"] = low.knows_skill("rest") and low.knows_skill("move")
	checks["event learning adds a skill"] = not low.knows_skill("hamstring") and (func() -> bool:
		low.learn_skill("hamstring")
		return low.knows_skill("hamstring")).call()
	var extra: Combatant = ContentFactory.unit({"id": "t_s3", "name": "T", "class_id": "ranger", "level": 1, "extra_skills": ["hamstring"]})
	checks["authored extra skills"] = extra.knows_skill("hamstring")
	# Every path is legal under the SK2 rules: advanced steps at ADVANCED_MIN_LEVEL+, one step per family, tier-1 rows only.
	var legal: bool = true
	for id: String in CLASSES:
		var seen: Array[String] = []
		for step: Dictionary in ClassArchetype.create(id).skill_path:
			var skill: SkillDef = SkillCatalog.find(str(step["skill"]))
			if skill == null:
				legal = false
				continue
			legal = legal and skill.tier == 1 and not seen.has(skill.family_id())
			legal = legal and (skill.grade == SkillDef.Grade.NOVICE or DictUtil.to_int(step["level"]) >= StatRules.ADVANCED_MIN_LEVEL)
			seen.append(skill.family_id())
	checks["paths: advanced steps >= ADVANCED_MIN_LEVEL, one step per family, tier-1 rows only"] = legal
	# A unit owns exactly the path skills whose family unlock level (path level minus any reduction) it has reached.
	var owns_by_level: bool = true
	for id: String in CLASSES:
		for lvl: int in [1, 3, 5, 7, 10]:
			var unit: Combatant = ContentFactory.unit({"id": "t_sp_%s" % id, "name": "T", "class_id": id, "level": lvl})
			for step: Dictionary in ClassArchetype.create(id).skill_path:
				var sid: String = str(step["skill"])
				var eff: int = unit.family_unlock_level(SkillCatalog.family_of(sid))
				owns_by_level = owns_by_level and eff <= DictUtil.to_int(step["level"]) and (unit.known_skills.has(sid) == (lvl >= eff))
	checks["path ownership follows the effective unlock level (never later than the path level)"] = owns_by_level
	checks["planner honours known skills"] = SkillCatalog.single_attacks(low, 5).any(func(s: SkillDef) -> bool: return s.id == "strike") and not SkillCatalog.single_attacks(low, 5).any(func(s: SkillDef) -> bool: return s.id == "shield_bash")
	return TestHelpers.report_ok(TAG, "Class skill paths unlock by level; events and authoring add skills; the planner respects them", checks.values().all(func(v: Variant) -> bool: return DictUtil.to_bool(v)), TestHelpers.failed(checks))
