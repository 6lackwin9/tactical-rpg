class_name RosterPanelUI
extends RefCounted

# ==============================================================================
# Tactical RPG - Roster Panels UI Component (COMBAT-UI3, layout A; was COMBAT-UI2-L1 layout C)
# Two slim columns of unit cards. Compact party cards (left): name, level, HP bar, "HP x/y · AP a/b"; muted
# outlines for free party slots; the selected-hero panel sits under them (CombatHeroPanel). Hostile cards (right):
# token, name, HP, one "Next:" line; the merged enemy card sits under them (CombatEnemyCardPanel). Heights and
# widths come from CombatHudMetrics and stay fixed per text size. A click selects the unit.
# ==============================================================================

signal combatant_selected(combatant: Combatant)
signal combatant_hovered(combatant: Combatant, card_node: Control)
signal combatant_unhovered()
signal combatant_focused(combatant: Combatant)   # keyboard / controller focus lands on a card
signal combatant_blurred()

const ALLY_COLOR: Color = Color(0.2, 0.6, 1.0)
const ENEMY_COLOR: Color = Color(0.9, 0.3, 0.3)

var party_panel: PanelContainer
var enemy_panel: PanelContainer
var party_title_lbl: Label
var enemy_title_lbl: Label
var party_cards_container: VBoxContainer
var enemy_cards_container: VBoxContainer
var enemy_scroll: ScrollContainer
var roster_cards: Dictionary = {} # Combatant -> PanelContainer
var metrics: CombatHudMetrics = null
var token_fn: Callable = Callable()       # func(c: Combatant) -> Texture2D
var behaviour_fn: Callable = Callable()   # func(c: Combatant) -> String: the hostile card's "Next:" line

func _init() -> void:
	metrics = CombatHudMetrics.compute(Vector2(1280, 720), UiTheme.text_scale(), 2, CombatHudMetrics.HOSTILE_SLOTS)
	_build_party_panel()
	_build_enemy_panel()
	set_metrics(metrics)

static func _panel_style(bg: Color, border: Color) -> StyleBoxFlat:
	return UiTheme.box(bg, border, 6, 6, CombatHudMetrics.PANEL_PAD)

func _header(text: String, color: Color) -> Label:
	var lbl: Label = UiTheme.label(text, UiTheme.hud_font(), color)
	lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	return lbl

func _build_party_panel() -> void:
	party_panel = PanelContainer.new()
	party_panel.name = "PartyPanel"
	party_panel.clip_contents = true
	party_panel.add_theme_stylebox_override("panel", _panel_style(Color(0.08, 0.11, 0.16, 0.95), Color(0.2, 0.38, 0.65, 0.8)))
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", CombatHudMetrics.CARD_GAP)
	party_panel.add_child(box)
	party_title_lbl = _header("Party", Color(0.56, 0.71, 0.96))
	party_title_lbl.tooltip_text = "Click a hero to plan for them"
	box.add_child(party_title_lbl)
	party_cards_container = VBoxContainer.new()
	party_cards_container.add_theme_constant_override("separation", CombatHudMetrics.CARD_GAP)
	box.add_child(party_cards_container)

func _build_enemy_panel() -> void:
	enemy_panel = PanelContainer.new()
	enemy_panel.name = "EnemyPanel"
	enemy_panel.clip_contents = true
	enemy_panel.add_theme_stylebox_override("panel", _panel_style(Color(0.12, 0.08, 0.10, 0.95), Color(0.65, 0.25, 0.25, 0.8)))
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", CombatHudMetrics.CARD_GAP)
	enemy_panel.add_child(box)
	enemy_title_lbl = _header("Hostiles", Color(0.94, 0.55, 0.53))
	enemy_title_lbl.tooltip_text = "Active hostiles; hover or click one for its card below"
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

