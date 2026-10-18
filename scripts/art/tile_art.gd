class_name TileArt
extends RefCounted

# ==============================================================================
# Tactical RPG - Terrain Tile Art
# Procedural SVG tiles (54x54) per Terrain.Type with 4 seeded variants each, so
# zones look natural instead of tiled, plus the exit overlay that sits on
# top of any terrain. Variants are chosen deterministically from the coordinate.
# ==============================================================================

const VARIANTS: int = 4

static func tile_at(terrain_type: Terrain.Type, coord: Vector2i) -> Texture2D:
	var variant: int = int(SvgRaster.hash01(coord.x, coord.y, 7) * VARIANTS) % VARIANTS
	return tile(terrain_type, variant)

# Through AssetRegistry (key = snake_case type name, variant v0-v3): a manifest file (a v0 file serves every variant
# without its own), else the template.
static func tile(terrain_type: Terrain.Type, variant: int = 0) -> Texture2D:
	return AssetRegistry.texture("tile", AssetRegistry.tile_key(terrain_type), "v%d" % variant)

# Overlay keys are "overlay_exit" (combat exit tiles) and "overlay_poi" (any other kind: the point-of-interest
# ring, drawn on region map point markers; it was the 9x9 centre-ring landmark until the 12x9 grid). MAPREAD1-L2: the other
# region map marker icons (npc, chest, chest_open, lore, exit_locked, encounter, roamer) are MapArt overlays.
static func overlay(kind: String) -> Texture2D:
	var key: String = "overlay_%s" % kind
	return AssetRegistry.texture("tile", key if AssetRegistry.TILE_OVERLAY_KEYS.has(key) else "overlay_poi", "v0")

# The generated tile / overlay (cached "tile|<type>|<variant>" / "overlay|<kind>"); used by AssetRegistry.
static func template_tile(terrain_type: Terrain.Type, variant: int) -> Texture2D:
	return SvgRaster.texture("tile|%d|%d" % [terrain_type, variant], tile_svg(terrain_type, variant), 2.0)

static func template_overlay(kind: String) -> Texture2D:
	return SvgRaster.texture("overlay|%s" % kind, overlay_svg(kind), 2.0)

static func _wrap(body: String) -> String:
	return ('<svg xmlns="http://www.w3.org/2000/svg" width="54" height="54" viewBox="0 0 54 54">' + body +
		'<rect x="0.5" y="0.5" width="53" height="53" rx="3" fill="none" stroke="#000000" stroke-opacity="0.35" stroke-width="1"/></svg>')

static func _base(top: String, bottom: String) -> String:
	return ('<defs><linearGradient id="b" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{t}"/><stop offset="1" stop-color="{b}"/></linearGradient></defs>' +
		'<rect width="54" height="54" rx="3" fill="url(#b)"/>').format({"t": top, "b": bottom})

# Deterministic pseudo-random value for detail i of a variant.
static func _r(variant: int, i: int, salt: int) -> float:
	return SvgRaster.hash01(variant + 11, i, salt)

static func _scatter(variant: int, count: int, salt: int, fmt: Callable) -> String:
	var parts: Array[String] = []
	for i: int in range(count):
		var x: float = 5.0 + _r(variant, i, salt) * 44.0
		var y: float = 5.0 + _r(variant, i, salt + 1) * 44.0
		var s: float = _r(variant, i, salt + 2)
		parts.append(str(fmt.call(x, y, s)))
	return "".join(parts)

