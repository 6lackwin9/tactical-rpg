extends SceneTree
# ITEM1 migration seed (ITEM1-L commit 1 writes and tests it; the asset-builder RUNS it in ITEM1-A).
# Idempotent, inside the system-edit scope: derives the 20 item families from the shipped flat items by the mechanical
# rules of docs/specs/FORGE1_ITEMS_MATRIX.md section 1 (lower medians, bands, hands, scaling), upserts the missing ones
# (all origin system), then re-upserts every item so it collapses to family + modifiers (items.json version 2 on save).
# Verifies the acceptance of FORGE1_ITEMS 8: every resolved row equals the pre-migration row on all engine keys except
# the strict-hand retype (shortsword, knuckles, dagger: off-hand category "") and the kite / buckler category "shield";
# item_value identical; every class and unit loadout total identical; validate_all empty.
# Run: tools/fc.sh run data/seeds/item1_migrate.gd   (saves only when the report has no problems)

# Hands the user confirmed beyond the shipped data (DECISIONS items-gear "FORGE1 item answers"): staff also 1H.
const EXTRA_HANDS: Dictionary = {"staff": ["main_1h"]}
# Families without shipped items: crossbow copies bow; material and quest are placeholders.
const COPY_FAMILIES: Dictionary = {"crossbow": "bow"}
const NON_GEAR_FAMILIES: Dictionary = {"accessory": "ACCESSORY", "consumable": "CONSUMABLE", "material": "MATERIAL", "quest": "QUEST"}
const WEAPON_ADJUSTABLE: Array[String] = ["power", "range", "block_cost", "burden"]
const PROTECTIVE_ADJUSTABLE: Array[String] = ["mitigation", "burden", "block_cost"]
const ACCESSORY_ADJUSTABLE: Array[String] = ["burden"]
# Matrix section 2: common to all weapon families (overrides rule 2 for these two numbers).
const WEAPON_COMMON_BANDS: Dictionary = {"block_cost": {"min": 1, "max": 3}, "burden": {"min": 0, "max": 4}}
# Matrix section 2 table values the planner kept over rule 2 (2026-10-17): bow (and the crossbow copy) range 2-6.
const BAND_OVERRIDES: Dictionary = {"bow": {"range": {"min": 2, "max": 6}}}

func _init() -> void:
	ContentCatalog.reload_from_disk()
	var report: Dictionary = migrate()
	for key: Variant in report:
		print("%s: %s" % [str(key), str(report[key])])
	if DictUtil.to_bool(report["ok"]):
		print("save_all: ", ContentCatalog.save_all())
	else:
		print("NOT SAVED: the report lists problems")
	quit()

# Migrates the CURRENT catalog root (the shipped catalog from _init, a sandbox fork in tests). Returns the report:
# items, families_created, families_existing, retyped, upsert_errors, engine_diffs, value_diffs, loadout_diffs, validate_all, ok.
static func migrate() -> Dictionary:
	var before: Array[Dictionary] = []
	for row: Dictionary in ContentCatalog.entries("items"):
		before.append(row.duplicate(true))
	var totals_before: Dictionary = _loadout_totals()
	var report: Dictionary = {"items": before.size(), "families_created": [], "families_existing": [], "retyped": [],
		"upsert_errors": [], "engine_diffs": [], "value_diffs": [], "loadout_diffs": []}
	var created: Array = []
	for f: Dictionary in derive_families(before):
		var id: String = str(f["id"])
		if ContentCatalog.has("item_families", id):
			var existing: Array = report["families_existing"]
			existing.append(id)
		else:
			created.append(f)
			var made: Array = report["families_created"]
			made.append(id)
	ContentCatalog.begin_system_edit()
	var errors: Array = report["upsert_errors"]
	if not created.is_empty():
		errors.append_array(ContentCatalog.upsert_many("item_families", created))
	var retyped: Array = report["retyped"]
	for row: Dictionary in before:
		var entry: Dictionary = row.duplicate(true)
		var family: Dictionary = ContentCatalog.get_entry("item_families", ItemFamilies.family_id_for_row(row))
		if str(row.get("family", "")).is_empty() and _is_retyped(row, family):
			retyped.append(str(row["id"]))
			var category: String = str(row.get("off_hand_category", ""))
			if str(row.get("icon_id", "")).is_empty() and ItemArt.TYPE_ICON_IDS.has(category):
				entry["icon_id"] = category   # keeps the shortsword icon once the legacy category is gone
		for message: String in ContentCatalog.upsert("items", entry):
			errors.append("%s: %s" % [str(row["id"]), message])
	ContentCatalog.end_system_edit()
	var engine_diffs: Array = report["engine_diffs"]
	var value_diffs: Array = report["value_diffs"]
	for row: Dictionary in before:
		var id: String = str(row["id"])
		var after: Dictionary = ContentCatalog.get_entry("items", id)
		for key: String in ItemModifiers.engine_diff(after, row):
			if not _allowed_diff(key, row, after, retyped.has(id)):
				engine_diffs.append("%s.%s: %s -> %s" % [id, key, str(row.get(key, null)), str(after.get(key, null))])
		if not is_equal_approx(GearBudget.item_value(after), GearBudget.item_value(row)):
			value_diffs.append("%s: %.2f -> %.2f" % [id, GearBudget.item_value(row), GearBudget.item_value(after)])
	var totals_after: Dictionary = _loadout_totals()
	var loadout_diffs: Array = report["loadout_diffs"]
	for key: Variant in totals_before:
		if not is_equal_approx(DictUtil.to_float(totals_before[key]), DictUtil.to_float(totals_after.get(key, -1.0))):
			loadout_diffs.append("%s: %.2f -> %.2f" % [str(key), DictUtil.to_float(totals_before[key]), DictUtil.to_float(totals_after.get(key, -1.0))])
	report["validate_all"] = ContentCatalog.validate_all()
	report["ok"] = errors.is_empty() and engine_diffs.is_empty() and value_diffs.is_empty() and loadout_diffs.is_empty() \
		and DictUtil.dict_of(report["validate_all"]).is_empty()
	return report

