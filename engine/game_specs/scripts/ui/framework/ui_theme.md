---
class: UiTheme
extends: RefCounted
description: "Tactical RPG - Gear Dojo Shared Palette & Widget Helpers Single source for colors, font sizes, panel styleboxes and small widget factories so every dojo component looks and behaves consistently."
---

# Properties
* `FONT_SM`: int = 10 (const)
* `FONT_MD`: int = 11 (const)
* `FONT_LG`: int = 13 (const)
* `COL_APP_BG`: Color = Color(0.05, 0.07, 0.10, 1.0) (const)
* `COL_PANEL_BG`: Color = Color(0.08, 0.11, 0.16, 0.98) (const)
* `COL_CARD_BG`: Color = Color(0.11, 0.15, 0.21, 0.95) (const)
* `COL_BORDER`: Color = Color(0.22, 0.32, 0.46, 0.85) (const)
* `COL_HEADER`: Color = Color(0.65, 0.85, 1.0) (const)
* `COL_TITLE`: Color = Color(0.95, 0.85, 0.35) (const)
* `COL_TEXT`: Color = Color(0.88, 0.92, 0.96) (const)
* `COL_MUTED`: Color = Color(0.55, 0.65, 0.75) (const)
* `COL_GOOD`: Color = Color(0.4, 0.9, 0.55) (const)
* `COL_BAD`: Color = Color(0.97, 0.45, 0.45) (const)
* `COL_WARN`: Color = Color(0.98, 0.78, 0.3) (const)
* `COL_INFO`: Color = Color(0.4, 0.8, 1.0) (const)
* `HUD_FONT`: int = 14   # UIR-5: the smallest combat HUD text at Normal size (const)
* `style`: Variant

# Methods

### `static text_scale() -> float`
```logic
return maxf(0.5, SettingsFile.get_float("ui/text_scale"))
```

### `static scaled(px: int) -> int`
```logic
return roundi(float(px) * text_scale())
```

### `static hud_font(px: int = HUD_FONT) -> int`
```logic
return scaled(maxi(px, HUD_FONT))
```

### `static label(text: String, size: int = FONT_MD, color: Color = COL_TEXT) -> Label`
```logic
var lbl := Label.new()
lbl.text = text
lbl.add_theme_font_size_override("font_size", size)
lbl.add_theme_color_override("font_color", color)
return lbl
```

### `static header(text: String, color: Color = COL_HEADER) -> Label`
```logic
return label(text, FONT_SM, color)
```

### `static button(text: String, tooltip: String, callback: Callable, size: int = FONT_SM) -> Button`
```logic
var btn := Button.new()
btn.text = text
btn.tooltip_text = tooltip
btn.focus_mode = Control.FOCUS_NONE
btn.add_theme_font_size_override("font_size", size)
btn.pressed.connect(callback)
return btn
```

### `static card(parent: Control, bg: Color = COL_CARD_BG) -> VBoxContainer`
```logic
var panel := PanelContainer.new()
panel.add_theme_stylebox_override("panel", box(bg))
panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
parent.add_child(panel)
var vbox := VBoxContainer.new()
vbox.add_theme_constant_override("separation", 2)
panel.add_child(vbox)
return vbox
```

### `static icon_rect(tex: Texture2D, px: int = 32) -> TextureRect`
```logic
var rect := TextureRect.new()
rect.texture = tex
rect.custom_minimum_size = Vector2(px, px)
rect.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
rect.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
rect.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
return rect
```

### `static item_row(parent: Control, item: Item, px: int = 34) -> VBoxContainer`
```logic
var row := HBoxContainer.new()
row.add_theme_constant_override("separation", 6)
parent.add_child(row)
var icon := icon_rect(ItemArt.icon(item), px)
icon.tooltip_text = item.description if not item.description.is_empty() else item.name
icon.mouse_filter = Control.MOUSE_FILTER_PASS
row.add_child(icon)
var info := VBoxContainer.new()
info.size_flags_horizontal = Control.SIZE_EXPAND_FILL
info.add_theme_constant_override("separation", 1)
row.add_child(info)
return info
```

