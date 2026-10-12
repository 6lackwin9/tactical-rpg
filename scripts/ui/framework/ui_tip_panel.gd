class_name UiTipPanel
extends PanelContainer

# ==============================================================================
# Tactical RPG - Formula Hover Panel
# A PanelContainer whose tooltip renders BBCode (colored, multi-line formulas).
# Assign the BBCode to `tooltip_text`.
# ==============================================================================

const TIP_WIDTH: float = 320.0

func _make_custom_tooltip(for_text: String) -> Object:
	var rich := RichTextLabel.new()
	rich.bbcode_enabled = true
	rich.fit_content = true
	rich.scroll_active = false
	rich.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	rich.custom_minimum_size = Vector2(TIP_WIDTH, 0)
	rich.add_theme_font_size_override("normal_font_size", UiTheme.FONT_SM)
	rich.add_theme_font_size_override("bold_font_size", UiTheme.FONT_SM)
	rich.add_theme_color_override("default_color", UiTheme.COL_TEXT)
	rich.text = for_text

	var frame := PanelContainer.new()
	frame.add_theme_stylebox_override("panel", UiTheme.box(Color(0.06, 0.09, 0.13, 0.98), Color(0.35, 0.5, 0.75, 0.95), 4, 8, 6))
	frame.add_child(rich)
	return frame
