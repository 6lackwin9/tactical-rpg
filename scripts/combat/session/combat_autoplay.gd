class_name CombatAutoplay
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat Autoplay (AUTO1-L, docs/specs/AUTO1.md): the party plays itself round after round.
# Framework-free state + loop next to a CombatSession; any combat layout binds to it (dropdowns, toggle, banner).
#
#   Tactics   a party tactic (PlanPresets.AUTO_IDS) and per-hero overrides (AUTO_IDS or CUSTOM). A round is drafted
#             with plan_apply_preset_all(party) and then plan_apply_preset(hero, override) for each overridden hero;
#             a CUSTOM hero ("I plan this hero myself") is left empty and autoplay waits for the player's Confirm.
#   Custom    a player edit of a row (plan_edited add / remove / move / undo / clear, or mark_edited) shows CUSTOM on
#             that row for this round only; picking a tactic again redrafts the row and drops the edits.
#   Loop      the host calls tick() on the timer that drives CombatSession.auto_play_tick(). While running, a fresh
#             planning round is drafted and confirmed (source "auto"; plays at the normal speed through the session's
#             auto_play), and a concluded round advances to the next one.
#   Pause     request_pause() (Space / the toggle): the current round finishes, the next round stops in planning with
#             the draft on the timeline; it stays paused until set_autoplay(true). Safety stops (a hero downed this
#             round, or a hero crossing SAFETY_HP_PCT % HP this round: above it at the start, at or below at the end)
#             and an invalid draft pause the same way
#             with a reason ("Autoplay paused: Bryn is at 22% HP"). A boss phase with pause_autoplay (BOSS1 rule 7) pauses
#             at the next planning phase with the reason "<trigger unit or encounter>: <phase name>".
#   Remembered  bind a GameState: its `autoplay` blob {on, party, heroes} is read at creation and written on every
#             change (hero keys from CombatEncounter.hero_key). Only player choices change `on`; stops do not.
# ==============================================================================

signal state_changed()                            # anything the controls show changed (tactics, running, reason)
signal paused(reason: String, by_player: bool)    # autoplay stopped; by_player false = safety stop or invalid draft
signal resumed()                                  # autoplay turned (back) on
signal waiting(reason: String)                    # running, but a CUSTOM row waits for the player's Confirm this round

const SAFETY_HP_PCT: int = 30
const EDIT_KINDS: Array[String] = ["add", "remove", "move", "undo", "clear"]

var session: CombatSession = null
var store: GameState = null                 # optional: where the choice is remembered (null = this battle only)
var safety_stops: bool = true               # spec Q2: On

var party_tactic: String = PlanPresets.DEFAULT_TACTIC
var hero_tactics: Dictionary = {}           # hero key (String) -> tactic id (AUTO_IDS or CUSTOM), only overrides
var running: bool = false                   # the Autoplay switch for this battle
var remembered_on: bool = false             # the switch as saved (player choices only)

var _pause_reason: String = ""
var _wait_reason: String = ""
var _invalid: Array[Combatant] = []
var _edited: Dictionary = {}                # Combatant -> true: row edited by the player this round (shows CUSTOM)
var _drafted: bool = false                  # the live planning round has been drafted by autoplay
var _stop_next: bool = false                # paused: advance once and stop the next round in planning, drafted
var _hp_at_start: Dictionary = {}           # Combatant -> HP when the round started (stops fire on crossings only)
var _drafting: bool = false

func _init(p_session: CombatSession, p_store: GameState = null) -> void:
	session = p_session
	store = p_store
	if store != null:
		load_settings(store.autoplay)
	session.round_started.connect(_on_round_started)
	session.round_reset.connect(_on_round_reset)
	session.round_concluded.connect(_on_round_concluded)
	session.plan_edited.connect(_on_plan_edited)
	session.phase_fired.connect(_on_phase_fired)
	if session.resolver != null:
		_on_round_started(session.current_round)

