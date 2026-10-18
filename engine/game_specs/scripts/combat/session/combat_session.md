---
class: CombatSession
extends: RefCounted
description: "Tactical RPG - Combat Session (headless state for the combat UI) Owns one battle: the encounter's combatants, grid and resolver, round flow, block-by-block resolution, per-block snapshots for replay, multi-round timeline traversal, selection/hover, and the action log. UI panels never talk to each other; they read this session and call its actions. Every state change ends in `changed`. Visual one-shots (impact FX, hover) use dedicated signals. No Node / scene-tree dependencies, so the whole battle flow is headless-testable. Round-flow marks for observers such as the playtest recorder (PLAYTEST1); emitting them changes no behaviour."
---

# Properties
* `MAX_TEXT_LOG`: int = 400 (const)
* `NO_POSITION`: Vector2i = Vector2i(-1, -1) (const)
* `COL_PLANNING`: Color = Color(0.95, 0.8, 0.35) (const)
* `COL_RESOLVING`: Color = Color(0.4, 0.8, 1.0) (const)
* `COL_REPLAY`: Color = Color(0.3, 0.9, 0.5) (const)
* `COL_VICTORY`: Color = Color(0.4, 0.9, 1.0) (const)
* `encounter`: CombatEncounter = null
* `grid`: CombatGrid = null
* `resolver`: CombatResolver = null
* `allies`: Array[Combatant] = []
* `enemies`: Array[Combatant] = []
* `total_blocks`: int = 10          # blocks per round (from the encounter)
* `current_round`: int = 1
* `current_block`: int = 1          # next block to resolve during RESOLVING
* `replay_block`: int = 1           # playhead while replaying / viewing a past round
* `inspect_block`: int = -1         # snapshot inspected during the live round (-1 = live)
* `replay_cursor`: int = -1         # replay: index into log_entries whose resulting state is shown (-1 = start of the viewed round)
* `entry_snapshots`: Dictionary = {} # ActionLogEntry -> BlockSnapshot (state right after that log point)
* `phase`: Phase = Phase.PLANNING
* `combat_over`: bool = false
* `outcome`: String = ""            # OBJ1: "" while the battle goes on, then the encounter's verdict (won, lost or fled)
* `is_auto_playing`: bool = false
* `viewed_round`: int = 1
* `max_unlocked_round`: int = 1
* `round_actions`: Dictionary = {}    # int -> Array[CombatResolver.ScheduledAction]
* `round_snapshots`: Dictionary = {}  # int -> Array[BlockSnapshot]
* `log_entries`: Array[ActionLogEntry] = []
* `active_log_index`: int = -1
* `text_log`: Array[String] = []
* `selected_combatant`: Combatant = null
* `focused_hero`: Combatant = null
* `hovered_combatant`: Combatant = null
* `phase_text`: String = "PLANNING PHASE"
* `phase_color`: Color = COL_PLANNING
* `status_text`: String = ""
* `next_round_label`: String = ""    # empty = no round advance available
* `plan`: CombatPlan = null               # per-unit command sequences (rebuilt each round)
* `plan_confirmed`: bool = false         # sequences locked and scheduled
* `confirm_source`: String = "player"     # who confirmed the live round's plan: "player" or "auto" (AUTO1 autoplay; recorded per round)
* `preview_step`: PlanStep = null         # hovered option being previewed on the grid
* `_enemies_on_grid_at_schedule`: Array[Combatant] = []   # active enemies on the grid when this round's enemies were scheduled (IntentRules idle vs arriving)
* `_pending_actions`: Array[CombatResolver.ScheduledAction] = []
* `raw`: Variant = round_snapshots.get(round_num, null)
* `snaps`: Array[BlockSnapshot] = []
* `raw_snaps`: Array = raw
* `empty`: Array[BlockSnapshot] = []
* `key`: int = viewed_round if round_num < 0 else round_num
* `actions`: Array[CombatResolver.ScheduledAction] = []
* `raw`: Variant = round_actions.get(key, null)
* `raw_actions`: Array = raw
* `none`: Array[Dictionary] = []
* `result`: Array[int] = []
* `result`: Array[Combatant] = []
* `snaps`: Array[BlockSnapshot] = _snapshots_for(viewed_round)
* `fully_live`: bool = is_live_view() and inspect_block < 0
* `last`: int = snaps.size() - 1 if fully_live else mini(playhead(), snaps.size() - 1)
* `snap`: BlockSnapshot = snaps[i]
* `pos_raw`: Variant = snap.positions.get(e, Vector2i(-1, -1))
* `active_raw`: Variant = snap.is_active.get(e, false)
* `on_field`: bool = pos_raw is Vector2i and pos_raw != Vector2i(-1, -1) and active_raw is bool and active_raw

