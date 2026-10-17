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
signal plan_preview_changed()                 # hover preview of a planned step (grid ghosts)
# Round-flow marks for observers such as the playtest recorder (PLAYTEST1); emitting them changes no behaviour.
signal round_started(round_num: int)          # end of _begin_round (new round or a redo of it)
signal round_confirmed(round_num: int)        # after a successful confirm_plan (the spec's `plan_confirmed`; that name is the bool below)
signal round_concluded(round_num: int, over: bool)
signal round_reset(round_num: int)            # start of reset_round, before the round is rewound
signal plan_refused(round_num: int, reason: String)   # confirm_plan refused an invalid plan (reason = status_text)
signal plan_edited(kind: String, actor: Combatant)   # add, remove, move, undo, clear, preset, preset_all (actor null)

enum Phase { PLANNING, RESOLVING, REPLAY }

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
var total_blocks: int = 10          # blocks per round (from the encounter)
var current_round: int = 1
var current_block: int = 1          # next block to resolve during RESOLVING
var replay_block: int = 1           # playhead while replaying / viewing a past round
var inspect_block: int = -1         # snapshot inspected during the live round (-1 = live)
var replay_cursor: int = -1         # replay: index into log_entries whose resulting state is shown (-1 = start of the viewed round)
var entry_snapshots: Dictionary = {} # ActionLogEntry -> BlockSnapshot (state right after that log point)
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

# Planning: the party's plan for the live round.
var plan: CombatPlan = null               # per-unit command sequences (rebuilt each round)
var plan_confirmed: bool = false         # sequences locked and scheduled
var preview_step: PlanStep = null         # hovered option being previewed on the grid

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
	entry_snapshots.clear()
	replay_cursor = -1
	active_log_index = -1
	text_log.clear()
	selected_combatant = null
	focused_hero = null
	hovered_combatant = null

	encounter.build()
	grid = encounter.grid
	allies = encounter.allies
	enemies = encounter.enemies
	total_blocks = encounter.total_blocks
	resolver = CombatResolver.new()
	resolver.grid = grid
	resolver.total_blocks = total_blocks
	resolver.rng.seed = encounter.rng_seed
	resolver.log_emitted.connect(_on_resolver_log)
	resolver.action_note.connect(_on_action_note)
	for c: Combatant in allies:
		resolver.register_combatant(c)
	for c: Combatant in enemies:
		resolver.register_combatant(c)
	_set_loadouts_locked(true)

	log_reset.emit()
	mounted.emit()
	_begin_round(1)

func _begin_round(round_num: int, roll_over: bool = true) -> void:
	if round_num > 1 and roll_over:
		resolver.finish_round()
	elif not roll_over:
		resolver.scheduled_actions.clear()
	if roll_over:
		# Last round's statuses expire; queued ones and AP-debt penalties take effect; AP recovers.
		for c: Combatant in resolver.registered_combatants:
			var debt: int = maxi(0, -c.ap)
			var active: Array[String] = StatusEffects.roll_over(c, encounter.ap_regen)
			if c.is_active() and (not active.is_empty() or debt > 0):
				var names: Array[String] = []
				for st_name: String in active:
					names.append(StatusEffects.display_name(st_name))
				_log_text("[color=#f59e0b]⚡ %s: %s%s[/color]" % [c.name, ("AP debt %d paid back; " % debt) if debt > 0 else "", (", ".join(names) + " this round") if not names.is_empty() else "no status"])
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

	plan = CombatPlan.new(encounter.plan_context())
	plan_confirmed = false
	encounter.schedule_round(resolver, round_num)
	_refresh_archive()

	var snaps: Array[BlockSnapshot] = []
	round_snapshots[round_num] = snaps
	_capture_snapshot(0)

	var first_active: Combatant = _first_active(allies)
	focused_hero = first_active
	selected_combatant = first_active
	_notify()
	round_started.emit(round_num)

# The active skill loadout can only change outside combat: locked from start() until dispose().
func _set_loadouts_locked(locked: bool) -> void:
	for c: Combatant in allies:
		c.loadout_locked = locked
	for c: Combatant in enemies:
		c.loadout_locked = locked

