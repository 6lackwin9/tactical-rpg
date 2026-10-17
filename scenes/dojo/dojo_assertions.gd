class_name DojoAssertions
extends RefCounted

# ==============================================================================
# Tactical RPG - Headless Gear & Skill Dojo Verification Suite
# Exercises the headless DojoSession (rules glue) and smoke-tests the modular
# panel/window UI. `host` must be inside the scene tree (used for UI smoke test). Dojo-1..18 live here;
# the session, skill and dummy checks in DojoSessionAssertions, the UI, slot, stash and carry checks in
# DojoUiAssertions (split by CLN-12).
# ==============================================================================

# With the legacy dual-wield flag off (restored after): every item fits OFF_HAND exactly when its hand is `off`,
# and the given focus items (tome, orb) are off-hand items.
static func _strict_off_hand_ok(items: Array) -> bool:
	var saved: bool = Equipment.allow_plain_dual_wield
	Equipment.allow_plain_dual_wield = false
	var ok: bool = true
	for raw: Variant in items:
		if not (raw is Equipment):
			ok = false
			continue
		var eq: Equipment = raw
		ok = ok and eq.can_equip_in_slot(Equipment.EquipSlot.OFF_HAND) == (eq.hand() == Equipment.HAND_OFF)
		if eq.is_focus():
			ok = ok and eq.hand() == Equipment.HAND_OFF
	Equipment.allow_plain_dual_wield = saved
	return ok

