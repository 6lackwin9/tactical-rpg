---
class: ItemModifiersTestSuite
extends: RefCounted
description: "Tactical RPG - Item modifiers suite (FORGE1_ITEMS 2, 3, 4 validation, 8 acceptance; ITEM1-L commit 1) On a sandbox fork with item families (TestHelpers.fork_migrated): the migration round trip over every shipped item, the budget breakdown, inheritance (a family base change moves only its inheriting items), modifier and tier rules, description tokens, both upsert shapes (stored and legacy flat) and the system-item lock. Fixtures are test-only ids on families picked by rule; shipped items are only read, except the key-lock check (refused, nothing changes)."
---

# Properties
* `TAG`: String = "ItemModifiers" (const)

# Methods

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING ITEM MODIFIERS TEST (ITEM1-L)           ===")
print("=======================================================")
var report: Dictionary = TestHelpers.fork_migrated(SANDBOX)
var ok: bool = _test_migration_report(report)
if not ContentCatalog.items_inherit():
	TestHelpers.cleanup(SANDBOX)
	return false
ok = _test_round_trip() and ok
ok = _test_budget_breakdown() and ok
ok = _test_inheritance() and ok
ok = _test_modifier_rules() and ok
ok = _test_tier_rules() and ok
ok = _test_tokens() and ok
ok = _test_upsert_shapes() and ok
TestHelpers.cleanup(SANDBOX)
return ok
```

### `static _weapon_family() -> Dictionary`
```logic
for f: Dictionary in ContentCatalog.entries("item_families"):
	var kinds: Array[String] = ContentFactory._strings(f.get("modifier_kinds", []))
	if ItemFamilies.item_type_of(f) == "WEAPON" and ItemFamilies.category(f) == "main" and kinds.size() == ItemModifiers.KINDS.size() \
			and ContentFactory._strings(f.get("adjustable", [])).has("power") and DictUtil.to_int(f.get("max_tiers", 0)) > 0:
		return f
return {}
```

### `static _family_of_type(item_type: String) -> Dictionary`
```logic
for f: Dictionary in ContentCatalog.entries("item_families"):
	if ItemFamilies.item_type_of(f) == item_type:
		return f
return {}
```

### `static _stored(id: String, family: Dictionary, extra: Dictionary = {}) -> Dictionary`
```logic
var e: Dictionary = {"id": id, "name": "T " + id, "family": str(family["id"]), "modifiers": [], "tiers": []}
var hands: Array[String] = ItemFamilies.hands_of(family)
if not hands.is_empty():
	e["hand"] = ItemFamilies.default_hand(family)
var slots: Array[String] = ContentFactory._strings(family.get("slots", []))
if not slots.is_empty():
	e["slot"] = slots[0]
e.merge(extra, true)
return e
```

### `static _req(stat: String, value: int) -> Dictionary`
```logic
return {"kind": ItemModifiers.REQUIREMENT, "stat": stat, "value": value}
```

### `static _tier(when: Dictionary, bonus: Dictionary, label: String = "T") -> Dictionary`
```logic
return {"label": label, "when": when, "bonus": bonus, "desc": ""}
```

### `static _test_migration_report(report: Dictionary) -> bool`
```logic
if report.is_empty():
	return TestHelpers.report_ok(TAG, "Migration seed", ContentCatalog.items_inherit(), "shipped data already uses families (seed not run)")
var checks: Dictionary = {}
checks["no upsert errors"] = DictUtil.array_of(report.get("upsert_errors", ["?"])).is_empty()
checks["every engine key unchanged except the accepted off-hand category changes"] = DictUtil.array_of(report.get("engine_diffs", ["?"])).is_empty()
checks["item_value identical for every item"] = DictUtil.array_of(report.get("value_diffs", ["?"])).is_empty()
checks["every class and unit loadout total identical"] = DictUtil.array_of(report.get("loadout_diffs", ["?"])).is_empty()
checks["validate_all empty after the migration"] = DictUtil.dict_of(report.get("validate_all", {"?": 1})).is_empty()
checks["every item migrated"] = DictUtil.to_int(report.get("items", -1)) == ContentCatalog.entries("items").size()
var created: Array = DictUtil.array_of(report.get("families_created", []))
var detail: String = "%d items, %d families created, retyped %s" % [DictUtil.to_int(report.get("items", 0)), created.size(), str(report.get("retyped", []))]
if not TestHelpers.failed(checks).is_empty():
	detail = str(report)
