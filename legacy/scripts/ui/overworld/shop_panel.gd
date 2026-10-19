class_name ShopPanel
extends Control

# ==============================================================================
# Tactical RPG - Shop popup (S11, spec DLG1-L commit 3; UIR-6 popup)
# Thin view over ShopRules for one open shop: party gold, the buy list (ShopRules.offers; sold-out or unaffordable
# rows disabled), the sell list (ShopRules.sell_list), the keeper faction's reputation band (REP1-L) and Close. Buys and
# sells one at a time (travel supplies x1 or x5, TRAV2-L); every change goes through ShopRules and the panel re-renders.
# Close emits `closed`; the owner hands it back to the dialogue.
# UIR-6: a MenuStyle popup over the map (full-rect dim that swallows clicks, centred panel sized by popup_size; the owner
# puts it in a canvas layer). Buy / Sell tabs (marked with MenuStyle.selected), a detail line for the hovered or focused
# row (stats, owned count, who in the party can use it; item_detail), "Nothing to sell" when the sell list is empty,
# Esc and right click close. Text follows the one text-size setting (UiTheme.scaled, X8).
# ==============================================================================

signal closed()
signal noted(text: String)

const SUPPLY_BULK: int = 5   # TRAV2-L: the bulk supplies button
const BUY: String = "buy"
const SELL: String = "sell"
const NOTHING_TO_SELL: String = "Nothing to sell"
const NOTHING_TO_BUY: String = "Nothing for sale"
const CLOSE_HINT: String = "Esc or right click: close"
const DIM: Color = MainMenuView.POPUP_DIM
const REF_H: float = 540.0
const PANEL_REF: Vector2 = Vector2(560, 400)   # the panel at a 960x540 view; scaled with the view height
const FONT_BASE: int = MainMenuView.BASE_FONT

var gs: GameState = null
var shop_qid: String = ""
var tab: String = BUY

var _panel: PanelContainer = null
var _title: Label = null
var _gold: Label = null
var _standing: Label = null   # REP1-L: the keeper faction's band ("" without a faction)
var _tab_buy: Button = null
var _tab_sell: Button = null
var _buy: VBoxContainer = null
var _sell: VBoxContainer = null
var _empty: Label = null
var _detail: Label = null
var _hint: Label = null
var _theme: Theme = null
var _first_detail: String = ""

func _init() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_STOP
	_theme = MenuStyle.theme()
	theme = _theme
	var dim := ColorRect.new()
	dim.color = DIM
	dim.set_anchors_preset(Control.PRESET_FULL_RECT)
	dim.mouse_filter = Control.MOUSE_FILTER_STOP
	add_child(dim)
	_panel = PanelContainer.new()
	_panel.add_theme_stylebox_override("panel", UiTheme.box(Color(UiTheme.COL_PANEL_BG, 1.0), UiTheme.COL_HEADER, 8, 18, 14, 2))   # the settings popup's frame
	add_child(_panel)
	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 8)
	_panel.add_child(root)
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 16)
	root.add_child(head)
	_title = UiTheme.label("", FONT_BASE, UiTheme.COL_TITLE)
	_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(_title)
	_gold = UiTheme.label("", FONT_BASE)
	head.add_child(_gold)
	_standing = UiTheme.label("", FONT_BASE, UiTheme.COL_MUTED)
	root.add_child(_standing)
	var tabs := HBoxContainer.new()
	root.add_child(tabs)
	_tab_buy = _plain_button("Buy", set_tab.bind(BUY))
	tabs.add_child(_tab_buy)
	_tab_sell = _plain_button("Sell", set_tab.bind(SELL))
	tabs.add_child(_tab_sell)
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	root.add_child(scroll)
	var lists := VBoxContainer.new()
	lists.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(lists)
	_buy = VBoxContainer.new()
	lists.add_child(_buy)
	_sell = VBoxContainer.new()
	lists.add_child(_sell)
	_empty = UiTheme.label("", FONT_BASE, UiTheme.COL_MUTED)
	lists.add_child(_empty)
	_detail = UiTheme.label("", FONT_BASE, UiTheme.COL_HEADER)   # one line: a wrapped label measured before layout would push the panel off screen
	_detail.clip_text = true
	_detail.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	root.add_child(_detail)
	var bar := HBoxContainer.new()
	root.add_child(bar)
	_hint = UiTheme.label(CLOSE_HINT, FONT_BASE, UiTheme.COL_MUTED)
	_hint.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bar.add_child(_hint)
	bar.add_child(_plain_button("Close", close))
	resized.connect(_layout)
	visible = false

static func _plain_button(text: String, callback: Callable) -> Button:
	var b := Button.new()
	b.text = text
	b.pressed.connect(callback)
	return b

