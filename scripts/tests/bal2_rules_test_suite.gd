class_name Bal2RulesTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - BAL2 Rules Test Suite (task BAL2-L)
# Commit 2: the PERC range bonus (+1 at RANGE_PERC_THRESHOLD) only for ranged weapons (base
# range > 1). Commit 3: the unit gear sentinel ContentSchema.NO_ITEM ("none") per slot and pouch.
# Commit 4: the elite / boss tier values. Units are built from the class rules (hero, default spend,
# class gear) or in memory; expectations derive from CharacterStats / DamageCalculator / StatRules.
# BAL3-S (docs/specs/BAL3.md 3.1-3.3): the sim's seeded start jitter, the clumped enemy block, the
# role scenario scale and the encounter archetypes; threat totals derive from the StatRules tiers.
# BAL5-SIM / BAL5-PH (docs/specs/BAL5_LEVERS.md section 8): K2 trio sweep and party seeds, K3 won-fight
# split, heal casts and down round, K4 tier_foe, K5 scenario edits (A1, S2) and the generated boss
# phase (B2), K6 jitter reach refusals as skipped fights. K1 (world=) lives in bal4_campaign_test_suite.
# ==============================================================================

const SimArgs = preload("res://tools/sim/sim_args.gd")
const SimRunner = preload("res://tools/sim/sim_runner.gd")
const SimVariants = preload("res://tools/sim/sim_variants.gd")
const SimScenarios = preload("res://tools/sim/sim_scenarios.gd")
const SimParty = preload("res://tools/sim/sim_party.gd")
const SimMetrics = preload("res://tools/sim/sim_metrics.gd")
const SimRoles = preload("res://tools/sim/sim_roles.gd")
const SimTargets = preload("res://tools/sim/sim_targets.gd")

# Records every unit's tile (by name) when a sim fight starts (after the runner's placement).
class StartObserver extends "res://tools/sim/sim_observer.gd":
	var tiles: Dictionary = {}
	func on_start(session: CombatSession) -> void:
		tiles = Bal2RulesTestSuite._tiles_of(session.grid, session.allies + session.enemies)

# The metrics collector plus an independent count of heal entries per actor, the first round each unit
# is at 0 HP, and whether the generated boss phase fired.
class CountingMetrics extends "res://tools/sim/sim_metrics.gd":
	var heal_entries: Dictionary = {}   # actor name -> heal entries this fight
	var downs: Dictionary = {}          # unit name -> round first seen at 0 HP
	var phase_fired: bool = false
	var _watched: CombatSession = null
	func on_start(session: CombatSession) -> void:
		super(session)
		heal_entries = {}
		downs = {}
		phase_fired = false
		_watched = session
		session.log_appended.connect(_count)
	func _count(e: ActionLogEntry) -> void:
		if e.category == "BUFF" and e.skill_subtype == "Support":
			heal_entries[e.actor_name] = DictUtil.int_of(heal_entries, e.actor_name) + 1
	func on_step(session: CombatSession) -> void:
		super(session)
		for c: Combatant in session.allies + session.enemies:
			if c.hp <= 0 and not downs.has(c.name):
				downs[c.name] = session.current_round
		var enc: CatalogEncounter = session.encounter as CatalogEncounter
		if enc != null and enc.phase_runtime.is_fired(str(SimScenarios.BOSS_PHASE["id"])):
			phase_fired = true
	func on_end(session: CombatSession, result: Dictionary) -> void:
		for c: Combatant in session.allies + session.enemies:
			if c.hp <= 0 and not downs.has(c.name):
				downs[c.name] = session.current_round
		if _watched != null and _watched.log_appended.is_connected(_count):
			_watched.log_appended.disconnect(_count)
		super(session, result)

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
	ok = _test_jitter_bounds() and ok
	ok = _test_jitter_order() and ok
	ok = _test_jitter_runner() and ok
	ok = _test_enemy_block() and ok
	ok = _test_role_scale() and ok
	ok = _test_archetypes() and ok
	ok = _test_bal5_args() and ok
	ok = _test_trio_sweep() and ok
	ok = _test_scenario_edits() and ok
	ok = _test_boss_phase() and ok
	ok = _test_won_split() and ok
	ok = _test_heal_down_metrics() and ok
	ok = _test_reach_skip() and ok
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
	# Shipped classes carry no pouch, so a sandbox priest gets one consumable (picked by rule) as the fixture.
	var forked: bool = TestHelpers.fork(SANDBOX)
	checks["sandbox fork"] = forked
	if forked:
		var consumable: String = _first_item(func(e: Dictionary) -> bool: return str(e.get("item_type", "")) == "CONSUMABLE")
		var cls: Dictionary = ContentCatalog.get_entry("classes", "priest").duplicate(true)
		cls["pouch"] = [consumable]
		var errors: Array[String] = ContentCatalog.upsert("classes", cls)
		checks["fixture class pouch accepted %s" % str(errors)] = consumable != "" and errors.is_empty()
		var empty_pouch: Combatant = ContentFactory.unit(_unit_entry("bal2_t_pouch_none", "priest", {"pouch": [ContentSchema.NO_ITEM]}))
		var inherit_pouch: Combatant = ContentFactory.unit(_unit_entry("bal2_t_pouch_inherit", "priest", {"pouch": []}))
		checks["pouch ['none']: battle pouch empty"] = empty_pouch.inventory.battle_pouch.is_empty()
		checks["pouch []: the class pouch [%s]" % consumable] = _pouch_ids(inherit_pouch) == [consumable]
	TestHelpers.cleanup(SANDBOX)
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
	checks["elite hp 1.25 / power 1.0 (BAL3 Q3)"] = is_equal_approx(StatRules.tier_hp("elite"), 1.25) and is_equal_approx(StatRules.tier_power("elite"), 1.0)
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
	checks["elite power %d = round(%.1f x tier power)" % [DamageCalculator.power_of(elite), unscaled_power]] = DamageCalculator.power_of(elite) == maxi(1, roundi(unscaled_power * StatRules.tier_power("elite")))
	var hp: Array[int] = []
	var pw: Array[int] = []
	for tier: String in ["minion", "standard", "hero", "elite", "boss"]:
		var c: Combatant = _tiered(class_id, level, tier)
		hp.append(c.max_hp)
		pw.append(DamageCalculator.power_of(c))
	checks["HP minion < standard < hero < elite < boss %s" % str(hp)] = hp[0] < hp[1] and hp[1] < hp[2] and hp[2] < hp[3] and hp[3] < hp[4]
	checks["power minion < standard < hero <= elite <= boss %s" % str(pw)] = pw[0] < pw[1] and pw[1] < pw[2] and pw[2] <= pw[3] and pw[3] <= pw[4]
	return TestHelpers.report(TAG, "Tier values: elite 1.25 / 1.0, boss 1.5 / 1.1; ordering", checks)

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

