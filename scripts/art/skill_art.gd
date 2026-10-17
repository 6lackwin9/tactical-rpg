class_name SkillArt
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill Icons (S14, spec S14-L1 rule 6): generated placeholder discs
# A 32x32 disc coloured by the family's KIND with one white glyph per kind and a grade rim (novice single, advanced
# double, expert gold). No text (ThorVG text needs fonts). Unknown family ids get the ATTACK look (key `skill`).
# Usage: SkillArt.icon("strike") -> 64x64 Texture2D (a manifest file for skill/<id>/icon replaces it).
# ==============================================================================

const KIND_COLORS: Dictionary = {
	"ATTACK": "#ef4444", "DEFEND": "#94a3b8", "SUPPORT": "#22c55e", "FOCUS": "#eab308", "MOVE": "#3b82f6",
	"DISENGAGE": "#3b82f6", "REST": "#64748b", "TARGET": "#e2e8f0", "STATUS": "#a855f7",
}
const KIND_GLYPHS: Dictionary = {
	"ATTACK": "sword", "DEFEND": "shield", "SUPPORT": "cross", "FOCUS": "star", "MOVE": "arrow",
	"DISENGAGE": "chevron", "REST": "dot", "TARGET": "ring", "STATUS": "wave",
}
# One SVG path per glyph (32x32 viewBox, drawn white with a dark outline).
const GLYPH_PATHS: Dictionary = {
	"sword": "M22 6 L26 10 L13 23 L11 25 L9 23 L11 21 Z M9 19 L13 23 M7 25 L9 23",
	"shield": "M16 7 L24 10 L24 16 C24 21 20 24 16 26 C12 24 8 21 8 16 L8 10 Z",
	"cross": "M13 8 H19 V13 H24 V19 H19 V24 H13 V19 H8 V13 H13 Z",
	"star": "M16 6 L18.6 13 L26 13 L20 17.6 L22.4 25 L16 20.4 L9.6 25 L12 17.6 L6 13 L13.4 13 Z",
	"arrow": "M7 14 H18 V9 L26 16 L18 23 V18 H7 Z",
	"chevron": "M9 20 L16 11 L23 20 L19.5 20 L16 15.5 L12.5 20 Z",
	"dot": "M16 11 A5 5 0 1 1 15.99 11 Z",
	"ring": "M16 8 A8 8 0 1 1 15.99 8 Z M16 12 A4 4 0 1 0 16.01 12 Z",
	"wave": "M6 17 C9 11 12 11 14 16 C16 21 19 21 22 15 L26 15 L26 18 L23.5 18 C20 24 15 24 12.5 18 C11 14.5 9.5 15 8.5 18 Z",
}
const FALLBACK_KIND: String = "ATTACK"
const INK: String = "#0b1220"
const WHITE: String = "#f8fafc"
const GOLD: String = "#fbbf24"

# The icon of a skill family (registry: manifest file, else this template, else the `skill` fallback).
static func icon(family_id: String) -> Texture2D:
	return AssetRegistry.texture("skill", family_id, "icon")

# The generated template (cached "skill|<id>"); used by AssetRegistry.
static func template(family_id: String) -> Texture2D:
	return SvgRaster.texture("skill|%s" % family_id, icon_svg(family_id), AssetRegistry.TEMPLATE_SCALE)

# {"kind", "grade"} of a family head row (the ATTACK novice look for unknown ids).
static func look_of(family_id: String) -> Dictionary:
	var row: Dictionary = ContentCatalog.get_entry("skills", family_id)
	var kind: String = str(row.get("kind", FALLBACK_KIND))
	if not KIND_COLORS.has(kind):
		kind = FALLBACK_KIND
	return {"kind": kind, "grade": str(row.get("grade", "novice"))}

static func icon_svg(family_id: String) -> String:
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
