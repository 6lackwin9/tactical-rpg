class_name LevelUpView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Level-up view (UI2-L; UIR-3): the allocation table (Stat, Now, Added, New with - / + per row; Auto and
# Revert beside the rows), the derived block before → after, and Confirm (permanent) as the bar's primary. parts() is the
# Level up tab of the Character popup; build() shows the same body alone when the battle results open a level-up.
# Renders LevelUpModel.
# ==============================================================================

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: LevelUpModel = model as LevelUpModel
	var member: UnitState = PartyPreview.member(m.member_id)
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	size_flags_vertical = Control.SIZE_EXPAND_FILL
	var detail: Label = CharacterView.new_detail("Move to a stat to read what it does.")
	CharacterView.adopt(self, CharacterView.frame(host, "Level up: %s" % (PartyPreview.name_of(member) if member != null else m.member_id), parts(host, m, detail), detail))

# {body, extras, primary} for the bar.
static func parts(host: MainMenuView, m: LevelUpModel, detail: Label) -> Dictionary:
	var body := VBoxContainer.new()
	body.name = "LevelUpBody"
	body.add_theme_constant_override("separation", 10)
	var member: UnitState = PartyPreview.member(m.member_id)
	var left: int = m.points_left()
	body.add_child(MainMenuView.text("Points left: %d   (confirmed points are permanent)" % left if left > 0 or m.has_draft() else "No points to spend. Points come with each level.", UiTheme.COL_MUTED, false))
	var row := HFlowContainer.new()   # wraps at the larger text sizes instead of overflowing
	row.name = "LevelUpRow"
	row.add_theme_constant_override("h_separation", 32)
	body.add_child(row)
	row.add_child(_allocation(host, m, member, detail))
	row.add_child(_tools(host, m, detail))
	row.add_child(CharacterView.derived_grid(m.before(), m.after()))
	var confirm: Button = MainMenuView.button("Confirm", _redraw.bind(host, m.ask_confirm.bind(host.router)), m.has_draft())
	return {"body": body, "extras": [] as Array[Button], "primary": confirm}

# The allocation table: Stat / Now / Added / New with - and + per row (Auto and Revert sit beside it, see _tools; swap the layout in parts()).
static func _allocation(host: MainMenuView, m: LevelUpModel, member: UnitState, detail: Label) -> Control:
	var col := VBoxContainer.new()
	col.name = "Allocation"
	col.add_theme_constant_override("separation", 10)
	var grid := GridContainer.new()
	grid.name = "AllocationGrid"
	grid.columns = 6
	grid.add_theme_constant_override("h_separation", 14)
	col.add_child(grid)
	for head: String in ["Stat", "Now", "Added", "New"]:
		grid.add_child(CharacterView.cell(head, UiTheme.COL_MUTED, head != "Stat"))
	grid.add_child(Control.new())
	grid.add_child(Control.new())
	var stats: Dictionary = StatsModel.new(m.member_id).stats() if member != null else {}
	var roles: Dictionary = {}
	for d: Dictionary in DojoStatDefs.STATS:
		roles[str(d["k"])] = "%s: %s" % [str(d["name"]), str(d["role"])]
	for s: String in StatRules.STATS:
		var drafted: int = DictUtil.to_int(m.draft.get(s, 0))
		var now: int = DictUtil.to_int(DictUtil.dict_of(stats.get(s, {})).get("total", 0))
		var hot: Color = UiTheme.COL_GOOD if drafted > 0 else UiTheme.COL_TEXT
		var tip: String = str(roles.get(s, s))
		grid.add_child(CharacterView.cell(s, UiTheme.COL_TEXT, false))
		grid.add_child(CharacterView.cell(str(now)))
		grid.add_child(CharacterView.cell("+%d" % drafted if drafted > 0 else "0", hot if drafted > 0 else UiTheme.COL_MUTED))
		grid.add_child(CharacterView.cell(str(now + drafted), hot))
		var minus: Button = MainMenuView.step(MainMenuView.button("-", _redraw.bind(host, m.remove.bind(s)), drafted > 0))
		minus.name = "Minus_" + s
		var plus: Button = MainMenuView.step(MainMenuView.button("+", _redraw.bind(host, m.add.bind(s)), m.points_left() > 0))
		plus.name = "Plus_" + s
		for b: Button in [minus, plus]:
			CharacterView.show_on_focus(detail, b, tip)
			grid.add_child(b)
	return col

# Auto and Revert beside the allocation rows (not in the action bar).
static func _tools(host: MainMenuView, m: LevelUpModel, detail: Label) -> Control:
	var col := VBoxContainer.new()
	col.name = "AllocationTools"
	col.add_theme_constant_override("separation", 10)
	var auto: Button = MainMenuView.small(MainMenuView.button("Auto", _redraw.bind(host, func() -> Array[String]:
		m.auto()
		return [] as Array[String]), m.points_left() > 0))
	auto.name = "AutoButton"
	CharacterView.show_on_focus(detail, auto, "Spend the remaining points by the class growth weights.")
	col.add_child(auto)
	var revert: Button = MainMenuView.small(MainMenuView.button("Revert", _redraw.bind(host, func() -> Array[String]:
		m.cancel()
		return [] as Array[String]), m.has_draft()))
	revert.name = "RevertButton"
	CharacterView.show_on_focus(detail, revert, "Clear the drafted points. Nothing is spent until you Confirm.")
	col.add_child(revert)
	return col

static func _redraw(host: MainMenuView, action: Callable) -> void:
	host.act(action)
	host.render()
