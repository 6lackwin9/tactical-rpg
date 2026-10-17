class_name SvgRaster
extends RefCounted

# ==============================================================================
# Tactical RPG - SVG Rasterizer & Texture Cache
# Generated art (units, tiles, item icons) is authored as SVG templates and
# rasterized at runtime. Textures are cached by key so each variant is built once.
# ==============================================================================

static var _cache: Dictionary = {} # String -> Texture2D

static func texture(key: String, svg: String, scale: float = 1.0) -> Texture2D:
	var cache_key: String = "%s@%.2f" % [key, scale]
	var cached: Variant = _cache.get(cache_key, null)
	if cached is Texture2D:
		var hit: Texture2D = cached
		return hit
	var img := Image.new()
	if img.load_svg_from_buffer(svg.to_utf8_buffer(), scale) != OK:
		push_error("SvgRaster: failed to rasterize '%s'" % key)
		return null
	var tex: Texture2D = ImageTexture.create_from_image(img)
	_cache[cache_key] = tex
	return tex

# Also forgets the AssetRegistry "failed to load" warnings (a cleared cache warns again once per id) and its skill
# family cache.
static func clear_cache() -> void:
	_cache.clear()
	AssetRegistry.clear_warnings()
	AssetRegistry.clear_family_cache()

# A texture built elsewhere (AssetRegistry file art, "file|<id>|<team>") under `key` exactly; null when not cached.
static func cached_texture(key: String) -> Texture2D:
	var hit: Variant = _cache.get(key, null)
	return hit if hit is Texture2D else null

static func store(key: String, tex: Texture2D) -> void:
	_cache[key] = tex

# Deterministic 0..1 noise from integer coordinates + salt (for tile variation).
static func hash01(x: int, y: int, salt: int = 0) -> float:
	var h: int = (x * 73856093) ^ (y * 19349663) ^ (salt * 83492791)
	h = (h ^ (h >> 13)) * 1274126177
	h = h ^ (h >> 16)
	return float(absi(h) % 10000) / 10000.0
