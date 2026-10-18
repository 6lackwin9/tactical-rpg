class_name TimelineDockUI
extends Control

# ==============================================================================
# Tactical RPG - Timeline Dock UI
# Top timeline row (COMBAT-UI2-L3): round arrows, the block ruler, the focused unit's action track and the control
# row: plan info / autoplay banner, party and hero Tactic dropdowns (AUTO1; the old Draft menu is gone), the Autoplay
# toggle, Clear and Confirm. All text follows UiTheme.text_scale() (14 px floor at Normal); the row keeps its fixed
# height (CombatHudMetrics.TOP_H) at every text size by dropping the clip subtitle and abbreviating labels, never by clipping.
# ==============================================================================

signal round_nav_requested(delta: int)
signal block_selected(block: int)
signal combatant_selected(c: Combatant)
signal hero_view_requested()
signal party_tactic_requested(tactic_id: String)               # the party dropdown (every hero at once)
signal hero_tactic_requested(tactic_id: String)                # the hero dropdown (the hero whose track is shown)
signal autoplay_toggled(on: bool)
signal clear_requested()                                       # planning: clear the hero's sequence
signal confirm_requested()                                     # planning: lock the round
signal step_remove_requested(step_index: int)
signal step_reorder_requested(from_step: int, to_step: int)
signal fit_changed()            # block width changed to fit the dock: the owner should redraw

const BLOCK_GAP: int = 4
const BANNER_TEXT: String = "🔒 Hostiles plan in secret; see Threat Intel on their card."
const TRACK_WIDTH: int = 830   # block widths scale so a round always fills this width

var btn_prev_round: Button
var btn_next_round_arrow: Button
var tracks_container: VBoxContainer
var tracks_ruler_row: HBoxContainer
var total_blocks: int = 10
var block_w: int = 80
var ruler_tick_nodes: Array[PanelContainer] = []
var ruler_tick_labels: Array[Label] = []
var single_track_row: HBoxContainer
var plan_row: HBoxContainer
var plan_info_lbl: Label
var party_option: OptionButton
var hero_option: OptionButton
var party_caption: Label
var hero_caption: Label
var autoplay_btn: Button
var confirm_row: HBoxContainer      # "redraft discards your edits?" Redraft / Keep
var confirm_lbl: Label
var confirm_yes_btn: Button
var confirm_no_btn: Button
var autoplay_state: Dictionary = {}
var autoplay_enabled: bool = true   # false in replay / playback arenas: no tactic dropdowns, no Autoplay toggle
var _m: Dictionary = {}              # row_metrics() of the current rebuild (computed once per rebuild)
var _pending: Dictionary = {}       # a tactic pick waiting for the discard confirmation: {party: bool, id: String}
var clear_btn: Button
var confirm_btn: Button

var planning: bool = false
var plan_step_fn: Callable   # func(block: int) -> int: index of the step behind the command at a block
var track_clips_hbox: HBoxContainer
var enemy_classified_banner: PanelContainer
var banner_lbl: Label

# Callable: func(combatant: Combatant, block: int) -> CombatResolver.ScheduledAction
var action_lookup_fn: Callable
var read_only: bool = false   # INTENT1-U: the shown track is a read enemy plan: no drag, no remove

func _init() -> void:
	name = "TimelineDockUI"
	resized.connect(func() -> void:
		if _fit_blocks():
			fit_changed.emit()
	)
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	size_flags_vertical = Control.SIZE_EXPAND_FILL
	_build_ui()

# The fitted row sizes for the current text size (one text line `lh`, the ruler tick, a button, a clip).
func row_metrics() -> Dictionary:
	var fs: int = UiTheme.hud_font()
	var lh: int = CombatHudMetrics.line_height(fs)
	var tick_h: int = lh + 2
	var btn_h: int = lh + 4
	var two_line: bool = tick_h + 2 + (2 * lh + 4) + 2 + btn_h <= CombatHudMetrics.TOP_H - 4   # the clip subtitle drops first
	return {"fs": fs, "lh": lh, "tick_h": tick_h, "btn_h": btn_h, "two_line": two_line, "clip_h": (2 if two_line else 1) * lh + 4,
		"short": UiTheme.text_scale() > 1.1}

static func _btn_style(bg: Color, border: Color) -> StyleBoxFlat:
	return UiTheme.box(bg, border, 4, 6, 2)

