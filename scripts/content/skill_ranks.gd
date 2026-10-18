class_name SkillRanks
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill ranks (FORGE1 sections 2-3)
# A ranked ENTRY (what skills.json version 2 stores and the Forge edits) is a family template
# plus up to two rank modifiers:  {id: "mend", name: "Mend", ..., "ranks": [{"level": 3, ...}, {...}]}.
# The engine keeps seeing flat per-rank rows (mend, mend_2, mend_3): expand() makes them, collapse()
# folds them back. A modifier is an ABSOLUTE replacement value for a rank-modifiable field and is
# resolved against the BASE (rank 1), never against the previous rank.
# Pure static functions; the flat rows carry the derived keys family / tier / template.
# ==============================================================================

# Fields a rank modifier may change (when the kind shows them). Everything else is base-only.
const RANK_KEYS: Array[String] = [
	"level", "blocks", "unlock_offset", "power_mult", "dash_tiles", "reach_min", "reach_max",
	"status_potency", "status_magnitude", "status_rounds", "heal_base", "heal_mult", "grant_status",
	"cleanse_statuses", "mitigation", "boost", "boost_rounds", "jump_tiles", "recover_ap",
]

# Combo fields (S12C-L) set on the family only: one value for all ranks; a rank modifier that sets one is refused
# with FAMILY_KEY_MESSAGE.
const FAMILY_KEYS: Array[String] = ["ends_in_stance", "disrupt", "exploit", "exploit_consume"]
const FAMILY_KEY_MESSAGE: String = "%s is set on the family, not per rank."

# Keys of a flat row that a ranked entry does not store per rank.
const DERIVED_KEYS: Array[String] = ["id", "family", "tier", "name", "description", "template"]

# Core keys always written to a ranked entry; every other key is omitted while it equals its default.
const CORE_KEYS: Array[String] = ["id", "name", "kind", "level", "blocks", "description"]

static func rank_fields(kind: String) -> Array[String]:
	var out: Array[String] = []
	for f: Dictionary in ContentSchema.fields("skills"):
		var key: String = str(f["key"])
		if RANK_KEYS.has(key) and SkillKinds.has_field(kind, key):
			out.append(key)
	return out

