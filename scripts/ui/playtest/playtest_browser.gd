class_name PlaytestBrowser
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Playtest Browser (PLAYTEST1 rule 6, the Scene Runner "Playtests" panel)
# Lists recordings from user://playtests and res://playtests/{inbox,repro} (badge `repo`), newest first,
# with battles, notes and a stale badge; per recording: Open (battle), Notes, Take over (round), Copy to
# repo. The recording folder path is shown as selectable text next to Open folder. Scenarios
# (res://playtests/scenarios) are listed below, one entry each with its seed.
# The runner owns launching (signals) and the `record_playtests` pref (record_toggled).
# ==============================================================================

signal open_requested(path: String, battle: int)
signal take_over_requested(path: String, battle: int, round_num: int)
signal scenario_requested(scenario: Dictionary)
signal record_toggled(on: bool)

const MAX_LISTED: int = 30
const DETAIL_HEIGHT: float = 80.0

var record_box: CheckBox = null
var folder_text: LineEdit = null
var list_box: VBoxContainer = null
var scenario_box: VBoxContainer = null
var detail: RichTextLabel = null
var status: Label = null
var dirs: Array[String] = []              # folders listed ([] = PlaytestStore defaults)
var scenario_dir: String = PlaytestStore.SCENARIO_DIR
var summaries: Array[Dictionary] = []
var summarized: int = 0                   # recordings (re)loaded by the last refresh (tests)

var _fingerprint: Dictionary = {}         # this build's ContentFingerprint, computed on the first / a full refresh
var _cache: Dictionary = {}               # path -> {mtime, summary}

func _init() -> void:
	name = "PlaytestBrowser"
	size_flags_vertical = Control.SIZE_EXPAND_FILL
	add_theme_constant_override("separation", 6)
	var top := HBoxContainer.new()
	add_child(top)
	record_box = CheckBox.new()
	record_box.text = "Record playtests"
	record_box.tooltip_text = "Record every battle of this run (debug builds), with the UI trail and F8 notes"
	record_box.button_pressed = true
	record_box.add_theme_font_size_override("font_size", UiTheme.FONT_MD)
	record_box.toggled.connect(func(on: bool) -> void: record_toggled.emit(on))
	top.add_child(record_box)
	top.add_child(UiTheme.button("⟳ Refresh", "List the recordings again", func() -> void: refresh(true), UiTheme.FONT_SM))
	top.add_child(UiTheme.label("F8 or ✎ Note in a running scene adds a note", UiTheme.FONT_SM, UiTheme.COL_MUTED))
	var folder_row := HBoxContainer.new()
	add_child(folder_row)
	folder_text = LineEdit.new()
	folder_text.editable = false
	folder_text.selecting_enabled = true
	folder_text.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	folder_text.add_theme_font_size_override("font_size", UiTheme.FONT_SM)
	folder_text.tooltip_text = "Where recordings are written (select and copy)"
	folder_row.add_child(folder_text)
	folder_row.add_child(UiTheme.button("📂 Open folder", "Open the recordings folder in the file manager",
		func() -> void: OS.shell_open(ProjectSettings.globalize_path(PlaytestStore.root)), UiTheme.FONT_SM))
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(scroll)
	var inner := VBoxContainer.new()
	inner.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	inner.add_theme_constant_override("separation", 6)
	scroll.add_child(inner)
	list_box = VBoxContainer.new()
	list_box.add_theme_constant_override("separation", 6)
	inner.add_child(list_box)
	inner.add_child(UiTheme.header("SCENARIOS"))
	scenario_box = VBoxContainer.new()
	scenario_box.add_theme_constant_override("separation", 6)
	inner.add_child(scenario_box)
	detail = RichTextLabel.new()
	detail.bbcode_enabled = true
	detail.custom_minimum_size.y = DETAIL_HEIGHT
	detail.selection_enabled = true
	detail.add_theme_font_size_override("normal_font_size", UiTheme.FONT_SM)
	detail.add_theme_font_size_override("bold_font_size", UiTheme.FONT_SM)
	detail.text = "[color=#94a3b8]Notes of a recording appear here.[/color]"
	add_child(detail)
	status = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_MUTED)
	status.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	add_child(status)

func set_record_enabled(on: bool) -> void:
	record_box.set_pressed_no_signal(on)

# Rebuilds the list. Only new or changed recordings are loaded again; `full` (the Refresh button) also
# recomputes the content fingerprint and reloads every recording.
func refresh(full: bool = false) -> void:
	folder_text.text = ProjectSettings.globalize_path(PlaytestStore.root)
	UiTheme.clear_children(list_box)
	UiTheme.clear_children(scenario_box)
	if full or _fingerprint.is_empty():
		_fingerprint = ContentFingerprint.compute()
		_cache.clear()
	summaries.clear()
	summarized = 0
	for entry: Dictionary in PlaytestStore.list(dirs).slice(0, MAX_LISTED):
		var path: String = str(entry["path"])
		var mtime: int = DictUtil.to_int(entry["mtime"])
		var cached: Dictionary = DictUtil.dict_of(_cache.get(path, {}))
		if cached.is_empty() or DictUtil.to_int(cached["mtime"]) != mtime:
			cached = {"mtime": mtime, "summary": PlaytestStore.summary(path, _fingerprint)}
			_cache[path] = cached
			summarized += 1
		summaries.append(DictUtil.dict_of(cached["summary"]))
	if summaries.is_empty():
		list_box.add_child(UiTheme.label("No recordings yet: launch a battle with recording on.", UiTheme.FONT_SM, UiTheme.COL_MUTED))
	for s: Dictionary in summaries:
		_recording_card(s)
	var found: Array[Dictionary] = PlaytestStore.scenarios(scenario_dir)
	if found.is_empty():
		scenario_box.add_child(UiTheme.label("No scenarios in playtests/scenarios.", UiTheme.FONT_SM, UiTheme.COL_MUTED))
	for sc: Dictionary in found:
		_scenario_card(sc)

