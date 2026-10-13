class_name CombatSession
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat Session (headless state for the combat UI)
# Owns one battle: the encounter's combatants, grid and resolver, round flow,
# block-by-block resolution, per-block snapshots for replay, multi-round
# timeline traversal, selection/hover, and the action log.
#
# UI panels never talk to each other; they read this session and call its
# actions. Every state change ends in `changed`. Visual one-shots (impact FX,
# hover) use dedicated signals. No Node / scene-tree dependencies, so the whole
# battle flow is headless-testable.
# ==============================================================================

signal changed()
signal mounted()                              # encounter (re)built: views rebuild tokens/cards
signal log_appended(entry: ActionLogEntry)
signal log_reset()
signal fx_requested(entry: ActionLogEntry)    # animate an action (attack lunge, move, buff...)
signal hover_changed(combatant: Combatant)    # null = hover cleared
signal text_logged(bbcode: String)

enum Phase { PLANNING, RESOLVING, REPLAY }

const TOTAL_BLOCKS: int = 10
const MAX_TEXT_LOG: int = 400
const NO_POSITION: Vector2i = Vector2i(-1, -1)

const COL_PLANNING: Color = Color(0.95, 0.8, 0.35)
const COL_RESOLVING: Color = Color(0.4, 0.8, 1.0)
const COL_REPLAY: Color = Color(0.3, 0.9, 0.5)
const COL_VICTORY: Color = Color(0.4, 0.9, 1.0)

var encounter: CombatEncounter = null
var grid: CombatGrid = null
var resolver: CombatResolver = null
var allies: Array[Combatant] = []
var enemies: Array[Combatant] = []

# Round & block flow
var current_round: int = 1
var current_block: int = 1          # next block to resolve during RESOLVING
var replay_block: int = 1           # playhead while replaying / viewing a past round
var inspect_block: int = -1         # snapshot inspected during the live round (-1 = live)
var phase: Phase = Phase.PLANNING
var combat_over: bool = false
var is_auto_playing: bool = false

# Multi-round timeline archive
var viewed_round: int = 1
var max_unlocked_round: int = 1
var round_actions: Dictionary = {}    # int -> Array[CombatResolver.ScheduledAction]
var round_snapshots: Dictionary = {}  # int -> Array[BlockSnapshot]

# Action log
var log_entries: Array[ActionLogEntry] = []
var active_log_index: int = -1
var text_log: Array[String] = []

# Selection & presentation (view-model strings shown by the playback panel)
var selected_combatant: Combatant = null
var focused_hero: Combatant = null
var hovered_combatant: Combatant = null
var phase_text: String = "PLANNING PHASE"
var phase_color: Color = COL_PLANNING
var status_text: String = ""
var next_round_label: String = ""    # empty = no round advance available

# Planning (encounters with player_planning): the party's plan for the live round.
var plan: CombatPlan = CombatPlan.new()
var armed_kind: int = -1                  # PlanEntry.Kind chosen in the palette, -1 = none
var armed_dir: Vector2i = Vector2i(1, 0)  # direction for armed Move / Sprint

var _pending_actions: Array[CombatResolver.ScheduledAction] = []

func _init(p_encounter: CombatEncounter) -> void:
	encounter = p_encounter

# ------------------------------------------------------------------------------
# 1. Battle lifecycle
# ------------------------------------------------------------------------------

func start() -> void:
	dispose()
	is_auto_playing = false
	combat_over = false
	max_unlocked_round = 1
	round_actions.clear()
	round_snapshots.clear()
	log_entries.clear()
	active_log_index = -1
	text_log.clear()
	selected_combatant = null
	focused_hero = null
	hovered_combatant = null

	encounter.build()
	grid = encounter.grid
	allies = encounter.allies
	enemies = encounter.enemies
	resolver = CombatResolver.new()
	resolver.grid = grid
	resolver.log_emitted.connect(_on_resolver_log)
	for c: Combatant in allies:
		resolver.register_combatant(c)
	for c: Combatant in enemies:
		resolver.register_combatant(c)

	log_reset.emit()
	mounted.emit()
	_begin_round(1)

