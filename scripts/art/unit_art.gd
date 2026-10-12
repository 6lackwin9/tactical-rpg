class_name UnitArt
extends RefCounted

# ==============================================================================
# Tactical RPG - Unit Art (tokens & portraits)
# Every unit's art is generated from SVG templates so it stays consistent:
#   TOKEN    - disc + rim in the TEAM color (blue friendly, red hostile, yellow
#              neutral) with a white CLASS glyph. Hostile rims are spiked and
#              neutral rims dashed, so teams also read without color.
#   PORTRAIT - shared bust template (shoulders, head, ears) + class headgear,
#              armor palette and weapon, on a team-colored backdrop.
# Usage: UnitArt.token("warrior", UnitArt.Team.FRIENDLY)
# ==============================================================================

enum Team { FRIENDLY, HOSTILE, NEUTRAL }

const CLASS_KEYS: Array[String] = ["warrior", "ranger", "archer", "scout", "brawler", "berserker", "shaman", "skirmisher", "dummy"]
const GOBLIN_CLASSES: Array[String] = ["archer", "berserker", "shaman", "skirmisher"]

const TEAM_PALETTES: Dictionary = {
	Team.FRIENDLY: {"light": "#60a5fa", "dark": "#1e3a8a", "rim": "#93c5fd", "accent": "#3b82f6"},
	Team.HOSTILE: {"light": "#f87171", "dark": "#7f1d1d", "rim": "#fca5a5", "accent": "#ef4444"},
	Team.NEUTRAL: {"light": "#facc15", "dark": "#854d0e", "rim": "#fde68a", "accent": "#eab308"},
}

const INK: String = "#0b1220"
const WHITE: String = "#f8fafc"

static func team_color(team: Team) -> Color:
	return Color(str(_palette(team).get("accent", "#ffffff")))

static func _palette(team: Team) -> Dictionary:
	var raw: Variant = TEAM_PALETTES.get(team, TEAM_PALETTES[Team.NEUTRAL])
	return raw if raw is Dictionary else {}

static func normalize_class(class_key: String) -> String:
	var key: String = class_key.to_lower()
	return key if CLASS_KEYS.has(key) else "unit"

# ------------------------------------------------------------------------------
# Tokens
# ------------------------------------------------------------------------------

static func token(class_key: String, team: Team) -> Texture2D:
	var key: String = normalize_class(class_key)
	return SvgRaster.texture("token|%s|%d" % [key, team], token_svg(key, team), 2.0)

static func token_svg(class_key: String, team: Team) -> String:
	var pal: Dictionary = _palette(team)
	var rim_extra: String = ""
	match team:
		Team.HOSTILE:
			rim_extra = _spikes(str(pal["rim"]))
		Team.NEUTRAL:
			rim_extra = '<circle cx="32" cy="32" r="30.5" fill="none" stroke="{rim}" stroke-width="1.6" stroke-dasharray="4 3"/>'.format({"rim": pal["rim"]})
	var svg: String = """<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64">
<defs><radialGradient id="g" cx="0.5" cy="0.36" r="0.7"><stop offset="0" stop-color="{light}"/><stop offset="1" stop-color="{dark}"/></radialGradient></defs>
{rim_extra}
<circle cx="32" cy="32" r="28.5" fill="{ink}"/>
<circle cx="32" cy="32" r="26.5" fill="url(#g)" stroke="{rim}" stroke-width="3"/>
<circle cx="32" cy="32" r="21.5" fill="none" stroke="{rim}" stroke-width="0.8" opacity="0.45"/>
{glyph}
</svg>"""
	return svg.format({
		"light": pal["light"], "dark": pal["dark"], "rim": pal["rim"], "ink": INK,
		"rim_extra": rim_extra, "glyph": _glyph(class_key, pal),
	})

static func _spikes(color: String) -> String:
	var parts: Array[String] = []
	for i: int in range(8):
		var a: float = TAU * float(i) / 8.0 + TAU / 16.0
		var tip := Vector2(32, 32) + Vector2(cos(a), sin(a)) * 31.5
		var l := Vector2(32, 32) + Vector2(cos(a - 0.17), sin(a - 0.17)) * 26.0
		var r := Vector2(32, 32) + Vector2(cos(a + 0.17), sin(a + 0.17)) * 26.0
		parts.append('<path d="M%.1f %.1f L%.1f %.1f L%.1f %.1f Z" fill="%s"/>' % [l.x, l.y, tip.x, tip.y, r.x, r.y, color])
	return "".join(parts)

