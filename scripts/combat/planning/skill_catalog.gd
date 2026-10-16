class_name SkillCatalog
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill Catalog (the command tiers available for planning)
# Built from the ContentCatalog "skills" entries (edit them in the Content Forge).
# One family per command kind, tiered by level 1-3 today (4-5 reserved for
# progression). A unit may use every owned tier whose unlock level it has reached
# (Combatant.tier_unlocked); AiProfile.skill_cap is only an explicit AP ceiling.
# Move and Rest are single skills: Move is one block at 1 AP per tile (up to
# Combatant.move_per_block() tiles), Rest is one free block that recovers AP.
# ==============================================================================

static var _all: Array[SkillDef] = []
static var _by_id: Dictionary = {}      # skill id -> SkillDef (built with _all)
static var _tier_one: Dictionary = {}   # family id -> its lowest-tier SkillDef

static func all() -> Array[SkillDef]:
	if _all.is_empty():
		_build()
	return _all

# Skills come from the content catalog (res://data/catalog/skills.json), in file
# order: the first skill of a kind is its "lowest" (Strike, Move...).
static func _build() -> void:
	_by_id.clear()
	_tier_one.clear()
	for entry: Dictionary in ContentCatalog.entries("skills"):
		var s: SkillDef = ContentFactory.skill(entry)
		_all.append(s)
		if not _by_id.has(s.id):
			_by_id[s.id] = s
		if not _tier_one.has(s.family_id()):
			_tier_one[s.family_id()] = s
		else:
			var low: SkillDef = _tier_one[s.family_id()]
			if s.tier < low.tier:
				_tier_one[s.family_id()] = s

# Drops the static cache. Call on shutdown: a static Array holding instances of a
# script keeps that script (and everything it references) alive past exit.
static func clear_cache() -> void:
	_all.clear()
	_by_id.clear()
	_tier_one.clear()

# The lowest-tier row of `family` (null for an unknown family): it carries the family's group, advanced flag and base AP.
static func tier_one(family: String) -> SkillDef:
	all()
	if not _tier_one.has(family):
		return null
	var s: SkillDef = _tier_one[family]
	return s

# Ids of the basic skills a unit owns from level 1 (engine skills are always known anyway), in catalog order.
static func basic_ids() -> Array[String]:
	var out: Array[String] = []
	for s: SkillDef in all():
		if s.basic and not s.system and not out.has(s.id):
			out.append(s.id)
	return out

static func is_system(skill_id: String) -> bool:
	var s: SkillDef = find(skill_id)
	return s != null and s.system

static func is_learned_only(skill_id: String) -> bool:
	var s: SkillDef = find(skill_id)
	return s != null and s.learned_only

static func find(skill_id: String) -> SkillDef:
	all()
	if not _by_id.has(skill_id):
		return null
	var s: SkillDef = _by_id[skill_id]
	return s

# Skills of `kind` the unit has unlocked, lowest level first.
static func available(kind: SkillDef.Kind, skill_cap: int, c: Combatant = null) -> Array[SkillDef]:
	var result: Array[SkillDef] = []
	for s: SkillDef in all():
		if s.kind == kind and s.level <= skill_cap and (c == null or c.can_use(s)):
			result.append(s)
	return result

# Single-target attacks for c (no area skills; the off-hand jab only for ranged units).
static func single_attacks(c: Combatant, skill_cap: int) -> Array[SkillDef]:
	var result: Array[SkillDef] = []
	for s: SkillDef in available(SkillDef.Kind.ATTACK, skill_cap, c):
		if s.is_area() or (s.offhand and c.attack_range <= 1):
			continue
		result.append(s)
	return result

static func area_attacks(skill_cap: int, c: Combatant = null) -> Array[SkillDef]:
	var result: Array[SkillDef] = []
	for s: SkillDef in available(SkillDef.Kind.ATTACK, skill_cap, c):
		if s.is_area():
			result.append(s)
	return result

# SUPPORT skills that remove debuffs and do not heal (cleanse-only), for c.
static func cleanse_only(skill_cap: int, c: Combatant = null) -> Array[SkillDef]:
	var result: Array[SkillDef] = []
	for s: SkillDef in available(SkillDef.Kind.SUPPORT, skill_cap, c):
		if s.cleanses() and not s.heals():
			result.append(s)
	return result

