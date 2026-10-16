extends SceneTree
# SKILL1-A migration seed (idempotent; FORGE1_SKILLS section 8 and 5.5).
# 1. skills.json -> version 2 ranked entries; each family's per-rank descriptions become one token template
#    (full token pass, then varying-fields-only pass, then literal fallback with per-rank literal descriptions).
# 2. advanced -> grade and offhand -> weapon_hand happen through ContentSchema's legacy normalisation on load and are
#    written by save. `requires` pairings (section 11.1) are applied (user accepted the seven pairings, DECISIONS "FORGE1 skill answers 2").
# 3. Basic skills are removed from every class skill_path.
# 4. Audit: class paths / unit skills whose `requires` are not met (prints and stops).
# Every change goes through ContentCatalog (upsert_skill / upsert inside the system-edit scope).
# Run: tools/fc.sh run data/seeds/skill1_ranks.gd

const APPLY_PAIRINGS: bool = true
const PAIRINGS: Dictionary = {
	"greater_mend": ["mend"],
	"deadeye": ["aimed_shot"],
	"judgement": ["smite"],
	"charge_in": ["lunge"],
	"lacerate": ["bleeding_cut"],
	"assassinate": ["quick_stab"],
	"cyclone_kick": ["sweep_kick"],
}

# Token candidates in priority order (each token is placed at most once).
# Left out on purpose: `ap` and `unlock` ("1 AP" / "+1" in prose rarely mean the skill's own cost or offset) and the
# bare ":n" forms except reach ("+1 tile per move block" is the haste bonus, not the skill's blocks).
const CANDIDATES: Array[String] = [
	"heal", "power", "potency", "magnitude", "mitigation", "boost", "cleanse", "weapons", "status", "recover",
	"rounds", "dash", "reach", "jump", "blocks", "proficiency", "reach:n",
]
const MAX_LEAVES: int = 20000

var counts: Dictionary = {"full": 0, "varying": 0, "literal": 0}
var literal_families: Array[String] = []
var failures: Array[String] = []