return TestHelpers.report_ok(TAG, "Migration seed on the fork", TestHelpers.failed(checks).is_empty(), detail if TestHelpers.failed(checks).is_empty() else TestHelpers.failed(checks) + " | " + detail)
```

### `static _test_round_trip() -> bool`
```logic
var checks: Dictionary = {}
var round_trip: bool = true
var stored_trip: bool = true
var bad: Array[String] = []
for row: Dictionary in ContentCatalog.entries("items"):
	var family: Dictionary = ContentCatalog.family_of(row)
	var again: Dictionary = ContentSchema.normalize("items", ItemModifiers.resolve(ItemModifiers.collapse(row, family), family))
	if not ItemModifiers.engine_equal(again, row):
		round_trip = false
		bad.append(str(row["id"]))
	var from_stored: Dictionary = ContentSchema.normalize("items", ItemModifiers.resolve(ItemModifiers.stored(row), family))
	stored_trip = stored_trip and ItemModifiers.engine_equal(from_stored, row) and ItemModifiers.same_value(from_stored["modifiers"], row["modifiers"])
checks["resolve(collapse(row)) == row on every engine key, every item"] = round_trip
checks["resolve(stored(row)) == row (the version 2 form is lossless)"] = stored_trip
var tiers_ok: bool = true
for row: Dictionary in ContentCatalog.entries("items"):
	for t: Dictionary in ItemModifiers.tiers_of(row):
		tiers_ok = tiers_ok and ItemModifiers.label_of_desc(ItemText.tier_desc(t)) == str(t["label"])
checks["tier labels and auto descriptions agree (legacy desc reproduced)"] = tiers_ok
return TestHelpers.report(TAG, "Round trip over every item%s" % ("" if bad.is_empty() else " (" + ", ".join(bad) + ")"), checks)
```

### `static _test_budget_breakdown() -> bool`
```logic
var checks: Dictionary = {}
var sums_ok: bool = true
var bad: Array[String] = []
for row: Dictionary in ContentCatalog.entries("items"):
	if not GearBudget.is_gear(row):
		continue
	var family: Dictionary = ContentCatalog.family_of(row)
	var total: float = GearBudget.base_value(family, str(row["hand"]))
	for mod: Dictionary in ItemModifiers.modifiers_of(row):
		total += GearBudget.modifier_value(mod)
	if not is_equal_approx(total, GearBudget.item_value(row)):
		sums_ok = false
		bad.append(str(row["id"]))
checks["base value + per-modifier values == item_value for every gear item"] = sums_ok
checks["adjust weights follow GearBudget (power, mitigation, burden, block cost, range 0)"] = (
	is_equal_approx(GearBudget.modifier_value({"kind": "adjust", "field": "power", "delta": 3}), 3 * GearBudget.W_POWER)
	and is_equal_approx(GearBudget.modifier_value({"kind": "adjust", "field": "mitigation", "delta": 2}), 2 * GearBudget.W_MITIGATION)
	and is_equal_approx(GearBudget.modifier_value({"kind": "adjust", "field": "burden", "delta": -2}), -2 * GearBudget.W_BURDEN)
	and is_equal_approx(GearBudget.modifier_value({"kind": "adjust", "field": "block_cost", "delta": 1}), GearBudget.W_EXTRA_BLOCK)
	and is_zero_approx(GearBudget.modifier_value({"kind": "adjust", "field": "range", "delta": 2})))
checks["stat bonuses count, requirements are free"] = is_equal_approx(GearBudget.modifier_value({"kind": "stat_bonus", "stat": "STR", "value": 2}), 2 * GearBudget.W_STAT_POINT) \
	and is_zero_approx(GearBudget.modifier_value(_req("STR", 14)))
