---
class: RosterPanelUI
extends: RefCounted
description: "Tactical RPG - Roster Panels UI Component (COMBAT-UI2-L1, layout C) Two slim columns of unit cards. Party cards (left): token, name, level, stance, plan state, HP and AP bars, effect chips, speed and reach; muted outlines for free party slots. Hostile cards (right): token, name, stance, HP bar, one behaviour line. Heights come from CombatHudMetrics and stay fixed per text size; as text grows, cards drop lines in the spec's order (speed/reach, effects, behaviour). A click selects the unit."
---

# Properties
* `ALLY_COLOR`: Color = Color(0.2, 0.6, 1.0) (const)
* `ENEMY_COLOR`: Color = Color(0.9, 0.3, 0.3) (const)
* `COL_PLAN_NONE`: Color = Color(0.98, 0.78, 0.3) (const)
* `party_panel`: PanelContainer
* `enemy_panel`: PanelContainer
* `party_title_lbl`: Label
* `enemy_title_lbl`: Label
* `party_cards_container`: VBoxContainer
* `enemy_cards_container`: VBoxContainer
* `enemy_scroll`: ScrollContainer
* `roster_cards`: Dictionary = {} # Combatant -> PanelContainer
* `metrics`: CombatHudMetrics = null
* `token_fn`: Callable = Callable()       # func(c: Combatant) -> Texture2D
* `behaviour_fn`: Callable = Callable()   # func(c: Combatant) -> String: the hostile card's behaviour line

# Signals
* `combatant_selected(combatant: Combatant)`
* `combatant_hovered(combatant: Combatant, card_node: Control)`
* `combatant_unhovered()`
* `combatant_focused(combatant: Combatant)`
* `combatant_blurred()`

# Methods

### `_init() -> void`
```logic
metrics = CombatHudMetrics.compute(Vector2(1280, 720), UiTheme.text_scale(), 2, CombatHudMetrics.HOSTILE_SLOTS)
_build_party_panel()
_build_enemy_panel()
set_metrics(metrics)
```

### `static _panel_style(bg: Color, border: Color) -> StyleBoxFlat`
```logic
var style: StyleBoxFlat = UiTheme.box(bg, border, 6, 6, CombatHudMetrics.PANEL_PAD)
return style
```

### `_header(text: String, color: Color) -> Label`
```logic
var lbl: Label = UiTheme.label(text, UiTheme.hud_font(), color)
lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
return lbl
```

### `_build_party_panel() -> void`
```logic
party_panel = PanelContainer.new()
party_panel.name = "PartyPanel"
party_panel.clip_contents = true
party_panel.add_theme_stylebox_override("panel", _panel_style(Color(0.08, 0.11, 0.16, 0.95), Color(0.2, 0.38, 0.65, 0.8)))
var box := VBoxContainer.new()
box.add_theme_constant_override("separation", CombatHudMetrics.CARD_GAP)
party_panel.add_child(box)
party_title_lbl = _header("PARTY", Color(0.38, 0.72, 1.0))
party_title_lbl.tooltip_text = "Click a hero to plan for them"
box.add_child(party_title_lbl)
party_cards_container = VBoxContainer.new()
party_cards_container.add_theme_constant_override("separation", CombatHudMetrics.CARD_GAP)
box.add_child(party_cards_container)
```

### `_build_enemy_panel() -> void`
```logic
enemy_panel = PanelContainer.new()
enemy_panel.name = "EnemyPanel"
enemy_panel.clip_contents = true
enemy_panel.add_theme_stylebox_override("panel", _panel_style(Color(0.12, 0.08, 0.10, 0.95), Color(0.65, 0.25, 0.25, 0.8)))
var box := VBoxContainer.new()
box.add_theme_constant_override("separation", CombatHudMetrics.CARD_GAP)
enemy_panel.add_child(box)
enemy_title_lbl = _header("HOSTILES", Color(1.0, 0.45, 0.45))
enemy_title_lbl.tooltip_text = "Active hostiles; click one to inspect it"
box.add_child(enemy_title_lbl)
enemy_scroll = ScrollContainer.new()
enemy_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
enemy_scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
enemy_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
box.add_child(enemy_scroll)
enemy_cards_container = VBoxContainer.new()
enemy_cards_container.size_flags_horizontal = Control.SIZE_EXPAND_FILL
enemy_cards_container.add_theme_constant_override("separation", CombatHudMetrics.CARD_GAP)
enemy_scroll.add_child(enemy_cards_container)
```

