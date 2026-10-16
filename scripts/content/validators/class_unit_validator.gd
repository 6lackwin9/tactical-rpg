class_name ClassUnitValidator
extends RefCounted

# Class, unit and loadout validation rules (moved from ContentCatalog, CLN-9); ContentCatalog.validate_entry dispatches here.

# Class rules: equal stat budget and growth (StatRules), valid proficiency, a skill path whose
# skills fit the level cap, and default gear the class can wield.
static func class_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = StatRules.validate_class(ContentFactory._ints(e["base_stats"]), ContentFactory._ints(e["growth"]))
	if not UnitArt.CLASS_KEYS.has(str(e["art"])):
		errors.append("Art '%s' isn't a known token/portrait key." % str(e["art"]))
	var prof_raw: Variant = e["proficiency"]
	var weapons: int = 0
	if not (prof_raw is Dictionary) or (prof_raw as Dictionary).is_empty():
		errors.append("A class needs a gear proficiency dictionary.")
	else:
		var prof: Dictionary = prof_raw
		for key: Variant in prof:
			var gear: String = str(key)
			var entry: Dictionary = DictUtil.dict_of(prof[key])
			if not Proficiency.gear_types().has(gear):
				errors.append("Proficiency: unknown gear type '%s'." % gear)
			if not entry.has("level") or DictUtil.to_int(entry.get("level", 0)) < 1 or DictUtil.to_int(entry.get("level", 0)) > Proficiency.MAX_LEVEL:
				errors.append("Proficiency %s: level must be 1-%d." % [gear, Proficiency.MAX_LEVEL])
			var scaling: float = DictUtil.to_float(entry.get("scaling", Proficiency.DEFAULT_SCALING), -1.0)
			if scaling < Proficiency.MIN_SCALING or scaling > Proficiency.MAX_SCALING:
				errors.append("Proficiency %s: scaling must be %.1f-%.1f." % [gear, Proficiency.MIN_SCALING, Proficiency.MAX_SCALING])
			if Proficiency.WEAPON_TYPES.has(gear):
				weapons += 1
		if weapons == 0:
			errors.append("Proficiency needs at least one weapon type.")
	var path: Variant = e["skill_path"]
	if not (path is Array):
		errors.append("The skill path must be a list of {level, skill}.")
	else:
		var last_level: int = 0
		var seen_families: Array[String] = []
		var family_levels: Dictionary = {}   # family head -> lowest path level (for the `requires` rule)
		for step_raw: Variant in (path as Array):
			var raw_step: Dictionary = DictUtil.dict_of(step_raw)
			var raw_skill: String = str(raw_step.get("skill", ""))
			var raw_fam: String = SkillRanks.head_of(ContentCatalog.get_entry("skills", raw_skill)) if ContentCatalog.has("skills", raw_skill) else raw_skill
			var raw_level: int = DictUtil.to_int(raw_step.get("level", 0))
			if not family_levels.has(raw_fam) or raw_level < DictUtil.int_of(family_levels, raw_fam):
				family_levels[raw_fam] = raw_level
		for step_raw: Variant in (path as Array):
			var step: Dictionary = DictUtil.dict_of(step_raw)
			var skill_id: String = str(step.get("skill", ""))
			var level: int = DictUtil.to_int(step.get("level", 0))
			var skill: Dictionary = ContentCatalog.get_entry("skills", skill_id)
			if level < 1 or level > StatRules.MAX_LEVEL:
				errors.append("Skill path: level %d is outside 1-%d." % [level, StatRules.MAX_LEVEL])
			if level < last_level:
				errors.append("Skill path must be listed in ascending level order.")
			last_level = maxi(last_level, level)
			if skill.is_empty():
				errors.append("Skill path: unknown skill '%s'." % skill_id)
			elif DictUtil.to_bool(skill.get("system", false)):
				errors.append("Skill path: '%s' is an engine skill (always known)." % skill_id)
			elif DictUtil.to_bool(skill.get("basic", false)):
				errors.append("Skill path: '%s' is a basic skill; every unit owns the basics from level 1, so they are not valid in a class path." % skill_id)
			else:
				var fam: String = str(skill.get("family", ""))
				if fam.is_empty():
					fam = skill_id
				if DictUtil.to_int(skill.get("tier", 1), 1) != 1:
					errors.append("Skill path: '%s' is a higher tier; list the tier 1 skill of the family (tiers open with it)." % skill_id)
				if seen_families.has(fam):
					errors.append("Skill path: family '%s' is listed more than once." % fam)
				seen_families.append(fam)
				if str(skill.get("grade", "novice")) != "novice" and level < StatRules.ADVANCED_MIN_LEVEL:
					errors.append("Skill path: %s is an advanced skill and needs unit level %d or higher (listed at %d)." % [skill_id, StatRules.ADVANCED_MIN_LEVEL, level])
				for required: String in ContentFactory._strings(skill.get("requires", [])):
					if not family_levels.has(required):
						errors.append("Skill path: %s needs %s on the same path." % [skill_id, required])
					elif DictUtil.int_of(family_levels, required) > level:
						errors.append("Skill path: %s (level %d) needs %s first, but %s is only listed at level %d." % [skill_id, level, required, required, DictUtil.int_of(family_levels, required)])
	errors.append_array(_loadout_errors(e, ContentFactory.class_archetype(e).make_proficiency(), true))
	return errors

