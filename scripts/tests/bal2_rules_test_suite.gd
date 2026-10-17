class_name Bal2RulesTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - BAL2 Rules Test Suite (task BAL2-L)
# Commit 2: the PERC range bonus (+1 at RANGE_PERC_THRESHOLD) only for ranged weapons (base
# range > 1). Commit 3: the unit gear sentinel ContentSchema.NO_ITEM ("none") per slot and pouch.
# Commit 4: the elite / boss tier values. Units are built from the class rules (hero, default spend,
# class gear) or in memory; expectations derive from CharacterStats / DamageCalculator / StatRules.
# ==============================================================================

const TAG: String = "BAL2"
const MAX_TEST_LEVEL: int = 9

static var SANDBOX: String = TestHelpers.sandbox("bal2_rules_test")

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING BAL2 RULES TEST (BAL2-L)                ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var ok: bool = true
	ok = _test_range_rule() and ok
	ok = _test_range_examples() and ok
	ok = _test_unarmed_range() and ok
	ok = _test_melee_strike() and ok
	ok = _test_no_item_slots() and ok
	ok = _test_no_item_pouch() and ok
	ok = _test_no_item_validation() and ok
	ok = _test_all_none_unarmed() and ok
	ok = _test_unarmed_brawler_fixture() and ok
	ok = _test_tier_values() and ok
	ok = _test_trained_fists() and ok
	ok = _test_fist_edges() and ok
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	return ok

# ------------------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------------------

static func _hero(class_id: String, level: int, extra: Dictionary = {}) -> Combatant:
	var entry: Dictionary = {"id": "bal2_t_%s" % class_id, "name": "T %s" % class_id, "side": "party", "class_id": class_id, "level": level, "tier": "hero"}
	entry.merge(extra, true)
	return ContentFactory.unit(entry)

static func _base_range(c: Combatant) -> int:
	var w: Equipment = c.inventory.get_main_weapon() if c.inventory != null else null
	return w.base_range if w != null else 1

static func _perc(c: Combatant) -> int:
	return c.stats.get_effective_stat("PERC", c.get_weapon_context())

static func _expected_range(c: Combatant) -> int:
	var base: int = _base_range(c)
	return base + (1 if base > 1 and _perc(c) >= CharacterStats.RANGE_PERC_THRESHOLD else 0)

# The first level 1..MAX_TEST_LEVEL where `class_id` reaches (want_high) or stays below the PERC threshold; 0 = none.
static func _level_where(class_id: String, want_high: bool) -> int:
	for lvl: int in range(1, MAX_TEST_LEVEL + 1):
		var high: bool = _perc(_hero(class_id, lvl)) >= CharacterStats.RANGE_PERC_THRESHOLD
		if high == want_high:
			return lvl
	return 0

# ------------------------------------------------------------------------------
# Commit 2: PERC range bonus
# ------------------------------------------------------------------------------

static func _test_range_rule() -> bool:
	var checks: Dictionary = {}
	var classes: Array[String] = ContentCatalog.ids("classes")
	checks["the catalog has classes"] = not classes.is_empty()
	for class_id: String in classes:
		for lvl: int in range(1, MAX_TEST_LEVEL + 1):
			var c: Combatant = _hero(class_id, lvl)
			if c.attack_range != _expected_range(c):
				checks["%s L%d range %d = base %d + PERC bonus (PERC %d)" % [class_id, lvl, c.attack_range, _base_range(c), _perc(c)]] = false
	checks["CharacterStats: melee base 1 never gains range"] = CharacterStats.new().get_effective_range(1, {}) == 1
	return TestHelpers.report(TAG, "Range = base + 1 only when base > 1 and PERC >= threshold (every class, L1-9)", checks)

