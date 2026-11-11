# The Vanguard (engine class `warrior`)

Status: `[LOCKED]` class role, `3/2/2` weapons & Act 1 Cold Steel kit; `[PROVISIONAL]` Act 2 Inoculated tree; **`[PLACEHOLDER]` Section 4 founder name, backstory, taint & look** ([CANON.md](../CANON.md) 3.1 wins).
Conventions (timing, armour scale, flat damage values) are in the [classes index](classes_and_skill_trees.md)
section 3; rules in [combat and the timeline](combat_and_timeline.md).

Former garrison infantry, Watch deserters and castellan runaways in dented, scavenged plate: masters of narrow
corridors, reach-2 billhook control, and poise breaks (`polearm 3 / sword 2 / axe 2`, `WEAPONROSTER1`). Profile: high HP, heavy armour, high poise.

## 1. Tree

| Tier | Cold Steel (the Iron Discipline) | Inoculated (the Marrow Graft) |
| --- | --- | --- |
| T1 active | Pommel Strike | Calcified Spike |
| T1 passive | Braced Poise | Marrow Chitin |
| T2 active | Intercepting Hook | Bile Siphon |
| T2 passive | Kinetic Leverage | Spasm Counter |
| T3 active | Sunder and Pin | Ribcage Lock |
| Keystone | Unbroken Phalanx | Aberrant Core |

## 2. Cold Steel (Act 1 on)

| Skill | Profile | Effect | Layer |
| --- | --- | --- | --- |
| Pommel Strike | 2 AP, Snap, reach 1, single | a blunt pommel thrust: 18 damage, stagger 25 | `[data]` |
| Braced Poise | passive | +20 max poise; while holding Active Guard, immune to knockback, pull and forced movement from human enemies | `[data]` on a forced-move resist `[kit hook]` |
| Intercepting Hook | 3 AP, Standard, reach 2, ally + attacker | steps up to 1 tile between an ally and an incoming melee attack, takes the attack on his own armour and hooks the attacker 1 tile closer | `[kit hook]` intercept, forced movement |
| Kinetic Leverage | passive | mitigating an attack while holding Active Guard refunds 1 AP next round (once per round) | `[kit hook]` |
| Sunder and Pin | 4 AP, Heavy 4 +1 rec, reach 1, single | a downward billhook cleave: 45 damage, expose 8 for the rest of the battle, rooted 1 round | `[data]` |
| Unbroken Phalanx | keystone | at Strain 0: crits against him become normal hits, and his poise cannot break while he holds Active Guard | `[kit hook]` |

## 3. Inoculated (Act 2 on; proposal)

| Skill | Profile | Effect | Layer |
| --- | --- | --- | --- |
| Calcified Spike | 1 AP, Strain 15, Snap, reach 1, single | bone spurs punch through the forearm: 22 damage, pierces armour, bleed 2 rounds (lore 8 per round) | `[data]` |
| Marrow Chitin | passive | dormant floor +10; +4 armour rating; healing draughts 20% weaker | `[data]` |
| Bile Siphon | 2 AP, Strain 25, Standard, reach 1, single | caustic enzymes on the blade: 35 damage, pierces, expose 4, heals him for 50% of the damage dealt | `[data]` |
| Spasm Counter | passive | on crossing into Flesh-Tension, a bone-whip counter on the last unit that hit him: 20 damage, once per round | `[kit hook]` reaction |
| Ribcage Lock | 1 AP, Strain 35, Heavy 3 +1 rec, self | the ribcage opens into a carapace: +50% mitigation for 1 round; when struck it bursts, 25 damage to every unit within 4 tiles | `[kit hook]` |
| Aberrant Core | keystone | dormant floor locked at 30; in the Aberrant Edge every active skill costs 1 AP less (minimum 1) | `[data]`; lasting cost open (index gaps) |

## 4. Founder: Kaelen

Junior guardsman of the Seventh Ward Castellan Watch (oath at 17), 21. Told to cripple a girl in a Tithing sweep, he
broke Corporal Fitch's jaw and deserted; Captain Vane leads the sweep that hunts him. His left forearm, wrapped in
linen, hides bone spurs from an involuntary Inquisition graft trial; they hum when the church bells toll (backstory and
an Act 2 hook; no mechanics in Act 1). Joins: prologue 0a.

| Kit | Profile | Effect | Act |
| --- | --- | --- | --- |
| Castellan's Penance (trait) | innate | when an adjacent ally is targeted by an attack, Kaelen raises Active Guard as a free Snap reaction and takes 35% of that hit on his own armour | 1 `[kit hook]` reaction, redirect |
| Notched Billhook Sunder (variant of Sunder and Pin) | 4 AP, Heavy 3 +1 rec, reach 1, single | his weighted billhook winds up one block faster (Heavy 3, not 4); expose 8 and root as the base skill; Clerisy or Inquisition targets are also disarmed (-50% damage dealt for 1 round; the nearest engine status, `weaken`, caps at 25% today) | 1 `[data]` |
| Seventh Ward Bulwark (signature) | 3 AP, Heavy 3 +1 rec, self and a 3-tile line | drives his weighted billhook and an iron trench-pavise into the flags: 30 damage and stagger 35 to every foe in front; a 3-tile barrier for 2 rounds that enemies cannot cross; allies behind it gain Active Guard mitigation (+35% baseline) and cannot be targeted by ranged attacks | 1 `[kit hook]` barrier |

## 5. Notes

- Pommel Strike's old "doubled against a windup (+50)" text was the general windup rule written twice; it is not a
  rider.
- The old table named the T2 passive "Stance Leverage"; stances are off, so the name is Kinetic Leverage.
- Intercepting Hook's old "breaks the attacker's recovery phase" is dropped (it would free the attacker); the pull
  stays.
- "Ranged attacks cannot target" units behind the Bulwark needs the line-of-sight check the engine lacks; how the
  barrier blocks ranged attacks until then is open (see the combat gaps: cover guard or a line-of-sight check).
