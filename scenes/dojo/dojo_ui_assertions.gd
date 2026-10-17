class_name DojoUiAssertions
extends RefCounted

# Dojo UI checks (split from DojoAssertions by CLN-12): the six workspace panels react to session events,
# the Party & Stash screen, slot rules, stash filters, the 4-slot party viewer, party carry and overlays.
# `host` must be inside the scene tree. Called by DojoAssertions.run.

static func run_ui_smoke_assertions(session: DojoSession, host: Node) -> bool:
	session.reset()
	var workspace := DojoWorkspace.new()
	host.add_child(workspace)
	var panels: Array[DojoPanel] = [
		DojoUnitInspectorPanel.new(),
		DojoGearPanel.new(),
		DojoStagePanel.new(),
		DojoConsolePanel.new(),
		DojoDerivedStatsPanel.new(),
		DojoLogPanel.new(),
	]
	workspace.setup(session, panels)

	var ids: Array[String] = []
	for panel: DojoPanel in panels:
		ids.append(panel.panel_id)
	var all_mounted: bool = true
	for id: String in ["inspector", "gear", "stage", "console", "derived", "log"]:
		if workspace.get_panel(id) == null or id not in ids:
			all_mounted = false

	session.perform_strike("main")
	session.apply_loadout("dual_wield")
	var log_panel_raw: UiPanel = workspace.get_panel("log")
	var shown: int = 0
	if log_panel_raw is DojoLogPanel:
		var log_panel: DojoLogPanel = log_panel_raw
		shown = log_panel.displayed_entry_count()
	var ok: bool = all_mounted and shown == session.log_entries.size() and shown >= 2

	DojoAssertions.dispose(workspace)
	if not ok:
		print("  [Dojo-26] Modular panels mount in workspace and mirror session log: FAIL (mounted: %s, cells: %d / %d)" % [all_mounted, shown, session.log_entries.size()])
	else:
		print("  [Dojo-26] Modular panels mount in workspace and mirror session log: PASS (6 panels, %d log cells)" % shown)

	var modal_ok: bool = run_party_stash_assertions(session, host)
	return ok and modal_ok

# Party & Stash screen: shared session, party selection, stash equip and gear sync
static func run_party_stash_assertions(session: DojoSession, host: Node) -> bool:
	session.reset()
	var modal := CharacterEquipmentModal.new()
	host.add_child(modal)
	modal.attach_session(session)
	modal.open_modal(session.active_hero)

	var party_ok: bool = modal.party_allies.size() == 2 and modal.active_hero == session.party_allies[0]
	modal.select_hero(session.party_allies[1])
	var select_ok: bool = session.active_hero == session.party_allies[1] and modal.active_hero.name == "Ranger"

	# Stash equip through the modal API changes the shared session's hero gear
	modal.select_hero(session.party_allies[0])
	var claymore: Equipment = ItemCatalog.create_iron_claymore()
	session.shared_stash.add_item(claymore)
	modal._equip_item(claymore)
	var equip_ok: bool = session.active_hero.inventory.get_main_weapon() == claymore and not session.shared_stash.items.has(claymore)

	DojoAssertions.dispose(modal)
	var ok: bool = party_ok and select_ok and equip_ok
	if not ok:
		print("  [Dojo-27] Party & Stash screen shares session (party list, selection, stash equip): FAIL (%s %s %s)" % [party_ok, select_ok, equip_ok])
	else:
		print("  [Dojo-27] Party & Stash screen shares session (party list, selection, stash equip): PASS")
	return ok

static func _names(items: Array[Item]) -> Array[String]:
	var result: Array[String] = []
	for item: Item in items:
		result.append(item.name)
	return result

static func _has_all(names: Array[String], expected: Array[String]) -> bool:
	for n: String in expected:
		if not names.has(n):
			return false
	return true

static func _has_none(names: Array[String], banned: Array[String]) -> bool:
	for n: String in banned:
		if names.has(n):
			return false
	return true