# Signals
* `changed()`
* `mounted()`
* `log_appended(entry: ActionLogEntry)`
* `log_reset()`
* `fx_requested(entry: ActionLogEntry)`
* `hover_changed(combatant: Combatant)`
* `text_logged(bbcode: String)`
* `plan_preview_changed()`
* `round_started(round_num: int)`
* `round_confirmed(round_num: int)`
* `round_concluded(round_num: int, over: bool)`
* `round_reset(round_num: int)`
* `plan_refused(round_num: int, reason: String)`
* `plan_edited(kind: String, actor: Combatant)`
* `phase_fired(entry: ActionLogEntry, info: Dictionary)`

# Methods

### `_init(p_encounter: CombatEncounter) -> void`
```logic
encounter = p_encounter
```

### `start() -> void`
```logic
dispose()
is_auto_playing = false
combat_over = false
outcome = ""
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
```

### `_begin_round(round_num: int, roll_over: bool = true) -> void`
```logic
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
var fired_now: Array[ActionLogEntry] = []
for raw_phase: Variant in DictUtil.array_of(intro.get("entries", [])):
	if raw_phase is ActionLogEntry:
		var phase_entry: ActionLogEntry = raw_phase
		_append_entry(phase_entry, false)
		fired_now.append(phase_entry)
_set_phase(str(intro.get("phase", "ROUND %d: PLANNING PHASE" % round_num)), COL_PLANNING)
status_text = str(intro.get("status", "Round %d: plan actions, then resolve." % round_num))
_log_text(str(intro.get("log", "")))

plan = CombatPlan.new(encounter.plan_context())
plan_confirmed = false
confirm_source = "player"
_enemies_on_grid_at_schedule = []
for foe: Combatant in enemies:
	if foe.is_active() and grid.has_combatant(foe):
		_enemies_on_grid_at_schedule.append(foe)
encounter.schedule_round(resolver, round_num)
_refresh_archive()

var snaps: Array[BlockSnapshot] = []
round_snapshots[round_num] = snaps
_capture_snapshot(0)

var first_active: Combatant = _first_active(allies)
focused_hero = first_active
selected_combatant = first_active
_notify()
if roll_over:   # after a reset the round-start phases are only re-logged (they fired before)
	for phase_entry: ActionLogEntry in fired_now:
		_emit_phase(phase_entry)
round_started.emit(round_num)
```

### `_emit_phase(entry: ActionLogEntry) -> void`
```logic
var info: Dictionary = encounter.phase_info(entry)
if not info.is_empty():
	phase_fired.emit(entry, info)
```

### `_set_loadouts_locked(locked: bool) -> void`
```logic
for c: Combatant in allies:
	c.loadout_locked = locked
for c: Combatant in enemies:
	c.loadout_locked = locked
```

### `dispose() -> void`
```logic
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
```

### `reset_round() -> void`
```logic
round_reset.emit(current_round)
if current_round == 1:
	start()
	return
_truncate_log_from_round(current_round)
combat_over = false
outcome = ""
# Rounds carry state over, so rewind to how this round started.
var start_snap: BlockSnapshot = snapshot_at(current_round, 0)
if start_snap != null:
	_restore_snapshot(start_snap)
for c: Combatant in resolver.registered_combatants:
	c.pending_statuses.clear()   # statuses inflicted in the undone round never happened
	c.pending_status_rounds.clear()
	c.pending_status_magnitudes.clear()
	c.pending_status_sources.clear()
_begin_round(current_round, false)
```

### `reset_battle() -> void`
```logic
start()
```

### `can_advance_round() -> bool`
```logic
return phase == Phase.REPLAY and not combat_over and current_round < encounter.max_rounds and current_round == max_unlocked_round
```