# Breaks reference cycles of the current battle (units locking onto each other via
# active_target) so a finished or abandoned battle can be freed. Safe to repeat.
func dispose() -> void:
	_set_loadouts_locked(false)
	for c: Combatant in allies:
		c.active_target = null
	for c: Combatant in enemies:
		c.active_target = null
	if resolver != null:
		resolver.scheduled_actions.clear()
	round_actions.clear()
	round_snapshots.clear()
	_pending_actions.clear()
	if plan != null:
		plan.clear_all()

func reset_round() -> void:
	round_reset.emit(current_round)
	if current_round == 1:
		start()
		return
	_truncate_log_from_round(current_round)
	combat_over = false
	# Rounds carry state over, so rewind to how this round started.
	var start_snap: BlockSnapshot = snapshot_at(current_round, 0)
	if start_snap != null:
		_restore_snapshot(start_snap)
	for c: Combatant in resolver.registered_combatants:
		c.pending_statuses.clear()   # statuses inflicted in the undone round never happened
		c.pending_status_rounds.clear()
		c.pending_status_magnitudes.clear()
	_begin_round(current_round, false)

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
		_replay_step(1)

func step_prev() -> void:
	if is_live_view():
		log_prev()
	else:
		_replay_step(-1)

func _step_resolution() -> void:
	inspect_block = -1
	if phase == Phase.PLANNING:
		if not plan_confirmed and not confirm_plan(false):
			return
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
		for reaction: Dictionary in act.command.reactions:
			_append_entry(CombatLogBuilder.riposte_entry(reaction, current_round, current_block, allies), true)

	if _pending_actions.is_empty():
		resolver.block_completed.emit(current_block)
		if current_block >= total_blocks:
			resolver.apply_round_end_effects()   # bleeding lands before the round is judged
		_capture_snapshot(current_block)
		var over: bool = resolver.is_combat_resolved(allies, enemies)
		if over or current_block >= total_blocks:
			_conclude_round(over)
			return
		current_block += 1
		status_text = "Ready for Block %d." % current_block
	_notify()

func _conclude_round(over: bool) -> void:
	combat_over = over
	phase = Phase.REPLAY
	replay_block = current_block
	replay_cursor = _round_entry_indices(current_round).back() if not _round_entry_indices(current_round).is_empty() else -1
	is_auto_playing = false
	var summary: Dictionary = encounter.round_summary(current_round, over)
	_set_phase(str(summary.get("phase", "ROUND %d CONCLUDED" % current_round)), COL_VICTORY if over else COL_REPLAY)
	status_text = str(summary.get("status", "Round %d concluded. Review the replay." % current_round))
	_log_text(str(summary.get("log", "")))
	export_log()
	next_round_label = encounter.advance_label(current_round) if can_advance_round() else ""
	_notify()
	round_concluded.emit(current_round, over)

func toggle_auto_play() -> void:
	if is_auto_playing:
		is_auto_playing = false
	else:
		if not is_live_view():
			_set_replay_cursor(-1)   # replays start from the positions at the start of the round
		is_auto_playing = true
	_notify()

# Called by the host's timer while auto-playing.
func auto_play_tick() -> void:
	if not is_auto_playing:
		return
	if is_live_view():
		_step_resolution()
	elif _replay_has_next():
		_replay_step(1)
	else:
		is_auto_playing = false
		_notify()

# Inspect the state at block `b` of the viewed round (replay, past round, or an
# already-resolved block of the live round).
func select_block(b: int) -> void:
	var block: int = clampi(b, 0, total_blocks)
	if is_live_view():
		if block < _snapshots_for(viewed_round).size():
			inspect_block = block
			status_text = "Round %d • Block %02d state inspected (live round continues on Step)." % [viewed_round, block]
		else:
			status_text = "Block %d has not resolved yet." % block
	else:
		# Replay: jump to the START of the block (state after everything before it).
		var cursor: int = -1
		for idx: int in _round_entry_indices(viewed_round):
			if log_entries[idx].block_num < block:
				cursor = idx
		_set_replay_cursor(cursor)
		replay_block = maxi(1, block)
		status_text = "Round %d • start of Block %02d." % [viewed_round, replay_block]
	_notify()

