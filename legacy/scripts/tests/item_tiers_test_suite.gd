class_name ItemTiersTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Item tiers suite (FORGE1_ITEMS 4: tier engine; ITEM1-L commit 2)
# On a sandbox fork with item families: each criterion (stat, proficiency, unit level) opens and closes its tier; each
# bonus (power, mitigation, stat bonuses, effects) reaches the engine; tiers never open other tiers (no circularity);
# static gear bonuses count, tier bonuses do not (evaluation order); items without tiers or with power-only tiers give
# identical stats and effects; shipped loadouts carry no non-power tier contribution; GearBudget.tier_potential is
# W_TIER x the tier value and never in item_value; Passives.effect_specs matches validate_effects. Fixtures are test-only
# stored items on families picked by rule; thresholds derive from a reference hero, never from shipped numbers.
# ==============================================================================

static var SANDBOX: String = TestHelpers.sandbox("item_tiers")
const TAG: String = "ItemTiers"

static var _weapon: Dictionary = {}
static var _armor: Dictionary = {}
static var _s_weapon: int = 0   # reference effective STR in the weapon context, proficiency 0 (tier criteria base)
static var _s_armor: int = 0

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING ITEM TIERS TEST (ITEM1-L commit 2)      ===")
	print("=======================================================")
	TestHelpers.fork_migrated(SANDBOX)
	if not ContentCatalog.items_inherit():
		TestHelpers.cleanup(SANDBOX)
		return TestHelpers.report_ok(TAG, "sandbox fork with item families", false)
	var ok: bool = _test_shipped_loadouts()   # before the fixtures exist: shipped rows only
	ok = _seed() and ok
	ok = _test_criteria() and ok
	ok = _test_bonuses() and ok
	ok = _test_no_circularity() and ok
	ok = _test_order() and ok
	ok = _test_identical_without_tiers() and ok
	ok = _test_resolved_rows() and ok
	ok = _test_tier_potential() and ok
	ok = _test_effect_specs() and ok
	TestHelpers.cleanup(SANDBOX)
	return ok

# ------------------------------------------------------------------------------
# Fixtures
# ------------------------------------------------------------------------------

# A main-category melee weapon family allowing every modifier kind with room for 3 tiers.
static func _pick_weapon() -> Dictionary:
	for f: Dictionary in ContentCatalog.entries("item_families"):
		var kinds: Array[String] = ContentFactory._strings(f.get("modifier_kinds", []))
		if ItemFamilies.item_type_of(f) == "WEAPON" and ItemFamilies.category(f) == "main" and kinds.size() == ItemModifiers.KINDS.size() \
				and DictUtil.to_int(f.get("max_tiers", 0)) >= 3 and not ContentFactory._strings(f.get("engine_rules", [])).has("always_ranged"):
			return f
	return {}

# An armor family with room for 2 tiers.
static func _pick_armor() -> Dictionary:
	for f: Dictionary in ContentCatalog.entries("item_families"):
		if ItemFamilies.item_type_of(f) == "ARMOR" and DictUtil.to_int(f.get("max_tiers", 0)) >= 2:
			return f
	return {}

static func _tier(when: Dictionary, bonus: Dictionary, label: String = "T") -> Dictionary:
	return {"label": label, "when": when, "bonus": bonus, "desc": ""}

static func _stat(stat: String, value: int) -> Dictionary:
	return {"type": ItemModifiers.WHEN_STAT, "stat": stat, "value": value}

static func _prof(level: int, gear: String = "") -> Dictionary:
	var w: Dictionary = {"type": ItemModifiers.WHEN_PROFICIENCY, "level": level}
	if not gear.is_empty():
		w["gear"] = gear
	return w

static func _level(value: int) -> Dictionary:
	return {"type": ItemModifiers.WHEN_LEVEL, "value": value}

