---
class: FoundationTestSuite
extends: RefCounted
description: "Tactical RPG - Foundation Test Suite (slice A, before the M2c-2b skill content) 1. Skill potency hook: Combatant.potency_for / potency_bonus, the `skill_potency` passive effect, and that the multiplier enters ONLY through the skill's magnitude (power_mult input / heal), never DamageCalculator.compute. With no bonus behaviour is identical. 2. The `unarmed` weapon_types token (no main-hand weapon; proficiency = the class's fist level). 3. Derived weapon hands (main_1h / main_2h / off) and the off-hand attack of a 2H wielder. Expectations come from the rules and catalog data, not magic numbers."
---

# Properties
* `TAG`: String = "Foundation" (const)
* `HAND_TABLE`: Dictionary = { (const)
* `RETYPED`: Array[String] = ["wpn_shortsword", "wpn_knuckles", "wpn_dagger"] (const)

# Methods

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING FOUNDATION TEST (slice A)               ===")
print("=======================================================")
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
var ok: bool = true
ok = _test_potency_defaults() and ok
ok = _test_potency_passive() and ok
ok = _test_potency_damage() and ok
ok = _test_potency_heal() and ok
ok = _test_unarmed_token() and ok
ok = _test_unarmed_validation() and ok
ok = _test_hand_table() and ok
ok = _test_hand_validation_and_preview() and ok
ok = _test_two_handed_offhand() and ok
return ok
```

### `static _test_potency_defaults() -> bool`
```logic
var checks: Dictionary = {}
var c := Combatant.new("Plain", 100, 10, 1)
checks["no bonuses by default"] = c.potency_bonus.is_empty()
checks["potency_for = 1.0 for any family"] = is_equal_approx(c.potency_for("strike"), 1.0) and is_equal_approx(c.potency_for("no_such_family"), 1.0)
var same: bool = true
for s: SkillDef in SkillCatalog.all():
	same = same and is_equal_approx(s.power_for(c), s.power_mult) and is_equal_approx(s.power_for(null), s.power_mult)
checks["power_for(unit) = power_mult for every skill when no bonus"] = same
c.potency_bonus["strike"] = 0.5
checks["potency_for = 1.0 + bonus"] = is_equal_approx(c.potency_for("strike"), 1.5) and is_equal_approx(c.potency_for("guard"), 1.0)
checks["power_for multiplies the skill's own multiplier"] = is_equal_approx(SkillCatalog.find("offhand_jab").power_for(c), SkillCatalog.find("offhand_jab").power_mult) and is_equal_approx(SkillCatalog.find("strike").power_for(c), SkillCatalog.find("strike").power_mult * 1.5)
return TestHelpers.report(TAG, "Potency defaults: no bonus changes nothing", checks)
```

### `static _test_potency_passive() -> bool`
```logic
var checks: Dictionary = {}
var skill_exists: Callable = func(id: String) -> bool: return SkillCatalog.find(id) != null
var good: Array = [{"type": "skill_potency", "family": "strike", "value": 0.25}]
checks["a valid skill_potency validates"] = Passives.validate_effects(good, skill_exists).is_empty()
checks["skill_potency is a listed type"] = Passives.TYPES.has("skill_potency")
checks["unknown family refused"] = not Passives.validate_effects([{"type": "skill_potency", "family": "nope", "value": 0.25}], skill_exists).is_empty()
checks["zero refused"] = not Passives.validate_effects([{"type": "skill_potency", "family": "strike", "value": 0.0}], skill_exists).is_empty()
checks["above the maximum refused"] = not Passives.validate_effects([{"type": "skill_potency", "family": "strike", "value": Passives.POTENCY_MAX + 0.5}], skill_exists).is_empty()
checks["below the minimum refused"] = not Passives.validate_effects([{"type": "skill_potency", "family": "strike", "value": Passives.POTENCY_MIN - 0.05}], skill_exists).is_empty()
checks["a penalty within range validates"] = Passives.validate_effects([{"type": "skill_potency", "family": "guard", "value": -0.2}], skill_exists).is_empty()
var entry: Dictionary = {"id": "t_pot_pass", "name": "Keen", "effects": good}
checks["the catalog accepts the passive"] = ContentCatalog.validate_entry("passives", entry).is_empty()
checks["describe mentions the family and percent"] = Passives.describe(entry).contains("+25%") and Passives.describe(entry).contains("strike")
var c := Combatant.new("Keen", 100, 10, 1)
Passives.apply(c, entry)
checks["apply adds to potency_bonus"] = is_equal_approx(DictUtil.float_of(c.potency_bonus, "strike"), 0.25) and is_equal_approx(c.potency_for("strike"), 1.25)
Passives.apply(c, entry)
checks["the same passive applies once"] = is_equal_approx(c.potency_for("strike"), 1.25)
Passives.apply(c, {"id": "t_pot_pass2", "name": "Keener", "effects": [{"type": "skill_potency", "family": "strike", "value": 0.1}]})
checks["different passives add up"] = is_equal_approx(c.potency_for("strike"), 1.35)
return TestHelpers.report(TAG, "skill_potency passive: validate, describe, apply", checks)
```

### `static _test_potency_damage() -> bool`
```logic
var checks: Dictionary = {}
var bonus: float = 0.25
var strike: SkillDef = SkillCatalog.find("strike")
var grid: CombatGrid = CoreRulesTestSuite._grid()
var keen := Combatant.new("Keen", 200, 12, 1)
var plain := Combatant.new("Plain", 200, 12, 1)
var keen_foe := Combatant.new("KeenFoe", 200, 10, 1)
var plain_foe := Combatant.new("PlainFoe", 200, 10, 1)
grid.place_combatant(keen, Vector2i(3, 4))
grid.place_combatant(keen_foe, Vector2i(4, 4))
grid.place_combatant(plain, Vector2i(3, 6))
grid.place_combatant(plain_foe, Vector2i(4, 6))
var allies: Array[Combatant] = [keen, plain]
var foes: Array[Combatant] = [keen_foe, plain_foe]
var ctx: PlanContext = TestHelpers.hand_ctx(grid, allies, foes)
Passives.apply(keen, {"id": "t_pot_dmg", "name": "Keen", "effects": [{"type": "skill_potency", "family": strike.family_id(), "value": bonus}]})
var resolver: CombatResolver = CoreRulesTestSuite._commit(ctx, {
	keen: [PlanStep.engage(strike, keen_foe)], plain: [PlanStep.engage(strike, plain_foe)],
})
var crits: Dictionary = {}
var mults: Dictionary = {}
for b: int in range(1, resolver.total_blocks + 1):
	resolver.execute_block(b)
for sa: CombatResolver.ScheduledAction in resolver.scheduled_actions:
	if sa.command is AttackCommand:
		var atk: AttackCommand = sa.command
		crits[sa.combatant] = atk.last_crit
		mults[sa.combatant] = atk.skill_mult
var power: int = ctx.power_of(keen)
var crit_k: float = DamageCalculator.CRIT_MULT if DictUtil.to_bool(crits.get(keen, false)) else 1.0
var crit_p: float = DamageCalculator.CRIT_MULT if DictUtil.to_bool(crits.get(plain, false)) else 1.0
var expected_keen: int = maxi(1, roundi(float(power) * (1.0 + bonus) * strike.power_mult * crit_k))
var expected_plain: int = maxi(1, roundi(float(power) * strike.power_mult * crit_p))
checks["the planned command carries power_mult x potency"] = is_equal_approx(DictUtil.to_float(mults.get(keen, 0.0)), strike.power_mult * (1.0 + bonus)) and is_equal_approx(DictUtil.to_float(mults.get(plain, 0.0)), strike.power_mult)
checks["a keen strike deals the +25% hit"] = 200 - keen_foe.hp == expected_keen
checks["an unmodified strike is unchanged"] = 200 - plain_foe.hp == expected_plain
checks["the keen strike hits harder"] = (200 - keen_foe.hp) > (200 - plain_foe.hp) or crit_k < crit_p
# DamageCalculator.compute is untouched: the estimate feeds it the potency-scaled skill_mult (strike.power_for).
checks["the planner estimate uses the same input"] = DamageCalculator.expected(keen, keen_foe, strike) == DictUtil.to_int(DamageCalculator.compute(keen, keen_foe, false, false, strike.power_for(keen), false, null, strike.stance(), false)["final_dmg"])
return TestHelpers.report(TAG, "Passive potency scales strike damage through the skill multiplier only", checks)
```

### `static _test_potency_heal() -> bool`
```logic
var checks: Dictionary = {}
var mend: SkillDef = SkillCatalog.find("mend")
var priest: Combatant = ContentFactory.unit({"id": "t_fd_priest", "name": "P", "class_id": "priest", "level": 5})
# Gear may carry its own potency (e.g. the priest's off-hand tome); start from a bare potency table.
priest.potency_bonus.clear()
priest.potency_group.clear()
priest.gear_potency_group.clear()
var stat_value: int = priest.stats.get_effective_stat(mend.heal_stat, priest.get_weapon_context())
var base_heal: int = mend.heal_base + roundi(float(stat_value) * mend.heal_mult)
checks["no bonus: heal = heal_base + round(stat x heal_mult)"] = SupportRules.heal_amount(priest, mend) == base_heal and base_heal > 0
priest.potency_bonus[mend.family_id()] = 0.5
checks["+50% potency: heal x1.5"] = SupportRules.heal_amount(priest, mend) == roundi(float(base_heal) * 1.5)
priest.potency_bonus[mend.family_id()] = -0.5
checks["a penalty lowers it"] = SupportRules.heal_amount(priest, mend) == roundi(float(base_heal) * 0.5)
priest.potency_bonus.clear()
checks["clearing restores the base heal"] = SupportRules.heal_amount(priest, mend) == base_heal
return TestHelpers.report(TAG, "Potency scales heals too", checks)
```

### `static _brawler() -> Combatant`
```logic
return ContentFactory.unit({"id": "t_fd_brawler", "name": "B", "class_id": "brawler", "level": 5})
```

### `static _test_unarmed_token() -> bool`
```logic
var checks: Dictionary = {}
var body_blow: SkillDef = SkillCatalog.find("body_blow")
var haymaker: SkillDef = SkillCatalog.find("haymaker")
checks["the token is 'unarmed' and trains with 'fist'"] = SkillDef.UNARMED_TOKEN == "unarmed" and SkillDef.UNARMED_PROFICIENCY == "fist"
checks["body_blow and haymaker list the token"] = body_blow.weapon_types.has("unarmed") and haymaker.weapon_types.has("unarmed")
var fighter: Combatant = _brawler()
fighter.inventory.unequip(Equipment.EquipSlot.MAIN_HAND)
checks["no main hand reads as unarmed"] = fighter.wielded_type() == SkillDef.UNARMED_TOKEN
var fist_level: int = fighter.stats.proficiency.level_of("fist")
checks["the brawler is trained in fist"] = fist_level >= haymaker.min_proficiency and fist_level >= body_blow.min_proficiency
checks["an unarmed brawler can use body_blow and haymaker"] = fighter.can_use(body_blow) and fighter.can_use(haymaker)
# A non-fist weapon that the skill does not list blocks it.
var sword: Equipment = ItemCatalog.create("wpn_broadsword") as Equipment
fighter.inventory.equip(sword)
checks["wielding an unlisted weapon type blocks it"] = fighter.wielded_type() == "sword" and not body_blow.weapon_types.has("sword") and not fighter.can_use(body_blow)
var mace: Equipment = ItemCatalog.create("wpn_flanged_mace") as Equipment
fighter.inventory.equip(mace)
checks["a listed weapon type (mace) still works"] = body_blow.weapon_types.has("mace") and fighter.can_use(body_blow)
# The token is optional: a skill without it never works bare-handed.
var gated := SkillDef.new()
gated.id = "t_fd_gated"
gated.display_name = "Gated"
gated.weapon_types = ["sword"]
gated.min_proficiency = 1
var rogue: Combatant = ContentFactory.unit({"id": "t_fd_rogue", "name": "R", "class_id": "rogue", "level": 5})
rogue.learn_skill(gated.id)
rogue.inventory.unequip(Equipment.EquipSlot.MAIN_HAND)
checks["a skill that does not list unarmed refuses bare hands"] = rogue.knows_skill(gated.id) and not rogue.can_use(gated)
# min_proficiency applies to unarmed through the fist level.
var deep := SkillDef.new()
deep.id = "t_fd_deep"
deep.display_name = "Deep"
deep.weapon_types = ["unarmed"]
deep.min_proficiency = 3
var shallow := SkillDef.new()
shallow.id = "t_fd_shallow"
shallow.display_name = "Shallow"
shallow.weapon_types = ["unarmed"]
shallow.min_proficiency = 1
rogue.learn_skill(deep.id)
rogue.learn_skill(shallow.id)
var rogue_fist: int = rogue.stats.proficiency.level_of("fist")
checks["min_proficiency uses the class's fist level (rogue)"] = rogue_fist >= 1 and rogue_fist < 3 and rogue.can_use(shallow) and not rogue.can_use(deep)
fighter.inventory.unequip(Equipment.EquipSlot.MAIN_HAND)
fighter.learn_skill(deep.id)
checks["a fist-3 class meets min_proficiency 3 unarmed"] = fighter.stats.proficiency.level_of("fist") >= 3 and fighter.can_use(deep)
var priest: Combatant = ContentFactory.unit({"id": "t_fd_priest2", "name": "P", "class_id": "priest", "level": 5})
priest.inventory.unequip(Equipment.EquipSlot.MAIN_HAND)
priest.learn_skill(shallow.id)
checks["a class with no fist training cannot use an unarmed skill"] = priest.stats.proficiency.level_of("fist") < 1 and not priest.can_use(shallow)
var hand_built := Combatant.new("Hand", 50, 10, 1)
checks["hand-built units stay unrestricted"] = hand_built.can_use(body_blow)
return TestHelpers.report(TAG, "The unarmed token gates skills by the empty main hand and the fist level", checks)
```

### `static _test_unarmed_validation() -> bool`
```logic
var checks: Dictionary = {}
var base: Dictionary = {"id": "t_fd_skill", "name": "U", "kind": "ATTACK", "min_proficiency": 1}
var ok_entry: Dictionary = base.duplicate(true)
ok_entry["weapon_types"] = ["fist", "unarmed"]
checks["'unarmed' validates beside gear types"] = ContentCatalog.validate_entry("skills", ok_entry).is_empty()
var bad: Dictionary = base.duplicate(true)
bad["weapon_types"] = ["unarmed", "banjo"]
var errors: Array[String] = ContentCatalog.validate_entry("skills", bad)
checks["an unknown token is refused"] = not errors.is_empty() and ", ".join(errors).contains("banjo")
checks["the allowed list is the gear types plus 'unarmed'"] = SkillDef.weapon_type_tokens().has("unarmed") and SkillDef.weapon_type_tokens().has("fist") and SkillDef.weapon_type_tokens().size() == Proficiency.gear_types().size() + 1
checks["the schema help documents the token"] = str(ContentSchema.field("skills", "weapon_types").get("help", "")).contains("unarmed")
var entry: Dictionary = ContentSchema.normalize("skills", ok_entry)
var back: Dictionary = ContentFactory.skill_to_entry(ContentFactory.skill(entry))
checks["the token survives the factory round trip"] = back == entry
var preview: String = ForgePreview.describe("skills", ContentCatalog.get_entry("skills", "body_blow"))
checks["the Forge preview explains unarmed"] = preview.contains("unarmed") and preview.contains("fist")
return TestHelpers.report(TAG, "Unarmed token: validation, round trip, Forge text", checks)
```

### `static _expected_hand(id: String) -> String`
```logic
if RETYPED.has(id) and not ContentCatalog.items_inherit():
	return Equipment.HAND_OFF
return str(HAND_TABLE[id])
```

### `static _test_hand_table() -> bool`
```logic
var checks: Dictionary = {}
var table_ok: bool = true
var missing: Array[String] = []
for id: Variant in HAND_TABLE:
	var item: Item = ItemCatalog.create(str(id))
	if item == null:
		missing.append(str(id))
		continue
	table_ok = table_ok and item.hand() == _expected_hand(str(id))
checks["the table covers existing items"] = missing.is_empty()
checks["every tabled item has the expected hand"] = table_ok
var rule_ok: bool = true
var slot_ok: bool = true
var covered: bool = true
for e: Dictionary in ContentCatalog.entries("items"):
	var item: Item = ContentFactory.item(e)
	if not (item is Equipment):
		rule_ok = rule_ok and item.hand().is_empty()
		continue
	var eq: Equipment = item
	covered = covered and HAND_TABLE.has(eq.id)
	var h: String = eq.hand()
	if eq.is_shield() or (eq.is_weapon() and eq.is_off_hand_weapon()):
		rule_ok = rule_ok and h == Equipment.HAND_OFF
	elif eq.is_weapon() and eq.is_two_handed:
		rule_ok = rule_ok and h == Equipment.HAND_MAIN_2H
	elif eq.is_weapon():
		rule_ok = rule_ok and h == Equipment.HAND_MAIN_1H
	else:
		rule_ok = rule_ok and h.is_empty()
	# The hand agrees with where the item may be equipped.
	var in_main: bool = eq.can_equip_in_slot(Equipment.EquipSlot.MAIN_HAND)
	var in_off: bool = eq.can_equip_in_slot(Equipment.EquipSlot.OFF_HAND)
	match h:
		Equipment.HAND_MAIN_2H:
			slot_ok = slot_ok and in_main and not in_off
		Equipment.HAND_OFF:
			slot_ok = slot_ok and in_off and not in_main
		Equipment.HAND_MAIN_1H:
			slot_ok = slot_ok and in_main and in_off == Equipment.allow_plain_dual_wield
checks["the hand follows shield / off-hand category / two-handed flags"] = rule_ok
checks["the hand agrees with can_equip_in_slot"] = slot_ok
var broadsword: Equipment = ItemCatalog.create("wpn_broadsword") as Equipment
var saved: bool = Equipment.allow_plain_dual_wield
Equipment.allow_plain_dual_wield = false
var strict_off: bool = broadsword.can_equip_in_slot(Equipment.EquipSlot.OFF_HAND)
Equipment.allow_plain_dual_wield = true
var legacy_off: bool = broadsword.can_equip_in_slot(Equipment.EquipSlot.OFF_HAND)
Equipment.allow_plain_dual_wield = saved
checks["flag false: a plain main_1h weapon is refused in the off hand"] = not strict_off and broadsword.hand() == Equipment.HAND_MAIN_1H
checks["flag true (legacy dual wield): it fits the off hand again"] = legacy_off
checks["every catalog item is in the table (add new ones)"] = covered
checks["HANDS lists the three hands"] = Equipment.HANDS == ["main_1h", "main_2h", "off"]
var potion: Item = ItemCatalog.create("potion_healing") if ContentCatalog.has("items", "potion_healing") else null
checks["consumables are not held"] = potion == null or potion.hand().is_empty()
return TestHelpers.report(TAG, "Derived hand for every current item", checks)
```

### `static _test_hand_validation_and_preview() -> bool`
```logic
var checks: Dictionary = {}
var claymore: Dictionary = ContentCatalog.get_entry("items", "wpn_claymore")
var off_slot: Dictionary = claymore.duplicate(true)
off_slot["id"] = "t_fd_offslot"
off_slot["slot"] = "OFF_HAND"
if ContentCatalog.items_inherit():
	# Families: the hand is one of the family's hands (a stored main_2h on a family without it is the error);
	# a weapon's slot follows its hand, so an OFF_HAND slot on a two-handed row resolves back to MAIN_HAND.
	var no_2h: String = ""
	for f: Dictionary in ContentCatalog.entries("item_families"):
		var hands: Array[String] = ItemFamilies.hands_of(f)
		if ItemFamilies.item_type_of(f) == "WEAPON" and ItemFamilies.category(f) == "main" and not hands.has(ItemFamilies.HAND_MAIN_2H):
			no_2h = str(f["id"])
			break
	var wrong: Dictionary = {"id": "t_fd_conflict", "name": "Conflict", "family": no_2h, "hand": ItemFamilies.HAND_MAIN_2H, "modifiers": [], "tiers": []}
	var errors: Array[String] = ContentCatalog.validate_entry("items", wrong)
	checks["main_2h on a family without it is refused (hand not allowed by family)"] = not no_2h.is_empty() and TestHelpers.any_contains(errors, "not allowed by family")
	var family: Dictionary = ContentCatalog.family_of(claymore)
	checks["a two-handed weapon's slot follows its hand (MAIN_HAND)"] = str(ItemModifiers.resolve(ItemModifiers.collapse(ContentSchema.normalize("items", off_slot), family), family)["slot"]) == "MAIN_HAND"
else:
	var conflict: Dictionary = claymore.duplicate(true)
	conflict["id"] = "t_fd_conflict"
	conflict["off_hand_category"] = "tome"
	var errors: Array[String] = ContentCatalog.validate_entry("items", conflict)
	checks["legacy row: main_2h + an off-hand category is a hand conflict"] = not errors.is_empty() and ", ".join(errors).contains("Hand conflict")
	checks["a two-handed weapon outside the main hand is still refused"] = not ContentCatalog.validate_entry("items", off_slot).is_empty()
checks["the shipped items validate"] = ContentCatalog.validate_entry("items", claymore, "wpn_claymore").is_empty()
var text: String = ForgePreview.describe("items", claymore)
checks["the Forge item preview shows the hand"] = text.contains("main_2h") and text.contains("Hand")
checks["an off-hand item previews as off"] = ForgePreview.describe("items", ContentCatalog.get_entry("items", "shd_kite")).contains("off hand")
checks["an engine item exported to an entry leaves the hand to the family (derived from its flags on upsert)"] = str(ContentFactory.item_to_entry(ItemCatalog.create("wpn_claymore")).get("hand", "")).is_empty()
return TestHelpers.report(TAG, "Hand consistency rule and Forge preview", checks)
```

### `static _test_two_handed_offhand() -> bool`
```logic
var checks: Dictionary = {}
var warrior: Combatant = ContentFactory.unit({"id": "t_fd_warrior", "name": "W", "class_id": "warrior", "level": 5})
var claymore: Equipment = ItemCatalog.create("wpn_claymore") as Equipment
warrior.inventory.equip(claymore)
warrior.inventory.unequip(Equipment.EquipSlot.OFF_HAND)
checks["the warrior holds a main_2h weapon with an empty off hand"] = claymore.hand() == Equipment.HAND_MAIN_2H and warrior.inventory.get_main_weapon() == claymore and warrior.inventory.get_offhand_weapon() == null
var power: Dictionary = DamageCalculator.attack_power(warrior, false, true)
var str_val: int = warrior.stats.get_effective_stat("STR", {})
var dex_val: int = warrior.stats.get_effective_stat("DEX", {})
checks["off-hand attack uses bare hands: STR+DEX"] = str(power["stat_name"]) == "STR+DEX" and power["weapon"] == null
checks["off-hand unit power = STR + DEX, weapon power 0"] = DictUtil.to_float(power["unit_power"]) == float(str_val + dex_val) and DictUtil.to_float(power["weapon_power"]) == 0.0
var main_power: Dictionary = DamageCalculator.attack_power(warrior, false, false)
checks["the main-hand attack still uses the claymore"] = main_power["weapon"] == claymore and str(main_power["stat_name"]) != "STR+DEX"
# Contrast: a one-handed main with an off-hand weapon swings that weapon instead.
# The dagger is a main_1h weapon under strict hand typing: holding it off-hand is the legacy dual-wield case (flag).
var saved: bool = Equipment.allow_plain_dual_wield
Equipment.allow_plain_dual_wield = true
var dagger: Equipment = ItemCatalog.create("wpn_dagger") as Equipment
var rogue: Combatant = ContentFactory.unit({"id": "t_fd_rogue2", "name": "R", "class_id": "rogue", "level": 5})
rogue.inventory.equip(dagger, Equipment.EquipSlot.OFF_HAND)
var off_power: Dictionary = DamageCalculator.attack_power(rogue, false, true)
checks["an off-hand dagger is swung with its own scaling"] = off_power["weapon"] == dagger and str(off_power["stat_name"]) != "STR+DEX"
Equipment.allow_plain_dual_wield = saved
return TestHelpers.report(TAG, "Off-hand attack of a two-handed wielder scales from bare hands", checks)
```