# SUPPORT / DEFEND skills of `kind` that grant `status` and do not heal.
static func granting(kind: SkillDef.Kind, status: String, skill_cap: int, c: Combatant = null) -> Array[SkillDef]:
	var result: Array[SkillDef] = []
	for s: SkillDef in available(kind, skill_cap, c):
		if s.grant_status == status and not s.heals():
			result.append(s)
	return result

# SUPPORT skills granting a buff other than `except_status`, with no heal (haste-like ally buffs).
static func ally_buffs(except_status: String, skill_cap: int, c: Combatant = null) -> Array[SkillDef]:
	var result: Array[SkillDef] = []
	for s: SkillDef in available(SkillDef.Kind.SUPPORT, skill_cap, c):
		if not s.grant_status.is_empty() and s.grant_status != except_status and not s.heals():
			result.append(s)
	return result

# Zero-power ATTACK skills that inflict a status (debuff-only attacks such as Intimidate), for c.
static func debuff_attacks(skill_cap: int, c: Combatant = null) -> Array[SkillDef]:
	var result: Array[SkillDef] = []
	for s: SkillDef in available(SkillDef.Kind.ATTACK, skill_cap, c):
		if not s.deals_damage() and not s.is_area() and not s.status_effect.is_empty():
			result.append(s)
	return result

static func lowest(kind: SkillDef.Kind) -> SkillDef:
	for s: SkillDef in all():
		if s.kind == kind:
			return s
	return null

# ------------------------------------------------------------------------------
# Skill families (tiers of one skill, chosen at planning time)
# ------------------------------------------------------------------------------

# The family of a skill (its own id when it stands alone or is unknown).
static func family_of(skill_id: String) -> String:
	var s: SkillDef = find(skill_id)
	return skill_id if s == null else s.family_id()

# Every tier of `family`, lowest tier first (ties keep file order).
static func family_tiers(family: String) -> Array[SkillDef]:
	var result: Array[SkillDef] = []
	for s: SkillDef in all():
		if s.family_id() == family:
			result.append(s)
	result.sort_custom(func(a: SkillDef, b: SkillDef) -> bool: return a.tier < b.tier)
	return result

# Every family id, in file order of its first skill.
static func families() -> Array[String]:
	var result: Array[String] = []
	for s: SkillDef in all():
		if not result.has(s.family_id()):
			result.append(s.family_id())
	return result

# Display name of a family: its lowest tier ("Mend" for Mend I / II / III).
static func family_name(family: String) -> String:
	var tiers: Array[SkillDef] = family_tiers(family)
	return family if tiers.is_empty() else tiers[0].display_name

# A basic family (any tier flagged basic) is available to every class and costs no loadout slot.
static func is_basic_family(family: String) -> bool:
	for s: SkillDef in family_tiers(family):
		if s.basic:
			return true
	return false

# Basic or engine skill: always passes the active-loadout filter.
static func is_always_active(skill_id: String) -> bool:
	var s: SkillDef = find(skill_id)
	return s != null and (s.system or is_basic_family(s.family_id()))

# `kind` skills the unit can plan, grouped by family with the affordable tiers (tier <= cap by AP level)
# lowest first: [{"family": id, "name": text, "tiers": Array[SkillDef]}], families in file order.
static func grouped(kind: SkillDef.Kind, skill_cap: int, c: Combatant = null) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for s: SkillDef in available(kind, skill_cap, c):
		var fam: String = s.family_id()
		var group: Dictionary = {}
		for g: Dictionary in out:
			if str(g["family"]) == fam:
				group = g
		if group.is_empty():
			var tiers: Array[SkillDef] = []
			group = {"family": fam, "name": family_name(fam), "tiers": tiers}
			out.append(group)
		var list: Array[SkillDef] = group["tiers"]
		list.append(s)
	for g: Dictionary in out:
		var list: Array[SkillDef] = g["tiers"]
		list.sort_custom(func(a: SkillDef, b: SkillDef) -> bool: return a.tier < b.tier)
	return out

static func roman(n: int) -> String:
	return ["", "I", "II", "III", "IV", "V"][clampi(n, 0, 5)]

# "Mend I / II / III" for a group from grouped() (single tier: just the name).
static func group_label(group: Dictionary) -> String:
	var tiers: Array[SkillDef] = group["tiers"]
	if tiers.size() <= 1:
		return str(group["name"]) if tiers.is_empty() else tiers[0].display_name
	var parts: Array[String] = []
	for s: SkillDef in tiers:
		parts.append(roman(s.tier))
	return "%s %s" % [str(group["name"]), " / ".join(parts)]
