class_name MainScreenView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Main menu screen view (UI1-L): renders MainMenuModel and forwards the buttons. UIR-2: a small muted
# version / build label sits at the bottom right (useful in playtest reports; `version_text`).
# ==============================================================================

const COLUMN_W: int = 320   # the widest button label sets the column when it is wider; every button fills the column

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: MainMenuModel = model as MainMenuModel
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	size_flags_vertical = Control.SIZE_EXPAND_FILL
	var centre := CenterContainer.new()
	centre.size_flags_vertical = Control.SIZE_EXPAND_FILL
	add_child(centre)
	var col := VBoxContainer.new()   # the column sits in the middle of the screen, both ways
	col.name = "MainColumn"
	col.add_theme_constant_override("separation", 10)
	col.custom_minimum_size = Vector2(COLUMN_W, 0)
	centre.add_child(col)
	var head: Label = MainMenuView.title("Tactical RPG")
	head.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	col.add_child(head)
	var sub: Label = MainMenuView.text("A small campaign of tactical battles.", UiTheme.COL_MUTED)
	sub.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	col.add_child(sub)
	var gap := Control.new()
	gap.custom_minimum_size = Vector2(0, 24)
	col.add_child(gap)
	if m.can_continue():
		col.add_child(_wide(MainMenuView.button("Continue", host.act.bind(m.continue_game))))
	col.add_child(_wide(MainMenuView.button("New game", m.open.bind(MenuRouter.NEW_GAME))))
	col.add_child(_wide(MainMenuView.button("Load game", m.open.bind(MenuRouter.LOAD))))
	col.add_child(_wide(MainMenuView.button("Settings", m.open.bind(MenuRouter.SETTINGS))))
	if m.show_tools():
		col.add_child(_wide(MainMenuView.button("Tools (Forge, Dojo, Test Arena)", host.open_tools)))
	if m.show_quit():
		col.add_child(_wide(MainMenuView.button("Quit", host.act.bind(m.quit_requested))))
	var version: Label = MainMenuView.text(MainMenuModel.version_text(), UiTheme.COL_MUTED, false)
	version.name = "VersionLabel"
	version.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	version.size_flags_horizontal = Control.SIZE_SHRINK_END
	version.add_theme_font_size_override("font_size", 13)
	add_child(version)

# Full column width, label centred (the other screens keep their left-aligned buttons).
static func _wide(b: Button) -> Button:
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	b.alignment = HORIZONTAL_ALIGNMENT_CENTER
	return b
