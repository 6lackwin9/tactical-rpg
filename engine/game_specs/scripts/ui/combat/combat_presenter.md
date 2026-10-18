---
class: CombatPresenter
extends: Node
description: "Tactical RPG - Combat Presenter (S14-L2) The one owner of combat animation. CombatGridPanel creates it as a child and calls play() from session.fx_requested and slide() from refresh(). Every tween is created here, kept in _live and dropped on `finished`; temporary nodes (labels, status icons) go under `overlay`, are kept in _temp and freed by their tween's last callback. kill_all() ends everything at its end state; callbacks from before it check a generation counter and do nothing. Instant mode (FxSettings.instant, always on headless) applies end states synchronously. Logic never waits on this."
---

# Properties
* `TILE_SIZE`: int = CombatGridView.TILE_SIZE (const)
* `tile_px`: int = TILE_SIZE   # the grid panel sets the fitted tile size
* `tween`: Tween
* `node`: CanvasItem
* `has_pos`: bool = false
* `end_pos`: Vector2 = Vector2.ZERO
* `has_mod`: bool = false
* `end_mod`: Color = Color.WHITE
* `overlay`: Control = null   # parent of temporary nodes (CombatGridView.fx_layer); the presenter itself when unset
* `callbacks_fired`: int = 0  # tween callbacks that ran (lifecycle suite)
* `_live`: Array[Run] = []
* `_temp`: Array[Node] = []
* `_gen`: int = 0
* `out`: Array[Tween] = []

# Signals
* `fx_played(preset_id: String, entry: ActionLogEntry)`

# Methods

### `_init() -> void`
```logic
name = "CombatPresenter"
```

### `_exit_tree() -> void`
```logic
kill_all()
```

### `live_count() -> int`
```logic
return _live.size()
```

### `temp_count() -> int`
```logic
return _temp.size()
```

### `play(entry: ActionLogEntry, actor: Control, target: Control, target_px: Vector2) -> void`
```logic
if entry == null:
	return
var distance: int = -1
if actor != null and (target != null or entry.category == "MOVE"):
	var d: Vector2 = (target_px - actor.position) / float(tile_px)
	distance = roundi(absf(d.x) + absf(d.y))
var presets: Array[String] = FxPresets.for_entry(entry, distance)
var planned: int = 0
for id: String in presets:
	planned += FxPresets.tween_count(id)
if live_count() + planned > FxPresets.MAX_LIVE:
	kill_all()
for id: String in presets:
	if _apply(id, entry, actor, target, target_px):
		fx_played.emit(id, entry)
```

### `slide(token: Control, to_px: Vector2) -> void`
```logic
if token == null:
	return
for run: Run in _live:
	if run.node == token and run.has_pos and run.end_pos == to_px:
		return
if token.position == to_px:
	return
_tween_position(token, [to_px], [FxPresets.duration("move")], to_px, true)
```

### `kill_all() -> void`
```logic
_gen += 1
for run: Run in _live:
	_finish(run)
_live.clear()
for node: Node in _temp:
	if is_instance_valid(node):
		if node.is_inside_tree():
			node.queue_free()
		else:
			node.free()
_temp.clear()
```

### `_instant() -> bool`
```logic
return FxSettings.instant or not is_inside_tree()
```

### `_apply(id: String, entry: ActionLogEntry, actor: Control, target: Control, target_px: Vector2) -> bool`
```logic
var on: Control = target if target != null else actor   # target-side presets fall back to the actor (self buffs)
match id:
	"move":
		if actor == null:
			return false
		_tween_position(actor, [target_px], [FxPresets.duration("move")], target_px, true)
	"attack_melee":
		if actor == null:
			return false
		_settle(actor)
		var orig: Vector2 = actor.position
		if (target_px - orig).length() >= 0.1:
			var bump: Vector2 = orig + (target_px - orig) * FxPresets.MELEE_BUMP
			_tween_position(actor, [bump, orig], FxPresets.durations("attack_melee"), orig, false)
	"attack_ranged":
		if actor == null:
			return false
		_pulse(actor, FxPresets.team_accent(entry.actor_is_ally), FxPresets.duration(id))
	"hit", "mitigated":
		if target == null:
			return false
		_settle(target)
		var base: Vector2 = target.position
		var points: Array[Vector2] = []
		for offset: Vector2 in FxPresets.SHAKE_OFFSETS:
			points.append(base + offset)
		_tween_position(target, points, FxPresets.durations(id).slice(0, 4), base, false)
		_pulse(target, FxPresets.color(id), FxPresets.duration(id, 4))
	"miss":
		if target == null:
			return false
		_settle(target)
		var home: Vector2 = target.position
		_tween_position(target, [home + FxPresets.MISS_SIDESTEP, home], FxPresets.durations(id), home, false)
	"heal", "buff":
		if on == null:
			return false
		_pulse(on, FxPresets.color(id), FxPresets.duration(id))
	"defend":
		if actor == null:
			return false
		_pulse(actor, FxPresets.color(id), FxPresets.duration(id))
	"status":
		if on == null:
			return false
		if not _instant():
			_status_pop(on, FxPresets.status_id_of(entry))
	"float_text":
		if not _instant():
			var anchor: Vector2 = FxPresets.AMBUSH_TEXT_PX if on == null else on.position
			_float_label(anchor, FxPresets.float_text(entry), FxPresets.float_color(entry))
	"defeat":
		if on == null:
			return false
		_settle(on)
		var faded: Color = Color(on.modulate, 0.0)
		if _instant():
			on.modulate = faded
		else:
			var run: Run = _track(on)
			run.has_mod = true
			run.end_mod = faded
			run.tween.tween_property(on, "modulate:a", 0.0, FxSettings.scaled(FxPresets.duration(id)))
			_close(run)
	_:
		return false
return true
```

