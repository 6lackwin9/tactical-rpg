class_name SaveSlotsView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Save / load slots view (UI1-L; UIR-2): a ScreenFrame (Back in the pinned bar) with one fixed-height row
# per slot (autosave, 1-5): two lines always (header, then "Empty" or where and who), and the row actions in fixed
# slots at the right (Load or Save here, Export, Import, Delete); an absent action leaves its slot empty (no disabled
# buttons). Export / Import go through a file dialog (SAVE-IO; hidden in the web build, which has no file system).
# ==============================================================================

const SLOT_NAMES: Array[String] = ["SlotMain", "SlotExport", "SlotImport", "SlotDelete"]
const SLOT_EXPORT: int = 1   # indices into SLOT_NAMES of the file-dialog slots (absent in the web build)
const SLOT_IMPORT: int = 2

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: SaveSlotsModel = model as SaveSlotsModel
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	size_flags_vertical = Control.SIZE_EXPAND_FILL
	var frame: ScreenFrame = ScreenFrame.make("Save game" if m.saving else "Load game", MainMenuView.button("Back", host.router.pop))
	add_child(frame)
	frame.scroll.name = "SlotScroll"   # the rows scroll; Back stays pinned below
	var files: bool = files_available()
	var first: Button = null
	for row: Dictionary in m.rows():
		var slot: int = DictUtil.to_int(row["slot"])
		var status: String = str(row["status"])
		var card: VBoxContainer = UiTheme.card(frame.column)
		card.get_parent().name = "SlotRow%d" % slot
		var line := HBoxContainer.new()
		line.add_theme_constant_override("separation", 8)
		card.add_child(line)
		var info := VBoxContainer.new()
		info.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		info.add_theme_constant_override("separation", 2)
		line.add_child(info)
		var head: String = str(row["label"])
		if status == "ok":
			head += "  ·  %s  ·  %s" % [str(row["world"]), str(row["saved_at"])]
		info.add_child(_line(head, UiTheme.COL_HEADER if status == "ok" else UiTheme.COL_MUTED))
		var second: String = "Empty"
		if status == "ok":
			var where: String = str(row["location"])
			second = ("%s  ·  " % where if not where.is_empty() else "") + str(row["party"])
		elif status != "empty":
			second = "Can't be read (%s)" % status
		info.add_child(_line(second, UiTheme.COL_TEXT if status == "ok" else UiTheme.COL_MUTED))
		var main: Button = null
		if m.saving and DictUtil.to_bool(row["can_save"]):
			main = MainMenuView.button("Save here", host.act.bind(m.save.bind(slot)))
		elif not m.saving and DictUtil.to_bool(row["can_load"]):
			main = MainMenuView.button("Load", host.act.bind(m.load.bind(slot)))
		var exp_b: Button = null
		if files and DictUtil.to_bool(row["can_export"]):
			exp_b = MainMenuView.button("Export", _pick.bind(host, FileDialog.FILE_MODE_SAVE_FILE, str(row["export_name"]), func(path: String) -> void: host.act(m.export_slot.bind(slot, path))))
		var imp_b: Button = null
		if files and DictUtil.to_bool(row["can_import"]):
			imp_b = MainMenuView.button("Import", _pick.bind(host, FileDialog.FILE_MODE_OPEN_FILE, "", func(path: String) -> void: host.act(m.import_file.bind(path, slot))))
		var del_b: Button = null
		if DictUtil.to_bool(row["can_delete"]) and slot != SaveGame.AUTOSAVE_SLOT:
			del_b = MainMenuView.button("Delete", host.act.bind(m.delete.bind(slot)))
		var actions: Array[Button] = [main, exp_b, imp_b, del_b]
		var widest: Array[String] = ["Save here" if m.saving else "Load", "Export", "Import", "Delete"]
		for i: int in actions.size():
			if (i == SLOT_EXPORT or i == SLOT_IMPORT) and not files:
				continue
			line.add_child(_slot(actions[i], widest[i], SLOT_NAMES[i]))
		if first == null and main != null:
			first = main
	if first != null:
		set_meta(MainMenuView.FOCUS_META, first)

# A one-line label that never wraps (rows keep one height); long text ends in an ellipsis.
static func _line(value: String, color: Color) -> Label:
	var l: Label = MainMenuView.text(value, color, false)
	l.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.custom_minimum_size = Vector2(80, 0)
	return l

# The row-class button for a fixed action slot; an absent action keeps the slot's width with an invisible stand-in of
# the same label, so the slots line up down the list at every text size.
static func _slot(action: Button, label: String, slot_name: String) -> Button:
	var b: Button = action
	if b == null:
		b = MainMenuView.button(label, func() -> void: pass, false)
		b.name = slot_name + "Empty"
		b.modulate = Color(1, 1, 1, 0)
		b.focus_mode = Control.FOCUS_NONE
		b.mouse_filter = Control.MOUSE_FILTER_IGNORE
		b.set_meta("placeholder", true)
	else:
		b.name = slot_name
	return MainMenuView.small(b)

static func files_available() -> bool:
	return not OS.has_feature("web")

# A file dialog on the whole file system (native when the platform has one); `chosen` gets the picked path.
static func _pick(host: MainMenuView, mode: FileDialog.FileMode, file_name: String, chosen: Callable) -> void:
	var dialog := FileDialog.new()
	dialog.file_mode = mode
	dialog.access = FileDialog.ACCESS_FILESYSTEM
	dialog.use_native_dialog = true
	dialog.filters = PackedStringArray(["*.json ; Save files"])
	dialog.current_dir = OS.get_system_dir(OS.SYSTEM_DIR_DOCUMENTS)
	if not file_name.is_empty():
		dialog.current_file = file_name
	dialog.title = "Export save" if mode == FileDialog.FILE_MODE_SAVE_FILE else "Import save"
	dialog.file_selected.connect(func(path: String) -> void:
		chosen.call(path)
		dialog.queue_free())
	dialog.canceled.connect(dialog.queue_free)
	host.add_child(dialog)
	dialog.popup_centered_ratio(0.7)
