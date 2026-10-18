---
class: SkillUnlockTestSuite
extends: RefCounted
description: "Tactical RPG - Skill Unlock Test Suite (SK2) Skills open by unit LEVEL, not by AP: a class path step says when a family is owned, its tiers open at fixed steps after it, advanced families need a higher path level, and passives / KNOW lower a family's unlock by a capped amount. Fixtures live in a sandbox copy of the catalog (a fixture class with its own families); expectations are derived from StatRules, never hard-coded."
---

# Properties
* `CLS`: String = "t_sk_cls" (const)
* `LVL_A`: int = 3     # ordinary, 3 tiers (AP 1 / 2 / 3) (const)
* `LVL_C`: int = 3     # ordinary, tier 1 costs AP 3 (const)
* `LVL_H`: int = 4     # ordinary heal-group family (const)
* `LVL_B`: int = 5     # advanced, 3 tiers (const)
* `LVL_O`: int = 6     # ordinary, tier 2 carries an explicit offset 0 (const)
* `LVL_D`: int = 7     # advanced (const)
* `TAG`: String = "SkillUnlock" (const)
* `e`: Dictionary = _class_entry(path)

# Methods

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING SKILL UNLOCK TEST (SK2)                 ===")
print("=======================================================")
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
var ok: bool = true
ok = _test_constants() and ok
ok = _test_fixture_rules() and ok
ok = _test_shipped_data() and ok
ok = _test_upsert_many() and ok
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
return ok
```

### `static _family(id: String, aps: Array[int], extra: Dictionary = {}) -> bool`
```logic
var ok: bool = true
for t: int in range(1, aps.size() + 1):
	var row: Dictionary = {"id": id if t == 1 else "%s_%d" % [id, t], "name": "%s %d" % [id, t], "kind": "ATTACK", "level": aps[t - 1], "blocks": 2, "tier": t, "family": id}
	row.merge(extra, true)
	ok = ContentCatalog.upsert("skills", row).is_empty() and ok
return ok
```

### `static _class_entry(path: Array[Dictionary]) -> Dictionary`
```logic
var e: Dictionary = ContentCatalog.get_entry("classes", "warrior").duplicate(true)
e["id"] = CLS
e["name"] = "Fixture"
e["skill_path"] = _path_dict(path)
return e
```

### `static _path_dict(path: Array[Dictionary]) -> Dictionary`
```logic
var out: Dictionary = {}
for step: Dictionary in path:
	out[str(step["skill"])] = DictUtil.to_int(step["level"])
return out
```

### `static _step(level: int, skill: String) -> Dictionary`
```logic
return {"level": level, "skill": skill}
```

### `static _build_fixture() -> bool`
```logic
if not TestHelpers.fork(SANDBOX):
	return false
