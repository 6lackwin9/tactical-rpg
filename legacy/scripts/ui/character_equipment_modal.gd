class_name CharacterEquipmentModal
extends PanelContainer

# ==============================================================================
# Tactical RPG - Party & Stash Screen
# Full-screen modal composed of three reusable columns that share one DojoSession:
#   A) Party viewer  - party member list + unit inspector / derived stats
#   B) Gear loadout  - equipped slots, battle pouch, inline equip pickers
#   C) Party stash   - filterable shared stash with one-click equip
# Public API (setup_heroes / select_hero / open_modal / close_modal / signals) is
# unchanged so the combat arena and its tests keep working.
# ==============================================================================

signal modal_closed()
signal equipment_updated(combatant: Combatant)

var session: DojoSession = null
var shared_inventory: Inventory = null

var party_allies: Array[Combatant]:
	get:
		var empty: Array[Combatant] = []
		return session.party_allies if session != null else empty

var active_hero: Combatant:
	get:
		return session.active_hero if session != null else null

var _party_panel: DojoPartyPanel
var _gear_panel: DojoGearPanel
var _stash_panel: DojoStashPanel
var _gear_title: Label
var _overlay_layer: UiOverlayLayer

func _init() -> void:
	name = "CharacterEquipmentModal"
	visible = false
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_STOP
	theme = UiTheme.tooltip_theme()
	add_theme_stylebox_override("panel", UiTheme.box(Color(0.02, 0.04, 0.07, 0.92), Color(0, 0, 0, 0), 0, 0, 0, 0))

	var margin := MarginContainer.new()
	for side: String in ["left", "right", "top", "bottom"]:
		margin.add_theme_constant_override("margin_%s" % side, 28)
	add_child(margin)

	var frame := PanelContainer.new()
	frame.add_theme_stylebox_override("panel", UiTheme.box(Color(0.07, 0.09, 0.14, 0.99), Color(0.28, 0.46, 0.75, 0.9), 8, 12, 10, 2))
	margin.add_child(frame)

	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 8)
	frame.add_child(root)

	root.add_child(_build_header())

	var columns := HBoxContainer.new()
	columns.add_theme_constant_override("separation", 10)
	columns.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root.add_child(columns)

	_party_panel = DojoPartyPanel.new()
	_gear_panel = DojoGearPanel.new()
	_gear_panel.show_quick_setup = false
	_stash_panel = DojoStashPanel.new()

	columns.add_child(_build_column("👥  PARTY", _party_panel))
	_gear_title = UiTheme.header("🎒  GEAR LOADOUT", UiTheme.COL_TITLE)
	columns.add_child(_build_column_with_label(_gear_title, _gear_panel))
	columns.add_child(_build_column("📦  PARTY STASH", _stash_panel))

	# Overlay windows (equip picker, stat allocation, custom stash filter) open above all columns.
	_overlay_layer = UiOverlayLayer.new()
	add_child(_overlay_layer)
	_overlay_layer.host_panel(_party_panel)
	_overlay_layer.host_panel(_gear_panel)
	_overlay_layer.host_panel(_stash_panel)

func _build_header() -> Control:
	var header := HBoxContainer.new()
	header.add_theme_constant_override("separation", 10)
	header.add_child(UiTheme.label("PARTY & STASH", UiTheme.FONT_LG + 2, UiTheme.COL_TITLE))
	header.add_child(UiTheme.label("Select a unit, review their stats, then move gear between the loadout and the stash.", UiTheme.FONT_SM, UiTheme.COL_MUTED))
	header.add_spacer(false)
	var close_btn: Button = UiTheme.button("✕ CLOSE (Esc)", "Close the Party & Stash screen", close_modal, UiTheme.FONT_SM)
	header.add_child(close_btn)
	return header

func _build_column(title: String, panel: DojoPanel) -> Control:
	return _build_column_with_label(UiTheme.header(title, UiTheme.COL_TITLE), panel)

func _build_column_with_label(title_lbl: Label, panel: DojoPanel) -> Control:
	var frame := PanelContainer.new()
	frame.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	frame.size_flags_stretch_ratio = 1.0
	frame.add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_PANEL_BG, UiTheme.COL_BORDER, 6, 6, 6))
	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 2)
	frame.add_child(vbox)
	title_lbl.add_theme_font_size_override("font_size", UiTheme.FONT_MD)
	vbox.add_child(title_lbl)
	vbox.add_child(HSeparator.new())
	panel.size_flags_vertical = Control.SIZE_EXPAND_FILL
	vbox.add_child(panel)
	return frame

# ------------------------------------------------------------------------------
# Session binding & lifecycle
# ------------------------------------------------------------------------------

# Shares an existing session (e.g. the Gear Dojo's) so every view stays in sync.
func attach_session(p_session: DojoSession) -> void:
	if session != null and session.changed.is_connected(_on_session_changed):
		session.changed.disconnect(_on_session_changed)
	session = p_session
	session.changed.connect(_on_session_changed)
	_overlay_layer.set_session(session)
	_party_panel.bind(session)
	_gear_panel.bind(session)
	_stash_panel.bind(session)
	_update_titles()

func setup_heroes(allies: Array[Combatant], shared_stash: Inventory = null) -> void:
	shared_inventory = shared_stash
	var stash: Inventory = shared_stash
	# Without a shared stash, a lone hero's own bag acts as the stash (legacy arena behavior).
	if stash == null and allies.size() == 1:
		stash = allies[0].inventory
	attach_session(DojoSession.wrap_party(allies, stash))

func select_hero(hero: Combatant) -> void:
	if session != null:
		session.set_active_hero(hero)

func open_modal(hero: Combatant = null) -> void:
	if hero != null:
		if session != null and session.party_allies.has(hero):
			session.set_active_hero(hero)
		else:
			var lone_party: Array[Combatant] = [hero]
			setup_heroes(lone_party, shared_inventory)
	visible = true
	move_to_front()

func close_modal() -> void:
	_overlay_layer.close_all()
	visible = false
	modal_closed.emit()

func _unhandled_key_input(event: InputEvent) -> void:
	if visible and event.is_action_pressed("ui_cancel"):
		close_modal()
		get_viewport().set_input_as_handled()

func _on_session_changed() -> void:
	_update_titles()
	if visible:
		equipment_updated.emit(session.active_hero)

func _update_titles() -> void:
	var hero_name: String = session.active_hero.name if (session != null and session.active_hero != null) else ""
	_gear_title.text = "🎒  GEAR LOADOUT%s" % ("  -  %s" % hero_name if not hero_name.is_empty() else "")

# Equips a stash item into its natural slot for the active hero (used by tests & callers).
func _equip_item(eq: Equipment) -> void:
	if session != null and eq != null:
		session.equip_item(eq, eq.slot)
