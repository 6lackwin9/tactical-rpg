class_name TimelineDockUI
extends Control

# ==============================================================================
# Tactical RPG - Planning strip (COMBAT-UI3, layout A; replaces the two-row timeline of COMBAT-UI2-L3)
# One slim row: ◀ round arrow, "<Hero> plan", the shown unit's round as continuous blocks (width by blocks, colour by
# action kind (PlanBlockText), empty blocks muted), then the tactic controls in the freed space (Party and Hero
# tactic dropdowns, Autoplay, Clear, Confirm round) and the ▶ round arrow. Hover or focus on a block shows a small
# card with how the action plays out. Confirm and a discarding tactic pick ask through a small non-blocking question
# (StripQuestion). Enemy plans stay masked during the live round (the banner holds the block 1 tell). Sizes come
# from CombatHudMetrics; all text follows UiTheme.text_scale() (14 px floor at Normal); above SHORT_ABOVE the
# dropdown captions drop and the buttons shorten instead of clipping.
# ==============================================================================

signal round_nav_requested(delta: int)
signal block_selected(block: int)
signal combatant_selected(c: Combatant)
signal hero_view_requested()
signal party_tactic_requested(tactic_id: String)               # the party dropdown (every hero at once)
signal hero_tactic_requested(tactic_id: String)                # the hero dropdown (the hero whose track is shown)
signal autoplay_toggled(on: bool)
signal clear_requested()                                       # planning: clear the hero's sequence
signal confirm_requested()                                     # planning: the Confirm round button (the owner decides whether to ask)
signal missing_confirmed()                                     # the missing-actions question was answered Confirm
signal step_remove_requested(step_index: int)
signal step_reorder_requested(from_step: int, to_step: int)
signal fit_changed()            # block width changed to fit the strip: the owner should redraw

const BANNER_TEXT: String = "🔒 Hostiles plan in secret; see the enemy card for their next action."
const Q_CONFIRM: String = "confirm"
const Q_REDRAFT: String = "redraft"

var btn_prev_round: Button
var btn_next_round_arrow: Button
var strip_panel: PanelContainer
var strip_row: HBoxContainer
var title_lbl: Label
var blocks_area: Control                 # the blocks' room: whatever the controls leave (clips, never pushes)
var track_clips_hbox: HBoxContainer
var total_blocks: int = 10
var block_w: int = 80
var controls_row: HBoxContainer
var party_option: OptionButton
var hero_option: OptionButton
var party_caption: Label
var hero_caption: Label
var autoplay_btn: Button
var clear_btn: Button
var confirm_btn: Button
var question: StripQuestion              # Confirm round's missing-actions warning and the redraft question
var confirm_yes_btn: Button              # the question's buttons (Redraft / Confirm and Keep / Back)
var confirm_no_btn: Button
var block_card: PanelContainer           # the hover / focus card of one block
var block_card_title: Label
var block_card_body: Label
var block_card_cost: Label
var autoplay_state: Dictionary = {}
var autoplay_enabled: bool = true   # false in replay / playback arenas: no tactic dropdowns, no Autoplay toggle
var _pending: Dictionary = {}       # a tactic pick waiting for the discard question: {party: bool, id: String}

var planning: bool = false
var plan_step_fn: Callable   # func(block: int) -> int: index of the step behind the command at a block
var enemy_classified_banner: PanelContainer
var banner_lbl: Label
var shown_unit: Combatant = null
var active_playhead: int = 0

# Callable: func(combatant: Combatant, block: int) -> CombatResolver.ScheduledAction
var action_lookup_fn: Callable
var read_only: bool = false   # INTENT1-U: the shown track is a read enemy plan: no drag, no remove

func _init() -> void:
	name = "TimelineDockUI"
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	size_flags_vertical = Control.SIZE_EXPAND_FILL
	_build_ui()

func fs() -> int:
	return UiTheme.hud_font()

func ctrl_h() -> int:
	return CombatHudMetrics.control_height(fs())

