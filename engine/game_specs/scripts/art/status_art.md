---
class: StatusArt
extends: RefCounted
description: "Tactical RPG - Status Icons (S14, spec S14-L1 rule 6): generated placeholder discs A 32x32 disc, green for buffs and red for debuffs (StatusRegistry kind), with one distinct white glyph per status id. A status without a glyph (a new StatusRegistry row) gets an up arrow (buff) or a down arrow (debuff); add its glyph to GLYPHS when the row is added. Usage: StatusArt.icon(\"stun\") -> 64x64 Texture2D."
---

# Properties
* `BUFF_COLOR`: String = "#16a34a" (const)
* `DEBUFF_COLOR`: String = "#dc2626" (const)
* `INK`: String = "#0b1220" (const)
* `WHITE`: String = "#f8fafc" (const)
* `GLYPHS`: Dictionary = { (const)
* `ARROW_UP`: String = "M16 6 L25 16 H19.5 V26 H12.5 V16 H7 Z" (const)
* `ARROW_DOWN`: String = "M16 26 L25 16 H19.5 V6 H12.5 V16 H7 Z" (const)

# Methods

### `static icon(status_id: String) -> Texture2D`
```logic
return AssetRegistry.texture("status", status_id, "icon")
```

### `static template(status_id: String) -> Texture2D`
```logic
return SvgRaster.texture("status|%s" % status_id, icon_svg(status_id), AssetRegistry.TEMPLATE_SCALE)
```

### `static is_buff(status_id: String) -> bool`
```logic
return str(StatusRegistry.row(status_id).get("kind", StatusRegistry.KIND_DEBUFF)) == StatusRegistry.KIND_BUFF
```

### `static glyph_of(status_id: String) -> String`
```logic
if GLYPHS.has(status_id):
	return str(GLYPHS[status_id])
return ARROW_UP if is_buff(status_id) else ARROW_DOWN
```

### `static icon_svg(status_id: String) -> String`
```logic
var disc: String = BUFF_COLOR if is_buff(status_id) else DEBUFF_COLOR
return ('<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">' +
	'<circle cx="16" cy="16" r="15.5" fill="%s"/>' % INK +
	'<circle cx="16" cy="16" r="14" fill="%s" stroke="%s" stroke-width="1.2"/>' % [disc, WHITE] +
	'<path d="%s" fill="%s" stroke="%s" stroke-width="1.2" stroke-linejoin="round" fill-rule="evenodd"/></svg>' % [glyph_of(status_id), WHITE, INK])
```

