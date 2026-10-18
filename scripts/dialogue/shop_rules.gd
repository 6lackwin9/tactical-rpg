class_name ShopRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Shop Rules (S11, spec DLG1-L)
# Pure validation of world shops (WorldCatalog kind `shops`): keeper, stock of base items (count -1 = unlimited,
# price -1 = derived from the item value), item loot sold once, buy / sell multipliers, the item types the shop
# buys and its open flags; and the runtime over GameState: open / closed, integer prices, offers, buy, sell.
# Prices (DLG1-L 2.5, REP1-L 2.3): bm = roundi(buy_mult x 100), sm = roundi(sell_mult x 100), rb / rs = the buy / sell
# factor of the keeper faction's reputation band (RepRules; a keeper without a faction is neutral, 100);
# buy = the fixed price when > 0, else maxi(1, ceil(value x bm x rb / 10000)); sell = floor(value x sm x rs / 10000).
# Loot prices are fixed. A hostile keeper's shop is closed (no offers, no buying, no selling, no service choice).
# Limited stock sold is kept in gs.shops; loot sold is claimed (RewardGrant.claim_loot). No restock in DLG1.
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
const BUY_MAX: int = 99   # most of one offer per buy (stock counts are 1..99); keeps price x count far from int overflow
const SUPPLIES: String = "supplies"   # TRAV2-L: offer kind and id of travel supplies
const SUPPLY_PRICE_MAX: int = 999   # TRAV2-L: gold per travel supply (0 = sells none)
const REP_NEUTRAL: int = 100   # rb = rs of the neutral band (and of a keeper without a faction)

# Item types a shop may buy: every ItemFamilies type but QUEST (also the schema default of `buys`).
static func buy_types() -> Array[String]:
	var out: Array[String] = []
	for t: String in ItemFamilies.ITEM_TYPES:
		if t != "QUEST":
			out.append(t)
	return out

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
	var supply_price: Variant = e.get("supply_price", 0)
	if not (supply_price is int) or DictUtil.to_int(supply_price) < 0 or DictUtil.to_int(supply_price) > SUPPLY_PRICE_MAX:
		errors.append("Supply price must be 0 (not sold) or 1..%d." % SUPPLY_PRICE_MAX)
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
	var allowed: Array[String] = buy_types()
	for v: Variant in DictUtil.array_of(raw):
		var t: String = str(v)
		if not (v is String) or not allowed.has(t):
			errors.append("Buys types: '%s' is not an item type a shop buys (%s)." % [t, ", ".join(allowed)])
		if seen.has(t):
			errors.append("Buys types lists '%s' twice." % t)
		seen[t] = true
	return errors

# ------------------------------------------------------------------------------
# Runtime (pure over GameState; refs resolve from gs.world_id)
# ------------------------------------------------------------------------------

static func shop_of(gs: GameState, shop_ref: String) -> Dictionary:
	return WorldCatalog.resolve(gs.world_id, "shops", shop_ref) if not shop_ref.is_empty() else {}

static func _holds(gs: GameState, raw: Variant) -> bool:
	for c: Variant in DictUtil.array_of(JsonUtil.normalize(raw)):
		if not QuestRules.condition_met(gs, DictUtil.dict_of(c)):
			return false
	return true

# Open: the shop resolves, its requires_flags hold, its keeper is not gone and the keeper's faction is not hostile.
static func is_open(gs: GameState, shop_ref: String) -> bool:
	var shop: Dictionary = shop_of(gs, shop_ref)
	return not shop.is_empty() and _holds(gs, shop.get("requires_flags", [])) and not DialogueRules.is_gone(gs, str(shop.get("keeper", ""))) and keeper_band(gs, shop) != RepRules.HOSTILE

# The resolved faction qid of the shop keeper ("" = no keeper, unknown keeper or a keeper without a faction).
static func keeper_faction(gs: GameState, shop: Dictionary) -> String:
	return DialogueRules.faction_qid(gs, str(shop.get("keeper", "")))

# The reputation band of the keeper's faction (neutral without a faction).
static func keeper_band(gs: GameState, shop: Dictionary) -> String:
	var fq: String = keeper_faction(gs, shop)
	return RepRules.band(RepRules.standing(gs, fq)) if not fq.is_empty() else RepRules.NEUTRAL

static func _mult(shop: Dictionary, key: String, fallback: float) -> int:
	return roundi(DictUtil.to_float(shop.get(key, fallback), fallback) * 100.0)