func _begin_round(round_num: int, roll_over: bool = true) -> void:
	if round_num > 1 and roll_over:
		resolver.finish_round()
	elif not roll_over:
		resolver.scheduled_actions.clear()
	current_round = round_num
	max_unlocked_round = maxi(max_unlocked_round, round_num)
	viewed_round = round_num
	current_block = 1
	replay_block = 1
	inspect_block = -1
	phase = Phase.PLANNING
	is_auto_playing = false
	next_round_label = ""
	_pending_actions.clear()

	var intro: Dictionary = encounter.prepare_round(round_num)
	var raw_entry: Variant = intro.get("entry", null)
	if raw_entry is ActionLogEntry:
		var intro_entry: ActionLogEntry = raw_entry
		_append_entry(intro_entry, false)
	_set_phase(str(intro.get("phase", "ROUND %d: PLANNING PHASE" % round_num)), COL_PLANNING)
	status_text = str(intro.get("status", "Round %d: plan actions, then resolve." % round_num))
	_log_text(str(intro.get("log", "")))

	plan.clear_all()
	armed_kind = -1
	encounter.schedule_round(resolver, round_num)
	_refresh_archive()

	var snaps: Array[BlockSnapshot] = []
	round_snapshots[round_num] = snaps
	_capture_snapshot(0)

	var first_active: Combatant = _first_active(allies)
	focused_hero = first_active
	selected_combatant = first_active
	_notify()

# Breaks reference cycles of the current battle (units locking onto each other via
# active_target) so a finished or abandoned battle can be freed. Safe to repeat.
func dispose() -> void:
	for c: Combatant in allies:
		c.active_target = null
	for c: Combatant in enemies:
		c.active_target = null
	if resolver != null:
		resolver.scheduled_actions.clear()
	round_actions.clear()
	round_snapshots.clear()
	_pending_actions.clear()
	plan.clear_all()

func reset_round() -> void:
	if current_round == 1:
		start()
		return
	_truncate_log_from_round(current_round)
	combat_over = false
	if encounter.player_planning:
		# Rounds carry state over, so rewind to how this round started.
		var start_snap: BlockSnapshot = snapshot_at(current_round, 0)
		if start_snap != null:
			_restore_snapshot(start_snap)
		_begin_round(current_round, false)
	else:
		_begin_round(current_round)

func reset_battle() -> void:
	start()

func can_advance_round() -> bool:
	return phase == Phase.REPLAY and not combat_over and current_round < encounter.max_rounds and current_round == max_unlocked_round

func advance_round() -> void:
	if can_advance_round():
		_begin_round(current_round + 1)

# ------------------------------------------------------------------------------
# 2. Resolution & playback
# ------------------------------------------------------------------------------

func is_live_view() -> bool:
	return viewed_round == current_round and phase != Phase.REPLAY

func step_next() -> void:
	if is_live_view():
		_step_resolution()
	else:
		select_block(replay_block + 1)

func step_prev() -> void:
	if is_live_view():
		log_prev()
	else:
		select_block(replay_block - 1)

func _step_resolution() -> void:
	inspect_block = -1
	if phase == Phase.PLANNING:
		_commit_plan()
		phase = Phase.RESOLVING
		_set_phase("RESOLVING COMBAT", COL_RESOLVING)
		current_block = 1
		_pending_actions.clear()

	if _pending_actions.is_empty():
		var trigger: ActionLogEntry = encounter.on_block_start(current_round, current_block)
		if trigger != null:
			_append_entry(trigger, true)
			status_text = "%s - Block %d" % [trigger.skill_name, current_block]
			_notify()
			return
		_log_text("[color=#fde047][b]━━━ BLOCK %d RESOLUTION (ROUND %d) ━━━[/b][/color]" % [current_block, current_round])
		_pending_actions = resolver.prepare_block(current_block)

	if not _pending_actions.is_empty():
		var act: CombatResolver.ScheduledAction = _pending_actions.pop_front()
		resolver.execute_single_action(act)
		_append_entry(CombatLogBuilder.from_action(act, current_round, current_block, allies, grid), true)

	if _pending_actions.is_empty():
		resolver.block_completed.emit(current_block)
		_capture_snapshot(current_block)
		var over: bool = resolver.is_combat_resolved(allies, enemies)
		if over or current_block >= TOTAL_BLOCKS:
			_conclude_round(over)
			return
		current_block += 1
		status_text = "Ready for Block %d." % current_block
	_notify()

func _conclude_round(over: bool) -> void:
	combat_over = over
	phase = Phase.REPLAY
	replay_block = current_block
	is_auto_playing = false
	var summary: Dictionary = encounter.round_summary(current_round, over)
	_set_phase(str(summary.get("phase", "ROUND %d CONCLUDED" % current_round)), COL_VICTORY if over else COL_REPLAY)
	status_text = str(summary.get("status", "Round %d concluded. Review the replay." % current_round))
	_log_text(str(summary.get("log", "")))
	next_round_label = encounter.advance_label(current_round) if can_advance_round() else ""
	_notify()

