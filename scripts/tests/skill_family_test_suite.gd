class_name SkillFamilyTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill Family & Loadout Test Suite (milestone M2c-2a1)
# Skill families (tiers of one skill, `family` + `tier` + `basic`), planning-time tier options
# grouped by family, and the active loadout (max StatRules.MAX_CLASS_SLOTS class + MAX_EXTRA_SLOTS
# extra families, basics free, locked during battle). Expectations come from the rules and the
# catalog data, not from magic numbers.
# ==============================================================================

const SANDBOX: String = "user://skill_family_test"

static func _report(label: String, checks: Dictionary) -> bool:
	var failed: Array[String] = []
	for key: Variant in checks:
		if not DictUtil.to_bool(checks[key]):
			failed.append(str(key))
	var ok: bool = failed.is_empty()
	print("    [SkillFamily] %s: %s%s" % [label, "PASS" if ok else "FAIL", "" if ok else " (%s)" % ", ".join(failed)])
	return ok

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING SKILL FAMILY TEST (M2c-2a1)             ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var ok: bool = true
	ok = _test_migration() and ok
	ok = _test_planner_tiers() and ok
	ok = _test_loadout_filter() and ok
	ok = _test_set_loadout_limits() and ok
	ok = _test_locked_in_battle() and ok
	ok = _test_default_loadout() and ok
	ok = _test_unit_override() and ok
	ok = _test_round_trip_and_validation() and ok
	ok = _test_family_rename() and ok
	return ok

static func _ids(skills: Array[SkillDef]) -> Array[String]:
	var out: Array[String] = []
	for s: SkillDef in skills:
		out.append(s.id)
	return out

# ------------------------------------------------------------------------------

static func _test_migration() -> bool:
	var checks: Dictionary = {}
	# Basics v3: exactly six single-tier basics with no variants; the old tier skills are gone.
	var basics: Array[String] = []
	var singles_ok: bool = true
	for fam: String in SkillCatalog.families():
		if SkillCatalog.is_basic_family(fam):
			basics.append(fam)
			var tiers_of: Array[SkillDef] = SkillCatalog.family_tiers(fam)
			singles_ok = singles_ok and tiers_of.size() == 1 and tiers_of[0].tier == 1 and tiers_of[0].id == fam and tiers_of[0].basic
	basics.sort()
	checks["basic families are exactly the six basics"] = basics == ["guard", "move", "offhand_jab", "rest", "strike", "switch_target"]
	checks["every basic is a single tier-1 skill, its own family"] = singles_ok
	var removed_ok: bool = true
	for gone: String in ["power_strike", "heavy_blow", "brace", "fortress", "deep_focus"]:
		removed_ok = removed_ok and SkillCatalog.find(gone) == null
	checks["removed variants are not in the catalog"] = removed_ok
	var tiers_ok: bool = true
	for fam: String in SkillCatalog.families():
		var tiers: Array[SkillDef] = SkillCatalog.family_tiers(fam)
		for i: int in range(tiers.size()):
			tiers_ok = tiers_ok and tiers[i].tier == i + 1
	checks["tiers are numbered 1..n in order"] = tiers_ok
	var strike: SkillDef = SkillCatalog.find("strike")
	checks["strike keeps AP = level and 2 blocks"] = strike.ap_cost() == strike.level and strike.blocks == 2
	checks["a single skill is its own family (tier 1)"] = SkillCatalog.family_of("mend") == "mend" and SkillCatalog.find("mend").tier == 1 and SkillCatalog.family_tiers("mend").size() == 1
	checks["focus is a single class-owned skill"] = SkillCatalog.family_tiers("focus").size() == 1 and not SkillCatalog.is_basic_family("focus")
	checks["class families are not basic"] = not SkillCatalog.is_basic_family("focus") and not SkillCatalog.is_basic_family("mend")
	checks["engine skills are always active"] = SkillCatalog.is_always_active("move") and SkillCatalog.is_always_active("rest")
	var kinds_ok: bool = true
	var cap_ok: bool = true
	for fam: String in SkillCatalog.families():
		var tiers: Array[SkillDef] = SkillCatalog.family_tiers(fam)
		for s: SkillDef in tiers:
			kinds_ok = kinds_ok and s.kind == tiers[0].kind
			cap_ok = cap_ok and s.tier <= (StatRules.MAX_TIER if s.basic else StatRules.MAX_CLASS_TIER)
	checks["every family has one kind and respects the tier limit"] = kinds_ok and cap_ok
	checks["limits are the named constants 5 / 3"] = StatRules.MAX_CLASS_SLOTS == 5 and StatRules.MAX_EXTRA_SLOTS == 3
	checks["whole catalog validates"] = ContentCatalog.validate_all().is_empty()
	return _report("Migrated families and unchanged numbers", checks)

