class_name ClassForgeModel
extends RefCounted

# ==============================================================================
# Tactical RPG - Class Forge Model (FORGE1_CLASSES 2-5)
# The headless brain of the class tab of the Content Forge (and the unit AI group): pure static functions over a class
# draft entry. Layout (the five sections from the schema `group`), each control's state (locked key fields on system
# classes, ranges equal to the rule constants, disabled options and the reason), a fresh valid class, the problems that
# disable Save (errors) and the warnings that never do, the skill path table (ordering, standard / custom levels, family
# picker, tier ladder, default-loadout preview), the stat, gear budget and proficiency coverage panels, and the gear
# pickers grouped by item family (dual-wield guard, duplicate-accessory flag). The controls only render what this returns.
# ==============================================================================

const SECTION_TITLES: Dictionary = {
	"identity": "Identity",
	"stats": "Stat behaviour",
	"proficiency": "Gear proficiency",
	"skills": "Skill progression",
	"loadout": "Starting gear",
	"ai": "AI",
	"ai_advanced": "AI (advanced)",
	"general": "Unit",
}
const LOCK_REASON: String = "System class: key field locked. Create your own copy to change it."
const ID_FIXED_REASON: String = "The id is fixed after the first save (rename through the catalog API rewrites references)."
const ROLE_HELP: Dictionary = {
	"AGGRESSOR": "Closes distance and trades blows with the nearest foe.",
	"SKIRMISHER": "Hunts the weakest foe and falls back when hurt.",
	"SNIPER": "Holds maximum range and backs away from melee (needs a range 2+ weapon).",
	"SUPPORT": "Heals and buffs allies first, then fights like a sniper.",
}
const PROFICIENCY_STEP: float = 0.05
const CUSTOM_LEVEL: int = -1   # level selector entry "Custom level..." (reveals a spinner 1..MAX_LEVEL)

# ------------------------------------------------------------------------------
# Layout and field state
# ------------------------------------------------------------------------------

# [{"id", "title", "fields": [keys]}] for `kind` "classes" (five sections) or "units" (the schema groups; AI last).
static func layout(kind: String = "classes", entry: Dictionary = {}) -> Array[Dictionary]:
	var order: Array[String] = []
	var by_group: Dictionary = {}
	for f: Dictionary in ContentSchema.fields(kind):
		var group: String = str(f.get("group", "identity" if kind == "classes" else "general"))
		var key: String = str(f["key"])
		if kind == "units" and not unit_field_visible(entry, key):
			continue
		if not by_group.has(group):
			by_group[group] = []
			order.append(group)
		var keys: Array = by_group[group]
		keys.append(key)
	if kind == "classes":
		order.assign(ClassUnitFields.CLASS_GROUPS)
	else:
		var preferred: Array[String] = ["general", ClassUnitFields.UNIT_AI_GROUP, ClassUnitFields.UNIT_AI_ADVANCED_GROUP, "loadout"]
		for group: String in order:
			if not preferred.has(group):
				preferred.append(group)
		order = preferred
	var out: Array[Dictionary] = []
	for group: String in order:
		if by_group.has(group):
			out.append({"id": group, "title": str(SECTION_TITLES.get(group, group.capitalize())), "fields": by_group[group]})
	return out

# The Forge group of a class / unit field ("" when the schema has none).
static func group_of(kind: String, key: String) -> String:
	return str(ContentSchema.field(kind, key).get("group", ""))

# True when the stored class `class_id` is a system asset whose key fields are locked (never inside the system-edit scope).
static func is_locked(class_id: String) -> bool:
	if class_id.is_empty() or ContentCatalog.in_system_edit() or not ContentCatalog.has("classes", class_id):
		return false
	return ContentCatalog.is_system_origin(ContentCatalog.get_entry("classes", class_id))

