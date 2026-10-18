---
class: StatModifier
extends: RefCounted
description: "Tactical RPG - Stat Modifier Model Composable modifier supporting flat & percentage adjustments with context gating"
---

# Properties
* `stat_name`: String = ""
* `flat_bonus`: int = 0
* `percent_bonus`: float = 0.0
* `source_id`: String = ""
* `context_tag`: String = ""
* `duration_rounds`: int = -1  # -1 = permanent / equip / trait, >= 0 = temporary buff

# Methods

### `is_permanent() -> bool`
```logic
return duration_rounds < 0
```

### `matches_context(context: Dictionary) -> bool`
```logic
if context_tag.is_empty() or context_tag == "always":
	return true

# Check context tag pairs like "weapon:sword" or "stance:offensive"
if context_tag.contains(":"):
	var parts: PackedStringArray = context_tag.split(":")
	if parts.size() == 2:
		var key: String = parts[0]
		var expected_val: String = parts[1]
		if context.has(key):
			return str(context[key]).to_lower() == expected_val.to_lower()
		return false

# Single tag check
return context.has(context_tag) and DictUtil.to_bool(context[context_tag])
```

### `tick_round() -> bool`
```logic
if duration_rounds > 0:
	duration_rounds -= 1
	return duration_rounds == 0
return false
```