static func tile_svg(t: Terrain.Type, v: int) -> String:
	match t:
		Terrain.Type.GRASS:
			return _wrap(_base("#3f7a35", "#2d5a27") + _grass_tufts(v, 7))
		Terrain.Type.DIRT:
			return _wrap(_base("#7a5634", "#5e4127") + _scatter(v, 6, 20, func(x: float, y: float, s: float) -> String:
				return '<ellipse cx="%.1f" cy="%.1f" rx="%.1f" ry="%.1f" fill="%s"/>' % [x, y, 1.2 + s * 1.8, 1.0 + s * 1.2, "#9b7650" if s > 0.5 else "#4a3220"]))
		Terrain.Type.STONE:
			return _wrap(_base("#4b5563", "#3a4250") + _cobbles(v))
		Terrain.Type.SAND:
			return _wrap(_base("#d6b77a", "#c3a061") + _scatter(v, 3, 30, func(x: float, y: float, _s: float) -> String:
				return '<path d="M%.1f %.1f q6 -3 12 0 t12 0" fill="none" stroke="#b08d4e" stroke-width="1.2" opacity="0.8"/>' % [x - 10.0, y]))
		Terrain.Type.SNOW:
			return _wrap(_base("#e8f0f6", "#cfdde8") + _scatter(v, 3, 40, func(x: float, y: float, _s: float) -> String:
				return '<path d="M%.1f %.1f q7 -4 14 0" fill="none" stroke="#a9bfd2" stroke-width="1.4"/>' % [x - 7.0, y]) +
				_scatter(v, 5, 45, func(x: float, y: float, s: float) -> String:
					return '<circle cx="%.1f" cy="%.1f" r="%.1f" fill="#ffffff"/>' % [x, y, 0.6 + s]))
		Terrain.Type.SHALLOW_WATER:
			return _wrap(_base("#3a87b3", "#2a6f97") + _waves(v, "#9fd8f0", 0.75))
		Terrain.Type.DEEP_WATER:
			return _wrap(_base("#16507a", "#0c3150") + _waves(v, "#3b82b6", 0.6) + _scatter(v, 2, 55, func(x: float, y: float, _s: float) -> String:
				return '<path d="M%.1f %.1f h4" stroke="#cdeaf7" stroke-width="1.2" stroke-linecap="round"/>' % [x, y]))
		Terrain.Type.MUD:
			return _wrap(_base("#54432f", "#3f3222") + _scatter(v, 4, 60, func(x: float, y: float, s: float) -> String:
				return '<ellipse cx="%.1f" cy="%.1f" rx="%.1f" ry="%.1f" fill="#2f2518"/><ellipse cx="%.1f" cy="%.1f" rx="%.1f" ry="0.8" fill="#7a6a52" opacity="0.6"/>' % [x, y, 4.0 + s * 4.0, 2.0 + s * 2.0, x - 1.0, y - 1.0, 1.5 + s * 2.0]))
		Terrain.Type.LAVA:
			return _wrap('<defs><radialGradient id="l" cx="0.5" cy="0.5" r="0.7"><stop offset="0" stop-color="#fdba74"/><stop offset="0.5" stop-color="#f97316"/><stop offset="1" stop-color="#b91c1c"/></radialGradient></defs><rect width="54" height="54" rx="3" fill="url(#l)"/>' +
				_scatter(v, 4, 70, func(x: float, y: float, s: float) -> String:
					return '<path d="M%.1f %.1f l%.1f -4 l6 3 l-2 7 l-8 1 Z" fill="#3f0d0d" stroke="#fb923c" stroke-width="0.8"/>' % [x - 4.0, y, 4.0 + s * 3.0]))
		Terrain.Type.WALL:
			return _wrap(_base("#3b4351", "#262c36") +
				'<path d="M4 16 L18 6 L40 8 L50 18 L48 44 L34 50 L12 48 L5 38 Z" fill="#4b5563" stroke="#1f2937" stroke-width="1.5"/>' +
				'<path d="M8 18 L19 10 L38 12 L45 20" fill="none" stroke="#9ca3af" stroke-width="1.5" opacity="0.7"/>' +
				_scatter(v, 2, 80, func(x: float, y: float, _s: float) -> String:
					return '<path d="M%.1f %.1f l5 4 l-2 6" fill="none" stroke="#1f2937" stroke-width="1.2"/>' % [clampf(x, 12, 40), clampf(y, 18, 40)]))
		Terrain.Type.TREE:
			return _wrap(_base("#3f7a35", "#2d5a27") + _grass_tufts(v, 3) +
				'<ellipse cx="29" cy="44" rx="15" ry="5" fill="#000000" opacity="0.3"/>' +
				'<circle cx="20" cy="24" r="11" fill="#166534"/><circle cx="34" cy="20" r="12" fill="#15803d"/><circle cx="29" cy="33" r="12" fill="#14532d"/>' +
				'<circle cx="31" cy="17" r="4" fill="#4ade80" opacity="0.35"/>')
		Terrain.Type.BRIDGE:
			return _wrap(_base("#3a87b3", "#2a6f97") + _waves(v, "#9fd8f0", 0.5) +
				'<rect x="0" y="12" width="54" height="30" fill="#8b5a2b"/>' +
				'<path d="M9 12 V42 M18 12 V42 M27 12 V42 M36 12 V42 M45 12 V42" stroke="#5c3a1a" stroke-width="1.5"/>' +
				'<rect x="0" y="10" width="54" height="3" fill="#a16b38"/><rect x="0" y="41" width="54" height="3" fill="#a16b38"/>')
		Terrain.Type.RUBBLE:
			return _wrap(_base("#2b3445", "#1e2533") + _scatter(v, 5, 90, func(x: float, y: float, s: float) -> String:
				var r: float = 2.5 + s * 3.5
				return '<path d="M%.1f %.1f l%.1f -%.1f l%.1f %.1f l-%.1f %.1f Z" fill="#6b7280" stroke="#374151" stroke-width="1"/>' % [x, y, r, r, r, r * 0.6, r * 1.4, r * 0.4]))
		Terrain.Type.BURNING_GROUND:   # S12T-L hazard: dark soil, orange ember dots
			return _wrap(_base("#3a2a1f", "#241912") + _scatter(v, 7, 110, func(x: float, y: float, s: float) -> String:
				return '<circle cx="%.1f" cy="%.1f" r="%.1f" fill="%s"/><circle cx="%.1f" cy="%.1f" r="%.1f" fill="#fb923c" opacity="0.25"/>' % [x, y, 0.9 + s * 1.3, "#fdba74" if s > 0.6 else "#f97316", x, y, 2.5 + s * 2.0]))
		Terrain.Type.TOXIC_BOG:   # S12T-L hazard: green-grey mud, purple bubbles
			return _wrap(_base("#4a5544", "#353f31") + _scatter(v, 3, 120, func(x: float, y: float, s: float) -> String:
				return '<ellipse cx="%.1f" cy="%.1f" rx="%.1f" ry="%.1f" fill="#2c3428"/>' % [x, y, 4.0 + s * 3.0, 2.0 + s * 1.5]) +
				_scatter(v, 5, 125, func(x: float, y: float, s: float) -> String:
					return '<circle cx="%.1f" cy="%.1f" r="%.1f" fill="#a855f7" fill-opacity="0.55" stroke="#d8b4fe" stroke-width="0.7"/>' % [x, y, 1.0 + s * 2.2]))
		Terrain.Type.ICE:   # S12T-L hazard: pale blue, white cracks
			return _wrap(_base("#cfe8f5", "#a7d0e8") + _scatter(v, 3, 130, func(x: float, y: float, s: float) -> String:
				return '<path d="M%.1f %.1f l%.1f 5 l-3 6 M%.1f %.1f l6 -2" fill="none" stroke="#ffffff" stroke-width="1.2" stroke-linecap="round" opacity="0.9"/>' % [x, y, 3.0 + s * 4.0, x + 2.0, y + 3.0]) +
				'<path d="M8 10 L20 8" stroke="#ffffff" stroke-width="2" stroke-linecap="round" opacity="0.6"/>')
	# FLOOR (default arena tile)
	return _wrap(_base("#1f2937", "#18202d") + '<rect x="3" y="3" width="48" height="48" rx="3" fill="none" stroke="#2c3a4f" stroke-width="1"/>' +
		_scatter(v, 1, 99, func(x: float, y: float, _s: float) -> String:
			return '<path d="M%.1f %.1f l4 3 l-1 5" fill="none" stroke="#111827" stroke-width="1"/>' % [x, y]))

