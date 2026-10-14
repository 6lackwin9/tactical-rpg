class_name WorldForge
extends Control

# ==============================================================================
# Tactical RPG - World Forge (authoring tool for lore worlds)
# Create worlds, extend or connect them, and fill them with regions, locations,
# factions, characters (party / NPCs / enemies), named items, disciplines,
# quests, lore and encounters. Every gameplay-bearing entry references a base
# asset from the Content Forge catalog and only adds lore plus explicit tuning
# deltas; the preview always shows what the entry plays as and where it differs.
#
#   [world ▾] [+ New World]  Overview|Regions|Locations|...   ● unsaved  Save Reload Validate
#   ┌ list (merged) ─┐ ┌ form (WorldSchema) ────────────┐ ┌ preview ──────────────────┐
#   │ ↑ = inherited  │ │ Apply  Revert                  │ │ lore + what it plays as,  │
#   │ New Dup Override Delete │ fields...               │ │ problems, ▶ Playtest      │
#   └────────────────┘ └────────────────────────────────┘ └───────────────────────────┘
#
# Inherited entries (↑ world) are read-only here: Override copies one into the
# current world, where it replaces the original for this world and everything
# below it. Edits live in a draft; Apply stores it in memory, Save writes the JSON.
# ==============================================================================

signal playtest_requested(encounter: CatalogEncounter)

const OVERVIEW: String = "world"

var world_id: String = ""
var kind: String = OVERVIEW
var selected_id: String = ""        # local id of the OWN entry being edited ("" = new draft)
var selected_inherited: String = "" # qualified id of an inherited entry being viewed
var draft_dirty: bool = false
var form: ForgeForm = null
var source: WorldForgeSource = null

var _world_pick: OptionButton = null
var _new_world_row: HBoxContainer = null
var _new_world_id: LineEdit = null
var _new_world_name: LineEdit = null
var _new_world_parent: OptionButton = null
var _kind_buttons: Dictionary = {}
var _list: ItemList = null
var _list_refs: Array[String] = []
var _search: LineEdit = null
var _title: Label = null
var _status: Label = null
var _dirty_label: Label = null
var _preview_icon: TextureRect = null
var _preview: RichTextLabel = null
var _problems: RichTextLabel = null
var _playtest_btn: Button = null
var _delete_btn: Button = null
var _override_btn: Button = null
var _dup_btn: Button = null
var _new_btn: Button = null
var _form_scroll: ScrollContainer = null
var _inherit_note: Label = null
var _pending_delete: String = ""

func _init() -> void:
	name = "WorldForge"
	set_anchors_preset(Control.PRESET_FULL_RECT)
	_build()

