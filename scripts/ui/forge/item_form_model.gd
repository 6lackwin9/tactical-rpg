class_name ItemFormModel
extends RefCounted

# ==============================================================================
# Tactical RPG - Item Form Model (FORGE1_ITEMS 6, matrix 6; ITEM1-L commit 3)
# The headless brain of the item and item-family tabs of the Content Forge: pure static functions over a draft.
# An item draft is a RESOLVED row (what ContentCatalog.get_entry returns); `sync` re-resolves it after every edit so the
# derived keys (item type, numbers, threshold tiers) follow the stored ones (family, hand, modifiers, tiers), and Apply
# stores `stored_of(draft)` (family + modifiers, ItemModifiers.stored). Layout (eight item sections, seven family
# sections), control state (locked key fields of system assets, options from the family, dynamic limits), the modifier
# and tier tables (rows, defaults, add / remove / move, limits from ItemModifiers.KINDS and Passives.effect_specs), the
# base-values grid, the budget breakdown, the family impact panel, the problems that disable Apply, the warnings that
# never do, the delete state and the item list filters. The controls (ItemTables, ForgeForm) only render what this returns;
# ContentForge wires it (state_fn, layout_fn, problems_fn, panel_fn) for the "items" and "item_families" tabs.
# ==============================================================================

const ITEM_SECTIONS: Array[String] = ["identity", "build", "base", "modifiers", "tiers", "special", "consumable", "preview"]
const FAMILY_SECTIONS: Array[String] = ["identity", "type", "numbers", "policy", "rules", "impact", "used_by"]
const SECTION_TITLES: Dictionary = {
	"items": {"identity": "1 Identity", "build": "2 Family and build", "base": "3 Base values", "modifiers": "4 Modifiers",
		"tiers": "5 Tier thresholds", "special": "6 Special", "consumable": "7 Consumable payload", "preview": "8 Preview"},
	"item_families": {"identity": "Identity", "type": "Type and hands", "numbers": "Template numbers and bands",
		"policy": "Modifier policy", "rules": "Engine rules", "impact": "Impact of the edits", "used_by": "Used by"},
}
# Stored item fields per section (derived keys never show as controls; `slot` only for armor and accessories).
const ITEM_FIELD_SECTIONS: Dictionary = {
	"id": "identity", "name": "identity", "description": "identity", "icon_id": "identity", "rarity": "identity",
	"value": "identity", "weight": "identity", "starter_stash": "identity", "origin": "identity",
	"family": "build", "hand": "build", "scaling_stat": "build", "slot": "build", "damage_type": "build",
	"modifiers": "modifiers", "tiers": "tiers", "special": "special", "special_note": "special",
	"consumable_type": "consumable", "heal": "consumable", "damage": "consumable", "buff_stat": "consumable",
	"buff_pct": "consumable", "duration": "consumable", "use_blocks": "consumable", "max_stack": "consumable",
}
const FAMILY_FIELD_SECTIONS: Dictionary = {
	"id": "identity", "name": "identity", "description": "identity", "art": "identity", "origin": "identity",
	"item_type": "type", "hands": "type", "default_hand": "type", "off_hand_category": "type", "slots": "type",
	"scaling_stats": "type", "damage_type": "type",
	"base": "numbers", "bands": "numbers",
	"adjustable": "policy", "modifier_kinds": "policy", "max_modifiers": "policy", "max_tiers": "policy",
	"engine_rules": "rules",
}
# Read-only panels a section shows: before its fields ("pre") or after them ("post").
const ITEM_PANELS: Dictionary = {"identity": {"post": ["tokens"]}, "build": {"pre": ["chips"]}, "base": {"post": ["base"]}, "preview": {"post": ["preview"]}}
const FAMILY_PANELS: Dictionary = {"impact": {"post": ["impact"]}, "used_by": {"post": ["used_by"]}}

const LOCK_REASON: String = "System asset: key field locked. Create your own copy to change it."
const FAMILY_USED_REASON: String = "%d item(s) use this family: the %s is fixed while they do."
const SAFE_DELETE_REASON: String = "System asset: delete is locked. Use SAFE DELETE (tools/fc.sh safe_delete) or make your own copy."
const USAGES: Array[String] = ["class", "unit", "named", "unused"]
const TRI: Array[String] = ["any", "yes", "no"]

# ------------------------------------------------------------------------------
# Drafts: resolve, sync, store
# ------------------------------------------------------------------------------

static func family_row(draft: Dictionary) -> Dictionary:
	return ContentCatalog.get_entry("item_families", str(draft.get("family", "")))

static func item_type(draft: Dictionary) -> String:
	var family: Dictionary = family_row(draft)
	return ItemFamilies.item_type_of(family) if not family.is_empty() else str(draft.get("item_type", "WEAPON"))

# The resolved, normalised row of an item draft (the draft itself, normalised, while its family is unknown).
static func resolved(draft: Dictionary) -> Dictionary:
	var family: Dictionary = family_row(draft)
	if family.is_empty():
		return ContentSchema.normalize("items", draft)
	return ContentSchema.normalize("items", ItemModifiers.resolve(ItemModifiers.stored(draft), family))

# What Apply hands to ContentCatalog.upsert: the stored shape (family + modifiers), so the list order and edits are kept.
static func stored_of(draft: Dictionary) -> Dictionary:
	if family_row(draft).is_empty():
		return draft.duplicate(true)
	return ItemModifiers.stored(draft)

# Re-resolves an item draft IN PLACE after the edit of `changed_key`. A family switch first moves hand, scaling stat and
# slot to the new family's defaults when the old values are not allowed there (the modifiers stay; validation names them).
static func sync(draft: Dictionary, changed_key: String = "") -> void:
	var family: Dictionary = family_row(draft)
	if family.is_empty():
		return
	if changed_key == "family":
		var hands: Array[String] = ItemFamilies.hands_of(family)
		if hands.is_empty():
			draft["hand"] = ""
		elif not hands.has(str(draft.get("hand", ""))):
			draft["hand"] = ItemFamilies.default_hand(family)
		var scaling: Array[String] = ContentFactory._strings(family.get("scaling_stats", []))
		if not scaling.is_empty() and not scaling.has(str(draft.get("scaling_stat", ""))):
			draft["scaling_stat"] = scaling[0]
		var slots: Array[String] = ContentFactory._strings(family.get("slots", []))
		if ItemFamilies.SLOT_TYPES.has(ItemFamilies.item_type_of(family)) and not slots.is_empty() and not slots.has(str(draft.get("slot", ""))):
			draft["slot"] = slots[0]
	var fresh: Dictionary = resolved(draft)
	for key: Variant in fresh:
		draft[key] = fresh[key]

# A fresh user item that validates: the first WEAPON family (else the first family) with its defaults, no modifiers.
static func new_entry(id: String) -> Dictionary:
	var family_id: String = ""
	for f: Dictionary in ContentCatalog.entries("item_families"):
		if ItemFamilies.item_type_of(f) == "WEAPON":
			family_id = str(f.get("id", ""))
			break
	if family_id.is_empty() and not ContentCatalog.entries("item_families").is_empty():
		family_id = str(ContentCatalog.entries("item_families")[0].get("id", ""))
	var e: Dictionary = ContentSchema.defaults("items")
	e["id"] = id
	e["origin"] = ContentSchema.ORIGIN_USER
	e["family"] = family_id
	e["hand"] = ""
	e["scaling_stat"] = ""
	e["modifiers"] = []
	e["tiers"] = []
	sync(e, "family")
	return e

# ------------------------------------------------------------------------------
# Layout and control state
# ------------------------------------------------------------------------------

# [{"id", "title", "fields": [keys], "pre": [panel ids], "post": [panel ids]}] for `kind` "items" or "item_families".
static func layout(kind: String, draft: Dictionary) -> Array[Dictionary]:
	var families: bool = kind == "item_families"
	var order: Array[String] = FAMILY_SECTIONS if families else ITEM_SECTIONS
	var sections: Dictionary = FAMILY_FIELD_SECTIONS if families else ITEM_FIELD_SECTIONS
	var panels: Dictionary = FAMILY_PANELS if families else ITEM_PANELS
	var titles: Dictionary = SECTION_TITLES[kind]
	var by_section: Dictionary = {}
	var visible_row: Dictionary = ItemFamilies.derive(ContentSchema.normalize(kind, draft)) if families else draft
	for raw_key: Variant in sections:   # the matrix order (identity: id, name, description, icon, rarity, value, weight, ...)
		var key: String = str(raw_key)
		if not field_visible(kind, draft, key, visible_row):
			continue
		var section: String = str(sections[key])
		var keys: Array = by_section.get(section, [])
		keys.append(key)
		by_section[section] = keys
	var out: Array[Dictionary] = []
	for section: String in order:
		var p: Dictionary = DictUtil.dict_of(panels.get(section, {}))
		var keys: Array = by_section.get(section, [])
		if keys.is_empty() and p.is_empty():
			continue
		if not families and section == "base" and ItemFamilies.number_fields_for(item_type(draft)).is_empty():
			continue
		out.append({"id": section, "title": str(titles.get(section, section.capitalize())), "fields": keys,
			"pre": DictUtil.array_of(p.get("pre", [])), "post": DictUtil.array_of(p.get("post", []))})
	return out