checks["effects use the item-effect weights"] = is_equal_approx(GearBudget.modifier_value({"kind": "effect", "effect": {"type": "spell_reach", "value": 1}}), GearBudget.W_SPELL_REACH)
return TestHelpers.report(TAG, "Budget breakdown sums to item_value%s" % ("" if bad.is_empty() else " (" + ", ".join(bad) + ")"), checks)
```

### `static _test_inheritance() -> bool`
```logic
var checks: Dictionary = {}
var family: Dictionary = _weapon_family()
var id: String = str(family["id"])
var hand: String = ItemFamilies.default_hand(family)
var before: Dictionary = {}
for row: Dictionary in ContentCatalog.entries("items"):
	before[str(row["id"])] = row
var edited: Dictionary = family.duplicate(true)
var base_row: Dictionary = DictUtil.dict_of(DictUtil.dict_of(edited["base"])[hand])
base_row["power"] = DictUtil.to_int(base_row["power"]) + 2
checks["a base edit is a free field (no seed scope)"] = ContentCatalog.upsert_many("item_families", [edited]).is_empty()
var moved_ok: bool = true
var others_ok: bool = true
var moved: int = 0
for row: Dictionary in ContentCatalog.entries("items"):
	var old: Dictionary = DictUtil.dict_of(before[str(row["id"])])
	if str(row["family"]) == id and str(row["hand"]) == hand:
		moved_ok = moved_ok and DictUtil.to_int(row["base_power"]) == DictUtil.to_int(old["base_power"]) + 2 and ItemModifiers.same_value(row["modifiers"], old["modifiers"])
		moved += 1
	else:
		others_ok = others_ok and ItemModifiers.engine_equal(row, old)
checks["every inheriting item gains the +2 (modifiers unchanged)"] = moved_ok and moved > 0
checks["items of other families and other hands are unchanged"] = others_ok
ContentCatalog.upsert_many("item_families", [family])
var restored: bool = true
for row: Dictionary in ContentCatalog.entries("items"):
	restored = restored and ItemModifiers.engine_equal(row, DictUtil.dict_of(before[str(row["id"])]))
checks["restoring the base restores every item"] = restored
var breaking: Dictionary = family.duplicate(true)
DictUtil.dict_of(DictUtil.dict_of(breaking["base"])[hand])["power"] = 0
var heavy: Dictionary = _stored("t_im_heavy_neg", family, {"modifiers": [{"kind": "adjust", "field": "power", "delta": -DictUtil.to_int(ItemFamilies.base_of(family, hand)["power"])}]})
checks["a user item at power 0 by its delta validates"] = ContentCatalog.upsert("items", heavy).is_empty()
var refused: Array[String] = ContentCatalog.upsert_many("item_families", [breaking])
checks["a base edit that pushes an item outside the schema range is refused with the item id"] = TestHelpers.any_contains(refused, "t_im_heavy_neg")
checks["the user fixture is removable"] = ContentCatalog.remove("items", "t_im_heavy_neg").is_empty()
return TestHelpers.report(TAG, "Inheritance: base edits move only inheriting items (%d moved, family %s)" % [moved, id], checks)
```

### `static _test_modifier_rules() -> bool`
```logic
var checks: Dictionary = {}
var weapon: Dictionary = _weapon_family()
var accessory: Dictionary = _family_of_type("ACCESSORY")
checks["a plain stored item on the family validates"] = _errors(_stored("t_im_ok", weapon, {"modifiers": [_req("STR", 10)]})).is_empty()
checks["unknown kind"] = TestHelpers.any_contains(_errors(_stored("t_im_k", weapon, {"modifiers": [{"kind": "t_kind"}]})), "unknown kind")
var sample: Dictionary = {"adjust": {"kind": "adjust", "field": "burden", "delta": 1}, "stat_bonus": {"kind": "stat_bonus", "stat": "STR", "value": 1},
	"requirement": _req("STR", 10), "effect": {"kind": "effect", "effect": {"type": "status_potency", "value": 5}}}
var missing_kind: String = ""
for k: String in ItemModifiers.kind_names():
	if not ContentFactory._strings(accessory.get("modifier_kinds", [])).has(k):
		missing_kind = k
