class_name CombatRosterPanel
extends CombatPanel

# ==============================================================================
# Tactical RPG - Combat Roster Panel (party or enemies)
# One panel per side, each owning its own RosterPanelUI half. Enemy cards are
# shown only for units taking part in the viewed round (session.enemies_in_view),
# so mid-round reinforcements appear exactly when they enter the field.
# COMBAT-UI3: hovering or focusing a card shows that unit in the selected-hero panel / the merged enemy card
# (through session.set_hovered); a click selects it (an enemy stays on its card until another is picked).
# ==============================================================================

const ALLY_COLOR: Color = Color(0.2, 0.6, 1.0)
const ENEMY_COLOR: Color = Color(0.9, 0.3, 0.3)

signal unit_clicked(c: Combatant)   # a click on a card (after it selected the unit)

var is_party: bool = true
var _roster: RosterPanelUI

static func create(p_is_party: bool) -> CombatRosterPanel:
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

func _init() -> void:
	super._init()
	_roster = RosterPanelUI.new()
	default_min_size = Vector2(CombatHudMetrics.LEFT_W, 120)

func _on_bound() -> void:
	_roster.token_fn = func(c: Combatant) -> Texture2D: return session.encounter.get_token_texture(c)
	_roster.behaviour_fn = _behaviour_line
	session.mounted.connect(_rebuild)
	_roster.combatant_selected.connect(func(c: Combatant) -> void:
		session.select_combatant(c)
		unit_clicked.emit(c)
	)
	_roster.combatant_focused.connect(func(c: Combatant) -> void: session.set_hovered(c))
	_roster.combatant_blurred.connect(func() -> void: session.set_hovered(null))
	_roster.combatant_hovered.connect(func(c: Combatant, _card: Control) -> void: session.set_hovered(c))
	_roster.combatant_unhovered.connect(func() -> void: session.set_hovered(null))
	_rebuild()

func _rebuild() -> void:
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

# The hostile card's one line: "Next: <first action>" while planning (INTENT1-U); otherwise its AI role (no numbers).
func _behaviour_line(c: Combatant) -> String:
	var rec: Dictionary = session.intent_of(c)
	if not rec.is_empty():
		return IntentText.first_line(rec) if str(rec["kind"]) == IntentRules.ARRIVING or str(rec["kind"]) == IntentRules.IDLE else "Next: " + IntentText.first_line(rec)
	return AiProfile.role_name(session.encounter.profile_of(c).role)

# New fitted sizes (CombatHudMetrics): fixed panel heights; the cards are rebuilt when their sizes changed.
func apply_metrics(m: CombatHudMetrics) -> void:
	var old: CombatHudMetrics = _roster.metrics
	var changed: bool = m.fs != old.fs or m.party_slots != old.party_slots or m.hostile_slots != old.hostile_slots \
		or m.left_w != old.left_w or m.right_w != old.right_w
	_roster.set_metrics(m)
	if changed or _roster.roster_cards.is_empty():
		_rebuild()
		refresh()

# Party and hostile counts the host sizes the columns for.
func unit_count() -> int:
	return session.allies.size() if is_party else session.enemies.size()

func refresh() -> void:
	if session == null or session.grid == null:
		return
	if not is_party:
		var in_view: Array[Combatant] = session.enemies_in_view()
		for enemy: Combatant in session.enemies:
			var card: Control = _card_for(enemy)
			if card != null:
				card.visible = in_view.has(enemy)
		_refresh_tells()
	_roster.update_highlights(session.selected_combatant, session.allies)
	_roster.update_all_roster_cards(session.allies, session.grid, session.display_snapshot())

# Hostile cards: the "Next:" line follows the tell and a revealed foe wears the eye badge.
func _refresh_tells() -> void:
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

func _card_for(c: Combatant) -> Control:
	var raw: Variant = _roster.roster_cards.get(c, null)
	if raw is Control:
		var card: Control = raw
		return card if is_instance_valid(card) else null
	return null

# Number of visible unit cards (excludes empty party placeholders).
func visible_card_count() -> int:
	var count: int = 0
	for c: Combatant in _roster.roster_cards:
		var card: Control = _card_for(c)
		if card != null and card.visible:
			count += 1
	return count

# Muted outlines for the party slots nobody fills.
func empty_slot_count() -> int:
	var count: int = 0
	for child: Node in _roster.party_cards_container.get_children():
		if child.has_meta("empty_slot") and not child.is_queued_for_deletion():
			count += 1
	return count

func content_fits() -> bool:
	return _roster.content_fits()