# Outlined line: dark underlay + white top stroke (reads on any team color).
static func _ln(x1: float, y1: float, x2: float, y2: float, w: float, color: String = WHITE) -> String:
	return ('<path d="M{x1} {y1} L{x2} {y2}" stroke="{ink}" stroke-width="{wo}" stroke-linecap="round"/>' +
		'<path d="M{x1} {y1} L{x2} {y2}" stroke="{c}" stroke-width="{w}" stroke-linecap="round"/>').format({
		"x1": x1, "y1": y1, "x2": x2, "y2": y2, "w": w, "wo": w + 2.2, "ink": INK, "c": color})

static func _shape(d: String, fill: String = WHITE) -> String:
	return '<path d="{d}" fill="{f}" stroke="{ink}" stroke-width="1.4" stroke-linejoin="round"/>'.format({"d": d, "f": fill, "ink": INK})

static func _glyph(class_key: String, pal: Dictionary) -> String:
	var dark: String = str(pal["dark"])
	match class_key:
		"warrior":  # sword behind a heater shield
			return (_ln(18, 47, 46, 17, 3.2) + _ln(36.5, 21.5, 42.5, 27.5, 2.6) +
				_shape("M32 20 L43 24 L43 33 C43 40 38 45 32 48 C26 45 21 40 21 33 L21 24 Z") +
				'<path d="M32 25 V42 M26 31 H38" stroke="{d}" stroke-width="2.6" stroke-linecap="round"/>'.format({"d": dark}))
		"ranger":  # drawn bow and arrow
			return ('<path d="M24 15 Q47 32 24 49" fill="none" stroke="{ink}" stroke-width="5.4" stroke-linecap="round"/>'.format({"ink": INK}) +
				'<path d="M24 15 Q47 32 24 49" fill="none" stroke="{w}" stroke-width="3.2" stroke-linecap="round"/>'.format({"w": WHITE}) +
				'<path d="M24 15 L24 49" stroke="{w}" stroke-width="1"/>'.format({"w": WHITE}) +
				_ln(16, 32, 44, 32, 2.4) + _shape("M49 32 L41 27.5 L41 36.5 Z") +
				_ln(16, 32, 13, 28, 1.6) + _ln(16, 32, 13, 36, 1.6))
		"archer":  # crossed arrows
			return (_ln(17, 47, 45, 19, 2.4) + _shape("M48 16 L40.5 18.5 L45.5 23.5 Z") +
				_ln(47, 47, 19, 19, 2.4) + _shape("M16 16 L18.5 23.5 L23.5 18.5 Z") +
				_ln(17, 47, 15, 42, 1.4) + _ln(17, 47, 22, 49, 1.4) + _ln(47, 47, 49, 42, 1.4) + _ln(47, 47, 42, 49, 1.4))
		"scout":  # dagger with motion streaks
			return (_shape("M32 11 L36.5 34 L27.5 34 Z") + _ln(23, 35.5, 41, 35.5, 2.6) +
				_shape("M29.5 37 H34.5 V46 H29.5 Z") + '<circle cx="32" cy="48.5" r="2.6" fill="{w}" stroke="{ink}" stroke-width="1.2"/>'.format({"w": WHITE, "ink": INK}) +
				_ln(15, 24, 21, 24, 1.6) + _ln(13, 30, 21, 30, 1.6) + _ln(15, 36, 21, 36, 1.6))
		"brawler":  # clenched fist
			return (_shape("M24 38 H40 V48 H24 Z") +
				_shape("M20 26 C20 21 24 20 26 22 C27 19 31 19 32 22 C33 19 37 19 38 22 C40 20 44 21 44 26 L44 36 C44 39 42 40 40 40 L24 40 C21 40 20 38 20 35 Z") +
				'<path d="M26 22 V29 M32 22 V29 M38 22 V29 M20 31 Q26 33 30 31" fill="none" stroke="{ink}" stroke-width="1.4" stroke-linecap="round"/>'.format({"ink": INK}))
		"berserker":  # double-bladed axe
			return (_ln(32, 13, 32, 51, 3) +
				_shape("M30 17 C19 14 14 23 16 33 C20 30 25 29.5 30 30 Z") +
				_shape("M34 17 C45 14 50 23 48 33 C44 30 39 29.5 34 30 Z") +
				'<circle cx="32" cy="23.5" r="2.4" fill="{d}"/>'.format({"d": dark}))
		"shaman":  # staff with crescent orb and sparks
			return (_ln(31, 24, 37, 51, 3) +
				'<circle cx="29" cy="18" r="8" fill="{w}" stroke="{ink}" stroke-width="1.4"/><circle cx="32.5" cy="15.5" r="6.4" fill="{d}"/>'.format({"w": WHITE, "ink": INK, "d": dark}) +
				_ln(44, 18, 44, 24, 1.4) + _ln(41, 21, 47, 21, 1.4) + _ln(17, 30, 17, 34, 1.2) + _ln(15, 32, 19, 32, 1.2))
		"skirmisher":  # crossed short blades
			return (_ln(18, 18, 44, 44, 2.8) + _ln(46, 18, 20, 44, 2.8) +
				_ln(38, 44, 44, 38, 2.4) + _ln(20, 38, 26, 44, 2.4) +
				_ln(44, 44, 48, 48, 2.6) + _ln(20, 44, 16, 48, 2.6))
		"dummy":  # practice target
			return ('<circle cx="32" cy="32" r="15" fill="{w}" stroke="{ink}" stroke-width="1.4"/><circle cx="32" cy="32" r="10" fill="{d}"/><circle cx="32" cy="32" r="5" fill="{w}"/><circle cx="32" cy="32" r="2" fill="{d}"/>'.format({"w": WHITE, "ink": INK, "d": dark}))
	# Generic unit: rounded helmet
	return _shape("M20 38 C20 22 44 22 44 38 L44 44 L20 44 Z") + '<path d="M24 36 H40" stroke="{d}" stroke-width="2.4" stroke-linecap="round"/>'.format({"d": dark})