static func _stored(id: String, family: Dictionary, tiers: Array, modifiers: Array = []) -> Dictionary:
	var e: Dictionary = {"id": id, "name": "T " + id, "family": str(family["id"]), "modifiers": modifiers, "tiers": tiers}
	if not ItemFamilies.hands_of(family).is_empty():
		e["hand"] = ItemFamilies.default_hand(family)
	var slots: Array[String] = ContentFactory._strings(family.get("slots", []))
	if not slots.is_empty():
		e["slot"] = slots[0]
	return e

# A level-1 hero (default archetype) with an empty proficiency table, `prof_levels` gear -> level.
static func _hero(level: int = 1, prof_levels: Dictionary = {}) -> Combatant:
	var st := CharacterStats.new(null, 1)
	st.level = level
	st.proficiency = Proficiency.new()
	for gear: Variant in prof_levels:
		st.proficiency.set_entry(str(gear), DictUtil.to_int(prof_levels[gear]))
	var c := Combatant.new("t_tier_hero", 100, 10, 1, st)
	c.sync_from_stats_and_gear()
	return c

# Typed int lists compared by value.
static func _is(a: Array[int], b: Array) -> bool:
	return str(a) == str(b)

# The worn piece built from catalog id `id` (null when not worn).
static func _worn(c: Combatant, id: String) -> Equipment:
	for eq: Equipment in c.inventory._iter_equipped_gear():
		if eq.id == id:
			return eq
	return null

static func _eq(id: String) -> Equipment:
	var it: Item = ContentFactory.item(ContentCatalog.get_entry("items", id))
	return it as Equipment

static func _wear(c: Combatant, ids: Array[String]) -> void:
	for id: String in ids:
		var eq: Equipment = _eq(id)
		c.inventory.equip(eq, eq.slot)
	c.sync_from_stats_and_gear()

# Tier bonuses used by the fixtures (the expectations read these, never literals elsewhere).
const POWER: int = 3
const DEX_BONUS: int = 2
const STATUS: int = 5
const MIT: int = 4
const STR_BONUS: int = 3

static func _seed() -> bool:
	_weapon = _pick_weapon()
	_armor = _pick_armor()
	if _weapon.is_empty() or _armor.is_empty():
		return TestHelpers.report_ok(TAG, "fixture families by rule", false, "weapon %s, armor %s" % [str(_weapon.get("id", "-")), str(_armor.get("id", "-"))])
	var ref: Combatant = _hero()
	_s_weapon = ref.stats.get_effective_stat("STR", {"weapon": str(_weapon["id"])})
	_s_armor = ref.stats.get_effective_stat("STR", {"weapon": str(_armor["id"])})
	var wfam: String = str(_weapon["id"])
	var status_fx: Dictionary = {"type": "status_potency", "value": STATUS}
	var entries: Array[Dictionary] = [
		# criteria: stat one above the reference, own-family proficiency 1, unit level 2 (a level-1 hero meets none)
		_stored("t_tier_crit", _weapon, [_tier(_stat("STR", _s_weapon + 1), {"power": POWER}, "Edge"),
			_tier(_prof(1), {"stat_bonuses": {"DEX": DEX_BONUS}}, "Drilled"),
			_tier(_level(2), {"effects": [status_fx]}, "Veteran")]),
		_stored("t_tier_mit", _armor, [_tier(_level(2), {"mitigation": MIT}, "Seasoned")]),
		# circularity: tier 1 (level 1, always met) gives STR; tier 2 needs exactly that STR
		_stored("t_tier_loop", _weapon, [_tier(_level(1), {"stat_bonuses": {"STR": STR_BONUS}}, "Strong"),
			_tier(_stat("STR", _s_weapon + 1), {"power": POWER}, "Loop")]),
		_stored("t_tier_loop_arm", _armor, [_tier(_stat("STR", _s_armor + 1), {"mitigation": MIT}, "Loop")]),
		# order: a static +2 STR modifier counts toward the item's own stat tier
		_stored("t_tier_static", _weapon, [_tier(_stat("STR", _s_weapon + 2), {"power": POWER}, "Static")],
			[{"kind": ItemModifiers.STAT_BONUS, "stat": "STR", "value": 2}]),
		# identical: power-only tier (met) versus the same item without tiers
		_stored("t_tier_pow", _weapon, [_tier(_stat("STR", _s_weapon), {"power": POWER}, "Pow")]),
		_stored("t_tier_none", _weapon, []),
		# an explicit gear token other than the own family
		_stored("t_tier_other", _weapon, [_tier(_prof(1, _other_gear(wfam)), {"power": POWER}, "Cross")]),
	]
	var refused: Array[String] = []
	for entry: Dictionary in entries:
		var errs: Array[String] = ContentCatalog.upsert("items", entry)
		if not errs.is_empty():
			refused.append("%s: %s" % [str(entry["id"]), str(errs)])
	return TestHelpers.report_ok(TAG, "fixtures (%s, %s; STR ref %d / %d)" % [wfam, str(_armor["id"]), _s_weapon, _s_armor], refused.is_empty(), "; ".join(refused))

