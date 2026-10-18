class_name ControlsList
extends RefCounted

# ==============================================================================
# Tactical RPG - Controls list (CONTROLS-HELP): the data behind the Settings "Controls" tab.
# Two sources, one list: (1) InputMap actions (project input actions, read live, so a changed binding shows up), (2) the keys the
# scenes handle themselves in _input / _unhandled_input (HARD_CODED). Each hard-coded row names the script that handles it and the
# tokens that must appear in that script's source; ControlsListTestSuite scans the sources, so the help cannot drift from the code.
# Rebinding is not supported; this is a read-only help.
# ==============================================================================

const GROUP_MENUS: String = "Menus"
const GROUP_MAPS: String = "Maps"
const GROUP_COMBAT: String = "Combat"
const GROUP_GENERAL: String = "General"
const GROUP_TESTING: String = "Testing"   # dev-only keys (arena edit mode, Scene Runner)
const GROUPS: Array[String] = [GROUP_MENUS, GROUP_MAPS, GROUP_COMBAT, GROUP_GENERAL, GROUP_TESTING]

# InputMap actions shown (when the project defines them): [action, plain-words name, group].
const INPUT_ACTIONS: Array[Array] = [
	["ui_accept", "Confirm / press the focused button", GROUP_MENUS],
	["ui_cancel", "Back / close the top panel", GROUP_MENUS],
	["ui_focus_next", "Focus the next control", GROUP_MENUS],
	["ui_focus_prev", "Focus the previous control", GROUP_MENUS],
	["ui_up", "Move focus / slider up", GROUP_MENUS],
	["ui_down", "Move focus / slider down", GROUP_MENUS],
	["ui_left", "Move focus / slider left", GROUP_MENUS],
	["ui_right", "Move focus / slider right", GROUP_MENUS],
]

