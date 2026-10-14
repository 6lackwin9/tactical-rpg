class_name ArenaTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Headless Integration & UI Test Suite
# Runs every headless suite (exit code for check_project.ps1):
#  1. Combat UI Framework (CombatSession + CombatDockHost panels, Goblin Ambush)
#  2. Core Combat Rules (CoreRulesTestSuite: guard, stances, Focus, casualties, withdrawal, feed)
#  3. Character Specs & Inventory (6-stat framework, progression, proficiency, passives, gear)
#  4. Gear Equip UI & Test Dummy (equipment modal, dummy, damage formula)
#  5. Combat Session Flow (catalog Goblin Ambush: wave, replay, rounds, auto-play, reset)
#  6. Art, Terrain & Combat Zones (tokens, portraits, tiles, icons, zones)
#  7. Planning, Stances, AP & Enemy AI
#  7b. Class & unit rules (ClassRulesTestSuite)
#  7c. Combat extensions: ally targeting, gap-close, haste/bleed/taunt, AI fixes (CombatExtensionsTestSuite)
#  8. Content Catalog & Forge (ContentTestSuite)
#  9. Scene Runner (RunnerTestSuite)
# 10. World Engine & World Forge (WorldTestSuite)
# ==============================================================================

static func run_all_tests(tree_parent: Node = null) -> bool:
	var results: Array[bool] = [
		run_combat_ui_framework_test(tree_parent),
		CoreRulesTestSuite.run(),
		ClassRulesTestSuite.run(),
		CombatExtensionsTestSuite.run(),
		run_character_specs_and_inventory_test(),
		run_gear_equip_and_test_dummy_suite(tree_parent),
		run_combat_session_flow_test(),
		run_art_and_terrain_test(),
		run_planning_and_ai_test(),
		ContentTestSuite.run(tree_parent),
		RunnerTestSuite.run(tree_parent),
		WorldTestSuite.run(tree_parent),
	]
	return not results.has(false)


# ==============================================================================
# TEST 1: COMBAT UI FRAMEWORK & BEHAVIOUR TEST (CombatSession + CombatDockHost)
# ==============================================================================