### `set_metrics(m: CombatHudMetrics) -> void`
```logic
metrics = m
party_panel.custom_minimum_size = Vector2(CombatHudMetrics.SIDE_W, m.party_panel_h)
enemy_panel.custom_minimum_size = Vector2(CombatHudMetrics.SIDE_W, m.hostile_panel_h)
party_title_lbl.add_theme_font_size_override("font_size", m.fs)
enemy_title_lbl.add_theme_font_size_override("font_size", m.fs)
```

### `clear_roster() -> void`
```logic
for container: Container in [party_cards_container, enemy_cards_container]:
	if container == null:
		continue
	for child: Node in container.get_children():
		container.remove_child(child)
		child.queue_free()
roster_cards.clear()
```

### `add_party_placeholders(filled: int) -> void`
```logic
var slots: int = metrics.party_slots
for i: int in range(filled, slots):
	var ph := PanelContainer.new()
	ph.name = "EmptySlot"
	ph.set_meta("empty_slot", true)
	ph.custom_minimum_size = Vector2(0, metrics.party_slim_h)
	ph.tooltip_text = "Empty party slot"
	ph.add_theme_stylebox_override("panel", UiTheme.box(Color(0.06, 0.08, 0.12, 0.25), Color(0.2, 0.28, 0.4, 0.45), 4, 3, 1))
	party_cards_container.add_child(ph)
party_title_lbl.text = "PARTY %d/%d" % [filled, slots]
```

### `_compact() -> bool`
```logic
return metrics.scale > 1.1
```

