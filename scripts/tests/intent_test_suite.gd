class_name IntentTestSuite
extends RefCounted

# ==============================================================================
# INTENT1-L: enemy intent tells (CombatSession.enemy_intents, IntentRules) and the
# `foresight` passive effect. Hand-built fixtures only: a scripted encounter whose enemy
# schedule is written by the test, and an AI encounter that plans with TacticalAI.
# Expectations come from the scheduled actions and the rules, never from shipped data.
# ==============================================================================

const TAG: String = "Intent"
const EYE: Dictionary = {"id": "test_eye", "name": "Test Eye", "effects": [{"type": "foresight"}]}

# Scripted battle: the test writes each enemy's schedule (no AI), so every reveal case is exact.
class ScriptedEncounter:
	extends CombatEncounter

	var with_foresight: bool = true
	var bryn: Combatant = null
	var sella: Combatant = null
	var stalker: Combatant = null    # attacks Bryn in block 4 only
	var cutter: Combatant = null     # attacks Sella, then rests
	var bomber: Combatant = null     # area aimed next to Bryn whose cells cover Bryn's tile
	var misser: Combatant = null     # area far from everyone
	var hunter: Combatant = null     # an Engage on Sella (moves, then attacks)
	var dazed: Combatant = null      # a stunned block
	var walker: Combatant = null     # walks first, attacks Bryn later
	var latecomer: Combatant = null  # off the grid when the round is scheduled (a wave unit)
	var bomber_aim: Vector2i = Vector2i(4, 3)
	var bomber_cells: Array[Vector2i] = [Vector2i(4, 3), Vector2i(4, 2), Vector2i(4, 4)]
	var misser_cells: Array[Vector2i] = [Vector2i(10, 8)]
	var engage_problem: String = ""   # CombatPlan.add_step result of the hunter's Engage ("" = planned)

	func build() -> void:
		max_rounds = 1
		total_blocks = 6
		grid = CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
		bryn = TestHelpers.unit("Bryn", grid, Vector2i(4, 4))
		sella = TestHelpers.unit("Sella", grid, Vector2i(4, 6))
		if with_foresight:
			Passives.apply(bryn, EYE)
		stalker = TestHelpers.unit("Stalker", grid, Vector2i(5, 4))
		cutter = TestHelpers.unit("Cutter", grid, Vector2i(5, 6))
		bomber = TestHelpers.unit("Bomber", grid, Vector2i(7, 3))
		misser = TestHelpers.unit("Misser", grid, Vector2i(10, 7))
		hunter = TestHelpers.unit("Hunter", grid, Vector2i(7, 7))
		dazed = TestHelpers.unit("Dazed", grid, Vector2i(9, 1))
		walker = TestHelpers.unit("Walker", grid, Vector2i(7, 4))
		latecomer = Combatant.new("Latecomer", 400, 10, 1)
		allies = [bryn, sella]
		enemies = [stalker, cutter, bomber, misser, hunter, dazed, walker, latecomer]
		ai_profiles.clear()
		for c: Combatant in allies + enemies:
			ai_profiles[c] = AiProfile.create(AiProfile.Role.AGGRESSOR, 0.0, 3)

	func schedule_round(resolver: CombatResolver, _round_num: int) -> void:
		resolver.schedule_action(stalker, AttackCommand.new(stalker, bryn, 1.0, 2, 0, grid), 4)
		resolver.schedule_action(cutter, AttackCommand.new(cutter, sella, 1.0, 2, 0, grid), 1)
		resolver.schedule_action(cutter, RestCommand.new(cutter, 2, 1), 3)
		var area := AreaAttackCommand.new(bomber, grid, bomber_cells, 1.0, 2)
		area.aim_cell = bomber_aim
		resolver.schedule_action(bomber, area, 1)
		resolver.schedule_action(misser, AreaAttackCommand.new(misser, grid, misser_cells, 1.0, 2), 2)
		var engage_plan := CombatPlan.new(plan_context())
		engage_problem = engage_plan.add_step(hunter, PlanStep.engage(SkillCatalog.find("strike"), sella), false)
		engage_plan.commit(resolver)
		resolver.schedule_action(dazed, StatusBlockCommand.new(dazed, "stun"), 1)
		var path: Array[Vector2i] = [Vector2i(7, 5)]
		var walk := WalkCommand.new(walker, grid, path, 1)
		walk.goal = Vector2i(7, 5)
		resolver.schedule_action(walker, walk, 1)
		resolver.schedule_action(walker, AttackCommand.new(walker, bryn, 1.0, 2, 0, grid), 3)

