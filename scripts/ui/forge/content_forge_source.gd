class_name ContentForgeSource
extends ForgeSource

# The Content Forge's source: ContentSchema fields, ContentCatalog references.

func fields(kind: String) -> Array[Dictionary]:
	return ContentSchema.fields(kind)

func normalize(kind: String, entry: Dictionary) -> Dictionary:
	return ContentSchema.normalize(kind, entry)

func ref_choices(f: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var ref_kind: String = str(f.get("ref", ""))
	for e: Dictionary in ContentCatalog.entries(ref_kind):
		if ContentSchema.passes_filter(f, e):
			var id: String = str(e.get("id", ""))
			out.append({"value": id, "label": ref_label(f, id)})
	return out

func ref_label(f: Dictionary, value: String) -> String:
	if value.is_empty():
		return "(none)"
	var e: Dictionary = ContentCatalog.get_entry(str(f.get("ref", "")), value)
	return "%s  [%s]" % [str(e.get("name", "?")), value] if not e.is_empty() else "%s (missing)" % value
