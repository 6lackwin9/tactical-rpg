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
signal results_done()                              # the battle results screen closed (the host resumes the game scene)

const MAIN: String = "main"
const NEW_GAME: String = "new_game"
const LOAD: String = "load"
const SAVE: String = "save"
const SETTINGS: String = "settings"
const HUB: String = "hub"             # UIR-2 (Q1 B): the game menu; during play a popup over the paused map
const PLAY: String = "play"           # UIR-2: the game scene itself (no model, no view); the bottom of the stack during play
const PARTY: String = "party"          # UI2-L: roster, stats, passives
const LEVEL_UP: String = "level_up"    # UI2-L: args {member}
const LOADOUT: String = "loadout"      # UI2-L: args {member}
const GEAR: String = "gear"            # UI2-L: args {member}
const RESULTS: String = "results"      # UI2-L: args {award, advanced, outcome (BATTLE1-L), rewards (BATTLE-FLOW, non-XP), report (BREPORT1-L, optional)}
const JOURNAL: String = "journal"      # UI3-L

var stack: Array[String] = []
var models: Dictionary = {}        # screen id on the stack -> its model
var play_scene: bool = true   # the host has a game scene to show (false: enter_game opens the full-screen hub instead of PLAY)
var pending_message: String = ""
var pending_destructive: bool = false   # the open question is destructive: No gets the focus
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

# The model for screen `id` (a fresh one each time the screen is opened); `args` carries a member id or a battle award.
func make_model(id: String, args: Dictionary = {}) -> RefCounted:
	var member: String = str(args.get("member", ""))
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
		PARTY:
			return PartyModel.new(self) if CurrentGame.has_game() else null
		LEVEL_UP:
			return LevelUpModel.new(member) if CurrentGame.has_game() else null
		LOADOUT:
			return LoadoutModel.new(member) if CurrentGame.has_game() else null
		GEAR:
			return GearModel.new(member) if CurrentGame.has_game() else null
		JOURNAL:
			return JournalModel.new() if CurrentGame.has_game() else null
		RESULTS:
			return BattleResultsModel.new(self, DictUtil.dict_of(args.get("award", {})), JsonUtil.strings(args.get("advanced", [])), str(args.get("outcome", "won")), DictUtil.dict_of(args.get("rewards", {})), DictUtil.dict_of(args.get("report", {}))) if CurrentGame.has_game() else null
	return null

func push(id: String, args: Dictionary = {}) -> void:
	if stack.has(id):
		models.erase(id)
		stack.erase(id)
	stack.append(id)
	models[id] = make_model(id, args)
	changed.emit()

# Leaves the top screen; asks first when it has a draft. The bottom screen stays.
func pop() -> void:
	if stack.size() <= 1 or top() == RESULTS:
		return   # the results screen closes only through BattleResultsModel.continue_on (autosave, results_done)
	if has_draft():
		ask("Discard your changes?", _pop_now)
		return
	_pop_now()

func _pop_now() -> void:
	if stack.size() <= 1:
		return
	models.erase(stack.pop_back())
	var back: RefCounted = model()
	if back != null and back.has_method("refresh"):
		back.call("refresh")   # the screen below re-reads what the popped screen may have changed
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
# `destructive` (overwrite, delete, quit without saving) makes the host focus No instead of Yes (UIR-1, X6).
func ask(message: String, action: Callable, destructive: bool = false) -> void:
	pending_message = message
	pending_destructive = destructive
	_pending = action
	confirm_needed.emit(message)

func is_asking() -> bool:
	return _pending.is_valid()

func confirm() -> void:
	var action: Callable = _pending
	_pending = Callable()
	pending_message = ""
	pending_destructive = false
	if action.is_valid():
		action.call()

func decline() -> void:
	_pending = Callable()
	pending_message = ""
	pending_destructive = false

# True while a game is being played (the game scene is the bottom of the stack).
func in_play() -> bool:
	return stack.has(PLAY)

# Starts playing `gs`: shows `warnings` once, puts the game scene (PLAY) at the bottom of the stack and tells the host
# (UIR-2: the hub is no longer a screen of its own; Esc / the Menu button push it as a popup over the paused map).
func enter_game(gs: GameState, warnings: Array[String] = []) -> void:
	if not warnings.is_empty():
		notice.emit(warnings)
	clear_to(PLAY if play_scene else HUB)   # decided before clearing: one stack change, no PLAY then HUB
	game_started.emit(gs)
