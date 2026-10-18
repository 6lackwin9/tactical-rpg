---
class: UnitInspectorUI
extends: PanelContainer
description: "Tactical RPG - Unit hover card (COMBAT-UI2-L2; replaces the old Unit Inspector panel) A floating, read-only card the grid panel shows beside a hovered, focused or pinned unit: portrait, HP and AP gauges, stance, speed, reach, position, target, effects, gear (allies, text only), then the plan steps (ally) or the AI intel and the numbers versus the selected hero (hostile). Content is rebuilt on every update so the text always follows UiTheme.text_scale() (14 px floor); the card sizes itself to its content (settle) and never scrolls or clips. Placement lives in CombatGridPanel."
---

# Properties
* `CARD_W`: float = 270.0   # at Normal text; grows with the text size (const)
* `name_lbl`: Label
* `pin_btn`: Button
* `is_pinned`: bool = false
* `current_inspected_combatant`: Combatant = null
* `_body`: VBoxContainer
* `c`: Combatant = selected_combatant
* `cur_hp`: int = c.hp
* `max_hp`: int = c.max_hp
* `stance`: CombatTypes.StanceType = c.current_stance
* `target_val`: Variant = c.active_target
* `boost_rds`: int = c.attack_boost_rounds_remaining
* `withdrawn`: bool = c.is_withdrawn
* `boost_mult`: float = c.attack_boost_multiplier
* `cur_ap`: int = c.ap
* `raw_hp`: Variant = snap.hps.get(c, cur_hp)
* `raw_max`: Variant = snap.max_hps.get(c, max_hp)
* `raw_stance`: Variant = snap.stances.get(c, stance)
* `raw_rds`: Variant = snap.attack_boost_rds.get(c, boost_rds)
* `raw_withdrawn`: Variant = snap.is_withdrawn.get(c, withdrawn)
* `alive`: bool = cur_hp > 0
* `fs`: int = UiTheme.hud_font()
* `team_col`: Color = Color(0.4, 0.75, 1.0) if is_ally else Color(1.0, 0.45, 0.45)
* `head`: Variant
* `avatar`: TextureRect = UiTheme.icon_rect(sprite_texture, 2 * CombatHudMetrics.line_height(fs))
* `names`: Variant
* `names_w`: float = _inner_width() - float(2 * CombatHudMetrics.line_height(fs)) - float(fs) * 4.5 - 12.0   # beside the portrait and the Pin button
* `team_text`: String = ("Hero" if is_ally else "Hostile") + ("" if alive else " - defeated") + (" - withdrew" if withdrawn else "")
* `band`: Vector2i = SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(c)
* `reach_text`: String = ("Reach %d tile" % band.y) if band.x == band.y else ("Reach %d-%d tiles" % [band.x, band.y])
* `tgt`: Combatant = target_val
* `effects`: Array[String] = []
* `gear`: String = "None"
* `wpn`: Equipment = c.inventory.get_main_weapon()
* `act_col`: Color = act.get("color", Color.WHITE)
* `desc`: String = str(act.get("desc", ""))
* `plan_head`: String = str(intent.get("head", ""))

# Signals
* `pin_toggled(pinned: bool)`
* `card_clicked()`

# Methods

### `_init() -> void`
```logic
name = "UnitHoverCard"
visible = false
mouse_filter = Control.MOUSE_FILTER_STOP
z_index = 50
add_theme_stylebox_override("panel", UiTheme.box(Color(0.06, 0.08, 0.12, 0.97), Color(0.35, 0.55, 0.85, 0.9), 6, 8, 6))
_body = VBoxContainer.new()
_body.add_theme_constant_override("separation", 2)
add_child(_body)
gui_input.connect(func(event: InputEvent) -> void:
	var mouse_event := event as InputEventMouseButton
	if mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT:
		card_clicked.emit()
)
```

### `card_width() -> float`
```logic
return roundf(CARD_W * maxf(1.0, UiTheme.text_scale()))
```

### `set_pinned(value: bool) -> void`
```logic
is_pinned = value
if pin_btn != null:
	pin_btn.text = "Unpin" if value else "Pin"
	pin_btn.tooltip_text = "Click to unpin" if value else "Keep this card open (Esc or a click elsewhere closes it)"
add_theme_stylebox_override("panel", UiTheme.box(Color(0.06, 0.08, 0.12, 0.97), UiTheme.COL_WARN if value else Color(0.35, 0.55, 0.85, 0.9), 6, 8, 6, 2 if value else 1))
```

### `settle() -> void`
```logic
size = Vector2(card_width(), 0)
for _pass: int in range(3):
	_sort_now(self)
	size = Vector2(card_width(), get_combined_minimum_size().y)
```

### `static _sort_now(node: Node) -> void`
```logic
if node is Container:
	(node as Container).notification(Container.NOTIFICATION_SORT_CHILDREN)
for child: Node in node.get_children():
	_sort_now(child)
```

### `content_fits() -> bool`
```logic
return size.y + 0.5 >= get_combined_minimum_size().y and size.x + 0.5 >= get_combined_minimum_size().x
```

### `_line(parent: Container, text: String, fs: int, color: Color, width: float = -1.0) -> Label`
```logic
var lbl: Label = UiTheme.label(text, fs, color)
lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
var w: float = width if width > 0.0 else _inner_width()
var font: Font = lbl.get_theme_font("font")
var flags: int = TextServer.BREAK_WORD_BOUND | TextServer.BREAK_ADAPTIVE | TextServer.BREAK_MANDATORY
var measured: Vector2 = font.get_multiline_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, w, fs, -1, flags)
lbl.custom_minimum_size = Vector2(w, ceilf(measured.y))   # a label's own minimum (same width) takes over if it is taller
parent.add_child(lbl)
return lbl
```

### `_inner_width() -> float`
```logic
return card_width() - 2.0 * 8.0 - 2.0   # the style's content margins and its border
```

### `_section(text: String, color: Color, fs: int) -> void`
```logic
_line(_body, text, fs, color)
```

### `_hp_color(hp: int, max_hp: int) -> Color`
```logic
var ratio: float = float(hp) / float(maxi(1, max_hp))
if ratio > 0.5:
	return Color(0.25, 0.75, 0.35)
return Color(0.9, 0.75, 0.2) if ratio > 0.25 else Color(0.9, 0.25, 0.25)
```

### `_gauge(tag: String, value_text: String, value: int, max_value: int, fill: Color, tag_color: Color, fs: int) -> void`
```logic
var box := VBoxContainer.new()
box.add_theme_constant_override("separation", 1)
_body.add_child(box)
var row := HBoxContainer.new()
box.add_child(row)
row.add_child(UiTheme.label(tag, fs, tag_color))
row.add_spacer(false)
row.add_child(UiTheme.label(value_text, fs, Color(0.9, 0.95, 1.0)))
var bar := ProgressBar.new()
bar.custom_minimum_size = Vector2(0, 6)
bar.show_percentage = false
bar.max_value = max_value
bar.value = value
var bg := StyleBoxFlat.new()
bg.bg_color = Color(0.12, 0.12, 0.16, 0.9)
bg.set_corner_radius_all(3)
var fill_box := StyleBoxFlat.new()
fill_box.bg_color = fill
fill_box.set_corner_radius_all(3)
bar.add_theme_stylebox_override("background", bg)
bar.add_theme_stylebox_override("fill", fill_box)
box.add_child(bar)
```

