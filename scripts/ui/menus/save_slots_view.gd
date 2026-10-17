class_name SaveSlotsView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Save / load slots view (UI1-L): one row per slot (autosave, 1-5) with its header from peek; Load or
# Save plus Delete per row; Export (readable saves) and Import (manual slots) through a file dialog (SAVE-IO; hidden in
# the web build, which has no file system to pick from).
# ==============================================================================

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: SaveSlotsModel = model as SaveSlotsModel
	add_theme_constant_override("separation", 10)
	add_child(MainMenuView.title("Save game" if m.saving else "Load game"))
	# The rows scroll (they fill the height left between the title and Back); Back stays on screen below the list.
	var scroll := ScrollContainer.new()
	scroll.name = "SlotScroll"
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.follow_focus = true
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	add_child(scroll)
	var list := VBoxContainer.new()
	list.add_theme_constant_override("separation", 8)
	list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(list)
	for row: Dictionary in m.rows():
		var card: VBoxContainer = UiTheme.card(list)
		var status: String = str(row["status"])
		var head: String = str(row["label"])
		match status:
			"empty":
				head += "  (empty)"
			"ok":
				head += "  %s  ·  %s" % [str(row["world"]), str(row["saved_at"])]
			_:
				head += "  (%s)" % status
		card.add_child(MainMenuView.text(head, UiTheme.COL_HEADER if status == "ok" else UiTheme.COL_MUTED))
		if status == "ok":
			var where: String = str(row["location"])
			card.add_child(MainMenuView.text(("%s  ·  " % where if not where.is_empty() else "") + str(row["party"])))
		var buttons := HBoxContainer.new()
		card.add_child(buttons)
		var slot: int = DictUtil.to_int(row["slot"])
		if m.saving:
			if DictUtil.to_bool(row["can_save"]):
				buttons.add_child(MainMenuView.button("Save here", host.act.bind(m.save.bind(slot))))
		else:
			buttons.add_child(MainMenuView.button("Load", host.act.bind(m.load.bind(slot)), DictUtil.to_bool(row["can_load"])))
		if DictUtil.to_bool(row["can_delete"]) and slot != SaveGame.AUTOSAVE_SLOT:
			buttons.add_child(MainMenuView.button("Delete", host.act.bind(m.delete.bind(slot))))
		if files_available():
			if DictUtil.to_bool(row["can_export"]):
				buttons.add_child(MainMenuView.small(MainMenuView.button("Export", _pick.bind(host, FileDialog.FILE_MODE_SAVE_FILE, str(row["export_name"]), func(path: String) -> void: host.act(m.export_slot.bind(slot, path)))), 140))
			if DictUtil.to_bool(row["can_import"]):
				buttons.add_child(MainMenuView.small(MainMenuView.button("Import", _pick.bind(host, FileDialog.FILE_MODE_OPEN_FILE, "", func(path: String) -> void: host.act(m.import_file.bind(path, slot)))), 140))
	var back: Button = MainMenuView.button("Back", host.router.pop)
	back.name = "BackButton"
	add_child(back)

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
