class_name ItemArt
extends RefCounted

# ==============================================================================
# Tactical RPG - Item Icons
# Baseline icon per gear type (sword, greatsword, hammer, bow, shield, plate,
# ring, potion, ...) inside a frame colored by rarity (rare and above glow).
# Items can set `Item.icon_id` to any item art key (a bespoke icon in
# UNIQUE_ICONS, a type icon or a manifest key); everything else falls back to
# the type icon automatically.
# ==============================================================================

const RARITY_COLORS: Array[String] = ["#d4d4d8", "#66e680", "#59bfff", "#bf73ff", "#ffcc33"]
const INK: String = "#0b1220"
const STEEL: String = "#e2e8f0"
const WOOD: String = "#a16207"
const GOLD: String = "#fbbf24"

# Bespoke icons for unique / rare items (Item.icon_id -> description). Add the
# drawing to _unique_glyph() when registering a new id.
const UNIQUE_ICONS: Dictionary = {
	"titan_warhammer": "Rune-charged colossal warhammer",
	"runed_staff": "Twisted staff with a glowing crystal",
}

# Every key with a generated icon (AssetRegistry "item" templates): the _glyph keys, the fallbacks and UNIQUE_ICONS.
const TEMPLATE_KEYS: Array[String] = [
	"sword", "greatsword", "shortsword", "dagger", "hammer", "axe", "bow", "staff", "wand", "tome", "knuckles", "shield",
	"plate", "leather", "robe", "ring", "amulet", "potion", "elixir", "bomb", "weapon", "misc", "titan_warhammer", "runed_staff",
]

static func _unique_glyph(icon_id: String) -> String:
	match icon_id:
		"titan_warhammer":
			return _glyph_titan_warhammer()
		"runed_staff":
			return _glyph_runed_staff()
	return _glyph("weapon")

# The item's icon through the registry (S14-L1 rule 4): a manifest row for its key wins, else the generated template.
static func icon(item: Item) -> Texture2D:
	if item == null:
		return null
	return AssetRegistry.texture("item", icon_key(item), "icon", int(item.rarity))

# The icon key an item resolves to (S14-L1 rule 5): its icon_id when that is a known item art key (bespoke, type or
# manifest key; wpn_shortsword keeps its icon through icon_id "shortsword"), else the key derived from its gear type.
static func icon_key(item: Item) -> String:
	if AssetRegistry.has_key("item", item.icon_id):
		return item.icon_id
	if item is Consumable:
		var con: Consumable = item
		match con.consumable_type:
			Consumable.ConsumableType.HEALTH_POTION:
				return "potion"
			Consumable.ConsumableType.THROWABLE_BOMB:
				return "bomb"
		return "elixir"
	if not (item is Equipment):
		return "misc"
	var eq: Equipment = item
	if eq.is_shield():
		return "shield"
	match eq.item_type:
		Item.ItemType.ARMOR:
			match eq.weapon_type:
				"plate":
					return "plate"
				"cloth":
					return "robe"
			return "leather"
		Item.ItemType.ACCESSORY:
			return "amulet" if "amulet" in eq.name.to_lower() else "ring"
	match eq.weapon_type:
		"sword":
			if eq.is_two_handed:
				return "greatsword"
			return "sword"
		"hammer", "mace":
			return "hammer"
		"axe":
			return "axe"
		"bow", "crossbow":
			return "bow"
		"dagger", "knife":
			return "dagger"
		"staff":
			return "staff"
		"wand":
			return "wand"
		"orb", "tome":
			return "tome"
		"fist":
			return "knuckles"
	return "weapon"

static func icon_svg(key: String, rarity: int) -> String:
	var frame: String = RARITY_COLORS[clampi(rarity, 0, RARITY_COLORS.size() - 1)]
	var glow: String = ""
	if rarity >= 2:
		glow = '<rect x="3" y="3" width="42" height="42" rx="7" fill="none" stroke="{f}" stroke-width="5" stroke-opacity="0.25"/>'.format({"f": frame})
	var glyph: String = _unique_glyph(key) if UNIQUE_ICONS.has(key) else _glyph(key)
	return ("""<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48">
<defs><radialGradient id="bg" cx="0.5" cy="0.4" r="0.75"><stop offset="0" stop-color="#1f2a3c"/><stop offset="1" stop-color="#0b1220"/></radialGradient></defs>
<rect x="2" y="2" width="44" height="44" rx="7" fill="url(#bg)" stroke="{f}" stroke-width="2"/>{glow}
{glyph}
</svg>""").format({"f": frame, "glow": glow, "glyph": glyph})

