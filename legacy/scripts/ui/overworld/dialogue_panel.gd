class_name DialoguePanel
extends PanelContainer

# ==============================================================================
# Tactical RPG - Dialogue panel (S11, spec DLG1-L commit 3; UIR-6 bottom bar)
# Thin view over one DialogueRunner: speaker, text, visible choices (locked ones disabled with their reason), Continue for
# `next` / end nodes, and the turn-in step (recipient picker over the active members, one checkbox per offered loot,
# Confirm / Cancel). Every action goes through the runner; the panel only re-renders. A shop or fight result is handed
# to the owner through the signals; the owner calls shop_closed() when its shop panel closes.
# UIR-6 (layout pick B, a UI preference): a full-width bar along the bottom of the map (bar_layout: BAR_REF_H at
# REF_H, scaled with the view height, at most BAR_MAX_FRAC of it), dark MenuStyle panel with a 2 px top border.
# Left the speaker's portrait (WorldPreview.texture_for; a labelled box without one), middle the gold bold name, the
# wrapping line and the key hint, right a CHOICES_REF_W column of numbered, left-aligned MenuStyle choice buttons
# (1-9 hotkeys; focus and hover share the gold edge). Esc and right click are Farewell when the node offers it, Cancel on
# the turn-in step, Continue on an end node (can_leave). Text follows the one text-size setting (UiTheme.scaled, X8).
# The layout lives in build_layout / bar_layout / dock so another arrangement swaps in one place.
# ==============================================================================

signal shop_opened(shop_qid: String)
signal fight_started(fight: Dictionary)
signal finished()
signal noted(text: String)

const REF_H: float = 540.0   # the reference view height of the layout numbers below
const BAR_REF_H: float = 236.0
const BAR_MAX_FRAC: float = 0.44
const PORTRAIT_REF: Vector2 = Vector2(140, 180)
const CHOICES_REF_W: float = 360.0
const HOTKEYS: int = 9
const FONT_BASE: int = MainMenuView.BASE_FONT   # the menus' Normal size; UiTheme.scaled applies the text size
const HINT_CHOOSE: String = "1-9 choose"
const HINT_LEAVE: String = "Esc or right click: %s"
const LAYER: int = 1   # the bar's canvas layer: above the map and the menu host's "Esc: menu" line, below every popup layer

var runner: DialogueRunner = null
var pending: Dictionary = {}          # the turn-in step: choose()'s {pending, quest, offers, picks_allowed}
var recipient_id: String = ""
var picks: Array[String] = []

var _speaker: Label = null
var _text: Label = null
var _hint: Label = null
var _portrait_box: PanelContainer = null
var _portrait: TextureRect = null
var _portrait_name: Label = null
var _column: ScrollContainer = null
var _choices: VBoxContainer = null
var _turn_in: VBoxContainer = null
var _theme: Theme = null

func _init() -> void:
	_theme = MenuStyle.theme()
	theme = _theme
	add_theme_stylebox_override("panel", bar_box())
	build_layout()
	visible = false

# The bar's panel: opaque dark, square, only a 2 px top border.
static func bar_box() -> StyleBoxFlat:
	var box: StyleBoxFlat = UiTheme.box(Color(UiTheme.COL_PANEL_BG, 1.0), UiTheme.COL_HEADER, 0, 12, 10, 0)
	box.border_width_top = 2
	return box

# Bar geometry for a view of `view` px (pure): {height, portrait (Vector2), choices_w}.
static func bar_layout(view: Vector2) -> Dictionary:
	var k: float = maxf(view.y, 1.0) / REF_H
	return {
		"height": roundf(minf(BAR_REF_H * k, view.y * BAR_MAX_FRAC)),
		"portrait": (PORTRAIT_REF * k).round(),
		"choices_w": roundf(minf(CHOICES_REF_W * k, view.x * 0.4)),
	}

func build_layout() -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 14)
	add_child(row)
	_portrait_box = PanelContainer.new()
	_portrait_box.add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_CARD_BG, UiTheme.COL_BORDER, 4, 4, 4, 1))
	_portrait_box.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
	row.add_child(_portrait_box)
	_portrait = TextureRect.new()
	_portrait.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	_portrait.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	_portrait_box.add_child(_portrait)
	_portrait_name = UiTheme.label("", FONT_BASE, UiTheme.COL_MUTED)
	_portrait_name.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_portrait_name.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	_portrait_name.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_portrait_box.add_child(_portrait_name)
	var mid := VBoxContainer.new()
	mid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	mid.add_theme_constant_override("separation", 6)
	row.add_child(mid)
	_speaker = UiTheme.label("", FONT_BASE, UiTheme.COL_TITLE)
	var bold := FontVariation.new()
	bold.base_font = ThemeDB.fallback_font
	bold.variation_embolden = 0.8
	_speaker.add_theme_font_override("font", bold)
	_speaker.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART   # long names wrap instead of pushing the choice column off screen
	mid.add_child(_speaker)
	_text = UiTheme.label("", FONT_BASE)
	_text.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_text.size_flags_vertical = Control.SIZE_EXPAND_FILL
	mid.add_child(_text)
	_hint = UiTheme.label("", FONT_BASE, UiTheme.COL_MUTED)
	_hint.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	mid.add_child(_hint)
	_column = ScrollContainer.new()
	_column.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	row.add_child(_column)
	var col := VBoxContainer.new()
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_column.add_child(col)
	_turn_in = VBoxContainer.new()   # the turn-in step's picker sits above its Confirm / Cancel buttons
	col.add_child(_turn_in)
	_choices = VBoxContainer.new()
	_choices.add_theme_constant_override("separation", 4)
	col.add_child(_choices)
	fit(Vector2(1280, 720))