# Integer prices (no floats in the money path): `fixed` > 0 wins; else ceil(value x bm x rb / 10000), at least 1.
# `rb` / `rs`: the band factor (RepRules.buy_pct / sell_pct of keeper_band); the default is neutral.
static func buy_price(shop: Dictionary, value: int, fixed: int = DERIVED, rb: int = REP_NEUTRAL) -> int:
	if fixed > 0:
		return fixed
	var num: int = maxi(value, 0) * _mult(shop, "buy_mult", 1.0) * rb
	@warning_ignore("integer_division")
	var up: int = (num + 9999) / 10000   # integer ceil (num >= 0)
	return maxi(1, up)

static func sell_price(shop: Dictionary, value: int, rs: int = REP_NEUTRAL) -> int:
	var num: int = maxi(value, 0) * _mult(shop, "sell_mult", 0.5) * rs
	@warning_ignore("integer_division")
	var down: int = num / 10000   # integer floor (num >= 0)
	return down

static func _value_of(item_id: String) -> int:
	return DictUtil.to_int(ContentCatalog.get_entry("items", item_id).get("value", 0))

static func _sold(gs: GameState, shop_qid: String, item_id: String) -> int:
	return DictUtil.to_int(DictUtil.dict_of(gs.shops.get(shop_qid, {})).get(item_id, 0))

