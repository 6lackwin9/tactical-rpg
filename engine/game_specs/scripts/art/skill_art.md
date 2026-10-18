---
class: SkillArt
extends: RefCounted
description: "Tactical RPG - Skill Icons (S14, spec S14-L1 rule 6): generated placeholder discs A 32x32 disc coloured by the family's KIND with one white glyph per kind and a grade rim (novice single, advanced double, expert gold). No text (ThorVG text needs fonts). Unknown family ids get the ATTACK look (key `skill`). Usage: SkillArt.icon(\"strike\") -> 64x64 Texture2D (a manifest file for skill/<id>/icon replaces it)."
---

# Properties
* `KIND_COLORS`: Dictionary = { (const)
* `KIND_GLYPHS`: Dictionary = { (const)
* `GLYPH_PATHS`: Dictionary = { (const)
* `FALLBACK_KIND`: String = "ATTACK" (const)
* `INK`: String = "#0b1220" (const)
* `WHITE`: String = "#f8fafc" (const)
* `GOLD`: String = "#fbbf24" (const)

# Methods

### `static icon(family_id: String) -> Texture2D`
```logic
return AssetRegistry.texture("skill", family_id, "icon")
```

### `static template(family_id: String) -> Texture2D`
```logic
return SvgRaster.texture("skill|%s" % family_id, icon_svg(family_id), AssetRegistry.TEMPLATE_SCALE)
```

### `static look_of(family_id: String) -> Dictionary`
```logic
var row: Dictionary = ContentCatalog.get_entry("skills", family_id)
var kind: String = str(row.get("kind", FALLBACK_KIND))
if not KIND_COLORS.has(kind):
	kind = FALLBACK_KIND
return {"kind": kind, "grade": str(row.get("grade", "novice"))}
```

### `static icon_svg(family_id: String) -> String`
```logic
var look: Dictionary = look_of(family_id)
var kind: String = str(look["kind"])
var rim: String
match str(look["grade"]):
	"expert":
		rim = '<circle cx="16" cy="16" r="14.6" fill="none" stroke="%s" stroke-width="2.2"/>' % GOLD
	"advanced":
		rim = '<circle cx="16" cy="16" r="15" fill="none" stroke="%s" stroke-width="1.1"/><circle cx="16" cy="16" r="12.6" fill="none" stroke="%s" stroke-width="1.1"/>' % [WHITE, WHITE]
	_:
		rim = '<circle cx="16" cy="16" r="14.6" fill="none" stroke="%s" stroke-width="1.4"/>' % WHITE
var glyph: String = str(GLYPH_PATHS[str(KIND_GLYPHS[kind])])
return ('<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">' +
	'<circle cx="16" cy="16" r="15.5" fill="%s"/>' % INK +
	'<circle cx="16" cy="16" r="14" fill="%s"/>' % str(KIND_COLORS[kind]) + rim +
	'<path d="%s" fill="%s" stroke="%s" stroke-width="1.2" stroke-linejoin="round"/></svg>' % [glyph, WHITE, INK])
```