func _init() -> void:
	ContentCatalog.reload_from_disk()
	var load_errs: Array[String] = ContentCatalog.load_errors()
	if not load_errs.is_empty():
		print("LOAD ERRORS: ", load_errs)
		quit(1)
		return
	var snapshot: Dictionary = {}
	for row: Dictionary in ContentCatalog.entries("skills"):
		snapshot[str(row["id"])] = _comparable(row)
	print("rows before: ", snapshot.size())

	# Caps (name 32 per derived rank name, description 240).
	var over: Array[String] = []
	for row: Dictionary in ContentCatalog.entries("skills"):
		if str(row["name"]).length() > 32:
			over.append("%s name %d" % [row["id"], str(row["name"]).length()])
		if str(row["description"]).length() > SkillText.MAX_LENGTH:
			over.append("%s description %d" % [row["id"], str(row["description"]).length()])
	if not over.is_empty():
		print("OVER CAPS (stop): ", over)
		quit(1)
		return

	ContentCatalog.begin_system_edit()
	var errs: Array[String] = []

	# Step 3 first: basics out of every class path (so the skill validation runs on a clean catalog).
	for c: Dictionary in ContentCatalog.entries("classes"):
		var cls: Dictionary = c.duplicate(true)
		var kept: Array = []
		var removed: Array[String] = []
		for raw: Variant in DictUtil.array_of(cls.get("skill_path", [])):
			var step: Dictionary = DictUtil.dict_of(raw)
			var sk: Dictionary = ContentCatalog.get_entry("skills", str(step.get("skill", "")))
			if DictUtil.to_bool(sk.get("basic", false)):
				removed.append(str(step["skill"]))
			else:
				kept.append(step)
		if removed.is_empty():
			continue
		cls["skill_path"] = kept
		var ce: Array[String] = ContentCatalog.upsert("classes", cls)
		print("class %s: removed basics %s %s" % [cls["id"], removed, "" if ce.is_empty() else str(ce)])
		errs.append_array(ce)

	# Step 1 (+2, +5 via normalisation): template every family.
	var heads: Array[String] = []
	for e: Dictionary in ContentCatalog.skill_entries():
		heads.append(str(e["id"]))
	for head: String in heads:
		var entry: Dictionary = _templated(head)
		if APPLY_PAIRINGS and PAIRINGS.has(head):
			entry["requires"] = PAIRINGS[head]
		var se: Array[String] = ContentCatalog.upsert_skill(entry)
		for m: String in se:
			errs.append("%s: %s" % [head, m])
	ContentCatalog.end_system_edit()

	print("upsert errors: ", errs)
	print("families: %d  templated fully %d, varying fields only %d, literal fallback %d" % [heads.size(), counts["full"], counts["varying"], counts["literal"]])
	print("literal fallback families: ", literal_families)
	if not failures.is_empty():
		print("TEMPLATE FAILURES (stop): ", failures)
		quit(1)
		return
	if not errs.is_empty():
		quit(1)
		return

	# Step 4: requires audit.
	var violators: Array[String] = _requires_audit()
	if not violators.is_empty():
		print("REQUIRES NOT MET (stop, user decides): ", violators)
		quit(1)
		return

	var va: Dictionary = ContentCatalog.validate_all()
	print("validate_all: ", va)
	print("warnings_all: ", ContentCatalog.warnings_all())
	if not va.is_empty():
		quit(1)
		return
	print("save_all: ", ContentCatalog.save_all())

	# Round trip: every expanded row must equal the snapshot field by field.
	ContentCatalog.reload_from_disk()
	print("load_errors after save: ", ContentCatalog.load_errors())
	var after: Array[Dictionary] = ContentCatalog.entries("skills")
	var diffs: Array[String] = []
	if after.size() != snapshot.size():
		diffs.append("row count %d != %d" % [after.size(), snapshot.size()])
	for row: Dictionary in after:
		var id: String = str(row["id"])
		if not snapshot.has(id):
			diffs.append("new row " + id)
			continue
		var was: Dictionary = snapshot[id]
		var now: Dictionary = _comparable(row)
		for key: Variant in was:
			if not now.has(key) or now[key] != was[key]:
				diffs.append("%s.%s: %s -> %s" % [id, key, str(was[key]), str(now.get(key, "<missing>"))])
		for key: Variant in now:
			if not was.has(key):
				diffs.append("%s.%s added: %s" % [id, key, str(now[key])])
	var with_ranks: int = 0
	var single: int = 0
	for e: Dictionary in ContentCatalog.skill_entries():
		if SkillRanks.rank_count(e) > 1:
			with_ranks += 1
		else:
			single += 1
	print("rows after: %d  entries: %d (%d ranked, %d single)" % [after.size(), with_ranks + single, with_ranks, single])
	print("round-trip diffs: ", diffs)
	print("validate_all after reload: ", ContentCatalog.validate_all())
	quit(0 if diffs.is_empty() else 1)

# A row without the derived template key (the snapshot rows from a version 1 file have none).
func _comparable(row: Dictionary) -> Dictionary:
	var out: Dictionary = ContentSchema.normalize("skills", row)
	out.erase("template")
	return out

func _stored_texts(head: String) -> Array[String]:
	var rows: Array[Dictionary] = ContentCatalog.family_rows(head)
	rows.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return DictUtil.to_int(a.get("tier", 1), 1) < DictUtil.to_int(b.get("tier", 1), 1))
	var out: Array[String] = []
	for r: Dictionary in rows:
		out.append(str(r["description"]))
	return out