static func _test_range_examples() -> bool:
	var checks: Dictionary = {}
	for pair: Array in [["priest", 7], ["rogue", 7], ["warrior", 9]]:
		var class_id: String = str(pair[0])
		var lvl: int = DictUtil.to_int(pair[1])
		if not ContentCatalog.has("classes", class_id):
			continue
		var c: Combatant = _hero(class_id, lvl)
		if _base_range(c) == 1:
			checks["%s L%d (melee weapon, PERC %d) has range 1" % [class_id, lvl, _perc(c)]] = c.attack_range == 1
	for class_id: String in ["ranger", "mage"]:
		if not ContentCatalog.has("classes", class_id):
			continue
		var high: int = _level_where(class_id, true)
		var low: int = _level_where(class_id, false)
		if high > 0:
			var h: Combatant = _hero(class_id, high)
			checks["%s L%d (PERC %d, base %d) range base + 1" % [class_id, high, _perc(h), _base_range(h)]] = _base_range(h) > 1 and h.attack_range == _base_range(h) + 1
		if low > 0:
			var l: Combatant = _hero(class_id, low)
			checks["%s L%d (PERC %d) range = base" % [class_id, low, _perc(l)]] = l.attack_range == _base_range(l)
		checks["%s reaches the PERC threshold by L%d" % [class_id, MAX_TEST_LEVEL]] = high > 0
	return TestHelpers.report(TAG, "Melee classes at high PERC keep range 1; ranger / mage gain +1 at the threshold", checks)

static func _test_unarmed_range() -> bool:
	var checks: Dictionary = {}
	var lvl: int = MAX_TEST_LEVEL
	var c: Combatant = _hero("rogue", lvl, {"spend": {"PERC": StatRules.spend_pool(lvl)}})
	c.inventory.unequip(Equipment.EquipSlot.MAIN_HAND)
	c.sync_from_stats_and_gear()
	checks["no main weapon"] = c.inventory.get_main_weapon() == null
	checks["PERC %d >= threshold" % _perc(c)] = _perc(c) >= CharacterStats.RANGE_PERC_THRESHOLD
	checks["unarmed range 1"] = c.attack_range == 1
	return TestHelpers.report(TAG, "An unarmed unit with high PERC keeps range 1", checks)

static func _test_melee_strike() -> bool:
	var checks: Dictionary = {}
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
	var priest: Combatant = _hero("priest", 7)
	var foe := Combatant.new("Foe", 999, 10, 1)
	foe.power_override = 10
	foe.armor_override = 0
	grid.place_combatant(priest, Vector2i(3, 4))
	grid.place_combatant(foe, Vector2i(4, 4))
	var rng_a := RandomNumberGenerator.new()
	rng_a.seed = 7
	var rng_b := RandomNumberGenerator.new()
	rng_b.seed = 7
	var cmd := AttackCommand.new(priest, foe, 1.0, 1, 0, grid)
	cmd.rng = rng_a
	cmd.execute()
	var melee: int = DictUtil.to_int(DamageCalculator.compute(priest, foe, false, false, 1.0, false, rng_b)["final_dmg"])
	checks["priest L7 range 1"] = priest.attack_range == 1
	checks["strike damage %d = melee compute %d" % [cmd.last_damage, melee]] = cmd.last_damage == melee and melee > 0
	return TestHelpers.report(TAG, "Priest L7 strike on an adjacent foe resolves as a melee hit", checks)

# ------------------------------------------------------------------------------
# Commit 3: unit gear sentinel ContentSchema.NO_ITEM
# ------------------------------------------------------------------------------

static func _unit_entry(id: String, class_id: String, extra: Dictionary = {}) -> Dictionary:
	var entry: Dictionary = {"id": id, "name": id, "side": "party", "class_id": class_id, "level": 5, "tier": "hero"}
	entry.merge(extra, true)
	return entry

static func _pouch_ids(c: Combatant) -> Array[String]:
	var out: Array[String] = []
	for con: Consumable in c.inventory.battle_pouch:
		out.append(con.id)
	return out

# The first catalog item matching `pred` (by catalog order); "" if none.
static func _first_item(pred: Callable) -> String:
	for e: Dictionary in ContentCatalog.entries("items"):
		if DictUtil.to_bool(pred.call(e)):
			return str(e.get("id", ""))
	return ""

static func _label(kind: String, key: String) -> String:
	for f: Dictionary in ContentSchema.fields(kind):
		if str(f["key"]) == key:
			return str(f["label"])
	return key