func toggle_auto_play() -> void:
	if is_auto_playing:
		is_auto_playing = false
	else:
		if not is_live_view():
			replay_block = 1
		is_auto_playing = true
	_notify()

# Called by the host's timer while auto-playing.
func auto_play_tick() -> void:
	if not is_auto_playing:
		return
	if is_live_view():
		_step_resolution()
	elif replay_block < TOTAL_BLOCKS:
		select_block(replay_block + 1)
	else:
		is_auto_playing = false
		_notify()

# Inspect the state at block `b` of the viewed round (replay, past round, or an
# already-resolved block of the live round).
func select_block(b: int) -> void:
	var block: int = clampi(b, 0, TOTAL_BLOCKS)
	if is_live_view():
		if block < _snapshots_for(viewed_round).size():
			inspect_block = block
			status_text = "Round %d • Block %02d state inspected (live round continues on Step)." % [viewed_round, block]
		else:
			status_text = "Block %d has not resolved yet." % block
	else:
		replay_block = maxi(1, block)
		status_text = "Round %d • Block %02d state inspected." % [viewed_round, replay_block]
	_notify()

func select_round(round_num: int) -> void:
	var target: int = clampi(round_num, 1, max_unlocked_round)
	if target == viewed_round:
		return
	viewed_round = target
	inspect_block = -1
	if not is_live_view():
		replay_block = clampi(replay_block, 1, TOTAL_BLOCKS)
	status_text = "Viewing Round %d Timeline." % target
	_notify()

func navigate_round(delta: int) -> void:
	select_round(viewed_round + delta)

func can_view_prev_round() -> bool:
	return viewed_round > 1

func can_view_next_round() -> bool:
	return viewed_round < max_unlocked_round

# Block highlighted on rulers / timelines.
func playhead() -> int:
	if is_live_view():
		return inspect_block if inspect_block >= 0 else current_block
	return replay_block

# Snapshot to render, or null for the live state.
func display_snapshot() -> BlockSnapshot:
	if is_live_view():
		return snapshot_at(viewed_round, inspect_block) if inspect_block >= 0 else null
	var snaps: Array[BlockSnapshot] = _snapshots_for(viewed_round)
	if snaps.is_empty():
		return null
	return snaps[clampi(replay_block, 0, snaps.size() - 1)]

func snapshot_at(round_num: int, block: int) -> BlockSnapshot:
	var snaps: Array[BlockSnapshot] = _snapshots_for(round_num)
	if block < 0 or block >= snaps.size():
		return null
	return snaps[block]

func _snapshots_for(round_num: int) -> Array[BlockSnapshot]:
	var raw: Variant = round_snapshots.get(round_num, null)
	if raw is Array:
		var snaps: Array[BlockSnapshot] = []
		var raw_snaps: Array = raw
		snaps.assign(raw_snaps)
		return snaps
	var empty: Array[BlockSnapshot] = []
	return empty

func recorded_actions(round_num: int = -1) -> Array[CombatResolver.ScheduledAction]:
	var key: int = viewed_round if round_num < 0 else round_num
	var actions: Array[CombatResolver.ScheduledAction] = []
	var raw: Variant = round_actions.get(key, null)
	if raw is Array:
		var raw_actions: Array = raw
		actions.assign(raw_actions)
	return actions

func find_recorded_action(combatant: Combatant, block: int) -> CombatResolver.ScheduledAction:
	for a: CombatResolver.ScheduledAction in recorded_actions():
		if a.combatant == combatant and a.start_block == block:
			return a
	return null

# ------------------------------------------------------------------------------
# 2b. Planning (player-planned encounters)
# The party's commands are placed on the timeline during the PLANNING phase and
# scheduled together with the enemies' (AI-planned) commands when resolution starts.
# ------------------------------------------------------------------------------

func is_planning() -> bool:
	return encounter.player_planning and phase == Phase.PLANNING and is_live_view() and not combat_over

func plan_power(actor: Combatant) -> int:
	return encounter.profile_of(actor).power

