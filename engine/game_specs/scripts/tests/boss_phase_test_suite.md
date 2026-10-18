---
class: BossPhaseTestSuite
extends: RefCounted
description: "BOSS1-L: encounter `phases` (EncounterPhases). Schema: the field and its defaults in both schemas, every validation message of spec sections 1.2 to 1.4 in both catalogs, parsing defaults. Runtime (spec rules 1 to 9): reserved phase waves, hp_below firing mid-round and at the next round start, round phases, drops, reset and replay, autoplay pause, XP / music / preview readers. Fixture refs are picked by rule (side / role / tier) from the catalogs and battles are HP-boosted so nobody falls by accident; writes go to sandbox forks only. Nothing depends on shipped numbers."
---

# Properties
* `TAG`: String = "BossPhase" (const)
* `MAX_ROUNDS`: int = 6 (const)
* `out`: Array[String] = []
* `HP_BOOST`: int = 20 (const)
* `MAX_STEPS`: int = 400 (const)
* `MAX_TICKS`: int = 4000 (const)
* `RUNTIME_ROUNDS`: int = 6 (const)
* `out`: Array[String] = []
* `out`: Array[ActionLogEntry] = []

# Methods

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING BOSS PHASE TEST (BOSS1-L)               ===")
print("=======================================================")
var ok: bool = true
ok = _test_schema_and_parse() and ok
ok = _test_catalog_validation() and ok
ok = _test_world_validation() and ok
ok = _test_shipped_clean() and ok
ok = _test_accepted_and_ignored() and ok
ok = _test_reserved_music() and ok
ok = _test_mid_round() and ok
ok = _test_last_block() and ok
ok = _test_round_phase_wave() and ok
ok = _test_drop_and_win() and ok
ok = _test_reset_and_redo() and ok
ok = _test_autoplay_pause() and ok
ok = _test_recorded_replay() and ok
return ok
```

### `static _catalog_refs() -> Dictionary`
```logic
var party: String = ""
var foes: Array[String] = []
for u: Dictionary in ContentCatalog.entries("units"):
	var side: String = str(u.get("side", ""))
	if side == "party" and party.is_empty():
		party = str(u["id"])
	elif side == "enemy" and foes.size() < 2:
		foes.append(str(u["id"]))
if foes.size() == 1:
	foes.append(foes[0])
if party.is_empty() or foes.is_empty():
	return {}
return {"party": party, "a": foes[0], "b": foes[1]}
```

### `static _world_refs() -> Dictionary`
```logic
for w: String in WorldCatalog.world_ids():
	var party: String = ""
	var foes: Array[String] = []
	for c: Dictionary in WorldCatalog.merged_entries(w, "characters"):
		var role: String = str(c.get("role", ""))
		if role == "party" and party.is_empty():
			party = WorldCatalog.ref_value(w, c)
		elif role == "enemy" and foes.size() < 2:
			foes.append(WorldCatalog.ref_value(w, c))
	if party.is_empty() or foes.is_empty():
		continue
	if foes.size() == 1:
		foes.append(foes[0])
	return {"world": w, "party": party, "a": foes[0], "b": foes[1]}
return {}
```

### `static _ranged_buff() -> String`
```logic
for s: String in StatusEffects.buffs():
	if StatusEffects.has_magnitude(s):
		return s
return ""
```

### `static _plain_buff() -> String`
```logic
for s: String in StatusEffects.buffs():
	if not StatusEffects.has_magnitude(s):
		return s
return ""
```

### `static _debuff() -> String`
```logic
var list: Array[String] = StatusRegistry.ids_where(StatusRegistry.KIND_DEBUFF)
return list[0] if not list.is_empty() else ""
```

### `static _good_phases(refs: Dictionary) -> Array`
```logic
var ranged: String = _ranged_buff()
return [
	{
		"id": "crown", "name": "Crown", "pause_autoplay": true,
		"trigger": {"kind": "hp_below", "unit": refs["a"], "pct": 50},
		"effects": [
			{"type": "line", "text": "The crown falls."},
			{"type": "status", "unit": refs["a"], "status": ranged, "rounds": 3, "magnitude": StatusRegistry.magnitude_min(ranged)},
			{"type": "wave", "units": [refs["b"]], "label": "More come!"},
			{"type": "ai", "unit": refs["b"], "reckless": true, "target_pref": "WEAKEST", "heal_below": 0.9},
		],
	},
	{
		"id": "late_call", "name": "Late Call", "pause_autoplay": false,
		"trigger": {"kind": "round", "round": MAX_ROUNDS},
		"effects": [{"type": "status", "unit": refs["a"], "status": _plain_buff()}],
	},
]
```

### `static _entry(refs: Dictionary, phases: Variant, enemies: Array = []) -> Dictionary`
```logic
return {
	"id": "phase_probe", "name": "Phase Probe", "max_rounds": MAX_ROUNDS,
	"party": [refs["party"]],
	"enemies": enemies if not enemies.is_empty() else [refs["a"]],
	"phases": phases,
}
```

### `static _phase(refs: Dictionary, changes: Dictionary) -> Dictionary`
```logic
var p: Dictionary = {"id": "p", "name": "P", "trigger": {"kind": "hp_below", "unit": refs["a"], "pct": 50}, "effects": [{"type": "line", "text": "Hi."}]}
p.merge(changes, true)
return p
```

### `static _fx(refs: Dictionary, effects: Array) -> Array`
```logic
return [_phase(refs, {"effects": effects})]
```

### `static _bad_cases(refs: Dictionary) -> Dictionary`
```logic
var ranged: String = _ranged_buff()
var plain: String = _plain_buff()
var a: String = str(refs["a"])
var lo: int = StatusRegistry.magnitude_min(ranged)
var hi: int = StatusRegistry.magnitude_max(ranged)
var five: Array = []
for i: int in range(EncounterPhases.MAX_PHASES + 1):
	five.append(_phase(refs, {"id": "p%d" % i}))