# State of one class control: {"enabled", "locked", "reason", "min", "max", "step", "options", "disabled_options": {value: reason}}.
# `stored_id` is the id of the stored class being edited ("" for a new draft).
static func state(entry: Dictionary, key: String, stored_id: String = "") -> Dictionary:
	var out: Dictionary = {"enabled": true, "locked": false, "reason": "", "min": null, "max": null, "step": null, "options": [], "disabled_options": {}}
	var f: Dictionary = ContentSchema.field("classes", key)
	if f.is_empty():
		return out
	if DictUtil.to_bool(f.get("readonly", false)):
		out["enabled"] = false
	if DictUtil.to_bool(f.get("key_field", false)) and is_locked(stored_id):
		out["enabled"] = false
		out["locked"] = true
		out["reason"] = LOCK_REASON
	elif key == "id" and not stored_id.is_empty():
		out["enabled"] = false
		out["reason"] = ID_FIXED_REASON
	match key:
		"ai_role":
			if not DictUtil.to_bool(out["locked"]):
				out["reason"] = str(ROLE_HELP.get(str(entry.get("ai_role", "")), ""))
		"main_hand", "off_hand", "head", "body", "accessory_1", "accessory_2":
			_slot_state(out, entry, key, "classes")
		"pouch":
			out["max"] = pouch_limit()
	return out

# Gear slot state shared by classes and units: grouped `choices` (records for ForgeForm), their disabled options, and the
# off hand disabled while the main hand is two-handed.
static func _slot_state(out: Dictionary, entry: Dictionary, key: String, kind: String) -> void:
	var disabled: Dictionary = out["disabled_options"]
	var choices: Array[Dictionary] = slot_choices(entry, key, kind)
	out["choices"] = choices
	for choice: Dictionary in choices:
		if not DictUtil.to_bool(choice["enabled"]):
			disabled[str(choice["id"])] = str(choice["reason"])
	if key == "off_hand":
		var main_id: String = str(entry.get("main_hand", ""))
		if main_id.is_empty() and kind == "units":
			var arch: ClassArchetype = ContentFactory.class_of(entry)
			main_id = str(arch.loadout.get("main_hand", "")) if arch != null else ""
		var main: Dictionary = ContentCatalog.get_entry("items", main_id)
		if DictUtil.to_bool(main.get("two_handed", false)):
			out["enabled"] = false
			out["reason"] = "The main hand holds a two-handed weapon: the off hand stays empty."

# State of one unit control (the unit tab uses the same hooks): gear pickers grouped by family (with "none"), the pouch limit,
# and the AI role dropdown whose empty entry names the class role.
static func unit_state(entry: Dictionary, key: String) -> Dictionary:
	var out: Dictionary = {"enabled": true, "locked": false, "reason": "", "min": null, "max": null, "step": null, "options": [], "disabled_options": {}}
	match key:
		"main_hand", "off_hand", "head", "body", "accessory_1", "accessory_2":
			_slot_state(out, entry, key, "units")
		"pouch":
			out["max"] = pouch_limit()
		"ai_role":
			var arch: ClassArchetype = ContentFactory.class_of(entry)
			out["empty_label"] = "Class role: %s" % (arch.ai_role if arch != null else "AGGRESSOR")
			out["reason"] = str(ROLE_HELP.get(effective_role(entry), ""))
	return out

# Control ranges of the class form: every value is a rule constant (acceptance 4, no duplicated literals).
static func ranges() -> Dictionary:
	return {
		"base_stats": {"min": StatRules.STAT_FLOOR, "max": StatRules.STAT_CEILING, "total": StatRules.CLASS_BUDGET},
		"growth": {"min": 0, "max": StatRules.GROWTH_STAT_MAX, "total": StatRules.GROWTH_TOTAL},
		"proficiency_level": {"min": 1, "max": Proficiency.MAX_LEVEL},
		"proficiency_scaling": {"min": Proficiency.MIN_SCALING, "max": Proficiency.MAX_SCALING, "step": PROFICIENCY_STEP, "default": Proficiency.DEFAULT_SCALING},
		"path_level": {"min": 1, "max": StatRules.MAX_LEVEL, "standard": ClassArchetype.STANDARD_PATH_LEVELS, "advanced_min": StatRules.ADVANCED_MIN_LEVEL},
		"name": {"max_len": ClassUnitFields.NAME_MAX_LEN},
		"description": {"max_len": ClassUnitFields.DESCRIPTION_MAX_LEN},
		"id": {"min_len": ClassUnitFields.ID_MIN_LEN, "max_len": ClassUnitFields.ID_MAX_LEN},
		"pouch": {"max": pouch_limit()},
		"heal_below": {"min": ClassUnitFields.HEAL_BELOW_MIN, "max": ClassUnitFields.HEAL_BELOW_MAX, "step": 0.05},
	}