checks["kind not allowed by the family"] = not missing_kind.is_empty() and TestHelpers.any_contains(_errors(_stored("t_im_acc", accessory, {"modifiers": [sample[missing_kind]]})), "not allowed by family")
var not_adjustable: String = ""
for field: String in ItemFamilies.number_fields_for("WEAPON"):
	if not ContentFactory._strings(weapon.get("adjustable", [])).has(field):
		not_adjustable = field
checks["a number the family does not let adjust"] = not not_adjustable.is_empty() and TestHelpers.any_contains(_errors(_stored("t_im_adj", weapon, {"modifiers": [{"kind": "adjust", "field": not_adjustable, "delta": 1}]})), "does not allow adjusting")
checks["the same stat twice"] = TestHelpers.any_contains(_errors(_stored("t_im_dup", weapon, {"modifiers": [_req("STR", 10), _req("STR", 12)]})), "repeats")
checks["a stat bonus above 8"] = TestHelpers.any_contains(_errors(_stored("t_im_big", weapon, {"modifiers": [{"kind": "stat_bonus", "stat": "STR", "value": 9}]})), "outside 1 to 8")
checks["a negative stat bonus"] = TestHelpers.any_contains(_errors(_stored("t_im_neg", weapon, {"modifiers": [{"kind": "stat_bonus", "stat": "STR", "value": -1}]})), "outside 1 to 8")
checks["an adjust of 0"] = TestHelpers.any_contains(_errors(_stored("t_im_zero", weapon, {"modifiers": [{"kind": "adjust", "field": "power", "delta": 0}]})), "changes nothing")
var cap: int = DictUtil.to_int(weapon["max_modifiers"])
var many: Array = []
for stat: String in ContentSchema.STAT_KEYS:
	many.append(_req(stat, 10))
for stat: String in ContentSchema.STAT_KEYS:
	many.append({"kind": "stat_bonus", "stat": stat, "value": 1})
checks["more modifiers than the family allows (%d)" % cap] = many.size() > cap and TestHelpers.any_contains(_errors(_stored("t_im_many", weapon, {"modifiers": many})), "At most %d modifiers" % cap)
var base_power: int = DictUtil.to_int(ItemFamilies.base_of(weapon, ItemFamilies.default_hand(weapon))["power"])
checks["a resolved number outside the schema range"] = TestHelpers.any_contains(_errors(_stored("t_im_low", weapon, {"modifiers": [{"kind": "adjust", "field": "power", "delta": -(base_power + 1)}]})), "resolves to -1")
checks["effects follow the item grammar"] = TestHelpers.any_contains(_errors(_stored("t_im_fx", weapon, {"modifiers": [{"kind": "effect", "effect": {"type": "ap_regen", "value": 1}}]})), "Modifier 1")
checks["unknown modifier keys are refused (nothing is dropped silently)"] = TestHelpers.any_contains(_errors(_stored("t_im_key", weapon, {"modifiers": [{"kind": "requirement", "stat": "STR", "value": 10, "extra": 1}]})), "unknown key")
var no_hand: String = ""
for h: String in ItemFamilies.MAIN_HANDS:
	if not ItemFamilies.hands_of(weapon).has(h):
		no_hand = h
no_hand = no_hand if not no_hand.is_empty() else ItemFamilies.HAND_OFF
checks["a hand the family does not have"] = TestHelpers.any_contains(_errors(_stored("t_im_hand", weapon, {"hand": no_hand})), "not allowed by family")
var no_scale: String = ""
for s: String in ItemFamilies.SCALING_STATS:
	if not ContentFactory._strings(weapon.get("scaling_stats", [])).has(s):
		no_scale = s
checks["a scaling stat outside the family list"] = not no_scale.is_empty() and TestHelpers.any_contains(_errors(_stored("t_im_scale", weapon, {"scaling_stat": no_scale})), "not in family")
var armor: Dictionary = _family_of_type("ARMOR")
var no_slot: String = ""
for s: String in ["ACCESSORY_1", "MAIN_HAND"]:
	if not ContentFactory._strings(armor.get("slots", [])).has(s):
		no_slot = s
		break