# The enemy an Attack is aimed at: the selected hostile, else the closest one.
func plan_target_for(actor: Combatant) -> Combatant:
	if selected_combatant != null and not is_ally(selected_combatant) and selected_combatant.is_active() and grid.has_combatant(selected_combatant):
		return selected_combatant
	var best: Combatant = null
	var best_dist: int = 999
	for e: Combatant in enemies:
		if e.is_active() and grid.has_combatant(e) and grid.has_combatant(actor):
			var d: int = grid.get_distance(grid.get_position(actor), grid.get_position(e))
			if d < best_dist:
				best_dist = d
				best = e
	return best

func arm(kind: int, dir: Vector2i = Vector2i.ZERO) -> void:
	if armed_kind == kind and (dir == Vector2i.ZERO or dir == armed_dir):
		armed_kind = -1
	else:
		armed_kind = kind
		if dir != Vector2i.ZERO:
			armed_dir = dir
	_notify()

# Short name of the armed palette command ("Attack", "Move →"), "" when none.
func armed_label() -> String:
	if armed_kind < 0:
		return ""
	var kind: PlanEntry.Kind = armed_kind as PlanEntry.Kind
	var text: String = PlanEntry.kind_name(kind)
	if PlanEntry.needs_direction(kind):
		text += " " + PlanEntry.dir_arrow(armed_dir)
	return text

func disarm() -> void:
	armed_kind = -1
	_notify()

# Places the armed command for the focused hero at `block` (replacing overlaps).
func place_armed(block: int) -> void:
	if not is_planning() or focused_hero == null:
		return
	if armed_kind < 0:
		status_text = "Pick a command from the palette first."
		_notify()
		return
	plan_add(focused_hero, armed_kind as PlanEntry.Kind, block, armed_dir)

# Returns "" on success, otherwise the reason it was rejected (also shown as status).
func plan_add(actor: Combatant, kind: PlanEntry.Kind, block: int, dir: Vector2i = Vector2i.ZERO, target: Combatant = null) -> String:
	if not is_planning():
		return "Planning is closed."
	var aim: Combatant = target
	if PlanEntry.needs_target(kind) and aim == null:
		aim = plan_target_for(actor)
	var entry: PlanEntry = PlanEntry.create(kind, block, dir if PlanEntry.needs_direction(kind) else Vector2i.ZERO, aim, plan_power(actor))
	var problem: String = plan.add(actor, entry, true)
	if problem.is_empty():
		var aim_text: String = " → %s" % aim.name if aim != null and PlanEntry.needs_target(kind) else ""
		status_text = "%s: %s%s at block %d (AP left %d)." % [actor.name, PlanEntry.kind_name(kind), aim_text, block, plan.ap_left(actor)]
		_refresh_archive()
	else:
		status_text = problem
	_notify()
	return problem

func plan_remove_at(actor: Combatant, block: int) -> void:
	if is_planning() and plan.remove_at(actor, block):
		status_text = "%s: cleared block %d." % [actor.name, block]
		_refresh_archive()
		_notify()

func plan_clear(actor: Combatant) -> void:
	if not is_planning():
		return
	plan.clear(actor)
	status_text = "%s: plan cleared." % actor.name
	_refresh_archive()
	_notify()

# Lets the AI plan `actor`'s round (the player can then tweak it).
func plan_auto(actor: Combatant) -> void:
	if not is_planning() or actor == null or not actor.is_active():
		return
	plan.set_entries(actor, encounter.plan_for(actor))
	status_text = "%s: auto-planned %d command(s)." % [actor.name, plan.entries_for(actor).size()]
	_refresh_archive()
	_notify()

func plan_auto_all() -> void:
	if not is_planning():
		return
	for a: Combatant in allies:
		if a.is_active():
			plan.set_entries(a, encounter.plan_for(a))
	status_text = "Party auto-planned. Adjust the timeline or resolve."
	_refresh_archive()
	_notify()

func _commit_plan() -> void:
	if encounter.player_planning:
		plan.commit(resolver, grid)
	armed_kind = -1
	_refresh_archive(false)

# Timeline copy of the live round's commands. While planning it also includes
# the party's unsent plan so the timeline shows it.
func _refresh_archive(with_plan: bool = true) -> void:
	var archived: Array[CombatResolver.ScheduledAction] = []
	for a: CombatResolver.ScheduledAction in resolver.scheduled_actions:
		archived.append(CombatResolver.ScheduledAction.new(a.combatant, a.command, a.start_block, a.end_block))
	if with_plan and encounter.player_planning and phase == Phase.PLANNING:
		archived.append_array(plan.preview_actions(grid))
	round_actions[current_round] = archived

