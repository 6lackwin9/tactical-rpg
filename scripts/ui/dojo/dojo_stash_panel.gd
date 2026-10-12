class_name DojoStashPanel
extends DojoPanel

# ==============================================================================
# Tactical RPG - Party Stash (Party & Stash screen, column C)
# Shared party stash with category filters. Each item shows its stats and how the
# active unit would fare with it (class restriction, tier bonus, deficit), with
# one-click equip into a specific slot or into the battle pouch.
# ==============================================================================

const FILTERS: Array[String] = ["ALL", "WEAPON", "ARMOR", "ACCESSORY", "CONSUMABLE"]

var _filter: String = "ALL"
var _filter_btns: Dictionary = {} # String -> Button
var _summary_lbl: Label
var _items_vbox: VBoxContainer
var _status_lbl: Label

func _init() -> void:
	super._init()
	panel_id = "stash"
	panel_title = "Party Stash"
	panel_short_title = "Stash"
	panel_icon = "📦"
	default_min_size = Vector2(260, 220)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 6)
	add_child(vbox)

	_summary_lbl = DojoTheme.label("", DojoTheme.FONT_SM, DojoTheme.COL_MUTED)
	vbox.add_child(_summary_lbl)

	var filters := HFlowContainer.new()
	filters.add_theme_constant_override("h_separation", 3)
	filters.add_theme_constant_override("v_separation", 3)
	vbox.add_child(filters)
	for cat: String in FILTERS:
		var btn: Button = DojoTheme.button(cat.capitalize(), "Show %s items" % cat.to_lower(), func() -> void: _set_filter(cat), 9)
		btn.toggle_mode = true
		filters.add_child(btn)
		_filter_btns[cat] = btn

	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	vbox.add_child(scroll)

	_items_vbox = VBoxContainer.new()
	_items_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_items_vbox.add_theme_constant_override("separation", 4)
	scroll.add_child(_items_vbox)

	_status_lbl = DojoTheme.label("", 9, DojoTheme.COL_MUTED)
	_status_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	vbox.add_child(_status_lbl)
	_update_filter_buttons()

func _on_bound() -> void:
	session.entry_logged.connect(_on_entry_logged)

func _on_entry_logged(entry: DojoLogEntry) -> void:
	if entry.category != "EVENT" and entry.category != "BUFF":
		return
	var failed: bool = entry.outcome_tag == "FAILED"
	_status_lbl.text = "%s %s" % ["⚠" if failed else "✔", entry.detail_text]
	_status_lbl.add_theme_color_override("font_color", DojoTheme.COL_BAD if failed else DojoTheme.COL_GOOD)

func _set_filter(category: String) -> void:
	_filter = category
	_update_filter_buttons()
	refresh()

func _update_filter_buttons() -> void:
	for cat: String in _filter_btns:
		var btn_raw: Variant = _filter_btns[cat]
		if btn_raw is Button:
			var btn: Button = btn_raw
			btn.set_pressed_no_signal(cat == _filter)

func _matches_filter(item: Item) -> bool:
	match _filter:
		"WEAPON":
			return item.item_type == Item.ItemType.WEAPON
		"ARMOR":
			return item.item_type == Item.ItemType.ARMOR or item.item_type == Item.ItemType.SHIELD
		"ACCESSORY":
			return item.item_type == Item.ItemType.ACCESSORY
		"CONSUMABLE":
			return item.item_type == Item.ItemType.CONSUMABLE
	return true

func refresh() -> void:
	if session == null or session.shared_stash == null:
		return
	DojoTheme.clear_children(_items_vbox)

	var stash: Inventory = session.shared_stash
	var shown: int = 0
	for item: Item in stash.items:
		if not _matches_filter(item):
			continue
		shown += 1
		_build_item_card(item)
	if shown == 0:
		_items_vbox.add_child(DojoTheme.label("No %s items in the stash." % ("matching" if _filter == "ALL" else _filter.to_lower()), DojoTheme.FONT_SM, DojoTheme.COL_MUTED))

	var hero_name: String = session.active_hero.name if session.active_hero != null else "-"
	_summary_lbl.text = "%d items  •  %.1f weight  •  equipping for %s" % [stash.items.size(), stash.get_total_weight(), hero_name]

