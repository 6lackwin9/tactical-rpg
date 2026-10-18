class_name LoadoutView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Loadout view (UI2-L rule 5): the class (5) and extra (3) slots, the owned families with their tiers
# (locked tiers show their unlock level) and the basics (always active). Renders LoadoutModel.
# ==============================================================================

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: LoadoutModel = model as LoadoutModel
	var member: UnitState = PartyPreview.member(m.member_id)
	add_theme_constant_override("separation", 8)
	add_child(MainMenuView.title("Skills: %s" % (PartyPreview.name_of(member) if member != null else m.member_id)))
	add_child(MainMenuView.text("Default loadout (picked by the class)" if m.using_default else "Custom loadout", UiTheme.COL_MUTED, false))
	var fams: Dictionary = m.families()
	for group: String in ["class", "extra"]:
		var list: Array[String] = m.class_slots if group == "class" else m.extra_slots
		add_child(MainMenuView.text("%s slots %d / %d" % [group.capitalize(), list.size(), m.limit(group)], UiTheme.COL_HEADER, false))
		var slots := HFlowContainer.new()
		add_child(slots)
		for i: int in range(list.size()):
			slots.add_child(MainMenuView.small(MainMenuView.button(SkillCatalog.family_name(list[i]) + "  ✕", LevelUpView._redraw.bind(host, m.unslot.bind(group, i)))))
		var rows: Array = DictUtil.array_of(fams.get(group, []))
		for raw: Variant in rows:
			var f: Dictionary = DictUtil.dict_of(raw)
			var fam: String = str(f["family"])
			var line := HBoxContainer.new()
			add_child(line)
			line.add_child(MainMenuView.small(MainMenuView.button("Slot", LevelUpView._redraw.bind(host, m.slot.bind(fam, group, list.size())), not list.has(fam)), 90))
			line.add_child(MainMenuView.text(family_line(f), UiTheme.COL_TEXT, false))
	var basics: Array[String] = []
	for fam: String in m.basics():
		basics.append(SkillCatalog.family_name(fam))
	add_child(MainMenuView.text(basics_line(basics), UiTheme.COL_MUTED))
	var buttons := HBoxContainer.new()
	add_child(buttons)
	buttons.add_child(MainMenuView.button("Use default", LevelUpView._redraw.bind(host, func() -> Array[String]:
		m.use_default()
		return [] as Array[String]), not m.using_default))
	buttons.add_child(MainMenuView.button("Confirm", LevelUpView._redraw.bind(host, m.confirm), m.has_draft()))
	buttons.add_child(MainMenuView.button("Cancel", LevelUpView._redraw.bind(host, func() -> Array[String]:
		m.cancel()
		return [] as Array[String]), m.has_draft()))
	buttons.add_child(MainMenuView.button("Back", host.router.pop))

# One owned family row (LoadoutModel.families row): "Name  [Grade]  T1, T2 (Lv 5)" (locked tiers show their unlock level).
static func family_line(f: Dictionary) -> String:
	var tiers: PackedStringArray = PackedStringArray()
	for t: Variant in DictUtil.array_of(f["tiers"]):
		var td: Dictionary = DictUtil.dict_of(t)
		tiers.append("T%d" % DictUtil.to_int(td["tier"]) if DictUtil.to_bool(td["castable"]) else "T%d (Lv %d)" % [DictUtil.to_int(td["tier"]), DictUtil.to_int(td["unlock_level"])])
	return "%s  [%s]  %s" % [str(f["name"]), str(f["grade"]), ", ".join(tiers)]

# The basics line under the slots.
static func basics_line(names: Array[String]) -> String:
	return "Basics (always active): " + ", ".join(PackedStringArray(names))