# Rewinds unit state to a snapshot (used to redo a planned round).
func _restore_snapshot(snap: BlockSnapshot) -> void:
	for c: Combatant in resolver.registered_combatants:
		c.hp = int(snap.hps.get(c, c.hp))
		var withdrawn_raw: Variant = snap.is_withdrawn.get(c, false)
		c.is_withdrawn = withdrawn_raw is bool and withdrawn_raw
		if not c.is_withdrawn:
			grid.withdrawn_combatants.erase(c)
		var stance_raw: Variant = snap.stances.get(c, c.current_stance)
		if stance_raw is int:
			c.current_stance = stance_raw as CombatTypes.StanceType
		c.active_target = snap.targets.get(c, null)
		c.attack_boost_multiplier = float(snap.attack_boost_mult.get(c, 1.0))
		c.attack_boost_rounds_remaining = int(snap.attack_boost_rds.get(c, 0))
		c.defense_mitigation_ratio = float(snap.defense_mit.get(c, 0.0))
		var pos_raw: Variant = snap.positions.get(c, NO_POSITION)
		if pos_raw is Vector2i and pos_raw != NO_POSITION:
			grid.combatant_positions[c] = pos_raw
		else:
			grid.combatant_positions.erase(c)

func _capture_snapshot(block_idx: int) -> void:
	var snap := BlockSnapshot.new()
	snap.block_index = block_idx
	snap.round_num = current_round
	snap.reinforcements_spawned = encounter.reinforcements_spawned
	for c: Combatant in resolver.registered_combatants:
		snap.positions[c] = grid.get_position(c)
		snap.hps[c] = c.hp
		snap.max_hps[c] = c.max_hp
		snap.stances[c] = c.current_stance
		snap.targets[c] = c.active_target
		snap.is_active[c] = c.is_active()
		snap.is_withdrawn[c] = c.is_withdrawn
		snap.attack_boost_mult[c] = c.attack_boost_multiplier
		snap.attack_boost_rds[c] = c.attack_boost_rounds_remaining
		snap.defense_mit[c] = c.defense_mitigation_ratio
	var snaps: Array[BlockSnapshot] = _snapshots_for(current_round)
	snaps.append(snap)
	round_snapshots[current_round] = snaps

# ------------------------------------------------------------------------------
# 3. Unit state queries (snapshot-aware)
# ------------------------------------------------------------------------------

func is_ally(c: Combatant) -> bool:
	return allies.has(c)

func position_of(c: Combatant) -> Vector2i:
	var snap: BlockSnapshot = display_snapshot()
	if snap != null:
		var raw: Variant = snap.positions.get(c, Vector2i(-1, -1))
		return raw if raw is Vector2i else Vector2i(-1, -1)
	return grid.get_position(c) if grid != null else Vector2i(-1, -1)

func is_on_field(c: Combatant) -> bool:
	var snap: BlockSnapshot = display_snapshot()
	var active: bool = c.is_active()
	if snap != null:
		var raw: Variant = snap.is_active.get(c, false)
		active = raw if raw is bool else false
	return active and position_of(c) != Vector2i(-1, -1)

func position_text(c: Combatant) -> String:
	if is_on_field(c):
		return "Pos: %s" % str(position_of(c))
	if c.is_withdrawn:
		return "Pos: Withdrawn (Safe)"
	if not c.is_alive():
		return "Pos: Defeated"
	return "Pos: Not deployed"

# Enemies taking part in the viewed round up to the playhead: those on the field
# at the start of the round plus any that entered since (e.g. reinforcements).
func enemies_in_view() -> Array[Combatant]:
	var result: Array[Combatant] = []
	var snaps: Array[BlockSnapshot] = _snapshots_for(viewed_round)
	var fully_live: bool = is_live_view() and inspect_block < 0
	var last: int = snaps.size() - 1 if fully_live else mini(playhead(), snaps.size() - 1)
	for e: Combatant in enemies:
		# Live: units that just entered (e.g. a mid-block ambush) count before the next snapshot.
		if fully_live and e.is_active() and grid.get_position(e) != Vector2i(-1, -1):
			result.append(e)
			continue
		for i: int in range(0, last + 1):
			var snap: BlockSnapshot = snaps[i]
			var pos_raw: Variant = snap.positions.get(e, Vector2i(-1, -1))
			var active_raw: Variant = snap.is_active.get(e, false)
			var on_field: bool = pos_raw is Vector2i and pos_raw != Vector2i(-1, -1) and active_raw is bool and active_raw
			if on_field:
				result.append(e)
				break
	return result