### `static clear_children(node: Node) -> void`
```logic
for child: Node in node.get_children():
	node.remove_child(child)
	child.queue_free()
```

### `static rarity_color(rarity: Item.ItemRarity) -> Color`
```logic
match rarity:
	Item.ItemRarity.UNCOMMON:
		return Color(0.4, 0.9, 0.5)
	Item.ItemRarity.RARE:
		return Color(0.35, 0.75, 1.0)
	Item.ItemRarity.EPIC:
		return Color(0.75, 0.45, 1.0)
	Item.ItemRarity.LEGENDARY:
		return Color(1.0, 0.8, 0.2)
	_:
		return Color(0.85, 0.85, 0.85)
```

### `static consumable_icon(con: Consumable) -> String`
```logic
match con.consumable_type:
	Consumable.ConsumableType.HEALTH_POTION:
		return "🧪"
	Consumable.ConsumableType.THROWABLE_BOMB:
		return "💣"
	_:
		return "✦"
```

### `static consumable_effect_text(con: Consumable) -> String`
```logic
if con.heal_amount > 0:
	return "+%d HP" % con.heal_amount
if con.damage_amount > 0:
	return "%d DMG" % con.damage_amount
return "+%d%% ATK (%d rds)" % [int(roundf(con.buff_percent * 100.0)), con.buff_duration_rounds]
```

### `static equipment_summary(eq: Equipment, st: CharacterStats) -> String`
```logic
var parts: Array[String] = []
if eq.base_power > 0:
	var tier: int = eq.get_unlocked_threshold_power_boost(st, st.proficiency) if st != null else 0
	var pen: Dictionary = eq.get_wield_penalty(st, st.proficiency) if st != null else {}
	if tier > 0:
		parts.append("Pwr %d (+%d tier)" % [eq.base_power + tier, tier])
	elif dict_bool(pen, "is_penalized"):
		var ratio: float = dict_float(pen, "power_penalty_ratio")
		parts.append("Pwr %d (-%d%% deficit)" % [maxi(1, int(roundf(float(eq.base_power) * (1.0 - ratio)))), int(roundf(ratio * 100.0))])
	else:
		parts.append("Pwr %d" % eq.base_power)
var tier_mit: int = eq.get_tier_mitigation_bonus(st, st.proficiency) if st != null else 0
if tier_mit > 0:
	parts.append("Mit +%d (+%d tier)" % [eq.mitigation + tier_mit, tier_mit])
elif eq.mitigation > 0:
	parts.append("Mit +%d" % eq.mitigation)
if eq.burden > 0:
	parts.append("Burden %d" % eq.burden)
if eq.base_range > 1:
	parts.append("Range %d" % eq.base_range)
if eq.is_two_handed:
	parts.append("2H")
var bonus: String = eq.get_stat_bonuses_display()
if not bonus.is_empty():
	parts.append(bonus)
return " • ".join(parts)
```

### `static tooltip_theme() -> Theme`
```logic
var theme := Theme.new()
theme.set_font_size("font_size", "TooltipLabel", FONT_SM)
theme.set_stylebox("panel", "TooltipPanel", box(Color(0.08, 0.11, 0.16, 0.97), Color(0.28, 0.38, 0.52, 0.9), 4, 6, 4))
return theme
```

### `static dict_int(d: Dictionary, key: String, fallback: int = 0) -> int`
```logic
var raw: Variant = d.get(key, fallback)
return raw if raw is int else fallback
```

### `static dict_float(d: Dictionary, key: String, fallback: float = 0.0) -> float`
```logic
var raw: Variant = d.get(key, fallback)
return raw if raw is float else fallback
```

### `static dict_bool(d: Dictionary, key: String, fallback: bool = false) -> bool`
```logic
var raw: Variant = d.get(key, fallback)
return raw if raw is bool else fallback
```

