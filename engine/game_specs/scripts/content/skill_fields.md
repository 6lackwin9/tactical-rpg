---
class: SkillFields
extends: RefCounted
description: "The skills field table of ContentSchema (moved here by CLN-9b; ContentSchema.fields(\"skills\") / field() are the API and cache the result). The kind lists (`when`), Forge sections and the rank flag are generated from SkillKinds."
---

# Properties
* `list`: Array[Dictionary] = [
* `key`: String = str(f["key"])
* `kinds`: Array[String] = SkillKinds.kinds_with(key)
* `sections`: Array[String] = []

# Methods

### `static _f(key: String, label: String, type: String, default: Variant, extra: Dictionary = {}) -> Dictionary`
```logic
return ContentSchema._f(key, label, type, default, extra)
```

