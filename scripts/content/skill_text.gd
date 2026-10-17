class_name SkillText
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill description tokens (FORGE1 section 5 / appendix L)
# One description TEMPLATE per skill family with {token} placeholders; each rank renders it with
# its EFFECTIVE values. Pure static functions: the Forge preview, the rank expander and the validator
# all call the same code, so they cannot drift apart. Literal braces are written {{ and }}.
# Problems are {"field": "description", "rank": n (0 = template), "message": text}.
# ==============================================================================

const MAX_LENGTH: int = 240

# token -> kinds it is valid for ("ALL" = every kind). Counted tokens (plural unit) accept the ":n" suffix.
const TOKEN_KINDS: Dictionary = {
	"name": ["ALL"],
	"ap": ["ALL"],
	"blocks": ["ALL"],
	"power": ["ATTACK"],
	"dash": ["ATTACK"],
	"reach": ["ATTACK", "SUPPORT"],
	"weapons": ["ATTACK", "DEFEND", "SUPPORT"],
	"proficiency": ["ATTACK", "DEFEND", "SUPPORT"],
	"status": ["ATTACK", "SUPPORT", "DEFEND"],
	"potency": ["ATTACK"],
	"magnitude": ["ATTACK", "SUPPORT", "DEFEND"],
	"rounds": ["ATTACK", "SUPPORT", "DEFEND", "FOCUS"],
	"heal": ["SUPPORT"],
	"cleanse": ["SUPPORT"],
	"mitigation": ["DEFEND"],
	"boost": ["FOCUS"],
	"jump": ["DISENGAGE"],
	"recover": ["REST"],
	"unlock": ["ALL"],
	"flow": ["ATTACK", "DEFEND", "SUPPORT", "FOCUS"],
	"exploit": ["ATTACK"],
	"disrupt": ["ATTACK"],
}

const COUNTED: Array[String] = ["blocks", "dash", "reach", "rounds", "jump"]

static func all_tokens() -> Array[String]:
	var out: Array[String] = []
	for key: Variant in TOKEN_KINDS:
		out.append(str(key))
	return out

static func tokens_for(kind: String) -> Array[String]:
	var out: Array[String] = []
	for token: String in all_tokens():
		if token_valid_for(token, kind):
			out.append(token)
	return out

static func token_valid_for(token: String, kind: String) -> bool:
	var raw: Variant = TOKEN_KINDS.get(token, [])
	var kinds: Array = raw if raw is Array else []
	return kinds.has("ALL") or kinds.has(kind)

# ------------------------------------------------------------------------------
# Formatting helpers
# ------------------------------------------------------------------------------

static func plural(n: int, unit: String) -> String:
	return "%d %s%s" % [n, unit, "" if n == 1 else "s"]

static func format_percent(x: float) -> String:
	return "%d%%" % roundi(x * 100.0)

# At least 1 and at most 2 decimals: 1.5 -> "1.5", 1.0 -> "1.0", 1.25 -> "1.25".
static func format_decimal(x: float) -> String:
	var text: String = String.num(snappedf(x, 0.01), 2)
	if text.contains("."):
		while text.ends_with("0") and text.length() - text.find(".") > 2:
			text = text.substr(0, text.length() - 1)
	else:
		text += ".0"
	return text

static func format_mult(x: float) -> String:
	return "x" + format_decimal(x)

static func format_reach(reach_min: int, reach_max: int, bare: bool = false) -> String:
	if reach_max <= 0:
		return ""
	if reach_min > 0 and reach_min != reach_max:
		return "%d-%d" % [reach_min, reach_max] if bare else "%d-%d tiles" % [reach_min, reach_max]
	return str(reach_max) if bare else plural(reach_max, "tile")

static func format_heal(base: int, mult: float, stat: String) -> String:
	if base > 0 and mult > 0.0:
		return "%d + %s x %s" % [base, format_decimal(mult), stat]
	if base > 0:
		return str(base)
	if mult > 0.0:
		return "%s x %s" % [format_decimal(mult), stat]
	return ""

static func format_cleanse(list: Array[String]) -> String:
	if list.is_empty():
		return ""
	if list.has(SkillDef.CLEANSE_ALL):
		return "all statuses"
	return ", ".join(list)

# ------------------------------------------------------------------------------
# Parsing
# ------------------------------------------------------------------------------

# {"parts": [{"lit": text} | {"token": name, "suffix": text}], "errors": [problems]}
static func parse(template: String) -> Dictionary:
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

static func _problem(rank: int, message: String) -> Dictionary:
	return {"field": "description", "rank": rank, "message": message}

# Escapes literal braces so a plain sentence can be used as a template.
static func escape(text: String) -> String:
	return text.replace("{", "{{").replace("}", "}}")

# ------------------------------------------------------------------------------
# Rendering
# ------------------------------------------------------------------------------

# The text of one token for the effective row of a rank; "" when it resolves to nothing.
static func token_value(token: String, suffix: String, row: Dictionary, rank: int) -> String:
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

# "ends in Defensive" for a flow skill's end stance name; "" when empty (S12C-L).
static func format_flow(stance_name: String) -> String:
	var i: int = CombatTypes.StanceType.keys().find(stance_name)
	if stance_name.is_empty() or i < 0:
		return ""
	return "ends in %s" % CombatTypes.stance_to_string(i as CombatTypes.StanceType)

# "x1.3 vs Exposed, x1.2 vs Slowed" for an exploit list (status display names); "" when empty (S12C-L).
static func format_exploit(entries: Array) -> String:
	var parts: Array[String] = []
	for raw: Variant in entries:
		var row: Dictionary = DictUtil.dict_of(raw)
		parts.append("%s vs %s" % [format_mult(DictUtil.to_float(row.get("power_mult", 1.0), 1.0)), StatusEffects.display_name(str(row.get("status", "")))])
	return ", ".join(parts)

