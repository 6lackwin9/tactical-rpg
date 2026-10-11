class_name ArenaTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Headless Integration & UI Test Suite
# Contains all automated headless tests:
# 1. UI Setup and Behaviour Test (CombatArenaController view framework)
# 2. Warrior vs 3 Goblins Battle (Stance, Multi-round focus buff, Damage mitigation)
# 3. Combat Grid & Withdrawal Test (9x9 grid, spatial range gate, exit withdrawal)
# 4. Resolution Feed & Action Logging Test (Event capture, structured log feed)
# 5. Goblin Ambush & Casualty Test (Reinforcements, casualty action cancellation)
# ==============================================================================

static func run_all_tests(tree_parent: Node = null) -> bool:
	var test0_passed: bool = run_combat_ui_framework_test(tree_parent)
	var test1_passed: bool = run_warrior_vs_3_goblins_battle()
	var test2_passed: bool = run_combat_grid_and_withdrawal_test()
	var test3_passed: bool = run_resolution_feed_logging_test()
	var test4_passed: bool = run_goblin_ambush_and_casualty_test()
	var test5_passed: bool = run_character_specs_and_inventory_test()

	return test0_passed and test1_passed and test2_passed and test3_passed and test4_passed and test5_passed


# ==============================================================================
# TEST 1: COMBAT UI FRAMEWORK & BEHAVIOUR TEST
# ==============================================================================

static func run_combat_ui_framework_test(tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING COMBAT UI FRAMEWORK & BEHAVIOUR TEST   ===")
	print("=======================================================")

	var arena := CombatArenaController.new()
	if tree_parent != null:
		tree_parent.add_child(arena)

	# 1. Verify core docks and child nodes are instantiated
	var ui_nodes_ok: bool = (
		arena.roster_panel != null and
		arena.action_log_dock != null and
		arena.unit_inspector != null and
		arena.grid_view != null and
		arena.tactical_forecast != null and
		arena.playback_controls != null and
		arena.timeline_dock != null and
		arena.log_modal != null and
		arena.timeline_modal != null
	)

	# 2. Scenario mounting & visual token generation
	var scenario := CombatScenario.create_default_arena_scenario()
	var exit_coord := Vector2i(8, 4)
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [exit_coord])
	grid.deploy_combatants(scenario.allies, scenario.enemies)
	arena.mount_scenario(scenario, grid)

	var ally0: Combatant = scenario.allies[0]
	var enemy0: Combatant = scenario.enemies[0]
	var tokens_created_ok: bool = (
		arena.grid_view.unit_token_nodes.has(ally0) and
		arena.grid_view.unit_token_nodes.has(enemy0) and
		arena.roster_panel.roster_cards.has(ally0) and
		arena.roster_panel.roster_cards.has(enemy0)
	)

	# 3. Combatant selection & Unit Inspector update
	arena.select_combatant(ally0, scenario.allies)
	arena.update_selected_unit_panel(ally0, true, "(4, 5)", scenario.get_sprite_texture(ally0))
	var inspector_sync_ok: bool = (
		arena.unit_inspector.inspector_name_lbl != null and
		arena.unit_inspector.inspector_name_lbl.text == ally0.name
	)

	# 4. Tactical Forecast update
	arena.update_tactical_forecast_panel(ally0, enemy0, grid)
	var forecast_ok: bool = (
		arena.tactical_forecast.forecast_target_lbl != null and
		enemy0.name in arena.tactical_forecast.forecast_target_lbl.text
	)

	# 5. Action Log Dock & Log Modal toggle lifecycle
	var test_entry := ActionLogEntry.new(1, 2, "ATTACK", ally0.name, enemy0.name, "-25 HP", 25, "HIT")
	arena.update_active_log_cell([test_entry], 0)
	arena.toggle_log_modal(true, [test_entry])
	var modal_open_ok: bool = arena.log_modal.visible and arena.log_modal.all_entries.size() == 1
	arena.toggle_log_modal(false)
	var modal_close_ok: bool = not arena.log_modal.visible
	var log_modal_ok: bool = modal_open_ok and modal_close_ok

	# 6. Timeline Modal toggle lifecycle
	arena.toggle_timeline_modal(true)
	var timeline_open_ok: bool = arena.timeline_modal.visible
	arena.toggle_timeline_modal(false)
	var timeline_close_ok: bool = not arena.timeline_modal.visible
	var timeline_modal_ok: bool = timeline_open_ok and timeline_close_ok

	# 7. UI Signal propagation
	var received_block: Array[int] = [0]
	arena.block_selected.connect(func(b: int) -> void: received_block[0] = b)
	arena.timeline_dock.block_selected.emit(4)
	var signals_ok: bool = (received_block[0] == 4)

	# 8. Visual Action FX & Floating Combat Text dispatch
	var fx_ok: bool = true
	if tree_parent != null:
		arena.trigger_action_fx(test_entry, ally0, enemy0)
		arena.trigger_action_fx(ActionLogEntry.new(1, 1, "DEFEND", ally0.name, "", "50%", 50, "DEFEND"), ally0, null)
		arena.trigger_action_fx(ActionLogEntry.new(1, 1, "BUFF", ally0.name, "", "+50%", 50, "FOCUS"), ally0, null)
		arena.trigger_action_fx(ActionLogEntry.new(1, 1, "MOVE", ally0.name, "", "-> (4, 4)", 0, "MOVED", Vector2i(4, 5), Vector2i(4, 4)), ally0, null)
		arena.trigger_action_fx(ActionLogEntry.new(1, 3, "EVENT", enemy0.name, "", "WITHDREW", 0, "WITHDRAWN"), enemy0, null)

	print("\n=======================================================")
	print("=== COMBAT UI FRAMEWORK TEST ASSERTIONS             ===")
	print("=======================================================")
	print("  [UI] Core dock & controller widgets created: %s" % ("PASS" if ui_nodes_ok else "FAIL"))
	print("  [UI] Scenario mounted unit tokens & roster cards: %s" % ("PASS" if tokens_created_ok else "FAIL"))
	print("  [UI] Combatant selection & Unit Inspector synced: %s" % ("PASS" if inspector_sync_ok else "FAIL"))
	print("  [UI] Tactical forecast computed & rendered: %s" % ("PASS" if forecast_ok else "FAIL"))
	print("  [UI] Action Log Modal toggle & entry view lifecycle: %s" % ("PASS" if log_modal_ok else "FAIL"))
	print("  [UI] Timeline Modal toggle lifecycle: %s" % ("PASS" if timeline_modal_ok else "FAIL"))
	print("  [UI] Timeline dock block selection signal dispatched: %s" % ("PASS" if signals_ok else "FAIL"))
	print("  [UI] Action FX and floating text triggered safely: %s" % ("PASS" if fx_ok else "FAIL"))

	# Cleanup
	if tree_parent != null:
		tree_parent.remove_child(arena)
	arena.queue_free()

	return (ui_nodes_ok and tokens_created_ok and inspector_sync_ok and forecast_ok and
		log_modal_ok and timeline_modal_ok and signals_ok and fx_ok)


