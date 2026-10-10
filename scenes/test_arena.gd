extends Node2D

# ==============================================================================
# Tactical RPG - Interactive Visual Combat Arena & Headless Test Suite
# ==============================================================================

# Combat Scenario & UI Controller
var scenario: CombatScenario
var arena_ui: CombatArenaController

# Active Combatant Unit Instances & Scenario State
var unit_warrior: Combatant
var unit_ranger: Combatant
var unit_brawler: Combatant
var unit_scout: Combatant
var unit_skirmisher: Combatant
var unit_archer: Combatant
var unit_berserker: Combatant
var unit_shaman: Combatant
var reinforcements_spawned: bool = false
var current_combat_round: int = 1

# Simulation & Replay State
var sim_grid: CombatGrid
var sim_resolver: CombatResolver
var sim_allies: Array[Combatant] = []
var sim_enemies: Array[Combatant] = []
var current_sim_block: int = 1
var replay_block: int = 1
var combined_selected_block: int = 1
var is_auto_playing: bool = false
var sim_finished: bool = false
var snapshots: Array[BlockSnapshot] = []
var recorded_actions: Array[CombatResolver.ScheduledAction] = []
var all_action_log_entries: Array[ActionLogEntry] = []
var active_log_entry_index: int = -1
var pending_block_actions: Array[CombatResolver.ScheduledAction] = []

# Multi-Round Timeline Traversal
var viewed_timeline_round: int = 1
var max_unlocked_round: int = 1
var round_recorded_actions: Dictionary = {} # int -> Array[CombatResolver.ScheduledAction]
var round_snapshots: Dictionary = {}        # int -> Array[BlockSnapshot]

# Selection and Inspector State
var selected_combatant: Combatant = null
var timeline_view_mode: TimelineViewMode = TimelineViewMode.TRACK_LAYERS
var combat_phase: CombatPhase = CombatPhase.PLANNING
var filter_active_only: bool = false
var selected_party_hero: Combatant = null
var auto_play_timer: Timer

# Delegated UI References
var ui_canvas: CanvasLayer
var grid_view: CombatGridView
var unit_token_nodes: Dictionary:
	get:
		return grid_view.unit_token_nodes if grid_view else {}
var action_log_dock: ActionLogDockUI
var roster_panel: RosterPanelUI
var log_rich_text: RichTextLabel
var log_modal: CombatLogModal
var playback_controls: PlaybackControlsUI
var timeline_dock: TimelineDockUI
var unit_inspector: UnitInspectorUI
var tactical_forecast: TacticalForecastUI
var timeline_modal: TimelineModal

var status_label: Label
var phase_badge_lbl: Label
var btn_step_prev: Button
var btn_play: Button
var btn_step_next: Button
var btn_reset_round: Button
var btn_reset_game: Button
var btn_popout: Button
var btn_next_round: Button
var btn_prev_round: Button
var btn_next_round_arrow: Button

const TILE_SIZE: int = 54
const GRID_SIZE: int = 9
const UNIT_BLOCK_W: int = 80
const BLOCK_GAP: int = 4
const TOTAL_BLOCKS: int = 10

enum TimelineViewMode {
	TRACK_LAYERS,
	COMBINED_BLOCK
}

enum CombatPhase {
	PLANNING,
	RESOLVING,
	REPLAY
}

func _get_combatant_token_texture(c: Combatant) -> Texture2D:
	return scenario.get_token_texture(c) if scenario else null

func _get_combatant_sprite_texture(c: Combatant) -> Texture2D:
	return scenario.get_sprite_texture(c) if scenario else null

func _ready() -> void:
	print("\n=======================================================")
	print("=== STARTING TACTICAL RPG ARENA HARNESS             ===")
	print("=======================================================")

	var test1_passed: bool = _run_warrior_vs_3_goblins_battle()
	var test2_passed: bool = _run_combat_grid_and_withdrawal_test()
	var test3_passed: bool = _run_resolution_feed_logging_test()
	var test4_passed: bool = _run_goblin_ambush_and_casualty_test()
	var all_tests_passed: bool = test1_passed and test2_passed and test3_passed and test4_passed

	var exit_code: int = 0 if all_tests_passed else 1

	if DisplayServer.get_name() == "headless":
		print("Execution finished headlessly. Exiting with code: %d\n" % exit_code)
		get_tree().quit(exit_code)
	else:
		print("Running in windowed mode. Initializing interactive tactical UI...\n")
		_setup_interactive_ui()
		_init_interactive_simulation()


# ==============================================================================
# SECTION 1: INTERACTIVE UI CREATION & ZERO-CLIPPING LAYOUT
# ==============================================================================

func _setup_interactive_ui() -> void:
	scenario = CombatScenario.create_default_arena_scenario()
	arena_ui = CombatArenaController.new()
	add_child(arena_ui)

	ui_canvas = arena_ui
	grid_view = arena_ui.grid_view
	action_log_dock = arena_ui.action_log_dock
	roster_panel = arena_ui.roster_panel
	log_rich_text = arena_ui.log_rich_text
	log_modal = arena_ui.log_modal
	playback_controls = arena_ui.playback_controls
	timeline_dock = arena_ui.timeline_dock
	unit_inspector = arena_ui.unit_inspector
	tactical_forecast = arena_ui.tactical_forecast
	timeline_modal = arena_ui.timeline_modal

	status_label = arena_ui.playback_controls.status_label
	phase_badge_lbl = arena_ui.playback_controls.phase_badge_lbl
	btn_step_prev = arena_ui.playback_controls.btn_step_prev
	btn_play = arena_ui.playback_controls.btn_play
	btn_step_next = arena_ui.playback_controls.btn_step_next
	btn_reset_round = arena_ui.playback_controls.btn_reset_round
	btn_reset_game = arena_ui.playback_controls.btn_reset_game
	btn_popout = arena_ui.playback_controls.btn_popout
	btn_next_round = arena_ui.playback_controls.btn_next_round
	btn_prev_round = arena_ui.timeline_dock.btn_prev_round
	btn_next_round_arrow = arena_ui.timeline_dock.btn_next_round_arrow

	# Connect arena controller signals to session orchestration
	arena_ui.combatant_selected.connect(_select_combatant)
	arena_ui.combatant_hovered.connect(func(c: Combatant, token: Control) -> void:
		_show_unit_hover_card(c, token)
	)
	arena_ui.combatant_unhovered.connect(_hide_unit_hover_card)
	arena_ui.block_selected.connect(func(b: int) -> void:
		_on_ruler_block_clicked(b)
		if timeline_modal != null and timeline_modal.visible:
			_update_modal_tracks_display()
	)
	arena_ui.step_prev_pressed.connect(_on_step_prev_pressed)
	arena_ui.play_pressed.connect(_on_play_round_pressed)
	arena_ui.step_next_pressed.connect(_on_step_next_pressed)
	arena_ui.reset_round_pressed.connect(_on_reset_active_round_pressed)
	arena_ui.reset_game_pressed.connect(_on_reset_game_pressed)
	arena_ui.next_round_pressed.connect(_on_next_round_pressed)
	arena_ui.round_nav_requested.connect(_navigate_timeline_round)
	arena_ui.log_entry_clicked.connect(_on_log_modal_cell_clicked)

	timeline_dock.action_lookup_fn = _find_recorded_action_starting_at
	timeline_dock.hero_view_requested.connect(func() -> void:
		if not sim_allies.is_empty():
			_select_combatant(sim_allies[0])
	)
	timeline_modal.action_lookup_fn = _find_recorded_action_starting_at
	timeline_modal.token_texture_fn = _get_combatant_token_texture

	# Auto-play timer
	auto_play_timer = Timer.new()
	auto_play_timer.wait_time = 0.55
	auto_play_timer.one_shot = false
	auto_play_timer.timeout.connect(_on_auto_play_tick)
	add_child(auto_play_timer)


# ==============================================================================
# SECTION 2: SIMULATION SETUP WITH PRE-PLANNED COMMANDS & RECORDED ARCHIVE
# ==============================================================================


