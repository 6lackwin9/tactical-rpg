---
class: ItemUseTestSuite
extends: RefCounted
description: "Tactical RPG - Item use test suite (ITEMUSE1) Stage 1 (schema): UnitState.pouch (serialisation, validation, PartyBuilder fill), save v8 (step 7 -> 8, fixtures), the consumable `use_reach` field (type defaults, validation, factory round trip, battle-offer warning), the ITEM plan step shape and the AiProfile `use_items` switch. Test items live in a sandbox catalog fork; expectations come from the rules (Consumable.TYPE_REACH, UnitState.POUCH_MAX, the save fixtures), never from shipped numbers. Stage 2 (behaviour) lives in ItemUseBattleChecks (item_use_battle_checks.gd)."
---

# Properties
* `TAG`: String = "ItemUse" (const)
* `POTION`: String = "t_iu_potion" (const)
* `BOMB`: String = "t_iu_bomb" (const)
* `ELIXIR`: String = "t_iu_elixir" (const)
* `BREW`: String = "t_iu_brew" (const)
* `errors`: Array[String] = []
* `out`: Array[String] = []

# Methods

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING ITEM USE TEST (ITEMUSE1)                ===")
print("=======================================================")
var ok: bool = true
ok = _test_type_reach() and ok
ok = _test_plan_step() and ok
ok = _test_ai_switch() and ok
ok = _test_saves() and ok
ok = ItemUseBattleChecks.run() and ok   # stage 2: options, resolution, reset, AI, sim key, write-back, gear draft
if not TestHelpers.fork(SANDBOX):
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "sandbox fork", {"forked": false})
var seeded: Array[String] = _seed_items()
ok = TestHelpers.report(TAG, "sandbox consumables", {"seeded": seeded.is_empty()}) and ok
if not seeded.is_empty():
	print("  seed errors: %s" % str(seeded))
else:
	ok = _test_use_reach_field() and ok
	ok = _test_unit_pouch() and ok
	ok = _test_party_builder() and ok
TestHelpers.cleanup(SANDBOX)
return ok
```

### `static _test_type_reach() -> bool`
```logic
var c: Dictionary = {}
c["potion reach 1 (self or adjacent ally)"] = Consumable.reach_for(Consumable.ConsumableType.HEALTH_POTION) == 1
c["elixir reach 0 (self)"] = Consumable.reach_for(Consumable.ConsumableType.FOCUS_INCENSE) == 0
c["bomb reach 3"] = Consumable.reach_for(Consumable.ConsumableType.THROWABLE_BOMB) == 3
for t: Consumable.ConsumableType in [Consumable.ConsumableType.STAMINA_BREW, Consumable.ConsumableType.ANTIDOTE]:
	c["%s not offered" % Consumable.ConsumableType.find_key(t)] = not Consumable.offered_in_battle(t) and Consumable.reach_for(t) == Consumable.REACH_TYPE_DEFAULT
for t: int in Consumable.TYPE_REACH:
	var reach: int = DictUtil.to_int(Consumable.TYPE_REACH[t])
	c["type %d default inside 0..REACH_MAX" % t] = reach >= 0 and reach <= Consumable.REACH_MAX
var bomb := Consumable.new("x", "x", Consumable.ConsumableType.THROWABLE_BOMB)
c["new consumable reads the type default"] = bomb.use_reach == Consumable.REACH_TYPE_DEFAULT and bomb.reach() == Consumable.reach_for(Consumable.ConsumableType.THROWABLE_BOMB)
bomb.use_reach = 0
c["own use_reach wins"] = bomb.reach() == 0
return TestHelpers.report(TAG, "type reach defaults", c)
```

### `static _reach_of(it: Item) -> int`
```logic
if it is Consumable:
	var con: Consumable = it
	return con.reach()
