class_name AssetRegistry
extends RefCounted

# ==============================================================================
# Tactical RPG - Asset Registry (S14, spec S14-L1): one place where an art / sound KEY resolves
# Content names art by key (class/unit/world character `art` -> unit, item `icon_id` -> item). A key resolves to
# (a) its manifest row (AssetCatalog; for tiles also the v0 row; an alias row resolves its target once), else (b) the
# generated SVG template, else (c) the namespace fallback key. Resolution never returns nothing for art.
# Folder pattern for row files: res://assets/art/<ns>/<key>_<variant>.<png|svg>, res://assets/audio/sfx/<key>.<ogg|wav>,
# res://assets/audio/music/<key>.ogg (ui: png only).
# ==============================================================================

const NAMESPACES: Array[String] = ["unit", "item", "tile", "skill", "status", "sfx", "music", "ui"]
const VARIANTS: Dictionary = {
	"unit": ["token", "portrait"], "item": ["icon"], "tile": ["v0", "v1", "v2", "v3"], "skill": ["icon"],
	"status": ["icon"], "sfx": ["stream"], "music": ["stream"], "ui": ["icon", "background"],
}
const AUDIO_NAMESPACES: Array[String] = ["sfx", "music"]
# The key whose template stands in for an unknown key ("" = blank texture for ui, silence for sound).
const FALLBACK_KEYS: Dictionary = {"unit": "unit", "item": "weapon", "tile": "floor", "skill": "skill", "status": "status", "sfx": "", "music": "", "ui": ""}
const EXTENSIONS: Dictionary = {
	"unit": ["png", "svg"], "item": ["png", "svg"], "tile": ["png", "svg"], "skill": ["png", "svg"], "status": ["png", "svg"],
	"sfx": ["ogg", "wav"], "music": ["ogg"], "ui": ["png"],
}
# Native file sizes (the placeholder raster sizes, so swapping keeps layouts). A unit token FILE is the 104x104 inner
# square; the composed token texture is TOKEN_SIZE (the template rim drawn by code around the file).
const DEFAULT_SIZES: Dictionary = {
	"unit/token": Vector2i(104, 104), "unit/portrait": Vector2i(240, 240), "item/icon": Vector2i(96, 96),
	"tile": Vector2i(108, 108), "skill/icon": Vector2i(64, 64), "status/icon": Vector2i(64, 64),
}
const TOKEN_SIZE: Vector2i = Vector2i(128, 128)
const TOKEN_INNER: Rect2i = Rect2i(12, 12, 104, 104)
const TILE_OVERLAY_KEYS: Array[String] = ["overlay_exit", "overlay_poi"]

# Ids that already logged their "failed to load" warning this session, and how many warnings were logged.
static var _warned: Dictionary = {}
static var _warn_count: int = 0
static var _quiet: bool = false
# Skill family ids (template keys of the skill namespace), cached; see skill_family_ids.
static var _family_list: Array[String] = []
static var _family_set: Dictionary = {}
static var _family_stamp: String = ""

static func namespaces() -> Array[String]:
	return NAMESPACES.duplicate()

static func variants(ns: String) -> Array[String]:
	var out: Array[String] = []
	out.assign(DictUtil.array_of(VARIANTS.get(ns, [])))
	return out

static func is_texture_namespace(ns: String) -> bool:
	return NAMESPACES.has(ns) and not AUDIO_NAMESPACES.has(ns)

static func resource_type(ns: String) -> String:
	return "AudioStream" if AUDIO_NAMESPACES.has(ns) else "Texture2D"

static func extensions(ns: String) -> Array[String]:
	var out: Array[String] = []
	out.assign(DictUtil.array_of(EXTENSIONS.get(ns, [])))
	return out

static func folder_of(ns: String) -> String:
	return "res://assets/audio/%s" % ns if AUDIO_NAMESPACES.has(ns) else "res://assets/art/%s" % ns

# The paths a row of ns/key/variant may use (folder pattern), one per allowed extension.
static func pattern_paths(ns: String, key: String, variant: String) -> Array[String]:
	var out: Array[String] = []
	var stem: String = key if AUDIO_NAMESPACES.has(ns) else "%s_%s" % [key, variant]
	for ext: String in extensions(ns):
		out.append("%s/%s.%s" % [folder_of(ns), stem, ext])
	return out

# "res://assets/art/unit/warrior_token.png or .svg" (the text of the folder-pattern message).
static func pattern_text(ns: String, key: String, variant: String) -> String:
	var paths: Array[String] = pattern_paths(ns, key, variant)
	if paths.is_empty():
		return ""
	var text: String = paths[0]
	for i: int in range(1, paths.size()):
		text += " or .%s" % paths[i].get_extension()
	return text

