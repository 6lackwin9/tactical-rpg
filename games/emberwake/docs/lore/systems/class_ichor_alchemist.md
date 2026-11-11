# The Ichor Alchemist (engine class `mage`)

Status: `[LOCKED]` class role, `3/2/2` weapons, Act 1 Cold Steel kit & `M4` join slot (post-first-slice); `[PROVISIONAL]` Act 2 Inoculated tree; **`[PLACEHOLDER]` Section 4 founder name, backstory, taint & look** ([CANON.md](../CANON.md) 3.1 wins).
Conventions (timing, armour scale, flat damage values) are in the [classes index](classes_and_skill_trees.md)
section 3; rules in [combat and the timeline](combat_and_timeline.md).

Tenement hedge chemists, illegal tallow distillers and gutter apothecaries: phosphorus fire, flash-powder, caustic
sprays and miasma. Profile: moderate HP, leather apron and bandolier, low poise. The class makes ground zones
(`[kit hook]` (exists) terrain hazards with round-end effects).

## 1. Tree

| Tier | Cold Steel (Volatile Pyrotechnics) | Inoculated (the Mutagenic Catalyst) |
| --- | --- | --- |
| T1 active | Phosphorus Pot | Caustic Vitriol |
| T1 passive | Combustion Formula | Toxic Blood |
| T2 active | Magnesium Flash-Bomb | Miasma of the Low Sleep |
| T2 passive | Catalytic Munitions | Aberrant Catalysis |
| T3 active | Suffocating Choke-Gas | Liquefaction Transmutation |
| Keystone | Master Alchemist | Primordial Core |

## 2. Cold Steel (Act 1 on)

| Skill | Profile | Effect | Layer |
| --- | --- | --- | --- |
| Phosphorus Pot | 2 AP, Standard, range 4, 3x3 ground | a burning flask: a 3x3 fire zone for 2 rounds, 16 damage and burn to units entering it or ending a round in it | `[kit hook]` (exists) hazard, `[data]` |
| Combustion Formula | passive | her flasks and fire skills deal +25% damage to units on tar, oil or wooden rubble tiles | `[data]` terrain condition |
| Magnesium Flash-Bomb | 3 AP, Snap, range 3, 3x3 blast | blinds every enemy caught (cannot target beyond 1 tile) and deals stagger 20 | `[kit hook]` blind status |
| Catalytic Munitions | passive | burning or blinded enemies get expose 4 for the rest of the battle | `[data]` |
| Suffocating Choke-Gas | 4 AP, Heavy 3 +1 rec, range 4, 4x4 ground | soot and brimstone: enemies inside lose 1 AP and take 20 damage per round; blocks ranged attacks once a line-of-sight check exists | `[kit hook]` (exists) hazard; line of sight open |
| Master Alchemist | keystone | at Strain 0, thrown flasks and grenade items gain +1 tile radius and cost 1 AP less (minimum 1) | `[data]` |

## 3. Inoculated (Act 2 on; proposal)

| Skill | Profile | Effect | Layer |
| --- | --- | --- | --- |
| Caustic Vitriol | 1 AP, Strain 15, Snap, range 4, single | raw aberrant enzymes: pierces, expose 8, corrosion 2 rounds (lore 8 per round) | `[data]` |
| Toxic Blood | passive | dormant floor +10; a melee attacker that hits her takes 15 corrosive recoil | `[kit hook]` reaction |
| Miasma of the Low Sleep | 2 AP, Strain 25, Standard, range 4, 3x3 ground | violet spore-gas: enemies inside hallucinate (seeded 50% chance to attack the nearest ally) and gain Strain (lore 5 per tick, about 30 per round) | `[kit hook]` confusion status; enemy Strain open |
| Aberrant Catalysis | passive | in Flesh-Tension her corrosive damage heals her for 25% of it | `[data]` |
| Liquefaction Transmutation | 2 AP, Strain 35, Heavy 3 +1 rec, range 4, 3x3 ground | the titan's tremor turns stone to tar-bog: units in it are rooted and take 25 acid damage per round | `[kit hook]` (exists) hazard, `[kit hook]` terrain change |
| Primordial Core | keystone | dormant floor locked at 30; in the Aberrant Edge her corrosive skills destroy enemy shields and weapons for the rest of the battle | `[kit hook]`; lasting cost open (index gaps) |

## 4. Founder: Elyse

Hedge chemist, 19, who brewed counterfeit tallow and anti-sedative tonics in a still beneath the Tallow Vats until a
purge cornered her. She carries a wicker bandolier of glass retorts, acid canisters and a pressurised brass sprayer.
Joins: Act 1 mission 4, the Tallow Vats with Guild-Master Orlov (bunk 6).

| Kit | Profile | Effect | Act |
| --- | --- | --- | --- |
| Volatile Reaction Catalyst (trait) | innate | her ground zones last 1 round longer; where two different zones of hers overlap (fire and acid, say) they detonate: a 3x3 blast, 30 damage that ignores armour, every unit there stunned | 1 `[kit hook]` zone overlap trigger |
| Pressurized Vitriol Siphon (variant of Caustic Vitriol) | 2 AP, Strain 20, Snap, range 3, 3-tile arc toward the aimed tile | a pumped acid arc: 26 damage to every target in it, pierces, expose 8, blind 1 round | 2 `[data]` |
| Spore-Bloom of the Sleeper (signature) | 2 AP, Strain 35, Standard, range 4, 3x3 ground | live marrow spores: enemies inside are frenzied (attack the nearest unit, friend or foe) and gain Strain (lore 8 per tick, about 48 per round); allies inside have their dormant floor lowered by 15 for the battle | 2 `[kit hook]`; enemy Strain open |

In Act 1 Elyse has her trait and the Cold Steel branch; her variant and signature are Inoculated and wait for Act 2.

## 5. Notes

- Old "per tick" Strain rates are restated per round because zone effects act at round end in the engine; whether
  enemies carry Strain at all is an index gap.
- "Frontal arc" needs facing, which is cut: the arc is aimed at a tile.
- Spore-Bloom's "floor temporarily -15" is read as for the rest of the battle; confirm with the Strain spec.
- Blind is not one of the engine's fifteen statuses today; it is a new status row.
