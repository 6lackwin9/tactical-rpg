class_name ItemUseBattleChecks
extends RefCounted

# ==============================================================================
# Tactical RPG - Item use battle checks (ITEMUSE1 stage 2, run by ItemUseTestSuite)
# Options and counts, ItemCommand resolution (heal, flat bomb, elixir), cancelled uses, undo / round reset, the codec
# pouch check, TacticalAI use_items, the sim pouch key, GameEncounter write-back and the GearModel pouch draft.
# Hand-built units and items (Consumable.new): every expectation comes from the item's own numbers and the rules.
# ==============================================================================

const SimArgs = preload("res://tools/sim/sim_args.gd")
const SimRunner = preload("res://tools/sim/sim_runner.gd")
const TAG: String = "ItemUse"
const MISSING: int = 30   # HP the hero starts below max (more than nothing, less than its max)

# Downs its victim when it resolves (a deterministic "the target fell before the item's block").
class DownCommand:
	extends BattleCommand
	var victim: Combatant = null

	func _init(p_source: Combatant, p_victim: Combatant) -> void:
		super(p_source, 5, 1)
		victim = p_victim
		command_name = "Down"

	func execute() -> void:
		if victim.is_alive():
			var _dealt: int = victim.take_damage(victim.hp, true)

# Hero and an adjacent mate against an idle foe at the bomb's reach and one a tile further. Options toggle a guarding,
# armored, Defensive near foe and a DownCommand on the mate.
class ItemEncounter:
	extends CombatEncounter

	var templates: Array[Consumable] = []   # the hero's pouch (fresh copies on every build)
	var reach: int = 3
	var start_hp_ratio: float = -1.0        # >= 0: the hero starts at this share of max HP (else max - MISSING)
	var guard_near: bool = false
	var down_mate_block: int = 0
	var hero: Combatant = null
	var mate: Combatant = null
	var near_foe: Combatant = null
	var far_foe: Combatant = null

	func build() -> void:
		max_rounds = 3
		total_blocks = 6
		grid = CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
		hero = TestHelpers.unit("Hero", grid, Vector2i(2, 1))
		mate = TestHelpers.unit("Mate", grid, Vector2i(1, 1))
		near_foe = TestHelpers.unit("Near", grid, Vector2i(2 + reach, 1))
		far_foe = TestHelpers.unit("Far", grid, Vector2i(2, 2 + reach))
		hero.hp = roundi(start_hp_ratio * float(hero.max_hp)) if start_hp_ratio >= 0.0 else hero.max_hp - MISSING
		for t: Consumable in templates:
			hero.inventory.add_to_battle_pouch(ItemUseBattleChecks.copy_of(t))
		if guard_near:
			near_foe.armor_override = 50
			near_foe.current_stance = CombatTypes.StanceType.DEFENSIVE
		allies = [hero, mate]
		enemies = [near_foe, far_foe]
		ai_profiles.clear()
		for c: Combatant in allies + enemies:
			ai_profiles[c] = AiProfile.create(AiProfile.Role.AGGRESSOR, 0.0, 3)

	func schedule_round(resolver: CombatResolver, _round_num: int) -> void:
		if guard_near:
			resolver.schedule_action(near_foe, DefendCommand.new(near_foe, 0.5, total_blocks, 1), 1)
		if down_mate_block > 0:
			resolver.schedule_action(near_foe, DownCommand.new(near_foe, mate), down_mate_block)

static func potion(blocks: int = 1) -> Consumable:
	return Consumable.new("t_iu_potion", "Healing Draught", Consumable.ConsumableType.HEALTH_POTION, 25, 0, "", 0.0, 0, blocks)

static func bomb() -> Consumable:
	return Consumable.new("t_iu_bomb", "Fire Bomb", Consumable.ConsumableType.THROWABLE_BOMB, 0, 20, "", 0.0, 0, 1)

static func elixir() -> Consumable:
	return Consumable.new("t_iu_elixir", "Elixir", Consumable.ConsumableType.FOCUS_INCENSE, 0, 0, "STR", 0.5, 2, 1)

static func copy_of(t: Consumable) -> Consumable:
	var c := Consumable.new(t.id, t.name, t.consumable_type, t.heal_amount, t.damage_amount, t.buff_stat, t.buff_percent, t.buff_duration_rounds, t.block_cost)
	c.use_reach = t.use_reach
	return c