# ------------------------------------------------------------------------------
# Portraits
# ------------------------------------------------------------------------------

const CLASS_LOOKS: Dictionary = {
	"warrior": {"armor": "#94a3b8", "shade": "#475569", "label": "WARRIOR"},
	"ranger": {"armor": "#166534", "shade": "#14532d", "label": "RANGER"},
	"archer": {"armor": "#92400e", "shade": "#78350f", "label": "GOBLIN ARCHER"},
	"scout": {"armor": "#334155", "shade": "#1f2937", "label": "SCOUT"},
	"brawler": {"armor": "#7c2d12", "shade": "#431407", "label": "BRAWLER"},
	"berserker": {"armor": "#78350f", "shade": "#451a03", "label": "GOBLIN BERSERKER"},
	"shaman": {"armor": "#3f3f46", "shade": "#27272a", "label": "GOBLIN SHAMAN"},
	"skirmisher": {"armor": "#9a3412", "shade": "#7c2d12", "label": "GOBLIN SKIRMISHER"},
	"dummy": {"armor": "#a16207", "shade": "#713f12", "label": "TRAINING DUMMY"},
	"unit": {"armor": "#64748b", "shade": "#334155", "label": "UNIT"},
}

static func portrait(class_key: String, team: Team) -> Texture2D:
	var key: String = normalize_class(class_key)
	return SvgRaster.texture("portrait|%s|%d" % [key, team], portrait_svg(key, team), 2.0)

