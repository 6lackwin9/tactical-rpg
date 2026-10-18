class_name MainMenuView
extends Control

# ==============================================================================
# Tactical RPG - Main menu host view (UI1-L commit 3; scene scenes/main_menu.tscn)
# Thin: owns the MenuRouter, renders its top screen (main menu, new game, load / save slots, settings, hub) through the
# per-screen views (UI2-L adds party, level-up, loadout, gear and battle results), shows the router's questions and notices as an overlay, and forwards keys (ui_cancel = Back or No).
# A started game opens the exploration scene (EXP1 region map) when it exists (router stack bottom PLAY); Esc or the
# Menu button there open the game menu (HUB) as a popup over the paused map, and every screen opened from it (party,
# level up, skills, gear, journal, save, load, settings) shows in the same popup (UIR-2, UI-REVIEW1 Q1 B).
# ==============================================================================

const GAME_SCENE: String = "res://scenes/overworld/region_map_debug.tscn"
const RUNNER_SCENE: String = "res://scenes/scene_runner.tscn"
const BASE_FONT: int = 18
const W_WIDE: int = 260   # UIR-1 (X2) button width classes: wide (menu entries), row (row actions), step (- / + and icons)
const W_ROW: int = 140
const W_STEP: int = 48
const FOCUS_META: StringName = &"focus_target"
const COL_BG: Color = Color(0.05, 0.07, 0.10, 1.0)
const POPUP_MAX: Vector2 = Vector2(900, 620)   # the settings popup's (and game menu's) largest size; it shrinks to fit the viewport
const POPUP_WIDE: Vector2 = Vector2(1216, 656)   # UIR-2: the in-play party / journal / save / load screens (1280x720 minus the margin)
const POPUP_MARGIN: float = 32.0
# UIR-2: in-play screens without their own ScreenFrame yet scroll inside the popup (UIR-4 frames the journal and drops out of this list)
const UNFRAMED_POPUP: Array[String] = [MenuRouter.JOURNAL]
const POPUP_DIM: Color = Color(0, 0, 0, 0.5)
const RESULTS_DIM: Color = Color(0, 0, 0, 0.35)   # BATTLE-FLOW: lighter, the battle stays readable behind the results
const POPUP_LAYER: int = 30   # above the arena's canvas layers, so the popup shows over a mounted battle
const SETTINGS_LAYER: int = 31   # CONTROLS-HELP: the settings popup, above the results popup, so it opens over any game scene (map, dialogue, combat)
const MODAL_LAYER: int = 32   # the Gear board, above every popup layer
const QUESTION_LAYER: int = 33   # the question overlay, above the popup and the Gear board so it never hides under them
const PILL_LAYER: int = 29   # the in-game Menu button, below both popups

var router: MenuRouter = null
var game_view: Node = null
var problems: Array[String] = []

var _overlay_default: Button = null
var _menu: Control = null
var _screen: Control = null
var _game_hint: Label = null
var _problems: Label = null
var _popup: Control = null         # SETTINGS-POPUP: the popup layer (dimmed backdrop + panel) over the screen below
var _popup_panel: PanelContainer = null
var _slayer: CanvasLayer = null    # CONTROLS-HELP: holds the settings popup
var _stheme: Theme = null
var _game_mode: Node.ProcessMode = Node.PROCESS_MODE_INHERIT   # game_view's process_mode before the popup paused it
var _menu_pill: Button = null      # CONTROLS-HELP: "☰ Menu" over the game scene; opens the game menu (UIR-2)
var _popup_max: Vector2 = POPUP_MAX   # the open popup's largest size (UIR-2: wide for in-play screens)
var _rlayer: CanvasLayer = null    # BATTLE-FLOW: the battle results popup, in a canvas layer above the arena's (a Control in this view would sit below it)
var _rtheme: Theme = null
var _rpopup: Control = null
var _rpanel: PanelContainer = null
var _results_pill: Button = null   # "Victory ▴" restore tab while the results popup is minimised
var _overlay: PanelContainer = null
var _mlayer: CanvasLayer = null    # holds the Gear board while it is open
var _qtheme: Theme = null
var _overlay_text: Label = null
var _overlay_buttons: HBoxContainer = null
var _notice_lines: Array[String] = []
var _confirm: SettingsModel = null   # DISPLAY1: the settings model whose window change waits for Keep
var _volumes_previewed: bool = false   # the buses hold an unsaved draft level