static func session_with(items: Array[Consumable], setup: Callable = Callable()) -> CombatSession:
	var enc := ItemEncounter.new()
	enc.templates = items
	enc.reach = bomb().reach()
	if setup.is_valid():
		setup.call(enc)
	var session := CombatSession.new(enc)
	session.start()
	return session

static func enc_of(session: CombatSession) -> ItemEncounter:
	return session.encounter as ItemEncounter

static func _resolve(session: CombatSession) -> bool:
	if not session.confirm_plan(false):
		return false
	TestHelpers.resolve_until_replay(session)
	return session.phase == CombatSession.Phase.REPLAY

static func _item_options(session: CombatSession, c: Combatant, target: Combatant) -> Array[PlanOption]:
	var out: Array[PlanOption] = []
	for o: PlanOption in session.plan.target_options(c, target):
		if o.step.mode == PlanStep.Mode.ITEM:
			out.append(o)
	return out

static func run() -> bool:
	var ok: bool = true
	ok = _test_options() and ok
	ok = _test_potion_and_reset() and ok
	ok = _test_bomb() and ok
	ok = _test_cancelled() and ok
	ok = _test_elixir() and ok
	ok = _test_codec() and ok
	ok = _test_ai() and ok
	ok = _test_sim_key() and ok
	ok = _test_write_back() and ok
	ok = _test_gear_model() and ok
	ok = _test_record_replay() and ok
	return ok

# Playtest: a recording stores each ally's battle pouch at battle start; the replayer rebuilds units with it, so a
# recorded item step replays identically; an emptied recorded pouch breaks the step (the build's pouch is not used);
# a recording without the key (older) replays as before.
static func _test_record_replay() -> bool:
	var c: Dictionary = {}
	var dir: String = TestHelpers.sandbox("item_use_playtest")
	TestHelpers.remove_sandbox(dir)
	DirAccess.make_dir_recursive_absolute(dir)
	var saved_enabled: int = PlaytestStore.force_enabled
	PlaytestStore.force_enabled = 1
	PlaytestStore.use_root(dir.path_join("rec"))
	# Fixture by rule: the first catalog encounter with an ally carrying an offered self-usable item.
	var enc_id: String = ""
	var item_id: String = ""
	var hero_index: int = -1
	for id: String in ContentCatalog.ids("encounters"):
		var probe := CombatSession.new(EncounterSource.create(id))
		probe.start()
		for i: int in range(probe.allies.size()):
			for pid: String in ItemCommand.pouch_ids(probe.allies[i]):
				var con: Consumable = ItemCommand.pouch_item(probe.allies[i], pid)
				if hero_index < 0 and ItemCommand.offered(con) and not ItemCommand.aims_at_foes(con) and probe.allies[i].is_active():
					hero_index = i
					item_id = pid
		probe.dispose()
		if hero_index >= 0:
			enc_id = id
			break
	c["fixture encounter"] = not enc_id.is_empty()
	if enc_id.is_empty():
		PlaytestStore.force_enabled = saved_enabled
		PlaytestStore.use_root(PlaytestStore.USER_ROOT)
		return TestHelpers.report(TAG, "playtest pouch record / replay", c)
	var rec: PlaytestRecorder = PlaytestRecorder.open("user")
	var session := CombatSession.new(EncounterSource.create(enc_id))
	rec.attach(session)
	session.start()
	var hero: Combatant = session.allies[hero_index]
	session.plan_apply_preset_all("balanced")
	session.plan.set_steps(hero, [PlanStep.use_item(item_id, hero)] as Array[PlanStep])
	var before: int = ItemCommand.pouch_count(hero, item_id)
	c["round with an item step resolves"] = _resolve(session)
	c["item used"] = ItemCommand.pouch_count(hero, item_id) == before - 1
	rec.flush()
	rec.detach()
	session.dispose()
	var data: Dictionary = PlaytestRecord.normalize(DictUtil.dict_of(PlaytestStore.load_json(rec.path).get("data", {})))
	var b: Dictionary = PlaytestRecord.battle_at(data, 0)
	var pouches: Array = DictUtil.array_of(b.get("pouches", []))
	var recorded_items: Array[String] = []
	if hero_index < pouches.size():
		recorded_items = JsonUtil.strings(DictUtil.dict_of(pouches[hero_index]).get("items", []))
	c["pouches recorded at battle start"] = not pouches.is_empty() and recorded_items.count(item_id) == before
	var plans_have_item: bool = false
	for raw_round: Variant in DictUtil.array_of(b.get("rounds", [])):
		for raw_plan: Variant in DictUtil.array_of(DictUtil.dict_of(raw_round).get("plans", [])):
			for raw_step: Variant in DictUtil.array_of(DictUtil.dict_of(raw_plan).get("steps", [])):
				plans_have_item = plans_have_item or str(DictUtil.dict_of(raw_step).get("item", "")) == item_id
	c["item step recorded"] = plans_have_item
	var rp: PlaytestReplayer = PlaytestReplayer.for_battle(b)
	c["replay starts"] = rp.run()
	c["replays identically (%s)" % str(rp.first_problem().get("detail", ""))] = not rp.rounds.is_empty() and rp.all_match() and rp.broken_at < 0
	var emptied: Dictionary = b.duplicate(true)
	var emptied_pouches: Array = DictUtil.array_of(emptied["pouches"])
	DictUtil.dict_of(emptied_pouches[hero_index])["items"] = []
	var rp_empty: PlaytestReplayer = PlaytestReplayer.for_battle(emptied)
	var _started: bool = rp_empty.run()
	c["recorded pouch is the one used (emptied -> step broken)"] = rp_empty.broken_at == 1 and str(rp_empty.first_problem().get("detail", "")).contains("battle pouch")
	var older: Dictionary = b.duplicate(true)
	older.erase("pouches")
	var rp_old: PlaytestReplayer = PlaytestReplayer.for_battle(PlaytestRecord.battle_at(PlaytestRecord.normalize({"battles": [older]}), 0))
	c["older recording (no key) replays as before"] = rp_old.run() and rp_old.all_match()
	PlaytestStore.force_enabled = saved_enabled
	PlaytestStore.use_root(PlaytestStore.USER_ROOT)
	TestHelpers.remove_sandbox(dir)
	return TestHelpers.report(TAG, "playtest pouch record / replay", c)

