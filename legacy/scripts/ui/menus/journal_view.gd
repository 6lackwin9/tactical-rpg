class_name JournalView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Journal view (UI3-L): the quest list by group on the left, the selected entry on the right (giver,
# current objective, progress, rewards, loot on offer, linked quests). Up / down move the focus and the selection with it. Renders JournalModel.
# ==============================================================================

const LINK_LABELS: Dictionary = {"follows": "Follows", "leads_to": "Leads to", "related_quests": "Related"}

var _host: MainMenuView = null
var _model: JournalModel = null

func build(host: MainMenuView, model: RefCounted) -> void:
	_host = host
	_model = model as JournalModel
	add_theme_constant_override("separation", 10)
	add_child(MainMenuView.title("Journal"))
	var groups: Array[Dictionary] = _model.groups()
	if groups.is_empty():
		add_child(MainMenuView.text("No quests yet. People in the world will ask for help.", UiTheme.COL_MUTED))
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 32)
	row.size_flags_vertical = Control.SIZE_EXPAND_FILL
	add_child(row)
	var list := VBoxContainer.new()
	list.custom_minimum_size = Vector2(360, 0)
	row.add_child(list)
	var selected_button: Button = null
	for g: Dictionary in groups:
		list.add_child(MainMenuView.text(str(g["label"]), UiTheme.COL_HEADER, false))
		for raw_row: Variant in DictUtil.array_of(g["rows"]):
			var row_data: Dictionary = DictUtil.dict_of(raw_row)
			var qid: String = str(row_data["qid"])
			var quest_name: String = str(row_data["name"])
			var b: Button = MainMenuView.button(("▶ " if qid == _model.selected else "") + quest_name, func() -> void:
				host.act(_model.select.bind(qid))
				host.render())
			b.focus_entered.connect(func() -> void:   # arrows move the focus, and the selection follows it (rule 5)
				if qid != _model.selected:
					var _e: Array[String] = _model.select(qid)
					host.render())
			list.add_child(b)
			if qid == _model.selected:
				selected_button = b
	row.add_child(_detail(_model.entry(_model.selected)))
	add_child(MainMenuView.button("Back", host.router.pop))
	if selected_button != null:
		set_meta(MainMenuView.FOCUS_META, selected_button)   # the host focuses the selected quest, not the first button

static func _detail(e: Dictionary) -> Control:
	var col := VBoxContainer.new()
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if e.is_empty():
		return col
	col.add_child(MainMenuView.text(str(e["name"]), UiTheme.COL_TITLE, false))
	if str(e["status"]) == "blocked":
		col.add_child(MainMenuView.text("On hold", UiTheme.COL_WARN, false))
	if not str(e["description"]).is_empty():
		col.add_child(MainMenuView.text(str(e["description"])))
	if str(e["status"]) == "blocked":
		return col
	var who: String = str(e["giver"])
	if not str(e["turn_in"]).is_empty() and e["turn_in"] != e["giver"]:
		who += "  ·  turn in to %s" % str(e["turn_in"])
	if not who.is_empty():
		col.add_child(MainMenuView.text("From " + who, UiTheme.COL_MUTED, false))
	col.add_child(MainMenuView.text("Progress %d / %d" % [DictUtil.to_int(e["objectives_done"]), DictUtil.to_int(e["objectives_total"])], UiTheme.COL_MUTED, false))
	if not str(e["objective"]).is_empty():
		col.add_child(MainMenuView.text("Now: " + str(e["objective"]), UiTheme.COL_GOOD))
	elif str(e["status"]) == "ready":
		col.add_child(MainMenuView.text("Ready to turn in.", UiTheme.COL_GOOD, false))
	var rewards: Array[String] = JsonUtil.strings(e["rewards"])
	if not rewards.is_empty():
		col.add_child(MainMenuView.text("Rewards: " + ", ".join(rewards)))
	var offers: Array[String] = JsonUtil.strings(e["offers"])
	if not offers.is_empty():
		col.add_child(MainMenuView.text("Choose %d of: %s" % [DictUtil.to_int(e["picks_allowed"]), ", ".join(offers)]))
	for l: Variant in DictUtil.array_of(e["links"]):
		var link: Dictionary = DictUtil.dict_of(l)
		col.add_child(MainMenuView.text("%s: %s" % [str(LINK_LABELS.get(str(link["kind"]), "Linked")), str(link["name"])], UiTheme.COL_MUTED, false))
	return col