func _ready() -> void:
	var worlds: Array[String] = WorldCatalog.world_ids()
	set_world(worlds[0] if not worlds.is_empty() else "")

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
	root.add_theme_constant_override("separation", 6)
	add_child(root)

	var bar := HBoxContainer.new()
	bar.add_theme_constant_override("separation", 6)
	root.add_child(bar)
	bar.add_child(UiTheme.label("🌍 WORLD FORGE", UiTheme.FONT_LG, UiTheme.COL_TITLE))
	_world_pick = OptionButton.new()
	_world_pick.tooltip_text = "The world being edited"
	_world_pick.custom_minimum_size.x = 170
	_world_pick.item_selected.connect(func(i: int) -> void: set_world(str(_world_pick.get_item_metadata(i))))
	bar.add_child(_world_pick)
	bar.add_child(UiTheme.button("+ New World", "Create a world (optionally extending another)", func() -> void: _new_world_row.visible = not _new_world_row.visible, UiTheme.FONT_MD))
	bar.add_child(UiTheme.button("🗑 Delete World", "Delete the selected world (click twice; refused while another world uses it)", func() -> void: delete_world(), UiTheme.FONT_MD))
	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bar.add_child(spacer)
	_dirty_label = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_WARN)
	bar.add_child(_dirty_label)
	bar.add_child(UiTheme.button("💾 Save", "Write every world to %s (Ctrl+S)" % WorldCatalog.get_root(), func() -> void: save_all(), UiTheme.FONT_MD))
	bar.add_child(UiTheme.button("⟲ Reload", "Discard unsaved changes and reload the worlds from disk", func() -> void: reload_all(), UiTheme.FONT_MD))
	bar.add_child(UiTheme.button("✔ Validate All", "Check every world, entry and cross-reference", func() -> void: validate_all(), UiTheme.FONT_MD))

	_new_world_row = HBoxContainer.new()
	_new_world_row.visible = false
	_new_world_row.add_theme_constant_override("separation", 6)
	root.add_child(_new_world_row)
	_new_world_row.add_child(UiTheme.label("New world  id:", UiTheme.FONT_MD, UiTheme.COL_MUTED))
	_new_world_id = LineEdit.new()
	_new_world_id.placeholder_text = "snake_case_id"
	_new_world_id.custom_minimum_size.x = 150
	_new_world_row.add_child(_new_world_id)
	_new_world_row.add_child(UiTheme.label("name:", UiTheme.FONT_MD, UiTheme.COL_MUTED))
	_new_world_name = LineEdit.new()
	_new_world_name.placeholder_text = "Display name"
	_new_world_name.custom_minimum_size.x = 180
	_new_world_row.add_child(_new_world_name)
	_new_world_row.add_child(UiTheme.label("extends:", UiTheme.FONT_MD, UiTheme.COL_MUTED))
	_new_world_parent = OptionButton.new()
	_new_world_row.add_child(_new_world_parent)
	_new_world_row.add_child(UiTheme.button("Create", "Create the world", func() -> void: create_world(_new_world_id.text.strip_edges(), _new_world_name.text.strip_edges(), _selected_parent()), UiTheme.FONT_MD))

	var kinds := HBoxContainer.new()
	kinds.add_theme_constant_override("separation", 4)
	root.add_child(kinds)
	var all_kinds: Array[String] = [OVERVIEW]
	all_kinds.append_array(WorldSchema.KINDS)
	for k: String in all_kinds:
		var btn: Button = UiTheme.button("%s %s" % [WorldSchema.kind_icon(k), WorldSchema.kind_label(k)], "Edit %s" % WorldSchema.kind_label(k).to_lower(), func() -> void: show_kind(k), UiTheme.FONT_MD)
		btn.toggle_mode = true
		_kind_buttons[k] = btn
		kinds.add_child(btn)

	var body := HBoxContainer.new()
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	body.add_theme_constant_override("separation", 8)
	root.add_child(body)

	var left: VBoxContainer = _column(body, 270)
	_search = LineEdit.new()
	_search.placeholder_text = "Search name or id..."
	_search.clear_button_enabled = true
	_search.text_changed.connect(func(_t: String) -> void: refresh_list())
	left.add_child(_search)
	_list = ItemList.new()
	_list.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_list.item_selected.connect(func(i: int) -> void:
		if i >= 0 and i < _list_refs.size():
			select(_list_refs[i])
	)
	left.add_child(_list)
	var btns := HFlowContainer.new()
	left.add_child(btns)
	_new_btn = UiTheme.button("+ New", "Start a new entry in this world", func() -> void: new_entry())
	btns.add_child(_new_btn)
	_dup_btn = UiTheme.button("⧉ Duplicate", "Copy the selected entry as a template", func() -> void: duplicate_selected())
	btns.add_child(_dup_btn)
	_override_btn = UiTheme.button("⇪ Override", "Copy the selected inherited entry into this world, replacing the original here and below", func() -> void: override_selected())
	btns.add_child(_override_btn)
	_delete_btn = UiTheme.button("🗑 Delete", "Delete the selected entry (click twice)", func() -> void: delete_selected())
	btns.add_child(_delete_btn)

	var centre: VBoxContainer = _column(body, 0)
	centre.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var head := HBoxContainer.new()
	centre.add_child(head)
	_title = UiTheme.label("", UiTheme.FONT_LG, UiTheme.COL_HEADER)
	_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(_title)
	head.add_child(UiTheme.button("✔ Apply", "Store the edits (Save writes them to disk)", func() -> void: apply(), UiTheme.FONT_MD))
	head.add_child(UiTheme.button("↺ Revert", "Discard the edits to this entry", func() -> void: revert(), UiTheme.FONT_MD))
	_inherit_note = UiTheme.label("", UiTheme.FONT_MD, UiTheme.COL_WARN)
	_inherit_note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_inherit_note.visible = false
	centre.add_child(_inherit_note)
	_form_scroll = ScrollContainer.new()
	_form_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_form_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	centre.add_child(_form_scroll)
	form = ForgeForm.new()
	form.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	form.edited.connect(_on_form_edited)
	_form_scroll.add_child(form)

	var right: VBoxContainer = _column(body, 350)
	right.add_child(UiTheme.header("WHAT IT MEANS"))
	_preview_icon = UiTheme.icon_rect(null, 72)
	right.add_child(_preview_icon)
	_preview = RichTextLabel.new()
	_preview.bbcode_enabled = true
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
	_playtest_btn = UiTheme.button("▶ Play this encounter", "Run it in the real combat arena with this world's characters", func() -> void: playtest(), UiTheme.FONT_MD)
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
# Worlds
# ------------------------------------------------------------------------------