var ok: bool = true
ok = _family("t_sk_a", [1, 2, 3]) and ok
ok = _family("t_sk_c", [3, 4, 5]) and ok
ok = _family("t_sk_b", [3, 4, 5], {"grade": "advanced"}) and ok
ok = _family("t_sk_d", [3, 4, 5], {"grade": "advanced"}) and ok
ok = _family("t_sk_o", [2, 3]) and ok
ok = ContentCatalog.upsert("skills", {"id": "t_sk_o_2", "name": "O2", "kind": "ATTACK", "level": 3, "blocks": 2, "tier": 2, "family": "t_sk_o", "unlock_offset": 0}).is_empty() and ok
ok = ContentCatalog.upsert("skills", {"id": "t_sk_h", "name": "Heal", "kind": "SUPPORT", "level": 2, "blocks": 2, "heal_base": 5, "tier": 1}).is_empty() and ok
ok = ContentCatalog.upsert("classes", _class_entry(_fixture_path())).is_empty() and ok
return ok
```

### `static _unit(level: int) -> Combatant`
```logic
return ContentFactory.unit({"id": "t_sk_u", "name": "U", "class_id": CLS, "level": level}, "", 12)
```

### `static _can(c: Combatant, id: String) -> bool`
```logic
var s: SkillDef = SkillCatalog.find(id)
return s != null and c.owns_skill(id) and c.tier_unlocked(s)
```

### `static _set_know(c: Combatant, target: int) -> void`
```logic
c.stats.remove_modifier("t_know")
c.stats.add_modifier(StatModifier.new("KNOW", target - c.unlock_know(), 0.0, "t_know", "always", -1))
c.refresh_path_skills()
```

### `static _unlock_passive(id: String, family: String, levels: int) -> Dictionary`
```logic
return {"id": id, "name": id, "effects": [{"type": "unlock", "family": family, "levels": levels}]}
```

### `static _expect(base: int, reduction: int, advanced: bool) -> int`
```logic
var floor_level: int = StatRules.ADVANCED_UNLOCK_FLOOR if advanced else 1
return mini(base, maxi(floor_level, base - mini(reduction, StatRules.MAX_UNLOCK_REDUCTION)))
```

### `static _test_constants() -> bool`
```logic
var checks: Dictionary = {}
checks["default unlock is the old rule inverted"] = true
for ap: int in range(1, 6):
	# A unit at default_unlock(ap) was exactly the first level with ceil(level / 2) >= ap.
	var lvl: int = StatRules.default_unlock(ap)
	checks["default unlock is the old rule inverted"] = checks["default unlock is the old rule inverted"] and ceili(float(lvl) / 2.0) >= ap and (lvl == 1 or ceili(float(lvl - 1) / 2.0) < ap)
checks["KNOW below the base step gives nothing"] = StatRules.know_unlock_reduction(StatRules.KNOW_UNLOCK_BASE + StatRules.KNOW_UNLOCK_STEP - 1) == 0
checks["each KNOW step lowers one level"] = StatRules.know_unlock_reduction(StatRules.KNOW_UNLOCK_BASE + StatRules.KNOW_UNLOCK_STEP) == 1 and StatRules.know_unlock_reduction(StatRules.KNOW_UNLOCK_BASE + 2 * StatRules.KNOW_UNLOCK_STEP) == mini(2, StatRules.MAX_UNLOCK_REDUCTION)
checks["KNOW reduction is capped"] = StatRules.know_unlock_reduction(999) == StatRules.MAX_UNLOCK_REDUCTION and StatRules.know_unlock_reduction(-5) == 0
checks["tier offsets step by TIER_UNLOCK_STEP"] = true
for t: int in [1, 2, 3]:
	var s: SkillDef = SkillDef.new()
	s.tier = t
	checks["tier offsets step by TIER_UNLOCK_STEP"] = checks["tier offsets step by TIER_UNLOCK_STEP"] and s.tier_offset() == StatRules.TIER_UNLOCK_STEP * (t - 1)
	s.unlock_offset = 0
	checks["tier offsets step by TIER_UNLOCK_STEP"] = checks["tier offsets step by TIER_UNLOCK_STEP"] and s.tier_offset() == 0
checks["schema carries grade and unlock_offset with defaults"] = str(ContentSchema.field("skills", "grade").get("default", "")) == "novice" and DictUtil.to_int(ContentSchema.field("skills", "unlock_offset")["default"]) == -1
checks["unlock passive describes itself"] = Passives.describe({"effects": [{"type": "unlock", "family": "riposte", "levels": 2}]}).contains("riposte")
return TestHelpers.report(TAG, "Unlock constants and helpers", checks)
```

### `static _test_fixture_rules() -> bool`
```logic
var checks: Dictionary = {}
checks["fixture catalog built"] = _build_fixture()
if not checks["fixture catalog built"]:
	return TestHelpers.report(TAG, "Unlock rules on a fixture class", checks)
var step: int = StatRules.TIER_UNLOCK_STEP

