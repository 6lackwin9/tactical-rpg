---
class: WebFonts
extends: RefCounted
description: "Tactical RPG - Web glyph fallbacks Desktop builds borrow emoji and symbol glyphs (⚔ ▶ ↻ 🌍 ...) from the OS fonts; the browser build has no OS fonts, so those glyphs render as boxes. install() appends bundled fallback fonts (assets/fonts, sources in its README) to the project's default font, on the web only. Desktop keeps its OS fallbacks."
---

# Properties
* `FALLBACKS`: Array[String] = [ (const)

# Methods

### `static install(force: bool = false) -> int`
```logic
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
```

