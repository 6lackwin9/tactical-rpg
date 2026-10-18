---
class: AutoplayTestSuite
extends: RefCounted
description: "Tactical RPG - Battle autoplay test suite (AUTO1-L, docs/specs/AUTO1.md; CombatAutoplay) Acceptance checks without UI: Balanced autoplay finishes an easy fight with no input and matches the sim party loop (plan_apply_preset_all + confirm + play + advance) on the same seed, rounds recorded with source \"auto\"; pause semantics (the round finishes, the next stops drafted, the one after is manual); per-hero tactics, the Custom flip and redraft, a deliberate Custom row stopping every round; safety stops at a downed hero and at the 30% line, not otherwise (crossings only: a hero staying under does not stop again, healing re-arms); the tactics and the switch survive save and load (GameState.autoplay, save v7). Units are sandbox rows (sim helpers) of the first buildable classes of a catalog fork; numbers derive from the rules."
---

# Properties
* `SimArgs`: Variant = preload("res://tools/sim/sim_args.gd") (const)
* `SimRunner`: Variant = preload("res://tools/sim/sim_runner.gd") (const)
* `SimScenarios`: Variant = preload("res://tools/sim/sim_scenarios.gd") (const)
* `TAG`: String = "Autoplay" (const)
* `HERO_LEVEL`: int = 5 (const)
* `MAX_TICKS`: int = 6000 (const)
* `HP_BOOST`: int = 10   # long fights for the flow checks: every unit's HP x10 after start (no stop, no early end) (const)
* `out`: Array[String] = []

# Methods

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING AUTOPLAY TEST (AUTO1-L)                 ===")
print("=======================================================")
var ok: bool = _setup()
if ok:
	ok = _test_easy_matches_sim() and ok
	ok = _test_pause() and ok
	ok = _test_tactics_and_custom() and ok
	ok = _test_deliberate_custom() and ok
	ok = _test_safety_reason() and ok
	ok = _test_safety_in_play() and ok
	ok = _test_safety_crossing() and ok
	ok = _test_invalid_draft() and ok
	ok = _test_remembered() and ok
TestHelpers.cleanup(SANDBOX)
return ok
```

### `static _setup() -> bool`
```logic
var checks: Dictionary = {"sandbox fork": TestHelpers.fork(SANDBOX)}
if not DictUtil.to_bool(checks["sandbox fork"]):
	return TestHelpers.report(TAG, "fixtures", checks)
var runner := SimRunner.new(SimArgs.new())
var classes: Array[String] = []
var ids: Array[String] = ContentCatalog.ids("classes")
ids.sort()
for class_id: String in ids:
	var probe: Array[String] = SimScenarios.define_party(runner, [class_id] as Array[String], HERO_LEVEL)
	if runner.build(probe[0]) != null:
		classes.append(class_id)
	if classes.size() == 3:
		break
checks["three buildable classes"] = classes.size() == 3
if classes.size() < 3:
	return TestHelpers.report(TAG, "fixtures", checks)
var party: Array[String] = SimScenarios.define_party(runner, classes, HERO_LEVEL)
var minions: Array[String] = SimScenarios.define_enemies(runner, [[classes[0], "minion"], [classes[0], "minion"]], 1, 1, classes[0], classes[0])
var foes: Array[String] = SimScenarios.define_enemies(runner, [[classes[0], "standard"], [classes[1], "standard"]], HERO_LEVEL, 3, classes[0], classes[0])
var written: bool = true
for id: String in party + minions + foes:
	written = runner.build(id) != null and written
checks["sandbox units written"] = written
var base: Dictionary = {"zone": "stone_arena", "arena": "CLOSED", "total_blocks": 6, "unit_ap": 12, "ap_regen": 4, "party": party}
_easy = base.duplicate(true)
_easy.merge({"id": "t_auto_easy", "name": "Autoplay Easy", "enemies": minions, "max_rounds": 10, "rng_seed": 7}, true)
_even = base.duplicate(true)
_even.merge({"id": "t_auto_even", "name": "Autoplay Even", "enemies": foes, "max_rounds": 12, "rng_seed": 11}, true)
return TestHelpers.report(TAG, "fixtures", checks)
```

### `static _session(entry: Dictionary, boost: bool = false) -> CombatSession`
```logic
var session := CombatSession.new(CatalogEncounter.from_entry(entry))
session.start()
if boost:
	for c: Combatant in session.allies + session.enemies:
		c.max_hp *= HP_BOOST
		c.hp = c.max_hp