func _init_interactive_simulation() -> void:
	current_combat_round = 1
	max_unlocked_round = 1
	viewed_timeline_round = 1
	round_recorded_actions.clear()
	round_snapshots.clear()
	current_sim_block = 1
	replay_block = 1
	combined_selected_block = 1
	sim_finished = false
	is_auto_playing = false
	reinforcements_spawned = false
	combat_phase = CombatPhase.PLANNING
	auto_play_timer.stop()
	btn_play.text = "▶"
	btn_play.tooltip_text = "Auto-Resolve Round 1"
	if btn_next_round:
		btn_next_round.visible = false
	phase_badge_lbl.text = "PLANNING PHASE"
	phase_badge_lbl.add_theme_color_override("font_color", Color(0.95, 0.8, 0.35))
	status_label.text = "Round 1: Set party actions, then Auto-Resolve."

	# Clear previous tokens
	if grid_view:
		grid_view.clear_tokens()

	# Clear roster cards in both party and enemy containers
	if roster_panel:
		roster_panel.clear_roster()

	all_action_log_entries.clear()
	active_log_entry_index = -1
	pending_block_actions.clear()
	_update_active_log_cell()

	log_rich_text.clear()
	_append_log("[color=cyan]>>> New 10-Block Tactical Battle Initialized (Round 1 Pre-planned Actions Loaded).[/color]")

	# 1. Initialize 9x9 Closed Arena with designated Exit at (8, 4)
	var exit_coord := Vector2i(8, 4)
	sim_grid = CombatGrid.new(CombatGrid.ArenaType.CLOSED, [exit_coord])
	sim_resolver = CombatResolver.new()
	sim_resolver.grid = sim_grid
	sim_resolver.log_emitted.connect(_on_resolver_log_emitted)

	# 2. Combatants (Allied Party & Enemy Forces)
	unit_warrior = Combatant.new("Warrior", 100, 15, 1)
	unit_ranger = Combatant.new("Ranger", 80, 14, 2)
	unit_brawler = Combatant.new("Enemy Brawler", 45, 12, 1)
	unit_scout = Combatant.new("Fleeing Scout", 30, 16, 1)

	# Goblin Ambush Reinforcements (Arriving Block 4 after Scout exit)
	unit_skirmisher = Combatant.new("Goblin Skirmisher", 40, 15, 1)
	unit_archer = Combatant.new("Goblin Archer", 35, 13, 3)
	unit_berserker = Combatant.new("Goblin Berserker", 55, 11, 1)
	unit_shaman = Combatant.new("Goblin Shaman", 35, 12, 2)

	sim_allies = [unit_warrior, unit_ranger]
	sim_enemies = [unit_brawler, unit_scout, unit_skirmisher, unit_archer, unit_berserker, unit_shaman]

	var initial_combatants: Array[Combatant] = [
		unit_warrior, unit_ranger, unit_brawler, unit_scout,
		unit_skirmisher, unit_archer, unit_berserker, unit_shaman
	]
	for c: Combatant in initial_combatants:
		sim_resolver.register_combatant(c)

	# 3. Center-ring deployment for initial combatants only
	sim_grid.deploy_combatants(sim_allies, [unit_brawler, unit_scout])

	# 4. Schedule Pre-Planned Commands for Round 1
	# Warrior:
	#   Block 1: Move Up to Center (4, 4) [1b]
	#   Block 2-3: Melee Attack on Brawler [2b]
	#   Block 4: Target Goblin Berserker [1b]
	#   Block 5-6: Attack Goblin Berserker (25 power) [2b]
	#   Block 7-8: Defend (50% mitigation against ambush) [2b]
	#   Block 9-10: Counter-Attack Goblin Skirmisher [2b]
	sim_resolver.schedule_action(unit_warrior, MoveCommand.new(unit_warrior, sim_grid, Vector2i(0, -1), 1), 1)
	sim_resolver.schedule_action(unit_warrior, AttackCommand.new(unit_warrior, unit_brawler, 25, 2, 0, sim_grid), 2)
	sim_resolver.schedule_action(unit_warrior, TargetCommand.new(unit_warrior, unit_berserker, 1), 4)
	sim_resolver.schedule_action(unit_warrior, AttackCommand.new(unit_warrior, unit_berserker, 25, 2, 0, sim_grid), 5)
	sim_resolver.schedule_action(unit_warrior, DefendCommand.new(unit_warrior, 0.5, 2, 1), 7)
	sim_resolver.schedule_action(unit_warrior, HoldStanceCommand.new(unit_warrior, 0), 9)

	# Ranger:
	#   Block 1-2: Skill - Focus (+50% ATK for 2 rds) [2b]
	#   Block 4: Move Up to (4, 5) [1b]
	#   Block 5-6: Boosted Ranged Attack on Brawler (resolves Block 6 -> Brawler defeated!) [2b]
	#   Block 7-8: Attack Archer (cancelled when Ranger falls to ambush at Block 6) [2b]
	sim_resolver.schedule_action(unit_ranger, FocusCommand.new(unit_ranger, 1.5, 2, 2, 0), 1)
	sim_resolver.schedule_action(unit_ranger, MoveCommand.new(unit_ranger, sim_grid, Vector2i(0, -1), 1), 4)
	sim_resolver.schedule_action(unit_ranger, AttackCommand.new(unit_ranger, unit_brawler, 30, 2, 0, sim_grid), 5)
	sim_resolver.schedule_action(unit_ranger, AttackCommand.new(unit_ranger, unit_archer, 25, 2, 0, sim_grid), 7)

	# Enemy Brawler (Scheduled in secret):
	#   Block 1: Target Warrior [1b]
	#   Block 2-3: Defend (50% mitigation) [2b]
	#   Block 4-5: Counter-Attack Warrior (15 power) [2b]
	sim_resolver.schedule_action(unit_brawler, TargetCommand.new(unit_brawler, unit_warrior, 2), 1)
	sim_resolver.schedule_action(unit_brawler, DefendCommand.new(unit_brawler, 0.5, 2, 1), 2)
	sim_resolver.schedule_action(unit_brawler, AttackCommand.new(unit_brawler, unit_warrior, 15, 2, 0, sim_grid), 4)

	# Fleeing Scout (Scheduled in secret):
	#   Block 1: Move East to (6, 4) [1b]
	#   Block 2-3: Sprint East 2 steps to (8, 4) Exit Tile -> Safely Withdraws! [2b]
	sim_resolver.schedule_action(unit_scout, MoveCommand.new(unit_scout, sim_grid, Vector2i(1, 0), 1), 1)
	sim_resolver.schedule_action(unit_scout, MoveCommand.new(unit_scout, sim_grid, Vector2i(1, 0), 2, 2), 2)

	# Ambush Reinforcements (Continuous actions through Block 10 without empty gaps):
	# Berserker (spawns (4, 5)):
	sim_resolver.schedule_action(unit_berserker, TargetCommand.new(unit_berserker, unit_ranger, 1), 4)
	sim_resolver.schedule_action(unit_berserker, AttackCommand.new(unit_berserker, unit_ranger, 30, 2, 0, sim_grid), 5)
	sim_resolver.schedule_action(unit_berserker, TargetCommand.new(unit_berserker, unit_warrior, 1), 7)
	sim_resolver.schedule_action(unit_berserker, AttackCommand.new(unit_berserker, unit_warrior, 15, 3, 0, sim_grid), 8)

	# Archer (spawns (6, 5)):
	sim_resolver.schedule_action(unit_archer, TargetCommand.new(unit_archer, unit_ranger, 1), 4)
	sim_resolver.schedule_action(unit_archer, AttackCommand.new(unit_archer, unit_ranger, 20, 2, 0, sim_grid), 5)
	sim_resolver.schedule_action(unit_archer, TargetCommand.new(unit_archer, unit_warrior, 1), 7)
	sim_resolver.schedule_action(unit_archer, AttackCommand.new(unit_archer, unit_warrior, 12, 3, 0, sim_grid), 8)

	# Shaman (spawns (6, 4)):
	sim_resolver.schedule_action(unit_shaman, TargetCommand.new(unit_shaman, unit_ranger, 1), 4)
	sim_resolver.schedule_action(unit_shaman, AttackCommand.new(unit_shaman, unit_ranger, 20, 2, 0, sim_grid), 5)
	sim_resolver.schedule_action(unit_shaman, TargetCommand.new(unit_shaman, unit_warrior, 1), 7)
	sim_resolver.schedule_action(unit_shaman, AttackCommand.new(unit_shaman, unit_warrior, 10, 3, 0, sim_grid), 8)

	# Skirmisher (spawns (5, 4)):
	sim_resolver.schedule_action(unit_skirmisher, TargetCommand.new(unit_skirmisher, unit_warrior, 1), 4)
	sim_resolver.schedule_action(unit_skirmisher, AttackCommand.new(unit_skirmisher, unit_warrior, 12, 3, 0, sim_grid), 5)
	sim_resolver.schedule_action(unit_skirmisher, DefendCommand.new(unit_skirmisher, 0.4, 2, 1), 8)
	sim_resolver.schedule_action(unit_skirmisher, HoldStanceCommand.new(unit_skirmisher, 0), 10)

	# 5. Clone All Scheduled Actions into Permanent Archive
	recorded_actions.clear()
	for a in sim_resolver.scheduled_actions:
		recorded_actions.append(CombatResolver.ScheduledAction.new(a.combatant, a.command, a.start_block, a.end_block))
	round_recorded_actions[1] = recorded_actions.duplicate()

	# 6. Build visual unit tokens and roster cards
	arena_ui.mount_scenario(scenario, sim_grid)

	# 7. Capture Initial Snapshot (State 0 before block 1 executes)
	snapshots.clear()
	_capture_snapshot(0)
	round_snapshots[1] = snapshots.duplicate()
	_update_round_traversal_ui()

	# 8. Setup 2x2 Party Focus Box & Default Selection
	selected_party_hero = unit_warrior

	# Set default selected unit to Warrior
	_select_combatant(unit_warrior)
	_update_timeline_tracks_display()
	_update_combined_block_display(1)