# ------------------------------------------------------------------------------

# The shipped catalog has no multi-tier family (basics are single-tier; class tiers arrive with M2c-2b),
# so tier machinery is tested on a three-tier fixture family in a sandbox copy. Caller restores the root.
const FIXTURE: String = "t_fix"

static func _fork_with_fixture() -> bool:
	for kind: String in ContentSchema.KINDS:
		DirAccess.remove_absolute(ContentCatalog.path_for(kind, SANDBOX))
	if ContentCatalog.fork_to(SANDBOX) != OK:
		return false
	var ok: bool = true
	for t: int in [1, 2, 3]:
		var id: String = FIXTURE if t == 1 else "%s_%d" % [FIXTURE, t]
		var entry: Dictionary = {"id": id, "name": "Fix Strike" if t == 1 else "Fix Strike %s" % SkillCatalog.roman(t), "kind": "ATTACK", "level": t, "blocks": 2, "tier": t, "family": FIXTURE, "power_mult": 1.0 + 0.5 * float(t - 1)}
		ok = ContentCatalog.upsert("skills", entry).is_empty() and ok
	return ok

static func _test_planner_tiers() -> bool:
	var checks: Dictionary = {}
	checks["fixture family set up"] = _fork_with_fixture()
	checks["fixture family has 3 tiers"] = SkillCatalog.family_tiers(FIXTURE).size() == 3
	checks["tier members share family_id()"] = SkillCatalog.find("t_fix_3").family_id() == FIXTURE and SkillCatalog.find(FIXTURE).family_id() == FIXTURE
	var fix_tiers: Array[SkillDef] = SkillCatalog.family_tiers(FIXTURE)
	checks["each tier costs more AP than the one below"] = fix_tiers.size() == 3 and fix_tiers[0].ap_cost() < fix_tiers[1].ap_cost() and fix_tiers[1].ap_cost() < fix_tiers[2].ap_cost()
	for cap: int in [1, 2, 3]:
		var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
		var hero := Combatant.new("Hero", 200, 10, 1)
		var foe := Combatant.new("Foe", 200, 10, 1)
		grid.place_combatant(hero, Vector2i(3, 4))
		grid.place_combatant(foe, Vector2i(4, 4))
		var ctx: PlanContext = ArenaTestSuite._hand_ctx(grid, [hero], [foe], cap)
		var plan := CombatPlan.new(ctx)
		var options: Array[PlanOption] = plan.target_options(hero, foe)
		var groups: Array[Dictionary] = PlanOption.group_by_family(options)
		var strike_group: Dictionary = {}
		for g: Dictionary in groups:
			if str(g["family"]) == FIXTURE:
				strike_group = g
		var members: Array[PlanOption] = []
		if not strike_group.is_empty():
			members = strike_group["options"]
		var skill_ids: Array[String] = []
		var ap_ok: bool = true
		for o: PlanOption in members:
			skill_ids.append(o.step.skill.id)
			ap_ok = ap_ok and o.valid and (o.ap - o.switch_ap) == o.step.skill.ap_cost(hero)
		var expected: Array[String] = []
		for s: SkillDef in SkillCatalog.family_tiers(FIXTURE):
			if s.level <= cap:
				expected.append(s.id)
		checks["cap %d: strike tiers offered = those with AP level <= cap" % cap] = skill_ids == expected and not expected.is_empty()
		checks["cap %d: option AP matches the tier's AP cost" % cap] = ap_ok
		var one_group: bool = true
		var seen: Array[String] = []
		for g: Dictionary in groups:
			one_group = one_group and not seen.has(str(g["family"]))
			seen.append(str(g["family"]))
		checks["cap %d: each family appears once, tiers stay inside it" % cap] = one_group
		if cap == 3:
			checks["label reads 'Fix Strike I / II / III'"] = str(strike_group.get("label", "")) == "Fix Strike I / II / III"
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	# Grouping at the catalog level (priest with Mend learned at level 3 sees one 'Mend' option).
	var priest: Combatant = ContentFactory.unit({"id": "t_sf_priest", "name": "P", "class_id": "priest", "level": 5})
	var groups: Array[Dictionary] = SkillCatalog.grouped(SkillDef.Kind.DEFEND, 5, priest)
	var guard_group: Dictionary = {}
	for g: Dictionary in groups:
		if str(g["family"]) == "guard":
			guard_group = g
	var tiers: Array[SkillDef] = []
	if not guard_group.is_empty():
		tiers = guard_group["tiers"]
	var expected_guard: Array[String] = []
	for s: SkillDef in SkillCatalog.family_tiers("guard"):
		if priest.can_use(s):
			expected_guard.append(s.id)
	checks["grouped(): guard tiers are what the priest can use"] = _ids(tiers) == expected_guard and not expected_guard.is_empty()
	return _report("Planner offers tiers grouped by family", checks)