# ------------------------------------------------------------------------------
# BAL3-S: sim start jitter, enemy block, scenario scale, encounter archetypes
# ------------------------------------------------------------------------------

const JITTER_PARTY: int = 4
const JITTER_FOES: int = 6

static func _tiles_of(grid: CombatGrid, units: Array[Combatant]) -> Dictionary:
	var out: Dictionary = {}
	for c: Combatant in units:
		if grid.has_combatant(c):
			out[c.name] = grid.get_position(c)
	return out

# A standard deployment of JITTER_PARTY allies and JITTER_FOES enemies (every second unit ranged)
# on a closed arena.
static func _deployed() -> Dictionary:
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
	var allies: Array[Combatant] = []
	var foes: Array[Combatant] = []
	for i: int in range(JITTER_PARTY):
		allies.append(Combatant.new("A%d" % i, 100, 10, 1 + (i % 2) * 3))
	for i: int in range(JITTER_FOES):
		foes.append(Combatant.new("E%d" % i, 100, 10, 1 + (i % 2) * 3))
	grid.deploy_combatants(allies, foes)
	return {"grid": grid, "allies": allies, "foes": foes}

static func _units(d: Dictionary, key: String) -> Array[Combatant]:
	var out: Array[Combatant] = []
	out.assign(DictUtil.array_of(d[key]))
	return out

static func _grid(d: Dictionary) -> CombatGrid:
	var g: CombatGrid = d["grid"]
	return g

# Every unit within 1 tile of its standard tile, on the same half, on a walkable non-exit tile, alone.
static func _test_jitter_bounds() -> bool:
	var checks: Dictionary = {}
	var seeds: Array[int] = SimArgs.new().seeds()
	var moved_total: int = 0
	for s: int in seeds + [_extra_seed()]:
		var d: Dictionary = _deployed()
		var grid: CombatGrid = _grid(d)
		var units: Array[Combatant] = _units(d, "allies") + _units(d, "foes")
		var before: Dictionary = _tiles_of(grid, units)
		moved_total += SimVariants.jitter_start(grid, _units(d, "allies"), _units(d, "foes"), SimVariants.jitter_rng(s))
		var after: Dictionary = _tiles_of(grid, units)
		var seen: Dictionary = {}
		for c: Combatant in units:
			var a: Vector2i = after[c.name]
			var b: Vector2i = before[c.name]
			if grid.get_distance(a, b) > 1 or CombatGrid.is_party_half(a) != CombatGrid.is_party_half(b) or not grid.is_walkable(a) or grid.is_exit_tile(a) or seen.has(a):
				checks["seed %d: %s %s -> %s inside its half, 1 tile, alone" % [s, c.name, str(b), str(a)]] = false
			seen[a] = true
		checks["seed %d: every unit still deployed" % s] = after.size() == units.size()
		var again: Dictionary = _deployed()
		SimVariants.jitter_start(_grid(again), _units(again, "allies"), _units(again, "foes"), SimVariants.jitter_rng(s))
		checks["seed %d: same seed, same layout" % s] = _tiles_of(_grid(again), _units(again, "allies") + _units(again, "foes")) == after
	checks["some unit moves over the seeds (%d moves)" % moved_total] = moved_total > 0
	# A blocked neighbour and the far half are never offered.
	var walled: Dictionary = _deployed()
	var wg: CombatGrid = _grid(walled)
	var w_front: Combatant = _units(walled, "allies")[0]
	var w_here: Vector2i = wg.get_position(w_front)
	var open_before: Array[Vector2i] = SimVariants.jitter_options(wg, w_front)
	var wall_tile: Vector2i = Vector2i(-1, -1)
	for t: Vector2i in open_before:
		if t != w_here:
			wall_tile = t
			break
	checks["a free neighbour exists to wall off"] = wg.is_valid_coord(wall_tile)
	if wg.is_valid_coord(wall_tile):
		wg.set_terrain(wall_tile, Terrain.Type.WALL)   # after deploy_combatants: really exercised
	var options: Array[Vector2i] = SimVariants.jitter_options(wg, w_front)
	checks["options start with the own tile"] = not options.is_empty() and options[0] == w_here
	var bad: bool = false
	for t: Vector2i in options:
		if t != w_here and (not wg.is_walkable(t) or wg.is_occupied(t) or not CombatGrid.is_party_half(t)):
			bad = true
	checks["options: walkable, free, own half (wall at %s refused)" % str(wall_tile)] = not bad and not options.has(wall_tile)
	return TestHelpers.report(TAG, "BAL3-S jitter: up to 1 tile inside the own half, never onto a unit or blocked tile, seeded", checks)