var six_fx: Array = []
for i: int in range(EncounterPhases.MAX_EFFECTS + 1):
	six_fx.append({"type": "status", "unit": a, "status": plain})
return {
	"not a list": [{"id": "x"}, "Phases must be a list"],
	"too many phases": [five, "at most %d phases" % EncounterPhases.MAX_PHASES],
	"duplicate id": [[_phase(refs, {}), _phase(refs, {})], "Phase ids must be unique: 'p'"],
	"bad id": [[_phase(refs, {"id": "Bad Id"})], "snake_case"],
	"unknown phase key": [[_phase(refs, {"music": "x"})], "unknown key 'music'"],
	"no name": [[_phase(refs, {"name": ""})], "Phase 'p' needs a name."],
	"long name": [[_phase(refs, {"name": "x".repeat(EncounterPhases.MAX_NAME + 1)})], "at most %d characters" % EncounterPhases.MAX_NAME],
	"pause not bool": [[_phase(refs, {"pause_autoplay": "yes"})], "pause_autoplay must be true or false"],
	"no trigger": [[_phase(refs, {"trigger": null})], "needs a trigger"],
	"unknown trigger": [[_phase(refs, {"trigger": {"kind": "death"}})], "unknown trigger 'death'"],
	"trigger unit not enemy": [[_phase(refs, {"trigger": {"kind": "hp_below", "unit": refs["party"], "pct": 50}})], "trigger unit '%s' is not in enemies" % str(refs["party"])],
	"pct low": [[_phase(refs, {"trigger": {"kind": "hp_below", "unit": a, "pct": 0}})], "pct must be 1..99"],
	"pct high": [[_phase(refs, {"trigger": {"kind": "hp_below", "unit": a, "pct": 100}})], "pct must be 1..99"],
	"round low": [[_phase(refs, {"trigger": {"kind": "round", "round": 1}})], "round must be 2..%d" % MAX_ROUNDS],
	"round high": [[_phase(refs, {"trigger": {"kind": "round", "round": MAX_ROUNDS + 1}})], "round must be 2..%d" % MAX_ROUNDS],
	"no effects": [_fx(refs, []), "needs 1 to %d effects" % EncounterPhases.MAX_EFFECTS],
	"too many effects": [_fx(refs, six_fx), "needs 1 to %d effects" % EncounterPhases.MAX_EFFECTS],
	"two lines": [_fx(refs, [{"type": "line", "text": "a"}, {"type": "line", "text": "b"}]), "one line per phase"],
	"long line": [_fx(refs, [{"type": "line", "text": "x".repeat(EncounterPhases.MAX_LINE + 1)}]), "a line needs 1 to"],
	"two waves": [_fx(refs, [{"type": "wave", "units": [a]}, {"type": "wave", "units": [a]}]), "one wave per phase"],
	"empty wave": [_fx(refs, [{"type": "wave", "units": []}]), "a wave needs at least one unit"],
	"wave unit not enemy": [_fx(refs, [{"type": "wave", "units": [refs["party"]]}]), "wave unit '%s' is not an enemy" % str(refs["party"])],
	"unknown effect": [_fx(refs, [{"type": "boom"}]), "unknown effect 'boom'"],
	"unknown effect key": [_fx(refs, [{"type": "line", "text": "a", "sound": "x"}]), "unknown line key 'sound'"],
	"not a buff": [_fx(refs, [{"type": "status", "unit": a, "status": _debuff()}]), "'%s' is not a buff" % _debuff()],
	"status unit unknown": [_fx(refs, [{"type": "status", "unit": refs["party"], "status": plain}]), "effect unit '%s' is not in this encounter" % str(refs["party"])],
	"rounds low": [_fx(refs, [{"type": "status", "unit": a, "status": plain, "rounds": 0}]), "rounds must be 1..5"],
	"rounds high": [_fx(refs, [{"type": "status", "unit": a, "status": plain, "rounds": 6}]), "rounds must be 1..5"],
	"magnitude zero on a range": [_fx(refs, [{"type": "status", "unit": a, "status": ranged}]), "%s magnitude must be %d..%d" % [ranged, lo, hi]],
	"magnitude over the range": [_fx(refs, [{"type": "status", "unit": a, "status": ranged, "magnitude": hi + 1}]), "%s magnitude must be" % ranged],
	"magnitude without a range": [_fx(refs, [{"type": "status", "unit": a, "status": plain, "magnitude": 3}]), "%s takes no magnitude" % plain],
	"ai without settings": [_fx(refs, [{"type": "ai", "unit": a}]), "an ai effect needs at least one setting"],
	"ai bad role": [_fx(refs, [{"type": "ai", "unit": a, "role": "TANK"}]), "role must be one of"],
	"ai bad target": [_fx(refs, [{"type": "ai", "unit": a, "target_pref": "FARTHEST"}]), "target_pref must be one of"],
	"ai reckless not bool": [_fx(refs, [{"type": "ai", "unit": a, "reckless": "yes"}]), "reckless must be true or false"],
	"ai ratio out of range": [_fx(refs, [{"type": "ai", "unit": a, "flee_ratio": 1.5}]), "flee_ratio must be 0.0..1.0"],
	"ai unit unknown": [_fx(refs, [{"type": "ai", "unit": refs["party"], "reckless": true}]), "effect unit '%s' is not in this encounter" % str(refs["party"])],
}
```

### `static _has(errors: Array[String], needle: String) -> bool`
```logic
return TestHelpers.any_contains(errors, needle)
```

### `static _matrix(refs: Dictionary, validate: Callable) -> Dictionary`
```logic
var checks: Dictionary = {}
var good: Array[String] = validate.call(_entry(refs, _good_phases(refs)))
checks["a valid two-phase entry is clean (%s)" % "; ".join(good)] = good.is_empty()
var none: Array[String] = validate.call(_entry(refs, []))
checks["no phases is clean"] = _phase_errors(none).is_empty()
var bad: Dictionary = _bad_cases(refs)
for label: Variant in bad:
	var pair: Array = bad[label]
	var errors: Array[String] = validate.call(_entry(refs, pair[0]))
	checks["%s -> '%s' (%s)" % [str(label), str(pair[1]), "; ".join(_phase_errors(errors))]] = _has(errors, str(pair[1]))