func select_round(round_num: int) -> void:
	var target: int = clampi(round_num, 1, max_unlocked_round)
	if target == viewed_round:
		return
	viewed_round = target
	inspect_block = -1
	if not is_live_view():
		_set_replay_cursor(-1)   # a round opens at its start
	status_text = "Viewing Round %d from its start." % target
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
	return log_entries[replay_cursor].block_num if replay_cursor >= 0 and replay_cursor < log_entries.size() else 0

# Snapshot to render, or null for the live state.
func display_snapshot() -> BlockSnapshot:
	if is_live_view():
		return snapshot_at(viewed_round, inspect_block) if inspect_block >= 0 else null
	if replay_cursor >= 0 and replay_cursor < log_entries.size():
		var raw: Variant = entry_snapshots.get(log_entries[replay_cursor], null)
		if raw is BlockSnapshot:
			var after_entry: BlockSnapshot = raw
			return after_entry
	var snaps: Array[BlockSnapshot] = _snapshots_for(viewed_round)
	return snaps[0] if not snaps.is_empty() else null

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
	return phase == Phase.PLANNING and is_live_view() and not combat_over and not plan_confirmed

# Next inputs for `actor`, each probed against the sequence so far (valid or why not).
func plan_options(actor: Combatant) -> Array[PlanOption]:
	return plan.options(actor) if is_planning() and actor != null else []

# Skills offered when the focused hero clicks a target (foe: attacks, self: self skills).
func plan_target_options(target: Combatant) -> Array[PlanOption]:
	if not is_planning() or focused_hero == null:
		return []
	return plan.target_options(focused_hero, target)

# What the focused hero can do about an empty tile (move there, or withdraw at the exit).
func plan_cell_options(cell: Vector2i) -> Array[PlanOption]:
	if not is_planning() or focused_hero == null:
		return []
	return plan.cell_options(focused_hero, cell)

func set_plan_preview(step: PlanStep) -> void:
	if preview_step == step:
		return
	preview_step = step
	plan_preview_changed.emit()

# The hero's compiled state with the hovered step appended (null when not previewing).
func plan_preview_state() -> CombatPlan.SimState:
	if preview_step == null or focused_hero == null or not is_planning():
		return null
	return plan.preview(focused_hero, preview_step)

# Index of the step that produced the focused hero's command at `block` (-1 = none).
func plan_step_index_at(block: int) -> int:
	if focused_hero == null or plan == null:
		return -1
	for e: PlanEntry in plan.entries_for(focused_hero):
		if block >= e.start_block and block <= e.end_block():
			return e.step_index
	return -1

# One-line planning status for the timeline bar.
func plan_info() -> Dictionary:
	var info: Dictionary = {"planning": is_planning(), "text": "", "warn": false, "can_confirm": false}
	if not is_planning() or focused_hero == null:
		return info
	var st: CombatPlan.SimState = plan.compile(focused_hero)
	var text: String = "%s  •  AP %d/%d  •  next block %d  •  %s" % [focused_hero.name, st.ap_left, focused_hero.max_ap, mini(st.next_block, total_blocks), CombatTypes.stance_to_string(st.stance)]
	for status: Variant in focused_hero.statuses:
		text += "  •  %s" % StatusEffects.display_name(str(status))
	if st.error_index >= 0:
		text += "  •  ⚠ step %d invalid: %s" % [st.error_index + 1, st.error_text]
		info["warn"] = true
	elif st.runs_dry():
		var penalty: String = StatusEffects.debt_status(st.debt())
		text += "  •  ⚠ AP debt %d -> %s next round" % [st.debt(), (StatusEffects.display_name(penalty) + " + less AP") if not penalty.is_empty() else "less AP"]
		info["warn"] = true
	info["text"] = text
	info["can_confirm"] = plan_is_valid()
	return info

# Where a unit stands for planning purposes: allies at their planned destination.
func planned_position(c: Combatant) -> Vector2i:
	if is_planning() and is_ally(c) and plan != null:
		return plan.planned_position(c)
	return grid.get_position(c)

