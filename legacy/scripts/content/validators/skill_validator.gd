class_name SkillValidator
extends RefCounted

# Skill validation rules (moved from ContentCatalog, CLN-9): family rules, row rules and soft warnings.
# ContentCatalog.validate_entry / warnings_for dispatch here; the ranked-skill API stays in ContentCatalog.

# One problem of a skill row: {"field", "rank", "message"}.
static func problem(field: String, rank: int, message: String) -> Dictionary:
	return {"field": field, "rank": rank, "message": message}

# Skill family rules (FORGE1 sections 2-3): a valid family id, ids <head> / <head>_<n>, ranks 1..n without gaps (basic 1,
# class 3), one kind per family, and every base-only field (not rank-modifiable, not derived) identical across the ranks.
static func family_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var id: String = str(e.get("id", ""))
	var family: String = str(e.get("family", ""))
	var head: String = id if family.is_empty() else family
	if not family.is_empty() and not ContentSchema.is_valid_id(family):
		errors.append("Family '%s' must be lowercase snake_case (a-z, 0-9, _)." % family)
	var tier: int = DictUtil.to_int(e.get("tier", 1), 1)
	var basic: bool = DictUtil.to_bool(e.get("basic", false))
	var system: bool = DictUtil.to_bool(e.get("system", false))
	var kind: String = str(e.get("kind", ""))
	if basic and tier > 1:
		errors.append("Basic skills have exactly one rank (tier %d is above the 1-tier limit for basic skills)." % tier)
	elif not basic and not system and tier > StatRules.MAX_CLASS_TIER:
		errors.append("Tier %d is above the %d-tier limit for class skills." % [tier, StatRules.MAX_CLASS_TIER])
	if tier <= 1:
		if not family.is_empty() and family != id:
			errors.append("A tier 1 skill is the family head: its id '%s' must equal the family '%s'." % [id, family])
	elif id != SkillRanks.rank_id(head, tier):
		errors.append("Tier %d of family '%s' must have the id '%s' (found '%s')." % [tier, head, SkillRanks.rank_id(head, tier), id])
	var tiers: Array[int] = [tier]
	for other: Dictionary in ContentCatalog.entries("skills"):
		var other_id: String = str(other.get("id", ""))
		if other_id == id or SkillRanks.head_of(other) != head:
			continue
		var other_tier: int = DictUtil.to_int(other.get("tier", 1), 1)
		if other_tier == tier:
			errors.append("Family '%s' already has a tier %d skill (%s)." % [head, tier, other_id])
		tiers.append(other_tier)
		if str(other.get("kind", "")) != kind:
			errors.append("Family '%s' mixes kinds (%s is %s)." % [head, other_id, str(other.get("kind", ""))])
		if DictUtil.to_bool(other.get("basic", false)) != basic:
			errors.append("Family '%s': basic must match across all tiers (%s differs)." % [head, other_id])
		for f: Dictionary in ContentSchema.fields("skills"):
			var key: String = str(f["key"])
			if SkillRanks.DERIVED_KEYS.has(key) or SkillRanks.RANK_KEYS.has(key) or key == "basic" or key == "kind" or not SkillKinds.has_field(kind, key):
				continue
			if other.get(key, null) != e.get(key, null):
				errors.append("Family '%s': %s must match across all tiers (%s differs)." % [head, str(f.get("label", key)).to_lower(), other_id])
	tiers.sort()
	for i: int in range(tiers.size()):
		if tiers[i] != i + 1:
			errors.append("Family '%s': tiers must run 1..n without gaps (it needs a tier %d skill)." % [head, i + 1])
			break
	return errors

static func entry_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = family_errors(e)
	for p: Dictionary in row_problems(e):
		errors.append(str(p["message"]))
	# The description template of a family is validated once, from its first rank, against all the family's ranks.
	var template: String = str(e.get("template", ""))
	if not template.is_empty() and DictUtil.to_int(e.get("tier", 1), 1) <= 1:
		var rows: Array[Dictionary] = [e]
		for other: Dictionary in ContentCatalog.family_rows(str(e.get("id", ""))):
			if str(other.get("id", "")) != str(e.get("id", "")):
				rows.append(other)
		var folded: Dictionary = SkillRanks.fold(str(e.get("id", "")), rows)
		var fold_errors: Array = folded["errors"]
		if fold_errors.is_empty():
			for p: Dictionary in SkillText.validate(template, DictUtil.dict_of(folded["entry"])):
				errors.append(str(p["message"]))
	return errors