# Docks the bar along the bottom of its parent (full width) and sizes it for `view`.
func dock(view: Vector2) -> void:
	set_anchors_preset(Control.PRESET_BOTTOM_WIDE)
	grow_vertical = Control.GROW_DIRECTION_BEGIN
	offset_left = 0.0
	offset_right = 0.0
	offset_bottom = 0.0
	fit(view)

func fit(view: Vector2) -> void:
	var lay: Dictionary = bar_layout(view)
	var h: float = DictUtil.to_float(lay["height"])
	custom_minimum_size = Vector2(0, h)
	offset_top = -h
	_portrait_box.custom_minimum_size = lay["portrait"]
	_column.custom_minimum_size = Vector2(DictUtil.to_float(lay["choices_w"]), 0)

# Font sizes from the one text-size setting (X8): the theme default, the name a step larger, the hint a step smaller.
func apply_text_scale() -> void:
	_theme.default_font_size = UiTheme.scaled(FONT_BASE)
	_speaker.add_theme_font_size_override("font_size", UiTheme.scaled(FONT_BASE + 4))
	_text.add_theme_font_size_override("font_size", UiTheme.scaled(FONT_BASE))
	_hint.add_theme_font_size_override("font_size", UiTheme.scaled(FONT_BASE - 4))
	_portrait_name.add_theme_font_size_override("font_size", UiTheme.scaled(FONT_BASE - 2))

func font_px() -> int:
	return _theme.default_font_size

# Shows the conversation of an opened runner (runner.errors must be empty).
func show_runner(r: DialogueRunner) -> void:
	runner = r
	pending = {}
	visible = true
	refresh()

func is_open() -> bool:
	return runner != null and visible

# Picks visible choice `index` (DialogueRunner.choose) and routes the result.
func pick(index: int) -> Dictionary:
	if runner == null:
		return {}
	var res: Dictionary = runner.choose(index)
	if not DictUtil.to_bool(res.get("ok", false)):
		noted.emit("Refused: %s" % str(res.get("errors", [])))
	elif str(res.get("pending", "")) == "turn_in":
		pending = res
		picks = []
		recipient_id = runner.gs.party.active_ids[0] if not runner.gs.party.active_ids.is_empty() else ""
	_route(res)
	return res

# Continue on a `next` node or an end node (DialogueRunner.advance).
func advance() -> Dictionary:
	if runner == null:
		return {}
	var res: Dictionary = runner.advance()
	if not DictUtil.to_bool(res.get("ok", false)):
		noted.emit("Refused: %s" % str(res.get("errors", [])))
	_route(res)
	return res

func set_recipient(id: String) -> void:
	recipient_id = id

# Ticks or unticks offered loot `lqid` for the turn-in.
func toggle_pick(lqid: String, on: bool) -> void:
	if on and not picks.has(lqid):
		picks.append(lqid)
	elif not on:
		picks.erase(lqid)

# Confirms the turn-in with the chosen recipient and picks (DialogueRunner.confirm_turn_in, all-or-nothing).
func confirm_turn_in() -> Dictionary:
	if runner == null or pending.is_empty():
		return {}
	var res: Dictionary = runner.confirm_turn_in(recipient_id, picks)
	if DictUtil.to_bool(res.get("ok", false)):
		pending = {}
		noted.emit("Quest complete: %s" % str(res.get("granted", {})))
	else:
		noted.emit("Turn-in refused: %s" % str(res.get("errors", [])))
	_route(res)
	return res

func cancel_turn_in() -> void:
	if runner == null:
		return
	var _r: Dictionary = runner.cancel()
	pending = {}
	refresh()

# Called by the owner when its shop panel closes (DialogueRunner.close_shop).
func shop_closed() -> void:
	if runner == null:
		return
	visible = true
	_route(runner.close_shop())

