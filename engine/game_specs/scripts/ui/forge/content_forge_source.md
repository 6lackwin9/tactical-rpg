---
class: ContentForgeSource
extends: ForgeSource
description: "The Content Forge's source: ContentSchema fields, ContentCatalog references."
---

# Properties
* `out`: Array[Dictionary] = []
* `ref_kind`: String = str(f.get("ref", ""))
* `id`: String = str(e.get("id", ""))

# Methods

### `normalize(kind: String, entry: Dictionary) -> Dictionary`
```logic
return ContentSchema.normalize(kind, entry)
```

### `ref_label(f: Dictionary, value: String) -> String`
```logic
if value.is_empty():
	return "(none)"
var e: Dictionary = ContentCatalog.get_entry(str(f.get("ref", "")), value)
return "%s  [%s]" % [str(e.get("name", "?")), value] if not e.is_empty() else "%s (missing)" % value
```

