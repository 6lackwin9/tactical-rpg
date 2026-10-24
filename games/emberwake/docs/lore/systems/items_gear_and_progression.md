# Items, gear and progression

Status: canon for the slots, the pouch, the resources and the Act 1 relic sources ([CANON](../CANON.md) 2.1, 2.5);
proposal for every catalog row and number. Related: [combat](combat_and_timeline.md),
[classes](classes_and_skill_trees.md), [safehouse economy](safehouse_economy_and_triage.md),
[bestiary](enemy_bestiary_and_ai.md).

Engine facts this doc builds on (GAME_BRIEF ITEM1, C1-L, ITEMUSE1, REWARD1): items live in
`games/emberwake/data/catalog/items.json` on the families of `item_families.json` (typed rows `emberwake_data::ItemDef`);
a family fixes hands, slots, scaling, base and bands; items add `modifiers`, `tiers` and `effects` (`skill_potency`,
`status_potency`, `spell_reach`, `resist`). World named items are the world layer's unique pieces and must resolve to a
family. Content is edited only through `aegis content`.

## 1. Slots and the battle pouch `[kit hook]` (exists)

| Slot | Engine name | Holds |
| --- | --- | --- |
| MAIN | `MAIN_HAND` | the weapon (one- or two-handed); sets the weapon family its skills need |
| OFF | `OFF_HAND` | shield, second weapon, orb or tome; empty under a two-handed weapon |
| HEAD | `HEAD` | helm, hood, mask |
| BODY | `BODY` | plate, leather or cloth |
| ACC1, ACC2 | `ACCESSORY_1`, `ACCESSORY_2` | talismans, relics, the keepsake; no duplicate accessory (`StatRules.ALLOW_DUPLICATE_ACCESSORY` false) |
| Pouch | `UnitState.pouch`, `POUCH_MAX` 3 | consumables taken into battle (the old "3-4" is fixed at 3) |

Graft slots (Cranial, Thoracic, Forearms, Visceral) are an Act 2 proposal `[kit hook]` (CANON 2.1).

Wield and burden, as the engine has them (`emberwake_rules::gear::WieldPenalty`, GAME_BRIEF core rules): a stat
deficit cuts power by 15% per missing point (at most 60%) and adds 1 block of cost at a deficit of 4 or more; armour
burden lowers Speed (Speed = 8 + DEX - burden), not tiles moved. The old "+1 AP and +1 tick windup" rule is replaced
by these.

## 2. Weapon families `[data]`

Lore family to engine family. A weapon family's speed is the speed tier of its skills (Snap 1 block, Standard 2,
Heavy 3-4; CANON 2.1); class training per family is class `proficiency` data.

| Lore family | Engine family | Hands | Reach | Typical skill speed | Scales | Trained classes |
| --- | --- | --- | --- | --- | --- | --- |
| Billhooks, poleaxes | none yet (`axe` as stand-in) | 1H / 2H | 1 (reach 2: gap 1) | Standard | STR | Vanguard |
| Broadswords, greatswords | `sword` | 1H / 2H | 1 | Standard / Heavy | STR | Vanguard, Slag-Brute |
| Bodkins, stilettos | `dagger` | 1H | 1 | Snap | DEX | Gutter-Stalker |
| Heavy arbalests | `crossbow` | 2H | 4-5 | Standard / Heavy | DEX / PERC | Arbalestier |
| Iron knuckles | `fist` | 1H / pair | 1 | Snap | STR / CONS | Slag-Brute |
| Foundry hammers | `hammer` | 2H | 1 | Heavy | STR | Slag-Brute, Vanguard |
| Bone-saws, censers | `mace` (censer), `dagger` (bone-saw) as stand-ins | 1H / 2H | 1 | Standard | KNOW / INT | Heretic Acolyte |
| Chemical sprayers, retorts | none yet (`wand` / `staff` as stand-in) | 1H / 2H | 3-4 | Standard | INT / PERC | Ichor Alchemist |

Family notes (proposal): billhooks have high poise damage (20-35) for breaking windups; daggers favour Snap skills that
land before slower windups; arbalests have Heavy variants with a long windup that reward a covered firing spot;
knuckles and chains deal heavy poise damage and slam a knocked-back target into scenery (forced movement: bestiary
index gap 2); sprayers leave 3 x 3 hazard zones (Phosphorus Fire, Caustic Vitriol; same gap).

## 3. Armour `[data]`

Armour rating is the engine's (curve rating / (rating + 50)); lore armour points x 2 = rating (CANON 2.1). Poise from
armour feeds the poise formula (CONS x 2 + armour poise; coefficients open). Evasion bonuses are cut with the evasion
rolls.