# Rule 2: one option per distinct item with its count; planned uses count down; plan edits never touch the pouch.
static func _test_options() -> bool:
	var c: Dictionary = {}
	var p: Consumable = potion()
	var session: CombatSession = session_with([p, p] as Array[Consumable])
	var enc: ItemEncounter = enc_of(session)
	var hero: Combatant = enc.hero
	var opts: Array[PlanOption] = _item_options(session, hero, hero)
	c["one option, 2 left"] = opts.size() == 1 and opts[0].valid and opts[0].label == "Use %s (2 left)" % p.name and opts[0].item_left == 2
	c["costs 1 AP and the item's blocks"] = not opts.is_empty() and opts[0].ap == ItemCommand.ITEM_AP and opts[0].blocks == p.block_cost
	c["heal preview capped at missing HP"] = not opts.is_empty() and opts[0].heal == mini(p.heal_amount, MISSING)
	c["first use plans"] = session.plan_add_step(hero, PlanStep.use_item(p.id, hero)).is_empty()
	opts = _item_options(session, hero, hero)
	c["then 1 left"] = opts.size() == 1 and opts[0].label == "Use %s (1 left)" % p.name
	c["second use plans"] = session.plan_add_step(hero, PlanStep.use_item(p.id, hero)).is_empty()
	c["then no option"] = _item_options(session, hero, hero).is_empty()
	c["a third use is refused"] = not session.plan_add_step(hero, PlanStep.use_item(p.id, hero)).is_empty()
	c["planning never touches the pouch"] = hero.inventory.battle_pouch.size() == 2
	session.focused_hero = hero
	session.plan_undo_last()
	session.plan_remove_step(hero, 0)
	c["undo / remove never touch the pouch"] = hero.inventory.battle_pouch.size() == 2 and session.plan.steps_of(hero).is_empty()
	c["potion offered on the adjacent mate"] = _item_options(session, hero, enc.mate).size() == 1
	c["potion not offered on a foe"] = _item_options(session, hero, enc.near_foe).is_empty()
	session.dispose()
	return TestHelpers.report(TAG, "item options and counts", c)

