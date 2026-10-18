---
class: SkillText
extends: RefCounted
description: "Tactical RPG - Skill description tokens (FORGE1 section 5 / appendix L) One description TEMPLATE per skill family with {token} placeholders; each rank renders it with its EFFECTIVE values. Pure static functions: the Forge preview, the rank expander and the validator all call the same code, so they cannot drift apart. Literal braces are written {{ and }}. Problems are {\"field\": \"description\", \"rank\": n (0 = template), \"message\": text}."
---

# Properties
* `MAX_LENGTH`: int = 240 (const)
* `TOKEN_KINDS`: Dictionary = { (const)
* `COUNTED`: Array[String] = ["blocks", "dash", "reach", "rounds", "jump"] (const)
* `out`: Array[String] = []
* `out`: Array[String] = []
* `errors`: Array[Dictionary] = []
* `kind`: String = str(entry.get("kind", "ATTACK"))
* `parsed`: Dictionary = parse(template)
* `parse_errors`: Array = parsed["errors"]
* `usable`: Array[String] = []
* `parts`: Array = parsed["parts"]
* `part`: Dictionary = raw
* `token`: String = str(part["token"])
* `suffix`: String = str(part.get("suffix", ""))
* `count`: int = SkillRanks.rank_count(entry)
* `row`: Dictionary = SkillRanks.effective(entry, rank)
* `token`: String = raw_token.get_slice(":", 0)
* `suffix`: String = raw_token.get_slice(":", 1) if raw_token.contains(":") else ""
* `text`: String = render(template, kind, row, rank)
* `out`: Array[String] = []
* `parsed`: Dictionary = parse(template)
* `kind`: String = str(entry.get("kind", "ATTACK"))
* `re`: Variant
* `parts`: Array = parsed["parts"]
* `first`: Dictionary = SkillRanks.effective(entry, 1)
* `part`: Dictionary = raw
* `digits`: String = m.get_string()
* `hint`: String = ""
* `ranks`: Array = DictUtil.array_of(entry.get("ranks", []))

# Methods

### `static token_valid_for(token: String, kind: String) -> bool`
```logic
var raw: Variant = TOKEN_KINDS.get(token, [])
var kinds: Array = raw if raw is Array else []
return kinds.has("ALL") or kinds.has(kind)
```

### `static plural(n: int, unit: String) -> String`
```logic
return "%d %s%s" % [n, unit, "" if n == 1 else "s"]
```

### `static format_percent(x: float) -> String`
```logic
return "%d%%" % roundi(x * 100.0)
```

### `static format_decimal(x: float) -> String`
```logic
var text: String = String.num(snappedf(x, 0.01), 2)
if text.contains("."):
	while text.ends_with("0") and text.length() - text.find(".") > 2:
		text = text.substr(0, text.length() - 1)
else:
	text += ".0"
return text
```

### `static format_mult(x: float) -> String`
```logic
return "x" + format_decimal(x)
```

### `static format_reach(reach_min: int, reach_max: int, bare: bool = false) -> String`
```logic
if reach_max <= 0:
	return ""
if reach_min > 0 and reach_min != reach_max:
	return "%d-%d" % [reach_min, reach_max] if bare else "%d-%d tiles" % [reach_min, reach_max]
return str(reach_max) if bare else plural(reach_max, "tile")
```

### `static format_heal(base: int, mult: float, stat: String) -> String`
```logic
if base > 0 and mult > 0.0:
	return "%d + %s x %s" % [base, format_decimal(mult), stat]
if base > 0:
	return str(base)
if mult > 0.0:
	return "%s x %s" % [format_decimal(mult), stat]
return ""
```

### `static format_cleanse(list: Array[String]) -> String`
```logic
if list.is_empty():
	return ""
if list.has(SkillDef.CLEANSE_ALL):
	return "all statuses"
return ", ".join(list)
```

### `static parse(template: String) -> Dictionary`
```logic
var parts: Array[Dictionary] = []
var errors: Array[Dictionary] = []
var lit: String = ""
var i: int = 0
var n: int = template.length()
while i < n:
	var ch: String = template[i]
	if ch == "{":
		if i + 1 < n and template[i + 1] == "{":
			lit += "{"
			i += 2
			continue
		var close: int = template.find("}", i + 1)
		var nested: int = template.find("{", i + 1)
		if close < 0 or (nested >= 0 and nested < close):
			errors.append(_problem(0, "Unbalanced '{' at position %d: close the token with '}' (write '{{' for a literal brace)." % i))
			lit += ch
			i += 1
			continue
		var body: String = template.substr(i + 1, close - i - 1)
		if body.strip_edges().is_empty():
			errors.append(_problem(0, "Empty token '{}' at position %d." % i))
		else:
			if not lit.is_empty():
				parts.append({"lit": lit})
				lit = ""
			var token: String = body
			var suffix: String = ""
			if body.contains(":"):
				token = body.get_slice(":", 0)
				suffix = body.get_slice(":", 1)
			parts.append({"token": token, "suffix": suffix, "raw": body})
		i = close + 1
	elif ch == "}":
		if i + 1 < n and template[i + 1] == "}":
			lit += "}"
			i += 2
		else:
			errors.append(_problem(0, "Stray '}' at position %d (write '}}' for a literal brace)." % i))
			i += 1
	else:
		lit += ch
		i += 1
if not lit.is_empty():
	parts.append({"lit": lit})
return {"parts": parts, "errors": errors}
```

### `static _problem(rank: int, message: String) -> Dictionary`
```logic
return {"field": "description", "rank": rank, "message": message}
```

### `static escape(text: String) -> String`
```logic
return text.replace("{", "{{").replace("}", "}}")
```

### `static token_value(token: String, suffix: String, row: Dictionary, rank: int) -> String`
```logic
var bare: bool = suffix == "n"
match token:
	"name":
		return str(row.get("name", ""))
	"ap":
		return "%d AP" % DictUtil.to_int(row.get("level", 1), 1)
	"blocks":
		var blocks: int = DictUtil.to_int(row.get("blocks", 1), 1)
		return str(blocks) if bare else plural(blocks, "block")
	"power":
		return format_percent(DictUtil.to_float(row.get("power_mult", 1.0), 1.0))
	"dash":
		var dash: int = DictUtil.to_int(row.get("dash_tiles", 0))
		if dash <= 0:
			return ""
		return str(dash) if bare else plural(dash, "tile")
	"reach":
		return format_reach(DictUtil.to_int(row.get("reach_min", 0)), DictUtil.to_int(row.get("reach_max", 0)), bare)
	"weapons":
		return "/".join(ContentFactory._strings(row.get("weapon_types", [])))
	"proficiency":
		var level: int = DictUtil.to_int(row.get("min_proficiency", 0))
		return str(level) if level > 0 else ""
	"status":
		var status: String = _status_of(row)
		return StatusEffects.display_name(status) if not status.is_empty() else ""
	"potency":
		if str(row.get("status_effect", "")).is_empty():
			return ""
		return format_percent(DictUtil.to_float(row.get("status_potency", 0)) / 100.0)
	"magnitude":
		return format_magnitude(magnitude_status(row), DictUtil.to_int(row.get("status_magnitude", 0)))
	"rounds":
		var rounds: int = DictUtil.to_int(row.get("boost_rounds", 1), 1) if str(row.get("kind", "")) == "FOCUS" else DictUtil.to_int(row.get("status_rounds", 1), 1)
		return str(rounds) if bare else plural(rounds, "round")
	"heal":
		return format_heal(DictUtil.to_int(row.get("heal_base", 0)), DictUtil.to_float(row.get("heal_mult", 0.0)), str(row.get("heal_stat", "")))
	"cleanse":
		return format_cleanse(ContentFactory._strings(row.get("cleanse_statuses", [])))
	"mitigation":
		return format_percent(DictUtil.to_float(row.get("mitigation", 0.0)))
	"boost":
		return format_mult(DictUtil.to_float(row.get("boost", 1.0), 1.0))
	"jump":
		var jump: int = DictUtil.to_int(row.get("jump_tiles", 0))
		if jump <= 0:
			return ""
		return str(jump) if bare else plural(jump, "tile")
	"recover":
		return "%d AP" % DictUtil.to_int(row.get("recover_ap", 0))
	"unlock":
		var offset: int = DictUtil.to_int(row.get("unlock_offset", -1), -1)
		return "+%d" % (offset if offset >= 0 else StatRules.TIER_UNLOCK_STEP * maxi(0, rank - 1))
	"flow":
		return format_flow(str(row.get("ends_in_stance", "")))
	"exploit":
		return format_exploit(DictUtil.array_of(row.get("exploit", [])))
	"disrupt":
		return "breaks stance" if str(row.get("disrupt", "NONE")) == "STANCE_BREAK" else ""
return ""
```

### `static format_flow(stance_name: String) -> String`
```logic
var i: int = CombatTypes.StanceType.keys().find(stance_name)
if stance_name.is_empty() or i < 0:
	return ""
return "ends in %s" % CombatTypes.stance_to_string(i as CombatTypes.StanceType)
```

### `static format_exploit(entries: Array) -> String`
```logic
var parts: Array[String] = []
for raw: Variant in entries:
	var row: Dictionary = DictUtil.dict_of(raw)
	parts.append("%s vs %s" % [format_mult(DictUtil.to_float(row.get("power_mult", 1.0), 1.0)), StatusEffects.display_name(str(row.get("status", "")))])
return ", ".join(parts)
```

### `static magnitude_status(row: Dictionary) -> String`
```logic
var inflicted: String = str(row.get("status_effect", ""))
return inflicted if not inflicted.is_empty() else str(row.get("grant_status", ""))
```

### `static format_magnitude(status: String, magnitude: int) -> String`
```logic
if magnitude <= 0 or not StatusEffects.has_magnitude(status):
	return ""
match StatusRegistry.magnitude_unit(status):
	StatusRegistry.UNIT_FLAT:
		return "%d HP" % magnitude if status == StatusEffects.POISON else "%d armor" % magnitude
	StatusRegistry.UNIT_PCT_MAX_HP:
		return "%d%% max HP" % magnitude
return "%d%%" % magnitude
```

### `static _status_of(row: Dictionary) -> String`
```logic
if str(row.get("kind", "")) == "ATTACK":
	return str(row.get("status_effect", ""))
return str(row.get("grant_status", ""))
```

### `static render(template: String, kind: String, row: Dictionary, rank: int) -> String`
```logic
var parsed: Dictionary = parse(template)
var out: String = ""
var parts: Array = parsed["parts"]
for raw: Variant in parts:
	var part: Dictionary = raw
	if part.has("lit"):
		out += str(part["lit"])
	else:
		var token: String = str(part["token"])
		if TOKEN_KINDS.has(token) and token_valid_for(token, kind):
			out += token_value(token, str(part.get("suffix", "")), row, rank)
return out
```

