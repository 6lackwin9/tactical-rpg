---
class: SkillTextTestSuite
extends: RefCounted
description: "Tactical RPG - Skill Text Test Suite (FORGE1 section 5 / SKILL1-L) Description tokens: every token against a fixture per kind, the formatting rules (percent, multiplier, plural, \":n\", heal forms), the escape rules, and every validation error / warning. Fixtures are built here; expected strings are the documented formats applied to the fixture values (never to shipped numbers). The migration generator (templates for the 127 shipped descriptions) belongs to the asset seed, not this suite."
---

# Properties
* `TAG`: String = "SkillText" (const)

# Methods

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING SKILL TEXT TEST (SKILL1-L)              ===")
print("=======================================================")
var ok: bool = true
ok = _test_tokens_per_kind() and ok
ok = _test_formats() and ok
ok = _test_validation() and ok
ok = _test_warnings_and_caps() and ok
ok = _test_warnings_all() and ok
TestHelpers.cleanup(SANDBOX)
return ok
```

### `static _render(template: String, entry: Dictionary, rank: int = 1) -> String`
```logic
var row: Dictionary = SkillRanks.effective(entry, rank)
return SkillText.render(template, str(row["kind"]), row, rank)
```

### `static _attack() -> Dictionary`
```logic
return {"id": "t_tx", "name": "Tx", "kind": "ATTACK", "level": 3, "blocks": 2, "power_mult": 2.0, "dash_tiles": 4, "reach_min": 2, "reach_max": 3,
	"weapon_types": ["bow", "crossbow"], "min_proficiency": 3, "status_effect": "weaken", "status_potency": 50, "status_magnitude": 20, "status_rounds": 1}