static func portrait_svg(class_key: String, team: Team) -> String:
	var pal: Dictionary = _palette(team)
	var look_raw: Variant = CLASS_LOOKS.get(class_key, CLASS_LOOKS["unit"])
	var look: Dictionary = look_raw if look_raw is Dictionary else {}
	var goblin: bool = GOBLIN_CLASSES.has(class_key)
	var skin: String = "#7cb342" if goblin else "#e0b393"
	var skin_shade: String = "#558b2f" if goblin else "#c08a5e"
	var eye: String = "#fde047" if goblin else INK

	var body: String
	if class_key == "dummy":
		body = ('<rect x="56" y="68" width="8" height="52" fill="#78350f"/><rect x="30" y="74" width="60" height="9" rx="3" fill="#92400e" stroke="#451a03" stroke-width="1.5"/>' +
			'<circle cx="60" cy="50" r="22" fill="#d97706" stroke="#fef08a" stroke-width="2"/><circle cx="60" cy="50" r="14" fill="#b45309" stroke="#fed7aa" stroke-width="1.5"/>' +
			'<circle cx="60" cy="50" r="6" fill="#dc2626"/><path d="M50 40 L70 60 M70 40 L50 60" stroke="#fde68a" stroke-width="2"/>')
	else:
		var ears: String = ""
		if goblin:
			ears = ('<path d="M44 50 L26 40 L43 58 Z" fill="{s}" stroke="{ss}" stroke-width="1.2"/><path d="M76 50 L94 40 L77 58 Z" fill="{s}" stroke="{ss}" stroke-width="1.2"/>').format({"s": skin, "ss": skin_shade})
		body = ('{behind}' +
			'<path d="M14 120 C14 92 32 80 60 80 C88 80 106 92 106 120 Z" fill="{armor}" stroke="{shade}" stroke-width="2"/>' +
			'<path d="M40 120 C42 100 50 90 60 90 C70 90 78 100 80 120 Z" fill="{shade}" opacity="0.55"/>' +
			'<rect x="53" y="66" width="14" height="16" rx="4" fill="{skin_shade}"/>' +
			'{ears}<ellipse cx="60" cy="54" rx="17" ry="19" fill="{skin}" stroke="{skin_shade}" stroke-width="1.5"/>' +
			'<ellipse cx="53" cy="54" rx="2.6" ry="3" fill="{eye}"/><ellipse cx="67" cy="54" rx="2.6" ry="3" fill="{eye}"/>' +
			'<path d="M54 64 Q60 67 66 64" fill="none" stroke="{skin_shade}" stroke-width="1.6" stroke-linecap="round"/>' +
			'{gear}').format({
				"behind": _portrait_behind(class_key), "armor": look.get("armor", "#64748b"), "shade": look.get("shade", "#334155"),
				"skin": skin, "skin_shade": skin_shade, "ears": ears, "eye": eye, "gear": _portrait_gear(class_key, pal),
			})

	var svg: String = """<svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120">
<defs><radialGradient id="bg" cx="0.5" cy="0.35" r="0.8"><stop offset="0" stop-color="{light}" stop-opacity="0.55"/><stop offset="1" stop-color="{dark}"/></radialGradient></defs>
<rect x="1" y="1" width="118" height="118" rx="10" fill="#0f172a"/>
<rect x="1" y="1" width="118" height="118" rx="10" fill="url(#bg)" stroke="{rim}" stroke-width="2"/>
<ellipse cx="60" cy="118" rx="46" ry="8" fill="#000000" opacity="0.3"/>
{body}
<rect x="18" y="104" width="84" height="13" rx="5" fill="{dark}" stroke="{rim}" stroke-width="1"/>
<text x="60" y="113.5" font-family="sans-serif" font-size="8" font-weight="bold" fill="#ffffff" text-anchor="middle">{label}</text>
</svg>"""
	return svg.format({"light": pal["light"], "dark": pal["dark"], "rim": pal["rim"], "body": body, "label": look.get("label", "UNIT")})

# Weapons / props drawn behind the body.
static func _portrait_behind(class_key: String) -> String:
	match class_key:
		"warrior":
			return '<path d="M84 96 L104 26" stroke="#cbd5e1" stroke-width="6" stroke-linecap="round"/><path d="M93 58 L103 61" stroke="#d97706" stroke-width="5" stroke-linecap="round"/>'
		"ranger", "archer":
			return '<path d="M22 22 Q52 64 26 108" fill="none" stroke="#92400e" stroke-width="5" stroke-linecap="round"/><path d="M22 22 L26 108" stroke="#e7e5e4" stroke-width="1"/>'
		"berserker":
			return '<path d="M88 108 L102 30" stroke="#78350f" stroke-width="5" stroke-linecap="round"/><path d="M98 30 C112 26 116 40 110 50 C106 44 100 42 96 44 Z" fill="#a8a29e" stroke="#44403c" stroke-width="1.5"/>'
		"shaman":
			return '<path d="M96 112 L90 34" stroke="#57534e" stroke-width="5" stroke-linecap="round"/><circle cx="89" cy="28" r="9" fill="#c084fc" stroke="#f5d0fe" stroke-width="2"/><circle cx="89" cy="28" r="15" fill="#c084fc" opacity="0.25"/>'
		"skirmisher":
			return '<path d="M16 110 L98 20" stroke="#78350f" stroke-width="4" stroke-linecap="round"/><path d="M98 20 L92 30 L102 26 Z" fill="#cbd5e1" stroke="#475569" stroke-width="1"/>'
	return ""