checks["an armor slot outside the family slots"] = TestHelpers.any_contains(_errors(_stored("t_im_slot", armor, {"slot": no_slot})), "slots")
checks["a stored item without a family is refused once families exist"] = TestHelpers.any_contains(_errors({"id": "t_im_nofam", "name": "N", "item_type": "WEAPON"}), "Family is required")
return TestHelpers.report(TAG, "Modifier rules (kinds, caps, uniqueness, ranges, hands)", checks)
```

### `static _test_tier_rules() -> bool`
```logic
var checks: Dictionary = {}
var weapon: Dictionary = _weapon_family()
var armor: Dictionary = _family_of_type("ARMOR")
var stat: Dictionary = {"type": "stat_at_least", "stat": "STR", "value": 16}
checks["a stat tier with a power bonus validates and resolves to the legacy row"] = _errors(_stored("t_it_ok", weapon, {"tiers": [_tier(stat, {"power": 3}, "Edge")]})).is_empty() \
	and ItemModifiers.legacy_tiers([_tier(stat, {"power": 3}, "Edge")])[0] == {"stat": "STR", "threshold": 16, "bonus_power": 3, "desc": "Edge: +3 Power"}
checks["proficiency and unit-level criteria validate"] = _errors(_stored("t_it_crit", weapon, {"tiers": [
	_tier({"type": "proficiency_at_least", "gear": str(weapon["id"]), "level": 2}, {"stat_bonuses": {"STR": 1}}),
	_tier({"type": "unit_level_at_least", "value": 5}, {"effects": [{"type": "status_potency", "value": 5}]})]})).is_empty()
checks["an unknown criterion type"] = TestHelpers.any_contains(_errors(_stored("t_it_type", weapon, {"tiers": [_tier({"type": "t_when"}, {"power": 1})]})), "criterion type")
checks["a power bonus on armor"] = TestHelpers.any_contains(_errors(_stored("t_it_arm", armor, {"tiers": [_tier(stat, {"power": 2})]})), "weapons only")
checks["a tier with no bonus"] = TestHelpers.any_contains(_errors(_stored("t_it_none", weapon, {"tiers": [_tier(stat, {"power": 0})]})), "non-zero bonus")
checks["the same criterion twice"] = TestHelpers.any_contains(_errors(_stored("t_it_dup", weapon, {"tiers": [_tier(stat, {"power": 1}), _tier(stat, {"power": 2})]})), "repeats the criterion")
var lower: Dictionary = {"type": "stat_at_least", "stat": "STR", "value": 12}
checks["tiers on one stat go up"] = TestHelpers.any_contains(_errors(_stored("t_it_asc", weapon, {"tiers": [_tier(stat, {"power": 1}), _tier(lower, {"power": 2})]})), "ascending")
var cap: int = DictUtil.to_int(weapon["max_tiers"])
var many: Array = []
for i: int in range(cap + 1):
	many.append(_tier({"type": "stat_at_least", "stat": "STR", "value": 10 + i}, {"power": 1}))
checks["more tiers than the family allows (%d)" % cap] = TestHelpers.any_contains(_errors(_stored("t_it_many", weapon, {"tiers": many})), "At most %d tiers" % cap)
checks["more than 3 tier stat bonuses"] = TestHelpers.any_contains(_errors(_stored("t_it_stats", weapon, {"tiers": [_tier(stat, {"stat_bonuses": {"STR": 1, "DEX": 1, "INT": 1, "CONS": 1}})]})), "at most 3")
return TestHelpers.report(TAG, "Tier rules (criteria, bonuses, counts, order)", checks)
```

### `static _test_tokens() -> bool`
```logic
var checks: Dictionary = {}
var weapon: Dictionary = _weapon_family()
var entry: Dictionary = _stored("t_tok_ok", weapon, {"description": "{name}: {power} power, needs {req:STR} STR; {tier1.power} more at {tier1.value}.",
	"modifiers": [_req("STR", 11)], "tiers": [_tier({"type": "stat_at_least", "stat": "STR", "value": 17}, {"power": 3}, "Edge")]})