return session
```

### `static _run_until(session: CombatSession, auto: CombatAutoplay, done: Callable, max_ticks: int = MAX_TICKS) -> bool`
```logic
for i: int in range(max_ticks):
	if DictUtil.to_bool(done.call()):
		return true
	auto.tick()
	session.auto_play_tick()
return DictUtil.to_bool(done.call())
```

### `static _play_round(session: CombatSession) -> void`
```logic
for i: int in range(MAX_TICKS):
	if session.phase == CombatSession.Phase.REPLAY:
		return
	session.step_next()
```

### `static _all_drafted(session: CombatSession) -> bool`
```logic
for a: Combatant in session.allies:
	if a.is_active() and session.plan.steps_of(a).is_empty():
		return false
return true
```

### `static _signals(auto: CombatAutoplay) -> Dictionary`
```logic
var sig_log: Dictionary = {"paused": [], "resumed": 0, "waiting": []}
auto.paused.connect(func(reason: String, by_player: bool) -> void:
	var list: Array = sig_log["paused"]
	list.append([reason, by_player]))
auto.resumed.connect(func() -> void: sig_log["resumed"] = DictUtil.to_int(sig_log["resumed"]) + 1)
auto.waiting.connect(func(reason: String) -> void:
	var list: Array = sig_log["waiting"]
	list.append(reason))
return sig_log
```

### `static _test_easy_matches_sim() -> bool`
```logic
var checks: Dictionary = {}
var sim: CombatSession = _session(_easy)
for r: int in range(sim.encounter.max_rounds):
	sim.plan_apply_preset_all("balanced")
	if not sim.confirm_plan():
		break
	_play_round(sim)
	if sim.combat_over or not sim.can_advance_round():
		break
	sim.advance_round()
var session := CombatSession.new(CatalogEncounter.from_entry(_easy))
var rec: PlaytestRecorder = PlaytestRecorder.open("agent")
rec.path = ""   # never written
rec.attach(session, "t_auto_easy")
session.start()
var auto := CombatAutoplay.new(session)
var sig_log: Dictionary = _signals(auto)
auto.set_autoplay(true)
var finished: bool = _run_until(session, auto, func() -> bool: return session.combat_over or (session.phase == CombatSession.Phase.REPLAY and not session.can_advance_round()))
checks["finishes with no input"] = finished and session.combat_over
checks["the party wins the easy fight"] = BattleBridge.outcome(session) == "won"
checks["never paused"] = DictUtil.array_of(sig_log["paused"]).is_empty() and auto.is_on()
var same: bool = sim.current_round == session.current_round and sim.combat_over == session.combat_over and sim.log_entries.size() == session.log_entries.size()
for i: int in range(sim.allies.size()):
	same = same and sim.allies[i].hp == session.allies[i].hp
for i: int in range(sim.enemies.size()):
	same = same and sim.enemies[i].hp == session.enemies[i].hp
checks["same result as the sim party preset (rounds, HP, log)"] = same
if not same:
	print("  sim round %d over %s log %d | auto round %d over %s log %d" % [sim.current_round, str(sim.combat_over), sim.log_entries.size(), session.current_round, str(session.combat_over), session.log_entries.size()])
var rounds: Array = DictUtil.array_of(rec.battle().get("rounds", []))
var all_auto: bool = rounds.size() == session.current_round
for raw: Variant in rounds:
	var r: Dictionary = DictUtil.dict_of(raw)
	all_auto = all_auto and str(r.get("source", "")) == "auto" and not DictUtil.array_of(r.get("plans", [])).is_empty()