# ==============================================================================
# TEST 2: WARRIOR VS 3 GOBLINS BATTLE
# ==============================================================================

static func run_warrior_vs_3_goblins_battle() -> bool:
	var resolver := CombatResolver.new()

	var warrior := Combatant.new("Warrior", 120, 15)
	var scout := Combatant.new("Goblin Scout", 35, 18)
	var brawler := Combatant.new("Goblin Brawler", 45, 12)
	var skirmisher := Combatant.new("Goblin Skirmisher", 40, 16)

	resolver.register_combatant(warrior)
	resolver.register_combatant(scout)
	resolver.register_combatant(brawler)
	resolver.register_combatant(skirmisher)

	# Round 1 Scheduling
	resolver.schedule_action(warrior, TargetCommand.new(warrior, scout), 1)
	resolver.schedule_action(warrior, FocusCommand.new(warrior, 1.5, 2), 2)
	resolver.schedule_action(warrior, EnterNeutralCommand.new(warrior), 4)
	resolver.schedule_action(warrior, AttackCommand.new(warrior, scout, 30), 5)
	resolver.schedule_action(warrior, EnterNeutralCommand.new(warrior), 7)
	resolver.schedule_action(warrior, DefendCommand.new(warrior, 0.5), 8)
	resolver.schedule_action(warrior, HoldStanceCommand.new(warrior), 10)

	resolver.schedule_action(scout, DefendCommand.new(scout, 0.5), 1)
	resolver.schedule_action(scout, AttackCommand.new(scout, warrior, 12), 3)
	resolver.schedule_action(scout, AttackCommand.new(scout, warrior, 12), 5)

	resolver.schedule_action(brawler, DefendCommand.new(brawler, 0.5), 1)
	resolver.schedule_action(brawler, DefendCommand.new(brawler, 0.5), 3)
	resolver.schedule_action(brawler, DefendCommand.new(brawler, 0.5), 5)
	resolver.schedule_action(brawler, EnterNeutralCommand.new(brawler), 7)
	resolver.schedule_action(brawler, AttackCommand.new(brawler, warrior, 14), 8)
	resolver.schedule_action(brawler, HoldStanceCommand.new(brawler), 10)

	resolver.schedule_action(skirmisher, TargetCommand.new(skirmisher, warrior), 1)
	resolver.schedule_action(skirmisher, AttackCommand.new(skirmisher, warrior, 10), 2)
	resolver.schedule_action(skirmisher, EnterNeutralCommand.new(skirmisher), 4)
	resolver.schedule_action(skirmisher, DefendCommand.new(skirmisher, 0.5), 5)
	resolver.schedule_action(skirmisher, EnterNeutralCommand.new(skirmisher), 7)
	resolver.schedule_action(skirmisher, AttackCommand.new(skirmisher, warrior, 12), 8)
	resolver.schedule_action(skirmisher, HoldStanceCommand.new(skirmisher), 10)

	resolver.resolve_round()

	var warrior_hp_r1: int = warrior.hp
	var scout_defeated_r1: bool = scout.hp == 0
	var warrior_hp_r1_ok: bool = warrior_hp_r1 == 73
	var focus_buff_persisted: bool = warrior.attack_boost_multiplier == 1.5 and warrior.attack_boost_rounds_remaining == 1
	var stance_persisted: bool = warrior.current_stance == CombatTypes.StanceType.DEFENSIVE

	# Round 2 Scheduling
	resolver.schedule_action(warrior, EnterNeutralCommand.new(warrior), 1)
	resolver.schedule_action(warrior, TargetCommand.new(warrior, brawler), 2)
	resolver.schedule_action(warrior, AttackCommand.new(warrior, brawler, 30), 3)
	resolver.schedule_action(warrior, EnterNeutralCommand.new(warrior), 5)
	resolver.schedule_action(warrior, TargetCommand.new(warrior, skirmisher), 6)
	resolver.schedule_action(warrior, AttackCommand.new(warrior, skirmisher, 30), 7)
	resolver.schedule_action(warrior, DefendCommand.new(warrior, 0.5), 9)

	resolver.schedule_action(brawler, AttackCommand.new(brawler, warrior, 10), 1)
	resolver.schedule_action(skirmisher, DefendCommand.new(skirmisher, 0.5), 1)

	resolver.resolve_round()

	var warrior_hp_r2: int = warrior.hp
	var brawler_defeated: bool = brawler.hp == 0
	var skirmisher_defeated: bool = skirmisher.hp == 0
	var warrior_survived: bool = warrior.is_alive() and warrior_hp_r2 == 63
	var all_goblins_down: bool = scout_defeated_r1 and brawler_defeated and skirmisher_defeated
	var buff_expired_after_r2: bool = warrior.attack_boost_multiplier == 1.0 and warrior.attack_boost_rounds_remaining == 0

	print("\n==========================================")
	print("=== FINAL INTEGRATION TEST ASSERTIONS ===")
	print("==========================================")
	print("  [R1] Goblin Scout defeated (0 HP): %s" % ("PASS" if scout_defeated_r1 else "FAIL"))
	print("  [R1] Warrior mitigated damage (HP == 73): %s (HP: %d)" % [("PASS" if warrior_hp_r1_ok else "FAIL"), warrior_hp_r1])
	print("  [R1] Defensive Stance carried to R2: %s" % ("PASS" if stance_persisted else "FAIL"))
	print("  [R1] Focus 2-round buff active into R2: %s" % ("PASS" if focus_buff_persisted else "FAIL"))
	print("  [R2] Goblin Brawler defeated (0 HP): %s" % ("PASS" if brawler_defeated else "FAIL"))
	print("  [R2] Goblin Skirmisher defeated (0 HP): %s" % ("PASS" if skirmisher_defeated else "FAIL"))
	print("  [R2] Warrior survived battle (HP == 63): %s (HP: %d/120)" % [("PASS" if warrior_survived else "FAIL"), warrior_hp_r2])
	print("  [R2] All 3 Goblins eliminated: %s" % ("PASS" if all_goblins_down else "FAIL"))
	print("  [R2] Focus 2-round buff expired cleanly: %s" % ("PASS" if buff_expired_after_r2 else "FAIL"))

	warrior.clear_target()
	scout.clear_target()
	brawler.clear_target()
	skirmisher.clear_target()
	resolver.registered_combatants.clear()

	return (scout_defeated_r1 and warrior_hp_r1_ok and stance_persisted and
		focus_buff_persisted and brawler_defeated and skirmisher_defeated and
		warrior_survived and all_goblins_down and buff_expired_after_r2)


