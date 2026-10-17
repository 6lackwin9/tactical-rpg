class_name RewardGrant
extends RefCounted

# ==============================================================================
# Tactical RPG - Reward grant (S8, spec M4-L): quest rewards as a passive / skill / item SOURCE.
# xp to every active member (LevelUp.award, M3 level rules); items to the party stash; passives and skills to ONE
# recipient (empty id = the first active member). Picked world loot entries fold into the same lists first, so the
# recipient and "already owned" rules apply to them too. Validates everything before changing anything.
# ==============================================================================

# Marks world loot `qid` as claimed (EXP1-L chests): appends it and keeps loot_claimed sorted. "" or why not.
static func claim_loot(gs: GameState, qid: String) -> String:
	if not QuestRules.blob_key_ok(qid):
		return "Loot '%s' must be a qualified id world:id." % qid
	if gs.loot_claimed.has(qid):
		return "Loot '%s' is already claimed." % qid
	gs.loot_claimed.append(qid)
	gs.loot_claimed.sort()
	return ""

# `rewards`: {xp, items, passives, skills}; `loot`: resolved world loot entries (with `_qid`).
# Returns {ok, errors, granted: {xp, items, passives, skills, loot, recipient}, notes}.
static func apply(gs: GameState, rewards: Dictionary, recipient_id: String = "", loot: Array[Dictionary] = []) -> Dictionary:
	var errors: Array[String] = []
	var notes: Array[String] = []
	var r: Dictionary = DictUtil.dict_of(JsonUtil.normalize(rewards))
	errors.append_array(QuestRules.rewards_errors(r))
	var xp: int = DictUtil.to_int(r.get("xp", 0))
	var items: Dictionary = JsonUtil.ints(r.get("items", {}))
	var passives: Array[String] = JsonUtil.strings(r.get("passives", []))
	var skills: Array[String] = JsonUtil.strings(r.get("skills", []))
	var loot_ids: Array[String] = []
	for entry: Dictionary in loot:
		var lqid: String = str(entry.get("_qid", entry.get("id", "")))
		for problem: String in QuestRules.loot_entry_errors(entry):
			errors.append("Loot '%s': %s" % [lqid, problem])
		loot_ids.append(lqid)
		var ref: String = str(entry.get("ref", ""))
		match str(entry.get("reward_kind", "item")):
			"item":
				items[ref] = DictUtil.to_int(items.get(ref, 0)) + DictUtil.to_int(entry.get("count", 1))
			"passive":
				passives.append(ref)
			"skill":
				skills.append(ref)
	var recipient: UnitState = null
	if not passives.is_empty() or not skills.is_empty():
		if recipient_id.is_empty():
			var active: Array[UnitState] = gs.party.active_members()
			if active.is_empty():
				errors.append("No active party member to receive the passive / skill rewards.")
			else:
				recipient = active[0]
		else:
			recipient = gs.party.member(recipient_id)
			if recipient == null:
				errors.append("Recipient '%s' is not a party member." % recipient_id)
	elif not recipient_id.is_empty() and gs.party.member(recipient_id) == null:
		notes.append("Recipient '%s' is not a party member (ignored: nothing here needs a recipient)." % recipient_id)
	var granted: Dictionary = {"xp": 0, "items": {}, "passives": [] as Array[String], "skills": [] as Array[String], "loot": loot_ids, "recipient": recipient.id if recipient != null else ""}
	if not errors.is_empty():
		return {"ok": false, "errors": errors, "granted": granted, "notes": notes}
	if xp > 0:
		for m: UnitState in gs.party.active_members():
			LevelUp.award(m, xp)
		granted["xp"] = xp
	for id: Variant in items:
		gs.party.add_to_stash(str(id), DictUtil.to_int(items[id]))
	granted["items"] = items
	var got_passives: Array[String] = []
	var got_skills: Array[String] = []
	if recipient != null:
		for p: String in passives:
			if recipient.passives.has(p):
				notes.append("%s already owns passive '%s'." % [recipient.id, p])
			else:
				recipient.passives.append(p)
				got_passives.append(p)
		for s: String in skills:
			if recipient.owned_families().has(s) or recipient.learned_skills.has(s):
				notes.append("%s already owns skill '%s'." % [recipient.id, s])
			else:
				recipient.learned_skills.append(s)
				got_skills.append(s)
	granted["passives"] = got_passives
	granted["skills"] = got_skills
	return {"ok": true, "errors": errors, "granted": granted, "notes": notes}