static func _other_gear(own: String) -> String:
	for g: String in Proficiency.gear_types():
		if g != own:
			return g
	return own

# ------------------------------------------------------------------------------
# 1. Criteria
# ------------------------------------------------------------------------------

static func _test_criteria() -> bool:
	var checks: Dictionary = {}
	var wfam: String = str(_weapon["id"])
	var eq: Equipment = _eq("t_tier_crit")
	var base: Combatant = _hero()
	checks["a level-1 hero without training meets no criterion"] = eq.get_active_tier_numbers(base.stats, base.stats.proficiency).is_empty()
	var buffed: Combatant = _hero()
	buffed.stats.add_modifier(StatModifier.new("STR", 1, 0.0, "t_buff", "always", -1))
	checks["stat_at_least: one more STR opens tier 1 only"] = _is(eq.get_active_tier_numbers(buffed.stats, buffed.stats.proficiency), [1])
	var trained: Combatant = _hero(1, {wfam: 1})
	# training also lowers stat criteria by LEVEL_DISCOUNT per level, so tier 1 opens too (spec 4.2 stat rule)
	var want: Array = [2]
	if Proficiency.LEVEL_DISCOUNT >= 1:
		want = [1, 2]
	checks["proficiency_at_least (own family by default) opens tier 2; the discount opens the stat tier"] = _is(eq.get_active_tier_numbers(trained.stats, trained.stats.proficiency), want)
	var veteran: Combatant = _hero(2)
	checks["unit_level_at_least: level 2 opens tier 3 only"] = _is(eq.get_active_tier_numbers(veteran.stats, veteran.stats.proficiency), [3])
	var other: Equipment = _eq("t_tier_other")
	var own_only: Combatant = _hero(1, {wfam: Proficiency.MAX_LEVEL})
	var cross: Combatant = _hero(1, {_other_gear(wfam): 1})
	checks["an explicit gear token reads that gear, not the item's family"] = other.get_active_tier_numbers(own_only.stats, own_only.stats.proficiency).is_empty() \
		and _is(other.get_active_tier_numbers(cross.stats, cross.stats.proficiency), [1])
	checks["no stats: no tier"] = eq.get_active_tier_numbers(null, null).is_empty()
	var legacy_rows: Array[Dictionary] = [{"stat": "STR", "threshold": _s_weapon, "bonus_power": POWER, "desc": "Old"}]
	var legacy := Equipment.new("t_legacy", "Legacy", Equipment.EquipSlot.MAIN_HAND, wfam, Item.ItemType.WEAPON, 10, {}, legacy_rows)
	checks["a legacy {stat, threshold, bonus_power} row still works"] = legacy.get_unlocked_threshold_power_boost(base.stats, base.stats.proficiency) == POWER
	var text: String = eq.get_tier_requirements_display()
	checks["the requirement line names the new criteria"] = text.contains("training 1") and text.contains("unit level 2")
	return TestHelpers.report(TAG, "Criteria (stat, proficiency, unit level)", checks)

# ------------------------------------------------------------------------------
# 2. Bonuses
# ------------------------------------------------------------------------------

