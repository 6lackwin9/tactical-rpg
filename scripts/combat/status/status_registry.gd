class_name StatusRegistry
extends RefCounted

# ==============================================================================
# Tactical RPG - Status Registry (the ONE table of every valid buff / debuff)
# StatusEffects derives debuffs() / buffs() / resistable() / support_buffs() / defend_buffs() /
# display_name() from ROWS and applies the stacking rule through stack().
#
# Row fields:
#   id             status key (a StatusEffects constant)
#   kind           "buff" | "debuff"
#   resistable     debuffs only: a unit / passive may carry status_resist for it, and attack skills
#                  roll it against resistance. Self-inflicted debuffs (AP debt) skip the roll.
#   default_rounds rounds it lasts when the source does not say (skills carry their own status_rounds)
#   stacking       what a second application does (see stack()):
#                    refresh_max_duration  rounds = longer of the two, potency = stronger of the two (today's behaviour)
#                    extend_duration       rounds add up (cap = max total rounds), potency = stronger
#                    stack_potency         rounds = longer, potency adds up (cap = max total potency)
#                    none                  the first application stays until it expires
#   cap            0 = no cap. extend_duration: max rounds; every other rule: max potency
#   display        player-facing name
#   grant          buffs only: "support" (queued by a SUPPORT skill) or "defend" (active now, DEFEND skill); "" for debuffs
#
# HOW TO ADD A STATUS (e.g. weaken, slice D):
#   1. add `const WEAKEN: String = "weaken"` to StatusEffects and one row to ROWS below (all fields);
#   2. effects of the status live where they act (blocked_blocks, damage hook, ...), never in the registry;
#   3. the derived lists, schema validation (status_effect / grant_status / cleanse), passive `resist` and the
#      Forge pick it up automatically; extend StatusEffects' header comment;
#   4. the registry test requires every StatusEffects id constant to have a row, so a forgotten row fails.
# ==============================================================================

const KIND_BUFF: String = "buff"
const KIND_DEBUFF: String = "debuff"

const REFRESH_MAX_DURATION: String = "refresh_max_duration"
const EXTEND_DURATION: String = "extend_duration"
const STACK_POTENCY: String = "stack_potency"
const NONE: String = "none"
const RULES: Array[String] = [REFRESH_MAX_DURATION, EXTEND_DURATION, STACK_POTENCY, NONE]

const ROWS: Array[Dictionary] = [
	{"id": "stagger", "kind": KIND_DEBUFF, "resistable": true, "default_rounds": 1, "stacking": REFRESH_MAX_DURATION, "cap": 0, "display": "Staggered", "grant": ""},
	{"id": "stun", "kind": KIND_DEBUFF, "resistable": true, "default_rounds": 1, "stacking": REFRESH_MAX_DURATION, "cap": 0, "display": "Stunned", "grant": ""},
	{"id": "root", "kind": KIND_DEBUFF, "resistable": true, "default_rounds": 1, "stacking": REFRESH_MAX_DURATION, "cap": 0, "display": "Rooted", "grant": ""},
	{"id": "bleed", "kind": KIND_DEBUFF, "resistable": true, "default_rounds": 1, "stacking": REFRESH_MAX_DURATION, "cap": 0, "display": "Bleeding", "grant": ""},
	{"id": "haste", "kind": KIND_BUFF, "resistable": false, "default_rounds": 1, "stacking": REFRESH_MAX_DURATION, "cap": 0, "display": "Hasted", "grant": "support"},
	{"id": "taunt", "kind": KIND_BUFF, "resistable": false, "default_rounds": 1, "stacking": REFRESH_MAX_DURATION, "cap": 0, "display": "Taunting", "grant": "support"},
	{"id": "riposte", "kind": KIND_BUFF, "resistable": false, "default_rounds": 2, "stacking": REFRESH_MAX_DURATION, "cap": 0, "display": "Riposte", "grant": "defend"},
]

# The row for `status`, or {} when unknown.
static func row(status: String) -> Dictionary:
	for r: Dictionary in ROWS:
		if str(r["id"]) == status:
			return r
	return {}

