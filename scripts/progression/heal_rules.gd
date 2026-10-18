class_name HealRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Heal rules (HEAL1-L, spec docs/specs/HEAL1.md section 3, option C): HP between battles.
# Each party member stores hp_lost (UnitState); current HP = clampi(max_hp - hp_lost, 1, max_hp) with max_hp derived
# from stats and gear. After a won battle every member who fought rallies RALLY_PCT of max HP (a downed hero gets up at
# that much); fled = at least 1 HP, no rally; lost = the defeat rule (on_defeat). A rest (rest place on entry, make camp
# for CAMP_COST supplies on the world map) fully restores every roster member. Pure and static; the sim shares hp_after.
# ==============================================================================

const RALLY_PCT: int = 25
const CAMP_COST: int = 3

const MODE_REST_PLACE: String = "rest_place"
const MODE_FORCED_REST: String = "forced_rest"
const MODE_BACK_TO_TOWN: String = "back_to_town"

const REFUSE_NOT_AT_NODE: String = "Make camp on the world map."
const REFUSE_RESTED: String = "Everyone is rested."
const REFUSE_SUPPLIES: String = "Making camp needs 3 supplies."

# HP a rally adds (rule 3.2): RALLY_PCT of max HP, rounded up.
static func rally(max_hp: int) -> int:
	return ceili(max_hp * RALLY_PCT / 100.0)

# Rule 3.2 for one hero who fought: won = +rally (a downed hero gets up at the rally), fled = at least 1 HP,
# lost = max_hp (the defeat heals; on_defeat applies it to the roster). Unknown outcomes read as fled.
static func hp_after(hp: int, max_hp: int, outcome: String) -> int:
	match outcome:
		"won":
			if hp <= 0:
				return mini(max_hp, rally(max_hp))
			return mini(max_hp, hp + rally(max_hp))
		"lost":
			return max_hp
	return clampi(hp, 1, maxi(max_hp, 1))

# Rule 3.1.1: start HP from a max HP and a stored hp_lost (never below 1, never above max).
static func start_hp(max_hp: int, hp_lost: int) -> int:
	return clampi(max_hp - maxi(hp_lost, 0), 1, maxi(max_hp, 1))

# The member's max HP from its current stats and gear as a game battle builds it (PartyBuilder with auto-spend, like
# GameEncounter; unspent points count), so read-outs and rest totals match the battle; 0 when it does not build.
static func max_hp_of(state: UnitState) -> int:
	if state == null:
		return 0
	var c: Combatant = PartyBuilder.build(state.duplicate_state())
	return c.max_hp if c != null else 0

# {hp, max} for a UI read-out (rule 3.1.1).
static func current_hp(state: UnitState) -> Dictionary:
	var max_hp: int = max_hp_of(state)
	var lost: int = state.hp_lost if state != null else 0
	return {"hp": start_hp(max_hp, lost) if max_hp > 0 else 0, "max": max_hp}

# Any roster member (active or reserve) hurt.
static func hurt(party: PartyState) -> bool:
	for m: UnitState in party.members:
		if m.hp_lost > 0:
			return true
	return false

# Rule 3.3: every roster member back to full; returns the HP restored in total (0 = nobody was hurt).
static func rest(party: PartyState) -> int:
	var restored: int = 0
	for m: UnitState in party.members:
		if m.hp_lost <= 0:
			continue
		var max_hp: int = max_hp_of(m)
		if max_hp > 0:
			restored += max_hp - start_hp(max_hp, m.hp_lost)
		m.hp_lost = 0
	return restored

# Whether a location ref (seen from gs.world_id; qualified ids of other worlds resolve too) is a rest place.
static func is_rest_place(gs: GameState, location_ref: String) -> bool:
	if location_ref.is_empty():
		return false
	return DictUtil.to_bool(ExploreRunner.resolve_location(gs, location_ref).get("rest", false))

# Rule 3.3.2: "" when the party can make camp, else the refusal text. Only the state is checked; the caller (world map)
# also offers it only while no battle, dialogue or trip is running.
static func can_camp(gs: GameState) -> String:
	if TravelRunner.current_node(gs).is_empty():
		return REFUSE_NOT_AT_NODE
	if not hurt(gs.party):
		return REFUSE_RESTED
	if gs.party.supplies < CAMP_COST:
		return REFUSE_SUPPLIES
	return ""

# Rule 3.3.2: {ok, errors, spent, restored, events}; all-or-nothing (a refusal changes nothing). No clock, no fight roll;
# the events end with an autosave like an arrival.
static func make_camp(gs: GameState) -> Dictionary:
	var errors: Array[String] = []
	var events: Array[Dictionary] = []
	var refusal: String = can_camp(gs)
	if not refusal.is_empty():
		errors.append(refusal)
		return {"ok": false, "errors": errors, "spent": 0, "restored": 0, "events": events}
	var spent: int = gs.party.spend_supplies(CAMP_COST)
	var restored: int = rest(gs.party)
	events.append({"kind": "camped", "location": gs.location_id, "spent": spent, "restored": restored})
	events.append({"kind": "autosave"})
	return {"ok": true, "errors": errors, "spent": spent, "restored": restored, "events": events}

# explore.last_rest as a qualified id when it still resolves, else "" (the start location; never a load error).
static func last_rest(gs: GameState) -> String:
	var ref: String = str(gs.explore.get("last_rest", ""))
	var hit: Dictionary = ExploreRunner.resolve_location(gs, ref) if not ref.is_empty() else {}
	return str(hit.get("_qid", "")) if not hit.is_empty() else ""

# Rule 3.3.3: heals every roster member and picks where the party wakes. `checkpoint` is the wake target ("" = the start
# location). Returns {mode, spent, to}: rest_place (the checkpoint rests; free), forced_rest (CAMP_COST supplies spent;
# wake at the checkpoint) or back_to_town (fewer supplies, kept; wake at last_rest, "" = the start location).
# = defeat_plan then settle_defeat; ExploreRunner.wake settles only once the wake target is known to be enterable.
static func on_defeat(gs: GameState, checkpoint: String) -> Dictionary:
	var plan: Dictionary = defeat_plan(gs, checkpoint)
	settle_defeat(gs, plan)
	return plan

# Rule 3.3.3 without changing anything: {mode, spent (what settling costs), to}.
static func defeat_plan(gs: GameState, checkpoint: String) -> Dictionary:
	var at: String = checkpoint if not checkpoint.is_empty() else MapRules.start_location(gs.world_id)
	if is_rest_place(gs, at):
		return {"mode": MODE_REST_PLACE, "spent": 0, "to": checkpoint}
	if gs.party.supplies >= CAMP_COST:
		return {"mode": MODE_FORCED_REST, "spent": CAMP_COST, "to": checkpoint}
	return {"mode": MODE_BACK_TO_TOWN, "spent": 0, "to": last_rest(gs)}

# Applies a defeat_plan: spends its supplies and fully heals every roster member.
static func settle_defeat(gs: GameState, plan: Dictionary) -> void:
	var _paid: int = gs.party.spend_supplies(DictUtil.to_int(plan.get("spent", 0)))
	for m: UnitState in gs.party.members:
		m.hp_lost = 0