# The unit that will stand on cell once planned moves happen (null = free).
func planned_occupant(cell: Vector2i) -> Combatant:
	for c: Combatant in allies + enemies:
		if c.is_active() and grid.has_combatant(c) and planned_position(c) == cell:
			return c
	return null

# Unit shown in the inspector/forecast: the hovered one while planning, else the selection.
func inspected_combatant() -> Combatant:
	if is_planning() and hovered_combatant != null:
		return hovered_combatant
	return selected_combatant
# Appends a step to the actor's sequence; returns "" or the reason it was rejected.
func plan_add_step(actor: Combatant, step: PlanStep) -> String:
	if step == null or actor == null:
		return "No action chosen."
	if not is_planning():
		return "Planning is closed."
	var problem: String = plan.add_step(actor, step)
	status_text = "%s: %s (AP left %d)." % [actor.name, step.title(), plan.ap_left(actor)] if problem.is_empty() else problem
	_after_plan_edit()
	if problem.is_empty():
		plan_edited.emit("add", actor)
	return problem

func plan_remove_step(actor: Combatant, index: int) -> void:
	if is_planning():
		plan.remove_step(actor, index)
		status_text = "%s: step removed." % actor.name
		_after_plan_edit()
		plan_edited.emit("remove", actor)

func plan_move_step(actor: Combatant, from_index: int, to_index: int) -> void:
	if is_planning():
		plan.move_step(actor, from_index, to_index)
		var st: CombatPlan.SimState = plan.compile(actor)
		status_text = "%s: sequence reordered." % actor.name if st.error_index < 0 else "%s: that order is invalid - %s" % [actor.name, st.error_text]
		_after_plan_edit()
		plan_edited.emit("move", actor)

# Removes the focused hero's last planned step (right-click on the grid / Backspace).
func plan_undo_last() -> void:
	if not is_planning() or focused_hero == null:
		return
	var steps: Array[PlanStep] = plan.steps_of(focused_hero)
	if steps.is_empty():
		status_text = "%s has nothing planned to undo." % focused_hero.name
		_notify()
		return
	var removed: PlanStep = steps[steps.size() - 1]
	plan.remove_step(focused_hero, steps.size() - 1)
	status_text = "%s: removed %s." % [focused_hero.name, removed.title()]
	_after_plan_edit()
	plan_edited.emit("undo", focused_hero)

func plan_clear(actor: Combatant) -> void:
	if is_planning():
		plan.clear(actor)
		status_text = "%s: sequence cleared." % actor.name
		_after_plan_edit()
		plan_edited.emit("clear", actor)

# Replaces `actor`'s sequence with an AI draft (see PlanPresets); still editable.
func plan_apply_preset(actor: Combatant, preset_id: String) -> void:
	if not is_planning() or actor == null or not actor.is_active():
		return
	encounter.draft_steps(actor, preset_id, plan)
	status_text = "%s: %s drafted %d step(s)." % [actor.name, PlanPresets.label(preset_id), plan.steps_of(actor).size()]
	_after_plan_edit()
	plan_edited.emit("preset", actor)

func plan_apply_preset_all(preset_id: String) -> void:
	if not is_planning():
		return
	for a: Combatant in allies:
		if a.is_active():
			encounter.draft_steps(a, preset_id, plan)
	# A later ally's draft can still break an earlier one (its move lands on the earlier
	# ally's route, so the approach detours and later steps no longer fit): re-draft those
	# against the claims now on the board, in planning order (bounded, one pass per ally).
	for _pass: int in range(allies.size()):
		var redrafted: bool = false
		for a: Combatant in allies:
			if a.is_active() and not plan.is_valid(a):
				encounter.draft_steps(a, preset_id, plan)
				redrafted = true
		if not redrafted:
			break
	status_text = "Party: %s drafted. Review the sequences, then confirm." % PlanPresets.label(preset_id)
	_after_plan_edit()
	plan_edited.emit("preset_all", null)

# True when every active ally's sequence is valid (an empty sequence is valid: the unit idles).
func plan_is_valid() -> bool:
	for a: Combatant in allies:
		if a.is_active() and not plan.is_valid(a):
			return false
	return true