static func _extra_seed() -> int:
	return SimVariants.JITTER_SALT   # one more seed outside the sim's default list

# The seeded party order keeps the members and is reproducible per seed.
static func _test_jitter_order() -> bool:
	var checks: Dictionary = {}
	var ids: Array[String] = []
	for c: String in SimArgs.DEFAULT_COMP:
		ids.append("p_" + c)
	var orders: Dictionary = {}
	for s: int in SimArgs.new().seeds() + [_extra_seed()]:
		var a: Array[String] = SimVariants.shuffled(ids, SimVariants.jitter_rng(s))
		var b: Array[String] = SimVariants.shuffled(ids, SimVariants.jitter_rng(s))
		var sa: Array[String] = a.duplicate()
		sa.sort()
		var si: Array[String] = ids.duplicate()
		si.sort()
		checks["seed %d: same members %s" % [s, str(a)]] = sa == si
		checks["seed %d: same order twice" % s] = a == b
		orders[",".join(a)] = true
	checks["the input list is not changed"] = ids.size() == SimArgs.DEFAULT_COMP.size() and ids[0] == "p_" + SimArgs.DEFAULT_COMP[0]
	checks["the order varies over the seeds (%d orders)" % orders.size()] = orders.size() > 1
	return TestHelpers.report(TAG, "BAL3-S jitter: seeded party order", checks)

# Runner level (sandbox): jitter on by default, jitter=off gives the standard deployment exactly;
# on keeps both sides inside their halves without overlap.
static func _test_jitter_runner() -> bool:
	var checks: Dictionary = {}
	checks["jitter default on"] = _sim_args(["party"]).jitter
	checks["jitter=off parses"] = not _sim_args(["party", "jitter=off"]).jitter
	checks["jitter=maybe refused"] = not SimArgs.new().parse(PackedStringArray(["party", "jitter=maybe"]))
	var forked: bool = TestHelpers.fork(SANDBOX)
	checks["sandbox fork"] = forked
	if forked:
		for on: bool in [false, true]:
			var args: SimArgs = _sim_args(["party", "5", "cap=1", "jitter=%s" % ("on" if on else "off")])
			var runner := SimRunner.new(args)
			var obs := StartObserver.new()
			runner.observer = obs
			var party: Array[String] = SimScenarios.define_party(runner, SimArgs.DEFAULT_COMP, args.level)
			var enc: Dictionary = SimScenarios.encounter(runner, "line", args.level, 0, false, args.elite_class, args.boss_class)
			var foes: Array[String] = []
			foes.assign(DictUtil.array_of(enc["enemies"]))
			var seed_value: int = args.seeds()[0]
			var res: Dictionary = runner.fight(party, foes, seed_value)
			checks["jitter %s: fight ran" % str(on)] = DictUtil.to_bool(res.get("ok", false))
			var plain := CatalogEncounter.from_entry({"id": "bal3_t_plain", "name": "T", "zone": "stone_arena", "arena": "CLOSED", "party": party, "enemies": foes, "max_rounds": 1, "rng_seed": seed_value})
			plain.build()
			var standard: Dictionary = _tiles_of(plain.grid, plain.allies + plain.enemies)
			if not on:
				checks["jitter off: start tiles = standard deployment %s" % str(obs.tiles)] = obs.tiles == standard and not standard.is_empty()
				continue
			var seen: Dictionary = {}
			var bad: Array[String] = []
			for n: Variant in obs.tiles:
				var t: Vector2i = obs.tiles[n]
				if CombatGrid.is_party_half(t) != str(n).begins_with("simp_") or seen.has(t):
					bad.append("%s %s" % [str(n), str(t)])
				seen[t] = true
			checks["jitter on: every unit on its own half, no overlap %s" % str(bad)] = bad.is_empty() and obs.tiles.size() == standard.size()
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "BAL3-S jitter in the runner: default on, off = the standard deployment", checks)

static func _sim_args(tokens: Array[String]) -> SimArgs:
	var a := SimArgs.new()
	a.parse(PackedStringArray(tokens))
	return a

# The aoe_swarm block: every block unit on one rectangle of adjacent tiles on the enemy half, the
# nearest column that keeps the start distance; the row shift moves it.
static func _test_enemy_block() -> bool:
	var checks: Dictionary = {}
	var size: Vector2i = SimScenarios.scenario_block("aoe_swarm")
	var swarm: int = DictUtil.array_of(DictUtil.dict_of(SimScenarios.SCENARIOS["aoe_swarm"])["enemies"]).size()
	checks["aoe_swarm has a block that holds every enemy (%s, %d enemies)" % [str(size), swarm]] = size.x > 0 and size.y > 0 and size.x * size.y == swarm
	checks["other scenarios keep the standard deployment"] = SimScenarios.scenario_block("tank_hold") == Vector2i.ZERO
	var dist: int = SimArgs.ROLE_DISTANCE
	for shift: int in [-1, 0, 1]:
		var d: Dictionary = _deployed()
		var grid: CombatGrid = _grid(d)
		var allies: Array[Combatant] = _units(d, "allies")
		var foes: Array[Combatant] = _units(d, "foes")
		checks["shift %d: block placed" % shift] = SimVariants.apply_block(grid, allies, foes, size, dist, shift)
		var tiles: Array[Vector2i] = []
		for c: Combatant in foes:
			tiles.append(grid.get_position(c))
		var front: int = CombatGrid.GRID_W
		var near: int = 1 << 30
		for t: Vector2i in tiles:
			front = mini(front, t.x)
			for a: Combatant in allies:
				near = mini(near, grid.get_distance(grid.get_position(a), t))
		var mid: int = floori(float(CombatGrid.GRID_H) / 2.0) + shift
		var expected: Array[Vector2i] = SimVariants.block_tiles(front, mid, size)
		var sorted_t: Array[Vector2i] = tiles.duplicate()
		sorted_t.sort()
		expected.sort()
		checks["shift %d: units fill the %s block at column %d, middle row %d" % [shift, str(size), front, mid]] = sorted_t == expected
		checks["shift %d: block on the enemy half" % shift] = not CombatGrid.is_party_half(Vector2i(front, 0))
		checks["shift %d: nearest ally distance %d = start distance %d (nearest fitting column)" % [shift, near, dist]] = near == dist
	return TestHelpers.report(TAG, "BAL3-S aoe_swarm: clumped enemy block at the start distance", checks)

