class_name BattleBridge
extends RefCounted

# ==============================================================================
# Tactical RPG - Battle bridge (S9, spec EXP1-L 2.6): the shared way into and out of a battle from the game state.
# Exploration-agnostic (ExploreRunner, TRAV1 road fights and DLG1 use it). Full heal is implicit: M3 stores no HP.
# Outcomes: won | lost | fled. X1 (user 2026-10-17): a battle that reaches its round limit with both sides
# standing counts as fled; so does a battle the whole party left by withdrawing (alive, off the field).
# ==============================================================================

const OUTCOMES: Array[String] = ["won", "lost", "fled"]

# {ok, errors, encounter: GameEncounter or null}: `encounter_qid` resolved from gs.world_id, played with the active
# party members; `zone` (non-empty) replaces the arena zone.
static func encounter_for(gs: GameState, encounter_qid: String, zone: String = "") -> Dictionary:
	var errors: Array[String] = []
	var hit: Dictionary = WorldCatalog.resolve(gs.world_id, "encounters", encounter_qid)
	if hit.is_empty():
		errors.append("Unknown encounter '%s'." % encounter_qid)
		return {"ok": false, "errors": errors, "encounter": null}
	if gs.party.active_members().is_empty():
		errors.append("No active party member to fight.")
		return {"ok": false, "errors": errors, "encounter": null}
	var hit_world: String = str(hit["_world"])
	var world: String = gs.world_id if WorldCatalog.lineage(gs.world_id).has(hit_world) else hit_world
	var enc := GameEncounter.new(world, str(hit["id"]), gs.party, zone)
	return {"ok": true, "errors": errors, "encounter": enc}

# won (combat over, a party member stands), lost (over, every party member down), fled (round limit reached with
# both sides standing, or the party withdrew); "" while the battle can still go on.
static func outcome(session: CombatSession) -> String:
	var standing: bool = false
	var alive: bool = false
	for c: Combatant in session.allies:
		standing = standing or c.is_active()
		alive = alive or c.is_alive()
	if session.combat_over:
		if standing:
			return "won"
		return "fled" if alive else "lost"
	var at_limit: bool = session.phase == CombatSession.Phase.REPLAY and session.current_round >= session.encounter.max_rounds
	return "fled" if at_limit else ""

# [{level, tier}] of the encounter's enemies and reinforcements (the XP inputs), seen from `world_id`.
static func enemy_levels(world_id: String, encounter_qid: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var enc: Dictionary = WorldCatalog.resolve(world_id, "encounters", encounter_qid)
	var refs: Array[String] = JsonUtil.strings(enc.get("enemies", []))
	refs.append_array(JsonUtil.strings(enc.get("wave_units", [])))
	for ref: String in refs:
		var entry: Dictionary = WorldFactory.unit_entry(WorldCatalog.resolve(world_id, "characters", ref))
		if entry.is_empty():
			continue
		var unit: Dictionary = ContentSchema.normalize("units", entry)
		out.append({"level": DictUtil.to_int(unit["level"], 1), "tier": str(unit["tier"])})
	return out

# {ok, errors, xp: member id -> LevelUp result, advanced: quest qids}. won: XP to every active member, then the quest
# hook; lost / fled change nothing here.
static func apply(gs: GameState, encounter_qid: String, result: String) -> Dictionary:
	var errors: Array[String] = []
	var advanced: Array[String] = []
	if not OUTCOMES.has(result):
		errors.append("Unknown battle outcome '%s' (use won, lost or fled)." % result)
		return {"ok": false, "errors": errors, "xp": {}, "advanced": advanced}
	if result != "won":
		return {"ok": true, "errors": errors, "xp": {}, "advanced": advanced}
	var xp: Dictionary = LevelUp.award_encounter(gs.party, enemy_levels(gs.world_id, encounter_qid))
	advanced = QuestRunner.on_encounter_won(gs, encounter_qid)
	return {"ok": true, "errors": errors, "xp": xp, "advanced": advanced}