static func _test_bonuses() -> bool:
	var checks: Dictionary = {}
	var wfam: String = str(_weapon["id"])
	# power: through Equipment and the DamageCalculator inputs (formula unchanged)
	var plain: Combatant = _hero()
	_wear(plain, ["t_tier_crit"])
	var buffed: Combatant = _hero()
	buffed.stats.add_modifier(StatModifier.new("STR", 1, 0.0, "t_buff", "always", -1))
	_wear(buffed, ["t_tier_crit"])
	var w_plain: Equipment = plain.inventory.get_main_weapon()
	var w_buffed: Equipment = buffed.inventory.get_main_weapon()
	checks["power: the open tier adds its power"] = w_buffed.get_effective_equipment_power(buffed.stats, buffed.stats.proficiency) \
		- w_plain.get_effective_equipment_power(plain.stats, plain.stats.proficiency) == POWER
	var res: Dictionary = DamageCalculator.attack_power(buffed, w_buffed.is_ranged_weapon())
	checks["power: DamageCalculator reads the tier boost"] = DictUtil.to_int(res.get("tier_boost", -1)) == POWER \
		and DictUtil.to_int(res.get("total_w_power", -1)) == w_buffed.base_power + POWER + DictUtil.to_int(res.get("training_pwr", 0))
	# stat bonuses: proficiency tier 2
	var trained: Combatant = _hero(1, {wfam: 1})
	var dex_before: int = trained.stats.get_effective_stat("DEX")
	_wear(trained, ["t_tier_crit"])
	checks["stat_bonuses: the open tier adds DEX as a %s modifier" % Equipment.TIER_SOURCE] = trained.stats.get_effective_stat("DEX") == dex_before + DEX_BONUS \
		and _source_count(trained.stats, Equipment.TIER_SOURCE) == 1
	trained.inventory.unequip(Equipment.EquipSlot.MAIN_HAND)
	trained.sync_from_stats_and_gear()
	checks["stat_bonuses: unequipping removes them"] = trained.stats.get_effective_stat("DEX") == dex_before and _source_count(trained.stats, Equipment.TIER_SOURCE) == 0
	# effects: unit-level tier 3
	var veteran: Combatant = _hero(2)
	var pot_before: int = veteran.status_potency_bonus()
	_wear(veteran, ["t_tier_crit"])
	checks["effects: the open tier applies its effect"] = veteran.status_potency_bonus() == pot_before + STATUS
	veteran.sync_from_stats_and_gear()
	checks["effects: a second sync does not stack them"] = veteran.status_potency_bonus() == pot_before + STATUS
	var closed: Combatant = _hero(1)
	_wear(closed, ["t_tier_crit"])
	checks["effects: a closed tier applies nothing"] = closed.status_potency_bonus() == pot_before
	# mitigation: armor unit-level tier
	var low: Combatant = _hero(1)
	_wear(low, ["t_tier_mit"])
	var high: Combatant = _hero(2)
	_wear(high, ["t_tier_mit"])
	var body: Equipment = _worn(low, "t_tier_mit")
	checks["mitigation: the open tier adds to the armor rating input"] = high.inventory.get_total_flat_mitigation(high.stats) == high.inventory.get_total_flat_mitigation() + MIT \
		and DamageCalculator.armor_rating(high) == high.stats.get_mitigation_rating(high.inventory.get_total_flat_mitigation() + MIT, high.get_weapon_context())
	checks["mitigation: closed, the rating is the plain item number"] = DamageCalculator.armor_rating(low) == low.stats.get_mitigation_rating(low.inventory.get_total_flat_mitigation(), low.get_weapon_context()) \
		and body != null and body.get_effective_mitigation(low.stats, low.stats.proficiency) == body.mitigation
	return TestHelpers.report(TAG, "Bonuses (power, stat bonuses, effects, mitigation)", checks)

static func _source_count(st: CharacterStats, source: String) -> int:
	var n: int = 0
	for mod: StatModifier in st.modifiers:
		if mod.source_id == source:
			n += 1
	return n

# ------------------------------------------------------------------------------
# 3. No circularity, 4. evaluation order
# ------------------------------------------------------------------------------