# The allowed engine-key changes of the migration (FORGE1_ITEMS 8 acceptance (a) and (b), user answer 2026-10-17).
static func _allowed_diff(key: String, row: Dictionary, after: Dictionary, retyped: bool) -> bool:
	if key != "off_hand_category":
		return false
	if retyped:
		return str(after.get(key, "")).is_empty()
	return str(row.get("item_type", "")) == "SHIELD" and str(row.get(key, "")).is_empty() and str(after.get(key, "")) == "shield"

# An off-hand row in a main-category family (strict hand typing turns it into main_1h).
static func _is_retyped(row: Dictionary, family: Dictionary) -> bool:
	return not family.is_empty() and ItemFamilies.category(family) == "main" and ItemModifiers.flat_hand(row) == ItemFamilies.HAND_OFF

static func _loadout_totals() -> Dictionary:
	var out: Dictionary = {}
	for kind: String in ["classes", "units"]:
		for e: Dictionary in ContentCatalog.entries(kind):
			out["%s:%s" % [kind, str(e.get("id", ""))]] = GearBudget.loadout_value(e)
	return out

# ------------------------------------------------------------------------------
# Family derivation (matrix section 1, mechanical)
# ------------------------------------------------------------------------------

# The family rows for flat item rows `items`: one per gear token, the non-gear catch-alls, the copies.
static func derive_families(items: Array[Dictionary]) -> Array[Dictionary]:
	var by_family: Dictionary = {}
	for row: Dictionary in items:
		var id: String = ItemFamilies.family_id_for_row(row)
		if id.is_empty():
			continue
		if not by_family.has(id):
			by_family[id] = []
		var list: Array = by_family[id]
		list.append(row)
	var out: Array[Dictionary] = []
	for token: String in Proficiency.gear_types():
		if COPY_FAMILIES.has(token) or not by_family.has(token):
			continue
		out.append(_gear_family(token, _rows(by_family[token])))
	for token: Variant in COPY_FAMILIES:
		var source: String = str(COPY_FAMILIES[token])
		if by_family.has(source):
			var copy: Dictionary = _gear_family(source, _rows(by_family[source]))
			copy["id"] = str(token)
			copy["name"] = str(token).capitalize()
			out.append(copy)
	for id: Variant in NON_GEAR_FAMILIES:
		out.append(_plain_family(str(id), str(NON_GEAR_FAMILIES[id]), _rows(by_family.get(id, []))))
	return out

static func _rows(raw: Variant) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for r: Variant in DictUtil.array_of(raw):
		out.append(DictUtil.dict_of(r))
	return out

