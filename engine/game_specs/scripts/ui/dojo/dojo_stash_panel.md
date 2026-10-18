---
class: DojoStashPanel
extends: DojoPanel
description: "Tactical RPG - Party Stash (Party & Stash screen, column C) Shared party stash. A standard gear-slot filter bar (Main Hand / Off Hand / Body / Accessory / Consumable) works on top of an optional custom filter (stat, rarity, weapon type, damage type, hand, usable, search, sort) that opens as an overlay. Items show how the active unit fares with them and equip with one click into a specific slot (or the battle pouch)."
---

# Properties
* `_slot_btns`: Dictionary = {} # String -> Button
* `_custom_btn`: Button
* `_summary_lbl`: Label
* `_filter_lbl`: Label
* `_carry_bar`: DojoCarryBar
* `_items_vbox`: VBoxContainer
* `_status_lbl`: Label
* `inv`: Inventory = session.active_hero.inventory

# Methods

### `_init() -> void`
```logic
super._init()
panel_id = "stash"
panel_title = "Party Stash"
panel_short_title = "Stash"
panel_icon = "📦"
default_min_size = Vector2(280, 260)

var vbox := VBoxContainer.new()
vbox.add_theme_constant_override("separation", 6)
add_child(vbox)

_carry_bar = DojoCarryBar.new()
vbox.add_child(_carry_bar)

_summary_lbl = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_MUTED)
_summary_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
vbox.add_child(_summary_lbl)

var slot_row := HFlowContainer.new()
slot_row.add_theme_constant_override("h_separation", 3)
slot_row.add_theme_constant_override("v_separation", 3)
vbox.add_child(slot_row)
for key: String in StashFilter.SLOT_FILTERS:
	var btn: Button = UiTheme.button(StashFilter.slot_filter_label(key), "Show items for this gear slot", func() -> void: session.set_slot_filter(key), 9)
	btn.toggle_mode = true
	slot_row.add_child(btn)
	_slot_btns[key] = btn

var custom_row := HBoxContainer.new()
custom_row.add_theme_constant_override("separation", 6)
vbox.add_child(custom_row)
_custom_btn = UiTheme.button("⚙ Custom Filter", "Filter by stat, rarity, weapon type, damage type, hands, usability, name and sort order", func() -> void: overlay_requested.emit(DojoStashFilterPanel.new()), 9)
_custom_btn.toggle_mode = true
custom_row.add_child(_custom_btn)
_filter_lbl = UiTheme.label("", 9, UiTheme.COL_WARN)
_filter_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
_filter_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
custom_row.add_child(_filter_lbl)
custom_row.add_child(UiTheme.button("✕", "Clear the custom filter", func() -> void: session.reset_custom_filter(), 9))

var scroll := ScrollContainer.new()
scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
vbox.add_child(scroll)

_items_vbox = VBoxContainer.new()
_items_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
_items_vbox.add_theme_constant_override("separation", 4)
scroll.add_child(_items_vbox)

_status_lbl = UiTheme.label("", 9, UiTheme.COL_MUTED)
_status_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
vbox.add_child(_status_lbl)
```

### `_on_bound() -> void`
```logic
session.entry_logged.connect(_on_entry_logged)
session.stash_filter_changed.connect(refresh)
```

### `_on_entry_logged(entry: DojoLogEntry) -> void`
```logic
if entry.category != "EVENT" and entry.category != "BUFF":
	return
var failed: bool = entry.outcome_tag == "FAILED"
_status_lbl.text = "%s %s" % ["⚠" if failed else "✔", entry.detail_text]
_status_lbl.add_theme_color_override("font_color", UiTheme.COL_BAD if failed else UiTheme.COL_GOOD)
```

### `refresh() -> void`
```logic
if session == null or session.shared_stash == null:
	return
var filter: StashFilter = session.stash_filter
for key: String in _slot_btns:
	var btn_raw: Variant = _slot_btns[key]
	if btn_raw is Button:
		var btn: Button = btn_raw
		btn.set_pressed_no_signal(key == filter.slot_filter)
var custom_n: int = filter.custom_count()
_custom_btn.set_pressed_no_signal(custom_n > 0)
_custom_btn.text = "⚙ Custom Filter%s" % (" (%d)" % custom_n if custom_n > 0 else "")
_filter_lbl.text = _describe_custom(filter)

UiTheme.clear_children(_items_vbox)
var stash: Inventory = session.shared_stash
var visible_items: Array[Item] = filter.apply(stash.items, session.active_hero)
for item: Item in visible_items:
	_build_item_card(item)
if visible_items.is_empty():
	_items_vbox.add_child(UiTheme.label("No items match the current filter.", UiTheme.FONT_SM, UiTheme.COL_MUTED))

var hero_name: String = session.active_hero.name if session.active_hero != null else "-"
_summary_lbl.text = "Showing %d of %d items  •  equipping for %s" % [visible_items.size(), stash.items.size(), hero_name]
_carry_bar.show_report(session.carry_report)
```

