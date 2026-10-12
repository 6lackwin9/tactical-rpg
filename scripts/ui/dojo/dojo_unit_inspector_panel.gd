class_name DojoUnitInspectorPanel
extends DojoPanel

# ==============================================================================
# Tactical RPG - Dojo Component 1: Unit Inspector
# Identity, level, 6 base attributes (with interactive point allocation) and
# weapon masteries for the active hero. Hover a stat for its breakdown.
# ==============================================================================

const STAT_DEFS: Array[Dictionary] = [
	{"k": "STR", "name": "Strength", "kind": "Active", "role": "Melee scaling (×1.5), flat mitigation (×0.25) & carry capacity.", "c": Color(0.95, 0.45, 0.45)},
	{"k": "CONS", "name": "Constitution", "kind": "Passive", "role": "Max Health (×8) & endurance.", "c": Color(0.95, 0.65, 0.3)},
	{"k": "DEX", "name": "Dexterity", "kind": "Active", "role": "Ranged scaling (×1.5), initiative speed & crit rate.", "c": Color(0.4, 0.9, 0.5)},
	{"k": "PERC", "name": "Perception", "kind": "Passive", "role": "Intent unmasking & sight range.", "c": Color(0.4, 0.8, 0.95)},
	{"k": "INT", "name": "Intelligence", "kind": "Active", "role": "Skill power (×1.5), buff potency & debuff resistance.", "c": Color(0.75, 0.5, 0.95)},
	{"k": "KNOW", "name": "Knowledge", "kind": "Passive", "role": "Lore checks & danger mitigation.", "c": Color(0.5, 0.7, 1.0)},
]

var _avatar: TextureRect
var _name_lbl: Label
var _class_lbl: Label
var _level_lbl: Label
var _points_btn: Button
var _stats_grid: GridContainer
var _stat_boxes: Dictionary = {}    # String -> DojoTipPanel
var _stat_values: Dictionary = {}   # String -> Label
var _plus_btns: Dictionary = {}     # String -> Button
var _minus_btns: Dictionary = {}    # String -> Button
var _alloc_box: VBoxContainer
var _alloc_lbl: Label
var _mastery_vbox: VBoxContainer
var _texture_cache: Dictionary = {} # String -> Texture2D

func _init() -> void:
	super._init()
	panel_id = "inspector"
	panel_title = "Unit Inspector"
	panel_short_title = "Inspector"
	panel_icon = "🧍"
	default_min_size = Vector2(240, 170)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	add_child(scroll)

	var vbox := VBoxContainer.new()
	vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	vbox.add_theme_constant_override("separation", 6)
	scroll.add_child(vbox)

	_build_identity(vbox)
	_build_stats(vbox)
	_build_allocation_controls(vbox)

	vbox.add_child(DojoTheme.header("WEAPON MASTERIES"))
	_mastery_vbox = VBoxContainer.new()
	_mastery_vbox.add_theme_constant_override("separation", 2)
	vbox.add_child(_mastery_vbox)

func _build_identity(parent: Control) -> void:
	var card := PanelContainer.new()
	card.add_theme_stylebox_override("panel", DojoTheme.box(DojoTheme.COL_CARD_BG, DojoTheme.COL_BORDER, 4, 8, 6))
	parent.add_child(card)

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	card.add_child(row)

	_avatar = TextureRect.new()
	_avatar.custom_minimum_size = Vector2(46, 46)
	_avatar.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	_avatar.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	row.add_child(_avatar)

	var info := VBoxContainer.new()
	info.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	info.add_theme_constant_override("separation", 1)
	row.add_child(info)

	_name_lbl = DojoTheme.label("Hero", DojoTheme.FONT_LG, Color.WHITE)
	info.add_child(_name_lbl)
	_class_lbl = DojoTheme.label("Archetype", DojoTheme.FONT_SM, DojoTheme.COL_MUTED)
	info.add_child(_class_lbl)
	_level_lbl = DojoTheme.label("Level 1 / 10", DojoTheme.FONT_SM, DojoTheme.COL_WARN)
	info.add_child(_level_lbl)

	_points_btn = DojoTheme.button("Unspent Points: 0", "Spend free attribute points", _on_points_pressed)
	parent.add_child(_points_btn)