func _ready() -> void:
	WebFonts.install()   # the browser build starts here (application/run/main_scene.web)
	SettingsModel.push_to_listeners()   # saved animation/volume settings load here on web (idempotent after the runner)
	DisplaySettings.apply_saved()   # window mode, size, vsync and frame cap, once (DISPLAY1; a no-op on web and headless)
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
	_menu.theme = MenuStyle.theme()   # bordered buttons for the menu screens only (SETTINGS-UI)
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
	_game_hint = text("Esc: menu", UiTheme.COL_MUTED, false)
	_game_hint.set_anchors_preset(Control.PRESET_BOTTOM_RIGHT)
	_game_hint.grow_horizontal = Control.GROW_DIRECTION_BEGIN
	_game_hint.grow_vertical = Control.GROW_DIRECTION_BEGIN
	_game_hint.offset_right = -16
	_game_hint.offset_bottom = -12
	_game_hint.visible = false
	add_child(_game_hint)
	_build_popup()
	_build_overlay()
	apply_text_scale()
	router = MenuRouter.new()
	router.play_scene = ResourceLoader.exists(GAME_SCENE)   # rule 9: without the exploration scene a started game opens the hub full screen
	router.changed.connect(render)
	router.confirm_needed.connect(_on_confirm_needed)
	router.notice.connect(_on_notice)
	router.game_started.connect(open_game)
	router.quit_requested.connect(_on_quit)
	router.results_done.connect(_on_results_done)
	MusicDirector.request(self, MusicDirector.CTX_MENU, MusicDirector.MENU)   # MUSIC1
	render()

# SOUND-PREVIEW: a volume slider of the settings popup moved; the bus follows the draft and a sample plays (throttled).
func preview_volume(setting: String, linear: float) -> void:
	var audio: AudioDirector = _director(true)
	if audio != null:
		_volumes_previewed = true
		var _played: bool = audio.preview(setting, linear)

# SOUND-MUTE: the Mute box of the popup was toggled; Master follows the draft (restored on close without Apply).
func preview_mute(on: bool) -> void:
	_volumes_previewed = true
	AudioDirector.set_mute(on)

# The newest director; `make` adds one to this view when none exists (the menu run on its own, as the web main scene).
func _director(make: bool) -> AudioDirector:
	var found: AudioDirector = AudioDirector.active
	if found == null and make:
		found = AudioDirector.new()
		add_child(found)
	return found

# A settings re-render: the bus holds the draft level when it differs from the saved one (no sample).
func sync_volume(setting: String, linear: float) -> void:
	var audio: AudioDirector = _director(false)
	if audio == null:
		return
	if _volumes_previewed or not is_equal_approx(linear, SettingsFile.get_float(setting)):
		_volumes_previewed = true
		audio.set_bus_linear(setting, linear)

# The saved bus volumes come back (the popup closed without Apply, or another screen is showing).
func restore_volumes() -> void:
	_volumes_previewed = false
	AudioDirector.apply_saved()

# The theme's default font size follows the `ui/text_scale` setting.
func apply_text_scale(preview: float = 0.0) -> void:
	var text_scale: float = preview if preview > 0.0 else DictUtil.to_float(SettingsFile.get_value("ui/text_scale", 1.0), 1.0)
	var t := Theme.new()
	t.default_font_size = roundi(BASE_FONT * text_scale)
	theme = t   # `preview` > 0 shows an unsaved size (the settings screen); every other render goes back to the saved one
	if _stheme != null:
		_stheme.default_font_size = t.default_font_size   # the settings popup sits in its own canvas layer too
	if _qtheme != null:
		_qtheme.default_font_size = t.default_font_size   # the question layer is not under this Control either
	if _rtheme != null:
		_rtheme.default_font_size = t.default_font_size   # the results layer is not under this Control, so it gets the size itself

# ------------------------------------------------------------------------------
# Rendering
# ------------------------------------------------------------------------------