func short_labels() -> bool:
	return UiTheme.text_scale() > CombatHudMetrics.SHORT_ABOVE

static func _btn_style(bg: Color, border: Color) -> StyleBoxFlat:
	return UiTheme.box(bg, border, 4, 6, 2)

# Flat, fixed-height look for every control of the strip (the default button styles are taller than a text line).
func _style_button(btn: Button, accent: Color = Color(0.38, 0.5, 0.68, 0.9)) -> void:
	btn.focus_mode = Control.FOCUS_NONE
	btn.add_theme_font_size_override("font_size", fs())
	btn.custom_minimum_size.y = ctrl_h()
	btn.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	btn.add_theme_stylebox_override("normal", _btn_style(Color(0.10, 0.14, 0.20, 0.95), accent))
	btn.add_theme_stylebox_override("hover", _btn_style(Color(0.16, 0.22, 0.32, 0.95), Color(0.55, 0.75, 1.0)))
	btn.add_theme_stylebox_override("pressed", _btn_style(Color(0.12, 0.30, 0.20, 0.95), UiTheme.COL_GOOD))
	btn.add_theme_stylebox_override("disabled", _btn_style(Color(0.08, 0.10, 0.14, 0.7), Color(0.2, 0.26, 0.34, 0.6)))
	btn.add_theme_stylebox_override("focus", StyleBoxEmpty.new())

func _make_option(tip: String, with_custom: bool) -> OptionButton:
	var opt := OptionButton.new()
	opt.tooltip_text = tip
	opt.clip_text = true
	opt.fit_to_longest_item = false
	opt.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	opt.custom_minimum_size = Vector2(roundf(float(fs()) * CombatHudMetrics.OPTION_EM), 0)
	_style_button(opt)
	opt.get_popup().add_theme_font_size_override("font_size", fs())
	for entry: Dictionary in CombatAutoplay.tactic_entries(with_custom):
		opt.add_item(str(entry["label"]))
		var idx: int = opt.item_count - 1
		opt.set_item_metadata(idx, str(entry["id"]))
		opt.set_item_tooltip(idx, str(entry["description"]))
	return opt