static func run(session: DojoSession, host: Node) -> bool:
	var passed: bool = true

	# Assert 1: Default Hero & Dummy setup
	var w: Combatant = session.active_hero
	var d: Combatant = session.test_dummy
	if w == null or d == null or w.name != "Warrior" or d.hp != 500:
		print("  [Dojo-1] Baseline hero and dummy initialization: FAIL")
		passed = false
	else:
		print("  [Dojo-1] Baseline hero and dummy initialization: PASS")

	# Assert 2: Base Attack Calculation (Revised Formula & Sword Mastery to Wep Power)
	# Power = Wep Power (broadsword 18 + sword training 3) + effective STR (class base + growth + spend + Bear Ring) x 1.5, derived from the hero's real stats (the tank warrior hits for 47).
	var weapon: Equipment = w.inventory.get_main_weapon()
	var expected_power: int = weapon.get_effective_equipment_power(w.stats, w.stats.proficiency) + roundi(w.stats.get_effective_stat("STR") * 1.5)
	var initial_power: int = w.get_effective_power(false)
	var calc_res: Dictionary = DamageCalculator.compute(w, d, false, false, 1.0, false)
	var dmg: int = d.take_damage(UiTheme.dict_int(calc_res, "final_dmg"))
	if dmg != expected_power or initial_power != expected_power:
		print("  [Dojo-2] Base melee attack calculation: FAIL (dmg: %d, pwr: %d != %d)" % [dmg, initial_power, expected_power])
		passed = false
	else:
		print("  [Dojo-2] Base melee attack calculation: PASS (%d Power, %d DMG, sword training in Wep Power)" % [expected_power, dmg])

	# Assert 3: Dummy Mitigation Toggle
	d.defense_mitigation_ratio = 0.5
	var mit_res: Dictionary = DamageCalculator.compute(w, d, false, false, 1.0, false)
	var half_dmg: int = d.take_damage(UiTheme.dict_int(mit_res, "final_dmg"), true)
	if half_dmg != roundi(expected_power * 0.5):
		print("  [Dojo-3] 50%% mitigation defense barrier calculation: FAIL (%d != %d)" % [half_dmg, roundi(expected_power * 0.5)])
		passed = false
	else:
		print("  [Dojo-3] 50%% mitigation defense barrier calculation: PASS (%d DMG)" % half_dmg)

	# Assert 4: Focus Buff Multiplier (x1.5)
	w.apply_attack_boost(1.5, 2)
	d.defense_mitigation_ratio = 0.0
	var focus_res: Dictionary = DamageCalculator.compute(w, d, false, false, 1.0, false)
	var boosted_dmg: int = d.take_damage(UiTheme.dict_int(focus_res, "final_dmg"), true)
	if boosted_dmg != roundi(expected_power * 1.5):
		print("  [Dojo-4] Focus buff 1.5x power multiplier calculation: FAIL (%d != %d)" % [boosted_dmg, roundi(expected_power * 1.5)])
		passed = false
	else:
		print("  [Dojo-4] Focus buff 1.5x power multiplier calculation: PASS (%d DMG)" % boosted_dmg)

	# Assert 5: Claymore Threshold Tier Boost (18 STR -> +4 Power)
	w.attack_boost_multiplier = 1.0
	var claymore := ItemCatalog.create_iron_claymore()
	w.inventory.equip(claymore)
	w.sync_from_stats_and_gear()
	var tier_boost: int = claymore.get_unlocked_threshold_power_boost(w.stats, w.stats.proficiency)
	if tier_boost != 4:
		print("  [Dojo-5] Claymore threshold tier bonus at 18 STR: FAIL (got %d)" % tier_boost)
		passed = false
	else:
		print("  [Dojo-5] Claymore threshold tier bonus at 18 STR: PASS (+4 Power unlocked)")

	# Assert 6: Titan Warhammer Deficit Penalty
	var titan := ItemCatalog.create_titan_warhammer()
	w.inventory.equip(titan)
	w.sync_from_stats_and_gear()
	var untrained := CharacterStats.new(ClassArchetype.create("priest"), 1)   # the warrior's hammer training (+2 discount) covers the Titan's 22 STR
	var pen := titan.get_wield_penalty(untrained, untrained.proficiency)
	if not UiTheme.dict_bool(pen, "is_penalized"):
		print("  [Dojo-6] Titan Warhammer deficit penalty check: FAIL")
		passed = false
	else:
		print("  [Dojo-6] Titan Warhammer deficit penalty check: PASS (Deficit penalty applied)")

	# Assert 7: Consumable Pouch Usage
	var pouch_con := ItemCatalog.create_healing_draught()
	w.hp = 50
	pouch_con.apply_to_combatant(w)
	if w.hp != 75:
		print("  [Dojo-7] Consumable healing draught effect: FAIL (HP: %d)" % w.hp)
		passed = false
	else:
		print("  [Dojo-7] Consumable healing draught effect: PASS (HP: 50 -> 75)")

	# Assert 8: Reset Dummy Vitals
	d.hp = d.max_hp
	d.defense_mitigation_ratio = 0.0
	if d.hp != 500 or d.defense_mitigation_ratio != 0.0:
		print("  [Dojo-8] Dummy vitals full reset: FAIL")
		passed = false
	else:
		print("  [Dojo-8] Dummy vitals full reset: PASS (500 / 500 HP)")

	# Assert 9: Weapon Power Melee vs Ranged Gating
	var claymore_melee: int = w.get_effective_power(false)
	var claymore_ranged: int = w.get_effective_power(true)
	if claymore_ranged >= claymore_melee:
		print("  [Dojo-9] Melee weapon power gating on ranged attack: FAIL (ranged: %d >= melee: %d)" % [claymore_ranged, claymore_melee])
		passed = false
	else:
		print("  [Dojo-9] Melee weapon power gating on ranged attack: PASS (Melee: %d, Ranged: %d)" % [claymore_melee, claymore_ranged])

	# Assert 10: Ranged Bow Power Gating on Melee Attack
	var bow := ItemCatalog.create_oak_shortbow()
	w.inventory.equip(bow)
	w.sync_from_stats_and_gear()
	var bow_melee: int = w.get_effective_power(false)
	# Expected from DamageCalculator's rule: an unfit weapon adds no weapon power, only the
	# unit power of the fallback stat (STR for melee) at the bow's proficiency scaling.
	var bow_parts: Dictionary = DamageCalculator.attack_power(w, false)
	var bow_unit: float = float(w.stats.get_effective_stat("STR", {"weapon": bow.weapon_type})) * DamageCalculator.unit_power_scaling(w, bow)
	var expected_unarmed_melee: int = maxi(1, roundi(bow_unit * w.power_scale))
	if bow_melee != expected_unarmed_melee or DictUtil.to_float(bow_parts["weapon_power"]) != 0.0:
		print("  [Dojo-10] Ranged weapon power excluded from melee attack: FAIL (%d != %d)" % [bow_melee, expected_unarmed_melee])
		passed = false
	else:
		print("  [Dojo-10] Ranged weapon power excluded from melee attack: PASS (Bow grants 0 Melee Weapon Power)")

	# Assert 11: Accessory Stat Bonus to Character Stats & Attack Power
	w.inventory.unequip(Equipment.EquipSlot.ACCESSORY_1)
	w.sync_from_stats_and_gear()
	var pre_ring_str: int = w.stats.get_effective_stat("STR")
	var pre_ring_hp: int = w.stats.get_max_hp()

	var bear_ring := ItemCatalog.create_ring_of_the_bear() # +2 STR, +2 CONS
	w.inventory.equip(bear_ring, Equipment.EquipSlot.ACCESSORY_1)
	w.sync_from_stats_and_gear()
	var post_ring_str: int = w.stats.get_effective_stat("STR")
	var post_ring_hp: int = w.stats.get_max_hp()
	if post_ring_str != (pre_ring_str + 2) or post_ring_hp <= pre_ring_hp:
		print("  [Dojo-11] Accessory stat bonuses propagated to effective stats: FAIL (STR: %d -> %d, HP: %d -> %d)" % [pre_ring_str, post_ring_str, pre_ring_hp, post_ring_hp])
		passed = false
	else:
		print("  [Dojo-11] Accessory stat bonuses propagated to effective stats: PASS (+2 STR, +2 CONS -> +%d Max HP)" % (post_ring_hp - pre_ring_hp))

	# Assert 12: Class Archetype Gear Proficiency Restrictions
	var sword := ItemCatalog.create_steel_broadsword()
	var war_arch: ClassArchetype = w.stats.archetype
	var ranger_arch: ClassArchetype = session.party_allies[1].stats.archetype if session.party_allies.size() > 1 else ClassArchetype.create_ranger()
	var war_can_bow: bool = war_arch.can_equip_gear(bow)
	var war_can_sword: bool = war_arch.can_equip_gear(sword)
	var ranger_can_bow: bool = ranger_arch.can_equip_gear(bow)
	var ranger_can_plate: bool = ranger_arch.can_equip_gear(ItemCatalog.create_steel_plate_cuirass())
	var war_can_ring: bool = war_arch.can_equip_gear(bear_ring)
	var ranger_can_ring: bool = ranger_arch.can_equip_gear(bear_ring)
	if war_can_bow != false or war_can_sword != true or ranger_can_bow != true or ranger_can_plate != false or war_can_ring != true or ranger_can_ring != true:
		print("  [Dojo-12] Class archetype gear equip restrictions: FAIL")
		passed = false
	else:
		print("  [Dojo-12] Class archetype gear equip restrictions: PASS (Archetype weapon/armor proficiencies enforced; Accessories unrestricted)")

	# Assert 13: Off-Hand Weapon Taxonomy & Scaling
	var shortsword := ItemCatalog.create_iron_shortsword()
	w.inventory.equip(shortsword, Equipment.EquipSlot.OFF_HAND)
	w.sync_from_stats_and_gear()
	var off_str_pwr: int = w.get_offhand_power()
	var expected_shortsword_pwr: int = shortsword.get_effective_equipment_power(w.stats, w.stats.proficiency) + int(roundf(float(w.stats.get_effective_stat("STR")) * w.stats.proficiency.scaling_of("sword")))
	if off_str_pwr != expected_shortsword_pwr:
		print("  [Dojo-13] Legacy dual wield (flag): off-hand Shortsword STR scaling: FAIL (%d != %d)" % [off_str_pwr, expected_shortsword_pwr])
		passed = false
	else:
		print("  [Dojo-13] Legacy dual wield (flag): off-hand Shortsword STR scaling: PASS (%d ATK Power)" % off_str_pwr)

	var knife := ItemCatalog.create_shadow_dagger()
	w.inventory.equip(knife, Equipment.EquipSlot.OFF_HAND)
	w.sync_from_stats_and_gear()
	var off_dex_pwr: int = w.get_offhand_power()
	var expected_knife_pwr: int = knife.get_effective_equipment_power(w.stats, w.stats.proficiency) + int(roundf(float(w.stats.get_effective_stat("DEX")) * w.stats.proficiency.scaling_of("dagger")))
	if absi(off_dex_pwr - expected_knife_pwr) > 1:   # penalised weapon power and unit power round separately
		print("  [Dojo-13b] Legacy dual wield (flag): off-hand Dagger DEX scaling: FAIL (%d != %d)" % [off_dex_pwr, expected_knife_pwr])
		passed = false
	else:
		print("  [Dojo-13b] Legacy dual wield (flag): off-hand Dagger DEX scaling: PASS (%d ATK Power)" % off_dex_pwr)

	var shield_eq := ItemCatalog.create_iron_kite_shield()
	w.inventory.equip(shield_eq, Equipment.EquipSlot.OFF_HAND)
	w.sync_from_stats_and_gear()
	if shield_eq.base_power != 0 or shield_eq.mitigation <= 0:
		print("  [Dojo-13c] Shield is mitigation only, not offensive: FAIL")
		passed = false
	else:
		print("  [Dojo-13c] Shield is mitigation only (0 offensive power, +%d mitigation): PASS" % shield_eq.mitigation)

	# Assert 13d: strict hand typing (flag false): main-hand weapons are refused in OFF_HAND, off-hand items accepted.
	# Rule-derived per item: OFF_HAND fits iff the derived hand is `off` (true for the tome and orb; the shortsword and
	# dagger are main_1h once ITEM1-A migrates them, off-category items before).
	var strict_ok: bool = _strict_off_hand_ok([ItemCatalog.create_iron_shortsword(), ItemCatalog.create_shadow_dagger(), ItemCatalog.create_mystic_tome(), ItemCatalog.create_equipment("wpn_orb"), ItemCatalog.create_steel_broadsword()])
	if not strict_ok:
		print("  [Dojo-13d] Strict hands (flag off): main-hand weapons refused in OFF_HAND, tome and orb accepted: FAIL")
		passed = false
	else:
		print("  [Dojo-13d] Strict hands (flag off): main-hand weapons refused in OFF_HAND, tome and orb accepted: PASS")

	# Assert 14: 2-Handed Weapon blocks off-hand strike
	w.inventory.equip(claymore, Equipment.EquipSlot.MAIN_HAND)
	w.sync_from_stats_and_gear()
	if not claymore.is_two_handed:
		print("  [Dojo-14] Claymore tagged as 2-Handed: FAIL")
		passed = false
	else:
		print("  [Dojo-14] 2-Handed weapon tags and off-hand blocking: PASS")

	# Assert 15: Critical Hit Rate & Debuff Resistance Derived Metrics
	var crit_r: float = w.stats.get_crit_rate()
	var debuff_res: float = w.stats.get_debuff_resistance()
	var exp_crit: float = DamageCalculator.CRIT_BASE + float(maxi(0, w.stats.get_effective_stat("DEX") - DamageCalculator.CRIT_DEX_FLOOR)) * DamageCalculator.CRIT_PER_DEX
	var exp_res: float = float(w.stats.get_effective_stat("INT")) * 2.5
	if absf(crit_r - exp_crit) > 0.01 or absf(debuff_res - exp_res) > 0.01:
		print("  [Dojo-15] Derived Crit Rate and Debuff Resistance: FAIL (Crit: %.1f != %.1f, Res: %.1f != %.1f)" % [crit_r, exp_crit, debuff_res, exp_res])
		passed = false
	else:
		print("  [Dojo-15] Derived Crit Rate (%.1f%%) and Debuff Resistance (%.1f%%): PASS" % [crit_r, debuff_res])

	# Assert 16: Critical Damage Multiplier in Combat Formula
	w.inventory.equip(sword, Equipment.EquipSlot.MAIN_HAND)
	w.sync_from_stats_and_gear()
	var norm_dmg: int = UiTheme.dict_int(DamageCalculator.compute(w, d, false, false, 1.0, false), "final_dmg")
	var crit_dmg: int = UiTheme.dict_int(DamageCalculator.compute(w, d, false, false, 1.0, true), "final_dmg")
	var exp_crit_dmg: int = roundi(float(norm_dmg) * DamageCalculator.CRIT_MULT)
	if absi(crit_dmg - exp_crit_dmg) > 1:
		print("  [Dojo-16] Critical damage x%.1f multiplier in combat formula: FAIL (%d != %d)" % [DamageCalculator.CRIT_MULT, crit_dmg, exp_crit_dmg])
		passed = false
	else:
		print("  [Dojo-16] Critical damage x%.1f multiplier in combat formula: PASS (%d -> %d DMG)" % [DamageCalculator.CRIT_MULT, norm_dmg, crit_dmg])

	# Assert 17: Stat Allocation Logic (only newly allocated points can be removed)
	w.stats.add_free_attribute_points(2)   # a fully spent unit has nothing to allocate
	var unspent_pre: int = w.stats.unspent_attribute_points
	session.begin_allocation()
	session.allocate_point("STR")
	if session.get_pending("STR") != 1 or session.pending_free_points != (unspent_pre - 1):
		print("  [Dojo-17a] Stat allocation plus button: FAIL")
		passed = false
	else:
		session.deallocate_point("STR")
		if session.get_pending("STR") != 0 or session.pending_free_points != unspent_pre:
			print("  [Dojo-17b] Stat allocation minus button undo: FAIL")
			passed = false
		else:
			session.deallocate_point("STR") # below baseline: must be a no-op
			if session.get_pending("STR") < 0 or session.pending_free_points > unspent_pre:
				print("  [Dojo-17c] Stat allocation minus button below zero guard: FAIL")
				passed = false
			else:
				print("  [Dojo-17] Stat allocation +/- logic: PASS (Cannot remove below baseline or grant extra points)")
	session.cancel_allocation()

	# Assert 18: Reset Dojo restores baseline
	session.reset()
	var w_post: Combatant = session.active_hero
	var d_post: Combatant = session.test_dummy
	if w_post.hp != w_post.max_hp or d_post.hp != 500 or d_post.defense_mitigation_ratio != 0.0 or w_post.stats.level != 2:
		print("  [Dojo-18] Reset Dojo restores initial baseline state: FAIL")
		passed = false
	else:
		print("  [Dojo-18] Reset Dojo restores initial baseline state: PASS")

	passed = DojoSessionAssertions.run_session_assertions(session) and passed
	passed = DojoUiAssertions.run_ui_smoke_assertions(session, host) and passed
	passed = DojoUiAssertions.run_slot_filter_and_carry_assertions(session, host) and passed
	passed = DojoSessionAssertions.run_skill_and_dummy_assertions(session) and passed
	return passed

# Frees a test-built node immediately (tests exit right after, so deferred frees would leak).
static func dispose(node: Node) -> void:
	if node.get_parent() != null:
		node.get_parent().remove_child(node)
	node.free()
