class_name SkillRanksPanel
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Skill ranks panel (FORGE1 section 3; read-only since TOOLS-RO)
# The ranks section of the skill viewer: one column per rank (rank 1 = the template values), the modifier cells of ranks 2+
# (absolute values, modified cells highlighted), and one resolved panel per rank (name, cost, numbers, rendered
# description). All rules come from SkillFormModel; the panel only renders them. Skills are edited through
# ContentCatalog.upsert seed scripts, not here.
# ==============================================================================

const MODIFIED_COLOR: Color = Color(1.0, 0.85, 0.4)

var entry: Dictionary = {}
var head: String = ""

func _init() -> void:
	name = "RanksPanel"
	add_theme_constant_override("separation", 6)

func show_entry(p_entry: Dictionary, p_head: String = "") -> void:
	entry = p_entry
	head = p_head
	UiTheme.clear_children(self)
	add_child(UiTheme.header("Ranks"))
	var view: Dictionary = SkillFormModel.ranks_view(entry, head)
	var rows: Array = view["rows"]
	if rows.size() > 1:
		add_child(_table(view))
	_add_resolved(rows)

func _table(view: Dictionary) -> Control:
	var rows: Array = view["rows"]
	var fields: Array = view["fields"]
	var grid := GridContainer.new()
	grid.name = "RankTable"
	grid.columns = rows.size() + 1
	grid.add_theme_constant_override("h_separation", 12)
	grid.add_theme_constant_override("v_separation", 4)
	grid.add_child(UiTheme.label("Field", UiTheme.FONT_SM, UiTheme.COL_MUTED))
	for raw: Variant in rows:
		var r: Dictionary = DictUtil.dict_of(raw)
		var rank: int = DictUtil.to_int(r["rank"])
		grid.add_child(UiTheme.label("Rank %d%s" % [rank, " (template)" if rank == 1 else ""], UiTheme.FONT_SM, UiTheme.COL_HEADER))
	var keys: Array[String] = []
	for key: Variant in fields:
		keys.append(str(key))
	keys.append("unlock_offset")
	for key: String in keys:
		var f: Dictionary = ContentSchema.field("skills", key)
		var label: Label = UiTheme.label(str(f.get("label", key)), UiTheme.FONT_SM, UiTheme.COL_MUTED)
		label.tooltip_text = str(f.get("help", ""))
		label.mouse_filter = Control.MOUSE_FILTER_STOP
		grid.add_child(label)
		for raw: Variant in rows:
			var r: Dictionary = DictUtil.dict_of(raw)
			grid.add_child(_cell(key, f, r, DictUtil.to_int(r["rank"])))
	return grid

# One read-only cell: the effective value of `key` at `rank` (rank 2+ cells a rank modifies are highlighted).
func _cell(key: String, f: Dictionary, row: Dictionary, rank: int) -> Control:
	var values: Dictionary = DictUtil.dict_of(row["values"])
	var shown: String = _show(values.get(key, f.get("default", 0)))
	if key == "unlock_offset" and rank == 1:
		shown = "+0"
	var cell: Label = UiTheme.label(shown, UiTheme.FONT_SM)
	cell.name = "Cell_%s_%d" % [key, rank]
	if rank > 1 and DictUtil.array_of(row["modified"]).has(key):
		cell.modulate = MODIFIED_COLOR
		cell.tooltip_text = "Rank %d modifies this value" % rank
	return cell

func _show(value: Variant) -> String:
	if value is Array:
		var parts: Array[String] = []
		for v: Variant in value:
			parts.append(str(v))
		return ", ".join(parts) if not parts.is_empty() else "(none)"
	if value is float:
		return "%s" % snappedf(DictUtil.to_float(value), 0.01)
	var text: String = str(value)
	return text if not text.is_empty() else "(none)"

# Read-only resolved panel per rank.
func _add_resolved(rows: Array) -> void:
	for raw: Variant in rows:
		var r: Dictionary = DictUtil.dict_of(raw)
		var rank: int = DictUtil.to_int(r["rank"])
		var values: Dictionary = DictUtil.dict_of(r["values"])
		var parts: Array[String] = ["%d AP" % DictUtil.to_int(values.get("level", 1), 1), SkillText.plural(DictUtil.to_int(values.get("blocks", 1), 1), "block")]
		for key: Variant in values:
			var k: String = str(key)
			if ["level", "blocks", "unlock_offset"].has(k):
				continue
			var f: Dictionary = ContentSchema.field("skills", k)
			var value: Variant = values[key]
			if value == f.get("default", null) and not ["power_mult", "mitigation", "boost"].has(k):
				continue
			parts.append("%s %s" % [str(f.get("label", k)), _show(value)])
		var panel := PanelContainer.new()
		panel.name = "Resolved_%d" % rank
		panel.add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_PANEL_BG, UiTheme.COL_BORDER, 4, 6, 6))
		var col := VBoxContainer.new()
		panel.add_child(col)
		col.add_child(UiTheme.label("%s  (rank %d%s)" % [str(r["name"]), rank, ", template" if rank == 1 else ""], UiTheme.FONT_MD, UiTheme.COL_HEADER))
		var detail: Label = UiTheme.label(" · ".join(parts), UiTheme.FONT_SM, UiTheme.COL_MUTED)
		detail.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		detail.custom_minimum_size.x = 60
		col.add_child(detail)
		var text: Label = UiTheme.label(str(r["text"]), UiTheme.FONT_SM)
		text.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		text.custom_minimum_size.x = 60
		col.add_child(text)
		add_child(panel)
