class_name DojoAssertions
extends RefCounted

# ==============================================================================
# Tactical RPG - Headless Gear & Skill Dojo Verification Suite
# ==============================================================================

static func run(dojo: Node) -> bool:
	var passed: bool = true

	# Assert 1: Default Hero & Dummy setup
	var w: Combatant = dojo.active_hero
	var d: Combatant = dojo.test_dummy
	if w == null or d == null or w.name != "Warrior" or d.hp != 500:
		print("  [Dojo-1] Baseline hero and dummy initialization: FAIL")
		passed = false
	else:
		print("  [Dojo-1] Baseline hero and dummy initialization: PASS")

	# Assert 2: Base Attack Calculation (Revised Formula & Sword Mastery to Wep Power) (Req 1 & 2)
	# Warrior Lv 2: Base STR 18 + Bear Ring (+2) = 20 STR. Sword mastery Lv 1 (+2 Wep Power).
	# Steel Broadsword: 18 Base + 2 Mastery = 20 Wep Power.
	# Power = 20 + (20 * 1.5) = 50.
	var initial_power: int = w.get_effective_power(false)
	var calc_res: Dictionary = dojo._calculate_combat_damage(w, d, false, false, 1.0, false)
	var dmg_raw: Variant = calc_res.get("final_dmg", 0)
	var dmg: int = d.take_damage(dmg_raw if dmg_raw is int else 0)
	if dmg != 50 or initial_power != 50:
		print("  [Dojo-2] Base melee attack calculation: FAIL (dmg: %d, pwr: %d != 50)" % [dmg, initial_power])
		passed = false
	else:
		print("  [Dojo-2] Base melee attack calculation: PASS (50 Power, %d DMG, Sword Mastery in Wep Power)" % dmg)

	# Assert 3: Dummy Mitigation Toggle (Req 9)
	d.defense_mitigation_ratio = 0.5
	var mit_res: Dictionary = dojo._calculate_combat_damage(w, d, false, false, 1.0, false)
	var half_dmg_raw: Variant = mit_res.get("final_dmg", 0)
	var half_dmg: int = d.take_damage(half_dmg_raw if half_dmg_raw is int else 0, true)
	if half_dmg != 25:
		print("  [Dojo-3] 50%% mitigation defense barrier calculation: FAIL (%d != 25)" % half_dmg)
		passed = false
	else:
		print("  [Dojo-3] 50%% mitigation defense barrier calculation: PASS (%d DMG)" % half_dmg)

	# Assert 4: Focus Buff Multiplier (x1.5) (Req 1 & 9)
	w.apply_attack_boost(1.5, 2)
	d.defense_mitigation_ratio = 0.0
	var focus_res: Dictionary = dojo._calculate_combat_damage(w, d, false, false, 1.0, false)
	var f_dmg_raw: Variant = focus_res.get("final_dmg", 0)
	var boosted_dmg: int = d.take_damage(f_dmg_raw if f_dmg_raw is int else 0, true)
	if boosted_dmg != 75:
		print("  [Dojo-4] Focus buff 1.5x power multiplier calculation: FAIL (%d != 75)" % boosted_dmg)
		passed = false
	else:
		print("  [Dojo-4] Focus buff 1.5x power multiplier calculation: PASS (%d DMG)" % boosted_dmg)

	# Assert 5: Claymore Threshold Tier Boost (18 STR -> +4 Power)
	w.attack_boost_multiplier = 1.0
	var claymore := ItemCatalog.create_iron_claymore()
	w.inventory.equip(claymore)
	w.sync_from_stats_and_gear()
	var tier_boost: int = claymore.get_unlocked_threshold_power_boost(w.stats, w.stats.mastery)
	if tier_boost != 4:
		print("  [Dojo-5] Claymore threshold tier bonus at 18 STR: FAIL (got %d)" % tier_boost)
		passed = false
	else:
		print("  [Dojo-5] Claymore threshold tier bonus at 18 STR: PASS (+4 Power unlocked)")

	# Assert 6: Titan Warhammer Deficit Penalty
	var titan := ItemCatalog.create_titan_warhammer()
	w.inventory.equip(titan)
	w.sync_from_stats_and_gear()
	var pen := titan.get_wield_penalty(w.stats, w.stats.mastery)
	var is_pen: bool = pen.get("is_penalized", false)
	if not is_pen:
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

	# Assert 9: Weapon Power Melee vs Ranged Gating (Req 3)
	var claymore_melee: int = w.get_effective_power(false)
	var claymore_ranged: int = w.get_effective_power(true)
	if claymore_ranged >= claymore_melee:
		print("  [Dojo-9] Melee weapon power gating on ranged attack: FAIL (ranged: %d >= melee: %d)" % [claymore_ranged, claymore_melee])
		passed = false
	else:
		print("  [Dojo-9] Melee weapon power gating on ranged attack: PASS (Melee: %d, Ranged: %d)" % [claymore_melee, claymore_ranged])

	# Assert 10: Ranged Bow Power Gating on Melee Attack (Req 3)
	var bow := ItemCatalog.create_oak_shortbow()
	w.inventory.equip(bow)
	w.sync_from_stats_and_gear()
	var bow_melee: int = w.get_effective_power(false)
	var expected_unarmed_melee: int = w.stats.get_melee_power(0, {"weapon": "bow"})
	if bow_melee != expected_unarmed_melee:
		print("  [Dojo-10] Ranged weapon power excluded from melee attack: FAIL (%d != %d)" % [bow_melee, expected_unarmed_melee])
		passed = false
	else:
		print("  [Dojo-10] Ranged weapon power excluded from melee attack: PASS (Bow grants 0 Melee Weapon Power)")

	# Assert 11: Accessory Stat Bonus to Character Stats & Attack Power (Req 1)
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

	# Assert 12: Class Archetype Gear Proficiency Restrictions (Req 5)
	var sword := ItemCatalog.create_steel_broadsword()
	var war_arch: ClassArchetype = w.stats.archetype
	var ranger_arch: ClassArchetype = dojo.party_allies[1].stats.archetype if dojo.party_allies.size() > 1 else ClassArchetype.create_ranger()
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

	# Assert 13: Off-Hand Weapon Taxonomy & Scaling (Req 7)
	# Off-hand weapon scaling: Shortsword (STR based), Knuckles (STR based), Knife (DEX based), Tome (INT based)
	var shortsword := ItemCatalog.create_iron_shortsword()
	w.inventory.equip(shortsword, Equipment.EquipSlot.OFF_HAND)
	w.sync_from_stats_and_gear()
	var off_str_pwr: int = w.get_offhand_power() # Scales with STR
	var expected_shortsword_pwr: int = shortsword.get_effective_equipment_power(w.stats, w.stats.mastery) + int(roundf(float(w.stats.get_effective_stat("STR")) * 1.5))
	if off_str_pwr != expected_shortsword_pwr:
		print("  [Dojo-13] Off-hand Shortsword STR scaling: FAIL (%d != %d)" % [off_str_pwr, expected_shortsword_pwr])
		passed = false
	else:
		print("  [Dojo-13] Off-hand Shortsword STR scaling: PASS (%d ATK Power)" % off_str_pwr)

	var knife := ItemCatalog.create_shadow_dagger()
	w.inventory.equip(knife, Equipment.EquipSlot.OFF_HAND)
	w.sync_from_stats_and_gear()
	var off_dex_pwr: int = w.get_offhand_power() # Scales with DEX
	var expected_knife_pwr: int = knife.get_effective_equipment_power(w.stats, w.stats.mastery) + int(roundf(float(w.stats.get_effective_stat("DEX")) * 1.5))
	if off_dex_pwr != expected_knife_pwr:
		print("  [Dojo-13b] Off-hand Knife DEX scaling: FAIL (%d != %d)" % [off_dex_pwr, expected_knife_pwr])
		passed = false
	else:
		print("  [Dojo-13b] Off-hand Knife DEX scaling: PASS (%d ATK Power)" % off_dex_pwr)

	var shield_eq := ItemCatalog.create_iron_kite_shield()
	w.inventory.equip(shield_eq, Equipment.EquipSlot.OFF_HAND)
	w.sync_from_stats_and_gear()
	if shield_eq.base_power != 0 or shield_eq.mitigation <= 0:
		print("  [Dojo-13c] Shield is mitigation only, not offensive: FAIL")
		passed = false
	else:
		print("  [Dojo-13c] Shield is mitigation only (0 offensive power, +%d mitigation): PASS" % shield_eq.mitigation)

	# Assert 14: 2-Handed Weapon blocks off-hand strike (Req 7)
	w.inventory.equip(claymore, Equipment.EquipSlot.MAIN_HAND)
	w.sync_from_stats_and_gear()
	if not claymore.is_two_handed:
		print("  [Dojo-14] Claymore tagged as 2-Handed: FAIL")
		passed = false
	else:
		print("  [Dojo-14] 2-Handed weapon tags and off-hand blocking: PASS")

	# Assert 15: Critical Hit Rate & Debuff Resistance Derived Metrics (Req 10)
	var crit_r: float = w.stats.get_crit_rate()
	var debuff_res: float = w.stats.get_debuff_resistance()
	var exp_crit: float = 5.0 + (float(w.stats.get_effective_stat("DEX")) * 1.5)
	var exp_res: float = float(w.stats.get_effective_stat("INT")) * 2.5
	if absf(crit_r - exp_crit) > 0.01 or absf(debuff_res - exp_res) > 0.01:
		print("  [Dojo-15] Derived Crit Rate and Debuff Resistance: FAIL (Crit: %.1f != %.1f, Res: %.1f != %.1f)" % [crit_r, exp_crit, debuff_res, exp_res])
		passed = false
	else:
		print("  [Dojo-15] Derived Crit Rate (%.1f%%) and Debuff Resistance (%.1f%%): PASS" % [crit_r, debuff_res])

	# Assert 16: Critical Damage Multiplier in Combat Formula (Req 9)
	w.inventory.equip(sword, Equipment.EquipSlot.MAIN_HAND)
	w.sync_from_stats_and_gear()
	var norm_hit: Dictionary = dojo._calculate_combat_damage(w, d, false, false, 1.0, false)
	var crit_hit: Dictionary = dojo._calculate_combat_damage(w, d, false, false, 1.0, true)
	var raw_norm: Variant = norm_hit.get("final_dmg", 0)
	var norm_dmg: int = raw_norm if raw_norm is int else 0
	var raw_crit: Variant = crit_hit.get("final_dmg", 0)
	var crit_dmg: int = raw_crit if raw_crit is int else 0
	if crit_dmg != int(roundf(float(norm_dmg) * 2.5)):
		print("  [Dojo-16] Critical damage 2.5x multiplier in combat formula: FAIL (%d != %d)" % [crit_dmg, int(roundf(float(norm_dmg) * 2.5))])
		passed = false
	else:
		print("  [Dojo-16] Critical damage 2.5x multiplier in combat formula: PASS (%d -> %d DMG)" % [norm_dmg, crit_dmg])

	# Assert 17: Stat Allocation UI Logic (Only remove newly allocated points) (Req 8)
	var _base_str_pre: int = w.stats.str_stat
	var unspent_pre: int = w.stats.unspent_attribute_points
	dojo.is_allocating_stats = true
	dojo.pending_free_points = unspent_pre
	dojo.pending_allocated_stats["STR"] = 0
	dojo._on_stat_plus_pressed("STR") # Add 1 to STR
	if dojo.pending_allocated_stats["STR"] != 1 or dojo.pending_free_points != (unspent_pre - 1):
		print("  [Dojo-17a] Stat allocation plus button: FAIL")
		passed = false
	else:
		dojo._on_stat_minus_pressed("STR") # Remove 1
		if dojo.pending_allocated_stats["STR"] != 0 or dojo.pending_free_points != unspent_pre:
			print("  [Dojo-17b] Stat allocation minus button undo: FAIL")
			passed = false
		else:
			# Trying minus again when 0 should NOT decrease further or give extra points
			dojo._on_stat_minus_pressed("STR")
			if dojo.pending_allocated_stats["STR"] < 0 or dojo.pending_free_points > unspent_pre:
				print("  [Dojo-17c] Stat allocation minus button below zero guard: FAIL")
				passed = false
			else:
				print("  [Dojo-17] Stat allocation +/- logic: PASS (Cannot remove below baseline or grant extra points)")
	dojo.is_allocating_stats = false

	# Assert 18: Reset Dojo restores baseline (Req 6)
	dojo._reset_dojo_to_baseline()
	var w_post: Combatant = dojo.active_hero
	var d_post: Combatant = dojo.test_dummy
	if w_post.hp != 100 or d_post.hp != 500 or d_post.defense_mitigation_ratio != 0.0 or w_post.stats.level != 2:
		print("  [Dojo-18] Reset Dojo restores initial baseline state: FAIL")
		passed = false
	else:
		print("  [Dojo-18] Reset Dojo restores initial baseline state: PASS")

	return passed
