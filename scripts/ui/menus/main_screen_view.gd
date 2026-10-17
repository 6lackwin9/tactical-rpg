class_name MainScreenView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Main menu screen view (UI1-L): renders MainMenuModel and forwards the buttons.
# ==============================================================================

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: MainMenuModel = model as MainMenuModel
	add_theme_constant_override("separation", 10)
	add_child(MainMenuView.title("Tactical RPG"))
	add_child(MainMenuView.text("A small campaign of tactical battles.", UiTheme.COL_MUTED))
	var gap := Control.new()
	gap.custom_minimum_size = Vector2(0, 24)
	add_child(gap)
	if m.can_continue():
		add_child(MainMenuView.button("Continue", host.act.bind(m.continue_game)))
	add_child(MainMenuView.button("New game", m.open.bind(MenuRouter.NEW_GAME)))
	add_child(MainMenuView.button("Load game", m.open.bind(MenuRouter.LOAD)))
	add_child(MainMenuView.button("Settings", m.open.bind(MenuRouter.SETTINGS)))
	if m.show_tools():
		add_child(MainMenuView.button("Tools (Forge, Dojo, Test Arena)", host.open_tools))
	add_child(MainMenuView.button("Quit", host.act.bind(m.quit_requested)))
