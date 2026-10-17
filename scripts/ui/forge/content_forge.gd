class_name ContentForge
extends Control

# ==============================================================================
# Tactical RPG - Content Forge (authoring tool for the content catalog)
# Browse, create, duplicate (as a template), edit, validate and delete skills,
# items, classes, units and encounters; preview what each entry becomes in the
# engine; save to res://data/catalog; and playtest an encounter (or a unit in a
# sandbox fight) in the real combat arena.
#
#   [Skills|Items|Classes|Units|Encounters]               [● unsaved] Save Reload Validate
#   ┌ list ─────────┐ ┌ form (ContentSchema) ──────────┐ ┌ preview ───────────────┐
#   │ search / side │ │ Apply  Revert                  │ │ icon, engine summary,  │
#   │ entries       │ │ fields...                      │ │ problems, ▶ Playtest   │
#   │ New Dup Del   │ └────────────────────────────────┘ └────────────────────────┘
#
# Edits live in a draft; Apply (or switching entry) stores it in the catalog in
# memory; Save writes the JSON files. The Forge never edits engine code.
#
# Skills (FORGE1): the list shows skill FAMILIES (the template plus up to two rank modifiers); the form edits the ranked
# entry (SkillFormModel decides sections, limits and locks, SkillRanksPanel is the ranks section); Apply stays disabled
# while ContentCatalog.validate_skill_entry returns problems, listed with their field and rank. System assets (origin
# system) have locked key fields and cannot be deleted here (SAFE DELETE); user assets can be promoted and demoted.
#
# Classes (CLASS1): five sections (ClassForgeModel.layout), typed controls (stat spinners with Remaining, proficiency rows,
# the skill path table, gear pickers grouped by item family), locked key fields on system classes, the live derived panels
# (ForgePreview.class_panels) under the form, and Apply disabled while the draft has errors (warnings never block). Units use
# the same hooks for their groups (the AI group shows heal_below only for a SUPPORT role, hides thresholds when reckless).
#
# Items and item families (ITEM1 commit 3): ItemFormModel decides sections, locks, options, limits and problems; ItemTables
# renders the modifier and tier tables, the family base / band grids and the read-only panels (tokens, chips, base values,
# preview, impact, used by). An item draft is re-resolved after every edit and Apply stores its stored shape (family +
# modifiers). A family Apply that changes items is two-step (the impact first, then a confirming click) and stores the
# family in one batch (upsert_many). The item list has filter chips (family, hand, rarity, special, tiers, effects, origin,
# used by). Apply is disabled while the draft has problems; warnings (bands, budget, tokens) never block them.
# ==============================================================================

signal playtest_requested(encounter: CatalogEncounter)

var kind: String = "skills"
var selected_id: String = ""     # id of the stored entry being edited ("" for a new draft)
var draft_dirty: bool = false
var form: ForgeForm = null
var skill_filters: Dictionary = SkillFilter.default_filters()
var skill_sort: String = "name"
var item_filters: Dictionary = ItemFormModel.default_filters()

var _list: ItemList = null
var _list_ids: Array[String] = []
var _search: LineEdit = null
var _side_filter: OptionButton = null
var _skill_bar: VBoxContainer = null
var _kind_buttons: Dictionary = {}
var _title: Label = null
var _status: Label = null
var _dirty_label: Label = null
var _preview_icon: TextureRect = null
var _preview: RichTextLabel = null
var _problems: RichTextLabel = null
var _playtest_btn: Button = null
var _delete_btn: Button = null
var _apply_btn: Button = null
var _origin_btn: Button = null
var _pending_delete: String = ""
var _pending_origin: String = ""
var _switching_kind: bool = false
var _class_panels: RichTextLabel = null   # the live derived panels under the class form (footer)
var _item_bar: VBoxContainer = null
var _pending_apply: String = ""   # a family draft whose impact was shown (the next Apply of the same draft confirms)

func _init() -> void:
	name = "ContentForge"
	set_anchors_preset(Control.PRESET_FULL_RECT)
	_build()

func _ready() -> void:
	show_kind("skills")

# ------------------------------------------------------------------------------
# Layout
# ------------------------------------------------------------------------------