static func _test_no_item_slots() -> bool:
	var checks: Dictionary = {}
	var cls: Dictionary = ContentCatalog.get_entry("classes", "priest")
	var class_main: String = str(cls.get("main_hand", ""))
	var class_off: String = str(cls.get("off_hand", ""))
	checks["the priest class has an off-hand item"] = not class_off.is_empty()
	var none_off: Combatant = ContentFactory.unit(_unit_entry("bal2_t_off_none", "priest", {"off_hand": ContentSchema.NO_ITEM}))
	var inherit_off: Combatant = ContentFactory.unit(_unit_entry("bal2_t_off_inherit", "priest", {"off_hand": ""}))
	var main_eq: Equipment = none_off.inventory.get_equipped(Equipment.EquipSlot.MAIN_HAND)
	checks["off_hand 'none': no off-hand item"] = none_off.inventory.get_equipped(Equipment.EquipSlot.OFF_HAND) == null
	checks["off_hand 'none': main hand = the class main_hand"] = (main_eq == null and class_main.is_empty()) or (main_eq != null and main_eq.id == class_main)
	var inherit_eq: Equipment = inherit_off.inventory.get_equipped(Equipment.EquipSlot.OFF_HAND)
	checks["off_hand '': the class off_hand (%s)" % class_off] = inherit_eq != null and inherit_eq.id == class_off
	checks["normalize keeps 'none'"] = str(ContentSchema.normalize("units", _unit_entry("bal2_t_n", "priest", {"off_hand": ContentSchema.NO_ITEM}))["off_hand"]) == ContentSchema.NO_ITEM
	return TestHelpers.report(TAG, "Unit gear slot 'none' leaves the slot empty; '' inherits the class item", checks)

static func _test_no_item_pouch() -> bool:
	var checks: Dictionary = {}
	var class_pouch: Array[String] = ContentFactory._strings(ContentCatalog.get_entry("classes", "priest").get("pouch", []))
	var empty_pouch: Combatant = ContentFactory.unit(_unit_entry("bal2_t_pouch_none", "priest", {"pouch": [ContentSchema.NO_ITEM]}))
	var inherit_pouch: Combatant = ContentFactory.unit(_unit_entry("bal2_t_pouch_inherit", "priest", {"pouch": []}))
	var expected: Array[String] = []
	for id: String in class_pouch:
		if ContentCatalog.has("items", id) and str(ContentCatalog.get_entry("items", id).get("item_type", "")) == "CONSUMABLE":
			expected.append(id)
	checks["pouch ['none']: battle pouch empty"] = empty_pouch.inventory.battle_pouch.is_empty()
	checks["pouch []: the class pouch (%d items)" % expected.size()] = _pouch_ids(inherit_pouch) == expected
	return TestHelpers.report(TAG, "Unit pouch ['none'] is empty; [] inherits the class pouch", checks)