func _build_stats(parent: Control) -> void:
	parent.add_child(DojoTheme.header("CORE ATTRIBUTES  (hover for breakdown)"))
	var grid := GridContainer.new()
	_stats_grid = grid
	grid.columns = 2
	grid.add_theme_constant_override("h_separation", 6)
	grid.add_theme_constant_override("v_separation", 4)
	parent.add_child(grid)

	for def: Dictionary in STAT_DEFS:
		var key: String = str(def.get("k", ""))
		var col: Color = def.get("c", Color.WHITE)

		var box := DojoTipPanel.new()
		box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		box.mouse_filter = Control.MOUSE_FILTER_STOP
		box.add_theme_stylebox_override("panel", DojoTheme.box(Color(0.11, 0.14, 0.20, 0.9), DojoTheme.COL_BORDER, 4, 6, 3))
		grid.add_child(box)

		var row := HBoxContainer.new()
		row.mouse_filter = Control.MOUSE_FILTER_PASS
		row.add_theme_constant_override("separation", 3)
		box.add_child(row)

		var tag := DojoTheme.label("%s" % key, DojoTheme.FONT_MD, col)
		tag.mouse_filter = Control.MOUSE_FILTER_PASS
		row.add_child(tag)
		var kind := DojoTheme.label(str(def.get("kind", "")), 8, DojoTheme.COL_MUTED)
		kind.mouse_filter = Control.MOUSE_FILTER_PASS
		row.add_child(kind)
		row.add_spacer(false)

		var minus := DojoTheme.button("-", "Undo a newly allocated point", func() -> void: session.deallocate_point(key), 9)
		minus.custom_minimum_size = Vector2(18, 18)
		minus.visible = false
		row.add_child(minus)

		var value := DojoTheme.label("10", DojoTheme.FONT_MD, Color.WHITE)
		value.mouse_filter = Control.MOUSE_FILTER_PASS
		row.add_child(value)

		var plus := DojoTheme.button("+", "Allocate a point", func() -> void: session.allocate_point(key), 9)
		plus.custom_minimum_size = Vector2(18, 18)
		plus.visible = false
		row.add_child(plus)

		_stat_boxes[key] = box
		_stat_values[key] = value
		_plus_btns[key] = plus
		_minus_btns[key] = minus

func _build_allocation_controls(parent: Control) -> void:
	_alloc_box = VBoxContainer.new()
	_alloc_box.add_theme_constant_override("separation", 3)
	_alloc_box.visible = false
	parent.add_child(_alloc_box)

	_alloc_lbl = DojoTheme.label("Allocating: 0 remaining", DojoTheme.FONT_SM, DojoTheme.COL_GOOD)
	_alloc_box.add_child(_alloc_lbl)

	var btn_row := HBoxContainer.new()
	btn_row.add_theme_constant_override("separation", 4)
	_alloc_box.add_child(btn_row)
	btn_row.add_child(DojoTheme.button("↺ Reset", "Return newly allocated points to the pool", func() -> void: session.reset_allocation()))
	btn_row.add_child(DojoTheme.button("✔ Confirm", "Commit allocated points", func() -> void: session.confirm_allocation()))
	btn_row.add_child(DojoTheme.button("✖ Cancel", "Discard this allocation session", func() -> void: session.cancel_allocation()))

func _on_points_pressed() -> void:
	session.begin_allocation()

func _token_texture(hero: Combatant) -> Texture2D:
	var key: String = hero.name.to_lower()
	if not _texture_cache.has(key):
		_texture_cache[key] = AssetUtils.load_svg_texture("res://assets/party/tokens/token_%s.svg" % key)
	var raw: Variant = _texture_cache[key]
	if raw is Texture2D:
		var tex: Texture2D = raw
		return tex
	return null

