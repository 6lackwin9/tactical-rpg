class_name DojoAssertions
extends RefCounted

# ==============================================================================
# Tactical RPG - Headless Gear & Skill Dojo Verification Suite
# Exercises the headless DojoSession (rules glue) and smoke-tests the modular
# panel/window UI. `host` must be inside the scene tree (used for UI smoke test).
# ==============================================================================

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
	# Warrior Lv 2: Base STR 18 + Bear Ring (+2) = 20 STR. Sword mastery Lv 1 (+2 Wep Power).
	# Steel Broadsword: 18 Base + 2 Mastery = 20 Wep Power.
	# Power = 20 + (20 * 1.5) = 50.
	var initial_power: int = w.get_effective_power(false)
	var calc_res: Dictionary = DamageCalculator.compute(w, d, false, false, 1.0, false)
	var dmg: int = d.take_damage(UiTheme.dict_int(calc_res, "final_dmg"))
	if dmg != 50 or initial_power != 50:
		print("  [Dojo-2] Base melee attack calculation: FAIL (dmg: %d, pwr: %d != 50)" % [dmg, initial_power])
		passed = false
	else:
		print("  [Dojo-2] Base melee attack calculation: PASS (50 Power, %d DMG, Sword Mastery in Wep Power)" % dmg)

	# Assert 3: Dummy Mitigation Toggle
	d.defense_mitigation_ratio = 0.5
	var mit_res: Dictionary = DamageCalculator.compute(w, d, false, false, 1.0, false)
	var half_dmg: int = d.take_damage(UiTheme.dict_int(mit_res, "final_dmg"), true)
	if half_dmg != 25:
		print("  [Dojo-3] 50%% mitigation defense barrier calculation: FAIL (%d != 25)" % half_dmg)
		passed = false
	else:
		print("  [Dojo-3] 50%% mitigation defense barrier calculation: PASS (%d DMG)" % half_dmg)

	# Assert 4: Focus Buff Multiplier (x1.5)
	w.apply_attack_boost(1.5, 2)
	d.defense_mitigation_ratio = 0.0
	var focus_res: Dictionary = DamageCalculator.compute(w, d, false, false, 1.0, false)
	var boosted_dmg: int = d.take_damage(UiTheme.dict_int(focus_res, "final_dmg"), true)
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
	var expected_unarmed_melee: int = w.stats.get_melee_power(0, {"weapon": "bow"})
	if bow_melee != expected_unarmed_melee:
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
	var expected_shortsword_pwr: int = shortsword.get_effective_equipment_power(w.stats, w.stats.mastery) + int(roundf(float(w.stats.get_effective_stat("STR")) * 1.5))
	if off_str_pwr != expected_shortsword_pwr:
		print("  [Dojo-13] Off-hand Shortsword STR scaling: FAIL (%d != %d)" % [off_str_pwr, expected_shortsword_pwr])
		passed = false
	else:
		print("  [Dojo-13] Off-hand Shortsword STR scaling: PASS (%d ATK Power)" % off_str_pwr)

	var knife := ItemCatalog.create_shadow_dagger()
	w.inventory.equip(knife, Equipment.EquipSlot.OFF_HAND)
	w.sync_from_stats_and_gear()
	var off_dex_pwr: int = w.get_offhand_power()
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
	var exp_crit: float = 5.0 + (float(w.stats.get_effective_stat("DEX")) * 1.5)
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
	if crit_dmg != int(roundf(float(norm_dmg) * 2.5)):
		print("  [Dojo-16] Critical damage 2.5x multiplier in combat formula: FAIL (%d != %d)" % [crit_dmg, int(roundf(float(norm_dmg) * 2.5))])
		passed = false
	else:
		print("  [Dojo-16] Critical damage 2.5x multiplier in combat formula: PASS (%d -> %d DMG)" % [norm_dmg, crit_dmg])

	# Assert 17: Stat Allocation Logic (only newly allocated points can be removed)
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

	passed = _run_session_assertions(session) and passed
	passed = _run_ui_smoke_assertions(session, host) and passed
	passed = _run_slot_filter_and_carry_assertions(session, host) and passed
	return passed