# Locks the party's sequences, schedules them with the enemies' and starts resolving
# (auto-play; `auto_play` false lets the caller step manually). Refused while any is invalid.
func confirm_plan(auto_play: bool = true) -> bool:
	if not is_planning():
		return false
	if not plan_is_valid():
		status_text = "Fix the highlighted steps before confirming."
		_notify()
		plan_refused.emit(current_round, status_text)
		return false
	plan.commit(resolver)
	plan_confirmed = true
	_refresh_archive(false)
	status_text = "Plan confirmed: resolving the round (Step to go block by block)."
	if auto_play:
		is_auto_playing = true   # confirming starts the round (Play while planning keeps playing); Play afterwards replays it
	_notify()
	round_confirmed.emit(current_round)
	return true

func _after_plan_edit() -> void:
	preview_step = null
	_refresh_archive()
	_notify()

# Rewinds unit state to a snapshot (used to redo a planned round).
func _restore_snapshot(snap: BlockSnapshot) -> void:
	for c: Combatant in resolver.registered_combatants:
		c.hp = DictUtil.int_of(snap.hps, c, c.hp)
		var withdrawn_raw: Variant = snap.is_withdrawn.get(c, false)
		c.is_withdrawn = withdrawn_raw is bool and withdrawn_raw
		if not c.is_withdrawn:
			grid.withdrawn_combatants.erase(c)
		var stance_raw: Variant = snap.stances.get(c, c.current_stance)
		if stance_raw is int:
			var stance_value: int = stance_raw
			c.current_stance = stance_value as CombatTypes.StanceType
		c.active_target = snap.targets.get(c, null)
		c.attack_boost_multiplier = DictUtil.float_of(snap.attack_boost_mult, c, 1.0)
		c.attack_boost_rounds_remaining = DictUtil.int_of(snap.attack_boost_rds, c)
		c.defense_mitigation_ratio = DictUtil.float_of(snap.defense_mit, c)
		c.ap = DictUtil.int_of(snap.aps, c, c.max_ap)
		c.statuses = _dict_of(snap.statuses, c)
		c.status_rounds_left = _dict_of(snap.status_rounds, c)
		c.status_magnitudes = _dict_of(snap.status_magnitudes, c)
		c.riposted.clear()
		var pos_raw: Variant = snap.positions.get(c, NO_POSITION)
		if pos_raw is Vector2i and pos_raw != NO_POSITION:
			grid.combatant_positions[c] = pos_raw
		else:
			grid.combatant_positions.erase(c)
# Timeline copy of the live round's commands. While planning it also includes
# the party's unsent plan so the timeline shows it.
func _refresh_archive(with_plan: bool = true) -> void:
	var archived: Array[CombatResolver.ScheduledAction] = []
	for a: CombatResolver.ScheduledAction in resolver.scheduled_actions:
		archived.append(CombatResolver.ScheduledAction.new(a.combatant, a.command, a.start_block, a.end_block))
	if with_plan and phase == Phase.PLANNING and not plan_confirmed:
		archived.append_array(plan.preview_actions())
	round_actions[current_round] = archived

func _capture_snapshot(block_idx: int) -> void:
	var snaps: Array[BlockSnapshot] = _snapshots_for(current_round)
	snaps.append(_make_snapshot(block_idx))
	round_snapshots[current_round] = snaps

# ------------------------------------------------------------------------------
# Replay (reviewing resolved rounds; a testing / sharing aid, not part of play)
# The cursor is a log point: the state right after that log entry. Stepping moves
# between log points of the viewed round; a round opens at its start, a block at
# its start (after everything before it).
# ------------------------------------------------------------------------------

func _round_entry_indices(round_num: int) -> Array[int]:
	var result: Array[int] = []
	for i: int in range(log_entries.size()):
		if log_entries[i].round_num == round_num:
			result.append(i)
	return result

func _set_replay_cursor(cursor: int) -> void:
	replay_cursor = cursor
	replay_block = log_entries[cursor].block_num if cursor >= 0 and cursor < log_entries.size() else 1
	if cursor >= 0 and cursor < log_entries.size():
		active_log_index = cursor

