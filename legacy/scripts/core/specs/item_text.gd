class_name ItemText
extends RefCounted

# ==============================================================================
# Tactical RPG - Item description tokens and derived text (FORGE1_ITEMS section 2.4)
# The item description is a {token} template rendered with the RESOLVED row's values (SkillText's API shape,
# on SkillText's parser and formatting helpers). The row keeps the template; ContentFactory.item and the Forge
# preview render it. Legacy descriptions without tokens render unchanged. Problems are
# {"field": "description", "index": 0, "message": text}.
# ==============================================================================

const MAX_LENGTH: int = 240
const EQUIP: Array[String] = ["WEAPON", "SHIELD", "ARMOR", "ACCESSORY"]

# token -> item types it is valid for ("ALL" = every type). "req" / "bon" take a stat suffix ({req:STR});
# tier tokens are written {tier<n>.value} (criterion value) and {tier<n>.power} (bonus power, weapons).
const TOKEN_TYPES: Dictionary = {
	"name": ["ALL"],
	"family": ["ALL"],
	"power": ["WEAPON"],
	"range": ["WEAPON"],
	"hand": ["WEAPON", "SHIELD"],
	"burden": EQUIP,
	"mitigation": EQUIP,
	"block_cost": EQUIP,
	"req": EQUIP,
	"bon": EQUIP,
	"tier.value": EQUIP,
	"tier.power": ["WEAPON"],
}
const STAT_TOKENS: Array[String] = ["req", "bon"]
const NUMBER_TOKENS: Dictionary = {"power": "base_power", "range": "base_range", "burden": "burden", "mitigation": "mitigation", "block_cost": "block_cost"}

static func all_tokens() -> Array[String]:
	var out: Array[String] = []
	for key: Variant in TOKEN_TYPES:
		out.append(str(key))
	return out

# The tokens valid for `item_type`, as the author writes them ({req:STAT}, {tier1.value}).
static func tokens_for(item_type: String) -> Array[String]:
	var out: Array[String] = []
	for token: String in all_tokens():
		if not token_valid_for(token, item_type):
			continue
		if STAT_TOKENS.has(token):
			out.append(token + ":STAT")
		elif token.begins_with("tier."):
			out.append("tier1." + token.get_slice(".", 1))
		else:
			out.append(token)
	return out

static func token_valid_for(token: String, item_type: String) -> bool:
	var raw: Variant = TOKEN_TYPES.get(token, [])
	var types: Array = raw if raw is Array else []
	return types.has("ALL") or types.has(item_type)

# "tier2.power" -> {"base": "tier.power", "n": 2}; any other token -> {"base": token, "n": 0}.
static func _split(token: String) -> Dictionary:
	var re := RegEx.new()
	re.compile("^tier(\\d+)\\.(value|power)$")
	var m: RegExMatch = re.search(token)
	if m != null:
		return {"base": "tier." + m.get_string(2), "n": m.get_string(1).to_int()}
	return {"base": token, "n": 0}

# ------------------------------------------------------------------------------
# Values and rendering
# ------------------------------------------------------------------------------

# The text of one token for a resolved row; "" when it has no value for this item.
static func token_value(token: String, suffix: String, row: Dictionary) -> String:
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

# The description of a resolved row with its tokens filled in (invalid tokens render as nothing).
static func render(template: String, row: Dictionary) -> String:
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

# ------------------------------------------------------------------------------
# Validation (errors block Save) and warnings (soft)
# ------------------------------------------------------------------------------

static func _problem(message: String) -> Dictionary:
	return {"field": "description", "index": 0, "message": message}