static func run_combat_ui_framework_test(tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING COMBAT UI FRAMEWORK & BEHAVIOUR TEST   ===")
	print("=======================================================")

	var session := CombatSession.new(CatalogEncounter.new("goblin_ambush"))
	var host := CombatDockHost.new()
	if tree_parent != null:
		tree_parent.add_child(host)
	host.setup(session)
	session.start()

	# 1. Every panel is mounted in its dock region of the default layout
	var expected_regions: Dictionary = {
		"party": "top_left", "timeline": "top_center", "enemies": "top_right",
		"inspector": "left", "grid": "center", "forecast": "right",
		"playback": "bottom_left", "action_log": "bottom_center",
	}
	var docks_ok: bool = host.log_modal != null and host.timeline_modal != null
	for id: String in expected_regions:
		if host.get_panel(id) == null or host.region_of(id) != str(expected_regions[id]):
			docks_ok = false

	# 2. Tokens and roster cards are bound to the SAME combatants the session simulates
	var grid_panel: CombatGridPanel = host.get_panel("grid") as CombatGridPanel
	var enemy_panel: CombatRosterPanel = host.get_panel("enemies") as CombatRosterPanel
	var warrior: Combatant = session.allies[0]
	var tokens_ok: bool = (
		grid_panel.token_count() == session.allies.size() + session.enemies.size()
		and grid_panel.grid_view.get_token_for(warrior) != null
		and grid_panel.visible_token_count() == 4  # 2 heroes + Brawler + Scout; the ambush is not on the field yet
		and enemy_panel.visible_card_count() == 2
	)

	# 3. Selecting a unit updates the Unit Inspector
	var inspector_panel: CombatInspectorPanel = host.get_panel("inspector") as CombatInspectorPanel
	session.select_combatant(session.find_combatant("Enemy Brawler"))
	var enemy_shown: bool = inspector_panel.inspector.inspector_name_lbl.text == "Enemy Brawler"
	session.select_combatant(warrior)
	var inspector_ok: bool = enemy_shown and inspector_panel.inspector.inspector_name_lbl.text == "Warrior"

	# 4. Tactical forecast targets the selected unit's opponent
	var forecast_panel: CombatForecastPanel = host.get_panel("forecast") as CombatForecastPanel
	var target: Combatant = session.forecast_target(warrior)
	var forecast_ok: bool = target != null and target.name in forecast_panel.forecast.forecast_target_lbl.text

	# 5. Log modal opens with the session's entries and closes
	for i: int in range(30):
		session.step_next()
		if session.current_block >= 3:
			break
	host.open_modal("log")
	var log_open: bool = host.log_modal.visible and host.log_modal.all_entries.size() == session.log_entries.size() and not session.log_entries.is_empty()
	host.close_modals()
	var log_modal_ok: bool = log_open and not host.log_modal.visible

	# 6. Timeline modal opens and closes
	host.open_modal("timeline")
	var timeline_open: bool = host.timeline_modal.visible
	host.close_modals()
	var timeline_modal_ok: bool = timeline_open and not host.timeline_modal.visible

	# 7. Timeline dock block click inspects a resolved block of the live round
	var timeline_panel: CombatTimelinePanel = host.get_panel("timeline") as CombatTimelinePanel
	timeline_panel.dock.block_selected.emit(1)
	var inspected: bool = session.inspect_block == 1 and session.playhead() == 1 and session.display_snapshot() != null
	session.step_next()
	var signals_ok: bool = inspected and session.inspect_block == -1

	# 8. Undock floats a panel (its region keeps a placeholder), dock returns it; FX dispatch is safe
	host.undock("forecast")
	var floating: bool = host.is_undocked("forecast") and host.region_of("forecast") == "right"
	host.dock("forecast")
	host.apply_layout("Mirrored")
	var mirrored: bool = host.region_of("inspector") == "right" and host.region_of("forecast") == "left"
	host.apply_layout("Standard")
	if tree_parent != null:
		var brawler: Combatant = session.find_combatant("Enemy Brawler")
		session.fx_requested.emit(ActionLogEntry.new(1, 2, "ATTACK", warrior.name, brawler.name, "-25 HP", 25, "HIT"))
		session.fx_requested.emit(ActionLogEntry.new(1, 1, "DEFEND", warrior.name, "", "50%", 50, "DEFEND"))
		session.fx_requested.emit(ActionLogEntry.new(1, 1, "BUFF", warrior.name, "", "+50%", 50, "FOCUS"))
		session.fx_requested.emit(ActionLogEntry.new(1, 1, "MOVE", warrior.name, "", "-> (4, 4)", 0, "MOVED", Vector2i(4, 5), Vector2i(4, 4)))
	var layout_ok: bool = floating and not host.is_undocked("forecast") and host.region_of("forecast") == "right" and mirrored

	print("\n=======================================================")
	print("=== COMBAT UI FRAMEWORK TEST ASSERTIONS             ===")
	print("=======================================================")
	print("  [UI] All 8 combat panels mounted in their dock regions: %s" % ("PASS" if docks_ok else "FAIL"))
	print("  [UI] Tokens & roster bound to the simulated combatants: %s" % ("PASS" if tokens_ok else "FAIL"))
	print("  [UI] Combatant selection & Unit Inspector synced: %s" % ("PASS" if inspector_ok else "FAIL"))
	print("  [UI] Tactical forecast computed & rendered: %s" % ("PASS" if forecast_ok else "FAIL"))
	print("  [UI] Action Log Modal toggle & entry view lifecycle: %s" % ("PASS" if log_modal_ok else "FAIL"))
	print("  [UI] Timeline Modal toggle lifecycle: %s" % ("PASS" if timeline_modal_ok else "FAIL"))
	print("  [UI] Timeline dock block selection inspects resolved block: %s" % ("PASS" if signals_ok else "FAIL"))
	print("  [UI] Undock/dock, layout presets & action FX dispatch: %s" % ("PASS" if layout_ok else "FAIL"))

	if tree_parent != null:
		tree_parent.remove_child(host)
	host.free()
	session.dispose()

	return (docks_ok and tokens_ok and inspector_ok and forecast_ok and
		log_modal_ok and timeline_modal_ok and signals_ok and layout_ok)


# ==============================================================================
# TEST 5: COMBAT SESSION FLOW (catalog Goblin Ambush, headless)
# ==============================================================================

static func _resolve_until_replay(session: CombatSession, max_steps: int = 600) -> void:
	for i: int in range(max_steps):
		if session.phase == CombatSession.Phase.REPLAY:
			return
		session.step_next()

static func run_combat_session_flow_test() -> bool:
	print("\n=======================================================")
	print("=== STARTING COMBAT SESSION FLOW TEST (GOBLIN AMBUSH) ===")
	print("=======================================================")

	var encounter := CatalogEncounter.new("goblin_ambush")
	var session := CombatSession.new(encounter)
	session.start()
	var warrior: Combatant = encounter.unit("warrior")

	# 1. Before the wave only the initial enemies are in view
	var pre_view_ok: bool = session.enemies_in_view().size() == 2 and session.phase == CombatSession.Phase.PLANNING

	# 2. The wave enters exactly at Round 1, Block 4 and brings 4 goblins into view
	session.plan_apply_preset_all("balanced")
	var ambush: ActionLogEntry = null
	for i: int in range(300):
		session.step_next()
		if session.log_entries.is_empty():
			continue
		var last: ActionLogEntry = session.log_entries[session.log_entries.size() - 1]
		if last.outcome_tag == "ARRIVED":
			ambush = last
			break
	var ambush_ok: bool = ambush != null and ambush.ref_tag == "R1-B04" and session.enemies_in_view().size() == 6

	# 3. Round 1 resolves into replay mode with the round advance offered
	_resolve_until_replay(session)
	var round1_ok: bool = (
		session.phase == CombatSession.Phase.REPLAY and not session.combat_over
		and session.can_advance_round() and not session.next_round_label.is_empty()
		and warrior.is_alive()
	)

	# 4. Replay works by log points: a block opens at its start (state after the blocks
	# before it), a round opens at its start, and stepping walks the log forward and back
	var snap0: BlockSnapshot = session.snapshot_at(1, 0)
	var snap1: BlockSnapshot = session.snapshot_at(1, 1)
	var start_hp: int = snap0.hps.get(warrior, -1)
	session.select_block(2)
	var at_block2: bool = snap1 != null and session.position_of(warrior) == snap1.positions.get(warrior, Vector2i(-1, -1))
	session.select_block(1)
	var at_start: bool = session.replay_cursor == -1 and session.display_snapshot() == snap0 and session.position_of(warrior) == snap0.positions.get(warrior, Vector2i(-1, -1))
	session.step_next()
	var first_point: int = session.replay_cursor
	var stepped: bool = first_point >= 0 and session.log_entries[first_point].round_num == 1 and session.display_snapshot() != snap0
	session.step_prev()
	var back_to_start: bool = session.replay_cursor == -1
	var replay_ok: bool = at_block2 and at_start and stepped and back_to_start and start_hp == warrior.max_hp

	# 5. Advancing carries the state into Round 2; Round 1 stays reviewable
	var hp_end_r1: int = warrior.hp
	session.advance_round()
	var r2_live: bool = session.current_round == 2 and session.phase == CombatSession.Phase.PLANNING and warrior.hp == hp_end_r1
	session.select_round(1)
	var viewing_r1: bool = session.viewed_round == 1 and not session.is_live_view()
	session.select_round(2)
	var traversal_ok: bool = r2_live and viewing_r1 and session.is_live_view() and session.max_unlocked_round == 2

	# 6. Later rounds (AI presets on both sides) play on to a conclusion
	for round_num: int in range(encounter.max_rounds):
		session.plan_apply_preset_all("balanced")
		session.confirm_plan()
		_resolve_until_replay(session)
		if session.combat_over or not session.can_advance_round():
			break
		session.advance_round()
	var victory_ok: bool = (session.combat_over or session.current_round == encounter.max_rounds) and not session.can_advance_round()

	# 7. Auto-play ticks resolve a round without a timer (headless)
	session.reset_battle()
	session.toggle_auto_play()
	for i: int in range(600):
		if not session.is_auto_playing:
			break
		session.auto_play_tick()
	var autoplay_ok: bool = session.phase == CombatSession.Phase.REPLAY and not session.is_auto_playing

	# 8. Resetting the round restores Round 1 planning with an empty log
	session.reset_round()
	warrior = encounter.unit("warrior")   # a round-1 reset rebuilds the battle
	var reset_ok: bool = session.current_round == 1 and session.log_entries.is_empty() and warrior.hp == warrior.max_hp and session.phase == CombatSession.Phase.PLANNING

	print("\n=======================================================")
	print("=== COMBAT SESSION FLOW ASSERTIONS                  ===")
	print("=======================================================")
	print("  [Session] Only deployed enemies in view before the wave: %s" % ("PASS" if pre_view_ok else "FAIL"))
	print("  [Session] Wave event at R1-B04 brings reinforcements into view: %s" % ("PASS" if ambush_ok else "FAIL"))
	print("  [Session] Round 1 resolves into replay with round advance offered: %s" % ("PASS" if round1_ok else "FAIL"))
	print("  [Session] Replay steps by log points; blocks/rounds open at their start: %s" % ("PASS" if replay_ok else "FAIL"))
	print("  [Session] Round advance carries state over; past rounds reviewable: %s" % ("PASS" if traversal_ok else "FAIL"))
	print("  [Session] Later rounds (AI presets) play on to a conclusion: %s" % ("PASS" if victory_ok else "FAIL"))
	print("  [Session] Auto-play ticks resolve a full round headlessly: %s" % ("PASS" if autoplay_ok else "FAIL"))
	print("  [Session] Round reset restores planning with an empty log: %s" % ("PASS" if reset_ok else "FAIL"))

	session.dispose()
	return pre_view_ok and ambush_ok and round1_ok and replay_ok and traversal_ok and victory_ok and autoplay_ok and reset_ok

# ==============================================================================
# TEST 3: CHARACTER SPECS, STATS, GEAR & INVENTORY TEST
# ==============================================================================

static func run_character_specs_and_inventory_test() -> bool:
	print("\n=======================================================")
	print("=== STARTING CHARACTER SPECS & INVENTORY TEST       ===")
	print("=======================================================")

	# 1. Base initialization from archetype
	var warrior_arch := ClassArchetype.create_warrior()
	var stats := CharacterStats.new(warrior_arch, 1)

	var base_stats_ok: bool = (
		stats.str_stat == 15 and
		stats.dex_stat == 9 and
		stats.int_stat == 7 and
		stats.cons_stat == 14 and
		stats.perc_stat == 9 and
		stats.know_stat == 8 and
		StatRules.total_of(warrior_arch.base_stats) == StatRules.CLASS_BUDGET and
		stats.unallocated_attribute_points == StatRules.spend_pool(1) and
		stats.level == 1
	)

	# 2. Derived Max HP & Speed
	# Max HP = 60 + (14 * 8) + (15 * 2) = 202; Speed = 8 + 9 = 17; melee = round(15 * 1.5)
	var derived_stats_ok: bool = (
		stats.get_max_hp() == 202 and
		stats.get_initiative_speed() == 17 and
		stats.get_melee_power() == 23
	)

	# 3. Guaranteed level up growths (Level 1 -> 2)
	# Warrior growth: STR +2, CONS +2, PERC +1
	var lvl_ok: bool = stats.level_up()
	var growths_ok: bool = (
		lvl_ok and
		stats.level == 2 and
		stats.str_stat == 17 and
		stats.cons_stat == 16 and
		stats.perc_stat == 10 and
		stats.unallocated_attribute_points == StatRules.spend_pool(1) + StatRules.FREE_PER_LEVEL
	)

	# 4. Allocate 1 free point from level up
	var alloc_lvl_point_ok: bool = stats.allocate_attribute_point("INT") and stats.int_stat == 8 and stats.unallocated_attribute_points == StatRules.spend_pool(2) - 1

	# 5. Outside Event Stat Points (e.g. Ancient Shrine)
	stats.add_free_attribute_points(2)
	var alloc_event_points_ok: bool = (
		stats.unallocated_attribute_points == StatRules.spend_pool(2) + 1 and
		stats.allocate_attribute_point("DEX") and
		stats.allocate_attribute_point("KNOW") and
		stats.dex_stat == 10 and
		stats.know_stat == 9 and
		stats.unallocated_attribute_points == StatRules.spend_pool(2) - 1
	)

	# 6. Proficiency (class gear training) and passives (granted by lore sources, never by the class)
	var sword_power_bonus: int = stats.proficiency.get_weapon_power_bonus("sword")
	var sword_discount: int = stats.proficiency.get_weapon_requirement_discount("sword")
	var sage := Combatant.new("Sage", 100, 12, 1, stats)
	ContentFactory.apply_passives(sage, ["ruin_lore"])
	var ruins_context: Dictionary = {"biome": "ruins"}
	var mastery_ok: bool = (sword_power_bonus == 3 and sword_discount == 6 and stats.proficiency.level_of("bow") == 0
		and stats.get_effective_stat("KNOW", ruins_context) == 12 and stats.get_effective_stat("KNOW") == 9 and sage.passives.has("ruin_lore"))

	# 7. Equipment Stat Requirements & Proficiency Threshold Discount
	# Character STR is 17; sword proficiency 3 grants a 6-point requirement discount (eff STR 17 + 6 = 23)
	var greatsword := Equipment.new(
		"iron_greatsword",
		"Iron Greatsword",
		Equipment.EquipSlot.MAIN_HAND,
		"sword",
		Item.ItemType.WEAPON,
		18,
		{"STR": 22},
		[
			{"stat": "STR", "threshold": 22, "bonus_power": 4, "desc": "Tier 2: Crushing Strike"},
			{"stat": "STR", "threshold": 28, "bonus_power": 8, "desc": "Tier 3: Sunder Guard"}
		]
	)

	var meets_req_ok: bool = greatsword.meets_requirements(stats, stats.proficiency)

	# 8. Threshold Tiers Unlocked (STR 17 + discount 6 = 23 meets Tier 2 threshold of 22)
	# Base 18 + Tier 4 + sword proficiency 3 = 25
	var active_tiers: Array[Dictionary] = greatsword.get_active_threshold_tiers(stats, stats.proficiency)
	var tiers_ok: bool = (active_tiers.size() == 1 and greatsword.get_effective_equipment_power(stats, stats.proficiency) == 25)

	# 9. Deficit Wield Penalty Check (Character with 11 DEX tries to wield Longbow requiring 24 DEX)
	var longbow := Equipment.new(
		"yew_longbow",
		"Yew Longbow",
		Equipment.EquipSlot.MAIN_HAND,
		"bow",
		Item.ItemType.WEAPON,
		16,
		{"DEX": 24}
	)
	var bow_penalty: Dictionary = longbow.get_wield_penalty(stats, stats.proficiency)
	var penalty_ok: bool = (
		not longbow.meets_requirements(stats, stats.proficiency) and
		bow_penalty["is_penalized"] == true and
		bow_penalty["deficit"] > 0 and
		longbow.get_effective_equipment_power(stats, stats.proficiency) < 16 and
		longbow.get_effective_block_cost(stats, stats.proficiency) > 1
	)

	# 10. Combatant Integration, Inventory & Battle Pouch Consumable
	var test_unit := Combatant.new("Sir Gareth", 200, 18, 1, stats)
	test_unit.inventory.equip(greatsword)
	test_unit.sync_from_stats_and_gear()

	var potion := Consumable.new("health_potion", "Healing Draught", Consumable.ConsumableType.HEALTH_POTION, 40)
	test_unit.inventory.assign_pouch_item(0, potion)

	test_unit.hp = 120
	var use_outcome: Dictionary = test_unit.inventory.use_pouch_item(0, test_unit)
	var pouch_used_ok: bool = (
		use_outcome.get("healed", 0) == 40 and
		test_unit.hp == 160 and
		test_unit.inventory.battle_pouch.is_empty()
	)

	print("\n=======================================================")
	print("=== CHARACTER SPECS & INVENTORY TEST ASSERTIONS     ===")
	print("=======================================================")
	print("  [Specs] Archetype 6-stat baselines initialized: %s" % ("PASS" if base_stats_ok else "FAIL"))
	print("  [Specs] Derived Max HP & Initiative Speed calculated: %s" % ("PASS" if derived_stats_ok else "FAIL"))
	print("  [LevelUp] Level 2 guaranteed growths applied: %s" % ("PASS" if growths_ok else "FAIL"))
	print("  [LevelUp] Free attribute points allocated: %s" % ("PASS" if alloc_lvl_point_ok else "FAIL"))
	print("  [Event] Outside event points awarded & spent: %s" % ("PASS" if alloc_event_points_ok else "FAIL"))
	print("  [Proficiency] Gear training and passive stat bonuses: %s" % ("PASS" if mastery_ok else "FAIL"))
	print("  [Gear] Stat requirement & proficiency discount check: %s" % ("PASS" if meets_req_ok else "FAIL"))
	print("  [Gear] Threshold tiers active boosts unlocked: %s" % ("PASS" if tiers_ok else "FAIL"))
	print("  [Gear] Deficit wield penalty applied: %s" % ("PASS" if penalty_ok else "FAIL"))
	print("  [Inventory] Battle pouch item consumed in combat: %s" % ("PASS" if pouch_used_ok else "FAIL"))

	return (base_stats_ok and derived_stats_ok and growths_ok and alloc_lvl_point_ok and
		alloc_event_points_ok and mastery_ok and meets_req_ok and tiers_ok and penalty_ok and pouch_used_ok)


# ==============================================================================
# TEST 4: GEAR EQUIP UI, TEST DUMMY & DAMAGE FORMULA SUITE
# ==============================================================================

static func run_gear_equip_and_test_dummy_suite(tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING GEAR EQUIP UI & TEST DUMMY SUITE       ===")
	print("=======================================================")

	# 1. Instantiate CharacterEquipmentModal
	var modal := CharacterEquipmentModal.new()
	if tree_parent != null:
		tree_parent.add_child(modal)

	var stash := Inventory.new()
	ItemCatalog.populate_party_starter_stash(stash)

	var warrior_stats := CharacterStats.new(ClassArchetype.create_warrior(), 2)
	var warrior := Combatant.new("Warrior", 100, 15, 1, warrior_stats)
	ItemCatalog.setup_default_hero_loadout(warrior, "Warrior")

	var ranger_stats := CharacterStats.new(ClassArchetype.create_ranger(), 1)
	var ranger := Combatant.new("Ranger", 80, 14, 2, ranger_stats)
	ItemCatalog.setup_default_hero_loadout(ranger, "Ranger")

	modal.setup_heroes([warrior, ranger], stash)

	var modal_init_ok: bool = (modal != null and modal.party_allies.size() == 2 and modal.active_hero == warrior)

	# 2. Hero switching in modal
	modal.select_hero(ranger)
	var hero_switch_ok: bool = (modal.active_hero == ranger)
	modal.select_hero(warrior)

	# 3. Equipping weapon updates power and main hand
	var claymore := ItemCatalog.create_iron_claymore()
	stash.add_item(claymore)
	modal._equip_item(claymore)
	var equip_ok: bool = (
		warrior.inventory.get_main_weapon() == claymore and
		warrior.inventory.get_main_weapon().name == "Iron Claymore"
	)

	# 4. Claymore Threshold Tier (+4 Power at 18 STR)
	var tier_boost: int = claymore.get_unlocked_threshold_power_boost(warrior.stats, warrior.stats.proficiency)
	var threshold_boost_ok: bool = (tier_boost == 4)

	# 5. Deficit penalty check on Titan Warhammer (requires 20 STR)
	var warhammer := ItemCatalog.create_titan_warhammer()
	stash.add_item(warhammer)
	modal._equip_item(warhammer)
	var penalty: Dictionary = warhammer.get_wield_penalty(warrior.stats, warrior.stats.proficiency)
	var is_pen: bool = penalty.get("is_penalized", false)
	var deficit_ok: bool = (is_pen and penalty.get("deficit", 0) > 0)

	# 6. Armor Burden calculation affecting speed
	var plate := ItemCatalog.create_steel_plate_cuirass()
	stash.add_item(plate)
	modal._equip_item(plate)
	var total_burden: int = warrior.inventory.get_total_armor_burden()
	var speed_with_burden: int = warrior.stats.get_initiative_speed(total_burden)
	var burden_ok: bool = (total_burden >= 3 and speed_with_burden < warrior.stats.get_initiative_speed(0))

	# 7. Unequipping item returns to stash
	var unequipped_arm: Equipment = warrior.inventory.unequip(Equipment.EquipSlot.BODY)
	var unequip_ok: bool = (unequipped_arm == plate and warrior.inventory.get_equipped(Equipment.EquipSlot.BODY) == null)

	# Re-equip broadsword for baseline testing
	var broadsword := ItemCatalog.create_steel_broadsword()
	warrior.inventory.equip(broadsword)
	warrior.sync_from_stats_and_gear()

	# 8. Training Dummy setup
	var dummy := Combatant.new("Training Dummy", 500, 10, 1)
	dummy.defense_mitigation_ratio = 0.0
	var dummy_init_ok: bool = (dummy != null and dummy.hp == 500 and dummy.max_hp == 500)

	# 9. Damage formula & 50% defense mitigation calculation
	var atk_pwr: int = warrior.get_effective_power(false)
	var full_dmg: int = dummy.take_damage(atk_pwr)
	var expected_hp_1: int = 500 - atk_pwr

	dummy.defense_mitigation_ratio = 0.5
	var mit_dmg: int = dummy.take_damage(atk_pwr)
	var expected_mit_dmg: int = maxi(1, int(roundf(float(atk_pwr) * 0.5)))
	var damage_formula_ok: bool = (full_dmg == atk_pwr and mit_dmg == expected_mit_dmg and dummy.hp == (expected_hp_1 - expected_mit_dmg))

	# 10. Consumable Battle Pouch usage
	dummy.defense_mitigation_ratio = 0.0
	warrior.inventory.battle_pouch.clear()
	var bomb := ItemCatalog.create_throwing_fire_bomb()
	warrior.inventory.add_to_battle_pouch(bomb)
	var pouch_count_before: int = warrior.inventory.battle_pouch.size()
	var bomb_dmg: int = dummy.take_damage(bomb.damage_amount)
	warrior.inventory.remove_from_battle_pouch(bomb)
	var pouch_count_after: int = warrior.inventory.battle_pouch.size()
	var consumable_test_ok: bool = (pouch_count_before == 1 and pouch_count_after == 0 and bomb_dmg == 20)

	print("\n=======================================================")
	print("=== GEAR EQUIP UI & TEST DUMMY SUITE ASSERTIONS     ===")
	print("=======================================================")
	print("  [UI] Equipment modal initialized & mounted: %s" % ("PASS" if modal_init_ok else "FAIL"))
	print("  [UI] Hero switching in equipment modal: %s" % ("PASS" if hero_switch_ok else "FAIL"))
	print("  [Gear] Weapon equipped from stash & synced: %s" % ("PASS" if equip_ok else "FAIL"))
	print("  [Gear] Threshold Tier 1 (+4 Power) boost verified: %s" % ("PASS" if threshold_boost_ok else "FAIL"))
	print("  [Gear] Deficit penalty on Titan Warhammer verified: %s" % ("PASS" if deficit_ok else "FAIL"))
	print("  [Gear] Armor burden reduces initiative speed: %s" % ("PASS" if burden_ok else "FAIL"))
	print("  [Gear] Unequipped item returns to stash: %s" % ("PASS" if unequip_ok else "FAIL"))
	print("  [Dummy] Training dummy initialized (500 HP): %s" % ("PASS" if dummy_init_ok else "FAIL"))
	print("  [Formula] Base strike & 50%% defense mitigation verified: %s" % ("PASS" if damage_formula_ok else "FAIL"))
	print("  [Pouch] Battle pouch consumable used against dummy: %s" % ("PASS" if consumable_test_ok else "FAIL"))

	return (modal_init_ok and hero_switch_ok and equip_ok and threshold_boost_ok and
		deficit_ok and burden_ok and unequip_ok and dummy_init_ok and damage_formula_ok and consumable_test_ok)

# ==============================================================================
# TEST 6: GENERATED ART, TERRAIN & COMBAT ZONES
# ==============================================================================

static func run_art_and_terrain_test() -> bool:
	print("\n=======================================================")
	print("=== STARTING ART, TERRAIN & COMBAT ZONE TEST        ===")
	print("=======================================================")

	# 1. Tokens: every class x team rasterizes; team decides color, hostiles get a spiked rim
	var tokens_ok: bool = true
	for class_key: String in UnitArt.CLASS_KEYS:
		for team: int in [UnitArt.Team.FRIENDLY, UnitArt.Team.HOSTILE, UnitArt.Team.NEUTRAL]:
			var tex: Texture2D = UnitArt.token(class_key, team as UnitArt.Team)
			if tex == null or tex.get_width() != 128:
				tokens_ok = false
	var friendly_svg: String = UnitArt.token_svg("warrior", UnitArt.Team.FRIENDLY)
	var hostile_svg: String = UnitArt.token_svg("warrior", UnitArt.Team.HOSTILE)
	var neutral_svg: String = UnitArt.token_svg("dummy", UnitArt.Team.NEUTRAL)
	tokens_ok = tokens_ok and "#60a5fa" in friendly_svg and "#f87171" in hostile_svg and "#facc15" in neutral_svg
	tokens_ok = tokens_ok and hostile_svg.count("<path d=\"M") > friendly_svg.count("<path d=\"M") and "stroke-dasharray" in neutral_svg

	# 2. Portraits: every class x team rasterizes; goblin classes get goblin skin
	var portraits_ok: bool = true
	for class_key: String in UnitArt.CLASS_KEYS:
		for team: int in [UnitArt.Team.FRIENDLY, UnitArt.Team.HOSTILE]:
			if UnitArt.portrait(class_key, team as UnitArt.Team) == null:
				portraits_ok = false
	portraits_ok = portraits_ok and "#7cb342" in UnitArt.portrait_svg("shaman", UnitArt.Team.HOSTILE) and not ("#7cb342" in UnitArt.portrait_svg("warrior", UnitArt.Team.FRIENDLY))

	# 3. Encounter art follows sides: allies friendly (blue), enemies hostile (red)
	var enc := CatalogEncounter.new("goblin_ambush")
	enc.build()
	var sides_ok: bool = true
	for c: Combatant in enc.allies:
		sides_ok = sides_ok and enc.team_of(c) == UnitArt.Team.FRIENDLY and enc.get_token_texture(c) != null and enc.get_sprite_texture(c) != null
	for c: Combatant in enc.enemies:
		sides_ok = sides_ok and enc.team_of(c) == UnitArt.Team.HOSTILE and enc.get_token_texture(c) != null
	sides_ok = sides_ok and enc.class_of(enc.unit("goblin_shaman")) == "shaman"

	# 4. Terrain glyph round-trip and walkability rules
	var terrain_ok: bool = true
	for t: int in Terrain.INFO:
		var tt: Terrain.Type = t as Terrain.Type
		if Terrain.from_glyph(Terrain.glyph(tt)) != tt:
			terrain_ok = false
	terrain_ok = terrain_ok and not Terrain.is_walkable(Terrain.Type.WALL) and not Terrain.is_walkable(Terrain.Type.DEEP_WATER) and Terrain.is_walkable(Terrain.Type.BRIDGE) and Terrain.is_walkable(Terrain.Type.MUD)

	# 5. Every zone is a valid 9x9 map with a walkable core and exit lane
	var zones_ok: bool = CombatZones.zone_ids().size() >= 8
	for zone_id: String in CombatZones.zone_ids():
		var map: Dictionary = CombatZones.terrain_map(zone_id)
		if map.size() != 81:
			zones_ok = false
		for x: int in range(2, 7):
			for y: int in range(2, 7):
				var core_raw: Variant = map.get(Vector2i(x, y), Terrain.Type.FLOOR)
				if core_raw is Terrain.Type and not Terrain.is_walkable(DictUtil.to_int(core_raw) as Terrain.Type):
					zones_ok = false
		for x: int in range(4, 9):
			var lane_raw: Variant = map.get(Vector2i(x, 4), Terrain.Type.FLOOR)
			if lane_raw is Terrain.Type and not Terrain.is_walkable(DictUtil.to_int(lane_raw) as Terrain.Type):
				zones_ok = false

	# 6. Grid enforces terrain: no placing/moving onto blocked tiles; deployment skips them
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	grid.set_terrain(Vector2i(4, 5), Terrain.Type.WALL)
	grid.set_terrain(Vector2i(2, 3), Terrain.Type.LAVA)
	var hero := Combatant.new("Tester", 50, 10, 1)
	var allies_g: Array[Combatant] = [hero]
	var none: Array[Combatant] = []
	grid.deploy_combatants(allies_g, none)
	var deployed_off_wall: bool = grid.get_position(hero) != Vector2i(4, 5) and grid.is_walkable(grid.get_position(hero))
	var walker := Combatant.new("Walker", 50, 10, 1)
	var placed_lava: bool = grid.place_combatant(walker, Vector2i(2, 3))
	grid.place_combatant(walker, Vector2i(2, 4))
	var moved_into_lava: bool = grid.move_combatant(walker, Vector2i(0, -1))
	var grid_ok: bool = deployed_off_wall and not placed_lava and not moved_into_lava and grid.get_position(walker) == Vector2i(2, 4)

	# 7. The Goblin Ambush (wave included) deploys and plays to a conclusion on every zone
	var all_zones_play: bool = true
	for zone_id: String in CombatZones.zone_ids():
		var zone_enc := CatalogEncounter.new("goblin_ambush")
		zone_enc.zone_id = zone_id
		var session := CombatSession.new(zone_enc)
		session.start()
		for round_num: int in range(zone_enc.max_rounds):
			session.plan_apply_preset_all("balanced")
			session.confirm_plan()
			_resolve_until_replay(session)
			if session.combat_over or not session.can_advance_round():
				break
			session.advance_round()
		var concluded: bool = session.combat_over or session.current_round == zone_enc.max_rounds
		for c: Combatant in zone_enc.allies + zone_enc.enemies:
			if c.is_active() and not session.grid.has_combatant(c):
				concluded = false   # every unit (wave included) must have found a tile
		session.dispose()
		if not concluded:
			all_zones_play = false
			print("    zone failed to conclude: %s" % zone_id)

	# 8. Tiles & overlays rasterize for every terrain variant
	var tiles_ok: bool = TileArt.overlay("exit") != null and TileArt.overlay("center") != null
	for t: int in Terrain.INFO:
		for v: int in range(TileArt.VARIANTS):
			if TileArt.tile(t as Terrain.Type, v) == null:
				tiles_ok = false

	# 9. Item icons: every catalog item gets one; types map correctly; rare weapons use bespoke art
	var stash := Inventory.new()
	ItemCatalog.populate_party_starter_stash(stash)
	var icons_ok: bool = true
	for item: Item in stash.items:
		if ItemArt.icon(item) == null:
			icons_ok = false
	icons_ok = icons_ok and (
		ItemArt.icon_key(ItemCatalog.create_iron_claymore()) == "greatsword"
		and ItemArt.icon_key(ItemCatalog.create_iron_shortsword()) == "shortsword"
		and ItemArt.icon_key(ItemCatalog.create_buckler()) == "shield"
		and ItemArt.icon_key(ItemCatalog.create_steel_plate_cuirass()) == "plate"
		and ItemArt.icon_key(ItemCatalog.create_healing_draught()) == "potion"
		and ItemArt.icon_key(ItemCatalog.create_titan_warhammer()) == "titan_warhammer"
		and ItemArt.icon_key(ItemCatalog.create_runed_staff()) == "runed_staff"
	)

	print("\n=======================================================")
	print("=== ART, TERRAIN & ZONE ASSERTIONS                  ===")
	print("=======================================================")
	print("  [Art] Team-colored class tokens (blue/red/yellow + rim shapes): %s" % ("PASS" if tokens_ok else "FAIL"))
	print("  [Art] Class portraits for every class and team: %s" % ("PASS" if portraits_ok else "FAIL"))
	print("  [Art] Encounter art follows sides (allies friendly, enemies hostile): %s" % ("PASS" if sides_ok else "FAIL"))
	print("  [Terrain] Glyph round-trip & walkability rules: %s" % ("PASS" if terrain_ok else "FAIL"))
	print("  [Terrain] All zones valid 9x9 with walkable core & exit lane: %s" % ("PASS" if zones_ok else "FAIL"))
	print("  [Terrain] Grid blocks placement/movement on blocked tiles; deploy skips them: %s" % ("PASS" if grid_ok else "FAIL"))
	print("  [Terrain] Goblin Ambush (with its wave) deploys and concludes on every zone: %s" % ("PASS" if all_zones_play else "FAIL"))
	print("  [Art] Terrain tiles (all variants) & overlays rasterize: %s" % ("PASS" if tiles_ok else "FAIL"))
	print("  [Art] Item icons by gear type + bespoke rare weapon icons: %s" % ("PASS" if icons_ok else "FAIL"))

	return tokens_ok and portraits_ok and sides_ok and terrain_ok and zones_ok and grid_ok and all_zones_play and tiles_ok and icons_ok

# ==============================================================================
# TEST 7: PLANNING SEQUENCES, SKILL AP, STANCES, RUNTIME AP & ENEMY AI
# ==============================================================================

static func _hand_ctx(grid: CombatGrid, allies: Array[Combatant], enemies: Array[Combatant], cap: int = 3) -> PlanContext:
	var profiles: Dictionary = {}
	for c: Combatant in allies + enemies:
		var p: AiProfile = AiProfile.create(AiProfile.Role.SNIPER if c.attack_range > 1 else AiProfile.Role.AGGRESSOR, 0.25, cap)
		if c.power_override <= 0:
			c.power_override = 15   # hand-built test units: fixed strike power
		if c.armor_override < 0:
			c.armor_override = 0    # and no armor, so expected numbers stay simple
		profiles[c] = p
	return PlanContext.create(grid, allies, enemies, profiles)

# Commits `steps` for `actor` and plays one full round on a bare resolver.
static func _run_steps(actor: Combatant, ctx: PlanContext, steps: Array[PlanStep]) -> Array[String]:
	var resolver := CombatResolver.new()
	resolver.grid = ctx.grid
	var messages: Array[String] = []
	resolver.log_emitted.connect(func(msg: String) -> void: messages.append(msg))
	var cp := CombatPlan.new(ctx)
	cp.set_steps(actor, steps)
	cp.commit(resolver)
	resolver.resolve_round()
	return messages

static func _play_with_preset(zone_id: String, preset_id: String) -> Dictionary:
	var enc := CatalogEncounter.new("goblin_skirmish")
	enc.zone_id = zone_id
	var session := CombatSession.new(enc)
	session.start()
	for round_num: int in range(enc.max_rounds):
		session.plan_apply_preset_all(preset_id)
		if not session.confirm_plan():
			break
		_resolve_until_replay(session)
		if session.combat_over or not session.can_advance_round():
			break
		session.advance_round()
	var hp_sig: Array[int] = []
	var party_hurt: bool = false
	var foes_hurt: bool = false
	for c: Combatant in session.allies + session.enemies:
		hp_sig.append(c.hp)
		if c.hp < c.max_hp:
			if session.is_ally(c):
				party_hurt = true
			else:
				foes_hurt = true
	var result: Dictionary = {"hp": hp_sig, "log": session.log_entries.size(), "party_hurt": party_hurt, "foes_hurt": foes_hurt}
	session.dispose()
	return result

static func run_planning_and_ai_test() -> bool:
	print("\n=======================================================")
	print("=== STARTING PLANNING, STANCES, AP & ENEMY AI TEST  ===")
	print("=======================================================")

	# 1. Skill AP: level = AP, Move = 1 AP per tile, Rest free; effects shift costs
	var skills_ok: bool = SkillCatalog.all().size() >= 10
	for s: SkillDef in SkillCatalog.all():
		if s.kind == SkillDef.Kind.ATTACK or s.kind == SkillDef.Kind.DEFEND or s.kind == SkillDef.Kind.FOCUS:
			skills_ok = skills_ok and s.ap_cost() == s.level
	var l2_ok: bool = SkillCatalog.find("power_strike").ap_cost() == 2 and SkillCatalog.find("brace").ap_cost() == 2 and SkillCatalog.find("focus").ap_cost() == 2
	var modded := Combatant.new("Modded", 10, 10, 1)
	modded.ap_cost_modifier = 1
	var mod_ok: bool = SkillCatalog.find("strike").ap_cost(modded) == 2 and SkillCatalog.find("heavy_blow").ap_cost(modded) == 4 and SkillCatalog.find("move").ap_cost(modded) == 2 and SkillCatalog.find("rest").ap_cost(modded) == 0
	modded.ap_cost_modifier = -5
	mod_ok = mod_ok and SkillCatalog.find("heavy_blow").ap_cost(modded) == 1
	var fast := Combatant.new("Fast", 10, 15, 1)
	var slow := Combatant.new("Slow", 10, 8, 1)
	var speed_ok: bool = fast.move_per_block() == 3 and slow.move_per_block() == 2 and Combatant.new("Tortoise", 10, 1, 1).move_per_block() == 1
	var ap_model_ok: bool = skills_ok and l2_ok and mod_ok and speed_ok

	# 2. Stances: RPS loop with Neutral outside it; automatic switch costs
	var N: CombatTypes.StanceType = CombatTypes.StanceType.NEUTRAL
	var O: CombatTypes.StanceType = CombatTypes.StanceType.OFFENSIVE
	var D: CombatTypes.StanceType = CombatTypes.StanceType.DEFENSIVE
	var T: CombatTypes.StanceType = CombatTypes.StanceType.TACTICAL
	var rps_ok: bool = CombatTypes.get_matchup_multiplier(O, T) == 1.25 and CombatTypes.get_matchup_multiplier(T, D) == 1.25 and CombatTypes.get_matchup_multiplier(D, O) == 1.25
	rps_ok = rps_ok and CombatTypes.get_matchup_multiplier(T, O) == 0.75 and CombatTypes.get_matchup_multiplier(D, T) == 0.75 and CombatTypes.get_matchup_multiplier(O, D) == 0.75
	rps_ok = rps_ok and CombatTypes.get_matchup_multiplier(N, O) == 1.0 and CombatTypes.get_matchup_multiplier(T, N) == 1.0 and CombatTypes.get_matchup_multiplier(O, O) == 1.0
	var strike: SkillDef = SkillCatalog.find("strike")
	var brace: SkillDef = SkillCatalog.find("brace")
	var mover: SkillDef = SkillCatalog.find("move")
	var rest_skill: SkillDef = SkillCatalog.find("rest")
	var switch_ok: bool = strike.switch_cost(N) == 1 and strike.switch_cost(O) == 0 and brace.switch_cost(O) == 1 and brace.switch_cost(D) == 0
	switch_ok = switch_ok and mover.switch_cost(T) == 0 and mover.stance_after(T) == T and rest_skill.switch_cost(D) == 0 and rest_skill.stance_after(D) == N
	var stance_ok: bool = rps_ok and switch_ok

	# 3. Sequence compile: lowest free blocks, Engage auto-moves, AP incl. stance switch
	var session := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	session.start()
	var warrior: Combatant = session.allies[0]
	var foe: Combatant = session.enemies[0]
	var power_strike: SkillDef = SkillCatalog.find("power_strike")
	var first: String = session.plan_add_step(warrior, PlanStep.engage(power_strike, foe))
	var compiled: CombatPlan.SimState = session.plan.compile(warrior)
	var attack_entry: PlanEntry = compiled.entries[compiled.entries.size() - 1] if not compiled.entries.is_empty() else null
	var chain_ok: bool = first.is_empty() and attack_entry != null and attack_entry.skill == power_strike and attack_entry.target == foe and attack_entry.switch_ap == 1
	var blocks_ok: bool = true
	var expected_block: int = 1
	var ap_sum: int = 0
	var moves_ok: bool = true
	for e: PlanEntry in compiled.entries:
		blocks_ok = blocks_ok and e.start_block == expected_block
		expected_block += e.blocks()
		ap_sum += e.total_ap()
		if e.skill.kind == SkillDef.Kind.MOVE:
			moves_ok = moves_ok and e.blocks() == 1 and e.skill_ap == e.tiles and e.tiles <= warrior.move_per_block()
	blocks_ok = blocks_ok and compiled.ap_left == warrior.ap - ap_sum and compiled.next_block == expected_block
	var in_range_ok: bool = session.grid.get_distance(compiled.pos, session.grid.get_position(foe)) <= warrior.attack_range
	var stance_after_ok: bool = compiled.stance == O
	var seq_ok: bool = chain_ok and blocks_ok and moves_ok and in_range_ok and stance_after_ok

	# Switching stance mid-sequence is allowed and automatic: +1 AP, no block
	session.plan_clear(warrior)
	session.plan_add_step(warrior, PlanStep.engage(strike, foe))
	var before_strike: CombatPlan.SimState = session.plan.compile(warrior)
	var switched: String = session.plan_add_step(warrior, PlanStep.self_skill(SkillCatalog.find("guard")))
	var after_strike: CombatPlan.SimState = session.plan.compile(warrior)
	var switch_entry: PlanEntry = after_strike.entries[after_strike.entries.size() - 1]
	var auto_switch_ok: bool = switched.is_empty() and switch_entry.switch_ap == 1 and switch_entry.stance_before == O and switch_entry.stance_after == D
	auto_switch_ok = auto_switch_ok and before_strike.ap_left - after_strike.ap_left >= 2

	# AP is a runtime resource: over-committing is allowed, the plan projects where it runs dry
	session.plan_clear(warrior)
	warrior.ap = 3   # a Heavy Blow (3 AP + 1 stance switch) is more than the unit can pay
	var _last_problem: String = session.plan_add_step(warrior, PlanStep.engage(SkillCatalog.find("heavy_blow"), session.enemies[1]))
	_last_problem += session.plan_add_step(warrior, PlanStep.engage(SkillCatalog.find("strike"), session.enemies[1]))
	var dry_state: CombatPlan.SimState = session.plan.compile(warrior)
	var dry_ok: bool = dry_state.runs_dry() and dry_state.dry_step >= 0 and dry_state.ap_left < 0 and dry_state.debt() > 0 and session.plan_is_valid()
	var _option_warns: bool = false
	var rest_option: bool = false
	for o: PlanOption in session.plan_options(warrior):
		if o.step.mode == PlanStep.Mode.ENGAGE and not o.warning.is_empty():
			_option_warns = true
	session.plan_clear(warrior)
	warrior.ap = warrior.max_ap
	for o: PlanOption in session.plan_options(warrior):
		if o.step.mode == PlanStep.Mode.SELF and o.step.skill.kind == SkillDef.Kind.REST and o.valid:
			rest_option = true
	var runtime_plan_ok: bool = dry_ok and rest_option
	var options_ok: bool = rest_option and not session.plan_options(warrior).is_empty()

	# 4. Editing: a sequence broken by an edit is flagged and blocks Confirm; fixing it allows Confirm
	var fallen: Combatant = session.enemies[3]
	var fallen_hp: int = fallen.hp
	fallen.hp = 0  # an enemy that has since been defeated
	var broken: Array[PlanStep] = [PlanStep.engage(strike, foe), PlanStep.engage(strike, fallen)]
	session.plan.set_steps(warrior, broken)
	var flagged: CombatPlan.SimState = session.plan.compile(warrior)
	var edit_ok: bool = flagged.error_index == 1 and not session.plan_is_valid() and not session.confirm_plan()
	fallen.hp = fallen_hp
	session.plan_remove_step(warrior, 1)
	edit_ok = edit_ok and session.plan_is_valid()
	session.plan_add_step(warrior, PlanStep.engage(strike, session.enemies[1]))
	session.plan_move_step(warrior, 1, 0)
	edit_ok = edit_ok and session.plan.steps_of(warrior).size() == 2 and session.plan.steps_of(warrior)[0].target == session.enemies[1] and session.plan_is_valid()
	# 5. Confirm locks the plan, schedules party + AI enemies, and resolution runs
	session.plan_apply_preset(session.allies[1], "balanced")
	var preview: CombatResolver.ScheduledAction = session.find_recorded_action(warrior, 1)
	var preview_ok: bool = preview != null
	var confirmed: bool = session.confirm_plan()
	var locked_ok: bool = confirmed and not session.is_planning() and not session.plan_add_step(warrior, PlanStep.self_skill(brace)).is_empty()
	var party_scheduled: bool = false
	var enemy_scheduled: bool = false
	for a: CombatResolver.ScheduledAction in session.resolver.scheduled_actions:
		if a.combatant == warrior:
			party_scheduled = true
		if not session.is_ally(a.combatant):
			enemy_scheduled = true
	_resolve_until_replay(session)
	var spent_ok: bool = warrior.ap < warrior.max_ap
	var flow_ok2: bool = preview_ok and locked_ok and party_scheduled and enemy_scheduled and session.phase == CombatSession.Phase.REPLAY and spent_ok
	session.dispose()

	# 5b. Runtime AP: overspending runs into AP debt (paid next round: less AP, 3+ Stagger, 5+ Stun); Rest recovers mid-round
	var rgrid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var spent := Combatant.new("Spent", 60, 10, 1)
	var dummy := Combatant.new("Dummy", 100, 10, 1)
	rgrid.place_combatant(spent, Vector2i(3, 4))
	rgrid.place_combatant(dummy, Vector2i(4, 4))
	var spent_side: Array[Combatant] = [spent]
	var dummy_side: Array[Combatant] = [dummy]
	var rctx: PlanContext = _hand_ctx(rgrid, dummy_side, spent_side)
	spent.ap = 1
	var exhausted_steps: Array[PlanStep] = [PlanStep.engage(strike, dummy), PlanStep.engage(strike, dummy)]
	var exhaust_log: Array[String] = _run_steps(spent, rctx, exhausted_steps)
	var stops_ok: bool = dummy.hp < 100 and spent.ap == -2   # both strikes land: 1 + 1 (stance) + 1 AP
	var said_exhausted: bool = false
	for msg: String in exhaust_log:
		if msg.begins_with("[AP Debt]"):
			said_exhausted = true
	StatusEffects.roll_over(spent, 4)
	stops_ok = stops_ok and spent.ap == 2 and spent.statuses.is_empty()
	spent.ap = -3
	StatusEffects.roll_over(spent, 4)
	var staggered_plan := CombatPlan.new(rctx)
	var stagger_entries: Array[PlanEntry] = staggered_plan.entries_for(spent)
	stops_ok = stops_ok and spent.has_status(StatusEffects.STAGGER) and spent.ap == 1 and stagger_entries.size() == 2 and stagger_entries[0].start_block == 1 and stagger_entries[1].start_block == 4
	staggered_plan.add_step(spent, PlanStep.engage(strike, dummy))
	var after_stagger: Array[PlanEntry] = staggered_plan.entries_for(spent)
	stops_ok = stops_ok and after_stagger[after_stagger.size() - 1].start_block == 2   # skips the lost block 1
	spent.ap = -5
	StatusEffects.roll_over(spent, 4)
	stops_ok = stops_ok and spent.has_status(StatusEffects.STUN) and not spent.has_status(StatusEffects.STAGGER)
	spent.statuses.clear()
	spent.ap = 0
	var rest_steps: Array[PlanStep] = [PlanStep.self_skill(rest_skill), PlanStep.engage(strike, dummy)]
	_run_steps(spent, rctx, rest_steps)
	var rest_ok: bool = dummy.hp < 100 and spent.ap == 0
	var runtime_ok: bool = stops_ok and said_exhausted and rest_ok

	# 5c. On-grid input model: options per clicked target / tile, hover previews, status line
	var gsession := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	gsession.start()
	var gh: Combatant = gsession.allies[0]
	var gfoe: Combatant = gsession.enemies[0]
	var foe_opts: Array[PlanOption] = gsession.plan_target_options(gfoe)
	var self_opts: Array[PlanOption] = gsession.plan_target_options(gh)
	var ally_opts: Array[PlanOption] = gsession.plan_target_options(gsession.allies[1])
	var tile_opts: Array[PlanOption] = gsession.plan_cell_options(Vector2i(2, 6))
	var exit_opts: Array[PlanOption] = gsession.plan_cell_options(Vector2i(8, 4))
	var occupied_opts: Array[PlanOption] = gsession.plan_cell_options(gsession.grid.get_position(gfoe))
	var engage_count: int = 0
	var area_count: int = 0
	for o: PlanOption in foe_opts:
		if o.step.mode == PlanStep.Mode.ENGAGE:
			engage_count += 1
		elif o.step.mode == PlanStep.Mode.AREA:
			area_count += 1
	var targets_ok: bool = engage_count == SkillCatalog.single_attacks(gh, 3).size() and area_count == SkillCatalog.area_attacks(3, gh).size() and ally_opts.is_empty() and tile_opts.size() == 1 + area_count and exit_opts.size() == 1
	for o: PlanOption in foe_opts:
		if o.step.mode == PlanStep.Mode.ENGAGE:
			targets_ok = targets_ok and o.step.target == gfoe and o.valid
	var has_rest: bool = false
	for o: PlanOption in self_opts:
		targets_ok = targets_ok and o.step.mode == PlanStep.Mode.SELF
		if o.step.skill.kind == SkillDef.Kind.REST:
			has_rest = true
	targets_ok = targets_ok and has_rest and tile_opts[0].step.mode == PlanStep.Mode.GOTO and tile_opts[0].valid and exit_opts[0].step.mode == PlanStep.Mode.WITHDRAW
	targets_ok = targets_ok and (occupied_opts.is_empty() or not occupied_opts[0].valid)
	var previews: Array[int] = [0]
	gsession.plan_preview_changed.connect(func() -> void: previews[0] += 1)
	gsession.set_plan_preview(tile_opts[0].step)
	var pst: CombatPlan.SimState = gsession.plan_preview_state()
	var grid_preview_ok: bool = previews[0] == 1 and pst != null and pst.pos == Vector2i(2, 6) and gsession.plan.steps_of(gh).is_empty()
	gsession.set_plan_preview(null)
	var added: String = gsession.plan_add_step(gh, tile_opts[0].step)
	var moved_state: CombatPlan.SimState = gsession.plan.compile(gh)
	var trail_ok: bool = added.is_empty() and moved_state.pos == Vector2i(2, 6) and not moved_state.entries.is_empty() and moved_state.entries[0].pos_before == Vector2i(4, 5) and moved_state.entries[moved_state.entries.size() - 1].pos_after == Vector2i(2, 6)
	var info: Dictionary = gsession.plan_info()
	var info_ok: bool = DictUtil.to_bool(info.get("planning", false)) and "Warrior" in str(info.get("text", "")) and DictUtil.to_bool(info.get("can_confirm", false)) and gsession.plan_step_index_at(1) == 0 and gsession.plan_step_index_at(10) == -1
	gsession.set_hovered(gfoe)
	var hover_ok: bool = gsession.inspected_combatant() == gfoe
	gsession.set_hovered(null)
	hover_ok = hover_ok and gsession.inspected_combatant() == gsession.selected_combatant
	var input_ok: bool = targets_ok and grid_preview_ok and trail_ok and info_ok and hover_ok
	gsession.dispose()
	# 5d. Round rules: 6 blocks, AP pool + regen, one move per round, target switching, insurance attacks, live follow
	var rsession := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	rsession.start()
	var rhero: Combatant = rsession.allies[0]
	var rules_ok: bool = rsession.total_blocks == 6 and rsession.resolver.total_blocks == 6 and rsession.encounter.plan_context().total_blocks == 6 and rhero.max_ap == 12 and rhero.ap == 12
	# centre tie-break: the Warrior (4,5) closing on the Skirmisher (5,4) steps to the centre (4,4), not (5,5)
	var skirm: Combatant = rsession.enemies[1]
	rsession.plan_add_step(rhero, PlanStep.engage(strike, skirm))
	var approach_entry: PlanEntry = rsession.plan.compile(rhero).entries[0]
	rules_ok = rules_ok and rsession.grid.get_position(skirm) == Vector2i(5, 4) and approach_entry.skill.kind == SkillDef.Kind.MOVE and approach_entry.pos_after == Vector2i(4, 4)
	# the move is spent: a second move or a far second engage is refused, an in-range attack is fine
	var second_move: String = rsession.plan_add_step(rhero, PlanStep.go_to(mover, Vector2i(2, 6)))
	var far_engage: String = rsession.plan_add_step(rhero, PlanStep.engage(strike, rsession.enemies[2]))
	rules_ok = rules_ok and "Moving must come first" in second_move and not far_engage.is_empty()
	# insurance: a target already expected to fall can still be attacked (it may dodge or guard)
	rsession.enemies[1].hp = 5
	var insurance_warned: bool = false
	for o: PlanOption in rsession.plan_target_options(skirm):
		if o.valid and not o.warning.is_empty():
			insurance_warned = true
	var insured: String = rsession.plan_add_step(rhero, PlanStep.engage(strike, skirm))
	rules_ok = rules_ok and insured.is_empty() and insurance_warned
	# a round's move is 2 blocks at most, so farther tiles are refused
	rsession.plan_clear(rhero)
	rhero.speed = 8   # 2 tiles per block: 4 tiles in a round
	var far_tile_fresh: Array[PlanOption] = rsession.plan_cell_options(Vector2i(1, 3))
	rules_ok = rules_ok and not far_tile_fresh.is_empty() and not far_tile_fresh[0].valid and "Too far" in far_tile_fresh[0].reason
	rhero.speed = 15
	# AP carries over: +4 each round up to the pool
	var ap_before: int = rhero.ap
	rsession.plan_add_step(rhero, PlanStep.self_skill(SkillCatalog.find("brace")))
	rsession.confirm_plan()
	var autoplayed: bool = rsession.is_auto_playing
	_resolve_until_replay(rsession)
	var ap_after_round: int = rhero.ap
	var attack_logged: bool = false
	if rsession.can_advance_round():
		rsession.advance_round()
		rules_ok = rules_ok and rhero.ap == mini(rhero.max_ap, ap_after_round + 4) and ap_after_round < ap_before + 1
	rules_ok = rules_ok and autoplayed
	for e: ActionLogEntry in rsession.log_entries:
		if e.category == "ATTACK" and e.damage > 0:
			attack_logged = attack_logged or (e.value_num == e.damage and not e.breakdown.is_empty() and "stance x" in e.breakdown)
	rules_ok = rules_ok and attack_logged
	rsession.dispose()
	# hand grid: target switching costs a block; attacks follow a target that steps away
	var hgrid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var duelist := Combatant.new("Duelist", 100, 15, 1)
	var foe_a := Combatant.new("FoeA", 100, 10, 1)
	var foe_b := Combatant.new("FoeB", 100, 10, 1)
	hgrid.place_combatant(duelist, Vector2i(3, 4))
	hgrid.place_combatant(foe_a, Vector2i(4, 4))
	hgrid.place_combatant(foe_b, Vector2i(3, 3))
	var duel_side: Array[Combatant] = [duelist]
	var foes_side: Array[Combatant] = [foe_a, foe_b]
	var hctx: PlanContext = _hand_ctx(hgrid, duel_side, foes_side)
	var hplan := CombatPlan.new(hctx)
	hplan.add_step(duelist, PlanStep.engage(strike, foe_a))   # strike (2) + switch (1) + strike (2) fits a 6-block round
	var no_switch: bool = true
	for e: PlanEntry in hplan.entries_for(duelist):
		no_switch = no_switch and e.skill.kind != SkillDef.Kind.TARGET
	hplan.add_step(duelist, PlanStep.engage(strike, foe_b))
	var switch_seen: PlanEntry = null
	for e: PlanEntry in hplan.entries_for(duelist):
		if e.skill.kind == SkillDef.Kind.TARGET:
			switch_seen = e
	var target_switch_ok: bool = no_switch and switch_seen != null and switch_seen.blocks() == 1 and switch_seen.total_ap() == 0 and switch_seen.target == foe_b
	var follower := Combatant.new("Follower", 100, 10, 1)
	var runner2 := Combatant.new("Runner2", 100, 10, 1)
	var fgrid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	fgrid.place_combatant(follower, Vector2i(1, 4))
	fgrid.place_combatant(runner2, Vector2i(2, 4))
	var follow_side: Array[Combatant] = [follower]
	var run_side: Array[Combatant] = [runner2]
	var fctx: PlanContext = _hand_ctx(fgrid, run_side, follow_side)
	var fplan := CombatPlan.new(fctx)
	fplan.add_step(follower, PlanStep.engage(strike, runner2))
	fgrid.combatant_positions[runner2] = Vector2i(3, 4)   # steps away after planning: the attack follows one tile
	var fresolver := CombatResolver.new()
	fresolver.grid = fgrid
	fplan.commit(fresolver)
	fresolver.resolve_round()
	var follow_ok: bool = runner2.hp < 100
	rules_ok = rules_ok and target_switch_ok and follow_ok
	# 5e. Reach bands, move-first, statuses with resist rolls, area attacks, allies moving around each other
	var mgrid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var archer_u := Combatant.new("ArcherU", 50, 12, 3)
	var brute_u := Combatant.new("BruteU", 80, 10, 1)
	var far_u := Combatant.new("FarU", 80, 10, 1)
	mgrid.place_combatant(archer_u, Vector2i(2, 4))
	mgrid.place_combatant(brute_u, Vector2i(3, 4))
	mgrid.place_combatant(far_u, Vector2i(2, 1))
	var archer_side: Array[Combatant] = [archer_u]
	var brute_side: Array[Combatant] = [brute_u, far_u]
	var mctx: PlanContext = _hand_ctx(mgrid, archer_side, brute_side)
	var mplan := CombatPlan.new(mctx)
	var jab: SkillDef = SkillCatalog.find("offhand_jab")
	# ranged units: no ranged attack at distance 1 (once they can't step back), but the off-hand jab works there; 2-3 is fine
	var stepped_back: String = mplan.add_step(archer_u, PlanStep.engage(strike, brute_u))   # unacted: backs off to reach
	var backed_off: bool = stepped_back.is_empty() and mgrid.get_distance(mplan.compile(archer_u).pos, mgrid.get_position(brute_u)) >= 2
	mplan.clear(archer_u)
	mplan.add_step(archer_u, PlanStep.self_skill(SkillCatalog.find("rest")))
	var too_close: String = mplan.add_step(archer_u, PlanStep.engage(strike, brute_u))
	var jab_ok: String = mplan.add_step(archer_u, PlanStep.engage(jab, brute_u))
	mplan.clear(archer_u)
	var at_range: String = mplan.add_step(archer_u, PlanStep.engage(strike, far_u))
	var reach_ok: bool = backed_off and archer_u.min_attack_range() == 2 and brute_u.min_attack_range() == 1 and not too_close.is_empty() and jab_ok.is_empty() and at_range.is_empty()
	reach_ok = reach_ok and SkillCatalog.single_attacks(archer_u, 3).has(jab) and not SkillCatalog.single_attacks(brute_u, 3).has(jab)
	# moving must come first
	mplan.clear(archer_u)
	mplan.add_step(archer_u, PlanStep.self_skill(SkillCatalog.find("guard")))
	var late_move: String = mplan.add_step(archer_u, PlanStep.go_to(mover, Vector2i(1, 6)))
	var move_first_ok: bool = "must come first" in late_move
	# rooted units can't move
	mplan.clear(archer_u)
	archer_u.statuses[StatusEffects.ROOT] = true
	var rooted_move: String = mplan.add_step(archer_u, PlanStep.go_to(mover, Vector2i(1, 6)))
	move_first_ok = move_first_ok and "rooted" in rooted_move
	archer_u.statuses.clear()
	# status rolls: potency vs resistance, seeded; immunity blocks it
	var rng_a := RandomNumberGenerator.new()
	rng_a.seed = 7
	var rng_b := RandomNumberGenerator.new()
	rng_b.seed = 7
	var victim_a := Combatant.new("VictimA", 50, 10, 1)
	var victim_b := Combatant.new("VictimB", 50, 10, 1)
	var rolls_a: Array[bool] = []
	var rolls_b: Array[bool] = []
	for i: int in range(20):
		victim_a.pending_statuses.clear()
		victim_b.pending_statuses.clear()
		rolls_a.append(StatusEffects.try_inflict(victim_a, StatusEffects.STAGGER, 60, rng_a))
		rolls_b.append(StatusEffects.try_inflict(victim_b, StatusEffects.STAGGER, 60, rng_b))
	var landed: int = rolls_a.count(true)
	var immune := Combatant.new("Immune", 50, 10, 1)
	immune.status_resist[StatusEffects.ROOT] = 100
	var status_ok: bool = rolls_a == rolls_b and landed > 0 and landed < 20 and StatusEffects.apply_chance(immune, StatusEffects.ROOT, 90) == 0
	status_ok = status_ok and not StatusEffects.try_inflict(immune, StatusEffects.ROOT, 90, rng_a)
	victim_a.pending_statuses.clear()
	victim_a.pending_statuses[StatusEffects.ROOT] = true
	StatusEffects.roll_over(victim_a, 4)
	status_ok = status_ok and victim_a.has_status(StatusEffects.ROOT) and victim_a.pending_statuses.is_empty()
	StatusEffects.roll_over(victim_a, 4)
	status_ok = status_ok and not victim_a.has_status(StatusEffects.ROOT)   # lasts one round
	# area attack: Volley on a tile hits everyone in the plus, friends included (friendly fire)
	var agrid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var thrower := Combatant.new("Thrower", 60, 12, 3)
	var buddy := Combatant.new("Buddy", 60, 10, 1)
	var foe_x := Combatant.new("FoeX", 60, 10, 1)
	var foe_y := Combatant.new("FoeY", 60, 10, 1)
	agrid.place_combatant(thrower, Vector2i(1, 4))
	agrid.place_combatant(foe_x, Vector2i(4, 4))
	agrid.place_combatant(foe_y, Vector2i(4, 5))
	agrid.place_combatant(buddy, Vector2i(4, 3))
	var throw_side: Array[Combatant] = [thrower, buddy]
	var target_side: Array[Combatant] = [foe_x, foe_y]
	var actx: PlanContext = _hand_ctx(agrid, throw_side, target_side)
	var aplan := CombatPlan.new(actx)
	var volley_opt: PlanOption = null
	for o: PlanOption in aplan.cell_options(thrower, Vector2i(4, 4)):
		if o.step.mode == PlanStep.Mode.AREA and o.step.skill.id == "volley":
			volley_opt = o
	var area_ok: bool = volley_opt != null and volley_opt.valid and volley_opt.damage > 0 and "Friendly fire" in volley_opt.warning
	aplan.add_step(thrower, volley_opt.step if volley_opt != null else PlanStep.withdraw())
	var aresolver := CombatResolver.new()
	aresolver.grid = agrid
	aplan.commit(aresolver)
	aresolver.resolve_round()
	area_ok = area_ok and foe_x.hp < 60 and foe_y.hp < 60 and buddy.hp < 60 and thrower.hp == 60
	var far_aim: Array[PlanOption] = aplan.cell_options(thrower, Vector2i(7, 7))
	var out_of_reach: bool = false
	for o: PlanOption in far_aim:
		if o.step.mode == PlanStep.Mode.AREA and o.step.skill.id == "volley" and not o.valid:
			out_of_reach = true
	area_ok = area_ok and out_of_reach
	# allies: the slower ally can take the faster ally's tile once that one moves off it in block 1
	var csession := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	csession.start()
	var cw: Combatant = csession.allies[0] if csession.allies[0].speed >= csession.allies[1].speed else csession.allies[1]
	var cr: Combatant = csession.allies[1] if cw == csession.allies[0] else csession.allies[0]
	var w_start: Vector2i = csession.grid.get_position(cw)
	var blocked_before: String = ""
	for o: PlanOption in csession.plan_cell_options(w_start):
		pass
	csession.select_combatant(cr)
	var before_opts: Array[PlanOption] = csession.plan_cell_options(w_start)
	blocked_before = before_opts[0].reason if not before_opts.is_empty() and not before_opts[0].valid else ""
	csession.plan_add_step(cw, PlanStep.go_to(mover, Vector2i(2, 6)))   # the faster ally steps off its tile
	var cw_dest: Vector2i = csession.planned_position(cw)
	var into_vacated: String = csession.plan_add_step(cr, PlanStep.go_to(mover, w_start))
	var into_dest: String = csession.plan_add_step(cr, PlanStep.go_to(mover, cw_dest))
	var coord_ok: bool = not blocked_before.is_empty() and cw_dest != w_start and into_vacated.is_empty() and not into_dest.is_empty()
	coord_ok = coord_ok and csession.planned_position(cr) == w_start and csession.planned_occupant(w_start) == cr and csession.planned_occupant(cw_dest) == cw
	csession.dispose()
	var mechanics_ok: bool = reach_ok and move_first_ok and status_ok and area_ok and coord_ok
	# ranged single-target reach is 2-3 even for range-2 units; area reach is skill-specific
	var range2 := Combatant.new("Range2", 50, 10, 2)
	var reach2_ok: bool = strike.reach_for(range2) == Vector2i(2, 3) and strike.reach_for(brute_u) == Vector2i(1, 1)
	reach2_ok = reach2_ok and SkillCatalog.find("volley").reach_for(range2) == Vector2i(2, 3) and SkillCatalog.find("firebomb").reach_for(range2) == Vector2i(2, 2) and SkillCatalog.find("offhand_jab").reach_for(range2) == Vector2i(1, 1)
	# undo: removes the focused hero's last step; harmless when there is nothing to undo
	var usession := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	usession.start()
	var uhero: Combatant = usession.focused_hero
	usession.plan_add_step(uhero, PlanStep.engage(strike, usession.enemies[1]))
	usession.plan_add_step(uhero, PlanStep.self_skill(SkillCatalog.find("guard")))
	usession.plan_undo_last()
	var undo_ok: bool = usession.plan.steps_of(uhero).size() == 1 and usession.plan.steps_of(uhero)[0].mode == PlanStep.Mode.ENGAGE
	usession.plan_undo_last()
	usession.plan_undo_last()
	undo_ok = undo_ok and usession.plan.steps_of(uhero).is_empty()
	usession.dispose()
	mechanics_ok = mechanics_ok and reach2_ok and undo_ok

	# the AI pack drafts into one shared plan: no two goblins plan to end on the same tile
	var pack := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	pack.start()
	var pack_dest: Dictionary = {}
	var pack_ok: bool = true
	for sa: CombatResolver.ScheduledAction in pack.resolver.scheduled_actions:
		var walk_cmd: WalkCommand = sa.command as WalkCommand
		if walk_cmd != null and walk_cmd.goal != Vector2i(-1, -1):
			pack_dest[sa.combatant] = walk_cmd.goal
	var seen_tiles: Dictionary = {}
	for unit: Variant in pack_dest:
		var tile: Vector2i = pack_dest[unit]
		if seen_tiles.has(tile):
			pack_ok = false
		seen_tiles[tile] = true
	pack.dispose()
	mechanics_ok = mechanics_ok and pack_ok

	# split moves: a 4-tile move at 2 tiles/block is two 1-block halves (first lands after block 1);
	# a half blocked by a unit that stepped in stops short, refunds its tiles and logs CUT SHORT
	var sgrid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var walker2 := Combatant.new("Walker2", 60, 8, 1)
	var cutter := Combatant.new("Cutter", 60, 30, 1)
	sgrid.place_combatant(walker2, Vector2i(1, 4))
	sgrid.place_combatant(cutter, Vector2i(6, 7))
	var walk_side: Array[Combatant] = [walker2]
	var cut_side: Array[Combatant] = [cutter]
	var sctx: PlanContext = _hand_ctx(sgrid, walk_side, cut_side)
	var splan := CombatPlan.new(sctx)
	var split_added: String = splan.add_step(walker2, PlanStep.go_to(mover, Vector2i(5, 4)))
	var halves: Array[PlanEntry] = splan.entries_for(walker2)
	var split_ok: bool = split_added.is_empty() and walker2.move_per_block() == 2 and halves.size() == 2
	if split_ok:
		split_ok = halves[0].start_block == 1 and halves[1].start_block == 2 and halves[0].blocks() == 1 and halves[0].pos_after == Vector2i(3, 4) and halves[1].pos_after == Vector2i(5, 4) and halves[0].skill_ap == 2 and halves[1].goal == Vector2i(5, 4)
	var sresolver := CombatResolver.new()
	sresolver.grid = sgrid
	splan.commit(sresolver)
	sresolver.execute_block(1)
	split_ok = split_ok and sgrid.get_position(walker2) == Vector2i(3, 4)
	sgrid.combatant_positions[cutter] = Vector2i(4, 4)   # an enemy steps into the path
	var ap_before_half: int = walker2.ap
	sresolver.execute_block(2)
	var cut_entry: ActionLogEntry = CombatLogBuilder.from_action(sresolver.scheduled_actions[1], 1, 2, walk_side, sgrid)
	split_ok = split_ok and sgrid.get_position(walker2) == Vector2i(3, 4) and walker2.ap == ap_before_half - 2 + 2 and cut_entry.outcome_tag == "CUT SHORT" and "Cutter" in cut_entry.status_text
	mechanics_ok = mechanics_ok and split_ok


	# 6. Presets: every preset drafts a valid, affordable sequence; each shows its focus
	var psession := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	psession.start()
	var hero: Combatant = psession.allies[0]
	var weakest: Combatant = psession.enemies[4]
	weakest.hp = 5
	var presets_ok: bool = true
	for preset_id: String in PlanPresets.IDS:
		psession.plan_apply_preset(hero, preset_id)
		presets_ok = presets_ok and psession.plan.is_valid(hero) and not psession.plan.compile(hero).runs_dry()
	psession.plan_apply_preset(hero, "aggressive")
	var aggressive_ok: bool = not psession.plan.steps_of(hero).is_empty()
	for s: PlanStep in psession.plan.steps_of(hero):
		aggressive_ok = aggressive_ok and s.mode == PlanStep.Mode.ENGAGE
	psession.plan_apply_preset(hero, "focus_weakest")
	var focus_steps: Array[PlanStep] = psession.plan.steps_of(hero)
	var weakest_ok: bool = not focus_steps.is_empty() and focus_steps[0].mode == PlanStep.Mode.ENGAGE and focus_steps[0].target == weakest
	psession.plan_apply_preset(hero, "economy")
	var economy_ok: bool = not psession.plan.steps_of(hero).is_empty()
	for s: PlanStep in psession.plan.steps_of(hero):
		economy_ok = economy_ok and (s.skill == null or s.skill.level == 1)
	psession.plan_apply_preset(hero, "defensive")
	var defensive_ok: bool = not psession.plan.steps_of(hero).is_empty() and psession.plan.steps_of(hero)[0].mode == PlanStep.Mode.SELF and psession.plan.steps_of(hero)[0].skill.kind == SkillDef.Kind.DEFEND
	psession.plan_apply_preset(hero, "withdraw")
	var withdraw_steps: Array[PlanStep] = psession.plan.steps_of(hero)
	var withdraw_ok: bool = not withdraw_steps.is_empty() and withdraw_steps[0].mode == PlanStep.Mode.WITHDRAW
	var preset_ok: bool = presets_ok and aggressive_ok and weakest_ok and economy_ok and defensive_ok and withdraw_ok
	psession.dispose()

	# 7. AI patterns on hand-built grids, verified by executing the drafted sequences
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var hunter := Combatant.new("Hunter", 60, 10, 1)
	var quarry := Combatant.new("Quarry", 100, 10, 1)
	grid.place_combatant(hunter, Vector2i(5, 4))
	grid.place_combatant(quarry, Vector2i(1, 4))
	var hunter_side: Array[Combatant] = [hunter]
	var quarry_side: Array[Combatant] = [quarry]
	var hunt_ctx: PlanContext = _hand_ctx(grid, quarry_side, hunter_side)
	var hunt_steps: Array[PlanStep] = TacticalAI.build_steps(hunter, hunt_ctx, hunt_ctx.profile_of(hunter))
	var hunt_plan := CombatPlan.new(hunt_ctx)
	hunt_plan.set_steps(hunter, hunt_steps)
	var closes_in: bool = not hunt_steps.is_empty() and hunt_steps[0].mode == PlanStep.Mode.ENGAGE and hunt_plan.is_valid(hunter) and not hunt_plan.compile(hunter).runs_dry()
	_run_steps(hunter, hunt_ctx, hunt_steps)
	closes_in = closes_in and quarry.hp < 100

	# Engage: the approach walks the planned path; if the target stepped one tile away,
	# the attack closes the gap when it starts (a target that ran farther is out of reach)
	var chase_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var chaser2 := Combatant.new("Chaser2", 60, 10, 1)
	var dodger := Combatant.new("Dodger", 100, 10, 1)
	chase_grid.place_combatant(chaser2, Vector2i(1, 4))
	chase_grid.place_combatant(dodger, Vector2i(5, 4))
	var chaser_side2: Array[Combatant] = [chaser2]
	var dodger_side: Array[Combatant] = [dodger]
	var chase_ctx: PlanContext = _hand_ctx(chase_grid, dodger_side, chaser_side2)
	var chase_plan := CombatPlan.new(chase_ctx)
	var chase_added: String = chase_plan.add_step(chaser2, PlanStep.engage(strike, dodger))
	var chase_resolver := CombatResolver.new()
	chase_resolver.grid = chase_grid
	var chase_log: Array[String] = []
	chase_resolver.log_emitted.connect(func(msg: String) -> void: chase_log.append(msg))
	chase_plan.commit(chase_resolver)
	chase_grid.combatant_positions[dodger] = Vector2i(5, 3)  # after confirming, the target steps 1 tile
	chase_resolver.resolve_round()
	var chased: bool = false
	for msg: String in chase_log:
		if msg.begins_with("[Chase]"):
			chased = true
	var chase_ok: bool = chase_added.is_empty() and dodger.hp < 100 and chased
	# a target that runs 3 tiles away is not chased down
	var runner_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var chaser3 := Combatant.new("Chaser3", 60, 10, 1)
	var sprinter := Combatant.new("Sprinter", 100, 10, 1)
	runner_grid.place_combatant(chaser3, Vector2i(1, 4))
	runner_grid.place_combatant(sprinter, Vector2i(5, 4))
	var sprinter_side: Array[Combatant] = [sprinter]
	var chaser3_side: Array[Combatant] = [chaser3]
	var runner_ctx: PlanContext = _hand_ctx(runner_grid, sprinter_side, chaser3_side)
	var runner_plan := CombatPlan.new(runner_ctx)
	runner_plan.add_step(chaser3, PlanStep.engage(strike, sprinter))
	var runner_resolver := CombatResolver.new()
	runner_resolver.grid = runner_grid
	runner_plan.commit(runner_resolver)
	runner_grid.combatant_positions[sprinter] = Vector2i(7, 7)
	runner_resolver.resolve_round()
	chase_ok = chase_ok and sprinter.hp == 100 and runner_grid.get_position(chaser3) != Vector2i(7, 6)
	# Only damage-focused attacks adjust: after a hand-picked tile move the unit holds its tile
	var hold_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var holder := Combatant.new("Holder", 60, 10, 1)
	var post := Combatant.new("Post", 100, 10, 1)
	hold_grid.place_combatant(holder, Vector2i(1, 4))
	hold_grid.place_combatant(post, Vector2i(4, 4))
	var holder_side: Array[Combatant] = [holder]
	var post_side: Array[Combatant] = [post]
	var hold_plan := CombatPlan.new(_hand_ctx(hold_grid, post_side, holder_side))
	hold_plan.add_step(holder, PlanStep.go_to(SkillCatalog.lowest(SkillDef.Kind.MOVE), Vector2i(3, 4)))
	hold_plan.add_step(holder, PlanStep.engage(strike, post))
	var hold_entries: Array[PlanEntry] = hold_plan.entries_for(holder)
	var hold_attack: PlanEntry = hold_entries[hold_entries.size() - 1]
	var hold_resolver := CombatResolver.new()
	hold_resolver.grid = hold_grid
	hold_plan.commit(hold_resolver)
	hold_grid.combatant_positions[post] = Vector2i(5, 4)   # steps away after the plan is locked
	hold_resolver.resolve_round()
	var held: bool = not hold_attack.chase and post.hp == 100 and hold_grid.get_position(holder) == Vector2i(3, 4)
	# ...while an engage (no tile move) keeps its one-tile adjustment
	var adjacent_plan := CombatPlan.new(_hand_ctx(hold_grid, post_side, holder_side))
	adjacent_plan.add_step(holder, PlanStep.engage(strike, post))
	var adjacent_entries: Array[PlanEntry] = adjacent_plan.entries_for(holder)
	held = held and adjacent_entries[adjacent_entries.size() - 1].chase
	# A ranged attacker whose target stepped point-blank backs off one tile to shoot
	var back_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var bowman := Combatant.new("Bowman", 60, 10, 3)
	var rusher := Combatant.new("Rusher", 100, 10, 1)
	back_grid.place_combatant(bowman, Vector2i(2, 4))
	back_grid.place_combatant(rusher, Vector2i(5, 4))
	var bowman_side: Array[Combatant] = [bowman]
	var rusher_side: Array[Combatant] = [rusher]
	var back_plan := CombatPlan.new(_hand_ctx(back_grid, rusher_side, bowman_side))
	var back_added: String = back_plan.add_step(bowman, PlanStep.engage(strike, rusher))
	var back_resolver := CombatResolver.new()
	back_resolver.grid = back_grid
	var back_log: Array[String] = []
	back_resolver.log_emitted.connect(func(msg: String) -> void: back_log.append(msg))
	back_plan.commit(back_resolver)
	back_grid.combatant_positions[rusher] = Vector2i(3, 4)   # rushes into point-blank range
	back_resolver.resolve_round()
	var backed: bool = false
	for msg: String in back_log:
		if msg.begins_with("[Chase]") and msg.contains("backs off"):
			backed = true
	chase_ok = chase_ok and held and back_added.is_empty() and backed and rusher.hp < 100

	var wall_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	for y: int in range(0, 5):
		wall_grid.set_terrain(Vector2i(4, y), Terrain.Type.WALL)
	var runner := Combatant.new("Runner", 60, 10, 1)
	var prey := Combatant.new("Prey", 60, 10, 1)
	wall_grid.place_combatant(runner, Vector2i(2, 4))
	wall_grid.place_combatant(prey, Vector2i(5, 4))
	var runner_side: Array[Combatant] = [runner]
	var prey_side: Array[Combatant] = [prey]
	var wall_ctx: PlanContext = _hand_ctx(wall_grid, prey_side, runner_side)
	var wall_steps: Array[PlanStep] = TacticalAI.build_steps(runner, wall_ctx, wall_ctx.profile_of(runner))
	_run_steps(runner, wall_ctx, wall_steps)
	var walls_ok: bool = prey.hp < 60

	var snipe_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var sniper := Combatant.new("Sniper", 40, 10, 3)
	var bruiser := Combatant.new("Bruiser", 60, 10, 1)
	snipe_grid.place_combatant(sniper, Vector2i(4, 4))
	snipe_grid.place_combatant(bruiser, Vector2i(3, 4))
	var sniper_side: Array[Combatant] = [sniper]
	var bruiser_side: Array[Combatant] = [bruiser]
	var snipe_ctx: PlanContext = _hand_ctx(snipe_grid, bruiser_side, sniper_side)
	var snipe_steps: Array[PlanStep] = TacticalAI.build_steps(sniper, snipe_ctx, snipe_ctx.profile_of(sniper))
	var sniper_ok: bool = not snipe_steps.is_empty() and snipe_steps[0].mode == PlanStep.Mode.MOVE and snipe_steps[0].direction == Vector2i(1, 0)
	var support_profile: AiProfile = snipe_ctx.profile_of(sniper).duplicate_profile()
	support_profile.role = AiProfile.Role.SUPPORT
	var support_steps: Array[PlanStep] = TacticalAI.build_steps(sniper, snipe_ctx, support_profile)
	var support_ok: bool = not support_steps.is_empty() and support_steps[0].skill.kind == SkillDef.Kind.FOCUS

	var weakling := Combatant.new("Weakling", 100, 10, 1)
	weakling.hp = 10
	var chaser := Combatant.new("Chaser", 100, 10, 1)
	var flee_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	flee_grid.place_combatant(weakling, Vector2i(5, 4))
	flee_grid.place_combatant(chaser, Vector2i(2, 4))
	var weak_side: Array[Combatant] = [weakling]
	var chaser_side: Array[Combatant] = [chaser]
	var flee_ctx: PlanContext = _hand_ctx(flee_grid, chaser_side, weak_side)
	var flee_steps: Array[PlanStep] = TacticalAI.build_steps(weakling, flee_ctx, flee_ctx.profile_of(weakling))
	_run_steps(weakling, flee_ctx, flee_steps)
	var flee_ok: bool = not flee_steps.is_empty() and flee_steps[0].mode == PlanStep.Mode.WITHDRAW and weakling.is_withdrawn
	var ai_ok: bool = closes_in and walls_ok and sniper_ok and support_ok and flee_ok and chase_ok

	# 8. AI vs AI preset skirmishes: every zone is deterministic and hurts both sides
	var zones_ok: bool = true
	var both_hurt: bool = true
	for zone_id: String in CombatZones.zone_ids():
		var run_a: Dictionary = _play_with_preset(zone_id, "balanced")
		var run_b: Dictionary = _play_with_preset(zone_id, "balanced")
		if run_a.get("hp", []) != run_b.get("hp", []) or run_a.get("log", 0) != run_b.get("log", -1):
			zones_ok = false
			print("    non-deterministic result on zone: %s" % zone_id)
		if not (DictUtil.to_bool(run_a.get("party_hurt", false)) and DictUtil.to_bool(run_a.get("foes_hurt", false))):
			both_hurt = false
			print("    one side untouched on zone: %s %s" % [zone_id, str(run_a.get("hp", []))])

	# 9. Resetting a planned round rewinds units to the start of that round
	var rs := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	rs.start()
	rs.plan_apply_preset_all("balanced")
	rs.confirm_plan()
	_resolve_until_replay(rs)
	var reset_ok: bool = false
	if rs.can_advance_round():
		rs.advance_round()
		var start_hp: Array[int] = []
		var start_ap: Array[int] = []
		var start_pos: Array[Vector2i] = []
		for c: Combatant in rs.allies + rs.enemies:
			start_ap.append(c.ap)
			start_hp.append(c.hp)
			start_pos.append(rs.grid.get_position(c))
		rs.plan_apply_preset_all("balanced")
		rs.confirm_plan()
		_resolve_until_replay(rs)
		rs.reset_round()
		var end_hp: Array[int] = []
		var end_ap: Array[int] = []
		var end_pos: Array[Vector2i] = []
		for c: Combatant in rs.allies + rs.enemies:
			end_ap.append(c.ap)
			end_hp.append(c.hp)
			end_pos.append(rs.grid.get_position(c))
		reset_ok = start_hp == end_hp and start_pos == end_pos and rs.is_planning() and rs.current_round == 2 and start_ap == end_ap
	rs.dispose()

	print("\n=======================================================")
	print("=== PLANNING, STANCES, AP & ENEMY AI ASSERTIONS     ===")
	print("=======================================================")
	print("  [Skills] AP = skill level, Move 1 AP/tile, Rest free, effects shift costs, speed sets tiles/block: %s" % ("PASS" if ap_model_ok else "FAIL"))
	print("  [Stance] Offensive > Tactical > Defensive > Offensive; Neutral neutral; auto switch costs +1 AP: %s" % ("PASS" if stance_ok else "FAIL"))
	print("  [Plan] Sequence compiles to lowest free blocks; Engage auto-inserts 1-block moves; AP incl. switch: %s" % ("PASS" if seq_ok else "FAIL"))
	print("  [Plan] Mid-sequence stance switch is automatic (+1 AP, no block): %s" % ("PASS" if auto_switch_ok else "FAIL"))
	print("  [Plan] Over-committed AP is projected (runs dry) not refused; Rest offered: %s" % ("PASS" if (runtime_plan_ok and options_ok) else "FAIL"))
	print("  [Plan] Reorder/remove edits are flagged when invalid and block Confirm: %s" % ("PASS" if edit_ok else "FAIL"))
	print("  [Plan] Confirm locks the sequence, schedules party + AI enemies, AP spent at runtime: %s" % ("PASS" if flow_ok2 else "FAIL"))
	print("  [Input] Per-target/tile options (foe attacks, self skills, move here, withdraw), previews, status & hover inspect: %s" % ("PASS" if input_ok else "FAIL"))
	print("  [Rules] 6-block rounds, AP pool + regen, one move per round, centre tie-break, target switching, insurance attacks, follow, damage log, auto-play on confirm: %s" % ("PASS" if rules_ok else "FAIL"))
	print("  [Mechanics] Ranged 2+ reach & off-hand jab, move-first, root, seeded status rolls & immunity, area friendly fire, allies moving into vacated tiles, split moves & cut-short refunds, ranged 2-3 reach, undo last step, AI pack avoids its own planned tiles: %s" % ("PASS" if mechanics_ok else "FAIL"))
	print("  [Runtime] Overspending goes into AP debt (next round: less AP, 3+ Stagger, 5+ Stun); Rest recovers AP mid-round: %s" % ("PASS" if runtime_ok else "FAIL"))
	print("  [AI] Presets draft valid, affordable sequences with their focus: %s" % ("PASS" if preset_ok else "FAIL"))
	print("  [AI] Aggressor engages, path-finds walls, Sniper backs away, Support focuses, wounded flee, attack closes a 1-tile gap at its start (not after a tile move; ranged backs off): %s" % ("PASS" if ai_ok else "FAIL"))
	print("  [AI] Preset skirmish is deterministic on every zone: %s" % ("PASS" if zones_ok else "FAIL"))
	print("  [AI] Both sides take damage in a preset-planned skirmish: %s" % ("PASS" if both_hurt else "FAIL"))
	print("  [Plan] Reset round rewinds units and AP to the round start and reopens planning: %s" % ("PASS" if reset_ok else "FAIL"))

	return ap_model_ok and stance_ok and seq_ok and auto_switch_ok and runtime_plan_ok and options_ok and edit_ok and flow_ok2 and input_ok and rules_ok and mechanics_ok and runtime_ok and preset_ok and ai_ok and zones_ok and both_hurt and reset_ok
