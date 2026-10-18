---
class: AnimationLifecycleTestSuite
extends: RefCounted
description: "Tactical RPG - Animation Lifecycle Test Suite (S14, spec S14-L2) CombatPresenter + FxPresets + FxSettings: the preset table and entry mapping, instant mode end states, tween/temp-node ownership and kill_all, speed scaling, the overload rule and fx_played order. Fixtures: bare Control tokens under a holder in the arena tree; tweens are advanced with Tween.custom_step (the suites run synchronously, no frames pass), so \"freed after one frame\" is checked as queued for deletion. FxSettings is restored at the end."
---

# Properties
* `TAG`: String = "FX" (const)
* `TILE`: int = CombatGridView.TILE_SIZE (const)
* `move`: Variant
* `withdraw`: Variant
* `heal`: Variant
* `haste`: Variant
* `defeated_event`: Variant
* `ambush`: Variant
* `out`: Array[String] = []
* `raw`: Variant = row.get("presets", [])
* `list`: Array = raw

# Methods

### `static _report(label: String, checks: Dictionary) -> bool`
```logic
return TestHelpers.report(TAG, label, checks)
```

### `static run(tree_parent: Node = null) -> bool`
```logic
print("\n=======================================================")
print("=== STARTING ANIMATION LIFECYCLE TEST (S14-L2)       ===")
print("=======================================================")
var saved_instant: bool = FxSettings.instant
var saved_speed: float = FxSettings.speed
var ok: bool = true
ok = _test_table() and ok
ok = _test_mapping() and ok
var holder := Control.new()
if tree_parent != null:
	tree_parent.add_child(holder)
var presenter := CombatPresenter.new()
holder.add_child(presenter)
ok = _test_instant(presenter, holder) and ok
if tree_parent != null:
	ok = _test_kill(presenter, holder) and ok
	ok = _test_speed(presenter, holder) and ok
	ok = _test_overload(presenter, holder) and ok
	ok = _test_settings(presenter, holder) and ok
ok = _test_signal(presenter, holder) and ok
FxSettings.instant = saved_instant
FxSettings.speed = saved_speed
presenter.kill_all()
if holder.is_inside_tree():
	holder.queue_free()
else:
	holder.free()
print("  [FX] Animation lifecycle suite: %s" % ("PASS" if ok else "FAIL"))
return ok
```

### `static _token(holder: Control, cell: Vector2i) -> Control`
```logic
var t := Control.new()
t.custom_minimum_size = Vector2(TILE, TILE)
t.position = Vector2(cell * TILE)
holder.add_child(t)
return t
```

### `static _entry(category: String, outcome: String = "", damage: int = 0, value: int = 0) -> ActionLogEntry`
```logic
var e := ActionLogEntry.new(1, 1, category, "Hero", "Foe", "", value, outcome)
e.damage = damage
return e
```

### `static _status_name(id: String) -> String`
```logic
return StatusEffects.display_name(id).to_upper()
```

### `static _row_entry(row: Dictionary) -> ActionLogEntry`
```logic
var raw: Variant = row.get("entry")
return raw if raw is ActionLogEntry else null
```

### `static _test_table() -> bool`
```logic
var checks: Dictionary = {}
for row: Dictionary in _rows():
	for id: String in FxPresets.for_entry(_row_entry(row), DictUtil.to_int(row.get("distance", -1))):
		checks["%s preset %s exists" % [row.get("name"), id]] = FxPresets.PRESETS.has(id)
for id: Variant in FxPresets.PRESETS:
	var durations: Array[float] = FxPresets.durations(str(id))
	checks["%s has durations" % id] = not durations.is_empty() and FxPresets.tween_count(str(id)) > 0
	for d: float in durations:
		if d <= 0.0:
			checks["%s durations > 0" % id] = false
return _report("Preset table: mapped ids exist, durations and tween counts > 0", checks)
```