### `advance_round() -> void`
```logic
if can_advance_round():
	_begin_round(current_round + 1)
```

### `is_live_view() -> bool`
```logic
return viewed_round == current_round and phase != Phase.REPLAY
```

### `step_next() -> void`
```logic
if is_live_view():
	_step_resolution()
else:
	_replay_step(1)
```

### `step_prev() -> void`
```logic
if is_live_view():
	log_prev()
else:
	_replay_step(-1)
```

### `_step_resolution() -> void`
```logic
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
		_emit_phase(trigger)
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
	var end_of_round: bool = current_block >= total_blocks
	if end_of_round:
		resolver.apply_round_end_effects()   # bleeding lands before the round is judged
	encounter.on_block_end(current_round, current_block)
	outcome = encounter.verdict(current_round, current_block, end_of_round)   # OBJ1 rule 3 (reserved units do not count)
	_capture_snapshot(current_block)
	var over: bool = not outcome.is_empty()
	if over or end_of_round:
		_conclude_round(over)
		return
	current_block += 1
	status_text = "Ready for Block %d." % current_block
_notify()
```

### `_conclude_round(over: bool) -> void`
```logic
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
```

### `toggle_auto_play() -> void`
```logic
if is_auto_playing:
	is_auto_playing = false
else:
	if not is_live_view():
		_set_replay_cursor(-1)   # replays start from the positions at the start of the round
	is_auto_playing = true
_notify()
```

### `auto_play_tick() -> void`
```logic
if not is_auto_playing:
	return
if is_live_view():
	_step_resolution()
elif _replay_has_next():
	_replay_step(1)
else:
	is_auto_playing = false
	_notify()
```

### `select_block(b: int) -> void`
```logic
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
```

### `select_round(round_num: int) -> void`
```logic
var target: int = clampi(round_num, 1, max_unlocked_round)
if target == viewed_round:
	return
viewed_round = target
inspect_block = -1
if not is_live_view():
	_set_replay_cursor(-1)   # a round opens at its start
status_text = "Viewing Round %d from its start." % target
_notify()
```

### `navigate_round(delta: int) -> void`
```logic
select_round(viewed_round + delta)
```

### `can_view_prev_round() -> bool`
```logic
return viewed_round > 1
```

### `can_view_next_round() -> bool`
```logic
return viewed_round < max_unlocked_round
```

### `playhead() -> int`
```logic
if is_live_view():
	return inspect_block if inspect_block >= 0 else current_block
return log_entries[replay_cursor].block_num if replay_cursor >= 0 and replay_cursor < log_entries.size() else 0
```

### `display_snapshot() -> BlockSnapshot`
```logic
if is_live_view():
	return snapshot_at(viewed_round, inspect_block) if inspect_block >= 0 else null
if replay_cursor >= 0 and replay_cursor < log_entries.size():
	var raw: Variant = entry_snapshots.get(log_entries[replay_cursor], null)
	if raw is BlockSnapshot:
		var after_entry: BlockSnapshot = raw
		return after_entry
var snaps: Array[BlockSnapshot] = _snapshots_for(viewed_round)
return snaps[0] if not snaps.is_empty() else null
```

### `snapshot_at(round_num: int, block: int) -> BlockSnapshot`
```logic
var snaps: Array[BlockSnapshot] = _snapshots_for(round_num)
if block < 0 or block >= snaps.size():
	return null
return snaps[block]
```

### `is_planning() -> bool`
```logic
return phase == Phase.PLANNING and is_live_view() and not combat_over and not plan_confirmed
```

### `intent_of(enemy: Combatant) -> Dictionary`
```logic
for record: Dictionary in enemy_intents():
	if record["unit"] == enemy:
		return record
return {}
```

### `set_plan_preview(step: PlanStep) -> void`
```logic
if preview_step == step:
	return
preview_step = step
plan_preview_changed.emit()
```

### `plan_step_index_at(block: int) -> int`
```logic
if focused_hero == null or plan == null:
	return -1
for e: PlanEntry in plan.entries_for(focused_hero):
	if block >= e.start_block and block <= e.end_block():
		return e.step_index
return -1
```

