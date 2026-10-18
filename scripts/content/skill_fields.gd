class_name SkillFields
extends RefCounted

# The skills field table of ContentSchema (moved here by CLN-9b; ContentSchema.fields("skills") / field() are the API
# and cache the result). The kind lists (`when`), Forge sections and the rank flag are generated from SkillKinds.

static func _f(key: String, label: String, type: String, default: Variant, extra: Dictionary = {}) -> Dictionary:
	return ContentSchema._f(key, label, type, default, extra)

static func build() -> Array[Dictionary]:
	var list: Array[Dictionary] = [
		_f("id", "ID", "id", "", {"key_field": true, "help": "Unique snake_case id used by units and tests. Ranks are <id>, <id>_2, <id>_3."}),
		_f("name", "Name", "string", "New Skill", {"max_len": 32, "help": "Rank names are derived: Mend, Mend II, Mend III (all must fit 32 characters)."}),
		_f("kind", "Kind", "enum", "ATTACK", {"options": SkillKinds.names(), "key_field": true, "help": "MOVE / TARGET / STATUS / REST are engine skills; author ATTACK, DEFEND, FOCUS, SUPPORT (heal / cleanse / buff a friendly unit or yourself) or DISENGAGE (jump away from an enemy)."}),
		_f("level", "Level (= AP cost)", "int", 1, {"min": 1, "max": 5}),
		_f("blocks", "Blocks", "int", 1, {"min": 1, "max": 6, "help": "Timeline length; the effect lands at the end of the last block."}),
		_f("description", "Description", "template", "", {"max_len": 240, "help": "A template: {name} {ap} {blocks} {power} {dash} {reach} {weapons} {proficiency} {status} {potency} {magnitude} {rounds} {heal} {cleanse} {mitigation} {boost} {jump} {recover} {unlock} {flow} {exploit} {disrupt} {extra}. {{ and }} are literal braces. Each rank renders it with its own numbers."}),
		_f("template", "Description template (internal)", "string", "", {"internal": true, "help": "Flat rows carry the template their description was rendered from."}),
		_f("system", "System skill", "bool", false, {"readonly": true, "key_field": true, "help": "Engine skills the planner inserts itself (Move, Rest, Switch Target, status blocks)."}),
		_f("basic", "Basic skill", "bool", false, {"key_field": true, "help": "Owned by every unit from level 1; not counted against the active loadout slots; exactly one rank; not valid in a class path."}),
		_f("learned_only", "Learn only", "bool", false, {"help": "Unrestricted hand-built units (tests, sandbox) don't know it unless they learn it; class-derived units get it from skill paths / extra skills as usual."}),
		_f("grade", "Grade", "enum", "novice", {"options": ["novice", "advanced", "expert"], "key_field": true, "help": "novice, advanced or expert (reserved: no expert skill ships). Advanced and expert class-path steps need unit level 5+ and KNOW can open them earlier (never below level 3)."}),
		_f("requires", "Requires", "refs", [], {"ref": "skills", "ref_filter": {"system": false, "basic": false}, "key_field": true, "help": "Skill families that must ALL be owned to own or learn this skill; each of a strictly lower grade."}),
		ContentSchema.origin_field(),
		_f("stance", "Stance", "enum", "AUTO", {"options": ["AUTO", "OFFENSIVE", "TACTICAL", "DEFENSIVE", "NEUTRAL"], "help": "AUTO: attacks use Offensive."}),
		_f("unlock_offset", "Unlock offset", "int", -1, {"min": -1, "max": 8, "help": "Levels after the family's unlock level (its class path level) at which THIS rank opens. -1 = derive: 0 / 2 / 4 for ranks 1 / 2 / 3. Rank 1 is always 0."}),
		_f("weapon_types", "Needs weapon type", "multi_pick", [], {"options_from": "weapon_types", "help": "Empty = any. Otherwise the wielded weapon must be one of these gear types, or the token 'unarmed' (no weapon in that hand; its proficiency is the class's fist level). Needs a proficiency of 1+. Class-derived units only."}),
		_f("min_proficiency", "Needs proficiency", "int", 0, {"min": 0, "max": 3, "help": "Training level in the wielded weapon type required to use the skill (0 = none)."}),
		_f("reach_min", "Reach min", "int", 0, {"min": 0, "max": 8, "help": "0 = the unit's default band (attacks: melee 1, ranged 2-3; support: 0-3)."}),
		_f("reach_max", "Reach max", "int", 0, {"min": 0, "max": 8}),
		_f("area", "Area", "enum", "NONE", {"options": ["NONE", "PLUS", "BURST", "LINE", "SWEEP"], "help": "NONE = single target. PLUS: tile + 4 neighbours, BURST: 3x3, LINE: 3 tiles from an adjacent tile, SWEEP: an L of 3 tiles around you (aim at a diagonal neighbour)."}),
		_f("power_mult", "Power x", "float", 1.0, {"min": 0.0, "max": 6.0, "step": 0.05, "help": "Multiplier on the unit's strike power. 0 = no damage (a debuff-only attack; needs an Inflicts status)."}),
		_f("weapon_hand", "Weapon hand", "enum", "main", {"options": ["main", "off", "both"], "help": "main: the main weapon. off: the off-hand weapon, or bare hands (melee, distance 1). both: dual-wield strike (engine pending)."}),
		_f("dash_tiles", "Gap-close tiles", "int", 0, {"min": 0, "max": 6, "help": "Lunge: move up to this many tiles to get into reach, then strike. Only offered while the target is out of reach; it is the unit's move for the round (one per round, blocked by root, terrain and units). AP is the skill's level alone."}),
		_f("status_effect", "Inflicts", "enum", "", {"options_from": "status"}),
		_f("status_potency", "Status potency %", "int", 0, {"min": 0, "max": 200, "when": {"key": "status_effect", "not_in": [""]}, "help": "Chance = potency - target debuff resistance - passive resistance."}),
		_f("status_magnitude", "Status magnitude", "int", 0, {"min": 0, "max": 100, "help": "Strength of the status (unit and range from the status registry: weaken %, expose/fortify armor points, regen/burn % max HP, poison HP per round)."}),
		_f("status_rounds", "Status rounds", "int", 1, {"min": 1, "max": 5, "help": "How many rounds an inflicted / granted status lasts once active (it starts next round; a DEFEND skill's riposte is active at once and counts the current round)."}),
		_f("target_mode", "Targets", "enum", "ALLY_OR_SELF", {"options": ["SELF", "ALLY", "ALLY_OR_SELF"], "help": "SELF: the caster only. ALLY: a friendly unit other than the caster. ALLY_OR_SELF: either."}),
		_f("heal_stat", "Heal scales with", "enum", "", {"options": ["", "INT", "KNOW"], "help": "Heal = heal base + stat x heal per point (SupportRules.heal_amount), never DamageCalculator."}),
		_f("heal_base", "Heal base", "int", 0, {"min": 0, "max": 500}),
		_f("heal_mult", "Heal per stat point", "float", 0.0, {"min": 0.0, "max": 10.0, "step": 0.05}),
		_f("grant_status", "Grants", "enum", "", {"options_from": "status_buff", "help": "SUPPORT: haste (+1 tile per move block) or taunt (enemy AI must target it while reachable), queued for next round. DEFEND: riposte (counters adjacent melee attackers for 50% power), active at once, lost on a stance change; or any support buff (taunt, haste), queued for next round like a support grant."}),
		_f("extra_grants", "Extra grants", "json", [], {"help": "BAL5-L: one more buff granted with the main grant, at the same timing (a DEFEND-only buff such as riposte at once, every other buff next round): [{\"status\": \"fortify\", \"rounds\": 2, \"magnitude\": 6}]. At most 1 entry; a buff other than Grants; rounds 1-3; magnitude inside the status range (0 for statuses without one). Ranks may override the whole list."}),
		_f("cleanse_statuses", "Cleanses", "multi_pick", [], {"options_from": "debuffs_or_all", "help": "Debuffs removed from the target (active and pending, never buffs): stagger, stun, root, bleed, or the keyword 'all' for every debuff. Empty = none. Combine freely with heal and grants."}),
		_f("mitigation", "Mitigation", "float", 0.5, {"min": 0.0, "max": 0.95, "step": 0.05}),
		_f("boost", "Attack boost x", "float", 1.5, {"min": 1.0, "max": 5.0, "step": 0.05}),
		_f("boost_rounds", "Boost rounds", "int", 2, {"min": 1, "max": 10}),
		_f("jump_tiles", "Jump tiles", "int", 0, {"min": 0, "max": 6, "help": "Disengage: jump up to this many tiles away from the nearest (or a chosen) enemy. It is the unit's move for the round (first in the sequence, refused when rooted); AP is the skill's level; units can be leapt over, walls cannot."}),
		_f("recover_ap", "AP recovered", "int", 2, {"min": 0, "max": 12}),
		# Combos (S12C-L): family-level fields (SkillRanks.FAMILY_KEYS), one value for every rank.
		_f("ends_in_stance", "Ends in stance", "enum", "", {"options": ["", "OFFENSIVE", "TACTICAL", "DEFENSIVE"], "help": "Flow move: after its last block the unit is in this stance for free, so a next action of that stance pays no switch. Empty = the skill's own stance. Leaving Defensive ends riposte, so a riposte DEFEND cannot flow. Set on the family, not per rank."}),
		_f("disrupt", "Disrupt", "enum", "NONE", {"options": ["NONE", "STANCE_BREAK"], "help": "STANCE_BREAK: a damaging hit the target survives knocks an Offensive or Defensive target into Tactical (its running guard and riposte end). Not a status: no resist roll. Set on the family, not per rank."}),
		_f("exploit", "Exploit", "json", [], {"help": "Status payoff: up to 2 entries [{\"status\": \"expose\", \"power_mult\": 1.3}]; against a target with one of these debuffs ACTIVE the skill's power is multiplied by the highest matching power (1.05-2.0). Set on the family, not per rank."}),
		_f("exploit_consume", "Consume exploited status", "bool", false, {"help": "After the hit, remove the matched active status with the highest multiplier. Needs an exploit status."}),
		_f("family", "Skill family", "string", "", {"readonly": true, "help": "Derived: the head skill id of a higher rank (empty on rank 1)."}),
		_f("tier", "Rank", "int", 1, {"min": 1, "max": 5, "readonly": true, "help": "Derived: 1 for the head skill, 2 / 3 for <id>_2 / <id>_3."}),
	]
	# Generated from SkillKinds: the kind lists (`when`), Forge sections and the rank flag.
	for f: Dictionary in list:
		var key: String = str(f["key"])
		var kinds: Array[String] = SkillKinds.kinds_with(key)
		if not f.has("when") and kinds.size() < SkillKinds.names().size():
			f["when"] = {"key": "kind", "in": kinds}
		var sections: Array[String] = []
		for section: Variant in SkillKinds.SECTION_FIELDS:
			if SkillKinds.section_fields(str(section)).has(key):
				sections.append(str(section))
		if not sections.is_empty():
			f["section"] = sections
		f["rank"] = SkillRanks.RANK_KEYS.has(key)
	return list
