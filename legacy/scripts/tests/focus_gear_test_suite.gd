class_name FocusGearTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Focus gear test suite (C1-L: orb / tome off-hand weapons, item `effects`)
# Item validation of the orb / tome rules, the item effects grammar (skill_potency family or group,
# status_potency, spell_reach), hand rules (a two-handed main weapon refuses an orb / tome), runtime
# hooks (power, heal, status chance, spell reach; equip and unequip are exact) and the reach clamp.
# Expectations come from the fixture entries and the rules, never from shipped numbers. Fixtures live in
# a sandbox fork of the catalog. The GearBudget weights and the shipped C1 asset checks moved to
# FocusGearBudgetTestSuite (CLN-12), which reuses seed_fixtures and the public helpers here.
# ==============================================================================

static var SANDBOX: String = TestHelpers.sandbox("focus_gear_test")

const TAG: String = "FocusGear"

static func has_error(errors: Array[String], fragment: String) -> bool:
	for err: String in errors:
		if err.contains(fragment):
			return true
	return false

static func near(a: float, b: float) -> bool:
	return absf(a - b) < 0.0001

# A valid orb entry; `extra` overrides fields.
static func focus(id: String, kind: String, extra: Dictionary = {}) -> Dictionary:
	var e: Dictionary = {"id": id, "name": id, "item_type": "WEAPON", "slot": "OFF_HAND", "weapon_type": kind,
		"off_hand_category": kind, "base_power": 0, "base_range": 1, "scaling_stat": "INT"}
	e.merge(extra, true)
	return ContentSchema.normalize("items", e)

static func _effect_value(entry: Dictionary, type: String, key: String = "") -> float:
	for raw: Variant in DictUtil.array_of(entry["effects"]):
		var fx: Dictionary = DictUtil.dict_of(raw)
		if str(fx.get("type", "")) == type and (key.is_empty() or fx.has(key)):
			return DictUtil.to_float(fx.get("value", 0.0))
	return 0.0

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING FOCUS GEAR TEST (C1-L)                  ===")
	print("=======================================================")
	if not TestHelpers.fork(SANDBOX):
		TestHelpers.cleanup(SANDBOX)
		return TestHelpers.report(TAG, "sandbox fork", {"forked": false})
	seed_fixtures()
	var ok: bool = true
	ok = _test_item_rules() and ok
	ok = _test_effect_grammar() and ok
	ok = _test_hands() and ok
	ok = _test_potency_and_heal() and ok
	ok = _test_status_potency() and ok
	ok = _test_spell_reach() and ok
	TestHelpers.cleanup(SANDBOX)
	return ok

static func seed_fixtures() -> void:
	var spell: Dictionary = {"type": "skill_potency", "group": "spell", "value": 0.10}
	var heal: Dictionary = {"type": "skill_potency", "group": "heal", "value": 0.10}
	var reach: Dictionary = {"type": "spell_reach", "value": 1}
	var status: Dictionary = {"type": "status_potency", "value": 5}
	var entries: Array[Dictionary] = [
		focus("t_orb", "orb", {"effects": [spell, reach], "required_stats": {"INT": 8}}),
		focus("t_tome", "tome", {"effects": [heal, status]}),
		ContentSchema.normalize("items", {"id": "t_reach_ring", "name": "Reach ring", "item_type": "ACCESSORY", "slot": "ACCESSORY_1", "effects": [reach]}),
		ContentSchema.normalize("items", {"id": "t_sword", "name": "Sword", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10}),
	]
	for entry: Dictionary in entries:
		var errs: Array[String] = ContentCatalog.upsert("items", entry)
		if not errs.is_empty():
			print("    [FocusGear] fixture %s refused: %s" % [str(entry["id"]), str(errs)])