static func _gear_family(id: String, rows: Array[Dictionary]) -> Dictionary:
	var type: String = str(rows[0].get("item_type", "WEAPON"))
	var all_off: bool = true
	for row: Dictionary in rows:
		all_off = all_off and ItemModifiers.flat_hand(row) == ItemFamilies.HAND_OFF
	var off_family: bool = all_off and ItemFamilies.OFF_CATEGORIES.has(id)
	var f: Dictionary = {"id": id, "name": id.capitalize(), "description": "", "item_type": type, "art": ""}
	var by_hand: Dictionary = {}
	if ItemFamilies.HAND_TYPES.has(type):
		for row: Dictionary in rows:
			var hand: String = ItemModifiers.flat_hand(row)
			if not off_family and hand == ItemFamilies.HAND_OFF:
				hand = ItemFamilies.HAND_MAIN_1H   # strict hand typing retype
			if not by_hand.has(hand):
				by_hand[hand] = []
			var list: Array = by_hand[hand]
			list.append(row)
		var hands: Array[String] = _ranked(by_hand, ItemFamilies.HANDS)
		for extra: String in ContentFactory._strings(EXTRA_HANDS.get(id, [])):
			if not hands.has(extra):
				hands.append(extra)
		f["hands"] = hands
		f["default_hand"] = hands[0]
	else:
		by_hand[ItemFamilies.NO_HAND_KEY] = rows
		f["slots"] = ItemFamilies.ARMOR_SLOTS.duplicate() if type == "ARMOR" else ItemFamilies.ACCESSORY_SLOTS.duplicate()
	if type == "WEAPON":
		var counts: Dictionary = {}
		for row: Dictionary in rows:
			var s: String = str(row.get("scaling_stat", "STR"))
			if not counts.has(s):
				counts[s] = []
			var list: Array = counts[s]
			list.append(row)
		f["scaling_stats"] = _ranked(counts, ItemFamilies.SCALING_STATS)
	var base: Dictionary = {}
	var bands: Dictionary = {}
	var numbers: Array[String] = ItemFamilies.number_fields_for(type)
	for hand: Variant in by_hand:
		var hand_rows: Array[Dictionary] = _rows(by_hand[hand])
		base[hand] = _base_row(hand_rows)
		bands[hand] = _band_row(hand_rows, numbers, type, off_family)
	if f.has("hands"):
		var default_hand: String = str(f["default_hand"])
		for hand: String in ContentFactory._strings(f["hands"]):
			if not base.has(hand):   # a NEW hand copies the default hand's numbers and bands
				base[hand] = DictUtil.dict_of(base[default_hand]).duplicate(true)
				bands[hand] = DictUtil.dict_of(bands[default_hand]).duplicate(true)
	var overrides: Dictionary = DictUtil.dict_of(BAND_OVERRIDES.get(id, {}))
	for hand: Variant in bands:
		var row: Dictionary = DictUtil.dict_of(bands[hand])
		for field: Variant in overrides:
			row[field] = DictUtil.dict_of(overrides[field]).duplicate()
	f["base"] = base
	f["bands"] = bands
	match type:
		"WEAPON":
			f["adjustable"] = WEAPON_ADJUSTABLE.duplicate()
			f["modifier_kinds"] = ItemModifiers.kind_names()
			f["max_modifiers"] = 8
			f["max_tiers"] = 3
		"SHIELD", "ARMOR":
			f["adjustable"] = PROTECTIVE_ADJUSTABLE.duplicate()
			f["modifier_kinds"] = ItemModifiers.kind_names()
			f["max_modifiers"] = 6
			f["max_tiers"] = 2
	var rules: Array[String] = []
	if Equipment.FOCUS_TYPES.has(id):
		rules.append(ItemFamilies.RULE_FOCUS)
	if Equipment.ALWAYS_RANGED_TYPES.has(id):
		rules.append(ItemFamilies.RULE_ALWAYS_RANGED)
	if type == "SHIELD":
		rules.append(ItemFamilies.RULE_SHIELD)
	if SkillDef.SPELL_SOURCE_TYPES.has(id):
		rules.append(ItemFamilies.RULE_SPELL_SOURCE)
	f["engine_rules"] = rules
	f["damage_type"] = _most_common(rows, "damage_type")
	return f

static func _plain_family(id: String, type: String, rows: Array[Dictionary]) -> Dictionary:
	var f: Dictionary = {"id": id, "name": id.capitalize(), "description": "", "item_type": type, "art": "", "engine_rules": [], "damage_type": "", "base": {}, "bands": {}}
	if type == "ACCESSORY":
		f["slots"] = ItemFamilies.ACCESSORY_SLOTS.duplicate()
		f["base"] = {ItemFamilies.NO_HAND_KEY: _base_row(rows)}
		f["bands"] = {ItemFamilies.NO_HAND_KEY: _band_row(rows, ItemFamilies.number_fields_for(type), type, false)}
		f["adjustable"] = ACCESSORY_ADJUSTABLE.duplicate()
		var kinds: Array[String] = [ItemModifiers.STAT_BONUS, ItemModifiers.EFFECT, ItemModifiers.REQUIREMENT]
		f["modifier_kinds"] = kinds
		f["max_modifiers"] = 4
		f["max_tiers"] = 0
	else:
		f["adjustable"] = []
		f["modifier_kinds"] = []
		f["max_modifiers"] = 0
		f["max_tiers"] = 0
	f["damage_type"] = _most_common(rows, "damage_type") if not rows.is_empty() else ""
	return f