static func _test_no_item_validation() -> bool:
	var checks: Dictionary = {}
	var none: String = ContentSchema.NO_ITEM
	var consumable: String = _first_item(func(e: Dictionary) -> bool: return str(e.get("item_type", "")) == "CONSUMABLE")
	checks["the catalog has a consumable"] = not consumable.is_empty()
	var slot_errors: Array[String] = ContentCatalog.validate_entry("units", _unit_entry("bal2_t_v1", "priest", {"off_hand": none}))
	checks["unit slot 'none' is valid %s" % str(slot_errors)] = slot_errors.is_empty()
	var all_slots: Dictionary = {"pouch": [none]}
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		all_slots[slot] = none
	var all_errors: Array[String] = ContentCatalog.validate_entry("units", _unit_entry("bal2_t_v2", "priest", all_slots))
	checks["unit with every slot 'none' and pouch ['none'] is valid %s" % str(all_errors)] = all_errors.is_empty()
	var mixed: Array[String] = ContentCatalog.validate_entry("units", _unit_entry("bal2_t_v3", "priest", {"pouch": [none, consumable]}))
	checks["pouch 'none' + another entry: exactly the pouch error %s" % str(mixed)] = mixed == ["Battle pouch: 'none' must be the only entry."]
	var cls_slot: Dictionary = ContentCatalog.get_entry("classes", "priest").duplicate(true)
	cls_slot["id"] = "bal2_t_cls_slot"
	cls_slot["name"] = "BAL2 test class"
	var base_errors: Array[String] = ContentCatalog.validate_entry("classes", cls_slot)
	checks["the class copy is valid %s" % str(base_errors)] = base_errors.is_empty()
	cls_slot["off_hand"] = none
	var cls_errors: Array[String] = ContentCatalog.validate_entry("classes", cls_slot)
	checks["class slot 'none': exactly the slot error %s" % str(cls_errors)] = cls_errors == ["%s: 'none' is only for units; leave the class slot empty." % _label("classes", "off_hand")]
	var cls_pouch: Dictionary = cls_slot.duplicate(true)
	cls_pouch["off_hand"] = str(ContentCatalog.get_entry("classes", "priest").get("off_hand", ""))
	cls_pouch["pouch"] = [none]
	var pouch_errors: Array[String] = ContentCatalog.validate_entry("classes", cls_pouch)
	checks["class pouch 'none': exactly the pouch error %s" % str(pouch_errors)] = pouch_errors == ["%s: 'none' is only for units; leave the class slot empty." % _label("classes", "pouch")]
	var item: Dictionary = ContentCatalog.get_entry("items", consumable).duplicate(true)
	item["id"] = none
	var item_errors: Array[String] = ContentCatalog.validate_entry("items", item)
	checks["item id 'none': exactly the reserved error %s" % str(item_errors)] = item_errors == ["The item id 'none' is reserved."]
	# A two-handed main weapon with off_hand 'none' (a class trained in its weapon type, picked by rule).
	var two_handed: String = ""
	var trained_class: String = ""
	for e: Dictionary in ContentCatalog.entries("items"):
		if not DictUtil.to_bool(e.get("two_handed", false)) or str(e.get("slot", "")) != "MAIN_HAND":
			continue
		for c: Dictionary in ContentCatalog.entries("classes"):
			if ContentFactory.class_archetype(c).make_proficiency().can_use(str(e.get("weapon_type", ""))):
				two_handed = str(e["id"])
				trained_class = str(c["id"])
				break
		if not two_handed.is_empty():
			break
	if not two_handed.is_empty():
		var th_errors: Array[String] = ContentCatalog.validate_entry("units", _unit_entry("bal2_t_v4", trained_class, {"main_hand": two_handed, "off_hand": none}))
		checks["2H %s (%s) with off_hand 'none' is valid %s" % [two_handed, trained_class, str(th_errors)]] = th_errors.is_empty()
	checks["validate_all() on shipped data stays {}"] = ContentCatalog.validate_all().is_empty()
	var warned: String = str(ContentCatalog.warnings_all())
	checks["warnings_all() never mentions the sentinel"] = not warned.contains("'none'")
	return TestHelpers.report(TAG, "Sentinel validation: units only, alone in the pouch, reserved item id", checks)

static func _test_all_none_unarmed() -> bool:
	var checks: Dictionary = {}
	var all_slots: Dictionary = {"pouch": [ContentSchema.NO_ITEM]}
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		all_slots[slot] = ContentSchema.NO_ITEM
	var bare: Combatant = ContentFactory.unit(_unit_entry("bal2_t_bare", "warrior", all_slots))
	var equipped: int = 0
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		if bare.inventory.get_equipped(_slot_enum(slot)) != null:
			equipped += 1
	checks["no equipment (%d equipped)" % equipped] = equipped == 0
	checks["empty battle pouch"] = bare.inventory.battle_pouch.is_empty()
	checks["attack power uses the unarmed path"] = str(DamageCalculator.attack_power(bare)["stat_name"]) == "STR+DEX"
	return TestHelpers.report(TAG, "A unit with every slot 'none' builds bare-handed", checks)

static func _slot_enum(slot: String) -> Equipment.EquipSlot:
	return Equipment.EquipSlot.keys().find(slot.to_upper()) as Equipment.EquipSlot

# class_rules / engine_pieces gear checks hold on a sandbox brawler with an empty class main hand.
static func _test_unarmed_brawler_fixture() -> bool:
	var checks: Dictionary = {}
	var forked: bool = TestHelpers.fork(SANDBOX)
	checks["sandbox fork"] = forked
	if forked:
		var bare: Dictionary = ContentCatalog.get_entry("classes", "brawler").duplicate(true)
		bare["main_hand"] = ""
		var errors: Array[String] = ContentCatalog.upsert("classes", bare)
		checks["sandbox brawler with main_hand '' accepted %s" % str(errors)] = errors.is_empty()
		checks["class_rules tiers (null-safe weapon compare)"] = ClassRulesTestSuite._test_variance_and_tiers()
		checks["engine_pieces unarmed (fixture weapon)"] = UnarmedTestSuite._test_unarmed()
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "Gear tests independent of the shipped brawler main hand (sandbox, main_hand '')", checks)

# ------------------------------------------------------------------------------
# Commit 4: elite / boss tier values
# ------------------------------------------------------------------------------

