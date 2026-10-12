class_name DojoPickerPanel
extends DojoPanel

# ==============================================================================
# Tactical RPG - Equip / Pouch Picker (overlay content)
# Opened as an overlay from the Unit Gear panel for one gear slot or a pouch
# slot. Lists only what the active hero's class can use in that slot. Closes
# itself after a pick.
# ==============================================================================

var _slot: Equipment.EquipSlot = Equipment.EquipSlot.MAIN_HAND
var _pouch_mode: bool = false
var _current_lbl: Label
var _list_vbox: VBoxContainer

static func for_slot(slot: Equipment.EquipSlot) -> DojoPickerPanel:
	var picker := DojoPickerPanel.new()
	picker._slot = slot
	picker.panel_title = "Equip: %s" % Equipment.slot_display_name(slot)
	return picker

static func for_pouch() -> DojoPickerPanel:
	var picker := DojoPickerPanel.new()
	picker._pouch_mode = true
	picker.panel_title = "Load Battle Pouch"
	return picker

func _init() -> void:
	super._init()
	panel_id = "picker"
	panel_short_title = "Picker"
	panel_icon = "🎒"
	default_min_size = Vector2(380, 380)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 6)
	add_child(vbox)

	_current_lbl = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_MUTED)
	_current_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	vbox.add_child(_current_lbl)

	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	vbox.add_child(scroll)

	_list_vbox = VBoxContainer.new()
	_list_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_list_vbox.add_theme_constant_override("separation", 4)
	scroll.add_child(_list_vbox)

func refresh() -> void:
	if session == null or session.active_hero == null:
		return
	UiTheme.clear_children(_list_vbox)
	if _pouch_mode:
		_refresh_pouch()
	else:
		_refresh_slot()

func _refresh_slot() -> void:
	var hero: Combatant = session.active_hero
	var st: CharacterStats = hero.stats
	var current: Equipment = hero.inventory.get_equipped(_slot)
	_current_lbl.text = "%s equips: %s" % [hero.name, current.name if current != null else "— empty —"]

	var items: Array[Equipment] = session.get_eligible_stash_items(_slot)
	if items.is_empty():
		_list_vbox.add_child(UiTheme.label("No eligible items in the stash for this slot.", UiTheme.FONT_SM, UiTheme.COL_MUTED))
		return
	for item: Equipment in items:
		var card := UiTheme.card(_list_vbox)
		var head := HBoxContainer.new()
		var info: VBoxContainer = UiTheme.item_row(card, item)
		info.add_child(head)
		head.add_child(UiTheme.label(item.name, UiTheme.FONT_MD, UiTheme.rarity_color(item.rarity)))
		head.add_spacer(false)
		head.add_child(UiTheme.button("Equip", "Equip %s" % item.name, func() -> void: _pick(item), 10))

		var summary: Label = UiTheme.label(UiTheme.equipment_summary(item, st), 9, Color(0.8, 0.9, 0.9))
		summary.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		info.add_child(summary)
		var status: Dictionary = item.get_requirement_status_summary(st, st.mastery)
		var badge_text: String = ("%s  %s" % [str(status.get("req_display", "")), str(status.get("status_tag", ""))]).strip_edges()
		if not badge_text.is_empty():
			var badge_col: Color = status.get("tag_color", Color.WHITE)
			var badge: Label = UiTheme.label(badge_text, 9, badge_col)
			badge.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
			info.add_child(badge)

func _refresh_pouch() -> void:
	var inv: Inventory = session.active_hero.inventory
	_current_lbl.text = "Battle pouch: %d / %d slots used" % [inv.battle_pouch.size(), inv.max_pouch_slots]
	var options: Array[Consumable] = session.get_stash_consumables()
	if options.is_empty():
		_list_vbox.add_child(UiTheme.label("No consumables in the stash.", UiTheme.FONT_SM, UiTheme.COL_MUTED))
		return
	for con: Consumable in options:
		var card := UiTheme.card(_list_vbox)
		var head := HBoxContainer.new()
		var con_info: VBoxContainer = UiTheme.item_row(card, con, 28)
		con_info.add_child(head)
		head.add_child(UiTheme.label("%s %s" % [UiTheme.consumable_icon(con), con.name], UiTheme.FONT_MD, UiTheme.COL_GOOD))
		head.add_spacer(false)
		var load_btn: Button = UiTheme.button("Load", "Load into the battle pouch", func() -> void: _pick_pouch(con), 10)
		load_btn.disabled = inv.battle_pouch.size() >= inv.max_pouch_slots
		head.add_child(load_btn)
		con_info.add_child(UiTheme.label(UiTheme.consumable_effect_text(con), 9, UiTheme.COL_WARN))

func _pick(item: Equipment) -> void:
	if session.equip_item(item, _slot):
		overlay_close_requested.emit()

func _pick_pouch(con: Consumable) -> void:
	if session.add_to_pouch(con):
		overlay_close_requested.emit()
