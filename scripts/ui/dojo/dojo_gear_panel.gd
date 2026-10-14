class_name DojoGearPanel
extends DojoPanel

# ==============================================================================
# Tactical RPG - Dojo Component 2: Unit Gear Viewer
# Equipped slots, battle pouch and one-click Quick Setup loadouts. Equip / Load
# buttons open an overlay picker (DojoPickerPanel) filtered to gear the active
# hero's class can use, so this panel's layout stays static.
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

func _init() -> void:
	super._init()
	panel_id = "gear"
	panel_title = "Unit Gear"
	panel_short_title = "Gear"
	panel_icon = "🎒"
	default_min_size = Vector2(260, 220)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(scroll)

	var vbox := VBoxContainer.new()
	vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	vbox.add_theme_constant_override("separation", 6)
	scroll.add_child(vbox)

	_build_quick_setup(vbox)

	vbox.add_child(UiTheme.header("EQUIPPED GEAR"))
	_slots_vbox = VBoxContainer.new()
	_slots_vbox.add_theme_constant_override("separation", 4)
	vbox.add_child(_slots_vbox)

	vbox.add_child(UiTheme.header("🧪 BATTLE POUCH (3 quick-slots)", Color(0.5, 0.85, 0.65)))
	_pouch_vbox = VBoxContainer.new()
	_pouch_vbox.add_theme_constant_override("separation", 3)
	vbox.add_child(_pouch_vbox)

func _build_quick_setup(parent: Control) -> void:
	var card := UiTheme.card(parent, Color(0.10, 0.16, 0.22, 0.95))
	_quick_card = card.get_parent() as Control
	_quick_card.visible = show_quick_setup
	card.add_child(UiTheme.header("⚡ QUICK SETUP", UiTheme.COL_TITLE))

	var pick_row := HBoxContainer.new()
	pick_row.add_theme_constant_override("separation", 4)
	card.add_child(pick_row)

	_preset_option = OptionButton.new()
	_preset_option.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_preset_option.fit_to_longest_item = false
	_preset_option.clip_text = true
	_preset_option.custom_minimum_size = Vector2(60, 0)
	_preset_option.focus_mode = Control.FOCUS_NONE
	_preset_option.add_theme_font_size_override("font_size", UiTheme.FONT_SM)
	for preset: Dictionary in DojoLoadouts.get_presets():
		_preset_option.add_item(str(preset.get("name", "Preset")))
		_preset_option.set_item_metadata(_preset_option.item_count - 1, str(preset.get("id", "")))
	_preset_option.item_selected.connect(func(_idx: int) -> void: _update_preset_desc())
	pick_row.add_child(_preset_option)
	pick_row.add_child(UiTheme.button("Apply", "Equip the selected loadout on the active hero", _on_apply_pressed))

	_preset_desc = UiTheme.label("", 9, UiTheme.COL_MUTED)
	_preset_desc.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	card.add_child(_preset_desc)

	var util_row := HFlowContainer.new()
	util_row.add_theme_constant_override("h_separation", 4)
	util_row.add_theme_constant_override("v_separation", 3)
	card.add_child(util_row)
	util_row.add_child(UiTheme.button("Clear All Gear", "Return every equipped item and pouch consumable to the stash", func() -> void: session.clear_all_gear()))
	util_row.add_child(UiTheme.button("📦 Stash Manager", "Open the full Party & Stash screen", func() -> void: stash_requested.emit()))
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
	session.apply_loadout(_selected_preset_id())

func refresh() -> void:
	if session == null or session.active_hero == null or session.active_hero.inventory == null:
		return
	var inv: Inventory = session.active_hero.inventory
	var st: CharacterStats = session.active_hero.stats

	UiTheme.clear_children(_slots_vbox)
	for def: Dictionary in SLOT_DEFS:
		var slot_raw: Variant = def.get("slot", Equipment.EquipSlot.MAIN_HAND)
		var slot_val: Equipment.EquipSlot = slot_raw if slot_raw is Equipment.EquipSlot else Equipment.EquipSlot.MAIN_HAND
		_build_slot_card(slot_val, str(def.get("icon", "")), inv.get_equipped(slot_val), st)

	_render_pouch(inv)

