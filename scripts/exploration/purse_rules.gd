class_name PurseRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Encounter purse (PURSE1, spec docs/specs/PURSE1.md)
# An encounter may carry an authored `purse`: a gold range rolled on a win and, optionally, one world loot
# entry. Field on catalog and world encounters (json, default {} = no purse):
#   gold_min  int     0..GOLD_CAP                                       (default 0)
#   gold_max  int     gold_min..GOLD_CAP; above GOLD_SOFT_CAP warns     (default gold_min)
#   loot      String  a loot id of the encounter's world, or `world:id`; reward_kind item (default "")
#   once      bool    true = pays only the first win of the save        (default true)
#
# Static part (commit 1, schema slice): `read` (defaults filled), `problems` (both catalogs) and `warnings`.
# Runtime part (commit 2, spec section 1): `roll` (pure) and `pay` (BattleBridge.apply on won only). State:
# GameState.purses {qualified encounter id: wins}; +1 on every win of an encounter with a purse, paid or not.
# The gold roll uses its own RandomNumberGenerator seeded from (encounter qid, gs.time, wins so far), never the
# combat RNG, so battles and recordings replay identically and a reload does not reroll.
# ==============================================================================

const GOLD_CAP: int = 500
const GOLD_SOFT_CAP: int = 100
const KEYS: Array[String] = ["gold_min", "gold_max", "loot", "once"]

# The purse of an encounter with every default filled: {gold_min, gold_max, loot, once}; {} = no purse
# (missing, empty or not a dictionary). Values are read as validated (bad values are left to `problems`).
static func read(entry: Dictionary) -> Dictionary:
	var raw: Variant = entry.get("purse", {})
	if not (raw is Dictionary):
		return {}
	var p: Dictionary = raw
	if p.is_empty():
		return {}
	var lo: int = DictUtil.to_int(p.get("gold_min", 0))
	var once_raw: Variant = p.get("once", true)
	var once: bool = true
	if once_raw is bool:
		once = once_raw
	return {
		"gold_min": lo,
		"gold_max": DictUtil.to_int(p.get("gold_max", lo), lo),
		"loot": str(p.get("loot", "")),
		"once": once,
	}

# Blocking problems of an encounter's purse. `loot_check(ref) -> String` returns "" when the loot ref is fine,
# "missing" when it does not resolve, or "kind" when it resolves to a non-item reward.
static func problems(entry: Dictionary, loot_check: Callable) -> Array[String]:
	var errors: Array[String] = []
	if not entry.has("purse"):
		return errors
	var raw: Variant = entry["purse"]
	if not (raw is Dictionary):
		errors.append("Purse must be an object {gold_min, gold_max, loot, once}.")
		return errors
	var p: Dictionary = raw
	if p.is_empty():
		return errors
	for key: Variant in p:
		if not KEYS.has(str(key)):
			errors.append("Purse key '%s' is not allowed." % str(key))
	var lo_raw: Variant = JsonUtil.normalize(p.get("gold_min", 0))
	var lo_ok: bool = lo_raw is int and DictUtil.to_int(lo_raw) >= 0 and DictUtil.to_int(lo_raw) <= GOLD_CAP
	if not lo_ok:
		errors.append("Purse gold_min must be a whole number 0..%d." % GOLD_CAP)
	var lo: int = DictUtil.to_int(lo_raw) if lo_ok else 0
	if p.has("gold_max"):
		var hi_raw: Variant = JsonUtil.normalize(p["gold_max"])
		if not (hi_raw is int) or DictUtil.to_int(hi_raw) < lo or DictUtil.to_int(hi_raw) > GOLD_CAP:
			errors.append("Purse gold_max must be gold_min..%d." % GOLD_CAP)
	if p.has("loot"):
		var loot_raw: Variant = p["loot"]
		if not (loot_raw is String):
			errors.append("Purse loot '%s' must be an item id." % str(loot_raw))
		else:
			var ref: String = loot_raw
			if not ref.is_empty():
				match str(loot_check.call(ref)):
					"missing":
						errors.append("Purse loot '%s' does not resolve." % ref)
					"kind":
						errors.append("Purse loot '%s' must be an item reward." % ref)
	if p.has("once") and not (p["once"] is bool):
		errors.append("Purse once must be true or false.")
	return errors

# Soft problems of an encounter's purse (never blocking): gold above the soft cap, an empty purse.
static func warnings(entry: Dictionary) -> Array[String]:
	var out: Array[String] = []
	var p: Dictionary = read(entry)
	if p.is_empty():
		return out
	var hi: int = DictUtil.to_int(p["gold_max"])
	if hi > GOLD_SOFT_CAP:
		out.append("Purse gold above %d: check the budget." % GOLD_SOFT_CAP)
	if hi <= 0 and str(p["loot"]).is_empty():
		out.append("Empty purse.")
	return out

# The shared loot check of `problems` for a world encounter: `ref` resolved from `world_id` (a qualified
# `world:id` resolves in that world). A catalog encounter has no world, so only qualified refs resolve there.
static func world_loot_check(world_id: String, ref: String) -> String:
	var hit: Dictionary = WorldCatalog.resolve(world_id, "loot", ref)
	if hit.is_empty():
		return "missing"
	if str(hit.get("reward_kind", "item")) != "item":
		return "kind"
	return ""