# Rules 4-6: a potion heals (capped) in its block, the pouch drops by one; reset_round gives the item and the HP back.
static func _test_potion_and_reset() -> bool:
	var c: Dictionary = {}
	var p: Consumable = potion()
	var session: CombatSession = session_with([p, p] as Array[Consumable])
	var hero: Combatant = enc_of(session).hero
	c["round 1 idles"] = _resolve(session) and session.can_advance_round()
	session.advance_round()
	var hp_before: int = hero.hp
	c["potion planned"] = session.plan_add_step(hero, PlanStep.use_item(p.id, hero)).is_empty()
	c["resolved"] = _resolve(session)
	c["healed min(heal, missing)"] = hero.hp - hp_before == mini(p.heal_amount, hero.max_hp - hp_before)
	c["one potion used"] = hero.inventory.battle_pouch.size() == 1
	var logged: bool = false
	for e: ActionLogEntry in session.log_entries:
		logged = logged or (e.skill_subtype == "Item" and e.value_num == hero.hp - hp_before)
	c["log entry"] = logged
	session.reset_round()
	c["reset_round gives the potion back"] = hero.inventory.battle_pouch.size() == 2
	c["reset_round gives the HP back"] = hero.hp == hp_before
	session.reset_battle()
	var fresh: Combatant = enc_of(session).hero
	c["reset_battle rebuilds the full pouch"] = fresh.inventory.battle_pouch.size() == 2
	session.dispose()
	return TestHelpers.report(TAG, "potion, reset_round", c)

# Rule 4: a bomb deals its flat damage whatever the foe's armor, guard or stance; one tile beyond its reach is invalid.
static func _test_bomb() -> bool:
	var c: Dictionary = {}
	var b: Consumable = bomb()
	var session: CombatSession = session_with([b] as Array[Consumable], func(e: ItemEncounter) -> void: e.guard_near = true)
	var enc: ItemEncounter = enc_of(session)
	var hero: Combatant = enc.hero
	c["fixture: near foe at the reach"] = session.grid.get_distance(session.grid.get_position(hero), session.grid.get_position(enc.near_foe)) == b.reach()
	c["fixture: far foe one beyond"] = session.grid.get_distance(session.grid.get_position(hero), session.grid.get_position(enc.far_foe)) == b.reach() + 1
	c["out of reach refused"] = session.plan_add_step(hero, PlanStep.use_item(b.id, enc.far_foe)) == "Out of reach for %s." % b.name
	c["no bomb option on the far foe"] = _item_options(session, hero, enc.far_foe).is_empty() or not _item_options(session, hero, enc.far_foe)[0].valid
	c["bomb not offered on self"] = _item_options(session, hero, hero).is_empty()
	var foe_hp: int = enc.near_foe.hp
	c["planned"] = session.plan_add_step(hero, PlanStep.use_item(b.id, enc.near_foe)).is_empty()
	c["resolved"] = _resolve(session)
	c["guarding, armored, Defensive foe"] = enc.near_foe.armor_override > 0 and enc.near_foe.current_stance == CombatTypes.StanceType.DEFENSIVE
	c["exactly the bomb's damage"] = foe_hp - enc.near_foe.hp == b.damage_amount
	c["bomb used up"] = hero.inventory.battle_pouch.is_empty()
	session.dispose()
	return TestHelpers.report(TAG, "bomb flat damage and reach", c)

# Rule 5: a potion whose ally target is downed before its block is not used; the pouch keeps it; the log says why.
static func _test_cancelled() -> bool:
	var c: Dictionary = {}
	var slow: Consumable = potion(2)
	var session: CombatSession = session_with([slow] as Array[Consumable], func(e: ItemEncounter) -> void: e.down_mate_block = 1)
	var enc: ItemEncounter = enc_of(session)
	var messages: Array[String] = []
	session.resolver.log_emitted.connect(func(msg: String) -> void: messages.append(msg))
	c["planned on the mate"] = session.plan_add_step(enc.hero, PlanStep.use_item(slow.id, enc.mate)).is_empty()
	c["resolved"] = _resolve(session)
	c["mate down"] = not enc.mate.is_alive()
	c["pouch keeps it"] = enc.hero.inventory.battle_pouch.size() == 1
	c["log says why"] = TestHelpers.any_contains(messages, "%s not used: %s is down." % [slow.name, enc.mate.name])
	session.dispose()
	return TestHelpers.report(TAG, "cancelled use keeps the item", c)