# Disconnects from the session (call when the battle scene closes).
func detach() -> void:
	if session == null:
		return
	for pair: Array in [[session.round_started, _on_round_started], [session.round_reset, _on_round_reset],
			[session.round_concluded, _on_round_concluded], [session.plan_edited, _on_plan_edited], [session.phase_fired, _on_phase_fired]]:
		var sig: Signal = pair[0]
		var cb: Callable = pair[1]
		if sig.is_connected(cb):
			sig.disconnect(cb)
	session = null

# ------------------------------------------------------------------------------
# Tactic lists (dropdown entries)
# ------------------------------------------------------------------------------

# [{id, label, description}] in dropdown order: the six tactics, then Custom (`with_custom` false: the six only).
static func tactic_entries(with_custom: bool = true) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for id: String in PlanPresets.tactic_ids():
		if id == PlanPresets.CUSTOM and not with_custom:
			continue
		out.append({"id": id, "label": PlanPresets.label(id), "description": PlanPresets.description(id)})
	return out

# ------------------------------------------------------------------------------
# Tactics
# ------------------------------------------------------------------------------

func get_party_tactic() -> String:
	return party_tactic

# Sets every hero at once (overrides cleared) and redrafts the whole party while planning. False for an unknown id.
func set_party_tactic(id: String) -> bool:
	if not PlanPresets.AUTO_IDS.has(id):
		return false
	party_tactic = id
	hero_tactics.clear()
	_edited.clear()
	if session.is_planning():
		_draft_round()
	_changed()
	return true

# The hero's chosen tactic (its override, else the party tactic); never the per-round edit flip.
func tactic_of(hero: Combatant) -> String:
	return str(hero_tactics.get(_key(hero), party_tactic))

# What the hero's row dropdown shows: CUSTOM after a player edit this round, else tactic_of.
func row_tactic(hero: Combatant) -> String:
	return PlanPresets.CUSTOM if _edited.has(hero) else tactic_of(hero)

func has_override(hero: Combatant) -> bool:
	return hero_tactics.has(_key(hero))

# Picks the hero's tactic (AUTO_IDS or CUSTOM): the row's edits are dropped and it is redrafted at once while planning
# (CUSTOM: emptied). The party tactic clears the override. False for an unknown id, a non-party unit or a unit with
# no hero key (an OBJ1 guest: drafted Balanced, never saved).
func set_hero_tactic(hero: Combatant, id: String) -> bool:
	if hero == null or not session.is_ally(hero) or not PlanPresets.is_tactic(id):
		return false
	var key: String = _key(hero)
	if key.is_empty():
		return false
	if id == party_tactic:
		hero_tactics.erase(key)
	else:
		hero_tactics[key] = id
	_edited.erase(hero)
	if session.is_planning() and hero.is_active():
		_drafting = true
		if id == PlanPresets.CUSTOM:
			session.plan_clear(hero)
		else:
			session.plan_apply_preset(hero, id)
		_drafting = false
	_changed()
	return true

# The player edited the hero's row (the UI may call it; session plan edits are picked up on their own).
func mark_edited(hero: Combatant) -> void:
	if hero == null or _edited.has(hero) or not session.is_ally(hero):
		return
	_edited[hero] = true
	_changed()

func is_edited(hero: Combatant) -> bool:
	return _edited.has(hero)

# ------------------------------------------------------------------------------
# On / off, pause
# ------------------------------------------------------------------------------

func is_on() -> bool:
	return running

# Turns autoplay on (resumes: the next tick drafts / confirms / advances) or off (= request_pause()).
func set_autoplay(on: bool) -> void:
	if on == running:
		return
	if not on:
		request_pause()
		return
	running = true
	remembered_on = true
	_pause_reason = ""
	_stop_next = false
	if session.is_planning() and not _drafted and _has_steps():
		_drafted = true   # turned on over a hand-made plan: keep it and confirm it (no redraft)
	_save()
	resumed.emit()
	_changed()

func toggle() -> void:
	set_autoplay(not running)