func _create_unit_token(combatant: Combatant, texture: Texture2D) -> void:
	if not grid_view or not sim_grid:
		return
	var pos: Vector2i = sim_grid.get_position(combatant)
	grid_view.create_unit_token(combatant, texture, pos)


func _show_unit_hover_card(c: Combatant, token: Control) -> void:
	if not grid_view or not is_instance_valid(c):
		return
	var is_ally: bool = sim_allies.has(c)
	var cur_snap: BlockSnapshot = null
	var snaps: Array = round_snapshots.get(viewed_timeline_round, snapshots)
	var cur_b: int = replay_block if combat_phase == CombatPhase.REPLAY else current_sim_block
	if cur_b >= 0 and cur_b < snaps.size():
		@warning_ignore("unsafe_cast")
		cur_snap = snaps[cur_b] as BlockSnapshot

	var pos: Vector2i = sim_grid.get_position(c) if sim_grid else Vector2i(-1, -1)
	grid_view.show_hover_card(c, token, cur_snap, is_ally, pos)


func _hide_unit_hover_card() -> void:
	if grid_view != null:
		grid_view.hide_hover_card()


func _create_roster_card(combatant: Combatant, team_color: Color) -> void:
	if not roster_panel:
		return
	var is_ally: bool = sim_allies.has(combatant)
	var grid_pos: Vector2i = sim_grid.get_position(combatant) if sim_grid else Vector2i(-1, -1)
	roster_panel.create_roster_card(combatant, team_color, is_ally, grid_pos)


func _add_party_placeholders() -> void:
	if roster_panel:
		roster_panel.add_party_placeholders()


func _add_enemy_placeholders() -> void:
	pass


# ==============================================================================
# SECTION 3: SNAPSHOTS, REPLAY RESTORATION & UNIFIED ROSTER UPDATES
# ==============================================================================

func _capture_snapshot(block_idx: int) -> void:
	var snap := BlockSnapshot.new()
	snap.block_index = block_idx
	snap.round_num = current_combat_round
	snap.reinforcements_spawned = reinforcements_spawned
	for c: Combatant in sim_resolver.registered_combatants:
		snap.positions[c] = sim_grid.get_position(c)
		snap.hps[c] = c.hp
		snap.max_hps[c] = c.max_hp
		snap.stances[c] = c.current_stance
		snap.targets[c] = c.active_target
		snap.is_active[c] = c.is_active()
		snap.is_withdrawn[c] = c.is_withdrawn
		snap.attack_boost_mult[c] = c.attack_boost_multiplier
		snap.attack_boost_rds[c] = c.attack_boost_rounds_remaining
		snap.defense_mit[c] = c.defense_mitigation_ratio
	snapshots.append(snap)
	round_snapshots[current_combat_round] = snapshots.duplicate()


func _restore_snapshot(snap: BlockSnapshot) -> void:
	reinforcements_spawned = snap.reinforcements_spawned
	if grid_view:
		grid_view.restore_token_states(snap)

	_update_all_roster_cards(snap)
	_update_selected_unit_panel()
	_update_tactical_forecast_panel()


func _update_all_roster_cards(snap: BlockSnapshot = null) -> void:
	if roster_panel:
		roster_panel.update_all_roster_cards(
			sim_allies,
			current_combat_round,
			reinforcements_spawned,
			sim_grid,
			snap
		)


# ==============================================================================
# SECTION 4: SELECTION & DYNAMIC INSPECTOR UPDATES
# ==============================================================================

func _select_combatant(combatant: Combatant) -> void:
	selected_combatant = combatant
	if sim_allies.has(combatant):
		selected_party_hero = combatant
	_update_roster_card_highlights()
	_update_unit_token_highlights()
	_update_selected_unit_panel()
	_update_tactical_forecast_panel()
	_update_timeline_tracks_display()
	_update_combined_block_display(combined_selected_block)

	# Ensure hover card displays for selected unit as well
	if grid_view and grid_view.unit_token_nodes.has(combatant):
		@warning_ignore("unsafe_cast")
		var tok := grid_view.unit_token_nodes[combatant] as Control
		if is_instance_valid(tok) and tok.visible:
			_show_unit_hover_card(combatant, tok)


func _update_roster_card_highlights() -> void:
	if roster_panel:
		roster_panel.update_highlights(selected_combatant, sim_allies)


func _update_unit_token_highlights() -> void:
	if grid_view:
		grid_view.update_selection_highlight(selected_combatant)


func _update_selected_unit_panel() -> void:
	if not selected_combatant or not unit_inspector:
		return

	var is_ally: bool = sim_allies.has(selected_combatant)
	var pos_text: String = ""
	if selected_combatant.is_active():
		pos_text = "Pos: %s" % str(sim_grid.get_position(selected_combatant))
	elif selected_combatant.is_withdrawn:
		pos_text = "Pos: Withdrawn (Safe)"
	else:
		pos_text = "Pos: Defeated"

	unit_inspector.update_inspector(
		selected_combatant,
		is_ally,
		pos_text,
		_get_combatant_sprite_texture(selected_combatant)
	)


func _update_tactical_forecast_panel() -> void:
	if not selected_combatant or not tactical_forecast:
		return

	var target: Combatant = null
	if selected_combatant.active_target is Combatant:
		@warning_ignore("unsafe_cast")
		target = selected_combatant.active_target as Combatant
	elif sim_allies.has(selected_combatant):
		target = sim_enemies[0]
	else:
		target = sim_allies[0]

	tactical_forecast.update_forecast(selected_combatant, target, sim_grid)


func _update_timeline_tracks_display() -> void:
	if timeline_dock == null:
		return

	var active_playhead: int = replay_block if combat_phase == CombatPhase.REPLAY else current_sim_block
	var is_enemy_selected: bool = selected_combatant != null and not sim_allies.has(selected_combatant)
	var is_enemy_masked: bool = is_enemy_selected and combat_phase != CombatPhase.REPLAY

	var target_unit: Combatant = null
	if is_enemy_selected and combat_phase == CombatPhase.REPLAY:
		target_unit = selected_combatant
	elif selected_party_hero != null:
		target_unit = selected_party_hero
	elif selected_combatant != null and sim_allies.has(selected_combatant):
		target_unit = selected_combatant
	elif not sim_allies.is_empty():
		target_unit = sim_allies[0]

	var can_go_prev: bool = (viewed_timeline_round > 1)
	var can_go_next: bool = (viewed_timeline_round < max_unlocked_round)

	timeline_dock.update_view(
		target_unit,
		active_playhead,
		is_enemy_masked,
		can_go_prev,
		can_go_next
	)


func _find_recorded_action_starting_at(combatant: Combatant, block: int) -> CombatResolver.ScheduledAction:
	for a: CombatResolver.ScheduledAction in recorded_actions:
		if a.combatant == combatant and a.start_block == block:
			return a
	return null


func _get_recorded_actions_at_block(block: int) -> Array[CombatResolver.ScheduledAction]:
	var res: Array[CombatResolver.ScheduledAction] = []
	for a: CombatResolver.ScheduledAction in recorded_actions:
		if a.start_block <= block and a.end_block >= block:
			res.append(a)
	return res



# ==============================================================================
# SECTION 6: COMBINED BLOCK INSPECTOR (SPLIT COLUMNS WITH REPLAY VISIBILITY)
# ==============================================================================

func _update_combined_block_display(_block: int) -> void:
	# Deprecated: Full screen log modal and master timeline modal replace old combined block inspector
	pass





func _navigate_combined_block(delta: int) -> void:
	var target_b: int = clampi(replay_block + delta, 1, TOTAL_BLOCKS)
	_on_ruler_block_clicked(target_b)


# ==============================================================================
# SECTION 7: STEPPING, AUTO-RESOLVE, REPLAY & INPUT CONTROLS
# ==============================================================================