func refresh() -> void:
	if session == null or session.active_hero == null or session.active_hero.stats == null:
		return
	var hero: Combatant = session.active_hero
	var st: CharacterStats = hero.stats
	var ctx: Dictionary = DojoSession.weapon_context(hero)

	_avatar.texture = _token_texture(hero)
	_name_lbl.text = hero.name
	_class_lbl.text = "Archetype: %s" % (st.archetype.display_name if st.archetype != null else "Hero")
	_level_lbl.text = "Level %d / %d   •   HP %d / %d" % [st.level, CharacterStats.MAX_LEVEL, hero.hp, hero.max_hp]

	var free_pts: int = st.unspent_attribute_points
	if session.is_allocating_stats:
		_points_btn.text = "Allocating (%d unspent) - click to cancel" % session.pending_free_points
		_points_btn.add_theme_color_override("font_color", DojoTheme.COL_WARN)
	else:
		_points_btn.text = "Unspent Points: %d%s" % [free_pts, " (click to allocate)" if free_pts > 0 else ""]
		_points_btn.add_theme_color_override("font_color", DojoTheme.COL_GOOD if free_pts > 0 else DojoTheme.COL_MUTED)

	for key: String in DojoSession.STAT_KEYS:
		var eff: int = st.get_effective_stat(key, ctx)
		var bonus: int = eff - st.get_base_stat(key)
		var text: String = "%d" % eff
		if bonus > 0:
			text += " (+%d)" % bonus
		var pending: int = session.get_pending(key)
		if session.is_allocating_stats and pending > 0:
			text += " [+%d]" % pending
		var value_raw: Variant = _stat_values.get(key, null)
		if value_raw is Label:
			var value_lbl: Label = value_raw
			value_lbl.text = text
		var plus_raw: Variant = _plus_btns.get(key, null)
		if plus_raw is Button:
			var plus_btn: Button = plus_raw
			plus_btn.visible = session.is_allocating_stats
			plus_btn.disabled = session.pending_free_points <= 0
		var minus_raw: Variant = _minus_btns.get(key, null)
		if minus_raw is Button:
			var minus_btn: Button = minus_raw
			minus_btn.visible = session.is_allocating_stats
			minus_btn.disabled = pending <= 0
		var box_raw: Variant = _stat_boxes.get(key, null)
		if box_raw is DojoTipPanel:
			var box: DojoTipPanel = box_raw
			box.tooltip_text = _stat_tooltip(key, hero, ctx)

	# One stat per row while allocating so the +/- buttons never overflow narrow windows.
	_stats_grid.columns = 1 if session.is_allocating_stats else 2
	_alloc_box.visible = session.is_allocating_stats
	_alloc_lbl.text = "Allocating: %d point(s) remaining" % session.pending_free_points
	_render_masteries(st)

func _render_masteries(st: CharacterStats) -> void:
	DojoTheme.clear_children(_mastery_vbox)
	if st.mastery == null or st.mastery.weapon_masteries.is_empty():
		_mastery_vbox.add_child(DojoTheme.label("— No weapon masteries —", DojoTheme.FONT_SM, DojoTheme.COL_MUTED))
		return
	for w_type: String in st.mastery.weapon_masteries:
		var rank: int = st.mastery.get_weapon_mastery(w_type)
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 6)
		row.add_child(DojoTheme.label(w_type.capitalize(), DojoTheme.FONT_SM, DojoTheme.COL_TEXT))
		var detail: Label = DojoTheme.label("Rank %d  •  +%d Wep Power, -%d Req" % [rank, rank * 2, st.mastery.get_weapon_requirement_discount(w_type)], 9, DojoTheme.COL_WARN)
		detail.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		detail.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
		detail.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		detail.custom_minimum_size = Vector2(60, 0)
		row.add_child(detail)
		_mastery_vbox.add_child(row)

func _stat_tooltip(key: String, hero: Combatant, ctx: Dictionary) -> String:
	var st: CharacterStats = hero.stats
	var role: String = ""
	var full_name: String = key
	var kind: String = "Passive"
	for def: Dictionary in STAT_DEFS:
		if str(def.get("k", "")) == key:
			role = str(def.get("role", ""))
			full_name = str(def.get("name", key))
			kind = str(def.get("kind", "Passive"))
	var base_val: int = st.get_base_stat(key)
	var eff_val: int = st.get_effective_stat(key, ctx)

	var lines: Array[String] = []
	lines.append("[color=#a5d8ff][b]%s (%s)[/b][/color]  [color=#94a3b8]%s[/color]" % [full_name, key, kind])
	lines.append("[color=#94a3b8]%s[/color]" % role)
	lines.append("Base [b]%d[/b]  (Lv %d %s)  →  Effective [color=#4ade80][b]%d[/b][/color]" % [base_val, st.level, st.archetype.display_name if st.archetype != null else "Hero", eff_val])

	var bonus_lines: Array[String] = []
	if st.mastery != null and ctx.has("weapon"):
		var mastery_bonus: int = st.mastery.get_conditional_stat_bonus(key, ctx)
		if mastery_bonus > 0:
			bonus_lines.append("• Mastery (%s): +%d" % [str(ctx["weapon"]).capitalize(), mastery_bonus])
	if hero.inventory != null:
		for eq: Equipment in hero.inventory._iter_equipped_gear():
			var gear_bonus: int = 0
			var raw_bonus: Variant = eq.stat_bonuses.get(key, 0)
			if raw_bonus is int:
				gear_bonus = raw_bonus
			if gear_bonus != 0:
				bonus_lines.append("• %s: %+d" % [eq.name, gear_bonus])
	for mod: StatModifier in st.modifiers:
		if mod.source_id != "gear" and mod.stat_name.to_upper() == key and mod.matches_context(ctx):
			bonus_lines.append("• Buff (%s): %+d" % [mod.source_id, mod.flat_bonus])
	if not bonus_lines.is_empty():
		lines.append("[color=#fbbf24]Bonuses:[/color]\n%s" % "\n".join(bonus_lines))
	return "\n".join(lines)