### `plan_info() -> Dictionary`
```logic
var info: Dictionary = {"planning": is_planning(), "text": "", "warn": false, "can_confirm": false}
if not is_planning() or focused_hero == null:
	return info
var st: CombatPlan.SimState = plan.compile(focused_hero)
var text: String = "%s  •  AP %d/%d  •  next block %d  •  %s" % [focused_hero.name, st.ap_left, focused_hero.max_ap, mini(st.next_block, total_blocks), CombatTypes.stance_to_string(st.stance)]
for status: Variant in focused_hero.statuses:
	text += "  •  %s" % StatusEffects.chip_text(focused_hero, str(status))
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
```

### `planned_position(c: Combatant) -> Vector2i`
```logic
if is_planning() and is_ally(c) and plan != null:
	return plan.planned_position(c)
return grid.get_position(c)
```

### `planned_occupant(cell: Vector2i) -> Combatant`
```logic
for c: Combatant in allies + enemies:
	if c.is_active() and grid.has_combatant(c) and planned_position(c) == cell:
		return c
return null
```

### `inspected_combatant() -> Combatant`
```logic
if is_planning() and hovered_combatant != null:
	return hovered_combatant
return selected_combatant
```

### `plan_add_step(actor: Combatant, step: PlanStep) -> String`
```logic
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
```

### `plan_remove_step(actor: Combatant, index: int) -> void`
```logic
if is_planning():
	plan.remove_step(actor, index)
	status_text = "%s: step removed." % actor.name
	_after_plan_edit()
	plan_edited.emit("remove", actor)
```

### `plan_move_step(actor: Combatant, from_index: int, to_index: int) -> void`
```logic
if is_planning():
	plan.move_step(actor, from_index, to_index)
	var st: CombatPlan.SimState = plan.compile(actor)
	status_text = "%s: sequence reordered." % actor.name if st.error_index < 0 else "%s: that order is invalid - %s" % [actor.name, st.error_text]
	_after_plan_edit()
	plan_edited.emit("move", actor)
```

### `plan_undo_last() -> void`
```logic
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
```

### `plan_clear(actor: Combatant) -> void`
```logic
if is_planning():
	plan.clear(actor)
	status_text = "%s: sequence cleared." % actor.name
	_after_plan_edit()
	plan_edited.emit("clear", actor)
```

### `plan_apply_preset(actor: Combatant, preset_id: String) -> void`
```logic
if not is_planning() or actor == null or not actor.is_active():
	return
encounter.draft_steps(actor, preset_id, plan)
status_text = "%s: %s drafted %d step(s)." % [actor.name, PlanPresets.label(preset_id), plan.steps_of(actor).size()]
_after_plan_edit()
plan_edited.emit("preset", actor)
```

### `plan_apply_preset_all(preset_id: String) -> void`
```logic
if not is_planning():
	return
for a: Combatant in allies:
	if a.is_active():
		encounter.draft_steps(a, encounter.party_preset_for(a, preset_id), plan)   # OBJ1 rule 2: guests Balanced
# A later ally's draft can still break an earlier one (its move lands on the earlier
# ally's route, so the approach detours and later steps no longer fit): re-draft those
# against the claims now on the board, in planning order (bounded, one pass per ally).
for _pass: int in range(allies.size()):
	var redrafted: bool = false
	for a: Combatant in allies:
		if a.is_active() and not plan.is_valid(a):
			encounter.draft_steps(a, encounter.party_preset_for(a, preset_id), plan)
			redrafted = true
	if not redrafted:
		break
status_text = "Party: %s drafted. Review the sequences, then confirm." % PlanPresets.label(preset_id)
_after_plan_edit()
plan_edited.emit("preset_all", null)
```

### `plan_is_valid() -> bool`
```logic
for a: Combatant in allies:
	if a.is_active() and not plan.is_valid(a):
		return false
return true
```

### `confirm_plan(auto_play: bool = true, source: String = "player") -> bool`
```logic
if not is_planning():
	return false
if not plan_is_valid():
	status_text = "Fix the highlighted steps before confirming."
	_notify()
	plan_refused.emit(current_round, status_text)
	return false
plan.commit(resolver)
plan_confirmed = true
confirm_source = source
_refresh_archive(false)
status_text = "Plan confirmed: resolving the round (Step to go block by block)."
if auto_play:
	is_auto_playing = true   # confirming starts the round (Play while planning keeps playing); Play afterwards replays it
_notify()
round_confirmed.emit(current_round)
return true
```

