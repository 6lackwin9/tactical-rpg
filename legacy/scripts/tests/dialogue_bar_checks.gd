class_name DialogueBarChecks
extends RefCounted

# ==============================================================================
# Tactical RPG - UIR-6 checks, run by DialoguePanelChecks on DialogueTestSuite's runtime fixture world (dg_run):
# lock reasons (DialogueRules.lock_reason), the dialogue bottom bar (layout, numbered choices, 1-9 hotkeys, locked
# reason on the button, Esc and right click = Farewell only where leaving is allowed), the shop popup (canvas layer,
# Buy / Sell tabs, detail line, "Nothing to sell", Esc and right click close) and the text size (X8) on both.
# Keys and clicks go through the viewport (push_input), not direct handler calls.
# ==============================================================================

const R: String = DialogueTestSuite.R

static func run(tree_parent: Node) -> bool:
	var ok: bool = _test_lock_reason()
	ok = _test_layout() and ok
	ok = _test_bar(tree_parent) and ok
	return ok

static func _report(label: String, checks: Dictionary) -> bool:
	return DialogueTestSuite._report(label, checks)

static func _test_lock_reason() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = DialogueTestSuite._gs()
	gs.party.stash = {}
	gs.party.gold = 0
	var item: String = DialogueTestSuite.I25
	var item_name: String = str(ContentCatalog.get_entry("items", item).get("name", item))
	checks["items: Needs <count> <item name>"] = DialogueRules.lock_reason(gs, {"requires": [{"kind": "items", "item": item, "count": 2}]}) == "Needs 2 %s" % item_name
	checks["gold: Needs <min> gold"] = DialogueRules.lock_reason(gs, {"requires": [{"kind": "gold", "min": 7}]}) == "Needs 7 gold"
	var above: int = DialogueRules.best_stat(gs, "KNOW") + 1
	checks["stat: Needs <STAT> <min>"] = DialogueRules.lock_reason(gs, {"requires": [{"kind": "stat", "stat": "KNOW", "min": above}]}) == "Needs KNOW %d" % above
	var rk: String = RepRules.key(WorldCatalog.qid(R, "no_faction"))
	var _rs: String = gs.set_flag(rk, 50)
	checks["reputation floor: Needs <faction> <band>"] = DialogueRules.lock_reason(gs, {"requires": [{"key": rk, "op": ">=", "value": 60}]}) == "Needs %s %s" % [rk, RepRules.band(60)]
	checks["reputation ceiling is not worded as a minimum"] = DialogueRules.lock_reason(gs, {"requires": [{"key": rk, "op": "<", "value": 10}]}) == "Needs %s below %s" % [rk, RepRules.band(10)] and DialogueRules.lock_reason(gs, {"requires": [{"key": rk, "op": "<=", "value": 10}]}) == "Needs %s at most %s" % [rk, RepRules.band(10)]
	checks["a story flag does not spoil"] = DialogueRules.lock_reason(gs, {"requires": [{"key": "dg.secret"}]}) == DialogueRules.LOCKED_TEXT
	checks["a holding unless reads not now"] = DialogueRules.lock_reason(gs, {"unless": [{"kind": "gold", "min": 0}]}) == DialogueRules.NOT_NOW_TEXT
	checks["the first failing condition names it"] = DialogueRules.lock_reason(gs, {"requires": [{"kind": "gold", "min": 0}, {"kind": "gold", "min": 3}]}) == "Needs 3 gold"
	checks["an open choice has no reason"] = DialogueRules.lock_reason(gs, {"requires": [{"kind": "gold", "min": 0}]}).is_empty()
	var r: DialogueRunner = DialogueRunner.open(gs, "mara")
	var v: Dictionary = r.view()
	var ask: Dictionary = DictUtil.dict_of(DictUtil.array_of(v["choices"])[DialogueTestSuite._find(v, "Ask")])
	var bye: Dictionary = DictUtil.dict_of(DictUtil.array_of(v["choices"])[DialogueTestSuite._find(v, DialogueTestSuite._line("farewell"))])
	checks["view carries the reason of a locked choice"] = DictUtil.to_bool(ask["locked"]) and str(ask["reason"]) == "Needs KNOW %d" % DictUtil.to_int(DialogueTestSuite._rx["know_hi"])
	checks["view marks the farewell service"] = str(bye["service"]) == "farewell" and str(ask["service"]).is_empty() and str(bye["reason"]).is_empty()
	return _report("UIR-6: lock reasons (DialogueRules.lock_reason, view reason and service)", checks)

