class_name HubView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - In-game hub view (UI1-L): where the party is, the party summary and the hub entries (Party opens the
# UI2-L party screen, a dot marks unspent level-up points; Journal opens the UI3-L quest journal); Explore returns to the
# game scene when it exists.
# ==============================================================================

const LABELS: Dictionary = {MenuRouter.PARTY: "Party", MenuRouter.JOURNAL: "Journal", MenuRouter.SAVE: "Save game", MenuRouter.SETTINGS: "Settings"}

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: HubModel = model as HubModel
	add_theme_constant_override("separation", 10)
	add_child(MainMenuView.title(m.world_name()))
	var where: String = m.location_name()
	add_child(MainMenuView.text(where if not where.is_empty() else "On the road", UiTheme.COL_MUTED))
	var card: VBoxContainer = UiTheme.card(self)
	for p: Dictionary in m.party():
		card.add_child(MainMenuView.text("%s  Lv %d%s" % [str(p["name"]), DictUtil.to_int(p["level"]), "" if DictUtil.to_bool(p["active"]) else "  (reserve)"]))
	if ResourceLoader.exists(MainMenuView.GAME_SCENE) and CurrentGame.has_game():
		add_child(MainMenuView.button("Explore", func() -> void: host.open_game(CurrentGame.state())))
	for id: String in HubModel.ENTRIES:
		var label: String = str(LABELS[id]) + ("  •" if id == MenuRouter.PARTY and m.can_open(id) and PartyModel.new(host.router).badge() else "")
		add_child(MainMenuView.button(label, host.act.bind(m.open.bind(id)), m.can_open(id)))
	add_child(MainMenuView.button("Quit to main menu", host.act.bind(m.quit_to_menu)))
