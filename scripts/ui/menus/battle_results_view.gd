class_name BattleResultsView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Battle results view (UI2-L rule 8): XP and levels per active member, new skill tiers, quests advanced,
# a skippable level-up prompt per member and Continue (autosaves, back to the map). Renders BattleResultsModel.
# ==============================================================================

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: BattleResultsModel = model as BattleResultsModel
	add_theme_constant_override("separation", 10)
	add_child(MainMenuView.title(m.headline()))
	if not m.outcome_note().is_empty():
		add_child(MainMenuView.text(m.outcome_note(), UiTheme.COL_MUTED))
	var card: VBoxContainer = UiTheme.card(self)
	for r: Dictionary in m.rows():
		var id: String = str(r["id"])
		var old_level: int = DictUtil.to_int(r["old_level"])
		var new_level: int = DictUtil.to_int(r["new_level"])
		var line := HBoxContainer.new()
		card.add_child(line)
		var lv: String = "Lv %d → %d" % [old_level, new_level] if new_level > old_level else "Lv %d" % new_level
		line.add_child(MainMenuView.text("%s   +%d XP   %s" % [str(r["name"]), DictUtil.to_int(r["xp"]), lv], UiTheme.COL_GOOD if new_level > old_level else UiTheme.COL_TEXT, false))
		if DictUtil.to_int(r["unspent"]) > 0:
			line.add_child(MainMenuView.small(MainMenuView.button("Level up (%d)" % DictUtil.to_int(r["unspent"]), host.act.bind(m.level_up_now.bind(id)))))
		if new_level > old_level:
			var names: PackedStringArray = PackedStringArray()
			for t: Dictionary in LevelUpModel.new(id).new_tiers(old_level):
				names.append(str(t["name"]))
			if not names.is_empty():
				var new_line := HBoxContainer.new()
				card.add_child(new_line)
				new_line.add_child(MainMenuView.text("   New: " + ", ".join(names), UiTheme.COL_HEADER, false))
				if m.has_free_slot(id):
					new_line.add_child(MainMenuView.small(MainMenuView.button("Open loadout", host.act.bind(m.open_loadout.bind(id)))))
	var rewards: String = m.rewards_text()
	if not rewards.is_empty():
		add_child(MainMenuView.text(rewards, UiTheme.COL_MUTED))
	if m.prompt_level_up():
		add_child(MainMenuView.text("Points to spend. Level up now, or later from the party screen.", UiTheme.COL_MUTED))
	add_child(MainMenuView.button("Continue", host.act.bind(m.continue_on)))
