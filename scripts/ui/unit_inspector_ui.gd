class_name UnitInspectorUI
extends PanelContainer

# ==============================================================================
# Tactical RPG - Unit hover card (COMBAT-UI2-L2; replaces the old Unit Inspector panel)
# A floating, read-only card the grid panel shows beside a hovered, focused or pinned unit:
# portrait, HP and AP gauges, stance, speed, reach, position, target, effects, gear (allies, text only), then
# the plan steps (ally) or the AI intel and the numbers versus the selected hero (hostile). Content is rebuilt
# on every update so the text always follows UiTheme.text_scale() (14 px floor); the card sizes itself to its
# content (settle) and never scrolls or clips. Placement lives in CombatGridPanel.
# ==============================================================================

signal pin_toggled(pinned: bool)
signal card_clicked()   # a click anywhere on the card pins it

const CARD_W: float = 270.0   # at Normal text; grows with the text size

var name_lbl: Label
var pin_btn: Button
var is_pinned: bool = false
var current_inspected_combatant: Combatant = null
var _body: VBoxContainer

func _init() -> void:
	name = "UnitHoverCard"
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP
	z_index = 50
	add_theme_stylebox_override("panel", UiTheme.box(Color(0.06, 0.08, 0.12, 0.97), Color(0.35, 0.55, 0.85, 0.9), 6, 8, 6))
	_body = VBoxContainer.new()
	_body.add_theme_constant_override("separation", 2)
	add_child(_body)
	gui_input.connect(func(event: InputEvent) -> void:
		var mouse_event := event as InputEventMouseButton
		if mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT:
			card_clicked.emit()
	)

func card_width() -> float:
	return roundf(CARD_W * maxf(1.0, UiTheme.text_scale()))

func set_pinned(value: bool) -> void:
	is_pinned = value
	if pin_btn != null:
		pin_btn.text = "Unpin" if value else "Pin"
		pin_btn.tooltip_text = "Click to unpin" if value else "Keep this card open (Esc or a click elsewhere closes it)"
	add_theme_stylebox_override("panel", UiTheme.box(Color(0.06, 0.08, 0.12, 0.97), UiTheme.COL_WARN if value else Color(0.35, 0.55, 0.85, 0.9), 6, 8, 6, 2 if value else 1))

