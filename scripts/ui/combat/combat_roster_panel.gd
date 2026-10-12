class_name CombatRosterPanel
extends CombatPanel

# ==============================================================================
# Tactical RPG - Combat Roster Panel (party or enemies)
# One panel per side, each owning its own RosterPanelUI half. Enemy cards are
# shown only for units taking part in the viewed round (session.enemies_in_view),
# so mid-round reinforcements appear exactly when they enter the field.
# ==============================================================================

const ALLY_COLOR: Color = Color(0.2, 0.6, 1.0)
const ENEMY_COLOR: Color = Color(0.9, 0.3, 0.3)

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
	default_min_size = Vector2(280, 84)

func _on_bound() -> void:
	session.mounted.connect(_rebuild)
	_roster.combatant_selected.connect(func(c: Combatant) -> void: session.select_combatant(c))
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
		_roster.add_party_placeholders()
	else:
		for enemy: Combatant in session.enemies:
			_roster.create_roster_card(enemy, ENEMY_COLOR, false, session.grid.get_position(enemy))

func refresh() -> void:
	if session == null or session.grid == null:
		return
	if not is_party:
		var in_view: Array[Combatant] = session.enemies_in_view()
		for enemy: Combatant in session.enemies:
			var card: Control = _card_for(enemy)
			if card != null:
				card.visible = in_view.has(enemy)
	_roster.update_highlights(session.selected_combatant, session.allies)
	_roster.update_all_roster_cards(session.allies, session.grid, session.display_snapshot())

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