func _build_ui() -> void:
	strip_panel = PanelContainer.new()
	strip_panel.name = "StripPanel"
	strip_panel.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	strip_panel.add_theme_stylebox_override("panel", UiTheme.box(Color(0.08, 0.11, 0.18, 0.97), Color(0.16, 0.23, 0.37, 1.0), 6,
		CombatHudMetrics.STRIP_PAD_H, CombatHudMetrics.STRIP_PAD_V))
	add_child(strip_panel)
	strip_row = HBoxContainer.new()
	strip_row.name = "StripRow"
	strip_row.add_theme_constant_override("separation", CombatHudMetrics.STRIP_SEP)
	strip_panel.add_child(strip_row)

	btn_prev_round = _arrow("◀", "Previous round", -1)
	strip_row.add_child(btn_prev_round)

	title_lbl = UiTheme.label("", fs(), UiTheme.COL_TITLE)
	title_lbl.name = "StripTitle"
	title_lbl.clip_text = true
	title_lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	title_lbl.custom_minimum_size = Vector2(roundf(float(fs()) * CombatHudMetrics.TITLE_EM), 0)
	title_lbl.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	strip_row.add_child(title_lbl)

	# The blocks' room: a plain Control (not a container), so wide blocks clip instead of widening the strip.
	blocks_area = Control.new()
	blocks_area.name = "BlocksArea"
	blocks_area.clip_contents = true
	blocks_area.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	blocks_area.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	blocks_area.custom_minimum_size = Vector2(CombatHudMetrics.BLOCK_MIN_W, ctrl_h())
	blocks_area.resized.connect(func() -> void:
		if _fit_blocks():
			fit_changed.emit()
	)
	strip_row.add_child(blocks_area)
	track_clips_hbox = HBoxContainer.new()
	track_clips_hbox.name = "ClipsHBox"
	track_clips_hbox.add_theme_constant_override("separation", CombatHudMetrics.BLOCK_GAP)
	blocks_area.add_child(track_clips_hbox)

	# Hostile timeline mask: one line in the blocks' place.
	enemy_classified_banner = PanelContainer.new()
	enemy_classified_banner.name = "MaskBanner"
	enemy_classified_banner.visible = false
	enemy_classified_banner.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	enemy_classified_banner.add_theme_stylebox_override("panel", UiTheme.box(Color(0.12, 0.08, 0.10, 0.95), Color(0.65, 0.25, 0.25, 0.8), 4, 6, 1))
	blocks_area.add_child(enemy_classified_banner)
	var banner_hbox := HBoxContainer.new()
	banner_hbox.add_theme_constant_override("separation", 8)
	enemy_classified_banner.add_child(banner_hbox)
	banner_lbl = UiTheme.label(BANNER_TEXT, fs(), Color(1.0, 0.45, 0.45))
	banner_lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	banner_lbl.clip_text = true
	banner_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	banner_lbl.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	banner_hbox.add_child(banner_lbl)
	var btn_view_party := Button.new()
	btn_view_party.name = "ViewHeroTrack"
	btn_view_party.text = "Hero plan"
	btn_view_party.tooltip_text = "Show the hero's plan again"
	_style_button(btn_view_party)
	btn_view_party.custom_minimum_size.y = 0
	btn_view_party.pressed.connect(func() -> void: hero_view_requested.emit())
	banner_hbox.add_child(btn_view_party)

	# The tactic controls (the space the old second row freed).
	controls_row = HBoxContainer.new()
	controls_row.name = "StripControls"
	controls_row.add_theme_constant_override("separation", CombatHudMetrics.STRIP_SEP)
	strip_row.add_child(controls_row)
	party_caption = UiTheme.label("Party", fs(), UiTheme.COL_MUTED)
	party_caption.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	controls_row.add_child(party_caption)
	party_option = _make_option("Party tactic: every hero drafts with it (clears the hero overrides)", false)
	party_option.item_selected.connect(func(index: int) -> void: _on_option_picked(true, str(party_option.get_item_metadata(index))))
	controls_row.add_child(party_option)
	hero_caption = UiTheme.label("Hero", fs(), UiTheme.COL_MUTED)
	hero_caption.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	controls_row.add_child(hero_caption)
	hero_option = _make_option("This hero's tactic (the hero whose plan is shown). Editing the plan shows Custom (edited).", true)
	hero_option.item_selected.connect(func(index: int) -> void: _on_option_picked(false, str(hero_option.get_item_metadata(index))))
	controls_row.add_child(hero_option)
	autoplay_btn = Button.new()
	autoplay_btn.name = "AutoplayButton"
	autoplay_btn.toggle_mode = true
	autoplay_btn.tooltip_text = "Autoplay: the party drafts, confirms and plays every round on its own [A]. Space pauses it."
	_style_button(autoplay_btn)
	autoplay_btn.toggled.connect(func(on: bool) -> void:
		if DictUtil.to_bool(autoplay_state.get("on", false)) != on:
			autoplay_toggled.emit(on)
	)
	controls_row.add_child(autoplay_btn)
	clear_btn = Button.new()
	clear_btn.name = "ClearButton"
	clear_btn.tooltip_text = "Remove this hero's whole plan"
	_style_button(clear_btn)
	clear_btn.pressed.connect(func() -> void: clear_requested.emit())
	controls_row.add_child(clear_btn)
	confirm_btn = Button.new()
	confirm_btn.name = "ConfirmButton"
	confirm_btn.tooltip_text = "Lock every hero's plan and start the round [Enter]. Right-click the grid or press Backspace to undo the last step."
	_style_button(confirm_btn, UiTheme.COL_WARN)
	confirm_btn.add_theme_color_override("font_color", UiTheme.COL_WARN)
	confirm_btn.pressed.connect(func() -> void: confirm_requested.emit())
	controls_row.add_child(confirm_btn)
	_apply_labels()

	btn_next_round_arrow = _arrow("▶", "Next round", 1)
	strip_row.add_child(btn_next_round_arrow)

	question = StripQuestion.new()
	question.answered.connect(_on_answered)
	add_child(question)
	confirm_yes_btn = question.yes_btn
	confirm_no_btn = question.no_btn
	_build_block_card()

