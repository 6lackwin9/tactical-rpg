class_name PlaytestUiTrail
extends Node

# ==============================================================================
# Tactical RPG - Playtest UI Trail (PLAYTEST1 section F, the node side)
# Feeds the hub's PlaytestRecorder with UI events from every scene:
#   button   BaseButton.pressed (connected via SceneTree.node_added)
#   panel    visibility of CombatPanel subclasses; modal: Window nodes (popups, dialogs), the project's
#            modal Controls (open_modal / close_modal) and the planning popup. Nodes visible when they
#            enter the tree log an `open` then (the scene's own panels, at scene start).
#   click    every mouse button press (wheel skipped) with the control under the mouse, the grid cell
#            and whether a GUI control consumed it (seen again in _unhandled_input = not handled)
#   key      key presses, skipped while a LineEdit / TextEdit has focus
#   hover    polled every 100 ms, logged on leave when the dwell was >= 700 ms
#   leaked   panels / windows of the previous scene still in the tree one frame after a scene change
# Hooks exist only while a recording is open: the hub calls start() / stop(), which connect and disconnect
# SceneTree.node_added, every per-node signal, the input callbacks and the hover timer (hooked() = false
# leaves zero hooks: release builds and recording off).
# ==============================================================================

const HOVER_POLL: float = 0.1
const BUTTON_META: StringName = &"playtest_trail"
const HOVER_MIN_MS: int = 700
const BUTTON_NAMES: Dictionary = {MOUSE_BUTTON_LEFT: "left", MOUSE_BUTTON_RIGHT: "right", MOUSE_BUTTON_MIDDLE: "middle",
	MOUSE_BUTTON_WHEEL_UP: "wheel_up", MOUSE_BUTTON_WHEEL_DOWN: "wheel_down", MOUSE_BUTTON_WHEEL_LEFT: "wheel_left", MOUSE_BUTTON_WHEEL_RIGHT: "wheel_right"}

var hub: PlaytestHub = null
var scene_root: Node = null        # the running scene (control paths are relative to it)

var _tracked: Dictionary = {}      # instance id -> {ref: WeakRef, scene: int, type: String, open: bool}
var _grids: Array[WeakRef] = []    # CombatGridView nodes (click cells)
var _scene_serial: int = 0
var _hover_id: int = 0             # instance id of the hovered control (0 = none)
var _hover_desc: Dictionary = {}
var _hover_since: int = 0
var _pending_click: InputEventMouseButton = null
var _pending_fields: Dictionary = {}
var _timer: Timer = null
var _buttons: Dictionary = {}      # instance id -> WeakRef of buttons whose pressed is connected

func _init() -> void:
	name = "PlaytestUiTrail"
	process_mode = Node.PROCESS_MODE_ALWAYS
	_timer = Timer.new()
	_timer.wait_time = HOVER_POLL
	add_child(_timer)

func _ready() -> void:
	if not hooked():   # the engine turns input processing on for scripts with _input: off until start()
		set_process_input(false)
		set_process_unhandled_input(false)

func recorder() -> PlaytestRecorder:
	return hub.recorder if hub != null else null

# Connects the hooks (a recording opened) and follows the nodes already under the running scene.
func start() -> void:
	if hooked():
		return
	get_tree().node_added.connect(_on_node_added)
	_timer.timeout.connect(_poll)
	_timer.start()
	set_process_input(true)
	set_process_unhandled_input(true)
	if scene_root != null and is_instance_valid(scene_root) and scene_root.is_inside_tree():
		_hook_tree(scene_root)

func _hook_tree(node: Node) -> void:
	_on_node_added(node)
	for child: Node in node.get_children():
		_hook_tree(child)