checks["every round recorded with source auto and its plans"] = all_auto
var bogus: Dictionary = rec.battle().duplicate(true)
DictUtil.dict_of(DictUtil.array_of(bogus["rounds"])[0])["source"] = "robot"
checks["round sources validate (auto ok, unknown refused)"] = not TestHelpers.any_contains(PlaytestRecord.battle_errors(rec.battle()), "unknown source") and TestHelpers.any_contains(PlaytestRecord.battle_errors(bogus), "unknown source 'robot'")
var manual := CombatSession.new(CatalogEncounter.from_entry(_easy))
var rec2: PlaytestRecorder = PlaytestRecorder.open("agent")
rec2.path = ""
rec2.attach(manual, "t_auto_easy")
manual.start()
manual.plan_apply_preset_all("balanced")
manual.confirm_plan()
checks["a player confirm records source player"] = str(rec2.round_entry(1).get("source", "")) == "player" and manual.confirm_source == "player"
rec.detach()
rec2.detach()
auto.detach()
for s: CombatSession in [sim, session, manual]:
	s.dispose()
return TestHelpers.report(TAG, "Balanced autoplay finishes the easy fight like the sim (acceptance 1, rule 10)", checks)
```

### `static _test_pause() -> bool`
```logic
var checks: Dictionary = {}
var session: CombatSession = _session(_even, true)
var auto := CombatAutoplay.new(session)
var sig_log: Dictionary = _signals(auto)
auto.set_autoplay(true)
checks["resumed emitted"] = DictUtil.to_int(sig_log["resumed"]) == 1
var resolving: bool = _run_until(session, auto, func() -> bool: return session.phase == CombatSession.Phase.RESOLVING and session.current_block >= 2)
checks["round 1 resolving on its own"] = resolving and session.current_round == 1 and session.confirm_source == "auto"
auto.request_pause()
checks["pause: off at once, by the player"] = not auto.is_on() and DictUtil.array_of(sig_log["paused"]).size() == 1 and DictUtil.to_bool(DictUtil.array_of(DictUtil.array_of(sig_log["paused"])[0])[1])
var stopped: bool = _run_until(session, auto, func() -> bool: return session.current_round == 2 and session.is_planning() and _all_drafted(session))
checks["the round finishes and the next stops in planning, drafted"] = stopped
for i: int in range(50):
	auto.tick()
	session.auto_play_tick()
checks["it stays stopped (not confirmed)"] = session.is_planning() and session.current_round == 2 and not session.plan_confirmed
checks["the draft is valid"] = session.plan_is_valid()
checks["the switch is remembered off"] = not DictUtil.to_bool(auto.settings()["on"])
checks["Confirm plays it"] = session.confirm_plan(true)
var played: bool = _run_until(session, auto, func() -> bool: return session.phase == CombatSession.Phase.REPLAY)
checks["the player's round plays, recorded as player"] = played and session.confirm_source == "player"
for i: int in range(50):
	auto.tick()
	session.auto_play_tick()
checks["no advance while paused"] = session.current_round == 2 and session.phase == CombatSession.Phase.REPLAY
session.advance_round()
for i: int in range(20):
	auto.tick()
var empty: bool = true
for a: Combatant in session.allies:
	empty = empty and session.plan.steps_of(a).is_empty()
checks["the round after stays manual (empty, unconfirmed)"] = session.current_round == 3 and session.is_planning() and empty
auto.set_autoplay(true)
auto.tick()
checks["turning it on drafts and confirms"] = session.plan_confirmed and session.confirm_source == "auto" and DictUtil.to_int(sig_log["resumed"]) == 2 and auto.pause_reason().is_empty()
auto.detach()
session.dispose()
return TestHelpers.report(TAG, "pause: the round finishes, the next stops drafted, then manual (acceptance 2)", checks)
```

### `static _test_tactics_and_custom() -> bool`
```logic
var checks: Dictionary = {}
var entries: Array[Dictionary] = CombatAutoplay.tactic_entries()
var listed: Array[String] = []
for e: Dictionary in entries:
	listed.append(str(e["id"]))
	checks["%s has a label and a description" % str(e["id"])] = not str(e["label"]).is_empty() and not str(e["description"]).is_empty()
