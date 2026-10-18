---
class: ForgeSource
extends: RefCounted
description: "Tactical RPG - Forge Source (what a ForgeForm edits) Tells the schema-driven form where its fields and reference choices come from, so the Content Forge (lore-free catalog) and the World Forge (lore worlds) share one editor. Subclasses: ContentForgeSource (ContentSchema / ContentCatalog) and WorldForgeSource (WorldSchema / WorldCatalog, in the context of one world)."
---

# Properties
* `none`: Array[Dictionary] = []
* `none`: Array[Dictionary] = []

# Methods

### `field(kind: String, key: String) -> Dictionary`
```logic
for f: Dictionary in fields(kind):
	if str(f.get("key", "")) == key:
		return f
return {}
```

### `normalize(_kind: String, entry: Dictionary) -> Dictionary`
```logic
return entry.duplicate(true)
```

### `ref_label(_f: Dictionary, value: String) -> String`
```logic
return "(none)" if value.is_empty() else value
```

