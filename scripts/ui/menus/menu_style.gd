class_name MenuStyle
extends RefCounted

# ==============================================================================
# Tactical RPG - Menu screen style (SETTINGS-UI): one Theme for the menu screens and their question overlay (never the
# combat UI): bordered buttons with hover / pressed / focus / disabled states and a CheckBox with a visible tick box.
# ==============================================================================

const COL_BTN: Color = Color(0.14, 0.19, 0.27, 1.0)
const COL_BTN_HOVER: Color = Color(0.20, 0.28, 0.40, 1.0)
const COL_BTN_PRESSED: Color = Color(0.09, 0.13, 0.19, 1.0)
const COL_EDGE: Color = Color(0.45, 0.6, 0.8, 1.0)
const BOX_PX: int = 24

static func theme() -> Theme:
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

# A square box with a light border; the checked one is filled and carries a tick.
static func _box_icon(checked: bool) -> ImageTexture:
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