var expected: Array[String] = PlanPresets.AUTO_IDS.duplicate()
expected.append(PlanPresets.CUSTOM)
checks["dropdown: the six presets then Custom, no Fall Back"] = listed == expected and not listed.has("withdraw") and PlanPresets.AUTO_IDS.size() == PlanPresets.IDS.size() - 1
checks["without Custom: the six"] = CombatAutoplay.tactic_entries(false).size() == PlanPresets.AUTO_IDS.size()
var session: CombatSession = _session(_even, true)
var twin: CombatSession = _session(_even, true)
var auto := CombatAutoplay.new(session)
var hero: Combatant = session.allies[0]
var tactic: String = PlanPresets.AUTO_IDS[3]
checks["defaults: Balanced, no override"] = auto.get_party_tactic() == PlanPresets.DEFAULT_TACTIC and auto.tactic_of(hero) == PlanPresets.DEFAULT_TACTIC and not auto.has_override(hero)
checks["Fall Back and unknown ids refused"] = not auto.set_hero_tactic(hero, "withdraw") and not auto.set_party_tactic("withdraw") and not auto.set_party_tactic(PlanPresets.CUSTOM) and not auto.set_hero_tactic(session.enemies[0], tactic)
auto.set_party_tactic(PlanPresets.DEFAULT_TACTIC)   # redrafts the party (autoplay off: a dropdown pick still drafts)
twin.plan_apply_preset_all(PlanPresets.DEFAULT_TACTIC)
checks["the party pick drafts every row"] = _all_drafted(session) and _titles(session, hero) == _titles(twin, twin.allies[0])
auto.set_hero_tactic(hero, tactic)
twin.plan_apply_preset(twin.allies[0], tactic)
checks["a row pick drafts that hero with its own tactic"] = auto.tactic_of(hero) == tactic and auto.row_tactic(hero) == tactic and auto.has_override(hero) and _titles(session, hero) == _titles(twin, twin.allies[0])
session.plan_remove_step(hero, session.plan.steps_of(hero).size() - 1)
checks["an edit flips the row to Custom (the tactic is kept)"] = auto.row_tactic(hero) == PlanPresets.CUSTOM and auto.tactic_of(hero) == tactic and auto.is_edited(hero)
auto.set_hero_tactic(hero, tactic)
checks["picking the tactic again redrafts and drops the edits"] = auto.row_tactic(hero) == tactic and _titles(session, hero) == _titles(twin, twin.allies[0])
auto.mark_edited(session.allies[1])
checks["mark_edited flips a row too"] = auto.row_tactic(session.allies[1]) == PlanPresets.CUSTOM
auto.set_autoplay(true)
auto.tick()
checks["autoplay confirms the edited round as it stands"] = session.plan_confirmed and session.confirm_source == "auto"
var next: bool = _run_until(session, auto, func() -> bool: return session.current_round == 2 and session.plan_confirmed)
checks["the edit lasted one round: the next row is back on its tactic"] = next and auto.row_tactic(session.allies[1]) == auto.tactic_of(session.allies[1]) and auto.row_tactic(hero) == tactic
auto.set_party_tactic(PlanPresets.AUTO_IDS[1])
checks["the party pick sets every hero (overrides cleared)"] = auto.tactic_of(hero) == PlanPresets.AUTO_IDS[1] and not auto.has_override(hero)
auto.set_hero_tactic(hero, PlanPresets.AUTO_IDS[1])
checks["a row set to the party tactic keeps no override"] = not auto.has_override(hero)
auto.detach()
session.dispose()
twin.dispose()
return TestHelpers.report(TAG, "row tactic, Custom flip and redraft (acceptance 3)", checks)
```

### `static _test_deliberate_custom() -> bool`
```logic
var checks: Dictionary = {}
var session: CombatSession = _session(_even, true)
var auto := CombatAutoplay.new(session)
var sig_log: Dictionary = _signals(auto)
var mine: Combatant = session.allies[1]
auto.set_hero_tactic(mine, PlanPresets.CUSTOM)
checks["Custom empties the row at once"] = session.plan.steps_of(mine).is_empty() and auto.row_tactic(mine) == PlanPresets.CUSTOM
auto.set_autoplay(true)
for round_num: int in [1, 2]:
	var waited: bool = _run_until(session, auto, func() -> bool: return session.current_round == round_num and session.is_planning() and not auto.wait_reason().is_empty())
	for i: int in range(30):
		auto.tick()
	var others: bool = true
	for a: Combatant in session.allies:
		if a != mine and a.is_active():
			others = others and not session.plan.steps_of(a).is_empty()
	checks["round %d stops in planning: others drafted, the Custom row empty" % round_num] = waited and not session.plan_confirmed and others and session.plan.steps_of(mine).is_empty()
	checks["round %d: still on, waiting names the hero" % round_num] = auto.is_on() and auto.wait_reason().contains(mine.name) and auto.pause_reason().is_empty()
	checks["round %d: the player's Confirm plays it" % round_num] = session.confirm_plan(true) and session.confirm_source == "player"