# ------------------------------------------------------------------------------
# Runtime (PURSE1-L commit 2)
# ------------------------------------------------------------------------------

# The encounter `encounter_ref` names from gs.world_id ({} if none); carries `_qid` and `_world` (rule 1.8).
static func encounter_of(gs: GameState, encounter_ref: String) -> Dictionary:
	return WorldCatalog.resolve(gs.world_id, "encounters", encounter_ref)

# Rule 1.3 seed: ("<qid>|<time>|<paid>").hash().
static func seed_of(encounter_qid: String, time: int, paid: int) -> int:
	return ("%s|%d|%d" % [encounter_qid, time, paid]).hash()

# Pure (nothing changes): {qid, has_purse, pays, gold, loot (loot qid or "")} for a win of `encounter_ref` now.
# pays = the encounter has a purse and (once is false or it was never won); gold 0 and loot "" when it does not pay;
# loot "" also when the entry is missing, already claimed or its requires_flags fail (rule 1.5).
static func roll(gs: GameState, encounter_ref: String) -> Dictionary:
	var enc: Dictionary = encounter_of(gs, encounter_ref)
	var qid: String = str(enc.get("_qid", ""))
	var out: Dictionary = {"qid": qid, "has_purse": false, "pays": false, "gold": 0, "loot": ""}
	var p: Dictionary = read(enc)
	if enc.is_empty() or p.is_empty():
		return out
	out["has_purse"] = true
	var paid: int = DictUtil.to_int(gs.purses.get(qid, 0))
	var pays: bool = paid == 0 or not DictUtil.to_bool(p["once"])
	out["pays"] = pays
	if not pays:
		return out
	var lo: int = clampi(DictUtil.to_int(p["gold_min"]), 0, GOLD_CAP)
	var hi: int = clampi(DictUtil.to_int(p["gold_max"]), lo, GOLD_CAP)
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_of(qid, gs.time, paid)
	out["gold"] = rng.randi_range(lo, hi)
	out["loot"] = _loot_qid(gs, str(enc.get("_world", "")), str(p["loot"]))
	return out

# The loot qid a paying win grants ("" = none): resolved in the encounter's world, unclaimed, flags met.
static func _loot_qid(gs: GameState, world_id: String, ref: String) -> String:
	if ref.is_empty():
		return ""
	var loot: Dictionary = WorldCatalog.resolve(world_id, "loot", ref)
	if loot.is_empty() or str(loot.get("reward_kind", "item")) != "item":
		return ""
	var lqid: String = str(loot["_qid"])
	if gs.loot_claimed.has(lqid):
		return ""
	for c: Variant in DictUtil.array_of(JsonUtil.normalize(loot.get("requires_flags", []))):
		if not QuestRules.condition_met(gs, DictUtil.dict_of(c)):
			return ""
	return lqid

# Applies a win of `encounter_ref` (call on won only; BattleBridge.apply does). Returns {} for an encounter without
# a purse (nothing changes), else {gold (added), loot (qid or ""), notes}: wins +1, gold through PartyState.add_gold
# (GOLD_MAX cap note), loot claimed and its item put in the stash (RewardGrant).
static func pay(gs: GameState, encounter_ref: String) -> Dictionary:
	var r: Dictionary = roll(gs, encounter_ref)
	if not DictUtil.to_bool(r["has_purse"]):
		return {}
	var qid: String = str(r["qid"])
	var notes: Array[String] = []
	var out: Dictionary = {"gold": 0, "loot": "", "notes": notes}
	gs.purses[qid] = DictUtil.to_int(gs.purses.get(qid, 0)) + 1
	if not DictUtil.to_bool(r["pays"]):
		return out
	var lqid: String = str(r["loot"])
	var loot: Array[Dictionary] = []
	if not lqid.is_empty():
		loot.append(WorldCatalog.resolve(gs.world_id, "loot", lqid))
	var granted: Dictionary = RewardGrant.apply(gs, {"gold": DictUtil.to_int(r["gold"])}, "", loot)
	if not DictUtil.to_bool(granted["ok"]) and not loot.is_empty():
		notes.append("Purse loot '%s' skipped: %s" % [lqid, "; ".join(JsonUtil.strings(granted["errors"]))])
		loot.clear()
		lqid = ""
		granted = RewardGrant.apply(gs, {"gold": DictUtil.to_int(r["gold"])}, "", loot)
	notes.append_array(JsonUtil.strings(granted.get("notes", [])))
	if not DictUtil.to_bool(granted["ok"]):
		notes.append_array(JsonUtil.strings(granted["errors"]))
		return out
	if not lqid.is_empty():
		var why: String = RewardGrant.claim_loot(gs, lqid)
		if not why.is_empty():
			notes.append(why)
	out["gold"] = DictUtil.to_int(DictUtil.dict_of(granted["granted"]).get("gold", 0))
	out["loot"] = lqid
	return out

# The settled-fight keys of a paid purse for the results popup (rule 1.6): gold when > 0, loot when set.
static func result_keys(purse: Dictionary) -> Dictionary:
	var out: Dictionary = {}
	if DictUtil.to_int(purse.get("gold", 0)) > 0:
		out["gold"] = DictUtil.to_int(purse["gold"])
	if not str(purse.get("loot", "")).is_empty():
		out["loot"] = str(purse["loot"])
	return out
