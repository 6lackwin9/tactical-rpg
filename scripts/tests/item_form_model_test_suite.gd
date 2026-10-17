class_name ItemFormModelTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Item Form Model Test Suite (ITEM1-L commit 3, FORGE1_ITEMS 6, matrix 4 suite 4 and 6)
# The headless item and item-family tabs: sections from the matrix layout (no derived key is a control), a new item and a
# new family that validate, problems from errors (never from warnings) with the row index of a table problem, the
# key-field rule on system items (no viewer locks), dynamic limits (delta keeps the resolved value in range, stat values
# from ItemModifiers.KINDS), defaults that validate for every modifier kind, effect type and tier criterion, armor and
# shield tiers through `tiers`, the token palette, the base-values grid and budget breakdown (sum = item_value), the
# family batch upsert and used-by, the item list filters, and the read-only Forge viewer item / family tabs.
# Fixtures are picked by rule; every catalog edit runs in a sandbox fork.
# ==============================================================================

const TAG: String = "ItemForm"
static var SANDBOX_MODEL: String = TestHelpers.sandbox("item_form_model")

static func run(tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING ITEM FORM MODEL TEST (ITEM1-L c3)       ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	if not ContentCatalog.items_inherit():
		return TestHelpers.report_ok(TAG, "Item form model (needs item families)", true, "skipped: legacy flat items")
	var ok: bool = true
	ok = _test_layout() and ok
	ok = _test_new_and_save_state() and ok
	ok = _test_key_fields() and ok
	ok = _test_modifier_limits() and ok
	ok = _test_effects() and ok
	ok = _test_tiers() and ok
	ok = _test_tokens_and_budget() and ok
	ok = _test_filters() and ok
	ok = _test_family_batch() and ok
	ok = _test_forge_ui(tree_parent) and ok
	return ok

# ------------------------------------------------------------------------------
# Fixtures (by rule)
# ------------------------------------------------------------------------------

# The first shipped item whose family has `type` and passes `extra` (a Callable(item) -> bool; empty = any).
static func _item_of_type(type: String, extra: Callable = Callable()) -> Dictionary:
	for item: Dictionary in ContentCatalog.entries("items"):
		if ItemFormModel.item_type(item) == type and (not extra.is_valid() or DictUtil.to_bool(extra.call(item))):
			return item.duplicate(true)
	return {}

static func _with_tier_room(item: Dictionary) -> bool:
	return ItemFormModel.max_tiers(item) > 0

static func _keys_of(blocks: Array[Dictionary]) -> Array[String]:
	var out: Array[String] = []
	for block: Dictionary in blocks:
		for k: Variant in DictUtil.array_of(block["fields"]):
			out.append(str(k))
	return out

static func _ids_of(blocks: Array[Dictionary]) -> Array[String]:
	var out: Array[String] = []
	for block: Dictionary in blocks:
		out.append(str(block["id"]))
	return out

static func _has_field(problems: Array[Dictionary], field: String, index: int = -1) -> bool:
	for p: Dictionary in problems:
		if str(p["field"]) == field and (index < 0 or DictUtil.to_int(p.get("index", 0)) == index):
			return true
	return false

# A draft with its modifiers replaced (and re-resolved).
static func _with_mods(draft: Dictionary, mods: Array) -> Dictionary:
	var d: Dictionary = draft.duplicate(true)
	d["modifiers"] = mods
	ItemFormModel.sync(d, "modifiers")
	return d

# ------------------------------------------------------------------------------

static func _test_layout() -> bool:
	var checks: Dictionary = {}
	var weapon: Dictionary = _item_of_type("WEAPON")
	var blocks: Array[Dictionary] = ItemFormModel.layout("items", weapon)
	var ids: Array[String] = _ids_of(blocks)
	var ordered: bool = true
	var last: int = -1
	for id: String in ids:
		var at: int = ItemFormModel.ITEM_SECTIONS.find(id)
		ordered = ordered and at > last
		last = at
	checks["item sections follow the matrix order"] = ordered and ids.has("identity") and ids.has("build") and ids.has("base") and ids.has("modifiers") and ids.has("preview")
	var keys: Array[String] = _keys_of(blocks)
	checks["no derived key is a control"] = not ItemModifiers.DERIVED_KEYS.any(func(k: String) -> bool: return keys.has(k))
	checks["a weapon shows hand and scaling, no slot, no consumable payload"] = keys.has("hand") and keys.has("scaling_stat") and not keys.has("slot") and not ids.has("consumable")
	var unique: Dictionary = {}
	for k: String in keys:
		unique[k] = true
	checks["every control appears once"] = unique.size() == keys.size()
	checks["the tier section shows iff the family allows tiers or the item has some"] = ids.has("tiers") == (ItemFormModel.max_tiers(weapon) > 0 or not ItemModifiers.tiers_of(weapon).is_empty())
	var armor: Dictionary = _item_of_type("ARMOR")
	var armor_keys: Array[String] = _keys_of(ItemFormModel.layout("items", armor))
	checks["armor shows its slot, never hand or scaling"] = not armor.is_empty() and armor_keys.has("slot") and not armor_keys.has("hand") and not armor_keys.has("scaling_stat")
	var consumable: Dictionary = _item_of_type("CONSUMABLE")
	var c_ids: Array[String] = _ids_of(ItemFormModel.layout("items", consumable))
	checks["a consumable shows its payload, no modifiers, tiers or base values"] = not consumable.is_empty() and c_ids.has("consumable") and not c_ids.has("modifiers") and not c_ids.has("tiers") and not c_ids.has("base")
	var family: Dictionary = ContentCatalog.entries("item_families")[0]
	var f_blocks: Array[Dictionary] = ItemFormModel.layout("item_families", family)
	var f_ids: Array[String] = _ids_of(f_blocks)
	checks["family sections: identity, type, numbers, policy, rules, used by"] = f_ids == ItemFormModel.FAMILY_SECTIONS
	var f_keys: Array[String] = _keys_of(f_blocks)
	var visible: int = 0
	var derived: Dictionary = ItemFamilies.derive(family)
	for f: Dictionary in ContentSchema.fields("item_families"):
		if ContentSchema.is_visible(f, derived):
			visible += 1
	checks["the family form lists every visible family field once"] = f_keys.size() == visible
	var panels: Dictionary = {}
	for block: Dictionary in blocks:
		for p: Variant in DictUtil.array_of(block["pre"]) + DictUtil.array_of(block["post"]):
			panels[str(p)] = true
	checks["item panels: tokens, chips, base values, preview"] = panels.has("tokens") and panels.has("chips") and panels.has("base") and panels.has("preview")
	return TestHelpers.report(TAG, "Sections from the matrix layout; derived keys are never controls", checks)

static func _test_new_and_save_state() -> bool:
	var checks: Dictionary = {}
	var fresh: Dictionary = ItemFormModel.new_entry(ContentCatalog.unique_id("items", "t_if_new"))
	checks["a new item validates and is a user asset"] = ItemFormModel.draft_problems("items", fresh).is_empty() and str(fresh["origin"]) == ContentSchema.ORIGIN_USER
	checks["...of a WEAPON family, resolved (item type, gear token, numbers)"] = str(fresh["item_type"]) == "WEAPON" and str(fresh["weapon_type"]) == str(fresh["family"]) and DictUtil.to_int(fresh["base_power"]) == DictUtil.to_int(ItemFamilies.base_of(ItemFormModel.family_row(fresh), str(fresh["hand"]))["power"])
	checks["...Save is allowed"] = not DictUtil.to_bool(ItemFormModel.save_state("items", fresh)["blocked"])
	checks["the stored shape keeps family + modifiers only"] = ItemModifiers.is_stored_shape(ItemFormModel.stored_of(fresh))
	var family: Dictionary = ItemFormModel.family_row(fresh)
	var power_range: Array = ItemFamilies.NUMBER_RANGES["power"]
	var top: int = DictUtil.to_int(power_range[1])
	var base_power: int = DictUtil.to_int(ItemFamilies.base_of(family, str(fresh["hand"]))["power"])
	var broken: Dictionary = _with_mods(fresh, [{"kind": "adjust", "field": "power", "delta": top - base_power + 1}])
	var problems: Array[Dictionary] = ItemFormModel.draft_problems("items", broken)
	checks["a resolved power above the schema range blocks Save on modifiers"] = _has_field(problems, "modifiers") and DictUtil.to_bool(ItemFormModel.save_state("items", broken)["blocked"])
	var zero: Dictionary = _with_mods(fresh, [{"kind": "adjust", "field": "power", "delta": 0}])
	checks["a zero delta is a problem of row 1"] = _has_field(ItemFormModel.draft_problems("items", zero), "modifiers", 1)
	checks["the problems bar title counts them"] = ItemFormModel.blocked_title(problems).begins_with("Cannot save: %d problem" % problems.size())
	var special: Dictionary = fresh.duplicate(true)
	special["special"] = true
	special["special_note"] = ""
	var s_state: Dictionary = ItemFormModel.save_state("items", special)
	checks["a warning only (special without note) never blocks"] = not DictUtil.to_bool(s_state["blocked"]) and not DictUtil.array_of(s_state["warnings"]).is_empty()
	var token: Dictionary = fresh.duplicate(true)
	token["description"] = "{no_such_token}"
	checks["an unknown token is a description problem"] = _has_field(ItemFormModel.draft_problems("items", token), "description")
	var fam: Dictionary = ItemFormModel.new_family(ContentCatalog.unique_id("item_families", "t_if_fam"))
	checks["a new family validates and is a user, non-gear type"] = ItemFormModel.draft_problems("item_families", fam).is_empty() and str(fam["origin"]) == ContentSchema.ORIGIN_USER and not ItemFamilies.is_gear_type(str(fam["item_type"]))
	checks["...its base has one row per grid key"] = DictUtil.dict_of(fam["base"]).keys().size() == ItemFormModel.grid_keys(fam).size()
	var switched: Dictionary = fam.duplicate(true)
	switched["item_type"] = "ARMOR"
	ItemFormModel.sync_family(switched, "item_type")
	checks["switching the type resets slots / adjustable to the type"] = ContentFactory._strings(switched["slots"]) == ItemFamilies.ARMOR_SLOTS and ContentFactory._strings(switched["adjustable"]) == ItemFamilies.number_fields_for("ARMOR")
	return TestHelpers.report(TAG, "New item and family validate; errors block Save with field and row; warnings never do", checks)

# TOOLS-RO: the viewer has no locks (no field state carries one); the data rule (ContentCatalog.key_field_errors) still
# makes a list change of a system item a key-field problem on modifiers, outside the system-edit scope only.
static func _test_key_fields() -> bool:
	var checks: Dictionary = {}
	var system: Dictionary = _item_of_type("WEAPON")
	var id: String = str(system["id"])
	var no_lock: bool = true
	for f: Dictionary in ContentSchema.fields("items"):
		no_lock = no_lock and not ItemFormModel.state("items", system, str(f["key"]), id).has("locked")
	checks["no item field state carries a lock"] = no_lock
	var edited: Dictionary = system.duplicate(true)
	var mods: Array = DictUtil.array_of(edited["modifiers"]).duplicate(true)
	mods.append({"kind": "stat_bonus", "stat": "PERC", "value": 1})
	edited = _with_mods(edited, mods)
	var key_problem: Callable = func(p: Dictionary) -> bool: return str(p["field"]) == "modifiers" and str(p["message"]).contains("key field")
	checks["a list change on a system item is a key-field problem on modifiers"] = ItemFormModel.draft_problems("items", edited, id).any(key_problem)
	ContentCatalog.begin_system_edit()
	checks["...but not inside the system-edit scope"] = not ItemFormModel.draft_problems("items", edited, id).any(key_problem)
	ContentCatalog.end_system_edit()
	return TestHelpers.report(TAG, "Key-field rule on system items (no viewer locks)", checks)

static func _test_modifier_limits() -> bool:
	var checks: Dictionary = {}
	var fresh: Dictionary = ItemFormModel.new_entry("t_if_limits")
	var family: Dictionary = ItemFormModel.family_row(fresh)
	var hand: String = str(fresh["hand"])
	var range_ok: bool = true
	for field: String in ItemFamilies.number_fields_for("WEAPON"):
		var limits: Dictionary = ItemFormModel.delta_limits(fresh, -1, field)
		var rng: Array = ItemFamilies.NUMBER_RANGES[field]
		var base: int = DictUtil.to_int(ItemFamilies.base_of(family, hand)[field])
		var spec: Dictionary = ItemModifiers.KINDS["adjust"]
		range_ok = range_ok and DictUtil.to_int(limits["max"]) == mini(DictUtil.to_int(spec["max"]), DictUtil.to_int(rng[1]) - base) and DictUtil.to_int(limits["min"]) == maxi(DictUtil.to_int(spec["min"]), DictUtil.to_int(rng[0]) - base)
	checks["delta limits keep every resolved number inside its schema range"] = range_ok
	var stat_ok: bool = true
	for kind: String in ["stat_bonus", "requirement"]:
		var st: Dictionary = ItemFormModel.stat_state(fresh, -1, kind)
		var spec_k: Dictionary = ItemModifiers.KINDS[kind]
		stat_ok = stat_ok and DictUtil.to_int(st["min"]) == DictUtil.to_int(spec_k["min"]) and DictUtil.to_int(st["max"]) == DictUtil.to_int(spec_k["max"])
	checks["stat values use the ItemModifiers.KINDS ranges"] = stat_ok
	var defaults_ok: bool = true
	var allowed: Array[String] = ContentFactory._strings(family.get("modifier_kinds", []))
	for kind: String in allowed:
		var d: Dictionary = _with_mods(fresh, ItemFormModel.with_modifier_added(fresh, kind))
		defaults_ok = defaults_ok and not _has_field(ItemFormModel.draft_problems("items", d), "modifiers")
	checks["the default modifier of every allowed kind validates"] = not allowed.is_empty() and defaults_ok
	var kind_state: Dictionary = ItemFormModel.modifier_kind_state(fresh)
	var disabled_ok: bool = true
	for kind: String in ItemModifiers.kind_names():
		disabled_ok = disabled_ok and (DictUtil.dict_of(kind_state["disabled"]).has(kind) != allowed.has(kind))
	checks["kinds the family does not allow are disabled"] = disabled_ok
	var one: Dictionary = _with_mods(fresh, ItemFormModel.with_modifier_added(fresh, "adjust"))
	var used_field: String = str(ItemModifiers.modifiers_of(one)[0]["field"])
	checks["an adjusted number is disabled for the next adjust row"] = DictUtil.dict_of(ItemFormModel.adjust_field_state(one, -1)["disabled"]).has(used_field) and not DictUtil.dict_of(ItemFormModel.adjust_field_state(one, 0)["disabled"]).has(used_field)
	var two: Dictionary = _with_mods(one, ItemFormModel.with_modifier_added(one, "requirement"))
	var moved: Array = ItemFormModel.with_modifier_moved(two, 1, -1)
	checks["move up swaps rows; remove drops one"] = str(DictUtil.dict_of(moved[0])["kind"]) == "requirement" and ItemFormModel.with_modifier_removed(two, 0).size() == 1
	checks["switching a row's kind gives that kind's default"] = str(DictUtil.dict_of(ItemFormModel.with_modifier_kind(two, 0, "stat_bonus")[0])["kind"]) == "stat_bonus"
	var cap: int = DictUtil.to_int(family.get("max_modifiers", 0))
	var full_list: Array = []
	for i: int in range(cap):
		full_list.append({"kind": "requirement", "stat": ContentSchema.STAT_KEYS[i % ContentSchema.STAT_KEYS.size()], "value": 1})
	var full: Dictionary = _with_mods(fresh, full_list)
	var add: Dictionary = ItemFormModel.can_add_modifier(full)
	checks["at the family cap Add is disabled with the reason"] = cap > 0 and not DictUtil.to_bool(add["allowed"]) and str(add["reason"]).begins_with("Full")
	checks["one modifier row per modifier, with its budget value"] = ItemFormModel.modifier_rows(two).size() == 2 and is_equal_approx(DictUtil.to_float(ItemFormModel.modifier_rows(two)[0]["value"]), GearBudget.modifier_value(ItemModifiers.modifiers_of(two)[0]))
	return TestHelpers.report(TAG, "Modifier table: dynamic limits, valid defaults, cap with reason", checks)

static func _test_effects() -> bool:
	var checks: Dictionary = {}
	var types: Array[String] = ItemFormModel.effect_types()
	checks["effect types = the item grammar (Passives.ITEM_TYPES)"] = types.size() == Passives.ITEM_TYPES.size() and types.all(func(t: String) -> bool: return Passives.ITEM_TYPES.has(t))
	var valid: bool = true
	var skill_ok: Callable = func(skill_id: String) -> bool: return ContentCatalog.has("skills", skill_id)
	for t: String in types:
		valid = valid and Passives.validate_effects([ItemFormModel.effect_default(t)], skill_ok, true, true).is_empty()
	checks["the default effect of every item type validates"] = valid
	var choice_type: String = ""
	var choice_keys: Array[String] = []
	for spec: Dictionary in Passives.effect_specs(true):
		var keys: Array[String] = []
		for raw: Variant in DictUtil.array_of(spec["params"]):
			if not str(DictUtil.dict_of(raw).get("choice", "")).is_empty():
				keys.append(str(DictUtil.dict_of(raw)["key"]))
		if keys.size() >= 2:
			choice_type = str(spec["type"])
			choice_keys = keys
			break
	if not choice_type.is_empty():
		var fx: Dictionary = ItemFormModel.effect_default(choice_type)
		var set_count: int = 0
		for k: String in choice_keys:
			set_count += 1 if fx.has(k) else 0
		checks["a choice group starts with exactly one param set"] = set_count == 1
		var other: String = choice_keys[0] if not fx.has(choice_keys[0]) else choice_keys[1]
		var switched: Dictionary = ItemFormModel.effect_with_choice(fx, other)
		var after: int = 0
		for k: String in choice_keys:
			after += 1 if switched.has(k) else 0
		checks["switching the choice clears the sibling"] = after == 1 and switched.has(other)
	var rows_ok: bool = true
	for t: String in types:
		var fx_rows: Array[Dictionary] = ItemFormModel.effect_param_rows(ItemFormModel.effect_default(t))
		for row: Dictionary in fx_rows:
			if str(row["kind"]) == "int" or str(row["kind"]) == "float":
				var spec_p: Dictionary = {}
				for raw: Variant in DictUtil.array_of(Passives.effect_spec(t)["params"]):
					if str(DictUtil.dict_of(raw)["key"]) == str(row["key"]):
						spec_p = DictUtil.dict_of(raw)
				rows_ok = rows_ok and is_equal_approx(DictUtil.to_float(row["min"]), DictUtil.to_float(spec_p["min"])) and is_equal_approx(DictUtil.to_float(row["max"]), DictUtil.to_float(spec_p["max"]))
	checks["effect param widgets use the Passives.effect_specs limits"] = rows_ok
	return TestHelpers.report(TAG, "Effect editor: item grammar, valid defaults, exclusive choices", checks)

static func _test_tiers() -> bool:
	var checks: Dictionary = {}
	var limits_ok: bool = true
	for type: String in ["WEAPON", "ARMOR", "SHIELD"]:
		var keys: Array[String] = ItemModifiers.tier_bonus_keys(type)
		limits_ok = limits_ok and keys.has("power") == (type == "WEAPON") and keys.has("mitigation") == (type != "WEAPON") and keys.has("stat_bonuses") and keys.has("effects")
	checks["bonus keys per type: power on weapons, mitigation on armor and shields"] = limits_ok
	var tested: int = 0
	var defaults_ok: bool = true
	var criteria_ok: bool = true
	var visible_ok: bool = true
	for type: String in ["WEAPON", "ARMOR", "SHIELD"]:
		var item: Dictionary = _item_of_type(type, func(i: Dictionary) -> bool: return _with_tier_room(i))
		if item.is_empty():
			continue
		tested += 1
		var draft: Dictionary = item.duplicate(true)
		draft["origin"] = ContentSchema.ORIGIN_USER   # an editable copy (never stored)
		draft["tiers"] = []
		ItemFormModel.sync(draft, "tiers")
		visible_ok = visible_ok and _keys_of(ItemFormModel.layout("items", draft)).has("tiers")
		var with_tier: Dictionary = draft.duplicate(true)
		with_tier["tiers"] = ItemFormModel.with_tier_added(draft)
		ItemFormModel.sync(with_tier, "tiers")
		defaults_ok = defaults_ok and ItemValidator.tier_problems(with_tier, ItemFormModel.family_row(with_tier)).is_empty()
		for when_type: String in ItemModifiers.WHEN_TYPES:
			var switched: Dictionary = with_tier.duplicate(true)
			switched["tiers"] = ItemFormModel.with_tier_when_type(with_tier, 0, when_type)
			criteria_ok = criteria_ok and ItemValidator.tier_problems(switched, ItemFormModel.family_row(switched)).is_empty()
	checks["weapon, armor and shield tiers are editable through `tiers`"] = tested > 0 and visible_ok
	checks["the default tier validates for each gear type"] = defaults_ok
	checks["every criterion type's default validates"] = criteria_ok
	var weapon: Dictionary = _item_of_type("WEAPON", func(i: Dictionary) -> bool: return _with_tier_room(i))
	weapon["tiers"] = []
	var limits: Dictionary = ItemFormModel.tier_limits(weapon)
	checks["tier limits are the ItemModifiers constants"] = DictUtil.to_int(limits["stat_max"]) == ItemModifiers.TIER_STAT_MAX and DictUtil.to_int(limits["proficiency_max"]) == Proficiency.MAX_LEVEL and DictUtil.to_int(limits["level_max"]) == StatRules.MAX_LEVEL and DictUtil.to_int(limits["max_stats"]) == ItemModifiers.TIER_MAX_STATS
	var full: Dictionary = weapon.duplicate(true)
	full["origin"] = ContentSchema.ORIGIN_USER
	var list: Array = []
	for i: int in range(ItemFormModel.max_tiers(weapon)):
		full["tiers"] = list.duplicate(true)
		list.append(ItemFormModel.default_tier(full))
	full["tiers"] = list
	ItemFormModel.sync(full, "tiers")
	checks["filling to max tiers stays valid (ascending criteria)"] = ItemValidator.tier_problems(full, ItemFormModel.family_row(full)).is_empty()
	checks["at max tiers Add is disabled with the reason"] = not DictUtil.to_bool(ItemFormModel.can_add_tier(full)["allowed"]) and str(ItemFormModel.can_add_tier(full)["reason"]).begins_with("Full")
	checks["tier rows: criterion text, auto description, preview value x W_TIER"] = ItemFormModel.tier_rows(full).size() == list.size() and is_equal_approx(DictUtil.to_float(ItemFormModel.tier_rows(full)[0]["value"]), GearBudget.tier_bonus_value(DictUtil.dict_of(DictUtil.dict_of(list[0])["bonus"])) * GearBudget.W_TIER)
	# Legacy flat rows check threshold tiers for every gear type (not only weapons).
	var flat: Dictionary = ContentSchema.normalize("items", {"id": "t_if_flat_armor", "name": "Flat", "item_type": "ARMOR", "slot": "BODY", "threshold_tiers": [{"bonus_power": 1}]})
	checks["legacy validation checks armor threshold tiers too"] = ItemValidator.item_errors(flat, {}, false).any(func(m: String) -> bool: return m.contains("threshold tier"))
	checks["threshold_tiers is derived and shown for every gear type"] = DictUtil.to_bool(ContentSchema.field("items", "threshold_tiers").get("readonly", false)) and ContentSchema.is_visible(ContentSchema.field("items", "threshold_tiers"), {"item_type": "ARMOR"})
	return TestHelpers.report(TAG, "Tier table: limits, valid defaults per criterion, armor / shield tiers", checks)

static func _test_tokens_and_budget() -> bool:
	var checks: Dictionary = {}
	var palette_ok: bool = true
	var budget_ok: bool = true
	var base_ok: bool = true
	var req_ok: bool = true
	var gear: int = 0
	for item: Dictionary in ContentCatalog.entries("items"):
		if not GearBudget.is_gear(item):
			continue
		gear += 1
		for token: String in ItemFormModel.token_palette(item):
			palette_ok = palette_ok and ItemText.validate("{%s}" % token, item).is_empty()
		var reqs: Dictionary = DictUtil.dict_of(item["required_stats"])
		for stat: String in ContentSchema.STAT_KEYS:
			req_ok = req_ok and ItemFormModel.token_palette(item).has("req:" + stat) == reqs.has(stat)
		var b: Dictionary = ItemFormModel.budget_breakdown(item)
		budget_ok = budget_ok and is_equal_approx(DictUtil.to_float(b["total"]), GearBudget.item_value(item)) and is_equal_approx(DictUtil.to_float(b["item_value"]), GearBudget.item_value(item))
		for row: Dictionary in ItemFormModel.base_rows(item):
			base_ok = base_ok and DictUtil.to_int(row["base"]) + DictUtil.to_int(row["delta"]) == DictUtil.to_int(row["result"]) and DictUtil.to_int(row["result"]) == DictUtil.to_int(item[str(ItemFamilies.FLAT_KEYS[str(row["field"])])])
	checks["every palette token of every gear item validates"] = gear > 0 and palette_ok
	checks["{req:STAT} is offered exactly for the item's requirements"] = req_ok
	checks["budget breakdown: base + modifiers = item_value for every gear item"] = budget_ok
	checks["base grid: family base + adjustments = the resolved number"] = base_ok
	var weapon: Dictionary = _item_of_type("WEAPON")
	weapon["description"] = "Hits for {power}."
	var d: Dictionary = ItemFormModel.description_preview(weapon)
	checks["the rendered description fills the resolved power"] = str(d["text"]) == "Hits for %d." % DictUtil.to_int(weapon["base_power"]) and DictUtil.array_of(d["problems"]).is_empty()
	var chips: Array[Dictionary] = ItemFormModel.chips(weapon)
	checks["build chips: item type, gear token, slot, hand"] = chips.size() >= 4 and str(chips[0]["value"]) == "WEAPON" and str(chips[1]["value"]) == str(weapon["family"])
	var p: Dictionary = ItemFormModel.preview(weapon)
	checks["preview lists the loadouts that use the item"] = JSON.stringify(DictUtil.array_of(p["users"])) == JSON.stringify(ContentCatalog.references_to("items", str(weapon["id"])))
	return TestHelpers.report(TAG, "Token palette, rendered text, base grid and budget breakdown", checks)

static func _test_filters() -> bool:
	var checks: Dictionary = {}
	var all: Array[Dictionary] = ContentCatalog.entries("items")
	checks["default filters keep every item"] = ItemFormModel.filter_items(all, ItemFormModel.default_filters()).size() == all.size()
	var family_id: String = str(all[0]["family"])
	var f: Dictionary = ItemFormModel.default_filters()
	f["family"] = family_id
	var by_family: Array[Dictionary] = ItemFormModel.filter_items(all, f)
	checks["family chip"] = by_family.size() == ContentCatalog.items_of_family(family_id).size() and by_family.all(func(r: Dictionary) -> bool: return str(r["family"]) == family_id)
	var expected_tiers: int = all.filter(func(r: Dictionary) -> bool: return not DictUtil.array_of(r["tiers"]).is_empty()).size()
	f = ItemFormModel.default_filters()
	f["has_tiers"] = "yes"
	checks["has tiers chip"] = ItemFormModel.filter_items(all, f).size() == expected_tiers
	f["has_tiers"] = "no"
	checks["...and its negation"] = ItemFormModel.filter_items(all, f).size() == all.size() - expected_tiers
	f = ItemFormModel.default_filters()
	f["hand"] = ItemFamilies.HAND_OFF
	checks["hand chip"] = ItemFormModel.filter_items(all, f).all(func(r: Dictionary) -> bool: return str(r["hand"]) == ItemFamilies.HAND_OFF)
	f = ItemFormModel.default_filters()
	f["origin"] = ContentSchema.ORIGIN_SYSTEM
	var system_n: int = all.filter(func(r: Dictionary) -> bool: return ContentCatalog.is_system_origin(r)).size()
	checks["origin chip"] = ItemFormModel.filter_items(all, f).size() == system_n
	f = ItemFormModel.default_filters()
	f["used_by"] = "class"
	var by_class: Array[Dictionary] = ItemFormModel.filter_items(all, f)
	checks["used by class chip"] = by_class.all(func(r: Dictionary) -> bool: return ContentCatalog.references_to("items", str(r["id"])).any(func(u: String) -> bool: return u.begins_with("classes:")))
	f["used_by"] = "unused"
	checks["unused chip excludes every class item"] = ItemFormModel.filter_items(all, f).all(func(r: Dictionary) -> bool: return not by_class.has(r))
	var specials: int = all.filter(func(r: Dictionary) -> bool: return DictUtil.to_bool(r.get("special", false))).size()
	f = ItemFormModel.default_filters()
	f["special"] = "yes"
	checks["special chip"] = ItemFormModel.filter_items(all, f).size() == specials
	checks["every chip has choices"] = ["family", "hand", "rarity", "special", "has_tiers", "has_effects", "origin", "used_by"].all(func(k: String) -> bool: return ItemFormModel.filter_options(k).size() > 1)
	return TestHelpers.report(TAG, "Item list filter chips", checks)

static func _test_family_batch() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX_MODEL)
	# The gear family with the most items (rule), its power (weapons) or mitigation (others) base +1 on every hand row.
	var best: Dictionary = {}
	for fam: Dictionary in ContentCatalog.entries("item_families"):
		if ItemFamilies.is_gear_type(ItemFamilies.item_type_of(fam)) and ContentCatalog.items_of_family(str(fam["id"])).size() > (ContentCatalog.items_of_family(str(best.get("id", ""))).size() if not best.is_empty() else 0):
			best = fam
	var fam_id: String = str(best.get("id", ""))
	var number: String = "power" if ItemFamilies.item_type_of(best) == "WEAPON" else "mitigation"
	var draft: Dictionary = best.duplicate(true)
	for key: String in ItemFormModel.grid_keys(draft):
		draft["base"] = ItemFormModel.with_base(draft, key, number, ItemFormModel.base_cell(draft, key, number) + 1)
	var users: Array[Dictionary] = ContentCatalog.items_of_family(fam_id)
	var before: Dictionary = {}
	for item: Dictionary in users:
		before[str(item["id"])] = DictUtil.to_int(item[str(ItemFamilies.FLAT_KEYS[number])])
	var errors: Array[String] = ContentCatalog.upsert_many("item_families", [draft])   # base numbers are free fields of a system family
	var moved: bool = errors.is_empty() and not users.is_empty()
	for item: Dictionary in ContentCatalog.items_of_family(fam_id):
		moved = moved and DictUtil.to_int(item[str(ItemFamilies.FLAT_KEYS[number])]) == DictUtil.to_int(before[str(item["id"])]) + 1
	checks["the batch upsert re-resolves every item of the family"] = moved
	# A user family with a user item: id and type fixed while used.
	var fam: Dictionary = ItemFormModel.new_family(ContentCatalog.unique_id("item_families", "t_if_user_fam"))
	var fam_errors: Array[String] = ContentCatalog.upsert("item_families", fam)
	var item: Dictionary = ContentSchema.defaults("items")
	item["id"] = ContentCatalog.unique_id("items", "t_if_user_item")
	item["family"] = str(fam["id"])
	item["modifiers"] = []
	item["tiers"] = []
	ItemFormModel.sync(item, "family")
	var item_errors: Array[String] = ContentCatalog.upsert("items", ItemFormModel.stored_of(item))
	var fid: String = str(fam["id"])
	checks["a user family and item store"] = fam_errors.is_empty() and item_errors.is_empty()
	var st: Dictionary = ItemFormModel.state("item_families", ContentCatalog.get_entry("item_families", fid), "item_type", fid)
	checks["a used family's item type is fixed with the reason"] = not DictUtil.to_bool(st["enabled"]) and str(st["reason"]).contains("item(s) use this family")
	checks["used by lists the item"] = ItemFormModel.used_by(fid).size() == 1
	TestHelpers.cleanup(SANDBOX_MODEL)
	return TestHelpers.report(TAG, "Family batch upsert, used family state and used-by", checks)