static func _test_no_circularity() -> bool:
	var checks: Dictionary = {}
	var c: Combatant = _hero()
	var str_before: int = c.stats.get_effective_stat("STR")
	_wear(c, ["t_tier_loop", "t_tier_loop_arm"])
	var w: Equipment = c.inventory.get_main_weapon()
	var a: Equipment = _worn(c, "t_tier_loop_arm")
	checks["the tier stat bonus applies"] = c.stats.get_effective_stat("STR") == str_before + STR_BONUS
	checks["it does not open the item's own stat tier"] = w != null and _is(w.get_active_tier_numbers(c.stats, c.stats.proficiency), [1]) \
		and w.get_unlocked_threshold_power_boost(c.stats, c.stats.proficiency) == 0
	checks["it does not open another item's stat tier"] = a != null and a.get_tier_mitigation_bonus(c.stats, c.stats.proficiency) == 0
	var snap: Dictionary = _snapshot(c)
	c.sync_from_stats_and_gear()
	c.sync_from_stats_and_gear()
	checks["repeated syncs give the same result"] = ItemModifiers.same_value(_snapshot(c), snap)
	return TestHelpers.report(TAG, "No circularity (tier bonuses never open tiers)", checks)

static func _test_order() -> bool:
	var checks: Dictionary = {}
	var c: Combatant = _hero()
	var eq: Equipment = _eq("t_tier_static")
	checks["before the gear sync the static bonus is missing, the tier is closed"] = eq.get_active_tier_numbers(c.stats, c.stats.proficiency).is_empty()
	_wear(c, ["t_tier_static"])
	var w: Equipment = c.inventory.get_main_weapon()
	checks["static gear stat bonuses count toward stat tiers (synced first)"] = w != null and _is(w.get_active_tier_numbers(c.stats, c.stats.proficiency), [1]) \
		and w.get_unlocked_threshold_power_boost(c.stats, c.stats.proficiency) == POWER
	return TestHelpers.report(TAG, "Evaluation order (static gear -> tiers -> tier stats -> effects)", checks)

# Everything a tier could move besides weapon power.
static func _snapshot(c: Combatant) -> Dictionary:
	var stats: Dictionary = {}
	for s: String in StatRules.STATS:
		stats[s] = c.stats.get_effective_stat(s, c.get_weapon_context())
	var sources: Array[String] = []
	for mod: StatModifier in c.stats.modifiers:
		sources.append("%s:%s:%d" % [mod.source_id, mod.stat_name, mod.flat_bonus])
	sources.sort()
	return {"stats": stats, "hp": c.max_hp, "speed": c.speed, "range": c.attack_range, "armor": DamageCalculator.armor_rating(c),
		"potency_family": c.gear_potency_family.duplicate(), "potency_group": c.gear_potency_group.duplicate(),
		"resist": c.gear_status_resist.duplicate(), "status": c.gear_status_potency, "reach": c.gear_spell_reach, "mods": sources}

# ------------------------------------------------------------------------------
# 5. Identical without tiers, 6. shipped loadouts
# ------------------------------------------------------------------------------

static func _test_identical_without_tiers() -> bool:
	var checks: Dictionary = {}
	var with_tier: Combatant = _hero()
	_wear(with_tier, ["t_tier_pow"])
	var without: Combatant = _hero()
	_wear(without, ["t_tier_none"])
	checks["a power-only tier leaves stats and effects identical to no tiers"] = ItemModifiers.same_value(_snapshot(with_tier), _snapshot(without))
	var wp: Equipment = with_tier.inventory.get_main_weapon()
	var wn: Equipment = without.inventory.get_main_weapon()
	checks["only the weapon power differs, by the tier"] = wp.get_effective_equipment_power(with_tier.stats, with_tier.stats.proficiency) \
		== wn.get_effective_equipment_power(without.stats, without.stats.proficiency) + POWER
	checks["no tier stat modifiers without stat tiers"] = _source_count(with_tier.stats, Equipment.TIER_SOURCE) == 0
	return TestHelpers.report(TAG, "Items without tiers or with power-only tiers are unchanged", checks)

