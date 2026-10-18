class_name LoadoutView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Loadout view (UI2-L rule 5; UIR-3): the class (5) and extra (3) slots as chips, the owned families with
# their grade, only the ACTIVE rank and its description, and the next locked rank's unlock level; the basics (always
# active). parts() is the Skills tab of the Character popup; build() shows the same body alone (battle results).
# Renders LoadoutModel.
# ==============================================================================

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: LoadoutModel = model as LoadoutModel
	var member: UnitState = PartyPreview.member(m.member_id)
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	size_flags_vertical = Control.SIZE_EXPAND_FILL
	var detail: Label = CharacterView.new_detail("Move to a skill to see its next rank.")
	CharacterView.adopt(self, CharacterView.frame(host, "Skills: %s" % (PartyPreview.name_of(member) if member != null else m.member_id), parts(host, m, detail), detail))

# {body, extras, primary} for the bar: Reset and Use default beside the primary Confirm.
static func parts(host: MainMenuView, m: LoadoutModel, detail: Label) -> Dictionary:
	var body := VBoxContainer.new()
	body.name = "SkillsBody"
	body.add_theme_constant_override("separation", 8)
	body.add_child(MainMenuView.text("Default loadout (picked by the class)" if m.using_default else "Custom loadout", UiTheme.COL_MUTED, false))
	var fams: Dictionary = m.families()
	for group: String in ["class", "extra"]:
		var list: Array[String] = m.class_slots if group == "class" else m.extra_slots
		var head: Label = MainMenuView.text("%s slots %d / %d" % [group.capitalize(), list.size(), m.limit(group)], UiTheme.COL_HEADER, false)
		head.name = "SlotsHead_" + group
		body.add_child(head)
		var slots := HFlowContainer.new()
		slots.name = "Chips_" + group
		body.add_child(slots)
		for i: int in range(list.size()):
			var chip: Button = MainMenuView.small(MainMenuView.button(SkillCatalog.family_name(list[i]) + "  ✕", LevelUpView._redraw.bind(host, m.unslot.bind(group, i))), 160)
			CharacterView.show_on_focus(detail, chip, "Take %s out of its slot." % SkillCatalog.family_name(list[i]))
			slots.add_child(chip)
		var rows: Array = DictUtil.array_of(fams.get(group, []))
		if rows.is_empty() and group == "extra":
			body.add_child(MainMenuView.text("No extra skills learned yet.", UiTheme.COL_MUTED, false))
		for raw: Variant in rows:
			body.add_child(_family_row(host, m, DictUtil.dict_of(raw), group, list, detail))
	var basics: Array[String] = []
	for fam: String in m.basics():
		basics.append(SkillCatalog.family_name(fam))
	body.add_child(MainMenuView.text(basics_line(basics), UiTheme.COL_MUTED))
	var reset: Button = MainMenuView.button("Reset", LevelUpView._redraw.bind(host, func() -> Array[String]:
		m.cancel()
		return [] as Array[String]), m.has_draft())
	reset.name = "ResetButton"
	var default_btn: Button = MainMenuView.button("Use default", LevelUpView._redraw.bind(host, func() -> Array[String]:
		m.use_default()
		return [] as Array[String]), not m.using_default)
	default_btn.name = "DefaultButton"
	var confirm: Button = MainMenuView.button("Confirm", LevelUpView._redraw.bind(host, m.confirm), m.has_draft())
	return {"body": body, "extras": [reset, default_btn] as Array[Button], "primary": confirm}

# One owned family: [Slotted / Add / Full] then "Name [Grade] Rank II" and the active rank's description under it.
static func _family_row(host: MainMenuView, m: LoadoutModel, f: Dictionary, group: String, list: Array[String], detail: Label) -> Control:
	var fam: String = str(f["family"])
	var line := HBoxContainer.new()
	line.name = "Family_" + fam
	line.add_theme_constant_override("separation", 12)
	var slotted: bool = list.has(fam)
	var full: bool = list.size() >= m.limit(group)
	var state: String = "Slotted" if slotted else ("Full" if full else "Add")
	var b: Button = MainMenuView.small(MainMenuView.button(state, LevelUpView._redraw.bind(host, m.slot.bind(fam, group, list.size())), not slotted and not full))
	b.name = "Slot_" + fam
	b.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
	CharacterView.show_on_focus(detail, b, next_rank_line(f))
	line.add_child(b)
	var text := VBoxContainer.new()
	text.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	line.add_child(text)
	text.add_child(MainMenuView.text(rank_line(f), UiTheme.COL_TEXT, false))
	var desc: Label = MainMenuView.text(str(f.get("description", "")), UiTheme.COL_MUTED)
	desc.name = "Desc_" + fam
	text.add_child(desc)
	return line

# "Name  [Grade]  Rank II": only the active rank (the family name when nothing is castable yet).
static func rank_line(f: Dictionary) -> String:
	var tier: int = DictUtil.to_int(f.get("active_tier", 0))
	return "%s  [%s]  %s" % [str(f["name"]), str(f["grade"]), "Rank " + SkillCatalog.roman(tier) if tier > 0 else "locked"]

# The detail pane text of a family: when its next rank arrives.
static func next_rank_line(f: Dictionary) -> String:
	var next: int = DictUtil.to_int(f.get("next_unlock", 0))
	return "%s: next rank at Lv %d." % [str(f["name"]), next] if next > 0 else "%s: top rank reached." % str(f["name"])

# One owned family row (LoadoutModel.families row): "Name  [Grade]  T1, T2 (Lv 5)" (locked tiers show their unlock level).
# Kept for the skill showcase (the forge preview), which lists every tier.
static func family_line(f: Dictionary) -> String:
	var tiers: PackedStringArray = PackedStringArray()
	for t: Variant in DictUtil.array_of(f["tiers"]):
		var td: Dictionary = DictUtil.dict_of(t)
		tiers.append("T%d" % DictUtil.to_int(td["tier"]) if DictUtil.to_bool(td["castable"]) else "T%d (Lv %d)" % [DictUtil.to_int(td["tier"]), DictUtil.to_int(td["unlock_level"])])
	return "%s  [%s]  %s" % [str(f["name"]), str(f["grade"]), ", ".join(tiers)]

# The basics line under the slots.
static func basics_line(names: Array[String]) -> String:
	return "Basics (always active): " + ", ".join(PackedStringArray(names))
