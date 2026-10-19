class_name RewardGrant
extends RefCounted

# ==============================================================================
# Tactical RPG - Reward grant (S8, spec M4-L): quest rewards as a passive / skill / item SOURCE.
# xp to every active member (LevelUp.award, M3 level rules); items to the party stash; passives and skills to ONE
# recipient (empty id = the first active member). Picked world loot entries fold into the same lists first, so the
# recipient and "already owned" rules apply to them too. Validates everything before changing anything.
# DLG1-L: gold to party.gold (quests, chests, dialogue gifts) and take() for dialogue `take` effects.
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

# `rewards`: {xp, items, passives, skills, gold}; `loot`: resolved world loot entries (with `_qid`).
# Returns {ok, errors, granted: {xp, items, passives, skills, gold, loot, recipient}, notes}.
# `gold` (DLG1-L) is checked here only as a whole number 0..PartyState.GOLD_MAX: the per-source caps (template
# QuestRules.GOLD_MAX, chest MapRules.CHEST_GOLD_MAX, dialogue DialogueRules.EFFECT_GOLD_MAX) are content validation.
static func apply(gs: GameState, rewards: Dictionary, recipient_id: String = "", loot: Array[Dictionary] = []) -> Dictionary:
	var errors: Array[String] = []
	var notes: Array[String] = []
	var r: Dictionary = DictUtil.dict_of(JsonUtil.normalize(rewards))
	var gold_raw: Variant = r.get("gold", 0)
	var checked: Dictionary = r.duplicate(true)
	checked.erase("gold")
	errors.append_array(QuestRules.rewards_errors(checked))
	if not (gold_raw is int) or DictUtil.to_int(gold_raw) < 0 or DictUtil.to_int(gold_raw) > PartyState.GOLD_MAX:
		errors.append("Rewards gold must be a whole number 0..%d." % PartyState.GOLD_MAX)
	var gold: int = DictUtil.to_int(gold_raw)
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
	var granted: Dictionary = {"xp": 0, "items": {}, "passives": [] as Array[String], "skills": [] as Array[String], "gold": 0, "loot": loot_ids, "recipient": recipient.id if recipient != null else ""}
	if not errors.is_empty():
		return {"ok": false, "errors": errors, "granted": granted, "notes": notes}
	if gold > 0:
		var added: int = gs.party.add_gold(gold)
		granted["gold"] = added
		if added < gold:
			notes.append("Gold is capped at %d (%d not added)." % [PartyState.GOLD_MAX, gold - added])
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

# Removes `items` ({base id: count}) and `gold` from the party (DLG1-L dialogue `take`). Checks every holding first
# ("Not enough %s."); nothing changes on any problem. Returns the problems.
static func take(gs: GameState, items: Dictionary, gold: int = 0) -> Array[String]:
	var errors: Array[String] = []
	var wanted: Dictionary = JsonUtil.ints(items)
	for id: Variant in wanted:
		var n: int = DictUtil.to_int(wanted[id])
		if n < 1:
			errors.append("Take count of '%s' must be >= 1." % str(id))
		elif DictUtil.to_int(gs.party.stash.get(str(id), 0)) < n:
			var entry: Dictionary = ContentCatalog.get_entry("items", str(id))
			errors.append("Not enough %s." % str(entry.get("name", id)))
	if gold < 0:
		errors.append("Take gold must be >= 0.")
	elif gs.party.gold < gold:
		errors.append("Not enough gold.")
	if not errors.is_empty():
		return errors
	for id: Variant in wanted:
		var _ok: bool = gs.party.remove_from_stash(str(id), DictUtil.to_int(wanted[id]))
	gs.party.gold -= gold
	return errors