static func validate(template: String, row: Dictionary) -> Array[Dictionary]:
	var errors: Array[Dictionary] = []
	var type: String = str(row.get("item_type", "WEAPON"))
	if template.length() > MAX_LENGTH:
		errors.append(_problem("Description is %d characters (max %d)." % [template.length(), MAX_LENGTH]))
	if not template.contains("{") and not template.contains("}"):
		return errors
	var parsed: Dictionary = SkillText.parse(template)
	for pe: Variant in DictUtil.array_of(parsed["errors"]):
		errors.append(_problem(str(DictUtil.dict_of(pe).get("message", ""))))
	var valid: String = ", ".join(tokens_for(type))
	for raw: Variant in DictUtil.array_of(parsed["parts"]):
		var part: Dictionary = DictUtil.dict_of(raw)
		if not part.has("token"):
			continue
		var token: String = str(part["token"])
		var suffix: String = str(part.get("suffix", ""))
		var base: String = str(_split(token)["base"])
		if not TOKEN_TYPES.has(base):
			errors.append(_problem("Unknown token {%s}. Valid for %s: %s." % [str(part["raw"]), type, valid]))
			continue
		if not token_valid_for(base, type):
			errors.append(_problem("Token {%s} does not apply to %s items. Valid: %s." % [str(part["raw"]), type, valid]))
			continue
		if STAT_TOKENS.has(base) and not ContentSchema.STAT_KEYS.has(suffix):
			errors.append(_problem("Token {%s} needs a stat: {%s:STR} (one of %s)." % [str(part["raw"]), base, ", ".join(ContentSchema.STAT_KEYS)]))
			continue
		if not STAT_TOKENS.has(base) and not suffix.is_empty():
			errors.append(_problem("Token {%s}: unknown suffix ':%s'." % [str(part["raw"]), suffix]))
			continue
		if token_value(token, suffix, row).is_empty():
			errors.append(_problem("Token {%s} has no value for this item (no such requirement, bonus or tier)." % str(part["raw"])))
	if errors.is_empty() and render(template, row).length() > MAX_LENGTH:
		errors.append(_problem("The rendered description is %d characters (max %d)." % [render(template, row).length(), MAX_LENGTH]))
	return errors

# Literal numbers equal to a resolved numeric field (soft: suggests the token). Defaults (range 1, block cost 1, 0) are
# ignored; legacy descriptions without any token are shown as is and never warn (section 2.4).
static func warnings(template: String, row: Dictionary) -> Array[String]:
	var out: Array[String] = []
	if not template.contains("{"):
		return out
	var type: String = str(row.get("item_type", "WEAPON"))
	var candidates: Dictionary = {}   # digits -> token
	for token: Variant in NUMBER_TOKENS:
		if not token_valid_for(str(token), type):
			continue
		var value: int = DictUtil.to_int(row.get(str(NUMBER_TOKENS[token]), 0))
		if value > 1 and not candidates.has(str(value)):
			candidates[str(value)] = str(token)
	for base: String in STAT_TOKENS:
		if not token_valid_for(base, type):
			continue
		var stats: Dictionary = DictUtil.dict_of(row.get("required_stats" if base == "req" else "stat_bonuses", {}))
		for stat: Variant in stats:
			var v: int = DictUtil.to_int(stats[stat])
			if v > 1 and not candidates.has(str(v)):
				candidates[str(v)] = "%s:%s" % [base, str(stat)]
	var tiers: Array[Dictionary] = ItemModifiers.tiers_of(row)
	for i: int in range(tiers.size()):
		for field: String in ["value", "power"]:
			var tok: String = "tier%d.%s" % [i + 1, field]
			var text: String = token_value(tok, "", row)
			if token_valid_for("tier." + field, type) and text.to_int() > 1 and not candidates.has(text):
				candidates[text] = tok
	if candidates.is_empty():
		return out
	var parsed: Dictionary = SkillText.parse(template)
	var re := RegEx.new()
	re.compile("\\d+")
	for raw: Variant in DictUtil.array_of(parsed["parts"]):
		var part: Dictionary = DictUtil.dict_of(raw)
		if not part.has("lit"):
			continue
		for m: RegExMatch in re.search_all(str(part["lit"])):
			var digits: String = m.get_string()
			if candidates.has(digits):
				out.append("Description has the literal number %s; it can drift from the item. Use {%s}." % [digits, str(candidates[digits])])
	return out

# ------------------------------------------------------------------------------
# Auto lines (derived, never stored)
# ------------------------------------------------------------------------------

# "+2 STR, +2 CONS; Req STR 14" (bonuses, then requirements); "" when the item has neither.
static func modifier_summary(row: Dictionary) -> String:
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

# "+4 Power", "+2 Mitigation, +1 STR", "+1 effect".
static func bonus_text(bonus: Dictionary) -> String:
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

# The auto description of a tier: "<label>: <bonus text>" ("Sunder Guard: +4 Power").
static func tier_desc(tier: Dictionary, _item_type: String = "") -> String:
	var label: String = str(tier.get("label", "")).strip_edges()
	var text: String = bonus_text(DictUtil.dict_of(tier.get("bonus", {})))
	if label.is_empty():
		return text
	return label + ": " + text if not text.is_empty() else label

static func hand_label(row: Dictionary) -> String:
	return Equipment.hand_display_name(str(row.get("hand", "")))