static func _classes(list: Array) -> Array[String]:
	var out: Array[String] = []
	for raw: Variant in list:
		out.append(str(DictUtil.array_of(raw)[0]))
	return out

static func _tiers(list: Array) -> Array[String]:
	var out: Array[String] = []
	for raw: Variant in list:
		out.append(str(DictUtil.array_of(raw)[1]))
	return out

static func _threat(list: Array) -> float:
	var t: float = 0.0
	for tier: String in _tiers(list):
		t += StatRules.tier_hp(tier) * StatRules.tier_power(tier)
	return t

# BAL3 3.2 role scale: aoe_swarm all melee minions with a third brawler, striker_priority's target
# standard only up to L3; ranged_dive stays standard melee divers (its third rogue was dropped: with
# jitter the ranger is already inside the band).
static func _test_role_scale() -> bool:
	var checks: Dictionary = {}
	var swarm: Array = SimScenarios.scenario_enemies("aoe_swarm", 5)
	var swarm_classes: Array[String] = _classes(swarm)
	checks["aoe_swarm: no ranger, three brawlers %s" % str(swarm_classes)] = not swarm_classes.has("ranger") and swarm_classes.count("brawler") == 3
	checks["aoe_swarm: every enemy a minion"] = _tiers(swarm).count("minion") == swarm.size()
	for c: String in swarm_classes:
		checks["aoe_swarm %s is melee at L5" % c] = _hero(c, 5).attack_range == 1
	var rule: Dictionary = DictUtil.dict_of(DictUtil.dict_of(SimScenarios.SCENARIOS["striker_priority"]).get("tier_until", {}))
	var last: int = DictUtil.int_of(rule, "level")
	var authored: Array = DictUtil.array_of(DictUtil.dict_of(SimScenarios.SCENARIOS["striker_priority"])["enemies"])
	checks["striker_priority: the rule ends at L3 (%d)" % last] = last == 3
	checks["striker_priority L%d: priority target standard" % last] = _tiers(SimScenarios.scenario_enemies("striker_priority", last))[0] == "standard"
	checks["striker_priority L%d: priority target keeps its authored tier %s" % [last + 2, _tiers(authored)[0]]] = _tiers(SimScenarios.scenario_enemies("striker_priority", last + 2))[0] == _tiers(authored)[0] and _tiers(authored)[0] == "elite"
	checks["striker_priority: only the priority slot changes"] = _tiers(SimScenarios.scenario_enemies("striker_priority", last)).slice(1) == _tiers(authored).slice(1) \
		and _classes(SimScenarios.scenario_enemies("striker_priority", last)) == _classes(authored)
	var dive: Array = SimScenarios.scenario_enemies("ranged_dive", 5)
	checks["ranged_dive: standard divers only %s" % str(_classes(dive))] = _tiers(dive).count("standard") == dive.size()
	var melee: int = 0
	for c: String in _classes(dive):
		melee += 1 if _hero(c, 5).attack_range == 1 else 0
	checks["ranged_dive: every one of the %d enemies is a melee diver (%d)" % [dive.size(), melee]] = melee == dive.size()
	return TestHelpers.report(TAG, "BAL3-S role scenario scale (aoe_swarm, striker_priority L3; ranged_dive divers)", checks)