func _arrow(text: String, tip: String, delta: int) -> Button:
	var btn := Button.new()
	btn.text = text
	btn.tooltip_text = tip
	_style_button(btn)
	btn.custom_minimum_size.x = CombatHudMetrics.ARROW_W
	btn.pressed.connect(func() -> void: round_nav_requested.emit(delta))
	return btn

func _build_block_card() -> void:
	block_card = PanelContainer.new()
	block_card.name = "BlockCard"
	block_card.visible = false
	block_card.top_level = true
	block_card.z_index = 65
	block_card.mouse_filter = Control.MOUSE_FILTER_IGNORE
	block_card.add_theme_stylebox_override("panel", UiTheme.box(Color(0.04, 0.06, 0.11, 0.98), UiTheme.COL_WARN, 4, 10, 8))
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 2)
	col.mouse_filter = Control.MOUSE_FILTER_IGNORE
	block_card.add_child(col)
	block_card_title = UiTheme.label("", fs(), UiTheme.COL_TITLE)
	block_card_body = UiTheme.label("", fs(), UiTheme.COL_TEXT)
	block_card_cost = UiTheme.label("", fs(), UiTheme.COL_MUTED)
	for lbl: Label in [block_card_title, block_card_body, block_card_cost]:
		lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		lbl.mouse_filter = Control.MOUSE_FILTER_IGNORE
		col.add_child(lbl)
	add_child(block_card)

# Button captions at the current text size: abbreviated above SHORT_ABOVE instead of clipped.
func _apply_labels() -> void:
	var short: bool = short_labels()
	clear_btn.text = "Clear"
	confirm_btn.text = "✔ Confirm" if short else "✔ Confirm round"
	party_caption.visible = party_option.visible and not short
	hero_caption.visible = hero_option.visible and not short
	var on: bool = DictUtil.to_bool(autoplay_state.get("on", false))
	autoplay_btn.text = ("⏸ Auto" if on else "▶ Auto") if short else ("⏸ Autoplay" if on else "▶ Autoplay")

func set_title(text: String) -> void:
	title_lbl.text = text
	title_lbl.tooltip_text = text

func set_total_blocks(n: int) -> void:
	if n == total_blocks:
		return
	total_blocks = n
	_fit_blocks()

# Sizes the blocks to the room the controls leave (the strip may sit in a narrow slot).
# Returns true when the block width changed (the owner redraws the blocks).
func _fit_blocks() -> bool:
	var avail: float = blocks_area.size.x
	if avail <= 0.0:
		return false
	var n: int = maxi(1, total_blocks)
	var w: int = clampi(int((avail - float((n - 1) * CombatHudMetrics.BLOCK_GAP)) / float(n)), CombatHudMetrics.BLOCK_MIN_W, CombatHudMetrics.BLOCK_MAX_W)
	if w == block_w:
		return false
	block_w = w
	return true

# Control state: info is CombatSession.plan_info(); step_fn maps a block to its step index. Outside planning only
# the Autoplay toggle shows (the strip keeps its height).
func set_planning(info: Dictionary, step_fn: Callable) -> void:
	planning = DictUtil.to_bool(info.get("planning", false))
	plan_step_fn = step_fn
	for node: Control in [party_option, hero_option, clear_btn, confirm_btn]:
		var tactic_control: bool = node == party_option or node == hero_option
		node.visible = planning and (autoplay_enabled or not tactic_control)
	autoplay_btn.visible = autoplay_enabled
	_apply_labels()
	if not planning:
		_cancel_redraft()
		if question.kind == Q_CONFIRM:
			question.close()
	else:
		confirm_btn.disabled = not DictUtil.to_bool(info.get("can_confirm", false))

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
	hero_option.tooltip_text = "Guests always play Balanced." if guest else "This hero's tactic (the hero whose plan is shown). Editing the plan shows Custom (edited)."

