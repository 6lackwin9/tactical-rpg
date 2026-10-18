---
class: GearView
extends: VBoxContainer
description: "Tactical RPG - Gear view (UI2-L rule 6): the member's six slots (class defaults marked), the stash with one button per slot an item fits, and the full gear board (CharacterEquipmentModal through PartyGearBridge). Renders GearModel."
---

# Methods

### `build(host: MainMenuView, model: RefCounted) -> void`
```logic
var m: GearModel = model as GearModel
var member: UnitState = PartyPreview.member(m.member_id)
add_theme_constant_override("separation", 8)
add_child(MainMenuView.title("Gear: %s" % (PartyPreview.name_of(member) if member != null else m.member_id)))
var scroll := ScrollContainer.new()   # ITEMUSE1-U: the pouch section can outgrow a large text size; Confirm/Back stay below
scroll.name = "GearScroll"
scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
scroll.follow_focus = true
add_child(scroll)
var body := VBoxContainer.new()
body.name = "GearBody"
body.add_theme_constant_override("separation", 12)
body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
scroll.add_child(body)
var row := HBoxContainer.new()
row.add_theme_constant_override("separation", 32)
body.add_child(row)
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
body.add_child(_pouch_section(host, m))
var buttons := HBoxContainer.new()
add_child(buttons)
buttons.add_child(MainMenuView.button("Gear board", _open_board.bind(host, m)))
buttons.add_child(MainMenuView.button("Confirm", LevelUpView._redraw.bind(host, m.confirm), m.has_draft()))
buttons.add_child(MainMenuView.button("Cancel", LevelUpView._redraw.bind(host, func() -> Array[String]:
	m.cancel()
	return [] as Array[String]), m.has_draft()))
buttons.add_child(MainMenuView.button("Back", host.router.pop))
```

### `static _pouch_section(host: MainMenuView, m: GearModel) -> Control`
```logic
var box := VBoxContainer.new()
box.name = "PouchSection"
box.add_theme_constant_override("separation", 4)
box.add_child(MainMenuView.text("Battle pouch", UiTheme.COL_HEADER, false))
var held: Array[Dictionary] = m.pouch_rows()
for i: int in range(UnitState.POUCH_MAX):
	var line := HBoxContainer.new()
	line.name = "PouchSlot%d" % i
	box.add_child(line)
	if i < held.size():
		line.add_child(MainMenuView.text("%d. %s" % [i + 1, str(held[i]["name"])], UiTheme.COL_TEXT, false))
		var remove: Button = MainMenuView.small(MainMenuView.button("Remove", LevelUpView._redraw.bind(host, m.pouch_remove.bind(i))))
		remove.name = "PouchRemove%d" % i
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
	box.add_child(line)
	line.add_child(MainMenuView.text("%s x%d" % [str(c["name"]), DictUtil.to_int(c["count"])], UiTheme.COL_TEXT, false))
	var add: Button = MainMenuView.small(MainMenuView.button("Add", LevelUpView._redraw.bind(host, m.pouch_add.bind(str(c["item"]))), not full))
	add.name = "PouchAdd_%s" % str(c["item"])
	line.add_child(add)
return box
```

### `static _open_board(host: MainMenuView, m: GearModel) -> void`
```logic
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
```

