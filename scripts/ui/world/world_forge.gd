class_name WorldForge
extends Control

# ==============================================================================
# Tactical RPG - World Forge (read-only viewer for lore worlds; TOOLS-RO)
# Browse every world and its regions, locations, factions, characters (party / NPCs / enemies), named items,
# disciplines, quests, lore and encounters, merged with what the world inherits. The preview shows what each entry
# plays as (its base asset from the content catalog) and where it differs; the problems panel lists its validation
# problems; encounters can be played in the real arena. The viewer never writes: worlds are edited through WorldCatalog
# seed scripts, and WorldCatalog.validate_all runs over the shipped worlds in the test suites.
#
#   [world ▾]  Overview|Regions|Locations|...                         Reload  Validate All
#   ┌ list (merged) ─┐ ┌ fields (read-only, WorldSchema) ─┐ ┌ preview ──────────────────┐
#   │ ↑ = inherited  │ │                                  │ │ lore + what it plays as,  │
#   │                │ │                                  │ │ problems, ▶ Playtest      │
#   └────────────────┘ └──────────────────────────────────┘ └───────────────────────────┘
# ==============================================================================

signal playtest_requested(encounter: CatalogEncounter)

const OVERVIEW: String = "world"

var world_id: String = ""
var kind: String = OVERVIEW
var selected_id: String = ""        # local id of the OWN entry shown ("" = none or inherited)
var selected_inherited: String = "" # qualified id of an inherited entry shown
var form: ForgeForm = null
var source: WorldForgeSource = null

var _world_pick: OptionButton = null
var _kind_buttons: Dictionary = {}
var _list: ItemList = null
var _list_refs: Array[String] = []
var _search: LineEdit = null
var _title: Label = null
var _status: Label = null
var _preview_icon: TextureRect = null
var _preview: RichTextLabel = null
var _problems: RichTextLabel = null
var _playtest_btn: Button = null
var _inherit_note: Label = null

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
	bar.add_child(UiTheme.label("🌍 WORLD FORGE (viewer)", UiTheme.FONT_LG, UiTheme.COL_TITLE))
	_world_pick = OptionButton.new()
	_world_pick.tooltip_text = "The world shown"
	_world_pick.custom_minimum_size.x = 170
	_world_pick.item_selected.connect(func(i: int) -> void: set_world(str(_world_pick.get_item_metadata(i))))
	bar.add_child(_world_pick)
	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bar.add_child(spacer)
	bar.add_child(UiTheme.button("⟲ Reload", "Reload the worlds from %s (picks up seed-script edits)" % WorldCatalog.get_root(), func() -> void: reload_all(), UiTheme.FONT_MD))
	bar.add_child(UiTheme.button("✔ Validate All", "Check every world, entry and cross-reference", func() -> void: validate_all(), UiTheme.FONT_MD))

	var kinds := HBoxContainer.new()
	kinds.add_theme_constant_override("separation", 4)
	root.add_child(kinds)
	var all_kinds: Array[String] = [OVERVIEW]
	all_kinds.append_array(WorldSchema.KINDS)
	for k: String in all_kinds:
		var btn: Button = UiTheme.button("%s %s" % [WorldSchema.kind_icon(k), WorldSchema.kind_label(k)], "Browse %s" % WorldSchema.kind_label(k).to_lower(), func() -> void: show_kind(k), UiTheme.FONT_MD)
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

	var centre: VBoxContainer = _column(body, 0)
	centre.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_title = UiTheme.label("", UiTheme.FONT_LG, UiTheme.COL_HEADER)
	_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	centre.add_child(_title)
	_inherit_note = UiTheme.label("", UiTheme.FONT_MD, UiTheme.COL_WARN)
	_inherit_note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_inherit_note.visible = false
	centre.add_child(_inherit_note)
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	centre.add_child(scroll)
	form = ForgeForm.new()
	form.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(form)

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

# ------------------------------------------------------------------------------
# Worlds
# ------------------------------------------------------------------------------

func refresh_worlds() -> void:
	_world_pick.clear()
	for w: String in WorldCatalog.world_ids():
		_world_pick.add_item("%s  [%s]" % [str(WorldCatalog.manifest(w).get("name", w)), w])
		_world_pick.set_item_metadata(_world_pick.item_count - 1, w)
		if w == world_id:
			_world_pick.select(_world_pick.item_count - 1)

# Switches the world shown.
func set_world(p_world_id: String) -> void:
	world_id = p_world_id
	source = WorldForgeSource.new(world_id)
	form.source = source
	refresh_worlds()
	selected_id = ""
	selected_inherited = ""
	show_kind(kind if not world_id.is_empty() else OVERVIEW)

# ------------------------------------------------------------------------------
# Navigation
# ------------------------------------------------------------------------------