static func pouch_limit() -> int:
	return Inventory.new().max_pouch_slots

# "Remaining: N" of a stats field: the budget (base 62, growth 5) minus the current total (0 = valid total).
static func remaining(entry: Dictionary, key: String) -> int:
	var stats: Dictionary = ContentFactory._ints(DictUtil.dict_of(entry.get(key, {})))
	if key == "growth":
		return StatRules.GROWTH_TOTAL - StatRules.total_of(stats)
	var total: int = 0
	for s: String in StatRules.STATS:
		total += DictUtil.int_of(stats, s, StatRules.STAT_FLOOR)
	return StatRules.CLASS_BUDGET - total

# ------------------------------------------------------------------------------
# New class, problems and Save state
# ------------------------------------------------------------------------------

# A fresh user class that validates: base stats from the floor up to the budget (round robin), growth 1 per stat up to the
# growth total, one weapon row (sword level 1, default scaling), empty path, no gear (FORGE1_CLASSES 2.0).
static func new_entry(id: String) -> Dictionary:
	var e: Dictionary = ContentSchema.defaults("classes")
	e["id"] = id
	e["origin"] = ContentSchema.ORIGIN_USER
	var base: Dictionary = {}
	for s: String in StatRules.STATS:
		base[s] = StatRules.STAT_FLOOR
	var left: int = StatRules.CLASS_BUDGET - StatRules.STAT_FLOOR * StatRules.STATS.size()
	var i: int = 0
	while left > 0:
		var stat: String = StatRules.STATS[i % StatRules.STATS.size()]
		if DictUtil.to_int(base[stat]) < StatRules.STAT_CEILING:
			base[stat] = DictUtil.to_int(base[stat]) + 1
			left -= 1
		i += 1
	e["base_stats"] = base
	var growth: Dictionary = {}
	var g_left: int = StatRules.GROWTH_TOTAL
	var j: int = 0
	while g_left > 0:
		var g_stat: String = StatRules.STATS[j % StatRules.STATS.size()]
		if DictUtil.int_of(growth, g_stat) < StatRules.GROWTH_STAT_MAX:
			growth[g_stat] = DictUtil.int_of(growth, g_stat) + 1
			g_left -= 1
		j += 1
	e["growth"] = growth
	e["proficiency"] = {"sword": {"level": 1, "scaling": Proficiency.DEFAULT_SCALING}}
	e["skill_path"] = {}
	return e

# Errors that disable Save: [{"field", "message"}] (validation plus the system key-field lock). Empty = Save allowed.
static func draft_problems(entry: Dictionary, stored_id: String = "") -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var messages: Array[String] = ContentCatalog.validate_entry("classes", entry, stored_id)
	var old: Dictionary = ContentCatalog.get_entry("classes", stored_id if not stored_id.is_empty() else str(entry.get("id", "")))
	if not stored_id.is_empty() or not old.is_empty():
		messages.append_array(ContentCatalog.key_field_errors("classes", old, ContentSchema.normalize("classes", entry)))
	for message: String in messages:
		out.append({"field": field_of_message(message, entry), "message": message})
	return out

