---
class: ShopPanel
extends: PanelContainer
description: "Tactical RPG - Shop debug panel (S11, spec DLG1-L commit 3) Thin view over ShopRules for one open shop: party gold, the buy list (ShopRules.offers; sold-out or unaffordable rows disabled), the sell list (ShopRules.sell_list), the keeper faction's reputation band (REP1-L) and Close. Buys and sells one at a time (travel supplies x1 or x5, TRAV2-L); every change goes through ShopRules and the panel re-renders. Close emits `closed`; the owner hands it back to the dialogue."
---

# Properties
* `SUPPLY_BULK`: int = 5   # TRAV2-L: the bulk supplies button (const)
* `gs`: GameState = null
* `shop_qid`: String = ""
* `_title`: Label = null
* `_gold`: Label = null
* `_standing`: Label = null   # REP1-L: the keeper faction's band ("" without a faction)
* `_buy`: VBoxContainer = null
* `_sell`: VBoxContainer = null

# Signals
* `closed()`
* `noted(text: String)`

# Methods

### `_init() -> void`
```logic
add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_PANEL_BG))
var root := VBoxContainer.new()
add_child(root)
_title = UiTheme.label("", UiTheme.FONT_LG, UiTheme.COL_TITLE)
root.add_child(_title)
_gold = UiTheme.label("")
root.add_child(_gold)
_standing = UiTheme.label("")
root.add_child(_standing)
root.add_child(UiTheme.header("Buy"))
_buy = VBoxContainer.new()
root.add_child(_buy)
root.add_child(UiTheme.header("Sell"))
_sell = VBoxContainer.new()
root.add_child(_sell)
root.add_child(UiTheme.button("Close", "Leave the shop (DialogueRunner.close_shop)", close))
visible = false
```

### `show_shop(p_gs: GameState, p_shop_qid: String) -> void`
```logic
gs = p_gs
shop_qid = p_shop_qid
visible = true
refresh()
```

### `is_open() -> bool`
```logic
return gs != null and visible
```

### `buy_one(kind: String, id: String, count: int = 1) -> Dictionary`
```logic
if gs == null:
	return {}
var res: Dictionary = ShopRules.buy(gs, shop_qid, kind, id, count)
var what: String = "%d supplies" % DictUtil.to_int(res.get("supplies", 0)) if kind == ShopRules.SUPPLIES else str(res.get("items", {}))
noted.emit("Bought %s for %d gold." % [what, DictUtil.to_int(res["price"])] if DictUtil.to_bool(res["ok"]) else "Buy refused: %s" % str(res["errors"]))
refresh()
return res
```

### `sell_one(item: String) -> Dictionary`
```logic
if gs == null:
	return {}
var res: Dictionary = ShopRules.sell(gs, shop_qid, item, 1)
noted.emit("Sold %s for %d gold." % [item, DictUtil.to_int(res["price"])] if DictUtil.to_bool(res["ok"]) else "Sell refused: %s" % str(res["errors"]))
for n: Variant in DictUtil.array_of(res.get("notes", [])):
	noted.emit(str(n))   # gold-cap note
refresh()
return res
```

### `close() -> void`
```logic
gs = null
shop_qid = ""
visible = false
closed.emit()
```

### `_item_name(id: String) -> String`
```logic
return str(ContentCatalog.get_entry("items", id).get("name", id))
```

### `standing_text() -> String`
```logic
return _standing.text
```

### `_add_supply_buttons(price: int, left: int) -> void`
```logic
for want: int in [1, SUPPLY_BULK]:
	var n: int = maxi(mini(want, left), 1)
	var label: String = "Supplies x%d - %d gold%s" % [n, price * n, "  (full)" if left == 0 else ""]
	var btn: Button = UiTheme.button(label, "ShopRules.buy (travel supplies)", buy_one.bind(ShopRules.SUPPLIES, ShopRules.SUPPLIES, n))
	btn.disabled = left == 0 or gs.party.gold < price * n
	_buy.add_child(btn)
```

### `refresh() -> void`
```logic
UiTheme.clear_children(_buy)
UiTheme.clear_children(_sell)
if gs == null:
	return
_title.text = str(ShopRules.shop_of(gs, shop_qid).get("name", shop_qid))
_gold.text = "Gold: %d  Supplies: %d/%d" % [gs.party.gold, gs.party.supplies, PartyState.SUPPLIES_MAX]
var shop: Dictionary = ShopRules.shop_of(gs, shop_qid)
var fq: String = ShopRules.keeper_faction(gs, shop)
_standing.text = RepRules.standing_text(str(WorldCatalog.resolve(gs.world_id, "factions", fq).get("name", fq)), ShopRules.keeper_band(gs, shop)) if not fq.is_empty() else ""
_standing.visible = not fq.is_empty()
for o: Dictionary in ShopRules.offers(gs, shop_qid):
	var left: int = DictUtil.to_int(o["left"])
	var price: int = DictUtil.to_int(o["price"])
	if str(o["kind"]) == ShopRules.SUPPLIES:
		_add_supply_buttons(price, left)
		continue
	var stock: String = "" if left == ShopRules.UNLIMITED else "  (%d left)" % left
	var btn: Button = UiTheme.button("%s x%d - %d gold%s" % [_item_name(str(o["item"])), DictUtil.to_int(o["count"]) if str(o["kind"]) == "loot" else 1, price, stock], "ShopRules.buy", buy_one.bind(str(o["kind"]), str(o["id"])))
	btn.disabled = left == 0 or gs.party.gold < price
	_buy.add_child(btn)
for r: Dictionary in ShopRules.sell_list(gs, shop_qid):
	_sell.add_child(UiTheme.button("%s (%d held) - %d gold" % [_item_name(str(r["item"])), DictUtil.to_int(r["count"]), DictUtil.to_int(r["price"])], "ShopRules.sell", sell_one.bind(str(r["item"]))))
```

