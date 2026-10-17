class_name ShopPanel
extends PanelContainer

# ==============================================================================
# Tactical RPG - Shop debug panel (S11, spec DLG1-L commit 3)
# Thin view over ShopRules for one open shop: party gold, the buy list (ShopRules.offers; sold-out or unaffordable
# rows disabled), the sell list (ShopRules.sell_list) and Close. Buys and sells one at a time; every change goes
# through ShopRules and the panel re-renders. Close emits `closed`; the owner hands it back to the dialogue.
# ==============================================================================

signal closed()
signal noted(text: String)

var gs: GameState = null
var shop_qid: String = ""

var _title: Label = null
var _gold: Label = null
var _buy: VBoxContainer = null
var _sell: VBoxContainer = null

func _init() -> void:
	add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_PANEL_BG))
	var root := VBoxContainer.new()
	add_child(root)
	_title = UiTheme.label("", UiTheme.FONT_LG, UiTheme.COL_TITLE)
	root.add_child(_title)
	_gold = UiTheme.label("")
	root.add_child(_gold)
	root.add_child(UiTheme.header("Buy"))
	_buy = VBoxContainer.new()
	root.add_child(_buy)
	root.add_child(UiTheme.header("Sell"))
	_sell = VBoxContainer.new()
	root.add_child(_sell)
	root.add_child(UiTheme.button("Close", "Leave the shop (DialogueRunner.close_shop)", close))
	visible = false

func show_shop(p_gs: GameState, p_shop_qid: String) -> void:
	gs = p_gs
	shop_qid = p_shop_qid
	visible = true
	refresh()

func is_open() -> bool:
	return gs != null and visible

# Buys one of offer `kind` / `id` (ShopRules.buy, all-or-nothing).
func buy_one(kind: String, id: String) -> Dictionary:
	if gs == null:
		return {}
	var res: Dictionary = ShopRules.buy(gs, shop_qid, kind, id, 1)
	noted.emit("Bought %s for %d gold." % [str(res["items"]), DictUtil.to_int(res["price"])] if DictUtil.to_bool(res["ok"]) else "Buy refused: %s" % str(res["errors"]))
	refresh()
	return res

# Sells one `item` from the stash (ShopRules.sell).
func sell_one(item: String) -> Dictionary:
	if gs == null:
		return {}
	var res: Dictionary = ShopRules.sell(gs, shop_qid, item, 1)
	noted.emit("Sold %s for %d gold." % [item, DictUtil.to_int(res["price"])] if DictUtil.to_bool(res["ok"]) else "Sell refused: %s" % str(res["errors"]))
	refresh()
	return res

func close() -> void:
	gs = null
	shop_qid = ""
	visible = false
	closed.emit()

func _item_name(id: String) -> String:
	return str(ContentCatalog.get_entry("items", id).get("name", id))

func refresh() -> void:
	UiTheme.clear_children(_buy)
	UiTheme.clear_children(_sell)
	if gs == null:
		return
	_title.text = str(ShopRules.shop_of(gs, shop_qid).get("name", shop_qid))
	_gold.text = "Gold: %d" % gs.party.gold
	for o: Dictionary in ShopRules.offers(gs, shop_qid):
		var left: int = DictUtil.to_int(o["left"])
		var price: int = DictUtil.to_int(o["price"])
		var stock: String = "" if left == ShopRules.UNLIMITED else "  (%d left)" % left
		var btn: Button = UiTheme.button("%s x%d - %d gold%s" % [_item_name(str(o["item"])), DictUtil.to_int(o["count"]) if str(o["kind"]) == "loot" else 1, price, stock], "ShopRules.buy", buy_one.bind(str(o["kind"]), str(o["id"])))
		btn.disabled = left == 0 or gs.party.gold < price
		_buy.add_child(btn)
	for r: Dictionary in ShopRules.sell_list(gs, shop_qid):
		_sell.add_child(UiTheme.button("%s (%d held) - %d gold" % [_item_name(str(r["item"])), DictUtil.to_int(r["count"]), DictUtil.to_int(r["price"])], "ShopRules.sell", sell_one.bind(str(r["item"]))))