# Flat, fixed-height look for every control of the row (the default button styles are taller than a text line).
func _style_button(btn: Button, fs: int, accent: Color = Color(0.38, 0.5, 0.68, 0.9)) -> void:
	btn.focus_mode = Control.FOCUS_NONE
	btn.add_theme_font_size_override("font_size", fs)
	btn.add_theme_stylebox_override("normal", _btn_style(Color(0.10, 0.14, 0.20, 0.95), accent))
	btn.add_theme_stylebox_override("hover", _btn_style(Color(0.16, 0.22, 0.32, 0.95), Color(0.55, 0.75, 1.0)))
	btn.add_theme_stylebox_override("pressed", _btn_style(Color(0.12, 0.30, 0.20, 0.95), UiTheme.COL_GOOD))
	btn.add_theme_stylebox_override("disabled", _btn_style(Color(0.08, 0.10, 0.14, 0.7), Color(0.2, 0.26, 0.34, 0.6)))
	btn.add_theme_stylebox_override("focus", StyleBoxEmpty.new())

func _make_option(tip: String, with_custom: bool, fs: int) -> OptionButton:
	var opt := OptionButton.new()
	opt.tooltip_text = tip
	opt.clip_text = true
	opt.fit_to_longest_item = false
	opt.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	opt.custom_minimum_size = Vector2(fs * 11, 0)
	_style_button(opt, fs)
	opt.get_popup().add_theme_font_size_override("font_size", fs)
	for entry: Dictionary in CombatAutoplay.tactic_entries(with_custom):
		opt.add_item(str(entry["label"]))
		var idx: int = opt.item_count - 1
		opt.set_item_metadata(idx, str(entry["id"]))
		opt.set_item_tooltip(idx, str(entry["description"]))
	return opt