# The player's pause (Space / the toggle): the round being played finishes, the next round stops in planning with the
# AI draft on the timeline; autoplay stays off until set_autoplay(true).
func request_pause() -> void:
	if not running:
		return
	remembered_on = false
	_save()
	_pause("Autoplay paused.", true)

func pause_reason() -> String:
	return _pause_reason

# Set while running and a CUSTOM row waits for the player's Confirm ("" otherwise).
func wait_reason() -> String:
	return _wait_reason if running else ""

# Heroes whose draft failed plan_is_valid (the invalid-draft stop); their broken steps show through plan_info.
func invalid_heroes() -> Array[Combatant]:
	return _invalid.duplicate()

# ------------------------------------------------------------------------------
# Loop (called by the host on its playback timer, and by tests)
# ------------------------------------------------------------------------------

func tick() -> void:
	if session == null or session.combat_over or session.current_round != session.max_unlocked_round:
		return
	if session.phase == CombatSession.Phase.REPLAY:
		if (running or _stop_next) and session.can_advance_round():
			session.advance_round()   # the next tick drafts it
		return
	if not session.is_planning():
		return
	if not _drafted:
		if not running and not _stop_next:
			return   # manual rounds start empty, as without autoplay
		_draft_round()
		_stop_next = false
		if not _invalid.is_empty():
			var names: Array[String] = []
			for c: Combatant in _invalid:
				names.append(c.name)
			_pause("Autoplay paused: no valid plan for %s." % ", ".join(names), false)
			return
	if not running:
		return
	var custom: Array[String] = _custom_names()
	if not custom.is_empty():
		var reason: String = "Autoplay waits: plan %s yourself, then confirm." % ", ".join(custom)
		if reason != _wait_reason:
			_wait_reason = reason
			waiting.emit(reason)
			_changed()
		return
	if not session.plan_is_valid():
		_pause("Autoplay paused: fix the highlighted steps.", false)
		return
	session.confirm_plan(true, "auto")

# Drafts the live round from the tactics: party preset, then each override (CUSTOM: emptied), then any row the
# overrides broke is redrafted once with its own tactic; `_invalid` lists those still broken.
func _draft_round() -> void:
	_drafting = true
	_edited.clear()
	_invalid.clear()
	session.plan_apply_preset_all(party_tactic)
	var overridden: Array[Combatant] = []
	for a: Combatant in session.allies:
		if a.is_active() and has_override(a):
			overridden.append(a)
			if tactic_of(a) == PlanPresets.CUSTOM:
				session.plan_clear(a)
			else:
				session.plan_apply_preset(a, tactic_of(a))
	if not overridden.is_empty():
		for a: Combatant in session.allies:
			if a.is_active() and not session.plan.is_valid(a) and tactic_of(a) != PlanPresets.CUSTOM:
				session.plan_apply_preset(a, session.encounter.party_preset_for(a, tactic_of(a)))   # OBJ1 rule 2: guests Balanced
	for a: Combatant in session.allies:
		if a.is_active() and not session.plan.is_valid(a):
			_invalid.append(a)
	_drafting = false
	_drafted = true
	_changed()

func _has_steps() -> bool:
	for a: Combatant in session.allies:
		if not session.plan.steps_of(a).is_empty():
			return true
	return false

func _custom_names() -> Array[String]:
	var out: Array[String] = []
	for a: Combatant in session.allies:
		if a.is_active() and tactic_of(a) == PlanPresets.CUSTOM:
			out.append(a.name)
	return out

func _pause(reason: String, by_player: bool) -> void:
	running = false
	_pause_reason = reason
	_wait_reason = ""
	# Stopped while the round plays or after it: the next round stops in planning, drafted.
	_stop_next = session.phase != CombatSession.Phase.PLANNING or session.plan_confirmed
	paused.emit(reason, by_player)
	_changed()

# ------------------------------------------------------------------------------
# Safety stops
# ------------------------------------------------------------------------------

