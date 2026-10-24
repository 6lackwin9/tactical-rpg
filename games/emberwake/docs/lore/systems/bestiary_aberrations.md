# Bestiary: Low Sleep aberrations

Status: proposal (maneuvers and tuning targets); names and roles canon. Index, conversion rules and gaps:
[enemy bestiary](enemy_bestiary_and_ai.md). Every unit row is `[data]`; AI lines are `[kit hook]` (AI profile).

Flesh twisted by the black marrow that vents from the sleeping titan under the city. Act 1 meets them once, in the
ENC-EVT-05 interlude beneath the Sluice Undercroft. Their Strain riders do nothing in Act 1, where operatives have no
Strain (CANON 2.2; index gap 4).

| Unit | Tier | Class (proposal) | Bulk | Armour rating | Poise | Mobility | Role |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Ash-Crawler | base (minion) | rogue (beast body) | 0.5 | 2 | 15 | quick, tunnels | burrow ambusher, acid |
| Calcified Husk | base (standard) | warrior | 1.2 | 10 | 45 | low | tank, bone shrapnel |
| Marrow Horror | elite | mage | 2.2 | 4 | 65 | normal | eldritch terror, Strain pressure |
| The Calcified Amalgam | boss | mage | 4.0 | 12 | 90 | low | "the Dreamer's Whelp" |

The Amalgam's rating 12 (old armour 6) is above CANON's "boss armour 3-5 -> rating 6-10" band; the balance lane
decides whether it stays the exception.

## Ash-Crawler (base)

- Look: many-legged chitinous centipede, a translucent abdomen of bubbling violet acid.
- Burrow Ambush: Snap (1 block), range 3: it sinks into the stone dust and erupts under a target, light hit that
  pierces armour, stagger (the engine status). Tunnelling through rubble is new `[kit hook]`.
- Caustic Vomit: Standard (2 blocks), range 3, light hit, poison, expose 4.
- AI `[kit hook]`: tunnels behind the line to strike operatives on rubble or in fog.

## Calcified Husk (base)

- Look: the petrified corpse of a miner or an Unsealed youth, its whole skeleton turned to brittle black bone that
  juts through rotten skin.
- Innate trait, Osseous Shrapnel: when its poise breaks, the ribcage bursts: medium hit to every adjacent unit (a
  trigger on a poise break: index gap 2).
- Bone-Club Slam: Standard (2 blocks), reach 1, medium hit, poise damage 25.
- AI `[kit hook]`: walks at the nearest living target; the player times the break to avoid the shrapnel.

## Marrow Horror (elite)

- Look: an amorphous mass of black muscle tendrils and fused skulls, violet marrow vents pulsing with the titan's
  heartbeat.
- Innate trait, Resonance Aura: an operative ending the round within 3 tiles gains +10 Strain (Act 2 on; aura: index
  gap 2).
- Marrow Tendril Flail: Standard (2 blocks), reach 2, 3-tile sweep, medium hit that pierces armour, bleed.
- Leviathan's Whispers: Heavy (3 blocks) + 2 recovery, range 4, one target: no damage, +30 Strain (Act 2 on) and
  Nightmare Hallucination (new status; its 50% roll is index gap 3).
- AI `[kit hook]`: pushes operatives' Strain toward the crisis brackets (Act 2 on).

## The Calcified Amalgam (boss, ENC-EVT-05)

Dozens of calcified corpses fused by pulsing black marrow tendrils, found in the deepest storm drains under the
Undercroft; it carries the titan's dreaming frequency. Boss of the Act 1 interlude "The Awakening Fissure" (not a
siege in Act 1). Template: Marrow Horror fused with Calcified Husks.

- Innate trait, Resonant Titan Nexus: every 2 rounds, when the cathedral bells toll, it pulses: operatives with a
  Calcification stage or 25+ Strain (Acute Resonance) take a medium hit that pierces armour and +10 Strain. In Act 1 no
  operative qualifies, so the trait is idle (index gap 4). A round-cadence trigger is `[kit hook]` new.
- Altered maneuver, Ossified Spine Volley (from Marrow Tendril Flail): Standard (2 blocks) + 2 recovery, radius 4,
  medium hit on every unit in it (allies of the Amalgam too) that pierces armour, bleed.
- Capstone, Catastrophic Ossification Bloom: a boss phase at 30% HP `[kit hook]` exists (BOSS1); the old second
  trigger "on a poise break" is new. Heavy (4 blocks) + 1 recovery, the whole arena: bone spires punch through the
  floor; every unit off the warded tiles takes a heavy hit and is rooted for 1 round. Warded safe tiles are arena
  data `[data]` once tiles can carry such a flag (index gap 2).
- Drops: see [Act 1 encounters](bestiary_encounters_act1.md) ENC-EVT-05.