func render() -> void:
	if _screen == null:
		return
	UiTheme.clear_children(_screen)
	UiTheme.clear_children(_popup_panel)
	UiTheme.clear_children(_rpanel)
	apply_text_scale()   # the saved size (the settings screen previews its draft in its build)
	if game_view != null and game_view.get_parent() == self and not router.in_play():
		_close_game()   # UIR-2: the game left the stack (quit to the main menu)
	var id: String = router.top()
	var over_battle: bool = _results_over_battle()
	var results_model: BattleResultsModel = router.model(MenuRouter.RESULTS) as BattleResultsModel if router.stack.has(MenuRouter.RESULTS) else null
	var in_play_popup: bool = play_popup(id)
	var popup: bool = id == MenuRouter.SETTINGS or in_play_popup
	_popup_max = POPUP_WIDE if in_play_popup and id != MenuRouter.SETTINGS and id != MenuRouter.HUB else POPUP_MAX
	if not popup and _volumes_previewed:
		restore_volumes()   # the popup closed: an unapplied draft volume goes back to the saved one
	var in_results: bool = id == MenuRouter.RESULTS and over_battle
	var minimised: bool = in_results and results_model != null and results_model.minimised
	_popup.visible = popup
	_rpopup.visible = in_results and not minimised
	_results_pill.visible = minimised
	var kept: BattleScreen = active_battle()
	var over_game: bool = popup and game_view != null and game_view.visible
	if kept != null:
		kept.set_keys_blocked((in_results and not minimised) or over_game)   # the arena ignores its shortcuts under a popup
	_block_game(over_game)
	_place_popup(over_game)
	if minimised:
		_results_pill.text = "%s ▴" % (results_model.headline() if results_model != null else "Results")
	_sync_game_visibility()
	if id == MenuRouter.SETTINGS and router.stack.size() > 1 and not in_play_popup:
		# The screen below stays visible behind the dim; it is only a backdrop, so nothing in it takes focus.
		var below: String = router.stack[-2]
		var under: Control = _make_view(below)
		if under != null:
			under.size_flags_vertical = Control.SIZE_EXPAND_FILL
			_screen.add_child(under)
			under.call("build", self, router.model(below))
			for n: Node in under.find_children("*", "Control", true, false):
				(n as Control).focus_mode = Control.FOCUS_NONE
	var view: Control = _make_view(id)
	if view == null:
		_show_problems()
		_sync_menu_pill()
		return   # PLAY: the game scene is all there is
	view.size_flags_vertical = Control.SIZE_EXPAND_FILL
	if popup and in_play_popup and UNFRAMED_POPUP.has(id):
		var scroll := ScrollContainer.new()
		scroll.name = "PopupScroll"
		scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
		scroll.follow_focus = true
		_popup_panel.add_child(scroll)
		view.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		scroll.add_child(view)
	elif popup:
		_popup_panel.add_child(view)
	elif in_results:
		_rpanel.add_child(view)
	else:
		_screen.add_child(view)
	view.call("build", self, router.model())
	if popup:
		_layout_popup()
	elif in_results:
		_layout_panel(_rpopup, _rpanel)
	_show_problems()
	_sync_menu_pill()
	focus_first(view)

# UIR-2: true when `id` shows as a popup over the paused game scene: any menu screen above PLAY while the game is open,
# except the battle results flow (results and the level-up / loadout opened from them keep their own layout).
func play_popup(id: String) -> bool:
	return id != MenuRouter.PLAY and router.in_play() and game_view != null and not router.stack.has(MenuRouter.RESULTS)

# BATTLE-FLOW: the results screen is a popup (not a full screen) while the settled fight's arena is still mounted.
func _results_over_battle() -> bool:
	return router.top() == MenuRouter.RESULTS and active_battle() != null

func active_battle() -> BattleScreen:
	if game_view == null or not game_view.has_method("active_battle"):
		return null
	var found: Variant = game_view.call("active_battle")
	if found is BattleScreen:
		return found
	return null

# While RESULTS is on the stack: the popup shows over the visible game and arena; a menu screen above it (level-up, loadout)
# hides both and shows the menu; without a kept arena the old full-screen results hide the game.
func _sync_game_visibility() -> void:
	if game_view == null or not router.stack.has(MenuRouter.RESULTS):
		return
	var battle: BattleScreen = active_battle()
	var popup: bool = _results_over_battle()
	game_view.visible = popup
	_menu.visible = not popup
	_game_hint.visible = false
	if battle != null:
		battle.set_shown(popup)