checks["waiting emitted once per round"] = DictUtil.array_of(sig_log["waiting"]).size() == 2 and DictUtil.array_of(sig_log["paused"]).is_empty()
auto.detach()
session.dispose()
return TestHelpers.report(TAG, "deliberate Custom stops every round (acceptance 3)", checks)
```

### `static _test_safety_reason() -> bool`
```logic
var checks: Dictionary = {}
var session: CombatSession = _session(_even)
var heroes: Array[Combatant] = session.allies
var a: Combatant = heroes[0]
var b: Combatant = heroes[1]
var full: Dictionary = {}
for c: Combatant in heroes:
	full[c] = c.max_hp
var line: int = (a.max_hp * CombatAutoplay.SAFETY_HP_PCT) / 100
checks["full HP: no stop"] = CombatAutoplay.safety_reason(heroes, full).is_empty()
a.hp = line + 1
checks["one HP above the line: no stop"] = CombatAutoplay.safety_reason(heroes, full).is_empty()
a.hp = line
checks["crossing onto the line: stop with the percentage"] = CombatAutoplay.safety_reason(heroes, full) == "Autoplay paused: %s is at %d%% HP" % [a.name, (line * 100) / a.max_hp]
a.hp = 1
checks["crossing below the line: stop"] = CombatAutoplay.safety_reason(heroes, full) == "Autoplay paused: %s is at %d%% HP" % [a.name, 100 / a.max_hp]
var under: Dictionary = full.duplicate()
under[a] = line
checks["started the round under the line, still under: no stop"] = CombatAutoplay.safety_reason(heroes, under).is_empty()
var just_above: Dictionary = full.duplicate()
just_above[a] = line + 1
a.hp = line
checks["one HP above at the start, on the line at the end: stop"] = not CombatAutoplay.safety_reason(heroes, just_above).is_empty()
var half: Dictionary = full.duplicate()
half[a] = a.max_hp / 2
a.hp = a.max_hp / 5
checks["50% to 20%: stop"] = CombatAutoplay.safety_reason(heroes, half) == "Autoplay paused: %s is at %d%% HP" % [a.name, (a.hp * 100) / a.max_hp]
a.hp = a.max_hp
b.hp = 0
checks["downed this round: stop"] = CombatAutoplay.safety_reason(heroes, full) == "Autoplay paused: %s is down" % b.name
var b_down: Dictionary = full.duplicate()
b_down[b] = 0
checks["downed earlier: no new stop"] = CombatAutoplay.safety_reason(heroes, b_down).is_empty()
var b_low: Dictionary = full.duplicate()
b_low[b] = 1
checks["downed from under the line: stop"] = CombatAutoplay.safety_reason(heroes, b_low) == "Autoplay paused: %s is down" % b.name
a.hp = line
checks["several: listed in party order"] = CombatAutoplay.safety_reason(heroes, full) == "Autoplay paused: %s is at %d%% HP, %s is down" % [a.name, (line * 100) / a.max_hp, b.name]
b.hp = b.max_hp
a.is_withdrawn = true
checks["a withdrawn hero is not a stop"] = CombatAutoplay.safety_reason(heroes, full).is_empty()
session.dispose()
return TestHelpers.report(TAG, "safety_reason: downed and crossing the 30% line only", checks)
```

### `static _scripted(plan: Dictionary) -> Array`
```logic
var session: CombatSession = _session(_even, true)
session.round_concluded.connect(func(r: int, _over: bool) -> void:
	if plan.has(r):
		var c: Combatant = session.allies[0]
		c.hp = (c.max_hp * DictUtil.to_int(plan[r])) / 100)