### `create_roster_card(combatant: Combatant, team_color: Color, is_ally: bool, _grid_pos: Vector2i = Vector2i(-1, -1)) -> PanelContainer`
```logic
var target: VBoxContainer = party_cards_container if is_ally else enemy_cards_container
var lh: int = metrics.lh
var fs: int = metrics.fs
var rows: int = metrics.party_rows if is_ally else metrics.hostile_rows
var card := PanelContainer.new()
card.name = "Card_%s" % combatant.name
card.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
card.custom_minimum_size = Vector2(0, metrics.party_card_h if is_ally else metrics.hostile_card_h)
card.clip_contents = true
card.tooltip_text = combatant.name
card.add_theme_stylebox_override("panel", UiTheme.box(Color(0.10, 0.13, 0.18, 0.92), team_color, 3, 3, CombatHudMetrics.CARD_PAD))

var body := VBoxContainer.new()
body.add_theme_constant_override("separation", CombatHudMetrics.ROW_GAP)
card.add_child(body)

# Rows 1-2: token beside the name row and the stance / plan row.
var head := HBoxContainer.new()
head.add_theme_constant_override("separation", 4)
body.add_child(head)
var token_px: int = 2 * lh + CombatHudMetrics.ROW_GAP
var token: TextureRect = UiTheme.icon_rect(token_fn.call(combatant) if token_fn.is_valid() else null, token_px)
token.name = "Token"
head.add_child(token)
var lines := VBoxContainer.new()
lines.size_flags_horizontal = Control.SIZE_EXPAND_FILL
lines.add_theme_constant_override("separation", CombatHudMetrics.ROW_GAP)
head.add_child(lines)

var name_row := HBoxContainer.new()
name_row.add_theme_constant_override("separation", 6)
lines.add_child(name_row)
var name_lbl: Label = UiTheme.label(_short_name(combatant, is_ally), fs, team_color.lightened(0.35))
name_lbl.name = "NameLabel"
name_lbl.clip_text = true
name_lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
name_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
name_row.add_child(name_lbl)
var stance_lbl: Label = UiTheme.label("", fs, Color(0.65, 0.78, 0.9) if is_ally else Color(0.8, 0.7, 0.7))
stance_lbl.name = "StanceLabel"
stance_lbl.clip_text = true
stance_lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
if is_ally:
	var lv: int = combatant.stats.level if combatant.stats != null else 1
	var lv_lbl: Label = UiTheme.label("Lv %d" % lv, fs, Color(0.75, 0.82, 0.9))
	lv_lbl.name = "LevelLabel"
	name_row.add_child(lv_lbl)
	var state_row := HBoxContainer.new()
	state_row.add_theme_constant_override("separation", 6)
	lines.add_child(state_row)
	stance_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	state_row.add_child(stance_lbl)
	var plan_lbl: Label = UiTheme.label("", fs, COL_PLAN_NONE)
	plan_lbl.name = "PlanLabel"
	plan_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	state_row.add_child(plan_lbl)
	# Row 3-4: HP and AP bars; rows 5-6: effect chips, speed and reach.
	_bar_row(body, "Hp", "HP", combatant.hp, combatant.max_hp, Color(0.25, 0.75, 0.35), fs, lh)
	_bar_row(body, "Ap", "AP", combatant.ap, maxi(1, combatant.max_ap), Color(0.2, 0.6, 0.95), fs, lh)
	if rows >= 5:
		var fx: Label = UiTheme.label("", fs, Color(0.7, 0.9, 0.7))
		fx.name = "EffectsLabel"
		fx.clip_text = true
		fx.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
		body.add_child(fx)
	if rows >= 6:
		var spd: Label = UiTheme.label("", fs, Color(0.7, 0.8, 0.9))
		spd.name = "SpeedLabel"
		spd.clip_text = true
		spd.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
		body.add_child(spd)
else:
	# Hostile: row 1 name + stance chip, row 2 HP bar (both beside the token), row 3 the behaviour line.
	stance_lbl.custom_minimum_size = Vector2(fs * 2.2, 0)   # the 3-letter chip
	name_row.add_child(stance_lbl)
	var eye: TextureRect = UiTheme.icon_rect(AssetRegistry.texture("ui", "badge_foresight", "icon"), lh)   # INTENT1-U: a foresight holder reads this foe's plan
	eye.name = "ForesightBadge"
	eye.tooltip_text = "Its whole round plan is read"
	eye.visible = false
	name_row.add_child(eye)
	name_row.move_child(eye, 1)
	_bar_row(lines, "Hp", "HP", combatant.hp, combatant.max_hp, Color(0.85, 0.3, 0.3), fs, lh)
	if rows >= 3:
		var beh: Label = UiTheme.label(behaviour_fn.call(combatant) if behaviour_fn.is_valid() else "", fs, Color(0.9, 0.65, 0.4))
		beh.name = "BehaviourLabel"
		beh.clip_text = true
		beh.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
		body.add_child(beh)

card.mouse_filter = Control.MOUSE_FILTER_STOP
for child: Control in card.find_children("*", "Control", true, false):
	child.mouse_filter = Control.MOUSE_FILTER_IGNORE
card.gui_input.connect(func(event: InputEvent) -> void:
	var mouse_event := event as InputEventMouseButton
	if mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT:
		combatant_selected.emit(combatant)
	elif event.is_action_pressed("ui_accept"):
		combatant_selected.emit(combatant)
)
card.focus_mode = Control.FOCUS_ALL
card.focus_entered.connect(func() -> void: combatant_focused.emit(combatant))
card.focus_exited.connect(func() -> void: combatant_blurred.emit())
card.mouse_entered.connect(func() -> void: combatant_hovered.emit(combatant, card))
card.mouse_exited.connect(func() -> void: combatant_unhovered.emit())
target.add_child(card)
roster_cards[combatant] = card
return card
```