### `_describe_custom(filter: StashFilter) -> String`
```logic
var parts: Array[String] = []
if not filter.stats.is_empty():
	parts.append("/".join(filter.stats))
if not filter.rarities.is_empty():
	parts.append("%d rarity" % filter.rarities.size())
if not filter.weapon_types.is_empty():
	parts.append("/".join(filter.weapon_types))
if not filter.damage_types.is_empty():
	parts.append("/".join(filter.damage_types))
if filter.hand != "ANY":
	parts.append(filter.hand)
if filter.usable_only:
	parts.append("usable")
if not filter.search.strip_edges().is_empty():
	parts.append("\"%s\"" % filter.search.strip_edges())
return ", ".join(parts)
```

### `_build_item_card(item: Item) -> void`
```logic
var card := UiTheme.card(_items_vbox)

var head := HBoxContainer.new()
var info: VBoxContainer = UiTheme.item_row(card, item)
info.add_child(head)
head.add_child(UiTheme.label(item.name, UiTheme.FONT_MD, UiTheme.rarity_color(item.rarity)))
head.add_spacer(false)
var type_text: String = Item.type_to_string(item.item_type)
if not item.damage_type.is_empty():
	type_text += " • %s" % item.damage_type
head.add_child(UiTheme.label(type_text, 9, UiTheme.COL_MUTED))

var buttons := HFlowContainer.new()
buttons.add_theme_constant_override("h_separation", 4)
buttons.add_theme_constant_override("v_separation", 3)

if item is Equipment:
	var eq: Equipment = item
	var st: CharacterStats = session.active_hero.stats if session.active_hero != null else null
	var summary: Label = UiTheme.label("%s  •  wt %.1f" % [UiTheme.equipment_summary(eq, st), eq.weight], 9, Color(0.8, 0.9, 0.9))
	summary.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	info.add_child(summary)

	var allowed: bool = session.can_hero_equip(eq)
	if not allowed:
		var arch: String = st.archetype.display_name if (st != null and st.archetype != null) else "Hero"
		var warn: Label = UiTheme.label("⚠ Class restricted: %s cannot use %s" % [arch, eq.weapon_type.capitalize()], 9, UiTheme.COL_BAD)
		warn.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		info.add_child(warn)
	elif st != null:
		var status: Dictionary = eq.get_requirement_status_summary(st, st.proficiency)
		var badge_text: String = ("%s  %s" % [str(status.get("req_display", "")), str(status.get("status_tag", ""))]).strip_edges()
		if not badge_text.is_empty():
			var badge_col: Color = status.get("tag_color", Color.WHITE)
			var badge: Label = UiTheme.label(badge_text, 9, badge_col)
			badge.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
			info.add_child(badge)

	card.add_child(buttons)
	# Buttons follow the item's real slot rules (Equipment.can_equip_in_slot).
	if eq.can_equip_in_slot(Equipment.EquipSlot.MAIN_HAND):
		_add_equip_button(buttons, "Main Hand", eq, Equipment.EquipSlot.MAIN_HAND, allowed)
	if eq.can_equip_in_slot(Equipment.EquipSlot.OFF_HAND):
		_add_equip_button(buttons, "Off Hand", eq, Equipment.EquipSlot.OFF_HAND, allowed)
	if eq.item_type == Item.ItemType.ARMOR:
		_add_equip_button(buttons, "Equip", eq, eq.slot, allowed)
	elif eq.item_type == Item.ItemType.ACCESSORY:
		_add_equip_button(buttons, "Equip", eq, _free_accessory_slot(), allowed)
elif item is Consumable:
	var con: Consumable = item
	info.add_child(UiTheme.label("%s  •  wt %.1f" % [UiTheme.consumable_effect_text(con), con.weight], 9, UiTheme.COL_WARN))
	card.add_child(buttons)
	var full: bool = session.active_hero != null and session.active_hero.inventory.battle_pouch.size() >= session.active_hero.inventory.max_pouch_slots
	var pouch_btn: Button = UiTheme.button("+ Pouch", "Load into the battle pouch" if not full else "Battle pouch is full", func() -> void: session.add_to_pouch(con), 9)
	pouch_btn.disabled = full
	buttons.add_child(pouch_btn)
```

### `_add_equip_button(parent: Control, text: String, eq: Equipment, slot: Equipment.EquipSlot, allowed: bool) -> void`
```logic
var tip: String = "Equip %s to %s" % [eq.name, Equipment.slot_display_name(slot)] if allowed else "This unit's class cannot equip %s" % eq.name
var btn: Button = UiTheme.button(text, tip, func() -> void: session.equip_item(eq, slot), 9)
btn.disabled = not allowed
parent.add_child(btn)
```