var auto := CombatAutoplay.new(session)
return [session, auto]
```

### `static _stopped_in(session: CombatSession, auto: CombatAutoplay, round_num: int) -> bool`
```logic
return session.current_round == round_num and not auto.is_on() and session.is_planning() and _all_drafted(session)
```

### `static _test_safety_crossing() -> bool`
```logic
var checks: Dictionary = {}
var pct: int = CombatAutoplay.SAFETY_HP_PCT
var pair: Array = _scripted({1: pct, 2: pct, 3: 100, 4: pct})
var session: CombatSession = pair[0]
var auto: CombatAutoplay = pair[1]
var sig_log: Dictionary = _signals(auto)
auto.set_autoplay(true)
checks["round 1 crosses the line: stop before round 2"] = _run_until(session, auto, func() -> bool: return _stopped_in(session, auto, 2)) and DictUtil.array_of(sig_log["paused"]).size() == 1
auto.set_autoplay(true)
checks["resumed with the hero still under: round 2 ends without a stop"] = _run_until(session, auto, func() -> bool: return session.current_round == 3 and session.plan_confirmed) and DictUtil.array_of(sig_log["paused"]).size() == 1
checks["healed above the line in round 3: no stop"] = _run_until(session, auto, func() -> bool: return session.current_round == 4 and session.plan_confirmed) and DictUtil.array_of(sig_log["paused"]).size() == 1
checks["re-armed: crossing again in round 4 stops"] = _run_until(session, auto, func() -> bool: return _stopped_in(session, auto, 5)) and DictUtil.array_of(sig_log["paused"]).size() == 2
auto.detach()
session.dispose()
var pair2: Array = _scripted({1: 50, 2: 20})
var s2: CombatSession = pair2[0]
var a2: CombatAutoplay = pair2[1]
var log2: Dictionary = _signals(a2)
a2.set_autoplay(true)
checks["50% after round 1: no stop"] = _run_until(s2, a2, func() -> bool: return s2.current_round == 2 and s2.plan_confirmed) and DictUtil.array_of(log2["paused"]).is_empty()
var hero: Combatant = s2.allies[0]
var stopped: bool = _run_until(s2, a2, func() -> bool: return _stopped_in(s2, a2, 3))
var expect: String = "Autoplay paused: %s is at %d%% HP" % [hero.name, (((hero.max_hp * 20) / 100) * 100) / hero.max_hp]
checks["50% to 20% in round 2: stop with the reason"] = stopped and DictUtil.array_of(log2["paused"]).size() == 1 and a2.pause_reason() == expect
a2.detach()
s2.dispose()
return TestHelpers.report(TAG, "safety stop fires on crossing the line only (re-armed by healing)", checks)
```

### `static _safety_run(set_hp: Callable) -> Dictionary`
```logic
var session: CombatSession = _session(_even, true)
var state: Dictionary = {"done": false}
session.round_concluded.connect(func(_r: int, _over: bool) -> void:
	if not DictUtil.to_bool(state["done"]):
		state["done"] = true
		set_hp.call(session.allies[0]))
var auto := CombatAutoplay.new(session)
var sig_log: Dictionary = _signals(auto)
auto.set_autoplay(true)
var reached: bool = _run_until(session, auto, func() -> bool: return session.current_round == 2 and (session.plan_confirmed or (not auto.is_on() and session.is_planning() and _all_drafted(session))))
var out: Dictionary = {"reached": reached, "paused": DictUtil.array_of(sig_log["paused"]).duplicate(true), "on": auto.is_on(), "confirmed": session.plan_confirmed,
	"reason": auto.pause_reason(), "remembered": DictUtil.to_bool(auto.settings()["on"]), "name": session.allies[0].name, "max": session.allies[0].max_hp}