# ==============================================================================
# TEST 3: COMBAT GRID & WITHDRAWAL TEST (9x9)
# ==============================================================================

static func run_combat_grid_and_withdrawal_test() -> bool:
	print("\n=======================================================")
	print("=== STARTING COMBAT GRID & WITHDRAWAL TEST (9x9)    ===")
	print("=======================================================")

	var exit_coord := Vector2i(8, 4)
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [exit_coord])
	var resolver := CombatResolver.new()
	resolver.grid = grid

	var warrior := Combatant.new("Warrior", 100, 15, 1)
	var ranger := Combatant.new("Ranger", 80, 14, 2)
	var brawler := Combatant.new("Enemy Brawler", 45, 12, 1)
	var scout := Combatant.new("Fleeing Scout", 30, 16, 1)

	var allies: Array[Combatant] = [warrior, ranger]
	var enemies: Array[Combatant] = [brawler, scout]

	resolver.register_combatant(warrior)
	resolver.register_combatant(ranger)
	resolver.register_combatant(brawler)
	resolver.register_combatant(scout)

	grid.deploy_combatants(allies, enemies)

	var warrior_pos: Vector2i = grid.get_position(warrior)
	var ranger_pos: Vector2i = grid.get_position(ranger)
	var brawler_pos: Vector2i = grid.get_position(brawler)
	var scout_pos: Vector2i = grid.get_position(scout)

	var warrior_center_ok: bool = grid.get_distance(warrior_pos, CombatGrid.CENTER) <= warrior.attack_range
	var ranger_center_ok: bool = grid.get_distance(ranger_pos, CombatGrid.CENTER) <= ranger.attack_range
	var brawler_center_ok: bool = grid.get_distance(brawler_pos, CombatGrid.CENTER) <= brawler.attack_range
	var scout_center_ok: bool = grid.get_distance(scout_pos, CombatGrid.CENTER) <= scout.attack_range

	var deployment_ok: bool = warrior_center_ok and ranger_center_ok and brawler_center_ok and scout_center_ok
	var initial_out_of_range: bool = not grid.can_target(warrior, brawler, warrior.attack_range)

	resolver.schedule_action(warrior, MoveCommand.new(warrior, grid, Vector2i(0, -1), 1), 1)
	resolver.schedule_action(warrior, AttackCommand.new(warrior, brawler, 25, 2, 0, grid), 2)

	resolver.schedule_action(ranger, MoveCommand.new(ranger, grid, Vector2i(0, -1), 1), 4)
	resolver.schedule_action(ranger, AttackCommand.new(ranger, brawler, 25, 2, 0, grid), 5)

	resolver.schedule_action(scout, MoveCommand.new(scout, grid, Vector2i(1, 0), 1), 1)
	resolver.schedule_action(scout, MoveCommand.new(scout, grid, Vector2i(1, 0), 2, 2), 2)
	resolver.schedule_action(scout, AttackCommand.new(scout, warrior, 10, 2, 0, grid), 4)

	resolver.resolve_round()

	var scout_withdrawn_ok: bool = scout.is_withdrawn and not scout.is_active() and grid.withdrawn_combatants.has(scout)
	var scout_removed_from_grid: bool = not grid.has_combatant(scout)
	var brawler_defeated_ok: bool = brawler.hp == 0 and not brawler.is_active()
	var combat_ended_ok: bool = resolver.is_combat_resolved(allies, enemies)
	var warrior_pos_after: Vector2i = grid.get_position(warrior)
	var warrior_moved_ok: bool = warrior_pos_after == Vector2i(4, 4)

	print("\n================================================")
	print("=== COMBAT GRID & WITHDRAWAL TEST ASSERTIONS ===")
	print("================================================")
	print("  [Grid] Center-ring deployment valid: %s" % ("PASS" if deployment_ok else "FAIL"))
	print("  [Grid] Spatial range gate verified: %s" % ("PASS" if initial_out_of_range else "FAIL"))
	print("  [Grid] Warrior stepped to (4, 4): %s (%s)" % [("PASS" if warrior_moved_ok else "FAIL"), str(warrior_pos_after)])
	print("  [Grid] Fleeing Scout partially withdrawn at (8, 4): %s" % ("PASS" if scout_withdrawn_ok else "FAIL"))
	print("  [Grid] Scout removed from active grid: %s" % ("PASS" if scout_removed_from_grid else "FAIL"))
	print("  [Grid] Brawler eliminated by ranged team: %s" % ("PASS" if brawler_defeated_ok else "FAIL"))
	print("  [Grid] Combat terminated (all hostiles resolved): %s" % ("PASS" if combat_ended_ok else "FAIL"))

	warrior.clear_target()
	ranger.clear_target()
	brawler.clear_target()
	scout.clear_target()
	resolver.registered_combatants.clear()

	return (deployment_ok and initial_out_of_range and warrior_moved_ok and 
		scout_withdrawn_ok and scout_removed_from_grid and brawler_defeated_ok and combat_ended_ok)