# Replay and playback arenas play recorded rounds: nothing there may draft or confirm live rounds.
func set_autoplay_enabled(enabled: bool) -> void:
	autoplay_enabled = enabled
	autoplay_btn.visible = enabled
	if not enabled:
		autoplay_state = {}
		_cancel_redraft()

func _index_of(opt: OptionButton, id: String) -> int:
	for i: int in range(opt.item_count):
		if str(opt.get_item_metadata(i)) == id:
			return i
	return -1

func _select_id(opt: OptionButton, id: String) -> void:
	var idx: int = _index_of(opt, id)
	if idx >= 0:
		opt.select(idx)

# ------------------------------------------------------------------------------
# Questions: a tactic pick that would throw edits away, and Confirm round with empty blocks
# ------------------------------------------------------------------------------

# A dropdown pick. While the row (or any row, for the party dropdown) holds edits, a redraft would throw them away:
# ask first (Redraft / Keep); otherwise it applies at once.
func _on_option_picked(whole_party: bool, id: String) -> void:
	var discards: bool = DictUtil.to_bool(autoplay_state.get("any_edited" if whole_party else "edited", false))
	if discards:
		_pending = {"party": whole_party, "id": id}
		var anchor: Control = party_option if whole_party else hero_option
		question.ask(Q_REDRAFT, "Redraft with %s? Your edits are thrown away." % PlanPresets.label(id), "Redraft", "Keep", anchor.get_global_rect())
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
	if question != null and question.kind == Q_REDRAFT:
		question.close()

func redraft_pending() -> bool:
	return not _pending.is_empty()

# COMBAT-UI3: the missing-actions warning under the Confirm button ("Warrior and Ranger have empty actions. Confirm anyway?").
func ask_missing_actions(names: Array[String]) -> void:
	_cancel_redraft()
	question.ask(Q_CONFIRM, missing_text(names), "Confirm", "Back", confirm_btn.get_global_rect())

static func missing_text(names: Array[String]) -> String:
	var who: String = names[0] if names.size() == 1 else "%s and %s" % [", ".join(PackedStringArray(names.slice(0, names.size() - 1))), names[names.size() - 1]]
	return "%s %s empty actions. Confirm anyway?" % [who, "has" if names.size() == 1 else "have"]

func question_open() -> bool:
	return question.is_open()

# Esc / a cancel: the open question closes as if answered no (Back / Keep).
func cancel_question() -> bool:
	if not question.is_open():
		return false
	var asked: String = question.kind
	question.close()
	_on_answered(asked, false)
	return true

func _on_answered(asked: String, accept: bool) -> void:
	if asked == Q_REDRAFT:
		resolve_redraft(accept)
	elif asked == Q_CONFIRM and accept:
		missing_confirmed.emit()

func update_round_arrows(can_go_prev: bool, can_go_next: bool) -> void:
	btn_prev_round.disabled = not can_go_prev
	btn_next_round_arrow.disabled = not can_go_next

# ------------------------------------------------------------------------------
# Blocks
# ------------------------------------------------------------------------------