### `_after_plan_edit() -> void`
```logic
preview_step = null
_refresh_archive()
_notify()
```

### `_restore_snapshot(snap: BlockSnapshot) -> void`
```logic
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
	c.pending_status_magnitudes = _dict_of(snap.pending_status_magnitudes, c)
	c.status_sources = _dict_of(snap.status_sources, c)
	c.pending_status_sources = _dict_of(snap.pending_status_sources, c)
	c.riposted.clear()
	var pouch_raw: Variant = snap.pouches.get(c, null)
	if pouch_raw is Array and c.inventory != null:
		var pouch_list: Array = pouch_raw
		c.inventory.battle_pouch.assign(pouch_list)   # ITEMUSE1 rule 6: items used after the snapshot come back
	var pos_raw: Variant = snap.positions.get(c, NO_POSITION)
	if pos_raw is Vector2i and pos_raw != NO_POSITION:
		grid.combatant_positions[c] = pos_raw
	else:
		grid.combatant_positions.erase(c)
encounter.reinforcements_spawned = snap.reinforcements_spawned   # a reset round re-runs its wave
encounter.restore_objective_state(snap.objective_state)           # OBJ1: hold rounds counted
encounter.restore_phase_state(snap.phase_state, snap.round_num)   # BOSS1: fired / due / dropped and live AI profiles
```

### `_refresh_archive(with_plan: bool = true) -> void`
```logic
var archived: Array[CombatResolver.ScheduledAction] = []
for a: CombatResolver.ScheduledAction in resolver.scheduled_actions:
	archived.append(CombatResolver.ScheduledAction.new(a.combatant, a.command, a.start_block, a.end_block))
if with_plan and phase == Phase.PLANNING and not plan_confirmed:
	archived.append_array(plan.preview_actions())
round_actions[current_round] = archived
```

### `_capture_snapshot(block_idx: int) -> void`
```logic
var snaps: Array[BlockSnapshot] = _snapshots_for(current_round)
snaps.append(_make_snapshot(block_idx))
round_snapshots[current_round] = snaps
```

### `_set_replay_cursor(cursor: int) -> void`
```logic
replay_cursor = cursor
replay_block = log_entries[cursor].block_num if cursor >= 0 and cursor < log_entries.size() else 1
if cursor >= 0 and cursor < log_entries.size():
	active_log_index = cursor
```

### `_replay_has_next() -> bool`
```logic
var indices: Array[int] = _round_entry_indices(viewed_round)
return not indices.is_empty() and replay_cursor < indices.back()
```

### `_replay_step(direction: int) -> void`
```logic
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
```

### `_make_snapshot(block_idx: int) -> BlockSnapshot`
```logic
var snap := BlockSnapshot.new()
snap.block_index = block_idx
snap.round_num = current_round
snap.reinforcements_spawned = encounter.reinforcements_spawned
snap.phase_state = encounter.phase_state()
snap.objective_state = encounter.objective_state()
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
	snap.pending_status_magnitudes[c] = c.pending_status_magnitudes.duplicate()
	snap.status_sources[c] = c.status_sources.duplicate()
	snap.pending_status_sources[c] = c.pending_status_sources.duplicate()
	if c.inventory != null:
		snap.pouches[c] = c.inventory.battle_pouch.duplicate()   # same Consumable objects (GameEncounter write-back tracks them)
return snap
```

### `static _dict_of(source: Dictionary, key: Combatant) -> Dictionary`
```logic
var raw: Variant = source.get(key, {})
if raw is Dictionary:
	var found: Dictionary = raw
	return found.duplicate()
return {}
```

### `is_ally(c: Combatant) -> bool`
```logic
return allies.has(c)
```

### `position_of(c: Combatant) -> Vector2i`
```logic
var snap: BlockSnapshot = display_snapshot()
if snap != null:
	var raw: Variant = snap.positions.get(c, Vector2i(-1, -1))
	return raw if raw is Vector2i else Vector2i(-1, -1)
return grid.get_position(c) if grid != null else Vector2i(-1, -1)
```