func _build() -> void:
	var bg := Panel.new()
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	bg.add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_APP_BG, UiTheme.COL_APP_BG, 0))
	add_child(bg)
	var root := VBoxContainer.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.offset_left = 10
	root.offset_top = 8
	root.offset_right = -10
	root.offset_bottom = -8
	root.add_theme_constant_override("separation", 8)
	add_child(root)

	var bar := HBoxContainer.new()
	bar.add_theme_constant_override("separation", 6)
	root.add_child(bar)
	bar.add_child(UiTheme.label("⚒ CONTENT FORGE", UiTheme.FONT_LG, UiTheme.COL_TITLE))
	bar.add_child(VSeparator.new())
	for k: String in ContentSchema.KINDS:
		var btn: Button = UiTheme.button(ContentSchema.kind_label(k), "Edit %s" % ContentSchema.kind_label(k).to_lower(), func() -> void: show_kind(k), UiTheme.FONT_MD)
		btn.toggle_mode = true
		_kind_buttons[k] = btn
		bar.add_child(btn)
	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bar.add_child(spacer)
	_dirty_label = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_WARN)
	bar.add_child(_dirty_label)
	bar.add_child(UiTheme.button("💾 Save", "Write every catalog file to %s (Ctrl+S)" % ContentCatalog.get_root(), func() -> void: save_all(), UiTheme.FONT_MD))
	bar.add_child(UiTheme.button("⟲ Reload", "Discard unsaved changes and reload the catalog from disk", func() -> void: reload_all(), UiTheme.FONT_MD))
	bar.add_child(UiTheme.button("✔ Validate All", "Check every entry and cross-reference", func() -> void: validate_all(), UiTheme.FONT_MD))

	var body := HBoxContainer.new()
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	body.add_theme_constant_override("separation", 8)
	root.add_child(body)

	# Left: entry list
	var left: VBoxContainer = _column(body, 270)
	_search = LineEdit.new()
	_search.placeholder_text = "Search name or id..."
	_search.clear_button_enabled = true
	_search.text_changed.connect(func(_t: String) -> void: refresh_list())
	left.add_child(_search)
	_side_filter = OptionButton.new()
	for side: String in ["All sides", "party", "enemy", "neutral"]:
		_side_filter.add_item(side)
	_side_filter.item_selected.connect(func(_i: int) -> void: refresh_list())
	left.add_child(_side_filter)
	_build_skill_bar(left)
	_build_item_bar(left)
	_list = ItemList.new()
	_list.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_list.item_selected.connect(func(i: int) -> void:
		if i >= 0 and i < _list_ids.size():
			select(_list_ids[i])
	)
	left.add_child(_list)
	var list_btns := HBoxContainer.new()
	left.add_child(list_btns)
	list_btns.add_child(UiTheme.button("+ New", "Start a new entry from defaults", func() -> void: new_entry()))
	list_btns.add_child(UiTheme.button("⧉ Duplicate", "Copy the selected entry as a template (a user copy)", func() -> void: duplicate_selected()))
	_delete_btn = UiTheme.button("🗑 Delete", "Delete the selected entry (click twice)", func() -> void: delete_selected())
	list_btns.add_child(_delete_btn)
	_origin_btn = UiTheme.button("Promote to system", "", func() -> void: toggle_origin())
	left.add_child(_origin_btn)

	# Centre: form
	var centre: VBoxContainer = _column(body, 0)
	centre.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var head := HBoxContainer.new()
	centre.add_child(head)
	_title = UiTheme.label("", UiTheme.FONT_LG, UiTheme.COL_HEADER)
	_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(_title)
	_apply_btn = UiTheme.button("✔ Apply", "Store the edits in the catalog (Save writes them to disk)", func() -> void: apply(), UiTheme.FONT_MD)
	head.add_child(_apply_btn)
	head.add_child(UiTheme.button("↺ Revert", "Discard the edits to this entry", func() -> void: revert(), UiTheme.FONT_MD))
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	centre.add_child(scroll)
	form = ForgeForm.new()
	form.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	form.edited.connect(_on_form_edited)
	scroll.add_child(form)

	# Right: preview & problems
	var right: VBoxContainer = _column(body, 340)
	right.add_child(UiTheme.header("ENGINE PREVIEW"))
	_preview_icon = UiTheme.icon_rect(null, 72)
	right.add_child(_preview_icon)
	_preview = RichTextLabel.new()
	_preview.bbcode_enabled = true
	_preview.fit_content = false
	_preview.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_preview.add_theme_font_size_override("normal_font_size", UiTheme.FONT_MD)
	_preview.add_theme_font_size_override("bold_font_size", UiTheme.FONT_LG)
	right.add_child(_preview)
	right.add_child(UiTheme.header("PROBLEMS"))
	_problems = RichTextLabel.new()
	_problems.bbcode_enabled = true
	_problems.custom_minimum_size.y = 110
	_problems.add_theme_font_size_override("normal_font_size", UiTheme.FONT_SM)
	right.add_child(_problems)
	_playtest_btn = UiTheme.button("▶ Playtest in Arena", "Load this into the real combat engine", func() -> void: playtest(), UiTheme.FONT_MD)
	right.add_child(_playtest_btn)

	_status = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_MUTED)
	root.add_child(_status)