return -99
```

### `static _test_use_reach_field() -> bool`
```logic
var c: Dictionary = {}
var f: Dictionary = ContentSchema.field("items", "use_reach")
c["field exists"] = not f.is_empty() and str(f.get("type", "")) == "int"
c["default = type default"] = DictUtil.to_int(f.get("default", 99)) == Consumable.REACH_TYPE_DEFAULT
var bomb_row: Dictionary = ContentCatalog.get_entry("items", BOMB)
c["consumable row carries use_reach -1"] = DictUtil.to_int(bomb_row.get("use_reach", 99)) == Consumable.REACH_TYPE_DEFAULT
c["visible on consumables"] = ContentSchema.is_visible(f, bomb_row)
c["hidden on gear"] = not ContentSchema.is_visible(f, {"item_type": "WEAPON"})
var built: Item = ContentFactory.item(bomb_row)
c["built bomb reach = type default"] = _reach_of(built) == Consumable.reach_for(Consumable.ConsumableType.THROWABLE_BOMB)
for v: int in [Consumable.REACH_TYPE_DEFAULT, 0, Consumable.REACH_MAX]:
	var row: Dictionary = ContentCatalog.get_entry("items", BOMB)
	row["use_reach"] = v
	var errs: Array[String] = ContentCatalog.upsert("items", row)
	c["use_reach %d accepted" % v] = errs.is_empty() and DictUtil.to_int(ContentCatalog.get_entry("items", BOMB).get("use_reach", 99)) == v
var explicit: Item = ContentFactory.item(ContentCatalog.get_entry("items", BOMB))
c["built bomb keeps its own reach"] = _reach_of(explicit) == Consumable.REACH_MAX
c["to_entry round-trips use_reach"] = DictUtil.to_int(ContentFactory.item_to_entry(explicit).get("use_reach", 99)) == Consumable.REACH_MAX
# Out-of-range values are clamped into -1..REACH_MAX by the schema (like every int field), never stored as given.
var clamp_cases: Dictionary = {Consumable.REACH_TYPE_DEFAULT - 3: Consumable.REACH_TYPE_DEFAULT, Consumable.REACH_MAX + 5: Consumable.REACH_MAX}
for bad: int in clamp_cases:
	var row: Dictionary = ContentCatalog.get_entry("items", BOMB)
	row["use_reach"] = bad
	var errs: Array[String] = ContentCatalog.upsert("items", row)
	c["use_reach %d clamped" % bad] = errs.is_empty() and DictUtil.to_int(ContentCatalog.get_entry("items", BOMB).get("use_reach", 99)) == DictUtil.to_int(clamp_cases[bad])
c["not-offered type warns"] = TestHelpers.any_contains(ContentCatalog.warnings_for("items", ContentCatalog.get_entry("items", BREW)), "not offered in battle")
c["offered type does not warn"] = not TestHelpers.any_contains(ContentCatalog.warnings_for("items", ContentCatalog.get_entry("items", POTION)), "not offered in battle")
c["catalog still valid"] = ContentCatalog.validate_all().is_empty()
return TestHelpers.report(TAG, "use_reach field", c)
```

### `static _party_base() -> String`
```logic
for e: Dictionary in ContentCatalog.entries("units"):
	var id: String = str(e.get("id", ""))
	if str(e.get("side", "")) == "party" and UnitState.create("t_iu_hero", id).validate().is_empty():
		return id
return ""
```

### `static _non_consumable() -> String`
```logic
for e: Dictionary in ContentCatalog.entries("items"):
	if str(e.get("item_type", "")) != "CONSUMABLE":
		return str(e.get("id", ""))
return ""
```

### `static _test_unit_pouch() -> bool`
```logic
var c: Dictionary = {}
var base: String = _party_base()
c["party base unit found"] = not base.is_empty()
if base.is_empty():
	return TestHelpers.report(TAG, "unit pouch", c)