# ------------------------------------------------------------------------------

static func _mage() -> Combatant:
	return ContentFactory.unit({"id": "t_sf_mage", "name": "M", "class_id": "mage", "level": 5})

static func _test_loadout_filter() -> bool:
	var checks: Dictionary = {}
	var mage: Combatant = _mage()
	var firebomb: SkillDef = SkillCatalog.find("firebomb")
	var volley: SkillDef = SkillCatalog.find("volley")
	checks["default loadout: every owned class skill usable"] = mage.can_use(firebomb) and mage.can_use(volley)
	var err: String = mage.set_loadout(["arcane_bolt", "volley", "focus", "hex"], [])
	checks["set_loadout succeeds outside combat"] = err.is_empty() and mage.active_families.size() == 4
	checks["a family outside the loadout is not usable"] = not mage.can_use(firebomb) and not mage.knows_skill("firebomb") and mage.owns_skill("firebomb")
	checks["in-loadout families still usable"] = mage.can_use(volley) and mage.can_use(SkillCatalog.find("hex"))
	checks["basics stay usable without a slot"] = mage.can_use(SkillCatalog.find("strike")) and mage.can_use(SkillCatalog.find("guard"))
	checks["engine skills stay usable"] = mage.knows_skill("move") and mage.knows_skill("rest")
	var avail: Array[String] = _ids(SkillCatalog.available(SkillDef.Kind.ATTACK, 5, mage))
	checks["SkillCatalog.available hides non-loadout skills"] = not avail.has("firebomb") and avail.has("volley") and avail.has("strike")
	var singles: Array[String] = _ids(SkillCatalog.single_attacks(mage, 5))
	checks["single_attacks hides them too"] = not singles.has("firebomb") and singles.has("arcane_bolt")
	# The planner offers no option for a hidden skill.
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
	var foe := Combatant.new("Foe", 200, 10, 1)
	grid.place_combatant(mage, Vector2i(3, 4))
	grid.place_combatant(foe, Vector2i(5, 4))
	var ctx: PlanContext = ArenaTestSuite._hand_ctx(grid, [mage], [foe], 5)
	var plan := CombatPlan.new(ctx)
	var planned: Array[String] = []
	for o: PlanOption in plan.target_options(mage, foe):
		if o.step.skill != null:
			planned.append(o.step.skill.id)
	for o: PlanOption in plan.cell_options(mage, Vector2i(4, 5)):
		if o.step.skill != null:
			planned.append(o.step.skill.id)
	checks["planner options leave the hidden skill out"] = not planned.has("firebomb") and (planned.has("arcane_bolt") or planned.has("strike"))
	var step := PlanStep.area_at(firebomb, Vector2i(5, 4))
	checks["a hand-planned hidden skill is rejected"] = plan.preview(mage, step).error_index >= 0
	# Empty loadout = no restriction (hand-built units, old tests).
	var plain := Combatant.new("Plain", 50, 10, 1)
	checks["hand-built unit has no loadout and no restriction"] = plain.active_families.is_empty() and plain.can_use(firebomb)
	checks["clear_loadout lifts the restriction"] = mage.clear_loadout().is_empty() and mage.can_use(firebomb)
	return _report("Active loadout hides non-loadout skills, keeps basics and engine skills", checks)