static func _hero(class_id: String, main_id: String, off_id: String = "", extra_ids: Array[String] = []) -> Combatant:
	var c: Combatant = ContentFactory.unit({"id": "t_" + class_id, "name": class_id, "class_id": class_id, "level": 3, "stat_mode": "derived"})
	for slot: Variant in c.inventory.equipped_gear.keys():
		c.inventory.unequip(slot as Equipment.EquipSlot)
	_equip(c, main_id, Equipment.EquipSlot.MAIN_HAND)
	if not off_id.is_empty():
		_equip(c, off_id, Equipment.EquipSlot.OFF_HAND)
	for id: String in extra_ids:
		_equip(c, id, Equipment.EquipSlot.ACCESSORY_1)
	c.sync_from_stats_and_gear()
	return c

static func _equip(c: Combatant, item_id: String, slot: Equipment.EquipSlot) -> void:
	var eq: Equipment = ItemCatalog.create_equipment(item_id)
	c.inventory.equip(eq, slot)

# ------------------------------------------------------------------------------
# Item validation: one negative test per rule row, and the catalog sweep
# ------------------------------------------------------------------------------

static func _test_item_rules() -> bool:
	var c: Dictionary = {}
	for kind: String in Equipment.FOCUS_TYPES:
		var good: Dictionary = focus("t_new_" + kind, kind)
		c["valid %s accepted" % kind] = ContentCatalog.validate_entry("items", good).is_empty()
		if ContentCatalog.items_inherit():
			_family_rules(c, kind)
			continue
		c["%s must be off hand" % kind] = has_error(ContentCatalog.validate_entry("items", focus("t_x_" + kind, kind, {"slot": "MAIN_HAND"})), "off-hand only")
		var other: String = "tome" if kind == "orb" else "orb"
		c["%s category must match" % kind] = has_error(ContentCatalog.validate_entry("items", focus("t_y_" + kind, kind, {"off_hand_category": other})), "needs off-hand category")
		c["%s empty category refused" % kind] = has_error(ContentCatalog.validate_entry("items", focus("t_y2_" + kind, kind, {"off_hand_category": ""})), "needs off-hand category")
		c["reverse match for category %s" % kind] = has_error(ContentCatalog.validate_entry("items", focus("t_z_" + kind, "sword", {"off_hand_category": kind})), "needs the matching gear type")
		c["%s never two-handed" % kind] = has_error(ContentCatalog.validate_entry("items", focus("t_w_" + kind, kind, {"two_handed": true})), "never two-handed")
		c["%s weapon only" % kind] = has_error(ContentCatalog.validate_entry("items", focus("t_v_" + kind, kind, {"item_type": "SHIELD"})), "item type WEAPON")
	# Sweep: every catalog item with an orb / tome type validates exactly when the rule table holds.
	var sweep_ok: bool = true
	for entry: Dictionary in ContentCatalog.entries("items"):
		var e: Dictionary = ContentSchema.normalize("items", entry)
		if not Equipment.FOCUS_TYPES.has(str(e["weapon_type"])):
			continue
		var rules_hold: bool = str(e["slot"]) == "OFF_HAND" and str(e["off_hand_category"]) == str(e["weapon_type"]) and not DictUtil.to_bool(e["two_handed"]) and str(e["item_type"]) == "WEAPON"
		var errors: Array[String] = ContentCatalog.validate_entry("items", e, str(e["id"]))
		sweep_ok = sweep_ok and (errors.is_empty() == rules_hold)
	c["catalog sweep: valid exactly when the rules hold"] = sweep_ok
	c["validate_all stays clean"] = ContentCatalog.validate_all().is_empty()
	return TestHelpers.report(TAG, "orb / tome item rules", c)

