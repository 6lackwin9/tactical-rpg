---
class: CharacterEquipmentModal
extends: PanelContainer
description: "Tactical RPG - Party & Stash Screen Full-screen modal composed of three reusable columns that share one DojoSession: A) Party viewer  - party member list + unit inspector / derived stats B) Gear loadout  - equipped slots, battle pouch, inline equip pickers C) Party stash   - filterable shared stash with one-click equip Public API (setup_heroes / select_hero / open_modal / close_modal / signals) is unchanged so the combat arena and its tests keep working."
---

# Properties
* `session`: DojoSession = null
* `shared_inventory`: Inventory = null
* `party_allies`: Array[Combatant]:
* `empty`: Array[Combatant] = []
* `active_hero`: Combatant:
* `_party_panel`: DojoPartyPanel
* `_gear_panel`: DojoGearPanel
* `_stash_panel`: DojoStashPanel
* `_gear_title`: Label
* `_overlay_layer`: UiOverlayLayer

# Signals
* `modal_closed()`
* `equipment_updated(combatant: Combatant)`

# Methods

### `_init() -> void`
```logic
name = "CharacterEquipmentModal"
visible = false
set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
mouse_filter = Control.MOUSE_FILTER_STOP
theme = UiTheme.tooltip_theme()
add_theme_stylebox_override("panel", UiTheme.box(Color(0.02, 0.04, 0.07, 0.92), Color(0, 0, 0, 0), 0, 0, 0, 0))

var margin := MarginContainer.new()
for side: String in ["left", "right", "top", "bottom"]:
	margin.add_theme_constant_override("margin_%s" % side, 28)
add_child(margin)

var frame := PanelContainer.new()
frame.add_theme_stylebox_override("panel", UiTheme.box(Color(0.07, 0.09, 0.14, 0.99), Color(0.28, 0.46, 0.75, 0.9), 8, 12, 10, 2))
margin.add_child(frame)

var root := VBoxContainer.new()
root.add_theme_constant_override("separation", 8)
frame.add_child(root)

root.add_child(_build_header())

var columns := HBoxContainer.new()
columns.add_theme_constant_override("separation", 10)
columns.size_flags_vertical = Control.SIZE_EXPAND_FILL
root.add_child(columns)

_party_panel = DojoPartyPanel.new()
_gear_panel = DojoGearPanel.new()
_gear_panel.show_quick_setup = false
_stash_panel = DojoStashPanel.new()

columns.add_child(_build_column("👥  PARTY", _party_panel))
_gear_title = UiTheme.header("🎒  GEAR LOADOUT", UiTheme.COL_TITLE)
columns.add_child(_build_column_with_label(_gear_title, _gear_panel))
columns.add_child(_build_column("📦  PARTY STASH", _stash_panel))

# Overlay windows (equip picker, stat allocation, custom stash filter) open above all columns.
_overlay_layer = UiOverlayLayer.new()
add_child(_overlay_layer)
_overlay_layer.host_panel(_party_panel)
_overlay_layer.host_panel(_gear_panel)
_overlay_layer.host_panel(_stash_panel)
```

### `_build_header() -> Control`
```logic
var header := HBoxContainer.new()
header.add_theme_constant_override("separation", 10)
header.add_child(UiTheme.label("PARTY & STASH", UiTheme.FONT_LG + 2, UiTheme.COL_TITLE))
header.add_child(UiTheme.label("Select a unit, review their stats, then move gear between the loadout and the stash.", UiTheme.FONT_SM, UiTheme.COL_MUTED))
header.add_spacer(false)
var close_btn: Button = UiTheme.button("✕ CLOSE (Esc)", "Close the Party & Stash screen", close_modal, UiTheme.FONT_SM)
header.add_child(close_btn)
return header
```

### `_build_column(title: String, panel: DojoPanel) -> Control`
```logic
return _build_column_with_label(UiTheme.header(title, UiTheme.COL_TITLE), panel)
```

### `_build_column_with_label(title_lbl: Label, panel: DojoPanel) -> Control`
```logic
var frame := PanelContainer.new()
frame.size_flags_horizontal = Control.SIZE_EXPAND_FILL
frame.size_flags_stretch_ratio = 1.0
frame.add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_PANEL_BG, UiTheme.COL_BORDER, 6, 6, 6))
var vbox := VBoxContainer.new()
vbox.add_theme_constant_override("separation", 2)
frame.add_child(vbox)
title_lbl.add_theme_font_size_override("font_size", UiTheme.FONT_MD)
vbox.add_child(title_lbl)
vbox.add_child(HSeparator.new())
panel.size_flags_vertical = Control.SIZE_EXPAND_FILL
vbox.add_child(panel)
return frame
```

### `attach_session(p_session: DojoSession) -> void`
```logic
if session != null and session.changed.is_connected(_on_session_changed):
	session.changed.disconnect(_on_session_changed)
session = p_session
session.changed.connect(_on_session_changed)
_overlay_layer.set_session(session)
_party_panel.bind(session)
_gear_panel.bind(session)
_stash_panel.bind(session)
_update_titles()
```

### `setup_heroes(allies: Array[Combatant], shared_stash: Inventory = null) -> void`
```logic
shared_inventory = shared_stash
var stash: Inventory = shared_stash
# Without a shared stash, a lone hero's own bag acts as the stash (legacy arena behavior).
if stash == null and allies.size() == 1:
	stash = allies[0].inventory
attach_session(DojoSession.wrap_party(allies, stash))
```

### `select_hero(hero: Combatant) -> void`
```logic
if session != null:
	session.set_active_hero(hero)
```

### `open_modal(hero: Combatant = null) -> void`
```logic
if hero != null:
	if session != null and session.party_allies.has(hero):
		session.set_active_hero(hero)
	else:
		var lone_party: Array[Combatant] = [hero]
		setup_heroes(lone_party, shared_inventory)
visible = true
move_to_front()
```

### `close_modal() -> void`
```logic
_overlay_layer.close_all()
visible = false
modal_closed.emit()
```

### `_unhandled_key_input(event: InputEvent) -> void`
```logic
if visible and event.is_action_pressed("ui_cancel"):
	close_modal()
	get_viewport().set_input_as_handled()
```

### `_on_session_changed() -> void`
```logic
_update_titles()
if visible:
	equipment_updated.emit(session.active_hero)
```

### `_update_titles() -> void`
```logic
var hero_name: String = session.active_hero.name if (session != null and session.active_hero != null) else ""
_gear_title.text = "🎒  GEAR LOADOUT%s" % ("  -  %s" % hero_name if not hero_name.is_empty() else "")
```

### `_equip_item(eq: Equipment) -> void`
```logic
if session != null and eq != null:
	session.equip_item(eq, eq.slot)
```