# ------------------------------------------------------------------------------

static func _test_set_loadout_limits() -> bool:
	var checks: Dictionary = {}
	var mage: Combatant = _mage()
	var before: Array[String] = mage.active_families.duplicate()
	var six: Array[String] = ["a1", "a2", "a3", "a4", "a5", "a6"]
	var err6: String = mage.set_loadout(six, [])
	checks["6 class families refused"] = err6.contains(str(StatRules.MAX_CLASS_SLOTS))
	var four: Array[String] = ["e1", "e2", "e3", "e4"]
	var err4: String = mage.set_loadout([], four)
	checks["4 extra families refused"] = err4.contains(str(StatRules.MAX_EXTRA_SLOTS))
	checks["unowned family refused"] = not mage.set_loadout(["mend"], []).is_empty()
	checks["basic family refused"] = not mage.set_loadout(["strike"], []).is_empty()
	checks["duplicate family refused"] = not mage.set_loadout(["hex", "hex"], []).is_empty()
	checks["errors change nothing"] = mage.active_families == before
	var owned: Array[String] = mage.owned_families()
	checks["exactly 5 owned class families fit"] = owned.size() == StatRules.MAX_CLASS_SLOTS and mage.set_loadout(owned, []).is_empty()
	# Learned skills are owned and can fill extra slots.
	mage.learn_skill("mend")
	checks["a learned skill is owned"] = mage.owned_families().has("mend")
	checks["owned learned family can take an extra slot"] = mage.set_loadout(["hex"], ["mend"]).is_empty() and mage.can_use(SkillCatalog.find("mend")) and not mage.can_use(SkillCatalog.find("firebomb"))
	# learn_skill into an active loadout takes a free extra slot automatically.
	var fresh: Combatant = _mage()
	fresh.learn_skill("rally")
	checks["learning with an active loadout slots the new family"] = fresh.active_extra_families.has("rally") and fresh.can_use(SkillCatalog.find("rally"))
	return _report("set_loadout enforces 5 class / 3 extra and ownership", checks)

# ------------------------------------------------------------------------------

static func _test_locked_in_battle() -> bool:
	var checks: Dictionary = {}
	var session := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	session.start()
	var hero: Combatant = session.allies[0]
	var before: Array[String] = hero.active_families.duplicate()
	var locked: bool = hero.loadout_locked
	var err: String = hero.set_loadout([], [])
	checks["the session locks every loadout on start"] = locked and session.enemies[0].loadout_locked
	checks["set_loadout is an error during battle"] = not err.is_empty() and hero.active_families == before
	checks["clear_loadout is an error during battle"] = not hero.clear_loadout().is_empty() and hero.active_families == before
	session.dispose()
	checks["dispose unlocks"] = not hero.loadout_locked
	checks["loadout can be changed again outside combat"] = hero.set_loadout(hero.active_class_families, hero.active_extra_families).is_empty()
	return _report("Loadout locked during battle", checks)