auto.detach()
session.dispose()
return out
```

### `static _test_safety_in_play() -> bool`
```logic
var checks: Dictionary = {}
var low: Dictionary = _safety_run(func(c: Combatant) -> void: c.hp = (c.max_hp * CombatAutoplay.SAFETY_HP_PCT) / 100)
var line: int = (DictUtil.to_int(low["max"]) * CombatAutoplay.SAFETY_HP_PCT) / 100
var expect: String = "Autoplay paused: %s is at %d%% HP" % [str(low["name"]), (line * 100) / DictUtil.to_int(low["max"])]
var low_paused: Array = DictUtil.array_of(low["paused"])
checks["30% line: paused with the reason, not by the player"] = low_paused.size() == 1 and str(DictUtil.array_of(low_paused[0])[0]) == expect and not DictUtil.to_bool(DictUtil.array_of(low_paused[0])[1]) and str(low["reason"]) == expect
checks["30% line: the next round stops drafted"] = DictUtil.to_bool(low["reached"]) and not DictUtil.to_bool(low["confirmed"]) and not DictUtil.to_bool(low["on"])
checks["a safety stop keeps the remembered switch on"] = DictUtil.to_bool(low["remembered"])
var down: Dictionary = _safety_run(func(c: Combatant) -> void: c.hp = 0)
var down_paused: Array = DictUtil.array_of(down["paused"])
checks["downed: paused"] = down_paused.size() == 1 and str(DictUtil.array_of(down_paused[0])[0]) == "Autoplay paused: %s is down" % str(down["name"]) and not DictUtil.to_bool(down["confirmed"])
var fine: Dictionary = _safety_run(func(c: Combatant) -> void: c.hp = (c.max_hp * CombatAutoplay.SAFETY_HP_PCT) / 100 + 1)
checks["above the line: no stop, the next round confirmed"] = DictUtil.to_bool(fine["reached"]) and DictUtil.array_of(fine["paused"]).is_empty() and DictUtil.to_bool(fine["confirmed"]) and DictUtil.to_bool(fine["on"])
return TestHelpers.report(TAG, "safety stops in play (acceptance 4)", checks)
```

### `static _test_invalid_draft() -> bool`
```logic
var checks: Dictionary = {}
var session: CombatSession = _session(_even, true)
var auto := CombatAutoplay.new(session)
var sig_log: Dictionary = _signals(auto)
var hero: Combatant = session.allies[0]
auto.set_autoplay(true)
# Break the hero's draft right after autoplay drafts it: the session's plan_edited "preset_all" fires inside the draft.
var state: Dictionary = {"armed": true}
session.plan_edited.connect(func(kind: String, _actor: Combatant) -> void:
	if kind == "preset_all" and DictUtil.to_bool(state["armed"]):
		state["armed"] = false
		var steps: Array[PlanStep] = session.plan.steps_of(hero).duplicate()
		for i: int in range(3):
			steps.append_array(session.plan.steps_of(hero))
		steps.append(PlanStep.withdraw())
		session.plan.set_steps(hero, steps))
auto.tick()
var paused: Array = DictUtil.array_of(sig_log["paused"])
var broken: bool = not session.plan.is_valid(hero)
checks["fixture: the draft is broken"] = broken
if broken:
	checks["paused on that round, the hero named"] = paused.size() == 1 and str(DictUtil.array_of(paused[0])[0]).contains(hero.name) and auto.invalid_heroes().has(hero) and not session.plan_confirmed and session.is_planning()
auto.detach()
session.dispose()
return TestHelpers.report(TAG, "invalid draft pauses (rule 7)", checks)
```

### `static _new_game() -> GameState`
```logic
for world_id: String in WorldCatalog.world_ids():
	var res: Dictionary = NewGame.create_for_world(world_id, 4242)
	var g: GameState = GameState.of(res.get("state", null))
	if g != null and g.validate().is_empty():
		return g
return null
```

### `static _test_remembered() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _new_game()
checks["fixture: a new game"] = gs != null
if gs == null:
	return TestHelpers.report(TAG, "remembered per save (acceptance 5)", checks)