# BAL3 3.3 archetypes: threat (tier HP x tier power, waves included) rises with the target band
# order mob < line < wave < elite < boss and stays under a full party; compositions per the spec.
static func _test_archetypes() -> bool:
	var checks: Dictionary = {}
	var std: float = StatRules.tier_hp("standard") * StatRules.tier_power("standard")
	var minion: float = StatRules.tier_hp("minion") * StatRules.tier_power("minion")
	var party: float = float(PartyState.PARTY_MAX) * StatRules.tier_hp("hero") * StatRules.tier_power("hero")
	var threat: Dictionary = {}
	var bodies: Dictionary = {}
	for enc: String in SimArgs.GENERATED:
		var g: Dictionary = SimScenarios.GENERATED[enc]
		var all: Array = DictUtil.array_of(g["enemies"]) + DictUtil.array_of(g.get("wave", []))
		threat[enc] = _threat(all)
		bodies[enc] = all.size()
	var order: Array[String] = ["mob", "line", "wave", "elite", "boss"]
	for i: int in range(order.size() - 1):
		checks["threat %s %.2f < %s %.2f" % [order[i], DictUtil.to_float(threat[order[i]]), order[i + 1], DictUtil.to_float(threat[order[i + 1]])]] = DictUtil.to_float(threat[order[i]]) < DictUtil.to_float(threat[order[i + 1]])
	checks["boss threat %.2f < party of %d %.2f" % [DictUtil.to_float(threat["boss"]), PartyState.PARTY_MAX, party]] = DictUtil.to_float(threat["boss"]) < party
	checks["line = %d standard" % PartyState.PARTY_MAX] = is_equal_approx(DictUtil.to_float(threat["line"]), float(PartyState.PARTY_MAX) * std) and DictUtil.to_int(bodies["line"]) == PartyState.PARTY_MAX
	var elite_t: float = StatRules.tier_hp("elite") * StatRules.tier_power("elite")
	checks["elite = elite + %d standard" % (PartyState.PARTY_MAX - 1)] = is_equal_approx(DictUtil.to_float(threat["elite"]), elite_t + float(PartyState.PARTY_MAX - 1) * std) and DictUtil.to_int(bodies["elite"]) == PartyState.PARTY_MAX
	var boss_g: Dictionary = SimScenarios.GENERATED["boss"]
	var boss_t: float = StatRules.tier_hp("boss") * StatRules.tier_power("boss")
	checks["boss = boss + 2 standard + 2 minion wave"] = is_equal_approx(DictUtil.to_float(threat["boss"]), boss_t + 2.0 * std + 2.0 * minion) \
		and _tiers(DictUtil.array_of(boss_g["wave"])) == ["minion", "minion"] and _tiers(DictUtil.array_of(boss_g["enemies"])).count("standard") == 2
	checks["mob and wave unchanged (5 minions; 3 standard + 3 minion wave)"] = is_equal_approx(DictUtil.to_float(threat["mob"]), 5.0 * minion) and is_equal_approx(DictUtil.to_float(threat["wave"]), 3.0 * std + 3.0 * minion)
	# The boss wave reaches the fight as wave units at the sim's wave round.
	var runner := SimRunner.new(_sim_args(["party"]))
	var enc: Dictionary = SimScenarios.encounter(runner, "boss", 5, 0, false, "warrior", "brawler")
	var base: Dictionary = DictUtil.dict_of(enc["base"])
	checks["boss encounter: %d wave units at round %d" % [DictUtil.array_of(base.get("wave_units", [])).size(), DictUtil.int_of(base, "wave_round")]] = DictUtil.array_of(base.get("wave_units", [])).size() == DictUtil.array_of(boss_g["wave"]).size() \
		and DictUtil.int_of(base, "wave_round") == SimScenarios.WAVE_ROUND
	return TestHelpers.report(TAG, "BAL3-S encounter archetypes: threat order, compositions from the tier rules", checks)

# ------------------------------------------------------------------------------
# BAL5-SIM / BAL5-PH (docs/specs/BAL5_LEVERS.md section 8)
# ------------------------------------------------------------------------------

# K2 seeds, K4 tier_foe, K5 phase / s2 keys: defaults, values, refusals.
static func _test_bal5_args() -> bool:
	var checks: Dictionary = {}
	checks["party cells default to PARTY_SEEDS (%d)" % SimArgs.PARTY_SEEDS] = _sim_args(["party"]).seed_count == SimArgs.PARTY_SEEDS
	checks["sweep cells default to PARTY_SEEDS"] = _sim_args(["sweep"]).seed_count == SimArgs.PARTY_SEEDS
	checks["duel keeps the general default (%d)" % SimArgs.new().seed_count] = _sim_args(["duel"]).seed_count == SimArgs.new().seed_count
	checks["role keeps the general default"] = _sim_args(["role"]).seed_count == SimArgs.new().seed_count
	checks["positional seeds win in party (5)"] = _sim_args(["party", "5", "5"]).seed_count == 5
	checks["quick wins in party (1 seed)"] = _sim_args(["party", "quick"]).seed_count == 1
	var sw: SimArgs = _sim_args(["sweep", "size=3", "lead=warrior"])
	checks["size=3 lead=warrior parse"] = sw.error.is_empty() and sw.comp_size == 3 and sw.lead == "warrior"
	checks["default size is SimParty.COMP_SIZE, no lead"] = SimArgs.new().comp_size == SimParty.COMP_SIZE and SimArgs.new().lead.is_empty()
	checks["size=0 refused"] = not SimArgs.new().parse(PackedStringArray(["sweep", "size=0"]))
	checks["size above PARTY_MAX refused"] = not SimArgs.new().parse(PackedStringArray(["sweep", "size=%d" % (PartyState.PARTY_MAX + 1)]))
	checks["lead=<unknown class> refused"] = not SimArgs.new().parse(PackedStringArray(["sweep", "lead=dragon"]))
	var mirror: SimArgs = _sim_args(["tier"])
	checks["tier_foe default mirror: each hero fights its own class"] = mirror.tier_foe == SimArgs.MIRROR and mirror.tier_foe_for("mage") == "mage"
	var fixed: SimArgs = _sim_args(["tier", "tier_foe=brawler"])
	var all_brawler: bool = true
	for c: String in SimArgs.CLASSES:
		all_brawler = all_brawler and fixed.tier_foe_for(c) == "brawler"
	checks["tier_foe=brawler: every hero fights a brawler"] = fixed.error.is_empty() and all_brawler
	checks["tier_foe header"] = fixed.header().contains("tier_foe brawler")
	checks["tier_foe=<unknown> refused"] = not SimArgs.new().parse(PackedStringArray(["tier", "tier_foe=dragon"]))
	checks["phase default on"] = _sim_args(["party"]).boss_phase
	checks["phase=off parses"] = not _sim_args(["party", "phase=off"]).boss_phase
	checks["phase=maybe refused"] = not SimArgs.new().parse(PackedStringArray(["party", "phase=maybe"]))
	checks["s2 default off, s2=on parses"] = not _sim_args(["role"]).s2 and _sim_args(["role", "s2=on"]).s2
	return TestHelpers.report(TAG, "BAL5-SIM keys: party seeds, size / lead, tier_foe, phase, s2", checks)

