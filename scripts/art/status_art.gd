class_name StatusArt
extends RefCounted

# ==============================================================================
# Tactical RPG - Status Icons (S14, spec S14-L1 rule 6): generated placeholder discs
# A 32x32 disc, green for buffs and red for debuffs (StatusRegistry kind), with one distinct white glyph per status id.
# A status without a glyph (a new StatusRegistry row) gets an up arrow (buff) or a down arrow (debuff); add its glyph
# to GLYPHS when the row is added. Usage: StatusArt.icon("stun") -> 64x64 Texture2D.
# ==============================================================================

const BUFF_COLOR: String = "#16a34a"
const DEBUFF_COLOR: String = "#dc2626"
const INK: String = "#0b1220"
const WHITE: String = "#f8fafc"

# One SVG path per StatusRegistry id (32x32 viewBox).
const GLYPHS: Dictionary = {
	"stagger": "M9 9 L15 15 L11 17 L19 25 L17 19 L21 17 L13 9 Z",
	"stun": "M16 7 L18 13 L24 13 L19 17 L21 23 L16 19.5 L11 23 L13 17 L8 13 L14 13 Z M7 25 H25 V27 H7 Z",
	"root": "M15 6 H17 V15 L23 21 L21.6 22.4 L17 17.8 V26 H15 V17.8 L10.4 22.4 L9 21 L15 15 Z",
	"bleed": "M16 6 C19 11 23 15 23 19.5 C23 23.6 19.9 26 16 26 C12.1 26 9 23.6 9 19.5 C9 15 13 11 16 6 Z",
	"weaken": "M8 11 H24 V14 H8 Z M11 17 H21 V20 H11 Z M14 23 H18 V26 H14 Z",
	"haste": "M18 5 L9 18 H15 L13 27 L23 13 H17 Z",
	"taunt": "M8 8 H24 V20 H15 L10 25 V20 H8 Z M15 10 V15 H17 V10 Z M15 17 V19 H17 V17 Z",
	"riposte": "M7 9 L10 7 L20 17 L22 15 L24 17 L21 20 L23 22 L21 24 L19 22 L16 25 L14 23 L16 21 Z M25 9 L22 7 L17 12 L19 14 Z",
}
const ARROW_UP: String = "M16 6 L25 16 H19.5 V26 H12.5 V16 H7 Z"
const ARROW_DOWN: String = "M16 26 L25 16 H19.5 V6 H12.5 V16 H7 Z"

# The icon of a status (registry: manifest file, else this template, else the `status` fallback).
static func icon(status_id: String) -> Texture2D:
	return AssetRegistry.texture("status", status_id, "icon")

# The generated template (cached "status|<id>"); used by AssetRegistry.
static func template(status_id: String) -> Texture2D:
	return SvgRaster.texture("status|%s" % status_id, icon_svg(status_id), AssetRegistry.TEMPLATE_SCALE)

static func is_buff(status_id: String) -> bool:
	return str(StatusRegistry.row(status_id).get("kind", StatusRegistry.KIND_DEBUFF)) == StatusRegistry.KIND_BUFF

# The glyph path of a status: its own, else the kind arrow.
static func glyph_of(status_id: String) -> String:
	if GLYPHS.has(status_id):
		return str(GLYPHS[status_id])
	return ARROW_UP if is_buff(status_id) else ARROW_DOWN

static func icon_svg(status_id: String) -> String:
	var disc: String = BUFF_COLOR if is_buff(status_id) else DEBUFF_COLOR
	return ('<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">' +
		'<circle cx="16" cy="16" r="15.5" fill="%s"/>' % INK +
		'<circle cx="16" cy="16" r="14" fill="%s" stroke="%s" stroke-width="1.2"/>' % [disc, WHITE] +
		'<path d="%s" fill="%s" stroke="%s" stroke-width="1.2" stroke-linejoin="round" fill-rule="evenodd"/></svg>' % [glyph_of(status_id), WHITE, INK])