func _build_ui() -> void:
	_m = row_metrics()
	var m: Dictionary = _m
	var fs: int = int(m["fs"])
	var timeline_flank_hbox := HBoxContainer.new()
	timeline_flank_hbox.set_anchors_preset(Control.PRESET_FULL_RECT)
	timeline_flank_hbox.add_theme_constant_override("separation", 4)
	add_child(timeline_flank_hbox)

	# 1. Left Round Traversal Arrow [ ◀ ] (Spans vertical height of ruler + action track)
	btn_prev_round = Button.new()
	btn_prev_round.text = "◀"
	btn_prev_round.tooltip_text = "Previous Round Timeline"
	btn_prev_round.custom_minimum_size = Vector2(22, 54)
	btn_prev_round.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_style_button(btn_prev_round, fs)
	btn_prev_round.pressed.connect(func() -> void: round_nav_requested.emit(-1))
	timeline_flank_hbox.add_child(btn_prev_round)

	# 2. Center Timeline Container: BLK Button tray on top, Action Timeline on bottom
	tracks_container = VBoxContainer.new()
	tracks_container.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	tracks_container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	tracks_container.add_theme_constant_override("separation", 2)
	timeline_flank_hbox.add_child(tracks_container)

	# Row 1: BLK Button Tray (Time Ruler Ticks BLK 01 .. BLK 10)
	tracks_ruler_row = HBoxContainer.new()
	tracks_ruler_row.add_theme_constant_override("separation", BLOCK_GAP)
	tracks_container.add_child(tracks_ruler_row)

	_build_ruler()

	# Row 2: Action Timeline (Selected Unit's Continuous Action Track Row)
	single_track_row = HBoxContainer.new()
	single_track_row.add_theme_constant_override("separation", BLOCK_GAP)
	tracks_container.add_child(single_track_row)

	track_clips_hbox = HBoxContainer.new()
	track_clips_hbox.name = "ClipsHBox"
	track_clips_hbox.add_theme_constant_override("separation", BLOCK_GAP)
	single_track_row.add_child(track_clips_hbox)

	# Control row: info / autoplay banner, tactic dropdowns, Autoplay, Clear, Confirm
	plan_row = HBoxContainer.new()
	plan_row.name = "ControlRow"
	plan_row.add_theme_constant_override("separation", 6)
	tracks_container.add_child(plan_row)
	plan_info_lbl = Label.new()
	plan_info_lbl.add_theme_font_size_override("font_size", fs)
	plan_info_lbl.add_theme_color_override("font_color", Color(0.98, 0.78, 0.3))
	plan_info_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	plan_info_lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	plan_info_lbl.clip_text = true
	plan_row.add_child(plan_info_lbl)

	confirm_row = HBoxContainer.new()
	confirm_row.name = "RedraftConfirm"
	confirm_row.visible = false
	confirm_row.add_theme_constant_override("separation", 6)
	confirm_row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	confirm_lbl = UiTheme.label("Redrafting throws away your edits.", fs, UiTheme.COL_WARN)
	confirm_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	confirm_lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	confirm_lbl.clip_text = true
	confirm_row.add_child(confirm_lbl)
	confirm_yes_btn = Button.new()
	confirm_yes_btn.text = "Redraft"
	_style_button(confirm_yes_btn, fs, UiTheme.COL_WARN)
	confirm_yes_btn.pressed.connect(func() -> void: resolve_redraft(true))
	confirm_row.add_child(confirm_yes_btn)
	confirm_no_btn = Button.new()
	confirm_no_btn.text = "Keep"
	_style_button(confirm_no_btn, fs)
	confirm_no_btn.pressed.connect(func() -> void: resolve_redraft(false))
	confirm_row.add_child(confirm_no_btn)
	plan_row.add_child(confirm_row)

	party_caption = UiTheme.label("Party", fs, UiTheme.COL_MUTED)
	plan_row.add_child(party_caption)
	party_option = _make_option("Party tactic: every hero drafts with it (clears the hero overrides)", false, fs)
	party_option.item_selected.connect(func(index: int) -> void: _on_option_picked(true, str(party_option.get_item_metadata(index))))
	plan_row.add_child(party_option)
	hero_caption = UiTheme.label("Hero", fs, UiTheme.COL_MUTED)
	plan_row.add_child(hero_caption)
	hero_option = _make_option("This hero's tactic (the hero whose track is shown). Editing the row shows Custom (edited).", true, fs)
	hero_option.item_selected.connect(func(index: int) -> void: _on_option_picked(false, str(hero_option.get_item_metadata(index))))
	plan_row.add_child(hero_option)
	autoplay_btn = Button.new()
	autoplay_btn.toggle_mode = true
	autoplay_btn.tooltip_text = "Autoplay: the party drafts, confirms and plays every round on its own [A]. Space pauses it."
	_style_button(autoplay_btn, fs)
	autoplay_btn.toggled.connect(func(on: bool) -> void:
		if DictUtil.to_bool(autoplay_state.get("on", false)) != on:
			autoplay_toggled.emit(on)
	)
	plan_row.add_child(autoplay_btn)
	clear_btn = Button.new()
	clear_btn.tooltip_text = "Remove this hero's whole sequence"
	_style_button(clear_btn, fs)
	clear_btn.pressed.connect(func() -> void: clear_requested.emit())
	plan_row.add_child(clear_btn)
	confirm_btn = Button.new()
	confirm_btn.tooltip_text = "Lock every hero's sequence and start the round [Enter]. Right-click the grid or press Backspace to undo the last step."
	_style_button(confirm_btn, fs, UiTheme.COL_WARN)
	confirm_btn.pressed.connect(func() -> void: confirm_requested.emit())
	plan_row.add_child(confirm_btn)
	_apply_labels()

	# 3. Right Round Traversal Arrow [ ▶ ] (Spans vertical height of ruler + action track)
	btn_next_round_arrow = Button.new()
	btn_next_round_arrow.text = "▶"
	btn_next_round_arrow.tooltip_text = "Next Round Timeline"
	btn_next_round_arrow.custom_minimum_size = Vector2(22, 54)
	btn_next_round_arrow.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_style_button(btn_next_round_arrow, fs)
	btn_next_round_arrow.pressed.connect(func() -> void: round_nav_requested.emit(1))
	timeline_flank_hbox.add_child(btn_next_round_arrow)

	# Hostile timeline mask: one line in the track row's place.
	enemy_classified_banner = PanelContainer.new()
	enemy_classified_banner.visible = false
	enemy_classified_banner.add_theme_stylebox_override("panel", UiTheme.box(Color(0.12, 0.08, 0.10, 0.95), Color(0.65, 0.25, 0.25, 0.8), 6, 8, 1))
	enemy_classified_banner.custom_minimum_size = Vector2(0, int(m["clip_h"]))
	tracks_container.add_child(enemy_classified_banner)
	tracks_container.move_child(enemy_classified_banner, single_track_row.get_index() + 1)

	var banner_hbox := HBoxContainer.new()
	banner_hbox.add_theme_constant_override("separation", 10)
	enemy_classified_banner.add_child(banner_hbox)

	banner_lbl = UiTheme.label(BANNER_TEXT, fs, Color(1.0, 0.45, 0.45))
	banner_lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	banner_lbl.clip_text = true
	banner_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	banner_hbox.add_child(banner_lbl)

	var btn_view_party := Button.new()
	btn_view_party.text = "View Hero Track"
	_style_button(btn_view_party, fs)
	btn_view_party.pressed.connect(func() -> void: hero_view_requested.emit())
	banner_hbox.add_child(btn_view_party)
	_apply_heights()