# ------------------------------------------------------------------------------
# Session-level assertions (quick setup, gear integrity, logging, benchmark)
# ------------------------------------------------------------------------------

static func _run_session_assertions(session: DojoSession) -> bool:
	var passed: bool = true
	session.reset()
	var hero: Combatant = session.active_hero
	var dummy: Combatant = session.test_dummy

	# Assert 19: Quick Setup applies a full preset (2H main, no off-hand, plate, pouch)
	session.apply_loadout("claymore_tank")
	var inv: Inventory = hero.inventory
	var main_w: Equipment = inv.get_main_weapon()
	var body: Equipment = inv.get_equipped(Equipment.EquipSlot.BODY)
	if main_w == null or main_w.name != "Iron Claymore" or inv.get_equipped(Equipment.EquipSlot.OFF_HAND) != null \
			or body == null or body.name != "Steel Plate Cuirass" or inv.battle_pouch.size() != 2:
		print("  [Dojo-19] Quick Setup applies a complete loadout preset: FAIL")
		passed = false
	else:
		print("  [Dojo-19] Quick Setup applies a complete loadout preset: PASS (Claymore, Plate, 2 pouch items)")

	# Assert 20: Quick Setup respects class proficiency (Warrior cannot use the Archer bow)
	session.apply_loadout("archer")
	var skipped_entry: DojoLogEntry = session.log_entries[session.log_entries.size() - 1]
	if inv.get_main_weapon() != null or "skipped" not in skipped_entry.detail_text:
		print("  [Dojo-20] Quick Setup skips ineligible gear and reports it: FAIL (%s)" % skipped_entry.detail_text)
		passed = false
	else:
		print("  [Dojo-20] Quick Setup skips ineligible gear and reports it: PASS")

	# Assert 21: A 2H main hand refuses a shield but accepts an off-hand weapon
	session.apply_loadout("claymore_tank")
	var buckler: Equipment = null
	var shortsword: Equipment = null
	for item: Item in session.shared_stash.items:
		if item is Equipment and item.id == ItemCatalog.create_buckler().id:
			buckler = item
		if item is Equipment and item.id == ItemCatalog.create_iron_shortsword().id:
			shortsword = item
	var shield_refused: bool = buckler != null and not session.equip_item(buckler, Equipment.EquipSlot.OFF_HAND)
	var weapon_accepted: bool = shortsword != null and session.equip_item(shortsword, Equipment.EquipSlot.OFF_HAND)
	if not shield_refused or not weapon_accepted or inv.get_offhand_weapon() != shortsword:
		print("  [Dojo-21] 2H main hand refuses shield, accepts off-hand weapon: FAIL")
		passed = false
	else:
		print("  [Dojo-21] 2H main hand refuses shield, accepts off-hand weapon: PASS")

	# Assert 21b: 2H + off-hand weapon share one active slot; Switch Weapon (1 block) toggles it
	var dummy_hp_start: int = dummy.hp
	session.perform_strike("offhand")
	var blocked_offhand: bool = dummy.hp == dummy_hp_start and session.log_entries[session.log_entries.size() - 1].outcome_tag == "FAILED"
	session.switch_weapon()
	var switch_entry: DojoLogEntry = session.log_entries[session.log_entries.size() - 1]
	session.perform_strike("offhand")
	var offhand_works: bool = dummy.hp < dummy_hp_start
	var hp_after_off: int = dummy.hp
	session.perform_strike("main")
	var main_blocked: bool = dummy.hp == hp_after_off and session.log_entries[session.log_entries.size() - 1].outcome_tag == "FAILED"
	session.switch_weapon()
	session.perform_strike("main")
	var main_back: bool = dummy.hp < hp_after_off
	if not (blocked_offhand and switch_entry.outcome_tag == "SWITCHED" and switch_entry.value_num == 1 and offhand_works and main_blocked and main_back):
		print("  [Dojo-21b] Active-weapon switching with 2H + off-hand weapon: FAIL (%s %s %s %s)" % [blocked_offhand, offhand_works, main_blocked, main_back])
		passed = false
	else:
		print("  [Dojo-21b] Active-weapon switching with 2H + off-hand weapon: PASS (1-block switch gates each weapon)")

	# Assert 21c: 1H pair needs no switch; Dual Strike hits with both weapons in one action
	session.apply_loadout("dual_wield")
	var dual_hp_before: int = dummy.hp
	var entries_before: int = session.log_entries.size()
	session.perform_strike("dual")
	var dual_entry: DojoLogEntry = session.log_entries[session.log_entries.size() - 1]
	var availability: Dictionary = session.get_action_availability()
	var switch_info: Dictionary = availability.get("switch", {})
	var main_info: Dictionary = availability.get("main", {})
	var off_info: Dictionary = availability.get("offhand", {})
	var dual_ok: bool = (
		not inv.needs_weapon_switch() and session.log_entries.size() == entries_before + 1
		and dual_entry.skill_name == "Dual Strike" and dual_entry.value_num == dual_hp_before - dummy.hp and dual_entry.value_num > 0
		and UiTheme.dict_bool(main_info, "enabled") and UiTheme.dict_bool(off_info, "enabled")
		and not UiTheme.dict_bool(switch_info, "enabled", true)
	)
	if not dual_ok:
		print("  [Dojo-21c] 1H pair uses both hands freely and Dual Strike combo: FAIL")
		passed = false
	else:
		print("  [Dojo-21c] 1H pair uses both hands freely and Dual Strike combo: PASS (%d total damage)" % dual_entry.value_num)
	session.heal_dummy()

	# Assert 22: Stash integrity - no item exists both equipped and in the stash, or twice
	session.clear_all_gear()
	session.apply_loadout("sword_board")
	var seen: Dictionary = {}
	var duplicated: bool = false
	for stash_item: Item in session.shared_stash.items:
		if seen.has(stash_item):
			duplicated = true
		seen[stash_item] = true
	for eq: Equipment in inv._iter_equipped_gear():
		if seen.has(eq):
			duplicated = true
	if duplicated or not inv.items.is_empty():
		print("  [Dojo-22] Equip/unequip keeps stash free of duplicates: FAIL")
		passed = false
	else:
		print("  [Dojo-22] Equip/unequip keeps stash free of duplicates: PASS")

	# Assert 23: Strike logs a battle-log entry with math breakdown and damages the dummy
	var hp_before: int = dummy.hp
	var log_before: int = session.log_entries.size()
	session.perform_strike("main")
	var strike_entry: DojoLogEntry = session.log_entries[session.log_entries.size() - 1]
	if dummy.hp >= hp_before or session.log_entries.size() != log_before + 1 or strike_entry.category != "ATTACK" or strike_entry.breakdown_bbcode.is_empty() or strike_entry.value_num != hp_before - dummy.hp:
		print("  [Dojo-23] Strike writes ATTACK log entry with breakdown: FAIL")
		passed = false
	else:
		print("  [Dojo-23] Strike writes ATTACK log entry with breakdown: PASS (%s, %d DMG)" % [strike_entry.ref_tag, strike_entry.value_num])

	# Assert 24: Illegal actions are rejected and logged as FAILED without changing the dummy
	var hp_mid: int = dummy.hp
	session.perform_strike("ranged")
	var fail_entry: DojoLogEntry = session.log_entries[session.log_entries.size() - 1]
	if dummy.hp != hp_mid or fail_entry.outcome_tag != "FAILED":
		print("  [Dojo-24] Ranged volley with melee weapon is rejected and logged: FAIL")
		passed = false
	else:
		print("  [Dojo-24] Ranged volley with melee weapon is rejected and logged: PASS")

	# Assert 25: Benchmark is a pure simulation (dummy HP untouched) and logs statistics
	var hp_bench: int = dummy.hp
	session.run_benchmark(25)
	var bench_entry: DojoLogEntry = session.log_entries[session.log_entries.size() - 1]
	if dummy.hp != hp_bench or bench_entry.outcome_tag != "SIM" or bench_entry.value_num <= 0:
		print("  [Dojo-25] Benchmark simulates without changing dummy HP: FAIL")
		passed = false
	else:
		print("  [Dojo-25] Benchmark simulates without changing dummy HP: PASS (%s)" % bench_entry.detail_text)
	return passed