# The panel size for a view of `view` px (pure): PANEL_REF scaled with the height, at most 90% of either side.
static func popup_size(view: Vector2) -> Vector2:
	var k: float = maxf(view.y, 1.0) / REF_H
	return Vector2(minf(PANEL_REF.x * k, view.x * 0.9), minf(PANEL_REF.y * k, view.y * 0.9)).round()

func _layout() -> void:
	var target: Vector2 = popup_size(size)
	_panel.custom_minimum_size = target
	_panel.size = target
	_panel.position = ((size - target) / 2.0).floor()

func show_shop(p_gs: GameState, p_shop_qid: String) -> void:
	gs = p_gs
	shop_qid = p_shop_qid
	tab = BUY
	visible = true
	refresh()
	_layout()

func is_open() -> bool:
	return gs != null and visible

func set_tab(which: String) -> void:
	tab = SELL if which == SELL else BUY
	refresh()

# Buys `count` (default one) of offer `kind` / `id` (ShopRules.buy, all-or-nothing).
func buy_one(kind: String, id: String, count: int = 1) -> Dictionary:
	if gs == null:
		return {}
	var res: Dictionary = ShopRules.buy(gs, shop_qid, kind, id, count)
	var what: String = "%d supplies" % DictUtil.to_int(res.get("supplies", 0)) if kind == ShopRules.SUPPLIES else str(res.get("items", {}))
	noted.emit("Bought %s for %d gold." % [what, DictUtil.to_int(res["price"])] if DictUtil.to_bool(res["ok"]) else "Buy refused: %s" % str(res["errors"]))
	refresh()
	return res

# Sells one `item` from the stash (ShopRules.sell).
func sell_one(item: String) -> Dictionary:
	if gs == null:
		return {}
	var res: Dictionary = ShopRules.sell(gs, shop_qid, item, 1)
	noted.emit("Sold %s for %d gold." % [item, DictUtil.to_int(res["price"])] if DictUtil.to_bool(res["ok"]) else "Sell refused: %s" % str(res["errors"]))
	for n: Variant in DictUtil.array_of(res.get("notes", [])):
		noted.emit(str(n))   # gold-cap note
	refresh()
	return res

func close() -> void:
	gs = null
	shop_qid = ""
	visible = false
	closed.emit()

# Esc and right click close (taken only while the popup shows and its game is running).
func _usable() -> bool:
	return is_open() and is_visible_in_tree() and can_process()

func _input(event: InputEvent) -> void:
	var mouse: InputEventMouseButton = event as InputEventMouseButton
	if mouse != null and mouse.pressed and mouse.button_index == MOUSE_BUTTON_RIGHT and _usable():
		close()
		get_viewport().set_input_as_handled()

func _unhandled_input(event: InputEvent) -> void:
	if _usable() and event.is_action_pressed("ui_cancel"):
		close()
		get_viewport().set_input_as_handled()

static func _item_name(id: String) -> String:
	return str(ContentCatalog.get_entry("items", id).get("name", id))

# The keeper faction's band line as shown ("" without a faction); for tests.
func standing_text() -> String:
	return _standing.text

func detail_text() -> String:
	return _detail.text

func empty_text() -> String:
	return _empty.text if _empty.visible else ""

# The detail line of item `id` (pure over gs): "<name>: " then stats (gear: ItemText-style summary; consumables: their effect),
# "Owned n" (stash plus worn by any member) and who can use it ("Usable by: A, B", "No one in the party can use it",
# "Anyone can use it" for consumables); parts joined by " • ".
static func item_detail(p_gs: GameState, id: String) -> String:
	var entry: Dictionary = ContentCatalog.get_entry("items", id)
	var parts: Array[String] = []
	var it: Item = ContentFactory.item(entry) if not entry.is_empty() else null
	var eq: Equipment = it as Equipment
	var con: Consumable = it as Consumable
	if eq != null:
		var stats: String = UiTheme.equipment_summary(eq, null)
		if not stats.is_empty():
			parts.append(stats)
	elif con != null:
		parts.append(UiTheme.consumable_effect_text(con))
	elif it != null and not it.description.is_empty():
		parts.append(it.description)
	parts.append("Owned %d" % owned_count(p_gs, id))
	if eq != null:
		var names: Array[String] = []
		for m: UnitState in p_gs.party.members:
			if m.archetype() != null and m.archetype().can_equip_gear(eq):   # the class gear rule (ClassArchetype)
				names.append(PartyPreview.name_of(m))
		parts.append("Usable by: " + ", ".join(PackedStringArray(names)) if not names.is_empty() else "No one in the party can use it")
	elif con != null:
		parts.append("Anyone can use it")
	return "%s: %s" % [_item_name(id), " • ".join(parts)]