c["POUCH_MAX = the inventory pouch"] = UnitState.POUCH_MAX == Inventory.new().max_pouch_slots
var s: UnitState = UnitState.create("t_iu_hero", base)
c["new state has an empty pouch"] = s.pouch.is_empty() and s.to_dict().has("pouch")
var legacy: Dictionary = s.to_dict()
legacy.erase("pouch")
c["missing key reads []"] = UnitState.from_dict(legacy).pouch.is_empty()
var full: Array[String] = []
for i: int in range(UnitState.POUCH_MAX):
	full.append(POTION if i < UnitState.POUCH_MAX - 1 else BOMB)
s.pouch = full.duplicate()
c["full pouch with repeats valid"] = s.validate().is_empty()
var back: UnitState = UnitState.from_dict(s.to_dict())
c["round-trip keeps order and repeats"] = back.pouch == full and back.equals(s)
var over: UnitState = s.duplicate_state()
over.pouch.append(ELIXIR)
c["over the limit refused"] = TestHelpers.any_contains(over.validate(), "Pouch holds at most %d items." % UnitState.POUCH_MAX)
var gear_id: String = _non_consumable()
var wrong: UnitState = s.duplicate_state()
wrong.pouch = [gear_id] as Array[String]
c["non-consumable refused"] = not gear_id.is_empty() and TestHelpers.any_contains(wrong.validate(), "Pouch: '%s' is not a consumable." % gear_id)
wrong.pouch = ["t_iu_no_such_item"] as Array[String]
c["unknown id refused"] = TestHelpers.any_contains(wrong.validate(), "Pouch: 't_iu_no_such_item' is not a consumable.")
return TestHelpers.report(TAG, "unit pouch", c)
```

### `static _test_party_builder() -> bool`
```logic
var c: Dictionary = {}
var base: String = _party_base()
var s: UnitState = UnitState.create("t_iu_hero", base)
var bare: Combatant = PartyBuilder.build(s)
c["bare state builds"] = bare != null
if bare == null:
	return TestHelpers.report(TAG, "party builder pouch", c)
var before: Array[String] = _pouch_ids(bare)
s.pouch = [BOMB, POTION] as Array[String]
var filled: Combatant = PartyBuilder.build(s)
var expected: Array[String] = before.duplicate()
for id: String in s.pouch:
	if expected.size() < bare.inventory.max_pouch_slots:
		expected.append(id)
c["pouch ids follow the unit-row pouch, in slot order"] = filled != null and _pouch_ids(filled) == expected
c["pouch items are not in the bag"] = filled != null and filled.inventory.items.size() == bare.inventory.items.size()
c["the state is not changed"] = s.pouch == ([BOMB, POTION] as Array[String])
return TestHelpers.report(TAG, "party builder pouch", c)
```

### `static _members_of(game_state: Dictionary) -> Array`
```logic
return DictUtil.array_of(DictUtil.dict_of(game_state.get("party", {})).get("members", []))
```

### `static _test_saves() -> bool`
```logic
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
SaveMigrations.reset()
var c: Dictionary = {}
c["version 8"] = SaveGame.VERSION >= 8
var v7_text: String = FileAccess.get_file_as_string(SaveCompatTestSuite.fixture_path(7))
var v7: Dictionary = DictUtil.dict_of(JSON.parse_string(v7_text))
var mig: Dictionary = SaveMigrations.migrate(v7, 8)
var mig_state: Dictionary = DictUtil.dict_of(DictUtil.dict_of(mig["data"]).get("game_state", {}))
var members: Array = _members_of(mig_state)
var all_empty: bool = not members.is_empty()
for raw: Variant in members:
	var m: Dictionary = DictUtil.dict_of(raw)
	all_empty = all_empty and m.has("pouch") and DictUtil.array_of(m["pouch"]).is_empty()
c["7 -> 8 gives every member pouch []"] = DictUtil.to_bool(mig["ok"]) and all_empty
var stripped: Dictionary = mig_state.duplicate(true)
for raw: Variant in _members_of(stripped):
	DictUtil.dict_of(raw).erase("pouch")
