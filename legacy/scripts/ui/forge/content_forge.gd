class_name ContentForge
extends Control

# ==============================================================================
# Tactical RPG - Content Forge (read-only asset viewer for the content catalog; TOOLS-RO)
# Browse and inspect skills, items, item families, passives, classes, units and encounters; preview what each entry
# becomes in the engine (skills: how each rank looks in the game's own UI, SkillShowcase); and playtest an encounter
# (or a unit in a sandbox fight) in the real combat arena. The Forge never writes: content is edited through ContentCatalog.upsert seed scripts,
# and every check the old editor ran on Apply / Save runs in the test suites over all shipped data.
#
#   [Skills|Items|Classes|Units|Encounters]                          Reload  Validate All
#   ┌ list ─────────┐ ┌ fields (display only, ContentSchema) ┐ ┌ preview ───────────────┐
#   │ search / side │ │ sections, label: value rows, tables, │ │ skills: rank bar + the │
#   │ filters       │ │ derived panels                       │ │ in-game elements;      │
#   │ (names only)  │ │                                      │ │ others: engine summary │
#   └───────────────┘ └──────────────────────────────────────┘ └────────────────────────┘
#
# FORGE-VIEW: list rows show names only; no problem marks or problems box (the validators run as tests over all shipped
# data; Validate All reports to the status line). Skill filters: kind, grade (basic = basic skills and engine rows, a view
# grouping), stance, learn only; sorts: name, kind, AP.
# Skills (FORGE1): the list shows skill FAMILIES; the form shows the ranked entry (SkillFormModel sections, SkillRanksPanel).
# Classes (CLASS1): ClassForgeModel sections and the live derived panels (ForgePreview.class_panels) under the form. Units
# use the same hooks for their groups. Items and item families (ITEM1): ItemFormModel sections and options, ItemTables
# tables and read-only panels; the item list has filter chips (family, hand, rarity, special, tiers, effects, origin, used by).
# ==============================================================================

signal playtest_requested(encounter: CatalogEncounter)

