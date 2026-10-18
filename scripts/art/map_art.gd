class_name MapArt
extends RefCounted

# ==============================================================================
# Tactical RPG - Map icons (MAPREAD1-L2): generated placeholders for the exploration maps. Real art later replaces only
# the files behind the same keys.
# Region map markers: tile namespace overlay keys (variant v0, 54x54 template drawn at 32 px): overlay_npc, overlay_chest,
# overlay_chest_open, overlay_lore, overlay_exit_locked, overlay_encounter, overlay_roamer (overlay_exit / overlay_poi
# stay in TileArt: combat uses them too).
# World map pins and npc badges: ui namespace keys, variant icon: pin_fortress / town / camp / ruin / gate / wild (24x24),
# pin_here (the gold ring under the party's pin), badge_quest (gold "!") and badge_ready (gold "?") (32x32).
# ==============================================================================

const OVERLAY_KINDS: Array[String] = ["npc", "chest", "chest_open", "lore", "exit_locked", "encounter", "roamer"]
const PIN_KINDS: Array[String] = ["fortress", "town", "camp", "ruin", "gate", "wild"]
const UI_KEYS: Array[String] = ["badge_quest", "badge_ready", "pin_fortress", "pin_town", "pin_camp", "pin_ruin", "pin_gate", "pin_wild", "pin_here"]
const GOLD: String = "#fbbf24"
const INK: String = "#0b1220"

static func has_overlay(kind: String) -> bool:
	return OVERLAY_KINDS.has(kind)

static func has_ui(key: String) -> bool:
	return UI_KEYS.has(key)

# The ui key of a location kind's pin ("" for a kind without a pin).
static func pin_key(kind: String) -> String:
	return "pin_%s" % kind if PIN_KINDS.has(kind) else ""

# A marker icon on a translucent plate in `color`: `glyph` is drawn in `ink`.
static func _plate(color: String, glyph: String) -> String:
	return ('<svg xmlns="http://www.w3.org/2000/svg" width="54" height="54" viewBox="0 0 54 54">' +
		'<rect x="3" y="3" width="48" height="48" rx="8" fill="%s" fill-opacity="0.22" stroke="%s" stroke-width="2.5"/>' % [color, color] + glyph + '</svg>')

static func overlay_svg(kind: String) -> String:
	match kind:
		"npc":   # cyan: head and shoulders
			return _plate("#22d3ee", '<circle cx="27" cy="19" r="7.5" fill="#67e8f9" stroke="%s" stroke-width="1.5"/>' % INK +
				'<path d="M13 42 C13 32 20 28.5 27 28.5 C34 28.5 41 32 41 42 Z" fill="#67e8f9" stroke="%s" stroke-width="1.5"/>' % INK)
		"chest":   # gold: box with a lid
			return _plate(GOLD, '<rect x="12" y="26" width="30" height="16" rx="2" fill="#d97706" stroke="%s" stroke-width="1.5"/>' % INK +
				'<path d="M12 26 C12 15 42 15 42 26 Z" fill="#fbbf24" stroke="%s" stroke-width="1.5"/>' % INK +
				'<rect x="24" y="25" width="6" height="8" rx="1" fill="#fef3c7" stroke="%s" stroke-width="1"/>' % INK)
		"chest_open":   # gold, lid thrown back, empty inside
			return _plate(GOLD, '<path d="M14 24 L18 11 L36 11 L40 24 Z" fill="#fbbf24" stroke="%s" stroke-width="1.5"/>' % INK +
				'<rect x="12" y="26" width="30" height="16" rx="2" fill="#92400e" stroke="%s" stroke-width="1.5"/>' % INK +
				'<rect x="15" y="28" width="24" height="5" fill="%s"/>' % INK)
		"lore":   # violet: open book
			return _plate("#a78bfa", '<path d="M27 19 C22 15 16 15 11 17 V40 C16 38 22 38 27 42 Z" fill="#ddd6fe" stroke="%s" stroke-width="1.5"/>' % INK +
				'<path d="M27 19 C32 15 38 15 43 17 V40 C38 38 32 38 27 42 Z" fill="#c4b5fd" stroke="%s" stroke-width="1.5"/>' % INK +
				'<path d="M15 22 C18 21 21 21 24 23 M15 28 C18 27 21 27 24 29 M30 23 C33 21 36 21 39 22 M30 29 C33 27 36 27 39 28" fill="none" stroke="#6d28d9" stroke-width="1.4"/>')
		"exit_locked":   # orange: the exit arrow behind a padlock
			return _plate("#f97316", '<path d="M17 24 L27 14 L37 24 M17 31 L27 21 L37 31" fill="none" stroke="#fdba74" stroke-width="3.5" stroke-linecap="round" stroke-linejoin="round" stroke-opacity="0.55"/>' +
				'<path d="M21 33 V29 C21 22 33 22 33 29 V33" fill="none" stroke="#fed7aa" stroke-width="3"/>' +
				'<rect x="17" y="32" width="20" height="13" rx="2" fill="#ea580c" stroke="%s" stroke-width="1.5"/>' % INK +
				'<circle cx="27" cy="38" r="2" fill="%s"/>' % INK)
		"encounter":   # red: crossed blades
			return _plate("#ef4444", _blades(27.0, 27.0, 14.0, 3.6, "#fecaca"))
		"roamer":   # red: small crossed blades inside a dashed ring
			return _plate("#ef4444", '<circle cx="27" cy="27" r="17" fill="none" stroke="#fca5a5" stroke-width="2.2" stroke-dasharray="6 4"/>' + _blades(27.0, 27.0, 8.5, 2.8, "#fecaca"))
	return ""

