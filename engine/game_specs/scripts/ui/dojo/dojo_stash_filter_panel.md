---
class: DojoStashFilterPanel
extends: DojoPanel
description: "Tactical RPG - Custom Stash Filter (overlay content) Filters the stash across extra dimensions on top of the standard gear-slot filter: stat, rarity, weapon type, damage type, hand, class usability, name search and sort. Weapon / damage type options come from the stash contents."
---

# Properties
* `RARITY_NAMES`: Array[String] = ["Common", "Uncommon", "Rare", "Epic", "Legendary"] (const)
* `_search_edit`: LineEdit
* `_chips_vbox`: VBoxContainer

# Methods

### `_init() -> void`
```logic
super._init()
panel_id = "stash_filter"
panel_title = "Custom Stash Filter"
panel_short_title = "Filter"
panel_icon = "⚙"
default_min_size = Vector2(360, 420)

var scroll := ScrollContainer.new()
scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
add_child(scroll)

var vbox := VBoxContainer.new()
vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
vbox.add_theme_constant_override("separation", 6)
scroll.add_child(vbox)

vbox.add_child(UiTheme.header("NAME SEARCH"))
_search_edit = LineEdit.new()
_search_edit.placeholder_text = "Type part of an item name..."
_search_edit.clear_button_enabled = true
_search_edit.add_theme_font_size_override("font_size", UiTheme.FONT_SM)
_search_edit.text_changed.connect(_on_search_changed)
vbox.add_child(_search_edit)

_chips_vbox = VBoxContainer.new()
_chips_vbox.add_theme_constant_override("separation", 6)
vbox.add_child(_chips_vbox)
```

### `_on_bound() -> void`
```logic
session.stash_filter_changed.connect(refresh)
```

### `_on_search_changed(text: String) -> void`
```logic
session.stash_filter.search = text
session.notify_filter_changed()
```

### `refresh() -> void`
```logic
if session == null:
	return
var filter: StashFilter = session.stash_filter
if _search_edit.text != filter.search:
	_search_edit.text = filter.search

UiTheme.clear_children(_chips_vbox)

_chips_vbox.add_child(UiTheme.header("STAT  (scales with, grants, or requires)"))
var stat_flow: HFlowContainer = _flow()
for key: String in StashFilter.STAT_KEYS:
	stat_flow.add_child(_chip(key, filter.stats.has(key), func() -> void: _toggle(func() -> void: filter.toggle_stat(key))))

_chips_vbox.add_child(UiTheme.header("RARITY"))
var rarity_flow: HFlowContainer = _flow()
for i: int in range(RARITY_NAMES.size()):
	rarity_flow.add_child(_chip(RARITY_NAMES[i], filter.rarities.has(i), func() -> void: _toggle(func() -> void: filter.toggle_rarity(i))))

var weapon_types: Array[String] = []
var damage_types: Array[String] = []
for item: Item in session.shared_stash.items:
	if item is Equipment:
		var eq: Equipment = item
		if not eq.weapon_type.is_empty() and not weapon_types.has(eq.weapon_type):
			weapon_types.append(eq.weapon_type)
	if not item.damage_type.is_empty() and not damage_types.has(item.damage_type):
		damage_types.append(item.damage_type)
# Keep already-selected values visible even if the stash no longer holds them.
for selected: String in filter.weapon_types:
	if not weapon_types.has(selected):
		weapon_types.append(selected)
for selected: String in filter.damage_types:
	if not damage_types.has(selected):
		damage_types.append(selected)
weapon_types.sort()
damage_types.sort()

_chips_vbox.add_child(UiTheme.header("WEAPON / ARMOR TYPE"))
var type_flow: HFlowContainer = _flow()
for wtype: String in weapon_types:
	type_flow.add_child(_chip(wtype.capitalize(), filter.weapon_types.has(wtype), func() -> void: _toggle(func() -> void: filter.toggle_weapon_type(wtype))))

_chips_vbox.add_child(UiTheme.header("DAMAGE TYPE"))
var dmg_flow: HFlowContainer = _flow()
for dtype: String in damage_types:
	dmg_flow.add_child(_chip(dtype.capitalize(), filter.damage_types.has(dtype), func() -> void: _toggle(func() -> void: filter.toggle_damage_type(dtype))))

_chips_vbox.add_child(UiTheme.header("HANDS"))
var hand_flow: HFlowContainer = _flow()
for mode: String in StashFilter.HAND_MODES:
	hand_flow.add_child(_chip("Any" if mode == "ANY" else mode, filter.hand == mode, func() -> void: _toggle(func() -> void: filter.hand = mode)))

var usable := CheckBox.new()
var hero_name: String = session.active_hero.name if session.active_hero != null else "the active unit"
usable.text = "Only gear %s can equip" % hero_name
usable.button_pressed = filter.usable_only
usable.focus_mode = Control.FOCUS_NONE
usable.add_theme_font_size_override("font_size", UiTheme.FONT_SM)
usable.toggled.connect(func(pressed: bool) -> void: _toggle(func() -> void: filter.usable_only = pressed))
_chips_vbox.add_child(usable)

_chips_vbox.add_child(UiTheme.header("SORT BY"))
var sort_option := OptionButton.new()
sort_option.focus_mode = Control.FOCUS_NONE
sort_option.fit_to_longest_item = false
sort_option.add_theme_font_size_override("font_size", UiTheme.FONT_SM)
for mode: String in StashFilter.SORT_MODES:
	sort_option.add_item(mode.capitalize())
sort_option.select(maxi(0, StashFilter.SORT_MODES.find(filter.sort_mode)))
sort_option.item_selected.connect(func(index: int) -> void: _toggle(func() -> void: filter.sort_mode = StashFilter.SORT_MODES[index]))
_chips_vbox.add_child(sort_option)

var footer := HBoxContainer.new()
footer.add_theme_constant_override("separation", 6)
_chips_vbox.add_child(footer)
footer.add_child(UiTheme.button("↺ Reset custom filter", "Clear every custom dimension (keeps the gear-slot filter)", func() -> void: session.reset_custom_filter()))
footer.add_spacer(false)
footer.add_child(UiTheme.label("%d active" % filter.custom_count(), UiTheme.FONT_SM, UiTheme.COL_WARN if filter.custom_count() > 0 else UiTheme.COL_MUTED))
```

### `_flow() -> HFlowContainer`
```logic
var flow := HFlowContainer.new()
flow.add_theme_constant_override("h_separation", 4)
flow.add_theme_constant_override("v_separation", 4)
_chips_vbox.add_child(flow)
return flow
```

### `_chip(text: String, active: bool, callback: Callable) -> Button`
```logic
var btn: Button = UiTheme.button(text, "", callback, UiTheme.FONT_SM)
btn.toggle_mode = true
btn.button_pressed = active
return btn
```

### `_toggle(mutation: Callable) -> void`
```logic
mutation.call()
session.notify_filter_changed()
```