# Row-level rules of one flat skill row: [{"field", "rank", "message"}].
static func row_problems(e: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var kind: String = str(e["kind"])
	var id: String = str(e["id"])
	var tier: int = DictUtil.to_int(e.get("tier", 1), 1)
	var basic: bool = DictUtil.to_bool(e.get("basic", false))
	var system: bool = DictUtil.to_bool(e.get("system", false))
	var grade: String = str(e["grade"])
	var unlock_offset: int = DictUtil.to_int(e.get("unlock_offset", -1), -1)
	if unlock_offset < -1 or unlock_offset > StatRules.MAX_UNLOCK_OFFSET:
		out.append(problem("unlock_offset", tier, "Unlock offset must be -1 (derived) or 0-%d." % StatRules.MAX_UNLOCK_OFFSET))
	elif unlock_offset > 0 and tier == 1:
		out.append(problem("unlock_offset", tier, "A tier 1 skill opens at its family's unlock level: leave the unlock offset at -1 or 0."))
	var requires: Array[String] = ContentFactory._strings(e.get("requires", []))
	if (basic or system) and grade != "novice":
		out.append(problem("grade", tier, "Only class skills can be advanced (not basic or engine skills)."))
	if (basic or system) and not requires.is_empty():
		out.append(problem("requires", tier, "Basic and engine skills cannot require other skills."))
	var head: String = SkillRanks.head_of(e)
	for required: String in requires:
		var target: Dictionary = ContentCatalog.get_entry("skills", required)
		if target.is_empty():
			continue
		if required == head:
			out.append(problem("requires", tier, "A skill cannot require itself."))
		elif not str(target.get("family", "")).is_empty() or DictUtil.to_int(target.get("tier", 1), 1) != 1:
			out.append(problem("requires", tier, "Requires: '%s' is a higher rank; name the family head." % required))
		elif ContentCatalog.grade_rank(str(target.get("grade", "novice"))) >= ContentCatalog.grade_rank(grade):
			out.append(problem("requires", tier, "Requires: '%s' (%s) must be a lower grade than this skill (%s)." % [required, str(target.get("grade", "novice")), grade]))
	if tier == 1:
		for other: Dictionary in ContentCatalog.entries("skills"):
			if SkillRanks.head_of(other) == head or not ContentFactory._strings(other.get("requires", [])).has(head):
				continue
			if DictUtil.to_int(other.get("tier", 1), 1) == 1 and ContentCatalog.grade_rank(str(other.get("grade", "novice"))) <= ContentCatalog.grade_rank(grade):
				out.append(problem("grade", tier, "Grade: '%s' requires this skill and must stay a higher grade (a grade change would break it)." % str(other.get("id", ""))))
	if not system and not SkillKinds.is_authorable(kind):
		out.append(problem("kind", tier, "%s skills are engine-only; author an %s skill." % [kind, " / ".join(SkillKinds.authorable())]))
	# One rule replaces the old per-field "belongs to" messages: a field is set but the kind has no section for it.
	var guarded: Array[String] = SkillKinds.sectioned_fields()
	guarded.append("stance")
	for key: String in guarded:
		if SkillKinds.has_field(kind, key) or key == "extra_grants":   # extra_grants has its own message (extra_grant_problems)
			continue
		var f: Dictionary = ContentSchema.field("skills", key)
		if e.get(key, null) != f["default"]:
			out.append(problem(key, tier, "%s belongs to %s skills, not %s." % [str(f["label"]), "/".join(SkillKinds.kinds_with(key)), kind]))
	var status_id: String = str(e["status_effect"])
	# The magnitude status (status_effect, else grant_status) decides the range from its registry row (S12S-L).
	var magnitude: int = DictUtil.to_int(e["status_magnitude"])
	var magnitude_status: String = SkillText.magnitude_status(e)
	if StatusEffects.has_magnitude(magnitude_status):
		var mag_min: int = StatusRegistry.magnitude_min(magnitude_status)
		var mag_max: int = StatusRegistry.magnitude_max(magnitude_status)
		if magnitude < mag_min or magnitude > mag_max:
			out.append(problem("status_magnitude", tier, "%s needs a magnitude %d-%d" % [magnitude_status, mag_min, mag_max]))
	elif magnitude != 0:
		out.append(problem("status_magnitude", tier, "magnitude only applies to %s" % ", ".join(StatusRegistry.magnitude_ids())))
	var grant: String = str(e["grant_status"])
	var reach_min: int = DictUtil.to_int(e["reach_min"])
	var reach_max: int = DictUtil.to_int(e["reach_max"])
	if kind == "SUPPORT":
		var heal_stat: String = str(e["heal_stat"])
		var heal_mult: float = DictUtil.to_float(e["heal_mult"])
		var heal_base: int = DictUtil.to_int(e["heal_base"])
		var cleanse: Array[String] = ContentFactory._strings(e["cleanse_statuses"])
		if heal_base <= 0 and heal_mult <= 0.0 and grant.is_empty() and cleanse.is_empty():
			out.append(problem("heal_base", tier, "A support skill needs a heal (base or per-stat), a cleanse or a granted status."))
		if cleanse.has(SkillDef.CLEANSE_ALL) and cleanse.size() > 1:
			out.append(problem("cleanse_statuses", tier, "Cleanses: '%s' stands alone (it already clears every debuff)." % SkillDef.CLEANSE_ALL))
		for status: String in cleanse:
			if status != SkillDef.CLEANSE_ALL and not StatusEffects.debuffs().has(status):
				out.append(problem("cleanse_statuses", tier, "Cleanses: '%s' is not a debuff status (%s, or '%s')." % [status, ", ".join(StatusEffects.debuffs()), SkillDef.CLEANSE_ALL]))
		if heal_mult > 0.0 and not SupportRules.HEAL_STATS.has(heal_stat):
			out.append(problem("heal_stat", tier, "Heal per stat point needs a heal stat (INT or KNOW)."))
		if heal_mult <= 0.0 and not heal_stat.is_empty():
			out.append(problem("heal_stat", tier, "Heal stat is set but heal per stat point is 0."))
		if not grant.is_empty() and not StatusEffects.support_buffs().has(grant):
			out.append(problem("grant_status", tier, "Grants: '%s' is not a support buff status (%s)." % [grant, ", ".join(StatusEffects.support_buffs())]))
		if reach_max > 0 and reach_min > reach_max:
			out.append(problem("reach_min", tier, "Reach min is above reach max."))
		if grant == StatusEffects.TAUNT and str(e["target_mode"]) != "SELF":
			out.append(problem("target_mode", tier, "Taunt is cast on the taunter itself: set Targets to SELF."))
	elif kind == "DEFEND":
		var defend_options: Array[String] = StatusEffects.defend_buffs()
		defend_options.append_array(StatusEffects.support_buffs())
		if not grant.is_empty() and not defend_options.has(grant):
			out.append(problem("grant_status", tier, "Grants: '%s' is not a defend or support buff status (%s)." % [grant, ", ".join(defend_options)]))
	if kind == "DISENGAGE" and DictUtil.to_int(e["jump_tiles"]) < 1:
		out.append(problem("jump_tiles", tier, "A disengage skill needs jump tiles of at least 1."))
	if kind == "ATTACK":
		var area: String = str(e["area"])
		var dash: int = DictUtil.to_int(e["dash_tiles"])
		var hand: String = str(e["weapon_hand"])
		if DictUtil.to_float(e["power_mult"]) <= 0.0:
			if status_id.is_empty():
				out.append(problem("power_mult", tier, "A skill with no damage needs a status"))
			if area != "NONE":
				out.append(problem("area", tier, "A skill with no damage can't be an area attack."))
		if area == "SWEEP" and (dash > 0 or reach_max > 1):
			out.append(problem("area", tier, "SWEEP hits the L of tiles around you; leave reach at 0 or 1 and no gap-close."))
		if dash > 0:
			if area != "NONE":
				out.append(problem("dash_tiles", tier, "An area attack can't be a gap-close."))
			if hand != "main":
				out.append(problem("dash_tiles", tier, "An off-hand or dual-wield strike can't be a gap-close."))
		if hand == "both" and not SkillDef.BOTH_ENABLED:
			out.append(problem("weapon_hand", tier, "Weapon hand 'both' is not playable yet (engine pending)."))
		if not status_id.is_empty() and not StatusEffects.debuffs().has(status_id):
			out.append(problem("status_effect", tier, "Inflicts: '%s' is not a debuff status (%s)." % [status_id, ", ".join(StatusEffects.debuffs())]))
		if reach_max > 0 and reach_min > reach_max:
			out.append(problem("reach_min", tier, "Reach min is above reach max."))
		if reach_max == 0 and reach_min > 0:
			out.append(problem("reach_max", tier, "Set reach max too (or leave both at 0 for the unit's default band)."))
		if area == "LINE" and reach_max > 1:
			out.append(problem("reach_max", tier, "LINE areas start from an adjacent tile; leave the reach at 0 or 1."))
		if not status_id.is_empty() and DictUtil.to_int(e["status_potency"]) <= 0:
			out.append(problem("status_potency", tier, "Give the status a potency above 0."))
	out.append_array(extra_grant_problems(e, kind, tier))
	out.append_array(combo_problems(e, kind, tier))
	# The weapon gate (ATTACK, DEFEND, SUPPORT): known gear types, and a type needs a proficiency to mean anything.
	var types: Array[String] = ContentFactory._strings(e["weapon_types"])
	for gear: String in types:
		if not SkillDef.weapon_type_tokens().has(gear):
			out.append(problem("weapon_types", tier, "Needs weapon type: unknown gear type '%s' (gear types or '%s')." % [gear, SkillDef.UNARMED_TOKEN]))
	if not types.is_empty() and DictUtil.to_int(e["min_proficiency"]) < 1:
		out.append(problem("min_proficiency", tier, "Needs proficiency: a weapon type is listed, so give a proficiency of at least 1 (otherwise the types do nothing)."))
	return out

const EXTRA_GRANTS_MAX_ENTRIES: int = 1
const EXTRA_GRANT_ROUNDS_MIN: int = 1
const EXTRA_GRANT_ROUNDS_MAX: int = 3

# extra_grants (BAL5-L, spec BAL5_LEVERS section 2 T3): DEFEND or SUPPORT only, at most 1 entry, a buff the kind can grant
# (DEFEND: defend or support buffs, SUPPORT: support buffs) other than grant_status, rounds 1-3, magnitude inside the
# status row's range (0 for a status without one). A taunt extra follows the grant_status rule: SUPPORT targets SELF.
static func extra_grant_problems(e: Dictionary, kind: String, tier: int) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var extras: Array = DictUtil.array_of(e.get("extra_grants", []))
	if extras.is_empty():
		return out
	if kind != "DEFEND" and kind != "SUPPORT":
		out.append(problem("extra_grants", tier, "extra_grants needs a DEFEND or SUPPORT skill."))
		return out
	if extras.size() > EXTRA_GRANTS_MAX_ENTRIES:
		out.append(problem("extra_grants", tier, "A skill holds at most one extra grant."))
	var options: Array[String] = StatusEffects.support_buffs()
	if kind == "DEFEND":
		options.append_array(StatusEffects.defend_buffs())
	var grant: String = str(e.get("grant_status", ""))
	for raw: Variant in extras:
		var row: Dictionary = DictUtil.dict_of(raw)
		var status: String = str(row.get("status", ""))
		if not options.has(status) or status == grant:
			out.append(problem("extra_grants", tier, "Extra grant '%s' must be another buff." % status))
			continue
		var rounds: int = DictUtil.to_int(row.get("rounds", 0))
		if rounds < EXTRA_GRANT_ROUNDS_MIN or rounds > EXTRA_GRANT_ROUNDS_MAX:
			out.append(problem("extra_grants", tier, "Extra grant '%s' needs rounds %d-%d." % [status, EXTRA_GRANT_ROUNDS_MIN, EXTRA_GRANT_ROUNDS_MAX]))
		var magnitude: int = DictUtil.to_int(row.get("magnitude", 0))
		if StatusEffects.has_magnitude(status):
			var mag_min: int = StatusRegistry.magnitude_min(status)
			var mag_max: int = StatusRegistry.magnitude_max(status)
			if magnitude < mag_min or magnitude > mag_max:
				out.append(problem("extra_grants", tier, "Extra grant '%s' needs a magnitude %d-%d." % [status, mag_min, mag_max]))
		elif magnitude != 0:
			out.append(problem("extra_grants", tier, "Extra grant '%s' takes no magnitude (leave it 0)." % status))
		if status == StatusEffects.TAUNT and kind == "SUPPORT" and str(e.get("target_mode", "")) != "SELF":
			out.append(problem("target_mode", tier, "Taunt is cast on the taunter itself: set Targets to SELF."))
	return out

const EXPLOIT_MAX_ENTRIES: int = 2
const EXPLOIT_MULT_MIN: float = 1.05
const EXPLOIT_MULT_MAX: float = 2.0

# Combo fields (S12C-L): flow end stance, stance-break disrupt, exploit list and consume. Fields the kind does not show
# are reported by the generic "belongs to" rule above.
static func combo_problems(e: Dictionary, kind: String, tier: int) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var ends: String = str(e.get("ends_in_stance", ""))
	if not ends.is_empty() and SkillKinds.has_field(kind, "ends_in_stance"):
		var own: CombatTypes.StanceType = ContentFactory.skill(e).stance()
		if ends == str(CombatTypes.StanceType.keys()[own]):
			out.append(problem("ends_in_stance", tier, "Ends in stance equals the skill's stance; leave it empty."))
		if kind == "DEFEND" and str(e.get("grant_status", "")) == StatusEffects.RIPOSTE:
			out.append(problem("ends_in_stance", tier, "A riposte skill cannot flow: a stance change ends riposte."))
	if kind != "ATTACK":
		return out
	var zero_power: bool = DictUtil.to_float(e.get("power_mult", 1.0), 1.0) <= 0.0
	if str(e.get("disrupt", "NONE")) == "STANCE_BREAK" and zero_power:
		out.append(problem("disrupt", tier, "A zero-power attack cannot break a stance."))
	var exploit: Array = DictUtil.array_of(e.get("exploit", []))
	if exploit.size() > EXPLOIT_MAX_ENTRIES:
		out.append(problem("exploit", tier, "Exploit takes at most 2 statuses."))
	var seen: Array[String] = []
	var debuffs: Array[String] = StatusEffects.debuffs()
	for raw: Variant in exploit:
		var row: Dictionary = DictUtil.dict_of(raw)
		var status: String = str(row.get("status", ""))
		if not debuffs.has(status):
			out.append(problem("exploit", tier, "Exploit status '%s' is not a debuff." % status))
		elif seen.has(status):
			out.append(problem("exploit", tier, "Exploit lists '%s' twice." % status))
		seen.append(status)
		var mult: float = DictUtil.to_float(row.get("power_mult", 0.0))
		if mult < EXPLOIT_MULT_MIN or mult > EXPLOIT_MULT_MAX:
			out.append(problem("exploit", tier, "Exploit power must be 1.05-2.0."))
	if not exploit.is_empty() and zero_power:
		out.append(problem("exploit", tier, "A zero-power attack cannot exploit."))
	if DictUtil.to_bool(e.get("exploit_consume", false)) and exploit.is_empty():
		out.append(problem("exploit_consume", tier, "Consume needs an exploit status."))
	return out

# `raw` is a ranked entry (no tier / family keys) or a flat row; `e` its normalized form.
static func warnings(raw: Dictionary, e: Dictionary) -> Array[String]:
	var out: Array[String] = []
	var ranked: bool = not raw.has("tier") and not raw.has("family")
	if ranked:
		out.append_array(SkillText.warnings(str(e["description"]), raw))
		var base: Dictionary = SkillRanks.effective(raw, 1)
		var mods: Array[Dictionary] = SkillRanks.modifiers_of(raw)
		for i: int in range(mods.size()):
			for key: Variant in mods[i]:
				if SkillRanks.RANK_KEYS.has(str(key)) and base.get(str(key), null) == mods[i][key]:
					out.append("Rank %d: %s equals the base value; the modifier is dropped on save." % [i + 2, str(key)])
	elif not str(e.get("template", "")).is_empty():
		out.append_array(SkillText.warnings(str(e["template"]), e))
	if str(e["weapon_hand"]) != "main":
		for gear: String in ContentFactory._strings(e["weapon_types"]):
			if gear != SkillDef.UNARMED_TOKEN:
				out.append("Weapon hand '%s' with weapon type '%s': off-hand slots hold shields, orbs and tomes, so only 'unarmed' can satisfy this gate." % [str(e["weapon_hand"]), gear])
	return out