# New sizes: the panels take their fixed heights; the caller rebuilds the cards when the sizes changed.
func set_metrics(m: CombatHudMetrics) -> void:
	metrics = m
	party_panel.custom_minimum_size = Vector2(m.left_w, m.party_panel_h)
	enemy_panel.custom_minimum_size = Vector2(m.right_w, m.hostile_panel_h)
	party_title_lbl.add_theme_font_size_override("font_size", m.fs)
	enemy_title_lbl.add_theme_font_size_override("font_size", m.fs)

func clear_roster() -> void:
	for container: Container in [party_cards_container, enemy_cards_container]:
		if container == null:
			continue
		for child: Node in container.get_children():
			container.remove_child(child)
			child.queue_free()
	roster_cards.clear()

# Muted outlines for the party slots nobody fills (no "+", they are not buttons).
func add_party_placeholders(filled: int) -> void:
	var slots: int = metrics.party_slots
	for i: int in range(filled, slots):
		var ph := PanelContainer.new()
		ph.name = "EmptySlot"
		ph.set_meta("empty_slot", true)
		ph.custom_minimum_size = Vector2(0, metrics.party_slim_h)
		ph.tooltip_text = "Empty party slot"
		ph.add_theme_stylebox_override("panel", UiTheme.box(Color(0.06, 0.08, 0.12, 0.25), Color(0.2, 0.28, 0.4, 0.45), 4, 3, 1))
		party_cards_container.add_child(ph)
	party_title_lbl.text = "Party %d/%d" % [filled, slots]

func _line_label(text: String, node_name: String, color: Color) -> Label:
	var lbl: Label = UiTheme.label(text, metrics.fs, color)
	lbl.name = node_name
	lbl.clip_text = true
	lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	lbl.custom_minimum_size = Vector2(0, metrics.lh)
	return lbl

func create_roster_card(combatant: Combatant, team_color: Color, is_ally: bool, _grid_pos: Vector2i = Vector2i(-1, -1)) -> PanelContainer:
	var target: VBoxContainer = party_cards_container if is_ally else enemy_cards_container
	var lh: int = metrics.lh
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
	if is_ally:
		# Row 1: name, level; row 2: the HP bar; row 3: "HP x/y · AP a/b".
		var name_row := HBoxContainer.new()
		name_row.add_theme_constant_override("separation", 6)
		body.add_child(name_row)
		var name_lbl: Label = _line_label(_short_name(combatant, true), "NameLabel", team_color.lightened(0.35))
		name_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		name_row.add_child(name_lbl)
		var lv: int = combatant.stats.level if combatant.stats != null else 1
		name_row.add_child(_line_label("Lv %d" % lv, "LevelLabel", UiTheme.COL_MUTED))
		var bar_row := HBoxContainer.new()
		bar_row.custom_minimum_size = Vector2(0, lh)
		body.add_child(bar_row)
		var bar: ProgressBar = _bar("HpBar", combatant.hp, combatant.max_hp, Color(0.25, 0.75, 0.35))
		bar.custom_minimum_size = Vector2(10, maxi(6, lh - 10))
		bar.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		bar.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		bar_row.add_child(bar)
		body.add_child(_line_label("", "StatsLabel", Color(0.75, 0.82, 0.9)))
	else:
		# Token beside row 1 (name, read badge, HP) and row 2 (the "Next:" line).
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
		name_row.add_theme_constant_override("separation", 4)
		lines.add_child(name_row)
		var name_lbl: Label = _line_label(_short_name(combatant, false), "NameLabel", team_color.lightened(0.35))
		name_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		name_row.add_child(name_lbl)
		var eye: TextureRect = UiTheme.icon_rect(AssetRegistry.texture("ui", "badge_foresight", "icon"), lh)   # INTENT1-U: a foresight holder reads this foe's plan
		eye.name = "ForesightBadge"
		eye.tooltip_text = "Its whole round plan is read"
		eye.visible = false
		name_row.add_child(eye)
		name_row.add_child(_line_label("", "HpLabel", Color(0.88, 0.92, 0.96)))
		var beh: Label = _line_label(behaviour_fn.call(combatant) if behaviour_fn.is_valid() else "", "BehaviourLabel", UiTheme.COL_WARN)
		lines.add_child(beh)

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