### `_bar_row(body: Container, key: String, tag: String, value: int, max_value: int, fill: Color, fs: int, lh: int) -> void`
```logic
var row := HBoxContainer.new()
row.add_theme_constant_override("separation", 4)
body.add_child(row)
var tag_lbl: Label = UiTheme.label(tag, fs, fill.lightened(0.2))
row.add_child(tag_lbl)
var bar := ProgressBar.new()
bar.name = "%sBar" % key
bar.custom_minimum_size = Vector2(10, lh - 6)
bar.size_flags_horizontal = Control.SIZE_EXPAND_FILL
bar.size_flags_vertical = Control.SIZE_SHRINK_CENTER
bar.show_percentage = false
bar.max_value = max_value
bar.value = value
var fill_box := StyleBoxFlat.new()
fill_box.bg_color = fill
fill_box.set_corner_radius_all(2)
bar.add_theme_stylebox_override("fill", fill_box)
var back := StyleBoxFlat.new()
back.bg_color = Color(0.05, 0.07, 0.1, 0.9)
back.set_corner_radius_all(2)
bar.add_theme_stylebox_override("background", back)
row.add_child(bar)
var val: Label = UiTheme.label("%d/%d" % [value, max_value], fs, Color(0.88, 0.92, 0.96))
val.name = "%sLabel" % key
row.add_child(val)
```

### `_short_name(c: Combatant, is_ally: bool) -> String`
```logic
if is_ally:
	return c.name
for prefix: String in ["Goblin ", "Enemy ", "Fleeing "]:
	if c.name.begins_with(prefix):
		return c.name.substr(prefix.length())
return c.name
```

### `_stance_text(stance: CombatTypes.StanceType, is_ally: bool) -> String`
```logic
var full: String = CombatTypes.stance_to_string(stance)
return full.left(3) if (_compact() or not is_ally) else full
```

### `update_all_roster_cards(allies: Array[Combatant], grid: CombatGrid, snap: BlockSnapshot = null) -> void`
```logic
var alive_enemies: int = 0
for combatant: Combatant in roster_cards:
	var raw: Variant = roster_cards[combatant]
	if not (raw is PanelContainer) or not is_instance_valid(raw):
		continue
	var card: PanelContainer = raw
	var is_ally: bool = allies.has(combatant)
	var cur_hp: int = DictUtil.int_of(snap.hps, combatant, combatant.hp) if snap != null else combatant.hp
	var max_hp: int = DictUtil.int_of(snap.max_hps, combatant, combatant.max_hp) if snap != null else combatant.max_hp
	var cur_ap: int = DictUtil.int_of(snap.aps, combatant, combatant.ap) if snap != null else combatant.ap
	var withdrawn: bool = combatant.is_withdrawn
	if snap != null:
		var wd: Variant = snap.is_withdrawn.get(combatant, withdrawn)
		withdrawn = wd if wd is bool else withdrawn
	var stance: CombatTypes.StanceType = combatant.current_stance
	if snap != null:
		var st: Variant = snap.stances.get(combatant, stance)
		stance = st if st is CombatTypes.StanceType else stance
	var alive: bool = cur_hp > 0
	if not is_ally and card.visible and alive and not withdrawn:
		alive_enemies += 1
	_set_bar(card, "Hp", cur_hp, max_hp)
	_set_bar(card, "Ap", cur_ap, maxi(1, combatant.max_ap))
	var stance_lbl: Label = card.find_child("StanceLabel", true, false) as Label
	if stance_lbl != null:
		if not alive:
			stance_lbl.text = "DEFEATED"
			stance_lbl.add_theme_color_override("font_color", Color(0.85, 0.35, 0.35))
		elif withdrawn:
			stance_lbl.text = "ESCAPED"
			stance_lbl.add_theme_color_override("font_color", Color(0.95, 0.75, 0.25))
		else:
			stance_lbl.text = _stance_text(stance, is_ally)
			stance_lbl.add_theme_color_override("font_color", Color(0.65, 0.78, 0.9) if is_ally else Color(0.8, 0.7, 0.7))
	var fx_lbl: Label = card.find_child("EffectsLabel", true, false) as Label
	if fx_lbl != null:
		var chips: Array[String] = []
		if combatant.attack_boost_rounds_remaining > 0:
			chips.append("Focus")
		for status: Variant in combatant.statuses:
			chips.append(StatusEffects.chip_text(combatant, str(status)))
		fx_lbl.text = ", ".join(chips) if not chips.is_empty() else "No effects"
		fx_lbl.add_theme_color_override("font_color", Color(0.7, 0.9, 0.7) if not chips.is_empty() else Color(0.45, 0.55, 0.62))
	var spd_lbl: Label = card.find_child("SpeedLabel", true, false) as Label
	if spd_lbl != null:
		var band: Vector2i = SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(combatant)
		spd_lbl.text = "Spd %d  Reach %s" % [combatant.speed, str(band.y) if band.x == band.y else "%d-%d" % [band.x, band.y]]
	if not alive:
		card.modulate = Color(0.55, 0.55, 0.55, 0.45)
	elif withdrawn:
		card.modulate = Color(0.65, 0.65, 0.65, 0.45)
	else:
		card.modulate = Color.WHITE
enemy_title_lbl.text = "HOSTILES %d active" % alive_enemies
if grid == null:
	return
```