checks["a token template validates"] = _errors(entry).is_empty()
checks["the stored row is upserted"] = ContentCatalog.upsert("items", entry).is_empty()
var row: Dictionary = ContentCatalog.get_entry("items", "t_tok_ok")
var built: Item = ContentFactory.item(row)
var expected: String = "%s: %d power, needs 11 STR; 3 more at 17." % [str(row["name"]), DictUtil.to_int(row["base_power"])]
checks["the engine item renders the template with the resolved values"] = built.description == expected
checks["the catalog row keeps the template"] = str(row["description"]).contains("{power}")
checks["an unknown token lists the valid ones"] = TestHelpers.any_contains(_errors(_stored("t_tok_bad", weapon, {"description": "{speed}"})), "Valid for WEAPON")
checks["a token with no value for this item"] = TestHelpers.any_contains(_errors(_stored("t_tok_none", weapon, {"description": "Needs {req:DEX}"})), "no value")
checks["a power token on armor does not apply"] = TestHelpers.any_contains(_errors(_stored("t_tok_arm", _family_of_type("ARMOR"), {"description": "{power}"})), "does not apply")
var literal: Dictionary = row.duplicate(true)
literal["description"] = "{name} hits for %d." % DictUtil.to_int(row["base_power"])
checks["a literal number equal to a resolved field warns (soft) and suggests the token"] = TestHelpers.any_contains(ContentCatalog.warnings_for("items", literal), "{power}")
checks["legacy descriptions without tokens never warn about numbers"] = ItemText.warnings("Gains +4 Power at 18 STR.", row).is_empty()
checks["the summary line lists bonuses and requirements"] = ItemText.modifier_summary({"stat_bonuses": {"STR": 2, "CONS": 2}, "required_stats": {"STR": 14}}) == "+2 STR, +2 CONS; Req STR 14"
checks["the user fixture is removable"] = ContentCatalog.remove("items", "t_tok_ok").is_empty()
return TestHelpers.report(TAG, "Description tokens and derived text", checks)
```

### `static _test_upsert_shapes() -> bool`
```logic
var checks: Dictionary = {}
var weapon: Dictionary = _weapon_family()
var hand: String = ItemFamilies.default_hand(weapon)
var base: Dictionary = ItemFamilies.base_of(weapon, hand)
var flat: Dictionary = {"id": "t_us_flat", "name": "Flat", "item_type": "WEAPON", "weapon_type": str(weapon["id"]), "slot": "MAIN_HAND",
	"two_handed": hand == ItemFamilies.HAND_MAIN_2H, "base_power": DictUtil.to_int(base["power"]) + 5, "base_range": DictUtil.to_int(base["range"]),
	"scaling_stat": ContentFactory._strings(weapon["scaling_stats"])[0], "required_stats": {"STR": 9}}
checks["a legacy flat row upserts"] = ContentCatalog.upsert("items", flat).is_empty()
var row: Dictionary = ContentCatalog.get_entry("items", "t_us_flat")
var mods: Array[Dictionary] = ItemModifiers.modifiers_of(row)
checks["it collapses to its family, hand and deltas"] = str(row["family"]) == str(weapon["id"]) and str(row["hand"]) == hand \
	and mods.size() == 2 and str(mods[0]["kind"]) == "adjust" and DictUtil.to_int(mods[0]["delta"]) == 5 and str(mods[1]["kind"]) == "requirement"