# Filter and sort controls of the skill list (hidden for the other kinds).
func _build_skill_bar(parent: VBoxContainer) -> void:
	_skill_bar = VBoxContainer.new()
	_skill_bar.name = "SkillBar"
	_skill_bar.add_theme_constant_override("separation", 2)
	parent.add_child(_skill_bar)
	var row1 := HBoxContainer.new()
	_skill_bar.add_child(row1)
	var kind_opt := OptionButton.new()
	kind_opt.name = "KindFilter"
	kind_opt.add_item("All kinds")
	for k: String in SkillKinds.authorable():
		kind_opt.add_item(SkillKinds.label_of(k))
		kind_opt.set_item_metadata(kind_opt.item_count - 1, k)
	kind_opt.item_selected.connect(func(i: int) -> void:
		set_skill_filter("kinds", [] if i == 0 else [str(kind_opt.get_item_metadata(i))]))
	kind_opt.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row1.add_child(kind_opt)
	var grade_opt := OptionButton.new()
	grade_opt.name = "GradeFilter"
	for g: String in ["any grade", "novice", "advanced", "expert"]:
		grade_opt.add_item(g)
	grade_opt.item_selected.connect(func(i: int) -> void: set_skill_filter("grade", "any" if i == 0 else grade_opt.get_item_text(i)))
	grade_opt.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row1.add_child(grade_opt)
	var row2 := HBoxContainer.new()
	_skill_bar.add_child(row2)
	var sort_opt := OptionButton.new()
	sort_opt.name = "SortBy"
	for s: String in SkillFilter.SORTS:
		sort_opt.add_item("sort: " + s)
	sort_opt.item_selected.connect(func(i: int) -> void:
		skill_sort = SkillFilter.SORTS[i]
		refresh_list())
	sort_opt.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row2.add_child(sort_opt)
	var ranks_opt := OptionButton.new()
	ranks_opt.name = "RankFilter"
	for r: String in ["any ranks", "1 rank", "2 ranks", "3 ranks"]:
		ranks_opt.add_item(r)
	ranks_opt.item_selected.connect(func(i: int) -> void: set_skill_filter("rank_count", i))
	ranks_opt.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row2.add_child(ranks_opt)
	var row3 := HFlowContainer.new()
	_skill_bar.add_child(row3)
	for spec: Array in [["Problems", "has_problems", "Only skills that have problems"], ["Engine rows", "show_system", "Show the engine skills (Move, Rest, ...)"]]:
		var check := CheckBox.new()
		check.text = str(spec[0])
		check.tooltip_text = str(spec[2])
		check.name = "Check_" + str(spec[1])
		var key: String = str(spec[1])
		check.toggled.connect(func(on: bool) -> void: set_skill_filter(key, on))
		row3.add_child(check)
	for spec: Array in [["Basic", "basic"], ["Learn only", "learned_only"]]:
		var tri := OptionButton.new()
		tri.name = "Tri_" + str(spec[1])
		for t: String in ["%s: any" % str(spec[0]), "%s: yes" % str(spec[0]), "%s: no" % str(spec[0])]:
			tri.add_item(t)
		var tri_key: String = str(spec[1])
		tri.item_selected.connect(func(i: int) -> void: set_skill_filter(tri_key, ["any", "yes", "no"][i]))
		row3.add_child(tri)

# Filter chips of the item list (matrix 6: family, hand, rarity, special, has tiers, has effects, origin, used by).
func _build_item_bar(parent: VBoxContainer) -> void:
	_item_bar = VBoxContainer.new()
	_item_bar.name = "ItemBar"
	_item_bar.add_theme_constant_override("separation", 2)
	parent.add_child(_item_bar)
	var flow := HFlowContainer.new()
	_item_bar.add_child(flow)
	var labels: Dictionary = {"family": "family", "hand": "hand", "rarity": "rarity", "origin": "origin", "used_by": "used by",
		"special": "special", "has_tiers": "tiers", "has_effects": "effects"}
	for key: String in ["family", "hand", "rarity", "origin", "used_by", "special", "has_tiers", "has_effects"]:
		var opt := OptionButton.new()
		opt.name = "ItemFilter_" + key
		opt.fit_to_longest_item = false
		opt.clip_text = true
		opt.custom_minimum_size.x = 80
		var filter_key: String = key
		opt.item_selected.connect(func(i: int) -> void: set_item_filter(filter_key, str(opt.get_item_metadata(i))))
		flow.add_child(opt)
	_fill_item_bar(labels)

# (Re)fills the chip choices (the family list follows the catalog) keeping the current filter values.
func _fill_item_bar(labels: Dictionary = {}) -> void:
	for child: Node in _item_bar.get_child(0).get_children():
		var opt: OptionButton = child as OptionButton
		if opt == null:
			continue
		var key: String = str(opt.name).trim_prefix("ItemFilter_")
		var title: String = str(labels.get(key, opt.get_meta("title", key)))
		opt.set_meta("title", title)
		opt.clear()
		for value: String in ItemFormModel.filter_options(key):
			var any: bool = value.is_empty() or value == "any"
			opt.add_item("%s: %s" % [title, "any" if any else value])
			opt.set_item_metadata(opt.item_count - 1, value)
			if value == str(item_filters.get(key, "")):
				opt.select(opt.item_count - 1)

func set_item_filter(key: String, value: Variant) -> void:
	item_filters[key] = value
	refresh_list()

