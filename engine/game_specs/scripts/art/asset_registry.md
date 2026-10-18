---
class: AssetRegistry
extends: RefCounted
description: "Tactical RPG - Asset Registry (S14, spec S14-L1): one place where an art / sound KEY resolves Content names art by key (class/unit/world character `art` -> unit, item `icon_id` -> item). A key resolves to (a) its manifest row (AssetCatalog; for tiles also the v0 row; an alias row resolves its target once), else (b) the generated SVG template, else (c) the namespace fallback key. Resolution never returns nothing for art. Folder pattern for row files: res://assets/art/<ns>/<key>_<variant>.<png|svg>, res://assets/audio/sfx/<key>.<ogg|wav>, res://assets/audio/music/<key>.ogg (ui: png only)."
---

# Properties
* `NAMESPACES`: Array[String] = ["unit", "item", "tile", "skill", "status", "sfx", "music", "ui"] (const)
* `VARIANTS`: Dictionary = { (const)
* `AUDIO_NAMESPACES`: Array[String] = ["sfx", "music"] (const)
* `FALLBACK_KEYS`: Dictionary = {"unit": "unit", "item": "weapon", "tile": "floor", "skill": "skill", "status": "status", "sfx": "", "music": "", "ui": ""} (const)
* `EXTENSIONS`: Dictionary = { (const)
* `DEFAULT_SIZES`: Dictionary = { (const)
* `TOKEN_SIZE`: Vector2i = Vector2i(128, 128) (const)
* `TOKEN_INNER`: Rect2i = Rect2i(12, 12, 104, 104) (const)
* `TILE_OVERLAY_KEYS`: Array[String] = ["overlay_exit", "overlay_poi", "overlay_npc", "overlay_chest", "overlay_chest_open", "overlay_lore", "overlay_exit_locked", "overlay_encounter", "overlay_roamer"]   # the last seven: MapArt (MAPREAD1-L2) (const)
* `out`: Array[String] = []
* `out`: Array[String] = []
* `out`: Array[String] = []
* `stem`: String = key if AUDIO_NAMESPACES.has(ns) else "%s_%s" % [key, variant]
* `out`: Array[String] = []
* `out`: Array[String] = template_keys(ns)
* `TEMPLATE_SCALE`: float = 2.0 (const)

# Methods

### `static is_texture_namespace(ns: String) -> bool`
```logic
return NAMESPACES.has(ns) and not AUDIO_NAMESPACES.has(ns)
```

### `static resource_type(ns: String) -> String`
```logic
return "AudioStream" if AUDIO_NAMESPACES.has(ns) else "Texture2D"
```

### `static folder_of(ns: String) -> String`
```logic
return "res://assets/audio/%s" % ns if AUDIO_NAMESPACES.has(ns) else "res://assets/art/%s" % ns
```

### `static pattern_text(ns: String, key: String, variant: String) -> String`
```logic
var paths: Array[String] = pattern_paths(ns, key, variant)
if paths.is_empty():
	return ""
var text: String = paths[0]
for i: int in range(1, paths.size()):
	text += " or .%s" % paths[i].get_extension()
return text
```

### `static _refresh_families() -> void`
```logic
var rows: Array[Dictionary] = ContentCatalog.entries("skills")
var stamp: String = "%s|%d" % [ContentCatalog.get_root(), rows.size()]
var dirty: bool = ContentCatalog.is_dirty("skills")
if not dirty and stamp == _family_stamp:
	return
_family_list.clear()
_family_set.clear()
for row: Dictionary in rows:
	var head: String = SkillRanks.head_of(row)
	if not _family_set.has(head):
		_family_set[head] = true
		_family_list.append(head)
_family_stamp = "" if dirty else stamp
```

### `static clear_family_cache() -> void`
```logic
_family_stamp = ""
```

### `static has_template(ns: String, key: String) -> bool`
```logic
return _has_template(ns, key)
```

### `static _has_template(ns: String, key: String) -> bool`
```logic
match ns:
	"skill":
		_refresh_families()
		return _family_set.has(key)
	"status":
		return StatusRegistry.has(key)
return template_keys(ns).has(key)
```

### `static has_key(ns: String, key: String) -> bool`
```logic
if key.is_empty():
	return false
return _has_template(ns, key) or AssetCatalog.keys_of(ns).has(key)
```

### `static tile_key(terrain_type: Terrain.Type) -> String`
```logic
return str(Terrain.Type.keys()[terrain_type]).to_lower()
```

### `static tile_type(key: String) -> int`
```logic
return Terrain.Type.keys().find(key.to_upper())
```

### `static default_size(ns: String, variant: String) -> Vector2i`
```logic
if ns == "tile":
	return DEFAULT_SIZES["tile"]
var raw: Variant = DEFAULT_SIZES.get("%s/%s" % [ns, variant], Vector2i.ZERO)
return raw if raw is Vector2i else Vector2i.ZERO
```

### `static _row_for(ns: String, key: String, variant: String) -> Dictionary`
```logic
var row: Dictionary = AssetCatalog.get_entry(AssetCatalog.id_for(ns, key, variant))
if row.is_empty() and ns == "tile" and variant != "v0":
	row = AssetCatalog.get_entry(AssetCatalog.id_for(ns, key, "v0"))
return row
```

### `static resolve(ns: String, key: String, variant: String) -> Dictionary`
```logic
var k: String = key
var row: Dictionary = _row_for(ns, k, variant)
var alias: String = str(row.get("alias_of", ""))
if not alias.is_empty():
	k = alias
	row = _row_for(ns, k, variant)
	if not str(row.get("alias_of", "")).is_empty():
		row = {}   # one hop only (validation refuses chains; a hand-edited file falls back to the template)
var path: String = str(row.get("path", ""))
var missing: bool = false
if not path.is_empty():
	if file_exists(path):
		return {"source": "file", "key": k, "path": path, "id": str(row.get("id", ""))}
	warn_once(str(row.get("id", "")))
	missing = true
if _has_template(ns, k):
	_observe(ns, key, variant, "missing_file" if missing else "template")
	return {"source": "template", "key": k, "path": "", "id": ""}
_observe(ns, key, variant, "missing_file" if missing else "fallback")
return {"source": "fallback", "key": str(FALLBACK_KEYS.get(ns, "")), "path": "", "id": ""}
```

### `static _observe(ns: String, key: String, variant: String, source: String) -> void`
```logic
if resolve_observer.is_valid():
	resolve_observer.call(ns, key, variant, source)
```

### `static texture(ns: String, key: String, variant: String, team: int = -1) -> Texture2D`
```logic
if not is_texture_namespace(ns):
	return null
var allowed: Array[String] = variants(ns)
var v: String = variant if allowed.has(variant) else allowed[0]
var r: Dictionary = resolve(ns, key, v)
var k: String = str(r["key"])
if str(r["source"]) == "file":
	var tex: Texture2D = _file_texture(ns, v, team, str(r["id"]), str(r["path"]))
	if tex != null:
		return tex
	warn_once(str(r["id"]))
	_observe(ns, key, v, "load_failed")
	if not _has_template(ns, k):
		k = str(FALLBACK_KEYS.get(ns, ""))
return _template_texture(ns, k, v, team)
```

### `static stream(ns: String, key: String) -> AudioStream`
```logic
if not AUDIO_NAMESPACES.has(ns):
	return null
var r: Dictionary = resolve(ns, key, "stream")
if str(r["source"]) != "file":
	return null
var res: Resource = load_file(ns, str(r["path"]))
if res is AudioStream:
	return res
warn_once(str(r["id"]))
_observe(ns, key, "stream", "load_failed")
return null
```

### `static _file_texture(ns: String, variant: String, team: int, id: String, path: String) -> Texture2D`
```logic
var cache_key: String = "file|%s|%d" % [id, team]
var hit: Texture2D = SvgRaster.cached_texture(cache_key)
if hit != null:
	return hit
var res: Resource = load_file(ns, path)
if not (res is Texture2D):
	return null
var tex: Texture2D = res
if ns == "unit":
	tex = _compose_unit(variant, team, tex.get_image())
	if tex == null:
		return null
SvgRaster.store(cache_key, tex)
return tex
```

### `static _compose_unit(variant: String, team: int, file_img: Image) -> Texture2D`
```logic
if file_img == null or file_img.is_empty():
	return null
var side: UnitArt.Team = _team_of(team)
var svg: String = UnitArt.portrait_backdrop_svg(side) if variant == "portrait" else UnitArt.token_svg("unit", side, false)
var base := Image.new()
if base.load_svg_from_buffer(svg.to_utf8_buffer(), TEMPLATE_SCALE) != OK:
	return null
base.convert(Image.FORMAT_RGBA8)
var top: Image = file_img.duplicate() as Image
if top.is_compressed() and top.decompress() != OK:   # imported (VRAM-compressed) textures cannot be converted or blended
	return null
top.convert(Image.FORMAT_RGBA8)
var area: Rect2i = Rect2i(Vector2i.ZERO, base.get_size()) if variant == "portrait" else TOKEN_INNER
if top.get_size() != area.size:
	top.resize(area.size.x, area.size.y, Image.INTERPOLATE_BILINEAR)
base.blend_rect(top, Rect2i(Vector2i.ZERO, area.size), area.position)
return ImageTexture.create_from_image(base)
```

### `static _template_texture(ns: String, key: String, variant: String, team: int) -> Texture2D`
```logic
match ns:
	"unit":
		var side: UnitArt.Team = _team_of(team)
		return UnitArt.template_portrait(key, side) if variant == "portrait" else UnitArt.template_token(key, side)
	"item":
		var rarity: int = maxi(team, 0)
		return SvgRaster.texture("item|%s|%d" % [key, rarity], ItemArt.icon_svg(key, rarity), TEMPLATE_SCALE)
	"tile":
		if TILE_OVERLAY_KEYS.has(key):
			return TileArt.template_overlay(key.trim_prefix("overlay_"))
		return TileArt.template_tile(maxi(tile_type(key), 0) as Terrain.Type, maxi(variants("tile").find(variant), 0))
	"skill":
		return SkillArt.template(key)
	"status":
		return StatusArt.template(key)
	"ui":
		if variant == "icon" and MapArt.has_ui(key):
			return SvgRaster.texture("ui|%s" % key, MapArt.ui_svg(key), TEMPLATE_SCALE)
return _blank()
```

### `static _blank() -> Texture2D`
```logic
var hit: Texture2D = SvgRaster.cached_texture("blank|ui")
if hit != null:
	return hit
var tex: Texture2D = ImageTexture.create_from_image(Image.create(8, 8, false, Image.FORMAT_RGBA8))
SvgRaster.store("blank|ui", tex)
return tex
```

### `static warn_once(id: String) -> void`
```logic
if _warned.has(id):
	return
_warned[id] = true
_warn_count += 1
if not _quiet:
	push_warning("AssetRegistry: '%s' failed to load; using the placeholder." % id)
```

### `static set_quiet(quiet: bool) -> void`
```logic
_quiet = quiet
```

### `static warning_count() -> int`
```logic
return _warn_count
```

### `static clear_warnings() -> void`
```logic
_warned.clear()
_warn_count = 0
```

### `static file_exists(path: String) -> bool`
```logic
var real: String = AssetCatalog.file_of(path)
return FileAccess.file_exists(real) or (real.begins_with("res://") and ResourceLoader.exists(real))
```

### `static load_file(ns: String, path: String) -> Resource`
```logic
var real: String = AssetCatalog.file_of(path)
var ext: String = real.get_extension().to_lower()
if real.begins_with("res://") and ResourceLoader.exists(real):
	var res: Resource = load(real)
	if AUDIO_NAMESPACES.has(ns):
		return res if res is AudioStream else null
	return res if res is Texture2D else null
if not FileAccess.file_exists(real):
	return null
if AUDIO_NAMESPACES.has(ns):
	if ext == "ogg":
		return AudioStreamOggVorbis.load_from_file(real)
	if ext == "wav":
		return AudioStreamWAV.load_from_file(real)
	return null
var img: Image = load_image(real)
return ImageTexture.create_from_image(img) if img != null else null
```

### `static load_image(real_path: String) -> Image`
```logic
var bytes: PackedByteArray = FileAccess.get_file_as_bytes(real_path)
if bytes.is_empty():
	return null
var img := Image.new()
var err: Error = ERR_FILE_UNRECOGNIZED
match real_path.get_extension().to_lower():
	"svg":
		err = img.load_svg_from_buffer(bytes)
	"png":
		err = img.load_png_from_buffer(bytes)
return img if err == OK and not img.is_empty() else null
```

### `static template_svg(ns: String, key: String, variant: String, team: int = -1) -> String`
```logic
var k: String = key if _has_template(ns, key) else str(FALLBACK_KEYS.get(ns, ""))
match ns:
	"unit":
		var side: UnitArt.Team = _team_of(team)
		return UnitArt.portrait_svg(k, side) if variant == "portrait" else UnitArt.token_svg(k, side)
	"item":
		return ItemArt.icon_svg(k, maxi(team, 0))
	"tile":
		if TILE_OVERLAY_KEYS.has(k):
			return TileArt.overlay_svg(k.trim_prefix("overlay_"))
		return TileArt.tile_svg(maxi(tile_type(k), 0) as Terrain.Type, maxi(variants("tile").find(variant), 0))
	"skill":
		return SkillArt.icon_svg(k)
	"status":
		return StatusArt.icon_svg(k)
	"ui":
		return MapArt.ui_svg(k) if variant == "icon" else ""
return ""
```

### `static template_image(ns: String, key: String, variant: String, team: int = -1) -> Image`
```logic
var svg: String = template_svg(ns, key, variant, team)
if svg.is_empty():
	return null
var img := Image.new()
if img.load_svg_from_buffer(svg.to_utf8_buffer(), TEMPLATE_SCALE) != OK:
	return null
return img
```

### `static export_placeholder(ns: String, key: String, variant: String, team: int = 0) -> Dictionary`
```logic
var out: Dictionary = {"path": "", "file": "", "size": Vector2i.ZERO, "error": ""}
if not is_texture_namespace(ns) or not variants(ns).has(variant):
	out["error"] = "No template for %s/%s/%s." % [ns, key, variant]
	return out
var img: Image
if ns == "unit" and variant == "token":
	var glyph := Image.new()
	if glyph.load_svg_from_buffer(UnitArt.glyph_svg(key if _has_template(ns, key) else "unit", _team_of(team)).to_utf8_buffer(), TEMPLATE_SCALE) == OK:
		img = glyph.get_region(TOKEN_INNER)
else:
	img = template_image(ns, key, variant, team)
if img == null:
	out["error"] = "No template for %s/%s/%s." % [ns, key, variant]
	return out
var path: String = "%s/%s_%s.png" % [folder_of(ns), key, variant]
var real: String = AssetCatalog.file_of(path)
DirAccess.make_dir_recursive_absolute(real.get_base_dir())
var err: Error = img.save_png(real)
if err != OK:
	out["error"] = "Could not write %s (error %d)." % [real, err]
	return out
out["path"] = path
out["file"] = real
out["size"] = img.get_size()
return out
```