# The 12-unit cap counts enemies, wave_units and phase waves together.
var many: Array = []
for i: int in range(EncounterPhases.MAX_UNITS):
	many.append(refs["a"])
var at_cap: Array[String] = validate.call(_entry(refs, [], many))
checks["12 enemies without phases fit"] = not _has(at_cap, "at most %d units" % EncounterPhases.MAX_UNITS)
var over: Array[String] = validate.call(_entry(refs, _fx(refs, [{"type": "wave", "units": [refs["b"]]}]), many))
checks["a phase wave past 12 units is refused"] = _has(over, "Encounter enemies, waves and phase waves hold at most %d units." % EncounterPhases.MAX_UNITS)
# A status / ai effect may target a unit that only arrives with a phase wave (of another phase).
var wave_only: Array = [
	_phase(refs, {"id": "w", "effects": [{"type": "wave", "units": [refs["b"]]}]}),
	_phase(refs, {"id": "s", "effects": [{"type": "ai", "unit": refs["b"], "role": "SNIPER"}]}),
]
var wave_ok: Array[String] = validate.call(_entry(refs, wave_only))
checks["effects may target phase-wave units (%s)" % "; ".join(_phase_errors(wave_ok))] = _phase_errors(wave_ok).is_empty()
return checks
```

### `static _test_schema_and_parse() -> bool`
```logic
var checks: Dictionary = {}
var cf: Dictionary = ContentSchema.field("encounters", "phases")
var wf: Dictionary = WorldSchema.field("encounters", "phases")
checks["catalog field phases is json, default []"] = str(cf.get("type", "")) == "json" and cf.get("default", null) is Array and DictUtil.array_of(cf.get("default", null)).is_empty()
checks["world field phases is json, default []"] = str(wf.get("type", "")) == "json" and wf.get("default", null) is Array and DictUtil.array_of(wf.get("default", null)).is_empty()
checks["normalised entries without phases get []"] = DictUtil.array_of(ContentSchema.normalize("encounters", {"id": "x"}).get("phases", null)).is_empty() and ContentSchema.normalize("encounters", {"id": "x"})["phases"] is Array
var parsed: Array[Dictionary] = EncounterPhases.parse([{"id": "p", "name": "P", "trigger": {"kind": "round", "round": 3.0},
	"effects": [{"type": "status", "unit": "u", "status": "haste"}, {"type": "wave", "units": ["w1", "w2"]}, {"type": "ai", "unit": "u", "flee_ratio": 0}]}])
