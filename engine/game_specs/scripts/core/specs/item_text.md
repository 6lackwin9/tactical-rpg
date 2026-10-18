---
class: ItemText
extends: RefCounted
description: "Tactical RPG - Item description tokens and derived text (FORGE1_ITEMS section 2.4) The item description is a {token} template rendered with the RESOLVED row's values (SkillText's API shape, on SkillText's parser and formatting helpers). The row keeps the template; ContentFactory.item and the Forge preview render it. Legacy descriptions without tokens render unchanged. Problems are {\"field\": \"description\", \"index\": 0, \"message\": text}."
---

# Properties
* `MAX_LENGTH`: int = 240 (const)
* `EQUIP`: Array[String] = ["WEAPON", "SHIELD", "ARMOR", "ACCESSORY"] (const)
* `TOKEN_TYPES`: Dictionary = { (const)
* `STAT_TOKENS`: Array[String] = ["req", "bon"] (const)
* `NUMBER_TOKENS`: Dictionary = {"power": "base_power", "range": "base_range", "burden": "burden", "mitigation": "mitigation", "block_cost": "block_cost"} (const)
* `out`: Array[String] = []
* `out`: Array[String] = []
* `errors`: Array[Dictionary] = []
* `type`: String = str(row.get("item_type", "WEAPON"))
* `parsed`: Dictionary = SkillText.parse(template)
* `valid`: String = ", ".join(tokens_for(type))
* `part`: Dictionary = DictUtil.dict_of(raw)
* `token`: String = str(part["token"])
* `suffix`: String = str(part.get("suffix", ""))
* `base`: String = str(_split(token)["base"])
* `out`: Array[String] = []
* `type`: String = str(row.get("item_type", "WEAPON"))
* `candidates`: Dictionary = {}   # digits -> token
* `value`: int = DictUtil.to_int(row.get(str(NUMBER_TOKENS[token]), 0))
* `stats`: Dictionary = DictUtil.dict_of(row.get("required_stats" if base == "req" else "stat_bonuses", {}))
* `v`: int = DictUtil.to_int(stats[stat])
* `tiers`: Array[Dictionary] = ItemModifiers.tiers_of(row)
* `tok`: String = "tier%d.%s" % [i + 1, field]
* `text`: String = token_value(tok, "", row)
* `parsed`: Dictionary = SkillText.parse(template)
* `re`: Variant
* `part`: Dictionary = DictUtil.dict_of(raw)
* `digits`: String = m.get_string()

# Methods

### `static token_valid_for(token: String, item_type: String) -> bool`
```logic
var raw: Variant = TOKEN_TYPES.get(token, [])
var types: Array = raw if raw is Array else []
return types.has("ALL") or types.has(item_type)
```

### `static _split(token: String) -> Dictionary`
```logic
var re := RegEx.new()
re.compile("^tier(\\d+)\\.(value|power)$")
var m: RegExMatch = re.search(token)
if m != null:
	return {"base": "tier." + m.get_string(2), "n": m.get_string(1).to_int()}
return {"base": token, "n": 0}
```

### `static token_value(token: String, suffix: String, row: Dictionary) -> String`
```logic
var split: Dictionary = _split(token)
var base: String = str(split["base"])
match base:
	"name":
		return str(row.get("name", ""))
	"family":
		return str(row.get("family", ""))
	"hand":
		var hand: String = str(row.get("hand", ""))
		return Equipment.hand_display_name(hand) if not hand.is_empty() else ""
	"req", "bon":
		var stats: Dictionary = DictUtil.dict_of(row.get("required_stats" if base == "req" else "stat_bonuses", {}))
		if not stats.has(suffix):
			return ""
		return str(DictUtil.to_int(stats[suffix]))
	"tier.value", "tier.power":
		var tiers: Array[Dictionary] = ItemModifiers.tiers_of(row)
		var n: int = DictUtil.to_int(split["n"])
		if n < 1 or n > tiers.size():
			return ""
		var tier: Dictionary = tiers[n - 1]
		if base == "tier.value":
			return str(DictUtil.to_int(DictUtil.dict_of(tier.get("when", {})).get("value", DictUtil.dict_of(tier.get("when", {})).get("level", 0))))
		var power: int = DictUtil.to_int(DictUtil.dict_of(tier.get("bonus", {})).get("power", 0))
		return str(power) if power > 0 else ""
if NUMBER_TOKENS.has(base):
	return str(DictUtil.to_int(row.get(str(NUMBER_TOKENS[base]), 0)))
return ""
```

### `static render(template: String, row: Dictionary) -> String`
```logic
if not template.contains("{") and not template.contains("}"):
	return template
var parsed: Dictionary = SkillText.parse(template)
var type: String = str(row.get("item_type", "WEAPON"))
var out: String = ""
for raw: Variant in DictUtil.array_of(parsed["parts"]):
	var part: Dictionary = DictUtil.dict_of(raw)
	if part.has("lit"):
		out += str(part["lit"])
		continue
	var token: String = str(part["token"])
	if TOKEN_TYPES.has(str(_split(token)["base"])) and token_valid_for(str(_split(token)["base"]), type):
		out += token_value(token, str(part.get("suffix", "")), row)
return out
```

### `static _problem(message: String) -> Dictionary`
```logic
return {"field": "description", "index": 0, "message": message}
```

### `static modifier_summary(row: Dictionary) -> String`
```logic
var bonus_parts: Array[String] = []
var bonuses: Dictionary = DictUtil.dict_of(row.get("stat_bonuses", {}))
for stat: Variant in bonuses:
	bonus_parts.append("+%d %s" % [DictUtil.to_int(bonuses[stat]), str(stat)])
var req_parts: Array[String] = []
var reqs: Dictionary = DictUtil.dict_of(row.get("required_stats", {}))
for stat: Variant in reqs:
	req_parts.append("%s %d" % [str(stat), DictUtil.to_int(reqs[stat])])
var parts: Array[String] = []
if not bonus_parts.is_empty():
	parts.append(", ".join(bonus_parts))
if not req_parts.is_empty():
	parts.append("Req " + ", ".join(req_parts))
return "; ".join(parts)
```

### `static bonus_text(bonus: Dictionary) -> String`
```logic
var parts: Array[String] = []
if DictUtil.to_int(bonus.get("power", 0)) != 0:
	parts.append("+%d Power" % DictUtil.to_int(bonus["power"]))
if DictUtil.to_int(bonus.get("mitigation", 0)) != 0:
	parts.append("+%d Mitigation" % DictUtil.to_int(bonus["mitigation"]))
var stats: Dictionary = DictUtil.dict_of(bonus.get("stat_bonuses", {}))
for stat: Variant in stats:
	parts.append("+%d %s" % [DictUtil.to_int(stats[stat]), str(stat)])
var fx: int = DictUtil.array_of(bonus.get("effects", [])).size()
if fx > 0:
	parts.append("+%d effect%s" % [fx, "" if fx == 1 else "s"])
return ", ".join(parts)
```

### `static tier_desc(tier: Dictionary, _item_type: String = "") -> String`
```logic
var label: String = str(tier.get("label", "")).strip_edges()
var text: String = bonus_text(DictUtil.dict_of(tier.get("bonus", {})))
if label.is_empty():
	return text
return label + ": " + text if not text.is_empty() else label
```

### `static hand_label(row: Dictionary) -> String`
```logic
return Equipment.hand_display_name(str(row.get("hand", "")))
```