# Gear problems of a class or unit entry: every item fits its slot, is trained for by
# `prof` (when `check_training`), and a two-handed weapon isn't paired with a shield.
static func _loadout_errors(e: Dictionary, prof: Proficiency, check_training: bool) -> Array[String]:
	var errors: Array[String] = []
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		var item_id: String = str(e.get(slot, ""))
		if item_id.is_empty():
			continue
		var item: Dictionary = ContentCatalog.get_entry("items", item_id)
		if item.is_empty():
			continue
		if not ContentFactory.item_fits_slot(item, slot.to_upper()):
			errors.append("%s can't be equipped in %s." % [str(item.get("name", item_id)), slot.capitalize()])
		elif check_training and not str(item.get("weapon_type", "")).is_empty() and not prof.can_use(str(item["weapon_type"])):
			errors.append("This class has no training in '%s' (%s)." % [str(item["weapon_type"]), str(item.get("name", item_id))])
	var main: Dictionary = ContentCatalog.get_entry("items", str(e.get("main_hand", "")))
	var off: Dictionary = ContentCatalog.get_entry("items", str(e.get("off_hand", "")))
	if DictUtil.to_bool(main.get("two_handed", false)) and not off.is_empty():
		errors.append("A two-handed weapon can't be paired with an off-hand item.")
	return errors

static func unit_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var fixed: bool = str(e["stat_mode"]) == "fixed"
	var arch: ClassArchetype = ContentFactory.class_of(e)
	if not fixed and arch == null:
		errors.append("Class-derived units need a class (or set the stat model to fixed).")
	if not fixed:
		errors.append_array(StatRules.validate_spend(ContentFactory._ints(e["spend"]), DictUtil.to_int(e["level"], 1)))
	elif not (e["spend"] as Dictionary).is_empty():
		errors.append("Fixed units don't spend points.")
	var prof: Proficiency = arch.make_proficiency() if arch != null else Proficiency.new()
	errors.append_array(_loadout_errors(e, prof, arch != null))
	if arch != null and not fixed:
		errors.append_array(_skill_loadout_errors(e, arch))
	errors.append_array(_unit_requires_errors(e, arch))
	return errors

# A skill in `extra_skills` or `loadout` needs every family of its `requires` on the class path (any level) or in extra_skills.
static func _unit_requires_errors(e: Dictionary, arch: ClassArchetype) -> Array[String]:
	var errors: Array[String] = []
	var owned: Array[String] = []
	if arch != null:
		for step: Dictionary in arch.skill_path:
			owned.append(SkillRanks.head_of(ContentCatalog.get_entry("skills", str(step.get("skill", "")))))
	var extras: Array[String] = ContentFactory._strings(e.get("extra_skills", []))
	for extra: String in extras:
		owned.append(SkillRanks.head_of(ContentCatalog.get_entry("skills", extra)))
	var checked: Array[String] = []
	checked.append_array(extras)
	checked.append_array(ContentFactory._strings(e.get("loadout", [])))
	for skill_id: String in checked:
		var skill: Dictionary = ContentCatalog.get_entry("skills", skill_id)
		for required: String in ContentFactory._strings(skill.get("requires", [])):
			if not owned.has(required):
				errors.append("Unit skill '%s' needs '%s' (on the class path or in the extra skills)." % [skill_id, required])
	return errors

# Active skill loadout of a unit: listed families must be owned (class path up to its level or extra skills),
# non-basic, unique, and fit MAX_CLASS_SLOTS class + MAX_EXTRA_SLOTS extra slots. The DEFAULT loadout
# (no `loadout` list) is trimmed to the limits, so only an explicit list can break them.
static func _skill_loadout_errors(e: Dictionary, arch: ClassArchetype) -> Array[String]:
	var errors: Array[String] = []
	var listed: Array[String] = ContentFactory._strings(e.get("loadout", []))
	if listed.is_empty():
		return errors
	var level: int = DictUtil.to_int(e.get("level", 1), 1)
	var owned_ids: Array[String] = arch.skills_at(level)
	owned_ids.append_array(ContentFactory._strings(e.get("extra_skills", [])))
	var owned: Array[String] = ContentFactory.families_of(owned_ids)
	var seen: Array[String] = []
	for fam: String in listed:
		if seen.has(fam):
			errors.append("Loadout: family '%s' is listed twice." % fam)
		seen.append(fam)
		if SkillCatalog.family_tiers(fam).is_empty():
			errors.append("Loadout: unknown skill family '%s'." % fam)
		elif SkillCatalog.is_basic_family(fam):
			errors.append("Loadout: '%s' is a basic family (always active, no slot)." % fam)
		elif not owned.has(fam):
			errors.append("Loadout: the unit does not own the skill family '%s'." % fam)
	var slots: Dictionary = ContentFactory.loadout_of(e, arch, level)
	var class_count: int = ContentFactory._strings(slots["class"]).size()
	var extra_count: int = ContentFactory._strings(slots["extra"]).size()
	if class_count > StatRules.MAX_CLASS_SLOTS:
		errors.append("Loadout: %d class families (max %d)." % [class_count, StatRules.MAX_CLASS_SLOTS])
	if extra_count > StatRules.MAX_EXTRA_SLOTS:
		errors.append("Loadout: %d extra families (max %d)." % [extra_count, StatRules.MAX_EXTRA_SLOTS])
	return errors
