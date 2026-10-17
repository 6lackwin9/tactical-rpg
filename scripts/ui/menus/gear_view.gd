class_name GearView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Gear view (UI2-L rule 6): the member's six slots (class defaults marked), the stash with one button
# per slot an item fits, and the full gear board (CharacterEquipmentModal through PartyGearBridge). Renders GearModel.
# ==============================================================================

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: GearModel = model as GearModel
	var member: UnitState = PartyPreview.member(m.member_id)
	add_theme_constant_override("separation", 8)
	add_child(MainMenuView.title("Gear: %s" % (PartyPreview.name_of(member) if member != null else m.member_id)))
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 32)
	add_child(row)
	var slots := VBoxContainer.new()
	slots.custom_minimum_size = Vector2(420, 0)
	row.add_child(slots)
	for s: Dictionary in m.slots():
		var slot: String = str(s["slot"])
		var line := HBoxContainer.new()
		slots.add_child(line)
		var shown: String = str(s["name"]) if not str(s["name"]).is_empty() else "-"
		line.add_child(MainMenuView.text("%s: %s%s" % [slot.replace("_", " ").capitalize(), shown, "" if DictUtil.to_bool(s["explicit"]) else "  (default)"], UiTheme.COL_TEXT, false))
		if DictUtil.to_bool(s["explicit"]):
			line.add_child(MainMenuView.small(MainMenuView.button("Unequip", LevelUpView._redraw.bind(host, m.unequip.bind(slot)))))
	var stash := VBoxContainer.new()
	row.add_child(stash)
	stash.add_child(MainMenuView.text("Stash", UiTheme.COL_HEADER, false))
	var rows: Array[Dictionary] = m.stash_rows()
	if rows.is_empty():
		stash.add_child(MainMenuView.text("Empty", UiTheme.COL_MUTED, false))
	for r: Dictionary in rows:
		var line := HBoxContainer.new()
		stash.add_child(line)
		line.add_child(MainMenuView.text("%s x%d" % [str(r["name"]), DictUtil.to_int(r["count"])], UiTheme.COL_TEXT, false))
		for slot: String in JsonUtil.strings(r["slots"]):
			line.add_child(MainMenuView.small(MainMenuView.button("→ " + slot.replace("_", " "), LevelUpView._redraw.bind(host, m.equip.bind(slot, str(r["item"]))))))
	var buttons := HBoxContainer.new()
	add_child(buttons)
	buttons.add_child(MainMenuView.button("Gear board", _open_board.bind(host, m)))
	buttons.add_child(MainMenuView.button("Confirm", LevelUpView._redraw.bind(host, m.confirm), m.has_draft()))
	buttons.add_child(MainMenuView.button("Cancel", LevelUpView._redraw.bind(host, func() -> Array[String]:
		m.cancel()
		return [] as Array[String]), m.has_draft()))
	buttons.add_child(MainMenuView.button("Back", host.router.pop))

# The drag-and-drop board on the same draft; closing it re-renders the list.
static func _open_board(host: MainMenuView, m: GearModel) -> void:
	var bridge := PartyGearBridge.new(m)
	if bridge.hero == null:
		return
	var modal := CharacterEquipmentModal.new()
	host.add_child(modal)
	bridge.attach(modal)
	modal.modal_closed.connect(func() -> void:
		host.show_problems(bridge.last_errors)
		modal.queue_free()
		host.render())
	modal.open_modal(bridge.hero)
	modal.set_meta(&"bridge", bridge)