# With item families (FORGE1_ITEMS, after ITEM1-A) a stored category / slot mismatch can no longer be expressed: the
# orb / tome rules are family rules (the `focus` engine rule forces hands = off on a WEAPON family) and item hands.
static func _family_rules(c: Dictionary, kind: String) -> void:
	var family: Dictionary = ContentCatalog.get_entry("item_families", kind)
	var stored: Dictionary = {"id": "t_fam_" + kind, "name": kind, "family": kind, "modifiers": [], "tiers": []}
	for hand: String in ItemFamilies.MAIN_HANDS:
		var wrong: Dictionary = stored.duplicate(true)
		wrong["hand"] = hand
		c["%s refuses hand %s (off-hand only, never two-handed)" % [kind, hand]] = has_error(ContentCatalog.validate_entry("items", wrong), "not allowed by family")
	var flat_main: Dictionary = focus("t_x_" + kind, kind, {"slot": "MAIN_HAND", "off_hand_category": ""})
	c["a flat %s without its category (a main-hand weapon) is still refused" % kind] = has_error(ContentCatalog.validate_entry("items", flat_main), "not allowed by family")
	var resolved: Dictionary = ItemModifiers.resolve(ItemModifiers.collapse(focus("t_y_" + kind, kind), family), family)
	c["%s resolves to the off hand and its own category" % kind] = str(resolved["slot"]) == "OFF_HAND" and str(resolved["off_hand_category"]) == kind and not DictUtil.to_bool(resolved["two_handed"])
	var main_focus: Dictionary = family.duplicate(true)
	main_focus["hands"] = [ItemFamilies.HAND_MAIN_1H]
	c["%s family: focus needs hands = off" % kind] = has_error(ContentCatalog.validate_entry("item_families", main_focus, kind), "focus needs")
	var shield_focus: Dictionary = family.duplicate(true)
	shield_focus["item_type"] = "SHIELD"
	c["%s family: focus is a WEAPON rule" % kind] = has_error(ContentCatalog.validate_entry("item_families", shield_focus, kind), "focus needs")
	c["%s family carries the focus rule" % kind] = ItemFamilies.has_rule(family, ItemFamilies.RULE_FOCUS)

# ------------------------------------------------------------------------------
# Effects grammar on items and passives
# ------------------------------------------------------------------------------

static func _test_effect_grammar() -> bool:
	var c: Dictionary = {}
	var exists: Callable = func(id: String) -> bool: return SkillCatalog.find(id) != null
	var item_errs: Callable = func(fx: Array) -> Array[String]: return Passives.validate_effects(fx, exists, true, true)
	var bad_types: Array[Dictionary] = [
		{"type": "stat", "stat": "INT", "value": 1}, {"type": "proficiency", "gear": "sword", "levels": 1},
		{"type": "ap_regen", "value": 1}, {"type": "skill", "skill": "strike"}]
	var all_rejected: bool = true
	for fx: Dictionary in bad_types:
		all_rejected = all_rejected and has_error(item_errs.call([fx]), "not allowed on items")
	c["stat / proficiency / ap_regen / skill rejected on items"] = all_rejected
	c["empty item effects allowed"] = (item_errs.call([]) as Array[String]).is_empty()
	c["a passive still needs an effect"] = not Passives.validate_effects([], exists).is_empty()
	c["family bonus validates"] = (item_errs.call([{"type": "skill_potency", "family": "strike", "value": 0.1}]) as Array[String]).is_empty()
	for group: String in Passives.GROUPS:
		c["group %s validates" % group] = (item_errs.call([{"type": "skill_potency", "group": group, "value": 0.1}]) as Array[String]).is_empty()
	c["both family and group refused"] = not (item_errs.call([{"type": "skill_potency", "family": "strike", "group": "spell", "value": 0.1}]) as Array[String]).is_empty()
	c["neither family nor group refused"] = not (item_errs.call([{"type": "skill_potency", "value": 0.1}]) as Array[String]).is_empty()
	c["unknown group refused"] = not (item_errs.call([{"type": "skill_potency", "group": "nope", "value": 0.1}]) as Array[String]).is_empty()
	c["group value range as families"] = not (item_errs.call([{"type": "skill_potency", "group": "spell", "value": Passives.POTENCY_MAX + 0.1}]) as Array[String]).is_empty() and not (item_errs.call([{"type": "skill_potency", "group": "spell", "value": 0.0}]) as Array[String]).is_empty()
	c["status_potency bounds"] = (item_errs.call([{"type": "status_potency", "value": Passives.STATUS_POTENCY_MIN}]) as Array[String]).is_empty() and (item_errs.call([{"type": "status_potency", "value": Passives.STATUS_POTENCY_MAX}]) as Array[String]).is_empty() and not (item_errs.call([{"type": "status_potency", "value": Passives.STATUS_POTENCY_MIN - 1}]) as Array[String]).is_empty() and not (item_errs.call([{"type": "status_potency", "value": Passives.STATUS_POTENCY_MAX + 1}]) as Array[String]).is_empty()
	c["spell_reach bounds"] = (item_errs.call([{"type": "spell_reach", "value": Passives.SPELL_REACH_MIN}]) as Array[String]).is_empty() and (item_errs.call([{"type": "spell_reach", "value": Passives.SPELL_REACH_MAX}]) as Array[String]).is_empty() and not (item_errs.call([{"type": "spell_reach", "value": Passives.SPELL_REACH_MIN - 1}]) as Array[String]).is_empty() and not (item_errs.call([{"type": "spell_reach", "value": Passives.SPELL_REACH_MAX + 1}]) as Array[String]).is_empty()
	c["resist allowed on items"] = (item_errs.call([{"type": "resist", "status": "stagger", "value": 10}]) as Array[String]).is_empty()
	c["catalog refuses a bad item effect"] = has_error(ContentCatalog.validate_entry("items", focus("t_bad_fx", "orb", {"effects": [{"type": "ap_regen", "value": 1}]})), "not allowed on items")
	c["catalog accepts an armor with effects"] = ContentCatalog.validate_entry("items", ContentSchema.normalize("items", {"id": "t_arm_fx", "name": "x", "item_type": "ARMOR", "slot": "BODY", "weapon_type": Proficiency.ARMOR_TYPES[0], "effects": [{"type": "status_potency", "value": 3}]})).is_empty()
	c["effects round-trip through the factory"] = ContentFactory.item_to_entry(ItemCatalog.create_equipment("t_orb"))["effects"] == ContentCatalog.get_entry("items", "t_orb")["effects"]
	c["describe covers group, status, reach"] = Passives.describe({"effects": [{"type": "skill_potency", "group": "heal", "value": 0.1}, {"type": "status_potency", "value": 5}, {"type": "spell_reach", "value": 1}]}).contains("heal") and Passives.describe({"effects": [{"type": "spell_reach", "value": 1}]}).contains("reach")
	return TestHelpers.report(TAG, "item effects grammar", c)