static func _ln(x1: float, y1: float, x2: float, y2: float, w: float, color: String) -> String:
	return ('<path d="M{x1} {y1} L{x2} {y2}" stroke="{ink}" stroke-width="{wo}" stroke-linecap="round"/><path d="M{x1} {y1} L{x2} {y2}" stroke="{c}" stroke-width="{w}" stroke-linecap="round"/>').format({
		"x1": x1, "y1": y1, "x2": x2, "y2": y2, "w": w, "wo": w + 2.0, "ink": INK, "c": color})

static func _shape(d: String, fill: String) -> String:
	return '<path d="{d}" fill="{f}" stroke="{ink}" stroke-width="1.3" stroke-linejoin="round"/>'.format({"d": d, "f": fill, "ink": INK})

static func _glyph(key: String) -> String:
	match key:
		"sword":
			return _ln(17, 31, 36, 12, 3.2, STEEL) + _ln(13, 27, 21, 35, 2.6, GOLD) + _ln(17, 31, 12, 36, 2.6, WOOD) + '<circle cx="11" cy="37" r="2.4" fill="{g}" stroke="{i}"/>'.format({"g": GOLD, "i": INK})
		"greatsword":
			return _ln(15, 33, 39, 9, 5, STEEL) + _ln(10, 28, 20, 38, 3, GOLD) + _ln(15, 33, 9, 39, 3, WOOD) + _ln(19, 29, 35, 13, 1, "#94a3b8")
		"shortsword":
			return _ln(19, 29, 33, 15, 3, STEEL) + _ln(15, 25, 23, 33, 2.4, "#94a3b8") + _ln(19, 29, 14, 34, 2.6, WOOD)
		"dagger":
			return _shape("M30 10 L34 14 L22 28 L18 26 Z", STEEL) + _ln(15, 23, 23, 31, 2.4, GOLD) + _ln(19, 27, 13, 33, 2.6, WOOD)
		"hammer":
			return _ln(15, 37, 29, 17, 3, WOOD) + _shape("M22 12 L34 20 L30 26 L18 18 Z", "#94a3b8") + _ln(23, 15, 31, 21, 1, STEEL)
		"axe":
			return _ln(16, 38, 30, 12, 3, WOOD) + _shape("M27 14 C36 12 40 20 37 28 C34 24 30 22 25 21 Z", STEEL)
		"bow":
			return ('<path d="M16 9 Q38 24 16 39" fill="none" stroke="{i}" stroke-width="5" stroke-linecap="round"/><path d="M16 9 Q38 24 16 39" fill="none" stroke="{w}" stroke-width="3" stroke-linecap="round"/><path d="M16 9 L16 39" stroke="{s}" stroke-width="1"/>'.format({"i": INK, "w": WOOD, "s": STEEL}) +
				_ln(12, 24, 34, 24, 1.8, STEEL) + _shape("M38 24 L32 21 L32 27 Z", STEEL))
		"staff":
			return _ln(16, 40, 30, 14, 3, WOOD) + '<circle cx="32" cy="12" r="5.5" fill="#a855f7" stroke="{i}" stroke-width="1.3"/><circle cx="30.5" cy="10.5" r="1.8" fill="#f5d0fe"/>'.format({"i": INK})
		"wand":
			return _ln(15, 38, 31, 18, 2.4, WOOD) + _shape("M33 9 L35 14 L40 14 L36 17.5 L37.5 23 L33 19.5 L28.5 23 L30 17.5 L26 14 L31 14 Z", "#67e8f9")
		"tome":
			return _shape("M13 12 H33 C35 12 36 13 36 15 V36 H15 C14 36 13 35 13 34 Z", "#6d28d9") + '<path d="M15 36 C14 36 13 35 13 33 C13 32 14 31 15 31 H36" fill="none" stroke="{s}" stroke-width="1.4"/><path d="M24 17 L28 23 L24 29 L20 23 Z" fill="{g}"/>'.format({"s": STEEL, "g": GOLD})
		"knuckles":
			return (_shape("M11 20 H37 V30 H11 Z", "#94a3b8") +
				'<circle cx="16" cy="25" r="2.6" fill="{i}"/><circle cx="22" cy="25" r="2.6" fill="{i}"/><circle cx="28" cy="25" r="2.6" fill="{i}"/><circle cx="34" cy="25" r="2.6" fill="{i}"/>'.format({"i": INK}) +
				_shape("M14 20 L16 14 L18 20 Z M20 20 L22 14 L24 20 Z M26 20 L28 14 L30 20 Z M32 20 L34 14 L36 20 Z", STEEL))
		"shield":
			return _shape("M24 9 L37 13 L37 23 C37 31 31 37 24 40 C17 37 11 31 11 23 L11 13 Z", "#475569") + _shape("M24 13 L33 16 L33 23 C33 29 29 33 24 35.5 Z", "#94a3b8") + '<circle cx="24" cy="23" r="3" fill="{g}" stroke="{i}"/>'.format({"g": GOLD, "i": INK})
		"plate":
			return _shape("M14 12 L20 10 Q24 14 28 10 L34 12 L37 20 L33 22 L33 38 Q24 41 15 38 L15 22 L11 20 Z", "#94a3b8") + '<path d="M24 16 V36 M18 24 H30" stroke="#475569" stroke-width="1.6"/>'
		"leather":
			return _shape("M15 11 L21 10 Q24 13 27 10 L33 11 L36 19 L32 21 L32 38 H16 L16 21 L12 19 Z", "#92400e") + '<path d="M24 14 V36 M21 18 L27 18 M21 23 L27 23 M21 28 L27 28" stroke="#fcd34d" stroke-width="1.2"/>'
		"robe":
			return _shape("M18 10 H30 L36 38 H12 Z", "#4c1d95") + '<path d="M24 10 V38" stroke="{g}" stroke-width="1.5"/>'.format({"g": GOLD})
		"ring":
			return '<circle cx="24" cy="27" r="9" fill="none" stroke="{i}" stroke-width="6"/><circle cx="24" cy="27" r="9" fill="none" stroke="{g}" stroke-width="3.5"/>'.format({"i": INK, "g": GOLD}) + _shape("M24 11 L29 16 L24 21 L19 16 Z", "#ef4444")
		"amulet":
			return '<path d="M13 10 Q24 26 35 10" fill="none" stroke="{g}" stroke-width="1.6" stroke-dasharray="2 1.5"/>'.format({"g": GOLD}) + _shape("M24 22 L31 29 L24 38 L17 29 Z", "#22d3ee") + '<path d="M24 25 L27 29 L24 33 L21 29 Z" fill="#cffafe"/>'
		"potion":
			return _shape("M21 10 H27 V17 C33 19 36 24 36 29 C36 35 31 39 24 39 C17 39 12 35 12 29 C12 24 15 19 21 17 Z", "#cbd5e1") + _shape("M14 28 H34 C34 34 30 37 24 37 C18 37 14 34 14 28 Z", "#ef4444") + '<rect x="20" y="8" width="8" height="3" rx="1" fill="{w}"/>'.format({"w": WOOD})
		"elixir":
			return _shape("M20 9 H28 V14 L31 18 V37 C31 39 30 40 28 40 H20 C18 40 17 39 17 37 V18 L20 14 Z", "#cbd5e1") + _shape("M18.5 24 H29.5 V37 C29.5 38 29 38.5 28 38.5 H20 C19 38.5 18.5 38 18.5 37 Z", "#a855f7") + '<path d="M22 28 L26 32 M26 28 L22 32" stroke="{g}" stroke-width="1.4"/>'.format({"g": GOLD})
		"bomb":
			return '<circle cx="22" cy="28" r="11" fill="#1f2937" stroke="{i}" stroke-width="1.5"/><circle cx="18" cy="24" r="3" fill="#475569"/>'.format({"i": INK}) + _ln(29, 19, 34, 12, 2, WOOD) + '<path d="M36 9 L38 6 M37 12 L40 12 M34 8 L33 5" stroke="#f97316" stroke-width="1.6" stroke-linecap="round"/>'
	return _ln(14, 34, 34, 14, 3, STEEL)