# Whether a schema field shows a control: the schema `when` holds (on the resolved type) and the field is stored.
static func field_visible(kind: String, draft: Dictionary, key: String, row: Dictionary = {}) -> bool:
	var f: Dictionary = ContentSchema.field(kind, key)
	if f.is_empty():
		return false
	var check: Dictionary = row if not row.is_empty() else draft
	if kind == "items":
		check = check.duplicate()
		check["item_type"] = item_type(draft)
		if key == "slot":
			return ItemFamilies.SLOT_TYPES.has(item_type(draft))
		if key == "tiers":
			return ContentSchema.is_visible(f, check) and (max_tiers(draft) > 0 or not ItemModifiers.tiers_of(draft).is_empty())
		if key == "modifiers":
			return ContentSchema.is_visible(f, check) and ItemFamilies.MODIFIABLE_TYPES.has(item_type(draft))
	return ContentSchema.is_visible(f, check)

# True when the stored `kind` asset `id` is a system asset whose key fields are locked (never inside the system-edit scope).
static func is_locked(kind: String, id: String) -> bool:
	if id.is_empty() or ContentCatalog.in_system_edit() or not ContentCatalog.has(kind, id):
		return false
	return ContentCatalog.is_system_origin(ContentCatalog.get_entry(kind, id))

static func _blank_state() -> Dictionary:
	return {"enabled": true, "locked": false, "reason": "", "min": null, "max": null, "step": null, "options": [], "disabled_options": {}}

# State of one control: {"enabled", "locked", "reason", "min", "max", "step", "options", "disabled_options": {value: reason},
# "option_help": {value: text}, "choices" (grouped ref records)}. `stored_id` = the stored entry being edited ("" = new draft).
static func state(kind: String, draft: Dictionary, key: String, stored_id: String = "") -> Dictionary:
	var out: Dictionary = _blank_state()
	var f: Dictionary = ContentSchema.field(kind, key)
	if f.is_empty():
		return out
	if DictUtil.to_bool(f.get("readonly", false)):
		out["enabled"] = false
	if DictUtil.to_bool(f.get("key_field", false)) and is_locked(kind, stored_id):
		out["enabled"] = false
		out["locked"] = true
		out["reason"] = LOCK_REASON
	if kind == "item_families":
		_family_state(out, draft, key, stored_id)
	else:
		_item_state(out, draft, key, stored_id)
	return out

static func _item_state(out: Dictionary, draft: Dictionary, key: String, stored_id: String) -> void:
	var family: Dictionary = family_row(draft)
	match key:
		"family":
			out["choices"] = family_choices()
		"hand":
			out["options"] = ItemFamilies.hands_of(family)
		"scaling_stat":
			out["options"] = ContentFactory._strings(family.get("scaling_stats", []))
		"slot":
			out["options"] = ContentFactory._strings(family.get("slots", []))
		"icon_id":
			out["options"] = icon_options()
		"damage_type":
			if not DictUtil.to_bool(out["locked"]):
				out["reason"] = "Empty = the family default (%s)." % (str(family.get("damage_type", "")) if not str(family.get("damage_type", "")).is_empty() else "none")
		"modifiers":
			out["max"] = DictUtil.to_int(family.get("max_modifiers", 0))
			out["add"] = can_add_modifier(draft, stored_id)
		"tiers":
			out["max"] = max_tiers(draft)
			out["add"] = can_add_tier(draft, stored_id)

static func _family_state(out: Dictionary, draft: Dictionary, key: String, stored_id: String) -> void:
	var f: Dictionary = ItemFamilies.derive(ContentSchema.normalize("item_families", draft))
	var type: String = ItemFamilies.item_type_of(f)
	var users: int = ContentCatalog.items_of_family(stored_id).size() if not stored_id.is_empty() else 0
	var disabled: Dictionary = out["disabled_options"]
	match key:
		"id", "item_type":
			if users > 0 and not DictUtil.to_bool(out["locked"]):
				out["enabled"] = false
				out["reason"] = FAMILY_USED_REASON % [users, "id" if key == "id" else "item type"]
		"hands":
			var hands: Array[String] = ItemFamilies.hands_of(f)
			for h: String in ItemFamilies.HANDS:
				if type == "SHIELD" and h != ItemFamilies.HAND_OFF:
					disabled[h] = "Shields are off-hand only."
				elif hands.has(ItemFamilies.HAND_OFF) and h != ItemFamilies.HAND_OFF:
					disabled[h] = "One hand category per family: uncheck off first (off-hand families are support items)."
				elif h == ItemFamilies.HAND_OFF and hands.any(func(x: String) -> bool: return ItemFamilies.MAIN_HANDS.has(x)):
					disabled[h] = "One hand category per family: uncheck the main hands first."
		"default_hand":
			out["options"] = ItemFamilies.hands_of(f)
		"slots":
			out["options"] = ItemFamilies.ARMOR_SLOTS if type == "ARMOR" else ItemFamilies.ACCESSORY_SLOTS
		"adjustable":
			out["options"] = ItemFamilies.number_fields_for(type)
		"modifier_kinds":
			if not ItemFamilies.MODIFIABLE_TYPES.has(type):
				out["enabled"] = false
				out["reason"] = "%s families take no modifiers." % type
		"engine_rules":
			out["option_help"] = ItemFamilies.RULE_HELP.duplicate()
			for rule: String in ItemFamilies.ENGINE_RULES:
				var why: String = rule_blocker(f, rule)
				if not why.is_empty():
					disabled[rule] = why
		"max_modifiers":
			out["max"] = 12
		"max_tiers":
			out["max"] = 4

# Why `rule` cannot be set on family `f` ("" = it can): the consistency rules of FORGE1_ITEMS 1.3.
static func rule_blocker(f: Dictionary, rule: String) -> String:
	var type: String = ItemFamilies.item_type_of(f)
	match rule:
		ItemFamilies.RULE_FOCUS:
			if type != "WEAPON" or ItemFamilies.category(f) != "off":
				return "focus needs a WEAPON family with hands = off."
		ItemFamilies.RULE_SHIELD:
			if type != "SHIELD":
				return "shield needs a SHIELD family."
		ItemFamilies.RULE_ALWAYS_RANGED, ItemFamilies.RULE_SPELL_SOURCE:
			if type != "WEAPON":
				return "%s needs a WEAPON family." % rule
	return ""

# Family picker records grouped by item type: [{"id", "label", "group", "enabled", "reason"}].
static func family_choices() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for type: String in ItemFamilies.ITEM_TYPES:
		for f: Dictionary in ContentCatalog.entries("item_families"):
			if ItemFamilies.item_type_of(f) != type:
				continue
			var id: String = str(f.get("id", ""))
			out.append({"id": id, "label": "%s  [%s]" % [str(f.get("name", id)), id], "group": type.capitalize(), "enabled": true, "reason": ""})
	return out

# Icon ids an item may name: "" (icon by gear type), the bespoke icons and the kept type icons (ItemArt).
static func icon_options() -> Array[String]:
	var out: Array[String] = [""]
	for key: Variant in ItemArt.UNIQUE_ICONS:
		out.append(str(key))
	for key: String in ItemArt.TYPE_ICON_IDS:
		if not out.has(key):
			out.append(key)
	return out

static func max_tiers(draft: Dictionary) -> int:
	return DictUtil.to_int(family_row(draft).get("max_tiers", 0))

# ------------------------------------------------------------------------------
# Problems (disable Apply) and warnings (never do)
# ------------------------------------------------------------------------------