static func _grass_tufts(v: int, count: int) -> String:
	return _scatter(v, count, 10, func(x: float, y: float, s: float) -> String:
		var c: String = "#6fae4f" if s > 0.5 else "#4d8a3a"
		return '<path d="M%.1f %.1f l-2 -5 M%.1f %.1f l0 -6 M%.1f %.1f l2 -5" stroke="%s" stroke-width="1.3" stroke-linecap="round"/>' % [x, y, x, y, x, y, c])

static func _cobbles(v: int) -> String:
	var parts: Array[String] = []
	for row: int in range(3):
		for col: int in range(3):
			var jitter: float = _r(v, row * 3 + col, 50) * 3.0
			var w: float = 13.0 + _r(v, row * 3 + col, 51) * 3.0
			parts.append('<rect x="%.1f" y="%.1f" width="%.1f" height="13" rx="4" fill="%s" stroke="#2d333d" stroke-width="1"/>' % [
				3.0 + col * 16.5 + jitter * 0.3, 3.0 + row * 16.5 + jitter * 0.2, w, "#6b7280" if (row + col + v) % 2 == 0 else "#5b6472"])
	return "".join(parts)

static func _waves(v: int, color: String, opacity: float) -> String:
	return _scatter(v, 4, 33, func(x: float, y: float, _s: float) -> String:
		return '<path d="M%.1f %.1f q3 -3 6 0 t6 0" fill="none" stroke="%s" stroke-width="1.3" opacity="%.2f"/>' % [x - 6.0, y, color, opacity])

static func overlay_svg(kind: String) -> String:
	if kind == "exit":
		return ('<svg xmlns="http://www.w3.org/2000/svg" width="54" height="54" viewBox="0 0 54 54">' +
			'<rect x="2" y="2" width="50" height="50" rx="4" fill="#f59e0b" fill-opacity="0.18" stroke="#f59e0b" stroke-width="2.5"/>' +
			'<path d="M17 32 L27 22 L37 32 M17 40 L27 30 L37 40" fill="none" stroke="#fbbf24" stroke-width="3.5" stroke-linecap="round" stroke-linejoin="round"/></svg>')
	if MapArt.has_overlay(kind):
		return MapArt.overlay_svg(kind)
	# point-of-interest ring
	return ('<svg xmlns="http://www.w3.org/2000/svg" width="54" height="54" viewBox="0 0 54 54">' +
		'<circle cx="27" cy="27" r="18" fill="#22d3ee" fill-opacity="0.08" stroke="#22d3ee" stroke-width="2" stroke-opacity="0.8"/>' +
		'<path d="M27 15 L39 27 L27 39 L15 27 Z" fill="none" stroke="#67e8f9" stroke-width="1.8" stroke-opacity="0.9"/>' +
		'<circle cx="27" cy="27" r="3" fill="#67e8f9"/></svg>')
