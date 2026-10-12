class_name DojoGearPanel
extends DojoPanel

# ==============================================================================
# Tactical RPG - Dojo Component 2: Unit Gear Viewer
# Equipped slots, battle pouch and one-click Quick Setup loadouts. Equip pickers
# expand inline under the slot (no floating dialogs), filtered to gear the active
# hero's class can actually wield.
# ==============================================================================

signal stash_requested()

const SLOT_DEFS: Array[Dictionary] = [
	{"slot": Equipment.EquipSlot.MAIN_HAND, "icon": "⚔"},
	{"slot": Equipment.EquipSlot.OFF_HAND, "icon": "🛡"},
	{"slot": Equipment.EquipSlot.BODY, "icon": "🥋"},
	{"slot": Equipment.EquipSlot.ACCESSORY_1, "icon": "💍"},
	{"slot": Equipment.EquipSlot.ACCESSORY_2, "icon": "💍"},
]

# Hide the Quick Setup card where spawning gear makes no sense (e.g. the real Party & Stash screen).
var show_quick_setup: bool = true:
	set(value):
		show_quick_setup = value
		if _quick_card != null:
			_quick_card.visible = value

var _quick_card: Control
var _preset_option: OptionButton
var _preset_desc: Label
var _slots_vbox: VBoxContainer
var _pouch_vbox: VBoxContainer
var _open_slot: int = -1
var _open_pouch_index: int = -1

func _init() -> void:
	super._init()
	panel_id = "gear"
	panel_title = "Unit Gear"
	panel_short_title = "Gear"
	panel_icon = "🎒"
	default_min_size = Vector2(250, 200)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	add_child(scroll)

	var vbox := VBoxContainer.new()
	vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	vbox.add_theme_constant_override("separation", 6)
	scroll.add_child(vbox)

	_build_quick_setup(vbox)

	vbox.add_child(DojoTheme.header("EQUIPPED GEAR"))
	_slots_vbox = VBoxContainer.new()
	_slots_vbox.add_theme_constant_override("separation", 4)
	vbox.add_child(_slots_vbox)

	vbox.add_child(DojoTheme.header("🧪 BATTLE POUCH (3 quick-slots)", Color(0.5, 0.85, 0.65)))
	_pouch_vbox = VBoxContainer.new()
	_pouch_vbox.add_theme_constant_override("separation", 3)
	vbox.add_child(_pouch_vbox)

func _build_quick_setup(parent: Control) -> void:
	var card := DojoTheme.card(parent, Color(0.10, 0.16, 0.22, 0.95))
	_quick_card = card.get_parent() as Control
	_quick_card.visible = show_quick_setup
	card.add_child(DojoTheme.header("⚡ QUICK SETUP", DojoTheme.COL_TITLE))

	var pick_row := HBoxContainer.new()
	pick_row.add_theme_constant_override("separation", 4)
	card.add_child(pick_row)

	_preset_option = OptionButton.new()
	_preset_option.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_preset_option.fit_to_longest_item = false
	_preset_option.clip_text = true
	_preset_option.custom_minimum_size = Vector2(60, 0)
	_preset_option.focus_mode = Control.FOCUS_NONE
	_preset_option.add_theme_font_size_override("font_size", DojoTheme.FONT_SM)
	for preset: Dictionary in DojoLoadouts.get_presets():
		_preset_option.add_item(str(preset.get("name", "Preset")))
		_preset_option.set_item_metadata(_preset_option.item_count - 1, str(preset.get("id", "")))
	_preset_option.item_selected.connect(func(_idx: int) -> void: _update_preset_desc())
	pick_row.add_child(_preset_option)
	pick_row.add_child(DojoTheme.button("Apply", "Equip the selected loadout on the active hero", _on_apply_pressed))

	_preset_desc = DojoTheme.label("", 9, DojoTheme.COL_MUTED)
	_preset_desc.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	card.add_child(_preset_desc)

	var util_row := HBoxContainer.new()
	util_row.add_theme_constant_override("separation", 4)
	card.add_child(util_row)
	util_row.add_child(DojoTheme.button("Clear All Gear", "Return every equipped item and pouch consumable to the stash", func() -> void: session.clear_all_gear()))
	util_row.add_child(DojoTheme.button("📦 Stash Manager", "Open the full stash & character equipment screen", func() -> void: stash_requested.emit()))
	_update_preset_desc()

