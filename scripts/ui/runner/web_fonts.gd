class_name WebFonts
extends RefCounted

# ==============================================================================
# Tactical RPG - Web glyph fallbacks
# Desktop builds borrow emoji and symbol glyphs (⚔ ▶ ↻ 🌍 ...) from the OS fonts;
# the browser build has no OS fonts, so those glyphs render as boxes. install()
# appends bundled fallback fonts (assets/fonts, sources in its README) to the
# project's default font, on the web only. Desktop keeps its OS fallbacks.
# ==============================================================================

const FALLBACKS: Array[String] = [
	"res://assets/fonts/dejavu_sans_symbols.ttf",   # arrows, box drawing, shapes, dingbats
	"res://assets/fonts/noto_emoji.ttf",            # emoji (monochrome)
	"res://assets/fonts/noto_symbols2_sub.ttf",     # ⏻ and other technical symbols
	"res://assets/fonts/noto_symbols_sub.ttf",      # ⛶
	"res://assets/fonts/noto_math_sub.ttf",         # ⧉
]

# Adds the fallbacks once; returns how many were added (0 off the web or when already installed).
static func install(force: bool = false) -> int:
	if not force and not OS.has_feature("web"):
		return 0
	var base: Font = ThemeDB.fallback_font
	var chain: Array[Font] = base.fallbacks.duplicate()
	var added: int = 0
	for path: String in FALLBACKS:
		var font: Font = load(path) as Font
		if font != null and not chain.has(font):
			chain.append(font)
			added += 1
	base.fallbacks = chain
	return added
