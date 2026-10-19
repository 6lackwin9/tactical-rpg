class_name WorldForgeSource
extends ForgeSource

# The World Forge's source: WorldSchema fields; references resolve from the world
# being edited (its own entries, its ancestors', or the base catalog).

var world_id: String = ""

func _init(p_world_id: String = "") -> void:
	world_id = p_world_id

func fields(kind: String) -> Array[Dictionary]:
	return WorldSchema.fields(kind)

func normalize(kind: String, entry: Dictionary) -> Dictionary:
	return WorldSchema.normalize(kind, entry)

func options_of(f: Dictionary) -> Array[String]:
	return WorldSchema.options_of(f)

func ref_choices(f: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var ref_kind: String = str(f.get("ref", ""))
	if ref_kind == "worlds":
		for w: String in WorldCatalog.world_ids():
			if w != world_id:
				out.append({"value": w, "label": ref_label(f, w)})
	elif ref_kind.begins_with("base:"):
		for e: Dictionary in ContentCatalog.entries(ref_kind.trim_prefix("base:")):
			if ContentSchema.passes_filter(f, e):
				var id: String = str(e.get("id", ""))
				out.append({"value": id, "label": ref_label(f, id)})
	else:
		for e: Dictionary in WorldCatalog.merged_entries(world_id, ref_kind):
			if ContentSchema.passes_filter(f, e):
				var value: String = WorldCatalog.ref_value(world_id, e)
				out.append({"value": value, "label": ref_label(f, value)})
	return out

func ref_label(f: Dictionary, value: String) -> String:
	if value.is_empty():
		return "(none)"
	var ref_kind: String = str(f.get("ref", ""))
	if ref_kind == "worlds":
		var m: Dictionary = WorldCatalog.manifest(value)
		return "%s  [%s]" % [str(m.get("name", "?")), value] if not m.is_empty() else "%s (missing)" % value
	if ref_kind.begins_with("base:"):
		var base: Dictionary = ContentCatalog.get_entry(ref_kind.trim_prefix("base:"), value)
		return "%s  [%s]" % [str(base.get("name", "?")), value] if not base.is_empty() else "%s (missing)" % value
	var hit: Dictionary = WorldCatalog.resolve(world_id, ref_kind, value)
	if hit.is_empty():
		return "%s (missing)" % value
	var origin: String = "" if str(hit["_world"]) == world_id else "  ↑ %s" % str(hit["_world"])
	return "%s  [%s]%s" % [str(hit.get("name", "?")), str(hit.get("id", "")), origin]