# Two crossed blades centred on (cx, cy), arm length `r`, line width `w`.
static func _blades(cx: float, cy: float, r: float, w: float, color: String) -> String:
	var g: float = r * 0.45
	return ('<path d="M%.1f %.1f L%.1f %.1f M%.1f %.1f L%.1f %.1f" stroke="%s" stroke-width="%.1f" stroke-linecap="round"/>' % [cx - r, cy - r, cx + r, cy + r, cx + r, cy - r, cx - r, cy + r, color, w] +
		'<path d="M%.1f %.1f L%.1f %.1f M%.1f %.1f L%.1f %.1f" stroke="%s" stroke-width="%.1f" stroke-linecap="round"/>' % [cx - r * 0.55 - g, cy + r * 0.55 - g, cx - r * 0.55 + g, cy + r * 0.55 + g, cx + r * 0.55 + g, cy + r * 0.55 - g, cx + r * 0.55 - g, cy + r * 0.55 + g, color, w])

# The SVG of a ui key ("" for a key without a template).
static func ui_svg(key: String) -> String:
	match key:
		"badge_quest":
			return _badge('<path d="M16 6.5 V17.5" stroke="%s" stroke-width="4" stroke-linecap="round"/><circle cx="16" cy="23.5" r="2.4" fill="%s"/>' % [INK, INK])
		"badge_ready":
			return _badge('<path d="M11.5 12 C11.5 5.5 20.5 5.5 20.5 12 C20.5 16 16 16 16 19.5" fill="none" stroke="%s" stroke-width="3.6" stroke-linecap="round"/><circle cx="16" cy="25" r="2.4" fill="%s"/>' % [INK, INK])
		"pin_here":
			return ('<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="%s" fill-opacity="0.18" stroke="%s" stroke-width="2.4"/></svg>' % [GOLD, GOLD])
		"pin_fortress":   # keep with battlements
			return _pin('<path d="M4 20 V8 H7 V10 H9.5 V8 H14.5 V10 H17 V8 H20 V20 Z M10 20 V15 C10 12.5 14 12.5 14 15 V20" fill="#e5e7eb" fill-rule="evenodd"/>')
		"pin_town":   # houses
			return _pin('<path d="M3 20 V12 L8 7.5 L13 12 V20 Z M13 20 V13 L17 9.5 L21 13 V20 Z" fill="#fde68a"/>')
		"pin_camp":   # tent
			return _pin('<path d="M2.5 20 L12 5.5 L21.5 20 Z M12 20 L9.5 20 L12 14 L14.5 20 Z" fill="#fdba74" fill-rule="evenodd"/>')
		"pin_ruin":   # broken columns
			return _pin('<path d="M4 20 V11 L6 9 V7 H9 V20 Z M13 20 V9 L15 7 H17.5 V13 L20 15 V20 Z" fill="#cbd5e1"/>')
		"pin_gate":   # archway
			return _pin('<path d="M3.5 20 V8 H20.5 V20 H16 V13.5 C16 9.5 8 9.5 8 13.5 V20 Z" fill="#a5b4fc"/>')
		"pin_wild":   # tree
			return _pin('<path d="M12 3.5 L18.5 12 H15 L19.5 18 H13.5 V21 H10.5 V18 H4.5 L9 12 H5.5 Z" fill="#86efac"/>')
	return ""

static func _pin(shape: String) -> String:
	return ('<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">' +
		'<g stroke="%s" stroke-width="1.6" stroke-linejoin="round">%s</g></svg>' % [INK, shape])

static func _badge(glyph: String) -> String:
	return ('<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">' +
		'<circle cx="16" cy="16" r="14.5" fill="%s" stroke="%s" stroke-width="1.8"/>' % [GOLD, INK] + glyph + '</svg>')