# Rule 4: the elixir sets the attack boost to 1 + buff for its rounds and replaces an active Focus boost.
static func _test_elixir() -> bool:
	var c: Dictionary = {}
	var e: Consumable = elixir()
	var session: CombatSession = session_with([e] as Array[Consumable])
	var hero: Combatant = enc_of(session).hero
	hero.apply_attack_boost(1.2, 1)   # an active Focus boost
	c["planned on self"] = session.plan_add_step(hero, PlanStep.use_item(e.id, hero)).is_empty()
	c["not offered on the mate (self only)"] = _item_options(session, hero, enc_of(session).mate).is_empty()
	c["resolved"] = _resolve(session)
	c["boost = 1 + buff"] = is_equal_approx(hero.attack_boost_multiplier, 1.0 + e.buff_percent)
	c["rounds = duration"] = hero.attack_boost_rounds_remaining == e.buff_duration_rounds
	var slots: int = 0
	for m: StatModifier in hero.active_buffs:
		if m.source_id == "focus_attack_boost":
			slots += 1
	c["replaces, never stacks"] = slots == 1
	session.dispose()
	return TestHelpers.report(TAG, "elixir replaces the focus boost", c)

# Replay: an ITEM step decodes only when the item is in the actor's pouch.
static func _test_codec() -> bool:
	var c: Dictionary = {}
	var p: Consumable = potion()
	var session: CombatSession = session_with([p] as Array[Consumable])
	var hero: Combatant = enc_of(session).hero
	var roster: Array = PlanCodec.roster(session)
	var encoded: Dictionary = PlanCodec.encode_step(session, PlanStep.use_item(p.id, hero))
	var errors: Array[String] = []
	var warnings: Array[String] = []
	var back: PlanStep = PlanCodec.decode_step(session, hero, encoded, roster, "t", errors, warnings)
	c["round trip"] = back != null and back.mode == PlanStep.Mode.ITEM and back.item_id == p.id and back.target == hero and errors.is_empty()
	encoded["item"] = "t_iu_missing"
	var missing: PlanStep = PlanCodec.decode_step(session, hero, encoded, roster, "t", errors, warnings)
	c["item not in the pouch refused"] = missing == null and TestHelpers.any_contains(errors, "has no 't_iu_missing' in the battle pouch")
	session.dispose()
	return TestHelpers.report(TAG, "codec pouch check", c)

static func _has_item_step(steps: Array[PlanStep]) -> bool:
	for s: PlanStep in steps:
		if s.mode == PlanStep.Mode.ITEM:
			return true
	return false

# Rules 8-9: drafts never add item steps with use_items off; with it on a hero at 30% HP drinks first; autoplay-style
# drafts give the same battle with and without pouches.
static func _test_ai() -> bool:
	var c: Dictionary = {}
	var p: Consumable = potion()
	var low: Callable = func(e: ItemEncounter) -> void: e.start_hp_ratio = 0.3
	var session: CombatSession = session_with([p, p] as Array[Consumable], low)
	var enc: ItemEncounter = enc_of(session)
	var off: Array[PlanStep] = enc.draft_steps(enc.hero, "balanced", session.plan)
	c["use_items off: no item step"] = not _has_item_step(off)
	var profile: AiProfile = enc.ai_profiles[enc.hero]
	profile.use_items = true
	var on: Array[PlanStep] = enc.draft_steps(enc.hero, "balanced", session.plan)
	c["use_items on at 30% HP: drinks first"] = not on.is_empty() and on[0].mode == PlanStep.Mode.ITEM and on[0].item_id == p.id and on[0].target == enc.hero
	c["at most one item step"] = on.filter(func(s: PlanStep) -> bool: return s.mode == PlanStep.Mode.ITEM).size() == 1
	profile.use_items = false
	session.dispose()
	# Same seed, with and without pouches: party-tactic drafts (the autoplay path) play the same battle.
	var results: Array[String] = []
	for items: Array[Consumable] in [[] as Array[Consumable], [p, bomb(), elixir()] as Array[Consumable]]:
		var s: CombatSession = session_with(items)
		var any_item: bool = false
		for r: int in range(s.encounter.max_rounds):
			s.plan_apply_preset_all("balanced")
			for a: Combatant in s.allies:
				any_item = any_item or _has_item_step(s.plan.steps_of(a))
			if not _resolve(s) or s.combat_over or not s.can_advance_round():
				break
			s.advance_round()
		var hps: Array[int] = []
		for u: Combatant in s.allies + s.enemies:
			hps.append(u.hp)
		results.append("%s|%s" % [str(hps), str(any_item)])
		s.dispose()
	c["no item step in party drafts"] = results.size() == 2 and results[1].ends_with("false")
	c["same battle with and without pouches"] = results.size() == 2 and results[0] == results[1]
	return TestHelpers.report(TAG, "AI use_items and autoplay drafts", c)

