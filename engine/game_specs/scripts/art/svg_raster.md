---
class: SvgRaster
extends: RefCounted
description: "Tactical RPG - SVG Rasterizer & Texture Cache Generated art (units, tiles, item icons) is authored as SVG templates and rasterized at runtime. Textures are cached by key so each variant is built once."
---

# Methods

### `static texture(key: String, svg: String, scale: float = 1.0) -> Texture2D`
```logic
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
```

### `static clear_cache() -> void`
```logic
_cache.clear()
AssetRegistry.clear_warnings()
AssetRegistry.clear_family_cache()
```

### `static cached_texture(key: String) -> Texture2D`
```logic
var hit: Variant = _cache.get(key, null)
return hit if hit is Texture2D else null
```

### `static store(key: String, tex: Texture2D) -> void`
```logic
_cache[key] = tex
```

### `static hash01(x: int, y: int, salt: int = 0) -> float`
```logic
var h: int = (x * 73856093) ^ (y * 19349663) ^ (salt * 83492791)
h = (h ^ (h >> 13)) * 1274126177
h = h ^ (h >> 16)
return float(absi(h) % 10000) / 10000.0
```