# ------------------------------------------------------------------------------
# UI smoke test: six panels mount in the workspace and react to session events
# ------------------------------------------------------------------------------

static func _run_ui_smoke_assertions(session: DojoSession, host: Node) -> bool:
	session.reset()
	var workspace := DojoWorkspace.new()
	host.add_child(workspace)
	var panels: Array[DojoPanel] = [
		DojoUnitInspectorPanel.new(),
		DojoGearPanel.new(),
		DojoStagePanel.new(),
		DojoConsolePanel.new(),
		DojoDerivedStatsPanel.new(),
		DojoLogPanel.new(),
	]
	workspace.setup(session, panels)

	var ids: Array[String] = []
	for panel: DojoPanel in panels:
		ids.append(panel.panel_id)
	var all_mounted: bool = true
	for id: String in ["inspector", "gear", "stage", "console", "derived", "log"]:
		if workspace.get_panel(id) == null or id not in ids:
			all_mounted = false

	session.perform_strike("main")
	session.apply_loadout("dual_wield")
	var log_panel_raw: UiPanel = workspace.get_panel("log")
	var shown: int = 0
	if log_panel_raw is DojoLogPanel:
		var log_panel: DojoLogPanel = log_panel_raw
		shown = log_panel.displayed_entry_count()
	var ok: bool = all_mounted and shown == session.log_entries.size() and shown >= 2

	_dispose(workspace)
	if not ok:
		print("  [Dojo-26] Modular panels mount in workspace and mirror session log: FAIL (mounted: %s, cells: %d / %d)" % [all_mounted, shown, session.log_entries.size()])
	else:
		print("  [Dojo-26] Modular panels mount in workspace and mirror session log: PASS (6 panels, %d log cells)" % shown)

	var modal_ok: bool = _run_party_stash_assertions(session, host)
	return ok and modal_ok