# Sim key: pouch= parses into consumable ids, the default header is unchanged, apply_pouch fills the party and turns
# use_items on for the party only.
static func _test_sim_key() -> bool:
	var c: Dictionary = {}
	var potion_id: String = ""
	for e: Dictionary in ContentCatalog.entries("items"):
		if str(e.get("item_type", "")) == "CONSUMABLE" and str(e.get("consumable_type", "")) == "HEALTH_POTION":
			potion_id = str(e["id"])
			break
	c["catalog potion found"] = not potion_id.is_empty()
	var plain := SimArgs.new()
	c["no key: no pouch, header without pouch"] = plain.parse(PackedStringArray(["party"])) and plain.pouch.is_empty() and not plain.header().contains("pouch")
	var keyed := SimArgs.new()
	c["pouch= parses"] = keyed.parse(PackedStringArray(["party", "pouch=%s,%s" % [potion_id, potion_id]])) and keyed.pouch == ([potion_id, potion_id] as Array[String])
	var bad := SimArgs.new()
	c["unknown id refused"] = not bad.parse(PackedStringArray(["party", "pouch=t_iu_nothing"])) and bad.error.contains("not a consumable")
	var low: Callable = func(e: ItemEncounter) -> void: e.start_hp_ratio = 0.3
	var session: CombatSession = session_with([] as Array[Consumable], low)
	SimRunner.apply_pouch(session, keyed.pouch)
	var enc: ItemEncounter = enc_of(session)
	c["party pouches filled"] = ItemCommand.pouch_count(enc.hero, potion_id) == 2 and ItemCommand.pouch_count(enc.mate, potion_id) == 2
	var hero_profile: AiProfile = enc.ai_profiles[enc.hero]
	var foe_profile: AiProfile = enc.ai_profiles[enc.near_foe]
	c["party uses items, enemies never"] = hero_profile.use_items and not foe_profile.use_items
	session.plan_apply_preset_all("balanced")
	var steps: Array[PlanStep] = session.plan.steps_of(enc.hero)
	c["the unit at 30% HP drinks"] = not steps.is_empty() and steps[0].mode == PlanStep.Mode.ITEM and steps[0].item_id == potion_id
	session.dispose()
	return TestHelpers.report(TAG, "sim pouch key", c)

# Rule 7: after a battle each member's saved pouch keeps only its unused items (every outcome); guests are never written.
static func _test_write_back() -> bool:
	var c: Dictionary = {}
	var potion_id: String = ""
	for e: Dictionary in ContentCatalog.entries("items"):
		if str(e.get("consumable_type", "")) == "HEALTH_POTION" and str(e.get("item_type", "")) == "CONSUMABLE":
			potion_id = str(e["id"])
			break
	var gs: GameState = GameState.of(NewGame.create_for_world("ashlands", 5)["state"])
	var qid: String = ""
	var guest_qid: String = ""
	for local: String in WorldCatalog.own_ids("ashlands", "encounters"):
		if qid.is_empty():
			qid = WorldCatalog.qid("ashlands", local)
		if guest_qid.is_empty() and not DictUtil.array_of(WorldCatalog.get_own("ashlands", "encounters", local).get("guests", [])).is_empty():
			guest_qid = WorldCatalog.qid("ashlands", local)
	c["fixture"] = gs != null and not potion_id.is_empty() and not qid.is_empty()
	if not DictUtil.to_bool(c["fixture"]):
		return TestHelpers.report(TAG, "write-back", c)
	var m: UnitState = gs.party.active_members()[0]
	for outcome_case: String in ["won", "lost"]:
		m.pouch = [potion_id, potion_id] as Array[String]
		var built: Dictionary = BattleBridge.encounter_for(gs, guest_qid if not guest_qid.is_empty() else qid)
		var raw_enc: Variant = built.get("encounter", null)
		var enc: GameEncounter = raw_enc if raw_enc is GameEncounter else null
		if enc == null:
			c["%s: encounter built" % outcome_case] = false
			continue
		var session := CombatSession.new(enc)
		session.start()
		var hero: Combatant = enc.heroes()[0]
		c["%s: pouch loaded" % outcome_case] = ItemCommand.pouch_count(hero, potion_id) == 2
		var cmd := ItemCommand.new(hero, hero, ItemCommand.pouch_item(hero, potion_id), session.grid)
		cmd.execute()
		if outcome_case == "lost":
			for a: Combatant in enc.heroes():
				var _d: int = a.take_damage(a.hp, true)
		enc.write_back_pouches()
		c["%s: one fewer" % outcome_case] = cmd.used and m.pouch == ([potion_id] as Array[String])
		for g: Combatant in enc.guests:
			c["%s: guest %s has no member" % [outcome_case, g.name]] = enc.hero_key(g) != m.id
		session.dispose()
	c["only party members written"] = gs.party.members.size() == GameState.of(NewGame.create_for_world("ashlands", 5)["state"]).party.members.size()
	return TestHelpers.report(TAG, "write-back (won and lost)", c)

