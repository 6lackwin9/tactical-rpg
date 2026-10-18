---
class: SkillFormModel
extends: RefCounted
description: "Tactical RPG - Skill Form Model (FORGE1 sections 3, 5, 6 and appendix F-I) The headless brain of the skill tab of the Content Forge. Pure static functions over a RANKED skill entry (the draft the form edits: a family template plus up to two rank modifiers): which sections and fields a kind shows, each field's state (visible, enabled, dynamic limits, allowed options and the reason when something is disabled), kind switching, the ranks table (modifier cells, resolved rows), the live description preview, and why Save is blocked. The controls (ForgeForm, SkillRanksPanel) only render what this returns, so every rule is tested without a window."
---

# Properties
* `SECTION_TITLES`: Dictionary = { (const)
* `CORE_SECTIONS`: Array[Dictionary] = [ (const)
* `KIND_DEFAULTS`: Dictionary = { (const)
* `kind`: String = str(entry.get("kind", "ATTACK"))
* `out`: Array[Dictionary] = []
* `keys`: Array[String] = []
* `keys`: Array[String] = []
* `out`: Array[String] = []
* `out`: Array[String] = [""]
* `out`: Array[String] = []
* `own`: String = str(entry.get("id", head))
* `grade`: int = ContentCatalog.grade_rank(str(entry.get("grade", "novice")))
* `out`: Array[String] = []
* `list`: Array = state_info.get("problems", [])
* `p`: Dictionary = DictUtil.dict_of(raw)
* `out`: Array[Dictionary] = []
* `row`: Dictionary = SkillRanks.resolve(entry, rank)
* `text`: String = str(row["description"])
* `kind`: String = str(entry.get("kind", "ATTACK"))
* `out`: Array[String] = SkillText.tokens_for(kind)

# Methods

### `static _shown_in_core(key: String) -> bool`
```logic
for core: Dictionary in CORE_SECTIONS:
	var core_fields: Array = core["fields"]
	if core_fields.has(key):
		return true
return false
```

### `static is_visible(entry: Dictionary, key: String) -> bool`
```logic
if ["family", "tier", "template", "unlock_offset"].has(key):
	return false
var kind: String = str(entry.get("kind", "ATTACK"))
if not SkillKinds.has_field(kind, key):
	return false
var f: Dictionary = ContentSchema.field("skills", key)
if f.is_empty():
	return false
if key == "system" and not DictUtil.to_bool(entry.get("system", false)):
	return false
return ContentSchema.is_visible(f, entry)
```

### `static state(entry: Dictionary, key: String, head: String = "") -> Dictionary`
```logic
var out: Dictionary = {"visible": is_visible(entry, key), "enabled": true, "reason": "", "min": null, "max": null, "options": [], "disabled_options": {}, "choices": null}
var f: Dictionary = ContentSchema.field("skills", key)
if f.is_empty():
	return out
var kind: String = str(entry.get("kind", "ATTACK"))
if DictUtil.to_bool(f.get("readonly", false)):
	out["enabled"] = false
var disabled: Dictionary = out["disabled_options"]
match key:
	"basic":
		out["enabled"] = false
		out["reason"] = "The Forge cannot create or change basic skills (every unit owns them from level 1)."
	"grade":
		disabled["expert"] = "No expert skills yet: the value is reserved."
		if DictUtil.to_bool(entry.get("basic", false)) or DictUtil.to_bool(entry.get("system", false)):
			out["enabled"] = false
			out["reason"] = "Basic and engine skills stay novice."
	"requires":
		if str(entry.get("grade", "novice")) == "novice":
			out["enabled"] = false
			out["reason"] = "A novice skill requires nothing: raise the grade first."
		out["choices"] = requires_choices(entry, head)
	"power_mult":
		if str(entry.get("status_effect", "")).is_empty():
			out["min"] = 0.05
			out["reason"] = "A skill with no damage needs a status: pick Inflicts first."
	"weapon_hand":
		if not SkillDef.BOTH_ENABLED:
			disabled["both"] = "Engine pending: the dual strike is not playable yet."
		if DictUtil.to_int(entry.get("dash_tiles", 0)) > 0:
			disabled["off"] = "A gap-close uses the main hand: set the gap-close to 0 first."
			disabled["both"] = "A gap-close uses the main hand: set the gap-close to 0 first."
	"area":
		if DictUtil.to_float(entry.get("power_mult", 1.0)) <= 0.0:
			for option: String in ["PLUS", "BURST", "LINE", "SWEEP"]:
				disabled[option] = "A skill with no damage can't be an area attack."
		elif DictUtil.to_int(entry.get("dash_tiles", 0)) > 0:
			for option: String in ["PLUS", "BURST", "LINE", "SWEEP"]:
				disabled[option] = "An area attack can't be a gap-close."
	"dash_tiles":
		if str(entry.get("area", "NONE")) != "NONE" or str(entry.get("weapon_hand", "main")) != "main":
			out["max"] = 0
			out["reason"] = "No gap-close with an area attack or an off-hand / dual strike."
	"reach_min":
		out["max"] = DictUtil.to_int(entry.get("reach_max", 0))
		if DictUtil.to_int(entry.get("reach_max", 0)) == 0:
			out["reason"] = "Set reach max first (0 = the unit's default band)."
	"reach_max":
		if str(entry.get("area", "NONE")) in ["LINE", "SWEEP"]:
			out["max"] = 1
			out["reason"] = "LINE and SWEEP start from an adjacent tile."
	"status_potency":
		out["min"] = 1
	"status_magnitude":
		var magnitude_status: String = SkillText.magnitude_status(entry)
		if StatusEffects.has_magnitude(magnitude_status):
			out["min"] = StatusRegistry.magnitude_min(magnitude_status)
			out["max"] = StatusRegistry.magnitude_max(magnitude_status)
		else:
			out["max"] = 0
			out["reason"] = "Only %s carry a magnitude." % ", ".join(StatusRegistry.magnitude_ids())
	"min_proficiency":
		if not ContentFactory._strings(entry.get("weapon_types", [])).is_empty():
			out["min"] = 1
			out["reason"] = "A listed weapon type needs a proficiency of at least 1."
	"target_mode":
		if str(entry.get("grant_status", "")) == StatusEffects.TAUNT:
			disabled["ALLY"] = "Taunt is cast on the taunter itself."
			disabled["ALLY_OR_SELF"] = "Taunt is cast on the taunter itself."
	"heal_stat":
		if DictUtil.to_float(entry.get("heal_mult", 0.0)) <= 0.0:
			out["options"] = [""]
			out["reason"] = "Needs a heal per stat point above 0."
		else:
			out["options"] = ["INT", "KNOW"]
	"grant_status":
		out["options"] = grant_options(kind)
	"cleanse_statuses":
		out["options"] = ContentSchema.options_of(f)
return out
```

### `static new_entry(id: String) -> Dictionary`
```logic
var d: Dictionary = ContentSchema.defaults("skills")
for derived: String in ["family", "tier", "template", "system"]:
	d.erase(derived)
d["id"] = id
d["origin"] = ContentSchema.ORIGIN_USER
return switch_kind(d, str(d["kind"]), true)
```

### `static switch_kind(entry: Dictionary, new_kind: String, force_defaults: bool = false) -> Dictionary`
```logic
var out: Dictionary = entry.duplicate(true)
var changed: bool = str(entry.get("kind", "")) != new_kind or force_defaults
out["kind"] = new_kind
for f: Dictionary in ContentSchema.fields("skills"):
	var key: String = str(f["key"])
	if not SkillKinds.has_field(new_kind, key) and out.has(key) and not ["family", "tier", "template"].has(key):
		out[key] = ContentSchema.defaults("skills")[key]
if not grant_options(new_kind).has(str(out.get("grant_status", ""))):
	out["grant_status"] = ""
var allowed: Array[String] = SkillRanks.rank_fields(new_kind)
var kept: Array = []
for raw: Variant in DictUtil.array_of(out.get("ranks", [])):
	var mod: Dictionary = DictUtil.dict_of(raw)
	var cleaned: Dictionary = {}
	for mod_key: Variant in mod:
		if allowed.has(str(mod_key)) or str(mod_key) == "description":
			cleaned[mod_key] = mod[mod_key]
	kept.append(cleaned)
if not kept.is_empty():
	out["ranks"] = kept
if changed and KIND_DEFAULTS.has(new_kind):
	var defaults: Dictionary = KIND_DEFAULTS[new_kind]
	for key: Variant in defaults:
		var f: Dictionary = ContentSchema.field("skills", str(key))
		if out.get(str(key), null) == f["default"]:
			out[key] = defaults[key]
return out
```

### `static save_state(entry: Dictionary, head: String = "") -> Dictionary`
```logic
var problems: Array[Dictionary] = ContentCatalog.validate_skill_entry(entry, head)
var reasons: Array[String] = []
for p: Dictionary in problems:
	var rank: int = DictUtil.to_int(p["rank"])
	reasons.append(("[rank %d] " % rank if rank > 0 and not str(p["message"]).begins_with("Rank") else "") + str(p["message"]))
return {"blocked": not problems.is_empty(), "problems": problems, "reasons": reasons, "warnings": ContentCatalog.warnings_for("skills", entry)}
```

### `static ranks_editable(entry: Dictionary, _head: String = "") -> Dictionary`
```logic
if DictUtil.to_bool(entry.get("basic", false)) or DictUtil.to_bool(entry.get("system", false)):
	return {"allowed": false, "reason": "single rank"}
return {"allowed": true, "reason": ""}
```

### `static ranks_view(entry: Dictionary, head: String = "") -> Dictionary`
```logic
var kind: String = str(entry.get("kind", "ATTACK"))
var fields: Array[String] = SkillRanks.rank_fields(kind)
fields.erase("unlock_offset")   # rank 1 is always 0; the offset is shown as its own row from rank 2
var rows: Array[Dictionary] = []
var count: int = SkillRanks.rank_count(entry)
var mods: Array[Dictionary] = SkillRanks.modifiers_of(entry)
for rank: int in range(1, count + 1):
	var resolved: Dictionary = SkillRanks.resolve(entry, rank)
	var values: Dictionary = {}
	for key: String in fields:
		values[key] = resolved[key]
	values["unlock_offset"] = resolved["unlock_offset"]
	var modified: Array[String] = []
	if rank >= 2:
		for key: Variant in mods[rank - 2]:
			if SkillRanks.RANK_KEYS.has(str(key)):
				modified.append(str(key))
	rows.append({"rank": rank, "name": str(resolved["name"]), "values": values, "modified": modified, "ap": DictUtil.to_int(resolved["level"]), "text": str(resolved["description"])})
var editable: Dictionary = ranks_editable(entry, head)
return {"fields": fields, "rows": rows, "can_add": DictUtil.to_bool(editable["allowed"]) and count < StatRules.MAX_CLASS_TIER,
	"can_remove": DictUtil.to_bool(editable["allowed"]) and count > 1, "reason": str(editable["reason"])}
```

### `static set_modifier(entry: Dictionary, rank: int, key: String, value: Variant) -> Dictionary`
```logic
var out: Dictionary = entry.duplicate(true)
var mods: Array = DictUtil.array_of(out.get("ranks", []))
if rank < 2 or rank - 2 >= mods.size() or not SkillRanks.RANK_KEYS.has(key):
	return out
var mod: Dictionary = DictUtil.dict_of(mods[rank - 2])
var base: Dictionary = SkillRanks.effective(entry, 1)
if value == base.get(key, null):
	mod.erase(key)
else:
	mod[key] = value
mods[rank - 2] = mod
out["ranks"] = mods
return out
```

### `static reset_modifier(entry: Dictionary, rank: int, key: String) -> Dictionary`
```logic
var out: Dictionary = entry.duplicate(true)
var mods: Array = DictUtil.array_of(out.get("ranks", []))
if rank >= 2 and rank - 2 < mods.size():
	var mod: Dictionary = DictUtil.dict_of(mods[rank - 2])
	mod.erase(key)
	mods[rank - 2] = mod
	out["ranks"] = mods
return out
```

### `static copy_previous(entry: Dictionary, rank: int) -> Dictionary`
```logic
var out: Dictionary = entry.duplicate(true)
var mods: Array = DictUtil.array_of(out.get("ranks", []))
if rank >= 3 and rank - 2 < mods.size():
	var previous: Dictionary = DictUtil.dict_of(mods[rank - 3])
	var copy: Dictionary = {}
	for key: Variant in previous:
		if SkillRanks.RANK_KEYS.has(str(key)):
			copy[key] = previous[key]
	mods[rank - 2] = copy
	out["ranks"] = mods
return out
```

### `static add_rank(entry: Dictionary) -> Dictionary`
```logic
var out: Dictionary = entry.duplicate(true)
var mods: Array = DictUtil.array_of(out.get("ranks", []))
if mods.size() + 1 < StatRules.MAX_CLASS_TIER:
	mods.append({})
out["ranks"] = mods
return out
```

### `static remove_top_rank(entry: Dictionary) -> Dictionary`
```logic
var out: Dictionary = entry.duplicate(true)
var mods: Array = DictUtil.array_of(out.get("ranks", []))
if not mods.is_empty():
	mods.pop_back()
if mods.is_empty():
	out.erase("ranks")
else:
	out["ranks"] = mods
return out
```

### `static insert_token(template: String, caret: int, token: String) -> Dictionary`
```logic
var at: int = clampi(caret, 0, template.length())
var piece: String = "{%s}" % token
return {"text": template.substr(0, at) + piece + template.substr(at), "caret": at + piece.length()}
```