func _step_simulation() -> void:
	if sim_finished and combat_phase != CombatPhase.REPLAY:
		return

	if combat_phase == CombatPhase.PLANNING:
		combat_phase = CombatPhase.RESOLVING
		phase_badge_lbl.text = "RESOLVING COMBAT"
		phase_badge_lbl.add_theme_color_override("font_color", Color(0.4, 0.8, 1.0))
		current_sim_block = 1
		pending_block_actions.clear()

	if combat_phase == CombatPhase.RESOLVING:
		# If no pending actions are queued for this block, prepare the block!
		if pending_block_actions.is_empty():
			# Mid-round ambush trigger: When Block 4 is reached in Round 1, 4 new goblins appear!
			if current_sim_block == 4 and not reinforcements_spawned and current_combat_round == 1:
				_spawn_goblin_reinforcements()
				var ambush_entry := ActionLogEntry.new()
				ambush_entry.round_num = current_combat_round
				ambush_entry.block_num = current_sim_block
				ambush_entry.ref_tag = "R%d-B%02d" % [current_combat_round, current_sim_block]
				ambush_entry.actor_name = "Goblin Pack"
				ambush_entry.actor_is_ally = false
				ambush_entry.category = "EVENT"
				ambush_entry.skill_name = "Ambush Reinforcements"
				ambush_entry.skill_subtype = "Perimeter Breach"
				ambush_entry.outcome_tag = "AMBUSH!"
				ambush_entry.detail_text = "4 Goblin reinforcements arrive from eastern ridge!"
				ambush_entry.status_text = "Hostiles: 6 total"
				all_action_log_entries.append(ambush_entry)
				active_log_entry_index = all_action_log_entries.size() - 1
				_update_active_log_cell()
				_trigger_action_impact_fx(ambush_entry)
				return

			_append_log("[color=#fde047][b]━━━ BLOCK %d RESOLUTION (ROUND %d) ━━━[/b][/color]" % [current_sim_block, current_combat_round])
			pending_block_actions = sim_resolver.prepare_block(current_sim_block)

		# Execute next single action if available
		if not pending_block_actions.is_empty():
			var act: CombatResolver.ScheduledAction = pending_block_actions.pop_front()
			sim_resolver.execute_single_action(act)
			var entry := _build_entry_from_action(act, current_combat_round, current_sim_block)
			all_action_log_entries.append(entry)
			active_log_entry_index = all_action_log_entries.size() - 1
			_update_active_log_cell()
			_trigger_action_impact_fx(entry)
			_sync_visuals_after_block()

		# If all actions in this block have finished:
		if pending_block_actions.is_empty():
			sim_resolver.block_completed.emit(current_sim_block)
			_capture_snapshot(current_sim_block)
			_sync_visuals_after_block()

			var is_over: bool = sim_resolver.is_combat_resolved(sim_allies, sim_enemies)
			if is_over or current_sim_block >= TOTAL_BLOCKS:
				sim_finished = true
				combat_phase = CombatPhase.REPLAY
				replay_block = current_sim_block
				auto_play_timer.stop()
				is_auto_playing = false
				btn_play.text = "▶"
				btn_play.tooltip_text = "Play Replay"
				phase_badge_lbl.text = "ROUND %d CONCLUDED - REPLAY MODE" % current_combat_round
				phase_badge_lbl.add_theme_color_override("font_color", Color(0.3, 0.9, 0.5))

				if btn_next_round:
					btn_next_round.visible = (not is_over and current_combat_round < 3)
					if current_combat_round == 1:
						btn_next_round.text = "⚔ ADVANCE TO ROUND 2"
					elif current_combat_round == 2:
						btn_next_round.text = "⚔ ADVANCE TO ROUND 3 (FINAL SHOWDOWN)"

				if is_over:
					phase_badge_lbl.text = "🏆 COMBAT CONCLUDED - TOTAL VICTORY"
					phase_badge_lbl.add_theme_color_override("font_color", Color(0.4, 0.9, 1.0))
					status_label.text = "🏆 All hostiles eliminated! Warrior stands victorious. Replay available."
					_append_log("\n[color=#38bdf8][b]🏆 TOTAL VICTORY: All hostiles eliminated! Use timeline arrows [◀ / ▶] to review all rounds.[/b][/color]")
				elif current_combat_round == 1:
					status_label.text = "Round 1 complete! Click Advance to Round 2."
					_append_log("\n[color=#4ade80][b]🎉 ROUND 1 RESOLVED: Ranger fell in the ambush, Warrior holds at 73 HP against 4 goblins! Click ADVANCE TO ROUND 2 to begin the Last Stand.[/b][/color]")
				elif current_combat_round == 2:
					status_label.text = "Round 2 complete! Skirmisher & Shaman eliminated. Advance to Round 3."
					_append_log("\n[color=#4ade80][b]🎉 ROUND 2 RESOLVED: Warrior eliminated Skirmisher and Shaman! Advance to Round 3 for the final confrontation.[/b][/color]")
				else:
					status_label.text = "Round 3 concluded. Review battle replay."
					_append_log("\n[color=#4ade80][b]🎉 ROUND 3 RESOLVED: Battle concluded. Review replay or Reset.[/b][/color]")

				_update_round_traversal_ui()
				_update_timeline_tracks_display()
				_update_combined_block_display(replay_block)
				return

			current_sim_block += 1
			status_label.text = "Ready for Block %d." % current_sim_block
			_update_timeline_tracks_display()
			_update_combined_block_display(current_sim_block)

	elif combat_phase == CombatPhase.REPLAY:
		# Step Replay through recorded action entries
		if not all_action_log_entries.is_empty():
			active_log_entry_index = (active_log_entry_index + 1) % all_action_log_entries.size()
			var replay_entry := all_action_log_entries[active_log_entry_index]
			_update_active_log_cell()
			_trigger_action_impact_fx(replay_entry)
			replay_block = replay_entry.block_num
			if replay_block >= 0 and replay_block < snapshots.size():
				_restore_snapshot(snapshots[replay_block])
			status_label.text = "Replaying: %s [%s]" % [replay_entry.skill_name, replay_entry.ref_tag]
			_update_timeline_tracks_display()
			_update_combined_block_display(replay_block)
		else:
			replay_block = (replay_block % TOTAL_BLOCKS) + 1
			status_label.text = "Replay Block %d of 10." % replay_block
			_update_timeline_tracks_display()
			_update_combined_block_display(replay_block)

		if timeline_modal != null and timeline_modal.visible:
			_update_modal_tracks_display()


func _sync_visuals_after_block() -> void:
	for combatant: Combatant in unit_token_nodes:
		@warning_ignore("unsafe_cast")
		var token := unit_token_nodes[combatant] as Control
		var target_coord: Vector2i = sim_grid.get_position(combatant)
		var is_alive: bool = combatant.is_alive()
		var is_withdrawn: bool = combatant.is_withdrawn
		var is_act: bool = combatant.is_active() and is_alive and not is_withdrawn

		# Requirement 7: Remove dead or withdrawn unit from board completely
		if target_coord == Vector2i(-1, -1) or not is_act:
			token.visible = false
			token.modulate.a = 0.0
		else:
			token.visible = true
			token.modulate = Color.WHITE
			var target_px := Vector2(target_coord.x * TILE_SIZE, target_coord.y * TILE_SIZE)
			if token.position != target_px:
				var tw := create_tween()
				tw.tween_property(token, "position", target_px, 0.25).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)

	_update_all_roster_cards()


func _spawn_goblin_reinforcements() -> void:
	reinforcements_spawned = true
	_append_log("\n[color=#ef4444][b]⚠️ AMBUSH! Scout's escape alerted the pack! 4 Goblin Reinforcements enter the arena![/b][/color]")
	_append_log("[color=#f87171]  • Goblin Berserker (55 HP) deployed at (4, 5)[/color]")
	_append_log("[color=#f87171]  • Goblin Skirmisher (40 HP) deployed at (5, 4)[/color]")
	_append_log("[color=#f87171]  • Goblin Shaman (35 HP) deployed at (6, 4)[/color]")
	_append_log("[color=#f87171]  • Goblin Archer (35 HP) deployed at (6, 5)[/color]")

	sim_grid.place_combatant(unit_berserker, Vector2i(4, 5))
	sim_grid.place_combatant(unit_skirmisher, Vector2i(5, 4))
	sim_grid.place_combatant(unit_shaman, Vector2i(6, 4))
	sim_grid.place_combatant(unit_archer, Vector2i(6, 5))

	var new_goblins: Array[Combatant] = [unit_berserker, unit_skirmisher, unit_archer, unit_shaman]
	for c: Combatant in new_goblins:
		if unit_token_nodes.has(c):
			@warning_ignore("unsafe_cast")
			var token := unit_token_nodes[c] as Control
			var coord: Vector2i = sim_grid.get_position(c)
			token.position = Vector2(coord.x * TILE_SIZE, coord.y * TILE_SIZE)
			var tw := create_tween()
			tw.tween_property(token, "modulate:a", 1.0, 0.35)
		if roster_panel and roster_panel.roster_cards.has(c):
			@warning_ignore("unsafe_cast")
			var card := roster_panel.roster_cards[c] as Control
			if is_instance_valid(card):
				card.visible = true

	_update_all_roster_cards()


