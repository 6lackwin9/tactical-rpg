---
class: MenuStyle
extends: RefCounted
description: "Tactical RPG - Menu screen style (SETTINGS-UI): one Theme for the menu screens and their question overlay (never the combat UI): bordered buttons with hover / pressed / focus / disabled states and a CheckBox with a visible tick box."
---

# Properties
* `COL_BTN`: Color = Color(0.14, 0.19, 0.27, 1.0) (const)
* `COL_BTN_HOVER`: Color = Color(0.20, 0.28, 0.40, 1.0) (const)
* `COL_BTN_PRESSED`: Color = Color(0.09, 0.13, 0.19, 1.0) (const)
* `COL_EDGE`: Color = Color(0.45, 0.6, 0.8, 1.0) (const)
* `COL_SELECTED`: Color = Color(0.30, 0.25, 0.08, 1.0)   # UIR-1 (X5): the filled gold-edged box of a selected row (const)
* `COL_SELECTED_HOVER`: Color = Color(0.38, 0.32, 0.12, 1.0) (const)
* `BOX_PX`: int = 24 (const)

# Methods

### `static selected_box(hover: bool = false) -> StyleBoxFlat`
```logic
return UiTheme.box(COL_SELECTED_HOVER if hover else COL_SELECTED, UiTheme.COL_TITLE, 4, 12, 6, 3)
```

### `static mark_selected(b: Button, on: bool) -> void`
```logic
if on:
	b.add_theme_stylebox_override("normal", selected_box())
	b.add_theme_stylebox_override("hover", selected_box(true))
	b.add_theme_stylebox_override("pressed", selected_box())
else:
	for state: String in ["normal", "hover", "pressed"]:
		b.remove_theme_stylebox_override(state)
b.set_meta("selected", on)
```

### `static is_selected(b: Button) -> bool`
```logic
return bool(b.get_meta("selected", false))
```

### `static theme() -> Theme`
```logic
var t := Theme.new()
t.set_stylebox("normal", "Button", UiTheme.box(COL_BTN, COL_EDGE, 4, 12, 6, 2))
t.set_stylebox("hover", "Button", UiTheme.box(COL_BTN_HOVER, UiTheme.COL_HEADER, 4, 12, 6, 2))
t.set_stylebox("pressed", "Button", UiTheme.box(COL_BTN_PRESSED, UiTheme.COL_TITLE, 4, 12, 6, 2))
t.set_stylebox("disabled", "Button", UiTheme.box(Color(0.10, 0.12, 0.16, 1.0), UiTheme.COL_BORDER, 4, 12, 6, 1))
t.set_stylebox("focus", "Button", UiTheme.box(Color(0, 0, 0, 0), UiTheme.COL_TITLE, 4, 12, 6, 3))
t.set_color("font_disabled_color", "Button", UiTheme.COL_MUTED)
# CheckBox: no button chrome, a bright border on focus, hover tint; the icons are drawn here so the box shows on the dark background.
var flat: StyleBoxFlat = UiTheme.box(Color(0, 0, 0, 0), Color(0, 0, 0, 0), 4, 6, 4, 0)
for state: String in ["normal", "pressed", "disabled"]:
	t.set_stylebox(state, "CheckBox", flat)
t.set_stylebox("hover", "CheckBox", UiTheme.box(COL_BTN_HOVER, Color(0, 0, 0, 0), 4, 6, 4, 0))
t.set_stylebox("hover_pressed", "CheckBox", UiTheme.box(COL_BTN_HOVER, Color(0, 0, 0, 0), 4, 6, 4, 0))
t.set_stylebox("focus", "CheckBox", UiTheme.box(Color(0, 0, 0, 0), UiTheme.COL_TITLE, 4, 6, 4, 2))
var off: ImageTexture = _box_icon(false)
var on: ImageTexture = _box_icon(true)
t.set_icon("unchecked", "CheckBox", off)
t.set_icon("checked", "CheckBox", on)
t.set_icon("unchecked_disabled", "CheckBox", off)
t.set_icon("checked_disabled", "CheckBox", on)
t.set_constant("h_separation", "CheckBox", 10)
return t
```

### `static _box_icon(checked: bool) -> ImageTexture`
```logic
var img: Image = Image.create(BOX_PX, BOX_PX, false, Image.FORMAT_RGBA8)
var edge: Color = UiTheme.COL_TEXT
var fill: Color = COL_BTN_PRESSED if not checked else Color(0.22, 0.40, 0.62, 1.0)
for x: int in BOX_PX:
	for y: int in BOX_PX:
		var on_edge: bool = x < 2 or y < 2 or x >= BOX_PX - 2 or y >= BOX_PX - 2
		img.set_pixel(x, y, edge if on_edge else fill)
if checked:
	var tick: Color = UiTheme.COL_TITLE
	var pts: Array[Vector2i] = []
	for i: int in 6:
		pts.append(Vector2i(5 + i, 12 + i))   # down stroke
	for i: int in 10:
		pts.append(Vector2i(10 + i, 17 - i))   # up stroke
	for pt: Vector2i in pts:
		for dx: int in 3:
			for dy: int in 3:
				img.set_pixel(pt.x + dx - 1, pt.y + dy - 1, tick)
return ImageTexture.create_from_image(img)
```

