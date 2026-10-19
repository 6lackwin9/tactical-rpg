class_name ForgeSource
extends RefCounted

# ==============================================================================
# Tactical RPG - Forge Source (what a ForgeForm edits)
# Tells the schema-driven form where its fields and reference choices come from, so
# the Content Forge (lore-free catalog) and the World Forge (lore worlds) share one
# editor. Subclasses: ContentForgeSource (ContentSchema / ContentCatalog) and
# WorldForgeSource (WorldSchema / WorldCatalog, in the context of one world).
# ==============================================================================

func fields(_kind: String) -> Array[Dictionary]:
	var none: Array[Dictionary] = []
	return none

func field(kind: String, key: String) -> Dictionary:
	for f: Dictionary in fields(kind):
		if str(f.get("key", "")) == key:
			return f
	return {}

func normalize(_kind: String, entry: Dictionary) -> Dictionary:
	return entry.duplicate(true)

func options_of(f: Dictionary) -> Array[String]:
	return ContentSchema.options_of(f)

# Choices for a ref / refs field: [{"value": stored string, "label": shown text}].
func ref_choices(_f: Dictionary) -> Array[Dictionary]:
	var none: Array[Dictionary] = []
	return none

# Shown text for a stored ref value ("(none)" for empty).
func ref_label(_f: Dictionary, value: String) -> String:
	return "(none)" if value.is_empty() else value