static func _test_layout() -> bool:
	var checks: Dictionary = {}
	var small: Dictionary = DialoguePanel.bar_layout(Vector2(960, 540))
	var big: Dictionary = DialoguePanel.bar_layout(Vector2(1280, 720))
	checks["bar height at the reference size"] = is_equal_approx(DictUtil.to_float(small["height"]), DialoguePanel.BAR_REF_H)
	checks["bar scales with the view height, capped"] = DictUtil.to_float(big["height"]) <= 720.0 * DialoguePanel.BAR_MAX_FRAC and DictUtil.to_float(big["height"]) > DictUtil.to_float(small["height"])
	checks["portrait and choice column at the reference size"] = small["portrait"] == DialoguePanel.PORTRAIT_REF and is_equal_approx(DictUtil.to_float(small["choices_w"]), DialoguePanel.CHOICES_REF_W)
	checks["choice label: numbered 1-9, reason after the text"] = DialoguePanel.choice_label(0, "Hi") == "1  Hi" and DialoguePanel.choice_label(2, "Trade", "Needs 2 gold") == "3  Trade  (Needs 2 gold)" and DialoguePanel.choice_label(DialoguePanel.HOTKEYS, "Tenth") == "Tenth"
	checks["choice label: a reason its prefix already shows is not repeated"] = DialoguePanel.choice_label(1, "[KNOW 9] Ask", "Needs KNOW 9") == "2  [KNOW 9] Ask"
	var shop: Vector2 = ShopPanel.popup_size(Vector2(1280, 720))
	checks["shop popup fits the view"] = shop.x <= 1280.0 * 0.9 and shop.y <= 720.0 * 0.9 and shop.x > 0.0
	return _report("UIR-6: bar and popup geometry", checks)

# A talk partner whose second node offers two choices and no Farewell (leaving is not allowed there).
static func _build_tel() -> bool:
	var errors: Array[String] = []
	errors.append_array(WorldCatalog.upsert(R, "characters", {"id": "tel", "name": "Tel", "base_unit": str(DialogueTestSuite._rx["lo"]), "dialogue": "d_tel"}))
	errors.append_array(WorldCatalog.upsert(R, "dialogues", {"id": "d_tel", "name": "Tel", "start": "a", "nodes": [
		{"id": "a", "text": "Listen.", "choices": [{"text": "Go on", "to": "b"},
			{"text": "Trade", "to": "a", "requires": [{"kind": "items", "item": DialogueTestSuite.I25, "count": 1}], "show_locked": true}]},
		{"id": "b", "text": "Will you help?", "choices": [{"text": "Yes", "to": "a"}, {"text": "No", "to": "a"}]},
	]}))
	return errors.is_empty() and WorldCatalog.validate_world(R).is_empty()

static func _key(vp: Viewport, code: Key) -> void:
	DialoguePanelChecks._key(vp, code)

static func _right_click(vp: Viewport) -> void:
	for down: bool in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_RIGHT
		ev.pressed = down
		ev.position = Vector2(10, 10)
		ev.global_position = Vector2(10, 10)
		vp.push_input(ev, true)