func _build_item_card(item: Item) -> void:
	var card := DojoTheme.card(_items_vbox)

	var head := HBoxContainer.new()
	card.add_child(head)
	head.add_child(DojoTheme.label(item.name, DojoTheme.FONT_MD, DojoTheme.rarity_color(item.rarity)))
	head.add_spacer(false)
	head.add_child(DojoTheme.label(Item.type_to_string(item.item_type), 9, DojoTheme.COL_MUTED))

	var buttons := HFlowContainer.new()
	buttons.add_theme_constant_override("h_separation", 4)
	buttons.add_theme_constant_override("v_separation", 3)

	if item is Equipment:
		var eq: Equipment = item
		var st: CharacterStats = session.active_hero.stats if session.active_hero != null else null
		var summary: Label = DojoTheme.label(DojoTheme.equipment_summary(eq, st), 9, Color(0.8, 0.9, 0.9))
		summary.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		card.add_child(summary)

		var allowed: bool = session.can_hero_equip(eq)
		if not allowed:
			var arch: String = st.archetype.display_name if (st != null and st.archetype != null) else "Hero"
			var warn: Label = DojoTheme.label("⚠ Class restricted: %s cannot use %s" % [arch, eq.weapon_type.capitalize()], 9, DojoTheme.COL_BAD)
			warn.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
			card.add_child(warn)
		elif st != null:
			var status: Dictionary = eq.get_requirement_status_summary(st, st.mastery)
			var badge_text: String = ("%s  %s" % [str(status.get("req_display", "")), str(status.get("status_tag", ""))]).strip_edges()
			if not badge_text.is_empty():
				var badge_col: Color = status.get("tag_color", Color.WHITE)
				var badge: Label = DojoTheme.label(badge_text, 9, badge_col)
				badge.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
				card.add_child(badge)

		card.add_child(buttons)
		if eq.is_weapon():
			_add_equip_button(buttons, "Main Hand", eq, Equipment.EquipSlot.MAIN_HAND, allowed)
			if not eq.is_two_handed:
				_add_equip_button(buttons, "Off Hand", eq, Equipment.EquipSlot.OFF_HAND, allowed)
		elif eq.is_shield():
			_add_equip_button(buttons, "Off Hand", eq, Equipment.EquipSlot.OFF_HAND, allowed)
		elif eq.slot == Equipment.EquipSlot.ACCESSORY_1 or eq.slot == Equipment.EquipSlot.ACCESSORY_2:
			_add_equip_button(buttons, "Equip", eq, _free_accessory_slot(), allowed)
		else:
			_add_equip_button(buttons, "Equip", eq, eq.slot, allowed)
	elif item is Consumable:
		var con: Consumable = item
		card.add_child(DojoTheme.label(DojoTheme.consumable_effect_text(con), 9, DojoTheme.COL_WARN))
		card.add_child(buttons)
		var full: bool = session.active_hero != null and session.active_hero.inventory.battle_pouch.size() >= session.active_hero.inventory.max_pouch_slots
		var pouch_btn: Button = DojoTheme.button("+ Pouch", "Load into the battle pouch" if not full else "Battle pouch is full", func() -> void: session.add_to_pouch(con), 9)
		pouch_btn.disabled = full
		buttons.add_child(pouch_btn)

func _free_accessory_slot() -> Equipment.EquipSlot:
	var inv: Inventory = session.active_hero.inventory
	if inv.get_equipped(Equipment.EquipSlot.ACCESSORY_1) == null:
		return Equipment.EquipSlot.ACCESSORY_1
	if inv.get_equipped(Equipment.EquipSlot.ACCESSORY_2) == null:
		return Equipment.EquipSlot.ACCESSORY_2
	return Equipment.EquipSlot.ACCESSORY_1

func _add_equip_button(parent: Control, text: String, eq: Equipment, slot: Equipment.EquipSlot, allowed: bool) -> void:
	var tip: String = "Equip %s to %s" % [eq.name, Equipment.slot_display_name(slot)] if allowed else "This unit's class cannot equip %s" % eq.name
	var btn: Button = DojoTheme.button(text, tip, func() -> void: session.equip_item(eq, slot), 9)
	btn.disabled = not allowed
	parent.add_child(btn)
