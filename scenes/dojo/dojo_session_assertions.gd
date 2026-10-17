class_name DojoSessionAssertions
extends RefCounted

# Dojo session checks (split from DojoAssertions by CLN-12): quick setup, gear integrity, logging and
# the benchmark; catalog skills, the dummy fighting back, and rounds. Called by DojoAssertions.run.

static func run_session_assertions(session: DojoSession) -> bool:
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

	# Assert 21d: strict hand typing (flag off): a 2H main refuses a shield, a focus item and a plain 1H weapon
	session.apply_loadout("claymore_tank")
	var saved_flag: bool = Equipment.allow_plain_dual_wield
	Equipment.allow_plain_dual_wield = false
	var strict_shield: bool = not session.equip_item(ItemCatalog.create_buckler(), Equipment.EquipSlot.OFF_HAND)
	var strict_focus: bool = not session.equip_item(ItemCatalog.create_mystic_tome(), Equipment.EquipSlot.OFF_HAND)
	var strict_plain: bool = not session.equip_item(ItemCatalog.create_steel_broadsword(), Equipment.EquipSlot.OFF_HAND)
	Equipment.allow_plain_dual_wield = saved_flag
	if not (strict_shield and strict_focus and strict_plain and inv.get_equipped(Equipment.EquipSlot.OFF_HAND) == null):
		print("  [Dojo-21d] Strict hands (flag off): 2H main refuses shield, focus and plain 1H in the off hand: FAIL (%s %s %s)" % [strict_shield, strict_focus, strict_plain])
		passed = false
	else:
		print("  [Dojo-21d] Strict hands (flag off): 2H main refuses shield, focus and plain 1H in the off hand: PASS")

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

	# Assert 21e: flag off, Quick Setup "dual_wield" skips the off-hand shortsword and reports it (the Dojo-20 mechanism)
	# when the shortsword is a main_1h weapon (after ITEM1-A); an off-category shortsword (before) still fits.
	var strict_saved: bool = Equipment.allow_plain_dual_wield
	Equipment.allow_plain_dual_wield = false
	session.apply_loadout("dual_wield")
	var setup_entry: DojoLogEntry = session.log_entries[session.log_entries.size() - 1]
	var blade: Equipment = ItemCatalog.create_iron_shortsword()
	var must_skip: bool = blade.hand() == Equipment.HAND_MAIN_1H
	var reported: bool = "skipped" in setup_entry.detail_text and blade.name in setup_entry.detail_text
	var held_off: Equipment = inv.get_equipped(Equipment.EquipSlot.OFF_HAND)
	Equipment.allow_plain_dual_wield = strict_saved
	if reported != must_skip or (held_off == null) != must_skip:
		print("  [Dojo-21e] Strict hands (flag off): Quick Setup skips a main-hand off-hand blade and reports it: FAIL (%s)" % setup_entry.detail_text)
		passed = false
	else:
		print("  [Dojo-21e] Strict hands (flag off): Quick Setup skips a main-hand off-hand blade and reports it: PASS (%s)" % ("skipped" if must_skip else "off-category blade fits"))
	session.apply_loadout("dual_wield")

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

static func _check(tag: String, label: String, ok: bool, detail: String = "") -> bool:
	print("  [%s] %s: %s%s" % [tag, label, "PASS" if ok else "FAIL", (" (%s)" % detail) if not detail.is_empty() else ""])
	return ok