# Button captions at the current text size: abbreviated above 1.1x (rule 8) instead of clipped.
func _apply_labels() -> void:
	var short: bool = DictUtil.to_bool(_m["short"])
	clear_btn.text = "Clear" if short else "🗑 Clear"
	confirm_btn.text = "✔ Confirm" if short else "✔ CONFIRM ROUND"
	party_caption.visible = not short
	hero_caption.visible = not short
	var on: bool = DictUtil.to_bool(autoplay_state.get("on", false))
	autoplay_btn.text = ("⏸ Auto" if on else "▶ Auto") if short else ("⏸ Autoplay" if on else "▶ Autoplay")

func _apply_heights() -> void:
	_m = row_metrics()
	var m: Dictionary = _m
	enemy_classified_banner.custom_minimum_size = Vector2(0, int(m["clip_h"]))
	for tick: PanelContainer in ruler_tick_nodes:
		tick.custom_minimum_size.y = int(m["tick_h"])

# (Re)builds the BLK ruler for the current round length.
func _build_ruler() -> void:
	_m = row_metrics()
	for child: Node in tracks_ruler_row.get_children():
		tracks_ruler_row.remove_child(child)
		child.queue_free()
	ruler_tick_nodes.clear()
	ruler_tick_labels.clear()
	for b: int in range(1, total_blocks + 1):
		var tick_panel := PanelContainer.new()
		tick_panel.custom_minimum_size = Vector2(block_w, int(_m["tick_h"]))
		tick_panel.clip_contents = true
		var tick_style := StyleBoxFlat.new()
		tick_style.bg_color = Color(0.10, 0.13, 0.18, 0.8)
		tick_style.border_color = Color(0.2, 0.28, 0.38, 0.8)
		tick_style.set_border_width_all(1)
		tick_style.set_corner_radius_all(3)
		tick_style.content_margin_left = 2
		tick_style.content_margin_right = 2
		tick_style.content_margin_top = 1
		tick_style.content_margin_bottom = 1
		tick_panel.add_theme_stylebox_override("panel", tick_style)

		var tick_lbl := Label.new()
		tick_lbl.text = "BLK %02d" % b
		tick_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		tick_lbl.add_theme_font_size_override("font_size", UiTheme.hud_font())
		tick_lbl.add_theme_color_override("font_color", Color(0.65, 0.75, 0.85))
		tick_lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
		tick_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		tick_panel.add_child(tick_lbl)

		var cur_b: int = b
		tick_panel.gui_input.connect(func(event: InputEvent) -> void:
			var mouse_event := event as InputEventMouseButton
			if mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT:
				block_selected.emit(cur_b)
		)

		tracks_ruler_row.add_child(tick_panel)
		ruler_tick_nodes.append(tick_panel)
		ruler_tick_labels.append(tick_lbl)

func set_total_blocks(n: int) -> void:
	if n == total_blocks:
		return
	total_blocks = n
	if not _fit_blocks():
		block_w = int(float(TRACK_WIDTH - (n - 1) * BLOCK_GAP) / float(n))
		_build_ruler()

# Sizes blocks to the width this dock actually has (it may sit in a narrow slot).
# Returns true when the block width changed (ruler rebuilt; clips need a redraw).
func _fit_blocks() -> bool:
	var avail: float = size.x - 60.0   # round arrows + spacing
	if avail <= 0.0:
		return false
	var w: int = clampi(int((avail - float((total_blocks - 1) * BLOCK_GAP)) / float(total_blocks)), 36, 140)
	if w == block_w:
		return false
	block_w = w
	_build_ruler()
	return true