var kind: String = "skills"
var selected_id: String = ""     # id of the stored entry shown
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
var _preview_icon: TextureRect = null
var _preview: RichTextLabel = null
var _showcase: SkillShowcase = null
var _stance_opt: OptionButton = null
var _playtest_btn: Button = null
var _class_panels: RichTextLabel = null   # the live derived panels under the class form (footer)
var _item_bar: VBoxContainer = null

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
	bar.add_child(UiTheme.label("⚒ CONTENT FORGE (viewer)", UiTheme.FONT_LG, UiTheme.COL_TITLE))
	bar.add_child(VSeparator.new())
	for k: String in ContentSchema.KINDS:
		var btn: Button = UiTheme.button(ContentSchema.kind_label(k), "Browse %s" % ContentSchema.kind_label(k).to_lower(), func() -> void: show_kind(k), UiTheme.FONT_MD)
		btn.toggle_mode = true
		_kind_buttons[k] = btn
		bar.add_child(btn)
	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bar.add_child(spacer)
	bar.add_child(UiTheme.button("⟲ Reload", "Reload the catalog from %s (picks up seed-script edits)" % ContentCatalog.get_root(), func() -> void: reload_all(), UiTheme.FONT_MD))
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

	# Centre: read-only fields
	var centre: VBoxContainer = _column(body, 0)
	centre.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_title = UiTheme.label("", UiTheme.FONT_LG, UiTheme.COL_HEADER)
	_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	centre.add_child(_title)
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	centre.add_child(scroll)
	form = ForgeForm.new()
	form.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(form)

	# Right: preview (skills: the in-game elements per rank; other kinds: icon and engine summary)
	var right: VBoxContainer = _column(body, 400)
	right.add_child(UiTheme.header("ENGINE PREVIEW"))
	var right_scroll := ScrollContainer.new()
	right_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	right_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	right.add_child(right_scroll)
	var right_box := VBoxContainer.new()
	right_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right_box.add_theme_constant_override("separation", 6)
	right_scroll.add_child(right_box)
	_preview_icon = UiTheme.icon_rect(null, 72)
	right_box.add_child(_preview_icon)
	_showcase = SkillShowcase.new()
	right_box.add_child(_showcase)
	_preview = RichTextLabel.new()
	_preview.bbcode_enabled = true
	_preview.fit_content = true
	_preview.scroll_active = false
	_preview.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_preview.add_theme_font_size_override("normal_font_size", UiTheme.FONT_MD)
	_preview.add_theme_font_size_override("bold_font_size", UiTheme.FONT_LG)
	right_box.add_child(_preview)
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
	var kind_opt: OptionButton = _bar_option(row1, "KindFilter")
	kind_opt.add_item("All kinds")
	for k: String in SkillKinds.authorable():
		kind_opt.add_item(SkillKinds.label_of(k))
		kind_opt.set_item_metadata(kind_opt.item_count - 1, k)
	kind_opt.item_selected.connect(func(i: int) -> void:
		set_skill_filter("kinds", [] if i == 0 else [str(kind_opt.get_item_metadata(i))]))
	var grade_opt: OptionButton = _bar_option(row1, "GradeFilter")
	for g: String in SkillFilter.grade_options():
		grade_opt.add_item("any grade" if g == "any" else g)
		grade_opt.set_item_metadata(grade_opt.item_count - 1, g)
	grade_opt.item_selected.connect(func(i: int) -> void: set_skill_filter("grade", str(grade_opt.get_item_metadata(i))))
	var row2 := HBoxContainer.new()
	_skill_bar.add_child(row2)
	var sort_opt: OptionButton = _bar_option(row2, "SortBy")
	for sort_key: String in SkillFilter.SORTS:
		sort_opt.add_item("sort: " + sort_key)
	sort_opt.item_selected.connect(func(i: int) -> void:
		skill_sort = SkillFilter.SORTS[i]
		refresh_list())
	_stance_opt = _bar_option(row2, "StanceFilter")
	_stance_opt.item_selected.connect(func(i: int) -> void: set_skill_filter("stance", str(_stance_opt.get_item_metadata(i))))
	_fill_stances()
	var row3 := HBoxContainer.new()
	_skill_bar.add_child(row3)
	var learn_opt: OptionButton = _bar_option(row3, "LearnFilter")
	for t: String in ["any", "yes", "no"]:
		learn_opt.add_item("Learn only: %s" % t)
		learn_opt.set_item_metadata(learn_opt.item_count - 1, t)
	learn_opt.item_selected.connect(func(i: int) -> void: set_skill_filter("learned_only", str(learn_opt.get_item_metadata(i))))

func _bar_option(row: HBoxContainer, node_name: String) -> OptionButton:
	var opt := OptionButton.new()
	opt.name = node_name
	opt.fit_to_longest_item = false
	opt.clip_text = true
	opt.custom_minimum_size.x = 80
	opt.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(opt)
	return opt

# (Re)fills the stance choices from the catalog (any stance first), keeping the current choice.
func _fill_stances() -> void:
	var current: String = str(skill_filters.get("stance", "any"))
	_stance_opt.clear()
	var choices: Array[String] = ["any"]
	choices.append_array(SkillFilter.stance_options(ContentCatalog.skill_entries()))
	for stance: String in choices:
		_stance_opt.add_item("any stance" if stance == "any" else stance)
		_stance_opt.set_item_metadata(_stance_opt.item_count - 1, stance)
		if stance == current:
			_stance_opt.select(_stance_opt.item_count - 1)

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

# ------------------------------------------------------------------------------
# Form hooks (sections, states and panels per kind)
# ------------------------------------------------------------------------------

func _skills() -> bool:
	return kind == "skills"