# ------------------------------------------------------------------------------

static func _test_default_loadout() -> bool:
	var checks: Dictionary = {}
	var warrior: Combatant = ContentFactory.unit({"id": "t_sf_w", "name": "W", "class_id": "warrior", "level": 5})
	var arch: ClassArchetype = ContentFactory.class_of({"class_id": "warrior"})
	var expected: Array[String] = ContentFactory.families_of(arch.skills_at(5))
	checks["class families = path families reached by level, in order"] = warrior.active_class_families == expected.slice(0, StatRules.MAX_CLASS_SLOTS)
	checks["basics are not slotted"] = not warrior.active_families.has("strike") and not warrior.active_families.has("guard")
	checks["every basic family stays out of the slots"] = not warrior.active_families.has("offhand_jab") and not warrior.active_families.has("move") and SkillCatalog.family_of("guard") == "guard"
	var low: Combatant = ContentFactory.unit({"id": "t_sf_wl", "name": "W", "class_id": "warrior", "level": 1})
	checks["levels not yet reached are not active"] = not low.active_families.has("riposte") and low.active_class_families.size() <= expected.size()
	# Extras: families of extra_skills fill the 3 extra slots; further extras are owned but inactive.
	var extras: Array[String] = ["mend", "rally", "taunt_shout", "bleeding_cut"]
	var mage: Combatant = ContentFactory.unit({"id": "t_sf_me", "name": "M", "class_id": "mage", "level": 1, "extra_skills": extras})
	checks["extra families fill the extra slots up to 3"] = mage.active_extra_families == ["mend", "rally", "taunt_shout"]
	checks["the 4th extra is owned but inactive"] = mage.owns_skill("bleeding_cut") and not mage.can_use(SkillCatalog.find("bleeding_cut"))
	checks["no class gets more than 5 class / 3 extra by default"] = true
	for class_id: String in ["warrior", "ranger", "priest", "brawler", "rogue", "mage"]:
		var u: Combatant = ContentFactory.unit({"id": "t_sf_d" + class_id, "name": "U", "class_id": class_id, "level": 10, "extra_skills": extras})
		checks["no class gets more than 5 class / 3 extra by default"] = checks["no class gets more than 5 class / 3 extra by default"] and u.active_class_families.size() <= StatRules.MAX_CLASS_SLOTS and u.active_extra_families.size() <= StatRules.MAX_EXTRA_SLOTS
	var passive_unit: Combatant = ContentFactory.unit({"id": "t_sf_pu", "name": "P", "class_id": "mage", "level": 1})
	Passives.apply(passive_unit, {"id": "t_sf_pass", "name": "x", "effects": [{"type": "skill", "skill": "riposte"}]})
	checks["a passive-granted skill takes a free extra slot"] = passive_unit.can_use(SkillCatalog.find("riposte")) and passive_unit.active_extra_families.has("riposte")
	return _report("Default loadout from class path and extra skills", checks)

# ------------------------------------------------------------------------------