var phase: Dictionary = parsed[0] if not parsed.is_empty() else {}
var fx: Array = DictUtil.array_of(phase.get("effects", []))
checks["parse: pause_autoplay defaults to true"] = DictUtil.to_bool(phase.get("pause_autoplay", false)) == true
checks["parse: trigger round as int"] = DictUtil.dict_of(phase.get("trigger", {})).get("round", null) is int and DictUtil.to_int(DictUtil.dict_of(phase["trigger"])["round"]) == 3
var status_fx: Dictionary = DictUtil.dict_of(fx[0]) if fx.size() > 0 else {}
checks["parse: status rounds 2, magnitude 0"] = DictUtil.to_int(status_fx.get("rounds", -1)) == EncounterPhases.DEFAULT_STATUS_ROUNDS and DictUtil.to_int(status_fx.get("magnitude", -1)) == 0
var wave_fx: Dictionary = DictUtil.dict_of(fx[1]) if fx.size() > 1 else {}
checks["parse: wave label defaults to ''"] = wave_fx.get("label", null) is String and str(wave_fx["label"]).is_empty()
var ai_fx: Dictionary = DictUtil.dict_of(fx[2]) if fx.size() > 2 else {}
checks["parse: ai keeps only given settings, ratios as float"] = ai_fx.get("flee_ratio", null) is float and not ai_fx.has("role")
checks["reserved_refs lists every phase-wave unit"] = EncounterPhases.reserved_refs({"phases": [phase, phase]}) == (["w1", "w2", "w1", "w2"] as Array[String])
return TestHelpers.report(TAG, "Schema field (json, default []) in both schemas; parse fills defaults; reserved refs", checks)
```

### `static _test_catalog_validation() -> bool`
```logic
var refs: Dictionary = _catalog_refs()
if refs.is_empty():
	return TestHelpers.report_ok(TAG, "Catalog validation", false, "no party / enemy units in the catalog")
var checks: Dictionary = _matrix(refs, func(e: Dictionary) -> Array[String]: return ContentCatalog.validate_entry("encounters", e))
return TestHelpers.report(TAG, "Catalog encounters: every phase message fires on a bad entry; a two-phase entry is clean; 12-unit cap", checks)
```

### `static _test_world_validation() -> bool`
```logic
var refs: Dictionary = _world_refs()
if refs.is_empty():
	return TestHelpers.report_ok(TAG, "World validation", false, "no world with party and enemy characters")
var world: String = str(refs["world"])
var checks: Dictionary = _matrix(refs, func(e: Dictionary) -> Array[String]: return WorldCatalog.validate_entry(world, "encounters", e))
return TestHelpers.report(TAG, "World encounters (%s): every phase message fires on a bad entry; a two-phase entry is clean; 12-unit cap" % world, checks)
```

### `static _test_shipped_clean() -> bool`
```logic
var checks: Dictionary = {}
var any_ref: Callable = func(_r: String) -> bool: return true
for e: Dictionary in ContentCatalog.entries("encounters"):
	var errs: Array[String] = EncounterPhases.problems(ContentSchema.normalize("encounters", e), any_ref)
	checks["catalog %s (%s)" % [str(e.get("id", "")), "; ".join(errs)]] = errs.is_empty()
for w: String in WorldCatalog.world_ids():
	for e: Dictionary in WorldCatalog.own_entries(w, "encounters"):
		var errs: Array[String] = EncounterPhases.problems(WorldSchema.normalize("encounters", e), any_ref)
		checks["%s:%s (%s)" % [w, str(e.get("id", "")), "; ".join(errs)]] = errs.is_empty()
return TestHelpers.report(TAG, "Shipped encounters pass the phase rules", checks)
```

### `static _test_accepted_and_ignored() -> bool`
```logic
var checks: Dictionary = {}
var refs: Dictionary = _catalog_refs()
var path: String = TestHelpers.sandbox("boss_phase")
if refs.is_empty() or not TestHelpers.fork(path):
	TestHelpers.cleanup(path)
	return TestHelpers.report_ok(TAG, "Accepted and ignored", false, "no fixture refs or fork failed")
var entry: Dictionary = _entry(refs, _good_phases(refs))
var errors: Array[String] = ContentCatalog.upsert("encounters", entry)
checks["catalog upsert accepts phases (%s)" % "; ".join(errors)] = errors.is_empty()
var stored: Dictionary = ContentCatalog.get_entry("encounters", "phase_probe")
checks["catalog keeps the phases"] = DictUtil.array_of(stored.get("phases", [])).size() == 2
var plain: Dictionary = entry.duplicate(true)
plain["phases"] = []
var with_phases := CatalogEncounter.from_entry(stored)
with_phases.build()
var without := CatalogEncounter.from_entry(plain)
without.build()
var reserved_ok: bool = with_phases.enemies.size() == without.enemies.size() + 1 and with_phases.allies.size() == without.allies.size()
for c: Combatant in with_phases.enemies:
	if without.enemies.size() > 0 and with_phases.enemies.find(c) >= without.enemies.size():
		reserved_ok = reserved_ok and with_phases.is_reserved(c) and not with_phases.grid.has_combatant(c)