func _recording_card(s: Dictionary) -> void:
	var card: VBoxContainer = UiTheme.card(list_box)
	var badges: String = ("  [repo]" if DictUtil.to_bool(s["repo"]) else "") + ("  [stale]" if DictUtil.to_bool(s["stale"]) else "") + ("  [agent]" if str(s["origin"]) == "agent" else "")
	card.add_child(UiTheme.label("%s%s" % [str(s["id"]), badges], UiTheme.FONT_MD, UiTheme.COL_WARN if DictUtil.to_bool(s["stale"]) else UiTheme.COL_HEADER))
	if not DictUtil.to_bool(s["ok"]):
		card.add_child(UiTheme.label(str(s["error"]), UiTheme.FONT_SM, UiTheme.COL_BAD))
		return
	var battles: Array = DictUtil.array_of(s["battles"])
	card.add_child(UiTheme.label("%d battle(s), %d note(s)%s" % [battles.size(), DictUtil.to_int(s["notes"]), "  ·  branch" if s["parent"] != null else ""], UiTheme.FONT_SM, UiTheme.COL_MUTED))
	if battles.is_empty():
		return
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 4)
	card.add_child(row)
	var pick := OptionButton.new()
	pick.tooltip_text = "Battle to open or take over"
	pick.add_theme_font_size_override("font_size", UiTheme.FONT_SM)
	pick.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pick.clip_text = true
	for raw: Variant in battles:
		var b: Dictionary = DictUtil.dict_of(raw)
		pick.add_item("%d %s r%d %s" % [DictUtil.to_int(b["index"]), str(b["encounter"]), DictUtil.to_int(b["rounds"]), str(DictUtil.dict_of(b["end"]).get("result", ""))])
	row.add_child(pick)
	var round_pick := SpinBox.new()
	round_pick.min_value = 1
	round_pick.prefix = "round"
	round_pick.get_line_edit().add_theme_font_size_override("font_size", UiTheme.FONT_SM)
	round_pick.tooltip_text = "Round to take over (rounds before it are replayed)"
	var path: String = str(s["path"])
	var sync_rounds: Callable = func(index: int) -> void:
		var chosen: Dictionary = DictUtil.dict_of(battles[clampi(index, 0, battles.size() - 1)])
		round_pick.max_value = DictUtil.to_int(chosen["rounds"]) + 1
	pick.item_selected.connect(sync_rounds)
	sync_rounds.call(0)
	row.add_child(round_pick)
	var buttons := HBoxContainer.new()
	buttons.add_theme_constant_override("separation", 4)
	card.add_child(buttons)
	buttons.add_child(UiTheme.button("▶ Open", "Play this battle back in the arena", func() -> void: open_requested.emit(path, pick.selected), UiTheme.FONT_SM))
	buttons.add_child(UiTheme.button("📝 Notes", "Show this recording's notes", func() -> void: show_notes(path), UiTheme.FONT_SM))
	buttons.add_child(UiTheme.button("⤷ Take over", "Replay to the chosen round, then plan it yourself (a new recording)",
		func() -> void: take_over_requested.emit(path, pick.selected, int(round_pick.value)), UiTheme.FONT_SM))
	if not DictUtil.to_bool(s["repo"]):
		buttons.add_child(UiTheme.button("⇪ Copy to repo", "Copy the recording and its images to playtests/inbox/<id>/ (then push it)",
			func() -> void: copy_to_repo(path), UiTheme.FONT_SM))

func _scenario_card(sc: Dictionary) -> void:
	var card: VBoxContainer = UiTheme.card(scenario_box)
	card.add_child(UiTheme.label("%s  (%s, seed %d)" % [str(sc.get("title", sc["id"])), str(sc["encounter"]), DictUtil.to_int(sc["seed"])], UiTheme.FONT_MD, UiTheme.COL_HEADER))
	var why: Label = UiTheme.label(str(sc.get("why", "")), UiTheme.FONT_SM, UiTheme.COL_MUTED)
	why.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	card.add_child(why)
	card.add_child(UiTheme.button("▶ Launch", "Play this encounter with the scenario's seed (recorded)", func() -> void: scenario_requested.emit(sc), UiTheme.FONT_SM))

func show_notes(path: String) -> void:
	var data: Dictionary = PlaytestRecord.normalize(DictUtil.dict_of(PlaytestStore.load_json(path).get("data", {})))
	var lines: Array[String] = ["[b]Notes of %s[/b]" % str(data.get("id", ""))]
	for raw: Variant in DictUtil.array_of(data.get("notes", [])):
		var n: Dictionary = DictUtil.dict_of(raw)
		var at: String = "outside a battle" if DictUtil.to_int(n["battle"]) < 0 and str(n["phase"]) != "replay" else "battle %d round %d block %d" % [DictUtil.to_int(n["battle"]), DictUtil.to_int(n["round"]), DictUtil.to_int(n["block"])]
		lines.append("#%d  %s  %s  [%s]  %s" % [DictUtil.to_int(n["n"]), at, str(n["phase"]), ", ".join(JsonUtil.strings(n["tags"])), str(n["text"]).replace("[", "[lb]")])
	if lines.size() == 1:
		lines.append("(no notes)")
	detail.text = "\n".join(lines)

func copy_to_repo(path: String) -> void:
	var result: Dictionary = PlaytestStore.copy_to_repo(path)
	if str(result["error"]).is_empty():
		status.text = "Copied to %s: push it on any branch and tell the agent the id." % ProjectSettings.globalize_path(str(result["dest"]))
	else:
		status.text = str(result["error"])