# The status a skill row's magnitude belongs to: status_effect if set, else grant_status (S12S-L).
static func magnitude_status(row: Dictionary) -> String:
	var inflicted: String = str(row.get("status_effect", ""))
	return inflicted if not inflicted.is_empty() else str(row.get("grant_status", ""))

# A magnitude with its registry unit: "15%", "8 armor", "4 HP", "5% max HP"; "" when the status has no range or it is 0.
# Flat magnitudes are armor points, except poison (HP per round).
static func format_magnitude(status: String, magnitude: int) -> String:
	if magnitude <= 0 or not StatusEffects.has_magnitude(status):
		return ""
	match StatusRegistry.magnitude_unit(status):
		StatusRegistry.UNIT_FLAT:
			return "%d HP" % magnitude if status == StatusEffects.POISON else "%d armor" % magnitude
		StatusRegistry.UNIT_PCT_MAX_HP:
			return "%d%% max HP" % magnitude
	return "%d%%" % magnitude

static func _status_of(row: Dictionary) -> String:
	if str(row.get("kind", "")) == "ATTACK":
		return str(row.get("status_effect", ""))
	return str(row.get("grant_status", ""))

# The template rendered for one rank. `row` is that rank's effective flat row (name already derived).
static func render(template: String, kind: String, row: Dictionary, rank: int) -> String:
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

# ------------------------------------------------------------------------------
# Validation
# ------------------------------------------------------------------------------

# Errors (block Save) of `template` for the ranked skill `entry` (any entry shape SkillRanks accepts).
static func validate(template: String, entry: Dictionary) -> Array[Dictionary]:
	var errors: Array[Dictionary] = []
	var kind: String = str(entry.get("kind", "ATTACK"))
	var parsed: Dictionary = parse(template)
	var parse_errors: Array = parsed["errors"]
	for pe: Variant in parse_errors:
		errors.append(DictUtil.dict_of(pe))
	if template.length() > MAX_LENGTH:
		errors.append(_problem(0, "The description template is %d characters (max %d)." % [template.length(), MAX_LENGTH]))
	var usable: Array[String] = []
	var parts: Array = parsed["parts"]
	for raw: Variant in parts:
		var part: Dictionary = raw
		if not part.has("token"):
			continue
		var token: String = str(part["token"])
		var suffix: String = str(part.get("suffix", ""))
		if not TOKEN_KINDS.has(token):
			errors.append(_problem(0, "Unknown token {%s}. Valid for %s: %s." % [str(part["raw"]), kind, ", ".join(tokens_for(kind))]))
			continue
		if not token_valid_for(token, kind):
			errors.append(_problem(0, "Token {%s} does not apply to %s skills. Valid: %s." % [token, kind, ", ".join(tokens_for(kind))]))
			continue
		if token == "unlock" and (DictUtil.to_bool(entry.get("basic", false)) or DictUtil.to_bool(entry.get("system", false))):
			errors.append(_problem(0, "Token {unlock} does not apply to basic or engine skills."))
			continue
		if not suffix.is_empty() and (suffix != "n" or not COUNTED.has(token)):
			errors.append(_problem(0, "Token {%s}: unknown suffix ':%s' (only ':n' on %s)." % [token, suffix, ", ".join(COUNTED)]))
			continue
		if not usable.has(str(part["raw"])):
			usable.append(str(part["raw"]))
	var count: int = SkillRanks.rank_count(entry)
	for rank: int in range(1, count + 1):
		var row: Dictionary = SkillRanks.effective(entry, rank)
		for raw_token: String in usable:
			var token: String = raw_token.get_slice(":", 0)
			var suffix: String = raw_token.get_slice(":", 1) if raw_token.contains(":") else ""
			if token_value(token, suffix, row, rank).is_empty():
				errors.append(_problem(rank, "Rank %d (%s): token {%s} resolves to nothing (the field is empty or 0 for that rank)." % [rank, str(row.get("name", "")), raw_token]))
		if errors.is_empty():
			var text: String = render(template, kind, row, rank)
			if text.length() > MAX_LENGTH:
				errors.append(_problem(rank, "Rank %d (%s): the rendered description is %d characters (max %d)." % [rank, str(row.get("name", "")), text.length(), MAX_LENGTH]))
	return errors

# Soft warnings (never block): literal digits outside tokens, a literal per-rank description.
static func warnings(template: String, entry: Dictionary) -> Array[String]:
	var out: Array[String] = []
	var parsed: Dictionary = parse(template)
	var kind: String = str(entry.get("kind", "ATTACK"))
	var re := RegEx.new()
	re.compile("\\d+")
	var parts: Array = parsed["parts"]
	var first: Dictionary = SkillRanks.effective(entry, 1)
	for raw: Variant in parts:
		var part: Dictionary = raw
		if not part.has("lit"):
			continue
		for m: RegExMatch in re.search_all(str(part["lit"])):
			var digits: String = m.get_string()
			var hint: String = ""
			for token: String in tokens_for(kind):
				if token == "name":
					continue
				if token_value(token, "n", first, 1) == digits or token_value(token, "", first, 1).trim_suffix("%").trim_suffix(" AP") == digits:
					hint = " Use {%s}." % token
					break
			out.append("Description has the literal number %s; it can drift from the fields.%s" % [digits, hint])
	var ranks: Array = DictUtil.array_of(entry.get("ranks", []))
	for i: int in range(ranks.size()):
		if DictUtil.dict_of(ranks[i]).has("description"):
			out.append("Rank %d has a literal description instead of the template." % (i + 2))
	return out
