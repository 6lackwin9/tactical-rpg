---
class: CombatTimelinePanel
extends: CombatPanel
description: "Tactical RPG - Timeline Dock Panel Block ruler plus the focused unit's action track for the viewed round, the Tactic dropdowns and the Autoplay toggle. Enemy tracks stay masked during the live round. ◀ ▶ traverse rounds; clicking a block inspects that block's state."
---

# Properties
* `dock`: TimelineDockUI
* `autoplay`: CombatAutoplay = null   # AUTO1 tactics and the Autoplay loop (the dock host owns it and binds it here)

# Methods

### `_init() -> void`
```logic
super._init()
panel_id = "timeline"
panel_title = "Timeline"
panel_icon = "⏱"
default_min_size = Vector2(600, 90)
custom_minimum_size = Vector2(0, 88)
size_flags_horizontal = Control.SIZE_EXPAND_FILL
dock = TimelineDockUI.new()
add_child(dock)
```

### `_on_bound() -> void`
```logic
dock.action_lookup_fn = session.find_recorded_action
dock.round_nav_requested.connect(func(delta: int) -> void: session.navigate_round(delta))
dock.block_selected.connect(func(b: int) -> void: session.select_block(b))
dock.combatant_selected.connect(func(c: Combatant) -> void: session.select_combatant(c))
dock.hero_view_requested.connect(func() -> void: session.focus_first_hero())
dock.fit_changed.connect(refresh)
dock.clear_requested.connect(func() -> void: session.plan_clear(session.focused_hero))
dock.confirm_requested.connect(func() -> void: session.confirm_plan())
dock.step_remove_requested.connect(func(i: int) -> void: session.plan_remove_step(session.focused_hero, i))
dock.step_reorder_requested.connect(func(from_i: int, to_i: int) -> void: session.plan_move_step(session.focused_hero, from_i, to_i))
```

### `bind_autoplay(p_autoplay: CombatAutoplay) -> void`
```logic
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
```

### `autoplay_state() -> Dictionary`
```logic
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
```

### `disable_autoplay() -> void`
```logic
dock.set_autoplay_enabled(false)
refresh()
```

### `refresh() -> void`
```logic
if session == null:
	return
dock.set_total_blocks(session.total_blocks)
if autoplay != null:
	dock.set_autoplay_state(autoplay_state())
dock.set_planning(session.plan_info(), session.plan_step_index_at)   # shows the autoplay note in place of the info line
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
dock.update_view(unit, session.playhead(), masked, session.can_view_prev_round(), session.can_view_next_round(), tell, lookup)
```