# Keys of `groups` ordered by group size (largest first), ties in `order`.
static func _ranked(groups: Dictionary, order: Array[String]) -> Array[String]:
	var out: Array[String] = []
	for key: String in order:
		if groups.has(key):
			out.append(key)
	out.sort_custom(func(a: String, b: String) -> bool:
		var na: int = DictUtil.array_of(groups[a]).size()
		var nb: int = DictUtil.array_of(groups[b]).size()
		return na > nb or (na == nb and order.find(a) < order.find(b)))
	return out

static func _values(rows: Array[Dictionary], field: String) -> Array[int]:
	var out: Array[int] = []
	var key: String = str(ItemFamilies.FLAT_KEYS[field])
	for row: Dictionary in rows:
		var rng: Array = ItemFamilies.NUMBER_RANGES[field]
		out.append(DictUtil.to_int(row.get(key, rng[0]), DictUtil.to_int(rng[0])))
	out.sort()
	return out

# Rule 1: the LOWER median of each number over the rows (all five numbers, so items without a delta stay exact).
static func _base_row(rows: Array[Dictionary]) -> Dictionary:
	var out: Dictionary = {}
	for field: String in ItemFamilies.NUMBER_FIELDS:
		var values: Array[int] = _values(rows, field)
		if values.is_empty():
			var rng: Array = ItemFamilies.NUMBER_RANGES[field]
			out[field] = DictUtil.to_int(rng[0])
		else:
			out[field] = values[(values.size() - 1) >> 1]
	return out

# Rule 2: [floor(0.75 x min), ceil(1.25 x max)] clamped to the schema range; zero-only power [0, 10], other zero-only
# numbers [0, 0]; weapons use the common block cost and burden bands; bonus_value [0, 4] main, [0, 6] off families.
static func _band_row(rows: Array[Dictionary], numbers: Array[String], type: String, off_family: bool) -> Dictionary:
	var out: Dictionary = {}
	for field: String in numbers:
		if type == "WEAPON" and WEAPON_COMMON_BANDS.has(field):
			out[field] = DictUtil.dict_of(WEAPON_COMMON_BANDS[field]).duplicate()
			continue
		var values: Array[int] = _values(rows, field)
		var rng: Array = ItemFamilies.NUMBER_RANGES[field]
		var lo: int = DictUtil.to_int(rng[0])
		var hi: int = DictUtil.to_int(rng[1])
		if values.is_empty() or values[values.size() - 1] == 0:
			out[field] = {"min": 0, "max": 10 if field == "power" else 0}
			continue
		out[field] = {"min": clampi(floori(0.75 * values[0]), lo, hi), "max": clampi(ceili(1.25 * values[values.size() - 1]), lo, hi)}
	if ItemFamilies.HAND_TYPES.has(type):
		out[ItemFamilies.BONUS_BAND] = {"min": 0, "max": 6 if off_family else 4}
	else:
		# Armor and accessories have no hand category: rule 2 applied to the bonus value (zero-only -> [0, 0]).
		var bonus: Array[float] = []
		for row: Dictionary in rows:
			var total: float = 0.0
			for raw: Variant in DictUtil.array_of(row.get("effects", [])):
				total += GearBudget.effect_value(DictUtil.dict_of(raw))
			var stats: Dictionary = DictUtil.dict_of(row.get("stat_bonuses", {}))
			for stat: Variant in stats:
				total += DictUtil.to_int(stats[stat]) * GearBudget.W_STAT_POINT
			bonus.append(total)
		bonus.sort()
		var top: float = bonus[bonus.size() - 1] if not bonus.is_empty() else 0.0
		out[ItemFamilies.BONUS_BAND] = {"min": maxi(0, floori(0.75 * bonus[0])) if top > 0.0 else 0, "max": ceili(1.25 * top)}
	return out

static func _most_common(rows: Array[Dictionary], key: String) -> String:
	var counts: Dictionary = {}
	var best: String = ""
	var best_n: int = 0
	for row: Dictionary in rows:
		var v: String = str(row.get(key, ""))
		counts[v] = DictUtil.to_int(counts.get(v, 0)) + 1
		if DictUtil.to_int(counts[v]) > best_n:
			best_n = DictUtil.to_int(counts[v])
			best = v
	return best