func refresh_worlds() -> void:
	_world_pick.clear()
	_new_world_parent.clear()
	_new_world_parent.add_item("(none: a root world)")
	_new_world_parent.set_item_metadata(0, "")
	for w: String in WorldCatalog.world_ids():
		var label: String = "%s  [%s]" % [str(WorldCatalog.manifest(w).get("name", w)), w]
		_world_pick.add_item(label)
		_world_pick.set_item_metadata(_world_pick.item_count - 1, w)
		if w == world_id:
			_world_pick.select(_world_pick.item_count - 1)
		_new_world_parent.add_item(label)
		_new_world_parent.set_item_metadata(_new_world_parent.item_count - 1, w)

func _selected_parent() -> Array[String]:
	var out: Array[String] = []
	var chosen: String = str(_new_world_parent.get_item_metadata(maxi(0, _new_world_parent.selected)))
	if not chosen.is_empty():
		out.append(chosen)
	return out

# Switches the edited world (committing a valid draft first).
func set_world(p_world_id: String) -> void:
	if p_world_id != world_id and not commit_draft():
		refresh_worlds()
		return
	world_id = p_world_id
	source = WorldForgeSource.new(world_id)
	form.source = source
	refresh_worlds()
	selected_id = ""
	selected_inherited = ""
	show_kind(kind if not world_id.is_empty() else OVERVIEW, true)

func create_world(new_id: String, new_name: String, parent: Array[String]) -> bool:
	var errors: Array[String] = WorldCatalog.create_world(new_id, new_name if not new_name.is_empty() else new_id.capitalize(), parent)
	if not errors.is_empty():
		_show_problems(errors)
		_set_status("World not created: %s" % errors[0], true)
		return false
	_new_world_row.visible = false
	_new_world_id.text = ""
	_new_world_name.text = ""
	set_world(new_id)
	_set_status("Created world %s (Save to write it to disk)." % new_id)
	return true

func delete_world() -> void:
	if world_id.is_empty():
		return
	if _pending_delete != "world:" + world_id:
		_pending_delete = "world:" + world_id
		_set_status("Click Delete World again to remove %s." % world_id)
		return
	_pending_delete = ""
	var problem: String = WorldCatalog.remove_world(world_id)
	if not problem.is_empty():
		_set_status(problem, true)
		return
	draft_dirty = false
	_set_status("Deleted world %s (Save to remove its files)." % world_id)
	var worlds: Array[String] = WorldCatalog.world_ids()
	world_id = ""
	set_world(worlds[0] if not worlds.is_empty() else "")

# ------------------------------------------------------------------------------
# Navigation
# ------------------------------------------------------------------------------

func show_kind(p_kind: String, force: bool = false) -> void:
	if not force and not commit_draft():
		_sync_kind_buttons()
		return
	kind = p_kind
	_sync_kind_buttons()
	selected_id = ""
	selected_inherited = ""
	_playtest_btn.visible = kind == "encounters"
	var overview: bool = kind == OVERVIEW
	_new_btn.visible = not overview
	_dup_btn.visible = not overview
	_override_btn.visible = not overview
	_delete_btn.visible = not overview
	_search.visible = not overview
	refresh_list()
	if overview:
		_open_manifest()
	elif not _list_refs.is_empty():
		select(_list_refs[0], true)
	elif not world_id.is_empty():
		new_entry(true)
	else:
		form.visible = false

func _sync_kind_buttons() -> void:
	for k: Variant in _kind_buttons:
		var btn: Button = _kind_buttons[k]
		btn.set_pressed_no_signal(str(k) == kind)

func _open_manifest() -> void:
	_list_worlds()
	_inherit_note.visible = false
	form.visible = not world_id.is_empty()
	if world_id.is_empty():
		_title.text = "No worlds yet: create one with + New World."
		_preview.text = ""
		return
	form.edit(OVERVIEW, WorldCatalog.manifest(world_id))
	draft_dirty = false
	_refresh_side_panels()