# What the shop sells now: [{kind: "item" | "loot", id, item, count, price, left}]. Stock entries whose requires_flags
# hold (left -1 = unlimited, else count - sold, 0 = sold out), then loot entries that are unclaimed and whose loot
# requires_flags hold (id = the loot qid, item / count = what it grants, left 1). [] when the shop is closed.
static func offers(gs: GameState, shop_ref: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if not is_open(gs, shop_ref):
		return out
	var shop: Dictionary = shop_of(gs, shop_ref)
	var sqid: String = str(shop["_qid"])
	var rb: int = RepRules.buy_pct(keeper_band(gs, shop))
	for raw: Variant in DictUtil.array_of(JsonUtil.normalize(shop.get("stock", []))):
		var s: Dictionary = DictUtil.dict_of(raw)
		if not _holds(gs, s.get("requires_flags", [])):
			continue
		var item: String = str(s.get("item", ""))
		var count: int = DictUtil.to_int(s.get("count", UNLIMITED), UNLIMITED)
		var left: int = UNLIMITED if count == UNLIMITED else maxi(count - _sold(gs, sqid, item), 0)
		out.append({"kind": "item", "id": item, "item": item, "count": count, "price": buy_price(shop, _value_of(item), DictUtil.to_int(s.get("price", DERIVED), DERIVED), rb), "left": left})
	for raw: Variant in DictUtil.array_of(JsonUtil.normalize(shop.get("loot", []))):
		var l: Dictionary = DictUtil.dict_of(raw)
		var loot: Dictionary = WorldCatalog.resolve(gs.world_id, "loot", str(l.get("loot", "")))
		var lqid: String = str(loot.get("_qid", ""))
		if lqid.is_empty() or gs.loot_claimed.has(lqid) or not _holds(gs, loot.get("requires_flags", [])):
			continue
		out.append({"kind": "loot", "id": lqid, "item": str(loot.get("ref", "")), "count": DictUtil.to_int(loot.get("count", 1), 1), "price": DictUtil.to_int(l.get("price", 1)), "left": 1})
	# TRAV2-L 2.3: travel supplies at the fixed supply_price (no band factor, unlimited); left = room up to the cap.
	var supply_price: int = DictUtil.to_int(shop.get("supply_price", 0))
	if supply_price > 0:
		out.append({"kind": SUPPLIES, "id": SUPPLIES, "item": "", "count": 1, "price": supply_price, "left": maxi(PartyState.SUPPLIES_MAX - gs.party.supplies, 0)})
	return out

static func _fail(message: String) -> Dictionary:
	var errors: Array[String] = [message]
	return {"ok": false, "errors": errors}

# Buys `count` of offer `kind` / `id` (loot: count 1). All-or-nothing: closed, not offered, sold out or not enough gold
# change nothing. Then gold falls, the stash gains the items, limited stock counts in gs.shops, loot is claimed
# (it leaves every quest pool). Returns {ok, errors, price (total), items: {id: count}}.
static func buy(gs: GameState, shop_ref: String, kind: String, id: String, count: int = 1) -> Dictionary:
	if not is_open(gs, shop_ref):
		return _fail("The shop is closed.")
	if count < 1 or (kind == "loot" and count != 1):
		return _fail("Loot is sold one at a time." if kind == "loot" else "Buy at least one.")
	if count > BUY_MAX:
		return _fail("Buy at most %d at a time." % BUY_MAX)
	var offer: Dictionary = {}
	for o: Dictionary in offers(gs, shop_ref):
		if str(o["kind"]) == kind and str(o["id"]) == id:
			offer = o
			break
	if offer.is_empty():
		return _fail("'%s' is not for sale here." % id)
	var left: int = DictUtil.to_int(offer["left"])
	if kind == SUPPLIES:
		return _buy_supplies(gs, DictUtil.to_int(offer["price"]), left, count)
	if left != UNLIMITED and left < count:
		return _fail("Sold out.")
	var total: int = DictUtil.to_int(offer["price"]) * count
	if gs.party.gold < total:
		return _fail("Not enough gold.")
	var item: String = str(offer["item"])
	var amount: int = DictUtil.to_int(offer["count"]) if kind == "loot" else count
	if kind == "loot":
		var why: String = RewardGrant.claim_loot(gs, id)
		if not why.is_empty():
			return _fail(why)
	gs.party.gold -= total
	gs.party.add_to_stash(item, amount)
	if kind == "item" and DictUtil.to_int(offer["count"]) != UNLIMITED:
		var sqid: String = str(shop_of(gs, shop_ref)["_qid"])
		var sold: Dictionary = DictUtil.dict_of(gs.shops.get(sqid, {}))
		sold[item] = DictUtil.to_int(sold.get(item, 0)) + count
		gs.shops[sqid] = sold
	var errors: Array[String] = []
	return {"ok": true, "errors": errors, "price": total, "items": {item: amount}}

# Supplies (TRAV2-L 2.3): all-or-nothing; no stash change, no gs.shops entry. {ok, errors, price (total), supplies}.
static func _buy_supplies(gs: GameState, price: int, left: int, count: int) -> Dictionary:
	if count > left:
		return _fail("You can carry at most %d supplies." % PartyState.SUPPLIES_MAX)
	var total: int = price * count
	if gs.party.gold < total:
		return _fail("Not enough gold.")
	gs.party.gold -= total
	var added: int = gs.party.add_supplies(count)
	var errors: Array[String] = []
	return {"ok": true, "errors": errors, "price": total, "supplies": added}

# Stash items the shop buys: type in `buys`, never QUEST, sell price >= 1: [{item, count (held), price}], stash order.
static func sell_list(gs: GameState, shop_ref: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if not is_open(gs, shop_ref):
		return out
	var shop: Dictionary = shop_of(gs, shop_ref)
	var buys: Array[String] = JsonUtil.strings(shop.get("buys", []))
	var rs: int = RepRules.sell_pct(keeper_band(gs, shop))
	for key: Variant in gs.party.stash:
		var id: String = str(key)
		var entry: Dictionary = ContentCatalog.get_entry("items", id)
		var type: String = str(entry.get("item_type", ""))
		if entry.is_empty() or type == "QUEST" or not buys.has(type):
			continue
		var price: int = sell_price(shop, DictUtil.to_int(entry.get("value", 0)), rs)
		if price >= 1:
			out.append({"item": id, "count": DictUtil.to_int(gs.party.stash[key]), "price": price})
	return out

# Sells `count` of `item` (it must be on sell_list and held). Stash falls, gold rises (capped at PartyState.GOLD_MAX).
# Sold items never restock the shop. Gold above PartyState.GOLD_MAX is dropped with a note (decision "DLG1 open items"
# (1)): `gold` is the amount actually added. Returns {ok, errors, price (total), gold, notes}.
static func sell(gs: GameState, shop_ref: String, item: String, count: int = 1) -> Dictionary:
	if not is_open(gs, shop_ref):
		return _fail("The shop is closed.")
	var row: Dictionary = {}
	for r: Dictionary in sell_list(gs, shop_ref):
		if str(r["item"]) == item:
			row = r
	if row.is_empty():
		return _fail("The shop does not buy '%s'." % item)
	if count < 1 or DictUtil.to_int(row["count"]) < count:
		return _fail("Not enough %s." % item)
	var total: int = DictUtil.to_int(row["price"]) * count
	var _removed: bool = gs.party.remove_from_stash(item, count)
	var added: int = gs.party.add_gold(total)
	var errors: Array[String] = []
	var notes: Array[String] = []
	if added < total:
		notes.append("Gold is capped at %d (%d not added)." % [PartyState.GOLD_MAX, total - added])
	return {"ok": true, "errors": errors, "price": total, "gold": added, "notes": notes}