func _on_next_round_pressed() -> void:
	if current_combat_round == 1:
		_start_round_2()
	elif current_combat_round == 2:
		_start_round_3()


func _start_round_2() -> void:
	current_combat_round = 2
	max_unlocked_round = maxi(max_unlocked_round, 2)
	sim_resolver.finish_round()

	current_sim_block = 1
	replay_block = 1
	combined_selected_block = 1
	sim_finished = false
	is_auto_playing = false
	combat_phase = CombatPhase.PLANNING
	auto_play_timer.stop()
	btn_play.text = "▶"
	btn_play.tooltip_text = "Auto-Resolve Round 2"
	if btn_next_round:
		btn_next_round.visible = false

	# Ensure proper combatant baseline for Round 2 (Exact final coordinates of Round 1: zero movement between rounds)
	unit_warrior.hp = 73
	unit_warrior.set_stance(CombatTypes.StanceType.NEUTRAL)
	sim_grid.combatant_positions[unit_warrior] = Vector2i(4, 4)

	unit_skirmisher.hp = 40
	unit_skirmisher.set_stance(CombatTypes.StanceType.NEUTRAL)
	sim_grid.combatant_positions[unit_skirmisher] = Vector2i(5, 4)

	unit_berserker.hp = 30
	unit_berserker.set_stance(CombatTypes.StanceType.NEUTRAL)
	sim_grid.combatant_positions[unit_berserker] = Vector2i(4, 5)

	unit_shaman.hp = 35
	unit_shaman.set_stance(CombatTypes.StanceType.NEUTRAL)
	sim_grid.combatant_positions[unit_shaman] = Vector2i(6, 4)

	unit_archer.hp = 35
	unit_archer.set_stance(CombatTypes.StanceType.NEUTRAL)
	sim_grid.combatant_positions[unit_archer] = Vector2i(6, 5)

	unit_brawler.hp = 0
	unit_scout.is_withdrawn = true
	unit_ranger.hp = 0

	# Prune any defeated or withdrawn enemies from roster cards for Round 2
	for enemy: Combatant in [unit_brawler, unit_scout]:
		if roster_panel:
			roster_panel.prune_enemy(enemy)

	for enemy: Combatant in [unit_skirmisher, unit_berserker, unit_archer, unit_shaman]:
		if roster_panel and not roster_panel.roster_cards.has(enemy):
			_create_roster_card(enemy, Color(0.9, 0.3, 0.3))
		elif roster_panel and roster_panel.roster_cards.has(enemy):
			@warning_ignore("unsafe_cast")
			var card := roster_panel.roster_cards[enemy] as Control
			if is_instance_valid(card):
				card.visible = true

	pending_block_actions.clear()
	var r2_entry := ActionLogEntry.new()
	r2_entry.round_num = 2
	r2_entry.block_num = 1
	r2_entry.ref_tag = "R2-B01"
	r2_entry.actor_name = "Warrior"
	r2_entry.actor_is_ally = true
	r2_entry.category = "EVENT"
	r2_entry.skill_name = "Round 2 Commenced"
	r2_entry.skill_subtype = "Last Stand"
	r2_entry.outcome_tag = "ROUND 2"
	r2_entry.detail_text = "Warrior stands alone at (4, 4) against 4 goblins"
	r2_entry.status_text = "Round 2 Active"
	all_action_log_entries.append(r2_entry)
	active_log_entry_index = all_action_log_entries.size() - 1
	_update_active_log_cell()

	phase_badge_lbl.text = "ROUND 2: PLANNING PHASE"
	phase_badge_lbl.add_theme_color_override("font_color", Color(0.95, 0.8, 0.35))
	status_label.text = "Round 2: Warrior (73 HP) stands alone against 4 goblins! Set actions or Auto-Resolve."

	_append_log("\n[color=#38bdf8][b]⚔⚔⚔ COMMENCING ROUND 2: WARRIOR'S LAST STAND ⚔⚔⚔[/b][/color]")
	_append_log("[color=#93c5fd]Surviving hero Warrior holds center at (4, 4). Pre-planned Round 2 actions loaded.[/color]")

	# Schedule Pre-Planned Commands for Round 2:
	# Warrior:
	#   Block 1-2: Cleaving Strike on Skirmisher at (5, 4) (power 40) [2b] -> Skirmisher DEFEATED!
	#   Block 3-4: Defend (50% mitigation) [2b]
	#   Block 5-6: Heavy Strike on Berserker at (4, 5) (power 35) [2b] -> Berserker DEFEATED!
	#   Block 7: Move East to (5, 4) [1b] (steps into vacated tile)
	#   Block 8-9: Strike on Shaman at (6, 4) (power 20) [2b] -> Shaman damaged (15 HP remaining)!
	#   Block 10: Shield Defend [1b]
	sim_resolver.schedule_action(unit_warrior, AttackCommand.new(unit_warrior, unit_skirmisher, 40, 2, 0, sim_grid), 1)
	sim_resolver.schedule_action(unit_warrior, DefendCommand.new(unit_warrior, 0.5, 2, 1), 3)
	sim_resolver.schedule_action(unit_warrior, AttackCommand.new(unit_warrior, unit_berserker, 35, 2, 0, sim_grid), 5)
	sim_resolver.schedule_action(unit_warrior, MoveCommand.new(unit_warrior, sim_grid, Vector2i(1, 0), 1), 7)
	sim_resolver.schedule_action(unit_warrior, AttackCommand.new(unit_warrior, unit_shaman, 20, 2, 0, sim_grid), 8)
	sim_resolver.schedule_action(unit_warrior, DefendCommand.new(unit_warrior, 0.5, 1, 1), 10)

	# Goblin Skirmisher (40 HP at (5, 4)):
	sim_resolver.schedule_action(unit_skirmisher, TargetCommand.new(unit_skirmisher, unit_warrior, 1), 1)
	sim_resolver.schedule_action(unit_skirmisher, AttackCommand.new(unit_skirmisher, unit_warrior, 10, 1, 0, sim_grid), 2)

	# Goblin Berserker (30 HP at (4, 5)):
	# Defeated at Block 6 by Warrior's Heavy Strike; NO action active at Block 7!
	sim_resolver.schedule_action(unit_berserker, TargetCommand.new(unit_berserker, unit_warrior, 1), 1)
	sim_resolver.schedule_action(unit_berserker, AttackCommand.new(unit_berserker, unit_warrior, 18, 3, 0, sim_grid), 2)
	sim_resolver.schedule_action(unit_berserker, TargetCommand.new(unit_berserker, unit_warrior, 1), 5)

	# Goblin Archer (35 HP at (6, 5) - continuous actions without empty gaps):
	sim_resolver.schedule_action(unit_archer, TargetCommand.new(unit_archer, unit_warrior, 1), 1)
	sim_resolver.schedule_action(unit_archer, AttackCommand.new(unit_archer, unit_warrior, 14, 3, 0, sim_grid), 2)
	sim_resolver.schedule_action(unit_archer, TargetCommand.new(unit_archer, unit_warrior, 1), 5)
	sim_resolver.schedule_action(unit_archer, AttackCommand.new(unit_archer, unit_warrior, 14, 3, 0, sim_grid), 6)
	sim_resolver.schedule_action(unit_archer, DefendCommand.new(unit_archer, 0.4, 2, 1), 9)

	# Goblin Shaman (35 HP at (6, 4) - continuous actions without empty gaps):
	sim_resolver.schedule_action(unit_shaman, TargetCommand.new(unit_shaman, unit_warrior, 1), 1)
	sim_resolver.schedule_action(unit_shaman, AttackCommand.new(unit_shaman, unit_warrior, 12, 3, 0, sim_grid), 2)
	sim_resolver.schedule_action(unit_shaman, TargetCommand.new(unit_shaman, unit_warrior, 1), 5)
	sim_resolver.schedule_action(unit_shaman, AttackCommand.new(unit_shaman, unit_warrior, 12, 3, 0, sim_grid), 6)
	sim_resolver.schedule_action(unit_shaman, DefendCommand.new(unit_shaman, 0.5, 2, 1), 9)

	# Record Round 2 actions
	recorded_actions.clear()
	for a in sim_resolver.scheduled_actions:
		recorded_actions.append(CombatResolver.ScheduledAction.new(a.combatant, a.command, a.start_block, a.end_block))
	round_recorded_actions[2] = recorded_actions.duplicate()

	# Snapshot 0 for Round 2
	snapshots.clear()
	_capture_snapshot(0)
	round_snapshots[2] = snapshots.duplicate()
	viewed_timeline_round = 2
	_update_round_traversal_ui()

	_select_combatant(unit_warrior)
	_update_timeline_tracks_display()
	_update_combined_block_display(1)
	_sync_visuals_after_block()
	_update_all_roster_cards()

	_update_selected_unit_panel()
	_update_tactical_forecast_panel()
	if timeline_modal != null and timeline_modal.visible:
		_update_modal_tracks_display()