# ------------------------------------------------------------------------------
# Hand rules
# ------------------------------------------------------------------------------

static func _test_hands() -> bool:
	var c: Dictionary = {}
	var orb: Equipment = ItemCatalog.create_equipment("t_orb")
	var tome: Equipment = ItemCatalog.create_equipment("t_tome")
	c["orb and tome are the off hand"] = orb.hand() == Equipment.HAND_OFF and tome.hand() == Equipment.HAND_OFF
	var shipped_ok: bool = true
	for id: String in ["wpn_orb", "wpn_tome"]:
		if ContentCatalog.has("items", id):
			var s: Equipment = ItemCatalog.create_equipment(id)
			shipped_ok = shipped_ok and s.hand() == Equipment.HAND_OFF and s.can_equip_in_slot(Equipment.EquipSlot.OFF_HAND) and not s.can_equip_in_slot(Equipment.EquipSlot.MAIN_HAND)
	c["shipped orb / tome are off hand only"] = shipped_ok
	# A mage with a two-handed staff cannot equip the orb or tome; with a one-handed weapon it can.
	var staff: Equipment = null
	var wand: Equipment = null
	for entry: Dictionary in ContentCatalog.entries("items"):
		var e: Dictionary = ContentSchema.normalize("items", entry)
		if str(e["item_type"]) == "WEAPON" and str(e["weapon_type"]) == "staff" and DictUtil.to_bool(e["two_handed"]) and staff == null:
			staff = ItemCatalog.create_equipment(str(e["id"]))
		if str(e["item_type"]) == "WEAPON" and str(e["weapon_type"]) == "wand" and not DictUtil.to_bool(e["two_handed"]) and wand == null:
			wand = ItemCatalog.create_equipment(str(e["id"]))
	c["a two-handed staff and a one-handed wand exist"] = staff != null and wand != null
	if staff == null or wand == null:
		return TestHelpers.report(TAG, "hand rules", c)
	var stash := Inventory.new()
	stash.add_item(orb)
	stash.add_item(tome)
	var mage: Combatant = _hero("mage", str(wand.id))
	var session: DojoSession = DojoSession.wrap_party([mage] as Array[Combatant], stash)
	c["mage with a wand is offered the orb"] = session.get_eligible_stash_items(Equipment.EquipSlot.OFF_HAND).has(orb)
	c["mage with a wand equips the orb"] = session.equip_item(orb, Equipment.EquipSlot.OFF_HAND)
	c["main hand is still the casting weapon"] = mage.inventory.get_main_weapon().id == wand.id and mage.wielded_type() == "wand"
	# Equipping the two-handed staff stows the orb (the off hand must be empty of focus gear).
	session.shared_stash.add_item(staff)
	c["equipping the staff succeeds"] = session.equip_item(staff, Equipment.EquipSlot.MAIN_HAND)
	c["the orb was stowed by the two-handed staff"] = mage.inventory.get_equipped(Equipment.EquipSlot.OFF_HAND) == null
	c["a staff mage is not offered the tome"] = not session.get_eligible_stash_items(Equipment.EquipSlot.OFF_HAND).has(tome)
	var refused: bool = not session.equip_item(tome, Equipment.EquipSlot.OFF_HAND)
	c["a staff mage cannot equip the tome"] = refused and mage.inventory.get_equipped(Equipment.EquipSlot.OFF_HAND) == null
	c["conflicts_with_main follows the main hand"] = tome.conflicts_with_main(staff) and not tome.conflicts_with_main(wand) and not tome.conflicts_with_main(null) and not wand.conflicts_with_main(staff)
	# Casting gate reads the main hand only: a skill gated on 'orb' is never usable with an orb in the off hand.
	var gated: SkillDef = SkillDef.create("t_orb_gated", "Gated", SkillDef.Kind.ATTACK, 1, 1, "x")
	gated.weapon_types = ["orb"]
	gated.min_proficiency = 1
	var holder: Combatant = _hero("mage", str(wand.id), "t_orb")
	holder.known_skills.append("t_orb_gated")
	c["holder wields the orb in the off hand"] = holder.inventory.get_equipped(Equipment.EquipSlot.OFF_HAND) != null
	c["an orb in the off hand never satisfies weapon_types"] = not holder.can_use(gated) and holder.wielded_type() == "wand"
	# The orb does not make the wielder ranged and the off-hand jab with power 0 does not crash.
	var melee: Combatant = _hero("warrior", "t_sword")
	var melee_orb: Combatant = _hero("warrior", "t_sword", "t_orb")
	c["orb leaves attack range alone"] = melee.attack_range == melee_orb.attack_range
	c["orb does not make a ranged wielder"] = not melee_orb.inventory.get_main_weapon().is_ranged_weapon() and not orb.is_ranged_weapon() and not tome.is_ranged_weapon()
	var dummy := Combatant.new("Dummy", 500, 5, 1)
	var jab: Dictionary = DamageCalculator.compute(melee_orb, dummy, false, true, 1.0, false)
	c["off-hand jab with a power 0 orb resolves"] = jab.has("final_dmg") and melee_orb.get_offhand_power() >= 0
	return TestHelpers.report(TAG, "hand rules", c)