# AI battle: both sides plan with TacticalAI (the enemy side through the default schedule_round).
class AiEncounter:
	extends CombatEncounter

	func build() -> void:
		max_rounds = 3
		total_blocks = 6
		grid = CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
		var a1: Combatant = TestHelpers.unit("Bryn", grid, Vector2i(3, 4))
		var a2: Combatant = TestHelpers.unit("Sella", grid, Vector2i(3, 5), 400, 10, 3)
		Passives.apply(a1, EYE)
		var f1: Combatant = TestHelpers.unit("FoeA", grid, Vector2i(7, 4))
		var f2: Combatant = TestHelpers.unit("FoeB", grid, Vector2i(7, 5))
		var f3: Combatant = TestHelpers.unit("FoeC", grid, Vector2i(9, 3), 400, 10, 3)
		allies = [a1, a2]
		enemies = [f1, f2, f3]
		ai_profiles.clear()
		for c: Combatant in allies + enemies:
			ai_profiles[c] = AiProfile.create(AiProfile.Role.SNIPER if c.attack_range > 1 else AiProfile.Role.AGGRESSOR, 0.25, 3)

static func run(_tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== INTENT1 ENEMY INTENT TELLS ASSERTIONS ===")
	print("=======================================================")
	var ok: bool = true
	ok = _test_foresight_effect() and ok
	ok = _test_kind_of() and ok
	ok = _test_matches_schedule() and ok
	ok = _test_reset_round() and ok
	ok = _test_engage_and_arriving() and ok
	ok = _test_pack_drafts() and ok
	ok = _test_reveal() and ok
	ok = _test_display_only() and ok
	return ok

# ------------------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------------------

static func _name_of(raw: Variant) -> String:
	if raw is Combatant:
		var c: Combatant = raw
		return c.name
	return "-"

static func _line(r: Dictionary) -> String:
	return "%s %s>%s %s %s" % [str(r["kind"]), str(r["skill"]), _name_of(r.get("target", null)), str(r["cell"]), str(r["blocks"])]

# Unit-name keyed text of a record list (stable across start(), which rebuilds the combatants).
static func _sig(records: Array[Dictionary]) -> Array[String]:
	var out: Array[String] = []
	for r: Dictionary in records:
		var plan_lines: Array[String] = []
		for p: Variant in DictUtil.array_of(r["plan"]):
			plan_lines.append(_line(DictUtil.dict_of(p)))
		out.append("%s: %s revealed=%s [%s]" % [_name_of(r["unit"]), _line(r), str(r["revealed"]), "; ".join(plan_lines)])
	return out

static func _record_for(records: Array[Dictionary], unit: Combatant) -> Dictionary:
	for r: Dictionary in records:
		if r["unit"] == unit:
			return r
	return {}

static func _first_action(session: CombatSession, unit: Combatant) -> CombatResolver.ScheduledAction:
	var first: CombatResolver.ScheduledAction = null
	for a: CombatResolver.ScheduledAction in session.resolver.scheduled_actions:
		if a.combatant == unit and (first == null or a.start_block < first.start_block):
			first = a
	return first

static func _count_actions(session: CombatSession, unit: Combatant) -> int:
	var n: int = 0
	for a: CombatResolver.ScheduledAction in session.resolver.scheduled_actions:
		if a.combatant == unit:
			n += 1
	return n

static func _scripted(with_foresight: bool) -> CombatSession:
	var enc := ScriptedEncounter.new()
	enc.with_foresight = with_foresight
	var session := CombatSession.new(enc)
	session.start()
	return session

# Every log entry of the battle plus the final HP of every unit.
static func _outcome(session: CombatSession) -> Array[String]:
	var out: Array[String] = []
	for e: ActionLogEntry in session.log_entries:
		out.append("%s %s %s %s %s %d %s" % [e.ref_tag, e.category, e.actor_name, e.skill_name, e.target_name, e.damage, e.outcome_tag])
	for c: Combatant in session.allies + session.enemies:
		out.append("%s hp %d" % [c.name, c.hp])
	return out

# ------------------------------------------------------------------------------
# 1. The foresight passive effect (schema: passives only, no params)
# ------------------------------------------------------------------------------

static func _test_foresight_effect() -> bool:
	var checks: Dictionary = {}
	var any_skill: Callable = func(_id: String) -> bool: return true
	checks["foresight is a listed type"] = Passives.TYPES.has("foresight")
	checks["not an item type"] = not Passives.ITEM_TYPES.has("foresight")
	checks["a passive with foresight validates"] = Passives.validate_effects([{"type": "foresight"}], any_skill).is_empty()
	checks["refused in the item grammar"] = not Passives.validate_effects([{"type": "foresight"}], any_skill, true, true).is_empty()
	checks["has an effect spec without params"] = DictUtil.array_of(Passives.effect_spec("foresight").get("params", ["x"])).is_empty()
	checks["described"] = Passives.describe(EYE) != "no effects"
	var plain := Combatant.new("Plain", 100, 10, 1)
	checks["no foresight by default"] = not plain.has_foresight()
	var holder := Combatant.new("Holder", 100, 10, 1)
	Passives.apply(holder, EYE)
	checks["applied passive grants it"] = holder.has_foresight() and holder.foresight_source == str(EYE["id"])
	var geared := Combatant.new("Geared", 100, 10, 1)
	Passives.apply_effects(geared, DictUtil.array_of(EYE["effects"]), "gear:test", true)
	checks["gear-sourced effects never grant it"] = not geared.has_foresight()
	return TestHelpers.report(TAG, "foresight effect (passives only, Combatant.has_foresight)", checks)

# ------------------------------------------------------------------------------
# 2. IntentRules.kind_of (one kind per command class; a walk to an exit is a flee)
# ------------------------------------------------------------------------------

static func _test_kind_of() -> bool:
	var checks: Dictionary = {}
	var exit_cell := Vector2i(0, 4)
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [exit_cell])
	var me: Combatant = TestHelpers.unit("Me", grid, Vector2i(3, 4))
	var foe: Combatant = TestHelpers.unit("Foe", grid, Vector2i(4, 4))
	var no_path: Array[Vector2i] = []
	var cells: Array[Vector2i] = [Vector2i(4, 4)]
	checks["attack"] = IntentRules.kind_of(AttackCommand.new(me, foe, 1.0, 2, 0, grid)) == "attack"
	checks["area"] = IntentRules.kind_of(AreaAttackCommand.new(me, grid, cells, 1.0, 2)) == "area"
	checks["support"] = IntentRules.kind_of(SupportCommand.new(me, me, null, grid, 1)) == "support"
	checks["guard"] = IntentRules.kind_of(DefendCommand.new(me)) == "guard"
	checks["focus"] = IntentRules.kind_of(FocusCommand.new(me)) == "focus"
	checks["rest"] = IntentRules.kind_of(RestCommand.new(me)) == "rest"
	checks["retreat"] = IntentRules.kind_of(DisengageCommand.new(me, grid, Vector2i(2, 4), 1)) == "retreat"
	var walk := WalkCommand.new(me, grid, no_path, 1)
	walk.goal = Vector2i(2, 4)
	checks["move"] = IntentRules.kind_of(walk) == "move"
	var flee := WalkCommand.new(me, grid, no_path, 1)
	flee.goal = exit_cell
	checks["flee (goal is an exit tile)"] = IntentRules.kind_of(flee) == "flee"
	checks["switch"] = IntentRules.kind_of(TargetCommand.new(me, foe)) == "switch"
	checks["stunned"] = IntentRules.kind_of(StatusBlockCommand.new(me, "stun")) == "stunned"
	checks["anything else: act"] = IntentRules.kind_of(BattleCommand.new(me)) == "act"
	checks["every kind is listed"] = IntentRules.KINDS.has("act") and IntentRules.KINDS.has(IntentRules.ARRIVING) and IntentRules.KINDS.has("flee")
	return TestHelpers.report(TAG, "IntentRules.kind_of", checks)