# Wires the form hooks for the current kind (the flat grid for the kinds without a model).
func _configure_form() -> void:
	form.panel_fn = Callable()
	if _skills():
		form.state_fn = func(key: String) -> Dictionary: return SkillFormModel.state(form.draft, key, selected_id)
		form.layout_fn = func() -> Array[Dictionary]: return SkillFormModel.layout(form.draft)
		form.footer_fn = func() -> Control:
			var panel := SkillRanksPanel.new()
			panel.show_entry(form.draft, selected_id)
			return panel
		form.problems_fn = func() -> Array[Dictionary]: return problem_list()
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
		form.problems_fn = func() -> Array[Dictionary]: return problem_list()
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
		form.problems_fn = func() -> Array[Dictionary]: return problem_list()
		form.panel_fn = func(id: String) -> Control: return ItemTables.panel(form, id, selected_id)
	else:
		form.state_fn = Callable()
		form.layout_fn = Callable()
		form.footer_fn = Callable()
		form.problems_fn = Callable()

# The item and item-family tabs (ItemFormModel hooks).
func _item_kind() -> bool:
	return kind == "items" or kind == "item_families"

func set_skill_filter(key: String, value: Variant) -> void:
	skill_filters[key] = value
	refresh_list()

# The stored entry as the form shows it: a ranked entry for skills, the flat entry otherwise.
func _stored(id: String) -> Dictionary:
	return ContentCatalog.skill_entry(id) if _skills() else ContentCatalog.get_entry(kind, id)

# ------------------------------------------------------------------------------
# Navigation
# ------------------------------------------------------------------------------

func show_kind(p_kind: String) -> void:
	kind = p_kind
	_sync_kind_buttons()
	_side_filter.visible = kind == "units"
	_skill_bar.visible = _skills()
	if _skills():
		_fill_stances()
	_item_bar.visible = kind == "items"
	if kind == "items":
		_fill_item_bar()
	_playtest_btn.visible = kind == "units" or kind == "encounters"
	_configure_form()
	selected_id = ""
	refresh_list()
	if not _list_ids.is_empty():
		select(_list_ids[0])
	else:
		_show_empty()

func _sync_kind_buttons() -> void:
	for k: Variant in _kind_buttons:
		var btn: Button = _kind_buttons[k]
		btn.set_pressed_no_signal(str(k) == kind)

func refresh_list() -> void:
	_list.clear()
	_list_ids.clear()
	if _skills():
		_refresh_skill_list()
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
		_list.add_item(label)
		_list.set_item_tooltip(_list.item_count - 1, id)
		_list_ids.append(id)
		if id == selected_id:
			_list.select(_list.item_count - 1)

# The skill families that pass the search text and the filter bar, in the chosen order.
func skill_list_entries() -> Array[Dictionary]:
	var filters: Dictionary = skill_filters.duplicate()
	filters["text"] = _search.text if _search != null else ""
	return SkillFilter.apply(ContentCatalog.skill_entries(), filters, skill_sort)

func _refresh_skill_list() -> void:
	for entry: Dictionary in skill_list_entries():
		var id: String = str(entry["id"])
		_list.add_item(str(SkillRanks.effective(entry, 1)["name"]))
		_list.set_item_tooltip(_list.item_count - 1, id)
		_list_ids.append(id)
		if id == selected_id:
			_list.select(_list.item_count - 1)

# Shows stored entry `id` (read-only).
func select(id: String) -> void:
	if id == selected_id and form.kind == kind:
		return
	selected_id = id
	form.edit(kind, _stored(id))
	refresh_list()
	_refresh_side_panels()

func _show_empty() -> void:
	selected_id = ""
	form.kind = kind
	form.draft = {}
	UiTheme.clear_children(form)
	_title.text = "No %s in the catalog." % kind.replace("_", " ")
	_preview_icon.visible = false
	_preview.text = ""
	_showcase.show_entry({})

func reload_all() -> void:
	ContentCatalog.reload_from_disk()
	var keep: String = selected_id
	show_kind(kind)
	if _list_ids.has(keep):
		select(keep)
	_set_status("Reloaded the catalog from %s." % ContentCatalog.get_root())