func _column(parent: Control, width: int) -> VBoxContainer:
	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_PANEL_BG, UiTheme.COL_BORDER, 6, 8, 8))
	if width > 0:
		panel.custom_minimum_size.x = width
	panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL if width == 0 else Control.SIZE_FILL
	parent.add_child(panel)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 6)
	panel.add_child(col)
	return col

func _unhandled_key_input(event: InputEvent) -> void:
	var key: InputEventKey = event as InputEventKey
	if key != null and key.pressed and key.ctrl_pressed and key.keycode == KEY_S:
		save_all()
		get_viewport().set_input_as_handled()

# ------------------------------------------------------------------------------
# Skill hooks (the skill tab edits ranked entries through SkillFormModel)
# ------------------------------------------------------------------------------

func _skills() -> bool:
	return kind == "skills"

# Wires the form hooks for the skill tab, clears them for every other kind.
func _configure_form() -> void:
	if _skills():
		form.state_fn = func(key: String) -> Dictionary: return SkillFormModel.state(form.draft, key, selected_id)
		form.layout_fn = func() -> Array[Dictionary]: return SkillFormModel.layout(form.draft)
		form.footer_fn = func() -> Control:
			var panel := SkillRanksPanel.new()
			panel.changed.connect(_on_ranks_changed)
			panel.show_entry(form.draft, selected_id)
			return panel
		form.problems_fn = func() -> Array[Dictionary]: return draft_problem_list()
	elif kind == "classes":
		form.state_fn = func(key: String) -> Dictionary: return ClassForgeModel.state(form.draft, key, selected_id)
		form.layout_fn = func() -> Array[Dictionary]: return ClassForgeModel.layout("classes")
		form.footer_fn = func() -> Control:
			_class_panels = RichTextLabel.new()
			_class_panels.name = "ClassPanels"
			_class_panels.bbcode_enabled = true
			_class_panels.fit_content = true
			_class_panels.scroll_active = false
			_class_panels.add_theme_font_size_override("normal_font_size", UiTheme.FONT_SM)
			_class_panels.add_theme_font_size_override("bold_font_size", UiTheme.FONT_MD)
			_class_panels.text = ForgePreview.class_panels(form.draft)
			return _class_panels
		form.problems_fn = func() -> Array[Dictionary]: return draft_problem_list()
	elif kind == "units":
		form.state_fn = func(key: String) -> Dictionary: return ClassForgeModel.unit_state(form.draft, key)
		form.layout_fn = func() -> Array[Dictionary]: return ClassForgeModel.layout("units", form.draft)
		form.footer_fn = Callable()
		form.problems_fn = Callable()
	elif _item_kind():
		var k: String = kind
		form.state_fn = func(key: String) -> Dictionary: return ItemFormModel.state(k, form.draft, key, selected_id)
		form.layout_fn = func() -> Array[Dictionary]: return ItemFormModel.layout(k, form.draft)
		form.footer_fn = Callable()
		form.problems_fn = func() -> Array[Dictionary]: return draft_problem_list()
		form.panel_fn = func(id: String) -> Control: return ItemTables.panel(form, id, selected_id)
		return
	else:
		form.state_fn = Callable()
		form.layout_fn = Callable()
		form.footer_fn = Callable()
		form.problems_fn = Callable()
	form.panel_fn = Callable()

# The item and item-family tabs (ItemFormModel hooks).
func _item_kind() -> bool:
	return kind == "items" or kind == "item_families"

# Kinds whose Apply is disabled while the draft has problems (the problems bar says "Cannot save").
func _gated() -> bool:
	return _skills() or kind == "classes" or _item_kind()

func _on_ranks_changed(entry: Dictionary) -> void:
	form.replace_draft(entry)

func set_skill_filter(key: String, value: Variant) -> void:
	skill_filters[key] = value
	refresh_list()

# The stored entry as the form edits it: a ranked entry for skills, the flat entry otherwise.
func _stored(id: String) -> Dictionary:
	return ContentCatalog.skill_entry(id) if _skills() else ContentCatalog.get_entry(kind, id)

# ------------------------------------------------------------------------------
# Navigation
# ------------------------------------------------------------------------------

func show_kind(p_kind: String) -> void:
	if not commit_draft():
		_sync_kind_buttons()
		return
	kind = p_kind
	_sync_kind_buttons()
	_side_filter.visible = kind == "units"
	_skill_bar.visible = _skills()
	_item_bar.visible = kind == "items"
	if kind == "items":
		_fill_item_bar()
	_pending_apply = ""
	_playtest_btn.visible = kind == "units" or kind == "encounters"
	_configure_form()
	selected_id = ""
	refresh_list()
	if not _list_ids.is_empty():
		select(_list_ids[0])
	else:
		new_entry()

func _sync_kind_buttons() -> void:
	for k: Variant in _kind_buttons:
		var btn: Button = _kind_buttons[k]
		btn.set_pressed_no_signal(str(k) == kind)