# Rebuilds the card for one unit. `intel` is [{"name", "desc", "color"}] (hostile AI behaviour); `plan_lines` the
# ally's plan steps; `intent` a hostile's tell ({first, head, lines}, INTENT1-U); `versus_lines` the numbers against the selected hero (hostile cards). `compact` is the safety
# valve for a board too short for the full card: HP and AP share one line and no gauge bars are drawn. `tight` (hostile with
# an intent tell) is the second valve: the AI behaviour and versus sections are left out, the tell stays.
func update_inspector(
	selected_combatant: Combatant,
	is_ally: bool,
	pos_text: String,
	sprite_texture: Texture2D,
	snap: BlockSnapshot = null,
	intel: Array[Dictionary] = [],
	plan_lines: Array[String] = [],
	versus_lines: Array[String] = [],
	compact: bool = false,
	intent: Dictionary = {},
	tight: bool = false
) -> void:
	if not selected_combatant:
		return
	var c: Combatant = selected_combatant
	var cur_hp: int = c.hp
	var max_hp: int = c.max_hp
	var stance: CombatTypes.StanceType = c.current_stance
	var target_val: Variant = c.active_target
	var boost_rds: int = c.attack_boost_rounds_remaining
	var withdrawn: bool = c.is_withdrawn
	var boost_mult: float = c.attack_boost_multiplier
	var cur_ap: int = c.ap
	if snap != null:
		cur_ap = DictUtil.int_of(snap.aps, c, cur_ap)
		boost_mult = DictUtil.float_of(snap.attack_boost_mult, c, boost_mult)
		var raw_hp: Variant = snap.hps.get(c, cur_hp)
		cur_hp = raw_hp if raw_hp is int else cur_hp
		var raw_max: Variant = snap.max_hps.get(c, max_hp)
		max_hp = raw_max if raw_max is int else max_hp
		var raw_stance: Variant = snap.stances.get(c, stance)
		stance = raw_stance if raw_stance is CombatTypes.StanceType else stance
		target_val = snap.targets.get(c, target_val)
		var raw_rds: Variant = snap.attack_boost_rds.get(c, boost_rds)
		boost_rds = raw_rds if raw_rds is int else boost_rds
		var raw_withdrawn: Variant = snap.is_withdrawn.get(c, withdrawn)
		withdrawn = raw_withdrawn if raw_withdrawn is bool else withdrawn
	var alive: bool = cur_hp > 0
	current_inspected_combatant = c

	for child: Node in _body.get_children():
		_body.remove_child(child)
		child.queue_free()
	var fs: int = UiTheme.hud_font()
	var team_col: Color = Color(0.4, 0.75, 1.0) if is_ally else Color(1.0, 0.45, 0.45)
	custom_minimum_size = Vector2(card_width(), 0)

	# Header: portrait, name + team, pin button.
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 6)
	_body.add_child(head)
	var avatar: TextureRect = UiTheme.icon_rect(sprite_texture, 2 * CombatHudMetrics.line_height(fs))
	head.add_child(avatar)
	var names := VBoxContainer.new()
	names.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	names.add_theme_constant_override("separation", 0)
	head.add_child(names)
	var names_w: float = _inner_width() - float(2 * CombatHudMetrics.line_height(fs)) - float(fs) * 4.5 - 12.0   # beside the portrait and the Pin button
	name_lbl = _line(names, c.name, UiTheme.hud_font(16), Color.WHITE, names_w)
	var team_text: String = ("Hero" if is_ally else "Hostile") + ("" if alive else " - defeated") + (" - withdrew" if withdrawn else "")
	_line(names, team_text, fs, team_col, names_w)
	pin_btn = Button.new()
	pin_btn.name = "PinButton"
	pin_btn.focus_mode = Control.FOCUS_NONE
	pin_btn.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
	pin_btn.add_theme_font_size_override("font_size", fs)
	pin_btn.pressed.connect(func() -> void: pin_toggled.emit(not is_pinned))
	head.add_child(pin_btn)
	set_pinned(is_pinned)

	if compact:
		_line(_body, "HP %d/%d   AP %d/%d%s" % [cur_hp, max_hp, cur_ap, c.max_ap, " (debt)" if cur_ap < 0 else ""], fs, Color(0.9, 0.95, 1.0))
	else:
		_gauge("HP", "%d/%d" % [cur_hp, max_hp], cur_hp, max_hp, _hp_color(cur_hp, max_hp), Color(0.9, 0.4, 0.4), fs)
		_gauge("AP", ("%d/%d" % [cur_ap, c.max_ap]) + (" (debt)" if cur_ap < 0 else ""), maxi(0, cur_ap), maxi(1, c.max_ap), Color(0.2, 0.6, 0.95), Color(0.3, 0.75, 1.0), fs)

	var band: Vector2i = SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(c)
	var reach_text: String = ("Reach %d tile" % band.y) if band.x == band.y else ("Reach %d-%d tiles" % [band.x, band.y])
	_line(_body, "Stance %s - %s" % [CombatTypes.stance_to_string(stance), pos_text.trim_prefix("Pos: ")], fs, Color(0.95, 0.85, 0.4))
	_line(_body, "Speed %d - %s" % [c.speed, reach_text], fs, Color(0.7, 0.8, 0.9))
	if target_val is Combatant:
		var tgt: Combatant = target_val
		_line(_body, "Target: " + tgt.name, fs, Color(0.9, 0.75, 0.4))

	var effects: Array[String] = []
	if boost_rds > 0:
		effects.append("Focus x%.2f ATK (%d rds)" % [boost_mult, boost_rds])
	for status: Variant in c.statuses:
		effects.append(StatusEffects.chip_text(c, str(status)))
	_line(_body, "Effects: " + (", ".join(effects) if not effects.is_empty() else "None active"), fs, Color(0.7, 0.9, 0.7))

	if is_ally:
		var gear: String = "None"
		if c.inventory != null:
			var wpn: Equipment = c.inventory.get_main_weapon()
			gear = wpn.name if wpn != null else "Unarmed"
		_line(_body, "Gear: " + gear, fs, Color(0.9, 0.8, 0.4))   # read-only text: gear changes outside battle
		_section("PLAN", Color(0.65, 0.8, 1.0), fs)
		if plan_lines.is_empty():
			_line(_body, "No plan", fs, UiTheme.COL_WARN)
		for text: String in plan_lines:
			_line(_body, text, fs, Color(0.8, 0.88, 0.96))
	else:
		if not tight or intent.is_empty():
			_section("THREAT INTEL (AI BEHAVIOUR)", Color(1.0, 0.45, 0.45), fs)
		if not alive:
			_line(_body, "Defeated.", fs, UiTheme.COL_MUTED)
		elif withdrawn:
			_line(_body, "Withdrew through the escape boundary.", fs, UiTheme.COL_MUTED)
		elif not tight or intent.is_empty():
			for act: Dictionary in intel:
				var act_col: Color = act.get("color", Color.WHITE)
				var desc: String = str(act.get("desc", ""))
				_line(_body, str(act.get("name", "")) + ((" - " + desc) if not desc.is_empty() else ""), fs, act_col)
			if intel.is_empty():
				_line(_body, "Hostiles plan in secret; their commands show as blocks resolve.", fs, UiTheme.COL_MUTED)
		if not intent.is_empty() and alive and not withdrawn:   # INTENT1-U: this round's tell
			_section("INTENT", Color(1.0, 0.45, 0.45), fs)
			_line(_body, str(intent.get("first", "")), fs, Color(1.0, 0.8, 0.6))
			var plan_head: String = str(intent.get("head", ""))
			if not plan_head.is_empty():
				_line(_body, plan_head, fs, Color(0.55, 0.9, 1.0))
				for text: Variant in DictUtil.array_of(intent.get("lines", [])):
					_line(_body, str(text), fs, Color(0.85, 0.9, 0.95))
		if not versus_lines.is_empty() and not tight:
			_section("VERSUS YOUR HERO", Color(0.98, 0.78, 0.3), fs)
			for text: String in versus_lines:
				_line(_body, text, fs, Color(0.85, 0.9, 0.95))