# {"blocked", "problems": [{field, message}], "reasons": [text], "warnings": [{section, text}]}. Warnings never block.
static func save_state(entry: Dictionary, stored_id: String = "") -> Dictionary:
	var problems: Array[Dictionary] = draft_problems(entry, stored_id)
	var reasons: Array[String] = []
	for p: Dictionary in problems:
		reasons.append(str(p["message"]))
	return {"blocked": not problems.is_empty(), "problems": problems, "reasons": reasons, "warnings": ClassUnitValidator.class_warnings(ContentSchema.normalize("classes", entry))}

# The class field an error message belongs to (outline red); "" when it is about the whole entry.
static func field_of_message(message: String, entry: Dictionary = {}) -> String:
	if message.begins_with("Skill path") or message.begins_with("The skill path"):
		return "skill_path"
	if message.begins_with("Base "):
		return "base_stats"
	if message.begins_with("Growth"):
		return "growth"
	if message.begins_with("Proficiency") or message.contains("gear proficiency"):
		return "proficiency"
	if message.begins_with("Art "):
		return "art"
	if message.begins_with("ID ") or message.begins_with("ID:"):
		return "id"
	if message.begins_with("Name"):
		return "name"
	if message.begins_with("Description"):
		return "description"
	if message.contains("accessory"):
		return "accessory_2"
	if message.contains("two-handed"):
		return "off_hand"
	if message.contains("key field"):
		for f: Dictionary in ContentSchema.fields("classes"):
			if message.contains("'%s'" % str(f.get("label", ""))):
				return str(f["key"])
	for slot: String in ContentSchema.LOADOUT_SLOTS + ["pouch"]:
		var label: String = str(ContentSchema.field("classes", slot).get("label", slot))
		if message.contains(label) or message.contains(slot.capitalize()):
			return slot
	if message.contains("no training") or message.contains("can't be equipped"):
		for slot: String in ContentSchema.LOADOUT_SLOTS:
			var item_name: String = str(ContentCatalog.get_entry("items", str(entry.get(slot, ""))).get("name", ""))
			if not item_name.is_empty() and (message.contains("(%s)" % item_name) or message.begins_with(item_name)):
				return slot
	return ""

# ------------------------------------------------------------------------------
# Skill path (section 3.5)
# ------------------------------------------------------------------------------

# The path as ordered steps (level ascending, then entry position): the same order as ClassArchetype.path_steps().
static func ordered_path(path: Dictionary) -> Array[Dictionary]:
	return ClassArchetype.ordered_steps(path)

# Level selector entries for `family`: the standard levels (advanced and higher grades only >= ADVANCED_MIN_LEVEL), then
# CUSTOM_LEVEL ("Custom level..." reveals a spinner).
static func level_options(family: String) -> Array[int]:
	var floor_level: int = advanced_floor(family)
	var out: Array[int] = []
	for lvl: int in ClassArchetype.STANDARD_PATH_LEVELS:
		if lvl >= floor_level:
			out.append(lvl)
	out.append(CUSTOM_LEVEL)
	return out

# Lowest valid path level of `family` (1, or ADVANCED_MIN_LEVEL for advanced / expert grades).
static func advanced_floor(family: String) -> int:
	var head: Dictionary = ContentCatalog.get_entry("skills", family)
	return StatRules.ADVANCED_MIN_LEVEL if str(head.get("grade", "novice")) != "novice" else 1

static func is_custom_level(level: int) -> bool:
	return not ClassArchetype.STANDARD_PATH_LEVELS.has(level)