# ------------------------------------------------------------------------------
# Forge viewer item and family tabs (scene; TOOLS-RO: read-only on the shipped catalog)
# ------------------------------------------------------------------------------

static func _test_forge_ui(tree_parent: Node) -> bool:
	if tree_parent == null:
		return TestHelpers.report_ok(TAG, "Forge viewer item tabs (needs a scene tree)", true, "skipped")
	var checks: Dictionary = {}
	var host: Node = (load("res://scenes/content_forge.tscn") as PackedScene).instantiate()
	tree_parent.add_child(host)
	var forge: ContentForge = host.get("forge")
	forge.show_kind("items")
	var shipped: Dictionary = _item_of_type("WEAPON", func(i: Dictionary) -> bool: return not ItemModifiers.modifiers_of(i).is_empty())
	var id: String = str(shipped["id"])
	forge.select(id)
	var table: Control = forge.form.field_control("modifiers")
	var rows: Node = table.find_child("Rows", true, false) if table != null else null
	checks["the modifier table shows one row per modifier"] = rows != null and rows.get_child_count() == ItemModifiers.modifiers_of(shipped).size()
	var first_row: Label = rows.get_child(0) as Label if rows != null and rows.get_child_count() > 0 else null
	var first_mod: Dictionary = ItemFormModel.modifier_rows(forge.form.draft, [])[0] if not ItemModifiers.modifiers_of(shipped).is_empty() else {}
	var add_row: Control = table.find_child("AddRow", true, false) as Control if table != null else null
	checks["each modifier row shows its summary text and the table has no add row"] = first_row != null and first_row.text.begins_with(str(first_mod.get("text", "?"))) and add_row != null and not add_row.visible
	var family_lbl: Label = forge.form.field_control("family") as Label
	var name_lbl: Label = forge.form.field_control("name") as Label
	checks["the family and the name show as plain labels"] = family_lbl != null and family_lbl.text.contains(str(shipped["family"])) and name_lbl != null and name_lbl.text == str(shipped["name"])
	var base_panel: RichTextLabel = forge.form.panel_control("base") as RichTextLabel
	checks["the base values panel shows base + adjustments = result"] = base_panel != null and base_panel.text.contains(" = ")
	checks["the preview panel shows the gear value"] = forge.form.panel_control("preview") != null and (forge.form.panel_control("preview") as RichTextLabel).text.contains("Gear value")
	checks["a shipped item has no problems"] = forge.entry_problems().is_empty()
	var live: Array[String] = TestHelpers.live_controls(forge.form)
	checks["no control of the item form can edit or act (%s)" % ", ".join(live)] = live.is_empty()
	# Filter chip: has tiers.
	forge.set_item_filter("has_tiers", "yes")
	var tiered: int = ContentCatalog.entries("items").filter(func(r: Dictionary) -> bool: return not DictUtil.array_of(r["tiers"]).is_empty()).size()
	checks["the has-tiers chip filters the list"] = forge.list_ids().size() == tiered
	forge.set_item_filter("has_tiers", "any")
	checks["the any chip lists every item"] = forge.list_ids().size() == ContentCatalog.entries("items").size()
	# Families: the base grid and the used-by panel (the impact panel left with the edits).
	forge.show_kind("item_families")
	var fam_id: String = ""
	for fam: Dictionary in ContentCatalog.entries("item_families"):
		if ItemFamilies.is_gear_type(ItemFamilies.item_type_of(fam)) and not ContentCatalog.items_of_family(str(fam["id"])).is_empty():
			fam_id = str(fam["id"])
			break
	forge.select(fam_id)
	var used: RichTextLabel = forge.form.panel_control("used_by") as RichTextLabel
	checks["the family form shows the base grid and who uses it, no impact panel"] = forge.form.field_control("base") != null and used != null and used.text.begins_with("%d item" % ContentCatalog.items_of_family(fam_id).size()) and forge.form.panel_control("impact") == null
	var fam_live: Array[String] = TestHelpers.live_controls(forge.form)
	checks["no control of the family form can edit or act (%s)" % ", ".join(fam_live)] = fam_live.is_empty()
	host.queue_free()
	return TestHelpers.report(TAG, "Forge viewer item and family tabs (scene)", checks)
