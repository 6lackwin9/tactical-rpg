class_name PartyBuilder
extends RefCounted

# ==============================================================================
# Tactical RPG - Party builder (S7): UnitState -> Combatant.
# Goes through the same ContentFactory.unit path as a catalog unit (class, tier, gear), with the
# level from XP and the spend from the state; then passives, learned skills, loadout, equipment.
# Unspent points are auto-spent on a COPY (the state is never changed here); `auto_spend` false leaves them unspent
# (the party screen's preview shows only confirmed and drafted points, UI2-L).
# ==============================================================================

# ITEMUSE1 (open question): the shipped warrior / ranger / priest / mage class rows carry a default pouch (a heal potion).
# false (spec reading "each hero carries a small pouch filled before the fight"): a hero's battle pouch is only its saved
# pouch, so no free potion each battle. true: the saved pouch is added after the class default (the planner's "adds").
const HERO_KEEPS_CLASS_POUCH: bool = false

# {"combatant": Combatant or null, "warnings": Array[String], "errors": Array[String], "pouch": Array[Consumable] (the
# battle_pouch items that came from the state's own pouch, ITEMUSE1: GameEncounter writes the unused ones back),
# "pouch_unloaded": Array[String] (saved pouch ids that found no free slot; kept as they are on write-back)}
static func build_report(state: UnitState, auto_spend: bool = true) -> Dictionary:
	var warnings: Array[String] = []
	var errors: Array[String] = state.validate()
	if not errors.is_empty():
		return {"combatant": null, "warnings": warnings, "errors": errors, "pouch": [] as Array[Consumable], "pouch_unloaded": [] as Array[String]}
	var work: UnitState = state.duplicate_state()
	if auto_spend:
		LevelUp.auto_for(work)
	var entry: Dictionary = ContentCatalog.get_entry("units", work.base_unit).duplicate(true)
	entry["level"] = work.level()
	entry["spend"] = work.spend.duplicate()
	if not auto_spend:
		entry[ContentFactory.NO_AUTO_SPEND] = true   # an all-unspent state must not be auto-spent by the factory either
	var c: Combatant = ContentFactory.unit(entry, display_name(work))   # RECRUIT1-L rule 2: a named hero keeps its name
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
	if not HERO_KEEPS_CLASS_POUCH:
		c.inventory.battle_pouch.clear()   # the class / unit-row default pouch is not a hero's: only the saved pouch counts
	var own: Array[Consumable] = []
	var unloaded: Array[String] = []
	for pouch_id: String in work.pouch:   # ITEMUSE1 rule 1: in slot order (a full battle pouch refuses the rest)
		var pouch_item: Item = ContentFactory.item(ContentCatalog.get_entry("items", pouch_id))
		if pouch_item is Consumable:
			var con: Consumable = pouch_item
			if c.inventory.add_to_battle_pouch(con):
				own.append(con)
			else:
				unloaded.append(pouch_id)
	c.sync_from_stats_and_gear()
	c.hp = c.max_hp
	var cls: Array[String] = JsonUtil.strings(work.loadout.get("class", []))
	var ext: Array[String] = JsonUtil.strings(work.loadout.get("extra", []))
	if not cls.is_empty() or not ext.is_empty():
		var err: String = c.set_loadout(cls, ext)
		if not err.is_empty():
			warnings.append("Unit '%s': stored loadout ignored (%s)." % [work.id, err])
	return {"combatant": c, "warnings": warnings, "errors": errors, "pouch": own, "pouch_unloaded": unloaded}

# The world character a member plays (RECRUIT1-L): `lore_ref` is its qualified id; {} when empty or unresolved.
static func character_of(state: UnitState) -> Dictionary:
	if state == null or state.lore_ref.is_empty():
		return {}
	var parts: Array[String] = WorldCatalog.split_qid(state.lore_ref)
	if parts.size() != 2 or not WorldCatalog.has_world(parts[0]):
		return {}
	return WorldCatalog.resolve(parts[0], "characters", state.lore_ref)

# The name shown for a member (RECRUIT1-L rule 2): its character's name when `lore_ref` resolves, else the base
# unit's name (else the member id). Party, results, level-up, loadout, gear and new-game screens use it.
static func display_name(state: UnitState) -> String:
	var character: Dictionary = character_of(state)
	if not character.is_empty() and not str(character.get("name", "")).is_empty():
		return str(character["name"])
	if state == null:
		return ""
	return str(ContentCatalog.get_entry("units", state.base_unit).get("name", state.id))

# The Combatant of `state`, or null when the state is invalid.
static func build(state: UnitState, auto_spend: bool = true) -> Combatant:
	var res: Dictionary = build_report(state, auto_spend)
	return res["combatant"] as Combatant

# Combatants of the active members, in roster order.
static func build_active(party: PartyState) -> Array[Combatant]:
	var out: Array[Combatant] = []
	for m: UnitState in party.active_members():
		var c: Combatant = build(m)
		if c != null:
			out.append(c)
	return out
