# Items, gear and progression

Status: `[LOCKED]` equipment slots, `POUCH_MAX = 3`, weapon families (`WEAPONROSTER1`), resource loops & Act 1-3 relic drop slots; `[PROVISIONAL]` catalog tuning numbers; **`[PLACEHOLDER]` item/relic/organ names, inscriptions & icons** ([CANON.md](../CANON.md) Section 5 wins on conflict). Related: [combat](combat_and_timeline.md), [classes](classes_and_skill_trees.md), [safehouse economy](safehouse_economy_and_triage.md), [bestiary](enemy_bestiary_and_ai.md).

Engine facts this doc builds on (GAME_BRIEF ITEM1, C1-L, ITEMUSE1, REWARD1, WEAPONROSTER1): items live in
`games/emberwake/data/catalog/items.json` on the families of `item_families.json` (typed rows `rpgkit_data::ItemDef`);
a family fixes hands, slots, scaling, base and bands (`range_band` for weapon reach); items add `modifiers`, `tiers` and
`effects` (`skill_potency`, `status_potency`, `resist`; legacy `spell_reach` retires in `PR5-28`). World named items are
the world layer's unique pieces and must resolve to a family. Content is edited only through `aegis content`.

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

Wield and burden, as the kit has them (`rpgkit_rules::gear`, `PK1-S2` / `WEAPONROSTER1`): untrained weapon families
remain equippable under soft wield (`0.75x` physical weapon potency and `+1` burden; class `proficiency` waives the
penalty and unlocks `[Signature]` skills). Burden lowers **Initiative** (`8 + DEX - burden`, same-block tie-breaker)
while **Move Rate** (`clamp(2 + floor(DEX / 6), 2, 4)` tiles per move block) is unburdened so plated front-liners keep
pace across the grid.

## 2. Weapon families `[data]`

Lore family to shipped `item_families.json` row (`WEAPONROSTER1`). A weapon family's speed is the speed tier of its
skills (Snap 1 block, Standard 2, Heavy 3-4; CANON 2.1); class training per family is class `proficiency` data.

| Lore family | Engine family | Hands | Reach (`range_band`) | Typical skill speed | Scales | Trained classes |
| --- | --- | --- | --- | --- | --- | --- |
| Billhooks, poleaxes | `polearm` | 2H | 1-2 | Standard / Heavy | STR | Vanguard |
| Broadswords, greatswords | `sword`, `greatsword` | 1H / 2H | 1 | Standard / Heavy | STR | Vanguard, Slag-Brute |
| Bodkins, stilettos | `dagger` | 1H | 1 | Snap | DEX | Gutter-Stalker |
| Heavy arbalests | `crossbow` | 2H | 2-5 | Standard / Heavy | DEX / PERC | Arbalestier |
| Iron knuckles | `fist` | 1H / pair | 1 | Snap | STR / CONS | Slag-Brute |
| Foundry hammers | `hammer` | 2H | 1 | Heavy | STR | Slag-Brute, Vanguard |
| Bone-saws, censers, scourges | `bone_saw`, `censer`, `scourge` | 1H | 1 (`scourge` 1-2) | Snap / Standard | KNOW / INT / STR | Heretic Acolyte, Inquisition |
| Chemical sprayers, retorts | `sprayer` | 2H | 2-4 | Standard | INT / PERC | Ichor Alchemist |

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
| Reinforced brigandine | `leather` | 4-6 | 15-20 | low | Arbalestier | the half-cover bonus is dropped (Q-54, gap 2) |
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
| Brimstone Tallow Draught (`con_brimstone_draught`) | Snap | grants `empower` (+30% outgoing skill potency) for 2 rounds | shipped (`PK3`) |
| Clerisy Communion Sedative | Snap | -40 Strain, then -2 AP next round (Act 2 on) | Strain `[kit hook]` |
| Black-Bile Ampoule | Snap | +30 Strain to reach the Aberrant Edge on purpose (Act 2 on) | Strain `[kit hook]` |
| Quenched Cinder Salt (`con_quenched_cinder_salt`) | Snap (in battle) | applies `unmark` to calm a Stage V Flesh Breach in battle before Terminal Drift takes the operative (Act 2 on) | shipped (`PK2` / `PK3`) |

## 5. Stash materials `[data]`

Rock Salt and the Aberrant Organs are stash materials (CANON 2.5): items of the existing `material` family in the
party stash, never in the pouch. They drop through REWARD1 (one roll per defeated enemy on its base unit's `loot`
table, `key_loot`, `objective_reward`). Apex organs are Tier III.

| Material | Drops from | Used at | Use (Act 2 on unless noted) |
| --- | --- | --- | --- |
| Rock Salt | mortuary carts, cathedral salt cellars, bone-mills | Apothecary's Slab; bribes | flushes (`-10` dormant floor for next battle, `PK2`); bribes (Act 1 too) |
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
([safehouse economy](safehouse_economy_and_triage.md) 4). Shipped in `rpgkit_world` (`PK2` `hub_facilities`): typed
facility services at the Hearth and Slab alongside dialogue-driven campaign unlocks.

## 7. Named relics `[data]`