func refresh_list() -> void:
	_list.clear()
	_list_ids.clear()
	if _skills():
		_refresh_skill_list()
		_update_dirty_label()
		return
	var needle: String = _search.text.strip_edges().to_lower()
	var side: String = _side_filter.get_item_text(_side_filter.selected) if _side_filter.selected > 0 else ""
	var name_count: Dictionary = {}
	for e: Dictionary in ContentCatalog.entries(kind):
		var entry_name: String = str(e.get("name", ""))
		name_count[entry_name] = DictUtil.int_of(name_count, entry_name) + 1
	var listed: Array[Dictionary] = ContentCatalog.entries(kind)
	if kind == "items":
		listed = ItemFormModel.filter_items(listed, item_filters)
	for e: Dictionary in listed:
		var id: String = str(e.get("id", ""))
		var label: String = str(e.get("name", id))
		if DictUtil.int_of(name_count, label) > 1:
			label += " [%s]" % id   # same display name: tell the entries apart
		if not needle.is_empty() and not label.to_lower().contains(needle) and not id.contains(needle):
			continue
		if kind == "units" and not side.is_empty() and str(e.get("side", "")) != side:
			continue
		var mark: String = "🔒 " if DictUtil.to_bool(e.get("system", false)) or ContentCatalog.is_system_origin(e) else ""
		if not ContentCatalog.validate_entry(kind, e, id).is_empty():
			mark = "⚠ " + mark
		var extra: String = ""
		if kind == "units":
			extra = "  · %s" % str(e.get("side", ""))
		elif kind == "items":
			extra = "  · %s" % (str(e.get("family", "")) if not str(e.get("family", "")).is_empty() else str(e.get("item_type", "")).to_lower())
		elif kind == "item_families":
			extra = "  · %s, %d items" % [str(e.get("item_type", "")).to_lower(), ContentCatalog.items_of_family(id).size()]
		_list.add_item("%s%s%s" % [mark, label, extra])
		_list.set_item_tooltip(_list.item_count - 1, id)
		_list_ids.append(id)
		if id == selected_id:
			_list.select(_list.item_count - 1)
	_update_dirty_label()

# The skill families that pass the search text and the filter bar, in the chosen order.
func skill_list_entries() -> Array[Dictionary]:
	var filters: Dictionary = skill_filters.duplicate()
	filters["text"] = _search.text if _search != null else ""
	return SkillFilter.apply(ContentCatalog.skill_entries(), filters, skill_sort)

func _refresh_skill_list() -> void:
	for entry: Dictionary in skill_list_entries():
		var id: String = str(entry["id"])
		var base: Dictionary = SkillRanks.effective(entry, 1)
		var mark: String = ""
		if ContentCatalog.family_has_problems(id):
			mark += "⚠ "
		if ContentCatalog.is_system_origin(base):
			mark += "🔒 "
		var ranks: int = SkillRanks.rank_count(entry)
		_list.add_item("%s%s  · %s L%d%s" % [mark, str(base["name"]), str(base["kind"]).to_lower(), DictUtil.to_int(base["level"]), ("  · %d ranks" % ranks) if ranks > 1 else ""])
		_list.set_item_tooltip(_list.item_count - 1, id)
		_list_ids.append(id)
		if id == selected_id:
			_list.select(_list.item_count - 1)

# Opens stored entry `id` in the form (committing the current draft first).
func select(id: String) -> void:
	if id == selected_id and not draft_dirty and form.kind == kind:
		return
	if not commit_draft():
		refresh_list()
		return
	selected_id = id
	_pending_delete = ""
	_pending_origin = ""
	form.edit(kind, _stored(id))
	draft_dirty = false
	refresh_list()
	_refresh_side_panels()

func new_entry() -> void:
	if not commit_draft():
		return
	var d: Dictionary = ContentSchema.defaults(kind)
	d["id"] = ContentCatalog.unique_id(kind, "new_" + kind.trim_suffix("es").trim_suffix("s"))
	if _skills():
		d = SkillFormModel.new_entry(str(d["id"]))
	elif kind == "classes":
		d = ClassForgeModel.new_entry(str(d["id"]))   # a class that validates as it starts (FORGE1_CLASSES 2.0)
	elif kind == "items":
		d = ItemFormModel.new_entry(str(d["id"]))
	elif kind == "item_families":
		d = ItemFormModel.new_family(ContentCatalog.unique_id(kind, "new_family"))
	if kind == "units" and _side_filter.selected > 0:
		d["side"] = _side_filter.get_item_text(_side_filter.selected)
	selected_id = ""
	form.edit(kind, d)
	draft_dirty = true
	_list.deselect_all()
	_refresh_side_panels()
	_set_status("New %s draft: fill it in and Apply." % kind.trim_suffix("s"))

func duplicate_selected() -> void:
	if selected_id.is_empty() or not commit_draft():
		return
	var copy_id: String = ContentCatalog.duplicate_entry(kind, selected_id)
	if copy_id.is_empty():
		_set_status("Couldn't duplicate %s." % selected_id, true)
		return
	_set_status("Duplicated %s as %s: edit the copy as a new entry." % [selected_id, copy_id])
	select(copy_id)

