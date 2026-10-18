---
class: ShopRules
extends: RefCounted
description: "Tactical RPG - Shop Rules (S11, spec DLG1-L) Pure validation of world shops (WorldCatalog kind `shops`): keeper, stock of base items (count -1 = unlimited, price -1 = derived from the item value), item loot sold once, buy / sell multipliers, the item types the shop buys and its open flags; and the runtime over GameState: open / closed, integer prices, offers, buy, sell. Prices (DLG1-L 2.5, REP1-L 2.3): bm = roundi(buy_mult x 100), sm = roundi(sell_mult x 100), rb / rs = the buy / sell factor of the keeper faction's reputation band (RepRules; a keeper without a faction is neutral, 100); buy = the fixed price when > 0, else maxi(1, ceil(value x bm x rb / 10000)); sell = floor(value x sm x rs / 10000). Loot prices are fixed. A hostile keeper's shop is closed (no offers, no buying, no selling, no service choice). Limited stock sold is kept in gs.shops; loot sold is claimed (RewardGrant.claim_loot). No restock in DLG1."
---

# Properties
* `MAX_STOCK`: int = 32 (const)
* `MAX_LOOT`: int = 8 (const)
* `STOCK_KEYS`: Array[String] = ["item", "count", "price", "requires_flags"] (const)
* `LOOT_KEYS`: Array[String] = ["loot", "price"] (const)
* `COUNT_MAX`: int = 99 (const)
* `PRICE_MAX`: int = 99999 (const)
* `BUY_MULT_MIN`: float = 0.5 (const)
* `BUY_MULT_MAX`: float = 3.0 (const)
* `SELL_MULT_MIN`: float = 0.0 (const)
* `SELL_MULT_MAX`: float = 1.0 (const)
* `UNLIMITED`: int = -1 (const)
* `DERIVED`: int = -1 (const)
* `BUY_MAX`: int = 99   # most of one offer per buy (stock counts are 1..99); keeps price x count far from int overflow (const)
* `SUPPLIES`: String = "supplies"   # TRAV2-L: offer kind and id of travel supplies (const)
* `SUPPLY_PRICE_MAX`: int = 999   # TRAV2-L: gold per travel supply (0 = sells none) (const)
* `REP_NEUTRAL`: int = 100   # rb = rs of the neutral band (and of a keeper without a faction) (const)
* `out`: Array[String] = []
* `errors`: Array[String] = []
* `supply_price`: Variant = e.get("supply_price", 0)
* `errors`: Array[String] = []
* `list`: Array = DictUtil.array_of(JsonUtil.normalize(raw))
* `seen`: Dictionary = {}
* `s`: Dictionary = list[i]
* `item`: String = str(s.get("item", ""))
* `label`: String = "Stock '%s'" % item if not item.is_empty() else "Stock %d" % (i + 1)
* `base`: Dictionary = ContentCatalog.get_entry("items", item) if s.get("item", null) is String else {}
* `count`: Variant = s.get("count", UNLIMITED)
* `price`: Variant = s.get("price", DERIVED)
* `errors`: Array[String] = []
* `list`: Array = DictUtil.array_of(JsonUtil.normalize(raw))
* `seen`: Dictionary = {}
* `l`: Dictionary = list[i]
* `ref`: String = str(l.get("loot", ""))
* `label`: String = "Loot for sale '%s'" % ref if not ref.is_empty() else "Loot for sale %d" % (i + 1)
* `ref_errs`: Array[String] = WorldCatalog.ref_errors(world_id, label, "loot", ref)
* `hit`: Dictionary = WorldCatalog.resolve(world_id, "loot", ref)
* `key`: String = str(hit.get("_qid", ref))
* `price`: Variant = l.get("price", null)
* `errors`: Array[String] = []
* `x`: float = DictUtil.to_float(raw)
* `errors`: Array[String] = []
* `seen`: Dictionary = {}
* `allowed`: Array[String] = buy_types()
* `t`: String = str(v)
* `out`: Array[Dictionary] = []
* `shop`: Dictionary = shop_of(gs, shop_ref)
* `sqid`: String = str(shop["_qid"])
* `rb`: int = RepRules.buy_pct(keeper_band(gs, shop))
* `s`: Dictionary = DictUtil.dict_of(raw)
* `item`: String = str(s.get("item", ""))
* `count`: int = DictUtil.to_int(s.get("count", UNLIMITED), UNLIMITED)
* `left`: int = UNLIMITED if count == UNLIMITED else maxi(count - _sold(gs, sqid, item), 0)
* `l`: Dictionary = DictUtil.dict_of(raw)
* `loot`: Dictionary = WorldCatalog.resolve(gs.world_id, "loot", str(l.get("loot", "")))
* `lqid`: String = str(loot.get("_qid", ""))
* `supply_price`: int = DictUtil.to_int(shop.get("supply_price", 0))
* `out`: Array[Dictionary] = []
* `shop`: Dictionary = shop_of(gs, shop_ref)
* `buys`: Array[String] = JsonUtil.strings(shop.get("buys", []))
* `rs`: int = RepRules.sell_pct(keeper_band(gs, shop))
* `id`: String = str(key)
* `entry`: Dictionary = ContentCatalog.get_entry("items", id)
* `type`: String = str(entry.get("item_type", ""))
* `price`: int = sell_price(shop, DictUtil.to_int(entry.get("value", 0)), rs)

