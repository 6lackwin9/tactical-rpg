class_name StripQuestion
extends PanelContainer

# ==============================================================================
# Tactical RPG - Strip question (COMBAT-UI3)
# A small, non-blocking yes / no question under a planning-strip control: the missing-actions warning of Confirm
# round ("Warrior and Ranger have empty actions. Confirm anyway?" Confirm / Back, Confirm focused) and the tactic
# redraft question (Redraft / Keep). No dim, no modal: the rest of the screen keeps working; Esc or Back closes it.
# Sizes: CombatHudMetrics.QUESTION_W (grows with the text size); all text follows UiTheme.hud_font().
# ==============================================================================

signal answered(asked: String, accept: bool)

var text_lbl: Label
var yes_btn: Button
var no_btn: Button
var kind: String = ""   # what is asked ("confirm", "redraft"); "" while closed

func _init() -> void:
	name = "StripQuestion"
	visible = false
	top_level = true       # floats over the grid below the strip (non-blocking, no backdrop)
	z_index = 70
	mouse_filter = Control.MOUSE_FILTER_STOP
	add_theme_stylebox_override("panel", UiTheme.box(Color(UiTheme.COL_PANEL_BG, 1.0), UiTheme.COL_WARN, 6, 10, 8, 2))
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 8)
	add_child(col)
	text_lbl = UiTheme.label("", UiTheme.hud_font(), UiTheme.COL_TEXT)
	text_lbl.name = "QuestionText"
	text_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	col.add_child(text_lbl)
	var row := HBoxContainer.new()
	row.alignment = BoxContainer.ALIGNMENT_END
	row.add_theme_constant_override("separation", 8)
	col.add_child(row)
	no_btn = _button("QuestionNo", func() -> void: _answer(false))
	row.add_child(no_btn)
	yes_btn = _button("QuestionYes", func() -> void: _answer(true))
	row.add_child(yes_btn)

func _button(node_name: String, on_press: Callable) -> Button:
	var btn := Button.new()
	btn.name = node_name
	btn.focus_mode = Control.FOCUS_ALL
	btn.pressed.connect(on_press)
	return btn

func is_open() -> bool:
	return visible

# Shows `text` with the two answers under `anchor` (a global rect; the popup's right edge lines up with the anchor's),
# kept inside the viewport. The yes button takes the keyboard focus (Enter answers it).
func ask(p_kind: String, text: String, yes_text: String, no_text: String, anchor: Rect2) -> void:
	kind = p_kind
	var fs: int = UiTheme.hud_font()
	var w: float = float(CombatHudMetrics.grown(CombatHudMetrics.QUESTION_W, UiTheme.text_scale()))
	text_lbl.text = text
	text_lbl.add_theme_font_size_override("font_size", fs)
	var font: Font = text_lbl.get_theme_font("font")
	var inner: float = w - 2.0 * 10.0 - 4.0
	text_lbl.custom_minimum_size = Vector2(inner, ceilf(font.get_multiline_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, inner, fs).y))
	for pair: Array in [[yes_btn, yes_text, UiTheme.COL_WARN], [no_btn, no_text, Color(0.38, 0.5, 0.68, 0.9)]]:
		var btn: Button = pair[0]
		btn.text = str(pair[1])
		btn.add_theme_font_size_override("font_size", fs)
		var accent: Color = pair[2]
		btn.add_theme_stylebox_override("normal", UiTheme.box(Color(0.10, 0.14, 0.20, 0.95), accent, 4, 10, 3))
		btn.add_theme_stylebox_override("hover", UiTheme.box(Color(0.16, 0.22, 0.32, 0.95), Color(0.55, 0.75, 1.0), 4, 10, 3))
		btn.add_theme_stylebox_override("focus", UiTheme.box(Color(0, 0, 0, 0), UiTheme.COL_WARN, 4, 10, 3, 2))
	custom_minimum_size = Vector2(w, 0)
	visible = true
	fit_now(self, w)
	var view: Rect2 = get_viewport_rect() if is_inside_tree() else Rect2(Vector2.ZERO, Vector2(1280, 720))
	var pos := Vector2(anchor.end.x - size.x, anchor.end.y + 4.0)
	pos.x = clampf(pos.x, view.position.x + 4.0, maxf(view.position.x + 4.0, view.end.x - size.x - 4.0))
	pos.y = clampf(pos.y, view.position.y, maxf(view.position.y, view.end.y - size.y))
	global_position = pos
	if is_inside_tree():
		yes_btn.grab_focus()
		_refit.call_deferred(w)

func _refit(w: float) -> void:
	if visible:
		fit_now(self, w)

# Sizes a floating panel (no container parent) to its content right now: a fixed width, the height its wrapped
# labels need. Controls never shrink on their own, so a label measured at a transient width would leave it too tall.
static func fit_now(panel: Control, w: float) -> void:
	panel.size = Vector2(w, 0)
	for _pass: int in range(3):
		_sort_now(panel)
		for node: Node in panel.find_children("*", "Label", true, false):
			(node as Label).update_minimum_size()   # a wrapped label re-measures on resize but keeps its cached minimum
		panel.size = Vector2(w, panel.get_combined_minimum_size().y)

static func _sort_now(node: Node) -> void:
	if node is Container:
		(node as Container).notification(Container.NOTIFICATION_SORT_CHILDREN)
	for child: Node in node.get_children():
		_sort_now(child)

func close() -> void:
	kind = ""
	visible = false

func _answer(accept: bool) -> void:
	var asked: String = kind
	close()
	answered.emit(asked, accept)
