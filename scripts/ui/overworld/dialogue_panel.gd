class_name DialoguePanel
extends PanelContainer

# ==============================================================================
# Tactical RPG - Dialogue debug panel (S11, spec DLG1-L commit 3)
# Thin view over one DialogueRunner: speaker, text, visible choices (locked ones greyed and disabled), Continue for
# `next` / end nodes, and the turn-in step (recipient picker over the active members, one checkbox per offered loot,
# Confirm / Cancel). Every action goes through the runner; the panel only re-renders. A shop or fight result is handed
# to the owner through the signals; the owner calls shop_closed() when its shop panel closes.
# ==============================================================================

signal shop_opened(shop_qid: String)
signal fight_started(fight: Dictionary)
signal finished()
signal noted(text: String)

var runner: DialogueRunner = null
var pending: Dictionary = {}          # the turn-in step: choose()'s {pending, quest, offers, picks_allowed}
var recipient_id: String = ""
var picks: Array[String] = []

var _speaker: Label = null
var _text: Label = null
var _choices: VBoxContainer = null
var _turn_in: VBoxContainer = null

func _init() -> void:
	add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_PANEL_BG))
	var root := VBoxContainer.new()
	add_child(root)
	_speaker = UiTheme.label("", UiTheme.FONT_LG, UiTheme.COL_TITLE)
	root.add_child(_speaker)
	_text = UiTheme.label("")
	_text.autowrap_mode = TextServer.AUTOWRAP_WORD
	_text.custom_minimum_size = Vector2(240, 0)
	root.add_child(_text)
	_choices = VBoxContainer.new()
	root.add_child(_choices)
	_turn_in = VBoxContainer.new()
	root.add_child(_turn_in)
	visible = false

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

func refresh() -> void:
	UiTheme.clear_children(_choices)
	UiTheme.clear_children(_turn_in)
	if runner == null:
		return
	var v: Dictionary = runner.view()
	_speaker.text = str(v.get("speaker_name", ""))
	_text.text = str(v.get("text", ""))
	if not pending.is_empty():
		_build_turn_in()
		return
	var choices: Array = DictUtil.array_of(v.get("choices", []))
	for i: int in range(choices.size()):
		var c: Dictionary = DictUtil.dict_of(choices[i])
		var locked: bool = DictUtil.to_bool(c.get("locked", false))
		var btn: Button = UiTheme.button(str(c.get("text", "")), "Locked" if locked else "DialogueRunner.choose(%d)" % i, pick.bind(i))
		btn.disabled = locked
		_choices.add_child(btn)
	if choices.is_empty() and (DictUtil.to_bool(v.get("next", false)) or DictUtil.to_bool(v.get("end", false))):
		_choices.add_child(UiTheme.button("Continue", "DialogueRunner.advance", advance))

func _build_turn_in() -> void:
	_turn_in.add_child(UiTheme.header("Turn in: %s" % str(pending.get("quest", ""))))
	var who := OptionButton.new()
	for id: String in runner.gs.party.active_ids:
		who.add_item(id)
		if id == recipient_id:
			who.select(who.item_count - 1)
	who.item_selected.connect(func(i: int) -> void: set_recipient(who.get_item_text(i)))
	_turn_in.add_child(who)
	var allowed: int = DictUtil.to_int(pending.get("picks_allowed", 0))
	if allowed > 0:
		_turn_in.add_child(UiTheme.label("Pick up to %d:" % allowed, UiTheme.FONT_SM, UiTheme.COL_MUTED))
	for raw: Variant in DictUtil.array_of(pending.get("offers", [])):
		var lqid: String = str(raw)
		var box := CheckBox.new()
		box.text = str(WorldCatalog.resolve(runner.gs.world_id, "loot", lqid).get("name", lqid))
		box.button_pressed = picks.has(lqid)
		box.toggled.connect(func(on: bool) -> void: toggle_pick(lqid, on))
		_turn_in.add_child(box)
	_turn_in.add_child(UiTheme.button("Confirm", "DialogueRunner.confirm_turn_in", confirm_turn_in))
	_turn_in.add_child(UiTheme.button("Cancel", "DialogueRunner.cancel", cancel_turn_in))