static func run_skill_and_dummy_assertions(session: DojoSession) -> bool:
	session.reset()
	var hero: Combatant = session.active_hero
	var dummy: Combatant = session.test_dummy
	var passed: bool = true

	# 33: the console's skills are the catalog's (engine-only skills hidden)
	var ids: Array[String] = []
	for s: SkillDef in session.usable_skills():
		ids.append(s.id)
	var catalog_ok: bool = ids.has("strike") and ids.has("riposte") and ids.has("shield_bash") and ids.has("guard") and ids.has("focus") and ids.has("rest") and not ids.has("move") and not ids.has("stunned")
	passed = _check("Dojo-33", "Console skills come from the content catalog", catalog_ok, "%d skills" % ids.size()) and passed

	# 34: a level-2 attack (Hamstring, x0.8) = the battle rules: AP (level + stance switch), its stance, DamageCalculator hit
	var power_strike: SkillDef = SkillCatalog.find("hamstring")
	var ap_before: int = hero.ap
	var stance_before: CombatTypes.StanceType = hero.current_stance
	var expected_cost: int = session.skill_ap(power_strike)
	var probe := RandomNumberGenerator.new()
	probe.seed = session.rng.seed
	probe.state = session.rng.state
	var expected: int = UiTheme.dict_int(DamageCalculator.compute(hero, dummy, false, false, power_strike.power_mult, false, probe, power_strike.stance()), "final_dmg")
	var hp_before: int = dummy.hp
	session.use_skill(power_strike.id)
	var dealt: int = hp_before - dummy.hp
	var strike_ok: bool = (dealt == expected and hero.ap == ap_before - expected_cost and expected_cost == power_strike.level + power_strike.switch_cost(stance_before)
		and hero.current_stance == power_strike.stance_after(stance_before) and hero.current_stance == CombatTypes.StanceType.OFFENSIVE)
	passed = _check("Dojo-34", "A level-2 attack follows battle rules (level AP + stance switch, Offensive, skill power_mult hit)", strike_ok, "dealt %d expected %d, AP %d -> %d" % [dealt, expected, ap_before, hero.ap]) and passed

	# 35: Guard mitigates (the skill's mitigation, Defensive); the dummy's attack uses its power, stance and crit vs the hero's guard & armor
	session.use_skill("guard")
	var guard_ok: bool = is_equal_approx(hero.defense_mitigation_ratio, SkillCatalog.find("guard").mitigation) and hero.current_stance == CombatTypes.StanceType.DEFENSIVE
	var probe2 := RandomNumberGenerator.new()
	probe2.seed = session.rng.seed
	probe2.state = session.rng.state
	var log_before: int = session.log_entries.size()
	var hero_hp: int = hero.hp
	var predicted: Dictionary = DamageCalculator.compute(dummy, hero, false, false, 1.0, false, probe2, CombatTypes.StanceType.OFFENSIVE)
	session.dummy_attack()
	var hit_entry: DojoLogEntry = session.log_entries[session.log_entries.size() - 1]
	var dummy_ok: bool = (guard_ok and hero_hp - hero.hp == UiTheme.dict_int(predicted, "final_dmg") and session.log_entries.size() == log_before + 1
		and hit_entry.category == "ATTACK" and hit_entry.actor_name == dummy.name and not hit_entry.actor_is_ally and hit_entry.target_name == hero.name
		and DamageCalculator.power_of(dummy) == 20 and not hit_entry.breakdown_bbcode.is_empty())
	passed = _check("Dojo-35", "Dummy attacks the hero through the same formula (guard, armor, stance)", dummy_ok, "took %d, predicted %d" % [hero_hp - hero.hp, UiTheme.dict_int(predicted, "final_dmg")]) and passed

	# 36: counterattack: every hero skill is answered by the dummy
	session.set_dummy_counter(true)
	var before_counter: int = session.log_entries.size()
	var hp_counter: int = hero.hp
	session.use_skill("strike")
	var last: DojoLogEntry = session.log_entries[session.log_entries.size() - 1]
	var counter_ok: bool = session.log_entries.size() == before_counter + 2 and last.actor_name == dummy.name and hero.hp < hp_counter
	session.set_dummy_counter(false)
	passed = _check("Dojo-36", "Counterattack: the dummy answers each hero skill", counter_ok) and passed

	# 37: a stunned dummy loses its action; next round ticks Focus, drops the guard and regains AP
	dummy.statuses[StatusEffects.STUN] = true
	var hp_stun: int = hero.hp
	session.dummy_attack()
	var stunned_ok: bool = hero.hp == hp_stun and session.log_entries[session.log_entries.size() - 1].outcome_tag == StatusEffects.display_name(StatusEffects.STUN).to_upper()
	session.use_skill("focus")
	session.use_skill("guard")
	var ap_round: int = hero.ap
	session.next_round()
	var round_ok: bool = (stunned_ok and session.round_num == 2 and hero.defense_mitigation_ratio == 0.0 and hero.attack_boost_rounds_remaining == 1
		and hero.ap == mini(hero.max_ap, ap_round + DojoSession.ROUND_AP_REGEN) and not dummy.has_status(StatusEffects.STUN))
	passed = _check("Dojo-37", "Stunned dummy loses its action; Next Round ticks Focus, drops guard, +4 AP", round_ok, "AP %d -> %d" % [ap_round, hero.ap]) and passed

	# 38: Rest recovers AP; a down hero can't act until healed
	hero.ap = 3
	session.use_skill("rest")
	var rest_ok: bool = hero.ap == 3 + SkillCatalog.find("rest").recover_ap and hero.current_stance == SkillCatalog.find("rest").stance()
	hero.hp = 0
	var log_down: int = session.log_entries.size()
	session.use_skill("strike")
	var refused: bool = session.log_entries[session.log_entries.size() - 1].outcome_tag == "FAILED" and session.log_entries.size() == log_down + 1
	session.heal_hero()
	passed = _check("Dojo-38", "Rest recovers AP; a downed hero is refused until Heal Hero", rest_ok and refused and hero.hp == hero.max_hp) and passed
	session.reset()
	return passed