# One row per path entry, in ENTRY order (the table edits rows in place): {"family", "name", "level", "custom", "kind",
# "grade", "requires", "requires_ok", "ladder", "active_by_default"}.
static func path_rows(entry: Dictionary) -> Array[Dictionary]:
	var path: Dictionary = DictUtil.dict_of(entry.get("skill_path", {}))
	var active: Array[String] = default_loadout(entry, StatRules.MAX_LEVEL)
	var out: Array[Dictionary] = []
	for key: Variant in path:
		var fam: String = str(key)
		var level: int = DictUtil.to_int(path[key], 1)
		var head: Dictionary = ContentCatalog.get_entry("skills", fam)
		var requires: Array[String] = ContentFactory._strings(head.get("requires", []))
		var requires_ok: bool = true
		for r: String in requires:
			requires_ok = requires_ok and path.has(r) and DictUtil.to_int(path[r]) <= level
		out.append({"family": fam, "name": str(head.get("name", fam)), "level": level, "custom": is_custom_level(level),
			"kind": str(head.get("kind", "")), "grade": str(head.get("grade", "novice")), "requires": requires, "requires_ok": requires_ok,
			"ladder": ladder(fam, level), "active_by_default": active.has(fam)})
	return out

# "N families, first 5 active by default".
static func path_counter(entry: Dictionary) -> String:
	var n: int = DictUtil.dict_of(entry.get("skill_path", {})).size()
	return "%d %s, first %d active by default" % [n, "family" if n == 1 else "families", StatRules.MAX_CLASS_SLOTS]

# Tier ladder of one entry: "T1 Lv L / T2 Lv L+off2 / T3 Lv L+off3", capped at MAX_LEVEL (marked "capped").
static func ladder(family: String, level: int) -> String:
	var parts: Array[String] = []
	for row: SkillDef in SkillCatalog.family_tiers(family):
		var opens: int = level + row.tier_offset()
		parts.append("T%d Lv %d%s" % [row.tier, mini(opens, StatRules.MAX_LEVEL), " (capped)" if opens > StatRules.MAX_LEVEL else ""])
	return " / ".join(parts) if not parts.is_empty() else "T1 Lv %d" % level

# The family picker: every family head that may sit on a path (tier 1, not engine, not basic), with {"id", "name", "kind",
# "grade", "requires", "used" (already on the path: greyed), "reason", "requires_missing" (requires not on the path)}.
static func family_choices(entry: Dictionary, current: String = "") -> Array[Dictionary]:
	var path: Dictionary = DictUtil.dict_of(entry.get("skill_path", {}))
	var out: Array[Dictionary] = []
	for row: Dictionary in ContentCatalog.entries("skills"):
		var id: String = str(row.get("id", ""))
		if DictUtil.to_int(row.get("tier", 1), 1) != 1 or not str(row.get("family", "")).is_empty():
			continue
		if DictUtil.to_bool(row.get("system", false)) or DictUtil.to_bool(row.get("basic", false)):
			continue
		var requires: Array[String] = ContentFactory._strings(row.get("requires", []))
		var missing: Array[String] = []
		for r: String in requires:
			if not path.has(r):
				missing.append(r)
		var used: bool = path.has(id) and id != current
		out.append({"id": id, "name": str(row.get("name", id)), "kind": str(row.get("kind", "")), "grade": str(row.get("grade", "novice")),
			"requires": requires, "requires_missing": missing, "used": used, "reason": "Already on the path." if used else ""})
	return out

# Copy of `entry` with `family` at `level`: an existing entry keeps its position (edited in place), a new one goes last.
static func set_path_level(entry: Dictionary, family: String, level: int) -> Dictionary:
	var out: Dictionary = entry.duplicate(true)
	var path: Dictionary = DictUtil.dict_of(out.get("skill_path", {})).duplicate()
	path[family] = level
	out["skill_path"] = path
	return out

static func add_path_family(entry: Dictionary, family: String, level: int = -1) -> Dictionary:
	if DictUtil.dict_of(entry.get("skill_path", {})).has(family):
		return entry.duplicate(true)
	return set_path_level(entry, family, level if level > 0 else maxi(1, advanced_floor(family)))

# Removal keeps the order of the others.
static func remove_path_family(entry: Dictionary, family: String) -> Dictionary:
	var out: Dictionary = entry.duplicate(true)
	var path: Dictionary = DictUtil.dict_of(out.get("skill_path", {})).duplicate()
	path.erase(family)
	out["skill_path"] = path
	return out