func _make_view(id: String) -> Control:
	match id:
		MenuRouter.MAIN:
			return MainScreenView.new()
		MenuRouter.NEW_GAME:
			return NewGameView.new()
		MenuRouter.LOAD, MenuRouter.SAVE:
			return SaveSlotsView.new()
		MenuRouter.SETTINGS:
			return SettingsView.new()
		MenuRouter.HUB:
			return HubView.new()
		MenuRouter.PARTY:
			return CharacterView.new()
		MenuRouter.LEVEL_UP:
			return LevelUpView.new()
		MenuRouter.LOADOUT:
			return LoadoutView.new()
		MenuRouter.GEAR:
			return GearView.new()
		MenuRouter.RESULTS:
			return BattleResultsView.new()
		MenuRouter.JOURNAL:
			return JournalView.new()
	return null

# SETTINGS-POPUP: a dimmed full-screen backdrop that swallows clicks (a left click on it does nothing; a right click anywhere is Back, see _input) and a centred panel.
func _build_popup() -> void:
	_slayer = CanvasLayer.new()   # the popup moves in here only while a game scene shows (see _place_popup); otherwise it is a plain child of this Control
	_slayer.layer = SETTINGS_LAYER
	add_child(_slayer)
	_stheme = MenuStyle.theme()
	var parts: Array[Control] = _popup_parts(self, POPUP_DIM, _stheme)
	_popup = parts[0]
	_popup_panel = parts[1] as PanelContainer
	_popup.resized.connect(_layout_popup)
	# BATTLE-FLOW: the same look for the battle results, in a canvas layer above the arena's, lighter dim, plus the restore pill.
	_rlayer = CanvasLayer.new()
	_rlayer.layer = POPUP_LAYER
	add_child(_rlayer)
	_rtheme = MenuStyle.theme()
	var rparts: Array[Control] = _popup_parts(_rlayer, RESULTS_DIM, _rtheme)
	_rpopup = rparts[0]
	_rpanel = rparts[1] as PanelContainer
	_rpopup.name = "ResultsLayer"
	_rpanel.name = "ResultsPanel"
	_rpopup.resized.connect(_layout_panel.bind(_rpopup, _rpanel))
	_results_pill = Button.new()
	_results_pill.name = "ResultsPill"
	_results_pill.tooltip_text = "Show the battle results again"
	_results_pill.focus_mode = Control.FOCUS_ALL
	_results_pill.set_anchors_preset(Control.PRESET_CENTER_BOTTOM)
	_results_pill.grow_horizontal = Control.GROW_DIRECTION_BOTH
	_results_pill.grow_vertical = Control.GROW_DIRECTION_BEGIN
	_results_pill.offset_bottom = -16
	_results_pill.visible = false
	_results_pill.pressed.connect(_restore_results)
	_results_pill.theme = _rtheme
	_rlayer.add_child(_results_pill)   # a Control straight under a canvas layer anchors to the viewport
	var pill_layer := CanvasLayer.new()
	pill_layer.layer = PILL_LAYER
	add_child(pill_layer)
	_menu_pill = Button.new()
	_menu_pill.name = "MenuPill"
	_menu_pill.text = "☰ Menu"
	_menu_pill.tooltip_text = "Game menu: party, journal, save, load, settings (Esc)"
	_menu_pill.focus_mode = Control.FOCUS_NONE   # keyboard focus stays with the game; Esc is the keyboard route
	_menu_pill.set_anchors_preset(Control.PRESET_TOP_RIGHT)
	_menu_pill.grow_horizontal = Control.GROW_DIRECTION_BEGIN
	_menu_pill.offset_right = -12
	_menu_pill.offset_top = 8
	_menu_pill.visible = false
	_menu_pill.pressed.connect(open_menu_in_game)
	_menu_pill.theme = _stheme
	pill_layer.add_child(_menu_pill)