### `static _test_mapping() -> bool`
```logic
var checks: Dictionary = {}
for row: Dictionary in _rows():
	checks[str(row.get("name"))] = FxPresets.for_entry(_row_entry(row), DictUtil.to_int(row.get("distance", -1))) == _row_presets(row)
# The combat_ui_framework fixtures animate what the old _on_fx did: lunge + impact, pulse, pulse, move.
var old_attack := ActionLogEntry.new(1, 2, "ATTACK", "Warrior", "Enemy Brawler", "-25 HP", 25, "HIT")
var old_defend := ActionLogEntry.new(1, 1, "DEFEND", "Warrior", "", "50%", 50, "DEFEND")
var old_buff := ActionLogEntry.new(1, 1, "BUFF", "Warrior", "", "+50%", 50, "FOCUS")
var old_move := ActionLogEntry.new(1, 1, "MOVE", "Warrior", "", "-> (4, 4)", 0, "MOVED", Vector2i(4, 5), Vector2i(4, 4))
checks["old ATTACK = lunge + impact"] = FxPresets.for_entry(old_attack) == (["attack_melee", "hit"] as Array[String])
checks["old DEFEND = pulse + its label"] = FxPresets.for_entry(old_defend) == (["defend", "float_text"] as Array[String])
checks["old BUFF = pulse + its label"] = FxPresets.for_entry(old_buff) == (["buff", "float_text"] as Array[String])
checks["old MOVE = move"] = FxPresets.for_entry(old_move) == (["move"] as Array[String])
checks["labels keep the old text"] = FxPresets.float_text(old_defend) == "🛡 DEFEND [50%]" and FxPresets.float_text(old_buff) == "✦ FOCUS"
checks["free text names no status"] = FxPresets.status_id_of(_entry("BUFF", "+50% ATK")).is_empty()
checks["unknown coords count as melee"] = FxPresets.for_entry(_entry("ATTACK", "HIT", 5, 5))[0] == "attack_melee"
return _report("for_entry follows the mapping table and the old _on_fx animations", checks)
```

### `static _test_instant(presenter: CombatPresenter, holder: Control) -> bool`
```logic
FxSettings.instant = true
var checks: Dictionary = {}
for row: Dictionary in _rows():
	var actor: Control = _token(holder, Vector2i(0, 0))
	var target: Control = _token(holder, Vector2i(0, 1) if DictUtil.to_int(row.get("distance", -1)) <= 1 else Vector2i(0, 3))
	actor.modulate = Color(0.2, 0.2, 0.2)
	target.modulate = Color(0.2, 0.2, 0.2)
	var entry: ActionLogEntry = _row_entry(row)
	var target_px: Vector2 = target.position
	if entry.category == "MOVE":
		target_px = Vector2(entry.coord_to * TILE)
	presenter.play(entry, actor, target, target_px)
	var label: String = str(row.get("name"))
	checks["%s: no live tween or temp node" % label] = presenter.live_count() == 0 and presenter.temp_count() == 0
	var presets: Array[String] = _row_presets(row)
	if presets.has("move"):
		checks["%s: at target_px" % label] = actor.position == target_px
	if presets.has("defeat"):
		checks["%s: alpha 0" % label] = is_zero_approx(target.modulate.a)
	if presets.has("hit") or presets.has("mitigated") or presets.has("heal") or presets.has("buff"):
		checks["%s: target modulate white" % label] = target.modulate == Color.WHITE
	if presets.has("defend") or presets.has("attack_ranged"):
		checks["%s: actor modulate white" % label] = actor.modulate == Color.WHITE
	if presets.has("attack_melee"):
		checks["%s: actor back in place" % label] = actor.position == Vector2.ZERO
return _report("Instant mode: end states, no tweens, no temporary nodes", checks)
```

