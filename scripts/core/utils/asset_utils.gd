class_name AssetUtils
extends RefCounted

# ==============================================================================
# Tactical RPG - Shared Asset Utilities
# ==============================================================================

static func load_svg_texture(res_path: String) -> Texture2D:
	if FileAccess.file_exists(res_path):
		var file := FileAccess.open(res_path, FileAccess.READ)
		if file != null:
			var svg_text: String = file.get_as_text()
			var img := Image.new()
			var err: Error = img.load_svg_from_buffer(svg_text.to_utf8_buffer())
			if err == OK:
				return ImageTexture.create_from_image(img)
	return null