# The row's family replaced by `new_family` at the same position and level (the picker of an existing row).
static func replace_path_family(entry: Dictionary, old_family: String, new_family: String) -> Dictionary:
	var out: Dictionary = entry.duplicate(true)
	var path: Dictionary = DictUtil.dict_of(out.get("skill_path", {}))
	if not path.has(old_family) or (path.has(new_family) and new_family != old_family):
		return out
	out["skill_path"] = ContentCatalog._renamed_key(path, old_family, new_family)
	return out

# Read-only preview of the default active class families at `level` (the unit with an empty `loadout`): owned path
# families sorted by (unlock level, entry position), first MAX_CLASS_SLOTS (= ContentFactory.loadout_of).
static func default_loadout(entry: Dictionary, level: int = StatRules.MAX_LEVEL) -> Array[String]:
	var arch: ClassArchetype = ContentFactory.class_archetype(entry)
	var slots: Dictionary = ContentFactory.loadout_of({}, arch, level)
	return ContentFactory._strings(slots["class"])

# ------------------------------------------------------------------------------
# Panels (sections 3.1, 3.2, 3.4)
# ------------------------------------------------------------------------------

# Stat panel: {"base_total", "budget", "growth_total", "growth_budget", "rows": [{stat, base, growth, lv5, lv10, bad}],
# "vitals": [{level, hp, speed}], "note"}.
static func stat_panel(entry: Dictionary) -> Dictionary:
	var arch: ClassArchetype = ContentFactory.class_archetype(entry)
	var rows: Array[Dictionary] = []
	var base_total: int = 0
	for s: String in StatRules.STATS:
		var base: int = DictUtil.int_of(arch.base_stats, s, StatRules.STAT_FLOOR)
		var growth: int = DictUtil.int_of(arch.growth_per_level, s)
		base_total += base
		var bad: String = ""
		if base < StatRules.STAT_FLOOR or base > StatRules.STAT_CEILING:
			bad = "base outside %d-%d" % [StatRules.STAT_FLOOR, StatRules.STAT_CEILING]
		elif growth < 0 or growth > StatRules.GROWTH_STAT_MAX:
			bad = "growth outside 0-%d" % StatRules.GROWTH_STAT_MAX
		rows.append({"stat": s, "base": base, "growth": growth, "lv5": base + growth * 4, "lv10": base + growth * (StatRules.MAX_LEVEL - 1), "bad": bad})
	var vitals: Array[Dictionary] = []
	for lvl: int in [1, 5, StatRules.MAX_LEVEL]:
		var st := CharacterStats.new(arch, lvl)
		vitals.append({"level": lvl, "hp": st.get_max_hp({}), "speed": st.get_initiative_speed(0, {})})
	return {"base_total": base_total, "budget": StatRules.CLASS_BUDGET, "growth_total": StatRules.total_of(arch.growth_per_level), "growth_budget": StatRules.GROWTH_TOTAL,
		"rows": rows, "vitals": vitals, "note": "Units add %d + (level - 1) free points (StatRules.SPEND_BASE)." % StatRules.SPEND_BASE}

# Gear budget panel (soft): {"rows": GearBudget.loadout_breakdown, "total", "target", "low", "high", "in_band", "specials": [names],
# "excused": bool (over the band with a documented special), "unarmed_value"}.
static func budget_panel(entry: Dictionary) -> Dictionary:
	var rows: Array[Dictionary] = GearBudget.loadout_breakdown(entry)
	var total: float = GearBudget.loadout_value(entry)
	var specials: Array[String] = []
	for row: Dictionary in rows:
		if DictUtil.to_bool(row["special"]):
			specials.append("%s (%s)" % [str(row["name"]), str(row["note"]) if not str(row["note"]).is_empty() else "no note"])
	var in_band: bool = GearBudget.in_band(total)
	return {"rows": rows, "total": total, "target": GearBudget.TARGET_BUDGET, "low": GearBudget.band_low(), "high": GearBudget.band_high(),
		"in_band": in_band, "specials": specials, "excused": not in_band and total > GearBudget.band_high() and GearBudget.has_documented_special(entry),
		"unarmed_value": GearBudget.UNARMED_VALUE}