func validate_all() -> Dictionary:
	var problems: Dictionary = ContentCatalog.validate_all()
	if problems.is_empty():
		_set_status("✔ Catalog valid: %d entries, every reference resolves." % _entry_count())
		_status.tooltip_text = ""
	else:
		var lines: Array[String] = []
		for key: Variant in problems:
			var errs: Array = problems[key]
			for err: Variant in errs:
				lines.append("%s: %s" % [str(key), str(err)])
		_set_status("%d entries have problems (hover for the list; the content tests fail on them too)." % problems.size(), true)
		_status.tooltip_text = "\n".join(lines)
	return problems

func _entry_count() -> int:
	var n: int = 0
	for k: String in ContentSchema.KINDS:
		n += ContentCatalog.entries(k).size()
	return n

# ------------------------------------------------------------------------------
# Problems of the shown entry (the same validators the test suites run over all shipped data)
# ------------------------------------------------------------------------------

# Problems of the shown entry as {"field", "rank", "message"} (field marks in the form).
func problem_list() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if selected_id.is_empty():
		return out
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

# Problems of the shown entry as plain texts.
func entry_problems() -> Array[String]:
	var errors: Array[String] = []
	for p: Dictionary in problem_list():
		var rank: int = DictUtil.to_int(p["rank"])
		var prefix: String = "[rank %d] " % rank if rank > 0 and not str(p["message"]).begins_with("Rank") else ""
		errors.append(prefix + str(p["message"]))
	return errors

# Soft warnings of the shown entry (never errors).
func entry_warnings() -> Array[String]:
	if selected_id.is_empty():
		var none: Array[String] = []
		return none
	if _item_kind():
		return ItemFormModel.warnings(kind, form.draft)
	return ContentCatalog.warnings_for(kind, form.draft)

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
	if selected_id.is_empty():
		_set_status("Select an entry to playtest.", true)
		return
	var enc: CatalogEncounter = playtest_encounter()
	if enc == null:
		return
	if kind == "encounters":
		var errors: Array[String] = ContentCatalog.validate_entry(kind, ContentCatalog.get_entry(kind, selected_id), selected_id)
		if not errors.is_empty():
			_set_status("Cannot playtest: %s" % "; ".join(errors), true)
			return
	_set_status("Playtesting %s..." % enc.encounter_name)
	playtest_requested.emit(enc)

# ------------------------------------------------------------------------------
# Side panels
# ------------------------------------------------------------------------------

func _refresh_side_panels() -> void:
	_title.text = "%s  [%s]" % [str(form.draft.get("name", "")), str(form.draft.get("id", ""))]
	_preview_icon.texture = ForgePreview.texture_for(kind, form.draft)
	_preview_icon.visible = _preview_icon.texture != null
	_showcase.visible = _skills()
	_preview.visible = not _skills()
	if _skills():
		_showcase.show_entry(form.draft, 1)
		_preview.text = ""
	else:
		_preview.text = ForgePreview.describe(kind, form.draft)
	if kind == "classes" and _class_panels != null and is_instance_valid(_class_panels):
		_class_panels.text = ForgePreview.class_panels(form.draft)

# Skills: the rank the preview shows (1-based, clamped; the screenshot helper and tests use it).
func set_preview_rank(rank: int) -> void:
	if _skills() and not selected_id.is_empty():
		_showcase.set_rank(rank)

# The skill preview control (tests read it).
func showcase() -> SkillShowcase:
	return _showcase

func _set_status(text: String, bad: bool = false) -> void:
	_status.text = text
	_status.add_theme_color_override("font_color", UiTheme.COL_BAD if bad else UiTheme.COL_MUTED)

# Status line text (tests read it).
func status_text() -> String:
	return _status.text

# The texts of the entry list rows, in list order (tests read it).
func list_texts() -> Array[String]:
	var out: Array[String] = []
	for i: int in range(_list.item_count):
		out.append(_list.get_item_text(i))
	return out

# The ids the entry list shows, in list order (tests read it).
func list_ids() -> Array[String]:
	return _list_ids.duplicate()

# The live derived panels text of the class form ("" for other kinds; tests read it).
func class_panels_text() -> String:
	return _class_panels.text if kind == "classes" and _class_panels != null and is_instance_valid(_class_panels) else ""