func delete_selected() -> void:
	if selected_id.is_empty():
		new_entry()   # discard an unapplied new draft
		return
	if _is_system(selected_id):
		_set_status("%s is a system asset: delete is locked. Use SAFE DELETE (tools/fc.sh safe_delete %s %s) or make your own copy." % [selected_id, kind, selected_id], true)
		return
	if _pending_delete != selected_id:
		_pending_delete = selected_id
		_delete_btn.text = "Confirm delete?"
		_set_status("Click Delete again to remove %s." % selected_id)
		return
	var problem: String = ContentCatalog.remove_skill(selected_id) if _skills() else ContentCatalog.remove(kind, selected_id)
	_delete_btn.text = "🗑 Delete"
	_pending_delete = ""
	if not problem.is_empty():
		_set_status(problem, true)
		return
	_set_status("Deleted %s (Save to write it to disk)." % selected_id)
	draft_dirty = false
	selected_id = ""
	refresh_list()
	if not _list_ids.is_empty():
		select(_list_ids[0])
	else:
		new_entry()

# True when the stored asset `id` is origin system (its delete and key fields are locked).
func _is_system(id: String) -> bool:
	if id.is_empty():
		return false
	if _skills():
		var rows: Array[Dictionary] = ContentCatalog.family_rows(id)
		return not rows.is_empty() and ContentCatalog.is_system_origin(rows[0])
	return ContentCatalog.is_system_origin(ContentCatalog.get_entry(kind, id))

# Promote a user asset to system / demote a system asset to user (asks twice; the text names the consequences).
func toggle_origin() -> void:
	if selected_id.is_empty() or not commit_draft():
		return
	var system: bool = _is_system(selected_id)
	if _pending_origin != selected_id:
		_pending_origin = selected_id
		if system:
			_set_status("Demote %s to a user asset? Delete becomes possible and key fields unlock. Click again to confirm." % selected_id)
		else:
			_set_status("Promote %s to a system asset? Its key fields lock and delete is locked (undo with Demote). Click again to confirm." % selected_id)
		return
	_pending_origin = ""
	var problem: String = ContentCatalog.demote(kind, selected_id) if system else ContentCatalog.promote(kind, selected_id)
	if not problem.is_empty():
		_set_status(problem, true)
		return
	_set_status("%s is now a %s asset." % [selected_id, "user" if system else "system"])
	var keep: String = selected_id
	selected_id = ""
	select(keep)

# ------------------------------------------------------------------------------
# Editing
# ------------------------------------------------------------------------------

func _on_form_edited(key: String) -> void:
	if _skills() and key == "kind" and not _switching_kind:
		_switching_kind = true
		form.replace_draft(SkillFormModel.switch_kind(form.draft, str(form.draft.get("kind", "ATTACK"))))
		_switching_kind = false
	if form.kind == "items" and kind == "items":
		ItemFormModel.sync(form.draft, key)   # derived keys follow family / hand / modifiers / tiers
		if key == "family" or key == "hand":
			form.rebuild.call_deferred()   # options and limits of the build section and the tables depend on them
	elif form.kind == "item_families" and kind == "item_families":
		ItemFormModel.sync_family(form.draft, key)
		if key == "hands" or key == "item_type":
			form.rebuild.call_deferred()   # the base / band grids have one row per hand
	_pending_apply = ""
	draft_dirty = true
	_refresh_side_panels()
	_update_dirty_label()

# Problems with the current draft as {"field", "rank", "message"} (skills only: the validator's own list).
func draft_problem_list() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for key: Variant in form.json_errors:
		out.append({"field": str(key), "rank": 0, "message": str(form.json_errors[key])})
	if _skills():
		out.append_array(ContentCatalog.validate_skill_entry(form.draft, selected_id))
	elif kind == "classes":
		for p: Dictionary in ClassForgeModel.draft_problems(form.draft, selected_id):
			out.append({"field": str(p["field"]), "rank": 0, "message": str(p["message"])})
	elif _item_kind():
		for p: Dictionary in ItemFormModel.draft_problems(kind, form.draft, selected_id):
			out.append({"field": str(p["field"]), "rank": 0, "index": DictUtil.to_int(p.get("index", 0)), "message": str(p["message"])})
	else:
		for message: String in ContentCatalog.validate_entry(kind, form.draft, selected_id):
			out.append({"field": "", "rank": 0, "message": message})
	return out

# Problems with the current draft (schema, references, JSON fields), as plain texts.
func draft_problems() -> Array[String]:
	var errors: Array[String] = []
	for p: Dictionary in draft_problem_list():
		var rank: int = DictUtil.to_int(p["rank"])
		var prefix: String = "[rank %d] " % rank if rank > 0 and not str(p["message"]).begins_with("Rank") else ""
		errors.append(prefix + str(p["message"]))
	return errors

