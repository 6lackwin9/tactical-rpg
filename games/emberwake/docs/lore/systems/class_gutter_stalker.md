# The Gutter-Stalker (engine class `rogue`)

Status: `[LOCKED]` class role, `3/2/2` weapons & Act 1 Cold Steel kit; `[PROVISIONAL]` Act 2 Inoculated tree; **`[PLACEHOLDER]` Section 4 founder name, backstory, taint & look** ([CANON.md](../CANON.md) 3.1 wins).
Conventions (timing, armour scale, flat damage values) are in the [classes index](classes_and_skill_trees.md)
section 3; rules in [combat and the timeline](combat_and_timeline.md).

Rooftop couriers, chimney sweeps and curfew runners of the Ash-Crawl: they slip through quarantine grates, strike the
staggered and the pinned, and leave before the Watch arrives. Profile: moderate HP, light leathers and rags, high
mobility, low poise. The old "high evasion (20%)" is cut with evasion rolls.

## 1. Tree

| Tier | Cold Steel (Cinder and Sinew) | Inoculated (the Umbral Graft) |
| --- | --- | --- |
| T1 active | Hamstring Sever | Void-Flicker |
| T1 passive | Prowler's Cadence | Viscous Shadow |
| T2 active | Ash-Cloud and Fade | Flensing Filaments |
| T2 passive | Anatomical Flaw | Phase-Twitch |
| T3 active | Throat-Carver's Art | Umbral Dissolution |
| Keystone | Ghost of the Undercroft | Predatory Singularity |

## 2. Cold Steel (Act 1 on)

| Skill | Profile | Effect | Layer |
| --- | --- | --- | --- |
| Hamstring Sever | 2 AP, Snap, reach 1, single | a low dagger slice: 16 damage, slowed 2 rounds (lore -2 tiles), and the target's reactions (riposte, counters) are off for the round | `[data]` |
| Prowler's Cadence | passive | moving through low cover, mist or rubble costs no extra AP | `[kit hook]` (exists) terrain move cost, `[data]` |
| Ash-Cloud and Fade | 3 AP, Snap, range 2, 3x3 ground | a soot flask: a 3x3 smoke zone for 2 rounds, then she jumps 2 tiles away | `[kit hook]` (exists) hazard zone, `[data]` |
| Anatomical Flaw | passive | attacks on stunned, staggered (status or poise break) or rooted targets ignore armour and gain +25% crit chance | `[kit hook]` (exists) `exploit` bonus, `[data]` |
| Throat-Carver's Art | 4 AP, Standard, reach 1, single | an execution strike: 55 damage; a target below 35% HP dies outright and she gets 2 AP back | `[kit hook]` execute |
| Ghost of the Undercroft | keystone | replacement open: its whole effect (stealth on a backstab kill, Fear as -20% accuracy) is cut | open |

## 3. Inoculated (Act 2 on; proposal)

| Skill | Profile | Effect | Layer |
| --- | --- | --- | --- |
| Void-Flicker | 1 AP, Strain 15, Snap, range 3, self | dissolves and reappears up to 3 tiles away on a free tile, through walls, obstacles and units | `[kit hook]` teleport |
| Viscous Shadow | passive | dormant floor +10; light and phosphorus attacks deal +25% damage to her; its benefit (+15% evasion) is cut, replacement open | `[data]`; open |
| Flensing Filaments | 2 AP, Strain 25, Standard, range 4, single | shadow tendons: pulls the target 2 tiles, corrosive 2 rounds (lore 6 per round), expose 6 | `[kit hook]` pull |
| Phase-Twitch | passive | in Acute Resonance, she flickers 1 tile away before a melee strike on her lands (once per round; a move, not an evasion roll) | `[kit hook]` reaction |
| Umbral Dissolution | 2 AP, Strain 35, Heavy 4 +1 rec, self | immune to physical damage for 1 round; her next attack after she re-forms deals double damage, and she takes 15% of max HP as recoil | `[kit hook]` |
| Predatory Singularity | keystone | dormant floor locked at 30; in the Aberrant Edge a crit vents 20 Strain and readies Void-Flicker again | `[data]`; lasting cost open (index gaps) |

## 4. Founder: Vesper

Ash-Crawl curfew runner and thief, 18. Her parents died of black-lung and were rendered at the Tallow Vats; Orlov's
Guild paid the informant who sold out her runner ring. Tess was her runner contact. Smoke drifts from her fingertips
and her shadow leans toward enemies whatever the lamps do (backstory only in Act 1). Joins: prologue 0b, the rooftop.
Her vendetta is Act 1 mission 4 against Guild-Master Orlov.

| Kit | Profile | Effect | Act |
| --- | --- | --- | --- |
| Cinder-Child's Reflex (trait) | innate | replacement open: its triggers (evading an attack, breaking line of sight, entering stealth) are cut; its reward was +1 tile on her next move and, Inoculated, -10 Strain | 1 open |
| Orlov's Retribution (variant of Throat-Carver's Art) | 3 AP, Standard, reach 1, single | one AP cheaper; if it kills, she dissolves into soot and reappears 2 tiles away on a free tile without spending her move | 1 `[kit hook]` teleport |
| Umbral Flensing (signature) | 2 AP, Strain 30, Snap, range 3, line 3 | smoke-fingers become razor filaments along a 3-tile line: 38 damage, pierces, pulls every target 1 tile toward her, mutilated (slowed 2 rounds, bleed 2 rounds, lore 10 per round) | 2 `[kit hook]` pull |

## 5. Notes

- Cut riders removed: Hamstring Sever's +40% from behind, Prowler's Cadence's smaller detection cone, Ash-Cloud's
  "into stealth", Throat-Carver's rear-arc requirement, Orlov's Retribution's "from behind" and stealth reset, Void-
  Flicker's "enemy engagement zones" (the engine has none).
- Ash-Cloud and Fade jumps after its strike: an act-then-move maneuver, open in the combat gaps; the smoke blocks
  ranged attacks only once a line-of-sight check exists (until then cover guard is the recommendation there).
- Orlov's Retribution struck "one tick earlier" in the anchor scheme; with relative windups its speed edge is open
  (a Snap execute would be strong).
- Recommendation for the open trait and keystone: rebuild them on hooks the engine has (`exploit` bonuses against
  statuses, the attack chase, intent foresight) rather than on stealth.