func _selected_preset_id() -> String:
	if _preset_option.selected < 0:
		return ""
	var meta: Variant = _preset_option.get_item_metadata(_preset_option.selected)
	return meta if meta is String else ""

func _update_preset_desc() -> void:
	var preset: Dictionary = DojoLoadouts.find_preset(_selected_preset_id())
	_preset_desc.text = str(preset.get("desc", ""))

func _on_apply_pressed() -> void:
	_open_slot = -1
	_open_pouch_index = -1
	session.apply_loadout(_selected_preset_id())

func refresh() -> void:
	if session == null or session.active_hero == null or session.active_hero.inventory == null:
		return
	var inv: Inventory = session.active_hero.inventory
	var st: CharacterStats = session.active_hero.stats

	DojoTheme.clear_children(_slots_vbox)
	for def: Dictionary in SLOT_DEFS:
		var slot_raw: Variant = def.get("slot", Equipment.EquipSlot.MAIN_HAND)
		var slot_val: Equipment.EquipSlot = slot_raw if slot_raw is Equipment.EquipSlot else Equipment.EquipSlot.MAIN_HAND
		_build_slot_card(slot_val, str(def.get("icon", "")), inv.get_equipped(slot_val), st)

	_render_pouch(inv)

func _build_slot_card(slot_val: Equipment.EquipSlot, icon: String, eq: Equipment, st: CharacterStats) -> void:
	var card := DojoTheme.card(_slots_vbox)

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 4)
	card.add_child(head)
	head.add_child(DojoTheme.label("%s %s" % [icon, Equipment.slot_display_name(slot_val)], DojoTheme.FONT_SM, Color(0.65, 0.8, 1.0)))
	var inv: Inventory = session.active_hero.inventory
	if eq != null and inv.needs_weapon_switch() and (slot_val == Equipment.EquipSlot.MAIN_HAND or slot_val == Equipment.EquipSlot.OFF_HAND):
		var is_ready: bool = inv.is_offhand_active() == (slot_val == Equipment.EquipSlot.OFF_HAND)
		head.add_child(DojoTheme.label("● ACTIVE" if is_ready else "○ sheathed", 9, DojoTheme.COL_GOOD if is_ready else DojoTheme.COL_MUTED))
	head.add_spacer(false)
	var is_open: bool = _open_slot == int(slot_val)
	head.add_child(DojoTheme.button("Close" if is_open else ("Swap" if eq != null else "Equip"), "Pick gear for this slot from the stash", func() -> void: _toggle_picker(slot_val), 9))
	if eq != null:
		head.add_child(DojoTheme.button("Unequip", "Return to stash", func() -> void: _on_unequip(slot_val), 9))

	if eq == null:
		card.add_child(DojoTheme.label("— Empty Slot —", DojoTheme.FONT_SM, DojoTheme.COL_MUTED))
	else:
		card.add_child(DojoTheme.label(eq.name, DojoTheme.FONT_MD, DojoTheme.rarity_color(eq.rarity)))
		var summary: Label = DojoTheme.label(DojoTheme.equipment_summary(eq, st), 9, Color(0.8, 0.9, 0.9))
		summary.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		card.add_child(summary)
		var status: Dictionary = eq.get_requirement_status_summary(st, st.mastery)
		var badge_text: String = "%s  %s" % [str(status.get("req_display", "")), str(status.get("status_tag", ""))]
		var badge_col: Color = status.get("tag_color", Color.WHITE)
		if not badge_text.strip_edges().is_empty():
			var badge: Label = DojoTheme.label(badge_text.strip_edges(), 9, badge_col)
			badge.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
			card.add_child(badge)

	if is_open:
		_build_slot_picker(card, slot_val)