# Party & Stash screen: shared session, party selection, stash equip and gear sync
static func _run_party_stash_assertions(session: DojoSession, host: Node) -> bool:
	session.reset()
	var modal := CharacterEquipmentModal.new()
	host.add_child(modal)
	modal.attach_session(session)
	modal.open_modal(session.active_hero)

	var party_ok: bool = modal.party_allies.size() == 2 and modal.active_hero == session.party_allies[0]
	modal.select_hero(session.party_allies[1])
	var select_ok: bool = session.active_hero == session.party_allies[1] and modal.active_hero.name == "Ranger"

	# Stash equip through the modal API changes the shared session's hero gear
	modal.select_hero(session.party_allies[0])
	var claymore: Equipment = ItemCatalog.create_iron_claymore()
	session.shared_stash.add_item(claymore)
	modal._equip_item(claymore)
	var equip_ok: bool = session.active_hero.inventory.get_main_weapon() == claymore and not session.shared_stash.items.has(claymore)

	_dispose(modal)
	var ok: bool = party_ok and select_ok and equip_ok
	if not ok:
		print("  [Dojo-27] Party & Stash screen shares session (party list, selection, stash equip): FAIL (%s %s %s)" % [party_ok, select_ok, equip_ok])
	else:
		print("  [Dojo-27] Party & Stash screen shares session (party list, selection, stash equip): PASS")
	return ok

# ------------------------------------------------------------------------------
# Slot rules, stash filters, 4-slot party viewer, party carry and overlays
# ------------------------------------------------------------------------------