# --- Validation -------------------------------------------------------------
checks["advanced family at ADVANCED_MIN_LEVEL is accepted"] = _probe_errors(_fixture_path()).is_empty()
var low_path: Array[Dictionary] = _fixture_path()
low_path[5] = _step(StatRules.ADVANCED_MIN_LEVEL - 1, "t_sk_b")
var low_errors: Array[String] = ContentCatalog.validate_entry("classes", _class_entry(low_path))
checks["advanced family below ADVANCED_MIN_LEVEL is refused"] = low_errors.any(func(m: String) -> bool: return m.contains("advanced"))
var dup_path: Array[Dictionary] = _fixture_path()
dup_path.append(_step(LVL_D, "t_sk_a"))
checks["a family is listed once by shape (CLASS1: a dict key)"] = _path_dict(dup_path).size() == _fixture_path().size()
var tier_path: Array[Dictionary] = _fixture_path()
tier_path[2] = _step(LVL_A, "t_sk_a_2")
checks["a tier row on a path is refused"] = ContentCatalog.validate_entry("classes", _class_entry(tier_path)).any(func(m: String) -> bool: return m.contains("higher tier"))
var mixed: Dictionary = ContentCatalog.get_entry("skills", "t_sk_b_2").duplicate(true)
mixed["grade"] = "novice"
checks["mixed grades in a family are refused"] = not ContentCatalog.validate_entry("skills", mixed).is_empty()
checks["tier 1 with an offset is refused"] = not ContentCatalog.validate_entry("skills", {"id": "t_sk_x", "name": "X", "kind": "ATTACK", "unlock_offset": 2}).is_empty()
checks["a skill AP level does not gate a path step"] = _probe_errors([_step(1, "t_sk_a"), _step(1, "t_sk_c")]).is_empty()

# --- Tier steps (derived offsets) and ownership --------------------------------
var before: Combatant = _unit(LVL_A - 1)
checks["a family is not owned before its path level"] = not before.owns_skill("t_sk_a") and not before.known_skills.has("t_sk_a")
var at_open: Combatant = _unit(LVL_A)
checks["owned and tier 1 castable at the path level"] = _can(at_open, "t_sk_a")
var tiers_ok: bool = true
for lvl: int in [LVL_A, LVL_A + 1, LVL_A + step, LVL_A + step * 2 - 1, mini(StatRules.MAX_LEVEL, LVL_A + 2 * step)]:
	var u: Combatant = _unit(lvl)
	tiers_ok = tiers_ok and _can(u, "t_sk_a") and _can(u, "t_sk_a_2") == (lvl >= LVL_A + step) and _can(u, "t_sk_a_3") == (lvl >= mini(StatRules.MAX_LEVEL, LVL_A + 2 * step))
checks["tier 2 opens one step, tier 3 two steps after the family"] = tiers_ok
checks["tiers open one by one: 1 level after owning gives only tier 1"] = not _can(_unit(LVL_A + 1), "t_sk_a_2")
checks["tier 3 is clamped to MAX_LEVEL"] = _unit(StatRules.MAX_LEVEL).skill_unlock_level(SkillCatalog.find("t_sk_d_3")) == StatRules.MAX_LEVEL
var ap_free: Combatant = _unit(LVL_C)
var t1_c: SkillDef = SkillCatalog.find("t_sk_c")
checks["a tier 1 with a high AP cost is castable at the path level"] = t1_c.level > ceili(float(LVL_C) / 2.0) and _can(ap_free, "t_sk_c")
var offset_unit: Combatant = _unit(LVL_O)
checks["an explicit offset 0 puts tier 2 beside tier 1"] = _can(offset_unit, "t_sk_o") and _can(offset_unit, "t_sk_o_2")
checks["system and basic skills are never gated"] = _unit(1).tier_unlocked(SkillCatalog.find("strike")) and _unit(1).tier_unlocked(SkillCatalog.find("move"))
checks["hand-built units are never gated"] = Combatant.new("H", 10, 5, 1).tier_unlocked(SkillCatalog.find("t_sk_b_3"))

