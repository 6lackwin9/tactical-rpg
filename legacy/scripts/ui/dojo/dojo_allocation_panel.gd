class_name DojoAllocationPanel
extends DojoPanel

# ==============================================================================
# Tactical RPG - Stat Allocation (overlay content)
# Opened as an overlay from the Unit Inspector so the inspector itself never
# reflows. Closes itself when the allocation session ends (confirm / cancel).
# ==============================================================================

var _header_lbl: Label
var _remaining_lbl: Label
var _rows_vbox: VBoxContainer

func _init() -> void:
	super._init()
	panel_id = "allocate"
	panel_title = "Allocate Attribute Points"
	panel_short_title = "Allocate"
	panel_icon = "⚡"
	default_min_size = Vector2(320, 330)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 6)
	add_child(vbox)

	_header_lbl = UiTheme.label("", UiTheme.FONT_MD, UiTheme.COL_TEXT)
	vbox.add_child(_header_lbl)
	_remaining_lbl = UiTheme.label("", UiTheme.FONT_MD, UiTheme.COL_GOOD)
	vbox.add_child(_remaining_lbl)

	_rows_vbox = VBoxContainer.new()
	_rows_vbox.add_theme_constant_override("separation", 4)
	vbox.add_child(_rows_vbox)

	vbox.add_spacer(false)
	var buttons := HBoxContainer.new()
	buttons.add_theme_constant_override("separation", 6)
	vbox.add_child(buttons)
	buttons.add_child(UiTheme.button("↺ Reset", "Return newly allocated points to the pool", func() -> void: session.reset_allocation()))
	buttons.add_spacer(false)
	buttons.add_child(UiTheme.button("✖ Cancel", "Discard this allocation session", func() -> void: session.cancel_allocation()))
	buttons.add_child(UiTheme.button("✔ Confirm", "Commit allocated points", func() -> void: session.confirm_allocation()))

func refresh() -> void:
	if session == null or session.active_hero == null or session.active_hero.stats == null:
		return
	if not session.is_allocating_stats:
		overlay_close_requested.emit()
		return
	var hero: Combatant = session.active_hero
	var st: CharacterStats = hero.stats
	var ctx: Dictionary = hero.get_weapon_context()
	_header_lbl.text = "%s  •  Lv %d" % [hero.name, st.level]
	_remaining_lbl.text = "Points remaining: %d" % session.pending_free_points

	UiTheme.clear_children(_rows_vbox)
	for def: Dictionary in DojoStatDefs.STATS:
		var key: String = str(def.get("k", ""))
		var col: Color = def.get("c", Color.WHITE)
		var pending: int = session.get_pending(key)
		var eff: int = st.get_effective_stat(key, ctx)

		var card := UiTheme.card(_rows_vbox)
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 6)
		card.add_child(row)
		row.add_child(UiTheme.label("%s  %s" % [key, str(def.get("name", ""))], UiTheme.FONT_MD, col))
		row.add_spacer(false)
		var value_text: String = "%d" % eff
		if pending > 0:
			value_text = "%d → %d" % [eff, eff + pending]
		row.add_child(UiTheme.label(value_text, UiTheme.FONT_MD, UiTheme.COL_WARN if pending > 0 else Color.WHITE))
		var minus: Button = UiTheme.button("-", "Undo a newly allocated point", func() -> void: session.deallocate_point(key), 11)
		minus.custom_minimum_size = Vector2(24, 22)
		minus.disabled = pending <= 0
		row.add_child(minus)
		var plus: Button = UiTheme.button("+", "Allocate a point", func() -> void: session.allocate_point(key), 11)
		plus.custom_minimum_size = Vector2(24, 22)
		plus.disabled = session.pending_free_points <= 0
		row.add_child(plus)
