extends SceneTree
# ART1-A seed (idempotent): manifest rows for every placeholder art file made by tools/gen_art.py, the Ashlands
# character `art` keys, the item icon link for the Ember Brigandine, and four 48x30 Ashlands map templates.
# Run: python3 tools/gen_art.py && tools/fc.sh run data/seeds/art1_a.gd   (import PNGs first: godot --headless --import)

const NAMESPACES: Array[String] = ["unit", "item", "tile", "skill", "status", "ui"]
var failed: bool = false

func _chk(label: String, errs: Array[String]) -> void:
	if not errs.is_empty():
		failed = true
		print("ERROR ", label, ": ", errs)

func _split(ns: String, stem: String) -> Array:
	for v: String in AssetRegistry.variants(ns):
		if stem.ends_with("_" + v):
			return [stem.trim_suffix("_" + v), v]
	return []

func _manifest() -> void:
	AssetCatalog.reload_from_disk()
	var rows: int = 0
	for ns: String in NAMESPACES:
		var dir: String = AssetRegistry.folder_of(ns)
		var files: PackedStringArray = DirAccess.get_files_at(dir)
		for f: String in files:
			if f.get_extension() != "png":
				continue
			var kv: Array = _split(ns, f.get_basename())
			if kv.is_empty():
				print("SKIP (no variant) ", f)
				continue
			var img: Image = AssetRegistry.load_image(AssetCatalog.file_of(dir + "/" + f))
			if img == null or img.is_empty():
				print("SKIP (unreadable) ", f)
				continue
			var key: String = str(kv[0])
			var variant: String = str(kv[1])
			var entry: Dictionary = {
				"id": AssetCatalog.id_for(ns, key, variant), "namespace": ns, "key": key, "variant": variant,
				"path": dir + "/" + f, "alias_of": "", "size": [img.get_width(), img.get_height()], "origin": "user",
				"note": "generated placeholder (ART1)",
			}
			if str(AssetCatalog.get_entry(str(entry["id"])).get("origin", "")) == "system":
				continue   # shipped row: only its file was regenerated
			_chk(str(entry["id"]), AssetCatalog.upsert(entry))
			rows += 1
	print("manifest rows upserted: ", rows)
	print("assets validate_all: ", AssetCatalog.validate_all())
	print("assets save: ", AssetCatalog.save())

func _characters() -> void:
	WorldCatalog.reload_from_disk()
	for c: Dictionary in WorldCatalog.own_entries("ashlands", "characters"):
		var id: String = str(c["id"])
		var e: Dictionary = WorldCatalog.get_own("ashlands", "characters", id)
		e["art"] = id
		_chk("char " + id, WorldCatalog.upsert("ashlands", "characters", e))
	var it: Dictionary = WorldCatalog.get_own("ashlands", "items", "ember_brigandine")
	if not it.is_empty():
		it["icon_id"] = "ember_brigandine"
		_chk("item ember_brigandine", WorldCatalog.upsert("ashlands", "items", it))

# --- map templates -----------------------------------------------------------

func _new(w: int, h: int) -> Array:
	var g: Array = []
	for y: int in h:
		var row: Array = []
		for x: int in w:
			row.append("#" if x == 0 or y == 0 or x == w - 1 or y == h - 1 else ".")
		g.append(row)
	return g

func _box(g: Array, x0: int, y0: int, x1: int, y1: int, ch: String) -> void:
	for y: int in range(y0, y1 + 1):
		for x: int in range(x0, x1 + 1):
			(g[y] as Array)[x] = ch

func _rows(g: Array) -> Array[String]:
	var out: Array[String] = []
	for row: Array in g:
		out.append("".join(row))
	return out

func _zone_rows(w: int, h: int, split: int) -> Array[String]:
	var out: Array[String] = []
	for y: int in h:
		out.append("s".repeat(split) + "d".repeat(w - split))
	return out

func _anchors(list: Array, g: Array) -> Array:
	var out: Array = []
	for a: Array in list:
		(g[int(a[2])] as Array)[int(a[1])] = "."
		out.append({"id": a[0], "x": a[1], "y": a[2]})
	return out

func _hpath(g: Array, y: int, x0: int, x1: int) -> Array:
	var p: Array = []
	for x: int in range(x0, x1 + 1):
		(g[y] as Array)[x] = "."
		p.append([x, y])
	return p

func _tpl(id: String, nm: String, g: Array, zones: Array[String], legend: Dictionary, anchors: Array, paths: Dictionary) -> Dictionary:
	return {"id": id, "name": nm, "rows": _rows(g), "zones": zones, "zone_legend": legend, "anchors": anchors, "paths": paths, "tags": ["ashlands"]}

func _gorge_anchors(extra: Array) -> Array:
	var base: Array = [["spawn", 3, 14], ["west", 0, 14], ["west_in", 1, 14], ["east", 47, 14], ["east_in", 46, 14]]
	base.append_array(extra)
	return base