# Headgear / accessories drawn over the head and torso.
static func _portrait_gear(class_key: String, pal: Dictionary) -> String:
	var accent: String = str(pal["light"])
	match class_key:
		"warrior":
			return ('<path d="M41 56 C41 30 79 30 79 56 L79 66 C72 72 48 72 41 66 Z" fill="#94a3b8" stroke="#334155" stroke-width="2"/>' +
				'<rect x="45" y="51" width="30" height="6" rx="2" fill="#0f172a"/><path d="M47 54 H57 M63 54 H73" stroke="#67e8f9" stroke-width="2" stroke-linecap="round"/>' +
				'<path d="M57 34 C55 22 66 18 68 30 C64 28 60 30 60 36 Z" fill="{a}"/>'.format({"a": accent}) +
				'<circle cx="32" cy="90" r="11" fill="#94a3b8" stroke="#334155" stroke-width="2"/><circle cx="88" cy="90" r="11" fill="#94a3b8" stroke="#334155" stroke-width="2"/>' +
				'<path d="M60 92 L66 102 L60 112 L54 102 Z" fill="{a}"/>'.format({"a": accent}))
		"ranger":
			return ('<path d="M38 62 C34 30 86 30 82 62 L78 76 C70 70 50 70 42 76 Z" fill="#15803d" stroke="#14532d" stroke-width="2"/>' +
				'<path d="M44 62 C46 44 74 44 76 62" fill="none" stroke="#052e16" stroke-width="3" opacity="0.5"/>' +
				'<path d="M34 86 L84 116" stroke="#78350f" stroke-width="4"/><circle cx="60" cy="88" r="4" fill="{a}"/>'.format({"a": accent}))
		"archer":
			return ('<path d="M42 48 C42 30 78 30 78 48 C70 44 50 44 42 48 Z" fill="#78350f" stroke="#451a03" stroke-width="2"/>' +
				'<path d="M30 92 L90 112" stroke="#451a03" stroke-width="3"/><circle cx="60" cy="100" r="4" fill="{a}"/>'.format({"a": accent}))
		"scout":
			return ('<path d="M42 58 L78 58 L76 70 C70 74 50 74 44 70 Z" fill="#1e293b"/><path d="M41 44 C48 38 72 38 79 44 L79 48 L41 48 Z" fill="{a}"/>'.format({"a": accent}) +
				'<path d="M46 100 L62 84 M74 100 L58 84" stroke="#cbd5e1" stroke-width="3" stroke-linecap="round"/><path d="M44 102 L49 97 M76 102 L71 97" stroke="#78350f" stroke-width="4" stroke-linecap="round"/>')
		"brawler":
			return ('<path d="M46 66 Q60 78 74 66 L72 72 Q60 82 48 72 Z" fill="#451a03"/>' +
				'<circle cx="36" cy="104" r="10" fill="#e7e5e4" stroke="#78716c" stroke-width="2"/><circle cx="84" cy="104" r="10" fill="#e7e5e4" stroke="#78716c" stroke-width="2"/>' +
				'<path d="M30 100 L42 100 M78 100 L90 100" stroke="{a}" stroke-width="2"/>'.format({"a": accent}))
		"berserker":
			return ('<path d="M41 50 C41 32 79 32 79 50 Z" fill="#57534e" stroke="#292524" stroke-width="2"/>' +
				'<path d="M42 42 C30 38 26 26 30 18 C34 30 40 34 46 36 Z" fill="#f5f5f4" stroke="#78716c" stroke-width="1.5"/>' +
				'<path d="M78 42 C90 38 94 26 90 18 C86 30 80 34 74 36 Z" fill="#f5f5f4" stroke="#78716c" stroke-width="1.5"/>' +
				'<path d="M34 88 L86 112 M86 88 L34 112" stroke="#451a03" stroke-width="4"/><circle cx="60" cy="100" r="5" fill="{a}"/>'.format({"a": accent}))
		"shaman":
			return ('<path d="M40 44 L36 22 L46 38 L50 16 L56 36 L60 12 L64 36 L70 16 L74 38 L84 22 L80 44 Z" fill="{a}" stroke="#0f172a" stroke-width="1.5"/>'.format({"a": accent}) +
				'<path d="M42 44 H78" stroke="#e7e5e4" stroke-width="4"/>' +
				'<path d="M38 88 Q60 104 82 88" fill="none" stroke="#f5f5f4" stroke-width="3" stroke-dasharray="3 3"/>' +
				'<path d="M60 90 L60 118" stroke="#a855f7" stroke-width="4"/>')
		"skirmisher":
			return ('<path d="M42 50 C42 32 78 32 78 50 L74 52 L60 46 L46 52 Z" fill="#64748b" stroke="#1e293b" stroke-width="2"/><path d="M60 46 L60 62" stroke="#475569" stroke-width="3"/>' +
				'<path d="M36 92 L84 92" stroke="#451a03" stroke-width="4"/><circle cx="60" cy="92" r="4" fill="{a}"/>'.format({"a": accent}))
	return ""