static func has(status: String) -> bool:
	return not row(status).is_empty()

static func ids() -> Array[String]:
	var out: Array[String] = []
	for r: Dictionary in ROWS:
		out.append(str(r["id"]))
	return out

# Ids matching a kind (and optionally a grant source), in table order.
static func ids_where(kind: String, resistable_only: bool = false, grant: String = "") -> Array[String]:
	var out: Array[String] = []
	for r: Dictionary in ROWS:
		if not kind.is_empty() and str(r["kind"]) != kind:
			continue
		if resistable_only and not DictUtil.to_bool(r["resistable"]):
			continue
		if not grant.is_empty() and str(r["grant"]) != grant:
			continue
		out.append(str(r["id"]))
	return out

static func default_rounds(status: String) -> int:
	return DictUtil.to_int(row(status).get("default_rounds", 1), 1)

static func stacking_of(status: String) -> String:
	return str(row(status).get("stacking", REFRESH_MAX_DURATION))

static func cap_of(status: String) -> int:
	return DictUtil.to_int(row(status).get("cap", 0))

# Problems with a row set (used by the registry test and when adding rows): empty = valid.
static func row_errors() -> Array[String]:
	var errors: Array[String] = []
	var seen: Array[String] = []
	for r: Dictionary in ROWS:
		var id: String = str(r.get("id", ""))
		if id.is_empty() or seen.has(id):
			errors.append("Duplicate or empty status id '%s'." % id)
		seen.append(id)
		if not [KIND_BUFF, KIND_DEBUFF].has(str(r.get("kind", ""))):
			errors.append("%s: kind must be buff or debuff." % id)
		if DictUtil.to_bool(r.get("resistable", false)) and str(r.get("kind", "")) != KIND_DEBUFF:
			errors.append("%s: only debuffs can be resistable." % id)
		if not RULES.has(str(r.get("stacking", ""))):
			errors.append("%s: unknown stacking rule '%s'." % [id, str(r.get("stacking", ""))])
		if DictUtil.to_int(r.get("default_rounds", 0)) < 1 or DictUtil.to_int(r.get("cap", 0)) < 0:
			errors.append("%s: default_rounds must be 1+ and cap 0+." % id)
		if str(r.get("display", "")).is_empty():
			errors.append("%s: needs a display name." % id)
		var grant: String = str(r.get("grant", ""))
		if (str(r.get("kind", "")) == KIND_BUFF) != (grant == "support" or grant == "defend"):
			errors.append("%s: buffs need grant support|defend, debuffs none." % id)
	return errors

# Combines an existing application (`existing_rounds` 0 = not present) with a new one under the
# status's stacking rule. Returns {"rounds": int, "potency": int}. Rounds are at least 1 once present.
# A status the registry does not know behaves as refresh_max_duration.
static func stack(existing_rounds: int, existing_potency: int, new_rounds: int, new_potency: int, status: String) -> Dictionary:
	return stack_with(stacking_of(status), cap_of(status), existing_rounds, existing_potency, new_rounds, new_potency)

# The stacking math for an explicit `rule` and `cap` (stack() reads both from the registry row).
static func stack_with(rule: String, cap: int, existing_rounds: int, existing_potency: int, new_rounds: int, new_potency: int) -> Dictionary:
	var add_rounds: int = maxi(1, new_rounds)
	var rounds: int = add_rounds
	var potency: int = new_potency
	if existing_rounds > 0:
		match rule:
			EXTEND_DURATION:
				rounds = existing_rounds + add_rounds
				potency = maxi(existing_potency, new_potency)
			STACK_POTENCY:
				rounds = maxi(existing_rounds, add_rounds)
				potency = existing_potency + new_potency
			NONE:
				rounds = existing_rounds
				potency = existing_potency
			_:
				rounds = maxi(existing_rounds, add_rounds)
				potency = maxi(existing_potency, new_potency)
	if cap > 0:
		if rule == EXTEND_DURATION:
			rounds = mini(rounds, cap)
		else:
			potency = mini(potency, cap)
	return {"rounds": rounds, "potency": potency}