```

### `static _test_tokens_per_kind() -> bool`
```logic
var checks: Dictionary = {}
var a: Dictionary = _attack()
checks["{name} is the derived rank name"] = _render("{name}", a) == "Tx" and _render("{name}", SkillRanks.entry(a, [{"level": 4}]), 2) == "Tx II"
checks["{ap}"] = _render("{ap}", a) == "%d AP" % DictUtil.to_int(a["level"])
checks["{blocks} pluralises"] = _render("{blocks}", a) == "2 blocks" and _render("{blocks}", {"id": "t_one", "kind": "ATTACK", "blocks": 1}) == "1 block"
checks["{power} is a percent"] = _render("{power}", a) == "%d%%" % roundi(DictUtil.to_float(a["power_mult"]) * 100.0)
checks["{dash} and {dash:n}"] = _render("{dash}", a) == "4 tiles" and _render("{dash:n}", a) == "4"
checks["{reach} shows a band"] = _render("{reach}", a) == "2-3 tiles"
var single: Dictionary = _attack()
single["reach_min"] = 1
single["reach_max"] = 1
checks["{reach} for one tile"] = _render("{reach}", single) == "1 tile" and _render("{reach:n}", single) == "1"
var open: Dictionary = _attack()
open["reach_min"] = 0
open["reach_max"] = 3
checks["{reach} with a zero minimum shows the maximum"] = _render("{reach}", open) == "3 tiles"
checks["{weapons} joins the types with /"] = _render("{weapons}", a) == "bow/crossbow"
checks["{proficiency}"] = _render("{proficiency}", a) == "3"
checks["{status} uses the registry display name"] = _render("{status}", a) == StatusEffects.display_name("weaken")
checks["{potency} and {magnitude} are percents"] = _render("{potency}", a) == "50%" and _render("{magnitude}", a) == "20%"
checks["{rounds} pluralises, {rounds:n} is bare"] = _render("{rounds}", a) == "1 round" and _render("{rounds:n}", a) == "1"
var rounds2: Dictionary = _attack()
rounds2["status_rounds"] = 2
checks["{rounds} plural"] = _render("{rounds}", rounds2) == "2 rounds"
checks["{unlock} follows the effective offset (auto = step x (rank - 1))"] = _render("{unlock}", a) == "+0" and _render("{unlock}", SkillRanks.entry(a, [{"level": 4}, {"level": 5}]), 3) == "+%d" % (StatRules.TIER_UNLOCK_STEP * 2)
var support: Dictionary = {"id": "t_su", "name": "Su", "kind": "SUPPORT", "level": 2, "blocks": 2, "heal_stat": "KNOW", "heal_base": 10, "heal_mult": 1.8, "grant_status": "haste", "status_rounds": 2, "cleanse_statuses": ["stun", "bleed"], "reach_min": 0, "reach_max": 2}
checks["{heal} base plus multiplier"] = _render("{heal}", support) == "10 + 1.8 x KNOW"
var base_only: Dictionary = support.duplicate()
base_only["heal_mult"] = 0.0
base_only["heal_stat"] = ""
base_only["heal_base"] = 12
checks["{heal} drops the zero part (base only)"] = _render("{heal}", base_only) == "12"
var mult_only: Dictionary = support.duplicate()
mult_only["heal_base"] = 0
mult_only["heal_mult"] = 1.5
mult_only["heal_stat"] = "INT"
checks["{heal} multiplier only"] = _render("{heal}", mult_only) == "1.5 x INT"
checks["{cleanse} lists statuses"] = _render("{cleanse}", support) == "stun, bleed"
var all_clear: Dictionary = support.duplicate()
all_clear["cleanse_statuses"] = ["all"]
checks["{cleanse} all statuses"] = _render("{cleanse}", all_clear) == "all statuses"
checks["{status} on SUPPORT is the granted buff"] = _render("{status}", support) == StatusEffects.display_name("haste") and _render("{rounds}", support) == "2 rounds"
var defend: Dictionary = {"id": "t_de", "name": "De", "kind": "DEFEND", "level": 2, "blocks": 2, "mitigation": 0.35}
checks["{mitigation} is a percent"] = _render("{mitigation}", defend) == "35%"
var focus: Dictionary = {"id": "t_fo", "name": "Fo", "kind": "FOCUS", "level": 2, "blocks": 1, "boost": 1.5, "boost_rounds": 3}
checks["{boost} is x plus the multiplier"] = _render("{boost}", focus) == "x1.5"
checks["{rounds} on FOCUS reads boost_rounds"] = _render("{rounds}", focus) == "3 rounds"
var disengage: Dictionary = {"id": "t_di", "name": "Di", "kind": "DISENGAGE", "level": 2, "blocks": 1, "jump_tiles": 3}
checks["{jump}"] = _render("{jump}", disengage) == "3 tiles"
var rest_row: Dictionary = {"name": "Rest", "kind": "REST", "recover_ap": 2}
checks["{recover}"] = SkillText.render("{recover}", "REST", rest_row, 1) == "2 AP"
checks["every kind lists name, ap, blocks and unlock"] = true
for kind: String in SkillKinds.authorable():
	var valid: Array[String] = SkillText.tokens_for(kind)
	checks["every kind lists name, ap, blocks and unlock"] = checks["every kind lists name, ap, blocks and unlock"] and valid.has("name") and valid.has("ap") and valid.has("blocks") and valid.has("unlock")