# Control row state: info is CombatSession.plan_info(); step_fn maps a block to its step index. The row is always
# there (fixed height); outside planning only the Autoplay toggle and the autoplay note show.
func set_planning(info: Dictionary, step_fn: Callable) -> void:
	_m = row_metrics()
	planning = DictUtil.to_bool(info.get("planning", false))
	plan_step_fn = step_fn
	for node: Control in [party_caption, party_option, hero_caption, hero_option, clear_btn, confirm_btn]:
		var tactic_control: bool = node == party_caption or node == party_option or node == hero_caption or node == hero_option
		node.visible = planning and not (node is Label and DictUtil.to_bool(_m["short"])) and (autoplay_enabled or not tactic_control)
	autoplay_btn.visible = autoplay_enabled
	if not planning:
		_cancel_redraft()
	var warn: bool = DictUtil.to_bool(info.get("warn", false))
	plan_info_lbl.text = str(info.get("text", "")) if planning else ""
	plan_info_lbl.add_theme_color_override("font_color", Color(1.0, 0.5, 0.4) if warn else Color(0.98, 0.78, 0.3))
	if planning:
		confirm_btn.disabled = not DictUtil.to_bool(info.get("can_confirm", false))
	_show_note()

# AUTO1 state for the controls (CombatTimelinePanel builds it from CombatAutoplay): party, row (the shown hero's
# dropdown value), edited (the row shows Custom (edited)), any_edited, guest (no row picker), has_hero, on, note, note_warn.
func set_autoplay_state(state: Dictionary) -> void:
	autoplay_state = state
	var on: bool = DictUtil.to_bool(state.get("on", false))
	autoplay_btn.set_pressed_no_signal(on)
	_apply_labels()
	_select_id(party_option, str(state.get("party", PlanPresets.DEFAULT_TACTIC)))
	var edited: bool = DictUtil.to_bool(state.get("edited", false))
	var custom_idx: int = _index_of(hero_option, PlanPresets.CUSTOM)
	if custom_idx >= 0:
		hero_option.set_item_text(custom_idx, PlanPresets.label(PlanPresets.CUSTOM) + (" (edited)" if edited else ""))
	_select_id(hero_option, str(state.get("row", PlanPresets.DEFAULT_TACTIC)))
	var guest: bool = DictUtil.to_bool(state.get("guest", false))
	var has_hero: bool = DictUtil.to_bool(state.get("has_hero", false))
	hero_option.disabled = guest or not has_hero
	hero_option.tooltip_text = "Guests always play Balanced." if guest else "This hero's tactic (the hero whose track is shown). Editing the row shows Custom (edited)."
	_show_note()

# The autoplay pause / wait reason takes the info label's place when there is one (a short banner: why it stopped).
# Replay and playback arenas play recorded rounds: nothing there may draft or confirm live rounds.
func set_autoplay_enabled(enabled: bool) -> void:
	autoplay_enabled = enabled
	autoplay_btn.visible = enabled
	if not enabled:
		autoplay_state = {}
		_cancel_redraft()

func _show_note() -> void:
	var note: String = str(autoplay_state.get("note", ""))
	if note.is_empty():
		plan_info_lbl.tooltip_text = plan_info_lbl.text
		return
	plan_info_lbl.text = note
	plan_info_lbl.tooltip_text = note
	plan_info_lbl.add_theme_color_override("font_color", UiTheme.COL_WARN if DictUtil.to_bool(autoplay_state.get("note_warn", true)) else UiTheme.COL_INFO)

func _index_of(opt: OptionButton, id: String) -> int:
	for i: int in range(opt.item_count):
		if str(opt.get_item_metadata(i)) == id:
			return i
	return -1

func _select_id(opt: OptionButton, id: String) -> void:
	var idx: int = _index_of(opt, id)
	if idx >= 0:
		opt.select(idx)

# A dropdown pick. While the row (or any row, for the party dropdown) holds edits, a redraft would throw them away:
# ask first (Redraft / Keep); otherwise it applies at once.
func _on_option_picked(whole_party: bool, id: String) -> void:
	var discards: bool = DictUtil.to_bool(autoplay_state.get("any_edited" if whole_party else "edited", false))
	if discards:
		_pending = {"party": whole_party, "id": id}
		confirm_lbl.text = "Redraft with %s? Your edits are thrown away." % PlanPresets.label(id)
		confirm_row.visible = true
		plan_info_lbl.visible = false
		set_autoplay_state(autoplay_state)   # the dropdown goes back to what is still in force
		return
	_apply_pick(whole_party, id)

func _apply_pick(whole_party: bool, id: String) -> void:
	if whole_party:
		party_tactic_requested.emit(id)
	else:
		hero_tactic_requested.emit(id)

func resolve_redraft(accept: bool) -> void:
	var pending: Dictionary = _pending
	_cancel_redraft()
	if accept and not pending.is_empty():
		_apply_pick(DictUtil.to_bool(pending.get("party", false)), str(pending.get("id", "")))