func _build_slot_picker(card: VBoxContainer, slot_val: Equipment.EquipSlot) -> void:
	card.add_child(HSeparator.new())
	var items: Array[Equipment] = session.get_eligible_stash_items(slot_val)
	if items.is_empty():
		card.add_child(DojoTheme.label("No eligible items in stash for this slot.", DojoTheme.FONT_SM, DojoTheme.COL_MUTED))
		return
	for item: Equipment in items:
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 4)
		card.add_child(row)

		var info := VBoxContainer.new()
		info.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		info.add_theme_constant_override("separation", 0)
		row.add_child(info)
		info.add_child(DojoTheme.label(item.name, DojoTheme.FONT_SM, DojoTheme.rarity_color(item.rarity)))
		var stats_lbl: Label = DojoTheme.label(DojoTheme.equipment_summary(item, session.active_hero.stats), 9, DojoTheme.COL_MUTED)
		stats_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		info.add_child(stats_lbl)

		row.add_child(DojoTheme.button("Equip", "Equip %s" % item.name, func() -> void: _on_pick(item, slot_val), 9))

func _toggle_picker(slot_val: Equipment.EquipSlot) -> void:
	_open_pouch_index = -1
	_open_slot = -1 if _open_slot == int(slot_val) else int(slot_val)
	refresh()

func _on_pick(item: Equipment, slot_val: Equipment.EquipSlot) -> void:
	_open_slot = -1
	if not session.equip_item(item, slot_val):
		refresh()

func _on_unequip(slot_val: Equipment.EquipSlot) -> void:
	_open_slot = -1
	session.unequip_slot(slot_val)

func _render_pouch(inv: Inventory) -> void:
	DojoTheme.clear_children(_pouch_vbox)
	for i: int in range(inv.max_pouch_slots):
		var con: Consumable = inv.battle_pouch[i] if i < inv.battle_pouch.size() else null
		var card := DojoTheme.card(_pouch_vbox, Color(0.09, 0.13, 0.17, 0.9))

		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 4)
		card.add_child(row)
		row.add_child(DojoTheme.label("Slot %d" % (i + 1), DojoTheme.FONT_SM, Color(0.5, 0.7, 0.9)))
		if con == null:
			row.add_child(DojoTheme.label("— empty —", DojoTheme.FONT_SM, DojoTheme.COL_MUTED))
			row.add_spacer(false)
			var is_open: bool = _open_pouch_index == i
			row.add_child(DojoTheme.button("Close" if is_open else "Load", "Load a consumable from the stash", func() -> void: _toggle_pouch_picker(i), 9))
			if is_open:
				_build_pouch_picker(card)
		else:
			row.add_child(DojoTheme.label("%s %s" % [DojoTheme.consumable_icon(con), con.name], DojoTheme.FONT_SM, DojoTheme.COL_GOOD))
			row.add_spacer(false)
			row.add_child(DojoTheme.label(DojoTheme.consumable_effect_text(con), 9, DojoTheme.COL_WARN))
			row.add_child(DojoTheme.button("✕", "Return to stash", func() -> void: session.remove_from_pouch(i), 9))

func _toggle_pouch_picker(index: int) -> void:
	_open_slot = -1
	_open_pouch_index = -1 if _open_pouch_index == index else index
	refresh()

func _build_pouch_picker(card: VBoxContainer) -> void:
	var options: Array[Consumable] = session.get_stash_consumables()
	if options.is_empty():
		card.add_child(DojoTheme.label("No consumables in stash.", DojoTheme.FONT_SM, DojoTheme.COL_MUTED))
		return
	for con: Consumable in options:
		var row := HBoxContainer.new()
		card.add_child(row)
		row.add_child(DojoTheme.label("%s %s" % [DojoTheme.consumable_icon(con), con.name], DojoTheme.FONT_SM, DojoTheme.COL_GOOD))
		row.add_spacer(false)
		row.add_child(DojoTheme.label(DojoTheme.consumable_effect_text(con), 9, DojoTheme.COL_WARN))
		row.add_child(DojoTheme.button("Load", "Load into the battle pouch", func() -> void: _on_pouch_pick(con), 9))

func _on_pouch_pick(con: Consumable) -> void:
	_open_pouch_index = -1
	if not session.add_to_pouch(con):
		refresh()