# Disconnects every hook (the recording closed).
func stop() -> void:
	if not hooked():
		return
	get_tree().node_added.disconnect(_on_node_added)
	_timer.timeout.disconnect(_poll)
	_timer.stop()
	set_process_input(false)
	set_process_unhandled_input(false)
	for key: Variant in _tracked:
		_unhook(_node_of(DictUtil.dict_of(_tracked[key])), DictUtil.to_int(key))
	for key: Variant in _buttons:
		var raw: Variant = _ref_target(_buttons[key])
		if raw is BaseButton:
			var btn: BaseButton = raw
			btn.remove_meta(BUTTON_META)
			for c: Dictionary in btn.pressed.get_connections():
				var fn: Callable = c["callable"]
				if fn.get_object() == self:
					btn.pressed.disconnect(fn)
	_tracked.clear()
	_buttons.clear()
	_grids.clear()
	_pending_click = null
	_pending_fields = {}
	_hover_id = 0
	_hover_desc = {}

func _unhook(node: Node, key: int) -> void:
	if node == null:
		return
	var vis: Callable = _on_visibility_changed.bind(key)
	if node is CanvasItem:
		var item: CanvasItem = node
		if item.visibility_changed.is_connected(vis):
			item.visibility_changed.disconnect(vis)
	if node is Window:
		var w: Window = node
		if w.visibility_changed.is_connected(vis):
			w.visibility_changed.disconnect(vis)
		if w.window_input.is_connected(_on_window_input):
			w.window_input.disconnect(_on_window_input)

func hooked() -> bool:
	return is_inside_tree() and get_tree().node_added.is_connected(_on_node_added)

# Hooks currently connected (tests: 0 while not recording).
func hook_count() -> int:
	var n: int = 1 if hooked() else 0
	n += _tracked.size() + _buttons.size()
	if _timer.timeout.is_connected(_poll):
		n += 1
	return n

# ------------------------------------------------------------------------------
# Scene changes and leaks
# ------------------------------------------------------------------------------

# Logs the scene event now; its `leaked` count is filled one frame later.
func scene_changed(path: String, root: Node) -> void:
	var rec: PlaytestRecorder = recorder()
	_scene_serial += 1
	scene_root = root
	_end_hover()
	_prune()
	if rec == null:
		return
	rec.add_scene(path, 0)
	var trail: Array = DictUtil.array_of(rec.data.get("trail", []))
	var event: Dictionary = DictUtil.dict_of(trail.back()) if not trail.is_empty() else {}
	var previous: int = _scene_serial - 1
	await get_tree().process_frame
	var leaked: int = leaked_count(previous)
	if leaked > 0 and str(event.get("type", "")) == "scene":
		event["leaked"] = leaked
	_prune()

# Tracked panels / windows of scene `serial` still inside the tree.
func leaked_count(serial: int) -> int:
	var n: int = 0
	for key: Variant in _tracked:
		var entry: Dictionary = DictUtil.dict_of(_tracked[key])
		var node: Node = _node_of(entry)
		if node != null and node.is_inside_tree() and DictUtil.to_int(entry["scene"]) == serial and not _is_own(node):
			n += 1
	return n

func _is_own(node: Node) -> bool:
	return hub != null and (hub.is_ancestor_of(node) or node == hub)

func _prune() -> void:
	for key: Variant in _tracked.keys():
		if _node_of(DictUtil.dict_of(_tracked[key])) == null:
			_tracked.erase(key)
	for key: Variant in _buttons.keys():
		if _ref_target(_buttons[key]) == null:
			_buttons.erase(key)
	var live: Array[WeakRef] = []
	for ref: WeakRef in _grids:
		if ref.get_ref() != null:
			live.append(ref)
	_grids = live

static func _ref_target(v: Variant) -> Variant:
	if v is WeakRef:
		var ref: WeakRef = v
		return ref.get_ref()
	return null

static func _node_of(entry: Dictionary) -> Node:
	var ref: WeakRef = entry.get("ref", null)
	if ref == null:
		return null
	var raw: Variant = ref.get_ref()
	if raw is Node:
		var node: Node = raw
		return node
	return null

# ------------------------------------------------------------------------------
# Node tracking
# ------------------------------------------------------------------------------