### `_set_bar(card: Control, key: String, value: int, max_value: int) -> void`
```logic
var bar: ProgressBar = card.find_child("%sBar" % key, true, false) as ProgressBar
var lbl: Label = card.find_child("%sLabel" % key, true, false) as Label
if bar != null:
	bar.max_value = max_value
	bar.value = value
	if key == "Hp":
		var ratio: float = float(value) / float(maxi(1, max_value))
		var fill := StyleBoxFlat.new()
		fill.set_corner_radius_all(2)
		fill.bg_color = Color(0.25, 0.75, 0.35) if ratio > 0.5 else (Color(0.9, 0.75, 0.2) if ratio > 0.25 else Color(0.9, 0.25, 0.25))
		bar.add_theme_stylebox_override("fill", fill)
if lbl != null:
	lbl.text = "%d/%d" % [value, max_value]
```

### `update_highlights(selected_combatant: Combatant, allies: Array[Combatant]) -> void`
```logic
for c: Combatant in roster_cards:
	var raw: Variant = roster_cards[c]
	if not (raw is PanelContainer) or not is_instance_valid(raw):
		continue
	var card: PanelContainer = raw
	var base_color: Color = ALLY_COLOR if allies.has(c) else ENEMY_COLOR
	var style: StyleBoxFlat = UiTheme.box(Color(0.11, 0.14, 0.19, 0.9), base_color, 3, 3, CombatHudMetrics.CARD_PAD, 1)
	if c == selected_combatant:
		style = UiTheme.box(Color(0.18, 0.22, 0.32, 0.95), Color(1.0, 0.85, 0.25), 3, 3, CombatHudMetrics.CARD_PAD, 2)
	card.add_theme_stylebox_override("panel", style)
```

### `content_fits() -> bool`
```logic
for raw: Variant in roster_cards.values():
	var card: Control = raw as Control
	if card == null or not is_instance_valid(card) or card.get_child_count() == 0:
		continue
	var body: Control = card.get_child(0) as Control
	if body.get_combined_minimum_size().y + 2.0 * CombatHudMetrics.CARD_PAD > card.custom_minimum_size.y + 0.5:
		return false
var party_box: Control = party_panel.get_child(0) as Control
if party_box.get_combined_minimum_size().y + 2.0 * CombatHudMetrics.PANEL_PAD > metrics.party_panel_h + 0.5:
	return false
return enemy_cards_container.get_combined_minimum_size().y + float(metrics.lh) + 2.0 * CombatHudMetrics.PANEL_PAD + CombatHudMetrics.CARD_GAP <= metrics.hostile_panel_h + 0.5
```