static func _tiered(class_id: String, level: int, tier: String) -> Combatant:
	return ContentFactory.unit({"id": "bal2_t_tier_%s" % tier, "name": tier, "side": "enemy", "class_id": class_id, "level": level, "tier": tier})

static func _test_tier_values() -> bool:
	var checks: Dictionary = {}
	checks["elite hp 1.25 / power 1.1"] = is_equal_approx(StatRules.tier_hp("elite"), 1.25) and is_equal_approx(StatRules.tier_power("elite"), 1.1)
	checks["boss hp 1.5 / power 1.1"] = is_equal_approx(StatRules.tier_hp("boss"), 1.5) and is_equal_approx(StatRules.tier_power("boss"), 1.1)
	checks["hero, minion, standard unchanged"] = is_equal_approx(StatRules.tier_hp("hero"), 1.0) and is_equal_approx(StatRules.tier_power("hero"), 1.0) \
		and is_equal_approx(StatRules.tier_hp("minion"), 0.4) and is_equal_approx(StatRules.tier_power("minion"), 0.6) \
		and is_equal_approx(StatRules.tier_hp("standard"), 0.7) and is_equal_approx(StatRules.tier_power("standard"), 0.8)
	var class_id: String = "warrior"
	var level: int = 5
	var hero: Combatant = _tiered(class_id, level, "hero")
	var elite: Combatant = _tiered(class_id, level, "elite")
	var unscaled_hp: int = hero.stats.get_max_hp(hero.get_weapon_context())
	var parts: Dictionary = DamageCalculator.attack_power(hero)
	var unscaled_power: float = DictUtil.to_float(parts["weapon_power"]) + DictUtil.to_float(parts["unit_power"])
	checks["elite max_hp %d = round(%d x 1.25)" % [elite.max_hp, unscaled_hp]] = elite.max_hp == roundi(float(unscaled_hp) * StatRules.tier_hp("elite"))
	checks["elite power %d = round(%.1f x 1.1)" % [DamageCalculator.power_of(elite), unscaled_power]] = DamageCalculator.power_of(elite) == maxi(1, roundi(unscaled_power * StatRules.tier_power("elite")))
	var hp: Array[int] = []
	var pw: Array[int] = []
	for tier: String in ["minion", "standard", "hero", "elite", "boss"]:
		var c: Combatant = _tiered(class_id, level, tier)
		hp.append(c.max_hp)
		pw.append(DamageCalculator.power_of(c))
	checks["HP minion < standard < hero < elite < boss %s" % str(hp)] = hp[0] < hp[1] and hp[1] < hp[2] and hp[2] < hp[3] and hp[3] < hp[4]
	checks["power minion < standard < hero < elite <= boss %s" % str(pw)] = pw[0] < pw[1] and pw[1] < pw[2] and pw[2] < pw[3] and pw[3] <= pw[4]
	return TestHelpers.report(TAG, "Tier values: elite 1.25 / 1.1, boss 1.5 / 1.1; ordering", checks)

# ------------------------------------------------------------------------------
# Commit 5: trained fists (DamageCalculator.UNARMED_BASE_POWER, fist proficiency >= 2)
# ------------------------------------------------------------------------------

static func _fist_power(c: Combatant) -> int:
	return DamageCalculator.UNARMED_BASE_POWER + c.stats.proficiency.get_weapon_power_bonus("fist")