# K2: size=3 lead=warrior gives the warrior plus every pair of the other classes (C(n-1, 2)); the default
# sweep stays the C(n, COMP_SIZE) distinct compositions.
static func _test_trio_sweep() -> bool:
	var checks: Dictionary = {}
	var others: int = SimArgs.CLASSES.size() - 1
	var trios: Array = SimParty.new(_sim_args(["sweep", "size=3", "lead=warrior"]), null, null).compositions()
	checks["%d trios = C(%d, 2)" % [trios.size(), others]] = trios.size() == others * (others - 1) / 2
	var keys: Dictionary = {}
	var shape: bool = true
	for raw: Variant in trios:
		var t: Array = DictUtil.array_of(raw)
		shape = shape and t.size() == 3 and str(t[0]) == "warrior" and not t.slice(1).has("warrior") and str(t[1]) != str(t[2])
		var sorted_t: Array = t.duplicate()
		sorted_t.sort()
		keys[",".join(PackedStringArray(sorted_t))] = true
	checks["each trio: the warrior first plus two different other classes"] = shape
	checks["all trios distinct"] = keys.size() == trios.size()
	var n: int = SimArgs.CLASSES.size()
	var k: int = SimParty.COMP_SIZE
	var expected: int = 1
	for i: int in range(k):
		expected = expected * (n - i) / (i + 1)
	checks["default sweep: C(%d, %d) = %d compositions" % [n, k, expected]] = SimParty.new(_sim_args(["sweep"]), null, null).compositions().size() == expected
	var pairs: Array = SimParty.new(_sim_args(["sweep", "size=2", "lead=priest"]), null, null).compositions()
	checks["size=2 lead=priest: priest + each other class (%d)" % others] = pairs.size() == others
	return TestHelpers.report(TAG, "BAL5-SIM K2 trio sweep: size=3 lead=warrior", checks)

# K5: A1 makes aoe_swarm a trio (warrior + priest refs); S2 (trigger fired in BAL5-R1b) is now the authored
# support_attrition: one ranger of the BAL2 list (ranger, ranger, mage, rogue standard) turned into a brawler; s2=on adds
# nothing and touches no other scenario.
static func _test_scenario_edits() -> bool:
	var checks: Dictionary = {}
	var swarm_allies: Array = DictUtil.array_of(DictUtil.dict_of(SimScenarios.SCENARIOS["aoe_swarm"])["allies"])
	checks["A1: aoe_swarm refs warrior + priest (a trio with the test unit) %s" % str(swarm_allies)] = swarm_allies.size() == 2 and swarm_allies.has("warrior") and swarm_allies.has("priest")
	var bal2: Array = [["ranger", "standard"], ["ranger", "standard"], ["mage", "standard"], ["rogue", "standard"]]
	var plain: Array = SimScenarios.scenario_enemies("support_attrition", 5)
	var s2: Array = SimScenarios.scenario_enemies("support_attrition", 5, true)
	checks["S2 authored: the scenario's enemies are S2_ENEMIES"] = plain == SimScenarios.S2_ENEMIES
	checks["S2 authored: same size and tiers as BAL2"] = plain.size() == bal2.size() and _tiers(plain) == _tiers(bal2)
	checks["S2 authored: one ranger fewer, one brawler more than BAL2"] = _classes(plain).count("ranger") == _classes(bal2).count("ranger") - 1 and _classes(plain).count("brawler") == _classes(bal2).count("brawler") + 1
	checks["S2 authored: mage and rogue kept"] = _classes(plain).count("mage") == _classes(bal2).count("mage") and _classes(plain).count("rogue") == _classes(bal2).count("rogue")
	checks["s2=on: no change"] = s2 == plain
	checks["S2 leaves other scenarios alone"] = SimScenarios.scenario_enemies("tank_hold", 5, true) == SimScenarios.scenario_enemies("tank_hold", 5)
	var forked: bool = TestHelpers.fork(SANDBOX)
	checks["sandbox fork"] = forked
	if forked:
		var runner := SimRunner.new(_sim_args(["role"]))
		var sc: Dictionary = SimScenarios.scenario(runner, "aoe_swarm", "mage", 5)
		checks["A1: aoe_swarm party of 3 (%d)" % DictUtil.array_of(sc["party"]).size()] = DictUtil.array_of(sc["party"]).size() == 3
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "BAL5 K5 scenario edits: A1 trio swarm, S2 authored (s2=on no change)", checks)