One copy per campaign: a world named item given once (a `once` objective reward or `key_loot`, or a quest `loot_pool`
entry, which leaves every other pool when claimed). Old "+N armour" values are doubled to engine rating. A relic whose
effect needs a hook names it.

| Relic | Slot | Source | Effect (proposal) | Needs |
| --- | --- | --- | --- | --- |
| The Deserter's Kettle Helm | HEAD | Kaelen's quest | rating +8, poise +15; while Kaelen holds Active Guard, adjacent allies take 20% less damage | aura (bestiary gap 2) |
| Orlov's Flensing Talon | MAIN dagger | M4 `ENC-EVT-04` `objective_reward` (left behind when Orlov withdraws via `orlov_withdraws`; `PR5-34`) | Snap; a hit on a bleeding target below 35% HP executes it at no AP (the rear-arc condition is cut) | execute rider `[kit hook]` |
| Hollis's Lever-Winch | ACC (an arbalest is two-handed, so not OFF) | Hollis's cache, Seventh Ward rafters | Heavy Winch Sunder lands 1 block sooner | per-skill speed modifier `[kit hook]` |
| Vane's Gilded Castellan Pavise | OFF shield | Captain Vane (ENC-EVT-01) | strong guard, poise +30; Active Guard returns 15 damage and poise damage 20 to melee attackers | guard reflect `[kit hook]` (riposte is the nearest) |
| Quell's Marrow Scalpel | MAIN dagger | Proctor Quell (ENC-EVT-02) | Snap; a hit vents 10 Strain from the wielder (Act 2 on) and bleeds the target | Strain `[kit hook]` |
| Grendel's Pneumatic Slag-Hammer | MAIN 2H hammer | Overseer Grendel (ENC-EVT-03) | Heavy; the shockwave also hits the unit behind the target | `line_2` area (`PK1-S1`) |
| Sister Aline's Embalming Crook | MAIN staff | Aline's loyalty milestone | healing +35% (`skill_potency` heal); Maren cleanses without the Bile Phial cost | cost waiver `[kit hook]` |
| The Dreamer's Crystalline Vertebra | ACC | the Calcified Amalgam (ENC-EVT-05) | Strain floor locked at 20; Inoculated attacks +30% damage (Act 2 on) | Strain `[kit hook]` |
| Nemeth's Warded Cassock | BODY | Nemeth, where he dies: the Silent Ossuary (ENC-ACT3-04, Act 3 node 20, optional; CANON 3.6) | rating +10, poise +35; immune to Phosphorus Fire and Blight Miasma | status immunity (`resist` exists) |
| The keepsake | ACC | the founder captured at the Crucible | an heirloom (planner pick); numbers are a CANON open question | - |

## 8. Item fields: what exists and what is new

| Old sample field | Engine today | Layer |
| --- | --- | --- |
| `item_type`, `weapon_type`, `slots`, `hands` | the family (hands, slot), `ItemDef` | exists |
| `tier`, `base_power`, `base_range`, `stat_requirements` | family base and bands, `tiers` criteria, modifiers | exists |
| `granted_passive` | named item `grants` (passives come from sources) | exists |
| `unique_world_relic` | a world named item given once | `[data]` |
| `speed_tier` | a skill field (speed tiers belong to actions) | `[kit hook]` |
| `stagger_power` (poise damage) | skill `stagger` + armour `poise` modifier | shipped (`PK1`) |
| `armor_piercing` | Inoculated attacks (`channel: true`) pierce (`PK1`, CANON 2.1) | shipped (`PK1`) |
| `strain_modifier` | skill `strain` + item/facility Strain effects (`PK1` / `PK2`) | shipped (`PK1` / `PK2`) |

## 9. Gaps

1. Reach 2 melee (billhooks, the Flagellant's scourge): **Resolved (`WEAPONROSTER1` + `PR5-28`)** — `polearm` and
   `scourge` families carry `range_band: [1, 2]`, and `PR5-28` wires `ATTACK` weapon-reach lookup to `range_band.max`.
2. Cover: the brigandine's half-cover bonus and "full cover" pillars assume cover rules no doc defines. Decided
   (2026-10-25, user, Q-54): cover is only the existing cover guard; the half-cover trait and "full cover" pillars are
   dropped; cover and line of sight become engine or kit systems after testing.
3. Quenched Cinder Salt vs. Rock Salt flush: **Resolved (`PK2` / `PK3`)** — Rock Salt flushes lower dormant Strain
   floor by `-10` at the hub Slab, while `con_quenched_cinder_salt` is an in-battle Snap consumable applying `unmark` to
   stabilize a Stage V breach.
4. Line and "behind the target" areas: **Resolved (`PK1-S1`)** — `Area::Line2`, `Line3`, and `Cone3` shipped in
   `rpgkit_data::Area`.
5. Nemeth's Warded Cassock depends on whether the cell can defeat Nemeth (CANON open question). Resolved (2026-10-25,
   user, Q-25 and its follow-up): it drops where Nemeth dies, at node 20 (CANON 3.6); a cell that skips the optional
   node never gets it.
6. Missing families: **Resolved (`WEAPONROSTER1`)** — `polearm`, `greatsword`, `censer`, `bone_saw`, `scourge`, and
   `sprayer` shipped as first-class rows in `item_families.json`.