checks["kind-specific tokens are limited to their kinds"] = SkillText.tokens_for("ATTACK").has("power") and not SkillText.tokens_for("DEFEND").has("power") and SkillText.tokens_for("SUPPORT").has("heal") and not SkillText.tokens_for("ATTACK").has("heal") and SkillText.tokens_for("FOCUS").has("boost") and SkillText.tokens_for("DISENGAGE").has("jump")
return TestHelpers.report(TAG, "Every token against a fixture per kind", checks)
```

### `static _test_formats() -> bool`
```logic
var checks: Dictionary = {}
checks["decimal keeps one to two places"] = SkillText.format_decimal(1.0) == "1.0" and SkillText.format_decimal(1.5) == "1.5" and SkillText.format_decimal(1.25) == "1.25" and SkillText.format_decimal(1.8) == "1.8"
checks["percent rounds to an integer"] = SkillText.format_percent(0.35) == "35%" and SkillText.format_percent(2.0) == "200%" and SkillText.format_percent(1.234) == "123%"
checks["multiplier"] = SkillText.format_mult(1.25) == "x1.25"
checks["plural"] = SkillText.plural(1, "tile") == "1 tile" and SkillText.plural(3, "tile") == "3 tiles" and SkillText.plural(0, "round") == "0 rounds"
checks["AP is never pluralised"] = _render("{ap}", {"id": "t_f", "kind": "ATTACK", "level": 1}) == "1 AP"
var a: Dictionary = _attack()
checks["{{ and }} are literal braces"] = _render("{{power}} = {power}", a) == "{power} = 200%"
checks["the rendered text is not re-parsed"] = _render("{name}", {"id": "t_br", "name": "A{power}B", "kind": "ATTACK"}) == "A{power}B"
checks["a ranked entry renders each rank with its own values"] = _render("{power}", SkillRanks.entry(a, [{"power_mult": 3.0}]), 2) == "300%" and _render("{power}", SkillRanks.entry(a, [{"power_mult": 3.0}]), 1) == "200%"
var parsed: Dictionary = SkillText.parse("a {power} b {rounds:n} c")
var parts: Array = parsed["parts"]
checks["parse splits literals and tokens"] = parts.size() == 5 and (parsed["errors"] as Array).is_empty()
checks["escape makes a plain sentence a safe template"] = SkillText.render(SkillText.escape("100% {x}"), "ATTACK", {"name": "X"}, 1) == "100% {x}"
return TestHelpers.report(TAG, "Formatting rules", checks)
```

### `static _first_error(template: String, entry: Dictionary) -> String`
```logic
var errors: Array[Dictionary] = SkillText.validate(template, entry)
return str(errors[0]["message"]) if not errors.is_empty() else ""
```

### `static _test_validation() -> bool`
```logic
var checks: Dictionary = {}
var a: Dictionary = _attack()
checks["a valid template has no errors"] = SkillText.validate("{name}: {power} for {rounds}", a).is_empty()
checks["an empty template is allowed"] = SkillText.validate("", a).is_empty()
checks["an unknown token is an error and lists the valid ones"] = _first_error("{poer}", a).contains("Unknown token") and _first_error("{poer}", a).contains("power")
checks["a token the kind lacks is an error ({heal} on ATTACK)"] = _first_error("{heal}", a).contains("does not apply")
checks["{potency} on DEFEND and {weapons} on FOCUS are errors"] = not SkillText.validate("{potency}", {"id": "t_d", "kind": "DEFEND"}).is_empty() and not SkillText.validate("{weapons}", {"id": "t_f", "kind": "FOCUS"}).is_empty()
var no_dash: Dictionary = SkillRanks.entry(a, [{"dash_tiles": 0}])
var dash_errors: Array[Dictionary] = SkillText.validate("{dash}", no_dash)
checks["a token that resolves to nothing for some rank is an error naming the rank"] = dash_errors.size() == 1 and DictUtil.to_int(dash_errors[0]["rank"]) == 2 and str(dash_errors[0]["message"]).contains("Rank 2")
var no_status: Dictionary = {"id": "t_ns", "name": "Ns", "kind": "ATTACK", "status_effect": ""}
checks["{status} with no status is an error"] = not SkillText.validate("{status}", no_status).is_empty()
var other_status: Dictionary = {"id": "t_ms", "name": "Ms", "kind": "ATTACK", "status_effect": "stagger", "status_potency": 40}
checks["{magnitude} without weaken is an error"] = not SkillText.validate("{magnitude}", other_status).is_empty()
checks["unbalanced braces are errors"] = not SkillText.validate("{power", a).is_empty() and not SkillText.validate("power}", a).is_empty()
checks["empty braces are an error"] = _first_error("a {} b", a).contains("Empty")
checks["an unknown suffix is an error"] = not SkillText.validate("{power:x}", a).is_empty() and not SkillText.validate("{power:n}", a).is_empty() and SkillText.validate("{dash:n}", a).is_empty()
checks["{unlock} on a basic skill is an error"] = not SkillText.validate("{unlock}", {"id": "t_b", "kind": "ATTACK", "basic": true}).is_empty()
checks["every error carries field description"] = SkillText.validate("{poer}", a).all(func(p: Dictionary) -> bool: return str(p["field"]) == "description")
var entry_problems: Array[String] = TestHelpers.messages(ContentCatalog.validate_skill_entry(SkillRanks.entry(TestHelpers.attack("t_tx_v", {"description": "{poer}"}), [])))
checks["validate_skill_entry reports template errors"] = TestHelpers.any_contains(entry_problems, "Unknown token")
return TestHelpers.report(TAG, "Validation errors", checks)
```

### `static _test_warnings_and_caps() -> bool`
```logic
var checks: Dictionary = {}
var a: Dictionary = _attack()
var digits: Array[String] = SkillText.warnings("heals 10 now", a)
checks["literal digits outside tokens warn and quote the digits"] = digits.size() == 1 and digits[0].contains("10")
checks["...and suggest the matching token when a rank 1 value equals it"] = SkillText.warnings("hits for 4 tiles", a)[0].contains("{dash}") or SkillText.warnings("hits for 4", a)[0].contains("{dash")
checks["no digits, no warning"] = SkillText.warnings("{name}: {power}", a).is_empty()
var literal: Dictionary = SkillRanks.entry(a, [{"level": 4, "description": "Literal rank two."}])
checks["a literal per-rank description warns"] = TestHelpers.any_contains(SkillText.warnings("{name}", literal), "literal description")
checks["...and is used as the rank text"] = str(SkillRanks.resolve(literal, 2)["description"]) == "Literal rank two."
var long_template: String = "x".repeat(SkillText.MAX_LENGTH + 1)
checks["a template over the cap is an error"] = _first_error(long_template, a).contains(str(SkillText.MAX_LENGTH))
var edge: String = "x".repeat(SkillText.MAX_LENGTH - 12) + "{name}"
var wide: Dictionary = a.duplicate()
wide["name"] = "WideName10"
var wide_ranked: Dictionary = SkillRanks.entry(wide, [{"level": 4}])
var rendered_errors: Array[Dictionary] = SkillText.validate(edge, wide_ranked)
checks["a rendered rank text over the cap is an error naming the rank"] = not rendered_errors.is_empty() and DictUtil.to_int(rendered_errors[0]["rank"]) == 2 and str(rendered_errors[0]["message"]).contains("rendered")
checks["the description cap is a schema max_len"] = DictUtil.to_int(ContentSchema.field("skills", "description")["max_len"]) == SkillText.MAX_LENGTH
checks["a flat row over the cap is refused by field validation"] = not ContentCatalog.validate_entry("skills", {"id": "t_cap", "name": "Cap", "kind": "ATTACK", "description": long_template}).is_empty()
return TestHelpers.report(TAG, "Warnings and length caps", checks)
```

### `static _test_warnings_all() -> bool`
```logic
var checks: Dictionary = {}
checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
var head: String = "t_tx_wa"
var digit: String = "7"
var base: Dictionary = TestHelpers.attack(head, {"description": "{name} hits %s times for {power}" % digit})
var mods: Array[Dictionary] = [{"level": 3, "description": "Literal rank two."}, {"level": 4}]
checks["fixture family stored"] = ContentCatalog.upsert_skill(SkillRanks.entry(base, mods)).is_empty()
var warned: Dictionary = ContentCatalog.warnings_all()
var found: Array[String] = ContentFactory._strings(warned.get("skills:%s" % head, []))
checks["the family is keyed by its head id"] = not found.is_empty()
checks["no key per rank row"] = not warned.has("skills:%s" % SkillRanks.rank_id(head, 2)) and not warned.has("skills:%s" % SkillRanks.rank_id(head, 3))
checks["a literal rank description warns through warnings_all"] = TestHelpers.any_contains(found, "literal description")
var digit_hits: int = 0
for text: String in found:
	if text.contains("literal number %s" % digit):
		digit_hits += 1
checks["a literal digit warns once per family, not once per rank"] = digit_hits == 1
return TestHelpers.report(TAG, "warnings_all reads ranked skill entries", checks)
```