### `_tween_position(node: Control, points: Array[Vector2], times: Array[float], end_px: Vector2, eased: bool) -> void`
```logic
if _instant():
	node.position = end_px
	return
_settle_position(node)
var run: Run = _track(node)
run.has_pos = true
run.end_pos = end_px
for i: int in points.size():
	var t: float = FxSettings.scaled(times[i] if i < times.size() else times[-1])
	var step: PropertyTweener = run.tween.tween_property(node, "position", points[i], t)
	if eased:
		step.set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	elif points.size() == 2:
		step.set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT if i == 0 else Tween.EASE_IN)
_close(run)
```

### `_pulse(node: Control, col: Color, time: float) -> void`
```logic
if _instant():
	node.modulate = Color.WHITE
	return
_settle_modulate(node)
var run: Run = _track(node)
run.has_mod = true
run.end_mod = Color.WHITE
node.modulate = col
run.tween.tween_property(node, "modulate", Color.WHITE, FxSettings.scaled(time)).from(col)   # a refresh may reset modulate before the first step
_close(run)
```

### `_status_pop(on: Control, status_id: String) -> void`
```logic
var icon := TextureRect.new()
icon.texture = StatusArt.icon(status_id)
icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
icon.custom_minimum_size = Vector2(FxPresets.STATUS_ICON_PX, FxPresets.STATUS_ICON_PX)
icon.size = icon.custom_minimum_size
icon.mouse_filter = Control.MOUSE_FILTER_IGNORE
var start: Vector2 = on.position + Vector2((tile_px - FxPresets.STATUS_ICON_PX) / 2.0, -FxPresets.STATUS_ICON_PX / 2.0)
icon.position = start
_add_temp(icon)
var run: Run = _track(icon)
var t: float = FxSettings.scaled(FxPresets.duration("status"))
run.tween.set_parallel(true)
run.tween.tween_property(icon, "position", start + Vector2(0, -FxPresets.STATUS_RISE_PX), t).set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
run.tween.tween_property(icon, "modulate:a", 0.0, t)
run.tween.chain().tween_callback(_free_temp.bind(icon, _gen))
_close(run)
```

### `_float_label(pos: Vector2, text: String, col: Color) -> void`
```logic
var lbl := Label.new()
lbl.text = text
lbl.add_theme_font_size_override("font_size", 10)
lbl.add_theme_color_override("font_color", col)
lbl.add_theme_color_override("font_shadow_color", Color(0, 0, 0, 0.9))
lbl.add_theme_constant_override("shadow_offset_x", 1)
lbl.add_theme_constant_override("shadow_offset_y", 1)
lbl.mouse_filter = Control.MOUSE_FILTER_IGNORE
lbl.position = pos + Vector2(8, -6)
lbl.scale = Vector2(1.2, 1.2)
_add_temp(lbl)
var run: Run = _track(lbl)
var times: Array[float] = FxPresets.durations("float_text")
run.tween.set_parallel(true)
run.tween.tween_property(lbl, "position", pos + FxPresets.FLOAT_RISE, FxSettings.scaled(times[0])).set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
run.tween.tween_property(lbl, "scale", Vector2(1.0, 1.0), FxSettings.scaled(times[1]))
run.tween.tween_property(lbl, "modulate:a", 0.0, FxSettings.scaled(times[2])).set_delay(FxSettings.scaled(FxPresets.FLOAT_DELAY))
run.tween.chain().tween_callback(_free_temp.bind(lbl, _gen))
_close(run)
```

### `_add_temp(node: Control) -> void`
```logic
if overlay != null and is_instance_valid(overlay):
	overlay.add_child(node)
else:
	add_child(node)
_temp.append(node)
```

### `_track(node: CanvasItem) -> Run`
```logic
var kept: Array[Run] = []
for old: Run in _live:   # drop runs whose node was freed
	if is_instance_valid(old.node):
		kept.append(old)
_live = kept
var run := Run.new()
run.node = node
run.tween = create_tween()
run.tween.bind_node(node)
_live.append(run)
return run
```

### `_close(run: Run) -> void`
```logic
run.tween.finished.connect(_on_run_finished.bind(run, _gen))
```

### `_on_run_finished(run: Run, gen: int) -> void`
```logic
if gen != _gen:
	return
callbacks_fired += 1
_live.erase(run)
```

### `_free_temp(node: Node, gen: int) -> void`
```logic
if gen != _gen:
	return
callbacks_fired += 1
_temp.erase(node)
if is_instance_valid(node):
	node.queue_free()
```

### `_settle(node: CanvasItem) -> void`
```logic
_settle_where(node, true, true)
```

### `_settle_position(node: CanvasItem) -> void`
```logic
_settle_where(node, true, false)
```

### `_settle_modulate(node: CanvasItem) -> void`
```logic
_settle_where(node, false, true)
```

### `_settle_where(node: CanvasItem, pos: bool, mod: bool) -> void`
```logic
var kept: Array[Run] = []
for run: Run in _live:
	if run.node == node and ((pos and run.has_pos) or (mod and run.has_mod)):
		_finish(run)
	else:
		kept.append(run)
_live = kept
```

### `_finish(run: Run) -> void`
```logic
if is_instance_valid(run.tween):
	run.tween.kill()
if not is_instance_valid(run.node):
	return
var node: CanvasItem = run.node
if run.has_pos:
	var ctrl: Control = node as Control
	if ctrl != null:
		ctrl.position = run.end_pos
if run.has_mod:
	node.modulate = run.end_mod
```

