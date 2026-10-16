class_name PartyBuilder
extends RefCounted

# ==============================================================================
# Tactical RPG - Party builder (S7): UnitState -> Combatant.
# Goes through the same ContentFactory.unit path as a catalog unit (class, tier, gear), with the
# level from XP and the spend from the state; then passives, learned skills, loadout, equipment.
# Unspent points are auto-spent on a COPY (the state is never changed here).
# ==============================================================================

# {"combatant": Combatant or null, "warnings": Array[String], "errors": Array[String]}
static func build_report(state: UnitState) -> Dictionary:
	var warnings: Array[String] = []
	var errors: Array[String] = state.validate()
	if not errors.is_empty():
		return {"combatant": null, "warnings": warnings, "errors": errors}
	var work: UnitState = state.duplicate_state()
	LevelUp.auto_for(work)   # unspent must be 0 to build
	var entry: Dictionary = ContentCatalog.get_entry("units", work.base_unit).duplicate(true)
	entry["level"] = work.level()
	entry["spend"] = work.spend.duplicate()
	var c: Combatant = ContentFactory.unit(entry)
	c.lore_ref = work.lore_ref
	ContentFactory.apply_passives(c, work.passives)
	for sk: String in work.learned_skills:
		c.learn_skill(sk, false)   # a saved unit keeps what it already learned (FORGE1 11.3: the requires check is for new learning)
	for key: Variant in work.equipment:
		var slot_name: String = str(key)
		var slot: int = Equipment.EquipSlot.keys().find(slot_name.to_upper())
		var built: Item = ContentFactory.item(ContentCatalog.get_entry("items", str(work.equipment[key])))
		if built is Equipment and slot >= 0:
			var eq: Equipment = built
			var old: Equipment = c.inventory.unequip(slot as Equipment.EquipSlot)
			if old != null:
				c.inventory.items.erase(old)
			c.inventory.equip(eq, slot)
	for item_id: String in work.carried:
		c.inventory.add_item(ContentFactory.item(ContentCatalog.get_entry("items", item_id)))
	c.sync_from_stats_and_gear()
	c.hp = c.max_hp
	var cls: Array[String] = JsonUtil.strings(work.loadout.get("class", []))
	var ext: Array[String] = JsonUtil.strings(work.loadout.get("extra", []))
	if not cls.is_empty() or not ext.is_empty():
		var err: String = c.set_loadout(cls, ext)
		if not err.is_empty():
			warnings.append("Unit '%s': stored loadout ignored (%s)." % [work.id, err])
	return {"combatant": c, "warnings": warnings, "errors": errors}

# The Combatant of `state`, or null when the state is invalid.
static func build(state: UnitState) -> Combatant:
	var res: Dictionary = build_report(state)
	return res["combatant"] as Combatant

# Combatants of the active members, in roster order.
static func build_active(party: PartyState) -> Array[Combatant]:
	var out: Array[Combatant] = []
	for m: UnitState in party.active_members():
		var c: Combatant = build(m)
		if c != null:
			out.append(c)
	return out