# ==============================================================================
# TEST 4: RESOLUTION FEED & ACTION LOGGING TEST
# ==============================================================================

static func run_resolution_feed_logging_test() -> bool:
	print("\n=======================================================")
	print("=== STARTING RESOLUTION FEED & ACTION LOGGING TEST  ===")
	print("=======================================================")
	var captured_logs: Array[String] = []

	var exit_coord := Vector2i(8, 4)
	var test_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [exit_coord])
	var test_resolver := CombatResolver.new()
	test_resolver.grid = test_grid
	test_resolver.log_emitted.connect(func(msg: String) -> void:
		captured_logs.append(msg)
	)

	var warrior := Combatant.new("Warrior", 100, 15, 1)
	var ranger := Combatant.new("Ranger", 80, 14, 2)
	var brawler := Combatant.new("Enemy Brawler", 45, 12, 1)
	var scout := Combatant.new("Fleeing Scout", 30, 16, 1)

	var allies: Array[Combatant] = [warrior, ranger]
	var enemies: Array[Combatant] = [brawler, scout]

	test_resolver.register_combatant(warrior)
	test_resolver.register_combatant(ranger)
	test_resolver.register_combatant(brawler)
	test_resolver.register_combatant(scout)

	test_grid.deploy_combatants(allies, enemies)

	test_resolver.schedule_action(warrior, MoveCommand.new(warrior, test_grid, Vector2i(0, -1), 1), 1)
	test_resolver.schedule_action(warrior, AttackCommand.new(warrior, brawler, 25, 2, 0, test_grid), 2)
	test_resolver.schedule_action(ranger, FocusCommand.new(ranger, 1.5, 2, 2, 0), 1)
	test_resolver.schedule_action(ranger, MoveCommand.new(ranger, test_grid, Vector2i(0, -1), 1), 4)
	test_resolver.schedule_action(ranger, AttackCommand.new(ranger, brawler, 25, 2, 0, test_grid), 5)
	test_resolver.schedule_action(brawler, TargetCommand.new(brawler, warrior, 2), 1)
	test_resolver.schedule_action(brawler, DefendCommand.new(brawler, 0.5, 2, 1), 2)
	test_resolver.schedule_action(brawler, AttackCommand.new(brawler, warrior, 20, 2, 0, test_grid), 4)
	test_resolver.schedule_action(scout, MoveCommand.new(scout, test_grid, Vector2i(1, 0), 1), 1)
	test_resolver.schedule_action(scout, MoveCommand.new(scout, test_grid, Vector2i(1, 0), 2, 2), 2)

	for b: int in range(1, 7):
		test_resolver.execute_block(b)

	var has_move_log: bool = false
	var has_skill_log: bool = false
	var has_defend_log: bool = false
	var has_damage_log: bool = false
	var has_withdrawal_log: bool = false
	var has_defeat_log: bool = false

	for log_msg in captured_logs:
		if "[Move]" in log_msg and "moved from" in log_msg:
			has_move_log = true
		if "[Skill: Focus]" in log_msg:
			has_skill_log = true
		if "[Defend]" in log_msg and "Defensive Stance" in log_msg:
			has_defend_log = true
		if "[Attack]" in log_msg and "damage" in log_msg:
			has_damage_log = true
		if "[Withdrawal]" in log_msg and "WITHDREW" in log_msg:
			has_withdrawal_log = true
		if "DEFEATED" in log_msg:
			has_defeat_log = true

	var battle_resolved: bool = test_resolver.is_combat_resolved(allies, enemies)

	print("\n=======================================================")
	print("=== RESOLUTION FEED LOGGING TEST ASSERTIONS        ===")
	print("=======================================================")
	print("  [Feed] Movement coordinates logged: %s" % ("PASS" if has_move_log else "FAIL"))
	print("  [Feed] Focus skill usage logged: %s" % ("PASS" if has_skill_log else "FAIL"))
	print("  [Feed] Defense stance mitigation logged: %s" % ("PASS" if has_defend_log else "FAIL"))
	print("  [Feed] Attack damage and target HP logged: %s" % ("PASS" if has_damage_log else "FAIL"))
	print("  [Feed] Exit tile withdrawal logged: %s" % ("PASS" if has_withdrawal_log else "FAIL"))
	print("  [Feed] Enemy defeat logged: %s" % ("PASS" if has_defeat_log else "FAIL"))
	print("  [Feed] Combat resolution state reached: %s" % ("PASS" if battle_resolved else "FAIL"))

	warrior.clear_target()
	ranger.clear_target()
	brawler.clear_target()
	scout.clear_target()
	test_resolver.registered_combatants.clear()

	return (has_move_log and has_skill_log and has_defend_log and 
		has_damage_log and has_withdrawal_log and has_defeat_log and battle_resolved)


