class_name DojoPartyPanel
extends DojoPanel

# ==============================================================================
# Tactical RPG - Party Viewer (Party & Stash screen, column A)
# Party member list (click to select the active unit) with the unit inspector
# and derived stats underneath, as tabs.
# ==============================================================================

var _list: VBoxContainer
var _inspector: DojoUnitInspectorPanel
var _derived: DojoDerivedStatsPanel
var _texture_cache: Dictionary = {} # String -> Texture2D

func _init() -> void:
	super._init()
	panel_id = "party"
	panel_title = "Party"
	panel_short_title = "Party"
	panel_icon = "👥"
	default_min_size = Vector2(260, 300)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 6)
	add_child(vbox)

	vbox.add_child(DojoTheme.header("PARTY MEMBERS  (click to select)"))
	_list = VBoxContainer.new()
	_list.add_theme_constant_override("separation", 4)
	vbox.add_child(_list)

	var tabs := TabContainer.new()
	tabs.size_flags_vertical = Control.SIZE_EXPAND_FILL
	tabs.add_theme_font_size_override("font_size", DojoTheme.FONT_SM)
	vbox.add_child(tabs)

	_inspector = DojoUnitInspectorPanel.new()
	_inspector.name = "Unit Inspector"
	tabs.add_child(_inspector)
	_derived = DojoDerivedStatsPanel.new()
	_derived.name = "Derived Stats"
	tabs.add_child(_derived)

func _on_bound() -> void:
	_inspector.bind(session)
	_derived.bind(session)

func _portrait(hero: Combatant) -> Texture2D:
	var key: String = hero.name.to_lower()
	if not _texture_cache.has(key):
		_texture_cache[key] = AssetUtils.load_svg_texture("res://assets/party/tokens/token_%s.svg" % key)
	var raw: Variant = _texture_cache[key]
	if raw is Texture2D:
		var tex: Texture2D = raw
		return tex
	return null

func refresh() -> void:
	if session == null:
		return
	DojoTheme.clear_children(_list)
	for hero: Combatant in session.party_allies:
		_list.add_child(_build_member_card(hero))

func _build_member_card(hero: Combatant) -> Control:
	var is_active: bool = hero == session.active_hero
	var card := PanelContainer.new()
	card.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	card.tooltip_text = "Select %s" % hero.name
	var border: Color = DojoTheme.COL_GOOD if is_active else DojoTheme.COL_BORDER
	var bg: Color = Color(0.12, 0.22, 0.2, 0.95) if is_active else DojoTheme.COL_CARD_BG
	card.add_theme_stylebox_override("panel", DojoTheme.box(bg, border, 5, 8, 5, 2 if is_active else 1))
	card.gui_input.connect(func(event: InputEvent) -> void:
		var click := event as InputEventMouseButton
		if click != null and click.pressed and click.button_index == MOUSE_BUTTON_LEFT:
			session.set_active_hero(hero)
	)

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.mouse_filter = Control.MOUSE_FILTER_PASS
	card.add_child(row)

	var avatar := TextureRect.new()
	avatar.texture = _portrait(hero)
	avatar.custom_minimum_size = Vector2(40, 40)
	avatar.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	avatar.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	avatar.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(avatar)

	var info := VBoxContainer.new()
	info.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	info.add_theme_constant_override("separation", 1)
	info.mouse_filter = Control.MOUSE_FILTER_PASS
	row.add_child(info)

	var arch: String = hero.stats.archetype.display_name if (hero.stats != null and hero.stats.archetype != null) else "Hero"
	var level: int = hero.stats.level if hero.stats != null else 1
	var title: Label = DojoTheme.label(hero.name + ("  ● ACTIVE" if is_active else ""), DojoTheme.FONT_MD, DojoTheme.COL_GOOD if is_active else Color.WHITE)
	info.add_child(title)
	info.add_child(DojoTheme.label("Lv %d %s" % [level, arch], DojoTheme.FONT_SM, DojoTheme.COL_MUTED))

	var hp_box := VBoxContainer.new()
	hp_box.custom_minimum_size = Vector2(80, 0)
	hp_box.add_theme_constant_override("separation", 2)
	hp_box.mouse_filter = Control.MOUSE_FILTER_PASS
	row.add_child(hp_box)
	var hp_lbl: Label = DojoTheme.label("%d / %d HP" % [hero.hp, hero.max_hp], 9, DojoTheme.COL_TEXT)
	hp_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	hp_box.add_child(hp_lbl)
	var bar := ProgressBar.new()
	bar.show_percentage = false
	bar.max_value = maxi(1, hero.max_hp)
	bar.value = hero.hp
	bar.custom_minimum_size = Vector2(0, 5)
	bar.mouse_filter = Control.MOUSE_FILTER_IGNORE
	bar.add_theme_stylebox_override("background", DojoTheme.box(Color(0.18, 0.08, 0.08, 0.9), Color(0, 0, 0, 0), 2, 0, 0, 0))
	bar.add_theme_stylebox_override("fill", DojoTheme.box(Color(0.25, 0.75, 0.35), Color(0, 0, 0, 0), 2, 0, 0, 0))
	hp_box.add_child(bar)
	return card