# ------------------------------------------------------------------------------
# Potency and heal hooks
# ------------------------------------------------------------------------------

static func _test_potency_and_heal() -> bool:
	var c: Dictionary = {}
	var orb_entry: Dictionary = ContentCatalog.get_entry("items", "t_orb")
	var tome_entry: Dictionary = ContentCatalog.get_entry("items", "t_tome")
	var spell_v: float = _effect_value(orb_entry, "skill_potency", "group")
	var heal_v: float = _effect_value(tome_entry, "skill_potency", "group")
	var bolt: SkillDef = SkillCatalog.find("arcane_bolt")
	var mend: SkillDef = SkillCatalog.find("mend")
	var strike: SkillDef = SkillCatalog.find("strike")
	c["skills resolve"] = bolt != null and mend != null and strike != null
	if bolt == null or mend == null or strike == null:
		return TestHelpers.report(TAG, "potency and heal", c)
	c["groups are derived from the skill"] = bolt.in_group("spell") and not bolt.in_group("heal") and mend.in_group("heal") and not mend.in_group("spell") and not strike.in_group("spell") and not strike.in_group("heal")
	var wand_id: String = ""
	for entry: Dictionary in ContentCatalog.entries("items"):
		if str(entry.get("weapon_type", "")) == "wand" and not DictUtil.to_bool(entry.get("two_handed", false)) and str(entry.get("slot", "")) == "MAIN_HAND":
			wand_id = str(entry["id"])
			break
	var mage: Combatant = _hero("mage", wand_id)
	var bolt_base: float = bolt.power_for(mage)
	var strike_base: float = strike.power_for(mage)
	var mend_base: int = SupportRules.heal_amount(mage, mend)
	var foe := Combatant.new("Foe", 500, 5, 1)
	var dmg_base: Dictionary = DamageCalculator.compute(mage, foe, false, false, 1.0, false)
	c["no bonus: potency_for_skill = potency_for"] = near(mage.potency_for_skill(bolt), mage.potency_for(bolt.family_id()))
	_equip(mage, "t_orb", Equipment.EquipSlot.OFF_HAND)
	mage.sync_from_stats_and_gear()
	c["orb raises spell power by exactly x(1 + value)"] = near(bolt.power_for(mage), bolt_base * (1.0 + spell_v))
	c["orb does not change a non-spell skill"] = near(strike.power_for(mage), strike_base)
	c["orb does not change the heal"] = SupportRules.heal_amount(mage, mend) == mend_base
	var dmg_orb: Dictionary = DamageCalculator.compute(mage, foe, false, false, 1.0, false)
	c["orb leaves DamageCalculator.compute alone"] = UiTheme.dict_int(dmg_orb, "final_dmg") == UiTheme.dict_int(dmg_base, "final_dmg")
	mage.inventory.unequip(Equipment.EquipSlot.OFF_HAND)
	mage.sync_from_stats_and_gear()
	c["unequipping the orb returns the power exactly"] = bolt.power_for(mage) == bolt_base and mage.gear_potency_group.is_empty() and mage.gear_spell_reach == 0
	# Tome: heal group.
	_equip(mage, "t_tome", Equipment.EquipSlot.OFF_HAND)
	mage.sync_from_stats_and_gear()
	var stat_value: int = mage.stats.get_effective_stat(mend.heal_stat, mage.get_weapon_context())
	var base_heal: int = mend.heal_base + roundi(float(stat_value) * mend.heal_mult)
	c["tome raises the heal by x(1 + value)"] = SupportRules.heal_amount(mage, mend) == roundi(float(base_heal) * (1.0 + heal_v))
	c["tome does not change spell power"] = near(bolt.power_for(mage), bolt_base)
	c["tome does not change strike"] = near(strike.power_for(mage), strike_base)
	var dmg_tome: Dictionary = DamageCalculator.compute(mage, foe, false, false, 1.0, false)
	c["tome leaves DamageCalculator.compute alone"] = UiTheme.dict_int(dmg_tome, "final_dmg") == UiTheme.dict_int(dmg_base, "final_dmg")
	mage.inventory.unequip(Equipment.EquipSlot.OFF_HAND)
	mage.sync_from_stats_and_gear()
	c["unequipping the tome returns the heal exactly"] = SupportRules.heal_amount(mage, mend) == mend_base
	# Repeated syncs do not stack, and a family bonus from a passive stays separate from gear.
	_equip(mage, "t_orb", Equipment.EquipSlot.OFF_HAND)
	mage.sync_from_stats_and_gear()
	mage.sync_from_stats_and_gear()
	c["repeated sync does not stack gear bonuses"] = near(bolt.power_for(mage), bolt_base * (1.0 + spell_v))
	Passives.apply(mage, {"id": "t_c1_pass", "name": "x", "effects": [{"type": "skill_potency", "group": "spell", "value": 0.05}]})
	c["passive and gear group bonuses add"] = near(bolt.power_for(mage), bolt_base * (1.0 + spell_v + 0.05))
	mage.inventory.unequip(Equipment.EquipSlot.OFF_HAND)
	mage.sync_from_stats_and_gear()
	c["unequipping subtracts only the gear part"] = near(bolt.power_for(mage), bolt_base * 1.05)
	return TestHelpers.report(TAG, "potency and heal hooks", c)