# Overview: the left list shows every world (click to switch), with its inheritance.
func _list_worlds() -> void:
	_list.clear()
	_list_refs.clear()
	for w: String in WorldCatalog.world_ids():
		var m: Dictionary = WorldCatalog.manifest(w)
		var parents: Array[String] = WorldCatalog._strings(m.get("extends", []))
		var extra: String = ("  ⇢ extends %s" % ", ".join(parents)) if not parents.is_empty() else "  (root)"
		_list.add_item("%s%s" % [str(m.get("name", w)), extra])
		_list.set_item_tooltip(_list.item_count - 1, w)
		_list_refs.append(w)
		if w == world_id:
			_list.select(_list.item_count - 1)

func refresh_list() -> void:
	if kind == OVERVIEW:
		_list_worlds()
		_update_dirty_label()
		return
	_list.clear()
	_list_refs.clear()
	if world_id.is_empty():
		_update_dirty_label()
		return
	var needle: String = _search.text.strip_edges().to_lower()
	for e: Dictionary in WorldCatalog.merged_entries(world_id, kind):
		var local: String = str(e.get("id", ""))
		var label: String = str(e.get("name", local))
		if not needle.is_empty() and not label.to_lower().contains(needle) and not local.contains(needle):
			continue
		var own: bool = str(e["_world"]) == world_id
		var mark: String = ""
		if own and not WorldCatalog.validate_entry(world_id, kind, e, local).is_empty():
			mark = "⚠ "
		var origin: String = "" if own else "  ↑ %s" % str(e["_world"])
		var extra: String = ""
		if kind == "characters":
			extra = "  · %s" % str(e.get("role", ""))
		if own and not str(e.get("override", "")).is_empty():
			extra += "  ⇪ override"
		_list.add_item("%s%s%s%s" % [mark, label, extra, origin])
		_list.set_item_tooltip(_list.item_count - 1, str(e["_qid"]))
		if not own:
			_list.set_item_custom_fg_color(_list.item_count - 1, UiTheme.COL_MUTED)
		_list_refs.append(WorldCatalog.ref_value(world_id, e))
		if (own and local == selected_id) or str(e["_qid"]) == selected_inherited:
			_list.select(_list.item_count - 1)
	_update_dirty_label()

# Opens entry `ref` (a local id or qualified id): own entries are editable, inherited ones read-only.
func select(ref: String, force: bool = false) -> void:
	if kind == OVERVIEW:
		if ref != world_id:
			set_world(ref)
		return
	if not force and not commit_draft():
		refresh_list()
		return
	var hit: Dictionary = WorldCatalog.resolve(world_id, kind, ref)
	if hit.is_empty():
		return
	_pending_delete = ""
	if str(hit["_world"]) == world_id:
		selected_id = str(hit.get("id", ""))
		selected_inherited = ""
		_inherit_note.visible = false
		form.visible = true
		form.edit(kind, WorldCatalog.get_own(world_id, kind, selected_id))
	else:
		selected_id = ""
		selected_inherited = str(hit["_qid"])
		var clean: Dictionary = hit.duplicate(true)
		clean.erase("_world")
		clean.erase("_qid")
		form.edit(kind, clean)
		form.visible = false
		_inherit_note.text = "Inherited from world '%s' (read-only here). Use ⇪ Override to copy it into '%s', where your copy replaces the original for this world and everything below it." % [str(hit["_world"]), world_id]
		_inherit_note.visible = true
	draft_dirty = false
	refresh_list()
	_refresh_side_panels()

func new_entry(force: bool = false) -> void:
	if world_id.is_empty() or kind == OVERVIEW:
		return
	if not force and not commit_draft():
		return
	var d: Dictionary = WorldSchema.defaults(kind)
	d["id"] = WorldCatalog.unique_id(world_id, kind, "new_" + kind.trim_suffix("s"))
	selected_id = ""
	selected_inherited = ""
	_inherit_note.visible = false
	form.visible = true
	form.edit(kind, d)
	draft_dirty = true
	_list.deselect_all()
	_refresh_side_panels()
	_set_status("New %s draft: fill it in and Apply." % kind.trim_suffix("s"))

