class_name CombatTimelinePanel
extends CombatPanel

# ==============================================================================
# Tactical RPG - Planning strip panel (COMBAT-UI3)
# The slim top strip (TimelineDockUI): the shown unit's round as continuous blocks, the Tactic dropdowns, Autoplay,
# Clear and Confirm round. Enemy tracks stay masked during the live round. ◀ ▶ traverse rounds; clicking a block of
# a resolved round inspects that block's state. Confirm round confirms at once, unless a living hero still has empty
# blocks and the setting `ui/warn_missing_actions` is on: then a small question asks first (Confirm / Back).
# ==============================================================================

const WARN_KEY: String = "ui/warn_missing_actions"

var dock: TimelineDockUI
var autoplay: CombatAutoplay = null   # AUTO1 tactics and the Autoplay loop (the dock host owns it and binds it here)

func _init() -> void:
	super._init()
	panel_id = "timeline"
	panel_title = "Plan"
	panel_icon = "⏱"
	default_min_size = Vector2(600, CombatHudMetrics.STRIP_MIN_H)
	custom_minimum_size = Vector2(0, CombatHudMetrics.STRIP_MIN_H)
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	dock = TimelineDockUI.new()
	add_child(dock)

func _on_bound() -> void:
	dock.action_lookup_fn = session.find_recorded_action
	dock.round_nav_requested.connect(func(delta: int) -> void: session.navigate_round(delta))
	dock.block_selected.connect(func(b: int) -> void: session.select_block(b))
	dock.combatant_selected.connect(func(c: Combatant) -> void: session.select_combatant(c))
	dock.hero_view_requested.connect(func() -> void: session.focus_first_hero())
	dock.fit_changed.connect(refresh)
	dock.clear_requested.connect(func() -> void: session.plan_clear(session.focused_hero))
	dock.confirm_requested.connect(request_confirm)
	dock.missing_confirmed.connect(func() -> void: session.confirm_plan())
	dock.step_remove_requested.connect(func(i: int) -> void: session.plan_remove_step(session.focused_hero, i))
	dock.step_reorder_requested.connect(func(from_i: int, to_i: int) -> void: session.plan_move_step(session.focused_hero, from_i, to_i))

# Confirm round (the button and Enter): confirms at once; with the warning on and a living hero still holding empty
# blocks, the question asks first. An invalid plan goes straight to the session, which refuses it and says why.
# Returns true when the round was confirmed now.
func request_confirm() -> bool:
	if session == null or not session.is_planning():
		return false
	if session.plan_is_valid() and SettingsFile.get_bool(WARN_KEY):
		var names: Array[String] = []
		for hero: Combatant in session.heroes_with_empty_blocks():
			names.append(hero.name)
		if not names.is_empty():
			dock.ask_missing_actions(names)
			return false
	return session.confirm_plan()

# Binds the autoplay state to the controls: dropdowns and toggle follow it; their picks go back to it.
func bind_autoplay(p_autoplay: CombatAutoplay) -> void:
	autoplay = p_autoplay
	autoplay.state_changed.connect(refresh)
	dock.party_tactic_requested.connect(func(id: String) -> void: autoplay.set_party_tactic(id))
	dock.hero_tactic_requested.connect(func(id: String) -> void:
		var hero: Combatant = session.focused_hero
		if hero != null:
			autoplay.set_hero_tactic(hero, id)
	)
	dock.autoplay_toggled.connect(func(on: bool) -> void: autoplay.set_autoplay(on))
	refresh()

# What the controls show: the shown hero's dropdown value (Custom (edited) after a manual edit), the party tactic,
# the Autoplay switch and the pause / wait reason (the status bar shows that one, CombatPlaybackPanel).
func autoplay_state() -> Dictionary:
	var state: Dictionary = {"party": autoplay.get_party_tactic(), "on": autoplay.is_on(), "has_hero": false, "guest": false,
		"row": autoplay.get_party_tactic(), "edited": false, "any_edited": false, "note": "", "note_warn": true}
	var hero: Combatant = session.focused_hero
	if hero != null and session.is_ally(hero):
		state["has_hero"] = true
		state["guest"] = session.encounter.is_guest(hero)
		state["row"] = autoplay.row_tactic(hero)
		state["edited"] = autoplay.is_edited(hero)
	for ally: Combatant in session.allies:
		if autoplay.is_edited(ally):
			state["any_edited"] = true
	if not autoplay.wait_reason().is_empty():
		state["note"] = autoplay.wait_reason()
	elif not autoplay.is_on():
		state["note"] = autoplay.pause_reason()
	return state

# The autoplay pause / wait reason ("" = none): the status bar's hint shows it.
func autoplay_note() -> String:
	return str(autoplay_state().get("note", "")) if autoplay != null and session != null else ""

# Playback / replay arenas: no autoplay controls at all.
func disable_autoplay() -> void:
	dock.set_autoplay_enabled(false)
	refresh()

func refresh() -> void:
	if session == null:
		return
	dock.set_total_blocks(session.total_blocks)
	if autoplay != null:
		dock.set_autoplay_state(autoplay_state())
	dock.set_planning(session.plan_info(), session.plan_step_index_at)
	var unit: Combatant = session.timeline_unit()
	var masked: bool = session.is_timeline_masked()
	var tell: Dictionary = {}
	var lookup: Callable = Callable()
	var foe: Combatant = session.selected_combatant
	if masked and foe != null:
		var rec: Dictionary = session.intent_of(foe)   # {} outside planning: the plain banner
		if not rec.is_empty():
			tell = {"name": foe.name, "line": IntentText.first_line(rec)}
			if DictUtil.to_bool(rec["revealed"]):   # a read foe: its whole track, unmasked and read-only
				masked = false
				unit = foe
				lookup = _planned_action_of
	var shown: Combatant = foe if masked and foe != null else unit
	dock.set_title(title_for(shown))
	dock.update_view(unit, session.playhead(), masked, session.can_view_prev_round(), session.can_view_next_round(), tell, lookup)

# "<Hero> plan" while planning; "<Unit> · Round n" for a round being watched or replayed.
func title_for(unit: Combatant) -> String:
	if unit == null:
		return "Plan"
	return "%s plan" % unit.name if session.is_planning() else "%s · Round %d" % [unit.name, session.viewed_round]

# The scheduled (not yet resolved) action `unit` starts at `block` this round (the revealed enemy track).
func _planned_action_of(unit: Combatant, block: int) -> CombatResolver.ScheduledAction:
	for a: CombatResolver.ScheduledAction in session.resolver.scheduled_actions:
		if a.combatant == unit and a.start_block == block and not a.is_cancelled:
			return a
	return null