# B2 / BAL5-PH: the generated boss carries one valid phase on the boss unit (hp_below), phase=off and the
# other archetypes carry none; in fights the phase fires with phase on and never with phase off.
static func _test_boss_phase() -> bool:
	var checks: Dictionary = {}
	var runner := SimRunner.new(_sim_args(["party"]))
	var on: Dictionary = SimScenarios.encounter(runner, "boss", 5, 0, false, "warrior", "brawler", true)
	var off: Dictionary = SimScenarios.encounter(runner, "boss", 5, 0, false, "warrior", "brawler", false)
	var foes: Array = DictUtil.array_of(on["enemies"])
	var phases: Array = DictUtil.array_of(DictUtil.dict_of(on["base"]).get("phases", []))
	checks["phase on: one phase"] = phases.size() == 1
	checks["phase off: no phase"] = DictUtil.array_of(DictUtil.dict_of(off["base"]).get("phases", [])).is_empty()
	for enc_id: String in SimArgs.GENERATED:
		if enc_id != "boss":
			var other: Dictionary = SimScenarios.encounter(runner, enc_id, 5, 0, false, "warrior", "brawler", true)
			checks["%s has no phase" % enc_id] = DictUtil.array_of(DictUtil.dict_of(other["base"]).get("phases", [])).is_empty()
	if phases.size() == 1:
		var p: Dictionary = DictUtil.dict_of(phases[0])
		var boss_id: String = str(foes[0])
		var trig: Dictionary = DictUtil.dict_of(p["trigger"])
		checks["trigger hp_below on the boss unit %s (tier boss)" % boss_id] = str(trig["kind"]) == EncounterPhases.TRIGGER_HP_BELOW and str(trig["unit"]) == boss_id and boss_id.contains("_boss_")
		var on_boss: bool = true
		for fx: Variant in DictUtil.array_of(p["effects"]):
			on_boss = on_boss and str(DictUtil.dict_of(fx).get("unit", boss_id)) == boss_id
		checks["every effect targets the boss"] = on_boss
		var entry: Dictionary = {"enemies": foes, "wave_units": DictUtil.dict_of(on["base"]).get("wave_units", []), "phases": phases, "max_rounds": SimArgs.new().cap}
		var problems: Array[String] = EncounterPhases.problems(entry, func(_ref: String) -> bool: return true)
		checks["the phase passes the BOSS1 validator %s" % str(problems)] = problems.is_empty()
		checks["BOSS_PHASE itself keeps the placeholder (copies are filled)"] = str(DictUtil.dict_of(SimScenarios.BOSS_PHASE["trigger"])["unit"]) == SimScenarios.BOSS_REF
	var forked: bool = TestHelpers.fork(SANDBOX)
	checks["sandbox fork"] = forked
	if forked:
		var fired: Dictionary = {}
		for phase_on: bool in [true, false]:
			var args: SimArgs = _sim_args(["party", "5", "1", "jitter=off", "cap=6"])
			var r := SimRunner.new(args)
			var obs := CountingMetrics.new()
			obs.quiet = true
			r.observer = obs
			var party: Array[String] = SimScenarios.define_party(r, ["warrior", "ranger", "mage", "rogue"] as Array[String], StatRules.MAX_LEVEL)
			var enc: Dictionary = SimScenarios.encounter(r, "boss", 5, 0, false, "warrior", "brawler", phase_on)
			var e_ids: Array[String] = []
			e_ids.assign(DictUtil.array_of(enc["enemies"]))
			var n: int = 0
			for s: int in SimArgs.new().seeds():
				r.fight(party, e_ids, s, DictUtil.dict_of(enc["base"]))
				n += 1 if obs.phase_fired else 0
			fired[phase_on] = n
		checks["phase on: fired in some fight (%d)" % DictUtil.to_int(fired[true])] = DictUtil.to_int(fired[true]) > 0
		checks["phase off: never fired"] = DictUtil.to_int(fired[false]) == 0
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "BAL5-PH generated boss phase (B2): one hp_below phase on the boss, phase=off restores BAL3", checks)

# A fight-set fixture of `count` enemies of class / tier at `level`.
static func _foes(runner: SimRunner, tag: String, class_id: String, tier: String, level: int, count: int) -> Array[String]:
	var ids: Array[String] = []
	for i: int in range(count):
		var id: String = "bal5_%s_%d" % [tag, i]
		runner.define(id, class_id, level, tier, false)
		ids.append(id)
	return ids

# K3 won-fight split: units_won holds exactly the cleared fights (an easy set adds to it, a hopeless one
# does not); the tank band is read on the won-fight KPI, the all-fight KPI has its own floor band.
static func _test_won_split() -> bool:
	var checks: Dictionary = {}
	var ks: Array[Dictionary] = SimRoles.kpis("tank")
	var bands: Dictionary = {}
	for k: Dictionary in ks:
		bands[str(k["id"])] = str(k["band"])
	checks["tank: allies hp_end won-fight KPI carries tank_ally_hp"] = str(bands.get("ally_hp_won", "")) == "tank_ally_hp"
	checks["tank: all-fight KPI carries tank_ally_hp_all"] = str(bands.get("ally_hp", "")) == "tank_ally_hp_all" and SimTargets.DEFAULTS.has("tank_ally_hp_all")
	checks["all-fight floor below the won-fight floor"] = SimTargets.lo(SimTargets.defaults(), "tank_ally_hp_all") < SimTargets.lo(SimTargets.defaults(), "tank_ally_hp")
	var forked: bool = TestHelpers.fork(SANDBOX)
	checks["sandbox fork"] = forked
	if forked:
		var args: SimArgs = _sim_args(["role", "5", "2", "jitter=off", "cap=4"])
		var runner := SimRunner.new(args)
		var metrics := SimMetrics.new()
		metrics.quiet = true
		runner.observer = metrics
		var party: Array[String] = SimScenarios.define_party(runner, ["warrior", "ranger"] as Array[String], StatRules.MAX_LEVEL)
		var easy: Array[String] = _foes(runner, "easy", "rogue", "minion", 1, 1)
		var wall: Array[String] = _foes(runner, "wall", "warrior", "boss", StatRules.MAX_LEVEL, 4)
		var acc: Dictionary = SimScenarios.empty_set()
		SimScenarios.run_seeds(runner, metrics, party, easy, {}, args.seeds(), "", acc)
		var after_easy: float = DictUtil.to_float(acc["cleared"])
		checks["easy set cleared every fight (%d of %d)" % [int(after_easy), int(DictUtil.to_float(acc["fights"]))]] = after_easy > 0.0 and after_easy == DictUtil.to_float(acc["fights"])
		var refs: Array[String] = [party[1]]
		var v_easy: Dictionary = {"refs": SimScenarios.sum_units(acc, refs), "refs_won": SimScenarios.sum_units(acc, refs, "units_won")}
		checks["every fight won: won-fight KPI = all-fight KPI"] = is_equal_approx(SimRoles.kpi_value("ally_hp_won", v_easy), SimRoles.kpi_value("ally_hp", v_easy))
		SimScenarios.run_seeds(runner, metrics, party, wall, {}, args.seeds(), "", acc)
		checks["hopeless set added fights, no clears"] = DictUtil.to_float(acc["fights"]) > after_easy and DictUtil.to_float(acc["cleared"]) == after_easy
		var won_fights: float = DictUtil.to_float(DictUtil.dict_of(DictUtil.dict_of(acc["units_won"]).get(party[0], {})).get("fights", 0.0))
		var all_fights: float = DictUtil.to_float(DictUtil.dict_of(DictUtil.dict_of(acc["units"]).get(party[0], {})).get("fights", 0.0))
		checks["units_won counts the cleared fights only (%d of %d)" % [int(won_fights), int(all_fights)]] = won_fights == DictUtil.to_float(acc["cleared"]) and all_fights == DictUtil.to_float(acc["fights"])
		var v: Dictionary = {"refs": SimScenarios.sum_units(acc, refs), "refs_won": SimScenarios.sum_units(acc, refs, "units_won")}
		checks["losses lower the all-fight KPI, not the won-fight one"] = SimRoles.kpi_value("ally_hp", v) < SimRoles.kpi_value("ally_hp_won", v) and is_equal_approx(SimRoles.kpi_value("ally_hp_won", v), SimRoles.kpi_value("ally_hp_won", v_easy))
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "BAL5 K3 won-fight split of allies hp_end", checks)

