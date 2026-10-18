---
class: ScreenFrame
extends: VBoxContainer
description: "Tactical RPG - ScreenFrame (UIR-1, UI-REVIEW1 X1, X3, X7): the one layout every menu screen adopts. title (centred)  ->  ScrollContainer \"FrameScroll\" holding a centred column \"FrameColumn\" (max COLUMN_MAX wide) ->  pinned action bar \"ActionBar\" below the scroll: Back first (left), optional extras, the primary action last (right). The bar sits outside the scroll, so it stays on screen at 1280x720 for every text size. Screens add their content to `column`. Per-screen adoption is done by later rows (UIR-2 .. UIR-6)."
---

# Properties
* `COLUMN_MAX`: int = 1000 (const)
* `SCROLLBAR_ALLOWANCE`: int = 24 (const)
* `BAR_MARGIN`: int = 12   # air under the pinned bar (const)
* `title_label`: Label = null
* `scroll`: ScrollContainer = null
* `column`: VBoxContainer = null
* `bar`: HBoxContainer = null
* `_bar_margin`: MarginContainer = null

# Methods

### `static make(title_text: String, back: Button = null, primary: Button = null, extras: Array[Button] = []) -> ScreenFrame`
```logic
var f := ScreenFrame.new()
f.name = "ScreenFrame"
f.set_anchors_preset(Control.PRESET_FULL_RECT)
f.size_flags_horizontal = Control.SIZE_EXPAND_FILL
f.size_flags_vertical = Control.SIZE_EXPAND_FILL
f.add_theme_constant_override("separation", 12)
f._build(title_text, back, primary, extras)
return f
```

### `_build(title_text: String, back: Button, primary: Button, extras: Array[Button]) -> void`
```logic
title_label = MainMenuView.title(title_text)
title_label.name = "FrameTitle"
title_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
title_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
add_child(title_label)
scroll = ScrollContainer.new()
scroll.name = "FrameScroll"
scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
scroll.follow_focus = true
add_child(scroll)
var centre := CenterContainer.new()
centre.size_flags_horizontal = Control.SIZE_EXPAND_FILL
scroll.add_child(centre)
column = VBoxContainer.new()
column.name = "FrameColumn"
column.add_theme_constant_override("separation", 10)
centre.add_child(column)
_bar_margin = MarginContainer.new()
_bar_margin.name = "ActionBarWrap"
_bar_margin.add_theme_constant_override("margin_bottom", BAR_MARGIN)
add_child(_bar_margin)
var bar_centre := CenterContainer.new()
_bar_margin.add_child(bar_centre)
bar = HBoxContainer.new()
bar.name = "ActionBar"
bar.add_theme_constant_override("separation", 12)
bar_centre.add_child(bar)
if back != null:
	back.name = "BackButton"
	bar.add_child(MainMenuView.small(back))
for b: Button in extras:
	bar.add_child(MainMenuView.small(b))
if primary != null:
	primary.name = "PrimaryButton"
	var spacer := Control.new()
	spacer.name = "BarSpacer"
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bar.add_child(spacer)
	bar.add_child(MainMenuView.small(primary))
resized.connect(_fit_width)
scroll.get_v_scroll_bar().visibility_changed.connect(_fit_width)
_fit_width()
```

### `_fit_width() -> void`
```logic
var w: float = clampf(size.x - SCROLLBAR_ALLOWANCE, 0.0, float(COLUMN_MAX))
if column != null:
	column.custom_minimum_size.x = w
if bar != null:
	bar.custom_minimum_size.x = w
if _bar_margin != null:
	var vbar: VScrollBar = scroll.get_v_scroll_bar()
	_bar_margin.add_theme_constant_override("margin_right", int(vbar.get_combined_minimum_size().x) if vbar.visible else 0)   # the bar lines up with the column
```