# Rule 1: the Gear tab drafts the pouch from the stash; confirm writes it, cancel restores.
static func _test_gear_model() -> bool:
	var c: Dictionary = {}
	var potion_id: String = ""
	var gear_id: String = ""
	for e: Dictionary in ContentCatalog.entries("items"):
		if potion_id.is_empty() and str(e.get("consumable_type", "")) == "HEALTH_POTION" and str(e.get("item_type", "")) == "CONSUMABLE":
			potion_id = str(e["id"])
		if gear_id.is_empty() and str(e.get("item_type", "")) != "CONSUMABLE":
			gear_id = str(e["id"])
	var gs: GameState = GameState.of(NewGame.create_for_world("ashlands", 6)["state"])
	c["fixture"] = gs != null and not potion_id.is_empty() and not gear_id.is_empty()
	if not DictUtil.to_bool(c["fixture"]):
		return TestHelpers.report(TAG, "gear model pouch draft", c)
	var had_game: bool = CurrentGame.has_game()
	var prior: GameState = CurrentGame.state() if had_game else null
	CurrentGame.start(gs)
	var m: UnitState = gs.party.members[0]
	m.pouch = [] as Array[String]
	gs.party.stash = {potion_id: UnitState.POUCH_MAX + 1, gear_id: 1}
	var gm := GearModel.new(m.id)
	var candidates: Array[String] = []
	for row: Dictionary in gm.pouch_candidates():
		candidates.append(str(row["item"]))
	c["only battle consumables offered"] = candidates.has(potion_id) and not candidates.has(gear_id)
	c["gear refused"] = not gm.pouch_add(gear_id).is_empty()
	for i: int in range(UnitState.POUCH_MAX):
		c["add %d" % (i + 1)] = gm.pouch_add(potion_id).is_empty()
	c["full pouch refused"] = not gm.pouch_add(potion_id).is_empty()
	c["stash drafted down"] = DictUtil.to_int(gm.stash.get(potion_id, 0)) == 1 and DictUtil.to_int(gs.party.stash.get(potion_id, 0)) == UnitState.POUCH_MAX + 1
	c["remove returns one"] = gm.pouch_remove(0).is_empty() and DictUtil.to_int(gm.stash.get(potion_id, 0)) == 2 and gm.pouch_rows().size() == UnitState.POUCH_MAX - 1
	c["member untouched before confirm"] = m.pouch.is_empty() and gm.has_draft()
	c["confirm writes pouch and stash"] = gm.confirm().is_empty() and m.pouch.size() == UnitState.POUCH_MAX - 1 and DictUtil.to_int(gs.party.stash.get(potion_id, 0)) == 2
	var _r: Array[String] = gm.pouch_remove(0)
	gm.cancel()
	c["cancel restores"] = gm.work.pouch.size() == UnitState.POUCH_MAX - 1 and not gm.has_draft()
	CurrentGame.end()
	if prior != null:
		CurrentGame.start(prior)
	return TestHelpers.report(TAG, "gear model pouch draft", c)
