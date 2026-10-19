class_name CharacterView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Character popup (UIR-3, UI-REVIEW1 Q2 B and Q4 B): one framed screen with a member strip on top (Q / E
# switches member), the tabs Stats / Level up / Skills / Gear, the tab's body, a fixed detail pane for the focused item
# and the pinned action bar (Reset and tab actions, the primary Confirm last). An X in the header closes it; right click and
# Esc come through MainMenuView.cancel_pressed (the router asks "Discard your changes?" when the open tab holds a draft).
# Each tab is a `parts()` of its own view class (LevelUpView, LoadoutView, GearView; Stats is below), so the old standalone
# screens (opened from the battle results) show the same body in the same frame. Layout lives in frame() and the _strip /
# _tabs builders, easy to swap at the UI review. Renders PartyModel.
# ==============================================================================

const DETAIL_LINES: int = 2
const DETAIL_LINE_H: float = 1.4   # a line's height as a multiple of the font size

static var _focus_name: String = ""   # the control that had the focus before the last redraw (it is rebuilt under the same name)
static var _last_model: int = 0
static var _last_tab: String = ""

var _host: MainMenuView = null
var _model: PartyModel = null

func build(host: MainMenuView, model: RefCounted) -> void:
	_host = host
	_model = model as PartyModel
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	size_flags_vertical = Control.SIZE_EXPAND_FILL
	var detail: Label = new_detail("Move to a row to read what it does.")
	var parts: Dictionary = tab_parts(host, _model, detail)
	adopt(self, frame(host, "Character", parts, detail, [_strip(host, _model), _tabs(host, _model)] as Array[Control]))
	# Focus: a tab switch or a fresh popup starts on the tab button; any other redraw (+, -, Auto, Equip ...) keeps the pressed control.
	var tab_button: Control = find_child("Tab_" + _model.tab, true, false) as Control
	var fresh: bool = _last_model != _model.get_instance_id() or _last_tab != _model.tab
	_last_model = _model.get_instance_id()
	_last_tab = _model.tab
	var keep: Control = null if fresh or _focus_name.is_empty() else find_child(_focus_name, true, false) as Control
	var target: Control = keep if keep is BaseButton and not (keep as BaseButton).disabled else tab_button
	if target != null:
		set_meta(MainMenuView.FOCUS_META, target)
	for n: Node in find_children("*", "BaseButton", true, false):
		(n as BaseButton).focus_entered.connect(_remember_focus.bind(n.name))

static func _remember_focus(node_name: StringName) -> void:
	_focus_name = str(node_name)

# The body and the bar buttons of the open tab: {body: Control, extras: Array[Button], primary: Button (or null)}.
static func tab_parts(host: MainMenuView, m: PartyModel, detail: Label) -> Dictionary:
	match m.tab:
		PartyModel.TAB_LEVEL_UP:
			return LevelUpView.parts(host, m.level_model(), detail)
		PartyModel.TAB_SKILLS:
			return LoadoutView.parts(host, m.loadout_model(), detail)
		PartyModel.TAB_GEAR:
			return GearView.parts(host, m.gear_model(), detail)
	return _stats_parts(host, m)

# Q / E (or the shoulder buttons) switch the member; ignored while a question or the Gear board is open.
func _unhandled_input(event: InputEvent) -> void:
	if _model == null or _host == null or _host.is_overlay_open() or _host.has_modal() or not is_visible_in_tree():
		return
	var step: int = 0
	var key: InputEventKey = event as InputEventKey
	var pad: InputEventJoypadButton = event as InputEventJoypadButton
	if key != null and key.pressed and not key.echo:
		step = -1 if key.keycode == KEY_Q else (1 if key.keycode == KEY_E else 0)
	elif pad != null and pad.pressed:
		step = -1 if pad.button_index == JOY_BUTTON_LEFT_SHOULDER else (1 if pad.button_index == JOY_BUTTON_RIGHT_SHOULDER else 0)
	if step != 0:
		_host.act(_model.cycle_member.bind(step))
		accept_event()

# ------------------------------------------------------------------------------
# Shared pieces (the standalone Level up / Skills / Gear screens use them too)
# ------------------------------------------------------------------------------

# The fixed detail pane: a bordered label with room for DETAIL_LINES lines at the current text size, so nothing shifts.
static func new_detail(first_text: String = "") -> Label:
	var l: Label = MainMenuView.text(first_text, UiTheme.COL_TEXT)
	l.name = "DetailText"
	l.add_theme_stylebox_override("normal", UiTheme.box(Color(0.08, 0.10, 0.14, 1.0), UiTheme.COL_BORDER, 4, 10, 6, 1))
	var font_px: float = float(MainMenuView.BASE_FONT) * DictUtil.to_float(SettingsFile.get_value("ui/text_scale", 1.0), 1.0)
	l.set_meta("box_h", ceilf(font_px * DETAIL_LINE_H * DETAIL_LINES) + 12.0)
	l.vertical_alignment = VERTICAL_ALIGNMENT_TOP
	return l

# The pane's fixed box: a plain Control of a set height that clips the wrapped text, so a long line never grows the popup.
static func _detail_box(detail: Label) -> Control:
	var box := Control.new()
	box.name = "DetailPane"
	box.custom_minimum_size = Vector2(0, DictUtil.to_float(detail.get_meta("box_h", 60.0)))
	box.clip_contents = true
	box.add_child(detail)
	detail.set_anchors_preset(Control.PRESET_FULL_RECT)
	return box

# Focus or hover on `c` shows `text` in the pane (Q4 B: works from the keyboard and a controller, not only the mouse).
static func show_on_focus(detail: Label, c: Control, text: String) -> void:
	c.focus_entered.connect(_set_detail.bind(detail, text))
	c.mouse_entered.connect(_set_detail.bind(detail, text))

static func _set_detail(detail: Label, text: String) -> void:
	if is_instance_valid(detail):
		detail.text = text

# The framed screen: header (title centred, X at the right), `above` rows, the body in the scroll, the detail pane and
# the bar (parts.extras then parts.primary last). The X is the only way out besides Esc and right click; no Back button.
static func frame(host: MainMenuView, title_text: String, parts: Dictionary, detail: Label, above: Array[Control] = []) -> ScreenFrame:
	var extras: Array[Button] = []
	extras.assign(parts.get("extras", []))
	var primary_v: Variant = parts.get("primary")
	var primary: Button = primary_v if primary_v is Button else null
	var body_v: Variant = parts["body"]
	var body: Control = body_v if body_v is Control else Control.new()
	var f: ScreenFrame = ScreenFrame.make(title_text, null, primary, extras)
	var header := HBoxContainer.new()
	header.name = "FrameHeader"
	var pad := Control.new()
	pad.custom_minimum_size = Vector2(MainMenuView.W_STEP, 0)
	header.add_child(pad)
	f.remove_child(f.title_label)
	header.add_child(f.title_label)
	var close: Button = MainMenuView.step(MainMenuView.button("✕", host.router.pop))
	close.name = "CloseButton"
	close.tooltip_text = "Close (Esc or right click)"
	header.add_child(close)
	f.add_child(header)
	f.move_child(header, 0)
	var at: int = 1
	for c: Control in above:
		f.add_child(c)
		f.move_child(c, at)
		at += 1
	f.column.add_child(body)
	var box: Control = _detail_box(detail)
	f.add_child(box)
	f.move_child(box, f.scroll.get_index() + 1)
	var first: Button = null   # the first live control of the body or the bar takes the focus (never the X)
	for n: Node in body.find_children("*", "BaseButton", true, false) + f.bar.find_children("*", "BaseButton", true, false):
		if first == null and n is Button and not (n as Button).disabled:
			first = n as Button
	if first != null:
		f.set_meta(MainMenuView.FOCUS_META, first)
	return f

# Adds the frame to `view` and hands its focus target on (MainMenuView.focus_first reads it from the view).
static func adopt(view: Control, f: ScreenFrame) -> void:
	view.add_child(f)
	if f.has_meta(MainMenuView.FOCUS_META):
		view.set_meta(MainMenuView.FOCUS_META, f.get_meta(MainMenuView.FOCUS_META))

# The derived block; with `after` non-empty each changed value shows "before → after".
static func derived_grid(before: Dictionary, after: Dictionary) -> GridContainer:
	var grid := GridContainer.new()
	grid.name = "DerivedGrid"
	grid.columns = 2
	grid.add_theme_constant_override("h_separation", 18)
	for key: String in PartyPreview.DERIVED_LABELS:
		grid.add_child(MainMenuView.text(str(PartyPreview.DERIVED_LABELS[key]), UiTheme.COL_MUTED, false))
		var was: String = str(before.get(key, "-"))
		var now: String = str(after.get(key, was))
		grid.add_child(MainMenuView.text(was if now == was else "%s → %s" % [was, now], UiTheme.COL_TEXT if now == was else UiTheme.COL_GOOD, false))
	return grid

# A grid cell label that right-aligns numbers.
static func cell(value: String, color: Color = UiTheme.COL_TEXT, right: bool = true) -> Label:
	var l: Label = MainMenuView.text(value, color, false)
	if right:
		l.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	return l

# ------------------------------------------------------------------------------
# Member strip and tab bar
# ------------------------------------------------------------------------------

func _strip(host: MainMenuView, m: PartyModel) -> Control:
	var strip := HFlowContainer.new()
	strip.name = "MemberStrip"
	strip.alignment = FlowContainer.ALIGNMENT_CENTER
	strip.add_theme_constant_override("h_separation", 8)
	strip.add_child(MainMenuView.text("Q", UiTheme.COL_MUTED, false))
	for p: Dictionary in m.members():
		var id: String = str(p["id"])
		var pick: Button = MainMenuView.small(MainMenuView.button("%s  Lv %d%s" % [str(p["name"]), DictUtil.to_int(p["level"]), "  +%d" % DictUtil.to_int(p["unspent"]) if DictUtil.to_int(p["unspent"]) > 0 else ""], host.act.bind(m.select.bind(id))))
		pick.name = "Member_" + id
		pick.tooltip_text = "%s, %s%s" % [str(p["name"]), str(p["class"]), "" if DictUtil.to_bool(p["active"]) else " (reserve)"]
		MenuStyle.mark_selected(pick, id == m.selected)
		strip.add_child(pick)
	strip.add_child(MainMenuView.text("E", UiTheme.COL_MUTED, false))
	return strip

func _tabs(host: MainMenuView, m: PartyModel) -> Control:
	var bar := HBoxContainer.new()
	bar.name = "TabBar"
	bar.alignment = BoxContainer.ALIGNMENT_CENTER
	bar.add_theme_constant_override("separation", 8)
	for id: String in PartyModel.TABS:
		var label: String = str(PartyModel.TAB_LABELS[id])
		if id == PartyModel.TAB_LEVEL_UP and m.unspent() > 0:
			label += " (%d)" % m.unspent()
		var b: Button = MainMenuView.small(MainMenuView.button(label, host.act.bind(m.switch_tab.bind(id))))
		b.name = "Tab_" + id
		MenuStyle.mark_selected(b, id == m.tab)
		bar.add_child(b)
	return bar

# ------------------------------------------------------------------------------
# Stats tab
# ------------------------------------------------------------------------------

static func _stats_parts(host: MainMenuView, m: PartyModel) -> Dictionary:
	var body := VBoxContainer.new()
	body.name = "StatsBody"
	body.add_theme_constant_override("separation", 10)
	var sm: StatsModel = m.stats()
	var xp: Dictionary = sm.xp()
	if xp.is_empty():
		return {"body": body, "extras": [] as Array[Button], "primary": null}
	body.add_child(MainMenuView.text("Level %d   ·   XP %d (%d to next)   ·   Gold %d" % [DictUtil.to_int(xp["level"]), DictUtil.to_int(xp["total"]), DictUtil.to_int(xp["to_next"]), m.gold()], UiTheme.COL_TEXT, false))
	body.add_child(_active_box(host, m))
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 32)
	body.add_child(row)
	var grid := GridContainer.new()
	grid.name = "StatsGrid"
	grid.columns = 5
	grid.add_theme_constant_override("h_separation", 18)
	row.add_child(grid)
	for head: String in ["Stat", "Base", "Growth", "Spent", "Total"]:
		grid.add_child(cell(head, UiTheme.COL_MUTED, head != "Stat"))
	var stats: Dictionary = sm.stats()
	for s: String in StatRules.STATS:
		var v: Dictionary = DictUtil.dict_of(stats.get(s, {}))
		grid.add_child(cell(s, UiTheme.COL_TEXT, false))
		for k: String in ["base", "growth", "spend", "total"]:
			grid.add_child(cell(str(DictUtil.to_int(v.get(k, 0)))))
	row.add_child(derived_grid(sm.derived(), {}))
	var passives: PackedStringArray = PackedStringArray()
	for p: Dictionary in m.passives():
		passives.append("%s (%s)" % [str(p["name"]), str(p["source"])])
	body.add_child(MainMenuView.text("Passives: " + (", ".join(passives) if not passives.is_empty() else "none"), UiTheme.COL_MUTED))
	return {"body": body, "extras": [] as Array[Button], "primary": null}

# "In party (n/max)": a CheckBox, disabled with the reason beside it when it can't change (X: no mystery greying).
static func _active_box(host: MainMenuView, m: PartyModel) -> Control:
	var line := HBoxContainer.new()
	line.add_theme_constant_override("separation", 16)
	var member: Dictionary = {}
	for p: Dictionary in m.members():
		if str(p["id"]) == m.selected:
			member = p
	var active: bool = DictUtil.to_bool(member.get("active", false))
	var box := CheckBox.new()
	box.name = "InPartyBox"
	box.text = "In party (%d/%d)" % [m.active_count(), m.max_active()]
	box.button_pressed = active
	box.focus_mode = Control.FOCUS_ALL
	var why: String = ""
	if active and m.active_count() <= 1:
		why = PartyModel.KEEP_ONE
	elif not active and m.active_count() >= m.max_active():
		why = PartyModel.MAX_ACTIVE % m.max_active()
	box.disabled = not why.is_empty()
	var id: String = m.selected
	box.pressed.connect(func() -> void:
		host.act(m.toggle_active.bind(id))
		host.render())
	line.add_child(box)
	if not why.is_empty():
		line.add_child(MainMenuView.text(why, UiTheme.COL_MUTED, false))
	return line