func _bar(node_name: String, value: int, max_value: int, fill: Color) -> ProgressBar:
	var bar := ProgressBar.new()
	bar.name = node_name
	bar.show_percentage = false
	bar.max_value = max_value
	bar.value = value
	var fill_box := StyleBoxFlat.new()
	fill_box.bg_color = fill
	fill_box.set_corner_radius_all(3)
	bar.add_theme_stylebox_override("fill", fill_box)
	var back := StyleBoxFlat.new()
	back.bg_color = Color(0.15, 0.19, 0.30, 0.9)
	back.set_corner_radius_all(3)
	bar.add_theme_stylebox_override("background", back)
	return bar

func _short_name(c: Combatant, is_ally: bool) -> String:
	if is_ally:
		return c.name
	for prefix: String in ["Goblin ", "Enemy ", "Fleeing "]:
		if c.name.begins_with(prefix):
			return c.name.substr(prefix.length())
	return c.name

static func hp_color(value: int, max_value: int) -> Color:
	var ratio: float = float(value) / float(maxi(1, max_value))
	return Color(0.25, 0.75, 0.35) if ratio > 0.5 else (Color(0.9, 0.75, 0.2) if ratio > 0.25 else Color(0.9, 0.25, 0.25))

func update_all_roster_cards(allies: Array[Combatant], grid: CombatGrid, snap: BlockSnapshot = null) -> void:
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
		var alive: bool = cur_hp > 0
		if not is_ally and card.visible and alive and not withdrawn:
			alive_enemies += 1
		var bar: ProgressBar = card.find_child("HpBar", true, false) as ProgressBar
		if bar != null:
			bar.max_value = max_hp
			bar.value = cur_hp
			var fill := StyleBoxFlat.new()
			fill.set_corner_radius_all(3)
			fill.bg_color = hp_color(cur_hp, max_hp)
			bar.add_theme_stylebox_override("fill", fill)
		var state: String = "Defeated" if not alive else ("Withdrew" if withdrawn else "")
		var stats: Label = card.find_child("StatsLabel", true, false) as Label
		if stats != null:
			stats.text = state if not state.is_empty() else "HP %d/%d · AP %d/%d" % [cur_hp, max_hp, cur_ap, combatant.max_ap]
			stats.tooltip_text = stats.text
		var hp_lbl: Label = card.find_child("HpLabel", true, false) as Label
		if hp_lbl != null:
			hp_lbl.text = state if not state.is_empty() else "%d/%d" % [cur_hp, max_hp]
			hp_lbl.add_theme_color_override("font_color", UiTheme.COL_MUTED if not state.is_empty() else hp_color(cur_hp, max_hp).lightened(0.25))
		if not alive:
			card.modulate = Color(0.55, 0.55, 0.55, 0.45)
		elif withdrawn:
			card.modulate = Color(0.65, 0.65, 0.65, 0.45)
		else:
			card.modulate = Color.WHITE
	enemy_title_lbl.text = "Hostiles %d" % alive_enemies
	if grid == null:
		return

func update_highlights(selected_combatant: Combatant, allies: Array[Combatant]) -> void:
	for c: Combatant in roster_cards:
		var raw: Variant = roster_cards[c]
		if not (raw is PanelContainer) or not is_instance_valid(raw):
			continue
		var card: PanelContainer = raw
		var base_color: Color = Color(0.16, 0.23, 0.37, 1.0)
		var style: StyleBoxFlat = UiTheme.box(Color(0.08, 0.11, 0.20, 0.95), base_color, 5, 6, CombatHudMetrics.CARD_PAD, 1)
		if c == selected_combatant:
			style = UiTheme.box(Color(0.14, 0.20, 0.35, 0.98), UiTheme.COL_WARN if allies.has(c) else Color(0.94, 0.55, 0.53), 5, 6, CombatHudMetrics.CARD_PAD, 2)
		card.add_theme_stylebox_override("panel", style)

# True when every card's content and both panels' content fit their fixed heights (nothing is clipped).
func content_fits() -> bool:
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
