---
class: CombatAutoplay
extends: RefCounted
description: "Tactical RPG - Combat Autoplay (AUTO1-L, docs/specs/AUTO1.md): the party plays itself round after round. Framework-free state + loop next to a CombatSession; any combat layout binds to it (dropdowns, toggle, banner). Tactics   a party tactic (PlanPresets.AUTO_IDS) and per-hero overrides (AUTO_IDS or CUSTOM). A round is drafted with plan_apply_preset_all(party) and then plan_apply_preset(hero, override) for each overridden hero; a CUSTOM hero (\"I plan this hero myself\") is left empty and autoplay waits for the player's Confirm. Custom    a player edit of a row (plan_edited add / remove / move / undo / clear, or mark_edited) shows CUSTOM on that row for this round only; picking a tactic again redrafts the row and drops the edits. Loop      the host calls tick() on the timer that drives CombatSession.auto_play_tick(). While running, a fresh planning round is drafted and confirmed (source \"auto\"; plays at the normal speed through the session's auto_play), and a concluded round advances to the next one. Pause     request_pause() (Space / the toggle): the current round finishes, the next round stops in planning with the draft on the timeline; it stays paused until set_autoplay(true). Safety stops (a hero downed this round, or a hero crossing SAFETY_HP_PCT % HP this round: above it at the start, at or below at the end) and an invalid draft pause the same way with a reason (\"Autoplay paused: Bryn is at 22% HP\"). A boss phase with pause_autoplay (BOSS1 rule 7) pauses at the next planning phase with the reason \"<trigger unit or encounter>: <phase name>\". Remembered  bind a GameState: its `autoplay` blob {on, party, heroes} is read at creation and written on every change (hero keys from CombatEncounter.hero_key). Only player choices change `on`; stops do not."
---

# Properties
* `SAFETY_HP_PCT`: int = 30 (const)
* `EDIT_KINDS`: Array[String] = ["add", "remove", "move", "undo", "clear"] (const)
* `session`: CombatSession = null
* `store`: GameState = null                 # optional: where the choice is remembered (null = this battle only)
* `safety_stops`: bool = true               # spec Q2: On
* `party_tactic`: String = PlanPresets.DEFAULT_TACTIC
* `hero_tactics`: Dictionary = {}           # hero key (String) -> tactic id (AUTO_IDS or CUSTOM), only overrides
* `running`: bool = false                   # the Autoplay switch for this battle
* `remembered_on`: bool = false             # the switch as saved (player choices only)
* `_pause_reason`: String = ""
* `_wait_reason`: String = ""
* `_invalid`: Array[Combatant] = []
* `_edited`: Dictionary = {}                # Combatant -> true: row edited by the player this round (shows CUSTOM)
* `_drafted`: bool = false                  # the live planning round has been drafted by autoplay
* `_stop_next`: bool = false                # paused: advance once and stop the next round in planning, drafted
* `_hp_at_start`: Dictionary = {}           # Combatant -> HP when the round started (stops fire on crossings only)
* `_drafting`: bool = false
* `out`: Array[Dictionary] = []
* `out`: Array[String] = []

# Signals
* `state_changed()`
* `paused(reason: String, by_player: bool)`
* `resumed()`
* `waiting(reason: String)`

# Methods

### `_init(p_session: CombatSession, p_store: GameState = null) -> void`
```logic
session = p_session
store = p_store
if store != null:
	load_settings(store.autoplay)
session.round_started.connect(_on_round_started)
session.round_reset.connect(_on_round_reset)
session.round_concluded.connect(_on_round_concluded)
session.plan_edited.connect(_on_plan_edited)
session.phase_fired.connect(_on_phase_fired)
if session.resolver != null:
	_on_round_started(session.current_round)
```

### `detach() -> void`
```logic
if session == null:
	return
for pair: Array in [[session.round_started, _on_round_started], [session.round_reset, _on_round_reset],
		[session.round_concluded, _on_round_concluded], [session.plan_edited, _on_plan_edited], [session.phase_fired, _on_phase_fired]]:
	var sig: Signal = pair[0]
	var cb: Callable = pair[1]
	if sig.is_connected(cb):
		sig.disconnect(cb)
session = null
```

