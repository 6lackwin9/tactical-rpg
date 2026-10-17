class_name MainScreenView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Main menu screen view (UI1-L): renders MainMenuModel and forwards the buttons.
# ==============================================================================

const COLUMN_W: int = 320   # the widest button label sets the column when it is wider; every button fills the column

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: MainMenuModel = model as MainMenuModel
	add_theme_constant_override("separation", 10)
	alignment = BoxContainer.ALIGNMENT_CENTER   # the column sits in the middle of the screen, both ways
	size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	custom_minimum_size = Vector2(COLUMN_W, 0)
	var head: Label = MainMenuView.title("Tactical RPG")
	head.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	add_child(head)
	var sub: Label = MainMenuView.text("A small campaign of tactical battles.", UiTheme.COL_MUTED)
	sub.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	add_child(sub)
	var gap := Control.new()
	gap.custom_minimum_size = Vector2(0, 24)
	add_child(gap)
	if m.can_continue():
		add_child(_wide(MainMenuView.button("Continue", host.act.bind(m.continue_game))))
	add_child(_wide(MainMenuView.button("New game", m.open.bind(MenuRouter.NEW_GAME))))
	add_child(_wide(MainMenuView.button("Load game", m.open.bind(MenuRouter.LOAD))))
	add_child(_wide(MainMenuView.button("Settings", m.open.bind(MenuRouter.SETTINGS))))
	if m.show_tools():
		add_child(_wide(MainMenuView.button("Tools (Forge, Dojo, Test Arena)", host.open_tools)))
	if m.show_quit():
		add_child(_wide(MainMenuView.button("Quit", host.act.bind(m.quit_requested))))

# Full column width, label centred (the other screens keep their left-aligned buttons).
static func _wide(b: Button) -> Button:
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	b.alignment = HORIZONTAL_ALIGNMENT_CENTER
	return b