# The family as a ranked entry with a token template (or the literal fallback).
func _templated(head: String) -> Dictionary:
	var base: Dictionary = ContentCatalog.skill_entry(head)
	var stored: Array[String] = _stored_texts(head)
	var plain: Dictionary = base.duplicate(true)
	var mods: Array[Dictionary] = SkillRanks.modifiers_of(plain)
	for m: Dictionary in mods:
		m.erase("description")
	if mods.is_empty():
		plain.erase("ranks")
	else:
		plain["ranks"] = mods
	var count: int = SkillRanks.rank_count(plain)
	var kind: String = str(plain["kind"])
	var rank1_text: String = stored[0]
	# Already templated (re-run): keep the stored template when it still reproduces every rank.
	var existing: String = str(base["description"])
	var all_tokens: Array[String] = []
	all_tokens.assign(CANDIDATES)
	var varying: Array[String] = _varying_tokens(plain, kind, count)
	var tries: Array[Array] = [["full", all_tokens], ["varying", varying]]
	for t: Array in tries:
		var label: String = t[0]
		var allowed: Array[String] = []
		allowed.assign(t[1])
		var template: String = _candidate(rank1_text, str(plain["name"]), plain, kind, allowed, stored)
		if not template.is_empty():
			var out: Dictionary = plain.duplicate(true)
			out["description"] = template
			counts[label] = DictUtil.to_int(counts[label]) + 1
			if template != existing:
				print("  %s [%s]: %s" % [head, label, template])
			return out
	if existing.contains("{") and not existing.contains("{{") and _reproduces(plain, existing, stored):
		var keep: Dictionary = plain.duplicate(true)
		keep["description"] = existing
		counts["full"] = DictUtil.to_int(counts["full"]) + 1
		return keep
	# Literal fallback: rank-1 text as a literal template, differing ranks as literal per-rank descriptions.
	var lit: Dictionary = plain.duplicate(true)
	lit["description"] = SkillText.escape(rank1_text)
	var lit_mods: Array[Dictionary] = SkillRanks.modifiers_of(lit)
	for i: int in range(lit_mods.size()):
		lit_mods[i]["description"] = stored[i + 1]
	if not lit_mods.is_empty():
		lit["ranks"] = lit_mods
	for i: int in range(lit_mods.size()):
		var rank: int = i + 2
		var rendered: String = SkillText.render(str(lit["description"]), kind, SkillRanks.effective(lit, rank), rank)
		if rendered == stored[i + 1]:
			lit_mods[i].erase("description")
	counts["literal"] = DictUtil.to_int(counts["literal"]) + 1
	literal_families.append(head)
	if not _reproduces(lit, str(lit["description"]), stored, true):
		failures.append(head)
	return lit

func _varying_tokens(entry: Dictionary, kind: String, count: int) -> Array[String]:
	var out: Array[String] = []
	for raw: String in CANDIDATES:
		var token: String = raw.get_slice(":", 0)
		if not SkillText.token_valid_for(token, kind):
			continue
		var suffix: String = raw.get_slice(":", 1) if raw.contains(":") else ""
		var first: String = SkillText.token_value(token, suffix, SkillRanks.effective(entry, 1), 1)
		for rank: int in range(2, count + 1):
			if SkillText.token_value(token, suffix, SkillRanks.effective(entry, rank), rank) != first:
				out.append(raw)
				break
	return out

# The first template (most tokens placed, earliest occurrence first) that reproduces every stored rank text, or "".
# Depth-first: for each candidate token in priority order, try each whole-word occurrence of its rank-1 value, then
# skipping it; a leaf is accepted only when the rendered text of every rank equals the stored text.
func _candidate(text: String, name: String, entry: Dictionary, kind: String, allowed: Array[String], stored: Array[String]) -> String:
	var template: String = SkillText.escape(text)
	if template.begins_with(name) and (template.length() == name.length() or not _is_word(template[name.length()])):
		template = "{name}" + template.substr(name.length())
	var row: Dictionary = SkillRanks.effective(entry, 1)
	var steps: Array[Array] = []
	var seen_tokens: Array[String] = []
	for raw: String in allowed:
		var token: String = raw.get_slice(":", 0)
		if seen_tokens.has(token) or not SkillText.token_valid_for(token, kind):
			continue
		if token == "unlock" and (DictUtil.to_bool(row.get("basic", false)) or DictUtil.to_bool(row.get("system", false))):
			continue
		var suffix: String = raw.get_slice(":", 1) if raw.contains(":") else ""
		var value: String = SkillText.token_value(token, suffix, row, 1)
		if value.is_empty():
			continue
		steps.append([raw, value])
	var budget: Array[int] = [MAX_LEAVES]
	return _search(template, steps, 0, [], entry, stored, budget)