func _cancel_redraft() -> void:
	_pending = {}
	if confirm_row != null:
		confirm_row.visible = false
		plan_info_lbl.visible = true

func redraft_pending() -> bool:
	return not _pending.is_empty()

func update_round_arrows(can_go_prev: bool, can_go_next: bool) -> void:
	if btn_prev_round != null:
		btn_prev_round.disabled = not can_go_prev
	if btn_next_round_arrow != null:
		btn_next_round_arrow.disabled = not can_go_next

func update_view(
	target_unit: Combatant,
	active_playhead: int,
	is_enemy_masked: bool,
	can_go_prev: bool,
	can_go_next: bool,
	tell: Dictionary = {},
	track_lookup: Callable = Callable()
) -> void:
	_m = row_metrics()
	read_only = track_lookup.is_valid()
	update_round_arrows(can_go_prev, can_go_next)

	# Update Ruler highlights
	for i: int in range(ruler_tick_nodes.size()):
		var panel: PanelContainer = ruler_tick_nodes[i]
		var lbl: Label = ruler_tick_labels[i]
		var b_num: int = i + 1
		var style := StyleBoxFlat.new()
		style.set_corner_radius_all(3)
		style.content_margin_left = 2
		style.content_margin_right = 2
		style.content_margin_top = 1
		style.content_margin_bottom = 1
		if b_num == active_playhead:
			style.bg_color = Color(0.2, 0.45, 0.75, 1.0)
			style.border_color = Color(0.5, 0.9, 1.0, 1.0)
			style.set_border_width_all(2)
			lbl.text = "▼ %02d" % b_num
			lbl.add_theme_color_override("font_color", Color(1.0, 0.95, 0.4))
		else:
			style.bg_color = Color(0.10, 0.13, 0.18, 0.8)
			style.border_color = Color(0.2, 0.28, 0.38, 0.8)
			style.set_border_width_all(1)
			lbl.text = "BLK %02d" % b_num
			lbl.add_theme_color_override("font_color", Color(0.65, 0.75, 0.85))
		panel.add_theme_stylebox_override("panel", style)

	# Handle enemy masked planning phase banner vs single track row
	if is_enemy_masked:
		# INTENT1-U: the banner stays, with the selected foe's block 1 tell in it
		banner_lbl.text = ("🔒 Hostiles plan in secret. %s next: %s" % [str(tell["name"]), str(tell["line"])]) if not tell.is_empty() else BANNER_TEXT
		enemy_classified_banner.visible = true
		single_track_row.visible = false
		return
	else:
		enemy_classified_banner.visible = false
		single_track_row.visible = true

	# Clear previous clips
	for child: Node in track_clips_hbox.get_children():
		track_clips_hbox.remove_child(child)   # out of the layout at once (queue_free alone keeps it there until the frame ends)
		child.queue_free()

	if target_unit == null:
		return

	# Rebuild continuous clips (Starts at X=0, exactly under Ruler BLK 01..10)
	var b: int = 1
	while b <= total_blocks:
		var act: CombatResolver.ScheduledAction = null
		var lookup: Callable = track_lookup if track_lookup.is_valid() else action_lookup_fn
		if lookup.is_valid():
			var found: Variant = lookup.call(target_unit, b)
			if found is CombatResolver.ScheduledAction:
				act = found

		if act != null:
			var cost: int = act.end_block - act.start_block + 1
			var clip_w: int = cost * block_w + (cost - 1) * BLOCK_GAP
			var clip := _create_action_clip_widget(act, clip_w, cost)
			track_clips_hbox.add_child(clip)
			b += cost
		else:
			var idle_slot := _create_idle_clip_widget(b == active_playhead)
			track_clips_hbox.add_child(idle_slot)
			b += 1