# ------------------------------------------------------------------------------
# Keys and mouse (UIR-6): 1-9 press the numbered button, Esc and right click leave when allowed
# ------------------------------------------------------------------------------

# What Esc / right click does now: "farewell" (the node offers the Farewell service), "cancel" (turn-in step),
# "continue" (an end node without choices), or "" (the conversation must go on: no shortcut).
func leave_action() -> String:
	if runner == null:
		return ""
	if not pending.is_empty():
		return "cancel"
	var v: Dictionary = runner.view()
	if _farewell_index(v) >= 0:
		return "farewell"
	if DictUtil.array_of(v.get("choices", [])).is_empty() and DictUtil.to_bool(v.get("end", false)) and not DictUtil.to_bool(v.get("next", false)):
		return "continue"
	return ""

func can_leave() -> bool:
	return not leave_action().is_empty()

static func _farewell_index(v: Dictionary) -> int:
	var choices: Array = DictUtil.array_of(v.get("choices", []))
	for i: int in range(choices.size()):
		var c: Dictionary = DictUtil.dict_of(choices[i])
		if str(c.get("service", "")) == "farewell" and not DictUtil.to_bool(c.get("locked", false)):
			return i
	return -1

# Esc / right click: does the leave_action; false (nothing done) when the dialogue does not allow leaving.
func leave() -> bool:
	match leave_action():
		"cancel":
			cancel_turn_in()
		"farewell":
			var _r: Dictionary = pick(_farewell_index(runner.view()))
		"continue":
			var _a: Dictionary = advance()
		_:
			return false
	return true

# Hotkey `n` (1-9): presses the n-th numbered button when it is enabled; false otherwise.
func press_number(n: int) -> bool:
	if n < 1 or n > mini(HOTKEYS, _choices.get_child_count()):
		return false
	var btn: Button = _choices.get_child(n - 1) as Button
	if btn == null or btn.disabled:
		return false
	btn.pressed.emit()
	return true

func _usable() -> bool:
	return is_open() and is_visible_in_tree() and can_process()

func _input(event: InputEvent) -> void:
	var mouse: InputEventMouseButton = event as InputEventMouseButton
	if mouse == null or not mouse.pressed or mouse.button_index != MOUSE_BUTTON_RIGHT or not _usable():
		return
	if leave():   # taken before the GUI, like the menus' right click
		get_viewport().set_input_as_handled()

func _unhandled_input(event: InputEvent) -> void:
	if not _usable():
		return
	if event.is_action_pressed("ui_cancel"):
		if leave():
			get_viewport().set_input_as_handled()   # otherwise Esc reaches the game menu
		return
	var key: InputEventKey = event as InputEventKey
	if key == null or not key.pressed or key.echo:
		return
	var n: int = number_of(key)
	if n > 0 and press_number(n):
		get_viewport().set_input_as_handled()

# 1-9 on the main row or the keypad; 0 for any other key.
static func number_of(key: InputEventKey) -> int:
	var code: Key = key.keycode if key.keycode != KEY_NONE else key.physical_keycode
	if code >= KEY_1 and code <= KEY_9:
		return code - KEY_0
	if code >= KEY_KP_1 and code <= KEY_KP_9:
		return code - KEY_KP_0
	return 0

# ------------------------------------------------------------------------------
# Rendering
# ------------------------------------------------------------------------------

func _route(res: Dictionary) -> void:
	for n: Variant in DictUtil.array_of(res.get("notes", [])):
		noted.emit(str(n))   # e.g. the gold-cap note
	if res.has("shop") and DictUtil.to_bool(res.get("ok", false)):
		visible = false
		shop_opened.emit(str(res["shop"]))
		return
	if res.has("fight"):
		_close()
		fight_started.emit(DictUtil.dict_of(res["fight"]))
		return
	if runner != null and runner.closed:
		_close()
		finished.emit()
		return
	refresh()

func _close() -> void:
	runner = null
	pending = {}
	visible = false

# The speaker line as shown (name plus the faction band, REP1-L); for tests.
func speaker_text() -> String:
	return _speaker.text

func hint_text() -> String:
	return _hint.text

# The numbered button labels as shown; for tests.
func choice_texts() -> Array[String]:
	var out: Array[String] = []
	for c: Node in _choices.get_children():
		out.append((c as Button).text)
	return out

# "3  Ask about the ruins" (numbered up to HOTKEYS) plus "  (Needs 1 Fire Bomb)" for a locked one; a reason the
# choice's generated "[KNOW 14] " / "[Wardens friendly] " prefix already says is not repeated.
static func choice_label(index: int, text: String, reason: String = "") -> String:
	var out: String = ("%d  %s" % [index + 1, text]) if index < HOTKEYS else text
	if reason.is_empty() or text.contains("[%s]" % reason.trim_prefix("Needs ")):
		return out
	return out + "  (%s)" % reason