# Proficiency coverage rows (3.4): [{"severity": error|warning|info, "text"}]. An empty main hand counts as using `fist`
# (bare hands are the fist weapon; DECISIONS content-forge "FORGE1 items/classes trim answers" 3).
static func coverage(entry: Dictionary) -> Array[Dictionary]:
	var e: Dictionary = ContentSchema.normalize("classes", entry)
	var out: Array[Dictionary] = []
	var prof: Proficiency = Proficiency.from_dict(DictUtil.dict_of(e["proficiency"]))
	var used: Array[String] = []
	var base: Dictionary = ContentFactory._ints(e["base_stats"])
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		var item: Dictionary = ContentCatalog.get_entry("items", str(e[slot]))
		if item.is_empty():
			if slot == "main_hand":
				used.append(SkillDef.UNARMED_PROFICIENCY)
			continue
		var gear: String = str(item.get("weapon_type", ""))
		if not gear.is_empty():
			used.append(gear)
			if not prof.can_use(gear):
				out.append({"severity": "error", "text": "%s (%s): no training in '%s'." % [str(item.get("name", "")), slot, gear]})
		var req: Dictionary = ContentFactory._ints(DictUtil.dict_of(item.get("required_stats", {})))
		for stat: Variant in req:
			var have: int = DictUtil.int_of(base, str(stat), StatRules.STAT_FLOOR)
			if ["STR", "DEX", "INT"].has(str(stat)) and DictUtil.to_int(req[stat]) > have:
				out.append({"severity": "info", "text": "%s needs %s %d; the class has %d at level 1." % [str(item.get("name", "")), str(stat), DictUtil.to_int(req[stat]), have]})
	for key: Variant in DictUtil.dict_of(e["proficiency"]):
		if not used.has(str(key)):
			out.append({"severity": "info", "text": "Trained in '%s' but no starter item uses it." % str(key)})
	for w: Dictionary in ClassUnitValidator.class_warnings(e):
		if str(w["section"]) == "proficiency":
			out.append({"severity": "warning", "text": str(w["text"])})
	var armor: bool = false
	for a: String in Proficiency.ARMOR_TYPES:
		armor = armor or prof.can_use(a)
	if not armor:
		out.append({"severity": "info", "text": "No armor type trained: the class can only wear untyped gear."})
	return out

# Remove of a proficiency row: refused for the last weapon type or a type the default gear uses ({"allowed", "reason"}).
static func can_remove_proficiency(entry: Dictionary, gear: String) -> Dictionary:
	var prof: Dictionary = DictUtil.dict_of(entry.get("proficiency", {}))
	if Proficiency.WEAPON_TYPES.has(gear):
		var weapons: int = 0
		for key: Variant in prof:
			if Proficiency.WEAPON_TYPES.has(str(key)):
				weapons += 1
		if weapons <= 1:
			return {"allowed": false, "reason": "A class needs at least one weapon type."}
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		var item: Dictionary = ContentCatalog.get_entry("items", str(entry.get(slot, "")))
		if str(item.get("weapon_type", "")) == gear:
			return {"allowed": false, "reason": "The default gear uses it (%s)." % str(item.get("name", ""))}
	return {"allowed": true, "reason": ""}

# Gear types the "add proficiency" dropdown offers (not yet listed, in Proficiency.gear_types() order).
static func proficiency_type_choices(entry: Dictionary) -> Array[String]:
	var prof: Dictionary = DictUtil.dict_of(entry.get("proficiency", {}))
	var out: Array[String] = []
	for gear: String in Proficiency.gear_types():
		if not prof.has(gear):
			out.append(gear)
	return out