func _create_action_clip_widget(act: CombatResolver.ScheduledAction, width: int, cost: int) -> PanelContainer:
	var clip := PanelContainer.new()
	clip.custom_minimum_size = Vector2(width, int(_m["clip_h"]))
	clip.clip_contents = true
	clip.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND

	var clip_style := StyleBoxFlat.new()
	clip_style.set_corner_radius_all(4)
	clip_style.content_margin_left = 4
	clip_style.content_margin_right = 4
	clip_style.content_margin_top = 1
	clip_style.content_margin_bottom = 1
	clip_style.set_border_width_all(1)

	var icon_str: String = "⚔"
	var subtitle: String = ""
	var cmd_display_name: String = act.command.command_name

	if act.command is AttackCommand:
		clip_style.bg_color = Color(0.45, 0.12, 0.14, 0.95)
		clip_style.border_color = Color(0.9, 0.3, 0.3, 1.0)
		icon_str = "⚔"
		var atk: AttackCommand = act.command as AttackCommand
		subtitle = "🎯 %s" % atk.target.name if atk != null and atk.target != null else "🎯 Strike"
		if atk != null and act.command.est_damage > 0:
			subtitle += " ~%d dmg" % act.command.est_damage
	elif act.command is DefendCommand:
		clip_style.bg_color = Color(0.12, 0.25, 0.48, 0.95)
		clip_style.border_color = Color(0.3, 0.6, 0.95, 1.0)
		icon_str = "🛡"
		subtitle = "Guard -%d%%" % roundi((act.command as DefendCommand).mitigation_ratio * 100.0)
	elif act.command is FocusCommand:
		clip_style.bg_color = Color(0.32, 0.14, 0.48, 0.95)
		clip_style.border_color = Color(0.75, 0.4, 0.95, 1.0)
		icon_str = "✦"
		subtitle = "x%.2f ATK (%d rds)" % [(act.command as FocusCommand).boost_multiplier, (act.command as FocusCommand).duration_rounds]
	elif act.command is WalkCommand:
		clip_style.bg_color = Color(0.08, 0.32, 0.44, 0.95)
		clip_style.border_color = Color(0.2, 0.8, 0.95, 1.0)
		icon_str = "➜"
		var walk_cmd: WalkCommand = act.command as WalkCommand
		var tiles: int = walk_cmd.path.size()
		if walk_cmd.approach_target != null:
			subtitle = "→ %s (%d)" % [walk_cmd.approach_target.name, tiles]
		else:
			subtitle = "%s %d tile%s" % [PlanEntry.dir_arrow(walk_cmd.direction()), tiles, "" if tiles == 1 else "s"]
	elif act.command is SupportCommand:
		clip_style.bg_color = Color(0.1, 0.34, 0.2, 0.95)
		clip_style.border_color = Color(0.35, 0.9, 0.5, 1.0)
		icon_str = "+"
		var support_cmd: SupportCommand = act.command as SupportCommand
		subtitle = "→ %s" % support_cmd.target.name if support_cmd.target != null else "Support"
		if support_cmd.est_heal > 0:
			subtitle += " ~+%d HP" % support_cmd.est_heal
		elif support_cmd.skill != null and not support_cmd.skill.grant_status.is_empty():
			subtitle += " %s" % StatusEffects.display_name(support_cmd.skill.grant_status)
	elif act.command is StatusBlockCommand:
		clip_style.bg_color = Color(0.25, 0.08, 0.08, 0.95)
		clip_style.border_color = Color(0.7, 0.25, 0.25, 1.0)
		icon_str = "✖"
		subtitle = "block lost"
	elif act.command is AreaAttackCommand:
		clip_style.bg_color = Color(0.45, 0.2, 0.08, 0.95)
		clip_style.border_color = Color(1.0, 0.55, 0.2, 1.0)
		icon_str = "✸"
		subtitle = "%d tiles" % (act.command as AreaAttackCommand).cells.size()
	elif act.command is RestCommand:
		clip_style.bg_color = Color(0.1, 0.3, 0.2, 0.95)
		clip_style.border_color = Color(0.3, 0.85, 0.5, 1.0)
		icon_str = "💤"
		subtitle = "+AP, Tactical"
	elif act.command is TargetCommand:
		clip_style.bg_color = Color(0.08, 0.38, 0.24, 0.95)
		clip_style.border_color = Color(0.25, 0.85, 0.5, 1.0)
		icon_str = "🎯"
		subtitle = "Focus Lock"
	else:
		clip_style.bg_color = Color(0.22, 0.26, 0.34, 0.95)
		clip_style.border_color = Color(0.5, 0.6, 0.7, 1.0)
		icon_str = "·"
		subtitle = "Action"

	clip.add_theme_stylebox_override("panel", clip_style)

	var vbox := VBoxContainer.new()
	vbox.alignment = BoxContainer.ALIGNMENT_CENTER
	vbox.add_theme_constant_override("separation", 0)
	vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	clip.add_child(vbox)

	var title_hbox := HBoxContainer.new()
	title_hbox.add_theme_constant_override("separation", 2)
	title_hbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	vbox.add_child(title_hbox)

	var name_lbl := Label.new()
	name_lbl.text = "%s %s" % [icon_str, cmd_display_name]
	name_lbl.add_theme_font_size_override("font_size", UiTheme.hud_font())
	name_lbl.add_theme_color_override("font_color", Color.WHITE)
	name_lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	name_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	title_hbox.add_child(name_lbl)

	var cost_lbl := Label.new()
	cost_lbl.text = "[%db · %dAP]" % [cost, act.command.ap_cost + act.command.stance_switch_cost]
	cost_lbl.add_theme_font_size_override("font_size", UiTheme.hud_font())
	cost_lbl.add_theme_color_override("font_color", Color(1.0, 0.9, 0.5))
	title_hbox.add_child(cost_lbl)

	var sub_lbl := Label.new()
	sub_lbl.text = subtitle
	sub_lbl.add_theme_font_size_override("font_size", UiTheme.hud_font())
	sub_lbl.add_theme_color_override("font_color", Color(0.8, 0.85, 0.9))
	sub_lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	sub_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	sub_lbl.visible = DictUtil.to_bool(_m["two_line"])   # the first line to drop as text grows
	vbox.add_child(sub_lbl)

	var cur_c: Combatant = act.combatant
	var start_b: int = act.start_block
	clip.gui_input.connect(func(event: InputEvent) -> void:
		var mouse_event := event as InputEventMouseButton
		if mouse_event == null or not mouse_event.pressed:
			return
		if planning and not read_only and mouse_event.button_index == MOUSE_BUTTON_RIGHT:
			var step_idx: int = _step_at(start_b)
			if step_idx >= 0:
				step_remove_requested.emit(step_idx)
		elif mouse_event.button_index == MOUSE_BUTTON_LEFT:
			combatant_selected.emit(cur_c)
	)
	if planning and not read_only:
		clip.tooltip_text = "%s%s\nAP %d • drag onto another command to reorder • right-click to remove" % [cmd_display_name, ("\nExpected hit ~%d damage" % act.command.est_damage) if act.command.est_damage > 0 else "", act.command.ap_cost + act.command.stance_switch_cost]
		clip.set_drag_forwarding(
			func(_pos: Vector2) -> Variant:
				var from_step: int = _step_at(start_b)
				if from_step < 0:
					return null
				var preview := Label.new()
				preview.text = "  ↕ %s" % cmd_display_name
				clip.set_drag_preview(preview)
				return {"timeline_step": from_step},
			func(_pos: Vector2, data: Variant) -> bool:
				if not (data is Dictionary):
					return false
				var d: Dictionary = data
				return DictUtil.int_of(d, "timeline_step", -1) >= 0 and DictUtil.int_of(d, "timeline_step", -1) != _step_at(start_b),
			func(_pos: Vector2, data: Variant) -> void:
				if data is Dictionary:
					var d: Dictionary = data
					step_reorder_requested.emit(DictUtil.int_of(d, "timeline_step", -1), _step_at(start_b))
		)

	return clip