### `static _test_kill(presenter: CombatPresenter, holder: Control) -> bool`
```logic
FxSettings.instant = false
var checks: Dictionary = {}
var actor: Control = _token(holder, Vector2i(2, 2))
var target: Control = _token(holder, Vector2i(2, 3))
presenter.play(_entry("ATTACK", "HIT", 20, 20), actor, target, target.position)
checks["tweens live"] = presenter.live_count() > 0
checks["one label"] = presenter.temp_count() == 1
var label: Node = null
for child: Node in presenter.get_children():
	if child is Label:
		label = child
checks["label under the presenter"] = label != null
var tweens: Array[Tween] = presenter.live_tweens()
var fired: int = presenter.callbacks_fired
presenter.kill_all()
checks["counts 0 after kill_all"] = presenter.live_count() == 0 and presenter.temp_count() == 0
checks["label freed"] = label == null or not is_instance_valid(label) or label.is_queued_for_deletion()
checks["actor and target at end state"] = actor.position == Vector2(2, 2) * TILE and target.position == Vector2(2, 3) * TILE and target.modulate == Color.WHITE
for i: int in 60:   # 0.6 s
	for tw: Tween in tweens:
		if is_instance_valid(tw):
			tw.custom_step(0.01)
checks["no callback after kill_all"] = presenter.callbacks_fired == fired
# Control: an unkilled pulse finishes, fires its callback and leaves the live list.
presenter.play(_entry("DEFEND", "50% MITIGATION", 0, 50), actor, null, Vector2.ZERO)
for tw: Tween in presenter.live_tweens():
	tw.custom_step(1.0)   # the label: 0.2 s delay + 0.55 s fade
checks["finished tweens leave the list"] = presenter.live_count() == 0 and presenter.temp_count() == 0 and presenter.callbacks_fired > fired
presenter.kill_all()
return _report("kill_all: tweens and labels end, no late callbacks", checks)
```

### `static _test_speed(presenter: CombatPresenter, holder: Control) -> bool`
```logic
FxSettings.instant = false
FxSettings.set_speed(2.0)
var checks: Dictionary = {}
var token: Control = _token(holder, Vector2i(5, 5))
var to_px := Vector2(Vector2i(5, 7) * TILE)
presenter.slide(token, to_px)
var tweens: Array[Tween] = presenter.live_tweens()
checks["one move tween"] = tweens.size() == 1
if tweens.size() == 1:
	tweens[0].custom_step(0.12)
	checks["still running at 0.12 s"] = presenter.live_count() == 1 and token.position != to_px
	tweens[0].custom_step(0.01)
	checks["done by 0.13 s"] = presenter.live_count() == 0 and token.position == to_px
FxSettings.set_speed(0.1)
checks["speed snaps to the lowest level"] = is_equal_approx(FxSettings.speed, FxSettings.SPEED_MIN)
FxSettings.set_speed(1.0)
presenter.kill_all()
return _report("Speed scales every duration", checks)
```

### `static _test_overload(presenter: CombatPresenter, holder: Control) -> bool`
```logic
FxSettings.instant = false
var peak: int = 0
var dropped: bool = false
for i: int in FxPresets.MAX_LIVE + 1:
	var before: int = presenter.live_count()
	presenter.play(_entry("DEFEND", "50% MITIGATION", 0, 50), _token(holder, Vector2i(i % 9, 8)), null, Vector2.ZERO)
	dropped = dropped or presenter.live_count() < before
	peak = maxi(peak, presenter.live_count())
var checks: Dictionary = {"at most MAX_LIVE": peak <= FxPresets.MAX_LIVE, "backlog killed": dropped}
presenter.kill_all()
return _report("Overload: a fast replay kills the backlog", checks)
```

