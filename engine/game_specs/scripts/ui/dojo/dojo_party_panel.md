---
class: DojoPartyPanel
extends: DojoPanel
description: "Tactical RPG - Party Viewer (Party & Stash screen, column A) Always shows MAX_PARTY_SIZE (4) member slots so the layout stays stable whether or not the party is full. Click a member to make them the active unit. A shared party carry bar and tabs for the unit inspector / derived stats sit below."
---

# Properties
* `SLOT_HEIGHT`: float = 46.0 (const)
* `_list`: VBoxContainer
* `_carry_bar`: DojoCarryBar
* `_inspector`: DojoUnitInspectorPanel
* `_derived`: DojoDerivedStatsPanel
* `_texture_cache`: Dictionary = {} # String -> Texture2D

# Methods

### `_init() -> void`
```logic
super._init()
panel_id = "party"
panel_title = "Party"
panel_short_title = "Party"
panel_icon = "👥"
default_min_size = Vector2(280, 320)

var vbox := VBoxContainer.new()
vbox.add_theme_constant_override("separation", 6)
add_child(vbox)

vbox.add_child(UiTheme.header("PARTY MEMBERS  (click to select)"))
_list = VBoxContainer.new()
_list.add_theme_constant_override("separation", 4)
vbox.add_child(_list)

_carry_bar = DojoCarryBar.new()
vbox.add_child(_carry_bar)

var tabs := TabContainer.new()
tabs.size_flags_vertical = Control.SIZE_EXPAND_FILL
tabs.add_theme_font_size_override("font_size", UiTheme.FONT_SM)
vbox.add_child(tabs)

_inspector = DojoUnitInspectorPanel.new()
_inspector.name = "Unit Inspector"
_inspector.overlay_requested.connect(func(content: UiPanel) -> void: overlay_requested.emit(content))
tabs.add_child(_inspector)
_derived = DojoDerivedStatsPanel.new()
_derived.name = "Derived Stats"
tabs.add_child(_derived)
```

### `_on_bound() -> void`
```logic
_inspector.bind(session)
_derived.bind(session)
```

### `_portrait(hero: Combatant) -> Texture2D`
```logic
var key: String = hero.name.to_lower()
if not _texture_cache.has(key):
	_texture_cache[key] = UnitArt.token(key, UnitArt.Team.FRIENDLY)
var raw: Variant = _texture_cache[key]
if raw is Texture2D:
	var tex: Texture2D = raw
	return tex
return null
```

### `slot_card_count() -> int`
```logic
return _list.get_child_count()
```

### `refresh() -> void`
```logic
if session == null:
	return
UiTheme.clear_children(_list)
for i: int in range(DojoSession.MAX_PARTY_SIZE):
	if i < session.party_allies.size():
		_list.add_child(_build_member_card(session.party_allies[i]))
	else:
		_list.add_child(_build_empty_slot(i + 1))
_carry_bar.show_report(session.carry_report)
```

### `_build_empty_slot(slot_number: int) -> Control`
```logic
var card := PanelContainer.new()
card.custom_minimum_size = Vector2(0, SLOT_HEIGHT)
card.add_theme_stylebox_override("panel", UiTheme.box(Color(0.07, 0.09, 0.13, 0.6), Color(0.2, 0.28, 0.4, 0.5), 5, 8, 5))
var lbl: Label = UiTheme.label("Slot %d  —  empty" % slot_number, UiTheme.FONT_SM, Color(0.4, 0.5, 0.6))
lbl.size_flags_vertical = Control.SIZE_SHRINK_CENTER
card.add_child(lbl)
return card
```

### `_build_member_card(hero: Combatant) -> Control`
```logic
var is_active: bool = hero == session.active_hero
var card := PanelContainer.new()
card.custom_minimum_size = Vector2(0, SLOT_HEIGHT)
card.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
card.tooltip_text = "Select %s" % hero.name
var border: Color = UiTheme.COL_GOOD if is_active else UiTheme.COL_BORDER
var bg: Color = Color(0.12, 0.22, 0.2, 0.95) if is_active else UiTheme.COL_CARD_BG
card.add_theme_stylebox_override("panel", UiTheme.box(bg, border, 5, 8, 4, 2 if is_active else 1))
card.gui_input.connect(func(event: InputEvent) -> void:
	var click := event as InputEventMouseButton
	if click != null and click.pressed and click.button_index == MOUSE_BUTTON_LEFT:
		session.set_active_hero(hero)
)

var row := HBoxContainer.new()
row.add_theme_constant_override("separation", 8)
row.mouse_filter = Control.MOUSE_FILTER_PASS
card.add_child(row)

var avatar := TextureRect.new()
avatar.texture = _portrait(hero)
avatar.custom_minimum_size = Vector2(34, 34)
avatar.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
avatar.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
avatar.mouse_filter = Control.MOUSE_FILTER_IGNORE
row.add_child(avatar)

var info := VBoxContainer.new()
info.size_flags_horizontal = Control.SIZE_EXPAND_FILL
info.add_theme_constant_override("separation", 0)
info.mouse_filter = Control.MOUSE_FILTER_PASS
row.add_child(info)

var arch: String = hero.stats.archetype.display_name if (hero.stats != null and hero.stats.archetype != null) else "Hero"
var level: int = hero.stats.level if hero.stats != null else 1
info.add_child(UiTheme.label(hero.name + ("  ● ACTIVE" if is_active else ""), UiTheme.FONT_MD, UiTheme.COL_GOOD if is_active else Color.WHITE))
info.add_child(UiTheme.label("Lv %d %s  •  Spd %d" % [level, arch, hero.speed], 9, UiTheme.COL_MUTED))

var hp_box := VBoxContainer.new()
hp_box.custom_minimum_size = Vector2(76, 0)
hp_box.add_theme_constant_override("separation", 2)
hp_box.mouse_filter = Control.MOUSE_FILTER_PASS
row.add_child(hp_box)
var hp_lbl: Label = UiTheme.label("%d / %d HP" % [hero.hp, hero.max_hp], 9, UiTheme.COL_TEXT)
hp_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
hp_box.add_child(hp_lbl)
var bar := ProgressBar.new()
bar.show_percentage = false
bar.max_value = maxi(1, hero.max_hp)
bar.value = hero.hp
bar.custom_minimum_size = Vector2(0, 5)
bar.mouse_filter = Control.MOUSE_FILTER_IGNORE
bar.add_theme_stylebox_override("background", UiTheme.box(Color(0.18, 0.08, 0.08, 0.9), Color(0, 0, 0, 0), 2, 0, 0, 0))
bar.add_theme_stylebox_override("fill", UiTheme.box(Color(0.25, 0.75, 0.35), Color(0, 0, 0, 0), 2, 0, 0, 0))
hp_box.add_child(bar)
return card
```