# ==============================================================================
# TEST 5: GOBLIN AMBUSH & CASUALTY TEST (MULTI-ROUND)
# ==============================================================================

static func run_goblin_ambush_and_casualty_test() -> bool:
	print("\n=======================================================")
	print("=== STARTING GOBLIN AMBUSH & CASUALTY TEST (MULTI-RD) ===")
	print("=======================================================")

	var test_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var test_resolver := CombatResolver.new()
	test_resolver.grid = test_grid

	var warrior := Combatant.new("Warrior", 100, 15, 1)
	var ranger := Combatant.new("Ranger", 80, 14, 2)
	var brawler := Combatant.new("Enemy Brawler", 45, 12, 1)
	var scout := Combatant.new("Fleeing Scout", 30, 16, 1)

	var skirmisher := Combatant.new("Goblin Skirmisher", 40, 15, 1)
	var archer := Combatant.new("Goblin Archer", 35, 13, 3)
	var berserker := Combatant.new("Goblin Berserker", 55, 11, 1)
	var shaman := Combatant.new("Goblin Shaman", 35, 12, 2)

	var allies: Array[Combatant] = [warrior, ranger]
	var initial_enemies: Array[Combatant] = [brawler, scout]
	var all_enemies: Array[Combatant] = [brawler, scout, skirmisher, archer, berserker, shaman]

	for c: Combatant in [warrior, ranger, brawler, scout, skirmisher, archer, berserker, shaman]:
		test_resolver.register_combatant(c)

	test_grid.deploy_combatants(allies, initial_enemies)

	var init_placement_ok: bool = (test_grid.has_combatant(warrior) and test_grid.has_combatant(ranger) and
		test_grid.has_combatant(brawler) and test_grid.has_combatant(scout) and
		not test_grid.has_combatant(skirmisher) and not test_grid.has_combatant(archer))

	# Schedule Round 1 actions:
	test_resolver.schedule_action(warrior, MoveCommand.new(warrior, test_grid, Vector2i(0, -1), 1), 1)
	test_resolver.schedule_action(warrior, AttackCommand.new(warrior, brawler, 25, 2, 0, test_grid), 2)
	test_resolver.schedule_action(warrior, TargetCommand.new(warrior, skirmisher, 1), 4)
	test_resolver.schedule_action(warrior, AttackCommand.new(warrior, skirmisher, 25, 2, 0, test_grid), 5)
	test_resolver.schedule_action(warrior, DefendCommand.new(warrior, 0.5, 2, 1), 7)
	test_resolver.schedule_action(warrior, HoldStanceCommand.new(warrior, 0), 9)

	test_resolver.schedule_action(ranger, FocusCommand.new(ranger, 1.5, 2, 2, 0), 1)
	test_resolver.schedule_action(ranger, MoveCommand.new(ranger, test_grid, Vector2i(0, -1), 1), 4)
	test_resolver.schedule_action(ranger, AttackCommand.new(ranger, brawler, 30, 2, 0, test_grid), 5)
	var ranger_future_act := AttackCommand.new(ranger, archer, 25, 2, 0, test_grid)
	test_resolver.schedule_action(ranger, ranger_future_act, 7)

	test_resolver.schedule_action(brawler, TargetCommand.new(brawler, warrior, 2), 1)
	test_resolver.schedule_action(brawler, DefendCommand.new(brawler, 0.5, 2, 1), 2)
	test_resolver.schedule_action(brawler, AttackCommand.new(brawler, warrior, 15, 2, 0, test_grid), 4)

	test_resolver.schedule_action(scout, MoveCommand.new(scout, test_grid, Vector2i(1, 0), 1), 1)
	test_resolver.schedule_action(scout, MoveCommand.new(scout, test_grid, Vector2i(1, 0), 2, 2), 2)

	# Execute Block 1..3
	for b: int in range(1, 4):
		test_resolver.execute_block(b)

	var scout_escaped_ok: bool = scout.is_withdrawn and not scout.is_active() and not test_grid.has_combatant(scout)

	# Block 4 Ambush: Reinforcements arrive on grid
	test_grid.place_combatant(berserker, Vector2i(5, 5))
	test_grid.place_combatant(skirmisher, Vector2i(6, 4))
	test_grid.place_combatant(archer, Vector2i(7, 5))
	test_grid.place_combatant(shaman, Vector2i(6, 5))

	test_resolver.schedule_action(berserker, TargetCommand.new(berserker, ranger, 1), 4)
	test_resolver.schedule_action(berserker, AttackCommand.new(berserker, ranger, 30, 2, 0, test_grid), 5)
	test_resolver.schedule_action(archer, TargetCommand.new(archer, ranger, 1), 4)
	test_resolver.schedule_action(archer, AttackCommand.new(archer, ranger, 25, 2, 0, test_grid), 5)
	test_resolver.schedule_action(shaman, TargetCommand.new(shaman, ranger, 1), 4)
	test_resolver.schedule_action(shaman, AttackCommand.new(shaman, ranger, 25, 2, 0, test_grid), 5)

	test_resolver.schedule_action(skirmisher, MoveCommand.new(skirmisher, test_grid, Vector2i(-1, 0), 1), 4)
	test_resolver.schedule_action(skirmisher, TargetCommand.new(skirmisher, warrior, 1), 5)
	test_resolver.schedule_action(skirmisher, AttackCommand.new(skirmisher, warrior, 12, 2, 0, test_grid), 7)

	# Execute Block 4..6
	for b: int in range(4, 7):
		test_resolver.execute_block(b)

	var ranger_defeated_ok: bool = not ranger.is_alive() and ranger.hp == 0
	var brawler_defeated_ok: bool = not brawler.is_alive() and brawler.hp == 0

	var ranger_future_cancelled_ok: bool = false
	for sa: CombatResolver.ScheduledAction in test_resolver.scheduled_actions:
		if sa.combatant == ranger and sa.start_block == 7:
			ranger_future_cancelled_ok = sa.is_cancelled
			break

	# Execute remaining Blocks 7..10 of Round 1
	for b: int in range(7, 11):
		test_resolver.execute_block(b)

	var warrior_survived_r1_ok: bool = warrior.is_alive() and warrior.hp == 73
	var r1_not_prematurely_ended: bool = not test_resolver.is_combat_resolved(allies, all_enemies)

	# Rollover to Round 2
	test_resolver.finish_round()
	var round_2_started_ok: bool = test_resolver.current_round == 2

	# Schedule Round 2 action for Warrior: Finish Skirmisher at (5, 4)
	test_resolver.schedule_action(warrior, AttackCommand.new(warrior, skirmisher, 25, 2, 0, test_grid), 1)
	test_resolver.execute_block(1)
	test_resolver.execute_block(2)

	var skirmisher_defeated_r2_ok: bool = not skirmisher.is_alive() and skirmisher.hp == 0

	print("\n=======================================================")
	print("=== GOBLIN AMBUSH & CASUALTY TEST ASSERTIONS        ===")
	print("=======================================================")
	print("  [Ambush] Initial grid deployment (4 units): %s" % ("PASS" if init_placement_ok else "FAIL"))
	print("  [Ambush] Scout withdrawal at Block 3: %s" % ("PASS" if scout_escaped_ok else "FAIL"))
	print("  [Casualty] Ranger defeated at Block 6: %s" % ("PASS" if ranger_defeated_ok else "FAIL"))
	print("  [Casualty] Ranger future actions cancelled: %s" % ("PASS" if ranger_future_cancelled_ok else "FAIL"))
	print("  [Casualty] Brawler defeated at Block 6: %s" % ("PASS" if brawler_defeated_ok else "FAIL"))
	print("  [Multi-Round] Warrior survived Round 1 (73 HP): %s" % ("PASS" if warrior_survived_r1_ok else "FAIL"))
	print("  [Multi-Round] Combat active with 1 ally & 4 goblins: %s" % ("PASS" if r1_not_prematurely_ended else "FAIL"))
	print("  [Multi-Round] Rollover to Round 2 succeeded: %s" % ("PASS" if round_2_started_ok else "FAIL"))
	print("  [Multi-Round] Round 2 combat resolved Skirmisher: %s" % ("PASS" if skirmisher_defeated_r2_ok else "FAIL"))

	warrior.clear_target()
	ranger.clear_target()
	brawler.clear_target()
	scout.clear_target()
	skirmisher.clear_target()
	archer.clear_target()
	berserker.clear_target()
	shaman.clear_target()
	test_resolver.registered_combatants.clear()

	return (init_placement_ok and scout_escaped_ok and ranger_defeated_ok and
		ranger_future_cancelled_ok and brawler_defeated_ok and warrior_survived_r1_ok and
		r1_not_prematurely_ended and round_2_started_ok and skirmisher_defeated_r2_ok)


