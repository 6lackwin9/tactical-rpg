# Bestiary: the Castellan Watch

Status: proposal (maneuvers and tuning targets); names and roles canon. Index, conversion rules and gaps:
[enemy bestiary](enemy_bestiary_and_ai.md). Every unit row is `[data]` (class + tier + tuning deltas); AI lines are
`[kit hook]` (AI profile); missing mechanics point to the index gaps.

The Watch is the Inquisition's city arm: curfew patrols in padded gambesons and dented plate, shield walls at
chokepoints, telegraphed heavy swings.

| Unit | Tier | Class (proposal) | Bulk | Armour rating | Poise | Mobility | Role |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Castellan Footman | base (standard) | warrior | 1.0 | 6 | 35 | normal | frontline anchor, Standard billhook |
| Watch Crossbowman | base (standard) | ranger (crossbow) | 0.8 | 2 | 25 | quick | ranged pin, Standard bolt |
| Bailiff Enforcer | elite | warrior (mace) | 1.7 | 8 | 55 | normal | heavy disrupter, Heavy cleave |
| Captain Vane | boss | warrior | 2.9 | 10 | 75 | normal | the Watch traitor |

## Castellan Footman (base)

- Look: padded gambeson under a dented breastplate, plain iron kettle helm, iron-rimmed heater shield, short billhook.
- Billhook Hook: Standard (2 blocks), reach 1, medium hit, poise damage 15. A target that is moving is pulled 1 tile
  and its move ends (forced movement: index gap 2).
- Shield Wall Bracing: Snap (1 block), self. The unit holds Active Guard for the rest of the round (the base move:
  the engine guard, capped by its guard cap, halves poise damage; CANON 2.1 replaces the lore's +40%).
- AI `[kit hook]`: moves onto chokepoint tiles next to other footmen; braces the round after an ally within 2 tiles is
  struck.

## Watch Crossbowman (base)

- Look: woollen watch cloak over boiled leather, hand-cranked stirrup arbalest, short side dagger.
- Aimed Bolt: Standard (2 blocks) + 2 recovery blocks, range 4, medium hit, expose 2 for the hit (old "ignores 1
  armour"), poise damage 10; the telegraph line follows the target.
- Buckler Jab: Snap (1 block), reach 1, light hit, pushes the attacker 1 tile back (index gap 2).
- AI `[kit hook]`: seeks rubble or raised tiles; targets the lowest-HP operative or one in a windup (to break it);
  backs off when engaged in melee (exists: today's AI kites to max range).

## Bailiff Enforcer (elite)

- Look: towering bailiff in blackened plate and an executioner's hood, two-handed flanged mace wrapped in chain.
- Innate trait, Unyielding Authority: immune to forced movement; Snap hits deal it no poise damage `[kit hook]`.
- Crushing Judgment: Heavy (4 blocks) + 1 recovery block, reach 1, 3-tile frontal cone, heavy hit, expose 6, stun on
  everyone in the cone; the cone is painted on the ground while it winds up (intent tells exist; ground decal new).
- Iron Shackle Drag: Standard (2 blocks), range 3, light hit, pulls the target 2 tiles closer (index gap 2) and roots
  it for 1 round.
- AI `[kit hook]`: walks at the cell's tank or largest cluster; commits to Crushing Judgment often, inviting a break.

## Captain Vane (boss, ENC-EVT-01)

Kaelen's former commanding officer in the Seventh Ward Castellan Watch, who sold his own men to the Inquisition for a church
commission ([cast registry](../background/cast_registry.md)). Gilded plate, executioner's billhook, heavy pavise. He
gives "Order 44" in prologue 0a and withdraws; he is the boss of Act 1 M1 "The Sweep". Template: Bailiff Enforcer.

- Innate trait, Traitor's Discipline: Watch allies within 3 tiles gain +10 poise and take no poise damage from Snap
  hits (aura: index gap 2). Against Kaelen his counter is always ready: riposte while Kaelen is adjacent (proposal: the
  engine `riposte` status; the old "auto-parry on tick 2" has no anchor any more).
- Altered maneuver, Gilded Billhook Sunder (from Crushing Judgment): Heavy (3 blocks, one faster than the Enforcer's)
  + 2 recovery blocks, reach 1, 3-tile arc, heavy hit, expose 8, knockback 2 tiles into rubble (index gap 2).
- Capstone, Seventh Ward Execution Wall: a boss phase at 50% HP or round 3 `[kit hook]` exists (BOSS1, PACE1
  `or_round`). Heavy (4 blocks) + 1 recovery, range 4, a 4-tile line: two shield-bearing footmen lock shields with him
  and the wall advances 2 tiles, medium hit, knocks down and pins the heroes it meets against walls (forced movement:
  index gap 2). The two footmen can come as the arrival wave (PHASEFOLD) or a phase `wave` effect.
- Drops: see [Act 1 encounters](bestiary_encounters_act1.md) ENC-EVT-01.