# ------------------------------------------------------------------------------
# status_potency reaches the attack command (not apply_chance)
# ------------------------------------------------------------------------------

static func _test_status_potency() -> bool:
	var c: Dictionary = {}
	var tome_entry: Dictionary = ContentCatalog.get_entry("items", "t_tome")
	var points: int = roundi(_effect_value(tome_entry, "status_potency"))
	var hex: SkillDef = SkillCatalog.find("hex")
	var strike: SkillDef = SkillCatalog.find("strike")
	c["hex applies a status"] = hex != null and not hex.status_effect.is_empty()
	if hex == null or hex.status_effect.is_empty():
		return TestHelpers.report(TAG, "status potency", c)
	var wand_id: String = ""
	for entry: Dictionary in ContentCatalog.entries("items"):
		if str(entry.get("weapon_type", "")) == "wand" and not DictUtil.to_bool(entry.get("two_handed", false)) and str(entry.get("slot", "")) == "MAIN_HAND":
			wand_id = str(entry["id"])
			break
	var priest: Combatant = _hero("priest", wand_id)
	var foe := Combatant.new("Foe", 500, 5, 1)
	var grid: CombatGrid = CoreRulesTestSuite._grid()
	grid.place_combatant(priest, Vector2i(3, 4))
	grid.place_combatant(foe, Vector2i(4, 4))
	var entry := PlanEntry.create(hex, 1, 0)
	entry.target = foe
	var base_potency: int = entry.build_command(priest, grid).status_potency
	var base_chance: int = StatusEffects.apply_chance(foe, hex.status_effect, base_potency)
	_equip(priest, "t_tome", Equipment.EquipSlot.OFF_HAND)
	priest.sync_from_stats_and_gear()
	var tome_potency: int = entry.build_command(priest, grid).status_potency
	c["the command gets the tome's points"] = tome_potency == base_potency + points
	var chance: int = StatusEffects.apply_chance(foe, hex.status_effect, tome_potency)
	c["hit chance rises by the points (before the clamp)"] = chance == clampi(roundi(float(tome_potency) - foe.debuff_resistance()), 0, 100)
	c["chance never drops"] = chance >= base_chance
	c["apply_chance itself is unchanged"] = StatusEffects.apply_chance(foe, hex.status_effect, base_potency) == base_chance
	var strike_entry := PlanEntry.create(strike, 1, 0)
	strike_entry.target = foe
	c["no status, no bonus"] = strike_entry.build_command(priest, grid).status_potency == strike.status_potency
	priest.inventory.unequip(Equipment.EquipSlot.OFF_HAND)
	priest.sync_from_stats_and_gear()
	c["unequip returns the command potency"] = entry.build_command(priest, grid).status_potency == base_potency
	# Gear resist is rebuilt too.
	ContentCatalog.upsert("items", ContentSchema.normalize("items", {"id": "t_resist_ring", "name": "x", "item_type": "ACCESSORY", "slot": "ACCESSORY_1", "effects": [{"type": "resist", "status": "stagger", "value": 20}]}))
	var before: int = priest.resist_for("stagger")
	_equip(priest, "t_resist_ring", Equipment.EquipSlot.ACCESSORY_1)
	priest.sync_from_stats_and_gear()
	c["gear resist adds while equipped"] = priest.resist_for("stagger") == mini(100, before + 20) and priest.status_resist.get("stagger", 0) == 0
	priest.inventory.unequip(Equipment.EquipSlot.ACCESSORY_1)
	priest.sync_from_stats_and_gear()
	c["gear resist leaves with the item"] = priest.resist_for("stagger") == before
	return TestHelpers.report(TAG, "status potency and gear resist", c)