func duplicate_selected() -> void:
	if selected_id.is_empty() or not commit_draft():
		return
	var copy_id: String = WorldCatalog.duplicate_entry(world_id, kind, selected_id)
	if copy_id.is_empty():
		_set_status("Couldn't duplicate %s." % selected_id, true)
		return
	_set_status("Duplicated %s as %s." % [selected_id, copy_id])
	select(copy_id, true)

# Copies the selected inherited entry into this world as an override.
func override_selected() -> void:
	if selected_inherited.is_empty():
		_set_status("Select an inherited (↑) entry to override.", true)
		return
	var new_id: String = WorldCatalog.override_entry(world_id, kind, selected_inherited)
	if new_id.is_empty():
		_set_status("Couldn't override %s." % selected_inherited, true)
		return
	_set_status("Overrode %s with %s:%s: edit the copy's lore." % [selected_inherited, world_id, new_id])
	select(new_id, true)

func delete_selected() -> void:
	if selected_id.is_empty():
		if not selected_inherited.is_empty():
			_set_status("Inherited entries belong to their own world; delete them there (or override them here).", true)
		else:
			new_entry(true)
		return
	if _pending_delete != selected_id:
		_pending_delete = selected_id
		_set_status("Click Delete again to remove %s." % selected_id)
		return
	var problem: String = WorldCatalog.remove(world_id, kind, selected_id)
	_pending_delete = ""
	if not problem.is_empty():
		_set_status(problem, true)
		return
	_set_status("Deleted %s (Save to write it to disk)." % selected_id)
	draft_dirty = false
	selected_id = ""
	refresh_list()
	if not _list_refs.is_empty():
		select(_list_refs[0], true)
	else:
		new_entry(true)

# ------------------------------------------------------------------------------
# Editing
# ------------------------------------------------------------------------------

func _on_form_edited(_key: String) -> void:
	draft_dirty = true
	_refresh_side_panels()
	_update_dirty_label()

func draft_problems() -> Array[String]:
	var errors: Array[String] = []
	for key: Variant in form.json_errors:
		errors.append(str(form.json_errors[key]))
	if kind == OVERVIEW:
		var changes: Dictionary = form.draft.duplicate(true)
		changes["id"] = world_id
		var probe: Dictionary = WorldSchema.normalize(OVERVIEW, changes)
		if str(probe.get("name", "")).strip_edges().is_empty():
			errors.append("The world needs a name.")
		return errors
	errors.append_array(WorldCatalog.validate_entry(world_id, kind, form.draft, selected_id))
	return errors

# What stops the draft from being stored (references may still dangle: they show as problems).
func blocking_problems() -> Array[String]:
	var errors: Array[String] = []
	for key: Variant in form.json_errors:
		errors.append(str(form.json_errors[key]))
	if kind == OVERVIEW:
		return draft_problems()
	errors.append_array(WorldCatalog.blocking_errors(world_id, kind, form.draft, selected_id))
	return errors

# Stores the draft; false (with the problems shown) if invalid.
func apply() -> bool:
	if not selected_inherited.is_empty():
		return true   # nothing editable
	var errors: Array[String] = blocking_problems()
	if errors.is_empty():
		if kind == OVERVIEW:
			errors = WorldCatalog.update_manifest(world_id, form.draft)
		else:
			errors = WorldCatalog.upsert(world_id, kind, form.draft, selected_id)
	if not errors.is_empty():
		_show_problems(errors)
		_set_status("Not applied: fix the problems listed on the right.", true)
		return false
	if kind != OVERVIEW:
		selected_id = str(form.draft.get("id", ""))
	draft_dirty = false
	refresh_worlds()
	refresh_list()
	_refresh_side_panels()
	var open_issues: int = draft_problems().size() if kind != OVERVIEW else 0
	_set_status("Applied %s (Save to write it to disk)%s." % [selected_id if kind != OVERVIEW else world_id, "; %d unresolved issue%s listed on the right" % [open_issues, "" if open_issues == 1 else "s"] if open_issues > 0 else ""])
	return true

func commit_draft() -> bool:
	if not draft_dirty or form == null or form.kind.is_empty():
		return true
	if form.kind != kind:
		return true
	return apply()

func revert() -> void:
	draft_dirty = false
	if kind == OVERVIEW:
		_open_manifest()
	elif selected_id.is_empty():
		new_entry(true)
	else:
		form.edit(kind, WorldCatalog.get_own(world_id, kind, selected_id))
		_refresh_side_panels()
	_set_status("Reverted.")

