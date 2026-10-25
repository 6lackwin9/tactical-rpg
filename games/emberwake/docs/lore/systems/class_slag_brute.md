# The Slag-Brute (engine class `brawler`)

Status: canon for the Cold Steel branch and Dunstan's Act 1 kit; proposal for the Inoculated branch (Act 2 on).
Conventions (timing, armour scale, flat damage values) are in the [classes index](classes_and_skill_trees.md)
section 3; rules in [combat and the timeline](combat_and_timeline.md).

Escaped bone-mill labourers, kiln stokers and foundry breakers: bare hands, shackle cestuses, body throws and broken
walls. Profile: very high HP, medium scavenged plates, very high poise. The class is the stagger engine of the squad.

## 1. Tree

| Tier | Cold Steel (the Forge Foundry) | Inoculated (the Marrow Titan) |
| --- | --- | --- |
| T1 active | Jaw-Breaker | Tendril Lash |
| T1 passive | Hypertrophy | Osseous Bulk |
| T2 active | Slag Slam and Throw | Boiling Slag Bile |
| T2 passive | Anvil Posture | Flesh Rampage |
| T3 active | Foundry Shatter | Aberrant Colossus |
| Keystone | Unbroken Iron | Juggernaut Core |

## 2. Cold Steel (Act 1 on)

| Skill | Profile | Effect | Layer |
| --- | --- | --- | --- |
| Jaw-Breaker | 2 AP, Snap, reach 1, single | a shackled punch: 20 damage, stagger 30, knockback 1 tile | `[kit hook]` knockback |
| Hypertrophy | passive | +20% max HP; his knockbacks push 1 tile further | `[data]` |
| Slag Slam and Throw | 3 AP, Standard, reach 1, enemy + landing tile | hurls an adjacent enemy up to 2 tiles into another enemy or a wall: both take 30 damage and stagger 25; a wall may crumble to rubble | `[kit hook]` throw, terrain change |
| Anvil Posture | passive | Snap attacks cannot break his poise; melee blunt damage against him -25% | `[kit hook]` |
| Foundry Shatter | 4 AP, Heavy 4 +1 rec, reach 1, 3-tile area toward the aimed tile | both fists into the flagstones: the tiles become rubble and every unit there is stunned | `[kit hook]` terrain change, `[kit hook]` (exists) `stun` |
| Unbroken Iron | keystone | at Strain 0, dropping below 30% HP refunds 2 AP and grants fortify 8 for 2 rounds | `[data]` trigger on a kit hook |

## 3. Inoculated (Act 2 on; proposal)

| Skill | Profile | Effect | Layer |
| --- | --- | --- | --- |
| Tendril Lash | 1 AP, Strain 15, Snap, range 2, single | muscle tendrils: 18 damage, pierces, yanks the target to an adjacent tile | `[kit hook]` pull |
| Osseous Bulk | passive | dormant floor +10; +4 armour rating; -1 tile per move block | `[data]` |
| Boiling Slag Bile | 2 AP, Strain 25, Standard, range 3, line 3 | boiling bile along a 3-tile line: the ground burns, and units caught get expose 4 per round for 2 rounds | `[kit hook]` (exists) hazard, area `LINE` |
| Flesh Rampage | passive | in Flesh-Tension, each melee hit he lands sends a shockwave: 15 damage to every adjacent foe | `[kit hook]` on-hit trigger |
| Aberrant Colossus | 2 AP, Strain 35, Heavy 3 +1 rec, self | his mass doubles for 2 rounds: immune to stun, root and forced movement; melee attacks hit 3 adjacent tiles for double damage; recoil 15% of max HP per round | `[kit hook]` |
| Juggernaut Core | keystone | dormant floor locked at 30; in the Aberrant Edge his HP cannot drop below 1 | `[data]`; lasting cost open (index gaps) |

## 4. Founder: Dunstan "Slag-Jaw"

Stoker at the Fourth Kiln bone-mill under Overseer Grendel, 22. Slag splashback burned his jaw and it is wired with
scrap; he was on the penal chain gang after cracking an overseer's skull, and he still wears the penal wrist shackles
as weighted cestuses. Joins: Act 1 mission 3, the Bone-Mill Raid (bunk 5).

| Kit | Profile | Effect | Act |
| --- | --- | --- | --- |
| Foundry-Wired Spite (trait) | innate | below 50% HP his poise cannot break and he is immune to stun; each hit he takes adds Slag Fury, +6% melee damage, up to 5 stacks for the battle | 1 `[kit hook]` |
| Shackle-Pummel Slam (variant of Jaw-Breaker) | 2 AP, Snap, reach 1, single | a two-handed cuff smash: 24 damage, stagger 35; if the knockback drives the target into a wall or obstacle, the damage doubles (48) and the target's remaining blocks this round are cancelled as a break | 1 `[kit hook]` wall slam |
| Molten Marrow Eruption (signature) | 2 AP, Strain 35, Heavy 3 +1 rec, self, radius 2 | unhinges his wired jaw: 35 damage to every enemy within 2 tiles, knockback 2, expose 6, burning slag on the adjacent tiles for 2 rounds | 2 `[kit hook]` knockback, `[kit hook]` (exists) hazard |

## 5. Notes

- "Frontal cone" (Foundry Shatter) needs facing, which is cut: the area is aimed at a tile instead; its exact shape is
  set with the data.
- Foundry Shatter's "stunned on tick 5" becomes the engine's `stun` status applied on impact.
- Wall slam damage is the general rule in [combat and the timeline](combat_and_timeline.md) section 6; Shackle-Pummel
  Slam's doubling replaces it for that skill.
- Dunstan's two-handed Shackle-Pummel uses his cuffs, not a two-handed weapon; a Stage IV Calcification would not bar
  it (bare-handed attacks then pierce).
