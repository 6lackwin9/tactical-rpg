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
# ==============================================================================

signal playtest_requested(encounter: CatalogEncounter)

var kind: String = "skills"
var selected_id: String = ""     # id of the stored entry being edited ("" for a new draft)
var draft_dirty: bool = false
var form: ForgeForm = null

var _list: ItemList = null
var _list_ids: Array[String] = []
var _search: LineEdit = null
var _side_filter: OptionButton = null
var _kind_buttons: Dictionary = {}
var _title: Label = null
var _status: Label = null
var _dirty_label: Label = null
var _preview_icon: TextureRect = null
var _preview: RichTextLabel = null
var _problems: RichTextLabel = null
var _playtest_btn: Button = null
var _delete_btn: Button = null
var _pending_delete: String = ""

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
	list_btns.add_child(UiTheme.button("⧉ Duplicate", "Copy the selected entry as a template", func() -> void: duplicate_selected()))
	_delete_btn = UiTheme.button("🗑 Delete", "Delete the selected entry (click twice)", func() -> void: delete_selected())
	list_btns.add_child(_delete_btn)

	# Centre: form
	var centre: VBoxContainer = _column(body, 0)
	centre.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var head := HBoxContainer.new()
	centre.add_child(head)
	_title = UiTheme.label("", UiTheme.FONT_LG, UiTheme.COL_HEADER)
	_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(_title)
	head.add_child(UiTheme.button("✔ Apply", "Store the edits in the catalog (Save writes them to disk)", func() -> void: apply(), UiTheme.FONT_MD))
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
# Navigation
# ------------------------------------------------------------------------------

func show_kind(p_kind: String) -> void:
	if not commit_draft():
		_sync_kind_buttons()
		return
	kind = p_kind
	_sync_kind_buttons()
	_side_filter.visible = kind == "units"
	_playtest_btn.visible = kind == "units" or kind == "encounters"
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
	var needle: String = _search.text.strip_edges().to_lower()
	var side: String = _side_filter.get_item_text(_side_filter.selected) if _side_filter.selected > 0 else ""
	var name_count: Dictionary = {}
	for e: Dictionary in ContentCatalog.entries(kind):
		var entry_name: String = str(e.get("name", ""))
		name_count[entry_name] = DictUtil.int_of(name_count, entry_name) + 1
	for e: Dictionary in ContentCatalog.entries(kind):
		var id: String = str(e.get("id", ""))
		var label: String = str(e.get("name", id))
		if DictUtil.int_of(name_count, label) > 1:
			label += " [%s]" % id   # same display name: tell the entries apart
		if not needle.is_empty() and not label.to_lower().contains(needle) and not id.contains(needle):
			continue
		if kind == "units" and not side.is_empty() and str(e.get("side", "")) != side:
			continue
		var mark: String = "🔒 " if DictUtil.to_bool(e.get("system", false)) else ""
		if not ContentCatalog.validate_entry(kind, e, id).is_empty():
			mark = "⚠ " + mark
		var extra: String = ""
		if kind == "units":
			extra = "  · %s" % str(e.get("side", ""))
		elif kind == "skills":
			extra = "  · %s L%d" % [str(e.get("kind", "")).to_lower(), DictUtil.to_int(e.get("level", 1))]
		elif kind == "items":
			extra = "  · %s" % str(e.get("item_type", "")).to_lower()
		_list.add_item("%s%s%s" % [mark, label, extra])
		_list.set_item_tooltip(_list.item_count - 1, id)
		_list_ids.append(id)
		if id == selected_id:
			_list.select(_list.item_count - 1)
	_update_dirty_label()

# Opens stored entry `id` in the form (committing the current draft first).
func select(id: String) -> void:
	if id == selected_id and not draft_dirty and form.kind == kind:
		return
	if not commit_draft():
		refresh_list()
		return
	selected_id = id
	_pending_delete = ""
	form.edit(kind, ContentCatalog.get_entry(kind, id))
	draft_dirty = false
	refresh_list()
	_refresh_side_panels()

func new_entry() -> void:
	if not commit_draft():
		return
	var d: Dictionary = ContentSchema.defaults(kind)
	d["id"] = ContentCatalog.unique_id(kind, "new_" + kind.trim_suffix("es").trim_suffix("s"))
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
	if _pending_delete != selected_id:
		_pending_delete = selected_id
		_delete_btn.text = "Confirm delete?"
		_set_status("Click Delete again to remove %s." % selected_id)
		return
	var problem: String = ContentCatalog.remove(kind, selected_id)
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

# ------------------------------------------------------------------------------
# Editing
# ------------------------------------------------------------------------------

func _on_form_edited(_key: String) -> void:
	draft_dirty = true
	_refresh_side_panels()
	_update_dirty_label()

# Problems with the current draft (schema, references, JSON fields).
func draft_problems() -> Array[String]:
	var errors: Array[String] = []
	for key: Variant in form.json_errors:
		errors.append(str(form.json_errors[key]))
	errors.append_array(ContentCatalog.validate_entry(kind, form.draft, selected_id))
	return errors

# Stores the draft in the catalog; false (with the problems shown) if invalid.
func apply() -> bool:
	var errors: Array[String] = draft_problems()
	if errors.is_empty():
		errors = ContentCatalog.upsert(kind, form.draft, selected_id)
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
	form.edit(kind, ContentCatalog.get_entry(kind, selected_id))
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
	ContentCatalog.reload()
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
	_preview.text = ForgePreview.describe(kind, form.draft)
	_show_problems(draft_problems())

func _show_problems(errors: Array[String]) -> void:
	if errors.is_empty():
		_problems.text = "[color=#4ade80]✔ No problems.[/color]"
		return
	var lines: Array[String] = []
	for err: String in errors:
		lines.append("[color=#f87171]•[/color] " + err)
	_problems.text = "\n".join(lines)

func _update_dirty_label() -> void:
	_dirty_label.text = "● unsaved changes" if ContentCatalog.is_dirty() else ""

func _set_status(text: String, bad: bool = false) -> void:
	_status.text = text
	_status.add_theme_color_override("font_color", UiTheme.COL_BAD if bad else UiTheme.COL_MUTED)

# Status line text (tests read it).
func status_text() -> String:
	return _status.text