# Stores the draft in the catalog; false (with the problems shown) if invalid.
func apply() -> bool:
	var errors: Array[String] = draft_problems()
	if errors.is_empty() and kind == "item_families" and not _confirm_family_apply():
		return false
	if errors.is_empty():
		if _skills():
			errors = ContentCatalog.upsert_skill(form.draft, selected_id)
		elif kind == "items":
			errors = ContentCatalog.upsert(kind, ItemFormModel.stored_of(form.draft), selected_id)   # family + modifiers (list order kept)
		elif kind == "item_families" and not selected_id.is_empty() and str(form.draft.get("id", "")) == selected_id:
			errors = ContentCatalog.upsert_many(kind, [form.draft])   # one batch; the family's items re-resolve (FORGE1_ITEMS 3 rail 2)
		else:
			errors = ContentCatalog.upsert(kind, form.draft, selected_id)
	_pending_apply = ""
	if not errors.is_empty():
		_show_problems(errors)
		_set_status("Not applied: fix the problems listed on the right.", true)
		return false
	selected_id = str(form.draft.get("id", ""))
	draft_dirty = false
	refresh_list()
	_refresh_side_panels()
	_set_status("Applied %s (Save to write it to disk)." % selected_id)
	return true

# Two-step Apply of a family edit that changes items (FORGE1_ITEMS 3 rail 5): the first click shows the impact and returns
# false; a second click on the SAME draft confirms. Edits in between reset it (_on_form_edited).
func _confirm_family_apply() -> bool:
	if not ItemFormModel.needs_confirm(form.draft, selected_id):
		return true
	var signature: String = JSON.stringify(ItemModifiers.sorted_value(form.draft))
	if _pending_apply == signature:
		return true
	_pending_apply = signature
	_set_status("Impact: %s. Click Apply again to confirm." % str(ItemFormModel.impact(form.draft, selected_id)["text"]))
	return false

# True while a family Apply waits for its confirming click (tests read it).
func apply_pending() -> bool:
	return not _pending_apply.is_empty()

# Applies a dirty draft before navigating away; false keeps the user on it.
func commit_draft() -> bool:
	if not draft_dirty or form == null or form.kind.is_empty():
		return true
	if form.kind != kind:
		return true
	return apply()

func revert() -> void:
	draft_dirty = false
	if selected_id.is_empty():
		new_entry()
		return
	form.edit(kind, _stored(selected_id))
	_refresh_side_panels()
	_set_status("Reverted %s." % selected_id)

func save_all() -> bool:
	if not commit_draft():
		return false
	var err: Error = ContentCatalog.save_all()
	if err != OK:
		_set_status("Save failed (%s): %s is read-only in exported builds." % [error_string(err), ContentCatalog.get_root()], true)
		return false
	_update_dirty_label()
	_set_status("Saved the catalog to %s." % ContentCatalog.get_root())
	return true

func reload_all() -> void:
	ContentCatalog.reload_from_disk()
	draft_dirty = false
	selected_id = ""
	show_kind(kind)
	_set_status("Reloaded the catalog from %s." % ContentCatalog.get_root())

func validate_all() -> Dictionary:
	var problems: Dictionary = ContentCatalog.validate_all()
	if problems.is_empty():
		_problems.text = "[color=#4ade80]✔ Every entry and reference is valid.[/color]"
		_set_status("Catalog valid: %d entries." % _entry_count())
	else:
		var lines: Array[String] = []
		for key: Variant in problems:
			var errs: Array = problems[key]
			for err: Variant in errs:
				lines.append("[color=#f87171]%s[/color]: %s" % [str(key), str(err)])
		_problems.text = "\n".join(lines)
		_set_status("%d entries have problems." % problems.size(), true)
	return problems

func _entry_count() -> int:
	var n: int = 0
	for k: String in ContentSchema.KINDS:
		n += ContentCatalog.entries(k).size()
	return n

# ------------------------------------------------------------------------------
# Playtest
# ------------------------------------------------------------------------------

# Encounters run as authored; a unit runs in a sandbox fight on the side it belongs to
# (party units vs a brawler and an archer, enemies vs the skirmish Warrior & Ranger).
func playtest_encounter() -> CatalogEncounter:
	if kind == "encounters":
		return CatalogEncounter.new(selected_id)
	if kind != "units":
		return null
	var unit_id: String = selected_id
	var party_side: bool = str(ContentCatalog.get_entry("units", unit_id).get("side", "")) == "party"
	var sandbox: Dictionary = ContentSchema.defaults("encounters")
	sandbox.merge({
		"id": "forge_sandbox", "name": "Forge Sandbox: %s" % str(ContentCatalog.get_entry("units", unit_id).get("name", unit_id)),
		"zone": "stone_arena",
		"party": [unit_id] if party_side else ["warrior", "ranger"],
		"enemies": ["goblin_brawler", "goblin_archer"] if party_side else [unit_id],
	}, true)
	return CatalogEncounter.from_entry(sandbox)