# --- Passives -----------------------------------------------------------------
var p_ord: Combatant = _unit(1)
Passives.apply(p_ord, _unlock_passive("t_p_a2", "t_sk_a", 2))
checks["passive on an ordinary family: E = U - levels (floor 1)"] = p_ord.family_unlock_level("t_sk_a") == _expect(LVL_A, 2, false) and p_ord.owns_skill("t_sk_a") == (1 >= _expect(LVL_A, 2, false))
var p_adv: Combatant = _unit(StatRules.ADVANCED_UNLOCK_FLOOR - 1)
Passives.apply(p_adv, _unlock_passive("t_p_b2", "t_sk_b", 2))
checks["passive on an advanced family never goes below the floor"] = p_adv.family_unlock_level("t_sk_b") == _expect(LVL_B, 2, true) and not p_adv.owns_skill("t_sk_b")
var p_adv_open: Combatant = _unit(StatRules.ADVANCED_UNLOCK_FLOOR)
Passives.apply(p_adv_open, _unlock_passive("t_p_b2", "t_sk_b", 2))
checks["...and opens at the floor level"] = p_adv_open.known_skills.has("t_sk_b")
var p_one: Combatant = _unit(1)
Passives.apply(p_one, _unlock_passive("t_p_h1", "t_sk_h", 1))
checks["one passive level lowers by one"] = p_one.family_unlock_level("t_sk_h") == LVL_H - 1
var grp: Combatant = _unit(1)
Passives.apply(grp, {"id": "t_p_grp", "name": "G", "effects": [{"type": "unlock", "group": "heal", "levels": 1}]})
checks["a group passive lowers every family of the group"] = grp.family_unlock_level("t_sk_h") == LVL_H - 1 and grp.family_unlock_level("t_sk_a") == LVL_A
var capped: Combatant = _unit(1)
Passives.apply(capped, _unlock_passive("t_p_c1", "t_sk_c", 2))
Passives.apply(capped, _unlock_passive("t_p_c2", "t_sk_c", 2))
Passives.apply(capped, {"id": "t_p_c3", "name": "C3", "effects": [{"type": "unlock", "family": "t_sk_c", "levels": 1}]})
checks["stacking passives never beat the cap"] = capped.unlock_reduction("t_sk_c") == mini(2, StatRules.MAX_UNLOCK_REDUCTION) and capped.family_unlock_level("t_sk_c") == _expect(LVL_C, 99, false)
var off_path: Combatant = _unit(1)
var known_before: Array[String] = off_path.known_skills.duplicate()
Passives.apply(off_path, _unlock_passive("t_p_off", "riposte", 2))
checks["a passive on a family outside the path grants nothing"] = off_path.known_skills == known_before and not off_path.owns_skill("riposte")
checks["off-path families use default_unlock"] = off_path.family_unlock_level("riposte") == StatRules.default_unlock(SkillCatalog.tier_one("riposte").level)
var bad_group: Combatant = _unit(1)
Passives.apply(bad_group, {"id": "t_p_bad", "name": "B", "effects": [{"type": "unlock", "group": "spell", "levels": 2}]})
checks["a group the family is not in changes nothing"] = bad_group.family_unlock_level("t_sk_a") == LVL_A