# [popup root (hidden), panel]: the dimmed click-swallowing backdrop and the opaque bordered panel, under `parent`.
func _popup_parts(parent: Node, dim_color: Color, popup_theme: Theme) -> Array[Control]:
	var root := Control.new()
	root.name = "PopupLayer"
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.theme = popup_theme
	root.visible = false
	parent.add_child(root)
	var dim := ColorRect.new()
	dim.name = "PopupBackdrop"
	dim.color = dim_color
	dim.set_anchors_preset(Control.PRESET_FULL_RECT)
	dim.mouse_filter = Control.MOUSE_FILTER_STOP
	root.add_child(dim)
	var panel := PanelContainer.new()
	panel.name = "SettingsPanel"
	panel.add_theme_stylebox_override("panel", UiTheme.box(Color(UiTheme.COL_PANEL_BG, 1.0), UiTheme.COL_HEADER, 8, 18, 14, 2))   # opaque: the screen below must not show through
	root.add_child(panel)
	return [root, panel] as Array[Control]

func _restore_results() -> void:
	var m: BattleResultsModel = router.model(MenuRouter.RESULTS) as BattleResultsModel
	var battle: BattleScreen = active_battle()
	if battle != null:
		battle.stop_replay()   # restoring the popup ends a replay in progress
	if m != null:
		m.set_minimised(false)

# The panel is its largest size (POPUP_MAX, or POPUP_WIDE for in-play screens), or the viewport minus a margin when that is smaller, and centred.
func _layout_popup() -> void:
	_layout_panel(_popup, _popup_panel, _popup_max)

func _layout_panel(root: Control, panel: PanelContainer, most: Vector2 = POPUP_MAX) -> void:
	if root == null or panel == null or not root.visible:
		return
	var target := Vector2(minf(most.x, root.size.x - 2.0 * POPUP_MARGIN), minf(most.y, root.size.y - 2.0 * POPUP_MARGIN))
	panel.custom_minimum_size = target
	panel.size = target
	panel.position = ((root.size - target) / 2.0).floor()

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
	b.custom_minimum_size = Vector2(W_WIDE, 0)
	b.alignment = HORIZONTAL_ALIGNMENT_CENTER
	b.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	b.pressed.connect(callback)
	return b

# A compact button for rows and toolbars: `width` 0 = the row class (W_ROW), never fit-to-text among peers; a longer label still grows it.
static func small(b: Button, width: int = 0) -> Button:
	b.custom_minimum_size = Vector2(width if width > 0 else W_ROW, 0)
	return b

# A step button (- / + / icon) at the step class width.
static func step(b: Button) -> Button:
	return small(b, W_STEP)

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
	_qtheme = MenuStyle.theme()
	_overlay.theme = _qtheme
	_mlayer = CanvasLayer.new()
	_mlayer.layer = MODAL_LAYER
	add_child(_mlayer)
	var qlayer := CanvasLayer.new()
	qlayer.layer = QUESTION_LAYER
	add_child(qlayer)
	qlayer.add_child(_overlay)
	var v := VBoxContainer.new()
	v.add_theme_constant_override("separation", 14)
	_overlay.add_child(v)
	_overlay_text = text("")
	_overlay_text.custom_minimum_size = Vector2(420, 0)
	v.add_child(_overlay_text)
	_overlay_buttons = HBoxContainer.new()
	v.add_child(_overlay_buttons)

# The parent for full-screen modals (the Gear board): a layer above the popups, so it shows over the in-play popup.
func modal_parent() -> Node:
	return _mlayer

func has_modal() -> bool:
	return _mlayer != null and _mlayer.get_child_count() > 0

func is_overlay_open() -> bool:
	return _overlay != null and _overlay.visible

func _open_overlay(message: String, buttons: Array[Button], focus_index: int = 0) -> void:
	_overlay_text.text = message
	UiTheme.clear_children(_overlay_buttons)
	for b: Button in buttons:
		b.custom_minimum_size = Vector2(120, 0)
		_overlay_buttons.add_child(b)
	_overlay.visible = true
	_overlay.reset_size()
	_overlay.position = (size - _overlay.size) / 2.0
	_overlay_default = buttons[focus_index]
	MainMenuView._focus.call_deferred(buttons[focus_index])

# The button of the open question that takes the focus (No for a destructive one); null when no question is open.
func overlay_default_button() -> Button:
	return _overlay_default if is_overlay_open() else null