func save_all() -> bool:
	if not commit_draft():
		return false
	var err: Error = WorldCatalog.save_all()
	if err != OK:
		_set_status("Save failed (%s): %s is read-only in exported builds." % [error_string(err), WorldCatalog.get_root()], true)
		return false
	_update_dirty_label()
	_set_status("Saved the worlds to %s." % WorldCatalog.get_root())
	return true

func reload_all() -> void:
	WorldCatalog.reload_from_disk()
	draft_dirty = false
	var keep: String = world_id
	world_id = ""
	set_world(keep if WorldCatalog.has_world(keep) else (WorldCatalog.world_ids()[0] if not WorldCatalog.world_ids().is_empty() else ""))
	_set_status("Reloaded the worlds from %s." % WorldCatalog.get_root())

func validate_all() -> Dictionary:
	var problems: Dictionary = WorldCatalog.validate_all()
	if problems.is_empty():
		_problems.text = "[color=#4ade80]✔ Every world, entry and reference is valid.[/color]"
		_set_status("All %d worlds valid." % WorldCatalog.world_ids().size())
	else:
		var lines: Array[String] = []
		for key: Variant in problems:
			for err: Variant in DictUtil.array_of(problems[key]):
				lines.append("[color=#f87171]%s[/color]: %s" % [str(key), str(err)])
		_problems.text = "\n".join(lines)
		_set_status("%d entries have problems." % problems.size(), true)
	return problems

# ------------------------------------------------------------------------------
# Playtest
# ------------------------------------------------------------------------------

func playtest() -> void:
	if not commit_draft():
		return
	var ref: String = selected_id if not selected_id.is_empty() else selected_inherited
	if kind != "encounters" or ref.is_empty():
		_set_status("Select an encounter to play.", true)
		return
	var hit: Dictionary = WorldCatalog.resolve(world_id, "encounters", ref)
	if hit.is_empty():
		_set_status("That encounter no longer exists.", true)
		return
	var errors: Array[String] = WorldCatalog.validate_entry(str(hit["_world"]), "encounters", hit, str(hit.get("id", "")))
	if not errors.is_empty():
		_show_problems(errors)
		return
	var enc: WorldEncounter = WorldEncounter.new(world_id, str(hit.get("id", "")))
	_set_status("Playing %s in %s..." % [enc.encounter_name, world_id])
	playtest_requested.emit(enc)

# ------------------------------------------------------------------------------
# Side panels
# ------------------------------------------------------------------------------

func _current_entry() -> Dictionary:
	return form.draft

func _refresh_side_panels() -> void:
	if world_id.is_empty():
		return
	var title: String = str(form.draft.get("name", ""))
	var id: String = str(form.draft.get("id", ""))
	if kind == OVERVIEW:
		_title.text = "%s  [%s]%s" % [title, world_id, "  ● edited" if draft_dirty else ""]
	elif not selected_inherited.is_empty():
		_title.text = "%s  [%s]  (inherited)" % [title, selected_inherited]
	else:
		_title.text = "%s  [%s]%s" % [title, id, "  ● edited" if draft_dirty else ""]
		if not selected_id.is_empty() and id != selected_id:
			_title.text += "  (renaming %s)" % selected_id
	_preview_icon.texture = WorldPreview.texture_for(kind, form.draft)
	_preview_icon.visible = _preview_icon.texture != null
	_preview.text = WorldPreview.describe(world_id, kind, form.draft)
	if selected_inherited.is_empty():
		_show_problems(draft_problems())
	else:
		_problems.text = "[color=#94a3b8]Read-only (inherited).[/color]"

func _show_problems(errors: Array[String]) -> void:
	if errors.is_empty():
		_problems.text = "[color=#4ade80]✔ No problems.[/color]"
		return
	var lines: Array[String] = []
	for err: String in errors:
		lines.append("[color=#f87171]•[/color] " + err)
	_problems.text = "\n".join(lines)

func _update_dirty_label() -> void:
	_dirty_label.text = "● unsaved changes" if WorldCatalog.is_dirty() else ""

func _set_status(text: String, bad: bool = false) -> void:
	_status.text = text
	_status.add_theme_color_override("font_color", UiTheme.COL_BAD if bad else UiTheme.COL_MUTED)

func status_text() -> String:
	return _status.text