static func _test_unit_override() -> bool:
	var checks: Dictionary = {}
	var base: Dictionary = {"id": "t_sf_ov", "name": "O", "class_id": "mage", "level": 5, "extra_skills": ["mend", "rally", "taunt_shout", "bleeding_cut"]}
	var good: Dictionary = base.duplicate(true)
	good["loadout"] = ["arcane_bolt", "hex", "mend"]
	checks["a valid override validates"] = ContentCatalog.validate_entry("units", good).is_empty()
	var unit: Combatant = ContentFactory.unit(good)
	checks["override: path families are class slots, others extra"] = unit.active_class_families == ["arcane_bolt", "hex"] and unit.active_extra_families == ["mend"]
	checks["override hides the rest"] = not unit.can_use(SkillCatalog.find("firebomb")) and unit.can_use(SkillCatalog.find("hex")) and unit.can_use(SkillCatalog.find("mend"))
	var unowned: Dictionary = base.duplicate(true)
	unowned["loadout"] = ["riposte"]
	checks["an unowned family fails validation"] = not ContentCatalog.validate_entry("units", unowned).is_empty()
	var future: Dictionary = base.duplicate(true)
	future["level"] = 1
	future["loadout"] = ["firebomb"]
	checks["a family not reached by level fails validation"] = not ContentCatalog.validate_entry("units", future).is_empty()
	var basic: Dictionary = base.duplicate(true)
	basic["loadout"] = ["strike"]
	checks["a basic family in the loadout fails validation"] = not ContentCatalog.validate_entry("units", basic).is_empty()
	var twice: Dictionary = base.duplicate(true)
	twice["loadout"] = ["hex", "hex"]
	checks["a duplicate fails validation"] = not ContentCatalog.validate_entry("units", twice).is_empty()
	var unknown: Dictionary = base.duplicate(true)
	unknown["loadout"] = ["no_such_family"]
	checks["an unknown family fails validation"] = not ContentCatalog.validate_entry("units", unknown).is_empty()
	var too_many_extra: Dictionary = base.duplicate(true)
	too_many_extra["loadout"] = ["mend", "rally", "taunt_shout", "bleeding_cut"]
	var extra_errors: Array[String] = ContentCatalog.validate_entry("units", too_many_extra)
	checks["4 extra families fail validation (max 3)"] = not extra_errors.is_empty() and ", ".join(extra_errors).contains(str(StatRules.MAX_EXTRA_SLOTS))
	var many_class: Dictionary = base.duplicate(true)
	many_class["level"] = 10
	many_class["loadout"] = ["arcane_bolt", "volley", "focus", "hex", "firebomb"]
	checks["5 class families are allowed"] = ContentCatalog.validate_entry("units", many_class).is_empty()
	return _report("Unit loadout override validation", checks)

# ------------------------------------------------------------------------------

