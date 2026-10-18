---
class: ItemCommand
extends: BattleCommand
description: "Tactical RPG - Item Command (ITEMUSE1 rules 3-5): use one battle-pouch consumable on a target. Resolves in the SupportCommand slot (priority 0, at the end of its last block). The item leaves the user's battle_pouch only when the command resolves; a down / withdrawn user, a down target or a target out of `reach` (Consumable.reach(), grid distance from the user) cancels it and the item stays. Effects: heal (Combatant.heal, capped at missing HP); damage as flat round-end-style damage (no armor, guard or stance; not a hit, so DamageCalculator and riposte never see it); attack boost (apply_attack_boost: the Focus slot, replaced, never stacked). The static helpers are shared with CombatPlan (compile, options) and TacticalAI."
---

# Properties
* `ITEM_AP`: int = 1   # rule 3: every item use costs 1 AP (AP debt allowed) (const)
* `target`: Combatant = null
* `item`: Consumable = null
* `grid`: CombatGrid = null
* `used`: bool = false
* `last_heal`: int = 0
* `last_damage`: int = 0
* `last_boost`: float = 0.0
* `not_used_reason`: String = ""
* `out`: Array[String] = []
* `out`: Array[String] = []

# Methods

### `_init(p_source: Combatant, p_target: Combatant, p_item: Consumable, p_grid: CombatGrid, p_block_cost: int = 1) -> void`
```logic
super(p_source, 0, p_block_cost)
target = p_target if p_target != null else p_source
item = p_item
grid = p_grid
command_name = "Use %s" % (p_item.name if p_item != null else "item")
```

### `static pouch_item(c: Combatant, item_id: String) -> Consumable`
```logic
if c == null or c.inventory == null:
	return null
for con: Consumable in c.inventory.battle_pouch:
	if con != null and con.id == item_id:
		return con
return null
```

### `static pouch_count(c: Combatant, item_id: String) -> int`
```logic
var n: int = 0
if c == null or c.inventory == null:
	return n
for con: Consumable in c.inventory.battle_pouch:
	if con != null and con.id == item_id:
		n += 1
return n
```

### `static offered(con: Consumable) -> bool`
```logic
return con != null and Consumable.offered_in_battle(con.consumable_type) and con.reach() >= 0
```

### `static aims_at_foes(con: Consumable) -> bool`
```logic
return con != null and con.damage_amount > 0 and con.heal_amount <= 0 and con.buff_percent <= 0.0
```

### `static side_problem(user: Combatant, con: Consumable, target: Combatant, foes: Array[Combatant]) -> String`
```logic
if target == null:
	return "Pick a target for %s." % con.name
var foe: bool = foes.has(target)
if aims_at_foes(con):
	if not foe:
		return "%s is thrown at a foe." % con.name
elif foe:
	return "%s is used on yourself or an ally." % con.name
if con.reach() == 0 and target != user:
	return "%s is used on yourself only." % con.name
return ""
```

### `static skill_for(con: Consumable) -> SkillDef`
```logic
var s: SkillDef = SkillDef.create("item_" + con.id, con.name, SkillDef.Kind.SUPPORT, 1, maxi(1, con.block_cost), con.description)
s.target_mode = SkillDef.TargetMode.ALLY_OR_SELF
s.system = true
return s
```

### `in_reach() -> bool`
```logic
if target == source:
	return true
if grid == null or not grid.has_combatant(source) or not grid.has_combatant(target):
	return false
return grid.get_distance(grid.get_position(source), grid.get_position(target)) <= item.reach()
```

### `problem() -> String`
```logic
if item == null:
	return "nothing to use"
if not source.is_active():
	return "%s can't act" % source.name
if target == null or not target.is_active():
	return "%s is down" % (target.name if target != null else "the target")
if not in_reach():
	return "%s is out of reach" % target.name
if not source.inventory.battle_pouch.has(item) and pouch_item(source, item.id) == null:
	return "none left in %s's pouch" % source.name
return ""
```

### `execute() -> void`
```logic
used = false
failed = false
last_heal = 0
last_damage = 0
last_boost = 0.0
not_used_reason = problem()
if not not_used_reason.is_empty():
	failed = true
	execution_log = "[Item] %s not used: %s." % [item.name if item != null else "Item", not_used_reason]
	return
var pouch: Array[Consumable] = source.inventory.battle_pouch
var idx: int = pouch.find(item)
if idx < 0:
	idx = pouch.find(pouch_item(source, item.id))
pouch.remove_at(idx)
used = true
var parts: Array[String] = []
if item.heal_amount > 0:
	var before: int = target.hp
	target.heal(item.heal_amount)
	last_heal = target.hp - before
	parts.append("healed %d (HP %d/%d)" % [last_heal, target.hp, target.max_hp])
if item.damage_amount > 0:
	last_damage = target.take_damage(item.damage_amount, true)   # flat: no armor, guard or stance; not a hit
	parts.append("%d damage (HP %d/%d)%s" % [last_damage, target.hp, target.max_hp, "" if target.is_alive() else ", defeated"])
if item.buff_percent > 0.0:
	last_boost = 1.0 + item.buff_percent
	target.apply_attack_boost(last_boost, item.buff_duration_rounds)
	parts.append("+%d%% attack for %d rounds" % [roundi(item.buff_percent * 100.0), item.buff_duration_rounds])
var who: String = "themselves" if target == source else target.name
execution_log = "[Item] %s used %s on %s: %s" % [source.name, item.name, who, ", ".join(parts) if not parts.is_empty() else "no effect"]
```