func forecast_target(c: Combatant) -> Combatant:
	if c == null:
		return null
	if c.active_target is Combatant:
		var locked: Combatant = c.active_target
		return locked
	if is_ally(c):
		var foes: Array[Combatant] = enemies_in_view()
		return _first_active(foes) if not foes.is_empty() else null
	return _first_active(allies)

# Unit whose plan the bottom timeline shows: the selected enemy during replay,
# otherwise the focused hero (enemy plans stay secret until resolved).
func timeline_unit() -> Combatant:
	if selected_combatant != null and not is_ally(selected_combatant) and not is_live_view():
		return selected_combatant
	if focused_hero != null:
		return focused_hero
	return allies[0] if not allies.is_empty() else null

func is_timeline_masked() -> bool:
	return selected_combatant != null and not is_ally(selected_combatant) and is_live_view()

func find_combatant(p_name: String) -> Combatant:
	for c: Combatant in allies:
		if c.name == p_name:
			return c
	for c: Combatant in enemies:
		if c.name == p_name:
			return c
	return null

static func _first_active(units: Array[Combatant]) -> Combatant:
	for c: Combatant in units:
		if c.is_active():
			return c
	return units[0] if not units.is_empty() else null

# ------------------------------------------------------------------------------
# 4. Selection & hover
# ------------------------------------------------------------------------------

func select_combatant(c: Combatant) -> void:
	if c == null:
		return
	selected_combatant = c
	if is_ally(c):
		focused_hero = c
	_notify()

func focus_first_hero() -> void:
	select_combatant(_first_active(allies))

func set_hovered(c: Combatant) -> void:
	if hovered_combatant == c:
		return
	hovered_combatant = c
	hover_changed.emit(c)

# ------------------------------------------------------------------------------
# 5. Action log
# ------------------------------------------------------------------------------

func _append_entry(entry: ActionLogEntry, with_fx: bool) -> void:
	log_entries.append(entry)
	active_log_index = log_entries.size() - 1
	log_appended.emit(entry)
	if with_fx:
		fx_requested.emit(entry)

# Log stepping keeps the live view during resolution; in replay it moves the playhead.
func log_prev() -> void:
	if active_log_index > 0:
		select_log_entry(log_entries[active_log_index - 1], not is_live_view())

func log_next() -> void:
	if active_log_index < log_entries.size() - 1:
		select_log_entry(log_entries[active_log_index + 1], not is_live_view())

# Jump to an entry: selects its actor and (when `navigate`) moves the timeline to
# the entry's round and block, unless that block belongs to the live round.
func select_log_entry(entry: ActionLogEntry, navigate: bool = true) -> void:
	var idx: int = log_entries.find(entry)
	if idx == -1:
		return
	active_log_index = idx
	fx_requested.emit(entry)
	var actor: Combatant = find_combatant(entry.actor_name)
	if actor != null:
		selected_combatant = actor
		if is_ally(actor):
			focused_hero = actor
	var in_live_round: bool = entry.round_num == current_round and phase != Phase.REPLAY
	if navigate and not in_live_round:
		viewed_round = entry.round_num
		replay_block = clampi(entry.block_num, 1, TOTAL_BLOCKS)
		inspect_block = -1
	status_text = "%s [%s]" % [entry.skill_name, entry.ref_tag]
	_notify()

func _truncate_log_from_round(round_num: int) -> void:
	var kept: Array[ActionLogEntry] = []
	for e: ActionLogEntry in log_entries:
		if e.round_num < round_num:
			kept.append(e)
	log_entries = kept
	active_log_index = log_entries.size() - 1
	log_reset.emit()

func _on_resolver_log(msg: String) -> void:
	_log_text(CombatLogBuilder.format_resolver_message(msg))

func _log_text(bbcode: String) -> void:
	if bbcode.is_empty():
		return
	text_log.append(bbcode)
	if text_log.size() > MAX_TEXT_LOG:
		text_log.remove_at(0)
	text_logged.emit(bbcode)

# ------------------------------------------------------------------------------
# 6. Notifications
# ------------------------------------------------------------------------------

func _set_phase(text: String, color: Color) -> void:
	phase_text = text
	phase_color = color

func notify_external_change() -> void:
	_notify()

func _notify() -> void:
	changed.emit()