# ------------------------------------------------------------------------------
# 3. Records match the scheduled actions at round start; party plan edits change nothing
# ------------------------------------------------------------------------------

static func _test_matches_schedule() -> bool:
	var checks: Dictionary = {}
	var session := CombatSession.new(AiEncounter.new())
	session.start()
	var records: Array[Dictionary] = session.enemy_intents()
	var on_grid: int = 0
	for e: Combatant in session.enemies:
		if e.is_active() and session.grid.has_combatant(e):
			on_grid += 1
	checks["one record per active enemy on the grid"] = records.size() == on_grid and on_grid > 0
	var all_match: bool = true
	var any_scheduled: bool = false
	for r: Dictionary in records:
		var unit: Combatant = r["unit"]
		var first: CombatResolver.ScheduledAction = _first_action(session, unit)
		if first == null:
			all_match = all_match and str(r["kind"]) == IntentRules.ARRIVING
			continue
		any_scheduled = true
		var expected_target: Combatant = null
		if first.command is AttackCommand:
			var attack: AttackCommand = first.command
			expected_target = attack.target
		elif first.command is SupportCommand:
			var support: SupportCommand = first.command
			expected_target = support.target
		elif first.command is WalkCommand:
			var walk: WalkCommand = first.command
			expected_target = walk.approach_target
		all_match = all_match and str(r["kind"]) == IntentRules.kind_of(first.command)
		all_match = all_match and r["target"] == expected_target
		all_match = all_match and r["blocks"] == Vector2i(first.start_block, first.end_block)
		all_match = all_match and str(r["skill"]) == first.command.command_name
	checks["the AI scheduled something"] = any_scheduled
	checks["kind, target, blocks and skill match the first scheduled action"] = all_match
	var before: Array[String] = _sig(records)
	session.plan_apply_preset_all("balanced")
	checks["after drafting the party plan"] = _sig(session.enemy_intents()) == before
	var hero: Combatant = session.allies[0]
	session.plan_clear(hero)
	checks["after clearing a sequence"] = _sig(session.enemy_intents()) == before
	session.plan_apply_preset(hero, "balanced")
	checks["after redrafting a sequence"] = _sig(session.enemy_intents()) == before
	session.plan_undo_last()
	checks["after undoing a step"] = _sig(session.enemy_intents()) == before
	session.plan_apply_preset_all("balanced")
	session.confirm_plan(false)
	checks["empty once the plan is confirmed"] = session.enemy_intents().is_empty() and session.intent_of(session.enemies[0]).is_empty()
	session.dispose()
	return TestHelpers.report(TAG, "enemy_intents matches resolver.scheduled_actions (%d records)" % records.size(), checks)