### `is_on_field(c: Combatant) -> bool`
```logic
var snap: BlockSnapshot = display_snapshot()
var active: bool = c.is_active()
if snap != null:
	var raw: Variant = snap.is_active.get(c, false)
	active = raw if raw is bool else false
return active and position_of(c) != Vector2i(-1, -1)
```

### `position_text(c: Combatant) -> String`
```logic
if is_on_field(c):
	return "Pos: %s" % str(position_of(c))
if c.is_withdrawn:
	return "Pos: Withdrawn (Safe)"
if not c.is_alive():
	return "Pos: Defeated"
return "Pos: Not deployed"
```

### `forecast_target(c: Combatant) -> Combatant`
```logic
if c == null:
	return null
if c.active_target is Combatant:
	var locked: Combatant = c.active_target
	return locked
if is_ally(c):
	var foes: Array[Combatant] = enemies_in_view()
	return _first_active(foes) if not foes.is_empty() else null
return _first_active(allies)
```

### `timeline_unit() -> Combatant`
```logic
if selected_combatant != null and not is_ally(selected_combatant) and not is_live_view():
	return selected_combatant
if focused_hero != null:
	return focused_hero
return allies[0] if not allies.is_empty() else null
```

### `is_timeline_masked() -> bool`
```logic
return selected_combatant != null and not is_ally(selected_combatant) and is_live_view()
```

### `find_combatant(p_name: String) -> Combatant`
```logic
for c: Combatant in allies:
	if c.name == p_name:
		return c
for c: Combatant in enemies:
	if c.name == p_name:
		return c
return null
```

### `static _first_active(units: Array[Combatant]) -> Combatant`
```logic
for c: Combatant in units:
	if c.is_active():
		return c
return units[0] if not units.is_empty() else null
```

### `select_combatant(c: Combatant) -> void`
```logic
if c == null:
	return
selected_combatant = c
if is_ally(c):
	focused_hero = c
_notify()
```

### `focus_first_hero() -> void`
```logic
select_combatant(_first_active(allies))
```

### `set_hovered(c: Combatant) -> void`
```logic
if hovered_combatant == c:
	return
hovered_combatant = c
hover_changed.emit(c)
```

### `_append_entry(entry: ActionLogEntry, with_fx: bool) -> void`
```logic
log_entries.append(entry)
entry_snapshots[entry] = _make_snapshot(current_block)   # replay steps through log points
active_log_index = log_entries.size() - 1
log_appended.emit(entry)
if with_fx:
	fx_requested.emit(entry)
```

### `log_prev() -> void`
```logic
if active_log_index > 0:
	select_log_entry(log_entries[active_log_index - 1], not is_live_view())
```

### `log_next() -> void`
```logic
if active_log_index < log_entries.size() - 1:
	select_log_entry(log_entries[active_log_index + 1], not is_live_view())
```

### `select_log_entry(entry: ActionLogEntry, navigate: bool = true) -> void`
```logic
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
```

### `export_log(path: String = "user://combat_last.log") -> void`
```logic
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
```

### `_truncate_log_from_round(round_num: int) -> void`
```logic
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
```

### `_on_action_note(action: CombatResolver.ScheduledAction, message: String) -> void`
```logic
if message.begins_with("[Chase]"):
	_append_entry(CombatLogBuilder.chase_entry(action, current_round, current_block, allies, grid, message), true)
elif message.begins_with("[Dash]"):
	_append_entry(CombatLogBuilder.dash_entry(action, current_round, current_block, allies, grid, message), true)
```

### `_on_resolver_log(msg: String) -> void`
```logic
_log_text(CombatLogBuilder.format_resolver_message(msg))
```

### `_log_text(bbcode: String) -> void`
```logic
if bbcode.is_empty():
	return
text_log.append(bbcode)
if text_log.size() > MAX_TEXT_LOG:
	text_log.remove_at(0)
text_logged.emit(bbcode)
```

### `_set_phase(text: String, color: Color) -> void`
```logic
phase_text = text
phase_color = color
```

### `notify_external_change() -> void`
```logic
_notify()
```

### `_notify() -> void`
```logic
changed.emit()
```