checks["a new game starts with autoplay off, Balanced"] = JsonUtil.deep_equal(gs.autoplay, GameState.default_autoplay()) and not DictUtil.to_bool(gs.autoplay["on"])
var session: CombatSession = _session(_even, true)
var auto := CombatAutoplay.new(session, gs)
checks["bound off: not running"] = not auto.is_on()
var a: Combatant = session.allies[0]
var b: Combatant = session.allies[1]
auto.set_party_tactic(PlanPresets.AUTO_IDS[5])
auto.set_hero_tactic(a, PlanPresets.AUTO_IDS[3])
auto.set_hero_tactic(b, PlanPresets.CUSTOM)
auto.set_autoplay(true)
checks["changes are written to the game state"] = gs.validate().is_empty() and str(gs.autoplay["party"]) == PlanPresets.AUTO_IDS[5] and DictUtil.to_bool(gs.autoplay["on"])
var text: String = JSON.stringify({"format": SaveGame.FORMAT, "version": SaveGame.VERSION, "saved_at": 1, "game_state": gs.to_dict()})
var res: Dictionary = SaveGame.decode(text)
var loaded: GameState = GameState.of(res["state"])
checks["save and load round-trip"] = DictUtil.to_bool(res["ok"]) and loaded != null and loaded.equals(gs)
auto.detach()
session.dispose()
if loaded != null:
	var again: CombatSession = _session(_even, true)
	var auto2 := CombatAutoplay.new(again, loaded)
	checks["the next battle starts with the same choice"] = auto2.is_on() and auto2.get_party_tactic() == PlanPresets.AUTO_IDS[5] and auto2.tactic_of(again.allies[0]) == PlanPresets.AUTO_IDS[3] and auto2.tactic_of(again.allies[1]) == PlanPresets.CUSTOM
	auto2.request_pause()
	checks["the player's pause is remembered off"] = not DictUtil.to_bool(loaded.autoplay["on"])
	auto2.detach()
	again.dispose()
var v6: Dictionary = gs.to_dict()
v6.erase("autoplay")
var mig: Dictionary = SaveGame.decode(JSON.stringify({"format": SaveGame.FORMAT, "version": 6, "saved_at": 1, "game_state": v6}))
var mig_state: GameState = GameState.of(mig["state"])
checks["v6 migrates to autoplay off"] = DictUtil.to_bool(mig["ok"]) and mig_state != null and JsonUtil.deep_equal(mig_state.autoplay, GameState.default_autoplay()) and SaveGame.VERSION >= 7
var bad: Array[Dictionary] = [
	{"on": "yes", "party": "balanced", "heroes": {}},
	{"on": false, "party": "withdraw", "heroes": {}},
	{"on": false, "party": PlanPresets.CUSTOM, "heroes": {}},
	{"on": false, "party": "balanced", "heroes": {"x": "withdraw"}},
	{"on": false, "party": "balanced"},
]
for blob: Dictionary in bad:
	checks["refused: %s" % JSON.stringify(blob)] = not GameState.autoplay_errors(blob).is_empty()
var raw: Dictionary = gs.to_dict()
raw["autoplay"] = "on"
checks["a non-object autoplay is a load problem"] = TestHelpers.any_contains(GameState.from_dict(raw).validate(), "autoplay must be")
checks["GameEncounter keys heroes by party member id"] = _member_keys_ok(gs)
return TestHelpers.report(TAG, "remembered per save (acceptance 5)", checks)
```

### `static _member_keys_ok(gs: GameState) -> bool`
```logic
var entries: Array[Dictionary] = WorldCatalog.merged_entries(gs.world_id, "encounters")
if entries.is_empty():
	return false
var enc := GameEncounter.new(gs.world_id, str(entries[0].get("id", "")), gs.party)
enc.build()
var keys: Array[String] = []
for c: Combatant in enc.allies:
	keys.append(enc.hero_key(c))
var ids: Array[String] = []
for m: UnitState in gs.party.active_members():
	ids.append(m.id)
keys.sort()
ids.sort()
return not keys.is_empty() and keys == ids
```