func update_view(
	target_unit: Combatant,
	p_playhead: int,
	is_enemy_masked: bool,
	can_go_prev: bool,
	can_go_next: bool,
	tell: Dictionary = {},
	track_lookup: Callable = Callable()
) -> void:
	read_only = track_lookup.is_valid()
	active_playhead = p_playhead
	shown_unit = target_unit
	update_round_arrows(can_go_prev, can_go_next)
	hide_block_card()
	for child: Node in track_clips_hbox.get_children():
		track_clips_hbox.remove_child(child)   # out of the layout at once (queue_free alone keeps it there until the frame ends)
		child.queue_free()

	# INTENT1-U: a masked enemy keeps the banner, with its block 1 tell in it
	enemy_classified_banner.visible = is_enemy_masked
	track_clips_hbox.visible = not is_enemy_masked
	if is_enemy_masked:
		banner_lbl.text = ("🔒 Hostiles plan in secret. %s next: %s" % [str(tell["name"]), str(tell["line"])]) if not tell.is_empty() else BANNER_TEXT
		banner_lbl.tooltip_text = banner_lbl.text
		return
	if target_unit == null:
		return

	# Continuous blocks from block 1: one per action (as wide as its blocks), one muted block per empty block.
	var lookup: Callable = track_lookup if track_lookup.is_valid() else action_lookup_fn
	var b: int = 1
	while b <= total_blocks:
		var act: CombatResolver.ScheduledAction = null
		if lookup.is_valid():
			var found: Variant = lookup.call(target_unit, b)
			if found is CombatResolver.ScheduledAction:
				act = found
		if act != null:
			var cost: int = act.end_block - act.start_block + 1
			track_clips_hbox.add_child(_action_block(act, cost))
			b += cost
		else:
			track_clips_hbox.add_child(_empty_block(b))
			b += 1
	track_clips_hbox.position = Vector2(0, maxf(0.0, (blocks_area.size.y - float(ctrl_h())) * 0.5))

func _block_style(kind: String, lit: bool) -> StyleBoxFlat:
	var style := UiTheme.box(PlanBlockText.bg(kind), UiTheme.COL_WARN if lit else PlanBlockText.border(kind), 4, 6, 1, 2 if lit else 1)
	return style

func _block_label(text: String, color: Color) -> Label:
	var lbl: Label = UiTheme.label(text, fs(), color)
	lbl.clip_text = true
	lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	lbl.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	lbl.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return lbl

func _block_width(cost: int) -> int:
	return cost * block_w + (cost - 1) * CombatHudMetrics.BLOCK_GAP

