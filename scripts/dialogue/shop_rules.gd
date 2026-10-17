class_name ShopRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Shop Rules (S11, spec DLG1-L)
# Pure validation of world shops (WorldCatalog kind `shops`): keeper, stock of base items (count -1 = unlimited,
# price -1 = derived from the item value), item loot sold once, buy / sell multipliers, the item types the shop
# buys and its open flags. Prices, offers, buying and selling arrive with the runtime (DLG1-L commit 2).
# ==============================================================================

const MAX_STOCK: int = 32
const MAX_LOOT: int = 8
const STOCK_KEYS: Array[String] = ["item", "count", "price", "requires_flags"]
const LOOT_KEYS: Array[String] = ["loot", "price"]
const COUNT_MAX: int = 99
const PRICE_MAX: int = 99999
const BUY_MULT_MIN: float = 0.5
const BUY_MULT_MAX: float = 3.0
const SELL_MULT_MIN: float = 0.0
const SELL_MULT_MAX: float = 1.0
const UNLIMITED: int = -1
const DERIVED: int = -1
# Item types a shop may buy (every ItemFamilies type but QUEST); also the schema default of `buys`.
const BUY_TYPES: Array[String] = ["WEAPON", "SHIELD", "ARMOR", "ACCESSORY", "CONSUMABLE", "MATERIAL"]

# ------------------------------------------------------------------------------
# Validation (called by WorldCatalog.validate_entry for kind `shops`)
# ------------------------------------------------------------------------------

static func shop_errors(world_id: String, e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	if str(e.get("keeper", "")).is_empty():
		errors.append("Choose the shop keeper.")
	errors.append_array(_stock_errors(e.get("stock", [])))
	errors.append_array(_loot_errors(world_id, e.get("loot", [])))
	errors.append_array(_mult_errors(e.get("buy_mult", 1.0), "Buy multiplier", BUY_MULT_MIN, BUY_MULT_MAX))
	errors.append_array(_mult_errors(e.get("sell_mult", 0.5), "Sell multiplier", SELL_MULT_MIN, SELL_MULT_MAX))
	errors.append_array(_buys_errors(e.get("buys", [])))
	errors.append_array(QuestRules.condition_errors(e.get("requires_flags", []), "Open when"))
	return errors

static func _stock_errors(raw: Variant) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is Array):
		errors.append("Stock must be a list of {item, count, price, requires_flags}.")
		return errors
	var list: Array = DictUtil.array_of(JsonUtil.normalize(raw))
	if list.size() > MAX_STOCK:
		errors.append("Stock holds at most %d entries (has %d)." % [MAX_STOCK, list.size()])
	var seen: Dictionary = {}
	for i: int in range(list.size()):
		if not (list[i] is Dictionary):
			errors.append("Stock %d must be an object {item, count, price, requires_flags}." % (i + 1))
			continue
		var s: Dictionary = list[i]
		var item: String = str(s.get("item", ""))
		var label: String = "Stock '%s'" % item if not item.is_empty() else "Stock %d" % (i + 1)
		for key: Variant in s:
			if not STOCK_KEYS.has(str(key)):
				errors.append("%s: unknown key '%s'." % [label, str(key)])
		var base: Dictionary = ContentCatalog.get_entry("items", item) if s.get("item", null) is String else {}
		if base.is_empty():
			errors.append("%s: unknown base item '%s'." % [label, item])
		elif str(base.get("item_type", "")) == "QUEST":
			errors.append("%s: Quest items can't be stocked." % label)
		if seen.has(item):
			errors.append("Stock lists '%s' twice." % item)
		seen[item] = true
		var count: Variant = s.get("count", UNLIMITED)
		if not (count is int) or not (DictUtil.to_int(count) == UNLIMITED or (DictUtil.to_int(count) >= 1 and DictUtil.to_int(count) <= COUNT_MAX)):
			errors.append("%s: count must be -1 (unlimited) or 1..%d." % [label, COUNT_MAX])
		var price: Variant = s.get("price", DERIVED)
		if not (price is int) or not (DictUtil.to_int(price) == DERIVED or (DictUtil.to_int(price) >= 1 and DictUtil.to_int(price) <= PRICE_MAX)):
			errors.append("%s: price must be -1 (derived) or 1..%d." % [label, PRICE_MAX])
		errors.append_array(QuestRules.condition_errors(s.get("requires_flags", []), label + " requires flags"))
	return errors

static func _loot_errors(world_id: String, raw: Variant) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is Array):
		errors.append("Loot for sale must be a list of {loot, price}.")
		return errors
	var list: Array = DictUtil.array_of(JsonUtil.normalize(raw))
	if list.size() > MAX_LOOT:
		errors.append("Loot for sale holds at most %d entries (has %d)." % [MAX_LOOT, list.size()])
	var seen: Dictionary = {}
	for i: int in range(list.size()):
		if not (list[i] is Dictionary):
			errors.append("Loot for sale %d must be an object {loot, price}." % (i + 1))
			continue
		var l: Dictionary = list[i]
		var ref: String = str(l.get("loot", ""))
		var label: String = "Loot for sale '%s'" % ref if not ref.is_empty() else "Loot for sale %d" % (i + 1)
		for key: Variant in l:
			if not LOOT_KEYS.has(str(key)):
				errors.append("%s: unknown key '%s'." % [label, str(key)])
		if not (l.get("loot", null) is String) or ref.is_empty():
			errors.append("%s: choose a loot entry." % label)
		else:
			var ref_errs: Array[String] = WorldCatalog.ref_errors(world_id, label, "loot", ref)
			errors.append_array(ref_errs)
			var hit: Dictionary = WorldCatalog.resolve(world_id, "loot", ref)
			if ref_errs.is_empty() and str(hit.get("reward_kind", "item")) != "item":
				errors.append("%s: Shops sell item loot only." % label)
			var key: String = str(hit.get("_qid", ref))
			if seen.has(key):
				errors.append("Loot for sale lists '%s' twice." % ref)
			seen[key] = true
		var price: Variant = l.get("price", null)
		if not (price is int) or DictUtil.to_int(price) < 1 or DictUtil.to_int(price) > PRICE_MAX:
			errors.append("%s: price must be 1..%d." % [label, PRICE_MAX])
	return errors

# lo..hi with at most two decimals.
static func _mult_errors(raw: Variant, label: String, lo: float, hi: float) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is float or raw is int):
		errors.append("%s must be a number." % label)
		return errors
	var x: float = DictUtil.to_float(raw)
	if x < lo - 0.00001 or x > hi + 0.00001:
		errors.append("%s must be %.1f..%.1f." % [label, lo, hi])
	if absf(x * 100.0 - roundf(x * 100.0)) > 0.001:
		errors.append("%s takes at most two decimals." % label)
	return errors

static func _buys_errors(raw: Variant) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is Array):
		errors.append("Buys types must be a list of item types.")
		return errors
	var seen: Dictionary = {}
	for v: Variant in DictUtil.array_of(raw):
		var t: String = str(v)
		if not (v is String) or not BUY_TYPES.has(t):
			errors.append("Buys types: '%s' is not an item type a shop buys (%s)." % [t, ", ".join(BUY_TYPES)])
		if seen.has(t):
			errors.append("Buys types lists '%s' twice." % t)
		seen[t] = true
	return errors
