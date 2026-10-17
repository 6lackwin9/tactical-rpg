class_name PartyView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Party view (UI2-L): the roster (active four marked, a dot on members with unspent points) and the
# selected member's stats, derived block, XP and passives, with doors to level-up, loadout and gear. Renders PartyModel.
# ==============================================================================

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: PartyModel = model as PartyModel
	add_theme_constant_override("separation", 10)
	add_child(MainMenuView.title("Party"))
	add_child(MainMenuView.text("Active %d / %d   ·   Gold %d" % [m.active_count(), PartyState.PARTY_MAX, m.gold()], UiTheme.COL_MUTED, false))
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 24)
	row.size_flags_vertical = Control.SIZE_EXPAND_FILL
	add_child(row)
	var roster := VBoxContainer.new()
	roster.custom_minimum_size = Vector2(420, 0)
	row.add_child(roster)
	for p: Dictionary in m.members():
		var id: String = str(p["id"])
		var line := HBoxContainer.new()
		roster.add_child(line)
		var label: String = "%s%s  Lv %d %s" % ["▶ " if id == m.selected else "", str(p["name"]), DictUtil.to_int(p["level"]), "•" if DictUtil.to_int(p["unspent"]) > 0 else ""]
		var pick: Button = MainMenuView.button(label, func() -> void:
			host.act(m.select.bind(id))
			host.render())
		line.add_child(pick)
		var toggle: Button = MainMenuView.button("Active" if DictUtil.to_bool(p["active"]) else "Reserve", func() -> void:
			host.act(m.toggle_active.bind(id))
			host.render())
		toggle.custom_minimum_size = Vector2(120, 0)
		line.add_child(toggle)
	row.add_child(_detail(host, m))
	add_child(MainMenuView.button("Back", host.router.pop))

func _detail(host: MainMenuView, m: PartyModel) -> Control:
	var col := VBoxContainer.new()
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var sm: StatsModel = m.stats()
	var xp: Dictionary = sm.xp()
	if xp.is_empty():
		return col
	col.add_child(MainMenuView.text("Level %d   ·   XP %d (%d to next)" % [DictUtil.to_int(xp["level"]), DictUtil.to_int(xp["total"]), DictUtil.to_int(xp["to_next"])], UiTheme.COL_TEXT, false))
	var grid := GridContainer.new()
	grid.columns = 5
	grid.add_theme_constant_override("h_separation", 18)
	col.add_child(grid)
	for head: String in ["Stat", "Base", "Growth", "Spent", "Total"]:
		grid.add_child(MainMenuView.text(head, UiTheme.COL_MUTED, false))
	var stats: Dictionary = sm.stats()
	for s: String in StatRules.STATS:
		var v: Dictionary = DictUtil.dict_of(stats.get(s, {}))
		grid.add_child(MainMenuView.text(s, UiTheme.COL_TEXT, false))
		for k: String in ["base", "growth", "spend", "total"]:
			grid.add_child(MainMenuView.text(str(DictUtil.to_int(v.get(k, 0))), UiTheme.COL_TEXT, false))
	col.add_child(derived_grid(sm.derived(), {}))
	var passives: PackedStringArray = PackedStringArray()
	for p: Dictionary in m.passives():
		passives.append("%s (%s)" % [str(p["name"]), str(p["source"])])
	col.add_child(MainMenuView.text("Passives: " + (", ".join(passives) if not passives.is_empty() else "none"), UiTheme.COL_MUTED))
	var buttons := HFlowContainer.new()
	col.add_child(buttons)
	var unspent: int = PartyPreview.member(m.selected).unspent() if PartyPreview.member(m.selected) != null else 0
	buttons.add_child(MainMenuView.small(MainMenuView.button("Level up (%d)" % unspent, host.act.bind(m.open.bind(MenuRouter.LEVEL_UP)), unspent > 0)))
	buttons.add_child(MainMenuView.small(MainMenuView.button("Skills", host.act.bind(m.open.bind(MenuRouter.LOADOUT)))))
	buttons.add_child(MainMenuView.small(MainMenuView.button("Gear", host.act.bind(m.open.bind(MenuRouter.GEAR)))))
	return col

# The derived block; with `after` non-empty each changed value shows "before → after".
static func derived_grid(before: Dictionary, after: Dictionary) -> GridContainer:
	var grid := GridContainer.new()
	grid.columns = 2
	grid.add_theme_constant_override("h_separation", 18)
	for key: String in PartyPreview.DERIVED_LABELS:
		grid.add_child(MainMenuView.text(str(PartyPreview.DERIVED_LABELS[key]), UiTheme.COL_MUTED, false))
		var was: String = str(before.get(key, "-"))
		var now: String = str(after.get(key, was))
		grid.add_child(MainMenuView.text(was if now == was else "%s → %s" % [was, now], UiTheme.COL_TEXT if now == was else UiTheme.COL_GOOD, false))
	return grid