func _on_confirm_needed(message: String) -> void:
	# X6: a destructive question (overwrite, delete, quit without saving) opens with No focused
	_open_overlay(message, [button("Yes", answer.bind(true)), button("No", answer.bind(false))] as Array[Button], 1 if router.pending_destructive else 0)

# DISPLAY1 rule 6: the "Keep these display settings?" box with a visible countdown; Revert has focus.
func begin_display_confirm(m: SettingsModel) -> void:
	_confirm = m
	_open_overlay(_confirm_text(), [button("Keep", _end_confirm.bind(true)), button("Revert", _end_confirm.bind(false))] as Array[Button], 1)

func _confirm_text() -> String:
	return "Keep these display settings? Reverting in %d s" % ceili(_confirm.confirm_left)

func is_confirming() -> bool:
	return _confirm != null

# Keep writes the mode and size; Revert (also Esc and the timeout) restores the old ones.
func _end_confirm(keep: bool) -> void:
	var m: SettingsModel = _confirm
	_confirm = null
	_overlay.visible = false
	if m != null:
		if keep:
			show_problems(m.keep_display())
		else:
			m.revert_display()
	render()

func _process(delta: float) -> void:
	if _confirm == null:
		return
	if _confirm.tick(delta):
		_end_confirm(false)
	elif _confirm != null:
		_overlay_text.text = _confirm_text()

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
	if game_view != null and game_view.get_parent() == self:
		_close_game()   # a game loaded from the in-play menu replaces the one on screen
	var packed: PackedScene = load(GAME_SCENE) as PackedScene if ResourceLoader.exists(GAME_SCENE) else null
	var node: Node = packed.instantiate() if packed != null else null
	if node == null or not node.has_method("play_state"):
		if node != null:
			node.free()
		if router.top() != MenuRouter.HUB:
			router.clear_to(MenuRouter.HUB)   # only a broken scene gets here (a missing one already made enter_game open the hub): full-screen menu, no Return to map
		return
	if not router.in_play():
		router.clear_to(MenuRouter.PLAY)   # the game scene is the bottom of the stack (also when a caller opens it directly)
	game_view = node
	if "debug_tools" in game_view:
		game_view.set("debug_tools", false)   # MAPDEV1-U: the scene file is the debug scene; in play no world picker / New game / debug status
	if "results_popup" in game_view:
		game_view.set("results_popup", true)   # a won / lost / fled battle keeps its arena for the results popup
	if game_view.has_signal("fight_settled"):
		game_view.connect("fight_settled", _on_fight_settled)
	add_child(game_view)
	move_child(game_view, _game_hint.get_index())
	_menu.visible = false
	_game_hint.visible = true
	_sync_menu_pill()
	game_view.call("play_state", gs)

# Frees the game scene and shows the menu screens again (UIR-2: quitting to the main menu or loading another game; the
# old "Back to hub" exit is gone, the hub is a popup over the map now).
func _close_game() -> void:
	if game_view == null:
		return
	if game_view.get_parent() == self:
		remove_child(game_view)
	game_view.queue_free()
	game_view = null
	_game_mode = Node.PROCESS_MODE_INHERIT
	_menu.visible = true
	_game_hint.visible = false

# UI2-L rule 8: a settled fight shows the results screen over the hidden map; Continue brings the map back.
func _on_fight_settled(res: Dictionary) -> void:
	if game_view == null or not CurrentGame.has_game():
		return
	game_view.visible = false
	_menu.visible = true
	_game_hint.visible = false
	router.push(MenuRouter.RESULTS, {"award": DictUtil.dict_of(res.get("xp", {})), "advanced": res.get("advanced", []), "outcome": str(res.get("outcome", "won")),
		"rewards": BattleResultsModel.rewards_of(res), "report": DictUtil.dict_of(res.get("report", {}))})
	var battle: BattleScreen = active_battle()
	var m: BattleResultsModel = router.model(MenuRouter.RESULTS) as BattleResultsModel
	if battle != null and m != null:
		m.replay_handler = battle.replay
		if not battle.replay_finished.is_connected(_on_replay_finished):
			battle.replay_finished.connect(_on_replay_finished)
		if not battle.tree_exited.is_connected(_on_battle_gone):
			battle.tree_exited.connect(_on_battle_gone)
		router.changed.emit()   # the popup was built before Replay existed

