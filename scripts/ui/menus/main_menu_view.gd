class_name MainMenuView
extends Control

# ==============================================================================
# Tactical RPG - Main menu host view (UI1-L commit 3; scene scenes/main_menu.tscn)
# Thin: owns the MenuRouter, renders its top screen (main menu, new game, load / save slots, settings, hub) through the
# per-screen views (UI2-L adds party, level-up, loadout, gear and battle results), shows the router's questions and notices as an overlay, and forwards keys (ui_cancel = Back or No).
# A started game opens the exploration scene (EXP1 region map) when it exists; Esc there returns to the hub.
# ==============================================================================

const GAME_SCENE: String = "res://scenes/overworld/region_map_debug.tscn"
const RUNNER_SCENE: String = "res://scenes/scene_runner.tscn"
const BASE_FONT: int = 18
const FOCUS_META: StringName = &"focus_target"
const COL_BG: Color = Color(0.05, 0.07, 0.10, 1.0)

var router: MenuRouter = null
var game_view: Node = null
var problems: Array[String] = []

var _menu: Control = null
var _screen: Control = null
var _game_hint: Label = null
var _problems: Label = null
var _overlay: PanelContainer = null
var _overlay_text: Label = null
var _overlay_buttons: HBoxContainer = null
var _notice_lines: Array[String] = []

func _ready() -> void:
	WebFonts.install()   # the browser build starts here (application/run/main_scene.web)
	set_anchors_preset(Control.PRESET_FULL_RECT)
	var bg := ColorRect.new()
	bg.color = COL_BG
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(bg)
	var margin := MarginContainer.new()
	margin.set_anchors_preset(Control.PRESET_FULL_RECT)
	for side: String in ["left", "right", "top", "bottom"]:
		margin.add_theme_constant_override("margin_" + side, 48)
	add_child(margin)
	_menu = margin
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 12)
	margin.add_child(col)
	_screen = VBoxContainer.new()
	_screen.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(_screen)
	_problems = Label.new()
	_problems.add_theme_color_override("font_color", UiTheme.COL_BAD)
	_problems.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	col.add_child(_problems)
	_game_hint = text("Esc: back to the hub", UiTheme.COL_MUTED, false)
	_game_hint.set_anchors_preset(Control.PRESET_BOTTOM_RIGHT)
	_game_hint.grow_horizontal = Control.GROW_DIRECTION_BEGIN
	_game_hint.grow_vertical = Control.GROW_DIRECTION_BEGIN
	_game_hint.offset_right = -16
	_game_hint.offset_bottom = -12
	_game_hint.visible = false
	add_child(_game_hint)
	_build_overlay()
	apply_text_scale()
	router = MenuRouter.new()
	router.changed.connect(render)
	router.confirm_needed.connect(_on_confirm_needed)
	router.notice.connect(_on_notice)
	router.game_started.connect(open_game)
	router.quit_requested.connect(_on_quit)
	router.results_done.connect(_on_results_done)
	render()

# The theme's default font size follows the `ui/text_scale` setting.
func apply_text_scale() -> void:
	var t := Theme.new()
	t.default_font_size = roundi(BASE_FONT * DictUtil.to_float(SettingsFile.get_value("ui/text_scale", 1.0), 1.0))
	theme = t

# ------------------------------------------------------------------------------
# Rendering
# ------------------------------------------------------------------------------

func render() -> void:
	if _screen == null:
		return
	UiTheme.clear_children(_screen)
	var id: String = router.top()
	var view: Control = null
	match id:
		MenuRouter.MAIN:
			view = MainScreenView.new()
		MenuRouter.NEW_GAME:
			view = NewGameView.new()
		MenuRouter.LOAD, MenuRouter.SAVE:
			view = SaveSlotsView.new()
		MenuRouter.SETTINGS:
			view = SettingsView.new()
		MenuRouter.HUB:
			view = HubView.new()
		MenuRouter.PARTY:
			view = PartyView.new()
		MenuRouter.LEVEL_UP:
			view = LevelUpView.new()
		MenuRouter.LOADOUT:
			view = LoadoutView.new()
		MenuRouter.GEAR:
			view = GearView.new()
		MenuRouter.RESULTS:
			view = BattleResultsView.new()
		MenuRouter.JOURNAL:
			view = JournalView.new()
	if view == null:
		return
	view.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_screen.add_child(view)
	view.call("build", self, router.model())
	_show_problems()
	focus_first(view)