func playtest() -> void:
	if not commit_draft() or selected_id.is_empty():
		_set_status("Apply the entry before playtesting.", true)
		return
	var enc: CatalogEncounter = playtest_encounter()
	if enc == null:
		return
	if kind == "encounters":
		var errors: Array[String] = ContentCatalog.validate_entry(kind, ContentCatalog.get_entry(kind, selected_id), selected_id)
		if not errors.is_empty():
			_show_problems(errors)
			return
	_set_status("Playtesting %s..." % enc.encounter_name)
	playtest_requested.emit(enc)

# ------------------------------------------------------------------------------
# Side panels
# ------------------------------------------------------------------------------

func _refresh_side_panels() -> void:
	var title: String = str(form.draft.get("name", ""))
	_title.text = "%s  [%s]%s" % [title, str(form.draft.get("id", "")), "  ● edited" if draft_dirty else ""]
	if not selected_id.is_empty() and str(form.draft.get("id", "")) != selected_id:
		_title.text += "  (renaming %s)" % selected_id
	_preview_icon.texture = ForgePreview.texture_for(kind, form.draft)
	_preview_icon.visible = _preview_icon.texture != null
	_preview.text = ForgePreview.describe(kind, form.draft, _skills())
	var errors: Array[String] = draft_problems()
	_show_problems(errors)
	# Apply is blocked while a skill or class draft has problems; Delete and the origin button follow the selected asset.
	if kind == "classes" and _class_panels != null and is_instance_valid(_class_panels):
		_class_panels.text = ForgePreview.class_panels(form.draft)
	_apply_btn.disabled = _gated() and not errors.is_empty()
	_apply_btn.tooltip_text = ("Cannot apply: %d problem%s (listed on the right)" % [errors.size(), "" if errors.size() == 1 else "s"]) if _apply_btn.disabled else "Store the edits in the catalog (Save writes them to disk)"
	var system: bool = _is_system(selected_id)
	_delete_btn.disabled = system
	_delete_btn.tooltip_text = "System asset: delete is locked. Use SAFE DELETE (tools/fc.sh safe_delete) or make your own copy." if system else "Delete the selected entry (click twice)"
	if _item_kind():
		var del: Dictionary = ItemFormModel.delete_state(kind, selected_id)
		_delete_btn.disabled = not DictUtil.to_bool(del["allowed"])
		if _delete_btn.disabled:
			_delete_btn.tooltip_text = str(del["reason"])
	_origin_btn.visible = not selected_id.is_empty()
	_origin_btn.text = "Demote to user" if system else "Promote to system"
	_origin_btn.tooltip_text = "Delete becomes possible and key fields unlock." if system else "Lock key fields and delete (use for assets that tests or tuning rely on)."

func _show_problems(errors: Array[String]) -> void:
	var item_warnings: Array[String] = []
	if _item_kind():
		item_warnings = ItemFormModel.warnings(kind, form.draft)
	if errors.is_empty():
		_problems.text = "[color=#4ade80]✔ No problems.[/color]"
		for w: String in item_warnings:   # item and family warnings (bands, budget, tokens) show even when Apply is allowed
			_problems.text += "\n[color=#fbbf24]•[/color] " + w
		return
	var lines: Array[String] = []
	var gated: bool = _gated()
	if gated:
		lines.append("[color=#f87171]Cannot save: %d problem%s[/color]" % [errors.size(), "" if errors.size() == 1 else "s"])
	for err: String in errors:
		lines.append("[color=#f87171]•[/color] " + err)
	if gated:
		if not _item_kind():
			item_warnings = ContentCatalog.warnings_for(kind, form.draft)
		for w: String in item_warnings:
			lines.append("[color=#fbbf24]•[/color] " + w)
	_problems.text = "\n".join(lines)

# The problems bar text (tests read it).
func problems_text() -> String:
	return _problems.text

func _update_dirty_label() -> void:
	_dirty_label.text = "● unsaved changes" if ContentCatalog.is_dirty() else ""

func _set_status(text: String, bad: bool = false) -> void:
	_status.text = text
	_status.add_theme_color_override("font_color", UiTheme.COL_BAD if bad else UiTheme.COL_MUTED)

# Status line text (tests read it).
func status_text() -> String:
	return _status.text

# True while Apply is disabled (a skill or class draft with errors; tests read it).
func apply_blocked() -> bool:
	return _apply_btn.disabled

# The ids the entry list shows, in list order (tests read it).
func list_ids() -> Array[String]:
	return _list_ids.duplicate()

# True while Delete is disabled for the selected entry (system asset, used family or item; tests read it).
func delete_blocked() -> bool:
	return _delete_btn.disabled

# The live derived panels text of the class form ("" for other kinds; tests read it).
func class_panels_text() -> String:
	return _class_panels.text if kind == "classes" and _class_panels != null and is_instance_valid(_class_panels) else ""