# K3 healer metrics: heal_casts = the heal entries an actor logged; downed / down_round = the first round
# at 0 HP (independent count); down_early = downed before EARLY_DOWN_ROUND; support KPIs read them.
static func _test_heal_down_metrics() -> bool:
	var checks: Dictionary = {}
	var ids: Array[String] = []
	for k: Dictionary in SimRoles.kpis("support"):
		ids.append(str(k["id"]))
	checks["support KPIs list heal casts, early down %, down round"] = ids.has("heal_casts") and ids.has("down_early") and ids.has("down_round")
	var forked: bool = TestHelpers.fork(SANDBOX)
	checks["sandbox fork"] = forked
	if forked:
		var args: SimArgs = _sim_args(["role", "5", "3", "jitter=off"])
		var runner := SimRunner.new(args)
		var obs := CountingMetrics.new()
		obs.quiet = true
		runner.observer = obs
		var party: Array[String] = SimScenarios.define_party(runner, ["priest", "warrior"] as Array[String], 5)
		var foes: Array[String] = _foes(runner, "press", "brawler", "standard", 6, 3)
		var heal_total: int = 0
		var downs_total: int = 0
		var casts_ok: bool = true
		var down_ok: bool = true
		for s: int in args.seeds():
			var res: Dictionary = runner.fight(party, foes, s)
			if not DictUtil.to_bool(res.get("ok", false)):
				continue
			for n: Variant in obs.units:
				var rec: Dictionary = DictUtil.dict_of(obs.units[n])
				var expected_casts: int = DictUtil.int_of(obs.heal_entries, n)
				heal_total += expected_casts
				casts_ok = casts_ok and DictUtil.int_of(rec, "heal_casts") == expected_casts
				var downed: bool = obs.downs.has(n)
				downs_total += 1 if downed else 0
				var round_at: int = DictUtil.int_of(obs.downs, n) if downed else 0
				down_ok = down_ok and DictUtil.int_of(rec, "downed") == (1 if downed else 0) and DictUtil.int_of(rec, "down_round") == round_at \
					and DictUtil.int_of(rec, "down_early") == (1 if downed and round_at < SimMetrics.EARLY_DOWN_ROUND else 0)
		checks["heal_casts = heal entries logged per actor (%d entries)" % heal_total] = casts_ok
		checks["the fixture heals (not vacuous)"] = heal_total > 0
		checks["downed / down_round / down_early match the first 0-HP round"] = down_ok
		checks["the fixture downs a unit (%d)" % downs_total] = downs_total > 0
		var t: Dictionary = {"heal_casts": 6.0, "fights": 3.0, "downed": 2.0, "down_round": 7.0, "down_early": 1.0}
		var v: Dictionary = {"test": t}
		checks["KPI heal casts/f = casts / fights"] = is_equal_approx(SimRoles.kpi_value("heal_casts", v), 2.0)
		checks["KPI down round = mean over downed fights"] = is_equal_approx(SimRoles.kpi_value("down_round", v), 3.5)
		checks["KPI down early % over all fights"] = is_equal_approx(SimRoles.kpi_value("down_early", v), 100.0 / 3.0)
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "BAL5 K3 healer metrics: heal casts, down round", checks)

# K6: the reach texts of CombatPlan (too far, no path) are reach errors; others are not.
static func _test_reach_skip() -> bool:
	var checks: Dictionary = {}
	for text: String in ["Too far: 9 tiles to the target (move cost 9), A can move 3 cost per block for 2 blocks in a round.", "No path to that tile.",
			"No path to get A within reach of B.", "B is too far for Lunge: 5 tiles to cross (move cost 5), it covers 2."]:
		checks["reach: %s" % text.left(30)] = SimRunner.is_reach_error(text)
	for text: String in ["Not enough AP for Strike.", "", "Target is not an enemy."]:
		checks["not reach: '%s'" % text.left(30)] = not SimRunner.is_reach_error(text)
	var r := SimRunner.new(_sim_args(["party"]))
	checks["a new runner has no skips or fails"] = r.plan_skips == 0 and r.plan_fails == 0
	return TestHelpers.report(TAG, "BAL5 K6 jitter PLAN SKIP: reach refusals are skipped fights", checks)