checks["the resolved numbers are the flat ones"] = DictUtil.to_int(row["base_power"]) == DictUtil.to_int(flat["base_power"]) and DictUtil.dict_of(row["required_stats"]).get("STR", 0) == 9
checks["a new row is a user asset"] = str(row["origin"]) == ContentSchema.ORIGIN_USER
var copy: Dictionary = row.duplicate(true)
copy["base_power"] = DictUtil.to_int(row["base_power"]) + 1
ContentCatalog.upsert("items", copy)
checks["a get_entry copy with a new number recomputes the delta (flat wins)"] = DictUtil.to_int(ItemModifiers.modifiers_of(ContentCatalog.get_entry("items", "t_us_flat"))[0]["delta"]) == 6
var listed: Dictionary = ContentCatalog.get_entry("items", "t_us_flat")
var new_mods: Array = DictUtil.array_of(listed["modifiers"]).duplicate(true)
new_mods.append({"kind": "stat_bonus", "stat": "DEX", "value": 1})
listed["modifiers"] = new_mods
ContentCatalog.upsert("items", listed)
checks["a get_entry copy with new modifiers only keeps them (modifiers win)"] = DictUtil.dict_of(ContentCatalog.get_entry("items", "t_us_flat")["stat_bonuses"]).get("DEX", 0) == 1
var mixed: Dictionary = ContentCatalog.get_entry("items", "t_us_flat")
var mixed_mods: Array = DictUtil.array_of(mixed["modifiers"]).duplicate(true)
mixed_mods.append({"kind": "stat_bonus", "stat": "CONS", "value": 1})
mixed["modifiers"] = mixed_mods
mixed["base_power"] = DictUtil.to_int(mixed["base_power"]) + 1
var mixed_before: Dictionary = ContentCatalog.get_entry("items", "t_us_flat")
checks["changing modifiers AND a flat number in one upsert is refused"] = TestHelpers.any_contains(ContentCatalog.upsert("items", mixed), "not both") \
	and TestHelpers.any_contains(ContentCatalog.upsert_many("items", [mixed]), "not both") \
	and ItemModifiers.same_value(ContentCatalog.get_entry("items", "t_us_flat"), mixed_before)
checks["the user item is removable"] = ContentCatalog.remove("items", "t_us_flat").is_empty()
# Modifier order is stable: a flat number edit keeps the stored (non-canonical) order of the list.
var ordered: Dictionary = _stored("t_us_order", weapon, {"modifiers": [_req("STR", 9), {"kind": "adjust", "field": "power", "delta": 2}]})
var order_ok: bool = ContentCatalog.upsert("items", ordered).is_empty()
var bumped: Dictionary = ContentCatalog.get_entry("items", "t_us_order")
bumped["base_power"] = DictUtil.to_int(bumped["base_power"]) + 1
order_ok = order_ok and ContentCatalog.upsert("items", bumped).is_empty()
var after_mods: Array[Dictionary] = ItemModifiers.modifiers_of(ContentCatalog.get_entry("items", "t_us_order"))
checks["a flat number edit keeps the modifier order (no re-sorting)"] = order_ok and after_mods.size() == 2 and str(after_mods[0]["kind"]) == "requirement" \
	and str(after_mods[1]["kind"]) == "adjust" and DictUtil.to_int(after_mods[1]["delta"]) == 3
checks["the ordered fixture is removable"] = ContentCatalog.remove("items", "t_us_order").is_empty()
# The system lock compares whole fields: adding a modifier to a shipped item is refused (user answer 2026-10-17).
var shipped: Dictionary = {}
for e: Dictionary in ContentCatalog.entries("items"):
	if ContentCatalog.is_system_origin(e) and GearBudget.is_gear(e):
		shipped = e
		break
var locked: Dictionary = shipped.duplicate(true)
var locked_mods: Array = DictUtil.array_of(locked["modifiers"]).duplicate(true)
locked_mods.append({"kind": "requirement", "stat": "PERC", "value": 5})
locked["modifiers"] = locked_mods
checks["adding a modifier to a system item is refused (whole-list key lock)"] = TestHelpers.any_contains(ContentCatalog.upsert("items", locked), "key field")
var renamed: Dictionary = shipped.duplicate(true)
renamed["name"] = "Renamed"
checks["a free field of a system item can change"] = ContentCatalog.upsert("items", renamed).is_empty()
checks["the system item keeps its modifiers"] = ItemModifiers.same_value(ContentCatalog.get_entry("items", str(shipped["id"]))["modifiers"], shipped["modifiers"])
ContentCatalog.upsert("items", shipped)
checks["a system item is never removed"] = not ContentCatalog.remove("items", str(shipped["id"])).is_empty()
return TestHelpers.report(TAG, "Upsert shapes and the system-item lock", checks)
```