func show_kind(p_kind: String) -> void:
	kind = p_kind
	_sync_kind_buttons()
	selected_id = ""
	selected_inherited = ""
	_playtest_btn.visible = kind == "encounters"
	var overview: bool = kind == OVERVIEW
	_search.visible = not overview
	refresh_list()
	if overview:
		_open_manifest()
	elif not _list_refs.is_empty():
		select(_list_refs[0])
	else:
		form.visible = false
		_inherit_note.visible = false
		_title.text = "No %s in this world." % kind
		_preview.text = ""
		_problems.text = ""

func _sync_kind_buttons() -> void:
	for k: Variant in _kind_buttons:
		var btn: Button = _kind_buttons[k]
		btn.set_pressed_no_signal(str(k) == kind)

func _open_manifest() -> void:
	_list_worlds()
	_inherit_note.visible = false
	form.visible = not world_id.is_empty()
	if world_id.is_empty():
		_title.text = "No worlds yet (worlds are added through WorldCatalog seed scripts)."
		_preview.text = ""
		return
	form.edit(OVERVIEW, WorldCatalog.manifest(world_id))
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
		return
	_list.clear()
	_list_refs.clear()
	if world_id.is_empty():
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

# Shows entry `ref` (a local id or qualified id); inherited entries show with a note naming their world.
func select(ref: String) -> void:
	if kind == OVERVIEW:
		if ref != world_id:
			set_world(ref)
		return
	var hit: Dictionary = WorldCatalog.resolve(world_id, kind, ref)
	if hit.is_empty():
		return
	form.visible = true
	if str(hit["_world"]) == world_id:
		selected_id = str(hit.get("id", ""))
		selected_inherited = ""
		_inherit_note.visible = false
		form.edit(kind, WorldCatalog.get_own(world_id, kind, selected_id))
	else:
		selected_id = ""
		selected_inherited = str(hit["_qid"])
		var clean: Dictionary = hit.duplicate(true)
		clean.erase("_world")
		clean.erase("_qid")
		form.edit(kind, clean)
		_inherit_note.text = "Inherited from world '%s': '%s' sees it unless it overrides it." % [str(hit["_world"]), world_id]
		_inherit_note.visible = true
	refresh_list()
	_refresh_side_panels()

func reload_all() -> void:
	WorldCatalog.reload_from_disk()
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
	var warn_lines: Array[String] = []   # soft lints (REP1-L review: reputation keys among them), shown below
	var warns: Dictionary = WorldCatalog.warnings_all()
	for w: Variant in warns:
		for line: Variant in DictUtil.array_of(warns[w]):
			warn_lines.append("[color=#facc15]warning[/color] %s" % str(line))
	if not warn_lines.is_empty():
		_problems.text += "\n" + "\n".join(warn_lines)
	return problems

# Problems of the entry shown (its own world's validators; the manifest's for the overview).
func entry_problems() -> Array[String]:
	if world_id.is_empty():
		var none: Array[String] = []
		return none
	if kind == OVERVIEW:
		return WorldCatalog.manifest_errors(world_id)
	if not selected_inherited.is_empty():
		var hit: Dictionary = WorldCatalog.resolve(world_id, kind, selected_inherited)
		return WorldCatalog.validate_entry(str(hit.get("_world", "")), kind, hit, str(hit.get("id", "")))
	return WorldCatalog.validate_entry(world_id, kind, form.draft, selected_id)

# ------------------------------------------------------------------------------
# Playtest
# ------------------------------------------------------------------------------

func playtest() -> void:
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

func _refresh_side_panels() -> void:
	if world_id.is_empty():
		return
	var title: String = str(form.draft.get("name", ""))
	if kind == OVERVIEW:
		_title.text = "%s  [%s]" % [title, world_id]
	elif not selected_inherited.is_empty():
		_title.text = "%s  [%s]  (inherited)" % [title, selected_inherited]
	else:
		_title.text = "%s  [%s]" % [title, str(form.draft.get("id", ""))]
	_preview_icon.texture = WorldPreview.texture_for(kind, form.draft)
	_preview_icon.visible = _preview_icon.texture != null
	_preview.text = WorldPreview.describe(world_id, kind, form.draft)
	_show_problems(entry_problems())

func _show_problems(errors: Array[String]) -> void:
	if errors.is_empty():
		_problems.text = "[color=#4ade80]✔ No problems.[/color]"
		return
	var lines: Array[String] = []
	for err: String in errors:
		lines.append("[color=#f87171]•[/color] " + err)
	_problems.text = "\n".join(lines)

func _set_status(text: String, bad: bool = false) -> void:
	_status.text = text
	_status.add_theme_color_override("font_color", UiTheme.COL_BAD if bad else UiTheme.COL_MUTED)

func status_text() -> String:
	return _status.text

# The problems panel text (tests read it).
func problems_text() -> String:
	return _problems.text