# Methods

### `static shop_of(gs: GameState, shop_ref: String) -> Dictionary`
```logic
return WorldCatalog.resolve(gs.world_id, "shops", shop_ref) if not shop_ref.is_empty() else {}
```

### `static _holds(gs: GameState, raw: Variant) -> bool`
```logic
for c: Variant in DictUtil.array_of(JsonUtil.normalize(raw)):
	if not QuestRules.condition_met(gs, DictUtil.dict_of(c)):
		return false
return true
```

### `static is_open(gs: GameState, shop_ref: String) -> bool`
```logic
var shop: Dictionary = shop_of(gs, shop_ref)
return not shop.is_empty() and _holds(gs, shop.get("requires_flags", [])) and not DialogueRules.is_gone(gs, str(shop.get("keeper", ""))) and keeper_band(gs, shop) != RepRules.HOSTILE
```

### `static keeper_faction(gs: GameState, shop: Dictionary) -> String`
```logic
return DialogueRules.faction_qid(gs, str(shop.get("keeper", "")))
```

### `static keeper_band(gs: GameState, shop: Dictionary) -> String`
```logic
var fq: String = keeper_faction(gs, shop)
return RepRules.band(RepRules.standing(gs, fq)) if not fq.is_empty() else RepRules.NEUTRAL
```

### `static _mult(shop: Dictionary, key: String, fallback: float) -> int`
```logic
return roundi(DictUtil.to_float(shop.get(key, fallback), fallback) * 100.0)
```

### `static buy_price(shop: Dictionary, value: int, fixed: int = DERIVED, rb: int = REP_NEUTRAL) -> int`
```logic
if fixed > 0:
	return fixed
var num: int = maxi(value, 0) * _mult(shop, "buy_mult", 1.0) * rb
@warning_ignore("integer_division")
var up: int = (num + 9999) / 10000   # integer ceil (num >= 0)
return maxi(1, up)
```

### `static sell_price(shop: Dictionary, value: int, rs: int = REP_NEUTRAL) -> int`
```logic
var num: int = maxi(value, 0) * _mult(shop, "sell_mult", 0.5) * rs
@warning_ignore("integer_division")
var down: int = num / 10000   # integer floor (num >= 0)
return down
```

### `static _value_of(item_id: String) -> int`
```logic
return DictUtil.to_int(ContentCatalog.get_entry("items", item_id).get("value", 0))
```

### `static _sold(gs: GameState, shop_qid: String, item_id: String) -> int`
```logic
return DictUtil.to_int(DictUtil.dict_of(gs.shops.get(shop_qid, {})).get(item_id, 0))
```

### `static _fail(message: String) -> Dictionary`
```logic
var errors: Array[String] = [message]
return {"ok": false, "errors": errors}
```

### `static buy(gs: GameState, shop_ref: String, kind: String, id: String, count: int = 1) -> Dictionary`
```logic
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
```

### `static _buy_supplies(gs: GameState, price: int, left: int, count: int) -> Dictionary`
```logic
if count > left:
	return _fail("You can carry at most %d supplies." % PartyState.SUPPLIES_MAX)
var total: int = price * count
if gs.party.gold < total:
	return _fail("Not enough gold.")
gs.party.gold -= total
var added: int = gs.party.add_supplies(count)
var errors: Array[String] = []
return {"ok": true, "errors": errors, "price": total, "supplies": added}
```

### `static sell(gs: GameState, shop_ref: String, item: String, count: int = 1) -> Dictionary`
```logic
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
```

