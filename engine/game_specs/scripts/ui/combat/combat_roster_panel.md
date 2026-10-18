---
class: CombatRosterPanel
extends: CombatPanel
description: "Tactical RPG - Combat Roster Panel (party or enemies) One panel per side, each owning its own RosterPanelUI half. Enemy cards are shown only for units taking part in the viewed round (session.enemies_in_view), so mid-round reinforcements appear exactly when they enter the field."
---

# Properties
* `ALLY_COLOR`: Color = Color(0.2, 0.6, 1.0) (const)
* `ENEMY_COLOR`: Color = Color(0.9, 0.3, 0.3) (const)
* `is_party`: bool = true
* `_roster`: RosterPanelUI

# Signals
* `unit_focused(c: Combatant)`
* `unit_blurred()`
* `unit_clicked(c: Combatant)`

# Methods

### `static create(p_is_party: bool) -> CombatRosterPanel`
```logic
var panel := CombatRosterPanel.new()
panel.is_party = p_is_party
panel.panel_id = "party" if p_is_party else "enemies"
panel.panel_title = "Party" if p_is_party else "Enemy Hostiles"
panel.panel_icon = "⚔" if p_is_party else "💀"
panel.add_child(panel._roster.party_panel if p_is_party else panel._roster.enemy_panel)
# RosterPanelUI always builds both halves; keep the unused one as a hidden child so
# the scene tree owns (and frees) it instead of leaking an orphan Control.
var unused: Control = panel._roster.enemy_panel if p_is_party else panel._roster.party_panel
unused.visible = false
panel.add_child(unused)
return panel
```

### `_init() -> void`
```logic
super._init()
_roster = RosterPanelUI.new()
default_min_size = Vector2(CombatHudMetrics.SIDE_W, 120)
```

### `_on_bound() -> void`
```logic
_roster.token_fn = func(c: Combatant) -> Texture2D: return session.encounter.get_token_texture(c)
_roster.behaviour_fn = _behaviour_line
session.mounted.connect(_rebuild)
_roster.combatant_selected.connect(func(c: Combatant) -> void:
	session.select_combatant(c)
	unit_clicked.emit(c)
)
_roster.combatant_focused.connect(func(c: Combatant) -> void: unit_focused.emit(c))
_roster.combatant_blurred.connect(func() -> void: unit_blurred.emit())
_roster.combatant_hovered.connect(func(c: Combatant, _card: Control) -> void: session.set_hovered(c))
_roster.combatant_unhovered.connect(func() -> void: session.set_hovered(null))
_rebuild()
```

### `_rebuild() -> void`
```logic
_roster.clear_roster()
if session == null or session.grid == null:
	return
if is_party:
	for ally: Combatant in session.allies:
		_roster.create_roster_card(ally, ALLY_COLOR, true, session.grid.get_position(ally))
	_roster.add_party_placeholders(session.allies.size())
else:
	for enemy: Combatant in session.enemies:
		_roster.create_roster_card(enemy, ENEMY_COLOR, false, session.grid.get_position(enemy))
```

### `_behaviour_line(c: Combatant) -> String`
```logic
var rec: Dictionary = session.intent_of(c)
if not rec.is_empty():
	return IntentText.first_line(rec) if str(rec["kind"]) == IntentRules.ARRIVING or str(rec["kind"]) == IntentRules.IDLE else "Next: " + IntentText.first_line(rec)
var profile: AiProfile = session.encounter.profile_of(c)
return "%s - hit %d" % [AiProfile.role_name(profile.role), DamageCalculator.power_of(c, c.attack_range > 1)]
```

### `apply_metrics(m: CombatHudMetrics) -> void`
```logic
var changed: bool = m.party_rows != _roster.metrics.party_rows or m.hostile_rows != _roster.metrics.hostile_rows \
	or m.fs != _roster.metrics.fs or m.party_slots != _roster.metrics.party_slots or m.hostile_slots != _roster.metrics.hostile_slots
_roster.set_metrics(m)
if changed or _roster.roster_cards.is_empty():
	_rebuild()
	refresh()
```

### `unit_count() -> int`
```logic
return session.allies.size() if is_party else session.enemies.size()
```

### `refresh() -> void`
```logic
if session == null or session.grid == null:
	return
if not is_party:
	var in_view: Array[Combatant] = session.enemies_in_view()
	for enemy: Combatant in session.enemies:
		var card: Control = _card_for(enemy)
		if card != null:
			card.visible = in_view.has(enemy)
if not is_party:
	_refresh_tells()
_roster.update_highlights(session.selected_combatant, session.allies)
_roster.update_all_roster_cards(session.allies, session.grid, session.display_snapshot())
if is_party:
	_tag_planning_state()
```

### `_refresh_tells() -> void`
```logic
for enemy: Combatant in session.enemies:
	var card: Control = _card_for(enemy)
	if card == null:
		continue
	var beh: Label = card.find_child("BehaviourLabel", true, false) as Label
	if beh != null:
		beh.text = _behaviour_line(enemy)
		beh.tooltip_text = beh.text   # the card clips long lines
	var badge: Control = card.find_child("ForesightBadge", true, false) as Control
	if badge != null:
		var rec: Dictionary = session.intent_of(enemy)
		badge.visible = not rec.is_empty() and DictUtil.to_bool(rec["revealed"])
```

### `_tag_planning_state() -> void`
```logic
for ally: Combatant in session.allies:
	var card: Control = _card_for(ally)
	if card == null or session.plan == null:
		continue
	var lbl: Label = card.find_child("PlanLabel", true, false) as Label
	if lbl == null:
		continue
	if not ally.is_active() or not session.is_planning():
		lbl.text = ""
		continue
	var steps: int = session.plan.steps_of(ally).size()
	var st: CombatPlan.SimState = session.plan.compile(ally)
	var short: bool = _roster.metrics.scale > 1.1
	if steps == 0:
		lbl.text = "No plan"
		lbl.add_theme_color_override("font_color", UiTheme.COL_WARN)
	else:
		var bad: bool = st.error_index >= 0 or st.runs_dry()
		var spent: int = ally.max_ap - st.ap_left
		lbl.text = ("%s%d AP" if short else "%sPlan %d AP") % ["! " if bad else "", spent]
		lbl.add_theme_color_override("font_color", UiTheme.COL_BAD if bad else UiTheme.COL_GOOD)
```

### `_card_for(c: Combatant) -> Control`
```logic
var raw: Variant = _roster.roster_cards.get(c, null)
if raw is Control:
	var card: Control = raw
	return card if is_instance_valid(card) else null
return null
```

### `visible_card_count() -> int`
```logic
var count: int = 0
for c: Combatant in _roster.roster_cards:
	var card: Control = _card_for(c)
	if card != null and card.visible:
		count += 1
return count
```

### `empty_slot_count() -> int`
```logic
var count: int = 0
for child: Node in _roster.party_cards_container.get_children():
	if child.has_meta("empty_slot") and not child.is_queued_for_deletion():
		count += 1
return count
```

### `content_fits() -> bool`
```logic
return _roster.content_fits()
```