# Stash count plus the members wearing it.
static func owned_count(p_gs: GameState, id: String) -> int:
	var n: int = DictUtil.to_int(p_gs.party.stash.get(id, 0))
	for m: UnitState in p_gs.party.members:
		for slot: Variant in m.equipment:
			if str(m.equipment[slot]) == id:
				n += 1
	return n

static func supplies_detail(p_gs: GameState) -> String:
	return "Travel supplies • Carried %d / %d" % [p_gs.party.supplies, PartyState.SUPPLIES_MAX]

# A row button: clicking buys / sells; hovering or focusing it shows `detail` in the detail line.
func _row(parent: VBoxContainer, text: String, detail: String, callback: Callable, disabled: bool = false) -> void:
	var b := Button.new()
	b.text = text
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.disabled = disabled
	b.pressed.connect(callback)
	b.mouse_entered.connect(_show_detail.bind(detail))
	b.focus_entered.connect(_show_detail.bind(detail))
	parent.add_child(b)
	if _first_detail.is_empty():
		_first_detail = detail

func _show_detail(text: String) -> void:
	_detail.text = text

# TRAV2-L: "Supplies x1" and "Supplies x5" (x the room when less than 5 is left); both disabled when full or short of
# gold. The price shown is the total of the button.
func _add_supply_buttons(price: int, left: int) -> void:
	for want: int in [1, SUPPLY_BULK]:
		var n: int = maxi(mini(want, left), 1)
		var label: String = "Supplies x%d - %d gold%s" % [n, price * n, "  (full)" if left == 0 else ""]
		_row(_buy, label, supplies_detail(gs), buy_one.bind(ShopRules.SUPPLIES, ShopRules.SUPPLIES, n), left == 0 or gs.party.gold < price * n)

func apply_text_scale() -> void:
	_theme.default_font_size = UiTheme.scaled(FONT_BASE)
	for l: Label in [_gold, _standing, _empty, _detail]:
		l.add_theme_font_size_override("font_size", UiTheme.scaled(FONT_BASE))
	_title.add_theme_font_size_override("font_size", UiTheme.scaled(FONT_BASE + 4))
	_hint.add_theme_font_size_override("font_size", UiTheme.scaled(FONT_BASE - 4))

func font_px() -> int:
	return _theme.default_font_size

func refresh() -> void:
	apply_text_scale()
	UiTheme.clear_children(_buy)
	UiTheme.clear_children(_sell)
	_first_detail = ""
	if gs == null:
		return
	MenuStyle.mark_selected(_tab_buy, tab == BUY)
	MenuStyle.mark_selected(_tab_sell, tab == SELL)
	_buy.visible = tab == BUY
	_sell.visible = tab == SELL
	_title.text = str(ShopRules.shop_of(gs, shop_qid).get("name", shop_qid))
	_gold.text = "Gold: %d  Supplies: %d/%d" % [gs.party.gold, gs.party.supplies, PartyState.SUPPLIES_MAX]
	var shop: Dictionary = ShopRules.shop_of(gs, shop_qid)
	var fq: String = ShopRules.keeper_faction(gs, shop)
	_standing.text = RepRules.standing_text(str(WorldCatalog.resolve(gs.world_id, "factions", fq).get("name", fq)), ShopRules.keeper_band(gs, shop)) if not fq.is_empty() else ""
	_standing.visible = not fq.is_empty()
	var buy_first: String = ""
	for o: Dictionary in ShopRules.offers(gs, shop_qid):
		var left: int = DictUtil.to_int(o["left"])
		var price: int = DictUtil.to_int(o["price"])
		if str(o["kind"]) == ShopRules.SUPPLIES:
			_add_supply_buttons(price, left)
			continue
		var stock: String = "" if left == ShopRules.UNLIMITED else "  (%d left)" % left
		var item: String = str(o["item"])
		var text: String = "%s x%d - %d gold%s" % [_item_name(item), DictUtil.to_int(o["count"]) if str(o["kind"]) == "loot" else 1, price, stock]
		_row(_buy, text, item_detail(gs, item), buy_one.bind(str(o["kind"]), str(o["id"])), left == 0 or gs.party.gold < price)
	buy_first = _first_detail
	_first_detail = ""
	for r: Dictionary in ShopRules.sell_list(gs, shop_qid):
		var item: String = str(r["item"])
		_row(_sell, "%s (%d held) - %d gold" % [_item_name(item), DictUtil.to_int(r["count"]), DictUtil.to_int(r["price"])], item_detail(gs, item), sell_one.bind(item))
	var sell_first: String = _first_detail
	var empty: bool = (_buy if tab == BUY else _sell).get_child_count() == 0
	_empty.text = (NOTHING_TO_BUY if tab == BUY else NOTHING_TO_SELL) if empty else ""
	_empty.visible = empty
	_detail.text = buy_first if tab == BUY else sell_first   # the first row's detail until another row is hovered or focused