func _build_slot_card(slot_val: Equipment.EquipSlot, icon: String, eq: Equipment, st: CharacterStats) -> void:
	var card := UiTheme.card(_slots_vbox)

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 4)
	card.add_child(head)
	head.add_child(UiTheme.label("%s %s" % [icon, Equipment.slot_display_name(slot_val)], UiTheme.FONT_SM, Color(0.65, 0.8, 1.0)))
	var inv: Inventory = session.active_hero.inventory
	if eq != null and inv.needs_weapon_switch() and (slot_val == Equipment.EquipSlot.MAIN_HAND or slot_val == Equipment.EquipSlot.OFF_HAND):
		var is_ready: bool = inv.is_offhand_active() == (slot_val == Equipment.EquipSlot.OFF_HAND)
		head.add_child(UiTheme.label("● ACTIVE" if is_ready else "○ sheathed", 9, UiTheme.COL_GOOD if is_ready else UiTheme.COL_MUTED))
	head.add_spacer(false)
	head.add_child(UiTheme.button("Swap" if eq != null else "Equip", "Pick gear for this slot from the stash", func() -> void: overlay_requested.emit(DojoPickerPanel.for_slot(slot_val)), 9))
	if eq != null:
		head.add_child(UiTheme.button("Unequip", "Return to stash", func() -> void: session.unequip_slot(slot_val), 9))

	if eq == null:
		card.add_child(UiTheme.label("— Empty Slot —", UiTheme.FONT_SM, UiTheme.COL_MUTED))
		return
	var info: VBoxContainer = UiTheme.item_row(card, eq)
	info.add_child(UiTheme.label(eq.name, UiTheme.FONT_MD, UiTheme.rarity_color(eq.rarity)))
	var summary: Label = UiTheme.label(UiTheme.equipment_summary(eq, st), 9, Color(0.8, 0.9, 0.9))
	summary.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	info.add_child(summary)
	var status: Dictionary = eq.get_requirement_status_summary(st, st.proficiency)
	var badge_text: String = ("%s  %s" % [str(status.get("req_display", "")), str(status.get("status_tag", ""))]).strip_edges()
	if not badge_text.is_empty():
		var badge_col: Color = status.get("tag_color", Color.WHITE)
		var badge: Label = UiTheme.label(badge_text, 9, badge_col)
		badge.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		info.add_child(badge)

func _render_pouch(inv: Inventory) -> void:
	UiTheme.clear_children(_pouch_vbox)
	for i: int in range(inv.max_pouch_slots):
		var con: Consumable = inv.battle_pouch[i] if i < inv.battle_pouch.size() else null
		var card := UiTheme.card(_pouch_vbox, Color(0.09, 0.13, 0.17, 0.9))

		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 4)
		card.add_child(row)
		row.add_child(UiTheme.label("Slot %d" % (i + 1), UiTheme.FONT_SM, Color(0.5, 0.7, 0.9)))
		if con == null:
			row.add_child(UiTheme.label("— empty —", UiTheme.FONT_SM, UiTheme.COL_MUTED))
			row.add_spacer(false)
			row.add_child(UiTheme.button("Load", "Load a consumable from the stash", func() -> void: overlay_requested.emit(DojoPickerPanel.for_pouch()), 9))
		else:
			row.add_child(UiTheme.icon_rect(ItemArt.icon(con), 20))
			row.add_child(UiTheme.label(con.name, UiTheme.FONT_SM, UiTheme.COL_GOOD))
			row.add_spacer(false)
			row.add_child(UiTheme.label(UiTheme.consumable_effect_text(con), 9, UiTheme.COL_WARN))
			row.add_child(UiTheme.button("✕", "Return to stash", func() -> void: session.remove_from_pouch(i), 9))