c["7 -> 8 changes nothing else"] = JsonUtil.deep_equal(stripped, DictUtil.dict_of(v7.get("game_state", {})))
var no_party: Dictionary = SaveMigrations._v7_to_v8({"party": "broken"})
c["7 -> 8 tolerates a non-object party"] = str(no_party.get("party", "")) == "broken"
for version: int in [1, 7]:
	var res: Dictionary = SaveGame.decode(FileAccess.get_file_as_string(SaveCompatTestSuite.fixture_path(version)))
	var g: GameState = GameState.of(res["state"])
	var empty: bool = g != null and not g.party.members.is_empty()
	if g != null:
		for m: UnitState in g.party.members:
			empty = empty and m.pouch.is_empty()
	c["v%d loads with empty pouches" % version] = DictUtil.to_bool(res["ok"]) and empty
var v8_res: Dictionary = SaveGame.decode(FileAccess.get_file_as_string(SaveCompatTestSuite.fixture_path(8)))
var g8: GameState = GameState.of(v8_res["state"])
var stocked: bool = false
if g8 != null:
	for m: UnitState in g8.party.members:
		stocked = stocked or not m.pouch.is_empty()
c["v8 fixture loads valid with a stocked pouch"] = DictUtil.to_bool(v8_res["ok"]) and g8 != null and g8.validate().is_empty() and stocked
if g8 != null:
	var again: Dictionary = SaveGame.decode(JSON.stringify({"format": SaveGame.FORMAT, "version": SaveGame.VERSION, "saved_at": 1, "game_state": g8.to_dict()}))
	var g8b: GameState = GameState.of(again["state"])
	c["v8 round-trips the pouch"] = DictUtil.to_bool(again["ok"]) and g8b != null and g8.equals(g8b) and g8b.party.members[0].pouch == g8.party.members[0].pouch
return TestHelpers.report(TAG, "save v8", c)
```

### `static _test_plan_step() -> bool`
```logic
var c: Dictionary = {}
var user := Combatant.new("Tester", 10, 10)
var step: PlanStep = PlanStep.use_item("con_x", user)
c["mode ITEM, skill-less"] = step.mode == PlanStep.Mode.ITEM and step.skill == null and step.item_id == "con_x" and step.target == user
c["ITEM is the last mode (recorded ints unchanged)"] = PlanStep.Mode.ITEM == PlanStep.Mode.size() - 1 and PlanStep.Mode.ALLY == PlanStep.Mode.ITEM - 1
var copy: PlanStep = step.duplicate_step()
c["duplicate keeps item and target"] = copy.mode == PlanStep.Mode.ITEM and copy.item_id == "con_x" and copy.target == user
var encoded: Dictionary = PlanCodec.encode_step(null, PlanStep.use_item("con_x", null))   # no target: no session lookup
c["codec records the item"] = str(encoded.get("mode", "")) == "ITEM" and str(encoded.get("item", "")) == "con_x"
c["codec leaves skill steps without an item key"] = not PlanCodec.encode_step(null, PlanStep.withdraw()).has("item")
c["title names the target"] = step.title().contains("Tester")
c["codec maps the mode name"] = PlanCodec.mode_of(PlanCodec.mode_name(PlanStep.Mode.ITEM)) == PlanStep.Mode.ITEM
var opt := PlanOption.new()
c["option item_left defaults to 0"] = opt.item_left == 0
c["skill-less option has no family"] = opt.family().is_empty()
return TestHelpers.report(TAG, "plan step shape", c)
```

### `static _test_ai_switch() -> bool`
```logic
var c: Dictionary = {}
var p := AiProfile.new()
c["off by default"] = not p.use_items
c["off in created profiles"] = not AiProfile.create(AiProfile.Role.SUPPORT).use_items
p.use_items = true
c["duplicate copies the switch"] = p.duplicate_profile().use_items
return TestHelpers.report(TAG, "ai use_items switch", c)
```

