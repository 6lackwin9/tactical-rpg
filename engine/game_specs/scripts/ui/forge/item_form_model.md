---
class: ItemFormModel
extends: RefCounted
description: "Tactical RPG - Item Form Model (FORGE1_ITEMS 6, matrix 6; ITEM1-L commit 3) The headless brain of the item and item-family tabs of the Content Forge viewer: pure static functions over an entry. An item draft is a RESOLVED row (what ContentCatalog.get_entry returns); `sync` re-resolves it so the derived keys (item type, numbers, threshold tiers) follow the stored ones (family, hand, modifiers, tiers); `stored_of(draft)` is its stored shape (family + modifiers, ItemModifiers.stored). Layout (eight item sections, six family sections), control state (options from the family, dynamic limits), the modifier and tier tables (rows, defaults, limits from ItemModifiers.KINDS and Passives.effect_specs), the base-values grid, the budget breakdown, the problems (errors), the warnings and the item list filters. The controls (ItemTables, ForgeForm) only render what this returns; ContentForge wires it (state_fn, layout_fn, problems_fn, panel_fn) for the \"items\" and \"item_families\" tabs."
---

# Properties
* `ITEM_SECTIONS`: Array[String] = ["identity", "build", "base", "modifiers", "tiers", "special", "consumable", "preview"] (const)
* `FAMILY_SECTIONS`: Array[String] = ["identity", "type", "numbers", "policy", "rules", "used_by"] (const)
* `SECTION_TITLES`: Dictionary = { (const)
* `ITEM_FIELD_SECTIONS`: Dictionary = { (const)
* `FAMILY_FIELD_SECTIONS`: Dictionary = { (const)
* `ITEM_PANELS`: Dictionary = {"identity": {"post": ["tokens"]}, "build": {"pre": ["chips"]}, "base": {"post": ["base"]}, "preview": {"post": ["preview"]}} (const)
* `FAMILY_PANELS`: Dictionary = {"used_by": {"post": ["used_by"]}} (const)
* `FAMILY_USED_REASON`: String = "%d item(s) use this family: the %s is fixed while they do." (const)
* `USAGES`: Array[String] = ["class", "unit", "named", "unused"] (const)
* `TRI`: Array[String] = ["any", "yes", "no"] (const)
* `families`: bool = kind == "item_families"
* `order`: Array[String] = FAMILY_SECTIONS if families else ITEM_SECTIONS
* `sections`: Dictionary = FAMILY_FIELD_SECTIONS if families else ITEM_FIELD_SECTIONS
* `panels`: Dictionary = FAMILY_PANELS if families else ITEM_PANELS
* `titles`: Dictionary = SECTION_TITLES[kind]
* `by_section`: Dictionary = {}
* `visible_row`: Dictionary = ItemFamilies.derive(ContentSchema.normalize(kind, draft)) if families else draft
* `key`: String = str(raw_key)
* `section`: String = str(sections[key])
* `keys`: Array = by_section.get(section, [])
* `out`: Array[Dictionary] = []
* `p`: Dictionary = DictUtil.dict_of(panels.get(section, {}))
* `keys`: Array = by_section.get(section, [])
* `out`: Array[Dictionary] = []
* `id`: String = str(f.get("id", ""))
* `out`: Array[String] = [""]
* `out`: Array[Dictionary] = []
* `messages`: Array[String] = []
* `candidate`: Dictionary = {}
* `row`: Dictionary = resolved(draft)
* `family`: Dictionary = family_row(draft)
* `known`: Dictionary = {}
* `old`: Dictionary = ContentCatalog.get_entry(kind, stored_id if not stored_id.is_empty() else str(draft.get("id", "")))
* `row`: Dictionary = resolved(draft)
* `out`: Array[String] = GearBudget.check_item(row)
* `out`: Array[String] = []
* `row`: Dictionary = resolved(draft)
* `out`: Array[Dictionary] = [
* `out`: Array[Dictionary] = []
* `family`: Dictionary = family_row(draft)
* `hand`: String = ItemModifiers.hand_in(draft, family)
* `base`: Dictionary = ItemFamilies.base_of(family, hand)
* `raw`: Dictionary = ItemModifiers.raw_numbers(draft, family)
* `rng`: Array = ItemFamilies.NUMBER_RANGES[field]
* `b`: int = DictUtil.to_int(base[field])
* `result`: int = DictUtil.to_int(raw[field])
* `band`: Dictionary = ItemFamilies.band_of(family, hand, field)
* `in_range`: bool = result >= DictUtil.to_int(rng[0]) and result <= DictUtil.to_int(rng[1])
* `in_band`: bool = band.is_empty() or (result >= DictUtil.to_int(band.get("min", result)) and result <= DictUtil.to_int(band.get("max", result)))
* `text`: String = "%d %s %d = %d" % [b, "-" if result - b < 0 else "+", absi(result - b), result]
* `row`: Dictionary = resolved(draft)
* `type`: String = str(row.get("item_type", "WEAPON"))
* `out`: Array[String] = []
* `stats`: Dictionary = DictUtil.dict_of(row.get("required_stats" if token == "req" else "stat_bonuses", {}))
* `tiers`: Array[Dictionary] = ItemModifiers.tiers_of(row)
* `candidate`: String = "tier%d.%s" % [i + 1, token.get_slice(".", 1)]
* `out`: Array[String] = []
* `out`: Array[Dictionary] = []
* `family`: Dictionary = family_row(draft)
* `raw`: Dictionary = ItemModifiers.raw_numbers(draft, family) if not family.is_empty() else {}
* `mods`: Array[Dictionary] = ItemModifiers.modifiers_of(draft)
* `mod`: Dictionary = mods[i]
* `result`: Variant = null
* `out`: Array[String] = []
* `out`: Array[String] = []
* `out`: Array[Dictionary] = []
* `spec`: Dictionary = Passives.effect_spec(str(fx.get("type", "")))
* `p`: Dictionary = DictUtil.dict_of(raw)
* `key`: String = str(p["key"])
* `options`: Array = DictUtil.array_of(p.get("options", []))
* `out`: Array[Dictionary] = []
* `tiers`: Array[Dictionary] = ItemModifiers.tiers_of(draft)
* `t`: Dictionary = tiers[i]
* `when`: Dictionary = DictUtil.dict_of(t.get("when", {}))
* `bonus`: Dictionary = DictUtil.dict_of(t.get("bonus", {}))
* `hands`: Array[String] = ItemFamilies.hands_of(draft)
* `out`: Array[String] = ItemFamilies.number_fields_for(ItemFamilies.item_type_of(draft))
* `out`: Array[Dictionary] = []
* `out`: Array[String] = []
* `out`: Array[Dictionary] = []

# Methods

### `static family_row(draft: Dictionary) -> Dictionary`
```logic
return ContentCatalog.get_entry("item_families", str(draft.get("family", "")))
```

### `static item_type(draft: Dictionary) -> String`
```logic
var family: Dictionary = family_row(draft)
return ItemFamilies.item_type_of(family) if not family.is_empty() else str(draft.get("item_type", "WEAPON"))
```

### `static resolved(draft: Dictionary) -> Dictionary`
```logic
var family: Dictionary = family_row(draft)
if family.is_empty():
	return ContentSchema.normalize("items", draft)
return ContentSchema.normalize("items", ItemModifiers.resolve(ItemModifiers.stored(draft), family))
```

### `static stored_of(draft: Dictionary) -> Dictionary`
```logic
if family_row(draft).is_empty():
	return draft.duplicate(true)
return ItemModifiers.stored(draft)
```

### `static sync(draft: Dictionary, changed_key: String = "") -> void`
```logic
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
```

### `static new_entry(id: String) -> Dictionary`
```logic
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
```

### `static field_visible(kind: String, draft: Dictionary, key: String, row: Dictionary = {}) -> bool`
```logic
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
```

### `static _blank_state() -> Dictionary`
```logic
return {"enabled": true, "reason": "", "min": null, "max": null, "step": null, "options": [], "disabled_options": {}}
```

### `static state(kind: String, draft: Dictionary, key: String, stored_id: String = "") -> Dictionary`
```logic
var out: Dictionary = _blank_state()
var f: Dictionary = ContentSchema.field(kind, key)
if f.is_empty():
	return out
if DictUtil.to_bool(f.get("readonly", false)):
	out["enabled"] = false
if kind == "item_families":
	_family_state(out, draft, key, stored_id)
else:
	_item_state(out, draft, key, stored_id)
return out
```

### `static _item_state(out: Dictionary, draft: Dictionary, key: String, stored_id: String) -> void`
```logic
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
		out["reason"] = "Empty = the family default (%s)." % (str(family.get("damage_type", "")) if not str(family.get("damage_type", "")).is_empty() else "none")
	"modifiers":
		out["max"] = DictUtil.to_int(family.get("max_modifiers", 0))
		out["add"] = can_add_modifier(draft, stored_id)
	"tiers":
		out["max"] = max_tiers(draft)
		out["add"] = can_add_tier(draft, stored_id)
```

### `static _family_state(out: Dictionary, draft: Dictionary, key: String, stored_id: String) -> void`
```logic
var f: Dictionary = ItemFamilies.derive(ContentSchema.normalize("item_families", draft))
var type: String = ItemFamilies.item_type_of(f)
var users: int = ContentCatalog.items_of_family(stored_id).size() if not stored_id.is_empty() else 0
var disabled: Dictionary = out["disabled_options"]
match key:
	"id", "item_type":
		if users > 0:
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
```

### `static rule_blocker(f: Dictionary, rule: String) -> String`
```logic
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
```

### `static max_tiers(draft: Dictionary) -> int`
```logic
return DictUtil.to_int(family_row(draft).get("max_tiers", 0))
```

### `static save_state(kind: String, draft: Dictionary, stored_id: String = "") -> Dictionary`
```logic
var problems: Array[Dictionary] = draft_problems(kind, draft, stored_id)
var reasons: Array[String] = []
for p: Dictionary in problems:
	reasons.append(str(p["message"]))
return {"blocked": not problems.is_empty(), "problems": problems, "reasons": reasons, "warnings": warnings(kind, draft)}
```

### `static blocked_title(problems: Array[Dictionary]) -> String`
```logic
if problems.is_empty():
	return ""
return "Cannot save: %d problem%s" % [problems.size(), "" if problems.size() == 1 else "s"]
```

### `static field_of_message(kind: String, message: String) -> String`
```logic
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
```

### `static budget_breakdown(draft: Dictionary) -> Dictionary`
```logic
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
```

### `static description_preview(draft: Dictionary) -> Dictionary`
```logic
var row: Dictionary = resolved(draft)
var template: String = str(draft.get("description", ""))
var problems: Array[String] = []
for p: Dictionary in ItemText.validate(template, row):
	problems.append(str(p["message"]))
return {"text": ItemText.render(template, row), "problems": problems, "warnings": ItemText.warnings(template, row)}
```

### `static preview(draft: Dictionary) -> Dictionary`
```logic
var row: Dictionary = resolved(draft)
var tiers: Array[String] = []
for t: Dictionary in tier_rows(draft, []):
	tiers.append("%s: %s" % [str(t["criterion_text"]), str(t["desc"]) if not str(t["desc"]).is_empty() else str(t["auto_desc"])])
var id: String = str(draft.get("id", ""))
return {"summary": ItemText.modifier_summary(row), "hand": ItemText.hand_label(row) if not str(row.get("hand", "")).is_empty() else "",
	"tiers": tiers, "budget": budget_breakdown(draft), "users": ContentCatalog.references_to("items", id), "named": named_users(id)}
```

### `static modifier_text(mod: Dictionary) -> String`
```logic
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
```

### `static effect_text(fx: Dictionary) -> String`
```logic
var parts: Array[String] = [str(fx.get("type", "?"))]
for key: Variant in fx:
	if str(key) != "type":
		parts.append("%s %s" % [str(key), str(fx[key])])
return " ".join(parts)
```

### `static modifier_kind_state(draft: Dictionary) -> Dictionary`
```logic
var family: Dictionary = family_row(draft)
var allowed: Array[String] = ContentFactory._strings(family.get("modifier_kinds", []))
var disabled: Dictionary = {}
for kind: String in ItemModifiers.kind_names():
	if not allowed.has(kind):
		disabled[kind] = "Family '%s' does not allow %s modifiers." % [str(family.get("id", "")), kind]
return {"options": ItemModifiers.kind_names(), "disabled": disabled}
```

### `static can_add_modifier(draft: Dictionary, _stored_id: String = "") -> Dictionary`
```logic
var family: Dictionary = family_row(draft)
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
```

### `static default_modifier(draft: Dictionary, kind: String) -> Dictionary`
```logic
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
```

### `static adjust_field_state(draft: Dictionary, index: int) -> Dictionary`
```logic
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
```

### `static delta_limits(draft: Dictionary, index: int, field: String = "") -> Dictionary`
```logic
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
```

### `static stat_state(draft: Dictionary, index: int, kind: String) -> Dictionary`
```logic
var disabled: Dictionary = {}
var mods: Array[Dictionary] = ItemModifiers.modifiers_of(draft)
for i: int in range(mods.size()):
	if i != index and str(mods[i].get("kind", "")) == kind:
		disabled[str(mods[i].get("stat", ""))] = "Already in row %d (one %s per stat)." % [i + 1, kind]
var spec: Dictionary = ItemModifiers.KINDS[kind]
return {"options": ContentSchema.STAT_KEYS.duplicate(), "disabled": disabled, "min": DictUtil.to_int(spec["min"]), "max": DictUtil.to_int(spec["max"])}
```

### `static _modifier_list(draft: Dictionary) -> Array`
```logic
return DictUtil.array_of(ContentSchema._copy(DictUtil.array_of(draft.get("modifiers", []))))
```

### `static with_modifier_added(draft: Dictionary, kind: String) -> Array`
```logic
var list: Array = _modifier_list(draft)
list.append(default_modifier(draft, kind))
return list
```

### `static with_modifier_set(draft: Dictionary, index: int, mod: Dictionary) -> Array`
```logic
var list: Array = _modifier_list(draft)
if index >= 0 and index < list.size():
	list[index] = mod.duplicate(true)
return list
```

### `static with_modifier_kind(draft: Dictionary, index: int, kind: String) -> Array`
```logic
var without: Dictionary = draft.duplicate()
var list: Array = _modifier_list(draft)
if index < 0 or index >= list.size():
	return list
list.remove_at(index)
without["modifiers"] = list
var fresh: Dictionary = default_modifier(without, kind)
list.insert(index, fresh)
return list
```

### `static with_modifier_removed(draft: Dictionary, index: int) -> Array`
```logic
var list: Array = _modifier_list(draft)
if index >= 0 and index < list.size():
	list.remove_at(index)
return list
```

### `static with_modifier_moved(draft: Dictionary, index: int, step: int) -> Array`
```logic
var list: Array = _modifier_list(draft)
var to: int = index + step
if index < 0 or index >= list.size() or to < 0 or to >= list.size():
	return list
var mod: Variant = list[index]
list.remove_at(index)
list.insert(to, mod)
return list
```

### `static effect_default(type: String) -> Dictionary`
```logic
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
```

### `static param_default(p: Dictionary) -> Variant`
```logic
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
```

### `static effect_with_param(fx: Dictionary, key: String, value: Variant) -> Dictionary`
```logic
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
```

### `static effect_with_choice(fx: Dictionary, key: String) -> Dictionary`
```logic
var spec: Dictionary = Passives.effect_spec(str(fx.get("type", "")))
for raw: Variant in DictUtil.array_of(spec.get("params", [])):
	var p: Dictionary = DictUtil.dict_of(raw)
	if str(p["key"]) == key:
		return effect_with_param(fx, key, param_default(p))
return fx.duplicate(true)
```

### `static tier_limits(draft: Dictionary) -> Dictionary`
```logic
var type: String = item_type(draft)
return {"when_types": ItemModifiers.WHEN_TYPES.duplicate(), "stat_min": ItemModifiers.TIER_STAT_MIN, "stat_max": ItemModifiers.TIER_STAT_MAX,
	"proficiency_min": 1, "proficiency_max": Proficiency.MAX_LEVEL, "level_min": 1, "level_max": StatRules.MAX_LEVEL,
	"gear_options": Proficiency.gear_types(), "gear_default": str(draft.get("family", "")),
	"bonus_keys": ItemModifiers.tier_bonus_keys(type), "bonus_ranges": ItemModifiers.TIER_BONUS_RANGES.duplicate(true),
	"max_stats": ItemModifiers.TIER_MAX_STATS, "max_effects": ItemModifiers.TIER_MAX_EFFECTS,
	"label_max": ItemModifiers.TIER_LABEL_MAX, "desc_max": ItemModifiers.TIER_DESC_MAX, "max_tiers": max_tiers(draft)}
```

### `static can_add_tier(draft: Dictionary, _stored_id: String = "") -> Dictionary`
```logic
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
```

### `static when_default(draft: Dictionary, type: String, skip_index: int = -1) -> Dictionary`
```logic
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
```

### `static default_tier(draft: Dictionary) -> Dictionary`
```logic
var keys: Array[String] = ItemModifiers.tier_bonus_keys(item_type(draft))
var bonus: Dictionary = {}
if not keys.is_empty():
	bonus = bonus_default(draft, keys[0])
return {"label": "Tier %d" % (ItemModifiers.tiers_of(draft).size() + 1), "when": when_default(draft, ItemModifiers.WHEN_STAT), "bonus": bonus, "desc": ""}
```

### `static bonus_default(draft: Dictionary, key: String) -> Dictionary`
```logic
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
```

### `static criterion_text(when: Dictionary) -> String`
```logic
match str(when.get("type", "")):
	ItemModifiers.WHEN_STAT:
		return "%s >= %d" % [str(when.get("stat", "")), DictUtil.to_int(when.get("value", 0))]
	ItemModifiers.WHEN_PROFICIENCY:
		return "%s training >= %d" % [str(when.get("gear", "")), DictUtil.to_int(when.get("level", 0))]
	ItemModifiers.WHEN_LEVEL:
		return "unit level >= %d" % DictUtil.to_int(when.get("value", 0))
return "?"
```

### `static _tier_list(draft: Dictionary) -> Array`
```logic
return DictUtil.array_of(ContentSchema._copy(DictUtil.array_of(draft.get("tiers", []))))
```

### `static with_tier_added(draft: Dictionary) -> Array`
```logic
var list: Array = _tier_list(draft)
list.append(default_tier(draft))
return list
```

### `static with_tier_set(draft: Dictionary, index: int, tier: Dictionary) -> Array`
```logic
var list: Array = _tier_list(draft)
if index >= 0 and index < list.size():
	list[index] = tier.duplicate(true)
return list
```

### `static with_tier_when_type(draft: Dictionary, index: int, type: String) -> Array`
```logic
var list: Array = _tier_list(draft)
if index < 0 or index >= list.size():
	return list
var tier: Dictionary = DictUtil.dict_of(list[index]).duplicate(true)
tier["when"] = when_default(draft, type, index)
list[index] = tier
return list
```

### `static with_tier_removed(draft: Dictionary, index: int) -> Array`
```logic
var list: Array = _tier_list(draft)
if index >= 0 and index < list.size():
	list.remove_at(index)
return list
```

### `static with_tier_moved(draft: Dictionary, index: int, step: int) -> Array`
```logic
var list: Array = _tier_list(draft)
var to: int = index + step
if index < 0 or index >= list.size() or to < 0 or to >= list.size():
	return list
var tier: Variant = list[index]
list.remove_at(index)
list.insert(to, tier)
return list
```

### `static new_family(id: String) -> Dictionary`
```logic
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
```

### `static sync_family(draft: Dictionary, changed_key: String = "") -> void`
```logic
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
```

### `static _minimum_row(type: String) -> Dictionary`
```logic
var row: Dictionary = {}
for field: String in ItemFamilies.number_fields_for(type):
	var rng: Array = ItemFamilies.NUMBER_RANGES[field]
	row[field] = DictUtil.to_int(rng[0])
return row
```

### `static grid_range(field: String) -> Dictionary`
```logic
if field == ItemFamilies.BONUS_BAND:
	return {"min": 0.0, "max": 99.0, "step": 0.5}
var rng: Array = ItemFamilies.NUMBER_RANGES[field]
return {"min": DictUtil.to_float(rng[0]), "max": DictUtil.to_float(rng[1]), "step": 1.0}
```

### `static base_cell(draft: Dictionary, key: String, field: String) -> int`
```logic
return DictUtil.to_int(ItemFamilies.base_of(draft, key)[field]) if ItemFamilies.hands_of(draft).has(key) or key == ItemFamilies.NO_HAND_KEY else 0
```

### `static with_base(draft: Dictionary, key: String, field: String, value: int) -> Dictionary`
```logic
var base: Dictionary = DictUtil.dict_of(draft.get("base", {})).duplicate(true)
var row: Dictionary = DictUtil.dict_of(base.get(key, _minimum_row(ItemFamilies.item_type_of(draft)))).duplicate()
row[field] = value
base[key] = row
return base
```

### `static band_cell(draft: Dictionary, key: String, field: String) -> Dictionary`
```logic
var band: Dictionary = DictUtil.dict_of(DictUtil.dict_of(DictUtil.dict_of(draft.get("bands", {})).get(key, {})).get(field, {}))
return {"set": not band.is_empty(), "min": band.get("min", null), "max": band.get("max", null)}
```

### `static with_band(draft: Dictionary, key: String, field: String, bound: String, value: float) -> Dictionary`
```logic
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
```

### `static without_band(draft: Dictionary, key: String, field: String) -> Dictionary`
```logic
var bands: Dictionary = DictUtil.dict_of(draft.get("bands", {})).duplicate(true)
var row: Dictionary = DictUtil.dict_of(bands.get(key, {})).duplicate(true)
row.erase(field)
if row.is_empty():
	bands.erase(key)
else:
	bands[key] = row
return bands
```

### `static default_filters() -> Dictionary`
```logic
return {"text": "", "family": "", "hand": "", "rarity": "", "special": "any", "has_tiers": "any", "has_effects": "any", "origin": "", "used_by": "any"}
```

### `static matches(row: Dictionary, filters: Dictionary) -> bool`
```logic
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
```

### `static _tri(mode: String, value: bool) -> bool`
```logic
return mode == "any" or (mode == "yes" and value) or (mode == "no" and not value)
```

