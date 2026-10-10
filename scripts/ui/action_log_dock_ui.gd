class_name ActionLogDockUI
extends PanelContainer

# ==============================================================================
# Tactical RPG - Action Log Compact Resolution Dock
# Single-cell live resolution dock with action navigation buttons,
# active block badge, and modal expand trigger.
# ==============================================================================

signal prev_pressed()
signal next_pressed()
signal expand_requested()
signal entry_clicked(entry: ActionLogEntry)

var log_title: Label
var log_step_badge: Label
var btn_log_prev: Button
var btn_log_next: Button
var btn_expand_log: Button
var active_cell_dock: PanelContainer

func _init() -> void:
	name = "ActionLogDockUI"
	custom_minimum_size = Vector2(430, 84)
	size_flags_horizontal = Control.SIZE_SHRINK_CENTER

	var log_style := StyleBoxFlat.new()
	log_style.bg_color = Color(0.06, 0.08, 0.12, 0.95)
	log_style.border_color = Color(0.25, 0.32, 0.44, 0.8)
	log_style.set_border_width_all(1)
	log_style.set_corner_radius_all(6)
	log_style.content_margin_left = 6
	log_style.content_margin_right = 6
	log_style.content_margin_top = 4
	log_style.content_margin_bottom = 4
	add_theme_stylebox_override("panel", log_style)

	_build_ui()

func _build_ui() -> void:
	var log_vbox := VBoxContainer.new()
	log_vbox.add_theme_constant_override("separation", 2)
	add_child(log_vbox)

	var log_hdr_hbox := HBoxContainer.new()
	log_hdr_hbox.add_theme_constant_override("separation", 6)
	log_vbox.add_child(log_hdr_hbox)

	log_title = Label.new()
	log_title.text = "📜 RESOLUTION LOG"
	log_title.add_theme_font_size_override("font_size", 10)
	log_title.add_theme_color_override("font_color", Color(0.96, 0.82, 0.35))
	log_hdr_hbox.add_child(log_title)

	log_step_badge = Label.new()
	log_step_badge.text = "[Awaiting Action]"
	log_step_badge.add_theme_font_size_override("font_size", 8)
	log_step_badge.add_theme_color_override("font_color", Color(0.55, 0.75, 0.95))
	log_hdr_hbox.add_child(log_step_badge)

	btn_log_prev = Button.new()
	btn_log_prev.text = "◀"
	btn_log_prev.custom_minimum_size = Vector2(20, 16)
	btn_log_prev.add_theme_font_size_override("font_size", 8)
	btn_log_prev.tooltip_text = "Previous Action Cell"
	btn_log_prev.pressed.connect(func() -> void: prev_pressed.emit())
	log_hdr_hbox.add_child(btn_log_prev)

	btn_log_next = Button.new()
	btn_log_next.text = "▶"
	btn_log_next.custom_minimum_size = Vector2(20, 16)
	btn_log_next.add_theme_font_size_override("font_size", 8)
	btn_log_next.tooltip_text = "Next Action Cell"
	btn_log_next.pressed.connect(func() -> void: next_pressed.emit())
	log_hdr_hbox.add_child(btn_log_next)

	log_hdr_hbox.add_spacer(false)

	btn_expand_log = Button.new()
	btn_expand_log.text = "⛶ EXPAND"
	btn_expand_log.custom_minimum_size = Vector2(66, 18)
	btn_expand_log.add_theme_font_size_override("font_size", 8)
	var exp_btn_style := StyleBoxFlat.new()
	exp_btn_style.bg_color = Color(0.16, 0.22, 0.32, 0.9)
	exp_btn_style.border_color = Color(0.38, 0.6, 0.9, 0.8)
	exp_btn_style.set_border_width_all(1)
	exp_btn_style.set_corner_radius_all(3)
	exp_btn_style.content_margin_left = 4
	exp_btn_style.content_margin_right = 4
	btn_expand_log.add_theme_stylebox_override("normal", exp_btn_style)
	btn_expand_log.tooltip_text = "Expand Full Chronological Action Log Modal (Shortcut: L)"
	btn_expand_log.pressed.connect(func() -> void: expand_requested.emit())
	log_hdr_hbox.add_child(btn_expand_log)

	# Active single action cell dock
	active_cell_dock = PanelContainer.new()
	active_cell_dock.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	active_cell_dock.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var dock_style := StyleBoxFlat.new()
	dock_style.bg_color = Color(0.04, 0.06, 0.09, 0.7)
	dock_style.border_color = Color(0.18, 0.24, 0.34, 0.5)
	dock_style.set_border_width_all(1)
	dock_style.set_corner_radius_all(4)
	dock_style.content_margin_left = 4
	dock_style.content_margin_right = 4
	dock_style.content_margin_top = 2
	dock_style.content_margin_bottom = 2
	active_cell_dock.add_theme_stylebox_override("panel", dock_style)
	log_vbox.add_child(active_cell_dock)

func update_active_cell(entries: Array[ActionLogEntry], active_idx: int) -> void:
	for child in active_cell_dock.get_children():
		child.queue_free()

	if entries.is_empty() or active_idx < 0:
		var empty_lbl := Label.new()
		empty_lbl.text = "Awaiting Action Resolution... Click Step [F6] or Play [Space] to begin."
		empty_lbl.add_theme_font_size_override("font_size", 9)
		empty_lbl.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
		active_cell_dock.add_child(empty_lbl)
		log_step_badge.text = "[Awaiting Action]"
		btn_log_prev.disabled = true
		btn_log_next.disabled = true
	else:
		var entry: ActionLogEntry = entries[active_idx]
		var cell := CombatLogModal.format_action_cell(entry, false, func(e: ActionLogEntry) -> void:
			entry_clicked.emit(e)
		)
		active_cell_dock.add_child(cell)
		log_step_badge.text = "[%s] (%d/%d)" % [entry.ref_tag, active_idx + 1, entries.size()]
		btn_log_prev.disabled = (active_idx <= 0)
		btn_log_next.disabled = (active_idx >= entries.size() - 1)