func _start_round_3() -> void:
	current_combat_round = 3
	max_unlocked_round = maxi(max_unlocked_round, 3)
	sim_resolver.finish_round()

	current_sim_block = 1
	replay_block = 1
	combined_selected_block = 1
	sim_finished = false
	is_auto_playing = false
	combat_phase = CombatPhase.PLANNING
	auto_play_timer.stop()
	btn_play.text = "▶"
	btn_play.tooltip_text = "Auto-Resolve Round 3"
	if btn_next_round:
		btn_next_round.visible = false

	# Setup Round 3 combatant baseline (Exact final coordinates of Round 2: zero movement between rounds)
	unit_warrior.hp = 35
	unit_warrior.set_stance(CombatTypes.StanceType.NEUTRAL)
	sim_grid.combatant_positions[unit_warrior] = Vector2i(5, 4)

	unit_shaman.hp = 15
	unit_shaman.set_stance(CombatTypes.StanceType.NEUTRAL)
	sim_grid.combatant_positions[unit_shaman] = Vector2i(6, 4)

	unit_archer.hp = 35
	unit_archer.set_stance(CombatTypes.StanceType.NEUTRAL)
	sim_grid.combatant_positions[unit_archer] = Vector2i(6, 5)

	unit_berserker.hp = 0  # Defeated in Round 2
	unit_skirmisher.hp = 0 # Defeated in Round 2
	unit_brawler.hp = 0    # Defeated in Round 1
	unit_scout.is_withdrawn = true
	unit_ranger.hp = 0

	# Prune defeated hostiles (Skirmisher, Berserker, Brawler, Scout) from roster cards
	for enemy: Combatant in [unit_skirmisher, unit_berserker, unit_brawler, unit_scout]:
		if roster_panel:
			roster_panel.prune_enemy(enemy)

	# Ensure surviving hostiles have roster cards
	for enemy: Combatant in [unit_shaman, unit_archer]:
		if roster_panel and not roster_panel.roster_cards.has(enemy):
			_create_roster_card(enemy, Color(0.9, 0.3, 0.3))
		elif roster_panel and roster_panel.roster_cards.has(enemy):
			@warning_ignore("unsafe_cast")
			var card := roster_panel.roster_cards[enemy] as Control
			if is_instance_valid(card):
				card.visible = true

	pending_block_actions.clear()
	var r3_entry := ActionLogEntry.new()
	r3_entry.round_num = 3
	r3_entry.block_num = 1
	r3_entry.ref_tag = "R3-B01"
	r3_entry.actor_name = "Warrior"
	r3_entry.actor_is_ally = true
	r3_entry.category = "EVENT"
	r3_entry.skill_name = "Round 3 Commenced"
	r3_entry.skill_subtype = "Final Showdown"
	r3_entry.outcome_tag = "ROUND 3"
	r3_entry.detail_text = "Warrior (35 HP) at (5, 4) against Shaman (15 HP) & Archer (35 HP)"
	r3_entry.status_text = "Round 3 Active"
	all_action_log_entries.append(r3_entry)
	active_log_entry_index = all_action_log_entries.size() - 1
	_update_active_log_cell()

	phase_badge_lbl.text = "ROUND 3: PLANNING PHASE (FINAL SHOWDOWN)"
	phase_badge_lbl.add_theme_color_override("font_color", Color(0.95, 0.8, 0.35))
	status_label.text = "Round 3: Final Showdown! Warrior confronts surviving Shaman & Archer. Auto-Resolve to conclude combat."

	_append_log("\n[color=#38bdf8][b]⚔⚔⚔ COMMENCING ROUND 3: FINAL SHOWDOWN ⚔⚔⚔[/b][/color]")
	_append_log("[color=#93c5fd]Warrior presses forward against the last two goblins. Victory is in sight![/color]")

	# Schedule Pre-Planned Commands for Round 3:
	# Warrior (Knight, at (5, 4)):
	#   Block 1-2: Decisive Slash on Shaman at (6, 4) (power 25) [2b] -> Shaman DEFEATED!
	#   Block 3: Tactical Advance East to (6, 4) [1b] (steps adjacent to Archer at (6, 5))
	#   Block 4-5: Defend (70% mitigation) [2b] (Deflects point-blank arrow!)
	#   Block 6: Target Archer [1b]
	#   Block 7-8: Vanguard Finishing Strike on Archer at (6, 5) (priority 1, power 40) [2b] -> Archer DEFEATED!
	#   Block 9-10: Hold Victory Stance [2b]
	sim_resolver.schedule_action(unit_warrior, AttackCommand.new(unit_warrior, unit_shaman, 25, 2, 0, sim_grid), 1)
	sim_resolver.schedule_action(unit_warrior, MoveCommand.new(unit_warrior, sim_grid, Vector2i(1, 0), 1), 3)
	sim_resolver.schedule_action(unit_warrior, DefendCommand.new(unit_warrior, 0.7, 2, 1), 4)
	sim_resolver.schedule_action(unit_warrior, TargetCommand.new(unit_warrior, unit_archer, 1), 6)
	sim_resolver.schedule_action(unit_warrior, AttackCommand.new(unit_warrior, unit_archer, 40, 2, 1, sim_grid), 7)
	sim_resolver.schedule_action(unit_warrior, HoldStanceCommand.new(unit_warrior, 0), 9)

	# Goblin Shaman (15 HP at (6, 4)):
	# Defeated at Block 2 by Warrior's Decisive Slash!
	sim_resolver.schedule_action(unit_shaman, TargetCommand.new(unit_shaman, unit_warrior, 1), 1)
	sim_resolver.schedule_action(unit_shaman, AttackCommand.new(unit_shaman, unit_warrior, 10, 2, 0, sim_grid), 1)

	# Goblin Archer (35 HP at (6, 5) - continuous actions without empty gaps):
	sim_resolver.schedule_action(unit_archer, TargetCommand.new(unit_archer, unit_warrior, 1), 1)
	sim_resolver.schedule_action(unit_archer, AttackCommand.new(unit_archer, unit_warrior, 14, 3, 0, sim_grid), 2)
	sim_resolver.schedule_action(unit_archer, TargetCommand.new(unit_archer, unit_warrior, 1), 5)
	sim_resolver.schedule_action(unit_archer, AttackCommand.new(unit_archer, unit_warrior, 16, 3, 0, sim_grid), 6)
	sim_resolver.schedule_action(unit_archer, DefendCommand.new(unit_archer, 0.5, 2, 1), 9)

	# Record Round 3 actions
	recorded_actions.clear()
	for a in sim_resolver.scheduled_actions:
		recorded_actions.append(CombatResolver.ScheduledAction.new(a.combatant, a.command, a.start_block, a.end_block))
	round_recorded_actions[3] = recorded_actions.duplicate()

	# Snapshot 0 for Round 3
	snapshots.clear()
	_capture_snapshot(0)
	round_snapshots[3] = snapshots.duplicate()
	viewed_timeline_round = 3
	_update_round_traversal_ui()

	_select_combatant(unit_warrior)
	_update_timeline_tracks_display()
	_update_combined_block_display(1)
	_sync_visuals_after_block()
	_update_all_roster_cards()

	_update_selected_unit_panel()
	_update_tactical_forecast_panel()
	if timeline_modal != null and timeline_modal.visible:
		_update_modal_tracks_display()


# ==============================================================================
# RULE-DRIVEN ACTION LOG STREAM & DYNAMIC CELL TEMPLATES
# ==============================================================================