static func modifiers_of(entry: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for raw: Variant in DictUtil.array_of(entry.get("ranks", [])):
		out.append(DictUtil.dict_of(raw))
	return out

static func rank_count(entry: Dictionary) -> int:
	return modifiers_of(entry).size() + 1

static func rank_id(head: String, rank: int) -> String:
	return head if rank <= 1 else "%s_%d" % [head, rank]

static func rank_name(base_name: String, rank: int) -> String:
	return base_name if rank <= 1 else "%s %s" % [base_name, SkillCatalog.roman(rank)]

# The head (family) id of a flat row.
static func head_of(row: Dictionary) -> String:
	var fam: String = str(row.get("family", ""))
	return fam if not fam.is_empty() else str(row.get("id", ""))

# A ranked entry from a base block and its modifier rows (tests, seeds).
static func entry(base: Dictionary, modifiers: Array[Dictionary]) -> Dictionary:
	var out: Dictionary = base.duplicate(true)
	out["ranks"] = modifiers.duplicate(true)
	return out

# ------------------------------------------------------------------------------
# Expansion
# ------------------------------------------------------------------------------

static func _base(e: Dictionary) -> Dictionary:
	var raw: Dictionary = e.duplicate(true)
	raw.erase("ranks")
	raw.erase("family")
	raw.erase("tier")
	raw.erase("template")
	return ContentSchema.normalize("skills", raw)

# The EFFECTIVE flat row of `rank` (1-based): base overlaid by that rank's modifiers, with derived id / family / tier /
# name. `description` still holds the template (see resolve for the rendered text).
static func effective(e: Dictionary, rank: int) -> Dictionary:
	var base: Dictionary = _base(e)
	var head: String = str(base["id"])
	var row: Dictionary = base.duplicate(true)
	if rank >= 2:
		var mods: Array[Dictionary] = modifiers_of(e)
		if rank - 2 < mods.size():
			var mod: Dictionary = mods[rank - 2]
			for key: Variant in mod:
				if RANK_KEYS.has(str(key)):
					row[key] = mod[key]
	row = ContentSchema.normalize("skills", row)
	row["id"] = rank_id(head, rank)
	row["family"] = "" if rank <= 1 else head
	row["tier"] = maxi(1, rank)
	row["name"] = rank_name(str(base["name"]), rank)
	row["template"] = str(base["description"])
	return row

# The flat row of `rank` with its description rendered (or the literal override of that rank).
static func resolve(e: Dictionary, rank: int) -> Dictionary:
	var row: Dictionary = effective(e, rank)
	var template: String = str(row["template"])
	var literal: Variant = null
	if rank >= 2:
		var mods: Array[Dictionary] = modifiers_of(e)
		if rank - 2 < mods.size() and mods[rank - 2].has("description"):
			literal = mods[rank - 2]["description"]
	row["description"] = str(literal) if literal != null else SkillText.render(template, str(row["kind"]), row, rank)
	return row

static func expand(e: Dictionary) -> Array[Dictionary]:
	var rows: Array[Dictionary] = []
	for rank: int in range(1, rank_count(e) + 1):
		rows.append(resolve(e, rank))
	return rows

# ------------------------------------------------------------------------------
# Collapse
# ------------------------------------------------------------------------------

# Folds flat rows into ranked entries: {"entries": [entry...] (first-appearance order), "errors": [text...]}.
# A family that cannot be folded is reported (family and field named) and left out of "entries".
static func collapse(rows: Array[Dictionary]) -> Dictionary:
	var order: Array[String] = []
	var groups: Dictionary = {}
	for raw: Dictionary in rows:
		var row: Dictionary = ContentSchema.normalize("skills", raw)
		var head: String = head_of(row)
		if not groups.has(head):
			groups[head] = []
			order.append(head)
		var list: Array = groups[head]
		list.append(row)
	var entries_out: Array[Dictionary] = []
	var errors: Array[String] = []
	for head: String in order:
		var list: Array = groups[head]
		var family_rows: Array[Dictionary] = []
		family_rows.assign(list)
		var folded: Dictionary = fold(head, family_rows)
		var fold_errors: Array = folded["errors"]
		if fold_errors.is_empty():
			var ranked: Dictionary = DictUtil.dict_of(folded["entry"])
			entries_out.append(ranked)
		else:
			for err: Variant in fold_errors:
				errors.append(str(err))
	return {"entries": entries_out, "errors": errors}

# One family: {"entry": ranked entry, "errors": [text...]}.
static func fold(head: String, family_rows: Array[Dictionary]) -> Dictionary:
	var errors: Array[String] = []
	var sorted_rows: Array[Dictionary] = family_rows.duplicate()
	sorted_rows.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return DictUtil.to_int(a.get("tier", 1), 1) < DictUtil.to_int(b.get("tier", 1), 1))
	var first: Dictionary = sorted_rows[0]
	var kind: String = str(first["kind"])
	if str(first["id"]) != head or DictUtil.to_int(first["tier"], 1) != 1:
		errors.append("Family '%s' has no tier 1 skill with that id (found %s)." % [head, str(first["id"])])
	for i: int in range(sorted_rows.size()):
		var row: Dictionary = sorted_rows[i]
		var tier: int = DictUtil.to_int(row["tier"], 1)
		if tier != i + 1:
			errors.append("Family '%s': tiers must run 1..n without gaps (found tier %d at position %d)." % [head, tier, i + 1])
		elif str(row["id"]) != rank_id(head, tier):
			errors.append("Family '%s': tier %d must have the id '%s' (found '%s')." % [head, tier, rank_id(head, tier), str(row["id"])])
	if not errors.is_empty():
		return {"entry": {}, "errors": errors}
	# Data a ranked entry cannot hold is reported instead of silently dropped on save: a field the kind does not use set
	# to a non-default value (any rank), and an unknown key on rank 2+ that rank 1 does not carry with the same value.
	var schema: Array[Dictionary] = ContentSchema.fields("skills")
	for i: int in range(sorted_rows.size()):
		var row: Dictionary = sorted_rows[i]
		var rank: int = i + 1
		for f: Dictionary in schema:
			var key: String = str(f["key"])
			if SkillKinds.has_field(kind, key) or not row.has(key) or row[key] == f["default"]:
				continue
			errors.append("Family '%s': rank %d sets '%s', which %s skills do not use (it would be lost on save)." % [head, rank, key, kind])
		if rank < 2:
			continue
		for raw_key: Variant in row:
			var key: String = str(raw_key)
			if not ContentSchema.field("skills", key).is_empty():
				continue
			if not first.has(raw_key) or first[raw_key] != row[raw_key]:
				errors.append("Family '%s': rank %d has the unknown key '%s' that rank 1 does not share (only rank 1 keeps unknown keys)." % [head, rank, key])
	if not errors.is_empty():
		return {"entry": {}, "errors": errors}
	var base_template: String = str(first["template"]) if not str(first["template"]).is_empty() else SkillText.escape(str(first["description"]))
	var modifiers: Array[Dictionary] = []
	for i: int in range(1, sorted_rows.size()):
		var row: Dictionary = sorted_rows[i]
		var rank: int = i + 1
		for f: Dictionary in ContentSchema.fields("skills"):
			var key: String = str(f["key"])
			if DERIVED_KEYS.has(key) or RANK_KEYS.has(key) or not SkillKinds.has_field(kind, key):
				continue
			if row[key] != first[key]:
				errors.append("Family '%s': '%s' must match across ranks (rank %d differs)." % [head, key, rank])
		if str(row["name"]) != rank_name(str(first["name"]), rank):
			errors.append("Family '%s': rank %d name '%s' is not the derived name '%s'." % [head, rank, str(row["name"]), rank_name(str(first["name"]), rank)])
		if not str(row["template"]).is_empty() and str(row["template"]) != base_template:
			errors.append("Family '%s': the description template must match across ranks (rank %d differs)." % [head, rank])
		var mod: Dictionary = {}
		for key: String in rank_fields(kind):
			if row[key] != first[key]:
				mod[key] = row[key]
		var probe: Dictionary = row.duplicate(true)
		var expected: String = SkillText.render(base_template, kind, probe, rank)
		if str(row["description"]) != expected:
			mod["description"] = str(row["description"])
		modifiers.append(mod)
	if not errors.is_empty():
		return {"entry": {}, "errors": errors}
	var out: Dictionary = {}
	for f: Dictionary in ContentSchema.fields("skills"):
		var key: String = str(f["key"])
		if key == "family" or key == "tier" or key == "template":
			continue
		if not SkillKinds.has_field(kind, key):
			continue
		var value: Variant = first[key]
		if key == "id":
			value = head
		elif key == "description":
			value = base_template
		if not CORE_KEYS.has(key) and value == f["default"]:
			continue
		out[key] = value
	for key: Variant in first:
		if not out.has(key) and ContentSchema.field("skills", str(key)).is_empty():
			out[key] = first[key]
	if not modifiers.is_empty():
		out["ranks"] = modifiers
	return {"entry": out, "errors": errors}
