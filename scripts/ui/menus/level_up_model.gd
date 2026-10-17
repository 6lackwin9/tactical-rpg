class_name LevelUpModel
extends RefCounted

# ==============================================================================
# Tactical RPG - Level-up model (UI2-L rule 3): a stat draft over the member's unspent points with a before / after
# preview; confirm() goes through LevelUp.confirm (errors verbatim) and is permanent. Deferring keeps the points.
# ==============================================================================

const NO_POINTS: String = "No points left."

var member_id: String = ""
var draft: Dictionary = {}   # stat -> drafted points (> 0 only)

func _init(p_member: String) -> void:
	member_id = p_member

func _member() -> UnitState:
	return PartyPreview.member(member_id)

func drafted() -> int:
	return StatRules.total_of(draft)

func points_left() -> int:
	var m: UnitState = _member()
	return m.unspent() - drafted() if m != null else 0

func has_draft() -> bool:
	return drafted() > 0

func add(stat: String) -> Array[String]:
	var errors: Array[String] = []
	if not StatRules.STATS.has(stat):
		errors.append("Unknown stat '%s'." % stat)
	elif points_left() <= 0:
		errors.append(NO_POINTS)
	else:
		draft[stat] = DictUtil.to_int(draft.get(stat, 0)) + 1
	return errors

func remove(stat: String) -> Array[String]:
	var errors: Array[String] = []
	if DictUtil.to_int(draft.get(stat, 0)) <= 0:
		errors.append("No drafted points in %s." % stat)
	else:
		draft[stat] = DictUtil.to_int(draft[stat]) - 1
		if DictUtil.to_int(draft[stat]) == 0:
			draft.erase(stat)
	return errors

# Fills the remaining points by the class growth weights (StatRules.auto_spend), on top of the current draft.
func auto() -> void:
	var m: UnitState = _member()
	var arch: ClassArchetype = m.archetype() if m != null else null
	if arch == null or points_left() <= 0:
		return
	var extra: Dictionary = StatRules.auto_spend(arch.growth_per_level, points_left())
	for key: Variant in extra:
		if DictUtil.to_int(extra[key]) > 0:
			draft[str(key)] = DictUtil.to_int(draft.get(str(key), 0)) + DictUtil.to_int(extra[key])

func confirm() -> Array[String]:
	var m: UnitState = _member()
	if m == null:
		var none: Array[String] = ["No party member '%s'." % member_id]
		return none
	var errors: Array[String] = LevelUp.confirm(m, draft.duplicate())
	if errors.is_empty():
		draft = {}
	return errors

func cancel() -> void:
	draft = {}

# The member with the draft applied (a copy; the member is untouched).
func drafted_state() -> UnitState:
	var work: UnitState = _member().duplicate_state()
	var _e: Array[String] = LevelUp.confirm(work, draft.duplicate())
	return work

func before() -> Dictionary:
	return PartyPreview.derived(_member())

func after() -> Dictionary:
	return PartyPreview.derived(drafted_state())

# Rule 4: [{family, name, skill, tier}] castable now but not at `old_level` (two built copies compared).
func new_tiers(old_level: int) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var m: UnitState = _member()
	if m == null:
		return out
	var then_state: UnitState = m.duplicate_state()
	then_state.total_xp = XpCurve.xp_for_level(old_level)
	then_state.spend = {}
	then_state.loadout = {"class": [], "extra": []}
	var now_state: UnitState = m.duplicate_state()
	now_state.loadout = {"class": [], "extra": []}
	var now: Combatant = PartyBuilder.build(now_state)
	var then: Combatant = PartyBuilder.build(then_state)
	if now == null or then == null:
		return out
	for fam: String in now.owned_families():
		for sk: SkillDef in SkillCatalog.family_tiers(fam):
			var was: bool = then.owned_families().has(fam) and then.tier_unlocked(sk)
			if now.tier_unlocked(sk) and not was:
				out.append({"family": fam, "name": sk.display_name, "skill": sk.id, "tier": sk.tier})
	return out