# [{"field", "index", "message"}]: validation (rows of the modifier / tier tables carry their 1-based index) plus the
# system key-field lock. Empty = Apply allowed.
static func draft_problems(kind: String, draft: Dictionary, stored_id: String = "") -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var messages: Array[String] = []
	var candidate: Dictionary = {}
	if kind == "items":
		var row: Dictionary = resolved(draft)
		candidate = row
		messages = ContentCatalog.validate_entry("items", stored_of(draft), stored_id)
		var family: Dictionary = family_row(draft)
		if not family.is_empty():
			out.append_array(ItemValidator.item_problems(row, family, ContentCatalog.items_inherit()))
	else:
		candidate = ItemFamilies.derive(ContentSchema.normalize("item_families", draft))
		messages = ContentCatalog.validate_entry("item_families", draft, stored_id)
		out.append_array(ItemValidator.family_problems(candidate, stored_id))
	var known: Dictionary = {}
	for p: Dictionary in out:
		known[str(p["message"])] = true
	for message: String in messages:
		if not known.has(message):
			out.append({"field": field_of_message(kind, message), "index": 0, "message": message})
			known[message] = true
	var old: Dictionary = ContentCatalog.get_entry(kind, stored_id if not stored_id.is_empty() else str(draft.get("id", "")))
	if not old.is_empty():
		for message: String in ContentCatalog.key_field_errors(kind, old, candidate):
			out.append({"field": field_of_message(kind, message), "index": 0, "message": message})
	return out

# {"blocked", "problems", "reasons": [text], "warnings": [text]}. Warnings (bands, budget, literal numbers, special note) never block.
static func save_state(kind: String, draft: Dictionary, stored_id: String = "") -> Dictionary:
	var problems: Array[Dictionary] = draft_problems(kind, draft, stored_id)
	var reasons: Array[String] = []
	for p: Dictionary in problems:
		reasons.append(str(p["message"]))
	return {"blocked": not problems.is_empty(), "problems": problems, "reasons": reasons, "warnings": warnings(kind, draft)}

static func warnings(kind: String, draft: Dictionary) -> Array[String]:
	if kind == "item_families":
		return ItemValidator.family_warnings(ItemFamilies.derive(ContentSchema.normalize("item_families", draft)))
	var row: Dictionary = resolved(draft)
	var out: Array[String] = GearBudget.check_item(row)
	out.append_array(ItemValidator.item_warnings(row, family_row(draft)))
	return out

# "Cannot save: N problems" (the problems bar title; "" when there are none).
static func blocked_title(problems: Array[Dictionary]) -> String:
	if problems.is_empty():
		return ""
	return "Cannot save: %d problem%s" % [problems.size(), "" if problems.size() == 1 else "s"]

# The field an error message belongs to (outline red); "" when it is about the whole entry.
static func field_of_message(kind: String, message: String) -> String:
	if message.contains("key field"):
		for f: Dictionary in ContentSchema.fields(kind):
			if message.contains("'%s'" % str(f.get("label", ""))):
				return str(f["key"])
	if message.begins_with("ID ") or message.begins_with("ID:"):
		return "id"
	if message.begins_with("Name"):
		return "name"
	if message.begins_with("Description") or message.begins_with("Token") or message.begins_with("The rendered description"):
		return "description"
	if message.begins_with("Family"):
		return "family"
	for f: Dictionary in ContentSchema.fields(kind):
		if message.begins_with(str(f.get("label", "")) + ":"):
			return str(f["key"])
	return ""

# Problems of one table row (`field` modifiers / tiers, 1-based `n`), as texts.
static func row_problems(problems: Array[Dictionary], field: String, n: int) -> Array[String]:
	var out: Array[String] = []
	for p: Dictionary in problems:
		if str(p["field"]) == field and DictUtil.to_int(p.get("index", 0)) == n:
			out.append(str(p["message"]))
	return out

# ------------------------------------------------------------------------------
# Build chips, base values, budget, description tokens, preview
# ------------------------------------------------------------------------------

# Read-only chips of the build section: [{"label", "value"}].
static func chips(draft: Dictionary) -> Array[Dictionary]:
	var row: Dictionary = resolved(draft)
	var out: Array[Dictionary] = [
		{"label": "Item type", "value": str(row.get("item_type", ""))},
		{"label": "Gear token", "value": str(row.get("weapon_type", "")) if not str(row.get("weapon_type", "")).is_empty() else "-"},
		{"label": "Slot", "value": str(row.get("slot", "")) if not str(row.get("slot", "")).is_empty() else "-"},
	]
	if not str(row.get("hand", "")).is_empty():
		out.append({"label": "Hand", "value": ItemText.hand_label(row)})
	if not str(row.get("off_hand_category", "")).is_empty():
		out.append({"label": "Off-hand category", "value": str(row["off_hand_category"])})
	return out