static func _test_trained_fists() -> bool:
	var checks: Dictionary = {}
	var fist_level: int = ClassArchetype.create("brawler").make_proficiency().level_of("fist")
	checks["the brawler class has fist %d >= %d" % [fist_level, DamageCalculator.TRAINED_FIST_MIN_LEVEL]] = fist_level >= DamageCalculator.TRAINED_FIST_MIN_LEVEL
	for lvl: int in [1, 3, 5, 7]:
		var bare: Combatant = _hero("brawler", lvl, {"main_hand": ContentSchema.NO_ITEM})
		var res: Dictionary = DamageCalculator.attack_power(bare)
		var str_val: int = bare.stats.get_effective_stat("STR", {})
		var dex_val: int = bare.stats.get_effective_stat("DEX", {})
		var expected: int = maxi(1, roundi(float(_fist_power(bare) + str_val + dex_val) * bare.power_scale))
		checks["brawler L%d bare: power %d = fists %d + STR %d + DEX %d" % [lvl, DictUtil.to_int(res["attack_power"]), _fist_power(bare), str_val, dex_val]] = DictUtil.to_bool(res["trained_fists"]) and DictUtil.to_int(res["attack_power"]) == expected
		var armed: Combatant = _hero("brawler", lvl, {"main_hand": "wpn_flanged_mace"})
		var w_res: Dictionary = DamageCalculator.attack_power(armed)
		var weapon_path: int = maxi(1, roundi((DictUtil.to_float(w_res["weapon_power"]) + DictUtil.to_float(w_res["unit_power"])) * armed.power_scale))
		checks["brawler L%d with a mace: weapon path, no trained fists" % lvl] = armed.inventory.get_main_weapon() != null and not DictUtil.to_bool(w_res["trained_fists"]) \
			and str(w_res["stat_name"]) != "STR+DEX" and DictUtil.to_int(w_res["attack_power"]) == weapon_path
	var rogue: Combatant = _hero("rogue", 5, {"main_hand": ContentSchema.NO_ITEM})
	var r_res: Dictionary = DamageCalculator.attack_power(rogue)
	var r_sum: int = rogue.stats.get_effective_stat("STR", {}) + rogue.stats.get_effective_stat("DEX", {})
	checks["rogue fist %d < %d" % [rogue.stats.proficiency.level_of("fist"), DamageCalculator.TRAINED_FIST_MIN_LEVEL]] = rogue.stats.proficiency.level_of("fist") < DamageCalculator.TRAINED_FIST_MIN_LEVEL
	checks["rogue bare: no trained fists, power STR + DEX"] = not DictUtil.to_bool(r_res["trained_fists"]) and DictUtil.to_int(r_res["attack_power"]) == maxi(1, roundi(float(r_sum) * rogue.power_scale))
	return TestHelpers.report(TAG, "Trained fists: fist >= 2 bare hands = base + training + STR + DEX; armed and untrained unchanged", checks)

static func _test_fist_edges() -> bool:
	var checks: Dictionary = {}
	var bare: Combatant = _hero("brawler", 5, {"main_hand": ContentSchema.NO_ITEM, "off_hand": ContentSchema.NO_ITEM})
	checks["off-hand bare-hand attack: no trained fists"] = not DictUtil.to_bool(DamageCalculator.attack_power(bare, false, true)["trained_fists"])
	checks["authored power: no trained fists"] = _authored_bare()
	var bare_crit: float = DamageCalculator.crit_chance(bare)
	var dex_crit: float = bare.stats.get_crit_rate({})
	checks["bare crit %.1f = DEX crit %.1f + fist bonus %.1f" % [bare_crit, dex_crit, bare.stats.proficiency.crit_bonus("fist")]] = is_equal_approx(bare_crit, dex_crit + bare.stats.proficiency.crit_bonus("fist"))
	var armed: Combatant = _hero("brawler", 5, {"main_hand": "wpn_flanged_mace"})
	var ctx: Dictionary = armed.get_weapon_context()
	checks["armed crit = DEX crit + weapon training bonus"] = is_equal_approx(DamageCalculator.crit_chance(armed), armed.stats.get_crit_rate(ctx) + armed.stats.proficiency.crit_bonus(str(ctx.get("weapon", ""))))
	checks["UNARMED_VALUE = UNARMED_BASE_POWER = 15"] = is_equal_approx(GearBudget.UNARMED_VALUE, float(DamageCalculator.UNARMED_BASE_POWER)) and is_equal_approx(GearBudget.UNARMED_VALUE, 15.0)
	var leather: Dictionary = ContentCatalog.get_entry("items", "arm_leather")
	if not leather.is_empty():
		var loadout: Dictionary = {"id": "t_bal2_bare", "main_hand": "", "body": "arm_leather"}
		checks["empty main hand + leather = UNARMED_VALUE + leather value"] = is_equal_approx(GearBudget.loadout_value(loadout), GearBudget.UNARMED_VALUE + GearBudget.item_value(leather))
	return TestHelpers.report(TAG, "Trained fists edges: off hand, authored power, crit, gear budget value", checks)

static func _authored_bare() -> bool:
	var c: Combatant = _hero("brawler", 5, {"main_hand": ContentSchema.NO_ITEM})
	c.power_override = 30
	var res: Dictionary = DamageCalculator.attack_power(c)
	return not DictUtil.to_bool(res["trained_fists"]) and DictUtil.to_int(res["attack_power"]) == 30 and not DamageCalculator.trained_fists(c)