# Sizes the card to its content right now (containers otherwise settle on the next frame): the width is fixed,
# the height is whatever the wrapped text needs.
func settle() -> void:
	size = Vector2(card_width(), 0)
	for _pass: int in range(3):
		_sort_now(self)
		size = Vector2(card_width(), get_combined_minimum_size().y)

static func _sort_now(node: Node) -> void:
	if node is Container:
		(node as Container).notification(Container.NOTIFICATION_SORT_CHILDREN)
	for child: Node in node.get_children():
		_sort_now(child)

# True when the card's rect holds all its content (nothing clipped).
func content_fits() -> bool:
	return size.y + 0.5 >= get_combined_minimum_size().y and size.x + 0.5 >= get_combined_minimum_size().x

# A wrapped line: its minimum height is measured here for the given width, so the card's size does not depend on
# the container pass order (a wrapped label otherwise reports its height for whatever width it had last).
func _line(parent: Container, text: String, fs: int, color: Color, width: float = -1.0) -> Label:
	var lbl: Label = UiTheme.label(text, fs, color)
	lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var w: float = width if width > 0.0 else _inner_width()
	var font: Font = lbl.get_theme_font("font")
	var flags: int = TextServer.BREAK_WORD_BOUND | TextServer.BREAK_ADAPTIVE | TextServer.BREAK_MANDATORY
	var measured: Vector2 = font.get_multiline_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, w, fs, -1, flags)
	lbl.custom_minimum_size = Vector2(w, ceilf(measured.y))   # a label's own minimum (same width) takes over if it is taller
	parent.add_child(lbl)
	return lbl

func _inner_width() -> float:
	return card_width() - 2.0 * 8.0 - 2.0   # the style's content margins and its border

func _section(text: String, color: Color, fs: int) -> void:
	_line(_body, text, fs, color)

func _hp_color(hp: int, max_hp: int) -> Color:
	var ratio: float = float(hp) / float(maxi(1, max_hp))
	if ratio > 0.5:
		return Color(0.25, 0.75, 0.35)
	return Color(0.9, 0.75, 0.2) if ratio > 0.25 else Color(0.9, 0.25, 0.25)

func _gauge(tag: String, value_text: String, value: int, max_value: int, fill: Color, tag_color: Color, fs: int) -> void:
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 1)
	_body.add_child(box)
	var row := HBoxContainer.new()
	box.add_child(row)
	row.add_child(UiTheme.label(tag, fs, tag_color))
	row.add_spacer(false)
	row.add_child(UiTheme.label(value_text, fs, Color(0.9, 0.95, 1.0)))
	var bar := ProgressBar.new()
	bar.custom_minimum_size = Vector2(0, 6)
	bar.show_percentage = false
	bar.max_value = max_value
	bar.value = value
	var bg := StyleBoxFlat.new()
	bg.bg_color = Color(0.12, 0.12, 0.16, 0.9)
	bg.set_corner_radius_all(3)
	var fill_box := StyleBoxFlat.new()
	fill_box.bg_color = fill
	fill_box.set_corner_radius_all(3)
	bar.add_theme_stylebox_override("background", bg)
	bar.add_theme_stylebox_override("fill", fill_box)
	box.add_child(bar)
