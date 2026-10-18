class_name BattleResultsView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Battle results view (UI2-L rule 8, BATTLE-FLOW): the popup over the combat screen. Header with the
# headline and the buttons Minimise, Replay and X (close = what Continue did: autosave, back to the map); per active
# member a row with level, an XP bar (progress to the next level, the gained part highlighted), "Level up!" and the
# skippable level-up / loadout actions; then every non-XP reward. Renders BattleResultsModel; the host (MainMenuView)
# shows it in the popup panel, or as a pill ("Victory ▴") while the model is minimised.
# ==============================================================================

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: BattleResultsModel = model as BattleResultsModel
	add_theme_constant_override("separation", 10)
	var header := HBoxContainer.new()
	header.name = "ResultsHeader"
	header.add_theme_constant_override("separation", 8)
	add_child(header)
	var head: Label = MainMenuView.title(m.headline())
	head.name = "ResultsTitle"
	head.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	header.add_child(head)
	if m.can_replay():
		header.add_child(_header_button("ResultsReplay", "Replay", "Replay the battle from the start (this window minimises while it plays)", host.act.bind(m.replay)))
	header.add_child(_header_button("ResultsMinimise", "—", "Minimise: look at the battlefield", m.toggle_minimised))
	var close: Button = _header_button("ResultsClose", "✕", "Close and carry on (Esc)", host.cancel_pressed)
	header.add_child(close)
	set_meta(MainMenuView.FOCUS_META, close)
	if not m.outcome_note().is_empty():
		add_child(MainMenuView.text(m.outcome_note(), UiTheme.COL_MUTED))
	var scroll := ScrollContainer.new()
	scroll.name = "ResultsScroll"
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.follow_focus = true
	add_child(scroll)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 10)
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(body)
	var rows: Array[Dictionary] = m.rows()
	if not rows.is_empty():
		var card: VBoxContainer = UiTheme.card(body)
		for r: Dictionary in rows:
			_member(card, host, m, r)
	body.add_child(UiTheme.header("Rewards"))
	var lines: Array[String] = m.reward_lines()
	if lines.is_empty():
		body.add_child(MainMenuView.text("No other rewards.", UiTheme.COL_MUTED))
	for line: String in lines:
		body.add_child(MainMenuView.text("•  " + line, UiTheme.COL_TEXT))
	if m.prompt_level_up():
		body.add_child(MainMenuView.text("Points to spend. Level up now, or later from the party screen.", UiTheme.COL_MUTED))

static func _header_button(node_name: String, label: String, tip: String, callback: Callable) -> Button:
	var b := Button.new()
	b.name = node_name
	b.text = label
	b.tooltip_text = tip
	b.focus_mode = Control.FOCUS_ALL
	b.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	b.pressed.connect(callback)
	return b

func _member(card: VBoxContainer, host: MainMenuView, m: BattleResultsModel, r: Dictionary) -> void:
	var id: String = str(r["id"])
	var old_level: int = DictUtil.to_int(r["old_level"])
	var new_level: int = DictUtil.to_int(r["new_level"])
	var leveled: bool = new_level > old_level
	var line := HBoxContainer.new()
	line.add_theme_constant_override("separation", 10)
	card.add_child(line)
	var who: Label = MainMenuView.text("%s   Lv %d" % [str(r["name"]), new_level], UiTheme.COL_GOOD if leveled else UiTheme.COL_TEXT, false)
	who.custom_minimum_size = Vector2(170, 0)
	line.add_child(who)
	var bar := XpBar.new(float(r["fill"]), float(r["gain"]))
	bar.name = "XpBar_" + id
	line.add_child(bar)
	var sub := HFlowContainer.new()   # wraps at big text sizes instead of widening the popup
	sub.add_theme_constant_override("h_separation", 12)
	card.add_child(sub)
	var tail: String = "   +%d XP" % DictUtil.to_int(r["xp"])
	tail += "  (max level)" if bool(r["maxed"]) else "  (%d to next level)" % DictUtil.to_int(r["to_next"])
	sub.add_child(MainMenuView.text(tail, UiTheme.COL_TITLE, false))
	if leveled:
		sub.add_child(MainMenuView.text("Level up!", UiTheme.COL_GOOD, false))
	if DictUtil.to_int(r["unspent"]) > 0:
		sub.add_child(MainMenuView.small(MainMenuView.button("Level up (%d)" % DictUtil.to_int(r["unspent"]), host.act.bind(m.level_up_now.bind(id)))))
	if leveled:
		var names: PackedStringArray = PackedStringArray()
		for t: Dictionary in LevelUpModel.new(id).new_tiers(old_level):
			names.append(str(t["name"]))
		if not names.is_empty():
			sub.add_child(MainMenuView.text("New: " + ", ".join(names), UiTheme.COL_HEADER, false))
			if m.has_free_slot(id):
				sub.add_child(MainMenuView.small(MainMenuView.button("Open loadout", host.act.bind(m.open_loadout.bind(id)))))