| Lore suite | Engine family | Rating | Armour poise | Burden | Class | Trait (engine effect) |
| --- | --- | --- | --- | --- | --- | --- |
| Heavy iron plate | `plate` | 8-12 | 25-40 | high | Vanguard | Snap hits deal no poise damage `[kit hook]` new |
| Soot-stained leathers | `leather` | 2-4 | 10-15 | none | Gutter-Stalker | no move penalty in fog or rubble `[kit hook]` new |
| Reinforced brigandine | `leather` | 4-6 | 15-20 | low | Arbalestier | the half-cover bonus needs cover rules (gap 2) |
| Penal foundry apron | `leather` | 4-8 | 30-50 | medium | Slag-Brute | `resist` burn |
| Embalmer's habit | `cloth` | 2 | 10-15 | none | Heretic Acolyte | `skill_potency` heal +20%; `resist` debuffs +15% |
| Tanner's rubbered coat | `cloth` | 2-4 | 10-15 | low | Ichor Alchemist | `resist` poison (its own splashback) |

## 4. Consumables (pouch) `[data]`

Strain items work from Act 2 (Act 1 has no Strain). Durations are in rounds.

| Item | Use speed | Effect | Engine |
| --- | --- | --- | --- |
| Sulfur Stitch-Poultice | Snap | heals 60 HP (lore scale), cleanses bleed and poison | heal + `cleanse_statuses` exist |
| Tallow Burn Salve | Snap | cleanses burn, regen | exist |
| Magnesium Flash-Pot | Snap | thrown, range 3, 3 x 3: poise damage 20 (the Blind rider is dropped with the accuracy rolls) | area item exists; poise new |
| Pitch Fire-Bomb | Standard | thrown, range 4, 3 x 3 burn for 2 rounds | exists (`con_fire_bomb` shape) |
| Sanctified Salt Sachet | Snap | vents 25 Strain (Act 2 on) | Strain `[kit hook]` |
| Brimstone Tallow Draught | Snap | halves poise damage taken for 3 rounds (the old "Nightmare Whispers" immunity went with the nightmares) | new status `[kit hook]` |
| Clerisy Communion Sedative | Snap | -40 Strain, then -2 AP next round (Act 2 on) | Strain `[kit hook]` |
| Black-Bile Ampoule | Snap | +30 Strain to reach the Aberrant Edge on purpose (Act 2 on) | Strain `[kit hook]` |
| Quenched Cinder Salt | hub only | prevents one Calcification advance after a crisis (Act 2 on) | Open: see gaps (gap 3) |

## 5. Stash materials `[data]`

Rock Salt and the Aberrant Organs are stash materials (CANON 2.5): items of the existing `material` family in the
party stash, never in the pouch. They drop through REWARD1 (one roll per defeated enemy on its base unit's `loot`
table, `key_loot`, `objective_reward`). Apex organs are Tier III.

| Material | Drops from | Used at | Use (Act 2 on unless noted) |
| --- | --- | --- | --- |
| Rock Salt | mortuary carts, cathedral salt cellars, bone-mills | Apothecary's Slab; bribes | flushes (effect: CANON open question); bribes (Act 1 too) |
| Titan Marrow Core (Apex) | the Calcified Amalgam (ENC-EVT-05), Marrow Horrors | Apothecary's Slab | Tier III Inoculated grafts (Ribcage Lock, Aberrant Colossus) |
| Stalker Optic Nerve | Render-Beasts (ENC-EVT-04) | Apothecary's Slab | Ocular Lock and night-sight passives |
| Inquisitor Marrow Valve | Inquisitor Proctors, Proctor Quell (ENC-EVT-02) | Apothecary's Slab | the Apex Strain Siphon (Maren's Strain balance) |
| Caustic Gall-Sac | Ash-Crawlers (ENC-EVT-05), Tallow Dredgers (ENC-EVT-03, -04) | Slab and Elyse | vitriol flasks; unlocks Caustic Vitriol |
| Calcified Bone Shard | Calcified Husks (ENC-EVT-05), the bone-mill (ENC-EVT-03) | Tinker's Hearth; "Nails" Skerrit | armour-shredding quarrels, reinforced knuckles (Act 1 too) |
| Render-Beast Tendon | Render-Beasts (ENC-EVT-04) | Tinker's Hearth | arbalest winch: +1 range, +10 poise damage (Act 1 too) |