func _on_node_added(node: Node) -> void:
	if _is_own(node):
		return
	if node is CombatGridView:
		_grids.append(weakref(node))
	if node is BaseButton and DictUtil.to_int(node.get_meta(BUTTON_META, 0)) != get_instance_id():
		var btn: BaseButton = node
		btn.set_meta(BUTTON_META, get_instance_id())   # connected once per trail, even when the button is reparented
		btn.pressed.connect(_on_button_pressed.bind(btn))
		_buttons[btn.get_instance_id()] = weakref(btn)
	var type: String = event_type_of(node)
	if type.is_empty():
		return
	var key: int = node.get_instance_id()
	if _tracked.has(key):
		return   # reparented (undocked panels): same node, no new open
	var open: bool = _visible(node)
	_tracked[key] = {"ref": weakref(node), "scene": _scene_serial, "type": type, "open": open}
	if node is CanvasItem:
		var item: CanvasItem = node
		item.visibility_changed.connect(_on_visibility_changed.bind(key))
	elif node is Window:
		var win: Window = node
		win.visibility_changed.connect(_on_visibility_changed.bind(key))
	if node is Window:
		var w: Window = node
		w.window_input.connect(_on_window_input)
	if open:
		_log_open(node, type, true)

# "panel", "modal" or "" for nodes the trail does not follow.
static func event_type_of(node: Node) -> String:
	if node is CombatPanel:
		return "panel"
	if node is Window or node is PlanActionPopup:
		return "modal"
	if node is Control and node.has_method("open_modal") and node.has_method("close_modal"):
		return "modal"
	return ""

static func _visible(node: Node) -> bool:
	if node is CanvasItem:
		var item: CanvasItem = node
		return item.is_visible_in_tree()
	if node is Window:
		var w: Window = node
		return w.visible
	return false

func _on_visibility_changed(key: int) -> void:
	var entry: Dictionary = DictUtil.dict_of(_tracked.get(key, {}))
	var node: Node = _node_of(entry)
	if node == null or not node.is_inside_tree():
		return
	var open: bool = _visible(node)
	if open == DictUtil.to_bool(entry.get("open", false)):
		return
	entry["open"] = open
	_log_open(node, str(entry["type"]), open)

func _log_open(node: Node, type: String, open: bool) -> void:
	var rec: PlaytestRecorder = recorder()
	if rec != null:
		rec.add_trail(type, {"ctl": describe(node), "open": open})

func _on_button_pressed(btn: BaseButton) -> void:
	var rec: PlaytestRecorder = recorder()
	if rec != null and is_instance_valid(btn):
		rec.add_trail("button", {"ctl": describe(btn)})

# {path (last segments under the running scene), cls, text, tip} for any node (null-safe: {}).
func describe(node: Node) -> Dictionary:
	if node == null or not is_instance_valid(node):
		return {}
	var path_text: String = str(node.get_path()) if node.is_inside_tree() else str(node.name)   # a button freed or detached by its own press has no path
	if scene_root != null and is_instance_valid(scene_root) and scene_root.is_ancestor_of(node):
		path_text = str(scene_root.get_path_to(node))
	var text: String = ""
	var tip: String = ""
	if node is Button:
		var b: Button = node
		text = b.text
	elif node is Label:
		var l: Label = node
		text = l.text
	if node is Control:
		var c: Control = node
		tip = c.tooltip_text
	var cls: String = node.get_class()
	var script: Script = node.get_script()
	if script != null and not script.get_global_name().is_empty():
		cls = script.get_global_name()
	return PlaytestRecorder.describe_control(path_text, cls, text, tip)

# ------------------------------------------------------------------------------
# Input: clicks, keys (root viewport and embedded windows), hover polling
# ------------------------------------------------------------------------------

func _input(event: InputEvent) -> void:
	_record_input(event, true)

func _unhandled_input(event: InputEvent) -> void:
	if event == _pending_click:
		_pending_fields["handled"] = false

func _on_window_input(event: InputEvent) -> void:
	_record_input(event, false)   # popups consume their own input