func _action_block(act: CombatResolver.ScheduledAction, cost: int) -> PanelContainer:
	var kind: String = PlanBlockText.kind_of(act.command)
	var clip := PanelContainer.new()
	clip.name = "Block%d" % act.start_block
	clip.custom_minimum_size = Vector2(_block_width(cost), ctrl_h())
	clip.clip_contents = true
	clip.focus_mode = Control.FOCUS_ALL
	clip.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	clip.set_meta("kind", kind)
	clip.set_meta("start", act.start_block)
	clip.set_meta("blocks", cost)
	clip.add_theme_stylebox_override("panel", _block_style(kind, active_playhead >= act.start_block and active_playhead <= act.end_block))
	clip.add_child(_block_label(PlanBlockText.label(act), Color.WHITE))

	var cur_c: Combatant = act.combatant
	var start_b: int = act.start_block
	var card: Callable = func() -> void: show_block_card(clip, PlanBlockText.title(act), PlanBlockText.body(act), _cost_text(act))
	clip.mouse_entered.connect(card)
	clip.focus_entered.connect(card)
	clip.mouse_exited.connect(hide_block_card)
	clip.focus_exited.connect(hide_block_card)
	clip.gui_input.connect(func(event: InputEvent) -> void:
		var mouse_event := event as InputEventMouseButton
		if mouse_event == null or not mouse_event.pressed:
			return
		if planning and not read_only and mouse_event.button_index == MOUSE_BUTTON_RIGHT:
			var step_idx: int = _step_at(start_b)
			if step_idx >= 0:
				step_remove_requested.emit(step_idx)
		elif mouse_event.button_index == MOUSE_BUTTON_LEFT:
			if planning:
				combatant_selected.emit(cur_c)
			else:   # resolved rounds: the block under the pointer is inspected
				block_selected.emit(start_b + clampi(int(mouse_event.position.x / float(block_w + CombatHudMetrics.BLOCK_GAP)), 0, cost - 1))
	)
	if planning and not read_only:
		clip.set_drag_forwarding(
			func(_pos: Vector2) -> Variant:
				var from_step: int = _step_at(start_b)
				if from_step < 0:
					return null
				var preview := Label.new()
				preview.text = "  ↕ %s" % act.command.command_name
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

func _cost_text(act: CombatResolver.ScheduledAction) -> String:
	var text: String = PlanBlockText.cost_line(act)
	if planning and not read_only:
		text += "\nDrag onto another block to reorder; right-click to remove."
	return text

func _empty_block(block: int) -> PanelContainer:
	var slot := PanelContainer.new()
	slot.name = "Block%d" % block
	slot.custom_minimum_size = Vector2(block_w, ctrl_h())
	slot.clip_contents = true
	slot.focus_mode = Control.FOCUS_ALL
	slot.set_meta("kind", PlanBlockText.KIND_EMPTY)
	slot.set_meta("start", block)
	slot.set_meta("blocks", 1)
	slot.add_theme_stylebox_override("panel", _block_style(PlanBlockText.KIND_EMPTY, block == active_playhead))
	slot.add_child(_block_label(PlanBlockText.empty_label(block), PlanBlockText.COL_EMPTY_TEXT))
	var who: String = shown_unit.name if shown_unit != null else "The unit"
	var card: Callable = func() -> void: show_block_card(slot, "Block %d: empty" % block, PlanBlockText.empty_body(who, block, planning and not read_only), "")
	slot.mouse_entered.connect(card)
	slot.focus_entered.connect(card)
	slot.mouse_exited.connect(hide_block_card)
	slot.focus_exited.connect(hide_block_card)
	slot.gui_input.connect(func(event: InputEvent) -> void:
		var mouse_event := event as InputEventMouseButton
		if mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT and not planning:
			block_selected.emit(block)
	)
	return slot

# The block card under a block (kept inside the viewport); one line each: title, how it plays out, blocks and AP.
func show_block_card(block: Control, title: String, body: String, cost: String) -> void:
	var w: float = float(CombatHudMetrics.grown(CombatHudMetrics.BLOCK_CARD_W, UiTheme.text_scale()))
	var inner: float = w - 2.0 * 10.0 - 2.0
	var size_px: int = fs()
	for pair: Array in [[block_card_title, title], [block_card_body, body], [block_card_cost, cost]]:
		var lbl: Label = pair[0]
		var text: String = str(pair[1])
		lbl.text = text
		lbl.visible = not text.is_empty()
		lbl.add_theme_font_size_override("font_size", size_px)
		var font: Font = lbl.get_theme_font("font")
		lbl.custom_minimum_size = Vector2(inner, ceilf(font.get_multiline_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, inner, size_px).y))
	block_card.custom_minimum_size = Vector2(w, 0)
	block_card.visible = true
	StripQuestion.fit_now(block_card, w)
	var r: Rect2 = block.get_global_rect()
	var view: Rect2 = get_viewport_rect() if is_inside_tree() else Rect2(Vector2.ZERO, Vector2(1280, 720))
	var pos := Vector2(r.position.x, r.end.y + 6.0)
	pos.x = clampf(pos.x, view.position.x + 4.0, maxf(view.position.x + 4.0, view.end.x - block_card.size.x - 4.0))
	block_card.global_position = pos
	if is_inside_tree():
		_refit_card.call_deferred(w)

func _refit_card(w: float) -> void:
	if block_card.visible:
		StripQuestion.fit_now(block_card, w)

func hide_block_card() -> void:
	if block_card != null:
		block_card.visible = false

func _step_at(block: int) -> int:
	if not plan_step_fn.is_valid():
		return -1
	var raw: Variant = plan_step_fn.call(block)
	return raw if raw is int else -1

# The block widgets of the shown unit, in order (tests and the panel read them).
func blocks() -> Array[Control]:
	var out: Array[Control] = []
	for child: Node in track_clips_hbox.get_children():
		if not child.is_queued_for_deletion():
			out.append(child as Control)
	return out