func _build_entry_from_action(act: CombatResolver.ScheduledAction, round_num: int, block_num: int) -> ActionLogEntry:
	var entry := ActionLogEntry.new()
	entry.round_num = round_num
	entry.block_num = block_num
	entry.ref_tag = "R%d-B%02d" % [round_num, block_num]
	entry.actor_name = act.combatant.name
	entry.actor_is_ally = sim_allies.has(act.combatant)

	if act.command is AttackCommand:
		var cmd := act.command as AttackCommand
		entry.category = "ATTACK"
		var target_c: Combatant = cmd.target
		if target_c:
			entry.target_name = target_c.name
			entry.target_is_ally = sim_allies.has(target_c)
		else:
			entry.target_name = "Target"

		var atk_range: int = cmd.source.attack_range
		if atk_range > 1:
			entry.skill_subtype = "Ranged Projectile"
			entry.skill_name = "Quick Snipe" if entry.actor_is_ally else "Longbow Volley"
		else:
			entry.skill_subtype = "Melee Physical"
			entry.skill_name = "Heavy Slash" if entry.actor_is_ally else "Power Strike"

		entry.value_num = cmd.power
		if target_c:
			if not target_c.is_alive():
				entry.outcome_tag = "DEFEATED"
			elif target_c.defense_mitigation_ratio > 0.0:
				entry.outcome_tag = "MITIGATED"
			else:
				entry.outcome_tag = "HIT"
			entry.status_text = "Target HP: %d/%d" % [target_c.hp, target_c.max_hp]
		else:
			entry.outcome_tag = "HIT"
		entry.detail_text = "Power: %d | Range: %d" % [cmd.power, atk_range]

	elif act.command is MoveCommand:
		var cmd := act.command as MoveCommand
		entry.category = "MOVE"
		entry.skill_name = "Sprint" if cmd.steps > 1 else "Tactical Move"
		entry.skill_subtype = "Sprint Relocation" if cmd.steps > 1 else "Standard Step"
		entry.value_num = cmd.steps
		entry.coord_to = sim_grid.get_position(act.combatant)
		if act.combatant.is_withdrawn:
			entry.outcome_tag = "WITHDRAWN"
			entry.status_text = "Safely exited battle grid"
		else:
			entry.outcome_tag = "MOVED"
			entry.status_text = "Position: %s" % str(entry.coord_to)
		entry.detail_text = "Stepped %d tiles (%s)" % [cmd.steps, str(cmd.direction)]

	elif act.command is DefendCommand:
		var cmd := act.command as DefendCommand
		entry.category = "DEFEND"
		entry.skill_name = "Defensive Stance"
		entry.skill_subtype = "Shield Guard"
		entry.value_num = int(cmd.mitigation_ratio * 100.0)
		entry.outcome_tag = "%d%% MITIGATION" % entry.value_num
		entry.detail_text = "Damage mitigation posture active"
		entry.status_text = "Duration: %d Blocks" % cmd.block_cost

	elif act.command is TargetCommand:
		var cmd := act.command as TargetCommand
		entry.category = "TARGET"
		entry.skill_name = "Target Lock"
		entry.skill_subtype = "Engagement Focus"
		var target_c: Combatant = null
		if cmd.target_value is Combatant:
			@warning_ignore("unsafe_cast")
			target_c = cmd.target_value as Combatant
		if target_c != null:
			entry.target_name = target_c.name
			entry.target_is_ally = sim_allies.has(target_c)
		elif cmd.target_value != null:
			entry.target_name = str(cmd.target_value)
		else:
			entry.target_name = "None"
		entry.outcome_tag = "LOCKED"
		entry.detail_text = "Assigned tactical target focus"
		entry.status_text = "Target Locked"

	elif act.command is FocusCommand:
		var cmd := act.command as FocusCommand
		entry.category = "BUFF"
		entry.skill_name = "Focus Skill"
		entry.skill_subtype = "Attack Multiplier"
		entry.value_num = int((cmd.boost_multiplier - 1.0) * 100.0)
		entry.outcome_tag = "+%d%% ATK" % entry.value_num
		entry.detail_text = "+%d%% Attack Power for %d Round(s)" % [entry.value_num, cmd.duration_rounds]
		entry.status_text = "Empowered"

	elif act.command is HoldStanceCommand:
		entry.category = "DEFEND"
		entry.skill_name = "Hold Stance"
		entry.skill_subtype = "Posture Maintenance"
		entry.outcome_tag = "MAINTAINED"
		entry.detail_text = "Current stance maintained"
		entry.status_text = "Active"

	else:
		entry.category = "EVENT"
		entry.skill_name = act.command.command_name
		entry.skill_subtype = "Standard Action"
		entry.outcome_tag = "EXECUTED"
		entry.detail_text = act.command.command_name

	return entry


func _find_combatant_by_name(p_name: String) -> Combatant:
	for c: Combatant in sim_allies:
		if c.name == p_name:
			return c
	for c: Combatant in sim_enemies:
		if c.name == p_name:
			return c
	return null


func _update_active_log_cell() -> void:
	if action_log_dock:
		action_log_dock.update_active_cell(all_action_log_entries, active_log_entry_index)


func _on_log_prev_pressed() -> void:
	if active_log_entry_index > 0:
		active_log_entry_index -= 1
		_update_active_log_cell()
		var entry := all_action_log_entries[active_log_entry_index]
		_trigger_action_impact_fx(entry)
		if combat_phase == CombatPhase.REPLAY and entry.block_num < snapshots.size():
			replay_block = entry.block_num
			_restore_snapshot(snapshots[replay_block])
			_update_timeline_tracks_display()
			_update_combined_block_display(replay_block)


func _on_log_next_pressed() -> void:
	if active_log_entry_index < all_action_log_entries.size() - 1:
		active_log_entry_index += 1
		_update_active_log_cell()
		var entry := all_action_log_entries[active_log_entry_index]
		_trigger_action_impact_fx(entry)
		if combat_phase == CombatPhase.REPLAY and entry.block_num < snapshots.size():
			replay_block = entry.block_num
			_restore_snapshot(snapshots[replay_block])
			_update_timeline_tracks_display()
			_update_combined_block_display(replay_block)


func _toggle_log_modal(open: bool) -> void:
	if log_modal == null:
		return
	if open:
		log_modal.set_entries(all_action_log_entries)
		log_modal.open_modal()
	else:
		log_modal.close_modal()



func _on_log_modal_cell_clicked(entry: ActionLogEntry) -> void:
	var idx: int = all_action_log_entries.find(entry)
	if idx != -1:
		active_log_entry_index = idx
		_update_active_log_cell()
		_trigger_action_impact_fx(entry)
		var actor := _find_combatant_by_name(entry.actor_name)
		if actor != null:
			_select_combatant(actor)
		if combat_phase == CombatPhase.REPLAY and entry.block_num < snapshots.size():
			replay_block = entry.block_num
			_restore_snapshot(snapshots[replay_block])
			_update_timeline_tracks_display()
			_update_combined_block_display(replay_block)


func _trigger_action_impact_fx(entry: ActionLogEntry) -> void:
	if not arena_ui:
		return
	var actor := _find_combatant_by_name(entry.actor_name)
	var target := _find_combatant_by_name(entry.target_name)
	arena_ui.trigger_action_fx(entry, actor, target)


func _on_step_prev_pressed() -> void:
	if combat_phase == CombatPhase.REPLAY:
		var target_b: int = clampi(replay_block - 1, 1, TOTAL_BLOCKS)
		_on_ruler_block_clicked(target_b)
	else:
		if active_log_entry_index > 0:
			_on_log_prev_pressed()


func _on_step_next_pressed() -> void:
	if combat_phase == CombatPhase.REPLAY:
		var target_b: int = clampi(replay_block + 1, 1, TOTAL_BLOCKS)
		_on_ruler_block_clicked(target_b)
	else:
		_step_simulation()


func _on_play_round_pressed() -> void:
	if is_auto_playing:
		is_auto_playing = false
		auto_play_timer.stop()
		btn_play.text = "▶"
	else:
		if combat_phase == CombatPhase.REPLAY:
			# Autoplay round from Block 1
			replay_block = 1
			current_sim_block = 1
			_on_ruler_block_clicked(1)
		is_auto_playing = true
		btn_play.text = "⏸"
		auto_play_timer.start()


func _on_auto_play_tick() -> void:
	if combat_phase == CombatPhase.REPLAY:
		if replay_block < TOTAL_BLOCKS:
			_on_step_next_pressed()
		else:
			is_auto_playing = false
			auto_play_timer.stop()
			btn_play.text = "▶"
	else:
		_step_simulation()


func _on_reset_active_round_pressed() -> void:
	if current_combat_round == 1:
		_init_interactive_simulation()
	elif current_combat_round == 2:
		_start_round_2()
	elif current_combat_round == 3:
		_start_round_3()


func _on_reset_game_pressed() -> void:
	_init_interactive_simulation()


# Backward compatibility wrappers
func _on_step_block_pressed() -> void:
	_on_step_next_pressed()

func _on_play_toggle_pressed() -> void:
	_on_play_round_pressed()

func _on_reset_pressed() -> void:
	_on_reset_game_pressed()

func _on_rewind_pressed() -> void:
	_on_step_prev_pressed()


func _on_toggle_view_mode_pressed() -> void:
	_toggle_timeline_modal(not (timeline_modal != null and timeline_modal.visible))


func _on_ruler_block_clicked(b: int) -> void:
	replay_block = b
	current_sim_block = b
	# Directly traverse through battle state for clicked block
	var snaps: Array = round_snapshots.get(viewed_timeline_round, snapshots)
	if b >= 0 and b < snaps.size():
		@warning_ignore("unsafe_cast")
		_restore_snapshot(snaps[b] as BlockSnapshot)
	elif not snaps.is_empty():
		@warning_ignore("unsafe_cast")
		_restore_snapshot(snaps[0] as BlockSnapshot)

	status_label.text = "Round %d • Block %02d State inspected." % [viewed_timeline_round, b]
	_update_timeline_tracks_display()
	_update_combined_block_display(b)
	if timeline_modal != null and timeline_modal.visible:
		_update_modal_tracks_display()


