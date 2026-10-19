class_name NewGameView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - New game view (UI1-L; UIR-2): a ScreenFrame with one selectable card per playable world (name, tagline,
# its fixed starting party); a click or Enter on a card selects it (MenuStyle selected box, the label never changes);
# Back first and Start (the selected world) last in the action bar.
# ==============================================================================

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: NewGameModel = model as NewGameModel
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	size_flags_vertical = Control.SIZE_EXPAND_FILL
	var worlds: Array[Dictionary] = m.worlds()
	var start: Button = MainMenuView.button("Start", host.act.bind(m.start.bind(-1)), not worlds.is_empty() and not m.selected.is_empty())
	var frame: ScreenFrame = ScreenFrame.make("New game", MainMenuView.button("Back", host.router.pop), start)
	add_child(frame)
	if worlds.is_empty():
		frame.column.add_child(MainMenuView.text("No world is playable yet (a world needs a start party and a start location).", UiTheme.COL_WARN))
	for w: Dictionary in worlds:
		var id: String = str(w["id"])
		var lines: PackedStringArray = PackedStringArray([str(w["name"])])
		if not str(w["tagline"]).is_empty():
			lines.append(str(w["tagline"]))
		lines.append(NewGameModel.party_line(DictUtil.array_of(w["party"])))
		var card: Button = MainMenuView.button("\n".join(lines), func() -> void:
			host.act(m.select.bind(id))
			host.render())
		card.name = "WorldCard_" + id
		card.alignment = HORIZONTAL_ALIGNMENT_LEFT
		card.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		card.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		MenuStyle.mark_selected(card, id == m.selected)
		frame.column.add_child(card)
		if id == m.selected:
			set_meta(MainMenuView.FOCUS_META, card)
