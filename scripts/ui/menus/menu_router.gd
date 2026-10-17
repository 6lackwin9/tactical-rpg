class_name MenuRouter
extends RefCounted

# ==============================================================================
# Tactical RPG - Menu router (UI1-L, S13): the screen stack shared by the main menu and the in-game hub.
# Each screen id has a RefCounted model (make_model); views render top() and forward input. Leaving a screen whose
# model has a draft, quitting with unsaved progress and overwriting or deleting a save all go through ask(): the
# router emits confirm_needed and runs the action only on confirm(). Headless: no Node is touched.
# ==============================================================================

signal changed()                                   # the stack changed (views re-render top())
signal confirm_needed(message: String)             # a yes / no question; answer with confirm() or decline()
signal notice(lines: Array[String])                # messages to show once (load warnings)
signal game_started(gs: GameState)                 # a new or loaded game is ready (the host opens the game scene)
signal quit_requested()                            # leave the application

const MAIN: String = "main"
const NEW_GAME: String = "new_game"
const LOAD: String = "load"
const SAVE: String = "save"
const SETTINGS: String = "settings"
const HUB: String = "hub"
const PARTY: String = "party"      # UI2-L
const JOURNAL: String = "journal"  # UI3-L

var stack: Array[String] = []
var models: Dictionary = {}        # screen id on the stack -> its model
var pending_message: String = ""
var _pending: Callable = Callable()

func _init(start: String = MAIN) -> void:
	if not start.is_empty():
		push(start)

func top() -> String:
	return stack[-1] if not stack.is_empty() else ""

func model(id: String = "") -> RefCounted:
	var key: String = top() if id.is_empty() else id
	return models.get(key, null) as RefCounted

# True when the screen's model holds an uncommitted draft.
func has_draft(id: String = "") -> bool:
	var m: RefCounted = model(id)
	return m != null and m.has_method("has_draft") and DictUtil.to_bool(m.call("has_draft"))

# The model for screen `id` (a fresh one each time the screen is opened).
func make_model(id: String) -> RefCounted:
	match id:
		MAIN:
			return MainMenuModel.new(self)
		NEW_GAME:
			return NewGameModel.new(self)
		LOAD:
			return SaveSlotsModel.new(self, false)
		SAVE:
			return SaveSlotsModel.new(self, true)
		SETTINGS:
			return SettingsModel.new()
		HUB:
			return HubModel.new(self)
	return null

func push(id: String) -> void:
	if stack.has(id):
		models.erase(id)
		stack.erase(id)
	stack.append(id)
	models[id] = make_model(id)
	changed.emit()

# Leaves the top screen; asks first when it has a draft. The bottom screen stays.
func pop() -> void:
	if stack.size() <= 1:
		return
	if has_draft():
		ask("Discard your changes?", _pop_now)
		return
	_pop_now()

func _pop_now() -> void:
	if stack.size() <= 1:
		return
	models.erase(stack.pop_back())
	changed.emit()

func replace(id: String) -> void:
	if not stack.is_empty():
		models.erase(stack.pop_back())
	push(id)

# Drops every screen and opens `id` alone (entering or leaving the game).
func clear_to(id: String) -> void:
	stack.clear()
	models.clear()
	push(id)

# Asks a yes / no question; `action` runs on confirm().
func ask(message: String, action: Callable) -> void:
	pending_message = message
	_pending = action
	confirm_needed.emit(message)

func is_asking() -> bool:
	return _pending.is_valid()

func confirm() -> void:
	var action: Callable = _pending
	_pending = Callable()
	pending_message = ""
	if action.is_valid():
		action.call()

func decline() -> void:
	_pending = Callable()
	pending_message = ""

# Starts playing `gs`: shows `warnings` once, opens the hub and tells the host.
func enter_game(gs: GameState, warnings: Array[String] = []) -> void:
	if not warnings.is_empty():
		notice.emit(warnings)
	clear_to(HUB)
	game_started.emit(gs)