static func run_slot_filter_and_carry_assertions(session: DojoSession, host: Node) -> bool:
	var passed: bool = true
	session.reset()
	var hero: Combatant = session.active_hero
	var stash: Array[Item] = session.shared_stash.items

	# Assert 28: Equipment slot rules (off-hand items ONLY in the off hand; main-hand weapons main hand only unless the
	# legacy flag). Rule-derived per item hand, so it holds before and after ITEM1-A retypes the shortsword to main_1h.
	var claymore: Equipment = ItemCatalog.create_iron_claymore()
	var broadsword: Equipment = ItemCatalog.create_steel_broadsword()
	var shortsword: Equipment = ItemCatalog.create_iron_shortsword()
	var buckler: Equipment = ItemCatalog.create_buckler()
	var tome: Equipment = ItemCatalog.create_mystic_tome()
	var main: Equipment.EquipSlot = Equipment.EquipSlot.MAIN_HAND
	var off: Equipment.EquipSlot = Equipment.EquipSlot.OFF_HAND
	var flag_saved: bool = Equipment.allow_plain_dual_wield
	var rules_ok: bool = true
	for flag: bool in [false, true]:
		Equipment.allow_plain_dual_wield = flag
		rules_ok = rules_ok and claymore.can_equip_in_slot(main) and not claymore.can_equip_in_slot(off)
		rules_ok = rules_ok and broadsword.can_equip_in_slot(main) and broadsword.can_equip_in_slot(off) == flag
		if shortsword.hand() == Equipment.HAND_MAIN_1H:
			rules_ok = rules_ok and shortsword.can_equip_in_slot(main) and shortsword.can_equip_in_slot(off) == flag
		else:
			rules_ok = rules_ok and shortsword.can_equip_in_slot(off) and not shortsword.can_equip_in_slot(main)
		rules_ok = rules_ok and buckler.can_equip_in_slot(off) and not buckler.can_equip_in_slot(main)
		rules_ok = rules_ok and tome.can_equip_in_slot(off) and not tome.can_equip_in_slot(main)
	Equipment.allow_plain_dual_wield = flag_saved
	var stash_tome: Equipment = null
	for item: Item in stash:
		if item.id == tome.id and item is Equipment:
			stash_tome = item
	if stash_tome == null:
		stash_tome = tome
	var refused_main: bool = not session.equip_item(stash_tome, main)
	if not rules_ok or not refused_main:
		print("  [Dojo-28] Slot rules: off-hand items (shield, orb, tome) off-hand only; main-hand weapons main-hand only unless the legacy flag: FAIL (%s %s)" % [rules_ok, refused_main])
		passed = false
	else:
		print("  [Dojo-28] Slot rules: off-hand items (shield, orb, tome) off-hand only; main-hand weapons main-hand only unless the legacy flag: PASS")

	# Assert 29: Stash filters - per-slot standard filter + custom dimensions
	var f := StashFilter.new()
	f.slot_filter = "MAIN_HAND"
	var main_names: Array[String] = _names(f.apply(stash, hero))
	f.slot_filter = "OFF_HAND"
	var off_names: Array[String] = _names(f.apply(stash, hero))
	f.slot_filter = "BODY"
	var body_names: Array[String] = _names(f.apply(stash, hero))
	f.slot_filter = "CONSUMABLE"
	var con_names: Array[String] = _names(f.apply(stash, hero))
	# The three light blades are off-category weapons before ITEM1-A and main_1h weapons after (strict hand typing);
	# a main_1h blade shows in the off-hand list only under the dojo's legacy dual-wield flag. Rule-derived per hand.
	var off_items: Array[String] = ["Mystic Spellbook", "Iron Kite Shield", "Duelist Buckler"]
	var blades: Array[String] = ["Iron Shortsword", "Spiked Knuckles", "Shadow Dagger"]
	var blades_main: bool = ItemCatalog.create_iron_shortsword().hand() == Equipment.HAND_MAIN_1H
	var blades_off: bool = not blades_main or Equipment.allow_plain_dual_wield
	var slots_ok: bool = (
		_has_all(main_names, ["Iron Claymore", "Titan Warhammer", "Oak Shortbow", "Runed Staff"])
		and _has_none(main_names, off_items)
		and (_has_all(main_names, blades) if blades_main else _has_none(main_names, blades))
		and _has_all(off_names, off_items)
		and (_has_all(off_names, blades) if blades_off else _has_none(off_names, blades))
		and _has_none(off_names, ["Iron Claymore", "Oak Shortbow", "Titan Warhammer", "Runed Staff"])
		and _has_all(body_names, ["Steel Plate Cuirass", "Leather Brigandine"]) and _has_none(body_names, ["Iron Kite Shield", "Duelist Buckler"])
		and con_names.size() == 3
	)

	f = StashFilter.new()
	f.stats = ["INT"]
	var int_names: Array[String] = _names(f.apply(stash, hero))
	f = StashFilter.new()
	f.rarities = [int(Item.ItemRarity.RARE)]
	var rare_names: Array[String] = _names(f.apply(stash, hero))
	f = StashFilter.new()
	f.damage_types = ["magic"]
	var magic_names: Array[String] = _names(f.apply(stash, hero))
	f = StashFilter.new()
	f.slot_filter = "MAIN_HAND"
	f.hand = "2H"
	f.usable_only = true
	var usable_2h: Array[String] = _names(f.apply(stash, hero)) # Warrior cannot use bow or staff
	f = StashFilter.new()
	f.slot_filter = "MAIN_HAND"
	f.sort_mode = "POWER"
	var sorted_main: Array[Item] = f.apply(stash, hero)
	f = StashFilter.new()
	f.search = "iron"
	var iron_names: Array[String] = _names(f.apply(stash, hero))
	var custom_ok: bool = (
		_has_all(int_names, ["Mystic Spellbook", "Runed Staff"]) and _has_none(int_names, ["Iron Claymore", "Titan Warhammer"])
		and _has_all(rare_names, ["Titan Warhammer", "Runed Staff"]) and _has_none(rare_names, ["Iron Claymore"])
		and magic_names.size() == 2
		and _has_all(usable_2h, ["Iron Claymore", "Titan Warhammer"]) and _has_none(usable_2h, ["Oak Shortbow", "Runed Staff"])
		and not sorted_main.is_empty() and sorted_main[0].name == "Titan Warhammer"
		and _has_all(iron_names, ["Iron Claymore", "Iron Shortsword", "Iron Kite Shield"]) and _has_none(iron_names, ["Titan Warhammer"])
		and f.custom_count() == 1
	)
	if not slots_ok or not custom_ok:
		print("  [Dojo-29] Stash filters: per-slot (shield in off-hand) + stat/rarity/damage/hand/usable/search/sort: FAIL (%s %s)" % [slots_ok, custom_ok])
		passed = false
	else:
		print("  [Dojo-29] Stash filters: per-slot (shield in off-hand) + stat/rarity/damage/hand/usable/search/sort: PASS")

	# Assert 30: Party viewer always renders 4 stable slots
	session.reset()
	var party_panel := DojoPartyPanel.new()
	host.add_child(party_panel)
	party_panel.bind(session)
	var two_member_slots: int = party_panel.slot_card_count()
	var lone: Array[Combatant] = [session.party_allies[0]]
	var lone_session: DojoSession = DojoSession.wrap_party(lone, null)
	party_panel.bind(lone_session)
	var one_member_slots: int = party_panel.slot_card_count()
	DojoAssertions.dispose(party_panel)
	if two_member_slots != DojoSession.MAX_PARTY_SIZE or one_member_slots != DojoSession.MAX_PARTY_SIZE or DojoSession.MAX_PARTY_SIZE != 4:
		print("  [Dojo-30] Party viewer keeps 4 slots regardless of party size: FAIL (%d / %d)" % [two_member_slots, one_member_slots])
		passed = false
	else:
		print("  [Dojo-30] Party viewer keeps 4 slots regardless of party size: PASS")

	# Assert 31: Party-wide carry vs per-unit burden
	session.reset()
	var warrior: Combatant = session.party_allies[0]
	var ranger: Combatant = session.party_allies[1]
	var expected_capacity: int = PartyCarry.BASE_CAPACITY
	for member: Combatant in session.party_allies:
		var ctx: Dictionary = member.get_weapon_context()
		expected_capacity += member.stats.get_effective_stat("STR", ctx) * 3 + member.stats.get_effective_stat("CONS", ctx)
	var report: Dictionary = session.carry_report
	var expected_load: float = 0.0
	for it: Item in session.shared_stash.items:
		expected_load += it.weight
	for member: Combatant in session.party_allies:
		for eq: Equipment in member.inventory._iter_equipped_gear():
			expected_load += eq.weight
		for con: Consumable in member.inventory.battle_pouch:
			expected_load += con.weight
	var base_ok: bool = (
		UiTheme.dict_int(report, "capacity") == expected_capacity
		and absf(UiTheme.dict_float(report, "load") - expected_load) < 0.01
		and str(report.get("tier", "")) == "LIGHT" and UiTheme.dict_int(report, "speed_penalty") == 0
	)

	# Per-unit burden: stripping only the warrior's armor must not change the ranger's speed.
	warrior.inventory.equip(ItemCatalog.create_steel_plate_cuirass(), Equipment.EquipSlot.BODY)   # the leather brigandine is trained away to 0 burden
	warrior.sync_from_stats_and_gear()
	var ranger_speed_before: int = ranger.speed
	var warrior_speed_before: int = warrior.speed
	session.unequip_slot(Equipment.EquipSlot.BODY)
	var burden_local: bool = warrior.speed > warrior_speed_before and ranger.speed == ranger_speed_before

	# Party overload: loading the stash beyond capacity slows EVERY unit equally.
	var baseline_w: int = warrior.speed
	var baseline_r: int = ranger.speed
	var ballast: Array[Equipment] = []
	for i: int in range(20):
		var plate: Equipment = ItemCatalog.create_steel_plate_cuirass()
		ballast.append(plate)
		session.shared_stash.add_item(plate)
	session.notify_external_change()
	var over: Dictionary = session.carry_report
	var penalty: int = UiTheme.dict_int(over, "speed_penalty")
	var overload_ok: bool = (
		str(over.get("tier", "")) == "OVERLOADED" and penalty >= 1
		and warrior.speed == baseline_w - penalty and ranger.speed == baseline_r - penalty
		and warrior.carry_speed_penalty == penalty and ranger.carry_speed_penalty == penalty
	)
	for plate: Equipment in ballast:
		session.shared_stash.remove_item(plate)
	session.notify_external_change()
	var recovered: bool = warrior.speed == baseline_w and ranger.speed == baseline_r and str(session.carry_report.get("tier", "")) != "OVERLOADED"
	if not (base_ok and burden_local and overload_ok and recovered):
		print("  [Dojo-31] Unit burden is per-unit; party carry is shared (overload slows everyone): FAIL (%s %s %s %s)" % [base_ok, burden_local, overload_ok, recovered])
		passed = false
	else:
		print("  [Dojo-31] Unit burden is per-unit; party carry is shared (overload slows everyone): PASS (capacity %d, overload -%d speed)" % [expected_capacity, penalty])

	# Assert 32: Overlay windows (picker, allocation, custom filter) open above the workspace without changing it
	session.reset()
	var workspace := DojoWorkspace.new()
	host.add_child(workspace)
	var inspector := DojoUnitInspectorPanel.new()
	var gear := DojoGearPanel.new()
	var stash_panel := DojoStashPanel.new()
	var panels: Array[DojoPanel] = [inspector, gear, stash_panel]
	workspace.setup(session, panels)
	var layer: UiOverlayLayer = workspace.get_overlay_layer()
	session.party_allies[0].stats.add_free_attribute_points(2)   # a fully spent unit has nothing to allocate
	inspector._on_points_pressed()
	var allocation_open: bool = layer.has_overlay("allocate") and session.is_allocating_stats
	gear.overlay_requested.emit(DojoPickerPanel.for_slot(Equipment.EquipSlot.MAIN_HAND))
	var picker_open: bool = layer.has_overlay("picker")
	stash_panel.overlay_requested.emit(DojoStashFilterPanel.new())
	var filter_open: bool = layer.has_overlay("stash_filter") and layer.overlay_count() == 3
	session.cancel_allocation()
	var allocation_closed: bool = not layer.has_overlay("allocate")
	layer.close_all()
	var all_closed: bool = layer.overlay_count() == 0
	DojoAssertions.dispose(workspace)
	var overlays_ok: bool = allocation_open and picker_open and filter_open and allocation_closed and all_closed
	if not overlays_ok:
		print("  [Dojo-32] Overlay windows open on request and close on completion: FAIL (%s %s %s %s %s)" % [allocation_open, picker_open, filter_open, allocation_closed, all_closed])
		passed = false
	else:
		print("  [Dojo-32] Overlay windows open on request and close on completion: PASS")
	return passed