checks["build spawns the phase wave reserved and off the field"] = reserved_ok
checks["live_enemies skips reserved units"] = with_phases.live_enemies().size() == without.enemies.size()
TestHelpers.cleanup(path)
var wrefs: Dictionary = _world_refs()
var wpath: String = TestHelpers.sandbox("boss_phase_world")
if not wrefs.is_empty() and TestHelpers.fork_worlds(wpath):
	var world: String = str(wrefs["world"])
	var werrors: Array[String] = WorldCatalog.upsert(world, "encounters", _entry(wrefs, _good_phases(wrefs)))
	checks["world upsert accepts phases (%s)" % "; ".join(werrors)] = werrors.is_empty()
	checks["world keeps the phases"] = DictUtil.array_of(WorldCatalog.get_own(world, "encounters", "phase_probe").get("phases", [])).size() == 2
	# Rule 8: XP counts enemies + the phase wave (one unit in _good_phases); rule 9: one preview line per phase.
	checks["enemy_levels adds phase-wave units"] = BattleBridge.enemy_levels(world, "phase_probe").size() == 2
	var preview: String = WorldPreview.describe(world, "encounters", WorldCatalog.get_own(world, "encounters", "phase_probe"))
	checks["preview: one line per phase"] = preview.contains("Phase Crown: ") and preview.contains("Phase Late Call: round %d -> status" % MAX_ROUNDS) and preview.contains("-> line, status, wave, ai")
else:
	checks["world fork with fixture refs"] = false
TestHelpers.cleanup_worlds(wpath)
return TestHelpers.report(TAG, "A valid phases field is stored by both catalogs; the build reserves phase waves; XP and preview read phases", checks)
```

### `static _rt_refs() -> Dictionary`
```logic
var refs: Dictionary = _catalog_refs()
if refs.is_empty() or str(refs["a"]) == str(refs["b"]):
	return {}
return refs
```

### `static _rt_entry(refs: Dictionary, phases: Array, enemies: Array, id: String = "phase_rt") -> Dictionary`
```logic
return {
	"id": id, "name": "Phase Runtime", "max_rounds": RUNTIME_ROUNDS, "total_blocks": 6,
	"party": [refs["party"]], "enemies": enemies, "phases": phases,
}
```

### `static _session(entry: Dictionary) -> CombatSession`
```logic
var s := CombatSession.new(CatalogEncounter.from_entry(entry))
s.start()
for c: Combatant in s.allies + s.enemies:
	c.max_hp *= HP_BOOST
	c.hp = c.max_hp
return s
```

### `static _enc(s: CombatSession) -> CatalogEncounter`
```logic
return s.encounter as CatalogEncounter
```

### `static _confirm(s: CombatSession) -> bool`
```logic
s.plan_apply_preset_all("balanced")
return s.confirm_plan(false)
```

### `static _run_block(s: CombatSession) -> void`
```logic
var b: int = s.current_block
var r: int = s.current_round
for i: int in range(MAX_STEPS):
	s.step_next()
	if s.phase == CombatSession.Phase.REPLAY or s.current_block != b or s.current_round != r:
		return
```

### `static _play_round(s: CombatSession) -> bool`
```logic
if not _confirm(s):
	return false
TestHelpers.resolve_until_replay(s, MAX_STEPS)
return s.phase == CombatSession.Phase.REPLAY
```

### `static _withdraws(kinds: Array[String]) -> bool`
```logic
return kinds.has("retreat") or kinds.has("flee")
```

### `static _fired_at(s: CombatSession, id: String) -> Array`
```logic
return DictUtil.array_of(_enc(s).phase_runtime.fired.get(id, []))
```

### `static _count_signals(s: CombatSession) -> Array`
```logic
var got: Array = []
s.phase_fired.connect(func(_e: ActionLogEntry, info: Dictionary) -> void: got.append(info))
return got
```

### `static _flee_fx(ref: String) -> Dictionary`
```logic
return {"type": "ai", "unit": ref, "flee_ratio": 1.0, "reckless": false}
```

### `static _test_reserved_music() -> bool`
```logic
var checks: Dictionary = {}
var refs: Dictionary = _rt_refs()
var boss: String = ""
var plain: String = ""
for u: Dictionary in ContentCatalog.entries("units"):
	if str(u.get("side", "")) != "enemy":
		continue
	if str(u.get("tier", "")) == "boss" and boss.is_empty():
		boss = str(u["id"])
	elif str(u.get("tier", "")) != "boss" and plain.is_empty():
		plain = str(u["id"])
if refs.is_empty() or boss.is_empty() or plain.is_empty():
	return TestHelpers.report_ok(TAG, "Music reads phase waves", false, "no boss-tier and plain enemy units")
var phases: Array = [{"id": "w", "name": "W", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "wave", "units": [boss]}]}]
var with_wave := CatalogEncounter.from_entry(_rt_entry(refs, phases, [plain]))
var without := CatalogEncounter.from_entry(_rt_entry(refs, [], [plain]))
checks["phase-wave boss -> boss context"] = MusicDirector.combat_context(with_wave) == MusicDirector.CTX_BOSS
checks["no phases -> combat context"] = MusicDirector.combat_context(without) == MusicDirector.CTX_COMBAT
return TestHelpers.report(TAG, "Music: a boss-tier unit in a phase wave picks the boss context", checks)
```

### `static _test_mid_round() -> bool`
```logic
var checks: Dictionary = {}
var refs: Dictionary = _rt_refs()
if refs.is_empty():
	return TestHelpers.report_ok(TAG, "Mid-round phase", false, "fewer than two enemy units")