func _replay_has_next() -> bool:
	var indices: Array[int] = _round_entry_indices(viewed_round)
	return not indices.is_empty() and replay_cursor < indices.back()

# One log point forward (animated) or back.
func _replay_step(direction: int) -> void:
	var indices: Array[int] = _round_entry_indices(viewed_round)
	if indices.is_empty():
		return
	var pos: int = indices.find(replay_cursor)   # -1 when at the start of the round
	var next_pos: int = clampi(pos + direction, -1, indices.size() - 1)
	if next_pos == pos:
		return
	_set_replay_cursor(indices[next_pos] if next_pos >= 0 else -1)
	if next_pos >= 0:
		var entry: ActionLogEntry = log_entries[indices[next_pos]]
		status_text = "Round %d • %s [%s]" % [viewed_round, entry.skill_name, entry.ref_tag]
		if direction > 0:
			fx_requested.emit(entry)
	else:
		status_text = "Round %d • start of the round." % viewed_round
	_notify()

func _make_snapshot(block_idx: int) -> BlockSnapshot:
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
		snap.aps[c] = c.ap
		snap.statuses[c] = c.statuses.duplicate()
		snap.status_rounds[c] = c.status_rounds_left.duplicate()
		snap.status_magnitudes[c] = c.status_magnitudes.duplicate()
	return snap

# A copy of the per-unit dictionary stored in a snapshot (empty when missing).
static func _dict_of(source: Dictionary, key: Combatant) -> Dictionary:
	var raw: Variant = source.get(key, {})
	if raw is Dictionary:
		var found: Dictionary = raw
		return found.duplicate()
	return {}

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
	entry_snapshots[entry] = _make_snapshot(current_block)   # replay steps through log points
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
		replay_cursor = idx
		replay_block = clampi(entry.block_num, 1, total_blocks)
		inspect_block = -1
	status_text = "%s [%s]" % [entry.skill_name, entry.ref_tag]
	_notify()

# Writes the readable combat log (cells + text feed) to user://combat_last.log so a
# finished round can be shared or inspected outside the game. Debug builds only.
func export_log(path: String = "user://combat_last.log") -> void:
	if not OS.is_debug_build():
		return
	var file := FileAccess.open(path, FileAccess.WRITE)
	if file == null:
		return
	file.store_line("%s - round %d (%s)" % [encounter.encounter_name, current_round, Time.get_datetime_string_from_system()])
	file.store_line("== Action log ==")
	for e: ActionLogEntry in log_entries:
		var dmg: String = (" -%d" % e.damage) if e.damage > 0 else ""
		file.store_line("[%s] %s %s %s -> %s [%s%s] %s | %s" % [e.ref_tag, e.category, e.actor_name, e.skill_name, e.target_name, e.outcome_tag, dmg, e.status_text, e.breakdown])
	file.store_line("== Feed ==")
	var bbcode := RegEx.new()
	bbcode.compile("\\[/?(b|i|u|s|color|font_size|url|center)(=[^\\]]*)?\\]")
	for line: String in text_log:
		file.store_line(bbcode.sub(line, "", true))
	file.close()

func _truncate_log_from_round(round_num: int) -> void:
	var kept: Array[ActionLogEntry] = []
	for e: ActionLogEntry in log_entries:
		if e.round_num < round_num:
			kept.append(e)
	log_entries = kept
	for e: Variant in entry_snapshots.keys():
		if e is ActionLogEntry and not kept.has(e):
			entry_snapshots.erase(e)
	active_log_index = log_entries.size() - 1
	log_reset.emit()

# A command did something as it started (a chase): give it its own log cell and animation.
func _on_action_note(action: CombatResolver.ScheduledAction, message: String) -> void:
	if message.begins_with("[Chase]"):
		_append_entry(CombatLogBuilder.chase_entry(action, current_round, current_block, allies, grid, message), true)
	elif message.begins_with("[Dash]"):
		_append_entry(CombatLogBuilder.dash_entry(action, current_round, current_block, allies, grid, message), true)

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