# --- KNOW -----------------------------------------------------------------------
var know_lo: Combatant = _unit(LVL_B - 1)
_set_know(know_lo, StatRules.KNOW_UNLOCK_BASE + StatRules.KNOW_UNLOCK_STEP - 1)
checks["KNOW just under the first step gains nothing"] = know_lo.family_unlock_level("t_sk_b") == LVL_B
_set_know(know_lo, StatRules.KNOW_UNLOCK_BASE + StatRules.KNOW_UNLOCK_STEP)
checks["KNOW at the first step opens an advanced family one level early"] = know_lo.family_unlock_level("t_sk_b") == _expect(LVL_B, 1, true) and know_lo.known_skills.has("t_sk_b")
checks["...and does not open it two levels early"] = not _unit(LVL_B - 2).owns_skill("t_sk_b")
_set_know(know_lo, StatRules.KNOW_UNLOCK_BASE + 2 * StatRules.KNOW_UNLOCK_STEP)
checks["KNOW at the second step lowers two levels (floored)"] = know_lo.family_unlock_level("t_sk_b") == _expect(LVL_B, 2, true)
_set_know(know_lo, 999)
checks["huge KNOW stays capped"] = know_lo.family_unlock_level("t_sk_b") == _expect(LVL_B, 99, true) and know_lo.family_unlock_level("t_sk_d") == _expect(LVL_D, 99, true)
checks["KNOW never lowers an ordinary family"] = know_lo.family_unlock_level("t_sk_a") == LVL_A and know_lo.family_unlock_level("t_sk_c") == LVL_C
var know_floor: Combatant = _unit(StatRules.ADVANCED_UNLOCK_FLOOR - 1)
_set_know(know_floor, 999)
checks["KNOW cannot open an advanced family below the floor"] = not know_floor.owns_skill("t_sk_b") and not know_floor.owns_skill("t_sk_d")
var combined: Combatant = _unit(1)
_set_know(combined, StatRules.KNOW_UNLOCK_BASE + StatRules.KNOW_UNLOCK_STEP)
Passives.apply(combined, _unlock_passive("t_p_d2", "t_sk_d", 2))
checks["passive and KNOW combine as the larger, not the sum"] = combined.unlock_reduction("t_sk_d") == 2
var combined_small: Combatant = _unit(1)
_set_know(combined_small, StatRules.KNOW_UNLOCK_BASE + StatRules.KNOW_UNLOCK_STEP)
Passives.apply(combined_small, _unlock_passive("t_p_d1", "t_sk_d", 1))
checks["equal sources are not summed"] = combined_small.unlock_reduction("t_sk_d") == 1
var buffed: Combatant = _unit(LVL_B - 1)
buffed.stats.add_modifier(StatModifier.new("KNOW", 99, 0.0, "t_buff", "always", 2))
buffed.refresh_path_skills()
checks["temporary combat buffs never count as KNOW"] = buffed.family_unlock_level("t_sk_b") == LVL_B

# --- Freeze -----------------------------------------------------------------------
var frozen: Combatant = _unit(LVL_B - 1)
frozen.loadout_locked = true
var owned_before: Array[String] = frozen.known_skills.duplicate()
var e_before: int = frozen.family_unlock_level("t_sk_b")
_set_know(frozen, 999)
Passives.apply(frozen, _unlock_passive("t_p_fz", "t_sk_b", 2))
checks["a locked unit keeps its ownership and unlock levels"] = frozen.known_skills == owned_before and frozen.family_unlock_level("t_sk_b") == e_before
frozen.loadout_locked = false
frozen.refresh_path_skills()
checks["unlocking lets the changes through"] = frozen.family_unlock_level("t_sk_b") == _expect(LVL_B, 2, true)

# --- Default loadout ------------------------------------------------------------------
var loadout_ok: bool = true
for lvl: int in range(1, StatRules.MAX_LEVEL + 1):
	var u: Combatant = _unit(lvl)
	var arch: ClassArchetype = ContentFactory.class_of({"class_id": CLS})
	var owned_path: Array[String] = []
	for st: Dictionary in arch.path_steps():
		if lvl >= u.family_unlock_level(SkillCatalog.family_of(str(st["skill"]))):
			owned_path.append(str(st["skill"]))
	var expected: Array[String] = ContentFactory.families_of(owned_path).slice(0, StatRules.MAX_CLASS_SLOTS)
	loadout_ok = loadout_ok and u.active_class_families == expected
checks["default loadout = first MAX_CLASS_SLOTS owned path families in path order, every level"] = loadout_ok
var early: Combatant = _unit(LVL_B - 1)
_set_know(early, 999)
var plain: Combatant = _unit(LVL_B - 1)
checks["an earlier-owned advanced family does not displace the default 5"] = ContentFactory.loadout_of({}, ContentFactory.class_of({"class_id": CLS}), LVL_B - 1, plain.known_skills)["class"] == plain.active_class_families