### `get_party_tactic() -> String`
```logic
return party_tactic
```

### `set_party_tactic(id: String) -> bool`
```logic
if not PlanPresets.AUTO_IDS.has(id):
	return false
party_tactic = id
hero_tactics.clear()
_edited.clear()
if session.is_planning():
	_draft_round()
_changed()
return true
```

### `tactic_of(hero: Combatant) -> String`
```logic
return str(hero_tactics.get(_key(hero), party_tactic))
```

### `row_tactic(hero: Combatant) -> String`
```logic
return PlanPresets.CUSTOM if _edited.has(hero) else tactic_of(hero)
```

### `has_override(hero: Combatant) -> bool`
```logic
return hero_tactics.has(_key(hero))
```

### `set_hero_tactic(hero: Combatant, id: String) -> bool`
```logic
if hero == null or not session.is_ally(hero) or not PlanPresets.is_tactic(id):
	return false
var key: String = _key(hero)
if key.is_empty():
	return false
if id == party_tactic:
	hero_tactics.erase(key)
else:
	hero_tactics[key] = id
_edited.erase(hero)
if session.is_planning() and hero.is_active():
	_drafting = true
	if id == PlanPresets.CUSTOM:
		session.plan_clear(hero)
	else:
		session.plan_apply_preset(hero, id)
	_drafting = false
_changed()
return true
```

### `mark_edited(hero: Combatant) -> void`
```logic
if hero == null or _edited.has(hero) or not session.is_ally(hero):
	return
_edited[hero] = true
_changed()
```

### `is_edited(hero: Combatant) -> bool`
```logic
return _edited.has(hero)
```

### `is_on() -> bool`
```logic
return running
```

### `set_autoplay(on: bool) -> void`
```logic
if on == running:
	return
if not on:
	request_pause()
	return
running = true
remembered_on = true
_pause_reason = ""
_stop_next = false
if session.is_planning() and not _drafted and _has_steps():
	_drafted = true   # turned on over a hand-made plan: keep it and confirm it (no redraft)
_save()
resumed.emit()
_changed()
```

### `toggle() -> void`
```logic
set_autoplay(not running)
```

### `request_pause() -> void`
```logic
if not running:
	return
remembered_on = false
_save()
_pause("Autoplay paused.", true)
```

### `pause_reason() -> String`
```logic
return _pause_reason
```

### `wait_reason() -> String`
```logic
return _wait_reason if running else ""
```

### `tick() -> void`
```logic
if session == null or session.combat_over or session.current_round != session.max_unlocked_round:
	return
if session.phase == CombatSession.Phase.REPLAY:
	if (running or _stop_next) and session.can_advance_round():
		session.advance_round()   # the next tick drafts it
	return
if not session.is_planning():
	return
if not _drafted:
	if not running and not _stop_next:
		return   # manual rounds start empty, as without autoplay
	_draft_round()
	_stop_next = false
	if not _invalid.is_empty():
		var names: Array[String] = []
		for c: Combatant in _invalid:
			names.append(c.name)
		_pause("Autoplay paused: no valid plan for %s." % ", ".join(names), false)
		return
if not running:
	return
var custom: Array[String] = _custom_names()
if not custom.is_empty():
	var reason: String = "Autoplay waits: plan %s yourself, then confirm." % ", ".join(custom)
	if reason != _wait_reason:
		_wait_reason = reason
		waiting.emit(reason)
		_changed()
	return
if not session.plan_is_valid():
	_pause("Autoplay paused: fix the highlighted steps.", false)
	return
session.confirm_plan(true, "auto")
```

### `_draft_round() -> void`
```logic
_drafting = true
_edited.clear()
_invalid.clear()
session.plan_apply_preset_all(party_tactic)
var overridden: Array[Combatant] = []
for a: Combatant in session.allies:
	if a.is_active() and has_override(a):
		overridden.append(a)
		if tactic_of(a) == PlanPresets.CUSTOM:
			session.plan_clear(a)
		else:
			session.plan_apply_preset(a, tactic_of(a))
if not overridden.is_empty():
	for a: Combatant in session.allies:
		if a.is_active() and not session.plan.is_valid(a) and tactic_of(a) != PlanPresets.CUSTOM:
			session.plan_apply_preset(a, session.encounter.party_preset_for(a, tactic_of(a)))   # OBJ1 rule 2: guests Balanced
for a: Combatant in session.allies:
	if a.is_active() and not session.plan.is_valid(a):
		_invalid.append(a)
_drafting = false
_drafted = true
_changed()
```