# Every shipped class and unit at the maximum level (the most tiers in reach): no tier stat modifier, no tier
# mitigation, no tier effect. With every shipped tier power-only (checked first), shipped numbers cannot move.
static func _test_shipped_loadouts() -> bool:
	var checks: Dictionary = {}
	var extended: Array[String] = []
	for row: Dictionary in ContentCatalog.entries("items"):
		for raw: Variant in DictUtil.array_of(row.get("threshold_tiers", [])):
			# before commit 2 only stat criteria resolved and only power was read: anything else would now act
			var bonus: Dictionary = Equipment.tier_bonus(DictUtil.dict_of(raw))
			var when: Dictionary = Equipment.tier_when(DictUtil.dict_of(raw))
			if bonus.size() != 1 or not bonus.has("power") or str(when.get("type", "")) != ItemModifiers.WHEN_STAT:
				extended.append(str(row["id"]))
	var hits: Array[String] = []
	var built: int = 0
	var specs: Array[Dictionary] = []
	for cls: Dictionary in ContentCatalog.entries("classes"):
		specs.append({"id": "t_tier_cls", "name": "C", "class_id": str(cls["id"]), "level": StatRules.MAX_LEVEL, "stat_mode": "derived"})
	for unit: Dictionary in ContentCatalog.entries("units"):
		var e: Dictionary = unit.duplicate(true)
		e["level"] = StatRules.MAX_LEVEL
		specs.append(e)
	for spec: Dictionary in specs:
		var c: Combatant = ContentFactory.unit(spec)
		if c == null:
			continue
		built += 1
		for eq: Equipment in c.inventory._iter_equipped_gear():
			var fx: int = 0
			for n: int in eq.get_active_tier_numbers(c.stats, c.stats.proficiency):
				fx += eq.tier_effects(n).size()
			if not eq.get_tier_stat_bonuses(c.stats, c.stats.proficiency).is_empty() or eq.get_tier_mitigation_bonus(c.stats, c.stats.proficiency) != 0 or fx > 0:
				hits.append("%s/%s" % [str(spec.get("class_id", spec.get("id", ""))), eq.id])
		if _source_count(c.stats, Equipment.TIER_SOURCE) > 0:
			hits.append("%s: tier modifiers" % str(spec.get("class_id", spec.get("id", ""))))
	print("    [%s] shipped: %d loadouts built, extended-tier items %s" % [TAG, built, str(extended)])
	checks["every shipped tier is power-only (or its effect is listed here)"] = extended.is_empty()
	checks["no shipped class or unit loadout gets a non-power tier contribution"] = built > 0 and hits.is_empty()
	return TestHelpers.report(TAG, "Shipped loadouts unchanged (%s)" % (", ".join(hits) if not hits.is_empty() else "no hits"), checks)

# ------------------------------------------------------------------------------
# 7. Resolved rows (every criterion reaches threshold_tiers; the flat row round-trips)
# ------------------------------------------------------------------------------