var ranged: String = _ranged_buff()
var a_ref: String = str(refs["a"])
var b_ref: String = str(refs["b"])
var phases: Array = [{"id": "rage", "name": "Rage", "trigger": {"kind": "hp_below", "unit": a_ref, "pct": 50},
	"effects": [{"type": "line", "text": "Rage!"}, {"type": "status", "unit": a_ref, "status": ranged, "rounds": 2, "magnitude": StatusRegistry.magnitude_min(ranged)}, _flee_fx(b_ref)]}]
var s: CombatSession = _session(_rt_entry(refs, phases, [a_ref, b_ref]))
var got: Array = _count_signals(s)
var a: Combatant = _enc(s).unit(a_ref, 0)
var b: Combatant = _enc(s).unit(b_ref, 0)
var base_flee: float = _enc(s).profile_of(b).flee_ratio
a.hp = a.max_hp / 2
checks["plan confirmed"] = _confirm(s)
var round1_kinds: Array[String] = _kinds(s, b)
_run_block(s)
checks["due after block 1, not fired yet"] = _enc(s).phase_runtime.due.has("rage") and _fired_at(s, "rage").is_empty()
s.step_next()
var last: ActionLogEntry = s.log_entries.back() if not s.log_entries.is_empty() else null
checks["block 2 starts with the phase EVENT"] = last != null and last.skill_subtype == EncounterPhases.SUBTYPE and last.skill_name == "Rage" and last.block_num == 2 and last.round_num == 1 and last.detail_text == "Rage!" and last.actor_name == a.name
checks["fired at [1, 2]"] = _fired_at(s, "rage") == [1, 2]
checks["buff active in block 2"] = a.statuses.has(ranged) and str(a.status_sources.get(ranged, "")) == "phase:rage"
checks["one phase_fired signal with the line"] = got.size() == 1 and str(DictUtil.dict_of(got[0]).get("line", "")) == "Rage!"
checks["AI unchanged this round (profile and plan)"] = is_equal_approx(_enc(s).profile_of(b).flee_ratio, base_flee) and _kinds(s, b) == round1_kinds and not _withdraws(round1_kinds)
TestHelpers.resolve_until_replay(s, MAX_STEPS)
s.advance_round()
checks["AI change in round 2's draft"] = is_equal_approx(_enc(s).profile_of(b).flee_ratio, 1.0) and _withdraws(_kinds(s, b))
s.dispose()
return TestHelpers.report(TAG, "hp_below crossed in block 1 fires before block 2; buff live at once; AI from the next round", checks)
```

### `static _test_last_block() -> bool`
```logic
var checks: Dictionary = {}
var refs: Dictionary = _rt_refs()
if refs.is_empty():
	return TestHelpers.report_ok(TAG, "Last-block phase", false, "fewer than two enemy units")
var a_ref: String = str(refs["a"])
var b_ref: String = str(refs["b"])
var phases: Array = [{"id": "late", "name": "Late", "trigger": {"kind": "hp_below", "unit": a_ref, "pct": 50},
	"effects": [{"type": "line", "text": "Late!"}, _flee_fx(b_ref)]}]
var s: CombatSession = _session(_rt_entry(refs, phases, [a_ref, b_ref]))
var got: Array = _count_signals(s)
var a: Combatant = _enc(s).unit(a_ref, 0)
var b: Combatant = _enc(s).unit(b_ref, 0)
checks["plan confirmed"] = _confirm(s)
for i: int in range(s.total_blocks - 1):
	_run_block(s)
checks["still in round 1 before the last block"] = s.current_round == 1 and s.current_block == s.total_blocks and _fired_at(s, "late").is_empty()
a.hp = (a.max_hp * 2) / 5
_run_block(s)
checks["round 1 concluded with the phase due, not fired"] = s.phase == CombatSession.Phase.REPLAY and _enc(s).phase_runtime.due.has("late") and _phase_entries(s, 1).is_empty()
s.advance_round()
var r2: Array[ActionLogEntry] = _phase_entries(s, 2)
checks["fired at the round 2 start [2, 0]"] = _fired_at(s, "late") == [2, 0] and r2.size() == 1 and r2[0].block_num == 0
checks["signal once"] = got.size() == 1
checks["round 2 draft already uses the change"] = is_equal_approx(_enc(s).profile_of(b).flee_ratio, 1.0) and _withdraws(_kinds(s, b))
s.dispose()
return TestHelpers.report(TAG, "A crossing at the end of the last block fires at the next round start, before the enemy draft", checks)
```

### `static _test_round_phase_wave() -> bool`
```logic
var checks: Dictionary = {}
var refs: Dictionary = _rt_refs()
if refs.is_empty():
	return TestHelpers.report_ok(TAG, "Round phase", false, "fewer than two enemy units")