func _create_idle_clip_widget(is_playhead: bool, width: int = -1) -> PanelContainer:
	var slot_w: int = width if width > 0 else block_w
	var slot := PanelContainer.new()
	slot.custom_minimum_size = Vector2(slot_w, int(_m["clip_h"]))
	slot.clip_contents = true

	var slot_style := StyleBoxFlat.new()
	slot_style.set_corner_radius_all(3)
	if is_playhead:
		slot_style.bg_color = Color(0.15, 0.22, 0.32, 0.85)
		slot_style.border_color = Color(0.4, 0.7, 0.95, 0.9)
		slot_style.set_border_width_all(1)
	else:
		slot_style.bg_color = Color(0.08, 0.10, 0.14, 0.6)
		slot_style.border_color = Color(0.18, 0.24, 0.32, 0.5)
		slot_style.set_border_width_all(1)
	slot.add_theme_stylebox_override("panel", slot_style)

	var lbl := Label.new()
	lbl.text = "· Idle"
	lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	lbl.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	lbl.add_theme_font_size_override("font_size", UiTheme.hud_font())
	lbl.add_theme_color_override("font_color", Color(0.4, 0.5, 0.6))
	lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	slot.add_child(lbl)

	return slot

func _step_at(block: int) -> int:
	if not plan_step_fn.is_valid():
		return -1
	var raw: Variant = plan_step_fn.call(block)
	return raw if raw is int else -1
