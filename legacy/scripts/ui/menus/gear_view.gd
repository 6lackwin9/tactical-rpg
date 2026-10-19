class_name GearView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Gear view (UI2-L rule 6; UIR-3): the member's six slots (empty ones read "empty"), the stash with one
# button per slot an item fits, the battle pouch (ITEMUSE1-U) and the optional gear board (CharacterEquipmentModal through
# PartyGearBridge). parts() is the Gear tab of the Character popup; build() shows the same body alone. Renders GearModel.
# ==============================================================================

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: GearModel = model as GearModel
	var member: UnitState = PartyPreview.member(m.member_id)
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	size_flags_vertical = Control.SIZE_EXPAND_FILL
	var detail: Label = CharacterView.new_detail("Move to an item to read what it does.")
	CharacterView.adopt(self, CharacterView.frame(host, "Gear: %s" % (PartyPreview.name_of(member) if member != null else m.member_id), parts(host, m, detail), detail))

# {body, extras, primary} for the bar: Reset and the Gear board beside the primary Confirm.
static func parts(host: MainMenuView, m: GearModel, detail: Label) -> Dictionary:
	var body := VBoxContainer.new()
	body.name = "GearBody"
	body.add_theme_constant_override("separation", 12)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 32)
	body.add_child(row)
	var slots := VBoxContainer.new()
	slots.name = "GearSlots"
	slots.custom_minimum_size = Vector2(420, 0)
	row.add_child(slots)
	for s: Dictionary in m.slots():
		var slot: String = str(s["slot"])
		var line := HBoxContainer.new()
		line.add_theme_constant_override("separation", 12)
		slots.add_child(line)
		var explicit: bool = DictUtil.to_bool(s["explicit"])
		var shown: String = str(s["name"]) if not str(s["name"]).is_empty() else "empty"
		var label: Label = MainMenuView.text("%s: %s" % [_slot_label(slot), shown], UiTheme.COL_TEXT if explicit else UiTheme.COL_MUTED, false)
		label.custom_minimum_size = Vector2(300, 0)
		line.add_child(label)
		if explicit:
			var off: Button = MainMenuView.small(MainMenuView.button("Unequip", LevelUpView._redraw.bind(host, m.unequip.bind(slot))))
			CharacterView.show_on_focus(detail, off, "Unequip %s: it returns to the stash and the class default takes the slot." % shown)
			line.add_child(off)
	var stash := VBoxContainer.new()
	stash.name = "GearStash"
	row.add_child(stash)
	stash.add_child(MainMenuView.text("Stash", UiTheme.COL_HEADER, false))
	var rows: Array[Dictionary] = m.stash_rows()
	if rows.is_empty():
		stash.add_child(MainMenuView.text("Empty", UiTheme.COL_MUTED, false))
	for r: Dictionary in rows:
		var line := HBoxContainer.new()
		line.add_theme_constant_override("separation", 8)
		stash.add_child(line)
		line.add_child(MainMenuView.text("%s x%d" % [str(r["name"]), DictUtil.to_int(r["count"])], UiTheme.COL_TEXT, false))
		for slot: String in JsonUtil.strings(r["slots"]):
			var put: Button = MainMenuView.small(MainMenuView.button("→ " + _slot_label(slot), LevelUpView._redraw.bind(host, m.equip.bind(slot, str(r["item"])))))
			CharacterView.show_on_focus(detail, put, "Equip %s in the %s slot; the item it replaces returns to the stash." % [str(r["name"]), _slot_label(slot).to_lower()])
			line.add_child(put)
	body.add_child(_pouch_section(host, m, detail))
	var reset: Button = MainMenuView.button("Reset", LevelUpView._redraw.bind(host, func() -> Array[String]:
		m.cancel()
		return [] as Array[String]), m.has_draft())
	reset.name = "ResetButton"
	var board: Button = MainMenuView.button("Gear board", _open_board.bind(host, m))
	board.name = "BoardButton"
	CharacterView.show_on_focus(detail, board, "The drag-and-drop gear board, on the same draft.")
	var confirm: Button = MainMenuView.button("Confirm", LevelUpView._redraw.bind(host, m.confirm), m.has_draft())
	return {"body": body, "extras": [reset, board] as Array[Button], "primary": confirm}

static func _slot_label(slot: String) -> String:
	return slot.replace("_", " ").capitalize()

# ITEMUSE1-U: the battle pouch (UnitState.POUCH_MAX slots) and the stash consumables that can go in it.
static func _pouch_section(host: MainMenuView, m: GearModel, detail: Label) -> Control:
	var box := VBoxContainer.new()
	box.name = "PouchSection"
	box.add_theme_constant_override("separation", 4)
	box.add_child(MainMenuView.text("Battle pouch", UiTheme.COL_HEADER, false))
	var held: Array[Dictionary] = m.pouch_rows()
	for i: int in range(UnitState.POUCH_MAX):
		var line := HBoxContainer.new()
		line.name = "PouchSlot%d" % i
		line.add_theme_constant_override("separation", 12)
		box.add_child(line)
		if i < held.size():
			line.add_child(MainMenuView.text("%d. %s" % [i + 1, str(held[i]["name"])], UiTheme.COL_TEXT, false))
			var remove: Button = MainMenuView.small(MainMenuView.button("Remove", LevelUpView._redraw.bind(host, m.pouch_remove.bind(i))))
			remove.name = "PouchRemove%d" % i
			CharacterView.show_on_focus(detail, remove, "Take %s out of the battle pouch; it returns to the stash." % str(held[i]["name"]))
			line.add_child(remove)
		else:
			line.add_child(MainMenuView.text("%d. Empty" % (i + 1), UiTheme.COL_MUTED, false))
	var full: bool = held.size() >= UnitState.POUCH_MAX
	var candidates: Array[Dictionary] = m.pouch_candidates()
	if candidates.is_empty():
		box.add_child(MainMenuView.text("No usable consumables in the stash.", UiTheme.COL_MUTED, false))
	for c: Dictionary in candidates:
		var line := HBoxContainer.new()
		line.name = "PouchCandidate_%s" % str(c["item"])
		line.add_theme_constant_override("separation", 12)
		box.add_child(line)
		line.add_child(MainMenuView.text("%s x%d" % [str(c["name"]), DictUtil.to_int(c["count"])], UiTheme.COL_TEXT, false))
		var add: Button = MainMenuView.small(MainMenuView.button("Add", LevelUpView._redraw.bind(host, m.pouch_add.bind(str(c["item"]))), not full))
		add.name = "PouchAdd_%s" % str(c["item"])
		CharacterView.show_on_focus(detail, add, "The pouch is full (%d)." % UnitState.POUCH_MAX if full else "Put %s in the battle pouch; it can be used in a fight." % str(c["name"]))
		line.add_child(add)
	return box

# The drag-and-drop board on the same draft; closing it re-renders the list.
static func _open_board(host: MainMenuView, m: GearModel) -> void:
	var bridge := PartyGearBridge.new(m)
	if bridge.hero == null:
		return
	var modal := CharacterEquipmentModal.new()
	host.modal_parent().add_child(modal)
	bridge.attach(modal)
	modal.modal_closed.connect(func() -> void:
		host.show_problems(bridge.last_errors)
		modal.queue_free()
		host.render())
	modal.open_modal(bridge.hero)
	modal.set_meta(&"bridge", bridge)