# --- Planner ------------------------------------------------------------------------------
var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
var hero: Combatant = _unit(LVL_C)
var foe := Combatant.new("Foe", 200, 1, 1)
grid.place_combatant(hero, Vector2i(3, 4))
grid.place_combatant(foe, Vector2i(4, 4))
var ctx: PlanContext = TestHelpers.hand_ctx(grid, [hero], [foe], AiProfile.NO_CEILING)
var plan := CombatPlan.new(ctx)
hero.set_stance(CombatTypes.StanceType.OFFENSIVE)
var high_ap: Array[PlanStep] = [PlanStep.engage(t1_c, foe)]
plan.set_steps(hero, high_ap)
var st_ok: CombatPlan.SimState = plan.compile(hero)
checks["planner: an unlocked high-AP tier 1 is planned, never 'unlocks at'"] = st_ok.error_index < 0 and not st_ok.error_text.contains("unlock")
hero.ap = t1_c.ap_cost(hero) - 1
var st_dry: CombatPlan.SimState = plan.compile(hero)
checks["planner: too little AP is an AP problem, not an unlock problem"] = (st_dry.runs_dry() or st_dry.error_index >= 0) and not st_dry.error_text.contains("unlock")
hero.ap = hero.max_ap
var locked_step: Array[PlanStep] = [PlanStep.engage(SkillCatalog.find("t_sk_a_3"), foe)]
var plan2 := CombatPlan.new(ctx)
plan2.set_steps(hero, locked_step)
var st_locked: CombatPlan.SimState = plan2.compile(hero)
checks["planner: a tier below its unlock level names the level"] = st_locked.error_index >= 0 and st_locked.error_text.contains("unlocks at unit level")

TestHelpers.cleanup(SANDBOX)
return TestHelpers.report(TAG, "Unlock rules on a fixture class (validation, tiers, passives, KNOW, freeze, loadout, planner)", checks)
```

### `static _test_shipped_data() -> bool`
```logic
var checks: Dictionary = {}
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
checks["shipped catalog validates"] = ContentCatalog.validate_all().is_empty()
var open_ok: bool = true
var order_ok: bool = true
var loadout_ok: bool = true
for class_id: String in ["warrior", "ranger", "priest", "brawler", "rogue", "mage"]:
	var arch: ClassArchetype = ClassArchetype.create(class_id)
	for step: Dictionary in arch.path_steps():
		var lvl: int = DictUtil.to_int(step["level"])
		var unit: Combatant = ContentFactory.unit({"id": "t_su_%s" % class_id, "name": "T", "class_id": class_id, "level": lvl})
		var t1: SkillDef = SkillCatalog.find(str(step["skill"]))
		open_ok = open_ok and t1 != null and unit.owns_skill(t1.id) and unit.tier_unlocked(t1)
	for lvl: int in range(1, StatRules.MAX_LEVEL + 1):
		var u: Combatant = ContentFactory.unit({"id": "t_su_%s" % class_id, "name": "T", "class_id": class_id, "level": lvl})
		var owned_path: Array[String] = []
		for step: Dictionary in arch.path_steps():
			if u.known_skills.has(str(step["skill"])):
				owned_path.append(str(step["skill"]))
		loadout_ok = loadout_ok and u.active_class_families == ContentFactory.families_of(owned_path).slice(0, StatRules.MAX_CLASS_SLOTS)
		order_ok = order_ok and owned_path.size() >= arch.skills_at(lvl).size()