func _search(template: String, steps: Array[Array], i: int, placed: Array, entry: Dictionary, stored: Array[String], budget: Array[int]) -> String:
	if budget[0] <= 0:
		return ""
	if i >= steps.size():
		budget[0] -= 1
		return template if _reproduces(entry, template, stored) else ""
	var raw: String = steps[i][0]
	var token: String = raw.get_slice(":", 0)
	if not placed.has(token):
		var re := RegEx.new()
		re.compile("(?<![\\w{])" + _re_escape(str(steps[i][1])) + "(?![\\w}])")
		for m: RegExMatch in re.search_all(template):
			var next: String = template.substr(0, m.get_start()) + "{" + raw + "}" + template.substr(m.get_end())
			var found: String = _search(next, steps, i + 1, placed + [token], entry, stored, budget)
			if not found.is_empty():
				return found
	return _search(template, steps, i + 1, placed, entry, stored, budget)

func _reproduces(entry: Dictionary, template: String, stored: Array[String], literal_ranks: bool = false) -> bool:
	var probe: Dictionary = entry.duplicate(true)
	probe["description"] = template
	if not literal_ranks:
		var mods: Array[Dictionary] = SkillRanks.modifiers_of(probe)
		for m: Dictionary in mods:
			m.erase("description")
		if not mods.is_empty():
			probe["ranks"] = mods
	if not SkillText.validate(template, probe).is_empty():
		return false
	var count: int = SkillRanks.rank_count(probe)
	if count != stored.size():
		return false
	for rank: int in range(1, count + 1):
		if str(SkillRanks.resolve(probe, rank)["description"]) != stored[rank - 1]:
			return false
	return true

func _is_word(ch: String) -> bool:
	return (ch >= "a" and ch <= "z") or (ch >= "A" and ch <= "Z") or (ch >= "0" and ch <= "9") or ch == "_"

func _re_escape(s: String) -> String:
	var out: String = ""
	for ch: String in s:
		if "\\.^$|?*+()[]{}/-".contains(ch):
			out += "\\" + ch
		else:
			out += ch
	return out

func _requires_audit() -> Array[String]:
	var out: Array[String] = []
	var req_of: Dictionary = {}
	for e: Dictionary in ContentCatalog.skill_entries():
		var reqs: Array = DictUtil.array_of(e.get("requires", []))
		if not reqs.is_empty():
			req_of[str(e["id"])] = reqs
	for c: Dictionary in ContentCatalog.entries("classes"):
		var at: Dictionary = {}
		for raw: Variant in DictUtil.array_of(c.get("skill_path", [])):
			var step: Dictionary = DictUtil.dict_of(raw)
			at[str(step.get("skill", ""))] = DictUtil.to_int(step.get("level", 1), 1)
		for sid: Variant in at:
			if not req_of.has(sid):
				continue
			for r: Variant in req_of[sid]:
				if not at.has(r) or DictUtil.to_int(at[r]) > DictUtil.to_int(at[sid]):
					out.append("class %s: %s needs %s" % [c["id"], sid, r])
	for u: Dictionary in ContentCatalog.entries("units"):
		var owned: Array = DictUtil.array_of(u.get("extra_skills", [])).duplicate()
		var cls: Dictionary = ContentCatalog.get_entry("classes", str(u.get("class", u.get("class_id", ""))))
		for raw: Variant in DictUtil.array_of(cls.get("skill_path", [])):
			owned.append(str(DictUtil.dict_of(raw).get("skill", "")))
		var used: Array = DictUtil.array_of(u.get("extra_skills", [])) + DictUtil.array_of(u.get("loadout", []))
		for sid: Variant in used:
			for r: Variant in DictUtil.array_of(req_of.get(str(sid), [])):
				if not owned.has(r):
					out.append("unit %s: %s needs %s" % [u["id"], sid, r])
	return out