func show_problems(list: Array[String]) -> void:
	problems = list
	_show_problems()

func _show_problems() -> void:
	if _problems != null:
		_problems.text = "\n".join(PackedStringArray(problems))

# Runs a model action and shows its problems (an empty list clears them).
func act(action: Callable) -> void:
	var res: Variant = action.call()
	var list: Array[String] = []
	if res is Array:
		list.assign(res)
	show_problems(list)

static func title(text: String) -> Label:
	var l := Label.new()
	l.text = text
	l.add_theme_font_size_override("font_size", 34)
	l.add_theme_color_override("font_color", UiTheme.COL_TITLE)
	return l

static func text(value: String, color: Color = UiTheme.COL_TEXT, wrap: bool = true) -> Label:
	var l := Label.new()
	l.text = value
	l.add_theme_color_override("font_color", color)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART if wrap else TextServer.AUTOWRAP_OFF
	return l

# A keyboard-focusable button (ui_accept presses it; arrows and Tab move between buttons).
static func button(label: String, callback: Callable, enabled: bool = true) -> Button:
	var b := Button.new()
	b.text = label
	b.focus_mode = Control.FOCUS_ALL
	b.disabled = not enabled
	b.custom_minimum_size = Vector2(260, 0)
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	b.pressed.connect(callback)
	return b

# A compact button for rows and toolbars (`width` 0 = fit the text).
static func small(b: Button, width: int = 0) -> Button:
	b.custom_minimum_size = Vector2(width, 0)
	return b

static func focus_first(root: Node) -> void:
	if root.has_meta(FOCUS_META):   # a view may name the control to focus (the journal's selected quest)
		MainMenuView._focus.call_deferred(root.get_meta(FOCUS_META))
		return
	for child: Node in root.find_children("*", "BaseButton", true, false):
		var b: BaseButton = child as BaseButton
		if b != null and not b.disabled and b.is_visible_in_tree():
			MainMenuView._focus.call_deferred(b)
			return

static func _focus(target: Variant) -> void:
	if not is_instance_valid(target):
		return
	var b: Control = target as Control
	if b != null and b.is_inside_tree() and b.is_visible_in_tree():
		b.grab_focus()

# ------------------------------------------------------------------------------
# Questions and notices
# ------------------------------------------------------------------------------

func _build_overlay() -> void:
	_overlay = PanelContainer.new()
	_overlay.add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_PANEL_BG, UiTheme.COL_TITLE, 6, 24, 18, 2))
	_overlay.set_anchors_preset(Control.PRESET_CENTER)
	_overlay.visible = false
	add_child(_overlay)
	var v := VBoxContainer.new()
	v.add_theme_constant_override("separation", 14)
	_overlay.add_child(v)
	_overlay_text = text("")
	_overlay_text.custom_minimum_size = Vector2(420, 0)
	v.add_child(_overlay_text)
	_overlay_buttons = HBoxContainer.new()
	v.add_child(_overlay_buttons)

func is_overlay_open() -> bool:
	return _overlay != null and _overlay.visible

func _open_overlay(message: String, buttons: Array[Button]) -> void:
	_overlay_text.text = message
	UiTheme.clear_children(_overlay_buttons)
	for b: Button in buttons:
		b.custom_minimum_size = Vector2(120, 0)
		_overlay_buttons.add_child(b)
	_overlay.visible = true
	_overlay.reset_size()
	_overlay.position = (size - _overlay.size) / 2.0
	MainMenuView._focus.call_deferred(buttons[0])