# ------------------------------------------------------------------------------
# Gear pickers (sections 2.5, 3.6): grouped by item family, strict hand typing, duplicate-accessory flag
# ------------------------------------------------------------------------------

# Items a gear slot of `entry` may offer, grouped by family (family order, then item order): [{"id", "label", "group",
# "enabled", "reason"}]. Only items that fit the slot are listed (Equipment.can_equip_in_slot honours
# Equipment.allow_plain_dual_wield); untrained items and, while StatRules.allow_duplicate_accessory is off, the item in the
# other accessory slot are disabled with the reason. `kind` "units" also offers ContentSchema.NO_ITEM.
static func slot_choices(entry: Dictionary, slot: String, kind: String = "classes") -> Array[Dictionary]:
	var prof: Proficiency = Proficiency.from_dict(DictUtil.dict_of(entry.get("proficiency", {})))
	var check_training: bool = kind == "classes"
	if kind == "units":
		var arch: ClassArchetype = ContentFactory.class_of(entry)
		check_training = arch != null
		if arch != null:
			prof = arch.make_proficiency()
	var other: String = ""
	if slot == "accessory_1":
		other = str(entry.get("accessory_2", ""))
	elif slot == "accessory_2":
		other = str(entry.get("accessory_1", ""))
	var groups: Array[String] = []
	var by_group: Dictionary = {}
	for item: Dictionary in ContentCatalog.entries("items"):
		if not ContentSchema.EQUIP_TYPES.has(str(item.get("item_type", ""))) or not ContentFactory.item_fits_slot(item, slot.to_upper()):
			continue
		var group: String = item_group(item)
		if not by_group.has(group):
			by_group[group] = []
			groups.append(group)
		var id: String = str(item.get("id", ""))
		var gear: String = str(item.get("weapon_type", ""))
		var reason: String = ""
		if check_training and not gear.is_empty() and not prof.can_use(gear):
			reason = "No training in %s." % gear
		elif ClassUnitValidator.is_duplicate_accessory(id, other):
			reason = "Already in the other accessory slot (the same accessory cannot fill both)."
		var list: Array = by_group[group]
		list.append({"id": id, "label": str(item.get("name", id)), "group": group, "enabled": reason.is_empty(), "reason": reason})
	var out: Array[Dictionary] = []
	if kind == "units":
		out.append({"id": ContentSchema.NO_ITEM, "label": "(no item)", "group": "", "enabled": true, "reason": ""})
	for group: String in groups:
		for raw: Variant in DictUtil.array_of(by_group[group]):
			out.append(DictUtil.dict_of(raw))
	return out

# The picker group of an item: its family name (ITEM1), else its gear type or item type (legacy flat items).
static func item_group(item: Dictionary) -> String:
	var family: Dictionary = ContentCatalog.family_of(item)
	if not family.is_empty():
		return str(family.get("name", family.get("id", "")))
	var gear: String = str(item.get("weapon_type", ""))
	return gear.capitalize() if not gear.is_empty() else str(item.get("item_type", "")).capitalize()

# ------------------------------------------------------------------------------
# Unit AI group (section 4)
# ------------------------------------------------------------------------------

# The unit's effective AI role: its own, else its class's ("" when it has neither).
static func effective_role(unit: Dictionary) -> String:
	var own: String = str(unit.get("ai_role", ""))
	if not own.is_empty():
		return own
	var arch: ClassArchetype = ContentFactory.class_of(unit)
	return arch.ai_role if arch != null else ""

# Unit form visibility (UI only, never `when`, so storage is unchanged): flee / guard thresholds hide while reckless,
# heal_below shows only for an effective SUPPORT role.
static func unit_field_visible(unit: Dictionary, key: String) -> bool:
	match key:
		"flee_ratio", "defend_below":
			return not DictUtil.to_bool(unit.get("reckless", false))
		"heal_below":
			return effective_role(unit) == "SUPPORT"
	var f: Dictionary = ContentSchema.field("units", key)
	return f.is_empty() or ContentSchema.is_visible(f, unit)