var a_ref: String = str(refs["a"])
var b_ref: String = str(refs["b"])
var phases: Array = [{"id": "horn", "name": "Horn", "trigger": {"kind": "round", "round": 3}, "effects": [{"type": "line", "text": "Horns!"}, {"type": "wave", "units": [b_ref]}]}]
var s: CombatSession = _session(_rt_entry(refs, phases, [a_ref]))
var b: Combatant = _enc(s).unit(b_ref, 0)
checks["wave unit spawned, reserved, off the field"] = b != null and s.enemies.has(b) and _enc(s).is_reserved(b) and not s.grid.has_combatant(b) and not _enc(s).live_enemies().has(b)
var played: bool = _play_round(s)
s.advance_round()
played = _play_round(s) and played
checks["rounds 1-2 played, still reserved, no tells"] = played and _enc(s).is_reserved(b) and not s.grid.has_combatant(b)
s.advance_round()
var r3: Array[ActionLogEntry] = _phase_entries(s, 3)
checks["phase in round 3's intro"] = r3.size() == 1 and r3[0].block_num == 0 and r3[0].actor_name == _enc(s).encounter_name and _fired_at(s, "horn") == [3, 0]
checks["wave placed and no longer reserved"] = b != null and s.grid.has_combatant(b) and not _enc(s).is_reserved(b)
checks["wave units have scheduled actions in round 3"] = b != null and not _kinds(s, b).is_empty()
s.dispose()
return TestHelpers.report(TAG, "A round 3 phase fires in round 3's intro and its wave is drafted that round", checks)
```

### `static _test_drop_and_win() -> bool`
```logic
var checks: Dictionary = {}
var refs: Dictionary = _rt_refs()
if refs.is_empty():
	return TestHelpers.report_ok(TAG, "Drop", false, "fewer than two enemy units")
var a_ref: String = str(refs["a"])
var b_ref: String = str(refs["b"])
var phases: Array = [{"id": "rage", "name": "Rage", "trigger": {"kind": "hp_below", "unit": a_ref, "pct": 50}, "effects": [{"type": "wave", "units": [b_ref]}]}]
var s: CombatSession = _session(_rt_entry(refs, phases, [a_ref]))
var a: Combatant = _enc(s).unit(a_ref, 0)
var b: Combatant = _enc(s).unit(b_ref, 0)
a.hp = (a.max_hp * 3) / 5
checks["plan confirmed"] = _confirm(s)
s.step_next()   # block 1 starts and its first action resolves
a.hp = 0        # ...and the trigger unit falls inside the block
TestHelpers.resolve_until_replay(s, MAX_STEPS)
checks["phase dropped, never fired"] = _enc(s).phase_runtime.is_dropped("rage") and _fired_at(s, "rage").is_empty() and _phase_entries(s).is_empty()
checks["battle over and won"] = s.combat_over and BattleBridge.outcome(s) == "won"
checks["wave never placed, still reserved"] = b != null and not s.grid.has_combatant(b) and _enc(s).is_reserved(b) and b.is_alive()
s.dispose()
return TestHelpers.report(TAG, "A trigger unit downed before its phase drops it; the battle is won without the reserved wave", checks)
```

### `static _test_reset_and_redo() -> bool`
```logic
var checks: Dictionary = {}
var refs: Dictionary = _rt_refs()
if refs.is_empty():
	return TestHelpers.report_ok(TAG, "Reset", false, "fewer than two enemy units")
var ranged: String = _ranged_buff()
var a_ref: String = str(refs["a"])
var b_ref: String = str(refs["b"])
var phases: Array = [
	{"id": "dawn", "name": "Dawn", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "line", "text": "Dawn."}, _flee_fx(b_ref)]},
	{"id": "rage", "name": "Rage", "trigger": {"kind": "hp_below", "unit": a_ref, "pct": 50},
		"effects": [{"type": "status", "unit": a_ref, "status": ranged, "rounds": 2, "magnitude": StatusRegistry.magnitude_min(ranged)}, {"type": "ai", "unit": a_ref, "flee_ratio": 1.0, "reckless": false}]},
]
var s: CombatSession = _session(_rt_entry(refs, phases, [a_ref, b_ref]))
var got: Array = _count_signals(s)
var a: Combatant = _enc(s).unit(a_ref, 0)
var b: Combatant = _enc(s).unit(b_ref, 0)
var a_base_flee: float = _enc(s).profile_of(a).flee_ratio
checks["round 1 played"] = _play_round(s)
s.advance_round()
checks["dawn fired at the round 2 start"] = _fired_at(s, "dawn") == [2, 0] and got.size() == 1 and is_equal_approx(_enc(s).profile_of(b).flee_ratio, 1.0)
a.hp = a.max_hp / 2
checks["round 2 plan confirmed"] = _confirm(s)
_run_block(s)
s.step_next()
var first: Array = _fired_at(s, "rage")
checks["rage fired mid-round 2"] = first.size() == 2 and DictUtil.to_int(first[0]) == 2 and a.statuses.has(ranged) and got.size() == 2
s.reset_round()
checks["after reset: rage rewound (not fired, not due)"] = _fired_at(s, "rage").is_empty() and not _enc(s).phase_runtime.due.has("rage")
checks["after reset: its buff is gone"] = not a.statuses.has(ranged)
checks["after reset: profiles back to the round-start ones"] = is_equal_approx(_enc(s).profile_of(a).flee_ratio, a_base_flee) and is_equal_approx(_enc(s).profile_of(b).flee_ratio, 1.0)
checks["after reset: dawn shown once, still fired, no new signal"] = _phase_entries(s, 2).size() == 1 and _fired_at(s, "dawn") == [2, 0] and got.size() == 2
a.hp = a.max_hp / 2
checks["redo plan confirmed"] = _confirm(s)
_run_block(s)
s.step_next()
checks["the same plan fires rage again at the same block"] = _fired_at(s, "rage") == first and a.statuses.has(ranged)
s.dispose()
return TestHelpers.report(TAG, "reset_round rewinds a mid-round phase and the redo fires it again; a round-start phase shows once", checks)
```

### `static _test_autoplay_pause() -> bool`
```logic
var checks: Dictionary = {}
var refs: Dictionary = _rt_refs()
if refs.is_empty():
	return TestHelpers.report_ok(TAG, "Autoplay", false, "fewer than two enemy units")