static func _test_resolved_rows() -> bool:
	var checks: Dictionary = {}
	var row: Dictionary = ContentCatalog.get_entry("items", "t_tier_crit")
	var legacy: Array = DictUtil.array_of(row.get("threshold_tiers", []))
	var stored: Array[Dictionary] = ItemModifiers.tiers_of(row)
	checks["every stored tier resolves to a threshold tier, in order"] = legacy.size() == stored.size()
	var keys_ok: bool = true
	for i: int in range(mini(legacy.size(), stored.size())):
		var t: Dictionary = DictUtil.dict_of(legacy[i])
		var is_stat: bool = str(DictUtil.dict_of(stored[i]["when"]).get("type", "")) == ItemModifiers.WHEN_STAT
		keys_ok = keys_ok and t.has("stat") and t.has("threshold") and t.has("bonus_power") and t.has("desc")
		keys_ok = keys_ok and ItemModifiers.same_value(Equipment.tier_when(t), stored[i]["when"]) and ItemModifiers.same_value(Equipment.tier_bonus(t), stored[i]["bonus"])
		keys_ok = keys_ok and (is_stat or (str(t["stat"]).is_empty() and DictUtil.to_int(t["threshold"]) == 0))
	checks["legacy keys kept; when/bonus read back; non-stat rows have stat \"\""] = keys_ok
	var family: Dictionary = ContentCatalog.family_of(row)
	var again: Dictionary = ContentSchema.normalize("items", ItemModifiers.resolve(ItemModifiers.collapse(row, family), family))
	checks["resolve(collapse(row)) keeps every tier"] = ItemModifiers.engine_equal(again, row) and ItemModifiers.same_value(again["tiers"], row["tiers"])
	checks["re-upserting the flat row changes nothing"] = ContentCatalog.upsert("items", row).is_empty() \
		and ItemModifiers.same_value(ContentCatalog.get_entry("items", "t_tier_crit")["tiers"], row["tiers"])
	var arm: Equipment = _eq("t_tier_mit")
	checks["armor keeps its tiers on the engine item"] = arm.threshold_tiers.size() == ItemModifiers.tiers_of(ContentCatalog.get_entry("items", "t_tier_mit")).size()
	return TestHelpers.report(TAG, "Resolved rows (threshold_tiers carry every criterion)", checks)

# ------------------------------------------------------------------------------
# 8. GearBudget.tier_potential
# ------------------------------------------------------------------------------

static func _test_tier_potential() -> bool:
	var checks: Dictionary = {}
	var row: Dictionary = ContentCatalog.get_entry("items", "t_tier_crit")
	var raw: float = POWER * GearBudget.W_POWER + DEX_BONUS * GearBudget.W_STAT_POINT + GearBudget.effect_value({"type": "status_potency", "value": STATUS})
	checks["tier_potential = W_TIER x the tier bonuses at the static weights"] = is_equal_approx(GearBudget.tier_potential(row), raw * GearBudget.W_TIER)
	var arm: Dictionary = ContentCatalog.get_entry("items", "t_tier_mit")
	checks["mitigation bonuses use the mitigation weight"] = is_equal_approx(GearBudget.tier_potential(arm), MIT * GearBudget.W_MITIGATION * GearBudget.W_TIER)
	checks["the stored entry and the resolved row agree"] = is_equal_approx(GearBudget.tier_potential(ItemModifiers.stored(row)), GearBudget.tier_potential(row))
	var legacy_only: Dictionary = {"threshold_tiers": [{"stat": "STR", "threshold": 10, "bonus_power": POWER, "desc": ""}]}
	checks["a legacy power row counts its bonus_power"] = is_equal_approx(GearBudget.tier_potential(legacy_only), POWER * GearBudget.W_POWER * GearBudget.W_TIER)
	checks["no tiers: 0"] = is_zero_approx(GearBudget.tier_potential(ContentCatalog.get_entry("items", "t_tier_none")))
	checks["tiers never enter item_value"] = is_equal_approx(GearBudget.item_value(ContentCatalog.get_entry("items", "t_tier_pow")), GearBudget.item_value(ContentCatalog.get_entry("items", "t_tier_none")))
	return TestHelpers.report(TAG, "GearBudget.tier_potential (informational)", checks)

# ------------------------------------------------------------------------------
# 9. Passives.effect_specs
# ------------------------------------------------------------------------------