### `static _test_signal(presenter: CombatPresenter, holder: Control) -> bool`
```logic
FxSettings.instant = true
var played: Array[String] = []
var record := func(id: String, _e: ActionLogEntry) -> void: played.append(id)
presenter.fx_played.connect(record)
var entry: ActionLogEntry = _entry("ATTACK", "MITIGATED", 6, 6)
var actor: Control = _token(holder, Vector2i(1, 1))
var target: Control = _token(holder, Vector2i(1, 4))
presenter.play(entry, actor, target, target.position)
presenter.fx_played.disconnect(record)
var checks: Dictionary = {"order": played == FxPresets.for_entry(entry, 3), "ranged": played == (["attack_ranged", "mitigated", "float_text"] as Array[String])}
# No target on a non-MOVE entry: the pixel distance is not used (log coordinates decide, unknown = melee).
played.clear()
presenter.fx_played.connect(record)
presenter.play(entry, actor, null, Vector2(TILE * 8, TILE * 8))
presenter.fx_played.disconnect(record)
var logged: Array[String] = FxPresets.for_entry(entry, -1)   # melee by log coordinates; target-side presets skipped with no target
logged.erase("mitigated")
checks["null target uses log coordinates"] = played == logged
# A null target: target-side presets do not run, so they are not reported (float_text needs no node).
played.clear()
presenter.fx_played.connect(record)
presenter.play(entry, actor, null, actor.position)
presenter.fx_played.disconnect(record)
var ran: Array[String] = ["attack_melee", "float_text"]
checks["null target reports only presets that ran"] = played == ran and not played.has("mitigated")
# MISS resolves to the miss preset (not hit), with "Miss" text and a silent sound until an asset exists.
var miss: ActionLogEntry = _entry("ATTACK", "MISS")
var miss_presets: Array[String] = FxPresets.for_entry(miss, 3)
checks["miss entry plays miss, not hit"] = miss_presets.has("miss") and not miss_presets.has("hit") and not miss_presets.has("mitigated")
checks["miss text"] = FxPresets.float_text(miss) == "Miss"
checks["miss preset in sound map"] = SoundEvents.FOR_PRESET.get("miss", "") == "miss" and SoundEvents.KEYS.has("miss")
played.clear()
presenter.fx_played.connect(record)
presenter.play(miss, actor, null, actor.position)
presenter.fx_played.disconnect(record)
var miss_ran: Array[String] = ["attack_melee", "float_text"]
checks["miss with null target: only presets that ran"] = played == miss_ran
return _report("fx_played fires once per preset in order", checks)
```

### `static _test_settings(presenter: CombatPresenter, holder: Control) -> bool`
```logic
var saved_instant: bool = FxSettings.instant
var saved_speed: float = FxSettings.speed
var cfg := ConfigFile.new()
cfg.set_value("anim", "speed", 2)
cfg.set_value("anim", "instant", false)
FxSettings.load_from(cfg)
var headless: bool = DisplayServer.get_name() == "headless"
var checks: Dictionary = {
	"speed read": is_equal_approx(FxSettings.speed, 2.0),
	"instant stays on headless": (not headless) or FxSettings.instant,
}
cfg.set_value("anim", "instant", true)
FxSettings.load_from(cfg)
checks["instant true is kept"] = FxSettings.instant
# Startup path (scene runner): the saved file is pushed to FxSettings without pressing Apply.
var dir: String = TestHelpers.sandbox("anim_settings")
DirAccess.make_dir_recursive_absolute(dir)
var saved := ConfigFile.new()
saved.set_value("anim", "speed", 1.5)
saved.save(dir + "/settings.cfg")
SettingsFile.use_path(dir + "/settings.cfg")
FxSettings.set_speed(1.0)
SettingsModel.push_to_listeners()
checks["startup load applies saved speed"] = is_equal_approx(FxSettings.speed, 1.5)
SettingsFile.use_path()
TestHelpers.remove_sandbox(dir)
FxSettings.instant = false
var node := Control.new()
holder.add_child(node)
presenter.slide(node, Vector2(TILE, TILE))
node.free()
var other: Control = _token(holder, Vector2i(3, 3))
presenter.slide(other, Vector2(TILE * 4, TILE * 4))
checks["freed node's run pruned"] = presenter.live_count() == 1
presenter.kill_all()
FxSettings.instant = saved_instant
FxSettings.speed = saved_speed
return _report("FxSettings.load_from: headless instant, speed, pruning", checks)
```