# ------------------------------------------------------------------------------
# 4. reset_round gives identical intents (round 1 rebuilds the battle; round 2 rewinds it)
# ------------------------------------------------------------------------------

static func _test_reset_round() -> bool:
	var checks: Dictionary = {}
	var session := CombatSession.new(AiEncounter.new())
	session.start()
	var r1: Array[String] = _sig(session.enemy_intents())
	session.plan_apply_preset_all("balanced")
	session.reset_round()
	checks["round 1 reset"] = _sig(session.enemy_intents()) == r1 and not r1.is_empty()
	session.plan_apply_preset_all("balanced")
	session.confirm_plan(false)
	TestHelpers.resolve_until_replay(session)
	checks["round 1 resolved, round 2 offered"] = session.can_advance_round()
	session.advance_round()
	var r2: Array[String] = _sig(session.enemy_intents())
	session.plan_apply_preset_all("balanced")
	session.reset_round()
	checks["round 2 reset"] = session.current_round == 2 and _sig(session.enemy_intents()) == r2 and not r2.is_empty()
	session.dispose()
	return TestHelpers.report(TAG, "reset_round gives identical intents", checks)

# ------------------------------------------------------------------------------
# 5. An Engage shows `move` with the attack target; a wave unit reports `arriving`
# ------------------------------------------------------------------------------