# ------------------------------------------------------------------------------
# Spell reach
# ------------------------------------------------------------------------------

static func _spell(id: String, area: SkillDef.Area, reach_min: int, reach_max: int, types: Array[String]) -> SkillDef:
	var s: SkillDef = SkillDef.create(id, id, SkillDef.Kind.ATTACK, 1, 1, "x")
	s.area = area
	s.reach_min = reach_min
	s.reach_max = reach_max
	s.weapon_types = types
	return s

static func _test_spell_reach() -> bool:
	var c: Dictionary = {}
	var wand_id: String = ""
	for entry: Dictionary in ContentCatalog.entries("items"):
		if str(entry.get("weapon_type", "")) == "wand" and not DictUtil.to_bool(entry.get("two_handed", false)) and str(entry.get("slot", "")) == "MAIN_HAND":
			wand_id = str(entry["id"])
			break
	var mage: Combatant = _hero("mage", wand_id)
	var staves: Array[String] = ["wand"]
	var plain: SkillDef = _spell("t_plain", SkillDef.Area.NONE, 0, 0, staves)
	var plus: SkillDef = _spell("t_plus", SkillDef.Area.PLUS, 2, 3, staves)
	var burst: SkillDef = _spell("t_burst", SkillDef.Area.BURST, 0, 0, staves)
	var line: SkillDef = _spell("t_line", SkillDef.Area.LINE, 0, 0, staves)
	var sweep: SkillDef = _spell("t_sweep", SkillDef.Area.SWEEP, 0, 0, staves)
	var non_spell: SkillDef = _spell("t_basic", SkillDef.Area.NONE, 0, 0, [])
	var skills: Array[SkillDef] = [plain, plus, burst, line, sweep, non_spell]
	var base: Dictionary = {}
	for s: SkillDef in skills:
		base[s.id] = s.reach_for(mage)
	_equip(mage, "t_orb", Equipment.EquipSlot.OFF_HAND)
	mage.sync_from_stats_and_gear()
	var bonus: int = mage.spell_reach_bonus()
	c["one orb gives the bonus from its entry"] = bonus == roundi(_effect_value(ContentCatalog.get_entry("items", "t_orb"), "spell_reach"))
	for s: SkillDef in [plain, plus, burst]:
		var was: Vector2i = base[s.id]
		c["%s: max reach +bonus, min unchanged" % s.id] = s.reach_for(mage) == Vector2i(was.x, was.y + bonus)
	c["default band: (default min, default max + bonus)"] = plain.reach_for(mage).x == (base["t_plain"] as Vector2i).x
	c["LINE never extended"] = line.reach_for(mage) == base["t_line"]
	c["SWEEP never extended"] = sweep.reach_for(mage) == base["t_sweep"]
	c["non-spell skills never extended"] = non_spell.reach_for(mage) == base["t_basic"]
	mage.inventory.unequip(Equipment.EquipSlot.OFF_HAND)
	mage.sync_from_stats_and_gear()
	var back: bool = true
	for s: SkillDef in skills:
		back = back and s.reach_for(mage) == base[s.id]
	c["unequip restores every band"] = back
	# Clamp: the orb plus a reach ring is two tiles of raw gear reach, clamped to the cap.
	var clamped: Combatant = _hero("mage", wand_id, "t_orb", ["t_reach_ring"])
	c["raw gear reach adds up"] = clamped.gear_spell_reach == 2 * bonus
	c["total gear reach is clamped"] = clamped.spell_reach_bonus() == mini(2 * bonus, Passives.GEAR_REACH_CAP) and plain.reach_for(clamped) == Vector2i((base["t_plain"] as Vector2i).x, (base["t_plain"] as Vector2i).y + Passives.GEAR_REACH_CAP)
	return TestHelpers.report(TAG, "spell reach", c)