# "" or the stop text for the round that just ended, from each hero's HP at its start (`hp_at_start`, missing = 0):
# heroes downed this round (HP > 0 at the start, not alive now) and heroes who crossed the line this round (above
# `pct` % HP at the start, still fighting at or below it now), in party order. A hero who stays under the line does
# not stop again; healing back above it re-arms the stop.
static func safety_reason(heroes: Array[Combatant], hp_at_start: Dictionary, pct: int = SAFETY_HP_PCT) -> String:
	var parts: Array[String] = []
	for c: Combatant in heroes:
		var start_hp: int = DictUtil.int_of(hp_at_start, c, 0)
		if start_hp > 0 and not c.is_alive():
			parts.append("%s is down" % c.name)
		elif c.is_active() and c.max_hp > 0 and start_hp * 100 > c.max_hp * pct and c.hp * 100 <= c.max_hp * pct:
			parts.append("%s is at %d%% HP" % [c.name, (c.hp * 100) / c.max_hp])
	return "" if parts.is_empty() else "Autoplay paused: %s" % ", ".join(parts)

# ------------------------------------------------------------------------------
# Remembered settings (GameState.autoplay)
# ------------------------------------------------------------------------------

func settings() -> Dictionary:
	return {"on": remembered_on, "party": party_tactic, "heroes": hero_tactics.duplicate(true)}

# Applies a saved blob (invalid parts fall back to the defaults); `on` starts this battle running.
func load_settings(blob: Dictionary) -> void:
	var party_raw: String = str(blob.get("party", PlanPresets.DEFAULT_TACTIC))
	party_tactic = party_raw if PlanPresets.AUTO_IDS.has(party_raw) else PlanPresets.DEFAULT_TACTIC
	hero_tactics.clear()
	var heroes: Dictionary = DictUtil.dict_of(blob.get("heroes", {}))
	for key: Variant in heroes:
		var id: String = str(heroes[key])
		if PlanPresets.is_tactic(id) and id != party_tactic and not str(key).is_empty():
			hero_tactics[str(key)] = id
	remembered_on = DictUtil.to_bool(blob.get("on", false))
	running = remembered_on
	_pause_reason = ""
	_changed()

func _save() -> void:
	if store != null:
		store.autoplay = settings()

func _changed() -> void:
	_save()
	state_changed.emit()

func _key(hero: Combatant) -> String:
	return session.encounter.hero_key(hero)

# ------------------------------------------------------------------------------
# Session signals (flags only: the session is never driven from inside its own signals)
# ------------------------------------------------------------------------------

func _on_round_started(_round_num: int) -> void:
	_drafted = false
	_edited.clear()
	_invalid.clear()
	_wait_reason = ""
	_hp_at_start.clear()
	for a: Combatant in session.allies:
		_hp_at_start[a] = a.hp if a.is_alive() else 0

func _on_round_reset(_round_num: int) -> void:
	_drafted = false

func _on_round_concluded(_round_num: int, over: bool) -> void:
	_wait_reason = ""
	if over or not running or not safety_stops:
		return
	var reason: String = safety_reason(session.allies, _hp_at_start)
	if not reason.is_empty():
		_pause(reason, false)

# BOSS1 rule 7: a battle-only stop like the safety stops (the saved switch stays on); the next planning phase stops
# drafted (a phase fired at a round start stops that round's planning).
func _on_phase_fired(_entry: ActionLogEntry, info: Dictionary) -> void:
	if not running or not DictUtil.to_bool(info.get("pause_autoplay", true), true):
		return
	_pause(phase_reason(info), false)
	_stop_next = true

# "<trigger unit or encounter>: <phase name>".
static func phase_reason(info: Dictionary) -> String:
	return "%s: %s" % [str(info.get("actor", "")), str(info.get("name", ""))]

func _on_plan_edited(kind: String, actor: Combatant) -> void:
	if _drafting or not EDIT_KINDS.has(kind) or actor == null:
		return
	mark_edited(actor)