### `_has_steps() -> bool`
```logic
for a: Combatant in session.allies:
	if not session.plan.steps_of(a).is_empty():
		return true
return false
```

### `_pause(reason: String, by_player: bool) -> void`
```logic
running = false
_pause_reason = reason
_wait_reason = ""
# Stopped while the round plays or after it: the next round stops in planning, drafted.
_stop_next = session.phase != CombatSession.Phase.PLANNING or session.plan_confirmed
paused.emit(reason, by_player)
_changed()
```

### `static safety_reason(heroes: Array[Combatant], hp_at_start: Dictionary, pct: int = SAFETY_HP_PCT) -> String`
```logic
var parts: Array[String] = []
for c: Combatant in heroes:
	var start_hp: int = DictUtil.int_of(hp_at_start, c, 0)
	if start_hp > 0 and not c.is_alive():
		parts.append("%s is down" % c.name)
	elif c.is_active() and c.max_hp > 0 and start_hp * 100 > c.max_hp * pct and c.hp * 100 <= c.max_hp * pct:
		parts.append("%s is at %d%% HP" % [c.name, (c.hp * 100) / c.max_hp])
return "" if parts.is_empty() else "Autoplay paused: %s" % ", ".join(parts)
```

### `settings() -> Dictionary`
```logic
return {"on": remembered_on, "party": party_tactic, "heroes": hero_tactics.duplicate(true)}
```

### `load_settings(blob: Dictionary) -> void`
```logic
var party_raw: String = str(blob.get("party", PlanPresets.DEFAULT_TACTIC))
party_tactic = party_raw if PlanPresets.AUTO_IDS.has(party_raw) else PlanPresets.DEFAULT_TACTIC
hero_tactics.clear()
var heroes: Dictionary = DictUtil.dict_of(blob.get("heroes", {}))
for key: Variant in heroes:
	var id: String = str(heroes[key])
	if PlanPresets.is_tactic(id) and id != party_tactic and not str(key).is_empty():
		hero_tactics[str(key)] = id
remembered_on = DictUtil.to_bool(blob.get("on", false))
running = remembered_on
_pause_reason = ""
_changed()
```

### `_save() -> void`
```logic
if store != null:
	store.autoplay = settings()
```

### `_changed() -> void`
```logic
_save()
state_changed.emit()
```

### `_key(hero: Combatant) -> String`
```logic
return session.encounter.hero_key(hero)
```

### `_on_round_started(_round_num: int) -> void`
```logic
_drafted = false
_edited.clear()
_invalid.clear()
_wait_reason = ""
_hp_at_start.clear()
for a: Combatant in session.allies:
	_hp_at_start[a] = a.hp if a.is_alive() else 0
```

### `_on_round_reset(_round_num: int) -> void`
```logic
_drafted = false
```

### `_on_round_concluded(_round_num: int, over: bool) -> void`
```logic
_wait_reason = ""
if over or not running or not safety_stops:
	return
var reason: String = safety_reason(session.allies, _hp_at_start)
if not reason.is_empty():
	_pause(reason, false)
```

### `_on_phase_fired(_entry: ActionLogEntry, info: Dictionary) -> void`
```logic
if not running or not DictUtil.to_bool(info.get("pause_autoplay", true), true):
	return
_pause(phase_reason(info), false)
_stop_next = true
```

### `static phase_reason(info: Dictionary) -> String`
```logic
return "%s: %s" % [str(info.get("actor", "")), str(info.get("name", ""))]
```

### `_on_plan_edited(kind: String, actor: Combatant) -> void`
```logic
if _drafting or not EDIT_KINDS.has(kind) or actor == null:
	return
mark_edited(actor)
```

