class_name SaveSlotsView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Save / load slots view (UI1-L): one row per slot (autosave, 1-5) with its header from peek; Load or
# Save plus Delete per row.
# ==============================================================================

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: SaveSlotsModel = model as SaveSlotsModel
	add_theme_constant_override("separation", 10)
	add_child(MainMenuView.title("Save game" if m.saving else "Load game"))
	for row: Dictionary in m.rows():
		var card: VBoxContainer = UiTheme.card(self)
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
	add_child(MainMenuView.button("Back", host.router.pop))