# ------------------------------------------------------------------------------
# Keys
# ------------------------------------------------------------------------------

# Keys with a generated template (empty for sfx, music, ui). Fallback keys are drawn even when not listed.
static func template_keys(ns: String) -> Array[String]:
	var out: Array[String] = []
	match ns:
		"unit":
			out.assign(UnitArt.CLASS_KEYS)
		"item":
			out.assign(ItemArt.TEMPLATE_KEYS)
		"tile":
			for name: String in Terrain.Type.keys():
				out.append(name.to_lower())
			out.append_array(TILE_OVERLAY_KEYS)
		"skill":
			out = skill_family_ids()
		"status":
			out = StatusRegistry.ids()
	return out

# Every skill family (head) id of the content catalog, in file order. Cached per catalog root and row count; while the
# skills are edited (dirty) it is rebuilt on every call and the cache is dropped, so a saved edit is picked up.
static func skill_family_ids() -> Array[String]:
	_refresh_families()
	return _family_list.duplicate()

static func _refresh_families() -> void:
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

# Drops the skill family cache (SvgRaster.clear_cache calls it too).
static func clear_family_cache() -> void:
	_family_stamp = ""

# True when ns/key has a generated template (removing its manifest rows brings the placeholder back).
static func has_template(ns: String, key: String) -> bool:
	return _has_template(ns, key)

static func _has_template(ns: String, key: String) -> bool:
	match ns:
		"skill":
			_refresh_families()
			return _family_set.has(key)
		"status":
			return StatusRegistry.has(key)
	return template_keys(ns).has(key)

# Template keys plus manifest keys, sorted, unique.
static func keys(ns: String) -> Array[String]:
	var out: Array[String] = template_keys(ns)
	for key: String in AssetCatalog.keys_of(ns):
		if not out.has(key):
			out.append(key)
	out.sort()
	return out

static func has_key(ns: String, key: String) -> bool:
	if key.is_empty():
		return false
	return _has_template(ns, key) or AssetCatalog.keys_of(ns).has(key)

# Terrain.Type <-> tile key ("shallow_water"); -1 for keys that are not terrain types (overlays, unknown).
static func tile_key(terrain_type: Terrain.Type) -> String:
	return str(Terrain.Type.keys()[terrain_type]).to_lower()

static func tile_type(key: String) -> int:
	return Terrain.Type.keys().find(key.to_upper())

static func default_size(ns: String, variant: String) -> Vector2i:
	if ns == "tile":
		return DEFAULT_SIZES["tile"]
	var raw: Variant = DEFAULT_SIZES.get("%s/%s" % [ns, variant], Vector2i.ZERO)
	return raw if raw is Vector2i else Vector2i.ZERO

# ------------------------------------------------------------------------------
# Resolution
# ------------------------------------------------------------------------------

# The manifest row serving ns/key/variant ({} when none): its own row, else (tiles) the v0 row.
static func _row_for(ns: String, key: String, variant: String) -> Dictionary:
	var row: Dictionary = AssetCatalog.get_entry(AssetCatalog.id_for(ns, key, variant))
	if row.is_empty() and ns == "tile" and variant != "v0":
		row = AssetCatalog.get_entry(AssetCatalog.id_for(ns, key, "v0"))
	return row

# {"source": "file" | "template" | "fallback", "key": the key after alias, "path": file path ("" unless file), "id": row id}.
# A row whose file is missing logs one warning per id per session and resolves as if it had no row.
static func resolve(ns: String, key: String, variant: String) -> Dictionary:
	var k: String = key
	var row: Dictionary = _row_for(ns, k, variant)
	var alias: String = str(row.get("alias_of", ""))
	if not alias.is_empty():
		k = alias
		row = _row_for(ns, k, variant)
		if not str(row.get("alias_of", "")).is_empty():
			row = {}   # one hop only (validation refuses chains; a hand-edited file falls back to the template)
	var path: String = str(row.get("path", ""))
	if not path.is_empty():
		if file_exists(path):
			return {"source": "file", "key": k, "path": path, "id": str(row.get("id", ""))}
		warn_once(str(row.get("id", "")))
	if _has_template(ns, k):
		return {"source": "template", "key": k, "path": "", "id": ""}
	return {"source": "fallback", "key": str(FALLBACK_KEYS.get(ns, "")), "path": "", "id": ""}