# --- Bespoke unique icons -----------------------------------------------------

static func _glyph_titan_warhammer() -> String:
	return ('<circle cx="24" cy="24" r="17" fill="#38bdf8" opacity="0.12"/>' +
		_ln(13, 40, 28, 16, 3.5, "#78350f") +
		_shape("M16 9 L37 21 L32 30 L11 18 Z", "#64748b") + _shape("M18 12 L34 21 L31 26 L15 17 Z", "#94a3b8") +
		'<path d="M22 15 L25 19 L23 20 L26 24" fill="none" stroke="#67e8f9" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>' +
		_ln(16, 9, 11, 18, 1.6, GOLD) + _ln(37, 21, 32, 30, 1.6, GOLD) +
		'<circle cx="13" cy="40" r="2.6" fill="{g}" stroke="{i}"/>'.format({"g": GOLD, "i": INK}))

static func _glyph_runed_staff() -> String:
	return ('<circle cx="32" cy="12" r="10" fill="#c084fc" opacity="0.18"/>' +
		'<path d="M14 41 C18 35 16 31 21 27 C25 24 23 20 28 17" fill="none" stroke="{i}" stroke-width="5" stroke-linecap="round"/><path d="M14 41 C18 35 16 31 21 27 C25 24 23 20 28 17" fill="none" stroke="#7c2d12" stroke-width="3" stroke-linecap="round"/>'.format({"i": INK}) +
		'<path d="M17 34 h2 M20 29 h2 M23 23 h2" stroke="#e879f9" stroke-width="1.4" stroke-linecap="round"/>' +
		_shape("M32 4 L37 11 L33 20 L28 12 Z", "#e9d5ff") + '<path d="M32 6 L33 18" stroke="#a855f7" stroke-width="1"/>')