# ==============================================================================
# TEST 6: CHARACTER SPECS, STATS, GEAR & INVENTORY TEST
# ==============================================================================

static func run_character_specs_and_inventory_test() -> bool:
	print("\n=======================================================")
	print("=== STARTING CHARACTER SPECS & INVENTORY TEST       ===")
	print("=======================================================")

	# 1. Base initialization from archetype
	var warrior_arch := ClassArchetype.create_warrior()
	var stats := CharacterStats.new(warrior_arch, 1)

	var base_stats_ok: bool = (
		stats.str_stat == 16 and
		stats.dex_stat == 10 and
		stats.int_stat == 8 and
		stats.cons_stat == 14 and
		stats.perc_stat == 10 and
		stats.know_stat == 8 and
		stats.level == 1
	)

	# 2. Derived Max HP & Speed
	# Max HP = 60 + (14 * 8) + (16 * 2) = 60 + 112 + 32 = 204
	# Speed = 8 + 10 = 18
	var derived_stats_ok: bool = (
		stats.get_max_hp() == 204 and
		stats.get_initiative_speed() == 18 and
		stats.get_melee_power() == 24
	)

	# 3. Guaranteed level up growths (Level 1 -> 2)
	# Warrior growth: STR +2, CONS +2, PERC +1
	var lvl_ok: bool = stats.level_up()
	var growths_ok: bool = (
		lvl_ok and
		stats.level == 2 and
		stats.str_stat == 18 and
		stats.cons_stat == 16 and
		stats.perc_stat == 11 and
		stats.unallocated_attribute_points == 1
	)

	# 4. Allocate 1 free point from level up
	var alloc_lvl_point_ok: bool = stats.allocate_attribute_point("INT") and stats.int_stat == 9 and stats.unallocated_attribute_points == 0

	# 5. Outside Event Stat Points (e.g. Ancient Shrine)
	stats.add_free_attribute_points(2)
	var alloc_event_points_ok: bool = (
		stats.unallocated_attribute_points == 2 and
		stats.allocate_attribute_point("DEX") and
		stats.allocate_attribute_point("KNOW") and
		stats.dex_stat == 11 and
		stats.know_stat == 9 and
		stats.unallocated_attribute_points == 0
	)

	# 6. Horizontal Progression: Mastery System & Contextual Bonuses
	stats.mastery.set_weapon_mastery("sword", 2)
	stats.mastery.set_lore_mastery("ancient_ruins", 1)

	var sword_context: Dictionary = {"weapon": "sword"}
	var ruins_context: Dictionary = {"biome": "ancient_ruins"}
	# STR base 18 + (Rank 2 * 2 = 4) = 22
	var mastery_str: int = stats.get_effective_stat("STR", sword_context)
	# KNOW base 9 + (Rank 1 * 2 = 2) = 11
	var mastery_know: int = stats.get_effective_stat("KNOW", ruins_context)
	var mastery_ok: bool = (mastery_str == 22 and mastery_know == 11)

	# 7. Equipment Stat Requirements & Mastery Threshold Discount
	var greatsword := Equipment.new(
		"iron_greatsword",
		"Iron Greatsword",
		Equipment.EquipSlot.MAIN_HAND,
		"sword",
		Item.ItemType.WEAPON,
		18,
		{"STR": 24},
		[
			{"stat": "STR", "threshold": 22, "bonus_power": 4, "desc": "Tier 2: Crushing Strike"},
			{"stat": "STR", "threshold": 28, "bonus_power": 8, "desc": "Tier 3: Sunder Guard"}
		]
	)

	var meets_req_ok: bool = greatsword.meets_requirements(stats, stats.mastery)

	# 8. Threshold Tiers Unlocked (STR 22 meets Tier 2 threshold of 22!)
	var active_tiers: Array[Dictionary] = greatsword.get_active_threshold_tiers(stats, stats.mastery)
	var tiers_ok: bool = (active_tiers.size() == 1 and greatsword.get_effective_equipment_power(stats, stats.mastery) == 22)

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
	var bow_penalty: Dictionary = longbow.get_wield_penalty(stats, stats.mastery)
	var penalty_ok: bool = (
		not longbow.meets_requirements(stats, stats.mastery) and
		bow_penalty["is_penalized"] == true and
		bow_penalty["deficit"] > 0 and
		longbow.get_effective_equipment_power(stats, stats.mastery) < 16 and
		longbow.get_effective_block_cost(stats, stats.mastery) > 1
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
	print("  [Mastery] Horizontal progression contextual bonuses: %s" % ("PASS" if mastery_ok else "FAIL"))
	print("  [Gear] Stat requirement & mastery discount check: %s" % ("PASS" if meets_req_ok else "FAIL"))
	print("  [Gear] Threshold tiers active boosts unlocked: %s" % ("PASS" if tiers_ok else "FAIL"))
	print("  [Gear] Deficit wield penalty applied: %s" % ("PASS" if penalty_ok else "FAIL"))
	print("  [Inventory] Battle pouch item consumed in combat: %s" % ("PASS" if pouch_used_ok else "FAIL"))

	return (base_stats_ok and derived_stats_ok and growths_ok and alloc_lvl_point_ok and
		alloc_event_points_ok and mastery_ok and meets_req_ok and tiers_ok and penalty_ok and pouch_used_ok)