func _record_input(event: InputEvent, root_viewport: bool) -> void:
	var rec: PlaytestRecorder = recorder()
	if rec == null:
		return
	if event is InputEventMouseButton:
		var mb: InputEventMouseButton = event
		if not mb.pressed:
			return
		var button: String = str(BUTTON_NAMES.get(mb.button_index, "button_%d" % mb.button_index))
		if PlaytestRecorder.WHEEL_BUTTONS.has(button):
			return
		var hovered: Control = get_viewport().gui_get_hovered_control() if root_viewport else null
		var ctl: Variant = null
		if hovered != null:
			ctl = describe(hovered)
		var cell: Variant = cell_at(mb.global_position, hovered)
		if not root_viewport:
			rec.add_click(button, mb.global_position, ctl, cell, true)
			return
		_flush_click()
		_pending_click = mb
		_pending_fields = {"button": button, "pos": mb.global_position, "ctl": ctl, "cell": cell, "handled": true}
		_flush_click.call_deferred()
	elif event is InputEventKey:
		var key: InputEventKey = event
		if not key.pressed or key.echo:
			return
		if key.keycode == KEY_F8:
			if not root_viewport and hub != null and not hub.is_note_open():
				hub.open_note()   # F8 over a popup (the hub's own _input covers the main viewport)
			return
		rec.add_key(OS.get_keycode_string(key.keycode), mods_text(key), text_focus())

func _flush_click() -> void:
	var rec: PlaytestRecorder = recorder()
	if _pending_click == null or rec == null:
		_pending_click = null
		return
	var pos: Vector2 = _pending_fields["pos"]
	rec.add_click(str(_pending_fields["button"]), pos, _pending_fields["ctl"], _pending_fields["cell"], DictUtil.to_bool(_pending_fields["handled"]))
	_pending_click = null
	_pending_fields = {}

static func mods_text(key: InputEventKey) -> String:
	var out: String = ""
	if key.ctrl_pressed:
		out += "ctrl+"
	if key.shift_pressed:
		out += "shift+"
	if key.alt_pressed:
		out += "alt+"
	if key.meta_pressed:
		out += "meta+"
	return out

# True while a LineEdit / TextEdit has keyboard focus (typed text never lands in the trail).
func text_focus() -> bool:
	var focus: Control = get_viewport().gui_get_focus_owner()
	return focus is LineEdit or focus is TextEdit

# The combat grid cell under `pos` (Vector2i) when the hovered control belongs to a grid view, else null.
func cell_at(pos: Vector2, hovered: Control) -> Variant:
	for ref: WeakRef in _grids:
		var raw: Variant = ref.get_ref()
		if not (raw is CombatGridView):
			continue
		var view: CombatGridView = raw
		if hovered != null and not view.is_ancestor_of(hovered) and hovered != view:
			continue
		for coord: Variant in view.grid_tile_nodes:
			var tile_raw: Variant = view.grid_tile_nodes[coord]
			if tile_raw is Control and coord is Vector2i:
				var tile: Control = tile_raw
				if tile.get_global_rect().has_point(pos):
					var cell: Vector2i = coord
					return cell
	return null

func _poll() -> void:
	var rec: PlaytestRecorder = recorder()
	if rec == null:
		return
	rec.drain_errors()
	var hovered: Control = get_viewport().gui_get_hovered_control()
	var hovered_id: int = hovered.get_instance_id() if hovered != null else 0
	if hovered_id == _hover_id:
		return
	_end_hover()
	_hover_id = hovered_id
	_hover_desc = describe(hovered) if hovered != null else {}
	_hover_since = Time.get_ticks_msec()

func _end_hover() -> void:
	var rec: PlaytestRecorder = recorder()
	if rec != null and not _hover_desc.is_empty():
		var dwell: int = Time.get_ticks_msec() - _hover_since
		if dwell >= HOVER_MIN_MS:
			rec.add_trail("hover", {"ctl": _hover_desc, "ms": dwell})
	_hover_id = 0
	_hover_desc = {}

func hovered_description() -> Dictionary:
	var hovered: Control = get_viewport().gui_get_hovered_control()
	return describe(hovered) if hovered != null else {}