# The texture of ns/key/variant (rule 1); never null for texture namespaces (null for sfx / music). `team` is the
# UnitArt.Team for units and the rarity for items (0 when negative); other namespaces ignore it. A row file that fails
# to load warns once per id and the template is used. File textures cache as "file|<id>|<team>" in SvgRaster.
static func texture(ns: String, key: String, variant: String, team: int = -1) -> Texture2D:
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
		if not _has_template(ns, k):
			k = str(FALLBACK_KEYS.get(ns, ""))
	return _template_texture(ns, k, v, team)

# The sound of ns/key (the `stream` variant); null when it does not resolve to a loadable file (sound is optional).
static func stream(ns: String, key: String) -> AudioStream:
	if not AUDIO_NAMESPACES.has(ns):
		return null
	var r: Dictionary = resolve(ns, key, "stream")
	if str(r["source"]) != "file":
		return null
	var res: Resource = load_file(ns, str(r["path"]))
	if res is AudioStream:
		return res
	warn_once(str(r["id"]))
	return null

static func _file_texture(ns: String, variant: String, team: int, id: String, path: String) -> Texture2D:
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

# Rule 3: unit files are drawn by code onto the team template (token: the glyph-less token at 128 px with the file
# resized to the 104x104 inner square; portrait: the team backdrop at 240 px with the file at full size), so one file
# serves all three teams and the spiked / dashed rims stay.
static func _compose_unit(variant: String, team: int, file_img: Image) -> Texture2D:
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

# The generated template (today's SvgRaster cache keys), or the blank texture for ui.
static func _template_texture(ns: String, key: String, variant: String, team: int) -> Texture2D:
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
	return _blank()

static func _blank() -> Texture2D:
	var hit: Texture2D = SvgRaster.cached_texture("blank|ui")
	if hit != null:
		return hit
	var tex: Texture2D = ImageTexture.create_from_image(Image.create(8, 8, false, Image.FORMAT_RGBA8))
	SvgRaster.store("blank|ui", tex)
	return tex

# Logs "AssetRegistry: '<id>' failed to load; using the placeholder." once per id per session.
static func warn_once(id: String) -> void:
	if _warned.has(id):
		return
	_warned[id] = true
	_warn_count += 1
	if not _quiet:
		push_warning("AssetRegistry: '%s' failed to load; using the placeholder." % id)

# Tests that break files on purpose count the warnings without printing them (check_project.ps1 reads any line with
# "fail" in it, case-insensitive, as a failed assertion).
static func set_quiet(quiet: bool) -> void:
	_quiet = quiet

# Warnings logged since the last clear_warnings (tests).
static func warning_count() -> int:
	return _warn_count

static func clear_warnings() -> void:
	_warned.clear()
	_warn_count = 0

# ------------------------------------------------------------------------------
# Files
# ------------------------------------------------------------------------------

static func file_exists(path: String) -> bool:
	var real: String = AssetCatalog.file_of(path)
	return FileAccess.file_exists(real) or (real.begins_with("res://") and ResourceLoader.exists(real))

# Loads a row file as the namespace's resource type (null when it does not load as one). Imported res:// files go
# through ResourceLoader; files outside the import system (test sandboxes) are read directly.
static func load_file(ns: String, path: String) -> Resource:
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

# An image file read directly (png or svg at its own size); null when it is not a readable image.
static func load_image(real_path: String) -> Image:
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

# ------------------------------------------------------------------------------
# Templates (the generated placeholders; rasterised at TEMPLATE_SCALE)
# ------------------------------------------------------------------------------

const TEMPLATE_SCALE: float = 2.0

# The SVG of the generated template for ns/key/variant ("" when the namespace has none). `team` is the UnitArt.Team for
# units and the rarity for items (0 when negative). An unknown key draws its namespace fallback.
static func template_svg(ns: String, key: String, variant: String, team: int = -1) -> String:
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
	return ""

static func _team_of(team: int) -> UnitArt.Team:
	return clampi(team, 0, UnitArt.Team.size() - 1) as UnitArt.Team

# The template rasterised to an Image (null for namespaces without templates).
static func template_image(ns: String, key: String, variant: String, team: int = -1) -> Image:
	var svg: String = template_svg(ns, key, variant, team)
	if svg.is_empty():
		return null
	var img := Image.new()
	if img.load_svg_from_buffer(svg.to_utf8_buffer(), TEMPLATE_SCALE) != OK:
		return null
	return img

# Writes the template of ns/key/variant as a PNG to its folder-pattern path (below the asset root) so real art can start
# from today's placeholder. For unit tokens only the glyph layer, cropped to the 104x104 inner square (the part a token
# file replaces). Returns {"path": row path, "file": real file, "size": Vector2i, "error": "" or why not}.
static func export_placeholder(ns: String, key: String, variant: String, team: int = 0) -> Dictionary:
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