static func _test_round_trip_and_validation() -> bool:
	var checks: Dictionary = {}
	for id: String in ["strike", "mend", "focus", "move"]:
		var entry: Dictionary = ContentCatalog.get_entry("skills", id)
		var back: Dictionary = ContentFactory.skill_to_entry(ContentFactory.skill(entry))
		checks["%s round-trips" % id] = back == ContentSchema.normalize("skills", entry)
	var strike_def: SkillDef = ContentFactory.skill(ContentCatalog.get_entry("skills", "strike"))
	checks["factory reads family / tier / basic"] = strike_def.family_id() == "strike" and strike_def.tier == 1 and strike_def.basic
	var stored: Dictionary = ContentSchema.to_storage("skills", ContentCatalog.get_entry("skills", "strike"))
	checks["storage keeps tier and basic"] = stored.has("tier") and stored.has("basic") and DictUtil.to_bool(stored["basic"])
	checks["schema defines skill family/tier/basic and unit loadout"] = not ContentSchema.field("skills", "family").is_empty() and not ContentSchema.field("skills", "tier").is_empty() and not ContentSchema.field("skills", "basic").is_empty() and not ContentSchema.field("units", "loadout").is_empty()
	var unit_entry: Dictionary = ContentSchema.normalize("units", {"id": "t_sf_rt", "name": "R", "class_id": "mage", "loadout": ["hex", "", "focus"]})
	checks["unit loadout normalises to a clean string list"] = unit_entry["loadout"] == ["hex", "focus"]
	# Validation (sandbox copy; nothing is saved).
	for kind: String in ContentSchema.KINDS:
		DirAccess.remove_absolute(ContentCatalog.path_for(kind, SANDBOX))
	var forked: bool = ContentCatalog.fork_to(SANDBOX) == OK
	checks["sandbox fork"] = forked
	if forked:
		var tier1: Dictionary = {"id": "t_fam_a", "name": "Fam A", "kind": "ATTACK", "family": "t_fam", "tier": 1}
		checks["a family's first tier is accepted"] = ContentCatalog.upsert("skills", tier1).is_empty()
		checks["a second skill in the same tier is refused"] = not ContentCatalog.upsert("skills", {"id": "t_fam_b", "name": "Fam B", "kind": "ATTACK", "family": "t_fam", "tier": 1}).is_empty()
		checks["tier 2 is accepted"] = ContentCatalog.upsert("skills", {"id": "t_fam_b", "name": "Fam B", "kind": "ATTACK", "family": "t_fam", "tier": 2, "level": 2}).is_empty()
		checks["class skills stop at tier 3"] = not ContentCatalog.upsert("skills", {"id": "t_fam_c", "name": "Fam C", "kind": "ATTACK", "family": "t_fam", "tier": 4}).is_empty()
		checks["a basic skill may reach tier 5"] = ContentCatalog.upsert("skills", {"id": "t_basic5", "name": "B5", "kind": "ATTACK", "family": "t_basic", "tier": 5, "basic": true}).is_empty()
		checks["a family can't mix kinds"] = not ContentCatalog.upsert("skills", {"id": "t_fam_d", "name": "Fam D", "kind": "DEFEND", "family": "t_fam", "tier": 3}).is_empty()
		checks["a family can't mix basic and class tiers"] = not ContentCatalog.upsert("skills", {"id": "t_fam_e", "name": "Fam E", "kind": "ATTACK", "family": "t_fam", "tier": 3, "basic": true}).is_empty()
		checks["a bad family id is refused"] = not ContentCatalog.upsert("skills", {"id": "t_fam_f", "name": "Fam F", "kind": "ATTACK", "family": "Bad Family"}).is_empty()
		checks["new tiers show up in the family"] = _ids(SkillCatalog.family_tiers("t_fam")) == ["t_fam_a", "t_fam_b"]
		var tier2: SkillDef = ContentFactory.skill(ContentCatalog.get_entry("skills", "t_fam_b"))
		checks["factory reads a tier 2 class skill"] = tier2.family == "t_fam" and tier2.tier == 2 and not tier2.basic
		var tier2_stored: Dictionary = ContentSchema.to_storage("skills", ContentCatalog.get_entry("skills", "t_fam_b"))
		checks["storage keeps family, tier and basic"] = tier2_stored.has("family") and tier2_stored.has("tier") and tier2_stored.has("basic") and str(tier2_stored["family"]) == "t_fam"
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	return _report("Catalog / schema round trip and validation", checks)

# Renaming a family root in the Forge must carry its sibling tiers and unit loadouts with it (sandbox copy).
static func _test_family_rename() -> bool:
	var checks: Dictionary = {}
	for kind: String in ContentSchema.KINDS:
		DirAccess.remove_absolute(ContentCatalog.path_for(kind, "user://skill_family_sandbox"))
	checks["sandbox fork"] = ContentCatalog.fork_to("user://skill_family_sandbox") == OK
	if checks["sandbox fork"]:
		# focus has no shipped sibling tier: add a fixture one, then rename the root.
		checks["fixture sibling accepted"] = ContentCatalog.upsert("skills", {"id": "deep_focus", "name": "Deep Focus", "kind": "FOCUS", "level": 3, "family": "focus", "tier": 2}).is_empty()
		var root: Dictionary = ContentCatalog.get_entry("skills", "focus")
		root["id"] = "concentrate"
		var errors: Array[String] = ContentCatalog.upsert("skills", root, "focus")
		checks["rename accepted"] = errors.is_empty()
		var sibling: Dictionary = ContentCatalog.get_entry("skills", "deep_focus")
		checks["sibling tier follows the family"] = str(sibling.get("family", "")) == "concentrate"
		checks["family still groups both tiers"] = SkillCatalog.family_tiers("concentrate").size() == 2
		checks["no orphan family left"] = SkillCatalog.family_tiers("focus").is_empty()
		for kind: String in ContentSchema.KINDS:
			DirAccess.remove_absolute(ContentCatalog.path_for(kind, "user://skill_family_sandbox"))
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	return _report("Renaming a family root keeps sibling tiers and loadouts attached", checks)