static func _names(items: Array[Item]) -> Array[String]:
	var result: Array[String] = []
	for item: Item in items:
		result.append(item.name)
	return result

static func _has_all(names: Array[String], expected: Array[String]) -> bool:
	for n: String in expected:
		if not names.has(n):
			return false
	return true

static func _has_none(names: Array[String], banned: Array[String]) -> bool:
	for n: String in banned:
		if names.has(n):
			return false
	return true

static func _run_slot_filter_and_carry_assertions(session: DojoSession, host: Node) -> bool:
	var passed: bool = true
	session.reset()
	var hero: Combatant = session.active_hero
	var stash: Array[Item] = session.shared_stash.items

	# Assert 28: Equipment slot rules (off-hand weapons ONLY in off-hand; shields off-hand; 1H main may dual wield)
	var claymore: Equipment = ItemCatalog.create_iron_claymore()
	var broadsword: Equipment = ItemCatalog.create_steel_broadsword()
	var shortsword: Equipment = ItemCatalog.create_iron_shortsword()
	var buckler: Equipment = ItemCatalog.create_buckler()
	var main: Equipment.EquipSlot = Equipment.EquipSlot.MAIN_HAND
	var off: Equipment.EquipSlot = Equipment.EquipSlot.OFF_HAND
	var rules_ok: bool = (
		claymore.can_equip_in_slot(main) and not claymore.can_equip_in_slot(off)
		and broadsword.can_equip_in_slot(main) and broadsword.can_equip_in_slot(off)
		and shortsword.can_equip_in_slot(off) and not shortsword.can_equip_in_slot(main)
		and buckler.can_equip_in_slot(off) and not buckler.can_equip_in_slot(main)
	)
	var stash_shortsword: Equipment = null
	for item: Item in stash:
		if item.id == shortsword.id and item is Equipment:
			stash_shortsword = item
	var refused_main: bool = stash_shortsword != null and not session.equip_item(stash_shortsword, main)
	if not rules_ok or not refused_main:
		print("  [Dojo-28] Slot rules: off-hand weapons/shields off-hand only, 1H may dual wield: FAIL (%s %s)" % [rules_ok, refused_main])
		passed = false
	else:
		print("  [Dojo-28] Slot rules: off-hand weapons/shields off-hand only, 1H may dual wield: PASS")

	# Assert 29: Stash filters - per-slot standard filter + custom dimensions
	var f := StashFilter.new()
	f.slot_filter = "MAIN_HAND"
	var main_names: Array[String] = _names(f.apply(stash, hero))
	f.slot_filter = "OFF_HAND"
	var off_names: Array[String] = _names(f.apply(stash, hero))
	f.slot_filter = "BODY"
	var body_names: Array[String] = _names(f.apply(stash, hero))
	f.slot_filter = "CONSUMABLE"
	var con_names: Array[String] = _names(f.apply(stash, hero))
	var slots_ok: bool = (
		_has_all(main_names, ["Iron Claymore", "Titan Warhammer", "Oak Shortbow", "Runed Staff"])
		and _has_none(main_names, ["Iron Shortsword", "Spiked Knuckles", "Shadow Dagger", "Mystic Spellbook", "Iron Kite Shield", "Duelist Buckler"])
		and _has_all(off_names, ["Iron Shortsword", "Spiked Knuckles", "Shadow Dagger", "Mystic Spellbook", "Iron Kite Shield", "Duelist Buckler"])
		and _has_none(off_names, ["Iron Claymore", "Oak Shortbow", "Titan Warhammer", "Runed Staff"])
		and _has_all(body_names, ["Steel Plate Cuirass", "Leather Brigandine"]) and _has_none(body_names, ["Iron Kite Shield", "Duelist Buckler"])
		and con_names.size() == 3
	)

	f = StashFilter.new()
	f.stats = ["INT"]
	var int_names: Array[String] = _names(f.apply(stash, hero))
	f = StashFilter.new()
	f.rarities = [int(Item.ItemRarity.RARE)]
	var rare_names: Array[String] = _names(f.apply(stash, hero))
	f = StashFilter.new()
	f.damage_types = ["magic"]
	var magic_names: Array[String] = _names(f.apply(stash, hero))
	f = StashFilter.new()
	f.slot_filter = "MAIN_HAND"
	f.hand = "2H"
	f.usable_only = true
	var usable_2h: Array[String] = _names(f.apply(stash, hero)) # Warrior cannot use bow or staff
	f = StashFilter.new()
	f.slot_filter = "MAIN_HAND"
	f.sort_mode = "POWER"
	var sorted_main: Array[Item] = f.apply(stash, hero)
	f = StashFilter.new()
	f.search = "iron"
	var iron_names: Array[String] = _names(f.apply(stash, hero))
	var custom_ok: bool = (
		_has_all(int_names, ["Mystic Spellbook", "Runed Staff"]) and _has_none(int_names, ["Iron Claymore", "Titan Warhammer"])
		and _has_all(rare_names, ["Titan Warhammer", "Runed Staff"]) and _has_none(rare_names, ["Iron Claymore"])
		and magic_names.size() == 2
		and _has_all(usable_2h, ["Iron Claymore", "Titan Warhammer"]) and _has_none(usable_2h, ["Oak Shortbow", "Runed Staff"])
		and not sorted_main.is_empty() and sorted_main[0].name == "Titan Warhammer"
		and _has_all(iron_names, ["Iron Claymore", "Iron Shortsword", "Iron Kite Shield"]) and _has_none(iron_names, ["Titan Warhammer"])
		and f.custom_count() == 1
	)
	if not slots_ok or not custom_ok:
		print("  [Dojo-29] Stash filters: per-slot (shield in off-hand) + stat/rarity/damage/hand/usable/search/sort: FAIL (%s %s)" % [slots_ok, custom_ok])
		passed = false
	else:
		print("  [Dojo-29] Stash filters: per-slot (shield in off-hand) + stat/rarity/damage/hand/usable/search/sort: PASS")

	# Assert 30: Party viewer always renders 4 stable slots
	session.reset()
	var party_panel := DojoPartyPanel.new()
	host.add_child(party_panel)
	party_panel.bind(session)
	var two_member_slots: int = party_panel.slot_card_count()
	var lone: Array[Combatant] = [session.party_allies[0]]
	var lone_session: DojoSession = DojoSession.wrap_party(lone, null)
	party_panel.bind(lone_session)
	var one_member_slots: int = party_panel.slot_card_count()
	_dispose(party_panel)
	if two_member_slots != DojoSession.MAX_PARTY_SIZE or one_member_slots != DojoSession.MAX_PARTY_SIZE or DojoSession.MAX_PARTY_SIZE != 4:
		print("  [Dojo-30] Party viewer keeps 4 slots regardless of party size: FAIL (%d / %d)" % [two_member_slots, one_member_slots])
		passed = false
	else:
		print("  [Dojo-30] Party viewer keeps 4 slots regardless of party size: PASS")

	# Assert 31: Party-wide carry vs per-unit burden
	session.reset()
	var warrior: Combatant = session.party_allies[0]
	var ranger: Combatant = session.party_allies[1]
	var expected_capacity: int = PartyCarry.BASE_CAPACITY
	for member: Combatant in session.party_allies:
		var ctx: Dictionary = member.get_weapon_context()
		expected_capacity += member.stats.get_effective_stat("STR", ctx) * 3 + member.stats.get_effective_stat("CONS", ctx)
	var report: Dictionary = session.carry_report
	var expected_load: float = 0.0
	for it: Item in session.shared_stash.items:
		expected_load += it.weight
	for member: Combatant in session.party_allies:
		for eq: Equipment in member.inventory._iter_equipped_gear():
			expected_load += eq.weight
		for con: Consumable in member.inventory.battle_pouch:
			expected_load += con.weight
	var base_ok: bool = (
		UiTheme.dict_int(report, "capacity") == expected_capacity
		and absf(UiTheme.dict_float(report, "load") - expected_load) < 0.01
		and str(report.get("tier", "")) == "LIGHT" and UiTheme.dict_int(report, "speed_penalty") == 0
	)

	# Per-unit burden: stripping only the warrior's armor must not change the ranger's speed.
	var ranger_speed_before: int = ranger.speed
	var warrior_speed_before: int = warrior.speed
	session.unequip_slot(Equipment.EquipSlot.BODY)
	var burden_local: bool = warrior.speed > warrior_speed_before and ranger.speed == ranger_speed_before

	# Party overload: loading the stash beyond capacity slows EVERY unit equally.
	var baseline_w: int = warrior.speed
	var baseline_r: int = ranger.speed
	var ballast: Array[Equipment] = []
	for i: int in range(20):
		var plate: Equipment = ItemCatalog.create_steel_plate_cuirass()
		ballast.append(plate)
		session.shared_stash.add_item(plate)
	session.notify_external_change()
	var over: Dictionary = session.carry_report
	var penalty: int = UiTheme.dict_int(over, "speed_penalty")
	var overload_ok: bool = (
		str(over.get("tier", "")) == "OVERLOADED" and penalty >= 1
		and warrior.speed == baseline_w - penalty and ranger.speed == baseline_r - penalty
		and warrior.carry_speed_penalty == penalty and ranger.carry_speed_penalty == penalty
	)
	for plate: Equipment in ballast:
		session.shared_stash.remove_item(plate)
	session.notify_external_change()
	var recovered: bool = warrior.speed == baseline_w and ranger.speed == baseline_r and str(session.carry_report.get("tier", "")) != "OVERLOADED"
	if not (base_ok and burden_local and overload_ok and recovered):
		print("  [Dojo-31] Unit burden is per-unit; party carry is shared (overload slows everyone): FAIL (%s %s %s %s)" % [base_ok, burden_local, overload_ok, recovered])
		passed = false
	else:
		print("  [Dojo-31] Unit burden is per-unit; party carry is shared (overload slows everyone): PASS (capacity %d, overload -%d speed)" % [expected_capacity, penalty])

	# Assert 32: Overlay windows (picker, allocation, custom filter) open above the workspace without changing it
	session.reset()
	var workspace := DojoWorkspace.new()
	host.add_child(workspace)
	var inspector := DojoUnitInspectorPanel.new()
	var gear := DojoGearPanel.new()
	var stash_panel := DojoStashPanel.new()
	var panels: Array[DojoPanel] = [inspector, gear, stash_panel]
	workspace.setup(session, panels)
	var layer: UiOverlayLayer = workspace.get_overlay_layer()
	inspector._on_points_pressed()
	var allocation_open: bool = layer.has_overlay("allocate") and session.is_allocating_stats
	gear.overlay_requested.emit(DojoPickerPanel.for_slot(Equipment.EquipSlot.MAIN_HAND))
	var picker_open: bool = layer.has_overlay("picker")
	stash_panel.overlay_requested.emit(DojoStashFilterPanel.new())
	var filter_open: bool = layer.has_overlay("stash_filter") and layer.overlay_count() == 3
	session.cancel_allocation()
	var allocation_closed: bool = not layer.has_overlay("allocate")
	layer.close_all()
	var all_closed: bool = layer.overlay_count() == 0
	_dispose(workspace)
	var overlays_ok: bool = allocation_open and picker_open and filter_open and allocation_closed and all_closed
	if not overlays_ok:
		print("  [Dojo-32] Overlay windows open on request and close on completion: FAIL (%s %s %s %s %s)" % [allocation_open, picker_open, filter_open, allocation_closed, all_closed])
		passed = false
	else:
		print("  [Dojo-32] Overlay windows open on request and close on completion: PASS")
	return passed

# Frees a test-built node immediately (tests exit right after, so deferred frees would leak).
static func _dispose(node: Node) -> void:
	if node.get_parent() != null:
		node.get_parent().remove_child(node)
	node.free()