static func _test_engage_and_arriving() -> bool:
	var checks: Dictionary = {}
	var session: CombatSession = _scripted(true)
	var enc: ScriptedEncounter = session.encounter as ScriptedEncounter
	var engage_target: Combatant = null
	var engage_walks: int = 0
	for a: CombatResolver.ScheduledAction in session.resolver.scheduled_actions:
		if a.combatant == enc.hunter and a.command is AttackCommand:
			var attack: AttackCommand = a.command
			engage_target = attack.target
		elif a.combatant == enc.hunter and a.command is WalkCommand:
			engage_walks += 1
	checks["fixture: the Engage scheduled walk(s) and the attack (%s)" % enc.engage_problem] = engage_walks > 0 and engage_target == enc.sella
	var hunt: Dictionary = session.intent_of(enc.hunter)
	checks["engage shows move"] = str(hunt.get("kind", "")) == "move"
	checks["with the attack target as target"] = hunt.get("target", null) == engage_target
	checks["a targeted move has no cell"] = hunt.get("cell", Vector2i.ZERO) == IntentRules.NO_CELL
	var walker: Dictionary = session.intent_of(enc.walker)
	checks["a plain move shows its goal"] = str(walker.get("kind", "")) == "move" and walker.get("target", enc.bryn) == null and walker.get("cell", IntentRules.NO_CELL) == Vector2i(7, 5)
	checks["stunned block"] = str(session.intent_of(enc.dazed).get("kind", "")) == "stunned"
	var bomb: Dictionary = session.intent_of(enc.bomber)
	checks["area: the aimed cell, no target"] = str(bomb.get("kind", "")) == "area" and bomb.get("cell", IntentRules.NO_CELL) == enc.bomber_aim and bomb.get("target", enc.bryn) == null
	checks["an area without a known aim falls back to its first cell"] = session.intent_of(enc.misser).get("cell", IntentRules.NO_CELL) == enc.misser_cells[0]
	checks["fixture: the latecomer is off the grid"] = session.intent_of(enc.latecomer).is_empty() and _count_actions(session, enc.latecomer) == 0
	session.grid.place_combatant(enc.latecomer, Vector2i(10, 1))   # a wave unit enters after the round was scheduled
	var late: Dictionary = session.intent_of(enc.latecomer)
	checks["a wave unit reports arriving"] = str(late.get("kind", "")) == IntentRules.ARRIVING
	checks["arriving has no target, cell, blocks or plan"] = late.get("target", enc.bryn) == null and late.get("cell", Vector2i.ZERO) == IntentRules.NO_CELL and late.get("blocks", Vector2i.ONE) == Vector2i.ZERO and not DictUtil.to_bool(late.get("revealed", true)) and DictUtil.array_of(late.get("plan", ["x"])).is_empty()
	# An enemy on the grid when the round was scheduled but with no action holds (idle), it is not "arriving".
	var held: Combatant = enc.walker
	for a: CombatResolver.ScheduledAction in session.resolver.scheduled_actions:
		if a.combatant == held:
			a.is_cancelled = true
	var hold: Dictionary = session.intent_of(held)
	checks["an on-grid enemy scheduled with no action reports idle"] = str(hold.get("kind", "")) == IntentRules.IDLE
	checks["idle has no target, cell, blocks or plan"] = hold.get("target", enc.bryn) == null and hold.get("cell", Vector2i.ZERO) == IntentRules.NO_CELL and hold.get("blocks", Vector2i.ONE) == Vector2i.ZERO and DictUtil.array_of(hold.get("plan", ["x"])).is_empty()
	checks["the latecomer still reports arriving"] = str(session.intent_of(enc.latecomer).get("kind", "")) == IntentRules.ARRIVING
	checks["idle is a listed kind"] = IntentRules.KINDS.has(IntentRules.IDLE)
	session.dispose()
	return TestHelpers.report(TAG, "engage, area, move cells, idle and arriving", checks)

# ------------------------------------------------------------------------------
# 5b. Packmates do not break each other's drafts (goblin_skirmish round 1)
# ------------------------------------------------------------------------------

static func _test_pack_drafts() -> bool:
	var checks: Dictionary = {}
	var session := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	session.start()
	var active: int = 0
	var acting: int = 0
	var none_arriving: bool = true
	for e: Combatant in session.enemies:
		if e.is_active() and session.grid.has_combatant(e):
			active += 1
			if _count_actions(session, e) > 0:
				acting += 1
			else:
				none_arriving = none_arriving and str(session.intent_of(e).get("kind", "")) != IntentRules.ARRIVING
	checks["fixture: the encounter has several enemies on the grid"] = active >= 2
	checks["every active on-grid enemy has a scheduled action in round 1 (%d/%d)" % [acting, active]] = acting == active
	checks["an enemy without one never reads arriving"] = none_arriving
	session.dispose()
	return TestHelpers.report(TAG, "pack drafts (goblin_skirmish round 1)", checks)

# ------------------------------------------------------------------------------
# 6. Who is revealed (rule 3, Q2 A) and what (rule 4)
# ------------------------------------------------------------------------------