static func _test_bar(tree_parent: Node) -> bool:
	var label: String = "UIR-6: dialogue bar and shop popup (hotkeys, locked reason, Esc / right click, tabs, empty sell, text size)"
	var checks: Dictionary = {"tel fixture": _build_tel()}
	if tree_parent == null:
		return _report(label + " (headless without a tree: fixture only)", checks)
	var view: RegionMapView = (load(DialoguePanelChecks.DEBUG_SCENE) as PackedScene).instantiate() as RegionMapView
	tree_parent.add_child(view)
	checks["new game"] = view.start_game(R).is_empty() and view.gs != null
	if view.gs == null:
		tree_parent.remove_child(view)
		view.free()
		return _report(label, checks)
	var gs: GameState = view.gs
	gs.party.active_ids = [str(DialogueTestSuite._rx["lo"])] as Array[String]
	var vp: Viewport = view.get_viewport()
	var dp: DialoguePanel = view.dialogue_panel
	var sp: ShopPanel = view.shop_panel
	# layout: a bottom bar over the map, the shop in the settings popup's canvas layer
	checks["the bar is docked full width at the bottom"] = dp.get_parent() is CanvasLayer and (dp.get_parent() as CanvasLayer).layer == DialoguePanel.LAYER and is_equal_approx(dp.anchor_left, 0.0) and is_equal_approx(dp.anchor_right, 1.0) and is_equal_approx(dp.anchor_top, 1.0) and is_equal_approx(dp.anchor_bottom, 1.0)
	checks["the shop sits in the settings popup's layer"] = sp.get_parent() is CanvasLayer and (sp.get_parent() as CanvasLayer).layer == MainMenuView.SETTINGS_LAYER
	# numbered choices, locked reason, hint
	view.talk("mara")
	var v: Dictionary = dp.runner.view() if dp.runner != null else {}
	var ask: int = DialogueTestSuite._find(v, "Ask")
	var bye: int = DialogueTestSuite._find(v, DialogueTestSuite._line("farewell"))
	var texts: Array[String] = dp.choice_texts()
	var reason: String = "Needs KNOW %d" % DictUtil.to_int(DialogueTestSuite._rx["know_hi"])
	checks["the conversation opens in the bar"] = dp.is_open() and ask >= 0 and bye >= 0 and texts.size() == DictUtil.array_of(v["choices"]).size()
	checks["choices are numbered from 1"] = not texts.is_empty() and texts[0].begins_with("1  ") and (texts.size() < 2 or texts[1].begins_with("2  "))
	var ask_btn: Button = dp._choices.get_child(ask) as Button if ask >= 0 else null
	checks["a stat-locked choice is disabled, its prefix names the need"] = ask_btn != null and ask_btn.disabled and ask_btn.text.contains("[%s]" % reason.trim_prefix("Needs ")) and not ask_btn.text.contains("(%s)" % reason)
	checks["choices are left-aligned"] = ask_btn != null and ask_btn.alignment == HORIZONTAL_ALIGNMENT_LEFT
	checks["the hint names the keys and Farewell"] = dp.hint_text().begins_with(DialoguePanel.HINT_CHOOSE) and dp.hint_text().contains(DialoguePanel.HINT_LEAVE % "Farewell")
	checks["portrait or a labelled box"] = dp.has_portrait() or dp._portrait_name.text == "Mara"
	# the locked choice's hotkey does nothing; the farewell's closes
	checks["fixture: Ask and Farewell have hotkeys"] = ask < DialoguePanel.HOTKEYS and bye < DialoguePanel.HOTKEYS
	if ask < DialoguePanel.HOTKEYS:
		_key(vp, (KEY_1 + ask) as Key)
		checks["the locked choice's number does nothing"] = dp.is_open() and dp.runner != null and dp.runner.node_id == dp.runner.start
	if bye < DialoguePanel.HOTKEYS:
		_key(vp, (KEY_1 + bye) as Key)
		checks["the Farewell's number picks it"] = not dp.is_open() and not view.busy()
	# Esc and right click are Farewell
	view.talk("mara")
	_key(vp, KEY_ESCAPE)
	checks["Esc is Farewell"] = not dp.is_open() and not view.busy()
	view.talk("mara")
	_right_click(vp)
	checks["right click is Farewell"] = not dp.is_open() and not view.busy()
	# no Farewell on the node: Esc and right click do nothing, the hint offers no leaving
	gs.party.stash.erase(DialogueTestSuite.I25)
	view.talk("tel")
	var trade_name: String = str(ContentCatalog.get_entry("items", DialogueTestSuite.I25).get("name", DialogueTestSuite.I25))
	var trade_btn: Button = dp._choices.get_child(1) as Button if dp._choices.get_child_count() > 1 else null
	checks["an item-locked choice is disabled and shows its reason"] = trade_btn != null and trade_btn.disabled and trade_btn.text == "2  Trade  (Needs 1 %s)" % trade_name
	var _go: Dictionary = dp.pick(0)
	checks["a node without Farewell does not allow leaving"] = dp.is_open() and not dp.can_leave() and not dp.hint_text().contains("Esc")
	_key(vp, KEY_ESCAPE)
	_right_click(vp)
	checks["Esc and right click keep the conversation there"] = dp.is_open() and dp.runner != null and dp.runner.node_id == "b"
	_key(vp, KEY_2)
	checks["2 picks the second choice"] = dp.is_open() and dp.runner != null and dp.runner.node_id == "a"
	dp._close()
	# shop popup: tabs, detail line, empty sell list, Esc and right click close back to the dialogue
	gs.party.stash = {}
	view.talk("hest")
	var _shop: Dictionary = dp.pick(DialogueTestSuite._find(dp.runner.view(), DialogueTestSuite._line("shop")))
	checks["the shop opens as a popup, the bar hides"] = sp.is_open() and sp.is_visible_in_tree() and not dp.visible and sp.tab == ShopPanel.BUY
	checks["Buy tab: the buy list shows, marked selected"] = sp._buy.visible and not sp._sell.visible and MenuStyle.is_selected(sp._tab_buy) and not MenuStyle.is_selected(sp._tab_sell) and sp._buy.get_child_count() > 0
	checks["the detail line describes the first offer"] = sp.detail_text().contains("Owned ")
	sp.set_tab(ShopPanel.SELL)
	checks["Sell tab with nothing to sell"] = sp._sell.visible and not sp._buy.visible and MenuStyle.is_selected(sp._tab_sell) and sp._sell.get_child_count() == 0 and sp.empty_text() == ShopPanel.NOTHING_TO_SELL
	gs.party.add_to_stash(DialogueTestSuite.I25, 2)
	sp.refresh()
	checks["Sell tab lists the stash item with its detail"] = sp._sell.get_child_count() > 0 and sp.empty_text().is_empty() and sp.detail_text().contains("Owned %d" % ShopPanel.owned_count(gs, DialogueTestSuite.I25))
	_key(vp, KEY_ESCAPE)
	checks["Esc closes the shop back to the dialogue"] = not sp.is_open() and dp.is_open()
	var _again: Dictionary = dp.pick(DialogueTestSuite._find(dp.runner.view(), DialogueTestSuite._line("shop")))
	_right_click(vp)
	checks["right click closes the shop back to the dialogue"] = not sp.is_open() and dp.is_open()
	# item detail rule: owned = stash + worn, usable names follow proficiency
	var weapon: String = str(DialogueTestSuite._rx["weapon"])
	var eq: Equipment = ContentFactory.item(ContentCatalog.get_entry("items", weapon)) as Equipment
	var any_can: bool = false
	for m: UnitState in gs.party.members:
		any_can = any_can or m.archetype().can_equip_gear(eq)
	var detail: String = ShopPanel.item_detail(gs, weapon)
	checks["detail: owned count and who can use it"] = detail.contains("Owned %d" % ShopPanel.owned_count(gs, weapon)) and detail.contains("Usable by: " if any_can else "No one in the party")
	# text size (X8): both read the one setting
	var saved: Variant = SettingsFile.get_value("ui/text_scale", 1.0)
	var all_sizes: bool = true
	for level: float in SettingsFile.TEXT_LEVELS:
		SettingsFile.set_value("ui/text_scale", level)
		dp.refresh()
		sp.show_shop(gs, sp.shop_qid if not sp.shop_qid.is_empty() else str(ShopRules.shop_of(gs, "s_hest").get("_qid", "")))
		var want: int = roundi(float(MainMenuView.BASE_FONT) * level)
		all_sizes = all_sizes and dp.font_px() == want and sp.font_px() == want and dp._text.get_theme_font_size("font_size") == want
	SettingsFile.set_value("ui/text_scale", saved)
	checks["dialogue and shop follow every text size"] = all_sizes
	sp.close()
	dp._close()
	tree_parent.remove_child(view)
	view.free()
	return _report(label, checks)