# The base-values grid: one row per number of the type, "family base + adjustments = result" with the band marker.
# [{"field", "label", "base", "delta", "result", "min", "max", "band_min", "band_max", "in_range", "in_band", "text"}].
static func base_rows(draft: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var family: Dictionary = family_row(draft)
	if family.is_empty():
		return out
	var hand: String = ItemModifiers.hand_in(draft, family)
	var base: Dictionary = ItemFamilies.base_of(family, hand)
	var raw: Dictionary = ItemModifiers.raw_numbers(draft, family)
	for field: String in ItemFamilies.number_fields_for(ItemFamilies.item_type_of(family)):
		var rng: Array = ItemFamilies.NUMBER_RANGES[field]
		var b: int = DictUtil.to_int(base[field])
		var result: int = DictUtil.to_int(raw[field])
		var band: Dictionary = ItemFamilies.band_of(family, hand, field)
		var in_range: bool = result >= DictUtil.to_int(rng[0]) and result <= DictUtil.to_int(rng[1])
		var in_band: bool = band.is_empty() or (result >= DictUtil.to_int(band.get("min", result)) and result <= DictUtil.to_int(band.get("max", result)))
		var text: String = "%d %s %d = %d" % [b, "-" if result - b < 0 else "+", absi(result - b), result]
		if not band.is_empty():
			text += "  (band %d-%d%s)" % [DictUtil.to_int(band["min"]), DictUtil.to_int(band["max"]), "" if in_band else ", outside"]
		if not in_range:
			text += "  outside %d-%d" % [DictUtil.to_int(rng[0]), DictUtil.to_int(rng[1])]
		out.append({"field": field, "label": field.capitalize(), "base": b, "delta": result - b, "result": result,
			"min": DictUtil.to_int(rng[0]), "max": DictUtil.to_int(rng[1]),
			"band_min": band.get("min", null), "band_max": band.get("max", null), "in_range": in_range, "in_band": in_band, "text": text})
	return out

# Budget breakdown: the family base value, then one row per modifier ({"label", "value"}); "total" equals
# GearBudget.item_value of the resolved row (FORGE1_ITEMS 2.3 acceptance).
static func budget_breakdown(draft: Dictionary) -> Dictionary:
	var rows: Array[Dictionary] = []
	var family: Dictionary = family_row(draft)
	var total: float = 0.0
	if not family.is_empty() and GearBudget.is_gear({"item_type": ItemFamilies.item_type_of(family)}):
		var hand: String = ItemModifiers.hand_in(draft, family)
		var base: float = GearBudget.base_value(family, hand)
		rows.append({"label": "Family base (%s%s)" % [str(family.get("id", "")), (" " + hand) if not hand.is_empty() else ""], "value": base})
		total += base
		for mod: Dictionary in ItemModifiers.modifiers_of(draft):
			var v: float = GearBudget.modifier_value(mod)
			rows.append({"label": modifier_text(mod), "value": v})
			total += v
	var row: Dictionary = resolved(draft)
	return {"rows": rows, "total": total, "item_value": GearBudget.item_value(row), "tier_potential": GearBudget.tier_potential(row)}

# The tokens the description palette offers for this item: only tokens that have a value now ({req:STR} only with a
# STR requirement, {tier2.power} only with two tiers), in ItemText.TOKEN_TYPES order.
static func token_palette(draft: Dictionary) -> Array[String]:
	var row: Dictionary = resolved(draft)
	var type: String = str(row.get("item_type", "WEAPON"))
	var out: Array[String] = []
	for token: String in ItemText.all_tokens():
		if not ItemText.token_valid_for(token, type):
			continue
		if ItemText.STAT_TOKENS.has(token):
			var stats: Dictionary = DictUtil.dict_of(row.get("required_stats" if token == "req" else "stat_bonuses", {}))
			for stat: Variant in stats:
				out.append("%s:%s" % [token, str(stat)])
		elif token.begins_with("tier."):
			var tiers: Array[Dictionary] = ItemModifiers.tiers_of(row)
			for i: int in range(tiers.size()):
				var candidate: String = "tier%d.%s" % [i + 1, token.get_slice(".", 1)]
				if not ItemText.token_value(candidate, "", row).is_empty():
					out.append(candidate)
		elif not ItemText.token_value(token, "", row).is_empty():
			out.append(token)
	return out

# {"text": rendered description, "problems": [text], "warnings": [text]} for the token help under the description.
static func description_preview(draft: Dictionary) -> Dictionary:
	var row: Dictionary = resolved(draft)
	var template: String = str(draft.get("description", ""))
	var problems: Array[String] = []
	for p: Dictionary in ItemText.validate(template, row):
		problems.append(str(p["message"]))
	return {"text": ItemText.render(template, row), "problems": problems, "warnings": ItemText.warnings(template, row)}

# The preview section: {"summary", "hand", "tiers": [text], "budget" (budget_breakdown), "users": [kind:id], "named": [qid]}.
static func preview(draft: Dictionary) -> Dictionary:
	var row: Dictionary = resolved(draft)
	var tiers: Array[String] = []
	for t: Dictionary in tier_rows(draft, []):
		tiers.append("%s: %s" % [str(t["criterion_text"]), str(t["desc"]) if not str(t["desc"]).is_empty() else str(t["auto_desc"])])
	var id: String = str(draft.get("id", ""))
	return {"summary": ItemText.modifier_summary(row), "hand": ItemText.hand_label(row) if not str(row.get("hand", "")).is_empty() else "",
		"tiers": tiers, "budget": budget_breakdown(draft), "users": ContentCatalog.references_to("items", id), "named": named_users(id)}

# World named items whose base is `item_id`, as world:id.
static func named_users(item_id: String) -> Array[String]:
	var out: Array[String] = []
	if item_id.is_empty():
		return out
	for world_id: String in WorldCatalog.world_ids():
		for named: Dictionary in WorldCatalog.own_entries(world_id, "items"):
			if str(named.get("base_item", "")) == item_id:
				out.append(WorldCatalog.qid(world_id, str(named.get("id", ""))))
	return out

# ------------------------------------------------------------------------------
# Modifier table (FORGE1_ITEMS 2.3; ItemModifiers.KINDS, Passives.effect_specs)
# ------------------------------------------------------------------------------

# "power +4", "STR bonus +2", "Req STR 14", "effect skill_potency group spell +0.1".
static func modifier_text(mod: Dictionary) -> String:
	match str(mod.get("kind", "")):
		ItemModifiers.ADJUST:
			return "%s %+d" % [str(mod.get("field", "")), DictUtil.to_int(mod.get("delta", 0))]
		ItemModifiers.STAT_BONUS:
			return "%s bonus +%d" % [str(mod.get("stat", "")), DictUtil.to_int(mod.get("value", 0))]
		ItemModifiers.REQUIREMENT:
			return "Req %s %d" % [str(mod.get("stat", "")), DictUtil.to_int(mod.get("value", 0))]
		ItemModifiers.EFFECT:
			return "effect " + effect_text(DictUtil.dict_of(mod.get("effect", {})))
	return str(mod.get("kind", "?"))

static func effect_text(fx: Dictionary) -> String:
	var parts: Array[String] = [str(fx.get("type", "?"))]
	for key: Variant in fx:
		if str(key) != "type":
			parts.append("%s %s" % [str(key), str(fx[key])])
	return " ".join(parts)

# Kinds the kind picker offers: every closed kind; kinds the family does not allow are disabled with the reason.
static func modifier_kind_state(draft: Dictionary) -> Dictionary:
	var family: Dictionary = family_row(draft)
	var allowed: Array[String] = ContentFactory._strings(family.get("modifier_kinds", []))
	var disabled: Dictionary = {}
	for kind: String in ItemModifiers.kind_names():
		if not allowed.has(kind):
			disabled[kind] = "Family '%s' does not allow %s modifiers." % [str(family.get("id", "")), kind]
	return {"options": ItemModifiers.kind_names(), "disabled": disabled}

# {"allowed", "reason"} of "+ Add modifier": locked list, a type without modifiers, the family cap or no allowed kind.
static func can_add_modifier(draft: Dictionary, stored_id: String = "") -> Dictionary:
	var family: Dictionary = family_row(draft)
	if is_locked("items", stored_id):
		return {"allowed": false, "reason": LOCK_REASON}
	if family.is_empty():
		return {"allowed": false, "reason": "Pick a family first."}
	var type: String = ItemFamilies.item_type_of(family)
	if not ItemFamilies.MODIFIABLE_TYPES.has(type):
		return {"allowed": false, "reason": "%s items take no modifiers." % type}
	var cap: int = DictUtil.to_int(family.get("max_modifiers", 0))
	var count: int = ItemModifiers.modifiers_of(draft).size()
	if count >= cap:
		return {"allowed": false, "reason": "Full: %d of %d modifiers (family '%s' max)." % [count, cap, str(family.get("id", ""))]}
	if ContentFactory._strings(family.get("modifier_kinds", [])).is_empty():
		return {"allowed": false, "reason": "Family '%s' allows no modifier kinds." % str(family.get("id", ""))}
	return {"allowed": true, "reason": ""}

# A valid new modifier of `kind` for this draft: an unused adjustable number (+1, or -1 at the top of its range), an
# unused stat at the kind minimum, or the first item effect with its defaults.
static func default_modifier(draft: Dictionary, kind: String) -> Dictionary:
	var spec: Dictionary = DictUtil.dict_of(ItemModifiers.KINDS.get(kind, {}))
	match kind:
		ItemModifiers.ADJUST:
			var options: Dictionary = adjust_field_state(draft, -1)
			var field: String = ""
			for candidate: String in ContentFactory._strings(options["options"]):
				if not DictUtil.dict_of(options["disabled"]).has(candidate):
					field = candidate
					break
			if field.is_empty():
				field = ItemFamilies.NUMBER_FIELDS[0]
			var mod: Dictionary = {"kind": kind, "field": field, "delta": 1}
			var limits: Dictionary = delta_limits(draft, -1, field)
			if DictUtil.to_int(limits["max"]) < 1:
				mod["delta"] = -1
			return mod
		ItemModifiers.STAT_BONUS, ItemModifiers.REQUIREMENT:
			var used: Dictionary = stat_state(draft, -1, kind)
			var stat: String = ContentSchema.STAT_KEYS[0]
			for candidate: String in ContentSchema.STAT_KEYS:
				if not DictUtil.dict_of(used["disabled"]).has(candidate):
					stat = candidate
					break
			return {"kind": kind, "stat": stat, "value": DictUtil.to_int(spec.get("min", 1))}
		ItemModifiers.EFFECT:
			var specs: Array[Dictionary] = Passives.effect_specs(true)
			return {"kind": kind, "effect": effect_default(str(specs[0]["type"]) if not specs.is_empty() else "")}
	return {"kind": kind}

# Numbers an adjust row may name: the type's numbers; not adjustable by the family, or adjusted by another row = disabled.
# `index` = the row (0-based; -1 for a new row).
static func adjust_field_state(draft: Dictionary, index: int) -> Dictionary:
	var family: Dictionary = family_row(draft)
	var adjustable: Array[String] = ContentFactory._strings(family.get("adjustable", []))
	var disabled: Dictionary = {}
	var mods: Array[Dictionary] = ItemModifiers.modifiers_of(draft)
	for field: String in ItemFamilies.number_fields_for(ItemFamilies.item_type_of(family)):
		if not adjustable.has(field):
			disabled[field] = "Family '%s' does not allow adjusting %s." % [str(family.get("id", "")), field]
			continue
		for i: int in range(mods.size()):
			if i != index and str(mods[i].get("kind", "")) == ItemModifiers.ADJUST and str(mods[i].get("field", "")) == field:
				disabled[field] = "Already adjusted in row %d." % (i + 1)
	return {"options": ItemFamilies.number_fields_for(ItemFamilies.item_type_of(family)), "disabled": disabled}

# Dynamic delta limits of an adjust row: the resolved value stays inside the schema range (and the kind range -100..100).
static func delta_limits(draft: Dictionary, index: int, field: String = "") -> Dictionary:
	var family: Dictionary = family_row(draft)
	var spec: Dictionary = ItemModifiers.KINDS[ItemModifiers.ADJUST]
	var mods: Array[Dictionary] = ItemModifiers.modifiers_of(draft)
	var number: String = field
	if number.is_empty() and index >= 0 and index < mods.size():
		number = str(mods[index].get("field", ""))
	if not ItemFamilies.NUMBER_RANGES.has(number):
		return {"min": DictUtil.to_int(spec["min"]), "max": DictUtil.to_int(spec["max"])}
	var rng: Array = ItemFamilies.NUMBER_RANGES[number]
	var base: int = DictUtil.to_int(ItemFamilies.base_of(family, ItemModifiers.hand_in(draft, family))[number])
	return {"min": maxi(DictUtil.to_int(spec["min"]), DictUtil.to_int(rng[0]) - base), "max": mini(DictUtil.to_int(spec["max"]), DictUtil.to_int(rng[1]) - base)}

# Stats a stat_bonus / requirement row may name; a stat another row of the same kind already uses is disabled.
static func stat_state(draft: Dictionary, index: int, kind: String) -> Dictionary:
	var disabled: Dictionary = {}
	var mods: Array[Dictionary] = ItemModifiers.modifiers_of(draft)
	for i: int in range(mods.size()):
		if i != index and str(mods[i].get("kind", "")) == kind:
			disabled[str(mods[i].get("stat", ""))] = "Already in row %d (one %s per stat)." % [i + 1, kind]
	var spec: Dictionary = ItemModifiers.KINDS[kind]
	return {"options": ContentSchema.STAT_KEYS.duplicate(), "disabled": disabled, "min": DictUtil.to_int(spec["min"]), "max": DictUtil.to_int(spec["max"])}

# One row per modifier: {"index" (0-based), "n", "kind", "text", "value" (GearBudget), "result" (adjust: the resolved
# number), "problems": [text]}.
static func modifier_rows(draft: Dictionary, problems: Array[Dictionary] = []) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var family: Dictionary = family_row(draft)
	var raw: Dictionary = ItemModifiers.raw_numbers(draft, family) if not family.is_empty() else {}
	var mods: Array[Dictionary] = ItemModifiers.modifiers_of(draft)
	for i: int in range(mods.size()):
		var mod: Dictionary = mods[i]
		var result: Variant = null
		if str(mod.get("kind", "")) == ItemModifiers.ADJUST and raw.has(str(mod.get("field", ""))):
			result = raw[str(mod.get("field", ""))]
		out.append({"index": i, "n": i + 1, "kind": str(mod.get("kind", "")), "text": modifier_text(mod), "value": GearBudget.modifier_value(mod),
			"result": result, "problems": row_problems(problems, "modifiers", i + 1)})
	return out

static func _modifier_list(draft: Dictionary) -> Array:
	return DictUtil.array_of(ContentSchema._copy(DictUtil.array_of(draft.get("modifiers", []))))

# New `modifiers` lists (the form puts them into the draft).
static func with_modifier_added(draft: Dictionary, kind: String) -> Array:
	var list: Array = _modifier_list(draft)
	list.append(default_modifier(draft, kind))
	return list

static func with_modifier_set(draft: Dictionary, index: int, mod: Dictionary) -> Array:
	var list: Array = _modifier_list(draft)
	if index >= 0 and index < list.size():
		list[index] = mod.duplicate(true)
	return list

# The row switched to `kind` (a fresh default of that kind).
static func with_modifier_kind(draft: Dictionary, index: int, kind: String) -> Array:
	var without: Dictionary = draft.duplicate()
	var list: Array = _modifier_list(draft)
	if index < 0 or index >= list.size():
		return list
	list.remove_at(index)
	without["modifiers"] = list
	var fresh: Dictionary = default_modifier(without, kind)
	list.insert(index, fresh)
	return list

static func with_modifier_removed(draft: Dictionary, index: int) -> Array:
	var list: Array = _modifier_list(draft)
	if index >= 0 and index < list.size():
		list.remove_at(index)
	return list

static func with_modifier_moved(draft: Dictionary, index: int, step: int) -> Array:
	var list: Array = _modifier_list(draft)
	var to: int = index + step
	if index < 0 or index >= list.size() or to < 0 or to >= list.size():
		return list
	var mod: Variant = list[index]
	list.remove_at(index)
	list.insert(to, mod)
	return list

# ------------------------------------------------------------------------------
# Effects (the passive grammar, item types only; Passives.effect_specs)
# ------------------------------------------------------------------------------

static func effect_types() -> Array[String]:
	var out: Array[String] = []
	for spec: Dictionary in Passives.effect_specs(true):
		out.append(str(spec["type"]))
	return out

# A valid effect of `type`: enums take their first option, numbers their smallest allowed value (non-zero kept), and of
# exclusive `choice` params the enum one is set (a group needs no catalog id).
static func effect_default(type: String) -> Dictionary:
	var spec: Dictionary = Passives.effect_spec(type)
	var fx: Dictionary = {"type": type}
	var chosen: Dictionary = {}
	for raw: Variant in DictUtil.array_of(spec.get("params", [])):
		var p: Dictionary = DictUtil.dict_of(raw)
		var choice: String = str(p.get("choice", ""))
		if not choice.is_empty():
			if chosen.has(choice) or str(p["kind"]) != "enum":
				continue
			chosen[choice] = true
		if DictUtil.to_bool(p.get("optional", false)):
			continue
		fx[str(p["key"])] = param_default(p)
	for raw: Variant in DictUtil.array_of(spec.get("params", [])):   # a choice group without an enum param: its first param
		var p: Dictionary = DictUtil.dict_of(raw)
		var choice: String = str(p.get("choice", ""))
		if not choice.is_empty() and not chosen.has(choice):
			chosen[choice] = true
			fx[str(p["key"])] = param_default(p)
	return fx

static func param_default(p: Dictionary) -> Variant:
	match str(p.get("kind", "")):
		"enum":
			var options: Array = DictUtil.array_of(p.get("options", []))
			return str(options[0]) if not options.is_empty() else ""
		"int":
			var lo: int = DictUtil.to_int(p.get("min", 0))
			return lo if lo != 0 or not DictUtil.to_bool(p.get("nonzero", false)) else 1
		"float":
			var lo_f: float = DictUtil.to_float(p.get("min", 0.0))
			var step: float = DictUtil.to_float(p.get("step", 0.05))
			if lo_f > 0.0:
				return lo_f
			return step if DictUtil.to_bool(p.get("nonzero", false)) else 0.0
		"skill":
			var families: Array[String] = skill_options()
			return families[0] if not families.is_empty() else ""
	return ""

# Skill families an effect param of kind "skill" may name.
static func skill_options() -> Array[String]:
	var out: Array[String] = []
	for fam: String in SkillCatalog.families():
		if not SkillCatalog.is_system(fam):
			out.append(fam)
	return out

# One row per param of the effect's type: {"key", "kind", "options", "min", "max", "step", "value", "choice", "active"}.
# Of an exclusive choice group exactly one param is active (the one the effect sets).
static func effect_param_rows(fx: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var spec: Dictionary = Passives.effect_spec(str(fx.get("type", "")))
	for raw: Variant in DictUtil.array_of(spec.get("params", [])):
		var p: Dictionary = DictUtil.dict_of(raw)
		var key: String = str(p["key"])
		if str(p.get("kind", "")) == "context":
			continue
		var options: Array = DictUtil.array_of(p.get("options", []))
		if str(p.get("kind", "")) == "skill":
			options = skill_options()
		out.append({"key": key, "kind": str(p.get("kind", "")), "options": options, "min": p.get("min", null), "max": p.get("max", null),
			"step": p.get("step", null), "value": fx.get(key, null), "choice": str(p.get("choice", "")), "active": fx.has(key)})
	return out

# The effect with param `key` set to `value`; setting a choice param clears its siblings.
static func effect_with_param(fx: Dictionary, key: String, value: Variant) -> Dictionary:
	var out: Dictionary = fx.duplicate(true)
	var spec: Dictionary = Passives.effect_spec(str(fx.get("type", "")))
	var choice: String = ""
	for raw: Variant in DictUtil.array_of(spec.get("params", [])):
		var p: Dictionary = DictUtil.dict_of(raw)
		if str(p["key"]) == key:
			choice = str(p.get("choice", ""))
	if not choice.is_empty():
		for raw: Variant in DictUtil.array_of(spec.get("params", [])):
			var p: Dictionary = DictUtil.dict_of(raw)
			if str(p.get("choice", "")) == choice and str(p["key"]) != key:
				out.erase(str(p["key"]))
	out[key] = value
	return out

# The effect switched to the other param of a choice group (that param gets its default).
static func effect_with_choice(fx: Dictionary, key: String) -> Dictionary:
	var spec: Dictionary = Passives.effect_spec(str(fx.get("type", "")))
	for raw: Variant in DictUtil.array_of(spec.get("params", [])):
		var p: Dictionary = DictUtil.dict_of(raw)
		if str(p["key"]) == key:
			return effect_with_param(fx, key, param_default(p))
	return fx.duplicate(true)

# ------------------------------------------------------------------------------
# Tier table (FORGE1_ITEMS 4)
# ------------------------------------------------------------------------------

# Every limit of the tier table: criterion types and ranges, gear options, bonus keys for the type and their ranges, counts.
static func tier_limits(draft: Dictionary) -> Dictionary:
	var type: String = item_type(draft)
	return {"when_types": ItemModifiers.WHEN_TYPES.duplicate(), "stat_min": ItemModifiers.TIER_STAT_MIN, "stat_max": ItemModifiers.TIER_STAT_MAX,
		"proficiency_min": 1, "proficiency_max": Proficiency.MAX_LEVEL, "level_min": 1, "level_max": StatRules.MAX_LEVEL,
		"gear_options": Proficiency.gear_types(), "gear_default": str(draft.get("family", "")),
		"bonus_keys": ItemModifiers.tier_bonus_keys(type), "bonus_ranges": ItemModifiers.TIER_BONUS_RANGES.duplicate(true),
		"max_stats": ItemModifiers.TIER_MAX_STATS, "max_effects": ItemModifiers.TIER_MAX_EFFECTS,
		"label_max": ItemModifiers.TIER_LABEL_MAX, "desc_max": ItemModifiers.TIER_DESC_MAX, "max_tiers": max_tiers(draft)}

# {"allowed", "reason"} of "+ Add tier".
static func can_add_tier(draft: Dictionary, stored_id: String = "") -> Dictionary:
	if is_locked("items", stored_id):
		return {"allowed": false, "reason": LOCK_REASON}
	var family: Dictionary = family_row(draft)
	if family.is_empty():
		return {"allowed": false, "reason": "Pick a family first."}
	var cap: int = max_tiers(draft)
	if cap <= 0:
		return {"allowed": false, "reason": "Family '%s' allows no tiers (max tiers 0)." % str(family.get("id", ""))}
	var count: int = ItemModifiers.tiers_of(draft).size()
	if count >= cap:
		return {"allowed": false, "reason": "Full: %d of %d tiers (family '%s' max)." % [count, cap, str(family.get("id", ""))]}
	return {"allowed": true, "reason": ""}

# A criterion of `type` for a new or switched tier: the item's first requirement stat (else the scaling stat) one above
# its requirement or the last tier on it, the own gear at level 1, or unit level 1; always inside the ranges.
static func when_default(draft: Dictionary, type: String, skip_index: int = -1) -> Dictionary:
	var row: Dictionary = resolved(draft)
	var tiers: Array[Dictionary] = ItemModifiers.tiers_of(draft)
	match type:
		ItemModifiers.WHEN_PROFICIENCY:
			var gear: String = str(row.get("weapon_type", ""))
			if gear.is_empty():
				gear = Proficiency.gear_types()[0]
			var level: int = 1
			for i: int in range(tiers.size()):
				var w: Dictionary = DictUtil.dict_of(tiers[i].get("when", {}))
				if i != skip_index and str(w.get("type", "")) == type and str(w.get("gear", gear)) == gear:
					level = maxi(level, DictUtil.to_int(w.get("level", 0)) + 1)
			return {"type": type, "gear": gear, "level": mini(level, Proficiency.MAX_LEVEL)}
		ItemModifiers.WHEN_LEVEL:
			var value: int = 1
			for i: int in range(tiers.size()):
				var w: Dictionary = DictUtil.dict_of(tiers[i].get("when", {}))
				if i != skip_index and str(w.get("type", "")) == type:
					value = maxi(value, DictUtil.to_int(w.get("value", 0)) + 1)
			return {"type": type, "value": mini(value, StatRules.MAX_LEVEL)}
	var reqs: Dictionary = DictUtil.dict_of(row.get("required_stats", {}))
	var stat: String = str(row.get("scaling_stat", "")) if str(row.get("item_type", "")) == "WEAPON" else ""
	if not reqs.is_empty() and (stat.is_empty() or not reqs.has(stat)):
		stat = str(reqs.keys()[0])
	if stat.is_empty():
		stat = ContentSchema.STAT_KEYS[0]
	var at: int = DictUtil.int_of(reqs, stat) + 1 if reqs.has(stat) else ItemModifiers.TIER_STAT_MIN
	for i: int in range(tiers.size()):
		var w: Dictionary = DictUtil.dict_of(tiers[i].get("when", {}))
		if i != skip_index and str(w.get("type", "")) == ItemModifiers.WHEN_STAT and str(w.get("stat", "")) == stat:
			at = maxi(at, DictUtil.to_int(w.get("value", 0)) + 1)
	return {"type": ItemModifiers.WHEN_STAT, "stat": stat, "value": clampi(at, ItemModifiers.TIER_STAT_MIN, ItemModifiers.TIER_STAT_MAX)}

# A valid new tier: "Tier N", the default criterion, the smallest bonus of the first bonus key the type allows.
static func default_tier(draft: Dictionary) -> Dictionary:
	var keys: Array[String] = ItemModifiers.tier_bonus_keys(item_type(draft))
	var bonus: Dictionary = {}
	if not keys.is_empty():
		bonus = bonus_default(draft, keys[0])
	return {"label": "Tier %d" % (ItemModifiers.tiers_of(draft).size() + 1), "when": when_default(draft, ItemModifiers.WHEN_STAT), "bonus": bonus, "desc": ""}

# {key: smallest non-zero value} of bonus `key` (stat_bonuses: the scaling / first stat +min; effects: one default effect).
static func bonus_default(draft: Dictionary, key: String) -> Dictionary:
	match key:
		"power", "mitigation":
			var rng: Array = ItemModifiers.TIER_BONUS_RANGES[key]
			return {key: maxi(1, DictUtil.to_int(rng[0]))}
		"stat_bonuses":
			var rng_s: Array = ItemModifiers.TIER_BONUS_RANGES[key]
			var stat: String = str(resolved(draft).get("scaling_stat", ""))
			if stat.is_empty() or not ContentSchema.STAT_KEYS.has(stat):
				stat = ContentSchema.STAT_KEYS[0]
			return {key: {stat: DictUtil.to_int(rng_s[0])}}
		"effects":
			var types: Array[String] = effect_types()
			return {key: [effect_default(types[0])] if not types.is_empty() else []}
	return {}

# One row per tier: {"index", "n", "label", "when", "bonus", "desc", "auto_desc", "criterion_text", "bonus_text",
# "value" (GearBudget preview weight), "problems"}.
static func tier_rows(draft: Dictionary, problems: Array[Dictionary] = []) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var tiers: Array[Dictionary] = ItemModifiers.tiers_of(draft)
	for i: int in range(tiers.size()):
		var t: Dictionary = tiers[i]
		var when: Dictionary = DictUtil.dict_of(t.get("when", {}))
		var bonus: Dictionary = DictUtil.dict_of(t.get("bonus", {}))
		out.append({"index": i, "n": i + 1, "label": str(t.get("label", "")), "when": when, "bonus": bonus, "desc": str(t.get("desc", "")),
			"auto_desc": ItemText.tier_desc(t), "criterion_text": criterion_text(when), "bonus_text": ItemText.bonus_text(bonus),
			"value": GearBudget.tier_bonus_value(bonus) * GearBudget.W_TIER, "problems": row_problems(problems, "tiers", i + 1)})
	return out

static func criterion_text(when: Dictionary) -> String:
	match str(when.get("type", "")):
		ItemModifiers.WHEN_STAT:
			return "%s >= %d" % [str(when.get("stat", "")), DictUtil.to_int(when.get("value", 0))]
		ItemModifiers.WHEN_PROFICIENCY:
			return "%s training >= %d" % [str(when.get("gear", "")), DictUtil.to_int(when.get("level", 0))]
		ItemModifiers.WHEN_LEVEL:
			return "unit level >= %d" % DictUtil.to_int(when.get("value", 0))
	return "?"

static func _tier_list(draft: Dictionary) -> Array:
	return DictUtil.array_of(ContentSchema._copy(DictUtil.array_of(draft.get("tiers", []))))

static func with_tier_added(draft: Dictionary) -> Array:
	var list: Array = _tier_list(draft)
	list.append(default_tier(draft))
	return list

static func with_tier_set(draft: Dictionary, index: int, tier: Dictionary) -> Array:
	var list: Array = _tier_list(draft)
	if index >= 0 and index < list.size():
		list[index] = tier.duplicate(true)
	return list

# The tier's criterion switched to `type` (its default for this item).
static func with_tier_when_type(draft: Dictionary, index: int, type: String) -> Array:
	var list: Array = _tier_list(draft)
	if index < 0 or index >= list.size():
		return list
	var tier: Dictionary = DictUtil.dict_of(list[index]).duplicate(true)
	tier["when"] = when_default(draft, type, index)
	list[index] = tier
	return list

static func with_tier_removed(draft: Dictionary, index: int) -> Array:
	var list: Array = _tier_list(draft)
	if index >= 0 and index < list.size():
		list.remove_at(index)
	return list

static func with_tier_moved(draft: Dictionary, index: int, step: int) -> Array:
	var list: Array = _tier_list(draft)
	var to: int = index + step
	if index < 0 or index >= list.size() or to < 0 or to >= list.size():
		return list
	var tier: Variant = list[index]
	list.remove_at(index)
	list.insert(to, tier)
	return list

# ------------------------------------------------------------------------------
# Families: new, sync, base / band grid, impact, used by
# ------------------------------------------------------------------------------

# A fresh user family that validates. Gear families (WEAPON, SHIELD, ARMOR) are named after a gear token and every token
# already has its family, so a new family takes the first modifiable non-gear type (ACCESSORY) with all its slots.
static func new_family(id: String) -> Dictionary:
	var type: String = "ACCESSORY"
	for t: String in ItemFamilies.ITEM_TYPES:
		if ItemFamilies.MODIFIABLE_TYPES.has(t) and not ItemFamilies.is_gear_type(t):
			type = t
			break
	var f: Dictionary = ContentSchema.defaults("item_families")
	f["id"] = id
	f["origin"] = ContentSchema.ORIGIN_USER
	f["item_type"] = type
	sync_family(f, "item_type")
	return f

# Fixes a family draft IN PLACE after the edit of `changed_key`: the lists follow the type (hands, slots, scaling stats,
# adjustable numbers, modifier kinds), every hand key gets a base row (a copy of the default hand's row, else the
# schema minimum) and rows of removed hands are dropped; then the derived fields (ItemFamilies.derive).
static func sync_family(draft: Dictionary, changed_key: String = "") -> void:
	var type: String = ItemFamilies.item_type_of(draft)
	if changed_key == "item_type":
		var none: Array[String] = []
		if ItemFamilies.HAND_TYPES.has(type):
			var hands: Array[String] = [ItemFamilies.HAND_OFF] if type == "SHIELD" else [ItemFamilies.HAND_MAIN_1H]
			draft["hands"] = hands
		else:
			draft["hands"] = none.duplicate()
		if ItemFamilies.SLOT_TYPES.has(type):
			draft["slots"] = (ItemFamilies.ARMOR_SLOTS if type == "ARMOR" else ItemFamilies.ACCESSORY_SLOTS).duplicate()
		var scaling: Array[String] = [ItemFamilies.SCALING_STATS[0]]
		draft["scaling_stats"] = scaling if type == "WEAPON" else none.duplicate()
		draft["adjustable"] = ItemFamilies.number_fields_for(type)
		draft["modifier_kinds"] = ItemModifiers.kind_names() if ItemFamilies.MODIFIABLE_TYPES.has(type) else none.duplicate()
		var rules: Array[String] = []
		for rule: String in ContentFactory._strings(draft.get("engine_rules", [])):
			if rule_blocker(ItemFamilies.derive(draft), rule).is_empty():
				rules.append(rule)
		draft["engine_rules"] = rules
	var keys: Array[String] = grid_keys(draft)
	var base: Dictionary = DictUtil.dict_of(draft.get("base", {})).duplicate(true)
	var bands: Dictionary = DictUtil.dict_of(draft.get("bands", {})).duplicate(true)
	var template: Dictionary = {}
	for key: Variant in base:
		if keys.has(str(key)):
			template = DictUtil.dict_of(base[key]).duplicate()
			break
	var next_base: Dictionary = {}
	for key: String in keys:
		if base.has(key):
			next_base[key] = base[key]
		elif ItemFamilies.MODIFIABLE_TYPES.has(type):
			next_base[key] = template.duplicate() if not template.is_empty() else _minimum_row(type)
	var next_bands: Dictionary = {}
	for key: Variant in bands:
		if keys.has(str(key)):
			next_bands[key] = bands[key]
	draft["base"] = next_base
	draft["bands"] = next_bands
	var derived: Dictionary = ItemFamilies.derive(draft)
	for key: Variant in derived:
		draft[key] = derived[key]

static func _minimum_row(type: String) -> Dictionary:
	var row: Dictionary = {}
	for field: String in ItemFamilies.number_fields_for(type):
		var rng: Array = ItemFamilies.NUMBER_RANGES[field]
		row[field] = DictUtil.to_int(rng[0])
	return row

# The rows of the base / band grid: the family's hands, or "none" for types without hands.
static func grid_keys(draft: Dictionary) -> Array[String]:
	var hands: Array[String] = ItemFamilies.hands_of(draft)
	if hands.is_empty():
		hands.append(ItemFamilies.NO_HAND_KEY)
	return hands

# The columns of the base grid (the type's numbers) and of the band grid (plus bonus_value).
static func grid_fields(draft: Dictionary, bands: bool = false) -> Array[String]:
	var out: Array[String] = ItemFamilies.number_fields_for(ItemFamilies.item_type_of(draft))
	if bands and ItemFamilies.MODIFIABLE_TYPES.has(ItemFamilies.item_type_of(draft)):
		out.append(ItemFamilies.BONUS_BAND)
	return out

# {"min", "max", "step"} of a grid cell: the schema range of the number (bonus_value: 0..99, step 0.5).
static func grid_range(field: String) -> Dictionary:
	if field == ItemFamilies.BONUS_BAND:
		return {"min": 0.0, "max": 99.0, "step": 0.5}
	var rng: Array = ItemFamilies.NUMBER_RANGES[field]
	return {"min": DictUtil.to_float(rng[0]), "max": DictUtil.to_float(rng[1]), "step": 1.0}

static func base_cell(draft: Dictionary, key: String, field: String) -> int:
	return DictUtil.to_int(ItemFamilies.base_of(draft, key)[field]) if ItemFamilies.hands_of(draft).has(key) or key == ItemFamilies.NO_HAND_KEY else 0

# The `base` dictionary with one cell set.
static func with_base(draft: Dictionary, key: String, field: String, value: int) -> Dictionary:
	var base: Dictionary = DictUtil.dict_of(draft.get("base", {})).duplicate(true)
	var row: Dictionary = DictUtil.dict_of(base.get(key, _minimum_row(ItemFamilies.item_type_of(draft)))).duplicate()
	row[field] = value
	base[key] = row
	return base

# A band cell: {"set": bool, "min", "max"}.
static func band_cell(draft: Dictionary, key: String, field: String) -> Dictionary:
	var band: Dictionary = DictUtil.dict_of(DictUtil.dict_of(DictUtil.dict_of(draft.get("bands", {})).get(key, {})).get(field, {}))
	return {"set": not band.is_empty(), "min": band.get("min", null), "max": band.get("max", null)}

# The `bands` dictionary with one bound set (an unset band starts at the base value / schema range on both bounds).
static func with_band(draft: Dictionary, key: String, field: String, bound: String, value: float) -> Dictionary:
	var bands: Dictionary = DictUtil.dict_of(draft.get("bands", {})).duplicate(true)
	var row: Dictionary = DictUtil.dict_of(bands.get(key, {})).duplicate(true)
	var band: Dictionary = DictUtil.dict_of(row.get(field, {})).duplicate()
	if band.is_empty():
		var start: float = 0.0 if field == ItemFamilies.BONUS_BAND else float(base_cell(draft, key, field))
		band = {"min": start, "max": start}
	var whole: bool = field != ItemFamilies.BONUS_BAND
	band[bound] = roundi(value) if whole else snappedf(value, 0.01)
	row[field] = band
	bands[key] = row
	return bands

static func without_band(draft: Dictionary, key: String, field: String) -> Dictionary:
	var bands: Dictionary = DictUtil.dict_of(draft.get("bands", {})).duplicate(true)
	var row: Dictionary = DictUtil.dict_of(bands.get(key, {})).duplicate(true)
	row.erase(field)
	if row.is_empty():
		bands.erase(key)
	else:
		bands[key] = row
	return bands

# The impact panel of a family edit (FORGE1_ITEMS 3 rail 1): every item of the stored family resolved against the
# draft. {"items": [{"id", "name", "changes": [{"field", "old", "new"}], "value_old", "value_new", "error"}],
# "loadouts": [{"kind", "id", "name", "old", "new"}], "named": [qid], "count", "errors", "text"}.
static func impact(draft: Dictionary, stored_id: String = "") -> Dictionary:
	var family_id: String = stored_id if not stored_id.is_empty() else str(draft.get("id", ""))
	var fam: Dictionary = ItemFamilies.derive(ContentSchema.normalize("item_families", draft))
	var items: Array[Dictionary] = []
	var delta_by_id: Dictionary = {}
	var errors: int = 0
	for item: Dictionary in ContentCatalog.items_of_family(family_id):
		var fresh: Dictionary = ContentSchema.normalize("items", ItemModifiers.resolve(ItemModifiers.stored(item), fam))
		var changes: Array[Dictionary] = []
		for field: String in ItemFamilies.NUMBER_FIELDS:
			var flat: String = str(ItemFamilies.FLAT_KEYS[field])
			if DictUtil.to_int(item.get(flat, 0)) != DictUtil.to_int(fresh.get(flat, 0)):
				changes.append({"field": field, "old": DictUtil.to_int(item.get(flat, 0)), "new": DictUtil.to_int(fresh.get(flat, 0))})
		var item_errors: Array[String] = ItemValidator.item_errors(fresh, fam, true)
		var old_value: float = GearBudget.item_value(item)
		var new_value: float = GearBudget.item_value(fresh)
		if changes.is_empty() and item_errors.is_empty():
			continue
		if not item_errors.is_empty():
			errors += 1
		delta_by_id[str(item.get("id", ""))] = new_value - old_value
		items.append({"id": str(item.get("id", "")), "name": str(item.get("name", "")), "changes": changes, "value_old": old_value,
			"value_new": new_value, "error": item_errors[0] if not item_errors.is_empty() else ""})
	var loadouts: Array[Dictionary] = []
	var named: Array[String] = []
	if not delta_by_id.is_empty():
		for kind: String in ["classes", "units"]:
			for entry: Dictionary in ContentCatalog.entries(kind):
				var shift: float = 0.0
				var hit: bool = false
				for row: Dictionary in GearBudget.loadout_breakdown(entry):
					if delta_by_id.has(str(row["id"])):
						shift += DictUtil.to_float(delta_by_id[str(row["id"])])
						hit = true
				if hit:
					var old_total: float = GearBudget.loadout_value(entry)
					loadouts.append({"kind": kind, "id": str(entry.get("id", "")), "name": str(entry.get("name", "")), "old": old_total, "new": old_total + shift})
		for id: Variant in delta_by_id:
			named.append_array(named_users(str(id)))
	return {"items": items, "loadouts": loadouts, "named": named, "count": items.size(), "errors": errors, "text": impact_text(items, loadouts, named)}

# "2 items change: wpn_x power 18 -> 20; class loadout totals warrior 27.0 -> 29.0; 1 named item" (or "No item changes.").
static func impact_text(items: Array[Dictionary], loadouts: Array[Dictionary], named: Array[String]) -> String:
	if items.is_empty():
		return "No item changes."
	var parts: Array[String] = []
	for item: Dictionary in items:
		var bits: Array[String] = []
		for c: Variant in DictUtil.array_of(item["changes"]):
			var change: Dictionary = DictUtil.dict_of(c)
			bits.append("%s %d -> %d" % [str(change["field"]), DictUtil.to_int(change["old"]), DictUtil.to_int(change["new"])])
		if not str(item["error"]).is_empty():
			bits.append("BREAKS: " + str(item["error"]))
		parts.append("%s %s" % [str(item["id"]), ", ".join(bits)])
	var text: String = "%d item%s change: %s" % [items.size(), "" if items.size() == 1 else "s", "; ".join(parts)]
	var totals: Array[String] = []
	for l: Dictionary in loadouts:
		totals.append("%s %.1f -> %.1f" % [str(l["id"]), DictUtil.to_float(l["old"]), DictUtil.to_float(l["new"])])
	if not totals.is_empty():
		text += "; loadout totals " + ", ".join(totals)
	if not named.is_empty():
		text += "; %d named item%s (%s)" % [named.size(), "" if named.size() == 1 else "s", ", ".join(named)]
	return text

# True when Apply of a family draft needs the second (confirm) click: it changes or breaks at least one item.
static func needs_confirm(draft: Dictionary, stored_id: String = "") -> bool:
	return DictUtil.to_int(impact(draft, stored_id)["count"]) > 0

# The items of a family: [{"id", "name", "hand"}].
static func used_by(family_id: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for item: Dictionary in ContentCatalog.items_of_family(family_id):
		out.append({"id": str(item.get("id", "")), "name": str(item.get("name", "")), "hand": str(item.get("hand", ""))})
	return out

# ------------------------------------------------------------------------------
# Delete state (FORGE_SHARED 12.3) and the item list filters (matrix 6)
# ------------------------------------------------------------------------------

# {"allowed", "reason"} of Delete for a stored asset: system assets go through SAFE DELETE; a family with items and an
# item a class, unit or other entry uses are refused with the users listed.
static func delete_state(kind: String, id: String) -> Dictionary:
	if id.is_empty() or not ContentCatalog.has(kind, id):
		return {"allowed": true, "reason": ""}
	if ContentCatalog.is_system_origin(ContentCatalog.get_entry(kind, id)):
		return {"allowed": false, "reason": SAFE_DELETE_REASON}
	if kind == "item_families":
		var n: int = ContentCatalog.items_of_family(id).size()
		if n > 0:
			return {"allowed": false, "reason": "%d item(s) use this family; a family with items cannot be removed." % n}
	var users: Array[String] = ContentCatalog.references_to(kind, id)
	if not users.is_empty():
		return {"allowed": false, "reason": "Used by: %s." % ", ".join(users)}
	return {"allowed": true, "reason": ""}

# Missing keys mean "any": family / hand / rarity / origin ("" = any), special / has_tiers / has_effects (any | yes | no),
# used_by (any | class | unit | named | unused), text (substring of id or name).
static func default_filters() -> Dictionary:
	return {"text": "", "family": "", "hand": "", "rarity": "", "special": "any", "has_tiers": "any", "has_effects": "any", "origin": "", "used_by": "any"}

# Choices of one filter chip (the first entry "" / "any" = no filter).
static func filter_options(key: String) -> Array[String]:
	var out: Array[String] = []
	match key:
		"family":
			out.append("")
			for f: Dictionary in ContentCatalog.entries("item_families"):
				out.append(str(f.get("id", "")))
		"hand":
			out.append("")
			out.append_array(ItemFamilies.HANDS)
		"rarity":
			out.append("")
			out.append_array(ContentSchema.options_of(ContentSchema.field("items", "rarity")))
		"origin":
			out.append_array(["", ContentSchema.ORIGIN_SYSTEM, ContentSchema.ORIGIN_USER])
		"used_by":
			out.append("any")
			out.append_array(USAGES)
		"special", "has_tiers", "has_effects":
			out.append_array(TRI)
	return out

static func matches(row: Dictionary, filters: Dictionary) -> bool:
	var needle: String = str(filters.get("text", "")).strip_edges().to_lower()
	var id: String = str(row.get("id", ""))
	if not needle.is_empty() and not ("%s %s" % [id, str(row.get("name", ""))]).to_lower().contains(needle):
		return false
	for key: String in ["family", "hand", "rarity"]:
		var want: String = str(filters.get(key, ""))
		if not want.is_empty() and str(row.get(key, "")) != want:
			return false
	var origin: String = str(filters.get("origin", ""))
	if not origin.is_empty() and (ContentCatalog.is_system_origin(row) != (origin == ContentSchema.ORIGIN_SYSTEM)):
		return false
	if not _tri(str(filters.get("special", "any")), DictUtil.to_bool(row.get("special", false))):
		return false
	if not _tri(str(filters.get("has_tiers", "any")), not DictUtil.array_of(row.get("tiers", [])).is_empty()):
		return false
	if not _tri(str(filters.get("has_effects", "any")), not DictUtil.array_of(row.get("effects", [])).is_empty()):
		return false
	var usage: String = str(filters.get("used_by", "any"))
	if usage != "any":
		var users: Array[String] = ContentCatalog.references_to("items", id)
		var named: Array[String] = named_users(id)
		match usage:
			"class":
				return users.any(func(u: String) -> bool: return u.begins_with("classes:"))
			"unit":
				return users.any(func(u: String) -> bool: return u.begins_with("units:"))
			"named":
				return not named.is_empty()
			"unused":
				return users.is_empty() and named.is_empty()
	return true

static func _tri(mode: String, value: bool) -> bool:
	return mode == "any" or (mode == "yes" and value) or (mode == "no" and not value)

# The item rows that pass `filters`, in catalog order.
static func filter_items(rows: Array[Dictionary], filters: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for row: Dictionary in rows:
		if matches(row, filters):
			out.append(row)
	return out