for pause: bool in [true, false]:
	var phases: Array = [{"id": "dawn", "name": "Dawn", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "line", "text": "Dawn."}], "pause_autoplay": pause}]
	var s: CombatSession = _session(_rt_entry(refs, phases, [refs["a"]]))
	var auto := CombatAutoplay.new(s)
	auto.set_autoplay(true)
	var target_round: int = 2 if pause else 3
	var reached: bool = false
	for i: int in range(MAX_TICKS):
		if s.combat_over or (s.current_round >= target_round and s.is_planning() and (not pause or not auto.is_on())):
			reached = true
			break
		auto.tick()
		s.auto_play_tick()
	if pause:
		checks["pause: stopped in round 2 planning"] = reached and s.current_round == 2 and s.is_planning() and not auto.is_on()
		checks["pause: reason '<encounter>: Dawn' (%s)" % auto.pause_reason()] = auto.pause_reason() == "%s: Dawn" % _enc(s).encounter_name
		checks["pause: saved switch stays on"] = auto.remembered_on
		for i: int in range(20):
			auto.tick()
		checks["pause: the round is drafted, not confirmed"] = s.is_planning() and not s.plan_confirmed and s.current_round == 2
	else:
		checks["no pause: still running in round 3"] = reached and s.current_round >= 3 and auto.is_on()
	auto.detach()
	s.dispose()
return TestHelpers.report(TAG, "Autoplay pauses after a pause_autoplay phase and keeps running without it", checks)
```

### `static _test_recorded_replay() -> bool`
```logic
var checks: Dictionary = {}
var refs: Dictionary = _rt_refs()
var path: String = TestHelpers.sandbox("boss_phase_rec")
if refs.is_empty() or not TestHelpers.fork(path):
	TestHelpers.cleanup(path)
	return TestHelpers.report_ok(TAG, "Recorded replay", false, "no fixture refs or fork failed")
var ranged: String = _ranged_buff()
var a_ref: String = str(refs["a"])
var b_ref: String = str(refs["b"])
var phases: Array = [
	{"id": "hurt", "name": "Hurt", "trigger": {"kind": "hp_below", "unit": a_ref, "pct": 99},
		"effects": [{"type": "line", "text": "Hurt."}, {"type": "status", "unit": a_ref, "status": ranged, "rounds": 2, "magnitude": StatusRegistry.magnitude_min(ranged)}, {"type": "ai", "unit": a_ref, "reckless": true}]},
	{"id": "horn", "name": "Horn", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "wave", "units": [b_ref]}]},
]
var errors: Array[String] = ContentCatalog.upsert("encounters", _rt_entry(refs, phases, [a_ref, b_ref], "phase_rec"))
checks["fixture stored (%s)" % "; ".join(errors)] = errors.is_empty()
var saved_enabled: int = PlaytestStore.force_enabled
PlaytestStore.force_enabled = 1
PlaytestStore.use_root(path.path_join("rec"))
DirAccess.make_dir_recursive_absolute(path.path_join("rec"))
var rec: PlaytestRecorder = PlaytestRecorder.open("user", "", "", path)
var s := CombatSession.new(EncounterSource.create("phase_rec"))
rec.attach(s)
s.start()
for r: int in range(3):
	if not _play_round(s) or s.combat_over or not s.can_advance_round():
		break
	s.advance_round()
checks["the recording fired phases"] = not _phase_entries(s).is_empty()
rec.flush()
rec.detach()
s.dispose()
var data: Dictionary = PlaytestRecord.normalize(DictUtil.dict_of(PlaytestStore.load_json(rec.path).get("data", {})))
var rp: PlaytestReplayer = PlaytestReplayer.for_battle(PlaytestRecord.battle_at(data, 0))
checks["replay starts"] = rp.run()
checks["every round matches (%s)" % str(rp.first_problem().get("detail", ""))] = not rp.rounds.is_empty() and rp.all_match()
if rp.session != null:
	rp.session.dispose()
PlaytestStore.force_enabled = saved_enabled
PlaytestStore.use_root(PlaytestStore.USER_ROOT)
TestHelpers.cleanup(path)
return TestHelpers.report(TAG, "A recorded battle with phases replays with identical round outcomes", checks)
```