Warded Tallow is a purse resource, not a stash item (the old "Warded Tallow Residue" row is folded into it). Scrap is
the currency (CANON 2.5; "Scrap Iron", "black iron slag" and gold are retired).

## 6. Exchange recipes `[kit hook]`

Crafting at the Hearth and the Slab is an exchange recipe: materials + Scrap in, an item, upgrade or unlock out
([safehouse economy](safehouse_economy_and_triage.md) 4). First-playable stand-in `[data]`: a dialogue choice on Bram
or Aline that `requires` the items (condition `items`) and Scrap (condition `gold`), `take`s them and `give`s the
result; both exist in `emberwake_world::dialogue` today.

## 7. Named relics `[data]`

One copy per campaign: a world named item given once (a `once` objective reward or `key_loot`, or a quest `loot_pool`
entry, which leaves every other pool when claimed). Old "+N armour" values are doubled to engine rating. A relic whose
effect needs a hook names it.

| Relic | Slot | Source | Effect (proposal) | Needs |
| --- | --- | --- | --- | --- |
| The Deserter's Kettle Helm | HEAD | Kaelen's quest | rating +8, poise +15; while Kaelen holds Active Guard, adjacent allies take 20% less damage | aura (bestiary gap 2) |
| Orlov's Flensing Talon | MAIN dagger | Orlov (ENC-EVT-04) | Snap; a hit on a bleeding target below 35% HP executes it at no AP (the rear-arc condition is cut) | execute rider `[kit hook]` |
| Hollis's Lever-Winch | ACC (an arbalest is two-handed, so not OFF) | Hollis's cache, Seventh Ward rafters | Heavy Winch Sunder lands 1 block sooner | per-skill speed modifier `[kit hook]` |
| Vane's Gilded Castellan Pavise | OFF shield | Captain Vane (ENC-EVT-01) | strong guard, poise +30; Active Guard returns 15 damage and poise damage 20 to melee attackers | guard reflect `[kit hook]` (riposte is the nearest) |
| Quell's Marrow Scalpel | MAIN dagger | Proctor Quell (ENC-EVT-02) | Snap; a hit vents 10 Strain from the wielder (Act 2 on) and bleeds the target | Strain `[kit hook]` |
| Grendel's Pneumatic Slag-Hammer | MAIN 2H hammer | Overseer Grendel (ENC-EVT-03) | Heavy; the shockwave also hits the unit behind the target | an area shape (gap 4) |
| Sister Aline's Embalming Crook | MAIN staff | Aline's loyalty milestone | healing +35% (`skill_potency` heal); Maren cleanses without the Bile Phial cost | cost waiver `[kit hook]` |
| The Dreamer's Crystalline Vertebra | ACC | the Calcified Amalgam (ENC-EVT-05) | Strain floor locked at 20; Inoculated attacks +30% damage (Act 2 on) | Strain `[kit hook]` |
| Nemeth's Warded Cassock | BODY | the Crucible (ENC-EVT-06) | rating +10, poise +35; immune to Phosphorus Fire and Blight Miasma | Open: see gaps (gap 5) |
| The keepsake | ACC | the founder captured at the Crucible | an heirloom (planner pick); numbers are a CANON open question | - |

## 8. Item fields: what exists and what is new

| Old sample field | Engine today | Layer |
| --- | --- | --- |
| `item_type`, `weapon_type`, `slots`, `hands` | the family (hands, slot), `ItemDef` | exists |
| `tier`, `base_power`, `base_range`, `stat_requirements` | family base and bands, `tiers` criteria, modifiers | exists |
| `granted_passive` | named item `grants` (passives come from sources) | exists |
| `unique_world_relic` | a world named item given once | `[data]` |
| `speed_tier` | a skill field (speed tiers belong to actions) | `[kit hook]` |
| `stagger_power` (poise damage) | none | `[kit hook]` |
| `armor_piercing` | Inoculated attacks pierce (binary, CANON 2.1) | `[kit hook]` |
| `strain_modifier` | none | `[kit hook]` (Act 2) |

## 9. Gaps

1. Reach 2 melee (billhooks, the Flagellant's scourge): the engine's melee reach is 1.
2. Cover: the brigandine's half-cover bonus and "full cover" pillars assume cover rules no doc defines.
3. Quenched Cinder Salt overlaps the Rock Salt flush (CANON open question).
4. Line and "behind the target" areas: the engine has areas (`Area.SWEEP` and others); which shape fits is unchecked.
5. Nemeth's Warded Cassock depends on whether the cell can defeat Nemeth (CANON open question).
6. Missing families: billhook / polearm, chemical sprayer, bone-saw and censer run on stand-in families.