static func _test_reveal() -> bool:
	var checks: Dictionary = {}
	var blind: CombatSession = _scripted(false)
	var none_revealed: bool = true
	for r: Dictionary in blind.enemy_intents():
		none_revealed = none_revealed and not DictUtil.to_bool(r["revealed"]) and DictUtil.array_of(r["plan"]).is_empty() and DictUtil.array_of(r["readers"]).is_empty()
	checks["no foresight holder: every plan is empty"] = none_revealed and not blind.enemy_intents().is_empty()
	blind.dispose()

	var session: CombatSession = _scripted(true)
	var enc: ScriptedEncounter = session.encounter as ScriptedEncounter
	var stalk: Dictionary = session.intent_of(enc.stalker)
	checks["attacking Bryn in block 4 only: revealed"] = DictUtil.to_bool(stalk.get("revealed", false))
	checks["its first block is block 4"] = stalk.get("blocks", Vector2i.ZERO) == Vector2i(4, 5) and stalk.get("target", null) == enc.bryn
	var readers: Array = DictUtil.array_of(stalk.get("readers", []))
	checks["read by Bryn"] = readers.size() == 1 and readers[0] == enc.bryn
	var cut: Dictionary = session.intent_of(enc.cutter)
	checks["attacking only Sella: not revealed"] = not DictUtil.to_bool(cut.get("revealed", true)) and DictUtil.array_of(cut.get("plan", ["x"])).is_empty()
	checks["...but its first-block tell is shown"] = str(cut.get("kind", "")) == "attack" and cut.get("target", null) == enc.sella
	checks["area whose cells cover Bryn's tile: revealed"] = DictUtil.to_bool(session.intent_of(enc.bomber).get("revealed", false))
	checks["area elsewhere: not revealed"] = not DictUtil.to_bool(session.intent_of(enc.misser).get("revealed", true))
	checks["engage on Sella: not revealed"] = not DictUtil.to_bool(session.intent_of(enc.hunter).get("revealed", true))
	var walk: Dictionary = session.intent_of(enc.walker)
	checks["walks first, attacks Bryn later: revealed"] = DictUtil.to_bool(walk.get("revealed", false)) and str(walk.get("kind", "")) == "move"
	var expected: Array[String] = []
	for a: CombatResolver.ScheduledAction in IntentRules.actions_of(enc.walker, session.resolver.scheduled_actions):
		expected.append(_line(IntentRules.action_record(a)))
	var shown: Array[String] = []
	for p: Variant in DictUtil.array_of(walk.get("plan", [])):
		shown.append(_line(DictUtil.dict_of(p)))
	checks["the revealed plan is every action in block order"] = shown == expected and shown.size() == _count_actions(session, enc.walker) and shown.size() == 2
	var plan_blocks: Array[int] = []
	for p: Variant in DictUtil.array_of(walk.get("plan", [])):
		var raw_blocks: Variant = DictUtil.dict_of(p)["blocks"]
		if raw_blocks is Vector2i:
			var b: Vector2i = raw_blocks
			plan_blocks.append(b.x)
	checks["plan lines ascend by start block"] = plan_blocks.size() == 2 and plan_blocks[0] < plan_blocks[1]
	var last: Dictionary = DictUtil.dict_of(DictUtil.array_of(walk.get("plan", [{}])).back())
	checks["plan line keys"] = last.has("kind") and last.has("target") and last.has("cell") and last.has("blocks") and last.has("skill") and last["target"] == enc.bryn
	# An inactive holder reveals nothing (rule 3: the holder must be active).
	enc.bryn.hp = 0
	var downed: Dictionary = IntentRules.intent_of(enc.stalker, session.allies, session.resolver.scheduled_actions, session.grid)
	checks["a downed holder reveals nothing"] = not DictUtil.to_bool(downed["revealed"]) and DictUtil.array_of(downed["plan"]).is_empty()
	session.dispose()
	return TestHelpers.report(TAG, "who is revealed (foresight holder targeted this round)", checks)

# ------------------------------------------------------------------------------
# 7. Display only: a battle resolves identically whether the tells are read or not
# ------------------------------------------------------------------------------

static func _play(read_tells: bool) -> Array[String]:
	var session := CombatSession.new(AiEncounter.new())
	session.start()
	for round_num: int in range(session.encounter.max_rounds):
		if read_tells:
			for r: Dictionary in session.enemy_intents():
				var foe: Combatant = r["unit"]
				session.intent_of(foe)
		session.plan_apply_preset_all("balanced")
		if read_tells:
			session.enemy_intents()
		session.confirm_plan(false)
		TestHelpers.resolve_until_replay(session)
		if session.combat_over or not session.can_advance_round():
			break
		session.advance_round()
	var out: Array[String] = _outcome(session)
	session.dispose()
	return out

static func _test_display_only() -> bool:
	var checks: Dictionary = {}
	var on: Array[String] = _play(true)
	var off: Array[String] = _play(false)
	checks["the battle logged actions"] = on.size() > 6
	checks["identical log and HP with tells read and not read"] = on == off
	return TestHelpers.report(TAG, "display only (results identical with tells on and off)", checks)