# The kept arena was freed under the popup (a lost road fight closes the world map): drop Replay and re-render.
func _on_battle_gone() -> void:
	if router == null or not is_inside_tree():
		return
	var m: BattleResultsModel = router.model(MenuRouter.RESULTS) as BattleResultsModel
	if m != null:
		m.replay_handler = Callable()
		if router.top() == MenuRouter.RESULTS:
			router.changed.emit()

func _on_replay_finished() -> void:
	_restore_results()

func _on_results_done() -> void:
	if game_view == null:
		return
	if game_view.has_method("close_settled_battle"):
		game_view.call("close_settled_battle")
	game_view.visible = true
	_menu.visible = false
	_game_hint.visible = true
	_sync_menu_pill.call_deferred()   # RESULTS leaves the stack right after results_done

# CONTROLS-HELP: true while the game scene (map, dialogue, shop, fight) is what the player sees.
func in_game() -> bool:
	return game_view != null and game_view.visible

# Esc / the Menu button during play: the game menu (HUB) as a popup over the paused game; only from the bare game scene
# (not over a results popup, another popup or a question).
func open_menu_in_game() -> void:
	if not in_game() or router.top() != MenuRouter.PLAY or is_overlay_open():
		return
	router.push(MenuRouter.HUB)

# The game under the settings popup stops: no timers, no key or mouse handling (combat also gets keys_blocked, set in render()).
func _block_game(blocked: bool) -> void:
	if game_view != null and is_instance_valid(game_view):
		if blocked:
			if game_view.process_mode != Node.PROCESS_MODE_DISABLED:
				_game_mode = game_view.process_mode
			game_view.process_mode = Node.PROCESS_MODE_DISABLED
		elif game_view.process_mode == Node.PROCESS_MODE_DISABLED:
			game_view.process_mode = _game_mode   # what it was before the popup, not a blanket INHERIT

# Over a game scene the popup sits in a canvas layer above the arena's own layers; over a menu screen it stays a plain child (layout and tests stay in this Control's rect).
func _place_popup(in_layer: bool) -> void:
	var target: Node = _slayer if in_layer else self
	if _popup != null and _popup.get_parent() != target:
		_popup.reparent(target, false)

func _sync_menu_pill() -> void:
	if _menu_pill != null:
		_menu_pill.visible = in_game() and router != null and router.top() == MenuRouter.PLAY

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

# Right click = Back, same path as Esc. It is taken in _input, before the GUI: panels, buttons and scroll areas use
# MOUSE_FILTER_STOP and would swallow the click before _unhandled_input (the question and countdown boxes treat it as No / Revert).
func _input(event: InputEvent) -> void:
	var mouse: InputEventMouseButton = event as InputEventMouseButton
	if mouse == null or not mouse.pressed or mouse.button_index != MOUSE_BUTTON_RIGHT or not is_visible_in_tree():
		return
	if in_game() and router.top() == MenuRouter.PLAY:
		return   # the map and the fights own the right button (not under a popup: there it is Back)
	cancel_pressed()
	get_viewport().set_input_as_handled()

func _unhandled_input(event: InputEvent) -> void:
	if not event.is_action_pressed("ui_cancel"):
		return
	cancel_pressed()
	accept_event()

# Esc. On the results screen it is the X (formerly Continue) button (autosave, results_done), never a plain pop (REVIEW-F2).
func cancel_pressed() -> void:
	if _confirm != null:
		_end_confirm(false)
	elif is_overlay_open():
		answer(false)
	elif router.top() == MenuRouter.RESULTS and router.model() is BattleResultsModel:
		act((router.model() as BattleResultsModel).continue_on)   # the popup's X: close = autosave, results_done
	elif router.top() == MenuRouter.SETTINGS:
		router.pop()   # asks first when the draft changed; also the in-game popup (right click and the cross come here too)
	elif in_game() and router.top() == MenuRouter.PLAY:
		open_menu_in_game()   # nothing in the scene used Esc (a panel or the arena takes it first), so it opens the game menu (UIR-2)
	else:
		router.pop()
