# The Arbalestier (engine class `ranger`)

Status: canon for the Cold Steel branch and Hollis's Act 1 kit; proposal for the Inoculated branch (Act 2 on).
Conventions (timing, armour scale, flat damage values) are in the [classes index](classes_and_skill_trees.md)
section 3; rules in [combat and the timeline](combat_and_timeline.md).

Crag poachers, timberline sharpshooters and ridge scouts with heavy winch arbalests and wire snares: they hold long
sightlines and break windups from range. Profile: moderate HP, medium brigandine, range 5, moderate poise. A Stage IV
Calcification (the Shakes) bars two-handed weapons, the arbalest included.

## 1. Tree

| Tier | Cold Steel (the Iron Sight) | Inoculated (the Ocular Graft) |
| --- | --- | --- |
| T1 active | Notched Quarrel | Ocular Lock |
| T1 passive | High-Ground Traverse | Predatory Pupil |
| T2 active | Tripwire Snare | Splintering Chitin Bolt |
| T2 passive | Cover Breaker | Reflex Twitch |
| T3 active | Heavy Winch Sunder | Arterial Laser Pulse |
| Keystone | Dead-Eye Discipline | Apex Stalker Core |

## 2. Cold Steel (Act 1 on)

| Skill | Profile | Effect | Layer |
| --- | --- | --- | --- |
| Notched Quarrel | 2 AP, Standard, range 4, single | a heavy steel quarrel: 24 damage, stagger 15, ignores 2 armour rating | `[data]` |
| High-Ground Traverse | passive | +1 range when shooting from rubble or roof tiles | `[kit hook]` terrain range bonus; the old "-15% hit chance against him" is cut |
| Tripwire Snare | 3 AP, Snap, an adjacent tile | a hidden wire trap: the first enemy to step on it takes 22 damage, loses its remaining move blocks this round and is rooted 1 round | `[kit hook]` trap tile |
| Cover Breaker | passive | the target's cover guard is halved; attacks on a unit in windup gain +25% crit chance and +15 stagger | `[kit hook]` (exists) cover, `[data]` |
| Heavy Winch Sunder | 4 AP, Heavy 4 +1 rec, range 5, single | a lever-winch bolt: 52 damage, knockback 2 tiles, expose 6, and the target's windup breaks outright | `[kit hook]` knockback, forced break |
| Dead-Eye Discipline | keystone | at Strain 0, a ranged kill refunds 2 AP and reveals every enemy's full plan for the next round | `[kit hook]` (exists) intent foresight, `[data]` |

## 3. Inoculated (Act 2 on; proposal)

| Skill | Profile | Effect | Layer |
| --- | --- | --- | --- |
| Ocular Lock | 1 AP, Strain 15, Snap, range 5, single | the grafted pupil paints the target: allied attacks on it gain +30% crit chance for 2 rounds | `[data]` mark status |
| Predatory Pupil | passive | dormant floor +10; ignores smoke zones and half cover when targeting | `[data]` |
| Splintering Chitin Bolt | 2 AP, Strain 25, Standard, range 4, target + plus shape | a bone bolt that splinters: 28 damage, pierces, bleed 2 rounds (lore 6 per round) on the target and every orthogonal neighbour | `[kit hook]` (exists) area `PLUS`, `[data]` |
| Reflex Twitch | passive | in Acute Resonance, when a ranged attack targets him during his windup, he fires a free needle at the shooter: 12 damage, stagger 15, once per round | `[kit hook]` reaction |
| Arterial Laser Pulse | 2 AP, Strain 35, Heavy 3 +1 rec, line 6 | a red beam through every unit in a 6-tile line: 48 damage, ignores armour and barriers | `[kit hook]` (exists) area `LINE`, `[data]` |
| Apex Stalker Core | keystone | dormant floor locked at 30; in the Aberrant Edge all his ranged attacks pierce armour | `[data]`; lasting cost open (index gaps) |

## 4. Founder: Hollis

Crag poacher from the limestone ridges outside Cinderreach, 20. He shot the Castellan commander who dragged his
sister Wren to the cages and was taken in chains; Wren escaped into the crags. His left socket holds a crude
night-stalker eye graft he cut in himself and regrets (backstory and an Act 2 hook; no mechanics in Act 1). Joins:
prologue 0c, freed from the iron mortuary cart at the canal bridge with Maren.

| Kit | Profile | Effect | Act |
| --- | --- | --- | --- |
| Crag Hawkeye (trait; renamed from "Crag Hawkeye", which recalled the retired clocktower origin) | innate | ignores partial cover; reveal open: the old trait showed the action type of enemies in his sight, which the intent tells already show everyone; recommendation: the foresight reveal of the whole plan of enemies within his range | 1 `[kit hook]` (exists) intent foresight; open |
| Poacher's Barbed Quarrel (variant of Notched Quarrel) | 2 AP, Standard, range 5, single | 24 damage and stagger 15 as the base; if the target moves later this round the barbs tear: 18 damage and its move stops on that tile | 1 `[kit hook]` |
| Predator Reticle Pulse (signature) | 2 AP, Strain 20, Standard, range 6, single | the eye graft paints a target: expose 8 for 2 rounds; every allied attack on it gains +30% crit chance and +15 stagger | 2 `[data]` |

## 5. Notes

- Cut riders removed: Ocular Lock's "cannot evade or enter stealth", Predator Reticle's "loses all evasion",
  Hawkeye's "accuracy penalties", High-Ground Traverse's "-15% hit chance".
- The engine's intent tells already show each enemy's first action to everyone; Dead-Eye's reveal is the foresight
  reveal (the whole plan) that the tells give a foresight holder today, and the same is recommended for Hawkeye.
- Heavy Winch Sunder "breaks any windup": a forced break regardless of poise (`[kit hook]`); balance should check it
  with the interrupt-lock risk in [risk and reward](risk_reward_and_corruption_balance.md).
- Old timing: Notched Quarrel listed a T1-T2 windup with impact on T4; it is a plain Standard (2 blocks).