static func _test_effect_specs() -> bool:
	var checks: Dictionary = {}
	var all: Array[Dictionary] = Passives.effect_specs()
	var types: Array[String] = []
	for spec: Dictionary in all:
		types.append(str(spec["type"]))
	checks["one spec per effect type, in TYPES order"] = types == Passives.TYPES
	var item_types: Array[String] = []
	for spec: Dictionary in Passives.effect_specs(true):
		item_types.append(str(spec["type"]))
	var sorted_items: Array[String] = Passives.ITEM_TYPES.duplicate()
	sorted_items.sort()
	item_types.sort()
	checks["items_only lists exactly ITEM_TYPES"] = item_types == sorted_items
	var any_skill: Callable = func(_id: String) -> bool: return true
	var bad: Array[String] = []
	for spec: Dictionary in all:
		var type: String = str(spec["type"])
		var items_only: bool = DictUtil.to_bool(spec["item"])
		for edge: String in ["min", "max"]:
			var fx: Dictionary = _sample(spec, edge)
			if not Passives.validate_effects([fx], any_skill, true, items_only).is_empty():
				bad.append("%s at %s refused" % [type, edge])
		for p: Variant in DictUtil.array_of(spec["params"]):
			var param: Dictionary = DictUtil.dict_of(p)
			if str(param["kind"]) != "int":
				continue
			for outside: int in [DictUtil.to_int(param["min"]) - 1, DictUtil.to_int(param["max"]) + 1]:
				var fx_out: Dictionary = _sample(spec, "min")
				fx_out[str(param["key"])] = outside
				if _bounded(type, str(param["key"])) and Passives.validate_effects([fx_out], any_skill, true, items_only).is_empty():
					bad.append("%s %s %d accepted" % [type, str(param["key"]), outside])
		var choices: Array[String] = []
		for p: Variant in DictUtil.array_of(spec["params"]):
			if DictUtil.dict_of(p).has("choice"):
				choices.append(str(DictUtil.dict_of(p)["key"]))
		if choices.size() > 1:
			var both: Dictionary = _sample(spec, "min")
			for key: String in choices:
				both[key] = _value_of(_param(spec, key), "min")
			if Passives.validate_effects([both], any_skill, true, items_only).is_empty():
				bad.append("%s with every choice param accepted" % type)
	checks["spec limits match validate_effects (edges pass, outside fails, choices exclusive)"] = bad.is_empty()
	checks["every spec has a label and typed params"] = _specs_typed(all)
	return TestHelpers.report(TAG, "Passives.effect_specs (%s)" % (", ".join(bad) if not bad.is_empty() else "consistent"), checks)

# The stat effect's value is only "non-zero" in validate_effects; its spec range is the widget range.
static func _bounded(type: String, key: String) -> bool:
	return not (type == "stat" and key == "value")

static func _param(spec: Dictionary, key: String) -> Dictionary:
	for p: Variant in DictUtil.array_of(spec["params"]):
		if str(DictUtil.dict_of(p)["key"]) == key:
			return DictUtil.dict_of(p)
	return {}

# A valid effect of `spec` with numbers at `edge` (min or max); of a choice group only the first param is set.
static func _sample(spec: Dictionary, edge: String) -> Dictionary:
	var fx: Dictionary = {"type": str(spec["type"])}
	var chosen: bool = false
	for p: Variant in DictUtil.array_of(spec["params"]):
		var param: Dictionary = DictUtil.dict_of(p)
		if DictUtil.to_bool(param.get("optional", false)):
			continue
		if param.has("choice"):
			if chosen:
				continue
			chosen = true
		fx[str(param["key"])] = _value_of(param, edge)
	return fx

static func _value_of(param: Dictionary, edge: String) -> Variant:
	match str(param["kind"]):
		"int":
			var v: int = DictUtil.to_int(param[edge])
			return v if v != 0 or not DictUtil.to_bool(param.get("nonzero", false)) else 1
		"float":
			return DictUtil.to_float(param[edge])
		"enum":
			var options: Array = DictUtil.array_of(param["options"])
			return options[0] if edge == "min" else options[options.size() - 1]
		"skill":
			return "t_any_skill"
	return null

static func _specs_typed(all: Array[Dictionary]) -> bool:
	for spec: Dictionary in all:
		if str(spec.get("label", "")).is_empty():
			return false
		for p: Variant in DictUtil.array_of(spec.get("params", [])):
			var param: Dictionary = DictUtil.dict_of(p)
			if not ["int", "float", "enum", "skill", "context"].has(str(param.get("kind", ""))):
				return false
			if (str(param["kind"]) == "int" or str(param["kind"]) == "float") and not (param.has("min") and param.has("max") and param.has("step")):
				return false
			if str(param["kind"]) == "enum" and DictUtil.array_of(param.get("options", [])).is_empty():
				return false
	return true