func refresh() -> void:
	apply_text_scale()
	UiTheme.clear_children(_choices)
	UiTheme.clear_children(_turn_in)
	if runner == null:
		return
	var v: Dictionary = runner.view()
	var who: String = str(v.get("speaker_name", ""))
	_speaker.text = who
	if not str(v.get("band", "")).is_empty():   # REP1-L 2.4: the talked-to character's faction band
		_speaker.text += "  (%s)" % RepRules.standing_text(str(v.get("faction_name", "")), str(v["band"]))
	_text.text = str(v.get("text", ""))
	_show_portrait(str(v.get("speaker", "")), who)
	if not pending.is_empty():
		_build_turn_in()
	else:
		var choices: Array = DictUtil.array_of(v.get("choices", []))
		for i: int in range(choices.size()):
			var c: Dictionary = DictUtil.dict_of(choices[i])
			var locked: bool = DictUtil.to_bool(c.get("locked", false))
			_add_choice(choice_label(i, str(c.get("text", "")), str(c.get("reason", "")) if locked else ""), locked, pick.bind(i))
		if choices.is_empty() and (DictUtil.to_bool(v.get("next", false)) or DictUtil.to_bool(v.get("end", false))):
			_add_choice(choice_label(0, "Continue"), false, advance)
	var leave_word: String = {"farewell": "Farewell", "cancel": "Cancel", "continue": "Continue"}.get(leave_action(), "")
	_hint.text = HINT_CHOOSE + ("   " + HINT_LEAVE % leave_word if not leave_word.is_empty() else "")
	if is_inside_tree():
		_focus_first.call_deferred()

func _add_choice(text: String, locked: bool, callback: Callable) -> void:
	var btn := Button.new()
	btn.text = text
	btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
	btn.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	btn.disabled = locked
	btn.focus_mode = Control.FOCUS_NONE if locked else Control.FOCUS_ALL
	btn.add_theme_stylebox_override("hover", choice_hover_box())   # hover shows the same gold edge as focus
	btn.mouse_entered.connect(func() -> void:
		if not btn.disabled and btn.is_inside_tree():
			btn.grab_focus())
	btn.pressed.connect(callback)
	_choices.add_child(btn)

# The hovered choice: MenuStyle's hover fill with the gold focus edge.
static func choice_hover_box() -> StyleBoxFlat:
	return UiTheme.box(MenuStyle.COL_BTN_HOVER, UiTheme.COL_TITLE, 4, 12, 6, 3)

func _focus_first() -> void:
	if not is_visible_in_tree():
		return
	for c: Node in _choices.get_children():
		var b: Button = c as Button
		if b != null and not b.disabled and not b.is_queued_for_deletion():
			b.grab_focus()
			return

# The speaker's portrait (WorldPreview: the character's unit art), else a box with the speaker's name.
func _show_portrait(speaker_qid: String, who: String) -> void:
	var tex: Texture2D = null
	if runner != null and not speaker_qid.is_empty():
		var entry: Dictionary = WorldCatalog.resolve(runner.gs.world_id, "characters", speaker_qid)
		if not entry.is_empty():
			tex = WorldPreview.texture_for("characters", entry)
	_portrait.texture = tex
	_portrait.visible = tex != null
	_portrait_name.text = who
	_portrait_name.visible = tex == null

func has_portrait() -> bool:
	return _portrait.texture != null

func _build_turn_in() -> void:
	_turn_in.add_child(UiTheme.label("Turn in: %s" % str(pending.get("quest", "")), UiTheme.scaled(FONT_BASE - 2), UiTheme.COL_HEADER))
	var who := OptionButton.new()
	for id: String in runner.gs.party.active_ids:
		who.add_item(id)
		if id == recipient_id:
			who.select(who.item_count - 1)
	who.item_selected.connect(func(i: int) -> void: set_recipient(who.get_item_text(i)))
	_turn_in.add_child(who)
	var allowed: int = DictUtil.to_int(pending.get("picks_allowed", 0))
	if allowed > 0:
		_turn_in.add_child(UiTheme.label("Pick up to %d:" % allowed, UiTheme.scaled(FONT_BASE - 4), UiTheme.COL_MUTED))
	for raw: Variant in DictUtil.array_of(pending.get("offers", [])):
		var lqid: String = str(raw)
		var box := CheckBox.new()
		box.text = str(WorldCatalog.resolve(runner.gs.world_id, "loot", lqid).get("name", lqid))
		box.button_pressed = picks.has(lqid)
		box.toggled.connect(func(on: bool) -> void: toggle_pick(lqid, on))
		_turn_in.add_child(box)
	_add_choice(choice_label(0, "Confirm"), false, confirm_turn_in)
	_add_choice(choice_label(1, "Cancel"), false, cancel_turn_in)