func _navigate_timeline_round(delta: int) -> void:
	var target_round: int = clampi(viewed_timeline_round + delta, 1, max_unlocked_round)
	if target_round != viewed_timeline_round:
		_select_timeline_round(target_round)


func _update_round_traversal_ui() -> void:
	# Requirement 3: Disable arrows if data not exposed (e.g. on load, R0 and R2 disabled)
	var can_go_prev: bool = (viewed_timeline_round > 1)
	var can_go_next: bool = (viewed_timeline_round < max_unlocked_round)

	if timeline_dock != null:
		timeline_dock.update_round_arrows(can_go_prev, can_go_next)
	elif btn_prev_round != null:
		btn_prev_round.disabled = not can_go_prev
	if btn_next_round_arrow != null and timeline_dock == null:
		btn_next_round_arrow.disabled = not can_go_next

	if timeline_modal != null and timeline_modal.visible:
		_update_modal_tracks_display()


func _select_timeline_round(rd: int) -> void:
	viewed_timeline_round = rd
	@warning_ignore("unsafe_cast")
	recorded_actions = round_recorded_actions.get(rd, []) as Array[CombatResolver.ScheduledAction]
	@warning_ignore("unsafe_cast")
	snapshots = round_snapshots.get(rd, []) as Array[BlockSnapshot]
	_update_round_traversal_ui()
	_update_timeline_tracks_display()
	if timeline_modal != null and timeline_modal.visible:
		_update_modal_tracks_display()

	var snaps: Array = round_snapshots.get(rd, [])
	var target_b: int = replay_block if combat_phase == CombatPhase.REPLAY else 1
	if target_b >= 0 and target_b < snaps.size():
		@warning_ignore("unsafe_cast")
		_restore_snapshot(snaps[target_b] as BlockSnapshot)
	elif not snaps.is_empty():
		@warning_ignore("unsafe_cast")
		_restore_snapshot(snaps[0] as BlockSnapshot)
	_update_combined_block_display(target_b)
	status_label.text = "Viewing Round %d Timeline." % rd


# ==============================================================================
# SECTION 8: FULL-SCREEN POPOUT MODAL TIMELINE SUITE (Extracted Component Delegation)
# ==============================================================================

func _update_modal_tracks_display() -> void:
	if timeline_modal == null or not timeline_modal.visible:
		return

	var active_playhead: int = replay_block if combat_phase == CombatPhase.REPLAY else current_sim_block

	# Query snapshot for active playhead in viewed round
	var snap: BlockSnapshot = null
	var snaps: Array = round_snapshots.get(viewed_timeline_round, snapshots)
	if active_playhead >= 0 and active_playhead < snaps.size():
		@warning_ignore("unsafe_cast")
		snap = snaps[active_playhead] as BlockSnapshot

	# Filter active hostiles for viewed round
	var active_enemies_for_view: Array[Combatant] = []
	var is_r1: bool = (viewed_timeline_round == 1)
	var reinforcements_active: bool = (snap.reinforcements_spawned if snap != null else reinforcements_spawned)

	for enemy in sim_enemies:
		if is_r1:
			if enemy in [unit_berserker, unit_skirmisher, unit_archer, unit_shaman]:
				if reinforcements_active:
					active_enemies_for_view.append(enemy)
			else:
				active_enemies_for_view.append(enemy)
		elif viewed_timeline_round == 2:
			if not (enemy in [unit_brawler, unit_scout]):
				active_enemies_for_view.append(enemy)
		elif viewed_timeline_round == 3:
			if enemy in [unit_shaman, unit_archer]:
				active_enemies_for_view.append(enemy)

	var can_go_prev: bool = (viewed_timeline_round > 1)
	var can_go_next: bool = (viewed_timeline_round < max_unlocked_round)

	timeline_modal.update_view(
		viewed_timeline_round,
		active_playhead,
		combat_phase == CombatPhase.REPLAY,
		sim_allies,
		active_enemies_for_view,
		snap,
		can_go_prev,
		can_go_next
	)


func _toggle_timeline_modal(open: bool) -> void:
	if timeline_modal == null:
		return
	if open:
		timeline_modal.open_modal()
		_update_modal_tracks_display()
	else:
		timeline_modal.close_modal()
		_update_timeline_tracks_display()
		_update_combined_block_display(combined_selected_block)


func _unhandled_input(event: InputEvent) -> void:
	var key_event := event as InputEventKey
	if key_event != null and key_event.pressed and not key_event.echo:
		if key_event.keycode == KEY_ESCAPE:
			if log_modal != null and log_modal.visible:
				_toggle_log_modal(false)
				get_viewport().set_input_as_handled()
				return
			elif timeline_modal != null and timeline_modal.visible:
				_toggle_timeline_modal(false)
				get_viewport().set_input_as_handled()
				return
		elif key_event.keycode == KEY_L:
			if log_modal != null:
				_toggle_log_modal(not log_modal.visible)
				get_viewport().set_input_as_handled()
				return
		elif key_event.keycode == KEY_F:
			if timeline_modal != null:
				_toggle_timeline_modal(not timeline_modal.visible)
				get_viewport().set_input_as_handled()
				return
		elif key_event.keycode == KEY_BRACKETLEFT:
			_on_log_prev_pressed()
			get_viewport().set_input_as_handled()
			return
		elif key_event.keycode == KEY_BRACKETRIGHT:
			_on_log_next_pressed()
			get_viewport().set_input_as_handled()
			return
		elif key_event.keycode == KEY_Q or key_event.keycode == KEY_LEFT:
			_navigate_combined_block(-1)
		elif key_event.keycode == KEY_E or key_event.keycode == KEY_RIGHT:
			_navigate_combined_block(1)
		elif key_event.keycode == KEY_SPACE:
			_on_play_toggle_pressed()
		elif key_event.keycode == KEY_TAB:
			_on_toggle_view_mode_pressed()


func _append_log(text: String) -> void:
	if log_rich_text:
		log_rich_text.append_text(text + "\n")


func _on_resolver_log_emitted(msg: String) -> void:
	var formatted: String = msg
	if msg.begins_with("[Attack]"):
		if "DEFEATED" in msg:
			formatted = "[color=#f87171]⚔ [/color][b][color=#ef4444]" + msg.substr(9) + "[/color][/b]"
		else:
			formatted = "[color=#f87171]⚔ " + msg.substr(9) + "[/color]"
	elif msg.begins_with("[Attack Out of Range]") or msg.begins_with("[Attack Miss]"):
		var tag_len: int = 22 if msg.begins_with("[Attack Out of Range]") else 14
		formatted = "[color=#fb7185]❌ [i]" + msg.substr(tag_len) + "[/i][/color]"
	elif msg.begins_with("[Move]") or msg.begins_with("[Sprint]"):
		var is_sprint: bool = msg.begins_with("[Sprint]")
		var tag_len: int = 9 if is_sprint else 7
		formatted = "[color=#38bdf8]➜ [b]" + (msg.left(tag_len - 1).strip_edges()) + "[/b]: " + msg.substr(tag_len) + "[/color]"
	elif msg.begins_with("[Move Blocked]") or msg.begins_with("[Sprint Blocked]"):
		formatted = "[color=#f59e0b]⚠ " + msg + "[/color]"
	elif msg.begins_with("[Withdrawal]"):
		formatted = "[b][color=#fbbf24]⚑ " + msg.substr(13) + "[/color][/b]"
	elif msg.begins_with("[Skill:"):
		formatted = "[color=#c084fc]✦ [b]" + msg + "[/b][/color]"
	elif msg.begins_with("[Defend]"):
		formatted = "[color=#60a5fa]🛡 [b]" + msg + "[/b][/color]"
	elif msg.begins_with("[Target]"):
		formatted = "[color=#34d399]🎯 " + msg.substr(9) + "[/color]"
	elif msg.begins_with("[Action Started]"):
		formatted = "[color=#94a3b8]⏳ " + msg.substr(17) + "[/color]"
	elif msg.begins_with("[Cancelled]"):
		formatted = "[color=#ef4444]⛔ " + msg.substr(12) + "[/color]"
	elif msg.strip_edges().begins_with("[") and "Status:" in msg:
		formatted = "[color=#64748b]" + msg + "[/color]"
	else:
		formatted = "[color=#e2e8f0]" + msg + "[/color]"

	_append_log("  " + formatted)


# ==============================================================================
# SECTION 4: HEADLESS AUTOMATED INTEGRATION TESTS
# ==============================================================================

func _run_warrior_vs_3_goblins_battle() -> bool:
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


func _run_combat_grid_and_withdrawal_test() -> bool:
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


func _run_resolution_feed_logging_test() -> bool:
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


func _run_goblin_ambush_and_casualty_test() -> bool:
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