checks["every shipped path step is castable (tier 1, any AP) at its own level"] = open_ok
checks["ownership is never later than the unreduced path"] = order_ok
checks["shipped default loadouts are the first owned path families"] = loadout_ok
var low: Combatant = ContentFactory.unit({"id": "t_su_w", "name": "W", "class_id": "warrior", "level": 1})
var p: AiProfile = ContentFactory.ai_profile({"class_id": "warrior", "level": 1}, low)
checks["a derived unit with skill_cap 0 has no AP ceiling"] = p.skill_cap == AiProfile.NO_CEILING
var explicit: AiProfile = ContentFactory.ai_profile({"class_id": "warrior", "level": 1, "skill_cap": 2}, low)
checks["an explicit skill_cap stays an AP ceiling"] = explicit.skill_cap == 2 and SkillCatalog.available(SkillDef.Kind.DEFEND, explicit.skill_cap, null).all(func(s: SkillDef) -> bool: return s.level <= 2)
checks["hand-built profiles keep their default cap"] = AiProfile.create(AiProfile.Role.AGGRESSOR).skill_cap == 1
var tier_ok: bool = true
for class_id: String in ["warrior", "priest", "mage"]:
	var arch2: ClassArchetype = ClassArchetype.create(class_id)
	for step: Dictionary in arch2.path_steps():
		var fam: String = SkillCatalog.family_of(str(step["skill"]))
		var rows: Array[SkillDef] = SkillCatalog.family_tiers(fam)
		if rows.size() < 2 or rows[0].basic:
			continue
		var base: int = DictUtil.to_int(step["level"])
		var u2: Combatant = ContentFactory.unit({"id": "t_su_t", "name": "T", "class_id": class_id, "level": base})
		var e: int = u2.family_unlock_level(fam)
		for r: SkillDef in rows:
			var want: bool = base >= mini(StatRules.MAX_LEVEL, e + r.tier_offset())
			tier_ok = tier_ok and u2.tier_unlocked(r) == want
checks["shipped multi-tier families: a tier is unlocked iff the unit level reaches unlock + offset"] = tier_ok
return TestHelpers.report(TAG, "Shipped data under the new rule", checks)
```

### `static _test_upsert_many() -> bool`
```logic
var checks: Dictionary = {}
var sandbox: String = TestHelpers.sandbox("sk2_upsert_many_test")
checks["sandbox fork"] = TestHelpers.fork(sandbox)
if checks["sandbox fork"]:
	# A test-only 3-rank user family (no shipped id: shipped grades and requires may change with content).
	var fam: String = "t_um"
	var mods: Array[Dictionary] = [{"level": 3}, {"level": 4}]
	checks["fixture family stored"] = ContentCatalog.upsert_skill(SkillRanks.entry(TestHelpers.attack(fam), mods)).is_empty()
	var before: Array[Dictionary] = ContentCatalog.entries("skills")
	var head: Dictionary = ContentCatalog.get_entry("skills", fam)
	var r2: Dictionary = ContentCatalog.get_entry("skills", SkillRanks.rank_id(fam, 2))
	var r3: Dictionary = ContentCatalog.get_entry("skills", SkillRanks.rank_id(fam, 3))
	var one_alone: Array[String] = ContentCatalog.upsert("skills", head.merged({"grade": "advanced"}, true))
	checks["a single rank alone is refused (grade must match)"] = not one_alone.is_empty() and _grade(fam) != "advanced"
	var bad: Array[String] = ContentCatalog.upsert_many("skills", [head.merged({"grade": "advanced"}, true), r2.merged({"grade": "advanced"}, true)])
	checks["a partial family is refused as a set"] = not bad.is_empty()
	checks["a refused set changes nothing"] = ContentCatalog.entries("skills").size() == before.size() and _grade(SkillRanks.rank_id(fam, 2)) != "advanced" and _grade(fam) != "advanced"
	var good: Array[String] = ContentCatalog.upsert_many("skills", [head.merged({"grade": "advanced"}, true), r2.merged({"grade": "advanced"}, true), r3.merged({"grade": "advanced"}, true)])
	checks["the whole family changes together"] = good.is_empty() and _grade(fam) == "advanced" and _grade(SkillRanks.rank_id(fam, 3)) == "advanced"
TestHelpers.cleanup(sandbox)
return TestHelpers.report(TAG, "upsert_many applies a rank family atomically (all or none)", checks)
```

### `static _grade(id: String) -> String`
```logic
return str(ContentCatalog.get_entry("skills", id).get("grade", "novice"))
```