func _on_confirm_needed(message: String) -> void:
	_open_overlay(message, [button("Yes", answer.bind(true)), button("No", answer.bind(false))] as Array[Button])

# Answers the open question (Yes runs the pending action) or closes a notice.
func answer(yes: bool) -> void:
	_overlay.visible = false
	if router.is_asking():
		if yes:
			router.confirm()
		else:
			router.decline()
		var m: RefCounted = router.model()
		if m != null and "last_errors" in m:
			var errs: Array[String] = []
			errs.assign(DictUtil.array_of(m.get("last_errors")))
			show_problems(errs)
	if not _notice_lines.is_empty():
		_show_next_notice()
	elif game_view == null:
		focus_first(_screen)

func _on_notice(lines: Array[String]) -> void:
	_notice_lines.append_array(lines)
	if not is_overlay_open():
		_show_next_notice()

func _show_next_notice() -> void:
	var lines: Array[String] = _notice_lines.duplicate()
	_notice_lines.clear()
	_open_overlay("Before you continue:\n" + "\n".join(PackedStringArray(lines)), [button("OK", answer.bind(true))] as Array[Button])

# ------------------------------------------------------------------------------
# Game scene, quit, tools
# ------------------------------------------------------------------------------

func open_game(gs: GameState) -> void:
	if not ResourceLoader.exists(GAME_SCENE):
		return   # rule 9: stay on the hub until the exploration scene exists
	var packed: PackedScene = load(GAME_SCENE) as PackedScene
	var node: Node = packed.instantiate() if packed != null else null
	if node == null or not node.has_method("play_state"):
		return
	game_view = node
	if game_view.has_signal("fight_settled"):
		game_view.connect("fight_settled", _on_fight_settled)
	add_child(game_view)
	move_child(game_view, _game_hint.get_index())
	_menu.visible = false
	_game_hint.visible = true
	game_view.call("play_state", gs)

# Back from the game scene to the hub (refused while a fight, conversation or shop waits).
func leave_game() -> bool:
	if game_view == null:
		return false
	if game_view.has_method("busy") and DictUtil.to_bool(game_view.call("busy")):
		show_problems(["Finish the fight or conversation first."] as Array[String])
		return false
	remove_child(game_view)
	game_view.queue_free()
	game_view = null
	_menu.visible = true
	_game_hint.visible = false
	render()
	return true

# UI2-L rule 8: a settled fight shows the results screen over the hidden map; Continue brings the map back.
func _on_fight_settled(res: Dictionary) -> void:
	if game_view == null or not CurrentGame.has_game():
		return
	game_view.visible = false
	_menu.visible = true
	_game_hint.visible = false
	router.push(MenuRouter.RESULTS, {"award": DictUtil.dict_of(res.get("xp", {})), "advanced": res.get("advanced", []), "outcome": str(res.get("outcome", "won"))})

func _on_results_done() -> void:
	if game_view == null:
		return
	game_view.visible = true
	_menu.visible = false
	_game_hint.visible = true

func open_tools() -> void:
	var host: Node = get_parent()
	if host != null and host.has_method("back_to_menu"):
		host.call("back_to_menu")
	elif ResourceLoader.exists(RUNNER_SCENE):
		var _err: Error = get_tree().change_scene_to_file(RUNNER_SCENE)

func _on_quit() -> void:
	var host: Node = get_parent()
	if host != null and host.has_method("back_to_menu"):
		host.call("back_to_menu")
	else:
		get_tree().quit()

func _unhandled_input(event: InputEvent) -> void:
	if not event.is_action_pressed("ui_cancel"):
		return
	if is_overlay_open():
		answer(false)
	elif game_view != null and game_view.visible and game_view.get("battle") != null:
		return   # BATTLE1-L: a battle covers the map; Esc stays with the arena (no hidden "finish the fight" overlay)
	elif game_view != null and game_view.visible:
		var _left: bool = leave_game()
	else:
		router.pop()
	accept_event()