# Hard-coded keys: {group, action (plain words), keys (shown text), src (script that handles it), tokens (strings that must appear in src)}.
const HARD_CODED: Array[Dictionary] = [
	{"group": GROUP_MENUS, "action": "Back / close (mouse)", "keys": "Right click", "src": "res://scripts/ui/menus/main_menu_view.gd", "tokens": ["MOUSE_BUTTON_RIGHT"]},
	{"group": GROUP_MAPS, "action": "Move the party", "keys": "Arrow keys, W A S D (hold to keep walking)", "src": "res://scripts/ui/overworld/region_map_view.gd", "tokens": ["KEY_UP", "KEY_DOWN", "KEY_LEFT", "KEY_RIGHT", "KEY_W", "KEY_A", "KEY_S", "KEY_D"]},
	{"group": GROUP_MAPS, "action": "Interact (talk, open, read)", "keys": "E", "src": "res://scripts/ui/overworld/region_map_view.gd", "tokens": ["KEY_E"]},
	{"group": GROUP_MAPS, "action": "Interact with a marker next to the party", "keys": "Left click", "src": "res://scripts/ui/overworld/region_map_view.gd", "tokens": ["MOUSE_BUTTON_LEFT"]},
	{"group": GROUP_MAPS, "action": "Open Settings (from any scene)", "keys": "Esc, the Menu button", "src": "res://scripts/ui/menus/main_menu_view.gd", "tokens": ["ui_cancel"]},
	{"group": GROUP_COMBAT, "action": "Close the log / timeline panel", "keys": "Esc", "src": "res://scripts/ui/combat/combat_dock_host.gd", "tokens": ["KEY_ESCAPE"]},
	{"group": GROUP_COMBAT, "action": "Combat log", "keys": "L", "src": "res://scripts/ui/combat/combat_dock_host.gd", "tokens": ["KEY_L"]},
	{"group": GROUP_COMBAT, "action": "Timeline", "keys": "F, Tab", "src": "res://scripts/ui/combat/combat_dock_host.gd", "tokens": ["KEY_F", "KEY_TAB"]},
	{"group": GROUP_COMBAT, "action": "Undo the last planned step", "keys": "Backspace", "src": "res://scripts/ui/combat/combat_dock_host.gd", "tokens": ["KEY_BACKSPACE"]},
	{"group": GROUP_COMBAT, "action": "Confirm the plan", "keys": "Enter", "src": "res://scripts/ui/combat/combat_dock_host.gd", "tokens": ["KEY_ENTER", "KEY_KP_ENTER"]},
	{"group": GROUP_COMBAT, "action": "Previous block", "keys": "Q, Left", "src": "res://scripts/ui/combat/combat_dock_host.gd", "tokens": ["KEY_Q", "KEY_LEFT"]},
	{"group": GROUP_COMBAT, "action": "Next block", "keys": "E, Right", "src": "res://scripts/ui/combat/combat_dock_host.gd", "tokens": ["KEY_E", "KEY_RIGHT"]},
	{"group": GROUP_COMBAT, "action": "Show / hide enemy tells (chips and arrows)", "keys": "I", "src": "res://scripts/ui/combat/combat_dock_host.gd", "tokens": ["KEY_I"]},
	{"group": GROUP_COMBAT, "action": "Auto-play on / off", "keys": "Space", "src": "res://scripts/ui/combat/combat_dock_host.gd", "tokens": ["KEY_SPACE"]},
	{"group": GROUP_COMBAT, "action": "Previous / next log entry", "keys": "[  ]", "src": "res://scripts/ui/combat/combat_dock_host.gd", "tokens": ["KEY_BRACKETLEFT", "KEY_BRACKETRIGHT"]},
	{"group": GROUP_TESTING, "action": "Slot edit mode (arena)", "keys": "F9", "src": "res://scripts/ui/combat/combat_dock_host.gd", "tokens": ["KEY_F9"]},
	{"group": GROUP_COMBAT, "action": "Pick action option 1 to 9", "keys": "1 to 9", "src": "res://scripts/ui/plan_action_popup.gd", "tokens": ["KEY_1", "KEY_9"]},
	{"group": GROUP_COMBAT, "action": "Take the default action", "keys": "Enter, Space", "src": "res://scripts/ui/plan_action_popup.gd", "tokens": ["KEY_ENTER", "KEY_SPACE"]},
	{"group": GROUP_COMBAT, "action": "Close the action popup", "keys": "Esc", "src": "res://scripts/ui/plan_action_popup.gd", "tokens": ["KEY_ESCAPE"]},
	{"group": GROUP_COMBAT, "action": "Select a tile or unit", "keys": "Left click", "src": "res://scripts/ui/combat_grid_view.gd", "tokens": ["MOUSE_BUTTON_LEFT"]},
	{"group": GROUP_COMBAT, "action": "Cancel the current pick", "keys": "Right click", "src": "res://scripts/ui/combat_grid_view.gd", "tokens": ["MOUSE_BUTTON_RIGHT"]},
	{"group": GROUP_COMBAT, "action": "Skip the battle intro", "keys": "Any click, Enter, Esc", "src": "res://scripts/ui/overworld/battle_screen.gd", "tokens": ["ui_accept", "ui_cancel"]},
	{"group": GROUP_GENERAL, "action": "Playtest note (screenshot + text)", "keys": "F8", "src": "res://scripts/ui/playtest/playtest_hub.gd", "tokens": ["KEY_F8"]},
	{"group": GROUP_GENERAL, "action": "Note box: cancel", "keys": "Esc", "src": "res://scripts/ui/playtest/note_box.gd", "tokens": ["KEY_ESCAPE"]},
	{"group": GROUP_GENERAL, "action": "Note box: save", "keys": "Ctrl+Enter", "src": "res://scripts/ui/playtest/note_box.gd", "tokens": ["KEY_ENTER", "ctrl_pressed"]},
	{"group": GROUP_TESTING, "action": "Scene Runner: reload and restart", "keys": "F5", "src": "res://scripts/ui/runner/scene_runner.gd", "tokens": ["KEY_F5"]},
	{"group": GROUP_TESTING, "action": "Scene Runner: restart the scene", "keys": "F6", "src": "res://scripts/ui/runner/scene_runner.gd", "tokens": ["KEY_F6"]},
	{"group": GROUP_TESTING, "action": "Scene Runner: back to the menu", "keys": "F10", "src": "res://scripts/ui/runner/scene_runner.gd", "tokens": ["KEY_F10"]},
	{"group": GROUP_TESTING, "action": "Scene Runner: fullscreen / windowed", "keys": "F11", "src": "res://scripts/ui/runner/scene_runner.gd", "tokens": ["KEY_F11"]},
]

# The keys and mouse buttons bound to an InputMap action, as text ("Escape, Enter"); joypad events are left out.
static func action_keys(action: String) -> String:
	if not InputMap.has_action(action):
		return ""
	var names: Array[String] = []
	for ev: InputEvent in InputMap.action_get_events(action):
		var text: String = ""
		if ev is InputEventKey:
			var key: InputEventKey = ev
			var code: Key = key.get_keycode_with_modifiers() if key.keycode != KEY_NONE else key.get_physical_keycode_with_modifiers()
			text = OS.get_keycode_string(code)
		elif ev is InputEventMouseButton:
			text = "Mouse button %d" % (ev as InputEventMouseButton).button_index
		if not text.is_empty() and not names.has(text):
			names.append(text)
	return ", ".join(PackedStringArray(names))

# Rows of one group: [{id, action, keys}] (InputMap rows first, then the hard-coded ones). `id` is "action:<name>" or "key:<n>".
static func rows(group: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for a: Array in INPUT_ACTIONS:
		if str(a[2]) == group and InputMap.has_action(str(a[0])):
			out.append({"id": "action_" + str(a[0]), "action": str(a[1]), "keys": action_keys(str(a[0]))})
	for i: int in HARD_CODED.size():
		var h: Dictionary = HARD_CODED[i]
		if str(h["group"]) == group:
			out.append({"id": "key_%d" % i, "action": str(h["action"]), "keys": str(h["keys"])})
	return out