const GORGE_LEGEND: Dictionary = {"s": {"id": "camp", "kind": "safe"}, "d": {"id": "ruins", "kind": "danger"}}

func _maps() -> void:
	MapCatalog.reload_from_disk()
	# Warden Keep: walled keep yard, south gate, hearth court (zone town).
	var g: Array = _new(48, 30)
	_box(g, 4, 3, 13, 8, "#")
	_box(g, 34, 3, 43, 8, "#")
	_box(g, 4, 17, 12, 23, "#")
	_box(g, 35, 17, 43, 23, "#")
	_box(g, 22, 14, 26, 16, "~")
	var za: Array[String] = []
	for y: int in 30:
		za.append("s".repeat(48))
	var keep_anchors: Array = [["spawn", 24, 27], ["gate", 24, 29], ["road", 2, 27], ["npc_a", 14, 12], ["npc_b", 33, 12], ["npc_c", 24, 2], ["npc_d", 45, 12], ["chest_a", 2, 2], ["lore_a", 24, 12]]
	_chk("keep", MapCatalog.upsert(_tpl("ashlands_warden_keep", "Ashlands: Warden Keep yard", g, za, {"s": {"id": "town", "kind": "safe"}}, _anchors(keep_anchors, g), {})))
	# Ruined Courtyard: broken arcades, a dry pool, patrol along the south colonnade.
	g = _new(48, 30)
	for x: int in [14, 22, 30, 38]:
		_box(g, x, 6, x + 1, 7, "#")
		_box(g, x, 21, x + 1, 22, "#")
	_box(g, 18, 10, 20, 10, "#")
	_box(g, 25, 10, 28, 10, "#")
	_box(g, 18, 17, 21, 17, "#")
	_box(g, 36, 4, 40, 7, "~")
	_box(g, 5, 22, 8, 25, "~")
	var pa: Dictionary = {}
	var anc: Array = _anchors(_gorge_anchors([["camp_npc", 3, 6], ["camp_chest", 2, 25], ["site_a", 30, 14], ["site_b", 40, 14], ["altar", 43, 3], ["ruin_chest", 44, 25], ["ruin_lore", 20, 24]]), g)
	pa["patrol"] = _hpath(g, 26, 16, 32)
	_chk("courtyard", MapCatalog.upsert(_tpl("ashlands_ruined_courtyard", "Ashlands: Ruined Courtyard", g, _zone_rows(48, 30, 10), GORGE_LEGEND, anc, pa)))
	# Cinder Gate: a gorge band with cliff shoulders and fallen blocks, the toll arch at the east.
	g = _new(48, 30)
	_box(g, 1, 1, 46, 8, "#")
	_box(g, 1, 21, 46, 28, "#")
	for x: int in [9, 15, 23, 33, 39]:
		_box(g, x, 9, x + 1, 10 + (x % 3), "#")
		_box(g, x + 2, 19 - (x % 2), x + 3, 20, "#")
	_box(g, 43, 12, 44, 16, "#")
	_box(g, 27, 13, 28, 14, "~")
	anc = _anchors(_gorge_anchors([["camp_npc", 3, 10], ["camp_chest", 2, 19], ["site_a", 30, 14], ["site_b", 40, 14], ["altar", 43, 10], ["ruin_chest", 44, 19], ["ruin_lore", 20, 18]]), g)
	pa = {"patrol": _hpath(g, 19, 12, 24)}
	_chk("gate", MapCatalog.upsert(_tpl("ashlands_cinder_gate", "Ashlands: Cinder Gate gorge", g, _zone_rows(48, 30, 10), GORGE_LEGEND, anc, pa)))
	# Ashmaw Camp: hide tents around the war fire, a tar pond on the east edge.
	g = _new(48, 30)
	for t: Array in [[12, 11], [34, 11], [18, 18], [30, 18], [12, 22], [38, 22], [26, 4], [8, 8]]:
		_box(g, int(t[0]), int(t[1]), int(t[0]) + 2, int(t[1]) + 2, "#")
	_box(g, 40, 3, 45, 6, "~")
	_box(g, 23, 13, 25, 15, "#")
	anc = _anchors(_gorge_anchors([["camp_npc", 3, 5], ["camp_chest", 2, 24], ["site_a", 26, 12], ["site_b", 24, 9], ["altar", 44, 9], ["ruin_chest", 44, 27], ["ruin_lore", 20, 25]]), g)
	pa = {"patrol": _hpath(g, 26, 10, 30)}
	_chk("camp", MapCatalog.upsert(_tpl("ashlands_ashmaw_camp", "Ashlands: Ashmaw Camp", g, _zone_rows(48, 30, 10), GORGE_LEGEND, anc, pa)))
	print("maps validate_all: ", MapCatalog.validate_all())
	print("maps save: ", MapCatalog.save())

func _init() -> void:
	_manifest()
	_characters()
	print("world validate_all: ", WorldCatalog.validate_all())
	print("world save: ", WorldCatalog.save_world("ashlands"))
	_maps()
	print("FAILED" if failed else "OK")
	quit()
